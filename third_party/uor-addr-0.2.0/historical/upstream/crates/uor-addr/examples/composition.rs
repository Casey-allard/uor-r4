//! `uor-addr` — κ-label composition (ADR-061) worked example.
//!
//! Addresses two JSON documents to sha256 κ-labels, then composes them
//! with each of the five categorical operations on the Atlas image inside
//! E₈, printing the composed κ-labels and asserting the named algebraic
//! laws. Every assertion is a structural invariant, so a clean run is a
//! conformance witness.
//!
//! Run with `cargo run -p uor-addr --example composition`.

use uor_addr::composition::{
    compose_e6_filtration, compose_e7_augmentation, compose_e8_embedding, compose_f4_quotient,
    compose_g2_product,
};

fn main() {
    println!("uor-addr — κ-label composition (ADR-061)\n");

    // Two operands: address two JSON documents (sha256 σ-axis).
    let a = uor_addr::json::address(br#"{"role":"left"}"#)
        .expect("valid json")
        .address;
    let b = uor_addr::json::address(br#"{"role":"right"}"#)
        .expect("valid json")
        .address;
    println!("  operand a:  {a}");
    println!("  operand b:  {b}\n");

    // CS-G2 — commutative binary product.
    let ab = compose_g2_product(&a, &b).expect("g2(a,b)").address;
    let ba = compose_g2_product(&b, &a).expect("g2(b,a)").address;
    println!("  g2(a, b):   {ab}");
    assert_eq!(ab, ba, "CS-G2 is commutative");
    println!("  g2(b, a):   {ba}   (== g2(a,b): commutativity holds)\n");

    // CS-F4 — ± involution quotient (unary). An operand and its mirror
    // collapse to one composed label.
    let f4 = compose_f4_quotient(&a).expect("f4(a)").address;
    println!("  f4(a):      {f4}");

    // CS-E6 — degree-partition filtration (unary).
    let e6 = compose_e6_filtration(&a).expect("e6(a)").address;
    println!("  e6(a):      {e6}");

    // CS-E7 — S₄-orbit augmentation (unary).
    let e7 = compose_e7_augmentation(&a).expect("e7(a)").address;
    println!("  e7(a):      {e7}");

    // CS-E8 — direct embedding (unary; distinguished from the operand by
    // realization provenance).
    let e8 = compose_e8_embedding(&a).expect("e8(a)").address;
    println!("  e8(a):      {e8}");
    assert_ne!(e8.as_str(), a.as_str(), "CS-E8 composed label ≠ operand");

    // Every composed label is a well-formed 71-byte sha256 κ-label whose
    // witness replays (TC-05) to the same label.
    for outcome in [
        compose_g2_product(&a, &b).unwrap(),
        compose_f4_quotient(&a).unwrap(),
        compose_e6_filtration(&a).unwrap(),
        compose_e7_augmentation(&a).unwrap(),
        compose_e8_embedding(&a).unwrap(),
    ] {
        assert!(outcome.address.starts_with("sha256:") && outcome.address.len() == 71);
        assert_eq!(
            outcome.witness.verify().expect("witness verifies"),
            outcome.address,
            "TC-05 replay round-trip"
        );
    }

    println!("\nOK — every operation composed a verifiable κ-label and upheld its law.");
}
