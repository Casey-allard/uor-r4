//! Product/Coproduct Completion Amendment §Gap 4 validation:
//! ST_8 variant nerve disjointness — brute-force verification.
//!
//! ST_8 asserts that a PartitionCoproduct constructed via ST_6 + ST_7 +
//! §4b' produces topologically disjoint variant nerves. This test enumerates
//! every site-value assignment over the combined shape's SITE_COUNT sites
//! and confirms that no assignment satisfies both L's variant constraints
//! AND R's variant constraints simultaneously.
//!
//! Uses the same canonical coproduct constraint layout as
//! `behavior_partition_coproduct_witness.rs` and
//! `behavior_validate_coproduct_structure.rs` (2 + 3 operand, tag_site = 3).

use uor_foundation::pipeline::{ConstraintRef, AFFINE_MAX_COEFFS};

// Phase 17: Affine coefficients are now stored inline as a fixed-size
// `[i64; AFFINE_MAX_COEFFS]` array. We zero-pad the original 4-wide
// pattern out to AFFINE_MAX_COEFFS and use coefficient_count = 4.
const TAG_COEFFS: [i64; AFFINE_MAX_COEFFS] = {
    let mut a = [0i64; AFFINE_MAX_COEFFS];
    a[3] = 1;
    a
};
const TAG_COEFF_COUNT: u32 = 4;

// Canonical coproduct layout: L's constraints + L's tag-pinner (bias 0) +
// R's constraints + R's tag-pinner (bias -1). tag_site = 3.
static COPRODUCT_CONSTRAINTS: [ConstraintRef; 7] = [
    // L's constraints — two Site reservations at 0, 1.
    ConstraintRef::Site { position: 0 },
    ConstraintRef::Site { position: 1 },
    // L's tag-pinner: pins site 3 to value 0.
    ConstraintRef::Affine {
        coefficients: TAG_COEFFS,
        coefficient_count: TAG_COEFF_COUNT,
        bias: 0,
    },
    // R's constraints — Site + Carry + Site at 0, 1, 2 (sharing data-site
    // space with L per ST_1).
    ConstraintRef::Site { position: 0 },
    ConstraintRef::Carry { site: 1 },
    ConstraintRef::Site { position: 2 },
    // R's tag-pinner: pins site 3 to value 1.
    ConstraintRef::Affine {
        coefficients: TAG_COEFFS,
        coefficient_count: TAG_COEFF_COUNT,
        bias: -1,
    },
];

const LEFT_CONSTRAINT_COUNT: usize = 3; // L's 2 constraints + L's tag-pinner
const SITE_COUNT: usize = 4; // sites 0..3, tag at 3

/// Evaluate a single constraint against a site-value assignment.
///
/// Only value-pinning constraints (`Affine`) actually filter assignments;
/// `Site` / `Carry` are structural reservations without value semantics,
/// so they are trivially satisfied by any assignment. Other variants
/// don't appear in this fixture's constraint set; if they did, the
/// always-true fallback would be conservative (over-admitting, never
/// under-admitting — so any counterexample the test exposes is genuine).
fn satisfies(c: &ConstraintRef, assignment: &[i64]) -> bool {
    match c {
        ConstraintRef::Site { .. } => true,
        ConstraintRef::Carry { .. } => true,
        ConstraintRef::Affine {
            coefficients,
            coefficient_count,
            bias,
        } => {
            let mut sum: i64 = 0;
            let count = (*coefficient_count as usize).min(AFFINE_MAX_COEFFS);
            for (i, coeff) in coefficients.iter().take(count).enumerate() {
                let site_value = assignment.get(i).copied().unwrap_or(0);
                sum += coeff * site_value;
            }
            sum + bias == 0
        }
        // No other variants in this fixture; conservative fallback.
        _ => true,
    }
}

fn all_satisfy(constraints: &[ConstraintRef], assignment: &[i64]) -> bool {
    constraints.iter().all(|c| satisfies(c, assignment))
}

#[test]
fn st_8_variant_nerves_share_no_satisfying_assignment() {
    // Partition the combined constraints into L region and R region per
    // the canonical §4b' layout.
    let left_constraints = &COPRODUCT_CONSTRAINTS[..LEFT_CONSTRAINT_COUNT];
    let right_constraints = &COPRODUCT_CONSTRAINTS[LEFT_CONSTRAINT_COUNT..];

    // Enumerate 2^SITE_COUNT = 16 binary assignments.
    let total = 1u32 << SITE_COUNT;
    let mut shared_assignments = 0;
    for bits in 0..total {
        let assignment = [
            (bits & 1) as i64,
            ((bits >> 1) & 1) as i64,
            ((bits >> 2) & 1) as i64,
            ((bits >> 3) & 1) as i64,
        ];
        if all_satisfy(left_constraints, &assignment) && all_satisfy(right_constraints, &assignment)
        {
            shared_assignments += 1;
        }
    }

    assert_eq!(
        shared_assignments, 0,
        "ST_8: canonical coproduct construction must have disjoint variant \
         nerves — no assignment should satisfy both L's and R's constraints \
         because the two tag-pinners demand incompatible values at the tag site"
    );
}

#[test]
fn each_variant_independently_has_satisfying_assignments() {
    // Sanity check: each variant region alone admits SOME assignments.
    // Guards against the disjointness test vacuously passing because one
    // variant is unsatisfiable — that would pass `intersection == 0`
    // trivially without reflecting real disjointness semantics.
    let left_constraints = &COPRODUCT_CONSTRAINTS[..LEFT_CONSTRAINT_COUNT];
    let right_constraints = &COPRODUCT_CONSTRAINTS[LEFT_CONSTRAINT_COUNT..];

    let total = 1u32 << SITE_COUNT;
    let mut left_satisfiable = 0;
    let mut right_satisfiable = 0;
    for bits in 0..total {
        let assignment = [
            (bits & 1) as i64,
            ((bits >> 1) & 1) as i64,
            ((bits >> 2) & 1) as i64,
            ((bits >> 3) & 1) as i64,
        ];
        if all_satisfy(left_constraints, &assignment) {
            left_satisfiable += 1;
        }
        if all_satisfy(right_constraints, &assignment) {
            right_satisfiable += 1;
        }
    }
    // Each variant's tag-pinner fixes one site to a specific value; the
    // other 3 sites are free (8 assignments per variant).
    assert_eq!(
        left_satisfiable, 8,
        "L region should admit 2^3 = 8 assignments (tag site fixed to 0)"
    );
    assert_eq!(
        right_satisfiable, 8,
        "R region should admit 2^3 = 8 assignments (tag site fixed to 1)"
    );
}
