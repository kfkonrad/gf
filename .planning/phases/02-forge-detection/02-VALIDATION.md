---
phase: 2
slug: forge-detection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in) |
| **Config file** | none — Wave 0 installs test stubs |
| **Quick run command** | `cargo test forge` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test forge`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | CORE-01 | unit | `cargo test forge::detection::https` | ❌ W0 | ⬜ pending |
| 2-01-02 | 01 | 1 | CORE-02 | unit | `cargo test forge::detection::ssh` | ❌ W0 | ⬜ pending |
| 2-01-03 | 01 | 1 | CORE-03 | unit | `cargo test forge::detection::remote_flag` | ❌ W0 | ⬜ pending |
| 2-01-04 | 01 | 2 | CORE-04 | unit | `cargo test forge::config` | ❌ W0 | ⬜ pending |
| 2-01-05 | 01 | 2 | CORE-05 | unit | `cargo test forge::detection::error` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/forge/mod.rs` — test module stubs for all 5 CORE requirements
- [ ] `src/forge/detection.rs` — unit test stubs for URL parsing (HTTPS + SSH)
- [ ] `src/forge/config.rs` — unit test stubs for TOML config loading

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Self-hosted domain config roundtrip | CORE-04 | Requires actual `~/.config/gf/config.toml` on disk | Add entry, run `gf detect` in a repo with matching remote, verify correct forge type returned |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
