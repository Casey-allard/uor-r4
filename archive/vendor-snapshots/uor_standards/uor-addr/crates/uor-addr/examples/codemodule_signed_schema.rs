//! `uor-addr` — Signed-code-module schema descendant comprehensive
//! example.
//!
//! Demonstrates [`uor_addr::schema::codemodule_signed::address`] —
//! the schema-pinned descendant that **imports in-toto Statement
//! v1** over the JSON realization. The in-toto Statement is the
//! industry-standard envelope used by sigstore, SLSA, and the
//! broader software-supply-chain attestation ecosystem.
//!
//! Authoritative sources:
//! - in-toto Statement v1 — <https://in-toto.io/Statement/v1>
//! - in-toto Attestation Framework —
//!   <https://github.com/in-toto/attestation/blob/main/spec/v1/README.md>
//! - SLSA Provenance v1 — <https://slsa.dev/spec/v1.0/provenance>
//!
//! Run with `cargo run -p uor-addr --example codemodule_signed_schema`.

use uor_addr::schema::codemodule_signed::{
    address, AddressFailure, SignedCodeModuleValue, REQUIRED_PROPERTIES, SHA256_HEX_BYTES,
    STATEMENT_TYPE_IRI,
};

fn main() {
    println!("uor-addr — Signed-code-module descendant (imports in-toto Statement v1)\n");

    // 1. Schema-import surface.
    println!("1. Schema import");
    println!(
        "   _type IRI:          {}",
        std::str::from_utf8(STATEMENT_TYPE_IRI).expect("ASCII IRI")
    );
    println!("   required props:     {:?}", REQUIRED_PROPERTIES);
    println!("   subject digest:     sha256 (lowercase hex, {SHA256_HEX_BYTES} chars)");
    println!();

    // 2. Admission of a valid in-toto Statement v1 attestation
    //    carrying a SLSA Provenance v1 predicate.
    let valid = br#"{
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
            "buildDefinition": {
                "buildType": "https://uor.foundation/build/v1",
                "externalParameters": {"source": "github.com/UOR-Foundation/uor-addr"},
                "resolvedDependencies": []
            },
            "runDetails": {
                "builder": {"id": "https://uor.foundation/builders/v1"},
                "metadata": {"invocationId": "build-001"}
            }
        }
    }"#;
    let outcome = address(valid).expect("valid in-toto Statement");
    let typed = SignedCodeModuleValue::parse(valid).expect("typed statement");
    println!("2. Admission of valid in-toto v1 Statement (with SLSA Provenance v1 predicate)");
    println!("   tagged bytes len: {} bytes", typed.tagged_bytes().len());
    println!("   κ-label:          {}\n", outcome.address);

    // 3. Multi-subject statement.
    let multi = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [
            {"name": "module-a", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}},
            {"name": "module-b", "digest": {"sha256": "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"}}
        ],
        "predicateType": "https://example.org/predicates/v1",
        "predicate": {"summary": "two related modules"}
    }"#;
    let outcome = address(multi).expect("multi-subject Statement");
    println!("3. Multi-subject Statement");
    println!("   κ-label: {}\n", outcome.address);

    // 4. κ-label equivalence with JSON realization.
    let from_signed = address(valid).expect("κ-label").address;
    let from_json = uor_addr::json::address(valid).expect("κ-label").address;
    assert_eq!(from_signed, from_json);
    println!("4. κ-label matches the JSON realization");
    println!("   match: {} ✓\n", from_signed == from_json);

    // 5. Rejection cases.
    println!("5. Schema-violation rejections");

    // 5a. Wrong _type IRI.
    let wrong_type = br#"{
        "_type": "https://example.org/CustomStatement",
        "subject": [{"name": "x", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(wrong_type) {
        Err(AddressFailure::SchemaViolation) => println!("   wrong _type IRI rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5b. Empty subject array.
    let empty_subject = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(empty_subject) {
        Err(AddressFailure::SchemaViolation) => println!("   empty subject[] rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5c. Subject without sha256 digest.
    let no_sha256 = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": "x", "digest": {"md5": "deadbeef"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(no_sha256) {
        Err(AddressFailure::SchemaViolation) => println!("   subject without sha256 rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5d. Truncated sha256 digest.
    let short_hex = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": "x", "digest": {"sha256": "tooshort"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    match address(short_hex) {
        Err(AddressFailure::SchemaViolation) => println!("   truncated sha256 rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    // 5e. Missing predicateType.
    let no_predicate_type = br#"{
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{"name": "x", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
        "predicate": {}
    }"#;
    match address(no_predicate_type) {
        Err(AddressFailure::SchemaViolation) => println!("   missing predicateType rejected ✓"),
        other => panic!("expected SchemaViolation: {other:?}"),
    }

    println!("\nOK — in-toto v1 Statement import discipline pinned at admission boundary.");
}
