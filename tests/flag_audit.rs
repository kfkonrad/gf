// tests/flag_audit.rs — Declarative test table for flag translations + integration audit
//
// This is the SINGLE SOURCE OF TRUTH for all (command, flag, forge) translation tests.
// Inline #[cfg(test)] blocks in adapter/*.rs are removed; all translation coverage lives here.
//
// Macro-generated tests use the public API: gf::adapter::translate() via gf::cmd::build_cli().
// Integration audit tests call real forge CLIs with --help to verify flag existence.

use gf::forge::ForgeType;

// ═══════════════════════════════════════════════════════════════════════════════
// translation_test! — generates one #[test] per invocation
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! translation_test {
    ($name:ident, input: [$($arg:expr),+ $(,)?], forge: $forge:expr, expected: [$($exp:expr),+ $(,)?]) => {
        #[test]
        fn $name() {
            let matches = gf::cmd::build_cli()
                .try_get_matches_from([$($arg),+])
                .unwrap_or_else(|e| panic!("clap parse failed: {e}"));
            let result = gf::adapter::translate($forge, &matches).unwrap_or_else(|e| panic!("translate returned error: {e}"));
            let expected: Vec<String> = vec![$($exp.to_string()),+];
            assert_eq!(result, expected, "forge={:?}", $forge);
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// forge_help_contains — integration audit helper
// ═══════════════════════════════════════════════════════════════════════════════

fn forge_help_contains(cli: &str, subargs: &[&str], expected_fragment: &str) {
    let output = std::process::Command::new(cli)
        .args(subargs)
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("{cli} not found on PATH: {e}"));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains(expected_fragment),
        "AUDIT FAIL: `{cli} {} --help` missing '{}'\nOutput:\n{text}",
        subargs.join(" "),
        expected_fragment
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// audit_test! — generates one #[test] per forge --help flag verification
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! audit_test {
    ($name:ident, cli: $cli:expr, args: [$($arg:expr),*], contains: $frag:expr) => {
        #[test]
        fn $name() {
            forge_help_contains($cli, &[$($arg),*], $frag);
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSLATION TEST TABLE — PR subcommand translations
// ═══════════════════════════════════════════════════════════════════════════════

// --- PR create: GitHub (all flags pass through) ---
translation_test!(pr_create_github_full,
    input: ["gf", "pr", "create", "--title", "fix", "--body", "details", "--base", "main", "--draft"],
    forge: ForgeType::Github,
    expected: ["pr", "create", "--title", "fix", "--body", "details", "--base", "main", "--draft"]
);

// --- PR create: GitLab (--body→--description, --base→--target-branch) ---
translation_test!(pr_create_glab_body,
    input: ["gf", "pr", "create", "--body", "hello"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "create", "--description", "hello"]
);

translation_test!(pr_create_glab_base,
    input: ["gf", "pr", "create", "--base", "main"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "create", "--target-branch", "main"]
);

translation_test!(pr_create_glab_draft,
    input: ["gf", "pr", "create", "--draft"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "create", "--draft"]
);

// --- PR create: Gitea (--body→--description, --base unchanged) ---
translation_test!(pr_create_tea_body,
    input: ["gf", "pr", "create", "--body", "hello"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "create", "--description", "hello"]
);

translation_test!(pr_create_tea_base,
    input: ["gf", "pr", "create", "--base", "main"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "create", "--base", "main"]
);

// --- PR create: Forgejo (--body and --base unchanged) ---
translation_test!(pr_create_fj_body,
    input: ["gf", "pr", "create", "--body", "hello"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "create", "--body", "hello"]
);

translation_test!(pr_create_fj_base,
    input: ["gf", "pr", "create", "--base", "main"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "create", "--base", "main"]
);

// --- PR view: GitHub ---
translation_test!(pr_view_github_number,
    input: ["gf", "pr", "view", "42"],
    forge: ForgeType::Github,
    expected: ["pr", "view", "42"]
);

translation_test!(pr_view_github_no_number,
    input: ["gf", "pr", "view"],
    forge: ForgeType::Github,
    expected: ["pr", "view"]
);

// --- PR view: GitLab ---
translation_test!(pr_view_glab_number,
    input: ["gf", "pr", "view", "7"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "view", "7"]
);

translation_test!(pr_view_glab_no_number,
    input: ["gf", "pr", "view"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "view"]
);

// --- PR view: Gitea (no "view" verb — tea uses "pulls <N>" directly) ---
translation_test!(pr_view_tea_number,
    input: ["gf", "pr", "view", "42"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "42"]
);

translation_test!(pr_view_tea_no_number,
    input: ["gf", "pr", "view"],
    forge: ForgeType::Gitea,
    expected: ["pulls"]
);

// --- PR create: --draft omitted for tea and fj (unsupported) ---
translation_test!(pr_create_tea_draft_omitted,
    input: ["gf", "pr", "create", "--draft"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "create"]
);

translation_test!(pr_create_fj_draft_omitted,
    input: ["gf", "pr", "create", "--draft"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "create"]
);

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSLATION TEST TABLE — Repo subcommand translations
// ═══════════════════════════════════════════════════════════════════════════════

// --- Repo view ---
translation_test!(repo_view_github,
    input: ["gf", "repo", "view"],
    forge: ForgeType::Github,
    expected: ["repo", "view"]
);

translation_test!(repo_view_gitea,
    input: ["gf", "repo", "view"],
    forge: ForgeType::Gitea,
    expected: ["repos", "view"]
);

// --- Repo create: --name positional for gh/glab, --name flag for tea/fj ---
translation_test!(repo_create_github_name,
    input: ["gf", "repo", "create", "--name", "myrepo"],
    forge: ForgeType::Github,
    expected: ["repo", "create", "myrepo"]
);

translation_test!(repo_create_gitea_name,
    input: ["gf", "repo", "create", "--name", "myrepo"],
    forge: ForgeType::Gitea,
    expected: ["repos", "create", "--name", "myrepo"]
);

// --- Repo create: --private/--public visibility translation ---
translation_test!(repo_create_glab_private,
    input: ["gf", "repo", "create", "--private"],
    forge: ForgeType::Gitlab,
    expected: ["repo", "create", "--visibility", "private"]
);

translation_test!(repo_create_github_private,
    input: ["gf", "repo", "create", "--private"],
    forge: ForgeType::Github,
    expected: ["repo", "create", "--private"]
);

translation_test!(repo_create_glab_public,
    input: ["gf", "repo", "create", "--public"],
    forge: ForgeType::Gitlab,
    expected: ["repo", "create", "--visibility", "public"]
);

// --- Repo fork ---
translation_test!(repo_fork_github,
    input: ["gf", "repo", "fork"],
    forge: ForgeType::Github,
    expected: ["repo", "fork"]
);

// --- Repo create: --homepage (gh-only, omitted for others) ---
translation_test!(repo_create_github_homepage,
    input: ["gf", "repo", "create", "--homepage", "https://example.com"],
    forge: ForgeType::Github,
    expected: ["repo", "create", "--homepage", "https://example.com"]
);

translation_test!(repo_create_glab_homepage_omitted,
    input: ["gf", "repo", "create", "--homepage", "https://example.com"],
    forge: ForgeType::Gitlab,
    expected: ["repo", "create"]
);

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSLATION TEST TABLE — Auth subcommand translations
// ═══════════════════════════════════════════════════════════════════════════════

// --- Auth login: subcommand remapping ---
translation_test!(auth_login_github,
    input: ["gf", "auth", "login"],
    forge: ForgeType::Github,
    expected: ["auth", "login"]
);

translation_test!(auth_login_glab,
    input: ["gf", "auth", "login"],
    forge: ForgeType::Gitlab,
    expected: ["auth", "login"]
);

translation_test!(auth_login_tea,
    input: ["gf", "auth", "login"],
    forge: ForgeType::Gitea,
    expected: ["logins", "add"]
);

translation_test!(auth_login_fj,
    input: ["gf", "auth", "login"],
    forge: ForgeType::Forgejo,
    expected: ["auth", "add-key"]
);

// --- Auth logout ---
translation_test!(auth_logout_tea,
    input: ["gf", "auth", "logout"],
    forge: ForgeType::Gitea,
    expected: ["logins", "rm"]
);

translation_test!(auth_logout_github,
    input: ["gf", "auth", "logout"],
    forge: ForgeType::Github,
    expected: ["auth", "logout"]
);

// --- Auth status ---
translation_test!(auth_status_tea,
    input: ["gf", "auth", "status"],
    forge: ForgeType::Gitea,
    expected: ["logins", "ls"]
);

translation_test!(auth_status_fj,
    input: ["gf", "auth", "status"],
    forge: ForgeType::Forgejo,
    expected: ["auth", "list"]
);

translation_test!(auth_status_github,
    input: ["gf", "auth", "status"],
    forge: ForgeType::Github,
    expected: ["auth", "status"]
);

// --- Auth login: flag translations ---
translation_test!(auth_login_hostname_github,
    input: ["gf", "auth", "login", "--hostname", "git.corp.com"],
    forge: ForgeType::Github,
    expected: ["auth", "login", "--hostname", "git.corp.com"]
);

translation_test!(auth_login_hostname_tea,
    input: ["gf", "auth", "login", "--hostname", "git.corp.com"],
    forge: ForgeType::Gitea,
    expected: ["logins", "add", "--url", "git.corp.com"]
);

translation_test!(auth_login_token_github,
    input: ["gf", "auth", "login", "--token", "abc123"],
    forge: ForgeType::Github,
    expected: ["auth", "login", "--token", "abc123"]
);

// --- Auth login: fj hostname and token silently omitted ---
translation_test!(auth_login_fj_hostname_omitted,
    input: ["gf", "auth", "login", "--hostname", "git.corp.com"],
    forge: ForgeType::Forgejo,
    expected: ["auth", "add-key"]
);

translation_test!(auth_login_fj_token_omitted,
    input: ["gf", "auth", "login", "--token", "abc123"],
    forge: ForgeType::Forgejo,
    expected: ["auth", "add-key"]
);

// ═══════════════════════════════════════════════════════════════════════════════
// INTEGRATION AUDIT TESTS — verify translated flags exist in real forge CLIs
// ═══════════════════════════════════════════════════════════════════════════════

// --- gh pr create flags ---
audit_test!(audit_gh_pr_create_title,     cli: "gh",   args: ["pr", "create"],   contains: "--title");
audit_test!(audit_gh_pr_create_body,      cli: "gh",   args: ["pr", "create"],   contains: "--body");
audit_test!(audit_gh_pr_create_base,      cli: "gh",   args: ["pr", "create"],   contains: "--base");
audit_test!(audit_gh_pr_create_draft,     cli: "gh",   args: ["pr", "create"],   contains: "--draft");

// --- glab mr create flags ---
audit_test!(audit_glab_mr_create_description,    cli: "glab", args: ["mr", "create"],   contains: "--description");
audit_test!(audit_glab_mr_create_target_branch,  cli: "glab", args: ["mr", "create"],   contains: "--target-branch");
audit_test!(audit_glab_mr_create_draft,          cli: "glab", args: ["mr", "create"],   contains: "--draft");

// --- tea pulls create flags ---
audit_test!(audit_tea_pulls_create_description,  cli: "tea",  args: ["pulls", "create"], contains: "--description");
audit_test!(audit_tea_pulls_create_base,         cli: "tea",  args: ["pulls", "create"], contains: "--base");

// --- fj pr create flags ---
audit_test!(audit_fj_pr_create_body,  cli: "fj",  args: ["pr", "create"], contains: "--body");
audit_test!(audit_fj_pr_create_base,  cli: "fj",  args: ["pr", "create"], contains: "--base");

// --- gh repo create flags ---
audit_test!(audit_gh_repo_create_description,  cli: "gh",   args: ["repo", "create"], contains: "--description");
audit_test!(audit_gh_repo_create_private,      cli: "gh",   args: ["repo", "create"], contains: "--private");

// --- tea repos create flags ---
audit_test!(audit_tea_repos_create_name,  cli: "tea",  args: ["repos", "create"], contains: "--name");

// --- Auth: gh auth login flags ---
audit_test!(audit_gh_auth_login_hostname,  cli: "gh",   args: ["auth", "login"],  contains: "--hostname");
// Note: gh uses --with-token (reads from stdin), NOT --token. Our --token flag is for glab/tea/fj.

// --- Auth: glab auth login flags ---
audit_test!(audit_glab_auth_login_hostname,  cli: "glab", args: ["auth", "login"],  contains: "--hostname");

// --- Auth: tea logins add flags ---
audit_test!(audit_tea_logins_add_url,    cli: "tea",  args: ["logins", "add"],  contains: "--url");
audit_test!(audit_tea_logins_add_token,  cli: "tea",  args: ["logins", "add"],  contains: "--token");

// ═══════════════════════════════════════════════════════════════════════════════
// v11_translation_test! — like translation_test! but #[ignore]d until Phase 8
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! v11_translation_test {
    ($name:ident, input: [$($arg:expr),+ $(,)?], forge: $forge:expr, expected: [$($exp:expr),+ $(,)?]) => {
        #[test]
        #[ignore] // Phase 8: remove #[ignore] when adapter is implemented
        fn $name() {
            let matches = gf::cmd::build_cli()
                .try_get_matches_from([$($arg),+])
                .unwrap_or_else(|e| panic!("clap parse failed: {e}"));
            let result = gf::adapter::translate($forge, &matches).unwrap_or_else(|e| panic!("translate returned error: {e}"));
            let expected: Vec<String> = vec![$($exp.to_string()),+];
            assert_eq!(result, expected, "forge={:?}", $forge);
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// unsupported_test! — verifies that an unsupported command/flag returns GfError::UnsupportedFeature
// ═══════════════════════════════════════════════════════════════════════════════

macro_rules! unsupported_test {
    ($name:ident, input: [$($arg:expr),+ $(,)?], forge: $forge:expr, feature_contains: $feature:expr) => {
        #[test]
        fn $name() {
            let matches = gf::cmd::build_cli()
                .try_get_matches_from([$($arg),+])
                .unwrap_or_else(|e| panic!("clap parse failed: {e}"));
            let result = gf::adapter::translate($forge, &matches);
            match result {
                Err(gf::error::GfError::UnsupportedFeature { ref feature, .. }) => {
                    assert!(feature.contains($feature),
                        "expected feature containing '{}', got '{}'", $feature, feature);
                }
                Ok(args) => panic!("expected UnsupportedFeature error, got Ok({:?})", args),
                Err(e) => panic!("expected UnsupportedFeature error, got {:?}", e),
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// v1.1 Pre-Mapped Translations — PR list/merge/checkout/review, issue, repo clone
// ═══════════════════════════════════════════════════════════════════════════════
//
// These entries document the canonical-to-forge translation BEFORE Phase 8
// writes the adapter code. All are #[ignore]d and will fail until Phase 8
// implements both the CLI subcommands and the adapter translation logic.
// Run with: cargo test -- --ignored

// ── PR LIST (PR-01): gf pr list → gh pr list / glab mr list / tea pulls list / fj pr search ──

translation_test!(v11_pr_list_github_state,
    input: ["gf", "pr", "list", "--state", "closed"],
    forge: ForgeType::Github,
    expected: ["pr", "list", "--state", "closed"]
);

translation_test!(v11_pr_list_glab_state_closed,
    input: ["gf", "pr", "list", "--state", "closed"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "list", "--closed"]
);

translation_test!(v11_pr_list_glab_state_merged,
    input: ["gf", "pr", "list", "--state", "merged"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "list", "--merged"]
);

translation_test!(v11_pr_list_glab_state_all,
    input: ["gf", "pr", "list", "--state", "all"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "list", "--all"]
);

translation_test!(v11_pr_list_tea_state,
    input: ["gf", "pr", "list", "--state", "closed"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "list", "--state", "closed"]
);

translation_test!(v11_pr_list_fj_state,
    input: ["gf", "pr", "list", "--state", "closed"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "search", "--state", "closed"]
);

translation_test!(v11_pr_list_github_author,
    input: ["gf", "pr", "list", "--author", "alice"],
    forge: ForgeType::Github,
    expected: ["pr", "list", "--author", "alice"]
);

translation_test!(v11_pr_list_glab_author,
    input: ["gf", "pr", "list", "--author", "alice"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "list", "--author", "alice"]
);

// tea pr list --author: UNSUPPORTED (tea pulls list has no --author flag)

translation_test!(v11_pr_list_fj_author,
    input: ["gf", "pr", "list", "--author", "alice"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "search", "--creator", "alice"]
);

translation_test!(v11_pr_list_github_label,
    input: ["gf", "pr", "list", "--label", "bug"],
    forge: ForgeType::Github,
    expected: ["pr", "list", "--label", "bug"]
);

translation_test!(v11_pr_list_glab_label,
    input: ["gf", "pr", "list", "--label", "bug"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "list", "--label", "bug"]
);

// tea pr list --label: UNSUPPORTED (tea pulls list has no --label flag)

translation_test!(v11_pr_list_fj_label,
    input: ["gf", "pr", "list", "--label", "bug"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "search", "--labels", "bug"]
);

// --- PR list: tea unsupported flags (Phase 8 hard-error policy) ---
unsupported_test!(pr_list_tea_author_unsupported,
    input: ["gf", "pr", "list", "--author", "alice"],
    forge: ForgeType::Gitea,
    feature_contains: "pr list --author"
);

unsupported_test!(pr_list_tea_label_unsupported,
    input: ["gf", "pr", "list", "--label", "bug"],
    forge: ForgeType::Gitea,
    feature_contains: "pr list --label"
);

// ── PR MERGE (PR-02): gf pr merge → gh pr merge / glab mr merge / tea pulls merge / fj pr merge ──

translation_test!(v11_pr_merge_github_squash,
    input: ["gf", "pr", "merge", "--squash"],
    forge: ForgeType::Github,
    expected: ["pr", "merge", "--squash"]
);

translation_test!(v11_pr_merge_glab_squash,
    input: ["gf", "pr", "merge", "--squash"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "merge", "--squash"]
);

translation_test!(v11_pr_merge_tea_squash,
    input: ["gf", "pr", "merge", "--squash"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "merge", "--style", "squash"]
);

translation_test!(v11_pr_merge_fj_squash,
    input: ["gf", "pr", "merge", "--squash"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "merge", "--method", "squash"]
);

translation_test!(v11_pr_merge_tea_rebase,
    input: ["gf", "pr", "merge", "--rebase"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "merge", "--style", "rebase"]
);

translation_test!(v11_pr_merge_fj_rebase,
    input: ["gf", "pr", "merge", "--rebase"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "merge", "--method", "rebase"]
);

translation_test!(v11_pr_merge_glab_merge_default,
    input: ["gf", "pr", "merge", "--merge"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "merge"]
);
// --merge is glab's default strategy, no flag needed

// --- PR merge: default strategy (no --squash/--rebase/--merge flag) ---
translation_test!(pr_merge_github_default_strategy,
    input: ["gf", "pr", "merge"],
    forge: ForgeType::Github,
    expected: ["pr", "merge", "--merge"]
);

translation_test!(pr_merge_glab_default_strategy,
    input: ["gf", "pr", "merge"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "merge"]
);

translation_test!(pr_merge_tea_default_strategy,
    input: ["gf", "pr", "merge"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "merge", "--style", "merge"]
);

translation_test!(pr_merge_fj_default_strategy,
    input: ["gf", "pr", "merge"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "merge", "--method", "merge"]
);

// --- PR merge: --delete-branch flag translation ---
translation_test!(pr_merge_github_delete_branch,
    input: ["gf", "pr", "merge", "--delete-branch"],
    forge: ForgeType::Github,
    expected: ["pr", "merge", "--merge", "--delete-branch"]
);

translation_test!(pr_merge_glab_delete_branch,
    input: ["gf", "pr", "merge", "--delete-branch"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "merge", "--remove-source-branch"]
);

translation_test!(pr_merge_fj_delete_branch,
    input: ["gf", "pr", "merge", "--delete-branch"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "merge", "--method", "merge", "--delete"]
);

unsupported_test!(pr_merge_tea_delete_branch_unsupported,
    input: ["gf", "pr", "merge", "--delete-branch"],
    forge: ForgeType::Gitea,
    feature_contains: "pr merge --delete-branch"
);

// ── PR CHECKOUT (PR-03): gf pr checkout → gh pr checkout / glab mr checkout / tea pulls checkout / fj pr checkout ──

translation_test!(v11_pr_checkout_github,
    input: ["gf", "pr", "checkout", "42"],
    forge: ForgeType::Github,
    expected: ["pr", "checkout", "42"]
);

translation_test!(v11_pr_checkout_glab,
    input: ["gf", "pr", "checkout", "42"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "checkout", "42"]
);

translation_test!(v11_pr_checkout_tea,
    input: ["gf", "pr", "checkout", "42"],
    forge: ForgeType::Gitea,
    expected: ["pulls", "checkout", "42"]
);

translation_test!(v11_pr_checkout_fj,
    input: ["gf", "pr", "checkout", "42"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "checkout", "42"]
);

// ── PR REVIEW (PR-04, PR-05): gf pr review → gh pr review / glab mr comment|approve / fj pr comment ──

translation_test!(v11_pr_review_comment_github,
    input: ["gf", "pr", "review", "42", "--comment", "--body", "looks good"],
    forge: ForgeType::Github,
    expected: ["pr", "review", "42", "--comment", "--body", "looks good"]
);

translation_test!(v11_pr_review_comment_glab,
    input: ["gf", "pr", "review", "42", "--comment", "--body", "looks good"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "comment", "42", "--message", "looks good"]
);
// glab uses separate subcommand: `glab mr comment <N> --message <text>`

// tea pr review --comment: UNSUPPORTED (tea has no pulls review subcommand)

translation_test!(v11_pr_review_comment_fj,
    input: ["gf", "pr", "review", "42", "--comment", "--body", "looks good"],
    forge: ForgeType::Forgejo,
    expected: ["pr", "comment", "42", "looks good"]
);
// fj uses positional body: `fj pr comment <N> <body>`

translation_test!(v11_pr_review_approve_github,
    input: ["gf", "pr", "review", "42", "--approve"],
    forge: ForgeType::Github,
    expected: ["pr", "review", "42", "--approve"]
);

translation_test!(v11_pr_review_approve_glab,
    input: ["gf", "pr", "review", "42", "--approve"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "approve", "42"]
);
// glab uses separate subcommand: `glab mr approve <N>`

// tea pr review --approve: UNSUPPORTED (tea has no pulls review/approve)
// fj pr review --approve: UNSUPPORTED (fj has no pr approve subcommand)

// --- PR review/approve: tea/fj unsupported (Phase 8 hard-error policy) ---
unsupported_test!(pr_review_comment_tea_unsupported,
    input: ["gf", "pr", "review", "42", "--comment", "--body", "text"],
    forge: ForgeType::Gitea,
    feature_contains: "pr review --comment"
);

unsupported_test!(pr_review_approve_tea_unsupported,
    input: ["gf", "pr", "review", "42", "--approve"],
    forge: ForgeType::Gitea,
    feature_contains: "pr review --approve"
);

unsupported_test!(pr_review_approve_fj_unsupported,
    input: ["gf", "pr", "review", "42", "--approve"],
    forge: ForgeType::Forgejo,
    feature_contains: "pr review --approve"
);

// --- PR approve: syntactic sugar for review --approve ---
translation_test!(pr_approve_github,
    input: ["gf", "pr", "approve", "42"],
    forge: ForgeType::Github,
    expected: ["pr", "review", "42", "--approve"]
);

translation_test!(pr_approve_glab,
    input: ["gf", "pr", "approve", "42"],
    forge: ForgeType::Gitlab,
    expected: ["mr", "approve", "42"]
);

unsupported_test!(pr_approve_tea_unsupported,
    input: ["gf", "pr", "approve", "42"],
    forge: ForgeType::Gitea,
    feature_contains: "pr approve"
);

unsupported_test!(pr_approve_fj_unsupported,
    input: ["gf", "pr", "approve", "42"],
    forge: ForgeType::Forgejo,
    feature_contains: "pr approve"
);

// ── ISSUE LIST (ISSUE-01): gf issue list → gh issue list / glab issue list / tea issues list / fj issue search ──

v11_translation_test!(v11_issue_list_github_state,
    input: ["gf", "issue", "list", "--state", "closed"],
    forge: ForgeType::Github,
    expected: ["issue", "list", "--state", "closed"]
);

v11_translation_test!(v11_issue_list_glab_state_closed,
    input: ["gf", "issue", "list", "--state", "closed"],
    forge: ForgeType::Gitlab,
    expected: ["issue", "list", "--closed"]
);

v11_translation_test!(v11_issue_list_tea_state,
    input: ["gf", "issue", "list", "--state", "closed"],
    forge: ForgeType::Gitea,
    expected: ["issues", "list", "--state", "closed"]
);

v11_translation_test!(v11_issue_list_fj_state,
    input: ["gf", "issue", "list", "--state", "closed"],
    forge: ForgeType::Forgejo,
    expected: ["issue", "search", "--state", "closed"]
);

v11_translation_test!(v11_issue_list_github_label,
    input: ["gf", "issue", "list", "--label", "bug"],
    forge: ForgeType::Github,
    expected: ["issue", "list", "--label", "bug"]
);

v11_translation_test!(v11_issue_list_tea_label,
    input: ["gf", "issue", "list", "--label", "bug"],
    forge: ForgeType::Gitea,
    expected: ["issues", "list", "--labels", "bug"]
);

v11_translation_test!(v11_issue_list_fj_label,
    input: ["gf", "issue", "list", "--label", "bug"],
    forge: ForgeType::Forgejo,
    expected: ["issue", "search", "--labels", "bug"]
);

// ── ISSUE VIEW (ISSUE-02): gf issue view → gh issue view / glab issue view / tea issues <N> / fj issue view ──

v11_translation_test!(v11_issue_view_github,
    input: ["gf", "issue", "view", "42"],
    forge: ForgeType::Github,
    expected: ["issue", "view", "42"]
);

v11_translation_test!(v11_issue_view_glab,
    input: ["gf", "issue", "view", "42"],
    forge: ForgeType::Gitlab,
    expected: ["issue", "view", "42"]
);

v11_translation_test!(v11_issue_view_tea,
    input: ["gf", "issue", "view", "42"],
    forge: ForgeType::Gitea,
    expected: ["issues", "42"]
);
// tea has no "view" verb — uses `tea issues <number>` directly

v11_translation_test!(v11_issue_view_fj,
    input: ["gf", "issue", "view", "42"],
    forge: ForgeType::Forgejo,
    expected: ["issue", "view", "42"]
);

// ── ISSUE CREATE (ISSUE-03): gf issue create → gh issue create / glab issue create / tea issues create ──

v11_translation_test!(v11_issue_create_github,
    input: ["gf", "issue", "create", "--title", "bug", "--body", "details"],
    forge: ForgeType::Github,
    expected: ["issue", "create", "--title", "bug", "--body", "details"]
);

v11_translation_test!(v11_issue_create_glab,
    input: ["gf", "issue", "create", "--title", "bug", "--body", "details"],
    forge: ForgeType::Gitlab,
    expected: ["issue", "create", "--title", "bug", "--description", "details"]
);
// glab: --body → --description

v11_translation_test!(v11_issue_create_tea,
    input: ["gf", "issue", "create", "--title", "bug", "--body", "details"],
    forge: ForgeType::Gitea,
    expected: ["issues", "create", "--title", "bug", "--description", "details"]
);
// tea: --body → --description

// ── ISSUE CLOSE (ISSUE-04): gf issue close → gh issue close / glab issue close / tea issues close / fj issue close ──

v11_translation_test!(v11_issue_close_github,
    input: ["gf", "issue", "close", "42"],
    forge: ForgeType::Github,
    expected: ["issue", "close", "42"]
);

v11_translation_test!(v11_issue_close_glab,
    input: ["gf", "issue", "close", "42"],
    forge: ForgeType::Gitlab,
    expected: ["issue", "close", "42"]
);

v11_translation_test!(v11_issue_close_tea,
    input: ["gf", "issue", "close", "42"],
    forge: ForgeType::Gitea,
    expected: ["issues", "close", "42"]
);
// tea: uses "issues" (plural) subcommand

v11_translation_test!(v11_issue_close_fj,
    input: ["gf", "issue", "close", "42"],
    forge: ForgeType::Forgejo,
    expected: ["issue", "close", "42"]
);

// ── ISSUE REOPEN (ISSUE-05): gf issue reopen → gh issue reopen / glab issue reopen / tea issues reopen ──
// fj: UNSUPPORTED (Forgejo CLI has no issue reopen command)

v11_translation_test!(v11_issue_reopen_github,
    input: ["gf", "issue", "reopen", "42"],
    forge: ForgeType::Github,
    expected: ["issue", "reopen", "42"]
);

v11_translation_test!(v11_issue_reopen_glab,
    input: ["gf", "issue", "reopen", "42"],
    forge: ForgeType::Gitlab,
    expected: ["issue", "reopen", "42"]
);

v11_translation_test!(v11_issue_reopen_tea,
    input: ["gf", "issue", "reopen", "42"],
    forge: ForgeType::Gitea,
    expected: ["issues", "reopen", "42"]
);
// tea: uses "issues" (plural) subcommand

unsupported_test!(issue_reopen_fj_unsupported,
    input: ["gf", "issue", "reopen", "42"],
    forge: ForgeType::Forgejo,
    feature_contains: "issue reopen"
);

// ── REPO CLONE (REPO-01): gf repo clone → gh repo clone / glab repo clone / fj repo clone ──

v11_translation_test!(v11_repo_clone_github,
    input: ["gf", "repo", "clone", "owner/repo"],
    forge: ForgeType::Github,
    expected: ["repo", "clone", "owner/repo"]
);

v11_translation_test!(v11_repo_clone_glab,
    input: ["gf", "repo", "clone", "owner/repo"],
    forge: ForgeType::Gitlab,
    expected: ["repo", "clone", "owner/repo"]
);

// tea repo clone: UNSUPPORTED (tea has no repos clone subcommand)

unsupported_test!(repo_clone_tea_unsupported,
    input: ["gf", "repo", "clone", "owner/repo"],
    forge: ForgeType::Gitea,
    feature_contains: "repo clone"
);

v11_translation_test!(v11_repo_clone_fj,
    input: ["gf", "repo", "clone", "owner/repo"],
    forge: ForgeType::Forgejo,
    expected: ["repo", "clone", "owner/repo"]
);

// ═══════════════════════════════════════════════════════════════════════════════
// v1.1 INTEGRATION AUDIT TESTS — verify target forge flags exist in CLI --help
// ═══════════════════════════════════════════════════════════════════════════════
//
// These are NOT ignored — they verify the forge CLIs actually support the
// flags we plan to translate TO. If a forge CLI removes a flag in a future
// version, these tests will catch it.

// --- gh pr list flags ---
audit_test!(audit_v11_gh_pr_list_state,    cli: "gh",   args: ["pr", "list"],    contains: "--state");
audit_test!(audit_v11_gh_pr_list_author,   cli: "gh",   args: ["pr", "list"],    contains: "--author");
audit_test!(audit_v11_gh_pr_list_label,    cli: "gh",   args: ["pr", "list"],    contains: "--label");

// --- glab mr list flags ---
audit_test!(audit_v11_glab_mr_list_closed, cli: "glab", args: ["mr", "list"],    contains: "--closed");
audit_test!(audit_v11_glab_mr_list_author, cli: "glab", args: ["mr", "list"],    contains: "--author");
audit_test!(audit_v11_glab_mr_list_label,  cli: "glab", args: ["mr", "list"],    contains: "--label");

// --- tea pulls list flags ---
audit_test!(audit_v11_tea_pulls_list_state, cli: "tea", args: ["pulls", "list"], contains: "--state");

// --- fj pr search flags ---
audit_test!(audit_v11_fj_pr_search_state,   cli: "fj",  args: ["pr", "search"],  contains: "--state");
audit_test!(audit_v11_fj_pr_search_creator, cli: "fj",  args: ["pr", "search"],  contains: "--creator");
audit_test!(audit_v11_fj_pr_search_labels,  cli: "fj",  args: ["pr", "search"],  contains: "--labels");

// --- gh/glab pr merge flags ---
audit_test!(audit_v11_gh_pr_merge_squash,    cli: "gh",   args: ["pr", "merge"],   contains: "--squash");
audit_test!(audit_v11_glab_mr_merge_squash,  cli: "glab", args: ["mr", "merge"],   contains: "--squash");

// --- tea pulls merge / fj pr merge flags ---
audit_test!(audit_v11_tea_pulls_merge_style, cli: "tea",  args: ["pulls", "merge"], contains: "--style");
audit_test!(audit_v11_fj_pr_merge_method,    cli: "fj",   args: ["pr", "merge"],    contains: "--method");

// --- pr checkout ---
audit_test!(audit_v11_gh_pr_checkout,        cli: "gh",   args: ["pr", "checkout"],    contains: "checkout");
audit_test!(audit_v11_glab_mr_checkout,      cli: "glab", args: ["mr", "checkout"],    contains: "checkout");
audit_test!(audit_v11_tea_pulls_checkout,    cli: "tea",  args: ["pulls", "checkout"], contains: "checkout");

// --- pr review / approve ---
audit_test!(audit_v11_gh_pr_review_approve,  cli: "gh",   args: ["pr", "review"],  contains: "--approve");
audit_test!(audit_v11_glab_mr_approve,       cli: "glab", args: ["mr", "approve"], contains: "approve");

// --- issue list flags ---
audit_test!(audit_v11_gh_issue_list_state,      cli: "gh",   args: ["issue", "list"],    contains: "--state");
audit_test!(audit_v11_glab_issue_list_closed,   cli: "glab", args: ["issue", "list"],    contains: "--closed");
audit_test!(audit_v11_tea_issues_list_state,    cli: "tea",  args: ["issues", "list"],   contains: "--state");
audit_test!(audit_v11_fj_issue_search_state,    cli: "fj",   args: ["issue", "search"],  contains: "--state");

// --- issue create flags ---
audit_test!(audit_v11_gh_issue_create_title,    cli: "gh",   args: ["issue", "create"],  contains: "--title");
audit_test!(audit_v11_glab_issue_create_title,  cli: "glab", args: ["issue", "create"],  contains: "--title");
audit_test!(audit_v11_glab_issue_create_description, cli: "glab", args: ["issue", "create"], contains: "--description");
audit_test!(audit_v11_tea_issues_create_title,  cli: "tea",  args: ["issues", "create"], contains: "--title");

// --- repo clone ---
audit_test!(audit_v11_gh_repo_clone,   cli: "gh",   args: ["repo", "clone"], contains: "clone");
audit_test!(audit_v11_glab_repo_clone, cli: "glab", args: ["repo", "clone"], contains: "clone");
audit_test!(audit_v11_fj_repo_clone,   cli: "fj",   args: ["repo", "clone"], contains: "clone");

// Note: No audit tests for UNSUPPORTED combinations:
// - UNSUPPORTED: tea pulls list --author (no --author on tea pulls list)
// - UNSUPPORTED: tea pulls list --label (no --label on tea pulls list)
// - UNSUPPORTED: tea pulls review (no tea pulls review subcommand)
// - UNSUPPORTED: tea pulls approve (no tea pulls approve subcommand)
// - UNSUPPORTED: fj pr approve (no fj pr approve subcommand)
// - UNSUPPORTED: tea repos clone (tea has no repos clone subcommand)
