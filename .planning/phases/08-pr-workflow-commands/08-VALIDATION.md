---
phase: 8
slug: pr-workflow-commands
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + assert_cmd 2 |
| **Config file** | Cargo.toml [dev-dependencies] |
| **Quick run command** | `cargo test --test flag_audit` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test flag_audit && cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | PR-01 | unit | `cargo test v11_pr_list -- --include-ignored` | ✅ (ignored) | ⬜ pending |
| 08-01-02 | 01 | 1 | PR-01 | unit | `cargo test pr_list_tea_author_unsupported` | ❌ W0 | ⬜ pending |
| 08-01-03 | 01 | 1 | PR-02 | unit | `cargo test v11_pr_merge -- --include-ignored` | ✅ (ignored) | ⬜ pending |
| 08-01-04 | 01 | 1 | PR-02 | unit | `cargo test pr_merge_delete_branch` | ❌ W0 | ⬜ pending |
| 08-01-05 | 01 | 1 | PR-02 | unit | `cargo test pr_merge_default_strategy` | ❌ W0 | ⬜ pending |
| 08-01-06 | 01 | 1 | PR-03 | unit | `cargo test v11_pr_checkout -- --include-ignored` | ✅ (ignored) | ⬜ pending |
| 08-01-07 | 01 | 1 | PR-04 | unit | `cargo test v11_pr_review_comment -- --include-ignored` | ✅ (ignored) | ⬜ pending |
| 08-01-08 | 01 | 1 | PR-04 | unit | `cargo test pr_review_tea_unsupported` | ❌ W0 | ⬜ pending |
| 08-01-09 | 01 | 1 | PR-05 | unit | `cargo test v11_pr_review_approve -- --include-ignored` | ✅ (ignored) | ⬜ pending |
| 08-01-10 | 01 | 1 | PR-05 | unit | `cargo test pr_approve_tea_unsupported` | ❌ W0 | ⬜ pending |
| 08-01-11 | 01 | 1 | PR-06 | unit | `cargo test pr_view` | ✅ passing | ⬜ pending |
| 08-01-12 | 01 | 1 | PR-07 | unit | `cargo test browse_pr_url` | ❌ W0 | ⬜ pending |
| 08-01-13 | 01 | 1 | PR-07 | unit | `cargo test browse_issue_url` | ❌ W0 | ⬜ pending |
| 08-01-14 | 01 | 1 | PR-07 | unit | `cargo test browse_pr_conflicts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/flag_audit.rs` — Add unsupported-error test macro and test cases for tea/fj unsupported combinations
- [ ] `tests/flag_audit.rs` — Add delete-branch translation tests (4 forges × 2 flag values)
- [ ] `tests/flag_audit.rs` — Add default merge strategy tests (4 forges)
- [ ] `src/browse/mod.rs` (unit tests section) — Add `build_pr_url` and `build_issue_url` unit tests
- [ ] `tests/integration_test.rs` — Add integration test for unsupported error output format
- [ ] Update `translation_test!` and `v11_translation_test!` macros to handle `Result` return type

*Existing infrastructure covers PR-06 (already passing).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Browse --pr opens browser | PR-07 | Requires browser launch | Run `gf browse --pr 1` in a repo, verify browser opens |

*All other behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
