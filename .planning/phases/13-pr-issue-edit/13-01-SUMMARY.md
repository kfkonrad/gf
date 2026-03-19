---
phase: 13-pr-issue-edit
plan: 01
title: "PR & Issue Edit Translation"
one_liner: "Validate-then-build edit translation for labels, reviewers, and assignees across 4 forges with per-flag UnsupportedFeature errors"
completed: "2026-03-19T10:46:00Z"
duration: "6m"
tasks_completed: 2
tasks_total: 2

dependency_graph:
  requires: [Phase 12 comment translation patterns, Phase 11 UnsupportedFeature 3-field variant]
  provides: [translate_pr_edit(), translate_issue_edit(), 56 new tests]
  affects: [src/cmd/mod.rs, src/adapter/pr.rs, src/adapter/issue.rs, tests/flag_audit.rs]

tech_stack:
  added: []
  patterns: [validate-then-build, subcommand routing (Forgejo labels), plural flag rename (Gitea), prefix semantics (GitLab +/-)]

key_files:
  created: []
  modified:
    - src/cmd/mod.rs
    - src/adapter/pr.rs
    - src/adapter/issue.rs
    - tests/flag_audit.rs

decisions:
  - "Used validate-then-build pattern: check all flags against forge capabilities before constructing args"
  - "Forgejo PR edit uses subcommand routing (labels --add/--rm) — only labels supported, reviewer/assignee unsupported"
  - "Gitea PR edit entirely unsupported (tea has no pulls edit command)"
  - "Gitea issue edit uses plural flag names (--add-labels, --remove-labels, --add-assignees)"
  - "GitLab uses update verb (not edit) and prefix semantics (+alice/-alice) for reviewer/assignee"
  - "22 audit tests (not 23 as originally estimated) — accurate count from plan's own spec"

metrics:
  duration: "6m"
  completed: "2026-03-19"
  test_count_before: 381
  test_count_after: 437
  new_tests: 56
  files_changed: 4
---

# Phase 13 Plan 01: PR & Issue Edit Translation Summary

Validate-then-build edit translation for labels, reviewers, and assignees across 4 forges with per-flag UnsupportedFeature errors.

## What Was Built

### `gf pr edit` (PR-09)
- **GitHub**: Direct flag passthrough (`gh pr edit <N> --add-label/--remove-label/--add-reviewer/--remove-reviewer/--add-assignee/--remove-assignee`)
- **GitLab**: Verb remap to `update` + prefix semantics (`glab mr update <N> --label/--unlabel/--reviewer +X/-X/--assignee +X/-X`)
- **Forgejo**: Subcommand routing for labels only (`fj pr edit <N> labels --add/--rm`); reviewer/assignee → UnsupportedFeature
- **Gitea**: Entire command → UnsupportedFeature (tea has no `pulls edit`)

### `gf issue edit` (ISSUE-08)
- **GitHub**: Direct flag passthrough (`gh issue edit <N> --add-label/--remove-label/--add-assignee/--remove-assignee`)
- **GitLab**: Verb remap to `update` + prefix semantics (`glab issue update <N> --label/--unlabel/--assignee +X/-X`)
- **Gitea**: Plural flag rename (`tea issues edit <N> --add-labels/--remove-labels/--add-assignees`); `--remove-assignees` → UnsupportedFeature
- **Forgejo**: All label/assignee flags → UnsupportedFeature (fj issue edit has no subcommands for these)

## Task Summary

### Task 1: Add edit clap subcommands and translation functions (TDD)
- **RED**: 34 failing tests added (18 PR edit + 16 issue edit) — commit `ab5341f`
- **GREEN**: Clap subcommands in `build_pr()` and `build_issue()`, dispatch arms, `translate_pr_edit()` and `translate_issue_edit()` with validate-then-build pattern — commit `b3df20c`
- **REFACTOR**: No changes needed (clippy clean for our files)

### Task 2: Add audit tests for forge CLI flag existence
- 22 audit tests verifying real forge CLI `--help` output for edit flags — commit `070abb9`
- Covers gh (10), glab (7), fj (2), tea (3) flag verifications
- Unsupported combinations documented in comments

## Test Results

- **Before**: 381 tests
- **After**: 437 tests (56 new: 34 translation/unsupported + 22 audit)
- **Regressions**: 0
- **Compiler warnings**: 0

## Deviations from Plan

### Minor Deviation
**1. Audit test count: 22 instead of 23**
- **Found during:** Task 2
- **Issue:** Plan states "23 audit tests" but actual spec lists only 22 distinct tests (6+4+2+4+3+3=22)
- **Fix:** Implemented the 22 tests actually specified; plan's "23" was an arithmetic error in the summary
- **Impact:** None — all specified forge flags are covered

## Decisions Made

1. **Validate-then-build pattern**: Check all flags against forge capabilities before constructing args — prevents partial argument construction on error paths
2. **Forgejo subcommand routing**: `fj pr edit <N> labels --add/--rm` inserts "labels" subcommand only when label flags present
3. **GitLab prefix semantics**: `+alice`/`-alice` format for reviewer/assignee add/remove
4. **Per-flag UnsupportedFeature**: Granular error messages (e.g., "pr edit --add-reviewer" not just "pr edit") for better user feedback

## Self-Check: PASSED

All files exist, all commits verified.
