mod error;
mod forge;
mod runner;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Parse --remote <name> flag before remaining args.
    // Simple hand-rolled parse — clap arrives in Phase 3 and will supersede this.
    // If multiple --remote flags appear, last one wins.
    let mut remote = "origin".to_string();
    let mut remaining: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--remote" {
            if i + 1 < args.len() {
                remote = args[i + 1].clone();
                i += 2; // skip both --remote and the name
            } else {
                eprintln!("--remote requires a value");
                std::process::exit(1);
            }
        } else {
            remaining.push(args[i].as_str());
            i += 1;
        }
    }

    // Phase 2: Detect the forge from the git remote URL.
    let forge_type = match forge::detect(&remote) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let cli = forge_type.cli_name();

    if let Err(e) = runner::run(cli, &remaining) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
