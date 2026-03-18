// src/cmd/mod.rs
use clap::{Arg, ArgAction, Command};

/// Build the complete gf CLI command tree.
///
/// Alias strategy:
///   - `mr` is a visible_alias for `pr` at the top level (CORE-09)
///   - `r` is a visible_alias for `repo` (CORE-08)
///   - `a` is a visible_alias for `auth` (CORE-08)
///   - Each verb has a one-letter visible_alias: c=create, v=view, f=fork, l=login, s=status (CORE-10)
///
/// Multi-word aliases (e.g., "mr create") are NOT expressed as clap aliases —
/// clap does not support multi-word aliases. Since `mr` aliases `pr`, the routing
/// `gf mr create` → `pr create` works automatically (mr resolves to pr, then create
/// resolves as a subcommand). Help text for `pr create` notes `mr create` in its
/// about string.
pub fn build_cli() -> Command {
    Command::new("gf")
        .about("Unified git forge CLI — works across GitHub, GitLab, Gitea, and Forgejo")
        .arg(
            Arg::new("remote")
                .long("remote")
                .value_name("NAME")
                .default_value("origin")
                .global(true)
                .help("Git remote to use for forge detection"),
        )
        .subcommand(build_pr())
        .subcommand(build_repo())
        .subcommand(build_auth())
        .subcommand(build_completions())
        .subcommand(build_browse())
}

fn build_pr() -> Command {
    Command::new("pr")
        .about("Pull/merge request commands")
        .visible_alias("mr") // CORE-09: gf mr ... works identically to gf pr ...
        .subcommand_required(false)
        .subcommand(
            Command::new("create")
                .about("Create a pull/merge request (aliases: c, mr create, mr c)")
                .visible_alias("c") // CORE-10
                .arg(
                    Arg::new("title")
                        .long("title")
                        .short('t')
                        .value_name("TITLE")
                        .help("PR title"),
                )
                .arg(
                    Arg::new("body")
                        .long("body")
                        .short('b')
                        .value_name("BODY")
                        .help("PR body/description"),
                )
                .arg(
                    Arg::new("base")
                        .long("base")
                        .short('B')
                        .value_name("BRANCH")
                        .help("Base branch"),
                )
                .arg(
                    Arg::new("draft")
                        .long("draft")
                        .action(ArgAction::SetTrue)
                        .help("Mark as draft"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Additional flags passed through to the underlying CLI"),
                ),
        )
        .subcommand(
            Command::new("view")
                .about("View a pull/merge request (aliases: v, mr view, mr v)")
                .visible_alias("v") // CORE-10
                .arg(
                    Arg::new("number")
                        .value_name("NUMBER")
                        .required(false)
                        .help("PR number (optional if on a PR branch)"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true)
                        .help("Additional flags passed through to the underlying CLI"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List pull/merge requests (aliases: l, mr list)")
                .visible_alias("l")
                .arg(Arg::new("state").long("state").value_name("STATE").help("Filter by state (open, closed, merged, all)"))
                .arg(Arg::new("author").long("author").value_name("USER").help("Filter by author"))
                .arg(Arg::new("label").long("label").value_name("LABEL").help("Filter by label"))
                .arg(Arg::new("extra").num_args(0..).allow_hyphen_values(true).last(true).help("Additional flags passed through to the underlying CLI")),
        )
        .subcommand(
            Command::new("merge")
                .about("Merge a pull/merge request (aliases: m, mr merge)")
                .visible_alias("m")
                .arg(Arg::new("number").value_name("NUMBER").required(false).help("PR number"))
                .arg(Arg::new("squash").long("squash").action(ArgAction::SetTrue).help("Squash merge"))
                .arg(Arg::new("rebase").long("rebase").action(ArgAction::SetTrue).help("Rebase merge"))
                .arg(Arg::new("merge").long("merge").action(ArgAction::SetTrue).help("Merge commit (default strategy)"))
                .arg(Arg::new("delete-branch").long("delete-branch").action(ArgAction::SetTrue).help("Delete branch after merge"))
                .arg(Arg::new("no-delete-branch").long("no-delete-branch").action(ArgAction::SetTrue).help("Keep branch after merge"))
                .arg(Arg::new("extra").num_args(0..).allow_hyphen_values(true).last(true).help("Additional flags passed through to the underlying CLI")),
        )
        .subcommand(
            Command::new("checkout")
                .about("Checkout a pull/merge request branch (aliases: co, mr checkout)")
                .visible_alias("co")
                .arg(Arg::new("number").value_name("NUMBER").required(false).help("PR number"))
                .arg(Arg::new("extra").num_args(0..).allow_hyphen_values(true).last(true).help("Additional flags passed through to the underlying CLI")),
        )
        .subcommand(
            Command::new("review")
                .about("Review a pull/merge request (mr review)")
                .arg(Arg::new("number").value_name("NUMBER").required(false).help("PR number"))
                .arg(Arg::new("comment").long("comment").action(ArgAction::SetTrue).help("Add a review comment"))
                .arg(Arg::new("approve").long("approve").action(ArgAction::SetTrue).help("Approve the PR"))
                .arg(Arg::new("body").long("body").short('b').value_name("TEXT").help("Comment body text"))
                .arg(Arg::new("extra").num_args(0..).allow_hyphen_values(true).last(true).help("Additional flags passed through to the underlying CLI")),
        )
        .subcommand(
            Command::new("approve")
                .about("Approve a pull/merge request (shorthand for review --approve)")
                .arg(Arg::new("number").value_name("NUMBER").required(false).help("PR number"))
                .arg(Arg::new("extra").num_args(0..).allow_hyphen_values(true).last(true).help("Additional flags passed through to the underlying CLI")),
        )
}

fn build_repo() -> Command {
    Command::new("repo")
        .about("Repository commands")
        .visible_alias("r") // CORE-08
        .subcommand_required(false)
        .subcommand(
            Command::new("view")
                .about("View repository info (aliases: v)")
                .visible_alias("v") // CORE-10
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new repository (aliases: c)")
                .visible_alias("c") // CORE-10
                .arg(
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .value_name("NAME")
                        .help("Repository name"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .value_name("DESC")
                        .help("Repository description"),
                )
                .arg(
                    Arg::new("private")
                        .long("private")
                        .action(ArgAction::SetTrue)
                        .help("Make repository private"),
                )
                .arg(
                    Arg::new("public")
                        .long("public")
                        .action(ArgAction::SetTrue)
                        .help("Make repository public (default)"),
                )
                .arg(
                    Arg::new("homepage")
                        .long("homepage")
                        .value_name("URL")
                        .help("Repository homepage URL"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
        .subcommand(
            Command::new("fork")
                .about("Fork a repository (aliases: f)")
                .visible_alias("f") // CORE-10
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
}

fn build_auth() -> Command {
    Command::new("auth")
        .about("Authentication commands")
        .visible_alias("a") // CORE-08
        .subcommand_required(false)
        .subcommand(
            Command::new("login")
                .about("Authenticate with the detected forge (aliases: l)")
                .visible_alias("l") // CORE-10
                .arg(
                    Arg::new("hostname")
                        .long("hostname")
                        .value_name("HOST")
                        .help("Forge hostname (for self-hosted)"),
                )
                .arg(
                    Arg::new("token")
                        .long("token")
                        .value_name("TOKEN")
                        .help("Authentication token"),
                )
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
        .subcommand(
            Command::new("logout")
                .about("Remove credentials for the detected forge")
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("Check current authentication state (aliases: s)")
                .visible_alias("s") // CORE-10
                .arg(
                    Arg::new("extra")
                        .num_args(0..)
                        .allow_hyphen_values(true)
                        .last(true),
                ),
        )
}

fn build_browse() -> Command {
    Command::new("browse")
        .about("Open repo, branch, or file in browser (alias: b)")
        .visible_alias("b") // CORE-08
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .required(false)
                .help("File path to open (relative or absolute)"),
        )
        .arg(
            Arg::new("branch")
                .long("branch")
                .value_name("BRANCH")
                .help("Branch to use instead of the current branch"),
        )
        .arg(
            Arg::new("no-browser")
                .long("no-browser")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Print URL without opening browser (for scripting/CI)"),
        )
        .arg(
            Arg::new("pr")
                .long("pr")
                .visible_alias("mr")
                .value_name("NUMBER")
                .conflicts_with_all(["issue", "file", "branch"])
                .help("Open PR/MR in browser (--mr is an alias)"),
        )
        .arg(
            Arg::new("issue")
                .long("issue")
                .value_name("NUMBER")
                .conflicts_with_all(["pr", "file", "branch"])
                .help("Open issue in browser"),
        )
}

/// Hidden subcommand: `gf completions --shell <bash|zsh|fish|...>`
/// Generates shell completion scripts to stdout. CORE-12.
fn build_completions() -> Command {
    use clap_complete::Shell;
    Command::new("completions")
        .hide(true)
        .about("Generate shell completion scripts")
        .arg(
            Arg::new("shell")
                .value_name("SHELL")
                .value_parser(clap::value_parser!(Shell))
                .required(true)
                .help("Shell type (bash, zsh, fish, elvish, powershell)"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_validates() {
        // clap's debug_assert checks the Command tree for internal consistency
        build_cli().debug_assert();
    }

    #[test]
    fn test_pr_has_mr_alias() {
        let cli = build_cli();
        let pr_cmd = cli.find_subcommand("pr").expect("pr subcommand exists");
        let aliases: Vec<&str> = pr_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"mr"),
            "pr should have visible alias 'mr'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_repo_has_r_alias() {
        let cli = build_cli();
        let repo_cmd = cli.find_subcommand("repo").expect("repo subcommand exists");
        let aliases: Vec<&str> = repo_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"r"),
            "repo should have visible alias 'r'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_auth_has_a_alias() {
        let cli = build_cli();
        let auth_cmd = cli.find_subcommand("auth").expect("auth subcommand exists");
        let aliases: Vec<&str> = auth_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"a"),
            "auth should have visible alias 'a'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_pr_create_has_c_alias() {
        let cli = build_cli();
        let pr_cmd = cli.find_subcommand("pr").unwrap();
        let create_cmd = pr_cmd
            .find_subcommand("create")
            .expect("create subcommand exists");
        let aliases: Vec<&str> = create_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"c"),
            "pr create should have alias 'c'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_pr_view_has_v_alias() {
        let cli = build_cli();
        let pr_cmd = cli.find_subcommand("pr").unwrap();
        let view_cmd = pr_cmd
            .find_subcommand("view")
            .expect("view subcommand exists");
        let aliases: Vec<&str> = view_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"v"),
            "pr view should have alias 'v'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_auth_login_has_l_alias() {
        let cli = build_cli();
        let auth_cmd = cli.find_subcommand("auth").unwrap();
        let login_cmd = auth_cmd
            .find_subcommand("login")
            .expect("login subcommand exists");
        let aliases: Vec<&str> = login_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"l"),
            "auth login should have alias 'l'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_auth_status_has_s_alias() {
        let cli = build_cli();
        let auth_cmd = cli.find_subcommand("auth").unwrap();
        let status_cmd = auth_cmd
            .find_subcommand("status")
            .expect("status subcommand exists");
        let aliases: Vec<&str> = status_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"s"),
            "auth status should have alias 's'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_repo_fork_has_f_alias() {
        let cli = build_cli();
        let repo_cmd = cli.find_subcommand("repo").unwrap();
        let fork_cmd = repo_cmd
            .find_subcommand("fork")
            .expect("fork subcommand exists");
        let aliases: Vec<&str> = fork_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"f"),
            "repo fork should have alias 'f'; got: {aliases:?}"
        );
    }

    #[test]
    fn test_browse_has_b_alias() {
        let cli = build_cli();
        let browse_cmd = cli
            .find_subcommand("browse")
            .expect("browse subcommand exists");
        let aliases: Vec<&str> = browse_cmd.get_visible_aliases().collect();
        assert!(
            aliases.contains(&"b"),
            "browse should have visible alias 'b'; got: {aliases:?}"
        );
    }

    /// Verifies that `mr` routes to the same subcommand as `pr` at the top level.
    /// This confirms CORE-09: `gf mr create` will route to the pr create handler.
    #[test]
    fn test_mr_resolves_to_pr() {
        let mut cli = build_cli();
        // get_matches_from on ["gf", "mr", "create"] should not error
        let result = cli.try_get_matches_from(["gf", "mr", "create"]);
        assert!(
            result.is_ok(),
            "gf mr create should parse without error: {:?}",
            result.err()
        );
        let matches = result.unwrap();
        let (subcmd_name, _) = matches.subcommand().expect("subcommand matched");
        // clap returns the canonical name (pr), not the alias (mr)
        assert_eq!(
            subcmd_name, "pr",
            "mr should route to pr subcommand; got: {subcmd_name}"
        );
    }
}
