//! `cargo xtask docs-rewrite-refs --map MAP.tsv [--exclude PREFIX ..] [--apply] [--json OUT]`
//!
//! Run on the pre-move tree, before `docs-move --apply`. Scans the files
//! `docs-inventory` scans for by-string doc-path references (same scope, same
//! `scan_refs`) and rewrites each reference whose path the map moves: the old
//! map prefix after any `../` chain becomes the new prefix, keeping the chain,
//! which stays valid because moves keep depth. A reference whose literal does
//! not contain the old prefix verbatim after its chain is reported
//! `UNREWRITTEN` and left alone.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::docs_inventory::{scan_refs, scanned_for_refs, CodeRef, DEFAULT_PREFIXES};
use crate::docs_links::{tracked_files, write_json, PathMap};
use crate::docs_relink::{covered_paths, parse_args, split_lines, under};

pub const SCHEMA: &str = "uor-r4.docs-rewrite-refs/1";

/// Files never rewritten by default.
pub const DEFAULT_EXCLUDES: &[&str] = &[
    "tools/r4-softmax-trainer/",
    "uor_standards/",
    "third_party/",
    "docs/harness/prep/",
    // The cutover tools themselves: their tests use synthetic docs/ paths on purpose.
    "xtask/src/archive_verify.rs",
    "xtask/src/docs_catalog.rs",
    "xtask/src/docs_inventory.rs",
    "xtask/src/docs_links.rs",
    "xtask/src/docs_move.rs",
    "xtask/src/docs_relink.rs",
    "xtask/src/docs_rewrite_refs.rs",
    "xtask/src/docs_stubs.rs",
];

/// Records are evidence and are never edited: dated lab records, pinned prep
/// sheets (the relink defaults) and SHA256SUMS-covered files.
pub fn is_protected(file: &str, covered: &std::collections::BTreeSet<String>) -> bool {
    crate::docs_relink::is_default_excluded(file) || covered.contains(file)
}

/// Whether references in `file` may be rewritten.
pub fn file_in_scope(file: &str, map: &PathMap, excludes: &[String]) -> bool {
    map.map(file) == file
        && !under(file, "archive")
        && !DEFAULT_EXCLUDES.iter().any(|e| under(file, e))
        && !excludes.iter().any(|e| under(file, e))
}

/// `None` when the map does not move `r.path`; otherwise the rewritten
/// literal, or `Err` when the old prefix is not verbatim after the chain.
pub fn replacement(r: &CodeRef, map: &PathMap) -> Option<Result<String, String>> {
    if map.map(&r.path) == r.path {
        return None;
    }
    let (old, new) = map.matched(&r.path)?;
    let lit = r.literal.as_str();
    let chain = lit.len() - lit.trim_start_matches("../").len();
    let s = &lit[chain..];
    if s == old || (s.starts_with(old) && s.as_bytes().get(old.len()) == Some(&b'/')) {
        Some(Ok(format!("{}{new}{}", &lit[..chain], &s[old.len()..])))
    } else {
        Some(Err(format!("old prefix {old} not in literal")))
    }
}

/// Literal tokens on one line exactly as `scan_refs` delimits them.
fn tokens<'a>(line: &'a str, prefixes: &[String]) -> Vec<(usize, &'a str)> {
    let b = line.as_bytes();
    let path_char = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'/' | b'-');
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_') {
            i += 1;
            continue;
        }
        let mut j = i;
        while b[j..].starts_with(b"../") {
            j += 3;
        }
        if !prefixes.iter().any(|p| b[j..].starts_with(p.as_bytes())) {
            i += 1;
            continue;
        }
        let mut end = j;
        while end < b.len() && path_char(b[end]) {
            end += 1;
        }
        out.push((i, line[i..end].trim_end_matches(['.', '/'])));
        i = end;
    }
    out
}

/// One rewritten or unrewritten reference occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefEdit {
    pub line: usize,
    pub old: String,
    pub new: String,
}

/// Result of rewriting one code text.
#[derive(Debug, Default)]
pub struct Rewritten {
    pub text: String,
    pub edits: Vec<RefEdit>,
    /// `(line, literal, reason)` for references left alone.
    pub unrewritten: Vec<(usize, String, String)>,
}

/// Rewrite the moved references of `file`'s `text`.
pub fn rewrite_text(file: &str, text: &str, map: &PathMap, prefixes: &[String]) -> Rewritten {
    // line -> literal -> replacement (Err: unrewritable)
    let mut wanted: BTreeMap<usize, BTreeMap<String, Result<String, String>>> = BTreeMap::new();
    for r in scan_refs(file, text, prefixes) {
        let Some(rep) = replacement(&r, map) else {
            continue;
        };
        let slot = wanted.entry(r.line).or_default();
        match slot.get(&r.literal) {
            None => {
                slot.insert(r.literal.clone(), rep);
            }
            Some(Ok(prev)) if rep.as_ref().is_ok_and(|n| n != prev) => {
                slot.insert(
                    r.literal.clone(),
                    Err("conflicting replacements".to_string()),
                );
            }
            Some(Err(_)) if rep.is_ok() => {
                // Another reading of the same literal (chain-resolved versus
                // chain-stripped) is rewritable; keep the rewritable one.
                slot.insert(r.literal.clone(), rep);
            }
            Some(_) => {}
        }
    }
    let mut out = Rewritten::default();
    if wanted.is_empty() {
        out.text = text.to_string();
        return out;
    }
    let mut buf = String::with_capacity(text.len() + 64);
    for (idx, (content, term)) in split_lines(text).into_iter().enumerate() {
        let line = idx + 1;
        let Some(lits) = wanted.get(&line) else {
            buf.push_str(content);
            buf.push_str(term);
            continue;
        };
        for (lit, rep) in lits {
            if let Err(why) = rep {
                out.unrewritten.push((line, lit.clone(), why.clone()));
            }
        }
        let mut at = 0;
        for (start, lit) in tokens(content, prefixes) {
            if let Some(Ok(new)) = lits.get(lit) {
                buf.push_str(&content[at..start]);
                buf.push_str(new);
                at = start + lit.len();
                out.edits.push(RefEdit {
                    line,
                    old: lit.to_string(),
                    new: new.clone(),
                });
            }
        }
        buf.push_str(&content[at..]);
        buf.push_str(term);
    }
    out.text = buf;
    out
}

/// `cargo xtask docs-rewrite-refs --map MAP.tsv [--exclude PREFIX ..] [--apply] [--json OUT]`
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let a = parse_args(args, "docs-rewrite-refs")?;
    let prefixes: Vec<String> = DEFAULT_PREFIXES.iter().map(|s| (*s).to_string()).collect();
    let tracked = tracked_files(root)?;
    let covered = covered_paths(root, &tracked);
    let mut files = 0usize;
    let mut edits_json = Vec::new();
    let mut unrewritten_json = Vec::new();
    let mut covered_edited = Vec::new();
    for f in tracked
        .iter()
        .filter(|p| scanned_for_refs(p) && file_in_scope(p, &a.map, &a.excludes))
    {
        let Ok(bytes) = std::fs::read(root.join(f)) else {
            continue; // a tracked symlink or a deleted file: nothing to edit
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let r = rewrite_text(f, &text, &a.map, &prefixes);
        for (line, lit, why) in &r.unrewritten {
            println!("UNREWRITTEN {f}:{line}: {lit} ({why})");
            unrewritten_json.push(json!({"file": f, "line": line, "literal": lit, "reason": why}));
        }
        if r.edits.is_empty() {
            continue;
        }
        // Records are evidence: a dated lab record, a pinned prep sheet or a
        // SHA256SUMS-covered file is never edited, even when a literal in it
        // points at a moved path. Report each such literal instead.
        if is_protected(f, &covered) {
            for e in &r.edits {
                let why = "protected record (dated lab record, prep sheet or SHA256SUMS-covered); not edited";
                println!("UNREWRITTEN {f}:{}: {} ({why})", e.line, e.old);
                unrewritten_json
                    .push(json!({"file": f, "line": e.line, "literal": e.old, "reason": why}));
            }
            continue;
        }
        files += 1;
        if covered.contains(f.as_str()) {
            covered_edited.push(f.clone());
        }
        for e in &r.edits {
            println!("{f}:{}: {} -> {}", e.line, e.old, e.new);
            edits_json.push(json!({"file": f, "line": e.line, "old": e.old, "new": e.new}));
        }
        if a.apply {
            std::fs::write(root.join(f), &r.text).map_err(|e| format!("{f}: {e}"))?;
        }
    }
    for c in &covered_edited {
        println!("WARNING sha256sums-covered file edited: {c}");
    }
    let refs = edits_json.len();
    println!(
        "docs-rewrite-refs: files {files}, refs {refs}, unrewritten {} ({})",
        unrewritten_json.len(),
        if a.apply { "applied" } else { "dry run" }
    );
    if let Some(out) = &a.json {
        let v: Value = json!({
            "schema": SCHEMA,
            "applied": a.apply,
            "files": files,
            "refs": refs,
            "edits": edits_json,
            "unrewritten": unrewritten_json,
            "sha256sums_covered_edited": covered_edited,
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

    fn prefixes() -> Vec<String> {
        DEFAULT_PREFIXES.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn include_bytes_chain_is_kept() {
        let m = map("docs/labs/a\tarchive/labs/a\n");
        let src = "static B: &[u8] = include_bytes!(\"../../../docs/labs/a/b.json\");\r\n";
        let r = rewrite_text("crates/x/src/lib.rs", src, &m, &prefixes());
        assert_eq!(
            r.text,
            "static B: &[u8] = include_bytes!(\"../../../archive/labs/a/b.json\");\r\n"
        );
        assert_eq!(r.edits.len(), 1);
        assert!(r.unrewritten.is_empty());
    }

    #[test]
    fn plain_literal_is_rewritten() {
        let m = map("docs/evidence\tarchive/evidence\n");
        let src = "let p = \"docs/evidence/x.json\"; let q = \"docs/evidencex/y\";";
        let r = rewrite_text("scripts/a.sh", src, &m, &prefixes());
        assert_eq!(
            r.text,
            "let p = \"archive/evidence/x.json\"; let q = \"docs/evidencex/y\";"
        );
    }

    #[test]
    fn records_and_covered_files_are_protected() {
        let mut covered = std::collections::BTreeSet::new();
        covered.insert("docs/evidence/x.json".to_string());
        assert!(is_protected(
            "docs/labs/w2-k16-consumer-verifiers-2026-10-19/k16/v.py",
            &covered
        ));
        assert!(is_protected(
            "docs/harness/prep/m1-x-2026-10-13.md",
            &covered
        ));
        assert!(is_protected("docs/evidence/x.json", &covered));
        assert!(!is_protected(
            "crates/uor-r4-discrete/src/completion.rs",
            &covered
        ));
    }

    #[test]
    fn cutover_tool_sources_are_never_rewritten() {
        let map = PathMap::parse_tsv("docs/history\tarchive/history\n").expect("map");
        assert!(!file_in_scope("xtask/src/docs_relink.rs", &map, &[]));
        assert!(!file_in_scope("xtask/src/archive_verify.rs", &map, &[]));
        assert!(file_in_scope("xtask/src/gnaf_scan.rs", &map, &[]));
    }

    #[test]
    fn excluded_and_moved_files_are_out_of_scope() {
        let m = map("docs/labs/a\tarchive/labs/a\n");
        let ex = vec!["crates/frozen".to_string()];
        assert!(file_in_scope("crates/x/src/lib.rs", &m, &ex));
        assert!(!file_in_scope("third_party/y/build.rs", &m, &ex));
        assert!(!file_in_scope("tools/r4-softmax-trainer/run.py", &m, &ex));
        assert!(!file_in_scope("uor_standards/a.json", &m, &ex));
        assert!(!file_in_scope("docs/harness/prep/x.json", &m, &ex));
        assert!(!file_in_scope("crates/frozen/src/lib.rs", &m, &ex));
        assert!(!file_in_scope("docs/labs/a/run.sh", &m, &ex));
        assert!(!file_in_scope("archive/old.rs", &m, &ex));
    }

    #[test]
    fn unmatchable_literal_is_unrewritten() {
        // From docs/labs/, `../archive/x/f.json` resolves to
        // docs/archive/x/f.json, which the map moves; the literal does not
        // carry the old prefix `docs/archive/x` after its chain.
        let m = map("docs/archive/x\tarchive/archive/x\n");
        let src = "cat ../archive/x/f.json\n";
        let r = rewrite_text("docs/labs/z.sh", src, &m, &prefixes());
        assert_eq!(r.text, src);
        assert!(r.edits.is_empty());
        assert_eq!(r.unrewritten.len(), 1);
        assert_eq!(r.unrewritten[0].0, 1);
        assert_eq!(r.unrewritten[0].1, "../archive/x/f.json");
    }

    #[test]
    fn unmoved_reference_is_untouched() {
        let m = map("docs/labs/a\tarchive/labs/a\n");
        let src = "\"docs/labs/ab/c.json\" \"docs/labs/README.md\"\n";
        let r = rewrite_text("crates/x/src/lib.rs", src, &m, &prefixes());
        assert_eq!(r.text, src);
        assert!(r.edits.is_empty() && r.unrewritten.is_empty());
    }
}
