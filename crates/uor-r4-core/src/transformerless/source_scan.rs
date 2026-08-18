//! The one source-scanner implementation behind every P-4-style source
//! witness in the workspace (#787 E-c).
//!
//! Four divergent copies previously existed (`transformerless::mod`'s
//! witnesses, `bott_fock`'s self-scan, graph-certify's `msa_selector_643`
//! scan, and graph-runtime's n-gram kernel scan), each with slightly
//! different comment handling and operator coverage — which meant four
//! places for a scanner bug to hide and no single falsifier suite. This
//! module is the canonical implementation: string-aware line-comment
//! stripping, value-operator detection for `*` `/` `%` (dereference-`*`
//! excluded), the method-form needles (`wrapping_mul(` and friends), an
//! optional float-token check, and an explicit, enumerable allowance
//! mechanism.
//!
//! ## Allowances
//!
//! A line whose comment carries `p4-allow(<scope>): <justification>` is
//! reported in [`ArithScanOutcome::allowed`] instead of `offenders`. Scans
//! that accept allowances must assert the exact expected `allowed` list,
//! so a new allowance can never ride in silently — it has to be added to
//! the witness's expectation, where review sees it. The marker is for
//! load-time/setup code inside otherwise-scanned modules (e.g. a
//! construction-phase `/ 2` that the steady-state boundary permits);
//! deployed hot-path code gets no allowances.

/// Strip a line comment, respecting string and char literals.
pub fn strip_line_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while i + 1 < bytes.len() {
        let ch = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if (in_string || in_char) && ch == b'\\' {
            escaped = true;
            i += 1;
            continue;
        }
        if !in_char && ch == b'"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if !in_string && ch == b'\'' {
            in_char = !in_char;
            i += 1;
            continue;
        }
        if !in_string && !in_char && ch == b'/' && bytes[i + 1] == b'/' {
            return &line[..i];
        }
        i += 1;
    }
    line
}

/// Outcome of a forbidden-arithmetic scan.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArithScanOutcome {
    /// Lines carrying forbidden value arithmetic (or float tokens, when
    /// requested) with no allowance marker. Any entry fails the witness.
    pub offenders: Vec<String>,
    /// Lines that WOULD have been offenders but carry an explicit
    /// `p4-allow(...)` marker. Witnesses must pin this list exactly.
    pub allowed: Vec<String>,
}

/// Marker that moves a flagged line from `offenders` to `allowed`.
pub const ALLOW_MARKER: &str = "p4-allow(";

/// Scan `src` for value `*` `/` `%` operators and the method forms
/// (`wrapping_mul(` etc.). Dereference `*x` and comment slashes are
/// excluded; comments are stripped string-aware before matching.
pub fn scan_for_forbidden_arith(src: &str) -> ArithScanOutcome {
    scan(src, false)
}

/// [`scan_for_forbidden_arith`] plus a float-token check (`f32`/`f64` in
/// stripped code) for witnesses that also assert float-freedom.
pub fn scan_for_forbidden_arith_and_floats(src: &str) -> ArithScanOutcome {
    scan(src, true)
}

fn prev_ident(code: &str, idx: usize) -> Option<&str> {
    let bytes = code.as_bytes();
    if idx == 0 {
        return None;
    }
    let mut j = idx;
    while j > 0 && bytes[j - 1] == b' ' {
        j -= 1;
    }
    let end = j;
    while j > 0 && (bytes[j - 1].is_ascii_alphanumeric() || bytes[j - 1] == b'_') {
        j -= 1;
    }
    if j == end {
        None
    } else {
        code.get(j..end)
    }
}

fn line_is_flagged(code: &str, check_floats: bool) -> bool {
    let b = code.as_bytes();
    for (i, &ch) in b.iter().enumerate() {
        if ch != b'*' && ch != b'/' && ch != b'%' {
            continue;
        }
        if ch == b'/' && ((i + 1 < b.len() && b[i + 1] == b'/') || (i >= 1 && b[i - 1] == b'/')) {
            continue; // comment slashes surviving inside string literals
        }
        let prev = if i >= 2 && b[i - 1] == b' ' {
            b[i - 2]
        } else if i >= 1 {
            b[i - 1]
        } else {
            b' '
        };
        let next = if i + 2 < b.len() && b[i + 1] == b' ' {
            b[i + 2]
        } else if i + 1 < b.len() {
            b[i + 1]
        } else {
            b' '
        };
        let operand_l = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b')' || c == b']';
        let operand_r = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'(';
        if ch == b'*'
            && operand_r(next)
            && matches!(
                prev_ident(code, i),
                None | Some("if")
                    | Some("while")
                    | Some("for")
                    | Some("loop")
                    | Some("match")
                    | Some("return")
                    | Some("let")
            )
        {
            continue; // pointer dereference
        }
        if operand_l(prev) && operand_r(next) {
            return true;
        }
    }
    for needle in [
        "wrapping_mul(",
        "saturating_mul(",
        "checked_mul(",
        ".mul(",
        "wrapping_div(",
        "saturating_div(",
        "checked_div(",
        ".div(",
        "wrapping_rem(",
        "saturating_rem(",
        "checked_rem(",
        ".rem(",
    ] {
        if code.contains(needle) {
            return true;
        }
    }
    if check_floats && (code.contains("f32") || code.contains("f64")) {
        return true;
    }
    false
}

/// Blank out string- and char-literal CONTENTS so literal text (paths
/// with `/`, prose mentioning `f32`) cannot flag the operator scan —
/// code inside a literal never executes. Lifetime ticks (`'a`) are NOT
/// treated as char-literal openers (a one-sided toggle would mask the
/// rest of the line and could hide a real operator), so a `'` only opens
/// a char literal when it closes like one (`'x'` or a short escape).
fn mask_literals(code: &str) -> String {
    let bytes = code.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while i < bytes.len() {
        let ch = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
                out.push(b' ');
            } else if ch == b'\\' {
                escaped = true;
                out.push(b' ');
            } else if ch == b'"' {
                in_string = false;
                out.push(b'"');
            } else {
                out.push(b' ');
            }
            i += 1;
            continue;
        }
        if ch == b'"' {
            in_string = true;
            out.push(b'"');
            i += 1;
            continue;
        }
        if ch == b'\'' {
            // Char literal only if it closes like one within a short
            // window; otherwise it is a lifetime tick and stays code.
            let close = bytes[i + 1..]
                .iter()
                .take(8)
                .position(|&c| c == b'\'')
                .map(|offset| i + 1 + offset);
            let is_char_literal = match close {
                Some(end) => end == i + 2 || bytes.get(i + 1) == Some(&b'\\'),
                None => false,
            };
            if let (true, Some(end)) = (is_char_literal, close) {
                out.push(b'\'');
                out.extend(core::iter::repeat_n(b' ', end - (i + 1)));
                out.push(b'\'');
                i = end + 1;
                continue;
            }
        }
        out.push(ch);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

fn scan(src: &str, check_floats: bool) -> ArithScanOutcome {
    let mut outcome = ArithScanOutcome::default();
    for (ln, line) in src.lines().enumerate() {
        let code = strip_line_comment(line).trim_start();
        if code.is_empty() {
            continue;
        }
        let masked = mask_literals(code);
        if line_is_flagged(&masked, check_floats) {
            let entry = format!("line {}: {}", ln + 1, code);
            if line.contains(ALLOW_MARKER) {
                outcome.allowed.push(entry);
            } else {
                outcome.offenders.push(entry);
            }
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #787 falsifiers: the scanner must fire on each seeded violation
    /// class and stay quiet on the legal look-alikes.
    #[test]
    fn seeded_violations_are_caught_and_lookalikes_pass() {
        let mul = "let x = a * b;";
        assert_eq!(scan_for_forbidden_arith(mul).offenders.len(), 1);
        let div = "let x = a / 2;";
        assert_eq!(scan_for_forbidden_arith(div).offenders.len(), 1);
        let rem = "let x = a % m;";
        assert_eq!(scan_for_forbidden_arith(rem).offenders.len(), 1);
        let method = "let x = a.wrapping_mul(b);";
        assert_eq!(scan_for_forbidden_arith(method).offenders.len(), 1);
        let float = "let x: f32 = 0.0;";
        assert!(scan_for_forbidden_arith(float).offenders.is_empty());
        assert_eq!(
            scan_for_forbidden_arith_and_floats(float).offenders.len(),
            1
        );

        let deref = "let x = *pointer;";
        assert!(scan_for_forbidden_arith(deref).offenders.is_empty());
        let comment = "// a * b in prose is fine";
        assert!(scan_for_forbidden_arith(comment).offenders.is_empty());
        let string_slash = "let path = \"a/b\";";
        assert!(scan_for_forbidden_arith(string_slash).offenders.is_empty());
        let string_float = "let label = \"f32 lanes\";";
        assert!(scan_for_forbidden_arith_and_floats(string_float)
            .offenders
            .is_empty());
        // A lifetime tick must NOT open a literal mask — masking the rest
        // of the line would hide the real multiply here.
        let lifetime_mul = "fn f<'a>(x: &'a i64, y: i64) -> i64 { x * y }";
        assert_eq!(scan_for_forbidden_arith(lifetime_mul).offenders.len(), 1);
        let char_slash = "let c = '/'; let ok = a ^ b;";
        assert!(scan_for_forbidden_arith(char_slash).offenders.is_empty());
    }

    /// The allowance mechanism is explicit and enumerable: a marked line
    /// moves to `allowed` (never silently passes), an unmarked one stays
    /// an offender.
    #[test]
    fn allow_marker_moves_lines_to_the_allowed_list() {
        let src = "let half = n / 2; // p4-allow(load-time): construction split\nlet bad = n / 2;";
        let outcome = scan_for_forbidden_arith(src);
        assert_eq!(outcome.allowed.len(), 1, "{outcome:?}");
        assert_eq!(outcome.offenders.len(), 1, "{outcome:?}");
        assert!(outcome.allowed[0].contains("line 1"));
        assert!(outcome.offenders[0].contains("line 2"));
    }

    /// String-aware comment stripping: `//` inside a string literal is not
    /// a comment, and code after a real comment is gone.
    #[test]
    fn comment_stripping_respects_string_literals() {
        assert_eq!(strip_line_comment("let a = 1; // b * c"), "let a = 1; ");
        assert_eq!(
            strip_line_comment("let url = \"http://x\"; let y = 2;"),
            "let url = \"http://x\"; let y = 2;"
        );
    }
}
