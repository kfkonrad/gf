---
phase: 12-issue-pr-comments
plan: 01
subsystem: cli-translation
tags: [rust, clap, forge-cli, issue-comment, pr-comment, gitlab-note, forgejo-positional]

# Dependency graph
requires:
  - phase: 11-pr-checks
    provides: "translate_pr_checks pattern, macro-based test infrastructure, UnsupportedFeature 3-field error"
provides:
  - "translate_issue_comment() — 4-forge dispatch for gf issue comment"
  - "translate_pr_comment() — 4-forge dispatch for gf pr comment"
  - "comment clap subcommands under build_issue() and build_pr()"
affects: [13-issue-pr-editing]

# Tech tracking
tech-stack:
  added: []
  patterns: ["verb remap (comment→note for GitLab)", "flag remap (--body→--message for GitLab)", "positional body (Forgejo strips flag prefix)"]

key-files:
  created: []
  modified:
    - src/adapter/issue.rs
    - src/adapter/pr.rs
    - src/cmd/mod.rs
    - tests/flag_audit.rs

key-decisions:
  - "Standalone comment functions coexist with existing translate_pr_review() comment branch — both paths remain valid"
  - "Issue comment number is required (can't infer from branch); PR comment number is optional (branch inference)"
  - "GitLab uses note verb + --message flag; Forgejo uses positional body (no flag); Gitea is unsupported"

patterns-established:
  - "Comment verb remap: GitLab maps 'comment' to 'note' for both issue and PR domains"
  - "Positional body pattern: Forgejo strips --body flag and passes text as positional arg"

requirements-completed: [ISSUE-07]

# Metrics
duration: 7min
completed: 2026-03-19
---

# Phase 12 Plan 01: Issue/PR Comment Translation Summary

**Standalone `gf issue comment` and `gf pr comment` with GitLab note+message remap, Forgejo positional body, and Gitea unsupported — 22 new tests (413 total)**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-19T10:05:44Z
- **Completed:** 2026-03-19T10:12:14Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- `gf issue comment 42 --body "text"` translates correctly for GitHub (passthrough), GitLab (note+message), Forgejo (positional), and Gitea (UnsupportedFeature)
- `gf pr comment 42 --body "text"` translates correctly for all 4 forges with same pattern as issue
- 22 new tests: 16 translation, 2 unsupported, 4 no-body/no-number variants (+ 6 audit verified inline but counted in test totals → actually 22 new test functions total)
- Full test suite: 413 tests, 0 failures, 0 regressions

## Task Commits

Each task was committed atomically:

1. **Task 1 (TDD RED): Add failing tests for comment behavior** - `2f76225` (test)
2. **Task 1 (TDD GREEN): Implement comment translation functions** - `81d3248` (feat)
3. **Task 2: Add comprehensive translation and audit tests** - `717318a` (test)

**Plan metadata:** (pending)

_Note: Task 1 used TDD flow with separate RED and GREEN commits._

## Files Created/Modified
- `src/adapter/issue.rs` - Added translate_issue_comment() with 4-forge dispatch (note verb for GitLab, positional for Forgejo, UnsupportedFeature for Gitea)
- `src/adapter/pr.rs` - Added translate_pr_comment() with identical forge mapping pattern
- `src/cmd/mod.rs` - Added comment subcommands to build_issue() (number required) and build_pr() (number optional)
- `tests/flag_audit.rs` - 22 new tests: translation, unsupported, no-body, no-number, and audit

## Decisions Made
- Standalone comment functions coexist with existing `translate_pr_review()` comment branch — `gf pr review --comment` and `gf pr comment` are both valid paths
- Issue comment number is `.required(true)` (can't infer issue from branch); PR comment number is `.required(false)` (branch inference supported)
- GitLab uses `note` verb + `--message` flag for both issue and PR domains
- Forgejo uses positional body (no flag prefix) for both issue and PR domains
- Gitea returns UnsupportedFeature for both issue and PR comments (tea CLI has no comment command)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Comment translation complete for both issue and PR domains
- Phase 13 (Issue/PR Editing) can proceed — editing uses similar verb/flag remap patterns
- Pattern established: GitLab verb remaps + Forgejo positional args will be reused in Phase 13

---
*Phase: 12-issue-pr-comments*
*Completed: 2026-03-19*
