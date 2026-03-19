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
        Some(("review", sub)) => translate_pr_review(forge, pr_cmd, sub),
        Some(("approve", sub)) => translate_pr_approve(forge, pr_cmd, sub),
        Some(("edit", sub)) => translate_pr_edit(forge, pr_cmd, sub),
        Some(("checks", sub)) => translate_pr_checks(forge, pr_cmd, sub),
        Some(("comment", sub)) => translate_pr_comment(forge, pr_cmd, sub),
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
fn translate_pr_create(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
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
fn translate_pr_list(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
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

    // --author: tea UNSUPPORTED, fj remaps to --creator
    if let Some(author) = matches.get_one::<String>("author") {
        match forge {
            ForgeType::Gitea => {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr list --author".to_string(),
                    forge: "Gitea".to_string(),
                    forge_cli: "tea".to_string(),
                })
            }
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
            ForgeType::Gitea => {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr list --label".to_string(),
                    forge: "Gitea".to_string(),
                    forge_cli: "tea".to_string(),
                })
            }
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
fn translate_pr_checkout(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
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
fn translate_pr_merge(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
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
            ForgeType::Gitea => {
                args.push("--style".to_string());
                args.push("squash".to_string());
            }
            ForgeType::Forgejo => {
                args.push("--method".to_string());
                args.push("squash".to_string());
            }
        }
    } else if rebase {
        match forge {
            ForgeType::Github | ForgeType::Gitlab => args.push("--rebase".to_string()),
            ForgeType::Gitea => {
                args.push("--style".to_string());
                args.push("rebase".to_string());
            }
            ForgeType::Forgejo => {
                args.push("--method".to_string());
                args.push("rebase".to_string());
            }
        }
    } else {
        // Default or explicit --merge: explicitly pass merge strategy per forge
        // glab has no --merge flag; merge is its default (no flag needed)
        match forge {
            ForgeType::Github => args.push("--merge".to_string()),
            ForgeType::Gitlab => {} // glab default is merge, no flag needed
            ForgeType::Gitea => {
                args.push("--style".to_string());
                args.push("merge".to_string());
            }
            ForgeType::Forgejo => {
                args.push("--method".to_string());
                args.push("merge".to_string());
            }
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
            ForgeType::Gitea => {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr merge --delete-branch".to_string(),
                    forge: "Gitea".to_string(),
                    forge_cli: "tea".to_string(),
                })
            }
        }
    }

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

/// Translate `gf pr review [<number>] [--comment --body <text>] [--approve]` (PR-04, PR-05).
///
/// Comment mapping:
///   gh: pr review <N> --comment --body <text>
///   glab: mr comment <N> --message <text>  (subcommand remap: review → comment, --body → --message)
///   fj: pr comment <N> <text>  (subcommand remap, body is positional)
///   tea: UNSUPPORTED
///
/// Approve mapping:
///   gh: pr review <N> --approve
///   glab: mr approve <N>  (subcommand remap: review → approve, flag removed)
///   tea: UNSUPPORTED
///   fj: UNSUPPORTED
fn translate_pr_review(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    let is_approve = matches.get_flag("approve");
    let is_comment = matches.get_flag("comment");
    let body = matches.get_one::<String>("body");

    if is_approve {
        match forge {
            ForgeType::Github => {
                let mut args = vec![pr_cmd.to_string(), "review".to_string()];
                if let Some(n) = number {
                    args.push(n.clone());
                }
                args.push("--approve".to_string());
                Ok(args)
            }
            ForgeType::Gitlab => {
                // glab mr approve <N> — subcommand remap
                let mut args = vec![pr_cmd.to_string(), "approve".to_string()];
                if let Some(n) = number {
                    args.push(n.clone());
                }
                Ok(args)
            }
            ForgeType::Gitea => Err(GfError::UnsupportedFeature {
                feature: "pr review --approve".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
            ForgeType::Forgejo => Err(GfError::UnsupportedFeature {
                feature: "pr review --approve".to_string(),
                forge: "Forgejo".to_string(),
                forge_cli: "fj".to_string(),
            }),
        }
    } else if is_comment {
        match forge {
            ForgeType::Github => {
                let mut args = vec![pr_cmd.to_string(), "review".to_string()];
                if let Some(n) = number {
                    args.push(n.clone());
                }
                args.push("--comment".to_string());
                if let Some(b) = body {
                    args.push("--body".to_string());
                    args.push(b.clone());
                }
                Ok(args)
            }
            ForgeType::Gitlab => {
                // glab mr comment <N> --message <text>
                let mut args = vec![pr_cmd.to_string(), "comment".to_string()];
                if let Some(n) = number {
                    args.push(n.clone());
                }
                if let Some(b) = body {
                    args.push("--message".to_string());
                    args.push(b.clone());
                }
                Ok(args)
            }
            ForgeType::Forgejo => {
                // fj pr comment <N> <body> — body is positional
                let mut args = vec![pr_cmd.to_string(), "comment".to_string()];
                if let Some(n) = number {
                    args.push(n.clone());
                }
                if let Some(b) = body {
                    args.push(b.clone());
                }
                Ok(args)
            }
            ForgeType::Gitea => Err(GfError::UnsupportedFeature {
                feature: "pr review --comment".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            }),
        }
    } else {
        // No --approve or --comment — pass through as generic review
        let mut args = vec![pr_cmd.to_string(), "review".to_string()];
        if let Some(n) = number {
            args.push(n.clone());
        }
        if let Some(extra) = matches.get_many::<String>("extra") {
            args.extend(extra.cloned());
        }
        Ok(args)
    }
}

/// Translate `gf pr approve [<number>]` — syntactic sugar for `gf pr review --approve`.
/// Produces the same output as translate_pr_review with --approve flag.
fn translate_pr_approve(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");

    match forge {
        ForgeType::Github => {
            let mut args = vec![pr_cmd.to_string(), "review".to_string()];
            if let Some(n) = number {
                args.push(n.clone());
            }
            args.push("--approve".to_string());
            Ok(args)
        }
        ForgeType::Gitlab => {
            let mut args = vec![pr_cmd.to_string(), "approve".to_string()];
            if let Some(n) = number {
                args.push(n.clone());
            }
            Ok(args)
        }
        ForgeType::Gitea => Err(GfError::UnsupportedFeature {
            feature: "pr approve".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        }),
        ForgeType::Forgejo => Err(GfError::UnsupportedFeature {
            feature: "pr approve".to_string(),
            forge: "Forgejo".to_string(),
            forge_cli: "fj".to_string(),
        }),
    }
}

/// Translate `gf pr view [<number>]` (PR-05, PR-06).
/// Delegates to the underlying CLI with or without number.
/// Current-branch PR lookup is handled natively by gh/glab/tea/fj.
fn translate_pr_view(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
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

/// Translate `gf pr edit [<number>] [--add-label X] [--remove-label X] [--add-reviewer X] [--remove-reviewer X] [--add-assignee X] [--remove-assignee X]` (PR-09).
///
/// Forge mapping:
///   GitHub: gh pr edit <N> --add-label/--remove-label/--add-reviewer/--remove-reviewer/--add-assignee/--remove-assignee (direct)
///   GitLab: glab mr update <N> --label/--unlabel/--reviewer +X/-X/--assignee +X/-X
///   Forgejo: fj pr edit <N> labels --add/--rm (labels only; reviewer/assignee unsupported)
///   Gitea: tea has no pulls edit → entire command unsupported
fn translate_pr_edit(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    let add_label = matches.get_one::<String>("add-label");
    let remove_label = matches.get_one::<String>("remove-label");
    let add_reviewer = matches.get_one::<String>("add-reviewer");
    let remove_reviewer = matches.get_one::<String>("remove-reviewer");
    let add_assignee = matches.get_one::<String>("add-assignee");
    let remove_assignee = matches.get_one::<String>("remove-assignee");

    // ── VALIDATE: check all flags against forge capabilities BEFORE building ──
    match forge {
        ForgeType::Gitea => {
            return Err(GfError::UnsupportedFeature {
                feature: "pr edit".to_string(),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            });
        }
        ForgeType::Forgejo => {
            if add_reviewer.is_some() {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr edit --add-reviewer".to_string(),
                    forge: "Forgejo".to_string(),
                    forge_cli: "fj".to_string(),
                });
            }
            if remove_reviewer.is_some() {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr edit --remove-reviewer".to_string(),
                    forge: "Forgejo".to_string(),
                    forge_cli: "fj".to_string(),
                });
            }
            if add_assignee.is_some() {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr edit --add-assignee".to_string(),
                    forge: "Forgejo".to_string(),
                    forge_cli: "fj".to_string(),
                });
            }
            if remove_assignee.is_some() {
                return Err(GfError::UnsupportedFeature {
                    feature: "pr edit --remove-assignee".to_string(),
                    forge: "Forgejo".to_string(),
                    forge_cli: "fj".to_string(),
                });
            }
        }
        _ => {} // Github and Gitlab support all flags
    }

    // ── BUILD: construct forge-specific args ──
    match forge {
        ForgeType::Github => {
            let mut args = vec![pr_cmd.to_string(), "edit".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(v) = add_label { args.push("--add-label".to_string()); args.push(v.clone()); }
            if let Some(v) = remove_label { args.push("--remove-label".to_string()); args.push(v.clone()); }
            if let Some(v) = add_reviewer { args.push("--add-reviewer".to_string()); args.push(v.clone()); }
            if let Some(v) = remove_reviewer { args.push("--remove-reviewer".to_string()); args.push(v.clone()); }
            if let Some(v) = add_assignee { args.push("--add-assignee".to_string()); args.push(v.clone()); }
            if let Some(v) = remove_assignee { args.push("--remove-assignee".to_string()); args.push(v.clone()); }
            if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
            Ok(args)
        }
        ForgeType::Gitlab => {
            let mut args = vec![pr_cmd.to_string(), "update".to_string()]; // "mr" "update"
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(v) = add_label { args.push("--label".to_string()); args.push(v.clone()); }
            if let Some(v) = remove_label { args.push("--unlabel".to_string()); args.push(v.clone()); }
            if let Some(v) = add_reviewer { args.push("--reviewer".to_string()); args.push(format!("+{}", v)); }
            if let Some(v) = remove_reviewer { args.push("--reviewer".to_string()); args.push(format!("-{}", v)); }
            if let Some(v) = add_assignee { args.push("--assignee".to_string()); args.push(format!("+{}", v)); }
            if let Some(v) = remove_assignee { args.push("--assignee".to_string()); args.push(format!("-{}", v)); }
            if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
            Ok(args)
        }
        ForgeType::Forgejo => {
            // Subcommand routing: fj pr edit <N> labels --add/--rm
            let mut args = vec![pr_cmd.to_string(), "edit".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            // Only add "labels" subcommand if label flags are present
            if add_label.is_some() || remove_label.is_some() {
                args.push("labels".to_string());
                if let Some(v) = add_label { args.push("--add".to_string()); args.push(v.clone()); }
                if let Some(v) = remove_label { args.push("--rm".to_string()); args.push(v.clone()); }
            }
            if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
            Ok(args)
        }
        ForgeType::Gitea => unreachable!(), // handled in validation above
    }
}

fn translate_pr_checks(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    let number = matches.get_one::<String>("number");
    let extra: Option<Vec<String>> = matches
        .get_many::<String>("extra")
        .map(|vals| vals.cloned().collect());

    match forge {
        ForgeType::Github => {
            let mut args = vec![pr_cmd.to_string(), "checks".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Gitlab => {
            let mut args = vec!["ci".to_string(), "status".to_string()];
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Forgejo => {
            let mut args = vec![pr_cmd.to_string(), "status".to_string()];
            if let Some(n) = number { args.push(n.clone()); }
            if let Some(e) = extra { args.extend(e); }
            Ok(args)
        }
        ForgeType::Gitea => Err(GfError::UnsupportedFeature {
            feature: "pr checks".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        }),
    }
}

fn translate_pr_comment(
    forge: ForgeType,
    pr_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    if matches!(forge, ForgeType::Gitea) {
        return Err(GfError::UnsupportedFeature {
            feature: "pr comment".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }
    let number = matches.get_one::<String>("number");
    let body = matches.get_one::<String>("body");
    let mut args = vec![pr_cmd.to_string()];
    match forge {
        ForgeType::Gitlab => args.push("note".to_string()),
        _ => args.push("comment".to_string()),
    }
    if let Some(n) = number { args.push(n.clone()); }
    if let Some(b) = body {
        match forge {
            ForgeType::Gitlab => { args.push("--message".to_string()); args.push(b.clone()); }
            ForgeType::Forgejo => { args.push(b.clone()); }
            _ => { args.push("--body".to_string()); args.push(b.clone()); }
        }
    }
    if let Some(extra) = matches.get_many::<String>("extra") { args.extend(extra.cloned()); }
    Ok(args)
}
