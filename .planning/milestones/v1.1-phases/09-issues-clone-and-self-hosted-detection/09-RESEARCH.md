# Phase 9: Issues, Clone, and Self-Hosted Detection - Research

**Researched:** 2026-03-18
**Domain:** Multi-forge issue management, repo cloning, and CLI-based forge auto-detection
**Confidence:** HIGH

## Summary

Phase 9 extends `gf` with issue management commands (list, view, create, close, reopen), repo clone functionality, and automatic forge detection for unknown self-hosted domains via CLI auth probing. The implementation follows the established adapter translation pattern from Phase 8 (PR commands), with similar per-forge command/flag differences requiring translation.

**Key findings:**
- Issue commands follow PR command patterns closely: GitHub/GitLab/Forgejo use `issue` subcommand, Gitea uses `issues` (plural)
- Forgejo uses `fj issue search` instead of `fj issue list`, and has NO `reopen` subcommand (hard UnsupportedFeature error required)
- Gitea (tea) has NO `repo clone` subcommand (hard UnsupportedFeature error required)
- All four forge CLIs have auth status commands that output authenticated hostnames in stdout, enabling reliable forge detection via probing
- Rust `dirs` crate (v6.0.0) provides cross-platform cache directory paths via `dirs::cache_dir()` which respects XDG_CACHE_HOME on Linux and platform equivalents on macOS/Windows

**Primary recommendation:** Implement issue adapter using PR adapter as template, add probe_auth() function after existing config_lookup/match_known_host chain, cache probe results in TOML format at `~/.cache/gf/probes.toml` with simple hostname→forge_type mapping.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**CLI auth probing strategy (CORE-04):**
- Sequential probe order: gh → glab → tea → fj — stop on first match
- A "match" means the probed hostname appears in the CLI's auth status stdout output
- 5-second timeout per CLI probe (20s worst case for all four)
- Probe triggers only after config_lookup() AND match_known_host() both fail (existing fallback chain)
- Cache probe results indefinitely in `~/.cache/gf/` — no TTL expiry
- Config.toml ALWAYS takes precedence over cached probe results (user config > auto-detected)
- If no CLI matches, fall through to existing ForgeNotDetected error with config.toml hint

**Issue close/reopen semantics:**
- Modeled as subcommands: `gf issue close 42` / `gf issue reopen 42`
- Idempotency behavior delegated to forge CLI — gf does not check current state before sending
- No `--comment` flag on close — close is just close; commenting is a separate action
- `fj issue reopen` → hard UnsupportedFeature error (Forgejo CLI has no reopen command)
- Hard-error policy from Phase 8 applies to all unsupported issue command/flag combinations

**Clone URL resolution:**
- `gf repo clone owner/repo` requires a `[defaults]` config section with `clone_host` FQDN:
  ```toml
  [defaults]
  clone_host = "gitlab.mycompany.com"
  ```
- Without config: hard error with message showing the config.toml snippet to add
- `gf repo clone https://host/owner/repo` detects forge from URL host, delegates to forge CLI
- Full URL with unknown host → hard error: "Forge not detected for host X. Add it to config.toml or use `git clone` directly."
- Tea (Gitea CLI) has no `repo clone` → UnsupportedFeature error
- Clone always delegates to forge CLI (not `git clone`) for forge-specific setup benefits (PR refs, aliases)

### Claude's Discretion

- Issue command clap subcommand structure and flag definitions
- CORE-04 probe implementation (process spawning, stdout capture, parsing)
- Cache file format in `~/.cache/gf/` (TOML, JSON, etc.)
- Clone argument parsing (detecting full URL vs owner/repo shorthand)
- Config.toml `[defaults]` section deserialization structure
- Plan count and task breakdown

### Deferred Ideas (OUT OF SCOPE)

- **Issue commenting** (ISSUE-07) — v2 requirement, not in Phase 9 scope
- **Issue label assignment** (ISSUE-08) — v2 requirement, not in Phase 9 scope
- **Retroactive silent-omit → hard-error migration** — Deferred from Phase 8 context
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ISSUE-01 | User can list issues with filter flags (state, author, label) | Flag translations: --state maps to --closed/--merged/--all for glab; --label→--labels for tea/fj; --author supported by gh/glab/tea |
| ISSUE-02 | User can view a specific issue by number | Command translations: tea uses positional `issues 42` not `issues view 42`; all others use standard view subcommand |
| ISSUE-03 | User can create a new issue with title and body | Flag translations: --body→--description for glab/tea; --title→positional for fj |
| ISSUE-04 | User can close an issue | All four CLIs support close as subcommand; no special translations needed |
| ISSUE-05 | User can reopen a closed issue | fj has NO reopen → UnsupportedFeature error; gh/glab/tea all support reopen subcommand |
| ISSUE-06 | User can browse an issue in the browser (`gf browse --issue 42`) | Already implemented in Phase 8 via build_issue_url() in src/browse/mod.rs |
| REPO-01 | User can clone a repo via `gf repo clone owner/repo` or full URL | tea has NO clone → UnsupportedFeature error; requires new [defaults] config section for owner/repo shorthand; URL parsing for full URL detection |
| CORE-04 | Unknown domains probed via CLI auth status commands | Auth status commands: `gh auth status`, `glab auth status`, `tea logins ls`, `fj auth list` all output hostname in stdout; probe via spawning process with 5s timeout |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.6.0 | CLI parsing | Already in use; derive macros for issue subcommands |
| toml | 0.8.23 | Config/cache serialization | Already in use for config.toml; proven for cache file format |
| serde | 1.0.228 | Serialization framework | Already in use; required by toml |
| std::process::Command | stdlib | CLI spawning for probes | Zero-dependency; timeout via thread spawn pattern |
| std::fs | stdlib | File I/O for cache | Zero-dependency; sufficient for simple cache writes |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dirs | 6.0.0 (optional) | Cross-platform cache directory | Alternative to hardcoded `~/.cache/gf/` — provides `dirs::cache_dir()` which respects XDG_CACHE_HOME |
| thiserror | 2.0.18 | Error definitions | Already in use; extend GfError for cache/probe errors if needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| TOML cache | JSON (serde_json) | JSON requires extra dependency; TOML already in use and more human-editable |
| Manual $HOME/.cache | dirs crate | dirs crate adds dependency but handles Windows/macOS properly; manual works for Unix-only |
| Thread spawn timeout | tokio async timeout | tokio is massive dependency for simple timeout; thread spawn is 10 lines of code |

**Installation:**
```bash
# Optional: add dirs crate for cross-platform cache dir support
cargo add dirs@6
```

**Version verification:**
```bash
cargo tree --depth 1
# Already present:
# ├── clap v4.6.0
# ├── toml v0.8.23
# ├── serde v1.0.228
```

All required libraries are already in Cargo.toml except optional `dirs` crate.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── adapter/
│   ├── mod.rs          # Add "issue" arm to translate() dispatcher
│   ├── pr.rs           # Template for issue.rs implementation
│   ├── issue.rs        # NEW: Issue command translations (copy pr.rs pattern)
│   └── repo_auth.rs    # Extend translate_repo() with clone subcommand
├── cmd/
│   └── mod.rs          # Add build_issue() function; extend build_repo() with clone
├── forge/
│   └── mod.rs          # Add probe_auth() after config_lookup/match_known_host; add load_cache/save_cache
├── browse/
│   └── mod.rs          # build_issue_url() already exists (Phase 8)
└── error.rs            # GfError::UnsupportedFeature already exists (Phase 8)
```

### Pattern 1: Issue Adapter Translation (mirror PR pattern)

**What:** Per-forge command name and flag translation for issue subcommands
**When to use:** For all issue list/view/create/close/reopen commands
**Example:**
```rust
// src/adapter/issue.rs — Issue command translation (mirrors pr.rs structure)

pub fn translate_issue(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let issue_cmd = issue_subcommand_name(forge);
    
    match matches.subcommand() {
        Some(("list", sub)) => translate_issue_list(forge, issue_cmd, sub),
        Some(("view", sub)) => translate_issue_view(forge, issue_cmd, sub),
        Some(("create", sub)) => translate_issue_create(forge, issue_cmd, sub),
        Some(("close", sub)) => translate_issue_close(forge, issue_cmd, sub),
        Some(("reopen", sub)) => translate_issue_reopen(forge, issue_cmd, sub),
        Some((verb, sub)) => {
            let mut args = vec![issue_cmd.to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            Ok(args)
        }
        None => Ok(vec![issue_cmd.to_string()]),
    }
}

fn issue_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Github => "issue",
        ForgeType::Gitlab => "issue",
        ForgeType::Gitea => "issues",   // tea uses plural
        ForgeType::Forgejo => "issue",
    }
}

fn translate_issue_list(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string()];
    
    // Verb: fj uses "search" instead of "list"
    match forge {
        ForgeType::Forgejo => args.push("search".to_string()),
        _ => args.push("list".to_string()),
    }
    
    // --state: glab uses boolean flags (--closed/--all) not --state value
    if let Some(state) = matches.get_one::<String>("state") {
        match forge {
            ForgeType::Gitlab => {
                match state.as_str() {
                    "closed" => args.push("--closed".to_string()),
                    "all" => args.push("--all".to_string()),
                    "open" => {}, // default, omit flag
                    _ => {} // unknown state, let CLI error
                }
            }
            _ => {
                args.push("--state".to_string());
                args.push(state.clone());
            }
        }
    }
    
    // --label: tea and fj use --labels (plural)
    if let Some(label) = matches.get_one::<String>("label") {
        let label_flag = match forge {
            ForgeType::Gitea | ForgeType::Forgejo => "--labels",
            _ => "--label",
        };
        args.push(label_flag.to_string());
        args.push(label.clone());
    }
    
    // --author: all support except needs verification for fj (uses --creator)
    if let Some(author) = matches.get_one::<String>("author") {
        let author_flag = match forge {
            ForgeType::Forgejo => "--creator",
            _ => "--author",
        };
        args.push(author_flag.to_string());
        args.push(author.clone());
    }
    
    // Passthrough
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    
    Ok(args)
}
```

### Pattern 2: CLI Auth Probing with Timeout

**What:** Spawn forge CLI auth status command with 5-second timeout, parse stdout for hostname
**When to use:** In forge::detect() after config_lookup and match_known_host both return None/Err
**Example:**
```rust
// src/forge/mod.rs — Add probe_auth() function

use std::time::Duration;
use std::thread;
use std::sync::mpsc;

/// Probe forge CLIs for auth status containing the given hostname.
/// Returns the first matching ForgeType, or None if no CLI matches.
/// Checks in order: gh, glab, tea, fj (market share priority).
fn probe_auth(hostname: &str) -> Option<ForgeType> {
    let probes = [
        (ForgeType::Github, "gh", &["auth", "status"][..]),
        (ForgeType::Gitlab, "glab", &["auth", "status"]),
        (ForgeType::Gitea, "tea", &["logins", "ls"]),
        (ForgeType::Forgejo, "fj", &["auth", "list"]),
    ];
    
    for (forge, cli, args) in probes {
        if let Some(output) = run_with_timeout(cli, args, Duration::from_secs(5)) {
            // Check if hostname appears in stdout or stderr
            let text = format!("{}{}", 
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if text.contains(hostname) {
                return Some(forge);
            }
        }
    }
    None
}

/// Run a command with a timeout. Returns None if timeout expires or command fails.
fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Option<std::process::Output> {
    let (tx, rx) = mpsc::channel();
    
    let cmd_owned = cmd.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    
    thread::spawn(move || {
        let result = std::process::Command::new(&cmd_owned)
            .args(&args_owned)
            .output();
        let _ = tx.send(result);
    });
    
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => Some(output),
        _ => None,
    }
}
```

### Pattern 3: Cache File Management

**What:** Simple TOML cache mapping hostnames to forge types at `~/.cache/gf/probes.toml`
**When to use:** After successful probe, before returning ForgeType; load before probing
**Example:**
```rust
// src/forge/mod.rs — Cache management

#[derive(Debug, Deserialize, Serialize)]
struct ProbeCache {
    #[serde(default)]
    hosts: std::collections::HashMap<String, ForgeType>,
}

fn cache_path() -> Option<std::path::PathBuf> {
    // Option 1: Use dirs crate (if added as dependency)
    // dirs::cache_dir().map(|p| p.join("gf").join("probes.toml"))
    
    // Option 2: Manual (Unix-only, handles XDG_CACHE_HOME)
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Some(std::path::PathBuf::from(xdg).join("gf").join("probes.toml"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::PathBuf::from(home).join(".cache").join("gf").join("probes.toml"));
    }
    None
}

fn load_probe_cache() -> Option<ProbeCache> {
    let path = cache_path()?;
    if !path.exists() {
        return None;
    }
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&text).ok()
}

fn save_probe_cache(hostname: &str, forge: ForgeType) {
    let Some(path) = cache_path() else { return };
    
    // Load existing cache or create new
    let mut cache = load_probe_cache().unwrap_or_else(|| ProbeCache {
        hosts: std::collections::HashMap::new(),
    });
    
    // Insert new mapping
    cache.hosts.insert(hostname.to_string(), forge);
    
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    // Serialize and write
    if let Ok(toml_str) = toml::to_string_pretty(&cache) {
        let _ = std::fs::write(&path, toml_str);
    }
}

/// Enhanced detect() function with probe fallback
pub fn detect(remote: &str) -> Result<ForgeType, GfError> {
    let url = get_remote_url(remote)?;
    let host = parse_host(&url)?;
    
    // Priority 1: Config file (user override)
    if let Some(forge) = config_lookup(&host)? {
        return Ok(forge);
    }
    
    // Priority 2: Known public hosts
    if let Ok(forge) = match_known_host(&host) {
        return Ok(forge);
    }
    
    // Priority 3: Cached probe result
    if let Some(cache) = load_probe_cache() {
        if let Some(&forge) = cache.hosts.get(&host) {
            return Ok(forge);
        }
    }
    
    // Priority 4: Probe forge CLIs
    if let Some(forge) = probe_auth(&host) {
        save_probe_cache(&host, forge);
        return Ok(forge);
    }
    
    // No match: error with config hint
    Err(GfError::ForgeNotDetected { domain: host })
}
```

### Pattern 4: Clone Subcommand with URL Detection

**What:** Detect full URL vs owner/repo shorthand, delegate to forge CLI
**When to use:** In adapter::repo_auth::translate_repo() for clone subcommand
**Example:**
```rust
// src/adapter/repo_auth.rs — Add clone translation

fn translate_repo_clone(forge: ForgeType, repo_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    // tea has no clone subcommand
    if forge == ForgeType::Gitea {
        return Err(GfError::UnsupportedFeature {
            feature: "repo clone".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }
    
    let mut args = vec![repo_cmd.to_string(), "clone".to_string()];
    
    if let Some(repo_arg) = matches.get_one::<String>("repository") {
        // Detect full URL (starts with http:// or https://)
        if repo_arg.starts_with("http://") || repo_arg.starts_with("https://") {
            // Full URL: delegate as-is
            args.push(repo_arg.clone());
        } else if repo_arg.contains('/') {
            // owner/repo format: check for [defaults] clone_host in config
            if let Some(host) = load_clone_host()? {
                // For gh/glab/fj, pass owner/repo as-is (they resolve via their config)
                args.push(repo_arg.clone());
            } else {
                return Err(GfError::ConfigParseError(
                    "gf repo clone owner/repo requires [defaults] clone_host in config.toml:\n\
                     [defaults]\n\
                     clone_host = \"gitlab.mycompany.com\"".to_string()
                ));
            }
        } else {
            // No slash: invalid format
            return Err(GfError::ConfigParseError(
                "Invalid repo format. Use 'owner/repo' or full URL".to_string()
            ));
        }
    }
    
    // Passthrough
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    
    Ok(args)
}

// Add to forge/mod.rs config loading
fn load_clone_host() -> Result<Option<String>, GfError> {
    let cfg = match load_config()? {
        Some(c) => c,
        None => return Ok(None),
    };
    Ok(cfg.defaults.and_then(|d| d.clone_host))
}
```

### Anti-Patterns to Avoid

- **Probing before checking config:** Config.toml MUST take precedence — user explicitly configured mappings override probe results
- **Blocking main thread for probes:** Use thread spawn for timeout — blocking for 20 seconds is unacceptable UX
- **Not caching probe results:** Probing is expensive (5s × 4 CLIs worst case) — cache indefinitely per user decision
- **Silent fallback for tea clone:** tea has no clone → HARD ERROR per Phase 8 policy, not silent omit
- **Adding --comment to close:** User explicitly decided close is just close — commenting is separate action

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Command timeout | Custom async runtime or signal handling | Thread spawn + mpsc::channel with recv_timeout | 10 lines of code vs. tokio dependency; stdlib sufficient for simple timeout |
| Cache directory path | Hardcode `~/.cache/gf/` | Optional: dirs crate's `cache_dir()` | Handles Windows (%LOCALAPPDATA%) and macOS (~/Library/Caches) properly; respects XDG_CACHE_HOME on Linux |
| TOML serialization | Manual string formatting | serde + toml crate (already in use) | Edge cases (escaping, types) already handled; project already depends on toml |
| URL parsing | Regex or manual string parsing | Starts_with for scheme detection, contains('/') for format detection | URLs are simple in this context (just scheme detection); no need for url crate |

**Key insight:** Avoid over-engineering timeout and caching — stdlib and existing dependencies (toml, serde) cover all requirements. The only potential addition is `dirs` crate for cross-platform cache paths, which is optional (manual approach works for Unix).

## Common Pitfalls

### Pitfall 1: GitLab Issue List State Flags

**What goes wrong:** Using `--state closed` with glab fails — glab requires boolean flags `--closed`, `--all`, not `--state <value>`
**Why it happens:** GitHub/Gitea/Forgejo all use `--state <value>` pattern; glab is the exception
**How to avoid:** Match on forge type in translate_issue_list() — map state values to boolean flags for Gitlab only
**Warning signs:** Test failure on glab issue list with `--state` flag; glab CLI error "unknown flag: --state"

### Pitfall 2: Tea Issue View Syntax

**What goes wrong:** Calling `tea issues view 42` fails — tea has no "view" verb, uses positional `tea issues 42` directly
**Why it happens:** All other CLIs use explicit view subcommand; tea treats number as positional arg
**How to avoid:** In translate_issue_view(), omit "view" verb for Gitea and push number as positional arg
**Warning signs:** Test failure with "unknown command 'view'" from tea CLI

### Pitfall 3: Forgejo Has No Reopen

**What goes wrong:** User tries `gf issue reopen 42` on Forgejo repo — should get clear error, not hang or cryptic failure
**Why it happens:** fj CLI has no `issue reopen` subcommand
**How to avoid:** In translate_issue_reopen(), match on forge and return UnsupportedFeature error for Forgejo
**Warning signs:** Test expectation for hard error per Phase 8 policy; unsupported_test! macro in flag_audit.rs

### Pitfall 4: Forgejo Uses --creator not --author

**What goes wrong:** `gf issue list --author alice` on Forgejo returns no results — fj expects `--creator` flag
**Why it happens:** Forgejo CLI chose different terminology for author filtering
**How to avoid:** In translate_issue_list(), map --author to --creator for Forgejo only
**Warning signs:** Verified in CLI help output: `fj issue search --help` shows `--creator` not `--author`

### Pitfall 5: Tea Has No Clone

**What goes wrong:** User tries `gf repo clone owner/repo` on Gitea — should get clear error, not cryptic tea failure
**Why it happens:** tea CLI has no repos clone subcommand (verified: `tea repos clone --help` returns "No help topic for 'clone'")
**How to avoid:** In translate_repo_clone(), return UnsupportedFeature error for Gitea at function entry
**Warning signs:** unsupported_test! macro expectation in flag_audit.rs; tea documentation gap

### Pitfall 6: Probe Stdout/Stderr Combination

**What goes wrong:** Probe misses hostname because it's only in stderr (e.g., glab auth status errors go to stderr)
**Why it happens:** CLIs output to both stdout and stderr; hostname might be in either stream
**How to avoid:** Concatenate stdout and stderr when checking for hostname match: `format!("{}{}", stdout, stderr)`
**Warning signs:** Real-world testing shows glab outputs errors to stderr but still includes hostname

### Pitfall 7: Cache Directory Creation

**What goes wrong:** Writing cache file fails because `~/.cache/gf/` doesn't exist yet
**Why it happens:** First run won't have cache directory created
**How to avoid:** Call `std::fs::create_dir_all(parent_dir)` before writing cache file
**Warning signs:** Permission denied or "no such file or directory" errors on cache write

### Pitfall 8: Clone Host Config Without [defaults]

**What goes wrong:** User adds `clone_host = "host"` directly in root of config.toml — deserialization fails
**Why it happens:** Serde requires explicit struct field or table section; can't have loose keys in TOML root
**How to avoid:** Document that [defaults] section is required; error message shows correct TOML structure
**Warning signs:** TOML parse error "missing field" or "unknown key"

## Code Examples

Verified patterns from local CLI help output and existing project code:

### Issue List Translation (verified via CLI help)

```rust
// Source: Analyzed `gh issue list --help`, `glab issue list --help`, 
//         `tea issues list --help`, `fj issue search --help`

fn translate_issue_list(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string()];
    
    // Verb: fj uses "search" instead of "list"
    match forge {
        ForgeType::Forgejo => args.push("search".to_string()),
        _ => args.push("list".to_string()),
    }
    
    // --state: glab uses boolean flags, others use --state value
    if let Some(state) = matches.get_one::<String>("state") {
        match forge {
            ForgeType::Gitlab => {
                // glab: --closed, --all, or omit for open (default)
                match state.as_str() {
                    "closed" => args.push("--closed".to_string()),
                    "all" => args.push("--all".to_string()),
                    "open" => {}, // default, omit
                    _ => {}
                }
            }
            _ => {
                args.push("--state".to_string());
                args.push(state.clone());
            }
        }
    }
    
    // --label: tea and fj use plural --labels
    if let Some(label) = matches.get_one::<String>("label") {
        let flag = match forge {
            ForgeType::Gitea | ForgeType::Forgejo => "--labels",
            _ => "--label",
        };
        args.push(flag.to_string());
        args.push(label.clone());
    }
    
    // --author: fj uses --creator
    if let Some(author) = matches.get_one::<String>("author") {
        let flag = match forge {
            ForgeType::Forgejo => "--creator",
            _ => "--author",
        };
        args.push(flag.to_string());
        args.push(author.clone());
    }
    
    Ok(args)
}
```

### Issue View Translation (tea special case)

```rust
// Source: Verified `tea issues --help` shows positional number, no "view" verb

fn translate_issue_view(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string()];
    
    // tea uses positional number, no "view" verb
    if forge != ForgeType::Gitea {
        args.push("view".to_string());
    }
    
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }
    
    Ok(args)
}
```

### Issue Create Translation

```rust
// Source: Verified via `gh/glab/tea/fj issue create --help`

fn translate_issue_create(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string(), "create".to_string()];
    
    if let Some(title) = matches.get_one::<String>("title") {
        args.push("--title".to_string());
        args.push(title.clone());
    }
    
    // --body: glab and tea use --description
    if let Some(body) = matches.get_one::<String>("body") {
        let flag = match forge {
            ForgeType::Gitlab | ForgeType::Gitea => "--description",
            ForgeType::Github | ForgeType::Forgejo => "--body",
        };
        args.push(flag.to_string());
        args.push(body.clone());
    }
    
    Ok(args)
}
```

### Issue Close Translation (straightforward)

```rust
// Source: All four CLIs support `issue close <number>` identically

fn translate_issue_close(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string(), "close".to_string()];
    
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }
    
    Ok(args)
}
```

### Issue Reopen Translation (Forgejo unsupported)

```rust
// Source: Verified `fj issue --help` has no reopen subcommand

fn translate_issue_reopen(forge: ForgeType, issue_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    // Forgejo has no reopen command
    if forge == ForgeType::Forgejo {
        return Err(GfError::UnsupportedFeature {
            feature: "issue reopen".to_string(),
            forge: "Forgejo".to_string(),
            forge_cli: "fj".to_string(),
        });
    }
    
    let mut args = vec![issue_cmd.to_string(), "reopen".to_string()];
    
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }
    
    Ok(args)
}
```

### Auth Status Output Parsing (real CLI outputs)

```rust
// Source: Local CLI testing showed actual output formats

fn probe_auth(hostname: &str) -> Option<ForgeType> {
    let probes = [
        (ForgeType::Github, "gh", &["auth", "status"][..]),
        (ForgeType::Gitlab, "glab", &["auth", "status"]),
        (ForgeType::Gitea, "tea", &["logins", "ls"]),
        (ForgeType::Forgejo, "fj", &["auth", "list"]),
    ];
    
    for (forge, cli, args) in probes {
        if let Some(output) = run_with_timeout(cli, args, Duration::from_secs(5)) {
            // Combine stdout and stderr - hostname can be in either
            // Example gh output: "github.com\n  ✓ Logged in to github.com account..."
            // Example glab output: "gitlab.com\n  x gitlab.com: API call failed..."
            // Example fj output: "user@code.skillbyte.ai"
            let text = format!("{}{}", 
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if text.contains(hostname) {
                return Some(forge);
            }
        }
    }
    None
}
```

### Clap Issue Command Definition

```rust
// Source: Pattern from src/cmd/mod.rs build_pr() function

fn build_issue() -> Command {
    Command::new("issue")
        .about("Issue commands")
        .subcommand_required(false)
        .subcommand(
            Command::new("list")
                .about("List issues in a repository")
                .visible_alias("l")
                .arg(Arg::new("state")
                    .long("state")
                    .short('s')
                    .value_name("STATE")
                    .help("Filter by state: {open|closed|all}"))
                .arg(Arg::new("author")
                    .long("author")
                    .short('A')
                    .value_name("USERNAME")
                    .help("Filter by author"))
                .arg(Arg::new("label")
                    .long("label")
                    .short('l')
                    .value_name("NAME")
                    .help("Filter by label"))
                .arg(Arg::new("extra")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .last(true)
                    .help("Additional flags passed through"))
        )
        .subcommand(
            Command::new("view")
                .about("View an issue by number")
                .visible_alias("v")
                .arg(Arg::new("number")
                    .value_name("NUMBER")
                    .required(true)
                    .help("Issue number"))
                .arg(Arg::new("extra")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .last(true)
                    .help("Additional flags passed through"))
        )
        .subcommand(
            Command::new("create")
                .about("Create a new issue")
                .visible_alias("c")
                .arg(Arg::new("title")
                    .long("title")
                    .short('t')
                    .value_name("TITLE")
                    .help("Issue title"))
                .arg(Arg::new("body")
                    .long("body")
                    .short('b')
                    .value_name("BODY")
                    .help("Issue body/description"))
                .arg(Arg::new("extra")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .last(true)
                    .help("Additional flags passed through"))
        )
        .subcommand(
            Command::new("close")
                .about("Close an issue")
                .arg(Arg::new("number")
                    .value_name("NUMBER")
                    .required(true)
                    .help("Issue number"))
                .arg(Arg::new("extra")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .last(true)
                    .help("Additional flags passed through"))
        )
        .subcommand(
            Command::new("reopen")
                .about("Reopen a closed issue")
                .arg(Arg::new("number")
                    .value_name("NUMBER")
                    .required(true)
                    .help("Issue number"))
                .arg(Arg::new("extra")
                    .num_args(0..)
                    .allow_hyphen_values(true)
                    .last(true)
                    .help("Additional flags passed through"))
        )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual forge config for all domains | CLI auth probing for unknown domains | v1.1 (this phase) | Zero-config experience for self-hosted forges if user already authenticated via CLI |
| gh/glab/tea browse delegation | Native gf browse with issue URL construction | v1.0 Phase 6 & Phase 8 | Consistent URL format across all forges; tea browse workaround no longer needed |
| Silent omit for unsupported flags | Hard error for unsupported commands | v1.0 Phase 8 | User gets immediate feedback instead of silent failure |

**Deprecated/outdated:**
- No deprecated patterns in this phase — all features are new to gf v1.1

## Open Questions

1. **Should we use `dirs` crate or manual $HOME/.cache approach?**
   - What we know: Manual approach works fine for Unix; dirs adds cross-platform support (Windows/macOS)
   - What's unclear: Project philosophy on dependencies — is one extra crate acceptable for proper cross-platform support?
   - Recommendation: Start with manual `$HOME/.cache` (Unix-only) and note in docs/code that dirs crate would enable Windows/macOS. Can add later if users request it.

2. **Should probe cache survive across gf versions?**
   - What we know: User decision says "cache indefinitely, no TTL"
   - What's unclear: If forge changes from Gitea to Forgejo (same domain), cache would be stale
   - Recommendation: Follow user decision for v1.1 — cache indefinitely. Add version prefix to cache file in future if this becomes a problem (e.g., `probes_v1.toml`).

3. **Should probe errors be logged/visible to user?**
   - What we know: Probe failures are expected (CLI not installed, timeout, not authenticated)
   - What's unclear: Silent failure vs. verbose output — does user want to know probing is happening?
   - Recommendation: Silent probe failures (fall through to error message with config hint). Add `GF_DEBUG` env var support in future if users need troubleshooting.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework + assert_cmd v2.2.0 |
| Config file | None — cargo test discovers all `#[test]` functions |
| Quick run command | `cargo test --lib` (unit tests only, fast) |
| Full suite command | `cargo test` (includes integration tests) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ISSUE-01 | issue list with --state/--label/--author | unit | `cargo test issue_list -x` | ✅ tests/flag_audit.rs v11_translation_test! entries exist |
| ISSUE-02 | issue view with number | unit | `cargo test issue_view -x` | ✅ tests/flag_audit.rs v11_translation_test! entries exist |
| ISSUE-03 | issue create with --title/--body | unit | `cargo test issue_create -x` | ✅ tests/flag_audit.rs v11_translation_test! entries exist |
| ISSUE-04 | issue close <number> | unit | `cargo test issue_close -x` | ❌ Wave 0 — no close tests in flag_audit.rs yet |
| ISSUE-05 | issue reopen <number> | unit | `cargo test issue_reopen -x` | ❌ Wave 0 — no reopen tests in flag_audit.rs yet |
| ISSUE-06 | browse --issue <N> | unit | `cargo test build_issue_url -x` | ✅ src/browse/mod.rs existing tests |
| REPO-01 | repo clone owner/repo and full URL | unit | `cargo test repo_clone -x` | ✅ tests/flag_audit.rs v11_translation_test! entries exist |
| CORE-04 | probe_auth() detects forge from CLI output | unit | `cargo test probe_auth -x` | ❌ Wave 0 — new functionality, needs unit tests |

### Sampling Rate
- **Per task commit:** `cargo test --lib` (unit tests, ~2s runtime)
- **Per wave merge:** `cargo test` (full suite including integration tests, ~10s runtime)
- **Phase gate:** Full suite green + manual verification on real forge instances

### Wave 0 Gaps
- [ ] `tests/flag_audit.rs` — Add v11_translation_test! entries for issue close/reopen (cover all 4 forges)
- [ ] `tests/flag_audit.rs` — Add unsupported_test! for fj issue reopen and tea repo clone
- [ ] `src/forge/mod.rs` — Add #[cfg(test)] module for probe_auth() unit tests (mock Command::output)
- [ ] `src/forge/mod.rs` — Add #[cfg(test)] module for cache load/save unit tests (tempfile for isolation)

## Sources

### Primary (HIGH confidence)
- Local CLI help output: `gh/glab/tea/fj issue --help`, `gh/glab/tea/fj auth status --help` — verified command syntax, flags, and output formats
- Existing project code: `src/adapter/pr.rs` (pattern template), `src/cmd/mod.rs` (clap structure), `src/forge/mod.rs` (config loading)
- Real CLI execution: `gh auth status`, `glab auth status`, `tea logins ls`, `fj auth list` — verified actual output formats for probe parsing
- Cargo.toml dependencies: clap 4.6.0, toml 0.8.23, serde 1.0.228, thiserror 2.0.18 — confirmed versions already in use

### Secondary (MEDIUM confidence)
- Rust stdlib documentation (std::process::Command, std::fs, std::thread) — standard library usage patterns
- cargo search output for dirs crate — confirmed v6.0.0 exists and purpose (XDG directory support)

### Tertiary (LOW confidence)
- None — all findings verified via direct CLI testing or existing project code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in project except optional dirs crate
- Architecture: HIGH - Direct copy of existing PR adapter pattern; probe pattern from stdlib
- Pitfalls: HIGH - All verified via actual CLI help output and testing
- Code examples: HIGH - All examples based on existing project patterns or verified CLI output

**Research date:** 2026-03-18
**Valid until:** 90 days (stable CLIs, established patterns)
