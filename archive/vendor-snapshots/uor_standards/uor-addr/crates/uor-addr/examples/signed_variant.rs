//! `uor-addr` — Signed cost-model variant comprehensive example.
//!
//! Demonstrates [`uor_addr::variant::signed`] — the
//! cost-model-bearing variant binding
//! `C = SingletonCommitment<UltrametricCloseTo<2>>` per ADR-048 +
//! ADR-049. Shows the ultrametric-proximity predicate behaviour,
//! the bandwidth measurement, the `TypedCommitment` surface, and
//! the architectural retargetability when
//! `prism::pipeline::SignatureCommitmentPredicate` is published.
//!
//! Run with `cargo run -p uor-addr --example signed_variant`.

use prism::pipeline::{ObservablePredicate, TypedCommitment, UltrametricCloseTo};
use uor_addr::variant::signed::{
    AddressSignedModel, AddressSignedRoute, SignedCommitment, SIGNATURE_PROXIMITY_K,
    SIGNATURE_REFERENCE, SIGNED_COMMITMENT_INSTANCE,
};

fn main() {
    println!("uor-addr — Signed cost-model variant (ADR-048 + ADR-049)\n");

    // 1. The bound C selection.
    println!("1. Cost-model binding");
    println!("   C = SingletonCommitment<UltrametricCloseTo<2>>");
    println!("   reference digest (32 bytes BE):");
    println!("     {:02X?}", SIGNATURE_REFERENCE);
    println!("   2-adic proximity threshold k = {SIGNATURE_PROXIMITY_K}");
    println!();

    // 2. Predicate evaluation — 2-adic ultrametric proximity.
    //    ν_2(d XOR reference) ≥ k iff the last k bits of d and
    //    reference agree.
    let predicate: UltrametricCloseTo<2> = UltrametricCloseTo {
        reference: SIGNATURE_REFERENCE,
        k: SIGNATURE_PROXIMITY_K,
    };
    let aligned: [u8; 32] = [0; 32];
    let mut last_bit_off: [u8; 32] = [0; 32];
    last_bit_off[31] = 0x01;
    let mut last_byte_off: [u8; 32] = [0; 32];
    last_byte_off[31] = 0xFF;
    println!("2. Predicate evaluation (2-adic ultrametric proximity, k = 1)");
    println!(
        "   all-zero digest    : {}",
        if predicate.evaluate(&aligned) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!(
        "   LSB-flipped digest : {}",
        if predicate.evaluate(&last_bit_off) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!(
        "   last-byte-FF digest: {}",
        if predicate.evaluate(&last_byte_off) {
            "ADMITTED"
        } else {
            "rejected"
        }
    );
    println!();

    // 3. Bandwidth measurement per ADR-048 + ADR-049.
    let bandwidth = SIGNED_COMMITMENT_INSTANCE.bandwidth_bits();
    println!("3. Bandwidth (ADR-048 + ADR-049)");
    println!("   bandwidth_bits = -log2(accept_prob)");
    println!("   accept_prob = 1 / 2^k = 0.5 for k = 1");
    println!("   bandwidth = {bandwidth:.6} bits");
    println!();

    // 4. TypedCommitment + PrismModel conformance at compile time.
    fn assert_typed_commitment<C: TypedCommitment>() {}
    assert_typed_commitment::<SignedCommitment>();
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
            SignedCommitment,
            Route = AddressSignedRoute,
        >,
    >() {
    }
    assert_prism_model::<AddressSignedModel>();
    println!("4. Compile-time conformance");
    println!("   SignedCommitment: TypedCommitment ✓");
    println!("   AddressSignedModel: PrismModel<…, SignedCommitment> ✓");

    println!();
    println!("Note: ARCHITECTURE.md's architectural commitment is");
    println!("  C = SingletonCommitment<SignatureCommitmentPredicate>.");
    println!("This module binds UltrametricCloseTo<2> — the closest standing");
    println!("ObservablePredicate from prism::pipeline's published roster — until");
    println!("a SignatureCommitmentPredicate primitive lands. The architectural");
    println!("surface (the SingletonCommitment<…> shape) does not change.");

    println!("\nOK — Signed variant ships a non-default C per ADR-048.");
}
