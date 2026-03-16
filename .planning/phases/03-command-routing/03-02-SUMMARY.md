---
phase: 03-command-routing
plan: 02
subsystem: adapter
tags: [rust, clap, pr, mr, flag-translation, forgejo, gitea, gitlab, github]

# Dependency graph
requires:
  - phase: 03-command-routing/03-01
    provides: clap CLI tree with pr/mr subcommands and adapter module skeleton
provides:
  - translate_pr() with full subcommand name + flag translation for all four forges
  - PR-01 through PR-06 requirements fully satisfied with 24 unit tests
affects: [main.rs wiring, end-to-end integration tests]

# Tech tracking
tech-stack:
  added: []
  patterns: [per-forge flag translation table in match arms, positional optional arg for PR number]

key-files:
  created: []
  modified:
    - src/adapter/pr.rs
    - src/cmd/mod.rs

key-decisions:
  - "pr view number arg changed from named --number flag to positional arg to match CLI semantics (gf pr view 42, not gf pr view --number 42)"

patterns-established:
  - "Pattern 1: forge flag translation via match on ForgeType inside translate_* functions, not a data table"
  - "Pattern 2: passthrough args collected as 'extra' positional after -- and appended verbatim to translated args"

requirements-completed: [PR-01, PR-02, PR-03, PR-04, PR-05, PR-06]

# Metrics
duration: 5min
completed: 2026-03-16
---

# Phase 3 Plan 02: PR/MR Adapter Summary

**Per-forge flag translation layer for gf pr create/view: --body to --description (glab/tea), --base to --target-branch (glab), subcommand name mapping pr->mr (glab) and pr->pulls (tea), with full passthrough for unrecognized flags**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-16T15:04:00Z
- **Completed:** 2026-03-16T15:09:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Replaced stub translate_pr() with full implementation handling all four forges
- 24 unit tests covering PR-01 through PR-06 (subcommand names, flag translation, passthrough, view with/without number)
- Fixed pr view number arg to be positional (not --number named flag) so `gf pr view 42` works correctly

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement translate_pr() with full flag and subcommand translation** - `17d94b7` (feat)

**Plan metadata:** (final docs commit - see below)

## Files Created/Modified
- `src/adapter/pr.rs` - Full translate_pr(), translate_pr_create(), translate_pr_view() with 24 unit tests
- `src/cmd/mod.rs` - Fixed pr view number arg from named --number to positional

## Decisions Made
- Changed `pr view` number arg from `long("number").short('n')` to positional `value_name("NUMBER")` — the CLI syntax should be `gf pr view 42`, not `gf pr view --number 42`. This matches gh/glab conventions and the plan's own test expectations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed pr view number arg definition**
- **Found during:** Task 1 (implementing translate_pr_view tests)
- **Issue:** cmd/mod.rs defined number as `--number` named flag, but plan tests used `parse_pr_view(&["42"])` (positional). `gf pr view 42` is the intended syntax per research doc (PR-05).
- **Fix:** Changed `Arg::new("number").long("number").short('n')` to `Arg::new("number").value_name("NUMBER")` in build_pr() view subcommand
- **Files modified:** src/cmd/mod.rs
- **Verification:** All 24 tests pass including test_pr_view_with_number_github/gitlab
- **Committed in:** 17d94b7 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in CLI arg definition)
**Impact on plan:** Fix essential for correct CLI UX. No scope creep.

## Issues Encountered
None - compilation and tests passed on first attempt after the arg definition fix.

## Next Phase Readiness
- PR/MR translation complete for all four forges
- repo and auth translation (03-03) can proceed independently
- main.rs wiring (connecting forge detection + adapter + runner) is the final integration step

---
*Phase: 03-command-routing*
*Completed: 2026-03-16*
