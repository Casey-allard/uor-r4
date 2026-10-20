//! **CL-CBOR — closed-loop conformance for the CBOR realization against
//! RFC 8949 §4.2 Deterministic Encoding.**
//!
//! Vectors are imported from the authoritative source: RFC 8949 Appendix A
//! ("Examples of Encoded CBOR Data Items") supplies the canonical
//! encodings; the non-canonical → canonical transformations exercise the
//! §4.2 rules (preferred integer/float encoding, definite lengths, sorted
//! map keys). Every canonical-form constant below was reproduced
//! byte-for-byte by the reference encoder (`cbor2`, `canonical=True`) and
//! matches RFC 8949 Appendix A.

#![cfg(feature = "alloc")]

use prism::crypto::{Blake3Hasher, Keccak256Hasher, Sha256Hasher, Sha3_256Hasher, Sha512Hasher};
use prism::vocabulary::Hasher;
use uor_addr::cbor;
use uor_addr::hash::AddrHash;

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn canon(hex: &str) -> Vec<u8> {
    cbor::canonicalize(&unhex(hex)).expect("well-formed cbor")
}

/// RFC 8949 Appendix A canonical encodings — `canonicalize` must be the
/// identity on each (idempotence ⇒ the encoder reproduces the shortest
/// integer/float form, definite lengths, and sorted keys).
const APPENDIX_A_CANONICAL: &[&str] = &[
    "00",
    "01",
    "0a",
    "17",
    "1818",
    "1819",
    "1864",
    "1903e8",
    "1a000f4240", // unsigned ints
    "20",
    "3863",
    "3903e7", // negative ints
    "f4",
    "f5",
    "f6", // false, true, null
    "40",
    "4401020304", // byte strings
    "60",
    "6161",
    "6449455446",
    "62225c", // text strings
    "80",
    "83010203",
    "8301820203820405", // arrays
    "a0",
    "a201020304",
    "a26161016162820203",                         // maps
    "826161a161626163",                           // ["a", {"b": "c"}]
    "a56161614161626142616361436164614461656145", // 5-entry map (sorted single-char keys)
    "f90000",
    "f98000",
    "f93c00",
    "f93e00",
    "f97bff",
    "fa47c35000",         // floats (half/single)
    "fb3ff199999999999a", // 1.1 (double — not representable shorter)
    "f97c00",
    "f97e00", // Infinity, NaN
];

#[test]
fn appendix_a_canonical_encodings_are_idempotent() {
    for &hex in APPENDIX_A_CANONICAL {
        let bytes = unhex(hex);
        assert_eq!(
            cbor::canonicalize(&bytes).expect("well-formed"),
            bytes,
            "canonicalize must be the identity on RFC 8949 canonical encoding {hex}"
        );
    }
}

#[test]
fn preferred_integer_encoding_shortens_non_minimal_heads() {
    // 24 encoded with a 1-byte argument (0x18 0x18) is canonical; 23 encoded
    // with a 1-byte argument (0x18 0x17) is NOT — 23 must be inline (0x17).
    assert_eq!(canon("1817"), unhex("17"));
    // 1000000 as an 8-byte argument shortens to its 4-byte form.
    assert_eq!(canon("1b00000000000f4240"), unhex("1a000f4240"));
}

#[test]
fn indefinite_lengths_fold_to_definite() {
    // (_ 1, 2, 3) → [1, 2, 3]
    assert_eq!(canon("9f010203ff"), unhex("83010203"));
    // indefinite map {_ 1:2, 3:4} → {1:2, 3:4}
    assert_eq!(canon("bf01020304ff"), unhex("a201020304"));
    // indefinite byte string (_ h'01', h'02') → h'0102'
    assert_eq!(canon("5f41014102ff"), unhex("420102"));
    // indefinite text string (_ "a", "b") → "ab"
    assert_eq!(canon("7f61616162ff"), unhex("626162"));
}

#[test]
fn map_keys_sort_bytewise_lexicographically() {
    // {"b": 1, "a": 2}  (a2 6162 01 6161 02)  →  {"a": 2, "b": 1}
    assert_eq!(canon("a2616201616102"), unhex("a2616102616201"));
    // integer keys {2: 0, 1: 0}  (a2 02 00 01 00)  →  {1: 0, 2: 0}
    assert_eq!(canon("a202000100"), unhex("a201000200"));
}

#[test]
fn floats_shorten_to_the_smallest_exact_representation() {
    // double 1.5 (fb 3ff8000000000000) → half (f9 3e00)
    assert_eq!(canon("fb3ff8000000000000"), unhex("f93e00"));
    // single 1.0 (fa 3f800000) → half (f9 3c00)
    assert_eq!(canon("fa3f800000"), unhex("f93c00"));
    // every NaN collapses to the canonical half quiet-NaN.
    assert_eq!(canon("fb7ff8000000000000"), unhex("f97e00")); // double NaN
    assert_eq!(canon("fa7fc00000"), unhex("f97e00")); // single NaN
}

#[test]
fn rejects_malformed_input() {
    // trailing bytes after a complete item
    assert!(cbor::canonicalize(&unhex("0000")).is_err());
    // reserved additional-info 28 (0x1c)
    assert!(cbor::canonicalize(&unhex("1c")).is_err());
    // text string claiming 1 byte that is invalid UTF-8 (0xff)
    assert!(cbor::canonicalize(&unhex("61ff")).is_err());
    // duplicate map keys {1:1, 1:2}
    assert!(cbor::canonicalize(&unhex("a201010102")).is_err());
    // truncated (head promises 4 bytes, none follow)
    assert!(cbor::canonicalize(&unhex("44")).is_err());
}

// ── Pipeline: κ-label = <prefix>:<hex(H(canonical_form))> for each axis ──

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn expect_label<const FP: usize, H: Hasher<FP> + AddrHash>(canonical: &[u8]) -> String {
    let d = H::initial().fold_bytes(canonical).finalize();
    format!(
        "{}:{}",
        <H as AddrHash>::LABEL_PREFIX,
        hex(&d[..<H as AddrHash>::OUTPUT_BYTES])
    )
}

#[test]
fn pipeline_mints_each_axis_over_canonical_form() {
    // {"b": [2, 3], "a": 1} — unsorted input; the κ-label binds the
    // RFC 8949 §4.2 canonical form (keys sorted), not the input order.
    let raw = unhex("a26162820203616101");
    let canonical = cbor::canonicalize(&raw).expect("valid cbor");
    assert_eq!(canonical, unhex("a26161016162820203")); // {"a":1,"b":[2,3]}

    assert_eq!(
        cbor::address(&raw).unwrap().address.as_str(),
        expect_label::<32, Sha256Hasher>(&canonical)
    );
    assert_eq!(
        cbor::address_blake3(&raw).unwrap().address.as_str(),
        expect_label::<32, Blake3Hasher>(&canonical)
    );
    assert_eq!(
        cbor::address_sha3_256(&raw).unwrap().address.as_str(),
        expect_label::<32, Sha3_256Hasher>(&canonical)
    );
    assert_eq!(
        cbor::address_keccak256(&raw).unwrap().address.as_str(),
        expect_label::<32, Keccak256Hasher>(&canonical)
    );
    assert_eq!(
        cbor::address_sha512(&raw).unwrap().address.as_str(),
        expect_label::<64, Sha512Hasher>(&canonical)
    );
}

#[test]
fn pipeline_axes_distinct_and_witness_verifies() {
    let raw = unhex("83010203"); // [1, 2, 3]
    let s = cbor::address(&raw).unwrap();
    let b = cbor::address_blake3(&raw).unwrap();
    assert_ne!(s.address.as_str(), b.address.as_str());
    assert!(s.address.starts_with("sha256:") && s.address.len() == 71);
    assert!(b.address.starts_with("blake3:") && b.address.len() == 71);
    assert!(s.witness.verify().is_ok());
    assert!(b.witness.verify().is_ok());
    assert!(cbor::address_keccak256(&raw)
        .unwrap()
        .witness
        .verify()
        .is_ok());
}

#[test]
fn distinct_structures_yield_distinct_labels() {
    let a = cbor::address(&unhex("83010203")).unwrap().address; // [1,2,3]
    let b = cbor::address(&unhex("83010204")).unwrap().address; // [1,2,4]
    assert_ne!(a.as_str(), b.as_str());
}
