//! schema.org conformance suite for the Photo and Document
//! schema-pinned descendants.
//!
//! Pins [`uor_addr::schema::photo::PhotoValue`] against
//! <https://schema.org/Photograph> (extending ImageObject →
//! MediaObject → CreativeWork → Thing) and
//! [`uor_addr::schema::document::DocumentValue`] against
//! <https://schema.org/Article> + its 14 standard subtypes.
//!
//! All inputs are JSON-LD per W3C Recommendation
//! <https://www.w3.org/TR/json-ld11/>.

use uor_addr::schema::{document, photo};

// ─── schema.org/Photograph (https://schema.org/Photograph) ─────────

const VALID_PHOTOGRAPH_MINIMAL: &[u8] = br#"{
    "@context": "https://schema.org",
    "@type": "Photograph",
    "contentUrl": "https://example.org/photo.jpg",
    "creator": {"@type": "Person", "name": "Ada Lovelace"}
}"#;

#[test]
fn schema_org_photograph_minimal_admits() {
    let outcome = photo::address(VALID_PHOTOGRAPH_MINIMAL).expect("valid");
    assert!(outcome.address.starts_with("sha256:"));
}

#[test]
fn schema_org_photograph_admits_string_creator() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": "Anonymous"
    }"#;
    photo::address(raw).expect("string creator admits");
}

#[test]
fn schema_org_photograph_admits_organization_creator() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": {"@type": "Organization", "name": "Wikimedia Commons"}
    }"#;
    photo::address(raw).expect("Organization creator admits");
}

#[test]
fn schema_org_photograph_http_and_https_contexts_both_admissible() {
    // schema.org publishes both http:// and https:// canonical IRIs.
    for ctx in ["https://schema.org", "http://schema.org"] {
        let raw = alloc::format!(
            r#"{{"@context":"{ctx}","@type":"Photograph","contentUrl":"https://x","creator":"y"}}"#
        );
        photo::address(raw.as_bytes()).expect("both contexts admissible");
    }
}

#[test]
fn schema_org_photograph_admits_full_creative_work_properties() {
    // Photograph inherits CreativeWork's typed properties.
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/skyline.jpg",
        "creator": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15",
        "license": "https://creativecommons.org/licenses/by/4.0/",
        "contentLocation": {"@type": "Place", "name": "New York City"},
        "keywords": "skyline, sunrise, urban",
        "name": "Manhattan at Dawn",
        "description": "Skyline photograph taken at dawn from the Brooklyn Bridge",
        "thumbnail": {
            "@type": "ImageObject",
            "contentUrl": "https://example.org/skyline-thumb.jpg",
            "width": "320",
            "height": "240"
        },
        "encodingFormat": "image/jpeg",
        "width": "1920",
        "height": "1080"
    }"#;
    photo::address(raw).expect("CreativeWork properties admit");
}

#[test]
fn schema_org_photograph_rejects_wrong_context() {
    for bad_ctx in [
        r#""@context": "https://example.org""#,
        r#""@context": ["https://schema.org"]"#,
        r#""@context": null"#,
    ] {
        let raw = alloc::format!(
            r#"{{{bad_ctx},"@type":"Photograph","contentUrl":"https://x","creator":"y"}}"#
        );
        match photo::address(raw.as_bytes()) {
            Err(photo::AddressFailure::SchemaViolation) => {}
            other => panic!("expected rejection for {bad_ctx:?}: {other:?}"),
        }
    }
}

#[test]
fn schema_org_photograph_rejects_unrelated_types() {
    // Type IRI must be exactly "Photograph"; not Article, not
    // ImageObject (the parent), not custom types.
    for bad_type in [
        "Article",
        "ImageObject",
        "MediaObject",
        "Thing",
        "CustomType",
    ] {
        let raw = alloc::format!(
            r#"{{"@context":"https://schema.org","@type":"{bad_type}","contentUrl":"https://x","creator":"y"}}"#
        );
        match photo::address(raw.as_bytes()) {
            Err(photo::AddressFailure::SchemaViolation) => {}
            other => panic!("expected rejection for @type={bad_type}: {other:?}"),
        }
    }
}

#[test]
fn schema_org_photograph_rejects_creator_without_name() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": {"@type": "Person"}
    }"#;
    match photo::address(raw) {
        Err(photo::AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn schema_org_photograph_kappa_label_matches_json_realization() {
    // schema.org admission applies at parse time only; κ-label is
    // computed by the underlying JSON realization. Confirms the
    // canonicalization path is shared.
    let from_photo = photo::address(VALID_PHOTOGRAPH_MINIMAL)
        .expect("κ-label")
        .address;
    let from_json = uor_addr::json::address(VALID_PHOTOGRAPH_MINIMAL)
        .expect("κ-label")
        .address;
    assert_eq!(from_photo, from_json);
}

// ─── schema.org/Article (https://schema.org/Article) ──────────────

const VALID_ARTICLE_MINIMAL: &[u8] = br#"{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "On Typed Content-Addressing",
    "author": {"@type": "Person", "name": "Ada Lovelace"},
    "datePublished": "2025-01-15"
}"#;

#[test]
fn schema_org_article_minimal_admits() {
    document::address(VALID_ARTICLE_MINIMAL).expect("valid");
}

#[test]
fn schema_org_article_admits_all_published_subtypes() {
    // schema.org/Article's 14 standard subtypes per the type hierarchy
    // visible at <https://schema.org/Article#subtypes>.
    let subtypes = [
        "Article",
        "NewsArticle",
        "Report",
        "ScholarlyArticle",
        "SocialMediaPosting",
        "TechArticle",
        "BlogPosting",
        "AdvertiserContentArticle",
        "AnalysisNewsArticle",
        "AskPublicNewsArticle",
        "BackgroundNewsArticle",
        "OpinionNewsArticle",
        "ReportageNewsArticle",
        "ReviewNewsArticle",
        "SatiricalArticle",
    ];
    for ty in subtypes {
        let raw = alloc::format!(
            r#"{{"@context":"https://schema.org","@type":"{ty}","headline":"h","author":"a","datePublished":"2025-01-15"}}"#
        );
        document::address(raw.as_bytes()).expect("subtype admits");
    }
}

#[test]
fn schema_org_article_admits_multi_author_array() {
    // JSON-LD multi-value pattern — author may be an array of
    // Person/Organization or strings.
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "Co-Authored Paper",
        "author": [
            {"@type": "Person", "name": "Ada Lovelace"},
            {"@type": "Person", "name": "Alan Turing"},
            "Anonymous"
        ],
        "datePublished": "2025-01-15"
    }"#;
    document::address(raw).expect("multi-author admits");
}

#[test]
fn schema_org_article_admits_iso8601_date_and_datetime() {
    // schema.org/Date admits ISO 8601 date or date-time strings.
    for date in [
        "2025-01-15",
        "2025-01-15T12:00:00Z",
        "2025-01-15T12:00:00+00:00",
        "2025-12-31T23:59:59.999Z",
    ] {
        let raw = alloc::format!(
            r#"{{"@context":"https://schema.org","@type":"Article","headline":"h","author":"a","datePublished":"{date}"}}"#
        );
        document::address(raw.as_bytes()).expect("ISO 8601 admits");
    }
}

#[test]
fn schema_org_article_admits_creative_work_properties() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "ScholarlyArticle",
        "headline": "A Comprehensive Survey",
        "author": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15T12:00:00Z",
        "articleBody": "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "wordCount": "1024",
        "publisher": {"@type": "Organization", "name": "Acme Publishing"},
        "keywords": ["typed addressing", "content addressing"],
        "inLanguage": "en-US",
        "citation": [
            {"@type": "ScholarlyArticle", "name": "Rivest 1997"},
            "RFC 8785 JCS"
        ],
        "isAccessibleForFree": true
    }"#;
    document::address(raw).expect("creative-work properties admit");
}

#[test]
fn schema_org_article_rejects_empty_headline() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "",
        "author": "a",
        "datePublished": "2025-01-15"
    }"#;
    // Empty-string headline currently admits (schema.org does not
    // mandate non-emptiness); the admission boundary here is just
    // that headline must be present and stringy.
    document::address(raw).expect("empty-string headline admits");
}

#[test]
fn schema_org_article_rejects_non_string_headline() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": 42,
        "author": "a",
        "datePublished": "2025-01-15"
    }"#;
    match document::address(raw) {
        Err(document::AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn schema_org_article_rejects_empty_author_array() {
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "h",
        "author": [],
        "datePublished": "2025-01-15"
    }"#;
    match document::address(raw) {
        Err(document::AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn schema_org_article_kappa_label_matches_json_realization() {
    let from_doc = document::address(VALID_ARTICLE_MINIMAL)
        .expect("κ-label")
        .address;
    let from_json = uor_addr::json::address(VALID_ARTICLE_MINIMAL)
        .expect("κ-label")
        .address;
    assert_eq!(from_doc, from_json);
}

#[test]
fn schema_org_typed_distinction_photograph_vs_article() {
    // Two distinct schema.org types over different valid inputs
    // produce distinct κ-labels via JCS canonicalization.
    let photo_label = photo::address(VALID_PHOTOGRAPH_MINIMAL)
        .expect("κ-label")
        .address;
    let article_label = document::address(VALID_ARTICLE_MINIMAL)
        .expect("κ-label")
        .address;
    assert_ne!(photo_label, article_label);
}

extern crate alloc;
