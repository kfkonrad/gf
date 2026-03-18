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

## Milestone: v1.1 — Feature Completeness & Quality

**Shipped:** 2026-03-18
**Phases:** 5 | **Plans:** 13

### What Was Built
- Line-range deep-linking for browse (`gf browse file.rs:42-55`) with per-forge URL fragments
- Declarative test macro infrastructure (translation_test!, audit_test!, unsupported_test!) — 165 generated tests
- Complete PR lifecycle: list, merge, checkout, review, approve, browse across 4 forges
- Full issue management: list, view, create, close, reopen across 4 forges
- Self-hosted forge auto-detection via CLI auth probing with persistent cache (CORE-04)
- Repo clone with URL/shorthand detection

### What Worked
- Pre-mapping tests in Phase 7 (ignored until adapters built) gave Phase 8/9 clear targets — remove `#[ignore]`, implement adapter, test passes
- UnsupportedFeature error pattern made forge limitations explicit and testable (unsupported_test! macro)
- Milestone audit caught browse probe/cache gap before shipping — fixed in-flight
- Phase 10 cleanup phase was small (1 plan) but closed all non-critical gaps cleanly

### What Was Inefficient
- browse::resolve_forge_type duplicated detection logic (config + known hosts only) instead of using full forge::detect chain — audit caught it but should have been designed correctly in Phase 6
- Phase 7 and Phase 9 ROADMAP.md checkboxes not auto-updated (showed `[ ]` for completed phases) — cosmetic but confusing
- v11_translation_test! macro created in Phase 7 then deleted in Phase 10 — should have used #[ignore] on regular translation_test! from the start

### Patterns Established
- detect_from_host() as public API for host-only detection (browse, future URL builders)
- Declarative test macros: translation_test! for flag mapping, audit_test! for CLI --help verification, unsupported_test! for error paths
- Mutex-guarded HOME env var tests to prevent parallel test races

### Key Lessons
1. When adding a new detection path (browse), reuse the full detection chain from day 1 — don't shortcut
2. Pre-mapping tests (#[ignore]d) are excellent for defining adapter contracts before implementation
3. Milestone audit is worth the time — caught a real integration gap that would have affected users
4. Cleanup phases work well for closing non-critical gaps that accumulate during feature phases

### Cost Observations
- Model mix: sonnet for execution/verification, opus for orchestration
- Sessions: completed across 2 days
- Notable: 13 plans across 5 phases; audit + fix cycle added browse probe support post-audit

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 5 | 12 | Established phase ordering and test isolation patterns |
| v1.1 | 5 | 13 | Pre-mapping tests, declarative test macros, milestone audit cycle |

### Cumulative Quality

| Milestone | Tests | LOC | Phases |
|-----------|-------|-----|--------|
| v1.0 | 96 unit + 25 integration | 2,689 | 5 |
| v1.1 | 97 unit + 165 flag audit + 25 integration = 284 | 3,600 | 10 |

### Top Lessons (Verified Across Milestones)

1. Strict dependency ordering between phases prevents integration surprises
2. Don't duplicate logic across modules — make the original public instead (v1.0 browse, v1.1 browse probe)
3. Pre-mapping tests define clear contracts for upcoming implementation phases
4. Milestone audits catch integration gaps that per-phase verification misses
