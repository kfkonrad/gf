---
phase: 08-pr-workflow-commands
plan: 03
subsystem: adapter
tags: [rust, clap, pr-review, pr-approve, glab-subcommand-remap, fj-positional-body]

requires:
  - phase: 08-02
    provides: translate_pr with list/checkout/merge arms

provides:
  - translate_pr_review function handling gh/glab/fj comment and approve paths with tea/fj unsupported errors
  - translate_pr_approve syntactic sugar producing identical output to review --approve
  - 5 un-ignored v11 review translation tests
  - 7 new tests (3 unsupported error tests + 4 pr approve tests)

affects: [08-04, 09-integration]

tech-stack:
  added: []
  patterns: [subcommand-remap for glab mr comment/approve, positional-body for fj pr comment, UnsupportedFeature hard-error for tea/fj review/approve]

key-files:
  created: []
  modified:
    - src/adapter/pr.rs
    - tests/flag_audit.rs

key-decisions:
  - "glab review --approve maps to mr approve <N> (subcommand remap, flag removed)"
  - "glab review --comment maps to mr comment <N> --message (subcommand remap + flag remap)"
  - "fj review --comment maps to pr comment <N> <body> (subcommand remap, body positional)"
  - "tea and fj return UnsupportedFeature for both review --comment and review --approve"
  - "pr approve is pure syntactic sugar — identical code path to review --approve"

patterns-established:
  - "Subcommand remap: translate_pr_review inspects forge and emits different second arg (review vs comment vs approve)"
  - "Positional body: fj comment body appended without flag name"

requirements-completed: [PR-04, PR-05]

duration: 10min
completed: 2026-03-18
---

# Phase 8 Plan 03: PR Review and Approve Translation Summary

**glab subcommand-remapped mr comment/approve and fj positional-body pr comment, with UnsupportedFeature hard errors for tea/fj approve**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-18T10:00:00Z
- **Completed:** 2026-03-18T10:10:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Implemented `translate_pr_review` with full per-forge routing: gh passthrough, glab subcommand remap (mr comment/mr approve), fj positional body, tea/fj UnsupportedFeature errors
- Implemented `translate_pr_approve` as syntactic sugar producing identical output to review --approve for each forge
- Un-ignored 5 previously-ignored v11 review tests
- Added 7 new tests: 3 unsupported error tests and 4 pr approve tests
- 133 flag_audit tests pass, full suite green

## Task Commits

1. **Task 1: Implement translate_pr_review() and translate_pr_approve()** - `77fa1cc` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified
- `src/adapter/pr.rs` - Added review/approve arms in translate_pr dispatch, implemented translate_pr_review and translate_pr_approve
- `tests/flag_audit.rs` - Un-ignored 5 v11 review tests, added 3 unsupported + 4 approve tests

## Decisions Made
- glab review --approve maps to `mr approve <N>` (subcommand remap, --approve flag dropped entirely)
- glab review --comment maps to `mr comment <N> --message <text>` (both subcommand and flag remapped)
- fj review --comment maps to `pr comment <N> <body>` (subcommand remap, body becomes positional)
- tea returns UnsupportedFeature for all review/approve operations
- fj returns UnsupportedFeature for --approve (but supports --comment via pr comment)
- pr approve is pure syntactic sugar — no additional logic, same match arms as review --approve path

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None.

## Next Phase Readiness
- PR review and approve translations complete
- All v11 review tests now active
- Ready for Phase 8 Plan 04 (issue and repo clone translations)

---
*Phase: 08-pr-workflow-commands*
*Completed: 2026-03-18*
