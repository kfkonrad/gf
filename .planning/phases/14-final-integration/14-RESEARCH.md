# Phase 14: Final Integration and Polish - Research

**Researched:** 2026-03-19
**Domain:** Rust CLI integration testing, code restoration, documentation
**Confidence:** HIGH

## Summary

Phase 14 is the final gating phase for v1.2. It must prove all new commands work end-to-end, update documentation, and confirm zero warnings. However, **a critical regression was discovered during research**: Phase 13's commit (`b3df20c`) overwrote the `checks` and `comment` subcommands from Phases 11 and 12 in `src/cmd/mod.rs`, `src/adapter/pr.rs`, `src/adapter/issue.rs`, and `tests/flag_audit.rs`. The `pr checks`, `pr comment`, and `issue comment` commands are completely missing from the current codebase — they don't appear in clap definitions, have no adapter dispatch arms, have no translation functions, and their 22 tests are gone.

The current state is: 437 tests pass (all Phase 13 edit tests work), but the Phase 11/12 features (checks, comment) are not functional. The `gf pr --help` output shows no `checks` or `comment` subcommand. This phase must first restore the lost code, then add the integration tests.

**Primary recommendation:** Restore lost Phase 11/12 code (checks + comment) from git history, then add assert_cmd integration tests for all v1.2 commands, verify help text, update PROJECT.md, and confirm zero warnings.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Integration tests exercising full `gf <command>` → translate pipeline for all new commands
- Updated PROJECT.md with new command surface and test counts
- Verification that all existing + new tests pass
- Zero warnings confirmed
- Updated ROADMAP.md and STATE.md

### Claude's Discretion
None specified — CONTEXT.md defines scope tightly.

### Deferred Ideas (OUT OF SCOPE)
- Any new feature work — this is integration and documentation only
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PR-08 | PR CI status viewing (`gf pr checks`) | **CRITICAL BUG**: Code was lost in Phase 13 commit — must be restored from git history (commit `81d3248`). Clap subcommand, adapter dispatch arm, translate_pr_checks(), and 10 tests all need restoration. |
| PR-09 | Add/remove reviewers on PRs (`gf pr edit --add-reviewer`) | Working correctly — Phase 13 code is intact. Needs integration test via assert_cmd. |
| ISSUE-07 | Comment on issues (`gf issue comment`) | **CRITICAL BUG**: Code was lost in Phase 13 commit — must be restored from git history (commit `81d3248`). Clap subcommand, adapter dispatch arm, translate_issue_comment(), translate_pr_comment(), and 12 tests all need restoration. |
| ISSUE-08 | Assign/remove labels on issues (`gf issue edit --add-label`) | Working correctly — Phase 13 code is intact. Needs integration test via assert_cmd. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| assert_cmd | 2 | Integration testing of CLI binaries | Already in dev-dependencies; standard for Rust CLI testing |
| predicates | 3 | String/output assertion matchers | Already in dev-dependencies; pairs with assert_cmd |
| tempfile | 3 | Temporary directories for test repos | Already in dev-dependencies; used by existing tests |

### Supporting
No new dependencies needed. All tools are already in Cargo.toml.

## Architecture Patterns

### Critical: Code Restoration from Git History

The following code was lost when Phase 13 commit `b3df20c` overwrote files:

#### Files Affected
```
src/cmd/mod.rs          — Missing: checks + comment subcommands in build_pr() and comment in build_issue()
src/adapter/pr.rs       — Missing: translate_pr_checks() + translate_pr_comment() + dispatch arms
src/adapter/issue.rs    — Missing: translate_issue_comment() + dispatch arm
tests/flag_audit.rs     — Missing: 22 tests (10 checks + 12 comment)
```

#### What Must Be Restored

**1. `src/cmd/mod.rs` — Add back 3 missing clap subcommands:**

In `build_pr()`, after the `edit` subcommand (before the closing `}`):
- `checks` subcommand: `Command::new("checks")` with optional `number` and `extra` args
- `comment` subcommand: `Command::new("comment")` with optional `number`, `--body`, and `extra` args

In `build_issue()`, after the `edit` subcommand (before the closing `}`):
- `comment` subcommand: `Command::new("comment")` with required `number`, `--body`, and `extra` args

**2. `src/adapter/pr.rs` — Add back 2 functions + 2 dispatch arms:**

Dispatch arms to add in `translate_pr()` match (before the catch-all `Some((verb, sub))` arm):
```rust
Some(("checks", sub)) => translate_pr_checks(forge, pr_cmd, sub),
Some(("comment", sub)) => translate_pr_comment(forge, pr_cmd, sub),
```

Functions to restore:
- `translate_pr_checks()` — GitHub: passthrough, GitLab: `ci status` (bypasses pr_cmd), Forgejo: `pr status`, Gitea: UnsupportedFeature
- `translate_pr_comment()` — GitHub: passthrough, GitLab: `mr note` + `--message`, Forgejo: positional body, Gitea: UnsupportedFeature

**3. `src/adapter/issue.rs` — Add back 1 function + 1 dispatch arm:**

Dispatch arm to add in `translate_issue()` match (before the catch-all):
```rust
Some(("comment", sub)) => translate_issue_comment(forge, issue_cmd, sub),
```

Function to restore:
- `translate_issue_comment()` — GitHub: passthrough, GitLab: `issue note` + `--message`, Forgejo: positional body, Gitea: UnsupportedFeature

**4. `tests/flag_audit.rs` — Add back 22 tests:**

Lost tests by category:
- 6 PR checks translations: `pr_checks_github_number`, `pr_checks_github_no_number`, `pr_checks_glab_number`, `pr_checks_glab_no_number`, `pr_checks_fj_number`, `pr_checks_fj_no_number`
- 1 PR checks unsupported: `pr_checks_tea_unsupported`
- 3 PR checks audit: `audit_gh_pr_checks`, `audit_glab_ci_status`, `audit_fj_pr_status`
- 5 issue comment translations: `issue_comment_github`, `issue_comment_github_no_body`, `issue_comment_github_extra_passthrough`, `issue_comment_glab`, `issue_comment_fj`
- 1 issue comment unsupported: `issue_comment_tea_unsupported`
- 5 PR comment translations: `pr_comment_github`, `pr_comment_github_no_body`, `pr_comment_github_no_number`, `pr_comment_glab`, `pr_comment_fj`
- 1 PR comment unsupported: `pr_comment_tea_unsupported`

**Source of truth:** Git commit `81d3248` (Phase 12 GREEN commit) contains all the correct code.

### Pattern 1: Integration Tests with assert_cmd

The existing integration test pattern in `tests/integration_test.rs` is well-established:

**What:** Tests use `Command::cargo_bin("gf")` with PATH isolation (git-only bin dir) to test CLI behavior without actually calling forge CLIs.

**When to use:** For verifying help text output and UnsupportedFeature error handling (commands that fail before trying to exec a forge CLI).

**Example (from existing tests):**
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_some_command_help() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["pr", "checks", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("checks"));
}
```

### Pattern 2: Translation Tests via Macros

The project's declarative test macro pattern (`translation_test!`, `unsupported_test!`, `audit_test!`) is the standard for verifying command translations. These are MORE comprehensive than assert_cmd for translation correctness because they test the public API directly.

**The integration tests should focus on:**
1. Help text verification (subcommands appear in help output)
2. CLI-level UnsupportedFeature errors (Gitea PATH-isolated tests)
3. End-to-end clap parsing → translate pipeline (already covered by macro tests)

### Pattern 3: Help Text Verification

For verifying that help text includes new commands:
```rust
#[test]
fn test_pr_help_shows_checks() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("checks"));
}
```

### Anti-Patterns to Avoid
- **Testing live forge CLI calls:** Integration tests should NOT call actual `gh`/`glab`/etc. Use PATH isolation or test only help/parsing behavior.
- **Duplicating macro test coverage:** Don't re-test every translation via assert_cmd — the macro tests already cover that. Focus integration tests on CLI-level behavior that macro tests can't verify.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Restoring lost code | Manually rewriting from memory | `git show 81d3248:<file>` to extract exact code | The original code is preserved in git history — use it directly |
| Test infrastructure | New test helpers | Existing `setup_github_repo()`, `make_git_only_bin_dir()` helpers | Already battle-tested in integration_test.rs |

## Common Pitfalls

### Pitfall 1: Incomplete Code Restoration
**What goes wrong:** Restoring the clap subcommand but forgetting the dispatch arm in the adapter, or vice versa. This compiles but the command falls through to the catch-all passthrough.
**Why it happens:** Code is spread across 3 files (cmd/mod.rs, adapter/pr.rs, adapter/issue.rs).
**How to avoid:** Restore ALL 4 files in a single commit. Verify with `gf pr --help` showing `checks` and `comment`, AND `gf issue --help` showing `comment`.
**Warning signs:** Help output doesn't show expected subcommands; tests involving forge-specific translation fail.

### Pitfall 2: Merge Conflict with Existing Code
**What goes wrong:** Phase 13 restructured the file; pasting Phase 12 code into the wrong location.
**Why it happens:** The `build_pr()` and `build_issue()` functions had their subcommand chain reordered in Phase 13.
**How to avoid:** Add new subcommands AFTER the `edit` subcommand (which is the last one Phase 13 added). The dispatch arms in adapter functions go BEFORE the catch-all `Some((verb, sub))` arm.
**Warning signs:** Compilation errors from duplicate match arms or misplaced code.

### Pitfall 3: Test Count Regression
**What goes wrong:** After restoration, test count doesn't match expectations.
**Why it happens:** Lost tests need to be re-added to `tests/flag_audit.rs` alongside the existing Phase 13 tests.
**How to avoid:** After restoration, count should be 437 + 22 = 459 tests. Verify with `cargo test 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum}'`.
**Warning signs:** Test count doesn't reach 459.

### Pitfall 4: PROJECT.md Test Count Mismatch
**What goes wrong:** Updating PROJECT.md with wrong test count.
**Why it happens:** Multiple changes happening (restoration + new integration tests).
**How to avoid:** Update documentation AFTER all code changes, using actual `cargo test` output.

## Code Examples

### Exact Code to Restore for cmd/mod.rs (from git show 81d3248)

In `build_pr()`, after `.subcommand(Command::new("edit")...),`:
```rust
        .subcommand(
            Command::new("checks")
                .about("Show CI/check status for a pull request")
                .arg(
                    Arg::new("number")
                        .value_name("NUMBER")
                        .required(false)
                        .help("PR number (optional if on a PR branch)"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Additional flags passed through to the underlying CLI"),
                ),
        )
        .subcommand(
            Command::new("comment")
                .about("Add a comment to a pull request")
                .arg(
                    Arg::new("number")
                        .value_name("NUMBER")
                        .required(false)
                        .help("PR number (optional if on a PR branch)"),
                )
                .arg(
                    Arg::new("body")
                        .long("body")
                        .short('b')
                        .value_name("TEXT")
                        .help("Comment body text"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Additional flags passed through to the underlying CLI"),
                ),
        )
```

In `build_issue()`, after `.subcommand(Command::new("edit")...),`:
```rust
        .subcommand(
            Command::new("comment")
                .about("Add a comment to an issue")
                .arg(
                    Arg::new("number")
                        .value_name("NUMBER")
                        .required(true)
                        .help("Issue number"),
                )
                .arg(
                    Arg::new("body")
                        .long("body")
                        .short('b')
                        .value_name("TEXT")
                        .help("Comment body text"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Additional flags passed through to the underlying CLI"),
                ),
        )
```

### Exact Code to Restore for adapter/pr.rs (from git show 81d3248)

Dispatch arms (add before `Some((verb, sub)) =>`):
```rust
        Some(("checks", sub)) => translate_pr_checks(forge, pr_cmd, sub),
        Some(("comment", sub)) => translate_pr_comment(forge, pr_cmd, sub),
```

Translation functions (add at end of file, before any `#[cfg(test)]` block):
```rust
fn translate_pr_checks(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    let extra: Option<Vec<String>> = matches
        .get_many::<String>("extra")
        .map(|vals| vals.cloned().collect());

    match forge {
        ForgeType::Github => {
            let mut args = vec![pr_cmd.to_string(), "checks".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Gitlab => {
            let mut args = vec!["ci".to_string(), "status".to_string()];
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Forgejo => {
            let mut args = vec![pr_cmd.to_string(), "status".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Gitea => Err(GfError::UnsupportedFeature {
            feature: "pr checks".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        }),
    }
}

fn translate_pr_comment(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    if matches!(forge, ForgeType::Gitea) {
        return Err(GfError::UnsupportedFeature {
            feature: "pr comment".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }
    let number = matches.get_one::<String>("number");
    let body = matches.get_one::<String>("body");
    let mut args = vec![pr_cmd.to_string()];
    match forge {
        ForgeType::Gitlab => args.push("note".to_string()),
        _ => args.push("comment".to_string()),
    }
    if let Some(n) = number { args.push(n.clone()); }
    if let Some(b) = body {
        match forge {
            ForgeType::Gitlab => { args.push("--message".to_string()); args.push(b.clone()); }
            ForgeType::Forgejo => { args.push(b.clone()); }
            _ => { args.push("--body".to_string()); args.push(b.clone()); }
        }
    }
    if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
    Ok(args)
}
```

### Exact Code to Restore for adapter/issue.rs (from git show 81d3248)

Dispatch arm (add before `Some((verb, sub)) =>`):
```rust
        Some(("comment", sub)) => translate_issue_comment(forge, issue_cmd, sub),
```

Translation function (add at end of file):
```rust
fn translate_issue_comment(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    if matches!(forge, ForgeType::Gitea) {
        return Err(GfError::UnsupportedFeature {
            feature: "issue comment".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }
    let number = matches.get_one::<String>("number");
    let body = matches.get_one::<String>("body");
    let mut args = vec![issue_cmd.to_string()];
    match forge {
        ForgeType::Gitlab => args.push("note".to_string()),
        _ => args.push("comment".to_string()),
    }
    if let Some(n) = number { args.push(n.clone()); }
    if let Some(b) = body {
        match forge {
            ForgeType::Gitlab => { args.push("--message".to_string()); args.push(b.clone()); }
            ForgeType::Forgejo => { args.push(b.clone()); }
            _ => { args.push("--body".to_string()); args.push(b.clone()); }
        }
    }
    if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
    Ok(args)
}
```

### Integration Test Pattern for v1.2 Commands

```rust
// In tests/integration_test.rs

// Help text tests — verify all v1.2 commands appear
#[test]
fn test_pr_help_shows_checks() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("checks"));
}

#[test]
fn test_pr_help_shows_comment() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("comment"));
}

#[test]
fn test_pr_help_shows_edit() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edit"));
}

#[test]
fn test_issue_help_shows_comment() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["issue", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("comment"));
}

#[test]
fn test_issue_help_shows_edit() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["issue", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("edit"));
}

// UnsupportedFeature tests — Gitea with PATH isolation
#[test]
fn test_pr_checks_gitea_unsupported_error() {
    let repo = setup_gitea_repo();  // needs new helper
    let home_dir = tempfile::tempdir().unwrap();
    let (_bin_tmp, git_only) = make_git_only_bin_dir();
    // tea not on PATH → CliNotFound error, not UnsupportedFeature
    // Better: test via translation_test! macro (already covered)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual test functions | Declarative test macros (translation_test!, audit_test!, unsupported_test!) | Phase 7 (v1.1) | 200+ tests from ~200 lines |
| Integration tests for all translations | Macro tests for translations + assert_cmd for CLI-level behavior | Phase 8 (v1.1) | Clean separation of concerns |

## Current Codebase State

| Metric | Value |
|--------|-------|
| Total tests | 437 (97 lib + 97 bin + 218 flag_audit + 25 integration) |
| Source LOC | 3,859 |
| Compiler warnings | 0 (1 clippy `too_many_arguments` — pre-existing) |
| v1.2 commands working | 2/4 (`pr edit`, `issue edit`) |
| v1.2 commands BROKEN | 2/4 (`pr checks`, `pr comment`, `issue comment`) |
| Lost code commit | `81d3248` (contains all Phase 11+12 code) |
| Lost tests | 22 (10 checks + 12 comment) |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) + assert_cmd 2 |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PR-08 | PR checks translation | unit (macro) | `cargo test pr_checks -x` | ❌ LOST — needs restoration |
| PR-08 | PR checks help text | integration | `cargo test test_pr_help_shows_checks -x` | ❌ Wave 0 |
| PR-09 | PR edit translation | unit (macro) | `cargo test pr_edit -x` | ✅ exists |
| PR-09 | PR edit help text | integration | `cargo test test_pr_help_shows_edit -x` | ❌ Wave 0 |
| ISSUE-07 | Issue comment translation | unit (macro) | `cargo test issue_comment -x` | ❌ LOST — needs restoration |
| ISSUE-07 | Issue comment help text | integration | `cargo test test_issue_help_shows_comment -x` | ❌ Wave 0 |
| ISSUE-08 | Issue edit translation | unit (macro) | `cargo test issue_edit -x` | ✅ exists |
| ISSUE-08 | Issue edit help text | integration | `cargo test test_issue_help_shows_edit -x` | ❌ Wave 0 |
| ALL | Zero compiler warnings | build | `cargo build 2>&1 | grep -c warning` | ✅ currently 0 |
| ALL | Full test suite green | all | `cargo test` | ✅ 437 pass |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test 2>&1` (full output)
- **Phase gate:** Full suite green + `cargo build` zero warnings + help text verified

### Wave 0 Gaps
- [ ] Restore lost code from `81d3248` — covers PR-08 (checks) and ISSUE-07 (comment)
- [ ] Restore 22 lost tests from `81d3248` — in `tests/flag_audit.rs`
- [ ] Add help text integration tests — in `tests/integration_test.rs`
- [ ] Update PROJECT.md test counts and command surface

## Open Questions

1. **Expected final test count**
   - What we know: Currently 437 tests. Restoring 22 lost tests = 459. New integration tests TBD.
   - What's unclear: Exactly how many integration tests the phase should add.
   - Recommendation: Target ~10-15 integration tests (help text + a few end-to-end), landing at ~470-475 total.

2. **Clippy warning policy**
   - What we know: There's a pre-existing `clippy::too_many_arguments` warning in browse.rs. `cargo build` shows zero warnings.
   - What's unclear: Whether "zero warnings" includes clippy.
   - Recommendation: Success criteria says `cargo build` zero warnings — clippy warning is pre-existing and out of scope.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection — all files read and verified
- Git history (`git show 81d3248`) — verified exact code that was lost
- `cargo test` output — confirmed 437 tests, 0 failures
- `cargo build` output — confirmed 0 warnings
- `gf pr --help` / `gf issue --help` output — confirmed missing subcommands

### Secondary (MEDIUM confidence)
- Phase summaries (11-01-SUMMARY.md, 12-01-SUMMARY.md, 13-01-SUMMARY.md) — cross-referenced with actual code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries needed, all existing
- Architecture: HIGH — existing patterns well-documented in codebase
- Pitfalls: HIGH — critical regression discovered and fully documented
- Code restoration: HIGH — exact code preserved in git history

**Research date:** 2026-03-19
**Valid until:** 2026-04-19 (stable codebase, no external dependencies changing)
