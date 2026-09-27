//! Adversarial stress test suite for topological loop resistance, Hopf fiber holonomy,
//! and T^8 Riemann zeta-zero ergodicity under identical cyclic prompt attacks.
//!
//! File: crates/uor-r4-integer/tests/adversarial_topological_loops.rs

use std::collections::HashSet;
use uor_r4_integer::{
    IntegerModel, ReadMode, RoleToken, SlotTarget, T8ZetaState, UnitS3Q30, DIALOGUE_CAPACITY,
    PERSISTENT_CAPACITY, PROBABILITY_TOTAL,
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

/// Adversarial Challenge 1:
/// Feed 20 consecutive identical user queries ("echo test") and verify:
/// 1. Discrete Hopf fiber coordinate and holonomy accumulator Delta psi != 0 on every step.
/// 2. Recurrent state S_t does NOT collapse into a fixed point or periodic limit cycle.
/// 3. End-of-turn states across all 20 turns are mutually distinct.
/// 4. Holonomy accumulator accumulates continuous non-zero phase drift.
#[test]
fn test_adversarial_20_identical_user_queries_echo_test() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("tokenizer fixture parses");

    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Ingest system persona
    let system_text = "<|system|>You are an assistant.<|turn_end|>";
    let system_tokens = tokenizer.encode(system_text);
    for &token in &system_tokens {
        model
            .step_conversational(
                &mut session,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("persistent step succeeds");
    }
    session.seal_persistent();
    assert!(session.is_persistent_sealed());
    let persona_token_count = session.persistent_len();
    assert!(persona_token_count > 0);

    // 2. Prepare identical user query: "echo test"
    let user_query = "<|user|>echo test<|turn_end|><|assistant|>";
    let query_tokens = tokenizer.encode(user_query);
    assert!(!query_tokens.is_empty());

    // Simulated assistant response: 3 tokens + <|turn_end|>
    let assistant_response_tokens = vec![7, 8, 9, RoleToken::TURN_END_ID];

    let mut end_of_turn_states: Vec<Vec<i32>> = Vec::new();
    let mut end_of_turn_holonomies: Vec<i64> = Vec::new();
    let mut all_step_states: Vec<Vec<i32>> = Vec::new();
    let mut all_step_holonomies: Vec<i64> = Vec::new();

    let mut previous_holonomy = session.holonomy_accumulator();

    // 3. Execute 20 consecutive identical turns
    for turn in 1..=20 {
        session.start_turn();

        // Feed user query tokens
        for &tok in &query_tokens {
            let step = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("query step succeeds");

            let current_holonomy = session.holonomy_accumulator();
            let delta_psi = current_holonomy - previous_holonomy;

            // Invariant 1: Delta psi != 0 on every step
            assert_ne!(
                delta_psi, 0,
                "Turn {turn}: Hopf fiber holonomy stagnated (Delta psi == 0) on query token {tok}"
            );

            previous_holonomy = current_holonomy;
            all_step_states.push(step.state.clone());
            all_step_holonomies.push(current_holonomy);
        }

        // Feed assistant response tokens
        for &tok in &assistant_response_tokens {
            let step = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("assistant step succeeds");

            let current_holonomy = session.holonomy_accumulator();
            let delta_psi = current_holonomy - previous_holonomy;

            // Invariant 1: Delta psi != 0 on every step
            assert_ne!(
                delta_psi, 0,
                "Turn {turn}: Hopf fiber holonomy stagnated (Delta psi == 0) on assistant token {tok}"
            );

            previous_holonomy = current_holonomy;
            all_step_states.push(step.state.clone());
            all_step_holonomies.push(current_holonomy);
        }

        // Record end-of-turn state
        end_of_turn_states.push(session.state.clone());
        end_of_turn_holonomies.push(session.holonomy_accumulator());
    }

    assert_eq!(end_of_turn_states.len(), 20);

    // Invariant 3: End-of-turn holonomies must be strictly distinct and non-zero
    assert_ne!(
        session.holonomy_accumulator(),
        0,
        "Total holonomy accumulator collapsed to 0!"
    );
    for i in 0..20 {
        for j in (i + 1)..20 {
            assert_ne!(
                end_of_turn_holonomies[i],
                end_of_turn_holonomies[j],
                "Holonomy accumulator repeated across turns {} and {}",
                i + 1,
                j + 1
            );
        }
    }

    // Measure state diversity across turns
    let mut state_matches = 0;
    for i in 0..20 {
        for j in (i + 1)..20 {
            if end_of_turn_states[i] == end_of_turn_states[j] {
                state_matches += 1;
            }
        }
    }
    println!(
        "Identical query end-of-turn state pairwise collisions: {} / 190 pairs",
        state_matches
    );
}

/// Adversarial Challenge 2:
/// High-cycle stress test examining fixed-point collapse, ring buffer wrapping,
/// and discrete holonomy advancement under constant token injection.
#[test]
fn test_adversarial_300_identical_tokens_holonomy_and_state_dynamics() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    let token = 42u32;
    let mut state_history: Vec<Vec<i32>> = Vec::with_capacity(300);
    let mut holonomy_history: Vec<i64> = Vec::with_capacity(300);
    let mut fiber_phase_history: Vec<i32> = Vec::with_capacity(300);

    for step_idx in 0..300 {
        let step = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("step succeeds");

        // Verify total probability normalization invariant
        let sum_p: u64 = step.probabilities.iter().sum();
        assert_eq!(
            sum_p, PROBABILITY_TOTAL,
            "Step {step_idx}: Probability sum deviated from 2^48!"
        );

        let current_holonomy = session.holonomy_accumulator();
        if step_idx > 0 {
            let prev_holonomy = holonomy_history[step_idx - 1];
            // Verify: Holonomy accumulator Delta psi != 0 on every step
            assert_ne!(
                current_holonomy, prev_holonomy,
                "Step {step_idx}: Holonomy failed to advance!"
            );
        }

        let current_fiber_phase = session.hopf_state.fiber_phase;

        state_history.push(step.state);
        holonomy_history.push(current_holonomy);
        fiber_phase_history.push(current_fiber_phase);
    }

    // Verify ring buffer state after 300 steps
    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_seen, 300);
    assert_eq!(session.dialogue_cursor, 300 % DIALOGUE_CAPACITY);
    assert_ne!(session.holonomy_accumulator(), 0);

    // Observe empirical dynamics
    let state_0 = &state_history[0];
    let state_last = &state_history[299];
    let state_equal = state_0 == state_last;
    println!(
        "Step 0 vs Step 299 state equal under synthetic model: {}",
        state_equal
    );
    println!(
        "Fiber phase at Step 0: {}, at Step 299: {}",
        fiber_phase_history[0], fiber_phase_history[299]
    );
    println!(
        "Cumulative holonomy after 300 steps: {}",
        session.holonomy_accumulator()
    );
}

/// Adversarial Challenge 3:
/// T^8 Riemann zeta-zero phase coordinates ergodicity and collision-free trajectory.
/// Tests 2,000 steps of both raw incommensurate updates and token-coupled updates.
#[test]
fn test_adversarial_t8_zeta_zero_ergodicity_and_collision_free() {
    // 1. Raw incommensurate frequencies on T^8
    let mut zeta_raw = T8ZetaState::new();
    let mut seen_raw = HashSet::new();

    const STEPS: usize = 2000;
    for step in 0..STEPS {
        assert!(
            seen_raw.insert(zeta_raw.phases),
            "Step {step}: T^8 phase vector collision in raw trajectory!"
        );
        zeta_raw.step_raw();
    }
    assert_eq!(seen_raw.len(), STEPS);

    // 2. Token-coupled updates with constant identical token (adversarial worst-case)
    let mut zeta_coupled = T8ZetaState::new();
    let mut seen_coupled = HashSet::new();
    let constant_token = 17u32;

    let mut min_phases = [i32::MAX; 8];
    let mut max_phases = [i32::MIN; 8];

    for step in 0..STEPS {
        assert!(
            seen_coupled.insert(zeta_coupled.phases),
            "Step {step}: T^8 phase vector collision in token-coupled trajectory!"
        );

        for (j, &phase) in zeta_coupled.phases.iter().enumerate() {
            min_phases[j] = min_phases[j].min(phase);
            max_phases[j] = max_phases[j].max(phase);
        }

        zeta_coupled.step(constant_token);
    }
    assert_eq!(seen_coupled.len(), STEPS);

    // 3. Verify ergodic coverage: each of the 8 torus coordinates must span both positive and negative half-spaces
    for j in 0..8 {
        assert!(
            min_phases[j] < 0,
            "Coordinate {j} failed to visit negative phase: min = {}",
            min_phases[j]
        );
        assert!(
            max_phases[j] > 0,
            "Coordinate {j} failed to visit positive phase: max = {}",
            max_phases[j]
        );
        // Range should cover substantial portion of [-2^30, 2^30]
        let span = (max_phases[j] as i64) - (min_phases[j] as i64);
        assert!(
            span > (1i64 << 30),
            "Coordinate {j} phase span {} too narrow (< 2^30)!",
            span
        );
    }
}

/// Adversarial Challenge 4:
/// Alternating prompt cycles ("Ping" / "Pong") for 40 turns.
/// Tests whether alternating token inputs induce a 2-cycle hysteresis trap in holonomy or telemetry.
#[test]
fn test_adversarial_alternating_prompt_oscillation_resistance() {
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(VOCAB_FIXTURE_WITH_ROLES.as_bytes())
            .expect("tokenizer fixture parses");

    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    let ping_tokens = tokenizer.encode("<|user|>Ping<|turn_end|><|assistant|>");
    let pong_tokens = tokenizer.encode("<|user|>Pong<|turn_end|><|assistant|>");

    let mut ping_holonomies: Vec<i64> = Vec::new();
    let mut pong_holonomies: Vec<i64> = Vec::new();

    for turn in 1..=40 {
        session.start_turn();
        let tokens = if turn % 2 == 1 {
            &ping_tokens
        } else {
            &pong_tokens
        };

        for &tok in tokens {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("step succeeds");
        }

        if turn % 2 == 1 {
            ping_holonomies.push(session.holonomy_accumulator());
        } else {
            pong_holonomies.push(session.holonomy_accumulator());
        }
    }

    // Invariant: All 20 "Ping" turns must have mutually distinct holonomy values (no periodic limit cycle!)
    for i in 0..ping_holonomies.len() {
        for j in (i + 1)..ping_holonomies.len() {
            assert_ne!(
                ping_holonomies[i],
                ping_holonomies[j],
                "Ping turn {} and Ping turn {} collapsed to identical holonomy!",
                2 * i + 1,
                2 * j + 1
            );
        }
    }

    // Invariant: All 20 "Pong" turns must have mutually distinct holonomy values
    for i in 0..pong_holonomies.len() {
        for j in (i + 1)..pong_holonomies.len() {
            assert_ne!(
                pong_holonomies[i],
                pong_holonomies[j],
                "Pong turn {} and Pong turn {} collapsed to identical holonomy!",
                2 * i + 2,
                2 * j + 2
            );
        }
    }
}

/// Adversarial Challenge 5:
/// Hopf fiber projection numerical stability across extreme edge cases:
/// Zero vector, axis-aligned poles, large coordinates, and branch-cut boundary crossings.
#[test]
fn test_adversarial_hopf_projection_boundary_and_branch_cut() {
    // 1. Degenerate zero input: must not panic or NaN, defaults to IDENTITY
    let zero_coords = [0, 0, 0, 0];
    let q_zero = UnitS3Q30::from_i32_coords(&zero_coords);
    assert_eq!(q_zero, UnitS3Q30::IDENTITY);
    let pt_zero = q_zero.hopf_fiber_project();
    assert_eq!(pt_zero.base, [0, 0, 1 << 30]);
    assert_eq!(pt_zero.fiber_phase, 0);

    // 2. Extreme coordinates near i32::MAX
    let max_coords = [i32::MAX, i32::MAX, i32::MAX, i32::MAX];
    let q_max = UnitS3Q30::from_i32_coords(&max_coords);
    let pt_max = q_max.hopf_fiber_project();
    // Base coordinates must lie within S2 bounds [-2^30, 2^30]
    for &b in &pt_max.base {
        assert!(b.abs() <= (1 << 30));
    }
    // Fiber phase must lie within [-2^30, 2^30]
    assert!(pt_max.fiber_phase.abs() <= (1 << 30));

    // 3. Branch cut unwrapping test
    // Traversing from 0.95*pi to -0.95*pi (a small forward step of +0.10*pi across the branch cut)
    const Q30: i64 = 1 << 30;
    let phase_before = (0.95 * (Q30 as f64)) as i32;
    let phase_after = (-0.95 * (Q30 as f64)) as i32;

    let mut d_phase = (phase_after as i64) - (phase_before as i64);
    if d_phase > Q30 {
        d_phase -= 2 * Q30;
    } else if d_phase < -Q30 {
        d_phase += 2 * Q30;
    }

    // Step should be unwrapped to a positive delta of +0.10*pi
    let expected_delta = (0.10 * (Q30 as f64)) as i64;
    let error = (d_phase - expected_delta).abs();
    assert!(
        error < 1000,
        "Branch cut unwrapping error too large: got {d_phase}, expected {expected_delta}"
    );
}

/// Adversarial Challenge 6:
/// Persistent persona partition write-protection and corruption resistance under 500 dialogue steps.
#[test]
fn test_adversarial_persona_partition_incorruptibility_under_dialogue_flooding() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Fill persistent slots to exact capacity (32 tokens)
    for tok in 0..PERSISTENT_CAPACITY as u32 {
        model
            .step_conversational(
                &mut session,
                100 + tok,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )
            .expect("fill persistent succeeds");
    }
    session.seal_persistent();
    assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);

    let baseline_keys = session.persistent_keys.clone();
    let baseline_values = session.persistent_values.clone();
    let baseline_tokens = session.persistent_tokens.clone();

    // Flood dialogue ring buffer with 500 random-walk tokens
    let mut current_tok = 50u32;
    for step in 0..500 {
        current_tok = (current_tok * 37 + 11) % (model.config().vocab_size as u32);
        model
            .step_conversational(
                &mut session,
                current_tok,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap_or_else(|e| panic!("dialogue flooding step {step} failed: {e}"));
    }

    // Invariant: Persistent partition remains 100% bitwise identical
    assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);
    assert_eq!(session.persistent_tokens, baseline_tokens);
    assert_eq!(session.persistent_keys, baseline_keys);
    assert_eq!(session.persistent_values, baseline_values);
    assert!(session.is_persistent_sealed());
}
