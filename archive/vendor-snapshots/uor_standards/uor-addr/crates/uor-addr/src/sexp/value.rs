//! `SExprValue` — the typed S-expression input handle (ADR-023 amended
//! by ADR-060).
//!
//! Under ADR-060 the S-expression realization no longer copies a
//! structurally-tagged byte form into a fixed buffer. The host-boundary
//! parser [`SExprCanon::validate`] checks the grammar in a single pass
//! over the borrowed input — balanced parentheses (tracked with an
//! unbounded `usize` depth counter), well-formed canonical atoms, exactly
//! one top-level value — and the input handle then flows through the
//! pipeline as a [`TermValue::Stream`] carrier ([`SExprCanon`]). The
//! carrier re-tokenizes the borrowed bytes on demand and emits Rivest
//! canonical S-expression bytes chunk-by-chunk; ψ₉ folds those chunks
//! through the σ-axis with bounded resident memory. There is no input
//! size ceiling, no atom-width cap, no element-count cap, and no nesting
//! depth cap.
//!
//! # Input syntax
//!
//! [`SExprCanon::validate`] admits two equivalent surface syntaxes:
//!
//! - **Canonical (Rivest 1997 §4.3)** — `<n>:<bytes>` for atoms,
//!   `(<value> <value>)` for cons, `()` for nil.
//! - **Token list** — whitespace-separated tokens between parentheses,
//!   each token interpreted as an atom whose bytes are the token's UTF-8
//!   representation.
//!
//! # Canonical form
//!
//! The carrier emits Rivest's canonical S-expression form: atoms as
//! `<n>:<bytes>` (raw length prefix, no quoting), proper lists as
//! `(s₁ s₂ … sₙ)` with single-space separators, nil as `()`. The
//! emission is a streaming structural rewrite — element order is
//! preserved, so it is reproducible chunk-by-chunk from the borrowed
//! input with no intermediate buffer.

use prism::operation::TermValue;
use prism::pipeline::{
    ConstrainedTypeShape, ConstraintRef, IntoBindingValue, PartitionProductFields, ShapeViolation,
    ViolationKind,
};
use prism::uor_foundation::pipeline::ChunkSource;

// ─── ShapeViolation IRI ─────────────────────────────────────────────────

/// The single grammar-validity violation the host-boundary parser raises.
/// ADR-060 removed the depth / atom-width / element-count / total-width
/// ceilings, so the only rejection is "not a well-formed UTF-8
/// S-expression".
pub(crate) const INVALID_SEXPR_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://uor.foundation/addr/SExprValue",
    constraint_iri: "https://uor.foundation/addr/SExprValue/validUtf8SExpr",
    property_iri: "https://uor.foundation/addr/inputBytes",
    expected_range: "https://uor.foundation/addr/ValidUtf8SExpr",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

// ─── Tokenizer ──────────────────────────────────────────────────────────

/// A lexical token of the S-expression surface syntax.
enum Tok<'a> {
    /// `(`
    Open,
    /// `)`
    Close,
    /// An atom's raw byte payload (canonical `<n>:<bytes>` or a bare
    /// whitespace-delimited token).
    Atom(&'a [u8]),
}

/// Single-pass tokenizer over `raw`. Invokes `on_tok` for each lexical
/// token in source order. Reports the lexical errors that are independent
/// of nesting structure (malformed canonical-atom length prefix,
/// truncated canonical atom); structural validation (balanced parens,
/// single top-level value) layers on top via the callback.
fn for_each_token<'a>(
    raw: &'a [u8],
    on_tok: &mut dyn FnMut(Tok<'a>),
) -> Result<(), ShapeViolation> {
    let mut pos = 0;
    while pos < raw.len() {
        let b = raw[pos];
        if b.is_ascii_whitespace() {
            pos += 1;
            continue;
        }
        if b == b'(' {
            on_tok(Tok::Open);
            pos += 1;
            continue;
        }
        if b == b')' {
            on_tok(Tok::Close);
            pos += 1;
            continue;
        }
        if b.is_ascii_digit() {
            // Canonical atom iff the leading digit run is terminated by ':'.
            let mut i = pos;
            while i < raw.len() && raw[i].is_ascii_digit() {
                i += 1;
            }
            if i < raw.len() && raw[i] == b':' {
                let len = parse_usize(&raw[pos..i]).ok_or(INVALID_SEXPR_VIOLATION)?;
                let start = i + 1;
                let end = start.checked_add(len).ok_or(INVALID_SEXPR_VIOLATION)?;
                if end > raw.len() {
                    return Err(INVALID_SEXPR_VIOLATION);
                }
                on_tok(Tok::Atom(&raw[start..end]));
                pos = end;
                continue;
            }
            // Digits not followed by ':' — fall through to a token atom.
        }
        // Token atom: a maximal run of non-whitespace, non-paren bytes.
        let start = pos;
        while pos < raw.len() {
            let c = raw[pos];
            if c.is_ascii_whitespace() || c == b'(' || c == b')' {
                break;
            }
            pos += 1;
        }
        // Non-empty by construction (`b` was neither whitespace nor paren).
        on_tok(Tok::Atom(&raw[start..pos]));
    }
    Ok(())
}

/// Parse an ASCII decimal digit run into a `usize`, returning `None` on
/// overflow (an atom length wider than the address space — rejected, not
/// capped at an arbitrary ceiling).
fn parse_usize(digits: &[u8]) -> Option<usize> {
    if digits.is_empty() {
        return None;
    }
    let mut n: usize = 0;
    for &d in digits {
        let v = (d.wrapping_sub(b'0')) as usize;
        if v > 9 {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(v)?;
    }
    Some(n)
}

/// `usize → ASCII decimal` into a 20-byte scratch (`u64::MAX` is 20
/// digits).
fn format_usize_into(buf: &mut [u8; 20], mut n: usize) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let mut idx = buf.len();
    while n > 0 {
        idx -= 1;
        buf[idx] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    &buf[idx..]
}

// ─── SExprCanon — the streaming canonical-form carrier ───────────────────

/// The Rivest-canonical-form [`ChunkSource`] over a borrowed,
/// grammar-validated S-expression byte slice. Constructed by
/// [`crate::sexp::address`] after [`SExprCanon::validate`] succeeds; lives
/// in the caller's stack frame while the model folds it.
#[derive(Clone, Copy, Debug)]
pub struct SExprCanon<'a> {
    raw: &'a [u8],
}

impl<'a> SExprCanon<'a> {
    /// Wrap a grammar-validated raw S-expression slice. Call
    /// [`SExprCanon::validate`] first; the carrier assumes well-formed
    /// input (its [`ChunkSource`] emission cannot surface errors).
    #[must_use]
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    /// Validate `raw` against the S-expression grammar in a single pass:
    /// UTF-8, balanced parentheses, exactly one top-level value, and
    /// well-formed canonical atoms.
    ///
    /// # Errors
    ///
    /// [`INVALID_SEXPR_VIOLATION`] (`validUtf8SExpr`) if `raw` is not a
    /// well-formed UTF-8 S-expression.
    pub fn validate(raw: &[u8]) -> Result<(), ShapeViolation> {
        core::str::from_utf8(raw).map_err(|_| INVALID_SEXPR_VIOLATION)?;
        let mut depth: usize = 0;
        let mut seen_top = false;
        let mut err: Option<ShapeViolation> = None;
        for_each_token(raw, &mut |tok| {
            if err.is_some() {
                return;
            }
            match tok {
                Tok::Atom(_) => {
                    if depth == 0 {
                        if seen_top {
                            err = Some(INVALID_SEXPR_VIOLATION);
                        } else {
                            seen_top = true;
                        }
                    }
                }
                Tok::Open => {
                    if depth == 0 && seen_top {
                        err = Some(INVALID_SEXPR_VIOLATION);
                    } else {
                        depth += 1;
                    }
                }
                Tok::Close => {
                    if depth == 0 {
                        err = Some(INVALID_SEXPR_VIOLATION);
                    } else {
                        depth -= 1;
                        if depth == 0 {
                            seen_top = true;
                        }
                    }
                }
            }
        })?;
        if let Some(e) = err {
            return Err(e);
        }
        if depth != 0 || !seen_top {
            return Err(INVALID_SEXPR_VIOLATION);
        }
        Ok(())
    }
}

impl ChunkSource for SExprCanon<'_> {
    fn for_each_chunk(&self, f: &mut dyn FnMut(&[u8])) {
        // Streaming Rivest canonicalization: a single space separates
        // consecutive elements of a list; no space immediately follows an
        // opening paren. A single running flag captures this across any
        // nesting depth — `need_space` is set after every atom and `)`,
        // cleared after every `(`.
        let mut need_space = false;
        // Validation has already succeeded for `self.raw`, so the
        // tokenizer cannot error here; ignore its Result.
        let _ = for_each_token(self.raw, &mut |tok| match tok {
            Tok::Open => {
                if need_space {
                    f(b" ");
                }
                f(b"(");
                need_space = false;
            }
            Tok::Close => {
                f(b")");
                need_space = true;
            }
            Tok::Atom(bytes) => {
                if need_space {
                    f(b" ");
                }
                let mut buf = [0u8; 20];
                f(format_usize_into(&mut buf, bytes.len()));
                f(b":");
                f(bytes);
                need_space = true;
            }
        });
    }
}

// ─── SExprValue — the typed input handle ─────────────────────────────────

/// Typed S-expression input handle (ADR-060 stream carrier). A thin,
/// `Copy` borrow of a [`SExprCanon`]; `as_binding_value` returns the
/// `Stream` carrier zero-copy.
#[derive(Clone, Copy, Debug)]
pub struct SExprValue<'a>(&'a SExprCanon<'a>);

impl<'a> SExprValue<'a> {
    /// Wrap a validated canonical-form carrier as a model input handle.
    #[must_use]
    pub fn new(canon: &'a SExprCanon<'a>) -> Self {
        Self(canon)
    }
}

impl ConstrainedTypeShape for SExprValue<'_> {
    const IRI: &'static str = "https://uor.foundation/addr/SExprValue";
    const SITE_COUNT: usize = 1;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
    const CYCLE_SIZE: u64 = u64::MAX;
}

impl prism::uor_foundation::pipeline::__sdk_seal::Sealed for SExprValue<'_> {}

impl<'a> IntoBindingValue<'a> for SExprValue<'a> {
    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> TermValue<'a, INLINE_BYTES> {
        // The Rivest canonical form streams from the borrowed carrier; ψ₉
        // folds it chunk-by-chunk. `self.0` is `&'a SExprCanon`, so the
        // returned carrier borrows the input's `'a`-lived data
        // independently of the `&self` call borrow.
        TermValue::stream(self.0)
    }
}

impl PartitionProductFields for SExprValue<'_> {
    const FIELDS: &'static [(u32, u32)] = &[];
    const FIELD_NAMES: &'static [&'static str] = &[];
}

// ─── Convenience alloc surface (feature = "alloc") ──────────────────────

/// Validate `raw` and materialize its Rivest canonical S-expression bytes
/// — the same byte sequence ψ₉ folds through the σ-axis.
///
/// **Available only under the `alloc` feature.** The no_alloc pipeline
/// path never materializes the canonical form; it streams it through the
/// hasher via [`SExprCanon`]'s [`ChunkSource`] impl.
///
/// # Errors
///
/// Surfaces the [`ShapeViolation`] [`SExprCanon::validate`] would raise.
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, ShapeViolation> {
    extern crate alloc;
    SExprCanon::validate(raw)?;
    let canon = SExprCanon::new(raw);
    let mut out = alloc::vec::Vec::new();
    canon.for_each_chunk(&mut |chunk| out.extend_from_slice(chunk));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_nil() {
        SExprCanon::validate(b"()").expect("valid nil");
    }

    #[test]
    fn validates_atom_canonical_form() {
        SExprCanon::validate(b"5:hello").expect("valid canonical atom");
    }

    #[test]
    fn validates_token_list() {
        SExprCanon::validate(b"(a b c)").expect("valid token list");
    }

    #[test]
    fn rejects_unbalanced_parens() {
        let err = SExprCanon::validate(b"((").expect_err("unbalanced parens");
        assert_eq!(err.constraint_iri, INVALID_SEXPR_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_two_top_level_values() {
        // Two bare token atoms, and two sibling lists — neither is a single
        // top-level value. (`5:a 5:b` would *not* qualify: the `5:` length
        // prefix swallows ` 5:b` as one canonical atom payload.)
        for raw in [b"abc def".as_slice(), b"(a)(b)".as_slice()] {
            let err = SExprCanon::validate(raw).expect_err("two top-level");
            assert_eq!(err.constraint_iri, INVALID_SEXPR_VIOLATION.constraint_iri);
        }
    }

    #[test]
    fn rejects_truncated_canonical_atom() {
        let err = SExprCanon::validate(b"5:abc").expect_err("declared 5, has 3");
        assert_eq!(err.constraint_iri, INVALID_SEXPR_VIOLATION.constraint_iri);
    }

    #[test]
    fn accepts_arbitrary_nesting_depth() {
        extern crate alloc;
        let mut s = alloc::string::String::new();
        for _ in 0..4096 {
            s.push('(');
        }
        s.push('x');
        for _ in 0..4096 {
            s.push(')');
        }
        // No depth cap (ADR-060): deep nesting is accepted, not rejected.
        SExprCanon::validate(s.as_bytes()).expect("deep nesting is valid");
    }

    #[cfg(feature = "alloc")]
    const CANONICAL_FIXTURES: &[(&[u8], &[u8])] = &[
        (b"()", b"()"),
        (b"(a b c)", b"(1:a 1:b 1:c)"),
        (b"5:hello", b"5:hello"),
        (b"(hello world)", b"(5:hello 5:world)"),
        (b"((a) (b))", b"((1:a) (1:b))"),
        (b"(a (b c) d)", b"(1:a (1:b 1:c) 1:d)"),
        (b"(  a\t b\n c  )", b"(1:a 1:b 1:c)"),
        (b"(1:a 1:b 1:c)", b"(1:a 1:b 1:c)"),
        (b"(())", b"(())"),
    ];

    #[cfg(feature = "alloc")]
    #[test]
    fn canonicalizer_matches_rivest_canonical_form() {
        for (raw, expected) in CANONICAL_FIXTURES {
            let canon = canonicalize(raw).expect("valid");
            assert_eq!(canon, *expected, "raw={raw:?}");
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn canonicalize_is_idempotent_on_its_own_output() {
        for (raw, _expected) in CANONICAL_FIXTURES {
            let once = canonicalize(raw).expect("valid");
            let twice = canonicalize(&once).expect("re-canonicalises");
            assert_eq!(once, twice, "idempotence broken for {raw:?}");
        }
    }
}
