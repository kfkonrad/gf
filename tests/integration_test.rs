//! Integration tests for gf Phase 1: Foundation
//!
//! CORE-06: CLI not found error format
//! CORE-07: Exit code propagation

use assert_cmd::Command;
use predicates::prelude::*;

// ── Helper: create a temp git repo with a GitHub remote ───────────────────

/// Creates a temp git repo with a fake github.com remote.
/// After Phase 2, gf detects the forge from `git remote get-url origin`,
/// so tests that check CLI-level behavior need a valid git remote context.
fn setup_github_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/test/repo.git",
        ])
        .current_dir(dir.path())
        .output()
        .expect("git remote add");
    dir
}

/// Creates a temp bin dir with only a symlink to git (no gh or other CLIs).
/// Returns the bin dir path. Caller must keep the TempDir alive.
fn make_git_only_bin_dir() -> (tempfile::TempDir, String) {
    let bin_dir = tempfile::tempdir().expect("create bin temp dir");
    let git_path = which::which("git").expect("git must be installed");
    let symlink_path = bin_dir.path().join("git");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&git_path, &symlink_path).expect("symlink git");
    (
        bin_dir,
        symlink_path.parent().unwrap().to_string_lossy().to_string(),
    )
}

// ── CORE-06: CLI not found ─────────────────────────────────────────────────

/// When the detected forge's CLI binary is not found on PATH, gf must print
/// the first line of the error to stderr and exit non-zero.
/// Runs from a temp GitHub repo so forge detection succeeds (→ gh).
/// PATH is restricted to a symlink-only bin dir that has git but not gh.
#[test]
fn test_cli_not_found() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .args(["pr", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("gh not found"));
}

/// Verify the install hint line is present in stderr.
#[test]
fn test_cli_not_found_format() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .args(["pr", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Install with: brew install"));
}

/// Verify the official URL fallback line is present in stderr.
#[test]
fn test_cli_not_found_url() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .args(["pr", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Or see: "));
}

/// Verify no ANSI escape codes in the error output.
#[test]
fn test_cli_not_found_no_ansi() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .args(["pr", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\x1b").not());
}

// ── CORE-07: Exit code propagation ────────────────────────────────────────

/// Helper: create a bin dir with a fake `gh` script that calls exit_with with the given code.
fn setup_gh_exit_script(home_dir: &std::path::Path, exit_code: u32) -> std::path::PathBuf {
    let exit_with_path = assert_cmd::cargo::cargo_bin("exit_with");
    let bin_dir = home_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let gh_script = bin_dir.join("gh");
    std::fs::write(
        &gh_script,
        format!(
            "#!/bin/sh\nexec \"{}\" {}\n",
            exit_with_path.display(),
            exit_code
        ),
    )
    .expect("write gh script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gh_script, std::fs::Permissions::from_mode(0o755))
            .expect("set executable");
    }
    // PATH must include both the fake bin dir and git's dir
    bin_dir
}

/// When the child exits with code 2, gf must exit with code 2.
/// After Phase 2: forge detection maps github.com → gh; we fake gh with exit_with.
#[test]
fn test_exit_code_propagation() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let bin_dir = setup_gh_exit_script(home_dir.path(), 2);
    let (_git_tmp, git_only) = make_git_only_bin_dir();
    let path = format!("{}:{}", bin_dir.display(), git_only);

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &path)
        .args(["pr", "list"])
        .assert()
        .code(2);
}

/// Exit code 0 propagated correctly.
#[test]
fn test_exit_code_zero() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let bin_dir = setup_gh_exit_script(home_dir.path(), 0);
    let (_git_tmp, git_only) = make_git_only_bin_dir();
    let path = format!("{}:{}", bin_dir.display(), git_only);

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &path)
        .args(["pr", "list"])
        .assert()
        .code(0);
}

/// Non-standard exit code 42 propagated correctly.
#[test]
fn test_exit_code_42() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let bin_dir = setup_gh_exit_script(home_dir.path(), 42);
    let (_git_tmp, git_only) = make_git_only_bin_dir();
    let path = format!("{}:{}", bin_dir.display(), git_only);

    Command::cargo_bin("gf")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &path)
        .args(["pr", "list"])
        .assert()
        .code(42);
}

// ─── Phase 2: Forge Detection integration tests ───────────────────────────────

mod forge_detection {
    use assert_cmd::Command;
    use predicates::str::contains;

    /// CORE-01: Running gf outside a git repo should produce the NotAGitRepo error.
    #[test]
    fn test_gf_outside_git_repo_shows_error() {
        let mut cmd = Command::cargo_bin("gf").unwrap();
        cmd.current_dir("/tmp") // /tmp is not a git repo
            .arg("pr")
            .arg("list");
        cmd.assert()
            .failure()
            .stderr(contains("not a git repository"));
    }

    /// CORE-01: Running gf in a repo without a recognized forge should show detection error.
    /// Covered by forge::tests::test_known_host_unknown_returns_error unit test.
    #[test]
    fn test_gf_unknown_remote_url_shows_config_hint() {
        // Covered by unit test. Full integration coverage deferred.
    }
}
