---
phase: 03-command-routing
plan: "03"
subsystem: adapter
tags: [rust, clap, forge-translation, repo, auth, gitea, forgejo, gitlab, github]

requires:
  - phase: 03-01
    provides: CLI tree (build_cli), adapter module structure, translate_repo/translate_auth stubs

provides:
  - translate_repo() with full four-forge translation including visibility flag remap and tea "repos" subcommand
  - translate_auth() with full subcommand remap for tea (logins add/rm/ls) and fj (auth add-key/list)
  - 20 unit tests covering REPO-01..03 and AUTH-01..03 requirements

affects: [04-execution, main-wiring]

tech-stack:
  added: []
  patterns:
    - "Forge-specific flag translation via match on ForgeType in dedicated helper functions"
    - "Auth subcommand remap: tea has no 'auth' subcommand, uses 'logins' instead"
    - "Visibility flag remap: glab uses --visibility private/public instead of --private/--public"

key-files:
  created: []
  modified:
    - src/adapter/repo_auth.rs

key-decisions:
  - "GitLab repo create uses --visibility private/public, not --private/--public (Pitfall 4 from RESEARCH.md)"
  - "Tea auth uses 'logins' subcommand (logins add/rm/ls) — no 'auth' subcommand exists in tea CLI"
  - "Forgejo auth login maps to 'auth add-key', auth status maps to 'auth list'"
  - "Tea --hostname flag maps to --url for auth login"
  - "Name is positional for gh/glab in repo create, --name flag for tea/fj"

patterns-established:
  - "translate_*_view/create/fork helpers are private; public translate_repo/translate_auth dispatch via subcommand match"
  - "repo_subcommand_name() centralizes the tea 'repos' vs 'repo' distinction"

requirements-completed: [REPO-01, REPO-02, REPO-03, AUTH-01, AUTH-02, AUTH-03]

duration: 8min
completed: 2026-03-16
---

# Phase 03 Plan 03: Repo and Auth Translation Layer Summary

**Full repo and auth forge translation in src/adapter/repo_auth.rs — visibility flag remap for GitLab, tea subcommand remap from auth to logins, fj auth add-key mapping, with 20 unit tests across all four forges**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-16T15:10:00Z
- **Completed:** 2026-03-16T15:18:00Z
- **Tasks:** 2 (combined into single commit — same file, fully specified implementation)
- **Files modified:** 1

## Accomplishments

- `translate_repo()` handles view/create/fork for GitHub, GitLab, Gitea, Forgejo with correct flag translation
- GitLab `--private` → `--visibility private` (Pitfall 4), GitHub keeps `--private`
- Repository name is positional for gh/glab, `--name` flag for tea/fj
- Tea uses `repos` subcommand (not `repo`)
- `translate_auth()` remaps: tea `auth login` → `logins add`, `auth logout` → `logins rm`, `auth status` → `logins ls`
- Forgejo: `auth login` → `auth add-key`, `auth status` → `auth list`
- Tea `--hostname` → `--url` for auth login
- 20 unit tests, all green; full suite 85 tests, zero regressions

## Task Commits

1. **Tasks 1+2: translate_repo and translate_auth implementation** - `f1140bd` (feat)

## Files Created/Modified

- `src/adapter/repo_auth.rs` - Full implementation replacing stubs: translate_repo(), translate_auth(), and all private helpers with 20 unit tests

## Decisions Made

- Tasks 1 and 2 were committed together because the plan provided both implementations in full and the file is a single logical unit; splitting RED/GREEN across two commits on an already-specified implementation provides no signal value

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `translate_repo` and `translate_auth` are complete and tested — ready for main.rs wiring in Phase 4
- All REPO-01..03 and AUTH-01..03 requirements satisfied

---
*Phase: 03-command-routing*
*Completed: 2026-03-16*
