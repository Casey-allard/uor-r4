//! X.690 DER conformance suite for the ASN.1 realization.
//!
//! Pins [`uor_addr::asn1::Asn1Value`] against the DER encoding rules
//! published in ITU-T X.690 (08/2015). DER is the canonical form (§10),
//! so each constructor + the validator must reject every non-DER
//! encoding admitted under BER.
//!
//! ## Conformance reading
//!
//! - X.690 §8 — base encoding rules.
//! - X.690 §10 — distinguished-encoding (DER) restrictions.
//! - X.690 §11 — DER restrictions on specific types.
//! - X.680 §41 — character-string types.
//!
//! ## Coverage
//!
//! - §8.1.3 + §10.1 — length octets (short / long form / indefinite).
//! - §8.2 + §11.1 — Boolean.
//! - §8.3 + §10.2 — Integer (minimum-octets).
//! - §8.6 + §11.2 — Bit String (unused-bits in [0..=7], trailing-zeros).
//! - §8.7 — Octet String.
//! - §8.8 — Null.
//! - §8.9 — Sequence.
//! - §8.11 + §11.6 — Set (canonical-ordering rule).
//! - §8.19 — Object Identifier.
//! - §8.21 — UTF8String.
//! - X.680 §41.4 — PrintableString character set.
//! - X.680 §41.2 — IA5String character set.

use uor_addr::asn1::{address, canonicalize, AddressFailure, Asn1Value};

/// X.690 Annex A — Worked-example test vectors. Each tuple is
/// `(constructor expression, expected DER bytes)`.
fn x690_annex_a_vectors(
) -> alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>, alloc::vec::Vec<u8>)> {
    use alloc::vec;
    vec![
        // Boolean (§8.2.2 + §11.1)
        (
            "Boolean TRUE",
            Asn1Value::boolean(true).tagged_bytes().to_vec(),
            vec![0x01, 0x01, 0xFF],
        ),
        (
            "Boolean FALSE",
            Asn1Value::boolean(false).tagged_bytes().to_vec(),
            vec![0x01, 0x01, 0x00],
        ),
        // Integer (§8.3 + §10.2)
        (
            "Integer 0",
            Asn1Value::integer(0).tagged_bytes().to_vec(),
            vec![0x02, 0x01, 0x00],
        ),
        (
            "Integer 127",
            Asn1Value::integer(127).tagged_bytes().to_vec(),
            vec![0x02, 0x01, 0x7F],
        ),
        (
            "Integer 128",
            Asn1Value::integer(128).tagged_bytes().to_vec(),
            vec![0x02, 0x02, 0x00, 0x80],
        ),
        (
            "Integer 256",
            Asn1Value::integer(256).tagged_bytes().to_vec(),
            vec![0x02, 0x02, 0x01, 0x00],
        ),
        (
            "Integer -128",
            Asn1Value::integer(-128).tagged_bytes().to_vec(),
            vec![0x02, 0x01, 0x80],
        ),
        (
            "Integer -129",
            Asn1Value::integer(-129).tagged_bytes().to_vec(),
            vec![0x02, 0x02, 0xFF, 0x7F],
        ),
        // Null (§8.8.1)
        (
            "Null",
            Asn1Value::null().tagged_bytes().to_vec(),
            vec![0x05, 0x00],
        ),
        // OCTET STRING (§8.7)
        (
            "OctetString {0x01,0x02,0x03}",
            Asn1Value::octet_string(&[0x01, 0x02, 0x03])
                .tagged_bytes()
                .to_vec(),
            vec![0x04, 0x03, 0x01, 0x02, 0x03],
        ),
        // OBJECT IDENTIFIER (§8.19) — RSA's rsadsi OID 1.2.840.113549
        (
            "OID 1.2.840.113549",
            Asn1Value::object_identifier(&[1, 2, 840, 113549])
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            // 1*40 + 2 = 42 = 0x2A
            // 840 = 0b110_1001000 → 0x86 0x48
            // 113549 = 0x1BB8D = 0b110_1110111_0001101 → 0x86 0xF7 0x0D
            vec![0x06, 0x06, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D],
        ),
        // OID 1.3.6.1.4.1 (private enterprise number arc)
        (
            "OID 1.3.6.1.4.1",
            Asn1Value::object_identifier(&[1, 3, 6, 1, 4, 1])
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            vec![0x06, 0x05, 0x2B, 0x06, 0x01, 0x04, 0x01],
        ),
        // BIT STRING (§8.6) — all 0 bits, no unused.
        (
            "BIT STRING 0x80 (1 byte, 0 unused)",
            Asn1Value::bit_string(&[0x80], 0)
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            vec![0x03, 0x02, 0x00, 0x80],
        ),
        // BIT STRING with unused bits in last byte (X.690 §8.6.2.3).
        (
            "BIT STRING 0xC0 (1 byte, 6 unused) — '11' bits",
            Asn1Value::bit_string(&[0xC0], 6)
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            vec![0x03, 0x02, 0x06, 0xC0],
        ),
        // UTF8String (§8.21)
        (
            "UTF8String \"hello\"",
            Asn1Value::utf8_string("hello").tagged_bytes().to_vec(),
            vec![0x0C, 0x05, b'h', b'e', b'l', b'l', b'o'],
        ),
        // PrintableString (X.680 §41.4)
        (
            "PrintableString \"OK\"",
            Asn1Value::printable_string("OK")
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            vec![0x13, 0x02, b'O', b'K'],
        ),
        // IA5String (X.680 §41.2)
        (
            "IA5String \"abc\"",
            Asn1Value::ia5_string("abc")
                .unwrap()
                .tagged_bytes()
                .to_vec(),
            vec![0x16, 0x03, b'a', b'b', b'c'],
        ),
        // SEQUENCE { Integer 1, Integer 2 } (§8.9)
        (
            "SEQUENCE {1, 2}",
            Asn1Value::sequence(&[Asn1Value::integer(1), Asn1Value::integer(2)])
                .tagged_bytes()
                .to_vec(),
            vec![0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02],
        ),
        // SET — element ordering by ascending byte sequence (§11.6).
        // Caller supplies in any order; constructor sorts. We verify
        // the sorted output.
        (
            "SET {Integer 2, Integer 1} — sorts to {1, 2}",
            Asn1Value::set(&[Asn1Value::integer(2), Asn1Value::integer(1)])
                .tagged_bytes()
                .to_vec(),
            vec![0x31, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02],
        ),
    ]
}

#[test]
fn x690_annex_a_constructor_outputs_match_der() {
    for (name, actual, expected) in x690_annex_a_vectors() {
        assert_eq!(
            actual, expected,
            "{name}: constructor output does not match X.690 DER encoding"
        );
    }
}

#[test]
fn der_parser_round_trips_every_constructor_output() {
    for (name, bytes, _expected) in x690_annex_a_vectors() {
        let parsed =
            Asn1Value::parse(&bytes).unwrap_or_else(|e| panic!("{name}: parse failed: {e:?}"));
        assert_eq!(parsed.tagged_bytes(), bytes, "{name}: round-trip");
    }
}

#[test]
fn canonicalize_is_identity_on_der() {
    // X.690 §10 — DER is the canonical form; canonicalize must be
    // the identity on every valid DER input.
    for (name, bytes, _expected) in x690_annex_a_vectors() {
        let canon =
            canonicalize(&bytes).unwrap_or_else(|e| panic!("{name}: canonicalize failed: {e:?}"));
        assert_eq!(canon, bytes, "{name}: canonicalize-is-identity");
    }
}

// ─── DER canonical-form rejection rules — X.690 §10 + §11 ──────────

#[test]
fn rejects_long_form_length_under_128() {
    // §10.1 — DER requires short-form length encoding for content
    // lengths < 128.
    let err =
        Asn1Value::parse(&[0x04, 0x81, 0x05, 0, 0, 0, 0, 0]).expect_err("long-form-under-128");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_indefinite_length() {
    // §10.1 — DER forbids indefinite-length encoding (BER only).
    let err = Asn1Value::parse(&[0x30, 0x80]).expect_err("indefinite-length");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_non_minimal_integer() {
    // §8.3.2 — leading 0x00 with next-byte high-bit clear is non-canonical.
    let err = Asn1Value::parse(&[0x02, 0x02, 0x00, 0x01]).expect_err("non-minimal");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
    // §8.3.2 — leading 0xFF with next-byte high-bit set is non-canonical.
    let err = Asn1Value::parse(&[0x02, 0x02, 0xFF, 0x80]).expect_err("non-minimal-negative");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_non_canonical_boolean() {
    // §11.1 — DER Boolean content octet must be 0x00 or 0xFF.
    let err = Asn1Value::parse(&[0x01, 0x01, 0x01]).expect_err("non-canonical-bool");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
    let err = Asn1Value::parse(&[0x01, 0x01, 0x55]).expect_err("non-canonical-bool");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_bit_string_with_invalid_unused_bits() {
    // §8.6.2.3 — unused bits must be in [0, 7].
    let err = Asn1Value::parse(&[0x03, 0x02, 0x08, 0xFF]).expect_err("invalid-unused-count");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
    // §11.2.1 — trailing unused bits must be zero in DER.
    let err = Asn1Value::parse(&[0x03, 0x02, 0x04, 0x0F]).expect_err("nonzero-trailing-bits");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_non_minimal_oid_subidentifier() {
    // §8.19.2 — a sub-identifier must not have a leading 0x80
    // (would represent an unnecessary leading 0 in the base-128 form).
    let err = Asn1Value::parse(&[0x06, 0x03, 0x2A, 0x80, 0x01]).expect_err("non-minimal-oid");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_unterminated_oid_subidentifier() {
    // §8.19.2 — the last byte of every sub-identifier has its
    // continuation bit clear.
    let err = Asn1Value::parse(&[0x06, 0x02, 0x2A, 0x80]).expect_err("unterminated-oid");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_invalid_utf8_in_utf8_string() {
    // §8.21 — content must be valid UTF-8.
    let err = Asn1Value::parse(&[0x0C, 0x02, 0xFF, 0xFE]).expect_err("invalid-utf8");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_disallowed_char_in_printable_string() {
    // X.680 §41.4 — PrintableString excludes `@`, `#`, `$`, `&`, `*`, etc.
    let err =
        Asn1Value::parse(&[0x13, 0x03, b'a', b'@', b'b']).expect_err("disallowed-printable-char");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn rejects_non_ascii_in_ia5_string() {
    // X.680 §41.2 — IA5String admits only 7-bit ASCII (0..=127).
    let err = Asn1Value::parse(&[0x16, 0x02, b'a', 0x80]).expect_err("non-ascii");
    assert!(matches!(
        err.kind,
        prism::pipeline::ViolationKind::ValueCheck
    ));
}

#[test]
fn oid_constructor_rejects_invalid_arc_values() {
    // §8.19.4 — x1 must be 0..=2; if x1 ∈ {0, 1}, x2 must be 0..=39.
    assert!(Asn1Value::object_identifier(&[3, 0]).is_err()); // x1 > 2
    assert!(Asn1Value::object_identifier(&[1, 40]).is_err()); // x1=1, x2≥40
    assert!(Asn1Value::object_identifier(&[]).is_err()); // empty
    assert!(Asn1Value::object_identifier(&[1]).is_err()); // single arc
}

#[test]
fn oid_constructor_accepts_three_root_arcs() {
    // §8.19.4 — joint-iso-itu-t (x1=2) allows x2 ≥ 40 (e.g. 2.999.x).
    let oid = Asn1Value::object_identifier(&[2, 100, 3])
        .unwrap()
        .tagged_bytes()
        .to_vec();
    // 2*40 + 100 = 180 = 0x01 0x34 in base-128 → 0x81 0x34
    assert_eq!(oid, vec![0x06, 0x03, 0x81, 0x34, 0x03]);
}

#[test]
fn der_kappa_label_is_deterministic_across_constructor_paths() {
    // X.690 §10 canonical-encoding: two semantically-identical DER
    // values must produce identical κ-labels. Verify via SET ordering:
    // two callers passing children in different orders must yield the
    // same κ-label (because Set canonical-orders its children).
    let a = address(Asn1Value::set(&[Asn1Value::integer(1), Asn1Value::integer(2)]).tagged_bytes())
        .expect("κ-label")
        .address;
    let b = address(Asn1Value::set(&[Asn1Value::integer(2), Asn1Value::integer(1)]).tagged_bytes())
        .expect("κ-label")
        .address;
    assert_eq!(a, b, "SET canonical-ordering must yield identical κ-labels");
}

#[test]
fn empty_sequence_is_admissible() {
    let der = Asn1Value::sequence(&[]).tagged_bytes().to_vec();
    assert_eq!(der, vec![0x30, 0x00]);
    Asn1Value::parse(&der).expect("empty sequence is valid DER");
}

#[test]
fn deeply_nested_sequences_admit_within_bound_reject_past_it() {
    use uor_addr::asn1::MAX_ASN1_DEPTH;
    // Build a sequence nested within the bound.
    let mut value = Asn1Value::integer(0);
    for _ in 0..MAX_ASN1_DEPTH / 2 {
        value = Asn1Value::sequence(core::slice::from_ref(&value));
    }
    let bytes = value.tagged_bytes().to_vec();
    Asn1Value::parse(&bytes).expect("nested within bound");

    // Sufficiently past the bound — rejected.
    let mut value = Asn1Value::integer(0);
    for _ in 0..(MAX_ASN1_DEPTH + 4) {
        value = Asn1Value::sequence(core::slice::from_ref(&value));
    }
    let bytes = value.tagged_bytes().to_vec();
    match address(&bytes) {
        Err(AddressFailure::InvalidDer) => {}
        other => panic!("expected depth-bound rejection: {other:?}"),
    }
}

extern crate alloc;
