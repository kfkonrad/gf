---
phase: 09-issues-clone-and-self-hosted-detection
plan: 00
subsystem: testing
tags: [test-scaffolding, tdd, nyquist-compliance]
dependency_graph:
  requires: [translation_test macro, v11_translation_test macro, unsupported_test macro]
  provides: [issue close test stubs, issue reopen test stubs, repo clone tea unsupported test]
  affects: [tests/flag_audit.rs]
tech_stack:
  added: []
  patterns: [v11_translation_test for pre-implementation stubs]
key_files:
  created: []
  modified: [tests/flag_audit.rs]
decisions: []
metrics:
  duration_seconds: 93
  completed_date: "2026-03-18T13:21:34Z"
---

# Phase 09 Plan 00: Test Scaffolding for Issue Close/Reopen Summary

**One-liner:** Created v11 test stubs for issue close (4 forges), issue reopen (3 forges + fj unsupported), and tea repo clone unsupported

## What Was Built

Added test scaffolding for issue close/reopen operations and unsupported combinations to satisfy Nyquist compliance requirements. All tests are marked as `#[ignore]` using the `v11_translation_test!` macro, ensuring they exist before implementation (Plan 01 will make them pass).

### Test Coverage Added

**Issue Close (ISSUE-04):**
- `v11_issue_close_github`: `gf issue close 42` → `gh issue close 42`
- `v11_issue_close_glab`: `gf issue close 42` → `glab issue close 42`
- `v11_issue_close_tea`: `gf issue close 42` → `tea issues close 42` (plural)
- `v11_issue_close_fj`: `gf issue close 42` → `fj issue close 42`

**Issue Reopen (ISSUE-05):**
- `v11_issue_reopen_github`: `gf issue reopen 42` → `gh issue reopen 42`
- `v11_issue_reopen_glab`: `gf issue reopen 42` → `glab issue reopen 42`
- `v11_issue_reopen_tea`: `gf issue reopen 42` → `tea issues reopen 42` (plural)
- `issue_reopen_fj_unsupported`: Forgejo CLI has no issue reopen command

**Repo Clone (REPO-01):**
- `repo_clone_tea_unsupported`: tea has no repos clone subcommand

## Task Breakdown

| Task | Name                                   | Status | Commit  | Files                 |
| ---- | -------------------------------------- | ------ | ------- | --------------------- |
| 1    | Add issue close tests (4 forges)       | ✓      | 992003c | tests/flag_audit.rs   |
| 2    | Add issue reopen tests + fj unsupported | ✓      | 8ad47dd | tests/flag_audit.rs   |
| 3    | Add tea repo clone unsupported test    | ✓      | d259204 | tests/flag_audit.rs   |

## Deviations from Plan

None - plan executed exactly as written.

## Technical Details

### Test Structure

All tests follow the established pattern:
- `v11_translation_test!` for supported translations (marked `#[ignore]` until implemented)
- `unsupported_test!` for forge×command combinations that cannot be supported

### Forge-Specific Notes

**tea (Gitea):**
- Uses "issues" (plural) subcommand instead of "issue"
- Has no `repos clone` subcommand (clone unsupported)

**fj (Forgejo):**
- Has no `issue reopen` command (reopen unsupported)

## Verification Results

**Test Counts:**
```
$ grep -c "v11_issue_close" tests/flag_audit.rs
4

$ grep -c "issue_reopen" tests/flag_audit.rs
4

$ grep "repo_clone_tea_unsupported" tests/flag_audit.rs | wc -l
1
```

**Compilation:**
```
$ cargo test --test flag_audit --no-run
   Compiling gf v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.94s
```

**Ignored Status:**
```
$ cargo test --test flag_audit 2>&1 | grep -E "v11_issue_(close|reopen)"
test v11_issue_close_fj ... ignored
test v11_issue_close_github ... ignored
test v11_issue_close_glab ... ignored
test v11_issue_close_tea ... ignored
test v11_issue_reopen_github ... ignored
test v11_issue_reopen_glab ... ignored
test v11_issue_reopen_tea ... ignored
```

All tests properly marked as ignored until implementation.

## Next Steps

Plan 01 (Wave 1) will implement the issue close/reopen adapters, making these tests pass through TDD execution:
1. RED: Run tests (currently ignored, will fail when un-ignored)
2. GREEN: Implement adapters to make tests pass
3. REFACTOR: Clean up if needed

## Self-Check: PASSED

**Created Files:** None (test-only plan)

**Modified Files:**
- [✓] tests/flag_audit.rs exists and contains all expected test entries

**Commits:**
- [✓] 992003c exists: test(09-00): add v11 issue close tests for all 4 forges
- [✓] 8ad47dd exists: test(09-00): add v11 issue reopen tests + fj unsupported
- [✓] d259204 exists: test(09-00): add tea repo clone unsupported test

All verification checks passed.
