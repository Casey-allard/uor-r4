//! Fresh-process `uor-chat --stack` parity for the artifact-carried native Q31 state.
//!
//! The `uor-r4.lut-stack/12` artifact holds the exact 48-byte state (codex-1). This
//! control proves the product CLI loads it from the file with no side-load: two fresh
//! `uor-chat` processes serve the same tokens and `UR4SESS1` bytes, the in-process
//! library run on the same artifact matches them, and the same CLI on the artifact
//! without the state serves different tokens.
//!
//! Run with both features:
//! `cargo test -p uor-r4-integer --features serving-wasm,fbs-synthetic --test q31_artifact_fresh_process`.

#![cfg(all(feature = "serving-wasm", feature = "fbs-synthetic"))]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uor_r4_integer::session_codec;
use uor_r4_integer::stack::synthetic::synthetic_fbs_trunk;
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, export_fbs_trunk_uorlut01_with_q31, FbsExportSnaps,
    IntegerStackModel,
};

/// The codex-1 fixture: gate `[2008113292, 336469910, 0, 0]`, epoch 64, step shift 8.
const TRAINED_STATE_HEX: &str = concat!(
    "8c60b17700000000961f0e1400000000",
    "00000000000000000000000000000000",
    "40000000000000000800000001000000",
);
const PROMPT_IDS: &[u32] = &[3, 5, 8, 13];
const MAX_TOKENS: usize = 4;

fn trained_state() -> [u8; 48] {
    hex::decode(TRAINED_STATE_HEX)
        .expect("fixture hex")
        .try_into()
        .expect("48-byte trained state")
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Deterministic byte-level BPE `tokenizer.json` with `vocab` pieces, matching the
/// synthetic stack's vocabulary. BOS 0, EOS 1, UNK 2 bind
/// `uor-r4.literal-role-dialogue/1`; raw `--prompt-ids` never encode text.
fn build_synthetic_tokenizer_json(vocab: usize) -> Vec<u8> {
    let mut map = serde_json::Map::with_capacity(vocab);
    for (idx, piece) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        map.insert((*piece).to_string(), json!(idx as u32));
    }
    for idx in 3..vocab {
        map.insert(format!("t{idx}"), json!(idx as u32));
    }
    let doc = json!({
        "model": {
            "type": "BPE",
            "vocab": map,
            "merges": []
        },
        "added_tokens": [
            { "id": 0, "content": "<|bos|>" },
            { "id": 1, "content": "<|eos|>" },
            { "id": 2, "content": "<|unk|>" }
        ],
        "pre_tokenizer": {
            "type": "ByteLevel",
            "add_prefix_space": false
        }
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
fn a_fresh_uor_chat_process_serves_the_artifact_q31_state() {
    let dir = std::env::temp_dir().join(format!("uor-q31-cli-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");

    let snap = synthetic_fbs_trunk(20_261_115);
    let q31 = export_fbs_trunk_uorlut01_with_q31(&snap, FbsExportSnaps::SNAPPED, &trained_state())
        .expect("export the /12 artifact");
    let bare = export_fbs_trunk_uorlut01(&snap).expect("export the plain artifact");
    let q31_path = dir.join("q31.lut");
    let bare_path = dir.join("bare.lut");
    let tokenizer_path: PathBuf = dir.join("tokenizer.json");
    fs::write(&q31_path, &q31).expect("write q31 artifact");
    fs::write(&bare_path, &bare).expect("write bare artifact");
    fs::write(&tokenizer_path, build_synthetic_tokenizer_json(256)).expect("write tokenizer");

    let first = run_cli(&q31_path, &tokenizer_path, &dir.join("first.sess"));
    let second = run_cli(&q31_path, &tokenizer_path, &dir.join("second.sess"));
    assert_eq!(
        first, second,
        "two fresh uor-chat processes served different tokens or session bytes"
    );
    assert_eq!(
        fs::read(dir.join("first.sess")).expect("first session"),
        fs::read(dir.join("second.sess")).expect("second session"),
        "two fresh processes wrote different UR4SESS1 bytes"
    );

    let library = run_library(&q31);
    assert_eq!(
        first, library,
        "the CLI and the in-process library run on the same artifact differ"
    );

    let bare_run = run_cli(&bare_path, &tokenizer_path, &dir.join("bare.sess"));
    assert_ne!(
        bare_run.generated_ids, first.generated_ids,
        "the artifact-carried Q31 state did not change the CLI's served tokens"
    );

    let _ = fs::remove_dir_all(&dir);
}
