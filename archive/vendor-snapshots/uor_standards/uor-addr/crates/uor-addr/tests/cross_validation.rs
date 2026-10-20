//! CN — Network class: live cross-validation against
//! `mcp.uor.foundation/tools/encode_address`. Verifies our κ-label
//! matches the reference implementation byte-for-byte. See
//! [CONFORMANCE.md §CN](../../../CONFORMANCE.md#cn--network-class--cross-validation-against-reference).
//!
//! All tests are `#[ignore]` by default — they need network access.
//! Run via `just cn` (sets `UOR_ADDR_LIVE=1` and `cargo test --ignored`).

#![allow(non_snake_case)]

use uor_addr::json::address;

const ENDPOINT: &str = "https://mcp.uor.foundation/tools/encode_address";

/// 12 canonical fixtures from upstream — matches `byte_identity.rs`.
fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("simple_object", br#"{"foo": "bar"}"#.to_vec()),
        ("empty_object", b"{}".to_vec()),
        ("empty_array", b"[]".to_vec()),
        ("key_sort_test", br#"{"b": 1, "a": 2}"#.to_vec()),
        (
            "unicode_muller",
            "{\"name\": \"Müller\"}".as_bytes().to_vec(),
        ),
        (
            "unicode_sao_paulo",
            "{\"city\": \"São Paulo\"}".as_bytes().to_vec(),
        ),
        (
            "unicode_cafe_composed",
            "{\"name\": \"caf\u{00E9}\"}".as_bytes().to_vec(),
        ),
        (
            "unicode_cafe_decomposed",
            "{\"name\": \"cafe\u{0301}\"}".as_bytes().to_vec(),
        ),
        (
            "mixed_types",
            br#"{"int": 42, "bool": true, "null_val": null}"#.to_vec(),
        ),
        (
            "nested",
            br#"{"nested": {"deep": {"value": "found"}}}"#.to_vec(),
        ),
        ("string_array", br#"["a", "b", "c"]"#.to_vec()),
        ("number_array", br#"[1, 2, 3]"#.to_vec()),
    ]
}

/// CN-RC01 — 12 reference fixtures match the live endpoint.
#[test]
#[ignore]
fn cn_rc01__live_fixture_agreement() {
    require_live();
    let mut failures = Vec::new();
    for (name, raw) in fixtures() {
        let remote = match call_remote(&raw) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("[{name}] network error: {e}"));
                continue;
            }
        };
        let local = address(&raw).expect("valid").address;
        if local != remote.as_str() {
            failures.push(format!(
                "[{name}] mismatch:\n  remote: {remote}\n  local:  {local}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "CN-RC01: {} fixture(s) disagree with the live endpoint:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// CN-RC02 — 100 freshly-generated random JSON values agree.
#[test]
#[ignore]
fn cn_rc02__live_random_agreement() {
    require_live();
    let mut rng: u64 = 0x554F525F41444452;
    let mut failures = Vec::new();
    for i in 0..100 {
        // xorshift
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let raw = format!("{{\"k\":\"{rng:016x}\"}}");
        let raw_bytes = raw.as_bytes();
        let remote = match call_remote(raw_bytes) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("[#{i}] network error: {e}"));
                continue;
            }
        };
        let local = address(raw_bytes).expect("valid").address;
        if local != remote.as_str() {
            failures.push(format!(
                "[#{i}] mismatch on {raw}: remote={remote} local={local}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "CN-RC02: {} of 100 random samples disagree:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ───────────────────────────────────────────────────────────────────────────
// Plumbing
// ───────────────────────────────────────────────────────────────────────────

fn require_live() {
    if std::env::var_os("UOR_ADDR_LIVE").is_none() {
        panic!("CN tests are gated — set UOR_ADDR_LIVE=1 to run");
    }
}

fn call_remote(raw_json: &[u8]) -> Result<String, String> {
    // The mcp server exposes `tools/encode_address` accepting the raw JSON
    // value and returning `{"address": "sha256:..."}`. Adjust per the live
    // endpoint shape if it differs.
    let payload: serde_json::Value =
        serde_json::from_slice(raw_json).map_err(|e| format!("input is not valid JSON: {e}"))?;
    let body = serde_json::json!({ "value": payload });
    let resp = ureq::post(ENDPOINT)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("HTTP: {e}"))?;
    let json: serde_json::Value = resp.into_json().map_err(|e| format!("decode JSON: {e}"))?;
    json.get("address")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| format!("missing `address` field in response: {json}"))
}
