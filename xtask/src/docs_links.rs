//! Markdown link checker for the docs reorganization (`cargo xtask links`).
//!
//! About 9,000 files move from `docs/` into `archive/` byte for byte, so the
//! only safe claim afterwards is "no link broke that was not already
//! broken". This task makes that claim checkable:
//!
//! * the universe is the git index (`git ls-files -z`), not the filesystem,
//!   so a proposed move can be simulated without moving a file;
//! * every tracked `*.md` is a source; inline links/images, reference
//!   definitions, HTML `href`/`src` and `<https://…>` autolinks are
//!   extracted outside fenced blocks and inline code spans;
//! * a target exists when it is a tracked file, a directory that prefixes a
//!   tracked file, or the repository root; other `http(s)` links are
//!   counted, never fetched;
//! * `--baseline` compares broken links keyed by `(source, resolved)`;
//!   `--renames-from REV` and `--simulate MAP.tsv` map the baseline (and,
//!   for a simulation, the universe and source locations) through a path
//!   map first.
//!
//! `--renames-from REV` reads `git diff --cached -M100% --name-status -z REV`,
//! i.e. REV against the index: that is the same state `git ls-files` lists,
//! so it covers renames that are committed (REV..HEAD) and renames that are
//! only staged. Only `R100` lines enter the map. Directory renames are
//! derived from the file renames and kept only when unambiguous and when the
//! old directory no longer holds any tracked file.
//!
//! A broken link in the current state is "pre-existing" when an unconsumed
//! baseline entry matches it with the source mapped and the resolved path
//! either mapped or unmapped. The unmapped alternative is needed because a
//! file that did not move keeps its link text, so its already-broken target
//! does not move with the directory.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

/// Schema tag of the JSON report and of the baselines it is compared with.
pub const SCHEMA: &str = "uor-r4.docs-links/1";

/// One link target extracted from a markdown source, with its 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub line: usize,
    pub target: String,
}

/// What a link target means once resolved against its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// Pure anchor, `mailto:`, `tel:`, `data:`, empty.
    Ignored,
    /// `http(s)` or another `scheme://` target: counted, never checked.
    External,
    /// Repository-relative normalized path (`""` is the root).
    Repo(String),
    /// A path that climbs above the root; kept in `../`-prefixed form.
    Escapes(String),
}

/// A link whose target does not exist in the universe.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Broken {
    pub source: String,
    pub line: usize,
    pub target: String,
    pub resolved: String,
}

/// The result of checking every source.
#[derive(Debug, Default)]
pub struct Report {
    pub sources: usize,
    pub links_checked: usize,
    pub external_skipped: usize,
    pub broken: Vec<Broken>,
}

/// The set of tracked paths and the directories they imply.
pub struct Universe {
    files: HashSet<String>,
    dirs: HashSet<String>,
}

impl Universe {
    pub fn new<I: IntoIterator<Item = String>>(paths: I) -> Self {
        let mut files = HashSet::new();
        let mut dirs = HashSet::new();
        for p in paths {
            let mut end = p.len();
            while let Some(i) = p[..end].rfind('/') {
                if !dirs.insert(p[..i].to_string()) {
                    break;
                }
                end = i;
            }
            files.insert(p);
        }
        Universe { files, dirs }
    }

    pub fn exists(&self, path: &str) -> bool {
        path.is_empty() || self.files.contains(path) || self.dirs.contains(path)
    }

    fn is_dir(&self, path: &str) -> bool {
        self.dirs.contains(path)
    }
}

/// Run git at the repository root and return its stdout.
pub fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

/// Every path in the index, sorted and de-duplicated.
pub fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let out = git(root, &["ls-files", "-z"])?;
    let mut v: Vec<String> = out
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    v.sort();
    v.dedup();
    Ok(v)
}

/// Whether a path is a markdown source.
pub fn is_md(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".md")
}

/// A fence line: its character, run length and whether only whitespace
/// follows the run. Up to three leading spaces are allowed.
fn fence_marker(line: &str) -> Option<(u8, usize, bool)> {
    let b = line.as_bytes();
    let indent = b.iter().take_while(|c| **c == b' ').count();
    if indent > 3 {
        return None;
    }
    let ch = *b.get(indent)?;
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let len = b[indent..].iter().take_while(|c| **c == ch).count();
    if len < 3 {
        return None;
    }
    let rest = &line[indent + len..];
    if ch == b'`' && rest.contains('`') {
        return None;
    }
    Some((ch, len, rest.trim().is_empty()))
}

/// Replace every inline code span (matching backtick runs) with spaces.
fn blank_code_spans(line: &str) -> String {
    let mut b = line.as_bytes().to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let n = b[i..].iter().take_while(|c| **c == b'`').count();
        let mut j = i + n;
        let mut close = None;
        while j < b.len() {
            if b[j] == b'`' {
                let m = b[j..].iter().take_while(|c| **c == b'`').count();
                if m == n {
                    close = Some(j + m);
                    break;
                }
                j += m;
            } else {
                j += 1;
            }
        }
        match close {
            Some(end) => {
                for c in &mut b[i..end] {
                    *c = b' ';
                }
                i = end;
            }
            None => i += n,
        }
    }
    // Only whole spans became ASCII spaces, so the bytes stay valid UTF-8.
    String::from_utf8_lossy(&b).into_owned()
}

/// Parse a link destination at the start of `s` (after `](` or `]:`).
fn parse_dest(s: &str) -> Option<String> {
    let s = s.trim_start_matches([' ', '\t']);
    if let Some(rest) = s.strip_prefix('<') {
        let end = rest.find('>')?;
        return Some(rest[..end].to_string());
    }
    let mut depth = 0usize;
    let mut end = s.len();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '(' => depth += 1,
            ')' if depth == 0 => {
                end = i;
                break;
            }
            ')' => depth -= 1,
            c if c.is_whitespace() => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    let d = &s[..end];
    (!d.is_empty()).then(|| d.to_string())
}

fn scan_line(line: &str, line_no: usize, out: &mut Vec<Link>) {
    let mut push = |t: String| {
        out.push(Link {
            line: line_no,
            target: t,
        })
    };
    // Reference definition: `[id]: target` (not a `[^footnote]:`).
    let indent = line.bytes().take_while(|c| *c == b' ').count();
    if indent <= 3 {
        let t = &line[indent..];
        if t.starts_with('[') && !t.starts_with("[^") {
            if let Some(j) = t.find("]:") {
                if j > 1 && !t[1..j].contains(']') {
                    if let Some(d) = parse_dest(&t[j + 2..]) {
                        push(d);
                    }
                    return;
                }
            }
        }
    }
    // Inline links and images: `[t](dest "title")`, `![a](dest)`.
    let mut from = 0;
    while let Some(p) = line[from..].find("](") {
        let start = from + p + 2;
        if let Some(d) = parse_dest(&line[start..]) {
            push(d);
        }
        from = start;
    }
    // HTML attributes and autolinks. ASCII lowercasing keeps byte offsets.
    let lower = line.to_ascii_lowercase();
    let bytes = line.as_bytes();
    for needle in ["href=", "src="] {
        let mut from = 0;
        while let Some(p) = lower[from..].find(needle) {
            let at = from + p;
            from = at + needle.len();
            if at == 0 || !bytes[at - 1].is_ascii_whitespace() {
                continue;
            }
            let q = match bytes.get(from) {
                Some(b'"') => '"',
                Some(b'\'') => '\'',
                _ => continue,
            };
            if let Some(end) = line[from + 1..].find(q) {
                push(line[from + 1..from + 1 + end].to_string());
                from = from + 1 + end;
            }
        }
    }
    for needle in ["<http://", "<https://"] {
        let mut from = 0;
        while let Some(p) = lower[from..].find(needle) {
            let at = from + p;
            from = at + 1;
            if at > 0 && bytes[at - 1] == b'(' {
                continue; // `](<https://…>)` is an inline destination
            }
            if let Some(end) = line[at + 1..].find('>') {
                let t = &line[at + 1..at + 1 + end];
                if !t.chars().any(char::is_whitespace) {
                    push(t.to_string());
                }
            }
        }
    }
}

/// Every link target in a markdown text, outside fences and code spans. A
/// fence still open at the end of the text was never closed: it is rescanned
/// as ordinary text, so one stray fence cannot hide every later link.
pub fn extract_links(text: &str) -> Vec<Link> {
    let lines: Vec<&str> = text.lines().collect();
    let mut unclosed: Vec<usize> = Vec::new();
    loop {
        let mut out = Vec::new();
        let mut fence: Option<(u8, usize, usize)> = None;
        for (idx, raw) in lines.iter().enumerate() {
            if let Some((ch, len, blank_rest)) = fence_marker(raw) {
                match fence {
                    None if !unclosed.contains(&idx) => {
                        fence = Some((ch, len, idx));
                        continue;
                    }
                    Some((fc, fl, _)) if fc == ch && len >= fl && blank_rest => {
                        fence = None;
                        continue;
                    }
                    _ => {}
                }
            }
            if fence.is_some() {
                continue;
            }
            scan_line(&blank_code_spans(raw), idx + 1, &mut out);
        }
        match fence {
            Some((_, _, open)) => unclosed.push(open),
            None => return out,
        }
    }
}

/// Resolve `.` and `..`; `Err` carries the `../`-prefixed form of a path
/// that climbs above the root.
pub fn normalize(path: &str) -> Result<String, String> {
    let mut parts: Vec<&str> = Vec::new();
    let mut escaped = 0usize;
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    escaped += 1;
                }
            }
            s => parts.push(s),
        }
    }
    let joined = parts.join("/");
    if escaped == 0 {
        Ok(joined)
    } else {
        Err(format!("{}{joined}", "../".repeat(escaped)))
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hex = std::str::from_utf8(&b[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok());
            if let Some(v) = hex {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The repo-absolute remainder of a `github.com/UOR-Foundation/uor-r4`
/// `blob|tree|raw/main/` URL.
fn strip_github(t: &str) -> Option<&str> {
    let lower = t.to_ascii_lowercase();
    let scheme = ["https://", "http://"]
        .iter()
        .find(|s| lower.starts_with(**s))?
        .len();
    let mut at = scheme;
    if lower[at..].starts_with("www.") {
        at += 4;
    }
    let repo = "github.com/uor-foundation/uor-r4/";
    if !lower[at..].starts_with(repo) {
        return None;
    }
    at += repo.len();
    for kind in ["blob/main/", "tree/main/", "raw/main/"] {
        if lower[at..].starts_with(kind) {
            return Some(&t[at + kind.len()..]);
        }
    }
    None
}

/// Resolve a link target found in `source` (a repository-relative path).
pub fn resolve(source: &str, target: &str) -> Resolved {
    let t = target.trim();
    let lower = t.to_ascii_lowercase();
    if t.is_empty()
        || ["mailto:", "tel:", "data:"]
            .iter()
            .any(|s| lower.starts_with(s))
    {
        return Resolved::Ignored;
    }
    let (path_part, repo_abs) = match strip_github(t) {
        Some(rest) => (rest, true),
        None if t.contains("://") => return Resolved::External,
        None => (t, false),
    };
    let cut = path_part.find(['#', '?']).unwrap_or(path_part.len());
    let decoded = percent_decode(&path_part[..cut]);
    if decoded.is_empty() && !repo_abs {
        return Resolved::Ignored;
    }
    let joined = if repo_abs {
        decoded
    } else if let Some(r) = decoded.strip_prefix('/') {
        r.to_string()
    } else {
        match source.rfind('/') {
            Some(i) => format!("{}/{decoded}", &source[..i]),
            None => decoded,
        }
    };
    match normalize(&joined) {
        Ok(p) => Resolved::Repo(p),
        Err(p) => Resolved::Escapes(p),
    }
}

/// Check one source (`source` is the location links resolve from).
pub fn check_one(report: &mut Report, universe: &Universe, source: &str, text: &str) {
    report.sources += 1;
    for link in extract_links(text) {
        let resolved = match resolve(source, &link.target) {
            Resolved::Ignored => continue,
            Resolved::External => {
                report.external_skipped += 1;
                continue;
            }
            Resolved::Repo(p) => {
                report.links_checked += 1;
                if universe.exists(&p) {
                    continue;
                }
                p
            }
            Resolved::Escapes(p) => {
                report.links_checked += 1;
                p
            }
        };
        report.broken.push(Broken {
            source: source.to_string(),
            line: link.line,
            target: link.target,
            resolved,
        });
    }
}

/// A path map applied on path-segment boundaries, longest prefix first.
#[derive(Debug, Default)]
pub struct PathMap {
    entries: HashMap<String, String>,
    /// Virtual files (`+path` lines) that a simulated move would add, such as
    /// pointer stubs; they join the universe but are never link sources.
    added: Vec<String>,
}

impl PathMap {
    fn insert(&mut self, old: &str, new: &str) {
        let old = old.trim_end_matches('/').to_string();
        let new = new.trim_end_matches('/').to_string();
        self.entries.insert(old, new);
    }

    /// `old<TAB>new` lines and `+path` virtual additions; `#` comments and blank
    /// lines are skipped.
    pub fn parse_tsv(text: &str) -> Result<Self, String> {
        let mut map = PathMap::default();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(path) = line.strip_prefix('+') {
                let p = normalize(path.trim())
                    .map_err(|_| format!("map line {}: escapes root", i + 1))?;
                map.added.push(p);
                continue;
            }
            let mut f = line.split('\t');
            match (f.next(), f.next(), f.next()) {
                (Some(o), Some(n), None) if !o.trim().is_empty() && !n.trim().is_empty() => {
                    let o = normalize(o.trim())
                        .map_err(|_| format!("map line {}: escapes root", i + 1))?;
                    let n = normalize(n.trim())
                        .map_err(|_| format!("map line {}: escapes root", i + 1))?;
                    map.insert(&o, &n);
                }
                _ => return Err(format!("map line {}: expected `old<TAB>new`", i + 1)),
            }
        }
        Ok(map)
    }

    /// The tracked set as it would be after the simulated move, plus the
    /// virtual `+path` additions.
    pub fn universe<'a, I: IntoIterator<Item = &'a str>>(&self, paths: I) -> Universe {
        Universe::new(
            paths
                .into_iter()
                .map(|p| self.map(p))
                .chain(self.added.iter().cloned()),
        )
    }

    /// `R100` entries of `git diff --name-status -z` output.
    pub fn parse_renames_z(out: &[u8]) -> Self {
        let mut map = PathMap::default();
        let mut tok = out
            .split(|b| *b == 0)
            .map(|s| String::from_utf8_lossy(s).into_owned());
        while let Some(status) = tok.next() {
            if status.is_empty() {
                continue;
            }
            if status.starts_with('R') || status.starts_with('C') {
                let (Some(old), Some(new)) = (tok.next(), tok.next()) else {
                    break;
                };
                if status == "R100" {
                    map.insert(&old, &new);
                }
            } else {
                tok.next();
            }
        }
        map
    }

    /// Add directory renames implied by the file renames: unambiguous ones
    /// whose old directory holds no tracked file in `current`.
    pub fn add_dir_renames(&mut self, current: &Universe) {
        let mut dirs: HashMap<String, Option<String>> = HashMap::new();
        for (old, new) in &self.entries {
            let o: Vec<&str> = old.split('/').collect();
            let n: Vec<&str> = new.split('/').collect();
            let common = o
                .iter()
                .rev()
                .zip(n.iter().rev())
                .take_while(|(a, b)| a == b)
                .count();
            for j in 1..=common {
                if j >= o.len() || j >= n.len() {
                    break;
                }
                let od = o[..o.len() - j].join("/");
                let nd = n[..n.len() - j].join("/");
                match dirs.get(&od) {
                    Some(Some(prev)) if *prev != nd => {
                        dirs.insert(od, None);
                    }
                    Some(_) => {}
                    None => {
                        dirs.insert(od, Some(nd));
                    }
                }
            }
        }
        for (od, nd) in dirs {
            if let Some(nd) = nd {
                if !current.is_dir(&od) && !self.entries.contains_key(&od) {
                    self.entries.insert(od, nd);
                }
            }
        }
    }

    /// Map a path through the longest matching entry.
    pub fn map(&self, path: &str) -> String {
        if self.entries.is_empty() {
            return path.to_string();
        }
        let mut end = path.len();
        loop {
            if let Some(new) = self.entries.get(&path[..end]) {
                let rest = &path[end..];
                return if new.is_empty() {
                    rest.trim_start_matches('/').to_string()
                } else {
                    format!("{new}{rest}")
                };
            }
            match path[..end].rfind('/') {
                Some(i) => end = i,
                None => return path.to_string(),
            }
        }
    }
}

/// Outcome of comparing current broken links with a mapped baseline.
#[derive(Debug)]
pub struct Comparison {
    pub new: Vec<Broken>,
    pub fixed: usize,
}

/// Compare; each baseline `(source, resolved)` absorbs one current entry
/// keyed by its mapped source with the mapped or the unmapped resolved path.
/// Known limitation: the unmapped-resolved leniency can, rarely, absorb a link
/// that a move newly broke when its new resolved path equals an old broken
/// path from the same source (a false "not new").
pub fn compare(baseline: &[(String, String)], maps: &[&PathMap], current: &[Broken]) -> Comparison {
    let map_all = |p: &str| maps.iter().fold(p.to_string(), |acc, m| m.map(&acc));
    let mut by_mapped: HashMap<(String, String), Vec<usize>> = HashMap::new();
    let mut by_source_only: HashMap<(String, String), Vec<usize>> = HashMap::new();
    for (i, (s, r)) in baseline.iter().enumerate() {
        let ms = map_all(s);
        by_mapped
            .entry((ms.clone(), map_all(r)))
            .or_default()
            .push(i);
        by_source_only.entry((ms, r.clone())).or_default().push(i);
    }
    let mut used = vec![false; baseline.len()];
    let mut take =
        |idx: &mut HashMap<(String, String), Vec<usize>>, key: (String, String)| -> bool {
            if let Some(list) = idx.get_mut(&key) {
                while let Some(i) = list.pop() {
                    if !used[i] {
                        used[i] = true;
                        return true;
                    }
                }
            }
            false
        };
    let mut leftover = Vec::new();
    for b in current {
        let key = (b.source.clone(), b.resolved.clone());
        if !take(&mut by_mapped, key) {
            leftover.push(b);
        }
    }
    let mut new = Vec::new();
    for b in leftover {
        let key = (b.source.clone(), b.resolved.clone());
        if !take(&mut by_source_only, key) {
            new.push(b.clone());
        }
    }
    let fixed = used.iter().filter(|u| !**u).count();
    Comparison { new, fixed }
}

pub fn report_json(r: &Report) -> Value {
    let broken: Vec<Value> = r
        .broken
        .iter()
        .map(|b| {
            json!({"source": b.source, "line": b.line, "target": b.target, "resolved": b.resolved})
        })
        .collect();
    json!({
        "schema": SCHEMA,
        "sources": r.sources,
        "links_checked": r.links_checked,
        "external_skipped": r.external_skipped,
        "broken": broken,
    })
}

fn load_baseline(path: &str) -> Result<Vec<(String, String)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
    if v.get("schema").and_then(Value::as_str) != Some(SCHEMA) {
        return Err(format!("{path}: schema is not {SCHEMA}"));
    }
    let list = v
        .get("broken")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{path}: no `broken` array"))?;
    list.iter()
        .map(|e| {
            let s = e.get("source").and_then(Value::as_str);
            let r = e.get("resolved").and_then(Value::as_str);
            match (s, r) {
                (Some(s), Some(r)) => Ok((s.to_string(), r.to_string())),
                _ => Err(format!("{path}: entry without source/resolved")),
            }
        })
        .collect()
}

/// Write `value` as pretty JSON, creating the parent directory.
pub fn write_json(path: &str, value: &Value) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    text.push('\n');
    std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))
}

/// Check every tracked markdown source, optionally relocated by `sim`.
pub fn check_repo(root: &Path, tracked: &[String], sim: &PathMap) -> Result<Report, String> {
    let universe = sim.universe(tracked.iter().map(String::as_str));
    let mut report = Report::default();
    for src in tracked.iter().filter(|p| is_md(p)) {
        let bytes = std::fs::read(root.join(src)).map_err(|e| format!("{src}: {e}"))?;
        check_one(
            &mut report,
            &universe,
            &sim.map(src),
            &String::from_utf8_lossy(&bytes),
        );
    }
    report.broken.sort();
    Ok(report)
}

/// `cargo xtask links [--json OUT] [--baseline FILE] [--renames-from REV] [--simulate MAP.tsv]`
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut json_out = None;
    let mut baseline = None;
    let mut renames_from = None;
    let mut simulate = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--json" => &mut json_out,
            "--baseline" => &mut baseline,
            "--renames-from" => &mut renames_from,
            "--simulate" => &mut simulate,
            other => return Err(format!("links: unknown argument `{other}`")),
        };
        *slot = Some(
            it.next()
                .ok_or_else(|| format!("links: {a} needs a value"))?
                .clone(),
        );
    }
    if simulate.is_some() && baseline.is_none() {
        return Err("links: --simulate needs --baseline".into());
    }

    let tracked = tracked_files(root)?;
    let sim = match &simulate {
        Some(p) => {
            PathMap::parse_tsv(&std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?)?
        }
        None => PathMap::default(),
    };
    let report = check_repo(root, &tracked, &sim)?;
    println!(
        "links: {} sources, {} checked, {} broken, {} external skipped",
        report.sources,
        report.links_checked,
        report.broken.len(),
        report.external_skipped
    );
    if let Some(out) = &json_out {
        write_json(out, &report_json(&report))?;
    }

    let Some(base_path) = baseline else {
        return Ok(());
    };
    let base = load_baseline(&base_path)?;
    let renames = match &renames_from {
        Some(rev) => {
            let out = git(
                root,
                &["diff", "--cached", "-M100%", "--name-status", "-z", rev],
            )?;
            let mut m = PathMap::parse_renames_z(&out);
            m.add_dir_renames(&Universe::new(tracked.iter().cloned()));
            m
        }
        None => PathMap::default(),
    };
    let cmp = compare(&base, &[&renames, &sim], &report.broken);
    println!("new broken: {}", cmp.new.len());
    println!("fixed: {}", cmp.fixed);
    let mut new = cmp.new;
    new.sort();
    for b in &new {
        println!(
            "  NEW {}:{} {} -> {}",
            b.source, b.line, b.target, b.resolved
        );
    }
    if new.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "links: {} new broken link(s) against {base_path}",
            new.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(text: &str) -> Vec<String> {
        extract_links(text).into_iter().map(|l| l.target).collect()
    }

    fn report_for(paths: &[&str], sources: &[(&str, &str)], sim: &PathMap) -> Report {
        let universe = sim.universe(paths.iter().copied());
        let mut r = Report::default();
        for (src, text) in sources {
            check_one(&mut r, &universe, &sim.map(src), text);
        }
        r.broken.sort();
        r
    }

    fn keys(r: &Report) -> Vec<(String, String)> {
        r.broken
            .iter()
            .map(|b| (b.source.clone(), b.resolved.clone()))
            .collect()
    }

    #[test]
    fn fenced_blocks_and_inline_code_are_skipped() {
        let text = "a [x](one.md)\n```rust\n[y](two.md)\n```\n   ~~~~\n[z](three.md)\n~~~~\n`[w](four.md)` and ``[v](five.md)`` [u](six.md)\n";
        assert_eq!(targets(text), vec!["one.md", "six.md"]);
        let l = extract_links(text);
        assert_eq!((l[0].line, l[1].line), (1, 8));
    }

    #[test]
    fn reference_html_angle_autolink_and_parens_are_extracted() {
        let text = "[id]: ref/a.md \"t\"\n[^1]: footnote text\n<a href=\"h.md\">x</a> <img src='i.png'>\n[s](<with space.md>) ![img](p(1).md \"title\")\nsee <https://example.org/x>\n";
        assert_eq!(
            targets(text),
            vec![
                "ref/a.md",
                "h.md",
                "i.png",
                "with space.md",
                "p(1).md",
                "https://example.org/x"
            ]
        );
    }

    #[test]
    fn resolution_rules() {
        assert_eq!(
            resolve(
                "docs/x.md",
                "https://github.com/UOR-Foundation/uor-r4/blob/main/crates/a.rs#L3"
            ),
            Resolved::Repo("crates/a.rs".into())
        );
        assert_eq!(
            resolve(
                "docs/x.md",
                "https://github.com/UOR-Foundation/uor-r4/tree/main/docs"
            ),
            Resolved::Repo("docs".into())
        );
        assert_eq!(
            resolve("docs/x.md", "https://example.org/a"),
            Resolved::External
        );
        assert_eq!(resolve("docs/x.md", "#anchor"), Resolved::Ignored);
        assert_eq!(resolve("docs/x.md", "mailto:a@b"), Resolved::Ignored);
        assert_eq!(
            resolve("docs/x.md", "my%20file.md?x=1#h"),
            Resolved::Repo("docs/my file.md".into())
        );
        assert_eq!(
            resolve("docs/a/x.md", "/README.md"),
            Resolved::Repo("README.md".into())
        );
        assert_eq!(
            resolve("docs/a/x.md", "./../b/./c.md"),
            Resolved::Repo("docs/b/c.md".into())
        );
    }

    #[test]
    fn directory_link_exists_and_root_escape_is_broken() {
        let r = report_for(
            &["docs/labs/x/README.md", "docs/a.md"],
            &[(
                "docs/a.md",
                "[d](labs/x/) [d2](labs) [root](../) [up](../../out.md)",
            )],
            &PathMap::default(),
        );
        assert_eq!(r.links_checked, 4);
        assert_eq!(
            keys(&r),
            vec![("docs/a.md".to_string(), "../out.md".to_string())]
        );
    }

    #[test]
    fn simulate_moved_source_relative_link_is_new_broken() {
        let paths = ["docs/a/s.md", "docs/c.md"];
        let srcs = [("docs/a/s.md", "[c](../c.md)")];
        let base = report_for(&paths, &srcs, &PathMap::default());
        assert!(base.broken.is_empty());
        let sim = PathMap::parse_tsv("# move\ndocs/a\tarchive/a\n").expect("map");
        let cur = report_for(&paths, &srcs, &sim);
        let cmp = compare(&keys(&base), &[&sim], &cur.broken);
        assert_eq!(cmp.new.len(), 1);
        assert_eq!(cmp.new[0].resolved, "archive/c.md");
    }

    #[test]
    fn unclosed_fence_does_not_hide_later_links() {
        let text = "intro\n```rust\nlet x = 1;\n[later](a.md)\n";
        let links = extract_links(text);
        assert_eq!(links.len(), 1, "a never-closed fence is rescanned as text");
        let closed = "```\n[hidden](b.md)\n```\n[shown](c.md)\n";
        assert_eq!(extract_links(closed).len(), 1);
    }

    #[test]
    fn simulate_virtual_stub_repairs_moved_source_link() {
        let paths = ["docs/a/s.md", "docs/c.md"];
        let srcs = [("docs/a/s.md", "[c](../c.md)")];
        let base = report_for(&paths, &srcs, &PathMap::default());
        let sim = PathMap::parse_tsv("docs/a\tarchive/a\n+archive/c.md\n").expect("map");
        let cur = report_for(&paths, &srcs, &sim);
        let cmp = compare(&keys(&base), &[&sim], &cur.broken);
        assert!(
            cmp.new.is_empty(),
            "the stub at archive/c.md resolves the link"
        );
    }

    #[test]
    fn simulate_with_mapped_baseline_reports_no_false_new() {
        let sim = PathMap::parse_tsv("a/\tb/\n").expect("map");
        // x.md -> a/f.md is already broken (a/ exists, f.md does not), and
        // a/s.md has an already-broken relative link that moves with it.
        let paths = ["x.md", "a/g.md", "a/s.md"];
        let srcs = [
            ("x.md", "[f](a/f.md)"),
            ("a/s.md", "[m](missing.md) [g](g.md)"),
        ];
        let base = report_for(&paths, &srcs, &PathMap::default());
        assert_eq!(base.broken.len(), 2);
        let cur = report_for(&paths, &srcs, &sim);
        let cmp = compare(&keys(&base), &[&sim], &cur.broken);
        assert!(cmp.new.is_empty(), "{:?}", cmp.new);
        assert_eq!(cmp.fixed, 0);
        // A link that worked before the move does break, and is reported.
        let paths = ["x.md", "a/f.md"];
        let srcs = [("x.md", "[f](a/f.md)")];
        let base = report_for(&paths, &srcs, &PathMap::default());
        let cur = report_for(&paths, &srcs, &sim);
        assert_eq!(compare(&keys(&base), &[&sim], &cur.broken).new.len(), 1);
    }

    #[test]
    fn baseline_compare_counts_duplicates() {
        let b = |line| Broken {
            source: "s.md".into(),
            line,
            target: "m.md".into(),
            resolved: "m.md".into(),
        };
        let base = vec![("s.md".to_string(), "m.md".to_string())];
        let cmp = compare(&base, &[], &[b(1), b(2)]);
        assert_eq!(cmp.new.len(), 1);
        let cmp = compare(&base, &[], &[]);
        assert_eq!(cmp.fixed, 1);
    }

    #[test]
    fn renames_parser_keeps_only_r100() {
        let out = b"R100\0docs/history/a.md\0archive/history/a.md\0M\0x.md\0R087\0p.md\0q.md\0A\0n.md\0R100\0docs/history/sub/b.md\0archive/history/sub/b.md\0";
        let mut m = PathMap::parse_renames_z(out);
        assert_eq!(m.map("docs/history/a.md"), "archive/history/a.md");
        assert_eq!(m.map("p.md"), "p.md");
        assert_eq!(m.map("x.md"), "x.md");
        m.add_dir_renames(&Universe::new(["docs/live.md".to_string()]));
        assert_eq!(
            m.map("docs/history/sub/gone.md"),
            "archive/history/sub/gone.md"
        );
        assert_eq!(m.map("docs/other.md"), "docs/other.md");
    }

    #[test]
    fn path_map_longest_prefix_on_segment_boundary() {
        let m = PathMap::parse_tsv("docs/history\tarchive/history\ndocs/history/keep\tdocs/kept\n")
            .expect("map");
        assert_eq!(m.map("docs/history/a.md"), "archive/history/a.md");
        assert_eq!(m.map("docs/history/keep/b.md"), "docs/kept/b.md");
        assert_eq!(m.map("docs/historyx/c.md"), "docs/historyx/c.md");
        assert_eq!(m.map("docs/history"), "archive/history");
        assert!(PathMap::parse_tsv("only-one-field\n").is_err());
    }
}
