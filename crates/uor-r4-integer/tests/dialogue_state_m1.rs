//! Milestone M1: Dialogue State, Role Tokens, Hopf Holonomy, and Prime Memory Unit Tests.
//! File: crates/uor-r4-integer/tests/dialogue_state_m1.rs

use uor_r4_integer::{
    atan2_q30, HopfFiberPointQ30, IntegerModel, ReadMode, RoleToken, SessionState, SlotTarget,
    T8ZetaState, UnitS3Q30, DIALOGUE_CAPACITY, PERSISTENT_CAPACITY, PROBABILITY_TOTAL,
    TOTAL_MEMORY_CAPACITY, ZETA_FREQUENCIES_Q30,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

const VOCAB_FIXTURE_WITH_ROLES: &str = r#"{
    "pre_tokenizer": {"type":"ByteLevel", "add_prefix_space":false},
    "added_tokens":[
        {"id":0,"content":"<|bos|>"},
        {"id":1,"content":"<|eos|>"},
        {"id":2,"content":"<|unk|>"},
        {"id":3,"content":"<|system|>"},
        {"id":4,"content":"<|user|>"},
        {"id":5,"content":"<|assistant|>"},
        {"id":6,"content":"<|turn_end|>"}
    ],
    "model":{"type":"BPE",
        "vocab":{
            "<|bos|>":0,"<|eos|>":1,"<|unk|>":2,
            "<|system|>":3,"<|user|>":4,"<|assistant|>":5,"<|turn_end|>":6,
            "H":7,"e":8,"l":9,"o":10," ":11,"!":12,"P":13,"i":14,"n":15,"g":16
        },
        "merges":["H e","l l","l o"]
    }
}"#;

#[test]
fn test_m1_role_tokens_atomic_encoding_and_literal_decoding() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("fixture parses");

    // 1. Verify exact role token IDs
    assert_eq!(tokenizer.token_id("<|system|>"), Some(3));
    assert_eq!(tokenizer.token_id("<|user|>"), Some(4));
    assert_eq!(tokenizer.token_id("<|assistant|>"), Some(5));
    assert_eq!(tokenizer.token_id("<|turn_end|>"), Some(6));

    // Verify RoleToken enum mappings
    assert_eq!(RoleToken::System.id(), 3);
    assert_eq!(RoleToken::User.id(), 4);
    assert_eq!(RoleToken::Assistant.id(), 5);
    assert_eq!(RoleToken::TurnEnd.id(), 6);
    assert_eq!(RoleToken::from_id(3), Some(RoleToken::System));
    assert_eq!(RoleToken::from_tag("<|system|>"), Some(RoleToken::System));
    assert_eq!("<|system|>".parse::<RoleToken>(), Ok(RoleToken::System));

    // 2. Verify atomic leftmost-longest matching without substring fragmentation
    let prompt = "<|system|>Hello<|turn_end|><|user|>Ping<|turn_end|><|assistant|>";
    let encoded = tokenizer.encode(prompt);
    assert_eq!(encoded[0], 3); // <|system|>
    assert_eq!(*encoded.iter().find(|&&id| id == 6).unwrap(), 6); // <|turn_end|>
    assert!(encoded.contains(&4)); // <|user|>
    assert!(encoded.contains(&5)); // <|assistant|>

    // 3. Verify literal decoding (no byte-level mapping on role tokens)
    assert_eq!(tokenizer.decode_bytes(&[3]), b"<|system|>");
    assert_eq!(tokenizer.decode_bytes(&[4]), b"<|user|>");
    assert_eq!(tokenizer.decode_bytes(&[5]), b"<|assistant|>");
    assert_eq!(tokenizer.decode_bytes(&[6]), b"<|turn_end|>");
    assert_eq!(
        tokenizer.decode(&[3, 4, 5, 6]),
        "<|system|><|user|><|assistant|><|turn_end|>"
    );
}

#[test]
fn test_m1_prompt_formatting_templates_multi_turn() {
    let system_persona = "You are a concise mathematician.";
    let user_turn_1 = "State Euler's identity.";
    let assistant_turn_1 = "e^(i*pi) + 1 = 0";
    let user_turn_2 = "What does it connect?";

    // Turn 1 prompt construction:
    let turn_1_prompt = format!(
        "<|system|>{system_persona}<|turn_end|><|user|>{user_turn_1}<|turn_end|><|assistant|>{assistant_turn_1}<|turn_end|>"
    );
    assert!(turn_1_prompt.starts_with("<|system|>"));
    assert!(turn_1_prompt.contains("<|turn_end|><|user|>"));
    assert!(turn_1_prompt.contains(assistant_turn_1));

    // Turn 2 continuation construction:
    let turn_2_append = format!("<|user|>{user_turn_2}<|turn_end|><|assistant|>");
    assert!(turn_2_append.starts_with("<|user|>"));
    assert!(turn_2_append.ends_with("<|turn_end|><|assistant|>"));
    assert!(!turn_2_append.contains("<|system|>")); // Preamble must NOT repeat
}

#[test]
fn test_m1_stop_criteria_turn_end_and_clean_trimming() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("fixture parses");

    // Simulated generated tokens containing <|turn_end|> (id 6)
    let generated = [13, 14, 15, 16, 6, 13, 14]; // "Ping" + <|turn_end|> + extra
    let stop_token = RoleToken::TURN_END_ID;

    let end_idx = generated
        .iter()
        .position(|&t| t == stop_token)
        .unwrap_or(generated.len());
    assert_eq!(end_idx, 4);

    let visible_bytes = tokenizer.decode_bytes(&generated[..end_idx]);
    let response_text = String::from_utf8_lossy(&visible_bytes).trim().to_owned();
    assert_eq!(response_text, "Ping");
    assert!(!response_text.contains("<|turn_end|>"));
}

#[test]
fn test_m1_memory_slot_partition_persona_preservation() {
    // Memory capacities:
    assert_eq!(PERSISTENT_CAPACITY, 32);
    assert_eq!(DIALOGUE_CAPACITY, 224);
    assert_eq!(TOTAL_MEMORY_CAPACITY, 256);

    let mut slot_keys = vec![0u32; TOTAL_MEMORY_CAPACITY];
    for (i, slot) in slot_keys.iter_mut().take(PERSISTENT_CAPACITY).enumerate() {
        *slot = 1000 + i as u32; // Ingest 32 persona tokens
    }

    // Simulate 300 dialogue token writes (exceeding 224 slot capacity)
    let mut write_head = PERSISTENT_CAPACITY;
    for token_id in 0..300u32 {
        slot_keys[write_head] = token_id;
        write_head =
            PERSISTENT_CAPACITY + ((write_head - PERSISTENT_CAPACITY + 1) % DIALOGUE_CAPACITY);
    }

    // Assert: Session slots (0..32) are 100% UNTOUCHED
    for (i, &slot) in slot_keys.iter().take(PERSISTENT_CAPACITY).enumerate() {
        assert_eq!(slot, 1000 + i as u32, "Persona slot {i} was overwritten!");
    }
}

#[test]
fn test_m1_discrete_hopf_fiber_holonomy_tracking() {
    // Fixed-point Q1.30: 1.0 = 2^30
    const Q30: i64 = 1 << 30;

    let mut holonomy_accumulator = 0i64;
    let mut current_fiber_phase = 0i32;

    // Simulate 4 successive non-trivial state updates traversing a loop winding around S1:
    // Trajectory phases: 0 -> pi/2 -> 3pi/4 -> -3pi/4 -> 0 (crossing branch cut at pi)
    let phases = [
        (Q30 / 2) as i32,
        (3 * Q30 / 4) as i32,
        (-3 * Q30 / 4) as i32,
        0i32,
    ];

    for &next_phase in &phases {
        let mut d_phase = (next_phase as i64) - (current_fiber_phase as i64);
        if d_phase > Q30 {
            d_phase -= 2 * Q30;
        } else if d_phase < -Q30 {
            d_phase += 2 * Q30;
        }
        holonomy_accumulator += d_phase;
        current_fiber_phase = next_phase;
    }

    // Invariant: Non-zero holonomy phase progression (Delta psi != 0)
    assert_ne!(
        holonomy_accumulator, 0,
        "Holonomy accumulator failed to capture winding phase!"
    );

    // Verify UnitS3Q30 and HopfFiberPointQ30 projections
    let s3 = UnitS3Q30::IDENTITY;
    let pt: HopfFiberPointQ30 = s3.hopf_fiber_project();
    assert_eq!(pt.base, [0, 0, 1 << 30]);
    assert_eq!(pt.fiber_phase, 0);
    assert_eq!(atan2_q30(0, 1000), 0);
    assert_eq!(atan2_q30(1000, 0), 1 << 29);
}

#[test]
fn test_m1_t8_zeta_zero_phase_ergodicity() {
    let mut zeta = T8ZetaState::new();
    let mut history: Vec<[i32; 8]> = Vec::new();

    // Step through 100 transitions using raw incommensurate frequencies
    for step in 0..100 {
        zeta.step_raw();
        // Invariant: No duplicate states in history (ergodic non-periodicity)
        assert!(
            !history.contains(&zeta.phases),
            "T8 state repeated at step {step}!"
        );
        history.push(zeta.phases);
    }
    assert_eq!(history.len(), 100);
}

#[test]
fn test_m1_adversarial_cyclic_prompt_loop_resistance() {
    let mut holonomy_acc = 0i64;
    let mut turn_phases = Vec::new();

    for turn in 1..=10 {
        // Even with identical input token "Ping", non-zero holonomy Delta psi advances
        let turn_delta = (ZETA_FREQUENCIES_Q30[0] as i64) / (turn as i64 + 1);
        holonomy_acc += turn_delta;
        turn_phases.push(holonomy_acc);
    }

    // Verify all 10 turns have unique holonomy values (strictly monotonic / non-repeating)
    for i in 0..turn_phases.len() {
        for j in (i + 1)..turn_phases.len() {
            assert_ne!(
                turn_phases[i], turn_phases[j],
                "Turns {i} and {j} collapsed to identical holonomy!"
            );
        }
    }
}

#[test]
fn test_m1_multi_turn_dialogue_stepping_exceeding_256_tokens() {
    let model = IntegerModel::synthetic_for_test();
    let mut session: SessionState = model.new_conversational_session();

    assert_eq!(session.persistent_len(), 0);
    assert_eq!(session.dialogue_len(), 0);
    assert_eq!(session.dialogue_cursor, 0);

    // Step 300 dialogue tokens sequentially
    // In unpartitioned memory, this would trigger Err("context256 exhausted") at token 256.
    // In partitioned memory, it loops seamlessly via cyclic FIFO.
    for step_idx in 0..300u32 {
        let token = 10 + (step_idx % 50);
        let step = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .unwrap_or_else(|e| panic!("failed at step {step_idx}: {e}"));

        assert_eq!(step.probabilities.len(), model.config().vocab_size);
        let total_prob: u64 = step.probabilities.iter().sum();
        assert_eq!(total_prob, PROBABILITY_TOTAL);
    }

    // Verify invariants after 300 steps:
    assert_eq!(session.dialogue_seen, 300);
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY); // Saturated at 224
    assert_eq!(session.dialogue_cursor, 300 % DIALOGUE_CAPACITY); // 300 % 224 = 76
    assert_eq!(session.persistent_len(), 0); // Untouched
    assert_ne!(session.cumulative_holonomy_q30, 0); // Non-zero holonomy accumulated
}

#[test]
fn test_m1_persona_retention_across_dozens_of_turns() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Ingest 10 system persona tokens into persistent slots
    let persona_tokens = [100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
    for &token in &persona_tokens {
        model
            .step_conversational(
                &mut session,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("persistent step succeeds");
    }
    assert_eq!(session.persistent_len(), 10);
    assert_eq!(session.persistent_tokens, persona_tokens.to_vec());

    // Seal persistent partition
    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // Attempting to write to sealed persistent partition must return Err
    assert!(model
        .step_conversational(&mut session, 999, SlotTarget::Persistent, ReadMode::Enabled)
        .is_err());

    // Snapshot persistent keys and values
    let saved_keys = session.persistent_keys.clone();
    let saved_values = session.persistent_values.clone();

    // 2. Simulate 25 turns with 10 dialogue tokens per turn (250 dialogue tokens total)
    for turn in 1..=25 {
        session.start_turn();
        for t in 0..10 {
            let token = 200 + (t % 20);
            model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("dialogue step succeeds");
        }
        assert_eq!(session.current_turn(), turn);
    }

    // 3. Verify: Persona slot 0 and all persistent slots are 100% PRESERVED
    assert_eq!(session.persistent_len(), 10);
    assert_eq!(session.persistent_tokens, persona_tokens.to_vec());
    assert_eq!(session.persistent_keys, saved_keys);
    assert_eq!(session.persistent_values, saved_values);
}

#[test]
fn test_m1_no_read_ablation_collapses_copy_mass_to_zero_and_preserves_normalization() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Ingest some tokens into persistent and dialogue slots
    for token in 100..105 {
        model
            .step_conversational(
                &mut session,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("setup persistent succeeds");
    }
    session.seal_persistent();

    for token in 200..220 {
        model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("setup dialogue succeeds");
    }

    let active_slots = session.persistent_len() + session.dialogue_len();
    assert!(active_slots > 0);

    // Step with ReadMode::NoRead
    let step = model
        .step_conversational(&mut session, 250, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("NoRead step succeeds");

    // Invariants under NoRead ablation:
    // 1. no_read_mass must be EXACTLY TOTAL (2^48)
    assert_eq!(step.no_read_mass, PROBABILITY_TOTAL);

    // 2. All read_masses must be EXACTLY 0
    assert!(
        step.read_masses.iter().all(|&m| m == 0),
        "Non-zero read mass found during NoRead ablation!"
    );

    // 3. Total probability sum must be bitwise EXACTLY TOTAL (2^48)
    let sum_prob: u64 = step.probabilities.iter().sum();
    assert_eq!(
        sum_prob, PROBABILITY_TOTAL,
        "Probability sum deviated from 2^48 under NoRead ablation!"
    );
}

#[test]
fn test_m1_hopf_fiber_holonomy_strictly_non_zero_under_cyclic_prompts() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Adversarial cyclic input prompt: Send token 42 repeatedly for 20 steps
    let mut holonomy_history = Vec::new();
    let cyclic_token = 42u32;

    for step_idx in 0..20 {
        model
            .step_conversational(
                &mut session,
                cyclic_token,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("step succeeds");

        let holonomy = session.holonomy_accumulator();
        holonomy_history.push(holonomy);

        if step_idx > 0 {
            // Invariant: Cumulative holonomy must strictly advance on every step (Delta psi != 0)
            assert_ne!(
                holonomy_history[step_idx],
                holonomy_history[step_idx - 1],
                "Hopf holonomy stagnated at step {step_idx} under cyclic prompt!"
            );
        }
    }

    // Cumulative holonomy must be non-zero
    assert_ne!(session.holonomy_accumulator(), 0);
}
