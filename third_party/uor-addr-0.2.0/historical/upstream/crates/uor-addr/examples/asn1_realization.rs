//! `uor-addr` — ASN.1 realization comprehensive example.
//!
//! Demonstrates [`uor_addr::asn1::address`] across the X.690 DER
//! conformance surface: Boolean (§8.2.2 + §11.1), Integer
//! minimum-octets (§8.3), Null (§8.8), Sequence (§8.9), length
//! encoding (§8.1.3 + §10.1), determinism, typed distinction, and
//! the canonical-form rejection rules for non-DER input.
//!
//! Run with `cargo run -p uor-addr --example asn1_realization`.

use uor_addr::asn1::{address, canonicalize, AddressFailure, Asn1Value};

fn main() {
    println!("uor-addr — ASN.1 realization (ITU-T X.690 DER)\n");

    // 1. Boolean encoding (X.690 §8.2.2 + §11.1).
    let true_val = Asn1Value::boolean(true);
    let false_val = Asn1Value::boolean(false);
    println!("1. Boolean (§8.2.2 + §11.1)");
    println!(
        "   TRUE  bytes: {:?}, κ-label: {}",
        true_val.tagged_bytes(),
        address(true_val.tagged_bytes()).expect("κ-label").address
    );
    println!(
        "   FALSE bytes: {:?}, κ-label: {}\n",
        false_val.tagged_bytes(),
        address(false_val.tagged_bytes()).expect("κ-label").address
    );

    // 2. Integer encoding minimum-octets (X.690 §8.3.2).
    let zero = Asn1Value::integer(0);
    let small_pos = Asn1Value::integer(127);
    let needs_pad = Asn1Value::integer(128);
    let small_neg = Asn1Value::integer(-1);
    println!("2. Integer minimum-octets (§8.3.2)");
    println!("   0    → {:?}", zero.tagged_bytes());
    println!("   127  → {:?}", small_pos.tagged_bytes());
    println!(
        "   128  → {:?} (leading 0x00 keeps sign positive)",
        needs_pad.tagged_bytes()
    );
    println!("   -1   → {:?}\n", small_neg.tagged_bytes());

    // 3. Null + Sequence composition.
    let seq = Asn1Value::sequence(&[
        Asn1Value::boolean(true),
        Asn1Value::integer(42),
        Asn1Value::octet_string(b"hello"),
        Asn1Value::null(),
    ]);
    let outcome = address(seq.tagged_bytes()).expect("κ-label");
    println!("3. Sequence composition (§8.9)");
    println!("   SEQUENCE {{TRUE, 42, OCTET STRING 'hello', NULL}}");
    println!("   bytes: {:?}", seq.tagged_bytes());
    println!("   κ-label: {}\n", outcome.address);

    // 4. Determinism — DER is canonical, identity round-trip.
    let canon = canonicalize(seq.tagged_bytes()).expect("valid DER");
    assert_eq!(canon, seq.tagged_bytes());
    println!("4. DER canonicalization is the identity (§10)");
    println!("   canonicalize(seq) == seq.tagged_bytes() ✓\n");

    // 5. Typed distinction — same VALUE / different TYPE → distinct κ-labels.
    let int_42 = address(Asn1Value::integer(42).tagged_bytes())
        .expect("κ-label")
        .address;
    let str_42 = address(Asn1Value::octet_string(b"42").tagged_bytes())
        .expect("κ-label")
        .address;
    assert_ne!(int_42, str_42);
    println!("5. Typed distinction");
    println!("   INTEGER 42        → {int_42}");
    println!("   OCTET STRING \"42\" → {str_42}");
    println!();

    // 6. Failure modes — non-canonical DER rejection.
    println!("6. Failure modes (DER canonical-form rejection)");
    // Non-minimal INTEGER encoding (§8.3.2)
    match address(&[0x02, 0x02, 0x00, 0x01]) {
        Err(AddressFailure::InvalidDer) => println!("   non-minimal INTEGER 0x00 0x01 rejected ✓"),
        other => panic!("expected InvalidDer: {other:?}"),
    }
    // Long-form length for value < 128 (§10.1)
    match address(&[0x04, 0x81, 0x05, 0, 0, 0, 0, 0]) {
        Err(AddressFailure::InvalidDer) => println!("   long-form-length-under-128 rejected ✓"),
        other => panic!("expected InvalidDer: {other:?}"),
    }
    // Indefinite-length (§8.1.3.6, BER not DER)
    match address(&[0x30, 0x80]) {
        Err(AddressFailure::InvalidDer) => println!("   indefinite-length (BER only) rejected ✓"),
        other => panic!("expected InvalidDer: {other:?}"),
    }
    // Non-canonical Boolean (§11.1)
    match address(&[0x01, 0x01, 0x01]) {
        Err(AddressFailure::InvalidDer) => println!("   non-canonical Boolean 0x01 rejected ✓"),
        other => panic!("expected InvalidDer: {other:?}"),
    }

    println!("\nOK — ASN.1 realization conforms to ITU-T X.690 DER.");
}
