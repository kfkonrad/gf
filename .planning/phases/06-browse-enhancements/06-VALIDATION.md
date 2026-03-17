---
phase: 6
slug: browse-enhancements
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none — standard `#[cfg(test)]` modules |
| **Quick run command** | `cargo test -p gf 2>&1` |
| **Full suite command** | `cargo test 2>&1` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p gf 2>&1`
- **After every plan wave:** Run `cargo test 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | BROWSE-02 | unit | `cargo test test_resolve_forge_type` | ✅ | ⬜ pending |
| 06-02-01 | 02 | 2 | BROWSE-01 | unit | `cargo test test_build_file_url_with_line` | ❌ W0 | ⬜ pending |
| 06-02-02 | 02 | 2 | BROWSE-01 | unit | `cargo test test_build_file_url_with_range_github` | ❌ W0 | ⬜ pending |
| 06-02-03 | 02 | 2 | BROWSE-01 | unit | `cargo test test_build_file_url_with_range_gitlab` | ❌ W0 | ⬜ pending |
| 06-02-04 | 02 | 2 | BROWSE-01 | unit | `cargo test test_parse_line_spec_zero_errors` | ❌ W0 | ⬜ pending |
| 06-02-05 | 02 | 2 | BROWSE-01 | unit | `cargo test test_parse_line_spec_reversed_errors` | ❌ W0 | ⬜ pending |
| 06-02-06 | 02 | 2 | BROWSE-01 | unit | `cargo test test_parse_line_spec_non_numeric_errors` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/browse/mod.rs` test module — stubs for `parse_line_spec` (valid, zero, reversed, non-numeric)
- [ ] `src/browse/mod.rs` test module — stubs for `build_file_url` with `Some(LineRange)` for all four forges
- [ ] `src/browse/mod.rs` test module — stubs for `split_file_and_line` helper

*Existing infrastructure covers framework — no new test files needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Browser opens correct URL | BROWSE-01 | Requires actual browser | Run `gf browse src/main.rs:42` and verify URL in browser address bar |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
