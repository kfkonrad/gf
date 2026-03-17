# Phase 4: Browse - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Native `gf browse` command that constructs forge-specific URLs locally and opens them in the default browser. Covers: repo view, file view, branch override, detached HEAD fallback. No delegation to underlying CLIs (tea's browse is broken). No forge API calls — URL built from git remote.

</domain>

<decisions>
## Implementation Decisions

### URL Construction
- Extend `src/forge/` module with `parse_remote_parts()` returning host, owner, repo — single source of truth for all remote URL parsing
- `parse_remote_parts()` handles both HTTPS (`https://host/owner/repo.git`) and SCP SSH (`git@host:owner/repo.git`) formats in one function, stripping `.git` suffix
- Self-hosted forge base URL derived from remote host automatically (`https://` + host) — no config required; works for any self-hosted instance

### Browser Open
- Use `webbrowser` crate (solved problem, cross-platform)
- Always print URL to stdout when opening (useful for scripting, debugging — like `gh browse` behavior)
- Add `-n` / `--no-browser` flag for non-interactive/CI/headless use — prints URL without opening browser
- On browser open failure: print URL + error message, exit non-zero

### File URL Format (per-forge)
- GitHub:  `https://github.com/owner/repo/blob/<branch>/<path>`
- GitLab:  `https://gitlab.com/owner/repo/-/blob/<branch>/<path>`
- Gitea:   `https://gitea.com/owner/repo/src/branch/<branch>/<path>`
- Forgejo: `https://codeberg.org/owner/repo/src/branch/<branch>/<path>`
- Self-hosted instances use same path patterns with derived host
- No local path validation — user can browse paths that exist on remote but not locally (e.g., deleted files)
- Auto-convert absolute paths to repo-relative: run `git rev-parse --show-toplevel`, strip repo root prefix from absolute paths; relative paths passed as-is

### Detached HEAD Fallback
- Use full 40-char SHA (unambiguous, works on all forges; short SHAs can theoretically collide)
- Detect detached HEAD when `git symbolic-ref HEAD` fails; fall back to `git rev-parse HEAD`

### Error Handling
- Browser open failure: print URL + error message, exit non-zero (URL still visible for manual use)
- URL construction failure (no remote, can't parse): clear error, exit non-zero

### Claude's Discretion
- Module structure for browse (e.g., `src/browse/mod.rs` vs inline in `src/cmd/`)
- Exact error message wording
- How `webbrowser` crate is integrated (direct call vs thin wrapper)

</decisions>

<canonical_refs>
## Canonical References

No external specs — requirements are fully captured in decisions above and REQUIREMENTS.md.

### Requirements
- `.planning/REQUIREMENTS.md` §BROWSE — BROWSE-01 through BROWSE-05 define the full acceptance criteria for this phase

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/forge/mod.rs` `parse_host()` — existing HTTPS/SSH host extraction; `parse_remote_parts()` extends this pattern
- `src/forge/mod.rs` `get_remote_url()` — fetches raw remote URL via `git remote get-url`; browse reuses this
- `src/cmd/mod.rs` `build_cli()` — browse subcommand with `b` alias already scaffolded by Plan 03-01
- `src/adapter/mod.rs` `translate()` — browse does NOT use the adapter layer (native, not delegated)
- `src/error.rs` `GfError` — error enum to extend with browse-specific variants

### Established Patterns
- Git operations via `std::process::Command` (see `get_remote_url`, `detect`)
- Error propagation via `GfError` enum + `thiserror`
- All forge-specific logic lives in `src/forge/` or `src/adapter/`

### Integration Points
- `src/main.rs` — browse subcommand handled before forge detection (like completions), since it needs to do its own git + forge detection
- `src/forge/mod.rs` — `parse_remote_parts()` added here; `ForgeType` used for URL format dispatch
- `Cargo.toml` — add `webbrowser` crate dependency

</code_context>

<specifics>
## Specific Ideas

- Use `webbrowser` crate — it's a solved problem (user's exact words)
- `-n` / `--no-browser` flag for non-interactive use (user's specific addition to the flag design)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-browse*
*Context gathered: 2026-03-16*
