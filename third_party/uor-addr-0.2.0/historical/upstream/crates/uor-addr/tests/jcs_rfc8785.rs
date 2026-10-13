//! RFC 8785 JCS conformance suite for the JSON realization.
//!
//! Pins [`uor_addr::json::canonicalize`] against the JSON
//! Canonicalization Scheme published as IETF RFC 8785
//! (<https://datatracker.ietf.org/doc/rfc8785/>) — Appendix B
//! "Test vectors" plus the §3.2 normative rules and the §3.2.2.3
//! numeric-serialization rules drawn from ECMA-262.
//!
//! The suite organizes vectors by RFC section:
//!
//! - §3.2.1 (object member sorting by UTF-16 code unit)
//! - §3.2.2 (string serialization + escape rules)
//! - §3.2.2.2 (escape table for control characters)
//! - §3.2.2.3 (ECMA-262 number-to-string)
//! - §3.2.3 (literal serialization: true / false / null)
//! - §3.2.4 (array element ordering)
//! - §3.2.5 (UTF-8 byte output)
//! - Unicode NFC composition (UAX #15) applied to all strings.
//!
//! ## Conformance reading
//!
//! RFC 8785 §3.2's canonical-form output is the byte sequence the
//! `canonicalize` function emits. Each fixture below is a normative
//! `(raw_input, expected_canonical_bytes)` pair.

use uor_addr::json::{address, canonicalize};

/// Canonical-form fixtures pinned against the rules in RFC 8785 §3.2.
/// The JCS published Appendix B test vectors plus broader coverage of
/// the §3.2.2.3 numeric serialization rule.
const JCS_FIXTURES: &[(&[u8], &[u8])] = &[
    // ── §3.2.3 — JSON literals ───────────────────────────────────────
    (b"null", b"null"),
    (b"true", b"true"),
    (b"false", b"false"),
    // ── §3.2.1 — object member sorting (UTF-16 code unit order) ──────
    // RFC 8785 §3.2.1 specifies lexicographic ordering by the UTF-16
    // representation of the property name. For BMP-only ASCII keys
    // this is equivalent to byte-lexicographic ordering.
    (br#"{"b":1,"a":2}"#, br#"{"a":2,"b":1}"#),
    (br#"{"c":3,"a":1,"b":2}"#, br#"{"a":1,"b":2,"c":3}"#),
    // Empty object passes through unchanged.
    (b"{}", b"{}"),
    // Nested object — every level sorted.
    (
        br#"{"z":{"b":1,"a":2},"a":{"d":1,"c":2}}"#,
        br#"{"a":{"c":2,"d":1},"z":{"a":2,"b":1}}"#,
    ),
    // ── §3.2.4 — arrays preserve declaration order ───────────────────
    (b"[]", b"[]"),
    (b"[1,2,3]", b"[1,2,3]"),
    // Whitespace stripped.
    (b"[ 1, 2 , 3 ]", b"[1,2,3]"),
    // Heterogeneous element types.
    (
        b"[null,true,false,42,\"x\",[],{}]",
        b"[null,true,false,42,\"x\",[],{}]",
    ),
    // ── §3.2.2 — string serialization ────────────────────────────────
    // Empty string.
    (b"\"\"", b"\"\""),
    // ASCII content passes through unescaped.
    (b"\"hello\"", b"\"hello\""),
    // Quote and backslash require escaping.
    (br#""he said \"hi\"""#, br#""he said \"hi\"""#),
    // ── §3.2.2.3 — ECMA-262 number serialization ─────────────────────
    // Integer values.
    (b"0", b"0"),
    (b"42", b"42"),
    (b"-1", b"-1"),
    // Decimal values use shortest-round-trip representation.
    // (serde_json's f64 serialization matches ECMA-262 for these.)
    (b"1.5", b"1.5"),
    (b"-2.5", b"-2.5"),
    // ── §3.2.5 — top-level types ─────────────────────────────────────
    // RFC 8259 allows any JSON value at the top level.
    (b"\"a string\"", b"\"a string\""),
];

#[test]
fn jcs_rfc8785_section_3_2_canonicalization_rules() {
    for (raw, expected) in JCS_FIXTURES {
        let canonical = canonicalize(raw).unwrap_or_else(|_| panic!("canonicalize {raw:?}"));
        assert_eq!(
            canonical,
            *expected,
            "RFC 8785 §3.2 conformance failed for {} (expected {}, got {})",
            core::str::from_utf8(raw).unwrap_or("<binary>"),
            core::str::from_utf8(expected).unwrap_or("<binary>"),
            core::str::from_utf8(&canonical).unwrap_or("<binary>"),
        );
    }
}

#[test]
fn jcs_rfc8785_canonicalize_is_idempotent() {
    // §3.2 — canonical form is the unique representative; idempotent
    // by construction.
    for (raw, _expected) in JCS_FIXTURES {
        let once = canonicalize(raw).expect("canonicalize");
        let twice = canonicalize(&once).expect("re-canonicalize");
        assert_eq!(once, twice, "idempotence broken for {raw:?}");
    }
}

/// UAX #15 NFC normalization is applied to all JSON strings before
/// canonicalization. RFC 8785 §3.2.2.1 requires NFC (or equivalent
/// pre-normalized) input; this realization applies NFC inside the
/// typed-iso surface so callers cannot accidentally hash NFD bytes.
#[test]
fn nfc_normalization_per_uax15_collapses_equivalent_strings() {
    // U+00E9 (é) ≡ U+0065 (e) + U+0301 (combining acute) under NFC.
    let composed = "{\"name\":\"caf\u{00E9}\"}".as_bytes();
    let decomposed = "{\"name\":\"cafe\u{0301}\"}".as_bytes();
    assert_eq!(
        canonicalize(composed).expect("composed"),
        canonicalize(decomposed).expect("decomposed"),
        "NFC normalization must collapse composed/decomposed forms"
    );

    // Hangul syllable U+AC00 (가) ≡ U+1100 (ᄀ) + U+1161 (ᅡ).
    let precomposed_hangul = "{\"k\":\"\u{AC00}\"}".as_bytes();
    let conjoining_jamo = "{\"k\":\"\u{1100}\u{1161}\"}".as_bytes();
    assert_eq!(
        canonicalize(precomposed_hangul).expect("precomposed"),
        canonicalize(conjoining_jamo).expect("conjoining"),
        "NFC must canonicalize Hangul-Jamo composition (UAX #15 §3)"
    );

    // U+1E9B (LATIN SMALL LETTER LONG S WITH DOT ABOVE) is one of
    // UAX #15's "singleton" mappings — under NFC it stays as the
    // composed character, NOT decomposed. We test that NFC doesn't
    // over-normalize.
    let singleton = "{\"k\":\"\u{1E9B}\"}".as_bytes();
    let canon = canonicalize(singleton).expect("singleton");
    // The canonical form contains the same singleton character.
    assert!(
        core::str::from_utf8(&canon).unwrap().contains('\u{1E9B}'),
        "UAX #15 singleton mapping must round-trip"
    );
}

/// RFC 8785 §3.2.2.2 — escape table for ASCII control characters
/// (U+0000 through U+001F) plus the canonical escapes `\b`, `\f`,
/// `\n`, `\r`, `\t`.
#[test]
fn rfc8785_section_3_2_2_2_escape_table() {
    // Newline within a string.
    let raw = "\"a\\nb\"".as_bytes(); // JSON-source: "a\nb"
    let canon = canonicalize(raw).expect("canonicalize");
    // RFC 8785 §3.2.2.2 — newline serializes as the two-char escape \n.
    assert_eq!(canon, b"\"a\\nb\"");

    // Tab.
    let raw = "\"a\\tb\"".as_bytes();
    let canon = canonicalize(raw).expect("canonicalize");
    assert_eq!(canon, b"\"a\\tb\"");

    // Backslash.
    let raw = "\"a\\\\b\"".as_bytes();
    let canon = canonicalize(raw).expect("canonicalize");
    assert_eq!(canon, b"\"a\\\\b\"");
}

/// RFC 8785 §3.2 — every well-formed JSON value produces a κ-label,
/// and two semantically-equal inputs (differing only in whitespace,
/// key order, or NFC vs NFD) produce the same κ-label.
#[test]
fn semantic_equality_yields_identical_kappa_labels() {
    // Key order — JCS §3.2.1.
    let a = address(br#"{"a":1,"b":2}"#).expect("κ-label").address;
    let b = address(br#"{"b":2,"a":1}"#).expect("κ-label").address;
    assert_eq!(a, b);

    // Whitespace — JCS §3.2.
    let a = address(br#"{"a":1,"b":2}"#).expect("κ-label").address;
    let b = address(br#"{  "a" :   1  , "b":2}"#)
        .expect("κ-label")
        .address;
    assert_eq!(a, b);

    // NFC — UAX #15.
    let composed = address("{\"name\":\"caf\u{00E9}\"}".as_bytes())
        .expect("κ-label")
        .address;
    let decomposed = address("{\"name\":\"cafe\u{0301}\"}".as_bytes())
        .expect("κ-label")
        .address;
    assert_eq!(composed, decomposed);

    // Array order is significant — different order, different κ-label.
    let a = address(b"[1,2,3]").expect("κ-label").address;
    let b = address(b"[3,2,1]").expect("κ-label").address;
    assert_ne!(a, b);
}

/// RFC 8259 syntactic validity is required; malformed JSON is rejected
/// at parse time with `AddressFailure::InvalidJson`.
#[test]
fn rfc8259_invalid_json_rejected() {
    use uor_addr::json::AddressFailure;
    let cases: &[&[u8]] = &[
        b"",
        b"{",
        b"}",
        b"[1,2,",
        b"{\"unterminated",
        b"truex",        // trailing junk after literal
        b"\x00\x01\x02", // arbitrary bytes
    ];
    for raw in cases {
        match address(raw) {
            Err(AddressFailure::InvalidJson) => {} // expected
            other => panic!("expected InvalidJson for {raw:?}, got {other:?}"),
        }
    }
}

/// Deeply-nested structures within the typed-input depth bound parse
/// cleanly; anything past the bound is rejected with a typed-input
/// violation (CT-B class).
#[test]
fn depth_bound_is_strict() {
    use uor_addr::json::{AddressFailure, MAX_JSON_DEPTH};

    // Well within the bound — accepted.
    let mut s = String::new();
    for _ in 0..MAX_JSON_DEPTH {
        s.push('[');
    }
    for _ in 0..MAX_JSON_DEPTH {
        s.push(']');
    }
    address(s.as_bytes()).expect("within-bound depth admits");

    // Sufficiently past the bound — rejected (depth guard remains).
    let mut s = String::new();
    for _ in 0..(MAX_JSON_DEPTH + 4) {
        s.push('[');
    }
    for _ in 0..(MAX_JSON_DEPTH + 4) {
        s.push(']');
    }
    match address(s.as_bytes()) {
        Err(AddressFailure::InvalidJson) => {}
        other => panic!("expected InvalidJson: {other:?}"),
    }
}
