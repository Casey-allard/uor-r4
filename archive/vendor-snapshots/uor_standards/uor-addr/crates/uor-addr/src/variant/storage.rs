//! **`uor_addr::variant::storage` — the cost-model-bearing variant**
//! (ARCHITECTURE.md "Cost-model-bearing variants" § `uor-addr-storage`).
//!
//! Content-addressed-storage realization of UOR-ADDR, binding the
//! 5th model parameter to a non-default `C: TypedCommitment` per
//! ADR-048. The κ-derivation surface is **identical** to
//! [`crate::json`]'s; the only difference is the `C` selection,
//! which expresses the storage tier's typed-bandwidth admission of
//! κ-labels as a typed `ObservablePredicate` conjunction per
//! ADR-047 U6's bandwidth-additivity.
//!
//! ## Concrete `C` selection
//!
//! ```text
//! C = AndCommitment<
//!         EmptyCommitment,
//!         SingletonCommitment<LexicographicLessEqThreshold>,
//!     >
//! ```
//!
//! The architectural commitment per ARCHITECTURE.md is
//! `AndCommitment<EmptyCommitment, PayloadCommitment<K>>` where
//! `PayloadCommitment<K>` is a K-bit typed-bandwidth admission
//! predicate. `prism::pipeline` does not currently publish a
//! `PayloadCommitment<K>` primitive; the closest standing
//! `ObservablePredicate` from the foundation's published roster is
//! [`LexicographicLessEqThreshold`] (a big-endian unsigned
//! threshold comparison; admits κ-labels whose σ-projection's
//! digest is lexicographically ≤ target). This module binds the
//! published primitive directly so the variant is **fully working
//! against the foundation's published surface**; the storage
//! variant's `accept_prob` is determined by the threshold (K-bit
//! payload admission is `2^-K`; the shipped 50% threshold is the
//! corresponding K = 1 case).
//!
//! When `PayloadCommitment<K>` lands in `prism::pipeline`, this
//! module will be retargeted at it; the architectural surface — the
//! `AndCommitment<EmptyCommitment, …>` shape — does not change.
//!
//! ## Threshold selection
//!
//! [`STORAGE_THRESHOLD`] is a 32-byte big-endian target that
//! admits approximately 50% of κ-labels (digest's high byte ≤
//! `0x7F`). This is the minimum non-trivial threshold demonstrating
//! the architectural surface admits non-default `C` selections.
//!
//! ## Wiki commitments
//!
//! - **ADR-048 `C: TypedCommitment`** — the 5th `PrismModel`
//!   parameter binds the cost-model commitment; the model
//!   declaration in [`AddressStorageModel`] supplies a non-default
//!   selection.
//! - **ADR-047 U6 bandwidth-additivity** — `AndCommitment<A, B>`
//!   composes via bandwidth-bits addition per the Hardening
//!   Principle's U6 axiom.
//! - **QS-06 storage-tier admission** — the κ-label's admission to
//!   the storage tier is determined by the typed predicate; the
//!   surface model demonstrates the QS-06 exemplar shape.

use prism::pipeline::{
    prism_model, AndCommitment, EmptyCommitment, LexicographicLessEqThreshold, SingletonCommitment,
};
use prism::vocabulary::DefaultHostTypes;

use crate::bounds::AddrBounds;
use crate::json::value::JsonCarrier;
use crate::label::AddressLabel;
use crate::resolvers::AddressResolverTuple;
use prism::crypto::Sha256Hasher;

#[allow(unused_imports)]
use crate::json::verbs::{address_inference, VERB_TERMS_ADDRESS_INFERENCE};

/// **The storage-admission threshold** — a 32-byte big-endian target
/// admitting approximately 50% of κ-labels. The predicate accepts iff
/// the digest's big-endian unsigned integer value is ≤ this target.
///
/// Approximate accept probability: `(0x7F + 1) / 256 = 0.5` (the
/// digest's high byte ≤ `0x7F`).
pub const STORAGE_THRESHOLD: &[u8] = &[
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];

/// **The cost-model commitment type for the storage variant** —
/// the architectural `AndCommitment<EmptyCommitment, …>` shape.
pub type StorageCommitment =
    AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>;

/// **The storage commitment instance** — used by the model's
/// `fn commitment()` clause to supply the runtime instance the
/// catamorphism's `CommitmentEvaluated` trace event consults.
pub const STORAGE_COMMITMENT_INSTANCE: StorageCommitment = AndCommitment {
    left: EmptyCommitment,
    right: SingletonCommitment {
        predicate: LexicographicLessEqThreshold {
            target: STORAGE_THRESHOLD,
        },
    },
};

prism_model! {
    pub struct AddressStorageModel;
    pub struct AddressStorageRoute;
    impl PrismModel<
        DefaultHostTypes,
        AddrBounds,
        Sha256Hasher,
        AddressResolverTuple<Sha256Hasher>,
        StorageCommitment
    > for AddressStorageModel {
        type Input = JsonCarrier<'a>;
        type Output = AddressLabel;
        type Route = AddressStorageRoute;
        fn route(input: Self::Input) -> Self::Output {
            address_inference(input)
        }
        fn commitment() -> StorageCommitment {
            STORAGE_COMMITMENT_INSTANCE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism::pipeline::{ObservablePredicate, TypedCommitment};

    #[test]
    fn storage_commitment_evaluates_predicate_against_digest() {
        // STORAGE_THRESHOLD admits digests whose high byte ≤ 0x7F.
        let admitted: [u8; 32] = [0; 32];
        let rejected: [u8; 32] = [0xFF; 32];
        let predicate = LexicographicLessEqThreshold {
            target: STORAGE_THRESHOLD,
        };
        assert!(predicate.evaluate(&admitted));
        assert!(!predicate.evaluate(&rejected));
    }

    #[test]
    fn storage_commitment_carries_nontrivial_bandwidth() {
        // ADR-048: bandwidth_bits = -log2(accept_prob). For the 50%
        // threshold the accept_prob ≈ 0.5, so bandwidth ≈ 1 bit.
        let bandwidth = STORAGE_COMMITMENT_INSTANCE.bandwidth_bits();
        assert!(
            (0.5..=1.5).contains(&bandwidth),
            "1-bit bandwidth ± rounding ({bandwidth})"
        );
    }

    #[test]
    fn storage_commitment_is_typed_commitment() {
        fn assert_is_typed_commitment<C: TypedCommitment>() {}
        assert_is_typed_commitment::<StorageCommitment>();
    }
}
