# Roadmap: gf (git forge)

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5 (shipped 2026-03-17)
- 🚧 **v1.1 Feature Completeness & Quality** — Phases 6-9 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-5) — SHIPPED 2026-03-17</summary>

- [x] Phase 1: Foundation (2/2 plans) — completed 2026-03-16
- [x] Phase 2: Forge Detection (3/3 plans) — completed 2026-03-16
- [x] Phase 3: Command Routing (4/4 plans) — completed 2026-03-16
- [x] Phase 4: Browse (2/2 plans) — completed 2026-03-16
- [x] Phase 5: Fix Self-Hosted Browse (1/1 plan) — completed 2026-03-16

</details>

### 🚧 v1.1 Feature Completeness & Quality (In Progress)

**Milestone Goal:** Complete the core command surface (PR workflows, issues, clone) and harden flag mappings across all forges.

- [x] **Phase 6: Browse Enhancements** — Line-range deep-linking and known-host deduplication (completed 2026-03-17)
- [ ] **Phase 7: Flag Normalization Audit** — Audit and verify all flag translations against live forge CLI help texts
- [x] **Phase 8: PR Workflow Commands** — Complete PR lifecycle: list, merge, checkout, review, approve, view, browse (completed 2026-03-18)
- [ ] **Phase 9: Issues, Clone, and Self-Hosted Detection** — Issue commands, repo clone, and CORE-04 forge probing

## Phase Details

### Phase 6: Browse Enhancements
**Goal**: Users can deep-link to specific line ranges in browser, and the known-host match table has a single source of truth
**Depends on**: Phase 5 (v1.0)
**Requirements**: BROWSE-01, BROWSE-02
**Success Criteria** (what must be TRUE):
  1. `gf browse src/main.rs:42` opens the browser to the correct line anchor for all four forges
  2. `gf browse src/main.rs:42-55` opens the browser to the correct line range anchor, with GitLab producing `#L42-55` and GitHub/Gitea/Forgejo producing `#L42-L55`
  3. Known-host matching logic exists in exactly one place in the codebase; browse and forge detection both use it
**Plans:** 2/2 plans complete
Plans:
- [ ] 06-01-PLAN.md — Deduplicate known-host match table (BROWSE-02)
- [ ] 06-02-PLAN.md — Line-range deep-linking with per-forge fragments (BROWSE-01)

### Phase 7: Flag Normalization Audit
**Goal**: Every canonical flag declared in the clap command tree is verified to produce the correct forge CLI flag in the adapter, with end-to-end test coverage
**Depends on**: Phase 6
**Requirements**: QUAL-01, QUAL-02, QUAL-03
**Success Criteria** (what must be TRUE):
  1. Running the test suite produces a passing result for every canonical flag × forge combination that is currently wired (no silent drops)
  2. Any flag mismatch found against current forge CLI `--help` output is fixed before Phase 8 begins
  3. New v1.1 flag normalizations (pr list, pr merge, pr checkout, pr review, issue list/view/create, repo clone) have verified mappings documented before their adapters are written
**Plans:** 1/2 plans executed
Plans:
- [ ] 07-01-PLAN.md — Macro test infrastructure, existing audit, mismatch fixes (QUAL-01, QUAL-03)
- [ ] 07-02-PLAN.md — v1.1 flag pre-mapping and integration audit (QUAL-02, QUAL-03)

### Phase 8: PR Workflow Commands
**Goal**: Users can perform the complete PR/MR lifecycle from the `gf` command without knowing which forge they are on
**Depends on**: Phase 7
**Requirements**: PR-01, PR-02, PR-03, PR-04, PR-05, PR-06, PR-07
**Success Criteria** (what must be TRUE):
  1. `gf pr list` shows open PRs/MRs and accepts `--state`, `--author`, and `--label` filter flags on all four forges
  2. `gf pr merge` merges a PR/MR with correct strategy flags (squash, rebase, merge) translated per forge
  3. `gf pr checkout` checks out a PR/MR branch locally on all four forges
  4. `gf pr review` / `gf pr approve` works on GitHub/GitLab and surfaces a clear unsupported error on tea/fj
  5. `gf browse --pr 123` opens the correct PR/MR URL in the browser for all four forges
**Plans:** 4/4 plans complete
Plans:
- [ ] 08-01-PLAN.md — Foundation: Result signature + UnsupportedFeature error + clap subcommands + test macros (PR-06)
- [ ] 08-02-PLAN.md — PR list + checkout + merge adapters + config schema (PR-01, PR-02, PR-03)
- [ ] 08-03-PLAN.md — PR review + approve adapters (PR-04, PR-05)
- [ ] 08-04-PLAN.md — Browse --pr/--mr/--issue URL builders (PR-07)

### Phase 9: Issues, Clone, and Self-Hosted Detection
**Goal**: Users can manage issues, clone repos, and have unknown self-hosted domains detected automatically via CLI auth probing
**Depends on**: Phase 8
**Requirements**: ISSUE-01, ISSUE-02, ISSUE-03, ISSUE-04, ISSUE-05, ISSUE-06, REPO-01, CORE-04
**Success Criteria** (what must be TRUE):
  1. `gf issue list`, `gf issue view <N>`, `gf issue create` work on all four forges with correct flag translations
  2. `gf issue close` and `gf issue reopen` change issue state on all four forges
  3. `gf browse --issue 42` opens the correct issue URL in the browser for all four forges
  4. `gf repo clone owner/repo` and `gf repo clone https://host/owner/repo` clone successfully on all four forges
  5. An unknown self-hosted domain not in config.toml is probed via forge CLI auth status commands and detected correctly; probe result is cached in `~/.cache/gf/`
**Plans:** 4 plans
Plans:
- [x] 09-00-PLAN.md — Test scaffolding for issue close/reopen + unsupported tests (ISSUE-04, ISSUE-05, REPO-01)
- [ ] 09-01-PLAN.md — Issue adapter (list, view, create, close, reopen) with per-forge translations (ISSUE-01, ISSUE-02, ISSUE-03, ISSUE-04, ISSUE-05)
- [x] 09-02-PLAN.md — Repo clone with [defaults] config and tea UnsupportedFeature (REPO-01)
- [ ] 09-03-PLAN.md — CORE-04 CLI auth probing with cache for self-hosted detection (CORE-04, ISSUE-06)

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-16 |
| 2. Forge Detection | v1.0 | 3/3 | Complete | 2026-03-16 |
| 3. Command Routing | v1.0 | 4/4 | Complete | 2026-03-16 |
| 4. Browse | v1.0 | 2/2 | Complete | 2026-03-16 |
| 5. Fix Self-Hosted Browse | v1.0 | 1/1 | Complete | 2026-03-16 |
| 6. Browse Enhancements | 2/2 | Complete   | 2026-03-17 | - |
| 7. Flag Normalization Audit | 1/2 | In Progress|  | - |
| 8. PR Workflow Commands | 4/4 | Complete   | 2026-03-18 | - |
| 9. Issues, Clone, and Self-Hosted Detection | v1.1 | 1/4 | In Progress | - |
