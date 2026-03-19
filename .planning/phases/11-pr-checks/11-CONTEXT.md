# Phase 11 — PR Checks — Context

**Milestone:** v1.2 Workflow Completeness
**Status:** complete

## Goal

`gf pr checks` shows CI/check status for a pull request, translating to the correct forge CLI command on GitHub, GitLab, and Forgejo, with UnsupportedFeature on Gitea.

## Why this Phase

CI status is the highest-risk command in v1.2 because the underlying CLIs diverge structurally: `gh pr checks <N>` is PR-scoped, `glab ci status` is branch-scoped, and `fj pr status <N>` is PR-scoped with different flags. Proving the adapter can handle this divergence de-risks the rest of the milestone.

## Scope

### In Scope

- `gf pr checks [<number>]` command with optional PR number
- GitHub: `gh pr checks [<number>]` with full flag passthrough
- GitLab: `glab ci status` (no-arg form only, current branch; number silently dropped)
- Forgejo: `fj pr status [<ID>]`
- Gitea: UnsupportedFeature error (tea has no CI status command)
- translation_test!, audit_test!, unsupported_test! entries for all 4 forges
- Clap subcommand definition with help text

### Out of Scope

- `--json` / `--jq` / `--template` output formatting flags (passthrough via `extra`)
- `--required` flag (gh-specific, passthrough via `extra`)
- Any direct API calls or output formatting by gf itself

## Key Decisions

- GitLab pr checks bypasses `pr_cmd` and hardcodes `"ci"` + `"status"` because glab CI status is branch-based, not PR-number-based
- Gitea returns UnsupportedFeature with three-field error (`feature`, `forge`, `forge_cli`) matching existing codebase convention
- Forges with structurally different command hierarchies can bypass `pr_cmd` by hardcoding the base command — pattern available for future phases
