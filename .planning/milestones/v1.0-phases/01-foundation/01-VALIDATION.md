---
phase: 1
slug: foundation
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-16
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + `assert_cmd` 2.x + `predicates` 3.x |
| **Config file** | None — `cargo test` discovers tests automatically |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test -- --include-ignored` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test -- --include-ignored`
- **Before `/gsd:verify-work`:** Full suite must be green + manual TTY/signal checks
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 0 | CORE-06 | integration | `cargo test test_cli_not_found` | ✅ | ✅ green |
| 1-01-02 | 01 | 0 | CORE-06 | integration | `cargo test test_cli_not_found_format` | ✅ | ✅ green |
| 1-01-03 | 01 | 1 | CORE-07 | integration | `cargo test test_exit_code_propagation` | ✅ | ✅ green |
| 1-01-04 | 01 | 1 | CORE-07 | manual | — requires TTY signal delivery | manual-only | ⬜ pending |
| 1-01-05 | 01 | 1 | CORE-07 | manual | — requires real TTY | manual-only | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/integration_test.rs` — stubs for CORE-06 (CliNotFound error format) and CORE-07 (exit code propagation)
- [ ] `tests/fixtures/exit_with.sh` (or small helper binary) — needed for exit code propagation tests
- [ ] `Cargo.toml` dev-dependencies: `assert_cmd = "2"` and `predicates = "3"`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ctrl+C causes exit 130 (signal re-raise) | CORE-07 | Requires TTY signal delivery — cannot send SIGINT to subprocess in CI without PTY harness | Run `gf <cmd>`, press Ctrl+C, check `echo $?` returns 130 |
| TTY inherited — child color output works | CORE-07 | Requires real TTY — `cargo test` runs in non-TTY context | Run `gf gh --color always status` in terminal, confirm color output identical to `gh --color always status` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
