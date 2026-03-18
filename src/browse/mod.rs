//! Native browse command — constructs forge URLs and opens them in the browser.
//! Does NOT delegate to gh/glab/tea/fj (BROWSE-05).

use crate::error::GfError;
use crate::forge::{detect_from_host, parse_remote_parts, ForgeType};
use clap::ArgMatches;

// ── Line-range types ─────────────────────────────────────────────────────────

/// Parsed line range from colon suffix (e.g., `:42` or `:42-55`).
#[derive(Debug)]
pub struct LineRange {
    start: u32,
    end: Option<u32>, // None = single line
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Entry point called from main.rs for the `browse` subcommand.
pub fn run(matches: &ArgMatches) -> Result<(), GfError> {
    // 1. Get remote name from global flag (default "origin")
    //    Note: browse handles its own remote detection — does NOT call forge::detect()
    //    because detect() discards owner/repo. We need all three parts.
    let remote = matches
        .get_one::<String>("remote")
        .map(|s| s.as_str())
        .unwrap_or("origin");

    // 2. Parse remote URL into parts
    let raw_url = get_remote_url(remote)?;
    let (host, owner, repo) = parse_remote_parts(&raw_url)?;

    // 3. Determine ForgeType from host (config lookup then known hosts)
    let forge_type = resolve_forge_type(&host)?;

    // Handle --pr/--mr and --issue early — these don't need git ref resolution
    if let Some(pr_num) = matches.get_one::<String>("pr") {
        let url = build_pr_url(&forge_type, &host, &owner, &repo, pr_num);
        println!("{url}");
        let no_browser = matches.get_flag("no-browser");
        if !no_browser {
            webbrowser::open(&url).map_err(|e| GfError::BrowseFailed(url.clone(), e))?;
        }
        return Ok(());
    }

    if let Some(issue_num) = matches.get_one::<String>("issue") {
        let url = build_issue_url(&forge_type, &host, &owner, &repo, issue_num);
        println!("{url}");
        let no_browser = matches.get_flag("no-browser");
        if !no_browser {
            webbrowser::open(&url).map_err(|e| GfError::BrowseFailed(url.clone(), e))?;
        }
        return Ok(());
    }

    // 4. Resolve branch/SHA ref
    let branch_override = matches.get_one::<String>("branch").map(|s| s.as_str());
    let (git_ref, is_sha) = resolve_ref(branch_override)?;

    // 5. Build URL
    let file_arg = matches.get_one::<String>("file").map(|s| s.as_str());
    let url = if let Some(raw_file) = file_arg {
        let (path_part, line_spec) = split_file_and_line(raw_file);
        let line_range = line_spec.map(parse_line_spec).transpose()?;
        let normalized = normalize_path(path_part)?;
        build_file_url(
            &forge_type,
            &host,
            &owner,
            &repo,
            &git_ref,
            is_sha,
            &normalized,
            line_range.as_ref(),
        )
    } else {
        build_repo_url(&forge_type, &host, &owner, &repo, &git_ref)
    };

    // 6. Print URL always (like gh browse behavior)
    println!("{url}");

    // 7. Open browser unless --no-browser flag is set
    let no_browser = matches.get_flag("no-browser");
    if !no_browser {
        webbrowser::open(&url).map_err(|e| GfError::BrowseFailed(url.clone(), e))?;
    }

    Ok(())
}

// ── URL construction ────────────────────────────────────────────────────────

/// Builds a repo URL for the given forge and ref.
/// Repo URL format:
///   GitHub:  https://host/owner/repo/tree/<ref>
///   GitLab:  https://host/owner/repo/-/tree/<ref>
///   Gitea:   https://host/owner/repo/src/branch/<ref>  (or src/commit/ for SHA)
///   Forgejo: https://host/owner/repo/src/branch/<ref>  (or src/commit/ for SHA)
pub fn build_repo_url(
    forge: &ForgeType,
    host: &str,
    owner: &str,
    repo: &str,
    git_ref: &str,
) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github => format!("{base}/tree/{git_ref}"),
        ForgeType::Gitlab => format!("{base}/-/tree/{git_ref}"),
        ForgeType::Gitea | ForgeType::Forgejo => format!("{base}/src/branch/{git_ref}"),
    }
}

/// Builds a file view URL for the given forge, ref, and file path.
/// is_sha: true when git_ref is a full commit SHA (affects Gitea/Forgejo URL segment).
/// line_range: optional line anchor (appended as per-forge fragment).
/// File URL format:
///   GitHub:  https://host/owner/repo/blob/<ref>/<path>[#L<n>[-L<m>]]
///   GitLab:  https://host/owner/repo/-/blob/<ref>/<path>[#L<n>[-<m>]]
///   Gitea:   https://host/owner/repo/src/branch/<ref>/<path>[#L<n>[-L<m>]]  (or src/commit/ if SHA)
///   Forgejo: https://host/owner/repo/src/branch/<ref>/<path>[#L<n>[-L<m>]]  (or src/commit/ if SHA)
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
    let fragment = line_range
        .map(|lr| line_fragment(forge, lr))
        .unwrap_or_default();
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

/// Builds a PR/MR URL for the given forge and PR number.
/// PR URL format:
///   GitHub:  https://host/owner/repo/pull/<number>
///   GitLab:  https://host/owner/repo/-/merge_requests/<number>
///   Gitea:   https://host/owner/repo/pulls/<number>
///   Forgejo: https://host/owner/repo/pulls/<number>
pub fn build_pr_url(
    forge: &ForgeType,
    host: &str,
    owner: &str,
    repo: &str,
    number: &str,
) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github => format!("{base}/pull/{number}"),
        ForgeType::Gitlab => format!("{base}/-/merge_requests/{number}"),
        ForgeType::Gitea | ForgeType::Forgejo => format!("{base}/pulls/{number}"),
    }
}

/// Builds an issue URL for the given forge and issue number.
/// Issue URL format:
///   GitHub:  https://host/owner/repo/issues/<number>
///   GitLab:  https://host/owner/repo/-/issues/<number>
///   Gitea:   https://host/owner/repo/issues/<number>
///   Forgejo: https://host/owner/repo/issues/<number>
pub fn build_issue_url(
    forge: &ForgeType,
    host: &str,
    owner: &str,
    repo: &str,
    number: &str,
) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github | ForgeType::Gitea | ForgeType::Forgejo => {
            format!("{base}/issues/{number}")
        }
        ForgeType::Gitlab => format!("{base}/-/issues/{number}"),
    }
}

// ── Line-range helpers ───────────────────────────────────────────────────────

/// Splits "path:linespec" on the last colon. Returns (path, optional_line_spec).
fn split_file_and_line(raw: &str) -> (&str, Option<&str>) {
    if let Some(pos) = raw.rfind(':') {
        let (path, rest) = (&raw[..pos], &raw[pos + 1..]);
        if !rest.is_empty() {
            return (path, Some(rest));
        }
        // Trailing colon with nothing after: strip the colon, no line spec
        return (path, None);
    }
    (raw, None)
}

/// Parses a line spec string (e.g., "42" or "42-55") into a LineRange.
fn parse_line_spec(spec: &str) -> Result<LineRange, GfError> {
    if let Some((start_str, end_str)) = spec.split_once('-') {
        let start: u32 = start_str.parse().map_err(|_| invalid_line_err(spec))?;
        let end: u32 = end_str.parse().map_err(|_| invalid_line_err(spec))?;
        if start == 0 || end == 0 {
            return Err(invalid_line_err(spec));
        }
        if end < start {
            return Err(GfError::BrowseUrlConstructionFailed(format!(
                "line range '{spec}' is reversed — end must be >= start"
            )));
        }
        Ok(LineRange {
            start,
            end: Some(end),
        })
    } else {
        let n: u32 = spec.parse().map_err(|_| invalid_line_err(spec))?;
        if n == 0 {
            return Err(invalid_line_err(spec));
        }
        Ok(LineRange {
            start: n,
            end: None,
        })
    }
}

fn invalid_line_err(spec: &str) -> GfError {
    GfError::BrowseUrlConstructionFailed(format!(
        "invalid line spec '{spec}' — expected N or N-M (e.g. 42, 42-55)"
    ))
}

/// Returns the per-forge URL fragment for a line range.
fn line_fragment(forge: &ForgeType, lr: &LineRange) -> String {
    match forge {
        ForgeType::Github | ForgeType::Gitea | ForgeType::Forgejo => match lr.end {
            None => format!("#L{}", lr.start),
            Some(end) => format!("#L{}-L{}", lr.start, end),
        },
        ForgeType::Gitlab => match lr.end {
            None => format!("#L{}", lr.start),
            Some(end) => format!("#L{}-{}", lr.start, end),
        },
    }
}

// ── Git queries ─────────────────────────────────────────────────────────────

/// Runs `git remote get-url <remote>` and returns the URL string.
fn get_remote_url(remote: &str) -> Result<String, GfError> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .map_err(GfError::GitCommandFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GfError::NotAGitRepo);
        }
        return Err(GfError::NoRemote(remote.to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Resolves the ForgeType for a given host using the full detection chain
/// (config → known hosts → cache → live probe).
fn resolve_forge_type(host: &str) -> Result<ForgeType, GfError> {
    detect_from_host(host)
}

/// Resolves the git ref to use in the URL.
/// Returns (ref_string, is_sha).
/// is_sha is true when detached HEAD fallback was used (full 40-char SHA).
pub fn resolve_ref(branch_override: Option<&str>) -> Result<(String, bool), GfError> {
    if let Some(b) = branch_override {
        return Ok((b.to_string(), false));
    }
    match get_current_branch() {
        Ok(branch) => Ok((branch, false)),
        Err(_) => {
            let sha = get_head_sha()?;
            Ok((sha, true))
        }
    }
}

/// Runs `git symbolic-ref --short HEAD` to get the current branch name.
/// Returns Err if HEAD is detached.
fn get_current_branch() -> Result<String, GfError> {
    let output = std::process::Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()
        .map_err(GfError::GitCommandFailed)?;
    if !output.status.success() {
        return Err(GfError::BrowseUrlConstructionFailed(
            "detached HEAD — use --branch or check out a branch".to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Runs `git rev-parse HEAD` to get the full 40-char SHA.
/// Used as fallback when HEAD is detached.
fn get_head_sha() -> Result<String, GfError> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(GfError::GitCommandFailed)?;
    if !output.status.success() {
        return Err(GfError::BrowseUrlConstructionFailed(
            "could not determine HEAD commit SHA".to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Runs `git rev-parse --show-toplevel` to get the repo root path.
fn get_repo_toplevel() -> Result<String, GfError> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(GfError::GitCommandFailed)?;
    if !output.status.success() {
        return Err(GfError::NotAGitRepo);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Normalizes a file path to be repo-relative.
/// Absolute paths: strip the repo root prefix.
/// Relative paths: pass through unchanged.
/// No local filesystem validation — path may only exist on remote (BROWSE-03 decision).
pub fn normalize_path(path: &str) -> Result<String, GfError> {
    if path.starts_with('/') {
        let toplevel = get_repo_toplevel()?;
        let stripped = path.strip_prefix(&toplevel).ok_or_else(|| {
            GfError::BrowseUrlConstructionFailed(format!(
                "path '{path}' is outside the repository root '{toplevel}'"
            ))
        })?;
        Ok(stripped.trim_start_matches('/').to_string())
    } else {
        Ok(path.to_string())
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::ForgeType;

    // ── build_repo_url ──

    #[test]
    fn test_build_repo_url_github() {
        let url = build_repo_url(&ForgeType::Github, "github.com", "alice", "myrepo", "main");
        assert_eq!(url, "https://github.com/alice/myrepo/tree/main");
    }

    #[test]
    fn test_build_repo_url_gitlab() {
        let url = build_repo_url(&ForgeType::Gitlab, "gitlab.com", "alice", "myrepo", "main");
        assert_eq!(url, "https://gitlab.com/alice/myrepo/-/tree/main");
    }

    #[test]
    fn test_build_repo_url_gitea() {
        let url = build_repo_url(&ForgeType::Gitea, "gitea.com", "alice", "myrepo", "main");
        assert_eq!(url, "https://gitea.com/alice/myrepo/src/branch/main");
    }

    #[test]
    fn test_build_repo_url_forgejo() {
        let url = build_repo_url(
            &ForgeType::Forgejo,
            "codeberg.org",
            "alice",
            "myrepo",
            "main",
        );
        assert_eq!(url, "https://codeberg.org/alice/myrepo/src/branch/main");
    }

    #[test]
    fn test_build_repo_url_self_hosted_gitlab() {
        // Self-hosted: host is derived from remote, same URL pattern
        let url = build_repo_url(
            &ForgeType::Gitlab,
            "git.mycompany.com",
            "team",
            "proj",
            "develop",
        );
        assert_eq!(url, "https://git.mycompany.com/team/proj/-/tree/develop");
    }

    // ── build_file_url (existing tests updated with None 8th arg) ──

    #[test]
    fn test_build_file_url_github() {
        let url = build_file_url(
            &ForgeType::Github,
            "github.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            None,
        );
        assert_eq!(url, "https://github.com/alice/myrepo/blob/main/src/lib.rs");
    }

    #[test]
    fn test_build_file_url_gitlab_has_infix() {
        let url = build_file_url(
            &ForgeType::Gitlab,
            "gitlab.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            None,
        );
        assert_eq!(
            url,
            "https://gitlab.com/alice/myrepo/-/blob/main/src/lib.rs"
        );
    }

    #[test]
    fn test_build_file_url_gitea_branch() {
        let url = build_file_url(
            &ForgeType::Gitea,
            "gitea.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            None,
        );
        assert_eq!(
            url,
            "https://gitea.com/alice/myrepo/src/branch/main/src/lib.rs"
        );
    }

    #[test]
    fn test_build_file_url_gitea_sha_uses_commit_segment() {
        // Pitfall 1: Gitea/Forgejo use src/commit/<sha> not src/branch/<sha> for detached HEAD
        let sha = "a".repeat(40);
        let url = build_file_url(
            &ForgeType::Gitea,
            "gitea.com",
            "alice",
            "myrepo",
            &sha,
            true,
            "src/lib.rs",
            None,
        );
        assert_eq!(
            url,
            format!("https://gitea.com/alice/myrepo/src/commit/{sha}/src/lib.rs")
        );
    }

    #[test]
    fn test_build_file_url_forgejo_sha_uses_commit_segment() {
        let sha = "b".repeat(40);
        let url = build_file_url(
            &ForgeType::Forgejo,
            "codeberg.org",
            "alice",
            "myrepo",
            &sha,
            true,
            "src/lib.rs",
            None,
        );
        assert_eq!(
            url,
            format!("https://codeberg.org/alice/myrepo/src/commit/{sha}/src/lib.rs")
        );
    }

    // ── split_file_and_line ──

    #[test]
    fn test_split_file_and_line_with_single_line() {
        let (path, spec) = split_file_and_line("src/main.rs:42");
        assert_eq!(path, "src/main.rs");
        assert_eq!(spec, Some("42"));
    }

    #[test]
    fn test_split_file_and_line_with_range() {
        let (path, spec) = split_file_and_line("src/main.rs:42-55");
        assert_eq!(path, "src/main.rs");
        assert_eq!(spec, Some("42-55"));
    }

    #[test]
    fn test_split_file_and_line_no_colon() {
        let (path, spec) = split_file_and_line("src/main.rs");
        assert_eq!(path, "src/main.rs");
        assert_eq!(spec, None);
    }

    #[test]
    fn test_split_file_and_line_trailing_colon() {
        // Trailing colon with nothing after it => no spec
        let (path, spec) = split_file_and_line("src/main.rs:");
        assert_eq!(path, "src/main.rs");
        assert_eq!(spec, None);
    }

    // ── parse_line_spec ──

    #[test]
    fn test_parse_line_spec_single() {
        let lr = parse_line_spec("42").unwrap();
        assert_eq!(lr.start, 42);
        assert_eq!(lr.end, None);
    }

    #[test]
    fn test_parse_line_spec_range() {
        let lr = parse_line_spec("42-55").unwrap();
        assert_eq!(lr.start, 42);
        assert_eq!(lr.end, Some(55));
    }

    #[test]
    fn test_parse_line_spec_zero_errors() {
        let err = parse_line_spec("0").unwrap_err();
        assert!(
            matches!(err, GfError::BrowseUrlConstructionFailed(_)),
            "expected BrowseUrlConstructionFailed, got: {err:?}"
        );
    }

    #[test]
    fn test_parse_line_spec_zero_start_in_range_errors() {
        let err = parse_line_spec("0-10").unwrap_err();
        assert!(matches!(err, GfError::BrowseUrlConstructionFailed(_)));
    }

    #[test]
    fn test_parse_line_spec_zero_end_in_range_errors() {
        let err = parse_line_spec("10-0").unwrap_err();
        assert!(matches!(err, GfError::BrowseUrlConstructionFailed(_)));
    }

    #[test]
    fn test_parse_line_spec_reversed_errors() {
        let err = parse_line_spec("55-42").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("reversed"),
            "expected 'reversed' in message, got: {msg}"
        );
    }

    #[test]
    fn test_parse_line_spec_non_numeric_errors() {
        let err = parse_line_spec("abc").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("invalid"),
            "expected 'invalid' in message, got: {msg}"
        );
    }

    // ── line_fragment ──

    #[test]
    fn test_line_fragment_github_single() {
        let lr = LineRange {
            start: 42,
            end: None,
        };
        assert_eq!(line_fragment(&ForgeType::Github, &lr), "#L42");
    }

    #[test]
    fn test_line_fragment_github_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        assert_eq!(line_fragment(&ForgeType::Github, &lr), "#L42-L55");
    }

    #[test]
    fn test_line_fragment_gitlab_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        assert_eq!(line_fragment(&ForgeType::Gitlab, &lr), "#L42-55");
    }

    #[test]
    fn test_line_fragment_gitea_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        assert_eq!(line_fragment(&ForgeType::Gitea, &lr), "#L42-L55");
    }

    #[test]
    fn test_line_fragment_forgejo_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        assert_eq!(line_fragment(&ForgeType::Forgejo, &lr), "#L42-L55");
    }

    // ── build_file_url with line ranges ──

    #[test]
    fn test_build_file_url_with_line_github_single() {
        let lr = LineRange {
            start: 42,
            end: None,
        };
        let url = build_file_url(
            &ForgeType::Github,
            "github.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            Some(&lr),
        );
        assert_eq!(
            url,
            "https://github.com/alice/myrepo/blob/main/src/lib.rs#L42"
        );
    }

    #[test]
    fn test_build_file_url_with_line_github_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        let url = build_file_url(
            &ForgeType::Github,
            "github.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            Some(&lr),
        );
        assert!(url.ends_with("src/lib.rs#L42-L55"), "url={url}");
    }

    #[test]
    fn test_build_file_url_with_line_gitlab_range() {
        let lr = LineRange {
            start: 42,
            end: Some(55),
        };
        let url = build_file_url(
            &ForgeType::Gitlab,
            "gitlab.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            Some(&lr),
        );
        assert!(url.ends_with("src/lib.rs#L42-55"), "url={url}");
    }

    #[test]
    fn test_build_file_url_no_line_range_unchanged() {
        // None produces same URL as original (no fragment)
        let url = build_file_url(
            &ForgeType::Github,
            "github.com",
            "alice",
            "myrepo",
            "main",
            false,
            "src/lib.rs",
            None,
        );
        assert_eq!(url, "https://github.com/alice/myrepo/blob/main/src/lib.rs");
    }

    // ── normalize_path ──

    #[test]
    fn test_normalize_path_relative_passthrough() {
        // Relative paths are returned unchanged (no git call needed)
        let result = normalize_path("src/lib.rs").unwrap();
        assert_eq!(result, "src/lib.rs");
    }

    #[test]
    fn test_normalize_path_relative_with_dotslash() {
        let result = normalize_path("./src/lib.rs").unwrap();
        // Relative paths pass through as-is (no stripping of ./)
        assert_eq!(result, "./src/lib.rs");
    }

    // ── resolve_ref ──

    #[test]
    fn test_resolve_ref_branch_override() {
        // When override is provided, returns it directly without calling git
        let (git_ref, is_sha) = resolve_ref(Some("main")).unwrap();
        assert_eq!(git_ref, "main");
        assert!(!is_sha);
    }

    #[test]
    fn test_resolve_ref_branch_override_is_not_sha() {
        let (_, is_sha) = resolve_ref(Some("feature/my-branch")).unwrap();
        assert!(!is_sha);
    }

    // --- build_pr_url tests ---

    #[test]
    fn test_build_pr_url_github() {
        let url = build_pr_url(&ForgeType::Github, "github.com", "alice", "repo", "123");
        assert_eq!(url, "https://github.com/alice/repo/pull/123");
    }

    #[test]
    fn test_build_pr_url_gitlab() {
        let url = build_pr_url(&ForgeType::Gitlab, "gitlab.com", "alice", "repo", "123");
        assert_eq!(url, "https://gitlab.com/alice/repo/-/merge_requests/123");
    }

    #[test]
    fn test_build_pr_url_gitea() {
        let url = build_pr_url(&ForgeType::Gitea, "gitea.com", "alice", "repo", "123");
        assert_eq!(url, "https://gitea.com/alice/repo/pulls/123");
    }

    #[test]
    fn test_build_pr_url_forgejo() {
        let url = build_pr_url(&ForgeType::Forgejo, "codeberg.org", "alice", "repo", "123");
        assert_eq!(url, "https://codeberg.org/alice/repo/pulls/123");
    }

    // --- build_issue_url tests ---

    #[test]
    fn test_build_issue_url_github() {
        let url = build_issue_url(&ForgeType::Github, "github.com", "alice", "repo", "42");
        assert_eq!(url, "https://github.com/alice/repo/issues/42");
    }

    #[test]
    fn test_build_issue_url_gitlab() {
        let url = build_issue_url(&ForgeType::Gitlab, "gitlab.com", "alice", "repo", "42");
        assert_eq!(url, "https://gitlab.com/alice/repo/-/issues/42");
    }

    #[test]
    fn test_build_issue_url_gitea() {
        let url = build_issue_url(&ForgeType::Gitea, "gitea.com", "alice", "repo", "42");
        assert_eq!(url, "https://gitea.com/alice/repo/issues/42");
    }

    #[test]
    fn test_build_issue_url_forgejo() {
        let url = build_issue_url(&ForgeType::Forgejo, "codeberg.org", "alice", "repo", "42");
        assert_eq!(url, "https://codeberg.org/alice/repo/issues/42");
    }

    // --- Self-hosted PR/issue URL tests ---

    #[test]
    fn test_build_pr_url_self_hosted_gitlab() {
        let url = build_pr_url(
            &ForgeType::Gitlab,
            "gitlab.company.com",
            "team",
            "project",
            "99",
        );
        assert_eq!(
            url,
            "https://gitlab.company.com/team/project/-/merge_requests/99"
        );
    }

    #[test]
    fn test_build_issue_url_self_hosted_gitlab() {
        let url = build_issue_url(
            &ForgeType::Gitlab,
            "gitlab.company.com",
            "team",
            "project",
            "7",
        );
        assert_eq!(url, "https://gitlab.company.com/team/project/-/issues/7");
    }

    // --- Clap conflict tests ---

    #[test]
    fn test_browse_pr_conflicts_with_file() {
        let result = crate::cmd::build_cli().try_get_matches_from([
            "gf",
            "browse",
            "--pr",
            "123",
            "src/main.rs",
        ]);
        assert!(result.is_err(), "browse --pr should conflict with file arg");
    }

    #[test]
    fn test_browse_pr_conflicts_with_branch() {
        let result = crate::cmd::build_cli()
            .try_get_matches_from(["gf", "browse", "--pr", "123", "--branch", "main"]);
        assert!(result.is_err(), "browse --pr should conflict with --branch");
    }

    #[test]
    fn test_browse_issue_conflicts_with_pr() {
        let result = crate::cmd::build_cli()
            .try_get_matches_from(["gf", "browse", "--issue", "42", "--pr", "123"]);
        assert!(result.is_err(), "browse --issue should conflict with --pr");
    }

    #[test]
    fn test_browse_mr_alias_works() {
        let result = crate::cmd::build_cli().try_get_matches_from(["gf", "browse", "--mr", "123"]);
        assert!(result.is_ok(), "browse --mr should parse as alias for --pr");
        let matches = result.unwrap();
        let (_, sub) = matches.subcommand().unwrap();
        let pr_val = sub.get_one::<String>("pr").unwrap();
        assert_eq!(pr_val, "123");
    }

    // ── resolve_forge_type: self-hosted via config ──
    // These tests modify HOME env var and must not run in parallel.
    static HOME_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_resolve_forge_type_self_hosted_via_config() {
        let _lock = HOME_MUTEX.lock().unwrap();
        use std::io::Write;
        let tmp_dir = std::path::PathBuf::from("/tmp/gf-test-phase5-config");
        let config_dir = tmp_dir.join(".config/gf");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(
            f,
            "[[forge]]\ndomain = \"git.mycompany.com\"\ntype = \"gitlab\""
        )
        .unwrap();
        unsafe { std::env::set_var("HOME", &tmp_dir) };
        let result = resolve_forge_type("git.mycompany.com").unwrap();
        assert_eq!(result, ForgeType::Gitlab);
        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_resolve_forge_type_self_hosted_unknown_still_errors() {
        let _lock = HOME_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("HOME", "/tmp/gf-test-phase5-empty") };
        let result = resolve_forge_type("unknown.example.com");
        assert!(matches!(result, Err(GfError::ForgeNotDetected { .. })));
    }
}
