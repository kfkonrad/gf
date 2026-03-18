---
phase: 10-cleanup-dead-code-and-test-gaps
plan: 01
subsystem: testing
tags: [dead-code, compiler-warnings, flag-audit, forgejo, translation-tests]

# Dependency graph
requires:
  - phase: 09-issues-clone-and-self-hosted-detection
    provides: "Issue/clone adapters and Forgejo flag translations"
provides:
  - "Zero-warning release build"
  - "Complete Forgejo issue flag translation test coverage"
  - "Dead code removed (unused error variant, function, macro)"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: ["cfg_attr(not(windows), allow(dead_code)) for platform-specific variants"]

key-files:
  created: []
  modified:
    - src/error.rs
    - src/forge/mod.rs
    - src/browse/mod.rs
    - tests/flag_audit.rs

key-decisions:
  - "Added cfg_attr allow(dead_code) for SpawnFailed variant (Windows-only, not unused)"
  - "Changed LineRange from pub(crate) to pub — safe for binary crate, no type leak risk"

patterns-established:
  - "cfg_attr(not(target_os), allow(dead_code)) for platform-specific enum variants"

requirements-completed: [REPO-01, QUAL-02, QUAL-03, ISSUE-01]

# Metrics
duration: 4min
completed: 2026-03-18
---

# Phase 10 Plan 01: Dead Code Removal & Test Gap Closure Summary

**Zero-warning release build with dead code removed (CloneHostNotConfigured, get_default_clone_host, v11_translation_test!) and 3 new Forgejo issue flag tests**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-18T14:07:58Z
- **Completed:** 2026-03-18T14:11:58Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- `cargo build --release` produces zero warnings (was 2 warnings)
- Removed 3 dead code items: CloneHostNotConfigured variant, get_default_clone_host() function, v11_translation_test! macro
- Added translation_test for Forgejo issue list --author → --creator remap
- Added 2 audit_tests for fj issue search --labels and --creator flags
- Full test suite (25 tests) passes with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove dead code and fix visibility warning** - `1fce56c` (fix)
2. **Task 2: Delete v11_translation_test! macro** - `cb1365c` (chore)
3. **Task 3: Add missing translation_test and audit_test entries** - `26baacd` (test)

## Files Created/Modified
- `src/error.rs` - Removed CloneHostNotConfigured variant; added cfg_attr for SpawnFailed
- `src/forge/mod.rs` - Removed unused get_default_clone_host() function
- `src/browse/mod.rs` - Changed LineRange from pub(crate) to pub
- `tests/flag_audit.rs` - Removed v11_translation_test! macro; added 3 new test entries

## Decisions Made
- Added `#[cfg_attr(not(windows), allow(dead_code))]` on SpawnFailed variant since it's used on Windows only (src/runner.rs) — compiler warned after removing CloneHostNotConfigured
- Changed LineRange to `pub` rather than adding an allow attribute — binary crate has no type leak risk

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SpawnFailed dead_code warning after removing CloneHostNotConfigured**
- **Found during:** Task 1 (Remove dead code)
- **Issue:** After removing CloneHostNotConfigured, cargo build --release still had 1 warning for SpawnFailed (Windows-only variant)
- **Fix:** Added `#[cfg_attr(not(windows), allow(dead_code))]` above SpawnFailed as plan anticipated
- **Files modified:** src/error.rs
- **Verification:** `cargo build --release` zero warnings
- **Committed in:** 1fce56c (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — plan anticipated this edge case)
**Impact on plan:** Minimal — plan included guidance for this exact scenario.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Codebase is clean with zero warnings
- All Forgejo issue flag translations have test coverage
- v1.1 milestone audit gaps are closed
- Ready for final milestone wrap-up

---
*Phase: 10-cleanup-dead-code-and-test-gaps*
*Completed: 2026-03-18*
