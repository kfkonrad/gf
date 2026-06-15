use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GfError {
    #[error("{cli} not found\nInstall with: brew install {brew_name}\nOr see: {url}")]
    CliNotFound {
        cli: String,
        brew_name: String,
        url: String,
    },

    #[error("failed to exec {0}: {1}")]
    ExecFailed(String, std::io::Error),

    #[error("failed to spawn {0}: {1}")]
    #[cfg_attr(not(windows), allow(dead_code))]
    SpawnFailed(String, std::io::Error),

    #[error("not a git repository (or any parent directory)")]
    NotAGitRepo,

    #[error("no remote named '{0}' — use --remote to specify one")]
    NoRemote(String),

    #[error("Could not detect forge for: {domain}\n\nSupported forges: github, gitlab, gitea, forgejo\n\nAdd a mapping to ~/.config/gf/config.toml:\n  [[forge]]\n  domain = \"{domain}\"\n  type = \"forgejo\"  # or github, gitlab, gitea")]
    ForgeNotDetected { domain: String },

    #[error("failed to parse config: {0}")]
    ConfigParseError(String),

    #[error("remote URL not recognized: {0}")]
    RemoteUrlUnrecognized(String),

    #[error("git command failed: {0}")]
    GitCommandFailed(#[from] io::Error),

    #[error("failed to open browser for {0}: {1}")]
    BrowseFailed(String, #[source] std::io::Error),

    #[error("cannot construct browse URL: {0}")]
    BrowseUrlConstructionFailed(String),

    #[error("`gf {feature}` is not supported on {forge}\n\n{forge_cli} does not have an equivalent for this command/flag.")]
    UnsupportedFeature {
        feature: String,
        forge: String,
        forge_cli: String,
    },
}

/// Known forge CLI info for install hints.
pub struct CliInfo {
    pub brew_name: String,
    pub url: &'static str,
}

/// Return install hint info for a known CLI, or a sensible default.
#[must_use] 
pub fn cli_info(cli: &str) -> CliInfo {
    match cli {
        "gh" => CliInfo {
            brew_name: "gh".to_string(),
            url: "https://cli.github.com",
        },
        "glab" => CliInfo {
            brew_name: "glab".to_string(),
            url: "https://gitlab.com/gitlab-org/cli",
        },
        "tea" => CliInfo {
            brew_name: "tea".to_string(),
            url: "https://gitea.com/gitea/tea",
        },
        "fj" => CliInfo {
            // NOTE: fj may require a tap; verify in Phase 3 when Forgejo is tested.
            // Using "fj" as brew name for now — flag for Phase 3 verification.
            brew_name: "fj".to_string(),
            url: "https://codeberg.org/forgejo/forgejo-cli",
        },
        other => CliInfo {
            brew_name: other.to_string(),
            url: "https://github.com/search?q=forge+cli",
        },
    }
}

#[cfg(test)]
mod forge_error_tests {
    use super::GfError;

    #[test]
    fn test_not_a_git_repo_display() {
        let msg = GfError::NotAGitRepo.to_string();
        assert_eq!(msg, "not a git repository (or any parent directory)");
    }

    #[test]
    fn test_no_remote_display() {
        let msg = GfError::NoRemote("origin".to_string()).to_string();
        assert_eq!(
            msg,
            "no remote named 'origin' \u{2014} use --remote to specify one"
        );
    }

    #[test]
    fn test_forge_not_detected_display() {
        let msg = GfError::ForgeNotDetected {
            domain: "git.internal.io".to_string(),
        }
        .to_string();
        assert!(
            msg.contains("Could not detect forge for: git.internal.io"),
            "missing header: {msg}"
        );
        assert!(
            msg.contains("Supported forges: github, gitlab, gitea, forgejo"),
            "missing forge list: {msg}"
        );
        assert!(
            msg.contains("domain = \"git.internal.io\""),
            "missing TOML snippet: {msg}"
        );
        assert!(
            msg.contains("type = \"forgejo\"  # or github, gitlab, gitea"),
            "missing type comment: {msg}"
        );
    }

    #[test]
    fn test_config_parse_error_display() {
        let msg = GfError::ConfigParseError("bad toml".to_string()).to_string();
        assert_eq!(msg, "failed to parse config: bad toml");
    }
}
