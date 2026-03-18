---
phase: 9
slug: issues-clone-and-self-hosted-detection
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-18
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test framework + assert_cmd v2.2.0 |
| **Config file** | None — cargo test discovers all `#[test]` functions |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10s

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 1 | ISSUE-01 | unit | `cargo test issue_list -x` | ✅ flag_audit.rs | ⬜ pending |
| 09-01-02 | 01 | 1 | ISSUE-02 | unit | `cargo test issue_view -x` | ✅ flag_audit.rs | ⬜ pending |
| 09-01-03 | 01 | 1 | ISSUE-03 | unit | `cargo test issue_create -x` | ✅ flag_audit.rs | ⬜ pending |
| 09-01-04 | 01 | 1 | ISSUE-04 | unit | `cargo test issue_close -x` | ❌ W0 | ⬜ pending |
| 09-01-05 | 01 | 1 | ISSUE-05 | unit | `cargo test issue_reopen -x` | ❌ W0 | ⬜ pending |
| 09-01-06 | 01 | 1 | ISSUE-06 | unit | `cargo test build_issue_url -x` | ✅ browse/mod.rs | ⬜ pending |
| 09-02-01 | 02 | 2 | CORE-04 | unit | `cargo test probe_auth -x` | ❌ W0 | ⬜ pending |
| 09-03-01 | 03 | 2 | REPO-01 | unit | `cargo test repo_clone -x` | ✅ flag_audit.rs | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `tests/flag_audit.rs` — Add v11_translation_test! entries for issue close/reopen (all 4 forges) — **Addressed by 09-00-PLAN.md**
- [x] `tests/flag_audit.rs` — Add unsupported_test! for fj issue reopen and tea repo clone — **Addressed by 09-00-PLAN.md**
- [ ] `src/forge/mod.rs` — Add #[cfg(test)] module for probe_auth() unit tests (mock Command::output)
- [ ] `src/forge/mod.rs` — Add #[cfg(test)] module for cache load/save unit tests (tempfile for isolation)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real forge auth probing | CORE-04 | Requires installed forge CLIs with auth | Install gh/glab/tea/fj, authenticate, run `gf` against unknown self-hosted domain |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
