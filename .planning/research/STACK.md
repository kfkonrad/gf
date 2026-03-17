# Stack Research

**Domain:** Rust CLI wrapper/router tool (forge CLI multiplexer)
**Researched:** 2026-03-17 (updated for v1.1)
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

## v1.1 Stack Additions

**Summary: No new mandatory dependencies.** Every v1.1 feature is implementable with the existing Cargo.toml. One optional crate (`serde_json`) may be added only if the tea CLI auth-probe approach requires it.

### PR list / merge / checkout / review

New verbs in `src/adapter/pr.rs` inside the existing `match matches.subcommand()` dispatch. New clap subcommand shapes declared in `src/cmd/`. No new crates. Same pattern as `create` and `view`.

### Repo clone

New arm in the repo adapter (equivalent of `src/adapter/repo.rs`). Maps `gf repo clone <url>` to `gh repo clone <url>`, `glab repo clone <url>`, `tea repos clone <url>`, `fj repo clone <url>`. No new crates.

### Issues commands

New module `src/adapter/issue.rs`. Top-level `gf issue` command (with `issues` alias) dispatching to the forge-specific issue subcommand. No new crates — identical structure to the PR adapter.

### Line-range browse (file.rs:42-55)

Extend `src/browse/mod.rs` URL builder to accept optional `(start_line, Option<end_line>)`. Parse with `str::split(':')` and `str::split('-')` — two stdlib calls.

Forge fragment format (verified against public forge URL behavior):

| Forge | Fragment format | Notes |
|-------|----------------|-------|
| GitHub | `#L42-L55` | Both line numbers prefixed with `L` |
| GitLab | `#L42-55` | First line prefixed, second bare |
| Gitea | `#L42` | Range anchor not supported in URL; use start line only |
| Forgejo | `#L42-L55` | Same as GitHub |

No regex crate needed. Input is `split(':').last()` on the file argument then `split('-')` for range. Two stdlib calls.

### Self-hosted forge detection via CLI auth probing (CORE-04)

Probe installed forge CLIs using exit codes from `std::process::Command::output()` — already used throughout the codebase. No new crates.

Probe strategy per forge CLI:

| CLI | Probe command | Success signal | Notes |
|-----|--------------|---------------|-------|
| `gh` | `gh auth status --hostname <host>` | exit 0 | `--hostname` flag confirmed stable in official docs |
| `glab` | `glab auth status --hostname <host>` | exit 0 | `--hostname` flag confirmed in official docs |
| `tea` | `tea logins list --output simple` | stdout contains host URL | Parse line-by-line with `str::contains`; no JSON needed |
| `fj` | Not probeable | N/A | No `auth status` equivalent in fj; config-file-only for Forgejo self-hosted |

Implementation uses only `std::process::Command::output()` and `str::contains`. `serde_json` is NOT needed if the `tea logins list --output simple` approach is used (tabular text, URL is in the first column).

### Flag mapping audit

Pure implementation work: review gh/glab/tea/fj man pages for `pr list`, `pr merge`, `pr checkout`, `pr review`, `issue list`, `issue view`, `issue create`, `repo clone` and update translation tables in `src/adapter/`. No new crates.

## Conditionally Add (v1.1)

| Library | Version | Purpose | Add When |
|---------|---------|---------|----------|
| serde_json | 1 | Parse `tea logins list --output json` for CORE-04 | Only if simple-format text parse proves too fragile. `tea logins list --output simple` is the preferred probe and avoids this dep. |

`serde_json 1` is compatible with `serde 1` already in Cargo.toml. Zero integration risk if added. Avoid adding speculatively.

## Installation

No Cargo.toml changes required for baseline v1.1.

If tea JSON parsing is needed for CORE-04 (defer until proven necessary):

```toml
# Cargo.toml [dependencies]
serde_json = "1"
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
| `str::split` for line-range parse | `regex` crate | `file.rs:42-55` has fixed structure — two `split` calls suffice. Regex adds compile time with zero benefit for structured input. |
| Exit-code probe for CORE-04 | `gh auth status --json hosts` JSON parse | `--json hosts` schema is undocumented/unstable (confirmed via GitHub CLI issue tracker). Exit-code on `--hostname` is documented, stable, and requires zero parsing. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| structopt | Merged into clap 3+, no longer maintained as separate crate | clap 4 with `features = ["derive"]` |
| argh | Missing `trailing_var_arg` and external subcommand support — both essential for CLI passthrough | clap 4 |
| tokio / async runtime | No concurrent I/O. An async runtime adds ~500KB to binary and startup latency for zero benefit. `gf` is synchronous by nature. | std::process::Command (sync) |
| git2 | Pulls in libgit2 as a C native dependency, complicating cross-compilation and adding ~3MB to binary. Only needed operation is reading a remote URL. | `git remote get-url origin` via subprocess |
| indicatif (progress bars) | `gf` is a transparent router — it should show exactly what the underlying CLI shows, nothing more. Progress bars would conflict with subprocess output. | Inherit subprocess stdio directly |
| `regex` crate | Overkill for parsing structured CLI output or line-range arguments; adds compile-time cost for no benefit over stdlib string ops | `str::split`, `str::contains`, `str::starts_with` |
| `reqwest` / HTTP clients | v1.x explicitly prohibits direct forge API calls | Delegate to gh/glab/tea/fj CLIs |
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
- Handle SSH (`git@host:org/repo`) and HTTPS (`https://host/org/repo`) formats with simple string matching — no URL parsing crate needed at this scope

**For CORE-04 self-hosted probing:**
- Run `gh auth status --hostname <host>` via `Command::output()`, check `status.success()`
- Run `glab auth status --hostname <host>` via `Command::output()`, check `status.success()`
- Run `tea logins list --output simple` via `Command::output()`, scan stdout lines for host substring
- For fj: skip probe — require config file entry

**For line-range browse:**
- Accept `<file>[:<start>[-<end>]]` as the browse path argument
- `let parts: Vec<&str> = arg.splitn(2, ':').collect()` — file is parts[0], optional range is parts[1]
- `let lines: Vec<&str> = range.splitn(2, '-').collect()` — start is lines[0], optional end is lines[1]
- Append forge-specific fragment to constructed URL

**For "CLI not installed" error path:**
- `which::which("gh")` returns `Err` if not on PATH
- Map to a `thiserror` variant with a formatted install hint message
- Surface via `anyhow::Error` with `.context()` for UX-friendly output

**For `forge browse`:**
- Detect forge from remote URL (same detection as other commands)
- Construct browse URL from remote host + org + repo + optional branch/file path + optional line fragment
- Call `webbrowser::open(&url)` — non-blocking, returns immediately

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| clap 4.6 | thiserror 2.x, anyhow 1.x | No interaction; independent libraries |
| thiserror 2.x | anyhow 1.x | anyhow can wrap any `std::error::Error`, including thiserror types |
| webbrowser 1.2 | Rust 1.75+ | Check MSRV if targeting older toolchains |
| which 8.x | Rust 1.70+ | which 8 is a semver-major bump from 7; pin to `"8"` not `"*"` |
| serde_json 1 | serde 1 (already in Cargo.toml) | Compatible; same major version family |

## Sources

- Cargo.toml in repo (v1.0) — current dependency versions (HIGH confidence)
- [gh auth status docs](https://cli.github.com/manual/gh_auth_status) — `--hostname` flag confirmed stable; `--json hosts` schema undocumented (HIGH confidence for exit-code approach)
- [glab auth status docs](https://docs.gitlab.com/cli/auth/status/) — `--hostname` flag confirmed (HIGH confidence)
- [tea CLI docs / CLI.md](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md) — `tea logins list --output simple|json` confirmed (HIGH confidence)
- [forgejo-cli](https://codeberg.org/Cyborus/forgejo-cli) — no auth status subcommand found; config-file-only for fj self-hosted (MEDIUM confidence — absence of evidence, not evidence of absence)
- [GitHub CLI issue #9326](https://github.com/cli/cli/issues/9326) — confirmed `--json hosts` schema is unstable/undocumented (HIGH confidence in avoiding this approach)
- Source code review of v1.0 codebase — `std::process::Command::output()` already used; no pattern gaps for new features (HIGH confidence)

---
*Stack research for: gf v1.1 — PR workflows, issues, clone, line-range browse, self-hosted detection, flag audit*
*Researched: 2026-03-17*
