---
phase: 03-command-routing
plan: 04
subsystem: cli
tags: [clap, clap_complete, rust, integration-tests, alias-routing]

# Dependency graph
requires:
  - phase: 03-command-routing/03-02
    provides: clap CLI tree with build_cli() + all alias registrations
  - phase: 03-command-routing/03-03
    provides: ForgeAdapter translation layer with adapter::translate()
  - phase: 02-forge-detection
    provides: forge::detect() and ForgeType with cli_name()
provides:
  - "src/main.rs: clap-based main() with parse -> detect -> translate -> exec flow"
  - "src/lib.rs: library crate exposing all modules for integration tests"
  - "11 alias routing integration tests covering CORE-08 through CORE-12"
  - "Shell completions via gf completions bash/zsh (CORE-12)"
affects: [04-advanced-features, any future phases that modify CLI routing]

# Tech tracking
tech-stack:
  added: [clap_complete (generate function used in main and tests)]
  patterns: [parse-detect-translate-exec in main(), lib crate pattern for integration test access]

key-files:
  created:
    - src/lib.rs
  modified:
    - src/main.rs
    - Cargo.toml
    - tests/integration_test.rs

key-decisions:
  - "Integration tests use pr view instead of pr list — clap validates subcommands now, list was never registered"
  - "cli_cmd.clone().get_matches() needed to preserve cli_cmd for completions generation after parsing"

patterns-established:
  - "Completions handled before forge detection — no git repo needed for shell tab-complete generation"
  - "Vec<String> from adapter::translate() converted to Vec<&str> inline for runner::run() borrow"

requirements-completed: [CORE-08, CORE-09, CORE-10, CORE-11, CORE-12]

# Metrics
duration: 15min
completed: 2026-03-16
---

# Phase 3 Plan 04: Wire main.rs + Integration Tests Summary

**clap-based main() wired to parse/detect/translate/exec with 11 alias routing tests and shell completions for CORE-08 through CORE-12**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-16T17:07:04Z
- **Completed:** 2026-03-16T17:22:00Z
- **Tasks:** 3 of 3 (Task 3 human-verify checkpoint: approved)
- **Files modified:** 4

## Accomplishments
- Replaced hand-rolled while-loop arg parser in main.rs with full clap + adapter flow
- Added src/lib.rs exposing all modules so integration tests can use `gf::cmd::build_cli`
- Added [lib] section to Cargo.toml without disturbing existing [[bin]] entries
- 11 alias routing integration tests all pass (CORE-08 through CORE-12 coverage)
- Shell completions working: `gf completions bash` outputs `_gf()` bash function

## Task Commits

1. **Task 1: Replace main.rs hand-rolled parser with clap + adapter flow** - `b36a0d1` (feat)
2. **Task 2: Add alias routing integration tests** - `0510174` (feat)

## Files Created/Modified
- `src/main.rs` - Full replacement: clap parse -> forge detect -> adapter translate -> runner exec
- `src/lib.rs` - New library crate exposing adapter, cmd, error, forge, runner modules
- `Cargo.toml` - Added [lib] section with name="gf" path="src/lib.rs"
- `tests/integration_test.rs` - Appended 11 alias routing tests; fixed pr list -> pr view

## Decisions Made
- `cli_cmd.clone().get_matches()` pattern used so `cli_cmd` remains owned and mutable for completions generation after ArgMatches are obtained
- Integration test args changed from `pr list` to `pr view` — clap now validates subcommands before forge detection runs, and `list` was never registered in the clap tree

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed integration tests using unregistered `pr list` subcommand**
- **Found during:** Task 2 (alias routing tests)
- **Issue:** Existing Phase 1/2 integration tests used `args(["pr", "list"])` but clap now validates subcommands; `list` was never registered, causing clap to error before forge detection ran
- **Fix:** Changed all occurrences of `pr list` to `pr view` (a registered subcommand) in tests/integration_test.rs
- **Files modified:** tests/integration_test.rs
- **Verification:** `cargo test` exits 0 with 20 tests passing (was 14 passing / 6 failing)
- **Committed in:** 0510174 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** Required for full test suite green. The fix correctly adapts tests to the new strict clap parsing — `pr view` tests the same CORE-06/CORE-07 behavior as `pr list` did.

## Issues Encountered
None beyond the auto-fixed test update above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 3 implementation complete: clap CLI tree, ForgeAdapter translation, main.rs wiring, integration tests
- Human checkpoint (Task 3) approved: --help shows aliases, completions bash generates output, all tests green
- Phase 4 can proceed

---
*Phase: 03-command-routing*
*Completed: 2026-03-16*
