//! `uor-review v1` review records and the review gate, as pure functions over strings.
//!
//! A review record is a PR comment whose first line is `[<lab>] uor-review v1`, followed by
//! `key: value` lines up to the first blank line, then optional free text (the findings):
//!
//! ```text
//! [<lab>] uor-review v1
//! pr: <number>
//! head: <40 lowercase hex>
//! lab: <lab>
//! model: <model id or "unknown">
//! launch: <launch id or "manual">
//! verdict: APPROVE | CHANGES
//! order: docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md
//! context: fresh            (optional)
//! done-when: host: <what was re-run and its result>  |  done-when: gpu: sha256:<64 hex>
//! ```
//!
//! [`gate`] decides whether a PR's records let it merge. A setup PR (one whose ORDER's
//! `Setup row:` names a row of the loop page's build-plan table, and whose item is
//! `infra-loop-harness` or that row's own item) counts only `claude` records while D59 is in
//! force. D59 ends only by an owner switch: the file [`D59_SWITCH`] reading `"d59": "ended"`,
//! with that exact path listed in `docs/harness/owner-paths.json` (`pr-check` reads both at the merge base; `loop review`, `loop next` and the bench at `origin/main`). The
//! claude-18 ORDER proposes the switch file; the owner adds it and lists it. Until both hold,
//! [`d59_in_force`] answers `true`.
//!
//! Nothing here calls git or GitHub; callers read the inputs and pass them in.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The owner's D59 switch file. Proposed by the claude-18 ORDER; added and listed in
/// `docs/harness/owner-paths.json` by the owner.
pub const D59_SWITCH: &str = "docs/harness/d59.json";

/// The text every record's first line carries.
const MARK: &str = "uor-review v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Approve,
    Changes,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Approve => "APPROVE",
            Verdict::Changes => "CHANGES",
        }
    }
}

/// What the reviewer re-ran: a host check with its result, or a GPU evidence digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoneWhen {
    /// The text after `host: `.
    Host(String),
    /// The 64 lowercase hex digits after `gpu: sha256:`.
    Gpu(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub lab: String,
    pub pr: u64,
    pub head: String,
    pub model: String,
    pub launch: String,
    pub verdict: Verdict,
    pub order: String,
    pub fresh_context: bool,
    pub done_when: DoneWhen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// The first line does not contain `uor-review v1`.
    NotARecord,
    /// It claims to be a record but is wrong; the reason says how.
    Malformed(String),
    Record(Record),
}

/// The record as comment text, which [`parse`] reads back to the same record.
pub fn render(r: &Record) -> String {
    let mut s = format!(
        "[{lab}] {MARK}\npr: {pr}\nhead: {head}\nlab: {lab}\nmodel: {model}\nlaunch: {launch}\nverdict: {verdict}\norder: {order}\n",
        lab = r.lab,
        pr = r.pr,
        head = r.head,
        model = r.model,
        launch = r.launch,
        verdict = r.verdict.as_str(),
        order = r.order,
    );
    if r.fresh_context {
        s.push_str("context: fresh\n");
    }
    match &r.done_when {
        DoneWhen::Host(t) => s.push_str(&format!("done-when: host: {t}\n")),
        DoneWhen::Gpu(h) => s.push_str(&format!("done-when: gpu: sha256:{h}\n")),
    }
    s
}

fn re(cell: &'static OnceLock<regex::Regex>, pattern: &str) -> &'static regex::Regex {
    cell.get_or_init(|| regex::Regex::new(pattern).expect("a valid pattern"))
}

fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

const KEYS: &[&str] = &[
    "pr",
    "head",
    "lab",
    "model",
    "launch",
    "verdict",
    "order",
    "context",
    "done-when",
];

/// Reads a comment body. See [`Parsed`] for the three outcomes.
pub fn parse(body: &str) -> Parsed {
    static TAG: OnceLock<regex::Regex> = OnceLock::new();
    static ORDER: OnceLock<regex::Regex> = OnceLock::new();
    let mut lines = body.lines().map(|l| l.strip_suffix('\r').unwrap_or(l));
    let first = lines.next().unwrap_or("");
    let bad = |why: String| Parsed::Malformed(why);
    if !first.contains(MARK) {
        if body.lines().skip(1).any(|l| l.contains(MARK)) {
            return bad(format!("a {MARK} record must start the comment"));
        }
        return Parsed::NotARecord;
    }
    let Some(tag) = re(&TAG, r"^\[([a-z0-9-]+)\] uor-review v1$")
        .captures(first)
        .map(|c| c[1].to_string())
    else {
        return bad(format!(
            "untagged: the first line must be exactly `[<lab>] {MARK}`, got {first:?}"
        ));
    };
    let mut fields: BTreeMap<&str, String> = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            return bad(format!("field line {line:?} is not `key: value`"));
        };
        let key = key.trim();
        let Some(&known) = KEYS.iter().find(|k| **k == key) else {
            return bad(format!("unknown field {key:?}"));
        };
        if fields.insert(known, value.trim().to_string()).is_some() {
            return bad(format!("duplicated field {key:?}"));
        }
    }
    for key in KEYS.iter().filter(|k| **k != "context") {
        if !fields.contains_key(key) {
            return bad(format!("missing field {key:?}"));
        }
    }
    let lab = fields["lab"].clone();
    if lab != tag {
        return bad(format!(
            "the tag [{tag}] differs from the lab: field {lab:?}"
        ));
    }
    let Ok(pr) = fields["pr"].parse::<u64>() else {
        return bad(format!("bad pr {:?}: not a PR number", fields["pr"]));
    };
    let head = fields["head"].clone();
    if !is_hex(&head, 40) {
        return bad(format!("bad head {head:?}: not 40 lowercase hex"));
    }
    let verdict = match fields["verdict"].as_str() {
        "APPROVE" => Verdict::Approve,
        "CHANGES" => Verdict::Changes,
        other => return bad(format!("bad verdict {other:?}: not APPROVE or CHANGES")),
    };
    for key in ["model", "launch"] {
        if fields[key].is_empty() {
            return bad(format!("empty field {key:?}"));
        }
    }
    let order = fields["order"].clone();
    if !re(
        &ORDER,
        r"^docs/loop/tasks/[^/\s]+/[a-z0-9-]+-[0-9]+/ORDER\.md$",
    )
    .is_match(&order)
    {
        return bad(format!(
            "bad order {order:?}: not docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md"
        ));
    }
    let fresh_context = match fields.get("context").map(String::as_str) {
        None => false,
        Some("fresh") => true,
        Some(other) => return bad(format!("bad context {other:?}: only `fresh` is allowed")),
    };
    let dw = fields["done-when"].as_str();
    let done_when = if let Some(t) = dw.strip_prefix("host:").map(str::trim) {
        if t.is_empty() {
            return bad("bad done-when: `host:` names nothing".to_string());
        }
        DoneWhen::Host(t.to_string())
    } else if let Some(h) = dw
        .strip_prefix("gpu:")
        .map(str::trim)
        .and_then(|g| g.strip_prefix("sha256:"))
        .filter(|h| is_hex(h, 64))
    {
        DoneWhen::Gpu(h.to_string())
    } else {
        return bad(format!(
            "bad done-when {dw:?}: not `host: <what ran>` or `gpu: sha256:<64 hex>`"
        ));
    };
    Parsed::Record(Record {
        lab,
        pr,
        head,
        model: fields["model"].clone(),
        launch: fields["launch"].clone(),
        verdict,
        order,
        fresh_context,
        done_when,
    })
}

/// The first column of the loop page's build-plan table: the table under `## Build plan`
/// whose header row starts `| PR | What it builds |`.
pub fn build_rows(loop_page: &str) -> Vec<String> {
    let mut lines = loop_page
        .lines()
        .skip_while(|l| l.trim_end() != "## Build plan")
        .skip(1);
    let mut rows = Vec::new();
    for line in lines.by_ref() {
        if line.starts_with("## ") {
            return rows;
        }
        if line.starts_with("| PR | What it builds |") {
            break;
        }
    }
    for line in lines {
        let Some(rest) = line.strip_prefix('|') else {
            break;
        };
        let cell = rest.split('|').next().unwrap_or("").trim();
        if cell.is_empty() || cell.bytes().all(|b| b == b'-' || b == b':') {
            continue;
        }
        rows.push(cell.to_string());
    }
    rows
}

/// The item-id prefix of a build-plan row: `6` → `infra-loop-pr06-`, `7a` →
/// `infra-loop-pr07a-`. Rows `A` and `1` have no item of their own (PR 1 made the items for
/// PRs 2–12), so they, and anything not digits with an optional lowercase letter, map to `None`.
pub fn row_item_prefix(row: &str) -> Option<String> {
    static ROW: OnceLock<regex::Regex> = OnceLock::new();
    let c = re(&ROW, r"^([0-9]+)([a-z]?)$").captures(row)?;
    let n: u32 = c[1].parse().ok()?;
    (n >= 2).then(|| format!("infra-loop-pr{n:02}{}-", &c[2]))
}

/// The row an ORDER's `Setup row:` names: a row token (`[0-9]+[a-z]?` or `A`) directly after
/// `Setup row:` and optional spaces or tabs, followed by `.`, whitespace or the end of the line.
/// The label may sit mid-line; prose such as "no `Setup row:` line" or `Setup row: **6**` names
/// no row.
pub fn setup_row(order_text: &str) -> Option<String> {
    static SETUP: OnceLock<regex::Regex> = OnceLock::new();
    re(&SETUP, r"(?m)Setup row:[ \t]*([0-9]+[a-z]?|A)(?:[.\s]|$)")
        .captures(order_text)
        .map(|c| c[1].to_string())
}

/// A setup PR: the ORDER's `Setup row:` names one of `rows`, and its item is
/// `infra-loop-harness` or starts with that row's item prefix.
pub fn is_setup_pr(setup_row: Option<&str>, rows: &[String], order_item: &str) -> bool {
    let Some(row) = setup_row else {
        return false;
    };
    if !rows.iter().any(|r| r == row) {
        return false;
    }
    order_item == "infra-loop-harness"
        || row_item_prefix(row).is_some_and(|p| order_item.starts_with(&p))
}

/// Why no record counts on this PR: its ORDER's item is a loop setup item (`infra-loop-…`) but
/// the ORDER names no valid build-plan row for it, so it can be neither a setup PR nor an
/// ordinary one until the ORDER is fixed.
fn bad_loop_item(input: &GateInput, setup_pr: bool) -> Option<String> {
    (!setup_pr && input.order_item.starts_with("infra-loop-")).then(|| {
        format!(
            "fail: a loop setup item's ORDER needs a valid `Setup row:` naming its build-plan row (item {}, row {})",
            input.order_item,
            input.setup_row.unwrap_or("none")
        )
    })
}

/// D59 is in force unless both: `owner_paths_json` lists exactly [`D59_SWITCH`], and
/// `switch_text` is JSON with `"d59": "ended"`. Anything missing or unreadable keeps it in force.
pub fn d59_in_force(owner_paths_json: &str, switch_text: Option<&str>) -> bool {
    let listed = serde_json::from_str::<serde_json::Value>(owner_paths_json)
        .ok()
        .and_then(|v| v.get("paths")?.as_array().cloned())
        .is_some_and(|paths| {
            paths
                .iter()
                .any(|p| p.get("path").and_then(|s| s.as_str()) == Some(D59_SWITCH))
        });
    if !listed {
        return true;
    }
    let ended = switch_text
        .and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok())
        .is_some_and(|v| v.get("d59").and_then(|s| s.as_str()) == Some("ended"));
    !ended
}

/// Everything the gate reads. `vendors` maps each registered lab to its vendor; `comments` are
/// the PR's comment bodies in posting order.
pub struct GateInput<'a> {
    pub pr: u64,
    pub head: &'a str,
    /// The path of the ORDER this PR adds; a record's `order` must equal it.
    pub order_path: &'a str,
    pub order_lab: &'a str,
    pub order_item: &'a str,
    pub setup_row: Option<&'a str>,
    pub rows: &'a [String],
    pub vendors: &'a BTreeMap<String, String>,
    pub d59_in_force: bool,
    pub doc_only: bool,
    pub comments: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    pub pass: bool,
    pub setup_pr: bool,
    pub details: Vec<String>,
}

/// Why a well-formed record does not count, or `None` when it does.
fn refusal(input: &GateInput, setup_pr: bool, r: &Record) -> Option<String> {
    if r.pr != input.pr {
        return Some(format!("it reviews PR {}, not PR {}", r.pr, input.pr));
    }
    if r.head != input.head {
        return Some(format!("not the current head {}", input.head));
    }
    if r.order != input.order_path {
        return Some(format!(
            "order {} is not this PR's ORDER {}",
            r.order, input.order_path
        ));
    }
    let Some(lab_vendor) = input.vendors.get(&r.lab) else {
        return Some(format!("lab {} is not a registered lab", r.lab));
    };
    if setup_pr && input.d59_in_force {
        if r.lab != "claude" {
            return Some(format!(
                "a setup PR under D59 counts only claude records, not {}",
                r.lab
            ));
        }
        // D59: Claude's fresh-context reviewers.
        return (!r.fresh_context)
            .then(|| "a claude record on a setup PR under D59 needs `context: fresh`".to_string());
    }
    let order_vendor = input
        .vendors
        .get(input.order_lab)
        .map(String::as_str)
        .unwrap_or("");
    let other_vendor =
        !lab_vendor.is_empty() && !order_vendor.is_empty() && lab_vendor != order_vendor;
    if other_vendor {
        return None;
    }
    if input.doc_only && r.lab == input.order_lab {
        return (!r.fresh_context)
            .then(|| "a same-lab record on a doc-only PR needs `context: fresh`".to_string());
    }
    Some(format!(
        "same vendor: lab {} (vendor {lab_vendor:?}) is not of another vendor than the ORDER lab {} (vendor {order_vendor:?}) (D46)",
        r.lab, input.order_lab
    ))
}

/// Passes when a counted record is an APPROVE and the last counted record is an APPROVE.
/// `details` names every record, malformed ones included, and why it counted or not.
pub fn gate(input: &GateInput) -> GateOutcome {
    let setup_pr = is_setup_pr(input.setup_row, input.rows, input.order_item);
    let bad_item = bad_loop_item(input, setup_pr);
    let mut details = Vec::new();
    let mut last: Option<Verdict> = None;
    for (i, body) in input.comments.iter().enumerate() {
        let n = i + 1;
        match parse(body) {
            Parsed::NotARecord => {}
            Parsed::Malformed(why) => {
                details.push(format!("comment {n}: refused, malformed record: {why}"))
            }
            Parsed::Record(r) => {
                let seen = format!(
                    "comment {n}: [{}] {} at {} on PR {}",
                    r.lab,
                    r.verdict.as_str(),
                    r.head,
                    r.pr
                );
                if bad_item.is_some() {
                    details.push(format!(
                        "{seen}: not counted, the ORDER names no valid setup row"
                    ));
                    continue;
                }
                match refusal(input, setup_pr, &r) {
                    Some(why) => details.push(format!("{seen}: not counted, {why}")),
                    None => {
                        details.push(format!("{seen}: counted"));
                        last = Some(r.verdict);
                    }
                }
            }
        }
    }
    let pass = last == Some(Verdict::Approve);
    details.push(match (bad_item, last) {
        (Some(why), _) => why,
        (None, Some(Verdict::Approve)) => {
            format!("pass: a counted APPROVE at {} stands", input.head)
        }
        (None, Some(Verdict::Changes)) => "fail: the last counted record is CHANGES".to_string(),
        (None, None) => format!("fail: no counted record at {}", input.head),
    });
    GateOutcome {
        pass,
        setup_pr,
        details,
    }
}

/// The counted records, in posting order: `(lab, verdict)` of each record [`gate`] counts.
pub fn counted(input: &GateInput) -> Vec<(String, Verdict)> {
    let setup_pr = is_setup_pr(input.setup_row, input.rows, input.order_item);
    if bad_loop_item(input, setup_pr).is_some() {
        return Vec::new();
    }
    input
        .comments
        .iter()
        .filter_map(|body| match parse(body) {
            Parsed::Record(r) if refusal(input, setup_pr, &r).is_none() => Some((r.lab, r.verdict)),
            _ => None,
        })
        .collect()
}

/// The index in `comments` of the last counted record when it is an APPROVE; `None` when the
/// last counted record is CHANGES or nothing counts. `loop queue` refuses any comment after it
/// that it cannot attribute to a lab other than the PM.
pub fn last_counted_approve(input: &GateInput) -> Option<usize> {
    let setup_pr = is_setup_pr(input.setup_row, input.rows, input.order_item);
    if bad_loop_item(input, setup_pr).is_some() {
        return None;
    }
    let mut last = None;
    for (i, body) in input.comments.iter().enumerate() {
        if let Parsed::Record(r) = parse(body) {
            if refusal(input, setup_pr, &r).is_none() {
                last = Some((i, r.verdict));
            }
        }
    }
    match last {
        Some((i, Verdict::Approve)) => Some(i),
        _ => None,
    }
}

/// Whether a record from `lab` at this PR and head would count. The record is taken to carry
/// `context: fresh`, which any reviewer may state, so on a doc-only PR the ORDER lab may review,
/// and on a setup PR under D59 claude may. A loop item whose ORDER names no valid setup row is
/// owed no review until the ORDER is fixed.
pub fn may_review(input: &GateInput, lab: &str) -> bool {
    let setup_pr = is_setup_pr(input.setup_row, input.rows, input.order_item);
    if bad_loop_item(input, setup_pr).is_some() {
        return false;
    }
    let r = Record {
        lab: lab.to_string(),
        pr: input.pr,
        head: input.head.to_string(),
        model: String::new(),
        launch: String::new(),
        verdict: Verdict::Approve,
        order: input.order_path.to_string(),
        fresh_context: true,
        done_when: DoneWhen::Host(String::new()),
    };
    refusal(input, setup_pr, &r).is_none()
}

// ---------------------------------------------------------------------------------------------
// The `uor-harness loop review` command. Everything above is pure; this section reads git and
// GitHub (through [`ReviewGh`]) and posts one comment.
// ---------------------------------------------------------------------------------------------

use std::path::{Path, PathBuf};

const USAGE: &str = "usage: uor-harness loop review --pr <n> --verdict APPROVE|CHANGES (--host <text> | --gpu <64 hex>) [--context fresh] [--model <id>] [--launch <id>] [--body-file <path>] [--dry-run]\n       uor-harness loop review --check --pr <n>";

/// TEST-ONLY: a JSON file standing in for `gh`: `{"pr": {"headRefOid", "files": [path]},
/// "files": {path: text}, "comments": [body], "failPost": bool}`. Each `post_comment` is
/// appended as a JSON line to `<file>.calls.jsonl`.
pub const REVIEW_GH_FAKE_ENV: &str = "UOR_LOOP_REVIEW_GH_FAKE";

/// A PR's head and every changed path.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ReviewPr {
    #[serde(rename = "headRefOid")]
    pub head_sha: String,
    #[serde(default)]
    pub files: Vec<String>,
}

/// Every GitHub call `loop review` makes.
pub trait ReviewGh {
    fn pr(&self, n: u64) -> Result<ReviewPr, String>;
    /// The text of `path` at PR `n`'s head.
    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String>;
    /// The comment bodies on PR `n`, oldest first, every page read.
    fn comments(&self, n: u64) -> Result<Vec<String>, String>;
    fn post_comment(&self, n: u64, body: &str) -> Result<(), String>;
}

/// `gh`, run from the repository root.
pub struct RealReviewGh {
    pub root: PathBuf,
}

impl ReviewGh for RealReviewGh {
    fn pr(&self, n: u64) -> Result<ReviewPr, String> {
        use crate::loopnext::NextGh as _;
        let text = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "view", &n.to_string(), "--json", "headRefOid,files"],
        )?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("gh pr view printed unreadable JSON: {e}"))?;
        let head_sha = v
            .get("headRefOid")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let mut files: Vec<String> = v
            .get("files")
            .and_then(|f| f.as_array())
            .into_iter()
            .flatten()
            .filter_map(|f| f.get("path")?.as_str().map(str::to_string))
            .collect();
        // The listed files stop at 100; a PR that reaches it may have more, so read every page.
        if files.len() >= 100 {
            files = crate::loopnext::RealNextGh {
                root: self.root.clone(),
            }
            .pr_files(n)?;
        }
        Ok(ReviewPr { head_sha, files })
    }

    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String> {
        use crate::loopnext::NextGh as _;
        crate::loopnext::RealNextGh {
            root: self.root.clone(),
        }
        .file_at_pr(n, path)
    }

    fn comments(&self, n: u64) -> Result<Vec<String>, String> {
        use crate::loopprcheck::PrGh as _;
        Ok(crate::loopprcheck::RealPrGh {
            root: self.root.clone(),
        }
        .comments(n)?
        .into_iter()
        .map(|c| c.body)
        .collect())
    }

    fn post_comment(&self, n: u64, body: &str) -> Result<(), String> {
        let tmp = std::env::temp_dir().join(format!(
            "uor-loop-review-{}-{n}-{}.md",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&tmp, body).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
        let tmp_s = tmp.to_string_lossy().into_owned();
        let out = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "comment", &n.to_string(), "--body-file", &tmp_s],
        );
        let _ = std::fs::remove_file(&tmp);
        out.map(|_| ())
    }
}

/// The in-memory stand-in for `gh`, read from the file [`REVIEW_GH_FAKE_ENV`] names.
#[derive(serde::Deserialize, Default)]
pub struct FakeReviewGh {
    #[serde(default)]
    pr: ReviewPr,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    comments: Vec<String>,
    #[serde(default, rename = "failPost")]
    fail_post: bool,
    #[serde(skip)]
    calls: PathBuf,
}

impl FakeReviewGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {REVIEW_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        let mut f: Self = serde_json::from_str(&text).map_err(|e| {
            format!(
                "{REVIEW_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })?;
        f.calls = PathBuf::from(format!("{}.calls.jsonl", path.display()));
        Ok(f)
    }
}

impl ReviewGh for FakeReviewGh {
    fn pr(&self, _n: u64) -> Result<ReviewPr, String> {
        Ok(self.pr.clone())
    }
    fn file_at_pr(&self, n: u64, path: &str) -> Result<String, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("fake PR #{n} has no text for {path}"))
    }
    fn comments(&self, _n: u64) -> Result<Vec<String>, String> {
        Ok(self.comments.clone())
    }
    fn post_comment(&self, n: u64, body: &str) -> Result<(), String> {
        if self.fail_post {
            return Err(format!("fake PR #{n}: posting the comment failed"));
        }
        use std::io::Write as _;
        let line = serde_json::json!({"call": "post_comment", "pr": n, "body": body});
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

/// What `origin/main` says about every PR: the build-plan rows, each lab's vendor and whether
/// D59 is in force. Shared by `loop review` and `loop next`; reads the local `origin/main`.
pub(crate) struct MainContext {
    pub rows: Vec<String>,
    pub vendors: BTreeMap<String, String>,
    pub d59_in_force: bool,
}

pub(crate) fn main_context(root: &Path) -> Result<MainContext, String> {
    let main = |path: &str| git(root, &["show", &format!("origin/main:{path}")]);
    let rows = build_rows(&main("docs/loop/README.md")?);
    let vendors = crate::labs::parse_labs(&main("docs/harness/labs.json")?)?
        .into_iter()
        .map(|(lab, l)| (lab, l.vendor))
        .collect();
    // Unreadable owner paths or a missing switch keep D59 in force.
    let d59 = d59_in_force(
        &main("docs/harness/owner-paths.json").unwrap_or_default(),
        main(D59_SWITCH).ok().as_deref(),
    );
    Ok(MainContext {
        rows,
        vendors,
        d59_in_force: d59,
    })
}

/// A doc-only PR: it changes at least one path, and every path is `docs` or `generated`.
pub fn doc_only<'p>(paths: impl IntoIterator<Item = &'p str>) -> bool {
    let mut any = false;
    for p in paths {
        if !matches!(crate::loopprcheck::size_class(p), "docs" | "generated") {
            return false;
        }
        any = true;
    }
    any
}

/// Everything about the PR the gate needs except its comments.
struct Context {
    head: String,
    order: String,
    order_lab: String,
    order_item: String,
    setup_row: Option<String>,
    main: MainContext,
    doc_only: bool,
}

fn context(root: &Path, gh: &dyn ReviewGh, n: u64) -> Result<Context, String> {
    git(root, &["fetch", "-q", "origin", "main"])?;
    let pr = gh.pr(n)?;
    let (order, text, order_lab) =
        crate::loopnext::added_order(root, n, &pr.files, &|p| gh.file_at_pr(n, p))
            .map_err(crate::loopnext::OrderErr::note)?;
    let main = main_context(root)?;
    let doc_only = doc_only(pr.files.iter().map(String::as_str));
    Ok(Context {
        head: pr.head_sha,
        order,
        order_lab,
        order_item: crate::loopclaim::order_item(&text).unwrap_or_default(),
        setup_row: setup_row(&text),
        main,
        doc_only,
    })
}

fn run_gate(pr: u64, c: &Context, comments: &[String]) -> GateOutcome {
    gate(&GateInput {
        pr,
        head: &c.head,
        order_path: &c.order,
        order_lab: &c.order_lab,
        order_item: &c.order_item,
        setup_row: c.setup_row.as_deref(),
        rows: &c.main.rows,
        vendors: &c.main.vendors,
        d59_in_force: c.main.d59_in_force,
        doc_only: c.doc_only,
        comments,
    })
}

/// The parsed command line.
#[derive(Default)]
struct Args {
    pr: Option<u64>,
    check: bool,
    verdict: Option<Verdict>,
    host: Option<String>,
    gpu: Option<String>,
    fresh: bool,
    model: Option<String>,
    launch: Option<String>,
    body_file: Option<String>,
    dry_run: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--pr" => {
                let v = value()?;
                a.pr = Some(v.parse().map_err(|_| format!("bad --pr {v:?}"))?);
            }
            "--check" => a.check = true,
            "--verdict" => {
                a.verdict = Some(match value()?.as_str() {
                    "APPROVE" => Verdict::Approve,
                    "CHANGES" => Verdict::Changes,
                    v => return Err(format!("bad --verdict {v:?}: APPROVE or CHANGES")),
                })
            }
            "--host" => a.host = Some(value()?),
            "--gpu" => a.gpu = Some(value()?),
            "--context" => match value()?.as_str() {
                "fresh" => a.fresh = true,
                v => return Err(format!("bad --context {v:?}: only `fresh`")),
            },
            "--model" => a.model = Some(value()?),
            "--launch" => a.launch = Some(value()?),
            "--body-file" => a.body_file = Some(value()?),
            "--dry-run" => a.dry_run = true,
            other => return Err(format!("unexpected argument {other:?}")),
        }
    }
    if a.pr.is_none() {
        return Err("--pr <n> is required".to_string());
    }
    let writes = a.verdict.is_some()
        || a.host.is_some()
        || a.gpu.is_some()
        || a.fresh
        || a.model.is_some()
        || a.launch.is_some()
        || a.body_file.is_some()
        || a.dry_run;
    if a.check && writes {
        return Err("--check takes only --pr".to_string());
    }
    if !a.check {
        if a.verdict.is_none() {
            return Err("give --check, or --verdict to write a record".to_string());
        }
        if a.host.is_some() == a.gpu.is_some() {
            return Err("give exactly one of --host and --gpu".to_string());
        }
    }
    Ok(a)
}

/// `flag`, else the non-empty environment variable `env`, else `default`.
fn or_env(flag: &Option<String>, env: &str, default: &str) -> String {
    flag.clone()
        .or_else(|| std::env::var(env).ok().filter(|v| !v.trim().is_empty()))
        .unwrap_or_else(|| default.to_string())
}

/// `loop review ...`: exit 0 posted (or printed, or the gate passes), 1 refused (the record would
/// not count, or the gate fails), 2 cannot run (usage, no lab, no ORDER, git or `gh` error).
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("uor-harness: loop review: {e}\n{USAGE}");
            return 2;
        }
    };
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop review: set UOR_LAB or pass --lab\n{USAGE}");
        return 2;
    }
    let gh: Box<dyn ReviewGh> = match std::env::var_os(REVIEW_GH_FAKE_ENV) {
        Some(p) => match FakeReviewGh::from_file(Path::new(&p)) {
            Ok(f) => {
                println!(
                    "FAKE GitHub: {REVIEW_GH_FAKE_ENV}={} (tests only)",
                    p.to_string_lossy()
                );
                Box::new(f)
            }
            Err(e) => {
                eprintln!("uor-harness: loop review: {e}");
                return 2;
            }
        },
        None => Box::new(RealReviewGh {
            root: root.to_path_buf(),
        }),
    };
    let n = a.pr.unwrap_or_default();
    let c = match context(root, gh.as_ref(), n) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uor-harness: loop review: {e}");
            return 2;
        }
    };
    if a.check {
        let comments = match gh.comments(n) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("uor-harness: loop review: {e}");
                return 2;
            }
        };
        let out = run_gate(n, &c, &comments);
        for d in &out.details {
            println!("{d}");
        }
        return if out.pass { 0 } else { 1 };
    }
    let record = Record {
        lab: lab.to_string(),
        pr: n,
        head: c.head.clone(),
        model: or_env(&a.model, "UOR_LOOP_MODEL", "unknown"),
        launch: or_env(&a.launch, "UOR_LOOP_LAUNCH", "manual"),
        verdict: a.verdict.unwrap_or(Verdict::Changes),
        order: c.order.clone(),
        fresh_context: a.fresh,
        done_when: match (&a.host, &a.gpu) {
            (Some(h), _) => DoneWhen::Host(h.clone()),
            (None, g) => DoneWhen::Gpu(g.clone().unwrap_or_default()),
        },
    };
    let mut body = render(&record);
    // The record must read back as itself: a bad --gpu, a newline in --host and the like.
    match parse(&body) {
        Parsed::Record(r) if r == record => {}
        Parsed::Malformed(why) => {
            eprintln!("uor-harness: loop review: the record would be malformed: {why}");
            return 2;
        }
        _ => {
            eprintln!("uor-harness: loop review: the record does not read back as written");
            return 2;
        }
    }
    let out = run_gate(n, &c, std::slice::from_ref(&body));
    if !out.details.iter().any(|d| d.ends_with(": counted")) {
        eprintln!("uor-harness: loop review: refused, this record would not count; nothing posted");
        for d in &out.details {
            eprintln!("  {d}");
        }
        return 1;
    }
    if let Some(path) = &a.body_file {
        match std::fs::read_to_string(path) {
            Ok(t) => {
                body.push('\n');
                body.push_str(&t);
            }
            Err(e) => {
                eprintln!("uor-harness: loop review: cannot read --body-file {path}: {e}");
                return 2;
            }
        }
    }
    if a.dry_run {
        print!("{body}");
        return 0;
    }
    if let Err(e) = gh.post_comment(n, &body) {
        eprintln!("uor-harness: loop review: {e}");
        return 2;
    }
    println!(
        "review: posted {} on PR #{n} at {}",
        record.verdict.as_str(),
        record.head
    );
    0
}
