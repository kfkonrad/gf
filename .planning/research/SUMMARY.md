# Project Research Summary

**Project:** gf — unified git forge CLI wrapper (v1.1)
**Domain:** Rust CLI subprocess router / forge multiplexer
**Researched:** 2026-03-17
**Confidence:** HIGH

## Executive Summary

`gf` is a thin, transparent Rust CLI that routes forge commands (`pr`, `issue`, `repo`, `browse`) to the appropriate underlying forge CLI (`gh`, `glab`, `tea`, `fj`) based on the git remote URL. The v1.1 milestone extends the v1.0 foundation to complete the PR workflow (`list`, `merge`, `checkout`, `review`), add issue commands (`list`, `view`, `create`), `repo clone`, and line-range browse (`file.rs:42-55`). The recommended approach is to follow the established `detect → translate → exec` pattern already proven in v1.0, adding new commands as translator functions inside the appropriate adapter modules with no changes to `main.rs` and no new mandatory dependencies.

The key architectural constraint that governs all decisions is that `gf` must remain a pure delegation layer: it never calls forge APIs directly, never manages authentication, and never captures subprocess output for delegation commands. All four forge CLIs diverge significantly at the subcommand and flag level — `tea` in particular uses different noun structures (`pulls`, `issues` plural, top-level `clone`) — so every new command requires a per-forge verification step before the adapter is written. The flag normalization audit is not optional; silent flag drops are the most common class of regression in this codebase.

The main risks for v1.1 are: (1) flag/subcommand mismatches that compile correctly but produce wrong forge CLI invocations, detectable only with end-to-end tests that exercise `build_cli()` and `translate_*()` together; (2) `glab mr approve` being a subcommand rather than a flag, requiring subcommand routing not just flag remapping in the PR review adapter; and (3) CORE-04 (self-hosted CLI probing) adding blocking subprocess calls to the hot path if not gated correctly behind the config lookup fallback. The line-range browse GitLab fragment format (`#L42-55` vs `#L42-L55`) is a subtle correctness hazard that requires per-forge test assertions.

## Key Findings

### Recommended Stack

No new mandatory dependencies are required for v1.1. The existing stack — clap 4.6, anyhow 1.0, thiserror 2.0, `which` 8.0, `webbrowser` 1.2, and `std::process::Command` — covers all new features. `serde_json` should be added only if the `tea logins list --output simple` text parse proves fragile for CORE-04; the tabular text format is preferred and avoids the dependency entirely.

**Core technologies:**
- `clap 4.6` (derive): subcommand dispatch and passthrough arg capture — the only parser with `trailing_var_arg` + external subcommand support; `allow_hyphen_values = true` required for transparent flag passthrough
- `std::process::Command` (stdlib): subprocess delegation — use `Command::status()` (not `output()`) to inherit all stdio streams and preserve TTY detection in child CLIs
- `anyhow` + `thiserror`: error handling — `thiserror` for matchable error variants (forge detection failures, unsupported commands per forge), `anyhow` for human-readable output at the CLI boundary
- `webbrowser 1.2`: browse URL opening — explicit browser guarantee prevents accidentally opening a file manager

See `.planning/research/STACK.md` for full rationale, alternatives considered, and what not to use.

### Expected Features

All v1.1 features are P1 table stakes. Every new command is implementable with the existing adapter pattern; complexity comes from per-forge divergence in subcommand names and flag shapes, not from technical novelty.

**Must have (table stakes — v1.1):**
- `gf pr list` — core workflow; requires `--state` normalization (gh=enum, glab=separate flags, tea=enum)
- `gf pr merge` — closes the PR workflow; merge strategy flags differ substantially across all four CLIs; delete-branch flag has three different names
- `gf pr checkout` — review workflow; `--branch` vs `--branch-name` normalization needed
- `gf pr review` — approve/request-changes; glab uses `mr approve` as a separate subcommand; tea/fj have no equivalent — emit a warning, not a silent pass-through
- `gf repo clone` — onboarding; `tea` uses a top-level `clone` command, not `tea repo clone`
- `gf issue list / view / create` — issue triage; `fj` uses `issue search` for listing (no `list` subcommand)
- `gf browse file.rs:42-55` — line-range deep-linking; GitLab fragment (`#L42-55`) differs from all others
- Flag normalization audit — correctness baseline; every canonical flag must have an end-to-end test from `build_cli()` through `translate_*()`

**Should have (v1.x after validation):**
- CORE-04 self-hosted forge detection via CLI auth probing — removes config.toml manual step; deferred because fragile in hot path
- `gf issue close` / `gf issue comment` — issue workflow completeness

**Defer (v2+):**
- `gf pr status` / CI status — requires direct forge API calls; violates no-API constraint
- Batch operations — niche, high complexity

See `.planning/research/FEATURES.md` for the full flag normalization reference and per-forge command coverage table.

### Architecture Approach

The v1.0 codebase establishes a clean five-step pipeline in `main.rs`: build clap tree → parse args → detect forge from remote URL → translate subcommand to forge-specific args → exec the forge CLI binary. All new v1.1 commands follow this same pipeline without changes to `main.rs`. New commands are added in three places: `cmd/mod.rs` (clap definition), the appropriate `adapter/*.rs` (translation logic), and `adapter/mod.rs` (dispatch arm). One structural improvement v1.1 should make: `browse/mod.rs` duplicates the known-host match table from `forge/mod.rs`; expose `forge::match_known_host()` as `pub(crate)` to eliminate the duplication before CORE-04 adds another consumer of that logic.

**Major components:**
1. `forge/mod.rs` — ForgeType enum, remote URL detection, config_lookup(), parse_remote_parts(); v1.1 adds optional CORE-04 probe fallback as a new detection tier after known-host matching fails
2. `adapter/` — translation layer: one module per command domain (`pr.rs` extended, `repo_auth.rs` extended, `issue.rs` new); each translator is `translate_<subcommand>_<verb>(forge, cmd_name, matches) -> Vec<String>`
3. `browse/mod.rs` — native URL construction for all four forges; v1.1 adds `:start[-end]` line-range parsing and per-forge fragment appending
4. `runner.rs` — `exec()` on Unix (process replacement), spawn+wait on Windows; no changes needed for v1.1
5. `cmd/mod.rs` — clap command tree receiving all new subcommand definitions

See `.planning/research/ARCHITECTURE.md` for the full structure diagram, data flow, anti-patterns, and recommended build order.

### Critical Pitfalls

1. **Flag declared in clap but not wired in adapter (silent drop)** — write end-to-end tests that go from `build_cli().try_get_matches_from()` through the full `translate_*()` call; testing parsing and translation in isolation as the only coverage for a canonical flag guarantees silent regressions
2. **`glab mr approve` is a subcommand, not a flag** — `translate_pr_review` must route to a different subcommand for GitLab, not remap a flag; a test asserting `glab mr approve` appears in the output Vec is mandatory before shipping
3. **Line-range GitLab fragment format** — GitLab uses `#L42-55` (no `L` before end line); GitHub/Gitea/Forgejo all use `#L42-L55`; write per-forge unit tests asserting the exact fragment string
4. **CORE-04 self-hosted probing in hot path** — probe only when `config_lookup()` returns None; never run on every invocation; gate as opt-in or experimental; cache per-host result in `~/.cache/gf/`
5. **`tea` subcommand noun divergence** — `tea` uses `pulls`, `issues` (plural), and top-level `clone`; run `tea help <noun>` before writing each adapter; do not assume `tea <noun> <verb>` exists

See `.planning/research/PITFALLS.md` for all 17 pitfalls, the "looks done but isn't" checklist, and security considerations.

## Implications for Roadmap

The ARCHITECTURE.md build order is grounded in actual code dependencies and drives phase sequencing directly.

### Phase 1: Browse Line-Range + Known-Host Deduplication

**Rationale:** Self-contained modification to `browse/mod.rs` with no new modules or dependencies. Ships immediately and validates the per-forge URL fragment approach before building more complex adapters. The known-host deduplication fix (`forge::match_known_host()` as `pub(crate)`) should be done here before CORE-04 adds another consumer of that logic in a later phase.
**Delivers:** `gf browse src/file.rs:42-55` opens the correct line range in the browser for all four forges; known-host match table deduplicated to single source of truth
**Addresses:** Line-range browse (FEATURES.md P1)
**Avoids:** Pitfall 15 (GitLab fragment format `#L42-55`) — requires per-forge unit tests for all four forges before merge

### Phase 2: Flag Normalization Audit

**Rationale:** The audit produces no new user-visible commands but establishes correct flag translation tables that all subsequent adapter phases depend on. Doing this before new command phases ensures they are built on verified flag names rather than assumptions. Discovered mismatches here are cheaper to fix than after adapters are written.
**Delivers:** Verified flag mapping tables for `pr list`, `pr merge`, `pr checkout`, `pr review`, `issue list/view/create`, `repo clone` across all four forge CLIs; end-to-end test coverage for all currently declared canonical flags in existing adapters
**Addresses:** Flag normalization audit (FEATURES.md P1)
**Avoids:** Pitfall 11 (silent flag drop), Pitfall 13 (merge strategy mismatch across forge CLIs)

### Phase 3: PR Workflow Commands (`list`, `merge`, `checkout`, `review`)

**Rationale:** The PR workflow is the highest-value command group (P1, HIGH user value, LOW-MEDIUM cost). These commands share `adapter/pr.rs` and can be implemented together. `pr review` is included despite the GitLab structural exception because deferring it leaves a glaringly incomplete PR workflow.
**Delivers:** Complete PR lifecycle in `gf`: list open PRs, merge with strategy, checkout for review, approve/request-changes
**Addresses:** `gf pr list`, `gf pr merge`, `gf pr checkout`, `gf pr review` (FEATURES.md P1)
**Avoids:** Pitfall 14 (`glab mr approve` subcommand routing must be routed, not remapped), Pitfall 13 (merge strategy flags — squash/rebase/delete-branch translation per forge)
**Implementation note:** For `gf pr review` on tea/fj, emit a clear `CommandNotSupported` error, not a silent empty passthrough

### Phase 4: Issues and Repo Clone

**Rationale:** Issues and clone are independent of PR workflow at the code level. `adapter/issue.rs` is a new module that mirrors `pr.rs`; the established pattern from Phase 3 makes this straightforward. Clone is a minimal addition to `adapter/repo_auth.rs`.
**Delivers:** `gf issue list/view/create`, `gf repo clone` — completes the standard forge workflow outside PRs
**Addresses:** Issue commands, `gf repo clone` (FEATURES.md P1)
**Avoids:** Pitfall 12 (tea noun divergence — verify `tea issues` plural and top-level `clone` before writing), Pitfall 16 (clone input forms — test both `alice/repo` and `https://host/alice/repo`)

### Phase 5: CORE-04 Self-Hosted Forge Detection (opt-in)

**Rationale:** Deferred from v1.0 as "too fragile." Added last because it modifies `forge::detect()` — the function every other command relies on — and has the highest risk of latency or detection regressions if implemented incorrectly. Must run only when `config_lookup()` and `match_known_host()` both fail. Ship as experimental.
**Delivers:** Self-hosted users who have authenticated a forge CLI can have `gf` detect the forge without a manual `config.toml` entry; probe result cached in `~/.cache/gf/`
**Addresses:** CORE-04 self-hosted detection (FEATURES.md P2)
**Avoids:** Pitfall 17 (probing in hot path), Pitfall 9 (self-hosted detection no ground truth)
**Implementation note:** Mark as experimental in initial release; `fj` has no auth probe — config-file-only path for Forgejo self-hosted

### Phase Ordering Rationale

- Browse line-range ships first: isolated, validates per-forge fragment dispatch, and delivers the deduplication cleanup before CORE-04 would otherwise be the first CORE-04 consumer of `forge::match_known_host()`
- Flag audit before PR/issue/clone adapters: discovered flag mismatches change the adapter implementation; doing the audit after would require rework
- PR commands before issues/clone: higher user value, `pr.rs` is the template module that `issue.rs` mirrors
- CORE-04 last: modifies the detection pipeline that all other phases rely on; adding it after those phases are stable minimizes regression risk

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 3 (PR review):** `glab mr approve` structure needs verification against current glab docs before implementation; confirm whether `--request-changes` equivalent exists for glab. Tea/fj passthrough-vs-error behavior needs an explicit design decision before implementation starts.
- **Phase 5 (CORE-04):** Caching strategy for probe results (file format, TTL, invalidation) needs a concrete design before implementation. `fj` config-file-only path needs clear user-facing documentation design.

Phases with standard patterns (skip research-phase):

- **Phase 1 (browse line-range):** Fragment format already researched and confirmed for all four forges; implementation is two stdlib `split` calls
- **Phase 2 (flag audit):** Mechanical survey of forge CLI `--help` output; no architectural decisions required
- **Phase 4 (issues/clone):** Follows the established adapter pattern from Phase 3; only tea noun verification needed before writing

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Existing Cargo.toml confirmed; no new dependencies; all patterns already working in v1.0 codebase |
| Features | HIGH | Verified against live `gh`, `glab`, `tea`, `fj` CLI `--help` output and existing v1.0 adapter patterns |
| Architecture | HIGH | Direct source code analysis of the v1.0 implementation; all patterns established and proven |
| Pitfalls | HIGH | Subprocess/signal mechanics verified via Rust stdlib docs; URL pitfalls confirmed via upstream issues; v1.1-specific pitfalls derived from CLI documentation and codebase inspection |

**Overall confidence:** HIGH

### Gaps to Address

- **`glab mr request-changes`:** Research confirmed `glab mr approve` as a subcommand but did not confirm the request-changes equivalent for glab. Verify against `glab mr --help` before implementing Phase 3.
- **`fj` merge strategies:** `fj` merge strategy flag names (`--method`) confirmed from docs but not verified against a live `fj` binary. Verify before shipping Phase 3.
- **`tea clone` exact argument form:** `tea clone <slug>` confirmed as a top-level command but the exact argument format (slug vs full URL) needs live CLI verification before Phase 4.
- **Self-hosted Gitea `ROOT_URL` subpath in browse URLs:** URL construction for Gitea instances with a non-root subpath (e.g., `https://host/git/`) was not resolved in this research cycle. Flag for Phase 1 edge case testing.

## Sources

### Primary (HIGH confidence)

- `/Users/derkev/tmp/gf-v2/src/` (v1.0 codebase, 2026-03-17) — direct source analysis; all architectural claims derived from actual implementation
- [gh auth status docs](https://cli.github.com/manual/gh_auth_status) — `--hostname` flag confirmed stable; `--json hosts` schema confirmed unstable via GitHub CLI issue #9326
- [glab auth status docs](https://docs.gitlab.com/cli/auth/status/) — `--hostname` flag confirmed in official docs
- [tea CLI.md](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md) — `tea logins list --output simple|json` confirmed
- [ExitStatusExt::signal() — Rust std docs](https://doc.rust-lang.org/std/os/unix/process/trait.ExitStatusExt.html) — signal re-raise pattern
- [ExitStatus::code() — Rust std docs](https://doc.rust-lang.org/std/process/struct.ExitStatus.html) — `None` on signal termination
- GitLab line anchor format — verified against gitlab.com source view URL patterns

### Secondary (MEDIUM confidence)

- [forgejo-cli (fj) Codeberg](https://codeberg.org/Cyborus/forgejo-cli) — no auth status equivalent found; config-file-only determination based on absence of evidence
- GitLab `#L42-55` fragment format — consistent with observed GitLab blob URLs; not from official specification
- Gitea/Forgejo line-range anchor `#L42-L55` — inferred from Gitea codebase deriving from GitHub conventions
- `glab mr approve` as separate subcommand — known from glab CLI design; verify against current docs before Phase 3

### Tertiary (LOW confidence)

- Self-hosted Gitea `ROOT_URL` subpath browse behavior — not researched; flagged as an implementation gap for Phase 1

---
*Research completed: 2026-03-17*
*Ready for roadmap: yes*
