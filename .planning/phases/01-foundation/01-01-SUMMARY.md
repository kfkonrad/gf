---
phase: 01-foundation
plan: 01
subsystem: infra
tags: [rust, cargo, thiserror, which, assert_cmd, predicates]

# Dependency graph
requires: []
provides:
  - Compilable Rust binary crate with GfError enum (CliNotFound, ExecFailed, SpawnFailed)
  - runner::run() stub ready for Plan 02 implementation
  - Integration test stubs for CORE-06 and CORE-07 that compile via cargo test --no-run
  - exit_with helper binary for exit code propagation tests
affects: [02-implementation, all-phases]

# Tech tracking
tech-stack:
  added: [which=8.0.2, thiserror=2, assert_cmd=2, predicates=3]
  patterns: [thiserror-typed-errors, unix-exec-model-stub, assert_cmd-integration-tests]

key-files:
  created:
    - Cargo.toml
    - src/main.rs
    - src/error.rs
    - src/runner.rs
    - tests/integration_test.rs
    - tests/helpers/exit_with.rs
  modified: []

key-decisions:
  - "Use thiserror for GfError enum to keep error variants match-able in tests and future phases"
  - "nix dep gated under cfg(windows) only — Unix exec() path needs no signal re-raise"
  - "dead_code allow on error.rs to silence stub warnings while keeping public API clean"

patterns-established:
  - "Pattern 1: GfError variants use structured fields (CliNotFound) not just strings for testability"
  - "Pattern 2: runner::run() takes (&str, &[&str]) — no owned strings at call boundary"

requirements-completed: [CORE-06, CORE-07]

# Metrics
duration: 2min
completed: 2026-03-16
---

# Phase 01 Plan 01: Foundation Scaffold Summary

**Rust binary crate with typed GfError enum (thiserror), runner stub, and Wave 0 integration test stubs that compile cleanly against assert_cmd + predicates**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-16T11:03:27Z
- **Completed:** 2026-03-16T11:05:25Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Cargo.toml configured with which=8.0.2, thiserror=2, assert_cmd=2, predicates=3 and nix gated to cfg(windows)
- GfError enum with CliNotFound (structured fields), ExecFailed, and SpawnFailed variants
- runner::run() stub with correct signature that Plan 02 will implement
- Wave 0 integration test stubs for CORE-06 (3 tests) and CORE-07 (1 stub) all compile via cargo test --no-run

## Task Commits

Each task was committed atomically:

1. **Task 1: Cargo.toml, project skeleton, and GfError type** - `4ca8493` (feat)
2. **Task 2: Wave 0 test infrastructure — integration stubs and exit helper** - `0df7e59` (feat)

## Files Created/Modified
- `Cargo.toml` - Binary crate manifest with all deps and exit_with bin entry
- `src/main.rs` - Arg-parsing skeleton wiring mod error and mod runner
- `src/error.rs` - GfError enum with thiserror derives and #[allow(dead_code)]
- `src/runner.rs` - Stub run(cli, args) returning Ok(()) placeholder
- `tests/integration_test.rs` - CORE-06 and CORE-07 test stubs using assert_cmd
- `tests/helpers/exit_with.rs` - Helper binary that exits with given code for propagation tests

## Decisions Made
- Used `#[allow(dead_code)]` on error.rs to suppress unused-variant warnings on the public stub API rather than adding fake usage
- Removed unused `use error::GfError` import from main.rs (runner::run error type inferred, not named at call site)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused GfError import causing warning**
- **Found during:** Task 1 (cargo build verification)
- **Issue:** `use error::GfError` in main.rs was unused, generating a warning that violated zero-warnings requirement
- **Fix:** Removed the import; the error type flows through runner::run's return type without naming it in main
- **Files modified:** src/main.rs
- **Verification:** cargo build exits 0 with no warnings
- **Committed in:** 4ca8493

**2. [Rule 1 - Bug] Added #[allow(dead_code)] to error.rs to silence variant warnings**
- **Found during:** Task 1 (cargo build verification)
- **Issue:** GfError variants are public stubs with no current usage, triggering dead_code warnings
- **Fix:** Added module-level #![allow(dead_code)] to error.rs
- **Files modified:** src/error.rs
- **Verification:** cargo build exits 0 with no warnings
- **Committed in:** 4ca8493

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug fixes for zero-warnings requirement)
**Impact on plan:** Both fixes essential to meet the "cargo build with zero warnings" must-have truth. No scope creep.

## Issues Encountered
None beyond the warning suppressions above.

## Next Phase Readiness
- Compilation baseline established; Plan 02 can implement runner::run() against existing error types and test stubs
- CORE-06 and CORE-07 integration tests will fail at runtime until Plan 02 adds the actual which-based PATH detection and exec() call
- No blockers

---
*Phase: 01-foundation*
*Completed: 2026-03-16*
