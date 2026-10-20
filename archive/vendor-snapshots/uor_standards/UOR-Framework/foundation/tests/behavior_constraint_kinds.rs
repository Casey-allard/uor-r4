//! Behavioral contract for the closed six-kind `ConstraintRef` set.
//!
//! Authority: target §1.5 + §4.7 — the six constraint subclasses
//! `Residue` / `Carry` / `Depth` / `Hamming` / `Site` / `Affine` form a
//! closed executable set. Workstream E (v0.2.2 closure) adds per-kind
//! satisfiability checks to `preflight_feasibility` and fills every arm
//! of `encode_constraint_to_clauses` with a canonical clause encoding.
//!
//! These tests exercise every variant: one satisfiable case (preflight
//! accepts) and one unsatisfiable case (preflight rejects).

use uor_foundation::pipeline::{preflight_feasibility, ConstraintRef};

#[test]
fn residue_satisfiable_passes_preflight() {
    let cs = &[ConstraintRef::Residue {
        modulus: 7,
        residue: 3,
    }];
    assert!(
        preflight_feasibility(cs).is_ok(),
        "residue 3 mod 7 is satisfiable (0 <= 3 < 7)"
    );
}

#[test]
fn residue_unsatisfiable_rejected_by_preflight() {
    // residue >= modulus — unsatisfiable by ontology.
    let cs = &[ConstraintRef::Residue {
        modulus: 7,
        residue: 10,
    }];
    assert!(
        preflight_feasibility(cs).is_err(),
        "residue 10 mod 7 is unsatisfiable (residue out of range)"
    );
}

#[test]
fn carry_always_passes_preflight() {
    // Carry is satisfiable for any site; ontology imposes no numeric bound.
    let cs = &[ConstraintRef::Carry { site: 0 }];
    assert!(preflight_feasibility(cs).is_ok());
}

#[test]
fn depth_satisfiable_min_le_max() {
    let cs = &[ConstraintRef::Depth { min: 2, max: 10 }];
    assert!(preflight_feasibility(cs).is_ok());
}

#[test]
fn depth_unsatisfiable_min_gt_max() {
    let cs = &[ConstraintRef::Depth { min: 10, max: 2 }];
    assert!(preflight_feasibility(cs).is_err());
}

#[test]
fn hamming_bound_within_range_passes() {
    let cs = &[ConstraintRef::Hamming { bound: 64 }];
    assert!(preflight_feasibility(cs).is_ok());
}

#[test]
fn hamming_bound_above_upper_ceiling_rejected() {
    // Workstream E: bound > 32_768 (the conservative upper bit-width)
    // fails preflight.
    let cs = &[ConstraintRef::Hamming { bound: 65_000 }];
    assert!(preflight_feasibility(cs).is_err());
}

#[test]
fn site_position_always_passes() {
    let cs = &[ConstraintRef::Site { position: 42 }];
    assert!(preflight_feasibility(cs).is_ok());
}

#[test]
fn affine_consistent_single_row_passes() {
    // Single-row affine constraint 2·x = 0 mod 2^n — consistent (x = 0).
    use uor_foundation::pipeline::AFFINE_MAX_COEFFS;
    let mut coeffs = [0i64; AFFINE_MAX_COEFFS];
    coeffs[0] = 2;
    let cs = &[ConstraintRef::Affine {
        coefficients: coeffs,
        coefficient_count: 1,
        bias: 0,
    }];
    assert!(preflight_feasibility(cs).is_ok());
}

#[test]
fn affine_zero_sum_nonzero_bias_rejected() {
    // sum(coefficients) = 0 but bias != 0 → inconsistent. Workstream E's
    // canonical single-row encoding rejects.
    use uor_foundation::pipeline::AFFINE_MAX_COEFFS;
    let mut coeffs = [0i64; AFFINE_MAX_COEFFS];
    coeffs[0] = 1;
    coeffs[1] = -1;
    let cs = &[ConstraintRef::Affine {
        coefficients: coeffs,
        coefficient_count: 2,
        bias: 5,
    }];
    assert!(preflight_feasibility(cs).is_err());
}

#[test]
fn affine_empty_coefficients_rejected() {
    use uor_foundation::pipeline::AFFINE_MAX_COEFFS;
    let cs = &[ConstraintRef::Affine {
        coefficients: [0i64; AFFINE_MAX_COEFFS],
        coefficient_count: 0,
        bias: 0,
    }];
    assert!(preflight_feasibility(cs).is_err());
}

#[test]
fn sat_clauses_with_zero_vars_and_clauses_rejected() {
    // Existing preflight behavior.
    let cs = &[ConstraintRef::SatClauses {
        clauses: &[&[(0u32, false)]],
        num_vars: 0,
    }];
    assert!(preflight_feasibility(cs).is_err());
}
