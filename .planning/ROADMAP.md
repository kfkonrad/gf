# Roadmap: gf (git forge)

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5 (shipped 2026-03-17)
- ✅ **v1.1 Feature Completeness & Quality** — Phases 6-10 (shipped 2026-03-18)
- 🔄 **v1.2 Workflow Completeness** — Phases 11-14 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-5) — SHIPPED 2026-03-17</summary>

- [x] Phase 1: Foundation (2/2 plans) — completed 2026-03-16
- [x] Phase 2: Forge Detection (3/3 plans) — completed 2026-03-16
- [x] Phase 3: Command Routing (4/4 plans) — completed 2026-03-16
- [x] Phase 4: Browse (2/2 plans) — completed 2026-03-16
- [x] Phase 5: Fix Self-Hosted Browse (1/1 plan) — completed 2026-03-16

</details>

<details>
<summary>✅ v1.1 Feature Completeness & Quality (Phases 6-10) — SHIPPED 2026-03-18</summary>

- [x] Phase 6: Browse Enhancements (2/2 plans) — completed 2026-03-17
- [x] Phase 7: Flag Normalization Audit (2/2 plans) — completed 2026-03-17
- [x] Phase 8: PR Workflow Commands (4/4 plans) — completed 2026-03-18
- [x] Phase 9: Issues, Clone, and Self-Hosted Detection (4/4 plans) — completed 2026-03-18
- [x] Phase 10: Cleanup — Dead Code and Test Gaps (1/1 plan) — completed 2026-03-18

</details>

### 🔄 v1.2 Workflow Completeness (Phases 11-14) — IN PROGRESS

- [x] Phase 11: PR Checks (1/1 plan) — completed 2026-03-19
- [x] Phase 12: Issue and PR Comments — pending (completed 2026-03-19)
- [x] Phase 13: PR and Issue Edit — pending (completed 2026-03-19)
- [ ] Phase 14: Final Integration and Polish — pending (depends on 11, 12, 13)

### Phase 11: PR Checks

**Goal:** `gf pr checks <number>` produces correct forge CLI args for GitHub, GitLab, Forgejo; returns UnsupportedFeature for Gitea.
**Depends on:** nothing
**Requirements:** PR-08

### Phase 12: Issue and PR Comments

**Goal:** `gf issue comment <number> --body "text"` and `gf pr comment <number> --body "text"` post comments on issues and PRs across all supported forges.
**Depends on:** nothing
**Requirements:** ISSUE-07
**Plans:** 1/1 plans complete

Plans:
- [ ] 12-01-PLAN.md — Add comment subcommands (clap + adapters + tests) for issue and PR domains

### Phase 13: PR and Issue Edit

**Goal:** `gf pr edit` and `gf issue edit` add and remove labels, reviewers, and assignees, with per-flag UnsupportedFeature errors when a forge CLI lacks the capability.
**Depends on:** nothing
**Requirements:** PR-09, ISSUE-08
**Plans:** 1/1 plans complete

Plans:
- [ ] 13-01-PLAN.md — Add edit subcommands (clap + translate_pr_edit/translate_issue_edit + 58 tests) for PR and issue domains

### Phase 14: Final Integration and Polish

**Goal:** All new v1.2 commands pass integration tests via assert_cmd; help text is correct; PROJECT.md updated; zero warnings confirmed.
**Depends on:** Phase 11, Phase 12, Phase 13
**Requirements:** PR-08, PR-09, ISSUE-07, ISSUE-08
**Plans:** 2 plans

Plans:
- [ ] 14-01-PLAN.md — Restore lost Phase 11/12 code (checks + comment)
- [ ] 14-02-PLAN.md — Integration tests and documentation update

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-16 |
| 2. Forge Detection | v1.0 | 3/3 | Complete | 2026-03-16 |
| 3. Command Routing | v1.0 | 4/4 | Complete | 2026-03-16 |
| 4. Browse | v1.0 | 2/2 | Complete | 2026-03-16 |
| 5. Fix Self-Hosted Browse | v1.0 | 1/1 | Complete | 2026-03-16 |
| 6. Browse Enhancements | v1.1 | 2/2 | Complete | 2026-03-17 |
| 7. Flag Normalization Audit | v1.1 | 2/2 | Complete | 2026-03-17 |
| 8. PR Workflow Commands | v1.1 | 4/4 | Complete | 2026-03-18 |
| 9. Issues, Clone, Self-Hosted | v1.1 | 4/4 | Complete | 2026-03-18 |
| 10. Cleanup & Test Gaps | v1.1 | 1/1 | Complete | 2026-03-18 |
| 11. PR Checks | v1.2 | 1/1 | Complete | 2026-03-19 |
| 12. Issue & PR Comments | 1/1 | Complete    | 2026-03-19 | — |
| 13. PR & Issue Edit | 1/1 | Complete    | 2026-03-19 | — |
| 14. Final Integration | v1.2 | 0/2 | Pending | — |
