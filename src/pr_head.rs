//! Native `pr list --head` — finds PRs by the branch containing the changes.
//!
//! Unlike the other subcommands this does not exec the forge CLI: it captures
//! machine-readable output, filters it, prints one line per matching PR and
//! reports whether anything matched, so `gf pr list --head <branch>` can exit 1
//! when the branch has no PR (usable as a check in scripts).
//!
//! Per-forge strategy:
//!   gh:   pr list --head <branch> --json ...                (filtered server-side)
//!   glab: mr list --source-branch <branch> --output json    (filtered server-side, paged)
//!   tea:  pulls list --output json --fields ...             (paged, filtered here)
//!   fj:   pr search --all for PR numbers, then pr view <N>  (filtered here)
//!
//! Every result is filtered by head branch and state here as well, so all
//! forges share the same semantics: "closed" excludes merged PRs, as on gh/glab.

use crate::browse::{build_pr_url, get_remote_url};
use crate::error::GfError;
use crate::forge::{parse_remote_parts, ForgeType};
use crate::runner::capture;
use clap::ArgMatches;
use serde::Deserialize;
use std::collections::HashMap;

/// Upper bound for paged listings; guards against a CLI that ignores `--page`.
const MAX_PAGES: u32 = 1000;
/// Items requested per page from glab (GitLab's maximum) and tea.
const GLAB_PER_PAGE: &str = "100";
const TEA_LIMIT: &str = "50";
/// Number of `fj pr view` calls run concurrently.
const FJ_PARALLELISM: usize = 8;

/// A pull/merge request, normalized across forges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrSummary {
    pub number: String,
    pub title: String,
    pub head: String,
    pub base: String,
    /// "open", "closed" or "merged" (other forge-specific values are kept lowercased)
    pub state: String,
    pub url: String,
}

/// The canonical `pr list` filters.
#[derive(Debug)]
pub struct Filters {
    pub head: String,
    pub state: String,
    pub author: Option<String>,
    pub label: Option<String>,
    pub extra: Vec<String>,
}

impl Filters {
    fn from_matches(matches: &ArgMatches) -> Option<Self> {
        Some(Self {
            head: matches.get_one::<String>("head")?.clone(),
            state: matches
                .get_one::<String>("state")
                .map_or_else(|| "open".to_string(), Clone::clone),
            author: matches.get_one::<String>("author").cloned(),
            label: matches.get_one::<String>("label").cloned(),
            extra: matches
                .get_many::<String>("extra")
                .map(|vals| vals.cloned().collect())
                .unwrap_or_default(),
        })
    }

    /// Whether a PR passes the head and state filters.
    fn accepts(&self, pr: &PrSummary) -> bool {
        head_branch(&pr.head) == self.head && (self.state == "all" || pr.state == self.state)
    }
}

/// Returns the `pr list` matches if this invocation is `gf pr list --head <branch>`.
#[must_use]
pub fn head_lookup(matches: &ArgMatches) -> Option<&ArgMatches> {
    match matches.subcommand() {
        Some(("pr", pr)) => match pr.subcommand() {
            Some(("list", list)) if list.get_one::<String>("head").is_some() => Some(list),
            _ => None,
        },
        _ => None,
    }
}

/// Entry point called from main.rs for `gf pr list --head <branch>`.
///
/// Prints one tab-separated line per matching PR (number, state,
/// `head -> base`, title, URL) and returns whether any PR matched.
///
/// # Errors
///
/// Returns an error if the forge CLI is missing or fails, its output cannot be
/// parsed, or a filter is not supported by the forge.
pub fn run(forge: ForgeType, remote: &str, matches: &ArgMatches) -> Result<bool, GfError> {
    let Some(filters) = Filters::from_matches(matches) else {
        return Ok(false);
    };

    let prs: Vec<PrSummary> = fetch(forge, remote, &filters)?
        .into_iter()
        .filter(|pr| filters.accepts(pr))
        .collect();

    for pr in &prs {
        println!(
            "#{}\t{}\t{} -> {}\t{}\t{}",
            pr.number, pr.state, pr.head, pr.base, pr.title, pr.url
        );
    }
    if prs.is_empty() {
        let state = if filters.state == "all" {
            String::new()
        } else {
            format!("{} ", filters.state)
        };
        eprintln!("no {state}pull request found for branch '{}'", filters.head);
    }

    Ok(!prs.is_empty())
}

fn fetch(forge: ForgeType, remote: &str, filters: &Filters) -> Result<Vec<PrSummary>, GfError> {
    match forge {
        ForgeType::Github => parse_gh(&capture("gh", &gh_args(filters))?),
        ForgeType::Gitlab => fetch_pages("glab", |page| glab_args(filters, page), parse_glab),
        ForgeType::Gitea => {
            reject_tea_filters(filters)?;
            fetch_pages("tea", |page| tea_args(filters, page), parse_tea)
        }
        ForgeType::Forgejo => {
            let numbers = parse_fj_search(&capture("fj", &fj_search_args(filters))?);
            let mut prs = fj_view_all(&numbers)?;
            let url_parts = get_remote_url(remote).and_then(|url| parse_remote_parts(&url));
            if let Ok((host, owner, repo)) = url_parts {
                for pr in &mut prs {
                    pr.url = build_pr_url(forge, &host, &owner, &repo, &pr.number);
                }
            }
            Ok(prs)
        }
    }
}

/// Requests pages 1, 2, … until one comes back empty. Stopping at the first
/// short page would be cheaper, but the server may cap the page size below
/// what was requested.
fn fetch_pages(
    cli: &str,
    args_for: impl Fn(u32) -> Vec<String>,
    parse: impl Fn(&str) -> Result<Vec<PrSummary>, GfError>,
) -> Result<Vec<PrSummary>, GfError> {
    let mut prs = Vec::new();
    for page in 1..=MAX_PAGES {
        let batch = parse(&capture(cli, &args_for(page))?)?;
        if batch.is_empty() {
            return Ok(prs);
        }
        prs.extend(batch);
    }
    Err(GfError::UnexpectedOutput {
        cli: cli.to_string(),
        reason: format!("still returning results after {MAX_PAGES} pages"),
    })
}

/// The branch part of a head ref; tea reports PRs from forks as `owner:branch`.
/// Git branch names cannot contain ':', so splitting on it is unambiguous.
fn head_branch(head: &str) -> &str {
    head.rsplit(':').next().unwrap_or(head)
}

fn unexpected(cli: &str, reason: impl std::fmt::Display) -> GfError {
    GfError::UnexpectedOutput {
        cli: cli.to_string(),
        reason: reason.to_string(),
    }
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(ToString::to_string).collect()
}

// ── GitHub (gh) ─────────────────────────────────────────────────────────────

/// `gh pr list --head <b> [--state] [--author] [--label] --limit 1000 --json ...`
#[must_use]
pub fn gh_args(filters: &Filters) -> Vec<String> {
    let mut args = strings(&[
        "pr",
        "list",
        "--head",
        &filters.head,
        "--state",
        &filters.state,
    ]);
    if let Some(author) = &filters.author {
        args.extend(strings(&["--author", author]));
    }
    if let Some(label) = &filters.label {
        args.extend(strings(&["--label", label]));
    }
    args.extend(strings(&[
        "--limit",
        "1000",
        "--json",
        "number,title,headRefName,baseRefName,state,url",
    ]));
    args.extend(filters.extra.iter().cloned());
    args
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPr {
    number: u64,
    title: String,
    head_ref_name: String,
    base_ref_name: String,
    state: String,
    url: String,
}

/// Parses `gh pr list --json` output.
///
/// # Errors
///
/// Returns [`GfError::UnexpectedOutput`] if the output is not the expected JSON.
pub fn parse_gh(json: &str) -> Result<Vec<PrSummary>, GfError> {
    let prs: Vec<GhPr> = serde_json::from_str(json).map_err(|e| unexpected("gh", e))?;
    Ok(prs
        .into_iter()
        .map(|pr| PrSummary {
            number: pr.number.to_string(),
            title: pr.title,
            head: pr.head_ref_name,
            base: pr.base_ref_name,
            state: pr.state.to_lowercase(),
            url: pr.url,
        })
        .collect())
}

// ── GitLab (glab) ───────────────────────────────────────────────────────────

/// `glab mr list --source-branch <b> [state flag] [--author] [--label] --output json --per-page 100 --page <n>`
#[must_use]
pub fn glab_args(filters: &Filters, page: u32) -> Vec<String> {
    let mut args = strings(&["mr", "list", "--source-branch", &filters.head]);
    // glab uses boolean flags instead of --state <value>; open is its default
    match filters.state.as_str() {
        "open" => {}
        "closed" => args.push("--closed".to_string()),
        "merged" => args.push("--merged".to_string()),
        "all" => args.push("--all".to_string()),
        other => args.extend(strings(&["--state", other])),
    }
    if let Some(author) = &filters.author {
        args.extend(strings(&["--author", author]));
    }
    if let Some(label) = &filters.label {
        args.extend(strings(&["--label", label]));
    }
    args.extend(strings(&[
        "--output",
        "json",
        "--per-page",
        GLAB_PER_PAGE,
        "--page",
        &page.to_string(),
    ]));
    args.extend(filters.extra.iter().cloned());
    args
}

#[derive(Deserialize)]
struct GlabMr {
    iid: u64,
    title: String,
    source_branch: String,
    target_branch: String,
    state: String,
    web_url: String,
}

/// Parses `glab mr list --output json` output.
///
/// # Errors
///
/// Returns [`GfError::UnexpectedOutput`] if the output is not the expected JSON.
pub fn parse_glab(json: &str) -> Result<Vec<PrSummary>, GfError> {
    let mrs: Vec<GlabMr> = serde_json::from_str(json).map_err(|e| unexpected("glab", e))?;
    Ok(mrs
        .into_iter()
        .map(|mr| PrSummary {
            number: mr.iid.to_string(),
            title: mr.title,
            head: mr.source_branch,
            base: mr.target_branch,
            // GitLab calls open MRs "opened"
            state: match mr.state.as_str() {
                "opened" => "open".to_string(),
                other => other.to_lowercase(),
            },
            url: mr.web_url,
        })
        .collect())
}

// ── Gitea (tea) ─────────────────────────────────────────────────────────────

fn reject_tea_filters(filters: &Filters) -> Result<(), GfError> {
    for (flag, value) in [("author", &filters.author), ("label", &filters.label)] {
        if value.is_some() {
            return Err(GfError::UnsupportedFeature {
                feature: format!("pr list --{flag}"),
                forge: "Gitea".to_string(),
                forge_cli: "tea".to_string(),
            });
        }
    }
    Ok(())
}

/// The `--state` to request from tea/fj, which only know open, closed (which
/// includes merged) and all. Merged PRs are picked out of "closed" afterwards.
fn coarse_state(state: &str) -> &str {
    match state {
        "merged" => "closed",
        other => other,
    }
}

/// `tea pulls list --state <s> --output json --fields ... --limit 50 --page <n>`
#[must_use]
pub fn tea_args(filters: &Filters, page: u32) -> Vec<String> {
    let mut args = strings(&[
        "pulls",
        "list",
        "--state",
        coarse_state(&filters.state),
        "--output",
        "json",
        "--fields",
        "index,state,head,base,title,url",
        "--limit",
        TEA_LIMIT,
        "--page",
        &page.to_string(),
    ]);
    args.extend(filters.extra.iter().cloned());
    args
}

/// Parses `tea pulls list --output json` output: an array of objects whose
/// values are all strings. Keys are matched case-insensitively.
///
/// # Errors
///
/// Returns [`GfError::UnexpectedOutput`] if the output is not the expected JSON.
pub fn parse_tea(json: &str) -> Result<Vec<PrSummary>, GfError> {
    let rows: Vec<HashMap<String, String>> =
        serde_json::from_str(json).map_err(|e| unexpected("tea", e))?;
    rows.into_iter()
        .map(|row| {
            let row: HashMap<String, String> = row
                .into_iter()
                .map(|(k, v)| (k.to_lowercase(), v))
                .collect();
            let field = |name: &str| {
                row.get(name)
                    .cloned()
                    .ok_or_else(|| unexpected("tea", format!("missing field '{name}'")))
            };
            Ok(PrSummary {
                number: field("index")?,
                title: field("title")?,
                head: field("head")?,
                base: field("base")?,
                state: field("state")?.to_lowercase(),
                url: field("url")?,
            })
        })
        .collect()
}

// ── Forgejo (fj) ────────────────────────────────────────────────────────────

/// `fj pr search --all --state <s> [--creator] [--labels]`
#[must_use]
pub fn fj_search_args(filters: &Filters) -> Vec<String> {
    let mut args = strings(&[
        "pr",
        "search",
        "--all",
        "--state",
        coarse_state(&filters.state),
    ]);
    if let Some(author) = &filters.author {
        args.extend(strings(&["--creator", author]));
    }
    if let Some(label) = &filters.label {
        args.extend(strings(&["--labels", label]));
    }
    args.extend(filters.extra.iter().cloned());
    args
}

/// Extracts the PR numbers from `fj pr search` output, which lists one PR per
/// line as `#<N>: <title> (by <user>)` below a `<count> pull requests` header.
#[must_use]
pub fn parse_fj_search(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix('#')?.split_once(':'))
        .map(|(number, _)| number)
        .filter(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
        .map(ToString::to_string)
        .collect()
}

/// Parses `fj pr view <N>` output, which starts with:
///
/// ```text
/// <title> #<N>
/// By <user> — <State> — +<added> -<removed>
/// From `<head>` into `<base>`
/// ```
///
/// The URL is left empty; fj does not print it.
///
/// # Errors
///
/// Returns [`GfError::UnexpectedOutput`] if any of these lines is missing.
pub fn parse_fj_view(number: &str, text: &str) -> Result<PrSummary, GfError> {
    let missing = |what: &str| unexpected("fj", format!("no {what} in `fj pr view {number}`"));
    let suffix = format!(" #{number}");

    let title = text
        .lines()
        .find_map(|line| line.strip_suffix(&suffix))
        .ok_or_else(|| missing("title line"))?;
    let state = text
        .lines()
        .find_map(|line| line.strip_prefix("By ")?.split(" — ").nth(1))
        .ok_or_else(|| missing("state"))?;
    // Branch names cannot contain spaces, so "` into `" separates them unambiguously
    let (head, base) = text
        .lines()
        .find_map(|line| {
            line.strip_prefix("From `")?
                .strip_suffix('`')?
                .split_once("` into `")
        })
        .ok_or_else(|| missing("branch line"))?;

    Ok(PrSummary {
        number: number.to_string(),
        title: title.to_string(),
        head: head.to_string(),
        base: base.to_string(),
        state: state.trim().to_lowercase(),
        url: String::new(),
    })
}

/// Runs `fj pr view` for every number, `FJ_PARALLELISM` at a time.
fn fj_view_all(numbers: &[String]) -> Result<Vec<PrSummary>, GfError> {
    let mut prs = Vec::with_capacity(numbers.len());
    for chunk in numbers.chunks(FJ_PARALLELISM) {
        let results: Vec<Result<PrSummary, GfError>> = std::thread::scope(|scope| {
            // Collecting first spawns every thread before the first join; joining
            // lazily would run the views one after another.
            #[allow(clippy::needless_collect)]
            let handles: Vec<_> = chunk
                .iter()
                .map(|number| {
                    scope.spawn(move || {
                        let args = strings(&["pr", "view", number]);
                        parse_fj_view(number, &capture("fj", &args)?)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .unwrap_or_else(|_| Err(unexpected("fj", "pr view worker panicked")))
                })
                .collect()
        });
        for result in results {
            prs.push(result?);
        }
    }
    Ok(prs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filters(head: &str, state: &str) -> Filters {
        Filters {
            head: head.to_string(),
            state: state.to_string(),
            author: None,
            label: None,
            extra: Vec::new(),
        }
    }

    fn pr(head: &str, state: &str) -> PrSummary {
        PrSummary {
            number: "1".to_string(),
            title: "t".to_string(),
            head: head.to_string(),
            base: "main".to_string(),
            state: state.to_string(),
            url: String::new(),
        }
    }

    #[test]
    fn accepts_matches_head_and_state() {
        let f = filters("feature", "open");
        assert!(f.accepts(&pr("feature", "open")));
        assert!(!f.accepts(&pr("feature", "closed")));
        assert!(!f.accepts(&pr("feature-2", "open")));
        assert!(!f.accepts(&pr("other:feature-2", "open")));
    }

    #[test]
    fn accepts_fork_heads_by_branch_name() {
        assert!(filters("feature", "open").accepts(&pr("someone:feature", "open")));
        assert!(filters("feature", "open").accepts(&pr("delete:feature", "open")));
    }

    #[test]
    fn accepts_closed_excludes_merged() {
        let f = filters("feature", "closed");
        assert!(f.accepts(&pr("feature", "closed")));
        assert!(!f.accepts(&pr("feature", "merged")));
        assert!(filters("feature", "merged").accepts(&pr("feature", "merged")));
        assert!(filters("feature", "all").accepts(&pr("feature", "merged")));
    }

    #[test]
    fn gh_args_request_json() {
        let mut f = filters("feature", "open");
        f.author = Some("alice".to_string());
        assert_eq!(
            gh_args(&f),
            strings(&[
                "pr",
                "list",
                "--head",
                "feature",
                "--state",
                "open",
                "--author",
                "alice",
                "--limit",
                "1000",
                "--json",
                "number,title,headRefName,baseRefName,state,url",
            ])
        );
    }

    #[test]
    fn parse_gh_normalizes_state() {
        let json = r#"[{"baseRefName":"main","headRefName":"feature","number":6,"state":"MERGED","title":"Fix it","url":"https://github.com/o/r/pull/6"}]"#;
        let prs = parse_gh(json).expect("valid gh json");
        assert_eq!(
            prs,
            vec![PrSummary {
                number: "6".to_string(),
                title: "Fix it".to_string(),
                head: "feature".to_string(),
                base: "main".to_string(),
                state: "merged".to_string(),
                url: "https://github.com/o/r/pull/6".to_string(),
            }]
        );
        assert_eq!(parse_gh("[]").expect("empty list"), Vec::new());
        assert!(parse_gh("no pull requests").is_err());
    }

    #[test]
    fn glab_args_map_state_to_boolean_flags() {
        let args = glab_args(&filters("feature", "merged"), 2);
        assert_eq!(
            args,
            strings(&[
                "mr",
                "list",
                "--source-branch",
                "feature",
                "--merged",
                "--output",
                "json",
                "--per-page",
                "100",
                "--page",
                "2",
            ])
        );
        assert!(!glab_args(&filters("feature", "open"), 1).contains(&"--state".to_string()));
        assert!(glab_args(&filters("feature", "all"), 1).contains(&"--all".to_string()));
    }

    #[test]
    fn parse_glab_maps_opened_to_open() {
        let json = r#"[{"iid":3,"title":"MR","source_branch":"feature","target_branch":"main","state":"opened","web_url":"https://gitlab.com/o/r/-/merge_requests/3","extra":1}]"#;
        let prs = parse_glab(json).expect("valid glab json");
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, "3");
        assert_eq!(prs[0].state, "open");
        assert_eq!(prs[0].head, "feature");
        assert_eq!(prs[0].base, "main");
    }

    #[test]
    fn tea_args_request_closed_for_merged() {
        let args = tea_args(&filters("feature", "merged"), 3);
        assert_eq!(
            args,
            strings(&[
                "pulls",
                "list",
                "--state",
                "closed",
                "--output",
                "json",
                "--fields",
                "index,state,head,base,title,url",
                "--limit",
                "50",
                "--page",
                "3",
            ])
        );
    }

    #[test]
    fn tea_rejects_author_and_label() {
        let mut f = filters("feature", "open");
        f.label = Some("bug".to_string());
        assert!(matches!(
            reject_tea_filters(&f),
            Err(GfError::UnsupportedFeature { ref feature, .. }) if feature == "pr list --label"
        ));
        assert!(reject_tea_filters(&filters("feature", "open")).is_ok());
    }

    #[test]
    fn parse_tea_reads_string_fields() {
        let json = r#"[{"index":"7","state":"merged","head":"someone:feature","base":"main","title":"PR","url":"https://gitea.com/o/r/pulls/7"}]"#;
        let prs = parse_tea(json).expect("valid tea json");
        assert_eq!(prs[0].number, "7");
        assert_eq!(prs[0].state, "merged");
        assert_eq!(prs[0].head, "someone:feature");
        assert!(parse_tea(r#"[{"index":"7"}]"#).is_err());
        assert_eq!(parse_tea("[]").expect("empty list"), Vec::new());
    }

    #[test]
    fn parse_tea_keys_are_case_insensitive() {
        let json = r#"[{"Index":"7","State":"open","Head":"feature","Base":"main","Title":"PR","URL":"u"}]"#;
        assert_eq!(parse_tea(json).expect("valid tea json")[0].url, "u");
    }

    #[test]
    fn fj_search_args_use_all_pages() {
        let mut f = filters("feature", "merged");
        f.author = Some("alice".to_string());
        assert_eq!(
            fj_search_args(&f),
            strings(&[
                "pr",
                "search",
                "--all",
                "--state",
                "closed",
                "--creator",
                "alice"
            ])
        );
    }

    #[test]
    fn parse_fj_search_extracts_numbers() {
        let text = "3 pull requests\n#84: service accounts: ssh keys (by kevin)\n#9: fix #12: thing (by s3)\n#x: not a pr\n";
        assert_eq!(parse_fj_search(text), strings(&["84", "9"]));
        assert_eq!(parse_fj_search("0 pull requests\n"), Vec::<String>::new());
    }

    #[test]
    fn parse_fj_view_reads_title_state_and_branches() {
        let text = "service accounts: ssh-keys-sa #84\nBy kevin.konrad — Open — +21 -0\nFrom `kfkonrad/2026-10-15-tqmvmwwuylkx` into `main`\n\n> body\n";
        let pr = parse_fj_view("84", text).expect("valid fj view");
        assert_eq!(pr.title, "service accounts: ssh-keys-sa");
        assert_eq!(pr.state, "open");
        assert_eq!(pr.head, "kfkonrad/2026-10-15-tqmvmwwuylkx");
        assert_eq!(pr.base, "main");
    }

    #[test]
    fn parse_fj_view_handles_merged_and_title_with_hash() {
        let text =
            "fix #12 for real #82\nBy simon — Merged — +11 -0\nFrom `feat/x` into `release/1`\n";
        let pr = parse_fj_view("82", text).expect("valid fj view");
        assert_eq!(pr.title, "fix #12 for real");
        assert_eq!(pr.state, "merged");
        assert_eq!(pr.base, "release/1");
    }

    #[test]
    fn parse_fj_view_rejects_unknown_format() {
        assert!(parse_fj_view("1", "something else entirely").is_err());
    }
}
