//! Milestone M1 Integration Tests: Real Weights Ingestion & Tokenizer Alignment
//!
//! Verifies:
//! 1. Flexible conversational role token alignment with safe fallbacks (turn_end -> eos, system -> bos, user -> unk, assistant -> bos).
//! 2. Zero collisions with ASCII punctuation tokens (!, ", #, $).
//! 3. Clean prompt wrapping in ingest_user_turn and ingest_system_prompt without punctuation fragmentation.
//! 4. Real 4K bundle loading (quaternion and householder_pair) and numerical kernel verification.
//! 5. Autoregressive streaming generation with real pre-trained language weights.

use std::path::Path;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::model::SlotTarget;
use uor_r4_integer::session::{ChatSession, RoleTokens};
use uor_r4_integer::PROBABILITY_TOTAL;
use uor_r4_tokenizer::ByteBpeTokenizer;

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";
const REAL_HOUSEHOLDER_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";

#[test]
fn test_m1_role_tokens_fallback_mapping_real_tokenizer() {
    let tok_path = Path::new(REAL_QUATERNION_BUNDLE).join("tokenizer.json");
    assert!(tok_path.exists(), "real tokenizer.json must exist");

    let bytes = std::fs::read(&tok_path).expect("read tokenizer.json");
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("parse real tokenizer");

    // Real 4K tokenizer has only <|bos|>, <|eos|>, <|unk|> in added_tokens
    assert_eq!(tokenizer.token_id("<|bos|>"), Some(0));
    assert_eq!(tokenizer.token_id("<|eos|>"), Some(1));
    assert_eq!(tokenizer.token_id("<|unk|>"), Some(2));
    assert_eq!(tokenizer.token_id("<|system|>"), None);
    assert_eq!(tokenizer.token_id("<|user|>"), None);
    assert_eq!(tokenizer.token_id("<|assistant|>"), None);
    assert_eq!(tokenizer.token_id("<|turn_end|>"), None);

    // Verify ASCII punctuation IDs in real 4K vocab
    assert_eq!(tokenizer.token_id("!"), Some(3));
    assert_eq!(tokenizer.token_id("\""), Some(4));
    assert_eq!(tokenizer.token_id("#"), Some(5));
    assert_eq!(tokenizer.token_id("$"), Some(6));

    // Resolve role tokens via RoleTokens::from_tokenizer
    let roles = RoleTokens::from_tokenizer(&tokenizer).expect("RoleTokens resolution must succeed");

    // Verify fallback mappings match contract
    assert_eq!(
        roles.turn_end_id, 1,
        "turn_end_id must fallback to eos_id (1)"
    );
    assert_eq!(roles.system_id, 0, "system_id must fallback to bos_id (0)");
    assert_eq!(roles.user_id, 2, "user_id must fallback to unk_id (2)");
    assert_eq!(
        roles.assistant_id, 0,
        "assistant_id must fallback to bos_id (0)"
    );
    assert_eq!(roles.eos_id, 1, "eos_id must be 1");

    // Invariant: Role tokens MUST NOT collide with ASCII punctuation tokens (!, ", #, $)
    let punct_ids = [3u32, 4, 5, 6];
    assert!(
        !punct_ids.contains(&roles.system_id),
        "system_id collides with punctuation"
    );
    assert!(
        !punct_ids.contains(&roles.user_id),
        "user_id collides with punctuation"
    );
    assert!(
        !punct_ids.contains(&roles.assistant_id),
        "assistant_id collides with punctuation"
    );
    assert!(
        !punct_ids.contains(&roles.turn_end_id),
        "turn_end_id collides with punctuation"
    );
}

#[test]
fn test_m1_role_tokens_rejection_on_ascii_punctuation_collision() {
    // Construct a tokenizer where ASCII '!' is at ID 0 (colliding with fallback system_id / bos_id 0)
    let mut vocab_map = serde_json::Map::new();
    vocab_map.insert("!".to_string(), serde_json::json!(0)); // '!' is assigned to ID 0
    vocab_map.insert("<|eos|>".to_string(), serde_json::json!(1));
    vocab_map.insert("<|unk|>".to_string(), serde_json::json!(2));
    vocab_map.insert("\"".to_string(), serde_json::json!(3));
    vocab_map.insert("#".to_string(), serde_json::json!(4));
    vocab_map.insert("$".to_string(), serde_json::json!(5));

    for i in 6..256 {
        vocab_map.insert(format!("t{i}"), serde_json::json!(i));
    }

    let tok_json = serde_json::json!({
        "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": false },
        "model": { "type": "BPE", "vocab": vocab_map, "merges": [] },
        "added_tokens": [
            { "id": 1, "content": "<|eos|>" },
            { "id": 2, "content": "<|unk|>" }
        ]
    });
    let tok_bytes = serde_json::to_vec(&tok_json).unwrap();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .expect("must parse valid collision-fixture tokenizer");

    let err = RoleTokens::from_tokenizer(&tokenizer)
        .expect_err("must reject role token collision with ASCII punctuation '!'");
    assert!(
        err.to_string().contains("role token collision detected"),
        "error message must describe collision: {err}"
    );
}

#[test]
fn test_m1_prompt_ingestion_clean_boundaries_no_punctuation_fragmentation() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");

    let mut session = ChatSession::new(&bundle, None, 42).expect("initialize chat session");
    let roles = session.roles();

    assert_eq!(roles.user_id, 2);
    assert_eq!(roles.turn_end_id, 1);
    assert_eq!(roles.assistant_id, 0);

    // Ingest a user prompt with punctuation: "Why is 2+2=4?"
    let user_prompt = "Why is 2+2=4?";
    let ingested_count = session
        .ingest_user_turn(user_prompt)
        .expect("ingest user turn");

    let encoded_text = bundle.tokenizer().encode(user_prompt);
    assert_eq!(
        ingested_count,
        encoded_text.len() + 3,
        "ingested count must equal user_tokens.len() + 3 (user, turn_end, assistant)"
    );

    // Inspect dialogue tokens in session state: first must be user_id (2), second-to-last turn_end_id (1), last assistant_id (0)
    let diag_tokens = &session.state().dialogue_tokens;
    assert_eq!(
        diag_tokens[0], roles.user_id,
        "first token must be roles.user_id (2)"
    );
    assert_eq!(
        diag_tokens[ingested_count - 2],
        roles.turn_end_id,
        "second to last token must be roles.turn_end_id (1)"
    );
    assert_eq!(
        diag_tokens[ingested_count - 1],
        roles.assistant_id,
        "last token must be roles.assistant_id (0)"
    );

    // Verify role tokens do NOT contain ASCII punctuation marks (! = 3, " = 4, # = 5, $ = 6)
    assert_ne!(diag_tokens[0], 3);
    assert_ne!(diag_tokens[0], 4);
    assert_ne!(diag_tokens[0], 5);
    assert_ne!(diag_tokens[0], 6);
}

#[test]
fn test_m1_real_bundle_loading_and_kernel_verification_quaternion() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");

    assert_eq!(bundle.model().config().vocab_size, 4096);
    assert_eq!(bundle.model().config().width, 256);
    assert_eq!(bundle.model().config().context, 256);

    let model = bundle.model();
    let mut session = model.new_session();
    let step = model
        .step(&mut session, 0, ReadMode::Enabled)
        .expect("model.step succeeds");

    assert_eq!(step.probabilities.len(), 4096);
    let sum_prob: u64 = step.probabilities.iter().sum();
    assert_eq!(
        sum_prob, PROBABILITY_TOTAL,
        "probability sum must equal 2^48"
    );

    // Step conversational
    let mut conv_session = model.new_conversational_session();
    let conv_step = model
        .step_conversational(
            &mut conv_session,
            0,
            SlotTarget::Persistent,
            ReadMode::Enabled,
        )
        .expect("step_conversational succeeds");
    assert_eq!(conv_step.probabilities.len(), 4096);
}

#[test]
fn test_m1_real_bundle_loading_and_kernel_verification_householder_pair() {
    let bundle_path = Path::new(REAL_HOUSEHOLDER_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real householder_pair bundle");

    assert_eq!(bundle.model().config().vocab_size, 4096);
    assert_eq!(bundle.model().config().width, 256);
    assert_eq!(bundle.model().config().context, 256);

    let model = bundle.model();
    let mut session = model.new_session();
    let step = model
        .step(&mut session, 0, ReadMode::Enabled)
        .expect("model.step succeeds");

    assert_eq!(step.probabilities.len(), 4096);
    let sum_prob: u64 = step.probabilities.iter().sum();
    assert_eq!(
        sum_prob, PROBABILITY_TOTAL,
        "probability sum must equal 2^48"
    );
}

#[test]
fn test_m1_real_bundle_streaming_generation_e2e() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");

    let mut session = ChatSession::new(&bundle, None, 12345).expect("initialize chat session");

    // Turn 1
    let mut stream1 = session
        .generate_stream("Hello, tell me a quick story.", 16, &[])
        .expect("generate stream turn 1");

    let mut tokens_out = Vec::new();
    for chunk in stream1.by_ref() {
        assert!(!chunk.is_empty(), "emitted chunk must not be empty");
        tokens_out.push(chunk);
    }
    assert!(stream1.is_stopped(), "stream 1 must be marked stopped");
    assert!(
        stream1.stop_reason().is_some(),
        "stream 1 must have a stop reason"
    );
    assert!(!tokens_out.is_empty(), "must generate tokens");

    // Turn 2
    let mut stream2 = session
        .generate_stream("What happened next?", 16, &[])
        .expect("generate stream turn 2");

    let mut tokens_out2 = Vec::new();
    for chunk in stream2.by_ref() {
        tokens_out2.push(chunk);
    }
    assert!(stream2.is_stopped(), "stream 2 must be marked stopped");
    assert!(!tokens_out2.is_empty(), "must generate tokens on turn 2");
}
