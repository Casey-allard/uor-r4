//! CL-GGUF byte-identity: the Rust realization's κ-label matches the
//! spec-attested label committed alongside each fixture.
//!
//! Each `.kappa-label` is produced by `tools/canonical-gguf.py` (the
//! executable form of the GGUF canonical-form specification) and
//! committed. This test reads the input fixture, computes its κ-label,
//! and asserts byte-identity against the committed label — the CF-W*
//! cross-validation invariant (see CONFORMANCE.md).

#![cfg(feature = "gguf")]

fn check(input: &[u8], expected_label: &str) {
    let outcome = uor_addr::gguf::address(input).expect("valid gguf fixture");
    assert_eq!(
        outcome.address.as_str(),
        expected_label.trim(),
        "Rust κ-label must equal the tools/canonical-gguf.py attested label"
    );
}

#[test]
fn synthetic_f32_matches_python_attestation() {
    check(
        include_bytes!("fixtures/gguf/synthetic-f32.gguf"),
        include_str!("fixtures/gguf/synthetic-f32.kappa-label"),
    );
}

#[test]
fn empty_metadata_matches_python_attestation() {
    check(
        include_bytes!("fixtures/gguf/empty-metadata.gguf"),
        include_str!("fixtures/gguf/empty-metadata.kappa-label"),
    );
}

#[test]
fn aligned_256_matches_python_attestation() {
    check(
        include_bytes!("fixtures/gguf/aligned-256.gguf"),
        include_str!("fixtures/gguf/aligned-256.kappa-label"),
    );
}
