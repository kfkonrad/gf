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
