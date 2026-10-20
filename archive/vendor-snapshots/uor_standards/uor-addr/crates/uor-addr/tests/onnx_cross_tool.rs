//! CT-ONNX — cross-tool validation against
//! `mcp.uor.foundation/tools/encode_onnx_address`. Gated behind
//! `UOR_ADDR_LIVE=1`; requires network access.

#![cfg(feature = "onnx")]

fn live() -> bool {
    std::env::var("UOR_ADDR_LIVE").as_deref() == Ok("1")
}

#[test]
#[ignore = "CT: requires UOR_ADDR_LIVE=1 + network access to mcp.uor.foundation"]
fn mcp_endpoint_matches_rust() {
    if !live() {
        return;
    }
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/onnx/synthetic.onnx"
    ))
    .unwrap();
    let rust = uor_addr::onnx::address(&bytes)
        .unwrap()
        .address
        .as_str()
        .to_string();
    let resp: serde_json::Value =
        ureq::post("https://mcp.uor.foundation/tools/encode_onnx_address")
            .send_bytes(&bytes)
            .expect("POST to MCP endpoint")
            .into_json()
            .expect("JSON response");
    assert_eq!(resp["kappa_label"].as_str().unwrap(), rust);
}
