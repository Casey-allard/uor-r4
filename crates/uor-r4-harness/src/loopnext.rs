//! `uor-harness loop next [--json]`: loop step 1, decide go or stop (loop PR 7b).
//!
//! **Why this exists.** The loop page's step 1 decides go or stop and which kind of session, in a
//! fixed order of preference: owner direction, repairs to the lab's own PRs, reviews, GPU
//! collections, premise probes, the top of the lab's lane, then `any`. It stops on the kill switch
//! (`local/loop/STOP` in the main checkout, or the label `loop:pause` on issue #2028), held work or
//! an empty queue. Each source prints one line; the first that yields work decides GO.
//!
//! **Reads only.** GitHub is read through [`NextGh`]; the real one runs `gh`, tests point the
//! binary at a JSON file through `UOR_LOOP_NEXT_GH_FAKE`. The `any` source asks
//! [`crate::loopclaim::check`] whether a claim would be accepted, which fetches `origin main`;
//! nothing else is written. Exit 0 GO, 1 STOP, 2 cannot run (usage, no lab, git or `gh` error).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::loopclaim::PrFile;
use crate::loopprcheck::{Comment, PrGh as _};

const USAGE: &str = "usage: uor-harness loop next [--json]";

/// TEST-ONLY: a JSON file standing in for `gh`: `{"prs": [{number, headRefName, headRefOid,
/// isDraft, mergeable, files: [{path}], checks: [{name, conclusion}]}], "comments": {"<n>":
/// [{author, body}]}, "files": {"<n>:<path>": text}, "labels": {"<n>": [name]}}`.
pub const NEXT_GH_FAKE_ENV: &str = "UOR_LOOP_NEXT_GH_FAKE";

/// The issue whose `loop:pause` label pauses every lab's loop.
const PAUSE_ISSUE: u64 = 2028;
const PAUSE_LABEL: &str = "loop:pause";

/// One check at a PR's head: its name and conclusion (`FAILURE`, `SUCCESS`, ...).
#[derive(Debug, Clone, Deserialize)]
pub struct PrCheck {
    pub name: String,
    #[serde(default)]
    pub conclusion: String,
}

/// One open PR, as `gh pr list --json number,headRefName,headRefOid,isDraft,mergeable,files,
/// statusCheckRollup` prints it, with the rollup reduced to [`PrCheck`]s.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextPr {
    pub number: u64,
    pub head_ref_name: String,
    #[serde(default)]
    pub head_ref_oid: String,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub mergeable: String,
    #[serde(default)]
    pub files: Vec<PrFile>,
    #[serde(default)]
    pub checks: Vec<PrCheck>,
}

/// Every GitHub read `loop next` makes; it never writes.
pub trait NextGh {
    fn open_prs(&self) -> Result<Vec<NextPr>, String>;
    /// The issue comments on PR `n`, oldest first.
    fn comments(&self, n: u64) -> Result<Vec<Comment>, String>;
    /// The text of `path` at PR `n`'s head.
    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String>;
    /// The label names on issue `n`.
    fn issue_labels(&self, n: u64) -> Result<Vec<String>, String>;
    /// Every changed path of PR `n`, all pages (the listed `files` stops at 100).
    fn pr_files(&self, n: u64) -> Result<Vec<String>, String>;
}

/// `gh`, run from the repository root.
pub struct RealNextGh {
    pub root: std::path::PathBuf,
}

impl NextGh for RealNextGh {
    fn open_prs(&self) -> Result<Vec<NextPr>, String> {
        let text = crate::loopclaim::run_gh(
            &self.root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--limit",
                "200",
                "--json",
                "number,headRefName,headRefOid,isDraft,mergeable,files,statusCheckRollup",
            ],
        )?;
        let v: Vec<serde_json::Value> = serde_json::from_str(&text)
            .map_err(|e| format!("gh pr list printed unreadable JSON: {e}"))?;
        v.into_iter()
            .map(|mut p| {
                // A CheckRun has `name` and `conclusion`; a StatusContext has `context` and `state`.
                let checks: Vec<serde_json::Value> = p
                    .get("statusCheckRollup")
                    .and_then(|r| r.as_array())
                    .into_iter()
                    .flatten()
                    .map(|c| {
                        let s = |k: &str| c.get(k).and_then(|v| v.as_str()).unwrap_or("");
                        let name = if s("name").is_empty() {
                            s("context")
                        } else {
                            s("name")
                        };
                        let conclusion = if s("conclusion").is_empty() {
                            s("state")
                        } else {
                            s("conclusion")
                        };
                        serde_json::json!({"name": name, "conclusion": conclusion})
                    })
                    .collect();
                p["checks"] = serde_json::Value::Array(checks);
                serde_json::from_value(p).map_err(|e| format!("gh pr list: unreadable PR: {e}"))
            })
            .collect()
    }

    fn comments(&self, n: u64) -> Result<Vec<Comment>, String> {
        crate::loopprcheck::RealPrGh {
            root: self.root.clone(),
        }
        .comments(n)
    }

    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String> {
        let endpoint = crate::loopclaim::contents_endpoint(path, n);
        crate::loopclaim::run_gh(
            &self.root,
            &[
                "api",
                "-H",
                "Accept: application/vnd.github.raw+json",
                &endpoint,
            ],
        )
    }

    fn issue_labels(&self, n: u64) -> Result<Vec<String>, String> {
        let args = paginated_args(&format!("repos/{{owner}}/{{repo}}/issues/{n}/labels"));
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        Ok(paged_values(&self.root, &args)?
            .iter()
            .filter_map(|l| l.get("name")?.as_str().map(str::to_string))
            .collect())
    }

    fn pr_files(&self, n: u64) -> Result<Vec<String>, String> {
        let args = paginated_args(&format!("repos/{{owner}}/{{repo}}/pulls/{n}/files"));
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        Ok(paged_values(&self.root, &args)?
            .iter()
            .filter_map(|f| f.get("filename")?.as_str().map(str::to_string))
            .collect())
    }
}

/// The `gh` arguments for a paginated `gh api` read of `endpoint`.
fn paginated_args(endpoint: &str) -> Vec<String> {
    ["api", endpoint, "--paginate"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The items of a paginated `gh api` array read: `--paginate` prints one JSON array per page.
fn paged_values(root: &Path, args: &[&str]) -> Result<Vec<serde_json::Value>, String> {
    let text = crate::loopclaim::run_gh(root, args)?;
    let mut items = Vec::new();
    for page in serde_json::Deserializer::from_str(&text).into_iter::<serde_json::Value>() {
        let page =
            page.map_err(|e| format!("gh {} printed unreadable JSON: {e}", args.join(" ")))?;
        match page {
            serde_json::Value::Array(v) => items.extend(v),
            other => items.push(other),
        }
    }
    Ok(items)
}

/// The in-memory stand-in for `gh`, read from the file `UOR_LOOP_NEXT_GH_FAKE` names.
#[derive(Deserialize, Default)]
pub struct FakeNextGh {
    #[serde(default)]
    prs: Vec<NextPr>,
    #[serde(default)]
    comments: BTreeMap<String, Vec<Comment>>,
    /// PR numbers whose comment read fails.
    #[serde(default, rename = "failComments")]
    fail_comments: Vec<u64>,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    labels: BTreeMap<String, Vec<String>>,
    #[serde(default, rename = "allFiles")]
    all_files: BTreeMap<String, Vec<String>>,
}

impl FakeNextGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {NEXT_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        serde_json::from_str(&text).map_err(|e| {
            format!(
                "{NEXT_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })
    }
}

impl NextGh for FakeNextGh {
    fn open_prs(&self) -> Result<Vec<NextPr>, String> {
        Ok(self.prs.clone())
    }
    fn comments(&self, n: u64) -> Result<Vec<Comment>, String> {
        if self.fail_comments.contains(&n) {
            return Err(format!("fake PR #{n}: comments read failed"));
        }
        Ok(self
            .comments
            .get(&n.to_string())
            .cloned()
            .unwrap_or_default())
    }
    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String> {
        self.files
            .get(&format!("{n}:{path}"))
            .cloned()
            .ok_or_else(|| format!("fake PR #{n} has no text for {path}"))
    }
    fn issue_labels(&self, n: u64) -> Result<Vec<String>, String> {
        Ok(self.labels.get(&n.to_string()).cloned().unwrap_or_default())
    }
    fn pr_files(&self, n: u64) -> Result<Vec<String>, String> {
        self.all_files
            .get(&n.to_string())
            .cloned()
            .ok_or_else(|| format!("fake PR #{n} has no allFiles"))
    }
}

/// One source's line: its name, `stop`, `work`, `none` or `not-built`, and what it found.
#[derive(Debug)]
pub struct Source {
    pub source: &'static str,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug)]
pub enum Decision {
    Go {
        kind: &'static str,
        target: String,
        reason: String,
    },
    Stop(String),
}

/// The ORDER a PR adds: its lab (from `<lab>-<n>/ORDER.md`), its item and `Setup row:`, and
/// whether the PR is doc-only; with `origin/main` and the comments, what the review gate reads.
struct PrOrder {
    path: String,
    lab: String,
    item: String,
    setup_row: Option<String>,
    doc_only: bool,
}

/// Why a PR has no ORDER: a `gh` read failed (an own PR fails closed on it), or the PR adds none, or
/// several.
pub(crate) enum OrderErr {
    Read(String),
    Missing(String),
}

impl OrderErr {
    pub(crate) fn note(self) -> String {
        match self {
            Self::Read(s) | Self::Missing(s) => s,
        }
    }
}

/// The ORDER PR `n` adds, as `(path, text, lab)`: among `listed` (every changed path of the PR),
/// the paths matching `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` that `read` finds at its head
/// (a deleted one is not) and that are absent from `origin/main` (an edited one is not). Exactly
/// one must remain. Shared by `loop next` and `loop review`.
pub(crate) fn added_order(
    root: &Path,
    n: u64,
    listed: &[String],
    read: &dyn Fn(&str) -> Result<String, String>,
) -> Result<(String, String, String), OrderErr> {
    let mut added = Vec::new();
    for path in listed
        .iter()
        .map(String::as_str)
        .filter(|p| crate::loopprcheck::order_parts(p).is_some())
    {
        let text = match read(path) {
            Ok(t) => t,
            // A path deleted at the head is absent (404); any other failure is a read error.
            Err(e) if path_absent(&e) => continue,
            Err(e) => return Err(OrderErr::Read(e)),
        };
        if git(root, &["cat-file", "-e", &format!("origin/main:{path}")]).is_ok() {
            continue;
        }
        added.push((path, text));
    }
    if added.is_empty() {
        return Err(OrderErr::Missing(format!("#{n} adds no ORDER")));
    }
    // A repair chain resolves to its highest `<lab>-<n>` (loop step 11).
    let paths: Vec<&str> = added.iter().map(|(p, _)| *p).collect();
    let active = crate::loopprcheck::active_order(&paths)
        .map_err(|e| OrderErr::Missing(format!("#{n} adds {e}")))?;
    let (path, text) = added
        .into_iter()
        .find(|(p, _)| *p == active)
        .expect("the active ORDER is one of the added ORDERs");
    let dir = path.trim_end_matches("/ORDER.md");
    let lab = dir
        .rsplit('/')
        .next()
        .and_then(|d| d.rsplit_once('-'))
        .map(|(l, _)| l.to_string())
        .unwrap_or_default();
    Ok((path.to_string(), text, lab))
}

/// The ORDER a PR adds, or why the PR is skipped (see [`added_order`]).
fn pr_order(root: &Path, gh: &dyn NextGh, pr: &NextPr) -> Result<PrOrder, OrderErr> {
    // The listed files stop at 100; a PR that reaches it may have more, so read every page.
    let listed: Vec<String> = if pr.files.len() >= 100 {
        gh.pr_files(pr.number).map_err(OrderErr::Read)?
    } else {
        pr.files.iter().map(|f| f.path.clone()).collect()
    };
    let (path, text, lab) =
        added_order(root, pr.number, &listed, &|p| gh.file_at_pr(pr.number, p))?;
    Ok(PrOrder {
        path,
        lab,
        item: crate::loopclaim::order_item(&text).unwrap_or_default(),
        setup_row: crate::loopreview::setup_row(&text),
        doc_only: crate::loopreview::doc_only(listed.iter().map(String::as_str)),
    })
}

/// What the review gate reads for this PR: its head, ORDER and comment bodies, and `origin/main`.
fn gate_input<'a>(
    pr: &'a NextPr,
    order: &'a PrOrder,
    main: &'a crate::loopreview::MainContext,
    comments: &'a [String],
) -> crate::loopreview::GateInput<'a> {
    crate::loopreview::GateInput {
        pr: pr.number,
        head: &pr.head_ref_oid,
        order_path: &order.path,
        order_lab: &order.lab,
        order_item: &order.item,
        setup_row: order.setup_row.as_deref(),
        rows: &main.rows,
        vendors: &main.vendors,
        d59_in_force: main.d59_in_force,
        doc_only: order.doc_only,
        comments,
    }
}

/// The PR's comment bodies, oldest first.
fn comment_bodies(gh: &dyn NextGh, n: u64) -> Result<Vec<String>, String> {
    Ok(gh.comments(n)?.into_iter().map(|c| c.body).collect())
}

fn source(source: &'static str, status: &'static str, detail: impl Into<String>) -> Source {
    Source {
        source,
        status,
        detail: detail.into(),
    }
}

fn not_built(name: &'static str, why: &str) -> Source {
    source(name, "not-built", format!("not built — {why}"))
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    crate::loopgates::git(root, args).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// The sources in the loop page's order, stopping at the first that decides.
pub fn run(
    root: &Path,
    lab: &str,
    gh: &dyn NextGh,
    claim_gh: &dyn crate::loopclaim::Gh,
) -> Result<(Vec<Source>, Decision), String> {
    let mut out = Vec::new();
    // 1. kill switch.
    let stop_file = crate::loopclaim::main_checkout(root)?.join("local/loop/STOP");
    if stop_file.exists() {
        let why = format!("kill switch: {} exists", stop_file.display());
        out.push(source("kill-switch", "stop", why.clone()));
        return Ok((out, Decision::Stop(why)));
    }
    let labels = gh
        .issue_labels(PAUSE_ISSUE)
        .map_err(|e| format!("cannot read the labels of #{PAUSE_ISSUE}: {e}"))?;
    if labels.iter().any(|l| l == PAUSE_LABEL) {
        let why = format!("kill switch: label {PAUSE_LABEL} on #{PAUSE_ISSUE}");
        out.push(source("kill-switch", "stop", why.clone()));
        return Ok((out, Decision::Stop(why)));
    }
    out.push(source(
        "kill-switch",
        "none",
        format!("off (no local/loop/STOP, no {PAUSE_LABEL} on #{PAUSE_ISSUE})"),
    ));
    // 2. owner direction.
    out.push(not_built(
        "owner-direction",
        "owner directions live in PLAN, loop PR 8",
    ));
    // The build-plan rows, vendors and D59 come from `origin/main`, not from this branch's
    // working tree.
    let main = crate::loopreview::main_context(root)
        .map_err(|e| format!("cannot read the review context at origin/main: {e}"))?;
    let mut prs = gh
        .open_prs()
        .map_err(|e| format!("cannot list open PRs: {e}"))?;
    prs.sort_by_key(|p| p.number);
    // 3. repairs to the lab's own PRs.
    let prefix = format!("{lab}/");
    let own: Vec<&NextPr> = prs
        .iter()
        .filter(|p| p.head_ref_name.starts_with(&prefix))
        .collect();
    for pr in &own {
        let mut why = Vec::new();
        if pr.mergeable == "CONFLICTING" {
            why.push("it conflicts with main".to_string());
        }
        for c in &pr.checks {
            if c.conclusion == "FAILURE" || c.conclusion == "TIMED_OUT" {
                why.push(format!("check {:?} is {}", c.name, c.conclusion));
            }
        }
        // An own PR fails closed: an unreadable ORDER or comment list stops `next` (exit 2).
        let unreadable =
            |e: String| format!("cannot run: could not read own PR #{}: {e}", pr.number);
        match pr_order(root, gh, pr) {
            Err(OrderErr::Read(e)) => return Err(unreadable(e)),
            Err(OrderErr::Missing(_)) => {
                why.push("its ORDER is missing or ambiguous".to_string());
            }
            Ok(o) => {
                let bodies = comment_bodies(gh, pr.number).map_err(unreadable)?;
                let records = crate::loopreview::counted(&gate_input(pr, &o, &main, &bodies));
                if let Some((rlab, crate::loopreview::Verdict::Changes)) = records.last() {
                    why.push(format!(
                        "the last counted review record at its head ({rlab}) says CHANGES"
                    ));
                }
            }
        }
        if !why.is_empty() {
            let reason = why.join("; ");
            out.push(source(
                "repairs",
                "work",
                format!("#{}: {reason}", pr.number),
            ));
            return Ok((
                out,
                Decision::Go {
                    kind: "repair",
                    target: format!("#{}", pr.number),
                    reason,
                },
            ));
        }
    }
    let detail = format!("{} own open PR(s), none needs a repair", own.len());
    out.push(source("repairs", "none", detail));
    // 4. reviews.
    let mut skipped = Vec::new();
    for pr in prs.iter().filter(|p| !p.is_draft) {
        let order = match pr_order(root, gh, pr) {
            Ok(o) => o,
            Err(note) => {
                skipped.push(note.note());
                continue;
            }
        };
        if !crate::loopreview::may_review(&gate_input(pr, &order, &main, &[]), lab) {
            continue;
        }
        let bodies = match comment_bodies(gh, pr.number) {
            Ok(b) => b,
            Err(e) => {
                skipped.push(format!("#{}: {e}", pr.number));
                continue;
            }
        };
        let records = crate::loopreview::counted(&gate_input(pr, &order, &main, &bodies));
        if records.iter().any(|(l, _)| l == lab) {
            continue;
        }
        let reason = format!(
            "#{} needs {lab}'s exact-head review at {}",
            pr.number, pr.head_ref_oid
        );
        out.push(source("reviews", "work", reason.clone()));
        return Ok((
            out,
            Decision::Go {
                kind: "review",
                target: format!("#{}", pr.number),
                reason,
            },
        ));
    }
    let mut detail = "no open PR needs this lab's exact-head review".to_string();
    if !skipped.is_empty() {
        detail.push_str(&format!(" (skipped: {})", skipped.join(", ")));
    }
    out.push(source("reviews", "none", detail));
    // 5-7. not built yet.
    out.push(not_built("gpu-collections", "watchers are loop PR 10"));
    out.push(not_built(
        "premise-probes",
        "PLAN's premise table is loop PR 8",
    ));
    out.push(not_built(
        "lane",
        "no lab has a lane and no item has `lane:`/`rank:` yet",
    ));
    // 8. any: ready, scoped items whose dependencies are all done and that `loop claim` would
    // accept, in id order.
    git(root, &["fetch", "-q", "origin", "main"])
        .map_err(|e| format!("cannot fetch origin main: {e}"))?;
    let listing = git(
        root,
        &[
            "ls-tree",
            "--name-only",
            "origin/main",
            "docs/roadmap/items/",
        ],
    )?;
    let (mut held, mut ready, mut waiting) = (0usize, Vec::new(), Vec::new());
    for id in listing
        .lines()
        .filter_map(|p| p.strip_prefix("docs/roadmap/items/")?.strip_suffix(".md"))
    {
        let Ok(item) = crate::loopclaim::read_item(root, id) else {
            continue;
        };
        match item.status.as_deref() {
            Some("held") => held += 1,
            Some("ready") if item.scope.is_some() => {
                // An item waits while any dependency is missing or not done on origin/main.
                let unmet: Vec<String> = item
                    .depends_on
                    .iter()
                    .filter_map(|dep| match crate::loopclaim::read_item(root, dep) {
                        Ok(d) if d.status.as_deref() == Some("done") => None,
                        Ok(d) => Some(format!(
                            "{dep} ({})",
                            d.status.as_deref().unwrap_or("no status")
                        )),
                        Err(_) => Some(format!("{dep} (missing)")),
                    })
                    .collect();
                if unmet.is_empty() {
                    ready.push(id.to_string());
                } else {
                    waiting.push(format!("{id} waits on {}", unmet.join(", ")));
                }
            }
            _ => {}
        }
    }
    let mut refused = Vec::new();
    for id in &ready {
        match crate::loopclaim::check(root, lab, id, claim_gh) {
            Ok(()) => {
                let reason = format!("item {id} is ready, scoped and claimable");
                out.push(source("any", "work", reason.clone()));
                return Ok((
                    out,
                    Decision::Go {
                        kind: "item",
                        target: id.clone(),
                        reason,
                    },
                ));
            }
            Err(e) => refused.push(format!("{id}: {e}")),
        }
    }
    let mut detail = format!("{} ready scoped item(s), none claimable", ready.len());
    if !refused.is_empty() {
        detail.push_str(&format!(" ({})", refused.join("; ")));
    }
    if !waiting.is_empty() {
        detail.push_str(&format!(
            "; {} waiting on dependencies ({})",
            waiting.len(),
            waiting.join("; ")
        ));
    }
    out.push(source("any", "none", detail));
    let stop = if held > 0 {
        format!("held work: {held} item(s) held, none ready and claimable")
    } else {
        "empty queue".to_string()
    };
    Ok((out, Decision::Stop(stop)))
}

pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let mut json = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            _ => {
                eprintln!("uor-harness: loop next: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
        }
    }
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop next: set UOR_LAB or pass --lab\n{USAGE}");
        return 2;
    }
    let gh: Box<dyn NextGh> = match std::env::var_os(NEXT_GH_FAKE_ENV) {
        Some(p) => match FakeNextGh::from_file(Path::new(&p)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("uor-harness: loop next: {e}");
                return 2;
            }
        },
        None => Box::new(RealNextGh {
            root: root.to_path_buf(),
        }),
    };
    let claim_gh: Box<dyn crate::loopclaim::Gh> =
        match std::env::var_os(crate::loopclaim::GH_FAKE_ENV) {
            Some(p) => match crate::loopclaim::FakeGh::from_file(Path::new(&p)) {
                Ok(f) => Box::new(f),
                Err(e) => {
                    eprintln!("uor-harness: loop next: {e}");
                    return 2;
                }
            },
            None => Box::new(crate::loopclaim::RealGh {
                root: root.to_path_buf(),
            }),
        };
    let (sources, decision) = match run(root, lab, gh.as_ref(), claim_gh.as_ref()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uor-harness: loop next: {e}");
            return 2;
        }
    };
    if json {
        let d = match &decision {
            Decision::Go {
                kind,
                target,
                reason,
            } => serde_json::json!({"go": true, "kind": kind, "target": target, "reason": reason}),
            Decision::Stop(reason) => serde_json::json!({
                "go": false, "kind": null, "target": null, "reason": reason
            }),
        };
        let v = serde_json::json!({
            "lab": lab,
            "sources": sources.iter().map(|s| serde_json::json!({
                "source": s.source, "status": s.status, "detail": s.detail,
            })).collect::<Vec<_>>(),
            "decision": d,
        });
        println!("{v}");
    } else {
        for s in &sources {
            println!("{}: {}", s.source, s.detail);
        }
        match &decision {
            Decision::Go {
                kind,
                target,
                reason,
            } => println!("GO {kind} {target} — {reason}"),
            Decision::Stop(reason) => println!("STOP {reason}"),
        }
    }
    match decision {
        Decision::Go { .. } => 0,
        Decision::Stop(_) => 1,
    }
}

/// Whether a `file_at_pr` error says the path is absent at the PR head. Only `gh`'s own message
/// after `" failed: "` is read: the command line `run_gh` echoes before it holds the PR number and
/// the path, either of which may contain "404".
fn path_absent(e: &str) -> bool {
    let msg = e.split_once(" failed: ").map_or(e, |(_, m)| m);
    msg.contains("(HTTP 404)") || msg.contains("Not Found") || msg.contains("has no text")
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_gh_s_own_message_can_say_a_path_is_absent() {
        let call = "gh api repos/{owner}/{repo}/contents/docs/loop/tasks/x/claude-404/ORDER.md?ref=refs%2Fpull%2F4040%2Fhead";
        assert!(super::path_absent(&format!(
            "{call} failed: gh: Not Found (HTTP 404)"
        )));
        assert!(!super::path_absent(&format!(
            "{call} failed: HTTP 502: Bad Gateway"
        )));
        assert!(!super::path_absent(&format!(
            "{call} failed: HTTP 401: Bad credentials"
        )));
        assert!(super::path_absent(
            "fake PR #7 has no text for docs/loop/tasks/x/claude-1/ORDER.md"
        ));
    }

    #[test]
    fn paginated_reads_pass_paginate() {
        let args = super::paginated_args("repos/{owner}/{repo}/issues/1/labels");
        assert!(args.iter().any(|a| a == "--paginate"), "{args:?}");
        assert_eq!(args[0], "api");
    }
}
