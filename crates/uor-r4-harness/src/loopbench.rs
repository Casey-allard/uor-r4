//! `uor-harness loop board --bench DIR [--every SECONDS]`: the bench feed (claude-25, #2904).
//!
//! Writes five files into DIR (`.gates.jsonl` and the decisions, plan, handoff and state pages) in
//! the shape the CorvusWing bench reads, from every lab's real state: roadmap items, decisions and
//! committed receipts on `origin/main` (read with `git show`, never the working tree), each lab's
//! open PRs with their ORDER and review record, lab presence, and each lab's local gate ledger.
//! Every write is a temp file in DIR then a rename; `INBOX.md` is never touched.
//!
//! With `--every`, each cycle first moves the checkout the feed runs in to the fetched
//! `origin/main`, since the bench's Git panel reads that checkout's HEAD (claude-111). Only a clean
//! detached HEAD behind `origin/main` moves; a named branch, a dirty tree, an operation in progress
//! or a commit `origin/main` lacks is left alone, and `STATE.md` says so.
//!
//! GitHub sits behind [`BenchGh`], which has read methods only, so the feed cannot write there.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::loopboard::{self, BoardGh, FakeBoardGh, OpenPr, RealBoardGh};
use crate::state::StatePaths;

/// Tests only: names a JSON file holding PRs, files, comments and contents for the feed.
pub const BENCH_GH_FAKE_ENV: &str = "UOR_LOOP_BENCH_GH_FAKE";

const USAGE: &str = "usage: uor-harness loop board --bench DIR [--every SECONDS]";

/// One PR comment.
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    #[serde(default)]
    pub author: Value,
    pub body: String,
}

/// The file paths and comments of one PR, read with a single `gh pr view`.
pub struct PrView {
    pub files: Vec<String>,
    pub comments: Vec<Comment>,
}

/// Every per-PR GitHub read the feed makes. Read methods only.
pub trait BenchGh {
    fn view(&self, pr: u64) -> Result<PrView, String>;
    fn contents(&self, pr: u64, head: &str, path: &str) -> Result<String, String>;
}

/// `gh` itself, run in the checkout.
pub struct RealBenchGh {
    pub root: PathBuf,
}

impl BenchGh for RealBenchGh {
    fn view(&self, pr: u64) -> Result<PrView, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "view", &pr.to_string(), "--json", "files,comments"],
        )?;
        let v: Value = serde_json::from_str(&out)
            .map_err(|e| format!("gh pr view printed malformed JSON: {e}"))?;
        let files = v["files"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|f| f["path"].as_str().map(str::to_string))
            .collect();
        let comments = serde_json::from_value(v["comments"].clone())
            .map_err(|e| format!("gh pr view comments are malformed: {e}"))?;
        Ok(PrView { files, comments })
    }
    fn contents(&self, _pr: u64, head: &str, path: &str) -> Result<String, String> {
        let endpoint = format!("repos/{{owner}}/{{repo}}/contents/{path}?ref={head}");
        crate::loopclaim::run_gh(
            &self.root,
            &["api", "-H", "Accept: application/vnd.github.raw", &endpoint],
        )
    }
}

/// The stand-in for `gh`, read from the file `UOR_LOOP_BENCH_GH_FAKE` names.
#[derive(Deserialize)]
pub struct FakeBenchGh {
    #[serde(default)]
    pub prs: Vec<OpenPr>,
    #[serde(default)]
    files: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    comments: BTreeMap<String, Vec<Comment>>,
    #[serde(default)]
    contents: BTreeMap<String, String>,
}

impl FakeBenchGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {BENCH_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        serde_json::from_str(&text).map_err(|e| {
            format!(
                "{BENCH_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })
    }
}

impl BenchGh for FakeBenchGh {
    fn view(&self, pr: u64) -> Result<PrView, String> {
        let key = pr.to_string();
        Ok(PrView {
            files: self.files.get(&key).cloned().unwrap_or_default(),
            comments: self.comments.get(&key).cloned().unwrap_or_default(),
        })
    }
    fn contents(&self, pr: u64, _head: &str, path: &str) -> Result<String, String> {
        self.contents
            .get(&format!("{pr}:{path}"))
            .cloned()
            .ok_or_else(|| format!("fake gh has no contents for {pr}:{path}"))
    }
}

impl BoardGh for FakeBenchGh {
    fn open_prs(&self) -> Result<Vec<OpenPr>, String> {
        Ok(self.prs.clone())
    }
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn show(root: &Path, path: &str) -> Option<String> {
    git(root, &["show", &format!("origin/main:{path}")]).ok()
}

fn ls(root: &Path, dir: &str) -> Vec<String> {
    git(root, &["ls-tree", "-r", "--name-only", "origin/main", dir])
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    for q in ['"', '\''] {
        if let Some(inner) = s.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner.to_string();
        }
    }
    s.to_string()
}

fn front(text: &str, key: &str) -> String {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return String::new();
    }
    lines
        .take_while(|l| l.trim() != "---")
        .find_map(|l| l.strip_prefix(&format!("{key}:")).map(unquote))
        .unwrap_or_default()
}

fn cell(s: &str) -> String {
    s.replace('\n', " ").replace('|', "\\|")
}

fn row(cells: &[String]) -> String {
    let joined: Vec<String> = cells.iter().map(|c| cell(c)).collect();
    format!("| {} |\n", joined.join(" | "))
}

fn table(head: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!("| {} |\n|{}\n", head.join(" | "), "---|".repeat(head.len()));
    for r in rows {
        out.push_str(&row(r));
    }
    out
}

struct Item {
    id: String,
    title: String,
    milestone: String,
    status: String,
    depends_on: Vec<String>,
    open_to: String,
}

struct LabPr {
    lab: String,
    pr: OpenPr,
    order_id: String,
    order: Option<String>,
    review: String,
    /// Why this PR's ORDER could not be read, when it could not.
    note: Option<String>,
}

/// The first `- ` bullet under the `## <heading>` section of an ORDER.
fn first_bullet(order: &str, heading: &str) -> String {
    let mut inside = false;
    for l in order.lines() {
        if l.starts_with("## ") {
            inside = l.trim_start_matches("## ").trim() == heading;
        } else if inside {
            if let Some(b) = l.strip_prefix("- ") {
                return b.trim().to_string();
            }
        }
    }
    String::new()
}

/// The review cell the merge gate would agree with: `APPROVE at <sha7>` / `CHANGES at <sha7>`
/// from the last record [`crate::loopreview::counted`] counts at the head; `review not counted
/// (<lab>)` when the last record at the head is one the gate refuses (D46, D59); else
/// `needs review`; `draft` for a draft.
fn gated_review_state(pr: &OpenPr, input: &crate::loopreview::GateInput) -> String {
    if pr.is_draft {
        return "draft".into();
    }
    let sha7 = pr.head_ref_oid.get(..7).unwrap_or(&pr.head_ref_oid);
    if let Some((_, v)) = crate::loopreview::counted(input).last() {
        return format!("{} at {sha7}", v.as_str());
    }
    uncounted_or_needs(pr, input.comments)
}

/// `review not counted (<lab>)` when a parsed review record sits at the PR's head, else
/// `needs review`: the cell for a PR the gate counts nothing for.
fn uncounted_or_needs(pr: &OpenPr, comments: &[String]) -> String {
    let at_head = comments
        .iter()
        .rev()
        .find_map(|b| match crate::loopreview::parse(b) {
            crate::loopreview::Parsed::Record(r)
                if r.pr == pr.number && r.head == pr.head_ref_oid =>
            {
                Some(r.lab)
            }
            _ => None,
        });
    match at_head {
        Some(lab) => format!("review not counted ({lab})"),
        None => "needs review".into(),
    }
}

/// The review cell of a PR with no readable ORDER: the gate counts nothing without one.
fn unordered_review_state(pr: &OpenPr, comments: &[String]) -> String {
    if pr.is_draft {
        return "draft".into();
    }
    uncounted_or_needs(pr, comments)
}

/// The Status cell of a PR whose ORDER could not be read: the first line of the reason, shortened.
fn unreadable(reason: &str) -> String {
    let line = reason.lines().next().unwrap_or("").trim();
    let short: String = line.chars().take(80).collect();
    format!("ORDER unreadable: {short}")
}

fn lab_prs(
    labs: &BTreeMap<String, crate::labs::Lab>,
    prs: &[OpenPr],
    gh: &dyn BenchGh,
    main: &crate::loopreview::MainContext,
) -> Result<Vec<LabPr>, String> {
    let re = regex::Regex::new(r"^docs/loop/tasks/[^/]+/([a-z0-9]+)-([0-9]+)/ORDER\.md$")
        .expect("a valid pattern");
    let mut out = Vec::new();
    for pr in prs {
        let Some(lab) = loopboard::lab_of(&pr.title, labs) else {
            continue;
        };
        // One PR's read error degrades that PR to a row; it never fails the refresh.
        let read = |view: &PrView| -> Result<Option<(u64, String, String)>, String> {
            let best = view
                .files
                .iter()
                .filter_map(|f| {
                    let c = re.captures(f)?;
                    (c[1] == *lab).then(|| (c[2].parse::<u64>().unwrap_or(0), f.clone()))
                })
                .max();
            let Some((n, path)) = best else {
                return Ok(None);
            };
            let order = gh.contents(pr.number, &pr.head_ref_oid, &path)?;
            Ok(Some((n, path, order)))
        };
        let (order_id, order, review, note) = match gh.view(pr.number) {
            Err(e) => (
                format!("#{}", pr.number),
                None,
                "needs review".to_string(),
                Some(e),
            ),
            Ok(view) => {
                let bodies: Vec<String> = view.comments.iter().map(|c| c.body.clone()).collect();
                match read(&view) {
                    Ok(Some((n, path, order))) => {
                        // A readable ORDER: show only what the merge gate counts.
                        let comments: Vec<String> =
                            view.comments.iter().map(|c| c.body.clone()).collect();
                        let item = crate::loopclaim::order_item(&order).unwrap_or_default();
                        let setup_row = crate::loopreview::setup_row(&order);
                        let input = crate::loopreview::GateInput {
                            pr: pr.number,
                            head: &pr.head_ref_oid,
                            order_path: &path,
                            order_lab: &lab,
                            order_item: &item,
                            setup_row: setup_row.as_deref(),
                            rows: &main.rows,
                            vendors: &main.vendors,
                            d59_in_force: main.d59_in_force,
                            doc_only: crate::loopreview::doc_only(
                                view.files.iter().map(String::as_str),
                            ),
                            comments: &comments,
                        };
                        let review = gated_review_state(pr, &input);
                        (format!("{lab}-{n}"), Some(order), review, None)
                    }
                    Ok(None) => (
                        format!("#{}", pr.number),
                        None,
                        unordered_review_state(pr, &bodies),
                        None,
                    ),
                    Err(e) => (
                        format!("#{}", pr.number),
                        None,
                        unordered_review_state(pr, &bodies),
                        Some(e),
                    ),
                }
            }
        };
        out.push(LabPr {
            order_id,
            lab,
            pr: pr.clone(),
            order,
            review,
            note: note.map(|e| unreadable(&e)),
        });
    }
    let rank = |r: &str| match r {
        _ if r.starts_with("CHANGES") => 0,
        "needs review" => 1,
        _ if r.starts_with("review not counted") => 1,
        _ if r.starts_with("APPROVE") => 2,
        _ => 3,
    };
    out.sort_by_key(|p| (rank(&p.review), p.pr.number));
    Ok(out)
}

fn items(root: &Path) -> Vec<Item> {
    ls(root, "docs/roadmap/items/")
        .into_iter()
        .filter(|p| p.ends_with(".md"))
        .filter_map(|p| {
            let text = show(root, &p)?;
            let id = Path::new(&p).file_stem()?.to_string_lossy().into_owned();
            let f = crate::loopclaim::parse_item(&text).ok();
            Some(Item {
                id,
                title: front(&text, "title"),
                milestone: front(&text, "milestone"),
                status: f
                    .as_ref()
                    .and_then(|f| f.status.clone())
                    .unwrap_or_default(),
                depends_on: f.map(|f| f.depends_on).unwrap_or_default(),
                open_to: front(&text, "open_to"),
            })
        })
        .collect()
}

/// The Status cell of an item: its first word puts the row in the bench's lane.
fn item_status(it: &Item, all: &[Item], claim: Option<&LabPr>) -> String {
    if it.status == "done" {
        return "done".into();
    }
    if let Some(p) = claim {
        return format!(
            "in progress: {}, PR #{}, {}",
            p.order_id, p.pr.number, p.review
        );
    }
    match it.status.as_str() {
        "held" => "held".into(),
        "decision" => "decision pending (owner)".into(),
        "ready" => {
            let waits = it.depends_on.iter().find(|d| {
                all.iter()
                    .find(|o| &&o.id == d)
                    .is_none_or(|o| o.status != "done")
            });
            match waits {
                Some(d) => format!("ready, waits on {d}"),
                None => "ready".into(),
            }
        }
        other => other.into(),
    }
}

fn decisions_rows(text: &str) -> Vec<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let split = |l: &str| -> Vec<String> {
        let mut cells = Vec::new();
        let mut cur = String::new();
        let (mut tick, mut esc) = (false, false);
        for ch in l.trim().trim_start_matches('|').chars() {
            if esc {
                cur.push(ch);
                esc = false;
            } else if ch == '\\' {
                cur.push(ch);
                esc = true;
            } else if ch == '`' {
                tick = !tick;
                cur.push(ch);
            } else if ch == '|' && !tick {
                cells.push(cur.trim().replace("\\|", "|"));
                cur.clear();
            } else {
                cur.push(ch);
            }
        }
        if !cur.trim().is_empty() {
            cells.push(cur.trim().replace("\\|", "|"));
        }
        cells
    };
    for i in 0..lines.len().saturating_sub(1) {
        let head = lines[i].trim();
        if head.starts_with('|')
            && lines[i + 1].trim().starts_with("|-")
            && split(head)
                .iter()
                .any(|c| c.eq_ignore_ascii_case("decision"))
        {
            return lines[i + 2..]
                .iter()
                .take_while(|l| l.trim().starts_with('|'))
                .map(|l| {
                    let mut c = split(l);
                    c.resize(6, String::new());
                    c
                })
                .collect();
        }
    }
    Vec::new()
}

/// The bench counts `PASS` and `FAIL`; the loop's ledgers say `GREEN` and `RED`. A `CAUGHT`
/// line is a planted-fault run (`loop gates --break`), not the gate's own result, so it is dropped.
fn bench_result(verdict: &str) -> Option<String> {
    match verdict.to_uppercase().as_str() {
        "GREEN" | "PASS" => Some("PASS".into()),
        "RED" | "FAIL" | "REFUSED" => Some("FAIL".into()),
        "CAUGHT" => None,
        other => Some(other.to_string()),
    }
}

fn gate_line(lab: &str, v: &Value) -> Option<(String, Value)> {
    let gate = v.get("gate").or_else(|| v.get("name"))?.as_str()?;
    let verdict = v.get("verdict").or_else(|| v.get("result"))?.as_str()?;
    Some((
        gate.to_string(),
        json!({
            "name": format!("{lab}/{gate}"),
            "result": bench_result(verdict)?,
            "seconds": v.get("seconds").cloned().unwrap_or(Value::Null),
            "time": v.get("ts").or_else(|| v.get("time")).cloned().unwrap_or(Value::Null),
            "commit": v.get("commit").cloned().unwrap_or(Value::Null),
        }),
    ))
}

/// Whether run `a` is newer than run `b`: by `time` when both are numbers or both are strings,
/// else the later-read run wins.
fn newer(a: &Value, b: &Value) -> bool {
    match (&a["time"], &b["time"]) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() >= y.as_f64(),
        (Value::String(x), Value::String(y)) => x >= y,
        _ => true,
    }
}

/// One row per gate: the newest run at `origin/main`'s head when there is one, else the newest run.
/// Every lab's PR-branch run is at an older commit than `main`, so listing them all would read as
/// no gate passing at the head.
fn gates(root: &Path, labs: &BTreeMap<String, crate::labs::Lab>) -> String {
    let re = regex::Regex::new(r"^docs/loop/tasks/[^/]+/([a-z0-9]+)-[0-9]+/receipts\.jsonl$")
        .expect("a valid pattern");
    let head = git(root, &["rev-parse", "origin/main"])
        .ok()
        .map(|h| h.trim().to_string());
    let at_head = |v: &Value| head.is_some() && v["commit"].as_str() == head.as_deref();
    let mut best: BTreeMap<String, Value> = BTreeMap::new();
    let mut push = |lab: &str, text: &str| {
        for l in text.lines().filter(|l| !l.trim().is_empty()) {
            let Some((gate, row)) = serde_json::from_str::<Value>(l)
                .ok()
                .and_then(|v| gate_line(lab, &v))
            else {
                continue;
            };
            let keep = match best.get(&gate) {
                None => true,
                Some(cur) if at_head(cur) != at_head(&row) => at_head(&row),
                Some(cur) => newer(&row, cur),
            };
            if keep {
                best.insert(gate, row);
            }
        }
    };
    for p in ls(root, "docs/loop/tasks/") {
        if let Some(c) = re.captures(&p) {
            if let Some(text) = show(root, &p) {
                push(&c[1], &text);
            }
        }
    }
    for lab in labs.keys() {
        if let Ok(text) = std::fs::read_to_string(root.join("local").join(lab).join("gates.jsonl"))
        {
            push(lab, &text);
        }
    }
    best.values().map(|row| format!("{row}\n")).collect()
}

/// What a cycle's checkout step did to the checkout the feed runs in.
enum Checkout {
    /// HEAD was already the fetched `origin/main`.
    Current,
    /// HEAD moved forward from this commit to `origin/main`.
    Moved(String),
    /// HEAD was left at this commit, for this reason.
    Left(String, String),
}

/// The files git keeps while an operation that a checkout would break is in progress.
const IN_PROGRESS: [(&str, &str); 6] = [
    ("MERGE_HEAD", "merge"),
    ("rebase-merge", "rebase"),
    ("rebase-apply", "rebase"),
    ("CHERRY_PICK_HEAD", "cherry-pick"),
    ("REVERT_HEAD", "revert"),
    ("BISECT_LOG", "bisect"),
];

/// Moves the checkout at `root` to `main` (the fetched `origin/main`) when HEAD is detached, the
/// tree is clean, no operation is in progress and HEAD is an ancestor of `main`; else leaves it.
/// A named branch is never switched, so a lab's working branch never moves.
fn sync_checkout(root: &Path, main: &str) -> Result<Checkout, String> {
    let head = git(root, &["rev-parse", "HEAD"])?.trim().to_string();
    let short = head.get(..12).unwrap_or(&head).to_string();
    let left = |why: String| Ok(Checkout::Left(short.clone(), why));
    let name = git(root, &["rev-parse", "--symbolic-full-name", "HEAD"])?;
    if name.trim() != "HEAD" {
        let branch = name.trim().trim_start_matches("refs/heads/");
        return left(format!("HEAD is branch `{branch}`"));
    }
    for (marker, op) in IN_PROGRESS {
        let p = git(root, &["rev-parse", "--git-path", marker])?;
        if root.join(p.trim()).exists() {
            return left(format!("a {op} is in progress"));
        }
    }
    let s = |n: &str| if n == "1" { "" } else { "s" };
    let dirty = git(root, &["status", "--porcelain"])?
        .lines()
        .count()
        .to_string();
    if dirty != "0" {
        return left(format!(
            "{dirty} uncommitted or untracked path{}",
            s(&dirty)
        ));
    }
    if head == main {
        return Ok(Checkout::Current);
    }
    let ahead = git(root, &["rev-list", "--count", &format!("{main}..HEAD")])?;
    let ahead = ahead.trim();
    if ahead != "0" {
        return left(format!("{ahead} commit{} not on origin/main", s(ahead)));
    }
    // A plain checkout, never forced: git itself refuses to overwrite a change made since the check.
    // A failing post-checkout hook fails the command after HEAD has moved, so HEAD is read again.
    if let Err(e) = git(root, &["checkout", "-q", "--detach", main]) {
        if git(root, &["rev-parse", "HEAD"])?.trim() != main {
            return Err(e);
        }
        eprintln!("uor-harness: loop board: bench feed: the checkout moved, but {e}");
    }
    Ok(Checkout::Moved(short))
}

fn write_atomic(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    let put = std::fs::write(&tmp, text)
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))
        .and_then(|()| {
            std::fs::rename(&tmp, dir.join(name))
                .map_err(|e| format!("cannot rename into {name}: {e}"))
        });
    if put.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    put
}

/// One refresh: returns (lab PRs, items, what the checkout step did). The checkout step runs only
/// when `sync` is set, which is each `--every` cycle.
fn refresh(
    root: &Path,
    dir: &Path,
    board_gh: &dyn BoardGh,
    gh: &dyn BenchGh,
    sync: bool,
) -> Result<(usize, usize, Option<Checkout>), String> {
    git(root, &["fetch", "-q", "origin", "main"])?;
    let sha = git(root, &["rev-parse", "origin/main"])?.trim().to_string();
    let sha12 = sha.get(..12).unwrap_or(&sha).to_string();
    // A failed step leaves the checkout as it was; the pages are still written.
    let checkout = sync.then(|| {
        sync_checkout(root, &sha).unwrap_or_else(|e| {
            let head = git(root, &["rev-parse", "HEAD"])
                .ok()
                .and_then(|h| h.trim().get(..12).map(str::to_string))
                .unwrap_or_else(|| "?".into());
            Checkout::Left(head, format!("the checkout step failed: {e}"))
        })
    });
    let date = git(root, &["show", "-s", "--format=%cs", "origin/main"])?
        .trim()
        .to_string();
    let now = crate::team::now_iso();
    let banner = format!(
        "<!-- generated by uor-harness loop board --bench from origin/main {sha12} at {now}; edits are overwritten -->\n\n"
    );
    // The registry comes from origin/main like everything else the feed reads.
    let labs = crate::labs::parse_labs(
        &show(root, "docs/harness/labs.json")
            .ok_or("cannot read docs/harness/labs.json at origin/main")?,
    )?;
    let prs = board_gh.open_prs()?;
    // Build-plan rows, vendors and D59 from origin/main, read once for every PR.
    let main = crate::loopreview::main_context(root)?;
    let lab_prs = lab_prs(&labs, &prs, gh, &main)?;
    let items = items(root);

    // PLAN.md
    let claims: BTreeMap<String, &LabPr> = lab_prs
        .iter()
        .filter_map(|p| Some((crate::loopclaim::order_item(p.order.as_deref()?)?, p)))
        .collect();
    let plan_rows: Vec<Vec<String>> = items
        .iter()
        .map(|it| {
            let claim = claims.get(&it.id).copied();
            vec![
                it.id.clone(),
                it.milestone.clone(),
                if it.title.is_empty() {
                    it.id.clone()
                } else {
                    it.title.clone()
                },
                claim
                    .map(|p| p.lab.clone())
                    .unwrap_or_else(|| it.open_to.clone()),
                String::new(),
                item_status(it, &items, claim),
            ]
        })
        .collect();
    let plan = format!(
        "{banner}# Plan\n\n## Tasks\n\n{}",
        table(
            &["Id", "Milestone", "Task", "Lab", "Size", "Status"],
            &plan_rows
        )
    );

    // HANDOFF.md
    let tag = regex::Regex::new(r"^\[[^\]]*\]\s*").expect("a valid pattern");
    let next_rows: Vec<Vec<String>> = lab_prs
        .iter()
        .map(|p| {
            vec![
                p.order_id.clone(),
                format!(
                    "[{}] #{} {}",
                    p.lab,
                    p.pr.number,
                    tag.replace(&p.pr.title, "")
                ),
                match (&p.order, &p.note) {
                    (Some(o), _) => first_bullet(o, "Done when"),
                    (None, None) => "(no ORDER)".into(),
                    (None, Some(_)) => String::new(),
                },
                p.order
                    .as_deref()
                    .map(|o| first_bullet(o, "Stop and ask if"))
                    .unwrap_or_default(),
                String::new(),
                p.note.clone().unwrap_or_else(|| p.review.clone()),
            ]
        })
        .collect();
    let blocked_rows: Vec<Vec<String>> = items
        .iter()
        .filter(|it| it.status == "decision")
        .map(|it| {
            vec![
                it.id.clone(),
                if it.title.is_empty() {
                    it.id.clone()
                } else {
                    it.title.clone()
                },
                "owner decision".into(),
            ]
        })
        .collect();
    let handoff = format!(
        "{banner}# Handoff\n\n## Next\n\n{}\n## Blocked\n\n{}",
        table(
            &["Id", "Task", "Done when", "Cancel when", "Size", "Status"],
            &next_rows
        ),
        table(&["Id", "Task", "Waiting on"], &blocked_rows)
    );

    // DECISIONS.md
    let dec_rows = decisions_rows(&show(root, "docs/integration/DECISIONS.md").unwrap_or_default());
    let decisions = format!(
        "{banner}# Decisions\n\n## Decisions\n\n{}",
        table(
            &[
                "Id",
                "Date",
                "Decision",
                "Reason",
                "Alternatives turned down",
                "Reversal cost"
            ],
            &dec_rows
        )
    );

    // STATE.md
    let agents = show(root, "AGENTS.md").unwrap_or_default();
    let sentence = agents
        .split("\n\n")
        .map(str::trim)
        .find(|p| !p.is_empty() && !p.starts_with('#'))
        .and_then(|p| p.lines().next())
        .unwrap_or("");
    let mut state = format!("{banner}# State\n\n## The project in one sentence\n\n{sentence}\n\n");
    if let Some(l) = agents.lines().find(|l| l.starts_with("It is pre-alpha")) {
        state.push_str(&format!("## Done looks like\n\n{l}\n\n"));
    }
    state.push_str(&format!(
        "Last checked at commit `{sha12}` on branch `main` ({date})\n\n## What works now\n\n"
    ));
    let paths = StatePaths::from_env(root);
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let board = loopboard::build(&paths, &labs, Vec::new(), now_unix);
    // First, because the page shows only the first five rows of this table.
    let mut state_rows: Vec<Vec<String>> = match &checkout {
        Some(Checkout::Left(head, why)) => vec![vec![
            "bench checkout".into(),
            format!("blocked: left at `{head}`, not moved to origin/main: {why}"),
            format!("the feed's checkout step, in {}", root.display()),
        ]],
        _ => Vec::new(),
    };
    state_rows.extend(board.labs.iter().map(|r| match &r.presence {
        Some(e) => vec![
            format!("{} ({})", r.name, r.lab.client),
            if e.row.doing.trim().is_empty() {
                e.state()
            } else {
                e.row.doing.clone()
            },
            format!("presence, {}, {}", e.source, e.state()),
        ],
        None => vec![
            format!("{} ({})", r.name, r.lab.client),
            "no presence".into(),
            "presence, none, -".into(),
        ],
    }));
    let count = |s: &str| items.iter().filter(|i| i.status == s).count();
    state_rows.push(vec![
        "roadmap".into(),
        format!(
            "{} ready, {} held, {} decision, {} done",
            count("ready"),
            count("held"),
            count("decision"),
            count("done")
        ),
        "docs/roadmap/items on origin/main".into(),
    ]);
    state.push_str(&table(&["Area", "Status", "How we know"], &state_rows));

    let gates = gates(root, &labs);
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    write_atomic(dir, "PLAN.md", &plan)?;
    write_atomic(dir, "HANDOFF.md", &handoff)?;
    write_atomic(dir, "DECISIONS.md", &decisions)?;
    write_atomic(dir, "STATE.md", &state)?;
    write_atomic(dir, ".gates.jsonl", &gates)?;
    Ok((lab_prs.len(), items.len(), checkout))
}

/// DIR made absolute and resolved through its nearest existing ancestor; `None` on a `..`.
fn resolve(root: &Path, dir: &Path) -> Option<PathBuf> {
    let abs = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        root.join(dir)
    };
    let mut base = abs.clone();
    let mut rest = Vec::new();
    while !base.exists() {
        rest.push(base.file_name()?.to_os_string());
        base = base.parent()?.to_path_buf();
    }
    let mut out = base.canonicalize().ok()?;
    for c in rest.iter().rev() {
        if Path::new(c)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return None;
        }
        out.push(c);
    }
    Some(out)
}

/// `loop board --bench DIR [--every SECONDS]`. Exit 0 when written, 1 on a failed single run,
/// 2 for a usage error or a refused DIR. Only `--every` runs the checkout step.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let (mut dir, mut every) = (None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match (a.as_str(), it.clone().next()) {
            ("--bench", Some(v)) => dir = Some(PathBuf::from(v)),
            ("--every", Some(v)) => match v.parse::<u64>() {
                Ok(n) => every = Some(n),
                Err(_) => {
                    eprintln!("uor-harness: loop board: --every takes whole seconds\n{USAGE}");
                    return 2;
                }
            },
            _ => {
                eprintln!("uor-harness: loop board: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
        }
        it.next();
    }
    let Some(dir) = dir else {
        eprintln!("{USAGE}");
        return 2;
    };
    if every.is_some_and(|n| n < 30) {
        eprintln!("uor-harness: loop board: --every must be at least 30 seconds");
        return 2;
    }
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let Some(dir) = resolve(&root_abs, &dir) else {
        eprintln!(
            "uor-harness: loop board: --bench {} cannot be resolved",
            dir.display()
        );
        return 2;
    };
    if dir.starts_with(&root_abs) && !dir.starts_with(root_abs.join("local")) {
        eprintln!(
            "uor-harness: loop board: --bench {} is inside the repository; use a folder under local/ or outside the repository",
            dir.display()
        );
        return 2;
    }
    let fake = std::env::var_os(BENCH_GH_FAKE_ENV);
    let (board_gh, gh): (Box<dyn BoardGh>, Box<dyn BenchGh>) = match &fake {
        Some(p) => {
            let load = || FakeBenchGh::from_file(Path::new(p));
            match (load(), load()) {
                (Ok(a), Ok(b)) => (Box::new(a), Box::new(b)),
                (Err(e), _) | (_, Err(e)) => {
                    eprintln!("uor-harness: loop board: {e}");
                    return 2;
                }
            }
        }
        None => match std::env::var_os(loopboard::BOARD_GH_FAKE_ENV) {
            Some(p) => match FakeBoardGh::from_file(Path::new(&p)) {
                Ok(f) => (
                    Box::new(f),
                    Box::new(RealBenchGh {
                        root: root_abs.clone(),
                    }),
                ),
                Err(e) => {
                    eprintln!("uor-harness: loop board: {e}");
                    return 2;
                }
            },
            None => (
                Box::new(RealBoardGh {
                    root: root_abs.clone(),
                }),
                Box::new(RealBenchGh {
                    root: root_abs.clone(),
                }),
            ),
        },
    };
    loop {
        match refresh(
            &root_abs,
            &dir,
            board_gh.as_ref(),
            gh.as_ref(),
            every.is_some(),
        ) {
            Ok((p, i, checkout)) => {
                match checkout {
                    Some(Checkout::Moved(from)) => {
                        println!("bench feed: checkout moved from {from} to origin/main")
                    }
                    Some(Checkout::Left(head, why)) => {
                        println!(
                            "bench feed: checkout left at {head}, not moved to origin/main: {why}"
                        )
                    }
                    Some(Checkout::Current) | None => {}
                }
                println!(
                    "bench feed: wrote 5 files at {} ({p} PRs, {i} items)",
                    crate::team::now_iso()
                )
            }
            Err(e) => {
                eprintln!("uor-harness: loop board: bench feed: {e}");
                if every.is_none() {
                    return 1;
                }
            }
        }
        match every {
            Some(n) => std::thread::sleep(std::time::Duration::from_secs(n)),
            None => return 0,
        }
    }
}
