//! Product/Coproduct Completion Amendment §Gap 3 validation:
//! `CartesianProductWitness` mint happy path + per-theorem failure modes.
//!
//! The Cartesian witness stores `combined_euler` and `combined_betti`
//! because CPT_3 / CPT_4 are axiomatic per §3c — mint verifies the
//! caller-supplied invariants against Künneth composition of the
//! component values. These tests construct plausible topologies (a
//! sphere × circle variant) and verify each gate cites the right IRI.

use uor_foundation::enforcement::MAX_BETTI_DIMENSION;
use uor_foundation::pipeline::kunneth_compose;
use uor_foundation::{
    CartesianProductMintInputs, CartesianProductWitness, ContentFingerprint, VerifiedMint,
};

fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

// Component topologies:
//   A: Euler = 2, Betti profile [1, 0, 1, 0, ...] — "sphere-like" (S²)
//   B: Euler = 0, Betti profile [1, 1, 0, 0, ...] — "circle-like" (S¹)
// Künneth-composed product has Betti [1, 1, 1, 1, 0, ...] and Euler 2 · 0 = 0.

const LEFT_BETTI: [u32; MAX_BETTI_DIMENSION] = [1, 0, 1, 0, 0, 0, 0, 0];
const RIGHT_BETTI: [u32; MAX_BETTI_DIMENSION] = [1, 1, 0, 0, 0, 0, 0, 0];

fn valid_inputs() -> CartesianProductMintInputs {
    // Compute expected Künneth-composed Betti at compile-parity-ish time.
    // `kunneth_compose` is `pub const fn`, so this is effectively free.
    let combined_betti = kunneth_compose(&LEFT_BETTI, &RIGHT_BETTI);
    CartesianProductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA0),
        right_fingerprint: fp(0xB0),
        left_site_budget: 3,
        right_site_budget: 2,
        left_total_site_count: 3,
        right_total_site_count: 2,
        left_euler: 2,
        right_euler: 0,
        left_betti: LEFT_BETTI,
        right_betti: RIGHT_BETTI,
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_site_budget: 5,
        combined_site_count: 5,
        // CPT_3: combined_euler = left_euler · right_euler = 2 · 0 = 0.
        combined_euler: 0,
        // CPT_4: Künneth composition.
        combined_betti,
        // CPT_5: additive entropy = 0 + ln 2.
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_fingerprint: fp(0xC0),
    }
}

#[test]
fn happy_path_mints_witness_with_invariant_snapshot() {
    let witness = CartesianProductWitness::mint_verified(valid_inputs())
        .expect("valid CPT_* invariants should mint");
    assert_eq!(witness.witt_bits(), 8);
    assert_eq!(witness.left_fingerprint(), fp(0xA0));
    assert_eq!(witness.right_fingerprint(), fp(0xB0));
    assert_eq!(witness.combined_site_budget(), 5);
    assert_eq!(witness.combined_site_count(), 5);
    assert_eq!(
        witness.combined_euler(),
        0,
        "CPT_3 multiplicative: 2 · 0 = 0"
    );
    let expected_betti: [u32; MAX_BETTI_DIMENSION] = [1, 1, 1, 1, 0, 0, 0, 0];
    assert_eq!(
        witness.combined_betti(),
        expected_betti,
        "CPT_4 Künneth: S² × S¹ has Betti profile [1, 1, 1, 1]"
    );
}

#[test]
fn cpt_1_violation_cites_op_cpt_1() {
    let mut inputs = valid_inputs();
    // CPT_1 says combined_site_budget = 3 + 2 = 5. Inject 6.
    inputs.combined_site_budget = 6;
    let err =
        CartesianProductWitness::mint_verified(inputs).expect_err("CPT_1 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/CPT_1"));
}

#[test]
fn cartesian_layout_width_violation_cites_foundation_invariant() {
    let mut inputs = valid_inputs();
    // CartesianLayoutWidth: SITE_COUNT = 3 + 2 = 5. Inject 6.
    inputs.combined_site_count = 6;
    let err = CartesianProductWitness::mint_verified(inputs)
        .expect_err("CartesianLayoutWidth violation should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/CartesianLayoutWidth")
    );
}

#[test]
fn cpt_3_multiplicative_euler_violation_cites_op_cpt_3() {
    let mut inputs = valid_inputs();
    // CPT_3: 2 · 0 = 0. Additive would give 2 + 0 = 2 — inject that as the wrong answer.
    inputs.combined_euler = 2;
    let err =
        CartesianProductWitness::mint_verified(inputs).expect_err("CPT_3 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/CPT_3"));
}

#[test]
fn cpt_4_kunneth_violation_cites_op_cpt_4() {
    let mut inputs = valid_inputs();
    // Additive (wrong) Betti: [2, 1, 1, 0, 0, 0, 0, 0]. Correct Künneth: [1, 1, 1, 1].
    inputs.combined_betti = [2, 1, 1, 0, 0, 0, 0, 0];
    let err =
        CartesianProductWitness::mint_verified(inputs).expect_err("CPT_4 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/CPT_4"));
}

#[test]
fn cpt_5_additive_entropy_violation_cites_op_cpt_5() {
    let mut inputs = valid_inputs();
    // CPT_5 additive: ln 2. Inject 2 · ln 2.
    inputs.combined_entropy_nats_bits = f64::to_bits(2.0 * core::f64::consts::LN_2);
    let err =
        CartesianProductWitness::mint_verified(inputs).expect_err("CPT_5 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/CPT_5"));
}

#[test]
fn cpt_5_non_finite_entropy_cites_op_cpt_5() {
    let mut inputs = valid_inputs();
    inputs.left_entropy_nats_bits = f64::to_bits(f64::INFINITY);
    let err =
        CartesianProductWitness::mint_verified(inputs).expect_err("infinite entropy should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/CPT_5"));
}

#[test]
fn kunneth_compose_is_commutative() {
    // Sanity: for Künneth composition, swapping A and B is the same function
    // of symmetric inputs up to the Betti profile's support. Assert this on
    // the specific pair used above.
    let ab = kunneth_compose(&LEFT_BETTI, &RIGHT_BETTI);
    let ba = kunneth_compose(&RIGHT_BETTI, &LEFT_BETTI);
    assert_eq!(ab, ba, "Künneth composition is commutative");
}
