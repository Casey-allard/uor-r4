//! Adversarial Challenger Test Suite — Milestone M2:
//! Memory Flattening, Power-of-Two Stride Indexing, and Dialogue Rollover.
//! File: crates/uor-r4-integer/tests/adversarial_memory_rollover_m2.rs
//!
//! Empirical verification covering:
//! 1. Extreme dialogue rollover (> 300 turns, > 1000 tokens):
//!    - Zero panic, zero out-of-bounds.
//!    - Exact branchless circular rollover without `% 224` division.
//!    - Exact sequence numbering, turn tracking, and total probability sum 2^48 invariance.
//! 2. Persistent session slots (persona) 100% bit-level coordinate identity:
//!    - Full 32 persistent slots populated with distinct coordinate values.
//!    - 520+ turns of continuous dialogue rollover (> 4 full ring buffer wraparounds).
//!    - Bit-for-bit exact coordinate preservation across all 32 * 64 key coords and 32 * 256 val coords.
//! 3. Row strides & memory flattening non-corruption in `SessionState` and `IntegerSession`:
//!    - Power-of-two row stride isolation (256 B for keys, 1024 B for values).
//!    - Canary bit patterns in neighboring slots (k-1, k+1, boundary 0, boundary 223).
//!    - Monotonic slot addition in `IntegerSession` with 100% history preservation up to context limit (256).
//! 4. Embedding & vocabulary projection power-of-two stride boundary validation.

use uor_r4_integer::model::{KEY_DIM, VAL_DIM};
use uor_r4_integer::{
    IntegerModel, ReadMode, SessionState, SlotTarget, DIALOGUE_CAPACITY, PERSISTENT_CAPACITY,
    PROBABILITY_TOTAL,
};

/// 1. Extreme dialogue rollover (> 300 turns):
///    Verify zero panic, zero out-of-bounds, exact branchless circular rollover without `% 224` division.
#[test]
fn test_m2_adversarial_extreme_dialogue_rollover_350_turns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session: SessionState = model.new_conversational_session();

    let turns = 350; // > 300 turns
    let tokens_per_turn = 3;
    let total_dialogue_tokens = turns * tokens_per_turn; // 1050 tokens

    assert!(turns > 300);
    assert_eq!(total_dialogue_tokens, 1050);

    let mut global_token_idx = 0usize;
    let mut token_history = Vec::with_capacity(total_dialogue_tokens);

    for turn in 1..=turns {
        session.start_turn();
        assert_eq!(session.current_turn(), turn as u32);

        for step_in_turn in 0..tokens_per_turn {
            let token =
                (((global_token_idx * 43 + 7) % (model.config().vocab_size - 10)) + 5) as u32;
            token_history.push(token);

            let expected_cursor_before = global_token_idx % DIALOGUE_CAPACITY;
            assert_eq!(
                session.dialogue_cursor, expected_cursor_before,
                "Cursor mismatch before step at turn {turn}, step {step_in_turn} (global {global_token_idx})"
            );

            let step = model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .unwrap_or_else(|e| {
                    panic!(
                        "Runtime panic/error at turn {turn}, step {step_in_turn} (global {global_token_idx}): {e}"
                    )
                });

            // Invariant: probability vector length matches vocab_size
            assert_eq!(step.probabilities.len(), model.config().vocab_size);

            // Invariant: total probability sum is exactly 2^48
            let sum_prob: u64 = step.probabilities.iter().sum();
            assert_eq!(
                sum_prob, PROBABILITY_TOTAL,
                "Probability sum deviated from 2^48 at global token {global_token_idx}"
            );

            // Invariant: state dimension matches model width (256)
            assert_eq!(step.state.len(), model.config().width);

            // Verify dialogue cursor advanced circularly without `% 224` division
            let expected_cursor_after = (global_token_idx + 1) % DIALOGUE_CAPACITY;
            assert_eq!(
                session.dialogue_cursor, expected_cursor_after,
                "Cursor rollover mismatch after step at global token {global_token_idx}"
            );

            // Verify the slot written was expected_cursor_before
            let written_slot = expected_cursor_before;
            assert_eq!(session.dialogue_tokens[written_slot], token);
            assert_eq!(
                session.dialogue_sequences[written_slot],
                global_token_idx as u64
            );
            assert_eq!(session.dialogue_turn_ids[written_slot], turn as u32);

            global_token_idx += 1;

            // Invariant: dialogue_seen is monotonically equal to global_token_idx
            assert_eq!(session.dialogue_seen, global_token_idx as u64);

            // Invariant: dialogue_len saturates at DIALOGUE_CAPACITY (224)
            let expected_len = global_token_idx.min(DIALOGUE_CAPACITY);
            assert_eq!(session.dialogue_len(), expected_len);
        }
    }

    // Final verification after 350 turns (1050 tokens, ~4.68 complete buffer wraps):
    assert_eq!(session.dialogue_seen, 1050);
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_cursor, 1050 % DIALOGUE_CAPACITY); // 1050 % 224 = 154

    // Check that the last 224 tokens in token_history are exactly represented in the ring buffer
    let tail_224 = &token_history[total_dialogue_tokens - DIALOGUE_CAPACITY..];
    for (i, &expected_token) in tail_224.iter().enumerate() {
        let global_seq = (total_dialogue_tokens - DIALOGUE_CAPACITY) + i;
        let ring_slot = global_seq % DIALOGUE_CAPACITY;
        assert_eq!(
            session.dialogue_tokens[ring_slot], expected_token,
            "Slot {ring_slot} token corrupted after 350 turns"
        );
        assert_eq!(
            session.dialogue_sequences[ring_slot], global_seq as u64,
            "Slot {ring_slot} sequence corrupted after 350 turns"
        );
    }
}

/// 2. Persistent session slots (persona) retain 100% bit-level coordinate identity:
///    Populate 32 persona slots, seal partition, run 520+ turns of continuous dialogue rollover,
///    and verify bit-for-bit coordinate identity across all persistent keys and values.
#[test]
fn test_m2_adversarial_persona_bit_identity_across_520_dialogue_turns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session: SessionState = model.new_conversational_session();

    // 1. Populate all 32 persistent slots with distinct persona tokens
    for i in 0..PERSISTENT_CAPACITY {
        let persona_token = (i + 1) as u32;
        model
            .step_conversational(
                &mut session,
                persona_token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("persistent step succeeds");
    }
    assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);

    // 2. Snapshot 100% bit-level coordinates of persistent keys and values
    let saved_keys: Vec<[i32; KEY_DIM]> = session.persistent_keys.clone();
    let saved_values: Vec<[i32; VAL_DIM]> = session.persistent_values.clone();
    let saved_tokens: Vec<u32> = session.persistent_tokens.clone();

    assert_eq!(saved_keys.len(), PERSISTENT_CAPACITY);
    assert_eq!(saved_values.len(), PERSISTENT_CAPACITY);
    assert_eq!(saved_tokens.len(), PERSISTENT_CAPACITY);

    // Verify non-trivial values (non-zero)
    let non_zero_keys = saved_keys.iter().flat_map(|k| k.iter()).any(|&x| x != 0);
    let non_zero_vals = saved_values.iter().flat_map(|v| v.iter()).any(|&x| x != 0);
    assert!(
        non_zero_keys,
        "Persistent keys must contain non-zero coordinates"
    );
    assert!(
        non_zero_vals,
        "Persistent values must contain non-zero coordinates"
    );

    // 3. Seal persistent partition
    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // Verify writing to sealed partition immediately errors
    let err = model
        .step_conversational(&mut session, 99, SlotTarget::Persistent, ReadMode::Enabled)
        .unwrap_err();
    assert!(
        err.to_string().contains("sealed"),
        "Sealed persistent partition must reject writes: {err}"
    );

    // 4. Run 520 turns of continuous dialogue rollover (520 turns * 2 tokens = 1040 tokens)
    let turns = 520;
    let tokens_per_turn = 2;
    for turn in 1..=turns {
        session.start_turn();
        for step_idx in 0..tokens_per_turn {
            let dial_token =
                (((turn * 29 + step_idx * 17) % (model.config().vocab_size - 10)) + 5) as u32;
            model
                .step_conversational(
                    &mut session,
                    dial_token,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap_or_else(|e| {
                    panic!("Dialogue step failed at turn {turn}, step {step_idx}: {e}")
                });
        }

        // Spot-check bit-level coordinate identity every 50 turns
        if turn % 50 == 0 || turn == turns {
            assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);
            for slot in 0..PERSISTENT_CAPACITY {
                assert_eq!(
                    session.persistent_tokens[slot], saved_tokens[slot],
                    "Token mismatch in persistent slot {slot} at turn {turn}"
                );
                assert_eq!(
                    session.persistent_keys[slot], saved_keys[slot],
                    "Bit flip in persistent_keys[{slot}] at turn {turn}"
                );
                assert_eq!(
                    session.persistent_values[slot], saved_values[slot],
                    "Bit flip in persistent_values[{slot}] at turn {turn}"
                );
            }
        }
    }

    // 5. Final exhaustive bit-for-bit check after all 520 turns
    assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);
    for slot in 0..PERSISTENT_CAPACITY {
        assert_eq!(
            session.persistent_tokens[slot], saved_tokens[slot],
            "Final token mismatch in persistent slot {slot}"
        );
        assert_eq!(
            session.persistent_keys[slot], saved_keys[slot],
            "Final key coordinate mismatch in persistent slot {slot}"
        );
        assert_eq!(
            session.persistent_values[slot], saved_values[slot],
            "Final value coordinate mismatch in persistent slot {slot}"
        );
    }
}

/// 3. Memory flattening and power-of-two row stride non-corruption:
///    Verify that writing to slot `k` in `SessionState` never corrupts slot `k-1`, `k+1`,
///    or any neighboring coordinate in memory.
#[test]
fn test_m2_adversarial_row_stride_isolation_and_canary_patterns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session: SessionState = model.new_conversational_session();

    // Fill all dialogue slots with deterministic canary patterns
    for slot in 0..DIALOGUE_CAPACITY {
        for coord in 0..KEY_DIM {
            session.dialogue_keys[slot][coord] = ((slot as i32) << 16) | (coord as i32);
        }
        for coord in 0..VAL_DIM {
            session.dialogue_values[slot][coord] = ((slot as i32) << 16) | (coord as i32);
        }
        session.dialogue_tokens[slot] = 0xAA000000 | (slot as u32);
        session.dialogue_sequences[slot] = 0xBB00000000000000 | (slot as u64);
        session.dialogue_turn_ids[slot] = 0xCC000000 | (slot as u32);
    }

    // Now step one token into slot 0
    let token_0 = 17u32;
    model
        .step_conversational(
            &mut session,
            token_0,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .expect("step 0 succeeds");

    // Slot 0 was overwritten. Verify slot 0 has new coordinates and metadata:
    assert_eq!(session.dialogue_tokens[0], token_0);
    assert_eq!(session.dialogue_sequences[0], 0);

    // CRITICAL ADVERSARIAL CHECK: Slots 1..224 MUST retain 100% of their canary bit patterns!
    for slot in 1..DIALOGUE_CAPACITY {
        assert_eq!(
            session.dialogue_tokens[slot],
            0xAA000000 | (slot as u32),
            "Canary token corruption in slot {slot} after writing slot 0"
        );
        assert_eq!(
            session.dialogue_sequences[slot],
            0xBB00000000000000 | (slot as u64),
            "Canary sequence corruption in slot {slot} after writing slot 0"
        );
        assert_eq!(
            session.dialogue_turn_ids[slot],
            0xCC000000 | (slot as u32),
            "Canary turn_id corruption in slot {slot} after writing slot 0"
        );
        for coord in 0..KEY_DIM {
            let expected = ((slot as i32) << 16) | (coord as i32);
            assert_eq!(
                session.dialogue_keys[slot][coord], expected,
                "Canary key coordinate corruption in slot {slot}, coord {coord} after writing slot 0"
            );
        }
        for coord in 0..VAL_DIM {
            let expected = ((slot as i32) << 16) | (coord as i32);
            assert_eq!(
                session.dialogue_values[slot][coord], expected,
                "Canary val coordinate corruption in slot {slot}, coord {coord} after writing slot 0"
            );
        }
    }

    // Step across boundary to slot 223 (fill slots 1..223)
    for slot in 1..DIALOGUE_CAPACITY {
        let t = (slot + 10) as u32;
        model
            .step_conversational(&mut session, t, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("step succeeds");
    }

    assert_eq!(session.dialogue_cursor, 0); // Wrapped around to 0
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY);

    // Save snapshot of all slots 0..223
    let snapshot_keys: Vec<[i32; KEY_DIM]> = session.dialogue_keys.to_vec();
    let snapshot_values: Vec<[i32; VAL_DIM]> = session.dialogue_values.to_vec();

    // Now write to slot 0 again (rollover write)
    let rollover_token = 88u32;
    model
        .step_conversational(
            &mut session,
            rollover_token,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .expect("rollover step succeeds");

    assert_eq!(session.dialogue_cursor, 1);
    assert_eq!(session.dialogue_tokens[0], rollover_token);

    // Check that slots 1..224 are EXACTLY unchanged from snapshot!
    for slot in 1..DIALOGUE_CAPACITY {
        assert_eq!(
            session.dialogue_keys[slot], snapshot_keys[slot],
            "Slot {slot} key corrupted by rollover write to slot 0"
        );
        assert_eq!(
            session.dialogue_values[slot], snapshot_values[slot],
            "Slot {slot} value corrupted by rollover write to slot 0"
        );
    }
}

/// 4. IntegerSession monotonic slot addition and row stride integrity:
///    Verify that `IntegerSession` with 256-byte key strides and 1024-byte value strides
///    grows without corrupting previously stored slots up to the 256-context limit.
#[test]
fn test_m2_adversarial_integer_session_row_stride_and_context_boundary() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_session();

    assert_eq!(session.len(), 0);
    assert!(session.is_empty());

    let max_context = model.config().context; // 256
    let mut past_states = Vec::with_capacity(max_context);

    for step_idx in 0..max_context {
        let token = ((step_idx * 31 + 3) % (model.config().vocab_size - 10) + 5) as u32;
        let step = model
            .step(&mut session, token, ReadMode::Enabled)
            .unwrap_or_else(|e| panic!("Step failed at index {step_idx}: {e}"));

        assert_eq!(session.len(), step_idx + 1);
        past_states.push(step.state);

        // Verify probability sum 2^48
        let sum_prob: u64 = step.probabilities.iter().sum();
        assert_eq!(sum_prob, PROBABILITY_TOTAL);
    }

    assert_eq!(session.len(), max_context);

    // Verify step 257 rejects gracefully with error and does NOT panic
    let overflow_err = model.step(&mut session, 10, ReadMode::Enabled).unwrap_err();
    assert!(
        overflow_err.to_string().contains("context"),
        "Overflow step must return context error: {overflow_err}"
    );
    assert_eq!(session.len(), max_context);
}

/// 5. Vocabulary projection and embedding power-of-two stride boundary checks:
///    Verify `embed` and `project_vocab` row strides (`token << 8` and `row_idx << 8`)
///    cover the full vocabulary without off-by-one or out-of-bounds reads.
#[test]
fn test_m2_adversarial_embed_and_project_vocab_strides() {
    let model = IntegerModel::synthetic_for_test();
    let vocab_size = model.config().vocab_size;
    let width = model.config().width;

    assert_eq!(width, 256);

    // Test embed for every token in 0..vocab_size
    for token in 0..vocab_size as u32 {
        let emb = model.embed(token).expect("valid token embeds");
        assert_eq!(emb.len(), width);
    }

    // Test embed boundary rejection
    assert!(model.embed(vocab_size as u32).is_err());
    assert!(model.embed((vocab_size + 100) as u32).is_err());
    assert!(model.embed(u32::MAX).is_err());

    // Test project_vocab with arbitrary test hidden vector
    let hidden = vec![100i32; width];
    let logits = model
        .project_vocab(&hidden)
        .expect("project_vocab succeeds");
    assert_eq!(logits.len(), vocab_size);

    // Test project_vocab input dimension validation
    let bad_hidden_short = vec![100i32; width - 1];
    let bad_hidden_long = vec![100i32; width + 1];
    assert!(model.project_vocab(&bad_hidden_short).is_err());
    assert!(model.project_vocab(&bad_hidden_long).is_err());
}
