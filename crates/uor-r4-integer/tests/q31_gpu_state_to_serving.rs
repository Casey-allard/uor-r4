//! GPU training-to-serving control for the one native Q31 space.
//!
//! The AOT CUDA qualifier (`crates/uor-r4-discrete/examples/native_vector_qualify.rs`) trains the
//! 48-byte state on the RTX 4090 and writes it. This check requires those bytes to equal the Rust
//! oracle's, exports them into a `uor-r4.lut-stack/12` artifact, and serves them from a fresh
//! `uor-chat` process and the in-process model; the same trunk without the state must differ.
//!
//! Run after the GPU qualifier, with both features and the ignored test enabled:
//! `UOR_Q31_STATE=<file> cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic \
//!   --test q31_gpu_state_to_serving -- --ignored`.

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;
use uor_r4_integer::session_codec;
use uor_r4_integer::stack::synthetic::synthetic_fbs_trunk;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, export_fbs_trunk_uorlut01_with_q31, FbsExportSnaps,
    IntegerStackModel,
};
use uor_r4_native_vector::Q31HamiltonModel;

const PROMPT_IDS: &[u32] = &[3, 5, 8, 13];
const MAX_TOKENS: usize = 4;

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// A deterministic byte-level BPE `tokenizer.json` with `vocab` pieces, matching the
/// synthetic stack's vocabulary. BOS 0, EOS 1, UNK 2 bind the literal-role protocol;
/// raw `--prompt-ids` never encode text.
fn build_synthetic_tokenizer_json(vocab: usize) -> Vec<u8> {
    let mut map = serde_json::Map::with_capacity(vocab);
    for (idx, piece) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        map.insert((*piece).to_string(), json!(idx as u32));
    }
    for idx in 3..vocab {
        map.insert(format!("t{idx}"), json!(idx as u32));
    }
    let doc = json!({
        "model": { "type": "BPE", "vocab": map, "merges": [] },
        "added_tokens": [
            { "id": 0, "content": "<|bos|>" },
            { "id": 1, "content": "<|eos|>" },
            { "id": 2, "content": "<|unk|>" }
        ],
        "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": false }
    });
    serde_json::to_vec_pretty(&doc).expect("serialize the synthetic tokenizer.json")
}

#[derive(Debug, PartialEq, Eq)]
struct Run {
    generated_ids: Vec<u32>,
    last_token: u32,
    session_sha256: String,
}

fn run_cli(artifact: &Path, tokenizer: &Path, save: &Path) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--stack")
        .arg(artifact)
        .arg("--tokenizer")
        .arg(tokenizer)
        .arg("--prompt-ids")
        .arg(PROMPT_IDS.iter().map(u32::to_string).collect::<Vec<_>>().join(","))
        .arg("--max-tokens")
        .arg(MAX_TOKENS.to_string())
        .arg("--save-session")
        .arg(save)
        .output()
        .expect("spawn uor-chat");
    assert!(
        output.status.success(),
        "uor-chat failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record: Value = serde_json::from_slice(&output.stdout).expect("uor-chat JSON");
    assert_eq!(
        record["schema"].as_str(),
        Some("uor-r4.uor-chat-stack-parity/1")
    );
    Run {
        generated_ids: record["generated_ids"]
            .as_array()
            .expect("generated_ids")
            .iter()
            .map(|id| u32::try_from(id.as_u64().expect("id")).expect("u32 id"))
            .collect(),
        last_token: u32::try_from(record["last_token"].as_u64().expect("last_token"))
            .expect("u32 last token"),
        session_sha256: record["session_sha256"]
            .as_str()
            .expect("session_sha256")
            .to_owned(),
    }
}

fn run_library(artifact: &[u8]) -> Run {
    let model = IntegerStackModel::parse(artifact).expect("parse the artifact");
    let mut session = model.session();
    let mut tok = PROMPT_IDS[0];
    for &next in &PROMPT_IDS[1..] {
        session.step(tok).expect("prompt step");
        tok = next;
    }
    let mut generated_ids = Vec::with_capacity(MAX_TOKENS);
    for _ in 0..MAX_TOKENS {
        tok = session.serve_step(tok).expect("serve step").0;
        generated_ids.push(tok);
    }
    let bytes = session_codec::encode(&session.save_state());
    Run {
        generated_ids,
        last_token: tok,
        session_sha256: sha256_hex(&bytes),
    }
}

#[test]
#[ignore = "needs UOR_Q31_STATE written by the CUDA qualifier"]
fn the_gpu_trained_state_is_the_state_the_artifact_serves() {
    // 1. The state the AOT CUDA kernel wrote must equal the Rust oracle's 64-step state.
    let path = std::env::var("UOR_Q31_STATE")
        .expect("set UOR_Q31_STATE to the file native_vector_qualify wrote");
    let state: [u8; 48] = fs::read(&path)
        .expect("read the GPU state file")
        .try_into()
        .expect("48-byte GPU state");
    let one = 1_i64 << 31;
    let target = [1_518_500_249_i64, 1_518_500_249, 0, 0];
    let mut oracle = Q31HamiltonModel::new([one, 0, 0, 0], 8).expect("oracle model");
    let values = vec![[one, 0, 0, 0]; 8];
    let targets = vec![target; 8];
    for _ in 0..64 {
        oracle.step_reference(&values, &targets).expect("oracle step");
    }
    assert_eq!(
        state,
        oracle.to_bytes(),
        "the CUDA-trained state differs from the Rust oracle"
    );

    // 2. The artifact carries exactly the GPU bytes and the same gate/cursor.
    let snap = synthetic_fbs_trunk(20_261_115);
    let artifact = export_fbs_trunk_uorlut01_with_q31(&snap, FbsExportSnaps::SNAPPED, &state)
        .expect("export the GPU state");
    let model = IntegerStackModel::parse(&artifact).expect("parse the GPU artifact");
    assert_eq!(model.native_q31_state_bytes(), Some(state));
    let served = model.native_q31_state().expect("attached serving state");
    assert_eq!(served.gate(), oracle.gate());
    assert_eq!(served.epoch(), oracle.epoch());
    assert_eq!(served.step_shift(), oracle.step_shift());

    // 3. A fresh CLI process and the in-process model serve the trained state identically.
    let dir = std::env::temp_dir().join(format!("uor-q31-train-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    let artifact_path = dir.join("trained.lut");
    let tokenizer_path = dir.join("tokenizer.json");
    fs::write(&artifact_path, &artifact).expect("write trained artifact");
    fs::write(&tokenizer_path, build_synthetic_tokenizer_json(256)).expect("write tokenizer");

    let first = run_cli(&artifact_path, &tokenizer_path, &dir.join("first.sess"));
    let second = run_cli(&artifact_path, &tokenizer_path, &dir.join("second.sess"));
    assert_eq!(first, second, "two fresh processes differ on the trained state");
    assert_eq!(first, run_library(&artifact), "CLI differs from the library");

    // 4. The same trunk without the state serves different tokens.
    let bare = export_fbs_trunk_uorlut01(&snap).expect("export the plain artifact");
    let bare_path = dir.join("bare.lut");
    fs::write(&bare_path, &bare).expect("write bare artifact");
    let bare_run = run_cli(&bare_path, &tokenizer_path, &dir.join("bare.sess"));
    assert_ne!(
        bare_run.generated_ids, first.generated_ids,
        "the trained state did not change the served tokens"
    );

    let _ = fs::remove_dir_all(&dir);
}
