//! `uor-addr` — Common architectural surface comprehensive example.
//!
//! Demonstrates the shared surface every format-specific UOR-ADDR
//! realization exposes (ADR-060): a host-boundary `address(raw)` entry
//! point that parses, canonicalizes, and folds the typed-input bytes
//! into the common [`uor_addr::AddressOutcome`] shape (a `KappaLabel`
//! plus a verifiable [`uor_addr::AddressWitness`]).
//!
//! Run with `cargo run -p uor-addr --example common_surface`.

fn show_outcome(
    name: &str,
    address: &uor_addr::KappaLabel<71>,
    witness: &uor_addr::AddressWitness<71>,
) {
    println!("─── {name} ─────────────────────────────────────");
    println!("   κ-label:              {address}");
    println!(
        "   content fingerprint:  {} bytes",
        witness.content_fingerprint().len()
    );

    // The witness re-derives the κ-label without re-invoking SHA-256 on
    // the original input; the recovered label must match.
    let recovered = witness.verify().expect("witness verifies");
    assert_eq!(
        &recovered, address,
        "verified κ-label matches minted address"
    );
    println!("   witness verifies:     yes\n");
}

fn main() {
    println!("uor-addr — common architectural surface (address entry points)\n");

    println!("Every realization exposes a host-boundary `address(raw)` that");
    println!("yields the common AddressOutcome shape:\n");
    println!("    pub struct AddressOutcome {{");
    println!("        pub address: KappaLabel,");
    println!("        pub witness: AddressWitness,");
    println!("    }}\n");

    // Walk every format-specific realization through its `address` entry.
    let o = uor_addr::json::address(br#"{"x":1}"#).expect("valid json");
    show_outcome("json", &o.address, &o.witness);

    let o = uor_addr::sexp::address(b"(a b c)").expect("valid sexp");
    show_outcome("sexp", &o.address, &o.witness);

    let o = uor_addr::xml::address(b"<root/>").expect("valid xml");
    show_outcome("xml", &o.address, &o.witness);

    let o = uor_addr::asn1::address(&[0x02, 0x01, 0x2A]).expect("valid der");
    show_outcome("asn1", &o.address, &o.witness);

    let o = uor_addr::ring::address(&[0u8, 0x42]).expect("valid ring element");
    show_outcome("ring", &o.address, &o.witness);

    let m = uor_addr::codemodule::CodeModuleValue::module("demo", &[]);
    let o = uor_addr::codemodule::address(m.tagged_bytes()).expect("valid ast");
    show_outcome("codemodule", &o.address, &o.witness);

    println!("OK — every realization exposes the common AddressOutcome surface.");
}
