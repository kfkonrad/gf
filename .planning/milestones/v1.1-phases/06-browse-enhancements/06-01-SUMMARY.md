---
phase: 06-browse-enhancements
plan: 01
subsystem: forge
tags: [rust, refactor, deduplication]

requires: []
provides:
  - "forge::match_known_host is public and is the single source of truth for known-host matching"
  - "browse::resolve_forge_type delegates to forge::match_known_host"
affects: [phase 7, phase 8, phase 9]

tech-stack:
  added: []
  patterns:
    - "Single-source-of-truth: known-host table lives only in forge::match_known_host; all callers delegate"

key-files:
  created: []
  modified:
    - src/forge/mod.rs
    - src/browse/mod.rs

key-decisions:
  - "Expose match_known_host as pub fn rather than re-exporting a closure to keep call sites idiomatic"

patterns-established:
  - "Known-host matching: add a host in forge::match_known_host only; browse picks it up automatically"

requirements-completed: [BROWSE-02]

duration: 5min
completed: 2026-03-17
---

# Phase 6 Plan 01: Deduplicate Known-Host Match Table Summary

**forge::match_known_host made public; browse::resolve_forge_type delegates to it, eliminating the duplicate four-arm match block**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-17T00:00:00Z
- **Completed:** 2026-03-17T00:05:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- `forge::match_known_host` is now `pub fn`, visible to sibling modules
- `browse::resolve_forge_type` removed its inline four-arm host match and calls `forge::match_known_host(host)` instead
- All 98 unit tests and 25 integration tests pass without any modification to test code

## Task Commits

Each task was committed atomically:

1. **Task 1: Make match_known_host public and deduplicate browse::resolve_forge_type** - `12226c9` (refactor)

## Files Created/Modified

- `src/forge/mod.rs` - Changed `fn match_known_host` to `pub fn match_known_host`
- `src/browse/mod.rs` - Added `match_known_host` to import; replaced duplicate match block with single delegation call

## Decisions Made

- Exposed `match_known_host` as a plain `pub fn` (not re-exported via a newtype or trait) — keeps call sites idiomatic and consistent with the existing private helper style.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Adding a new public forge host now requires exactly one edit in `src/forge/mod.rs`
- `browse::resolve_forge_type` is clean and ready for probe/cache extension in later plans

---
*Phase: 06-browse-enhancements*
*Completed: 2026-03-17*
