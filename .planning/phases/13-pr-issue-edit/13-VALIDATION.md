---
phase: 13
slug: pr-issue-edit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2025-07-18
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml (test targets auto-discovered) |
| **Quick run command** | `cargo test --lib` |
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
| 13-01-01 | 01 | 1 | PR-09 | unit (translation) | `cargo test pr_edit_github_add_label` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | unit (translation) | `cargo test pr_edit_glab_add_reviewer` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | unit (unsupported) | `cargo test pr_edit_fj_add_reviewer_unsupported` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | unit (unsupported) | `cargo test pr_edit_tea_unsupported` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | unit (translation) | `cargo test pr_edit_fj_add_label` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | integration (audit) | `cargo test audit_gh_pr_edit_add_label` | ❌ W0 | ⬜ pending |
| 13-01-01 | 01 | 1 | PR-09 | integration (audit) | `cargo test audit_glab_mr_update_label` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ISSUE-08 | unit (translation) | `cargo test issue_edit_github_add_label` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ISSUE-08 | unit (translation) | `cargo test issue_edit_glab_add_label` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ISSUE-08 | unit (unsupported) | `cargo test issue_edit_fj_add_label_unsupported` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ISSUE-08 | unit (translation) | `cargo test issue_edit_tea_add_label` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ISSUE-08 | unit (unsupported) | `cargo test issue_edit_tea_remove_assignee_unsupported` | ❌ W0 | ⬜ pending |

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
