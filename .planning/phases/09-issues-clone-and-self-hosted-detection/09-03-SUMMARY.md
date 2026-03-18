---
phase: 09-issues-clone-and-self-hosted-detection
plan: 03
subsystem: forge-detection
tags: [self-hosted, probe, cache, auth-detection]
dependency_graph:
  requires: [09-01, 09-02]
  provides: [CORE-04]
  affects: [forge-detection-chain]
tech_stack:
  added: [probe-cache-toml, cli-timeout-probing]
  patterns: [xdg-cache-home, sequential-probe]
key_files:
  created: []
  modified:
    - src/forge/mod.rs
decisions:
  - decision: "Probe order: gh → glab → tea → fj (market share priority)"
    rationale: "Most users have GitHub/GitLab CLIs installed; probe most likely first"
    alternatives: ["Alphabetical order", "Random order"]
  - decision: "5-second timeout per CLI probe"
    rationale: "Balance between responsiveness and allowing slow systems to respond"
    alternatives: ["3 seconds", "10 seconds"]
  - decision: "Cache indefinitely without TTL"
    rationale: "Self-hosted domains rarely change forge type; manual cache clear if needed"
    alternatives: ["7-day TTL", "30-day TTL"]
metrics:
  duration: "2m 41s"
  tasks_completed: 3
  files_modified: 1
  tests_added: 2
  commits: 3
  completed_date: "2026-03-18"
---

# Phase 09 Plan 03: Self-Hosted Auto-Detection Summary

**One-liner:** CLI-based forge detection with probe caching for self-hosted domains using sequential auth status checks

## What Was Built

Implemented automatic forge type detection for unknown self-hosted domains via CLI auth probing:

1. **probe_auth() function**: Sequentially tries forge CLIs (gh, glab, tea, fj) to detect authenticated hosts
2. **Timeout system**: 5-second timeout per CLI using thread-based mpsc channels
3. **Probe cache**: ~/.cache/gf/probes.toml stores hostname → ForgeType mappings
4. **Cache management**: XDG_CACHE_HOME support, load/save/lookup functions
5. **detect() integration**: Full priority chain: config → known host → cached probe → live probe → error

## Detection Priority Chain

The `detect()` function now implements a complete fallback chain:

```rust
1. config_lookup()     // User explicit mapping in ~/.config/gf/config.toml (ALWAYS wins)
2. match_known_host()  // Built-in public hosts (github.com, gitlab.com, etc.)
3. cache_lookup()      // Previously probed self-hosted domains
4. probe_auth()        // Live CLI probing (gh → glab → tea → fj)
5. ForgeNotDetected    // Error with config.toml hint
```

## Probe Mechanics

**CLI commands tested:**
- GitHub: `gh auth status` → checks for hostname in stdout/stderr
- GitLab: `glab auth status` → checks for hostname in stdout/stderr
- Gitea: `tea logins ls` → checks for hostname in stdout/stderr
- Forgejo: `fj auth list` → checks for hostname in stdout/stderr

**Timeout implementation:**
- Each CLI gets 5 seconds to respond
- Thread-based execution with mpsc channel for result communication
- Worst case: 20 seconds (all 4 CLIs timeout)
- Typical case: <1 second (first or second CLI matches)

**Cache behavior:**
- Location: `~/.cache/gf/probes.toml` (respects `XDG_CACHE_HOME`)
- Format: TOML with `[hosts]` table mapping hostname → forge type
- No TTL expiry (cache indefinitely)
- Config.toml ALWAYS takes precedence over cache

## Task Completion

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 7c8e4c6 | probe_auth() with 5s timeout and CLI spawning |
| 2 | eb31fa6 | Probe cache at ~/.cache/gf/probes.toml |
| 3 | c8199d6 | detect() integration with full priority chain |

## Files Modified

**src/forge/mod.rs:**
- Added `probe_auth()` — sequential CLI probing function
- Added `run_with_timeout()` — thread-based timeout wrapper
- Added `ProbeCache` struct — hostname → ForgeType HashMap
- Added `cache_path()` — XDG_CACHE_HOME aware path resolver
- Added `load_probe_cache()`, `save_probe_cache()`, `cache_lookup()` — cache management
- Updated `detect()` — integrated probe chain with caching
- Added `Serialize` derive to `ForgeType`
- Added unit tests: `test_cache_path_with_home()`, `test_probe_cache_roundtrip()`

## Test Coverage

**New tests:**
1. `test_cache_path_with_home()` — verifies cache path construction when HOME is set
2. `test_probe_cache_roundtrip()` — tests save/load cycle with temp XDG_CACHE_HOME

**Verification results:**
- `cargo test` — all 25 integration tests pass ✅
- `cargo build --release` — clean build ✅
- Cache functions work with temporary directories ✅

## Deviations from Plan

None — plan executed exactly as written.

## Requirements Satisfied

**CORE-04:** Self-hosted forge auto-detection
- ✅ Unknown domains trigger probe sequence
- ✅ Probe order: gh → glab → tea → fj (market share priority)
- ✅ Successful probes cached in ~/.cache/gf/probes.toml
- ✅ Config.toml always takes precedence over cache
- ✅ 5-second timeout per CLI

**ISSUE-06:** Browse with --issue flag
- ✅ Verified still works (no changes to browse module)
- ✅ All existing tests pass

## Usage Example

**First run with self-hosted GitLab:**
```bash
# User authenticated to gitlab.company.com via glab
cd my-repo
gf pr list  # No config.toml entry for gitlab.company.com

# Detection flow:
# 1. config_lookup("gitlab.company.com") → None
# 2. match_known_host("gitlab.company.com") → Err (not public host)
# 3. cache_lookup("gitlab.company.com") → None (first run)
# 4. probe_auth("gitlab.company.com"):
#    - gh auth status → no match (timeout or hostname not found)
#    - glab auth status → MATCH (hostname found in output)
# 5. save_probe_cache("gitlab.company.com", Gitlab)
# → Routes to: glab mr list
```

**Subsequent runs:**
```bash
gf pr list

# Detection flow:
# 1. config_lookup() → None
# 2. match_known_host() → Err
# 3. cache_lookup("gitlab.company.com") → Some(Gitlab) ✅ CACHE HIT
# → Routes to: glab mr list (no probing needed)
```

**Manual config.toml override:**
```toml
# User realizes their self-hosted "gitlab.company.com" is actually Forgejo
[[forge]]
domain = "gitlab.company.com"
type = "forgejo"
```

```bash
gf pr list

# Detection flow:
# 1. config_lookup("gitlab.company.com") → Some(Forgejo) ✅ CONFIG WINS
# → Routes to: fj pr list (cache ignored)
```

## Cache File Format

**~/.cache/gf/probes.toml:**
```toml
[hosts]
"gitlab.company.com" = "gitlab"
"git.internal.io" = "forgejo"
"code.myteam.dev" = "gitea"
```

## Edge Cases Handled

1. **CLI not installed**: Timeout returns None, tries next CLI
2. **CLI installed but not authenticated**: No hostname match, tries next CLI
3. **Multiple self-hosted forges**: Each cached independently by hostname
4. **Config.toml vs cache conflict**: Config always wins (priority 1 vs priority 3)
5. **No matching CLI**: Returns ForgeNotDetected error with config.toml hint
6. **XDG_CACHE_HOME set**: Respects Linux/BSD convention
7. **No HOME variable**: cache_path() returns None (cache disabled gracefully)

## Performance Impact

- **Cache hit**: No performance impact (simple HashMap lookup)
- **First probe**: 0.1s - 20s depending on which CLI matches (typically 0.1-2s)
- **No match**: 20 seconds (all 4 CLIs timeout)

## Next Steps

Phase 09 Plan 04: Final integration tests and edge case validation.

## Self-Check

✅ **Files created:**
- (None — all modifications)

✅ **Files modified:**
- src/forge/mod.rs exists and contains probe_auth(), cache functions, updated detect()

✅ **Commits exist:**
- 7c8e4c6 ✅ (feat: probe_auth implementation)
- eb31fa6 ✅ (feat: probe cache)
- c8199d6 ✅ (feat: detect integration)

✅ **Tests pass:**
- cargo test — 25 passed ✅
- cargo build --release — success ✅

## Self-Check: PASSED
