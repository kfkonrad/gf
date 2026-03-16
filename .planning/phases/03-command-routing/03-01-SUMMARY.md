---
phase: 03-command-routing
plan: 01
subsystem: cli
tags: [clap, clap_complete, rust, command-routing, aliases]

# Dependency graph
requires:
  - phase: 02-forge-detection
    provides: ForgeType enum and cli_name() used by adapter::translate dispatch key
provides:
  - clap 4 CLI command tree with full alias set (pr/mr, repo/r, auth/a, verb short aliases)
  - adapter module skeleton with translate() dispatcher
  - Parallel execution skeleton: Plans 02 and 03 can implement against stable interfaces
affects:
  - 03-command-routing/03-02 (implements adapter::pr)
  - 03-command-routing/03-03 (implements adapter::repo_auth)
  - 03-command-routing/03-04 (wires clap into main.rs)

# Tech tracking
tech-stack:
  added:
    - clap 4.6.x (builder API, derive feature)
    - clap_complete 4.6.x (shell completion generation, CORE-12)
  patterns:
    - Builder API (not derive) for precise alias visibility control
    - Visible aliases on subcommands (not hidden) so they appear in --help output
    - .last(true) + allow_hyphen_values for passthrough extra args
    - Stub modules with TODO(PlanN) comments as parallel execution contract

key-files:
  created:
    - src/cmd/mod.rs
    - src/adapter/mod.rs
    - src/adapter/pr.rs
    - src/adapter/repo_auth.rs
  modified:
    - Cargo.toml
    - src/main.rs

key-decisions:
  - "Use clap builder API (not derive macro) — gives precise control over alias display and visible_alias on subcommands"
  - "mr is a visible_alias on pr subcommand, not a separate top-level command — clap routes mr create to pr create automatically"
  - "Removed -h short from auth login hostname arg — conflicts with clap auto-generated --help short -h"
  - "pr view number arg changed from positional index(1) to --number flag — positional conflicted with last(true) extra arg"

patterns-established:
  - "Pattern 1: All forge-visible subcommand aliases use visible_alias() not alias() so they appear in --help (CORE-11)"
  - "Pattern 2: Passthrough extra args use .num_args(0..).allow_hyphen_values(true).last(true) in each subcommand"
  - "Pattern 3: Stub modules use let _ = (forge, matches) to suppress unused variable warnings while remaining compilable"

requirements-completed: [CORE-08, CORE-09, CORE-10, CORE-11, CORE-12]

# Metrics
duration: 15min
completed: 2026-03-16
---

# Phase 3 Plan 01: CLI Command Tree and Adapter Skeleton Summary

**clap 4 CLI tree with pr/mr, repo/r, auth/a aliases and one-letter verb aliases plus adapter module skeleton enabling parallel Plan 02/03 implementation**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-16T14:35:00Z
- **Completed:** 2026-03-16T14:50:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Added clap 4.6 and clap_complete 4.6 to Cargo.toml, verified compilation
- Created complete CLI tree in src/cmd/mod.rs with all required aliases (CORE-08 through CORE-12)
- Created adapter module skeleton (mod.rs, pr.rs, repo_auth.rs) with translate() dispatcher
- 10 unit tests all passing: alias tests, debug_assert validation, mr-resolves-to-pr routing test

## Task Commits

1. **Task 1: Add clap dependencies** - `a11f92c` (chore)
2. **Task 2: Create src/cmd/mod.rs** - `77f3a93` (feat)
3. **Task 3: Create src/adapter/ skeleton** - `6dbc0a7` (feat)

## Files Created/Modified
- `Cargo.toml` - Added clap 4 and clap_complete 4 dependencies
- `src/cmd/mod.rs` - build_cli() with full command tree and 10 unit tests
- `src/adapter/mod.rs` - translate() dispatcher to pr and repo_auth modules
- `src/adapter/pr.rs` - translate_pr() stub (Plan 02 fills this)
- `src/adapter/repo_auth.rs` - translate_repo() and translate_auth() stubs (Plan 03 fills these)
- `src/main.rs` - Added mod adapter and mod cmd declarations

## Decisions Made
- Used clap builder API instead of derive macro — builder gives direct control over visible_alias placement
- mr implemented as visible_alias on pr subcommand, so `gf mr create` routes to pr create handler automatically — no multi-word alias needed
- Removed `-h` short flag from auth login hostname to avoid conflict with clap's auto-generated `-h/--help`
- Changed pr view number from positional index(1) to `--number` flag to avoid conflict with `last(true)` extra args positional

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed positional argument conflict in pr view subcommand**
- **Found during:** Task 2 (cmd/mod.rs creation — test_cli_validates failure)
- **Issue:** `number` arg with `.index(1)` conflicted with `extra` arg using `.last(true)` — same positional index
- **Fix:** Changed `number` from positional to `--number` flag
- **Files modified:** src/cmd/mod.rs
- **Verification:** test_cli_validates passes, debug_assert() clean
- **Committed in:** 77f3a93 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed short flag conflict in auth login subcommand**
- **Found during:** Task 2 (cmd/mod.rs creation — test_cli_validates failure)
- **Issue:** `hostname` arg used `.short('h')` which conflicts with clap's auto-generated `-h/--help` short flag
- **Fix:** Removed `.short('h')` from hostname arg — hostname accessible via `--hostname` only
- **Files modified:** src/cmd/mod.rs
- **Verification:** test_cli_validates passes
- **Committed in:** 77f3a93 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 — bugs in plan's code template)
**Impact on plan:** Both fixes required for clap internal validation. No scope change.

## Issues Encountered
- Project is binary-only (no lib target) — `cargo test --lib` fails; tests run via `cargo test` against the bin target. All 10 cmd tests passed with `cargo test`.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plans 02 and 03 can now run in parallel — adapter/pr.rs and adapter/repo_auth.rs have stable function signatures
- Plan 04 can wire clap parsing into main.rs once 02 and 03 complete
- CORE-08 through CORE-12 all satisfied by this plan

## Self-Check: PASSED

All 4 created files found on disk. All 3 task commits verified in git history.

---
*Phase: 03-command-routing*
*Completed: 2026-03-16*
