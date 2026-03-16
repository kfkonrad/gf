# Roadmap: gf (git forge)

## Overview

`gf` is built in strict dependency order: correct subprocess delegation first (everything runs on this), then forge detection (the prerequisite for all dispatch), then command routing with flag normalization and all delegated command groups, then the native browse implementation. Each phase delivers a coherent, testable capability before the next begins. The project is complete when a user can run any `gf` command on any of the four supported forges and get the correct result.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation** - Subprocess delegation, error types, and CLI presence detection
- [ ] **Phase 2: Forge Detection** - Auto-detect forge from git remote with full edge case coverage
- [ ] **Phase 3: Command Routing** - ForgeAdapter trait, flag normalization, PR / repo / auth commands, aliases
- [ ] **Phase 4: Browse** - Native URL construction and browser open for all four forges

## Phase Details

### Phase 1: Foundation
**Goal**: The subprocess runner is correct and the error system is in place — all future phases build on this without retrofitting
**Depends on**: Nothing (first phase)
**Requirements**: CORE-06, CORE-07
**Success Criteria** (what must be TRUE):
  1. Running `gf <any command>` when the required forge CLI is not on PATH prints a human-readable error with an install hint (e.g., "glab not found — install with: brew install glab")
  2. When the underlying CLI exits normally, `gf` exits with the same exit code
  3. When the underlying CLI is killed by a signal (e.g., Ctrl+C), `gf` re-raises the signal so the shell sees exit 130, not exit 1
  4. The underlying CLI receives an inherited TTY — color output and interactive prompts work identically to calling the CLI directly
**Plans**: TBD

### Phase 2: Forge Detection
**Goal**: Given any git repo, `gf` reliably identifies which forge it lives on — including self-hosted instances and non-standard remotes
**Depends on**: Phase 1
**Requirements**: CORE-01, CORE-02, CORE-03, CORE-04, CORE-05
**Success Criteria** (what must be TRUE):
  1. `gf` correctly detects GitHub, GitLab, Gitea, and Codeberg from HTTPS remote URLs (e.g., `https://github.com/owner/repo.git`)
  2. `gf` correctly detects the forge from SSH SCP-style remote URLs (e.g., `git@github.com:owner/repo.git`)
  3. `gf <command> --remote upstream` uses the `upstream` remote instead of `origin` for forge detection
  4. A domain-to-forge mapping in `~/.config/gf/config.toml` allows a user on a self-hosted Forgejo instance to have `gf` correctly route commands after adding that domain
  5. When detection fails and no config override exists, `gf` prints a clear error explaining that the forge could not be detected and how to configure it
**Plans**: TBD

### Phase 3: Command Routing
**Goal**: Users can run `gf pr`, `gf repo`, and `gf auth` commands on any supported forge with canonical flags, and all aliases resolve correctly
**Depends on**: Phase 2
**Requirements**: CORE-08, CORE-09, CORE-10, CORE-11, CORE-12, PR-01, PR-02, PR-03, PR-04, PR-05, PR-06, REPO-01, REPO-02, REPO-03, AUTH-01, AUTH-02, AUTH-03
**Success Criteria** (what must be TRUE):
  1. `gf pr create --title "fix" --body "details" --base main --draft` works on GitHub (delegates to `gh`), GitLab (translates `--body` to `--description`, `--base` to `--target-branch`, delegates to `glab mr create`), and Gitea/Forgejo equivalents
  2. `gf pr view` (no number) on a branch with an open PR shows that PR; unrecognized flags passed through to the underlying CLI unchanged
  3. `gf repo view`, `gf repo create`, and `gf repo fork` each delegate to the correct underlying CLI for the detected forge
  4. `gf auth login`, `gf auth logout`, and `gf auth status` each delegate to the correct underlying CLI for the detected forge
  5. `gf p c`, `gf mr create`, `gf r v`, `gf a s` and all other defined aliases and abbreviations resolve to the same underlying command as their full forms, and appear in `--help` and shell completion output
**Plans**: TBD

### Phase 4: Browse
**Goal**: Users can open any repo, branch, or file in their browser from the terminal using `gf browse`, natively and correctly across all four forges
**Depends on**: Phase 3
**Requirements**: BROWSE-01, BROWSE-02, BROWSE-03, BROWSE-04, BROWSE-05
**Success Criteria** (what must be TRUE):
  1. `gf browse` opens the current repo's web page on the detected forge in the default browser
  2. `gf browse` uses the current branch in the URL; when HEAD is detached, it falls back to the commit SHA
  3. `gf browse path/to/file.rs` opens the file's view URL on the correct forge
  4. `gf browse --branch main` opens the repo at the `main` branch regardless of which branch is checked out
  5. Browse URLs are constructed locally — no delegation to `gh browse`, `glab browse`, or `tea browse`
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 0/TBD | Not started | - |
| 2. Forge Detection | 0/TBD | Not started | - |
| 3. Command Routing | 0/TBD | Not started | - |
| 4. Browse | 0/TBD | Not started | - |
