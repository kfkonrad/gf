# Phase 3: Command Routing - Research

**Researched:** 2026-03-16
**Domain:** Rust/clap CLI routing, per-forge flag translation, alias systems
**Confidence:** MEDIUM-HIGH (clap patterns HIGH; fj CLI flags MEDIUM)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Flag normalization scope:**
  - `pr` commands: normalize `--title`, `--body` → `--description` (glab), `--base` → `--target-branch` (glab), `--draft`
  - `repo` commands: full canonical flag set for `repo create` (name, description, visibility, homepage, etc.) mapped to each forge's equivalents
  - `auth` commands: normalize any flags that differ across forges (researcher to identify divergences)
  - Unrecognized flags: pass through unchanged to the underlying CLI (PR-04) — no warnings, fully transparent
- **Forgejo (`fj`) CLI:** Fully implement in Phase 3 — first-class support, not a stub
- **Alias presentation:** Aliases appear inline under the canonical command in clap `--help`:
  ```
  pr create    Create a pull/merge request
               [aliases: c, mr create, mr c]
  ```
  `mr` is listed as an alias under `pr` at the top level. `gf mr --help` and `gf mr create` work identically.
- **`gf pr view` with no number:** Delegate entirely to underlying CLI — call `gh pr view` / `glab mr view` with no number. For `tea` and `fj`: researcher verifies whether current-branch lookup is supported.

### Claude's Discretion

- Internal clap subcommand structure (e.g., how to model `mr` as an alias for `pr` in clap)
- ForgeAdapter trait vs data table vs translation function
- Module layout for the router/adapter code
- Exact canonical flag set for `auth` commands (after researcher identifies divergences)
- Exact canonical flag set additions beyond the defined ones for `repo create`

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-08 | Non-pr command groups have one-letter alias (`r`=repo, `a`=auth, `b`=browse) | clap `visible_alias` on Command |
| CORE-09 | `mr` is a full alias for `pr` (e.g. `gf mr create` works like `gf pr create`) | clap `visible_alias("mr")` on pr subcommand |
| CORE-10 | Every verb has one-letter alias (`c`=create, `v`=view, `f`=fork, `l`=login, `s`=status) | clap `visible_alias` on each verb subcommand |
| CORE-11 | All aliases appear in clap-generated `--help` text | `visible_alias` (not hidden `alias`) ensures this |
| CORE-12 | All aliases included in shell completion scripts from clap | `clap_complete` respects `visible_alias`; hidden aliases also included in completions |
| PR-01 | `gf pr create` (and all aliases) creates PR/MR using canonical flags | ForgeAdapter maps canonical → forge args before runner::run |
| PR-02 | System translates canonical flags to forge-specific equivalents | Flag translation table per ForgeType |
| PR-03 | System translates command group name to forge equivalent | e.g. `pr` → `mr` (glab), `pr` → `pulls` (tea) |
| PR-04 | Unrecognized flags pass through unchanged | `allow_external_subcommands` or collect remaining args |
| PR-05 | `gf pr view [<number>]` — if on branch with open PR, number optional | Delegate to underlying CLI no-number path |
| PR-06 | Fork PR lookup delegates to underlying CLI | Transparent delegation confirmed correct approach |
| REPO-01 | `gf repo view` (aliases: `gf r v`) delegates to forge CLI | ForgeAdapter maps repo view → forge equivalent |
| REPO-02 | `gf repo create` (aliases: `gf r c`) creates repo | Full canonical flag set translated per forge |
| REPO-03 | `gf repo fork` (aliases: `gf r f`) forks repo | Forge-specific fork command routing |
| AUTH-01 | `gf auth login` (aliases: `gf a l`) authenticates | Delegates with flag normalization |
| AUTH-02 | `gf auth logout` removes credentials | Delegates; `tea` uses `tea logins rm` |
| AUTH-03 | `gf auth status` (aliases: `gf a s`) checks auth state | Delegates with flag normalization |
</phase_requirements>

---

## Summary

Phase 3 introduces clap-based CLI structure replacing the hand-rolled arg parser in `main.rs`. The two primary challenges are: (1) modeling `mr` as a top-level visible alias for `pr` in clap such that `gf mr create` routes identically to `gf pr create`, and (2) building a per-forge translation table that maps canonical flags to forge-specific equivalents before handing translated args to `runner::run`.

The existing `runner::run(cli, args)` function is preserved — it is the exec boundary. The ForgeAdapter layer is purely an arg-translation step between the clap parse result and the `runner::run` call. Flag passthrough (PR-04) means unrecognized args collected during clap parsing are appended verbatim to the translated arg list.

The `tea` CLI uses `pulls` not `pr` for its PR command group, and uses `--description` (not `--body`) for the PR body. The `fj` CLI uses `--body` for PR description (matching canonical form) and `--base`/`--head` for branch names (also matching canonical form) — making `fj` the easiest to translate for `pr create`. Auth for `tea` uses a completely different command structure (`tea logins add/rm/ls` instead of `tea auth login/logout/status`) requiring a full remapping. Forgejo `fj` uses `auth logout` natively.

**Primary recommendation:** Build `src/cmd/` module with clap definitions and `src/adapter.rs` with a `translate(forge, subcommand, canonical_args) -> Vec<String>` function backed by per-forge flag maps.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.x (already in ecosystem; add to Cargo.toml) | CLI parsing, subcommands, aliases, completions | Standard Rust CLI library; built-in visible_alias support |
| clap_complete | 4.x (clap companion crate) | Shell completion script generation | Companion crate, same version family; respects visible aliases |

**Installation:**
```bash
# Add to Cargo.toml [dependencies]
clap = { version = "4", features = ["derive"] }

# Add to Cargo.toml [dev-dependencies] or [dependencies] depending on completion approach
clap_complete = "4"
```

Note: clap feature selection matters. `derive` macro feature enables the `#[derive(Parser, Subcommand, Args)]` API. The `cargo` feature brings in extra completions. Use minimal feature set.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| clap derive | clap builder API | Builder is more verbose but sometimes more flexible for dynamic structures; derive is cleaner for static command trees |
| clap_complete | manual completion scripts | Manual is unmaintainable; clap_complete auto-generates from the Command tree |

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs          # Thin: parse args → detect forge → translate → runner::run
├── cmd/
│   ├── mod.rs       # Cli struct (top-level Parser)
│   ├── pr.rs        # PrArgs, PrCommand subcommands (create, view, etc.)
│   ├── repo.rs      # RepoArgs, RepoCommand subcommands
│   └── auth.rs      # AuthArgs, AuthCommand subcommands
├── adapter.rs       # ForgeAdapter: translate canonical → forge args
├── forge/mod.rs     # (existing) ForgeType, detect()
├── runner.rs        # (existing) run()
└── error.rs         # (existing) GfError
```

### Pattern 1: Top-Level CLI with `mr` as Visible Alias for `pr`

**What:** `mr` appears as a visible alias for `pr` at the top level. Clap routes `gf mr ...` to the same subcommand handler as `gf pr ...`.

**When to use:** Whenever an alias must work identically but appear in help text.

**Example (builder API, gives most control over alias display):**
```rust
// src/cmd/mod.rs
use clap::{Command, ArgMatches};

pub fn build_cli() -> Command {
    Command::new("gf")
        .subcommand(
            build_pr_command()
                .visible_alias("mr")   // CORE-09: mr aliases pr at top level
        )
        .subcommand(
            Command::new("repo")
                .visible_alias("r")    // CORE-08: r aliases repo
                .subcommand(build_repo_create())
                .subcommand(build_repo_view())
                .subcommand(build_repo_fork())
        )
        .subcommand(
            Command::new("auth")
                .visible_alias("a")    // CORE-08: a aliases auth
                .subcommand(build_auth_login())
                .subcommand(build_auth_logout())
                .subcommand(build_auth_status())
        )
}

fn build_pr_command() -> Command {
    Command::new("pr")
        .subcommand(
            Command::new("create")
                .visible_alias("c")              // CORE-10
                .visible_aliases(["mr create", "mr c"])  // cross-group aliases in help
                .arg(Arg::new("title").long("title").short('t'))
                .arg(Arg::new("body").long("body").short('b'))
                .arg(Arg::new("base").long("base").short('B'))
                .arg(Arg::new("draft").long("draft").action(ArgAction::SetTrue))
                .allow_hyphen_values(true)
                .trailing_var_arg(true)  // capture passthrough args
        )
        .subcommand(
            Command::new("view")
                .visible_alias("v")              // CORE-10
                .arg(Arg::new("number").index(1).required(false))
                .trailing_var_arg(true)
        )
}
```

**Important note on `mr create` as multi-word alias:** clap does NOT natively support multi-word aliases (e.g., `mr create` as a single alias string). The correct approach for `gf mr create` routing is: since `mr` is a visible alias for `pr` at the top level, `gf mr create` naturally routes to `pr create` subcommand. No additional alias is needed. The help display text for `pr create` can show `[aliases: c, mr create, mr c]` via the `.after_help()` or `.long_about()` if clap cannot express multi-word aliases — or simply display `[aliases: c]` and note `mr` at the `pr` level.

### Pattern 2: ForgeAdapter Translation

**What:** Pure function that takes a `ForgeType`, the canonical subcommand path (e.g., `["pr", "create"]`), and canonical args, and returns the translated arg vector.

**When to use:** Between clap parse result and `runner::run`.

```rust
// src/adapter.rs

pub fn translate(
    forge: ForgeType,
    subcommand: &[&str],   // e.g. ["pr", "create"]
    canonical_args: &[String],
    passthrough: &[String],
) -> Vec<String> {
    let translated_subcmd = translate_subcommand(forge, subcommand);
    let translated_flags = translate_flags(forge, subcommand, canonical_args);
    let mut result = translated_subcmd;
    result.extend(translated_flags);
    result.extend(passthrough.iter().cloned());
    result
}

fn translate_subcommand(forge: ForgeType, subcmd: &[&str]) -> Vec<String> {
    match (forge, subcmd) {
        (ForgeType::Gitlab, ["pr", verb]) => vec!["mr".to_string(), verb.to_string()],
        (ForgeType::Gitea,  ["pr", verb]) => vec!["pulls".to_string(), verb.to_string()],
        // fj uses "pr" natively — no translation needed
        (_, subcmd) => subcmd.iter().map(|s| s.to_string()).collect(),
    }
}
```

**Flag translation table per forge:**
```rust
// Canonical → GitLab mapping for pr create
fn glab_pr_create_flags(flag: &str) -> Option<&'static str> {
    match flag {
        "--body"  => Some("--description"),
        "--base"  => Some("--target-branch"),
        _         => None,  // unknown = passthrough
    }
}

// Canonical → Gitea (tea) mapping for pr create
fn tea_pr_create_flags(flag: &str) -> Option<&'static str> {
    match flag {
        "--body"  => Some("--description"),
        // --base and --title map directly; tea uses --base natively
        _         => None,
    }
}

// fj: canonical flags mostly match — --body, --base, --title, --draft all native
// No translation needed for pr create
```

### Pattern 3: Passthrough Args Collection

**What:** Args not recognized by clap are collected and appended verbatim (PR-04).

**How:** Use `Command::allow_external_subcommands(false)` but add `.trailing_var_arg(true)` and a catch-all `Arg::new("rest").num_args(0..).last(true)` to capture remaining args after known flags.

Alternative: Parse known flags manually from `env::args()` after clap's `try_get_matches` — but this is complex. Preferred: use `trailing_var_arg` with `allow_hyphen_values`.

```rust
// In each subcommand that supports passthrough:
Command::new("create")
    .arg(Arg::new("title").long("title"))
    .arg(Arg::new("body").long("body"))
    .arg(Arg::new("base").long("base"))
    .arg(Arg::new("draft").long("draft").action(ArgAction::SetTrue))
    .arg(
        Arg::new("extra")
            .num_args(0..)
            .allow_hyphen_values(true)
            .last(true)
    )
```

### Pattern 4: Shell Completion Generation (CORE-12)

```rust
// Can be a hidden subcommand or a build.rs step
use clap_complete::{generate, Shell};

fn print_completions(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
```

`clap_complete` respects `visible_alias` — aliases appear as completion candidates. Hidden aliases (`.alias()`) are also completed but not shown in help (CORE-11 requires visible, CORE-12 requires completion — both needs met by `visible_alias`).

### Anti-Patterns to Avoid

- **Separate `mr` top-level subcommand:** Don't add `Command::new("mr")` as a duplicate of `pr`. Use `visible_alias("mr")` on the `pr` command. Otherwise you have two command trees to maintain.
- **String-matching after exec:** Don't try to intercept or parse the underlying CLI's output. `runner::run` uses `exec()` — there's no return.
- **Translating `--draft` for glab:** glab's `mr create` supports `--draft` natively (verified as standard flag). No translation needed.
- **Treating `tea logins` as `auth`:** tea has no `tea auth` subcommand. `gf auth login` must translate to `tea logins add`, `gf auth logout` to `tea logins rm`, `gf auth status` to `tea logins ls` (or `tea logins whoami`).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arg parsing | Custom tokenizer | clap | Handles --flag=value, -f value, quoting, help generation |
| Shell completions | Manual bash/zsh scripts | clap_complete | Auto-derived from Command tree, stays in sync |
| PATH lookup | `std::fs::exists("/usr/bin/gh")` | `which` crate (already present) | Cross-platform, handles symlinks, $PATH |

---

## Forge-Specific Flag Reference

### PR Create — Canonical → Forge Mapping

| Canonical flag | `gh` | `glab` | `tea` | `fj` |
|---------------|------|--------|-------|------|
| `--title` | `--title` | `--title` | `--title` | positional title or `--title` (verify) |
| `--body` | `--body` | `--description` | `--description` | `--body` |
| `--base` | `--base` | `--target-branch` | `--base` | `--base` |
| `--draft` | `--draft` | `--draft` | not supported (passthrough) | unknown — verify |

**fj confidence:** MEDIUM — verified `--body` and `--base` from 2026-02 guide. `--title` is MEDIUM (likely positional). `--draft` is LOW — verify against `fj pr create --help`.

### PR subcommand name per forge

| Canonical | `gh` | `glab` | `tea` | `fj` |
|-----------|------|--------|-------|------|
| `pr` | `pr` | `mr` | `pulls` | `pr` |

**Sources:** gh and glab are well-known. tea uses `pulls` (verified via gitea.com/gitea/tea docs CLI.md). fj uses `pr` (verified from Codeberg forgejo-cli description).

### PR View — current-branch lookup support

| CLI | `view` no number | Behavior |
|----|-----------------|---------|
| `gh` | `gh pr view` | Native: finds PR for current branch. HIGH confidence. |
| `glab` | `glab mr view` | Native: finds MR for current branch. HIGH confidence. |
| `tea` | `tea pulls view` | Unclear — tea may require a PR number. MEDIUM/LOW confidence. Flag as "may require number; delegate and let tea error if unsupported." |
| `fj` | `fj pr view` | Unclear — needs verification. Same policy: delegate and surface CLI error. |

**Decision (locked):** Delegate entirely; let underlying CLI handle it. No gf-level lookup.

### Repo Create — Canonical Flag Mapping

| Canonical flag | `gh` | `glab` | `tea` | `fj` |
|---------------|------|--------|-------|------|
| `--name` | `<name>` (positional) | `<name>` (positional or `--name`) | `--name` | `--name` (verify) |
| `--description` | `--description` | `--description` | `--description` | `--description` (verify) |
| `--private` | `--private` | `--visibility private` | `--private` | verify |
| `--public` | `--public` | `--visibility public` | (default) | verify |
| `--homepage` | `--homepage` | (passthrough) | (not supported?) | verify |

**Note:** `--visibility` vs `--private`/`--public` is a key divergence. glab uses `--visibility private/public/internal`; gh uses `--private`/`--public` flags. Canonical form: `--private` / `--public` booleans → translate to glab `--visibility`.

### Auth — Canonical Command Mapping

| gf canonical | `gh` | `glab` | `tea` | `fj` |
|-------------|------|--------|-------|------|
| `auth login` | `auth login` | `auth login` | `logins add` | `auth add-key` (MEDIUM confidence) |
| `auth logout` | `auth logout` | `auth logout` | `logins rm` | `auth logout` |
| `auth status` | `auth status` | `auth status` | `logins ls` | `auth list` (MEDIUM confidence) |

**tea auth is completely different:** tea has no `auth` subcommand. The mapping `auth login → logins add`, `auth logout → logins rm`, `auth status → logins ls` is a hard remapping, not a flag rename.

**fj auth confidence:** MEDIUM. From 2026-02 guide: `auth add-key` for registration, `auth list` for listing, `auth logout` for logout. The `auth logout` form matches canonical. `auth login` → `auth add-key` is the key translation.

**Auth flag divergences (canonical set recommendation):**

For `auth login`:
- `--hostname` / `-h` — all CLIs support specifying a host; canonical `--hostname` maps directly to glab `--hostname`, gh `--hostname`; tea uses `--url`; fj likely `--url` or `-H` per the guide
- `--token` — gh and glab support `--token`; tea uses `--token`; canonical `--token` passes through
- No other auth flags need normalization beyond the subcommand remapping for tea

---

## Common Pitfalls

### Pitfall 1: Multi-word aliases in clap
**What goes wrong:** Attempting `Command::new("pr create").visible_alias("mr create")` — clap does not support multi-word aliases. Alias resolution is single-token only.
**Why it happens:** Conceptually `mr create` seems like an alias for `pr create`, but clap routing is token-by-token.
**How to avoid:** `mr` aliases `pr` at the top level. Then `gf mr create` routes as: `mr` → resolves to `pr` command → then `create` subcommand. The help display for `pr create` can note `mr create` as an equivalent in its description string, but this is cosmetic.
**Warning signs:** If you see `visible_alias("mr create")` with a space, that will silently not match at runtime.

### Pitfall 2: `trailing_var_arg` + `allow_hyphen_values` ordering
**What goes wrong:** Passthrough args with leading `--` (e.g., `--draft --assignee foo`) get consumed by clap's unknown arg rejection before reaching the catch-all arg.
**Why it happens:** clap rejects unknown flags unless `allow_external_subcommands` or `allow_unknown_args` is set.
**How to avoid:** Use `.allow_unknown_args(true)` on the leaf subcommands that need passthrough, combined with collecting remaining args via `ArgMatches::get_many::<String>("extra")`.

### Pitfall 3: tea's auth structure
**What goes wrong:** Translating `gf auth login` to `tea auth login` — tea has no `auth` subcommand.
**Why it happens:** `gh`, `glab`, and `fj` all have `auth` subcommands; tea is the exception.
**How to avoid:** The ForgeAdapter must remap the entire subcommand for tea auth: `["auth", "login"] → ["logins", "add"]`, `["auth", "logout"] → ["logins", "rm"]`, `["auth", "status"] → ["logins", "ls"]`.

### Pitfall 4: glab repo visibility
**What goes wrong:** Passing `--private` to `glab repo create` — glab uses `--visibility private`, not `--private`.
**Why it happens:** gh uses `--private`/`--public` boolean flags; glab uses `--visibility <level>` string.
**How to avoid:** The canonical `--private` flag must translate to `--visibility private` for glab in the repo create adapter.

### Pitfall 5: exec() means no output capture
**What goes wrong:** Trying to intercept or modify the underlying CLI output after exec.
**Why it happens:** `runner::run` uses `exec()` on Unix — gf is replaced. There's nothing to intercept.
**How to avoid:** All translation happens before `runner::run`. The adapter is the only place to act.

---

## Code Examples

### main.rs Phase 3 flow
```rust
// Source: architecture from CONTEXT.md code_context section
fn main() {
    let mut cmd = build_cli();
    let matches = cmd.clone().get_matches();

    let remote = matches.get_one::<String>("remote")
        .map(|s| s.as_str())
        .unwrap_or("origin");

    let forge_type = match forge::detect(remote) {
        Ok(f) => f,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let translated_args = adapter::translate_matches(forge_type, &matches);

    if let Err(e) = runner::run(forge_type.cli_name(), &translated_args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
```

### clap_complete integration for CORE-12
```rust
// As a hidden subcommand: gf completions --shell bash
Command::new("completions")
    .hide(true)
    .arg(Arg::new("shell").value_parser(value_parser!(Shell)))
    .action(|matches| {
        let shell = matches.get_one::<Shell>("shell").unwrap();
        generate(*shell, &mut build_cli(), "gf", &mut io::stdout());
    })
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clap v3 `App::new()` | clap v4 `Command::new()` | clap 4.0 (2022) | `App` renamed to `Command` — docs.rs shows v4 API |
| `structopt` derive | clap derive (`#[derive(Parser)]`) | clap 3+ | structopt merged into clap; use clap directly |
| clap v3 `visible_alias` | clap v4 `visible_alias` (same method) | No change | API stable across v3→v4 |

**Note:** Current clap on crates.io is 4.5.x. The project should add `clap = { version = "4", features = ["derive"] }` to Cargo.toml.

---

## Open Questions

1. **`fj pr create --title` format**
   - What we know: `fj pr create --repo <REPO> --base <BASE> --head <HEAD> [TITLE] --body <BODY>` — title appears positional in one guide
   - What's unclear: Is `--title` also accepted? Is title purely positional?
   - Recommendation: Treat as `--title` canonical flag → pass `--title <value>` to fj. If fj rejects it, fall back to positional. Verify with `fj pr create --help` during implementation.

2. **`fj pr create --draft` support**
   - What we know: Not documented in available sources
   - What's unclear: Whether fj supports draft PRs
   - Recommendation: Attempt passthrough; document in adapter as unverified.

3. **`fj auth` exact subcommand names**
   - What we know: `auth add-key`, `auth list`, `auth logout` from one 2026-02 blog post (MEDIUM confidence)
   - What's unclear: Whether `auth login` exists as a shorthand; exact flag names
   - Recommendation: Verify with `fj auth --help` during implementation; use `auth add-key` mapping as working assumption.

4. **`tea pulls view` without number**
   - What we know: tea requires specifying which PR to view in most documented examples
   - What's unclear: Whether `tea pulls view` with no args finds the current-branch PR
   - Recommendation: Delegate and let tea surface its own error. Document behavior in PR-06 implementation note.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in + assert_cmd 2.x + predicates 3.x |
| Config file | none (cargo test discovers automatically) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-08 | `r` aliases `repo`, `a` aliases `auth` in help | unit | `cargo test --lib cmd::tests::test_aliases_visible` | ❌ Wave 0 |
| CORE-09 | `gf mr create` routes to pr create handler | unit | `cargo test --lib cmd::tests::test_mr_alias_routes_to_pr` | ❌ Wave 0 |
| CORE-10 | `c`/`v`/`f`/`l`/`s` verb aliases work | unit | `cargo test --lib cmd::tests::test_verb_aliases` | ❌ Wave 0 |
| CORE-11 | Aliases appear in --help text | unit | `cargo test --lib cmd::tests::test_help_contains_aliases` | ❌ Wave 0 |
| CORE-12 | Completions include aliases | unit | `cargo test --lib cmd::tests::test_completions_include_aliases` | ❌ Wave 0 |
| PR-01 | `gf pr create --title X` produces correct args for each forge | unit | `cargo test --lib adapter::tests::test_pr_create_github` | ❌ Wave 0 |
| PR-02 | `--body` → `--description` for glab, `--base` → `--target-branch` | unit | `cargo test --lib adapter::tests::test_pr_create_glab_flags` | ❌ Wave 0 |
| PR-03 | `pr` → `mr` (glab), `pr` → `pulls` (tea) | unit | `cargo test --lib adapter::tests::test_subcommand_translation` | ❌ Wave 0 |
| PR-04 | Unrecognized flags pass through unchanged | unit | `cargo test --lib adapter::tests::test_passthrough_flags` | ❌ Wave 0 |
| PR-05 | `gf pr view` no number delegates correctly | integration | `cargo test --test integration_test test_pr_view_no_number` | ❌ Wave 0 |
| REPO-02 | `--private` → `--visibility private` for glab | unit | `cargo test --lib adapter::tests::test_repo_create_glab_visibility` | ❌ Wave 0 |
| AUTH-01 | `auth login` → `tea logins add` for gitea | unit | `cargo test --lib adapter::tests::test_auth_login_tea_translation` | ❌ Wave 0 |
| AUTH-02 | `auth logout` → `tea logins rm` for gitea | unit | `cargo test --lib adapter::tests::test_auth_logout_tea_translation` | ❌ Wave 0 |
| AUTH-03 | `auth status` → `tea logins ls` for gitea | unit | `cargo test --lib adapter::tests::test_auth_status_tea_translation` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/cmd/mod.rs` — clap Command tree (covers CORE-08 through CORE-12)
- [ ] `src/adapter.rs` — flag translation with unit tests (covers PR-01 through PR-04, REPO-01 through REPO-03, AUTH-01 through AUTH-03)
- [ ] `Cargo.toml` — add `clap = { version = "4", features = ["derive"] }` and `clap_complete = "4"` to `[dependencies]`

---

## Sources

### Primary (HIGH confidence)
- https://docs.rs/clap/latest/clap/struct.Command.html — `visible_alias`, `visible_aliases` methods verified on clap 4.6.0
- https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md — tea `pulls create` flags, `logins` subcommand structure
- https://docs.gitlab.com/cli/auth/login/ — glab `auth login` flags including `--hostname`

### Secondary (MEDIUM confidence)
- https://blog.n-daisuke897.com/posts/2026-02-01-operating-self-hosted-forgejo-via-cli-a-forgejo-cli-guide-2/ — fj `pr create` flags, fj `auth` subcommand names (2026-02 post, single source)
- https://crates.io/crates/clap_complete — clap_complete current version (4.5.66)

### Tertiary (LOW confidence)
- WebSearch synthesis on fj `--draft` support — unverified, single source
- fj `auth login` → `auth add-key` mapping — MEDIUM (one post, verify with `--help`)

---

## Metadata

**Confidence breakdown:**
- Standard stack (clap): HIGH — docs.rs verified, clap 4 well-established
- Architecture (ForgeAdapter pattern): HIGH — follows locked decisions and existing runner.rs pattern
- Pitfalls: HIGH — multi-word alias pitfall verified by clap behavior; tea auth structure verified via docs
- Forge flag tables: MEDIUM — gh/glab/tea HIGH; fj MEDIUM (single blog source)
- fj auth mapping: MEDIUM — needs verification against live `fj auth --help`

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable libraries; fj CLI may update sooner — verify fj flags at implementation time)
