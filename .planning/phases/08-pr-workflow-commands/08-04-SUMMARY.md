---
phase: 08-pr-workflow-commands
plan: "04"
subsystem: browse
tags: [browse, pr, issue, url-builder, clap, flags]
dependency_graph:
  requires: ["08-01"]
  provides: ["build_pr_url", "build_issue_url", "browse --pr", "browse --issue"]
  affects: ["src/browse/mod.rs", "src/cmd/mod.rs"]
tech_stack:
  added: []
  patterns: ["clap visible_alias for --mr", "early-return before git ref resolution"]
key_files:
  created: []
  modified:
    - src/browse/mod.rs
    - src/cmd/mod.rs
    - src/adapter/pr.rs
decisions:
  - "--mr implemented as visible_alias on --pr (not a separate flag) — single source of truth"
  - "Early return in browse::run() before git ref resolution for --pr/--issue — no git subprocess needed"
  - "Conflict enforcement done via clap conflicts_with_all — no runtime error handling needed"
metrics:
  duration: "108s"
  completed_date: "2026-03-18"
  tasks_completed: 2
  files_modified: 3
---

# Phase 08 Plan 04: Browse --pr/--issue Flags Summary

One-liner: PR/MR and issue URL builders for all 4 forges with clap conflict enforcement and --mr alias support.

## What Was Built

Added `--pr` (with `--mr` as a visible alias) and `--issue` flags to the browse subcommand. Both flags short-circuit the normal git-ref resolution path in `browse::run()`, building and opening the URL directly using `build_pr_url()` or `build_issue_url()`. Clap enforces mutual exclusivity between `--pr`, `--issue`, the file positional arg, and `--branch`.

## Task Summary

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add --pr/--mr/--issue clap flags with conflict rules | bf16c63 | src/cmd/mod.rs, src/adapter/pr.rs |
| 2 | Implement build_pr_url(), build_issue_url(), browse::run() early-return | 7d714fb | src/browse/mod.rs |

## Key Functions Added

- `build_pr_url(forge, host, owner, repo, number) -> String`
  - GitHub: `.../pull/<N>`
  - GitLab: `.../-/merge_requests/<N>`
  - Gitea/Forgejo: `.../pulls/<N>`

- `build_issue_url(forge, host, owner, repo, number) -> String`
  - GitHub/Gitea/Forgejo: `.../issues/<N>`
  - GitLab: `.../-/issues/<N>`

## Tests Added

14 new unit tests in `src/browse/mod.rs`:
- 4 `build_pr_url` tests (one per forge)
- 4 `build_issue_url` tests (one per forge)
- 2 self-hosted URL tests (GitLab)
- 4 clap conflict/alias tests (`--pr` vs file, `--pr` vs `--branch`, `--issue` vs `--pr`, `--mr` alias)

Total test count: 91 lib tests + 25 integration tests = 116 passing.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing translate_pr_list/checkout/merge functions**
- **Found during:** Task 1 (cargo build failure)
- **Issue:** src/adapter/pr.rs dispatch referenced `translate_pr_list`, `translate_pr_checkout`, `translate_pr_merge` which were not defined — left as stubs from 08-01 plan work
- **Fix:** Added all three functions with correct forge-specific flag translation logic (matching patterns from CONTEXT.md and Phase 7 pre-mapping tests)
- **Files modified:** src/adapter/pr.rs
- **Commit:** bf16c63

## Self-Check: PASSED

- [x] src/browse/mod.rs contains `pub fn build_pr_url(`
- [x] src/browse/mod.rs contains `pub fn build_issue_url(`
- [x] src/cmd/mod.rs contains `Arg::new("pr")` with `.visible_alias("mr")`
- [x] src/cmd/mod.rs contains `Arg::new("issue")`
- [x] All 91 lib tests pass, 25 integration tests pass
- [x] Commits bf16c63 and 7d714fb exist
