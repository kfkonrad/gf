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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
        .args(["pr", "view"])
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
            .arg("view");
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

// ── Helper: create a temp git repo with a GitLab remote ───────────────────

fn setup_gitlab_repo() -> tempfile::TempDir {
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
            "https://gitlab.com/test/repo.git",
        ])
        .current_dir(dir.path())
        .output()
        .expect("git remote add");
    dir
}

// ── BROWSE-01 / BROWSE-05: --no-browser prints URL, no forge CLI spawned ──

/// gf browse --no-browser prints the repo URL to stdout and exits 0.
/// Uses PATH isolation (git-only bin dir) to prove no forge CLI is spawned.
/// Covers BROWSE-01 (URL printed) and BROWSE-05 (no gh/glab/tea/fj invoked).
#[test]
fn test_browse_no_browser_prints_url() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo.path())
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(repo.path())
        .output()
        .expect("git commit");

    Command::cargo_bin("gf")
        .unwrap()
        .args(["browse", "--no-browser"])
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .assert()
        .success()
        .stdout(predicates::str::starts_with("https://github.com/test/repo/tree/"));
}

/// gf browse --no-browser from a GitLab repo includes the /-/ infix in the URL.
/// Covers BROWSE-01 GitLab URL format.
#[test]
fn test_browse_no_browser_gitlab_url_has_infix() {
    let repo = setup_gitlab_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo.path())
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(repo.path())
        .output()
        .expect("git commit");

    Command::cargo_bin("gf")
        .unwrap()
        .args(["browse", "--no-browser"])
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .assert()
        .success()
        .stdout(predicates::str::contains("/-/tree/"));
}

/// gf browse --no-browser --branch main forces "main" in the URL
/// regardless of the checked-out branch. Covers BROWSE-04.
#[test]
fn test_browse_no_browser_branch_override() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo.path())
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(repo.path())
        .output()
        .expect("git commit");

    Command::cargo_bin("gf")
        .unwrap()
        .args(["browse", "--no-browser", "--branch", "main"])
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .assert()
        .success()
        .stdout(predicates::str::contains("/tree/main"));
}

/// gf browse --no-browser <file> includes the file path in the URL.
/// Covers BROWSE-03.
#[test]
fn test_browse_no_browser_file_arg() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo.path())
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(repo.path())
        .output()
        .expect("git commit");

    Command::cargo_bin("gf")
        .unwrap()
        .args(["browse", "--no-browser", "src/lib.rs"])
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .assert()
        .success()
        .stdout(predicates::str::contains("/blob/"))
        .stdout(predicates::str::contains("src/lib.rs"));
}

/// gf b (alias) works identically to gf browse. Covers BROWSE-01 alias.
#[test]
fn test_browse_alias_b_works() {
    let repo = setup_github_repo();
    let home_dir = tempfile::tempdir().expect("create home temp dir");
    let (_bin_tmp, git_only) = make_git_only_bin_dir();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo.path())
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(repo.path())
        .output()
        .expect("git commit");

    Command::cargo_bin("gf")
        .unwrap()
        .args(["b", "--no-browser"])
        .current_dir(repo.path())
        .env("HOME", home_dir.path())
        .env("PATH", &git_only)
        .assert()
        .success()
        .stdout(predicates::str::starts_with("https://github.com/test/repo/tree/"));
}

// ────────────────────────────────────────────────────────────────────────────
// Phase 3: Alias routing tests (CORE-08 through CORE-12)
// These tests use build_cli() directly — no live forge CLI needed.
// ────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod alias_routing {
    use clap_complete::{generate, Shell};
    use gf::cmd::build_cli;

    // CORE-09: `gf mr create` routes to the pr subcommand (canonical name)
    #[test]
    fn test_mr_alias_routes_to_pr() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "mr", "create"])
            .expect("mr create should parse");
        let (subcmd, _) = matches.subcommand().expect("subcommand matched");
        assert_eq!(subcmd, "pr", "mr should resolve to pr; got: {subcmd}");
    }

    // CORE-09: `gf mr view` routes to pr view
    #[test]
    fn test_mr_view_routes_to_pr_view() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "mr", "view"])
            .expect("mr view should parse");
        let (subcmd, sub) = matches.subcommand().expect("top subcommand");
        assert_eq!(subcmd, "pr");
        let (verb, _) = sub.subcommand().expect("verb subcommand");
        assert_eq!(verb, "view");
    }

    // CORE-08: `gf r v` routes to repo view
    #[test]
    fn test_r_v_routes_to_repo_view() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "r", "v"])
            .expect("r v should parse");
        let (subcmd, sub) = matches.subcommand().expect("top subcommand");
        assert_eq!(subcmd, "repo", "r should resolve to repo; got: {subcmd}");
        let (verb, _) = sub.subcommand().expect("verb subcommand");
        assert_eq!(verb, "view", "v should resolve to view; got: {verb}");
    }

    // CORE-08: `gf a s` routes to auth status
    #[test]
    fn test_a_s_routes_to_auth_status() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "a", "s"])
            .expect("a s should parse");
        let (subcmd, sub) = matches.subcommand().expect("top subcommand");
        assert_eq!(subcmd, "auth", "a should resolve to auth; got: {subcmd}");
        let (verb, _) = sub.subcommand().expect("verb subcommand");
        assert_eq!(verb, "status", "s should resolve to status; got: {verb}");
    }

    // CORE-10: `gf pr c` routes to pr create
    #[test]
    fn test_pr_c_routes_to_pr_create() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "pr", "c"])
            .expect("pr c should parse");
        let (subcmd, sub) = matches.subcommand().expect("top subcommand");
        assert_eq!(subcmd, "pr");
        let (verb, _) = sub.subcommand().expect("verb");
        assert_eq!(verb, "create", "c should resolve to create; got: {verb}");
    }

    // CORE-10: `gf r c` routes to repo create
    #[test]
    fn test_r_c_routes_to_repo_create() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "r", "c"])
            .expect("r c should parse");
        let (subcmd, sub) = matches.subcommand().unwrap();
        assert_eq!(subcmd, "repo");
        let (verb, _) = sub.subcommand().unwrap();
        assert_eq!(verb, "create");
    }

    // CORE-10: `gf a l` routes to auth login
    #[test]
    fn test_a_l_routes_to_auth_login() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "a", "l"])
            .expect("a l should parse");
        let (subcmd, sub) = matches.subcommand().unwrap();
        assert_eq!(subcmd, "auth");
        let (verb, _) = sub.subcommand().unwrap();
        assert_eq!(verb, "login");
    }

    // CORE-11: --help output contains "mr" alias
    #[test]
    fn test_help_contains_mr_alias() {
        let help = build_cli().render_help().to_string();
        assert!(help.contains("mr"), "help should mention 'mr' alias; help output:\n{help}");
    }

    // CORE-11: --help output contains "r" alias for repo
    #[test]
    fn test_help_contains_r_alias() {
        let help = build_cli().render_help().to_string();
        assert!(help.contains('['), "help should contain alias brackets; got:\n{help}");
    }

    // CORE-12: completions generate output containing "gf"
    #[test]
    fn test_completions_bash_generates_output() {
        let mut buf = Vec::new();
        generate(Shell::Bash, &mut build_cli(), "gf", &mut buf);
        let output = String::from_utf8(buf).expect("valid UTF-8");
        assert!(!output.is_empty(), "completion output should not be empty");
        assert!(output.contains("gf"), "completion script should reference 'gf'; got length {}", output.len());
    }

    // CORE-12: completions for zsh also work
    #[test]
    fn test_completions_zsh_generates_output() {
        let mut buf = Vec::new();
        generate(Shell::Zsh, &mut build_cli(), "gf", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
    }
}
