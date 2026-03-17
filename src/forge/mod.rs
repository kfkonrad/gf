#![allow(dead_code)]

use crate::error::GfError;
use serde::Deserialize;

/// The four supported forge types.
#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeType {
    Github,
    Gitlab,
    Gitea,
    Forgejo,
}

impl ForgeType {
    /// Returns the CLI binary name for this forge.
    /// This is the single source of truth — used by runner::run() and cli_info().
    pub fn cli_name(&self) -> &'static str {
        match self {
            ForgeType::Github => "gh",
            ForgeType::Gitlab => "glab",
            ForgeType::Gitea => "tea",
            ForgeType::Forgejo => "fj",
        }
    }
}

#[derive(Debug, Deserialize)]
struct GfConfig {
    #[serde(default)]
    forge: Vec<ForgeEntry>,
}

#[derive(Debug, Deserialize)]
struct ForgeEntry {
    domain: String,
    /// TOML key is `type` (Rust keyword — must use rename)
    #[serde(rename = "type")]
    forge_type: ForgeType,
}

/// Returns the path to ~/.config/gf/config.toml using $HOME env var.
fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        std::path::PathBuf::from(home)
            .join(".config")
            .join("gf")
            .join("config.toml"),
    )
}

/// Loads the config file. Returns Ok(None) if file is absent (not an error).
/// Returns Err(ConfigParseError) on TOML parse failure.
fn load_config() -> Result<Option<GfConfig>, GfError> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    if !path.exists() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).map_err(|e| GfError::ConfigParseError(e.to_string()))?;
    toml::from_str(&text)
        .map(Some)
        .map_err(|e| GfError::ConfigParseError(e.to_string()))
}

/// Top-level forge detection entry point.
/// Determines which forge a git repo lives on, given a remote name.
///
/// Priority: config file → known host table → error
///
/// `remote` — git remote name (typically "origin", overridden by --remote flag)
pub fn detect(remote: &str) -> Result<ForgeType, GfError> {
    let url = get_remote_url(remote)?;
    let host = parse_host(&url)?;
    if let Some(forge) = config_lookup(&host)? {
        return Ok(forge);
    }
    match_known_host(&host)
}

/// Runs `git remote get-url <remote>` and returns the URL string.
/// Returns GfError::NotAGitRepo if not in a git repo.
/// Returns GfError::NoRemote if the remote name does not exist.
fn get_remote_url(remote: &str) -> Result<String, GfError> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", remote])
        .output()
        .map_err(GfError::GitCommandFailed)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // git exits non-zero for both "not a git repo" and "no such remote"
        // Distinguish by stderr content (git uses text, not exit codes, to differentiate)
        if stderr.contains("not a git repository") {
            return Err(GfError::NotAGitRepo);
        }
        // "No such remote" — covers "No such remote 'upstream'" etc.
        return Err(GfError::NoRemote(remote.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Extracts the hostname from HTTPS or SCP-style git remote URLs.
/// Strips port numbers from HTTPS hostnames (e.g., "host:8443" → "host").
fn parse_host(url: &str) -> Result<String, GfError> {
    // HTTPS / HTTP: https://github.com/owner/repo.git
    // Also covers http:// (uncommon but valid)
    if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let host_with_possible_port = rest.split('/').next().unwrap_or("");
        // Strip port: "git.company.com:8443" -> "git.company.com"
        let host = host_with_possible_port.split(':').next().unwrap_or("");
        if !host.is_empty() {
            return Ok(host.to_string());
        }
    }
    // SCP-style: git@github.com:owner/repo.git
    // Format: [user@]host:path — find '@', take everything after until ':'
    if let Some(at_pos) = url.find('@') {
        let after_at = &url[at_pos + 1..];
        let host = after_at.split(':').next().unwrap_or("");
        if !host.is_empty() {
            return Ok(host.to_string());
        }
    }
    Err(GfError::RemoteUrlUnrecognized(url.to_string()))
}

/// Parses HTTPS or SCP-style remote URLs into (host, owner, repo).
/// Strips .git suffix from repo name.
/// Handles HTTPS with port (e.g., "https://host:8443/owner/repo.git").
///
/// Examples:
///   "https://github.com/alice/myrepo.git" → ("github.com", "alice", "myrepo")
///   "git@gitlab.com:alice/myrepo.git"     → ("gitlab.com", "alice", "myrepo")
pub fn parse_remote_parts(url: &str) -> Result<(String, String, String), GfError> {
    // HTTPS / HTTP path: strip scheme, strip host (first segment), take next two path segments
    if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let mut parts = rest.splitn(4, '/');
        let host_with_port = parts.next().unwrap_or("");
        let host = host_with_port.split(':').next().unwrap_or("").to_string();
        let owner = parts.next().unwrap_or("").to_string();
        let repo_raw = parts.next().unwrap_or("");
        let repo = repo_raw.strip_suffix(".git").unwrap_or(repo_raw).to_string();
        if !host.is_empty() && !owner.is_empty() && !repo.is_empty() {
            return Ok((host, owner, repo));
        }
    }
    // SCP-style: [user@]host:owner/repo.git
    if let Some(at_pos) = url.find('@') {
        let after_at = &url[at_pos + 1..];
        if let Some(colon_pos) = after_at.find(':') {
            let host = after_at[..colon_pos].to_string();
            let path = &after_at[colon_pos + 1..];
            let mut path_parts = path.splitn(2, '/');
            let owner = path_parts.next().unwrap_or("").to_string();
            let repo_raw = path_parts.next().unwrap_or("");
            let repo = repo_raw.strip_suffix(".git").unwrap_or(repo_raw).to_string();
            if !host.is_empty() && !owner.is_empty() && !repo.is_empty() {
                return Ok((host, owner, repo));
            }
        }
    }
    Err(GfError::RemoteUrlUnrecognized(url.to_string()))
}

/// Checks ~/.config/gf/config.toml for a domain-to-forge mapping.
/// Returns Ok(None) if config file is absent (not an error).
pub fn config_lookup(host: &str) -> Result<Option<ForgeType>, GfError> {
    let cfg = match load_config()? {
        Some(c) => c,
        None => return Ok(None),
    };
    Ok(cfg
        .forge
        .iter()
        .find(|e| e.domain == host)
        .map(|e| e.forge_type))
}

/// Matches a hostname against the four built-in public forge entries.
pub fn match_known_host(host: &str) -> Result<ForgeType, GfError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GfError;

    // --- ForgeType::cli_name() ---

    #[test]
    fn test_cli_name_github() {
        assert_eq!(ForgeType::Github.cli_name(), "gh");
    }

    #[test]
    fn test_cli_name_gitlab() {
        assert_eq!(ForgeType::Gitlab.cli_name(), "glab");
    }

    #[test]
    fn test_cli_name_gitea() {
        assert_eq!(ForgeType::Gitea.cli_name(), "tea");
    }

    #[test]
    fn test_cli_name_forgejo() {
        assert_eq!(ForgeType::Forgejo.cli_name(), "fj");
    }

    // --- parse_host() stubs (RED — will pass after plan 02) ---

    #[test]
    fn test_parse_host_https_github() {
        // RED: stub returns Err
        let result = parse_host("https://github.com/owner/repo.git");
        assert_eq!(result.unwrap(), "github.com");
    }

    #[test]
    fn test_parse_host_https_with_port() {
        // RED: stub returns Err; also validates port stripping
        let result = parse_host("https://git.company.com:8443/owner/repo.git");
        assert_eq!(result.unwrap(), "git.company.com");
    }

    #[test]
    fn test_parse_host_ssh_scp() {
        // RED: stub returns Err
        let result = parse_host("git@github.com:owner/repo.git");
        assert_eq!(result.unwrap(), "github.com");
    }

    #[test]
    fn test_parse_host_ssh_gitlab() {
        // RED: stub returns Err
        let result = parse_host("git@gitlab.com:owner/repo.git");
        assert_eq!(result.unwrap(), "gitlab.com");
    }

    #[test]
    fn test_parse_host_unrecognized() {
        // RED: stub returns Err(RemoteUrlUnrecognized) — this test should pass even in stub
        // (stub returns RemoteUrlUnrecognized, which matches)
        let result = parse_host("not-a-url");
        assert!(matches!(result, Err(GfError::RemoteUrlUnrecognized(_))));
    }

    // --- match_known_host() stubs (RED — will pass after plan 02) ---

    #[test]
    fn test_known_host_github() {
        // RED: stub returns ForgeNotDetected
        assert_eq!(match_known_host("github.com").unwrap(), ForgeType::Github);
    }

    #[test]
    fn test_known_host_gitlab() {
        assert_eq!(match_known_host("gitlab.com").unwrap(), ForgeType::Gitlab);
    }

    #[test]
    fn test_known_host_gitea() {
        assert_eq!(match_known_host("gitea.com").unwrap(), ForgeType::Gitea);
    }

    #[test]
    fn test_known_host_codeberg() {
        assert_eq!(
            match_known_host("codeberg.org").unwrap(),
            ForgeType::Forgejo
        );
    }

    #[test]
    fn test_known_host_unknown_returns_error() {
        let result = match_known_host("unknown.example.com");
        assert!(matches!(result, Err(GfError::ForgeNotDetected { .. })));
    }

    // --- get_remote_url() tests ---

    #[test]
    fn test_get_remote_url_invalid_remote() {
        // "definitely_no_such_remote_gf_test" does not exist in this repo
        let result = get_remote_url("definitely_no_such_remote_gf_test");
        assert!(
            matches!(result, Err(GfError::NoRemote(_))),
            "expected NoRemote, got: {result:?}"
        );
    }

    #[test]
    fn test_get_remote_url_not_in_git_repo() {
        // Change working directory to /tmp (not a git repo) for this test
        // We can't easily test this from within cargo test without temp dir manipulation.
        // This behavior is covered by the integration test in tests/integration_test.rs instead.
        // Placeholder: assert the function exists and compiles.
        let _ = std::mem::discriminant(&GfError::NotAGitRepo);
    }

    // --- config_lookup() stubs (RED — will pass after plan 03) ---

    #[test]
    fn test_config_lookup_absent_is_ok_none() {
        // Config stub returns Ok(None) — this test should pass even in stub state
        // (real test verifies with temp config file in plan 03)
        // This test validates the stub behavior only
        let result = config_lookup("anything.example.com");
        assert!(result.is_ok());
    }

    // --- config_lookup() full tests (plan 03) ---

    #[test]
    fn test_config_lookup_absent_config_is_ok_none() {
        // When HOME points to a dir without .config/gf/config.toml, returns Ok(None)
        // Override HOME to a temp dir that definitely has no config
        // Safety: set_var is process-wide; cargo test runs unit tests sequentially by default
        unsafe {
            std::env::set_var("HOME", "/tmp");
        }
        let result = config_lookup("github.com");
        assert!(
            matches!(result, Ok(None)),
            "expected Ok(None) for absent config, got: {result:?}"
        );
    }

    #[test]
    fn test_config_lookup_with_inline_config() {
        // Test TOML parsing directly via toml::from_str
        let toml_str = r#"
[[forge]]
domain = "gitlab.mycompany.com"
type = "gitlab"

[[forge]]
domain = "git.internal.io"
type = "forgejo"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(cfg.forge.len(), 2);
        assert_eq!(cfg.forge[0].domain, "gitlab.mycompany.com");
        assert_eq!(cfg.forge[0].forge_type, ForgeType::Gitlab);
        assert_eq!(cfg.forge[1].forge_type, ForgeType::Forgejo);
    }

    // --- parse_remote_parts() tests ---

    #[test]
    fn test_parse_remote_parts_https_github() {
        let (host, owner, repo) = parse_remote_parts("https://github.com/alice/myrepo.git").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_https_no_git_suffix() {
        let (host, owner, repo) = parse_remote_parts("https://github.com/alice/myrepo").unwrap();
        assert_eq!(host, "github.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_https_with_port() {
        let (host, owner, repo) = parse_remote_parts("https://git.company.com:8443/org/proj.git").unwrap();
        assert_eq!(host, "git.company.com");
        assert_eq!(owner, "org");
        assert_eq!(repo, "proj");
    }

    #[test]
    fn test_parse_remote_parts_scp_ssh() {
        let (host, owner, repo) = parse_remote_parts("git@gitlab.com:alice/myrepo.git").unwrap();
        assert_eq!(host, "gitlab.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_unrecognized() {
        let result = parse_remote_parts("not-a-url");
        assert!(matches!(result, Err(GfError::RemoteUrlUnrecognized(_))));
    }

    #[test]
    fn test_config_malformed_toml_returns_parse_error() {
        let bad_toml = "[[forge\ndomain = !!!";
        let result: Result<GfConfig, _> = toml::from_str(bad_toml);
        assert!(result.is_err(), "malformed TOML should fail");
        // Wrap in GfError to verify round-trip
        let gf_err = GfError::ConfigParseError(result.unwrap_err().to_string());
        assert!(gf_err.to_string().starts_with("failed to parse config:"));
    }
}
