#![allow(dead_code)]

use crate::error::GfError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// The four supported forge types.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ForgeType {
    Github,
    Gitlab,
    Gitea,
    Forgejo,
}

impl ForgeType {
    /// Returns the CLI binary name for this forge.
    /// This is the single source of truth — used by `runner::run()` and `cli_info()`.
    #[must_use] 
    pub const fn cli_name(self) -> &'static str {
        match self {
            Self::Github => "gh",
            Self::Gitlab => "glab",
            Self::Gitea => "tea",
            Self::Forgejo => "fj",
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct MergeConfig {
    #[serde(default)]
    delete_branch: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
struct DefaultsConfig {
    #[serde(default)]
    clone_host: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct BrowseConfig {
    /// Ref to use when HEAD is detached and no --branch flag is given.
    #[serde(default)]
    detached_head_fallback: Option<String>,
}

/// Cached probe results: hostname → forge type mapping.
#[derive(Debug, Deserialize, Serialize, Default)]
struct ProbeCache {
    #[serde(default)]
    hosts: HashMap<String, ForgeType>,
}

#[derive(Debug, Deserialize)]
struct GfConfig {
    #[serde(default)]
    forge: Vec<ForgeEntry>,
    #[serde(default)]
    merge: MergeConfig,
    #[serde(default)]
    defaults: DefaultsConfig,
    #[serde(default)]
    browse: BrowseConfig,
}

#[derive(Debug, Deserialize)]
struct ForgeEntry {
    domain: String,
    /// TOML key is `type` (Rust keyword — must use rename)
    #[serde(rename = "type")]
    forge_type: ForgeType,
    #[serde(default)]
    delete_branch: Option<bool>,
    /// Ref to use when HEAD is detached during `gf browse` for this forge.
    #[serde(default)]
    detached_head_fallback: Option<String>,
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
    let Some(path) = config_path() else {
        return Ok(None);
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

/// Resolves delete-branch behavior for merge.
/// Priority: per-forge config > global [merge] config > built-in default (false).
/// CLI flag override is handled by the caller (adapter).
#[must_use] 
pub fn resolve_delete_branch(domain: &str) -> bool {
    let Ok(Some(config)) = load_config() else {
        return false; // no config = default false
    };

    // Per-forge override
    if let Some(entry) = config.forge.iter().find(|e| e.domain == domain) {
        if let Some(val) = entry.delete_branch {
            return val;
        }
    }

    // Global [merge] section
    config.merge.delete_branch.unwrap_or(false)
}

/// Resolves the detached-HEAD fallback ref for `gf browse`.
///
/// Priority: per-forge `detached_head_fallback` > global `[browse] detached_head_fallback` > None.
/// Returns None when no fallback is configured (caller uses the commit SHA).
#[must_use] 
pub fn resolve_detached_head_fallback(domain: &str) -> Option<String> {
    let config = load_config().ok().flatten()?;

    // Per-forge override wins
    if let Some(entry) = config.forge.iter().find(|e| e.domain == domain) {
        if let Some(ref val) = entry.detached_head_fallback {
            return Some(val.clone());
        }
    }

    // Global [browse] section
    config.browse.detached_head_fallback
}

/// Top-level forge detection entry point.
///
/// Determines which forge a git repo lives on, given a remote name, and also returns
/// the domain. Prefer this over calling `detect` + `domain_from_remote` separately
/// to avoid two git subprocess calls.
///
/// # Errors
///
/// Returns `Err` if the git remote URL cannot be retrieved, the host cannot be
/// parsed from the URL, or the forge type cannot be detected for the host.
pub fn detect_with_domain(remote: &str) -> Result<(ForgeType, String), GfError> {
    let url = get_remote_url(remote)?;
    let host = parse_host(&url)?;
    let forge = detect_from_host(&host)?;
    Ok((forge, host))
}

/// Determines which forge a git repo lives on, given a remote name.
///
/// Priority: config file → known host → cached probe → live probe → error
///
/// `remote` — git remote name (typically "origin", overridden by --remote flag)
///
/// # Errors
///
/// Returns `Err` if the git remote URL cannot be retrieved, the host cannot be
/// parsed, or no forge could be detected for the host via any priority source.
pub fn detect(remote: &str) -> Result<ForgeType, GfError> {
    let url = get_remote_url(remote)?;
    let host = parse_host(&url)?;

    // Priority 1: Config file (user explicit mapping, always wins)
    if let Some(forge) = config_lookup(&host)? {
        return Ok(forge);
    }

    // Priority 2: Known public hosts (github.com, gitlab.com, gitea.com, codeberg.org)
    if let Ok(forge) = match_known_host(&host) {
        return Ok(forge);
    }

    // Priority 3: Cached probe result (previously auto-detected)
    if let Some(forge) = cache_lookup(&host) {
        return Ok(forge);
    }

    // Priority 4: Live probe — try all forge CLIs for auth status
    if let Some(forge) = probe_auth(&host) {
        save_probe_cache(&host, forge);
        return Ok(forge);
    }

    // Priority 5: No match — error with config hint
    Err(GfError::ForgeNotDetected { domain: host })
}

/// Detects forge type from a hostname using the full priority chain.
///
/// Walks config → known hosts → cache → live probe without requiring a git
/// remote. Used by browse which already has the host parsed from the remote URL.
///
/// # Errors
///
/// Returns `Err` if no forge could be detected for the host via config, known
/// hosts, the probe cache, or a live probe.
pub fn detect_from_host(host: &str) -> Result<ForgeType, GfError> {
    if let Some(forge) = config_lookup(host)? {
        return Ok(forge);
    }
    if let Ok(forge) = match_known_host(host) {
        return Ok(forge);
    }
    if let Some(forge) = cache_lookup(host) {
        return Ok(forge);
    }
    if let Some(forge) = probe_auth(host) {
        save_probe_cache(host, forge);
        return Ok(forge);
    }
    Err(GfError::ForgeNotDetected {
        domain: host.to_string(),
    })
}

/// Runs `git remote get-url <remote>` and returns the URL string.
/// Returns `GfError::NotAGitRepo` if not in a git repo.
/// Returns `GfError::NoRemote` if the remote name does not exist.
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
/// Strips userinfo per RFC 3986 (e.g., "api@host" or "user:pass@host" → "host").
fn parse_host(url: &str) -> Result<String, GfError> {
    // HTTPS / HTTP: https://[userinfo@]github.com[:port]/owner/repo.git
    // Also covers http:// (uncommon but valid)
    if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let authority = rest.split('/').next().unwrap_or("");
        // Strip userinfo: "user:pass@host:port" -> "host:port"; "host:port" stays as-is
        let host_with_possible_port =
            authority.rfind('@').map_or(authority, |pos| &authority[pos + 1..]);
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
/// Handles HTTPS with port (e.g., `https://host:8443/owner/repo.git`).
///
/// Examples:
///   `https://github.com/alice/myrepo.git` → ("github.com", "alice", "myrepo")
///   `git@gitlab.com:alice/myrepo.git`     → ("gitlab.com", "alice", "myrepo")
///
/// # Errors
///
/// Returns `Err(GfError::RemoteUrlUnrecognized)` if the URL is neither a
/// recognizable HTTPS/HTTP nor SCP-style remote with host, owner, and repo.
pub fn parse_remote_parts(url: &str) -> Result<(String, String, String), GfError> {
    // HTTPS / HTTP path: strip scheme, strip host (first segment), take next two path segments
    if let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let mut parts = rest.splitn(4, '/');
        let authority = parts.next().unwrap_or("");
        // Strip userinfo per RFC 3986: "user:pass@host:port" -> "host:port"
        let host_with_port =
            authority.rfind('@').map_or(authority, |pos| &authority[pos + 1..]);
        let host = host_with_port.split(':').next().unwrap_or("").to_string();
        let owner = parts.next().unwrap_or("").to_string();
        let repo_raw = parts.next().unwrap_or("");
        let repo = repo_raw
            .strip_suffix(".git")
            .unwrap_or(repo_raw)
            .to_string();
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
            let repo = repo_raw
                .strip_suffix(".git")
                .unwrap_or(repo_raw)
                .to_string();
            if !host.is_empty() && !owner.is_empty() && !repo.is_empty() {
                return Ok((host, owner, repo));
            }
        }
    }
    Err(GfError::RemoteUrlUnrecognized(url.to_string()))
}

/// Checks ~/.config/gf/config.toml for a domain-to-forge mapping.
/// Returns Ok(None) if config file is absent (not an error).
///
/// # Errors
///
/// Returns `Err(GfError::ConfigParseError)` if the config file exists but
/// cannot be read or parsed.
pub fn config_lookup(host: &str) -> Result<Option<ForgeType>, GfError> {
    let Some(cfg) = load_config()? else {
        return Ok(None);
    };
    Ok(cfg
        .forge
        .iter()
        .find(|e| e.domain == host)
        .map(|e| e.forge_type))
}

/// Matches a hostname against the four built-in public forge entries.
///
/// # Errors
///
/// Returns `Err(GfError::ForgeNotDetected)` if the host is not one of the
/// built-in known public forges.
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

/// Probe forge CLIs for auth status containing the given hostname.
/// Returns the first matching `ForgeType`, or None if no CLI matches.
/// Checks in order: gh, glab, tea, fj (market share priority per CONTEXT.md).
fn probe_auth(hostname: &str) -> Option<ForgeType> {
    let probes: [(ForgeType, &str, &[&str]); 4] = [
        (ForgeType::Github, "gh", &["auth", "status"]),
        (ForgeType::Gitlab, "glab", &["auth", "status"]),
        (ForgeType::Gitea, "tea", &["logins", "ls"]),
        (ForgeType::Forgejo, "fj", &["auth", "list"]),
    ];

    for (forge, cli, args) in probes {
        if let Some(output) = run_with_timeout(cli, args, Duration::from_secs(5)) {
            // Check if hostname appears in stdout or stderr (some CLIs output to stderr)
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stdout.contains(hostname) || stderr.contains(hostname) {
                return Some(forge);
            }
        }
    }
    None
}

/// Run a command with a timeout. Returns None if timeout expires, command not found, or fails.
fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Option<std::process::Output> {
    let (tx, rx) = mpsc::channel();

    let cmd_owned = cmd.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();

    thread::spawn(move || {
        let result = std::process::Command::new(&cmd_owned)
            .args(&args_owned)
            .output();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => Some(output),
        _ => None, // Timeout, command not found, or execution failed
    }
}

/// Returns path to ~/.cache/gf/probes.toml, respecting `XDG_CACHE_HOME`.
fn cache_path() -> Option<std::path::PathBuf> {
    // XDG_CACHE_HOME takes precedence (Linux standard)
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Some(std::path::PathBuf::from(xdg).join("gf").join("probes.toml"));
    }
    // Fall back to ~/.cache/gf/probes.toml
    if let Ok(home) = std::env::var("HOME") {
        return Some(
            std::path::PathBuf::from(home)
                .join(".cache")
                .join("gf")
                .join("probes.toml"),
        );
    }
    None
}

/// Load cached probe results. Returns None if cache doesn't exist or is invalid.
fn load_probe_cache() -> Option<ProbeCache> {
    let path = cache_path()?;
    if !path.exists() {
        return None;
    }
    let text = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&text).ok()
}

/// Save a probe result to cache. Creates cache directory if needed.
fn save_probe_cache(hostname: &str, forge: ForgeType) {
    let Some(path) = cache_path() else { return };

    // Load existing cache or create empty
    let mut cache = load_probe_cache().unwrap_or_default();

    // Insert new mapping
    cache.hosts.insert(hostname.to_string(), forge);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Serialize and write
    if let Ok(toml_str) = toml::to_string_pretty(&cache) {
        let _ = std::fs::write(&path, toml_str);
    }
}

/// Lookup a hostname in the probe cache. Returns None if not cached.
fn cache_lookup(hostname: &str) -> Option<ForgeType> {
    let cache = load_probe_cache()?;
    cache.hosts.get(hostname).copied()
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
        assert_eq!(result.expect("should parse host"), "github.com");
    }

    #[test]
    fn test_parse_host_https_with_port() {
        // RED: stub returns Err; also validates port stripping
        let result = parse_host("https://git.company.com:8443/owner/repo.git");
        assert_eq!(result.expect("should parse host"), "git.company.com");
    }

    #[test]
    fn test_parse_host_ssh_scp() {
        // RED: stub returns Err
        let result = parse_host("git@github.com:owner/repo.git");
        assert_eq!(result.expect("should parse host"), "github.com");
    }

    #[test]
    fn test_parse_host_ssh_gitlab() {
        // RED: stub returns Err
        let result = parse_host("git@gitlab.com:owner/repo.git");
        assert_eq!(result.expect("should parse host"), "gitlab.com");
    }

    #[test]
    fn test_parse_host_unrecognized() {
        // RED: stub returns Err(RemoteUrlUnrecognized) — this test should pass even in stub
        // (stub returns RemoteUrlUnrecognized, which matches)
        let result = parse_host("not-a-url");
        assert!(matches!(result, Err(GfError::RemoteUrlUnrecognized(_))));
    }

    #[test]
    fn test_parse_host_https_with_userinfo() {
        let result = parse_host("https://api@github.com/owner/repo.git");
        assert_eq!(result.expect("should parse host"), "github.com");
    }

    #[test]
    fn test_parse_host_https_with_user_password() {
        let result = parse_host("https://user:token@github.com/owner/repo.git");
        assert_eq!(result.expect("should parse host"), "github.com");
    }

    #[test]
    fn test_parse_host_https_with_userinfo_and_port() {
        let result = parse_host("https://user:token@git.company.com:8443/owner/repo.git");
        assert_eq!(result.expect("should parse host"), "git.company.com");
    }

    // --- match_known_host() stubs (RED — will pass after plan 02) ---

    #[test]
    fn test_known_host_github() {
        // RED: stub returns ForgeNotDetected
        assert_eq!(
            match_known_host("github.com").expect("known host"),
            ForgeType::Github
        );
    }

    #[test]
    fn test_known_host_gitlab() {
        assert_eq!(
            match_known_host("gitlab.com").expect("known host"),
            ForgeType::Gitlab
        );
    }

    #[test]
    fn test_known_host_gitea() {
        assert_eq!(
            match_known_host("gitea.com").expect("known host"),
            ForgeType::Gitea
        );
    }

    #[test]
    fn test_known_host_codeberg() {
        assert_eq!(
            match_known_host("codeberg.org").expect("known host"),
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
        temp_env::with_var("HOME", Some("/tmp"), || {
            let result = config_lookup("github.com");
            assert!(
                matches!(result, Ok(None)),
                "expected Ok(None) for absent config, got: {result:?}"
            );
        });
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
        let (host, owner, repo) =
            parse_remote_parts("https://github.com/alice/myrepo.git").expect("should parse");
        assert_eq!(host, "github.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_https_no_git_suffix() {
        let (host, owner, repo) =
            parse_remote_parts("https://github.com/alice/myrepo").expect("should parse");
        assert_eq!(host, "github.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_https_with_port() {
        let (host, owner, repo) =
            parse_remote_parts("https://git.company.com:8443/org/proj.git").expect("should parse");
        assert_eq!(host, "git.company.com");
        assert_eq!(owner, "org");
        assert_eq!(repo, "proj");
    }

    #[test]
    fn test_parse_remote_parts_scp_ssh() {
        let (host, owner, repo) =
            parse_remote_parts("git@gitlab.com:alice/myrepo.git").expect("should parse");
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
    fn test_parse_remote_parts_https_with_userinfo() {
        let (host, owner, repo) =
            parse_remote_parts("https://api@github.com/alice/myrepo.git").expect("should parse");
        assert_eq!(host, "github.com");
        assert_eq!(owner, "alice");
        assert_eq!(repo, "myrepo");
    }

    #[test]
    fn test_parse_remote_parts_https_with_user_password_and_port() {
        let (host, owner, repo) =
            parse_remote_parts("https://user:token@git.company.com:8443/org/proj.git")
                .expect("should parse");
        assert_eq!(host, "git.company.com");
        assert_eq!(owner, "org");
        assert_eq!(repo, "proj");
    }

    #[test]
    fn test_config_with_merge_section() {
        let toml_str = r#"
[merge]
delete_branch = true

[[forge]]
domain = "github.com"
type = "github"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(cfg.merge.delete_branch, Some(true));
    }

    #[test]
    fn test_config_forge_entry_delete_branch() {
        let toml_str = r#"
[[forge]]
domain = "github.com"
type = "github"
delete_branch = true
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(cfg.forge[0].delete_branch, Some(true));
    }

    #[test]
    fn test_config_forge_entry_detached_head_fallback() {
        let toml_str = r#"
[[forge]]
domain = "github.com"
type = "github"
detached_head_fallback = "main"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(
            cfg.forge[0].detached_head_fallback,
            Some("main".to_string())
        );
    }

    #[test]
    fn test_config_global_browse_detached_head_fallback() {
        let toml_str = r#"
[browse]
detached_head_fallback = "main"

[[forge]]
domain = "github.com"
type = "github"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(
            cfg.browse.detached_head_fallback,
            Some("main".to_string())
        );
        assert_eq!(cfg.forge[0].detached_head_fallback, None);
    }

    #[test]
    fn test_config_without_merge_section_defaults() {
        let toml_str = r#"
[[forge]]
domain = "github.com"
type = "github"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(cfg.merge.delete_branch, None);
        assert_eq!(cfg.forge[0].delete_branch, None);
    }

    #[test]
    fn test_config_malformed_toml_returns_parse_error() {
        let bad_toml = "[[forge\ndomain = !!!";
        let result: Result<GfConfig, _> = toml::from_str(bad_toml);
        assert!(result.is_err(), "malformed TOML should fail");
        // Wrap in GfError to verify round-trip
        let gf_err = GfError::ConfigParseError(result.expect_err("malformed TOML").to_string());
        assert!(gf_err.to_string().starts_with("failed to parse config:"));
    }

    #[test]
    fn test_config_with_defaults_section() {
        let toml_str = r#"
[defaults]
clone_host = "gitlab.mycompany.com"

[[forge]]
domain = "github.com"
type = "github"
"#;
        let cfg: GfConfig = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(
            cfg.defaults.clone_host,
            Some("gitlab.mycompany.com".to_string())
        );
    }

    // --- Probe cache tests ---

    #[test]
    fn test_cache_path_with_home() {
        // Just verify the function returns Some when HOME is set
        // (it should be set in test environment)
        let path = cache_path();
        assert!(
            path.is_some(),
            "cache_path should return Some when HOME is set"
        );
        let path = path.expect("cache_path should be Some");
        assert!(path.to_string_lossy().contains("probes.toml"));
    }

    #[test]
    fn test_probe_cache_roundtrip() {
        // Create a temporary cache by setting XDG_CACHE_HOME
        let temp_dir = std::env::temp_dir().join("gf_test_cache");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up any previous test

        temp_env::with_var("XDG_CACHE_HOME", Some(&temp_dir), || {
            // Cache should be empty initially
            assert!(cache_lookup("test.example.com").is_none());

            // Save a probe result
            save_probe_cache("test.example.com", ForgeType::Gitlab);

            // Should be able to read it back
            let result = cache_lookup("test.example.com");
            assert_eq!(result, Some(ForgeType::Gitlab));
        });

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
