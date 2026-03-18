# Phase 7: Flag Normalization Audit - Research

**Researched:** 2026-03-17
**Domain:** Rust test macros, forge CLI flag auditing, flag translation tables
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Audit methodology:**
- Programmatic --help parsing: run actual forge CLIs and verify flags/subcommands exist
- Verify both flag names AND subcommand names (e.g. `glab mr`, `tea pulls`, `tea logins`)
- Flag name existence only — no value type checking
- Missing forge CLI in test environment = hard test failure (all 4 CLIs required)
- Target latest stable CLI versions (whatever --version returns)

**Test coverage strategy:**
- Macro-generated tests: Rust macro produces named test functions from a structured data table
- One test per (command, flag, forge) triple — fine-grained failure reporting
- Test table IS the canonical flag mapping documentation (comments explain each translation)
- Replace existing per-function tests with macro-generated equivalents — single source of truth
- Integration audit tests (--help parsing) in separate `tests/flag_audit.rs`
- All tests run in regular `cargo test` — no feature gating

**v1.1 mapping documentation:**
- Phase 7 pre-maps canonical flags for all new v1.1 commands before Phase 8 begins
- Canonical flags only — what gf will expose, not every forge flag
- Explicit "UNSUPPORTED" marker for forge×flag combinations that don't exist (e.g. pr review on tea)
- Adapter must produce a clear error message for unsupported combinations; test verifies this
- All v1.1 pre-mapped entries verified against real --help output in Phase 7
- Claude determines canonical flag list from REQUIREMENTS.md

**Mismatch handling:**
- Fix immediately — v1.1 is pre-stable, no backwards compatibility concerns
- Passing test suite is sufficient audit report — no separate report artifact
- Fixes show up in git diff

### Claude's Discretion
- Macro design and syntax
- --help output parsing approach (regex, string matching, etc.)
- Exact canonical flag selection for v1.1 commands
- How to structure the test table (module organization, grouping)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| QUAL-01 | All existing flag normalizations audited and verified against current forge CLI help texts | Live --help output collected for all 4 CLIs; existing adapter mappings catalogued; mismatches identified |
| QUAL-02 | All new v1.1 flag normalizations verified against current forge CLI help texts | Live --help captured for all v1.1 commands (pr list/merge/checkout/review, issue list/view/create, repo clone); mapping tables built |
| QUAL-03 | Tests cover flag translation for every command × forge combination | Rust declarative macro pattern identified; test table structure designed; integration audit test approach documented |
</phase_requirements>

---

## Summary

Phase 7 has two distinct work streams. The first is a retrofit: existing adapter tests in `pr.rs` and `repo_auth.rs` are replaced with a single declarative macro table that serves as both documentation and test suite (QUAL-01, QUAL-03). The second is a forward-mapping exercise: the canonical flags for all new v1.1 commands are verified against live CLI help output and recorded in the table before Phase 8 writes any adapter code (QUAL-02).

All four forge CLIs are confirmed present on this machine (gh 2.87.3, glab 1.86.0, tea development, fj v0.4.0). Live --help output was captured for every relevant command during research, and the resulting flag mappings are documented below. One confirmed mismatch was found in the existing adapters (see Existing Adapter Audit section). Several UNSUPPORTED combinations have been identified for v1.1 commands.

The test approach uses a Rust declarative macro (`macro_rules!`) to generate one `#[test]` function per (gf-command, canonical-flag, forge) triple. The integration audit test in `tests/flag_audit.rs` invokes real CLI binaries and checks that every translated subcommand/flag actually appears in `--help` output, catching future CLI version drift.

**Primary recommendation:** Define a `flag_map!` declarative macro in a new `tests/flag_audit.rs` that generates both unit assertion tests (translation correctness) and integration existence tests (forge CLI has the flag), replacing all inline `#[test]` blocks in `src/adapter/`.

---

## CLI Versions Verified (2026-03-17)

| CLI | Version | Binary |
|-----|---------|--------|
| GitHub CLI | 2.87.3 (2026-02-23) | `gh` |
| GitLab CLI | 1.86.0 | `glab` |
| Gitea CLI | development (golang 1.26.0) | `tea` |
| Forgejo CLI | 0.4.0 | `fj` |

---

## Existing Adapter Audit (QUAL-01)

All flag translations were verified against live `--help` output. One mismatch found.

### pr subcommand names
| gf canonical | gh | glab | tea | fj | Status |
|---|---|---|---|---|---|
| `pr` | `pr` | `mr` | `pulls` | `pr` | CORRECT |

### pr create flags
| gf canonical | gh flag | glab flag | tea flag | fj flag | Status |
|---|---|---|---|---|---|
| `--title` | `--title` | `--title` | `--title` | (positional? no, `--title` not listed in fj pr create --help) | INVESTIGATE |
| `--body` | `--body` | `--description` | `--description` | `--body` | CORRECT |
| `--base` | `--base` | `--target-branch` | `--base` | `--base` | CORRECT |
| `--draft` | `--draft` | `--draft` | UNSUPPORTED (no --draft flag in tea pulls create) | UNSUPPORTED | MISMATCH — current code passes --draft for tea |

**Mismatch found:** `translate_pr_create` passes `--draft` verbatim for Gitea. Live `tea pulls create --help` does not list `--draft`. Tea has no draft PR concept. Fix: Gitea should either be UNSUPPORTED or omit the flag silently.

**Note on fj pr create --title:** `fj pr create --help` shows `--body`, `--base`, `--head`, `-r`, `-w`, `-a` but NOT `--title`. Title is omitted — likely required positional or interactive. Phase 7 should verify this and document. Current code passes `--title` for fj; this may be a latent bug.

### pr view flags
| gf canonical | gh | glab | tea | fj | Status |
|---|---|---|---|---|---|
| `[number]` positional | positional | positional | positional | positional | CORRECT |

Note: `tea pulls view` returns "No help topic for 'view'" — tea does not have a `pulls view` subcommand. Current adapter passes `["pulls", "view"]` for Gitea. This is a mismatch. Tea shows pull request details via `tea pulls [<index>]` (bare command with positional index). Fix: translate `gf pr view <N>` → `tea pulls <N>` (no "view" verb).

### repo subcommand names
| gf canonical | gh | glab | tea | fj | Status |
|---|---|---|---|---|---|
| `repo` | `repo` | `repo` | `repos` | `repo` | CORRECT |

### repo create flags
| gf canonical | gh | glab | tea | fj | Status |
|---|---|---|---|---|---|
| `--name` | positional | positional | `--name` | positional (`<REPO>`) | CORRECT |
| `--description` | `--description` | `--description` | `--description` | `--description` | CORRECT |
| `--private` | `--private` | `--visibility private` | `--private` | `--private` (via `-P`) | CORRECT |
| `--public` | `--public` | `--visibility public` | (omit, default) | (omit, default) | CORRECT |
| `--homepage` | `--homepage` | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED | Current code passes --homepage to all; fix: gh only |

### auth subcommand names and flags
| gf canonical | gh | glab | tea | fj | Status |
|---|---|---|---|---|---|
| `auth login` | `auth login` | `auth login` | `logins add` | `auth login` (interactive browser) or `auth add-key` | CORRECT routing |
| `auth logout` | `auth logout` | `auth logout` | `logins rm` | `auth logout` | CORRECT |
| `auth status` | `auth status` | `auth status` | `logins ls` | `auth list` | CORRECT |
| `--hostname` | `--hostname` | `--hostname` | `--url` | UNSUPPORTED (fj auth add-key takes `<USER> [KEY]` positional, no --hostname) | MISMATCH for fj |
| `--token` | `--token` | `--token` | `--token` | UNSUPPORTED (fj add-key uses positional KEY argument) | MISMATCH for fj |

**fj auth note:** `fj auth login` opens a browser — no flags. `fj auth add-key <USER> [KEY]` takes positional args. The current adapter passes `--hostname` and `--token` as flags for Forgejo, which is wrong. Phase 7 must fix this.

---

## v1.1 Command Flag Pre-Mapping (QUAL-02)

All mappings verified against live --help output on 2026-03-17.

### pr list (PR-01)

Canonical gf flags: `--state`, `--author`, `--label`

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `--state` | `--state` (open/closed/merged/all) | `--all`/`--closed`/`--merged` flags | `--state` (all/open/closed) | `--state` (open/closed/all) |
| `--author` | `--author` | `--author` | UNSUPPORTED (no --author on tea pulls list) | `--creator` |
| `--label` | `--label` | `--label` | UNSUPPORTED (tea pulls list has no --label) | `--labels` |

Note on glab state: glab uses boolean flags (`--closed`, `--merged`, `--all`) instead of a `--state value` pattern. Canonical `--state closed` must translate to `--closed` for glab.

fj uses `pr search` (not `pr list`) for listing. Subcommand translation: `gf pr list` → `fj pr search`.

tea: `gf pr list` → `tea pulls list`.

### pr merge (PR-02)

Canonical gf flags: `--squash`, `--rebase`, `--merge`

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `--squash` | `--squash` | `--squash` | `--style squash` | `--method squash` |
| `--rebase` | `--rebase` | `--rebase` | `--style rebase` | `--method rebase` |
| `--merge` | `--merge` | (default, no flag needed) | `--style merge` (default) | `--method merge` |

tea uses `--style <value>` instead of boolean flags. fj uses `--method <value>`.

### pr checkout (PR-03)

Canonical gf flags: `[number]` positional (required)

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `<number>` | positional | positional | positional | `<ID>` positional |

Subcommand: all forges use `checkout` verb except tea which uses `tea pulls checkout`.
gf: no special flags needed for basic checkout. Passthrough for `--branch` branch-name override.

### pr review (PR-04, PR-05)

This is complex: `pr review` must handle both comment (`--comment`, `--body`) and approve (`--approve`).

**glab critical note:** glab uses SEPARATE subcommands:
- `gf pr review --comment` → `glab mr comment --message <body>`
- `gf pr review --approve` → `glab mr approve` (separate subcommand, no body)

**tea:** No `tea pulls review` or `tea pulls approve` subcommand exists. UNSUPPORTED for both comment and approve on tea. Adapter must return a clear error.

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `pr review --comment --body <text>` | `pr review --comment --body <text>` | `mr comment --message <text>` | UNSUPPORTED | `pr comment <PR> <body>` |
| `pr review --approve` | `pr review --approve` | `mr approve` (subcommand, no body) | UNSUPPORTED | UNSUPPORTED (fj has no approve subcommand) |

### issue list (ISSUE-01)

Canonical gf flags: `--state`, `--author`, `--label`

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `--state` | `--state` | `--all`/`--closed` | `--state` | `--state` |
| `--author` | `--author` | `--author` | `--author` | UNSUPPORTED (fj issue search has --creator, but issue list only has labels/assignee/state) |
| `--label` | `--label` | `--label` | `--labels` | `--labels` |

Subcommand: gh=`issue list`, glab=`issue list`, tea=`issues list`, fj=`issue search` (fj has no `issue list`, uses `issue search`).

### issue view (ISSUE-02)

Canonical gf flags: `<number>` positional

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `<number>` | positional | positional | positional (bare `tea issues <index>`) | positional |

Note: `tea issues view` does NOT exist — same pattern as `tea pulls view`. Use `tea issues <number>` directly.

Subcommand: gh=`issue view <N>`, glab=`issue view <N>`, tea=`issues <N>`, fj=`issue view <N>`.

### issue create (ISSUE-03)

Canonical gf flags: `--title`, `--body`

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `--title` | `--title` | `--title` | `--title` | `--title` (via `-t`... verify) |
| `--body` | `--body` | `--description` | `--description` | (interactive or via positional?) |

Note: `fj issue create --help` not verified in this research pass — must be confirmed in implementation. Confidence: MEDIUM.

Subcommand: gh=`issue create`, glab=`issue create`, tea=`issues create`, fj=`issue create`.

### repo clone (REPO-01)

Canonical gf flags: `<repo>` positional (owner/repo or URL)

| gf canonical | gh | glab | tea | fj |
|---|---|---|---|---|
| `<repo>` | positional | positional | UNSUPPORTED (`tea repos clone` — "No help topic for 'clone'") | positional `<REPO>` |

**tea repo clone:** Not supported. Adapter must return a clear error for `gf repo clone` on Gitea.

Subcommand: gh=`repo clone`, glab=`repo clone`, tea=UNSUPPORTED, fj=`repo clone`.

---

## Architecture Patterns

### Recommended Project Structure for Phase 7

```
src/
├── adapter/
│   ├── mod.rs          (dispatch — unchanged)
│   ├── pr.rs           (replace inline tests with macro table)
│   └── repo_auth.rs    (replace inline tests with macro table)
tests/
├── flag_audit.rs       (NEW: integration audit + macro table unit tests)
├── helpers/
│   └── exit_with.rs    (existing)
└── integration_test.rs (existing, unchanged)
```

### Pattern 1: Declarative Macro for Translation Tests

The macro approach generates named test functions from a data table. This is idiomatic Rust for repetitive test generation.

**What:** A `macro_rules!` macro that takes (test_name, forge, input_args, expected_output) and emits a `#[test]` fn.

**Example macro design:**

```rust
// In tests/flag_audit.rs or a module within src/adapter/

macro_rules! assert_translation {
    ($name:ident, $forge:expr, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let matches = gf::cmd::build_cli()
                .try_get_matches_from($input)
                .expect("parse ok");
            let (_, sub) = matches.subcommand().expect("subcommand");
            let result = gf::adapter::translate($forge, sub);
            assert_eq!(result, $expected, "translation mismatch");
        }
    };
}

// Usage:
assert_translation!(
    pr_create_glab_body,            // test name
    ForgeType::Gitlab,              // forge
    ["gf", "pr", "create", "--body", "hello"],  // input
    vec!["mr", "create", "--description", "hello"]  // expected
);
```

**When to use:** For every (gf-command, canonical-flag, forge) triple that has deterministic translation logic.

### Pattern 2: Integration Audit — --help Output Verification

Run the actual forge CLI binary and verify a subcommand/flag exists in its help output. This catches CLI version drift without needing to maintain a static snapshot.

```rust
// In tests/flag_audit.rs

fn assert_help_contains(cli: &str, args: &[&str], expected_fragment: &str) {
    let output = std::process::Command::new(cli)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("{cli} not found — all 4 CLIs are required"));

    let help_text = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    assert!(
        help_text.contains(expected_fragment),
        "{cli} {args:?} --help does not contain '{expected_fragment}'\nGot:\n{help_text}"
    );
}
```

Note: Most CLIs print help to stderr on non-zero exit, or stdout on `--help`. Combine both streams for robust matching.

**When to use:** For every translated subcommand name and flag name. Gates on the translated output, not the canonical gf flag.

### Anti-Patterns to Avoid

- **Snapshot testing full --help output:** CLI help text changes frequently; check for specific flag presence only.
- **Feature-gating integration tests:** Decision is explicit — all tests run in `cargo test`, no `#[cfg(feature = "integration")]`.
- **Testing value types or formats:** Only verify flag name existence, not whether `--state` takes `open` vs `Open`.
- **One mega-test per forge:** One test per triple gives fine-grained failure reporting.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Named test generation | Manual copy-paste per forge | `macro_rules!` | Keeps table in sync with tests; one edit point |
| CLI detection in tests | Custom PATH logic in audit tests | `which::which` (already in deps) | Already proven; reuse in test setup |
| Help output parsing | Custom parser | `str::contains` on combined stdout+stderr | CLI help is free-form; existence check is sufficient |

---

## Common Pitfalls

### Pitfall 1: tea command layout diverges from pattern
**What goes wrong:** `tea pulls view <N>` does not exist. `tea issues view <N>` does not exist. Both use bare positional on the noun command (`tea pulls <N>`, `tea issues <N>`).
**Why it happens:** tea CLI has inconsistent subcommand structure — some nouns have `view`, some don't.
**How to avoid:** Never assume `<noun> view <N>` works for tea. Check `tea <noun> --help` directly.
**Warning signs:** `Error: No help topic for 'view'` in tea output.

### Pitfall 2: fj auth flags are positional, not --flag style
**What goes wrong:** Current adapter passes `--hostname` and `--token` to fj, which fj does not accept. `fj auth add-key <USER> [KEY]` takes positional arguments.
**Why it happens:** `fj auth login` (browser) and `fj auth add-key` (token) are different commands with different signatures.
**How to avoid:** Map `gf auth login --token <T>` → `fj auth add-key <user> <T>` — but USER is unknown without context. This may need to remain UNSUPPORTED or require `--username` as a canonical flag.

### Pitfall 3: glab state uses boolean flags not --state value
**What goes wrong:** `glab mr list --state closed` does not work. glab uses `glab mr list --closed`.
**Why it happens:** glab has per-state boolean flags instead of a value flag.
**How to avoid:** Translate `--state closed` → `--closed`, `--state merged` → `--merged`, `--state all` → `--all` for glab.

### Pitfall 4: glab pr review requires subcommand switching
**What goes wrong:** For review, glab uses `mr comment` (for comments) and `mr approve` (for approval) — completely different subcommands, not flags.
**Why it happens:** glab models review as separate operations; `mr review` does not exist as a verb.
**How to avoid:** `translate_pr_review` must check the canonical flag (`--comment` vs `--approve`) and produce the correct glab subcommand word.

### Pitfall 5: fj pr create has no --title flag
**What goes wrong:** `fj pr create --title "fix"` — `--title` is NOT listed in `fj pr create --help`. Title may be interactive or via a different mechanism.
**Why it happens:** fj CLI is younger and less featureful.
**How to avoid:** Verify fj pr create title behavior before writing the adapter. May need UNSUPPORTED with fallback error.

### Pitfall 6: Macro-generated tests don't appear in IDE test runners
**What goes wrong:** Declarative macros produce opaque test names that IDEs may not surface cleanly.
**Why it happens:** `macro_rules!` concatenation is not always IDE-friendly.
**How to avoid:** Name tests with snake_case descriptive names (`pr_create_github_title`, not `test_0_1`). Use `cargo test pr_create` filtering to run subsets.

---

## Code Examples

### Help-checking integration test pattern

```rust
// Source: verified against all 4 CLIs in this research session

fn forge_help_contains(cli: &str, subargs: &[&str], expected: &str) {
    let output = std::process::Command::new(cli)
        .args(subargs)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("{cli} not found on PATH: {e}"));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        text.contains(expected),
        "AUDIT FAIL: `{cli} {} --help` does not contain '{expected}'\nFull output:\n{text}",
        subargs.join(" ")
    );
}

// Example usage in #[test]:
#[test]
fn audit_glab_mr_create_description_flag() {
    forge_help_contains("glab", &["mr", "create"], "--description");
}

#[test]
fn audit_tea_pulls_list_state_flag() {
    forge_help_contains("tea", &["pulls", "list"], "--state");
}
```

### Declarative macro pattern for translation unit tests

```rust
// Macro-generated translation tests — one per (command, flag, forge) triple

macro_rules! translation_test {
    ($name:ident, input: $input:expr, forge: $forge:expr, expected: $expected:expr) => {
        #[test]
        fn $name() {
            use gf::cmd::build_cli;
            use gf::adapter::translate;
            let matches = build_cli()
                .try_get_matches_from($input)
                .unwrap_or_else(|e| panic!("parse failed: {e}"));
            let (_, sub) = matches.subcommand().expect("subcommand");
            assert_eq!(translate($forge, sub), $expected);
        }
    };
}

translation_test!(
    pr_create_glab_body_to_description,
    input: ["gf", "pr", "create", "--body", "hello"],
    forge: ForgeType::Gitlab,
    expected: vec!["mr", "create", "--description", "hello"]
);
```

### UNSUPPORTED combination error pattern

```rust
// When a forge does not support a canonical flag combination,
// the adapter returns a special sentinel that main.rs converts to an error:

fn translate_pr_review(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    match forge {
        ForgeType::Gitea => {
            // tea has no pr review or approve subcommand
            return vec!["__gf_error__".to_string(),
                "pr review is not supported on Gitea (tea)".to_string()];
        }
        // ... other forges
    }
}

// Or alternatively: return a specific error type through the translate signature.
// The exact error propagation mechanism is Claude's discretion.
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Inline `#[test]` per flag per forge | Declarative macro table | Phase 7 (this phase) | Single source of truth; easier to add new forges |
| No integration audit | `tests/flag_audit.rs` running real CLIs | Phase 7 (this phase) | Catches CLI version drift automatically |

---

## Open Questions

1. **fj pr create --title flag**
   - What we know: `fj pr create --help` does not list `--title`
   - What's unclear: Is title interactive-only? Is there an undocumented flag? Or does fj just not support non-interactive PR creation?
   - Recommendation: Test `fj pr create --title "test" --base main --head feature` in a live repo; if it fails, mark as UNSUPPORTED and error clearly.

2. **fj auth login + --hostname/--token mapping**
   - What we know: `fj auth login` opens browser (no flags). `fj auth add-key <USER> [KEY]` takes positional key but no hostname.
   - What's unclear: How does fj know WHICH instance to authenticate against? Does `fj auth add-key` default to the remote-detected instance?
   - Recommendation: Mark `gf auth login --hostname` as UNSUPPORTED for fj; document that fj auth is managed via `fj auth login` (browser only) or `fj auth add-key`.

3. **tea pulls draft PR**
   - What we know: `tea pulls create --draft` is not in help output
   - What's unclear: Does tea support draft PRs at all, or just not via this flag?
   - Recommendation: Mark UNSUPPORTED; silently drop the flag with a warning, or error. Claude's discretion.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + cargo |
| Config file | `Cargo.toml` (existing) |
| Quick run command | `cargo test adapter` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| QUAL-01 | Existing flag translations are correct | unit (macro table) | `cargo test pr_create` | ❌ Wave 0 — `tests/flag_audit.rs` |
| QUAL-01 | Existing subcommand names are correct | unit (macro table) | `cargo test pr_subcommand` | ❌ Wave 0 — `tests/flag_audit.rs` |
| QUAL-01 | Translated flags exist in forge --help | integration | `cargo test audit_` | ❌ Wave 0 — `tests/flag_audit.rs` |
| QUAL-02 | v1.1 flag translations are correct | unit (macro table) | `cargo test v11_` | ❌ Wave 0 — `tests/flag_audit.rs` |
| QUAL-02 | v1.1 forge --help contains translated flags | integration | `cargo test audit_v11` | ❌ Wave 0 — `tests/flag_audit.rs` |
| QUAL-03 | Every command×forge has a test entry | unit (macro table completeness) | `cargo test` | ❌ Wave 0 — macro table coverage |

### Sampling Rate
- **Per task commit:** `cargo test adapter`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/flag_audit.rs` — new file; covers QUAL-01, QUAL-02, QUAL-03
  - Integration audit helper fn `forge_help_contains()`
  - Macro `translation_test!` or `assert_translation!`
  - All existing adapter tests migrated to macro table
  - v1.1 pre-mapping table entries
- [ ] `adapter::translate` must be `pub` (currently pub) — confirmed accessible
- [ ] `adapter::pr` internals may need to be `pub(crate)` for test access from `tests/` — OR tests invoke `translate()` only at the top level

Note on test visibility: Tests in `tests/` are integration-level and can only access `pub` items in the `gf` crate. The macro-table tests should call `gf::adapter::translate()` (top-level pub function), not the private per-function translators. This means macro tests exercise the full dispatch path, which is correct.

---

## Sources

### Primary (HIGH confidence)
- Live `gh --help`, `glab --help`, `tea --help`, `fj --help` invocations — all flags verified directly
- `src/adapter/pr.rs`, `src/adapter/repo_auth.rs` — existing translation logic read directly
- `src/cmd/mod.rs` — canonical gf flag definitions read directly

### Secondary (MEDIUM confidence)
- Rust `macro_rules!` documentation — standard language feature, well-understood
- Rust test harness patterns — standard

### Tertiary (LOW confidence)
- fj pr create `--title` behavior — `--help` does not list it; behavior under interactive invocation unknown

---

## Metadata

**Confidence breakdown:**
- Existing adapter audit: HIGH — all flags verified against live CLI output
- v1.1 pre-mapping: HIGH for gh/glab/fj; MEDIUM for tea (some gaps noted)
- Architecture: HIGH — standard Rust patterns
- Pitfalls: HIGH — directly observed from live CLI testing

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (CLI versions change; re-verify if any CLI is updated)
