# Phase 13: PR and Issue Edit - Research

**Researched:** 2026-03-19
**Domain:** CLI adapter translation — edit/update commands across 4 forge CLIs
**Confidence:** HIGH

## Summary

Phase 13 adds `gf pr edit` and `gf issue edit` commands for adding/removing labels, reviewers, and assignees. This is the most structurally divergent translation in the project: GitHub uses `--add-*/--remove-*` flags, GitLab uses `update` verb with `--label`/`--unlabel` and prefix semantics (`+`/`-`) for reviewers/assignees, Forgejo uses subcommand routing (`fj pr edit <N> labels --add`), and Gitea has no PR edit at all but has issue edit with plural flag names.

The core challenge is that **each forge has a different support matrix** — some flags work on some forges but not others. The adapter must check ALL flags before exec and return `UnsupportedFeature` for any that can't map, ensuring no partial execution.

**Primary recommendation:** Build `translate_pr_edit()` and `translate_issue_edit()` functions using a "validate-then-build" pattern: first scan all flags for unsupported combinations, then construct the forge-specific command. Follow existing patterns from `translate_pr_checks()` and `translate_pr_comment()`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- `gf pr edit [<number>]` with `--add-label`, `--remove-label`, `--add-reviewer`, `--remove-reviewer`, `--add-assignee`, `--remove-assignee`
- `gf issue edit <number>` with `--add-label`, `--remove-label`, `--add-assignee`, `--remove-assignee`
- GitHub: `gh pr edit` / `gh issue edit` — direct flag mapping
- GitLab: `glab mr update` / `glab issue update` — remap to `--label`/`--unlabel`, `--reviewer`/prefix, `--assignee`/prefix
- Forgejo PR: `fj pr edit <N> labels --add/--rm` for labels; UnsupportedFeature for reviewer/assignee
- Forgejo Issue: UnsupportedFeature for labels and assignees
- Gitea PR: UnsupportedFeature for entire `gf pr edit` (tea has no pulls edit)
- Gitea Issue: `tea issues edit --add-labels/--remove-labels/--add-assignees` — map supported flags
- Per-flag UnsupportedFeature errors
- Per-flag error granularity: adapter should error before exec if any flag in the invocation can't map — don't partially execute

### Locked Out of Scope
- `--title`, `--body`, `--milestone`, `--project` editing
- `--draft` / `--ready` PR state changes
- Editing PR/issue descriptions or comments
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PR-09 | Add/remove reviewers on PRs (`gf pr edit --add-reviewer`) | Full CLI verification for gh (direct), glab (prefix semantics), fj (UnsupportedFeature), tea (UnsupportedFeature). Also covers labels and assignees on PR edit. |
| ISSUE-08 | Assign/remove labels on issues (`gf issue edit --add-label`) | Full CLI verification for gh (direct), glab (label/unlabel), fj (UnsupportedFeature), tea (add-labels/remove-labels with plural rename). Also covers assignees on issue edit. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.x | CLI argument parsing | Already in use — `build_cli()` builder API for subcommand definitions |
| thiserror | 1.x/2.x | Error types | Already in use — `GfError::UnsupportedFeature` |

No new dependencies needed. This phase only adds new `edit` subcommands and translation functions to existing modules.

## Architecture Patterns

### Recommended Project Structure (changes only)
```
src/
├── adapter/
│   ├── pr.rs         # + translate_pr_edit() function
│   └── issue.rs      # + translate_issue_edit() function
├── cmd/
│   └── mod.rs        # + edit subcommands in build_pr() and build_issue()
tests/
└── flag_audit.rs     # + translation_test!, unsupported_test!, audit_test! entries
```

### Pattern 1: Validate-Then-Build for Per-Flag Errors
**What:** Before constructing forge-specific args, scan ALL provided flags and collect any unsupported ones. Error immediately if any flag can't map.
**When to use:** When multiple independent flags may each have different support levels per forge.
**Why:** The constraint says "adapter should error before exec if any flag in the invocation can't map — don't partially execute."
**Example:**
```rust
fn translate_pr_edit(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    let add_label = matches.get_one::<String>("add-label");
    let remove_label = matches.get_one::<String>("remove-label");
    let add_reviewer = matches.get_one::<String>("add-reviewer");
    let remove_reviewer = matches.get_one::<String>("remove-reviewer");
    let add_assignee = matches.get_one::<String>("add-assignee");
    let remove_assignee = matches.get_one::<String>("remove-assignee");

    // VALIDATE FIRST: check every flag against forge capabilities
    // Return UnsupportedFeature for the FIRST unsupported flag found
    match forge {
        ForgeType::Gitea => {
            // tea has no pulls edit at all
            return Err(GfError::UnsupportedFeature {
                feature: "pr edit".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            });
        }
        ForgeType::Forgejo => {
            // fj pr edit only supports labels subcommand
            if add_reviewer.is_some() {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr edit --add-reviewer".to_string(),
                    forge: "Forgejo".to_string(),
                    forge_cli: "fj".to_string(),
                });
            }
            // ... check remove_reviewer, add_assignee, remove_assignee too
        }
        _ => {} // Github, Gitlab support all flags
    }

    // BUILD: construct forge-specific command
    // ...
}
```

### Pattern 2: Verb Remap (glab update instead of edit)
**What:** GitLab uses `update` verb where gf uses `edit`.
**When to use:** `gf pr edit` → `glab mr update`, `gf issue edit` → `glab issue update`
**Example:**
```rust
// In translate_pr_edit for GitLab:
let mut args = vec![pr_cmd.to_string(), "update".to_string()]; // "mr" "update"
// In translate_issue_edit for GitLab:
let mut args = vec![issue_cmd.to_string(), "update".to_string()]; // "issue" "update"
```

### Pattern 3: GitLab Prefix Semantics for Reviewers/Assignees
**What:** glab uses `+`/`-` prefixes on `--reviewer` and `--assignee` to add/remove.
**When to use:** Translating `--add-reviewer X` → `--reviewer +X` and `--remove-reviewer X` → `--reviewer -X`
**Example:**
```rust
// gf pr edit --add-reviewer alice → glab mr update --reviewer +alice
if let Some(reviewer) = add_reviewer {
    args.push("--reviewer".to_string());
    args.push(format!("+{}", reviewer));
}
// gf pr edit --remove-reviewer alice → glab mr update --reviewer -alice
if let Some(reviewer) = remove_reviewer {
    args.push("--reviewer".to_string());
    args.push(format!("-{}", reviewer));
}
```

### Pattern 4: GitLab Label/Unlabel Split
**What:** glab uses `--label` to add and `--unlabel` to remove (separate flags).
**When to use:** Translating `--add-label` → `--label` and `--remove-label` → `--unlabel`
**Example:**
```rust
// gf pr edit --add-label bug → glab mr update --label bug
if let Some(label) = add_label {
    args.push("--label".to_string());
    args.push(label.clone());
}
// gf pr edit --remove-label old → glab mr update --unlabel old
if let Some(label) = remove_label {
    args.push("--unlabel".to_string());
    args.push(label.clone());
}
```

### Pattern 5: Forgejo Subcommand Routing
**What:** fj uses subcommands instead of flags: `fj pr edit <N> labels --add <label> --rm <label>`
**When to use:** Translating label flags to fj's structural layout
**Critical detail:** The PR number comes BEFORE the subcommand: `fj pr edit 42 labels --add bug`
**Example:**
```rust
// gf pr edit 42 --add-label bug --remove-label old
// → fj pr edit 42 labels --add bug --rm old
ForgeType::Forgejo => {
    let mut args = vec![pr_cmd.to_string(), "edit".to_string()];
    if let Some(n) = number {
        args.push(n.clone());
    }
    args.push("labels".to_string());
    if let Some(label) = add_label {
        args.push("--add".to_string());
        args.push(label.clone());
    }
    if let Some(label) = remove_label {
        args.push("--rm".to_string());
        args.push(label.clone());
    }
    Ok(args)
}
```

### Pattern 6: Gitea Plural Flag Rename (Issue Only)
**What:** tea uses plural flag names: `--add-labels`, `--remove-labels`, `--add-assignees`
**When to use:** `gf issue edit` → `tea issues edit`
**Example:**
```rust
// gf issue edit 42 --add-label bug → tea issues edit 42 --add-labels bug
if let Some(label) = add_label {
    args.push("--add-labels".to_string());
    args.push(label.clone());
}
```

### Anti-Patterns to Avoid
- **Partial execution:** Never start building the command for some flags and then error on a later flag. Validate ALL flags first.
- **Splitting into multiple exec calls:** gf uses `exec()` to replace the process. You cannot make two separate calls to fj for labels + something else. This is fine because fj only supports labels anyway.
- **Treating the whole edit as unsupported:** Per-flag granularity means a Forgejo user CAN do `gf pr edit --add-label bug` even though `--add-reviewer` is unsupported. Only error on flags actually present in the invocation.

## Forge CLI Capability Matrix (Verified from --help)

### PR Edit Support Matrix
| gf Flag | gh pr edit | glab mr update | fj pr edit | tea pulls edit |
|---------|-----------|----------------|------------|----------------|
| `--add-label` | ✅ `--add-label` | ✅ `--label` | ✅ `labels --add` | ❌ no edit cmd |
| `--remove-label` | ✅ `--remove-label` | ✅ `--unlabel` | ✅ `labels --rm` | ❌ no edit cmd |
| `--add-reviewer` | ✅ `--add-reviewer` | ✅ `--reviewer +X` | ❌ no subcommand | ❌ no edit cmd |
| `--remove-reviewer` | ✅ `--remove-reviewer` | ✅ `--reviewer -X` | ❌ no subcommand | ❌ no edit cmd |
| `--add-assignee` | ✅ `--add-assignee` | ✅ `--assignee +X` | ❌ no subcommand | ❌ no edit cmd |
| `--remove-assignee` | ✅ `--remove-assignee` | ✅ `--assignee -X` | ❌ no subcommand | ❌ no edit cmd |

### Issue Edit Support Matrix
| gf Flag | gh issue edit | glab issue update | fj issue edit | tea issues edit |
|---------|-------------|-------------------|---------------|-----------------|
| `--add-label` | ✅ `--add-label` | ✅ `--label` | ❌ no subcommand | ✅ `--add-labels` |
| `--remove-label` | ✅ `--remove-label` | ✅ `--unlabel` | ❌ no subcommand | ✅ `--remove-labels` |
| `--add-assignee` | ✅ `--add-assignee` | ✅ `--assignee +X` | ❌ no subcommand | ✅ `--add-assignees` |
| `--remove-assignee` | ✅ `--remove-assignee` | ✅ `--assignee -X` | ❌ no subcommand | ❌ not supported |

**Note:** Issues do NOT have `--add-reviewer`/`--remove-reviewer` — reviewers are a PR-only concept.

### Verification Details
- **gh pr edit:** All 6 flags confirmed via `gh pr edit --help` — `--add-label`, `--remove-label`, `--add-reviewer`, `--remove-reviewer`, `--add-assignee`, `--remove-assignee`
- **gh issue edit:** 4 flags confirmed — `--add-label`, `--remove-label`, `--add-assignee`, `--remove-assignee`. No reviewer flags.
- **glab mr update:** `--label` (add), `--unlabel` (remove labels), `--reviewer` (prefix: `+` to add, `-`/`!` to remove), `--assignee` (prefix: `+` to add, `-`/`!` to remove). Verb is `update`, not `edit`.
- **glab issue update:** `--label`, `--unlabel`, `--assignee` (prefix semantics). No `--reviewer`. Verb is `update`.
- **fj pr edit:** Subcommand-based. Only `labels` subcommand exists (`--add`/`--rm`). No `reviewers` or `assignees` subcommands. Number goes before subcommand: `fj pr edit 42 labels --add bug`.
- **fj issue edit:** Only `title`, `body`, `comment` subcommands. NO labels/assignees subcommands at all.
- **tea pulls edit:** Does NOT exist. `tea pulls` has no `edit` subcommand.
- **tea issues edit:** `--add-labels` (plural!), `--remove-labels` (plural!), `--add-assignees` (plural!). No `--remove-assignees`. 

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-flag validation | Custom trait system | Simple if-chain on flag presence | The existing match-on-forge pattern in this codebase is clean and explicit; keep it simple |
| Argument construction | Builder pattern / struct | `Vec<String>` push pattern | Consistent with all existing translate_* functions |
| Error types | New error variant | Existing `GfError::UnsupportedFeature` | Already has the 3 fields needed (feature, forge, forge_cli) |
| Test generation | Manual test functions | `translation_test!`, `unsupported_test!`, `audit_test!` macros | Already established in tests/flag_audit.rs |

## Common Pitfalls

### Pitfall 1: glab Prefix Semantics — Format String
**What goes wrong:** Passing bare username to `--reviewer` replaces ALL existing reviewers instead of adding.
**Why it happens:** `glab mr update --reviewer alice` means "set reviewers to only alice", not "add alice".
**How to avoid:** ALWAYS prefix with `+` for add and `-` for remove: `--reviewer +alice` / `--reviewer -alice`
**Warning signs:** User reports all other reviewers being removed when adding one.

### Pitfall 2: Forgejo Number Positioning
**What goes wrong:** Putting PR number after the subcommand instead of before.
**Why it happens:** gh uses `gh pr edit 42 --add-label bug` (number then flags). fj uses `fj pr edit 42 labels --add bug` (number then subcommand). Easy to place number wrong.
**How to avoid:** For fj, build args as: `["pr", "edit", number, "labels", "--add", label]`
**Warning signs:** fj returns an error about invalid subcommand or missing PR number.

### Pitfall 3: Gitea Singular vs Plural Flag Names
**What goes wrong:** Passing `--add-label` to tea instead of `--add-labels`.
**Why it happens:** gf's canonical flags are singular (`--add-label`), tea's are plural (`--add-labels`).
**How to avoid:** In the Gitea branch, always use the plural forms: `--add-labels`, `--remove-labels`, `--add-assignees`.
**Warning signs:** tea reports unknown flag error.

### Pitfall 4: Not Checking All Flags Before Building Command
**What goes wrong:** Building partial command for supported flags, then erroring on unsupported flag halfway through.
**Why it happens:** Following the linear pattern of "translate each flag as encountered" without pre-validation.
**How to avoid:** Check ALL flags at the top of the function, error on the first unsupported one, THEN build the command.
**Warning signs:** Inconsistent error behavior depending on flag order.

### Pitfall 5: tea --remove-assignees Doesn't Exist
**What goes wrong:** Assuming tea supports `--remove-assignees` because it supports `--add-assignees`.
**Why it happens:** Natural assumption of symmetry.
**How to avoid:** Return `UnsupportedFeature` for `issue edit --remove-assignee` on Gitea. Verified: tea issues edit --help shows `--add-assignees` but no `--remove-assignees`.
**Warning signs:** tea error about unknown flag.

### Pitfall 6: Forgejo PR Edit with No Label Flags
**What goes wrong:** If user passes only reviewer/assignee flags on Forgejo, we error (UnsupportedFeature). But if they pass NO flags at all, we might generate an incomplete command like `fj pr edit 42 labels` with no `--add`/`--rm`.
**How to avoid:** Only route to `labels` subcommand if at least one label flag is present. If no label flags are present and no other flags are present, just pass through to `fj pr edit`.

## Code Examples

### Clap Subcommand Definition for PR Edit
```rust
// Source: follows existing build_pr() pattern in src/cmd/mod.rs
Command::new("edit")
    .about("Edit a pull request (add/remove labels, reviewers, assignees)")
    .visible_alias("e")
    .arg(
        Arg::new("number")
            .value_name("NUMBER")
            .required(false)
            .help("PR number (optional if on a PR branch)"),
    )
    .arg(
        Arg::new("add-label")
            .long("add-label")
            .value_name("NAME")
            .help("Add labels by name"),
    )
    .arg(
        Arg::new("remove-label")
            .long("remove-label")
            .value_name("NAME")
            .help("Remove labels by name"),
    )
    .arg(
        Arg::new("add-reviewer")
            .long("add-reviewer")
            .value_name("LOGIN")
            .help("Add reviewers by login"),
    )
    .arg(
        Arg::new("remove-reviewer")
            .long("remove-reviewer")
            .value_name("LOGIN")
            .help("Remove reviewers by login"),
    )
    .arg(
        Arg::new("add-assignee")
            .long("add-assignee")
            .value_name("LOGIN")
            .help("Add assignees by login"),
    )
    .arg(
        Arg::new("remove-assignee")
            .long("remove-assignee")
            .value_name("LOGIN")
            .help("Remove assignees by login"),
    )
    .arg(
        Arg::new("extra")
            .num_args(0..)
            .allow_hyphen_values(true)
            .last(true)
            .help("Additional flags passed through to the underlying CLI"),
    )
```

### Clap Subcommand Definition for Issue Edit
```rust
// Same pattern but no reviewer flags (reviewers are PR-only)
Command::new("edit")
    .about("Edit an issue (add/remove labels, assignees)")
    .visible_alias("e")
    .arg(
        Arg::new("number")
            .value_name("NUMBER")
            .required(true) // issue edit always requires a number
            .help("Issue number"),
    )
    .arg(Arg::new("add-label").long("add-label").value_name("NAME").help("Add labels by name"))
    .arg(Arg::new("remove-label").long("remove-label").value_name("NAME").help("Remove labels by name"))
    .arg(Arg::new("add-assignee").long("add-assignee").value_name("LOGIN").help("Add assignees by login"))
    .arg(Arg::new("remove-assignee").long("remove-assignee").value_name("LOGIN").help("Remove assignees by login"))
    .arg(/* extra passthrough */)
```

### Test Macro Examples
```rust
// GitHub PR edit — direct flag mapping
translation_test!(pr_edit_github_add_label,
    input: ["gf", "pr", "edit", "42", "--add-label", "bug"],
    forge: ForgeType::Github,
    expected: ["pr", "edit", "42", "--add-label", "bug"]
);

// GitLab PR edit — verb remap + label/unlabel
translation_test!(pr_edit_glab_add_label,
    input: ["gf", "pr", "edit", "42", "--add-label", "bug"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "update", "42", "--label", "bug"]
);

// GitLab PR edit — reviewer prefix semantics
translation_test!(pr_edit_glab_add_reviewer,
    input: ["gf", "pr", "edit", "42", "--add-reviewer", "alice"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "update", "42", "--reviewer", "+alice"]
);

// Forgejo PR edit — subcommand routing for labels
translation_test!(pr_edit_fj_add_label,
    input: ["gf", "pr", "edit", "42", "--add-label", "bug"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "edit", "42", "labels", "--add", "bug"]
);

// Forgejo PR edit — reviewer is unsupported
unsupported_test!(pr_edit_fj_add_reviewer_unsupported,
    input: ["gf", "pr", "edit", "42", "--add-reviewer", "alice"],
    forge: ForgeType::Forgejo,
    feature_contains: "pr edit --add-reviewer"
);

// Gitea PR edit — entire command unsupported
unsupported_test!(pr_edit_tea_unsupported,
    input: ["gf", "pr", "edit", "42", "--add-label", "bug"],
    forge: ForgeType::Gitea,
    feature_contains: "pr edit"
);

// Gitea issue edit — plural flag rename
translation_test!(issue_edit_tea_add_label,
    input: ["gf", "issue", "edit", "42", "--add-label", "bug"],
    forge: ForgeType::Gitea,
    expected: ["issues", "edit", "42", "--add-labels", "bug"]
);

// Gitea issue edit — remove-assignee unsupported
unsupported_test!(issue_edit_tea_remove_assignee_unsupported,
    input: ["gf", "issue", "edit", "42", "--remove-assignee", "alice"],
    forge: ForgeType::Gitea,
    feature_contains: "issue edit --remove-assignee"
);
```

## Complete Translation Map

### PR Edit: `gf pr edit [<number>] [flags]`

| Flag | GitHub → | GitLab → | Forgejo → | Gitea → |
|------|----------|----------|-----------|---------|
| (verb) | `pr edit` | `mr update` | `pr edit` | ❌ UnsupportedFeature |
| `--add-label X` | `--add-label X` | `--label X` | `labels --add X` | ❌ |
| `--remove-label X` | `--remove-label X` | `--unlabel X` | `labels --rm X` | ❌ |
| `--add-reviewer X` | `--add-reviewer X` | `--reviewer +X` | ❌ UnsupportedFeature | ❌ |
| `--remove-reviewer X` | `--remove-reviewer X` | `--reviewer -X` | ❌ UnsupportedFeature | ❌ |
| `--add-assignee X` | `--add-assignee X` | `--assignee +X` | ❌ UnsupportedFeature | ❌ |
| `--remove-assignee X` | `--remove-assignee X` | `--assignee -X` | ❌ UnsupportedFeature | ❌ |

### Issue Edit: `gf issue edit <number> [flags]`

| Flag | GitHub → | GitLab → | Forgejo → | Gitea → |
|------|----------|----------|-----------|---------|
| (verb) | `issue edit` | `issue update` | see below | `issues edit` |
| `--add-label X` | `--add-label X` | `--label X` | ❌ UnsupportedFeature | `--add-labels X` |
| `--remove-label X` | `--remove-label X` | `--unlabel X` | ❌ UnsupportedFeature | `--remove-labels X` |
| `--add-assignee X` | `--add-assignee X` | `--assignee +X` | ❌ UnsupportedFeature | `--add-assignees X` |
| `--remove-assignee X` | `--remove-assignee X` | `--assignee -X` | ❌ UnsupportedFeature | ❌ UnsupportedFeature |

### UnsupportedFeature Error Count
- **PR edit:** Gitea = 1 whole-command error; Forgejo = 4 per-flag errors (reviewer×2, assignee×2)
- **Issue edit:** Forgejo = 1 per-flag (all 4 flags); Gitea = 1 per-flag (remove-assignee only)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Whole-command UnsupportedFeature | Per-flag UnsupportedFeature | Phase 13 | Users get more granular errors — can use supported flags even when some aren't available |
| Simple flag passthrough | Structural remap to subcommands | Phase 13 | fj pr edit requires converting flags → subcommand+flags layout |

## Open Questions

1. **Multiple values per flag**
   - What we know: gh supports comma-separated values (`--add-label "bug,fix"`). glab supports comma-separated AND repeated flags (`--label bug --label fix`). fj `labels --add` appears to take a single value.
   - What's unclear: Should gf accept multiple values per flag (e.g., `--add-label bug --add-label fix`)? Or single comma-separated?
   - Recommendation: Accept a single value per flag (matching existing codebase pattern). Users can use comma-separated values as supported by their underlying forge CLI. The value is passed through as-is.

2. **Forgejo PR edit with only passthrough flags**
   - What we know: If no label flags are provided but `--extra` passthrough flags exist, what command to generate for fj?
   - Recommendation: If no recognized edit flags are present, pass through as `fj pr edit [number] -- [extra]`. This matches the existing unknown-verb passthrough pattern.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust test framework) |
| Config file | Cargo.toml (test targets auto-discovered) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PR-09 | PR edit add-label, GitHub direct | unit (translation) | `cargo test pr_edit_github_add_label` | ❌ Wave 0 |
| PR-09 | PR edit add-reviewer, GitLab prefix | unit (translation) | `cargo test pr_edit_glab_add_reviewer` | ❌ Wave 0 |
| PR-09 | PR edit add-reviewer, Forgejo unsupported | unit (unsupported) | `cargo test pr_edit_fj_add_reviewer_unsupported` | ❌ Wave 0 |
| PR-09 | PR edit, Gitea whole-command unsupported | unit (unsupported) | `cargo test pr_edit_tea_unsupported` | ❌ Wave 0 |
| PR-09 | PR edit labels, Forgejo subcommand routing | unit (translation) | `cargo test pr_edit_fj_add_label` | ❌ Wave 0 |
| PR-09 | gh pr edit --add-label exists | integration (audit) | `cargo test audit_gh_pr_edit_add_label` | ❌ Wave 0 |
| PR-09 | glab mr update --label exists | integration (audit) | `cargo test audit_glab_mr_update_label` | ❌ Wave 0 |
| ISSUE-08 | Issue edit add-label, GitHub direct | unit (translation) | `cargo test issue_edit_github_add_label` | ❌ Wave 0 |
| ISSUE-08 | Issue edit, GitLab verb remap + label | unit (translation) | `cargo test issue_edit_glab_add_label` | ❌ Wave 0 |
| ISSUE-08 | Issue edit, Forgejo all unsupported | unit (unsupported) | `cargo test issue_edit_fj_add_label_unsupported` | ❌ Wave 0 |
| ISSUE-08 | Issue edit, Gitea plural rename | unit (translation) | `cargo test issue_edit_tea_add_label` | ❌ Wave 0 |
| ISSUE-08 | Issue edit remove-assignee, Gitea unsupported | unit (unsupported) | `cargo test issue_edit_tea_remove_assignee_unsupported` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/flag_audit.rs` — needs ~40+ new translation_test!, unsupported_test!, audit_test! entries
- [ ] `src/cmd/mod.rs` — needs `edit` subcommand definitions in build_pr() and build_issue()
- [ ] `src/adapter/pr.rs` — needs `translate_pr_edit()` function
- [ ] `src/adapter/issue.rs` — needs `translate_issue_edit()` function

*(All gaps are code to be written in implementation — no framework setup needed)*

## Sources

### Primary (HIGH confidence)
- `gh pr edit --help` — verified all 6 add/remove flags (label, reviewer, assignee)
- `gh issue edit --help` — verified 4 add/remove flags (no reviewer on issues)
- `glab mr update --help` — verified `--label`/`--unlabel`, `--reviewer` (prefix), `--assignee` (prefix), verb is `update`
- `glab issue update --help` — verified `--label`/`--unlabel`, `--assignee` (prefix), verb is `update`
- `fj pr edit --help` — verified subcommands: title, body, comment, labels. No reviewers/assignees.
- `fj pr edit labels --help` — verified `--add <ADD>` and `--rm <RM>` flags
- `fj issue edit --help` — verified subcommands: title, body, comment. No labels/assignees.
- `tea pulls edit --help` — verified: command does NOT exist ("No help topic for 'edit'")
- `tea issues edit --help` — verified: `--add-labels`, `--remove-labels`, `--add-assignees`. No `--remove-assignees`.
- Existing codebase: `src/adapter/pr.rs`, `src/adapter/issue.rs`, `src/cmd/mod.rs`, `tests/flag_audit.rs`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, all patterns established
- Architecture: HIGH — follows existing translate_* pattern with validate-first extension
- Pitfalls: HIGH — all verified directly against CLI --help output
- Translation map: HIGH — every cell verified against actual CLI --help

**Research date:** 2026-03-19
**Valid until:** 2026-04-19 (stable — forge CLIs don't change flags frequently)
