# Phase 9: Issues, Clone, and Self-Hosted Detection - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can manage issues (list, view, create, close, reopen), clone repos via forge CLIs, and have unknown self-hosted domains detected automatically via CLI auth probing. Browse --issue is already implemented (Phase 8).

</domain>

<decisions>
## Implementation Decisions

### CLI auth probing strategy (CORE-04)
- Sequential probe order: gh → glab → tea → fj — stop on first match
- A "match" means the probed hostname appears in the CLI's auth status stdout output
- 5-second timeout per CLI probe (20s worst case for all four)
- Probe triggers only after config_lookup() AND match_known_host() both fail (existing fallback chain)
- Cache probe results indefinitely in `~/.cache/gf/` — no TTL expiry
- Config.toml ALWAYS takes precedence over cached probe results (user config > auto-detected)
- If no CLI matches, fall through to existing ForgeNotDetected error with config.toml hint

### Issue close/reopen semantics
- Modeled as subcommands: `gf issue close 42` / `gf issue reopen 42`
- Idempotency behavior delegated to forge CLI — gf does not check current state before sending
- No `--comment` flag on close — close is just close; commenting is a separate action
- `fj issue reopen` → hard UnsupportedFeature error (Forgejo CLI has no reopen command)
- Hard-error policy from Phase 8 applies to all unsupported issue command/flag combinations

### Clone URL resolution
- `gf repo clone owner/repo` requires a `[defaults]` config section with `clone_host` FQDN:
  ```toml
  [defaults]
  clone_host = "gitlab.mycompany.com"
  ```
- Without config: hard error with message showing the config.toml snippet to add
- `gf repo clone https://host/owner/repo` detects forge from URL host, delegates to forge CLI
- Full URL with unknown host → hard error: "Forge not detected for host X. Add it to config.toml or use `git clone` directly."
- Tea (Gitea CLI) has no `repo clone` → UnsupportedFeature error
- Clone always delegates to forge CLI (not `git clone`) for forge-specific setup benefits (PR refs, aliases)

### Claude's Discretion
- Issue command clap subcommand structure and flag definitions
- CORE-04 probe implementation (process spawning, stdout capture, parsing)
- Cache file format in `~/.cache/gf/` (TOML, JSON, etc.)
- Clone argument parsing (detecting full URL vs owner/repo shorthand)
- Config.toml `[defaults]` section deserialization structure
- Plan count and task breakdown

</decisions>

<specifics>
## Specific Ideas

- The `[defaults]` section uses FQDN (not forge type) to handle multiple self-hosted instances of the same forge type unambiguously
- Probe order matches market share priority — most common forges tried first for faster detection

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Adapter implementation (existing pattern)
- `src/adapter/mod.rs` — Translation dispatch router; needs new `issue` arm and `repo clone` arm
- `src/adapter/pr.rs` — PR command translations; template for issue command implementation
- `src/adapter/repo_auth.rs` — Repo and Auth translations; extend with clone subcommand

### CLI definitions
- `src/cmd/mod.rs` — Clap command tree; needs `build_issue()` function with list/view/create/close/reopen subcommands, and clone subcommand on repo
- `src/forge/mod.rs` — ForgeType enum, `detect()` function, config_lookup(), match_known_host(); integration point for CORE-04 probing

### Browse (already done)
- `src/browse/mod.rs` — `build_issue_url()` already implemented in Phase 8 (ISSUE-06 complete)

### Pre-mapped flag translations (Phase 7)
- `tests/flag_audit.rs` lines 724-837 — `v11_translation_test!` entries for issue list/view/create and repo clone; remove `#[ignore]` as adapters land

### Error handling
- `src/error.rs` — `GfError::UnsupportedFeature` variant for hard errors on unsupported commands/flags

### Config file
- `src/forge/mod.rs` — Config file loading at `~/.config/gf/config.toml`; extend with `[defaults]` section

### Requirements
- `.planning/REQUIREMENTS.md` — ISSUE-01 through ISSUE-06, REPO-01, CORE-04
- `.planning/ROADMAP.md` — Phase 9 success criteria (5 criteria)

### Prior context decisions
- `.planning/phases/08-pr-workflow-commands/08-CONTEXT.md` — Hard-error policy for unsupported features, browse --issue implementation
- `.planning/phases/07-flag-normalization-audit/07-CONTEXT.md` — Flag translation macro patterns, test strategy

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `adapter::pr::translate_pr()` — Subcommand router pattern; copy for `translate_issue()`
- `adapter::pr::pr_subcommand_name()` — Per-forge name mapping; create `issue_subcommand_name()` (github=issue, gitlab=issue, gitea=issues, forgejo=issue)
- `browse::build_issue_url()` — Already implemented; ISSUE-06 is done
- `forge::config_lookup()` — Config loading; extend for `[defaults]` section
- `forge::detect()` — Detection chain; add probe step between config_lookup and known_host error
- `translation_test!` / `v11_translation_test!` / `unsupported_test!` macros in `tests/flag_audit.rs`

### Established Patterns
- Translation functions: `(ForgeType, &ArgMatches) → Result<Vec<String>, GfError>`
- Passthrough: unrecognized flags after `--` appended verbatim
- Per-forge `match` arms for flag/subcommand differences
- Hard UnsupportedFeature error for impossible forge×command combinations
- Forgejo uses `issue search` (not `issue list`) for listing issues

### Integration Points
- `adapter/mod.rs::translate()` — Add `Some(("issue", sub)) => issue::translate_issue(forge, sub)`
- `adapter/repo_auth.rs::translate_repo()` — Add `Some(("clone", sub))` match arm
- `cmd/mod.rs` — Add `build_issue()` with 5 subcommands; add `clone` to `build_repo()`
- `forge/mod.rs::detect()` — Insert probe step: `config_lookup → known_host → probe_auth → error`
- `~/.cache/gf/` — New cache directory for probe results

</code_context>

<deferred>
## Deferred Ideas

- **Issue commenting** (ISSUE-07) — v2 requirement, not in Phase 9 scope
- **Issue label assignment** (ISSUE-08) — v2 requirement, not in Phase 9 scope
- **Retroactive silent-omit → hard-error migration** — Deferred from Phase 8 context

</deferred>

---

*Phase: 09-issues-clone-and-self-hosted-detection*
*Context gathered: 2026-03-18*
