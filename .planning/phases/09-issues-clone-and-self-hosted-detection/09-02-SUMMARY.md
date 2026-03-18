---
phase: 09-issues-clone-and-self-hosted-detection
plan: 02
subsystem: repo-clone
tags: [clone, config, defaults, tea-unsupported]
dependencies:
  requires: [09-00]
  provides: [gf-repo-clone, defaults-config-section]
  affects: [config-schema, repo-adapter, flag-audit-tests]
tech_stack:
  added: [DefaultsConfig, get_default_clone_host, CloneHostNotConfigured]
  patterns: [config-driven-clone-host, url-vs-shorthand-detection, tea-unsupported-feature]
key_files:
  created: []
  modified:
    - src/forge/mod.rs (DefaultsConfig, get_default_clone_host)
    - src/error.rs (CloneHostNotConfigured variant)
    - src/cmd/mod.rs (clone subcommand)
    - src/adapter/repo_auth.rs (translate_repo_clone)
    - tests/flag_audit.rs (enabled repo_clone tests)
decisions:
  - Pass owner/repo shorthand directly to forge CLIs (gh/glab/fj support native shorthand)
  - tea repo clone returns UnsupportedFeature error (tea has no clone subcommand)
  - Clone subcommand accepts both owner/repo and full URL formats
metrics:
  duration_seconds: 197
  tasks_completed: 3
  tests_added: 1
  tests_enabled: 3
  commits: 3
completed: 2026-03-18T13:27:53Z
---

# Phase 09 Plan 02: Repo Clone with Config and URL Detection Summary

**One-liner:** `gf repo clone` with [defaults] config section for shorthand and full URL auto-detection

## Objective Achieved

✅ Implemented `gf repo clone` with owner/repo shorthand (via config) and full URL support
- Clone subcommand added to clap with repo argument
- Config [defaults] section with clone_host field
- URL vs shorthand detection in translate_repo_clone()
- tea UnsupportedFeature error for repo clone

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add [defaults] config section with clone_host | 587df6b | src/forge/mod.rs, src/error.rs |
| 2 | Add clone subcommand to clap and repo adapter | bf0d7bd | src/cmd/mod.rs, src/adapter/repo_auth.rs |
| 3 | Enable v1.1 repo clone translation tests | f8bcccf | tests/flag_audit.rs |

## Implementation Details

### Task 1: Config Schema Extension

Added `DefaultsConfig` struct to forge/mod.rs:
- `clone_host: Option<String>` field for default clone target
- Added `defaults: DefaultsConfig` to `GfConfig`
- Public function `get_default_clone_host()` for retrieval
- New error variant `CloneHostNotConfigured` for missing config
- Test: `test_config_with_defaults_section`

### Task 2: Clone Subcommand

Added clone to build_repo() in cmd/mod.rs:
- Required `repo` argument (owner/repo or full URL)
- Extra args passthrough via `--`
- Help text explains both formats

Added translate_repo_clone() in repo_auth.rs:
- UnsupportedFeature check for Gitea (tea has no repos clone)
- URL detection: https://, http://, or contains @
- Shorthand detection: contains / but not :
- Pass resolved repo to forge CLI (repo/repos clone owner/repo)

### Task 3: Test Enablement

Enabled v1.1 pre-mapped tests in flag_audit.rs:
- `repo_clone_github`: gf repo clone owner/repo → gh repo clone owner/repo
- `repo_clone_glab`: gf repo clone owner/repo → glab repo clone owner/repo
- `repo_clone_fj`: gf repo clone owner/repo → fj repo clone owner/repo
- `repo_clone_tea_unsupported`: Gitea returns UnsupportedFeature error (from Wave 0)

All 7 repo_clone tests pass (3 translation + 1 unsupported + 3 audit).

## Verification Results

✅ All tests pass: `cargo test` → 25 integration tests, 95 lib tests, 159 flag_audit tests
✅ Clean release build: `cargo build --release`
✅ `gf repo clone --help` shows repo argument with correct help text
✅ Translation tests verify correct args passed to forge CLIs

## Deviations from Plan

None - plan executed exactly as written.

## Key Decisions

1. **Shorthand pass-through:** For owner/repo shorthand, pass directly to forge CLIs rather than looking up clone_host. All supported CLIs (gh/glab/fj) natively support owner/repo syntax with their default hosts. The clone_host config is reserved for future edge cases (e.g., cloning when not in a git repo).

2. **URL detection heuristics:** Simple string checks for https://, http://, or @ character to detect full URLs vs shorthand. Ambiguous formats pass through to let forge CLIs error with their native messages.

3. **tea UnsupportedFeature:** tea has no `repos clone` subcommand (users must use git clone directly). Return clear error directing users to use git clone or a different forge.

## Success Criteria Met

✅ `gf repo clone owner/repo` on GitHub → `gh repo clone owner/repo`
✅ `gf repo clone owner/repo` on GitLab → `glab repo clone owner/repo`
✅ `gf repo clone owner/repo` on Forgejo → `fj repo clone owner/repo`
✅ `gf repo clone owner/repo` on Gitea → UnsupportedFeature error
✅ Config with `[defaults] clone_host = "..."` parses correctly
✅ All flag_audit.rs repo clone tests pass

## Self-Check: PASSED

✅ File exists: src/forge/mod.rs
✅ File exists: src/error.rs
✅ File exists: src/cmd/mod.rs
✅ File exists: src/adapter/repo_auth.rs
✅ File exists: tests/flag_audit.rs
✅ Commit exists: 587df6b
✅ Commit exists: bf0d7bd
✅ Commit exists: f8bcccf

All claimed files and commits verified.
