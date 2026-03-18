# Phase 6: Browse Enhancements - Research

**Researched:** 2026-03-17
**Domain:** Rust CLI — URL fragment construction, Clap argument parsing, module refactoring
**Confidence:** HIGH

## Summary

Phase 6 is a focused, low-risk Rust refactor with two independent work items. The codebase is already well-understood from prior phases. No new dependencies are needed.

Item 1 (BROWSE-01): Add line-range parsing to `gf browse <file>:line` and `<file>:start-end`. The existing `build_file_url` function needs an optional line fragment appended. URL fragment formats differ per forge: GitHub/Gitea/Forgejo use `#L42-L55`, GitLab uses `#L42-55`. Parsing lives in `browse::run` before `normalize_path` is called — the colon suffix must be split from the file path first.

Item 2 (BROWSE-02): `browse::resolve_forge_type` (lines 131–139 of `src/browse/mod.rs`) duplicates the same four-host match arm as `forge::match_known_host` (lines 193–202 of `src/forge/mod.rs`). The fix is to make `forge::match_known_host` pub and call it from `browse::resolve_forge_type`, deleting the duplicate match. No behavior change — purely structural.

**Primary recommendation:** Implement deduplication (BROWSE-02) first as a pure refactor with no behavior change, then add line-range support (BROWSE-01) on top. This keeps each plan reviewable in isolation.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Input format: colon syntax only — `file.rs:42` and `file.rs:42-55`; no `L` prefix in input
- Output fragments:
  - GitHub/Gitea/Forgejo: `#L42` (single), `#L42-L55` (range)
  - GitLab: `#L42` (single), `#L42-55` (range)
- Error on invalid input: `:0`, reversed ranges like `:55-42`, non-numeric values
- Fail fast with clear error message consistent with existing `BrowseUrlConstructionFailed` style
- Deduplication target: `browse::resolve_forge_type` → replaced by a call to `forge::match_known_host`

### Claude's Discretion
- How to parse the colon suffix (regex vs manual split)
- Where to place the shared known-host function (forge module is the natural home — already is)
- Whether to add a new error variant or reuse `BrowseUrlConstructionFailed`

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| BROWSE-01 | User can deep-link to line ranges (`gf browse file.rs:42-55`) with correct per-forge fragment | `build_file_url` already has a `match forge` arm; fragment appended as optional suffix |
| BROWSE-02 | Known-host match table deduplicated between browse and forge detection modules | `forge::match_known_host` is private (line 193); making it `pub` lets `browse::resolve_forge_type` delegate to it |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Already in Cargo.toml | — | No new dependencies needed | Line parsing and string formatting use std only |

No new crates required. All work is within existing `std`, `thiserror`, and `clap` usage.

**Installation:** none

## Architecture Patterns

### Recommended Project Structure
No structural changes to directory layout. Changes are within:
```
src/
├── forge/mod.rs    # make match_known_host pub
└── browse/mod.rs   # parse line suffix; call forge::match_known_host; extend build_file_url
```

### Pattern 1: Colon-suffix parsing (manual split, no regex)

**What:** Split on the LAST colon in the file argument. Everything before the last colon is the path; everything after is the line spec.

**When to use:** Always for the `file` argument in `browse::run`.

**Why last colon:** Windows paths (`C:\...`) are not a concern for git paths, but the normalized path passed to forge URLs never contains colons. Still, splitting on the last colon is the conventional approach (matches how editors like `vim +42 file.rs` and compilers output `file.rs:42:col`).

**Example:**
```rust
// In browse::run, before normalize_path:
fn split_file_and_line(raw: &str) -> (&str, Option<&str>) {
    // Find last ':' — everything after is the optional line spec
    if let Some(pos) = raw.rfind(':') {
        let (path, rest) = (&raw[..pos], &raw[pos + 1..]);
        if !rest.is_empty() {
            return (path, Some(rest));
        }
    }
    (raw, None)
}
```

**Parsing the line spec:**
```rust
struct LineRange {
    start: u32,
    end: Option<u32>, // None = single line
}

fn parse_line_spec(spec: &str) -> Result<LineRange, GfError> {
    if let Some((start_str, end_str)) = spec.split_once('-') {
        let start: u32 = start_str.parse().map_err(|_| invalid_line_err(spec))?;
        let end: u32   = end_str.parse().map_err(|_| invalid_line_err(spec))?;
        if start == 0 || end == 0 {
            return Err(invalid_line_err(spec));
        }
        if end < start {
            return Err(GfError::BrowseUrlConstructionFailed(
                format!("line range '{spec}' is reversed — end must be >= start")
            ));
        }
        Ok(LineRange { start, end: Some(end) })
    } else {
        let n: u32 = spec.parse().map_err(|_| invalid_line_err(spec))?;
        if n == 0 {
            return Err(invalid_line_err(spec));
        }
        Ok(LineRange { start: n, end: None })
    }
}

fn invalid_line_err(spec: &str) -> GfError {
    GfError::BrowseUrlConstructionFailed(
        format!("invalid line spec '{spec}' — expected N or N-M (e.g. 42, 42-55)")
    )
}
```

### Pattern 2: Fragment construction per forge

**What:** Append fragment to the string returned by `build_file_url`. Fragment is computed before or passed in.

**Recommended approach:** Change `build_file_url` to accept `line_range: Option<&LineRange>` and append the fragment inside the function, inside the existing `match forge` block.

```rust
// Fragment per forge
fn line_fragment(forge: &ForgeType, lr: &LineRange) -> String {
    match forge {
        ForgeType::Github | ForgeType::Gitea | ForgeType::Forgejo => {
            match lr.end {
                None      => format!("#L{}", lr.start),
                Some(end) => format!("#L{}-L{}", lr.start, end),
            }
        }
        ForgeType::Gitlab => {
            match lr.end {
                None      => format!("#L{}", lr.start),
                Some(end) => format!("#L{}-{}", lr.start, end),
            }
        }
    }
}
```

### Pattern 3: Known-host deduplication (BROWSE-02)

**What:** Make `forge::match_known_host` pub, then replace the `match host { ... }` block inside `browse::resolve_forge_type` with a call to it.

**Current state:**
- `forge::match_known_host` is a private `fn` at line 193 of `src/forge/mod.rs`
- `browse::resolve_forge_type` has an identical inline match at lines 131–139 of `src/browse/mod.rs`

**After change:**
```rust
// forge/mod.rs — change `fn` to `pub fn`
pub fn match_known_host(host: &str) -> Result<ForgeType, GfError> { ... }

// browse/mod.rs — replace inline match with delegation
fn resolve_forge_type(host: &str) -> Result<ForgeType, GfError> {
    if let Some(forge_type) = config_lookup(host)? {
        return Ok(forge_type);
    }
    forge::match_known_host(host)
}
```

No behavior change — same four hosts, same error variant.

### Anti-Patterns to Avoid

- **Parsing with regex:** The line spec (`42` or `42-55`) is trivial to parse with `split_once('-')` and `str::parse::<u32>()`. No regex crate needed.
- **Silent correction of reversed ranges:** The decision is to fail fast. Do not swap start/end silently.
- **Adding a new error variant:** `BrowseUrlConstructionFailed(String)` carries a message; use it for all line-parse errors to stay consistent with existing browse error style.
- **Appending fragment outside `build_file_url`:** Keeping it inside the function keeps all URL construction in one place and makes testing clean.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| u32 parsing | Custom digit parser | `str::parse::<u32>()` | Already in std; handles overflow |
| Fragment differences | Single universal fragment format | Per-forge match arm | GitLab's `#L42-55` vs GitHub's `#L42-L55` are genuinely different |

## Common Pitfalls

### Pitfall 1: Splitting on the wrong colon

**What goes wrong:** `src/main.rs:42` has one colon — trivial. But `C:/Users/foo/bar.rs:42` (Windows) or a path like `./dir:with:colons/file.rs:42` could produce wrong splits.

**Why it happens:** `split(':')` takes the first colon; `rfind(':')` takes the last.

**How to avoid:** Use `rfind(':')` to split on the last colon. Git paths are Unix-style and don't contain colons, so this is safe for all practical inputs.

**Warning signs:** Test with `src/foo.rs:42` and verify path is `src/foo.rs`, not `src/foo.rs` with no spec.

### Pitfall 2: Line number 0

**What goes wrong:** `:0` should error but `u32` parses `0` successfully.

**How to avoid:** After parsing, explicitly check `n == 0` and return `BrowseUrlConstructionFailed`.

### Pitfall 3: build_file_url signature change breaks tests

**What goes wrong:** Existing tests call `build_file_url` with the current 7-argument signature. Adding `line_range: Option<&LineRange>` as an 8th parameter breaks every existing test call site.

**How to avoid:** Update all existing test calls to pass `None`. The change is mechanical — grep for `build_file_url(` and add `, None` before the closing paren.

### Pitfall 4: match_known_host visibility change breaks nothing but needs pub use check

**What goes wrong:** Making `match_known_host` pub exposes it in `forge`'s public API. This is intentional and correct. Verify there is no `pub use` re-export in `src/lib.rs` that would accidentally expose it at crate root if that's not desired.

**How to avoid:** Check `src/lib.rs` — currently it has no re-exports of forge internals, so making it `pub` is safe.

## Code Examples

### Full browse::run flow with line parsing

```rust
// Source: derived from src/browse/mod.rs (lines 32-38)
let file_arg = matches.get_one::<String>("file").map(|s| s.as_str());
let url = if let Some(raw_file) = file_arg {
    let (path_part, line_spec) = split_file_and_line(raw_file);
    let line_range = line_spec.map(parse_line_spec).transpose()?;
    let normalized = normalize_path(path_part)?;
    build_file_url(&forge_type, &host, &owner, &repo, &git_ref, is_sha, &normalized, line_range.as_ref())
} else {
    build_repo_url(&forge_type, &host, &owner, &repo, &git_ref)
};
```

### Updated build_file_url signature

```rust
pub fn build_file_url(
    forge: &ForgeType,
    host: &str,
    owner: &str,
    repo: &str,
    git_ref: &str,
    is_sha: bool,
    path: &str,
    line_range: Option<&LineRange>,
) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    let fragment = line_range.map(|lr| line_fragment(forge, lr)).unwrap_or_default();
    match forge {
        ForgeType::Github => format!("{base}/blob/{git_ref}/{path}{fragment}"),
        ForgeType::Gitlab => format!("{base}/-/blob/{git_ref}/{path}{fragment}"),
        ForgeType::Gitea | ForgeType::Forgejo => {
            if is_sha {
                format!("{base}/src/commit/{git_ref}/{path}{fragment}")
            } else {
                format!("{base}/src/branch/{git_ref}/{path}{fragment}")
            }
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Duplicate known-host match in browse | Single `pub fn match_known_host` in forge module | Single source of truth — adding a new public host requires one edit |
| `build_file_url` returns path only | `build_file_url` accepts optional `LineRange`, appends fragment | All URL construction stays in one function |

## Open Questions

1. **Gitea ROOT_URL subpath behavior**
   - What we know: State.md flags this as unresolved: "Gitea ROOT_URL subpath browse behavior unresolved — flag for edge case testing"
   - What's unclear: If Gitea is deployed at `https://host/gitea/`, the current `build_file_url` produces `https://host/owner/repo/...` which is wrong. This pre-existed Phase 6 and is out of scope.
   - Recommendation: Add a code comment in `build_file_url` noting the subpath limitation. Do not block Phase 6 on it.

2. **`LineRange` struct location**
   - What we know: It's only used by browse module functions.
   - Recommendation: Define it in `src/browse/mod.rs`. No need to put it in a shared module for Phase 6.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — standard `#[cfg(test)]` modules |
| Quick run command | `cargo test -p gf 2>&1` |
| Full suite command | `cargo test 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BROWSE-01 | Single line: `file.rs:42` produces `#L42` for all four forges | unit | `cargo test test_build_file_url_with_line` | ❌ Wave 0 |
| BROWSE-01 | Range: `file.rs:42-55` produces `#L42-L55` for GitHub/Gitea/Forgejo | unit | `cargo test test_build_file_url_with_range_github` | ❌ Wave 0 |
| BROWSE-01 | Range: `file.rs:42-55` produces `#L42-55` for GitLab | unit | `cargo test test_build_file_url_with_range_gitlab` | ❌ Wave 0 |
| BROWSE-01 | Error on `:0` | unit | `cargo test test_parse_line_spec_zero_errors` | ❌ Wave 0 |
| BROWSE-01 | Error on reversed range `:55-42` | unit | `cargo test test_parse_line_spec_reversed_errors` | ❌ Wave 0 |
| BROWSE-01 | Error on non-numeric | unit | `cargo test test_parse_line_spec_non_numeric_errors` | ❌ Wave 0 |
| BROWSE-02 | `browse::resolve_forge_type` delegates to `forge::match_known_host` | unit | `cargo test test_resolve_forge_type` | ✅ existing (behavior preserved) |

### Sampling Rate
- **Per task commit:** `cargo test 2>&1`
- **Per wave merge:** `cargo test 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Tests for `parse_line_spec` (valid, zero, reversed, non-numeric) — add to `src/browse/mod.rs` test module
- [ ] Tests for `build_file_url` with `Some(LineRange)` for all four forges — add to `src/browse/mod.rs` test module
- [ ] Tests for `split_file_and_line` helper — add to `src/browse/mod.rs` test module

No new test files needed — all tests go in the existing `#[cfg(test)]` block in `src/browse/mod.rs`.

## Sources

### Primary (HIGH confidence)
- Direct source code read: `src/browse/mod.rs` — full implementation of current browse command
- Direct source code read: `src/forge/mod.rs` — `match_known_host` at line 193, `config_lookup` at line 180
- Direct source code read: `src/error.rs` — `GfError` variants, `BrowseUrlConstructionFailed` at line 43
- `.planning/phases/06-browse-enhancements/06-CONTEXT.md` — locked decisions for this phase

### Secondary (MEDIUM confidence)
- GitHub line anchor format `#L42`, `#L42-L55`: observable from any GitHub file view URL
- GitLab line anchor format `#L42`, `#L42-55`: observable from any GitLab file view URL
- Gitea/Forgejo line anchor format `#L42`, `#L42-L55`: matches GitHub behavior (Gitea/Forgejo use GitHub-compatible fragment syntax)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all existing Rust std
- Architecture: HIGH — code was read directly; patterns derived from existing codebase conventions
- Pitfalls: HIGH — identified from direct code inspection (duplicate match arms, signature change, zero check)

**Research date:** 2026-03-17
**Valid until:** 2026-06-17 (stable domain — Rust std, forge URL formats are stable)
