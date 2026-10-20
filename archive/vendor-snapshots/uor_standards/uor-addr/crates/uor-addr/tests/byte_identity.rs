//! Byte-identity tests: reproduce the 12 reference fixtures harvested
//! from the [UOR Foundation](https://uor.foundation) canonical
//! endpoint `mcp.uor.foundation/tools/encode_address` (mcp-uor-server
//! v0.2.1, algorithm `uor-sha256-v1`, canonicalisation
//! `jcs-rfc8785+nfc`) through the pure-Prism `AddressModel` pipeline
//! declared in this crate.
//!
//! Two layers of assertion:
//!
//! 1. [`shim_layer_reproduces_harvested_fixtures`] — `pipeline::address`
//!    routes through `AddressModel::forward` → ψ-chain → ψ_9 resolver →
//!    `H::initial().fold_bytes(canonical).finalize()` and returns the
//!    same 71-byte address the reference endpoint produced.
//!
//! 2. [`canonicalize_kernel_matches_expected_canonical_form`] — the
//!    in-surface canonicalizer (reached via [`canonicalize`])
//!    produces the expected canonical-form bytes for each fixture.

use uor_addr::json::{address, canonicalize};

struct Fixture {
    name: &'static str,
    /// Raw JSON input bytes the host provides at the boundary.
    raw_json: &'static [u8],
    expected_address: &'static str,
    expected_canonical_form: &'static [u8],
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "simple_object",
            raw_json: br#"{"foo": "bar"}"#,
            expected_address:
                "sha256:7a38bf81f383f69433ad6e900d35b3e2385593f76a7b7ab5d4355b8ba41ee24b",
            expected_canonical_form: br#"{"foo":"bar"}"#,
        },
        Fixture {
            name: "empty_object",
            raw_json: b"{}",
            expected_address:
                "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            expected_canonical_form: b"{}",
        },
        Fixture {
            name: "empty_array",
            raw_json: b"[]",
            expected_address:
                "sha256:4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945",
            expected_canonical_form: b"[]",
        },
        Fixture {
            name: "key_sort_test",
            raw_json: br#"{"b": 1, "a": 2}"#,
            expected_address:
                "sha256:d3626ac30a87e6f7a6428233b3c68299976865fa5508e4267c5415c76af7a772",
            expected_canonical_form: br#"{"a":2,"b":1}"#,
        },
        Fixture {
            name: "unicode_muller",
            raw_json: "{\"name\": \"Müller\"}".as_bytes(),
            expected_address:
                "sha256:5e92260d138e28f7118a40c3c44be922d4569a39f9f1de676ca334cf19c3a37c",
            expected_canonical_form: "{\"name\":\"Müller\"}".as_bytes(),
        },
        Fixture {
            name: "unicode_sao_paulo",
            raw_json: "{\"city\": \"São Paulo\"}".as_bytes(),
            expected_address:
                "sha256:0906524dcbec3bdb6aa4d7c22ed65d671ba32177b10823b6f256546280aa526b",
            expected_canonical_form: "{\"city\":\"São Paulo\"}".as_bytes(),
        },
        Fixture {
            name: "unicode_cafe_composed",
            // U+00E9 — single composed code point for é.
            raw_json: "{\"name\": \"caf\u{00E9}\"}".as_bytes(),
            expected_address:
                "sha256:645fa443126a8954fc6d871912b8fc67bc2ee8feae417efe55546251962ca74d",
            expected_canonical_form: "{\"name\":\"café\"}".as_bytes(),
        },
        Fixture {
            name: "unicode_cafe_decomposed",
            // U+0065 U+0301 — 'e' + combining acute. Must produce the SAME
            // address as the composed form after NFC normalisation.
            raw_json: "{\"name\": \"cafe\u{0301}\"}".as_bytes(),
            expected_address:
                "sha256:645fa443126a8954fc6d871912b8fc67bc2ee8feae417efe55546251962ca74d",
            expected_canonical_form: "{\"name\":\"café\"}".as_bytes(),
        },
        Fixture {
            name: "mixed_types",
            raw_json: br#"{"int": 42, "bool": true, "null_val": null}"#,
            expected_address:
                "sha256:0966918f3851b97071b0e04d2576a2a86a197e71072b07061c8bfdaa6b6a5d2c",
            expected_canonical_form: br#"{"bool":true,"int":42,"null_val":null}"#,
        },
        Fixture {
            name: "nested",
            raw_json: br#"{"nested": {"deep": {"value": "found"}}}"#,
            expected_address:
                "sha256:b18dce7d3cbf2ac3b908ad75c8420c4a42077192df69dffdbf786755724eff1d",
            expected_canonical_form: br#"{"nested":{"deep":{"value":"found"}}}"#,
        },
        Fixture {
            name: "string_array",
            raw_json: br#"["a", "b", "c"]"#,
            expected_address:
                "sha256:fa1844c2988ad15ab7b49e0ece09684500fad94df916859fb9a43ff85f5bb477",
            expected_canonical_form: br#"["a","b","c"]"#,
        },
        Fixture {
            name: "number_array",
            raw_json: br#"[1, 2, 3]"#,
            expected_address:
                "sha256:a615eeaee21de5179de080de8c3052c8da901138406ba71c38c032845f7d54f4",
            expected_canonical_form: b"[1,2,3]",
        },
    ]
}

#[test]
fn shim_layer_reproduces_harvested_fixtures() {
    let mut failures = Vec::new();
    for fixture in fixtures() {
        match address(fixture.raw_json) {
            Ok(outcome) => {
                if outcome.address != fixture.expected_address {
                    failures.push(format!(
                        "[{}] address mismatch:\n  expected: {}\n  got:      {}",
                        fixture.name, fixture.expected_address, outcome.address
                    ));
                }
            }
            Err(e) => failures.push(format!("[{}] pipeline error: {:?}", fixture.name, e)),
        }
    }
    assert!(
        failures.is_empty(),
        "{} byte-identity failure(s) against UOR-ADDR reference fixtures:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn canonicalize_kernel_matches_expected_canonical_form() {
    let mut failures = Vec::new();
    for fixture in fixtures() {
        match canonicalize(fixture.raw_json) {
            Ok(canon) => {
                if canon != fixture.expected_canonical_form {
                    failures.push(format!(
                        "[{}] canonical-form mismatch:\n  expected: {:?}\n  got:      {:?}",
                        fixture.name,
                        String::from_utf8_lossy(fixture.expected_canonical_form),
                        String::from_utf8_lossy(&canon),
                    ));
                }
            }
            Err(e) => failures.push(format!("[{}] canonicalize error: {:?}", fixture.name, e)),
        }
    }
    assert!(
        failures.is_empty(),
        "{} canonical-form failure(s):\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn pipeline_rejects_invalid_json() {
    match address(b"not json") {
        Err(uor_addr::json::AddressFailure::InvalidJson) => {}
        other => panic!("expected InvalidJson, got {:?}", other),
    }
}

#[test]
fn pipeline_address_is_seventy_one_ascii_bytes() {
    let outcome = address(br#"{"foo":"bar"}"#).expect("valid JSON");
    assert_eq!(outcome.address.len(), 71);
    assert!(outcome.address.is_ascii());
    assert!(outcome.address.starts_with("sha256:"));
    // The 64 hex digits must all be lowercase.
    assert!(outcome.address[7..]
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn pipeline_witness_recovers_kappa_label() {
    // The AddressWitness carried inside AddressOutcome::witness is
    // foundation-sealed (constructible only via PrismModel::forward) and
    // its replay must recover the exact 71-byte κ-label.
    let outcome = address(br#"{"foo":"bar"}"#).expect("valid JSON");
    let label = outcome.witness.kappa_label();
    assert_eq!(label.len(), 71);
    assert_eq!(label, outcome.address);
    // verify() replays the derivation and returns the recovered κ-label.
    let recovered = outcome.witness.verify().expect("replay");
    assert_eq!(recovered, outcome.address);
}

#[test]
fn pipeline_distinct_inputs_yield_distinct_addresses() {
    let a = address(br#"{"foo":"bar"}"#).unwrap().address;
    let b = address(br#"{"foo":"baz"}"#).unwrap().address;
    assert_ne!(a, b);
}

#[test]
fn pipeline_key_order_invariant() {
    // Semantically-equal JSON values with different key orderings must
    // produce identical addresses.
    let a = address(br#"{"b": 1, "a": 2}"#).unwrap().address;
    let b = address(br#"{"a": 2, "b": 1}"#).unwrap().address;
    assert_eq!(a, b);
}

#[test]
fn pipeline_nfc_invariant() {
    // composed "café" (U+00E9) vs decomposed "cafe\u0301" — must collapse
    // to the same content address after NFC normalisation.
    let composed = address("{\"name\":\"caf\u{00E9}\"}".as_bytes())
        .unwrap()
        .address;
    let decomposed = address("{\"name\":\"cafe\u{0301}\"}".as_bytes())
        .unwrap()
        .address;
    assert_eq!(composed, decomposed);
}
