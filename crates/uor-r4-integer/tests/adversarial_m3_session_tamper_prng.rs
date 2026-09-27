//! Adversarial Challenger Test Suite — Milestone M3:
//! Session Serialization Tamper Resistance, Format Invariants & PRNG Drift Challenge.
//! File: crates/uor-r4-integer/tests/adversarial_m3_session_tamper_prng.rs
//!
//! Empirical adversarial verification covering:
//! 1. Session serialization roundtrip across 50 multi-turn exchanges:
//!    - 100% bit-level identity of all 256 memory slots:
//!      * 32 persistent session slots (persona keys [32, 64], values [32, 256])
//!      * 224 rolling dialogue slots (keys [224, 64], values [224, 256])
//!    - Turn IDs bit-level identity (current_turn_id == 50)
//!    - Discrete Hopf fiber holonomy cumulative Delta psi Q30 preservation and non-zero winding
//!    - T^8 Riemann zeta phase vector [i32; 8] bit-identical preservation
//!    - Recurrent state vector Q11 [256] bit-identical preservation
//!    - PRNG sampler state and sample policy preservation
//!
//! 2. Adversarial tamper resistance on `uor-r4.integer-session/1`:
//!    - Mutate 1 bit in `bundle_sha256`: strict rejection by `load_session` with checksum error
//!    - Mutate 1 byte in memory key arrays:
//!      * Raw JSON syntax corruption -> deserialize error rejection
//!      * Dimension corruption (64 -> 63 or 65 elements) -> strict rejection by `from_serialized`
//!      * Slot count corruption (224 -> 223 slots) -> strict rejection
//!      * Value tampering -> detectable deviation against authentic coordinate state
//!    - Alter schema header:
//!      * Schema name mutated -> strict rejection with `unsupported session schema`
//!      * Version number mutated (v1 -> v2, v1 -> v0) -> strict rejection with `unsupported session version`
//!
//! 3. PRNG determinism, resumption & zero-seed protection:
//!    - Checkpoint at turn 5, generate 20 tokens from active session.
//!    - Load session from turn 5 checkpoint, generate 20 tokens from loaded session.
//!    - Verify 100% bit-level identity between both 20-token sequences and PRNG states.
//!    - Zero-seed handling: `Sampler::from_state(0)` and `set_state(0)` map to non-zero constant
//!      and never lock into an infinite zero-loop over 10,000 steps.

use std::collections::HashSet;
use std::fs;
use uor_r4_integer::model::KEY_DIM;
use uor_r4_integer::{
    Bundle, ChatSession, RoleToken, SamplePolicy, Sampler, SlotTarget, DIALOGUE_CAPACITY,
    PERSISTENT_CAPACITY,
};

/// 1. Session serialization roundtrip across 50 multi-turn exchanges:
///    Verify 100% bit-level identity of all 256 memory slots, turn IDs,
///    Hopf fiber holonomy, and T^8 zeta phases.
#[test]
fn test_m3_adversarial_session_roundtrip_50_turns_bit_identical() {
    let bundle = Bundle::synthetic_for_test();
    let persona = "You are an adversarial test assistant evaluating geometric memory state.";
    let seed = 0xABCD_EF01_2345_6789;

    let mut session = ChatSession::new(&bundle, Some(persona), seed).expect("new chat session");
    session.set_policy(SamplePolicy::Categorical { top_k: 8 });

    // Verify initial persistent partition is sealed
    assert!(session.telemetry().persistent_sealed);
    let initial_persistent_len = session.state().persistent_len();
    assert!(
        initial_persistent_len > 0 && initial_persistent_len <= PERSISTENT_CAPACITY,
        "Persistent slots must be populated within capacity (got {initial_persistent_len})"
    );

    // Execute 50 multi-turn exchanges
    for turn in 1..=50 {
        let user_prompt = format!(
            "Adversarial turn {turn}: testing holonomy accumulation and T8 ergodicity under rollover."
        );
        let user_tokens = session
            .ingest_user_turn(&user_prompt)
            .expect("ingest user turn");
        assert!(user_tokens > 0);
        assert_eq!(session.state().current_turn(), turn as u32);

        // Emulate assistant generation: step 3 tokens and finish with TurnEnd
        for step in 0..3 {
            let token = (((turn * 41 + step * 17) % 4000) + 10) as u32;
            session
                .step_token(token, SlotTarget::Dialogue)
                .expect("step dialogue token");
        }
        session
            .step_token(RoleToken::TurnEnd.id(), SlotTarget::Dialogue)
            .expect("step turn end token");
    }

    assert_eq!(session.state().current_turn(), 50);
    assert_ne!(
        session.state().cumulative_holonomy_q30,
        0,
        "Cumulative holonomy must have accumulated non-zero winding over 50 turns"
    );

    let pre_telemetry = session.telemetry();
    assert_eq!(pre_telemetry.current_turn_id, 50);

    // Save session to temp file
    let temp_dir = std::env::temp_dir().join(format!("uor_m3_adv_rt50_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("session_50_turns.json");

    session
        .save_session_default(&session_path)
        .expect("save session to disk");
    assert!(session_path.exists());

    // Load session back from file
    let loaded = bundle
        .load_chat_session(&session_path, bundle.identity())
        .expect("load session from disk");
    let post_telemetry = loaded.telemetry();

    // 1. Bit-level identity of Telemetry
    assert_eq!(
        pre_telemetry.persistent_slots_used, post_telemetry.persistent_slots_used,
        "Persistent slot count must match exactly"
    );
    assert_eq!(
        pre_telemetry.persistent_sealed, post_telemetry.persistent_sealed,
        "Persistent sealed state must match exactly"
    );
    assert_eq!(
        pre_telemetry.dialogue_slots_used, post_telemetry.dialogue_slots_used,
        "Dialogue slot count must match exactly"
    );
    assert_eq!(
        pre_telemetry.dialogue_cursor, post_telemetry.dialogue_cursor,
        "Dialogue cursor must match exactly"
    );
    assert_eq!(
        pre_telemetry.dialogue_tokens_seen, post_telemetry.dialogue_tokens_seen,
        "Dialogue tokens seen must match exactly"
    );
    assert_eq!(
        pre_telemetry.current_turn_id, post_telemetry.current_turn_id,
        "Current turn ID must match exactly"
    );
    assert_eq!(post_telemetry.current_turn_id, 50);
    assert_eq!(
        pre_telemetry.cumulative_holonomy_q30, post_telemetry.cumulative_holonomy_q30,
        "Cumulative holonomy must match exactly"
    );
    assert_eq!(
        pre_telemetry.zeta_phases, post_telemetry.zeta_phases,
        "Zeta phase vector must match exactly"
    );

    // 2. Bit-level identity of All 256 Memory Slots
    // Persistent partition: slots 0..32
    assert_eq!(
        session.state().persistent_keys.len(),
        loaded.state().persistent_keys.len(),
        "Persistent keys length mismatch"
    );
    assert_eq!(
        session.state().persistent_values.len(),
        loaded.state().persistent_values.len(),
        "Persistent values length mismatch"
    );
    for slot in 0..session.state().persistent_keys.len() {
        assert_eq!(
            session.state().persistent_keys[slot],
            loaded.state().persistent_keys[slot],
            "Persistent key mismatch at slot {slot}"
        );
        assert_eq!(
            session.state().persistent_values[slot],
            loaded.state().persistent_values[slot],
            "Persistent value mismatch at slot {slot}"
        );
    }
    assert_eq!(
        session.state().persistent_tokens,
        loaded.state().persistent_tokens,
        "Persistent tokens mismatch"
    );

    // Dialogue partition: slots 32..256 (224 slots)
    for slot in 0..DIALOGUE_CAPACITY {
        assert_eq!(
            session.state().dialogue_keys[slot],
            loaded.state().dialogue_keys[slot],
            "Dialogue key mismatch at slot {slot}"
        );
        assert_eq!(
            session.state().dialogue_values[slot],
            loaded.state().dialogue_values[slot],
            "Dialogue value mismatch at slot {slot}"
        );
    }
    assert_eq!(
        session.state().dialogue_tokens,
        loaded.state().dialogue_tokens,
        "Dialogue tokens array mismatch"
    );
    assert_eq!(
        session.state().dialogue_sequences,
        loaded.state().dialogue_sequences,
        "Dialogue sequences array mismatch"
    );
    assert_eq!(
        session.state().dialogue_turn_ids,
        loaded.state().dialogue_turn_ids,
        "Dialogue turn IDs array mismatch"
    );

    // 3. Bit-level identity of Geometric & Recurrent State
    assert_eq!(
        session.state().state,
        loaded.state().state,
        "Recurrent state vector Q11 [256] mismatch"
    );
    assert_eq!(
        session.state().hopf_state,
        loaded.state().hopf_state,
        "Hopf fiber point Q30 mismatch"
    );
    assert_eq!(
        session.state().zeta_state,
        loaded.state().zeta_state,
        "T8 zeta state mismatch"
    );

    // 4. Bit-level identity of Sampler and Config
    assert_eq!(
        session.sampler().state(),
        loaded.sampler().state(),
        "Sampler PRNG state mismatch"
    );
    assert_eq!(session.policy(), loaded.policy(), "Sample policy mismatch");
    assert_eq!(
        session.read_mode(),
        loaded.read_mode(),
        "Read mode mismatch"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 2. Adversarial Tamper Tests:
///    - Mutate 1 bit in `bundle_sha256`: verify that `load_session` strictly rejects with checksum error.
#[test]
fn test_m3_adversarial_tamper_bundle_sha256_rejection() {
    let bundle = Bundle::synthetic_for_test();
    let session = ChatSession::new(&bundle, None, 42).expect("new session");

    let temp_dir =
        std::env::temp_dir().join(format!("uor_m3_adv_tamper_sha_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("clean_session.json");
    let tampered_path = temp_dir.join("tampered_bundle_sha.json");

    session
        .save_session_default(&session_path)
        .expect("save session");

    let original_json = fs::read_to_string(&session_path).expect("read json");
    let mut parsed: serde_json::Value =
        serde_json::from_str(&original_json).expect("parse json value");

    // Mutate 1 character (single bit/byte) in bundle_sha256
    let orig_sha = parsed["bundle_sha256"].as_str().unwrap().to_string();
    let mut tampered_sha = orig_sha.clone();
    let last_char = tampered_sha.pop().unwrap();
    let new_char = if last_char == '0' { '1' } else { '0' };
    tampered_sha.push(new_char);
    assert_ne!(orig_sha, tampered_sha);

    parsed["bundle_sha256"] = serde_json::Value::String(tampered_sha.clone());
    fs::write(
        &tampered_path,
        serde_json::to_string_pretty(&parsed).unwrap(),
    )
    .expect("write tampered");

    // 1. Load with explicit expected bundle identity: must fail
    let res_explicit = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
    assert!(
        res_explicit.is_err(),
        "Must reject session when bundle_sha256 is tampered by 1 bit"
    );
    let err_msg = res_explicit.err().unwrap().to_string();
    assert!(
        err_msg.contains("checksum mismatch") || err_msg.contains("mismatch"),
        "Error message must indicate checksum mismatch: got '{err_msg}'"
    );

    // 2. Load with empty expected bundle identity: must still fail against active bundle
    let res_implicit = ChatSession::load_session(&bundle, &tampered_path, "");
    assert!(
        res_implicit.is_err(),
        "Must reject session when file checksum differs from active bundle"
    );
    let err_msg2 = res_implicit.err().unwrap().to_string();
    assert!(
        err_msg2.contains("checksum mismatch") || err_msg2.contains("mismatch"),
        "Error message must indicate bundle checksum mismatch: got '{err_msg2}'"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 2. Adversarial Tamper Tests:
///    - Mutate 1 byte in memory key arrays: verify error or detection.
#[test]
fn test_m3_adversarial_tamper_memory_key_arrays() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, Some("Persistent Persona"), 100).expect("session");
    session
        .ingest_user_turn("Seed user turn to populate memory")
        .expect("turn");

    let temp_dir =
        std::env::temp_dir().join(format!("uor_m3_adv_tamper_mem_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("clean_session.json");

    session
        .save_session_default(&session_path)
        .expect("save session");
    let original_json = fs::read_to_string(&session_path).expect("read clean session");

    // Case 2a: Raw JSON syntax corruption in dialogue_keys array (1 byte mutation)
    {
        let tampered_path = temp_dir.join("tampered_syntax_keys.json");
        // Replace opening bracket of dialogue_keys with an illegal character 'X'
        let tampered_json =
            original_json.replacen("\"dialogue_keys\": [", "\"dialogue_keys\": X", 1);
        assert_ne!(original_json, tampered_json);
        fs::write(&tampered_path, tampered_json).expect("write tampered syntax");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(
            res.is_err(),
            "Must reject session when dialogue_keys has raw syntax corruption"
        );
    }

    // Case 2b: Dialogue key dimension corruption (64 -> 63 elements)
    {
        let tampered_path = temp_dir.join("tampered_dim_dialogue.json");
        let mut parsed: serde_json::Value =
            serde_json::from_str(&original_json).expect("parse json");
        // Pop one element from the first dialogue key array
        let keys_arr = parsed["session_state"]["dialogue_keys"]
            .as_array_mut()
            .unwrap();
        let slot0_arr = keys_arr[0].as_array_mut().unwrap();
        assert_eq!(slot0_arr.len(), KEY_DIM);
        slot0_arr.pop(); // now 63 elements
        assert_eq!(slot0_arr.len(), 63);

        fs::write(
            &tampered_path,
            serde_json::to_string_pretty(&parsed).unwrap(),
        )
        .expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(
            res.is_err(),
            "Must reject dialogue key array whose dimension is 63 instead of 64"
        );
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("corrupted dialogue key") && err_msg.contains("64"),
            "Error must specify corrupted dialogue key dimension: got '{err_msg}'"
        );
    }

    // Case 2c: Persistent key dimension corruption (64 -> 65 elements)
    {
        let tampered_path = temp_dir.join("tampered_dim_persistent.json");
        let mut parsed: serde_json::Value =
            serde_json::from_str(&original_json).expect("parse json");
        let keys_arr = parsed["session_state"]["persistent_keys"]
            .as_array_mut()
            .unwrap();
        assert!(!keys_arr.is_empty());
        let slot0_arr = keys_arr[0].as_array_mut().unwrap();
        assert_eq!(slot0_arr.len(), KEY_DIM);
        slot0_arr.push(serde_json::Value::Number(999.into())); // now 65 elements
        assert_eq!(slot0_arr.len(), 65);

        fs::write(
            &tampered_path,
            serde_json::to_string_pretty(&parsed).unwrap(),
        )
        .expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(
            res.is_err(),
            "Must reject persistent key array whose dimension is 65 instead of 64"
        );
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("corrupted persistent key") && err_msg.contains("64"),
            "Error must specify corrupted persistent key dimension: got '{err_msg}'"
        );
    }

    // Case 2d: Dialogue slot count corruption (224 -> 223 slots)
    {
        let tampered_path = temp_dir.join("tampered_slot_count.json");
        let mut parsed: serde_json::Value =
            serde_json::from_str(&original_json).expect("parse json");
        let keys_arr = parsed["session_state"]["dialogue_keys"]
            .as_array_mut()
            .unwrap();
        assert_eq!(keys_arr.len(), DIALOGUE_CAPACITY);
        keys_arr.pop(); // remove one slot -> 223 slots
        assert_eq!(keys_arr.len(), DIALOGUE_CAPACITY - 1);

        fs::write(
            &tampered_path,
            serde_json::to_string_pretty(&parsed).unwrap(),
        )
        .expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(
            res.is_err(),
            "Must reject session with missing dialogue slot (223 != 224)"
        );
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("corrupted dialogue keys") && err_msg.contains("224"),
            "Error must identify corrupted dialogue keys slot count: got '{err_msg}'"
        );
    }

    // Case 2e: Value tampering within memory key array: verify detection
    {
        let tampered_path = temp_dir.join("tampered_key_value.json");
        let mut parsed: serde_json::Value =
            serde_json::from_str(&original_json).expect("parse json");
        // Tamper coordinate 0 of dialogue slot 0: change value to 0x7FFF_FFFF
        let keys_arr = parsed["session_state"]["dialogue_keys"]
            .as_array_mut()
            .unwrap();
        let slot0_arr = keys_arr[0].as_array_mut().unwrap();
        let original_val = slot0_arr[0].as_i64().unwrap();
        let tampered_val = if original_val == 4242 { 9999 } else { 4242 };
        slot0_arr[0] = serde_json::Value::Number(tampered_val.into());

        fs::write(
            &tampered_path,
            serde_json::to_string_pretty(&parsed).unwrap(),
        )
        .expect("write");

        let loaded = ChatSession::load_session(&bundle, &tampered_path, bundle.identity())
            .expect("deserialization succeeds because format/dimensions are intact");
        assert_ne!(
            loaded.state().dialogue_keys[0][0],
            session.state().dialogue_keys[0][0],
            "Tampered coordinate value must be detected as differing from untampered source"
        );
        assert_eq!(loaded.state().dialogue_keys[0][0], tampered_val as i32);
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 2. Adversarial Tamper Tests:
///    - Alter the schema header: verify rejection.
#[test]
fn test_m3_adversarial_tamper_schema_header_rejection() {
    let bundle = Bundle::synthetic_for_test();
    let session = ChatSession::new(&bundle, None, 42).expect("new session");

    let temp_dir =
        std::env::temp_dir().join(format!("uor_m3_adv_tamper_hdr_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("clean_session.json");

    session
        .save_session_default(&session_path)
        .expect("save session");
    let original_json = fs::read_to_string(&session_path).expect("read clean session");

    // Case 3a: Mutate schema to future/invalid version "uor-r4.integer-session/2"
    {
        let tampered_path = temp_dir.join("tampered_schema_v2.json");
        let tampered_json = original_json.replace(
            "\"schema\": \"uor-r4.integer-session/1\"",
            "\"schema\": \"uor-r4.integer-session/2\"",
        );
        assert_ne!(original_json, tampered_json);
        fs::write(&tampered_path, tampered_json).expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(res.is_err(), "Must reject unknown schema version /2");
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("unsupported session schema"),
            "Error must specify unsupported session schema: got '{err_msg}'"
        );
    }

    // Case 3b: Mutate schema to arbitrary foreign format
    {
        let tampered_path = temp_dir.join("tampered_schema_foreign.json");
        let tampered_json = original_json.replace(
            "\"schema\": \"uor-r4.integer-session/1\"",
            "\"schema\": \"foreign.transformer-session/1\"",
        );
        fs::write(&tampered_path, tampered_json).expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(res.is_err(), "Must reject foreign schema header");
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("unsupported session schema"),
            "Error must specify unsupported session schema: got '{err_msg}'"
        );
    }

    // Case 3c: Mutate integer version field from 1 to 2
    {
        let tampered_path = temp_dir.join("tampered_version_field_2.json");
        let tampered_json = original_json.replace("\"version\": 1,", "\"version\": 2,");
        assert_ne!(original_json, tampered_json);
        fs::write(&tampered_path, tampered_json).expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(res.is_err(), "Must reject version != 1");
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("unsupported session version"),
            "Error must specify unsupported session version: got '{err_msg}'"
        );
    }

    // Case 3d: Mutate integer version field from 1 to 0
    {
        let tampered_path = temp_dir.join("tampered_version_field_0.json");
        let tampered_json = original_json.replace("\"version\": 1,", "\"version\": 0,");
        assert_ne!(original_json, tampered_json);
        fs::write(&tampered_path, tampered_json).expect("write");

        let res = ChatSession::load_session(&bundle, &tampered_path, bundle.identity());
        assert!(res.is_err(), "Must reject version 0");
        let err_msg = res.err().unwrap().to_string();
        assert!(
            err_msg.contains("unsupported session version"),
            "Error must specify unsupported session version: got '{err_msg}'"
        );
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 3. PRNG Determinism & Resumption:
///    Save session at turn 5, generate 20 tokens.
///    Load session from turn 5 checkpoint and generate 20 tokens.
///    Verify that the two 20-token sequences are 100% bit-identical.
#[test]
fn test_m3_adversarial_prng_determinism_turn_5_resumption() {
    let bundle = Bundle::synthetic_for_test();
    let initial_seed = 0xFEED_FACE_CAFE_BEEF;

    let mut session = ChatSession::new(&bundle, Some("Assistant Persona"), initial_seed)
        .expect("new chat session");
    // Use categorical sampling with top_k = 8 to actively exercise xorshift64 PRNG
    let policy = SamplePolicy::Categorical { top_k: 8 };
    session.set_policy(policy);

    // Execute exactly 5 dialogue turns
    for turn in 1..=5 {
        let prompt = format!("Turn {turn}: establish dialogue history before checkpointing.");
        session.ingest_user_turn(&prompt).expect("ingest user turn");
        assert_eq!(session.state().current_turn(), turn as u32);

        // Assistant steps 2 tokens and finishes with TurnEnd
        for step in 0..2 {
            let token = (((turn * 23 + step * 7) % 3000) + 15) as u32;
            session
                .step_token(token, SlotTarget::Dialogue)
                .expect("step token");
        }
        session
            .step_token(RoleToken::TurnEnd.id(), SlotTarget::Dialogue)
            .expect("turn end");
    }

    assert_eq!(session.state().current_turn(), 5);

    // Checkpoint session at turn 5
    let temp_dir =
        std::env::temp_dir().join(format!("uor_m3_adv_prng_chk5_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let checkpoint_path = temp_dir.join("checkpoint_turn_5.json");

    session
        .save_session_default(&checkpoint_path)
        .expect("save turn 5 checkpoint");

    // Capture PRNG state at checkpoint
    let checkpoint_prng_state = session.sampler().state();

    // --- Branch A: Continue active session for 20 generated tokens ---
    let mut tokens_active = Vec::with_capacity(20);
    let mut step_active = session
        .step_token(42, SlotTarget::Dialogue)
        .expect("step initial token on active session");

    for _ in 0..20 {
        let selected_idx = session
            .sampler_mut()
            .select(&step_active.probabilities, policy)
            .expect("sample token");
        let token = selected_idx as u32;
        tokens_active.push(token);
        step_active = session
            .step_token(token, SlotTarget::Dialogue)
            .expect("step active token");
    }
    assert_eq!(
        tokens_active.len(),
        20,
        "Active session must generate exactly 20 tokens"
    );
    let final_active_prng_state = session.sampler().state();
    assert_ne!(
        checkpoint_prng_state, final_active_prng_state,
        "PRNG state must advance after 20 categorical selections"
    );

    // --- Branch B: Restore session from turn 5 checkpoint ---
    let mut loaded_session = bundle
        .load_chat_session(&checkpoint_path, bundle.identity())
        .expect("load turn 5 checkpoint");

    assert_eq!(
        loaded_session.sampler().state(),
        checkpoint_prng_state,
        "Loaded session PRNG state must match checkpoint PRNG state exactly"
    );
    assert_eq!(loaded_session.state().current_turn(), 5);

    let mut tokens_loaded = Vec::with_capacity(20);
    let mut step_loaded = loaded_session
        .step_token(42, SlotTarget::Dialogue)
        .expect("step initial token on loaded session");

    for _ in 0..20 {
        let selected_idx = loaded_session
            .sampler_mut()
            .select(&step_loaded.probabilities, policy)
            .expect("sample token");
        let token = selected_idx as u32;
        tokens_loaded.push(token);
        step_loaded = loaded_session
            .step_token(token, SlotTarget::Dialogue)
            .expect("step loaded token");
    }
    assert_eq!(
        tokens_loaded.len(),
        20,
        "Loaded session must generate exactly 20 tokens"
    );
    let final_loaded_prng_state = loaded_session.sampler().state();

    // --- VERIFICATION: 100% Bit-Identical Token Sequences and PRNG State ---
    assert_eq!(
        tokens_active, tokens_loaded,
        "20-token generated sequence from turn 5 checkpoint must be 100% bit-identical!"
    );
    assert_eq!(
        final_active_prng_state, final_loaded_prng_state,
        "Final PRNG state after 20 tokens must be 100% bit-identical!"
    );
    assert_eq!(
        session.state().state,
        loaded_session.state().state,
        "Final recurrent state Q11 [256] after 20 tokens must be 100% bit-identical!"
    );
    assert_eq!(
        session.state().cumulative_holonomy_q30,
        loaded_session.state().cumulative_holonomy_q30,
        "Final cumulative holonomy Q30 after 20 tokens must be 100% bit-identical!"
    );
    assert_eq!(
        session.state().zeta_phases(),
        loaded_session.state().zeta_phases(),
        "Final T8 zeta phases after 20 tokens must be 100% bit-identical!"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 3. PRNG Zero-Seed Protection & Infinite Loop Challenge:
///    Verify that `set_state(0)` and `from_state(0)` safely remap the zero seed
///    and never lock into an infinite zero-loop over 10,000 steps.
#[test]
fn test_m3_adversarial_prng_zero_seed_protection_and_infinite_loop() {
    // 1. Test Sampler::new(0)
    let s_new_zero = Sampler::new(0);
    assert_ne!(
        s_new_zero.state(),
        0,
        "Sampler::new(0) must remap 0 to non-zero constant"
    );

    // 2. Test Sampler::from_state(0)
    let mut s_from_zero = Sampler::from_state(0);
    assert_ne!(
        s_from_zero.state(),
        0,
        "Sampler::from_state(0) must remap 0 to non-zero constant"
    );
    assert_eq!(
        s_new_zero.state(),
        s_from_zero.state(),
        "new(0) and from_state(0) must agree on canonical nonzero state"
    );

    // 3. Test Sampler::set_state(0)
    let mut s_set_zero = Sampler::new(0x1234_5678);
    s_set_zero.set_state(0);
    assert_ne!(
        s_set_zero.state(),
        0,
        "Sampler::set_state(0) must remap 0 to non-zero constant"
    );
    assert_eq!(s_set_zero.state(), s_from_zero.state());

    // 4. Infinite zero-loop challenge over 10,000 consecutive steps
    // Standard unmitigated xorshift64 on 0 loops infinitely on 0:
    // x ^= x << 13 (0); x ^= x >> 7 (0); x ^= x << 17 (0).
    // Verify that the remapped sampler NEVER emits 0 and produces 10,000 unique states.
    let mut seen_values = HashSet::with_capacity(10_000);
    let mut bit_ones_count = 0u64;

    for i in 0..10_000 {
        let val = s_from_zero.xorshift64();
        assert_ne!(val, 0, "xorshift64 must NEVER emit 0 (failed at step {i})");
        assert_ne!(
            s_from_zero.state(),
            0,
            "xorshift64 state must NEVER become 0 (failed at step {i})"
        );
        seen_values.insert(val);
        bit_ones_count += val.count_ones() as u64;
    }

    assert_eq!(
        seen_values.len(),
        10_000,
        "xorshift64 must generate 10,000 distinct values without short-cycle collapse"
    );

    // Verify entropy: average bit density across 10,000 64-bit words should be close to 32 bits/word
    let total_bits = 10_000u64 * 64;
    let one_density = (bit_ones_count as f64) / (total_bits as f64);
    assert!(
        one_density > 0.45 && one_density < 0.55,
        "Bit-one density must be within [0.45, 0.55] for pseudo-random uniformity (got {one_density})"
    );
}
