---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: planning
stopped_at: Completed 04-01-PLAN.md
last_updated: "2026-03-16T16:14:33.185Z"
last_activity: 2026-03-16 — Roadmap created
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 11
  completed_plans: 10
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-16)

**Core value:** One `gf` command syntax that works on any forge, with zero knowledge of which forge you're on
**Current focus:** Phase 1 — Foundation

## Current Position

Phase: 1 of 4 (Foundation)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-03-16 — Roadmap created

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**
- Total plans completed: 0
- Average duration: -
- Total execution time: 0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| - | - | - | - |

**Recent Trend:**
- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
| Phase 01-foundation P01 | 2 | 2 tasks | 6 files |
| Phase 01-foundation P02 | 3 | 2 tasks | 4 files |
| Phase 02-forge-detection P01 | 8 | 2 tasks | 3 files |
| Phase 02-forge-detection P02 | 2 | 2 tasks | 2 files |
| Phase 02-forge-detection P03 | 525548 | 2 tasks | 4 files |
| Phase 03-command-routing P01 | 15 | 3 tasks | 6 files |
| Phase 03-command-routing P03 | 8 | 2 tasks | 1 files |
| Phase 03-command-routing P02 | 5 | 1 tasks | 2 files |
| Phase 03-command-routing P04 | 15 | 2 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Initialization: Wrap existing CLIs rather than implement forge APIs (reduces scope, leverages battle-tested CLIs)
- Initialization: Native browse implementation (tea's browse is broken)
- Initialization: Auth fully delegated (no value in duplicating auth management)
- Initialization: Normalize known flags, passthrough unknown (canonical UX + escape hatch)
- [Phase 01-foundation]: Use thiserror for GfError enum to keep error variants match-able in tests and future phases
- [Phase 01-foundation]: nix dep gated under cfg(windows) only — Unix exec() path needs no signal re-raise
- [Phase 01-foundation]: CliInfo.brew_name uses String not static str to handle unknown CLI names
- [Phase 01-foundation]: TTY inheritance and signal re-raise (exit 130) confirmed in real terminal via human verification
- [Phase 02-forge-detection]: GitCommandFailed uses #[from] io::Error for ergonomic ? conversion; ExecFailed/SpawnFailed retain String context without #[from]
- [Phase 02-forge-detection]: Discriminate NotAGitRepo vs NoRemote by checking stderr.contains('not a git repository') — git uses text not separate exit codes
- [Phase 02-forge-detection]: Integration test test_gf_outside_git_repo_shows_error intentionally RED — main.rs wiring deferred to plan 03
- [Phase 02-forge-detection]: toml 0.8 used instead of 1.0 — 1.x does not exist on crates.io; 0.8 is current stable with identical API
- [Phase 02-forge-detection]: Integration tests rewritten with temp git repos + isolated PATH bin dirs to survive Phase 2 forge-detection-first interface change
- [Phase 03-command-routing]: clap builder API (not derive) used for CLI tree — gives precise control over visible_alias placement for mr/pr routing
- [Phase 03-command-routing]: mr implemented as visible_alias on pr subcommand — clap routes gf mr create to pr handler automatically, no multi-word alias needed
- [Phase 03-command-routing]: GitLab repo create uses --visibility private/public, not --private/--public (Pitfall 4)
- [Phase 03-command-routing]: Tea auth uses logins subcommand (logins add/rm/ls) — no auth subcommand in tea CLI
- [Phase 03-command-routing]: pr view number arg changed from named --number flag to positional arg to match gf pr view 42 syntax
- [Phase 03-command-routing]: clap-based main() wired with clone before get_matches() to preserve cli_cmd for completions generation
- [Phase 03-command-routing]: Integration tests updated from pr list to pr view since clap validates subcommands before forge detection
- [Phase 03-command-routing]: clap-based main() wired with clone before get_matches() to preserve cli_cmd for completions generation
- [Phase 03-command-routing]: Integration tests updated from pr list to pr view since clap validates subcommands before forge detection
- [Phase 04-browse]: resolve_forge_type() in browse module replicates known-host match rather than calling private forge::match_known_host() — avoids making private functions public
- [Phase 04-browse]: mod browse declared in both lib.rs and main.rs — bin and lib are separate Rust crates sharing source files, matching existing pattern
- [Phase 04-browse]: normalize_path() passes relative paths through unchanged; strips repo root prefix only for absolute paths — no filesystem validation since paths may only exist on remote

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 2: Self-hosted forge detection via `gh`/`glab auth status` output parsing needs validation that output format is stable across CLI versions — flag for Phase 2 planning
- Phase 3: `fj` (Forgejo CLI) flag names have MEDIUM confidence — verify `fj pr create` flags against actual binary or Codeberg source during Phase 3 planning

## Session Continuity

Last session: 2026-03-16T16:14:28.779Z
Stopped at: Completed 04-01-PLAN.md
Resume file: None
