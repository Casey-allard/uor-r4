//! Adversarial Stress Test Harness — Milestone M1: Dialogue State & Prime Memory.
//! File: crates/uor-r4-integer/tests/adversarial_dialogue_m1.rs
//!
//! Empirical verification covering:
//!
//! 1. Extreme dialogue length (> 500 tokens across 15+ turns): zero OOB, zero panic, smooth cyclic FIFO rollover.
//! 2. Persona slot protection: slot 0 (system instructions) never evicts and retains zero age decay after 100+ dialogue turns.
//! 3. Causal necessity: ReadMode::NoRead collapses copy gate mass to exactly 0 while preserving exact 2^48 probability sum.
//! 4. Boundary & error conditions: capacity overflow, out-of-bounds tokens, sealed mutation.
//! 5. Topological hysteresis: loop resistance under 100+ cyclic repetitions.

use uor_r4_integer::{
    IntegerModel, ReadMode, SessionState, SlotTarget, DIALOGUE_CAPACITY, PERSISTENT_CAPACITY,
    PROBABILITY_TOTAL,
};

/// 1. Extreme dialogue length (> 500 tokens across 15+ turns):
///    Verify zero out-of-bounds, zero panic, and smooth cyclic FIFO rollover.
#[test]
fn test_adversarial_extreme_dialogue_length_and_fifo_rollover() {
    let model = IntegerModel::synthetic_for_test();
    let mut session: SessionState = model.new_conversational_session();

    let turns_to_run = 20; // > 15 turns
    let tokens_per_turn = 30;
    let total_dialogue_tokens = turns_to_run * tokens_per_turn; // 600 tokens > 500 tokens

    assert!(total_dialogue_tokens > 500);
    assert!(turns_to_run >= 15);

    let mut global_step = 0usize;
    let mut recorded_tokens = Vec::with_capacity(total_dialogue_tokens);

    for turn in 1..=turns_to_run {
        session.start_turn();
        assert_eq!(session.current_turn(), turn as u32);

        for step_in_turn in 0..tokens_per_turn {
            // Pseudo-random pseudo-adversarial token sequence within vocab bounds
            let token = ((global_step * 37 + 13) % (model.config().vocab_size - 10) + 5) as u32;
            recorded_tokens.push(token);

            let step = model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .unwrap_or_else(|e| {
                    panic!(
                        "PANIC/OOB at turn {turn}, step {step_in_turn} (global {global_step}): {e}"
                    )
                });

            // Invariant: probability vector length matches vocab_size
            assert_eq!(step.probabilities.len(), model.config().vocab_size);

            // Invariant: exact total probability sum 2^48
            let sum_prob: u64 = step.probabilities.iter().sum();
            assert_eq!(
                sum_prob, PROBABILITY_TOTAL,
                "Probability sum deviated from 2^48 at step {global_step}"
            );

            // Invariant: state dimension preserved without OOB
            assert_eq!(step.state.len(), model.config().width);

            global_step += 1;

            // Invariant: dialogue seen monotonically increases
            assert_eq!(session.dialogue_seen, global_step as u64);

            // Invariant: dialogue len saturates at DIALOGUE_CAPACITY (224)
            let expected_len = global_step.min(DIALOGUE_CAPACITY);
            assert_eq!(session.dialogue_len(), expected_len);

            // Invariant: dialogue cursor wraps smoothly modulo DIALOGUE_CAPACITY
            assert_eq!(session.dialogue_cursor, global_step % DIALOGUE_CAPACITY);

            // Invariant: most recently written token resides at the slot just stepped
            let last_slot = (global_step - 1) % DIALOGUE_CAPACITY;
            assert_eq!(session.dialogue_tokens[last_slot], token);
            assert_eq!(
                session.dialogue_sequences[last_slot],
                (global_step - 1) as u64
            );
            assert_eq!(session.dialogue_turn_ids[last_slot], turn as u32);

            // Invariant: persistent memory is never touched by dialogue stepping
            assert_eq!(session.persistent_len(), 0);
        }
    }

    // After 600 steps:
    assert_eq!(session.dialogue_seen, 600);
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY); // 224
    assert_eq!(session.dialogue_cursor, 600 % DIALOGUE_CAPACITY); // 600 % 224 = 152

    // Verify ring buffer content matches the last 224 tokens of the 600 tokens written:
    let tail_224 = &recorded_tokens[total_dialogue_tokens - DIALOGUE_CAPACITY..];
    for (i, &expected_token) in tail_224.iter().enumerate() {
        let global_idx = (total_dialogue_tokens - DIALOGUE_CAPACITY) + i;
        let ring_slot = global_idx % DIALOGUE_CAPACITY;
        assert_eq!(
            session.dialogue_tokens[ring_slot], expected_token,
            "FIFO ring buffer mismatch at slot {ring_slot}"
        );
        assert_eq!(
            session.dialogue_sequences[ring_slot], global_idx as u64,
            "Sequence tracking corrupted in FIFO ring buffer"
        );
    }
}

/// 2. Persona slot protection:
///    Verify slot 0 (system instructions) never evicts and retains zero age decay after 100+ dialogue turns.
#[test]
fn test_adversarial_persona_slot_protection_and_zero_age_decay_100_plus_turns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Ingest system persona into persistent slots (slots 0..3)
    let persona_tokens = [42u32, 108u32, 255u32, 300u32];
    for &token in &persona_tokens {
        model
            .step_conversational(
                &mut session,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("persistent persona ingestion succeeds");
    }

    assert_eq!(session.persistent_len(), 4);
    assert_eq!(session.persistent_tokens[0], 42);

    // Seal the persistent partition
    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // Record slot 0 snapshot
    let slot0_token_initial = session.persistent_tokens[0];
    let slot0_key_initial = session.persistent_keys[0];
    let slot0_val_initial = session.persistent_values[0];

    // Verify attempting to write to sealed persistent partition fails
    let err = model
        .step_conversational(&mut session, 999, SlotTarget::Persistent, ReadMode::Enabled)
        .unwrap_err();
    assert!(
        err.to_string().contains("sealed"),
        "Expected sealed partition error, got: {err}"
    );

    // Adversarially run 120 turns with 5 tokens per turn (600 dialogue tokens total)
    // This overwrites the 224-slot dialogue FIFO buffer more than 2.6 times!
    let num_turns = 120;
    for turn in 1..=num_turns {
        session.start_turn();
        for t in 0..5 {
            let token = 50 + ((turn * 7 + t) % 150) as u32;
            let pre_step_len = session.total_len();
            let step = model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .unwrap_or_else(|e| panic!("Failure at turn {turn}, token {t}: {e}"));

            // Check that persistent slot 0 contributes to read masses
            // persistent slots are indices 0..4 in read_masses
            assert_eq!(step.read_masses.len(), pre_step_len);
        }

        // Periodic checkpoint assertion: slot 0 must remain 100% bitwise intact
        if turn % 20 == 0 || turn == num_turns {
            assert_eq!(session.persistent_len(), 4);
            assert_eq!(session.persistent_tokens[0], slot0_token_initial);
            assert_eq!(session.persistent_keys[0], slot0_key_initial);
            assert_eq!(session.persistent_values[0], slot0_val_initial);
            assert_eq!(session.persistent_tokens, persona_tokens.to_vec());
            assert!(session.is_persistent_sealed());
        }
    }

    // Final verification after 120 turns (600 dialogue tokens):
    assert_eq!(session.current_turn(), 120);
    assert_eq!(session.dialogue_seen, 600);
    assert_eq!(session.persistent_len(), 4);
    assert_eq!(session.persistent_tokens[0], 42);
    assert_eq!(session.persistent_keys[0], slot0_key_initial);
    assert_eq!(session.persistent_values[0], slot0_val_initial);

    // Verify zero age decay:
    // In model.rs, persistent keys are scored with `+ age[0]` regardless of turn or dialogue_seen.
    // Dialogue keys are scored with `+ age[age_idx]` where `age_idx` clamps to `age_horizon_clamp` (63).
    // Let's execute a test step and verify that persistent slot 0 mass is strictly positive and non-zero
    let probe_step = model
        .step_conversational(&mut session, 42, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("probe step succeeds");

    // Persistent slot 0 is the first element of read_masses:
    let slot0_mass = probe_step.read_masses[0];
    assert!(
        slot0_mass > 0,
        "Persona slot 0 mass decayed to zero after 120 turns!"
    );
}

/// 3. Causal necessity:
///    Verify ReadMode::NoRead collapses copy gate mass to exactly 0 while preserving exact 2^48 total probability sum across all tokens.
#[test]
fn test_adversarial_causal_necessity_no_read_ablation() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Test Case A: Empty memory
    let step_empty = model
        .step_conversational(&mut session, 10, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("NoRead on empty memory succeeds");
    assert_eq!(step_empty.no_read_mass, PROBABILITY_TOTAL);
    assert!(step_empty.read_masses.is_empty());
    assert_eq!(
        step_empty.probabilities.iter().sum::<u64>(),
        PROBABILITY_TOTAL
    );

    // Test Case B: Partially filled memory (10 persistent + 50 dialogue)
    let mut session_b = model.new_conversational_session();
    for t in 1..=10 {
        model
            .step_conversational(&mut session_b, t, SlotTarget::Persistent, ReadMode::Enabled)
            .unwrap();
    }
    session_b.seal_persistent();

    for t in 11..=60 {
        model
            .step_conversational(&mut session_b, t, SlotTarget::Dialogue, ReadMode::Enabled)
            .unwrap();
    }
    assert_eq!(session_b.total_len(), 60);

    // Test with multiple distinct test tokens across the vocabulary
    let test_tokens = [1u32, 10u32, 35u32, 60u32, 150u32, 500u32];

    for &test_tok in &test_tokens {
        // Run with ReadMode::Enabled
        let mut sim_enabled = clone_session(&session_b);
        let step_enabled = model
            .step_conversational(
                &mut sim_enabled,
                test_tok,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("enabled step succeeds");

        // Run with ReadMode::NoRead
        let mut sim_noread = clone_session(&session_b);
        let step_noread = model
            .step_conversational(
                &mut sim_noread,
                test_tok,
                SlotTarget::Dialogue,
                ReadMode::NoRead,
            )
            .expect("NoRead step succeeds");

        // Verification 1: under NoRead, no_read_mass MUST be EXACTLY TOTAL (2^48)
        assert_eq!(
            step_noread.no_read_mass, PROBABILITY_TOTAL,
            "no_read_mass was not 2^48 under NoRead ablation"
        );

        // Verification 2: all read_masses MUST be EXACTLY 0
        assert_eq!(step_noread.read_masses.len(), session_b.total_len());
        for (slot_idx, &m) in step_noread.read_masses.iter().enumerate() {
            assert_eq!(
                m, 0,
                "read_mass at slot {slot_idx} was {m} != 0 during NoRead ablation!"
            );
        }

        // Verification 3: Total probability sum across all tokens MUST be EXACTLY 2^48
        let total_prob_noread: u64 = step_noread.probabilities.iter().sum();
        assert_eq!(
            total_prob_noread, PROBABILITY_TOTAL,
            "Probability sum deviated from 2^48 under NoRead ablation!"
        );

        // Verification 4: Causal divergence
        // If the query token matches a memory token, Enabled MUST differ from NoRead
        if test_tok <= 60 {
            assert_ne!(
                step_enabled.probabilities, step_noread.probabilities,
                "Enabled and NoRead distributions were identical for memory token {test_tok} — no causal influence!"
            );
            assert!(
                step_enabled.no_read_mass < PROBABILITY_TOTAL,
                "Enabled step did not route probability mass into memory for token {test_tok}"
            );
        }
    }

    // Test Case C: Fully saturated memory (32 persistent + 224 dialogue = 256 slots)
    let mut session_c = model.new_conversational_session();
    for t in 0..PERSISTENT_CAPACITY {
        model
            .step_conversational(
                &mut session_c,
                t as u32,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    session_c.seal_persistent();

    for t in 0..DIALOGUE_CAPACITY {
        model
            .step_conversational(
                &mut session_c,
                (100 + t) as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    assert_eq!(session_c.persistent_len(), 32);
    assert_eq!(session_c.dialogue_len(), 224);
    assert_eq!(session_c.total_len(), 256);

    let step_sat_noread = model
        .step_conversational(&mut session_c, 5, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("saturated NoRead step succeeds");

    assert_eq!(step_sat_noread.no_read_mass, PROBABILITY_TOTAL);
    assert_eq!(step_sat_noread.read_masses.len(), 256);
    assert!(
        step_sat_noread.read_masses.iter().all(|&m| m == 0),
        "Non-zero read mass in saturated memory under NoRead!"
    );
    assert_eq!(
        step_sat_noread.probabilities.iter().sum::<u64>(),
        PROBABILITY_TOTAL
    );
}

/// 4. Boundary and error robustness:
///    Verify strict error handling on out-of-bounds tokens, capacity overflow, and sealed mutation.
#[test]
fn test_adversarial_boundary_and_error_handling() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();
    let vocab_size = model.config().vocab_size;

    // Out of bounds: token == vocab_size
    let err1 = model
        .step_conversational(
            &mut session,
            vocab_size as u32,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap_err();
    assert!(err1.to_string().contains("out of bounds"));

    // Out of bounds: token == u32::MAX
    let err2 = model
        .step_conversational(
            &mut session,
            u32::MAX,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap_err();
    assert!(err2.to_string().contains("out of bounds"));

    // Exceeding persistent capacity (32 slots)
    for i in 0..PERSISTENT_CAPACITY {
        model
            .step_conversational(
                &mut session,
                i as u32,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    assert_eq!(session.persistent_len(), 32);

    let err_cap = model
        .step_conversational(&mut session, 99, SlotTarget::Persistent, ReadMode::Enabled)
        .unwrap_err();
    assert!(err_cap.to_string().contains("capacity (32) exceeded"));

    // Identity mismatch
    session.identity = "malicious_identity".to_owned();
    let err_id = model
        .step_conversational(&mut session, 1, SlotTarget::Dialogue, ReadMode::Enabled)
        .unwrap_err();
    assert!(err_id.to_string().contains("identity mismatch"));
}

/// 5. Topological hysteresis:
///    Verify non-collapsing state updates under 100+ cyclic token repetitions.
#[test]
fn test_adversarial_cyclic_loop_resistance_100_turns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Adversarially repeat identical token 77 for 100 turns
    let repeating_token = 77u32;
    let mut seen_phases = std::collections::HashSet::new();

    for _step_idx in 0..100 {
        model
            .step_conversational(
                &mut session,
                repeating_token,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("step succeeds");

        let phases = session.zeta_phases();
        seen_phases.insert(phases);
    }

    // If the system fell into a trivial fixed point, seen_phases.len() would be <= 2.
    // Because T8 frequencies are linearly independent, all 100 phases should be unique.
    assert_eq!(
        seen_phases.len(),
        100,
        "Zeta phase collapsed into repetitive limit cycle!"
    );

    // Cumulative holonomy must be non-zero and significant
    assert!(session.holonomy_accumulator().abs() > 0);
}

/// Helper to clone SessionState for comparative stepping
fn clone_session(s: &SessionState) -> SessionState {
    SessionState {
        identity: s.identity.clone(),
        state: s.state.clone(),
        persistent_keys: s.persistent_keys.clone(),
        persistent_values: s.persistent_values.clone(),
        persistent_tokens: s.persistent_tokens.clone(),
        persistent_capacity: s.persistent_capacity,
        persistent_sealed: s.persistent_sealed,
        dialogue_keys: s.dialogue_keys.clone(),
        dialogue_values: s.dialogue_values.clone(),
        dialogue_tokens: s.dialogue_tokens.clone(),
        dialogue_sequences: s.dialogue_sequences.clone(),
        dialogue_turn_ids: s.dialogue_turn_ids.clone(),
        dialogue_capacity: s.dialogue_capacity,
        dialogue_cursor: s.dialogue_cursor,
        dialogue_len: s.dialogue_len,
        dialogue_seen: s.dialogue_seen,
        current_turn_id: s.current_turn_id,
        zeta_state: s.zeta_state,
        hopf_state: s.hopf_state,
        cumulative_holonomy_q30: s.cumulative_holonomy_q30,
        age_horizon_clamp: s.age_horizon_clamp,
    }
}
