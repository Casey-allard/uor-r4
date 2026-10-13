//! CN-GGUF — cross-validation against the spec-side Python canonical-form
//! encoder (`tools/canonical-gguf.py`).
//!
//! Gated behind `UOR_ADDR_LIVE=1` (run via `just cn`). For each committed
//! fixture, runs the Python encoder in a subprocess and asserts its
//! κ-label matches the Rust `uor_addr::gguf::address` label — the
//! independent-implementation attestation of the canonical form.

#![cfg(feature = "gguf")]

use std::process::Command;

fn live() -> bool {
    std::env::var("UOR_ADDR_LIVE").as_deref() == Ok("1")
}

fn python_label(fixture: &str) -> String {
    let out = Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tools/canonical-gguf.py"
        ))
        .arg(fixture)
        .output()
        .expect("run canonical-gguf.py");
    assert!(
        out.status.success(),
        "python tool failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
#[ignore = "CN: requires UOR_ADDR_LIVE=1 + python3"]
fn python_matches_rust_for_fixtures() {
    if !live() {
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gguf");
    for name in [
        "synthetic-f32.gguf",
        "empty-metadata.gguf",
        "aligned-256.gguf",
    ] {
        let path = format!("{dir}/{name}");
        let bytes = std::fs::read(&path).unwrap();
        let rust = uor_addr::gguf::address(&bytes)
            .unwrap()
            .address
            .as_str()
            .to_string();
        assert_eq!(python_label(&path), rust, "mismatch on {name}");
    }
}
