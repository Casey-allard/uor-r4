//! Docs inventory for the docs reorganization (`cargo xtask docs-inventory`).
//!
//! About 9,000 files move from `docs/` into `archive/` byte for byte. This
//! task records what is there and what points at it, so the move can be
//! planned and then checked:
//!
//! * `files`: every tracked file under `docs/` with size, SHA-256, extension,
//!   the date embedded in its path, whether a `SHA256SUMS`-style manifest
//!   covers it, and inbound/outbound markdown link counts (links come from
//!   `docs_links`, so the extraction and resolution rules are the same as
//!   `cargo xtask links`);
//! * `code_refs`: every place where code, workflows or JSON read a doc path
//!   by string (a path-like substring starting with a known prefix,
//!   optionally behind a `../` chain);
//! * `summary`: counts per subtree and prefix, and the sorted set of
//!   referenced directories cut to three path components.
//!
//! The universe is the git index (`git ls-files -z`), never the filesystem.
//! Binary and non-UTF-8 files are skipped when scanning for references.
//! Output is sorted and deterministic.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::docs_links::{
    extract_links, is_md, normalize, resolve, tracked_files, write_json, Resolved,
};

pub const SCHEMA: &str = "uor-r4.docs-inventory/1";

/// Path prefixes whose occurrence in code counts as a doc-path reference.
pub const DEFAULT_PREFIXES: &[&str] = &[
    "docs/",
    "archive/",
    "handoff/",
    "uor_standards/",
    "models/",
    "tools/",
    "proofs/",
    "spiralcore-base/",
    "research/",
    "profiles/",
    "ops/",
    "relation-pools/",
    "t2/",
    "prototypes/",
    "assets/",
];

const CODE_EXTS: &[&str] = &[
    "rs", "py", "sh", "yml", "yaml", "toml", "json", "js", "html", "mjs",
];
const DOCS_CODE_EXTS: &[&str] = &["rs", "py", "sh", "toml"];

/// One by-string reference to a doc path found in a code file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CodeRef {
    pub file: String,
    pub line: usize,
    pub literal: String,
    pub path: String,
}

fn ext_of(path: &str) -> &str {
    let base = path.rsplit('/').next().unwrap_or(path);
    match base.rfind('.') {
        Some(i) if i > 0 => &base[i + 1..],
        _ => "",
    }
}

fn dir_of(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..i])
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// First `YYYY-MM-DD` or `YYYYMMDD` (years 2024-2027) in `path`, as `YYYY-MM-DD`.
pub fn date_of(path: &str) -> Option<String> {
    let b = path.as_bytes();
    let digits = |s: &[u8]| -> Option<u32> {
        if s.iter().all(u8::is_ascii_digit) {
            std::str::from_utf8(s).ok()?.parse().ok()
        } else {
            None
        }
    };
    let valid = |y: u32, m: u32, d: u32| {
        (2024..=2027).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)
    };
    for i in 0..b.len() {
        if i > 0 && b[i - 1].is_ascii_digit() {
            continue;
        }
        if i + 10 <= b.len() && b[i + 4] == b'-' && b[i + 7] == b'-' {
            if let (Some(y), Some(m), Some(d)) = (
                digits(&b[i..i + 4]),
                digits(&b[i + 5..i + 7]),
                digits(&b[i + 8..i + 10]),
            ) {
                if valid(y, m, d) && !b.get(i + 10).is_some_and(u8::is_ascii_digit) {
                    return Some(format!("{y:04}-{m:02}-{d:02}"));
                }
            }
        }
        if i + 8 <= b.len() {
            if let (Some(y), Some(m), Some(d)) = (
                digits(&b[i..i + 4]),
                digits(&b[i + 4..i + 6]),
                digits(&b[i + 6..i + 8]),
            ) {
                if valid(y, m, d) && !b.get(i + 8).is_some_and(u8::is_ascii_digit) {
                    return Some(format!("{y:04}-{m:02}-{d:02}"));
                }
            }
        }
    }
    None
}

/// Whether `path` names a checksum manifest.
pub fn is_manifest(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    let lower = base.to_ascii_lowercase();
    base.contains("SHA256SUMS") || (lower.contains("sha256sums") && lower.ends_with(".txt"))
}

/// Paths covered by a manifest located at `manifest_dir`. Bad lines are ignored.
pub fn parse_manifest(manifest_dir: &str, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches(['\r', '\n']);
        let b = line.as_bytes();
        if b.len() < 66
            || !b[..64].iter().all(u8::is_ascii_hexdigit)
            || !matches!(b[64], b' ' | b'\t')
        {
            continue;
        }
        let rest = line[64..].trim_start_matches([' ', '\t']);
        let rest = rest.strip_prefix('*').unwrap_or(rest);
        if rest.is_empty() {
            continue;
        }
        let joined = if manifest_dir.is_empty() {
            rest.to_string()
        } else {
            format!("{manifest_dir}/{rest}")
        };
        if let Ok(p) = normalize(&joined) {
            out.push(p);
        }
    }
    out
}

fn is_path_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'/' | b'-')
}

/// Doc-path references in `text` read from `file` (repo-relative).
pub fn scan_refs(file: &str, text: &str, prefixes: &[String]) -> Vec<CodeRef> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let b = line.as_bytes();
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
            while end < b.len() && is_path_char(b[end]) {
                end += 1;
            }
            let literal = line[i..end].trim_end_matches(['.', '/']);
            // A `../` chain is file-relative for `include_*!` but may be
            // manifest- or cwd-relative at run time: when resolving it from the
            // referencing file does not land under a prefix (or climbs above the
            // root), also record the chain-stripped path so no reference is lost.
            let stripped = line[j..end].trim_end_matches(['.', '/']);
            let mut paths = Vec::new();
            if literal.starts_with("../") {
                match normalize(&format!("{}/{literal}", dir_of(file))) {
                    Ok(p) => {
                        let under = prefixes.iter().any(|pre| p.starts_with(pre.as_str()));
                        paths.push(p);
                        if !under {
                            paths.push(stripped.to_string());
                        }
                    }
                    Err(_) => paths.push(stripped.to_string()),
                }
            } else {
                paths.push(literal.to_string());
            }
            for path in paths {
                out.push(CodeRef {
                    file: file.to_string(),
                    line: n + 1,
                    literal: literal.to_string(),
                    path,
                });
            }
            i = end;
        }
    }
    out
}

fn scanned_for_refs(path: &str) -> bool {
    let ext = ext_of(path);
    if let Some(rest) = path.strip_prefix("docs/") {
        return (path.starts_with("docs/harness/") && ext == "json")
            || (rest.starts_with("labs/") && DOCS_CODE_EXTS.contains(&ext));
    }
    path.starts_with(".github/")
        || path.rsplit('/').next() == Some("Justfile")
        || CODE_EXTS.contains(&ext)
}

fn subtree_of(path: &str) -> String {
    let mut parts = path.split('/');
    parts.next();
    match (parts.next(), parts.next()) {
        (Some(second), Some(_)) => second.to_string(),
        _ => "(root)".to_string(),
    }
}

fn prefix_of<'a>(literal: &str, prefixes: &'a [String]) -> Option<&'a str> {
    let stripped = literal.trim_start_matches("../");
    prefixes
        .iter()
        .map(String::as_str)
        .find(|p| stripped.starts_with(p))
}

/// `cargo xtask docs-inventory --out FILE.json [--prefix P ...]`
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut out_path = None;
    let mut prefixes: Vec<String> = DEFAULT_PREFIXES.iter().map(|s| (*s).to_string()).collect();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let v = match a.as_str() {
            "--out" | "--prefix" => it
                .next()
                .ok_or_else(|| format!("docs-inventory: {a} needs a value"))?
                .clone(),
            other => return Err(format!("docs-inventory: unknown argument `{other}`")),
        };
        if a == "--out" {
            out_path = Some(v);
        } else if !prefixes.contains(&v) {
            prefixes.push(v);
        }
    }
    let out_path = out_path.ok_or("docs-inventory: --out FILE.json is required")?;

    let mut tracked = tracked_files(root)?;
    tracked.sort();
    let docs: Vec<&String> = tracked.iter().filter(|p| p.starts_with("docs/")).collect();

    // Manifest coverage.
    let mut covered: BTreeSet<String> = BTreeSet::new();
    for m in tracked.iter().filter(|p| is_manifest(p)) {
        let Ok(bytes) = std::fs::read(root.join(m)) else {
            continue;
        };
        if let Ok(text) = String::from_utf8(bytes) {
            covered.extend(parse_manifest(dir_of(m), &text));
        }
    }

    // Markdown links: inbound counts per docs file, outbound per source.
    let docs_set: BTreeSet<&str> = docs.iter().map(|s| s.as_str()).collect();
    let mut inbound: HashMap<String, usize> = HashMap::new();
    let mut outbound: HashMap<&str, usize> = HashMap::new();
    for src in tracked.iter().filter(|p| is_md(p)) {
        let bytes = std::fs::read(root.join(src)).map_err(|e| format!("{src}: {e}"))?;
        let links = extract_links(&String::from_utf8_lossy(&bytes));
        if src.starts_with("docs/") {
            outbound.insert(src.as_str(), links.len());
        }
        for l in &links {
            if let Resolved::Repo(p) = resolve(src, &l.target) {
                if docs_set.contains(p.as_str()) {
                    *inbound.entry(p).or_insert(0) += 1;
                }
            }
        }
    }

    // File entries.
    let mut files = Vec::with_capacity(docs.len());
    let mut by_subtree: BTreeMap<String, usize> = BTreeMap::new();
    let mut covered_files = 0usize;
    for p in &docs {
        let bytes = std::fs::read(root.join(p.as_str())).map_err(|e| format!("{p}: {e}"))?;
        let is_covered = covered.contains(p.as_str());
        covered_files += usize::from(is_covered);
        *by_subtree.entry(subtree_of(p)).or_insert(0) += 1;
        files.push(json!({
            "path": p,
            "bytes": bytes.len(),
            "sha256": hex(&Sha256::digest(&bytes)),
            "ext": ext_of(p),
            "date": date_of(p),
            "sha256sums_covered": is_covered,
            "inbound_md_links": inbound.get(p.as_str()).copied().unwrap_or(0),
            "outbound_md_links": outbound.get(p.as_str()).copied().unwrap_or(0),
        }));
    }

    // Code references.
    let mut refs: Vec<CodeRef> = Vec::new();
    for p in tracked.iter().filter(|p| scanned_for_refs(p)) {
        let Ok(bytes) = std::fs::read(root.join(p)) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        refs.extend(scan_refs(p, &text, &prefixes));
    }
    refs.sort();
    let mut by_prefix: BTreeMap<String, usize> = BTreeMap::new();
    let mut dirs: BTreeSet<String> = BTreeSet::new();
    for r in &refs {
        let key = prefix_of(&r.literal, &prefixes).unwrap_or("(other)");
        *by_prefix.entry(key.to_string()).or_insert(0) += 1;
        dirs.insert(r.path.split('/').take(3).collect::<Vec<_>>().join("/"));
    }

    let summary = json!({
        "files_by_subtree": by_subtree,
        "covered_files": covered_files,
        "code_refs_by_prefix": by_prefix,
        "referenced_dirs": dirs,
    });
    let ref_json: Vec<Value> = refs
        .iter()
        .map(|r| json!({"file": r.file, "line": r.line, "literal": r.literal, "path": r.path}))
        .collect();
    let doc = json!({
        "schema": SCHEMA,
        "prefixes": prefixes,
        "files": files,
        "code_refs": ref_json,
        "summary": summary,
    });
    write_json(&out_path, &doc)?;
    println!(
        "inventory: {} docs files, {} sha256-covered, {} code refs",
        files.len(),
        covered_files,
        refs.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pf() -> Vec<String> {
        DEFAULT_PREFIXES.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn dotdot_chain_resolves_from_referencing_file() {
        let r = scan_refs(
            "crates/uor-r4-discrete/src/x.rs",
            r#"let p = "../../../docs/labs/a/b.json";"#,
            &pf(),
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].literal, "../../../docs/labs/a/b.json");
        assert_eq!(r[0].path, "docs/labs/a/b.json");
        assert_eq!(r[0].line, 1);
    }

    #[test]
    fn runtime_relative_chain_also_records_stripped_path() {
        // CARGO_MANIFEST_DIR-relative: resolving from the file gives
        // crates/docs/harness, so the stripped docs/harness is recorded too.
        let r = scan_refs(
            "crates/uor-r4-harness/tests/t.rs",
            r#"let d = "../../docs/harness";"#,
            &pf(),
        );
        let paths: Vec<&str> = r.iter().map(|c| c.path.as_str()).collect();
        assert!(paths.contains(&"docs/harness"), "{paths:?}");
        // A chain that climbs above the root is kept, not dropped.
        let up = scan_refs("a.rs", r#""../../../docs/x.md""#, &pf());
        assert_eq!(up.len(), 1);
        assert_eq!(up[0].path, "docs/x.md");
    }

    #[test]
    fn manifest_lines_parse() {
        let h = "a".repeat(64);
        let text =
            format!("{h}  one.txt\r\n{h} *sub/two.bin\n{h}\nnot a line\n{h}  ../up/three.md\n");
        assert_eq!(
            parse_manifest("docs/labs/x", &text),
            vec![
                "docs/labs/x/one.txt",
                "docs/labs/x/sub/two.bin",
                "docs/labs/up/three.md"
            ]
        );
        assert!(is_manifest("docs/a/SHA256SUMS"));
        assert!(is_manifest("docs/a/foo.SHA256SUMS.txt"));
        assert!(is_manifest("docs/a/My_Sha256Sums.TXT"));
        assert!(!is_manifest("docs/a/sums.txt"));
    }

    #[test]
    fn dates_from_paths() {
        assert_eq!(
            date_of("docs/labs/x-2026-10-19/README.md").as_deref(),
            Some("2026-10-19")
        );
        assert_eq!(
            date_of("docs/history/foo-20261010.patch").as_deref(),
            Some("2026-10-10")
        );
        assert_eq!(date_of("docs/labs/plain/README.md"), None);
        assert_eq!(date_of("docs/a/2023-01-01.md"), None);
    }

    #[test]
    fn prefix_needs_a_path_boundary_and_trims() {
        assert!(scan_refs("a.rs", "foodocs/x", &pf()).is_empty());
        let r = scan_refs("a.rs", r#"open("docs/x"); see docs/y/z.md."#, &pf());
        let lits: Vec<&str> = r.iter().map(|c| c.literal.as_str()).collect();
        assert_eq!(lits, vec!["docs/x", "docs/y/z.md"]);
    }

    #[test]
    fn extra_prefix_and_scope() {
        let mut p = pf();
        p.push("zz/".to_string());
        assert_eq!(scan_refs("a.rs", "zz/q", &p).len(), 1);
        assert!(scanned_for_refs(".github/workflows/ci.yml"));
        assert!(scanned_for_refs("docs/harness/x/y.json"));
        assert!(scanned_for_refs("docs/labs/a/b.rs"));
        assert!(!scanned_for_refs("docs/labs/a/b.md"));
        assert_eq!(subtree_of("docs/README.md"), "(root)");
        assert_eq!(subtree_of("docs/labs/a.md"), "labs");
    }
}
