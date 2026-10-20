//! CS-E7 ψ-chain content-address derivation verbs, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e7::value::E7Carrier;
use crate::label::{
    CompositionLabelE7Blake3, CompositionLabelE7Keccak256, CompositionLabelE7Sha256,
    CompositionLabelE7Sha3_256, CompositionLabelE7Sha512,
};

addr_verbs! {
    input: E7Carrier<'_>,
    { shape: CompositionLabelE7Sha256, verb: compose_e7_inference },
    { shape: CompositionLabelE7Blake3, verb: compose_e7_inference_blake3 },
    { shape: CompositionLabelE7Sha3_256, verb: compose_e7_inference_sha3_256 },
    { shape: CompositionLabelE7Keccak256, verb: compose_e7_inference_keccak256 },
    { shape: CompositionLabelE7Sha512, verb: compose_e7_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = compose_e7_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = compose_e7_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
