//! `uor-addr` — CBOR realization, basic surface example.
//!
//! Demonstrates [`uor_addr::cbor::address`] (RFC 8949 §4.2 deterministic
//! encoding) and the σ-axis selection entry points
//! ([`uor_addr::cbor::address_blake3`] et al.). Each panic is a structural
//! invariant, so a clean run is a conformance assertion.
//!
//! Run with `cargo run -p uor-addr --example cbor_address`.

use uor_addr::cbor::{
    address, address_blake3, address_keccak256, address_sha3_256, canonicalize, AddressFailure,
};

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn main() {
    println!("uor-addr — CBOR realization (RFC 8949 §4.2)\n");

    // Diagnostic ⇄ encoding (RFC 8949 Appendix A):
    //   42                       -> 0x182a            (uint 42)
    //   [1, 2, 3]                -> 0x83010203
    //   {"a": 1, "b": [2, 3]}    -> 0xa26161016162820203
    //   {"b": [2,3], "a": 1}     -> non-canonical key order; canonicalizes to the above
    let cases: &[(&str, &str)] = &[
        ("uint 42", "182a"),
        ("[1, 2, 3]", "83010203"),
        ("unsorted map {\"b\":[2,3],\"a\":1}", "a26162820203616101"),
    ];

    for (label, hex) in cases {
        let raw = unhex(hex);
        match address(&raw) {
            Ok(outcome) => {
                let canon = canonicalize(&raw).expect("well-formed CBOR canonicalizes");
                println!("  input ({label}): {hex}");
                println!(
                    "  canonical:        {}",
                    canon.iter().map(|b| format!("{b:02x}")).collect::<String>()
                );
                println!("  sha256:           {}", outcome.address);
                println!(
                    "  blake3:           {}",
                    address_blake3(&raw).unwrap().address
                );
                println!(
                    "  sha3-256:         {}",
                    address_sha3_256(&raw).unwrap().address
                );
                println!(
                    "  keccak256:        {}\n",
                    address_keccak256(&raw).unwrap().address
                );
                assert!(outcome.witness.verify().is_ok(), "witness must verify");
            }
            Err(AddressFailure::InvalidCbor) => panic!("example input must be valid CBOR"),
            Err(AddressFailure::PipelineFailure) => panic!("unreachable substrate failure"),
        }
    }

    // The unsorted map canonicalizes to the sorted form, so both address to
    // the same κ-label.
    let sorted = address(&unhex("a26161016162820203")).unwrap().address;
    let unsorted = address(&unhex("a26162820203616101")).unwrap().address;
    assert_eq!(
        sorted, unsorted,
        "RFC 8949 §4.2.1 map-key sorting must collapse key order"
    );

    println!("OK — every CBOR value produced its κ-label across all four σ-axes.");
}
