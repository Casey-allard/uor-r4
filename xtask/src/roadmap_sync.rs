//! `cargo xtask roadmap sync [--apply] [--repo OWNER/NAME]`: one GitHub issue
//! per roadmap item, kept current from `docs/roadmap/items/<id>.md`.
//!
//! Without `--apply` this only reads GitHub (`gh issue list`, `gh issue view`,
//! `gh label list`, `gh api` GETs) and prints the plan; nothing is written,
//! locally or remotely. With `--apply` it creates missing labels, creates and
//! updates issues, refreshes bodies whose "Depends on" line gained numbers,
//! rewrites the children checklist in the M1..M8 milestone issues
//! (#2029..#2036), then writes `docs/roadmap/issues.json` and re-renders
//! `docs/roadmap/README.md`. A plan with an error (an item whose relative link
//! climbs above the repository root) stops the run before any write.
//!
//! An item's issue is the number in `issues.json`, else the open or closed
//! `roadmap:item` issue whose body holds `<!-- roadmap:id=<id> -->`.
//!
//! Label ownership: the labels this tool manages (`roadmap:item`, `area:*`,
//! `help wanted`, `good first issue`, `status:held`, `owner-decision`) follow
//! the item file and are added or removed to match it. Every other label, and
//! every assignee a person added, is kept ([`managed_label`] is the list).
//!
//! Problems that need a person are warnings, never writes: an issue whose
//! marker id has no item file, one id on two issues (the lowest number is
//! used), a milestone body with malformed children markers (left alone), an
//! `issues.json` number GitHub cannot show. A milestone body is spliced raw, so
//! its bytes outside the children markers (CRLF included) are never changed.
//!
//! All GitHub access goes through the [`Gh`] trait: [`GhCli`] spawns
//! `gh`, and the tests use an in-memory fake. Planning is a pure function.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::roadmap::{self, Item, AREAS};

const DEFAULT_REPO: &str = "UOR-Foundation/uor-r4";
const ITEMS_DIR: &str = "docs/roadmap/items";
const ITEM_LABEL: &str = "roadmap:item";
const CHILDREN_START: &str = "<!-- roadmap:children:start -->";
const CHILDREN_END: &str = "<!-- roadmap:children:end -->";
/// M1..M8 tracking issues, in milestone order.
const MILESTONE_ISSUES: [u64; 8] = [2029, 2030, 2031, 2032, 2033, 2034, 2035, 2036];
/// GitHub milestone number of M1; M2..M8 follow consecutively.
const FIRST_MILESTONE_NUMBER: u64 = 22;

type Issues = BTreeMap<String, u64>;

/// One GitHub issue as `gh --json number,title,body,labels,milestone,state,assignees` gives it.
#[derive(Debug, Clone, PartialEq)]
pub struct GhIssue {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    /// Milestone title, if any.
    pub milestone: Option<String>,
    /// `OPEN` or `CLOSED`.
    pub state: String,
    pub assignees: Vec<String>,
}

/// A new issue to create.
#[derive(Debug, Clone, PartialEq)]
pub struct Create {
    pub id: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub assignee: Option<String>,
    /// The item is done: close the issue right after creating it.
    pub close: bool,
}

/// Changes to one existing issue; `None` / empty means leave as is.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Update {
    pub id: String,
    pub number: u64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    /// `Some(None)` removes the milestone.
    pub milestone: Option<Option<String>>,
    pub add_assignee: Option<String>,
}

impl Update {
    fn fields(&self) -> Vec<String> {
        let mut f = Vec::new();
        if self.title.is_some() {
            f.push("title".to_string());
        }
        if self.body.is_some() {
            f.push("body".to_string());
        }
        if !self.add_labels.is_empty() || !self.remove_labels.is_empty() {
            let mut l: Vec<String> = self.add_labels.iter().map(|a| format!("+{a}")).collect();
            l.extend(self.remove_labels.iter().map(|r| format!("-{r}")));
            f.push(format!("labels ({})", l.join(" ")));
        }
        if let Some(m) = &self.milestone {
            f.push(format!("milestone ({})", m.as_deref().unwrap_or("none")));
        }
        if let Some(a) = &self.add_assignee {
            f.push(format!("assignee (+{a})"));
        }
        f
    }
}

/// Everything GitHub holds that the plan depends on.
#[derive(Debug, Clone, Default)]
pub struct Remote {
    pub labels: Vec<String>,
    /// GitHub milestone number -> title.
    pub milestones: BTreeMap<u64, String>,
    /// Every `roadmap:item` issue, plus any `issues.json` issue that lacks the label.
    pub issues: Vec<GhIssue>,
    /// Milestone tracking issue number -> body.
    pub milestone_bodies: BTreeMap<u64, String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plan {
    pub labels: Vec<String>,
    pub creates: Vec<Create>,
    pub updates: Vec<Update>,
    /// (id, number) of open issues whose item is done.
    pub closes: Vec<(String, u64)>,
    /// (milestone issue number, new body).
    pub checklists: Vec<(u64, String)>,
    pub warnings: Vec<String>,
    /// Items that cannot be planned; `run` stops before any write when non-empty.
    pub errors: Vec<String>,
    /// id -> number for every item that already has an issue.
    pub numbers: Issues,
}

impl Plan {
    fn close_count(&self) -> usize {
        self.closes.len() + self.creates.iter().filter(|c| c.close).count()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
            && self.creates.is_empty()
            && self.updates.is_empty()
            && self.closes.is_empty()
            && self.checklists.is_empty()
    }
}

/// GitHub access. Read methods are used by the dry run; the rest only under `--apply`.
pub trait Gh {
    fn list_labels(&self) -> Result<Vec<String>, String>;
    fn list_milestones(&self) -> Result<BTreeMap<u64, String>, String>;
    /// Every issue labelled `roadmap:item`, open or closed.
    fn list_issues(&self) -> Result<Vec<GhIssue>, String>;
    fn view_issue(&self, number: u64) -> Result<GhIssue, String>;
    fn create_label(&self, name: &str, color: &str, description: &str) -> Result<(), String>;
    /// Returns the new issue number.
    fn create_issue(&self, c: &Create) -> Result<u64, String>;
    fn edit_issue(&self, u: &Update) -> Result<(), String>;
    fn close_issue(&self, number: u64) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Pure pieces: links, bodies, labels, checklists.

/// `target` resolved from `docs/roadmap/items/` to a `blob/main` URL; external
/// (`scheme:`) and `#anchor` targets are returned unchanged. A target that
/// climbs above the repository root is an error, never clamped.
fn absolute_link(target: &str, repo: &str) -> Result<String, String> {
    let t = target.trim();
    let scheme = t
        .split_once(':')
        .is_some_and(|(s, _)| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic()));
    if t.is_empty() || t.starts_with('#') || scheme {
        return Ok(target.to_string());
    }
    let (path, fragment) = match t.split_once('#') {
        Some((p, f)) => (p, format!("#{f}")),
        None => (t, String::new()),
    };
    let mut parts: Vec<&str> = if path.starts_with('/') {
        Vec::new()
    } else {
        ITEMS_DIR.split('/').collect()
    };
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(format!(
                        "link `{}` climbs above the repository root",
                        target.trim()
                    ));
                }
            }
            s => parts.push(s),
        }
    }
    Ok(format!(
        "https://github.com/{repo}/blob/main/{}{fragment}",
        parts.join("/")
    ))
}

/// Every Markdown link target `](target)` in `text` made absolute; the first
/// target that climbs above the repository root is the error.
pub fn rewrite_links(text: &str, repo: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..at + 2]);
        rest = &rest[at + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        out.push_str(&absolute_link(&rest[..end], repo)?);
        rest = &rest[end..];
    }
    out.push_str(rest);
    Ok(out)
}

fn marker(id: &str) -> String {
    format!("<!-- roadmap:id={id} -->")
}

/// The issue body for `item`; `numbers` turns dependencies into `#N`. An error
/// names the item and the link that climbs above the repository root.
pub fn issue_body(item: &Item, numbers: &Issues, repo: &str) -> Result<String, String> {
    let mut s = format!(
        "{} · {} · open to {} · needs GPU {}\n",
        item.milestone,
        item.status,
        item.open_to,
        if item.needs_gpu { "yes" } else { "no" }
    );
    if !item.depends_on.is_empty() {
        let deps: Vec<String> = item
            .depends_on
            .iter()
            .map(|d| match numbers.get(d) {
                Some(n) => format!("#{n}"),
                None => format!("`{d}`"),
            })
            .collect();
        s.push_str(&format!("\nDepends on: {}\n", deps.join(", ")));
    }
    s.push('\n');
    s.push_str(&rewrite_links(item.body.trim(), repo).map_err(|e| format!("{}: {e}", item.id))?);
    s.push_str("\n\n---\n\n");
    let path = format!("{ITEMS_DIR}/{}.md", item.id);
    s.push_str(&format!(
        "Generated from [{path}](https://github.com/{repo}/blob/main/{path}) by `cargo xtask roadmap sync`. \
         Edit that file in a PR, not this issue.\n\n{}\n",
        marker(&item.id)
    ));
    Ok(s)
}

/// Whether sync owns `label`: it adds and removes these, and leaves every other label alone.
pub fn managed_label(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l == ITEM_LABEL
        || l == "help wanted"
        || l == "good first issue"
        || l == "status:held"
        || l == "owner-decision"
        || AREAS.iter().any(|a| l == format!("area:{a}"))
}

/// The managed labels `item` should carry.
pub fn labels_for(item: &Item) -> BTreeSet<String> {
    let mut l = BTreeSet::from([ITEM_LABEL.to_string(), format!("area:{}", item.area)]);
    if item.open_to == "anyone" && item.status == "ready" {
        l.insert("help wanted".into());
    }
    if item.good_first {
        l.insert("good first issue".into());
    }
    match item.status.as_str() {
        "held" => {
            l.insert("status:held".into());
        }
        "decision" => {
            l.insert("owner-decision".into());
        }
        _ => {}
    }
    l
}

/// (color, description) for a label sync may need to create.
fn label_spec(name: &str) -> (&'static str, String) {
    match name {
        ITEM_LABEL => (
            "1d76db",
            "One roadmap item, generated from docs/roadmap/items".into(),
        ),
        "help wanted" => ("008672", "Extra attention is needed".into()),
        "good first issue" => ("7057ff", "Good for newcomers".into()),
        "status:held" => ("e99695", "Held until the owner releases it".into()),
        "owner-decision" => ("d93f0b", "Waits on an owner decision".into()),
        other => (
            "c5def5",
            format!(
                "Roadmap area: {}",
                other.strip_prefix("area:").unwrap_or(other)
            ),
        ),
    }
}

fn milestone_index(milestone: &str) -> Option<usize> {
    milestone
        .strip_prefix('M')
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|n| (1..=8).contains(n))
}

fn milestone_title(item: &Item, remote: &Remote) -> Option<String> {
    milestone_index(&item.milestone)
        .and_then(|k| {
            remote
                .milestones
                .get(&(FIRST_MILESTONE_NUMBER + k as u64 - 1))
        })
        .cloned()
}

/// The checklist lines for milestone `m` (`M1`..`M8`), sorted as `render` sorts.
pub fn children_lines(items: &[Item], m: &str, numbers: &Issues) -> String {
    let mut rows: Vec<&Item> = items.iter().filter(|i| i.milestone == m).collect();
    rows.sort_by(|a, b| {
        (roadmap::status_rank(&a.status), &a.id).cmp(&(roadmap::status_rank(&b.status), &b.id))
    });
    if rows.is_empty() {
        return "_No roadmap items yet._".into();
    }
    rows.iter()
        .map(|i| {
            let mark = if i.status == "done" { "x" } else { " " };
            let at = match numbers.get(&i.id) {
                Some(n) => format!("#{n}"),
                None => format!("`{}`", i.id),
            };
            format!("- [{mark}] {at} {} ({})", i.title, i.status)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `body` with the text between the children markers replaced by `lines`, or
/// a new `## Roadmap items` section appended when the markers are missing.
/// `body` is spliced raw: bytes outside the markers are kept, and the new text
/// uses `\r\n` when the body does. Markers that are doubled, unpaired or in the
/// wrong order are an `Err` naming the problem; the body is then left alone.
pub fn splice_children(body: &str, lines: &str) -> Result<String, String> {
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let lines = lines.replace('\n', eol);
    let starts: Vec<usize> = body.match_indices(CHILDREN_START).map(|(i, _)| i).collect();
    let ends: Vec<usize> = body.match_indices(CHILDREN_END).map(|(i, _)| i).collect();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Ok(format!(
            "{body}{eol}{eol}## Roadmap items{eol}{eol}{CHILDREN_START}{eol}{lines}{eol}{CHILDREN_END}{eol}"
        )),
        ([s], [e]) if s < e => Ok(format!(
            "{}{eol}{lines}{eol}{}",
            &body[..s + CHILDREN_START.len()],
            &body[*e..]
        )),
        ([_], [_]) => Err("the end marker comes before the start marker".into()),
        (s, e) => Err(format!(
            "{} start and {} end markers, expected one of each",
            s.len(),
            e.len()
        )),
    }
}

/// GitHub may drop a trailing newline or turn `\n` into `\r\n`; compare without either.
fn same_body(a: &str, b: &str) -> bool {
    a.replace("\r\n", "\n").trim_end() == b.replace("\r\n", "\n").trim_end()
}

/// The milestone body edits, and a warning for each body that is left alone.
fn checklist_plan(
    items: &[Item],
    numbers: &Issues,
    remote: &Remote,
) -> (Vec<(u64, String)>, Vec<String>) {
    let (mut out, mut warnings) = (Vec::new(), Vec::new());
    for (k, n) in MILESTONE_ISSUES.iter().enumerate() {
        let Some(body) = remote.milestone_bodies.get(n) else {
            continue;
        };
        match splice_children(
            body,
            &children_lines(items, &format!("M{}", k + 1), numbers),
        ) {
            Ok(new) if !same_body(&new, body) => out.push((*n, new)),
            Ok(_) => {}
            Err(why) => warnings.push(format!(
                "milestone #{n}: roadmap children markers are malformed ({why}); fix them by hand"
            )),
        }
    }
    (out, warnings)
}

fn checklist_edits(items: &[Item], numbers: &Issues, remote: &Remote) -> Vec<(u64, String)> {
    checklist_plan(items, numbers, remote).0
}

/// The changes that bring GitHub in line with `items`. Pure: reads only its arguments.
pub fn plan(items: &[Item], remote: &Remote, issues_json: &Issues, repo: &str) -> Plan {
    let mut p = Plan::default();
    let by_number: BTreeMap<u64, &GhIssue> = remote.issues.iter().map(|i| (i.number, i)).collect();
    for item in items {
        let found = issues_json.get(&item.id).copied().or_else(|| {
            let m = marker(&item.id);
            remote
                .issues
                .iter()
                .filter(|i| i.body.contains(&m))
                .map(|i| i.number)
                .min()
        });
        if let Some(n) = found {
            p.numbers.insert(item.id.clone(), n);
        }
    }

    // Marker id -> every issue number carrying it.
    let mut carriers: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
    for i in &remote.issues {
        for id in marker_ids(&i.body) {
            carriers.entry(id).or_default().insert(i.number);
        }
    }
    let known: BTreeSet<&str> = items.iter().map(|i| i.id.as_str()).collect();
    for (id, numbers) in &carriers {
        if known.contains(id.as_str()) {
            if numbers.len() > 1 {
                let list: Vec<String> = numbers.iter().map(|n| format!("#{n}")).collect();
                let (last, head) = list.split_last().expect("two or more");
                p.warnings.push(format!(
                    "roadmap id `{id}` is on {} and {last}; close the duplicate by hand",
                    head.join(", ")
                ));
            }
        } else {
            for n in numbers {
                let issue = &by_number[n];
                let labelled = issue
                    .labels
                    .iter()
                    .any(|l| l.eq_ignore_ascii_case(ITEM_LABEL));
                // A closed issue for a retired item needs nothing more.
                if labelled && issue.state.eq_ignore_ascii_case("open") {
                    p.warnings.push(format!(
                        "#{n} carries roadmap id `{id}` but {ITEMS_DIR}/{id}.md does not exist; close it by hand or restore the file"
                    ));
                }
            }
        }
    }

    let have: BTreeSet<String> = remote
        .labels
        .iter()
        .map(|l| l.to_ascii_lowercase())
        .collect();
    let wanted: BTreeSet<String> = items.iter().flat_map(labels_for).collect();
    p.labels = wanted
        .into_iter()
        .filter(|l| !have.contains(&l.to_ascii_lowercase()))
        .collect();

    for item in items {
        let labels = labels_for(item);
        let milestone = milestone_title(item, remote);
        if milestone_index(&item.milestone).is_some() && milestone.is_none() {
            p.warnings.push(format!(
                "{}: no GitHub milestone found for {}",
                item.id, item.milestone
            ));
        }
        let body = match issue_body(item, &p.numbers, repo) {
            Ok(b) => b,
            Err(e) => {
                p.errors.push(e);
                continue;
            }
        };
        let Some(&number) = p.numbers.get(&item.id) else {
            p.creates.push(Create {
                id: item.id.clone(),
                title: item.title.clone(),
                body,
                labels: labels.into_iter().collect(),
                milestone,
                assignee: item.assignee.clone(),
                close: item.status == "done",
            });
            continue;
        };
        let Some(issue) = by_number.get(&number) else {
            p.warnings.push(format!(
                "{}: issue #{number} from issues.json was not found on GitHub",
                item.id
            ));
            continue;
        };
        let mut u = Update {
            id: item.id.clone(),
            number,
            ..Update::default()
        };
        if issue.title != item.title {
            u.title = Some(item.title.clone());
        }
        if !same_body(&issue.body, &body) {
            u.body = Some(body);
        }
        let current: BTreeSet<String> = issue
            .labels
            .iter()
            .filter(|l| managed_label(l))
            .cloned()
            .collect();
        let lower: BTreeSet<String> = current.iter().map(|l| l.to_ascii_lowercase()).collect();
        u.add_labels = labels
            .iter()
            .filter(|l| !lower.contains(&l.to_ascii_lowercase()))
            .cloned()
            .collect();
        let wanted_lower: BTreeSet<String> =
            labels.iter().map(|l| l.to_ascii_lowercase()).collect();
        u.remove_labels = current
            .into_iter()
            .filter(|l| !wanted_lower.contains(&l.to_ascii_lowercase()))
            .collect();
        if issue.milestone != milestone
            && !(milestone.is_none() && milestone_index(&item.milestone).is_some())
        {
            u.milestone = Some(milestone);
        }
        if let Some(a) = &item.assignee {
            if !issue.assignees.iter().any(|x| x.eq_ignore_ascii_case(a)) {
                u.add_assignee = Some(a.clone());
            }
        }
        if u != (Update {
            id: item.id.clone(),
            number,
            ..Update::default()
        }) {
            p.updates.push(u);
        }
        let open = issue.state.eq_ignore_ascii_case("open");
        if item.status == "done" && open {
            p.closes.push((item.id.clone(), number));
        } else if item.status != "done" && !open {
            p.warnings.push(format!(
                "{}: issue #{number} is closed but the item is `{}`; not reopening it",
                item.id, item.status
            ));
        }
    }
    let (edits, warnings) = checklist_plan(items, &p.numbers, remote);
    p.checklists = edits;
    p.warnings.extend(warnings);
    p
}

/// Every id in a `<!-- roadmap:id=<id> -->` marker in `body`.
fn marker_ids(body: &str) -> Vec<String> {
    const OPEN: &str = "<!-- roadmap:id=";
    let mut ids = Vec::new();
    let mut rest = body;
    while let Some(at) = rest.find(OPEN) {
        rest = &rest[at + OPEN.len()..];
        if let Some(end) = rest.find(" -->") {
            ids.push(rest[..end].to_string());
        }
    }
    ids
}

/// `issues.json` text: pretty, keys sorted, trailing newline.
pub fn issues_json(numbers: &Issues) -> String {
    if numbers.is_empty() {
        return "{}\n".into();
    }
    let rows: Vec<String> = numbers
        .iter()
        .map(|(id, n)| format!("  {}: {n}", Value::String(id.clone())))
        .collect();
    format!("{{\n{}\n}}\n", rows.join(",\n"))
}

pub fn print_plan(p: &Plan, repo: &str, apply: bool) {
    let mode = if apply { "apply" } else { "dry run" };
    println!("roadmap sync ({mode}) against {repo}");
    if p.is_empty() {
        println!("  nothing to change");
    }
    for c in &p.creates {
        println!(
            "  create {}: {} [{}]{}{}",
            c.id,
            c.title,
            c.labels.join(", "),
            c.milestone
                .as_deref()
                .map(|m| format!(" milestone {m}"))
                .unwrap_or_default(),
            if c.close { " (then close: done)" } else { "" }
        );
    }
    for u in &p.updates {
        println!("  update #{} {}: {}", u.number, u.id, u.fields().join(", "));
    }
    for (id, n) in &p.closes {
        println!("  close #{n} {id}: item is done");
    }
    for w in &p.warnings {
        println!("  warning: {w}");
    }
    for e in &p.errors {
        println!("  error: {e}");
    }
    if !p.labels.is_empty() {
        println!("  labels to create: {}", p.labels.join(", "));
    }
    for (n, _) in &p.checklists {
        println!("  milestone checklist #{n}: children list changes");
    }
    println!(
        "roadmap sync plan: {} labels to create, {} issues to create, {} to update, {} to close, {} milestone checklists to change",
        p.labels.len(),
        p.creates.len(),
        p.updates.len(),
        p.close_count(),
        p.checklists.len()
    );
}

// ---------------------------------------------------------------------------
// Reading GitHub and executing a plan.

/// Read everything `plan` needs: labels, milestones, `roadmap:item` issues, any
/// `issues.json` issue missing from that list, and the milestone issue bodies.
pub fn gather(gh: &dyn Gh, issues_json: &Issues) -> Result<Remote, String> {
    let mut remote = Remote {
        labels: gh.list_labels()?,
        milestones: gh.list_milestones()?,
        issues: gh.list_issues()?,
        milestone_bodies: BTreeMap::new(),
    };
    let listed: BTreeSet<u64> = remote.issues.iter().map(|i| i.number).collect();
    for n in issues_json.values().collect::<BTreeSet<_>>() {
        if !listed.contains(n) {
            // A number GitHub cannot show stays out of the remote state, so
            // `plan` warns that it was not found rather than the run aborting.
            match gh.view_issue(*n) {
                Ok(issue) => remote.issues.push(issue),
                Err(e) => {
                    eprintln!("warning: issues.json names #{n}, which could not be read: {e}")
                }
            }
        }
    }
    for n in MILESTONE_ISSUES {
        remote.milestone_bodies.insert(n, gh.view_issue(n)?.body);
    }
    Ok(remote)
}

/// Execute `p` in the documented order; returns id -> number for every item with an issue.
pub fn apply(
    gh: &dyn Gh,
    items: &[Item],
    remote: &Remote,
    p: &Plan,
    repo: &str,
) -> Result<Issues, String> {
    if !p.errors.is_empty() {
        return Err(format!(
            "not applying a plan with errors: {}",
            p.errors.join("; ")
        ));
    }
    for name in &p.labels {
        let (color, description) = label_spec(name);
        gh.create_label(name, color, &description)?;
        println!("  created label {name}");
    }
    let mut numbers = p.numbers.clone();
    // Body each issue holds after this run's writes, for the refresh pass.
    let mut written: BTreeMap<u64, String> = remote
        .issues
        .iter()
        .map(|i| (i.number, i.body.clone()))
        .collect();
    for c in &p.creates {
        let n = gh.create_issue(c)?;
        println!("  created #{n} {}", c.id);
        numbers.insert(c.id.clone(), n);
        written.insert(n, c.body.clone());
        if c.close {
            gh.close_issue(n)?;
            println!("  closed #{n} {} (done)", c.id);
        }
    }
    for u in &p.updates {
        gh.edit_issue(u)?;
        println!(
            "  updated #{} {}: {}",
            u.number,
            u.id,
            u.fields().join(", ")
        );
        if let Some(b) = &u.body {
            written.insert(u.number, b.clone());
        }
    }
    for (id, n) in &p.closes {
        gh.close_issue(*n)?;
        println!("  closed #{n} {id} (done)");
    }
    for item in items {
        let Some(&n) = numbers.get(&item.id) else {
            continue;
        };
        let body = issue_body(item, &numbers, repo)?;
        if written.get(&n).is_some_and(|w| same_body(w, &body)) {
            continue;
        }
        gh.edit_issue(&Update {
            id: item.id.clone(),
            number: n,
            body: Some(body),
            ..Update::default()
        })?;
        println!("  refreshed #{n} {}: depends-on numbers", item.id);
    }
    for (n, body) in checklist_edits(items, &numbers, remote) {
        gh.edit_issue(&Update {
            id: format!("milestone #{n}"),
            number: n,
            body: Some(body),
            ..Update::default()
        })?;
        println!("  updated milestone checklist #{n}");
    }
    Ok(numbers)
}

/// [`Gh`] through the `gh` CLI, one process per call.
pub struct GhCli {
    pub repo: String,
}

impl GhCli {
    fn gh(&self, args: &[&str], stdin: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new("gh");
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            });
        let call = format!("gh {}", args.join(" "));
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("`{call}` did not start: {e}"))?;
        // A failed write (gh exited and closed the pipe) is not returned yet:
        // gh's own stderr says why it exited, and the caller needs that.
        let mut write_failed = None;
        if let Some(text) = stdin {
            let mut pipe = child.stdin.take().expect("stdin is piped");
            write_failed = pipe.write_all(text.as_bytes()).err();
        }
        let out = child
            .wait_with_output()
            .map_err(|e| format!("`{call}` did not finish: {e}"))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if let Some(e) = write_failed {
            return Err(format!(
                "`{call}`: writing the body failed: {e}; gh said ({}): {}",
                out.status,
                stderr.trim()
            ));
        }
        if !out.status.success() {
            return Err(format!(
                "`{call}` failed ({}): {}",
                out.status,
                stderr.trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// A call that creates or changes something on GitHub, followed by a pause:
    /// GitHub asks for at least a second between content writes, and a first
    /// run makes about 150 of them.
    fn write(&self, args: &[&str], stdin: Option<&str>) -> Result<String, String> {
        let out = self.gh(args, stdin)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(out)
    }

    fn json(&self, args: &[&str]) -> Result<Value, String> {
        let text = self.gh(args, None)?;
        serde_json::from_str(&text)
            .map_err(|e| format!("`gh {}` did not return JSON: {e}", args.join(" ")))
    }
}

const ISSUE_FIELDS: &str = "number,title,body,labels,milestone,state,assignees";

fn parse_issue(v: &Value) -> Result<GhIssue, String> {
    let s = |k: &str| v[k].as_str().unwrap_or_default().to_string();
    let names = |k: &str, f: &str| -> Vec<String> {
        v[k].as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x[f].as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(GhIssue {
        number: v["number"]
            .as_u64()
            .ok_or_else(|| format!("gh returned an issue without a number: {v}"))?,
        title: s("title"),
        body: s("body"),
        labels: names("labels", "name"),
        milestone: v["milestone"]["title"].as_str().map(String::from),
        state: s("state"),
        assignees: names("assignees", "login"),
    })
}

impl Gh for GhCli {
    fn list_labels(&self) -> Result<Vec<String>, String> {
        let v = self.json(&[
            "label", "list", "--repo", &self.repo, "--limit", "500", "--json", "name",
        ])?;
        Ok(v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|l| l["name"].as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default())
    }

    fn list_milestones(&self) -> Result<BTreeMap<u64, String>, String> {
        let path = format!("repos/{}/milestones?state=all&per_page=100", self.repo);
        let v = self.json(&["api", &path])?;
        Ok(v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|m| Some((m["number"].as_u64()?, m["title"].as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default())
    }

    fn list_issues(&self) -> Result<Vec<GhIssue>, String> {
        let v = self.json(&[
            "issue",
            "list",
            "--repo",
            &self.repo,
            "--label",
            ITEM_LABEL,
            "--state",
            "all",
            "--limit",
            "1000",
            "--json",
            ISSUE_FIELDS,
        ])?;
        v.as_array()
            .ok_or("gh issue list did not return a JSON array")?
            .iter()
            .map(parse_issue)
            .collect()
    }

    fn view_issue(&self, number: u64) -> Result<GhIssue, String> {
        let n = number.to_string();
        parse_issue(&self.json(&[
            "issue",
            "view",
            &n,
            "--repo",
            &self.repo,
            "--json",
            ISSUE_FIELDS,
        ])?)
    }

    fn create_label(&self, name: &str, color: &str, description: &str) -> Result<(), String> {
        self.write(
            &[
                "label",
                "create",
                name,
                "--repo",
                &self.repo,
                "--color",
                color,
                "--description",
                description,
            ],
            None,
        )
        .map(drop)
    }

    fn create_issue(&self, c: &Create) -> Result<u64, String> {
        let mut args = vec![
            "issue",
            "create",
            "--repo",
            &self.repo,
            "--title",
            &c.title,
            "--body-file",
            "-",
        ];
        for l in &c.labels {
            args.extend(["--label", l.as_str()]);
        }
        if let Some(m) = &c.milestone {
            args.extend(["--milestone", m.as_str()]);
        }
        if let Some(a) = &c.assignee {
            args.extend(["--assignee", a.as_str()]);
        }
        let out = self.write(&args, Some(&c.body))?;
        // gh prints the new issue's URL; its last path segment is the number.
        out.trim()
            .rsplit('/')
            .next()
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("gh issue create printed no issue URL: {}", out.trim()))
    }

    fn edit_issue(&self, u: &Update) -> Result<(), String> {
        let n = u.number.to_string();
        let mut args = vec!["issue", "edit", n.as_str(), "--repo", &self.repo];
        if let Some(t) = &u.title {
            args.extend(["--title", t.as_str()]);
        }
        if u.body.is_some() {
            args.extend(["--body-file", "-"]);
        }
        let add = u.add_labels.join(",");
        if !add.is_empty() {
            args.extend(["--add-label", add.as_str()]);
        }
        let remove = u.remove_labels.join(",");
        if !remove.is_empty() {
            args.extend(["--remove-label", remove.as_str()]);
        }
        match &u.milestone {
            Some(Some(m)) => args.extend(["--milestone", m.as_str()]),
            Some(None) => args.push("--remove-milestone"),
            None => {}
        }
        if let Some(a) = &u.add_assignee {
            args.extend(["--add-assignee", a.as_str()]);
        }
        self.write(&args, u.body.as_deref()).map(drop)
    }

    fn close_issue(&self, number: u64) -> Result<(), String> {
        let n = number.to_string();
        self.write(&["issue", "close", &n, "--repo", &self.repo], None)
            .map(drop)
    }
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut repo = DEFAULT_REPO.to_string();
    let mut do_apply = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--apply" => do_apply = true,
            "--repo" => {
                repo = it
                    .next()
                    .ok_or("--repo needs OWNER/NAME")?
                    .clone();
            }
            other => {
                return Err(format!(
                    "roadmap sync: unknown argument `{other}`; usage: sync [--apply] [--repo OWNER/NAME]"
                ))
            }
        }
    }
    let base = root.join("docs/roadmap");
    let items_dir = base.join("items");
    let issues_path = base.join("issues.json");
    let (items, issues) = roadmap::load_checked(&items_dir, &issues_path)?;
    let issues = issues.unwrap_or_default();
    let gh = GhCli { repo: repo.clone() };
    let remote = gather(&gh, &issues)?;
    let p = plan(&items, &remote, &issues, &repo);
    print_plan(&p, &repo, do_apply);
    if !p.errors.is_empty() {
        return Err(format!(
            "roadmap sync: {} item(s) cannot be planned; nothing was written",
            p.errors.len()
        ));
    }
    if !do_apply {
        return Ok(());
    }
    let numbers = apply(&gh, &items, &remote, &p, &repo)?;
    std::fs::write(&issues_path, issues_json(&numbers))
        .map_err(|e| format!("write {}: {e}", issues_path.display()))?;
    println!(
        "roadmap sync: wrote {} ({} issues)",
        issues_path.display(),
        numbers.len()
    );
    roadmap::render_to(&items_dir, &issues_path, &base.join("README.md"), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const R: &str = "R";

    fn item(id: &str, milestone: &str, status: &str, extra: &str, deps: &str) -> Item {
        let mut text = format!(
            "---\nid: {id}\ntitle: Do {id}\nmilestone: {milestone}\narea: mechanism\n\
             status: {status}\nopen_to: anyone\nneeds_gpu: false\ndepends_on: [{deps}]\n{extra}---\n"
        );
        for name in [
            "Why it matters",
            "What is missing",
            "Done when",
            "How to start",
            "Rules that apply",
        ] {
            text.push_str(&format!("\n## {name}\n\nText for {name}.\n"));
        }
        text.push_str("\n## Sources\n\n- [`a`](../../start/a.md) and [b](#local)\n");
        roadmap::parse_item(&format!("{id}.md"), &text).unwrap()
    }

    fn milestones() -> BTreeMap<u64, String> {
        (0..8)
            .map(|k| (FIRST_MILESTONE_NUMBER + k, format!("M{} title", k + 1)))
            .collect()
    }

    fn remote_with(issues: Vec<GhIssue>) -> Remote {
        let mut labels: Vec<String> = ["roadmap:item", "area:mechanism", "help wanted"]
            .map(String::from)
            .to_vec();
        labels.push("status:held".into());
        Remote {
            labels,
            milestones: milestones(),
            issues,
            milestone_bodies: MILESTONE_ISSUES
                .iter()
                .map(|n| (*n, format!("Milestone {n}")))
                .collect(),
        }
    }

    /// The issue GitHub would hold after creating `c` as number `n`.
    fn issue_from(c: &Create, n: u64) -> GhIssue {
        GhIssue {
            number: n,
            title: c.title.clone(),
            body: c.body.clone(),
            labels: c.labels.clone(),
            milestone: c.milestone.clone(),
            state: if c.close { "CLOSED" } else { "OPEN" }.into(),
            assignees: c.assignee.iter().cloned().collect(),
        }
    }

    #[test]
    fn roadmap_sync_rewrites_links() {
        assert_eq!(
            rewrite_links("[a](../../start/a.md)", R).unwrap(),
            "[a](https://github.com/R/blob/main/docs/start/a.md)"
        );
        assert_eq!(
            rewrite_links("see [y](../../../crates/x/src/y.rs#L3).", R).unwrap(),
            "see [y](https://github.com/R/blob/main/crates/x/src/y.rs#L3)."
        );
        assert_eq!(
            rewrite_links("[s](sibling.md)", R).unwrap(),
            "[s](https://github.com/R/blob/main/docs/roadmap/items/sibling.md)"
        );
        for same in [
            "[w](https://example.org/a.md)",
            "[w](http://example.org/../b)",
            "[h](#anchor)",
            "no links (here)",
        ] {
            assert_eq!(rewrite_links(same, R).unwrap(), same);
        }
    }

    #[test]
    fn roadmap_sync_body_has_sections_footer_marker_and_numbered_deps() {
        let it = item("b", "M1", "ready", "", "a, c");
        let numbers = Issues::from([("a".to_string(), 41u64)]);
        let body = issue_body(&it, &numbers, R).unwrap();
        assert!(
            body.starts_with("M1 · ready · open to anyone · needs GPU no\n"),
            "{body}"
        );
        assert!(body.contains("\nDepends on: #41, `c`\n"), "{body}");
        for s in roadmap::SECTIONS {
            assert!(body.contains(&format!("## {s}\n")), "{s} missing: {body}");
        }
        assert!(body.contains("(https://github.com/R/blob/main/docs/start/a.md)"));
        assert!(body.contains("[b](#local)"));
        assert!(body.contains(
            "Generated from [docs/roadmap/items/b.md](https://github.com/R/blob/main/docs/roadmap/items/b.md) \
             by `cargo xtask roadmap sync`. Edit that file in a PR, not this issue."
        ));
        assert!(body.trim_end().ends_with("<!-- roadmap:id=b -->"));
        let none = issue_body(&item("a", "M1", "ready", "", ""), &numbers, R).unwrap();
        assert!(!none.contains("Depends on"));
    }

    #[test]
    fn roadmap_sync_labels() {
        let has = |it: &Item, l: &str| labels_for(it).contains(l);
        let ready = item("a", "M1", "ready", "", "");
        assert!(
            has(&ready, "help wanted")
                && has(&ready, "roadmap:item")
                && has(&ready, "area:mechanism")
        );
        let held = item("a", "M1", "held", "", "");
        assert!(has(&held, "status:held") && !has(&held, "help wanted"));
        assert!(has(&item("a", "M1", "decision", "", ""), "owner-decision"));
        let lab = item("a", "M1", "ready", "", "");
        let lab = Item {
            open_to: "lab".into(),
            ..lab
        };
        assert!(!has(&lab, "help wanted"));
        assert!(has(
            &item("a", "M1", "ready", "good_first: true\n", ""),
            "good first issue"
        ));
        assert!(managed_label("Help Wanted") && !managed_label("lab:claude"));
    }

    #[test]
    fn roadmap_sync_plans_creates_when_nothing_exists() {
        let items = vec![
            item("a", "M1", "ready", "", ""),
            item("b", "research", "held", "", ""),
        ];
        let mut remote = remote_with(vec![]);
        remote.labels.clear();
        let p = plan(&items, &remote, &Issues::new(), R);
        assert_eq!(p.creates.len(), 2);
        assert_eq!(p.creates[0].milestone.as_deref(), Some("M1 title"));
        assert_eq!(p.creates[1].milestone, None);
        assert!(p.updates.is_empty() && p.closes.is_empty());
        assert_eq!(
            p.labels,
            [
                "area:mechanism",
                "help wanted",
                "roadmap:item",
                "status:held"
            ]
        );
        assert_eq!(p.checklists.len(), 8, "every milestone body lacks markers");
    }

    #[test]
    fn roadmap_sync_plans_nothing_for_identical_issues_and_detects_changes() {
        let items = vec![
            item("a", "M1", "ready", "", ""),
            item("b", "M1", "done", "", "a"),
        ];
        let first = plan(&items, &remote_with(vec![]), &Issues::new(), R);
        // GitHub after creating a as #10 and b as #11, with b's body refreshed.
        let numbers = Issues::from([("a".to_string(), 10u64), ("b".to_string(), 11u64)]);
        let mut issues: Vec<GhIssue> = first
            .creates
            .iter()
            .zip([10, 11])
            .map(|(c, n)| issue_from(c, n))
            .collect();
        issues[1].body = issue_body(&items[1], &numbers, R).unwrap();
        let mut remote = remote_with(issues);
        for (n, body) in checklist_edits(&items, &numbers, &remote) {
            remote.milestone_bodies.insert(n, body);
        }
        // Found by marker alone (no issues.json), and found through issues.json.
        assert!(plan(&items, &remote, &Issues::new(), R).is_empty());
        assert!(plan(&items, &remote, &numbers, R).is_empty());

        let mut changed = remote.clone();
        changed.issues[0].title = "Old title".into();
        changed.issues[0].labels.push("lab:claude".into());
        let p = plan(&items, &changed, &numbers, R);
        assert_eq!(p.updates.len(), 1, "{p:?}");
        assert_eq!(p.updates[0].title.as_deref(), Some("Do a"));
        assert!(
            p.updates[0].remove_labels.is_empty(),
            "a human label is kept"
        );

        let mut reopened = remote.clone();
        reopened.issues[1].state = "OPEN".into();
        let p = plan(&items, &reopened, &numbers, R);
        assert_eq!(p.closes, vec![("b".to_string(), 11)]);
        assert!(p.updates.is_empty());

        let mut closed = remote.clone();
        closed.issues[0].state = "CLOSED".into();
        let p = plan(&items, &closed, &numbers, R);
        assert!(p.is_empty() && p.warnings.iter().any(|w| w.contains("not reopening")));

        let mut stale = remote;
        stale.issues[0].labels.push("status:held".into());
        let p = plan(&items, &stale, &numbers, R);
        assert_eq!(p.updates[0].remove_labels, ["status:held"]);
    }

    #[test]
    fn roadmap_sync_splices_checklist() {
        let body = format!("Intro\n\n{CHILDREN_START}\nold\n{CHILDREN_END}\n\nTail **kept**\n");
        assert_eq!(
            splice_children(&body, "- [ ] #1 x (ready)").unwrap(),
            format!(
                "Intro\n\n{CHILDREN_START}\n- [ ] #1 x (ready)\n{CHILDREN_END}\n\nTail **kept**\n"
            )
        );
        assert_eq!(
            splice_children("Intro", "L").unwrap(),
            format!("Intro\n\n## Roadmap items\n\n{CHILDREN_START}\nL\n{CHILDREN_END}\n")
        );
        let items = vec![
            item("b", "M2", "done", "", ""),
            item("a", "M2", "ready", "", ""),
        ];
        let numbers = Issues::from([("b".to_string(), 5u64)]);
        assert_eq!(
            children_lines(&items, "M2", &numbers),
            "- [ ] `a` Do a (ready)\n- [x] #5 Do b (done)"
        );
        let mut remote = remote_with(vec![]);
        let current =
            splice_children("Milestone 2030", &children_lines(&items, "M2", &numbers)).unwrap();
        remote.milestone_bodies.insert(2030, current);
        let edits = checklist_edits(&items, &numbers, &remote);
        assert!(
            edits.iter().all(|(n, _)| *n != 2030),
            "unchanged body plans no edit"
        );
        assert_eq!(edits.len(), 7);
    }

    #[test]
    fn roadmap_sync_malformed_markers_are_not_spliced() {
        let end_first = format!("A\n{CHILDREN_END}\nold\n{CHILDREN_START}\nB\n");
        let two_starts = format!("{CHILDREN_START}\na\n{CHILDREN_START}\nb\n{CHILDREN_END}\n");
        let two_ends = format!("{CHILDREN_START}\na\n{CHILDREN_END}\n{CHILDREN_END}\n");
        let lone_start = format!("{CHILDREN_START}\na\n");
        for body in [&end_first, &two_starts, &two_ends, &lone_start] {
            assert!(splice_children(body, "L").is_err(), "{body}");
        }
        assert!(splice_children(&end_first, "L")
            .unwrap_err()
            .contains("before the start"));
        // A malformed milestone body plans no edit and warns; the others still plan.
        let items = vec![item("a", "M2", "ready", "", "")];
        let mut remote = remote_with(vec![]);
        remote.milestone_bodies.insert(2030, end_first);
        let p = plan(&items, &remote, &Issues::new(), R);
        assert!(p.checklists.iter().all(|(n, _)| *n != 2030), "{p:?}");
        assert_eq!(p.checklists.len(), 7);
        assert!(
            p.warnings.iter().any(|w| w
                .starts_with("milestone #2030: roadmap children markers are malformed (")
                && w.ends_with("; fix them by hand")),
            "{:?}",
            p.warnings
        );
    }

    #[test]
    fn roadmap_sync_crlf_body_keeps_crlf_outside_markers() {
        let body =
            format!("Intro\r\n\r\n{CHILDREN_START}\r\nold\r\n{CHILDREN_END}\r\n\r\nTail\r\n");
        let got = splice_children(&body, "- [ ] #1 x\n- [ ] #2 y").unwrap();
        assert_eq!(
            got,
            format!(
                "Intro\r\n\r\n{CHILDREN_START}\r\n- [ ] #1 x\r\n- [ ] #2 y\r\n{CHILDREN_END}\r\n\r\nTail\r\n"
            )
        );
        // An unchanged list plans no edit even though the body is CRLF.
        let items = vec![item("a", "M2", "ready", "", "")];
        let mut remote = remote_with(vec![]);
        let lines = children_lines(&items, "M2", &Issues::new());
        remote
            .milestone_bodies
            .insert(2030, splice_children(&body, &lines).unwrap());
        assert!(checklist_edits(&items, &Issues::new(), &remote)
            .iter()
            .all(|(n, _)| *n != 2030));
    }

    #[test]
    fn roadmap_sync_warns_on_orphan_and_duplicate_markers() {
        let items = vec![item("a", "M1", "ready", "", "")];
        let first = plan(&items, &remote_with(vec![]), &Issues::new(), R);
        let issue = |n: u64, id: &str| GhIssue {
            body: format!("x\n\n{}\n", marker(id)),
            ..issue_from(&first.creates[0], n)
        };
        let closed = GhIssue {
            state: "CLOSED".into(),
            ..issue(11, "retired")
        };
        let remote = remote_with(vec![issue(7, "a"), issue(5, "a"), issue(9, "gone"), closed]);
        let p = plan(&items, &remote, &Issues::new(), R);
        assert!(
            !p.warnings.iter().any(|w| w.contains("#11")),
            "a closed orphan needs no warning"
        );
        assert_eq!(p.numbers["a"], 5, "the lowest number stays the choice");
        assert!(p
            .warnings
            .contains(&"roadmap id `a` is on #5 and #7; close the duplicate by hand".to_string()));
        assert!(p.warnings.contains(
            &"#9 carries roadmap id `gone` but docs/roadmap/items/gone.md does not exist; close it by hand or restore the file"
                .to_string()
        ));
        assert!(p.closes.is_empty(), "an orphan is never closed");
    }

    #[test]
    fn roadmap_sync_link_above_root_is_a_plan_error() {
        assert!(rewrite_links("[x](../../../../x.md)", R)
            .unwrap_err()
            .contains("../../../../x.md"));
        assert!(absolute_link("/../x", R).is_err());
        let mut it = item("a", "M1", "ready", "", "");
        it.body.push_str("\n[bad](../../../../../etc/x)\n");
        let e = issue_body(&it, &Issues::new(), R).unwrap_err();
        assert!(
            e.starts_with("a: ") && e.contains("../../../../../etc/x"),
            "{e}"
        );
        let p = plan(&[it], &remote_with(vec![]), &Issues::new(), R);
        assert_eq!(p.errors.len(), 1);
        assert!(p.creates.is_empty());
        let gh = FakeGh {
            labels: RefCell::new(vec![]),
            issues: RefCell::new(vec![]),
        };
        assert!(apply(&gh, &[], &remote_with(vec![]), &p, R).is_err());
        assert!(gh.issues.borrow().is_empty() && gh.labels.borrow().is_empty());
    }

    #[test]
    fn roadmap_sync_unviewable_issues_json_number_warns_instead_of_aborting() {
        let milestone_issue = |n: u64| GhIssue {
            number: n,
            title: String::new(),
            body: String::new(),
            labels: vec![],
            milestone: None,
            state: "OPEN".into(),
            assignees: vec![],
        };
        let gh = FakeGh {
            labels: RefCell::new(vec![]),
            issues: RefCell::new(
                MILESTONE_ISSUES
                    .iter()
                    .map(|n| milestone_issue(*n))
                    .collect(),
            ),
        };
        let known = Issues::from([("a".to_string(), 999u64)]);
        let remote = gather(&gh, &known).unwrap();
        assert!(remote.issues.iter().all(|i| i.number != 999));
        let items = vec![item("a", "M1", "ready", "", "")];
        let p = plan(&items, &remote, &known, R);
        assert!(
            p.warnings
                .iter()
                .any(|w| w.contains("#999") && w.contains("not found on GitHub")),
            "{:?}",
            p.warnings
        );
        assert!(p.creates.is_empty());
    }

    #[test]
    fn roadmap_sync_issues_json_is_sorted_and_pretty() {
        let n = Issues::from([("b".to_string(), 2u64), ("a".to_string(), 1u64)]);
        assert_eq!(issues_json(&n), "{\n  \"a\": 1,\n  \"b\": 2\n}\n");
    }

    /// GitHub in memory: every write changes the state the next read sees.
    struct FakeGh {
        labels: RefCell<Vec<String>>,
        issues: RefCell<Vec<GhIssue>>,
    }

    impl Gh for FakeGh {
        fn list_labels(&self) -> Result<Vec<String>, String> {
            Ok(self.labels.borrow().clone())
        }
        fn list_milestones(&self) -> Result<BTreeMap<u64, String>, String> {
            Ok(milestones())
        }
        fn list_issues(&self) -> Result<Vec<GhIssue>, String> {
            Ok(self
                .issues
                .borrow()
                .iter()
                .filter(|i| i.labels.iter().any(|l| l == ITEM_LABEL))
                .cloned()
                .collect())
        }
        fn view_issue(&self, number: u64) -> Result<GhIssue, String> {
            self.issues
                .borrow()
                .iter()
                .find(|i| i.number == number)
                .cloned()
                .ok_or(format!("no issue #{number}"))
        }
        fn create_label(&self, name: &str, _: &str, _: &str) -> Result<(), String> {
            self.labels.borrow_mut().push(name.into());
            Ok(())
        }
        fn create_issue(&self, c: &Create) -> Result<u64, String> {
            let n = 100 + self.issues.borrow().len() as u64;
            let mut issue = issue_from(c, n);
            issue.state = "OPEN".into();
            // GitHub drops the trailing newline.
            issue.body = issue.body.trim_end().into();
            self.issues.borrow_mut().push(issue);
            Ok(n)
        }
        fn edit_issue(&self, u: &Update) -> Result<(), String> {
            let mut issues = self.issues.borrow_mut();
            let i = issues.iter_mut().find(|i| i.number == u.number).unwrap();
            if let Some(t) = &u.title {
                i.title = t.clone();
            }
            if let Some(b) = &u.body {
                i.body = b.trim_end().into();
            }
            i.labels.retain(|l| !u.remove_labels.contains(l));
            i.labels.extend(u.add_labels.iter().cloned());
            if let Some(m) = &u.milestone {
                i.milestone = m.clone();
            }
            i.assignees.extend(u.add_assignee.iter().cloned());
            Ok(())
        }
        fn close_issue(&self, number: u64) -> Result<(), String> {
            let mut issues = self.issues.borrow_mut();
            issues
                .iter_mut()
                .find(|i| i.number == number)
                .unwrap()
                .state = "CLOSED".into();
            Ok(())
        }
    }

    #[test]
    fn roadmap_sync_apply_then_second_plan_is_empty() {
        let milestone_issue = |n: u64| GhIssue {
            number: n,
            title: format!("milestone {n}"),
            body: format!("Milestone {n} text"),
            labels: vec!["milestone".into()],
            milestone: None,
            state: "OPEN".into(),
            assignees: vec!["someone".into()],
        };
        let gh = FakeGh {
            labels: RefCell::new(vec!["Help Wanted".into()]),
            issues: RefCell::new(
                MILESTONE_ISSUES
                    .iter()
                    .map(|n| milestone_issue(*n))
                    .collect(),
            ),
        };
        let items = vec![
            item("a", "M1", "ready", "assignee: alice\n", ""),
            item("b", "M1", "held", "", "a"),
            item("c", "infra", "done", "", ""),
        ];
        let remote = gather(&gh, &Issues::new()).unwrap();
        let p = plan(&items, &remote, &Issues::new(), R);
        assert_eq!(
            (p.creates.len(), p.close_count(), p.checklists.len()),
            (3, 1, 8)
        );
        assert!(
            !p.labels.iter().any(|l| l == "help wanted"),
            "case-insensitive label match"
        );
        let numbers = apply(&gh, &items, &remote, &p, R).unwrap();
        assert_eq!(numbers.len(), 3);

        let b = gh.view_issue(numbers["b"]).unwrap();
        assert!(
            b.body.contains(&format!("Depends on: #{}", numbers["a"])),
            "{}",
            b.body
        );
        assert_eq!(gh.view_issue(numbers["c"]).unwrap().state, "CLOSED");
        assert_eq!(gh.view_issue(numbers["a"]).unwrap().assignees, ["alice"]);
        let m1 = gh.view_issue(2029).unwrap().body;
        assert!(
            m1.starts_with("Milestone 2029 text\n\n## Roadmap items\n"),
            "{m1}"
        );
        assert!(
            m1.contains(&format!("- [ ] #{} Do a (ready)", numbers["a"])),
            "{m1}"
        );

        // Second run, with and without issues.json, plans nothing.
        for known in [numbers.clone(), Issues::new()] {
            let remote = gather(&gh, &known).unwrap();
            let again = plan(&items, &remote, &known, R);
            assert!(again.is_empty(), "{again:?}");
        }
        // A person assigns someone else: sync never removes them.
        gh.issues
            .borrow_mut()
            .iter_mut()
            .find(|i| i.number == numbers["a"])
            .unwrap()
            .assignees
            .push("bob".into());
        let remote = gather(&gh, &numbers).unwrap();
        assert!(plan(&items, &remote, &numbers, R).is_empty());
    }
}
