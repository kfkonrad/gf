// src/adapter/repo_auth.rs — Repo and Auth translation
use crate::error::GfError;
use crate::forge::ForgeType;
use clap::ArgMatches;

// ─── Repo ────────────────────────────────────────────────────────────────────

/// Translate `gf repo ...` `ArgMatches` into forge-specific args.
///
/// # Errors
/// Returns [`GfError::UnsupportedFeature`] when the requested repo operation is
/// not supported by the target forge.
pub fn translate_repo(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    let repo_cmd = repo_subcommand_name(forge);

    match matches.subcommand() {
        Some(("view", sub)) => Ok(translate_repo_view(forge, repo_cmd, sub)),
        Some(("create", sub)) => Ok(translate_repo_create(forge, repo_cmd, sub)),
        Some(("fork", sub)) => Ok(translate_repo_fork(forge, repo_cmd, sub)),
        Some(("clone", sub)) => translate_repo_clone(forge, repo_cmd, sub),
        Some((verb, sub)) => {
            let mut args = vec![repo_cmd.to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            Ok(args)
        }
        None => Ok(vec![repo_cmd.to_string()]),
    }
}

/// The repo subcommand name per forge.
/// tea uses "repos" (plural); all others use "repo".
const fn repo_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Gitea => "repos",
        _ => "repo",
    }
}

fn translate_repo_view(forge: ForgeType, repo_cmd: &str, matches: &ArgMatches) -> Vec<String> {
    let _ = forge;
    let mut args = vec![repo_cmd.to_string(), "view".to_string()];
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}

fn translate_repo_create(forge: ForgeType, repo_cmd: &str, matches: &ArgMatches) -> Vec<String> {
    let mut args = vec![repo_cmd.to_string(), "create".to_string()];

    // --name: positional for gh and glab; --name flag for tea and fj (REPO-02)
    if let Some(name) = matches.get_one::<String>("name") {
        match forge {
            ForgeType::Github | ForgeType::Gitlab => {
                // Positional — just push the value, no flag
                args.push(name.clone());
            }
            ForgeType::Gitea | ForgeType::Forgejo => {
                args.push("--name".to_string());
                args.push(name.clone());
            }
        }
    }

    // --description: same flag name for all forges
    if let Some(desc) = matches.get_one::<String>("description") {
        args.push("--description".to_string());
        args.push(desc.clone());
    }

    // --private / --public: translate to --visibility for glab (Pitfall 4 from RESEARCH.md)
    if matches.get_flag("private") {
        match forge {
            ForgeType::Gitlab => {
                args.push("--visibility".to_string());
                args.push("private".to_string());
            }
            _ => {
                args.push("--private".to_string());
            }
        }
    } else if matches.get_flag("public") {
        match forge {
            ForgeType::Gitlab => {
                args.push("--visibility".to_string());
                args.push("public".to_string());
            }
            ForgeType::Github => {
                args.push("--public".to_string());
            }
            // Gitea and Forgejo: public is default, omit flag
            _ => {}
        }
    }

    // --homepage: only gh supports it; others silently omit
    if let Some(url) = matches.get_one::<String>("homepage") {
        if forge == ForgeType::Github {
            args.push("--homepage".to_string());
            args.push(url.clone());
        } else {
            // --homepage not supported on glab/tea/fj; silently omit
        }
    }

    // Passthrough
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    args
}

fn translate_repo_fork(forge: ForgeType, repo_cmd: &str, matches: &ArgMatches) -> Vec<String> {
    let _ = forge;
    let mut args = vec![repo_cmd.to_string(), "fork".to_string()];
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}

/// Translate `gf repo clone <repo>` where repo is either:
///   - owner/repo shorthand (requires [defaults] `clone_host` config)
///   - full URL (<https://host/owner/repo> or git@host:owner/repo)
///
/// Tea has no clone subcommand → `UnsupportedFeature` error.
///
/// # Errors
/// Returns [`GfError::UnsupportedFeature`] when `forge` is Gitea, which has no
/// clone subcommand.
fn translate_repo_clone(
    forge: ForgeType,
    repo_cmd: &str,
    matches: &ArgMatches,
) -> Result<Vec<String>, GfError> {
    // tea has no repos clone
    if forge == ForgeType::Gitea {
        return Err(GfError::UnsupportedFeature {
            feature: "repo clone".to_string(),
            forge: "Gitea".to_string(),
            forge_cli: "tea".to_string(),
        });
    }

    let repo = matches.get_one::<String>("repo").expect("repo is required");

    // Detect if repo is a full URL or owner/repo shorthand.
    // In every case we currently pass the value through as-is:
    //   - Full URL (https://…, http://…, git@…): pass through verbatim.
    //   - owner/repo shorthand: gh/glab/fj know their default hosts, so pass through.
    //   - Unrecognized format: pass through and let the forge CLI error.
    let resolved_repo = repo.clone();

    let mut args = vec![repo_cmd.to_string(), "clone".to_string(), resolved_repo];

    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    Ok(args)
}

// ─── Auth ────────────────────────────────────────────────────────────────────

/// Translate `gf auth ...` `ArgMatches` into forge-specific args.
///
/// Tea has NO `auth` subcommand — its auth is under `logins` (Pitfall 3 from RESEARCH.md):
///   gf auth login  → tea logins add
///   gf auth logout → tea logins rm
///   gf auth status → tea logins ls
///
/// Forgejo uses `auth add-key` for login, `auth list` for status.
#[must_use]
pub fn translate_auth(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    match matches.subcommand() {
        Some(("login", sub)) => translate_auth_login(forge, sub),
        Some(("logout", sub)) => translate_auth_logout(forge, sub),
        Some(("status", sub)) => translate_auth_status(forge, sub),
        Some((verb, sub)) => {
            let mut args = vec!["auth".to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            args
        }
        None => vec!["auth".to_string()],
    }
}

fn translate_auth_login(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    // Subcommand remap per forge
    let mut args = match forge {
        ForgeType::Github | ForgeType::Gitlab => vec!["auth".to_string(), "login".to_string()],
        ForgeType::Gitea => vec!["logins".to_string(), "add".to_string()],
        ForgeType::Forgejo => vec!["auth".to_string(), "add-key".to_string()],
    };

    // --hostname: gh and glab use --hostname; tea uses --url; fj does not support it
    if let Some(host) = matches.get_one::<String>("hostname") {
        match forge {
            ForgeType::Github | ForgeType::Gitlab => {
                args.push("--hostname".to_string());
                args.push(host.clone());
            }
            ForgeType::Gitea => {
                args.push("--url".to_string());
                args.push(host.clone());
            }
            ForgeType::Forgejo => {
                // fj auth add-key takes positional args, no --hostname; silently omit
            }
        }
    }

    // --token: all CLIs accept --token except fj (positional args for add-key)
    if let Some(token) = matches.get_one::<String>("token") {
        if forge == ForgeType::Forgejo {
            // fj auth add-key <USER> [KEY] — token is positional, not --token flag
            // Cannot map without username; silently omit
        } else {
            args.push("--token".to_string());
            args.push(token.clone());
        }
    }

    // Passthrough
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    args
}

fn translate_auth_logout(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    let mut args = match forge {
        ForgeType::Github | ForgeType::Gitlab | ForgeType::Forgejo => {
            vec!["auth".to_string(), "logout".to_string()]
        }
        ForgeType::Gitea => vec!["logins".to_string(), "rm".to_string()],
    };
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}

fn translate_auth_status(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    let mut args = match forge {
        ForgeType::Github | ForgeType::Gitlab => vec!["auth".to_string(), "status".to_string()],
        ForgeType::Gitea => vec!["logins".to_string(), "ls".to_string()],
        ForgeType::Forgejo => vec!["auth".to_string(), "list".to_string()],
    };
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}
