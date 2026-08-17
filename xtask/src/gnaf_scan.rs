//! GNAF forbidden-construct scan (SPEC 19), ported from the vendored
//! `proofs/wasm-gemm-gnaf/Tools/scan.py` (#653 phase-2 follow-up, third
//! `Tools/*.py` port after `gnaf_firewall.rs` and `gnaf_root.rs`).
//!
//! SPEC 19 bans `sorry` / `admit` / `native_decide` / a project-declared
//! `axiom` / `unsafe` / `partial` on the proof path. A plain text search
//! can't express that safely: `sorry` appears legitimately in doc comments
//! that explain the ban, and a gate that fires on its own documentation
//! gets switched off, which is the same failure as a gate that never
//! fires. This strips Lean block comments (nested `/- -/`), line comments
//! (`--`), and string literals first, then searches what remains -- the
//! same two-pass shape `Tools/scan.py` uses.
//!
//! The decisive audit is still `#print axioms` over the compiled
//! environment (`Tools/axioms.py`, which needs the Lean toolchain and so
//! isn't ported here per `docs/gnaf_integration_653.md`); this is defence
//! in depth, same as the Python original says of itself.

use std::path::{Path, PathBuf};

use crate::Fail;

/// SPEC 19: no `sorry`/`admit`/`native_decide` placeholder, no
/// project-declared `axiom`, no `unsafe`/`partial` declaration, in real
/// code (comments and string literals excluded).
pub fn gnaf_scan(root: &Path) -> Result<(), Fail> {
    let base = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf");
    if !base.exists() {
        return Err(format!(
            "{} not found; the vendored GNAF proof tree (#742) is expected \
             at this path",
            base.display()
        )
        .into());
    }

    let mut files = Vec::new();
    collect_lean(&base, &mut files)?;
    files.sort();

    let mut hits = Vec::new();
    for path in &files {
        // Python opens with errors='replace'; from_utf8_lossy is the same
        // tolerance (invalid sequences become U+FFFD rather than an error).
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        let stripped = strip_comments_and_strings(&text);
        for (i, line) in stripped.lines().enumerate() {
            if let Some(why) = banned_reason(line) {
                let shown: String = line.trim().chars().take(80).collect();
                hits.push(format!("{}:{}: {why}: {shown}", path.display(), i + 1));
            }
        }
    }

    if !hits.is_empty() {
        return Err(format!(
            "FORBIDDEN CONSTRUCT ON THE PROOF PATH (SPEC 19):\n{}",
            hits.iter()
                .map(|h| format!("  {h}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
        .into());
    }
    println!(
        "gnaf-scan: forbidden-construct scan clean: {} modules (comments \
         and strings excluded, SPEC 19)",
        files.len()
    );
    Ok(())
}

fn collect_lean(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_lean(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "lean") {
            out.push(path);
        }
    }
    Ok(())
}

/// Blank out Lean block comments (`/- -/`, nesting-aware, so `/-- -/` doc
/// comments are handled the same as plain ones), line comments (`--`), and
/// string literal bodies, while preserving line structure (a blanked
/// character becomes a space, a blanked newline stays a newline) so line
/// numbers in a later report line up with the original file. A faithful
/// character-by-character port of `Tools/scan.py`'s `strip()`.
pub(crate) fn strip_comments_and_strings(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut depth: i32 = 0;

    let two = |i: usize| -> Option<(char, char)> {
        if i + 1 < n {
            Some((chars[i], chars[i + 1]))
        } else {
            None
        }
    };

    while i < n {
        if depth == 0 && two(i) == Some(('/', '-')) {
            depth = 1;
            i += 2;
            out.push_str("  ");
            continue;
        }
        if depth > 0 {
            if two(i) == Some(('/', '-')) {
                depth += 1;
                i += 2;
                out.push_str("  ");
                continue;
            }
            if two(i) == Some(('-', '/')) {
                depth -= 1;
                i += 2;
                out.push_str("  ");
                continue;
            }
            out.push(if chars[i] == '\n' { '\n' } else { ' ' });
            i += 1;
            continue;
        }
        if two(i) == Some(('-', '-')) {
            while i < n && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if chars[i] == '"' {
            out.push(' ');
            i += 1;
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < n {
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                out.push(if chars[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            if i < n {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Words banned anywhere on the code path, matching `Tools/scan.py`'s
/// `(^|[^A-Za-z_.])(sorry|admit|native_decide)([^A-Za-z_]|$)`: not preceded
/// by a letter, underscore, or dot (so `Foo.sorry` and `asorry` don't
/// count), not followed by a letter or underscore (so `sorry2` still
/// counts -- a digit is not excluded, matching the Python character class
/// exactly).
const BANNED_WORDS: [&str; 3] = ["sorry", "admit", "native_decide"];

/// Does `line` contain `word` at a valid boundary, per the rule above?
fn contains_banned_word(line: &str, word: &str) -> bool {
    let mut start = 0usize;
    while let Some(rel) = line[start..].find(word) {
        let pos = start + rel;
        let before_ok = match line[..pos].chars().next_back() {
            None => true,
            Some(c) => !(c.is_ascii_alphabetic() || c == '_' || c == '.'),
        };
        let after = pos + word.len();
        let after_ok = match line[after..].chars().next() {
            None => true,
            Some(c) => !(c.is_ascii_alphabetic() || c == '_'),
        };
        if before_ok && after_ok {
            return true;
        }
        // Advance by one byte (word is ASCII, so this is also a char
        // boundary) to find the next occurrence, mirroring re.search
        // scanning past a boundary-rejected match.
        start = pos + 1;
    }
    false
}

/// Why (if at all) `line` trips SPEC 19, or `None` if it's clean. Matches
/// `Tools/scan.py`'s three `BANNED` patterns, checked in the same order.
fn banned_reason(line: &str) -> Option<&'static str> {
    for word in BANNED_WORDS {
        if contains_banned_word(line, word) {
            return Some("placeholder");
        }
    }

    // `^\s*axiom\s`: after stripping leading whitespace, the literal
    // "axiom" followed by exactly one whitespace character.
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("axiom") {
        if rest.starts_with(char::is_whitespace) {
            return Some("project-declared axiom");
        }
    }

    // `^\s*(unsafe|partial)\s+(def|abbrev|instance|theorem)`: after
    // stripping leading whitespace, one of the two keywords, then one or
    // more whitespace characters, then one of the four declaration
    // keywords (no trailing boundary required, matching the Python regex
    // exactly -- "theoremFoo" still counts).
    for keyword in ["unsafe", "partial"] {
        let Some(rest) = trimmed.strip_prefix(keyword) else {
            continue;
        };
        let after_ws = rest.trim_start();
        if after_ws.len() == rest.len() {
            continue; // no whitespace was actually consumed: \s+ needs >= 1
        }
        for decl in ["def", "abbrev", "instance", "theorem"] {
            if after_ws.starts_with(decl) {
                return Some("unsafe/partial");
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_blanks_nested_block_comments_and_keeps_line_count() {
        let src = "a\n/- outer /- inner -/ still outer -/\nb\n";
        let out = strip_comments_and_strings(src);
        assert_eq!(out.lines().count(), src.lines().count());
        assert!(!out.contains("outer"));
        assert!(!out.contains("inner"));
        assert!(out.starts_with("a\n"));
        assert!(out.trim_end().ends_with('b'));
    }

    #[test]
    fn strip_blanks_line_comments_but_stops_at_newline() {
        let out = strip_comments_and_strings("code -- sorry, a real comment\nmore code\n");
        assert!(out.starts_with("code "));
        assert!(!out.contains("sorry"));
        assert!(out.contains("more code"));
    }

    #[test]
    fn strip_blanks_string_bodies_including_escaped_quotes() {
        let out = strip_comments_and_strings(r#"let s := "sorry \" still inside" ; code"#);
        assert!(!out.contains("sorry"));
        assert!(out.contains("let s :="));
        assert!(out.contains("; code"));
    }

    #[test]
    fn banned_word_respects_boundaries_like_the_python_character_classes() {
        assert_eq!(
            banned_reason("theorem bad := by sorry"),
            Some("placeholder")
        );
        // preceded by a letter, underscore, or dot: not a hit.
        assert_eq!(banned_reason("let asorry := 1"), None);
        assert_eq!(banned_reason("Foo.sorry"), None);
        assert_eq!(banned_reason("let x_sorry := 1"), None);
        // followed by a digit is still a hit -- the Python class only
        // excludes letters/underscore after, not digits.
        assert_eq!(banned_reason("sorry2"), Some("placeholder"));
        // followed by an underscore or letter: not a hit.
        assert_eq!(banned_reason("sorry_helper()"), None);
        assert_eq!(banned_reason("sorryish()"), None);
    }

    #[test]
    fn axiom_requires_exactly_one_whitespace_char_after_the_keyword() {
        assert_eq!(
            banned_reason("axiom foo : Nat"),
            Some("project-declared axiom")
        );
        assert_eq!(
            banned_reason("  axiom foo : Nat"),
            Some("project-declared axiom")
        );
        assert_eq!(banned_reason("axiomatic foo"), None);
    }

    #[test]
    fn unsafe_partial_requires_whitespace_then_a_declaration_keyword() {
        assert_eq!(banned_reason("unsafe def foo := 1"), Some("unsafe/partial"));
        assert_eq!(
            banned_reason("partial   theorem bar : True := trivial"),
            Some("unsafe/partial")
        );
        assert_eq!(banned_reason("unsafedef foo"), None);
        assert_eq!(banned_reason("unsafe helper foo"), None);
    }

    fn scratch_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xtask-gnaf-scan-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    /// Regression guard mirroring `Tools/mutation.py`'s M5: a `sorry`
    /// mentioned legitimately inside a doc comment must NOT trip the scan,
    /// and a real `sorry` in a theorem body must. Built entirely under
    /// `std::env::temp_dir()`, never against the real vendored tree.
    #[test]
    fn gnaf_scan_ignores_a_documented_mention_but_catches_a_real_sorry() {
        let root = scratch_dir("doc-mention");
        let base = root.join("proofs/wasm-gemm-gnaf/WasmGemmGnaf/Foundation");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(
            base.join("Planted.lean"),
            "/-- doc mentioning sorry legitimately -/\n\
             theorem good : True := trivial\n",
        )
        .unwrap();
        assert!(gnaf_scan(&root).is_ok());

        std::fs::write(
            base.join("Planted.lean"),
            "/-- doc mentioning sorry legitimately -/\n\
             theorem good : True := trivial\n\
             theorem bad : True := by sorry\n",
        )
        .unwrap();
        assert!(gnaf_scan(&root).is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
