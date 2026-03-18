# Phase 10: Cleanup — Dead Code and Test Gaps - Research

**Researched:** 2026-03-18
**Domain:** Rust dead code elimination, test macro cleanup, zero-warning build
**Confidence:** HIGH

## Summary

Phase 10 is a pure code-quality cleanup phase. No new features are added. Every gap was
identified by a prior milestone audit; the research task is to understand the exact state of
each gap so the planner can map tasks one-to-one with things that need changing.

All four gaps are verified by direct code inspection of the live tree. There are no unknowns.

**Primary recommendation:** Four discrete tasks, one per gap. No library changes required.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REPO-01 | User can clone a repo via `gf repo clone` | `translate_repo_clone` is implemented; `get_default_clone_host()` was planned as a helper but is never called from it — must either wire in or delete |
| QUAL-02 | All new v1.1 flag normalizations verified against forge CLI help texts | `audit_test!` entries for `fj issue search --labels` and `fj issue search --creator` are missing from `tests/flag_audit.rs` |
| QUAL-03 | Tests cover flag translation for every command × forge combination | `translation_test!` for `gf issue list --author` (Forgejo) is missing; `v11_translation_test!` macro still exists with zero invocations |
| ISSUE-01 | User can list issues with filter flags (state, author, label) | Code in `issue.rs` already remaps `--author` → `--creator` for Forgejo, but no `translation_test!` exercises that path |
</phase_requirements>

## Standard Stack

No new libraries needed. This phase edits existing Rust source and test files only.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (existing) | — | Rust + Cargo | No additions required |

**Installation:** None.

## Gap Inventory (verified by direct code inspection)

### Gap 1: Dead function — `get_default_clone_host()`

**File:** `src/forge/mod.rs` line 122
**Status:** Defined, public, never called from production code.

`translate_repo_clone` in `src/adapter/repo_auth.rs` (lines 141-181) does NOT call
`get_default_clone_host()`. The shorthand `owner/repo` path simply passes the value straight
through to the forge CLI. The design decision from Phase 9 CONTEXT.md says:

> For shorthand, we just pass owner/repo to the CLI — gh/glab/fj know their default hosts

So the function was written in anticipation of a need that was not materialised. Two valid
resolutions:

1. **Delete** `get_default_clone_host()` and `CloneHostNotConfigured` — correct if the
   design intent is to let the forge CLI handle host resolution.
2. **Wire in** — call `get_default_clone_host()` in `translate_repo_clone`, error if None
   when shorthand is detected. This would enforce config-backed host resolution for shorthand.

The Phase 9 CONTEXT.md locked decision states full URL passes through; for shorthand it says
"we just pass owner/repo to the CLI". This implies Option 1 (delete) is the correct resolution.
However, the success criterion says "wired into `translate_repo_clone` OR removed" — both are
valid. The planner should choose based on the success-criterion wording; either resolves the
dead-code warning.

**Compiler warning produced:** Yes — `cargo build --release` emits:
```
warning: variants `SpawnFailed` and `CloneHostNotConfigured` are never constructed
```

Note: `SpawnFailed` is actually used in `src/runner.rs` line 45 (Windows spawn path). The
compiler warning fires because the dead code analysis does not count the Windows-only path as
a construction. `SpawnFailed` does NOT need to be deleted; only `CloneHostNotConfigured` is
genuinely unused.

### Gap 2: Dead macro — `v11_translation_test!`

**File:** `tests/flag_audit.rs` lines 380-393
**Status:** Macro defined, zero invocations.

The macro was introduced in Phase 7 to pre-map flag translations with `#[ignore]` until Phase
8 implemented the adapters. All 45 pre-mapped entries were later converted to live
`translation_test!` calls (the `#[ignore]` was removed). The macro definition itself was
never deleted.

**Resolution:** Delete the macro definition block (lines 377-393 including the comment header).
No callers exist so this is safe. No tests will be lost.

### Gap 3: Missing `translation_test!` — Forgejo `--author` → `--creator`

**File:** `tests/flag_audit.rs`
**Status:** Adapter code is correct; no test exercises the Forgejo author-remap path.

`src/adapter/issue.rs` lines 76-87 remaps `--author` to `--creator` for Forgejo:
```rust
ForgeType::Forgejo => {
    args.push("--creator".to_string());
    args.push(author.clone());
}
```

Existing `translation_test!` entries cover:
- `issue_list_github_state` (state, GitHub)
- `issue_list_glab_state_closed` (state, GitLab)
- `issue_list_tea_state` (state, Gitea)
- `issue_list_fj_state` (state, Forgejo)
- `issue_list_github_label` (label, GitHub)
- `issue_list_tea_label` (label, Gitea)
- `issue_list_fj_label` (label, Forgejo)

Missing: `issue_list_fj_author` — Forgejo `--author` → `--creator` remap.

**Required test:**
```rust
translation_test!(issue_list_fj_author,
    input: ["gf", "issue", "list", "--author", "alice"],
    forge: ForgeType::Forgejo,
    expected: ["issue", "search", "--creator", "alice"]
);
```

### Gap 4: Missing `audit_test!` entries for `fj issue search` flags

**File:** `tests/flag_audit.rs`
**Status:** Two audit tests are missing.

Existing issue audit tests (lines 944-953):
```
audit_v11_gh_issue_list_state
audit_v11_glab_issue_list_closed
audit_v11_tea_issues_list_state
audit_v11_fj_issue_search_state       ← exists
audit_v11_gh_issue_create_title
audit_v11_glab_issue_create_title
audit_v11_glab_issue_create_description
audit_v11_tea_issues_create_title
```

Missing:
- `audit_v11_fj_issue_search_labels`  — verifies `fj issue search --help` contains `--labels`
- `audit_v11_fj_issue_search_creator` — verifies `fj issue search --help` contains `--creator`

**Required additions:**
```rust
audit_test!(audit_v11_fj_issue_search_labels,  cli: "fj", args: ["issue", "search"], contains: "--labels");
audit_test!(audit_v11_fj_issue_search_creator, cli: "fj", args: ["issue", "search"], contains: "--creator");
```

### Gap 5: Second compiler warning — `LineRange` visibility mismatch

**File:** `src/browse/mod.rs` line 12
**Warning:** `pub fn build_file_url` takes `Option<&LineRange>` but `LineRange` is `pub(crate)`.

This is a separate warning from the dead-code one:
```
warning: type `LineRange` is more private than the item `build_file_url`
```

**Resolution:** Change `pub(crate) struct LineRange` to `pub struct LineRange` on line 12.
This matches the visibility of the function that exposes it.

## Architecture Patterns

No new patterns introduced. This phase follows existing patterns:

### Macro deletion pattern
Remove macro definition block plus its comment header. Verify with `grep -n "v11_translation_test"` that zero invocations remain.

### Adding `translation_test!` entries
Copy the pattern from an adjacent test (e.g., `issue_list_fj_state`) and adjust input/expected. The macro signature is:
```rust
translation_test!($name,
    input: ["gf", ...],
    forge: ForgeType::$Forge,
    expected: [...]
);
```

### Adding `audit_test!` entries
Copy adjacent `audit_test!` entry. The macro signature is:
```rust
audit_test!($name, cli: $cli, args: [$...], contains: $flag_str);
```
The macro runs `$cli $args --help` and asserts the output contains `$flag_str`.

### Deleting dead code
1. Remove the unused variant from `GfError` enum in `src/error.rs`.
2. Remove the unused function from `src/forge/mod.rs`.
3. Confirm `cargo build --release` shows zero warnings.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Verifying zero warnings | Custom script | `cargo build --release 2>&1 | grep warning` |
| Finding unused items | Manual grep | `cargo build --release` warning output |

## Common Pitfalls

### Pitfall 1: Deleting `SpawnFailed` by mistake
**What goes wrong:** The compiler warning lists both `SpawnFailed` and `CloneHostNotConfigured`
as never constructed. `SpawnFailed` IS used — in `src/runner.rs` line 45 (Windows path).
Only `CloneHostNotConfigured` should be deleted.
**How to avoid:** Check each variant individually before deleting.

### Pitfall 2: Deleting `get_default_clone_host` without checking callers
**What goes wrong:** The function is `pub` — external code (tests) could call it.
**How to avoid:** Run `grep -rn "get_default_clone_host"` across the whole tree before
deleting. Currently only `src/forge/mod.rs` defines it with no callers.

### Pitfall 3: `audit_test!` requires the forge CLI to be installed
**What goes wrong:** `audit_test!` actually invokes the CLI binary. If `fj` is not installed
in the test environment, the test fails with "not found" not a meaningful assertion failure.
**How to avoid:** These tests are already in the file and presumably passing. Adding two more
for the same CLI (`fj issue search`) poses no additional risk. Document that CI must have `fj`.

### Pitfall 4: `LineRange` visibility fix may expose the type in public API
**What goes wrong:** Changing `pub(crate)` to `pub` is a semver-breaking change for library
consumers — but `gf` is a binary crate, not a library, so this is safe.

## Code Examples

### Current warning output (from live `cargo build --release`)
```
warning: type `LineRange` is more private than the item `build_file_url`
  --> src/browse/mod.rs:124:1

warning: variants `SpawnFailed` and `CloneHostNotConfigured` are never constructed
  --> src/error.rs:17:5
```

### Expected result after phase
```
Finished `release` profile [optimized] target(s) in X.XXs
```
(Zero warning lines.)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo build --release` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| QUAL-03 | `gf issue list --author alice` (Forgejo) → `fj issue search --creator alice` | unit | `cargo test issue_list_fj_author` | ❌ Wave 0 |
| QUAL-02 | `fj issue search --help` contains `--labels` | audit | `cargo test audit_v11_fj_issue_search_labels` | ❌ Wave 0 |
| QUAL-02 | `fj issue search --help` contains `--creator` | audit | `cargo test audit_v11_fj_issue_search_creator` | ❌ Wave 0 |
| REPO-01 | `CloneHostNotConfigured` removed, zero warnings | build | `cargo build --release 2>&1 \| grep -c warning` = 0 | ✅ (existing build) |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo build --release`
- **Phase gate:** Zero warnings from `cargo build --release` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/flag_audit.rs` — add `translation_test!(issue_list_fj_author, ...)` (covers QUAL-03 / ISSUE-01)
- [ ] `tests/flag_audit.rs` — add `audit_test!(audit_v11_fj_issue_search_labels, ...)` (covers QUAL-02)
- [ ] `tests/flag_audit.rs` — add `audit_test!(audit_v11_fj_issue_search_creator, ...)` (covers QUAL-02)

## Open Questions

1. **Wire or delete `get_default_clone_host()`?**
   - What we know: Both resolutions satisfy the success criterion.
   - What's unclear: Whether future phases will want shorthand-with-explicit-host behavior.
   - Recommendation: Delete. The Phase 9 locked decision says the forge CLI handles host
     resolution for shorthand; the function adds complexity without current callers.

2. **`SpawnFailed` warning suppressible without deletion?**
   - What we know: It is used on Windows only; dead-code analysis fires on all platforms.
   - What's unclear: Whether `#[allow(dead_code)]` on the variant is acceptable.
   - Recommendation: Leave `SpawnFailed` as-is. The success criterion only requires zero
     warnings; if the compiler continues to warn about it after `CloneHostNotConfigured` is
     removed, add `#[cfg_attr(not(windows), allow(dead_code))]` to the variant.

## Sources

### Primary (HIGH confidence)
- Direct inspection of `src/error.rs`, `src/forge/mod.rs`, `src/adapter/repo_auth.rs`,
  `src/adapter/issue.rs`, `src/browse/mod.rs`, `tests/flag_audit.rs`
- Live `cargo build --release` output (2026-03-18)

### Metadata

**Confidence breakdown:**
- Gap inventory: HIGH — all gaps verified by code inspection + live compiler output
- Resolution approach: HIGH — standard Rust dead code removal, no ambiguity
- Test additions: HIGH — macro patterns already exist in the same file

**Research date:** 2026-03-18
**Valid until:** Until any of the listed files change
