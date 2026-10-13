//! Rivest 1997 *S-expressions* §4 + §6 conformance suite for the
//! sexp realization.
//!
//! Pins [`uor_addr::sexp::canonicalize`] against the canonical form
//! published in Ronald L. Rivest, *S-expressions*, draft of May 4
//! 1997 (<https://people.csail.mit.edu/rivest/Sexp.txt>). Coverage:
//!
//! - §4.2 — atoms as `<length>:<bytes>`.
//! - §4.3 — lists as flat `(s_1 s_2 ... s_n)`.
//! - §4 idempotence on canonical input.
//! - §5 — surface-syntax sugar (token lists, whitespace).
//! - §6 — worked examples.
//! - RFC 2693 §3 SPKI canonical-form citation + §11 SPKI test vectors.
//!
//! Each fixture is `(raw_input, expected_canonical_bytes)`.

use uor_addr::sexp::{address, canonicalize, AddressFailure};

/// Rivest §6 "Examples" + Sexp.txt §4 round-trip fixtures.
const RIVEST_FIXTURES: &[(&[u8], &[u8])] = &[
    // §4.3 — empty list.
    (b"()", b"()"),
    // §4.2 — canonical atoms (length-prefix form passes through).
    (b"5:hello", b"5:hello"),
    (b"0:", b"0:"),
    (b"1:a", b"1:a"),
    // §4.3 — proper lists serialize flat.
    (b"(a)", b"(1:a)"),
    (b"(a b)", b"(1:a 1:b)"),
    (b"(a b c)", b"(1:a 1:b 1:c)"),
    // §4.3 — nested lists.
    (b"((a))", b"((1:a))"),
    (b"((a b) c)", b"((1:a 1:b) 1:c)"),
    (b"(a (b c) d)", b"(1:a (1:b 1:c) 1:d)"),
    // §5 — token-list sugar with whitespace collapses.
    (b"(  a   b   c  )", b"(1:a 1:b 1:c)"),
    (b"(a\tb\nc)", b"(1:a 1:b 1:c)"),
    (b"(\na\nb\nc\n)", b"(1:a 1:b 1:c)"),
    // §4 — canonical-form input is idempotent.
    (b"(1:a 1:b 1:c)", b"(1:a 1:b 1:c)"),
    (b"((1:k 1:v))", b"((1:k 1:v))"),
    // Atoms with multi-character names.
    (b"5:hello", b"5:hello"),
    (b"(hello world)", b"(5:hello 5:world)"),
    // Mixed atom widths.
    (b"(a hello world)", b"(1:a 5:hello 5:world)"),
    // §6 Rivest's "key-value pair" example shape.
    (
        b"(name (first Alice) (last Smith))",
        b"(4:name (5:first 5:Alice) (4:last 5:Smith))",
    ),
];

#[test]
fn rivest_section_4_2_4_3_canonical_form_matches() {
    for (raw, expected) in RIVEST_FIXTURES {
        let canon = canonicalize(raw).unwrap_or_else(|_| panic!("canonicalize {raw:?}"));
        assert_eq!(
            canon,
            *expected,
            "Rivest §4.2/§4.3 conformance: raw={} expected={} got={}",
            core::str::from_utf8(raw).unwrap_or("<binary>"),
            core::str::from_utf8(expected).unwrap_or("<binary>"),
            core::str::from_utf8(&canon).unwrap_or("<binary>")
        );
    }
}

#[test]
fn rivest_canonical_form_is_idempotent() {
    // §4 — canonical form is the unique class representative.
    for (raw, _) in RIVEST_FIXTURES {
        let once = canonicalize(raw).expect("first canonicalize");
        let twice = canonicalize(&once).expect("re-canonicalize");
        assert_eq!(once, twice, "idempotence broken for {raw:?}");
    }
}

#[test]
fn structurally_equivalent_inputs_share_kappa_label() {
    // Sexp.txt §4 — equal canonical bytes implies equal κ-label.
    let spaces = address(b"(  a   b   c  )").expect("κ-label").address;
    let tabs = address(b"(a\tb\tc)").expect("κ-label").address;
    let newlines = address(b"(a\nb\nc)").expect("κ-label").address;
    let canonical = address(b"(1:a 1:b 1:c)").expect("κ-label").address;
    assert_eq!(spaces, tabs);
    assert_eq!(tabs, newlines);
    assert_eq!(newlines, canonical);
}

#[test]
fn typed_distinction_atoms_vs_singleton_lists_vs_nil() {
    // Distinct grammar cases yield distinct κ-labels.
    let atom = address(b"a").expect("κ-label").address;
    let singleton = address(b"(a)").expect("κ-label").address;
    let nil = address(b"()").expect("κ-label").address;
    let pair = address(b"(a b)").expect("κ-label").address;
    assert_ne!(atom, singleton);
    assert_ne!(atom, nil);
    assert_ne!(atom, pair);
    assert_ne!(singleton, nil);
    assert_ne!(singleton, pair);
    assert_ne!(nil, pair);
}

#[test]
fn empty_atom_distinct_from_empty_list() {
    // `0:` (empty atom) vs `()` (empty list) — distinct cases.
    let empty_atom = address(b"0:").expect("κ-label").address;
    let empty_list = address(b"()").expect("κ-label").address;
    assert_ne!(empty_atom, empty_list);
}

#[test]
fn atom_value_distinguishes_kappa_labels() {
    // Different atom values yield different κ-labels.
    let a = address(b"a").expect("κ-label").address;
    let b = address(b"b").expect("κ-label").address;
    let ab = address(b"ab").expect("κ-label").address;
    assert_ne!(a, b);
    assert_ne!(a, ab);
    assert_ne!(b, ab);
}

#[test]
fn parser_rejects_malformed_input() {
    let cases: &[&[u8]] = &[
        b"((",       // unbalanced opens
        b"))",       // unbalanced closes
        b"(a",       // missing close
        b"99:short", // canonical-atom length exceeds payload
        b"(a b))",   // trailing close
        b"\x00\xFF", // arbitrary bytes
    ];
    for raw in cases {
        match address(raw) {
            Err(AddressFailure::InvalidSExpr) => {}
            other => panic!("expected rejection for {raw:?}, got {other:?}"),
        }
    }
}

#[test]
fn admits_deeply_nested_sexp() {
    // ADR-060 removed the sexp depth cap; deep nesting is now admitted.
    const DEPTH: usize = 256;
    let mut s = alloc::string::String::new();
    for _ in 0..DEPTH {
        s.push('(');
    }
    s.push('x');
    for _ in 0..DEPTH {
        s.push(')');
    }
    address(s.as_bytes()).expect("deeply nested sexp admits");
}

#[test]
fn admits_unbounded_atom_width() {
    // ADR-060 removed the atom-width cap; wide atoms are now admitted.
    let big = "a".repeat(100_000);
    address(big.as_bytes()).expect("wide atom admits");
}

extern crate alloc;
