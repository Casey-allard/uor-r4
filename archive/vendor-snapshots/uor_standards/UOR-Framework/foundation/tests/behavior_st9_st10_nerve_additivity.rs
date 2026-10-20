//! Product/Coproduct Completion Amendment §Gap 4 validation:
//! ST_9 (χ additivity) and ST_10 (Betti additivity) verified end-to-end
//! through the foundation's nerve primitive plus the verified-mint pipeline.
//!
//! The amendment's mint primitive `pc_primitive_partition_coproduct`
//! gates ST_9 and ST_10 by checking that caller-supplied combined Euler
//! / Betti match the additive sum of operand values. This test proves
//! the mint accepts the additive prediction: it computes operand Betti
//! via `primitive_simplicial_nerve_betti` on real shapes (CircleNerve
//! and TetrahedronBoundary, the same fixtures used in `phase_x4_betti.rs`),
//! sums them per ST_10, derives combined χ via the alternating-sum
//! formula, and constructs a `PartitionCoproductMintInputs` whose
//! combined values match the additive prediction. A successful mint
//! confirms the ST_9 / ST_10 gates accept what the amendment says they
//! should.

use uor_foundation::enforcement::{primitive_simplicial_nerve_betti, MAX_BETTI_DIMENSION};
use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef, AFFINE_MAX_COEFFS};
use uor_foundation::{
    ContentFingerprint, PartitionCoproductMintInputs, PartitionCoproductWitness, VerifiedMint,
};

/// Phase 17 helper: build an Affine coefficient buffer from a const
/// slice of `i64` values, zero-padding to `AFFINE_MAX_COEFFS`. Returns
/// `(coefficients, coefficient_count)` for inline construction.
const fn pad_coeffs(items: &[i64]) -> ([i64; AFFINE_MAX_COEFFS], u32) {
    let mut out = [0i64; AFFINE_MAX_COEFFS];
    let mut i = 0;
    while i < items.len() && i < AFFINE_MAX_COEFFS {
        out[i] = items[i];
        i += 1;
    }
    (out, items.len() as u32)
}

const CIRCLE_C0: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 1, 0, 0]);
const CIRCLE_C1: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[0, 1, 1, 0]);
const CIRCLE_C2: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 0, 1, 0]);
const TETRA_C0: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 1, 1, 1, 0, 0, 0]);
const TETRA_C1: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 1, 1, 0, 1, 0, 0]);
const TETRA_C2: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 1, 0, 1, 1, 0, 0]);
const TETRA_C3: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[1, 0, 1, 1, 1, 0, 0]);

/// Phase 1a wrapper: `primitive_simplicial_nerve_betti` now returns
/// `Result<[u32; MAX_BETTI_DIMENSION], GenericImpossibilityWitness>` and
/// fails fast on oversized inputs. Every operand shape in this test is
/// ≤ cap, so the result is always `Ok`; panic on `Err` indicates a
/// test-setup bug.
#[allow(clippy::panic)]
fn unwrap_betti<T: ConstrainedTypeShape + ?Sized>() -> [u32; MAX_BETTI_DIMENSION] {
    match primitive_simplicial_nerve_betti::<T>() {
        Ok(b) => b,
        Err(w) => panic!("test shape exceeded nerve caps: {:?}", w.identity()),
    }
}

// CircleNerve — three Affine constraints with pairwise overlap forming a
// triangle nerve. Expected: Betti [1, 1, 0, ...], χ = 0.
struct CircleNerve;
impl ConstrainedTypeShape for CircleNerve {
    const IRI: &'static str = "https://example.org/st9_st10/CircleNerve";
    const SITE_COUNT: usize = 4;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Affine {
            coefficients: CIRCLE_C0.0,
            coefficient_count: CIRCLE_C0.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: CIRCLE_C1.0,
            coefficient_count: CIRCLE_C1.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: CIRCLE_C2.0,
            coefficient_count: CIRCLE_C2.1,
            bias: 0,
        },
    ];
    const CYCLE_SIZE: u64 = 1;
}

// TetrahedronBoundary — four Affine constraints forming the 2-skeleton of
// a tetrahedron. Expected: Betti [1, 0, 1, 0, ...], χ = 2.
struct TetrahedronBoundary;
impl ConstrainedTypeShape for TetrahedronBoundary {
    const IRI: &'static str = "https://example.org/st9_st10/TetrahedronBoundary";
    const SITE_COUNT: usize = 7;
    const CONSTRAINTS: &'static [ConstraintRef] = &[
        ConstraintRef::Affine {
            coefficients: TETRA_C0.0,
            coefficient_count: TETRA_C0.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C1.0,
            coefficient_count: TETRA_C1.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C2.0,
            coefficient_count: TETRA_C2.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C3.0,
            coefficient_count: TETRA_C3.1,
            bias: 0,
        },
    ];
    const CYCLE_SIZE: u64 = 1;
}

/// Compute Euler characteristic from a Betti profile via the alternating
/// sum formula: χ = β_0 − β_1 + β_2 − β_3 + …
fn euler_from_betti(betti: &[u32; MAX_BETTI_DIMENSION]) -> i32 {
    let mut chi: i32 = 0;
    for (k, b) in betti.iter().enumerate() {
        let signed = *b as i32;
        if k % 2 == 0 {
            chi += signed;
        } else {
            chi -= signed;
        }
    }
    chi
}

/// Additive sum of two Betti profiles: per ST_10, the combined Betti of
/// a PartitionCoproduct construction equals the componentwise sum.
fn additive_sum(
    a: &[u32; MAX_BETTI_DIMENSION],
    b: &[u32; MAX_BETTI_DIMENSION],
) -> [u32; MAX_BETTI_DIMENSION] {
    let mut out = [0u32; MAX_BETTI_DIMENSION];
    for k in 0..MAX_BETTI_DIMENSION {
        out[k] = a[k] + b[k];
    }
    out
}

fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

#[test]
fn operand_betti_match_published_phase_x4_expectations() {
    // Sanity: confirm the operands' Betti profiles match what
    // phase_x4_betti.rs documents — anchors the rest of the test against
    // a known-good reference.
    let circle = unwrap_betti::<CircleNerve>();
    let tetra = unwrap_betti::<TetrahedronBoundary>();

    // CircleNerve: connected (β_0 = 1), one independent loop (β_1 = 1).
    assert_eq!(circle[0], 1);
    assert_eq!(circle[1], 1);
    for &bk in circle.iter().skip(2) {
        assert_eq!(bk, 0);
    }

    // TetrahedronBoundary: 2-sphere — connected (β_0 = 1), no 1-cycles
    // (β_1 = 0), one void (β_2 = 1).
    assert_eq!(tetra[0], 1);
    assert_eq!(tetra[1], 0);
    assert_eq!(tetra[2], 1);
    for &bk in tetra.iter().skip(3) {
        assert_eq!(bk, 0);
    }

    // Euler characteristics from the formula.
    assert_eq!(euler_from_betti(&circle), 0);
    assert_eq!(euler_from_betti(&tetra), 2);
}

#[test]
fn st_9_st_10_mint_accepts_additive_prediction_end_to_end() {
    // Compute operand invariants from real primitives (no hardcoded values).
    let left_betti = unwrap_betti::<CircleNerve>();
    let right_betti = unwrap_betti::<TetrahedronBoundary>();
    let left_euler = euler_from_betti(&left_betti);
    let right_euler = euler_from_betti(&right_betti);

    // Apply ST_9 / ST_10 to derive the combined invariants the mint
    // primitive expects.
    let combined_betti = additive_sum(&left_betti, &right_betti);
    let combined_euler = left_euler + right_euler;

    // Construct the canonical coproduct constraint array per amendment
    // §4d, with tag_site = max(SITE_COUNT(A), SITE_COUNT(B)) = 7. The
    // tag-pinner coefficient slice is length 8 (tag_site + 1) with a
    // single 1 at position 7.
    const TAG_COEFFS_8: ([i64; AFFINE_MAX_COEFFS], u32) = pad_coeffs(&[0, 0, 0, 0, 0, 0, 0, 1]);
    static COMBINED_CONSTRAINTS: [ConstraintRef; 9] = [
        // CircleNerve's 3 Affine constraints (sites 0..3, no overlap with
        // tag site at 7).
        ConstraintRef::Affine {
            coefficients: CIRCLE_C0.0,
            coefficient_count: CIRCLE_C0.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: CIRCLE_C1.0,
            coefficient_count: CIRCLE_C1.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: CIRCLE_C2.0,
            coefficient_count: CIRCLE_C2.1,
            bias: 0,
        },
        // L's tag-pinner.
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS_8.0,
            coefficient_count: TAG_COEFFS_8.1,
            bias: 0,
        },
        // TetrahedronBoundary's 4 Affine constraints (sites 0..6, no
        // overlap with tag site at 7).
        ConstraintRef::Affine {
            coefficients: TETRA_C0.0,
            coefficient_count: TETRA_C0.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C1.0,
            coefficient_count: TETRA_C1.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C2.0,
            coefficient_count: TETRA_C2.1,
            bias: 0,
        },
        ConstraintRef::Affine {
            coefficients: TETRA_C3.0,
            coefficient_count: TETRA_C3.1,
            bias: 0,
        },
        // R's tag-pinner.
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS_8.0,
            coefficient_count: TAG_COEFFS_8.1,
            bias: -1,
        },
    ];

    let inputs = PartitionCoproductMintInputs {
        witt_bits: 8,
        left_fingerprint: fp(0xA0),
        right_fingerprint: fp(0xB0),
        // Use SITE_COUNT as the budget here since both leaf operands have
        // no inherited bookkeeping (SITE_BUDGET defaults to SITE_COUNT).
        left_site_budget: <CircleNerve as ConstrainedTypeShape>::SITE_COUNT as u16,
        right_site_budget: <TetrahedronBoundary as ConstrainedTypeShape>::SITE_COUNT as u16,
        left_total_site_count: <CircleNerve as ConstrainedTypeShape>::SITE_COUNT as u16,
        right_total_site_count: <TetrahedronBoundary as ConstrainedTypeShape>::SITE_COUNT as u16,
        left_euler,
        right_euler,
        // Operand entropies: 0 for both leaf shapes (no residual sites
        // by hypothesis — the test fixtures don't model entropy).
        left_entropy_nats_bits: 0_u64,
        right_entropy_nats_bits: 0_u64,
        left_betti,
        right_betti,
        // ST_1: budget = max(4, 7) = 7.
        combined_site_budget: 7,
        // CoproductLayoutWidth: max(4, 7) + 1 = 8.
        combined_site_count: 8,
        // ST_9: combined_euler = left + right (the amendment's prediction).
        combined_euler,
        // ST_2: combined_entropy = ln 2 + max(0, 0) = ln 2.
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        // ST_10: combined_betti = additive sum (the amendment's prediction).
        combined_betti,
        combined_fingerprint: fp(0xC0),
        combined_constraints: &COMBINED_CONSTRAINTS,
        // L region size: L's 3 constraints + L's tag-pinner = 4.
        left_constraint_count: 4,
        tag_site: 7,
    };

    let witness = PartitionCoproductWitness::mint_verified(inputs).expect(
        "ST_9 (χ additivity) and ST_10 (Betti additivity) — the mint primitive \
         must accept the additive prediction derived from the foundation's own \
         nerve primitives applied to CircleNerve and TetrahedronBoundary",
    );

    // Anchor: combined witness records the §4b' tag site index.
    assert_eq!(witness.tag_site_index(), 7);
    assert_eq!(witness.combined_site_budget(), 7);
    assert_eq!(witness.combined_site_count(), 8);
}
