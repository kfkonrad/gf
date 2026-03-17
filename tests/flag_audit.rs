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
            let result = gf::adapter::translate($forge, &matches);
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
