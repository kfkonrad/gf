# Phase 2: Forge Detection - Research

**Researched:** 2026-03-16
**Domain:** Rust — git remote URL parsing, TOML config, module design
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Detection priority:** Config → URL match → error. Config checked first and always wins (even for known public hosts). Enables mirror/proxy scenarios.
- **Known public hosts:** `github.com` → gh, `gitlab.com` → glab, `gitea.com` → tea, `codeberg.org` → fj. Both HTTPS and SSH SCP-style URLs supported.
- **Self-hosted (CORE-04 dropped):** Auth probing is dropped. Unknown domains not in config produce a detection failure error with copy-pasteable TOML.
- **Config format:** `[[forge]]` array of inline tables in `~/.config/gf/config.toml`. Valid `type` values: `github`, `gitlab`, `gitea`, `forgejo`. Parsed on use only.
- **`--remote` flag (CORE-02):** Uses specified remote URL instead of `origin`. Detection logic is identical.
- **Error format — unknown domain:** Three-block plain text showing domain, supported types, and copy-pasteable TOML snippet. `forgejo` listed first in the type comment.
- **Error format — not a git repo:** `not a git repository (or any parent directory)`
- **Error format — no origin remote:** `no remote named 'origin' — use --remote to specify one`
- All errors: stderr, plain text, no prefix, no ANSI color (Phase 1 style).

### Claude's Discretion

- Rust crate for TOML parsing (`toml` crate is the obvious choice)
- Rust crate or stdlib for git remote URL parsing
- Module structure for the detector (`forge/mod.rs`, `detect.rs`, etc.)
- Exact `git remote get-url` invocation vs parsing `.git/config` directly

### Deferred Ideas (OUT OF SCOPE)

- `gf config add-forge <domain> <type>` command — the detection failure error message with copy-pasteable TOML is sufficient
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-01 | Detect forge type from `origin` git remote URL (HTTPS and SCP/SSH formats) | URL parsing section below; `git remote get-url` invocation pattern |
| CORE-02 | Support `--remote <name>` flag to override default `origin` remote | Same URL parsing path, just parameterized remote name |
| CORE-03 | Detect known public forge hosts: github.com (gh), gitlab.com (glab), gitea.com (tea), codeberg.org (fj) | Built-in match table in detect module |
| CORE-04 | (Dropped per CONTEXT.md) Auth probing removed; replaced by config-only self-hosted detection | N/A |
| CORE-05 | Domain-to-forge-type mappings in `~/.config/gf/config.toml` | TOML crate usage, config file path, deserialization pattern |
</phase_requirements>

---

## Summary

Phase 2 adds forge detection between `main()` and `runner::run()`. The detection path is: read optional config file, parse the remote URL to extract the hostname, check config entries first (wins if found), then fall back to built-in host table, then emit a structured error.

All three subtasks are pure Rust with no external process calls except `git remote get-url` (one subprocess, already established in Phase 1's runner pattern). The `toml` crate (v1.0, current) handles config parsing via serde. Git remote URL parsing can be handled by either `git-url-parse` (crate) or a simple hand-written regex covering HTTPS and SCP-style — the regex path is a viable choice given the narrow scope (extract hostname only, two formats).

**Primary recommendation:** Use `git remote get-url <remote>` via `std::process::Command` to fetch the remote URL, then extract the hostname with a small inline parser. Use the `toml` crate with serde for config. New module at `src/forge/mod.rs` with sub-functions for each concern.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `toml` | 1.0.6 | Parse `config.toml` | Official TOML library for Rust; what Cargo itself uses |
| `serde` | 1.x | Derive deserialization for config structs | Required companion to `toml` |

### Already Present (no new deps needed for URL parsing)
| Approach | Why Sufficient |
|----------|---------------|
| `std::process::Command` + regex/hand-parse | `git remote get-url` returns a clean single-line string; host extraction is 2 patterns |

### Alternative Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-written URL host extractor | `git-url-parse` 0.6.0 | `git-url-parse` adds a dep for minimal benefit: we only need the hostname, not a full GitUrl struct. Hand-parsing 2 regex patterns is 10 lines. |
| `git remote get-url` subprocess | Parse `.git/config` directly | Subprocess is simpler: git handles worktrees, submodules, config inheritance. Direct parse is fragile. |

**Installation (new deps only):**
```bash
cargo add toml serde --features serde/derive
```

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs          # calls forge::detect(remote_name) → cli_name, then runner::run()
├── error.rs         # GfError — add ForgeNotDetected, NotAGitRepo, NoRemote, ConfigParseError variants
├── runner.rs        # unchanged from Phase 1
└── forge/
    └── mod.rs       # detect(), get_remote_url(), parse_host(), load_config(), match_host()
```

### Pattern 1: Detection Pipeline
**What:** Sequential fallible steps, each returning `Result<ForgeType, GfError>`.
**When to use:** Every `gf` invocation that needs a forge.

```rust
// src/forge/mod.rs
pub fn detect(remote: &str) -> Result<ForgeType, GfError> {
    let url = get_remote_url(remote)?;          // runs `git remote get-url <remote>`
    let host = parse_host(&url)?;               // extracts domain from HTTPS or SCP URL
    let config = load_config();                 // returns Ok(None) if file absent
    if let Some(cfg) = config? {
        if let Some(forge) = cfg.lookup(&host) {
            return Ok(forge);                   // config always wins
        }
    }
    match_known_host(&host)                     // built-in table or ForgeNotDetected error
}
```

### Pattern 2: `git remote get-url` via subprocess
**What:** Run `git remote get-url <name>`, capture stdout, strip newline.

```rust
// Source: std::process::Command docs
let output = std::process::Command::new("git")
    .args(["remote", "get-url", remote])
    .output()
    .map_err(|e| GfError::GitCommandFailed(e))?;

if !output.status.success() {
    // git exits non-zero when remote doesn't exist OR when not in a repo
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("not a git repository") {
        return Err(GfError::NotAGitRepo);
    }
    return Err(GfError::NoRemote(remote.to_string()));
}

let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
```

### Pattern 3: Host extraction from two URL formats
**What:** Extract hostname from HTTPS or SCP-style URL.

```rust
fn parse_host(url: &str) -> Result<String, GfError> {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(rest) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) {
        let host = rest.split('/').next().unwrap_or("");
        if !host.is_empty() { return Ok(host.to_string()); }
    }
    // SCP-style: git@github.com:owner/repo.git
    if let Some(at_pos) = url.find('@') {
        let after_at = &url[at_pos + 1..];
        let host = after_at.split(':').next().unwrap_or("");
        if !host.is_empty() { return Ok(host.to_string()); }
    }
    Err(GfError::RemoteUrlUnrecognized(url.to_string()))
}
```

### Pattern 4: TOML config deserialization
**What:** Serde + `toml::from_str` for `[[forge]]` array.

```rust
// Source: https://docs.rs/toml/latest/toml/
use serde::Deserialize;

#[derive(Deserialize)]
struct GfConfig {
    #[serde(default)]
    forge: Vec<ForgeEntry>,
}

#[derive(Deserialize)]
struct ForgeEntry {
    domain: String,
    #[serde(rename = "type")]
    forge_type: ForgeType,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ForgeType { Github, Gitlab, Gitea, Forgejo }

fn load_config() -> Result<Option<GfConfig>, GfError> {
    let path = config_path()?;          // ~/.config/gf/config.toml
    if !path.exists() { return Ok(None); }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| GfError::ConfigReadError(e))?;
    toml::from_str(&text)
        .map(Some)
        .map_err(|e| GfError::ConfigParseError(e.to_string()))
}
```

### Pattern 5: Config file path resolution
**What:** `~/.config/gf/config.toml` — use `dirs` crate or `HOME` env var.

```rust
// Option A: dirs crate (adds dep)
let base = dirs::config_dir().ok_or(GfError::ConfigDirNotFound)?;
let path = base.join("gf").join("config.toml");

// Option B: env var (no dep, simpler)
let home = std::env::var("HOME").map_err(|_| GfError::ConfigDirNotFound)?;
let path = std::path::PathBuf::from(home).join(".config").join("gf").join("config.toml");
```

Recommendation: Use `HOME` env var approach — no extra dep, sufficient for Unix (primary platform). If Windows support is needed later, add `dirs`.

### Anti-Patterns to Avoid
- **Calling `git config --get remote.origin.url` directly:** Returns URL for `origin` only; `get-url` handles `--pushurl` overrides and is the canonical way to get a remote URL.
- **Parsing `.git/config` with string splitting:** Fragile against includes, worktrees, and conditional configs.
- **Using `url::Url::parse()` for SCP URLs:** The `url` crate rejects `git@host:path` as malformed — it is not a valid RFC 3986 URL.
- **Starting config parse at every invocation:** CONTEXT.md says parse on use only. Load lazily inside `detect()` only if URL match fails.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML parsing | Custom parser | `toml` crate | Handles edge cases: multiline strings, escaped chars, inline tables, spec 1.1 |
| PATH lookup for `git` binary | Manual PATH split | `which::which("git")` (already in deps) | Already used in Phase 1 |

**Key insight:** URL host extraction is simple enough here (two patterns, hostname only) that no dedicated URL crate is needed. The complexity is in TOML — use the crate.

---

## Common Pitfalls

### Pitfall 1: `git remote get-url` exit codes are ambiguous
**What goes wrong:** Git exits non-zero for both "not a git repo" and "remote name doesn't exist", but the stderr messages differ.
**Why it happens:** Git uses stderr text, not exit codes, to distinguish error types.
**How to avoid:** Check stderr string content: `not a git repository` vs `No such remote`.
**Warning signs:** Tests pass in git repos but fail outside them.

### Pitfall 2: Hostname may include port
**What goes wrong:** `https://git.company.com:8443/owner/repo.git` — naively splitting on `/` gives `git.company.com:8443` as the "host". Config entry would need to match this exact string.
**Why it happens:** Ports are valid in HTTPS URLs.
**How to avoid:** Strip port from extracted host before lookup. `host.split(':').next()` on the HTTPS-extracted segment handles this.
**Warning signs:** Self-hosted instance with non-standard port never matches config.

### Pitfall 3: Config file absent is not an error
**What goes wrong:** `std::fs::read_to_string` returns `Err` when file is absent. Treating this as a fatal error breaks all users without config files (the majority).
**Why it happens:** `io::Error` doesn't distinguish "not found" from "permission denied".
**How to avoid:** Check `path.exists()` first, or match on `io::ErrorKind::NotFound` and return `Ok(None)`.

### Pitfall 4: `serde` `rename = "type"` required
**What goes wrong:** `type` is a Rust keyword. A struct field named `type` fails to compile.
**Why it happens:** TOML uses `type` as a plain key, Rust reserves it.
**How to avoid:** Use `#[serde(rename = "type")] forge_type: ForgeType`.

### Pitfall 5: ForgeType display must return CLI binary name, not forge name
**What goes wrong:** Detection returns `ForgeType::Gitea` but `runner::run()` needs the binary name `"tea"`.
**Why it happens:** Config `type` field uses forge name (`gitea`) but the CLI binary is `tea` (and `fj` for Forgejo).
**How to avoid:** Implement a `fn cli_name(&self) -> &'static str` method on `ForgeType`. Make this the single source of truth — used in both detection and existing `cli_info()` in `error.rs`.

---

## Code Examples

### Error variant additions to `src/error.rs`
```rust
// Source: established Phase 1 pattern — thiserror
#[error("not a git repository (or any parent directory)")]
NotAGitRepo,

#[error("no remote named '{0}' — use --remote to specify one")]
NoRemote(String),

#[error("Could not detect forge for: {domain}\n\nSupported forges: github, gitlab, gitea, forgejo\n\nAdd a mapping to ~/.config/gf/config.toml:\n  [[forge]]\n  domain = \"{domain}\"\n  type = \"forgejo\"  # or github, gitlab, gitea")]
ForgeNotDetected { domain: String },

#[error("failed to parse config: {0}")]
ConfigParseError(String),

#[error("remote URL not recognized: {0}")]
RemoteUrlUnrecognized(String),
```

### Built-in host table
```rust
fn match_known_host(host: &str) -> Result<ForgeType, GfError> {
    match host {
        "github.com"   => Ok(ForgeType::Github),
        "gitlab.com"   => Ok(ForgeType::Gitlab),
        "gitea.com"    => Ok(ForgeType::Gitea),
        "codeberg.org" => Ok(ForgeType::Forgejo),
        other          => Err(GfError::ForgeNotDetected { domain: other.to_string() }),
    }
}
```

### `main.rs` integration point (Phase 2 replaces placeholder)
```rust
// Phase 2: replace the arg-as-cli-name placeholder
let remote = /* parse --remote flag, default "origin" */;
let forge = forge::detect(&remote)?;
let cli = forge.cli_name();
runner::run(cli, &remaining_args)?;
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `toml` 0.5 (pre-serde 1.0 unification) | `toml` 1.0 (serde 1.0, spec 1.1) | Breaking change in 0.x to 1.x; use 1.x directly |
| `url::Url::parse` for git remotes | Purpose-built parser or regex | `url` crate rejects SCP-style URLs |

**Deprecated:**
- CORE-04 (auth probing): Dropped by user decision. Do not implement.

---

## Open Questions

1. **Port stripping scope**
   - What we know: Hostname with port (`host:8443`) won't match a config `domain = "host"` entry if not stripped.
   - What's unclear: Whether the CONTEXT.md intent is to match on bare domain only or domain:port.
   - Recommendation: Strip port during host extraction (match on bare domain). Self-hosted instances rarely use non-standard ports for git; if they do, config entry would need the port. Default to stripping.

2. **`--remote` flag parsing location**
   - What we know: Phase 2 owns forge detection; clap is not yet introduced (Phase 3 adds full CLI parsing).
   - What's unclear: Whether Phase 2 should introduce clap for `--remote`, use hand-rolled arg parsing, or just thread a `remote` parameter with `"origin"` as default.
   - Recommendation: Thread `remote: &str` parameter from `main.rs` with simple `args` iteration for `--remote`. Clap arrives in Phase 3 and will supersede this. Keep Phase 2 minimal.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + `assert_cmd` 2.x + `predicates` 3.x |
| Config file | `Cargo.toml` `[dev-dependencies]` (already configured) |
| Quick run command | `cargo test 2>&1` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | Detect forge from HTTPS remote URL | unit | `cargo test forge::tests::test_detect_https` | Wave 0 |
| CORE-01 | Detect forge from SCP SSH remote URL | unit | `cargo test forge::tests::test_detect_ssh` | Wave 0 |
| CORE-01 | Not-a-git-repo error | integration | `cargo test test_not_a_git_repo` | Wave 0 |
| CORE-01 | No-origin error | integration | `cargo test test_no_origin_remote` | Wave 0 |
| CORE-02 | `--remote upstream` uses upstream URL | unit | `cargo test forge::tests::test_detect_custom_remote` | Wave 0 |
| CORE-03 | All four known hosts detected correctly | unit | `cargo test forge::tests::test_known_hosts` | Wave 0 |
| CORE-05 | Config entry overrides built-in host | unit | `cargo test forge::tests::test_config_override` | Wave 0 |
| CORE-05 | Config entry detects self-hosted domain | unit | `cargo test forge::tests::test_config_self_hosted` | Wave 0 |
| CORE-05 | Missing config file is not an error | unit | `cargo test forge::tests::test_config_absent` | Wave 0 |
| CORE-05 | Malformed config produces ConfigParseError | unit | `cargo test forge::tests::test_config_malformed` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test 2>&1`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/forge/mod.rs` — module with unit test submodule (`#[cfg(test)] mod tests`)
- [ ] Integration tests for error messages in `tests/integration_test.rs` (extend existing file)

---

## Sources

### Primary (HIGH confidence)
- `cargo search toml` (live crates.io) — version 1.0.6+spec-1.1.0 confirmed current
- `cargo search git-url-parse` (live crates.io) — version 0.6.0
- https://docs.rs/git-url-parse — SCP-style URL support confirmed, `host()` accessor
- https://docs.rs/toml/latest/toml/ — serde integration, `from_str` API
- Existing `src/error.rs`, `src/runner.rs`, `Cargo.toml` — Phase 1 established patterns

### Secondary (MEDIUM confidence)
- WebSearch: `url::Url::parse` incompatibility with SCP-style URLs — confirmed by cargo/servo GitHub issues
- WebSearch: `toml` 1.0 serde `#[derive(Deserialize)]` pattern — consistent across multiple docs

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — live crates.io search, official docs verified
- Architecture: HIGH — patterns derived from Phase 1 established code + CONTEXT.md locked decisions
- Pitfalls: HIGH — SCP/`url` crate incompatibility is well-documented in cargo/servo issues; `type` keyword issue is Rust fundamentals

**Research date:** 2026-03-16
**Valid until:** 2026-06-16 (stable Rust ecosystem; `toml` and `serde` are long-stable)
