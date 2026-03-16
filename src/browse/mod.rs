//! Native browse command — constructs forge URLs and opens them in the browser.
//! Does NOT delegate to gh/glab/tea/fj (BROWSE-05).

use crate::error::GfError;
use crate::forge::{config_lookup, parse_remote_parts, ForgeType};
use clap::ArgMatches;

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

    // 4. Resolve branch/SHA ref
    let branch_override = matches.get_one::<String>("branch").map(|s| s.as_str());
    let (git_ref, is_sha) = resolve_ref(branch_override)?;

    // 5. Build URL
    let file_arg = matches.get_one::<String>("file").map(|s| s.as_str());
    let url = if let Some(file) = file_arg {
        let normalized = normalize_path(file)?;
        build_file_url(&forge_type, &host, &owner, &repo, &git_ref, is_sha, &normalized)
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
/// File URL format:
///   GitHub:  https://host/owner/repo/blob/<ref>/<path>
///   GitLab:  https://host/owner/repo/-/blob/<ref>/<path>
///   Gitea:   https://host/owner/repo/src/branch/<ref>/<path>  (or src/commit/ if SHA)
///   Forgejo: https://host/owner/repo/src/branch/<ref>/<path>  (or src/commit/ if SHA)
pub fn build_file_url(
    forge: &ForgeType,
    host: &str,
    owner: &str,
    repo: &str,
    git_ref: &str,
    is_sha: bool,
    path: &str,
) -> String {
    let base = format!("https://{host}/{owner}/{repo}");
    match forge {
        ForgeType::Github => format!("{base}/blob/{git_ref}/{path}"),
        ForgeType::Gitlab => format!("{base}/-/blob/{git_ref}/{path}"),
        ForgeType::Gitea | ForgeType::Forgejo => {
            if is_sha {
                format!("{base}/src/commit/{git_ref}/{path}")
            } else {
                format!("{base}/src/branch/{git_ref}/{path}")
            }
        }
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

/// Resolves the ForgeType for a given host.
/// Config lookup first (CORE-05) — mirrors forge::detect() priority.
fn resolve_forge_type(host: &str) -> Result<ForgeType, GfError> {
    // Config lookup first (CORE-05) — mirrors forge::detect() priority
    if let Some(forge_type) = config_lookup(host)? {
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
        let stripped = path
            .strip_prefix(&toplevel)
            .ok_or_else(|| {
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
        let url = build_repo_url(&ForgeType::Forgejo, "codeberg.org", "alice", "myrepo", "main");
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

    // ── build_file_url ──

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
        );
        assert_eq!(url, "https://gitlab.com/alice/myrepo/-/blob/main/src/lib.rs");
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
        );
        assert_eq!(
            url,
            format!("https://codeberg.org/alice/myrepo/src/commit/{sha}/src/lib.rs")
        );
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

    // ── resolve_forge_type: self-hosted via config ──

    #[test]
    fn test_resolve_forge_type_self_hosted_via_config() {
        use std::io::Write;
        let tmp_dir = std::path::PathBuf::from("/tmp/gf-test-phase5-config");
        let config_dir = tmp_dir.join(".config/gf");
        std::fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("config.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "[[forge]]\ndomain = \"git.mycompany.com\"\ntype = \"gitlab\"").unwrap();
        unsafe { std::env::set_var("HOME", &tmp_dir) };
        let result = resolve_forge_type("git.mycompany.com").unwrap();
        assert_eq!(result, ForgeType::Gitlab);
        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_resolve_forge_type_self_hosted_unknown_still_errors() {
        unsafe { std::env::set_var("HOME", "/tmp/gf-test-phase5-empty") };
        let result = resolve_forge_type("unknown.example.com");
        assert!(matches!(result, Err(GfError::ForgeNotDetected { .. })));
    }
}
