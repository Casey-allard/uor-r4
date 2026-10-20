//! `uor-addr` — Photo schema descendant comprehensive example.
//!
//! Demonstrates [`uor_addr::schema::photo::address`] — the
//! schema-pinned descendant that **imports schema.org's `Photograph`
//! type** over the JSON realization. Shows admission of valid
//! schema.org/Photograph JSON-LD, rejection of malformed inputs,
//! κ-label equivalence with the underlying JSON realization, and
//! schema-import-discipline surface inspection.
//!
//! Authoritative sources:
//! - schema.org/Photograph — <https://schema.org/Photograph>
//! - JSON-LD 1.1 — <https://www.w3.org/TR/json-ld11/>
//!
//! Run with `cargo run -p uor-addr --example photo_schema`.

use uor_addr::schema::photo::{
    address, AddressFailure, PhotoValue, PHOTOGRAPH_TYPE, REQUIRED_PROPERTIES, SCHEMA_ORG_CONTEXTS,
};

fn main() {
    println!("uor-addr — Photo schema descendant (imports schema.org/Photograph)\n");

    // 1. Schema-import surface.
    println!("1. Schema import");
    println!("   shape IRI:       https://schema.org/Photograph");
    println!("   @context values: {:?}", SCHEMA_ORG_CONTEXTS);
    println!(
        "   @type value:     \"{}\"",
        std::str::from_utf8(PHOTOGRAPH_TYPE).expect("ASCII type")
    );
    println!("   required props:  {:?}", REQUIRED_PROPERTIES);
    println!();

    // 2. Admission of a valid schema.org/Photograph.
    let valid = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/skyline.jpg",
        "creator": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15",
        "contentLocation": {"@type": "Place", "name": "New York City"}
    }"#;
    let outcome = address(valid).expect("valid Photograph");
    let typed = PhotoValue::parse(valid).expect("typed photo");
    println!("2. Admission of valid schema.org/Photograph JSON-LD");
    println!("   tagged bytes len: {} bytes", typed.tagged_bytes().len());
    println!("   κ-label:          {}\n", outcome.address);

    // 3. κ-label equivalence with the JSON realization.
    let from_json = uor_addr::json::address(valid).expect("κ-label").address;
    assert_eq!(outcome.address, from_json);
    println!("3. κ-label matches the JSON realization");
    println!("   (schema admission applies at parse time, not in the ψ-pipeline)");
    println!("   match: {} ✓\n", outcome.address == from_json);

    // 4. Determinism.
    let a = address(valid).expect("κ-label").address;
    let b = address(valid).expect("κ-label").address;
    assert_eq!(a, b);
    println!("4. Determinism");
    println!("   κ-label (run 1): {a}");
    println!("   κ-label (run 2): {b}");
    println!("   match: {} ✓\n", a == b);

    // 5. Rejection cases.
    println!("5. Schema-violation rejections");

    // 5a. Wrong @context — not schema.org.
    let wrong_context = br#"{
        "@context": "https://example.org/custom",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": "Ada Lovelace"
    }"#;
    match address(wrong_context) {
        Err(AddressFailure::SchemaViolation) => println!("   non-schema.org @context rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5b. Wrong @type — not Photograph.
    let wrong_type = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": "Ada Lovelace"
    }"#;
    match address(wrong_type) {
        Err(AddressFailure::SchemaViolation) => println!("   non-Photograph @type rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5c. Missing required contentUrl (MediaObject property).
    let no_url = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "creator": "Ada Lovelace"
    }"#;
    match address(no_url) {
        Err(AddressFailure::SchemaViolation) => println!("   missing contentUrl rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5d. Creator with unsupported @type.
    let bad_creator = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": {"@type": "Robot", "name": "A.L.I.C.E."}
    }"#;
    match address(bad_creator) {
        Err(AddressFailure::SchemaViolation) => {
            println!("   creator @type ∉ {{Person, Organization}} rejected ✓")
        }
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    println!("\nOK — schema.org/Photograph import discipline pinned at admission boundary.");
}
