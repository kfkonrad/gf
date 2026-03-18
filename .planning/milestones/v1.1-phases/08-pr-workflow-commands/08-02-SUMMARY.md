---
phase: 08-pr-workflow-commands
plan: 02
subsystem: adapter
tags: [rust, clap, pr, mr, forge-translation, flag-mapping]

requires:
  - phase: 08-01
    provides: translate_pr() dispatch skeleton, UnsupportedFeature error type, pr list/checkout/merge CLI subcommands

provides:
  - translate_pr_list: state/author/label filtering with per-forge flag mappings, tea hard errors
  - translate_pr_checkout: positional number pass-through for all 4 forges
  - translate_pr_merge: strategy flags (--squash/--rebase/default) with forge-specific mapping, delete-branch translation
  - MergeConfig struct and extended GfConfig with per-forge delete_branch field
  - pub fn resolve_delete_branch(domain) for config-aware delete-branch defaults

affects: [08-03, 08-04, phase-09]

tech-stack:
  added: []
  patterns:
    - "Hard-error policy: unsupported forge+flag combos return GfError::UnsupportedFeature (not silent omit)"
    - "Forgejo pr list uses 'search' verb, not 'list'"
    - "glab state uses boolean flags (--closed/--merged/--all) not --state value"
    - "tea merge strategy uses --style; fj uses --method; gh/glab use --squash/--rebase/--merge"
    - "Default merge (no strategy flag): explicitly emit merge strategy per forge (except glab which uses no flag)"

key-files:
  created: []
  modified:
    - src/adapter/pr.rs
    - src/forge/mod.rs
    - tests/flag_audit.rs

key-decisions:
  - "Forgejo pr list uses 'pr search' verb not 'pr list' — required by fj CLI design"
  - "tea --author and tea --label on pr list are hard errors (UnsupportedFeature), not silent omissions"
  - "tea --delete-branch on pr merge is a hard error; gh/glab/fj each have distinct flags"
  - "Default merge (no flag) explicitly emits strategy per forge so behavior is deterministic"
  - "resolve_delete_branch takes domain string not ForgeType to allow per-host config overrides"

patterns-established:
  - "Forge-specific flag remapping in adapter functions, not in CLI layer"
  - "Hard errors for missing forge feature (UnsupportedFeature) vs silent omit for cosmetic flags (--draft)"

requirements-completed: [PR-01, PR-02, PR-03]

duration: 3min
completed: 2026-03-18
---

# Phase 08 Plan 02: PR List/Checkout/Merge Adapter Translations Summary

**PR list/checkout/merge adapter with forge-specific strategy flags, Forgejo search-verb remapping, and tea hard errors for unsupported --author/--label/--delete-branch**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-18T09:42:57Z
- **Completed:** 2026-03-18T09:45:47Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `translate_pr_list`: Forgejo uses `pr search` verb, remaps `--author` to `--creator` and `--label` to `--labels`; glab uses `--closed/--merged/--all` boolean flags; tea returns `UnsupportedFeature` for `--author`/`--label`
- `translate_pr_checkout`: simple positional pass-through for all 4 forges with correct verb (`pr`/`mr`/`pulls`)
- `translate_pr_merge`: forge-specific strategy mapping (`--style` for tea, `--method` for fj), explicit default merge strategy even when no flag given, delete-branch with forge-specific flag names, tea hard error
- Config schema extended with `MergeConfig` and per-forge `delete_branch` field; `resolve_delete_branch()` public function for config-aware defaults
- 24 new tests passing (17 list/checkout un-ignored, 7 merge un-ignored, 4 default-strategy, 4 delete-branch, 2 tea unsupported)

## Task Commits

1. **Task 1: translate_pr_list, translate_pr_checkout, translate_pr_merge** - `20a3958` (feat)
2. **Task 2: MergeConfig schema + resolve_delete_branch + config unit tests** - `ce13bc9` (feat)

## Files Created/Modified

- `src/adapter/pr.rs` - Added `translate_pr_list`, `translate_pr_checkout`, `translate_pr_merge` functions; dispatch arms in `translate_pr`
- `src/forge/mod.rs` - Added `MergeConfig` struct, `delete_branch` on `ForgeEntry`, `merge` on `GfConfig`, `pub fn resolve_delete_branch`
- `tests/flag_audit.rs` - Un-ignored 24 v11 tests, added 11 new passing tests

## Decisions Made

- Forgejo `pr list` routes to `pr search` because `fj pr list` does not exist as a command
- tea `--author` and `--label` on pr list are hard errors (not silent omit) — matches Phase 8 hard-error policy
- Default merge (no strategy flag) explicitly emits forge strategy to avoid ambiguity: gh gets `--merge`, glab gets nothing (its default), tea gets `--style merge`, fj gets `--method merge`
- `resolve_delete_branch` accepts a domain string (not ForgeType) to support per-host TOML config overrides

## Deviations from Plan

**1. [Rule 1 - Bug] Linter injected incorrect stub implementations before edits**

- **Found during:** Task 1 (translate_pr_list)
- **Issue:** A code formatter/linter injected placeholder stubs for `translate_pr_list`, `translate_pr_checkout`, and `translate_pr_merge` that were missing Forgejo remapping, tea unsupported errors, and forge-specific merge strategy logic
- **Fix:** Replaced all three stubs with correct implementations matching plan spec
- **Files modified:** `src/adapter/pr.rs`
- **Verification:** `cargo test --test flag_audit` passes all 121 tests

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in linter-generated stub)
**Impact on plan:** Necessary correction. No scope creep.

## Issues Encountered

None beyond the linter stub issue documented above.

## Next Phase Readiness

- PR list/checkout/merge translations complete for all 4 forges
- `resolve_delete_branch` available for future config-aware merge behavior
- Remaining Phase 8 work: pr review/approve translations (08-03)

---
*Phase: 08-pr-workflow-commands*
*Completed: 2026-03-18*
