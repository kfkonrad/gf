# Phase 1: Foundation - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Subprocess delegation, error types, and CLI presence detection. The subprocess runner must be correct — future phases build on this without retrofitting. Covers: PATH detection for forge CLIs, human-readable error with install hint when CLI is missing, exact exit code propagation, signal re-raising, and inherited TTY for color/interactive prompts.

</domain>

<decisions>
## Implementation Decisions

### Install hints (CORE-06)
- When a forge CLI is not on PATH: show the CLI name, `brew install <cli>`, AND the official URL
- Format (two-line, no prefix):
  ```
  glab not found
  Install with: brew install glab
  Or see: https://gitlab.com/gitlab-org/cli
  ```
- Always use `brew install <cli>` for consistency — no per-platform detection
- Official URL is always included as fallback for edge cases

### Error output format
- gf's own errors go to stderr, plain text, no prefix or ANSI styling
- Child process stderr streams through directly (passthrough) — architecturally separate from gf's own errors
- No `gf:` prefix, no `error:` label, no color

### Signal handling (CORE-07)
- Totally transparent: re-raise all signals (SIGINT, SIGTERM, SIGHUP, etc.) on self after child exits
- Exit 130 on SIGINT (standard shell convention)
- No output from gf on signal — completely invisible to the user
- Same transparent re-raise behavior for all signals, not just SIGINT

### Subprocess execution model
- **Unix**: `exec()` — gf replaces itself with the child process (execvp). Perfect TTY inheritance, zero signal complexity for the common case
- **Windows**: `spawn()` — gf stays alive as parent, waits for child, propagates exit code. cfg-gated at compile time
- Cross-platform from day one: `#[cfg(unix)]` exec path, `#[cfg(windows)]` spawn path

### Claude's Discretion
- Exact Rust crate choices for exec/spawn (std::os::unix::process::CommandExt vs nix, etc.)
- Error type hierarchy design (thiserror, anyhow, or custom)
- Module structure within the crate

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CORE-06 (CLI not found error), CORE-07 (exit code + signal propagation)

### Project context
- `.planning/PROJECT.md` — Tech stack (Rust), constraints (no forge API calls), key decisions

No external specs — requirements are fully captured in decisions above and REQUIREMENTS.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None yet — Phase 1 establishes the patterns future phases follow

### Integration Points
- Phase 1 output: a subprocess runner module that Phase 2+ will call to invoke detected forge CLIs
- Phase 1 output: error types that Phase 2+ will use (forge not detected, CLI not found, etc.)

</code_context>

<specifics>
## Specific Ideas

- The exec model preview resonated: "gf disappears; the forge CLI becomes the process" — this is the mental model to preserve
- Cross-platform from the start: cfg-gate exec (Unix) vs spawn (Windows) at compile time, not runtime

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-16*
