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
