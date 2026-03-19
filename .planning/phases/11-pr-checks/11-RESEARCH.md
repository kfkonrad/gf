# Phase 11 — PR Checks — Research

**Date:** 2026-03-19
**Depth:** Targeted (known patterns, structural divergence across forges is the main complexity)

## Summary

`gf pr checks` needs to show CI/check status for a pull request. The four forge CLIs diverge structurally more than any previous command — this isn't flag remapping, it's entirely different command trees:

- **GitHub:** `gh pr checks [<number>] [--watch] [--required] [--json ...]` — a subcommand under `pr`
- **GitLab:** `glab ci status [--branch <branch>] [--compact] [--live]` — a subcommand under `ci`, not `mr`; identifies by branch, not PR number
- **Forgejo:** `fj pr status [<id>] [--wait]` — a subcommand under `pr`, but named `status` not `checks`
- **Gitea (tea):** No equivalent command at all — `tea pulls` has no CI/checks/status subcommand

The adapter pattern used for all other commands (remap flags within the same verb structure) doesn't fully apply here. GitLab routes to an entirely different top-level command (`ci status` vs `mr`). However, the existing adapter architecture already handles subcommand remapping, and the `Vec<String>` return type accommodates this without architectural changes.

## Recommendation

Add a `checks` subcommand to the `pr` clap definition and a `translate_pr_checks()` function in `adapter/pr.rs`:

1. **Clap:** Add `checks` subcommand to `build_pr()` following `view` shape (optional `number` + `extra`).
2. **Adapter:** Add `Some(("checks", sub)) => translate_pr_checks(forge, pr_cmd, sub)` match arm.
3. **Translation function:** Per-forge match:
   - GitHub: `["pr", "checks", "<number>"]`
   - GitLab: `["ci", "status"]` (branch-based, number silently dropped)
   - Forgejo: `["pr", "status", "<number>"]`
   - Gitea: `Err(GfError::UnsupportedFeature { ... })`
4. **Tests:** `translation_test!` for all 4 forges, `unsupported_test!` for Gitea, `audit_test!` for gh/glab/fj.

## Key Files

- `src/cmd/mod.rs` — Add `checks` subcommand to `build_pr()`. ~15 lines.
- `src/adapter/pr.rs` — Add `translate_pr_checks()` function. ~40 lines.
- `tests/flag_audit.rs` — Add translation/audit/unsupported tests. ~30 lines.

## Constraints

- GitLab `ci status` does not accept an MR number — it works on the current branch. The adapter must drop the `<number>` argument for GitLab.
- The `pr_cmd` helper returns `"mr"` for GitLab, but `glab ci status` doesn't use `mr` at all — translated args must start with `"ci"`, bypassing `pr_cmd`.
- `tea` (Gitea) has no CI/checks command whatsoever — must return `UnsupportedFeature`.

## Common Pitfalls

- **Using `pr_cmd` for GitLab checks** — `translate_pr_checks()` must NOT use `pr_cmd` for GitLab. Output must be `["ci", "status"]`, not `["mr", "status"]`.
- **Assuming glab ci status takes a number** — it doesn't. It takes `--branch`. Number is silently dropped.
