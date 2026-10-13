//! CS-E8 ψ-chain content-address derivation verbs, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e8::value::E8Carrier;
use crate::label::{
    CompositionLabelE8Blake3, CompositionLabelE8Keccak256, CompositionLabelE8Sha256,
    CompositionLabelE8Sha3_256, CompositionLabelE8Sha512,
};

addr_verbs! {
    input: E8Carrier<'_>,
    { shape: CompositionLabelE8Sha256, verb: compose_e8_inference },
    { shape: CompositionLabelE8Blake3, verb: compose_e8_inference_blake3 },
    { shape: CompositionLabelE8Sha3_256, verb: compose_e8_inference_sha3_256 },
    { shape: CompositionLabelE8Keccak256, verb: compose_e8_inference_keccak256 },
    { shape: CompositionLabelE8Sha512, verb: compose_e8_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = compose_e8_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = compose_e8_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
