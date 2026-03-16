---
phase: 02-forge-detection
plan: 03
subsystem: forge-detection
tags: [rust, toml, serde, config, forge-detection, integration-tests]

requires:
  - phase: 02-01
    provides: ForgeType enum, GfError variants, detect() stub
  - phase: 02-02
    provides: parse_host, match_known_host, get_remote_url, integration test stubs

provides:
  - config_lookup: reads ~/.config/gf/config.toml, matches domain to ForgeType override
  - load_config / config_path: TOML loading helpers with absent-is-ok-none semantics
  - main.rs Phase 2 wiring: forge::detect() called before runner::run(), --remote flag parsed
  - Full end-to-end forge detection: git remote URL → hostname → config or built-in → CLI binary

affects:
  - 03 (clap integration will replace hand-rolled --remote flag parsing in main.rs)

tech-stack:
  added:
    - "toml 0.8 — TOML deserialization"
    - "serde 1 with derive feature — Deserialize for ForgeType, GfConfig, ForgeEntry"
    - "tempfile 3 (dev) — temp dir helpers for integration tests"
  patterns:
    - "Absent config file is not an error — Ok(None) semantics"
    - "config_lookup checked before match_known_host — config overrides built-in table"
    - "ForgeType derives Deserialize with serde rename_all lowercase for TOML type field"
    - "Integration tests use symlink-only temp bin dirs to isolate CLI not-found error path"
    - "Fake gh shell scripts wrapping exit_with binary for exit code propagation tests"

key-files:
  created: []
  modified:
    - src/forge/mod.rs
    - src/main.rs
    - tests/integration_test.rs
    - Cargo.toml

key-decisions:
  - "toml 0.8 used instead of 1.0 (plan specified 1.0 which does not exist on crates.io; 0.8 is current stable)"
  - "GfConfig and ForgeEntry derive Debug in addition to Deserialize to satisfy unwrap_err bound in tests"
  - "Integration tests rewritten with temp git repos + isolated PATH to survive Phase 2 interface change"
  - "make_git_only_bin_dir() creates symlink-only bin dirs — portable across any git install path"

patterns-established:
  - "Config override pattern: config_lookup() returns Some(forge) to short-circuit match_known_host"
  - "Fake CLI pattern: shell script wrapper named after target binary + exec to test binary in PATH"

requirements-completed: [CORE-05]

duration: 20min
completed: 2026-03-16
---

# Phase 2 Plan 03: Config Lookup and main.rs Wiring Summary

**TOML config loading with domain override, --remote flag parsing, and full end-to-end forge detection wired into main.rs; all 34 tests GREEN**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-03-16T14:50:00Z
- **Completed:** 2026-03-16T15:10:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- config_lookup() reads ~/.config/gf/config.toml; absent file returns Ok(None), malformed returns ConfigParseError
- ForgeType derives Deserialize with serde rename_all lowercase — TOML `type = "github"` maps correctly
- main.rs fully replaces Phase 1 placeholder: forge::detect(remote) called first, then runner::run(cli, remaining)
- --remote flag parsed inline before remaining args; consumed by gf and not forwarded to the CLI
- Integration tests rewritten to survive the interface change (forge detection first, no longer CLI-name first arg)
- All 25 unit tests and all 9 integration tests GREEN including the previously-RED test_gf_outside_git_repo_shows_error

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement config_lookup with TOML parsing** - `5599ced` (feat)
2. **Task 2: Wire forge::detect into main.rs with --remote flag** - `749a364` (feat)

## Files Created/Modified
- `src/forge/mod.rs` - GfConfig/ForgeEntry structs, config_path, load_config, config_lookup implemented; Deserialize added to ForgeType; new unit tests
- `src/main.rs` - Phase 1 placeholder replaced with forge detection + --remote flag parsing
- `tests/integration_test.rs` - All tests rewritten for forge detection model; temp repo + isolated PATH helpers added
- `Cargo.toml` - toml 0.8, serde 1 (derive), tempfile 3 (dev) dependencies added

## Decisions Made
- toml 0.8 used (plan specified 1.0 which is unreleased; 0.8 is the current stable series)
- GfConfig and ForgeEntry derive Debug in addition to Deserialize — required by unwrap_err() test bound
- Integration tests fully rewritten rather than deleted — the Phase 1 test behaviors are still valid, just need forge detection context

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added Debug derive to GfConfig and ForgeEntry**
- **Found during:** Task 1 (config tests)
- **Issue:** unwrap_err() on Result<GfConfig, _> requires T: Debug; plan omitted Debug from derive list
- **Fix:** Added Debug to both struct derives
- **Files modified:** src/forge/mod.rs
- **Verification:** cargo test forge::tests::test_config_malformed_toml_returns_parse_error passes
- **Committed in:** 5599ced (Task 1 commit)

**2. [Rule 1 - Bug] Rewrote integration tests for Phase 2 interface change**
- **Found during:** Task 2 (main.rs wiring)
- **Issue:** Phase 1 integration tests passed CLI name as first arg (e.g., `gf gh pr list`); Phase 2 changes interface to forge-detection-first — all Phase 1 tests would break
- **Fix:** Rewrote CORE-06 tests to use temp git repos with github.com remotes and isolated PATH (symlink-only bin dir with only git, no gh); rewrote CORE-07 exit code tests to use fake gh shell scripts wrapping exit_with binary
- **Files modified:** tests/integration_test.rs, Cargo.toml (added tempfile dev dep)
- **Verification:** All 9 integration tests GREEN
- **Committed in:** 749a364 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
- toml crate version: plan specified `toml = "1.0"` but 1.x does not exist on crates.io; used 0.8 (current stable with identical API)
- Integration test isolation: PATH="" caused git itself to fail (git is needed for forge detection); solved with symlink-only bin dir containing only git

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 2 complete: gf performs full forge detection end-to-end
- `~/.config/gf/config.toml` supports domain overrides for self-hosted instances
- --remote flag parses correctly; Phase 3 clap integration will supersede hand-rolled parsing
- All forge detection requirements (CORE-01 through CORE-05) satisfied

---
*Phase: 02-forge-detection*
*Completed: 2026-03-16*
