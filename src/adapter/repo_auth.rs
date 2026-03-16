// src/adapter/repo_auth.rs — Repo and Auth translation
use crate::forge::ForgeType;
use clap::ArgMatches;

// ─── Repo ────────────────────────────────────────────────────────────────────

/// Translate `gf repo ...` ArgMatches into forge-specific args.
pub fn translate_repo(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    let repo_cmd = repo_subcommand_name(forge);

    match matches.subcommand() {
        Some(("view", sub)) => translate_repo_view(forge, repo_cmd, sub),
        Some(("create", sub)) => translate_repo_create(forge, repo_cmd, sub),
        Some(("fork", sub)) => translate_repo_fork(forge, repo_cmd, sub),
        Some((verb, sub)) => {
            let mut args = vec![repo_cmd.to_string(), verb.to_string()];
            if let Some(extra) = sub.get_many::<String>("extra") {
                args.extend(extra.cloned());
            }
            args
        }
        None => vec![repo_cmd.to_string()],
    }
}

/// The repo subcommand name per forge.
/// tea uses "repos" (plural); all others use "repo".
fn repo_subcommand_name(forge: ForgeType) -> &'static str {
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

    // --homepage: gh supports it; others receive it as passthrough
    if let Some(url) = matches.get_one::<String>("homepage") {
        args.push("--homepage".to_string());
        args.push(url.clone());
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

// ─── Auth ────────────────────────────────────────────────────────────────────

/// Translate `gf auth ...` ArgMatches into forge-specific args.
///
/// Tea has NO `auth` subcommand — its auth is under `logins` (Pitfall 3 from RESEARCH.md):
///   gf auth login  → tea logins add
///   gf auth logout → tea logins rm
///   gf auth status → tea logins ls
///
/// Forgejo uses `auth add-key` for login, `auth list` for status.
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
        ForgeType::Github => vec!["auth".to_string(), "login".to_string()],
        ForgeType::Gitlab => vec!["auth".to_string(), "login".to_string()],
        ForgeType::Gitea => vec!["logins".to_string(), "add".to_string()],
        ForgeType::Forgejo => vec!["auth".to_string(), "add-key".to_string()],
    };

    // --hostname: gh and glab use --hostname; tea uses --url; fj passthrough
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
                // fj: passthrough — push as-is
                args.push("--hostname".to_string());
                args.push(host.clone());
            }
        }
    }

    // --token: all CLIs accept --token
    if let Some(token) = matches.get_one::<String>("token") {
        args.push("--token".to_string());
        args.push(token.clone());
    }

    // Passthrough
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    args
}

fn translate_auth_logout(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    let mut args = match forge {
        ForgeType::Github => vec!["auth".to_string(), "logout".to_string()],
        ForgeType::Gitlab => vec!["auth".to_string(), "logout".to_string()],
        ForgeType::Gitea => vec!["logins".to_string(), "rm".to_string()],
        ForgeType::Forgejo => vec!["auth".to_string(), "logout".to_string()],
    };
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}

fn translate_auth_status(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    let mut args = match forge {
        ForgeType::Github => vec!["auth".to_string(), "status".to_string()],
        ForgeType::Gitlab => vec!["auth".to_string(), "status".to_string()],
        ForgeType::Gitea => vec!["logins".to_string(), "ls".to_string()],
        ForgeType::Forgejo => vec!["auth".to_string(), "list".to_string()],
    };
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::build_cli;

    // ── Repo helpers ──────────────────────────────────────────────────────────

    fn parse_repo_subcommand(verb: &str, extra_args: &[&str]) -> ArgMatches {
        let mut cmd_args = vec!["gf", "repo", verb];
        cmd_args.extend_from_slice(extra_args);
        let matches = build_cli().try_get_matches_from(cmd_args).expect("parse ok");
        let (_, repo_sub) = matches.subcommand().expect("repo subcommand");
        let (_, verb_sub) = repo_sub.subcommand().expect("verb subcommand");
        verb_sub.clone()
    }

    // ── Auth helpers ──────────────────────────────────────────────────────────

    fn parse_auth_subcommand(verb: &str, extra_args: &[&str]) -> ArgMatches {
        let mut cmd_args = vec!["gf", "auth", verb];
        cmd_args.extend_from_slice(extra_args);
        let matches = build_cli().try_get_matches_from(cmd_args).expect("parse ok");
        let (_, auth_sub) = matches.subcommand().expect("auth subcommand");
        let (_, verb_sub) = auth_sub.subcommand().expect("verb subcommand");
        verb_sub.clone()
    }

    // ── REPO-01: view ─────────────────────────────────────────────────────────

    #[test]
    fn test_repo_view_github() {
        let sub = parse_repo_subcommand("view", &[]);
        let result = translate_repo_view(ForgeType::Github, "repo", &sub);
        assert_eq!(result, vec!["repo", "view"]);
    }

    #[test]
    fn test_repo_view_gitea_uses_repos() {
        let matches = build_cli().try_get_matches_from(["gf", "repo", "view"]).unwrap();
        let (_, s) = matches.subcommand().unwrap();
        let result = translate_repo(ForgeType::Gitea, s);
        assert_eq!(&result[0], "repos", "tea should use 'repos': {result:?}");
    }

    // ── REPO-02: create visibility ────────────────────────────────────────────

    #[test]
    fn test_repo_create_private_glab_visibility() {
        let sub = parse_repo_subcommand("create", &["--private"]);
        let result = translate_repo_create(ForgeType::Gitlab, "repo", &sub);
        assert!(result.contains(&"--visibility".to_string()), "glab should use --visibility: {result:?}");
        assert!(result.contains(&"private".to_string()));
        assert!(!result.contains(&"--private".to_string()), "glab should NOT use --private: {result:?}");
    }

    #[test]
    fn test_repo_create_private_github() {
        let sub = parse_repo_subcommand("create", &["--private"]);
        let result = translate_repo_create(ForgeType::Github, "repo", &sub);
        assert!(result.contains(&"--private".to_string()), "gh uses --private: {result:?}");
        assert!(!result.contains(&"--visibility".to_string()));
    }

    #[test]
    fn test_repo_create_public_glab_visibility() {
        let sub = parse_repo_subcommand("create", &["--public"]);
        let result = translate_repo_create(ForgeType::Gitlab, "repo", &sub);
        assert!(result.contains(&"--visibility".to_string()));
        assert!(result.contains(&"public".to_string()));
    }

    #[test]
    fn test_repo_create_name_positional_for_github() {
        let sub = parse_repo_subcommand("create", &["--name", "myrepo"]);
        let result = translate_repo_create(ForgeType::Github, "repo", &sub);
        assert!(result.contains(&"myrepo".to_string()), "name should appear: {result:?}");
        assert!(!result.contains(&"--name".to_string()), "gh uses positional, no --name flag: {result:?}");
    }

    #[test]
    fn test_repo_create_name_flag_for_gitea() {
        let sub = parse_repo_subcommand("create", &["--name", "myrepo"]);
        let result = translate_repo_create(ForgeType::Gitea, "repos", &sub);
        assert!(result.contains(&"--name".to_string()), "tea uses --name flag: {result:?}");
        assert!(result.contains(&"myrepo".to_string()));
    }

    // ── REPO-03: fork ─────────────────────────────────────────────────────────

    #[test]
    fn test_repo_fork_github() {
        let sub = parse_repo_subcommand("fork", &[]);
        let result = translate_repo_fork(ForgeType::Github, "repo", &sub);
        assert_eq!(&result[0..2], &["repo", "fork"]);
    }

    // ── AUTH-01: login ────────────────────────────────────────────────────────

    #[test]
    fn test_auth_login_github() {
        let sub = parse_auth_subcommand("login", &[]);
        let result = translate_auth_login(ForgeType::Github, &sub);
        assert_eq!(&result[0..2], &["auth", "login"]);
    }

    #[test]
    fn test_auth_login_gitlab() {
        let sub = parse_auth_subcommand("login", &[]);
        let result = translate_auth_login(ForgeType::Gitlab, &sub);
        assert_eq!(&result[0..2], &["auth", "login"]);
    }

    #[test]
    fn test_auth_login_tea_remaps_to_logins_add() {
        let sub = parse_auth_subcommand("login", &[]);
        let result = translate_auth_login(ForgeType::Gitea, &sub);
        assert_eq!(&result[0..2], &["logins", "add"], "tea auth login → logins add: {result:?}");
    }

    #[test]
    fn test_auth_login_fj_remaps_to_auth_add_key() {
        let sub = parse_auth_subcommand("login", &[]);
        let result = translate_auth_login(ForgeType::Forgejo, &sub);
        assert_eq!(&result[0..2], &["auth", "add-key"], "fj auth login → auth add-key: {result:?}");
    }

    // ── AUTH-02: logout ───────────────────────────────────────────────────────

    #[test]
    fn test_auth_logout_tea_remaps_to_logins_rm() {
        let sub = parse_auth_subcommand("logout", &[]);
        let result = translate_auth_logout(ForgeType::Gitea, &sub);
        assert_eq!(&result[0..2], &["logins", "rm"], "tea auth logout → logins rm: {result:?}");
    }

    #[test]
    fn test_auth_logout_github() {
        let sub = parse_auth_subcommand("logout", &[]);
        let result = translate_auth_logout(ForgeType::Github, &sub);
        assert_eq!(&result[0..2], &["auth", "logout"]);
    }

    // ── AUTH-03: status ───────────────────────────────────────────────────────

    #[test]
    fn test_auth_status_tea_remaps_to_logins_ls() {
        let sub = parse_auth_subcommand("status", &[]);
        let result = translate_auth_status(ForgeType::Gitea, &sub);
        assert_eq!(&result[0..2], &["logins", "ls"], "tea auth status → logins ls: {result:?}");
    }

    #[test]
    fn test_auth_status_fj_remaps_to_auth_list() {
        let sub = parse_auth_subcommand("status", &[]);
        let result = translate_auth_status(ForgeType::Forgejo, &sub);
        assert_eq!(&result[0..2], &["auth", "list"], "fj auth status → auth list: {result:?}");
    }

    #[test]
    fn test_auth_status_github() {
        let sub = parse_auth_subcommand("status", &[]);
        let result = translate_auth_status(ForgeType::Github, &sub);
        assert_eq!(&result[0..2], &["auth", "status"]);
    }

    // ── Auth flag translation ─────────────────────────────────────────────────

    #[test]
    fn test_auth_login_hostname_github() {
        let sub = parse_auth_subcommand("login", &["--hostname", "git.corp.com"]);
        let result = translate_auth_login(ForgeType::Github, &sub);
        assert!(result.contains(&"--hostname".to_string()));
        assert!(result.contains(&"git.corp.com".to_string()));
    }

    #[test]
    fn test_auth_login_hostname_tea_uses_url() {
        let sub = parse_auth_subcommand("login", &["--hostname", "git.corp.com"]);
        let result = translate_auth_login(ForgeType::Gitea, &sub);
        assert!(result.contains(&"--url".to_string()), "tea uses --url not --hostname: {result:?}");
        assert!(result.contains(&"git.corp.com".to_string()));
        assert!(!result.contains(&"--hostname".to_string()), "tea should NOT have --hostname: {result:?}");
    }

    #[test]
    fn test_auth_login_token_passthrough() {
        let sub = parse_auth_subcommand("login", &["--token", "abc123"]);
        let result = translate_auth_login(ForgeType::Github, &sub);
        assert!(result.contains(&"--token".to_string()));
        assert!(result.contains(&"abc123".to_string()));
    }
}
