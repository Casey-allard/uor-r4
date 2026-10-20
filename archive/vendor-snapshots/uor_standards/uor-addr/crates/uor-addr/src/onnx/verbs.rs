//! ONNX realization's ψ-chain content-address derivation verb. Identical
//! at the term-arena level to [`crate::ring::verbs`] — the canonical
//! k-invariants branch ψ_1 → ψ_7 → ψ_8 → ψ_9 — over `Input = OnnxCarrier`.

use crate::label::{
    AddressLabelBlake3, AddressLabelKeccak256, AddressLabelSha256, AddressLabelSha3_256,
    AddressLabelSha512,
};
use crate::onnx::value::OnnxCarrier;

addr_verbs! {
    input: OnnxCarrier<'_>,
    { shape: AddressLabelSha256, verb: address_inference },
    { shape: AddressLabelBlake3, verb: address_inference_blake3 },
    { shape: AddressLabelSha3_256, verb: address_inference_sha3_256 },
    { shape: AddressLabelKeccak256, verb: address_inference_keccak256 },
    { shape: AddressLabelSha512, verb: address_inference_sha512 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::operation::Term;

    #[test]
    fn verb_arena_is_canonical_k_invariants_branch() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena
            .iter()
            .any(|t| matches!(t, Term::PostnikovTower { .. })));
        assert!(arena
            .iter()
            .any(|t| matches!(t, Term::HomotopyGroups { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
    }
}
