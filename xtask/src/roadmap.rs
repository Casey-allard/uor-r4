//! `cargo xtask roadmap check | render [--check]`: the roadmap made of one
//! Markdown file per documented gap, `docs/roadmap/items/<id>.md`.
//!
//! Each item starts with a header between two `---` lines (hand-parsed here:
//! `key: value`, with `depends_on: [a, b]` as the only list), followed by six
//! required sections in a fixed order. `check` validates every item, the
//! dependency graph and `docs/roadmap/issues.json` (an optional JSON object of
//! id -> issue number). `render` writes `docs/roadmap/README.md`
//! deterministically from the items; `render --check` compares instead. `sync`
//! turns the items into GitHub issues (see `roadmap_sync.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::Value;

const MILESTONES: &[&str] = &[
    "M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "research", "infra",
];
pub(crate) const AREAS: &[&str] = &[
    "architecture",
    "mechanism",
    "training",
    "serving",
    "evaluation",
    "tooling",
    "docs",
];
pub(crate) const STATUSES: &[&str] = &["ready", "held", "decision", "done"];
const OPEN_TO: &[&str] = &["anyone", "lab"];
pub(crate) const SECTIONS: &[&str] = &[
    "Why it matters",
    "What is missing",
    "Done when",
    "How to start",
    "Rules that apply",
    "Sources",
];
const REQUIRED_KEYS: &[&str] = &[
    "id",
    "title",
    "milestone",
    "area",
    "status",
    "open_to",
    "needs_gpu",
    "depends_on",
];
/// The loop keys (`scope` .. `done_class`) are optional until the PM backfills
/// the ready items; each is validated when present (see `check_loop_keys`).
const OPTIONAL_KEYS: &[&str] = &[
    "assignee",
    "good_first",
    "scope",
    "size",
    "lane",
    "rank",
    "notebook",
    "done_class",
];
const DONE_CLASSES: &[&str] = &["host", "gpu"];
/// A `scope:` glob covers the whole repository, and the loop page refuses it, when every
/// path segment left after leading `./` and `/` is made only of `*` and `?` (`**`, `./*`, `?*`).
fn is_root_wide(glob: &str) -> bool {
    let mut rest = glob;
    while let Some(r) = rest.strip_prefix("./").or_else(|| rest.strip_prefix('/')) {
        rest = r;
    }
    rest.split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .all(|s| s.chars().all(|c| c == '*' || c == '?'))
}
/// Optional body section; when present it must not be empty.
const CANCEL_IF: &str = "Cancel if";
const MAX_TITLE: usize = 80;
const ISSUE_URL: &str = "https://github.com/UOR-Foundation/uor-r4/issues/";

/// `issues.json`: item id -> GitHub issue number.
type Issues = BTreeMap<String, u64>;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub file: String,
    pub id: String,
    pub title: String,
    pub milestone: String,
    pub area: String,
    pub status: String,
    pub open_to: String,
    pub needs_gpu: bool,
    pub depends_on: Vec<String>,
    /// Optional GitHub login from the `assignee` header key.
    pub assignee: Option<String>,
    pub good_first: bool,
    /// Everything after the closing `---`: the six sections, verbatim.
    pub body: String,
}

/// Split `text` into its header fields and the body after the closing `---`.
pub fn parse_header(text: &str) -> Result<(BTreeMap<String, String>, String), String> {
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return Err("the file does not start with a `---` header line".into());
    }
    let mut fields = BTreeMap::new();
    let mut closed = false;
    let mut body = Vec::new();
    for line in lines.by_ref() {
        if line.trim_end() == "---" {
            closed = true;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("header line `{line}` is not `key: value`"))?;
        let key = key.trim().to_string();
        let mut value = value.trim();
        // Inline `# comment`, except in the title, which may legitimately hold `#`.
        if key != "title" {
            if let Some(at) = value.find(" #") {
                value = value[..at].trim_end();
            }
        }
        let value = unquote(value).to_string();
        if fields.insert(key.clone(), value).is_some() {
            return Err(format!("header key `{key}` appears twice"));
        }
    }
    if !closed {
        return Err("the header has no closing `---` line".into());
    }
    body.extend(lines);
    Ok((fields, body.join("\n")))
}

fn unquote(value: &str) -> &str {
    for q in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(q) && value.ends_with(q) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn parse_list(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or_else(|| format!("depends_on `{value}` is not a `[a, b]` list"))?;
    Ok(inner
        .split(',')
        .map(|s| unquote(s.trim()).to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn parse_bool(key: &str, value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("{key} is `{other}`, expected true or false")),
    }
}

/// Body section errors: each required `## ` section present, non-empty, in order.
fn check_sections(body: &str) -> Vec<String> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            found.push((h.trim().to_string(), String::new()));
        } else if let Some(last) = found.last_mut() {
            last.1.push_str(line.trim());
        }
    }
    let mut errors = Vec::new();
    let mut last_pos: Option<usize> = None;
    for name in SECTIONS {
        match found.iter().position(|(h, _)| h == name) {
            None => errors.push(format!("missing section `## {name}`")),
            Some(pos) => {
                if found[pos].1.is_empty() {
                    errors.push(format!("section `## {name}` is empty"));
                }
                if last_pos.is_some_and(|p| pos < p) {
                    errors.push(format!("section `## {name}` is out of order"));
                }
                last_pos = Some(pos);
            }
        }
    }
    if found
        .iter()
        .any(|(h, text)| h == CANCEL_IF && text.is_empty())
    {
        errors.push(format!("section `## {CANCEL_IF}` is empty"));
    }
    errors
}

/// A positive integer written as plain digits, without a leading zero.
fn positive_int(value: &str) -> bool {
    value.bytes().all(|b| b.is_ascii_digit())
        && !value.starts_with('0')
        && value.parse::<u64>().is_ok_and(|n| n > 0)
}

/// `notebook:` form: a `.md` page under `docs/research/notebook/`.
fn notebook_form_ok(path: &str) -> bool {
    path.starts_with("docs/research/notebook/")
        && path.ends_with(".md")
        && !path.split('/').any(|s| s == "..")
}

/// Errors for the optional loop keys that are present in `fields`.
fn check_loop_keys(fields: &BTreeMap<String, String>) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(v) = fields.get("scope") {
        match parse_list(v) {
            Err(_) => errors.push(format!("scope `{v}` is not a `[a, b]` list of globs")),
            Ok(globs) if globs.is_empty() => {
                errors.push("scope is empty; list at least one glob".into())
            }
            Ok(globs) => {
                for glob in &globs {
                    if glob.contains(['[', ']', '{', '}']) {
                        errors.push(format!(
                            "scope `{glob}` uses `[]` or `{{}}`, which scope globs do not support; list the paths separately"
                        ));
                    } else if is_root_wide(glob) {
                        errors.push(format!(
                            "scope `{glob}` is root-wide and refused; name the paths the item changes"
                        ));
                    } else if glob.starts_with('/') || glob.split('/').any(|s| s == "..") {
                        errors.push(format!("scope `{glob}` is not repository-relative"));
                    }
                }
            }
        }
    }
    for key in ["size", "rank"] {
        if let Some(v) = fields.get(key) {
            if !positive_int(v) {
                errors.push(format!("{key} `{v}` is not a positive integer"));
            }
        }
    }
    if let Some(v) = fields.get("lane") {
        if v.is_empty()
            || !v
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            errors.push(format!(
                "lane `{v}` must be lowercase letters, digits and hyphens"
            ));
        }
    }
    if let Some(v) = fields.get("notebook") {
        if !notebook_form_ok(v) {
            errors.push(format!(
                "notebook `{v}` is not a page under docs/research/notebook/"
            ));
        }
    }
    if let Some(v) = fields.get("done_class") {
        if !DONE_CLASSES.contains(&v.as_str()) {
            errors.push(format!(
                "done_class `{v}` is not one of {}",
                DONE_CLASSES.join(", ")
            ));
        }
    }
    errors
}

/// `notebook:` paths of well-formed items that name no file. The repository
/// root is three levels above `docs/roadmap/items`; any other directory is an
/// error when an item names a notebook, since the root cannot be found.
fn missing_notebooks(items_dir: &Path, files: &[(String, String)]) -> Vec<String> {
    let named: Vec<(&String, String)> = files
        .iter()
        .filter_map(|(file, text)| {
            let (fields, _) = parse_header(text).ok()?;
            let nb = fields.get("notebook")?;
            notebook_form_ok(nb).then(|| (file, nb.clone()))
        })
        .collect();
    if named.is_empty() {
        return Vec::new();
    }
    let root = items_dir
        .ends_with("docs/roadmap/items")
        .then(|| items_dir.ancestors().nth(3))
        .flatten();
    let Some(root) = root else {
        return vec![format!(
            "cannot find the repository root from {}; notebook paths were not checked",
            items_dir.display()
        )];
    };
    named
        .into_iter()
        .filter(|(_, nb)| !root.join(nb).is_file())
        .map(|(file, nb)| format!("{file}: notebook `{nb}` does not exist"))
        .collect()
}

/// `^[a-z0-9]+(-[a-z0-9]+)*$`, checked by hand.
fn valid_id(id: &str) -> bool {
    id.split('-').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

/// Parse and check one item file on its own; errors are prefixed with `file`.
pub fn parse_item(file: &str, text: &str) -> Result<Item, Vec<String>> {
    let fail = |m: String| vec![format!("{file}: {m}")];
    let (fields, body) = parse_header(text).map_err(fail)?;
    let mut errors = Vec::new();
    for key in REQUIRED_KEYS {
        if !fields.contains_key(*key) {
            errors.push(format!("missing header key `{key}`"));
        }
    }
    for key in fields.keys() {
        if !REQUIRED_KEYS.contains(&key.as_str()) && !OPTIONAL_KEYS.contains(&key.as_str()) {
            errors.push(format!("unknown header key `{key}`"));
        }
    }
    if !errors.is_empty() {
        return Err(errors.into_iter().map(|e| format!("{file}: {e}")).collect());
    }
    let get = |k: &str| fields[k].clone();
    let id = get("id");
    let title = get("title");
    if !valid_id(&id) {
        errors.push(format!(
            "id `{id}` must be lowercase letters, digits and single hyphens"
        ));
    }
    if title.is_empty() {
        errors.push("title is empty".into());
    }
    if title.chars().count() > MAX_TITLE {
        errors.push(format!(
            "title is {} characters, over the {MAX_TITLE} limit",
            title.chars().count()
        ));
    }
    for (key, allowed) in [
        ("milestone", MILESTONES),
        ("area", AREAS),
        ("status", STATUSES),
        ("open_to", OPEN_TO),
    ] {
        let v = get(key);
        if !allowed.contains(&v.as_str()) {
            errors.push(format!("{key} `{v}` is not one of {}", allowed.join(", ")));
        }
    }
    let needs_gpu = parse_bool("needs_gpu", &get("needs_gpu")).unwrap_or_else(|e| {
        errors.push(e);
        false
    });
    if let Some(v) = fields.get("good_first") {
        if let Err(e) = parse_bool("good_first", v) {
            errors.push(e);
        }
    }
    let depends_on = parse_list(&get("depends_on")).unwrap_or_else(|e| {
        errors.push(e);
        Vec::new()
    });
    let mut listed = BTreeSet::new();
    for dep in &depends_on {
        if !listed.insert(dep.as_str()) {
            errors.push(format!("depends_on lists `{dep}` twice"));
        }
    }
    errors.extend(check_loop_keys(&fields));
    errors.extend(check_sections(&body));
    if !errors.is_empty() {
        return Err(errors.into_iter().map(|e| format!("{file}: {e}")).collect());
    }
    Ok(Item {
        file: file.to_string(),
        id,
        title,
        milestone: get("milestone"),
        area: get("area"),
        status: get("status"),
        open_to: get("open_to"),
        needs_gpu,
        depends_on,
        assignee: fields.get("assignee").filter(|a| !a.is_empty()).cloned(),
        good_first: fields.get("good_first").is_some_and(|v| v == "true"),
        body,
    })
}

/// Every `*.md` file in `dir` as (file name, text), sorted by name.
pub fn load_dir(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = fs::read_to_string(&path).map_err(|e| format!("read {name}: {e}"))?;
        files.push((name, text));
    }
    files.sort();
    Ok(files)
}

/// `issues.json` as id -> issue number, or `None` when the file does not exist.
pub fn read_issues(path: &Path) -> Result<Option<BTreeMap<String, u64>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|e| format!("issues.json is not JSON: {e}"))?;
    let object = value
        .as_object()
        .ok_or("issues.json is not a JSON object of id -> issue number")?;
    let mut map = BTreeMap::new();
    for (id, n) in object {
        let n = n
            .as_u64()
            .ok_or_else(|| format!("issues.json: `{id}` is not an issue number"))?;
        map.insert(id.clone(), n);
    }
    Ok(Some(map))
}

/// Check every item plus the cross-item rules; returns the items sorted by id.
pub fn validate(
    files: &[(String, String)],
    issues: Option<&BTreeMap<String, u64>>,
) -> Result<Vec<Item>, Vec<String>> {
    let mut errors = Vec::new();
    let mut items: Vec<Item> = Vec::new();
    for (file, text) in files {
        match parse_item(file, text) {
            Ok(item) => items.push(item),
            Err(e) => errors.extend(e),
        }
    }
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for item in &items {
        if item.file != format!("{}.md", item.id) {
            errors.push(format!(
                "{}: file name is not `<id>.md` for id `{}`",
                item.file, item.id
            ));
        }
        if let Some(first) = seen.insert(&item.id, &item.file) {
            errors.push(format!(
                "{}: duplicate id `{}` (also in {first})",
                item.file, item.id
            ));
        }
    }
    let ids: BTreeSet<&str> = items.iter().map(|i| i.id.as_str()).collect();
    // Known ids are the file stems, so a dependency on an item that failed to
    // parse is not also reported as unknown.
    let known: BTreeSet<&str> = files
        .iter()
        .map(|(file, _)| file.strip_suffix(".md").unwrap_or(file))
        .collect();
    for item in &items {
        for dep in &item.depends_on {
            if !known.contains(dep.as_str()) {
                errors.push(format!(
                    "{}: depends_on names unknown id `{dep}`",
                    item.file
                ));
            }
        }
    }
    errors.extend(find_cycles(&items));
    if let Some(issues) = issues {
        for id in issues.keys() {
            if !ids.contains(id.as_str()) {
                errors.push(format!("issues.json: id `{id}` matches no item"));
            }
        }
    }
    if errors.is_empty() {
        items.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(items)
    } else {
        Err(errors)
    }
}

/// One message per dependency cycle found by depth-first search (a back edge).
fn find_cycles(items: &[Item]) -> Vec<String> {
    let graph: BTreeMap<&str, Vec<&str>> = items
        .iter()
        .map(|i| {
            (
                i.id.as_str(),
                i.depends_on.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    // 0 = unvisited, 1 = on the current path, 2 = done.
    let mut state: BTreeMap<&str, u8> = BTreeMap::new();
    let mut errors = Vec::new();
    fn visit<'a>(
        node: &'a str,
        graph: &BTreeMap<&'a str, Vec<&'a str>>,
        state: &mut BTreeMap<&'a str, u8>,
        path: &mut Vec<&'a str>,
        errors: &mut Vec<String>,
    ) {
        state.insert(node, 1);
        path.push(node);
        for &next in graph.get(node).map(Vec::as_slice).unwrap_or(&[]) {
            if !graph.contains_key(next) {
                continue;
            }
            match state.get(next).copied().unwrap_or(0) {
                0 => visit(next, graph, state, path, errors),
                1 => {
                    let start = path.iter().position(|&p| p == next).unwrap();
                    let mut cycle: Vec<&str> = path[start..].to_vec();
                    cycle.push(next);
                    errors.push(format!("dependency cycle: {}", cycle.join(" -> ")));
                }
                _ => {}
            }
        }
        path.pop();
        state.insert(node, 2);
    }
    for &node in graph.keys() {
        if state.get(node).copied().unwrap_or(0) == 0 {
            visit(node, &graph, &mut state, &mut Vec::new(), &mut errors);
        }
    }
    errors
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn item_link(item: &Item) -> String {
    format!("[{}](items/{}.md)", cell(&item.title), item.id)
}

fn issue_cell(id: &str, issues: Option<&BTreeMap<String, u64>>) -> String {
    match issues.and_then(|m| m.get(id)) {
        Some(n) => format!("[#{n}]({ISSUE_URL}{n})"),
        None => "not yet synced".to_string(),
    }
}

pub(crate) fn status_rank(status: &str) -> usize {
    STATUSES
        .iter()
        .position(|s| *s == status)
        .unwrap_or(STATUSES.len())
}

fn node(id: &str) -> String {
    format!("n_{}", id.replace('-', "_"))
}

/// The roadmap page; `items` must be validated. Output depends only on inputs.
pub fn render(items: &[Item], issues: Option<&BTreeMap<String, u64>>) -> String {
    let mut sorted: Vec<&Item> = items.iter().collect();
    sorted.sort_by(|a, b| (status_rank(&a.status), &a.id).cmp(&(status_rank(&b.status), &b.id)));
    let mut out = String::new();
    let mut line = |s: &str| {
        out.push_str(s);
        out.push('\n');
    };
    line("# Roadmap");
    line("");
    line(
        "<!-- Generated by `cargo xtask roadmap render` from items/*.md. Do not edit by hand. -->",
    );
    line("");
    line(
        "This page lists every documented gap between the project today and its eight milestones. \
         Each item is tracked as one GitHub issue. \
         The files in `items/` are the source, and this page is generated from them.",
    );
    line("");
    line(
        "Each item's GitHub issue is generated from its file by `cargo xtask roadmap sync`: \
         the title, body, roadmap labels (`roadmap:item`, `area:*`, `help wanted`, \
         `good first issue`, `status:held`, `owner-decision`) and milestone follow the file, \
         so change the file in a PR rather than the issue.",
    );
    line("");
    line("## How this roadmap works");
    line("");
    line("- Each gap has one file in `items/`, and those files are the source of truth.");
    line("- `cargo xtask roadmap sync` creates or updates one GitHub issue per item.");
    line("- Claim an issue with a comment before you start work on it.");
    line("- A pull request that finishes an item says `Closes #N` for its issue.");
    line("- A new gap is a new item file, added in a pull request.");
    line("- The PM runs `sync` and `render` once a week.");
    line(
        "- Outside contributors can take items marked \"anyone\". \
         They may run GPU steps on their own GPU and attach the hardware specs to the PR. \
         When the hardware differs from the RTX 5090 baseline, a maintainer re-runs the step \
         (owner, 20 Oct 2026).",
    );
    line("");
    line("## Counts");
    line("");
    line("| Status | Items |");
    line("|---|---|");
    for status in STATUSES {
        let n = items.iter().filter(|i| i.status == *status).count();
        line(&format!("| {status} | {n} |"));
    }
    line(&format!("| total | {} |", items.len()));
    line("");
    line("## Start here");
    line("");
    let start: Vec<&&Item> = sorted
        .iter()
        .filter(|i| i.status == "ready" && i.open_to == "anyone")
        .collect();
    if start.is_empty() {
        line("No item is both ready and open to anyone yet.");
    } else {
        line("These items are ready and open to anyone.");
        line("");
        line("| Item | Milestone | Issue |");
        line("|---|---|---|");
        for item in start {
            line(&format!(
                "| {} | {} | {} |",
                item_link(item),
                item.milestone,
                issue_cell(&item.id, issues)
            ));
        }
    }
    let by_id: BTreeMap<&str, &Item> = items.iter().map(|i| (i.id.as_str(), i)).collect();
    for (milestone, heading) in [
        ("M1", "M1"),
        ("M2", "M2"),
        ("M3", "M3"),
        ("M4", "M4"),
        ("M5", "M5"),
        ("M6", "M6"),
        ("M7", "M7"),
        ("M8", "M8"),
        ("research", "Research"),
        ("infra", "Infrastructure"),
    ] {
        line("");
        line(&format!("## {heading}"));
        line("");
        let rows: Vec<&&Item> = sorted.iter().filter(|i| i.milestone == milestone).collect();
        if rows.is_empty() {
            line("No items yet.");
            continue;
        }
        line("| Item | Status | Who can take it | GPU | Depends on | Issue |");
        line("|---|---|---|---|---|---|");
        for item in rows {
            let deps = if item.depends_on.is_empty() {
                "none".to_string()
            } else {
                item.depends_on
                    .iter()
                    .map(|d| match by_id.get(d.as_str()) {
                        Some(_) => format!("[{d}](items/{d}.md)"),
                        None => d.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            line(&format!(
                "| {} | {} | {} | {} | {} | {} |",
                item_link(item),
                item.status,
                if item.open_to == "anyone" {
                    "anyone"
                } else {
                    "a lab"
                },
                if item.needs_gpu { "yes" } else { "no" },
                deps,
                issue_cell(&item.id, issues)
            ));
        }
    }
    line("");
    line("## Waiting on the owner");
    line("");
    let decisions: Vec<&&Item> = sorted.iter().filter(|i| i.status == "decision").collect();
    if decisions.is_empty() {
        line("Nothing is waiting on the owner.");
    } else {
        line("These items need an owner decision before work can start.");
        line("");
        for item in decisions {
            line(&format!("- {} ({})", item_link(item), item.milestone));
        }
    }
    line("");
    line("## Dependencies");
    line("");
    let mut edges: BTreeSet<(&str, &str)> = BTreeSet::new();
    for item in items {
        for dep in &item.depends_on {
            edges.insert((dep.as_str(), item.id.as_str()));
        }
    }
    if edges.is_empty() {
        line("No item depends on another yet.");
    } else {
        line("An arrow points from an item to the item that depends on it.");
        line("");
        line("```mermaid");
        line("flowchart LR");
        let nodes: BTreeSet<&str> = edges.iter().flat_map(|(a, b)| [*a, *b]).collect();
        for id in &nodes {
            line(&format!("  {}[\"{id}\"]", node(id)));
        }
        for (from, to) in &edges {
            line(&format!("  {} --> {}", node(from), node(to)));
        }
        line("```");
    }
    out
}

fn counts_line(items: &[Item], key: impl Fn(&Item) -> &str, order: &[&str]) -> String {
    order
        .iter()
        .map(|k| format!("{k} {}", items.iter().filter(|i| key(i) == *k).count()))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn load_checked(
    items_dir: &Path,
    issues_path: &Path,
) -> Result<(Vec<Item>, Option<Issues>), String> {
    let files = load_dir(items_dir)?;
    // An unreadable issues.json joins the item errors instead of hiding them.
    let (issues, issues_error) = match read_issues(issues_path) {
        Ok(issues) => (issues, None),
        Err(e) => (None, Some(e)),
    };
    let items = validate(&files, issues.as_ref());
    let mut late = missing_notebooks(items_dir, &files);
    late.extend(issues_error);
    let items = match items {
        Ok(items) if late.is_empty() => items,
        Ok(_) => return Err(roadmap_failed(&late)),
        Err(mut errors) => {
            errors.extend(late);
            return Err(roadmap_failed(&errors));
        }
    };
    Ok((items, issues))
}

fn roadmap_failed(errors: &[String]) -> String {
    format!(
        "roadmap check failed with {} problem(s):\n  {}",
        errors.len(),
        errors.join("\n  ")
    )
}

/// `render` against explicit paths; `check` compares instead of writing.
pub fn render_to(
    items_dir: &Path,
    issues_path: &Path,
    readme: &Path,
    check: bool,
) -> Result<(), String> {
    let (items, issues) = load_checked(items_dir, issues_path)?;
    let page = render(&items, issues.as_ref());
    if !check {
        fs::write(readme, &page).map_err(|e| format!("write {}: {e}", readme.display()))?;
        println!(
            "roadmap render: wrote {} ({} items)",
            readme.display(),
            items.len()
        );
        return Ok(());
    }
    if !readme.exists() {
        return Err(format!(
            "{} does not exist; run `cargo xtask roadmap render`",
            readme.display()
        ));
    }
    let current =
        fs::read_to_string(readme).map_err(|e| format!("read {}: {e}", readme.display()))?;
    if current == page {
        println!("roadmap render --check: {} is current", readme.display());
        return Ok(());
    }
    let (mut want, mut have) = (page.lines(), current.lines());
    let mut n = 1;
    loop {
        match (want.next(), have.next()) {
            (Some(w), Some(h)) if w == h => n += 1,
            (None, None) => {
                return Err(format!(
                    "{} differs only in line endings or the final newline; run `cargo xtask roadmap render`",
                    readme.display()
                ));
            }
            (w, h) => {
                return Err(format!(
                    "{} is stale; run `cargo xtask roadmap render`. First difference at line {n}:\n  expected: {}\n  found:    {}",
                    readme.display(),
                    w.unwrap_or("<end of file>"),
                    h.unwrap_or("<end of file>")
                ));
            }
        }
    }
}

/// The arguments after `render`: only `--check` is allowed.
fn parse_render_args(args: &[String]) -> Result<bool, String> {
    let mut check = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            other => return Err(format!("roadmap render: unknown argument `{other}`")),
        }
    }
    Ok(check)
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let base = root.join("docs/roadmap");
    let items_dir = base.join("items");
    let issues_path = base.join("issues.json");
    match args.first().map(String::as_str) {
        Some("check") => {
            let (items, _) = load_checked(&items_dir, &issues_path)?;
            println!("roadmap check: {} items OK", items.len());
            println!(
                "  by status: {}",
                counts_line(&items, |i| &i.status, STATUSES)
            );
            println!(
                "  by milestone: {}",
                counts_line(&items, |i| &i.milestone, MILESTONES)
            );
            Ok(())
        }
        Some("render") => {
            let check = parse_render_args(&args[1..])?;
            render_to(&items_dir, &issues_path, &base.join("README.md"), check)
        }
        Some("sync") => crate::roadmap_sync::run(root, &args[1..]),
        _ => Err("usage: cargo xtask roadmap check | render [--check] | sync [--apply] [--repo OWNER/NAME]".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_text(id: &str, deps: &str, sections: &[&str]) -> String {
        let mut s = format!(
            "---\nid: {id}\ntitle: Do {id}\nmilestone: M1            # M1..M8\narea: mechanism\n\
             status: ready\nopen_to: anyone\nneeds_gpu: false\ndepends_on: [{deps}]\n---\n"
        );
        for name in sections {
            s.push_str(&format!("\n## {name}\n\nText for {name}.\n"));
        }
        s
    }

    fn file(id: &str, deps: &str) -> (String, String) {
        (format!("{id}.md"), item_text(id, deps, SECTIONS))
    }

    fn errors_of(files: &[(String, String)]) -> Vec<String> {
        validate(files, None).expect_err("expected check to fail")
    }

    #[test]
    fn roadmap_parses_header() {
        let text = "---\nid: a-b\ntitle: \"Fix #12 now\"\nmilestone: M1  # note\n\
                    depends_on: [x, y]\n---\nbody line\n";
        let (fields, body) = parse_header(text).unwrap();
        assert_eq!(fields["id"], "a-b");
        assert_eq!(fields["title"], "Fix #12 now");
        assert_eq!(fields["milestone"], "M1");
        assert_eq!(parse_list(&fields["depends_on"]).unwrap(), vec!["x", "y"]);
        assert_eq!(body, "body line");
        assert!(parse_header("---\nid: a\n").is_err());
        assert!(parse_header("id: a\n---\n").is_err());

        let item = parse_item("a.md", &item_text("a", "", SECTIONS)).unwrap();
        assert_eq!(item.milestone, "M1");
        assert!(!item.needs_gpu);
        assert!(item.depends_on.is_empty());
    }

    #[test]
    fn roadmap_check_flags_bad_values_and_name() {
        let text = item_text("a", "", SECTIONS)
            .replace("needs_gpu: false", "needs_gpu: maybe")
            .replace("status: ready", "status: doing");
        let errors = errors_of(&[("b.md".into(), text)]);
        assert!(
            errors.iter().any(|e| e.contains("needs_gpu is `maybe`")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("status `doing`")),
            "{errors:?}"
        );
        let errors = errors_of(&[("b.md".into(), item_text("a", "", SECTIONS))]);
        assert!(
            errors.iter().any(|e| e.contains("not `<id>.md`")),
            "{errors:?}"
        );
        let long = item_text("a", "", SECTIONS)
            .replace("title: Do a", &format!("title: {}", "x".repeat(81)));
        let errors = errors_of(&[("a.md".into(), long)]);
        assert!(
            errors.iter().any(|e| e.contains("over the 80 limit")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_check_catches_duplicate_id() {
        let dup = ("copy.md".to_string(), item_text("a", "", SECTIONS));
        let errors = errors_of(&[file("a", ""), dup]);
        assert!(
            errors.iter().any(|e| e.contains("duplicate id `a`")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_check_catches_unknown_dependency() {
        let errors = errors_of(&[file("a", "ghost")]);
        assert!(
            errors.iter().any(|e| e.contains("unknown id `ghost`")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_check_catches_missing_empty_and_out_of_order_sections() {
        let missing = item_text("a", "", &SECTIONS[..5]);
        let errors = errors_of(&[("a.md".into(), missing)]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing section `## Sources`")),
            "{errors:?}"
        );

        let empty = item_text("a", "", SECTIONS).replace("Text for Done when.", "");
        let errors = errors_of(&[("a.md".into(), empty)]);
        assert!(
            errors.iter().any(|e| e.contains("`## Done when` is empty")),
            "{errors:?}"
        );

        let mut swapped = SECTIONS.to_vec();
        swapped.swap(0, 1);
        let errors = errors_of(&[("a.md".into(), item_text("a", "", &swapped))]);
        assert!(
            errors.iter().any(|e| e.contains("out of order")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_check_catches_cycle() {
        let errors = errors_of(&[file("a", "b"), file("b", "c"), file("c", "a")]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("dependency cycle: a -> b -> c -> a")),
            "{errors:?}"
        );
        assert!(validate(&[file("a", "b"), file("b", "")], None).is_ok());
    }

    #[test]
    fn roadmap_check_catches_stray_issue_id() {
        let issues = BTreeMap::from([("nope".to_string(), 5u64)]);
        let errors = validate(&[file("a", "")], Some(&issues)).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("`nope` matches no item")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_render_is_deterministic_and_check_detects_staleness() {
        let dir = std::env::temp_dir().join(format!("xtask-roadmap-render-{}", std::process::id()));
        let items_dir = dir.join("items");
        fs::create_dir_all(&items_dir).unwrap();
        for (name, text) in [
            file("b-two", "a-one"),
            file("a-one", ""),
            file("c-three", "a-one, b-two"),
        ] {
            fs::write(items_dir.join(name), text).unwrap();
        }
        fs::write(items_dir.join("a-one.md.tmp"), "ignored").unwrap();
        let issues = dir.join("issues.json");
        fs::write(&issues, "{\"a-one\": 7}").unwrap();
        let readme = dir.join("README.md");

        render_to(&items_dir, &issues, &readme, false).unwrap();
        let first = fs::read_to_string(&readme).unwrap();
        render_to(&items_dir, &issues, &readme, false).unwrap();
        assert_eq!(first, fs::read_to_string(&readme).unwrap());
        render_to(&items_dir, &issues, &readme, true).unwrap();

        assert!(first.contains(&format!("[#7]({ISSUE_URL}7)")));
        assert!(first.contains("| [Do b-two](items/b-two.md) | M1 | not yet synced |"));
        assert!(first.contains("  n_a_one --> n_c_three"));

        let mut stale = first.replace("| ready | 3 |", "| ready | 2 |");
        fs::write(&readme, &stale).unwrap();
        let err = render_to(&items_dir, &issues, &readme, true).unwrap_err();
        let line = first.lines().position(|l| l == "| ready | 3 |").unwrap() + 1;
        assert!(err.contains(&format!("line {line}")), "{err}");
        stale.clear();
        fs::remove_dir_all(&dir).unwrap();
    }

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-roadmap-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("items")).unwrap();
        dir
    }

    #[test]
    fn roadmap_check_rejects_bad_id_characters() {
        for bad in ["a_b", "A-b", "a--b", "a.b"] {
            let errors = errors_of(&[(format!("{bad}.md"), item_text(bad, "", SECTIONS))]);
            assert!(
                errors
                    .iter()
                    .any(|e| e.contains(&format!("id `{bad}` must be lowercase letters"))),
                "{bad}: {errors:?}"
            );
        }
        assert!(validate(&[file("a-b2", "")], None).is_ok());
    }

    #[test]
    fn roadmap_check_reports_bad_issues_json_with_item_errors() {
        let dir = scratch("issues-and-items");
        let bad = item_text("a", "", SECTIONS).replace("status: ready", "status: doing");
        fs::write(dir.join("items/a.md"), bad).unwrap();
        let issues = dir.join("issues.json");
        fs::write(&issues, "{\"a\": \"seven\"}").unwrap();
        let err = load_checked(&dir.join("items"), &issues).unwrap_err();
        assert!(err.contains("status `doing`"), "{err}");
        assert!(err.contains("`a` is not an issue number"), "{err}");
        assert!(err.contains("2 problem(s)"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn roadmap_check_does_not_call_a_broken_item_unknown() {
        let broken = ("broken.md".to_string(), "not an item".to_string());
        let errors = errors_of(&[file("a", "broken"), broken]);
        assert!(
            errors.iter().any(|e| e.starts_with("broken.md:")),
            "{errors:?}"
        );
        assert!(
            !errors.iter().any(|e| e.contains("unknown id")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_render_check_says_when_readme_is_missing() {
        let dir = scratch("missing-readme");
        let (name, text) = file("a", "");
        fs::write(dir.join("items").join(name), text).unwrap();
        let err = render_to(
            &dir.join("items"),
            &dir.join("issues.json"),
            &dir.join("README.md"),
            true,
        )
        .unwrap_err();
        assert!(
            err.contains("does not exist; run `cargo xtask roadmap render`"),
            "{err}"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn roadmap_render_check_says_when_only_line_endings_differ() {
        let dir = scratch("line-endings");
        let (name, text) = file("a", "");
        fs::write(dir.join("items").join(name), text).unwrap();
        let (issues, readme) = (dir.join("issues.json"), dir.join("README.md"));
        render_to(&dir.join("items"), &issues, &readme, false).unwrap();
        let page = fs::read_to_string(&readme).unwrap();
        for changed in [page.trim_end().to_string(), page.replace('\n', "\r\n")] {
            fs::write(&readme, changed).unwrap();
            let err = render_to(&dir.join("items"), &issues, &readme, true).unwrap_err();
            assert!(
                err.contains("differs only in line endings or the final newline"),
                "{err}"
            );
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn roadmap_render_rejects_unknown_arguments() {
        let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(parse_render_args(&args(&[])), Ok(false));
        assert_eq!(parse_render_args(&args(&["--check"])), Ok(true));
        let err = parse_render_args(&args(&["--chek"])).unwrap_err();
        assert!(err.contains("`--chek`"), "{err}");
        assert!(parse_render_args(&args(&["--check", "extra"])).is_err());
    }

    #[test]
    fn roadmap_check_covers_header_key_rules() {
        let text = item_text("a", "", SECTIONS);
        let errors = errors_of(&[("a.md".into(), text.replace("area: mechanism\n", ""))]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing header key `area`")),
            "{errors:?}"
        );
        let errors = errors_of(&[("a.md".into(), text.replace("---\n", "---\nflavor: x\n"))]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("unknown header key `flavor`")),
            "{errors:?}"
        );
        let errors = errors_of(&[("a.md".into(), text.replace("---\n", "---\nid: a\n"))]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("header key `id` appears twice")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_check_rejects_non_number_issue_value() {
        let dir = scratch("issue-value");
        fs::write(dir.join("issues.json"), "{\"a\": \"seven\"}").unwrap();
        let err = read_issues(&dir.join("issues.json")).unwrap_err();
        assert!(err.contains("`a` is not an issue number"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn roadmap_check_rejects_self_dependency_and_duplicate_dependency() {
        let errors = errors_of(&[file("a", "a")]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("dependency cycle: a -> a")),
            "{errors:?}"
        );
        let errors = errors_of(&[file("a", "b, b"), file("b", "")]);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("a.md: depends_on lists `b` twice")),
            "{errors:?}"
        );
    }

    #[test]
    fn roadmap_crlf_item_parses_like_lf() {
        let lf = item_text("a", "b", SECTIONS);
        let crlf = lf.replace('\n', "\r\n");
        let from_lf = parse_item("a.md", &lf).unwrap();
        let from_crlf = parse_item("a.md", &crlf).unwrap();
        assert_eq!(from_lf, from_crlf);
    }

    #[test]
    fn roadmap_render_escapes_brackets_and_states_issue_ownership() {
        let mut item = parse_item("a.md", &item_text("a", "", SECTIONS)).unwrap();
        item.title = "Fix [x] | y".to_string();
        let page = render(&[item], None);
        assert!(page.contains("[Fix \\[x\\] \\| y](items/a.md)"), "{page}");
        assert!(page.contains("so change the file in a PR rather than the issue."));
        assert!(page.contains("by `cargo xtask roadmap sync`"));
    }

    const LOOP_KEYS: &str = "scope: [xtask/src/**, docs/roadmap/*.md]\nsize: 250\nlane: infra\n\
                             rank: 3\nnotebook: docs/research/notebook/x.md\ndone_class: host\n";

    /// Item `a` with `extra` header lines and, when `cancel` is `Some`, a
    /// `## Cancel if` section holding that text.
    fn loop_item(extra: &str, cancel: Option<&str>) -> String {
        let mut text = item_text("a", "", SECTIONS).replacen("---\n", &format!("---\n{extra}"), 1);
        if let Some(c) = cancel {
            text.push_str(&format!("\n## Cancel if\n\n{c}\n"));
        }
        text
    }

    fn loop_errors(extra: &str) -> Vec<String> {
        errors_of(&[("a.md".into(), loop_item(extra, None))])
    }

    fn assert_error(errors: &[String], want: &str) {
        assert!(
            errors.iter().any(|e| e.contains(want)),
            "{want}: {errors:?}"
        );
    }

    #[test]
    fn roadmap_loop_keys_are_optional_and_accepted_when_well_formed() {
        assert!(validate(&[file("a", "")], None).is_ok());
        let full = loop_item(LOOP_KEYS, Some("The probe finds the keys already checked."));
        assert!(validate(&[("a.md".into(), full)], None).is_ok());
        let quoted = loop_item("scope: [\"xtask/**\"]\ndone_class: gpu\n", None);
        assert!(validate(&[("a.md".into(), quoted)], None).is_ok());
    }

    #[test]
    fn roadmap_loop_scope_refuses_every_root_wide_glob() {
        for glob in [
            "**", "**/*", "/**", "*", "./**", "**/**", "./*", "*/**", "./**/*", "/", ".", "?*",
            "**/?*", "?", "./?*/**",
        ] {
            let errors = loop_errors(&format!("scope: [docs/**, {glob}]\n"));
            assert_error(
                &errors,
                &format!("a.md: scope `{glob}` is root-wide and refused"),
            );
        }
    }

    #[test]
    fn roadmap_loop_scope_refuses_unsupported_glob_syntax() {
        // The list splits on commas, so `{docs,xtask}/**` reports its first piece.
        for (written, reported) in [
            ("[!.]*", "[!.]*"),
            ("{**}", "{**}"),
            ("{docs,xtask}/**", "{docs"),
        ] {
            let errors = loop_errors(&format!("scope: [{written}]\n"));
            assert_error(
                &errors,
                &format!(
                    "scope `{reported}` uses `[]` or `{{}}`, which scope globs do not support; list the paths separately"
                ),
            );
        }
    }

    #[test]
    fn roadmap_loop_scope_accepts_ordinary_globs() {
        for glob in ["docs/**", "*.md", "**/*.rs", "./docs/**", "xtask/src/?.rs"] {
            let text = loop_item(&format!("scope: [{glob}]\n"), None);
            assert!(validate(&[("a.md".into(), text)], None).is_ok(), "{glob}");
        }
    }

    #[test]
    fn roadmap_loop_scope_must_be_a_non_empty_relative_list() {
        assert_error(&loop_errors("scope: []\n"), "scope is empty");
        assert_error(
            &loop_errors("scope: docs/**\n"),
            "scope `docs/**` is not a `[a, b]` list",
        );
        for bad in ["/etc/**", "docs/../x/**", "../x"] {
            assert_error(
                &loop_errors(&format!("scope: [{bad}]\n")),
                &format!("scope `{bad}` is not repository-relative"),
            );
        }
    }

    #[test]
    fn roadmap_loop_size_must_be_a_positive_integer() {
        for bad in ["0", "-3", "+5", "1.5", "many", "", "05", "007"] {
            assert_error(
                &loop_errors(&format!("size: {bad}\n")),
                &format!("size `{bad}` is not a positive integer"),
            );
        }
    }

    #[test]
    fn roadmap_loop_rank_must_be_a_positive_integer() {
        for bad in ["0", "first", "2x", "05", "007"] {
            assert_error(
                &loop_errors(&format!("rank: {bad}\n")),
                &format!("rank `{bad}` is not a positive integer"),
            );
        }
    }

    #[test]
    fn roadmap_loop_lane_must_be_a_slug() {
        for bad in ["", "Infra", "in fra", "in_fra"] {
            assert_error(
                &loop_errors(&format!("lane: {bad}\n")),
                &format!("lane `{bad}` must be lowercase letters, digits and hyphens"),
            );
        }
    }

    #[test]
    fn roadmap_loop_done_class_is_host_or_gpu() {
        assert_error(
            &loop_errors("done_class: cpu\n"),
            "done_class `cpu` is not one of host, gpu",
        );
    }

    #[test]
    fn roadmap_loop_notebook_must_be_a_relative_md_path() {
        for bad in [
            "",
            "/abs/x.md",
            "docs/../x.md",
            "docs/research/notebook/../x.md",
            "docs/research/notebook/x.txt",
            "docs/x.md",
            "docs/nb/x.md",
        ] {
            assert_error(
                &loop_errors(&format!("notebook: {bad}\n")),
                &format!("notebook `{bad}` is not a page under docs/research/notebook/"),
            );
        }
    }

    #[test]
    fn roadmap_loop_notebook_outside_the_notebook_directory_is_refused() {
        assert_error(
            &loop_errors("notebook: docs/research/other/x.md\n"),
            "notebook `docs/research/other/x.md` is not a page under docs/research/notebook/",
        );
    }

    #[test]
    fn roadmap_loop_notebook_needs_a_findable_repository_root() {
        let dir = scratch("nb-root");
        fs::write(
            dir.join("items/a.md"),
            loop_item("notebook: docs/research/notebook/x.md\n", None),
        )
        .unwrap();
        let err = load_checked(&dir.join("items"), &dir.join("issues.json")).unwrap_err();
        assert!(
            err.contains("cannot find the repository root from")
                && err.contains("notebook paths were not checked"),
            "{err}"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn roadmap_loop_notebook_must_exist_under_the_repository_root() {
        let root = std::env::temp_dir().join(format!("xtask-roadmap-nb-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let items = root.join("docs/roadmap/items");
        fs::create_dir_all(&items).unwrap();
        fs::create_dir_all(root.join("docs/research/notebook")).unwrap();
        fs::write(root.join("docs/research/notebook/here.md"), "# notebook\n").unwrap();
        let issues = root.join("docs/roadmap/issues.json");
        fs::write(
            items.join("a.md"),
            loop_item("notebook: docs/research/notebook/here.md\n", None),
        )
        .unwrap();
        assert!(load_checked(&items, &issues).is_ok());
        fs::write(
            items.join("a.md"),
            loop_item("notebook: docs/research/notebook/gone.md\n", None),
        )
        .unwrap();
        let err = load_checked(&items, &issues).unwrap_err();
        assert!(
            err.contains("a.md: notebook `docs/research/notebook/gone.md` does not exist"),
            "{err}"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn roadmap_cancel_if_section_must_not_be_empty() {
        let errors = errors_of(&[("a.md".into(), loop_item("", Some("")))]);
        assert_error(&errors, "a.md: section `## Cancel if` is empty");
    }
}
