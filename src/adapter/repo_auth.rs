// src/adapter/repo_auth.rs — Repo and Auth translation (implemented in Plan 03)
use crate::forge::ForgeType;
use clap::ArgMatches;

pub fn translate_repo(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    // TODO(Plan 03): implement full repo translation
    let _ = (forge, matches);
    vec![]
}

pub fn translate_auth(forge: ForgeType, matches: &ArgMatches) -> Vec<String> {
    // TODO(Plan 03): implement full auth translation
    let _ = (forge, matches);
    vec![]
}
