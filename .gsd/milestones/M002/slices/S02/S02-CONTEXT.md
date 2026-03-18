---
id: S02
milestone: M002
status: ready
---

# S02: Issue and PR Comments — Context

## Goal

`gf issue comment <number> --body "text"` and `gf pr comment <number> --body "text"` post comments on issues and PRs across all supported forges.

## Why this Slice

Comments are the most common secondary workflow action after viewing and creating. Adding standalone comment verbs makes the gf command surface match how users actually think ("comment on this issue") rather than routing through the review subcommand. Independent of S01 and S03, so it can execute in parallel.

## Scope

### In Scope

- `gf issue comment <number> --body "text"` — new standalone verb under `issue`
- `gf pr comment <number> --body "text"` — new standalone verb under `pr`
- GitHub: `gh issue comment <N> --body "text"` / `gh pr comment <N> --body "text"`
- GitLab: `glab issue note <N> --message "text"` / `glab mr note <N> --message "text"` (verb remap + flag remap)
- Forgejo: `fj issue comment <N> "text"` / `fj pr comment <N> "text"` (body is positional, not a flag)
- Gitea: UnsupportedFeature error for both `gf issue comment` and `gf pr comment` (tea has no comment command)
- translation_test!, audit_test!, unsupported_test! entries for all 4 forges × both commands
- Clap subcommand definitions with help text

### Out of Scope

- `--body-file` flag — users can use shell substitution (`--body "$(cat file.md)"`) or passthrough via `extra`
- `--editor` / `--web` flags (gh-specific, available via passthrough)
- `--edit-last` / `--delete-last` (gh-specific comment management, available via passthrough)
- Modifying the existing `gf pr review --comment` path — it stays as-is; two ways to comment on a PR is fine
- Commenting on specific PR review threads or diff lines

## Constraints

- **`--body` is the only canonical flag**: text content passed via `--body "text"`. Translated to `--message` on glab, positional arg on fj.
- **Number is required**: unlike `gf pr view` which can infer current-branch PR, comments always require an explicit issue/PR number.
- **Existing `gf pr review --comment` is untouched**: the new `gf pr comment` is a separate verb. Both produce correct output for their respective forge CLI paths.
- **Standard UnsupportedFeature for Gitea**: tea has no comment capability for issues or PRs.
- **Must follow existing `detect → translate → exec` pipeline** — no new patterns.

## Integration Points

### Consumes

- `adapter/issue.rs` — existing `translate_issue()` dispatch; new `comment` arm added
- `adapter/pr.rs` — existing `translate_pr()` dispatch; new `comment` arm added
- `cmd/mod.rs` — existing `build_issue()` and `build_pr()` functions; new `comment` subcommands added
- `GfError::UnsupportedFeature` — reused for Gitea

### Produces

- `translate_issue_comment()` function in `adapter/issue.rs`
- `translate_pr_comment()` function in `adapter/pr.rs`
- `comment` clap subcommands under both `issue` and `pr` in `cmd/mod.rs`
- translation_test! entries for GitHub, GitLab, Forgejo × both commands
- unsupported_test! entries for Gitea × both commands
- audit_test! entries for help text

## Open Questions

- None — all behavioral decisions resolved during discuss phase.
