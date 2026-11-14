//! How fresh each book page is (`cargo xtask book-fresh [--json OUT]`).
//!
//! Every page `docs/SUMMARY.md` lists may start with one freshness record, an
//! HTML comment GitHub and mdBook both hide:
//!
//! ```text
//! <!-- freshness: sources=crates/foo/src/**,xtask/src/book_stage.rs checked=<40-hex commit> -->
//! ```
//!
//! - `sources=` is a comma-separated list of repo-root paths or git globs
//!   (`*` stays in one path segment, `**` crosses segments) naming what the
//!   page describes, or `none` when it describes nothing in the repository.
//! - `checked=` is the full hash of the commit the page was read against. It
//!   is left out when nobody has read the page against its sources yet.
//!
//! For each page this counts the commits after `checked` that touch its
//! sources, using `git log` with every `GIT_*` variable removed from its
//! environment. The report lists the stale, unchecked, unrecorded and broken
//! pages. It is advisory (D76): it exits 0 whatever freshness it finds, and a
//! page it cannot read or assess is listed as broken. Only a bad argument, an
//! unreadable SUMMARY, a missing HEAD (no git history) or an
//! unwritable `--json` file exits 1.
//! `book-stage` uses the same assessment for the banner on each staged page.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::book_stage::{summary_targets, Book};
use crate::docs_links::normalize;

const OPEN: &str = "<!-- freshness:";
const CLOSE: &str = "-->";

/// One page's freshness record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Repo-root paths or globs; empty for `sources=none`.
    pub sources: Vec<String>,
    /// The full commit hash the page was read against, if it was.
    pub checked: Option<String>,
}

/// Whether `line` is meant as a freshness record (it may still be malformed).
pub fn is_record_line(line: &str) -> bool {
    line.trim_start_matches('\u{feff}')
        .trim_start()
        .starts_with(OPEN)
}

/// The record on the first line of `text`: `Ok(None)` when the page has none,
/// `Err` when the first line is a freshness comment that does not parse.
pub fn parse_record(text: &str) -> Result<Option<Record>, String> {
    let first = text.lines().next().unwrap_or("");
    if !is_record_line(first) {
        return Ok(None);
    }
    let body = first
        .trim_start_matches('\u{feff}')
        .trim()
        .strip_prefix(OPEN)
        .and_then(|b| b.strip_suffix(CLOSE))
        .ok_or_else(|| format!("the record does not end with {CLOSE} on its line"))?;
    let mut sources: Option<Vec<String>> = None;
    let mut checked = None;
    for token in body.split_whitespace() {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| format!("`{token}` is not key=value"))?;
        match key {
            "sources" if sources.is_none() => {
                sources = Some(if value == "none" {
                    Vec::new()
                } else {
                    let list: Vec<String> = value.split(',').map(str::to_string).collect();
                    if list.iter().any(|s| s.is_empty() || s == "none") {
                        return Err(format!(
                            "sources `{value}` has an empty entry or mixes in `none`"
                        ));
                    }
                    if let Some(bad) = list
                        .iter()
                        .find(|s| s.starts_with('/') || s.split('/').any(|p| p == ".."))
                    {
                        return Err(format!("source `{bad}` is not a repo-root path"));
                    }
                    list
                });
            }
            "checked" if checked.is_none() => {
                if value.len() != 40
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                {
                    return Err(format!(
                        "checked `{value}` is not a full 40-character lowercase commit hash"
                    ));
                }
                checked = Some(value.to_string());
            }
            "sources" | "checked" => return Err(format!("`{key}` is given twice")),
            other => return Err(format!("unknown key `{other}`")),
        }
    }
    let sources = sources
        .ok_or("the record has no sources= (write sources=none if the page describes no files)")?;
    Ok(Some(Record { sources, checked }))
}

/// The page text with its first-line record removed, if it has one.
pub fn strip_record(text: &str) -> &str {
    let first_end = text.find('\n').map_or(text.len(), |i| i + 1);
    if is_record_line(&text[..first_end]) {
        &text[first_end..]
    } else {
        text
    }
}

/// What `book-fresh` needs from git; [`GitCli`] runs the real thing.
pub trait Git {
    /// Whether any tracked file at HEAD matches `source`.
    fn matches_any(&self, source: &str) -> Result<bool, String>;
    /// Whether `commit` is a commit that HEAD contains.
    fn is_ancestor_of_head(&self, commit: &str) -> Result<bool, String>;
    /// The commits after `commit` up to HEAD that touch any of `sources` other than
    /// the page file `page` itself (editing a page is not a change to its sources),
    /// newest first.
    fn commits_since(
        &self,
        commit: &str,
        sources: &[String],
        page: &str,
    ) -> Result<Vec<String>, String>;
    /// The commit date of `commit`, as YYYY-MM-DD.
    fn commit_date(&self, commit: &str) -> Result<String, String>;
}

/// `git` in the repository root, with every `GIT_*` variable removed so a
/// caller's environment (a hook, a worktree override) cannot redirect it.
pub struct GitCli {
    pub root: PathBuf,
}

impl GitCli {
    /// The `git` command for `args`, removing each `GIT_*` name in `env_keys`
    /// (the caller's environment in `run`; a fixed list in the test).
    fn command<I: IntoIterator<Item = std::ffi::OsString>>(
        &self,
        args: &[&str],
        env_keys: I,
    ) -> Command {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.root).args(args);
        for key in env_keys {
            if key.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(&key);
            }
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Result<std::process::Output, String> {
        self.command(args, std::env::vars_os().map(|(k, _)| k))
            .output()
            .map_err(|e| format!("run git: {e}"))
    }

    fn ok(&self, args: &[&str]) -> Result<String, String> {
        let out = self.run(args)?;
        if !out.status.success() {
            return Err(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

fn pathspec(source: &str) -> String {
    format!(":(glob){source}")
}

impl Git for GitCli {
    fn matches_any(&self, source: &str) -> Result<bool, String> {
        Ok(!self
            .ok(&["ls-files", "--", &pathspec(source)])?
            .trim()
            .is_empty())
    }

    fn is_ancestor_of_head(&self, commit: &str) -> Result<bool, String> {
        let spec = format!("{commit}^{{commit}}");
        if !self.run(&["cat-file", "-e", &spec])?.status.success() {
            return Ok(false);
        }
        let out = self.run(&["merge-base", "--is-ancestor", commit, "HEAD"])?;
        match out.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(format!(
                "git merge-base --is-ancestor {commit} HEAD failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
        }
    }

    fn commits_since(
        &self,
        commit: &str,
        sources: &[String],
        page: &str,
    ) -> Result<Vec<String>, String> {
        let range = format!("{commit}..HEAD");
        let mut specs: Vec<String> = sources.iter().map(|s| pathspec(s)).collect();
        specs.push(format!(":(exclude,literal){page}"));
        // --no-show-signature: a user's log.showSignature would add lines to %H.
        let mut args = vec![
            "log",
            "--no-show-signature",
            "--format=%H",
            range.as_str(),
            "--",
        ];
        args.extend(specs.iter().map(String::as_str));
        Ok(self
            .ok(&args)?
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    fn commit_date(&self, commit: &str) -> Result<String, String> {
        Ok(self
            .ok(&["show", "-s", "--no-show-signature", "--format=%cs", commit])?
            .trim()
            .to_string())
    }
}

/// A page's freshness, as `book-fresh` reports it and `book-stage` shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Checked, and no later commit touches its sources.
    Fresh,
    /// Checked, and `later` commits since then touch its sources.
    Stale,
    /// A record with sources, but no `checked` yet.
    Unchecked,
    /// `sources=none`: the page names nothing to go stale against.
    NoSources,
    /// The page has no record line.
    Unrecorded,
    /// A source matches no tracked file, or `checked` is not a commit HEAD contains.
    Broken,
    /// The first line is a freshness comment that does not parse.
    Malformed,
}

impl Status {
    pub fn name(&self) -> &'static str {
        match self {
            Status::Fresh => "fresh",
            Status::Stale => "stale",
            Status::Unchecked => "unchecked",
            Status::NoSources => "no-sources",
            Status::Unrecorded => "unrecorded",
            Status::Broken => "broken",
            Status::Malformed => "malformed",
        }
    }
}

/// One page's assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub page: String,
    pub status: Status,
    pub record: Option<Record>,
    /// Commits after `checked` touching the sources, newest first.
    pub later: Vec<String>,
    /// The date of the `checked` commit, when it is a commit HEAD contains.
    pub checked_date: Option<String>,
    /// Sources that match no tracked file.
    pub missing_sources: Vec<String>,
    /// Why the page is broken or malformed.
    pub problem: Option<String>,
}

/// Assess one page from its text.
pub fn assess(page: &str, text: &str, git: &dyn Git) -> Result<Assessment, String> {
    let mut a = Assessment {
        page: page.to_string(),
        status: Status::Unrecorded,
        record: None,
        later: Vec::new(),
        checked_date: None,
        missing_sources: Vec::new(),
        problem: None,
    };
    let record = match parse_record(text) {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(a),
        Err(e) => {
            a.status = Status::Malformed;
            a.problem = Some(e);
            return Ok(a);
        }
    };
    a.record = Some(record.clone());
    if record.sources.is_empty() {
        a.status = Status::NoSources;
        return Ok(a);
    }
    for s in &record.sources {
        if !git.matches_any(s)? {
            a.missing_sources.push(s.clone());
        }
    }
    let mut problems = Vec::new();
    if !a.missing_sources.is_empty() {
        problems.push(format!(
            "sources match no tracked file: {}",
            a.missing_sources.join(", ")
        ));
    }
    a.status = match &record.checked {
        None => Status::Unchecked,
        Some(c) if !git.is_ancestor_of_head(c)? => {
            problems.push(format!("checked {c} is not a commit HEAD contains"));
            Status::Broken
        }
        Some(c) => {
            a.checked_date = Some(git.commit_date(c)?);
            a.later = git.commits_since(c, &record.sources, page)?;
            if a.later.is_empty() {
                Status::Fresh
            } else {
                Status::Stale
            }
        }
    };
    if !problems.is_empty() {
        a.status = Status::Broken;
        a.problem = Some(problems.join("; "));
    }
    Ok(a)
}

/// `assess`, with a page that cannot be read or a git call that fails
/// reported as that page being broken, so one bad page never stops the report.
pub fn assess_or_broken(page: &str, text: Result<String, String>, git: &dyn Git) -> Assessment {
    let failed = |problem: String| Assessment {
        page: page.to_string(),
        status: Status::Broken,
        record: None,
        later: Vec::new(),
        checked_date: None,
        missing_sources: Vec::new(),
        problem: Some(problem),
    };
    match text {
        Err(e) => failed(e),
        Ok(t) => assess(page, &t, git).unwrap_or_else(|e| Assessment {
            record: parse_record(&t).ok().flatten(),
            ..failed(e)
        }),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The one-line banner `book-stage` puts at the top of a staged page, with a
/// blank line after it so the page's own first block starts fresh.
pub fn banner(a: &Assessment) -> String {
    let checked = a.record.as_ref().and_then(|r| r.checked.as_deref());
    let at = |c: &str| {
        let date = a
            .checked_date
            .as_deref()
            .map(|d| format!(" ({d})"))
            .unwrap_or_default();
        format!(
            "Checked against its sources at <a href=\"{}/commit/{c}\"><code>{}</code></a>{date}",
            crate::book_stage::REPO_URL,
            &c[..9]
        )
    };
    let text = match (&a.status, checked) {
        (Status::Fresh, Some(c)) => format!("{}; no later commit changes them.", at(c)),
        (Status::Stale, Some(c)) => {
            let n = a.later.len();
            let (s, verb) = if n == 1 {
                ("", "changes")
            } else {
                ("s", "change")
            };
            format!("{}; {n} later commit{s} {verb} its sources.", at(c))
        }
        (Status::Unchecked, _) => "Not yet checked against its sources.".to_string(),
        (Status::Unrecorded, _) => {
            "Not yet checked: this page has no freshness record.".to_string()
        }
        (Status::NoSources, _) => "This page names no sources to check against.".to_string(),
        (_, _) => format!(
            "Not yet checked: its freshness record needs repair ({}).",
            escape_html(a.problem.as_deref().unwrap_or("unknown problem"))
        ),
    };
    format!("<p class=\"page-freshness\"><small>{text}</small></p>\n\n")
}

/// The book's pages in SUMMARY order, without SUMMARY.md itself.
pub fn book_pages(root: &Path) -> Result<Vec<String>, String> {
    let summary_path = root.join("docs/SUMMARY.md");
    let summary = fs::read_to_string(&summary_path)
        .map_err(|e| format!("read {}: {e}", summary_path.display()))?;
    // Validates the SUMMARY the same way book-stage does.
    Book::from_summary(&summary)?;
    let mut pages: Vec<String> = Vec::new();
    for target in summary_targets(&summary) {
        let repo = normalize(&format!("docs/{target}"))
            .map_err(|p| format!("SUMMARY entry {target} climbs above the repository: {p}"))?;
        if !pages.contains(&repo) {
            pages.push(repo);
        }
    }
    Ok(pages)
}

/// Counts by status name, every status present (zero when unused).
pub fn summary_counts(all: &[Assessment]) -> BTreeMap<&'static str, usize> {
    let mut counts: BTreeMap<&'static str, usize> = [
        Status::Fresh,
        Status::Stale,
        Status::Unchecked,
        Status::NoSources,
        Status::Unrecorded,
        Status::Broken,
        Status::Malformed,
    ]
    .iter()
    .map(|s| (s.name(), 0))
    .collect();
    for a in all {
        *counts.entry(a.status.name()).or_default() += 1;
    }
    counts
}

fn to_json(head: &str, all: &[Assessment]) -> serde_json::Value {
    let pages: Vec<serde_json::Value> = all
        .iter()
        .map(|a| {
            serde_json::json!({
                "page": a.page,
                "status": a.status.name(),
                "sources": a.record.as_ref().map(|r| r.sources.clone()),
                "checked": a.record.as_ref().and_then(|r| r.checked.clone()),
                "checked_date": a.checked_date,
                "later_commits": a.later,
                "later_count": a.later.len(),
                "missing_sources": a.missing_sources,
                "problem": a.problem,
            })
        })
        .collect();
    serde_json::json!({
        "schema": "uor-r4.book-fresh/1",
        "head": head,
        "summary": summary_counts(all),
        "pages": pages,
    })
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut json_out: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json_out = Some(PathBuf::from(it.next().ok_or("--json needs a file")?)),
            other => return Err(format!("book-fresh: unknown argument {other}")),
        }
    }
    let git = GitCli {
        root: root.to_path_buf(),
    };
    let head = git.ok(&["rev-parse", "HEAD"])?.trim().to_string();
    let mut all = Vec::new();
    for page in book_pages(root)? {
        let text = fs::read_to_string(root.join(&page)).map_err(|e| format!("read {page}: {e}"));
        all.push(assess_or_broken(&page, text, &git));
    }
    let counts = summary_counts(&all);
    let line: Vec<String> = counts.iter().map(|(k, v)| format!("{k} {v}")).collect();
    println!(
        "book-fresh: {} pages at {}: {}",
        all.len(),
        &head[..head.len().min(12)],
        line.join(", ")
    );
    for status in [
        Status::Broken,
        Status::Malformed,
        Status::Stale,
        Status::Unchecked,
        Status::Unrecorded,
    ] {
        let listed: Vec<&Assessment> = all.iter().filter(|a| a.status == status).collect();
        if listed.is_empty() {
            continue;
        }
        println!("\n{} ({}):", status.name(), listed.len());
        for a in listed {
            match (&a.status, &a.problem) {
                (Status::Stale, _) => println!(
                    "  {} ({} later commit{})",
                    a.page,
                    a.later.len(),
                    if a.later.len() == 1 { "" } else { "s" }
                ),
                (_, Some(p)) => println!("  {}: {p}", a.page),
                _ => println!("  {}", a.page),
            }
        }
    }
    println!("\nadvisory (D76): book-fresh reports and never fails a build");
    if let Some(path) = json_out {
        let path = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        let text =
            serde_json::to_string_pretty(&to_json(&head, &all)).map_err(|e| e.to_string())?;
        fs::write(&path, text + "\n").map_err(|e| format!("write {}: {e}", path.display()))?;
        println!("book-fresh: wrote {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

    /// A git with a fixed set of tracked sources, known commits and commits per source.
    struct FakeGit {
        tracked: Vec<&'static str>,
        known: Vec<&'static str>,
        later: Vec<(&'static str, &'static str)>,
        calls: RefCell<Vec<String>>,
    }

    impl Git for FakeGit {
        fn matches_any(&self, source: &str) -> Result<bool, String> {
            Ok(self.tracked.contains(&source))
        }
        fn is_ancestor_of_head(&self, commit: &str) -> Result<bool, String> {
            Ok(self.known.contains(&commit))
        }
        fn commits_since(
            &self,
            commit: &str,
            sources: &[String],
            _page: &str,
        ) -> Result<Vec<String>, String> {
            self.calls
                .borrow_mut()
                .push(format!("{commit} {}", sources.join(",")));
            Ok(self
                .later
                .iter()
                .filter(|(s, _)| sources.iter().any(|x| x == s))
                .map(|(_, c)| c.to_string())
                .collect())
        }
        fn commit_date(&self, _: &str) -> Result<String, String> {
            Ok("2026-11-14".into())
        }
    }

    fn fake() -> FakeGit {
        FakeGit {
            tracked: vec!["a/**", "b.rs"],
            known: vec![SHA],
            later: vec![("b.rs", "c1"), ("b.rs", "c2")],
            calls: RefCell::new(Vec::new()),
        }
    }

    fn page(record: &str) -> String {
        format!("{record}\n# Title\n\nBody.\n")
    }

    #[test]
    fn parses_sources_and_checked() {
        let r = parse_record(&page(&format!(
            "<!-- freshness: sources=a/**,b.rs checked={SHA} -->"
        )))
        .unwrap()
        .unwrap();
        assert_eq!(r.sources, vec!["a/**", "b.rs"]);
        assert_eq!(r.checked.as_deref(), Some(SHA));
        let r = parse_record(&page("<!-- freshness: sources=none -->"))
            .unwrap()
            .unwrap();
        assert!(r.sources.is_empty() && r.checked.is_none());
        assert_eq!(
            parse_record("# Title\n<!-- freshness: sources=a -->\n").unwrap(),
            None
        );
    }

    #[test]
    fn malformed_records_are_errors() {
        for bad in [
            "<!-- freshness: checked=0123 sources=a -->".to_string(),
            format!(
                "<!-- freshness: sources=a checked={} -->",
                SHA.to_uppercase()
            ),
            "<!-- freshness: sources=a,,b -->".into(),
            "<!-- freshness: sources=../x -->".into(),
            "<!-- freshness: sources=/abs -->".into(),
            "<!-- freshness: sources=a sources=b -->".into(),
            "<!-- freshness: sources=a date=today -->".into(),
            format!("<!-- freshness: checked={SHA} -->"),
            "<!-- freshness: sources=a".into(),
        ] {
            assert!(parse_record(&page(&bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn strip_removes_only_a_first_line_record() {
        let text = page("<!-- freshness: sources=none -->");
        assert_eq!(strip_record(&text), "# Title\n\nBody.\n");
        assert_eq!(strip_record("# T\n"), "# T\n");
    }

    #[test]
    fn fresh_stale_and_unchecked() {
        let g = fake();
        let fresh = assess(
            "p.md",
            &page(&format!("<!-- freshness: sources=a/** checked={SHA} -->")),
            &g,
        )
        .unwrap();
        assert_eq!(fresh.status, Status::Fresh);
        assert_eq!(fresh.checked_date.as_deref(), Some("2026-11-14"));
        let stale = assess(
            "p.md",
            &page(&format!(
                "<!-- freshness: sources=a/**,b.rs checked={SHA} -->"
            )),
            &g,
        )
        .unwrap();
        assert_eq!(stale.status, Status::Stale);
        assert_eq!(stale.later, vec!["c1", "c2"]);
        // The count reads every source of the record in one query from `checked`.
        assert_eq!(
            g.calls.borrow().last().unwrap(),
            &format!("{SHA} a/**,b.rs")
        );
        let unchecked = assess("p.md", &page("<!-- freshness: sources=b.rs -->"), &g).unwrap();
        assert_eq!(unchecked.status, Status::Unchecked);
        assert!(unchecked.later.is_empty());
    }

    #[test]
    fn unrecorded_no_sources_and_malformed() {
        let g = fake();
        assert_eq!(
            assess("p.md", "# T\n", &g).unwrap().status,
            Status::Unrecorded
        );
        assert_eq!(
            assess("p.md", &page("<!-- freshness: sources=none -->"), &g)
                .unwrap()
                .status,
            Status::NoSources
        );
        let m = assess("p.md", &page("<!-- freshness: sources= -->"), &g).unwrap();
        assert_eq!(m.status, Status::Malformed);
        assert!(m.problem.is_some());
    }

    #[test]
    fn missing_sources_and_unknown_commits_are_broken() {
        let g = fake();
        let a = assess(
            "p.md",
            &page(&format!(
                "<!-- freshness: sources=a/**,gone/** checked={SHA} -->"
            )),
            &g,
        )
        .unwrap();
        assert_eq!(a.status, Status::Broken);
        assert_eq!(a.missing_sources, vec!["gone/**"]);
        let other = "fedcba9876543210fedcba9876543210fedcba98";
        let b = assess(
            "p.md",
            &page(&format!("<!-- freshness: sources=a/** checked={other} -->")),
            &g,
        )
        .unwrap();
        assert_eq!(b.status, Status::Broken);
        assert!(b.problem.unwrap().contains(other));
        // An unchecked page with a missing source is broken too.
        let c = assess("p.md", &page("<!-- freshness: sources=gone/** -->"), &g).unwrap();
        assert_eq!(c.status, Status::Broken);
    }

    #[test]
    fn banner_says_what_the_record_says() {
        let g = fake();
        let b = |rec: &str| banner(&assess("p.md", &page(rec), &g).unwrap());
        let fresh = b(&format!("<!-- freshness: sources=a/** checked={SHA} -->"));
        assert!(fresh.starts_with("<p class=\"page-freshness\"><small>Checked against its sources at <a href=\"https://github.com/UOR-Foundation/uor-r4/commit/0123456789abcdef"), "{fresh}");
        assert!(
            fresh
                .contains("<code>012345678</code></a> (2026-11-14); no later commit changes them."),
            "{fresh}"
        );
        assert!(fresh.ends_with("</small></p>\n\n"));
        let stale = b(&format!("<!-- freshness: sources=b.rs checked={SHA} -->"));
        assert!(
            stale.contains("; 2 later commits change its sources."),
            "{stale}"
        );
        assert!(
            b("<!-- freshness: sources=b.rs -->").contains("Not yet checked against its sources.")
        );
        assert!(banner(&assess("p.md", "# T\n", &g).unwrap())
            .contains("Not yet checked: this page has no freshness record."));
        assert!(b("<!-- freshness: sources=none -->").contains("names no sources"));
        let broken = b("<!-- freshness: sources=gone/** -->");
        assert!(
            broken.contains("needs repair (sources match no tracked file: gone/**)"),
            "{broken}"
        );
        let one = FakeGit {
            later: vec![("b.rs", "c1")],
            ..fake()
        };
        let a = assess(
            "p.md",
            &page(&format!("<!-- freshness: sources=b.rs checked={SHA} -->")),
            &one,
        )
        .unwrap();
        assert!(banner(&a).contains("; 1 later commit changes its sources."));
    }

    #[test]
    fn counts_cover_every_status() {
        let g = fake();
        let all = vec![
            assess("a.md", "# T\n", &g).unwrap(),
            assess("b.md", &page("<!-- freshness: sources=b.rs -->"), &g).unwrap(),
            assess("c.md", &page("<!-- freshness: sources=b.rs -->"), &g).unwrap(),
        ];
        let c = summary_counts(&all);
        assert_eq!(c.len(), 7);
        assert_eq!((c["unrecorded"], c["unchecked"], c["fresh"]), (1, 2, 0));
    }

    /// A real repository in a temporary directory, with every `GIT_*` variable
    /// removed (the same environment `GitCli` gives git).
    #[test]
    fn git_cli_counts_commits_touching_sources() {
        let dir = std::env::temp_dir().join(format!("book-fresh-git-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src/deep")).unwrap();
        let dir = dir.canonicalize().unwrap();
        assert!(dir.starts_with(std::env::temp_dir().canonicalize().unwrap()));
        let git = GitCli { root: dir.clone() };
        let sh = |args: &[&str]| git.ok(args).unwrap();
        sh(&["init", "-q"]);
        sh(&["config", "user.email", "t@example.invalid"]);
        sh(&["config", "user.name", "t"]);
        sh(&["config", "commit.gpgsign", "false"]);
        // The test must write into its own repository, never an enclosing one.
        assert_eq!(
            Path::new(sh(&["rev-parse", "--show-toplevel"]).trim())
                .canonicalize()
                .unwrap(),
            dir
        );
        let commit = |file: &str, body: &str| {
            fs::write(dir.join(file), body).unwrap();
            sh(&["add", "-A"]);
            sh(&["commit", "-q", "-m", file]);
            sh(&["rev-parse", "HEAD"]).trim().to_string()
        };
        let base = commit("src/deep/a.rs", "1");
        commit("other.txt", "x");
        let c2 = commit("src/deep/a.rs", "2");
        let c3 = commit("src/b.rs", "3");
        // An edit of the page itself is not a change to its sources, even when a glob covers it.
        commit("src/page.md", "the page");
        let p = "src/page.md";
        assert!(git.matches_any("src/**").unwrap());
        assert!(git.matches_any("src/*.rs").unwrap());
        assert!(!git.matches_any("nope/**").unwrap());
        assert!(git.is_ancestor_of_head(&base).unwrap());
        assert!(!git
            .is_ancestor_of_head("fedcba9876543210fedcba9876543210fedcba98")
            .unwrap());
        assert_eq!(
            git.commits_since(&base, &["src/**".into()], p).unwrap(),
            vec![c3.clone(), c2.clone()]
        );
        // `*` stays in one segment: src/*.rs is src/b.rs only.
        assert_eq!(
            git.commits_since(&base, &["src/*.rs".into()], p).unwrap(),
            vec![c3]
        );
        assert!(
            git.commits_since(&base, &["other.txt".into()], p)
                .unwrap()
                .len()
                == 1
        );
        assert_eq!(git.commit_date(&base).unwrap().len(), 10);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn git_command_removes_every_git_variable() {
        let git = GitCli {
            root: PathBuf::from("/repo"),
        };
        let keys = ["GIT_DIR", "GIT_WORK_TREE", "HOME", "XGIT_DIR"].map(std::ffi::OsString::from);
        let cmd = git.command(&["status"], keys);
        let removed: Vec<String> = cmd
            .get_envs()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k.to_string_lossy().into_owned())
            .collect();
        assert_eq!(removed, vec!["GIT_DIR", "GIT_WORK_TREE"]);
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, vec!["-C", "/repo", "status"]);
    }

    /// A git whose every call fails, as with a damaged repository.
    struct BrokenGit;

    impl Git for BrokenGit {
        fn matches_any(&self, _: &str) -> Result<bool, String> {
            Err("git ls-files failed: not a git repository".into())
        }
        fn is_ancestor_of_head(&self, _: &str) -> Result<bool, String> {
            Err("no".into())
        }
        fn commits_since(&self, _: &str, _: &[String], _: &str) -> Result<Vec<String>, String> {
            Err("no".into())
        }
        fn commit_date(&self, _: &str) -> Result<String, String> {
            Err("no".into())
        }
    }

    #[test]
    fn unreadable_pages_and_git_errors_are_broken() {
        let a = assess_or_broken("docs/x.md", Err("read docs/x.md: gone".into()), &fake());
        assert_eq!(a.status, Status::Broken);
        assert_eq!(a.problem.as_deref(), Some("read docs/x.md: gone"));
        let a = assess_or_broken(
            "docs/y.md",
            Ok(page("<!-- freshness: sources=a/** -->")),
            &BrokenGit,
        );
        assert_eq!(a.status, Status::Broken);
        assert_eq!(a.record.as_ref().map(|r| r.sources.len()), Some(1));
        assert!(a.problem.unwrap().contains("not a git repository"));
        // A page that needs no git call is still assessed normally.
        let a = assess_or_broken("docs/z.md", Ok(page("# Title")), &BrokenGit);
        assert_eq!(a.status, Status::Unrecorded);
    }

    #[test]
    fn json_has_schema_every_count_and_null_sources_when_unrecorded() {
        let git = fake();
        let all = vec![
            assess("docs/a.md", &page("# Only a title"), &git).unwrap(),
            assess(
                "docs/b.md",
                &page(&format!("<!-- freshness: sources=b.rs checked={SHA} -->")),
                &git,
            )
            .unwrap(),
        ];
        let v = to_json("head1", &all);
        assert_eq!(v["schema"], "uor-r4.book-fresh/1");
        assert_eq!(v["head"], "head1");
        let summary = v["summary"].as_object().unwrap();
        assert_eq!(summary.len(), 7);
        assert_eq!(summary["unrecorded"], 1);
        assert_eq!(summary["stale"], 1);
        assert_eq!(summary["fresh"], 0);
        assert!(v["pages"][0]["sources"].is_null());
        assert_eq!(v["pages"][1]["later_count"], 2);
        assert_eq!(v["pages"][1]["checked"], SHA);
    }
}
