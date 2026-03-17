---
phase: 5
slug: fix-self-hosted-browse
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`) |
| **Config file** | none (Cargo.toml [dev-dependencies]) |
| **Quick run command** | `cargo test browse` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test browse`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | CORE-05 | unit | `cargo test browse::tests::test_resolve_forge_type_self_hosted_via_config` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 1 | CORE-05 | unit | `cargo test browse::tests::test_resolve_forge_type_self_hosted_unknown_still_errors` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 1 | BROWSE-01–04 | unit | `cargo test browse` | ✅ | ⬜ pending |
| 05-01-04 | 01 | 1 | CORE-05 | unit | `cargo test forge` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/browse/mod.rs` — add `test_resolve_forge_type_self_hosted_via_config` stub
- [ ] `src/browse/mod.rs` — add `test_resolve_forge_type_self_hosted_unknown_still_errors` stub

*Existing infrastructure covers framework needs.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
