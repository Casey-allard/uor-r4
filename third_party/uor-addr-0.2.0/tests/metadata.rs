//! Raw JSON/address identity controls; no model or numerical operator execution.
use uor_addr::json::{address_blake3, canonicalize, AddressFailure};

#[test]
fn canonical_bytes_match_pinned_upstream_fixtures() {
    // The first fixture is the upstream hash_kat.rs JSON pipeline fixture.
    for (raw, canonical) in [
        (&br#"{"b":2,"a":1}"#[..], &br#"{"a":1,"b":2}"#[..]),
        (
            &br#" [true, null, false, -2, "line\n"] "#[..],
            &br#"[true,null,false,-2,"line\n"]"#[..],
        ),
        (
            &br#"{"x":"e\u0301","a":"\u0001"}"#[..],
            "{\"a\":\"\\u0001\",\"x\":\"é\"}".as_bytes(),
        ),
        // Retain the pinned implementation's Ryu f64 formatting, including 1.0.
        (&br#"{"z":-0,"b":1e0}"#[..], &br#"{"b":1.0,"z":-0.0}"#[..]),
    ] {
        assert_eq!(canonicalize(raw).expect("valid fixture"), canonical);
    }
}

#[test]
fn label_hashes_exact_canonical_bytes_and_ignores_json_surface_order() {
    let canonical = br#"{"a":1,"b":2}"#;
    // Match the upstream Blake3Hasher initial/fold_bytes/finalize path and
    // resolver label framing, independently of the facade's direct hash call.
    let mut upstream_hasher = blake3::Hasher::new();
    upstream_hasher.update(canonical);
    let expected = format!("blake3:{}", upstream_hasher.finalize().to_hex());
    let first = address_blake3(br#"{"b":2,"a":1}"#).expect("valid fixture");
    let second = address_blake3(canonical).expect("valid fixture");
    assert_eq!(first.address.as_str(), expected);
    assert_eq!(first.address.as_bytes().len(), 71);
    assert_eq!(first, second);
}

#[test]
fn invalid_json_refuses_before_label_creation() {
    for raw in [
        b"{".as_slice(),
        b"[1,]".as_slice(),
        b"true trailing".as_slice(),
        &[0xff],
    ] {
        assert_eq!(address_blake3(raw), Err(AddressFailure::InvalidJson));
    }
}
