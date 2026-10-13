//! `uor-addr` — Document schema descendant comprehensive example.
//!
//! Demonstrates [`uor_addr::schema::document::address`] — the
//! schema-pinned descendant that **imports schema.org's `Article`
//! type** (extending `CreativeWork`) over the JSON realization.
//! Shows admission across the Article subtype family (Article,
//! NewsArticle, ScholarlyArticle, …), rejection of malformed
//! inputs, and κ-label equivalence with the underlying JSON
//! realization.
//!
//! Authoritative sources:
//! - schema.org/Article — <https://schema.org/Article>
//! - JSON-LD 1.1 — <https://www.w3.org/TR/json-ld11/>
//!
//! Run with `cargo run -p uor-addr --example document_schema`.

use uor_addr::schema::document::{
    address, AddressFailure, DocumentValue, ARTICLE_TYPES, REQUIRED_PROPERTIES, SCHEMA_ORG_CONTEXTS,
};

fn main() {
    println!("uor-addr — Document schema descendant (imports schema.org/Article)\n");

    // 1. Schema-import surface.
    println!("1. Schema import");
    println!("   shape IRI:        https://schema.org/Article");
    println!("   @context values:  {:?}", SCHEMA_ORG_CONTEXTS);
    println!(
        "   admissible @type: {} subtypes (Article + 14 subtypes)",
        ARTICLE_TYPES.len()
    );
    println!("   required props:   {:?}", REQUIRED_PROPERTIES);
    println!();

    // 2. Admission of a valid schema.org/Article.
    let valid = br#"{
        "@context": "https://schema.org",
        "@type": "ScholarlyArticle",
        "headline": "On Typed Content-Addressing",
        "author": [
            {"@type": "Person", "name": "Ada Lovelace"},
            {"@type": "Person", "name": "Alan Turing"}
        ],
        "datePublished": "2025-01-15T12:00:00Z",
        "articleBody": "Schemas pin admissibility; the kappa-label remains canonical.",
        "citation": [
            {"@type": "ScholarlyArticle", "name": "Rivest 1997 S-expressions"},
            {"@type": "ScholarlyArticle", "name": "RFC 8785 JSON Canonicalization Scheme"}
        ]
    }"#;
    let outcome = address(valid).expect("valid Article");
    let typed = DocumentValue::parse(valid).expect("typed article");
    println!("2. Admission of valid schema.org/ScholarlyArticle JSON-LD");
    println!("   tagged bytes len: {} bytes", typed.tagged_bytes().len());
    println!("   κ-label:          {}\n", outcome.address);

    // 3. Subtype admission — NewsArticle, BlogPosting, TechArticle, …
    println!("3. Subtype admission across the Article family");
    for ty in ["Article", "NewsArticle", "BlogPosting", "TechArticle"] {
        let raw = alloc::format!(
            r#"{{"@context":"https://schema.org","@type":"{ty}","headline":"x","author":"y","datePublished":"2025-01-15"}}"#
        );
        let outcome = address(raw.as_bytes()).expect("valid subtype");
        println!("   @type={ty:<14} → κ-label: {}", outcome.address);
    }
    println!();

    // 4. κ-label equivalence with the JSON realization.
    let from_doc = address(valid).expect("κ-label").address;
    let from_json = uor_addr::json::address(valid).expect("κ-label").address;
    assert_eq!(from_doc, from_json);
    println!("4. κ-label matches the JSON realization");
    println!("   match: {} ✓\n", from_doc == from_json);

    // 5. Rejection cases.
    println!("5. Schema-violation rejections");

    let wrong_context = br#"{
        "@context": "https://example.org",
        "@type": "Article",
        "headline": "x", "author": "y", "datePublished": "2025-01-15"
    }"#;
    match address(wrong_context) {
        Err(AddressFailure::SchemaViolation) => println!("   non-schema.org @context rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    let wrong_type = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "headline": "x", "author": "y", "datePublished": "2025-01-15"
    }"#;
    match address(wrong_type) {
        Err(AddressFailure::SchemaViolation) => {
            println!("   @type ∉ Article-subtypes rejected ✓")
        }
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    let missing_headline = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "author": "y", "datePublished": "2025-01-15"
    }"#;
    match address(missing_headline) {
        Err(AddressFailure::SchemaViolation) => println!("   missing headline rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    let missing_date = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "x", "author": "y"
    }"#;
    match address(missing_date) {
        Err(AddressFailure::SchemaViolation) => println!("   missing datePublished rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    println!("\nOK — schema.org/Article import discipline pinned at admission boundary.");
}

extern crate alloc;
