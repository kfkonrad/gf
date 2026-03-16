mod adapter;
mod browse;
mod cmd;
mod error;
mod forge;
mod runner;

use clap_complete::{generate, Shell};

fn main() {
    let mut cli_cmd = cmd::build_cli();
    let matches = cli_cmd.clone().get_matches();

    // Handle the hidden `gf completions --shell <shell>` subcommand (CORE-12).
    // Must be handled before forge detection — completions don't need a git repo.
    if let Some(("completions", sub)) = matches.subcommand() {
        let shell = sub.get_one::<Shell>("shell").copied().unwrap_or(Shell::Bash);
        generate(shell, &mut cli_cmd, "gf", &mut std::io::stdout());
        return;
    }

    // Handle `gf browse` / `gf b` natively (BROWSE-05).
    // Browse intercepts early — it does its own git + forge detection internally.
    if let Some(("browse", sub)) = matches.subcommand() {
        if let Err(e) = browse::run(sub) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

    // Extract --remote (global flag, defaults to "origin")
    let remote = matches
        .get_one::<String>("remote")
        .map(|s| s.as_str())
        .unwrap_or("origin");

    // Detect the forge from the git remote URL (Phase 2)
    let forge_type = match forge::detect(remote) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    // Translate canonical gf args → forge-specific args (Phase 3)
    let translated: Vec<String> = adapter::translate(forge_type, &matches);

    // runner::run takes &[&str] — convert Vec<String> to temporary &[&str]
    let args_refs: Vec<&str> = translated.iter().map(|s| s.as_str()).collect();

    // Exec the forge CLI (replaces current process on Unix)
    if let Err(e) = runner::run(forge_type.cli_name(), &args_refs) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
