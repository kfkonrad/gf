# Phase 4: Browse - Research

**Researched:** 2026-03-16
**Domain:** Rust CLI — native URL construction + cross-platform browser launch
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Extend `src/forge/` module with `parse_remote_parts()` returning host, owner, repo — single source of truth for all remote URL parsing
- `parse_remote_parts()` handles both HTTPS (`https://host/owner/repo.git`) and SCP SSH (`git@host:owner/repo.git`) formats in one function, stripping `.git` suffix
- Self-hosted forge base URL derived from remote host automatically (`https://` + host) — no config required
- Use `webbrowser` crate (solved problem, cross-platform)
- Always print URL to stdout when opening (useful for scripting, debugging)
- Add `-n` / `--no-browser` flag for non-interactive/CI/headless use
- On browser open failure: print URL + error message, exit non-zero
- GitHub file URL:  `https://github.com/owner/repo/blob/<branch>/<path>`
- GitLab file URL:  `https://gitlab.com/owner/repo/-/blob/<branch>/<path>`
- Gitea file URL:   `https://gitea.com/owner/repo/src/branch/<branch>/<path>`
- Forgejo file URL: `https://codeberg.org/owner/repo/src/branch/<branch>/<path>`
- Self-hosted instances use same path patterns with derived host
- No local path validation — user can browse paths that exist on remote but not locally
- Auto-convert absolute paths to repo-relative via `git rev-parse --show-toplevel`; relative paths passed as-is
- Use full 40-char SHA for detached HEAD (unambiguous on all forges)
- Detect detached HEAD when `git symbolic-ref HEAD` fails; fall back to `git rev-parse HEAD`
- URL construction failure (no remote, can't parse): clear error, exit non-zero
- Module structure for browse (Claude's discretion)
- Exact error message wording (Claude's discretion)
- How `webbrowser` crate is integrated — direct call vs thin wrapper (Claude's discretion)

### Claude's Discretion
- Module structure for browse (e.g., `src/browse/mod.rs` vs inline in `src/cmd/`)
- Exact error message wording
- How `webbrowser` crate is integrated (direct call vs thin wrapper)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BROWSE-01 | `gf browse` (alias `gf b`) opens current repo in browser at correct forge URL | `webbrowser::open()` API; `parse_remote_parts()` provides host/owner/repo; repo URL format per forge |
| BROWSE-02 | Uses current branch by default; falls back to HEAD SHA when detached | `git symbolic-ref HEAD` for branch; `git rev-parse HEAD` for SHA; existing `std::process::Command` pattern |
| BROWSE-03 | `gf browse <file>` opens file in forge file view URL | Per-forge `/blob/` vs `/src/branch/` path patterns; `git rev-parse --show-toplevel` for absolute path normalization |
| BROWSE-04 | `--branch <name>` overrides detected branch | Clap `Arg::new("branch")` with `.long("branch")`; straightforward flag passthrough to URL builder |
| BROWSE-05 | Browse URL construction implemented natively (no CLI delegation) | Entire browse handler bypasses adapter/runner; new code path in `main.rs` before forge detection block |

</phase_requirements>

## Summary

Phase 4 implements `gf browse` as a fully native command — it constructs forge-specific URLs locally from the git remote and opens them in the system browser. No forge CLI is invoked. The implementation requires three distinct capabilities: parsing the git remote URL into (host, owner, repo) parts, resolving the current branch or commit SHA from git, and opening a constructed URL in the system browser.

All design decisions are locked from the CONTEXT.md discussion. The primary technical work is: (1) adding `parse_remote_parts()` to `src/forge/mod.rs`, (2) creating a browse handler (either `src/browse/mod.rs` or a function in `src/cmd/`), (3) wiring the `browse` subcommand in `main.rs` before the forge-detection-plus-delegation block, and (4) adding the `webbrowser` crate to `Cargo.toml`.

The `webbrowser` crate (v1.2.0) provides a three-function API. `webbrowser::open(url)` returns `Result<Output, std::io::Error>`. The browse subcommand is structurally similar to the existing `completions` handler — it intercepts early in `main()` before the standard forge detection + adapter + runner path.

**Primary recommendation:** Implement browse as `src/browse/mod.rs` exposing a single `run(matches)` function, keeping forge-specific logic (URL format dispatch) co-located with the browse concern rather than adding to `src/forge/`.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| webbrowser | 1.2.0 | Cross-platform browser launch | Solves xdg-open/open/start platform differences; `open()` returns Result; confirmed current on docs.rs |
| clap (builder API) | 4.x | CLI argument parsing | Already in use; browse subcommand added to existing `build_cli()` |
| thiserror | 2.x | Error enum variants | Already in use; add `BrowseFailed`, `BrowseUrlConstructionFailed` variants to `GfError` |
| std::process::Command | stdlib | Git queries (branch, SHA, toplevel) | Already established pattern for all git operations in this codebase |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none new beyond webbrowser) | — | — | — |

**Installation:**
```bash
# Add to Cargo.toml [dependencies]:
webbrowser = "1"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── browse/
│   └── mod.rs       # browse handler: URL construction + git queries + open
├── forge/
│   └── mod.rs       # add parse_remote_parts() here (extends existing parse_host pattern)
├── cmd/
│   └── mod.rs       # add build_browse() and register in build_cli()
├── error.rs         # add BrowseFailed, BrowseUrlConstructionFailed variants
└── main.rs          # handle "browse" subcommand before forge-detection block
```

Rationale for `src/browse/mod.rs`: browse logic is non-trivial (git queries, URL dispatch, browser open) and does not belong in the existing adapter/runner path. A dedicated module keeps `main.rs` thin and makes unit testing the URL construction functions straightforward without spawning processes.

### Pattern 1: Early Subcommand Intercept in main.rs

Browse is handled like `completions` — matched before forge detection, because browse does its own git + forge detection internally.

```rust
// src/main.rs — after completions handler, before forge::detect()
if let Some(("browse", sub)) = matches.subcommand() {
    if let Err(e) = browse::run(sub) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    return;
}
```

### Pattern 2: parse_remote_parts() in src/forge/mod.rs

Extends the existing `parse_host()` pattern. Returns `(host, owner, repo)` as a tuple or dedicated struct.

```rust
/// Parses HTTPS or SCP-style remote URLs into (host, owner, repo).
/// Strips .git suffix from repo name.
/// Examples:
///   "https://github.com/alice/myrepo.git" → ("github.com", "alice", "myrepo")
///   "git@gitlab.com:alice/myrepo.git"     → ("gitlab.com", "alice", "myrepo")
pub fn parse_remote_parts(url: &str) -> Result<(String, String, String), GfError> {
    // HTTPS: strip scheme, split on '/', take [0]=host [1]=owner [2]=repo
    // SCP:   strip [user@], split host:path on ':', split path on '/', take [0]=owner [1]=repo
}
```

### Pattern 3: Per-Forge URL Construction

Dispatch on `ForgeType` for file URL path format. Repo URL is `https://{base}/{owner}/{repo}`. File URL path segment differs by forge:

```
GitHub/GitLab (public): https://{base}/{owner}/{repo}/blob/{ref}/{path}
  — GitLab adds "/-/" prefix: /blob/ becomes /-/blob/
Gitea/Forgejo:          https://{base}/{owner}/{repo}/src/branch/{ref}/{path}
  — For commit SHA (detached HEAD): /src/commit/{sha}/{path}
```

Note: Gitea and Forgejo use `src/commit/<sha>` when the ref is a full SHA (detached HEAD), not `src/branch/<sha>`. GitHub and GitLab accept a SHA directly in the branch position of the URL.

### Pattern 4: Branch / SHA Resolution

```rust
// Get current branch (fails if detached HEAD)
fn get_current_branch() -> Result<String, GfError> {
    // git symbolic-ref --short HEAD
    // Returns branch name on success, Err on detached HEAD
}

// Get current HEAD SHA (fallback for detached HEAD)
fn get_head_sha() -> Result<String, GfError> {
    // git rev-parse HEAD
    // Returns full 40-char SHA
}

// Resolve ref to use in URL: branch name or full SHA
fn resolve_ref(branch_override: Option<&str>) -> Result<(String, bool), GfError> {
    // bool = is_sha (affects Gitea/Forgejo URL path segment)
    if let Some(b) = branch_override { return Ok((b.to_string(), false)); }
    match get_current_branch() {
        Ok(branch) => Ok((branch, false)),
        Err(_) => Ok((get_head_sha()?, true)),
    }
}
```

### Pattern 5: Absolute Path Normalization

```rust
fn normalize_path(path: &str) -> Result<String, GfError> {
    if path.starts_with('/') {
        let toplevel = get_repo_toplevel()?; // git rev-parse --show-toplevel
        let stripped = path.strip_prefix(&toplevel)
            .ok_or_else(|| GfError::BrowsePathOutsideRepo(path.to_string()))?;
        Ok(stripped.trim_start_matches('/').to_string())
    } else {
        Ok(path.to_string())
    }
}
```

### Anti-Patterns to Avoid

- **Delegating to gh browse / glab browse / tea browse**: Explicitly forbidden (BROWSE-05). Tea's browse is broken. Build the URL natively.
- **Using forge API calls**: No HTTP requests. URL is derived entirely from the git remote string and git commands.
- **Validating that paths exist locally**: Explicitly out of scope. User may browse deleted/remote-only paths.
- **Using short SHAs**: Use full 40-char SHA for detached HEAD (unambiguous on all forges, short SHAs can collide).
- **Placing browse logic in adapter/runner**: Browse never goes through the adapter layer or runner. It is a self-contained command.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-platform browser launch | Custom xdg-open/open/start logic | `webbrowser::open()` | Handles macOS `open`, Linux `xdg-open`, Windows `start`, headless detection edge cases |

**Key insight:** The `webbrowser` crate handles platform detection (macOS, Linux, Windows, WSL) and subprocess management. A hand-rolled approach would require `#[cfg(target_os)]` branching and testing on each platform.

## Common Pitfalls

### Pitfall 1: Gitea/Forgejo SHA vs Branch URL Segment
**What goes wrong:** Using `src/branch/<sha>` for detached HEAD on Gitea/Forgejo returns a 404. These forges use `src/commit/<sha>` for commit references.
**Why it happens:** GitHub/GitLab accept a SHA in the branch position; Gitea/Forgejo distinguish between branch refs and commit refs in the URL path.
**How to avoid:** Track whether the resolved ref is a SHA (`is_sha: bool`) and use `src/commit/<sha>` vs `src/branch/<name>` accordingly for Gitea/Forgejo.
**Warning signs:** 404 in browser for Gitea/Forgejo repos when on detached HEAD.

### Pitfall 2: GitLab's `/-/` URL Infix
**What goes wrong:** GitLab file URLs require `/-/blob/<branch>/<path>` not `/blob/<branch>/<path>`. Without the `/-/`, the URL is invalid.
**Why it happens:** GitLab namespacing: the `/-/` prefix disambiguates repository paths from group/project namespaces.
**How to avoid:** GitLab branch is `"gitlab.com" => format!("https://{base}/{owner}/{repo}/-/blob/{ref}/{path}")`.

### Pitfall 3: `.git` Suffix Not Stripped
**What goes wrong:** `parse_remote_parts()` returns `"repo.git"` as the repo name, producing invalid URLs.
**How to avoid:** Strip `.git` suffix in `parse_remote_parts()` before returning. Use `.strip_suffix(".git").unwrap_or(name)`.

### Pitfall 4: SCP URL Owner Parsing
**What goes wrong:** `git@github.com:owner/repo.git` — if you split on `:` first and then `/`, you must handle that the path segment after `:` is `owner/repo.git`, not `host/owner/repo.git`.
**How to avoid:** After stripping `[user@]host:`, split the remaining path by `/` to get `[owner, repo]`.

### Pitfall 5: browse Subcommand Needs Its Own Forge Detection
**What goes wrong:** Wiring browse after `forge::detect(remote)` in `main.rs` causes forge detection to run even when browse doesn't need the adapter/runner path. This is fine functionally but the existing `detect()` only returns `ForgeType` — browse also needs `host`, `owner`, `repo`. Browse must call `get_remote_url` + `parse_remote_parts` independently.
**How to avoid:** Browse handler calls `parse_remote_parts(get_remote_url(remote)?)` directly. It does NOT reuse `forge::detect()` (which discards owner/repo). It does use `ForgeType` from `match_known_host()` or config for URL format dispatch.

### Pitfall 6: Clap `browse` Subcommand Missing `b` Alias
**What goes wrong:** CORE-08 specifies `b` as the alias for `browse`. Missing it from `build_browse()` fails the requirement.
**How to avoid:** Add `.visible_alias("b")` to the browse subcommand definition.

## Code Examples

### webbrowser::open() usage
```rust
// Source: https://docs.rs/webbrowser/1.2.0/webbrowser/
use webbrowser;

fn open_url(url: &str) -> Result<(), GfError> {
    println!("{url}");  // always print URL first (like gh browse behavior)
    webbrowser::open(url).map_err(|e| GfError::BrowseFailed(url.to_string(), e))?;
    Ok(())
}
```

### webbrowser return type
`webbrowser::open(url: &str) -> Result<std::process::Output, std::io::Error>`

The error is a plain `std::io::Error`, directly mappable to a `GfError` variant.

### Clap browse subcommand
```rust
fn build_browse() -> Command {
    Command::new("browse")
        .about("Open repo, branch, or file in browser (alias: b)")
        .visible_alias("b")  // CORE-08
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .required(false)
                .help("File path to open (optional)"),
        )
        .arg(
            Arg::new("branch")
                .long("branch")
                .short('b')  // note: conflicts with 'b' alias — use long only or different short
                .value_name("BRANCH")
                .help("Branch to use instead of current"),
        )
        .arg(
            Arg::new("no-browser")
                .long("no-browser")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Print URL without opening browser"),
        )
}
```

**Note on `-b` short flag:** The `b` single-letter alias is on the `browse` subcommand itself, not a flag. Using `-b` as a short for `--branch` is valid since aliases are subcommand-level, not flag-level. However, verify no clap conflict exists — if there is one, use `--branch` with no short form.

### GfError variants to add
```rust
#[error("failed to open browser for {0}: {1}")]
BrowseFailed(String, std::io::Error),

#[error("cannot construct browse URL: {0}")]
BrowseUrlConstructionFailed(String),
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| webbrowser 0.x | webbrowser 1.x | 2023 | Stable API, same `open()` function signature |

No significant ecosystem shifts relevant to this phase.

## Open Questions

1. **Short flag `-b` for `--branch` vs `b` alias on `browse` subcommand**
   - What we know: Clap distinguishes subcommand aliases from flag short forms; they don't conflict at the clap level
   - What's unclear: Whether `-b` feels ergonomic given `browse` itself is aliased as `b`
   - Recommendation: Use `--branch` with no short form to avoid user confusion (typing `gf b -b main` is awkward). Claude's discretion.

2. **`parse_remote_parts()` visibility**
   - What we know: Must be `pub` to be called from `src/browse/mod.rs`
   - What's unclear: Whether to expose `RemoteParts` as a named struct or return a tuple
   - Recommendation: Named struct `RemoteParts { host, owner, repo }` is more readable than a 3-tuple and enables adding fields later. Claude's discretion.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + assert_cmd 2.x + predicates 3.x |
| Config file | none — `cargo test` discovers tests automatically |
| Quick run command | `cargo test browse` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BROWSE-01 | `gf browse` opens repo URL for each of the 4 forges | unit (URL construction) | `cargo test browse::tests` | ❌ Wave 0 |
| BROWSE-01 | `gf browse` with `--no-browser` prints URL, no error | integration | `cargo test test_browse_no_browser_prints_url` | ❌ Wave 0 |
| BROWSE-02 | URL uses current branch name | unit | `cargo test browse::tests::test_resolve_ref_branch` | ❌ Wave 0 |
| BROWSE-02 | Detached HEAD falls back to full 40-char SHA | unit | `cargo test browse::tests::test_resolve_ref_detached` | ❌ Wave 0 |
| BROWSE-03 | `gf browse path/to/file.rs` generates correct file URL per forge | unit | `cargo test browse::tests::test_file_url_` | ❌ Wave 0 |
| BROWSE-03 | Absolute path normalized to repo-relative | unit | `cargo test browse::tests::test_normalize_path` | ❌ Wave 0 |
| BROWSE-04 | `--branch main` overrides detected branch in URL | unit | `cargo test browse::tests::test_branch_override` | ❌ Wave 0 |
| BROWSE-05 | No `gh`/`glab`/`tea`/`fj` subprocess spawned during browse | integration (PATH isolation) | `cargo test test_browse_no_forge_cli_spawned` | ❌ Wave 0 |

**Note on BROWSE-01 integration test:** Use `--no-browser` flag in all integration tests to avoid opening an actual browser in CI. The `webbrowser::open()` call itself is tested via unit test with a mock or skipped (CI headless environments will cause `open()` to fail non-fatally).

### Sampling Rate
- **Per task commit:** `cargo test browse`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/browse/mod.rs` — module file must exist before unit tests compile
- [ ] `src/browse/mod.rs` — unit tests for `build_repo_url()`, `build_file_url()`, `parse_remote_parts()`, `resolve_ref()`, `normalize_path()`
- [ ] Integration test stubs in `tests/integration_test.rs` — `test_browse_no_browser_prints_url`, `test_browse_no_forge_cli_spawned`
- [ ] `webbrowser = "1"` in `Cargo.toml` — required before `src/browse/mod.rs` compiles

## Sources

### Primary (HIGH confidence)
- https://docs.rs/webbrowser/latest/webbrowser/ — API surface, version 1.2.0, `open()` signature
- `src/forge/mod.rs` (in-repo) — `parse_host()` pattern, `get_remote_url()`, `ForgeType` enum
- `src/cmd/mod.rs` (in-repo) — `build_cli()` builder API pattern, alias conventions
- `src/error.rs` (in-repo) — `GfError` enum, existing variants
- `src/main.rs` (in-repo) — early-intercept pattern for completions subcommand

### Secondary (MEDIUM confidence)
- GitHub file URL format: `github.com/owner/repo/blob/<ref>/<path>` — widely documented, HIGH confidence
- GitLab `/-/blob/` infix — documented in GitLab URL structure, MEDIUM (verified by direct observation; official docs do not call this out prominently)
- Gitea `src/branch/` vs `src/commit/` — MEDIUM confidence (observed from Gitea/Codeberg repos; should be validated during implementation against a live Gitea instance)
- Forgejo `src/branch/` — mirrors Gitea (Forgejo is a Gitea fork); MEDIUM confidence

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `webbrowser` crate version confirmed on docs.rs; all other dependencies already in Cargo.toml
- Architecture: HIGH — locked decisions from CONTEXT.md; patterns derived directly from existing codebase
- URL formats: MEDIUM — GitHub/GitLab formats are well-known; Gitea `src/commit` vs `src/branch` is the one area to validate during implementation
- Pitfalls: HIGH — derived from URL format analysis and existing codebase patterns

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable domain; `webbrowser` API unlikely to change)
