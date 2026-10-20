//! Behavioral contract for the fold-unroll threshold (wiki ADR-026 G14).
//!
//! Per ADR-026 G14, `fold_n(<n>, init, |state, idx| step)` lowers to
//! either an unrolled `Term::Application` chain (when `n <=
//! FOLD_UNROLL_THRESHOLD`) or `Term::Recurse` (otherwise). Foundation
//! fixes the threshold so two implementations compiling the same
//! closure-body emit the same Term tree.

use uor_foundation::pipeline::FOLD_UNROLL_THRESHOLD;

#[test]
fn fold_unroll_threshold_is_public_foundation_constant() {
    // Pin the foundation-fixed value (ADR-026 G14). Implementations that
    // disagree on this number would emit different Term trees for the
    // same `fold_n` invocation.
    assert_eq!(FOLD_UNROLL_THRESHOLD, 8);
}

#[test]
fn fold_unroll_threshold_admits_typical_iteration_counts() {
    // The threshold gates the unroll-vs-Term::Recurse decision; values
    // <= threshold unroll, values > threshold lower to Term::Recurse.
    // Pin that typical iteration counts (e.g., 4, 8) fall on the unroll
    // side so route declarations using `fold_n(4, ...)` and `fold_n(8, ...)`
    // emit unrolled chains.
    let typical_counts = [1usize, 2, 4, 8];
    for &n in &typical_counts {
        assert!(
            n <= FOLD_UNROLL_THRESHOLD,
            "iteration count {n} should fall under the unroll threshold {FOLD_UNROLL_THRESHOLD}",
        );
    }
}
