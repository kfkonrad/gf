---
phase: 06-browse-enhancements
plan: 02
subsystem: browse
tags: [rust, url-construction, line-anchors, deep-linking, forges]

# Dependency graph
requires:
  - phase: 06-01
    provides: browse module foundation with build_file_url, normalize_path, resolve_ref

provides:
  - LineRange struct for parsed line specs
  - split_file_and_line function for colon-suffix extraction
  - parse_line_spec function with zero/reversed/non-numeric error handling
  - line_fragment function with per-forge fragment format differences
  - Updated build_file_url with optional line_range parameter
  - run() updated to parse line spec before normalize_path

affects: [06-browse-enhancements, future browse tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-forge URL fragment differences: GitHub/Gitea/Forgejo use #L42-L55, GitLab uses #L42-55"
    - "Colon suffix extraction via rfind(':') on the raw file arg before normalize_path"
    - "Option<&LineRange> parameter appended to build_file_url for backward-compatible None callers"

key-files:
  created: []
  modified:
    - src/browse/mod.rs

key-decisions:
  - "LineRange is pub(crate) to satisfy private_interfaces lint while build_file_url remains pub"
  - "split_file_and_line strips trailing colon (path:) returning (path, None) — trailing colon treated as no spec"
  - "parse_line_spec rejects line 0 as invalid — lines are 1-indexed"

patterns-established:
  - "line_fragment dispatches on ForgeType for per-forge fragment differences"
  - "parse_line_spec uses split_once('-') to distinguish single-line from range"

requirements-completed: [BROWSE-01]

# Metrics
duration: 15min
completed: 2026-03-17
---

# Phase 6 Plan 02: Line-Range Deep-Linking Summary

**Per-forge line-anchor deep-linking via `gf browse file.rs:42` and `gf browse file.rs:42-55` using LineRange struct with GitHub/Gitea/Forgejo `#L42-L55` vs GitLab `#L42-55` fragment formats**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-17T10:00:00Z
- **Completed:** 2026-03-17T10:15:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Line spec parsing with clean error messages for zero, reversed, and non-numeric inputs
- Per-forge URL fragment construction (GitHub/Gitea/Forgejo vs GitLab differ on range separator)
- Backward-compatible `build_file_url` signature change (existing callers pass `None`)
- 20 new tests covering all behaviors plus corrected 5 existing tests with updated signature

## Task Commits

Each task was committed atomically:

1. **Task 1: Add line-range parsing and fragment construction with tests** - `c600aa8` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified
- `src/browse/mod.rs` - Added LineRange, split_file_and_line, parse_line_spec, line_fragment; updated build_file_url and run()

## Decisions Made
- `LineRange` made `pub(crate)` to resolve `private_interfaces` warning while keeping `build_file_url` as `pub`
- Trailing colon `"src/main.rs:"` treated as no line spec (path stripped of colon, `None` returned)
- Lines are 1-indexed: line 0 is rejected with a clear error

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed split_file_and_line trailing colon behavior**
- **Found during:** Task 1 (test execution)
- **Issue:** Original implementation returned `(raw, None)` for trailing colon, giving back `"src/main.rs:"` instead of `"src/main.rs"`
- **Fix:** Return `(path, None)` from the colon branch when rest is empty
- **Files modified:** src/browse/mod.rs
- **Verification:** `test_split_file_and_line_trailing_colon` passes
- **Committed in:** c600aa8 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Added #[derive(Debug)] and pub(crate) to LineRange**
- **Found during:** Task 1 (compilation)
- **Issue:** Compiler requires `Debug` on `LineRange` for `unwrap_err()` in tests; `private_interfaces` warning for pub function using private type
- **Fix:** Added `#[derive(Debug)]` and changed to `pub(crate)`
- **Files modified:** src/browse/mod.rs
- **Verification:** All tests compile and pass
- **Committed in:** c600aa8 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes were required for correct compilation and test execution. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## Next Phase Readiness
- Line-range deep-linking complete for all four forges
- BROWSE-01 requirement satisfied
- Ready for Phase 6 Plan 03 (next browse enhancement)

---
*Phase: 06-browse-enhancements*
*Completed: 2026-03-17*
