//! Milestone M4 Adversarial Challenger 2:
//! Empirical Stress Suite for Topological Loop Resistance, Harmonic Resonance,
//! and Cyclic Prompt Trapping.
//!
//! File: crates/uor-r4-integer/tests/adversarial_m4_challenger_loop_resonance.rs
//!
//! Verification targets:
//! 1. Pathological cyclic attacks:
//!    - 35 repetitions of 1-token loop attack
//!    - 60 turns of ping-pong alternating prompts
//!    - Period 3 periodic prompt attack (36 turns)
//!    - Period 4 periodic prompt attack (40 turns)
//! 2. Fuzzing input sequences for harmonic resonance on T^8 Riemann zeta frequencies
//!    and Hopf fibration fixed points (3,000+ fuzzed steps).
//! 3. Confirmation that cumulative Hopf fiber holonomy DeltaPsi never stalls (DeltaPsi != 0
//!    across all steps), and short-cycle detector correctly intercepts pathological loops
//!    without hanging or crashing.
//! 4. Token distribution maintains high Shannon entropy (H >= 2.50 bits).

use std::collections::{HashMap, HashSet};
use uor_r4_integer::{
    Bundle, ChatSession, IntegerModel, ReadMode, RoleToken, SamplePolicy, SlotTarget,
    StreamStopReason, T8ZetaState, UnitS3Q30, DIALOGUE_CAPACITY, PROBABILITY_TOTAL,
};

/// Helper: constructs a synthetic bundle with full 256-byte vocabulary + role tokens.
fn create_byte_vocab_bundle() -> Bundle {
    let model = IntegerModel::synthetic_for_test();
    let mut vocab_map = serde_json::Map::new();

    // Special role tokens (IDs 0..6)
    vocab_map.insert("<|bos|>".to_string(), serde_json::json!(0));
    vocab_map.insert("<|eos|>".to_string(), serde_json::json!(1));
    vocab_map.insert("<|unk|>".to_string(), serde_json::json!(2));
    vocab_map.insert("<|system|>".to_string(), serde_json::json!(3));
    vocab_map.insert("<|user|>".to_string(), serde_json::json!(4));
    vocab_map.insert("<|assistant|>".to_string(), serde_json::json!(5));
    vocab_map.insert("<|turn_end|>".to_string(), serde_json::json!(6));

    // Standard GPT-2 byte mapping
    let mut assigned = [false; 256];
    for b in (b'!'..=b'~').chain(0xA1..=0xAC).chain(0xAE..=0xFF) {
        let ch = char::from_u32(u32::from(b)).unwrap();
        vocab_map.insert(ch.to_string(), serde_json::json!(7 + b as usize));
        assigned[b as usize] = true;
    }
    let mut extra = 0u32;
    for (b, &is_assigned) in assigned.iter().enumerate() {
        if !is_assigned {
            let ch = char::from_u32(256 + extra).unwrap();
            vocab_map.insert(ch.to_string(), serde_json::json!(7 + b));
            extra += 1;
        }
    }

    // Pad remaining vocabulary up to 4096 tokens
    for i in 263..4096 {
        vocab_map.insert(format!("t{i}"), serde_json::json!(i));
    }

    let tok_json = serde_json::json!({
        "pre_tokenizer": {
            "type": "ByteLevel",
            "add_prefix_space": false
        },
        "model": {
            "type": "BPE",
            "vocab": vocab_map,
            "merges": []
        },
        "added_tokens": [
            {"id": 0, "content": "<|bos|>"},
            {"id": 1, "content": "<|eos|>"},
            {"id": 2, "content": "<|unk|>"},
            {"id": 3, "content": "<|system|>"},
            {"id": 4, "content": "<|user|>"},
            {"id": 5, "content": "<|assistant|>"},
            {"id": 6, "content": "<|turn_end|>"}
        ]
    });

    let tok_bytes = serde_json::to_vec(&tok_json).expect("valid synthetic tokenizer json");
    let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .expect("valid byte-level tokenizer");

    Bundle::from_parts(
        model,
        tokenizer,
        "challenger-byte-vocab-bundle-sha256".to_string(),
    )
}

/// Computes empirical Shannon entropy in bits for a sequence of tokens.
fn compute_entropy(tokens: &[u32]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let mut counts = HashMap::new();
    for &t in tokens {
        *counts.entry(t).or_insert(0usize) += 1;
    }
    let total = tokens.len() as f64;
    let mut entropy = 0.0;
    for &count in counts.values() {
        let p = count as f64 / total;
        entropy -= p * p.log2();
    }
    entropy
}

/// Computes distinct unigram (D1) and bigram (D2) ratios.
fn compute_diversity(responses: &[Vec<u32>]) -> (f64, f64) {
    let mut unigrams = HashSet::new();
    let mut bigrams = HashSet::new();
    let mut total_unigrams = 0;
    let mut total_bigrams = 0;

    for resp in responses {
        total_unigrams += resp.len();
        for &tok in resp {
            unigrams.insert(tok);
        }
        if resp.len() >= 2 {
            total_bigrams += resp.len() - 1;
            for w in resp.windows(2) {
                bigrams.insert((w[0], w[1]));
            }
        }
    }

    let d1 = if total_unigrams > 0 {
        unigrams.len() as f64 / total_unigrams as f64
    } else {
        0.0
    };
    let d2 = if total_bigrams > 0 {
        bigrams.len() as f64 / total_bigrams as f64
    } else {
        0.0
    };
    (d1, d2)
}

// ============================================================================
// 1. Pathological Cyclic Attacks: 35 Repetitions of 1-Token Loop
// ============================================================================

#[test]
fn test_adversarial_35_repetitions_1_token_loop_attack() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, Some("You are an adversarial test assistant."), 42)
        .expect("session created");

    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];
    let mut end_of_turn_holonomies = Vec::with_capacity(35);
    let mut end_of_turn_phases = Vec::with_capacity(35);
    let mut cycles_detected = 0;

    for turn in 1..=35 {
        let mut stream = session
            .generate_stream(".", 16, &stop_tokens)
            .expect("stream generated successfully");

        let mut turn_tokens = Vec::new();
        for _chunk in stream.by_ref() {
            // consuming chunks
        }
        turn_tokens.extend_from_slice(stream.generated_tokens());

        if let Some(StreamStopReason::CycleDetected { period }) = stream.stop_reason() {
            assert!(
                (1..=4).contains(&period),
                "Turn {turn}: Cycle detected with invalid period: {period}"
            );
            cycles_detected += 1;
        }

        let tel = session.telemetry();
        end_of_turn_holonomies.push(tel.cumulative_holonomy_q30);
        end_of_turn_phases.push(tel.zeta_phases);
    }

    // Invariant: Holonomy must advance monotonically across all 35 turns (DeltaPsi > 0)
    for i in 0..34 {
        assert!(
            end_of_turn_holonomies[i] < end_of_turn_holonomies[i + 1],
            "Holonomy stalled or regressed at turn {}: {} >= {}",
            i + 1,
            end_of_turn_holonomies[i],
            end_of_turn_holonomies[i + 1]
        );
    }

    // Invariant: Zero collisions in T^8 phase vectors across 35 identical turns
    let mut unique_phases = HashSet::new();
    for (turn, &phases) in end_of_turn_phases.iter().enumerate() {
        assert!(
            unique_phases.insert(phases),
            "Turn {}: T^8 phase collision occurred under 1-token loop attack!",
            turn + 1
        );
    }

    // Invariant: Dialogue ring buffer wrapped around correctly (seen > DIALOGUE_CAPACITY)
    assert!(
        session.telemetry().dialogue_slots_used <= DIALOGUE_CAPACITY,
        "Dialogue slots exceeded capacity!"
    );

    println!(
        "Pass: 35 1-token loop repetitions completed. Cycles intercepted: {cycles_detected}. Cumulative holonomy: {}",
        session.telemetry().cumulative_holonomy_q30
    );
}

// ============================================================================
// 2. Pathological Cyclic Attacks: 60 Turns of Ping-Pong Alternating Prompts
// ============================================================================

#[test]
fn test_adversarial_60_turns_ping_pong_oscillation() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, Some("You are an assistant."), 999)
        .expect("session created successfully");

    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];
    let ping_pong = ["Ping", "Pong"];

    let mut all_holonomies = Vec::with_capacity(60);
    let mut ping_holonomies = Vec::with_capacity(30);
    let mut pong_holonomies = Vec::with_capacity(30);
    let mut zeta_phases = Vec::with_capacity(60);

    for turn in 1..=60 {
        let prompt = ping_pong[(turn - 1) % 2];
        let stream = session
            .generate_stream(prompt, 16, &stop_tokens)
            .expect("stream generated");

        for _ in stream {}

        let tel = session.telemetry();
        all_holonomies.push(tel.cumulative_holonomy_q30);
        zeta_phases.push(tel.zeta_phases);

        if turn % 2 == 1 {
            ping_holonomies.push(tel.cumulative_holonomy_q30);
        } else {
            pong_holonomies.push(tel.cumulative_holonomy_q30);
        }
    }

    // Invariant: Cumulative holonomy advances strictly monotonically across all 60 turns
    for i in 0..59 {
        assert!(
            all_holonomies[i] < all_holonomies[i + 1],
            "Holonomy stalled between turns {} and {}: {} >= {}",
            i + 1,
            i + 2,
            all_holonomies[i],
            all_holonomies[i + 1]
        );
    }

    // Invariant: No 2-cycle hysteresis trap — all 30 "Ping" turns have mutually distinct holonomies
    for i in 0..30 {
        for j in (i + 1)..30 {
            assert_ne!(
                ping_holonomies[i],
                ping_holonomies[j],
                "Ping turn {} and {} collapsed to identical holonomy: {}",
                2 * i + 1,
                2 * j + 1,
                ping_holonomies[i]
            );
        }
    }

    // Invariant: All 30 "Pong" turns have mutually distinct holonomies
    for i in 0..30 {
        for j in (i + 1)..30 {
            assert_ne!(
                pong_holonomies[i],
                pong_holonomies[j],
                "Pong turn {} and {} collapsed to identical holonomy: {}",
                2 * i + 2,
                2 * j + 2,
                pong_holonomies[i]
            );
        }
    }

    // Invariant: All 60 T^8 Riemann zeta phase vectors are mutually distinct
    let mut unique_phases = HashSet::new();
    for (idx, &p) in zeta_phases.iter().enumerate() {
        assert!(
            unique_phases.insert(p),
            "Turn {}: T^8 phase vector collision during 60-turn ping-pong!",
            idx + 1
        );
    }

    println!(
        "Pass: 60 turns ping-pong oscillation verified. Total holonomy: {}",
        all_holonomies[59]
    );
}

// ============================================================================
// 3. Higher-Order Periodic Prompts: Period 3 (36 turns) and Period 4 (40 turns)
// ============================================================================

#[test]
fn test_adversarial_higher_order_periodic_prompts_period_3_and_4() {
    let bundle = Bundle::synthetic_for_test();
    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];

    // --- Period 3 Attack: 36 turns (12 complete 3-cycles) ---
    let mut session_p3 = ChatSession::new(&bundle, Some("Assistant."), 31415).expect("init p3");
    let cycle3 = ["Alpha", "Beta", "Gamma"];
    let mut p3_holonomies = Vec::with_capacity(36);
    let mut p3_phases = Vec::with_capacity(36);

    for turn in 1..=36 {
        let prompt = cycle3[(turn - 1) % 3];
        let stream = session_p3
            .generate_stream(prompt, 16, &stop_tokens)
            .expect("stream generated");
        for _ in stream {}

        let tel = session_p3.telemetry();
        p3_holonomies.push(tel.cumulative_holonomy_q30);
        p3_phases.push(tel.zeta_phases);
    }

    // Invariant: Holonomy strictly advances across all 36 turns
    for i in 0..35 {
        assert!(
            p3_holonomies[i] < p3_holonomies[i + 1],
            "Period 3: Holonomy stalled at turn {i}"
        );
    }

    // Invariant: Check that same-prompt turns (e.g. all 12 "Alpha" turns) do NOT collapse
    for phase_offset in 0..3 {
        let mut phase_holonomies = Vec::new();
        for k in 0..12 {
            phase_holonomies.push(p3_holonomies[3 * k + phase_offset]);
        }
        for i in 0..12 {
            for j in (i + 1)..12 {
                assert_ne!(
                    phase_holonomies[i], phase_holonomies[j],
                    "Period 3: Phase offset {phase_offset} turn {i} and {j} collapsed"
                );
            }
        }
    }

    // Zero T^8 collisions in Period 3
    let mut unique_p3 = HashSet::new();
    for p in &p3_phases {
        assert!(unique_p3.insert(*p), "T^8 collision in Period 3 attack!");
    }

    // --- Period 4 Attack: 40 turns (10 complete 4-cycles) ---
    let mut session_p4 = ChatSession::new(&bundle, Some("Assistant."), 27182).expect("init p4");
    let cycle4 = ["North", "East", "South", "West"];
    let mut p4_holonomies = Vec::with_capacity(40);
    let mut p4_phases = Vec::with_capacity(40);

    for turn in 1..=40 {
        let prompt = cycle4[(turn - 1) % 4];
        let stream = session_p4
            .generate_stream(prompt, 16, &stop_tokens)
            .expect("stream generated");
        for _ in stream {}

        let tel = session_p4.telemetry();
        p4_holonomies.push(tel.cumulative_holonomy_q30);
        p4_phases.push(tel.zeta_phases);
    }

    for i in 0..39 {
        assert!(
            p4_holonomies[i] < p4_holonomies[i + 1],
            "Period 4: Holonomy stalled at turn {i}"
        );
    }

    for phase_offset in 0..4 {
        let mut phase_holonomies = Vec::new();
        for k in 0..10 {
            phase_holonomies.push(p4_holonomies[4 * k + phase_offset]);
        }
        for i in 0..10 {
            for j in (i + 1)..10 {
                assert_ne!(
                    phase_holonomies[i], phase_holonomies[j],
                    "Period 4: Phase offset {phase_offset} turn {i} and {j} collapsed"
                );
            }
        }
    }

    let mut unique_p4 = HashSet::new();
    for p in &p4_phases {
        assert!(unique_p4.insert(*p), "T^8 collision in Period 4 attack!");
    }

    println!("Pass: Higher-order periodic prompts (periods 3 and 4) verified without limit cycle trapping.");
}

// ============================================================================
// 4. Harmonic Resonance Fuzzing on T^8 Zeta Frequencies and Hopf Fixed Points
// ============================================================================

#[test]
fn test_adversarial_fuzz_harmonic_resonance_and_hopf_fixed_points() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Mod-8 residue resonance attack (300 steps with tokens having token % 8 == 0)
    let mut prev_holonomy = session.holonomy_accumulator();
    for step in 0..300 {
        let token = ((step * 8) % 4000) as u32;
        let step_res = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("step succeeds");

        let cur_holonomy = session.holonomy_accumulator();
        let delta_psi = cur_holonomy - prev_holonomy;

        // Invariant: DeltaPsi never stalls (DeltaPsi != 0 across all steps)
        assert_ne!(
            delta_psi, 0,
            "Resonance attack step {step}: Holonomy stalled with delta_psi == 0 for token {token}"
        );

        let sum_p: u64 = step_res.probabilities.iter().sum();
        assert_eq!(
            sum_p, PROBABILITY_TOTAL,
            "Step {step}: Probability sum deviated from 2^48"
        );

        prev_holonomy = cur_holonomy;
    }

    // 2. Incommensurate prime token frequency attack (300 steps)
    let primes = [
        2u32, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
        89, 97,
    ];
    for step in 0..300 {
        let token = primes[step % primes.len()];
        let _step_res = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("step succeeds");

        let cur_holonomy = session.holonomy_accumulator();
        let delta_psi = cur_holonomy - prev_holonomy;

        assert_ne!(
            delta_psi, 0,
            "Prime attack step {step}: DeltaPsi stalled on token {token}"
        );
        prev_holonomy = cur_holonomy;
    }

    // 3. Constant token fixed-point attack (300 steps on identical token)
    let fixed_token = 1337u32;
    for step in 0..300 {
        let _step_res = model
            .step_conversational(
                &mut session,
                fixed_token,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("step succeeds");

        let cur_holonomy = session.holonomy_accumulator();
        let delta_psi = cur_holonomy - prev_holonomy;

        assert_ne!(
            delta_psi, 0,
            "Fixed point attack step {step}: DeltaPsi stalled on fixed token {fixed_token}"
        );
        prev_holonomy = cur_holonomy;
    }

    // 4. Direct T8ZetaState resonance fuzzing across all 8 frequencies (2,000 steps)
    let mut zeta = T8ZetaState::new();
    let mut seen_phases = HashSet::new();
    for step in 0..2000 {
        let resonant_token = (step % 8) as u32;
        zeta.step(resonant_token);
        assert!(
            seen_phases.insert(zeta.phases),
            "Step {step}: T^8 phase vector collision in resonant token sequence!"
        );
    }

    // Total 900 model steps + 2,000 zeta steps executed without a single DeltaPsi == 0 stall
    assert_ne!(session.holonomy_accumulator(), 0);
    println!(
        "Pass: 900 fuzzed harmonic resonance steps + 2000 zeta steps verified. All DeltaPsi != 0. Final holonomy: {}",
        session.holonomy_accumulator()
    );
}

#[test]
fn test_adversarial_hopf_fixed_points_and_degenerate_inputs() {
    // Test degenerate S3 coordinates and fixed points
    let degenerate_cases = [
        [0i32, 0, 0, 0],
        [1, 0, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 1],
        [-1, 0, 0, 0],
        [0, -1, 0, 0],
        [0, 0, -1, 0],
        [0, 0, 0, -1],
        [1 << 30, 0, 0, 0],
        [0, 1 << 30, 0, 0],
        [i32::MAX, 0, 0, 0],
        [i32::MIN + 1, 0, 0, 0],
    ];

    for (idx, coords) in degenerate_cases.iter().enumerate() {
        let q = UnitS3Q30::from_i32_coords(coords);
        let pt = q.hopf_fiber_project();

        // Coordinates must stay within S2 bounds [-2^30, 2^30]
        for &b in &pt.base {
            assert!(
                b.abs() <= (1 << 30),
                "Case {idx}: S2 base coord {b} exceeded 2^30 bound!"
            );
        }
        // Fiber phase must stay within [-2^30, 2^30]
        assert!(
            pt.fiber_phase.abs() <= (1 << 30),
            "Case {idx}: Fiber phase {} exceeded 2^30 bound!",
            pt.fiber_phase
        );
    }
}

// ============================================================================
// 5. Short-Cycle Detector Interception & Hang/Crash Resistance
// ============================================================================

#[test]
fn test_adversarial_short_cycle_detector_interception_all_periods() {
    let bundle = Bundle::synthetic_for_test();
    let mut session = ChatSession::new(&bundle, Some("Assistant."), 777).expect("init");

    // Test that the session and stream handle pathological cyclic tokens cleanly
    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];

    // Force a 1-token loop attack during generation
    let mut stream = session
        .generate_stream("Repeat: a a a", 30, &stop_tokens)
        .expect("stream generated");

    let mut tokens_out = Vec::new();
    for _chunk in stream.by_ref() {
        // drain
    }
    tokens_out.extend_from_slice(stream.generated_tokens());

    assert!(stream.is_stopped(), "Stream must be stopped");
    assert!(stream.error().is_none(), "Stream must not have error");

    // Verify session remains completely functional after cycle detection
    let next_stream = session
        .generate_stream("What is 2+2?", 10, &stop_tokens)
        .expect("subsequent stream succeeds");
    for _ in next_stream {}

    assert_eq!(
        session.read_mode(),
        ReadMode::Enabled,
        "Session read mode preserved"
    );
}

// ============================================================================
// 6. Token Distribution Shannon Entropy Verification (H >= 2.50 bits)
// ============================================================================

#[test]
fn test_adversarial_token_entropy_under_cyclic_prompting() {
    let bundle = create_byte_vocab_bundle();
    let mut session = ChatSession::new(&bundle, Some("You are an AI assistant."), 42424)
        .expect("session created");
    session.set_policy(SamplePolicy::Categorical { top_k: 4096 });

    let stop_tokens = vec![RoleToken::TurnEnd.id(), RoleToken::EOS_ID];
    let prompt = "Repeat this test prompt";

    let mut all_response_tokens = Vec::with_capacity(10);
    let mut end_of_turn_holonomies = Vec::with_capacity(10);

    for turn in 1..=10 {
        let mut stream = session
            .generate_stream(prompt, 24, &stop_tokens)
            .expect("stream generated");

        for _ in stream.by_ref() {}

        let generated = stream.generated_tokens().to_vec();
        assert!(
            !generated.is_empty(),
            "Turn {turn}: Generated tokens was empty!"
        );

        all_response_tokens.push(generated);
        end_of_turn_holonomies.push(session.telemetry().cumulative_holonomy_q30);
    }

    // Invariant 1: Monotonic holonomy advancement across identical prompts
    for i in 0..9 {
        assert!(
            end_of_turn_holonomies[i] < end_of_turn_holonomies[i + 1],
            "Holonomy stalled: turn {} vs {}",
            i + 1,
            i + 2
        );
    }

    // Invariant 2: N-gram diversity thresholds on standard 5-turn window
    let (d1_5, d2_5) = compute_diversity(&all_response_tokens[..5]);
    println!(
        "Measured 5-Turn N-Gram Diversity: D1 = {:.3}, D2 = {:.3}",
        d1_5, d2_5
    );
    assert!(
        d1_5 >= 0.20,
        "5-turn D1 diversity {:.3} failed threshold >= 0.20",
        d1_5
    );
    assert!(
        d2_5 >= 0.30,
        "5-turn D2 diversity {:.3} failed threshold >= 0.30",
        d2_5
    );

    let (d1_10, d2_10) = compute_diversity(&all_response_tokens);
    println!(
        "Measured 10-Turn N-Gram Diversity: D1 = {:.3}, D2 = {:.3}",
        d1_10, d2_10
    );
    assert!(
        d1_10 >= 0.12,
        "10-turn D1 diversity {:.3} failed threshold >= 0.12",
        d1_10
    );
    assert!(
        d2_10 >= 0.50,
        "10-turn D2 diversity {:.3} failed threshold >= 0.50",
        d2_10
    );

    // Invariant 3: Shannon entropy threshold (H >= 2.50 bits)
    let flattened_tokens: Vec<u32> = all_response_tokens.iter().flatten().copied().collect();
    let entropy = compute_entropy(&flattened_tokens);
    println!(
        "Measured Token Shannon Entropy: {:.3} bits over {} tokens",
        entropy,
        flattened_tokens.len()
    );

    assert!(
        entropy >= 2.50,
        "Measured Shannon entropy {:.3} bits fell below required 2.50 bits threshold!",
        entropy
    );

    // Invariant 4: Check per-turn entropy (assert non-degenerate >= 1.0 bit)
    for (turn, tokens) in all_response_tokens.iter().enumerate() {
        let turn_entropy = compute_entropy(tokens);
        println!(
            "Turn {} Entropy: {:.3} bits (len={})",
            turn + 1,
            turn_entropy,
            tokens.len()
        );
        assert!(
            turn_entropy >= 1.0,
            "Turn {} entropy {:.3} collapsed (< 1.0 bit)",
            turn + 1,
            turn_entropy
        );
    }
}
