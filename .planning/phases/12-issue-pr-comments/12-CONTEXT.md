# Phase 12 — Issue and PR Comments — Context

**Milestone:** v1.2 Workflow Completeness
**Status:** pending (next up)

## Goal

`gf issue comment <number> --body "text"` and `gf pr comment <number> --body "text"` post comments on issues and PRs across all supported forges.

## Why this Phase

Comments are the most common secondary workflow action after viewing and creating. Adding standalone comment verbs makes the gf command surface match how users actually think ("comment on this issue") rather than routing through the review subcommand. Independent of Phase 11 and Phase 13, so it can execute in parallel.

## Scope

### In Scope

- `gf issue comment <number> --body "text"` — new standalone verb under `issue`
- `gf pr comment <number> --body "text"` — new standalone verb under `pr`
- GitHub: `gh issue comment <N> --body "text"` / `gh pr comment <N> --body "text"`
- GitLab: `glab issue note <N> --message "text"` / `glab mr note <N> --message "text"` (verb remap + flag remap)
- Forgejo: `fj issue comment <N> "text"` / `fj pr comment <N> "text"` (body is positional, not a flag)
- Gitea: UnsupportedFeature error for both commands (tea has no comment command)
- translation_test!, audit_test!, unsupported_test! entries for all 4 forges × both commands
- Clap subcommand definitions with help text

### Out of Scope

- `--body-file` flag — users can use shell substitution or passthrough via `extra`
- `--editor` / `--web` flags (gh-specific, available via passthrough)
- Modifying the existing `gf pr review --comment` path — it stays as-is
- Commenting on specific PR review threads or diff lines

## Constraints

- **`--body` is the only canonical flag**: translated to `--message` on glab, positional arg on fj
- **Number is required**: comments always require an explicit issue/PR number
- **Standard UnsupportedFeature for Gitea**: tea has no comment capability

## Integration Points

### Consumes

- `adapter/issue.rs` — existing `translate_issue()` dispatch; new `comment` arm
- `adapter/pr.rs` — existing `translate_pr()` dispatch; new `comment` arm
- `cmd/mod.rs` — existing `build_issue()` and `build_pr()`; new `comment` subcommands
- `GfError::UnsupportedFeature` — reused for Gitea

### Produces

- `translate_issue_comment()` function in `adapter/issue.rs`
- `translate_pr_comment()` function in `adapter/pr.rs`
- `comment` clap subcommands under both `issue` and `pr` in `cmd/mod.rs`
- translation_test!, unsupported_test!, audit_test! entries
