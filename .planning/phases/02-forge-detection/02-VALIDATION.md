---
phase: 2
slug: forge-detection
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
audited: 2026-03-16
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

All detection code lives in `src/forge/mod.rs`. Test paths use `forge::tests::*`.

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 2-01-01 | 01 | 1 | CORE-01 | unit (RED) | `cargo test forge::tests::test_parse_host_https_github` | ✅ green |
| 2-01-02 | 01 | 1 | CORE-01 | unit (RED) | `cargo test forge::tests::test_parse_host_ssh_scp` | ✅ green |
| 2-01-03 | 01 | 1 | CORE-02 | unit (RED) | `cargo test forge::tests::test_get_remote_url_invalid_remote` | ✅ green |
| 2-01-04 | 01 | 1 | CORE-03 | unit (RED) | `cargo test forge::tests::test_known_hosts` | ✅ green |
| 2-01-05 | 01 | 1 | CORE-05 | unit (RED) | `cargo test forge::tests::test_config_lookup_with_inline_config` | ✅ green |
| 2-02-01 | 02 | 2 | CORE-01 | unit (GREEN) | `cargo test forge::tests::test_parse_host_https_github` | ✅ green |
| 2-02-02 | 02 | 2 | CORE-01 | unit (GREEN) | `cargo test forge::tests::test_parse_host_ssh_scp` | ✅ green |
| 2-02-03 | 02 | 2 | CORE-02 | unit (GREEN) | `cargo test forge::tests::test_get_remote_url_invalid_remote` | ✅ green |
| 2-02-04 | 02 | 2 | CORE-03 | unit (GREEN) | `cargo test forge::tests::test_known_host_github` | ✅ green |
| 2-03-01 | 03 | 3 | CORE-05 | unit (GREEN) | `cargo test forge::tests::test_config_lookup_with_inline_config` | ✅ green |
| 2-03-02 | 03 | 3 | CORE-01 | integration | `cargo test forge_detection::test_gf_outside_git_repo_shows_error` | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

Note: CORE-04 (auth probing) was dropped per locked user decision in CONTEXT.md — no tasks or tests exist for it.

---

## Wave 0 Requirements

Wave 0 is satisfied by plan 01 (stubs + failing tests committed before any implementation):

- [x] `src/forge/mod.rs` — `ForgeType` enum and all function stubs created
- [x] `src/forge/mod.rs` — `#[cfg(test)] mod tests` with stubs for all CORE requirements:
  - `test_parse_host_https_github` (CORE-01 HTTPS)
  - `test_parse_host_ssh_scp` (CORE-01 SSH)
  - `test_get_remote_url_invalid_remote` (CORE-02)
  - `test_known_host_github` (CORE-03)
  - `test_config_lookup_with_inline_config` (CORE-05)
- [x] All stub tests compile and fail RED before plan 02 runs

Wave 0 gate command: `cargo test forge::tests 2>&1` — compile must succeed, most tests must fail.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Self-hosted domain roundtrip via real config file on disk | CORE-05 | Requires actual `~/.config/gf/config.toml` | Add entry, run `gf` in a repo with matching remote, verify correct forge type returned |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** 2026-03-16 (retroactive audit — all 11 tasks green, 32 unit tests + 2 integration tests pass)

---

## Validation Audit 2026-03-16

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
| Tasks updated to green | 11 |
| Tests passing | 34 (32 unit + 2 integration) |
