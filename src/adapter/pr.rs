// src/adapter/pr.rs — PR/MR subcommand and flag translation
use crate::error::GfError;
use crate::forge::ForgeType;
use clap::ArgMatches;

/// Translate `gf pr ...` ArgMatches into forge-specific args.
/// Called by adapter::translate() when the matched subcommand is "pr" (or "mr" alias).
pub fn translate_pr(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    // The PR subcommand name differs per forge (PR-03)
    let pr_cmd = pr_subcommand_name(forge);

    match matches.subcommand() {
        Some(("create", sub)) => translate_pr_create(forge, pr_cmd, sub),
        Some(("view", sub)) => translate_pr_view(forge, pr_cmd, sub),
        Some(("list", sub)) => translate_pr_list(forge, pr_cmd, sub),
        Some(("checkout", sub)) => translate_pr_checkout(forge, pr_cmd, sub),
        Some(("merge", sub)) => translate_pr_merge(forge, pr_cmd, sub),
        Some((verb, sub)) => {
            // Unknown verb: pass through as-is with any extra args
            let mut args = vec![pr_cmd.to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            Ok(args)
        }
        None => Ok(vec![pr_cmd.to_string()]),
    }
}

/// Maps the canonical "pr" command to the forge-specific equivalent (PR-03).
fn pr_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Github => "pr",
        ForgeType::Gitlab => "mr",
        ForgeType::Gitea => "pulls",
        ForgeType::Forgejo => "pr",
    }
}

/// Translate `gf pr create` with canonical flags (PR-01, PR-02, PR-04).
fn translate_pr_create(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![pr_cmd.to_string(), "create".to_string()];

    // --title: canonical flag name matches all forges
    if let Some(title) = matches.get_one::<String>("title") {
        args.push("--title".to_string());
        args.push(title.clone());
    }

    // --body: translate to --description for glab and tea (PR-02)
    if let Some(body) = matches.get_one::<String>("body") {
        let body_flag = match forge {
            ForgeType::Gitlab | ForgeType::Gitea => "--description",
            ForgeType::Github | ForgeType::Forgejo => "--body",
        };
        args.push(body_flag.to_string());
        args.push(body.clone());
    }

    // --base: translate to --target-branch for glab (PR-02); all others use --base natively
    if let Some(base) = matches.get_one::<String>("base") {
        let base_flag = match forge {
            ForgeType::Gitlab => "--target-branch",
            _ => "--base",
        };
        args.push(base_flag.to_string());
        args.push(base.clone());
    }

    // --draft: gh and glab support natively; tea and fj do not — silently omit
    if matches.get_flag("draft") {
        match forge {
            ForgeType::Gitea | ForgeType::Forgejo => {
                // tea and fj do not support --draft; silently omit
            }
            _ => {
                args.push("--draft".to_string());
            }
        }
    }

    // Passthrough: unrecognized flags appended verbatim (PR-04)
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf pr list` with filter flags (PR-01).
/// Forgejo uses `pr search` instead of `pr list`, `--creator` instead of `--author`, `--labels` instead of `--label`.
/// GitLab uses boolean flags (--closed/--merged/--all) instead of `--state <value>`.
/// Gitea (tea) does not support --author or --label → hard error.
fn translate_pr_list(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![pr_cmd.to_string()];

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
                "merged" => args.push("--merged".to_string()),
                "all" => args.push("--all".to_string()),
                "open" => {} // glab default, no flag needed
                _ => { args.push("--state".to_string()); args.push(state.clone()); }
            },
            _ => { args.push("--state".to_string()); args.push(state.clone()); }
        }
    }

    // --author: tea UNSUPPORTED, fj remaps to --creator
    if let Some(author) = matches.get_one::<String>("author") {
        match forge {
            ForgeType::Gitea => return Err(GfError::UnsupportedFeature {
                feature: "pr list --author".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
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

    // --label: tea UNSUPPORTED, fj remaps to --labels
    if let Some(label) = matches.get_one::<String>("label") {
        match forge {
            ForgeType::Gitea => return Err(GfError::UnsupportedFeature {
                feature: "pr list --label".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
            ForgeType::Forgejo => {
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

/// Translate `gf pr checkout [<number>]`.
fn translate_pr_checkout(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![pr_cmd.to_string(), "checkout".to_string()];

    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    let _ = forge; // forge-specific routing may be added later
    Ok(args)
}

/// Translate `gf pr merge [<number>] [--squash|--rebase|--merge] [--delete-branch|--no-delete-branch]` (PR-02).
///
/// Strategy mapping:
///   --squash: gh --squash, glab --squash, tea --style squash, fj --method squash
///   --rebase: gh --rebase, glab --rebase, tea --style rebase, fj --method rebase
///   --merge (or default): gh --merge, glab (no flag), tea --style merge, fj --method merge
///
/// Delete-branch mapping:
///   gh: --delete-branch, glab: --remove-source-branch, fj: --delete, tea: UNSUPPORTED
fn translate_pr_merge(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![pr_cmd.to_string(), "merge".to_string()];

    // Number (optional positional)
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    // Strategy flags
    let squash = matches.get_flag("squash");
    let rebase = matches.get_flag("rebase");

    if squash {
        match forge {
            ForgeType::Github | ForgeType::Gitlab => args.push("--squash".to_string()),
            ForgeType::Gitea => { args.push("--style".to_string()); args.push("squash".to_string()); }
            ForgeType::Forgejo => { args.push("--method".to_string()); args.push("squash".to_string()); }
        }
    } else if rebase {
        match forge {
            ForgeType::Github | ForgeType::Gitlab => args.push("--rebase".to_string()),
            ForgeType::Gitea => { args.push("--style".to_string()); args.push("rebase".to_string()); }
            ForgeType::Forgejo => { args.push("--method".to_string()); args.push("rebase".to_string()); }
        }
    } else {
        // Default or explicit --merge: explicitly pass merge strategy per forge
        // glab has no --merge flag; merge is its default (no flag needed)
        match forge {
            ForgeType::Github => args.push("--merge".to_string()),
            ForgeType::Gitlab => {} // glab default is merge, no flag needed
            ForgeType::Gitea => { args.push("--style".to_string()); args.push("merge".to_string()); }
            ForgeType::Forgejo => { args.push("--method".to_string()); args.push("merge".to_string()); }
        }
    }

    // Delete-branch: CLI flag > built-in default (false)
    let explicit_delete = matches.get_flag("delete-branch");
    let explicit_no_delete = matches.get_flag("no-delete-branch");

    let should_delete = explicit_delete && !explicit_no_delete;

    if should_delete {
        match forge {
            ForgeType::Github => args.push("--delete-branch".to_string()),
            ForgeType::Gitlab => args.push("--remove-source-branch".to_string()),
            ForgeType::Forgejo => args.push("--delete".to_string()),
            ForgeType::Gitea => return Err(GfError::UnsupportedFeature {
                feature: "pr merge --delete-branch".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
        }
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf pr view [<number>]` (PR-05, PR-06).
/// Delegates to the underlying CLI with or without number.
/// Current-branch PR lookup is handled natively by gh/glab/tea/fj.
fn translate_pr_view(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let mut args = vec![pr_cmd.to_string()];

    // tea does not have "pulls view" — use "pulls <N>" directly
    if !matches!(forge, ForgeType::Gitea) {
        args.push("view".to_string());
    }

    // Number is optional (PR-05): if not provided, the underlying CLI finds the current-branch PR
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    // Passthrough for any extra flags (PR-04)
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}
