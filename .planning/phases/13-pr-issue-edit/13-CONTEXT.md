# Phase 13 — PR and Issue Edit — Context

**Milestone:** v1.2 Workflow Completeness
**Status:** pending

## Goal

`gf pr edit` and `gf issue edit` add and remove labels, reviewers, and assignees on PRs and issues, with per-flag UnsupportedFeature errors when a forge CLI lacks the capability.

## Why this Phase

Edit commands are the last gap in the v1.2 command surface. They're medium-risk because the forge CLIs diverge structurally: gh uses `--add-*`/`--remove-*` flags, glab uses `--label`/`--unlabel`/prefix semantics, fj uses subcommand-based editing (`fj pr edit <N> labels --add`), and tea only supports issue editing.

## Scope

### In Scope

- `gf pr edit [<number>]` with `--add-label`, `--remove-label`, `--add-reviewer`, `--remove-reviewer`, `--add-assignee`, `--remove-assignee`
- `gf issue edit <number>` with `--add-label`, `--remove-label`, `--add-assignee`, `--remove-assignee`
- GitHub: `gh pr edit` / `gh issue edit` — direct flag mapping
- GitLab: `glab mr update` / `glab issue update` — remap to `--label`/`--unlabel`, `--reviewer`/prefix, `--assignee`/prefix
- Forgejo PR: `fj pr edit <N> labels --add/--rm` for labels; UnsupportedFeature for reviewer/assignee
- Forgejo Issue: UnsupportedFeature for labels and assignees
- Gitea PR: UnsupportedFeature for entire `gf pr edit` (tea has no pulls edit)
- Gitea Issue: `tea issues edit --add-labels/--remove-labels/--add-assignees` — map supported flags
- Per-flag UnsupportedFeature errors

### Out of Scope

- `--title`, `--body`, `--milestone`, `--project` editing
- `--draft` / `--ready` PR state changes
- Editing PR/issue descriptions or comments

## Key Unknowns

- **glab prefix semantics**: `glab mr update --reviewer +alice` adds, `--reviewer -alice` removes. Need to verify `+`/`-` prefix behavior during research.
- **fj subcommand routing**: `gf pr edit 42 --add-label bug` → `fj pr edit 42 labels --add bug`. Structural remap, not just flag renaming.

## Constraints

- **Per-flag error granularity**: adapter should error before exec if any flag in the invocation can't map — don't partially execute
- **glab uses verb `update`, not `edit`**: must remap `gf pr edit` → `glab mr update`
- **fj uses subcommand routing**: structural remap required

## Integration Points

### Consumes

- `adapter/pr.rs`, `adapter/issue.rs` — new `edit` arms
- `cmd/mod.rs` — new `edit` subcommands
- `GfError::UnsupportedFeature`

### Produces

- `translate_pr_edit()`, `translate_issue_edit()` functions
- `edit` clap subcommands with `--add-*`/`--remove-*` flags
- translation_test!, unsupported_test! entries for all forge × flag combinations
