# Phase 3: Command Routing - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Add clap-based CLI structure, a ForgeAdapter translation layer, canonical flag normalization for `pr`/`repo`/`auth` commands, and alias resolution across all four forges. Users can run `gf pr create`, `gf repo view`, `gf auth login` (and all aliases) on any supported forge with canonical flags automatically translated to forge equivalents.

</domain>

<decisions>
## Implementation Decisions

### Flag normalization scope
- `pr` commands: normalize canonical flags (`--title`, `--body` → `--description` for glab, `--base` → `--target-branch` for glab, `--draft`, etc.)
- `repo` commands: also normalize — implement a **full canonical flag set** for `repo create` (name, description, visibility, homepage, etc.) mapped to each forge's equivalents
- `auth` commands: normalize any flags that differ across forges (researcher to identify divergences)
- Unrecognized flags: **pass through unchanged** to the underlying CLI (per PR-04) — no warnings, fully transparent

### Forgejo (`fj`) CLI
- **Fully implement** in Phase 3 — first-class support, not a stub
- Phase 3 researcher must **verify actual `fj pr create` flags** against Forgejo CLI source/docs before planning (flag names currently MEDIUM confidence per STATE.md)
- Fallback policy: if `fj` diverges significantly from gh/glab structure, normalize what matches and pass through the rest — same policy as other forges
- Do NOT raise a blocker or defer; ship best-effort with verified research

### Alias presentation
- Aliases appear **inline under the canonical command** in clap `--help`:
  ```
  pr create    Create a pull/merge request
               [aliases: c, mr create, mr c]
  pr view      View a pull/merge request [aliases: v]
  repo view    View repo info [aliases: r v]
  auth login   Authenticate [aliases: a l]
  ```
- `mr` is listed as an alias under `pr` at the top level — it does NOT appear as a separate top-level command
- `gf mr --help` and `gf mr create` work identically to their `pr` equivalents
- Shell completion scripts (CORE-12) also include all aliases

### `gf pr view` with no number
- **Delegate entirely to the underlying CLI** — call `gh pr view` / `glab mr view` with no number; they handle current-branch PR lookup natively
- gf stays transparent; underlying CLI errors surface through unchanged
- For `tea` and `fj`: researcher verifies whether current-branch lookup is supported; implement delegation if supported, document limitation if not

### Claude's Discretion
- Internal clap subcommand structure (e.g., how to model `mr` as an alias for `pr` in clap)
- ForgeAdapter trait vs data table vs translation function — design the translation layer
- Module layout for the router/adapter code
- Exact canonical flag set for `auth` commands (after researcher identifies divergences)
- Exact canonical flag set additions beyond the defined ones for `repo create`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CORE-08 through CORE-12 (aliases), PR-01 through PR-06, REPO-01 through REPO-03, AUTH-01 through AUTH-03

### Project context
- `.planning/PROJECT.md` — Tech stack (Rust, clap), constraints (no forge API calls), key decisions (normalize known flags + passthrough unknown)

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/forge/mod.rs` — `ForgeType` enum with `cli_name()` method. The ForgeAdapter/translation layer plugs into this — ForgeType is the dispatch key
- `src/error.rs` — `GfError` enum. Phase 3 may add new variants (e.g., `UnknownSubcommand`, `FlagTranslationError` if needed)
- `src/runner.rs` — `run(cli, args)` function. Phase 3 builds translated args and hands them to this

### Established Patterns
- Error display: stderr, plain text, no prefix, no ANSI color (Phase 1)
- `exec()` on Unix — gf replaces itself with child process (Phase 1)
- `ForgeType::cli_name()` is the single source of truth for CLI binary name

### Integration Points
- `src/main.rs` currently: detects forge → calls `runner::run(cli, remaining_args)`. Phase 3 replaces this with: detect forge → parse subcommand + flags via clap → translate flags via ForgeAdapter → call `runner::run(cli, translated_args)`
- New modules needed: command parsing (clap), flag translation table per forge
- `src/forge/mod.rs` gains translation logic (or a companion `src/adapter.rs`)

</code_context>

<specifics>
## Specific Ideas

- "gf stays transparent" — the exec model from Phase 1 is the mental model to preserve. ForgeAdapter's job is purely arg translation before exec, not a new layer of control
- Inline aliases in --help follows standard clap convention; user explicitly chose this over a separate alias section

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-command-routing*
*Context gathered: 2026-03-16*
