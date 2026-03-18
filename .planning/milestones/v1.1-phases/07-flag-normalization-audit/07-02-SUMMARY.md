---
phase: 07-flag-normalization-audit
plan: 02
subsystem: testing
tags: [rust-macros, flag-translation, pre-mapping, integration-audit, v1.1-commands]

# Dependency graph
requires:
  - phase: 07-flag-normalization-audit/plan-01
    provides: translation_test!, audit_test! macros and forge_help_contains helper
provides:
  - 45 v1.1 pre-mapping translation tests documenting expected flag translations
  - 30 integration audit tests verifying v1.1 target forge flags exist
  - Complete UNSUPPORTED combination documentation (13 entries)
affects: [08-pr-commands, 09-issue-commands]

# Tech tracking
tech-stack:
  added: []
  patterns: [v11-translation-test-macro, pre-mapping-documentation-tests]

key-files:
  created: []
  modified:
    - tests/flag_audit.rs

key-decisions:
  - "v11_translation_test! macro reuses same API path as translation_test! but adds #[ignore]"
  - "All v1.1 tests are #[ignore]d — Phase 8 removes #[ignore] as adapters are implemented"
  - "UNSUPPORTED combos documented with inline comments, not test entries"
  - "glab --state uses boolean flags (--closed/--merged/--all) not --state value"
  - "glab mr approve is a subcommand, not a flag — documented in review section"

patterns-established:
  - "Pre-mapping pattern: write failing tests first, implement adapters against them"
  - "v11_translation_test! macro: identical to translation_test! but #[ignore]d"

requirements-completed: [QUAL-02, QUAL-03]

# Metrics
duration: 3min
completed: 2026-03-17
---

# Phase 7 Plan 02: v1.1 Pre-Mapping Translation Tests Summary

**45 ignored translation tests covering pr list/merge/checkout/review, issue list/view/create, repo clone across 4 forges, with 30 integration audit tests verifying target flags**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-17T13:15:16Z
- **Completed:** 2026-03-17T13:18:48Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Created `v11_translation_test!` macro (identical to `translation_test!` but `#[ignore]`d)
- Added 45 v1.1 pre-mapping translation test entries:
  - PR list (13): state×4, author×3, label×3 across gh/glab/tea/fj
  - PR merge (7): squash×4, rebase×2, merge-default×1
  - PR checkout (4): positional number across all forges
  - PR review (5): comment×3, approve×2 (comment+approve)
  - Issue list (7): state×4, label×3
  - Issue view (4): positional number across all forges
  - Issue create (3): title+body across gh/glab/tea
  - Repo clone (3): positional repo across gh/glab/fj
- Added 30 integration audit tests verifying target forge CLI flags exist in --help
- Documented 13 UNSUPPORTED combinations with explicit comments
- All 88 non-ignored tests pass; all 45 ignored tests fail as expected

## Task Commits

1. **Task 1: Add v1.1 pre-mapping translation tests and integration audit tests** - `3d6ba6e` (test)

## Files Modified

- `tests/flag_audit.rs` — +400 lines: v11_translation_test! macro, 45 translation tests, 30 audit tests, 13 UNSUPPORTED comments

## Key Mapping Highlights

| Command | gh | glab | tea | fj |
|---|---|---|---|---|
| pr list | pr list | mr list | pulls list | pr search |
| pr merge --squash | --squash | --squash | --style squash | --method squash |
| pr checkout | pr checkout | mr checkout | pulls checkout | pr checkout |
| pr review --comment | pr review --comment | mr comment --message | UNSUPPORTED | pr comment (positional) |
| pr review --approve | pr review --approve | mr approve (subcommand) | UNSUPPORTED | UNSUPPORTED |
| issue list | issue list | issue list | issues list | issue search |
| issue view | issue view | issue view | issues \<N\> | issue view |
| issue create --body | --body | --description | --description | N/A |
| repo clone | repo clone | repo clone | UNSUPPORTED | repo clone |

## Decisions Made

- `v11_translation_test!` macro embeds `#[ignore]` in the macro body so every invocation is automatically ignored
- glab state translation uses boolean flags (`--closed`, `--merged`, `--all`) instead of `--state value`
- glab mr approve is a separate subcommand, not a flag on mr review — adapter must route based on `--approve` presence
- fj pr comment uses positional body arg, not `--body` flag
- tea has no review/approve and no repo clone — these are hard UNSUPPORTED

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

- Phase 8 can implement adapters by making ignored tests pass one by one:
  1. Add CLI subcommands (pr list/merge/checkout/review, issue *, repo clone)
  2. Add adapter translation logic
  3. Remove `#[ignore]` from tests as each adapter is completed
- All integration audit tests confirm target forge flags exist in current CLI versions

---
*Phase: 07-flag-normalization-audit*
*Completed: 2026-03-17*

## Self-Check: PASSED
