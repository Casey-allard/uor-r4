//! CS-G2 ψ-chain content-address derivation verbs, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::g2::value::G2Carrier;
use crate::label::{
    CompositionLabelG2Blake3, CompositionLabelG2Keccak256, CompositionLabelG2Sha256,
    CompositionLabelG2Sha3_256, CompositionLabelG2Sha512,
};

addr_verbs! {
    input: G2Carrier<'_>,
    { shape: CompositionLabelG2Sha256, verb: compose_g2_inference },
    { shape: CompositionLabelG2Blake3, verb: compose_g2_inference_blake3 },
    { shape: CompositionLabelG2Sha3_256, verb: compose_g2_inference_sha3_256 },
    { shape: CompositionLabelG2Keccak256, verb: compose_g2_inference_keccak256 },
    { shape: CompositionLabelG2Sha512, verb: compose_g2_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = compose_g2_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = compose_g2_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
