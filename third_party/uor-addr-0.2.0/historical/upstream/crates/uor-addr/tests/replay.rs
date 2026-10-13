//! CL-R — TC-05 replay round-trip via the address witness.
//!
//! Demonstrates the wiki TC-05 commitment: every `AddressWitness` the
//! address pipeline emits can be replayed by a downstream verifier
//! through [`AddressWitness::verify`] to recover the κ-label, **without**
//! re-invoking the canonical hash axis on the original input. The witness
//! carries the `content_fingerprint`; `verify()` re-derives the κ-label
//! from it; no `Sha256Hasher::fold_bytes` call is made on the verifier
//! side.
//!
//! This is the load-bearing demonstration of the discipline-scope
//! separation between minting (which requires the substrate hasher)
//! and verification (which requires only the witness).
//!
//! See [CONFORMANCE.md §CL](../../../CONFORMANCE.md#cl--formal-class--lean-mechanised-theorems)
//! and [ARCHITECTURE.md §5](../../../ARCHITECTURE.md#5-seal-regime).

#![allow(non_snake_case)]

use uor_addr::json::address;

/// CL-R01 — round-trip: take the `AddressWitness` an `address(b)`
/// invocation produced and call [`AddressWitness::verify`]. The recovered
/// κ-label must equal the outcome's `address` (QS-05 replay
/// equivalence — bit-identical round-trip), and the witness's
/// `kappa_label()` must agree as well.
#[test]
fn cl_r01__address_witness_round_trips_through_verifier() {
    // Mint: forward the canonical address pipeline end-to-end.
    let outcome = address(br#"{"foo":"bar"}"#).expect("valid JSON");

    // Verify: re-derive the κ-label without invoking the canonical hash
    // axis on the original input.
    let recovered = outcome.witness.verify().expect("witness must verify");

    assert_eq!(
        recovered, outcome.address,
        "CL-R01: verified κ-label must equal the minted address \
         (QS-05 replay equivalence)"
    );
    assert_eq!(outcome.witness.kappa_label(), outcome.address);
    // The witness carries a 32-byte content fingerprint.
    assert_eq!(outcome.witness.content_fingerprint().len(), 32);
}

/// CL-R02 — every reference fixture's `AddressWitness` verifies and
/// reproduces the minted κ-label. Establishes that round-trip
/// equivalence holds across the entire byte-identity baseline, not
/// just one input.
#[test]
fn cl_r02__every_reference_fixture_round_trips() {
    let fixtures: &[&[u8]] = &[
        br#"{"foo": "bar"}"#,
        b"{}",
        b"[]",
        br#"{"b": 1, "a": 2}"#,
        "{\"name\": \"Müller\"}".as_bytes(),
        "{\"city\": \"São Paulo\"}".as_bytes(),
        "{\"name\": \"caf\u{00E9}\"}".as_bytes(),
        "{\"name\": \"cafe\u{0301}\"}".as_bytes(),
        br#"{"int": 42, "bool": true, "null_val": null}"#,
        br#"{"nested": {"deep": {"value": "found"}}}"#,
        br#"["a", "b", "c"]"#,
        br#"[1, 2, 3]"#,
    ];
    for raw in fixtures {
        let outcome = address(raw).expect("valid fixture");
        let recovered = outcome.witness.verify().expect("witness must verify");
        assert_eq!(
            recovered,
            outcome.address,
            "CL-R02: round-trip κ-label mismatch for {:?}",
            std::str::from_utf8(raw).unwrap_or("(non-utf8)")
        );
    }
}
