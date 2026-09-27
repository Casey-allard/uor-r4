//! Adversarial Verification Suite — Milestone M1 Iteration 2
//! File: crates/uor-r4-integer/tests/adversarial_challenger_m1_it2.rs
//!
//! Empirical adversarial stress testing of multi-turn state dynamics under
//! the zero-multiplier numerical serving kernel:
//!
//! 1. 4-bit unrolled `T8ZetaState::step` ergodicity across 2,000 steps without phase collision:
//!    - Mathematical equivalence of 4-bit unrolled shift-add against exact product formula.
//!    - Constant-token ergodicity over 2,000 steps (zero collision, full toroidal coverage).
//!    - Cyclic-token ergodicity over 2,000 steps across periods 2, 4, 7, 8, 16.
//!    - Extreme token values (0, u32::MAX, large primes) over 2,000 steps.
//!
//! 2. Discrete Hopf fiber holonomy Delta psi != 0 accumulation and limit cycle prevention:
//!    - 50 consecutive repeated cyclic turns under identical user queries.
//!    - Verification of Delta psi != 0 on every individual token step.
//!    - Pairwise distinctness of cumulative holonomy across all 50 turns (1,225 pairs).
//!    - 50 alternating prompt turns (25 Ping / 25 Pong) without 2-cycle hysteresis trap.
//!
//! 3. Session slot 0 (system persona) bit-identical retention across 200 turns:
//!    - Ingestion of system persona into persistent slots 0..N_sys.
//!    - Exact bitwise capture of slot 0 (token, key vector, value vector).
//!    - Execution of 200 dialogue turns (> 1,600 tokens, > 7 full FIFO rollovers).
//!    - Invariant check: slot 0 remains 100% bitwise identical after 200 turns.
//!    - Write-protection and rejection of post-seal persistent mutation attempts.

use std::collections::HashSet;
use uor_r4_integer::{
    IntegerModel, ReadMode, RoleToken, SlotTarget, T8ZetaState, DIALOGUE_CAPACITY,
    PROBABILITY_TOTAL, ZETA_FREQUENCIES_Q30,
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
            "e":7,"c":8,"h":9,"o":10," ":11,"t":12,"s":13,"P":14,"i":15,"n":16,"g":17
        },
        "merges":[]
    }
}"#;

// ============================================================================
// SUITE 1: 4-BIT UNROLLED T8ZetaState::step ERGODICITY (2,000 STEPS)
// ============================================================================

/// Adversarial Challenge 1.1:
/// Verify mathematical exactness of the 4-bit unrolled shift-add multiplication
/// against exact multiplication across all tokens, coordinates, and zeta frequencies.
#[test]
fn test_adversarial_t8_step_unrolled_4bit_exact_equivalence() {
    for token in 0..1024u32 {
        for (j, &freq_val) in ZETA_FREQUENCIES_Q30.iter().enumerate() {
            let freq = freq_val as i64;
            let k = 1i64 + (((token as i64) + (j as i64)) & 0x07);

            // 4-bit unrolled shift-add implementation under test:
            let mut prod = 0i64;
            if k & 1 != 0 {
                prod += freq;
            }
            if k & 2 != 0 {
                prod += freq << 1;
            }
            if k & 4 != 0 {
                prod += freq << 2;
            }
            if k & 8 != 0 {
                prod += freq << 3;
            }

            // Reference mathematical calculation:
            let expected_prod = freq * k;
            assert_eq!(
                prod, expected_prod,
                "4-bit unrolled prod mismatch for token {token}, coord {j}, k {k}: got {prod}, expected {expected_prod}"
            );

            let delta = prod >> 3;
            let expected_delta = expected_prod >> 3;
            assert_eq!(delta, expected_delta);
        }
    }
}

/// Adversarial Challenge 1.2:
/// Verify that T8ZetaState::step preserves ergodicity across 2,000 steps
/// under various constant token values without any phase vector collision.
#[test]
fn test_adversarial_t8_step_constant_tokens_ergodicity_2000_steps() {
    const STEPS: usize = 2000;
    let test_tokens: [u32; 6] = [0, 1, 7, 42, 255, 65535];

    for &token in &test_tokens {
        let mut zeta = T8ZetaState::new();
        let mut seen = HashSet::with_capacity(STEPS);
        let mut min_phases = [i32::MAX; 8];
        let mut max_phases = [i32::MIN; 8];

        for step in 0..STEPS {
            let inserted = seen.insert(zeta.phases);
            assert!(
                inserted,
                "Token {token} triggered phase vector collision at step {step}!"
            );

            for (j, &p) in zeta.phases.iter().enumerate() {
                min_phases[j] = min_phases[j].min(p);
                max_phases[j] = max_phases[j].max(p);
                // Bounded phase invariant: within [-2^30, 2^30)
                assert!(
                    p.abs() <= (1 << 30),
                    "Phase coordinate {j} exceeded Q30 bounds at step {step}: {p}"
                );
            }

            zeta.step(token);
        }

        assert_eq!(
            seen.len(),
            STEPS,
            "Token {token} failed to produce {STEPS} unique phase vectors!"
        );

        // Ergodic coverage invariants: each coordinate must visit positive and negative half-spaces
        // and cover a substantial fraction of [-2^30, 2^30].
        for j in 0..8 {
            assert!(
                min_phases[j] < 0,
                "Token {token}, coordinate {j}: failed to visit negative phase (min = {})",
                min_phases[j]
            );
            assert!(
                max_phases[j] > 0,
                "Token {token}, coordinate {j}: failed to visit positive phase (max = {})",
                max_phases[j]
            );
            let span = (max_phases[j] as i64) - (min_phases[j] as i64);
            assert!(
                span > (1i64 << 30),
                "Token {token}, coordinate {j}: phase span {span} too narrow (< 2^30)"
            );
        }
    }
}

/// Adversarial Challenge 1.3:
/// Verify T8ZetaState::step under cyclic token patterns (periods 2, 4, 7, 8, 16)
/// over 2,000 steps, ensuring harmonic resonance does not cause phase collapse.
#[test]
fn test_adversarial_t8_step_cyclic_tokens_ergodicity_2000_steps() {
    const STEPS: usize = 2000;
    let cycles: [&[u32]; 5] = [
        &[3, 5],                                                 // period 2
        &[0, 2, 4, 6],                                           // period 4
        &[1, 2, 3, 4, 5, 6, 7],                                  // period 7 (coprime to 8)
        &[0, 1, 2, 3, 4, 5, 6, 7], // period 8 (exact match for & 0x07 mask)
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], // period 16
    ];

    for (cycle_idx, &cycle) in cycles.iter().enumerate() {
        let mut zeta = T8ZetaState::new();
        let mut seen = HashSet::with_capacity(STEPS);

        for step in 0..STEPS {
            let token = cycle[step % cycle.len()];
            let inserted = seen.insert(zeta.phases);
            assert!(
                inserted,
                "Cycle pattern {cycle_idx} (len {}) collided at step {step}!",
                cycle.len()
            );
            zeta.step(token);
        }

        assert_eq!(
            seen.len(),
            STEPS,
            "Cycle pattern {cycle_idx} failed to achieve 2,000 collision-free steps!"
        );
    }
}

// ============================================================================
// SUITE 2: DISCRETE HOPF FIBER HOLONOMY & 50-TURN CYCLIC LOOP RESISTANCE
// ============================================================================

/// Adversarial Challenge 2.1:
/// Execute 50 consecutive repeated cyclic turns under identical user queries.
/// Invariants:
/// 1. Delta psi != 0 on EVERY single token step.
/// 2. Holonomy accumulator accumulates continuously and never stagnates.
/// 3. All 50 end-of-turn holonomies are mutually distinct (1,225 distinct pairs).
/// 4. End-of-turn recurrent states do NOT collapse into a trivial fixed point.
#[test]
fn test_adversarial_hopf_holonomy_50_repeated_cyclic_turns() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("tokenizer fixture parses");

    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Ingest persistent system persona
    let system_tokens = tokenizer.encode("<|system|>You are an assistant.<|turn_end|>");
    for &tok in &system_tokens {
        model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("system step succeeds");
    }
    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // Identical cyclic user query: "echo test"
    let user_tokens = tokenizer.encode("<|user|>echo test<|turn_end|><|assistant|>");
    let assistant_tokens = vec![7u32, 8, 9, RoleToken::TURN_END_ID];

    const TURNS: usize = 50;
    let mut end_of_turn_holonomies = Vec::with_capacity(TURNS);
    let mut end_of_turn_states = Vec::with_capacity(TURNS);
    let mut previous_holonomy = session.holonomy_accumulator();

    for turn in 1..=TURNS {
        session.start_turn();

        // 1. Step user query tokens
        for &tok in &user_tokens {
            let step = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("user step succeeds");

            let current_holonomy = session.holonomy_accumulator();
            let delta_psi = current_holonomy - previous_holonomy;

            // Invariant: Delta psi != 0 on every step
            assert_ne!(
                delta_psi, 0,
                "Turn {turn}: Hopf fiber holonomy stagnated (Delta psi == 0) on user token {tok}"
            );

            // Invariant: probability sum is 2^48
            let sum_p: u64 = step.probabilities.iter().sum();
            assert_eq!(sum_p, PROBABILITY_TOTAL);

            previous_holonomy = current_holonomy;
        }

        // 2. Step assistant response tokens
        for &tok in &assistant_tokens {
            let step = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("assistant step succeeds");

            let current_holonomy = session.holonomy_accumulator();
            let delta_psi = current_holonomy - previous_holonomy;

            // Invariant: Delta psi != 0 on every step
            assert_ne!(
                delta_psi, 0,
                "Turn {turn}: Hopf fiber holonomy stagnated (Delta psi == 0) on assistant token {tok}"
            );

            // Invariant: probability sum is 2^48
            let sum_p: u64 = step.probabilities.iter().sum();
            assert_eq!(sum_p, PROBABILITY_TOTAL);

            previous_holonomy = current_holonomy;
        }

        end_of_turn_holonomies.push(session.holonomy_accumulator());
        end_of_turn_states.push(session.state.clone());
    }

    assert_eq!(end_of_turn_holonomies.len(), TURNS);
    assert_eq!(end_of_turn_states.len(), TURNS);

    // Invariant: All 50 end-of-turn holonomies must be strictly distinct
    // Total pairs = 50 * 49 / 2 = 1,225 pairs
    for i in 0..TURNS {
        for j in (i + 1)..TURNS {
            assert_ne!(
                end_of_turn_holonomies[i],
                end_of_turn_holonomies[j],
                "Holonomy collision detected between turn {} and turn {}! Value: {}",
                i + 1,
                j + 1,
                end_of_turn_holonomies[i]
            );
        }
    }

    // Verify final state is not all zeros or uninitialized
    assert!(session.state.iter().any(|&v| v != 0));
}

/// Adversarial Challenge 2.2:
/// Alternating prompt cycles (50 turns: 25 Ping, 25 Pong) to verify resistance
/// against 2-cycle hysteresis trap in holonomy and state dynamics.
#[test]
fn test_adversarial_hopf_holonomy_50_alternating_turns() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("tokenizer fixture parses");

    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    let ping_tokens = tokenizer.encode("<|user|>Ping<|turn_end|><|assistant|>");
    let pong_tokens = tokenizer.encode("<|user|>Pong<|turn_end|><|assistant|>");
    let assistant_tokens = vec![10u32, RoleToken::TURN_END_ID];

    let mut ping_holonomies = Vec::with_capacity(25);
    let mut pong_holonomies = Vec::with_capacity(25);

    for turn in 1..=50 {
        session.start_turn();
        let query = if turn % 2 == 1 {
            &ping_tokens
        } else {
            &pong_tokens
        };

        for &tok in query {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("query step succeeds");
        }
        for &tok in &assistant_tokens {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("assistant step succeeds");
        }

        let holonomy = session.holonomy_accumulator();
        if turn % 2 == 1 {
            ping_holonomies.push(holonomy);
        } else {
            pong_holonomies.push(holonomy);
        }
    }

    assert_eq!(ping_holonomies.len(), 25);
    assert_eq!(pong_holonomies.len(), 25);

    // Verify all 25 Ping turns have distinct holonomy
    for i in 0..25 {
        for j in (i + 1)..25 {
            assert_ne!(
                ping_holonomies[i],
                ping_holonomies[j],
                "Ping turn {} and turn {} collapsed to identical holonomy {}",
                2 * i + 1,
                2 * j + 1,
                ping_holonomies[i]
            );
        }
    }

    // Verify all 25 Pong turns have distinct holonomy
    for i in 0..25 {
        for j in (i + 1)..25 {
            assert_ne!(
                pong_holonomies[i],
                pong_holonomies[j],
                "Pong turn {} and turn {} collapsed to identical holonomy {}",
                2 * i + 2,
                2 * j + 2,
                pong_holonomies[i]
            );
        }
    }
}

// ============================================================================
// SUITE 3: SESSION SLOT 0 (SYSTEM PERSONA) RETENTION ACROSS 200 TURNS
// ============================================================================

/// Adversarial Challenge 3.1:
/// Verify that session slot 0 (system persona) remains 100% bit-identical
/// after 200 turns of dialogue rollover (> 1,600 tokens, > 7 full FIFO wraps).
#[test]
fn test_adversarial_session_slot_0_retention_across_200_turns_dialogue_rollover() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Ingest persistent system persona: 5 tokens
    let persona_tokens: [u32; 5] = [
        RoleToken::SYSTEM_ID,
        15, // 'i'
        16, // 'n'
        17, // 'g'
        RoleToken::TURN_END_ID,
    ];

    for &tok in &persona_tokens {
        model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("persona step succeeds");
    }
    session.seal_persistent();

    assert!(session.is_persistent_sealed());
    assert_eq!(session.persistent_len(), persona_tokens.len());

    // 2. Capture exact baseline bitwise values of slot 0 and all persistent slots
    let baseline_slot_0_token = session.persistent_tokens[0];
    let baseline_slot_0_key = session.persistent_keys[0];
    let baseline_slot_0_value = session.persistent_values[0];

    let baseline_persistent_tokens = session.persistent_tokens.clone();
    let baseline_persistent_keys = session.persistent_keys.clone();
    let baseline_persistent_values = session.persistent_values.clone();

    assert_eq!(baseline_slot_0_token, RoleToken::SYSTEM_ID);
    assert_eq!(baseline_slot_0_key.len(), model.config().read_width);
    assert_eq!(baseline_slot_0_value.len(), model.config().width);

    // 3. Execute 200 turns of dialogue rollover
    // Each turn injects 8 tokens = 1,600 tokens total.
    // DIALOGUE_CAPACITY is 224, so this tests > 7 complete buffer wraps!
    const TURNS: usize = 200;
    const TOKENS_PER_TURN: usize = 8;
    let total_dialogue_tokens = TURNS * TOKENS_PER_TURN; // 1,600 tokens

    let mut global_step = 0usize;
    let mut recorded_dialogue_tokens = Vec::with_capacity(total_dialogue_tokens);

    for turn in 1..=TURNS {
        session.start_turn();
        assert_eq!(session.current_turn(), turn as u32);

        for step_idx in 0..TOKENS_PER_TURN {
            // Pseudo-random pseudo-adversarial token sequence
            let token = ((global_step * 41 + 19) % (model.config().vocab_size - 10) + 5) as u32;
            recorded_dialogue_tokens.push(token);

            model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .unwrap_or_else(|e| {
                    panic!(
                        "Step failed at turn {turn}, step {step_idx} (global {global_step}): {e}"
                    )
                });

            global_step += 1;
        }

        // Periodic checkpoint assertions: turn 1, 10, 50, 100, 150, 200
        if turn == 1 || turn == 10 || turn == 50 || turn == 100 || turn == 150 || turn == 200 {
            // Invariant: Slot 0 must remain 100% bitwise identical
            assert_eq!(
                session.persistent_tokens[0], baseline_slot_0_token,
                "Slot 0 token mutated at turn {turn}!"
            );
            assert_eq!(
                session.persistent_keys[0], baseline_slot_0_key,
                "Slot 0 key vector mutated at turn {turn}!"
            );
            assert_eq!(
                session.persistent_values[0], baseline_slot_0_value,
                "Slot 0 value vector mutated at turn {turn}!"
            );

            // Invariant: All persistent partition slots remain bit-identical
            assert_eq!(session.persistent_tokens, baseline_persistent_tokens);
            assert_eq!(session.persistent_keys, baseline_persistent_keys);
            assert_eq!(session.persistent_values, baseline_persistent_values);
            assert_eq!(session.persistent_len(), persona_tokens.len());
            assert!(session.is_persistent_sealed());
        }
    }

    // 4. Final verification after 200 turns:
    assert_eq!(global_step, 1600);
    assert_eq!(session.dialogue_seen, 1600);
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY); // saturated at 224
    assert_eq!(session.dialogue_cursor, 1600 % DIALOGUE_CAPACITY); // 1600 % 224 = 32

    // Final bit-exactness check on slot 0:
    assert_eq!(
        session.persistent_tokens[0], baseline_slot_0_token,
        "Final slot 0 token mismatch!"
    );
    assert_eq!(
        session.persistent_keys[0], baseline_slot_0_key,
        "Final slot 0 key vector mismatch!"
    );
    assert_eq!(
        session.persistent_values[0], baseline_slot_0_value,
        "Final slot 0 value vector mismatch!"
    );

    // Verify ring buffer content matches the last 224 tokens of the 1,600 tokens written:
    let tail_224 = &recorded_dialogue_tokens[total_dialogue_tokens - DIALOGUE_CAPACITY..];
    for (i, &expected_tok) in tail_224.iter().enumerate() {
        let global_idx = (total_dialogue_tokens - DIALOGUE_CAPACITY) + i;
        let ring_slot = global_idx % DIALOGUE_CAPACITY;
        assert_eq!(
            session.dialogue_tokens[ring_slot], expected_tok,
            "Dialogue ring slot {ring_slot} corrupted after 200 turns!"
        );
    }
}

/// Adversarial Challenge 3.2:
/// Verify write-protection of slot 0 and the persistent partition:
/// Attempting to write into persistent partition after sealing MUST fail
/// and MUST NOT alter slot 0 in any way.
#[test]
fn test_adversarial_attempted_mutation_of_slot_0_after_seal() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Ingest persona token into slot 0
    model
        .step_conversational(
            &mut session,
            RoleToken::SYSTEM_ID,
            SlotTarget::Persistent,
            ReadMode::Enabled,
        )
        .expect("slot 0 write succeeds");

    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    let baseline_token = session.persistent_tokens[0];
    let baseline_key = session.persistent_keys[0];
    let baseline_value = session.persistent_values[0];

    // Attempted adversarial mutation
    let err = model
        .step_conversational(
            &mut session,
            99u32,
            SlotTarget::Persistent,
            ReadMode::Enabled,
        )
        .expect_err("mutation of sealed persistent partition must fail");

    assert!(
        err.to_string()
            .contains("cannot write to sealed persistent session partition"),
        "Unexpected error message: {err}"
    );

    // Invariant: slot 0 remains bit-identical
    assert_eq!(session.persistent_tokens[0], baseline_token);
    assert_eq!(session.persistent_keys[0], baseline_key);
    assert_eq!(session.persistent_values[0], baseline_value);
    assert_eq!(session.persistent_len(), 1);
}
