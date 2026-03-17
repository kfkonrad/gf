---
phase: 3
slug: command-routing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test runner) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test 2>&1 | tail -5` |
| **Full suite command** | `cargo test -- --nocapture` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1 | tail -5`
- **After every plan wave:** Run `cargo test -- --nocapture`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 0 | CORE-08..12 | unit | `cargo test cmd::` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 1 | PR-01, PR-02 | unit | `cargo test adapter::pr` | ❌ W0 | ⬜ pending |
| 3-01-03 | 01 | 1 | PR-03, PR-04 | unit | `cargo test adapter::passthrough` | ❌ W0 | ⬜ pending |
| 3-01-04 | 01 | 1 | PR-05, PR-06 | unit | `cargo test adapter::pr_view` | ❌ W0 | ⬜ pending |
| 3-02-01 | 02 | 1 | REPO-01..03 | unit | `cargo test adapter::repo` | ❌ W0 | ⬜ pending |
| 3-02-02 | 02 | 1 | AUTH-01..03 | unit | `cargo test adapter::auth` | ❌ W0 | ⬜ pending |
| 3-03-01 | 03 | 2 | CORE-08..12 | integration | `cargo test integration::aliases` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/integration/aliases.rs` — stubs for alias routing (CORE-08..12)
- [ ] `src/adapter.rs` — translate() function with stub tests
- [ ] `src/cmd/mod.rs` — clap CLI struct stubs

*Existing `cargo test` infrastructure covers framework needs; test files need creation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `gf pr create` on live GitHub repo creates real PR | PR-01 | Requires live GitHub credentials | Run against test repo with `gh` configured |
| `glab mr create` translation works with live GitLab | PR-02 | Requires live GitLab credentials | Run against test GitLab repo |
| Shell completions function in bash/zsh | CORE-12 | Requires live shell evaluation | Source generated completion script, tab-complete `gf pr ` |
| `fj` auth and pr flags accepted correctly | AUTH-01, PR-01 | fj flags MEDIUM confidence; verify at runtime | Run `fj --version` and `fj pr create --help` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
