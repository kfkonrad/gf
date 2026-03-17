# Phase 8: PR Workflow Commands - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can perform the complete PR/MR lifecycle from the `gf` command without knowing which forge they are on: list, merge, checkout, review, approve, view, and browse. Also includes `gf browse --issue` (ISSUE-06 pulled forward from Phase 9).

</domain>

<decisions>
## Implementation Decisions

### Unsupported command/flag behavior
- **Hard error** (exit non-zero + stderr message) for ANY unsupported command or flag translation
- Applies to both entire commands (`gf pr review` on tea) AND individual flags (`--author` on tea pr list)
- This is a NEW policy for Phase 8 — Phase 7's silent-omit convention is NOT followed
- Retroactive fix of Phase 7's silent omissions (e.g., `--draft` on tea) is deferred to a separate phase
- Error message format: clear, compile-error style — state what's unsupported and on which forge

### Merge strategy defaults
- When no `--squash`/`--rebase`/`--merge` flag is given, gf **explicitly passes** the merge flag to each forge
  - GitHub: (default behavior, but pass explicitly if gh supports it)
  - GitLab: (default behavior, pass explicitly)
  - Gitea: `--style merge`
  - Forgejo: `--method merge`
- Explicit > implicit — defense against forge default changes

### Delete-branch behavior
- `--delete-branch` and `--no-delete-branch` flags supported on `gf pr merge`
- Default behavior: **no delete** (conservative)
- Per-forge default configurable in `~/.config/gf/config.toml` — new config schema key
- Flag on command line overrides config file setting
- Translated to forge-specific equivalents per forge

### Browse --pr / --mr / --issue
- `gf browse --pr 123` opens the PR/MR URL in the browser
- `--mr` is an alias for `--pr` (consistent with `gf mr` subcommand alias)
- `gf browse --issue 42` opens the issue URL in the browser (ISSUE-06 pulled from Phase 9)
- **Flag conflicts**: `--pr`/`--mr`/`--issue` are mutually exclusive with `--branch` and file arguments — hard error if combined
- PR URL patterns per forge:
  - GitHub: `https://host/owner/repo/pull/123`
  - GitLab: `https://host/owner/repo/-/merge_requests/123`
  - Gitea: `https://host/owner/repo/pulls/123`
  - Forgejo: `https://host/owner/repo/pulls/123`
- Issue URL patterns per forge:
  - GitHub: `https://host/owner/repo/issues/42`
  - GitLab: `https://host/owner/repo/-/issues/42`
  - Gitea: `https://host/owner/repo/issues/42`
  - Forgejo: `https://host/owner/repo/issues/42`

### Claude's Discretion
- How to structure the new clap subcommands (list, merge, checkout, review, approve)
- Config file schema design for delete-branch defaults
- How to integrate the config reading into the merge translation path
- Error message wording and format for unsupported commands/flags
- Whether to add new GfError variants or reuse existing ones
- Plan count and task breakdown

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Adapter implementation (existing pattern)
- `src/adapter/mod.rs` — Translation dispatch router; needs new arms for list/merge/checkout/review/approve
- `src/adapter/pr.rs` — Existing PR create/view translations; extend with new subcommand translators
- `src/adapter/repo_auth.rs` — Example of subcommand remapping pattern (tea auth login → logins add)

### CLI definitions
- `src/cmd/mod.rs` — Clap command tree; `build_pr()` needs new subcommands (list, merge, checkout, review, approve)
- `src/forge/mod.rs` — ForgeType enum with `cli_name()` method

### Browse implementation
- `src/browse/mod.rs` — Current browse; needs `--pr`/`--mr`/`--issue` flags and URL builders

### Config file
- `src/forge/mod.rs` — Config file loading for self-hosted domains; extend schema for merge defaults

### Pre-mapped flag translations (Phase 7)
- `tests/flag_audit.rs` lines 380-550+ — `v11_translation_test!` macro with `#[ignore]`d tests for PR list/merge/checkout; remove `#[ignore]` as adapters are implemented

### Requirements
- `.planning/REQUIREMENTS.md` — PR-01 through PR-07 (PR workflows), ISSUE-06 (browse --issue)
- `.planning/ROADMAP.md` — Phase 8 success criteria (5 criteria)

### Prior context decisions
- `.planning/phases/07-flag-normalization-audit/07-CONTEXT.md` — Flag translation patterns, macro testing strategy
- `.planning/STATE.md` — Accumulated context including Phase 8 risks (glab mr approve subcommand routing)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `adapter::pr::translate_pr()` — Existing subcommand router for PR; extend with new match arms
- `adapter::pr::pr_subcommand_name()` — Maps canonical "pr" to forge-specific names (pr/mr/pulls)
- `browse::build_file_url()` / `browse::build_repo_url()` — URL construction pattern to follow for PR/issue URLs
- `forge::config_lookup()` — Config file loading; extend for merge defaults
- `runner::run()` — exec() process replacement; unchanged

### Established Patterns
- Translation functions: `(ForgeType, &ArgMatches) → Vec<String>`
- Passthrough: unrecognized flags after `--` appended verbatim
- Per-forge `match` arms for flag/subcommand differences
- `translation_test!` macro for testing translations through full dispatch path
- `v11_translation_test!` macro with pre-mapped expected values

### Integration Points
- `adapter/mod.rs::translate()` — Add routing for new PR subcommands
- `cmd/mod.rs::build_pr()` — Add list, merge, checkout, review, approve subcommands with canonical flags
- `browse/mod.rs::run()` — Handle `--pr`/`--mr`/`--issue` flags before file/repo URL path
- `forge/mod.rs` — Config schema extension for `[merge]` or per-forge merge settings
- `tests/flag_audit.rs` — Remove `#[ignore]` from v1.1 tests as adapters land; add new tests for review/approve

### Known Risks
- `glab mr approve` is a subcommand, not a flag — `translate_pr_review` needs to handle subcommand routing differently for GitLab
- `tea` has no review/approve — hard UNSUPPORTED; must produce error per new policy
- `glab` state uses boolean flags (`--closed`/`--merged`/`--all`) not `--state value` pattern
- `tea pr view` uses `pulls <N>` directly (no "view" verb) — existing pattern in `translate_pr_view()`

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

- **Retroactive silent-omit → hard-error migration**: Phase 7's existing silent flag omissions (e.g., `--draft` on tea for `pr create`) should be changed to hard errors for consistency with Phase 8's policy. Captured as separate phase/todo.
- **Per-forge config defaults for other settings**: The config schema extension for `--delete-branch` could expand to other per-forge defaults in the future.

</deferred>

---

*Phase: 08-pr-workflow-commands*
*Context gathered: 2026-03-17*
