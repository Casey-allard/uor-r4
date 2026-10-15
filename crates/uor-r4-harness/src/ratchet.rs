//! The D23 (normalised exponential) and D25 (dense product) ratchets, in Rust (D30: Rust only).
//!
//! A ratchet cannot prove the tree is clean. It stops a count from growing, so removal can proceed
//! without new violations landing underneath it. Each one keeps a per-file baseline
//! (the two `scripts/no-*-baseline.json` files): a file above its baseline,
//! or a file absent from it with any hit, fails. `--update-baseline` may only lower counts, so it
//! cannot bless a new site.
//!
//! This is a port of the two Python checkers, `scripts/check_no_matmul.py` and its D23 twin, with the same
//! scan, the same baseline files and the same verdicts, so either implementation reads the other's
//! baseline. Dated records under `docs/` are not scanned: they are evidence, not plans.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories scanned, relative to the repository root.
const SCAN_DIRS: [&str; 1] = ["crates"];
/// File suffixes scanned (the last extension, case-sensitive).
const SUFFIXES: [&str; 4] = ["rs", "cu", "cuh", "wgsl"];

/// The word D23 counts, assembled so this file does not count against its own ratchet.
pub const D23_WORD: &str = concat!("soft", "max");
/// Tensor-core fragment intrinsics, reported by the D25 ratchet and never gated.
const TC_FRAGMENT: &str = concat!("w", "mma");
const TC_SYNC: &str = concat!("mma", "_sync");

/// Which ratchet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// D23: every case-insensitive occurrence of [`D23_WORD`] in source text, comments included.
    NormExp,
    /// D25: dense-product call sites in code (comments and plain strings removed, raw strings kept).
    Matmul,
}

impl Kind {
    pub fn parse(word: &str) -> Option<Kind> {
        match word {
            w if w == D23_WORD || w == "d23" => Some(Kind::NormExp),
            "matmul" | "d25" => Some(Kind::Matmul),
            _ => None,
        }
    }

    pub fn baseline_path(self, repo: &Path) -> PathBuf {
        repo.join("scripts").join(match self {
            Kind::NormExp => concat!("no-", "soft", "max", "-baseline.json"),
            Kind::Matmul => "no-matmul-baseline.json",
        })
    }
}

/// One counted site (no-matmul only keeps detail).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    pub line: usize,
    pub kind: &'static str,
    pub text: String,
}

/// The result of one scan.
#[derive(Debug, Default)]
pub struct Scan {
    /// Repo-relative path -> number of gated sites (files with zero are absent).
    pub counts: BTreeMap<String, u64>,
    /// Matmul only: every gated site, for `--list`.
    pub detail: BTreeMap<String, Vec<Site>>,
    /// Matmul only, informational: tensor-core intrinsic sites and the files holding them.
    pub tensor_core: u64,
    pub tensor_files: u64,
}

impl Scan {
    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }
}

#[derive(Debug)]
pub enum RatchetError {
    Io(PathBuf, std::io::Error),
    Baseline(PathBuf, String),
    /// R10: a baseline is a tracked file, so it is written in a lab's own checkout and never in the
    /// shared one. Carries the refusal text, which names the rule and the fix.
    Refused(String),
}

impl fmt::Display for RatchetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RatchetError::Io(p, e) => write!(f, "{}: {e}", p.display()),
            RatchetError::Baseline(p, e) => write!(f, "{}: unreadable baseline: {e}", p.display()),
            RatchetError::Refused(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for RatchetError {}

/// Every scanned source file under the scan directories, as (repo-relative path, absolute path).
/// Directory symlinks are not followed; any path component named `target` is skipped.
fn source_files(repo: &Path) -> Result<Vec<(String, PathBuf)>, RatchetError> {
    let mut out = Vec::new();
    for dir in SCAN_DIRS {
        let base = repo.join(dir);
        if !base.is_dir() {
            continue;
        }
        let mut stack = vec![base];
        while let Some(d) = stack.pop() {
            let entries = fs::read_dir(&d).map_err(|e| RatchetError::Io(d.clone(), e))?;
            for entry in entries {
                let entry = entry.map_err(|e| RatchetError::Io(d.clone(), e))?;
                let path = entry.path();
                let Ok(meta) = fs::symlink_metadata(&path) else {
                    continue;
                };
                if meta.is_dir() {
                    stack.push(path);
                    continue;
                }
                if !path.is_file() {
                    continue;
                }
                let suffix_ok = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| SUFFIXES.contains(&e));
                if !suffix_ok {
                    continue;
                }
                let Ok(rel) = path.strip_prefix(repo) else {
                    continue;
                };
                if rel.components().any(|c| c.as_os_str() == "target") {
                    continue;
                }
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((rel, path));
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Non-overlapping, ASCII case-insensitive occurrences of `needle` (lowercase) in `hay`.
fn count_ci(hay: &str, needle: &str) -> u64 {
    let lower = hay.to_ascii_lowercase();
    lower.matches(needle).count() as u64
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `text` with comments and plain string literals replaced by spaces, line breaks kept, so a site's
/// line number in the result is its own. Raw strings (`r"..."`, `r#"..."#`) are kept: the CUDA
/// kernel sources lived in one, and that text is executable. Block comments nest, as in Rust.
pub fn strip_comments_and_strings(text: &str) -> String {
    let t: Vec<char> = text.chars().collect();
    let n = t.len();
    let mut out = t.clone();
    let starts = |at: usize, s: &str| {
        s.chars()
            .enumerate()
            .all(|(k, c)| t.get(at + k) == Some(&c))
    };
    let mut i = 0;
    while i < n {
        let c = t[i];
        if c == '/' && i + 1 < n && t[i + 1] == '/' {
            let mut j = i;
            while j < n && t[j] != '\n' {
                out[j] = ' ';
                j += 1;
            }
            i = j;
        } else if c == '/' && i + 1 < n && t[i + 1] == '*' {
            let mut depth = 0usize;
            let mut j = i;
            while j < n {
                if starts(j, "/*") {
                    depth += 1;
                    j += 2;
                } else if starts(j, "*/") {
                    depth = depth.saturating_sub(1);
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            let end = j.min(n);
            for slot in out.iter_mut().take(end).skip(i) {
                if *slot != '\n' {
                    *slot = ' ';
                }
            }
            i = end;
        } else if c == 'r' && raw_string_end(&t, i).is_some() {
            i = raw_string_end(&t, i).unwrap_or(n);
        } else if c == '"' {
            let mut j = i + 1;
            while j < n {
                if t[j] == '\\' {
                    j += 2;
                    continue;
                }
                if t[j] == '"' {
                    break;
                }
                j += 1;
            }
            let end = (j + 1).min(n);
            for slot in out.iter_mut().take(end).skip(i) {
                if *slot != '\n' {
                    *slot = ' ';
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out.into_iter().collect()
}

/// The offset just past a raw string starting at `i`, or `None` if none starts there.
fn raw_string_end(t: &[char], i: usize) -> Option<usize> {
    if t.get(i) != Some(&'r') {
        return None;
    }
    let mut j = i + 1;
    let mut hashes = 0;
    while j < t.len() && t[j] == '#' {
        hashes += 1;
        j += 1;
    }
    if j >= t.len() || t[j] != '"' {
        return None;
    }
    let closing: Vec<char> = std::iter::once('"')
        .chain(std::iter::repeat_n('#', hashes))
        .collect();
    let mut k = j + 1;
    while k + closing.len() <= t.len() {
        if t[k..k + closing.len()] == closing[..] {
            return Some(k + closing.len());
        }
        k += 1;
    }
    Some(t.len())
}

/// The D25 gated sites on one line of stripped code, left to right per pattern (pattern order is
/// the Python script's: `.matmul(`, `MatMul`, `cublas`, word `gemm`).
fn matmul_sites(line: &str, number: usize, into: &mut Vec<Site>) {
    for _ in line.match_indices(".matmul(") {
        into.push(Site {
            line: number,
            kind: "matmul-method",
            text: ".matmul(".into(),
        });
    }
    for _ in line.match_indices("MatMul") {
        into.push(Site {
            line: number,
            kind: "candle-op",
            text: "MatMul".into(),
        });
    }
    let lower = line.to_ascii_lowercase();
    for (at, _) in lower.match_indices("cublas") {
        into.push(Site {
            line: number,
            kind: "cublas",
            text: line[at..at + 6].into(),
        });
    }
    for (at, _) in lower.match_indices("gemm") {
        let before = line[..at].chars().next_back();
        let after = line[at + 4..].chars().next();
        if before.is_none_or(|c| !is_word(c)) && after.is_none_or(|c| !is_word(c)) {
            into.push(Site {
                line: number,
                kind: "gemm",
                text: line[at..at + 4].into(),
            });
        }
    }
}

/// Tensor-core fragment intrinsics ([`TC_FRAGMENT`] or [`TC_SYNC`], any case), non-overlapping, on raw text.
fn tensor_core_sites(text: &str) -> u64 {
    let lower = text.to_ascii_lowercase();
    let b = lower.as_bytes();
    let (mut i, mut n) = (0, 0);
    while i < b.len() {
        if b[i..].starts_with(TC_FRAGMENT.as_bytes()) {
            n += 1;
            i += 4;
        } else if b[i..].starts_with(TC_SYNC.as_bytes()) {
            n += 1;
            i += 8;
        } else {
            i += 1;
        }
    }
    n
}

/// Scan the repository for one ratchet.
pub fn scan(repo: &Path, kind: Kind) -> Result<Scan, RatchetError> {
    let mut result = Scan::default();
    for (rel, path) in source_files(repo)? {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        match kind {
            Kind::NormExp => {
                let n = count_ci(&text, D23_WORD);
                if n > 0 {
                    result.counts.insert(rel, n);
                }
            }
            Kind::Matmul => {
                let code = strip_comments_and_strings(&text);
                let mut sites = Vec::new();
                for (k, line) in code.split('\n').enumerate() {
                    matmul_sites(line, k + 1, &mut sites);
                }
                if !sites.is_empty() {
                    result.counts.insert(rel.clone(), sites.len() as u64);
                    result.detail.insert(rel, sites);
                }
                let tc = tensor_core_sites(&text);
                if tc > 0 {
                    result.tensor_core += tc;
                    result.tensor_files += 1;
                }
            }
        }
    }
    Ok(result)
}

/// The baseline, or `None` when its file is absent.
pub fn load_baseline(
    repo: &Path,
    kind: Kind,
) -> Result<Option<BTreeMap<String, u64>>, RatchetError> {
    let path = kind.baseline_path(repo);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| RatchetError::Io(path.clone(), e))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| RatchetError::Baseline(path, e.to_string()))
}

/// The baseline file's text for `counts`: two-space indent, sorted keys, trailing newline (the
/// format the Python scripts write, so the two implementations produce identical files).
pub fn render_baseline(counts: &BTreeMap<String, u64>) -> String {
    if counts.is_empty() {
        return "{}\n".to_string();
    }
    let mut s = String::from("{\n");
    let last = counts.len() - 1;
    for (k, (path, n)) in counts.iter().enumerate() {
        let key = serde_json::to_string(path).unwrap_or_else(|_| format!("{path:?}"));
        s.push_str(&format!(
            "  {key}: {n}{}\n",
            if k == last { "" } else { "," }
        ));
    }
    s.push_str("}\n");
    s
}

/// Files whose count rose above the baseline: (path, baseline, current).
pub fn growth(
    current: &BTreeMap<String, u64>,
    baseline: &BTreeMap<String, u64>,
) -> Vec<(String, u64, u64)> {
    current
        .iter()
        .filter_map(|(p, &n)| {
            let base = baseline.get(p).copied().unwrap_or(0);
            (n > base).then(|| (p.clone(), base, n))
        })
        .collect()
}

/// Sites removed since the baseline.
pub fn removed(current: &BTreeMap<String, u64>, baseline: &BTreeMap<String, u64>) -> u64 {
    baseline
        .iter()
        .map(|(p, &b)| b.saturating_sub(current.get(p).copied().unwrap_or(0)))
        .sum()
}

/// What `--update-baseline` decided.
#[derive(Debug, PartialEq, Eq)]
pub enum Update {
    /// Written; `first` when no baseline existed before.
    Written {
        first: bool,
        files: usize,
        total: u64,
    },
    /// Refused: these files would rise.
    Refused(Vec<(String, u64, u64)>),
}

/// Write the current counts as the baseline, refusing any rise. An absent baseline is set at the
/// current counts. For D23 an empty baseline also counts as absent (the Python D23 rule); for
/// matmul an empty baseline is the ratchet at zero.
pub fn update_baseline(
    repo: &Path,
    kind: Kind,
    current: &BTreeMap<String, u64>,
) -> Result<Update, RatchetError> {
    // R10 first: `scripts/no-*-baseline.json` is a TRACKED file, so this is a checkout write like any
    // other. It was the one that falsified "every remaining committed write refuses the shared
    // checkout" (review finding D2: it wrote from a shared primary, exit 0).
    crate::state::refuse_shared_checkout_at(repo, "the ratchet baseline")
        .map_err(RatchetError::Refused)?;
    let old = load_baseline(repo, kind)?;
    let first = match (&old, kind) {
        (None, _) => true,
        (Some(m), Kind::NormExp) => m.is_empty(),
        (Some(_), Kind::Matmul) => false,
    };
    if !first {
        let raised = growth(current, old.as_ref().unwrap_or(&BTreeMap::new()));
        if !raised.is_empty() {
            return Ok(Update::Refused(raised));
        }
    }
    let path = kind.baseline_path(repo);
    fs::write(&path, render_baseline(current)).map_err(|e| RatchetError::Io(path, e))?;
    Ok(Update::Written {
        first,
        files: current.len(),
        total: current.values().sum(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripper_blanks_comments_and_plain_strings_but_keeps_raw_strings_and_lines() {
        let src = "a.matmul(b) // gemm\n/* MatMul /* nested */ cublas */ x\n\"gemm\\\" cublas\" r#\"gemm(\"#\n";
        let code = strip_comments_and_strings(src);
        assert_eq!(code.chars().count(), src.chars().count());
        assert_eq!(code.matches('\n').count(), src.matches('\n').count());
        let mut sites = Vec::new();
        for (k, line) in code.split('\n').enumerate() {
            matmul_sites(line, k + 1, &mut sites);
        }
        let kinds: Vec<_> = sites.iter().map(|s| (s.line, s.kind)).collect();
        assert_eq!(kinds, vec![(1, "matmul-method"), (3, "gemm")]);
    }

    #[test]
    fn gemm_is_a_whole_word_any_case() {
        let mut sites = Vec::new();
        matmul_sites("GEMM(x) sgemm gemm_x my_gemm (Gemm) cuBLAS", 1, &mut sites);
        let kinds: Vec<_> = sites.iter().map(|s| s.kind).collect();
        assert_eq!(kinds, vec!["cublas", "gemm", "gemm"]);
    }

    #[test]
    fn tensor_core_count_is_non_overlapping() {
        let s = format!(
            "nvcuda::{}::{}(a) {}_sync",
            TC_FRAGMENT.to_uppercase(),
            TC_SYNC,
            TC_FRAGMENT
        );
        assert_eq!(tensor_core_sites(&s), 3);
    }

    #[test]
    fn d23_word_counts_every_case() {
        let w = D23_WORD;
        let s = format!("{} {w}_rows LOG_{}", w.to_uppercase(), w.to_uppercase());
        assert_eq!(count_ci(&s, w), 3);
    }

    #[test]
    fn baseline_render_matches_python_json_dumps_indent2_sorted() {
        let mut m = BTreeMap::new();
        m.insert("crates/b.rs".to_string(), 2);
        m.insert("crates/a.rs".to_string(), 10);
        assert_eq!(
            render_baseline(&m),
            "{\n  \"crates/a.rs\": 10,\n  \"crates/b.rs\": 2\n}\n"
        );
        assert_eq!(render_baseline(&BTreeMap::new()), "{}\n");
    }

    fn repo_with(files: &[(&str, String)], baseline: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for (rel, text) in files {
            let p = dir.path().join(rel);
            fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            fs::write(p, text).expect("write");
        }
        fs::create_dir_all(dir.path().join("scripts")).expect("mkdir scripts");
        if let Some(b) = baseline {
            fs::write(Kind::NormExp.baseline_path(dir.path()), b).expect("baseline");
        }
        dir
    }

    #[test]
    fn update_lowers_but_refuses_a_rise_and_skips_target_and_docs() {
        let w = format!("{D23_WORD} ");
        let dir = repo_with(
            &[
                ("crates/x/src/a.rs", w.repeat(2)),
                ("crates/x/target/debug/b.rs", w.clone()),
                ("crates/x/src/note.md", w.clone()),
                ("docs/old.rs", w.clone()),
            ],
            Some("{\n  \"crates/x/src/a.rs\": 3\n}\n"),
        );
        let s = scan(dir.path(), Kind::NormExp).expect("scan");
        assert_eq!(s.counts.len(), 1);
        assert_eq!(s.total(), 2);
        let lowered = update_baseline(dir.path(), Kind::NormExp, &s.counts).expect("update");
        assert_eq!(
            lowered,
            Update::Written {
                first: false,
                files: 1,
                total: 2
            }
        );
        fs::write(dir.path().join("crates/x/src/a.rs"), w.repeat(3)).expect("write");
        let s = scan(dir.path(), Kind::NormExp).expect("scan");
        match update_baseline(dir.path(), Kind::NormExp, &s.counts).expect("update") {
            Update::Refused(r) => assert_eq!(r, vec![("crates/x/src/a.rs".to_string(), 2, 3)]),
            other => panic!("a rise must be refused, got {other:?}"),
        }
    }
}
