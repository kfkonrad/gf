# Milestones

## v1.1 Feature Completeness & Quality (Shipped: 2026-03-18)

**Phases completed:** 5 phases, 13 plans
**Timeline:** 2026-03-17 → 2026-03-18 (2 days)
**Codebase:** 3,600 LOC Rust | 56 files changed | +12,666/-756 lines | 65 commits

**Key accomplishments:**
- Line-range deep-linking with per-forge URL fragments (`gf browse file.rs:42-55`)
- Complete flag normalization audit — 85 translation tests + 51 CLI audit tests across all forges
- Full PR lifecycle: list, merge, checkout, review, approve, browse across 4 forges
- Issue management: list, view, create, close, reopen across 4 forges
- Self-hosted forge auto-detection via CLI auth probing with persistent cache (CORE-04)
- Zero compiler warnings, 284 tests, clean codebase

**Archive:** `.planning/milestones/v1.1-ROADMAP.md`, `.planning/milestones/v1.1-REQUIREMENTS.md`

---

## v1.0 MVP (Shipped: 2026-03-17)

**Phases completed:** 5 phases, 12 plans
**Timeline:** 2026-03-16 (1 day)
**Codebase:** 2,689 LOC Rust | 68 files | 17 feat commits

**Key accomplishments:**
- Subprocess delegation with exec() process replacement, TTY inheritance, and correct exit code propagation
- Forge auto-detection from git remote URLs (HTTPS/SSH) with config file override for self-hosted instances
- Full command routing with flag normalization for PR, repo, and auth commands across GitHub/GitLab/Gitea/Forgejo
- Native browse URL construction for all four forges (no CLI delegation)
- Comprehensive alias system (mr=pr, r=repo, a=auth, b=browse, one-letter verbs) with shell completions

**Known gaps:**
- CORE-04 (self-hosted forge detection via CLI auth probing) deferred to v2
- Nyquist validation incomplete for phases 3, 4, 5

**Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`

---

