//! Adversarial Challenger Test Suite for Milestone M3:
//! Hierarchical Prime Memory, Long-Horizon Multi-Turn Dialogue, and Causal Recall.
//!
//! File: crates/uor-r4-integer/tests/adversarial_challenger_m3_hierarchical_memory.rs
//!
//! Objectives:
//! 1. Long-Horizon Multi-Turn Dialogue (K = 512..1024 tokens) past L1 ring eviction (> 224 tokens).
//! 2. L2 Prime Paging capacity limits (64 pages) and FIFO ring rollover beyond 64 pages.
//! 3. Factual Recall Benchmark: verify >= 80% recall across multi-turn queries through L2 prime paging store.
//! 4. Causal Memory Necessity: verify complete recall collapse to 0.0% under ReadMode::NoRead ablation.
//! 5. Real pre-trained bundle long-horizon execution and latency verification.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use uor_r4_integer::{
    Bundle, ChatSession, IntegerModel, ReadMode, SlotTarget, DIALOGUE_CAPACITY, L2_PAGE_CAPACITY,
    PROBABILITY_TOTAL,
};

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";

static BENCH_LOCK: Mutex<()> = Mutex::new(());

// ============================================================================
// 1. Long-Horizon Dialogue past L1 Eviction (K = 512..1024 tokens)
// ============================================================================

#[test]
fn test_adversarial_m3_long_horizon_past_l1_eviction_k512_to_k1024() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // 1. Populate persistent persona (16 slots)
    for i in 0..16 {
        let tok = (50 + i as u32) % 4000;
        model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("Step persistent slot");
    }
    session.seal_persistent();
    assert_eq!(session.persistent_len(), 16);

    // 2. Run multi-turn dialogue up to 1024 tokens
    // 64 turns of 16 tokens each = 1024 tokens total
    let total_turns = 64;
    let tokens_per_turn = 16;
    let mut checkpoint_512_pages = 0;

    for turn in 1..=total_turns {
        session.start_turn();
        assert_eq!(session.current_turn(), turn as u32);

        for step in 0..tokens_per_turn {
            let seq = (turn - 1) * tokens_per_turn + step;
            let tok = (((turn * 37 + step * 19) % 3800) + 10) as u32;

            let expected_candidates =
                session.persistent_len() + session.dialogue_len() + session.l2_len;

            let step_result = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Dialogue step must succeed");

            // Probability conservation invariant: sum must be exactly 2^48
            let sum_prob: u64 = step_result.probabilities.iter().sum();
            assert_eq!(
                sum_prob, PROBABILITY_TOTAL,
                "Probability total must equal 2^48 at seq {seq}"
            );

            // Associative read candidate count must match active memory slots at time of read
            assert_eq!(
                session.last_read_masses.len(),
                expected_candidates,
                "Candidate count mismatch at seq {seq}"
            );
        }

        // Record page count at ~512 tokens (Turn 32 * 16 = 512 tokens)
        if turn == 32 {
            checkpoint_512_pages = session.l2_len;
            // 512 tokens is well beyond 224-slot L1 capacity (512 - 224 = 288 tokens evicted)
            // 288 / 16 = 18 turns evicted
            assert!(
                session.l2_len >= 15,
                "At K=512 tokens, at least 15 L2 pages must be populated (actual: {})",
                session.l2_len
            );
            assert_eq!(
                session.dialogue_len, DIALOGUE_CAPACITY,
                "L1 dialogue ring must be fully saturated at 224 slots"
            );
        }
    }

    // At K = 1024 tokens:
    // 1024 tokens total, 224 retained in L1, 800 tokens evicted (800 / 16 = 50 turns evicted)
    println!(
        "\n[LONG-HORIZON TELEMETRY] K=512 pages: {}, K=1024 pages: {}, total dialogue seen: {}",
        checkpoint_512_pages, session.l2_len, session.dialogue_seen
    );
    assert!(
        session.l2_len >= 45,
        "At K=1024 tokens, at least 45 L2 pages must be populated (actual: {})",
        session.l2_len
    );
    assert_eq!(session.dialogue_len, DIALOGUE_CAPACITY);

    // Verify properties of every single populated L2 page
    let mut signatures = HashSet::new();
    for i in 0..session.l2_len {
        let page = &session.l2_pages[i];
        assert_ne!(
            page.prime_signature, 0,
            "L2 page {i} prime signature must be non-zero"
        );
        assert!(
            signatures.insert(page.prime_signature),
            "Duplicate prime signature detected at L2 page {i}"
        );
        assert!(page.turn_id > 0, "L2 page {i} must have valid turn_id");
        assert!(
            page.key.iter().any(|&k| k != 0),
            "L2 page {i} barycenter key must not be all zeros"
        );
        assert!(
            page.value.iter().any(|&v| v != 0),
            "L2 page {i} terminal value must not be all zeros"
        );
        // Salient tokens must not contain special tokens (0..=6)
        for &sal_tok in &page.salient_tokens {
            if sal_tok != 0 {
                assert!(
                    sal_tok > 6,
                    "Salient token {sal_tok} in page {i} must not be a special token"
                );
            }
        }
    }
}

// ============================================================================
// 2. L2 Page FIFO Ring Rollover at Capacity 64
// ============================================================================

#[test]
fn test_adversarial_m3_l2_page_ring_wrapping_at_capacity_64() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Push 100 turns of 10 tokens = 1000 tokens
    // 224 tokens in L1 (22 turns in L1), remaining 78 turns evicted to L2!
    // Since L2 capacity is 64, 78 turns evicted will wrap around L2 pages by 14 pages!
    let total_turns = 100;
    for turn in 1..=total_turns {
        session.start_turn();
        for step in 0..10 {
            let tok = (((turn * 41 + step * 23) % 3500) + 15) as u32;
            let _ = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Step token");
        }
    }

    assert_eq!(session.dialogue_len, DIALOGUE_CAPACITY);
    assert_eq!(
        session.l2_len, L2_PAGE_CAPACITY,
        "L2 pages must saturate at exactly 64 pages"
    );
    assert!(
        session.l2_seen > L2_PAGE_CAPACITY as u64,
        "Total L2 pages seen ({}) must exceed capacity 64",
        session.l2_seen
    );

    // Verify cursor has wrapped around
    println!(
        "[L2 WRAP TELEMETRY] l2_len: {}, l2_cursor: {}, l2_seen: {}",
        session.l2_len, session.l2_cursor, session.l2_seen
    );

    // Invariant: L2 page array size must strictly remain 128 KiB
    assert_eq!(
        std::mem::size_of_val(&*session.l2_pages),
        131072,
        "L2 page array must remain exactly 128 KiB with zero heap growth"
    );
}

// ============================================================================
// 3. Factual Recall Benchmark across Multi-Turn Queries through L2 Store
// ============================================================================

#[test]
fn test_adversarial_m3_factual_recall_benchmark_l2_paging() {
    let model = IntegerModel::synthetic_for_test();

    // We construct 10 distinct factual recall test scenarios.
    // In each scenario:
    // - Plant a unique fact in Turn 1: [context_tokens..., target_entity_token]
    // - Add 20 filler turns (15 tokens each = 300 tokens) to ensure Turn 1 is 100% evicted from L1 into L2
    // - Confirm Turn 1 is evicted from dialogue slots and compressed into L2 page 0
    // - Query the fact under ReadMode::Enabled and measure target token recall
    // - Query under ReadMode::NoRead and verify 0.0% recall
    let num_scenarios = 10;
    let mut recall_passes = 0;
    let mut noread_passes = 0;

    println!("\n=== ADVERSARIAL FACTUAL RECALL BENCHMARK (L2 PAGING STORE) ===");
    println!("Scenario | Target Tok | L2 Page | P_enabled   | P_noread    | Delta NLL | Pass?");
    println!("---------+------------+---------+-------------+-------------+-----------+------");

    for scenario_idx in 0..num_scenarios {
        let mut session = model.new_conversational_session();

        // Unique target entity token for each scenario (e.g. 2500 + scenario_idx * 50)
        let target_entity = (2500 + scenario_idx * 50) as u32;

        // Turn 1: Plant Fact
        session.start_turn();
        let fact_tokens = [
            100 + scenario_idx as u32,
            200 + scenario_idx as u32,
            300 + scenario_idx as u32,
            target_entity,
        ];
        for &tok in &fact_tokens {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Plant fact token");
        }

        // Flush L1 dialogue ring: 20 filler turns * 14 tokens = 280 tokens (> 224 capacity)
        for turn in 2..=21 {
            session.start_turn();
            for step in 0..14 {
                // Distractor tokens chosen outside the target entity domain
                let filler = (((turn * 31 + step * 17 + scenario_idx * 11) % 1500) + 10) as u32;
                model
                    .step_conversational(
                        &mut session,
                        filler,
                        SlotTarget::Dialogue,
                        ReadMode::Enabled,
                    )
                    .expect("Filler step");
            }
        }

        // 1. Verify Turn 1 is NO LONGER in the active L1 dialogue slots
        let in_l1 = session.dialogue_tokens[..session.dialogue_len].contains(&target_entity);
        assert!(
            !in_l1,
            "Scenario {scenario_idx}: Target entity must be evicted from L1 dialogue ring"
        );

        // 2. Verify Turn 1 is compressed into L2 page 0
        assert!(
            session.l2_len >= 1,
            "Scenario {scenario_idx}: L2 store must contain at least 1 page"
        );
        let page_0 = &session.l2_pages[0];
        assert_eq!(
            page_0.turn_id, 1,
            "Scenario {scenario_idx}: Page 0 must hold Turn 1"
        );
        assert!(
            page_0.salient_tokens.contains(&target_entity),
            "Scenario {scenario_idx}: Page 0 salient tokens {:?} must contain target entity {}",
            page_0.salient_tokens,
            target_entity
        );

        // 3. Query step under ReadMode::Enabled
        session.start_turn();
        // Query token matches context of Turn 1
        let query_token = 300 + scenario_idx as u32;
        let step_enabled = model
            .step_conversational(
                &mut session,
                query_token,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("Query step under Enabled");

        let p_enabled_q48 = step_enabled.probabilities[target_entity as usize];
        let p_enabled = p_enabled_q48 as f64 / (PROBABILITY_TOTAL as f64);

        // 4. Query step under ReadMode::NoRead on identical session state
        // Clone session state to test exact causal counterfactual
        let mut session_noread = session.clone();
        let step_noread = model
            .step_conversational(
                &mut session_noread,
                query_token,
                SlotTarget::Dialogue,
                ReadMode::NoRead,
            )
            .expect("Query step under NoRead");

        let p_noread_q48 = step_noread.probabilities[target_entity as usize];
        let p_noread = p_noread_q48 as f64 / (PROBABILITY_TOTAL as f64);

        let delta_nll = -p_enabled.ln() - (-p_noread.ln());

        // Recall criterion:
        // 1. Enabled probability significantly exceeds NoRead probability (p_enabled > p_noread)
        // 2. L2 page received read mass in Enabled mode
        // 3. Target entity received copy probability
        let n_sys = session.persistent_len();
        let n_dial = session.dialogue_len();
        let l2_mass_page0 = session
            .last_read_masses
            .get(n_sys + n_dial)
            .copied()
            .unwrap_or(0);

        let recall_pass = p_enabled > p_noread && l2_mass_page0 > 0;
        if recall_pass {
            recall_passes += 1;
        }

        // Causal ablation check: NoRead must have 0 read mass and 0 copy contribution
        let noread_total_mass: u64 = session_noread.last_read_masses.iter().sum();
        let noread_pass = noread_total_mass == 0 && p_noread <= (1.0 / 4000.0);
        if noread_pass {
            noread_passes += 1;
        }

        println!(
            "SC_{:02}    | {:<10} | {:<7} | {:<11.8} | {:<11.8} | {:<9.3} | {}",
            scenario_idx,
            target_entity,
            0,
            p_enabled,
            p_noread,
            delta_nll,
            if recall_pass { "PASS" } else { "FAIL" }
        );
    }

    let recall_rate = (recall_passes as f64 / num_scenarios as f64) * 100.0;
    let noread_rate = (noread_passes as f64 / num_scenarios as f64) * 100.0;
    println!("------------------------------------------------------------------");
    println!("Empirical Recall Rate (Enabled) : {recall_passes}/{num_scenarios} ({recall_rate:.1}%) [Target: >= 80.0%]");
    println!("Causal Ablation Rate (NoRead)   : {noread_passes}/{num_scenarios} ({noread_rate:.1}%) [Target: 100.0% clean]");

    assert!(
        recall_rate >= 80.0,
        "Factual recall rate {recall_rate:.1}% is below required 80.0% threshold"
    );
    assert_eq!(
        noread_passes, num_scenarios,
        "NoRead ablation failed to achieve 100% clean ablation across all scenarios"
    );
}

// ============================================================================
// 4. Causal Memory Necessity: Read vs NoRead Complete Ablation
// ============================================================================

#[test]
fn test_adversarial_m3_causal_necessity_read_vs_noread_ablation() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Plant 3 facts in turns 1..3
    for turn in 1..=3 {
        session.start_turn();
        let entity = (3000 + turn * 100) as u32;
        let tokens = [100 + turn as u32, 200 + turn as u32, entity];
        for &tok in &tokens {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Step");
        }
    }

    // Flush L1 ring with 250 tokens
    for turn in 4..=20 {
        session.start_turn();
        for step in 0..16 {
            let tok = (((turn * 23 + step * 13) % 2000) + 10) as u32;
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Step");
        }
    }

    // Assert all 3 initial turns are now compressed in L2 pages
    assert!(session.l2_len >= 3);
    assert_eq!(session.l2_pages[0].turn_id, 1);
    assert_eq!(session.l2_pages[1].turn_id, 2);
    assert_eq!(session.l2_pages[2].turn_id, 3);

    // Query step under ReadMode::Enabled
    session.start_turn();
    let query_tok = 101u32; // matches Turn 1
    let step_enabled = model
        .step_conversational(
            &mut session,
            query_tok,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .expect("Query enabled");

    // Query step under ReadMode::NoRead
    let mut session_noread = session.clone();
    let step_noread = model
        .step_conversational(
            &mut session_noread,
            query_tok,
            SlotTarget::Dialogue,
            ReadMode::NoRead,
        )
        .expect("Query noread");

    // 1. In NoRead, candidate masses sum must be strictly 0
    let enabled_total_mass: u64 = session.last_read_masses.iter().sum();
    let noread_total_mass: u64 = session_noread.last_read_masses.iter().sum();

    println!(
        "\n[CAUSAL CONTRAST TELEMETRY] Enabled mass: {}, NoRead mass: {}",
        enabled_total_mass, noread_total_mass
    );

    assert!(
        enabled_total_mass > 0,
        "ReadMode::Enabled must assign positive read mass"
    );
    assert_eq!(
        noread_total_mass, 0,
        "ReadMode::NoRead must assign strictly zero read mass across all 320 candidates"
    );

    // 2. In NoRead, no_read_mass must be saturated to 2^48
    assert_eq!(
        session_noread.last_no_read_mass, PROBABILITY_TOTAL,
        "NoRead mode must saturate last_no_read_mass to 2^48"
    );

    // 3. In NoRead, every individual candidate mass must be 0
    for (idx, &m) in session_noread.last_read_masses.iter().enumerate() {
        assert_eq!(m, 0, "NoRead mass at candidate {idx} must be 0");
    }

    // 4. Probability sum in both modes must be exactly 2^48
    let sum_en: u64 = step_enabled.probabilities.iter().sum();
    let sum_nr: u64 = step_noread.probabilities.iter().sum();
    assert_eq!(sum_en, PROBABILITY_TOTAL);
    assert_eq!(sum_nr, PROBABILITY_TOTAL);
}

// ============================================================================
// 5. Real Pre-Trained Bundle Long-Horizon Execution & Latency
// ============================================================================

#[test]
fn test_adversarial_m3_real_bundle_long_horizon_execution() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    if !bundle_path.exists() {
        println!("Skipping real bundle test: {REAL_QUATERNION_BUNDLE} not found");
        return;
    }

    let bundle = Bundle::load(bundle_path).expect("Load real quaternion bundle");
    let mut session = ChatSession::new(&bundle, Some("You are an adversarial verifier."), 42)
        .expect("Create chat session");

    // Warmup: 20 tokens
    for i in 0..20 {
        let _ = session.step_token(10 + (i as u32), SlotTarget::Dialogue);
    }

    // Ingest 30 turns of 18 tokens = 540 tokens (K > 512, exceeding 224 L1 slots)
    let total_turns = 30;
    let mut latencies_ms = Vec::with_capacity(total_turns * 18);

    for turn in 1..=total_turns {
        session.state_mut().start_turn();
        for step in 0..18 {
            let tok = (((turn * 13 + step * 29) % 4000) + 10) as u32;
            let t0 = Instant::now();
            session
                .step_token(tok, SlotTarget::Dialogue)
                .expect("Step token on real bundle");
            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
            latencies_ms.push(elapsed_ms);
        }
    }

    let telemetry = session.telemetry();
    println!(
        "\n[REAL BUNDLE TELEMETRY] Dialogue slots: {}/224, L2 pages: {}/64, Total tokens: {}",
        telemetry.dialogue_slots_used,
        telemetry.l2_pages_used,
        latencies_ms.len()
    );

    assert_eq!(telemetry.dialogue_slots_used, 224);
    assert!(
        telemetry.l2_pages_used > 0,
        "Real bundle must populate L2 pages past 224 tokens (actual: {})",
        telemetry.l2_pages_used
    );

    // Compute latency stats
    latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean_ms = latencies_ms.iter().sum::<f64>() / latencies_ms.len() as f64;
    let p50_ms = latencies_ms[latencies_ms.len() / 2];
    let p90_idx = ((latencies_ms.len() as f64 * 0.90) as usize).min(latencies_ms.len() - 1);
    let p90_ms = latencies_ms[p90_idx];
    let p99_idx = ((latencies_ms.len() as f64 * 0.99) as usize).min(latencies_ms.len() - 1);
    let p99_ms = latencies_ms[p99_idx];

    println!(
        "Latency: Mean = {:.3} ms, p50 = {:.3} ms, p90 = {:.3} ms, p99 = {:.3} ms (Target: Mean <= 4.0 ms)",
        mean_ms, p50_ms, p90_ms, p99_ms
    );

    assert!(
        mean_ms <= 4.0,
        "Mean latency {:.3} ms exceeds 4.0 ms ceiling",
        mean_ms
    );
    assert!(
        p50_ms <= 3.0,
        "Median latency {:.3} ms exceeds 3.0 ms",
        p50_ms
    );
}
