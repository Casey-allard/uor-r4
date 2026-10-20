//! `uor-addr`'s ψ-chain content-address derivation verbs
//! (wiki ADR-024, ADR-035, ADR-036) — one per admissible σ-axis.
//!
//! The address-derivation inference is the **k-invariant branch** of
//! the ψ-pipeline applied to a `JsonInput`:
//!
//! ```text
//! JsonInput  (canonical-form JCS+NFC bytes)
//!    ↓ ψ_1 Nerve            (Constraints → SimplicialComplex)
//!    ↓ ψ_7 PostnikovTower   (SimplicialComplex → PostnikovTower)
//!    ↓ ψ_8 HomotopyGroups   (PostnikovTower → HomotopyGroups)
//!    ↓ ψ_9 KInvariants      (HomotopyGroups → KInvariants)
//! AddressLabel — the κ-label (`<algorithm>:<hex>`)
//! ```
//!
//! The verb body is identical across σ-axes — only the output-shape return
//! type differs (one per axis). Foundation's catamorphism evaluates the
//! chain end-to-end via the application's `ResolverTuple`
//! ([`crate::resolvers::AddressResolverTuple`]); the canonical hash axis is
//! consumed by ψ_9's resolver
//! ([`crate::resolvers::AddressKInvariantResolver`]), **never** by the verb
//! body's term composition (ADR-035 ψ-residuals discipline; the
//! `verb_arena_contains_no_sigma_residuals` test pins it).

use crate::json::value::JsonCarrier;
use crate::label::{
    AddressLabelBlake3, AddressLabelKeccak256, AddressLabelSha256, AddressLabelSha3_256,
    AddressLabelSha512,
};

addr_verbs! {
    input: JsonCarrier<'_>,
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
    fn verb_arena_contains_psi_1_nerve() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(arena.iter().any(|t| matches!(t, Term::Nerve { .. })));
    }

    #[test]
    fn verb_arena_contains_psi_7_postnikov_tower() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(arena
            .iter()
            .any(|t| matches!(t, Term::PostnikovTower { .. })));
    }

    #[test]
    fn verb_arena_contains_psi_8_homotopy_groups() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(arena
            .iter()
            .any(|t| matches!(t, Term::HomotopyGroups { .. })));
    }

    #[test]
    fn verb_arena_contains_psi_9_k_invariants() {
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        assert!(arena.iter().any(|t| matches!(t, Term::KInvariants { .. })));
    }

    #[test]
    fn verb_arena_contains_no_sigma_residuals() {
        // Wiki ADR-035 ψ-residuals discipline: no σ-enumeration in the
        // verb body. The canonical hash axis is consumed by resolvers per
        // ADR-046's discipline boundary — never by the verb body.
        let arena = address_inference_term_arena::<{ crate::ADDR_INLINE_BYTES }>();
        let has_first_admit = arena.iter().any(|t| matches!(t, Term::FirstAdmit { .. }));
        let has_axis_invocation = arena
            .iter()
            .any(|t| matches!(t, Term::AxisInvocation { .. }));
        let has_le_or_concat = arena.iter().any(|t| {
            matches!(
                t,
                Term::Application {
                    operator: prism::operation::PrimitiveOp::Le
                        | prism::operation::PrimitiveOp::Concat
                        | prism::operation::PrimitiveOp::Lt
                        | prism::operation::PrimitiveOp::Ge
                        | prism::operation::PrimitiveOp::Gt,
                    ..
                }
            )
        });
        assert!(
            !has_first_admit,
            "FirstAdmit is a σ-enumeration residual — must not appear in the pure-prism verb body"
        );
        assert!(
            !has_axis_invocation,
            "AxisInvocation belongs in resolvers, not in the verb body's composition"
        );
        assert!(
            !has_le_or_concat,
            "byte-comparison/concat ops are σ-residuals — admission is structural"
        );
    }
}
