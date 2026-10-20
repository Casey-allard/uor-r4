//! CL-ONNX byte-identity: the Rust realization's κ-label matches the
//! spec-attested label committed alongside each fixture.
//!
//! Each `.kappa-label` is produced by `tools/canonical-onnx.py` (the
//! executable form of the ONNX canonical-form specification) and
//! committed. This test reads the input fixture, computes its κ-label,
//! and asserts byte-identity — the CF-W* cross-validation invariant.

#![cfg(feature = "onnx")]

fn check(input: &[u8], expected_label: &str) {
    let outcome = uor_addr::onnx::address(input).expect("valid onnx fixture");
    assert_eq!(
        outcome.address.as_str(),
        expected_label.trim(),
        "Rust κ-label must equal the tools/canonical-onnx.py attested label"
    );
}

#[test]
fn synthetic_matches_python_attestation() {
    check(
        include_bytes!("fixtures/onnx/synthetic.onnx"),
        include_str!("fixtures/onnx/synthetic.kappa-label"),
    );
}

#[test]
fn synthetic_typed_matches_python_attestation() {
    // Exercises the value_info `TypeProto` field-order canonicalization
    // and the external-data tensor reference binding.
    check(
        include_bytes!("fixtures/onnx/synthetic-typed.onnx"),
        include_str!("fixtures/onnx/synthetic-typed.kappa-label"),
    );
}
