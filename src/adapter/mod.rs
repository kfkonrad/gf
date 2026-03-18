// src/adapter/mod.rs
//
// ForgeAdapter translation layer.
// Translates canonical gf subcommand + flags into forge-specific args
// before handing to runner::run().
//
// Module layout:
//   adapter::pr         — PR/MR subcommand + flag translation
//   adapter::repo_auth  — Repo and Auth subcommand + flag translation

mod issue;
mod pr;
mod repo_auth;

use crate::error::GfError;
use crate::forge::ForgeType;
use clap::ArgMatches;

/// Translate clap ArgMatches into a Vec<String> of args for the forge CLI.
///
/// Called after clap parsing in main.rs. The returned Vec is passed directly
/// to runner::run(forge.cli_name(), &translated).
///
/// Dispatches to per-subcommand translators in pr.rs and repo_auth.rs.
pub fn translate(forge: ForgeType, matches: &ArgMatches) -> Result<Vec<String>, GfError> {
    match matches.subcommand() {
        Some(("pr", sub)) => pr::translate_pr(forge, sub),
        Some(("repo", sub)) => repo_auth::translate_repo(forge, sub),
        Some(("auth", sub)) => repo_auth::translate_auth(forge, sub),
        Some(("issue", sub)) => issue::translate_issue(forge, sub),
        Some((other, _)) => Ok(vec![other.to_string()]),
        None => Ok(vec![]),
    }
}
