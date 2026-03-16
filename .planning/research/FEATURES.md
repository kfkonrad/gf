# Feature Research

**Domain:** Unified git forge CLI wrapper (gh, glab, tea, fj)
**Researched:** 2026-03-16
**Confidence:** HIGH (command structures verified against official docs and man pages)

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `gf pr create` | Core reason to use the tool — creating PRs/MRs is the highest-frequency operation | MEDIUM | Must normalize --title, --body/--description, --draft, --base across all four forges |
| `gf pr list` | Second most common operation — checking open PRs | LOW | Passthrough works well here |
| `gf pr view [number]` | Inspecting a specific PR before acting on it | LOW | Passthrough; tea uses pulls not pr |
| `gf pr merge [number]` | Closing the loop on reviewed PRs | LOW | gh/glab/tea/fj all support this |
| `gf repo clone <url>` | Universal entry point for working on a new project | LOW | All four CLIs support clone |
| `gf repo fork` | Fork-and-clone workflow is standard across all forges | LOW | Direct passthrough per forge |
| `gf repo create` | Creating new repos from CLI is expected | LOW | Passthrough |
| `gf repo view` | Viewing repo details in terminal | LOW | Passthrough |
| `gf auth login` | Users must authenticate before any operation | LOW | Fully delegated to underlying CLI |
| `gf auth logout` | Clean session termination | LOW | Fully delegated |
| `gf auth status` | Verifying which account/token is active | LOW | Fully delegated |
| `gf browse` | Opening repo or file in browser — ubiquitous in gh, expected in a wrapper | MEDIUM | Native implementation required (tea's is broken); must construct URL from remote |
| Forge auto-detection | The defining feature — users expect `gf` to just work on any repo | MEDIUM | Parse git remote URL; detect github.com, gitlab.com, *.gitea.*, *.forgejo.* or custom hosts |
| Clear error + install hint | When underlying CLI is absent, a cryptic error is worse than useless | LOW | Check PATH at startup for required binary |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Flag normalization (known flags) | Users learn one flag set (`--body` not `--description`); biggest friction point in switching forges | MEDIUM | Map canonical flags to forge equivalents at dispatch time; unknown flags pass through untouched |
| `gf browse` with file deep-link | `gh browse path/to/file.rs` is beloved; tea doesn't support it reliably; making it work everywhere adds real value | MEDIUM | Parse git remote + current branch + file path; construct forge-specific URL pattern |
| `--remote` override flag | Power users have multi-remote repos (upstream + fork); explicit routing is safer than guessing | LOW | Accept `--remote <name>` as global flag; resolve that remote instead of origin |
| Single binary distribution | Rust produces a static binary; no runtime, no dependency manager, instant install | LOW | Design constraint already chosen; just do it right |
| Informative passthrough | When delegating, optionally show users what command was actually run (--verbose or env var) | LOW | Useful for debugging and learning; debug log what `gf pr create` expanded to |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Own config file / token store | Users want `gf` to handle all auth in one place | Duplicates auth state with gh/glab/tea/fj; creates two-source-of-truth bugs; security surface for token storage | Fully delegate auth to underlying CLIs; they handle token refresh, OAuth, keychain integration |
| Issues commands (v1) | Issues are the other half of forge workflows | Doubles scope without validating the PR/repo core loop; different forges have different issue models (GitLab has work items, iterations, incidents) | Defer to v2 after validating core PR commands |
| Line-range browse links (file.rs:42-55) | Developers frequently share file+line references | Each forge has a different URL scheme for line ranges; fragile to implement correctly for all four | Defer to v2; build URL construction abstraction first in v1 |
| CI/pipeline commands | glab ci and gh run are popular; users want unified pipeline access | Each forge's CI model is radically different (Actions YAML vs .gitlab-ci.yml vs Gitea Actions vs Forgejo Actions); normalization is nearly impossible | Expose raw passthrough for `gf ci` → `glab ci`, `gf run` → `gh run` without attempting normalization |
| Direct forge API calls | Bypassing CLIs would allow richer functionality | Breaks the "thin router" design; requires maintaining API clients for 4 forges; auth token management complexity; CLIs already handle pagination, error handling, retries | Stay with CLI delegation in v1; API access only if a CLI doesn't cover a needed operation |
| Multi-remote auto-routing | Detect all remotes and pick the "right" forge automatically | Ambiguous when origin is GitHub fork and upstream is GitLab — which forge do you mean? Silent wrong-forge calls are worse than an error | Require explicit `--remote` flag for non-origin remotes; default is always origin |
| Shell aliases / config system | Let users remap `gf mr` → `gf pr` etc. | Config systems become load-bearing infrastructure; versioning, migration, conflicts with forge-native configs | Hardcode canonical command names; teach users the mapping once |

## Feature Dependencies

```
forge auto-detection
    └──required by──> ALL other commands (every dispatch requires knowing the forge)
                          └──required by──> flag normalization (must know target forge to map flags)

auth login/logout/status
    └──required for──> pr create, pr merge, repo create, repo fork (write operations)
    (read ops like pr list, repo view, browse work without auth on public repos)

browse (native)
    └──requires──> forge auto-detection
    └──requires──> git remote URL parsing
    └──enhanced by──> file path argument support (v1.x)
    └──enhanced by──> line-range support (v2+)

pr create
    └──requires──> forge auto-detection
    └──requires──> flag normalization (--body vs --description)
    └──enhanced by──> --draft flag normalization (same name across all forges — safe to pass through)

repo clone
    └──independent──> (does NOT require forge auto-detection; takes explicit URL)

missing CLI detection
    └──required by──> ALL commands (must check before any dispatch)
```

### Dependency Notes

- **All commands require forge auto-detection:** This is the first thing that must be built; everything else is blocked on it.
- **Flag normalization requires knowing the forge:** Normalization is a post-detection step; the flag mapping table is keyed by forge.
- **Browse is native:** It cannot delegate because tea's browse is broken; it is the one command `gf` must implement end-to-end.
- **repo clone is independent:** The URL contains the forge host; no need to detect from git remote.

## Flag Normalization Reference

Critical for implementation — flags that differ across forges for the same concept:

| Canonical Flag | gh | glab | tea | fj | Notes |
|---------------|----|------|-----|----|-------|
| `--title` | `--title` / `-t` | `--title` / `-t` | `--title` | `--title` | Same across all — no mapping needed |
| `--body` | `--body` / `-b` | `--description` / `-d` | `--description` | `--description` | **MISMATCH**: gh uses `--body`, others use `--description` |
| `--draft` | `--draft` / `-d` | `--draft` | `--draft` (unsupported, Gitea has drafts since v1.20) | `--draft` | Mostly consistent; tea support is version-dependent |
| `--base` | `--base` / `-B` | `--target-branch` / `-b` | `--base` | `--base` | **MISMATCH**: glab uses `--target-branch` |
| `--head` | `--head` / `-H` | `--source-branch` / `-s` | `--head` | `--head` | **MISMATCH**: glab uses `--source-branch` |
| `--reviewer` | `--reviewer` / `-r` | `--reviewer` | (not supported) | `--reviewer` | tea lacks reviewer assignment in CLI |
| `--assignee` | `--assignee` / `-a` | `--assignee` / `-a` | `--assignees` | `--assignee` | tea uses plural `--assignees` |
| `--label` | `--label` / `-l` | `--label` / `-l` | `--labels` | `--label` | tea uses plural `--labels` |
| `--web` | `--web` / `-w` | `--web` / `-w` | (not available) | `--web` | tea has no --web for pr create |
| `--fill` | `--fill` / `-f` | `--fill` / `-f` | (no equivalent) | (no equivalent) | gh/glab only |

**Canonical flag set recommendation:** Use `gh`'s flag names as canonical (`--body` not `--description`, `--base` not `--target-branch`) since gh has the largest user base and most developers will arrive from GitHub familiarity.

## PR/MR Subcommand Coverage by Forge

| Subcommand | gh | glab | tea | fj |
|------------|----|------|-----|----|
| `pr create` | `gh pr create` | `glab mr create` | `tea pulls create` | `fj pr create` |
| `pr list` | `gh pr list` | `glab mr list` | `tea pulls list` | `fj pr list` |
| `pr view` | `gh pr view` | `glab mr view` | `tea pulls ls` (no view) | `fj pr view` |
| `pr merge` | `gh pr merge` | `glab mr merge` | `tea pulls merge` | `fj pr merge` |
| `pr checkout` | `gh pr checkout` | `glab mr checkout` | `tea pulls checkout` | (not documented) |
| `pr review` | `gh pr review` | `glab mr approve/revoke` | `tea pulls review` | (not documented) |
| `pr close` | `gh pr close` | `glab mr close` | `tea pulls close` | `fj pr close` |
| `pr reopen` | `gh pr reopen` | `glab mr reopen` | `tea pulls reopen` | (not documented) |
| `pr diff` | `gh pr diff` | `glab mr diff` | (not available) | (not documented) |
| `pr status` | `gh pr status` | (not available) | (not available) | (not documented) |
| `pr comment` | `gh pr comment` | `glab mr note` | `tea comment` (issue+pr) | (not documented) |

**Note:** tea uses `pulls` as the command group, not `pr`. glab uses `mr` (merge request). `gf` maps all to `pr` as the canonical name.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] Forge auto-detection from git remote — core value proposition; nothing else works without it
- [ ] Missing CLI detection with install hint — prevents confusing errors that make the tool seem broken
- [ ] `gf pr create` with flag normalization for --body, --base, --head — highest-frequency write operation
- [ ] `gf pr list` — highest-frequency read operation
- [ ] `gf pr view` — view before act
- [ ] `gf pr merge` — close the loop
- [ ] `gf repo clone` — universal entry point
- [ ] `gf repo view` — repo inspection
- [ ] `gf auth login/logout/status` — delegated auth (passthrough, low effort)
- [ ] `gf browse` native implementation — native because tea's is broken; file path argument in scope
- [ ] `--remote` global flag — escape hatch for non-origin remotes

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] `gf pr checkout` — common in PR review workflows; once pr commands are stable
- [ ] `gf pr review` / `gf pr approve` — review workflow; depends on how PR create performs
- [ ] `gf repo create` / `gf repo fork` — write operations; validate read operations first
- [ ] `gf browse` with line-range support (file.rs:42-55) — after URL construction abstraction is solid
- [ ] Verbose/debug mode showing expanded underlying command — useful for power users and debugging

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] Issues commands (`gf issue create/list/view/close`) — doubles scope; validate PR workflow first
- [ ] CI/pipeline passthrough (`gf ci`, `gf run`) — raw passthrough with no normalization; low value in v1
- [ ] Snippet/gist commands — niche; forge support varies widely
- [ ] Release commands — `gf release create` is useful but not core to the PR workflow being validated

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Forge auto-detection | HIGH | MEDIUM | P1 |
| `gf pr create` + flag normalization | HIGH | MEDIUM | P1 |
| `gf browse` native | HIGH | MEDIUM | P1 |
| Missing CLI error + hint | HIGH | LOW | P1 |
| `gf pr list/view/merge` | HIGH | LOW | P1 |
| `gf auth login/logout/status` | HIGH | LOW | P1 |
| `gf repo clone/view` | MEDIUM | LOW | P1 |
| `--remote` global flag | MEDIUM | LOW | P1 |
| `gf pr checkout/review` | MEDIUM | LOW | P2 |
| `gf repo create/fork` | MEDIUM | LOW | P2 |
| Browse line-range support | MEDIUM | MEDIUM | P2 |
| Verbose/debug mode | LOW | LOW | P2 |
| Issues commands | HIGH | HIGH | P3 |
| CI/pipeline passthrough | MEDIUM | LOW | P3 |
| Release commands | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | gh | glab | tea | fj | gf approach |
|---------|-----|------|-----|----|-------------|
| PR/MR create | `gh pr create` | `glab mr create` | `tea pulls create` | `fj pr create` | `gf pr create` → normalized dispatch |
| Browse | `gh browse [path]` | (no equivalent) | `tea open` (broken) | (not documented) | Native implementation |
| Auto-detect forge | Single forge (GitHub) | Single forge (GitLab) | Single forge (Gitea) | Single forge (Forgejo) | **Core differentiator** |
| Flag normalization | N/A (own flags) | N/A (own flags) | N/A (own flags) | N/A (own flags) | **Core differentiator** |
| Auth | `gh auth login` | `glab auth login` | `tea logins add` | `fj auth login` | Delegated passthrough |
| Repo clone | `gh repo clone` | `glab repo clone` | `tea clone` | `fj repo clone` | `gf repo clone` → dispatch |
| Shell completion | `gh completion` | `glab completion` | `tea completion` | `fj completion` | `gf completion` → generate for gf |
| JSON output | `--json` flag | `--output json` | (limited) | (limited) | Passthrough; no normalization in v1 |
| API escape hatch | `gh api` | `glab api` | (tea admin) | (fj api?) | Not exposed in v1 |

## Sources

- [GitHub CLI Manual](https://cli.github.com/manual/) — official, HIGH confidence
- [gh pr create flags](https://cli.github.com/manual/gh_pr_create) — official, HIGH confidence
- [GitLab CLI docs](https://docs.gitlab.com/cli/) — official, HIGH confidence
- [glab mr create flags](https://docs.gitlab.com/cli/mr/create/) — official, HIGH confidence
- [tea CLI docs](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md) — official, HIGH confidence
- [forgejo-cli (fj) Codeberg](https://codeberg.org/forgejo-contrib/forgejo-cli) — official, MEDIUM confidence (docs sparse)
- [glab-mr-create man page (Ubuntu)](https://manpages.ubuntu.com/manpages/noble/man1/glab-mr-create.1.html) — MEDIUM confidence (may lag upstream)

---
*Feature research for: unified forge CLI wrapper (gf)*
*Researched: 2026-03-16*
