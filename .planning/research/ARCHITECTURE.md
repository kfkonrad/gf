# Architecture Research

**Domain:** Rust CLI — unified git forge wrapper (v1.1 additions to existing v1.0 codebase)
**Researched:** 2026-03-17
**Confidence:** HIGH — based on direct source code analysis of the v1.0 implementation

## Standard Architecture

### System Overview (v1.0 Actual)

```
┌─────────────────────────────────────────────────────────────────────┐
│                          main.rs (entry point)                       │
│  1. cmd::build_cli() → clap parse                                    │
│  2. Early intercepts: completions (no repo), browse (native)         │
│  3. forge::detect(remote) → ForgeType                                │
│  4. adapter::translate(forge, matches) → Vec<String>                 │
│  5. runner::run(forge.cli_name(), &args)                             │
└─────────────────────────────────────────────────────────────────────┘
         │                   │                   │
         ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────────┐   ┌──────────────┐
│  forge/      │   │  adapter/        │   │  browse/     │
│  mod.rs      │   │  mod.rs          │   │  mod.rs      │
│  - detect()  │   │  - translate()   │   │  - run()     │
│  - ForgeType │   │  pr.rs           │   │  Native URL  │
│  - config_   │   │  repo_auth.rs    │   │  construction│
│    lookup()  │   │                  │   │  No CLI      │
│  - parse_    │   │                  │   │  delegation  │
│    remote_   │   │                  │   │              │
│    parts()   │   │                  │   │              │
└──────────────┘   └──────────────────┘   └──────────────┘
                                │
                                ▼
                    ┌──────────────────────┐
                    │  runner::run()        │
                    │  Unix: exec() replaces│
                    │  process (zero-cost)  │
                    │  Windows: spawn+wait  │
                    └──────────────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              ▼                 ▼                   ▼
           gh CLI           glab CLI          tea / fj CLI
```

### Component Responsibilities

| Component | Responsibility | v1.1 Impact |
|-----------|----------------|-------------|
| `cmd/mod.rs` | Defines the entire clap command tree with aliases via build_cli() | Add subcommands: `pr list/merge/checkout/review`, `repo clone`, `issue` group |
| `forge/mod.rs` | ForgeType enum, detect(), config_lookup(), parse_remote_parts() | Optional: add CLI auth probe fallback for CORE-04 |
| `adapter/mod.rs` | Routes subcommand name to per-module translator function | Add `issue` dispatch arm; forward `repo clone` to repo_auth |
| `adapter/pr.rs` | Translates `gf pr *` matches into forge CLI args | Add translate_pr_list, translate_pr_merge, translate_pr_checkout, translate_pr_review; audit flag maps |
| `adapter/repo_auth.rs` | Translates `gf repo/auth *` matches into forge CLI args | Add translate_repo_clone; audit existing flag mappings |
| `adapter/issue.rs` | (NEW) Translates `gf issue *` matches into forge CLI args | New module, mirrors pr.rs structure |
| `browse/mod.rs` | Native URL construction and browser open | Add line-range parsing: `file.rs:42-55` → append `#L42-L55` fragment |
| `runner.rs` | exec() or spawn+wait the forge CLI binary | No changes expected |
| `error.rs` | GfError enum and install-hint display | Possibly add CliAuthProbeFailed for CORE-04 |

## Recommended Project Structure

```
src/
├── adapter/
│   ├── mod.rs          # MODIFIED: add issue dispatch arm
│   ├── pr.rs           # MODIFIED: add list, merge, checkout, review; audit flags
│   ├── repo_auth.rs    # MODIFIED: add clone translator; audit existing maps
│   └── issue.rs        # NEW: translate gf issue * → forge CLI args
├── browse/
│   └── mod.rs          # MODIFIED: parse line-range from file arg; build fragment
├── cmd/
│   └── mod.rs          # MODIFIED: add pr subcommands, repo clone, issue group
├── forge/
│   └── mod.rs          # MODIFIED (minimal): CORE-04 CLI auth probe as fallback tier
├── error.rs            # possibly add new error variant for CORE-04
├── lib.rs              # no change
├── main.rs             # no change
└── runner.rs           # no change
```

### Structure Rationale

- **adapter/issue.rs (new):** Issue commands warrant their own module following the existing pattern. pr.rs and repo_auth.rs are the templates. Issue commands have their own flag-translation complexity (labels, assignees, state filters) that will grow.
- **browse/mod.rs (modified in-place):** Line-range parsing is a small, contained addition to existing URL construction logic. No new module needed.
- **forge/mod.rs (minimal CORE-04 change):** CLI auth probing is a new detection strategy fitting inside detect() as an additional fallback tier. If it grows complex, extract to forge/probe.rs.
- **main.rs unchanged:** All new v1.1 commands follow the standard detect→translate→exec flow. No new early intercepts needed.

## Architectural Patterns

### Pattern 1: Subcommand Translator Function

**What:** Every new `gf` verb maps to a private function `translate_<subcommand>_<verb>(forge, cmd_name, matches) -> Vec<String>` inside the appropriate adapter module. The public function dispatches to these.

**When to use:** All new pr, repo, issue verbs follow this pattern exactly.

**Trade-offs:** Straightforward to add, test, and audit. Slight verbosity but isolates translation logic cleanly.

**Example for pr list:**
```rust
fn translate_pr_list(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Vec<String> {
    let mut args = vec![pr_cmd.to_string(), "list".to_string()];

    if let Some(state) = matches.get_one::<String>("state") {
        args.push("--state".to_string());
        args.push(state.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}
```

### Pattern 2: Subcommand Name Remapping

**What:** Each forge CLI uses different command names for the same concept. A `*_subcommand_name(forge)` function is the single source of truth for that mapping.

**When to use:** Required for every new top-level concept.

**Trade-offs:** Centralizes all divergence into one match block. Easy to audit. Adding a new forge requires only touching this function.

**Example for issues:**
```rust
fn issue_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Github  => "issue",
        ForgeType::Gitlab  => "issue",
        ForgeType::Gitea   => "issues",   // tea uses plural — verify against tea docs
        ForgeType::Forgejo => "issue",
    }
}
```

### Pattern 3: Early Intercept in main.rs (do NOT extend)

**What:** Commands that bypass the detect→translate→exec flow are handled before forge detection via explicit `if let Some(("name", sub))` arms in main.rs.

**When to use:** ONLY for truly native commands (browse) or non-repo commands (completions). All new v1.1 commands follow the standard flow.

**Trade-offs:** Keeps special cases visible. Do not add new intercepts — extending this pattern creates maintenance surface.

### Pattern 4: Passthrough for Unknown Flags

**What:** All translator functions collect unknown flags in the `extra` arg (defined with `last(true)` in clap) and append them verbatim to the output Vec.

**When to use:** Always — this is what makes gf a thin wrapper rather than a full abstraction layer.

**Trade-offs:** Users get the escape hatch they need for forge-specific flags not in gf's canonical set.

## Data Flow

### Standard Command Flow (all new v1.1 delegated commands)

```
User: gf pr list --state open

cmd::build_cli()
  → ArgMatches{ subcommand: "pr", sub: { subcommand: "list", state: "open" } }

forge::detect("origin")
  → git remote get-url origin → "git@github.com:alice/repo.git"
  → parse_host() → "github.com"
  → config_lookup() → None
  → match_known_host() → ForgeType::Github

adapter::translate(Github, matches)
  → adapter::pr::translate_pr(Github, pr_sub)
  → translate_pr_list(Github, "pr", list_sub)
  → Vec["pr", "list", "--state", "open"]

runner::run("gh", &["pr", "list", "--state", "open"])
  → exec() replaces process: gh pr list --state open
```

### Browse Line-Range Flow (native, no CLI delegation)

```
User: gf browse src/main.rs:42-55

browse::run(matches)
  → file_arg = "src/main.rs:42-55"
  → parse_line_range("src/main.rs:42-55")
      → path = "src/main.rs", start = 42, end = Some(55)
  → normalize_path("src/main.rs") → "src/main.rs"
  → build_file_url(..., "src/main.rs")
      → base URL: "https://github.com/alice/repo/blob/main/src/main.rs"
      → append_line_fragment(forge, 42, Some(55))
          GitHub/GitLab/Gitea/Forgejo: "#L42-L55"
  → url = "https://github.com/alice/repo/blob/main/src/main.rs#L42-L55"
  → println! + webbrowser::open()
```

### CORE-04 Self-Hosted Detection (optional extended detect())

```
forge::detect(remote)
  → get_remote_url() → url
  → parse_host() → "git.corp.com"
  → config_lookup("git.corp.com") → None (not in config)
  → match_known_host("git.corp.com") → Err(ForgeNotDetected)
  → [NEW FALLBACK] cli_auth_probe("git.corp.com")
      → try: gh auth status --hostname git.corp.com  (exit 0?) → Github
      → try: glab auth status --hostname git.corp.com (exit 0?) → Gitlab
      → ... (timeout fast — probes must not block)
      → all fail → Err(ForgeNotDetected { domain })
```

## Integration Points

### New Features → Existing Files

| New Feature | Files Modified | Files Created | Key Integration |
|-------------|----------------|---------------|-----------------|
| `pr list` | `cmd/mod.rs`, `adapter/pr.rs` | none | Add to build_pr(); add match arm in translate_pr() |
| `pr merge` | `cmd/mod.rs`, `adapter/pr.rs` | none | tea: `pulls merge`; glab: `mr merge`; verify fj |
| `pr checkout` | `cmd/mod.rs`, `adapter/pr.rs` | none | gh: `pr checkout`; glab: `mr checkout`; tea: needs verification |
| `pr review` | `cmd/mod.rs`, `adapter/pr.rs` | none | glab: `mr approve`; tea may not support — passthrough or error |
| `repo clone` | `cmd/mod.rs`, `adapter/repo_auth.rs` | none | All CLIs: `repo clone <repo>`; flag audit needed for --depth etc. |
| `issue` group | `cmd/mod.rs`, `adapter/mod.rs` | `adapter/issue.rs` | New dispatch arm in adapter::translate(); new module mirrors pr.rs |
| Browse line-range | `browse/mod.rs` | none | parse_line_range() helper + fragment appended in build_file_url() |
| CORE-04 probe | `forge/mod.rs` | possibly `forge/probe.rs` | New fallback tier in detect() after match_known_host fails |
| Flag audit | `adapter/pr.rs`, `adapter/repo_auth.rs` | none | Systematic check of each forge CLI's current flag names |

### Internal Module Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `browse` ↔ `forge` | Direct calls: `forge::config_lookup()`, `forge::parse_remote_parts()` | Already coupled in v1.0; intentional for URL correctness |
| `adapter` ↔ `forge` | `ForgeType` value passed as argument | Adapter is forge-aware but does not import forge module internals |
| `main.rs` ↔ all | Orchestrates detect → translate → run pipeline | main.rs is the composition root; it is the only file that calls all three |
| `cmd` ↔ adapters/browse | One-way import: adapter tests use `cmd::build_cli()` as parse helper | cmd has no runtime deps on other modules |

### Known Duplication to Resolve

The v1.0 codebase has one notable structural issue: `browse/mod.rs` duplicates the known-host match table from `forge/mod.rs` (browse lines 131-139, forge lines 193-203). v1.1 is a good time to fix this by making `forge::match_known_host()` pub(crate) and calling it from browse's `resolve_forge_type()`.

## Build Order for v1.1

Dependencies within the v1.1 scope:

1. **Browse line-range** — self-contained modification to browse/mod.rs. No new dependencies. Build and ship first.
2. **Flag audit** — modify existing translator functions. No new modules. Build alongside browse.
3. **`pr list/merge/checkout/review`** — extend adapter/pr.rs and cmd/mod.rs. No new modules. Depends on: flag audit complete (clean baseline).
4. **`repo clone`** — extend adapter/repo_auth.rs and cmd/mod.rs. Parallel with step 3.
5. **`issue` group** — requires new adapter/issue.rs and cmd/mod.rs additions. Depends on: understanding the established pattern from steps 3-4.
6. **CORE-04 self-hosted probe** — modifies forge::detect(). Must be added last; it touches the detection pipeline that all other commands rely on.

## Anti-Patterns

### Anti-Pattern 1: Adding Early Intercepts for Delegated Commands

**What people do:** Add `if let Some(("pr", sub)) = matches.subcommand() { ... return; }` in main.rs for new commands that should use the standard flow.

**Why it's wrong:** Bypasses forge detection. Forces duplication of detection logic. The early intercept pattern is ONLY for browse (native) and completions (no repo needed).

**Do this instead:** Add the translator function in the appropriate adapter module, add the match arm in adapter::translate(), add the subcommand in cmd/mod.rs. main.rs stays unchanged.

### Anti-Pattern 2: Making CORE-04 Probe Blocking on Every Invocation

**What people do:** Call all four CLI auth probes on every `gf` invocation when the host is unknown, blocking until all probes time out.

**Why it's wrong:** Probing 4 CLIs on every command invocation adds unacceptable latency. This is why CORE-04 was deferred from v1.0.

**Do this instead:** Probe only when both config_lookup and match_known_host fail. Set tight timeouts on each probe subprocess. Consider caching the probe result in ~/.cache/gf/. Alternatively, make CORE-04 an explicit user action (`gf detect` command that writes to config) rather than fully automatic.

### Anti-Pattern 3: Embedding Line-Range as Separate Flags

**What people do:** Add `--line-start` and `--line-end` flags to build_browse() to handle line ranges separately.

**Why it's wrong:** The natural user syntax is `gf browse file.rs:42-55` (same convention as gh browse, editors, etc.). Separate flags break ergonomics and established convention.

**Do this instead:** Accept the colon syntax in the existing `file` positional arg. Parse `path:start[-end]` in browse::run() before calling normalize_path(). The `file` arg definition in cmd/mod.rs does not need to change.

### Anti-Pattern 4: Silently Ignoring Unsupported Commands per Forge

**What people do:** Return an empty Vec from a translator when a forge CLI does not support a canonical command (e.g., `tea` may not support `pr review`).

**Why it's wrong:** Silently produces `exec("tea", [])` which shows tea help text with no error. User has no idea why.

**Do this instead:** Return a meaningful error from the translator for commands that genuinely have no equivalent. Add a new GfError variant like `CommandNotSupported { forge, command }` and surface it before exec.

### Anti-Pattern 5: Duplicating the Known-Host Table

**What people do:** Copy the host→ForgeType match from forge/mod.rs into browse/mod.rs (this already happened in v1.0).

**Why it's wrong:** Two sources of truth. Adding a new known host requires changing both. Adding a new forge requires finding all copies.

**Do this instead:** In v1.1, expose `forge::match_known_host()` as `pub(crate)` and have browse::resolve_forge_type() call it instead of duplicating the match.

## Sources

- Direct source analysis of `/Users/derkev/tmp/gf-v2/src/` (v1.0, 2026-03-17)
- Confidence: HIGH — all claims derived from the actual source code

---
*Architecture research for: gf v1.1 — PR workflows, issues, clone, browse line-range, self-hosted detection*
*Researched: 2026-03-17*
