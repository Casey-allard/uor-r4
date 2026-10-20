//! Closed-loop (CL-GGUF) conformance for the GGUF v3 realization.
//!
//! These vectors synthesize minimal-but-valid GGUF v3 byte buffers
//! in-process (no external fixtures), then assert the realization's
//! canonical-form invariants:
//!
//! - every well-formed input yields a 71-byte `sha256:<64hex>` κ-label;
//! - the κ-label is deterministic;
//! - it is invariant under metadata-KV and tensor reordering and under
//!   tensor-data relayout (the canonical form sorts + recomputes
//!   offsets);
//! - it is sensitive to tensor-data content (weights bind via streamed
//!   digests);
//! - malformed inputs (bad magic, wrong version, deprecated dtype) are
//!   rejected.

#![cfg(feature = "gguf")]

use uor_addr::gguf::{self, AddressFailure};

const GGUF_MAGIC: u32 = 0x4655_4747;
const ALIGN: u64 = 32;

// GGUF metadata value type tags.
const T_STRING: u32 = 8;
const T_UINT32: u32 = 4;

const GGML_F32: u32 = 0;
const GGML_Q4_2_DEPRECATED: u32 = 4;

#[derive(Clone)]
struct Kv {
    key: Vec<u8>,
    type_tag: u32,
    value: Vec<u8>, // wire-encoded value payload
}

#[derive(Clone)]
struct Tensor {
    name: Vec<u8>,
    dims: Vec<u64>,
    ggml_type: u32,
    data: Vec<u8>,
}

fn str_value(s: &str) -> Vec<u8> {
    let mut v = (s.len() as u64).to_le_bytes().to_vec();
    v.extend_from_slice(s.as_bytes());
    v
}

fn u32_value(n: u32) -> Vec<u8> {
    n.to_le_bytes().to_vec()
}

fn align_up(o: usize, a: usize) -> usize {
    o.div_ceil(a) * a
}

/// Serialize a GGUF v3 file. `kvs` and `tensors` are emitted in the given
/// order (so we can test reorder-invariance); tensor data is laid out
/// sequentially with alignment padding and the offsets recorded into the
/// tensor-info section.
fn build(kvs: &[Kv], tensors: &[Tensor]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    out.extend_from_slice(&3u32.to_le_bytes());
    out.extend_from_slice(&(tensors.len() as u64).to_le_bytes());
    out.extend_from_slice(&(kvs.len() as u64).to_le_bytes());

    for kv in kvs {
        out.extend_from_slice(&(kv.key.len() as u64).to_le_bytes());
        out.extend_from_slice(&kv.key);
        out.extend_from_slice(&kv.type_tag.to_le_bytes());
        out.extend_from_slice(&kv.value);
    }

    // Compute per-tensor data offsets (relative to the aligned data
    // section start), packing in emission order with alignment padding.
    let mut offsets = Vec::new();
    let mut cursor = 0usize;
    for t in tensors {
        offsets.push(cursor as u64);
        cursor = align_up(cursor + t.data.len(), ALIGN as usize);
    }

    for (t, off) in tensors.iter().zip(&offsets) {
        out.extend_from_slice(&(t.name.len() as u64).to_le_bytes());
        out.extend_from_slice(&t.name);
        out.extend_from_slice(&(t.dims.len() as u32).to_le_bytes());
        for d in &t.dims {
            out.extend_from_slice(&d.to_le_bytes());
        }
        out.extend_from_slice(&t.ggml_type.to_le_bytes());
        out.extend_from_slice(&off.to_le_bytes());
    }

    // Pad to the alignment boundary, then emit tensor data at the
    // recorded offsets.
    let data_start = align_up(out.len(), ALIGN as usize);
    out.resize(data_start, 0);
    for (t, off) in tensors.iter().zip(&offsets) {
        let at = data_start + *off as usize;
        if out.len() < at + t.data.len() {
            out.resize(at + t.data.len(), 0);
        }
        out[at..at + t.data.len()].copy_from_slice(&t.data);
    }
    out
}

fn f32_tensor(name: &str, vals: &[f32]) -> Tensor {
    let mut data = Vec::new();
    for v in vals {
        data.extend_from_slice(&v.to_le_bytes());
    }
    Tensor {
        name: name.as_bytes().to_vec(),
        dims: vec![vals.len() as u64],
        ggml_type: GGML_F32,
        data,
    }
}

fn label(bytes: &[u8]) -> String {
    let outcome = gguf::address(bytes).expect("valid gguf");
    outcome.address.as_str().to_string()
}

fn is_kappa(s: &str) -> bool {
    s.len() == 71
        && s.starts_with("sha256:")
        && s[7..]
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn sample_kvs() -> Vec<Kv> {
    vec![
        Kv {
            key: b"general.architecture".to_vec(),
            type_tag: T_STRING,
            value: str_value("llama"),
        },
        Kv {
            key: b"general.name".to_vec(),
            type_tag: T_STRING,
            value: str_value("tiny"),
        },
        Kv {
            key: b"general.alignment".to_vec(),
            type_tag: T_UINT32,
            value: u32_value(ALIGN as u32),
        },
    ]
}

fn sample_tensors() -> Vec<Tensor> {
    vec![
        f32_tensor("token_embd.weight", &[1.0, 2.0, 3.0, 4.0]),
        f32_tensor("output.weight", &[5.0, 6.0]),
    ]
}

#[test]
fn well_formed_yields_kappa_label() {
    let bytes = build(&sample_kvs(), &sample_tensors());
    let l = label(&bytes);
    assert!(is_kappa(&l), "not a 71-byte sha256:<64hex>: {l}");
}

#[test]
fn label_is_deterministic() {
    let bytes = build(&sample_kvs(), &sample_tensors());
    assert_eq!(label(&bytes), label(&bytes));
}

#[test]
fn invariant_under_metadata_reorder() {
    let kvs = sample_kvs();
    let mut reordered = kvs.clone();
    reordered.reverse();
    let a = label(&build(&kvs, &sample_tensors()));
    let b = label(&build(&reordered, &sample_tensors()));
    assert_eq!(a, b, "metadata KV order must not affect the κ-label");
}

#[test]
fn invariant_under_tensor_reorder_and_relayout() {
    let tensors = sample_tensors();
    let mut reordered = tensors.clone();
    reordered.reverse(); // different stored offsets + emission order
    let a = label(&build(&sample_kvs(), &tensors));
    let b = label(&build(&sample_kvs(), &reordered));
    assert_eq!(
        a, b,
        "tensor order / data layout must not affect the κ-label"
    );
}

#[test]
fn sensitive_to_tensor_data() {
    let base = label(&build(&sample_kvs(), &sample_tensors()));
    let mut perturbed = sample_tensors();
    perturbed[0] = f32_tensor("token_embd.weight", &[1.0, 2.0, 3.0, 4.5]); // one weight changed
    let other = label(&build(&sample_kvs(), &perturbed));
    assert_ne!(base, other, "changing a weight must change the κ-label");
}

#[test]
fn sensitive_to_metadata_value() {
    let base = label(&build(&sample_kvs(), &sample_tensors()));
    let mut kvs = sample_kvs();
    kvs[1].value = str_value("small"); // general.name changed
    let other = label(&build(&kvs, &sample_tensors()));
    assert_ne!(base, other, "changing metadata must change the κ-label");
}

#[test]
fn rejects_bad_magic() {
    let mut bytes = build(&sample_kvs(), &sample_tensors());
    bytes[0] ^= 0xFF;
    assert_eq!(
        gguf::address(&bytes).unwrap_err(),
        AddressFailure::InvalidGguf
    );
}

#[test]
fn rejects_wrong_version() {
    let mut bytes = build(&sample_kvs(), &sample_tensors());
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes()); // version = 2
    assert_eq!(
        gguf::address(&bytes).unwrap_err(),
        AddressFailure::InvalidGguf
    );
}

#[test]
fn rejects_deprecated_dtype() {
    let mut t = f32_tensor("w", &[1.0]);
    t.ggml_type = GGML_Q4_2_DEPRECATED;
    // (data length need not match a real Q4_2 block for this rejection —
    // the deprecated type ID is refused before any data read.)
    let bytes = build(&sample_kvs(), &[t]);
    assert_eq!(
        gguf::address(&bytes).unwrap_err(),
        AddressFailure::InvalidGguf
    );
}
