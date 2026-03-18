---
phase: 10
slug: cleanup-dead-code-and-test-gaps
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | REPO-01 | compile | `cargo build --release 2>&1 \| grep -c warning` | ✅ | ⬜ pending |
| 10-01-02 | 01 | 1 | QUAL-02 | grep | `grep -c v11_translation_test tests/flag_audit.rs` | ✅ | ⬜ pending |
| 10-01-03 | 01 | 1 | QUAL-03 | unit | `cargo test issue_list_fj_author` | ❌ W0 | ⬜ pending |
| 10-01-04 | 01 | 1 | ISSUE-01 | unit | `cargo test fj_issue_search` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements. Test macros (`translation_test!`, `audit_test!`) already exist in `tests/flag_audit.rs`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Zero compiler warnings | QUAL-02 | Build output check | `cargo build --release 2>&1 \| grep warning` should return nothing |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
