//! Tests for the teacher-floor recorded-probability clamp (issue #231).
//!
//! The legacy floor reads corpus-recorded integer-percent weights; a top-3
//! token with weight 0 (renormalized p < 0.5%) hit by the sampled `next`
//! previously produced `-ln(0)` = +inf, poisoning the whole floor sum.

#![cfg(not(target_arch = "wasm32"))]

use uor_r4_graph_certify::certify::recorded_next_prob;

fn toks(a: u32, b: u32, c: u32) -> [u32; 8] {
    [a, b, c, 0, 0, 0, 0, 0]
}

fn weights(a: u32, b: u32, c: u32) -> [u32; 8] {
    [a, b, c, 0, 0, 0, 0, 0]
}

#[test]
fn top0_hit_uses_recorded_weight() {
    let (p, clamped) = recorded_next_prob(&toks(7, 8, 9), &weights(63, 30, 7), 7);
    assert_eq!(p, 0.63);
    assert!(!clamped);
}

#[test]
fn top2_hit_with_zero_weight_clamps_not_inf() {
    let (p, clamped) = recorded_next_prob(&toks(7, 8, 9), &weights(70, 30, 0), 9);
    assert_eq!(p, 0.005);
    assert!(clamped);
    let bits = -p.ln() / std::f64::consts::LN_2;
    assert!(
        bits.is_finite(),
        "clamped probability must give finite bits"
    );
}

#[test]
fn outside_top3_uses_fallback() {
    let (p, clamped) = recorded_next_prob(&toks(7, 8, 9), &weights(70, 20, 10), 42);
    assert_eq!(p, 0.01);
    assert!(!clamped);
}

#[test]
fn zero_weight_only_flags_actual_hits() {
    // A zero weight in the table does not flag when `next` hits a nonzero slot.
    let (p, clamped) = recorded_next_prob(&toks(7, 8, 9), &weights(99, 1, 0), 8);
    assert_eq!(p, 0.01); // weight 1 → 0.01 via the recorded path, not the fallback
    assert!(!clamped);
}

#[test]
fn floor_sum_stays_finite_across_mixed_positions() {
    // Property: any mix of recorded weights (including zero-collisions)
    // yields a finite floor.
    let cases = [
        (toks(1, 2, 3), weights(50, 0, 0), 2u32),
        (toks(1, 2, 3), weights(100, 0, 0), 1),
        (toks(1, 2, 3), weights(34, 33, 33), 3),
        (toks(1, 2, 3), weights(1, 0, 0), 9), // outside top-3
    ];
    let mut floor = 0f64;
    for (t, w, next) in cases {
        let (p, _) = recorded_next_prob(&t, &w, next);
        floor += -p.ln() / std::f64::consts::LN_2;
    }
    assert!(floor.is_finite());
}
