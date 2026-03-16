#![allow(dead_code)]

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
    SpawnFailed(String, std::io::Error),
}

/// Known forge CLI info for install hints.
pub struct CliInfo {
    pub brew_name: String,
    pub url: &'static str,
}

/// Return install hint info for a known CLI, or a sensible default.
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
