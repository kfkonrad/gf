use crate::error::GfError;

/// Run the given forge CLI with the provided arguments.
/// On Unix, replaces the current process via exec().
/// On Windows, spawns a child and waits for it.
///
/// Returns Err only if the CLI cannot be found or exec/spawn fails before the
/// child starts. Once exec() succeeds on Unix, this function does not return.
pub fn run(cli: &str, args: &[&str]) -> Result<(), GfError> {
    // Implementation in Plan 02
    let _ = (cli, args);
    Ok(())
}
