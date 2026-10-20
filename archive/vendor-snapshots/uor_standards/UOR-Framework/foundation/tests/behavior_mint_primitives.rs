//! Behavioral contract for the four ADR-016 UOR-domain mint primitives.
//!
//! Per the UOR-Framework wiki ADR-016, foundation exposes a set of
//! `pub` mint primitives that are the cross-crate construction surface
//! for the four UOR-domain sealed types: `mint_datum`, `mint_triad`,
//! `mint_derivation`, `mint_freerank`. The wiki names these explicitly
//! as the architectural surface; ADR-011 + ADR-016 together commit
//! that prism's pipeline is the only sanctioned caller (a normative
//! commitment, not a Rust-language access restriction).

use uor_foundation::enforcement::{
    mint_datum, mint_derivation, mint_freerank, mint_triad, ContentFingerprint,
};
use uor_foundation::WittLevel;

#[test]
fn mint_datum_succeeds_for_well_sized_w8_bytes() {
    // W8 has byte width 1; mint_datum accepts a 1-byte input.
    let datum = mint_datum(WittLevel::W8, &[0x42]).expect("W8 datum mints");
    assert_eq!(datum.level(), WittLevel::W8);
    assert_eq!(datum.as_bytes(), &[0x42][..]);
}

#[test]
fn mint_datum_rejects_byte_width_mismatch() {
    // W16 expects 2 bytes; passing 1 byte fails.
    let result = mint_datum(WittLevel::W16, &[0x42]);
    assert!(
        result.is_err(),
        "byte-width mismatch must surface ShapeViolation"
    );
}

#[test]
fn mint_triad_constructs_sealed_triad() {
    // mint_triad is the cross-crate surface; foundation users construct
    // a `Triad<L>` only through this primitive (or through internal
    // grounding-time projection from a unit address).
    let triad = mint_triad::<()>(0x0a, 0xbb, 0xcc);
    assert_eq!(triad.stratum(), 0x0a);
    assert_eq!(triad.spectrum(), 0xbb);
    assert_eq!(triad.address(), 0xcc);
}

#[test]
fn mint_derivation_constructs_sealed_derivation() {
    let fp = ContentFingerprint::default();
    let der = mint_derivation(7, 16, fp);
    assert_eq!(der.step_count(), 7);
    assert_eq!(der.witt_level_bits(), 16);
}

#[test]
fn mint_freerank_constructs_sealed_freerank() {
    let fr = mint_freerank(8, 3);
    assert_eq!(fr.total(), 8);
    // The `pinned()` accessor is exposed; check it.
    assert_eq!(fr.pinned(), 3);
}
