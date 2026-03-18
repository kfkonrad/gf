---
id: S01
milestone: M002
status: ready
---

# S01: PR Checks — Context

## Goal

`gf pr checks` shows CI/check status for a pull request, translating to the correct forge CLI command on GitHub, GitLab, and Forgejo, with UnsupportedFeature on Gitea.

## Why this Slice

CI status is the highest-risk command in M002 because the underlying CLIs diverge structurally: `gh pr checks <N>` is PR-scoped, `glab ci status` is branch-scoped, and `fj pr status <N>` is PR-scoped with different flags. Proving the adapter can handle this divergence de-risks the rest of the milestone.

## Scope

### In Scope

- `gf pr checks [<number>]` command with optional PR number
- `--watch` flag translated to `gh --watch`, `glab --live`, `fj --wait`
- GitHub: `gh pr checks [<number>]` with full flag passthrough
- GitLab: `glab ci status` (no-arg form only, current branch); error if MR number provided
- Forgejo: `fj pr status [<ID>]` with `--wait` mapping
- Gitea: UnsupportedFeature error (tea has no CI status command)
- translation_test!, audit_test!, unsupported_test! entries for all 4 forges
- Clap subcommand definition with help text

### Out of Scope

- `--json` / `--jq` / `--template` output formatting flags (passthrough via `extra` is fine)
- `--required` flag (gh-specific, passthrough via `extra`)
- `glab ci status --compact` / `--live` as standalone gf flags (only `--watch` is canonical)
- Any direct API calls or output formatting by gf itself
- `gf ci` top-level command (checks live under `gf pr`)

## Constraints

- **PR-scoped interface**: `gf pr checks` is semantically about a PR's CI status, not a branch's pipeline. The adapter translates to branch-scoped commands where needed (GitLab), but the user-facing contract is PR-scoped.
- **GitLab number restriction**: `gf pr checks 42` on GitLab returns an error because glab ci status cannot look up a pipeline by MR number. Only `gf pr checks` (no argument, current branch) is supported on GitLab.
- **Standard UnsupportedFeature for Gitea**: Same error pattern as other unsupported commands — no fallback suggestions, no special handling.
- **Must follow existing `detect → translate → exec` pipeline** — no new patterns in main.rs or adapter/mod.rs dispatch.

## Integration Points

### Consumes

- `adapter/pr.rs` — existing `translate_pr()` dispatch; new `checks` arm added
- `cmd/mod.rs` — existing `build_pr()` function; new `checks` subcommand added
- `ForgeType` enum — unchanged, used for match arms in translation
- `GfError::UnsupportedFeature` — existing error variant, reused for Gitea and GitLab-with-number

### Produces

- `translate_pr_checks()` function in `adapter/pr.rs`
- `checks` clap subcommand under `pr` in `cmd/mod.rs`
- translation_test! entries proving correct args for GitHub, GitLab (no-arg), Forgejo
- unsupported_test! entries for Gitea and GitLab-with-number
- audit_test! entry for `gf pr checks --help`

## Open Questions

- None — all behavioral decisions resolved during discuss phase.
