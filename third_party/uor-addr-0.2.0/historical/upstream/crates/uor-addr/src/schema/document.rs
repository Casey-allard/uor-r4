//! **`uor_addr::schema::document` — Document content-addressing**
//! (ARCHITECTURE.md "Schema-pinned descendants" § `uor-addr-document`).
//!
//! Schema-pinned descendant of [`crate::json`]. **Imports
//! schema.org's `Article` type** (extending `CreativeWork`) — the
//! host-boundary parser admits only JSON-LD values conforming to
//! schema.org's published Article taxon.
//!
//! # `no_std` + `no_alloc`
//!
//! Schema admission walks the parsed [`crate::json::JsonValue`]'s
//! tagged bytes via [`crate::json::JsonValueRef`]. No `serde_json`,
//! no allocator.
//!
//! # Authoritative sources
//!
//! - **schema.org Article type** — <https://schema.org/Article>.
//! - **JSON-LD 1.1** — W3C REC — <https://www.w3.org/TR/json-ld11/>.
//!
//! # Admission predicate
//!
//! 1. `@context` is `"https://schema.org"` or `"http://schema.org"`.
//! 2. `@type` is `"Article"` or one of its admissible subtypes.
//! 3. `headline` — string.
//! 4. `author` — string, Person/Organization object, or non-empty
//!    array of either.
//! 5. `datePublished` — non-empty string (ISO 8601 / RFC 3339).

use prism::pipeline::{ShapeViolation, ViolationKind};

use crate::json::{JsonValue, JsonValueRef};

const DOC_SCHEMA_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://schema.org/Article",
    constraint_iri: "https://schema.org/Article/schemaOrgConformance",
    property_iri: "https://schema.org/Article",
    expected_range: "https://schema.org/Article",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

pub const SCHEMA_ORG_CONTEXTS: &[&[u8]] = &[b"https://schema.org", b"http://schema.org"];

/// Admissible `@type` values — `Article` plus its standard subtypes
/// per <https://schema.org/Article>.
pub const ARTICLE_TYPES: &[&[u8]] = &[
    b"Article",
    b"NewsArticle",
    b"Report",
    b"ScholarlyArticle",
    b"SocialMediaPosting",
    b"TechArticle",
    b"BlogPosting",
    b"AdvertiserContentArticle",
    b"AnalysisNewsArticle",
    b"AskPublicNewsArticle",
    b"BackgroundNewsArticle",
    b"OpinionNewsArticle",
    b"ReportageNewsArticle",
    b"ReviewNewsArticle",
    b"SatiricalArticle",
];

pub const REQUIRED_PROPERTIES: &[&[u8]] = &[
    b"@context",
    b"@type",
    b"headline",
    b"author",
    b"datePublished",
];

#[derive(Clone)]
pub struct DocumentValue {
    inner: JsonValue,
}

impl core::fmt::Debug for DocumentValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DocumentValue").finish_non_exhaustive()
    }
}

impl DocumentValue {
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        let inner = JsonValue::parse(raw).map_err(|_| DOC_SCHEMA_VIOLATION)?;
        let root = JsonValueRef::root(&inner);
        if !root.is_object() {
            return Err(DOC_SCHEMA_VIOLATION);
        }
        // @context
        let context = root
            .get(b"@context")
            .and_then(|v| v.as_str())
            .ok_or(DOC_SCHEMA_VIOLATION)?;
        if !SCHEMA_ORG_CONTEXTS.contains(&context) {
            return Err(DOC_SCHEMA_VIOLATION);
        }
        // @type
        let ty = root
            .get(b"@type")
            .and_then(|v| v.as_str())
            .ok_or(DOC_SCHEMA_VIOLATION)?;
        if !ARTICLE_TYPES.contains(&ty) {
            return Err(DOC_SCHEMA_VIOLATION);
        }
        // headline
        let _ = root
            .get(b"headline")
            .and_then(|v| v.as_str())
            .ok_or(DOC_SCHEMA_VIOLATION)?;
        // author
        validate_author(root.get(b"author"))?;
        // datePublished
        let date = root
            .get(b"datePublished")
            .and_then(|v| v.as_str())
            .ok_or(DOC_SCHEMA_VIOLATION)?;
        if date.is_empty() {
            return Err(DOC_SCHEMA_VIOLATION);
        }
        Ok(Self { inner })
    }

    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        self.inner.tagged_bytes()
    }
}

pub fn address(raw: &[u8]) -> Result<crate::AddressOutcome<71>, AddressFailure> {
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

/// As [`address`], but binds the `blake3` σ-axis ([`crate::hash`]). Schema
/// admission is identical; only the κ-derivation hash differs.
///
/// # Errors
///
/// As [`address`].
pub fn address_blake3(raw: &[u8]) -> Result<crate::AddressOutcome<71>, AddressFailure> {
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address_blake3(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

/// As [`address`], but binds the `sha3_256` σ-axis ([`crate::hash`]). Schema
/// admission is identical; only the κ-derivation hash differs.
///
/// # Errors
///
/// As [`address`].
pub fn address_sha3_256(raw: &[u8]) -> Result<crate::AddressOutcome<73>, AddressFailure> {
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address_sha3_256(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

/// As [`address`], but binds the `keccak256` σ-axis ([`crate::hash`]). Schema
/// admission is identical; only the κ-derivation hash differs.
///
/// # Errors
///
/// As [`address`].
pub fn address_keccak256(raw: &[u8]) -> Result<crate::AddressOutcome<74>, AddressFailure> {
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address_keccak256(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

/// As [`address`], but binds the `sha512` σ-axis ([`crate::hash`]).
///
/// # Errors
///
/// As [`address`].
pub fn address_sha512(raw: &[u8]) -> Result<crate::AddressOutcome<135, 64>, AddressFailure> {
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address_sha512(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    SchemaViolation,
    PipelineFailure,
}

/// **Available only under the `alloc` feature.**
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, AddressFailure> {
    extern crate alloc;
    DocumentValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::canonicalize(raw).map_err(|_| AddressFailure::PipelineFailure)
}

fn validate_author(value: Option<JsonValueRef<'_>>) -> Result<(), ShapeViolation> {
    let v = value.ok_or(DOC_SCHEMA_VIOLATION)?;
    if v.as_str().is_some() {
        return Ok(());
    }
    if v.is_object() {
        return validate_author_object(v);
    }
    if let Some(iter) = v.iter_array() {
        let mut count = 0;
        for item in iter {
            validate_author_item(item)?;
            count += 1;
        }
        if count == 0 {
            return Err(DOC_SCHEMA_VIOLATION);
        }
        return Ok(());
    }
    Err(DOC_SCHEMA_VIOLATION)
}

fn validate_author_item(value: JsonValueRef<'_>) -> Result<(), ShapeViolation> {
    if value.as_str().is_some() {
        return Ok(());
    }
    if value.is_object() {
        return validate_author_object(value);
    }
    Err(DOC_SCHEMA_VIOLATION)
}

fn validate_author_object(value: JsonValueRef<'_>) -> Result<(), ShapeViolation> {
    let at = value
        .get(b"@type")
        .and_then(|v| v.as_str())
        .ok_or(DOC_SCHEMA_VIOLATION)?;
    if at != b"Person" && at != b"Organization" {
        return Err(DOC_SCHEMA_VIOLATION);
    }
    let _ = value
        .get(b"name")
        .and_then(|v| v.as_str())
        .ok_or(DOC_SCHEMA_VIOLATION)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ARTICLE: &[u8] = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "On Typed Content Addressing",
        "author": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15"
    }"#;

    #[test]
    fn admits_valid_schema_org_article() {
        let d = DocumentValue::parse(VALID_ARTICLE).expect("valid");
        assert!(!d.tagged_bytes().is_empty());
    }

    #[test]
    fn admits_scholarly_article_subtype() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "ScholarlyArticle",
            "headline": "P vs. NP",
            "author": "Anonymous",
            "datePublished": "2025-01-15T12:00:00Z"
        }"#;
        DocumentValue::parse(raw).expect("valid");
    }

    #[test]
    fn admits_news_article_subtype() {
        let raw = br#"{
            "@context": "http://schema.org",
            "@type": "NewsArticle",
            "headline": "Breaking news",
            "author": "Newsdesk",
            "datePublished": "2025-01-15"
        }"#;
        DocumentValue::parse(raw).expect("valid");
    }

    #[test]
    fn rejects_non_schema_org_context() {
        let raw = br#"{
            "@context": "https://example.org",
            "@type": "Article",
            "headline": "x",
            "author": "y",
            "datePublished": "2025-01-15"
        }"#;
        let err = DocumentValue::parse(raw).expect_err("not schema.org");
        assert_eq!(err.constraint_iri, DOC_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_non_article_type() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Photograph",
            "headline": "x",
            "author": "y",
            "datePublished": "2025-01-15"
        }"#;
        let err = DocumentValue::parse(raw).expect_err("not Article");
        assert_eq!(err.constraint_iri, DOC_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_missing_headline() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Article",
            "author": "y",
            "datePublished": "2025-01-15"
        }"#;
        let err = DocumentValue::parse(raw).expect_err("missing headline");
        assert_eq!(err.constraint_iri, DOC_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn address_matches_json_realization() {
        let from_doc = address(VALID_ARTICLE).expect("κ-label").address;
        let from_json = crate::json::address(VALID_ARTICLE)
            .expect("κ-label")
            .address;
        assert_eq!(from_doc, from_json);
    }
}
