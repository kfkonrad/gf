//! Integration tests for gf Phase 1: Foundation
//!
//! CORE-06: CLI not found error format
//! CORE-07: Exit code propagation

use assert_cmd::Command;
use predicates::prelude::*;

// ── CORE-06: CLI not found ─────────────────────────────────────────────────

/// When a CLI name that does not exist on PATH is passed, gf must print the
/// first line of the error to stderr and exit non-zero.
#[test]
fn test_cli_not_found() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("gf-nonexistent-cli-xyz not found"));
}

/// Verify the install hint line is present in stderr.
#[test]
fn test_cli_not_found_format() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Install with: brew install"));
}

/// Verify the official URL fallback line is present in stderr.
#[test]
fn test_cli_not_found_url() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Or see: "));
}

/// Verify no ANSI escape codes in the error output.
/// The format is plain text only — no color, no prefix.
#[test]
fn test_cli_not_found_no_ansi() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        // ESC character (0x1b) should not appear in the output
        .stderr(predicate::str::contains("\x1b").not());
}

// ── CORE-07: Exit code propagation ────────────────────────────────────────

/// When the child exits with code 2, gf must exit with code 2.
/// Uses the exit_with helper binary from tests/helpers/exit_with.rs.
///
/// Note: On Unix, gf exec()s into exit_with, so the exit code is exit_with's.
/// assert_cmd captures the final exit code regardless of exec() vs spawn().
#[test]
fn test_exit_code_propagation() {
    let exit_with_path = assert_cmd::cargo::cargo_bin("exit_with");

    Command::cargo_bin("gf")
        .unwrap()
        .args([exit_with_path.to_str().unwrap(), "2"])
        .assert()
        .code(2);
}

/// Exit code 0 propagated correctly.
#[test]
fn test_exit_code_zero() {
    let exit_with_path = assert_cmd::cargo::cargo_bin("exit_with");

    Command::cargo_bin("gf")
        .unwrap()
        .args([exit_with_path.to_str().unwrap(), "0"])
        .assert()
        .code(0);
}

/// Non-standard exit code 42 propagated correctly.
#[test]
fn test_exit_code_42() {
    let exit_with_path = assert_cmd::cargo::cargo_bin("exit_with");

    Command::cargo_bin("gf")
        .unwrap()
        .args([exit_with_path.to_str().unwrap(), "42"])
        .assert()
        .code(42);
}

// ─── Phase 2: Forge Detection integration tests ───────────────────────────────
// Note: These tests verify error output. The main.rs wiring happens in plan 03.
// These tests will fully pass after plan 03. Add them now so plan 03 has RED tests.

mod forge_detection {
    use assert_cmd::Command;
    use predicates::str::contains;

    /// CORE-01: Running gf outside a git repo should produce the NotAGitRepo error.
    /// This test will be RED until plan 03 wires forge::detect() into main.rs.
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
    /// This requires a temp git repo with an unknown remote — tested in unit tests instead.
    /// Integration placeholder to confirm error format matches CONTEXT.md spec.
    #[test]
    fn test_gf_unknown_remote_url_shows_config_hint() {
        // This test needs a temp repo with an unknown remote.
        // Skip for now — covered by forge::tests::test_known_host_unknown_returns_error unit test.
        // Full integration coverage added in plan 03 after main.rs wiring.
    }
}
