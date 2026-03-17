---
phase: 04-browse
plan: 01
subsystem: browse
tags: [rust, webbrowser, clap, url-construction, forge, github, gitlab, gitea, forgejo]

requires:
  - phase: 03-command-routing
    provides: "cmd/mod.rs with build_cli(), forge module with ForgeType and detect(), GfError enum"

provides:
  - "webbrowser = 1 dependency in Cargo.toml"
  - "GfError::BrowseFailed and BrowseUrlConstructionFailed error variants"
  - "pub fn parse_remote_parts() in forge/mod.rs — parses HTTPS/SCP URLs into (host, owner, repo)"
  - "build_browse() CLI subcommand with visible_alias('b') in cmd/mod.rs"
  - "src/browse/mod.rs — full browse implementation with URL construction and git queries"
  - "browse subcommand wired in main.rs via early-intercept before forge::detect()"

affects: [04-browse-plan-02, integration-tests]

tech-stack:
  added: [webbrowser = "1"]
  patterns:
    - "Early-intercept pattern in main.rs — browse handles its own forge detection before global forge::detect()"
    - "URL construction functions take forge type + parts as pure functions — fully unit-testable without git"
    - "parse_remote_parts() in forge module extends parse_host() pattern to extract (host, owner, repo)"

key-files:
  created:
    - src/browse/mod.rs
  modified:
    - Cargo.toml
    - src/error.rs
    - src/forge/mod.rs
    - src/cmd/mod.rs
    - src/lib.rs
    - src/main.rs

key-decisions:
  - "resolve_forge_type() in browse module replicates known-host match rather than calling private forge::match_known_host() — avoids making private functions public; self-hosted instances return ForgeNotDetected with TOML config hint"
  - "mod browse declared in both lib.rs (pub mod browse) and main.rs (mod browse) — bin and lib are separate crates sharing source; this is the existing pattern for all modules"
  - "normalize_path() passes relative paths through unchanged; only strips repo root prefix for absolute paths — no filesystem validation since paths may only exist on remote"

patterns-established:
  - "browse::run() matches early-intercept pattern after completions handler and before forge::detect()"
  - "URL construction: build_repo_url() and build_file_url() are pure functions taking ForgeType enum — easy to test and extend"

requirements-completed: [BROWSE-01, BROWSE-02, BROWSE-03, BROWSE-04, BROWSE-05]

duration: 15min
completed: 2026-03-16
---

# Phase 4 Plan 1: Browse Foundation Summary

**Native browse command with per-forge URL construction (GitHub/GitLab/-/ infix/Gitea+Forgejo src/branch vs src/commit), webbrowser dependency, and early-intercept wiring in main.rs**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-16T16:00:00Z
- **Completed:** 2026-03-16T16:15:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Full browse module with correct URL patterns for all four supported forges including GitLab's `/-/` infix and Gitea/Forgejo's `src/commit/` vs `src/branch/` distinction
- `parse_remote_parts()` added to forge module handling HTTPS (with port), HTTP, and SCP-style (`git@host:owner/repo.git`) remote URLs
- 96 unit tests + 20 integration tests all pass GREEN

## Task Commits

1. **Task 1: webbrowser dep, GfError variants, parse_remote_parts, build_browse CLI** - `488b52f` (feat)
2. **Task 2: Implement src/browse/mod.rs** - `ca4f887` (feat)
3. **Task 3: Wire browse in main.rs** - `60671d4` (feat)

## Files Created/Modified
- `src/browse/mod.rs` - Full browse implementation: run(), build_repo_url(), build_file_url(), resolve_ref(), normalize_path(), git query helpers
- `Cargo.toml` - Added `webbrowser = "1"` dependency
- `src/error.rs` - Added BrowseFailed(String, #[source] io::Error) and BrowseUrlConstructionFailed(String)
- `src/forge/mod.rs` - Added pub fn parse_remote_parts() with HTTPS/SCP parsing and unit tests
- `src/cmd/mod.rs` - Added build_browse() with visible_alias("b"), registered in build_cli()
- `src/lib.rs` - Added pub mod browse
- `src/main.rs` - Added mod browse and early-intercept handler for "browse" subcommand

## Decisions Made
- `resolve_forge_type()` in browse module replicates the known-host match (github.com/gitlab.com/gitea.com/codeberg.org) rather than calling private `forge::match_known_host()`. Self-hosted instances get ForgeNotDetected with the existing TOML config hint.
- `mod browse` declared in both lib.rs and main.rs following the existing codebase pattern where the binary and library are separate Rust crates sharing source files.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness
- Plan 02 can wire integration tests and verify `gf browse --no-browser` produces correct URLs against live git repos
- Self-hosted forge config_lookup() integration for browse is deferred to a future improvement (noted in code comments)

---
*Phase: 04-browse*
*Completed: 2026-03-16*
