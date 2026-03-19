---
phase: 12-issue-pr-comments
verified: 2026-03-19T10:17:43Z
status: passed
score: 5/5 must-haves verified
re_verification: false
---

# Phase 12: Issue/PR Comments Verification Report

**Phase Goal:** `gf issue comment <number> --body "text"` and `gf pr comment <number> --body "text"` post comments on issues and PRs across all supported forges.
**Verified:** 2026-03-19T10:17:43Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | gf issue comment 42 --body 'text' translates correctly for GitHub, GitLab, and Forgejo | ✓ VERIFIED | `translate_issue_comment()` at line 229 of issue.rs on M002; GitHub passthrough, GitLab note+message, Forgejo positional. Tests `issue_comment_github`, `issue_comment_glab`, `issue_comment_fj` all pass. |
| 2 | gf pr comment 42 --body 'text' translates correctly for GitHub, GitLab, and Forgejo | ✓ VERIFIED | `translate_pr_comment()` at line 532 of pr.rs on M002; identical forge mapping. Tests `pr_comment_github`, `pr_comment_glab`, `pr_comment_fj` all pass. |
| 3 | Both issue comment and pr comment return UnsupportedFeature error for Gitea | ✓ VERIFIED | Both functions check `ForgeType::Gitea` and return `GfError::UnsupportedFeature`. Tests `issue_comment_tea_unsupported` and `pr_comment_tea_unsupported` pass. |
| 4 | PR comment number is optional (branch inference); issue comment number is required | ✓ VERIFIED | `build_issue()` comment subcommand: `.required(true)` for number. `build_pr()` comment subcommand: `.required(false)` for number. Test `pr_comment_github_no_number` verifies omission works. |
| 5 | GitLab uses 'note' verb and '--message' flag; Forgejo uses positional body (no flag) | ✓ VERIFIED | Both `translate_issue_comment()` and `translate_pr_comment()` match on `ForgeType::Gitlab` → "note" verb + "--message" flag; `ForgeType::Forgejo` → body pushed positionally without flag. Tests confirm: `issue_comment_glab` expects `["issue", "note", "42", "--message", "looks good"]`; `issue_comment_fj` expects `["issue", "comment", "42", "looks good"]`. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/adapter/issue.rs` | translate_issue_comment function with 4-forge dispatch | ✓ VERIFIED | Function at line 229 on M002 branch. Handles GitHub (passthrough), GitLab (note+message), Forgejo (positional), Gitea (UnsupportedFeature). Match arm `Some(("comment", sub))` at line 17 wires it into `translate_issue()`. |
| `src/adapter/pr.rs` | translate_pr_comment function with 4-forge dispatch | ✓ VERIFIED | Function at line 532 on M002 branch. Same 4-forge dispatch pattern. Match arm `Some(("comment", sub))` at line 21 wires it into `translate_pr()`. |
| `src/cmd/mod.rs` | comment subcommands under both build_issue() and build_pr() | ✓ VERIFIED | `Command::new("comment")` at line 265 (build_pr) and line 583 (build_issue). PR: number optional, body via --body/-b, extra passthrough. Issue: number required, same body/extra pattern. |
| `tests/flag_audit.rs` | translation, unsupported, and audit tests for comment commands | ✓ VERIFIED | 20 comment-specific test functions: 12 translation_test! (6 issue + 6 PR), 2 unsupported_test! (1 issue + 1 PR), 6 audit_test! (3 issue + 3 PR). All 20 pass. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/adapter/issue.rs` | `translate_issue_comment` | match arm `Some(("comment", sub))` in `translate_issue()` | ✓ WIRED | Line 17 on M002: `Some(("comment", sub)) => translate_issue_comment(forge, issue_cmd, sub)` |
| `src/adapter/pr.rs` | `translate_pr_comment` | match arm `Some(("comment", sub))` in `translate_pr()` | ✓ WIRED | Line 21 on M002: `Some(("comment", sub)) => translate_pr_comment(forge, pr_cmd, sub)` |
| `src/cmd/mod.rs` | `build_pr()` | `.subcommand(Command::new("comment"))` | ✓ WIRED | Line 265 on M002: comment subcommand registered under PR command |
| `src/cmd/mod.rs` | `build_issue()` | `.subcommand(Command::new("comment"))` | ✓ WIRED | Line 583 on M002: comment subcommand registered under issue command |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| ISSUE-07 | 12-01-PLAN | Comment on issues (`gf issue comment`) | ✓ SATISFIED | `translate_issue_comment()` implemented with full 4-forge dispatch. 12 issue-comment tests pass. Clap subcommand with required number and optional --body flag. |

No orphaned requirements — ISSUE-07 is the only requirement mapped to Phase 12 in `v1.2-REQUIREMENTS.md`, and it is claimed and satisfied.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | None found | — | — |

No TODO/FIXME/PLACEHOLDER comments, no empty implementations, no stub returns. Zero compiler warnings, zero test failures.

### Human Verification Required

### 1. CLI End-to-End Comment Posting

**Test:** Run `gf issue comment 1 --body "test comment"` in a real GitHub repo.
**Expected:** Comment appears on issue #1 in the forge's web UI.
**Why human:** Automated tests verify translation output (CLI args), not actual API calls to forge services.

### 2. GitLab Note Verb Acceptance

**Test:** Run `gf issue comment 1 --body "test"` in a GitLab repo.
**Expected:** Translates to `glab issue note 1 --message "test"` and comment appears.
**Why human:** Needs real glab CLI and GitLab instance to confirm the note+message mapping is correct.

### 3. Forgejo Positional Body Acceptance

**Test:** Run `gf pr comment 1 --body "lgtm"` in a Forgejo repo.
**Expected:** Translates to `fj pr comment 1 lgtm` and comment appears.
**Why human:** Needs real fj CLI and Forgejo instance to confirm positional body parsing works.

### Gaps Summary

No gaps found. All 5 must-have truths verified. All 4 required artifacts exist, are substantive (not stubs), and are wired. All 4 key links confirmed. ISSUE-07 requirement satisfied.

**Important note on branch topology:** The implementation exists on the `milestone/M002` branch (commit `717318a`), not on `main`. The `main` branch only has planning documentation commits. This is the expected workflow — the milestone branch accumulates implementation and gets merged upon milestone completion. All verification was performed against the `milestone/M002` worktree where all 413 tests pass with zero failures and zero compiler warnings.

---

_Verified: 2026-03-19T10:17:43Z_
_Verifier: Claude (gsd-verifier)_
