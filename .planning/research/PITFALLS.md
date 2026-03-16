# Pitfalls Research

**Domain:** Rust CLI forge wrapper (subprocess delegation, remote URL detection, TTY passthrough)
**Researched:** 2026-03-16
**Confidence:** HIGH (subprocess/signal/TTY mechanics verified via official Rust docs + community; URL edge cases verified via Git docs + real bug reports)

---

## Critical Pitfalls

### Pitfall 1: Exit Code Lost When Child Terminates by Signal

**What goes wrong:**
`ExitStatus::code()` returns `None` on Unix when the child process is killed by a signal (e.g., SIGINT, SIGTERM). A naive wrapper that calls `.code().unwrap_or(1)` maps every signal-terminated child to exit code 1, hiding the real cause. Tools like `gh pr create` can be interrupted mid-flow; the parent wrapper must propagate the signal-death semantics faithfully.

**Why it happens:**
Developers test happy paths (exit code 0) and error paths (exit code non-zero) but forget the third case: signal termination. The Rust `ExitStatus::code()` API silently returns `None` for this case rather than panicking.

**How to avoid:**
On Unix, use `std::os::unix::process::ExitStatusExt::signal()` to detect signal termination. Re-raise the same signal in the wrapper process so the parent shell sees signal-death, not a synthetic exit code. Pattern:

```rust
use std::os::unix::process::ExitStatusExt;

let status = child.wait()?;
if let Some(code) = status.code() {
    std::process::exit(code);
} else if let Some(sig) = status.signal() {
    // Re-raise so the shell sees signal termination
    unsafe { libc::raise(sig); }
}
```

**Warning signs:**
- Bash `$?` is always 1 after Ctrl+C instead of 130 (128 + SIGINT)
- Shell scripts that check `[[ $? -eq 130 ]]` for Ctrl+C never match

**Phase to address:** Core subprocess delegation (the phase that wires up `Command::spawn` + `wait`)

---

### Pitfall 2: Wrapper Consumes Ctrl+C Before Child Sees It

**What goes wrong:**
When the Rust wrapper process receives SIGINT (Ctrl+C), if it has installed a signal handler (even via `ctrlc` crate), the default forwarding to the child process group does NOT happen automatically. The child (`gh`, `glab`, etc.) never receives SIGINT and hangs or continues running while the wrapper exits.

**Why it happens:**
On Unix, Ctrl+C sends SIGINT to the entire foreground process group. If the child was spawned with `Command::spawn()`, it inherits the process group and would normally receive the signal too. However, if the wrapper installs ANY signal handler (even a no-op), it intercepts the signal before the OS delivers it to the group. The `ctrlc` crate is a common offender.

**How to avoid:**
For the delegation path, the simplest correct approach is to NOT install a SIGINT handler — let the OS deliver to the entire process group naturally. Only install a handler if the wrapper needs to do cleanup work, and in that case explicitly forward the signal to the child's PID using `kill(child_pid, SIGINT)` before doing cleanup.

**Warning signs:**
- Running `gf pr create` and pressing Ctrl+C leaves a `gh` process running in the background
- `ps aux | grep gh` shows orphaned child processes after wrapper exits

**Phase to address:** Core subprocess delegation

---

### Pitfall 3: SCP-Style SSH Remote URLs Fail URL Parsing

**What goes wrong:**
Git SSH remotes commonly use SCP syntax: `git@github.com:owner/repo.git`. This is NOT a valid URL — it has no scheme and uses `:` as a path separator instead of `/`. Standard URL parsers (including Rust's `url` crate and `rust-url`) reject or misparse it. The wrapper fails to detect the forge for SSH-configured repos.

**Why it happens:**
Developers test with HTTPS remotes (`https://github.com/owner/repo`) but forget that many developers (especially those with SSH keys configured) use SCP-style remotes. The `url` crate explicitly does not support SCP syntax and the `rust-url` crate has a documented issue (#220) for this.

**How to avoid:**
Parse remote URLs with a two-pass approach:
1. Try standard URL parsing first (handles `https://`, `git://`, `ssh://`)
2. Fall back to regex for SCP format: `^([^@]+@)?([^:]+):(.+?)(?:\.git)?$`

Extract hostname from group 2, path from group 3. Never use a URL library alone for git remotes.

**Warning signs:**
- Forge detection returns "unknown" for SSH-configured repos
- Test suite only has HTTPS remote fixtures
- `git remote get-url origin` output starts with `git@`

**Phase to address:** Forge detection / remote URL parsing

---

### Pitfall 4: `url.insteadOf` Git Config Rewrites Break Forge Detection

**What goes wrong:**
Git's `url.<base>.insteadOf` config allows rewriting remote URLs transparently. A developer's `.gitconfig` might have `url."git@github.com:".insteadOf = "https://github.com/"`, meaning `git remote get-url origin` returns the pre-rewrite URL, but `git remote get-url --push origin` might return something different. Corporate environments often use this to proxy GitHub through an internal host (e.g., `github.internal.company.com`). The wrapper reads the raw remote URL and fails to detect GitHub because the hostname doesn't match `github.com`.

**Why it happens:**
Tools read `.git/config` or call `git remote get-url origin` and assume the result is a canonical forge URL. `insteadOf` rewrites are applied by git at fetch/push time but the stored remote URL is the pre-rewrite value.

**How to avoid:**
Use `git ls-remote --get-url origin` which applies `insteadOf` rewrites before returning. Alternatively, document that `--forge` flag can override auto-detection for non-standard setups, and surface a clear error with instructions when detection is ambiguous.

**Warning signs:**
- Users in enterprise environments report forge detection failing
- The detected hostname doesn't match any known forge pattern
- `git remote get-url origin` and `git ls-remote --get-url origin` return different values

**Phase to address:** Forge detection / remote URL parsing

---

### Pitfall 5: `.git` is a File, Not a Directory, in Worktrees

**What goes wrong:**
In Git worktrees (created with `git worktree add`), the `.git` entry in the worktree is a TEXT FILE pointing to the actual gitdir (e.g., `gitdir: /path/to/repo/.git/worktrees/branch`). If the wrapper tries to read `.git/config` directly to get remote URLs, it reads a file that says `gitdir: ...` and fails. Similarly, submodules have `.git` as a file.

**Why it happens:**
Developers test in normal single-worktree repos. The worktree case is only discovered by users of `git worktree`, which is a power-user feature.

**How to avoid:**
Never read `.git/config` directly. Always use `git remote get-url origin` (shell subprocess) or the `git2` crate which handles all `.git` forms correctly. The `git2` crate's `Repository::discover()` correctly walks up the directory tree and resolves worktree `.git` files.

**Warning signs:**
- `git worktree add` then trying `gf` fails with "not a git repo" or "no remotes found"
- Code does `fs::read_to_string(".git/config")`

**Phase to address:** Forge detection / remote URL parsing

---

### Pitfall 6: TTY Detection — Child CLI Disables Color/Interactivity

**What goes wrong:**
`gh`, `glab`, and other forge CLIs detect whether stdout is a TTY to decide whether to show colored output, progress spinners, and interactive prompts. When a wrapper spawns them with `Stdio::piped()` (to capture output), the child detects no TTY and produces plain text with no interactivity. Conversely, using `Stdio::inherit()` enables TTY detection but prevents the wrapper from inspecting output.

For `gf`, since the goal is transparent delegation (not output capture), the correct mode is `Stdio::inherit()` for all three streams — but this must be a conscious decision, not an accidental default.

**Why it happens:**
Developers write tests that capture output (requiring `Stdio::piped()`), then ship code that uses `Stdio::piped()` in production too. Or they use `Command::output()` which defaults all streams to `piped`.

**How to avoid:**
Use `Command::status()` (not `Command::output()`) for delegation. `status()` defaults stdin/stdout/stderr to `inherit`. Never use `output()` for delegation commands. For testing, use integration tests that check exit codes rather than captured output, or use a PTY library (`pty` crate) if testing interactive behavior is required.

**Warning signs:**
- `gh pr create` run through wrapper shows no color/spinner
- `glab auth login` (which is interactive) hangs waiting for user input that never arrives
- Code uses `.output()` for delegation

**Phase to address:** Core subprocess delegation

---

### Pitfall 7: clap Intercepts Flags Intended for the Child CLI

**What goes wrong:**
When using clap with passthrough subcommands, clap eagerly parses flags it recognizes before the `--` separator. If `gf` defines a `--verbose` flag and the user runs `gf pr create --verbose`, clap consumes `--verbose` for `gf` and never passes it to `gh`. The child CLI never sees `--verbose`. This is especially subtle with flags that both `gf` and the child CLIs share (like `--help`, `--version`, `--json`).

**Why it happens:**
Passthrough arg handling requires specific clap configuration (`trailing_var_arg = true` + `allow_hyphen_values = true`). Without these, clap parses everything and silently drops unknown flags or errors on them.

**How to avoid:**
For passthrough subcommands, define the trailing positional as:
```rust
#[arg(trailing_var_arg = true, allow_hyphen_values = true)]
args: Vec<String>,
```
Avoid defining top-level flags with the same names as common child CLI flags. For known normalization flags (the canonical flag translation feature), strip them from `args` before delegation and translate to child-specific equivalents. Do NOT define a top-level `--verbose` or `--json` if those should pass through.

**Warning signs:**
- `gf pr create --draft` passes `--draft` correctly but `gf pr create --help` shows gf's help instead of gh's help
- Users report that `gf pr list --json` doesn't work
- Flag interaction bugs appear when gf and child CLI share a flag name

**Phase to address:** Core subprocess delegation + flag normalization

---

### Pitfall 8: `--help` Ambiguity — User Expects Child CLI Help

**What goes wrong:**
Running `gf pr create --help` is ambiguous: the user might want `gf`'s help (which flags does `gf` understand for `pr create`?) or `gh pr create --help` (what does the real command support?). If `gf` intercepts `--help` and shows its own sparse help, users can't discover the full feature set of the underlying CLI.

**Why it happens:**
clap intercepts `--help` by default before args reach the passthrough collection. This is correct behavior for clap but wrong for transparent delegation.

**How to avoid:**
Two options:
1. Let `gf` intercept `--help` but have it show a brief header and then delegate to `gh pr create --help` (run child with `--help` appended).
2. Use clap's `disable_help_flag(true)` on passthrough subcommands and include `--help` in the passthrough args.

Option 1 is more user-friendly. Document the distinction clearly in `gf --help` (top level) vs subcommand help.

**Warning signs:**
- `gf pr create --help` shows a stub help page with no flag details
- Users file issues asking "why does gf hide gh's flags?"

**Phase to address:** Core subprocess delegation + CLI UX

---

### Pitfall 9: Self-Hosted Forge Detection Has No Ground Truth

**What goes wrong:**
GitHub is at `github.com`. GitLab.com is at `gitlab.com`. But self-hosted GitLab could be at `git.mycompany.com`, self-hosted Gitea/Forgejo at `forgejo.internal`. There is no reliable hostname-based detection for self-hosted instances. A wrapper that only matches known public hostnames silently fails for every self-hosted user.

**Why it happens:**
The public forge case is easy and gets tested. Self-hosted is treated as an edge case but is actually a primary use case for Gitea and Forgejo users (virtually all Gitea/Forgejo installs are self-hosted).

**How to avoid:**
Forge detection strategy (in priority order):
1. Check `gf.forge` git config key (explicit override, highest priority)
2. Check `--forge` CLI flag
3. Match against known public hostnames (`github.com`, `gitlab.com`)
4. Probe the underlying CLIs: check if `gh`/`glab`/`tea`/`fj` has an authenticated host matching the remote hostname
5. Fall back to error with clear message listing `gf.forge` as the fix

For step 4: `gh auth status` lists authenticated hosts; `glab auth status` does the same. Parse these to detect self-hosted instances.

**Warning signs:**
- Forge detection only has a hostname allowlist with no fallback
- Self-hosted Forgejo users get "unknown forge" errors
- No `gf.forge` config override mechanism exists

**Phase to address:** Forge detection

---

### Pitfall 10: stdin Not Forwarded — Piped Input to Wrapper Broken

**What goes wrong:**
Some forge CLI workflows read from stdin (e.g., `echo "body text" | gh pr create --body-file -`). If the wrapper uses `Stdio::piped()` for stdin or fails to forward stdin from the wrapper's own stdin to the child, piped input silently disappears or the child blocks waiting for input.

**Why it happens:**
Developers test interactive usage (terminal). Piped/non-interactive stdin is only discovered when users try to script `gf` in CI or shell pipelines.

**How to avoid:**
Use `Stdio::inherit()` for stdin in the delegation path. `Command::status()` does this by default — confirm this is being used, not `Command::output()`.

Separately: when gf itself is run non-interactively (stdin is not a TTY), this should propagate correctly through `inherit`.

**Warning signs:**
- `echo "title" | gf pr create` behaves differently than `echo "title" | gh pr create`
- CI scripts using piped input to `gf` fail

**Phase to address:** Core subprocess delegation

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Regex-only remote URL parsing | Fast to implement | Breaks on unusual but valid URL forms (e.g., `git://host/path`, URLs with ports) | Never — use two-pass approach (URL lib + SCP fallback) |
| Hardcode github.com / gitlab.com hostname match only | Covers 80% of cases | All self-hosted users broken; Forgejo/Gitea users broken by default | Never for v1 ship |
| Using `Command::output()` for delegation | Easy to test | Disables TTY, breaks interactive CLIs, strips color | Never for delegation; only for explicit capture (e.g., `git remote get-url`) |
| No signal forwarding / no re-raise | Simpler code | Exit codes wrong in scripts; orphaned child processes | Never |
| Skip `--` passthrough support | Simpler arg parsing | Users can't escape to child CLI's raw flags | Never — this is core to the product |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `gh` CLI | Spawning with captured stdout breaks interactive prompts (e.g., `gh auth login`) | Always use `Stdio::inherit()` for delegation; only capture for explicit read commands |
| `glab` CLI | Self-hosted GitLab URL has no standard hostname pattern | Probe `glab auth status` output to detect configured hosts |
| `tea` / `fj` | Assuming these CLIs have parity with `gh`/`glab` flags | Test each CLI's actual flag names; normalization map must be verified per-CLI |
| `git remote` | Calling `git remote get-url` from wrong working directory | Always set `Command::current_dir()` to the repo root when shelling out to git |
| browse / URL construction | Assuming path is always `owner/repo` | Some self-hosted Gitea instances use non-standard root paths; `ROOT_URL` in Forgejo config can have a subpath |

---

## Performance Traps

This project is a thin CLI wrapper. Performance traps are minor but worth noting:

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Running `git remote get-url` twice (once for detection, once for browse) | Adds ~10ms per extra subprocess | Cache the remote URL in the current invocation | Not a real threshold issue — just sloppy |
| Probing all 4 CLIs at startup to detect forge | Adds 40–100ms per invocation | Only probe the CLI for the detected forge; detect forge from URL first | Any usage — always avoid |
| `which gh` / `which glab` at every invocation | Adds subprocess overhead | Check PATH presence once at startup and cache, or just let the exec fail with a clear error | Any usage |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Shell-interpolating remote URL into a command string | Remote URL could contain shell metacharacters; code injection | Always pass args as `Vec<String>` to `Command::args()`, never via shell string interpolation |
| Trusting `gf.forge` git config without validation | A malicious `.git/config` in a repo could set `gf.forge` to an unexpected value | Validate that the config value is one of the known forge identifiers; log when using config override |
| Passing unvalidated file paths to browse URL construction | Path traversal in constructed URLs (low severity but surprising) | Canonicalize file paths relative to repo root before embedding in URLs |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| "Forge CLI not found" error with no install hint | User doesn't know what to install | Include install command in error: "Install with: brew install gh" |
| Generic "delegation failed" error with no context | User can't debug | Show the exact command that was run and its stderr output |
| Silent fallthrough when forge unknown | User runs command, nothing happens | Always exit non-zero with a clear message when forge detection fails |
| `--help` shows gf stub instead of child CLI help | Users can't discover child CLI flags | Delegate `--help` to child CLI after brief gf header |
| Forge detected incorrectly, no override visible | User stuck with wrong forge | Document `gf.forge` config and `--forge` flag prominently in error messages |

---

## "Looks Done But Isn't" Checklist

- [ ] **Exit code propagation:** Works for exit(0) and exit(1) — verify signal termination also propagates (test with Ctrl+C mid-run)
- [ ] **SSH remotes:** Forge detection tested with `git@github.com:owner/repo.git` format, not just HTTPS
- [ ] **Worktree support:** `gf` works when run inside a `git worktree` checkout
- [ ] **Self-hosted Gitea/Forgejo:** Forge detection works for non-public hostnames (test with `glab auth status` probe)
- [ ] **Piped stdin:** `echo "text" | gf pr create` forwards stdin correctly to child CLI
- [ ] **Interactive auth flows:** `gf auth login` (delegating to `gh auth login`) works with full terminal interactivity
- [ ] **`--` passthrough:** `gf pr create -- --some-exotic-gh-flag` passes the flag through to `gh`
- [ ] **browse with subpath:** Self-hosted Gitea with a `ROOT_URL` subpath generates correct browse URLs
- [ ] **Detached HEAD branch:** `gf browse` with detached HEAD uses commit hash, not a branch name
- [ ] **Missing CLI install hint:** Running `gf` in a Forgejo repo without `fj` installed shows clear error + install command

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Signal propagation broken | LOW | Add `ExitStatusExt::signal()` check + re-raise; 10 lines of code |
| SCP URL parsing broken | LOW | Add regex fallback after URL parse attempt; existing test suite catches it once SSH fixtures added |
| TTY passthrough broken (`output()` used) | LOW | Replace `Command::output()` calls with `Command::status()`; caught by integration test checking color output |
| Self-hosted detection missing | MEDIUM | Requires `glab/gh auth status` probe + `gf.forge` override; needs design + implementation |
| clap eating passthrough flags | MEDIUM | Requires refactoring subcommand definitions to use `trailing_var_arg`; may need clap API changes |
| browse subpath URLs wrong | LOW | URL construction fix; unit testable |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Signal death exit code lost | Core subprocess delegation | Test: `gf` returns exit 130 when child receives SIGINT |
| Ctrl+C not forwarded to child | Core subprocess delegation | Test: child process not orphaned after Ctrl+C |
| SCP URL parsing | Forge detection | Test fixtures: SSH remotes for all forges |
| `insteadOf` rewrite bypass | Forge detection | Test: mock `git ls-remote --get-url` output |
| Worktree `.git` file | Forge detection | Test: run `gf` inside a worktree directory |
| TTY passthrough broken | Core subprocess delegation | Integration test: gh interactive command works |
| clap flag interception | Core subprocess delegation + flag normalization | Test: `gf pr create --some-unknown-flag` passes flag through |
| `--help` ambiguity | CLI UX / subcommand wiring | Manual test: `gf pr create --help` shows gh's help |
| Self-hosted detection | Forge detection | Test: forge detected from `gh auth status` output for custom hostname |
| Stdin not forwarded | Core subprocess delegation | Test: `echo x | gf pr create` delivers stdin to child |

---

## Sources

- [ExitStatusExt::signal() — Rust std docs](https://doc.rust-lang.org/std/os/unix/process/trait.ExitStatusExt.html)
- [ExitStatus::code() — Rust std docs](https://doc.rust-lang.org/std/process/struct.ExitStatus.html)
- [Signal handling — Rust CLI book](https://rust-cli.github.io/book/in-depth/signals.html)
- [Dealing with process termination in Linux (Rust examples)](https://iximiuz.com/en/posts/dealing-with-processes-termination-in-Linux/)
- [TTY passthrough Rust forum discussion](https://users.rust-lang.org/t/how-to-fool-a-subprocess-into-thinking-its-stdout-stderr-was-a-tty-while-still-reading-output-of-its-stdout-stderr/79810)
- [Cannot parse SCP-style git URLs — rust-url issue #220](https://github.com/servo/rust-url/issues/220)
- [Cannot parse SCP-style git URLs — cargo issue #3014](https://github.com/rust-lang/cargo/issues/3014)
- [git-lfs insteadOf not honored in submodules](https://github.com/git-lfs/git-lfs/issues/5665)
- [clap TrailingVarArg doesn't work without -- issue #1538](https://github.com/clap-rs/clap/issues/1538)
- [Leleat/git-forge — comparable project](https://github.com/Leleat/git-forge)
- [git-scm: The Protocols — URL format reference](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols)

---
*Pitfalls research for: Rust CLI forge wrapper (gf)*
*Researched: 2026-03-16*
