---
phase: 4
slug: browse
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + assert_cmd 2.x + predicates 3.x |
| **Config file** | none — `cargo test` discovers tests automatically |
| **Quick run command** | `cargo test browse` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test browse`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 0 | BROWSE-01,03 | unit stub | `cargo test browse` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | BROWSE-01 | unit | `cargo test browse::tests::test_build_repo_url` | ❌ W0 | ⬜ pending |
| 4-01-03 | 01 | 1 | BROWSE-02 | unit | `cargo test browse::tests::test_resolve_ref_branch` | ❌ W0 | ⬜ pending |
| 4-01-04 | 01 | 1 | BROWSE-02 | unit | `cargo test browse::tests::test_resolve_ref_detached` | ❌ W0 | ⬜ pending |
| 4-01-05 | 01 | 1 | BROWSE-03 | unit | `cargo test browse::tests::test_file_url_` | ❌ W0 | ⬜ pending |
| 4-01-06 | 01 | 1 | BROWSE-03 | unit | `cargo test browse::tests::test_normalize_path` | ❌ W0 | ⬜ pending |
| 4-01-07 | 01 | 1 | BROWSE-04 | unit | `cargo test browse::tests::test_branch_override` | ❌ W0 | ⬜ pending |
| 4-01-08 | 01 | 2 | BROWSE-01 | integration | `cargo test test_browse_no_browser_prints_url` | ❌ W0 | ⬜ pending |
| 4-01-09 | 01 | 2 | BROWSE-05 | integration | `cargo test test_browse_no_forge_cli_spawned` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/browse/mod.rs` — module stub must exist before unit tests compile
- [ ] `src/browse/mod.rs` — unit test stubs for `build_repo_url()`, `build_file_url()`, `parse_remote_parts()`, `resolve_ref()`, `normalize_path()`
- [ ] `tests/integration_test.rs` — stubs for `test_browse_no_browser_prints_url`, `test_browse_no_forge_cli_spawned`
- [ ] `Cargo.toml` — `webbrowser = "1"` dependency added

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `gf browse` actually opens a browser | BROWSE-01 | CI is headless; `webbrowser::open()` fails non-fatally in headless env | Run `gf browse` in a local terminal, observe browser opens to correct forge URL |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
