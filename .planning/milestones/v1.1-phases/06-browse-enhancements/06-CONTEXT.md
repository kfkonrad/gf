# Phase 6: Browse Enhancements - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Add line-range deep-linking to `gf browse` (`file.rs:42-55`) with correct per-forge URL fragments, and deduplicate the known-host match table so browse and forge detection share a single source of truth.

</domain>

<decisions>
## Implementation Decisions

### Line-range input format
- Colon syntax only: `file.rs:42` (single line), `file.rs:42-55` (range)
- Matches grep/compiler output convention — no `L` prefix accepted in input
- Output fragments differ per forge:
  - GitHub/Gitea/Forgejo: `#L42-L55` (single: `#L42`)
  - GitLab: `#L42-55` (single: `#L42`)

### Edge case behavior
- Error on invalid input: `:0`, `:55-42` (reversed), non-numeric values
- Fail fast with clear error message — no silent correction
- Consistent with existing codebase error style (e.g., `BrowseUrlConstructionFailed`)

### Known-host deduplication
- `browse/mod.rs:resolve_forge_type` (lines 131-139) duplicates `forge/mod.rs:match_known_host` (line 193+)
- Consolidate to a single function used by both modules

### Claude's Discretion
- How to parse the colon suffix (regex vs manual split)
- Where to place the shared known-host function (forge module is the natural home)
- Whether to add a new error variant or reuse `BrowseUrlConstructionFailed`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Browse implementation
- `src/browse/mod.rs` — Current browse implementation; `build_file_url` needs line fragment support; `resolve_forge_type` has duplicated known-host table
- `src/forge/mod.rs` — `match_known_host` function (line 193+) is the canonical known-host table to keep

### Requirements
- `.planning/REQUIREMENTS.md` — BROWSE-01 (line-range deep-linking), BROWSE-02 (known-host deduplication)
- `.planning/ROADMAP.md` — Phase 6 success criteria with exact expected URL formats

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `forge::config_lookup` — Config-file forge resolution, already shared
- `forge::match_known_host` — The canonical known-host table to consolidate around
- `forge::parse_remote_parts` — Already used by browse for host/owner/repo extraction
- `browse::build_file_url` — Needs line fragment parameter added

### Established Patterns
- URL construction in `build_file_url` uses `match forge` for per-forge format differences — line fragments follow this pattern
- Error handling uses `GfError` enum variants with descriptive messages

### Integration Points
- `browse::resolve_forge_type` → replace with call to `forge::match_known_host` (after config lookup)
- `build_file_url` signature changes to accept optional line range
- File arg parsing needs to split `path:line` before passing to `normalize_path`

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-browse-enhancements*
*Context gathered: 2026-03-17*
