//! Generated archive catalog for the docs reorganization (`cargo xtask docs-catalog`).
//!
//! Reads `archive/catalog.jsonl` (one JSON record per research record) and
//! renders, deterministically:
//!
//! * `archive/CATALOG.md`: totals by outcome, a research-line table and a
//!   mechanism table;
//! * `archive/catalog/<line>.md`: one table per research line, sorted by date
//!   (null dates last, then by path).
//!
//! A record links to `path` when that path is tracked now; otherwise, for
//! `docs/<rest>`, to `archive/<rest>` (the equal-depth move rule); otherwise it
//! is marked `(path not found)` and linked to nothing. Without `--check` the
//! files are written; with `--check` they are compared byte for byte.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::docs_links::{normalize, tracked_files, Universe};

const PR_URL: &str = "https://github.com/UOR-Foundation/uor-r4/pull/";
const DEFAULT_INPUT: &str = "archive/catalog.jsonl";

/// One catalog record, reduced to the fields the pages use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub path: String,
    pub title: String,
    pub date: Option<String>,
    pub lab: String,
    pub line: String,
    pub mechanisms: Vec<String>,
    pub outcome: String,
    pub one_line: String,
    pub prs: Vec<u64>,
}

fn str_field(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

/// Parse the JSONL text; blank lines are skipped.
pub fn parse_records(text: &str) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        if raw.trim().is_empty() {
            continue;
        }
        let v: Value =
            serde_json::from_str(raw).map_err(|e| format!("catalog line {}: {e}", i + 1))?;
        let path = str_field(&v, "path");
        if path.is_empty() {
            return Err(format!("catalog line {}: missing path", i + 1));
        }
        out.push(Record {
            path,
            title: str_field(&v, "title"),
            date: match v.get("date") {
                Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
                _ => None,
            },
            lab: str_field(&v, "lab"),
            line: str_field(&v, "line"),
            mechanisms: v
                .get("mechanisms")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|m| m.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            outcome: str_field(&v, "outcome"),
            one_line: str_field(&v, "one_line"),
            prs: v
                .get("prs")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default(),
        });
    }
    Ok(out)
}

/// Resolve a record path to the path its link should point at.
pub fn resolve_target(path: &str, exists: &dyn Fn(&str) -> bool) -> Option<String> {
    let p = path.trim_end_matches('/');
    if exists(p) {
        return Some(p.to_string());
    }
    let rest = p.strip_prefix("docs/")?;
    let moved = format!("archive/{rest}");
    exists(&moved).then_some(moved)
}

/// Relative link from the directory `from_dir` to repo path `target`.
pub fn relative_link(from_dir: &str, target: &str) -> String {
    let from: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<&str> = vec![".."; from.len() - common];
    parts.extend(&to[common..]);
    parts.join("/")
}

/// Percent-encode the characters that would break a markdown link target.
fn encode_target(t: &str) -> String {
    t.replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29")
}

/// Make text safe for one table cell.
pub fn escape_cell(s: &str) -> String {
    s.replace("\r\n", " ")
        .replace(['\n', '\r'], " ")
        .replace('|', "\\|")
}

fn escape_link_text(s: &str) -> String {
    escape_cell(s).replace('[', "\\[").replace(']', "\\]")
}

/// Sort key: dated records first by date, null dates last, then by path.
pub fn sort_records(recs: &mut [&Record]) {
    recs.sort_by(|a, b| {
        (a.date.is_none(), &a.date, &a.path).cmp(&(b.date.is_none(), &b.date, &b.path))
    });
}

fn page_name(line: &str) -> String {
    line.replace('/', "_")
}

fn line_page(line: &str, recs: &[&Record], exists: &dyn Fn(&str) -> bool) -> String {
    let has_prs = recs.iter().any(|r| !r.prs.is_empty());
    let mut s = format!("# {line}\n\n{} records.\n\n", recs.len());
    s.push_str("| Date | Record | Lab | Outcome | Summary |");
    if has_prs {
        s.push_str(" PRs |");
    }
    s.push_str("\n|---|---|---|---|---|");
    if has_prs {
        s.push_str("---|");
    }
    s.push('\n');
    for r in recs {
        let text = escape_link_text(if r.title.is_empty() {
            &r.path
        } else {
            &r.title
        });
        let rec = match resolve_target(&r.path, exists) {
            Some(t) => format!(
                "[{text}]({})",
                encode_target(&relative_link("archive/catalog", &t))
            ),
            None => format!("{text} (path not found)"),
        };
        s.push_str(&format!(
            "| {} | {} | {} | {} | {} |",
            r.date.as_deref().unwrap_or(""),
            rec,
            escape_cell(&r.lab),
            escape_cell(&r.outcome),
            escape_cell(&r.one_line)
        ));
        if has_prs {
            let prs: Vec<String> = r
                .prs
                .iter()
                .map(|n| format!("[#{n}]({PR_URL}{n})"))
                .collect();
            s.push_str(&format!(" {} |", prs.join(", ")));
        }
        s.push('\n');
    }
    s
}

/// Render every output file as `(repo path, contents)`, sorted by path.
pub fn render(recs: &[Record], exists: &dyn Fn(&str) -> bool) -> Vec<(String, String)> {
    let mut by_line: BTreeMap<&str, Vec<&Record>> = BTreeMap::new();
    let mut by_outcome: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_mech: BTreeMap<&str, usize> = BTreeMap::new();
    for r in recs {
        by_line.entry(&r.line).or_default().push(r);
        *by_outcome.entry(&r.outcome).or_default() += 1;
        let uniq: BTreeSet<&str> = r.mechanisms.iter().map(String::as_str).collect();
        for m in uniq {
            *by_mech.entry(m).or_default() += 1;
        }
    }
    let mut files = Vec::new();
    let mut idx = String::from(
        "# Archive catalog\n\n\
         The archive holds dated research records. Records are byte-identical to their originals and are never edited.\n\
         Pages moved from `docs/X` to `archive/X` at equal depth, so a record's path differs only in its first segment.\n\
         Records that are still live stay under `docs/` and are linked from here in place.\n\
         This file and `archive/catalog/` are generated by `cargo run -q -p xtask -- docs-catalog`\n\
         from `archive/catalog.jsonl`; edit the catalog data, not these pages.\n\n",
    );
    idx.push_str(&format!(
        "## Totals\n\n{} records.\n\n| Outcome | Records |\n|---|---|\n",
        recs.len()
    ));
    for (o, n) in &by_outcome {
        idx.push_str(&format!("| {} | {n} |\n", escape_cell(o)));
    }
    idx.push_str("\n## Research lines\n\n| Line | Records | First date | Last date | Positive | Negative | Page |\n|---|---|---|---|---|---|---|\n");
    for (line, rs) in &mut by_line {
        sort_records(rs);
        let dates: Vec<&str> = rs.iter().filter_map(|r| r.date.as_deref()).collect();
        let count = |o: &str| rs.iter().filter(|r| r.outcome == o).count();
        idx.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | [{}](catalog/{}.md) |\n",
            escape_cell(line),
            rs.len(),
            dates.first().copied().unwrap_or(""),
            dates.last().copied().unwrap_or(""),
            count("positive"),
            count("negative"),
            escape_link_text(line),
            encode_target(&page_name(line))
        ));
        files.push((
            format!("archive/catalog/{}.md", page_name(line)),
            line_page(line, rs, exists),
        ));
    }
    idx.push_str("\n## Mechanisms\n\n| Mechanism | Records |\n|---|---|\n");
    for (m, n) in &by_mech {
        idx.push_str(&format!("| {} | {n} |\n", escape_cell(m)));
    }
    files.push(("archive/CATALOG.md".to_string(), idx));
    files.sort();
    files
}

fn git_universe(root: &Path) -> Result<Universe, String> {
    Ok(Universe::new(tracked_files(root)?))
}

/// `docs-catalog [--input FILE] [--check]`.
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut input = DEFAULT_INPUT.to_string();
    let mut check = false;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--input" => input = it.next().ok_or("--input needs a value")?.clone(),
            "--check" => check = true,
            "--write" => {}
            other => return Err(format!("docs-catalog: unknown argument {other}")),
        }
    }
    let input = normalize(&input)?;
    let text =
        std::fs::read_to_string(root.join(&input)).map_err(|e| format!("read {input}: {e}"))?;
    let recs = parse_records(&text)?;
    let uni = git_universe(root)?;
    // Generated outputs are not tracked yet on a first run; the on-disk
    // catalog input itself never needs to resolve, so tracked state suffices.
    let exists = |p: &str| uni.exists(p);
    let files = render(&recs, &exists);
    let missing = recs
        .iter()
        .filter(|r| resolve_target(&r.path, &exists).is_none())
        .count();
    let lines: BTreeSet<&str> = recs.iter().map(|r| r.line.as_str()).collect();
    if check {
        let mut stale = Vec::new();
        for (p, c) in &files {
            match std::fs::read_to_string(root.join(p)) {
                Ok(on_disk) if on_disk == *c => {}
                _ => stale.push(p.clone()),
            }
        }
        let want: BTreeSet<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        if let Ok(rd) = std::fs::read_dir(root.join("archive/catalog")) {
            for e in rd.flatten() {
                let p = format!("archive/catalog/{}", e.file_name().to_string_lossy());
                if !want.contains(p.as_str()) {
                    stale.push(format!("{p} (orphan)"));
                }
            }
        }
        stale.sort();
        if !stale.is_empty() {
            return Err(format!(
                "docs-catalog: {} stale files:\n  {}",
                stale.len(),
                stale.join("\n  ")
            ));
        }
    } else {
        std::fs::create_dir_all(root.join("archive/catalog")).map_err(|e| e.to_string())?;
        for (p, c) in &files {
            std::fs::write(root.join(p), c).map_err(|e| format!("write {p}: {e}"))?;
        }
    }
    println!(
        "catalog: {} records, {} lines, {} paths not found",
        recs.len(),
        lines.len(),
        missing
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(path: &str, date: Option<&str>) -> Record {
        Record {
            path: path.into(),
            title: "T".into(),
            date: date.map(String::from),
            lab: "claude".into(),
            line: "L".into(),
            mechanisms: vec![],
            outcome: "positive".into(),
            one_line: "x".into(),
            prs: vec![],
        }
    }

    #[test]
    fn relative_links_from_catalog_dir() {
        assert_eq!(
            relative_link("archive/catalog", "archive/labs/x/README.md"),
            "../labs/x/README.md"
        );
        assert_eq!(
            relative_link("archive/catalog", "docs/labs/y/README.md"),
            "../../docs/labs/y/README.md"
        );
    }

    #[test]
    fn equal_depth_fallback() {
        let tracked = ["archive/labs/a.md", "docs/live.md"];
        let ex = |p: &str| tracked.contains(&p);
        assert_eq!(
            resolve_target("docs/labs/a.md", &ex).as_deref(),
            Some("archive/labs/a.md")
        );
        assert_eq!(
            resolve_target("docs/live.md", &ex).as_deref(),
            Some("docs/live.md")
        );
        assert_eq!(resolve_target("docs/gone.md", &ex), None);
        assert_eq!(resolve_target("other/gone.md", &ex), None);
    }

    #[test]
    fn cell_escaping() {
        assert_eq!(escape_cell("a|b\nc\r\nd"), "a\\|b c d");
    }

    #[test]
    fn dates_sort_nulls_last_then_path() {
        let a = rec("b.md", None);
        let b = rec("z.md", Some("2026-01-02"));
        let c = rec("a.md", None);
        let d = rec("y.md", Some("2026-01-01"));
        let mut v = vec![&a, &b, &c, &d];
        sort_records(&mut v);
        let paths: Vec<&str> = v.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, ["y.md", "z.md", "a.md", "b.md"]);
    }

    #[test]
    fn line_page_has_pr_column_only_when_needed() {
        let ex = |p: &str| p == "archive/a.md";
        let mut r = rec("docs/a.md", Some("2026-01-01"));
        let without = line_page("L", &[&r], &ex);
        assert!(!without.contains("PRs"));
        assert!(without.contains("](../a.md)"));
        r.prs = vec![12];
        let with = line_page("L", &[&r], &ex);
        assert!(with.contains("| PRs |"));
        assert!(with.contains("[#12](https://github.com/UOR-Foundation/uor-r4/pull/12)"));
    }

    #[test]
    fn missing_path_is_marked_not_linked() {
        let ex = |_: &str| false;
        let page = line_page("L", &[&rec("docs/zz.md", None)], &ex);
        assert!(page.contains("T (path not found)"));
    }
}
