//! Product/Coproduct Completion Amendment §2.3g / plan §A4 validation:
//! targeted coverage of `validate_coproduct_structure`'s edge cases.
//!
//! The validator is `pub(crate)` so it's exercised indirectly through
//! `PartitionCoproductWitness::mint_verified`. These tests complement
//! the main coproduct behavior test by stressing:
//!
//! - `ConstraintRef::Conjunction` recursion (valid + invalid conjuncts);
//! - `Carry { site }` at the tag site;
//! - multi-site `Affine` whose support reaches the tag site
//!   (case (d) in the three-way classification);
//! - semantically-equivalent but non-canonical `Affine` tag-pinner
//!   byte patterns (case (b), `foundation/CoproductTagEncoding`).
//!
//! Phase 17 — `Affine` carries a fixed-size
//! `[i64; AFFINE_MAX_COEFFS]` array + `coefficient_count`, and
//! `Conjunction.conjuncts` is a `[LeafConstraintRef; CONJUNCTION_MAX_TERMS]`
//! depth-1 array + `conjunct_count`. All literals updated accordingly.

use uor_foundation::pipeline::{
    ConstraintRef, LeafConstraintRef, AFFINE_MAX_COEFFS, CONJUNCTION_MAX_TERMS,
};
use uor_foundation::{
    ContentFingerprint, PartitionCoproductMintInputs, PartitionCoproductWitness, VerifiedMint,
};

fn fp(byte: u8) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

// All tests in this file share a (2 + 3) operand layout with tag_site = 3
// (same as behavior_partition_coproduct_witness.rs).

const TAG_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
    let mut a = [0i64; AFFINE_MAX_COEFFS];
    a[3] = 1;
    a
};
const TAG_COEFF_COUNT: u32 = 4;

fn numerics(combined_constraints: &'static [ConstraintRef]) -> PartitionCoproductMintInputs {
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
        combined_site_budget: 3,
        combined_site_count: 4,
        combined_euler: 3,
        combined_entropy_nats_bits: f64::to_bits(core::f64::consts::LN_2),
        combined_betti: [2, 1, 0, 0, 0, 0, 0, 0],
        combined_fingerprint: fp(0xC0),
        combined_constraints,
        // We'll override left_constraint_count per test since each case
        // has a different L-region size.
        left_constraint_count: 3,
        tag_site: 3,
    }
}

// --- Conjunction recursion --------------------------------------------------

const fn pad_conjuncts(
    items: &[LeafConstraintRef],
) -> ([LeafConstraintRef; CONJUNCTION_MAX_TERMS], u32) {
    let mut out = [LeafConstraintRef::Site { position: 0 }; CONJUNCTION_MAX_TERMS];
    let mut i = 0;
    while i < items.len() && i < CONJUNCTION_MAX_TERMS {
        out[i] = items[i];
        i += 1;
    }
    (out, items.len() as u32)
}

#[test]
fn conjunction_with_valid_data_site_passes() {
    // L has a Conjunction wrapping two data-site constraints. Both are
    // within `tag_site = 3`, so the recursion passes them through.
    const INNER_PAIR: [LeafConstraintRef; 2] = [
        LeafConstraintRef::Site { position: 0 },
        LeafConstraintRef::Site { position: 1 },
    ];
    const INNER_CONJ: ([LeafConstraintRef; CONJUNCTION_MAX_TERMS], u32) =
        pad_conjuncts(&INNER_PAIR);
    static COMBINED: [ConstraintRef; 6] = [
        ConstraintRef::Conjunction {
            conjuncts: INNER_CONJ.0,
            conjunct_count: INNER_CONJ.1,
        },
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
    let mut inputs = numerics(&COMBINED);
    // L region = Conjunction + L's tag-pinner = 2 entries.
    inputs.left_constraint_count = 2;
    let witness = PartitionCoproductWitness::mint_verified(inputs)
        .expect("Conjunction over valid data-site constraints should mint");
    assert_eq!(witness.tag_site_index(), 3);
}

#[test]
fn conjunction_containing_site_at_tag_site_cites_op_st_6() {
    // The outer Conjunction masks a Site at index 3 — must be detected
    // by the recursive classification and rejected as ST_6.
    const INNER_BAD: [LeafConstraintRef; 2] = [
        LeafConstraintRef::Site { position: 0 },
        LeafConstraintRef::Site { position: 3 }, // collides with tag_site
    ];
    const INNER_CONJ: ([LeafConstraintRef; CONJUNCTION_MAX_TERMS], u32) = pad_conjuncts(&INNER_BAD);
    static COMBINED: [ConstraintRef; 6] = [
        ConstraintRef::Conjunction {
            conjuncts: INNER_CONJ.0,
            conjunct_count: INNER_CONJ.1,
        },
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
    let mut inputs = numerics(&COMBINED);
    inputs.left_constraint_count = 2;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("Conjunction hiding Site at tag_site should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_6"));
}

#[test]
fn conjunction_containing_carry_at_tag_site_cites_op_st_6() {
    // Carry at the tag site is a site-bearing constraint collision.
    const INNER_BAD: [LeafConstraintRef; 1] = [LeafConstraintRef::Carry { site: 3 }];
    const INNER_CONJ: ([LeafConstraintRef; CONJUNCTION_MAX_TERMS], u32) = pad_conjuncts(&INNER_BAD);
    static COMBINED: [ConstraintRef; 6] = [
        ConstraintRef::Conjunction {
            conjuncts: INNER_CONJ.0,
            conjunct_count: INNER_CONJ.1,
        },
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
    let mut inputs = numerics(&COMBINED);
    inputs.left_constraint_count = 2;
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("Conjunction containing Carry at tag_site should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_6"));
}

// --- Non-canonical tag encoding --------------------------------------------

#[test]
fn alternate_bias_value_cites_coproduct_tag_encoding() {
    // Canonical tag-pinner with coefficient = 1 but bias = 5 (nonsense —
    // neither 0 nor -1). Semantically this says `site_3 = -5`, which
    // isn't either variant. Per the case (a) bias-value sub-classifier
    // this must cite foundation/CoproductTagEncoding rather than ST_7.
    static BAD: [ConstraintRef; 7] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 1 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: 5, // nonsense bias
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
    let inputs = numerics(&BAD);
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("non-canonical bias should reject");
    assert_eq!(
        err.identity(),
        Some("https://uor.foundation/foundation/CoproductTagEncoding")
    );
}

// --- Multi-site Affine reaching tag_site ------------------------------------

#[test]
fn multisite_affine_reaching_tag_site_cites_op_st_6() {
    // An operand Affine constraint with a nonzero coefficient at tag_site
    // AND at another index is NOT a tag-pinner candidate; per case (d) of
    // the three-way classifier, it's a data-site constraint reaching the
    // reserved tag site, which violates ST_6.
    const MULTISITE_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
        let mut a = [0i64; AFFINE_MAX_COEFFS];
        a[0] = 1;
        a[3] = 1;
        a
    };
    static BAD: [ConstraintRef; 7] = [
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Affine {
            coefficients: MULTISITE_COEFFS,
            coefficient_count: 4,
            bias: 0,
        },
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
    let inputs = numerics(&BAD);
    let err = PartitionCoproductWitness::mint_verified(inputs)
        .expect_err("multi-site Affine reaching tag_site should reject");
    assert_eq!(err.identity(), Some("https://uor.foundation/op/ST_6"));
}

// --- No-site-reference constraint variants pass through ---------------------

#[test]
fn residue_hamming_depth_satclauses_bound_pass_through() {
    // These five variants carry no site references at the validator
    // layer. A coproduct with all five as operand constraints should mint
    // cleanly (numerics wired to match ST_* / CoproductLayoutWidth).
    static CLAUSES: [&[(u32, bool)]; 1] = [&[(0, true)]];
    static COMBINED: [ConstraintRef; 9] = [
        ConstraintRef::Residue {
            modulus: 7,
            residue: 3,
        },
        ConstraintRef::Hamming { bound: 2 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: 0,
        },
        ConstraintRef::Depth { min: 0, max: 5 },
        ConstraintRef::SatClauses {
            clauses: &CLAUSES,
            num_vars: 1,
        },
        ConstraintRef::Bound {
            observable_iri: "https://example.org/obs",
            bound_shape_iri: "https://example.org/shape",
            args_repr: "{}",
        },
        ConstraintRef::Site { position: 0 },
        ConstraintRef::Site { position: 2 },
        ConstraintRef::Affine {
            coefficients: TAG_COEFFS,
            coefficient_count: TAG_COEFF_COUNT,
            bias: -1,
        },
    ];
    let mut inputs = numerics(&COMBINED);
    // L region = 3 entries (Residue, Hamming, tag-pinner).
    inputs.left_constraint_count = 3;
    let witness = PartitionCoproductWitness::mint_verified(inputs)
        .expect("non-site variants should pass the validator");
    assert_eq!(witness.tag_site_index(), 3);
}
