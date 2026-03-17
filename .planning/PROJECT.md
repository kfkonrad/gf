# gf (git forge)

## What This Is

`gf` is a Rust CLI that wraps `gh`, `glab`, `tea`, and `fj` to provide a unified command interface across all major git forges. It auto-detects which forge you're on from the git remote and delegates to the appropriate CLI transparently — so `gf pr create` just works, whether you're on GitHub, GitLab, Gitea, or Forgejo.

## Core Value

One `gf` command syntax that works on any forge, with zero knowledge of which forge you're on.

## Requirements

### Validated

- ✓ Auto-detect forge from git remote (origin by default, --remote to override) — v1.0
- ✓ Unified PR/MR commands: create, view — delegated to underlying CLI — v1.0
- ✓ Unified repo commands: fork, create, view — delegated to underlying CLI — v1.0
- ✓ Unified auth commands: login, logout, status — delegated to underlying CLI — v1.0
- ✓ Canonical flag normalization: known flags translated to forge equivalents; unknown flags passed through as-is — v1.0
- ✓ Native browse command: open repo/file in browser at correct forge URL — v1.0
- ✓ Clear error with install hint when required forge CLI is not on PATH — v1.0
- ✓ Auth delegated to underlying CLIs (gh auth, glab auth, etc.) — v1.0
- ✓ Config file for self-hosted forge domain-to-type mappings — v1.0

### Active

- [ ] PR list, merge, checkout, review commands
- [ ] Repo clone command
- [ ] Issues commands
- [ ] Line-range deep-linking in browse (file.rs:42-55)
- [ ] Self-hosted forge detection via CLI auth probing (CORE-04)

### Out of Scope

- Own config file / centralized token management — auth is fully delegated to gh/glab/tea/fj
- Multi-remote forge routing (non-origin remotes) — use --remote flag for explicit override

## Context

Shipped v1.0 with 2,689 LOC Rust. Tech stack: Rust, clap 4, webbrowser, toml, serde.

- `gf` wraps `gh`, `glab`, `tea`, and `fj` — normalizes the common command subset and passes through the rest.
- `tea`'s browse is broken — `gf browse` is implemented natively for all forges.
- The tool feels like a thin, transparent router — `exec()` replaces the process on Unix for zero overhead.
- Self-hosted forges supported via `~/.config/gf/config.toml` domain-to-forge mappings.

## Constraints

- **Tech stack**: Rust — chosen for performance, single-binary distribution, and strong CLI ecosystem (clap, etc.)
- **Dependencies**: No forge API calls in v1 — all forge operations delegate to existing CLIs
- **Availability**: Requires the relevant forge CLI to be installed; surfaces clear install hints otherwise

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Wrap existing CLIs rather than implement forge APIs | Reduces scope enormously; leverages battle-tested CLI implementations | ✓ Good — shipped v1.0 in 1 day |
| Native browse implementation | tea's browse is broken; building natively ensures correctness across all forges | ✓ Good — correct URLs for all 4 forges |
| Normalize known flags, passthrough unknown | Best of both worlds — canonical UX for common operations, escape hatch for forge-specific extras | ✓ Good — clean adapter pattern |
| Auth fully delegated | Users already have forge CLIs configured; no value in duplicating auth management | ✓ Good — zero auth bugs |
| exec() process replacement on Unix | Zero overhead, TTY inherited automatically | ✓ Good — colors and signals work perfectly |
| clap builder API (not derive) | Precise control over visible_alias placement for mr/pr routing | ✓ Good — aliases work as designed |
| CORE-04 deferred | Self-hosted CLI auth probing too fragile for v1; config file covers the use case | ✓ Good — simpler, more reliable |

---
*Last updated: 2026-03-17 after v1.0 milestone*
