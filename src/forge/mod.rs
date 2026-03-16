#![allow(dead_code)]

use crate::error::GfError;

/// The four supported forge types.
#[derive(Debug, PartialEq, Clone, Copy)]
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
    // TODO: implement in plan 02
    let _ = remote;
    Err(GfError::NotAGitRepo)
}

/// Extracts the hostname from HTTPS or SCP-style git remote URLs.
/// Strips port numbers from HTTPS hostnames (e.g., "host:8443" → "host").
fn parse_host(url: &str) -> Result<String, GfError> {
    // TODO: implement in plan 02
    let _ = url;
    Err(GfError::RemoteUrlUnrecognized("stub".to_string()))
}

/// Checks ~/.config/gf/config.toml for a domain-to-forge mapping.
/// Returns Ok(None) if config file is absent (not an error).
fn config_lookup(host: &str) -> Result<Option<ForgeType>, GfError> {
    // TODO: implement in plan 03
    let _ = host;
    Ok(None)
}

/// Matches a hostname against the four built-in public forge entries.
fn match_known_host(host: &str) -> Result<ForgeType, GfError> {
    // TODO: implement in plan 02
    let _ = host;
    Err(GfError::ForgeNotDetected {
        domain: host.to_string(),
    })
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
        assert_eq!(match_known_host("codeberg.org").unwrap(), ForgeType::Forgejo);
    }

    #[test]
    fn test_known_host_unknown_returns_error() {
        let result = match_known_host("unknown.example.com");
        assert!(matches!(result, Err(GfError::ForgeNotDetected { .. })));
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
}
