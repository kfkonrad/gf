---
phase: 05-fix-self-hosted-browse
plan: 01
subsystem: forge
tags: [rust, browse, config-lookup, self-hosted, forge-detection]

requires:
  - phase: 04-browse
    provides: browse module with resolve_forge_type and native URL construction
  - phase: 02-forge-detection
    provides: config_lookup function reading ~/.config/gf/config.toml

provides:
  - browse::resolve_forge_type now calls forge::config_lookup before known-host table
  - pub fn config_lookup in src/forge/mod.rs
  - Two unit tests proving self-hosted config path and unknown-host fallback

affects:
  - any future plan touching browse URL construction or forge detection priority

tech-stack:
  added: []
  patterns:
    - "Config-first resolution: config_lookup before known-host match, consistent across forge::detect and browse::resolve_forge_type"

key-files:
  created: []
  modified:
    - src/forge/mod.rs
    - src/browse/mod.rs

key-decisions:
  - "config_lookup made pub (not re-implemented) — single source of truth for domain-to-forge mapping"
  - "resolve_forge_type mirrors forge::detect() priority: config first, known hosts second, ForgeNotDetected last"

patterns-established:
  - "All forge resolution paths (detect + resolve_forge_type) share the same priority order: config > known hosts > error"

requirements-completed:
  - CORE-05
  - BROWSE-01
  - BROWSE-02
  - BROWSE-03
  - BROWSE-04

duration: 5min
completed: 2026-03-16
---

# Phase 05 Plan 01: Fix Self-Hosted Browse Summary

**forge::config_lookup made pub and wired into browse::resolve_forge_type, closing the gap where browse ignored config.toml domain mappings**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-16T22:00:00Z
- **Completed:** 2026-03-16T22:05:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Made `config_lookup` pub in `src/forge/mod.rs` (one-line change)
- Updated `browse::resolve_forge_type` to call `config_lookup` before the known-host match, mirroring `forge::detect()` priority
- Removed stale NOTE comment (the future improvement is now done)
- Added two unit tests: self-hosted domain resolved via config.toml, unknown domain still returns ForgeNotDetected

## Task Commits

1. **Task 1: Wire config_lookup into browse** - `c9855da` (feat)
2. **Task 2: Add self-hosted browse unit tests** - `5eccff4` (test)

## Files Created/Modified

- `src/forge/mod.rs` - `fn config_lookup` changed to `pub fn config_lookup`
- `src/browse/mod.rs` - Updated use statement, replaced resolve_forge_type body, added two tests

## Decisions Made

- config_lookup made pub rather than duplicating logic — keeps domain-to-forge mapping as a single source of truth in the forge module

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- FINDING-02 from v1.0 audit is closed
- Self-hosted users with ~/.config/gf/config.toml entries can now use `gf browse`
- Phase 05 complete — no further plans in this phase

---
*Phase: 05-fix-self-hosted-browse*
*Completed: 2026-03-16*
