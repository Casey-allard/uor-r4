//! `uor-addr` — XML realization comprehensive example.
//!
//! Demonstrates [`uor_addr::xml::address`] across the full
//! conformance surface: basic minting, attribute-order invariance
//! (XML-C14N 1.1 §1.1 rule 3), CDATA-to-Text collapse (§1.1
//! rule 5), character escaping (§1.1 rules 4–5), typed
//! distinction across structural shapes, and failure modes.
//!
//! Run with `cargo run -p uor-addr --example xml_realization`.

use uor_addr::xml::{address, canonicalize, AddressFailure};

fn main() {
    println!("uor-addr — XML realization (W3C XML-C14N 1.1 subset)\n");

    // 1. Basic minting.
    let outcome = address(b"<root/>").expect("κ-label");
    println!("1. Basic minting");
    println!("   raw:   <root/>");
    println!("   κ-label: {}\n", outcome.address);

    // 2. Determinism — same input always yields the same κ-label.
    let a = address(b"<doc><child/></doc>").expect("κ-label").address;
    let b = address(b"<doc><child/></doc>").expect("κ-label").address;
    assert_eq!(a, b);
    println!("2. Determinism");
    println!("   <doc><child/></doc> κ-label (run 1): {a}");
    println!("   <doc><child/></doc> κ-label (run 2): {b}");
    println!("   match: {}\n", a == b);

    // 3. Attribute-order invariance — XML-C14N 1.1 §1.1 rule 3.
    let attr_ascending = address(br#"<el a="1" b="2" c="3"/>"#)
        .expect("κ-label")
        .address;
    let attr_reversed = address(br#"<el c="3" b="2" a="1"/>"#)
        .expect("κ-label")
        .address;
    assert_eq!(attr_ascending, attr_reversed);
    println!("3. Attribute-order invariance (§1.1 rule 3)");
    println!("   <el a=\"1\" b=\"2\" c=\"3\"/> κ-label: {attr_ascending}");
    println!("   <el c=\"3\" b=\"2\" a=\"1\"/> κ-label: {attr_reversed}");
    println!("   match: {}\n", attr_ascending == attr_reversed);

    // 4. CDATA-to-Text collapse — XML-C14N 1.1 §1.1.
    let cdata = address(b"<r><![CDATA[<v>]]></r>").expect("κ-label").address;
    let escaped = address(b"<r>&lt;v&gt;</r>").expect("κ-label").address;
    assert_eq!(cdata, escaped);
    println!("4. CDATA-to-Text collapse (§1.1)");
    println!("   <r><![CDATA[<v>]]></r> κ-label: {cdata}");
    println!("   <r>&lt;v&gt;</r>       κ-label: {escaped}");
    println!("   match: {}\n", cdata == escaped);

    // 5. Canonical-form inspection.
    let canon = canonicalize(br#"<el  b="2"   a="1" />"#).expect("valid");
    println!("5. Canonical-form output");
    println!("   raw:       <el  b=\"2\"   a=\"1\" />");
    println!(
        "   canonical: {}",
        core::str::from_utf8(&canon).unwrap_or("<binary>")
    );
    println!();

    // 6. Typed distinction — different structural shapes yield
    //    different κ-labels.
    let elem = address(b"<a/>").expect("κ-label").address;
    let text = address(b"<a>x</a>").expect("κ-label").address;
    let nested = address(b"<a><b/></a>").expect("κ-label").address;
    assert_ne!(elem, text);
    assert_ne!(elem, nested);
    assert_ne!(text, nested);
    println!("6. Typed distinction");
    println!("   <a/>          → {elem}");
    println!("   <a>x</a>      → {text}");
    println!("   <a><b/></a>   → {nested}");
    println!();

    // 7. Failure modes.
    println!("7. Failure modes");
    match address(b"<a></b>") {
        Err(AddressFailure::InvalidXml) => {
            println!("   mismatched-close-tag rejected as InvalidXml ✓")
        }
        other => panic!("expected InvalidXml: {other:?}"),
    }
    match address(b"not-xml") {
        Err(AddressFailure::InvalidXml) => println!("   non-XML input rejected as InvalidXml ✓"),
        other => panic!("expected InvalidXml: {other:?}"),
    }

    println!("\nOK — XML realization conforms to W3C XML-C14N 1.1 (subset).");
}
