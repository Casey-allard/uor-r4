//! CS-E6 ψ-chain content-address derivation verbs, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e6::value::E6Carrier;
use crate::label::{
    CompositionLabelE6Blake3, CompositionLabelE6Keccak256, CompositionLabelE6Sha256,
    CompositionLabelE6Sha3_256, CompositionLabelE6Sha512,
};

addr_verbs! {
    input: E6Carrier<'_>,
    { shape: CompositionLabelE6Sha256, verb: compose_e6_inference },
    { shape: CompositionLabelE6Blake3, verb: compose_e6_inference_blake3 },
    { shape: CompositionLabelE6Sha3_256, verb: compose_e6_inference_sha3_256 },
    { shape: CompositionLabelE6Keccak256, verb: compose_e6_inference_keccak256 },
    { shape: CompositionLabelE6Sha512, verb: compose_e6_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = compose_e6_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = compose_e6_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
