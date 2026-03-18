# Requirements: gf (git forge)

**Defined:** 2026-03-17
**Core Value:** One `gf` command syntax that works on any forge, with zero knowledge of which forge you're on

## v1.1 Requirements

Requirements for v1.1 Feature Completeness & Quality. Each maps to roadmap phases.

### PR Workflows

- [x] **PR-01**: User can list PRs/MRs with filter flags (state, author, label)
- [x] **PR-02**: User can merge a PR/MR with strategy flags (squash, rebase, merge)
- [x] **PR-03**: User can checkout a PR/MR branch locally
- [x] **PR-04**: User can review a PR/MR (comment)
- [x] **PR-05**: User can approve a PR/MR
- [x] **PR-06**: User can view a specific PR/MR by number
- [x] **PR-07**: User can browse a PR/MR in the browser (`gf browse --pr 123`)

### Issues

- [x] **ISSUE-01**: User can list issues with filter flags (state, author, label)
- [x] **ISSUE-02**: User can view a specific issue by number
- [x] **ISSUE-03**: User can create a new issue with title and body
- [x] **ISSUE-04**: User can close an issue
- [x] **ISSUE-05**: User can reopen a closed issue
- [ ] **ISSUE-06**: User can browse an issue in the browser (`gf browse --issue 42`)

### Repository

- [x] **REPO-01**: User can clone a repo via `gf repo clone owner/repo` or full URL

### Browse Enhancements

- [x] **BROWSE-01**: User can deep-link to line ranges (`gf browse file.rs:42-55`) with correct per-forge fragment
- [x] **BROWSE-02**: Known-host match table deduplicated between browse and forge detection modules

### Quality

- [x] **QUAL-01**: All existing flag normalizations audited and verified against current forge CLI help texts
- [x] **QUAL-02**: All new v1.1 flag normalizations verified against current forge CLI help texts
- [x] **QUAL-03**: Tests cover flag translation for every command × forge combination

### Self-Hosted Detection

- [ ] **CORE-04**: Unknown domains probed via CLI auth status commands (gh, glab, tea, fj) with fallback to config file

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### PR Workflows

- **PR-08**: User can view PR/MR CI status
- **PR-09**: User can add reviewers to a PR/MR

### Issues

- **ISSUE-07**: User can comment on an issue
- **ISSUE-08**: User can assign labels to an issue

### Moderation

- **MODR-01**: Batch operations across multiple PRs/issues

## Out of Scope

| Feature | Reason |
|---------|--------|
| Direct forge API calls | Auth is fully delegated to underlying CLIs; no API tokens managed by gf |
| Own config for auth/tokens | Auth delegation to gh/glab/tea/fj is a core design decision |
| Mobile/GUI interface | gf is a CLI tool |
| PR status/CI integration | Requires API calls, violates no-API constraint |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BROWSE-01 | Phase 6 | Complete |
| BROWSE-02 | Phase 6 | Complete |
| QUAL-01 | Phase 7 | Complete |
| QUAL-02 | Phase 7 | Complete |
| QUAL-03 | Phase 7 | Complete |
| PR-01 | Phase 8 | Complete |
| PR-02 | Phase 8 | Complete |
| PR-03 | Phase 8 | Complete |
| PR-04 | Phase 8 | Complete |
| PR-05 | Phase 8 | Complete |
| PR-06 | Phase 8 | Complete |
| PR-07 | Phase 8 | Complete |
| ISSUE-01 | Phase 9 | Complete |
| ISSUE-02 | Phase 9 | Complete |
| ISSUE-03 | Phase 9 | Complete |
| ISSUE-04 | Phase 9 | Complete |
| ISSUE-05 | Phase 9 | Complete |
| ISSUE-06 | Phase 9 | Pending |
| REPO-01 | Phase 9 | Complete |
| CORE-04 | Phase 9 | Pending |

**Coverage:**
- v1.1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-18 — Phase 9 Plan 00 completed (test scaffolding)*
