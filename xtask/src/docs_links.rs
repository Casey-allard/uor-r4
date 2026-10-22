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
//!
//! Replaced stubs: when a cutover pointer stub at `archive/X` is replaced by
//! the record it named, git reports `D docs/X` plus `M archive/X`, not a
//! rename. A deleted path is paired with a modified path whose working-tree
//! blob equals the deleted path's blob at REV; the pair maps baseline entries
//! only, and only byte-equal pairs count.
//!
//! Moved targets: records under `archive/` and `docs/harness/prep/` are
//! immutable, so a PR that moves a live target into `archive/` cannot fix the
//! links they hold. A NEW broken link is reported as `MOVED` (count line
//! `moved (immutable source)`) and does not fail the gate when its source is
//! immutable, unchanged since REV (`git diff --quiet REV -- source`) and its
//! target is the old side of an R100 rename. Every other NEW link fails.
//!
//! Archived copies: §4.7 copies a live file byte for byte into `archive/` and
//! leaves the original in place as a pointer stub, so there is no rename and
//! the copy's relative links break at the archive depth. A NEW broken link is
//! reported as `COPIED` (count line `copied (archived copy)`) and does not
//! fail the gate when its source is under `archive/`, did not exist at REV,
//! has a working-tree blob equal to the blob of a path `O` at REV (one
//! `git ls-tree -r REV` index by blob id; the lexicographically first path
//! wins), and the same link resolved from `O` names a path that exists now.
//! An edited copy, or a copy whose link is broken from `O` too, still fails.
//! Neither `MOVED` nor `COPIED` applies under `--simulate`.

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

/// A git command bound to `root` only: an inherited `GIT_DIR` (set inside git
/// hooks) would otherwise point every call at another repository.
fn git_command(root: &Path) -> Command {
    let mut c = Command::new("git");
    c.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .arg("-C")
        .arg(root);
    c
}

/// Run git at the repository root and return its stdout.
pub fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = git_command(root)
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
pub fn fence_marker(line: &str) -> Option<(u8, usize, bool)> {
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
pub fn blank_code_spans(line: &str) -> String {
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

    /// The longest entry `(old, new)` whose `old` is `path` or a segment
    /// prefix of it, as used by [`PathMap::map`].
    pub fn matched(&self, path: &str) -> Option<(&str, &str)> {
        let mut end = path.len();
        loop {
            if let Some((old, new)) = self.entries.get_key_value(&path[..end]) {
                return Some((old.as_str(), new.as_str()));
            }
            end = path[..end].rfind('/')?;
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

/// Paths an immutable record may keep even when its target moved away.
pub fn is_immutable_source(path: &str) -> bool {
    path.starts_with("archive/") || path.starts_with("docs/harness/prep/")
}

/// `D` and `M` paths of `git diff --name-status -z` output.
pub fn parse_deleted_modified_z(out: &[u8]) -> (Vec<String>, Vec<String>) {
    let (mut deleted, mut modified) = (Vec::new(), Vec::new());
    let mut tok = out
        .split(|b| *b == 0)
        .map(|s| String::from_utf8_lossy(s).into_owned());
    while let Some(status) = tok.next() {
        if status.is_empty() {
            continue;
        }
        if status.starts_with('R') || status.starts_with('C') {
            tok.next();
            tok.next();
        } else if let Some(path) = tok.next() {
            match status.as_str() {
                "D" => deleted.push(path),
                "M" => modified.push(path),
                _ => {}
            }
        }
    }
    (deleted, modified)
}

fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Pair each deleted `(path, blob at REV)` with a modified `(path, working
/// blob)` of byte-equal content: the replaced-stub case, where a record
/// takes the place of the pointer stub that named it. Each path pairs once.
/// When several deleted or several modified paths share a blob, only a pair
/// with the same file name that is unique on both sides is taken; any other
/// shape is ambiguous and that blob is left unpaired.
pub fn pair_replaced_stubs(
    deleted: &[(String, String)],
    modified: &[(String, String)],
) -> Vec<(String, String)> {
    let mut taken = HashSet::new();
    let mut pairs = Vec::new();
    for (old, blob) in deleted {
        let dels: Vec<&String> = deleted
            .iter()
            .filter(|(_, b)| b == blob)
            .map(|(p, _)| p)
            .collect();
        let mods: Vec<&String> = modified
            .iter()
            .filter(|(_, b)| b == blob)
            .map(|(p, _)| p)
            .collect();
        let hit = if dels.len() == 1 && mods.len() == 1 {
            Some(mods[0])
        } else {
            let same = |ps: &[&String]| -> Vec<String> {
                ps.iter()
                    .filter(|p| basename(p) == basename(old))
                    .map(|p| (*p).clone())
                    .collect()
            };
            let (d, m) = (same(&dels), same(&mods));
            if d.len() == 1 && m.len() == 1 {
                mods.iter().copied().find(|p| **p == m[0])
            } else {
                None
            }
        };
        if let Some(new) = hit {
            if !taken.contains(new.as_str()) {
                taken.insert(new.as_str());
                pairs.push((old.clone(), new.clone()));
            }
        }
    }
    pairs
}

/// Build the replaced-stub map from the `--renames-from` diff output.
fn replaced_stub_map(root: &Path, rev: &str, out: &[u8]) -> Result<PathMap, String> {
    let (del, modi) = parse_deleted_modified_z(out);
    let mut map = PathMap::default();
    if del.is_empty() {
        return Ok(map);
    }
    let mut deleted = Vec::new();
    for p in del {
        let blob = git(root, &["rev-parse", &format!("{rev}:{p}")])?;
        deleted.push((p, String::from_utf8_lossy(&blob).trim().to_string()));
    }
    let blobs = hash_working_files(root, &modi)?;
    let modified: Vec<(String, String)> = modi.into_iter().zip(blobs).collect();
    for (old, new) in pair_replaced_stubs(&deleted, &modified) {
        map.insert(&old, &new);
    }
    Ok(map)
}

/// The blob id of each working-tree file in `paths`, in order
/// (`git hash-object --stdin-paths`).
fn hash_working_files(root: &Path, paths: &[String]) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let mut child = git_command(root)
        .args(["hash-object", "--stdin-paths"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("git hash-object: {e}"))?;
    let input = paths.join("\n") + "\n";
    let mut stdin = child.stdin.take().ok_or("git hash-object: no stdin")?;
    let writer = std::thread::spawn(move || {
        use std::io::Write;
        stdin.write_all(input.as_bytes())
    });
    let res = child
        .wait_with_output()
        .map_err(|e| format!("git hash-object: {e}"))?;
    writer
        .join()
        .map_err(|_| "git hash-object: writer panicked".to_string())?
        .map_err(|e| format!("git hash-object: {e}"))?;
    if !res.status.success() {
        return Err(format!(
            "git hash-object failed: {}",
            String::from_utf8_lossy(&res.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&res.stdout).into_owned();
    let blobs: Vec<String> = text.lines().map(str::to_string).collect();
    if blobs.len() != paths.len() {
        return Err("git hash-object: unexpected output length".into());
    }
    Ok(blobs)
}

/// The paths of `git ls-tree -r -z REV` output, and an index from blob id to
/// the lexicographically first path holding it. Non-blob entries are skipped.
pub fn parse_ls_tree_z(out: &[u8]) -> (HashSet<String>, HashMap<String, String>) {
    let (mut paths, mut by_blob) = (HashSet::new(), HashMap::<String, String>::new());
    for rec in out.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let rec = String::from_utf8_lossy(rec);
        let Some((meta, path)) = rec.split_once('\t') else {
            continue;
        };
        let mut f = meta.split(' ');
        let (Some(_mode), Some(kind), Some(blob)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
        if kind != "blob" {
            continue;
        }
        paths.insert(path.to_string());
        by_blob
            .entry(blob.to_string())
            .and_modify(|p| {
                if path < p.as_str() {
                    *p = path.to_string();
                }
            })
            .or_insert_with(|| path.to_string());
    }
    (paths, by_blob)
}

/// One `COPIED` link: the broken link, the original `O` it was copied from,
/// and the path the link resolves to from `O`.
pub type Copied = (Broken, String, String);

/// Split NEW broken links into real failures and `COPIED` ones: the source is
/// under `archive/`, did not exist at REV (`existed_at_rev`), its bytes equal
/// a file `O` at REV (`originals`), and the link resolved from `O` exists now
/// (`exists_now`). Returns `(new, copied)`.
pub fn split_copied(
    new: Vec<Broken>,
    originals: &impl Fn(&str) -> Option<String>,
    existed_at_rev: &impl Fn(&str) -> bool,
    exists_now: &impl Fn(&str) -> bool,
) -> (Vec<Broken>, Vec<Copied>) {
    let (mut fail, mut copied) = (Vec::new(), Vec::new());
    for b in new {
        let hit = if b.source.starts_with("archive/") && !existed_at_rev(&b.source) {
            originals(&b.source).and_then(|o| match resolve(&o, &b.target) {
                Resolved::Repo(p) if exists_now(&p) => Some((o, p)),
                _ => None,
            })
        } else {
            None
        };
        match hit {
            Some((o, p)) => copied.push((b, o, p)),
            None => fail.push(b),
        }
    }
    (fail, copied)
}

/// `split_copied` against the repository: one `ls-tree` of REV and one
/// `hash-object` batch for the candidate sources.
fn split_copied_in_repo(
    root: &Path,
    rev: &str,
    new: Vec<Broken>,
    now_tracked: &Universe,
) -> Result<(Vec<Broken>, Vec<Copied>), String> {
    let mut cands: Vec<String> = new
        .iter()
        .filter(|b| b.source.starts_with("archive/"))
        .map(|b| b.source.clone())
        .collect();
    if cands.is_empty() {
        return Ok((new, Vec::new()));
    }
    let (rev_paths, by_blob) = parse_ls_tree_z(&git(root, &["ls-tree", "-r", "-z", rev])?);
    cands.sort();
    cands.dedup();
    cands.retain(|s| !rev_paths.contains(s));
    let blobs = hash_working_files(root, &cands)?;
    let blob_of: HashMap<String, String> = cands.into_iter().zip(blobs).collect();
    Ok(split_copied(
        new,
        &|src| blob_of.get(src).and_then(|b| by_blob.get(b)).cloned(),
        &|src| rev_paths.contains(src),
        &|p| now_tracked.exists(p),
    ))
}

/// A record is "unchanged since REV" only if it exists at REV and now, and
/// `git diff --quiet` is clean. A nonexistent path makes `diff --quiet` exit 0,
/// so existence is checked separately.
pub fn source_unchanged(exists_at_rev: bool, exists_now: bool, diff_clean: bool) -> bool {
    exists_at_rev && exists_now && diff_clean
}

/// Split NEW broken links into real failures and `MOVED` ones: the source is
/// immutable and unchanged since REV (`unchanged`), the target is the old
/// side of an R100 rename (or inferred directory rename), and the renamed
/// path exists in `now_tracked`. Returns `(new, moved)`; each moved link
/// carries the target's new path.
pub fn split_moved(
    new: Vec<Broken>,
    renames: &PathMap,
    now_tracked: &Universe,
    unchanged: impl Fn(&str) -> bool,
) -> (Vec<Broken>, Vec<(Broken, String)>) {
    let (mut fail, mut moved) = (Vec::new(), Vec::new());
    for b in new {
        let now = renames.map(&b.resolved);
        if is_immutable_source(&b.source)
            && now != b.resolved
            && now_tracked.exists(&now)
            && unchanged(&b.source)
        {
            moved.push((b, now));
        } else {
            fail.push(b);
        }
    }
    (fail, moved)
}

/// Compare against the baseline through every map: renames, replaced stubs
/// and the simulation, in that order.
pub fn compare_with_maps(
    baseline: &[(String, String)],
    renames: &PathMap,
    stubs: &PathMap,
    sim: &PathMap,
    current: &[Broken],
) -> Comparison {
    compare(baseline, &[renames, stubs, sim], current)
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

/// True when git exits 0 for `args` (used with `diff --quiet`); any other
/// outcome, including a spawn failure, reads as "changed".
fn git_env_clean(root: &Path, args: &[&str]) -> bool {
    git_command(root)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
    let (renames, stubs) = match &renames_from {
        Some(rev) => {
            let out = git(
                root,
                &["diff", "--cached", "-M100%", "--name-status", "-z", rev],
            )?;
            let mut m = PathMap::parse_renames_z(&out);
            m.add_dir_renames(&Universe::new(tracked.iter().cloned()));
            (m, replaced_stub_map(root, rev, &out)?)
        }
        None => (PathMap::default(), PathMap::default()),
    };
    let cmp = compare_with_maps(&base, &renames, &stubs, &sim, &report.broken);
    let now_tracked = Universe::new(tracked.iter().cloned());
    // Under --simulate the tree is hypothetical: nothing may pass as MOVED.
    let (mut new, mut moved) = match (&renames_from, &simulate) {
        (Some(rev), None) => split_moved(cmp.new, &renames, &now_tracked, |src| {
            let spec = format!("{rev}:{src}");
            source_unchanged(
                git_env_clean(root, &["cat-file", "-e", &spec]),
                now_tracked.exists(src),
                git_env_clean(
                    root,
                    &["--literal-pathspecs", "diff", "--quiet", rev, "--", src],
                ),
            )
        }),
        _ => (cmp.new, Vec::new()),
    };
    // The same hold applies to COPIED: a simulated tree has no real copies.
    let mut copied = Vec::new();
    if let (Some(rev), None) = (&renames_from, &simulate) {
        (new, copied) = split_copied_in_repo(root, rev, new, &now_tracked)?;
    }
    println!("new broken: {}", new.len());
    println!("fixed: {}", cmp.fixed);
    println!("moved (immutable source): {}", moved.len());
    println!("copied (archived copy): {}", copied.len());
    new.sort();
    moved.sort_by(|a, b| a.0.cmp(&b.0));
    copied.sort_by(|a, b| a.0.cmp(&b.0));
    for (b, now) in &moved {
        println!(
            "  MOVED {}:{} {} -> {} (now {})",
            b.source, b.line, b.target, b.resolved, now
        );
    }
    for (b, original, path) in &copied {
        println!(
            "  COPIED {}:{} {} (original {}, resolves to {})",
            b.source, b.line, b.target, original, path
        );
    }
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

    fn broken(source: &str, resolved: &str) -> Broken {
        Broken {
            source: source.to_string(),
            line: 1,
            target: resolved.to_string(),
            resolved: resolved.to_string(),
        }
    }

    fn rename_map() -> PathMap {
        PathMap::parse_renames_z(b"R100\0docs/old/a.md\0archive/old/a.md\0")
    }

    fn now_universe() -> Universe {
        Universe::new(["archive/old/a.md".to_string(), "archive/rec.md".to_string()])
    }

    #[test]
    fn unchanged_immutable_source_with_renamed_target_is_moved() {
        let (fail, moved) = split_moved(
            vec![broken("archive/rec.md", "docs/old/a.md")],
            &rename_map(),
            &now_universe(),
            |_| true,
        );
        assert!(fail.is_empty());
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].1, "archive/old/a.md");
        let (fail, moved) = split_moved(
            vec![broken("docs/harness/prep/p.md", "docs/old/a.md")],
            &rename_map(),
            &now_universe(),
            |_| true,
        );
        assert!((fail.len(), moved.len()) == (0, 1));
    }

    #[test]
    fn changed_immutable_source_with_renamed_target_is_new() {
        let (fail, moved) = split_moved(
            vec![broken("archive/rec.md", "docs/old/a.md")],
            &rename_map(),
            &now_universe(),
            |_| false,
        );
        assert_eq!((fail.len(), moved.len()), (1, 0));
    }

    #[test]
    fn live_source_with_renamed_target_is_new() {
        let (fail, moved) = split_moved(
            vec![broken("docs/live.md", "docs/old/a.md")],
            &rename_map(),
            &now_universe(),
            |_| true,
        );
        assert_eq!((fail.len(), moved.len()), (1, 0));
        // An unmoved target stays NEW even from an unchanged record.
        let (fail, moved) = split_moved(
            vec![broken("archive/rec.md", "docs/never/b.md")],
            &rename_map(),
            &now_universe(),
            |_| true,
        );
        assert_eq!((fail.len(), moved.len()), (1, 0));
    }

    #[test]
    fn replaced_stub_pairs_only_byte_equal_blobs() {
        let del = vec![("docs/X.md".to_string(), "aaa".to_string())];
        let same = vec![("archive/X.md".to_string(), "aaa".to_string())];
        let diff = vec![("archive/X.md".to_string(), "bbb".to_string())];
        assert_eq!(
            pair_replaced_stubs(&del, &same),
            vec![("docs/X.md".to_string(), "archive/X.md".to_string())]
        );
        assert!(pair_replaced_stubs(&del, &diff).is_empty());
        let (d, m) = parse_deleted_modified_z(b"D\0docs/X.md\0M\0archive/X.md\0R100\0a\0b\0A\0n\0");
        assert_eq!(
            (d, m),
            (
                vec!["docs/X.md".to_string()],
                vec!["archive/X.md".to_string()]
            )
        );
    }

    #[test]
    fn directory_inferred_rename_with_deleted_sibling_stays_new() {
        // `a.md` was renamed, so `docs/old` -> `archive/old` is inferred; `b.md`
        // was deleted, and the inferred map must not turn its link into MOVED.
        let mut m = rename_map();
        m.add_dir_renames(&Universe::new(["archive/rec.md".to_string()]));
        assert_eq!(m.map("docs/old/b.md"), "archive/old/b.md");
        let (fail, moved) = split_moved(
            vec![
                broken("archive/rec.md", "docs/old/b.md"),
                broken("archive/rec.md", "docs/old/a.md"),
            ],
            &m,
            &now_universe(),
            |_| true,
        );
        assert_eq!(fail.len(), 1);
        assert_eq!(fail[0].resolved, "docs/old/b.md");
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].1, "archive/old/a.md");
    }

    #[test]
    fn source_unchanged_needs_existence_at_rev_and_now() {
        assert!(source_unchanged(true, true, true));
        // An added file: absent at REV, where `diff --quiet` still exits 0.
        assert!(!source_unchanged(false, true, true));
        // A nonexistent path.
        assert!(!source_unchanged(false, false, true));
        // Deleted since REV.
        assert!(!source_unchanged(true, false, true));
        assert!(!source_unchanged(true, true, false));
    }

    #[test]
    fn stub_pairing_is_one_to_one_and_unambiguous() {
        let d = |p: &str, b: &str| (p.to_string(), b.to_string());
        // One modified path never pairs with two deletions.
        let pairs = pair_replaced_stubs(
            &[d("docs/X.md", "aaa"), d("docs/Y.md", "aaa")],
            &[d("archive/X.md", "aaa")],
        );
        assert_eq!(
            pairs,
            vec![("docs/X.md".to_string(), "archive/X.md".to_string())]
        );
        // Two identical deleted files with the same name: ambiguous, unpaired.
        let pairs = pair_replaced_stubs(
            &[d("docs/a/X.md", "aaa"), d("docs/b/X.md", "aaa")],
            &[d("archive/X.md", "aaa")],
        );
        assert!(pairs.is_empty());
        // Several modified paths: the same-name one wins.
        let pairs = pair_replaced_stubs(
            &[d("docs/X.md", "aaa")],
            &[d("archive/X.md", "aaa"), d("archive/Z.md", "aaa")],
        );
        assert_eq!(
            pairs,
            vec![("docs/X.md".to_string(), "archive/X.md".to_string())]
        );
        // Several modified paths, none with the same name: unpaired.
        let pairs = pair_replaced_stubs(
            &[d("docs/X.md", "aaa")],
            &[d("archive/P.md", "aaa"), d("archive/Q.md", "aaa")],
        );
        assert!(pairs.is_empty());
    }

    /// `archive/live/deep/a.md` is a new byte-equal copy of `docs/live/a.md`;
    /// `archive/edited.md` matches nothing at REV; `archive/old.md` existed.
    fn copy_split(b: Broken) -> (Vec<Broken>, Vec<Copied>) {
        let now = Universe::new(["docs/live/b.md".to_string(), "docs/live/a.md".to_string()]);
        split_copied(
            vec![b],
            &|src| (src != "archive/edited.md").then(|| "docs/live/a.md".to_string()),
            &|src| src == "archive/old.md",
            &|p| now.exists(p),
        )
    }

    fn copy_link(source: &str, target: &str) -> Broken {
        let Resolved::Repo(resolved) = resolve(source, target) else {
            panic!("{target} does not resolve inside the repository");
        };
        Broken {
            source: source.into(),
            line: 3,
            target: target.into(),
            resolved,
        }
    }

    #[test]
    fn faithful_archive_copy_link_resolving_from_original_is_copied() {
        let (fail, copied) = copy_split(copy_link("archive/live/deep/a.md", "b.md"));
        assert!(fail.is_empty());
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0].1, "docs/live/a.md");
        assert_eq!(copied[0].2, "docs/live/b.md");
    }

    #[test]
    fn archive_copy_link_broken_from_original_too_fails() {
        let (fail, copied) = copy_split(copy_link("archive/live/deep/a.md", "gone.md"));
        assert_eq!((fail.len(), copied.len()), (1, 0));
    }

    #[test]
    fn archive_file_matching_no_rev_blob_fails() {
        let (fail, copied) = copy_split(copy_link("archive/edited.md", "b.md"));
        assert_eq!((fail.len(), copied.len()), (1, 0));
    }

    #[test]
    fn archive_source_that_existed_at_rev_is_not_copied() {
        let (fail, copied) = copy_split(copy_link("archive/old.md", "b.md"));
        assert_eq!((fail.len(), copied.len()), (1, 0));
    }

    #[test]
    fn source_outside_archive_is_never_copied() {
        let (fail, copied) = copy_split(copy_link("docs/other/deep/a.md", "b.md"));
        assert_eq!((fail.len(), copied.len()), (1, 0));
    }

    #[test]
    fn ls_tree_index_keeps_blobs_only_and_first_path() {
        let out = b"100644 blob aa\tz/x.md\x00100644 blob aa\tb/x.md\x00160000 commit cc\tsub\x00";
        let (paths, by_blob) = parse_ls_tree_z(out);
        assert_eq!(paths.len(), 2);
        assert!(!paths.contains("sub"));
        assert_eq!(by_blob["aa"], "b/x.md");
    }

    #[test]
    fn compare_uses_the_stub_map() {
        let cur = Broken {
            source: "archive/X.md".into(),
            line: 1,
            target: "t.md".into(),
            resolved: "t.md".into(),
        };
        // The baseline knew the link under the stub's old path.
        let base = vec![("docs/X.md".to_string(), "t.md".to_string())];
        let mut stubs = PathMap::default();
        stubs.insert("docs/X.md", "archive/X.md");
        let none = PathMap::default();
        let cmp = compare_with_maps(&base, &none, &stubs, &none, std::slice::from_ref(&cur));
        assert_eq!((cmp.new.len(), cmp.fixed), (0, 0));
        let cmp = compare_with_maps(&base, &none, &none, &none, std::slice::from_ref(&cur));
        assert_eq!(cmp.new.len(), 1);
    }
}
