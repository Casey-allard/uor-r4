//! **`uor_addr::variant::signed` — the signature-required-on-
//! emission cost-model variant** (ARCHITECTURE.md "Cost-model-bearing
//! variants" § `uor-addr-signed`).
//!
//! Content-addressing variant of the JSON realization that binds
//! the 5th `PrismModel` parameter to a typed-commitment expressing
//! a signature-shape admission predicate per ADR-049 + ADR-048.
//!
//! ## Concrete `C` selection
//!
//! ```text
//! C = SingletonCommitment<UltrametricCloseTo>
//! ```
//!
//! The architectural commitment per ARCHITECTURE.md is
//! `SingletonCommitment<SignatureCommitmentPredicate>` where
//! `SignatureCommitmentPredicate` is a per-κ-label signature-shape
//! `ObservablePredicate`. The foundation's `ObservablePredicate`
//! trait is sealed via `__sdk_seal::Sealed` — application code
//! cannot introduce new predicates without a foundation-level
//! extension. The closest published `ObservablePredicate` from
//! `prism::pipeline`'s roster that fits the
//! "signature-admission-shape" semantics is
//! [`UltrametricCloseTo`]: it admits κ-labels whose σ-projection's
//! digest is ultrametrically close (Hamming-prefix-aligned) to a
//! configured target — the same byte-shape property a signature
//! commitment per ADR-049's `axis::cryptanalyze` witness would
//! validate.
//!
//! When `prism::pipeline` publishes a
//! `SignatureCommitmentPredicate` primitive, this module retargets;
//! the architectural surface — the
//! `SingletonCommitment<…>` shape — does not change.
//!
//! ## Authoritative sources
//!
//! - **ADR-048 typed-commitment surface**
//!   (<https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-048>).
//! - **ADR-049 `axis::cryptanalyze` witness**
//!   (<https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-049>).

use prism::pipeline::{prism_model, SingletonCommitment, UltrametricCloseTo};
use prism::vocabulary::DefaultHostTypes;

use crate::bounds::AddrBounds;
use crate::json::value::JsonCarrier;
use crate::label::AddressLabel;
use crate::resolvers::AddressResolverTuple;
use prism::crypto::Sha256Hasher;

#[allow(unused_imports)]
use crate::json::verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// **The signature reference digest** — a 32-byte target the
/// κ-label's digest is XOR-compared against. The predicate's
/// 2-adic valuation of `(digest XOR reference)` must be ≥
/// [`SIGNATURE_PROXIMITY_K`] for the signature to admit.
pub const SIGNATURE_REFERENCE: &[u8] = &[0; 32];

/// **The signature-shape proximity threshold** — 2-adic valuation
/// ≥ k bits of `(digest XOR reference)` for admission. The shipped
/// k = 1 yields accept_prob = 1/2 (digest's LSB matches the
/// reference's LSB); larger k tightens the signature-shape
/// admission predicate per ADR-049's bandwidth-additivity.
pub const SIGNATURE_PROXIMITY_K: u32 = 1;

/// **The cost-model commitment type for the signed variant** — the
/// architectural `SingletonCommitment<…>` shape.
pub type SignedCommitment = SingletonCommitment<UltrametricCloseTo<2>>;

/// **The commitment instance** — used by the model's `fn commitment()`
/// clause.
pub const SIGNED_COMMITMENT_INSTANCE: SignedCommitment = SingletonCommitment {
    predicate: UltrametricCloseTo {
        reference: SIGNATURE_REFERENCE,
        k: SIGNATURE_PROXIMITY_K,
    },
};

prism_model! {
    pub struct AddressSignedModel;
    pub struct AddressSignedRoute;
    impl PrismModel<
        DefaultHostTypes,
        AddrBounds,
        Sha256Hasher,
        AddressResolverTuple<Sha256Hasher>,
        SignedCommitment
    > for AddressSignedModel {
        type Input = JsonCarrier<'a>;
        type Output = AddressLabel;
        type Route = AddressSignedRoute;
        fn route(input: Self::Input) -> Self::Output {
            address_inference(input)
        }
        fn commitment() -> SignedCommitment {
            SIGNED_COMMITMENT_INSTANCE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::pipeline::{ObservablePredicate, TypedCommitment};

    #[test]
    fn signed_commitment_is_typed_commitment() {
        fn assert_typed_commitment<C: TypedCommitment>() {}
        assert_typed_commitment::<SignedCommitment>();
    }

    #[test]
    fn signature_predicate_evaluates_against_digest() {
        let admitted: [u8; 32] = [0; 32];
        let predicate: UltrametricCloseTo<2> = UltrametricCloseTo {
            reference: SIGNATURE_REFERENCE,
            k: SIGNATURE_PROXIMITY_K,
        };
        assert!(predicate.evaluate(&admitted));
        // 2-adic valuation ≥ 1 means LSB must match. A digest whose
        // last byte differs in the LSB rejects.
        let mut rejected: [u8; 32] = [0; 32];
        rejected[31] = 0x01;
        assert!(!predicate.evaluate(&rejected));
    }

    #[test]
    fn signed_model_is_a_distinct_prism_model() {
        fn assert_is_prism_model<
            'a,
            M: prism::pipeline::PrismModel<
                'a,
                prism::vocabulary::DefaultHostTypes,
                crate::AddrBounds,
                crate::Sha256Hasher,
                { crate::ADDR_INLINE_BYTES },
                32,
                crate::AddressResolverTuple<crate::Sha256Hasher>,
                SignedCommitment,
                Route = AddressSignedRoute,
            >,
        >() {
        }
        assert_is_prism_model::<AddressSignedModel>();
    }

    #[test]
    fn signature_reference_is_thirty_two_bytes() {
        assert_eq!(SIGNATURE_REFERENCE.len(), 32);
    }
}
