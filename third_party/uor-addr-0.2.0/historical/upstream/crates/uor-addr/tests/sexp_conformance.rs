//! Conformance tests for the S-expression realization
//! ([`uor_addr::sexp`]) against Rivest's canonical S-expression
//! specification.
//!
//! ## Authoritative source
//!
//! Ronald L. Rivest, *S-expressions*, Internet Draft, dated May 4,
//! 1997. The current archival copy lives at
//! <https://people.csail.mit.edu/rivest/Sexp.txt> (with the I-D
//! form at
//! <https://datatracker.ietf.org/doc/html/draft-rivest-sexp-00>).
//! Cited by RFC 2693 (SPKI Certificate Theory, 1999) §3 "Canonical
//! S-Expressions" and the SPKI test vectors at
//! <https://datatracker.ietf.org/doc/html/rfc2693#section-11>.
//!
//! ## What this conformance suite pins
//!
//! - The Rivest canonical form for proper lists is the **flat list**
//!   `(s_1 s_2 … s_n)` (Sexp.txt §4.3), not the nested
//!   `(s_1 (s_2 (s_3 ())))` form.
//! - Atoms in canonical form are `<length>:<bytes>` with the length
//!   in ASCII decimal (Sexp.txt §4.2).
//! - The empty list canonical form is `()` (Sexp.txt §4.3).
//! - The token-list sugared form `(a b c)` parses to
//!   `Cons(Atom("a"), Cons(Atom("b"), Cons(Atom("c"), Nil)))` per
//!   the canonical Cons representation Lisp-family languages adopt.
//! - Canonical-form input round-trips through the canonicalizer
//!   byte-identically (idempotence).
//! - Different structural inputs yield different κ-labels
//!   (typed-distinction theorem from ARCHITECTURE.md V&V framework).

use uor_addr::sexp::{address, canonicalize, AddressFailure};

/// Fixtures derived from Rivest 1997 §6 "Examples", with the
/// canonical-form output computed per §4.3's flat-list rule and
/// §4.2's length-prefix atoms. Each fixture is a `(raw_input,
/// expected_canonical_bytes)` pair.
const RIVEST_FIXTURES: &[(&[u8], &[u8])] = &[
    // Empty list — Sexp.txt §4.3.
    (b"()", b"()"),
    // Atoms in canonical form pass through unchanged.
    (b"5:hello", b"5:hello"),
    (b"0:", b"0:"),
    // Token-list sugar collapses to flat canonical-form list.
    (b"(a b c)", b"(1:a 1:b 1:c)"),
    // Mixed-depth structure — Sexp.txt §4.3 nested lists.
    (b"(a (b c) d)", b"(1:a (1:b 1:c) 1:d)"),
    // Whitespace within token lists collapses in canonical form.
    (b"(  a   b   c  )", b"(1:a 1:b 1:c)"),
    (b"(a\tb\nc)", b"(1:a 1:b 1:c)"),
    // Canonical-form input round-trips.
    (b"(1:a 1:b 1:c)", b"(1:a 1:b 1:c)"),
];

#[test]
fn rivest_canonical_form_matches_published_specification() {
    for (raw, expected) in RIVEST_FIXTURES {
        let canonical = canonicalize(raw).unwrap_or_else(|_| panic!("canonicalize {raw:?}"));
        assert_eq!(
            canonical,
            *expected,
            "raw={:?} canonical={:?}",
            core::str::from_utf8(raw).unwrap_or("<binary>"),
            core::str::from_utf8(&canonical).unwrap_or("<binary>")
        );
    }
}

#[test]
fn canonicalize_is_idempotent_on_canonical_input() {
    // Sexp.txt §4 — canonical form is the unique representative of
    // an equivalence class.
    for (raw, _) in RIVEST_FIXTURES {
        let once = canonicalize(raw).unwrap_or_else(|_| panic!("first canonicalize"));
        let twice =
            canonicalize(&once).unwrap_or_else(|_| panic!("second canonicalize on {once:?}"));
        assert_eq!(once, twice, "idempotence broken for {raw:?}");
    }
}

#[test]
fn deterministic_kappa_derivation_across_runs() {
    // The κ-derivation determinism theorem (ARCHITECTURE.md V&V).
    let raw = b"(determinism (check (here)))";
    let a = address(raw).expect("κ-label").address;
    let b = address(raw).expect("κ-label").address;
    let c = address(raw).expect("κ-label").address;
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn structurally_equivalent_inputs_share_kappa_label() {
    // The structural-equivalence classification theorem — Rivest
    // canonical form is the equivalence class representative, so
    // any two inputs whose canonical bytes coincide share their
    // κ-label.
    let with_spaces = address(b"(  a   b   c  )").expect("κ-label").address;
    let without_spaces = address(b"(a b c)").expect("κ-label").address;
    let canonical_input = address(b"(1:a 1:b 1:c)").expect("κ-label").address;
    assert_eq!(with_spaces, without_spaces);
    assert_eq!(without_spaces, canonical_input);
}

#[test]
fn typed_distinction_atoms_vs_lists() {
    // The typed-distinction theorem — `Atom("a")` and `Cons(Atom("a"),
    // Nil)` (a 1-element list containing `a`) are structurally
    // distinct cases at the SExprValue grammar's case-IRI level, so
    // their κ-labels must differ.
    let atom = address(b"1:a").expect("κ-label").address;
    let singleton_list = address(b"(a)").expect("κ-label").address;
    let nil = address(b"()").expect("κ-label").address;
    assert_ne!(atom, singleton_list);
    assert_ne!(atom, nil);
    assert_ne!(singleton_list, nil);
}

#[test]
fn pipeline_rejects_invalid_input() {
    let unbalanced = address(b"((");
    assert!(matches!(unbalanced, Err(AddressFailure::InvalidSExpr)));

    let bad_canonical = address(b"99:short");
    assert!(matches!(bad_canonical, Err(AddressFailure::InvalidSExpr)));
}

#[test]
fn cross_format_distinction_against_json_realization() {
    // The same surface text parses differently in the two
    // realizations: `(a b c)` is a 3-atom list in S-expressions, and
    // not valid JSON. The κ-label IRIs are scoped per-axis (both use
    // SHA-256), but the realizations' typed-input shapes (`JsonValue`
    // vs `SExprValue`) produce different canonicalization byte
    // sequences for the same surface intent, so κ-labels differ.
    let sexpr_label = address(b"(a b c)").expect("κ-label").address;
    let json_label = uor_addr::json::address(br#"["a","b","c"]"#)
        .expect("κ-label")
        .address;
    assert_ne!(
        sexpr_label, json_label,
        "S-expression and JSON realizations produce distinct κ-labels for surface-equivalent inputs"
    );
}
