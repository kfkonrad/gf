// src/adapter/pr.rs — PR/MR subcommand and flag translation
use crate::forge::ForgeType;
use clap::ArgMatches;

/// Translate `gf pr ...` ArgMatches into forge-specific args.
/// Called by adapter::translate() when the matched subcommand is "pr" (or "mr" alias).
pub fn translate_pr(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
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
            args
        }
        None => vec![pr_cmd.to_string()],
    }
}

/// Maps the canonical "pr" command to the forge-specific equivalent (PR-03).
fn pr_subcommand_name(forge: ForgeType) -> &'static str {
    match forge {
        ForgeType::Github  => "pr",
        ForgeType::Gitlab  => "mr",
        ForgeType::Gitea   => "pulls",
        ForgeType::Forgejo => "pr",
    }
}

/// Translate `gf pr create` with canonical flags (PR-01, PR-02, PR-04).
fn translate_pr_create(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Vec<String> {
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

    // --draft: gh and glab support natively; tea and fj: pass through (let CLI handle it)
    if matches.get_flag("draft") {
        args.push("--draft".to_string());
    }

    // Passthrough: unrecognized flags appended verbatim (PR-04)
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    args
}

/// Translate `gf pr view [<number>]` (PR-05, PR-06).
/// Delegates to the underlying CLI with or without number.
/// Current-branch PR lookup is handled natively by gh/glab/tea/fj.
fn translate_pr_view(forge: ForgeType, pr_cmd: &str, matches: &ArgMatches) -> Vec<String> {
    let mut args = vec![pr_cmd.to_string(), "view".to_string()];

    // Number is optional (PR-05): if not provided, the underlying CLI finds the current-branch PR
    if let Some(number) = matches.get_one::<String>("number") {
        args.push(number.clone());
    }

    // Passthrough for any extra flags (PR-04)
    if let Some(extra) = matches.get_many::<String>("extra") {
        args.extend(extra.cloned());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::build_cli;

    /// Helper: parse `gf pr create [args...]` and extract the pr submatches.
    fn parse_pr_create(extra_args: &[&str]) -> ArgMatches {
        let mut cmd_args = vec!["gf", "pr", "create"];
        cmd_args.extend_from_slice(extra_args);
        let matches = build_cli()
            .try_get_matches_from(cmd_args)
            .expect("parse succeeded");
        let (_, pr_sub) = matches.subcommand().expect("pr subcommand");
        let (_, create_sub) = pr_sub.subcommand().expect("create subcommand");
        create_sub.clone()
    }

    /// Helper: parse `gf pr view [args...]` and extract pr submatches.
    fn parse_pr_view(extra_args: &[&str]) -> ArgMatches {
        let mut cmd_args = vec!["gf", "pr", "view"];
        cmd_args.extend_from_slice(extra_args);
        let matches = build_cli()
            .try_get_matches_from(cmd_args)
            .expect("parse succeeded");
        let (_, pr_sub) = matches.subcommand().expect("pr subcommand");
        let (_, view_sub) = pr_sub.subcommand().expect("view subcommand");
        view_sub.clone()
    }

    // --- PR-03: Subcommand name translation ---

    #[test]
    fn test_pr_subcommand_github() {
        assert_eq!(pr_subcommand_name(ForgeType::Github), "pr");
    }

    #[test]
    fn test_pr_subcommand_gitlab() {
        assert_eq!(pr_subcommand_name(ForgeType::Gitlab), "mr");
    }

    #[test]
    fn test_pr_subcommand_gitea() {
        assert_eq!(pr_subcommand_name(ForgeType::Gitea), "pulls");
    }

    #[test]
    fn test_pr_subcommand_forgejo() {
        assert_eq!(pr_subcommand_name(ForgeType::Forgejo), "pr");
    }

    // --- PR-01: Full GitHub invocation ---

    #[test]
    fn test_pr_create_github_full() {
        let sub = parse_pr_create(&["--title", "fix", "--body", "details", "--base", "main", "--draft"]);
        let result = translate_pr_create(ForgeType::Github, "pr", &sub);
        assert_eq!(result, vec!["pr", "create", "--title", "fix", "--body", "details", "--base", "main", "--draft"]);
    }

    // --- PR-02: GitLab flag translation ---

    #[test]
    fn test_pr_create_glab_body_translates_to_description() {
        let sub = parse_pr_create(&["--body", "hello"]);
        let result = translate_pr_create(ForgeType::Gitlab, "mr", &sub);
        assert!(result.contains(&"--description".to_string()), "should contain --description: {result:?}");
        assert!(result.contains(&"hello".to_string()), "should contain the value: {result:?}");
        assert!(!result.contains(&"--body".to_string()), "should NOT contain --body: {result:?}");
    }

    #[test]
    fn test_pr_create_glab_base_translates_to_target_branch() {
        let sub = parse_pr_create(&["--base", "main"]);
        let result = translate_pr_create(ForgeType::Gitlab, "mr", &sub);
        assert!(result.contains(&"--target-branch".to_string()), "should contain --target-branch: {result:?}");
        assert!(result.contains(&"main".to_string()));
        assert!(!result.contains(&"--base".to_string()), "should NOT contain --base: {result:?}");
    }

    #[test]
    fn test_pr_create_glab_title_unchanged() {
        let sub = parse_pr_create(&["--title", "fix"]);
        let result = translate_pr_create(ForgeType::Gitlab, "mr", &sub);
        assert!(result.contains(&"--title".to_string()));
        assert!(result.contains(&"fix".to_string()));
    }

    #[test]
    fn test_pr_create_glab_draft_passes_through() {
        let sub = parse_pr_create(&["--draft"]);
        let result = translate_pr_create(ForgeType::Gitlab, "mr", &sub);
        assert!(result.contains(&"--draft".to_string()), "--draft should appear in glab args (supported natively): {result:?}");
    }

    // --- PR-02: Gitea flag translation ---

    #[test]
    fn test_pr_create_tea_body_translates_to_description() {
        let sub = parse_pr_create(&["--body", "hello"]);
        let result = translate_pr_create(ForgeType::Gitea, "pulls", &sub);
        assert!(result.contains(&"--description".to_string()), "tea should use --description: {result:?}");
        assert!(!result.contains(&"--body".to_string()), "tea should NOT use --body: {result:?}");
    }

    #[test]
    fn test_pr_create_tea_base_unchanged() {
        let sub = parse_pr_create(&["--base", "main"]);
        let result = translate_pr_create(ForgeType::Gitea, "pulls", &sub);
        assert!(result.contains(&"--base".to_string()), "tea uses --base natively: {result:?}");
    }

    // --- PR-02: Forgejo flag translation ---

    #[test]
    fn test_pr_create_fj_body_unchanged() {
        let sub = parse_pr_create(&["--body", "hello"]);
        let result = translate_pr_create(ForgeType::Forgejo, "pr", &sub);
        assert!(result.contains(&"--body".to_string()), "fj uses --body natively: {result:?}");
        assert!(result.contains(&"hello".to_string()));
    }

    #[test]
    fn test_pr_create_fj_base_unchanged() {
        let sub = parse_pr_create(&["--base", "main"]);
        let result = translate_pr_create(ForgeType::Forgejo, "pr", &sub);
        assert!(result.contains(&"--base".to_string()));
        assert!(result.contains(&"main".to_string()));
    }

    // --- PR-04: Passthrough flags ---

    #[test]
    fn test_pr_create_passthrough_assignee() {
        // Extra args after -- are captured as passthrough
        let sub = parse_pr_create(&["--title", "fix", "--", "--assignee", "alice"]);
        let result = translate_pr_create(ForgeType::Github, "pr", &sub);
        assert!(result.contains(&"--assignee".to_string()), "passthrough --assignee should appear: {result:?}");
        assert!(result.contains(&"alice".to_string()));
    }

    #[test]
    fn test_pr_create_passthrough_label() {
        let sub = parse_pr_create(&["--", "--label", "bug"]);
        let result = translate_pr_create(ForgeType::Github, "pr", &sub);
        assert!(result.contains(&"--label".to_string()), "passthrough --label should appear: {result:?}");
        assert!(result.contains(&"bug".to_string()));
    }

    // --- PR-05: View with and without number ---

    #[test]
    fn test_pr_view_no_number_github() {
        let sub = parse_pr_view(&[]);
        let result = translate_pr_view(ForgeType::Github, "pr", &sub);
        assert_eq!(result, vec!["pr", "view"], "no number: {result:?}");
    }

    #[test]
    fn test_pr_view_no_number_gitlab() {
        let sub = parse_pr_view(&[]);
        let result = translate_pr_view(ForgeType::Gitlab, "mr", &sub);
        assert_eq!(result, vec!["mr", "view"], "glab no number: {result:?}");
    }

    #[test]
    fn test_pr_view_with_number_github() {
        let sub = parse_pr_view(&["42"]);
        let result = translate_pr_view(ForgeType::Github, "pr", &sub);
        assert_eq!(result, vec!["pr", "view", "42"]);
    }

    #[test]
    fn test_pr_view_with_number_gitlab() {
        let sub = parse_pr_view(&["7"]);
        let result = translate_pr_view(ForgeType::Gitlab, "mr", &sub);
        assert_eq!(result, vec!["mr", "view", "7"]);
    }

    // --- Integration: translate_pr dispatch ---

    #[test]
    fn test_translate_pr_dispatch_create() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "pr", "create", "--title", "t"])
            .unwrap();
        let (_, pr_sub) = matches.subcommand().unwrap();
        let result = translate_pr(ForgeType::Github, pr_sub);
        assert_eq!(&result[0], "pr");
        assert_eq!(&result[1], "create");
    }

    #[test]
    fn test_translate_pr_dispatch_view() {
        let matches = build_cli()
            .try_get_matches_from(["gf", "pr", "view"])
            .unwrap();
        let (_, pr_sub) = matches.subcommand().unwrap();
        let result = translate_pr(ForgeType::Gitlab, pr_sub);
        assert_eq!(&result[0], "mr");
        assert_eq!(&result[1], "view");
    }
}
