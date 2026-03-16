# Architecture Research

**Domain:** Rust CLI forge wrapper / command router
**Researched:** 2026-03-16
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Entry Point (main.rs)                     │
│               clap parse → Cli struct → dispatch                 │
├─────────────────────────────────────────────────────────────────┤
│                        Command Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │  pr subcommand│  │ repo subcmd  │  │  browse subcommand │    │
│  └──────┬───────┘  └──────┬───────┘  └────────┬───────────┘    │
│         │                 │                   │                 │
├─────────┴─────────────────┴───────────────────┴─────────────────┤
│                        Core Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ ForgeDetector│  │ CommandRouter│  │    UrlBuilder      │    │
│  └──────┬───────┘  └──────┬───────┘  └────────┬───────────┘    │
│         │                 │                   │                 │
├─────────┴─────────────────┴───────────────────┴─────────────────┤
│                        Forge Adapters                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
│  │  GitHub  │  │  GitLab  │  │  Gitea   │  │   Forgejo      │  │
│  │  (gh)    │  │  (glab)  │  │  (tea)   │  │   (fj)         │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      Infrastructure Layer                        │
│  ┌──────────────────────┐  ┌──────────────────────────────────┐ │
│  │   Git Remote Reader  │  │      Subprocess Runner           │ │
│  │  (.git/config parse) │  │  (std::process::Command)         │ │
│  └──────────────────────┘  └──────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Boundary |
|-----------|----------------|----------|
| `ForgeDetector` | Read git remote URL, classify forge type, extract host/owner/repo | Input: remote URL string. Output: `ForgeContext` struct |
| `CommandRouter` | Map canonical subcommand + flags to forge-specific CLI argv | Input: `ForgeType` + canonical args. Output: `Vec<OsString>` |
| `FlagNormalizer` | Translate known canonical flags to forge equivalents; pass through unknown flags unchanged | Input: canonical flags + forge type. Output: translated flag list |
| `UrlBuilder` | Construct web URLs for browse command without delegating to underlying CLI | Input: `ForgeContext` + path/branch. Output: URL string |
| `SubprocessRunner` | Exec the underlying CLI binary, inheriting stdin/stdout/stderr | Input: binary name + argv. Output: exit code (process replaces self via exec on Unix) |
| `GitRemoteReader` | Shell out to `git remote get-url <remote>` or parse `.git/config` | Input: remote name (default: origin). Output: raw URL string |
| Forge Adapters | Per-forge command and flag mapping tables | Static data: command maps, flag maps, binary name |

## Recommended Project Structure

```
src/
├── main.rs                 # Entry: parse CLI args, run, propagate exit code
├── cli.rs                  # clap Cli struct and subcommand enums
├── error.rs                # AppError type (thiserror)
├── forge/
│   ├── mod.rs              # ForgeType enum, ForgeContext struct, pub re-exports
│   ├── detector.rs         # ForgeDetector: remote URL → ForgeType + host/owner/repo
│   ├── url_builder.rs      # Forge-specific web URL construction
│   └── adapters/
│       ├── mod.rs          # ForgeAdapter trait definition
│       ├── github.rs       # gh command/flag maps
│       ├── gitlab.rs       # glab command/flag maps
│       ├── gitea.rs        # tea command/flag maps
│       └── forgejo.rs      # fj command/flag maps
├── router/
│   ├── mod.rs              # CommandRouter: dispatch canonical cmd → adapter argv
│   └── flags.rs            # FlagNormalizer: canonical flag → forge flag translation
└── runner/
    └── mod.rs              # SubprocessRunner: exec binary with args
```

### Structure Rationale

- **`forge/`:** All forge-awareness lives here. The detector and adapters form a closed set; adding a new forge means adding a new adapter file and a new `ForgeType` variant — nothing else changes.
- **`router/`:** Separated from forge adapters because routing logic (argument reordering, subcommand aliasing) is distinct from per-forge flag translation.
- **`runner/`:** Isolated so it can be swapped for a test double in unit tests and so exec semantics (Unix `execvp` vs Windows `spawn`) stay in one place.

## Architectural Patterns

### Pattern 1: ForgeAdapter Trait

**What:** Each forge backend implements a shared trait that maps canonical commands and flags to forge-specific equivalents.
**When to use:** Required. Provides the extensibility point for adding new forges without touching routing logic.
**Trade-offs:** Static dispatch (enum match) is simpler for a fixed set of forges; a trait gives better modularity. Because the forge set is closed and small (4 forges), an enum dispatch with a `fn adapter(&self) -> &dyn ForgeAdapter` method is the right balance.

**Example:**
```rust
pub trait ForgeAdapter {
    fn binary_name(&self) -> &'static str;
    fn map_subcommand(&self, canonical: &CanonicalCommand) -> Option<Vec<&'static str>>;
    fn map_flag(&self, canonical: &str) -> Option<&'static str>;
}

pub enum ForgeType {
    GitHub,
    GitLab,
    Gitea { host: String },
    Forgejo { host: String },
}
```

### Pattern 2: Transparent Passthrough for Unknown Flags

**What:** The flag normalizer only translates flags that appear in its known map; everything else is passed through unchanged to the underlying CLI.
**When to use:** Always — this is what makes `gf` a thin wrapper rather than a full abstraction layer.
**Trade-offs:** Users get the escape hatch they need for forge-specific flags. The risk is silent breakage if a canonical flag name collides with a forge-specific flag that means something different — document this in PITFALLS.

**Example:**
```rust
fn normalize_flags(flags: &[String], adapter: &dyn ForgeAdapter) -> Vec<String> {
    flags.iter().flat_map(|f| {
        match adapter.map_flag(f) {
            Some(translated) => vec![translated.to_string()],
            None => vec![f.clone()],
        }
    }).collect()
}
```

### Pattern 3: exec() Replacement (Unix) / spawn-and-wait (Windows)

**What:** On Unix, replace the `gf` process with the underlying CLI using `exec` so that signals, TTY, and exit codes are all inherited transparently. On Windows, spawn and wait.
**When to use:** For all delegated commands. Do not capture stdout/stderr — the underlying CLI should own the terminal directly.
**Trade-offs:** `exec` means no cleanup code runs after delegation. This is correct behavior — `gf` is a transparent router, not a wrapper that post-processes output.

**Example:**
```rust
use std::os::unix::process::CommandExt;

pub fn exec_forge(binary: &str, args: &[OsString]) -> Result<(), AppError> {
    let err = std::process::Command::new(binary)
        .args(args)
        .exec();          // only returns if exec fails (binary not found)
    Err(AppError::ExecFailed { binary: binary.to_string(), source: err })
}
```

## Data Flow

### Command Execution Flow

```
User runs: gf pr create --title "Fix bug" --draft

    ↓
[cli.rs] clap parses → Cli { subcommand: Pr(PrCmd::Create { title: "Fix bug", draft: true, extra: [] }) }

    ↓
[forge/detector.rs] git remote get-url origin
    → "git@github.com:owner/repo.git"
    → ForgeContext { forge: ForgeType::GitHub, host: "github.com", owner: "owner", repo: "repo" }

    ↓
[forge/adapters/github.rs] map_subcommand(PrCreate)
    → ["pr", "create"]

[router/flags.rs] normalize flags for GitHub adapter
    → --title "Fix bug" (unchanged), --draft (unchanged, gh uses --draft)

    ↓
[runner/mod.rs] exec("gh", ["pr", "create", "--title", "Fix bug", "--draft"])
    → process is replaced; gh owns the terminal
```

### Browse Flow (native, no delegation)

```
User runs: gf browse src/main.rs

    ↓
[cli.rs] clap parses → Cli { subcommand: Browse { path: Some("src/main.rs"), branch: None } }

    ↓
[forge/detector.rs] → ForgeContext { forge: GitHub, host: "github.com", owner, repo }

    ↓ (no adapter needed — URL construction is forge-specific but native)
[forge/url_builder.rs] current_branch() from git
    → branch = "main"

    ↓
[forge/url_builder.rs] build_url(ForgeContext, path="src/main.rs", branch="main")
    → "https://github.com/owner/repo/blob/main/src/main.rs"

    ↓
open::that(url)  // opens in default browser
```

## Build Order

The components have strict dependency ordering. Build in this sequence:

1. **`error.rs`** — All other components return `AppError`; needed first.
2. **`runner/`** — No dependencies on forge logic; can be built and tested in isolation.
3. **`forge/detector.rs`** + **`GitRemoteReader`** — Core detection logic; everything downstream depends on `ForgeContext`.
4. **`forge/adapters/`** — Implement the `ForgeAdapter` trait per forge. Start with GitHub (most familiar), then GitLab, then Gitea/Forgejo.
5. **`router/`** — Depends on adapters. Build after at least one adapter is working.
6. **`forge/url_builder.rs`** — Independent of adapters; depends only on `ForgeContext`. Can be built in parallel with router.
7. **`cli.rs`** + **`main.rs`** — Wire everything together last.

This order enables integration-testing each component before the full pipeline exists.

## Forgejo vs Gitea Detection

This is the primary ambiguity in forge detection because both platforms use the same self-hosted model with arbitrary domain names.

**Detection strategy (ordered by reliability):**

| Signal | Reliability | Notes |
|--------|-------------|-------|
| Known public host (`codeberg.org`) | HIGH | Codeberg is the largest Forgejo instance; hard-code it as Forgejo |
| Known public host (`gitea.com`) | HIGH | gitea.com is Gitea; hard-code it |
| API endpoint probe (`/api/forgejo/v1/version`) | MEDIUM | Forgejo exposes this; Gitea does not. Requires network call — out of scope for v1 |
| User config override (`~/.config/gf/forges.toml`) | HIGH | Escape hatch for private instances |
| Default fallback | — | Unknown self-hosted hosts default to Gitea (conservative; `tea` works for both Forgejo and Gitea since Forgejo is API-compatible) |

**v1 recommendation:** Hard-code `codeberg.org` → Forgejo. Hard-code `gitea.com` → Gitea. For all other unknown hosts, default to Gitea (`tea`) with a `--forge` flag override. Add a config file for persistent host→forge mappings. Defer API probing to v2.

**Key fact:** Forgejo is API-compatible with Gitea. `tea` will work against a Forgejo instance. So the `tea`/`fj` choice is mostly about UX — `fj` may have Forgejo-specific features, but `tea` will function. This makes the default-to-Gitea fallback safe.

## Anti-Patterns

### Anti-Pattern 1: Capturing subprocess stdout

**What people do:** Pipe subprocess stdout through `gf` to post-process or format output.
**Why it's wrong:** Breaks TTY detection in `gh`/`glab` (they disable color when not attached to a TTY), breaks interactive prompts, and adds latency.
**Do this instead:** Use `exec()` on Unix. On Windows, use `spawn().wait()` with inherited stdio — never capture.

### Anti-Pattern 2: Implementing forge API calls in gf

**What people do:** Call the GitHub/GitLab REST API directly to avoid shelling out.
**Why it's wrong:** Duplicates auth management, token storage, and API client maintenance. The whole value proposition is delegation.
**Do this instead:** Shell out to the existing CLI. If a CLI is missing a feature, file an issue upstream or add it to v2 scope.

### Anti-Pattern 3: One giant match in main

**What people do:** Single `match args.subcommand` with all forge logic inline.
**Why it's wrong:** Untestable, unmaintainable when adding a 5th forge.
**Do this instead:** The ForgeAdapter trait + per-forge adapter modules with static command/flag maps.

### Anti-Pattern 4: Assuming SSH and HTTPS remote URLs parse identically

**What people do:** Regex for `github.com` assuming HTTPS format.
**Why it's wrong:** SSH remotes look like `git@github.com:owner/repo.git`. Both formats must be handled.
**Do this instead:** Use the `git-url-parse` crate (Rust) or write a parser covering both formats explicitly. Test both.

## Integration Points

### External Processes

| Process | Integration Pattern | Notes |
|---------|---------------------|-------|
| `git remote get-url origin` | `std::process::Command` with captured stdout | Used only during forge detection; not on hot path |
| `git branch --show-current` | `std::process::Command` with captured stdout | Used only for browse; fallback to `git rev-parse HEAD` for detached HEAD |
| `gh` / `glab` / `tea` / `fj` | `exec()` replacement (Unix) or `spawn().wait()` (Windows) | Inherits all stdio; must be on PATH |
| Browser | `open` crate (`open::that(url)`) | Cross-platform; handles macOS `open`, Linux `xdg-open`, Windows `start` |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `detector` → `router` | `ForgeContext` struct (owned) | Detector produces; router consumes |
| `router` → `runner` | `(String, Vec<OsString>)` — binary + argv | Router produces final argv; runner just execs |
| `cli` → `detector` | remote name string (default "origin") | CLI can pass `--remote` override |
| `url_builder` → `runner` | URL string → `open::that()` | Browse path never reaches subprocess runner |

## Sources

- [git-url-parse crate (Rust)](https://docs.rs/git-url-parse) — URL parsing for both SSH and HTTPS git remote formats
- [std::process::Command (Rust std)](https://doc.rust-lang.org/std/process/struct.Command.html) — Standard subprocess API
- [std::os::unix::process::CommandExt](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html) — Unix exec() replacement
- [clap-rs/clap](https://github.com/clap-rs/clap) — CLI argument parsing; derive API recommended
- [Rain's Rust CLI recommendations — handling arguments](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html) — Community best practices
- [Forgejo forking forward announcement (2024)](https://forgejo.org/2024-02-forking-forward/) — Forgejo/Gitea divergence context
- [gitea/tea CLI](https://gitea.com/gitea/tea) — tea works against Forgejo instances (API compatible)

---
*Architecture research for: Rust CLI forge wrapper (gf)*
*Researched: 2026-03-16*
