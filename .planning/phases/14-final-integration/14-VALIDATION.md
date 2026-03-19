---
phase: 14
slug: final-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2025-07-18
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + assert_cmd 2 |
| **Config file** | Cargo.toml `[dev-dependencies]` |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~20 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test 2>&1` (full output)
- **Before `/gsd-verify-work`:** Full suite green + `cargo build` zero warnings + help text verified
- **Max feedback latency:** 20 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | PR-08 | unit (macro) | `cargo test pr_checks` | ❌ LOST | ⬜ pending |
| 14-01-01 | 01 | 1 | ISSUE-07 | unit (macro) | `cargo test issue_comment` | ❌ LOST | ⬜ pending |
| 14-01-01 | 01 | 1 | ISSUE-07 | unit (macro) | `cargo test pr_comment` | ❌ LOST | ⬜ pending |
| 14-01-02 | 01 | 1 | PR-08 | integration | `cargo test test_pr_help_shows_checks` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | PR-09 | integration | `cargo test test_pr_help_shows_edit` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | ISSUE-07 | integration | `cargo test test_issue_help_shows_comment` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | ISSUE-08 | integration | `cargo test test_issue_help_shows_edit` | ❌ W0 | ⬜ pending |
| 14-01-03 | 01 | 1 | ALL | build | `cargo build 2>&1 \| grep -c warning` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Restore lost code from git commit `81d3248` (PR checks, issue/PR comments)
- Restore 22 lost tests from same commit
- Add help text integration tests
- Update PROJECT.md

*Code restoration is prerequisite for all other work.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 20s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
