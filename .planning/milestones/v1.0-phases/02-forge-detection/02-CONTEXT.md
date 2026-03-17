# Phase 2: Forge Detection - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-detect which forge a git repo lives on, given its remote URL. Covers: HTTPS and SSH URL parsing, `--remote` override, known public host matching, user-supplied config for self-hosted instances, and clear errors when detection fails. Does not cover command routing, flag normalization, or any forge CLI invocation (Phase 3+).

</domain>

<decisions>
## Implementation Decisions

### Detection priority order
- Config → URL match → error (no auth probing)
- Config is checked first and always wins — even for known public hosts (e.g., a config entry for `github.com` overrides built-in detection)
- This enables mirror/proxy scenarios and makes config the single authority

### Known public hosts (URL match)
- Match remote domain against four built-in entries: `github.com` → gh, `gitlab.com` → glab, `gitea.com` → tea, `codeberg.org` → fj
- Supports both HTTPS (`https://github.com/owner/repo.git`) and SSH SCP-style (`git@github.com:owner/repo.git`) remote URL formats

### Self-hosted forge detection (CORE-04 dropped)
- Auth probing (`gh auth status`, `glab auth status`, etc.) is dropped — these commands don't provide reliable host-to-forge mapping
- Self-hosted forges require a config entry; unknown domains that aren't in config produce a detection failure error
- This is a deliberate simplification: the error message shows exactly what to add

### Config file format (`~/.config/gf/config.toml`)
- Array of `[[forge]]` inline tables — extensible for future per-forge fields (e.g., CLI path override)
- Valid `type` values: `github`, `gitlab`, `gitea`, `forgejo` (full forge names, not CLI binary names)
- Config is parsed/validated on use only — no startup overhead for repos on known public hosts
- Example:
  ```toml
  [[forge]]
  domain = "gitlab.mycompany.com"
  type = "gitlab"

  [[forge]]
  domain = "git.internal.io"
  type = "forgejo"
  ```

### `--remote` flag (CORE-02)
- User can pass `--remote <name>` to use a different remote instead of `origin`
- Detection logic is identical — just uses the specified remote's URL instead

### Detection failure errors
- **Unknown domain (no config entry):** Detailed error — show the domain, list all supported forge types, and include a copy-pasteable TOML snippet. `forgejo` is listed first in the type comment:
  ```
  Could not detect forge for: git.internal.io

  Supported forges: github, gitlab, gitea, forgejo

  Add a mapping to ~/.config/gf/config.toml:
    [[forge]]
    domain = "git.internal.io"
    type = "forgejo"  # or github, gitlab, gitea
  ```
- **Not a git repo:** Distinct error — `not a git repository (or any parent directory)`
- **No origin remote:** Distinct error — `no remote named 'origin' — use --remote to specify one`
- All errors follow Phase 1 style: stderr, plain text, no prefix, no ANSI color

### Claude's Discretion
- Rust crate for TOML parsing (`toml` crate is the obvious choice)
- Rust crate or stdlib for git remote URL parsing
- Module structure for the detector (`forge/mod.rs`, `detect.rs`, etc.)
- Exact `git remote get-url` invocation vs parsing `.git/config` directly

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CORE-01 through CORE-05 (forge detection requirements; note CORE-04 is dropped per context decisions above)

### Project context
- `.planning/PROJECT.md` — Tech stack (Rust), constraints (no forge API calls), key decisions

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/error.rs` — `GfError` enum with `thiserror`. Phase 2 adds new variants: `ForgeNotDetected`, `NotAGitRepo`, `NoRemote`, `ConfigParseError`
- `src/runner.rs` — `run(cli, args)` function. Phase 2 doesn't call this directly but the module structure it establishes informs where detector code lives

### Established Patterns
- Error display: two-line format, no prefix, stderr only (set in Phase 1)
- `which::which()` used for PATH checking — may also be useful for confirming forge CLIs exist during detection
- `cfg(unix)` / `cfg(windows)` gating pattern is established

### Integration Points
- `src/main.rs` currently takes the CLI name as the first arg (placeholder). Phase 2 replaces this with forge detection — main calls `detect_forge(remote)` to get the CLI name, then passes it to `runner::run()`
- New module needed: `src/forge/` or `src/detect.rs` — forge detection logic lives here

</code_context>

<specifics>
## Specific Ideas

- Config format was chosen as `[[forge]]` array specifically to allow future per-forge fields (e.g., `cli_path = "/usr/local/bin/glab"` for non-standard install paths)
- Forgejo is listed first in detection failure error type comment — this is intentional (codeberg.org is a major Forgejo instance)

</specifics>

<deferred>
## Deferred Ideas

- `gf config add-forge <domain> <type>` command — dropped; the detection failure error message with copy-pasteable TOML is sufficient

</deferred>

---

*Phase: 02-forge-detection*
*Context gathered: 2026-03-16*
