# forge

## What This Is

`forge` is a Rust CLI that wraps `gh`, `glab`, `tea`, and `fj` to provide a unified command interface across all major git forges. It auto-detects which forge you're on from the git remote and delegates to the appropriate CLI transparently — so `forge pr create` just works, whether you're on GitHub, GitLab, Gitea, or Forgejo.

## Core Value

One command syntax that works on any forge, with zero knowledge of which forge you're on.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Auto-detect forge from git remote (origin by default, --remote to override)
- [ ] Unified PR/MR commands: create, view, list, merge, review — delegated to underlying CLI
- [ ] Unified repo commands: clone, fork, create, view — delegated to underlying CLI
- [ ] Unified auth commands: login, logout, status — delegated to underlying CLI
- [ ] Canonical flag normalization: known flags translated to forge equivalents; unknown flags passed through as-is
- [ ] Native `forge browse` command: open repo/file in browser at correct forge URL
  - Branch defaults to current branch (or HEAD commit if detached)
  - Accepts file path argument to deep-link to specific file
  - Accepts branch flag to override detected branch
- [ ] Clear error with install hint when required forge CLI is not on PATH
- [ ] Auth delegated to underlying CLIs (gh auth, glab auth, etc.)

### Out of Scope

- Own config file / centralized token management — auth is fully delegated to gh/glab/tea/fj
- Issues commands — deferred to v2
- Line-range deep-linking in browse (e.g. file.rs:42-55) — deferred to v2
- Multi-remote forge routing (non-origin remotes) — use --remote flag for explicit override

## Context

- The four forge CLIs (`gh`, `glab`, `tea`, `fj`) each have their own command structure and flag conventions. `forge` normalizes the common subset and passes through the rest.
- `tea`'s browse implementation is known to be partially broken — `forge browse` is implemented natively rather than delegated.
- The tool should feel like a thin, transparent router — not an abstraction layer. Users should be able to drop back to the underlying CLI when needed.

## Constraints

- **Tech stack**: Rust — chosen for performance, single-binary distribution, and strong CLI ecosystem (clap, etc.)
- **Dependencies**: No forge API calls in v1 — all forge operations delegate to existing CLIs
- **Availability**: Requires the relevant forge CLI to be installed; surfaces clear install hints otherwise

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Wrap existing CLIs rather than implement forge APIs | Reduces scope enormously; leverages battle-tested CLI implementations | — Pending |
| Native browse implementation | tea's browse is broken; building natively ensures correctness across all forges | — Pending |
| Normalize known flags, passthrough unknown | Best of both worlds — canonical UX for common operations, escape hatch for forge-specific extras | — Pending |
| Auth fully delegated | Users already have forge CLIs configured; no value in duplicating auth management | — Pending |

---
*Last updated: 2026-03-16 after initialization*
