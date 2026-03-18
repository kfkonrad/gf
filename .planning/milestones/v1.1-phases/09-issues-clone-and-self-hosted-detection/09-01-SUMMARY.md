---
phase: 09-issues-clone-and-self-hosted-detection
plan: 01
subsystem: cli
tags: [issue-management, clap, adapter-translation, per-forge-flags]

# Dependency graph
requires:
  - phase: 08-pr-merge-review-checkout
    provides: adapter translation pattern and translation_test! macro
provides:
  - Issue lifecycle commands (list, view, create, close, reopen) with per-forge translations
  - issue adapter module with flag normalization
  - build_issue() clap subcommands with alias support
affects: [09-02, 09-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Issue adapter mirrors PR adapter structure for consistency"
    - "UnsupportedFeature error for Forgejo issue reopen (CLI has no command)"
    - "Gitea uses 'issues' (plural) subcommand, others use 'issue' (singular)"

key-files:
  created:
    - src/adapter/issue.rs
  modified:
    - src/cmd/mod.rs
    - src/adapter/mod.rs
    - tests/flag_audit.rs

key-decisions:
  - "Gitea (tea) uses 'issues' (plural) subcommand consistently across all issue operations"
  - "Forgejo uses 'issue search' instead of 'issue list' for listing"
  - "GitLab --state maps to boolean flags (--closed/--all) not --state value"
  - "Forgejo issue reopen returns hard UnsupportedFeature error (CLI lacks command)"
  - "Tea issue view omits 'view' verb: 'tea issues 42' not 'tea issues view 42'"

patterns-established:
  - "Pattern: Issue adapter mirrors PR adapter structure (subcommand dispatcher → per-verb translators)"
  - "Pattern: Per-forge flag remapping (--body→--description, --label→--labels, --author→--creator)"
  - "Pattern: Verb remapping for Forgejo (search vs list) and Gitea (omit view)"

requirements-completed: [ISSUE-01, ISSUE-02, ISSUE-03, ISSUE-04, ISSUE-05]

# Metrics
duration: 3min
completed: 2026-03-18
---

# Phase 09 Plan 01: Issue Commands Summary

**Complete issue lifecycle management (list, view, create, close, reopen) with per-forge command/flag translations across GitHub, GitLab, Gitea, and Forgejo**

## Performance

- **Duration:** 3 minutes
- **Started:** 2026-03-18T13:24:38Z
- **Completed:** 2026-03-18T13:28:14Z
- **Tasks:** 3
- **Files modified:** 3 created, 3 modified

## Accomplishments

- Issue command clap subcommands with 5 verbs (list, view, create, close, reopen) and visible alias 'i'
- Issue adapter with complete per-forge translations for all 4 forges
- UnsupportedFeature error for Forgejo issue reopen (CLI has no reopen command)
- Enabled 21 v1.1 issue translation tests (all passing)
- Gitea plural 'issues' subcommand handling across all issue operations
- Flag remapping: --body→--description (glab/tea), --label→--labels (tea/fj), --author→--creator (fj)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add issue clap subcommands to CLI** - `3072876` (feat)
2. **Task 2: Create issue adapter with all translations** - `bf0d7bd` (feat)
3. **Task 3: Enable v1.1 issue translation tests** - `ba311f3` (test)

_Note: Task 2 was TDD-based (tests existed from Phase 9 Plan 0, implementation made them pass)_

## Files Created/Modified

- `src/adapter/issue.rs` - Issue command translation with per-forge verb and flag mappings
- `src/cmd/mod.rs` - build_issue() function with 5 subcommands
- `src/adapter/mod.rs` - Wire issue::translate_issue into dispatcher
- `tests/flag_audit.rs` - Convert v11_translation_test! to translation_test! for 21 issue tests

## Decisions Made

**Gitea plural subcommand:** Gitea uses "issues" (plural) consistently across all operations, not just list. This differs from all other forges which use "issue" (singular).

**Forgejo verb remapping:** Forgejo uses "issue search" instead of "issue list" for listing issues.

**GitLab state flags:** GitLab uses boolean flags (--closed, --all) instead of --state <value> pattern used by other forges.

**Tea view omission:** Gitea (tea) does not have "issues view" verb — uses `tea issues 42` directly without "view".

**Forgejo reopen unsupported:** Forgejo CLI has no "issue reopen" command. Returns hard UnsupportedFeature error as per Phase 8 unsupported feature policy.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Issue commands complete and tested
- Ready for Phase 9 Plan 2 (repo clone implementation)
- Adapter pattern established for clone command translations
- All flag_audit.rs issue tests passing

---
*Phase: 09-issues-clone-and-self-hosted-detection*
*Completed: 2026-03-18*
