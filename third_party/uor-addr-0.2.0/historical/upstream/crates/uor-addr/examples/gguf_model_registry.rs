//! `uor-addr` — GGUF model registry (content-addressed dedup + integrity).
//!
//! Real-world use case: a model registry / cache that keys GGUF v3
//! weight files by a **typed content address** instead of a filename or
//! an opaque file hash. The κ-label is computed over the GGUF
//! *canonical structural form* (ADR-046), so it is:
//!
//! - **dedup-stable** — two uploads of the same model that differ only
//!   in metadata-KV ordering or tensor-data padding collapse to one
//!   registry key (no duplicate storage);
//! - **integrity-bearing** — flipping a single weight changes the key,
//!   so a tampered or truncated download is detected on lookup;
//! - **σ-axis-selectable** — the same model can be keyed under
//!   `keccak256` for an on-chain registry while staying `sha256` for the
//!   local cache;
//! - **offline-verifiable** — the witness replays (TC-05) to the same
//!   key without re-reading the (large) tensor data.
//!
//! The GGUF bytes here are synthesized in-process (a minimal-but-valid
//! v3 file) so the example is self-contained.
//!
//! Run with `cargo run -p uor-addr --example gguf_model_registry`.

const GGUF_MAGIC: u32 = 0x4655_4747;
const ALIGN: usize = 32;
const T_STRING: u32 = 8;
const GGML_F32: u32 = 0;

/// A metadata key/value (value is the wire-encoded payload).
struct Kv {
    key: &'static str,
    type_tag: u32,
    value: Vec<u8>,
}

/// A named tensor with its raw little-endian f32 data.
struct Tensor {
    name: &'static str,
    data: Vec<u8>,
    dims: Vec<u64>,
}

fn gguf_string(s: &str) -> Vec<u8> {
    let mut v = (s.len() as u64).to_le_bytes().to_vec();
    v.extend_from_slice(s.as_bytes());
    v
}

fn f32_tensor(name: &'static str, vals: &[f32]) -> Tensor {
    let mut data = Vec::new();
    for v in vals {
        data.extend_from_slice(&v.to_le_bytes());
    }
    Tensor {
        name,
        data,
        dims: vec![vals.len() as u64],
    }
}

fn align_up(o: usize, a: usize) -> usize {
    o.div_ceil(a) * a
}

/// Serialize a GGUF v3 file. `kvs` and `tensors` are emitted in the given
/// order — so re-ordering them models two byte-different uploads of the
/// same model.
fn build(kvs: &[Kv], tensors: &[Tensor]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
    out.extend_from_slice(&3u32.to_le_bytes());
    out.extend_from_slice(&(tensors.len() as u64).to_le_bytes());
    out.extend_from_slice(&(kvs.len() as u64).to_le_bytes());

    for kv in kvs {
        out.extend_from_slice(&(kv.key.len() as u64).to_le_bytes());
        out.extend_from_slice(kv.key.as_bytes());
        out.extend_from_slice(&kv.type_tag.to_le_bytes());
        out.extend_from_slice(&kv.value);
    }

    let mut offsets = Vec::new();
    let mut cursor = 0usize;
    for t in tensors {
        offsets.push(cursor as u64);
        cursor = align_up(cursor + t.data.len(), ALIGN);
    }
    for (t, off) in tensors.iter().zip(&offsets) {
        out.extend_from_slice(&(t.name.len() as u64).to_le_bytes());
        out.extend_from_slice(t.name.as_bytes());
        out.extend_from_slice(&(t.dims.len() as u32).to_le_bytes());
        for d in &t.dims {
            out.extend_from_slice(&d.to_le_bytes());
        }
        out.extend_from_slice(&GGML_F32.to_le_bytes());
        out.extend_from_slice(&off.to_le_bytes());
    }

    let data_start = align_up(out.len(), ALIGN);
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

fn registry_key(model: &[u8]) -> String {
    uor_addr::gguf::address(model)
        .expect("valid GGUF v3 file")
        .address
        .as_str()
        .to_string()
}

fn main() {
    println!("uor-addr — GGUF model registry (content-addressed)\n");

    // ── The model: two metadata KVs + two weight tensors. ──
    let meta = || {
        vec![
            Kv {
                key: "general.architecture",
                type_tag: T_STRING,
                value: gguf_string("llama"),
            },
            Kv {
                key: "general.name",
                type_tag: T_STRING,
                value: gguf_string("demo-7b"),
            },
        ]
    };
    let weights = || {
        vec![
            f32_tensor("token_embd.weight", &[0.10, 0.20, 0.30, 0.40]),
            f32_tensor("output.weight", &[0.50, 0.60]),
        ]
    };

    let model = build(&meta(), &weights());
    let key = registry_key(&model);
    println!("  registry key:   {key}");
    println!("  (model is {} bytes)\n", model.len());

    // ── Dedup: the same model uploaded with metadata KVs in the other
    //    order is byte-different on the wire but addresses identically. ──
    let mut reordered_meta = meta();
    reordered_meta.reverse();
    let upload_2 = build(&reordered_meta, &weights());
    assert_ne!(model, upload_2, "the two uploads are byte-different");
    let key_2 = registry_key(&upload_2);
    assert_eq!(key, key_2, "canonical form dedups reordered metadata");
    println!("  re-uploaded with reordered metadata → same key (dedup):");
    println!("                  {key_2}\n");

    // ── Integrity: flip one weight → a different key (tamper detected). ──
    let mut tampered_weights = weights();
    tampered_weights[0] = f32_tensor("token_embd.weight", &[0.10, 0.20, 0.30, 0.41]);
    let tampered = build(&meta(), &tampered_weights);
    let tampered_key = registry_key(&tampered);
    assert_ne!(key, tampered_key, "a flipped weight must change the key");
    println!("  one weight flipped 0.40 → 0.41 → different key (integrity):");
    println!("                  {tampered_key}\n");

    // ── σ-axis: key the same model for an on-chain registry (keccak256). ──
    let onchain = uor_addr::gguf::address_keccak256(&model)
        .expect("valid GGUF")
        .address;
    assert!(onchain.as_str().starts_with("keccak256:"));
    println!("  on-chain key (keccak256 σ-axis):");
    println!("                  {onchain}\n");

    // ── Offline re-attestation: the witness replays to the same key
    //    without re-hashing the tensor data (TC-05). ──
    let attested = uor_addr::gguf::address(&model).expect("valid GGUF");
    let replayed = attested.witness.verify().expect("witness replays");
    assert_eq!(
        replayed, attested.address,
        "TC-05: witness replays to the registry key"
    );
    println!("  witness replay (TC-05) recovers the key offline: OK");

    println!("\nOK — content-addressed dedup, integrity, σ-axis, and replay all hold.");
}
