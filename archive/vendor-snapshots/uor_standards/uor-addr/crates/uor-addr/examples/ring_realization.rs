//! `uor-addr` — Ring-element realization comprehensive example.
//!
//! Demonstrates [`uor_addr::ring::address`] across the
//! UOR-Framework Amendment 43 §2 canonical-bytes layout:
//! `header(k) || le_bytes(x, k+1)`. Exercises every admissible
//! Witt level (0..=3), shows the canonicalizer-is-identity
//! property, demonstrates typed distinction across Witt levels and
//! coefficients, and walks the failure modes.
//!
//! Run with `cargo run -p uor-addr --example ring_realization`.

use uor_addr::ring::{address, AddressFailure, RingElement, MAX_WITT_LEVEL};

fn main() {
    println!("uor-addr — ring-element realization (Amendment 43 §2)\n");

    // 1. Every admissible Witt level.
    println!("1. All admissible Witt levels (0..={MAX_WITT_LEVEL})");
    for k in 0..=MAX_WITT_LEVEL {
        let coeff = 0x12_34_56_78u64;
        let element = RingElement::from_components(k, coeff).expect("valid");
        let bytes = element.tagged_bytes();
        let outcome = address(bytes).expect("κ-label");
        println!(
            "   k={k}, coefficient=0x{coeff:08X}, bytes={bytes:02X?} → κ-label: {}",
            outcome.address
        );
    }
    println!();

    // 2. Canonicalizer-is-identity property (Amendment 43 §2 pins
    //    canonical bytes at construction).
    let e = RingElement::from_components(2, 0xCAFE_BABE).expect("valid");
    let bytes = e.tagged_bytes().to_vec();
    let canon = RingElement::parse(&bytes)
        .expect("re-parse canonical bytes")
        .tagged_bytes()
        .to_vec();
    assert_eq!(canon, bytes);
    println!("2. Canonical bytes are pinned at construction (Amendment 43 §2)");
    println!("   element bytes: {:02X?}", bytes);
    println!("   re-parsed:     {:02X?}", canon);
    println!("   match: {} ✓\n", canon == bytes);

    // 3. Determinism.
    let e1 = RingElement::from_components(1, 0xABCD).expect("valid");
    let a = address(e1.tagged_bytes()).expect("κ-label").address;
    let b = address(e1.tagged_bytes()).expect("κ-label").address;
    assert_eq!(a, b);
    println!("3. Determinism");
    println!("   κ-label run 1: {a}");
    println!("   κ-label run 2: {b}");
    println!("   match: {} ✓\n", a == b);

    // 4. Typed distinction — different Witt levels with the same
    //    coefficient bytes yield distinct κ-labels.
    let k0 = RingElement::from_components(0, 0x42).expect("valid");
    let k1 = RingElement::from_components(1, 0x42).expect("valid");
    let k2 = RingElement::from_components(2, 0x42).expect("valid");
    let l0 = address(k0.tagged_bytes()).expect("κ-label").address;
    let l1 = address(k1.tagged_bytes()).expect("κ-label").address;
    let l2 = address(k2.tagged_bytes()).expect("κ-label").address;
    assert_ne!(l0, l1);
    assert_ne!(l0, l2);
    assert_ne!(l1, l2);
    println!("4. Typed distinction (different Witt levels)");
    println!("   k=0, x=0x42 → {l0}");
    println!("   k=1, x=0x42 → {l1}");
    println!("   k=2, x=0x42 → {l2}");
    println!();

    // 5. Distinct coefficients within the same Witt level.
    let v1 = RingElement::from_components(1, 0x0001).expect("valid");
    let v2 = RingElement::from_components(1, 0x0002).expect("valid");
    let a1 = address(v1.tagged_bytes()).expect("κ-label").address;
    let a2 = address(v2.tagged_bytes()).expect("κ-label").address;
    assert_ne!(a1, a2);
    println!("5. Distinct coefficients (same Witt level)");
    println!("   k=1, x=0x0001 → {a1}");
    println!("   k=1, x=0x0002 → {a2}");
    println!();

    // 6. Failure modes.
    println!("6. Failure modes");
    match RingElement::from_components(MAX_WITT_LEVEL + 1, 0) {
        Err(v) if v.constraint_iri.ends_with("/wittLevelBound") => {
            println!("   witt_level > {MAX_WITT_LEVEL} rejected via wittLevelBound ✓");
        }
        other => panic!("expected wittLevelBound: {other:?}"),
    }
    match address(&[]) {
        Err(AddressFailure::InvalidRingElement) => {
            println!("   empty input rejected as InvalidRingElement ✓")
        }
        other => panic!("expected InvalidRingElement: {other:?}"),
    }
    match address(&[2, 0, 0]) {
        // witt_level=2 requires 1 + 3 = 4 bytes total
        Err(AddressFailure::InvalidRingElement) => println!("   truncated bytes rejected ✓"),
        other => panic!("expected InvalidRingElement: {other:?}"),
    }

    println!("\nOK — ring realization conforms to UOR-Framework Amendment 43 §2.");
}
