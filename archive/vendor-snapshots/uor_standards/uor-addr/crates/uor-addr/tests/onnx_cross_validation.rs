//! CN-ONNX — cross-validation against `tools/canonical-onnx.py`.
//! Gated behind `UOR_ADDR_LIVE=1`.

#![cfg(feature = "onnx")]

use std::process::Command;

fn live() -> bool {
    std::env::var("UOR_ADDR_LIVE").as_deref() == Ok("1")
}

#[test]
#[ignore = "CN: requires UOR_ADDR_LIVE=1 + python3"]
fn python_matches_rust_for_fixtures() {
    if !live() {
        return;
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/onnx");
    let fixtures = ["synthetic.onnx", "synthetic-typed.onnx"];
    for name in fixtures {
        let path = format!("{dir}/{name}");
        let bytes = std::fs::read(&path).unwrap();
        let rust = uor_addr::onnx::address(&bytes)
            .unwrap()
            .address
            .as_str()
            .to_string();
        let out = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tools/canonical-onnx.py"
            ))
            .arg(&path)
            .output()
            .expect("run canonical-onnx.py");
        assert!(
            out.status.success(),
            "python tool failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let py = String::from_utf8(out.stdout).unwrap().trim().to_string();
        assert_eq!(py, rust, "mismatch on {name}");
    }
}
