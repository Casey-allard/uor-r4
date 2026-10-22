//! Three-host M7 bit-identity and fresh-process `UR4SESS1` snapshot parity suite
//! across the Rust library (`IntegerStackModel` / `IntegerStackSession`), the
//! `uor-chat` CLI (`--stack`), and the Browser WASM / Studio Web Worker
//! (`m7-parity-library-cli-browser`, Issue #2867, Milestone #2035).

#![cfg(feature = "serving-wasm")]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uor_r4_integer::session_codec;
use uor_r4_integer::stack::{stack_argmax, IntegerStackModel};

const SYNTHETIC_SEED: u64 = 5;
const EXPECTED_ARTIFACT_BYTES: usize = 469_252;
const EXPECTED_ARTIFACT_SHA256: &str =
    "d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df";
const EXPECTED_WEIGHTS_PER_TOKEN: u64 = 15_968;

struct PromptCase {
    id: &'static str,
    prompt_ids: &'static [u32],
}

const M7_PARITY_PROMPTS: &[PromptCase] = &[
    PromptCase {
        id: "P0_bos",
        prompt_ids: &[0],
    },
    PromptCase {
        id: "P1_quad",
        prompt_ids: &[0, 7, 19, 31],
    },
    PromptCase {
        id: "P2_boundary",
        prompt_ids: &[0, 1, 38, 39],
    },
    PromptCase {
        id: "P3_fibonacci",
        prompt_ids: &[0, 1, 2, 3, 5, 8, 13, 21],
    },
    PromptCase {
        id: "P4_primes",
        prompt_ids: &[0, 2, 3, 5, 7, 11, 13, 17],
    },
    PromptCase {
        id: "P5_alternating",
        prompt_ids: &[0, 39, 1, 38, 2, 37, 3, 36],
    },
    PromptCase {
        id: "P6_repeated",
        prompt_ids: &[0, 17, 17, 17, 25, 25],
    },
    PromptCase {
        id: "P7_octave",
        prompt_ids: &[0, 4, 8, 12, 16, 20, 24, 28],
    },
];

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Deterministic 40-token byte-level BPE `tokenizer.json` matching the `vocab = 40`
/// synthetic stack (`mk-synthetic-stack`) and binding `uor-r4.literal-role-dialogue/1`.
fn build_synthetic_40_tokenizer_json() -> Vec<u8> {
    let pieces: [&str; 40] = [
        "<|bos|>", "<|eos|>", "<|unk|>", ":", "Ġ", "Ċ", "U", "s", "e", "r", "A", "i", "t", "a",
        "n", "S", "y", "m", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "b", "c", "d", "f",
        "g", "h", "k", "l", "o", "p", "q", "v",
    ];
    let mut vocab = serde_json::Map::with_capacity(pieces.len());
    for (idx, piece) in pieces.iter().enumerate() {
        vocab.insert((*piece).to_string(), json!(idx as u32));
    }
    let doc = json!({
        "model": {
            "type": "BPE",
            "vocab": vocab,
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
    serde_json::to_vec_pretty(&doc).expect("serialize 40-token tokenizer.json")
}

struct LibraryRunOut {
    generated_ids: Vec<u32>,
    last_token: u32,
    session_bytes: Vec<u8>,
    session_sha256: String,
}

fn run_library_sequence(
    model: &IntegerStackModel,
    restore_bytes: Option<&[u8]>,
    prompt_ids: &[u32],
    max_new_tokens: usize,
) -> Result<LibraryRunOut, String> {
    let mut session = model.session();
    if let Some(raw) = restore_bytes {
        let parsed = session_codec::decode(raw)?;
        session.restore_state(&parsed).map_err(|e| format!("{e}"))?;
    }
    let mut tok = prompt_ids[0];
    for &next_prompt_tok in &prompt_ids[1..] {
        session.step(tok).map_err(|e| format!("{e}"))?;
        tok = next_prompt_tok;
    }
    let mut generated_ids = Vec::with_capacity(max_new_tokens);
    for _ in 0..max_new_tokens {
        let logits = session.step(tok).map_err(|e| format!("{e}"))?;
        tok = stack_argmax(logits) as u32;
        generated_ids.push(tok);
    }
    let session_bytes = session_codec::encode(&session.save_state());
    let session_sha256 = sha256_hex(&session_bytes);
    Ok(LibraryRunOut {
        generated_ids,
        last_token: tok,
        session_bytes,
        session_sha256,
    })
}

struct CliRunOut {
    generated_ids: Vec<u32>,
    last_token: u32,
    session_bytes: Vec<u8>,
    session_sha256: String,
    weights_per_token: u64,
}

fn run_cli_sequence(
    artifact_path: &Path,
    tokenizer_path: &Path,
    restore_path: Option<&Path>,
    save_path: &Path,
    prompt_ids: &[u32],
    max_new_tokens: usize,
) -> Result<CliRunOut, String> {
    let uor_chat_bin = env!("CARGO_BIN_EXE_uor-chat");
    let csv = prompt_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let mut cmd = Command::new(uor_chat_bin);
    cmd.arg("--stack")
        .arg(artifact_path)
        .arg("--tokenizer")
        .arg(tokenizer_path)
        .arg("--prompt-ids")
        .arg(&csv)
        .arg("--max-tokens")
        .arg(max_new_tokens.to_string())
        .arg("--save-session")
        .arg(save_path);
    if let Some(restore) = restore_path {
        cmd.arg("--restore-session").arg(restore);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn uor-chat: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "uor-chat exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("invalid uor-chat JSON output: {e}"))?;
    if parsed["schema"].as_str() != Some("uor-r4.uor-chat-stack-parity/1") {
        return Err(format!("unexpected uor-chat schema: {}", parsed["schema"]));
    }
    let generated_ids = parsed["generated_ids"]
        .as_array()
        .ok_or_else(|| "missing generated_ids array".to_string())?
        .iter()
        .map(|v| {
            v.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| "non-u32 generated id".to_string())
        })
        .collect::<Result<Vec<u32>, String>>()?;
    let last_token = parsed["last_token"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| "missing last_token".to_string())?;
    let weights_per_token = parsed["weights_per_token"]
        .as_u64()
        .ok_or_else(|| "missing weights_per_token".to_string())?;
    let session_bytes =
        fs::read(save_path).map_err(|e| format!("failed to read saved CLI session: {e}"))?;
    let session_sha256 = sha256_hex(&session_bytes);
    if parsed["session_sha256"].as_str() != Some(session_sha256.as_str()) {
        return Err(format!(
            "CLI JSON session_sha256 ({:?}) != saved file sha256 ({session_sha256})",
            parsed["session_sha256"].as_str()
        ));
    }
    Ok(CliRunOut {
        generated_ids,
        last_token,
        session_bytes,
        session_sha256,
        weights_per_token,
    })
}

/// Pure comparison helper for three-leg token and snapshot parity (exercised directly
/// by both the positive 8-prompt panel test and the planted-fault tamper controls).
fn check_three_way_row_parity(
    prompt_id: &str,
    lib_tokens_48: &[u32],
    cli_tokens_48: &[u32],
    browser_tokens_48: &[u32],
    lib_sha32: &str,
    cli_sha32: &str,
    browser_sha32: &str,
    lib_sha48: &str,
    cli_sha48: &str,
    browser_sha48: &str,
    lib_restored_sha48: &str,
    cli_restored_sha48: &str,
    browser_restored_sha48: &str,
) -> Result<(), String> {
    if lib_tokens_48 != cli_tokens_48 {
        return Err(format!(
            "{prompt_id}: library vs uor-chat CLI 48-token trajectory mismatch"
        ));
    }
    if lib_tokens_48 != browser_tokens_48 {
        return Err(format!(
            "{prompt_id}: library vs browser WASM 48-token trajectory mismatch"
        ));
    }
    if lib_sha32 != cli_sha32 || lib_sha32 != browser_sha32 {
        return Err(format!(
            "{prompt_id}: 32-step UR4SESS1 snapshot sha256 mismatch (lib={lib_sha32}, cli={cli_sha32}, browser={browser_sha32})"
        ));
    }
    if lib_sha48 != cli_sha48 || lib_sha48 != browser_sha48 {
        return Err(format!(
            "{prompt_id}: 48-step UR4SESS1 snapshot sha256 mismatch (lib={lib_sha48}, cli={cli_sha48}, browser={browser_sha48})"
        ));
    }
    if lib_restored_sha48 != lib_sha48
        || cli_restored_sha48 != lib_sha48
        || browser_restored_sha48 != lib_sha48
    {
        return Err(format!(
            "{prompt_id}: 32+16 restored UR4SESS1 snapshot sha256 mismatch against uninterrupted 48-step snapshot"
        ));
    }
    Ok(())
}

fn make_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "uor-m7-parity-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn library_cli_and_browser_produce_bit_identical_tokens_and_snapshots_across_frozen_panel() {
    let temp_dir = make_temp_dir("panel");
    let artifact_path = temp_dir.join("synthetic-stack.lut");
    let tokenizer_path = temp_dir.join("tokenizer-40.json");

    // 1. Generate the seed-5 synthetic UORLUT01 artifact using `mk-synthetic-stack`.
    let mk_bin = env!("CARGO_BIN_EXE_mk-synthetic-stack");
    let mk_out = Command::new(mk_bin)
        .arg(SYNTHETIC_SEED.to_string())
        .arg(&artifact_path)
        .output()
        .expect("run mk-synthetic-stack");
    assert!(
        mk_out.status.success(),
        "mk-synthetic-stack failed: {}",
        String::from_utf8_lossy(&mk_out.stderr)
    );
    let artifact_bytes = fs::read(&artifact_path).expect("read synthetic-stack.lut");
    assert_eq!(artifact_bytes.len(), EXPECTED_ARTIFACT_BYTES);
    assert_eq!(sha256_hex(&artifact_bytes), EXPECTED_ARTIFACT_SHA256);

    let tokenizer_bytes = build_synthetic_40_tokenizer_json();
    fs::write(&tokenizer_path, &tokenizer_bytes).expect("write tokenizer-40.json");

    // 2. Run the Browser WASM + Studio Web Worker parity harness (`web/studio/verify_m7_parity.mjs`).
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let verify_mjs = repo_root.join("web/studio/verify_m7_parity.mjs");
    let browser_out = Command::new("node")
        .arg(&verify_mjs)
        .env("UOR_STACK_PATH", &artifact_path)
        .output()
        .expect("run node web/studio/verify_m7_parity.mjs");
    assert!(
        browser_out.status.success(),
        "verify_m7_parity.mjs failed: {}",
        String::from_utf8_lossy(&browser_out.stderr)
    );
    let browser_report: Value =
        serde_json::from_slice(&browser_out.stdout).expect("parse browser parity JSON");
    assert_eq!(
        browser_report["schema"].as_str(),
        Some("uor-r4.m7-browser-parity/1")
    );
    assert_eq!(
        browser_report["artifact_sha256"].as_str(),
        Some(EXPECTED_ARTIFACT_SHA256)
    );
    assert_eq!(
        browser_report["weights_per_token"].as_u64(),
        Some(EXPECTED_WEIGHTS_PER_TOKEN)
    );
    assert_eq!(browser_report["remote_requests"].as_u64(), Some(0));
    let browser_rows = browser_report["rows"]
        .as_array()
        .expect("browser rows array");
    assert_eq!(browser_rows.len(), M7_PARITY_PROMPTS.len());

    // 3. Load two independent library models (`model_a`, `model_b`) to verify fresh-instance restore.
    let model_a = IntegerStackModel::parse(&artifact_bytes).expect("parse model_a");
    let model_b = IntegerStackModel::parse(&artifact_bytes).expect("parse model_b");
    assert_eq!(model_a.weights_per_token(), EXPECTED_WEIGHTS_PER_TOKEN);

    for (idx, prompt) in M7_PARITY_PROMPTS.iter().enumerate() {
        // Leg 1: Rust library uninterrupted 48 tokens + 32+16 fresh-instance restore.
        let lib_full_48 = run_library_sequence(&model_a, None, prompt.prompt_ids, 48)
            .expect("lib uninterrupted 48");
        let lib_first_32 =
            run_library_sequence(&model_a, None, prompt.prompt_ids, 32).expect("lib first 32");
        let lib_cont_16 = run_library_sequence(
            &model_b,
            Some(&lib_first_32.session_bytes),
            &[lib_first_32.last_token],
            16,
        )
        .expect("lib cont 16");
        let mut lib_combined_48 = lib_first_32.generated_ids.clone();
        lib_combined_48.extend_from_slice(&lib_cont_16.generated_ids);
        assert_eq!(
            lib_combined_48, lib_full_48.generated_ids,
            "{}: library 32+16 tokens != uninterrupted 48",
            prompt.id
        );
        assert_eq!(
            lib_cont_16.session_bytes, lib_full_48.session_bytes,
            "{}: library 32+16 UR4SESS1 bytes != uninterrupted 48 bytes",
            prompt.id
        );

        // Leg 2: `uor-chat` CLI subprocess uninterrupted 48 tokens + 32+16 fresh-process restore.
        let cli_48_path = temp_dir.join(format!("{}_cli_48.sess", prompt.id));
        let cli_32_path = temp_dir.join(format!("{}_cli_32.sess", prompt.id));
        let cli_restored_48_path = temp_dir.join(format!("{}_cli_restored_48.sess", prompt.id));

        let cli_full_48 = run_cli_sequence(
            &artifact_path,
            &tokenizer_path,
            None,
            &cli_48_path,
            prompt.prompt_ids,
            48,
        )
        .expect("cli uninterrupted 48");
        let cli_first_32 = run_cli_sequence(
            &artifact_path,
            &tokenizer_path,
            None,
            &cli_32_path,
            prompt.prompt_ids,
            32,
        )
        .expect("cli first 32");
        let cli_cont_16 = run_cli_sequence(
            &artifact_path,
            &tokenizer_path,
            Some(&cli_32_path),
            &cli_restored_48_path,
            &[cli_first_32.last_token],
            16,
        )
        .expect("cli cont 16");
        assert_eq!(cli_full_48.weights_per_token, EXPECTED_WEIGHTS_PER_TOKEN);
        let mut cli_combined_48 = cli_first_32.generated_ids.clone();
        cli_combined_48.extend_from_slice(&cli_cont_16.generated_ids);
        assert_eq!(
            cli_combined_48, cli_full_48.generated_ids,
            "{}: CLI 32+16 tokens != uninterrupted 48",
            prompt.id
        );
        assert_eq!(
            cli_first_32.session_bytes, lib_first_32.session_bytes,
            "{}: CLI 32-step UR4SESS1 bytes != library bytes",
            prompt.id
        );
        assert_eq!(
            cli_full_48.session_bytes, lib_full_48.session_bytes,
            "{}: CLI 48-step UR4SESS1 bytes != library bytes",
            prompt.id
        );
        assert_eq!(
            cli_cont_16.session_bytes, lib_full_48.session_bytes,
            "{}: CLI restored 48-step UR4SESS1 bytes != library bytes",
            prompt.id
        );

        // Cross-host restore check: restore CLI process A's `cli_32.sess` into the Rust library
        // and verify its 16-token continuation and final 48-step snapshot are byte-identical.
        let cross_cont_16 = run_library_sequence(
            &model_b,
            Some(&cli_first_32.session_bytes),
            &[cli_first_32.last_token],
            16,
        )
        .expect("cross-host CLI->library cont 16");
        assert_eq!(cross_cont_16.generated_ids, lib_cont_16.generated_ids);
        assert_eq!(cross_cont_16.session_bytes, lib_full_48.session_bytes);

        // Leg 3: Browser WASM + Studio Web Worker row comparison.
        let brow = &browser_rows[idx];
        assert_eq!(brow["id"].as_str(), Some(prompt.id));
        let browser_tokens_48: Vec<u32> = brow["generated_48"]
            .as_array()
            .expect("browser generated_48")
            .iter()
            .map(|v| v.as_u64().expect("u64") as u32)
            .collect();
        let browser_sha32 = brow["session_32_sha256"].as_str().expect("browser_sha32");
        let browser_sha48 = brow["session_48_sha256"].as_str().expect("browser_sha48");
        let browser_restored_sha48 = brow["session_restored_48_sha256"]
            .as_str()
            .expect("browser_restored_sha48");
        assert_eq!(brow["worker_matches_wasm"].as_bool(), Some(true));

        check_three_way_row_parity(
            prompt.id,
            &lib_full_48.generated_ids,
            &cli_full_48.generated_ids,
            &browser_tokens_48,
            &lib_first_32.session_sha256,
            &cli_first_32.session_sha256,
            browser_sha32,
            &lib_full_48.session_sha256,
            &cli_full_48.session_sha256,
            browser_sha48,
            &lib_cont_16.session_sha256,
            &cli_cont_16.session_sha256,
            browser_restored_sha48,
        )
        .expect("three-way row parity must hold");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn planted_fault_controls_detect_token_and_snapshot_tampering() {
    let temp_dir = make_temp_dir("faults");
    let artifact_path = temp_dir.join("synthetic-stack.lut");
    let tokenizer_path = temp_dir.join("tokenizer-40.json");

    let mk_bin = env!("CARGO_BIN_EXE_mk-synthetic-stack");
    let mk_out = Command::new(mk_bin)
        .arg(SYNTHETIC_SEED.to_string())
        .arg(&artifact_path)
        .output()
        .expect("run mk-synthetic-stack");
    assert!(mk_out.status.success());
    let artifact_bytes = fs::read(&artifact_path).expect("read artifact");
    fs::write(&tokenizer_path, build_synthetic_40_tokenizer_json()).expect("write tokenizer");

    let model = IntegerStackModel::parse(&artifact_bytes).expect("parse model");
    let baseline =
        run_library_sequence(&model, None, &[0, 7, 19, 31], 32).expect("baseline 32 steps");

    // 1. Planted fault: tamper with a single token in the CLI or browser trajectory.
    let mut tampered_tokens = baseline.generated_ids.clone();
    tampered_tokens[15] ^= 1;
    assert!(
        check_three_way_row_parity(
            "P1_quad",
            &baseline.generated_ids,
            &tampered_tokens,
            &baseline.generated_ids,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
        )
        .is_err(),
        "planted token fault in CLI leg must be rejected"
    );
    assert!(
        check_three_way_row_parity(
            "P1_quad",
            &baseline.generated_ids,
            &baseline.generated_ids,
            &tampered_tokens,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
            &baseline.session_sha256,
        )
        .is_err(),
        "planted token fault in browser leg must be rejected"
    );

    // 2. Planted fault: flip one hex character of `artifact_sha256` inside a UR4SESS1 snapshot.
    let mut state = session_codec::decode(&baseline.session_bytes).expect("decode valid snapshot");
    let mut bad_sha = state.artifact_sha256.clone();
    let replacement = if bad_sha.starts_with('0') { "1" } else { "0" };
    bad_sha.replace_range(0..1, replacement);
    state.artifact_sha256 = bad_sha;
    let bad_sha_bytes = session_codec::encode(&state);
    let mut fresh_session = model.session();
    assert!(
        fresh_session
            .restore_state(&session_codec::decode(&bad_sha_bytes).expect("decode"))
            .is_err(),
        "library restore_state must reject UR4SESS1 snapshot with mismatched artifact_sha256"
    );

    let bad_sha_path = temp_dir.join("bad_sha.sess");
    let out_path = temp_dir.join("out.sess");
    fs::write(&bad_sha_path, &bad_sha_bytes).expect("write bad_sha.sess");
    assert!(
        run_cli_sequence(
            &artifact_path,
            &tokenizer_path,
            Some(&bad_sha_path),
            &out_path,
            &[baseline.last_token],
            4,
        )
        .is_err(),
        "uor-chat --restore-session must reject UR4SESS1 snapshot with mismatched artifact_sha256"
    );

    // 3. Planted fault: truncate a UR4SESS1 snapshot by 1 byte.
    let truncated = &baseline.session_bytes[..baseline.session_bytes.len() - 1];
    assert!(
        session_codec::decode(truncated).is_err(),
        "session_codec::decode must reject 1-byte truncated UR4SESS1 stream"
    );
    let trunc_path = temp_dir.join("truncated.sess");
    fs::write(&trunc_path, truncated).expect("write truncated.sess");
    assert!(
        run_cli_sequence(
            &artifact_path,
            &tokenizer_path,
            Some(&trunc_path),
            &out_path,
            &[baseline.last_token],
            4,
        )
        .is_err(),
        "uor-chat --restore-session must reject 1-byte truncated UR4SESS1 stream"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
