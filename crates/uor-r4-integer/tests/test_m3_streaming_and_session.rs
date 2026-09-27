//! Integration Tests for Milestone M3: Streaming API, Integer Sampling & Session Persistence.
//! File: crates/uor-r4-integer/tests/test_m3_streaming_and_session.rs

use std::fs;
use uor_r4_integer::{
    Bundle, ChatSession, ChatTokenStream, IncrementalUtf8Decoder, RoleToken, Sampler,
};

#[test]
fn test_m3_incremental_utf8_split_emoji_and_cjk() {
    let mut decoder = IncrementalUtf8Decoder::new();

    // 1. Multi-byte test cases:
    // - 4-byte emoji: "🦀" -> [0xF0, 0x9F, 0xA6, 0x80]
    // - 3-byte CJK: "語" -> [0xE8, 0xAA, 0x9E]
    // - 2-byte accent: "é" -> [0xC3, 0xA9]
    // - 1-byte ASCII: "!" -> [0x21]

    let crab_bytes = "🦀".as_bytes();
    assert_eq!(crab_bytes.len(), 4);

    // Push first byte only
    let chunk = decoder.push_bytes(&crab_bytes[0..1]);
    assert_eq!(
        chunk, None,
        "Incomplete 1st byte of 4-byte sequence must produce None"
    );
    assert_eq!(decoder.pending_bytes.len(), 1);

    // Push 2nd and 3rd bytes
    let chunk = decoder.push_bytes(&crab_bytes[1..3]);
    assert_eq!(
        chunk, None,
        "Incomplete 3 bytes of 4-byte sequence must produce None"
    );
    assert_eq!(decoder.pending_bytes.len(), 3);

    // Push 4th byte + start of CJK character (first 2 bytes of 3)
    let cjk_bytes = "語".as_bytes();
    assert_eq!(cjk_bytes.len(), 3);

    let mut mixed = vec![crab_bytes[3]];
    mixed.extend_from_slice(&cjk_bytes[0..2]);
    let chunk = decoder.push_bytes(&mixed);
    assert_eq!(
        chunk.as_deref(),
        Some("🦀"),
        "Completed crab emoji must be extracted, CJK bytes retained"
    );
    assert_eq!(decoder.pending_bytes.len(), 2);

    // Push remaining CJK byte + full 2-byte accent "é" + ASCII "!"
    let mut tail = vec![cjk_bytes[2]];
    tail.extend_from_slice("é!".as_bytes());
    let chunk = decoder.push_bytes(&tail);
    assert_eq!(
        chunk.as_deref(),
        Some("語é!"),
        "All completed multibyte and ASCII characters extracted"
    );
    assert!(
        decoder.pending_bytes.is_empty(),
        "Decoder pending bytes should now be empty"
    );

    // 2. Test flush with incomplete trailing bytes
    let chunk = decoder.push_bytes(&[0xF0, 0x9F]); // start of 4-byte emoji
    assert_eq!(chunk, None);
    let flushed = decoder.flush();
    assert_eq!(
        flushed.as_deref(),
        Some("\u{FFFD}"),
        "Flush must emit standard unicode replacement char for incomplete bytes"
    );
    assert!(decoder.pending_bytes.is_empty());
}

#[test]
fn test_m3_sampler_state_resumption_and_determinism() {
    let seed = 0xDEADBEEFCAFE1234;
    let mut sampler1 = Sampler::new(seed);

    // Advance sampler by 15 steps
    for _ in 0..15 {
        let _ = sampler1.xorshift64();
    }
    let saved_state = sampler1.state();
    assert_ne!(saved_state, seed, "State should advance after generation");

    // Generate next 20 pseudorandom values
    let mut expected_values = Vec::new();
    for _ in 0..20 {
        expected_values.push(sampler1.xorshift64());
    }

    // Restore second sampler from saved state
    let mut sampler2 = Sampler::from_state(saved_state);
    assert_eq!(sampler2.state(), saved_state);

    let mut actual_values = Vec::new();
    for _ in 0..20 {
        actual_values.push(sampler2.xorshift64());
    }

    assert_eq!(
        expected_values, actual_values,
        "Resumed sampler must generate exact bit-identical pseudorandom sequence"
    );

    // Verify serde JSON roundtrip
    let serialized = serde_json::to_string(&sampler1).expect("serialize sampler");
    let mut deserialized: Sampler = serde_json::from_str(&serialized).expect("deserialize sampler");
    assert_eq!(deserialized.state(), sampler1.state());

    let val1 = sampler1.xorshift64();
    let val2 = deserialized.xorshift64();
    assert_eq!(val1, val2, "Deserialized sampler matches active sampler");
}

#[test]
fn test_m3_sampler_zero_seed_protection() {
    // 0 seed must not cause xorshift64 fixed point loop (always 0)
    let mut sampler = Sampler::new(0);
    assert_ne!(
        sampler.state(),
        0,
        "Zero seed must be mapped to non-zero constant"
    );

    let val = sampler.xorshift64();
    assert_ne!(val, 0);

    // Set state to 0
    sampler.set_state(0);
    assert_ne!(
        sampler.state(),
        0,
        "set_state(0) must be mapped to non-zero constant"
    );
}

#[test]
fn test_m3_stop_token_suppression() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, None, 42).expect("session creation");

    // Test ChatTokenStream constructor and stop token list
    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];
    let stream = ChatTokenStream::new(&mut session, 100, &stop_tokens);

    assert_eq!(stream.tokens_generated(), 0);
    assert!(!stream.is_stopped());
    assert_eq!(stream.stop_reason(), None);
}

#[test]
fn test_m3_session_save_load_roundtrip() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, Some("You are a helpful assistant."), 42)
        .expect("session creation");

    assert!(session.telemetry().persistent_sealed);
    assert!(session.telemetry().persistent_slots_used > 0);

    // Ingest user prompt into dialogue memory
    let user_tokens = session
        .ingest_user_turn("Hello!")
        .expect("ingest user turn");
    assert!(user_tokens > 0);
    let pre_telemetry = session.telemetry();

    // Create temp directory for session file
    let temp_dir = std::env::temp_dir().join(format!("uor_m3_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("test_session.json");

    // Save session
    session
        .save_session_default(&session_path)
        .expect("save session");
    assert!(session_path.exists());

    // Verify file content schema
    let content = fs::read_to_string(&session_path).expect("read session json");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse json");
    assert_eq!(parsed["schema"], "uor-r4.integer-session/1");
    assert_eq!(parsed["bundle_sha256"], bundle.identity());
    assert_eq!(
        parsed["session_state"]["dialogue_seen"],
        pre_telemetry.dialogue_tokens_seen
    );
    assert_eq!(
        parsed["session_state"]["persistent_tokens"]
            .as_array()
            .unwrap()
            .len(),
        pre_telemetry.persistent_slots_used
    );

    // Load session back
    let loaded_session = bundle
        .load_chat_session(&session_path, bundle.identity())
        .expect("load session from path");

    let post_telemetry = loaded_session.telemetry();
    assert_eq!(
        pre_telemetry.persistent_slots_used,
        post_telemetry.persistent_slots_used
    );
    assert_eq!(
        pre_telemetry.persistent_sealed,
        post_telemetry.persistent_sealed
    );
    assert_eq!(
        pre_telemetry.dialogue_slots_used,
        post_telemetry.dialogue_slots_used
    );
    assert_eq!(
        pre_telemetry.dialogue_tokens_seen,
        post_telemetry.dialogue_tokens_seen
    );
    assert_eq!(
        pre_telemetry.dialogue_cursor,
        post_telemetry.dialogue_cursor
    );
    assert_eq!(
        pre_telemetry.cumulative_holonomy_q30,
        post_telemetry.cumulative_holonomy_q30
    );
    assert_eq!(pre_telemetry.zeta_phases, post_telemetry.zeta_phases);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_m3_session_mismatched_bundle_rejection() {
    let bundle = Bundle::synthetic_for_test();
    let session = ChatSession::new(&bundle, None, 42).expect("session creation");

    let temp_dir = std::env::temp_dir().join(format!("uor_m3_mismatch_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("mismatch_session.json");

    session
        .save_session_default(&session_path)
        .expect("save session");

    // Attempt to load with deliberate hash mismatch
    let result =
        ChatSession::load_session(&bundle, &session_path, "different_bundle_sha256_hash_here");

    assert!(
        result.is_err(),
        "Must reject load when expected bundle SHA-256 differs"
    );
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("bundle SHA-256 mismatch") || err_str.contains("mismatch"),
        "Error message must clearly identify bundle hash mismatch, got: {err_str}"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_m3_session_tampered_schema_rejection() {
    let bundle = Bundle::synthetic_for_test();
    let session = ChatSession::new(&bundle, None, 42).expect("session creation");

    let temp_dir = std::env::temp_dir().join(format!("uor_m3_tamper_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("tampered_session.json");

    session
        .save_session_default(&session_path)
        .expect("save session");

    // Tamper with schema version header
    let content = fs::read_to_string(&session_path).expect("read file");
    let tampered = content.replace("uor-r4.integer-session/1", "uor-r4.integer-session/999");
    fs::write(&session_path, tampered).expect("write tampered file");

    let result = ChatSession::load_session(&bundle, &session_path, bundle.identity());
    assert!(result.is_err(), "Must reject invalid schema header");
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("unsupported session schema"),
        "Error must explain unsupported schema, got: {err_str}"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
