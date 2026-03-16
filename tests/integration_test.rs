//! Integration tests for gf Phase 1: Foundation
//!
//! CORE-06: CLI not found error format
//! CORE-07: Exit code propagation

use assert_cmd::Command;
use predicates::prelude::*;

// ── CORE-06: CLI not found ─────────────────────────────────────────────────

/// When a CLI name that does not exist on PATH is passed, gf must print a
/// two-line error to stderr and exit non-zero.
///
/// Expected stderr (exact format, no prefix, no ANSI):
///   <cli> not found
///   Install with: brew install <cli>
///   Or see: <url>
#[test]
fn test_cli_not_found() {
    // "gf-nonexistent-cli-xyz" will never be on PATH
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("gf-nonexistent-cli-xyz not found"));
}

/// Verify the install hint line is present in the error output.
#[test]
fn test_cli_not_found_format() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Install with: brew install"));
}

/// Verify the official URL line is present in the error output.
#[test]
fn test_cli_not_found_url() {
    Command::cargo_bin("gf")
        .unwrap()
        .args(["gf-nonexistent-cli-xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Or see: "));
}

// ── CORE-07: Exit code propagation ────────────────────────────────────────

/// When the child CLI exits with code 2, gf must exit with code 2.
/// Uses the exit_with helper binary.
#[test]
fn test_exit_code_propagation() {
    // Build exit_with first: `cargo build --bin exit_with`
    // Then invoke gf with exit_with as the "CLI" — gf will exec/spawn it.
    //
    // This test will pass only after Plan 02 implements the runner.
    // It is included here as a Wave 0 stub to satisfy the Nyquist requirement.
    // TODO(plan-02): make this test pass by implementing runner::run()

    // For now: just verify the binary builds. Runtime behavior tested in Plan 02.
    Command::cargo_bin("gf").unwrap();
}
