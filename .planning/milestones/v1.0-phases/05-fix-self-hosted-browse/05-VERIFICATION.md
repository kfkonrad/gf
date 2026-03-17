---
phase: 05-fix-self-hosted-browse
verified: 2026-03-16T22:30:00Z
status: passed
score: 3/3 must-haves verified
re_verification: false
---

# Phase 05: Fix Self-Hosted Browse — Verification Report

**Phase Goal:** `gf browse` consults `forge::config_lookup()` for self-hosted forge users, matching the behavior of all other `gf` commands
**Verified:** 2026-03-16T22:30:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A user with a self-hosted forge domain in ~/.config/gf/config.toml can run `gf browse` without getting ForgeNotDetected | VERIFIED | `resolve_forge_type` in `src/browse/mod.rs` calls `config_lookup(host)?` before the known-host match; `test_resolve_forge_type_self_hosted_via_config` proves the path end-to-end |
| 2 | `browse::resolve_forge_type()` calls `forge::config_lookup()` before returning an error | VERIFIED | Lines 127-129 of `src/browse/mod.rs`: `if let Some(forge_type) = config_lookup(host)? { return Ok(forge_type); }` — config check precedes the known-host match and the ForgeNotDetected error arm |
| 3 | All existing browse and forge tests still pass | VERIFIED | `cargo test -- --test-threads=1`: 25 passed, 0 failed, 0 ignored |

**Score:** 3/3 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/forge/mod.rs` | Public `config_lookup` function | VERIFIED | Line 180: `pub fn config_lookup(host: &str) -> Result<Option<ForgeType>, GfError>` |
| `src/browse/mod.rs` | Config-aware `resolve_forge_type` + self-hosted tests | VERIFIED | Imports `config_lookup` on line 5, calls it on line 127, two new tests at lines 385-405 |

Both artifacts exist, are substantive, and are wired together.

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/browse/mod.rs` | `src/forge/mod.rs` | `crate::forge::config_lookup` call in `resolve_forge_type` | VERIFIED | Import: `use crate::forge::{config_lookup, parse_remote_parts, ForgeType};` (line 5). Call site: `if let Some(forge_type) = config_lookup(host)?` (line 127). Response is used: returns `Ok(forge_type)` when `Some`. |

---

### Requirements Coverage

| Requirement | Description | Status | Evidence |
|-------------|-------------|--------|----------|
| CORE-05 | User can configure domain-to-forge-type mappings in config.toml for forges that cannot be auto-detected | SATISFIED | `pub fn config_lookup` reads `~/.config/gf/config.toml`; browse now consults it, closing the gap |
| BROWSE-01 | User can run `gf browse` to open the current repo in the browser at the correct forge URL | SATISFIED | Self-hosted domains now resolve via config; integration tests (25 passing) confirm browse wiring |
| BROWSE-02 | `gf browse` uses the current branch by default; falls back to HEAD SHA if detached | SATISFIED | `resolve_ref` logic unchanged; existing tests `test_resolve_ref_branch_override` and `test_resolve_ref_branch_override_is_not_sha` continue to pass |
| BROWSE-03 | User can run `gf browse <file>` to open a specific file in the browser | SATISFIED | File URL construction unchanged; `test_build_file_url_*` tests pass |
| BROWSE-04 | User can specify `--branch <name>` to override the detected branch | SATISFIED | `--branch` handling unchanged; `test_resolve_ref_branch_override` passes |

All five requirement IDs declared in the PLAN frontmatter are satisfied. REQUIREMENTS.md confirms all five are marked complete with "Phase 5 closes browse gap" notes.

No orphaned requirements found — every ID from the plan maps to a passing test and visible implementation.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| None | — | — | — |

No TODO, FIXME, placeholder comments, empty implementations, or stub returns found in the modified files. The stale NOTE comment ("NOTE: Self-hosted instances need...") that existed in the original `resolve_forge_type` was removed as planned.

---

### Human Verification Required

None. All behavioral claims are exercisable via unit tests:

- Config-file lookup through `resolve_forge_type` is tested by `test_resolve_forge_type_self_hosted_via_config` (writes a real config.toml to a temp dir, sets HOME, asserts `Ok(ForgeType::Gitlab)`).
- Unknown-host fallback is tested by `test_resolve_forge_type_self_hosted_unknown_still_errors`.
- Browser-open behavior (`webbrowser::open`) is excluded from automated tests by the `--no-browser` flag path exercised in integration tests — this is pre-existing, not a gap introduced in this phase.

---

### Summary

Phase 05 achieved its goal. The single structural gap identified in the v1.0 audit (FINDING-02: browse ignored config.toml domain mappings) is closed. The fix is minimal and correct:

1. `config_lookup` was made `pub` in `src/forge/mod.rs` — one character change, no logic changes.
2. `browse::resolve_forge_type` was updated to call `config_lookup` before the known-host table, mirroring the priority order already used by `forge::detect()`.
3. Two unit tests were added proving the self-hosted path works and the unknown-host error path is preserved.

Both task commits (`c9855da`, `5eccff4`) exist in git history. The full test suite (25 tests) passes with zero failures.

---

_Verified: 2026-03-16T22:30:00Z_
_Verifier: Claude (gsd-verifier)_
