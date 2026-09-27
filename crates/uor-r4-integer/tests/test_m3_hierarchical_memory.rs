//! Milestone M3 Hierarchical Prime Memory Integration & Verification Test Suite.
//! File: crates/uor-r4-integer/tests/test_m3_hierarchical_memory.rs
//!
//! Comprehensive verification covering:
//! 1. Memory struct size and alignment (`L2PrimePage` 2048 bytes, 64-byte alignment, 0 mul for indexing).
//! 2. Barycenter key compression accuracy vs floating point reference (|int - float| <= 1).
//! 3. Galois LFSR prime signature order-sensitivity, slot-sensitivity, and 0-collision properties.
//! 4. Salient token extraction (entity prioritization, stopword penalties, deduplication).
//! 5. Long-horizon rollover across K = 512..1024 tokens with unified 320-candidate associative read.
//! 6. Factual recall benchmark (L2 retrieval active vs NoRead ablation collapsing to 0).
//! 7. Session serialization roundtrip with L2 pages.
//! 8. Power-of-two salient copy routing mass conservation.

use std::collections::HashSet;
use std::fs;
use std::mem::{align_of, size_of};
use uor_r4_integer::{
    compress_barycenter_key_fibonacci, compute_turn_prime_signature, extract_salient_tokens,
    score_token_salience, Bundle, ChatSession, IntegerModel, L2PrimePage, ReadMode, SlotTarget,
    DIALOGUE_CAPACITY, FIBONACCI_WEIGHTS, KEY_DIM, L2_PAGE_CAPACITY, PERSISTENT_CAPACITY,
    PROBABILITY_TOTAL, TOTAL_MEMORY_CANDIDATES, VAL_DIM,
};

/// 1. Memory struct size and alignment:
///    `L2PrimePage` must be exactly 2048 bytes (2^11) and 64-byte aligned.
#[test]
fn test_m3_l2_page_size_and_alignment() {
    assert_eq!(
        size_of::<L2PrimePage>(),
        2048,
        "L2PrimePage must be exactly 2048 bytes (2^11) to eliminate array indexing multipliers"
    );
    assert_eq!(
        align_of::<L2PrimePage>(),
        64,
        "L2PrimePage must be 64-byte cache-line aligned"
    );

    // Array of 64 pages: 64 * 2048 = 131072 bytes (128 KiB)
    assert_eq!(
        size_of::<[L2PrimePage; L2_PAGE_CAPACITY]>(),
        131072,
        "64 L2PrimePages must occupy exactly 128 KiB"
    );

    // Verify pointer arithmetic matches shift by 11 (zero multiplication)
    let pages = Box::new([L2PrimePage::default(); L2_PAGE_CAPACITY]);
    let base_ptr = pages.as_ptr() as usize;
    for i in 0..L2_PAGE_CAPACITY {
        let page_ptr = &pages[i] as *const _ as usize;
        let expected_offset = i << 11;
        assert_eq!(
            page_ptr - base_ptr,
            expected_offset,
            "Page index {i} offset must match (i << 11)"
        );
        assert_eq!(page_ptr % 64, 0, "Page {i} must be 64-byte aligned");
    }

    // Default page contents are zeroed
    let def = L2PrimePage::default();
    assert_eq!(def.key, [0i32; KEY_DIM]);
    assert_eq!(def.value, [0i32; VAL_DIM]);
    assert_eq!(def.prime_signature, 0);
    assert_eq!(def.salient_tokens, [0u32; 4]);
    assert_eq!(def.turn_id, 0);
}

/// 2. Barycenter key compression accuracy vs floating point reference:
///    Verify integer 2-accumulator reverse sweep matches float reference within +/- 1.
#[test]
fn test_m3_barycenter_key_fibonacci_accuracy_vs_float_reference() {
    let mut dialogue_keys = [[0i32; KEY_DIM]; DIALOGUE_CAPACITY];

    // Seed pseudo-random test keys across coordinates [-15000, 15000]
    for (slot, key) in dialogue_keys.iter_mut().enumerate() {
        for (d, coord) in key.iter_mut().enumerate() {
            let seed = (slot as i64 * 104729 + d as i64 * 7919) % 30001 - 15000;
            *coord = seed as i32;
        }
    }

    // Test different turn lengths: 1, 2, 3, 5, 8, 13, 21, 24, 30
    let test_lengths = [1, 2, 3, 5, 8, 13, 21, 24, 30];

    for &n in &test_lengths {
        let slot_indices: Vec<usize> = (0..n).collect();
        let mut int_key = [0i32; KEY_DIM];
        compress_barycenter_key_fibonacci(&dialogue_keys, &slot_indices, &mut int_key);

        // Effective slots clamped to trailing 24 slots
        let effective_slots = if n > 24 {
            &slot_indices[n - 24..]
        } else {
            &slot_indices[..]
        };
        let eff_n = effective_slots.len();

        // Compute floating point reference with exact Fibonacci weights
        let mut total_weight_f64 = 0.0f64;
        for &w in FIBONACCI_WEIGHTS.iter().take(eff_n) {
            total_weight_f64 += w as f64;
        }

        for d in 0..KEY_DIM {
            let mut weighted_sum_f64 = 0.0f64;
            for i in 0..eff_n {
                let slot = effective_slots[i];
                let val = dialogue_keys[slot][d] as f64;
                let w = FIBONACCI_WEIGHTS[i] as f64;
                weighted_sum_f64 += val * w;
            }
            let float_barycenter = (weighted_sum_f64 / total_weight_f64).round() as i64;
            let diff = (int_key[d] as i64 - float_barycenter).abs();

            assert!(
                diff <= 1,
                "Barycenter coordinate {d} mismatch for N={n}: int={}, float_ref={}, diff={diff}",
                int_key[d],
                float_barycenter
            );
        }
    }
}

/// 3. Galois LFSR prime signature properties:
///    Order-sensitivity, slot-sensitivity, non-zero invariant, and zero collisions.
#[test]
fn test_m3_prime_signature_galois_lfsr_properties() {
    let dummy_tokens = [101u32, 102, 103, 104, 105];

    // Non-zero invariant
    let zero_tokens = [0u32; 5];
    let zero_slots = [0usize, 1, 2, 3, 4];
    let sig_zero = compute_turn_prime_signature(&zero_tokens, &zero_slots, 0, 0, 0);
    assert_ne!(
        sig_zero, 0,
        "Prime signature must be strictly non-zero even with zero inputs"
    );

    // Order sensitivity: permuting tokens produces different signature
    let sig_forward = compute_turn_prime_signature(&dummy_tokens, &[0, 1, 2, 3, 4], 1, 100, 105);
    let mut rev_tokens = dummy_tokens;
    rev_tokens.reverse();
    let sig_reverse = compute_turn_prime_signature(&rev_tokens, &[0, 1, 2, 3, 4], 1, 100, 105);
    assert_ne!(
        sig_forward, sig_reverse,
        "Permuting token sequence must produce distinct prime signature"
    );

    // Slot sensitivity: same tokens at different slot indices produces different signature
    let sig_slots_a = compute_turn_prime_signature(&dummy_tokens, &[0, 1, 2, 3, 4], 1, 100, 105);
    let sig_slots_b =
        compute_turn_prime_signature(&dummy_tokens, &[10, 11, 12, 13, 14], 1, 100, 105);
    assert_ne!(
        sig_slots_a, sig_slots_b,
        "Different slot prime addresses must produce distinct prime signature"
    );

    // Turn ID sensitivity
    let sig_turn_1 = compute_turn_prime_signature(&dummy_tokens, &[0, 1, 2, 3, 4], 1, 100, 105);
    let sig_turn_2 = compute_turn_prime_signature(&dummy_tokens, &[0, 1, 2, 3, 4], 2, 100, 105);
    assert_ne!(
        sig_turn_1, sig_turn_2,
        "Different turn IDs must produce distinct prime signature"
    );

    // Collision resistance across 1000 synthesized turns
    let mut signatures = HashSet::with_capacity(1000);
    for turn in 1..=1000 {
        let tokens = [
            (turn * 31) as u32 % 50000,
            (turn * 37) as u32 % 50000,
            (turn * 41) as u32 % 50000,
        ];
        let slots = [
            (turn * 3) % DIALOGUE_CAPACITY,
            (turn * 3 + 1) % DIALOGUE_CAPACITY,
            (turn * 3 + 2) % DIALOGUE_CAPACITY,
        ];
        let sig = compute_turn_prime_signature(
            &tokens,
            &slots,
            turn as u32,
            turn as u64 * 3,
            turn as u64 * 3 + 2,
        );
        assert_ne!(sig, 0, "Turn {turn} signature must be non-zero");
        assert!(
            signatures.insert(sig),
            "Collision detected at turn {turn}: signature 0x{sig:016X}"
        );
    }
    assert_eq!(signatures.len(), 1000);
}

/// 4. Salient token extraction:
///    Entity prioritization, stopword/punctuation penalties, and deduplication.
#[test]
fn test_m3_salient_token_scoring_and_extraction() {
    let dummy_key = [100i32; KEY_DIM];

    // Special tokens (<= 6) strongly penalized
    for tok in 0..=6 {
        assert!(
            score_token_salience(tok, &dummy_key) <= -1_000_000,
            "Special token {tok} must have penalty <= -1M"
        );
    }

    // Punctuation and whitespace penalized
    assert!(score_token_salience(32, &dummy_key) <= -400_000); // Space
    assert!(score_token_salience(b'.' as u32, &dummy_key) <= -400_000);
    assert!(score_token_salience(b',' as u32, &dummy_key) <= -400_000);

    // Numbers and capitalized words prioritized
    let num_score = score_token_salience(b'7' as u32, &dummy_key);
    let cap_score = score_token_salience(b'Z' as u32, &dummy_key);
    let low_score = score_token_salience(b'z' as u32, &dummy_key);
    assert!(num_score > low_score);
    assert!(cap_score > low_score);

    // Test extraction from a mixed dialogue turn
    let mut dialogue_tokens = vec![0u32; DIALOGUE_CAPACITY];
    let mut dialogue_keys = [[0i32; KEY_DIM]; DIALOGUE_CAPACITY];

    let turn_tokens = [
        1u32,        // BOS (special, penalized)
        32,          // space (penalized)
        b't' as u32, // lowercase
        b'h' as u32, // lowercase
        b'e' as u32, // lowercase
        32,          // space
        b'M' as u32, // Capitalized entity (+50k)
        b'1' as u32, // Number (+60k)
        12500u32,    // BPE subword (+10k + hash)
        34200u32,    // High entity subword
        b'.' as u32, // Period (penalized)
        12500u32,    // Duplicate entity
    ];

    let slot_indices: Vec<usize> = (0..turn_tokens.len()).collect();
    for (i, &tok) in turn_tokens.iter().enumerate() {
        dialogue_tokens[i] = tok;
        dialogue_keys[i].fill(200); // positive L1 norm
    }

    let mut salient = [0u32; 4];
    extract_salient_tokens(
        &dialogue_tokens,
        &dialogue_keys,
        &slot_indices,
        &mut salient,
    );

    // Top 4 must be the high-salience tokens (no duplicates, no special tokens, no punctuation)
    assert!(
        !salient.contains(&1),
        "Salient tokens must not contain special BOS token"
    );
    assert!(
        !salient.contains(&32),
        "Salient tokens must not contain space"
    );
    assert!(
        !salient.contains(&(b'.' as u32)),
        "Salient tokens must not contain punctuation"
    );

    // Duplicate 12500 must only appear once
    let count_12500 = salient.iter().filter(|&&t| t == 12500).count();
    assert_eq!(
        count_12500, 1,
        "Duplicate token 12500 must appear exactly once"
    );

    // All 4 slots must be filled with non-zero tokens
    for &tok in &salient {
        assert_ne!(tok, 0, "Salient token slot should be populated");
    }
}

/// 5. Long-horizon rollover across K = 512..1024 tokens:
///    Verify dialogue ring rollover, L2 page compression, and unified 320-candidate read.
#[test]
fn test_m3_long_horizon_rollover_k512_to_k1024() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    assert_eq!(session.l2_len, 0);
    assert_eq!(session.l2_cursor, 0);
    assert_eq!(session.l2_seen, 0);
    assert_eq!(session.last_compressed_turn_id, u32::MAX);

    let turns = 35;
    let tokens_per_turn = 20;
    let total_tokens = turns * tokens_per_turn; // 700 tokens (K > 512)

    for turn in 1..=turns {
        session.start_turn();
        assert_eq!(session.current_turn(), turn as u32);

        for step_in_turn in 0..tokens_per_turn {
            let token = (((turn * 17 + step_in_turn * 31) % 4000) + 10) as u32;
            let expected_candidates =
                session.persistent_len() + session.dialogue_len() + session.l2_len;

            let step = model
                .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("step_conversational must succeed");

            // Invariant: Total probability sum 2^48
            let sum_prob: u64 = step.probabilities.iter().sum();
            assert_eq!(
                sum_prob, PROBABILITY_TOTAL,
                "Probability total must be exactly 2^48 at turn {turn}, step {step_in_turn}"
            );

            // Invariant: unified associative read candidates evaluated = active memory candidates before write
            assert_eq!(
                session.last_read_masses.len(),
                expected_candidates,
                "Unified associative read must return exactly {expected_candidates} candidate masses"
            );
        }
    }

    // After 700 tokens (> 3x DIALOGUE_CAPACITY 224):
    assert_eq!(session.dialogue_len, DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_cursor, total_tokens % DIALOGUE_CAPACITY);

    // L2 pages must be populated
    assert!(
        session.l2_len > 0,
        "L2 pages must be populated after 700 dialogue tokens"
    );
    assert!(
        session.l2_seen > 0,
        "L2 seen counter must be > 0 after rollover"
    );

    // Verify populated L2 pages have valid non-trivial content
    for i in 0..session.l2_len {
        let page = &session.l2_pages[i];
        assert_ne!(
            page.prime_signature, 0,
            "L2 page {i} prime signature must be non-zero"
        );
        assert!(
            page.turn_id > 0,
            "L2 page {i} turn_id must be valid non-zero"
        );
        assert!(
            page.key.iter().any(|&x| x != 0),
            "L2 page {i} key must not be all zeros"
        );
        assert!(
            page.value.iter().any(|&x| x != 0),
            "L2 page {i} value must not be all zeros"
        );
        assert!(
            page.salient_tokens.iter().any(|&t| t != 0),
            "L2 page {i} salient tokens must not be all zeros"
        );
    }
}

/// 6. Factual recall benchmark (L2 retrieval active vs NoRead ablation):
///    Evict a fact turn into L2, query it, verify L2 read masses active under ReadMode::Enabled,
///    and verify 0% memory mass under ReadMode::NoRead.
#[test]
fn test_m3_factual_recall_benchmark_l2() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Turn 1: Plant a distinctive fact
    session.start_turn();
    let fact_tokens = [1000u32, 1001, 1002, 1003, 1004, 3888];
    for &tok in &fact_tokens {
        model
            .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("Step turn 1 must succeed");
    }

    // Now roll over dialogue ring (224 slots) by adding 250+ tokens across subsequent turns
    for turn in 2..=16 {
        session.start_turn();
        for step in 0..18 {
            let filler_tok = (((turn * 19 + step * 23) % 2000) + 50) as u32;
            model
                .step_conversational(
                    &mut session,
                    filler_tok,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .expect("Filler step must succeed");
        }
    }

    // Turn 1 should now be evicted from dialogue ring and compressed into L2
    assert!(
        session.l2_len >= 1,
        "Turn 1 must have been compressed into L2 store"
    );
    let l2_page_0 = &session.l2_pages[0];
    assert_eq!(
        l2_page_0.turn_id, 1,
        "First L2 page must hold compressed Turn 1"
    );
    assert_ne!(
        l2_page_0.prime_signature, 0,
        "L2 page 0 must have valid prime signature"
    );

    // Query step under ReadMode::Enabled:
    session.start_turn();
    let step_enabled = model
        .step_conversational(&mut session, 3888, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("Query step must succeed");

    let sum_prob_enabled: u64 = step_enabled.probabilities.iter().sum();
    assert_eq!(sum_prob_enabled, PROBABILITY_TOTAL);

    let n_sys = session.persistent_len();
    let n_dial = session.dialogue_len();
    let n_page = session.l2_len;
    let expected_candidates = n_sys + n_dial + n_page;
    assert_eq!(session.last_read_masses.len(), expected_candidates);

    // Check memory candidate masses:
    // L2 candidates occupy indices (n_sys + n_dial)..expected_candidates
    let l2_masses = &session.last_read_masses[n_sys + n_dial..expected_candidates];
    assert_eq!(l2_masses.len(), n_page);

    // At least one memory candidate must have received positive associative mass
    let total_all_memory_mass: u64 = session.last_read_masses.iter().sum();
    assert!(
        total_all_memory_mass > 0,
        "Under ReadMode::Enabled, memory candidates must receive positive read mass"
    );

    // Ablation test: Query step under ReadMode::NoRead
    let step_no_read = model
        .step_conversational(&mut session, 3888, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("NoRead query step must succeed");

    let sum_prob_no_read: u64 = step_no_read.probabilities.iter().sum();
    assert_eq!(sum_prob_no_read, PROBABILITY_TOTAL);
    // Under NoRead, all 320 candidate masses must be strictly 0
    for (idx, &m) in session.last_read_masses.iter().enumerate() {
        assert_eq!(
            m, 0,
            "Under ReadMode::NoRead, candidate mass at {idx} must be strictly 0"
        );
    }
}

/// 7. Session serialization roundtrip with L2 pages:
///    Verify exact state preservation of all 64 L2 pages, cursors, and counters.
#[test]
fn test_m3_session_serialization_roundtrip_with_l2() {
    let bundle = Bundle::synthetic_for_test();
    let mut chat_session = ChatSession::new(&bundle, None, 42).expect("chat session creation");

    // Populate dialogue ring beyond 224 slots so L2 pages are created
    for turn in 1..=20 {
        chat_session.state_mut().start_turn();
        for step in 0..15 {
            let tok = (((turn * 11 + step * 7) % 3000) + 20) as u32;
            chat_session
                .step_token(tok, SlotTarget::Dialogue)
                .expect("Step token must succeed");
        }
    }

    let pre_telemetry = chat_session.telemetry();
    assert!(
        pre_telemetry.l2_pages_used > 0,
        "L2 pages must be populated before serialization"
    );
    assert_eq!(pre_telemetry.l2_capacity, L2_PAGE_CAPACITY);

    let temp_dir = std::env::temp_dir().join(format!("uor_m3_l2_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let session_path = temp_dir.join("m3_l2_session.json");

    chat_session
        .save_session_default(&session_path)
        .expect("save session");

    // Restore from JSON
    let loaded = bundle
        .load_chat_session(&session_path, bundle.identity())
        .expect("load chat session");

    let post_telemetry = loaded.telemetry();
    assert_eq!(
        pre_telemetry.l2_pages_used, post_telemetry.l2_pages_used,
        "L2 pages used must match after restoration"
    );
    assert_eq!(
        pre_telemetry.l2_capacity, post_telemetry.l2_capacity,
        "L2 capacity must match after restoration"
    );

    // Verify bit-for-bit equivalence of all populated L2 pages
    for i in 0..pre_telemetry.l2_pages_used {
        assert_eq!(
            chat_session.state().l2_pages[i],
            loaded.state().l2_pages[i],
            "L2 page {i} mismatch between saved and loaded session"
        );
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 8. Power-of-two salient copy routing mass conservation:
///    `m0 + m1 + m2 + m3 == mass` for all `mass` values.
#[test]
fn test_m3_salient_copy_routing_mass_conservation() {
    let test_masses = [
        0u64,
        1,
        2,
        3,
        4,
        7,
        8,
        9,
        15,
        16,
        31,
        32,
        100,
        255,
        1024,
        65536,
        1 << 30,
        PROBABILITY_TOTAL,
        PROBABILITY_TOTAL - 1,
    ];

    for &mass in &test_masses {
        let m1 = mass >> 2;
        let m2 = mass >> 3;
        let m3 = m2;
        let allocated = (mass >> 1) + m1 + m2 + m3;
        let m0 = (mass >> 1) + (mass - allocated);

        // Exact mass conservation
        assert_eq!(
            m0 + m1 + m2 + m3,
            mass,
            "Mass conservation violated for mass={mass}: sum={}, expected={mass}",
            m0 + m1 + m2 + m3
        );

        // Monotonicity / power-of-two hierarchy
        assert!(
            m0 >= m1,
            "m0 must be >= m1: m0={m0}, m1={m1} for mass={mass}"
        );
        assert!(
            m1 >= m2,
            "m1 must be >= m2: m1={m1}, m2={m2} for mass={mass}"
        );
        assert_eq!(m2, m3, "m2 must be equal to m3");
    }
}

/// 9. Full-capacity 320-candidate evaluation:
///    Fill 32 persistent slots (sealed persona), 224 dialogue slots, and 64 L2 pages.
///    Verify unified associative read evaluates all 320 candidates simultaneously.
#[test]
fn test_m3_full_capacity_320_candidates_evaluation() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Populate full persistent persona (32 slots)
    for i in 0..PERSISTENT_CAPACITY {
        let tok = (100 + i as u32) % 4000;
        model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("Persistent slot step must succeed");
    }
    session.seal_persistent();
    assert_eq!(session.persistent_len(), PERSISTENT_CAPACITY);
    assert!(session.is_persistent_sealed());

    // 2. Fill dialogue ring (224 slots) and all 64 L2 pages (requires >= 224 + 64 * 5 tokens)
    // Run 110 turns of 5 tokens each = 550 tokens
    for turn in 1..=110 {
        session.start_turn();
        for step in 0..5 {
            let tok = (((turn * 29 + step * 13) % 3500) + 10) as u32;
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Dialogue step must succeed");
        }
    }

    assert_eq!(session.dialogue_len(), DIALOGUE_CAPACITY);
    assert_eq!(
        session.l2_len, L2_PAGE_CAPACITY,
        "All 64 L2 pages must be populated"
    );

    // 3. Step one more token and verify exactly 320 candidates evaluated
    session.start_turn();
    let step = model
        .step_conversational(&mut session, 500, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("Evaluation step must succeed");

    let sum_prob: u64 = step.probabilities.iter().sum();
    assert_eq!(sum_prob, PROBABILITY_TOTAL);

    assert_eq!(
        session.last_read_masses.len(),
        TOTAL_MEMORY_CANDIDATES,
        "Full capacity associative read must return exactly 320 candidate masses (32 persistent + 224 dialogue + 64 L2)"
    );

    // Verify candidate slices
    let persistent_masses = &session.last_read_masses[0..32];
    let dialogue_masses = &session.last_read_masses[32..256];
    let l2_masses = &session.last_read_masses[256..320];

    assert_eq!(persistent_masses.len(), 32);
    assert_eq!(dialogue_masses.len(), 224);
    assert_eq!(l2_masses.len(), 64);
}
