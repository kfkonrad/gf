# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — MVP

**Shipped:** 2026-03-17
**Phases:** 5 | **Plans:** 12

### What Was Built
- Subprocess runner with exec() process replacement and TTY inheritance
- Forge auto-detection from git remote URLs with config file override
- Command routing with flag normalization for PR/repo/auth across 4 forges
- Native browse URL construction for GitHub/GitLab/Gitea/Forgejo
- Alias system with shell completions

### What Worked
- Strict dependency ordering (foundation → detection → routing → browse) prevented integration surprises
- Wave 0 test infrastructure in each phase caught issues early
- exec() decision eliminated TTY/signal complexity — OS handles everything
- clap builder API gave precise alias control without derive macro limitations

### What Was Inefficient
- Phase 4 browse duplicated forge detection logic (known-host table) instead of reusing forge module — required Phase 5 to fix self-hosted gap
- CORE-04 was planned for Phase 2 then dropped mid-phase — should have been scoped out earlier during discuss-phase
- Duplicate decisions recorded in STATE.md (Phases 3 and 4 had copy-paste duplicates)

### Patterns Established
- Integration tests use temp git repos + isolated PATH bin dirs — survives interface changes between phases
- Human verification checkpoints for TTY/browser behaviors that can't be automated
- browse module uses early-intercept pattern (before forge::detect) since it handles detection internally

### Key Lessons
1. When a module duplicates logic from another module, immediately check if the original should be made public instead — prevents integration gaps
2. exec() on Unix is the right default for CLI wrappers — zero overhead, perfect TTY behavior
3. clap visible_alias is superior to hidden_alias for UX — aliases show up in --help automatically

### Cost Observations
- Model mix: primarily sonnet for execution, opus for planning/verification
- Sessions: completed in single day
- Notable: 12 plans across 5 phases executed rapidly due to clear phase boundaries

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 5 | 12 | Established phase ordering and test isolation patterns |

### Cumulative Quality

| Milestone | Tests | LOC | Phases |
|-----------|-------|-----|--------|
| v1.0 | 96 unit + 25 integration | 2,689 | 5 |

### Top Lessons (Verified Across Milestones)

1. Strict dependency ordering between phases prevents integration surprises
2. Don't duplicate logic across modules — make the original public instead
