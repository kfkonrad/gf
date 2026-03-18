# Phase 8: PR Workflow Commands - Research

**Researched:** 2026-03-17
**Domain:** CLI adapter translation, forge-specific PR/MR lifecycle commands, config schema extension
**Confidence:** HIGH

## Summary

Phase 8 implements the complete PR/MR lifecycle (`list`, `merge`, `checkout`, `review`, `approve`) and extends `browse` with `--pr`/`--mr`/`--issue` flags. The codebase already has 45 pre-mapped translation tests (all `#[ignore]`d) from Phase 7 that define the exact expected translations. The implementation follows the well-established adapter pattern: add clap subcommands, write translation functions, un-ignore tests.

The most significant architectural change is that `adapter::translate()` must change from `Vec<String>` to `Result<Vec<String>, GfError>` to support the new **hard error policy** for unsupported commands/flags. This is a cross-cutting change that affects main.rs, all translation functions, and both test macros. This change should be the first task since everything else depends on it. A secondary complexity point is the config schema extension for `--delete-branch` defaults, which adds per-forge configuration to the existing `~/.config/gf/config.toml`.

**Primary recommendation:** Start with the `translate()` → `Result` signature change, then implement subcommands in batches (list+checkout together as simpler; merge as standalone due to config complexity; review+approve together due to subcommand routing complexity; browse --pr/--issue last as independent module).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Hard error for unsupported commands/flags**: Exit non-zero + stderr message for ANY unsupported command or flag translation. Applies to both entire commands (`gf pr review` on tea) AND individual flags (`--author` on tea pr list). Error message format: clear, compile-error style — state what's unsupported and on which forge.
- **This is a NEW policy for Phase 8** — Phase 7's silent-omit convention is NOT followed. Retroactive fix of Phase 7's silent omissions deferred to a separate phase.
- **Merge strategy defaults**: When no `--squash`/`--rebase`/`--merge` flag is given, gf **explicitly passes** the merge flag to each forge (GitHub: `--merge`, GitLab: default/no flag, Gitea: `--style merge`, Forgejo: `--method merge`). Explicit > implicit.
- **Delete-branch behavior**: `--delete-branch` and `--no-delete-branch` flags on `gf pr merge`. Default: **no delete** (conservative). Per-forge default configurable in `~/.config/gf/config.toml`. Flag on command line overrides config.
- **Browse --pr / --mr / --issue**: `--mr` alias for `--pr`. `--issue` for ISSUE-06 (pulled from Phase 9). Mutually exclusive with `--branch` and file arguments — hard error if combined.
- **PR URL patterns**: GitHub `pull/N`, GitLab `-/merge_requests/N`, Gitea `pulls/N`, Forgejo `pulls/N`
- **Issue URL patterns**: GitHub `issues/N`, GitLab `-/issues/N`, Gitea `issues/N`, Forgejo `issues/N`

### Claude's Discretion
- How to structure the new clap subcommands (list, merge, checkout, review, approve)
- Config file schema design for delete-branch defaults
- How to integrate the config reading into the merge translation path
- Error message wording and format for unsupported commands/flags
- Whether to add new GfError variants or reuse existing ones
- Plan count and task breakdown

### Deferred Ideas (OUT OF SCOPE)
- Retroactive silent-omit → hard-error migration for Phase 7's existing silent flag omissions
- Per-forge config defaults for other settings beyond delete-branch
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PR-01 | User can list PRs/MRs with filter flags (state, author, label) | Pre-mapped tests exist (v11_pr_list_*). Translation patterns verified against all 4 CLIs. tea lacks --author/--label → UNSUPPORTED hard error. fj uses `pr search` not `pr list`, --creator not --author, --labels not --label. glab uses boolean flags (--closed/--merged/--all) not --state value. |
| PR-02 | User can merge a PR/MR with strategy flags (squash, rebase, merge) | Pre-mapped tests exist (v11_pr_merge_*). tea uses `--style`, fj uses `--method`. gh/glab use `--squash`/`--rebase`. Config schema needed for delete-branch defaults. Delete-branch flag: gh `--delete-branch`, glab `--remove-source-branch`, fj `--delete`, tea UNSUPPORTED. |
| PR-03 | User can checkout a PR/MR branch locally | Pre-mapped tests exist (v11_pr_checkout_*). Simple positional translation. All 4 forges support checkout. |
| PR-04 | User can review a PR/MR (comment) | Pre-mapped tests exist (v11_pr_review_comment_*). gh uses `pr review --comment --body`, glab uses `mr note --message` (subcommand remap), fj uses `pr comment <body>` (positional). tea UNSUPPORTED for non-interactive review. |
| PR-05 | User can approve a PR/MR | Pre-mapped tests exist (v11_pr_review_approve_*). gh uses `pr review --approve`, glab uses `mr approve` (subcommand remap). tea UNSUPPORTED, fj UNSUPPORTED. |
| PR-06 | User can view a specific PR/MR by number | Already implemented in Phase 3 (`translate_pr_view`). Tests passing. No additional work needed. |
| PR-07 | User can browse a PR/MR in browser (`gf browse --pr 123`) | New flags on browse subcommand. URL pattern builders following existing `build_repo_url`/`build_file_url` pattern. Also includes ISSUE-06 (`--issue`) pulled from Phase 9. |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4 | CLI command tree + arg parsing | Already used; extend with new subcommands |
| toml + serde | 0.8 / 1 | Config file deserialization | Already used for forge config; extend schema |
| thiserror | 2 | Error type derive macros | Already used for GfError |
| webbrowser | 1 | Open URLs in browser | Already used in browse module |

### No New Dependencies
Phase 8 requires zero new crate dependencies. All work is extending existing adapter/cmd/browse/forge/error modules with the same patterns used since Phase 1.

## Architecture Patterns

### Recommended Project Structure (changes only)
```
src/
├── adapter/
│   ├── mod.rs           # translate() signature: Vec<String> → Result<Vec<String>, GfError>
│   ├── pr.rs            # NEW match arms: list, merge, checkout, review, approve
│   └── repo_auth.rs     # Unchanged
├── browse/
│   └── mod.rs           # NEW: --pr/--mr/--issue flags, build_pr_url(), build_issue_url()
├── cmd/
│   └── mod.rs           # NEW subcommands: list, merge, checkout, review, approve on build_pr()
├── error.rs             # NEW variant: UnsupportedFeature { feature, forge }
└── forge/
    └── mod.rs           # Extended GfConfig/ForgeEntry with merge.delete_branch
tests/
├── flag_audit.rs        # Remove #[ignore] from v11 tests; add unsupported-error tests
└── integration_test.rs  # Add unsupported-error integration test
```

### Pattern 1: translate() Signature Change to Result
**What:** Change `adapter::translate()` and all sub-translators from `Vec<String>` to `Result<Vec<String>, GfError>`.
**When to use:** Must happen before any unsupported-error logic can be implemented.
**Impact scope:** `adapter/mod.rs`, `adapter/pr.rs`, `adapter/repo_auth.rs`, `src/main.rs`, `tests/flag_audit.rs` (both macros).

Current:
```rust
// src/adapter/mod.rs
pub fn translate(forge: ForgeType, matches: &ArgMatches) -> Vec<String> { ... }
// src/main.rs  
let translated: Vec<String> = adapter::translate(forge_type, &matches);
```

After:
```rust
// src/adapter/mod.rs
pub fn translate(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> { ... }
// src/main.rs
let translated = match adapter::translate(forge_type, &matches) {
    Ok(args) => args,
    Err(e) => {
        eprintln!("{e}");
        std::process::exit(1);
    }
};
```

All sub-translators cascade: `translate_pr()`, `translate_pr_list()`, etc. return `Result<Vec<String>, GfError>`. `translate_repo()` and `translate_auth()` also change signature for consistency (wrap returns in `Ok()`).

Test macros must also change:
```rust
macro_rules! translation_test {
    ($name:ident, input: [...], forge: $forge:expr, expected: [...]) => {
        #[test]
        fn $name() {
            let matches = gf::cmd::build_cli()
                .try_get_matches_from([...])
                .unwrap_or_else(|e| panic!("clap parse failed: {e}"));
            let result = gf::adapter::translate($forge, &matches)
                .unwrap_or_else(|e| panic!("translate returned error: {e}"));
            // ... rest unchanged
        }
    };
}
```

### Pattern 2: UnsupportedFeature Error Variant
**What:** New GfError variant for unsupported forge commands/flags.
**Why:** The hard-error policy requires clear, actionable error messages.

```rust
// src/error.rs
#[derive(Debug, Error)]
pub enum GfError {
    // ... existing variants ...
    
    #[error("`gf {feature}` is not supported on {forge}\n\n{forge_cli} does not have an equivalent for this command/flag.")]
    UnsupportedFeature {
        feature: String,
        forge: String,
        forge_cli: String,
    },
}
```

Usage in adapter:
```rust
// src/adapter/pr.rs
fn translate_pr_list(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    // ...
    if matches.get_one::<String>("author").is_some() {
        match forge {
            ForgeType::Gitea => return Err(GfError::UnsupportedFeature {
                feature: "pr list --author".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
            // ...
        }
    }
    Ok(args)
}
```

### Pattern 3: Subcommand Routing for glab mr approve/comment
**What:** When the canonical `gf pr review --approve` maps to `glab mr approve` (separate subcommand, not a flag), the translator must remap the verb entirely.
**Why:** glab exposes approve and comment as sibling subcommands of `mr`, not flags on `mr review`.

```rust
fn translate_pr_review(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    
    if matches.get_flag("approve") {
        match forge {
            ForgeType::Github => {
                let mut args = vec![pr_cmd.to_string(), "review".to_string()];
                if let Some(n) = number { args.push(n.clone()); }
                args.push("--approve".to_string());
                Ok(args)
            }
            ForgeType::Gitlab => {
                // glab mr approve <N> — subcommand remap
                let mut args = vec![pr_cmd.to_string(), "approve".to_string()];
                if let Some(n) = number { args.push(n.clone()); }
                Ok(args)
            }
            ForgeType::Gitea | ForgeType::Forgejo => {
                Err(GfError::UnsupportedFeature { ... })
            }
        }
    } else if matches.get_flag("comment") {
        // comment path...
    }
}
```

### Pattern 4: Config-Aware Translation (delete-branch)
**What:** The merge translator reads config to determine delete-branch default, then overrides with CLI flag if present.
**Config schema:**
```toml
# ~/.config/gf/config.toml
[merge]
delete_branch = false   # global default (overrides gf's built-in "no delete")

[[forge]]
domain = "github.com"
type = "github"
delete_branch = true    # per-forge override
```

**Rust struct extension:**
```rust
#[derive(Debug, Deserialize)]
struct GfConfig {
    #[serde(default)]
    forge: Vec<ForgeEntry>,
    #[serde(default)]
    merge: MergeConfig,
}

#[derive(Debug, Deserialize, Default)]
struct MergeConfig {
    #[serde(default)]
    delete_branch: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ForgeEntry {
    domain: String,
    #[serde(rename = "type")]
    forge_type: ForgeType,
    #[serde(default)]
    delete_branch: Option<bool>,
}
```

**Resolution order:** CLI flag > per-forge config > global merge config > built-in default (false).

The config needs to be read during merge translation, which means `translate_pr_merge()` needs access to config. Two clean approaches:
1. Load config in `translate_pr_merge()` directly via `forge::load_config()` (make it pub)
2. Pass config as parameter from main.rs through translate chain

Recommendation: Option 1 — load config directly in `translate_pr_merge()`. The config loading is already fast (file read + TOML parse, cached by OS), and threading config through the entire translate chain would require changing every function signature for a feature only merge uses.

### Pattern 5: Browse URL Builders for PR/Issue
**What:** Two new URL builder functions following the existing `build_repo_url`/`build_file_url` pattern.

```rust
// src/browse/mod.rs
pub fn build_pr_url(forge: &ForgeType, host: &str, owner: &str, repo: &str, number: u32) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github => format!("{base}/pull/{number}"),
        ForgeType::Gitlab => format!("{base}/-/merge_requests/{number}"),
        ForgeType::Gitea | ForgeType::Forgejo => format!("{base}/pulls/{number}"),
    }
}

pub fn build_issue_url(forge: &ForgeType, host: &str, owner: &str, repo: &str, number: u32) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github => format!("{base}/issues/{number}"),
        ForgeType::Gitlab => format!("{base}/-/issues/{number}"),
        ForgeType::Gitea | ForgeType::Forgejo => format!("{base}/issues/{number}"),
    }
}
```

The `browse::run()` function gets a new early-return path:
```rust
pub fn run(matches: &ArgMatches) -> Result<(), GfError> {
    // Handle --pr/--mr/--issue before file/repo path
    if let Some(pr_num) = matches.get_one::<String>("pr") { ... }
    if let Some(issue_num) = matches.get_one::<String>("issue") { ... }
    // ... existing file/repo path ...
}
```

### Anti-Patterns to Avoid
- **Don't duplicate forge detection in browse for --pr/--issue:** Reuse existing `resolve_forge_type()` + `parse_remote_parts()` already in browse.
- **Don't add config threading through all translate functions:** Only merge needs config; load it locally.
- **Don't mix Phase 7 silent-omit with Phase 8 hard-error:** Phase 8 functions return `Err()` for unsupported; do not touch Phase 7 functions (deferred).
- **Don't use process::exit() in adapter functions:** Return `Result` and let main.rs handle exit. This preserves testability.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config file parsing | Manual TOML parser | `toml` + `serde` crate (existing) | Already in use; extend structs |
| CLI arg parsing | Manual arg matching | `clap` Command tree (existing) | Already in use; add subcommands |
| Error formatting | Manual eprintln!() | `thiserror` derive (existing) | Consistent error format across all errors |
| Mutual exclusivity of browse flags | Manual if/else chains | `clap::ArgGroup` or `.conflicts_with()` | clap handles conflict errors with good messages |

## Common Pitfalls

### Pitfall 1: glab State Uses Boolean Flags, Not --state Value
**What goes wrong:** Translating `--state closed` directly as `--state closed` for glab fails because glab mr list uses `--closed`, `--merged`, `--all` as standalone boolean flags, not `--state <value>`.
**Why it happens:** glab diverges from the gh/tea/fj pattern.
**How to avoid:** Match on the state value and produce the right boolean flag:
```rust
match state.as_str() {
    "closed" => args.push("--closed".to_string()),
    "merged" => args.push("--merged".to_string()),
    "all" => args.push("--all".to_string()),
    "open" => {} // glab default, no flag needed
    _ => {} // passthrough or error
}
```
**Warning signs:** Pre-mapped tests `v11_pr_list_glab_state_*` catch this.

### Pitfall 2: glab mr approve Is a Subcommand, Not a Flag
**What goes wrong:** Trying to pass `--approve` to `glab mr review` — glab has no `mr review` subcommand.
**Why it happens:** gh bundles review/approve/comment under `pr review --flag`; glab splits them into separate subcommands (`mr approve`, `mr note`).
**How to avoid:** The translator for `gf pr review --approve` must remap to `glab mr approve <N>` (verb change, flag removal). Similarly, `gf pr review --comment --body "text"` → `glab mr note <N> --message "text"`.
**Warning signs:** Pre-mapped tests `v11_pr_review_approve_glab` and `v11_pr_review_comment_glab` verify this.

### Pitfall 3: fj Uses `pr search` Not `pr list`
**What goes wrong:** Translating `gf pr list` as `fj pr list` — fj has no `pr list` subcommand.
**Why it happens:** fj organizes list-like operations under `search`.
**How to avoid:** Map the verb: `list` → `search` for Forgejo. Also map `--author` → `--creator` and `--label` → `--labels`.
**Warning signs:** Pre-mapped tests `v11_pr_list_fj_*` verify this.

### Pitfall 4: tea Lacks --author and --label on pulls list
**What goes wrong:** Passing `--author alice` or `--label bug` to `tea pulls list` — these flags don't exist.
**Why it happens:** tea has a simpler pull request listing with fewer filter options.
**How to avoid:** Return `Err(GfError::UnsupportedFeature {...})` per the hard-error policy.
**Warning signs:** No v11 pre-mapped tests for these (intentionally absent = UNSUPPORTED).

### Pitfall 5: tea Has No --delete-branch on pulls merge
**What goes wrong:** Passing `--delete-branch` equivalent to `tea pulls merge` — tea lacks this flag.
**How to avoid:** Return `Err(GfError::UnsupportedFeature {...})` when `--delete-branch` is used with tea.
**Verified:** `tea pulls merge --help` shows only `--style`, `--title`, `--message`, `--repo`, `--remote`, `--login`, `--output`.

### Pitfall 6: glab Has No --merge Flag
**What goes wrong:** Trying to explicitly pass `--merge` to `glab mr merge` — glab has no `--merge` flag; merge is the default behavior (no flag needed).
**How to avoid:** When merge strategy is "merge" on glab, emit no strategy flag. The pre-mapped test `v11_pr_merge_glab_merge_default` expects just `["mr", "merge"]` with no `--merge`.
**Verified:** `glab mr merge --help` only shows `--squash` and `--rebase`, no `--merge`.

### Pitfall 7: translate() Return Type Must Change Before Unsupported Errors
**What goes wrong:** Trying to return errors from translate functions that return `Vec<String>`.
**Why it happens:** The function signature doesn't support error returns.
**How to avoid:** Change the signature to `Result<Vec<String>, GfError>` as the FIRST task. This change cascades to all sub-translators, both test macros, and main.rs.

### Pitfall 8: Browse --pr/--mr/--issue Don't Need Git Ref
**What goes wrong:** The browse module resolves the current branch/SHA for every URL. PR/issue URLs don't use git refs.
**How to avoid:** Short-circuit before ref resolution when --pr or --issue is set. Only need host/owner/repo + number.

## Code Examples

### Complete Forge Flag Translation Map (verified via CLI --help)

#### pr list
| Canonical | gh | glab | tea | fj |
|-----------|-----|------|-----|-----|
| verb: `list` | `pr list` | `mr list` | `pulls list` | `pr search` |
| `--state open` | `--state open` | *(default)* | `--state open` | `--state open` |
| `--state closed` | `--state closed` | `--closed` | `--state closed` | `--state closed` |
| `--state merged` | `--state merged` | `--merged` | *(N/A — tea uses closed)* | *(N/A)* |
| `--state all` | `--state all` | `--all` | `--state all` | `--state all` |
| `--author alice` | `--author alice` | `--author alice` | **UNSUPPORTED** | `--creator alice` |
| `--label bug` | `--label bug` | `--label bug` | **UNSUPPORTED** | `--labels bug` |

#### pr merge
| Canonical | gh | glab | tea | fj |
|-----------|-----|------|-----|-----|
| verb: `merge` | `pr merge` | `mr merge` | `pulls merge` | `pr merge` |
| `--squash` | `--squash` | `--squash` | `--style squash` | `--method squash` |
| `--rebase` | `--rebase` | `--rebase` | `--style rebase` | `--method rebase` |
| `--merge` (or default) | `--merge` | *(no flag)* | `--style merge` | `--method merge` |
| `--delete-branch` | `--delete-branch` | `--remove-source-branch` | **UNSUPPORTED** | `--delete` |
| `--no-delete-branch` | *(omit flag)* | *(omit flag)* | *(N/A)* | *(omit flag)* |
| `<number>` | positional | positional | positional | positional |

#### pr checkout
| Canonical | gh | glab | tea | fj |
|-----------|-----|------|-----|-----|
| verb: `checkout` | `pr checkout` | `mr checkout` | `pulls checkout` | `pr checkout` |
| `<number>` | positional | positional | positional | positional |

#### pr review (comment)
| Canonical | gh | glab | tea | fj |
|-----------|-----|------|-----|-----|
| verb: `review --comment` | `pr review --comment --body` | `mr note --message` | **UNSUPPORTED** | `pr comment <body>` |

#### pr review (approve) / pr approve
| Canonical | gh | glab | tea | fj |
|-----------|-----|------|-----|-----|
| `review --approve` | `pr review --approve` | `mr approve` | **UNSUPPORTED** | **UNSUPPORTED** |

#### pr view (PR-06 — already implemented)
Already implemented in Phase 3 `translate_pr_view()`. Tests pass. No work needed.

### Delete-Branch Flag Translation Per Forge (verified via --help)
```
gh pr merge:   -d, --delete-branch           Delete the local and remote branch after merge
glab mr merge: -d --remove-source-branch     Remove source branch on merge.
fj pr merge:   -d, --delete                  Option to delete the corresponding branch afterwards
tea pulls merge: (NO DELETE FLAG)             UNSUPPORTED → hard error
```

### Config Schema Design
```toml
# ~/.config/gf/config.toml

# Global merge defaults
[merge]
delete_branch = false    # default: don't delete

# Per-forge entries (existing format, extended with optional merge settings)
[[forge]]
domain = "github.com"
type = "github"
delete_branch = true     # override for this forge

[[forge]]
domain = "gitlab.mycompany.com"
type = "gitlab"
# no delete_branch = inherits from [merge] section, or built-in default
```

### gf pr approve as Separate Subcommand
The CONTEXT mentions `approve` as both a flag on `review` (`--approve`) and a separate subcommand. Recommendation: implement `gf pr approve <number>` as syntactic sugar that translates identically to `gf pr review <number> --approve`. This gives users two ergonomic options:
```
gf pr review 42 --approve    # flag form
gf pr approve 42             # subcommand form (sugar)
```
Both map to the same translation output.

## State of the Art

| Old Approach (Phase 7) | Current Approach (Phase 8) | Impact |
|------------------------|---------------------------|--------|
| Silent omit for unsupported flags | Hard error (exit non-zero + stderr) | `translate()` return type changes to `Result` |
| `Vec<String>` return type | `Result<Vec<String>, GfError>` | Cascading signature change |
| No config for merge behavior | Config-aware merge defaults | Config schema extension |
| Browse: files and repos only | Browse: + PR URLs + issue URLs | New URL builders |

## Open Questions

1. **tea pulls review/approve exist but are treated as UNSUPPORTED**
   - What we know: `tea pulls review` exists but is interactive (opens diff viewer). `tea pulls approve` exists and could map from `gf pr approve <N>`.
   - What's decided: CONTEXT.md locks the decision to treat both as UNSUPPORTED on tea. The canonical `--comment --body` / `--approve` patterns don't cleanly map to tea's interface.
   - Recommendation: Follow locked decision. Future phases can revisit.

2. **glab mr note vs glab mr comment**
   - What we know: `glab mr note` is the canonical subcommand name; `comment` appears to be an alias. The pre-mapped test expects `["mr", "comment", ...]`.
   - Recommendation: Use `note` as the canonical output (it's what glab --help shows as the primary name). Update the pre-mapped test expectation if needed. **LOW confidence on this — needs verification against actual glab behavior.**

3. **PR number as optional vs required per subcommand**
   - What we know: `merge`, `checkout`, `review`, `approve` typically need a PR number but all forge CLIs also support inferring from current branch.
   - Recommendation: Make number optional in clap (like existing `pr view`). Pass through if present; let the forge CLI handle inference if absent.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) + assert_cmd 2 |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --test flag_audit` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PR-01 | PR list with --state/--author/--label filters | unit | `cargo test v11_pr_list -- --include-ignored` | ✅ (ignored in flag_audit.rs) |
| PR-01 | Unsupported --author/--label on tea → error | unit | `cargo test pr_list_tea_author_unsupported` | ❌ Wave 0 |
| PR-02 | PR merge with --squash/--rebase/--merge | unit | `cargo test v11_pr_merge -- --include-ignored` | ✅ (ignored in flag_audit.rs) |
| PR-02 | Delete-branch flag translation | unit | `cargo test pr_merge_delete_branch` | ❌ Wave 0 |
| PR-02 | Merge default explicit strategy | unit | `cargo test pr_merge_default_strategy` | ❌ Wave 0 |
| PR-03 | PR checkout with number | unit | `cargo test v11_pr_checkout -- --include-ignored` | ✅ (ignored in flag_audit.rs) |
| PR-04 | PR review --comment --body | unit | `cargo test v11_pr_review_comment -- --include-ignored` | ✅ (ignored in flag_audit.rs) |
| PR-04 | Review unsupported on tea → error | unit | `cargo test pr_review_tea_unsupported` | ❌ Wave 0 |
| PR-05 | PR review --approve | unit | `cargo test v11_pr_review_approve -- --include-ignored` | ✅ (ignored in flag_audit.rs) |
| PR-05 | Approve unsupported on tea/fj → error | unit | `cargo test pr_approve_tea_unsupported` | ❌ Wave 0 |
| PR-06 | PR view by number | unit | `cargo test pr_view` | ✅ Already passing |
| PR-07 | Browse --pr builds correct URL | unit | `cargo test browse_pr_url` | ❌ Wave 0 |
| PR-07 | Browse --issue builds correct URL | unit | `cargo test browse_issue_url` | ❌ Wave 0 |
| PR-07 | Browse --pr conflicts with file/branch | unit | `cargo test browse_pr_conflicts` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --test flag_audit && cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/flag_audit.rs` — Add unsupported-error test macro and test cases for tea/fj unsupported combinations
- [ ] `tests/flag_audit.rs` — Add delete-branch translation tests (4 forges × 2 flag values)
- [ ] `tests/flag_audit.rs` — Add default merge strategy tests (4 forges)
- [ ] `src/browse/mod.rs` (unit tests section) — Add `build_pr_url` and `build_issue_url` unit tests
- [ ] `tests/integration_test.rs` — Add integration test for unsupported error output format
- [ ] Update `translation_test!` and `v11_translation_test!` macros to handle `Result` return type

## Sources

### Primary (HIGH confidence)
- `gh pr merge --help` — verified: --delete-branch, --squash, --rebase, --merge flags
- `glab mr merge --help` — verified: --remove-source-branch, --squash, --rebase (NO --merge)
- `tea pulls merge --help` — verified: --style (NO delete-branch flag)
- `fj pr merge --help` — verified: --method, --delete
- `gh pr list --help` — verified: --state, --author, --label
- `glab mr list --help` — verified: --closed, --merged, --all, --author, --label
- `tea pulls list --help` — verified: --state (NO --author, NO --label)
- `fj pr search --help` — verified: --state, --creator, --labels
- `gh pr review --help` — verified: --approve, --comment, --body
- `glab mr note --help` — verified: --message
- `glab mr approve --help` — verified: exists as separate subcommand
- `tea pulls approve --help` — verified: exists but treated as UNSUPPORTED per locked decision
- `fj pr comment --help` — verified: positional body argument
- Codebase analysis: `src/adapter/mod.rs`, `src/adapter/pr.rs`, `src/cmd/mod.rs`, `src/browse/mod.rs`, `src/forge/mod.rs`, `src/error.rs`, `tests/flag_audit.rs`

### Secondary (MEDIUM confidence)
- Pre-mapped test expectations in `tests/flag_audit.rs` (v11_translation_test! entries) — authored during Phase 7 research, cross-verified with --help

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, extending existing patterns
- Architecture: HIGH — adapter pattern well-established across 7 prior phases
- Flag translations: HIGH — every flag verified against actual forge CLI --help output
- Config schema: MEDIUM — design is sound but untested; no prior per-forge config beyond domain/type
- Pitfalls: HIGH — pre-mapped tests catch most issues; unsupported combinations verified

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable — forge CLIs change infrequently)
