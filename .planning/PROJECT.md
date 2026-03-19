# gf (git forge)

## What This Is

`gf` is a Rust CLI that wraps `gh`, `glab`, `tea`, and `fj` to provide a unified command interface across all major git forges. It auto-detects which forge you're on from the git remote — including self-hosted instances via CLI auth probing — and delegates to the appropriate CLI transparently. `gf pr list`, `gf issue create`, `gf browse file.rs:42-55` all just work, whether you're on GitHub, GitLab, Gitea, or Forgejo.

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
- ✓ PR list, merge, checkout, review, approve, browse commands — v1.1
- ✓ Repo clone command — v1.1
- ✓ Issue list, view, create, close, reopen, browse commands — v1.1
- ✓ Line-range deep-linking in browse (file.rs:42-55) — v1.1
- ✓ Self-hosted forge detection via CLI auth probing with cache (CORE-04) — v1.1
- ✓ Audit and fix flag normalization mappings across all forge CLIs — v1.1
- ✓ PR CI status viewing (PR-08) — v1.2
- ✓ Add/remove reviewers on PRs (PR-09) — v1.2
- ✓ Comment on issues (ISSUE-07) — v1.2
- ✓ Assign/remove labels on issues (ISSUE-08) — v1.2

### Active

(none — all v1.2 requirements validated)

### Out of Scope

- Own config file / centralized token management — auth is fully delegated to gh/glab/tea/fj
- Multi-remote forge routing (non-origin remotes) — use --remote flag for explicit override

## Context

Shipped v1.2 with ~4,000 LOC Rust. Tech stack: Rust, clap 4, webbrowser, toml, serde.

- `gf` wraps `gh`, `glab`, `tea`, and `fj` — normalizes the common command subset and passes through the rest.
- Complete command surface: PR lifecycle (list/merge/checkout/review/approve/checks/comment/edit), issues (list/view/create/close/reopen/comment/edit), repo (clone/fork/create/view), auth, browse.
- 469 tests: 97 unit, 97 translation, 240 macro-based (translation + audit + unsupported), 35 integration. Zero warnings.
- Self-hosted forges supported via config.toml mappings AND automatic CLI auth probing with persistent cache.
- `tea`'s browse is broken — `gf browse` is implemented natively for all forges with line-range deep-linking.
- The tool feels like a thin, transparent router — `exec()` replaces the process on Unix for zero overhead.

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
| UnsupportedFeature error pattern | Return clear errors for forge+command combos that don't work, rather than silently dropping flags | ✓ Good — explicit UX across all forges |
| Declarative test macros | translation_test!, audit_test!, unsupported_test! generate test functions from tables | ✓ Good — 165 tests from ~200 lines of declarations |
| detect_from_host for browse | Browse uses full detection chain (config → known → cache → probe) without needing git remote name | ✓ Good — self-hosted browse works after first probe |

---
*Last updated: 2026-03-19 after v1.2 milestone completion*
