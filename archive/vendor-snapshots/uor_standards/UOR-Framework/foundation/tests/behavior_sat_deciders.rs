//! Behavioral contract for the public `decide_two_sat` and `decide_horn_sat`
//! functions in `uor_foundation::pipeline`.
//!
//! Clause encoding: each literal is `(variable_index, is_negated)`.
//! `is_negated == true` represents `¬x_i`; `is_negated == false` represents `x_i`.
//!
//! Deciders:
//! - `decide_two_sat(clauses, num_vars)` — Aspvall-Plass-Tarjan
//!   strongly-connected-component decider (O(n+m)). Returns `true` iff
//!   the 2-SAT formula is satisfiable.
//! - `decide_horn_sat(clauses, num_vars)` — unit-propagation decider.
//!   Returns `true` iff the Horn formula (clauses with ≤1 positive literal)
//!   is satisfiable.
//!
//! A regression where either decider returns `true` for unsat inputs or
//! `false` for sat inputs fails here.

use uor_foundation::pipeline::{decide_horn_sat, decide_two_sat};

// ─── 2-SAT ──────────────────────────────────────────────────────────────

#[test]
fn two_sat_accepts_vacuous_empty_clauses() {
    assert!(
        decide_two_sat(&[], 0),
        "empty 2-SAT instance must be satisfiable"
    );
}

#[test]
fn two_sat_accepts_single_positive_unit_clause() {
    // (x1): [(0, false)]
    let clause: &[(u32, bool)] = &[(0, false)];
    assert!(
        decide_two_sat(&[clause], 1),
        "unit clause (x1) must be satisfiable"
    );
}

#[test]
fn two_sat_accepts_single_negative_unit_clause() {
    // (¬x1): [(0, true)]
    let clause: &[(u32, bool)] = &[(0, true)];
    assert!(
        decide_two_sat(&[clause], 1),
        "unit clause (\u{00ac}x\u{2081}) must be satisfiable"
    );
}

#[test]
fn two_sat_rejects_direct_contradiction() {
    // (x1) ∧ (¬x1): [(0, false)] ∧ [(0, true)]
    let c1: &[(u32, bool)] = &[(0, false)];
    let c2: &[(u32, bool)] = &[(0, true)];
    assert!(
        !decide_two_sat(&[c1, c2], 1),
        "(x\u{2081}) \u{2227} (\u{00ac}x\u{2081}) must be unsat"
    );
}

#[test]
fn two_sat_accepts_implication_chain() {
    // (x1 ∨ x2) ∧ (¬x1 ∨ x2): sat (x2 = true).
    let c1: &[(u32, bool)] = &[(0, false), (1, false)];
    let c2: &[(u32, bool)] = &[(0, true), (1, false)];
    assert!(
        decide_two_sat(&[c1, c2], 2),
        "(x\u{2081}\u{2228}x\u{2082}) \u{2227} (\u{00ac}x\u{2081}\u{2228}x\u{2082}) must be sat"
    );
}

#[test]
fn two_sat_rejects_cyclic_contradiction() {
    // (x1 ∨ x2) ∧ (x1 ∨ ¬x2) ∧ (¬x1 ∨ x2) ∧ (¬x1 ∨ ¬x2): unsat.
    let c1: &[(u32, bool)] = &[(0, false), (1, false)];
    let c2: &[(u32, bool)] = &[(0, false), (1, true)];
    let c3: &[(u32, bool)] = &[(0, true), (1, false)];
    let c4: &[(u32, bool)] = &[(0, true), (1, true)];
    assert!(
        !decide_two_sat(&[c1, c2, c3, c4], 2),
        "4-clause tautology over 2 vars must be unsat"
    );
}

// ─── Horn-SAT ───────────────────────────────────────────────────────────

#[test]
fn horn_sat_accepts_vacuous_empty_clauses() {
    assert!(decide_horn_sat(&[], 0));
}

#[test]
fn horn_sat_accepts_single_positive_literal_clause() {
    // (x1): [(0, false)] — 1 positive literal, Horn.
    let c: &[(u32, bool)] = &[(0, false)];
    assert!(decide_horn_sat(&[c], 1));
}

#[test]
fn horn_sat_accepts_all_negative_clause() {
    // (¬x1 ∨ ¬x2 ∨ ¬x3): [(0,true),(1,true),(2,true)] — 0 positives, Horn.
    // Satisfiable by setting all to false (all ¬xi true).
    let c: &[(u32, bool)] = &[(0, true), (1, true), (2, true)];
    assert!(
        decide_horn_sat(&[c], 3),
        "all-negative Horn clause must be sat (set all vars false)"
    );
}

#[test]
fn horn_sat_accepts_implication_chain() {
    // (x1) ∧ (¬x1 ∨ x2) ∧ (¬x2 ∨ x3) — each has ≤1 positive, Horn, sat.
    let c1: &[(u32, bool)] = &[(0, false)];
    let c2: &[(u32, bool)] = &[(0, true), (1, false)];
    let c3: &[(u32, bool)] = &[(1, true), (2, false)];
    assert!(
        decide_horn_sat(&[c1, c2, c3], 3),
        "Horn implication chain must be sat"
    );
}

#[test]
fn horn_sat_rejects_forced_contradiction() {
    // (x1) ∧ (¬x1): unit propagation finds conflict, unsat.
    let c1: &[(u32, bool)] = &[(0, false)];
    let c2: &[(u32, bool)] = &[(0, true)];
    assert!(
        !decide_horn_sat(&[c1, c2], 1),
        "(x\u{2081}) \u{2227} (\u{00ac}x\u{2081}) must be unsat in Horn"
    );
}

#[test]
fn horn_sat_rejects_non_horn_multi_positive_clause() {
    // (x1 ∨ x2): 2 positives → not Horn → decider must reject (return false).
    let c: &[(u32, bool)] = &[(0, false), (1, false)];
    assert!(
        !decide_horn_sat(&[c], 2),
        "non-Horn input (2+ positives) must be rejected"
    );
}
