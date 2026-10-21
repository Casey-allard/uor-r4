//! `uor-harness loop queue --pr <n>`: put a loop PR in the merge queue only when every
//! `loop pr-check` rule passes at the commit GitHub reports as the head, and nothing the owner or
//! the PM said came after the approval.
//!
//! The head is fetched from `pull/<n>/head` and must equal GitHub's `headRefOid`; the rules run
//! against base `origin/main` through [`crate::loopprcheck::evaluate`], unchanged. Then the
//! after-approval rule reads every comment after the last counted record (an APPROVE, found by
//! [`crate::loopreview::last_counted_approve`]): records are the gate's business, `[bot]` authors
//! and `[<lab>]` comments from a registered lab that is not the PM are ignored, and anything else
//! refuses, because every lab posts from one GitHub account (D55) and an untagged comment is the
//! owner's as far as the tool can tell. The only write is `gh pr merge <n> --match-head-commit
//! <sha>`, through [`QueueGh`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::loopprcheck::{Comment, FakePrGh, Github, Pr, PrGh, RealPrGh, RuleResult, Verdict};

const USAGE: &str = "usage: uor-harness loop queue --pr <n> [--dry-run] [--json]";

/// The metadata-and-merge side of `gh` and the read side pr-check's rules use.
type Ghs = (Box<dyn QueueGh>, Box<dyn PrGh>);

/// TEST-ONLY: a JSON file standing in for `gh`: `{"pr": {"headRefOid", "headRefName",
/// "baseRefName", "state", "isDraft"}, "comments": [{"author", "body"}], "checkRuns": [...], "statuses": [...],
/// "failMerge": bool}` (the reads as `UOR_PRCHECK_GH_FAKE` writes them). Each merge call is
/// appended as a JSON line to `<file>.calls.jsonl`.
pub const QUEUE_GH_FAKE_ENV: &str = "UOR_LOOP_QUEUE_GH_FAKE";

/// What `gh pr view` says about the PR.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuePr {
    pub head_ref_oid: String,
    pub head_ref_name: String,
    pub base_ref_name: String,
    pub state: String,
    pub is_draft: bool,
}

/// The PR metadata `loop queue` reads and the one write it makes.
pub trait QueueGh {
    fn pr(&self, n: u64) -> Result<QueuePr, String>;
    /// `gh pr merge <n> --match-head-commit <sha>`.
    fn merge(&self, n: u64, sha: &str) -> Result<(), String>;
}

/// `gh`, run from the repository root.
pub struct RealQueueGh {
    pub root: PathBuf,
}

impl QueueGh for RealQueueGh {
    fn pr(&self, n: u64) -> Result<QueuePr, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "view",
                &n.to_string(),
                "--json",
                "headRefOid,headRefName,baseRefName,state,isDraft",
            ],
        )?;
        serde_json::from_str(&out)
            .map_err(|e| format!("gh pr view {n} printed unreadable JSON: {e}"))
    }
    fn merge(&self, n: u64, sha: &str) -> Result<(), String> {
        crate::loopclaim::run_gh(
            &self.root,
            &["pr", "merge", &n.to_string(), "--match-head-commit", sha],
        )
        .map(|_| ())
    }
}

/// The in-memory stand-in for `gh`, read from the file [`QUEUE_GH_FAKE_ENV`] names.
#[derive(serde::Deserialize, Default)]
pub struct FakeQueueGh {
    #[serde(default)]
    pr: QueuePr,
    #[serde(default, rename = "failMerge")]
    fail_merge: bool,
    #[serde(skip)]
    calls: PathBuf,
}

impl FakeQueueGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {QUEUE_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        let mut f: Self = serde_json::from_str(&text).map_err(|e| {
            format!(
                "{QUEUE_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })?;
        f.calls = PathBuf::from(format!("{}.calls.jsonl", path.display()));
        Ok(f)
    }
}

impl QueueGh for FakeQueueGh {
    fn pr(&self, _n: u64) -> Result<QueuePr, String> {
        Ok(self.pr.clone())
    }
    fn merge(&self, n: u64, sha: &str) -> Result<(), String> {
        if self.fail_merge {
            return Err(format!("fake PR #{n}: the merge call failed"));
        }
        use std::io::Write as _;
        let line = serde_json::json!({"call": "merge", "pr": n, "matchHeadCommit": sha});
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.calls)
            .and_then(|mut f| writeln!(f, "{line}"))
            .map_err(|e| format!("cannot log to {}: {e}", self.calls.display()))
    }
}

/// `git` from `root` with every `GIT_*` variable removed except the ones that only say how to
/// reach a remote.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = std::process::Command::new("git");
    for (k, _) in std::env::vars_os() {
        let k = k.to_string_lossy().into_owned();
        if k.starts_with("GIT_")
            && !["GIT_SSH_COMMAND", "GIT_SSH", "GIT_ASKPASS"].contains(&k.as_str())
        {
            cmd.env_remove(&k);
        }
    }
    let out = cmd
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The after-approval rule over the comments after index `approve` (the last counted record, an
/// APPROVE): `(refused, details)`. A well-formed `uor-review v1` record is left to the gate; a
/// malformed one refuses, because the gate ignores it and it may be a hold written in the
/// record's shape; a `[bot]` author is ignored; a body starting `[<lab>]` for a registered lab
/// whose role is not `pm` is ignored; any other comment refuses.
pub fn after_approval(
    comments: &[Comment],
    approve: usize,
    labs: &BTreeMap<String, crate::labs::Lab>,
) -> (bool, Vec<String>) {
    let mut refused = false;
    let mut details = Vec::new();
    for (i, c) in comments.iter().enumerate().skip(approve + 1) {
        let n = i + 1;
        match crate::loopreview::parse(&c.body) {
            crate::loopreview::Parsed::NotARecord => {}
            crate::loopreview::Parsed::Record(_) => {
                details.push(format!("comment {n}: a review record, left to the gate"));
                continue;
            }
            crate::loopreview::Parsed::Malformed(why) => {
                refused = true;
                details.push(format!(
                    "comment {n}: refused, a malformed uor-review v1 record after the approval ({why}); it may be a hold"
                ));
                continue;
            }
        }
        if c.author.ends_with("[bot]") {
            details.push(format!("comment {n}: ignored, from bot {}", c.author));
            continue;
        }
        let tag = c
            .body
            .strip_prefix('[')
            .and_then(|rest| rest.split_once(']'))
            .map(|(lab, _)| lab);
        match tag.and_then(|t| labs.get(t).map(|l| (t, l))) {
            Some((lab, l)) if l.role.as_deref() != Some("pm") => {
                details.push(format!("comment {n}: ignored, from lab {lab}"));
            }
            Some((lab, _)) => {
                refused = true;
                details.push(format!(
                    "comment {n}: refused, the PM ({lab}) commented after the approval"
                ));
            }
            None => {
                refused = true;
                details.push(format!(
                    "comment {n}: refused, an untagged comment after the approval may be the owner's or the PM's (one GitHub account for every lab, D55)"
                ));
            }
        }
    }
    (refused, details)
}

/// One line of the report: a pr-check rule or one of queue's own checks.
struct Check {
    name: String,
    verdict: Verdict,
    summary: String,
    details: Vec<String>,
}

impl Check {
    fn new(name: &str, pass: bool, summary: String, details: Vec<String>) -> Check {
        Check {
            name: name.to_string(),
            verdict: if pass { Verdict::Pass } else { Verdict::Fail },
            summary,
            details,
        }
    }
}

impl From<RuleResult> for Check {
    fn from(r: RuleResult) -> Check {
        Check {
            name: r.rule.to_string(),
            verdict: r.verdict,
            summary: r.summary,
            details: r.details,
        }
    }
}

struct Args {
    pr: u64,
    dry_run: bool,
    json: bool,
}

fn parse_args(args: &[String]) -> Option<Args> {
    let (mut pr, mut dry_run, mut json) = (None, false, false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            "--pr" => {
                pr = Some(args.get(i + 1)?.parse::<u64>().ok()?);
                i += 1;
            }
            _ => return None,
        }
        i += 1;
    }
    Some(Args {
        pr: pr?,
        dry_run,
        json,
    })
}

/// What the run decided, before printing.
enum Outcome {
    Refused,
    WouldQueue,
    Queued,
    /// Every check passed and the merge call failed.
    MergeFailed,
}

fn print(a: &Args, head: &str, checks: &[Check], outcome: &Outcome, fake: Option<&str>) {
    let verdict = match outcome {
        Outcome::Refused => "refused",
        Outcome::WouldQueue => "would-queue",
        Outcome::Queued => "queued",
        Outcome::MergeFailed => "merge-failed",
    };
    if a.json {
        let checks: Vec<serde_json::Value> = checks
            .iter()
            .map(|c| {
                serde_json::json!({
                    "check": c.name,
                    "verdict": c.verdict.text().to_lowercase(),
                    "summary": c.summary,
                    "details": c.details,
                })
            })
            .collect();
        let out = serde_json::json!({
            "pr": a.pr,
            "head": head,
            "dry_run": a.dry_run,
            "verdict": verdict,
            "fake_github": fake.is_some(),
            "checks": checks,
        });
        println!("{out}");
        return;
    }
    if let Some(p) = fake {
        println!("FAKE GitHub: {QUEUE_GH_FAKE_ENV}={p} (tests only)");
    }
    for c in checks {
        println!("{} {} {}", c.verdict.text(), c.name, c.summary);
        for d in &c.details {
            println!("  {d}");
        }
    }
    if matches!(outcome, Outcome::WouldQueue) {
        println!(
            "dry run: would run `gh pr merge {} --match-head-commit {head}`",
            a.pr
        );
    }
    println!("verdict: {}", verdict.to_uppercase());
}

/// `loop queue --pr <n> [--dry-run] [--json]`. Exit 0 queued (or would queue), 1 refused, 2 the
/// run could not finish (including a failed merge call, reported as `merge-failed`).
pub fn cli(root: &Path, _lab: &str, args: &[String]) -> u8 {
    let Some(a) = parse_args(args) else {
        eprintln!("{USAGE}");
        return 2;
    };
    let fake = std::env::var_os(QUEUE_GH_FAKE_ENV).map(|p| p.to_string_lossy().into_owned());
    let built: Result<Ghs, String> = match &fake {
        Some(p) => FakeQueueGh::from_file(Path::new(p)).and_then(|q| {
            let reads = FakePrGh::from_file(Path::new(p))?;
            Ok((
                Box::new(q) as Box<dyn QueueGh>,
                Box::new(reads) as Box<dyn PrGh>,
            ))
        }),
        None => Ok((
            Box::new(RealQueueGh {
                root: root.to_path_buf(),
            }),
            Box::new(RealPrGh {
                root: root.to_path_buf(),
            }),
        )),
    };
    let (gh, reads) = match built {
        Ok(g) => g,
        Err(e) => {
            eprintln!("uor-harness: loop queue: {e}");
            return 2;
        }
    };
    let cannot = |e: String| {
        eprintln!("uor-harness: loop queue: {e}");
        2
    };
    let meta = match gh.pr(a.pr) {
        Ok(m) => m,
        Err(e) => return cannot(e),
    };
    let head = meta.head_ref_oid.clone();
    let mut checks = Vec::new();
    let on_main = meta.base_ref_name == "main";
    let open = meta.state == "OPEN" && !meta.is_draft && on_main;
    checks.push(Check::new(
        "pr-state",
        open,
        format!(
            "PR {} is {}{}{}",
            a.pr,
            meta.state,
            if meta.is_draft { " and a draft" } else { "" },
            if on_main {
                String::new()
            } else {
                format!(", based on {:?}, not main", meta.base_ref_name)
            }
        ),
        Vec::new(),
    ));
    if !open {
        print(&a, &head, &checks, &Outcome::Refused, fake.as_deref());
        return 1;
    }
    let pull = format!("pull/{}/head", a.pr);
    let fetched = git(root, &["fetch", "-q", "origin", "main"])
        .and_then(|_| git(root, &["fetch", "-q", "origin", &pull]))
        .and_then(|_| git(root, &["rev-parse", "FETCH_HEAD"]));
    let fetched = match fetched {
        Ok(s) => s.trim().to_string(),
        Err(e) => return cannot(e),
    };
    let same = fetched == head;
    checks.push(Check::new(
        "head",
        same,
        if same {
            format!("origin {pull} is {head}, the head GitHub reports")
        } else {
            format!("the PR moved; re-run: origin {pull} is {fetched}, GitHub reports {head}")
        },
        Vec::new(),
    ));
    if !same {
        print(&a, &head, &checks, &Outcome::Refused, fake.as_deref());
        return 1;
    }
    let mut pr = match Pr::open(root, "origin/main", &head) {
        Ok(p) => p,
        Err(e) => return cannot(e),
    };
    pr.head_ref = meta.head_ref_name.clone();
    pr.github = Github::Pr(a.pr, reads);
    let results = crate::loopprcheck::evaluate(&pr);
    let failed = results.iter().any(|r| r.verdict == Verdict::Fail);
    checks.extend(results.into_iter().map(Check::from));

    // The comments are read after the rules ran, so a record posted meanwhile is seen here.
    let after = after_approval_check(root, &pr, a.pr);
    let after = match after {
        Ok(c) => c,
        Err(e) => return cannot(e),
    };
    let after_failed = after.verdict == Verdict::Fail;
    checks.push(after);
    if failed || after_failed {
        print(&a, &head, &checks, &Outcome::Refused, fake.as_deref());
        return 1;
    }
    if a.dry_run {
        print(&a, &head, &checks, &Outcome::WouldQueue, fake.as_deref());
        return 0;
    }
    if let Err(e) = gh.merge(a.pr, &head) {
        checks.push(Check::new(
            "merge",
            false,
            format!(
                "`gh pr merge {} --match-head-commit {head}` failed: {e}",
                a.pr
            ),
            Vec::new(),
        ));
        print(&a, &head, &checks, &Outcome::MergeFailed, fake.as_deref());
        return 2;
    }
    checks.push(Check::new(
        "merge",
        true,
        format!("ran `gh pr merge {} --match-head-commit {head}`", a.pr),
        Vec::new(),
    ));
    print(&a, &head, &checks, &Outcome::Queued, fake.as_deref());
    0
}

/// The after-approval check: the PR's comments with authors, the last counted record (which must
/// be an APPROVE) and [`after_approval`] over the comments after it, with the labs of
/// `origin/main`. `Err` when a read fails.
fn after_approval_check(root: &Path, pr: &Pr, number: u64) -> Result<Check, String> {
    const N: &str = "after-approval";
    let Github::Pr(_, gh) = &pr.github else {
        unreachable!("loop queue always reads GitHub")
    };
    let comments = gh.comments(number)?;
    let labs =
        crate::labs::parse_labs(&git(root, &["show", "origin/main:docs/harness/labs.json"])?)?;
    let ctx = match crate::loopprcheck::ReviewContext::of(pr) {
        Ok(c) => c,
        Err(e) => return Ok(Check::new(N, false, e, Vec::new())),
    };
    let bodies: Vec<String> = comments.iter().map(|c| c.body.clone()).collect();
    let Some(approve) =
        crate::loopreview::last_counted_approve(&ctx.input(number, &pr.head, &bodies))
    else {
        return Ok(Check::new(
            N,
            false,
            format!("the last counted record at {} is not an APPROVE", pr.head),
            Vec::new(),
        ));
    };
    let (refused, details) = after_approval(&comments, approve, &labs);
    let summary = if refused {
        format!(
            "a comment after the approval (comment {}) may be the owner's or the PM's",
            approve + 1
        )
    } else {
        format!(
            "nothing after the approval (comment {}) needs the owner or the PM",
            approve + 1
        )
    };
    Ok(Check::new(N, !refused, summary, details))
}
