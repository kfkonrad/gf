# Phase 1: Foundation - Research

**Researched:** 2026-03-16
**Domain:** Rust subprocess execution, signal propagation, TTY inheritance, error handling
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Install hints (CORE-06):** Two-line format, no prefix:
  ```
  glab not found
  Install with: brew install glab
  Or see: https://gitlab.com/gitlab-org/cli
  ```
  Always use `brew install <cli>` — no per-platform detection. Official URL always included.

- **Error output format:** gf's own errors go to stderr, plain text, no prefix, no ANSI styling. Child process stderr is passthrough (architecturally separate).

- **Signal handling (CORE-07):** Totally transparent — re-raise all signals (SIGINT, SIGTERM, SIGHUP, etc.) on self after child exits. Exit 130 on SIGINT. No output from gf on signal. Same behavior for all signals.

- **Subprocess execution model:**
  - Unix: `exec()` — gf replaces itself with the child process (execvp). Perfect TTY inheritance, zero signal complexity.
  - Windows: `spawn()` — gf stays alive as parent, waits for child, propagates exit code.
  - Cross-platform from day one: `#[cfg(unix)]` exec path, `#[cfg(windows)]` spawn path.

### Claude's Discretion

- Exact Rust crate choices for exec/spawn (std::os::unix::process::CommandExt vs nix, etc.)
- Error type hierarchy design (thiserror, anyhow, or custom)
- Module structure within the crate

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-06 | Print a clear error with install hint when required forge CLI is not on PATH | `which` crate for PATH detection; two-line stderr format defined in constraints |
| CORE-07 | Propagate child CLI's exit code exactly, including signal-terminated processes | `CommandExt::exec()` on Unix eliminates exit code problem entirely; `ExitStatusExt::signal()` + `nix::sys::signal::raise()` for Windows spawn path |
</phase_requirements>

---

## Summary

Phase 1 builds the subprocess runner that all later phases call. The Unix `exec()` model (gf replaces itself with the child process) satisfies CORE-07 trivially: there is no exit code to propagate because gf is no longer alive when the child exits. Signal handling for the Unix path is also trivially correct — the shell sends signals directly to the child after exec. Only the Windows `spawn()` path requires explicit exit code inspection and signal re-raise logic.

For CORE-06, the `which` crate (v8.x) is the standard Rust equivalent of the Unix `which(1)` command and handles cross-platform PATH lookup correctly. When detection fails, gf prints the two-line error format to stderr (no prefix, no ANSI) and exits with a non-zero code.

Error type design is at Claude's discretion. The recommendation is `thiserror` for defining a typed `GfError` enum (makes specific error cases like `CliNotFound` match-able in tests and future phases) paired with `anyhow` for internal propagation context. This is the standard pattern for Rust CLIs that expose structured errors to callers.

**Primary recommendation:** Use `CommandExt::exec()` on Unix (zero signal/TTY complexity), `which` for PATH detection, `thiserror` for the `GfError` type, and `nix` only for the Windows-gated signal re-raise path.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `which` | 8.0.2 | Find executable on PATH | De facto Rust equivalent of `which(1)`; cross-platform; returns `PathBuf` |
| `thiserror` | 2.x | Define typed `GfError` enum | Minimal boilerplate; derives `std::error::Error`; keeps error variants match-able |
| `nix` | 0.29+ | `raise(Signal)` for re-raise on Windows path | Safe Unix syscall bindings; only needed in `#[cfg(windows)]` spawn path |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `anyhow` | 2.x | Context propagation in `main` | Optional — if internal error chains need human-readable context |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `which` | `std::env::var("PATH")` + manual split | Hand-rolling PATH split is platform-fragile; `which` handles Windows `.exe` extension, empty PATH, symlinks |
| `thiserror` | `anyhow` only | `anyhow` hides error variants; harder to pattern-match `CliNotFound` in tests |
| `nix::raise` | `libc::raise` directly | `libc` is unsafe; `nix` provides safe wrapper with no extra cost |

**Installation:**
```bash
cargo add which thiserror
cargo add nix --features signal   # for signal re-raise on Windows path only
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── main.rs          # arg parsing, forge detection stub, calls runner::run()
├── runner.rs        # subprocess execution: exec() on Unix, spawn() on Windows
├── error.rs         # GfError enum with thiserror derives
└── forge/
    └── mod.rs       # (stub) forge detection — populated in Phase 2
```

This is a single-binary crate. No workspace needed at this stage. Phase 2 will add forge detection; Phase 3 will add command translation. The `runner` module is the deliverable of Phase 1.

### Pattern 1: Unix exec() — Replace Self

**What:** `CommandExt::exec()` replaces the current process with the child. gf ceases to exist; the forge CLI becomes the process. TTY, signals, exit code — all inherited automatically.

**When to use:** Always on Unix (the `#[cfg(unix)]` branch).

```rust
// Source: https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html
#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(unix)]
pub fn exec_child(program: &str, args: &[&str]) -> GfError {
    // exec() only returns on failure
    let err = std::process::Command::new(program)
        .args(args)
        .exec();
    // If we reach here, exec failed (program not found after PATH check, etc.)
    GfError::ExecFailed(err)
}
```

Note: call `which(program)` before this to give a friendly CORE-06 error. `exec()` itself returns an `io::Error` on failure, which is less user-friendly.

### Pattern 2: Windows spawn() — Wait and Propagate

**What:** Spawn a child process, wait for it, inspect `ExitStatus`. If it exited normally, propagate the code. If terminated by signal, re-raise that signal on self.

**When to use:** The `#[cfg(windows)]` branch only.

```rust
// Source: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
//         https://docs.rs/nix/latest/nix/sys/signal/fn.raise.html
#[cfg(windows)]
pub fn spawn_child(program: &str, args: &[&str]) -> Result<(), GfError> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .map_err(GfError::SpawnFailed)?;

    match status.code() {
        Some(code) => std::process::exit(code),
        None => {
            // Signal-terminated (Unix ExitStatusExt; Windows won't hit this)
            // Re-raise to let shell see correct exit code
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = status.signal() {
                    nix::sys::signal::raise(
                        nix::sys::signal::Signal::try_from(sig).unwrap()
                    ).ok();
                }
            }
            std::process::exit(1);
        }
    }
}
```

### Pattern 3: CORE-06 Error Format

**What:** Check PATH before attempting exec. On miss, print two-line message to stderr and exit non-zero.

```rust
// Locked format from CONTEXT.md
fn cli_not_found_message(cli: &str, brew_name: &str, url: &str) -> String {
    format!("{cli} not found\nInstall with: brew install {brew_name}\nOr see: {url}")
}

// In main or runner:
if which::which(cli_name).is_err() {
    eprintln!("{}", cli_not_found_message(cli_name, brew_name, url));
    std::process::exit(1);
}
```

### Anti-Patterns to Avoid

- **Don't use `spawn()` on Unix when `exec()` is available.** Spawn leaves gf alive as a parent process, creating signal indirection complexity (Ctrl+C sends SIGINT to the process group, but job control, shell interaction, and TTY forwarding all get complicated). exec() sidesteps all of it.
- **Don't check `which` and then rely on `exec()` error for the user message.** The `exec()` error is a low-level `io::Error`. Check `which` first to give the CORE-06 human-readable message.
- **Don't use `process::exit(130)` as a hardcoded value.** Re-raise the signal (`raise(SIGINT)`) so the shell receives the actual signal and computes exit 130 itself. Hardcoding the number skips the signal entirely.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PATH lookup | Manual `$PATH` split + file existence check | `which` crate | Handles Windows `.exe`, empty PATH, symlinks, permission bits correctly |
| Error boilerplate | `impl fmt::Display for GfError` manually | `thiserror` | Derives Display, Error, and From automatically |
| Signal number to name | Integer constants | `nix::sys::signal::Signal` enum | Type-safe; `try_from(i32)` for conversion |

---

## Common Pitfalls

### Pitfall 1: exec() Called After Destructors Would Matter

**What goes wrong:** Calling `exec()` after acquiring resources that need cleanup (open files, lock guards) silently skips those destructors.
**Why it happens:** exec() does not run Rust destructors — it's equivalent to `process::exit()` from the runtime's perspective.
**How to avoid:** Call `exec()` as early as possible in `main`, after all argument parsing but before any resource acquisition. In practice, `gf` acquires almost nothing before exec, so this is low risk.
**Warning signs:** Any `Drop` impl that does meaningful work (e.g., writing to a file, releasing a lock) acquired before the exec call.

### Pitfall 2: which() Succeeds But exec() Fails

**What goes wrong:** The binary exists on PATH but is not executable (permission denied), or the filesystem is read-only. `which` finds it, exec fails, and gf gets an `io::Error` with no friendly message.
**How to avoid:** Wrap the `exec()` error in a `GfError::ExecFailed` variant. Surface it with enough context for debugging, but accept that this is an unusual case — the CORE-06 path covers the common "not installed" case.

### Pitfall 3: Signal Re-Raise on Windows Spawn Path Skipped

**What goes wrong:** On Windows, `ExitStatus::code()` returns `None` for abnormal termination. Without handling the `None` case, gf exits with code 1 instead of the correct signal-derived code.
**Why it happens:** Windows doesn't have POSIX signals; `ExitStatusExt::signal()` is a Unix-only trait. On Windows, `code()` is almost always `Some(...)`.
**How to avoid:** The `None` branch in the Windows spawn path should still `exit(1)` as a safe fallback, with a comment explaining why.

### Pitfall 4: stderr vs stdout Confusion for gf Errors

**What goes wrong:** Printing gf's own errors to stdout instead of stderr. This corrupts stdout output (e.g., if a caller captures gf's stdout).
**How to avoid:** Always use `eprintln!()` for gf's own error messages. Child process stderr flows through without interception.

---

## Code Examples

### PATH Detection (CORE-06)

```rust
// Source: https://docs.rs/which/8.0.2/which/
use which::which;

pub fn find_cli(name: &str) -> Option<std::path::PathBuf> {
    which(name).ok()
}
```

### GfError Type

```rust
// Source: https://docs.rs/thiserror/2.x/thiserror/
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GfError {
    #[error("{cli} not found\nInstall with: brew install {brew_name}\nOr see: {url}")]
    CliNotFound {
        cli: String,
        brew_name: String,
        url: String,
    },

    #[error("failed to exec {0}: {1}")]
    ExecFailed(String, std::io::Error),

    #[error("failed to spawn {0}: {1}")]
    SpawnFailed(String, std::io::Error),
}
```

### Signal Re-Raise (Unix, for completeness in spawn path)

```rust
// Source: https://docs.rs/nix/latest/nix/sys/signal/fn.raise.html
#[cfg(unix)]
fn reraise_signal(sig: i32) {
    use nix::sys::signal::{raise, Signal};
    if let Ok(signal) = Signal::try_from(sig) {
        let _ = raise(signal);
    }
    // If raise fails or signal unknown, fall through to exit(1)
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual `io::Error` matching | `thiserror` 2.x derive macros | thiserror 2.0 released 2024 | Cleaner derive syntax, no breaking change |
| `which` 4.x API | `which` 8.x (same core API) | 2023-2024 | API stable; `which::which()` unchanged |

**Deprecated/outdated:**
- `assert_cli`: Superseded by `assert_cmd` + `predicates` for integration testing.

---

## Open Questions

1. **nix feature flags for target platforms**
   - What we know: `nix` requires `--features signal` to expose `raise()` and `Signal`
   - What's unclear: Whether `nix` is needed at all on the Unix exec path (it isn't — exec() means no re-raise needed on Unix)
   - Recommendation: Add `nix` to `Cargo.toml` only if building the Windows spawn path in Phase 1. If Windows support is deferred, `nix` is not needed in Phase 1 at all.

2. **Forge-to-install-hint mapping**
   - What we know: The four CLIs are `gh`, `glab`, `tea`, `fj`; brew names and URLs are well-known
   - What's unclear: `fj` (Forgejo CLI) brew tap name — may require a tap, not just `brew install fj`
   - Recommendation: Define the mapping in a static table; flag `fj` brew name for verification in Phase 2/3 when Forgejo support is tested.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + `assert_cmd` 2.x + `predicates` 3.x |
| Config file | None — `cargo test` discovers tests automatically |
| Quick run command | `cargo test` |
| Full suite command | `cargo test -- --include-ignored` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-06 | `gf <cmd>` when CLI not on PATH prints two-line error to stderr, exits non-zero | integration | `cargo test test_cli_not_found` | Wave 0 |
| CORE-06 | Error message matches exact two-line format (no prefix, no ANSI) | integration | `cargo test test_cli_not_found_format` | Wave 0 |
| CORE-07 | `gf <cmd>` exits with same code as child (e.g. code 2) | integration | `cargo test test_exit_code_propagation` | Wave 0 |
| CORE-07 | `gf <cmd>` with Ctrl+C causes exit 130 (signal re-raise) | manual | manual — requires TTY signal delivery | manual-only |
| CORE-07 | TTY inherited — child color output works | manual | manual — requires real TTY | manual-only |

Note: TTY inheritance and interactive signal delivery cannot be automated in CI without a PTY harness (e.g., `rexpect`). These are manual verification items for the phase gate.

### Sampling Rate

- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test -- --include-ignored`
- **Phase gate:** Full suite green + manual TTY/signal checks before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/integration_test.rs` — covers CORE-06 (CliNotFound error format) and CORE-07 (exit code propagation)
- [ ] `tests/fixtures/exit_with.sh` or a small helper binary — needed for exit code propagation tests
- [ ] `Cargo.toml` dev-dependencies: `assert_cmd = "2"` and `predicates = "3"`

---

## Sources

### Primary (HIGH confidence)
- [std::os::unix::process::CommandExt](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html) — exec() behavior, signature, TTY inheritance
- [std::process::ExitStatus](https://doc.rust-lang.org/std/process/struct.ExitStatus.html) — code(), signal() (ExitStatusExt), None on signal termination
- [nix::sys::signal::raise](https://docs.rs/nix/latest/nix/sys/signal/fn.raise.html) — raise(Signal) signature and behavior
- [nix::sys::wait::WaitStatus](https://docs.rs/nix/latest/nix/sys/wait/enum.WaitStatus.html) — Exited and Signaled variants
- [which crate docs.rs](https://docs.rs/which/) — v8.0.2 API, which() function signature

### Secondary (MEDIUM confidence)
- [assert_cmd crates.io](https://crates.io/crates/assert_cmd) — integration testing for CLI binaries; verified as standard by Rust CLI book
- [thiserror 2.x recommendation](https://leapcell.io/blog/choosing-the-right-rust-error-handling-tool) — thiserror for libraries/error types, anyhow for applications; multiple corroborating sources
- [Rust CLI book — signals](https://rust-cli.github.io/book/in-depth/signals.html) — signal handling guidance

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — which v8.x, thiserror v2.x, nix v0.29+ all verified via docs.rs
- Architecture: HIGH — exec() model verified against official std docs; Windows spawn path well-understood
- Pitfalls: MEDIUM — exec() destructor warning from official docs; others from reasoning about the design

**Research date:** 2026-03-16
**Valid until:** 2026-09-16 (stable APIs; which/thiserror/nix are slow-moving)
