---
phase: 12
slug: issue-pr-comments
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2025-07-18
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + macro generators in tests/flag_audit.rs |
| **Config file** | Cargo.toml `[dev-dependencies]` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | ISSUE-07-a | unit (translation_test!) | `cargo test issue_comment_github` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-b | unit (translation_test!) | `cargo test issue_comment_glab` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-c | unit (translation_test!) | `cargo test issue_comment_fj` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-d | unit (unsupported_test!) | `cargo test issue_comment_tea_unsupported` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-e | unit (translation_test!) | `cargo test pr_comment_github` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-f | unit (translation_test!) | `cargo test pr_comment_glab` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-g | unit (translation_test!) | `cargo test pr_comment_fj` | ❌ W0 | ⬜ pending |
| 12-01-01 | 01 | 1 | ISSUE-07-h | unit (unsupported_test!) | `cargo test pr_comment_tea_unsupported` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-i | audit (audit_test!) | `cargo test audit_gh_issue_comment` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-j | audit (audit_test!) | `cargo test audit_glab_issue_note` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-k | audit (audit_test!) | `cargo test audit_fj_issue_comment` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-l | audit (audit_test!) | `cargo test audit_gh_pr_comment` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-m | audit (audit_test!) | `cargo test audit_glab_mr_note` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | ISSUE-07-n | audit (audit_test!) | `cargo test audit_fj_pr_comment` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- All tests listed above need to be added to `tests/flag_audit.rs`
- No framework install needed — test infrastructure fully exists
- No config changes needed

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
