//! CBOR realization's ψ-chain content-address derivation verbs (wiki
//! ADR-024 + ADR-035 + ADR-036), one per admissible σ-axis.
//!
//! The verb body is **identical at the term-arena level** to every other
//! UOR-ADDR realization — the canonical k-invariants branch
//! (ψ₁ → ψ₇ → ψ₈ → ψ₉). Only the input handle type and the per-axis output
//! shape vary.

use crate::cbor::value::CborCarrier;
use crate::label::{
    AddressLabelBlake3, AddressLabelKeccak256, AddressLabelSha256, AddressLabelSha3_256,
    AddressLabelSha512,
};

addr_verbs! {
    input: CborCarrier<'_>,
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
    fn verb_term_arena_is_emitted_and_nonempty() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.is_empty());
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        // ADR-035 ψ-residuals discipline: the σ-axis is consumed by the ψ₉
        // resolver, never by the verb body's term composition.
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(!arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. })));
        assert!(!arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }
}
