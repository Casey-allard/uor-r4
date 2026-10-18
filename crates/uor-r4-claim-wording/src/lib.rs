//! Claim-wording gate (issue #123, docs/formal_vocabulary.md section 2.1).
//!
//! Rust port (owner rule D27) of the retired `scripts/check_claim_wording.py`, with the same scan,
//! the same three regular expressions copied verbatim, and byte-identical output.
//!
//! Scans normative Markdown (`docs/**/*.md`, `crates/*/README.md`, root `README.md`) for prohibited
//! claim phrases: "machine-verified", "machine verified", "exact equivalence", "exact teacher
//! equivalence", "provably equivalent". A line containing a prohibited phrase passes only when the
//! same line also links a proof artifact or certificate, explicitly disavows the claim, or carries
//! the escape hatch `claim-wording: allow`.
//!
//! Verbatim evidence archives (`docs/history/github-archive-*`) preserve historical comments
//! byte-identically; the gate governs normative documents, not preserved evidence. Files that are
//! not valid UTF-8 are skipped, as the Python gate skipped them.
#![forbid(unsafe_code)]

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

const PROHIBITED: &str =
    r"machine[- ]verified|exact(?:ly)?[- ](?:teacher[- ])?equivalen(?:ce|t)|provably[- ]equivalent";

const DISAVOWAL: &str = concat!(
    r"\bno\b|\bnot\b|\bnever\b|\bwithout\b|\bavoid\w*\b|\bprevent\w*\b",
    r"|\bprohibit\w*\b|\bforbid\w*\b|\bban(?:ned|s)?\b|\bdisavow\w*\b",
    r"|\bdo(?:es)?\s+not\b|\bdon't\b|\binformal\b",
);

const PROOF_LINK: &str = concat!(
    r"\]\([^)]*(?:proof|cert)[^)]*\)", // markdown link to a proof/cert artifact
    r"|proof_matrix|PROOF\.md|ProofStatus::Verified|certificate",
);

/// The same-line escape hatch.
pub const ALLOW_MARKER: &str = "claim-wording: allow";

/// Printed (with a trailing newline) when no violation is found.
pub const PASS_LINE: &str =
    "Claim-wording gate passed: no unlinked machine-verified/exact-equivalence wording.";

/// Path fragment of the verbatim evidence archives the gate does not govern.
const ARCHIVE_FRAGMENT: &str = "docs/history/github-archive-";

/// One prohibited phrase with no proof link, disavowal or allow marker on its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Path relative to the scanned root, `/`-separated.
    pub path: String,
    /// 1-based line number.
    pub line: usize,
    /// The line with surrounding whitespace removed.
    pub text: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.path, self.line, self.text)
    }
}

/// A file or directory the scan could not read.
#[derive(Debug)]
pub struct Error {
    pub path: PathBuf,
    pub source: std::io::Error,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot read {}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

struct Rules {
    prohibited: Regex,
    disavowal: Regex,
    proof_link: Regex,
}

fn rules() -> &'static Rules {
    static RULES: OnceLock<Rules> = OnceLock::new();
    RULES.get_or_init(|| {
        // The patterns are compile-time constants covered by every test; a failure here is a
        // programming error, not a recoverable condition.
        let build = |p: &str| match Regex::new(&format!("(?i){p}")) {
            Ok(r) => r,
            Err(e) => panic!("claim-wording pattern does not compile: {e}"),
        };
        Rules {
            prohibited: build(PROHIBITED),
            disavowal: build(DISAVOWAL),
            proof_link: build(PROOF_LINK),
        }
    })
}

/// Whether one line violates the gate.
pub fn line_violates(line: &str) -> bool {
    let r = rules();
    r.prohibited.is_match(line)
        && !(line.contains(ALLOW_MARKER)
            || r.disavowal.is_match(line)
            || r.proof_link.is_match(line))
}

/// Scan `root` and return every violation, in the Python gate's order.
pub fn check(root: &Path) -> Result<Vec<Violation>, Error> {
    let mut violations = Vec::new();
    for rel in candidate_files(root)? {
        let full = root.join(&rel);
        let bytes = std::fs::read(&full).map_err(|source| Error {
            path: full.clone(),
            source,
        })?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let rel_str = posix(&rel);
        for (idx, line) in split_lines(&text).into_iter().enumerate() {
            if line_violates(line) {
                violations.push(Violation {
                    path: rel_str.clone(),
                    line: idx + 1,
                    text: py_strip(line).to_string(),
                });
            }
        }
    }
    Ok(violations)
}

/// The gate's full stdout for `violations`, exactly as the Python gate printed it.
pub fn report(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return format!("{PASS_LINE}\n");
    }
    let mut out = String::from(
        "Claim-wording gate FAILED (docs/formal_vocabulary.md section 2.1):\n\
         Prohibited claim phrase without a linked proof artifact/certificate,\n\
         a same-line disavowal, or a 'claim-wording: allow' marker:\n\n",
    );
    for v in violations {
        out.push_str(&format!("  {v}\n"));
    }
    out
}

/// `docs/**/*.md`, then `crates/*/README.md`, then `README.md`, each group sorted by path
/// components (Python's `Path` ordering), archives excluded. Paths are relative to `root`.
fn candidate_files(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut docs = Vec::new();
    walk_md(root, Path::new("docs"), &mut docs)?;
    docs.sort();

    let mut crates = Vec::new();
    let crates_dir = root.join("crates");
    if crates_dir.is_dir() {
        for entry in read_dir(&crates_dir)? {
            let readme = Path::new("crates")
                .join(entry.file_name())
                .join("README.md");
            if root.join(&readme).is_file() {
                crates.push(readme);
            }
        }
    }
    crates.sort();

    let mut files = docs;
    files.extend(crates);
    if root.join("README.md").is_file() {
        files.push(PathBuf::from("README.md"));
    }
    files.retain(|p| !posix(p).contains(ARCHIVE_FRAGMENT));
    Ok(files)
}

fn walk_md(root: &Path, rel: &Path, out: &mut Vec<PathBuf>) -> Result<(), Error> {
    let dir = root.join(rel);
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in read_dir(&dir)? {
        let child = rel.join(entry.file_name());
        let full = root.join(&child);
        if full.is_dir() {
            walk_md(root, &child, out)?;
        } else if entry.file_name().to_string_lossy().ends_with(".md") {
            out.push(child);
        }
    }
    Ok(())
}

fn read_dir(dir: &Path) -> Result<Vec<std::fs::DirEntry>, Error> {
    let err = |source| Error {
        path: dir.to_path_buf(),
        source,
    };
    std::fs::read_dir(dir)
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err)
}

fn posix(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Python `str.splitlines()`: every Unicode line boundary, `\r\n` counted once, no trailing empty
/// line after a final boundary.
fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let is_break = matches!(
            c,
            '\n' | '\r'
                | '\u{0b}'
                | '\u{0c}'
                | '\u{1c}'
                | '\u{1d}'
                | '\u{1e}'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        );
        if !is_break {
            continue;
        }
        lines.push(&text[start..i]);
        let mut end = i + c.len_utf8();
        if c == '\r' {
            if let Some(&(j, '\n')) = chars.peek() {
                chars.next();
                end = j + 1;
            }
        }
        start = end;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

/// Python `str.strip()`: Rust's White_Space set plus the information separators U+001C..U+001F,
/// which Python's `str.isspace` also counts.
fn py_strip(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitlines_matches_python() {
        assert_eq!(split_lines("a\r\nb\rc\nd"), ["a", "b", "c", "d"]);
        assert_eq!(split_lines("a\n"), ["a"]);
        assert_eq!(split_lines("a\n\nb"), ["a", "", "b"]);
        assert_eq!(split_lines("a\u{2028}b\u{85}c"), ["a", "b", "c"]);
        assert!(split_lines("").is_empty());
    }

    #[test]
    fn strip_matches_python() {
        assert_eq!(py_strip("\u{a0}\t x \u{1f}"), "x");
    }
}
