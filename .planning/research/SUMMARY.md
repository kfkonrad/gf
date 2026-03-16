# Project Research Summary

**Project:** gf (git forge)
**Domain:** Rust CLI wrapper / forge router (gh, glab, tea, fj)
**Researched:** 2026-03-16
**Confidence:** HIGH

## Executive Summary

`gf` is a thin subprocess router, not a forge abstraction layer. The tool's entire value proposition rests on two mechanics: detecting which forge a repo lives on (from the git remote URL) and delegating to the appropriate CLI binary (`gh`, `glab`, `tea`, or `fj`) with flags translated to that CLI's conventions. All four research threads converge on the same design: keep `gf` as transparent as possible — inherit stdio, use `exec()` on Unix, normalize only the flags you must, and pass everything else through unchanged.

The recommended implementation is a layered Rust binary using clap 4 (derive API with `trailing_var_arg`) for argument parsing, a `ForgeAdapter` trait for per-forge command/flag mapping, and `std::process::Command` with `Stdio::inherit()` for all delegation. No async runtime, no config files, no forge API calls in v1. The most complex component is forge detection, which must handle SSH SCP-style remote URLs, `url.insteadOf` git config rewrites, git worktrees, and self-hosted Gitea/Forgejo instances — all before any command can be dispatched.

The primary risk profile is correctness, not complexity. The codebase is small but full of sharp edges around subprocess signal propagation, TTY passthrough, and clap flag interception. Every pitfall identified in research has a known, testable fix — the risk is skipping the edge case tests, not architectural misjudgment. Build the subprocess delegation layer with signal re-raise and `Stdio::inherit()` from day one; retrofitting this is painful.

## Key Findings

### Recommended Stack

The stack is narrow and well-chosen for the problem. Clap 4 with the derive API and `trailing_var_arg = true` is the only viable argument parser — no competitor supports both external subcommand capture and flag passthrough. `std::process::Command` from stdlib covers all delegation needs; no async runtime is warranted. `which` handles CLI presence detection cleanly. `webbrowser` is preferred over `open` for the browse command because it provides an explicit browser guarantee (not file manager). Error handling follows the library-vs-binary split: `thiserror` for matchable domain errors (forge detection layer), `anyhow` for ergonomic propagation at the CLI surface.

**Core technologies:**
- `clap 4.6` (derive): argument parsing and subcommand dispatch — the only parser with `trailing_var_arg` + external subcommand support
- `anyhow 1.0`: CLI-level error propagation with human-readable context chaining
- `thiserror 2.0`: structured error types for the forge detection and routing layer (matchable variants)
- `which 8.0`: PATH presence check for forge CLIs — handles Windows extension resolution correctly
- `webbrowser 1.2`: cross-platform browser open for `gf browse` — explicit browser guarantee
- `std::process::Command` (stdlib): subprocess delegation with full stdio inheritance

See `.planning/research/STACK.md` for full rationale, alternatives considered, and what to avoid.

### Expected Features

The highest-frequency operations are PR creation and listing; forge auto-detection is the prerequisite for everything. Browse is the one command that must be implemented natively (tea's browse is broken). Flag normalization for `pr create` (`--body` vs `--description`, `--base` vs `--target-branch`) is the key UX differentiator.

**Must have (table stakes):**
- Forge auto-detection from git remote — nothing else works without it
- Missing CLI detection with install hint — prevents confusing failures
- `gf pr create` with flag normalization (`--body`, `--base`, `--head`) — highest-frequency write operation
- `gf pr list`, `gf pr view`, `gf pr merge` — complete the core PR loop
- `gf repo clone`, `gf repo view` — universal entry points
- `gf auth login/logout/status` — fully delegated passthrough (low effort)
- `gf browse` native implementation — tea's is broken; file path deep-link in scope for v1
- `--remote` global flag — escape hatch for non-origin remotes

**Should have (competitive):**
- `gf pr checkout`, `gf pr review/approve` — complete PR review workflow
- `gf repo create`, `gf repo fork` — write operations; validate reads first
- Browse line-range support (`file.rs:42-55`) — after URL construction abstraction is stable
- Verbose/debug mode showing the expanded underlying command

**Defer (v2+):**
- Issues commands (`gf issue create/list/view/close`) — doubles scope; validate PR workflow first
- CI/pipeline passthrough (`gf ci`, `gf run`) — normalization nearly impossible across forge models
- Release commands — useful but not core to the PR workflow being validated

See `.planning/research/FEATURES.md` for the full flag normalization reference and per-forge command coverage table.

### Architecture Approach

The system has four layers: a command layer (clap-parsed subcommands), a core layer (`ForgeDetector`, `CommandRouter`, `UrlBuilder`), forge adapters (per-forge static command/flag maps behind a `ForgeAdapter` trait), and an infrastructure layer (`GitRemoteReader`, `SubprocessRunner`). The `runner/` module is intentionally isolated so it can be swapped for a test double and so Unix `exec()` semantics stay in one place. The `forge/` module is self-contained — adding a 5th forge means adding one adapter file and one `ForgeType` variant, nothing else.

**Major components:**
1. `ForgeDetector` — reads git remote URL, classifies forge type, extracts host/owner/repo into `ForgeContext`
2. `CommandRouter` + `FlagNormalizer` — maps canonical subcommand + flags to forge-specific CLI argv
3. `UrlBuilder` — constructs web URLs natively for `gf browse` (no delegation)
4. `SubprocessRunner` — execs the underlying CLI binary with full stdio inheritance; uses `exec()` on Unix
5. `ForgeAdapter` trait — per-forge static command/flag maps; `GitHub`, `GitLab`, `Gitea`, `Forgejo` variants

Build order is: `error.rs` → `runner/` → `forge/detector.rs` → `forge/adapters/` → `router/` → `forge/url_builder.rs` → `cli.rs` + `main.rs`. Each component is testable in isolation before the full pipeline exists.

See `.planning/research/ARCHITECTURE.md` for the full project structure, data flow diagrams, and Forgejo/Gitea detection strategy.

### Critical Pitfalls

1. **Exit code lost on signal termination** — `ExitStatus::code()` returns `None` when the child is killed by a signal. Use `ExitStatusExt::signal()` and re-raise the signal so the shell sees exit 130 on Ctrl+C, not exit 1. Address in: core subprocess delegation.

2. **SCP-style SSH remote URLs fail URL parsing** — `git@github.com:owner/repo.git` is not a valid URL; standard URL libraries reject it. Use a two-pass approach: URL library first, then regex fallback for SCP format. Address in: forge detection.

3. **clap intercepts flags intended for the child CLI** — Without `trailing_var_arg = true` and `allow_hyphen_values = true`, clap consumes flags that should pass through. Never define top-level `gf` flags with the same names as common child CLI flags. Address in: subcommand wiring.

4. **Self-hosted forge detection has no ground truth** — Virtually all Gitea/Forgejo users are self-hosted. Hostname allowlist alone covers only 20% of the user base. Build a detection priority: explicit `gf.forge` git config > `--forge` flag > known public hostnames > probe `gh`/`glab` auth status output > clear error with override instructions. Address in: forge detection.

5. **TTY detection broken when using `Command::output()`** — `output()` defaults to `Stdio::piped()`, which tells `gh`/`glab` they're not attached to a TTY, disabling color and breaking interactive prompts. Always use `Command::status()` for delegation. Address in: core subprocess delegation.

See `.planning/research/PITFALLS.md` for 10 pitfalls total, a "looks done but isn't" checklist, and security considerations.

## Implications for Roadmap

Based on research, the architecture has strict dependency ordering that drives phase structure. All commands require forge detection. Forge detection has multiple correctness traps. The subprocess delegation layer has signal/TTY requirements that cannot be retrofitted cheaply. Build foundations first.

### Phase 1: Foundation — Subprocess Delegation and Error Handling

**Rationale:** Every subsequent phase depends on a correct subprocess runner and error system. Pitfalls 1, 2, 6, and 10 (signal propagation, TTY passthrough, stdin forwarding) are all in this layer. Getting this wrong and discovering it later is expensive. This phase has no external dependencies.
**Delivers:** `SubprocessRunner` with `exec()` on Unix, `Stdio::inherit()`, correct signal re-raise, exit code propagation. `AppError` type. `which`-based CLI presence check with install hints.
**Addresses:** Missing CLI detection with install hint (table stakes).
**Avoids:** Signal death exit code lost (Pitfall 1), TTY detection broken (Pitfall 6), stdin not forwarded (Pitfall 10), Ctrl+C orphaning child (Pitfall 2).
**Research flag:** Standard patterns — skip research-phase. Rust subprocess and signal handling is well-documented.

### Phase 2: Forge Detection

**Rationale:** Forge detection is the prerequisite for all command dispatch. It has the most correctness traps (SSH URL parsing, `insteadOf` rewrites, worktree `.git` files, self-hosted instances). Isolating it as a phase with full test coverage before any commands are wired prevents silent failures from reaching users.
**Delivers:** `ForgeDetector` producing `ForgeContext`. `GitRemoteReader` using `git ls-remote --get-url` (applies `insteadOf`). Two-pass SSH URL parser. Self-hosted detection via `gh`/`glab auth status` probe. `gf.forge` git config override. `--remote` global flag.
**Addresses:** Forge auto-detection (table stakes), `--remote` global flag (differentiator).
**Avoids:** SCP URL parsing failure (Pitfall 3), `insteadOf` bypass (Pitfall 4), worktree `.git` file (Pitfall 5), self-hosted detection missing (Pitfall 9).
**Research flag:** May need research-phase for the `glab auth status` parsing approach and Forgejo-vs-Gitea probing strategy on ambiguous self-hosted hosts.

### Phase 3: Core PR Commands with Flag Normalization

**Rationale:** `gf pr create/list/view/merge` is the primary use case. Flag normalization (`--body` vs `--description`, `--base` vs `--target-branch`) is the key UX differentiator. Building all four forges' adapters in one phase ensures the `ForgeAdapter` trait is stable before it spreads to repo/auth commands.
**Delivers:** `ForgeAdapter` trait. `github.rs`, `gitlab.rs`, `gitea.rs`, `forgejo.rs` adapter modules. `CommandRouter` and `FlagNormalizer`. `gf pr create/list/view/merge` working across all four forges. clap subcommand wiring with `trailing_var_arg`.
**Addresses:** All must-have PR commands. Flag normalization differentiator.
**Avoids:** clap flag interception (Pitfall 7), `--help` ambiguity (Pitfall 8). Canonical flag set uses `gh` naming conventions (`--body`, `--base`, `--head`).
**Research flag:** Standard patterns for the adapter trait. May need targeted research-phase for `fj` (Forgejo CLI docs are sparse; MEDIUM confidence).

### Phase 4: Browse Native Implementation

**Rationale:** Browse is the only command that cannot be delegated — tea's implementation is broken, and glab has no equivalent. URL construction must cover GitHub, GitLab, Gitea, and Forgejo URL schemes, plus file path deep-links, detached HEAD handling, and self-hosted subpath ROOT_URLs. Isolating this phase ensures URL construction gets dedicated test coverage.
**Delivers:** `UrlBuilder` for all four forges. `gf browse` with optional file path argument. Current branch detection with detached HEAD fallback to commit hash. `webbrowser::open()` integration.
**Addresses:** `gf browse` native implementation (table stakes). File path deep-link (differentiator).
**Avoids:** Detached HEAD browse crash. Self-hosted subpath URL construction errors.
**Research flag:** Standard patterns for URL construction. No research-phase needed; URL patterns for all four forges are documented.

### Phase 5: Repo and Auth Commands

**Rationale:** Repo and auth commands are straightforward delegation — lower complexity than PR commands. Defer until the core PR loop is validated to avoid building more before the core use case is confirmed.
**Delivers:** `gf repo clone/view`. `gf auth login/logout/status` (fully delegated). `gf repo create/fork` (v1.x, delegated).
**Addresses:** All must-have repo and auth table stakes.
**Research flag:** Skip research-phase — pure delegation with no normalization required.

### Phase 6: Extended PR Commands and Polish

**Rationale:** Complete the PR workflow with checkout and review, add verbose/debug mode, and close any gaps discovered during validation of earlier phases.
**Delivers:** `gf pr checkout`, `gf pr review/approve`. Verbose mode showing expanded underlying command. Shell completion. Release polish.
**Addresses:** Should-have features.
**Research flag:** Skip research-phase — standard delegation patterns.

### Phase Ordering Rationale

- Phase 1 before Phase 2: The runner is needed to shell out for git commands used in detection. Error types are needed everywhere.
- Phase 2 before Phase 3: Every PR command dispatch starts with forge detection. Testing Phase 3 without Phase 2 complete means mocking a foundational layer.
- Phase 3 before Phase 4: The `ForgeAdapter` trait established in Phase 3 is reused by `UrlBuilder` for forge-specific URL schemes.
- Phase 4 before Phase 5: Browse is higher user value and more technically complex than repo/auth delegation.
- Phase 5 after Phase 3-4: Defer lower-complexity commands until the primary use case is validated.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Forge Detection):** Self-hosted detection via CLI auth status probe needs design validation. Forgejo/Gitea disambiguation for unknown hosts is ambiguous. Research should verify `gh auth status` and `glab auth status` output format stability.
- **Phase 3 (Core PR Commands):** `fj` (Forgejo CLI) documentation is sparse (MEDIUM confidence). Research should verify `fj pr create` flag names and behavior before building the Forgejo adapter.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Subprocess Delegation):** Rust subprocess, signal handling, and `exec()` patterns are thoroughly documented.
- **Phase 4 (Browse):** Forge URL schemes for repo/file/branch links are stable and documentable from first principles.
- **Phase 5 (Repo/Auth):** Pure delegation; no normalization required.
- **Phase 6 (Extended PR/Polish):** Standard delegation patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified on crates.io and docs.rs. clap `trailing_var_arg` pattern confirmed via official docs. Subprocess patterns are stdlib. |
| Features | HIGH | gh, glab, tea flag names verified against official docs and man pages. fj docs are sparse (MEDIUM for Forgejo-specific flags). |
| Architecture | HIGH | ForgeAdapter trait + layered structure is well-established for this class of tool. Build order verified against component dependency graph. |
| Pitfalls | HIGH | Signal propagation, TTY, and SCP URL pitfalls verified via Rust std docs, community, and real open issues in comparable projects. |

**Overall confidence:** HIGH

### Gaps to Address

- **`fj` (Forgejo CLI) flag coverage:** Documentation is sparse. During Phase 3 planning, verify `fj pr create` flags against the actual binary or Codeberg source. The MEDIUM confidence on `fj` flag names is the main research gap.
- **Self-hosted Forgejo/Gitea detection via auth probe:** The proposed approach of parsing `gh auth status` / `glab auth status` output to detect configured custom hostnames needs validation that output format is stable across CLI versions. Address in Phase 2 planning.
- **Forgejo ROOT_URL subpath in browse URLs:** Some self-hosted Forgejo/Gitea installs use a non-root `ROOT_URL` (e.g., `https://host/gitea/`). The browse URL construction must account for this. Low frequency but a known failure mode. Flag for Phase 4.

## Sources

### Primary (HIGH confidence)
- [crates.io/crates/clap](https://crates.io/crates/clap) — version 4.6, `trailing_var_arg`, derive API
- [docs.rs/anyhow](https://docs.rs/anyhow), [docs.rs/thiserror](https://docs.rs/thiserror) — versions and usage patterns
- [docs.rs/which](https://docs.rs/which) — version 8.0.2, PATH resolution
- [docs.rs/webbrowser](https://docs.rs/webbrowser) — version 1.2.0, platform support
- [doc.rust-lang.org — std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html) — Stdio::inherit(), exec()
- [doc.rust-lang.org — ExitStatusExt](https://doc.rust-lang.org/std/os/unix/process/trait.ExitStatusExt.html) — signal() for re-raise
- [GitHub CLI Manual](https://cli.github.com/manual/) — gh pr create flags, official
- [GitLab CLI docs](https://docs.gitlab.com/cli/) — glab mr create flags, official
- [tea CLI docs](https://gitea.com/gitea/tea) — tea pulls create flags, official

### Secondary (MEDIUM confidence)
- [forgejo-cli (fj) Codeberg](https://codeberg.org/forgejo-contrib/forgejo-cli) — fj flag names (docs sparse)
- [rust-url issue #220](https://github.com/servo/rust-url/issues/220) — SCP URL parsing limitation confirmed
- [clap issue #1538](https://github.com/clap-rs/clap/issues/1538) — trailing_var_arg behavior
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html) — community best practices

### Tertiary (LOW confidence)
- git2 vs subprocess trade-off for remote URL reading — pragmatic call, not a hard constraint; revisit if subprocess overhead becomes measurable

---
*Research completed: 2026-03-16*
*Ready for roadmap: yes*
