---
phase: 01-foundation
plan: 02
subsystem: core
tags: [rust, which, exec, subprocess, exit-code, integration-tests]

# Dependency graph
requires:
  - phase: 01-foundation/01-01
    provides: GfError enum, runner stub, exit_with helper binary, test infrastructure

provides:
  - Full runner::run() implementation with which::which() PATH detection
  - Unix process replacement via CommandExt::exec()
  - Windows spawn() with exit code propagation
  - CliInfo struct and cli_info() lookup function in error.rs
  - 7 passing integration tests covering CORE-06 and CORE-07
  - 1 unit test for error display format

affects: [phase-02-forge-detection, all future phases calling runner::run]

# Tech tracking
tech-stack:
  added: [which crate (PATH detection)]
  patterns:
    - "PATH check before exec/spawn using which::which()"
    - "cfg(unix) / cfg(windows) for platform-specific process handling"
    - "CommandExt::exec() for TTY-transparent process replacement on Unix"

key-files:
  created: []
  modified:
    - src/runner.rs
    - src/error.rs
    - tests/integration_test.rs
    - .planning/phases/01-foundation/01-VALIDATION.md

key-decisions:
  - "CliInfo.brew_name uses String (not &'static str) to allow unknown CLI names from &str input"
  - "cli_info() returns sensible fallback for unknown CLIs pointing to forge+cli GitHub search"

patterns-established:
  - "Pattern 1: Unix exec() path — process is replaced, TTY/signals transparent, no parent remains"
  - "Pattern 2: Integration tests use assert_cmd::cargo::cargo_bin() to resolve helper binary paths"

requirements-completed: [CORE-06, CORE-07]

# Metrics
duration: 3min
completed: 2026-03-16
---

# Phase 1 Plan 2: Subprocess Runner Summary

**which::which() PATH detection + Unix exec() process replacement + Windows spawn() with exit code propagation, verified by 8 passing tests**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-16T12:08:54Z
- **Completed:** 2026-03-16T13:07:41Z
- **Tasks:** 3 of 3 completed
- **Files modified:** 4

## Accomplishments
- runner::run() fully implemented: which::which() for PATH detection, exec() on Unix (process replacement), spawn() on Windows (exit code propagation)
- CliInfo struct and cli_info() lookup in error.rs with mappings for gh, glab, tea, fj, and unknown CLI fallback
- 7 integration tests green: 4 CORE-06 (not found, format, URL, no ANSI) and 3 CORE-07 (exit codes 0, 2, 42)
- 1 unit test verifying exact GfError::CliNotFound display format
- VALIDATION.md updated: nyquist_compliant=true, wave_0_complete=true, automated tests marked green

## Task Commits

1. **Task 1: Implement runner.rs** - `bd485ee` (feat)
2. **Task 2: Complete integration tests** - `79a647f` (test)
3. **Task 3: TTY/signal checkpoint** - `2dc86d1` (chore — human-verified: exit 130 on Ctrl+C, color passthrough confirmed)

## Files Created/Modified
- `src/runner.rs` - Full runner implementation with PATH check, Unix exec(), Windows spawn()
- `src/error.rs` - Added CliInfo struct and cli_info() lookup function
- `tests/integration_test.rs` - Complete CORE-06 and CORE-07 integration tests (7 tests)
- `.planning/phases/01-foundation/01-VALIDATION.md` - Updated nyquist_compliant, wave_0_complete, test statuses

## Decisions Made
- `CliInfo.brew_name` field uses `String` instead of `&'static str` to handle unknown CLI names derived from user input `&str`. This was a minor type adjustment discovered during compilation (Rule 1 auto-fix).
- cli_info() fallback for unknown CLIs uses the input string as brew_name, pointing to `https://github.com/search?q=forge+cli` as URL. Reasonable default without hardcoding every possible CLI.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CliInfo lifetime error — brew_name field type**
- **Found during:** Task 1 (implementing runner.rs)
- **Issue:** Plan specified `pub brew_name: &'static str` in CliInfo struct, but the `other =>` match arm in cli_info() takes a `&str` input which doesn't satisfy `'static`. Compiler error.
- **Fix:** Changed `brew_name` field from `&'static str` to `String`. Updated all known-CLI branches to use `.to_string()`. Updated runner.rs to use `info.brew_name` directly (already a String, no `.to_string()` needed).
- **Files modified:** src/error.rs, src/runner.rs
- **Verification:** `cargo build` exits 0
- **Committed in:** bd485ee (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - type/lifetime bug)
**Impact on plan:** Necessary fix for compilation. No behavior change, no scope creep.

## Issues Encountered
None beyond the CliInfo lifetime issue described above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- runner::run() is complete and stable. Phase 2 forge detection can call `runner::run(detected_cli, args)` and rely on transparent delegation.
- TTY/signal human verification complete: Ctrl+C on `gf sleep 30` produced exit 130; color output passed through identically.
- Phase 2 forge detection can call `runner::run(detected_cli, args)` and rely on transparent delegation.

---
*Phase: 01-foundation*
*Completed: 2026-03-16*
