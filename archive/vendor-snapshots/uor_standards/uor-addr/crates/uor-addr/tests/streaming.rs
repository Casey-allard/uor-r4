//! **CM-STREAM — the ADR-060 streaming / bounded-carrier proof.**
//!
//! GGUF and ONNX canonicalize to a *flat Merkle skeleton* whose
//! variable-length leaves (tensor data, large array/string payloads) are
//! replaced by their streamed SHA-256 digest. This file proves, without
//! any network download, the two properties that matter for arbitrarily
//! large models:
//!
//!   * **Bounded carrier** — the canonical skeleton (the ψ-pipeline
//!     carrier) has a size determined only by the tensor / KV / node
//!     *counts*, never by the tensor-*data* size. A model with a 64 MiB
//!     weight tensor skeletonizes to exactly as many bytes as one with a
//!     1 KiB tensor of the same rank. (The 531 MB → 28 KB real-model
//!     vector lives in the `UOR_ADDR_LIVE` `external_models` suite.)
//!   * **Every byte binds** — flipping a single byte anywhere in the
//!     (large) tensor-data region changes the κ-label, so the streamed
//!     per-tensor digest binds the full weights, not a capped prefix.
//!
//! Plus determinism across repeated calls.

#![cfg(all(feature = "gguf", feature = "onnx"))]

const LARGE: usize = 64 << 20; // 64 MiB of tensor data — genuinely streamed.
const SMALL: usize = 1 << 10;

// ─── GGUF v3 builder: one F32 tensor "w" with `data_len` bytes of data ──

fn build_gguf(data_len: usize) -> Vec<u8> {
    assert_eq!(data_len % 4, 0);
    let n_elements = (data_len / 4) as u64; // F32 = 4 bytes/element
    let mut v = Vec::new();
    v.extend_from_slice(&0x4655_4747u32.to_le_bytes()); // "GGUF"
    v.extend_from_slice(&3u32.to_le_bytes()); // version
    v.extend_from_slice(&1u64.to_le_bytes()); // tensor_count
    v.extend_from_slice(&0u64.to_le_bytes()); // kv_count
                                              // tensor info: name "w", 1 dim, type 0 (F32), offset 0.
    v.extend_from_slice(&1u64.to_le_bytes());
    v.push(b'w');
    v.extend_from_slice(&1u32.to_le_bytes()); // n_dims
    v.extend_from_slice(&n_elements.to_le_bytes()); // dim
    v.extend_from_slice(&0u32.to_le_bytes()); // GGML_TYPE_F32
    v.extend_from_slice(&0u64.to_le_bytes()); // stored offset (data-section relative)
                                              // pad to the 32-byte default alignment, then the tensor data.
    while v.len() % 32 != 0 {
        v.push(0);
    }
    v.resize(v.len() + data_len, 0xAB);
    v
}

// ─── Minimal protobuf + ONNX ModelProto builder ─────────────────────────

fn pb_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8 & 0x7F) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}
fn pb_varint_field(out: &mut Vec<u8>, field: u64, v: u64) {
    pb_varint(out, field << 3); // wire type 0
    pb_varint(out, v);
}
fn pb_len_field(out: &mut Vec<u8>, field: u64, bytes: &[u8]) {
    pb_varint(out, (field << 3) | 2); // wire type 2
    pb_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// ONNX `ModelProto` (IR 13, one default-domain opset@13) with a single
/// FLOAT initializer `w` carrying `data_len` bytes of `raw_data`.
fn build_onnx(data_len: usize) -> Vec<u8> {
    let raw_data = vec![0xCDu8; data_len];

    let mut opset = Vec::new();
    pb_varint_field(&mut opset, 2, 13); // version (domain "" omitted => default)

    let mut tensor = Vec::new();
    pb_varint_field(&mut tensor, 1, (data_len / 4) as u64); // dims[0] (repeated int64)
    pb_varint_field(&mut tensor, 2, 1); // data_type = FLOAT
    pb_len_field(&mut tensor, 8, b"w"); // name
    pb_len_field(&mut tensor, 9, &raw_data); // raw_data

    let mut graph = Vec::new();
    pb_len_field(&mut graph, 2, b"g"); // name
    pb_len_field(&mut graph, 5, &tensor); // initializer

    let mut model = Vec::new();
    pb_varint_field(&mut model, 1, 13); // ir_version
    pb_len_field(&mut model, 8, &opset);
    pb_len_field(&mut model, 7, &graph);
    model
}

// ─── GGUF ────────────────────────────────────────────────────────────────

#[test]
fn gguf_carrier_size_is_independent_of_tensor_data_size() {
    let small = uor_addr::gguf::canonicalize(&build_gguf(SMALL)).expect("small");
    let large = uor_addr::gguf::canonicalize(&build_gguf(LARGE)).expect("large");
    assert_eq!(
        small.len(),
        large.len(),
        "GGUF skeleton size must not grow with tensor-data size"
    );
    // And it is genuinely tiny: header + one tensor record, no KVs.
    assert!(
        small.len() < 256,
        "skeleton {} unexpectedly large",
        small.len()
    );
}

#[test]
fn gguf_every_tensor_byte_binds() {
    let model = build_gguf(LARGE);
    let base = uor_addr::gguf::address(&model).unwrap().address;
    // Flip a byte deep in the (aligned) tensor-data region.
    let mut flipped = model.clone();
    let mid = flipped.len() / 2;
    flipped[mid] ^= 0xFF;
    let other = uor_addr::gguf::address(&flipped).unwrap().address;
    assert_ne!(base, other, "a deep tensor-data byte must bind into κ");
    // Last byte too.
    let mut last = model.clone();
    *last.last_mut().unwrap() ^= 0x01;
    assert_ne!(base, uor_addr::gguf::address(&last).unwrap().address);
}

#[test]
fn gguf_large_model_is_deterministic() {
    let model = build_gguf(LARGE);
    assert_eq!(
        uor_addr::gguf::address(&model).unwrap().address,
        uor_addr::gguf::address(&model).unwrap().address
    );
}

// ─── ONNX ──────────────────────────────────────────────────────────────

#[test]
fn onnx_carrier_size_is_independent_of_tensor_data_size() {
    let small = uor_addr::onnx::canonicalize(&build_onnx(SMALL)).expect("small");
    let large = uor_addr::onnx::canonicalize(&build_onnx(LARGE)).expect("large");
    assert_eq!(
        small.len(),
        large.len(),
        "ONNX skeleton size must not grow with raw_data size"
    );
    assert!(
        small.len() < 512,
        "skeleton {} unexpectedly large",
        small.len()
    );
}

#[test]
fn onnx_every_tensor_byte_binds() {
    let model = build_onnx(LARGE);
    let base = uor_addr::onnx::address(&model).unwrap().address;
    let mut flipped = model.clone();
    let mid = flipped.len() / 2;
    flipped[mid] ^= 0xFF;
    assert_ne!(
        base,
        uor_addr::onnx::address(&flipped).unwrap().address,
        "a deep raw_data byte must bind into κ"
    );
}

#[test]
fn onnx_large_model_is_deterministic() {
    let model = build_onnx(LARGE);
    assert_eq!(
        uor_addr::onnx::address(&model).unwrap().address,
        uor_addr::onnx::address(&model).unwrap().address
    );
}
