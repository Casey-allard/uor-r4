//! in-toto Statement v1 conformance suite for the
//! [`uor_addr::schema::codemodule_signed`] schema-pinned descendant.
//!
//! Pins admission against the in-toto Attestation Framework v1
//! Statement specification at
//! <https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md>
//! and the canonical `_type` IRI <https://in-toto.io/Statement/v1>.
//!
//! Predicate-shape coverage walks the SLSA Provenance v1
//! (<https://slsa.dev/spec/v1.0/provenance>), SLSA Verification
//! Summary, sigstore-bundle, and SCAI patterns documented in the
//! in-toto predicate-types registry.

use uor_addr::schema::codemodule_signed::{address, AddressFailure, STATEMENT_TYPE_IRI};

const SHA256_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn minimal_statement(predicate_type: &str, predicate: &str) -> alloc::vec::Vec<u8> {
    alloc::format!(
        r#"{{
            "_type": "https://in-toto.io/Statement/v1",
            "subject": [{{"name": "x", "digest": {{"sha256": "{SHA256_HEX}"}}}}],
            "predicateType": "{predicate_type}",
            "predicate": {predicate}
        }}"#
    )
    .into_bytes()
}

#[test]
fn in_toto_statement_v1_type_iri_is_normative() {
    // The Statement v1 spec pins `_type` to exactly this IRI.
    assert_eq!(STATEMENT_TYPE_IRI, b"https://in-toto.io/Statement/v1");
}

#[test]
fn admits_slsa_provenance_v1_predicate() {
    // SLSA Provenance v1 — common predicateType in the in-toto
    // attestation ecosystem.
    let raw = minimal_statement(
        "https://slsa.dev/provenance/v1",
        r#"{
            "buildDefinition": {
                "buildType": "https://uor.foundation/build/v1",
                "externalParameters": {"source": "github.com/UOR-Foundation/uor-addr"},
                "internalParameters": {},
                "resolvedDependencies": []
            },
            "runDetails": {
                "builder": {"id": "https://uor.foundation/builders/v1"},
                "metadata": {"invocationId": "build-001", "startedOn": "2025-01-15T00:00:00Z"}
            }
        }"#,
    );
    address(&raw).expect("SLSA Provenance v1 admits");
}

#[test]
fn admits_slsa_verification_summary_v1_predicate() {
    let raw = minimal_statement(
        "https://slsa.dev/verification_summary/v1",
        r#"{
            "verifier": {"id": "https://uor.foundation/verifier/v1"},
            "timeVerified": "2025-01-15T00:00:00Z",
            "resourceUri": "https://example.org/artifact",
            "policy": {"uri": "https://uor.foundation/policy/v1"},
            "verificationResult": "PASSED",
            "verifiedLevels": ["SLSA_BUILD_LEVEL_2"]
        }"#,
    );
    address(&raw).expect("SLSA VSA admits");
}

#[test]
fn admits_scai_v0_predicate() {
    // SCAI (Software Component Attestation Information) predicate type.
    let raw = minimal_statement(
        "https://in-toto.io/attestation/scai/attribute-report/v0.2",
        r#"{
            "attributes": [
                {"attribute": "WITH_PATCH", "evidence": {"name": "patch.diff"}}
            ]
        }"#,
    );
    address(&raw).expect("SCAI v0.2 predicate admits");
}

#[test]
fn admits_sbom_predicate() {
    // SPDX SBOM as a predicate.
    let raw = minimal_statement(
        "https://spdx.dev/Document",
        r#"{
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": "uor-addr-sbom",
            "documentNamespace": "https://uor.foundation/sbom/uor-addr-v0.1.0"
        }"#,
    );
    address(&raw).expect("SPDX SBOM predicate admits");
}

#[test]
fn admits_multiple_subjects() {
    let raw = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [
            {"name": "artifact-a", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}},
            {"name": "artifact-b", "digest": {"sha256": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"}},
            {"name": "artifact-c", "digest": {"sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"}}
        ],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {}
    }"#;
    address(raw).expect("multi-subject admits");
}

#[test]
fn admits_subject_with_multiple_digest_algorithms() {
    // in-toto Statement v1 admits multiple digest algorithms per
    // subject; we require sha256 but other keys may coexist.
    let raw = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{
            "name": "x",
            "digest": {
                "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "sha512": "f1f2f3f4",
                "sha1": "deadbeef",
                "md5": "cafebabe"
            }
        }],
        "predicateType": "https://slsa.dev/provenance/v1",
        "predicate": {}
    }"#;
    address(raw).expect("multi-algorithm digest admits");
}

#[test]
fn rejects_wrong_statement_type_iri() {
    for bad_iri in [
        "https://in-toto.io/Statement/v0.1", // older draft
        "https://in-toto.io/Link/v1",        // wrong document type
        "https://example.org/CustomStatement",
        "", // empty IRI
    ] {
        let raw = alloc::format!(
            r#"{{"_type":"{bad_iri}","subject":[{{"name":"x","digest":{{"sha256":"{SHA256_HEX}"}}}}],"predicateType":"x","predicate":{{}}}}"#
        );
        match address(raw.as_bytes()) {
            Err(AddressFailure::SchemaViolation) => {}
            other => panic!("expected SchemaViolation for _type={bad_iri:?}: {other:?}"),
        }
    }
}

#[test]
fn rejects_empty_subject_array() {
    let raw = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(raw) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_subject_missing_name() {
    let raw = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(raw) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_subject_with_empty_name() {
    let raw = alloc::format!(
        r#"{{"_type":"https://in-toto.io/Statement/v1","subject":[{{"name":"","digest":{{"sha256":"{SHA256_HEX}"}}}}],"predicateType":"x","predicate":{{}}}}"#
    );
    match address(raw.as_bytes()) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_subject_without_sha256() {
    let raw = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": "x", "digest": {"sha1": "deadbeef"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(raw) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_sha256_with_wrong_length() {
    for bad in [
        "tooshort",
        "0123456789abcdef", // 16 hex
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde", // 63 hex
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0", // 65 hex
    ] {
        let raw = alloc::format!(
            r#"{{"_type":"https://in-toto.io/Statement/v1","subject":[{{"name":"x","digest":{{"sha256":"{bad}"}}}}],"predicateType":"x","predicate":{{}}}}"#
        );
        match address(raw.as_bytes()) {
            Err(AddressFailure::SchemaViolation) => {}
            other => panic!("expected SchemaViolation for sha256={bad:?}: {other:?}"),
        }
    }
}

#[test]
fn rejects_sha256_with_non_hex_chars() {
    // 64 chars total but contains uppercase letters (out-of-charset).
    let raw: &[u8] = br#"{"_type":"https://in-toto.io/Statement/v1","subject":[{"name":"x","digest":{"sha256":"0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF"}}],"predicateType":"x","predicate":{}}"#;
    match address(raw) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation for uppercase hex: {other:?}"),
    }
}

#[test]
fn rejects_predicate_type_missing() {
    let raw = alloc::format!(
        r#"{{"_type":"https://in-toto.io/Statement/v1","subject":[{{"name":"x","digest":{{"sha256":"{SHA256_HEX}"}}}}],"predicate":{{}}}}"#
    );
    match address(raw.as_bytes()) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_predicate_type_empty_string() {
    let raw = alloc::format!(
        r#"{{"_type":"https://in-toto.io/Statement/v1","subject":[{{"name":"x","digest":{{"sha256":"{SHA256_HEX}"}}}}],"predicateType":"","predicate":{{}}}}"#
    );
    match address(raw.as_bytes()) {
        Err(AddressFailure::SchemaViolation) => {}
        other => panic!("expected SchemaViolation: {other:?}"),
    }
}

#[test]
fn rejects_predicate_not_object() {
    for bad_predicate in ["null", "[1,2,3]", r#""string""#, "42", "true"] {
        let raw = alloc::format!(
            r#"{{"_type":"https://in-toto.io/Statement/v1","subject":[{{"name":"x","digest":{{"sha256":"{SHA256_HEX}"}}}}],"predicateType":"x","predicate":{bad_predicate}}}"#
        );
        match address(raw.as_bytes()) {
            Err(AddressFailure::SchemaViolation) => {}
            other => panic!("expected SchemaViolation for predicate={bad_predicate}: {other:?}"),
        }
    }
}

#[test]
fn kappa_label_matches_json_realization() {
    let raw = minimal_statement("https://slsa.dev/provenance/v1", "{}");
    let from_signed = address(&raw).expect("κ-label").address;
    let from_json = uor_addr::json::address(&raw).expect("κ-label").address;
    assert_eq!(from_signed, from_json);
}

#[test]
fn typed_distinction_between_predicate_types() {
    // Different predicateType IRIs over the same subject yield
    // distinct κ-labels.
    let provenance = address(&minimal_statement("https://slsa.dev/provenance/v1", "{}"))
        .expect("κ-label")
        .address;
    let scai = address(&minimal_statement(
        "https://in-toto.io/attestation/scai/attribute-report/v0.2",
        "{}",
    ))
    .expect("κ-label")
    .address;
    assert_ne!(provenance, scai);
}

extern crate alloc;
