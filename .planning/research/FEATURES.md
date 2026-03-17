# Feature Research

**Domain:** Unified git forge CLI wrapper (PR workflows, issues, clone, browse)
**Researched:** 2026-03-17
**Confidence:** HIGH (verified against live gh, glab, tea, fj CLIs)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `gf pr list` | Core PR workflow — first thing engineers do after `pr create` | LOW | All 4 CLIs support; flag shapes differ (see dependency notes) |
| `gf pr merge` | Completing PRs is the whole point of the workflow | LOW | All 4 CLIs support; strategy flags differ significantly |
| `gf pr checkout` | Review-driven development requires checking out peers' PRs | LOW | All 4 CLIs support; branch naming flag differs |
| `gf pr review` | Approve/request-changes is a universal forge operation | MEDIUM | Asymmetric: gh has unified `review` command; glab splits into `approve` and `mr note`; tea/fj lack a full review command |
| `gf repo clone` | Cloning is how every project starts | LOW | Asymmetric: gh/glab/fj use `repo clone`; tea uses top-level `clone` command |
| `gf issue list` | Issues are the other half of forge workflows | LOW | All 4 CLIs support; flag shapes differ |
| `gf issue view` | Reading issue details is table stakes alongside listing | LOW | All 4 CLIs support |
| `gf issue create` | Creating issues is as common as creating PRs | LOW | All 4 CLIs support |
| Line-range browse (`gf browse src/main.rs:42-55`) | Deep-linking to code is a daily workflow for code review | MEDIUM | Browse is already native; need to parse `:start-end` suffix and append forge-specific anchor |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Self-hosted forge detection via CLI auth probing (CORE-04) | Removes config.toml manual step for self-hosted users; zero-setup experience | HIGH | Probe `gh auth status`, `glab auth status`, `tea logins list` output for non-public hosts; fragile by design — treat as fallback only |
| Flag normalization audit across all forge CLIs | Prevents silent wrong-flag passthrough bugs; builds user trust | MEDIUM | Systematically verify `--state`, `--assignee`, `--label`, `--limit`, `--squash`, `--rebase`, `--delete-branch` across all 4 CLIs for all new commands |
| `gf pr list --state merged` canonical form | gh uses `--state merged`; glab uses `--merged` (separate flag); normalizing this avoids user confusion | LOW | Part of flag audit; worth explicit attention |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Own auth/token management | Single config for all forges sounds convenient | Auth is already solved by each CLI; duplicating it creates two sources of truth, security surface area, and maintenance burden | Delegate fully; surface clear "run `gh auth login`" error messages |
| Interactive TUI for PR review | Rich review experience | Requires terminal UI library, complex state management, diverges from "thin router" model | Let each forge CLI's own interactive mode handle it via passthrough |
| `gf pr review` with comment threading | Feature parity with forge web UIs | tea has no equivalent command at all; fj has no `review` command either; building it natively breaks the delegation model | Scope `gf pr review` to approve/request-changes only; passthrough for CLI-specific extras |

## Feature Dependencies

```
gf pr list
    requires: forge detection (DONE v1.0)
    requires: pr_subcommand_name() translation (DONE v1.0)
    NEW: --state flag normalization (gh=--state enum; glab=--closed/--merged/--all flags; tea=--state enum)

gf pr merge
    requires: forge detection (DONE v1.0)
    NEW: merge strategy flag normalization (--squash, --rebase differ per forge)
    NEW: --delete-branch normalization (gh=--delete-branch; glab=--remove-source-branch; fj=--delete)

gf pr checkout
    requires: forge detection (DONE v1.0)
    NEW: branch flag normalization (gh/glab=--branch; fj=--branch-name)

gf pr review
    requires: forge detection (DONE v1.0)
    NEW: approve/review command mapping (gh=review; glab=approve; tea=N/A; fj=N/A)
    NOTE: passthrough with warning on tea/fj (no native equivalent)

gf repo clone
    requires: forge detection (DONE v1.0)
    NEW: clone subcommand routing (gh/glab/fj=repo clone; tea=top-level clone)

gf issue list / view / create
    requires: forge detection (DONE v1.0)
    NEW: issue flag normalization (--state vs --closed/--all differ)
    NOTE: fj uses search-based listing (fj issue search), not a list subcommand

gf browse <file>:L42-55 (line range)
    requires: existing browse URL construction (DONE v1.0)
    NEW: :line-range suffix parser
    NEW: forge-specific anchor format (#L42-L55 vs #L42-55 differ)

Self-hosted detection via CLI probing (CORE-04)
    requires: forge detection fallback chain (DONE v1.0)
    enhances: all commands (removes manual config.toml step)
```

### Dependency Notes

- **gf pr merge strategy flag normalization:** gh uses boolean flags `--merge`/`--squash`/`--rebase`; glab uses `--squash`/`--rebase` (merge is default, no flag); tea uses `--style merge|squash|rebase`; fj uses `--method merge|squash|rebase`. Canonical canonical flags: `--squash` and `--rebase` as booleans, translate to style/method for tea/fj.
- **gf pr merge delete branch:** gh=`--delete-branch`; glab=`--remove-source-branch`; fj=`--delete`; tea has no equivalent. Canonical: `--delete-branch`.
- **gf pr checkout branch flag:** gh/glab use `--branch`; fj uses `--branch-name`. Normalize to `--branch`.
- **gf repo clone is structurally asymmetric:** tea clone is a top-level command (`tea clone <slug>`), not under `tea repos`. The adapter must route `gf repo clone` to `tea clone` (not `tea repo clone`) for Gitea.
- **gf pr review is partially infeasible for tea/fj:** tea has no approve command; fj has no review command. These forges must receive passthrough treatment; `gf pr review --approve` should warn on unsupported forges rather than silently fail.
- **Line-range anchor format per forge:**
  - GitHub: `#L42-L55` (each line number prefixed with L)
  - GitLab: `#L42-55` (first line prefixed with L, second bare)
  - Gitea/Forgejo: `#L42-L55` (same as GitHub, consistent with GitHub-derived codebase)

## MVP Definition

### Launch With (v1.1 — current milestone)

- [ ] `gf pr list` — basic listing with `--state` normalization — core workflow completion
- [ ] `gf pr merge` — merge with `--squash`/`--rebase`/`--delete-branch` normalized — closes PR workflow
- [ ] `gf pr checkout` — checkout with `--branch` normalized — review workflow
- [ ] `gf pr review` — approve/request-changes, passthrough with warning on tea/fj — code review
- [ ] `gf repo clone` — clone with tea top-level routing special-case — onboarding workflows
- [ ] `gf issue list / view / create` — issue triage workflows
- [ ] `gf browse src/file.rs:42-55` — line range deep-linking — code review sharing
- [ ] Flag normalization audit — correctness baseline for all new and existing commands

### Add After Validation (v1.x)

- [ ] Self-hosted forge detection via CLI auth probing (CORE-04) — add when config.toml friction becomes a reported blocker
- [ ] `gf issue close` / `gf issue comment` — add when issue management workflow completeness is requested

### Future Consideration (v2+)

- [ ] `gf pr status` / CI status — requires forge API calls, breaks no-API constraint
- [ ] Batch operations (merge multiple PRs) — niche, high complexity

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `gf pr list` | HIGH | LOW | P1 |
| `gf pr merge` | HIGH | LOW | P1 |
| `gf pr checkout` | HIGH | LOW | P1 |
| `gf issue list/view/create` | HIGH | LOW | P1 |
| `gf repo clone` | HIGH | LOW (one special-case for tea) | P1 |
| Line-range browse | MEDIUM | MEDIUM (anchor format per forge) | P1 |
| `gf pr review` | MEDIUM | MEDIUM (infeasible on tea/fj) | P1 |
| Flag normalization audit | HIGH | MEDIUM (systematic survey) | P1 |
| Self-hosted CLI auth probing | MEDIUM | HIGH (fragile output parsing) | P2 |

**Priority key:**
- P1: Must have for v1.1 launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | gh | glab | tea | fj | gf canonical |
|---------|-----|------|-----|-----|--------------|
| PR list | `gh pr list` | `glab mr list` | `tea pulls list` | `fj pr search` | `gf pr list` |
| PR merge | `gh pr merge` | `glab mr merge` | `tea pulls merge` | `fj pr merge` | `gf pr merge` |
| PR checkout | `gh pr checkout` | `glab mr checkout` | `tea pulls checkout` | `fj pr checkout` | `gf pr checkout` |
| PR review/approve | `gh pr review --approve` | `glab mr approve` | no equivalent | no equivalent | `gf pr review --approve` (passthrough warning on tea/fj) |
| Repo clone | `gh repo clone` | `glab repo clone` | `tea clone` (top-level) | `fj repo clone` | `gf repo clone` |
| Issue list | `gh issue list` | `glab issue list` | `tea issues list` | `fj issue search` | `gf issue list` |
| Issue view | `gh issue view` | `glab issue view` | `tea issues view` | `fj issue view` | `gf issue view` |
| Issue create | `gh issue create` | `glab issue create` | `tea issues create` | `fj issue create` | `gf issue create` |
| Merge strategy | `--merge/--squash/--rebase` (boolean) | `--squash/--rebase` (boolean) | `--style merge\|squash\|rebase` | `--method merge\|squash\|rebase` | `--squash`/`--rebase` booleans; translate to style/method |
| Delete branch on merge | `--delete-branch` | `--remove-source-branch` | no equivalent | `--delete` | normalize to `--delete-branch`; skip on tea |
| List state filter | `--state open\|closed\|merged\|all` | `--closed`/`--merged`/`--all` flags | `--state all\|open\|closed` | search-based | normalize `--state` to forge-specific flags |

## Sources

- Live CLI `--help` output: gh (verified locally 2026-03-17), glab (verified locally), tea (verified locally), fj (verified locally)
- Existing `src/adapter/pr.rs` flag translation patterns (v1.0 codebase)
- Existing `src/browse/mod.rs` URL construction patterns (v1.0 codebase)
- GitHub line-range URL format: `#L42-L55` anchor (HIGH confidence — documented behavior)
- GitLab line-range URL format: `#L42-55` (MEDIUM confidence — consistent with observed GitLab blob URLs)
- Gitea/Forgejo line-range: same as GitHub `#L42-L55` (MEDIUM confidence — Gitea codebase derives from GitHub conventions)

---
*Feature research for: gf unified git forge CLI — v1.1 milestone*
*Researched: 2026-03-17*
