//! `cargo xtask docs-relink --map MAP.tsv [--exclude PREFIX ..] [--apply] [--json OUT]`
//!
//! Run on the pre-move tree, before `docs-move --apply`. For every tracked
//! markdown file that stays live (not moved by the map, not under `archive/`,
//! not covered by a SHA256SUMS manifest, not excluded), every link whose
//! resolved repository path the map moves is rewritten to the moved path:
//! a source-relative `../` form (or `/`-absolute when the original was), or,
//! for a `github.com/UOR-Foundation/uor-r4/(blob|tree|raw)/main/` URL, only the
//! path part. Fragment, query, a trailing `/` and angle brackets are kept, and
//! no other byte changes: line endings and the final newline are preserved.
//!
//! Default excludes: `docs/harness/prep/` (SHA-pinned FIT sheets) and every
//! dated lab record directory `docs/labs/<name ending in a date>/`. Records are
//! never edited; they resolve again when swept to archive at equal depth.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{json, Value};

use crate::docs_inventory::{is_manifest, parse_manifest};
use crate::docs_links::{
    blank_code_spans, extract_links, is_md, resolve, tracked_files, write_json, PathMap, Resolved,
};

pub const SCHEMA: &str = "uor-r4.docs-relink/1";

/// One rewritten (or unmatched) occurrence: `old` target text and its replacement.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edit {
    pub line: usize,
    pub old: String,
    pub new: String,
}

/// Result of relinking one markdown text.
#[derive(Debug, Default)]
pub struct Relinked {
    pub text: String,
    pub edits: Vec<Edit>,
    /// Targets whose occurrences on the line could not be located exactly as
    /// extraction reported them; left untouched.
    pub unmatched: Vec<Edit>,
}

/// Whether `path` is under `prefix` on a segment boundary.
pub fn under(path: &str, prefix: &str) -> bool {
    let p = prefix.trim_end_matches('/');
    !p.is_empty()
        && (path == p || (path.starts_with(p) && path.as_bytes().get(p.len()) == Some(&b'/')))
}

fn is_date_suffix(name: &str) -> bool {
    let b = name.as_bytes();
    let digits = |s: &[u8]| s.iter().all(u8::is_ascii_digit);
    let dashed = b.len() >= 10 && {
        let t = &b[b.len() - 10..];
        digits(&t[..4]) && t[4] == b'-' && digits(&t[5..7]) && t[7] == b'-' && digits(&t[8..])
    };
    let compact = b.len() >= 8 && digits(&b[b.len() - 8..]);
    let start = if dashed {
        b.len() - 10
    } else {
        b.len().saturating_sub(8)
    };
    (dashed || compact) && (start == 0 || matches!(b[start - 1], b'-' | b'_'))
}

/// Default source excludes: FIT sheets and dated lab record directories.
pub fn is_default_excluded(path: &str) -> bool {
    if under(path, "docs/harness/prep") {
        return true;
    }
    let mut seg = path.split('/');
    matches!(
        (seg.next(), seg.next(), seg.next(), seg.next()),
        (Some("docs"), Some("labs"), Some(name), Some(_)) if is_date_suffix(name)
    )
}

/// Whether a tracked markdown file is a relink source.
pub fn is_source(
    path: &str,
    map: &PathMap,
    covered: &BTreeSet<String>,
    excludes: &[String],
) -> bool {
    is_md(path)
        && map.map(path) == path
        && !under(path, "archive")
        && !covered.contains(path)
        && !is_default_excluded(path)
        && !excludes.iter().any(|e| under(path, e))
}

/// Shortest relative path from directory `from_dir` to `to`.
pub fn relative(from_dir: &str, to: &str) -> String {
    let f: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let t: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    let mut parts = vec![".."; f.len() - common];
    parts.extend(&t[common..]);
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

/// Length of the `https://github.com/UOR-Foundation/uor-r4/<kind>/main/` head.
fn github_head(t: &str) -> Option<usize> {
    let lower = t.to_ascii_lowercase();
    let repo = "github.com/uor-foundation/uor-r4/";
    let at = lower.find(repo)? + repo.len();
    ["blob/main/", "tree/main/", "raw/main/"]
        .iter()
        .find(|k| lower[at..].starts_with(**k))
        .map(|k| at + k.len())
}

/// The replacement for `target` in `source`, when the map moves its path.
pub fn new_target(source: &str, target: &str, map: &PathMap) -> Option<String> {
    let Resolved::Repo(p) = resolve(source, target) else {
        return None;
    };
    let np = map.map(&p);
    if np == p {
        return None;
    }
    let (head, rest) = if target.contains("://") {
        let h = github_head(target)?;
        (&target[..h], &target[h..])
    } else {
        ("", target)
    };
    let cut = rest.find(['#', '?']).unwrap_or(rest.len());
    let (path_raw, suffix) = rest.split_at(cut);
    let mut path = if !head.is_empty() {
        np.clone()
    } else if path_raw.starts_with('/') {
        format!("/{np}")
    } else {
        let dir = source.rfind('/').map_or("", |i| &source[..i]);
        relative(dir, &np)
    };
    if path_raw.contains('%') {
        path = path.replace(' ', "%20");
    }
    if path_raw.ends_with('/') && !path.ends_with('/') {
        path.push('/');
    }
    Some(format!("{head}{path}{suffix}"))
}

/// Whether `target` found at `before | target | after` sits in a link
/// destination position the extractor reports.
fn link_position(before: &str, target: &str, after: Option<char>) -> bool {
    if !matches!(after, None | Some(')' | '>' | '"' | '\'' | ' ' | '\t')) {
        return false;
    }
    let lb = before.to_ascii_lowercase();
    if ["href=\"", "href='", "src=\"", "src='"]
        .iter()
        .any(|n| lb.ends_with(n))
    {
        return true;
    }
    let dest = |b: &str| {
        let b = b.trim_end_matches([' ', '\t']);
        b.ends_with("](") || b.ends_with("]:")
    };
    match before.strip_suffix('<') {
        Some(b) => dest(b) || target.to_ascii_lowercase().starts_with("http"),
        None => dest(before),
    }
}

/// Byte offsets of `target` occurrences in link positions of `blanked`.
fn positions(blanked: &str, target: &str) -> Vec<usize> {
    let step = target.chars().next().map_or(1, char::len_utf8);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(p) = blanked[from..].find(target) {
        let a = from + p;
        let e = a + target.len();
        if link_position(&blanked[..a], target, blanked[e..].chars().next()) {
            out.push(a);
            from = e;
        } else {
            from = a + step;
        }
    }
    out
}

/// Split a text into `(content, terminator)` lines, numbered like `str::lines`.
pub fn split_lines(text: &str) -> Vec<(&str, &str)> {
    text.split_inclusive('\n')
        .map(|piece| {
            let body = piece.strip_suffix('\n').unwrap_or(piece);
            let body = if piece.ends_with('\n') {
                body.strip_suffix('\r').unwrap_or(body)
            } else {
                body
            };
            (body, &piece[body.len()..])
        })
        .collect()
}

/// Rewrite every moved link target in one markdown text read from `source`.
pub fn relink_text(source: &str, text: &str, map: &PathMap) -> Relinked {
    // line -> target -> (replacement, occurrences reported)
    let mut wanted: BTreeMap<usize, BTreeMap<String, (String, usize)>> = BTreeMap::new();
    for link in extract_links(text) {
        if let Some(new) = new_target(source, &link.target, map) {
            wanted
                .entry(link.line)
                .or_default()
                .entry(link.target)
                .or_insert((new, 0))
                .1 += 1;
        }
    }
    let mut out = Relinked::default();
    if wanted.is_empty() {
        out.text = text.to_string();
        return out;
    }
    let mut buf = String::with_capacity(text.len() + 64);
    for (idx, (content, term)) in split_lines(text).into_iter().enumerate() {
        let line = idx + 1;
        let Some(targets) = wanted.get(&line) else {
            buf.push_str(content);
            buf.push_str(term);
            continue;
        };
        let blanked = blank_code_spans(content);
        let mut spans: Vec<(usize, usize, &str)> = Vec::new();
        for (target, (new, count)) in targets {
            let found = positions(&blanked, target);
            let edit = Edit {
                line,
                old: target.clone(),
                new: new.clone(),
            };
            if found.len() == *count {
                spans.extend(found.iter().map(|a| (*a, a + target.len(), new.as_str())));
                out.edits.extend(std::iter::repeat(edit).take(*count));
            } else {
                out.unmatched.push(edit);
            }
        }
        spans.sort();
        let mut at = 0;
        for (a, e, new) in spans {
            if a < at {
                continue; // overlapping candidates cannot both be destinations
            }
            buf.push_str(&content[at..a]);
            buf.push_str(new);
            at = e;
        }
        buf.push_str(&content[at..]);
        buf.push_str(term);
    }
    out.text = buf;
    out
}

/// Parsed common arguments of the cutover tools.
pub struct Args {
    pub map: PathMap,
    pub excludes: Vec<String>,
    pub apply: bool,
    pub json: Option<String>,
}

pub fn parse_args(args: &[String], task: &str) -> Result<Args, String> {
    let mut map_path = None;
    let mut excludes = Vec::new();
    let mut apply = false;
    let mut json = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--apply" => apply = true,
            "--map" | "--exclude" | "--json" => {
                let v = it
                    .next()
                    .ok_or_else(|| format!("{task}: {a} needs a value"))?;
                match a.as_str() {
                    "--map" => map_path = Some(v.clone()),
                    "--exclude" => excludes.push(v.trim_end_matches('/').to_string()),
                    _ => json = Some(v.clone()),
                }
            }
            other => return Err(format!("{task}: unknown argument {other}")),
        }
    }
    let map_path = map_path.ok_or_else(|| format!("{task}: --map MAP.tsv is required"))?;
    let text = std::fs::read_to_string(&map_path).map_err(|e| format!("{map_path}: {e}"))?;
    Ok(Args {
        map: PathMap::parse_tsv(&text)?,
        excludes,
        apply,
        json,
    })
}

/// Every path covered by a tracked SHA256SUMS manifest.
pub fn covered_paths(root: &Path, tracked: &[String]) -> BTreeSet<String> {
    let mut covered = BTreeSet::new();
    for m in tracked.iter().filter(|p| is_manifest(p)) {
        let Ok(bytes) = std::fs::read(root.join(m)) else {
            continue;
        };
        if let Ok(text) = String::from_utf8(bytes) {
            let dir = m.rfind('/').map_or("", |i| &m[..i]);
            covered.extend(parse_manifest(dir, &text));
        }
    }
    covered
}

/// `cargo xtask docs-relink --map MAP.tsv [--exclude PREFIX ..] [--apply] [--json OUT]`
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let a = parse_args(args, "docs-relink")?;
    let tracked = tracked_files(root)?;
    let covered = covered_paths(root, &tracked);
    let mut files = 0usize;
    let mut edits_json = Vec::new();
    let mut unmatched_json = Vec::new();
    let mut skipped = Vec::new();
    for src in tracked
        .iter()
        .filter(|p| is_source(p, &a.map, &covered, &a.excludes))
    {
        let bytes = std::fs::read(root.join(src)).map_err(|e| format!("{src}: {e}"))?;
        let Ok(text) = String::from_utf8(bytes) else {
            skipped.push(src.clone());
            continue;
        };
        let r = relink_text(src, &text, &a.map);
        for e in &r.unmatched {
            println!("UNMATCHED {src}:{}: {} -> {}", e.line, e.old, e.new);
            unmatched_json.push(json!({"file": src, "line": e.line, "old": e.old, "new": e.new}));
        }
        if r.edits.is_empty() {
            continue;
        }
        files += 1;
        for e in &r.edits {
            println!("{src}:{}: {} -> {}", e.line, e.old, e.new);
            edits_json.push(json!({"file": src, "line": e.line, "old": e.old, "new": e.new}));
        }
        if a.apply {
            std::fs::write(root.join(src), &r.text).map_err(|e| format!("{src}: {e}"))?;
        }
    }
    for s in &skipped {
        println!("SKIPPED (not UTF-8) {s}");
    }
    let links = edits_json.len();
    println!(
        "docs-relink: files {files}, links {links}, unmatched {}, skipped {} ({})",
        unmatched_json.len(),
        skipped.len(),
        if a.apply { "applied" } else { "dry run" }
    );
    if let Some(out) = &a.json {
        let v: Value = json!({
            "schema": SCHEMA,
            "applied": a.apply,
            "files": files,
            "links": links,
            "edits": edits_json,
            "unmatched": unmatched_json,
            "skipped_non_utf8": skipped,
        });
        write_json(out, &v)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(tsv: &str) -> PathMap {
        PathMap::parse_tsv(tsv).unwrap()
    }

    #[test]
    fn readme_link_into_moved_history() {
        let m = map("docs/history\tarchive/history\n");
        let r = relink_text("README.md", "See [x](docs/history/x.md).\n", &m);
        assert_eq!(r.text, "See [x](archive/history/x.md).\n");
        assert_eq!(r.edits.len(), 1);
    }

    #[test]
    fn nested_source_gets_shortest_relative_form_with_fragment() {
        let m = map("docs/labs/old-2026-10-01\tarchive/labs/old-2026-10-01\n");
        let r = relink_text(
            "docs/mechanisms/a.md",
            "[r](../labs/old-2026-10-01/README.md#sec)\n",
            &m,
        );
        assert_eq!(
            r.text,
            "[r](../../archive/labs/old-2026-10-01/README.md#sec)\n"
        );
    }

    #[test]
    fn blob_main_url_rewrites_path_part_only() {
        let m = map("docs/history\tarchive/history\n");
        let src = "[u](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/history/x.md?plain=1#L3)\n";
        let r = relink_text("docs/mechanisms/a.md", src, &m);
        assert_eq!(
            r.text,
            "[u](https://github.com/UOR-Foundation/uor-r4/blob/main/archive/history/x.md?plain=1#L3)\n"
        );
    }

    #[test]
    fn crlf_and_missing_final_newline_are_preserved() {
        let m = map("docs/history\tarchive/history\n");
        let r = relink_text(
            "README.md",
            "a\r\n[x](docs/history/)\r\nb [y](<docs/history/y.md>)",
            &m,
        );
        assert_eq!(
            r.text,
            "a\r\n[x](archive/history/)\r\nb [y](<archive/history/y.md>)"
        );
        assert_eq!(r.edits.len(), 2);
    }

    #[test]
    fn excluded_and_moved_sources_are_not_sources() {
        let m = map("docs/history\tarchive/history\n");
        let none = BTreeSet::new();
        let ex = vec!["docs/private".to_string()];
        assert!(is_source("docs/mechanisms/a.md", &m, &none, &ex));
        assert!(!is_source("docs/history/a.md", &m, &none, &ex));
        assert!(!is_source("archive/x.md", &m, &none, &ex));
        assert!(!is_source("docs/harness/prep/m1-x.md", &m, &none, &ex));
        assert!(!is_source(
            "docs/labs/foo-2026-10-12/README.md",
            &m,
            &none,
            &ex
        ));
        assert!(!is_source(
            "docs/labs/lab-state-20261009/README.md",
            &m,
            &none,
            &ex
        ));
        assert!(is_source("docs/labs/README.md", &m, &none, &ex));
        assert!(is_source("docs/labs/protocol.md", &m, &none, &ex));
        assert!(!is_source("docs/private/a.md", &m, &none, &ex));
        let covered: BTreeSet<String> = ["docs/sealed.md".to_string()].into();
        assert!(!is_source("docs/sealed.md", &m, &covered, &ex));
    }

    #[test]
    fn two_occurrences_on_one_line_are_both_rewritten() {
        let m = map("docs/history\tarchive/history\n");
        let r = relink_text(
            "README.md",
            "[a](docs/history/x.md) and [b](docs/history/x.md) `[c](docs/history/x.md)`\n",
            &m,
        );
        assert_eq!(
            r.text,
            "[a](archive/history/x.md) and [b](archive/history/x.md) `[c](docs/history/x.md)`\n"
        );
        assert_eq!(r.edits.len(), 2);
        assert!(r.unmatched.is_empty());
    }

    #[test]
    fn unmoved_link_and_code_fence_are_untouched() {
        let m = map("docs/history\tarchive/history\n");
        let src = "[a](docs/historyx/y.md) [b](STATUS.md)\n```\n[c](docs/history/x.md)\n```\n";
        let r = relink_text("README.md", src, &m);
        assert_eq!(r.text, src);
        assert!(r.edits.is_empty());
    }

    #[test]
    fn reference_definitions_and_href_are_rewritten() {
        let m = map("docs/history\tarchive/history\n");
        let src = "[id]: docs/history/x.md \"t\"\n<a href=\"docs/history/\">h</a>\n";
        let r = relink_text("README.md", src, &m);
        assert_eq!(
            r.text,
            "[id]: archive/history/x.md \"t\"\n<a href=\"archive/history/\">h</a>\n"
        );
    }
}
