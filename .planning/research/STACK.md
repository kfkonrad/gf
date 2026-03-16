# Stack Research

**Domain:** Rust CLI wrapper/router tool (forge CLI multiplexer)
**Researched:** 2026-03-16
**Confidence:** HIGH

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (stable) | 1.85+ | Implementation language | Single-binary distribution, zero-cost abstractions, strong type system catches routing bugs at compile time. Already project-decided. |
| clap | 4.6 | Argument parsing and subcommand dispatch | De-facto standard for Rust CLIs. Derive API reduces boilerplate. `allow_external_subcommands` and `trailing_var_arg` are the exact features needed for passthrough arg capture. No serious competitor. |
| anyhow | 1.0.100 | Application-level error handling | Ergonomic `?`-propagation with context chaining (`with_context`). Best fit for a CLI binary (not a library) where you need human-readable error messages without defining exhaustive error enums. |
| thiserror | 2.0.18 | Structured error types for domain errors | Use alongside anyhow for the forge-detection and routing layer where you need to match on error variants (e.g., `ForgeNotDetected`, `CliNotInstalled`). Derive macro, zero runtime overhead. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| which | 8.0.2 | Locate forge CLIs on PATH | Use to check whether `gh`, `glab`, `tea`, `fj` are installed before attempting to exec them. Provides clean error path for "not installed" case with install hint. |
| webbrowser | 1.2.0 | Open URLs in the default browser | Use for `forge browse`. Supports macOS, Windows, Linux/WSL, *BSD. Non-blocking for GUI browsers. Preferred over the `open` crate because it has an explicit "browser guarantee" — it won't just open a file manager. |
| std::process::Command | stdlib | Subprocess execution and arg passthrough | No external crate needed. `Command::new("gh").args(&args).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status()` gives full transparent passthrough including interactive prompts, colors, and TTY detection. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo | Build, test, dependency management | Standard. Use `cargo add` for dependency management. |
| clippy | Linting | `cargo clippy -- -D warnings` in CI to catch common mistakes. |
| rustfmt | Formatting | Enforce with `cargo fmt --check` in CI. |
| cargo-nextest | Fast test runner | Optional but recommended. `cargo nextest run` is significantly faster than `cargo test` for test suites with many unit tests. |
| cargo-dist | Release binary packaging | Generates GitHub Actions release workflows, cross-compilation, and installer scripts. Standard for OSS Rust CLIs distributed via `cargo install` or direct download. |

## Installation

```toml
# Cargo.toml

[dependencies]
clap = { version = "4.6", features = ["derive"] }
anyhow = "1.0"
thiserror = "2.0"
which = "8.0"
webbrowser = "1.2"

[dev-dependencies]
# No special test dependencies required for this project scope
```

```bash
# Add deps via cargo
cargo add clap --features derive
cargo add anyhow thiserror which webbrowser
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| clap 4 derive API | clap 4 builder API | Use builder API when you need to construct commands dynamically at runtime (not applicable here — all subcommands are static). |
| clap | argh | argh is smaller/faster to compile but has very limited features. Not suitable: lacks `trailing_var_arg` and external subcommand support, both essential for arg passthrough. |
| clap | structopt | Obsolete. structopt was merged into clap 3+. Never use structopt for new projects. |
| anyhow | miette | miette adds rich diagnostic output (source spans, labels). Overkill for a thin CLI router where errors are simple strings. Add later if users want prettier errors. |
| webbrowser | open | Both work cross-platform. `open` is more general (opens files, URLs, apps). `webbrowser` has an explicit browser guarantee. For `forge browse`, browser guarantee matters — don't risk opening a file manager. |
| std::process::Command | tokio::process::Command | Async subprocess only needed if running multiple forge CLIs concurrently. `gf` always delegates to exactly one CLI, so async adds complexity with no benefit. |
| which crate | std::env checking manually | `which` handles Windows extension (.exe) resolution and edge cases correctly. Don't re-implement. |
| git remote URL via subprocess (`git remote get-url origin`) | git2 / gix crates | For this project, running `git remote get-url origin` via `std::process::Command` is sufficient. git2 adds ~3MB to binary size and a native libgit2 dep. gix (0.80.0) is pure Rust but still heavy. The only operation needed is reading one remote URL — a subprocess call is appropriate at this scope. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| structopt | Merged into clap 3+, no longer maintained as separate crate | clap 4 with `features = ["derive"]` |
| argh | Missing `trailing_var_arg` and external subcommand support — both essential for CLI passthrough | clap 4 |
| tokio / async runtime | No concurrent I/O. An async runtime adds ~500KB to binary and startup latency for zero benefit. `gf` is synchronous by nature. | std::process::Command (sync) |
| git2 | Pulls in libgit2 as a C native dependency, complicating cross-compilation and adding ~3MB to binary. Only needed operation is reading a remote URL. | `git remote get-url origin` via subprocess |
| indicatif (progress bars) | `gf` is a transparent router — it should show exactly what the underlying CLI shows, nothing more. Progress bars would conflict with subprocess output. | Inherit subprocess stdio directly |
| serde / config files | The PROJECT.md explicitly rules out own config file. No JSON/TOML parsing needed in v1. | No config layer |
| clap's `external_subcommand` for the full dispatch | `allow_external_subcommands` captures unknown subcommand names but doesn't preserve flag normalization. Build explicit subcommand enum with `trailing_var_arg` for known commands, then translate flags before passing through. | Explicit clap subcommand + trailing args + manual flag mapping |

## Stack Patterns by Variant

**For subprocess passthrough (known subcommand, translated flags):**
- Parse with clap derive: `#[arg(trailing_var_arg = true)] extra: Vec<String>`
- Normalize known flags in the routing layer before building the subprocess args
- Use `std::process::Command` with `Stdio::inherit()` for stdin/stdout/stderr
- Propagate exit code: `std::process::exit(status.code().unwrap_or(1))`

**For forge detection:**
- Run `git remote get-url origin` (or `--remote` override) via `Command::output()` (captured, not inherited)
- Parse the URL string to identify host: github.com → gh, gitlab.com → glab, etc.
- Handle SSH (`git@host:org/repo`) and HTTPS (`https://host/org/repo`) formats with simple regex or string matching — no URL parsing crate needed at this scope

**For "CLI not installed" error path:**
- `which::which("gh")` returns `Err` if not on PATH
- Map to a `thiserror` variant with a formatted install hint message
- Surface via `anyhow::Error` with `.context()` for UX-friendly output

**For `forge browse`:**
- Detect forge from remote URL (same detection as other commands)
- Construct browse URL from remote host + org + repo + optional branch/file path
- Call `webbrowser::open(&url)` — non-blocking, returns immediately

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| clap 4.6 | thiserror 2.x, anyhow 1.x | No interaction; independent libraries |
| thiserror 2.x | anyhow 1.x | anyhow can wrap any `std::error::Error`, including thiserror types |
| webbrowser 1.2 | Rust 1.75+ | Check MSRV if targeting older toolchains |
| which 8.x | Rust 1.70+ | which 8 is a semver-major bump from 7; pin to `"8"` not `"*"` |

## Sources

- [crates.io/crates/clap](https://crates.io/crates/clap) — version confirmed as 4.5.x/4.6 (HIGH confidence)
- [crates.io/crates/which](https://crates.io/crates/which) via docs.rs — version 8.0.2 (HIGH confidence)
- [docs.rs/webbrowser/latest](https://docs.rs/webbrowser/latest/webbrowser/) — version 1.2.0, platform support confirmed (HIGH confidence)
- [docs.rs/crate/thiserror/latest](https://docs.rs/crate/thiserror/latest) — version 2.0.18 (HIGH confidence)
- [docs.rs/crate/anyhow/latest](https://docs.rs/crate/anyhow/latest) — version 1.0.100 (HIGH confidence)
- [doc.rust-lang.org/std/process/struct.Command.html](https://doc.rust-lang.org/std/process/struct.Command.html) — Stdio::inherit() passthrough pattern (HIGH confidence)
- WebSearch: clap `trailing_var_arg` and `allow_external_subcommands` patterns — confirmed via clap docs.rs and Rust forum discussions (HIGH confidence)
- WebSearch: git2 vs subprocess for remote URL reading — subprocess approach adequate for single-read use case (MEDIUM confidence — pragmatic call, not a hard technical constraint)

---
*Stack research for: Rust CLI wrapper/router (gf — git forge)*
*Researched: 2026-03-16*
