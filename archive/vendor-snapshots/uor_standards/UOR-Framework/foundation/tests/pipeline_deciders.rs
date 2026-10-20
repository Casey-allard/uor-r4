//! v0.2.1 integration tests for the 2-SAT and Horn-SAT deciders in the
//! reduction-pipeline driver.
//!
//! These tests exercise the real decision algorithms (Aspvall-Plass-Tarjan
//! for 2-SAT, unit propagation for Horn-SAT) against hand-crafted clause
//! lists and verify the documented return values.

use uor_foundation::pipeline::ConstraintRef;
use uor_foundation::pipeline::{decide_horn_sat, decide_two_sat, fragment_classify, FragmentKind};

// ── 2-SAT decider ────────────────────────────────────────────────────────

#[test]
fn two_sat_empty_is_satisfiable() {
    assert!(decide_two_sat(&[], 0));
}

#[test]
fn two_sat_single_unit_clause() {
    // (x₀) — satisfiable with x₀ = true.
    let clauses: &[&[(u32, bool)]] = &[&[(0, false)]];
    assert!(decide_two_sat(clauses, 1));
}

#[test]
fn two_sat_satisfiable_implication_chain() {
    // (x₀ ∨ x₁) ∧ (¬x₀ ∨ x₁) — satisfiable (x₁ = true).
    let clauses: &[&[(u32, bool)]] = &[&[(0, false), (1, false)], &[(0, true), (1, false)]];
    assert!(decide_two_sat(clauses, 2));
}

#[test]
fn two_sat_unsatisfiable_contradiction() {
    // (x₀) ∧ (¬x₀) — unsatisfiable: both x₀ and ¬x₀ forced.
    let clauses: &[&[(u32, bool)]] = &[&[(0, false)], &[(0, true)]];
    assert!(!decide_two_sat(clauses, 1));
}

#[test]
fn two_sat_unsatisfiable_cycle() {
    // (x₀ ∨ x₁) ∧ (x₀ ∨ ¬x₁) ∧ (¬x₀ ∨ x₁) ∧ (¬x₀ ∨ ¬x₁)
    // All 4 clauses over 2 vars — forces every assignment to fail.
    let clauses: &[&[(u32, bool)]] = &[
        &[(0, false), (1, false)],
        &[(0, false), (1, true)],
        &[(0, true), (1, false)],
        &[(0, true), (1, true)],
    ];
    assert!(!decide_two_sat(clauses, 2));
}

// ── Horn-SAT decider ─────────────────────────────────────────────────────

#[test]
fn horn_sat_empty_is_satisfiable() {
    assert!(decide_horn_sat(&[], 0));
}

#[test]
fn horn_sat_single_positive_unit() {
    // (x₀) — satisfiable with x₀ = true.
    let clauses: &[&[(u32, bool)]] = &[&[(0, false)]];
    assert!(decide_horn_sat(clauses, 1));
}

#[test]
fn horn_sat_chain_propagation() {
    // (x₀) ∧ (¬x₀ ∨ x₁) ∧ (¬x₁ ∨ x₂) — satisfiable with all true.
    let clauses: &[&[(u32, bool)]] = &[
        &[(0, false)],
        &[(0, true), (1, false)],
        &[(1, true), (2, false)],
    ];
    assert!(decide_horn_sat(clauses, 3));
}

#[test]
fn horn_sat_unsatisfiable_goal_clause() {
    // (x₀) ∧ (¬x₀) — unit propagation forces x₀ true, then contradicts.
    let clauses: &[&[(u32, bool)]] = &[&[(0, false)], &[(0, true)]];
    assert!(!decide_horn_sat(clauses, 1));
}

#[test]
fn horn_sat_non_horn_rejected() {
    // (x₀ ∨ x₁) — two positive literals; not Horn. The decider returns
    // false immediately.
    let clauses: &[&[(u32, bool)]] = &[&[(0, false), (1, false)]];
    assert!(!decide_horn_sat(clauses, 2));
}

// ── Fragment classifier ──────────────────────────────────────────────────

#[test]
fn classify_no_sat_clauses_is_residual() {
    let constraints = &[ConstraintRef::Residue {
        modulus: 256,
        residue: 255,
    }];
    assert_eq!(fragment_classify(constraints), FragmentKind::Residual);
}

#[test]
fn classify_width_two_clause_list_is_two_sat() {
    static CLAUSES: &[&[(u32, bool)]] = &[&[(0, false), (1, false)], &[(0, true), (1, true)]];
    let constraints = &[ConstraintRef::SatClauses {
        clauses: CLAUSES,
        num_vars: 2,
    }];
    assert_eq!(fragment_classify(constraints), FragmentKind::TwoSat);
}

#[test]
fn classify_three_positive_literals_not_horn_not_two_sat_is_residual() {
    static CLAUSES: &[&[(u32, bool)]] = &[&[(0, false), (1, false), (2, false)]];
    let constraints = &[ConstraintRef::SatClauses {
        clauses: CLAUSES,
        num_vars: 3,
    }];
    assert_eq!(fragment_classify(constraints), FragmentKind::Residual);
}

#[test]
fn classify_three_literals_one_positive_is_horn() {
    static CLAUSES: &[&[(u32, bool)]] = &[&[(0, true), (1, true), (2, false)]];
    let constraints = &[ConstraintRef::SatClauses {
        clauses: CLAUSES,
        num_vars: 3,
    }];
    assert_eq!(fragment_classify(constraints), FragmentKind::Horn);
}
