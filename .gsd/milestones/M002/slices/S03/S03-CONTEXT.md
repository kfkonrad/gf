---
id: S03
milestone: M002
status: ready
---

# S03: PR and Issue Edit — Context

## Goal

`gf pr edit` and `gf issue edit` add and remove labels, reviewers, and assignees on PRs and issues, with per-flag UnsupportedFeature errors when a forge CLI lacks the capability.

## Why this Slice

Edit commands are the last gap in the M002 command surface. They're medium-risk because the forge CLIs diverge structurally: gh uses `--add-*`/`--remove-*` flags, glab uses `--label`/`--unlabel`/prefix semantics, fj uses subcommand-based editing (`pr edit <N> labels --add`), and tea only supports issue editing. Proving the adapter handles all four patterns completes the milestone's risk retirement.

## Scope

### In Scope

- `gf pr edit [<number>]` with `--add-label`, `--remove-label`, `--add-reviewer`, `--remove-reviewer`, `--add-assignee`, `--remove-assignee`
- `gf issue edit <number>` with `--add-label`, `--remove-label`, `--add-assignee`, `--remove-assignee`
- GitHub: `gh pr edit` / `gh issue edit` — direct flag mapping (`--add-label` → `--add-label`, etc.)
- GitLab: `glab mr update` / `glab issue update` — remap to `--label`/`--unlabel`, `--reviewer`/prefix, `--assignee`/prefix
- Forgejo PR: `fj pr edit <N> labels --add/--rm` for labels; UnsupportedFeature for `--add-reviewer`, `--remove-reviewer`, `--add-assignee`, `--remove-assignee`
- Forgejo Issue: UnsupportedFeature for `--add-label`/`--remove-label` (fj issue edit has no labels subcommand); UnsupportedFeature for assignees
- Gitea PR: UnsupportedFeature for entire `gf pr edit` command (tea has no pulls edit)
- Gitea Issue: `tea issues edit --add-labels/--remove-labels/--add-assignees` — map supported flags; UnsupportedFeature for `--remove-assignee` if tea doesn't support it
- Per-flag UnsupportedFeature errors — if a command is partially supported on a forge, the supported flags work and only the unsupported flags error
- translation_test!, audit_test!, unsupported_test! entries for all forge × flag combinations

### Out of Scope

- `--title`, `--body`, `--milestone`, `--project` editing — not in scope for M002
- `--draft` / `--ready` PR state changes (glab mr update supports these, but they're not edit-metadata)
- Editing PR/issue descriptions or comments via `gf pr edit` / `gf issue edit`

## Constraints

- **Per-flag error granularity**: when a user runs `gf pr edit --add-label bug --add-reviewer alice` on Forgejo, the label addition succeeds conceptually in translation, but reviewer fails with UnsupportedFeature. The adapter should error before exec if any flag in the invocation can't map — don't partially execute.
- **Canonical flags are `--add-*` / `--remove-*`**: the gf interface uses gh-style additive/subtractive flags. These get translated to whatever the underlying CLI expects (glab prefix semantics, fj subcommands, tea flags).
- **glab uses verb `update`, not `edit`**: the adapter must remap `gf pr edit` → `glab mr update` and `gf issue edit` → `glab issue update`.
- **fj uses subcommand routing**: `gf pr edit 42 --add-label bug` → `fj pr edit 42 labels --add bug`. This is a structural remap, not just flag renaming.
- **Must follow existing `detect → translate → exec` pipeline** — no new patterns.

## Integration Points

### Consumes

- `adapter/pr.rs` — existing `translate_pr()` dispatch; new `edit` arm added
- `adapter/issue.rs` — existing `translate_issue()` dispatch; new `edit` arm added
- `cmd/mod.rs` — existing `build_pr()` and `build_issue()` functions; new `edit` subcommands added
- `GfError::UnsupportedFeature` — reused for per-flag and whole-command unsupported cases

### Produces

- `translate_pr_edit()` function in `adapter/pr.rs`
- `translate_issue_edit()` function in `adapter/issue.rs`
- `edit` clap subcommands under both `pr` and `issue` in `cmd/mod.rs`
- translation_test! entries for all supported forge × flag combinations
- unsupported_test! entries for Gitea PR edit, Forgejo reviewer/assignee, Forgejo issue labels

## Open Questions

- **glab prefix semantics for reviewers/assignees**: glab mr update `--reviewer alice` replaces all reviewers, `--reviewer +alice` adds, `--reviewer -alice` removes. The adapter needs to map `--add-reviewer alice` → `--reviewer +alice` and `--remove-reviewer alice` → `--reviewer -alice`. Need to verify this works correctly during research/planning — the `+`/`-` prefix behavior is documented but not tested yet.
