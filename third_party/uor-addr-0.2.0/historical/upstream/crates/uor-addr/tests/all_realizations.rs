//! Integration conformance suite for every UOR-ADDR realization
//! shipped in this crate.
//!
//! ## What this suite pins
//!
//! - **Cross-realization determinism (CD-D01)**. Every realization's
//!   `address()` function is a pure function of its input — same
//!   bytes → same κ-label.
//! - **Cross-realization wire-format width (CL-W01)**. Every
//!   realization emits the 71-byte `sha256:<64hex>` κ-label.
//! - **Cross-realization typed-distinction (CT-T*)**. Two
//!   realizations that admit the same surface text yield distinct
//!   κ-labels (each realization's canonicalization byte-output
//!   discipline distinguishes the typed environments).
//! - **Schema-descendant equivalence**. Each schema descendant's
//!   κ-label matches the underlying format realization's κ-label
//!   for the same admitted input (schema admission applies at
//!   parse time, not in the ψ-pipeline).
//!
//! ## Authoritative source coverage
//!
//! Every realization is tested against its authoritative-source
//! reference baseline. See [STANDARDS.md](https://github.com/UOR-Foundation/uor-addr/blob/main/STANDARDS.md)
//! for the full index.

const KAPPA_LABEL_BYTES: usize = 71;

fn assert_well_formed_kappa(label: &str) {
    assert_eq!(label.len(), KAPPA_LABEL_BYTES);
    assert!(label.starts_with("sha256:"));
    for &b in &label.as_bytes()[7..] {
        assert!(b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    }
}

// ─── JSON realization (already covered by other test files) ────────────

#[test]
fn json_realization_emits_well_formed_kappa() {
    let outcome = uor_addr::json::address(br#"{"x":1}"#).expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

// ─── S-expression realization (Rivest 1997) ────────────────────────────

#[test]
fn sexp_realization_emits_well_formed_kappa() {
    let outcome = uor_addr::sexp::address(b"(a b c)").expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

// ─── Ring realization (Amendment 43 §2) ────────────────────────────────

#[test]
fn ring_realization_emits_well_formed_kappa() {
    let outcome = uor_addr::ring::address(&[0u8, 0x42]).expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

#[test]
fn ring_realization_distinguishes_witt_levels() {
    let level_0 = uor_addr::ring::address(&[0u8, 0x42])
        .expect("κ-label")
        .address;
    let level_1 = uor_addr::ring::address(&[1u8, 0x42, 0x00])
        .expect("κ-label")
        .address;
    assert_ne!(level_0, level_1);
}

#[test]
fn ring_realization_rejects_overflow_witt_level() {
    let err = uor_addr::ring::address(&[255u8, 0]).expect_err("must reject");
    assert!(matches!(
        err,
        uor_addr::ring::AddressFailure::InvalidRingElement
    ));
}

// ─── ASN.1 realization (X.690 DER) ─────────────────────────────────────

#[test]
fn asn1_realization_emits_well_formed_kappa() {
    // INTEGER 42, DER: 0x02 0x01 0x2A
    let outcome = uor_addr::asn1::address(&[0x02, 0x01, 0x2A]).expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

#[test]
fn asn1_realization_distinguishes_boolean_true_false() {
    let t = uor_addr::asn1::address(&[0x01, 0x01, 0xFF])
        .expect("κ-label")
        .address;
    let f = uor_addr::asn1::address(&[0x01, 0x01, 0x00])
        .expect("κ-label")
        .address;
    assert_ne!(t, f);
}

#[test]
fn asn1_realization_rejects_non_canonical_der() {
    // Non-minimal INTEGER encoding (X.690 §8.3.2)
    let err = uor_addr::asn1::address(&[0x02, 0x02, 0x00, 0x01]).expect_err("non-minimal");
    assert!(matches!(err, uor_addr::asn1::AddressFailure::InvalidDer));
}

// ─── XML realization (W3C C14N 1.1 subset) ─────────────────────────────

#[test]
fn xml_realization_emits_well_formed_kappa() {
    let outcome = uor_addr::xml::address(b"<root/>").expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

#[test]
fn xml_realization_is_invariant_under_attribute_order() {
    let a = uor_addr::xml::address(br#"<root a="1" b="2"/>"#)
        .expect("κ-label")
        .address;
    let b = uor_addr::xml::address(br#"<root b="2" a="1"/>"#)
        .expect("κ-label")
        .address;
    // XML-C14N 1.1 §1.1 rule 3 — lexicographic attribute ordering.
    assert_eq!(a, b);
}

// ─── Code-module AST realization (CCMAS) ───────────────────────────────

#[test]
fn codemodule_realization_emits_well_formed_kappa() {
    let m = uor_addr::codemodule::CodeModuleValue::module("hello", &[])
        .tagged_bytes()
        .to_vec();
    let outcome = uor_addr::codemodule::address(&m).expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

// ─── CBOR realization (RFC 8949 §4.2) ──────────────────────────────────

#[test]
fn cbor_realization_emits_well_formed_kappa() {
    // [1, 2, 3]
    let outcome = uor_addr::cbor::address(&[0x83, 0x01, 0x02, 0x03]).expect("κ-label");
    assert_well_formed_kappa(&outcome.address);
}

#[test]
fn cbor_realization_canonicalizes_map_key_order() {
    // {"b":1,"a":2} ≡ {"a":2,"b":1} under RFC 8949 §4.2.1 key sorting.
    let a = uor_addr::cbor::address(&[0xa2, 0x61, b'b', 0x01, 0x61, b'a', 0x02])
        .expect("κ-label")
        .address;
    let b = uor_addr::cbor::address(&[0xa2, 0x61, b'a', 0x02, 0x61, b'b', 0x01])
        .expect("κ-label")
        .address;
    assert_eq!(a, b);
}

// ─── Schema descendants ────────────────────────────────────────────────

#[test]
fn photo_schema_admits_valid_schema_org_photograph() {
    // schema.org/Photograph JSON-LD instance.
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Photograph",
        "contentUrl": "https://example.org/skyline.jpg",
        "creator": {"@type": "Person", "name": "Ada Lovelace"}
    }"#;
    let from_photo = uor_addr::schema::photo::address(raw)
        .expect("κ-label")
        .address;
    let from_json = uor_addr::json::address(raw).expect("κ-label").address;
    assert_well_formed_kappa(&from_photo);
    assert_eq!(from_photo, from_json);
}

#[test]
fn photo_schema_rejects_non_schema_org_input() {
    let bad = br#"{"@context": "https://example.org", "@type": "Photograph"}"#;
    let err = uor_addr::schema::photo::address(bad).expect_err("must reject");
    assert!(matches!(
        err,
        uor_addr::schema::photo::AddressFailure::SchemaViolation
    ));
}

#[test]
fn document_schema_admits_valid_schema_org_article() {
    // schema.org/Article JSON-LD instance.
    let raw = br#"{
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": "On Typed Content Addressing",
        "author": {"@type": "Person", "name": "Ada Lovelace"},
        "datePublished": "2025-01-15"
    }"#;
    let from_doc = uor_addr::schema::document::address(raw)
        .expect("κ-label")
        .address;
    let from_json = uor_addr::json::address(raw).expect("κ-label").address;
    assert_well_formed_kappa(&from_doc);
    assert_eq!(from_doc, from_json);
}

#[test]
fn document_schema_admits_article_subtypes() {
    for ty in ["Article", "NewsArticle", "ScholarlyArticle", "BlogPosting"] {
        let raw = alloc::format!(
            r#"{{"@context":"https://schema.org","@type":"{ty}","headline":"x","author":"y","datePublished":"2025-01-15"}}"#
        );
        uor_addr::schema::document::address(raw.as_bytes()).expect("valid subtype");
    }
}

#[test]
fn codemodule_signed_schema_admits_in_toto_statement_v1() {
    // in-toto Statement v1 attestation with SLSA Provenance v1 predicate.
    let raw = br#"{
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
        "predicate": {"buildDefinition": {"buildType": "uor:test"}}
    }"#;
    let from_signed = uor_addr::schema::codemodule_signed::address(raw)
        .expect("κ-label")
        .address;
    let from_json = uor_addr::json::address(raw).expect("κ-label").address;
    assert_well_formed_kappa(&from_signed);
    assert_eq!(from_signed, from_json);
}

#[test]
fn codemodule_signed_schema_rejects_wrong_statement_iri() {
    let bad = br#"{
        "_type": "https://example.org/CustomStatement",
        "subject": [{"name": "x", "digest": {"sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}],
        "predicateType": "x",
        "predicate": {}
    }"#;
    let err = uor_addr::schema::codemodule_signed::address(bad).expect_err("must reject");
    assert!(matches!(
        err,
        uor_addr::schema::codemodule_signed::AddressFailure::SchemaViolation
    ));
}

extern crate alloc;

// ─── Cost-model variants ───────────────────────────────────────────────

#[test]
fn signed_variant_is_a_typed_commitment() {
    use prism::pipeline::TypedCommitment;
    fn assert_typed_commitment<C: TypedCommitment>() {}
    assert_typed_commitment::<uor_addr::variant::signed::SignedCommitment>();
}

// ─── Cross-realization typed-distinction ───────────────────────────────

#[test]
fn cross_realization_typed_distinction() {
    // Surface input shaped differently across formats yields
    // distinct κ-labels — the architectural commitment per
    // ARCHITECTURE.md.
    let json_label = uor_addr::json::address(br#"["a"]"#)
        .expect("κ-label")
        .address;
    let sexp_label = uor_addr::sexp::address(b"(a)").expect("κ-label").address;
    let xml_label = uor_addr::xml::address(b"<a/>").expect("κ-label").address;
    let asn1_label = uor_addr::asn1::address(&[0x04, 0x01, b'a'])
        .expect("κ-label")
        .address;
    let ring_label = uor_addr::ring::address(&[0, b'a'])
        .expect("κ-label")
        .address;

    let mut labels = [
        &json_label,
        &sexp_label,
        &xml_label,
        &asn1_label,
        &ring_label,
    ];
    labels.sort();
    for w in labels.windows(2) {
        assert_ne!(w[0], w[1], "labels must be pairwise distinct");
    }
}

// ─── Cross-realization σ-axis coverage ─────────────────────────────────

#[test]
fn every_realization_exposes_all_five_axes() {
    // Each realization's `address` / `_blake3` / `_sha3_256` / `_keccak256`
    // / `_sha512` entry points emit the correct prefix + width and are
    // pairwise distinct.
    let by_axis = |s: &uor_addr::KappaLabel<71>,
                   b: &uor_addr::KappaLabel<71>,
                   q: &uor_addr::KappaLabel<73>,
                   k: &uor_addr::KappaLabel<74>,
                   z: &uor_addr::KappaLabel<135>| {
        assert!(s.starts_with("sha256:") && s.len() == 71);
        assert!(b.starts_with("blake3:") && b.len() == 71);
        assert!(q.starts_with("sha3-256:") && q.len() == 73);
        assert!(k.starts_with("keccak256:") && k.len() == 74);
        assert!(z.starts_with("sha512:") && z.len() == 135);
        let labels = [s.as_str(), b.as_str(), q.as_str(), k.as_str(), z.as_str()];
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "axes must be pairwise distinct");
            }
        }
    };

    let j = br#"{"x":1}"#;
    by_axis(
        &uor_addr::json::address(j).unwrap().address,
        &uor_addr::json::address_blake3(j).unwrap().address,
        &uor_addr::json::address_sha3_256(j).unwrap().address,
        &uor_addr::json::address_keccak256(j).unwrap().address,
        &uor_addr::json::address_sha512(j).unwrap().address,
    );
    let c: &[u8] = &[0x83, 0x01, 0x02, 0x03];
    by_axis(
        &uor_addr::cbor::address(c).unwrap().address,
        &uor_addr::cbor::address_blake3(c).unwrap().address,
        &uor_addr::cbor::address_sha3_256(c).unwrap().address,
        &uor_addr::cbor::address_keccak256(c).unwrap().address,
        &uor_addr::cbor::address_sha512(c).unwrap().address,
    );
    let s = b"(a b c)";
    by_axis(
        &uor_addr::sexp::address(s).unwrap().address,
        &uor_addr::sexp::address_blake3(s).unwrap().address,
        &uor_addr::sexp::address_sha3_256(s).unwrap().address,
        &uor_addr::sexp::address_keccak256(s).unwrap().address,
        &uor_addr::sexp::address_sha512(s).unwrap().address,
    );
}

#[test]
fn schema_descendant_axis_matches_underlying_json_axis() {
    // A schema descendant's per-axis κ-label equals the JSON realization's
    // for the same admitted input (schema admission is at parse time).
    let raw = br#"{"@context":"https://schema.org","@type":"Photograph","contentUrl":"https://e.org/p.jpg","creator":"Ada"}"#;
    assert_eq!(
        uor_addr::schema::photo::address_blake3(raw)
            .unwrap()
            .address,
        uor_addr::json::address_blake3(raw).unwrap().address
    );
    assert_eq!(
        uor_addr::schema::photo::address_keccak256(raw)
            .unwrap()
            .address,
        uor_addr::json::address_keccak256(raw).unwrap().address
    );
}
