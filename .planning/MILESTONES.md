# Milestones

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

