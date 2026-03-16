mod error;
mod runner;

fn main() {
    // Placeholder: Phase 2 adds forge detection.
    // For now, parse the CLI name as the first arg for testing.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if arg_refs.is_empty() {
        eprintln!("usage: gf <forge-cli> [args...]");
        std::process::exit(1);
    }

    let cli = arg_refs[0];
    let rest = &arg_refs[1..];

    if let Err(e) = runner::run(cli, rest) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
