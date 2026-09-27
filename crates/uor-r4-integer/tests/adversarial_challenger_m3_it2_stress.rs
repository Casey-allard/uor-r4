//! Deep Adversarial Stress Harness for Milestone M3 Hierarchical Memory
//! Challenger: challenger_chatbot7_m3_it2_1
//!
//! Vectors:
//! 1. Multi-fact retention and recall across long horizons (K = 512..1024 tokens)
//! 2. L2 page eviction beyond capacity 64 and FIFO overwrite dynamics
//! 3. Single-turn overflow (> 224 tokens in a single turn)
//! 4. Empty and degenerate turn sequences
//! 5. Causal necessity matrix: FullRead vs NoRead vs L2-Ablated vs Salient-Ablated
//! 6. Real bundle stress test (if available)

use std::collections::HashSet;
use std::path::Path;
use uor_r4_integer::{
    Bundle, ChatSession, IntegerModel, ReadMode, SlotTarget, DIALOGUE_CAPACITY, L2_PAGE_CAPACITY,
    PROBABILITY_TOTAL,
};

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";

/// Test 1: Multi-fact long-horizon retrieval (K = 512..1024 tokens).
/// Plants 5 distinct facts in turns 1..5.
/// Then injects 40 filler turns (15 tokens each = 600 tokens of distractors).
/// Then queries each of the 5 planted facts.
/// Verifies >= 80% recall across the 5 facts, and verifies 0% recall under NoRead.
#[test]
fn test_adversarial_m3_it2_multi_fact_long_horizon_retrieval() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    let num_facts = 5;
    let mut target_entities = Vec::new();
    let mut query_tokens = Vec::new();

    // Plant 5 facts in turns 1..5
    for turn in 1..=num_facts {
        session.start_turn();
        let entity = (2100 + turn * 100) as u32;
        let query_tok = (500 + turn * 50) as u32;
        target_entities.push(entity);
        query_tokens.push(query_tok);

        let fact_tokens = [query_tok, 1500, 1600, entity];
        for &tok in &fact_tokens {
            model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Plant fact");
        }
    }

    // Ingest 40 filler turns of 15 tokens each = 600 tokens (Total K > 620 tokens)
    // Ensures turns 1..5 are 100% evicted from L1 (224 slots) into L2
    for turn in (num_facts + 1)..=(num_facts + 40) {
        session.start_turn();
        for step in 0..15 {
            let filler = (((turn * 29 + step * 31) % 1800) + 10) as u32;
            model
                .step_conversational(
                    &mut session,
                    filler,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .expect("Filler token");
        }
    }

    // Verify all 5 facts are evicted from L1
    for &entity in &target_entities {
        let in_l1 = session.dialogue_tokens[..session.dialogue_len].contains(&entity);
        assert!(!in_l1, "Target entity {entity} must be evicted from L1");
    }

    // Verify all 5 turns are compressed in L2 pages
    assert!(session.l2_len >= num_facts);
    for i in 0..num_facts {
        assert_eq!(
            session.l2_pages[i].turn_id,
            (i + 1) as u32,
            "L2 page {i} must hold Turn {}",
            i + 1
        );
        assert!(
            session.l2_pages[i]
                .salient_tokens
                .contains(&target_entities[i]),
            "L2 page {i} must contain entity {}",
            target_entities[i]
        );
    }

    // Query each fact under ReadMode::Enabled and ReadMode::NoRead
    let mut recalled_count = 0;
    let mut noread_count = 0;

    for i in 0..num_facts {
        let query_tok = query_tokens[i];
        let target_entity = target_entities[i];

        // 1. Query Enabled
        session.start_turn();
        let step_en = model
            .step_conversational(
                &mut session,
                query_tok,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .expect("Query enabled");
        let p_en = step_en.probabilities[target_entity as usize];

        // 2. Query NoRead
        let mut session_nr = session.clone();
        let step_nr = model
            .step_conversational(
                &mut session_nr,
                query_tok,
                SlotTarget::Dialogue,
                ReadMode::NoRead,
            )
            .expect("Query noread");
        let p_nr = step_nr.probabilities[target_entity as usize];

        // Check if page i received read mass
        let n_sys = session.persistent_len();
        let n_dial = session.dialogue_len();
        let page_mass = session
            .last_read_masses
            .get(n_sys + n_dial + i)
            .copied()
            .unwrap_or(0);

        if p_en > p_nr && page_mass > 0 {
            recalled_count += 1;
        }

        let nr_total_mass: u64 = session_nr.last_read_masses.iter().sum();
        if nr_total_mass == 0 && p_nr <= (PROBABILITY_TOTAL / 4000) {
            noread_count += 1;
        }

        println!(
            "Fact {}: Entity={}, P_en={}, P_nr={}, L2_mass={}, Recall={}",
            i + 1,
            target_entity,
            p_en,
            p_nr,
            page_mass,
            p_en > p_nr && page_mass > 0
        );
    }

    let recall_rate = (recalled_count as f64 / num_facts as f64) * 100.0;
    println!("Multi-Fact Recall Rate: {recalled_count}/{num_facts} ({recall_rate:.1}%)");
    assert!(
        recall_rate >= 80.0,
        "Recall rate {recall_rate:.1}% is below 80% benchmark"
    );
    assert_eq!(
        noread_count, num_facts,
        "NoRead must achieve 100% causal collapse"
    );
}

/// Test 2: Fine-grained Causal Ablation Matrix.
/// Verifies:
/// 1. Fact evicted to L2 is recalled under Full Read.
/// 2. If L2 is specifically ablated (session.l2_len = 0), recall for the evicted fact collapses!
/// 3. Meanwhile, a recent fact still in L1 is STILL recalled even when L2 is ablated.
/// This proves the strict causal necessity of the L2 tier for evicted facts.
#[test]
fn test_adversarial_m3_it2_l2_specific_causal_ablation() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    // Turn 1: Plant Old Fact (will be evicted to L2)
    session.start_turn();
    let old_entity = 3100u32;
    let old_query = 410u32;
    for &t in &[old_query, 1200, old_entity] {
        model
            .step_conversational(&mut session, t, SlotTarget::Dialogue, ReadMode::Enabled)
            .unwrap();
    }

    // Ingest 250 filler tokens across turns 2..15
    for turn in 2..=15 {
        session.start_turn();
        for s in 0..18 {
            let filler = (((turn * 17 + s * 13) % 1500) + 10) as u32;
            model
                .step_conversational(
                    &mut session,
                    filler,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap();
        }
    }

    // Turn 16: Plant Recent Fact (stays in L1)
    session.start_turn();
    let recent_entity = 3200u32;
    let recent_query = 420u32;
    for &t in &[recent_query, 1300, recent_entity] {
        model
            .step_conversational(&mut session, t, SlotTarget::Dialogue, ReadMode::Enabled)
            .unwrap();
    }

    // Verify old fact is evicted from L1 and present in L2 page 0
    assert!(!session.dialogue_tokens[..session.dialogue_len].contains(&old_entity));
    assert_eq!(session.l2_pages[0].turn_id, 1);
    assert!(session.l2_pages[0].salient_tokens.contains(&old_entity));

    // Verify recent fact IS present in L1
    assert!(session.dialogue_tokens[..session.dialogue_len].contains(&recent_entity));

    // Case A: Query Old Fact with L2 intact
    session.start_turn();
    let step_old_intact = model
        .step_conversational(
            &mut session,
            old_query,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap();
    let p_old_intact = step_old_intact.probabilities[old_entity as usize];

    // Case B: Query Old Fact with L2 ABLATED (set l2_len = 0)
    let mut session_l2_ablated = session.clone();
    session_l2_ablated.l2_len = 0; // Ablate L2 tier specifically
    let step_old_ablated = model
        .step_conversational(
            &mut session_l2_ablated,
            old_query,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap();
    let p_old_ablated = step_old_ablated.probabilities[old_entity as usize];

    println!(
        "Old Fact (Evicted to L2): Intact P = {}, L2-Ablated P = {}",
        p_old_intact, p_old_ablated
    );
    assert!(
        p_old_intact > p_old_ablated,
        "Ablating L2 must strictly reduce recall probability of evicted fact"
    );

    // Case C: Query Recent Fact with L2 intact vs L2 ABLATED
    let step_recent_intact = model
        .step_conversational(
            &mut session,
            recent_query,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap();
    let p_recent_intact = step_recent_intact.probabilities[recent_entity as usize];

    let step_recent_ablated = model
        .step_conversational(
            &mut session_l2_ablated,
            recent_query,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap();
    let p_recent_ablated = step_recent_ablated.probabilities[recent_entity as usize];

    println!(
        "Recent Fact (in L1): Intact P = {}, L2-Ablated P = {}",
        p_recent_intact, p_recent_ablated
    );
    // Recent fact lives in L1, so it is preserved even when L2 is ablated!
    assert!(
        p_recent_ablated > (PROBABILITY_TOTAL / 4000),
        "Recent fact must still be recalled when L2 is ablated because it lives in L1"
    );
}

/// Test 3: Single-Turn Overflow (> 224 tokens in a single turn).
/// Tests that ingesting 300 tokens in a SINGLE turn without start_turn()
/// does not crash, does not corrupt ring buffer indices, and maintains invariants.
#[test]
fn test_adversarial_m3_it2_single_turn_overflow_beyond_224() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    session.start_turn();
    assert_eq!(session.current_turn(), 1);

    // Ingest 300 tokens in Turn 1
    for step in 0..300 {
        let tok = ((step * 17) % 3500 + 10) as u32;
        let step_res = model
            .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("Step during single-turn overflow");
        let sum_p: u64 = step_res.probabilities.iter().sum();
        assert_eq!(sum_p, PROBABILITY_TOTAL);
    }

    assert_eq!(session.dialogue_len, DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_cursor, 300 % DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_seen, 300);

    // Turn 1 should have been compressed when the cursor wrapped at step 224
    assert!(
        session.l2_len >= 1,
        "Overflow should have triggered L2 compression"
    );
    assert_eq!(session.l2_pages[0].turn_id, 1);
    assert_ne!(session.l2_pages[0].prime_signature, 0);
}

/// Test 4: Extreme FIFO Wraparound of L2 Pages (2000 tokens, 100 turns).
/// Sits through multiple complete rotations of the 64-page L2 store.
/// Verifies invariant bounds, no memory leak, and monotonic sequence accounting.
#[test]
fn test_adversarial_m3_it2_extreme_l2_fifo_wraparound() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    let total_turns = 120;
    let tokens_per_turn = 20; // 2400 tokens total

    for turn in 1..=total_turns {
        session.start_turn();
        for step in 0..tokens_per_turn {
            let tok = (((turn * 43 + step * 29) % 3900) + 10) as u32;
            let _ = model
                .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
                .expect("Step in extreme wraparound");
        }
    }

    assert_eq!(session.dialogue_len, DIALOGUE_CAPACITY);
    assert_eq!(session.dialogue_seen, 2400);
    assert_eq!(session.l2_len, L2_PAGE_CAPACITY);
    assert!(session.l2_seen > 90);

    // Verify all 64 L2 pages are populated and hold recent turns
    let mut turn_ids = HashSet::new();
    for i in 0..L2_PAGE_CAPACITY {
        let page = &session.l2_pages[i];
        assert_ne!(page.prime_signature, 0);
        assert!(page.turn_id > 0);
        turn_ids.insert(page.turn_id);
    }
    // All 64 pages should hold distinct turns
    assert_eq!(
        turn_ids.len(),
        L2_PAGE_CAPACITY,
        "All 64 pages must hold distinct turns"
    );
}

/// Test 5: Empty turns and rapid turn switching.
/// Verifies starting turns without stepping tokens does not produce invalid L2 pages.
#[test]
fn test_adversarial_m3_it2_empty_turns_edge_cases() {
    let model = IntegerModel::synthetic_for_test();
    let mut session = model.new_conversational_session();

    for _ in 0..50 {
        session.start_turn();
    }
    assert_eq!(session.current_turn(), 50);
    assert_eq!(session.l2_len, 0, "Empty turns must not generate L2 pages");
    assert_eq!(session.dialogue_len, 0);

    // Now step 1 token in turn 50
    model
        .step_conversational(&mut session, 100, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("Step after empty turns");
    assert_eq!(session.dialogue_len, 1);
    assert_eq!(session.l2_len, 0);
}

/// Test 6: Real Quaternion Bundle Long-Horizon Verification (if bundle exists).
#[test]
fn test_adversarial_m3_it2_real_bundle_factual_recall_and_noread() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    if !bundle_path.exists() {
        println!("Skipping real bundle test: bundle not present");
        return;
    }

    let bundle = Bundle::load(bundle_path).expect("Load real bundle");
    let mut chat = ChatSession::new(&bundle, Some("You are an empirical challenger."), 999)
        .expect("Create chat session");

    // Plant fact in Turn 1
    chat.state_mut().start_turn();
    let fact_tok = 2888u32;
    for &t in &[500u32, 600, fact_tok] {
        chat.step_token(t, SlotTarget::Dialogue).unwrap();
    }

    // Roll over L1 (250 tokens across 15 turns)
    for turn in 2..=16 {
        chat.state_mut().start_turn();
        for step in 0..17 {
            let filler = (((turn * 19 + step * 37) % 3500) + 20) as u32;
            chat.step_token(filler, SlotTarget::Dialogue).unwrap();
        }
    }

    let tel = chat.telemetry();
    assert_eq!(tel.dialogue_slots_used, 224);
    assert!(tel.l2_pages_used >= 1);

    // Verify fact is in L2
    assert!(!chat.state().dialogue_tokens[..224].contains(&fact_tok));
    assert!(chat.state().l2_pages[0].salient_tokens.contains(&fact_tok));

    // Step query under Enabled
    chat.state_mut().start_turn();
    let step_en = bundle
        .model()
        .step_conversational(
            chat.state_mut(),
            500,
            SlotTarget::Dialogue,
            ReadMode::Enabled,
        )
        .unwrap();
    let p_en = step_en.probabilities[fact_tok as usize];

    // Step query under NoRead
    let mut state_nr = chat.state().clone();
    let step_nr = bundle
        .model()
        .step_conversational(&mut state_nr, 500, SlotTarget::Dialogue, ReadMode::NoRead)
        .unwrap();
    let p_nr = step_nr.probabilities[fact_tok as usize];

    println!(
        "Real Bundle Fact Recall: P_enabled = {}, P_noread = {}",
        p_en, p_nr
    );
    assert!(
        p_en > p_nr,
        "Real bundle must exhibit positive memory recall boost"
    );
    let nr_mass: u64 = state_nr.last_read_masses.iter().sum();
    assert_eq!(
        nr_mass, 0,
        "NoRead must have strictly 0 read mass on real bundle"
    );
}
