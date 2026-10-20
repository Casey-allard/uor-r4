//! Conformance tests for [`uor_addr::variant::storage`] — the
//! cost-model-bearing variant binding
//! `C = AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>`.
//!
//! ## Authoritative source
//!
//! UOR-Framework wiki ADR-048 "Typed Commitments" defines the
//! `C: TypedCommitment` parameterization of `PrismModel`:
//! <https://github.com/UOR-Foundation/UOR-Framework/wiki/ADR-048>.
//! The `AndCommitment` / `SingletonCommitment` / `EmptyCommitment`
//! composition shapes are foundation-provided per ADR-022 D1
//! (closure-vocabulary primitives). ADR-047 U6 "Hardening
//! Principle" specifies the bandwidth-additivity property:
//! `bandwidth_bits(AndCommitment<A, B>) = bandwidth_bits(A) +
//! bandwidth_bits(B)`.

use prism::pipeline::{ObservablePredicate, TypedCommitment};
use uor_addr::variant::storage::{
    AddressStorageModel, AddressStorageRoute, StorageCommitment, STORAGE_COMMITMENT_INSTANCE,
    STORAGE_THRESHOLD,
};

#[test]
fn storage_commitment_evaluates_predicate_threshold() {
    // A digest whose high byte is ≤ 0x7F admits; otherwise rejects.
    let admitted: [u8; 32] = [0x00; 32];
    let just_under: [u8; 32] = {
        let mut a = [0u8; 32];
        a[0] = 0x7F;
        a
    };
    let just_over: [u8; 32] = {
        let mut a = [0u8; 32];
        a[0] = 0x80;
        a
    };
    let rejected: [u8; 32] = [0xFF; 32];

    let predicate = prism::pipeline::LexicographicLessEqThreshold {
        target: STORAGE_THRESHOLD,
    };
    assert!(predicate.evaluate(&admitted));
    assert!(predicate.evaluate(&just_under));
    assert!(!predicate.evaluate(&just_over));
    assert!(!predicate.evaluate(&rejected));
}

#[test]
fn storage_commitment_carries_nontrivial_bandwidth() {
    // ADR-048 bandwidth-bits: −log_2(accept_prob). The 50% threshold
    // (digest high byte ≤ 0x7F) yields accept_prob ≈ 0.5, so
    // bandwidth ≈ 1.0 bit.
    let bandwidth = STORAGE_COMMITMENT_INSTANCE.bandwidth_bits();
    assert!(
        (0.9..=1.1).contains(&bandwidth),
        "1-bit bandwidth ± rounding ({bandwidth})"
    );
}

#[test]
fn storage_commitment_is_typed_commitment_at_compile_time() {
    fn assert_typed_commitment<C: TypedCommitment>() {}
    assert_typed_commitment::<StorageCommitment>();
}

#[test]
fn storage_model_compiles_as_a_distinct_prism_model() {
    // The compile-time witness — if `AddressStorageModel` is not a
    // conforming `PrismModel<…, StorageCommitment>`, this fn does
    // not compile. The model is structurally distinct from
    // [`uor_addr::AddressModel`] (different `C` selection) at the
    // type level per ADR-048 + the typed-iso commitment of
    // ADR-001 + ADR-017.
    fn assert_is_prism_model<
        'a,
        M: prism::pipeline::PrismModel<
            'a,
            prism::vocabulary::DefaultHostTypes,
            uor_addr::AddrBounds,
            uor_addr::Sha256Hasher,
            { uor_addr::ADDR_INLINE_BYTES },
            32,
            uor_addr::AddressResolverTuple<uor_addr::Sha256Hasher>,
            StorageCommitment,
            Route = AddressStorageRoute,
        >,
    >() {
    }
    assert_is_prism_model::<AddressStorageModel>();
}

#[test]
fn storage_threshold_is_thirty_two_bytes() {
    // The threshold byte-width matches the SHA-256 σ-projection's
    // 32-byte digest so the lexicographic comparison aligns
    // big-endian without zero-padding gymnastics per
    // `LexicographicLessEqThreshold::evaluate`.
    assert_eq!(STORAGE_THRESHOLD.len(), 32);
    assert_eq!(STORAGE_THRESHOLD[0], 0x7F);
    for byte in &STORAGE_THRESHOLD[1..] {
        assert_eq!(*byte, 0xFF);
    }
}
