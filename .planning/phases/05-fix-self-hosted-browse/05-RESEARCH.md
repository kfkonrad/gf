# Phase 5: Fix Self-Hosted Browse Forge Detection — Research

**Researched:** 2026-03-16
**Domain:** Rust module visibility, forge detection plumbing
**Confidence:** HIGH

## Summary

This is a surgical bug-fix phase. The problem is precisely diagnosed in the v1.0 audit (FINDING-02): `browse::resolve_forge_type()` duplicates the known-host match table from `forge::match_known_host()` but does not call `forge::config_lookup()`. A user with a self-hosted domain in `~/.config/gf/config.toml` (e.g. `git.mycompany.com`) gets `GfError::ForgeNotDetected` from `gf browse`, even though `gf pr` and every other command consult the config first.

The fix requires exactly one visibility change (`forge::config_lookup` from `fn` to `pub fn`) and a two-line call site addition in `browse::resolve_forge_type`. No new dependencies, no new error variants, no API changes elsewhere. The audit note in `browse/mod.rs` line 138–142 even pre-describes the fix verbatim.

All existing browse unit tests and integration tests continue to pass because the public-forge code path (known-host match) is unchanged. The only behavior change is that self-hosted hosts now succeed when a config entry exists instead of always returning `ForgeNotDetected`.

**Primary recommendation:** Make `forge::config_lookup` pub, call it from `browse::resolve_forge_type` before the `ForgeNotDetected` return, and add unit tests covering the self-hosted path.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CORE-05 | User can configure domain-to-forge-type mappings in `~/.config/gf/config.toml` | `forge::config_lookup()` already implements this; browse just needs to call it |
| BROWSE-01 | `gf browse` opens current repo in browser at correct forge URL | Currently broken for self-hosted; calling config_lookup fixes the URL construction path |
| BROWSE-02 | `gf browse` uses current branch by default | Unaffected by fix; branch resolution is independent of forge type resolution |
| BROWSE-03 | `gf browse <file>` opens specific file in browser | Unaffected by fix; file URL construction uses forge type, which will be resolved correctly once forge type is found |
| BROWSE-04 | `--branch <name>` override | Unaffected by fix; same as BROWSE-02 |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| toml | 0.8 | Config file parsing | Already a project dependency; same crate used in forge::load_config |
| thiserror | (existing) | Error derivation | Already a project dependency; no new error variants needed |

No new dependencies needed. This fix is purely internal wiring.

**Installation:** No new packages — existing Cargo.toml is sufficient.

## Architecture Patterns

### Recommended Project Structure

No structural change. All modifications are within existing files:

```
src/
├── forge/mod.rs     # Change config_lookup visibility: fn → pub fn
└── browse/mod.rs    # Call forge::config_lookup() in resolve_forge_type()
```

### Pattern 1: Config-First Forge Resolution

This is the existing pattern in `forge::detect()` (lines 77–84 of `src/forge/mod.rs`):

```rust
// Source: src/forge/mod.rs — existing detect() function
pub fn detect(remote: &str) -> Result<ForgeType, GfError> {
    let url = get_remote_url(remote)?;
    let host = parse_host(&url)?;
    if let Some(forge) = config_lookup(&host)? {  // config first
        return Ok(forge);
    }
    match_known_host(&host)  // known hosts second
}
```

`browse::resolve_forge_type()` must follow the same priority: config first, known hosts second. The function signature and return type stay the same.

### Pattern 2: After fix — resolve_forge_type

```rust
// src/browse/mod.rs — resolve_forge_type() after fix
fn resolve_forge_type(host: &str) -> Result<ForgeType, GfError> {
    // Config lookup first (CORE-05) — mirrors forge::detect() priority
    if let Some(forge_type) = crate::forge::config_lookup(host)? {
        return Ok(forge_type);
    }
    // Known public forges second
    match host {
        "github.com" => Ok(ForgeType::Github),
        "gitlab.com" => Ok(ForgeType::Gitlab),
        "gitea.com" => Ok(ForgeType::Gitea),
        "codeberg.org" => Ok(ForgeType::Forgejo),
        other => Err(GfError::ForgeNotDetected {
            domain: other.to_string(),
        }),
    }
}
```

### Anti-Patterns to Avoid

- **Calling `forge::detect()` from browse:** `forge::detect()` re-runs `git remote get-url` and host parsing, but `browse::run()` already has `host` in hand. Call `config_lookup(host)` directly — don't re-enter detect().
- **Making `match_known_host` pub:** Only `config_lookup` needs to be pub. `match_known_host` stays private; browse already has its own inline copy of the same match.
- **Moving the known-host table to forge module:** Out of scope for this fix. The inline duplication is acceptable tech debt; de-duplication would require changing `match_known_host` to pub and updating browse to call it. The audit says to fix the config_lookup gap; leave the duplication as-is.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config file reading | Custom TOML loader | `forge::config_lookup(host)` | Already implemented, tested, handles absent-file case |
| Self-hosted forge detection | New function | Expose existing `forge::config_lookup` | Tested in 6 existing unit tests |

**Key insight:** The entire fix is a visibility change (`fn` → `pub fn`) plus two lines at the call site. No new logic.

## Common Pitfalls

### Pitfall 1: Exposing the wrong function
**What goes wrong:** Making `forge::detect()` or `forge::match_known_host()` pub instead of `forge::config_lookup()`.
**Why it happens:** `detect()` looks like the right entry point because it does config lookup + known hosts.
**How to avoid:** `detect()` takes a remote name and re-runs git commands. Browse already has the host string — call `config_lookup(host)` directly, not `detect(remote)`.

### Pitfall 2: Error propagation in config_lookup call
**What goes wrong:** Using `?` with `config_lookup` but not handling the `Ok(None)` case.
**Why it happens:** `config_lookup` returns `Result<Option<ForgeType>, GfError>`. `?` only propagates the `Err` case; the `None` case falls through to the known-host match.
**How to avoid:** Use the `if let Some(forge_type) = crate::forge::config_lookup(host)?` pattern shown above. The `?` propagates `Err(ConfigParseError)`, and `None` continues to the match block.

### Pitfall 3: HOME environment isolation in tests
**What goes wrong:** Unit tests that call `config_lookup` may read the developer's real `~/.config/gf/config.toml`.
**Why it happens:** `config_lookup` reads `$HOME`.
**How to avoid:** Override `HOME` to `/tmp` in any test that calls `config_lookup` with a real domain, as the existing forge tests already do (see `test_config_lookup_absent_config_is_ok_none` in `forge/mod.rs` line 341).

### Pitfall 4: Test isolation with set_var
**What goes wrong:** `std::env::set_var` is process-wide; if tests run in parallel, one test's `HOME=/tmp` pollutes another test.
**Why it happens:** Cargo test runs unit tests in threads within the same process.
**How to avoid:** Follow the existing project pattern — use `unsafe { std::env::set_var("HOME", ...) }`. Tests that use set_var should be marked or run with `-- --test-threads=1` if ordering becomes an issue. The project already has precedent for this approach.

## Code Examples

### config_lookup visibility change
```rust
// Source: src/forge/mod.rs — change fn to pub fn
pub fn config_lookup(host: &str) -> Result<Option<ForgeType>, GfError> {
    // ... body unchanged
}
```

### New unit test for browse self-hosted path
```rust
// src/browse/mod.rs — new test
#[test]
fn test_resolve_forge_type_self_hosted_via_config() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let config_dir = tmp.path().join(".config/gf");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.toml");
    let mut f = std::fs::File::create(&config_path).unwrap();
    writeln!(f, "[[forge]]\ndomain = \"git.mycompany.com\"\ntype = \"gitlab\"").unwrap();
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let result = resolve_forge_type("git.mycompany.com").unwrap();
    assert_eq!(result, ForgeType::Gitlab);
}

#[test]
fn test_resolve_forge_type_self_hosted_unknown_still_errors() {
    unsafe { std::env::set_var("HOME", "/tmp") };
    let result = resolve_forge_type("unknown.example.com");
    assert!(matches!(result, Err(GfError::ForgeNotDetected { .. })));
}
```

Note: `tempfile` is not currently in Cargo.toml. The test can also use a hard-coded temp path under `/tmp/gf-test-phase5/` that is created and cleaned up inline, matching the pattern from Phase 2 integration tests. Alternatively, the HOME override to `/tmp` and direct file write to `/tmp/.config/gf/config.toml` suffices for a unit test.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none (Cargo.toml [dev-dependencies]) |
| Quick run command | `cargo test -p gf browse` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-05 (browse gap) | `resolve_forge_type` finds self-hosted domain via config | unit | `cargo test browse::tests::test_resolve_forge_type_self_hosted_via_config` | ❌ Wave 0 |
| CORE-05 (browse gap) | `resolve_forge_type` returns ForgeNotDetected when no config entry and not known host | unit | `cargo test browse::tests::test_resolve_forge_type_self_hosted_unknown_still_errors` | ❌ Wave 0 |
| BROWSE-01/02/03/04 | All existing browse tests still pass | unit | `cargo test browse` | ✅ existing |
| CORE-05 | `forge::config_lookup` now pub — still passes its own tests | unit | `cargo test forge` | ✅ existing |

### Sampling Rate
- **Per task commit:** `cargo test browse`
- **Per wave merge:** `cargo test`
- **Phase gate:** `cargo test` full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/browse/mod.rs` — add `test_resolve_forge_type_self_hosted_via_config` (covers BROWSE-01–04 self-hosted path)
- [ ] `src/browse/mod.rs` — add `test_resolve_forge_type_self_hosted_unknown_still_errors` (regression for ForgeNotDetected when no config entry)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `resolve_forge_type` only checks known-host table | After fix: checks config first, then known-host table | Phase 5 | Self-hosted forge users can run `gf browse` |

**Deprecated/outdated:**
- The comment block at `browse/mod.rs` lines 138–142 ("NOTE: Self-hosted instances need...") should be removed after the fix is applied.

## Open Questions

1. **tempfile crate vs. inline temp dir**
   - What we know: The project doesn't use `tempfile` today. Phase 2 integration tests create temp dirs manually using `std::fs::create_dir_all` + `/tmp/...` paths.
   - What's unclear: Whether the planner should add `tempfile` as a dev-dependency or use inline temp path creation.
   - Recommendation: Follow the existing project pattern — create temp dirs inline under `/tmp/gf-test-phase5/` without adding a new dependency. Simpler, consistent.

## Sources

### Primary (HIGH confidence)
- `src/browse/mod.rs` — current `resolve_forge_type` implementation (lines 125–142), current tests
- `src/forge/mod.rs` — `config_lookup` signature and implementation (lines 180–190), `detect()` priority pattern (lines 77–84)
- `.planning/v1.0-MILESTONE-AUDIT.md` — FINDING-02 precise diagnosis and prescribed fix
- `src/error.rs` — `GfError` variants (no changes needed)

### Secondary (MEDIUM confidence)
- `.planning/STATE.md` — Phase 04-browse decisions confirm why `resolve_forge_type` was written without calling `config_lookup` (avoidance of making private functions public)

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies, existing codebase fully read
- Architecture: HIGH — fix is prescribed verbatim in audit, code is fully read
- Pitfalls: HIGH — based on actual code structure and existing test patterns in same repo

**Research date:** 2026-03-16
**Valid until:** Indefinite — this is internal Rust code with no external dependencies changing
