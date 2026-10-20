//! Product/Coproduct Completion Amendment §Gap 1 + §Gap 4 validation:
//! `PartitionCoproductWitness` mint happy path + per-theorem failure modes.
//!
//! Unlike `PartitionProductWitness`, the coproduct mint primitive runs a
//! structural validator (`validate_coproduct_structure`) over a supplied
//! `ConstraintRef` array to verify ST_6 / ST_7 / ST_8 numerically. The
//! tests below assemble a canonical-layout constraint array once and
//! perturb individual numeric inputs to exercise each gate.

use uor_foundation::pipeline::ConstraintRef;
use uor_foundation::{
    ContentFingerprint, PartitionCoproductMintInputs, PartitionCoproductWitness, VerifiedMint,
};

fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

// Canonical coproduct constraint layout for A + B with:
//   left operand:  SITE_COUNT = 2, constraints at sites 0, 1
//   right operand: SITE_COUNT = 3, constraints at sites 0, 1, 2
//   tag_site = max(2, 3) = 3
//   SITE_COUNT(A + B) = 3 + 1 = 4
//
// Per §4d canonical layout:
//   [L::CONSTRAINTS] ∪ {L tag-pinner (bias = 0)}
//     ∪ [R::CONSTRAINTS] ∪ {R tag-pinner (bias = -1)}
// Total length: 2 + 1 + 3 + 1 = 7.

use uor_foundation::pipeline::AFFINE_MAX_COEFFS;
const TAG_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
    let mut a = [0i64; AFFINE_MAX_COEFFS];
    a[3] = 1;
    a
};
const TAG_COEFF_COUNT: u32 = 4;

static COPRODUCT_CONSTRAINTS: [ConstraintRef; 7] = [
    // L's constraints.
    ConstraintRef::Site { position: 0 },
    ConstraintRef::Site { position: 1 },
    // L's tag-pinner: coefficient 1 at tag_site = 3, bias 0.
    ConstraintRef::Affine {
        coefficients: TAG_COEFFS,
        coefficient_count: TAG_COEFF_COUNT,
        bias: 0,
    },
    // R's constraints (unshifted — variants share data-site space per ST_1).
    ConstraintRef::Site { position: 0 },
    ConstraintRef::Carry { site: 1 },
    ConstraintRef::Site { position: 2 },
    // R's tag-pinner: same coefficients, bias -1.
    ConstraintRef::Affine {
        coefficients: TAG_COEFFS,
        coefficient_count: TAG_COEFF_COUNT,
        bias: -1,
    },
];

const LEFT_CONSTRAINT_COUNT: usize = 3; // L's 2 constraints + L's tag-pinner

fn valid_inputs() -> PartitionCoproductMintInputs {
    PartitionCoproductMintInputs {
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
        right_entropy_nats_bits: 0_u64,
        left_betti: [1, 0, 0, 0, 0, 0, 0, 0],
        right_betti: [1, 1, 0, 0, 0, 0, 0, 0],
        // ST_1: budget = max(2, 3) = 3.
        combined_site_budget: 3,
        // CoproductLayoutWidth: max(2, 3) + 1 = 4.
        combined_site_count: 4,
        // ST_9: euler = 1 + 2 = 3.
        combined_euler: 3,
        // ST_2: ln 2 + max(0, 0) = ln 2.
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        // ST_10: betti_k = left_k + right_k per k.
        combined_betti: [2, 1, 0, 0, 0, 0, 0, 0],
        combined_fingerprint: fp(0xC0),
        combined_constraints: &COPRODUCT_CONSTRAINTS,
        left_constraint_count: LEFT_CONSTRAINT_COUNT,
        tag_site: 3,
    }
}

#[test]
fn happy_path_mints_witness_with_tag_site_index() {
    let witness = PartitionCoproductWitness::mint_verified(valid_inputs())
        .expect("valid ST_* invariants should mint");
    assert_eq!(witness.witt_bits(), 8);
    assert_eq!(witness.left_fingerprint(), fp(0xA0));
    assert_eq!(witness.right_fingerprint(), fp(0xB0));
    assert_eq!(witness.combined_site_budget(), 3);
    assert_eq!(witness.combined_site_count(), 4);
    assert_eq!(
        witness.tag_site_index(),
        3,
        "tag_site_index must equal max(SITE_COUNT(A), SITE_COUNT(B)) per §4b'"
    );
}

#[test]
fn st_1_violation_cites_op_st_1() {
    let mut inputs = valid_inputs();
    // ST_1 says combined_site_budget = max(2, 3) = 3. Inject 4.
    inputs.combined_site_budget = 4;
    let err =
        PartitionCoproductWitness::mint_verified(inputs).expect_err("ST_1 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_1"));
}

#[test]
fn coproduct_layout_width_violation_cites_foundation_invariant() {
    let mut inputs = valid_inputs();
    // CoproductLayoutWidth says combined_site_count = max(2, 3) + 1 = 4. Inject 5.
    inputs.combined_site_count = 5;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("CoproductLayoutWidth violation should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/CoproductLayoutWidth")
    );
}

#[test]
fn tag_site_misalignment_cites_foundation_invariant() {
    let mut inputs = valid_inputs();
    // tag_site must equal max(SITE_COUNT(A), SITE_COUNT(B)) = 3. Inject 2.
    inputs.tag_site = 2;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("tag-site misalignment should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/CoproductLayoutWidth")
    );
}

#[test]
fn st_2_violation_cites_op_st_2() {
    let mut inputs = valid_inputs();
    // ST_2 says combined_entropy = ln 2 + max(0, 0) = ln 2. Inject 2 × ln 2.
    inputs.combined_entropy_nats_bits = f64::to_bits(2.0 * core::f64::consts::LN_2);
    let err =
        PartitionCoproductWitness::mint_verified(inputs).expect_err("ST_2 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_2"));
}

#[test]
fn st_9_violation_cites_op_st_9() {
    let mut inputs = valid_inputs();
    // ST_9 says combined_euler = left + right = 3. Inject 4.
    inputs.combined_euler = 4;
    let err =
        PartitionCoproductWitness::mint_verified(inputs).expect_err("ST_9 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_9"));
}

#[test]
fn st_10_violation_cites_op_st_10() {
    let mut inputs = valid_inputs();
    // ST_10 says combined_betti[k] = left_k + right_k. Break index 1.
    inputs.combined_betti = [2, 99, 0, 0, 0, 0, 0, 0];
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("ST_10 violation should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_10"));
}

#[test]
fn missing_right_tag_pinner_cites_op_st_6() {
    // Build a degenerate constraint array with only L's tag-pinner (no R).
    // This violates ST_6's per-region unique-existence requirement.
    static BAD: [ConstraintRef; 5] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: 0,
        },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Carry { site: 1 },
        // (no R tag-pinner)
    ];
    let mut inputs = valid_inputs();
    inputs.combined_constraints = &BAD;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("missing R tag-pinner should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_6"));
}

#[test]
fn wrong_bias_cites_op_st_7() {
    // Build a constraint array where the LEFT tag-pinner has bias -1
    // instead of bias 0. ST_6 still passes (exactly one canonical
    // tag-pinner per region), but ST_7's variant-tagging convention is
    // broken (bias inverted).
    static BAD: [ConstraintRef; 7] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1, // wrong — should be 0 for the left variant
        },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Carry { site: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1,
        },
    ];
    let mut inputs = valid_inputs();
    inputs.combined_constraints = &BAD;
    let err =
        PartitionCoproductWitness::mint_verified(inputs).expect_err("wrong bias should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_7"));
}

#[test]
fn non_canonical_tag_encoding_cites_foundation_invariant() {
    // A tag-pinner with coefficient = 2 (instead of 1) at tag_site is
    // semantically still asserting `2 * site_3 = 0`, which has the same
    // solution set as `site_3 = 0`, but violates the canonical byte
    // pattern content-addressing depends on. Must cite
    // foundation/CoproductTagEncoding, NOT op/ST_6 — the logical tag-site
    // existence claim holds, only the encoding normalization fails.
    const NONCANON_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
        let mut a = [0i64; AFFINE_MAX_COEFFS];
        a[3] = 2;
        a
    };
    static BAD: [ConstraintRef; 7] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Affine {
            coefficients: NONCANON_COEFFS,
            coefficient_count: 4,
            bias: 0,
        },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Carry { site: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1,
        },
    ];
    let mut inputs = valid_inputs();
    inputs.combined_constraints = &BAD;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("non-canonical tag encoding should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/CoproductTagEncoding")
    );
}

#[test]
fn operand_site_at_tag_site_cites_op_st_6() {
    // An operand `Site { position: 3 }` — where 3 is the tag_site — is a
    // data-site reservation colliding with the tag. Per ST_6 the tag site
    // is distinct from every data site of either operand.
    static BAD: [ConstraintRef; 7] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 3 }, // collides with tag_site
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: 0,
        },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Carry { site: 1 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1,
        },
    ];
    let mut inputs = valid_inputs();
    inputs.combined_constraints = &BAD;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("operand site colliding with tag_site should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_6"));
}
