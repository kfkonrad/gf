---
phase: 7
slug: flag-normalization-audit
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + cargo |
| **Config file** | `Cargo.toml` (existing) |
| **Quick run command** | `cargo test adapter` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test adapter`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 7-01-01 | 01 | 1 | QUAL-01 | unit (macro table) | `cargo test pr_create` | ❌ W0 — `tests/flag_audit.rs` | ⬜ pending |
| 7-01-02 | 01 | 1 | QUAL-01 | unit (macro table) | `cargo test pr_subcommand` | ❌ W0 — `tests/flag_audit.rs` | ⬜ pending |
| 7-01-03 | 01 | 1 | QUAL-01 | integration | `cargo test audit_` | ❌ W0 — `tests/flag_audit.rs` | ⬜ pending |
| 7-02-01 | 02 | 1 | QUAL-02 | unit (macro table) | `cargo test v11_` | ❌ W0 — `tests/flag_audit.rs` | ⬜ pending |
| 7-02-02 | 02 | 1 | QUAL-02 | integration | `cargo test audit_v11` | ❌ W0 — `tests/flag_audit.rs` | ⬜ pending |
| 7-03-01 | 03 | 1 | QUAL-03 | unit (macro table completeness) | `cargo test` | ❌ W0 — macro table | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/flag_audit.rs` — new file; covers QUAL-01, QUAL-02, QUAL-03
  - Integration audit helper fn `forge_help_contains()`
  - Macro `translation_test!` or `assert_translation!`
  - All existing adapter tests migrated to macro table
  - v1.1 pre-mapping table entries

*Existing `cargo test` infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| fj pr create --title availability | QUAL-01 | Requires live fj repo interaction | Run `fj pr create --help` and verify --title flag presence |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
