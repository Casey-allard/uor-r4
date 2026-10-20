//! **`uor_addr::schema::photo` — Photo content-addressing**
//! (ARCHITECTURE.md "Schema-pinned descendants" § `uor-addr-photo`).
//!
//! Schema-pinned descendant of [`crate::json`]. **Imports
//! schema.org's `Photograph` type** — the host-boundary parser
//! admits only JSON-LD values that conform to schema.org's published
//! Photograph taxon. ψ-pipeline and κ-derivation are inherited from
//! the JSON realization without modification.
//!
//! Per UOR's schema-import discipline (per the
//! [UOR-Framework wiki](https://github.com/UOR-Foundation/UOR-Framework/wiki)),
//! this module does **not** define a custom photo schema; it imports
//! `https://schema.org/Photograph` and applies the schema-validation
//! rules schema.org publishes.
//!
//! # `no_std` + `no_alloc`
//!
//! Schema admission walks the parsed [`crate::json::JsonValue`]'s
//! tagged bytes via [`crate::json::JsonValueRef`]. There is no
//! intermediate `serde_json::Value`; no allocator is touched.
//!
//! # Authoritative sources
//!
//! - **schema.org Photograph type** — <https://schema.org/Photograph>.
//! - **JSON-LD 1.1** — W3C REC — <https://www.w3.org/TR/json-ld11/>.
//!
//! # Admission predicate (the schema.org/Photograph contract)
//!
//! The input must be a JSON-LD object satisfying:
//!
//! 1. `@context` is `"https://schema.org"` or `"http://schema.org"`.
//! 2. `@type` is `"Photograph"`.
//! 3. `contentUrl` — string URL.
//! 4. `creator` — string OR object with `@type` in
//!    `{Person, Organization}` and a `name` string.

use prism::pipeline::{ShapeViolation, ViolationKind};

use crate::json::{JsonValue, JsonValueRef};

// ─── ShapeViolation IRIs ────────────────────────────────────────────────

const PHOTO_SCHEMA_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://schema.org/Photograph",
    constraint_iri: "https://schema.org/Photograph/schemaOrgConformance",
    property_iri: "https://schema.org/Photograph",
    expected_range: "https://schema.org/Photograph",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

/// schema.org canonical context IRIs (HTTP + HTTPS variants).
pub const SCHEMA_ORG_CONTEXTS: &[&[u8]] = &[b"https://schema.org", b"http://schema.org"];

/// schema.org Photograph type IRI fragment (used in the `@type` field).
pub const PHOTOGRAPH_TYPE: &[u8] = b"Photograph";

/// Required properties for a schema.org/Photograph instance.
pub const REQUIRED_PROPERTIES: &[&[u8]] = &[b"@context", b"@type", b"contentUrl", b"creator"];

/// Typed Photo content-addressing input. Wraps a [`JsonValue`] whose
/// runtime JSON structure conforms to schema.org/Photograph.
#[derive(Clone)]
pub struct PhotoValue {
    inner: JsonValue,
}

impl core::fmt::Debug for PhotoValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhotoValue").finish_non_exhaustive()
    }
}

impl PhotoValue {
    /// Parse + admit. Accepts raw JSON bytes; admits only inputs
    /// that conform to schema.org/Photograph.
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        let inner = JsonValue::parse(raw).map_err(|_| PHOTO_SCHEMA_VIOLATION)?;
        let root = JsonValueRef::root(&inner);
        if !root.is_object() {
            return Err(PHOTO_SCHEMA_VIOLATION);
        }

        // @context — string in SCHEMA_ORG_CONTEXTS.
        let context = root
            .get(b"@context")
            .and_then(|v| v.as_str())
            .ok_or(PHOTO_SCHEMA_VIOLATION)?;
        if !SCHEMA_ORG_CONTEXTS.contains(&context) {
            return Err(PHOTO_SCHEMA_VIOLATION);
        }

        // @type — "Photograph".
        let ty = root
            .get(b"@type")
            .and_then(|v| v.as_str())
            .ok_or(PHOTO_SCHEMA_VIOLATION)?;
        if ty != PHOTOGRAPH_TYPE {
            return Err(PHOTO_SCHEMA_VIOLATION);
        }

        // contentUrl — string.
        let _ = root
            .get(b"contentUrl")
            .and_then(|v| v.as_str())
            .ok_or(PHOTO_SCHEMA_VIOLATION)?;

        // creator — string OR object with @type ∈ {Person, Organization} + name.
        let creator = root.get(b"creator").ok_or(PHOTO_SCHEMA_VIOLATION)?;
        if creator.as_str().is_some() {
            // string form — ok.
        } else if creator.is_object() {
            let ct = creator
                .get(b"@type")
                .and_then(|v| v.as_str())
                .ok_or(PHOTO_SCHEMA_VIOLATION)?;
            if ct != b"Person" && ct != b"Organization" {
                return Err(PHOTO_SCHEMA_VIOLATION);
            }
            if creator.get(b"name").and_then(|v| v.as_str()).is_none() {
                return Err(PHOTO_SCHEMA_VIOLATION);
            }
        } else {
            return Err(PHOTO_SCHEMA_VIOLATION);
        }

        Ok(Self { inner })
    }

    /// Borrow the inner JSON tagged bytes.
    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        self.inner.tagged_bytes()
    }
}

/// Mint a κ-label over a schema.org/Photograph-admitted JSON value.
/// The κ-label is byte-identical to [`crate::json::address`]'s
/// κ-label for the same JSON input — schema admission applies at
/// parse time per SD2 Grounding, not in the ψ-pipeline.
pub fn address(raw: &[u8]) -> Result<crate::AddressOutcome<71>, AddressFailure> {
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::address_sha512(raw).map_err(|e| match e {
        crate::json::AddressFailure::InvalidJson => AddressFailure::SchemaViolation,
        crate::json::AddressFailure::PipelineFailure => AddressFailure::PipelineFailure,
    })
}

/// Failure modes from [`address`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    /// Input did not conform to schema.org/Photograph.
    SchemaViolation,
    /// Defensive: substrate-level shape violation.
    PipelineFailure,
}

/// **Available only under the `alloc` feature.** Canonical-bytes
/// accessor — the schema admission applies at ingress; the canonical
/// bytes are JCS-RFC8785 + NFC per the JSON realization.
#[cfg(feature = "alloc")]
pub fn canonicalize(raw: &[u8]) -> Result<alloc::vec::Vec<u8>, AddressFailure> {
    extern crate alloc;
    PhotoValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::canonicalize(raw).map_err(|_| AddressFailure::PipelineFailure)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PHOTO: &[u8] = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/photo.jpg",
        "creator": {"@type": "Person", "name": "Ada Lovelace"}
    }"#;

    #[test]
    fn admits_valid_schema_org_photograph() {
        let p = PhotoValue::parse(VALID_PHOTO).expect("valid");
        assert!(!p.tagged_bytes().is_empty());
    }

    #[test]
    fn admits_string_creator() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Photograph",
            "contentUrl": "https://example.org/photo.jpg",
            "creator": "Ada Lovelace"
        }"#;
        let p = PhotoValue::parse(raw).expect("valid");
        assert!(!p.tagged_bytes().is_empty());
    }

    #[test]
    fn admits_http_context() {
        let raw = br#"{
            "@context": "http://schema.org",
            "@type": "Photograph",
            "contentUrl": "https://example.org/photo.jpg",
            "creator": "Ada Lovelace"
        }"#;
        PhotoValue::parse(raw).expect("valid");
    }

    #[test]
    fn rejects_wrong_context() {
        let raw = br#"{
            "@context": "https://example.org/custom",
            "@type": "Photograph",
            "contentUrl": "https://example.org/photo.jpg",
            "creator": "Ada Lovelace"
        }"#;
        let err = PhotoValue::parse(raw).expect_err("not schema.org");
        assert_eq!(err.constraint_iri, PHOTO_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_wrong_type() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Article",
            "contentUrl": "https://example.org/photo.jpg",
            "creator": "Ada Lovelace"
        }"#;
        let err = PhotoValue::parse(raw).expect_err("not Photograph");
        assert_eq!(err.constraint_iri, PHOTO_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_missing_content_url() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Photograph",
            "creator": "Ada Lovelace"
        }"#;
        let err = PhotoValue::parse(raw).expect_err("missing contentUrl");
        assert_eq!(err.constraint_iri, PHOTO_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_creator_with_unsupported_type() {
        let raw = br#"{
            "@context": "https://schema.org",
            "@type": "Photograph",
            "contentUrl": "https://example.org/photo.jpg",
            "creator": {"@type": "Robot", "name": "A.L.I.C.E."}
        }"#;
        let err = PhotoValue::parse(raw).expect_err("unsupported creator @type");
        assert_eq!(err.constraint_iri, PHOTO_SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn address_matches_json_realization_for_admitted_input() {
        let from_photo = address(VALID_PHOTO).expect("κ-label").address;
        let from_json = crate::json::address(VALID_PHOTO).expect("κ-label").address;
        assert_eq!(from_photo, from_json);
    }
}
