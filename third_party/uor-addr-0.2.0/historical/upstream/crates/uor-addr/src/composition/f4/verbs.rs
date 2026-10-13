//! CS-F4 ψ-chain content-address derivation verbs, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::f4::value::F4Carrier;
use crate::label::{
    CompositionLabelF4Blake3, CompositionLabelF4Keccak256, CompositionLabelF4Sha256,
    CompositionLabelF4Sha3_256, CompositionLabelF4Sha512,
};

addr_verbs! {
    input: F4Carrier<'_>,
    { shape: CompositionLabelF4Sha256, verb: compose_f4_inference },
    { shape: CompositionLabelF4Blake3, verb: compose_f4_inference_blake3 },
    { shape: CompositionLabelF4Sha3_256, verb: compose_f4_inference_sha3_256 },
    { shape: CompositionLabelF4Keccak256, verb: compose_f4_inference_keccak256 },
    { shape: CompositionLabelF4Sha512, verb: compose_f4_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = compose_f4_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = compose_f4_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
