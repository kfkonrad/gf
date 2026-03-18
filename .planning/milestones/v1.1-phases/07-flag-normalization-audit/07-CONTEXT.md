# Phase 7: Flag Normalization Audit - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Audit and verify all flag translations against live forge CLI help texts. Fix any mismatches found. Pre-map and verify new v1.1 flag normalizations (pr list, pr merge, pr checkout, pr review, issue list/view/create, repo clone) before their adapters are written in Phase 8.

</domain>

<decisions>
## Implementation Decisions

### Audit methodology
- Programmatic --help parsing: run actual forge CLIs and verify flags/subcommands exist
- Verify both flag names AND subcommand names (e.g. `glab mr`, `tea pulls`, `tea logins`)
- Flag name existence only — no value type checking
- Missing forge CLI in test environment = hard test failure (all 4 CLIs required)
- Target latest stable CLI versions (whatever --version returns)

### Test coverage strategy
- Macro-generated tests: Rust macro produces named test functions from a structured data table
- One test per (command, flag, forge) triple — fine-grained failure reporting
- Test table IS the canonical flag mapping documentation (comments explain each translation)
- Replace existing per-function tests with macro-generated equivalents — single source of truth
- Integration audit tests (--help parsing) in separate `tests/flag_audit.rs`
- All tests run in regular `cargo test` — no feature gating

### v1.1 mapping documentation
- Phase 7 pre-maps canonical flags for all new v1.1 commands before Phase 8 begins
- Canonical flags only — what gf will expose, not every forge flag
- Explicit "UNSUPPORTED" marker for forge×flag combinations that don't exist (e.g. pr review on tea)
- Adapter must produce a clear error message for unsupported combinations; test verifies this
- All v1.1 pre-mapped entries verified against real --help output in Phase 7
- Claude determines canonical flag list from REQUIREMENTS.md

### Mismatch handling
- Fix immediately — v1.1 is pre-stable, no backwards compatibility concerns
- Passing test suite is sufficient audit report — no separate report artifact
- Fixes show up in git diff

### Claude's Discretion
- Macro design and syntax
- --help output parsing approach (regex, string matching, etc.)
- Exact canonical flag selection for v1.1 commands
- How to structure the test table (module organization, grouping)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Adapter implementation
- `src/adapter/mod.rs` — Translation dispatch; routes to per-subcommand translators
- `src/adapter/pr.rs` — PR flag translations (create, view); existing tests to be replaced by macro table
- `src/adapter/repo_auth.rs` — Repo and Auth flag translations; existing tests to be replaced by macro table

### Requirements
- `.planning/REQUIREMENTS.md` — QUAL-01 (audit existing), QUAL-02 (verify new v1.1), QUAL-03 (test coverage per command×forge)
- `.planning/ROADMAP.md` — Phase 7 success criteria; Phase 8 requirements (PR-01 through PR-07) define new canonical flags

### CLI definitions
- `src/cmd/mod.rs` — Clap command tree defining canonical flags
- `src/forge/mod.rs` — ForgeType enum (Github, Gitlab, Gitea, Forgejo)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `adapter::pr::translate_pr_create` — Example of per-forge flag translation pattern (--body→--description, --base→--target-branch)
- `adapter::repo_auth::translate_auth_login` — Example of subcommand remapping (auth login→logins add for tea)
- `cmd::build_cli()` — Used by existing tests to parse args through clap

### Established Patterns
- Translation functions take (ForgeType, &ArgMatches) → Vec<String>
- Passthrough: unrecognized flags after `--` appended verbatim
- Per-forge match arms for flag/subcommand differences

### Integration Points
- New macro table replaces inline `#[test]` blocks in `adapter/pr.rs` and `adapter/repo_auth.rs`
- Integration test `tests/flag_audit.rs` invokes `gh`, `glab`, `tea`, `fj` binaries
- v1.1 table entries feed directly into Phase 8 adapter implementation

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-flag-normalization-audit*
*Context gathered: 2026-03-17*
