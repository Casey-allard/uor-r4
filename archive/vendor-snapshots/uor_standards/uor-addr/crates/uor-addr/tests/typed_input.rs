//! CT — Typed-input class: claims that follow from the
//! `JsonValue`-typed input surface (CONFORMANCE.md §CT). The typed
//! input shape lets us distinguish `42` from `"42"`, reject
//! oversize/over-deep inputs at construction, and prove
//! structural-equivalence classes collapse to a single κ-label.
//!
//! Run via `just test` (debug) or `just conformance` (release).

#![allow(non_snake_case)]

use uor_addr::json::{address, AddressFailure, JsonValue, MAX_JSON_DEPTH};

// ───────────────────────────────────────────────────────────────────────────
// CT-T — Type distinction. Different scalar types must produce distinct
// κ-labels even when their textual rendering looks similar.
// ───────────────────────────────────────────────────────────────────────────

/// CT-T01 — `42` and `"42"` produce distinct κ-labels.
#[test]
fn ct_t01__integer_distinct_from_string_of_same_digits() {
    let int_addr = address(b"42").expect("valid").address;
    let str_addr = address(br#""42""#).expect("valid").address;
    assert_ne!(int_addr, str_addr);
}

/// CT-T02 — `null` and `false` produce distinct κ-labels.
#[test]
fn ct_t02__null_distinct_from_false() {
    let null_addr = address(b"null").expect("valid").address;
    let false_addr = address(b"false").expect("valid").address;
    assert_ne!(null_addr, false_addr);
}

/// CT-T03 — `true`, `false`, `null` are pairwise distinct.
#[test]
fn ct_t03__three_scalars_pairwise_distinct() {
    let t = address(b"true").expect("valid").address;
    let f = address(b"false").expect("valid").address;
    let n = address(b"null").expect("valid").address;
    assert_ne!(t, f);
    assert_ne!(t, n);
    assert_ne!(f, n);
}

/// CT-T04 — empty object and empty array are distinct.
#[test]
fn ct_t04__empty_object_distinct_from_empty_array() {
    let obj = address(b"{}").expect("valid").address;
    let arr = address(b"[]").expect("valid").address;
    assert_ne!(obj, arr);
}

/// CT-T05 — `[1,2,3]` (numbers) and `["1","2","3"]` (strings) distinct.
#[test]
fn ct_t05__number_array_distinct_from_string_array() {
    let nums = address(b"[1,2,3]").expect("valid").address;
    let strs = address(br#"["1","2","3"]"#).expect("valid").address;
    assert_ne!(nums, strs);
}

// ───────────────────────────────────────────────────────────────────────────
// CT-E — Structural equivalence. Inputs that differ only in syntactic
// noise (key order, whitespace, Unicode normalisation form) must
// collapse to one κ-label.
// ───────────────────────────────────────────────────────────────────────────

/// CT-E01 — key ordering invariance (re-statement of CD-I01a at the
/// typed-input layer).
#[test]
fn ct_e01__key_ordering_invariance() {
    let a = address(br#"{"alpha":1,"beta":2,"gamma":3}"#)
        .expect("valid")
        .address;
    let b = address(br#"{"gamma":3,"beta":2,"alpha":1}"#)
        .expect("valid")
        .address;
    assert_eq!(a, b);
}

/// CT-E02 — whitespace invariance (re-statement at the typed-input layer).
#[test]
fn ct_e02__whitespace_invariance() {
    let a = address(b"{ \"x\" : [ 1 , 2 , 3 ] }")
        .expect("valid")
        .address;
    let b = address(br#"{"x":[1,2,3]}"#).expect("valid").address;
    assert_eq!(a, b);
}

/// CT-E03 — Unicode-NFC invariance: composed `caf\u{00E9}` ≡ decomposed
/// `cafe\u{0301}`.
#[test]
fn ct_e03__nfc_invariance() {
    let composed = address("{\"x\":\"caf\u{00E9}\"}".as_bytes())
        .expect("valid")
        .address;
    let decomposed = address("{\"x\":\"cafe\u{0301}\"}".as_bytes())
        .expect("valid")
        .address;
    assert_eq!(composed, decomposed);
}

/// CT-E04 — nested key ordering invariance through depth 3.
#[test]
fn ct_e04__nested_key_ordering_invariance() {
    let a = br#"{"z":{"y":{"x":1,"w":2}},"a":1}"#;
    let b = br#"{"a":1,"z":{"y":{"w":2,"x":1}}}"#;
    let addr_a = address(a).expect("valid").address;
    let addr_b = address(b).expect("valid").address;
    assert_eq!(addr_a, addr_b);
}

// ───────────────────────────────────────────────────────────────────────────
// CT-B — Bound-enforcement at construction. The typed-input parser
// rejects inputs that violate any AddrBounds typed-input ceiling.
// ───────────────────────────────────────────────────────────────────────────

/// CT-B01 — over-deep nesting is rejected at parse (the stack-safety
/// depth guard remains, surfacing as InvalidJson).
#[test]
fn ct_b01__over_deep_nesting_rejected_at_parse() {
    let mut s = String::new();
    for _ in 0..(MAX_JSON_DEPTH + 4) {
        s.push('[');
    }
    for _ in 0..(MAX_JSON_DEPTH + 4) {
        s.push(']');
    }
    let err = address(s.as_bytes()).expect_err("must reject");
    assert!(matches!(err, AddressFailure::InvalidJson));
}

/// CT-B02 — wide strings are admitted (ADR-060 removed the width cap).
#[test]
fn ct_b02__wide_string_admitted() {
    let big: String = "a".repeat(100_000);
    let raw = format!("\"{big}\"");
    assert!(address(raw.as_bytes()).is_ok());
}

/// CT-B03 — exactly-at-bound depth is accepted.
#[test]
fn ct_b03__exactly_at_depth_bound_accepted() {
    let mut s = String::new();
    for _ in 0..MAX_JSON_DEPTH {
        s.push('[');
    }
    s.push('1');
    for _ in 0..MAX_JSON_DEPTH {
        s.push(']');
    }
    let addr = address(s.as_bytes()).expect("at-bound depth must be accepted");
    assert_eq!(addr.address.len(), 71);
}

/// CT-B04 — invalid JSON syntax is rejected with InvalidJson (not TooLarge).
#[test]
fn ct_b04__invalid_json_rejected_distinct_from_size_bound() {
    let err = address(b"not json").expect_err("must reject");
    assert!(matches!(err, AddressFailure::InvalidJson));
}

// ───────────────────────────────────────────────────────────────────────────
// CT-C — Cost-model commitment. The PrismModel declares
// `EmptyCommitment` (ADR-048); the model's typed-iso surface carries no
// auxiliary cost dimension beyond the κ-derivation.
// ───────────────────────────────────────────────────────────────────────────

/// CT-C01 — `EmptyCommitment` is the declared cost-model commitment.
#[test]
fn ct_c01__cost_model_is_empty_commitment() {
    // Compile-time check: re-export reaches our public surface.
    let _: uor_addr::EmptyCommitment = Default::default();
}

// ───────────────────────────────────────────────────────────────────────────
// CT-P — Parse-side typed-input bound: the JsonValue parser surface.
// ───────────────────────────────────────────────────────────────────────────

/// CT-P01 — JsonValue::parse on a valid input returns Ok with
/// non-empty tagged bytes.
#[test]
fn ct_p01__parse_returns_tagged_bytes() {
    let v = JsonValue::parse(br#"{"k":42}"#).expect("valid");
    assert!(!v.tagged_bytes().is_empty());
}

/// CT-P02 — JsonValue::parse rejects invalid JSON.
#[test]
fn ct_p02__parse_rejects_invalid_json() {
    let err = JsonValue::parse(b"{").expect_err("invalid JSON must fail");
    assert!(err.constraint_iri.contains("validUtf8Json"));
}
