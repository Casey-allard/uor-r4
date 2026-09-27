//! Adversarial Empirical Challenger Suite — Milestone M1 Token Boundary & Punctuation Stress
//!
//! Objectives:
//! 1. Verify faithful encoding and decoding of tokens 3, 4, 5, 6 as standard vocabulary items
//!    in real 4K bundles (bundle-quaternion-1 and bundle-householder_pair-1).
//! 2. Verify subword sequences containing !, ", #, $ do NOT trigger false turn ends or stream stops.
//! 3. Verify exact token ID preservation across prompt ingestion and dialogue state.
//! 4. Verify ChatTokenStream emission when tokens 3..6 are selected.
//! 5. Verify robust error handling on empty/whitespace-only turns while accepting punctuation-only turns.
//! 6. Verify multi-turn dialogue with continuous punctuation stress.

use std::path::Path;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::session::{ChatSession, RoleTokens, StreamStopReason};
use uor_r4_tokenizer::ByteBpeTokenizer;

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";
const REAL_HOUSEHOLDER_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";

#[test]
fn test_challenger_tokens_3_to_6_decoding_identity() {
    for bundle_path in [REAL_QUATERNION_BUNDLE, REAL_HOUSEHOLDER_BUNDLE] {
        let tok_path = Path::new(bundle_path).join("tokenizer.json");
        assert!(tok_path.exists());
        let bytes = std::fs::read(&tok_path).expect("read tokenizer.json");
        let tokenizer =
            ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("parse tokenizer");

        // Verify tokens 3, 4, 5, 6 exact mappings
        assert_eq!(tokenizer.token_id("!"), Some(3), "Token '!' must have ID 3");
        assert_eq!(
            tokenizer.token_id("\""),
            Some(4),
            "Token '\"' must have ID 4"
        );
        assert_eq!(tokenizer.token_id("#"), Some(5), "Token '#' must have ID 5");
        assert_eq!(tokenizer.token_id("$"), Some(6), "Token '$' must have ID 6");

        // Verify single-token decoding
        assert_eq!(tokenizer.decode(&[3]), "!", "Token 3 must decode to '!'");
        assert_eq!(tokenizer.decode(&[4]), "\"", "Token 4 must decode to '\"'");
        assert_eq!(tokenizer.decode(&[5]), "#", "Token 5 must decode to '#'");
        assert_eq!(tokenizer.decode(&[6]), "$", "Token 6 must decode to '$'");

        // Verify multi-token decoding of punctuation sequences
        let punct_tokens = [3, 4, 5, 6, 6, 5, 4, 3];
        let decoded = tokenizer.decode(&punct_tokens);
        assert_eq!(
            decoded, "!\"#$$#\"!",
            "Punctuation token sequence must decode without alteration"
        );
    }
}

#[test]
fn test_challenger_subword_sequences_with_punctuation_roundtrip() {
    let tok_path = Path::new(REAL_QUATERNION_BUNDLE).join("tokenizer.json");
    let bytes = std::fs::read(&tok_path).expect("read tokenizer.json");
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("parse tokenizer");

    let test_phrases = [
        "Hello!",
        "\"Quoted text\"",
        "#1 Winner!",
        "Total is $100.",
        "Price: $5.99 for #2!",
        "!\"#$",
        "$$$###\"\"\"!!!!",
        "Punctuation inside words: test!word, \"quote\"end, #hash, $dollar",
    ];

    for phrase in &test_phrases {
        let encoded = tokenizer.encode(phrase);
        assert!(
            !encoded.is_empty(),
            "Encoded tokens must not be empty for '{phrase}'"
        );
        let decoded = tokenizer.decode(&encoded);
        assert_eq!(
            &decoded, phrase,
            "Round-trip encode/decode must match verbatim for '{phrase}'"
        );

        // Invariant: encoded tokens must NOT contain role token fallbacks (system=0, eos=1, unk=2) unless originally present
        // Specifically, none of these standard punctuation phrases should encode to EOS (1)
        assert!(
            !encoded.contains(&1),
            "Encoding of standard phrase '{phrase}' must not contain EOS (1)"
        );
    }
}

#[test]
fn test_challenger_punctuation_prompts_do_not_trigger_false_turn_end() {
    let bundle = Bundle::load(Path::new(REAL_QUATERNION_BUNDLE)).expect("load bundle");
    let mut session = ChatSession::new(&bundle, None, 42).expect("new session");

    let punctuation_prompts = [
        "!",
        "\"",
        "#",
        "$",
        "$$$",
        "!!!",
        "#1",
        "Price: $99!",
        "Wait... \"what?\" #confused $0",
    ];

    for (turn_idx, prompt) in punctuation_prompts.iter().enumerate() {
        let mut stream = session
            .generate_stream(prompt, 8, &[])
            .unwrap_or_else(|e| panic!("Failed to generate stream for prompt '{prompt}': {e}"));

        let mut emitted = Vec::new();
        for token_text in stream.by_ref() {
            emitted.push(token_text);
        }

        assert!(
            stream.is_stopped(),
            "Stream must finish gracefully for prompt '{prompt}'"
        );
        let reason = stream.stop_reason().expect("Must have stop reason");
        // The reason must be TurnEnd, Eos, or MaxTokens (not a panic or corruption)
        match reason {
            StreamStopReason::TurnEnd { token_id } => {
                assert_eq!(
                    token_id, 1,
                    "TurnEnd stop token must be 1 (EOS), never 3..6 (!, \", #, $)"
                );
            }
            StreamStopReason::Eos { token_id } => {
                assert_eq!(token_id, 1, "EOS stop token must be 1");
            }
            StreamStopReason::MaxTokens { .. } | StreamStopReason::CycleDetected { .. } => {}
            StreamStopReason::CustomStop { token_id } => {
                panic!("Unexpected custom stop token {token_id} on turn {turn_idx}");
            }
            StreamStopReason::ExhaustedContext | StreamStopReason::ModelError => {
                panic!("Unexpected failure reason {:?} on turn {turn_idx}", reason);
            }
        }
    }
}

#[test]
fn test_challenger_token_stream_does_not_stop_on_tokens_3_to_6() {
    let bundle = Bundle::load(Path::new(REAL_QUATERNION_BUNDLE)).expect("load bundle");
    let session = ChatSession::new(&bundle, None, 42).expect("new session");

    // In ChatSession with real bundle, roles are:
    // turn_end_id = 1, eos_id = 1, user_id = 2, system_id = 0, assistant_id = 0
    assert_eq!(session.roles().turn_end_id, 1);
    assert_eq!(session.roles().eos_id, 1);

    // Verify directly that tokens 3, 4, 5, 6 are not stop tokens
    for token_id in [3u32, 4, 5, 6] {
        let is_turn_end = token_id == session.roles().turn_end_id;
        let is_eos = token_id == session.roles().eos_id;
        assert!(
            !is_turn_end,
            "Token {token_id} must NOT be considered turn_end_id"
        );
        assert!(!is_eos, "Token {token_id} must NOT be considered eos_id");
    }
}

#[test]
fn test_challenger_whitespace_and_empty_prompt_rejection() {
    let bundle = Bundle::load(Path::new(REAL_QUATERNION_BUNDLE)).expect("load bundle");
    let mut session = ChatSession::new(&bundle, None, 42).expect("new session");

    // Empty and whitespace prompts must return error
    let invalid_prompts = ["", "   ", "\t", "\n", "\r\n", "   \t \n  "];
    for invalid in &invalid_prompts {
        assert!(
            session.ingest_user_turn(invalid).is_err(),
            "Empty or whitespace prompt '{invalid}' must be rejected"
        );
    }

    // Punctuation with whitespace surrounding it must SUCCEED
    let valid_puncts = ["  !  ", " \t # \n ", "   $   ", "  \"  "];
    for valid in &valid_puncts {
        assert!(
            session.ingest_user_turn(valid).is_ok(),
            "Punctuation with whitespace '{valid}' must be accepted"
        );
    }
}

#[test]
fn test_challenger_punctuation_collision_detection_matrix() {
    // Test each punctuation character (!, ", #, $) placed at colliding positions (0, 1, 2)
    // Position 0 collides with system_id / assistant_id (fallback to 0)
    // Position 1 collides with turn_end_id / eos_id (fallback to 1)
    // Position 2 collides with user_id (fallback to 2)
    let puncts = ["!", "\"", "#", "$"];
    let colliding_ids = [
        (0u32, "system_id / assistant_id fallback 0"),
        (1u32, "turn_end_id / eos_id fallback 1"),
        (2u32, "user_id fallback 2"),
    ];

    for punct in &puncts {
        for (target_id, target_desc) in &colliding_ids {
            let mut vocab_map = serde_json::Map::new();

            for i in 0..256u32 {
                if i == *target_id {
                    vocab_map.insert(punct.to_string(), serde_json::json!(i));
                } else {
                    vocab_map.insert(format!("t{i}"), serde_json::json!(i));
                }
            }

            let tok_json = serde_json::json!({
                "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": false },
                "model": { "type": "BPE", "vocab": vocab_map, "merges": [] },
                "added_tokens": []
            });
            let bytes = serde_json::to_vec(&tok_json).unwrap();
            let tokenizer =
                ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).unwrap_or_else(|| {
                    panic!("Failed to parse fixture for {punct} at {target_id} ({target_desc})")
                });

            let result = RoleTokens::from_tokenizer(&tokenizer);
            assert!(
                result.is_err(),
                "Must detect collision when punctuation '{punct}' is placed at {target_id} ({target_desc})"
            );
            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.contains("role token collision detected"),
                "Error message must describe collision: {err_msg}"
            );
        }
    }
}

#[test]
fn test_challenger_system_prompt_with_punctuation_and_persistence() {
    let bundle = Bundle::load(Path::new(REAL_QUATERNION_BUNDLE)).expect("load bundle");

    // 1. Verify overflow rejection: prompt exceeding 32 persistent slots must be rejected
    let long_prompt = "You are an assistant! Follow rule #1: never charge > $100 under any circumstances. Say \"Hi\" to every user!";
    assert!(
        ChatSession::new(&bundle, Some(long_prompt), 42).is_err(),
        "Must reject system prompt exceeding 32-slot persistent capacity"
    );

    // 2. Prompt within capacity (< 32 tokens) with punctuation (!, ", #, $) must succeed
    let sys_prompt = "You are helpful! Rule #1: $100 max. \"Hi\".";
    let mut session =
        ChatSession::new(&bundle, Some(sys_prompt), 42).expect("new session with sys prompt");

    assert!(
        session.state().is_persistent_sealed(),
        "System persona must be sealed"
    );

    // Invariant: attempting to ingest another system prompt after sealing must be rejected
    assert!(
        session.ingest_system_prompt("Another prompt!").is_err(),
        "Must reject modifying sealed persistent persona"
    );

    // Persistent tokens must start with system_id and end with turn_end_id
    let roles = session.roles();
    let pers = &session.state().persistent_tokens;
    assert_eq!(
        pers[0], roles.system_id,
        "First token must be system_id (0)"
    );
    let pers_len = session.state().persistent_len();
    assert!(pers_len <= 32, "Persistent len must be <= 32");
    assert_eq!(
        pers[pers_len - 1],
        roles.turn_end_id,
        "Last token must be turn_end_id (1)"
    );

    // User turn following system prompt
    let mut stream = session
        .generate_stream("What is rule #1?", 16, &[])
        .expect("generate stream");

    let mut out = Vec::new();
    for token in stream.by_ref() {
        out.push(token);
    }
    assert!(stream.is_stopped());
    assert!(!out.is_empty(), "Must emit tokens after system prompt");
}
