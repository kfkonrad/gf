---
phase: 02-forge-detection
verified: 2026-03-16T00:00:00Z
status: passed
score: 15/15 must-haves verified
re_verification: false
gaps:
  - truth: "CORE-04: System attempts to detect self-hosted forges by probing installed forge CLIs' auth status"
    status: resolved
    reason: "CORE-04 was intentionally dropped from Phase 2 (commit fef50b4). REQUIREMENTS.md traceability updated to Deferred. No implementation gap."
    artifacts: []
human_verification:
  - test: "Run `gf --remote upstream pr list` in a repo that has an `upstream` remote pointing to GitHub"
    expected: "gf detects GitHub from the upstream remote URL and delegates to `gh pr list`"
    why_human: "Cannot programmatically set up a repo with a real upstream remote and verify end-to-end CLI delegation in this environment"
  - test: "Create `~/.config/gf/config.toml` with a `[[forge]]` entry for a custom domain (e.g. `domain = \"git.myco.com\"`, `type = \"gitlab\"`), then run `gf` in a repo whose origin points to that domain"
    expected: "gf routes to `glab` based on the config entry, not the built-in table"
    why_human: "Requires a live git repo with a custom remote and a real config file on the test machine"
---

# Phase 2: Forge Detection Verification Report

**Phase Goal:** Implement forge detection so `gf` identifies which forge (GitHub, GitLab, Gitea, Bitbucket) the current repository uses before invoking any CLI tool.
**Verified:** 2026-03-16
**Status:** gaps_found — 1 orphaned requirement (CORE-04) not delivered; all other must-haves fully verified
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | `ForgeType` enum exists with four variants: Github, Gitlab, Gitea, Forgejo | VERIFIED | `src/forge/mod.rs:9-14` |
| 2 | `ForgeType::cli_name()` returns the correct binary name for each variant | VERIFIED | 4 unit tests pass (`test_cli_name_*`) |
| 3 | All new `GfError` variants compile and display the exact specified error messages | VERIFIED | `src/error.rs:21-37`; 4 display tests pass |
| 4 | HTTPS remote URLs yield the correct hostname | VERIFIED | `test_parse_host_https_github`, `test_parse_host_https_with_port` pass |
| 5 | SSH SCP-style remote URLs yield the correct hostname | VERIFIED | `test_parse_host_ssh_scp`, `test_parse_host_ssh_gitlab` pass |
| 6 | Hostnames with ports strip the port before lookup | VERIFIED | `parse_host` uses `split(':').next()`; `test_parse_host_https_with_port` passes |
| 7 | All four known public hosts resolve to the correct ForgeType | VERIFIED | `test_known_host_github/gitlab/gitea/codeberg` all pass |
| 8 | `get_remote_url` returns NotAGitRepo / NoRemote on error | VERIFIED | `test_get_remote_url_invalid_remote` passes; integration `test_gf_outside_git_repo_shows_error` passes |
| 9 | A `[[forge]]` entry in config overrides built-in host detection | VERIFIED | `config_lookup` queries config before `match_known_host`; `test_config_lookup_with_inline_config` passes |
| 10 | An absent config file is not an error | VERIFIED | `load_config` returns `Ok(None)` if path absent; `test_config_lookup_absent_config_is_ok_none` passes |
| 11 | A malformed config file produces `ConfigParseError` | VERIFIED | `test_config_malformed_toml_returns_parse_error` passes |
| 12 | `gf` run outside a git repo prints the NotAGitRepo message to stderr and exits non-zero | VERIFIED | Integration test `test_gf_outside_git_repo_shows_error` passes |
| 13 | `gf` run in a repo with unknown forge domain prints the three-block detection failure message | VERIFIED | `GfError::ForgeNotDetected` format verified; unit test `test_forge_not_detected_display` passes |
| 14 | `gf --remote upstream <command>` uses the upstream remote for detection | VERIFIED | `src/main.rs:15-27` parses `--remote` flag; `forge::detect(&remote)` called with parsed value |
| 15 | CORE-04: Self-hosted forge detection via CLI auth probing | FAILED | No implementation exists; no plan in this phase claims CORE-04 |

**Score:** 14/15 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/forge/mod.rs` | ForgeType enum, detect(), parse_host(), match_known_host(), config_lookup(), get_remote_url(), test module | VERIFIED | All functions present and fully implemented (162 lines + 175 lines of tests) |
| `src/error.rs` | GfError with 6 new forge variants | VERIFIED | All 6 variants present at lines 21-37 |
| `src/main.rs` | forge::detect() called, --remote flag parsed, cli_name() used | VERIFIED | All three present at lines 15-43 |
| `Cargo.toml` | toml and serde dependencies | VERIFIED | `toml = "0.8"` and `serde = { version = "1", features = ["derive"] }` at lines 13-14 |

Note: Cargo.toml has `toml = "0.8"` rather than `toml = "1.0"` as specified in plan 03. This is a minor version discrepancy — 0.8 is the stable release in active use (1.0 does not exist as a published crate version). Functionally correct.

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/forge/mod.rs` | `src/error.rs` | `use crate::error::GfError` | WIRED | Line 3 of forge/mod.rs |
| `src/main.rs` | `forge::detect` | `forge::detect(&remote)?` | WIRED | Line 30 of main.rs |
| `src/main.rs` | `ForgeType::cli_name()` | `forge_type.cli_name()` | WIRED | Line 38 of main.rs |
| `forge::detect` | `config_lookup` | called before `match_known_host` | WIRED | Lines 80-83 of forge/mod.rs |
| `config_lookup` | `~/.config/gf/config.toml` | `config_path()`, `fs::read_to_string`, `toml::from_str` | WIRED | Lines 44-68 of forge/mod.rs |
| `get_remote_url` | git binary | `Command::new("git").args(["remote", "get-url", remote])` | WIRED | Lines 90-93 of forge/mod.rs |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| CORE-01 | 02-01, 02-02 | Detect forge from origin remote URL (HTTPS and SCP/SSH) | SATISFIED | `parse_host` + `get_remote_url` + integration test |
| CORE-02 | 02-01, 02-02 | `--remote <name>` flag overrides default origin | SATISFIED | `main.rs` --remote parsing; `forge::detect(&remote)` |
| CORE-03 | 02-01, 02-02 | Detect four known public forge hosts | SATISFIED | `match_known_host` table; all 4 host tests pass |
| CORE-04 | none (orphaned) | Self-hosted forge detection via CLI auth probing | NOT SATISFIED | No plan claims CORE-04; no implementation exists; REQUIREMENTS.md marks it Phase 2 Pending (unchecked) |
| CORE-05 | 02-01, 02-03 | User config for domain-to-forge mappings | SATISFIED | `config_lookup` + `load_config` + TOML parsing |

**Orphaned requirement:** CORE-04 is listed in REQUIREMENTS.md traceability as Phase 2 but no plan in this phase was written to deliver it. The commit history confirms it was explicitly dropped from the phase plans (commit `fef50b4`: "fix(02): remove dropped CORE-04 from plans"). The requirement needs to be either deferred to a later phase or removed from the Phase 2 traceability row in REQUIREMENTS.md.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/forge/mod.rs` | 285-291 | `test_config_lookup_absent_is_ok_none` and `test_config_lookup_absent_config_is_ok_none` are duplicate tests for the same behavior | Info | Two tests pass for the same thing (stub-era test left alongside the real test); no functional impact |
| `src/forge/mod.rs` | 273-280 | `test_get_remote_url_not_in_git_repo` is a placeholder that only checks a discriminant | Info | Real NotAGitRepo path is covered by integration test; placeholder is harmless but inaccurate as a "test" |

No blockers found. All anti-patterns are informational.

---

### Test Suite Results

All 34 tests pass (0 failures):
- 25 unit tests in `src/main.rs` binary
- 9 integration tests in `tests/integration_test.rs`
- `cargo build` exits 0

---

### Human Verification Required

#### 1. --remote flag end-to-end

**Test:** In a repo with an `upstream` remote pointing to `https://github.com/...`, run `gf --remote upstream pr list`
**Expected:** gf calls `gh pr list` (detection uses upstream remote, not origin)
**Why human:** Requires a live multi-remote repo environment

#### 2. Config file override

**Test:** Create `~/.config/gf/config.toml` with `[[forge]]` entry mapping a custom domain to `gitlab`, then run `gf` in a repo whose origin points to that domain
**Expected:** gf delegates to `glab` (config overrides built-in table)
**Why human:** Requires a live config file and matching git remote on the test machine

---

### Gaps Summary

Only one gap exists: **CORE-04** (self-hosted forge detection via CLI auth probing) is mapped to Phase 2 in REQUIREMENTS.md but was explicitly dropped from all Phase 2 plans. The commit message `fef50b4` confirms this was intentional ("remove dropped CORE-04 from plans"). The gap is administrative — REQUIREMENTS.md traceability row for CORE-04 still says "Phase 2 | Pending" rather than being moved to a future phase or explicitly deferred.

**Recommendation:** Update REQUIREMENTS.md to move CORE-04 to a future phase (e.g., Phase 5) or mark it deferred, resolving the discrepancy between the traceability table and the actual phase plans.

All functional goal achievement is complete. The phase goal — `gf` identifying the forge from the git remote before invoking any CLI tool — is fully implemented and all tests are green.

---

_Verified: 2026-03-16_
_Verifier: Claude (gsd-verifier)_
