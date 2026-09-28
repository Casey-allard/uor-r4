use uor_r4_integer::{
    Bundle, ChatSession, IntegerModel, L2PrimePage, ReadGeometry, ReadMode, SamplePolicy,
    SlotTarget, TOTAL_MEMORY_CANDIDATES,
};

const TOTAL: u64 = 1u64 << 48;

#[test]
fn test_conversational_refusal_without_hyperbolic_flag() {
    let model = IntegerModel::synthetic_lorentz_for_test();
    assert_eq!(model.config().read_geometry, ReadGeometry::Lorentz);
    assert!(model.is_lorentz());

    let mut session = model.new_conversational_session();
    assert!(!session.allow_hyperbolic_cache);

    // Default conversational step on Lorentz without allow_hyperbolic_cache must error
    // (exact backward-compatibility contract)
    let result =
        model.step_conversational(&mut session, 10, SlotTarget::Dialogue, ReadMode::Enabled);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("dot-read"),
        "error must mention dot-read: {err_str}"
    );
}

#[test]
fn test_conversational_hyperbolic_cache_stepping_and_paging() {
    let model = IntegerModel::synthetic_lorentz_for_test();
    let mut session = model.new_conversational_session();
    session.enable_hyperbolic_cache();
    assert!(session.allow_hyperbolic_cache);

    // 1. Ingest into persistent persona (slots 0..3)
    for tok in [101, 102, 103] {
        let step = model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("persistent slot stepping");
        let sum: u64 = step.probabilities.iter().sum();
        assert_eq!(sum, TOTAL, "probabilities must sum to 2^48");
    }
    assert_eq!(session.persistent_keys.len(), 3);
    assert_eq!(session.persistent_key_norms.len(), 3);
    for &kn in &session.persistent_key_norms {
        assert!(kn >= 0, "Lorentz key norm must be non-negative: {kn}");
    }

    session.seal_persistent();
    assert!(session.is_persistent_sealed());

    // 2. Ingest dialogue tokens into rolling ring buffer
    session.start_turn();
    for tok in 200..210 {
        let step = model
            .step_conversational(&mut session, tok, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("dialogue stepping");
        let sum: u64 = step.probabilities.iter().sum();
        assert_eq!(sum, TOTAL);
    }
    assert_eq!(session.dialogue_len(), 10);
    for idx in 0..10 {
        assert!(session.dialogue_key_norms[idx] >= 0);
    }

    // 3. Test ReadMode::NoRead on hyperbolic cache
    let step_noread = model
        .step_conversational(&mut session, 999, SlotTarget::Dialogue, ReadMode::NoRead)
        .expect("step with NoRead");
    assert_eq!(step_noread.no_read_mass, TOTAL);
    assert!(step_noread.read_masses.iter().all(|&m| m == 0));

    // 4. Exercise L2 Prime Page eviction by filling dialogue capacity (224 slots)
    // Create multiple turns to trigger L2 compression
    for turn in 1..=4 {
        session.start_turn();
        for tok in 0..60 {
            model
                .step_conversational_hyperbolic(
                    &mut session,
                    (turn * 100 + tok) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .expect("stepping towards L2 paging");
        }
    }

    // Dialogue capacity is 224, 10 + 240 = 250 steps have occurred, so L2 pages must have been created!
    assert!(
        session.l2_len() > 0,
        "L2 prime paging must have compressed evicted turns into pages (l2_len = {})",
        session.l2_len()
    );
    for p_idx in 0..session.l2_len() {
        assert!(
            session.l2_page_norms[p_idx] >= 0,
            "L2 page key norm must be computed: {}",
            session.l2_page_norms[p_idx]
        );
    }

    // Verify stepping with active L2 pages scores correctly and sums to TOTAL
    let step_with_l2 = model
        .step_conversational(&mut session, 500, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("step with active L2 pages in hyperbolic cache");
    let sum_l2: u64 = step_with_l2.probabilities.iter().sum();
    assert_eq!(sum_l2, TOTAL);
    assert!(step_with_l2.read_masses.len() <= TOTAL_MEMORY_CANDIDATES);
}

#[test]
fn test_chat_session_hyperbolic_cache_creation_and_streaming() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    assert!(bundle.model().is_lorentz());

    let persona = "Hyperbolic bot.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 42).expect("chat session creation");

    // Hyperbolic cache should be automatically enabled for Lorentz bundle
    assert!(chat.state().allow_hyperbolic_cache);
    assert!(chat.state().is_persistent_sealed());
    assert!(chat.state().persistent_len() > 0);
    assert_eq!(
        chat.state().persistent_key_norms.len(),
        chat.state().persistent_len()
    );

    // Stream generation tokens using Categorical policy from a user prompt
    chat.set_policy(SamplePolicy::Categorical { top_k: 8 });
    let user_msg = "Explain hyperbolic spacetime geometry.";
    let mut stream = chat
        .generate_stream(user_msg, 10, &[])
        .expect("generate stream");
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next() {
        chunks.push(chunk);
    }
    assert!(stream.tokens_generated() > 0);
    assert!(stream.tokens_generated() <= 10);
    assert_eq!(stream.tokens_generated(), stream.generated_tokens().len());
    assert!(stream.is_stopped());
}

#[test]
fn test_chat_session_hyperbolic_serialization_roundtrip() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    let persona = "Lorentz assistant.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 12345).expect("chat session creation");
    chat.set_policy(SamplePolicy::Categorical { top_k: 4 });

    // Multi-turn exchanges
    for turn in 1..=3 {
        let msg = format!("Turn {turn} message text query");
        let mut stream = chat.generate_stream(&msg, 5, &[]).expect("stream for turn");
        while let Some(_) = stream.next() {}
    }

    // Save session to temporary file
    let tmp_dir = std::env::temp_dir();
    let save_path = tmp_dir.join(format!(
        "uor_r4_hyperbolic_session_{}.json",
        std::process::id()
    ));

    chat.save_session_default(&save_path)
        .expect("save session to json");
    assert!(save_path.exists());

    // Restore session from JSON
    let mut restored = bundle
        .load_chat_session(&save_path, "")
        .expect("load hyperbolic chat session");

    // Clean up temp file
    let _ = std::fs::remove_file(&save_path);

    // Verification of restored session state
    assert!(restored.state().allow_hyperbolic_cache);
    assert_eq!(
        restored.state().persistent_len(),
        chat.state().persistent_len()
    );
    assert_eq!(restored.state().dialogue_len(), chat.state().dialogue_len());
    assert_eq!(restored.state().current_turn(), chat.state().current_turn());

    // Verify bit-for-bit key norms equality
    assert_eq!(
        restored.state().persistent_key_norms,
        chat.state().persistent_key_norms
    );
    for i in 0..chat.state().dialogue_len() {
        assert_eq!(
            restored.state().dialogue_key_norms[i],
            chat.state().dialogue_key_norms[i]
        );
    }
    for i in 0..chat.state().l2_len() {
        assert_eq!(
            restored.state().l2_page_norms[i],
            chat.state().l2_page_norms[i]
        );
    }

    // Verify determinism: next turn generation from original and restored must yield identical tokens
    let test_prompt = "Verify deterministic continuation.";
    let mut stream_orig = chat
        .generate_stream(test_prompt, 15, &[])
        .expect("stream orig");
    let mut tokens_orig = Vec::new();
    while let Some(_) = stream_orig.next() {}
    tokens_orig.extend_from_slice(stream_orig.generated_tokens());

    let mut stream_rest = restored
        .generate_stream(test_prompt, 15, &[])
        .expect("stream rest");
    let mut tokens_rest = Vec::new();
    while let Some(_) = stream_rest.next() {}
    tokens_rest.extend_from_slice(stream_rest.generated_tokens());

    assert_eq!(
        tokens_orig, tokens_rest,
        "Original and restored hyperbolic sessions must produce bit-identical token sequences!"
    );
}

fn get_process_rss_mb() -> Option<f64> {
    let pid = std::process::id();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = std::str::from_utf8(&output.stdout).ok()?.trim();
    let rss_kib: f64 = text.parse().ok()?;
    Some(rss_kib / 1024.0)
}

#[test]
fn test_conversational_hyperbolic_cache_l2_wrapping_past_64_pages() {
    let model = IntegerModel::synthetic_lorentz_for_test();
    let mut session = model.new_conversational_session();
    session.enable_hyperbolic_cache();

    // 1. Ingest 3 persistent persona slots
    for tok in [101, 102, 103] {
        model
            .step_conversational(&mut session, tok, SlotTarget::Persistent, ReadMode::Enabled)
            .expect("persistent slot stepping");
    }
    session.seal_persistent();

    // 2. Generate 90 turns, each with 10 tokens (900 dialogue steps)
    // Dialogue capacity is 224, so turns will be evicted and compressed into L2 pages.
    // Evicting 90 - 22 = ~68 turns will wrap L2 cursor (capacity 64) past 64!
    for turn in 1..=90 {
        session.start_turn();
        for tok in 0..10 {
            let token_id = ((turn * 17 + tok * 3) % 4000 + 7) as u32;
            let step = model
                .step_conversational_hyperbolic(
                    &mut session,
                    token_id,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .expect("stepping dialogue with hyperbolic cache");
            let sum: u64 = step.probabilities.iter().sum();
            assert_eq!(sum, TOTAL, "step probabilities must sum to TOTAL");
        }
    }

    // Verify L2 page wrapping invariants
    assert!(
        session.l2_seen >= 65,
        "Must have compressed >= 65 turns into L2 pages (seen: {})",
        session.l2_seen
    );
    assert_eq!(
        session.l2_len(),
        64,
        "L2 capacity is 64; len must be clamped at 64"
    );
    assert_eq!(
        session.l2_cursor,
        (session.l2_seen % 64) as usize,
        "L2 cursor must wrap modulo 64"
    );

    // Verify all 64 L2 page norms are strictly positive
    for (idx, &norm) in session.l2_page_norms.iter().enumerate() {
        assert!(
            norm > 0,
            "L2 page norm at index {idx} must be strictly positive: {norm}"
        );
    }

    // Verify stepping with 64 wrapped L2 pages succeeds and read_masses covers all candidates
    let step_with_wrapped_l2 = model
        .step_conversational(&mut session, 2048, SlotTarget::Dialogue, ReadMode::Enabled)
        .expect("stepping with wrapped L2 pages");
    let sum_wrapped: u64 = step_with_wrapped_l2.probabilities.iter().sum();
    assert_eq!(sum_wrapped, TOTAL);
    assert_eq!(
        step_with_wrapped_l2.read_masses.len(),
        3 + 224 + 64, // 291 active candidate slots
        "Candidate read masses must cover 3 persistent + 224 dialogue + 64 L2 slots"
    );
}

#[test]
fn test_chat_session_hyperbolic_serialization_roundtrip_with_wrapped_l2_pages() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    let persona = "Lorentz memory.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 99999).expect("chat session creation");
    chat.set_policy(SamplePolicy::Categorical { top_k: 4 });

    // Multi-turn exchanges to fill dialogue capacity and generate multiple L2 pages
    for turn in 1..=30 {
        let msg = format!("Message for turn {turn} testing L2 compression");
        let mut stream = chat
            .generate_stream(&msg, 10, &[])
            .expect("stream for turn");
        while let Some(_) = stream.next() {}
    }

    assert!(
        chat.state().l2_len() > 0,
        "ChatSession must have generated L2 prime pages after 30 turns (l2_len: {})",
        chat.state().l2_len()
    );

    let tmp_dir = std::env::temp_dir();
    let save_path = tmp_dir.join(format!(
        "uor_r4_hyperbolic_l2_session_{}.json",
        std::process::id()
    ));

    chat.save_session_default(&save_path)
        .expect("save session with L2 pages to json");
    assert!(save_path.exists());

    let mut restored = bundle
        .load_chat_session(&save_path, "")
        .expect("load hyperbolic chat session with L2 pages");
    let _ = std::fs::remove_file(&save_path);

    // Verify state equality
    assert_eq!(
        restored.state().allow_hyperbolic_cache,
        chat.state().allow_hyperbolic_cache
    );
    assert_eq!(
        restored.state().persistent_len(),
        chat.state().persistent_len()
    );
    assert_eq!(restored.state().dialogue_len(), chat.state().dialogue_len());
    assert_eq!(restored.state().l2_len(), chat.state().l2_len());
    assert_eq!(restored.state().l2_cursor, chat.state().l2_cursor);
    assert_eq!(restored.state().l2_seen, chat.state().l2_seen);

    // Verify key norms bit-identity across persistent, dialogue, and L2 pages
    assert_eq!(
        restored.state().persistent_key_norms,
        chat.state().persistent_key_norms
    );
    for i in 0..chat.state().dialogue_len() {
        assert_eq!(
            restored.state().dialogue_key_norms[i],
            chat.state().dialogue_key_norms[i]
        );
    }
    for i in 0..chat.state().l2_len() {
        assert_eq!(
            restored.state().l2_page_norms[i],
            chat.state().l2_page_norms[i],
            "L2 page norm at {i} must match exactly after deserialization"
        );
    }

    // Verify deterministic continuation
    let test_prompt = "Verify post-L2 restoration deterministic continuation.";
    let mut stream_orig = chat
        .generate_stream(test_prompt, 15, &[])
        .expect("stream orig");
    let mut tokens_orig = Vec::new();
    while let Some(_) = stream_orig.next() {}
    tokens_orig.extend_from_slice(stream_orig.generated_tokens());

    let mut stream_rest = restored
        .generate_stream(test_prompt, 15, &[])
        .expect("stream rest");
    let mut tokens_rest = Vec::new();
    while let Some(_) = stream_rest.next() {}
    tokens_rest.extend_from_slice(stream_rest.generated_tokens());

    assert_eq!(
        tokens_orig, tokens_rest,
        "Original and restored hyperbolic sessions with active L2 pages must produce bit-identical token sequences!"
    );
}

#[test]
fn test_chat_session_hyperbolic_streaming_latency_and_rss_invariants() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    let persona = "Latency bot.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 777).expect("chat session creation");
    chat.set_policy(SamplePolicy::Categorical { top_k: 4 });

    let initial_rss = get_process_rss_mb();

    // Ingest 128 tokens across hyperbolic cache
    let mut pending = Vec::with_capacity(256);
    let mut latencies_ms = Vec::with_capacity(128);

    for i in 0..128 {
        let token = 7 + (i % 250) as u32;
        let start = std::time::Instant::now();
        let _ = chat.step_stream(token, &mut pending).expect("step_stream");
        latencies_ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let final_rss = get_process_rss_mb();
    let mean_latency = latencies_ms.iter().sum::<f64>() / (latencies_ms.len() as f64);

    let initial_rss_str = match initial_rss {
        Some(rss) => format!("{:.2} MB", rss),
        None => "unavailable".to_string(),
    };
    let final_rss_str = match final_rss {
        Some(rss) => format!("{:.2} MB", rss),
        None => "unavailable".to_string(),
    };

    println!(
        "Hyperbolic Cache Telemetry: {} tokens, mean latency: {:.3} ms/token, RSS: {} -> {}",
        latencies_ms.len(),
        mean_latency,
        initial_rss_str,
        final_rss_str,
    );

    assert_eq!(latencies_ms.len(), 128);
    // Wall-clock latency ceilings are recorded, not asserted: a pass/fail
    // assertion on measured latency is flaky under unknown machine load.
    // The RSS/memory-leak checks below remain asserted (functional).
    let ceiling = if cfg!(debug_assertions) { 8.0 } else { 4.0 };
    println!(
        "Latency ceiling (recorded, not asserted): mean {mean_latency:.3} ms vs {ceiling:.1} ms/token ceiling (meets: {})",
        mean_latency <= ceiling
    );

    if let (Some(init), Some(fin)) = (initial_rss, final_rss) {
        let growth = (fin - init).max(0.0);
        assert!(
            growth < 5.0,
            "Hyperbolic cache memory leak detected: {:.2} MB growth across 128 tokens",
            growth
        );
    }
    if let Some(fin) = final_rss {
        assert!(fin > 0.0, "Process RSS must be positive: {fin:.2} MB");
    }
}

#[test]
fn test_invalid_serialized_key_coordinates_rejected() {
    let bundle = Bundle::synthetic_lorentz_for_test();
    let persona = "Guard bot.";
    let mut chat = ChatSession::new(&bundle, Some(persona), 101).expect("chat session creation");

    // Ingest dialogue turns to create persistent and dialogue keys
    for turn in 1..=30 {
        let msg = format!("Turn {turn} message text for key bounds test");
        let mut stream = chat.generate_stream(&msg, 5, &[]).expect("stream for turn");
        while let Some(_) = stream.next() {}
    }
    assert!(chat.state().persistent_len() > 0);
    assert!(chat.state().dialogue_len() > 0);
    assert!(chat.state().l2_len() > 0);

    let valid_serialized = chat.to_serialized(None);

    // 1. Corrupt persistent key coordinate (e.g. 65536 outside [-32767, 32767])
    let mut corrupted_persistent = valid_serialized.clone();
    corrupted_persistent.session_state.persistent_keys[0][0] = 65536;
    let res_persistent = ChatSession::from_serialized(&bundle, corrupted_persistent, "");
    assert!(
        res_persistent.is_err(),
        "must reject out-of-range persistent key coordinate"
    );
    let err_p = res_persistent.err().unwrap().to_string();
    assert!(
        err_p.contains("corrupted persistent key coordinate"),
        "error must identify persistent key coordinate: {err_p}"
    );

    // 2. Corrupt dialogue key coordinate (e.g. -32768 outside [-32767, 32767])
    let mut corrupted_dialogue = valid_serialized.clone();
    corrupted_dialogue.session_state.dialogue_keys[0][0] = -32768;
    let res_dialogue = ChatSession::from_serialized(&bundle, corrupted_dialogue, "");
    assert!(
        res_dialogue.is_err(),
        "must reject out-of-range dialogue key coordinate"
    );
    let err_d = res_dialogue.err().unwrap().to_string();
    assert!(
        err_d.contains("corrupted dialogue key coordinate"),
        "error must identify dialogue key coordinate: {err_d}"
    );

    // 3. Corrupt L2 page key coordinate (e.g. 40000 outside [-32767, 32767])
    let mut corrupted_l2 = valid_serialized.clone();
    corrupted_l2.session_state.l2_pages[0].key[0] = 40000;
    let res_l2 = ChatSession::from_serialized(&bundle, corrupted_l2, "");
    assert!(
        res_l2.is_err(),
        "must reject out-of-range L2 page key coordinate"
    );
    let err_l2 = res_l2.err().unwrap().to_string();
    assert!(
        err_l2.contains("corrupted L2 page key coordinate"),
        "error must identify L2 page key coordinate: {err_l2}"
    );

    // 4. Exact boundary checks: 32767 and -32767 are valid; 32768 is invalid
    let mut boundary_valid = valid_serialized.clone();
    boundary_valid.session_state.persistent_keys[0][0] = 32767;
    boundary_valid.session_state.persistent_keys[0][1] = -32767;
    assert!(
        ChatSession::from_serialized(&bundle, boundary_valid, "").is_ok(),
        "boundary values -32767 and 32767 must be accepted"
    );

    let mut boundary_invalid = valid_serialized.clone();
    boundary_invalid.session_state.persistent_keys[0][0] = 32768;
    assert!(
        ChatSession::from_serialized(&bundle, boundary_invalid, "").is_err(),
        "boundary value 32768 must be rejected"
    );

    // 5. L2 page count exceeds capacity
    let mut corrupted_l2_count = valid_serialized.clone();
    corrupted_l2_count
        .session_state
        .l2_pages
        .resize(65, L2PrimePage::default());
    let res_l2_count = ChatSession::from_serialized(&bundle, corrupted_l2_count, "");
    assert!(res_l2_count.is_err(), "must reject L2 page count > 64");
    assert!(res_l2_count
        .err()
        .unwrap()
        .to_string()
        .contains("l2 page count"));

    // 6. L2 cursor out of bounds
    let mut corrupted_l2_cursor = valid_serialized.clone();
    corrupted_l2_cursor.session_state.l2_cursor = 64;
    let res_l2_cursor = ChatSession::from_serialized(&bundle, corrupted_l2_cursor, "");
    assert!(res_l2_cursor.is_err(), "must reject L2 cursor >= 64");
    assert!(res_l2_cursor
        .err()
        .unwrap()
        .to_string()
        .contains("l2 cursor"));

    // 7. L2 len exceeds capacity
    let mut corrupted_l2_len = valid_serialized.clone();
    corrupted_l2_len.session_state.l2_len = 65;
    let res_l2_len = ChatSession::from_serialized(&bundle, corrupted_l2_len, "");
    assert!(res_l2_len.is_err(), "must reject L2 len > 64");
    assert!(res_l2_len.err().unwrap().to_string().contains("l2 len"));

    // 8. L2 pages count less than l2_len
    let mut corrupted_l2_short = valid_serialized.clone();
    corrupted_l2_short.session_state.l2_len = 10;
    corrupted_l2_short.session_state.l2_pages.truncate(5);
    let res_l2_short = ChatSession::from_serialized(&bundle, corrupted_l2_short, "");
    assert!(res_l2_short.is_err(), "must reject L2 pages count < l2_len");
    assert!(res_l2_short
        .err()
        .unwrap()
        .to_string()
        .contains("less than l2_len"));

    // 9. Dialogue len exceeds capacity
    let mut corrupted_dialogue_len = valid_serialized.clone();
    corrupted_dialogue_len.session_state.dialogue_len = 225;
    let res_dialogue_len = ChatSession::from_serialized(&bundle, corrupted_dialogue_len, "");
    assert!(res_dialogue_len.is_err(), "must reject dialogue len > 224");
    assert!(res_dialogue_len
        .err()
        .unwrap()
        .to_string()
        .contains("dialogue len"));

    // 10. Dialogue tokens count mismatch
    let mut corrupted_dialogue_tokens = valid_serialized.clone();
    corrupted_dialogue_tokens
        .session_state
        .dialogue_tokens
        .truncate(10);
    let res_dt = ChatSession::from_serialized(&bundle, corrupted_dialogue_tokens, "");
    assert!(res_dt.is_err(), "must reject dialogue tokens count != 224");
    assert!(res_dt
        .err()
        .unwrap()
        .to_string()
        .contains("dialogue tokens"));

    // 11. Dialogue sequences count mismatch
    let mut corrupted_dialogue_seq = valid_serialized.clone();
    corrupted_dialogue_seq
        .session_state
        .dialogue_sequences
        .truncate(10);
    let res_ds = ChatSession::from_serialized(&bundle, corrupted_dialogue_seq, "");
    assert!(
        res_ds.is_err(),
        "must reject dialogue sequences count != 224"
    );
    assert!(res_ds
        .err()
        .unwrap()
        .to_string()
        .contains("dialogue sequences"));

    // 12. Dialogue turn IDs count mismatch
    let mut corrupted_dialogue_turn = valid_serialized.clone();
    corrupted_dialogue_turn
        .session_state
        .dialogue_turn_ids
        .truncate(10);
    let res_dturn = ChatSession::from_serialized(&bundle, corrupted_dialogue_turn, "");
    assert!(
        res_dturn.is_err(),
        "must reject dialogue turn IDs count != 224"
    );
    assert!(res_dturn
        .err()
        .unwrap()
        .to_string()
        .contains("dialogue turn IDs"));

    // 13. Persistent values count does not match persistent keys count
    let mut corrupted_pv = valid_serialized.clone();
    corrupted_pv.session_state.persistent_values.truncate(0);
    let res_pv = ChatSession::from_serialized(&bundle, corrupted_pv, "");
    assert!(
        res_pv.is_err(),
        "must reject persistent values count mismatch"
    );
    assert!(res_pv
        .err()
        .unwrap()
        .to_string()
        .contains("persistent values count"));

    // 14. Persistent tokens count does not match persistent keys count
    let mut corrupted_pt = valid_serialized.clone();
    corrupted_pt.session_state.persistent_tokens.truncate(0);
    let res_pt = ChatSession::from_serialized(&bundle, corrupted_pt, "");
    assert!(
        res_pt.is_err(),
        "must reject persistent tokens count mismatch"
    );
    assert!(res_pt
        .err()
        .unwrap()
        .to_string()
        .contains("persistent tokens count"));

    // 15. Persistent capacity mismatch
    let mut corrupted_pcap = valid_serialized.clone();
    corrupted_pcap.session_state.persistent_capacity = 64;
    let res_pcap = ChatSession::from_serialized(&bundle, corrupted_pcap, "");
    assert!(
        res_pcap.is_err(),
        "must reject persistent capacity mismatch"
    );
    assert!(res_pcap
        .err()
        .unwrap()
        .to_string()
        .contains("persistent capacity"));

    // 16. Dialogue capacity mismatch
    let mut corrupted_dcap = valid_serialized;
    corrupted_dcap.session_state.dialogue_capacity = 512;
    let res_dcap = ChatSession::from_serialized(&bundle, corrupted_dcap, "");
    assert!(res_dcap.is_err(), "must reject dialogue capacity mismatch");
    assert!(res_dcap
        .err()
        .unwrap()
        .to_string()
        .contains("dialogue capacity"));
}
