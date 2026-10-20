//! Product/Coproduct Completion Amendment §Gap 1 validation:
//! `PartitionProductWitness` mint happy path + per-theorem failure modes.
//!
//! Each test exercises one PT_* / foundation-layout-width gate. Failures
//! are asserted to return a `GenericImpossibilityWitness` citing exactly
//! the right ontology IRI, confirming the mint primitive attributes
//! failure to the specific identity rather than silently muddling them.

use uor_foundation::{
    ContentFingerprint, PartitionProductMintInputs, PartitionProductWitness, VerifiedMint,
};

/// Helper: build a test fingerprint from a single byte. Distinct bytes
/// produce distinct fingerprints, which is sufficient for these tests
/// (no real hashing required because we construct the witnesses directly
/// without running the pipeline).
fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

fn valid_inputs() -> PartitionProductMintInputs {
    // Two leaf operands:
    //   A: site_budget=2, site_count=2, euler=1, entropy=0.0
    //   B: site_budget=3, site_count=3, euler=2, entropy=0.6931471805599453 (ln 2)
    // Combined per PT_1/ProductLayoutWidth/PT_3/PT_4 should be:
    //   site_budget=5, site_count=5, euler=3, entropy=ln 2
    PartitionProductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA0),
        right_fingerprint: fp(0xB0),
        left_site_budget: 2,
        right_site_budget: 3,
        left_total_site_count: 2,
        right_total_site_count: 3,
        left_euler: 1,
        right_euler: 2,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_site_budget: 5,
        combined_site_count: 5,
        combined_euler: 3,
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_fingerprint: fp(0xC0),
    }
}

#[test]
fn happy_path_mints_witness_and_accessors_round_trip() {
    let inputs = valid_inputs();
    let witness =
        PartitionProductWitness::mint_verified(inputs).expect("valid PT_* invariants should mint");
    assert_eq!(witness.witt_bits(), 8);
    assert_eq!(witness.left_fingerprint(), fp(0xA0));
    assert_eq!(witness.right_fingerprint(), fp(0xB0));
    assert_eq!(witness.content_fingerprint(), fp(0xC0));
    assert_eq!(witness.combined_site_budget(), 5);
    assert_eq!(witness.combined_site_count(), 5);
}

#[test]
fn pt_1_violation_cites_op_pt_1() {
    let mut inputs = valid_inputs();
    // Break PT_1: combined budget should be 5 (2 + 3), inject 6.
    inputs.combined_site_budget = 6;
    let err =
        PartitionProductWitness::mint_verified(inputs).expect_err("PT_1 violation should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/op/PT_1"),
        "PT_1 failure must cite op/PT_1 exactly, got {err:?}"
    );
}

#[test]
fn product_layout_width_violation_cites_foundation_invariant() {
    let mut inputs = valid_inputs();
    // Keep PT_1 happy (budget ok) but break layout: 2 + 3 = 5, inject 6.
    inputs.combined_site_count = 6;
    let err = PartitionProductWitness::mint_verified(inputs)
        .expect_err("layout-width violation should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/ProductLayoutWidth"),
        "ProductLayoutWidth failure must cite foundation/ProductLayoutWidth, got {err:?}"
    );
}

#[test]
fn pt_3_violation_cites_op_pt_3() {
    let mut inputs = valid_inputs();
    // Expected euler = 1 + 2 = 3; inject 4.
    inputs.combined_euler = 4;
    let err =
        PartitionProductWitness::mint_verified(inputs).expect_err("PT_3 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/PT_3"));
}

#[test]
fn pt_4_violation_on_mismatched_entropy_cites_op_pt_4() {
    let mut inputs = valid_inputs();
    // Expected entropy = 0 + ln 2; inject 2 × ln 2.
    inputs.combined_entropy_nats_bits = f64::to_bits(2.0 * core::f64::consts::LN_2);
    let err =
        PartitionProductWitness::mint_verified(inputs).expect_err("PT_4 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/PT_4"));
}

#[test]
fn pt_4_violation_on_non_finite_input_cites_op_pt_4() {
    let mut inputs = valid_inputs();
    // Non-finite operand entropy must be rejected up-front, not silently
    // propagated into the tolerance check.
    inputs.left_entropy_nats_bits = f64::to_bits(f64::NAN);
    let err =
        PartitionProductWitness::mint_verified(inputs).expect_err("NaN entropy should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/PT_4"));
}

#[test]
fn pt_4_violation_on_negative_input_cites_op_pt_4() {
    let mut inputs = valid_inputs();
    inputs.right_entropy_nats_bits = f64::to_bits(-1.0);
    let err =
        PartitionProductWitness::mint_verified(inputs).expect_err("negative entropy should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/PT_4"));
}

#[test]
fn entropy_within_tolerance_still_mints() {
    let mut inputs = valid_inputs();
    // Inject a tiny perturbation within `entropy_tolerance` scale.
    inputs.combined_entropy_nats_bits =
        f64::to_bits(f64::from_bits(inputs.combined_entropy_nats_bits) + (1e-14));
    let witness = PartitionProductWitness::mint_verified(inputs)
        .expect("tolerance-scale perturbation should not reject PT_4");
    assert_eq!(witness.combined_site_budget(), 5);
}
