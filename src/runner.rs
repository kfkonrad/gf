use crate::error::{cli_info, GfError};

/// Run the given forge CLI with the provided arguments.
///
/// On Unix: replaces the current process via exec(). This function does not
/// return on success — the forge CLI becomes the process. TTY, signals, and
/// exit codes are inherited automatically by the shell.
///
/// On Windows: spawns a child process, waits for it, and propagates the exit
/// code. Handles signal-terminated processes (exit code None) with exit(1).
///
/// Returns Err(GfError::CliNotFound) if the CLI is not on PATH.
/// Returns Err(GfError::ExecFailed) if exec() fails after PATH check (unusual).
/// Returns Err(GfError::SpawnFailed) if spawn() fails (Windows only).
pub fn run(cli: &str, args: &[&str]) -> Result<(), GfError> {
    // CORE-06: Check PATH before attempting exec/spawn.
    // which() handles cross-platform PATH lookup, .exe extension on Windows,
    // symlinks, and permission bits. Do not hand-roll this.
    if which::which(cli).is_err() {
        let info = cli_info(cli);
        return Err(GfError::CliNotFound {
            cli: cli.to_string(),
            brew_name: info.brew_name,
            url: info.url.to_string(),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // exec() replaces the current process with cli. If it returns, it failed.
        // IMPORTANT: exec() does not run Rust destructors. Acquire no resources
        // between here and exec(). In practice, gf has nothing to drop at this point.
        let err = std::process::Command::new(cli).args(args).exec();
        // Only reached on failure (e.g., permission denied after which() succeeded).
        Err(GfError::ExecFailed(cli.to_string(), err))
    }

    #[cfg(windows)]
    {
        // On Windows, spawn the child and wait. Then propagate exit code.
        let status = std::process::Command::new(cli)
            .args(args)
            .status()
            .map_err(|e| GfError::SpawnFailed(cli.to_string(), e))?;

        match status.code() {
            Some(code) => std::process::exit(code),
            None => {
                // None means the process was terminated abnormally (signal on Unix,
                // unusual on Windows). Exit 1 as a safe fallback.
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::GfError;

    #[test]
    fn test_cli_not_found_display_format() {
        let err = GfError::CliNotFound {
            cli: "glab".to_string(),
            brew_name: "glab".to_string(),
            url: "https://gitlab.com/gitlab-org/cli".to_string(),
        };
        let msg = err.to_string();
        assert_eq!(
            msg,
            "glab not found\nInstall with: brew install glab\nOr see: https://gitlab.com/gitlab-org/cli"
        );
    }
}
