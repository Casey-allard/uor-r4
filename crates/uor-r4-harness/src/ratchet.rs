//! The D23 (normalised exponential, squashing, activation and rotary), D25 (dense product and
//! tensor-core), and D32 (discrete floating-point type and literal) ratchets, in Rust (D30: Rust only).
//!
//! A ratchet cannot prove the tree is clean. It stops a count from growing, so removal can proceed
//! without new violations landing underneath it. Each one keeps a per-file baseline
//! (the `scripts/no-*-baseline.json` files): a file above its baseline,
//! or a file absent from it with any hit, fails. `--update-baseline` may only lower counts, so it
//! cannot bless a new site.
//!
//! Scanned directories cover `crates/` and `docs/labs/*/instrument/` (active instrument crates under
//! dated lab records). Dated prose/evidence outside `instrument/` is not scanned.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Directories scanned, relative to the repository root (`*` matches one directory component).
const SCAN_DIRS: [&str; 2] = ["crates", "docs/labs/*/instrument"];
/// D37 item 6 scanned allowlist entry for the offline float training module.
///
/// Files under this crate are scanned by the D23 (`NormExp`) and D25 (`Matmul`) ratchets with a
/// hard zero-site ceiling (and refused by `--update-baseline` even on an empty baseline), while
/// explicitly allowlisted from D32 (`DiscreteFloat`).
pub const D37_SCANNED_FLOAT_CRATES: [&str; 1] = ["crates/uor-r4-float-train/"];
/// File suffixes scanned (the last extension, case-sensitive).
const SUFFIXES: [&str; 4] = ["rs", "cu", "cuh", "wgsl"];

/// The primary word D23 counts across raw source text, assembled so this file does not count against its own ratchet.
pub const D23_WORD: &str = concat!("soft", "max");
/// Prohibited D23/D25 squashing, activation and rotary identifier families counted on stripped code
/// (comments and plain string literals removed, raw strings kept), matched on ASCII-alphanumeric
/// identifier segments so English words like `proper` or `property` do not false-positive.
pub const D23_IDENT_FAMILIES: [&str; 8] = [
    concat!("soft", "plus"),
    concat!("sig", "moid"),
    concat!("ta", "nh"),
    concat!("ge", "lu"),
    concat!("si", "lu"),
    concat!("ro", "pe"),
    concat!("ro", "pei"),
    concat!("rot", "ary"),
];
/// Tensor-core fragment, sync, Hopper warpgroup, and Blackwell 5th-gen intrinsics, gated by the D25 ratchet on stripped code.
const TC_FRAGMENT: &str = concat!("w", "mma");
const TC_SYNC: &str = concat!("mma", "_sync");
const TC_WARPGROUP: &str = concat!("wg", "mma");
const TC_FIFTH_GEN: &str = concat!("tcgen", "05");

/// Prohibited floating-point type tokens gated by the D32 ratchet across discrete-model and
/// standalone-instrument sources (`crates/uor-r4-discrete/` and `docs/labs/*/instrument/`).
pub const D32_FLOAT_TYPES: [&str; 9] = [
    "f16",
    "bf16",
    "f32",
    "f64",
    "tf32",
    "float",
    "double",
    "__half",
    "__nv_bfloat16",
];

/// Which ratchet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// D23: every case-insensitive occurrence of [`D23_WORD`] in source text (comments included), plus
    /// every stripped-code identifier segment matching [`D23_IDENT_FAMILIES`].
    NormExp,
    /// D25: dense-product and tensor-core call sites in code (comments and plain strings removed, raw strings kept).
    Matmul,
    /// D32: floating-point type identifiers and floating-point literals in discrete-model and
    /// standalone-instrument code (`crates/uor-r4-discrete/` and `docs/labs/*/instrument/`).
    DiscreteFloat,
}

impl Kind {
    pub fn parse(word: &str) -> Option<Kind> {
        match word {
            w if w == D23_WORD || w == "d23" => Some(Kind::NormExp),
            "matmul" | "d25" => Some(Kind::Matmul),
            "discrete-float" | "d32" => Some(Kind::DiscreteFloat),
            _ => None,
        }
    }

    pub fn baseline_path(self, repo: &Path) -> PathBuf {
        repo.join("scripts").join(match self {
            Kind::NormExp => concat!("no-", "soft", "max", "-baseline.json"),
            Kind::Matmul => "no-matmul-baseline.json",
            Kind::DiscreteFloat => "no-discrete-float-baseline.json",
        })
    }
}

/// One counted site (`Matmul` and `DiscreteFloat` keep detail for `--list`).
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
    /// Matmul only: gated tensor-core intrinsic sites and the files holding them.
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

/// Expand `SCAN_DIRS` entries (supporting one `/*/` wildcard segment) into concrete base directories.
fn scan_roots(repo: &Path) -> Result<Vec<PathBuf>, RatchetError> {
    let mut roots = Vec::new();
    for pattern in SCAN_DIRS {
        if let Some((prefix, suffix)) = pattern.split_once("/*/") {
            let parent = repo.join(prefix);
            let Ok(meta) = fs::symlink_metadata(&parent) else {
                continue;
            };
            if !meta.is_dir() {
                continue;
            }
            let entries = fs::read_dir(&parent).map_err(|e| RatchetError::Io(parent.clone(), e))?;
            for entry in entries {
                let entry = entry.map_err(|e| RatchetError::Io(parent.clone(), e))?;
                let child = entry.path();
                let Ok(cmeta) = fs::symlink_metadata(&child) else {
                    continue;
                };
                if !cmeta.is_dir() {
                    continue;
                }
                let candidate = child.join(suffix);
                let Ok(smeta) = fs::symlink_metadata(&candidate) else {
                    continue;
                };
                if smeta.is_dir() {
                    roots.push(candidate);
                }
            }
        } else {
            let base = repo.join(pattern);
            if base.is_dir() {
                roots.push(base);
            }
        }
    }
    for entry in D37_SCANNED_FLOAT_CRATES {
        let base = repo.join(entry.trim_end_matches('/'));
        if base.is_dir() && !roots.contains(&base) {
            roots.push(base);
        }
    }
    Ok(roots)
}

/// Every scanned source file under the scan directories, as (repo-relative path, absolute path).
/// Directory symlinks are not followed; any path component named `target` is skipped.
fn source_files(repo: &Path) -> Result<Vec<(String, PathBuf)>, RatchetError> {
    let mut out = Vec::new();
    for base in scan_roots(repo)? {
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
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by(|a, b| a.0 == b.0);
    Ok(out)
}

/// Non-overlapping, ASCII case-insensitive occurrences of `needle` (lowercase) in `hay`.
#[cfg(test)]
fn count_ci(hay: &str, needle: &str) -> u64 {
    let lower = hay.to_ascii_lowercase();
    lower.matches(needle).count() as u64
}

/// Whether an ASCII-alphanumeric code segment (case-preserved) matches one of
/// [`D23_IDENT_FAMILIES`].
///
/// - For `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, and `rotary`, any case-insensitive
///   substring match inside the alphanumeric segment counts (catching standalone identifiers,
///   C float intrinsics like `tanhf`, and CamelCase/PascalCase compounds like `SigmoidGate`,
///   `RotaryEmbedding`, `SoftplusLoss`, `GeluActivation`, `SiluGate`, `SiLU`, `GeLU`).
/// - For `rope` and `ropei`, a segment counts when its lowercased form starts with `rope`
///   (catching `rope`, `ropei`, `RopeMode`, `RopeTheta`, `RoPE`, `RoPEConfig`) or its
///   case-preserved form contains `Rope`, `RoPE`, or `ROPE` (catching prefixed PascalCase/camelCase
///   compounds such as `PreRopeProjectionObservations`, `PreRopeProjectionProbe`, `applyRope`),
///   while rejecting English `proper`/`Proper`/`property`/`Properties`/`WriterProperties`.
fn is_d23_ident_segment(seg: &str) -> bool {
    if seg.len() < 4 {
        return false;
    }
    let mut stack_buf = [0u8; 128];
    let owned: String;
    let lower: &str = if seg.len() <= stack_buf.len() {
        for (dst, &src) in stack_buf[..seg.len()].iter_mut().zip(seg.as_bytes()) {
            *dst = src.to_ascii_lowercase();
        }
        std::str::from_utf8(&stack_buf[..seg.len()]).unwrap_or("")
    } else {
        owned = seg.to_ascii_lowercase();
        &owned
    };
    if lower.starts_with(concat!("ro", "pe"))
        || seg.contains(concat!("Ro", "pe"))
        || seg.contains(concat!("Ro", "PE"))
        || seg.contains(concat!("RO", "PE"))
    {
        return true;
    }
    D23_IDENT_FAMILIES
        .iter()
        .filter(|&&fam| fam != concat!("ro", "pe") && fam != concat!("ro", "pei"))
        .any(|&fam| lower.contains(fam))
}

/// Count D23 sites in `text`: every case-insensitive occurrence of [`D23_WORD`] in raw text, plus
/// every ASCII-alphanumeric identifier segment matching [`D23_IDENT_FAMILIES`] in stripped code
/// (comments and plain strings removed, raw strings kept).
fn d23_sites(text: &str) -> u64 {
    let lower = text.to_ascii_lowercase();
    let base = lower.matches(D23_WORD).count() as u64;
    let has_ident_family = D23_IDENT_FAMILIES
        .iter()
        .filter(|&&fam| fam != concat!("ro", "pei"))
        .any(|&fam| lower.contains(fam));
    if !has_ident_family {
        return base;
    }
    let code = strip_comments_and_strings(text);
    let extra: u64 = code
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|seg| !seg.is_empty() && is_d23_ident_segment(seg))
        .count() as u64;
    base + extra
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn push_blanked(out: &mut String, slice: &str) {
    for c in slice.chars() {
        out.push(if c == '\n' { '\n' } else { ' ' });
    }
}

/// `text` with comments and plain string literals replaced by spaces, line breaks kept, so a site's
/// line number in the result is its own. Raw strings (`r"..."`, `r#"..."#`) are kept: the CUDA
/// kernel sources lived in one, and that text is executable. Block comments nest, as in Rust.
pub fn strip_comments_and_strings(text: &str) -> String {
    strip_comments_and_strings_mode(text, false)
}

/// Like [`strip_comments_and_strings`], and also blanks single-quoted character literals (`'x'`,
/// `'"'`, `'\"'`, `'\n'`) while preserving Rust lifetimes (`'a`) and C++14 numeric separators
/// (`1'000`) so an unescaped `'"'` character literal cannot open a string literal on the D32 path.
pub fn strip_comments_strings_and_char_literals(text: &str) -> String {
    strip_comments_and_strings_mode(text, true)
}

fn strip_comments_and_strings_mode(text: &str, strip_char_literals: bool) -> String {
    let b = text.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n);
    let mut copied_until = 0usize;
    let mut i = 0usize;
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            out.push_str(&text[copied_until..i]);
            let mut j = i + 2;
            while j < n && b[j] != b'\n' {
                j += 1;
            }
            push_blanked(&mut out, &text[i..j]);
            i = j;
            copied_until = i;
        } else if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            out.push_str(&text[copied_until..i]);
            let mut depth = 0usize;
            let mut j = i;
            while j < n {
                if j + 1 < n && b[j] == b'/' && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if j + 1 < n && b[j] == b'*' && b[j + 1] == b'/' {
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
            push_blanked(&mut out, &text[i..end]);
            i = end;
            copied_until = i;
        } else if c == b'r' {
            if let Some(end) = raw_string_end_bytes(b, i) {
                i = end;
            } else {
                i += 1;
            }
        } else if strip_char_literals && c == b'\'' {
            if let Some(end) = char_literal_end_bytes(b, i) {
                out.push_str(&text[copied_until..i]);
                push_blanked(&mut out, &text[i..end]);
                i = end;
                copied_until = i;
            } else {
                i += 1;
            }
        } else if c == b'"' {
            out.push_str(&text[copied_until..i]);
            let mut j = i + 1;
            while j < n {
                if b[j] == b'\\' {
                    j += 1;
                    if j < n {
                        // Advance past the escaped UTF-8 character.
                        j += text[j..].chars().next().map_or(1, char::len_utf8);
                    }
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let end = j.min(n);
            push_blanked(&mut out, &text[i..end]);
            i = end;
            copied_until = i;
        } else {
            i += 1;
        }
    }
    out.push_str(&text[copied_until..n]);
    out
}

/// The byte offset just past a character literal (`'x'`, `'"'`, `'\"'`, `'\n'`, `'\x7f'`,
/// `'\u{1234}'`) starting at `i`, or `None` when `i` starts a Rust lifetime (`'a`) or a C++14
/// numeric separator (`1'000`).
fn char_literal_end_bytes(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'\'') {
        return None;
    }
    if i > 0 {
        let prev = b[i - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            let is_prefix = matches!(prev, b'b' | b'u' | b'U' | b'L')
                && (i < 2 || !(b[i - 2].is_ascii_alphanumeric() || b[i - 2] == b'_'));
            if !is_prefix {
                return None;
            }
        }
    }
    let next = *b.get(i + 1)?;
    if next == b'\'' || next == b'\n' || next == b'\r' {
        return None;
    }
    if next == b'\\' {
        let mut j = i + 2;
        while j < b.len() && b[j] != b'\n' && b[j] != b'\r' && j - i <= 14 {
            if b[j] == b'\\' {
                j += 2;
                continue;
            }
            if b[j] == b'\'' {
                return Some(j + 1);
            }
            j += 1;
        }
        return None;
    }
    let tail = std::str::from_utf8(&b[i + 1..]).ok()?;
    let ch = tail.chars().next()?;
    let close = i + 1 + ch.len_utf8();
    if b.get(close) == Some(&b'\'') {
        Some(close + 1)
    } else {
        None
    }
}

/// The byte offset just past a raw string starting at `i`, or `None` if none starts there.
fn raw_string_end_bytes(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'r') {
        return None;
    }
    let mut j = i + 1;
    let mut hashes = 0usize;
    while j < b.len() && b[j] == b'#' {
        hashes += 1;
        j += 1;
    }
    if j >= b.len() || b[j] != b'"' {
        return None;
    }
    let mut k = j + 1;
    while k + 1 + hashes <= b.len() {
        if b[k] == b'"' && b[k + 1..k + 1 + hashes].iter().all(|&x| x == b'#') {
            return Some(k + 1 + hashes);
        }
        k += 1;
    }
    Some(b.len())
}

/// The D25 gated sites on one line of stripped code, left to right per pattern (`.matmul(`,
/// `MatMul`, `cublas`, word `gemm`, and tensor-core intrinsics `wmma` / `mma_sync` / `wgmma` / `tcgen05`).
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
    if lower.contains(TC_FRAGMENT)
        || lower.contains(TC_SYNC)
        || lower.contains(TC_WARPGROUP)
        || lower.contains(TC_FIFTH_GEN)
    {
        let b = lower.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if let Some(pat) = [TC_FRAGMENT, TC_SYNC, TC_WARPGROUP, TC_FIFTH_GEN]
                .into_iter()
                .find(|pat| b[i..].starts_with(pat.as_bytes()))
            {
                into.push(Site {
                    line: number,
                    kind: "tensor-core",
                    text: line[i..i + pat.len()].into(),
                });
                i += pat.len();
            } else {
                i += 1;
            }
        }
    }
}

/// Whether `rel` belongs to the D37 scanned offline float training module.
pub fn is_d37_float_scope(rel: &str) -> bool {
    D37_SCANNED_FLOAT_CRATES
        .iter()
        .any(|prefix| rel.starts_with(prefix))
}

/// Whether `rel` belongs to the D32 discrete-model and standalone-instrument scope.
fn is_discrete_scope(rel: &str) -> bool {
    !is_d37_float_scope(rel)
        && (rel.starts_with("crates/uor-r4-discrete/") || rel.starts_with("docs/labs/"))
}

/// The D32 gated floating-point type and literal sites on one line of stripped code.
fn discrete_float_sites(line: &str, number: usize, is_cuda: bool, into: &mut Vec<Site>) {
    let b = line.as_bytes();
    let n = b.len();
    let mut i = 0usize;
    while i < n {
        let prev_word = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        let prev_dot = i > 0 && b[i - 1] == b'.';
        let prev_single_dot = prev_dot && (i < 2 || b[i - 2] != b'.');
        if (b[i].is_ascii_alphabetic() || b[i] == b'_') && !prev_word {
            let mut k = i + 1;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                k += 1;
            }
            let word = &line[i..k];
            if D32_FLOAT_TYPES.iter().any(|t| word.eq_ignore_ascii_case(t)) {
                into.push(Site {
                    line: number,
                    kind: "float-type",
                    text: word.to_string(),
                });
            }
            i = k;
            continue;
        }
        if b[i] == b'.'
            && !prev_word
            && !prev_dot
            && b.get(i + 1).is_some_and(|c| c.is_ascii_digit())
        {
            let prev_tuple_receiver = !is_cuda;
            let mut k = i + 2;
            while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\'')) {
                k += 1;
            }
            let mut has_exp = false;
            if matches!(b.get(k), Some(b'e' | b'E')) {
                has_exp = true;
                k += 1;
                if matches!(b.get(k), Some(b'+' | b'-')) {
                    k += 1;
                }
                while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\''))
                {
                    k += 1;
                }
            }
            let suff_start = k;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                k += 1;
            }
            let suff = &line[suff_start..k];
            if !prev_tuple_receiver || has_exp || !suff.is_empty() {
                into.push(Site {
                    line: number,
                    kind: "float-literal",
                    text: line[i..k].to_string(),
                });
            }
            i = k;
            continue;
        }
        if b[i].is_ascii_digit() && !prev_word && !prev_single_dot {
            if b[i] == b'0' && matches!(b.get(i + 1), Some(b'x' | b'X' | b'b' | b'B' | b'o' | b'O'))
            {
                let is_hex = matches!(b.get(i + 1), Some(b'x' | b'X'));
                let mut k = i + 2;
                if is_hex {
                    while k < n
                        && (b[k].is_ascii_hexdigit() || b[k] == b'_' || (is_cuda && b[k] == b'\''))
                    {
                        k += 1;
                    }
                    let has_hex_radix = b.get(k) == Some(&b'.')
                        && b.get(k + 1) != Some(&b'.')
                        && b.get(k + 1)
                            .is_some_and(|c| c.is_ascii_hexdigit() || matches!(c, b'p' | b'P'));
                    if has_hex_radix {
                        k += 1;
                        while k < n
                            && (b[k].is_ascii_hexdigit()
                                || b[k] == b'_'
                                || (is_cuda && b[k] == b'\''))
                        {
                            k += 1;
                        }
                    }
                    if matches!(b.get(k), Some(b'p' | b'P')) {
                        k += 1;
                        if matches!(b.get(k), Some(b'+' | b'-')) {
                            k += 1;
                        }
                        while k < n
                            && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\''))
                        {
                            k += 1;
                        }
                        while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                            k += 1;
                        }
                        into.push(Site {
                            line: number,
                            kind: "float-literal",
                            text: line[i..k].to_string(),
                        });
                        i = k;
                        continue;
                    }
                }
                while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                    k += 1;
                }
                i = k;
                continue;
            }
            let mut k = i + 1;
            while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\'')) {
                k += 1;
            }
            let mut has_dot = false;
            if b.get(k) == Some(&b'.') {
                let next = b.get(k + 1).copied();
                let is_cuda_dot_suff = is_cuda
                    && matches!(next, Some(b'f' | b'F' | b'd' | b'D'))
                    && !b
                        .get(k + 2)
                        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
                let is_cuda_dot_exp = is_cuda
                    && matches!(next, Some(b'e' | b'E'))
                    && (b.get(k + 2).is_some_and(|c| c.is_ascii_digit())
                        || (matches!(b.get(k + 2), Some(b'+' | b'-'))
                            && b.get(k + 3).is_some_and(|c| c.is_ascii_digit())));
                let is_range_or_ident = next == Some(b'.')
                    || (!is_cuda_dot_suff
                        && !is_cuda_dot_exp
                        && next.is_some_and(|c| c.is_ascii_alphabetic() || c == b'_'));
                if !is_range_or_ident {
                    has_dot = true;
                    k += 1;
                    while k < n
                        && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\''))
                    {
                        k += 1;
                    }
                }
            }
            let mut has_exp = false;
            if matches!(b.get(k), Some(b'e' | b'E')) {
                has_exp = true;
                k += 1;
                if matches!(b.get(k), Some(b'+' | b'-')) {
                    k += 1;
                }
                while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\''))
                {
                    k += 1;
                }
            }
            let suff_start = k;
            while k < n && (b[k].is_ascii_alphanumeric() || b[k] == b'_') {
                k += 1;
            }
            let suff = &line[suff_start..k];
            let has_float_suffix = ["f16", "bf16", "f32", "f64"]
                .iter()
                .any(|s| suff.eq_ignore_ascii_case(s))
                || (is_cuda && matches!(suff, "f" | "F" | "d" | "D"));
            if has_dot || has_exp || has_float_suffix {
                into.push(Site {
                    line: number,
                    kind: "float-literal",
                    text: line[i..k].to_string(),
                });
            }
            i = k;
            continue;
        }
        i += 1;
    }
}

/// Tensor-core intrinsics ([`TC_FRAGMENT`], [`TC_SYNC`], [`TC_WARPGROUP`], or [`TC_FIFTH_GEN`], any case),
/// non-overlapping, on stripped code.
#[cfg(test)]
fn tensor_core_sites(text: &str) -> u64 {
    let code = strip_comments_and_strings(text);
    let mut sites = Vec::new();
    for (k, line) in code.split('\n').enumerate() {
        matmul_sites(line, k + 1, &mut sites);
    }
    sites.iter().filter(|s| s.kind == "tensor-core").count() as u64
}

/// Scan the repository for one ratchet.
pub fn scan(repo: &Path, kind: Kind) -> Result<Scan, RatchetError> {
    let mut result = Scan::default();
    for (rel, path) in source_files(repo)? {
        if kind == Kind::DiscreteFloat && !is_discrete_scope(&rel) {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        match kind {
            Kind::NormExp => {
                let n = d23_sites(&text);
                if n > 0 {
                    result.counts.insert(rel, n);
                }
            }
            Kind::Matmul => {
                let lower = text.to_ascii_lowercase();
                if !lower.contains("matmul")
                    && !lower.contains("cublas")
                    && !lower.contains("gemm")
                    && !lower.contains(TC_FRAGMENT)
                    && !lower.contains(TC_SYNC)
                    && !lower.contains(TC_WARPGROUP)
                    && !lower.contains(TC_FIFTH_GEN)
                {
                    continue;
                }
                let code = strip_comments_and_strings(&text);
                let mut sites = Vec::new();
                for (k, line) in code.split('\n').enumerate() {
                    matmul_sites(line, k + 1, &mut sites);
                }
                let tc = sites.iter().filter(|s| s.kind == "tensor-core").count() as u64;
                if tc > 0 {
                    result.tensor_core += tc;
                    result.tensor_files += 1;
                }
                if !sites.is_empty() {
                    result.counts.insert(rel.clone(), sites.len() as u64);
                    result.detail.insert(rel, sites);
                }
            }
            Kind::DiscreteFloat => {
                let is_cuda = rel.ends_with(".cu") || rel.ends_with(".cuh");
                let code = strip_comments_strings_and_char_literals(&text);
                let mut sites = Vec::new();
                for (k, line) in code.split('\n').enumerate() {
                    discrete_float_sites(line, k + 1, is_cuda, &mut sites);
                }
                if !sites.is_empty() {
                    result.counts.insert(rel.clone(), sites.len() as u64);
                    result.detail.insert(rel, sites);
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
/// Paths inside `D37_SCANNED_FLOAT_CRATES` always enforce a strict zero ceiling (base = 0).
pub fn growth(
    current: &BTreeMap<String, u64>,
    baseline: &BTreeMap<String, u64>,
) -> Vec<(String, u64, u64)> {
    current
        .iter()
        .filter_map(|(p, &n)| {
            let base = if is_d37_float_scope(p) {
                0
            } else {
                baseline.get(p).copied().unwrap_or(0)
            };
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
/// matmul and discrete-float an empty baseline is the ratchet at zero.
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
    if matches!(kind, Kind::NormExp | Kind::Matmul) {
        let d37_violations: Vec<(String, u64, u64)> = current
            .iter()
            .filter(|(p, &n)| is_d37_float_scope(p) && n > 0)
            .map(|(p, &n)| (p.clone(), 0, n))
            .collect();
        if !d37_violations.is_empty() {
            return Ok(Update::Refused(d37_violations));
        }
    }
    let old = load_baseline(repo, kind)?;
    let first = match (&old, kind) {
        (None, _) => true,
        (Some(m), Kind::NormExp) => m.is_empty(),
        (Some(_), Kind::Matmul | Kind::DiscreteFloat) => false,
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
    fn tensor_core_count_is_non_overlapping_and_gated() {
        let s = format!(
            "nvcuda::{}::{}(a) {}_sync {}.mma_async {}.mma // {} \"{}\"",
            TC_FRAGMENT.to_uppercase(),
            TC_SYNC,
            TC_FRAGMENT,
            TC_WARPGROUP,
            TC_FIFTH_GEN,
            TC_SYNC,
            TC_FRAGMENT
        );
        assert_eq!(tensor_core_sites(&s), 5);
        let dir = repo_with(&[("crates/x/src/tc.cu", s)], None);
        let res = scan(dir.path(), Kind::Matmul).expect("scan matmul");
        assert_eq!(res.counts.get("crates/x/src/tc.cu"), Some(&5));
        assert_eq!(res.tensor_core, 5);
        assert_eq!(res.tensor_files, 1);
        assert_eq!(res.detail["crates/x/src/tc.cu"].len(), 5);
        assert!(res.detail["crates/x/src/tc.cu"]
            .iter()
            .all(|site| site.kind == "tensor-core"));
    }

    #[test]
    fn d23_word_counts_every_case() {
        let w = D23_WORD;
        let s = format!("{} {w}_rows LOG_{}", w.to_uppercase(), w.to_uppercase());
        assert_eq!(count_ci(&s, w), 3);
    }

    #[test]
    fn d23_ident_families_match_stripped_code_segments_not_comments_strings_or_proper() {
        let code_hits = "fn f() { rank_softplus(); stack_sigmoid_q31(); x.tanh(); gelu_erf(); silu_fwd(); apply_rope_f32(); ropei_table(); rotary_pos(); }\n";
        assert_eq!(d23_sites(code_hits), 8);

        let camel_and_c_hits = "enum E { RopeMode, RopeTheta, PreRopeProjectionProbe, RotaryEmbedding, SigmoidGate, SoftplusLoss, GeluActivation, SiluGate, RoPE, SiLU, GeLU } fn k() { tanhf(x); }\n";
        assert_eq!(d23_sites(camel_and_c_hits), 12);

        let clean = "// softplus sigmoid tanh gelu silu rope ropei rotary\nlet s = \"softplus sigmoid tanh gelu silu rope ropei rotary\";\nlet proper = property.properties(is_proper_prefix, Proper, Property, Properties, WriterProperties);\n";
        assert_eq!(d23_sites(clean), 0);

        let raw_kernel =
            "let k = r#\"__device__ void apply_rope_f32() { float y = tanhf(x); }\"#;\n";
        assert_eq!(d23_sites(raw_kernel), 2);
    }

    #[test]
    fn discrete_float_ratchet_catches_types_and_literals_without_tuple_or_range_false_positives() {
        let rust_code = "fn f<'a>(a: f32, b: f64, c: bf16, d: f16, s: &'a str) { let quote = '\"'; let esc = '\\\"'; let x = 1.0 + 2. + 1e-5 + 3f32 + 0x1.8p-2; let r = 0..1.0; let t = pair.0 + pair.0.1 + (pair).0 + arr[0].1
            .1; for i in 0..10 { let _ = i.max(1); } // f32 1.0\nlet lit = \"f64 2.5\"; }\n";
        let stripped = strip_comments_strings_and_char_literals(rust_code);
        let mut sites = Vec::new();
        for (k, line) in stripped.split('\n').enumerate() {
            discrete_float_sites(line, k + 1, false, &mut sites);
        }
        let found: Vec<_> = sites.iter().map(|s| (s.kind, s.text.as_str())).collect();
        assert_eq!(
            found,
            vec![
                ("float-type", "f32"),
                ("float-type", "f64"),
                ("float-type", "bf16"),
                ("float-type", "f16"),
                ("float-literal", "1.0"),
                ("float-literal", "2."),
                ("float-literal", "1e-5"),
                ("float-literal", "3f32"),
                ("float-literal", "0x1.8p-2"),
                ("float-literal", "1.0"),
            ]
        );

        let cuda_code =
            "__device__ void k() { float x = .5f + 1.f + 1.e3 + 2.e-4f + 1'000.25 + 0x1p-4; double y = 2.0; }\n";
        let stripped_cu = strip_comments_strings_and_char_literals(cuda_code);
        let mut cu_sites = Vec::new();
        for (k, line) in stripped_cu.split('\n').enumerate() {
            discrete_float_sites(line, k + 1, true, &mut cu_sites);
        }
        let cu_found: Vec<_> = cu_sites.iter().map(|s| (s.kind, s.text.as_str())).collect();
        assert_eq!(
            cu_found,
            vec![
                ("float-type", "float"),
                ("float-literal", ".5f"),
                ("float-literal", "1.f"),
                ("float-literal", "1.e3"),
                ("float-literal", "2.e-4f"),
                ("float-literal", "1'000.25"),
                ("float-literal", "0x1p-4"),
                ("float-type", "double"),
                ("float-literal", "2.0"),
            ]
        );

        let dir = repo_with(
            &[
                (
                    "crates/uor-r4-discrete/src/lib.rs",
                    "pub fn f() -> f32 { let quote = '\"'; 1.0 }\n".into(),
                ),
                (
                    "docs/labs/k16-timing-2026-10-15/instrument/src/main.rs",
                    "fn main() { let x = 1e-3; }\n".into(),
                ),
                (
                    "crates/uor-r4-training/src/lib.rs",
                    "pub fn f() -> f32 { 1.0 }\n".into(),
                ),
            ],
            None,
        );
        let res = scan(dir.path(), Kind::DiscreteFloat).expect("scan d32");
        assert_eq!(
            res.counts.get("crates/uor-r4-discrete/src/lib.rs"),
            Some(&2)
        );
        assert_eq!(
            res.counts
                .get("docs/labs/k16-timing-2026-10-15/instrument/src/main.rs"),
            Some(&1)
        );
        assert!(!res.counts.contains_key("crates/uor-r4-training/src/lib.rs"));
    }

    #[test]
    fn scan_includes_docs_labs_instrument_but_skips_other_docs_labs_files() {
        let hit = "fn f() { apply_rope_f32(); }\n".to_string();
        let dir = repo_with(
            &[
                (
                    "docs/labs/k16-timing-2026-10-15/instrument/src/lib.rs",
                    hit.clone(),
                ),
                (
                    "docs/labs/k16-timing-2026-10-15/instrument/target/debug/gen.rs",
                    hit.clone(),
                ),
                ("docs/labs/k16-timing-2026-10-15/other.rs", hit.clone()),
                ("docs/labs/k16-timing-2026-10-15/README.md", hit),
            ],
            None,
        );
        let s = scan(dir.path(), Kind::NormExp).expect("scan");
        assert_eq!(
            s.counts.keys().cloned().collect::<Vec<_>>(),
            vec!["docs/labs/k16-timing-2026-10-15/instrument/src/lib.rs".to_string()]
        );
        assert_eq!(s.total(), 1);
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

    #[test]
    fn d37_scanned_float_crate_is_scanned_for_d23_d25_with_zero_ceiling_and_allowlisted_from_d32() {
        assert_eq!(D37_SCANNED_FLOAT_CRATES, ["crates/uor-r4-float-train/"]);
        assert!(is_d37_float_scope("crates/uor-r4-float-train/src/lib.rs"));
        assert!(!is_d37_float_scope("crates/uor-r4-training/src/lib.rs"));
        assert!(!is_discrete_scope("crates/uor-r4-float-train/src/lib.rs"));

        let clean_float = "pub fn step(x: f32) -> f32 { x * 0.5f32 }\n".to_string();
        let dir = repo_with(
            &[("crates/uor-r4-float-train/src/lib.rs", clean_float)],
            None,
        );
        let d23 = scan(dir.path(), Kind::NormExp).expect("scan d23");
        let d25 = scan(dir.path(), Kind::Matmul).expect("scan d25");
        let d32 = scan(dir.path(), Kind::DiscreteFloat).expect("scan d32");
        assert_eq!(d23.total(), 0);
        assert_eq!(d25.total(), 0);
        assert_eq!(d32.total(), 0);

        let bad_d23 = format!("pub fn bad() {{ {}(); }}\n", D23_IDENT_FAMILIES[1]);
        fs::write(
            dir.path().join("crates/uor-r4-float-train/src/lib.rs"),
            bad_d23,
        )
        .expect("write bad d23");
        let d23_hit = scan(dir.path(), Kind::NormExp).expect("scan d23 hit");
        assert_eq!(
            d23_hit.counts.get("crates/uor-r4-float-train/src/lib.rs"),
            Some(&1)
        );
        match update_baseline(dir.path(), Kind::NormExp, &d23_hit.counts).expect("update d23") {
            Update::Refused(r) => assert_eq!(
                r,
                vec![("crates/uor-r4-float-train/src/lib.rs".to_string(), 0, 1)]
            ),
            other => panic!("D37 crate must refuse any D23 baseline site, got {other:?}"),
        }
        let mut forged_baseline = BTreeMap::new();
        forged_baseline.insert("crates/uor-r4-float-train/src/lib.rs".to_string(), 5u64);
        assert_eq!(
            growth(&d23_hit.counts, &forged_baseline),
            vec![("crates/uor-r4-float-train/src/lib.rs".to_string(), 0, 1)]
        );
    }
}
