// src/adapter/issue.rs — Issue subcommand and flag translation
use crate::error::GfError;
use crate::forge::ForgeType;
use clap::ArgMatches;

/// Translate `gf issue ...` ArgMatches into forge-specific args.
/// Called by adapter::translate() when the matched subcommand is "issue".
pub fn translate_issue(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let issue_cmd = issue_subcommand_name(forge);

    match matches.subcommand() {
        Some(("list", sub)) => translate_issue_list(forge, issue_cmd, sub),
        Some(("view", sub)) => translate_issue_view(forge, issue_cmd, sub),
        Some(("create", sub)) => translate_issue_create(forge, issue_cmd, sub),
        Some(("close", sub)) => translate_issue_close(forge, issue_cmd, sub),
        Some(("reopen", sub)) => translate_issue_reopen(forge, issue_cmd, sub),
        Some((verb, sub)) => {
            // Unknown verb: pass through as-is with any extra args
            let mut args = vec![issue_cmd.to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            Ok(args)
        }
        None => Ok(vec![issue_cmd.to_string()]),
    }
}

/// Maps the canonical "issue" command to the forge-specific equivalent.
/// Gitea uses "issues" (plural), all others use "issue" (singular).
fn issue_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Gitea => "issues",
        _ => "issue",
    }
}

/// Translate `gf issue list` with filter flags.
/// - Forgejo uses "search" verb instead of "list"
/// - GitLab uses boolean flags (--closed/--all) instead of --state <value>
/// - Forgejo remaps --author to --creator, --label to --labels
/// - Gitea remaps --label to --labels
fn translate_issue_list(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string()];

    // Verb: fj uses "search" instead of "list"
    match forge {
        ForgeType::Forgejo => args.push("search".to_string()),
        _ => args.push("list".to_string()),
    }

    // --state: glab uses boolean flags, others use --state <value>
    if let Some(state) = matches.get_one::<String>("state") {
        match forge {
            ForgeType::Gitlab => match state.as_str() {
                "closed" => args.push("--closed".to_string()),
                "all" => args.push("--all".to_string()),
                "open" => {} // glab default, no flag needed
                _ => {
                    args.push("--state".to_string());
                    args.push(state.clone());
                }
            },
            _ => {
                args.push("--state".to_string());
                args.push(state.clone());
            }
        }
    }

    // --author: fj remaps to --creator
    if let Some(author) = matches.get_one::<String>("author") {
        match forge {
            ForgeType::Forgejo => {
                args.push("--creator".to_string());
                args.push(author.clone());
            }
            _ => {
                args.push("--author".to_string());
                args.push(author.clone());
            }
        }
    }

    // --label: tea and fj remap to --labels
    if let Some(label) = matches.get_one::<String>("label") {
        match forge {
            ForgeType::Gitea | ForgeType::Forgejo => {
                args.push("--labels".to_string());
                args.push(label.clone());
            }
            _ => {
                args.push("--label".to_string());
                args.push(label.clone());
            }
        }
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf issue view <number>`.
/// - Tea does not have "issues view" — use "issues <N>" directly
/// - All others use standard "issue view <N>" pattern
fn translate_issue_view(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string()];

    // tea does not have "issues view" — use "issues <N>" directly
    if !matches!(forge, ForgeType::Gitea) {
        args.push("view".to_string());
    }

    // Number is required
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf issue create [--title <title>] [--body <body>]`.
/// - GitLab and Gitea map --body to --description
/// - GitHub and Forgejo use --body natively
fn translate_issue_create(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string(), "create".to_string()];

    // --title: canonical flag name matches all forges
    if let Some(title) = matches.get_one::<String>("title") {
        args.push("--title".to_string());
        args.push(title.clone());
    }

    // --body: translate to --description for glab and tea
    if let Some(body) = matches.get_one::<String>("body") {
        let body_flag = match forge {
            ForgeType::Gitlab | ForgeType::Gitea => "--description",
            ForgeType::Github | ForgeType::Forgejo => "--body",
        };
        args.push(body_flag.to_string());
        args.push(body.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf issue close <number>`.
/// All forges support this with standard pattern: [issue_cmd] close <number>
fn translate_issue_close(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let mut args = vec![issue_cmd.to_string(), "close".to_string()];

    // Number is required
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    let _ = forge; // All forges use same pattern for close
    Ok(args)
}

/// Translate `gf issue reopen <number>`.
/// - Forgejo does NOT support reopen — returns UnsupportedFeature error
/// - GitHub, GitLab, and Gitea all support standard reopen pattern
fn translate_issue_reopen(
    forge: ForgeType,
    issue_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    // Forgejo CLI has no issue reopen command
    if matches!(forge, ForgeType::Forgejo) {
        return Err(GfError::UnsupportedFeature {
            feature: "issue reopen".to_string(),
            forge: "Forgejo".to_string(),
            forge_cli: "fj".to_string(),
        });
    }

    let mut args = vec![issue_cmd.to_string(), "reopen".to_string()];

    // Number is required
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}
