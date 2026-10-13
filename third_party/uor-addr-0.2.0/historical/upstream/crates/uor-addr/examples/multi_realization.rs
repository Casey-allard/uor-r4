//! `uor-addr` — multi-realization showcase.
//!
//! Walks every shipped realization (JSON, S-expression, XML,
//! ASN.1 DER, ring elements, code-module AST, schema descendants,
//! cost-model variants) end-to-end and prints the resulting
//! κ-label.
//!
//! Run with `cargo run -p uor-addr --example multi_realization`.

fn main() {
    println!("uor-addr — multi-realization showcase\n");

    // JSON
    let raw = br#"{"hello":"world"}"#;
    let outcome = uor_addr::json::address(raw).expect("κ-label");
    println!("  json:          {}", outcome.address);

    // S-expression
    let outcome = uor_addr::sexp::address(b"(hello world)").expect("κ-label");
    println!("  sexp:          {}", outcome.address);

    // XML
    let outcome = uor_addr::xml::address(br#"<greet to="world">hello</greet>"#).expect("κ-label");
    println!("  xml:           {}", outcome.address);

    // ASN.1 DER — INTEGER 42
    let outcome = uor_addr::asn1::address(&[0x02, 0x01, 0x2A]).expect("κ-label");
    println!("  asn1:          {}", outcome.address);

    // Ring — Witt level 0, value 0x42
    let outcome = uor_addr::ring::address(&[0u8, 0x42]).expect("κ-label");
    println!("  ring:          {}", outcome.address);

    // Code-module AST
    let body = uor_addr::codemodule::CodeModuleValue::atom("body");
    let ret = uor_addr::codemodule::CodeModuleValue::atom("u32");
    let f = uor_addr::codemodule::CodeModuleValue::function("hello", &[], &ret, &body);
    let m = uor_addr::codemodule::CodeModuleValue::module("demo", &[f]);
    let outcome = uor_addr::codemodule::address(m.tagged_bytes()).expect("κ-label");
    println!("  codemodule:    {}", outcome.address);

    // Schema descendants — schema.org/Photograph, schema.org/Article,
    // in-toto Statement v1.
    let photo = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/skyline.jpg",
        "creator": {"@type": "Person", "name": "Ada Lovelace"}
    }"#;
    let outcome = uor_addr::schema::photo::address(photo).expect("κ-label");
    println!("  photo (schema.org/Photograph):   {}", outcome.address);

    let doc = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "Hello",
        "author": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15"
    }"#;
    let outcome = uor_addr::schema::document::address(doc).expect("κ-label");
    println!("  article (schema.org/Article):    {}", outcome.address);

    let attestation = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": "uor-addr-v0.1.0", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {"buildDefinition": {"buildType": "uor:test"}}
    }"#;
    let outcome = uor_addr::schema::codemodule_signed::address(attestation).expect("κ-label");
    println!("  signed (in-toto Statement v1):   {}", outcome.address);

    println!("\nOK — every realization produced its 71-byte sha256:<64hex> κ-label.");
}
