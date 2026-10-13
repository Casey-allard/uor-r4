//! **`uor_addr::schema::codemodule_signed` — signed-code-module
//! content-addressing** (ARCHITECTURE.md "Schema-pinned descendants"
//! § `uor-addr-codemodule-signed`).
//!
//! Schema-pinned descendant of [`crate::json`] that **imports the
//! in-toto Statement v1 attestation format** — the host-boundary
//! parser admits only JSON-LD-style values conforming to in-toto's
//! published Statement contract per
//! <https://in-toto.io/Statement/v1>.
//!
//! # `no_std` + `no_alloc`
//!
//! Schema admission walks the parsed [`crate::json::JsonValue`]'s
//! tagged bytes via [`crate::json::JsonValueRef`]. No `serde_json`,
//! no allocator.
//!
//! # Authoritative sources
//!
//! - **in-toto Statement v1** —
//!   <https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md>.
//! - **SLSA Provenance v1** — <https://slsa.dev/spec/v1.0/provenance>.
//! - **sigstore signature spec** —
//!   <https://docs.sigstore.dev/cosign/signature_specification/>.
//!
//! # Admission predicate
//!
//! 1. `_type` is `"https://in-toto.io/Statement/v1"`.
//! 2. `subject` is a non-empty array; each element is an object with:
//!    - `name` — non-empty string.
//!    - `digest` — object with at least one `sha256` entry whose value
//!      is a 64-character lowercase-hex SHA-256 digest.
//! 3. `predicateType` — non-empty string IRI.
//! 4. `predicate` — JSON object.

use prism::pipeline::{ShapeViolation, ViolationKind};

use crate::json::{JsonValue, JsonValueRef};

const SCHEMA_VIOLATION: ShapeViolation = ShapeViolation {
    shape_iri: "https://in-toto.io/Statement/v1",
    constraint_iri: "https://in-toto.io/Statement/v1/schemaConformance",
    property_iri: "https://in-toto.io/Statement/v1",
    expected_range: "https://in-toto.io/Statement/v1",
    min_count: 0,
    max_count: 1,
    kind: ViolationKind::ValueCheck,
};

/// in-toto Statement v1 `_type` IRI.
pub const STATEMENT_TYPE_IRI: &[u8] = b"https://in-toto.io/Statement/v1";

/// SHA-256 digest hex byte width.
pub const SHA256_HEX_BYTES: usize = 64;

pub const REQUIRED_PROPERTIES: &[&[u8]] = &[b"_type", b"subject", b"predicateType", b"predicate"];

#[derive(Clone)]
pub struct SignedCodeModuleValue {
    inner: JsonValue,
}

impl core::fmt::Debug for SignedCodeModuleValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignedCodeModuleValue")
            .finish_non_exhaustive()
    }
}

impl SignedCodeModuleValue {
    pub fn parse(raw: &[u8]) -> Result<Self, ShapeViolation> {
        let inner = JsonValue::parse(raw).map_err(|_| SCHEMA_VIOLATION)?;
        let root = JsonValueRef::root(&inner);
        if !root.is_object() {
            return Err(SCHEMA_VIOLATION);
        }
        // _type must be the in-toto Statement v1 IRI.
        let ty = root
            .get(b"_type")
            .and_then(|v| v.as_str())
            .ok_or(SCHEMA_VIOLATION)?;
        if ty != STATEMENT_TYPE_IRI {
            return Err(SCHEMA_VIOLATION);
        }
        // subject — non-empty array of {name, digest} objects.
        let subject = root.get(b"subject").ok_or(SCHEMA_VIOLATION)?;
        let subjects = subject.iter_array().ok_or(SCHEMA_VIOLATION)?;
        let mut subject_count = 0;
        for s in subjects {
            if !s.is_object() {
                return Err(SCHEMA_VIOLATION);
            }
            let name = s
                .get(b"name")
                .and_then(|v| v.as_str())
                .ok_or(SCHEMA_VIOLATION)?;
            if name.is_empty() {
                return Err(SCHEMA_VIOLATION);
            }
            let digest = s.get(b"digest").ok_or(SCHEMA_VIOLATION)?;
            if !digest.is_object() {
                return Err(SCHEMA_VIOLATION);
            }
            // Require at least one entry, and a sha256 entry whose
            // value is 64 lowercase hex bytes.
            let digest_iter = digest.iter_object().ok_or(SCHEMA_VIOLATION)?;
            let mut digest_count = 0;
            for (_k, _v) in digest_iter {
                digest_count += 1;
            }
            if digest_count == 0 {
                return Err(SCHEMA_VIOLATION);
            }
            let sha256 = digest
                .get(b"sha256")
                .and_then(|v| v.as_str())
                .ok_or(SCHEMA_VIOLATION)?;
            if sha256.len() != SHA256_HEX_BYTES
                || !sha256
                    .iter()
                    .all(|&b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(SCHEMA_VIOLATION);
            }
            subject_count += 1;
        }
        if subject_count == 0 {
            return Err(SCHEMA_VIOLATION);
        }
        // predicateType — non-empty string.
        let pt = root
            .get(b"predicateType")
            .and_then(|v| v.as_str())
            .ok_or(SCHEMA_VIOLATION)?;
        if pt.is_empty() {
            return Err(SCHEMA_VIOLATION);
        }
        // predicate — object.
        let predicate = root.get(b"predicate").ok_or(SCHEMA_VIOLATION)?;
        if !predicate.is_object() {
            return Err(SCHEMA_VIOLATION);
        }
        Ok(Self { inner })
    }

    #[must_use]
    pub fn tagged_bytes(&self) -> &[u8] {
        self.inner.tagged_bytes()
    }
}

/// Mint a κ-label over an in-toto-v1-Statement-admitted JSON value.
pub fn address(raw: &[u8]) -> Result<crate::AddressOutcome<71>, AddressFailure> {
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
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
    SignedCodeModuleValue::parse(raw).map_err(|_| AddressFailure::SchemaViolation)?;
    crate::json::canonicalize(raw).map_err(|_| AddressFailure::PipelineFailure)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_STATEMENT: &[u8] = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [
            {
                "name": "uor-addr-v0.1.0",
                "digest": {
                    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
            }
        ],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {
            "buildDefinition": {"buildType": "uor:test"},
            "runDetails": {"builder": {"id": "uor:test-builder"}}
        }
    }"#;

    #[test]
    fn admits_valid_in_toto_statement() {
        let s = SignedCodeModuleValue::parse(VALID_STATEMENT).expect("valid");
        assert!(!s.tagged_bytes().is_empty());
    }

    #[test]
    fn admits_multiple_subjects() {
        let raw = br#"{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [
                {"name": "a", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}},
                {"name": "b", "digest": {"sha256": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"}}
            ],
            "predicateType": "https://slsa.dev/provenance/v1",
            "predicate": {}
        }"#;
        SignedCodeModuleValue::parse(raw).expect("valid");
    }

    #[test]
    fn rejects_wrong_statement_type_iri() {
        let raw = br#"{
            "_type": "https://example.org/CustomStatement",
            "subject": [{"name": "x", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
            "predicateType": "x",
            "predicate": {}
        }"#;
        let err = SignedCodeModuleValue::parse(raw).expect_err("wrong _type");
        assert_eq!(err.constraint_iri, SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_empty_subject() {
        let raw = br#"{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [],
            "predicateType": "x",
            "predicate": {}
        }"#;
        let err = SignedCodeModuleValue::parse(raw).expect_err("empty subject");
        assert_eq!(err.constraint_iri, SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_subject_without_sha256_digest() {
        let raw = br#"{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "x", "digest": {"md5": "deadbeef"}}],
            "predicateType": "x",
            "predicate": {}
        }"#;
        let err = SignedCodeModuleValue::parse(raw).expect_err("no sha256");
        assert_eq!(err.constraint_iri, SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_sha256_with_wrong_length() {
        let raw = br#"{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "x", "digest": {"sha256": "tooshort"}}],
            "predicateType": "x",
            "predicate": {}
        }"#;
        let err = SignedCodeModuleValue::parse(raw).expect_err("short hex");
        assert_eq!(err.constraint_iri, SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn rejects_missing_predicate_type() {
        let raw = br#"{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{"name": "x", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
            "predicate": {}
        }"#;
        let err = SignedCodeModuleValue::parse(raw).expect_err("missing predicateType");
        assert_eq!(err.constraint_iri, SCHEMA_VIOLATION.constraint_iri);
    }

    #[test]
    fn address_matches_json_realization() {
        let from_signed = address(VALID_STATEMENT).expect("κ-label").address;
        let from_json = crate::json::address(VALID_STATEMENT)
            .expect("κ-label")
            .address;
        assert_eq!(from_signed, from_json);
    }
}
