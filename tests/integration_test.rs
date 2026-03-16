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
