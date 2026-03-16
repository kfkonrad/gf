//! Helper binary: exit_with <code>
//! Used by integration tests to simulate a child process with a known exit code.
//! Usage: exit_with 2
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code: i32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    std::process::exit(code);
}
