//! `uor-addr` — Storage cost-model variant comprehensive example.
//!
//! Demonstrates [`uor_addr::variant::storage`] — the
//! cost-model-bearing variant binding
//! `C = AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>`
//! per ADR-048 + QS-06. Shows the threshold-predicate evaluation,
//! the bandwidth measurement under ADR-047 U6, and the
//! `TypedCommitment` surface.
//!
//! Run with `cargo run -p uor-addr --example storage_variant`.

use prism::pipeline::{LexicographicLessEqThreshold, ObservablePredicate, TypedCommitment};
use uor_addr::variant::storage::{
    AddressStorageModel, AddressStorageRoute, StorageCommitment, STORAGE_COMMITMENT_INSTANCE,
    STORAGE_THRESHOLD,
};

fn main() {
    println!("uor-addr — Storage cost-model variant (ADR-048 + QS-06)\n");

    // 1. The bound C selection.
    println!("1. Cost-model binding");
    println!(
        "   C = AndCommitment<EmptyCommitment, SingletonCommitment<LexicographicLessEqThreshold>>"
    );
    println!("   threshold (32 bytes BE):");
    println!("     {:02X?}", STORAGE_THRESHOLD);
    println!();

    // 2. Predicate evaluation.
    let predicate = LexicographicLessEqThreshold {
        target: STORAGE_THRESHOLD,
    };
    let admitted: [u8; 32] = [0; 32];
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
    println!("2. Predicate evaluation (storage-admission boundary at 0x7F)");
    println!(
        "   digest all-zero   ({:02X}…): {}",
        admitted[0],
        if predicate.evaluate(&admitted) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!(
        "   digest 0x7F…      ({:02X}…): {}",
        just_under[0],
        if predicate.evaluate(&just_under) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!(
        "   digest 0x80…      ({:02X}…): {}",
        just_over[0],
        if predicate.evaluate(&just_over) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!(
        "   digest all-ones   ({:02X}…): {}",
        rejected[0],
        if predicate.evaluate(&rejected) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!();

    // 3. Bandwidth measurement per ADR-048 + ADR-047 U6.
    let bandwidth = STORAGE_COMMITMENT_INSTANCE.bandwidth_bits();
    println!("3. Bandwidth (ADR-048 + ADR-047 U6 bandwidth-additivity)");
    println!("   bandwidth_bits(AndCommitment<E, P>) = bandwidth_bits(E) + bandwidth_bits(P)");
    println!("   = 0 + 1 = {bandwidth:.6} bits (50% accept_prob → 1 bit)");
    println!();

    // 4. TypedCommitment trait conformance at compile time.
    fn assert_typed_commitment<C: TypedCommitment>() {}
    assert_typed_commitment::<StorageCommitment>();
    println!("4. Compile-time conformance");
    println!("   StorageCommitment: TypedCommitment ✓");

    // 5. Model conformance at compile time.
    fn assert_prism_model<
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
    assert_prism_model::<AddressStorageModel>();
    println!("   AddressStorageModel: PrismModel<…, StorageCommitment> ✓");

    println!("\nOK — Storage variant ships a non-default C per ADR-048.");
}
