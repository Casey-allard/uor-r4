//! `uor-harness loop next [--json] [--wait [--poll SECS] [--max SECS]]`: loop step 1, decide go, wait or stop (loop PR 7b).
//!
//! **Why this exists.** The loop page's step 1 decides go or stop and which kind of session, in a
//! fixed order of preference: owner direction, repairs to the lab's own PRs, reviews, GPU
//! collections, premise probes, the top of the lab's lane, then `any`. It stops only on the kill
//! switch (`local/loop/STOP` in the main checkout, or the label `loop:pause` on issue #2028); on
//! held work, a full open-PR limit or an empty queue it waits (`WAIT … — check again in 600s`).
//! Each source prints one line; the first that yields work decides GO.
//!
//! **Reads only.** GitHub is read through [`NextGh`]; the real one runs `gh`, tests point the
//! binary at a JSON file through `UOR_LOOP_NEXT_GH_FAKE`. The `any` source asks
//! [`crate::loopclaim::check`] whether a claim would be accepted, which fetches `origin main`;
//! nothing else is written. Exit 0 GO, 1 STOP, 3 WAIT, 2 cannot run (usage, no lab, git or `gh`
//! error).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::loopclaim::PrFile;
use crate::loopprcheck::{Comment, PrGh as _};

const USAGE: &str = "usage: uor-harness loop next [--json] [--wait [--poll SECS] [--max SECS]]";

/// TEST-ONLY: a JSON file standing in for `gh`: `{"prs": [{number, headRefName, headRefOid,
/// isDraft, mergeable, files: [{path}], checks: [{name, conclusion}]}], "comments": {"<n>":
/// [{author, body}]}, "files": {"<n>:<path>": text}, "labels": {"<n>": [name]}}`.
pub const NEXT_GH_FAKE_ENV: &str = "UOR_LOOP_NEXT_GH_FAKE";

/// The issue whose `loop:pause` label pauses every lab's loop.
const PAUSE_ISSUE: u64 = 2028;
const PAUSE_LABEL: &str = "loop:pause";

/// Seconds before a lab told WAIT looks again.
pub const RECHECK_SECS: u64 = 600;

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
    /// No work now, and not the kill switch: look again in [`RECHECK_SECS`].
    Wait(String),
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

/// The text of `path` on `origin/main`, or `None` when it is not there.
fn origin_file(root: &Path, path: &str) -> Option<String> {
    git(root, &["show", &format!("origin/main:{path}")])
        .ok()
        .map(|s| s.to_string())
}

/// A Markdown table under one `## <heading>` section of PLAN.md: the header row and its data
/// rows. Columns are found by header name, never by position.
struct PlanTable {
    heading: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl PlanTable {
    /// The index of the column whose header reads `name` exactly (case-insensitive).
    fn column(&self, name: &str) -> Result<usize, String> {
        self.headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                format!(
                    "PLAN.md table `## {}` has no `{name}` column header",
                    self.heading
                )
            })
    }

    /// The rows whose status cell reads `open`, with their 1-based data row number.
    fn open_rows(&self, status: usize) -> Vec<(usize, &Vec<String>)> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                row.get(status)
                    .is_some_and(|c| c.eq_ignore_ascii_case("open"))
            })
            .map(|(n, row)| (n + 2, row))
            .collect()
    }
}

/// The table under the `## <heading>` section of PLAN.md. `Err` names a missing PLAN.md, a
/// missing section/table, or a table with no rows; the header row is the first table row.
fn plan_table(root: &Path, heading: &str) -> Result<PlanTable, String> {
    let Some(text) = origin_file(root, "PLAN.md") else {
        return Err("PLAN.md is not on origin/main yet (the PM writes it)".to_string());
    };
    let wanted = format!("## {}", heading.to_ascii_lowercase());
    let (mut inside, mut header, mut rows) = (false, None, Vec::new());
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
            if inside && header.is_some() {
                break;
            }
            inside = trimmed.to_ascii_lowercase().starts_with(&wanted);
            continue;
        }
        if !inside || !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        // The `|---|` separator carries no data.
        if !cells.is_empty()
            && cells
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
        {
            continue;
        }
        if header.is_none() {
            header = Some(cells);
        } else {
            rows.push(cells);
        }
    }
    let Some(headers) = header else {
        return Err(format!(
            "PLAN.md has no `## {heading}` table (add one with the documented headers)"
        ));
    };
    Ok(PlanTable {
        heading: heading.to_string(),
        headers,
        rows,
    })
}

/// The first open premise probe: `(premise, command)`. A malformed open row (no backticked
/// command in the Probe column) is skipped, named if no valid row follows, never hidden.
fn premise_probe(root: &Path) -> Result<Option<(String, String)>, String> {
    let table = plan_table(root, "Premises")?;
    let premise = table.column("Premise")?;
    let probe = table.column("Probe")?;
    let status = table.column("Status")?;
    let mut malformed = Vec::new();
    for (line, row) in table.open_rows(status) {
        let premise_text = row.get(premise).map(String::as_str).unwrap_or("").trim();
        let Some(command) = row
            .get(probe)
            .and_then(|c| c.split('`').nth(1))
            .map(str::trim)
            .filter(|c| !c.is_empty())
        else {
            malformed.push(format!(
                "table row {line}: no probe command in backticks in the Probe column"
            ));
            continue;
        };
        return Ok(Some((premise_text.to_string(), command.to_string())));
    }
    if !malformed.is_empty() {
        return Err(format!(
            "the Premises table has malformed open row(s): {}",
            malformed.join("; ")
        ));
    }
    Ok(None)
}

/// The first open owner direction: `(direction, malformed notes)`. The Direction and Status
/// columns are found by header name, so a `| done | ... | open |` note cell cannot be read as a
/// status; a malformed open row is skipped and named rather than hidden.
fn owner_direction(root: &Path) -> Result<Option<(String, Vec<String>)>, String> {
    let table = plan_table(root, "Owner directions")?;
    let direction = table.column("Direction")?;
    let status = table.column("Status")?;
    let mut malformed = Vec::new();
    for (line, row) in table.open_rows(status) {
        let text = row.get(direction).map(String::as_str).unwrap_or("").trim();
        if text.is_empty() {
            malformed.push(format!("table row {line}: the Direction cell is empty"));
            continue;
        }
        return Ok(Some((text.to_string(), malformed)));
    }
    if !malformed.is_empty() {
        return Err(format!(
            "the Owner directions table has malformed open row(s): {}",
            malformed.join("; ")
        ));
    }
    Ok(None)
}
/// The lane the owner assigned this lab in the owner-only registry, if any.
fn lab_lane(root: &Path, lab: &str) -> Result<String, String> {
    let Some(text) = origin_file(root, "docs/harness/labs.json") else {
        return Err("no docs/harness/labs.json on origin/main".to_string());
    };
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("labs.json is not JSON: {e}"))?;
    value
        .get("labs")
        .and_then(|labs| labs.get(lab))
        .and_then(|entry| entry.get("lane"))
        .and_then(|lane| lane.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            format!("no lane is assigned to {lab} in docs/harness/labs.json (the owner adds it)")
        })
}

/// The ready, scoped items on `origin/main` whose dependencies are all done: `(held, ready, waiting)`.
fn ready_items(root: &Path) -> Result<(usize, Vec<String>, Vec<String>), String> {
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
                // Dependencies that cannot be read cannot be checked, so the item waits.
                if let Some(e) = &item.depends_on_error {
                    waiting.push(format!("{id} waits: {e}"));
                    continue;
                }
                // An item waits while any dependency is missing, unreadable or not done on
                // origin/main.
                let unmet: Vec<String> = item
                    .depends_on
                    .iter()
                    .filter_map(|dep| match crate::loopclaim::read_item_opt(root, dep) {
                        Ok(Some(d)) if d.status.as_deref() == Some("done") => None,
                        Ok(Some(d)) => Some(format!(
                            "{dep} ({})",
                            d.status.as_deref().unwrap_or("no status")
                        )),
                        Ok(None) => Some(format!("{dep} (missing)")),
                        Err(e) => Some(format!("{dep} (unreadable: {e})")),
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
    Ok((held, ready, waiting))
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
    // 2. owner direction: the first open row of PLAN.md's `## Owner directions` table.
    match owner_direction(root) {
        Ok(Some((target, malformed))) => {
            let mut reason = format!("PLAN owner direction {target:?} is open");
            if !malformed.is_empty() {
                reason.push_str(&format!(" (skipped malformed: {})", malformed.join("; ")));
            }
            out.push(source("owner-direction", "work", reason.clone()));
            return Ok((
                out,
                Decision::Go {
                    kind: "owner-direction",
                    target,
                    reason,
                },
            ));
        }
        Ok(None) => out.push(source(
            "owner-direction",
            "none",
            "PLAN.md has no open owner direction",
        )),
        Err(why) => out.push(not_built("owner-direction", &why)),
    }
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
                let standing =
                    crate::loopreview::standing_changes(&gate_input(pr, &o, &main, &bodies));
                if !standing.is_empty() {
                    why.push(format!(
                        "a counted CHANGES at its head stands ({})",
                        standing.join(", ")
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
        // One counted record at the head settles the review, whichever lab made it: an APPROVE
        // passes the gate and a CHANGES waits on the author's repair, which is a new head.
        let records = crate::loopreview::counted(&gate_input(pr, &order, &main, &bodies));
        if !records.is_empty() {
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
    // 5. GPU collections: a run needs a collection record. PR 10 built the watcher that
    // notes a leased run, but no collection record is defined yet, so this source names the
    // gap instead of guessing (it is a separate unit; the watcher file is outside this item).
    out.push(not_built(
        "gpu-collections",
        "a GPU run needs a collection record the watcher (loop PR 10) does not write yet",
    ));
    // 6. premise probes: the first open row of PLAN.md's `## Premises` table. The probe cell
    // holds the command in backticks; a row is offered only when its status cell reads `open`.
    match premise_probe(root) {
        Ok(Some((premise, probe))) => {
            let reason = format!("PLAN premise {premise:?} has the open probe `{probe}`");
            out.push(source("premise-probes", "work", reason.clone()));
            return Ok((
                out,
                Decision::Go {
                    kind: "premise-probe",
                    target: probe,
                    reason,
                },
            ));
        }
        Ok(None) => out.push(source(
            "premise-probes",
            "none",
            "PLAN.md has no open premise probe",
        )),
        Err(why) => out.push(not_built("premise-probes", &why)),
    }
    // 7. lane: the top of the lab's lane. The lab's lane comes from the owner-only registry
    // (`docs/harness/labs.json`, an optional `lane` key); the ranking comes from the PM's
    // `lane:` and `rank:` item keys. With either absent the source says what is missing.
    let ready_state = ready_items(root);
    match lab_lane(root, lab) {
        Ok(lane) => match &ready_state {
            Err(why) => out.push(not_built(
                "lane",
                &format!("cannot list ready items: {why}"),
            )),
            Ok((_, ready, _)) => {
                let mut in_lane: Vec<(u32, String)> = Vec::new();
                for id in ready {
                    if let Ok(item) = crate::loopclaim::read_item(root, id) {
                        if item.lane.as_deref() == Some(lane.as_str()) {
                            in_lane.push((item.rank.unwrap_or(u32::MAX), id.clone()));
                        }
                    }
                }
                in_lane.sort();
                let mut refused = Vec::new();
                let mut chosen = None;
                for (_, id) in &in_lane {
                    match crate::loopclaim::check(root, lab, id, claim_gh) {
                        Ok(()) => {
                            chosen = Some(id.clone());
                            break;
                        }
                        Err(e) => refused.push(format!("{id}: {e}")),
                    }
                }
                if let Some(id) = chosen {
                    let reason = format!(
                        "top of {lab}'s {lane} lane: item {id} is ready, scoped and claimable"
                    );
                    out.push(source("lane", "work", reason.clone()));
                    return Ok((
                        out,
                        Decision::Go {
                            kind: "item",
                            target: id,
                            reason,
                        },
                    ));
                }
                let mut detail = format!("no claimable item in {lab}'s {lane} lane");
                if !refused.is_empty() {
                    detail.push_str(&format!(" ({})", refused.join("; ")));
                }
                out.push(source("lane", "none", detail));
            }
        },
        Err(why) => out.push(not_built("lane", &why)),
    }
    // 8. any: ready, scoped items whose dependencies are all done and that `loop claim` would
    // accept, in id order.
    let (held, ready, waiting) = ready_state?;
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
    // Only the kill switch stops; every other no-work case waits and says what is pending.
    let mut pending = Vec::new();
    if refused.iter().any(|r| r.contains("open_pr_limit")) {
        let mine: Vec<String> = prs
            .iter()
            .filter(|p| p.head_ref_name.starts_with(&prefix))
            .map(|p| format!("#{}", p.number))
            .collect();
        pending.push(format!(
            "open_pr_limit reached: own PRs {} await review",
            mine.join(", ")
        ));
    }
    if held > 0 {
        pending.push(format!("held work: {held} item(s) held"));
    }
    if !waiting.is_empty() {
        pending.push(format!("{} item(s) waiting on dependencies", waiting.len()));
    }
    if pending.is_empty() {
        pending.push("empty queue".to_string());
    }
    Ok((out, Decision::Wait(pending.join("; "))))
}

pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let mut json = false;
    let mut wait = false;
    let mut poll = RECHECK_SECS;
    let mut max = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--wait" => wait = true,
            "--poll" | "--max" => {
                let Some(secs) = it.next().and_then(|v| v.parse::<u64>().ok()) else {
                    eprintln!(
                        "uor-harness: loop next: {a} needs a whole number of seconds\n{USAGE}"
                    );
                    return 2;
                };
                if a == "--poll" {
                    poll = secs;
                } else {
                    max = Some(secs);
                }
            }
            _ => {
                eprintln!("uor-harness: loop next: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
        }
    }
    if !wait && (poll != RECHECK_SECS || max.is_some()) {
        eprintln!("uor-harness: loop next: --poll and --max need --wait\n{USAGE}");
        return 2;
    }
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop next: set UOR_LAB or pass --lab\n{USAGE}");
        return 2;
    }
    if !wait {
        return once(root, lab, json);
    }
    let start = std::time::Instant::now();
    wait_loop(
        poll,
        max,
        || once(root, lab, json),
        |secs| std::thread::sleep(std::time::Duration::from_secs(secs)),
        || start.elapsed().as_secs(),
    )
}

/// Read failures in a row that end `loop next --wait` with exit 2.
pub const WAIT_MAX_ERRORS: u32 = 3;

/// `loop next --wait`: ask until GO (0) or STOP (1). On WAIT (3) sleep `poll` seconds and ask
/// again, until `max` seconds have elapsed (exit 3; the last ask printed its WAIT line). A read
/// failure (2) is retried at the next poll; `WAIT_MAX_ERRORS` in a row end with exit 2. `ask`,
/// `sleep` and `elapsed` are injected so tests never sleep.
pub fn wait_loop(
    poll: u64,
    max: Option<u64>,
    mut ask: impl FnMut() -> u8,
    mut sleep: impl FnMut(u64),
    mut elapsed: impl FnMut() -> u64,
) -> u8 {
    let mut errors = 0;
    loop {
        let code = ask();
        match code {
            0 | 1 => return code,
            2 => {
                errors += 1;
                if errors >= WAIT_MAX_ERRORS {
                    return 2;
                }
                eprintln!("uor-harness: loop next --wait: read failed ({errors} in a row); retrying in {poll}s");
            }
            _ => errors = 0,
        }
        let nap = match max {
            None => poll,
            Some(m) => {
                let spent = elapsed();
                if spent >= m {
                    // A run that ends on a read failure says so; otherwise it ends on WAIT.
                    return if code == 2 { 2 } else { 3 };
                }
                poll.min(m - spent)
            }
        };
        sleep(nap);
    }
}

fn once(root: &Path, lab: &str, json: bool) -> u8 {
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
            Decision::Wait(reason) => serde_json::json!({
                "go": false, "kind": null, "target": null, "reason": reason,
                "wait": true, "recheck_secs": RECHECK_SECS
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
            Decision::Wait(reason) => println!("{}", wait_line(reason)),
        }
    }
    match decision {
        Decision::Go { .. } => 0,
        Decision::Stop(_) => 1,
        Decision::Wait(_) => 3,
    }
}

/// The line `loop next` prints on WAIT.
pub fn wait_line(reason: &str) -> String {
    format!("WAIT {reason} — check again in {RECHECK_SECS}s")
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
    /// Drives `wait_loop` with scripted exit codes, a fake clock that advances only by the naps it
    /// is asked for, and no real sleeping. Returns the exit code, the asks made and the naps taken.
    fn wait_run(poll: u64, max: Option<u64>, codes: &[u8]) -> (u8, usize, Vec<u64>) {
        let clock = std::cell::Cell::new(0u64);
        let naps = std::cell::RefCell::new(Vec::new());
        let mut asks = 0;
        let code = super::wait_loop(
            poll,
            max,
            || {
                asks += 1;
                codes.get(asks - 1).copied().unwrap_or(3)
            },
            |s| {
                naps.borrow_mut().push(s);
                clock.set(clock.get() + s);
            },
            || clock.get(),
        );
        (code, asks, naps.into_inner())
    }

    #[test]
    fn wait_ends_on_go_once_work_appears() {
        assert_eq!(wait_run(600, None, &[3, 3, 0]), (0, 3, vec![600, 600]));
        assert_eq!(wait_run(600, None, &[3, 1]), (1, 2, vec![600]));
    }

    #[test]
    fn wait_gives_exit_3_when_max_elapses() {
        // 600 + 600 + 300 = 1500: the last nap is cut to what remains, then one last ask.
        assert_eq!(wait_run(600, Some(1500), &[]), (3, 4, vec![600, 600, 300]));
        assert_eq!(wait_run(600, Some(0), &[]), (3, 1, vec![]));
    }

    #[test]
    fn wait_gives_exit_2_after_three_read_failures_in_a_row() {
        assert_eq!(wait_run(10, None, &[2, 2, 2]), (2, 3, vec![10, 10]));
        assert_eq!(wait_run(10, None, &[2, 3, 2, 2, 2]), (2, 5, vec![10; 4]));
    }

    #[test]
    fn wait_retries_a_single_read_failure_then_takes_go() {
        assert_eq!(wait_run(10, None, &[2, 0]), (0, 2, vec![10]));
    }

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
