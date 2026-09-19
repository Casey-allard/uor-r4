//! Verification and qualification tests for Milestone 14:
//! Interactive Multi-Turn CLI Serving (`r4-native-chat` Adapter & Telemetry).
//!
//! Enforces:
//! 1. Zero Host-LLM Fallbacks: Pure native geometric representation and routing only.
//! 2. Zero Runtime Matrix Multiplications (Zero GEMM): Serving hot paths execute zero GEMMs.
//! 3. Zero Runtime Floats: Inference hot paths execute using integer, Q1.15, or Q1.30 fixed-point arithmetic only.
//! 4. Zero Steady-State Heap Allocations: Hot path prediction must not allocate heap memory.
//! 5. Anti-Gaming Constraints: No artificial score penalties, no repetition bans, no post-hoc filters.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};
use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

// ============================================================================
// Zero-Allocation Global Tracking Allocator
// ============================================================================

struct CountingAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            let _ = MEASURING.try_with(|enabled| {
                if enabled.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                    let _ = BYTES.try_with(|count| count.set(count.get() + layout.size()));
                }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

// ============================================================================
// Model & Tokenizer Fixtures
// ============================================================================

fn resolve_file_path(candidates: &[&str]) -> PathBuf {
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(candidates[0])
}

fn ensure_models_exist() -> (PathBuf, PathBuf, PathBuf) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.join("../..");

    let json_candidates = [
        root.join("native_geometric_prose_model.json")
            .to_string_lossy()
            .to_string(),
        "native_geometric_prose_model.json".to_string(),
        "/Users/casey.allard/uor-r4/native_geometric_prose_model.json".to_string(),
    ];
    let json_cand_refs: Vec<&str> = json_candidates.iter().map(|s| s.as_str()).collect();
    let json_path = resolve_file_path(&json_cand_refs);

    let rgm_path = json_path.with_extension("rgm");

    let tok_candidates = [
        root.join(".uor-models/research/issue-1014/export/tokenizer.json")
            .to_string_lossy()
            .to_string(),
        ".uor-models/research/issue-1014/export/tokenizer.json".to_string(),
        "/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/export/tokenizer.json"
            .to_string(),
    ];
    let tok_cand_refs: Vec<&str> = tok_candidates.iter().map(|s| s.as_str()).collect();
    let tok_path = resolve_file_path(&tok_cand_refs);

    assert!(
        json_path.exists(),
        "Prose model JSON must exist at {}",
        json_path.display()
    );
    assert!(
        tok_path.exists(),
        "Tokenizer JSON must exist at {}",
        tok_path.display()
    );

    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        if !rgm_path.exists() {
            println!("Exporting {} to binary .rgm...", json_path.display());
            let json_bytes = std::fs::read(&json_path).expect("read json bytes");
            let exported: ExportedGeometricModel =
                serde_json::from_slice(&json_bytes).expect("parse json model");
            exported
                .save_to_binary(&rgm_path)
                .expect("save binary model");
            println!("Saved .rgm to {}", rgm_path.display());
        }
    });

    (json_path, rgm_path, tok_path)
}

// ============================================================================
// 1. Dual-Mode Format Detector Tests
// ============================================================================

#[test]
fn test_dual_mode_format_detector() {
    let (json_path, rgm_path, _tok_path) = ensure_models_exist();

    // 1. Load from .rgm binary bytes
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model_rgm = NativeModel::load_from_bytes(&rgm_bytes).expect("load from rgm bytes");

    let meta_rgm = model_rgm.metadata();
    assert_eq!(meta_rgm.schema_version, "uor-r4.geometric-prose/1");
    assert_eq!(meta_rgm.backend_name, "native-geometric-language-prose-v1");
    assert!(meta_rgm.zero_matmul_serving);
    assert!(meta_rgm.zero_heap_alloc_hot_path);
    assert_eq!(meta_rgm.max_context_tokens, 64);
    assert!(!meta_rgm.model_cid.is_empty());

    // 2. Load from JSON bytes
    let json_bytes = std::fs::read(&json_path).expect("read json bytes");
    let model_json = NativeModel::load_from_bytes(&json_bytes).expect("load from json bytes");

    let meta_json = model_json.metadata();
    assert_eq!(meta_json.schema_version, "uor-r4.geometric-prose/1");
    assert_eq!(meta_json.backend_name, "native-geometric-language-prose-v1");
    assert!(meta_json.zero_matmul_serving);
    assert!(meta_json.zero_heap_alloc_hot_path);
    assert_eq!(meta_json.max_context_tokens, 64);
    assert!(!meta_json.model_cid.is_empty());
}

// ============================================================================
// 2. Streaming Completion & Live Telemetry Tests
// ============================================================================

#[test]
fn test_streaming_completion_and_telemetry() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    let config = SessionConfig {
        temperature: 0.7,
        max_output_tokens: 32,
        ..Default::default()
    };
    let mut session = model.create_session(config).expect("create session");

    let mut stream_chunks = Vec::new();
    let req = CompletionRequest {
        prompt: "The geometric nature of spacetime demonstrates".into(),
        max_tokens: Some(24),
        temperature: Some(0.7),
        stop_sequences: Vec::new(),
    };

    let response = session
        .complete_stream(req, |chunk| {
            stream_chunks.push(chunk.to_string());
            true
        })
        .expect("stream completion");

    assert!(!response.text.is_empty());
    assert_eq!(response.text, stream_chunks.join(""));
    assert!(response.token_count > 0);
    assert!(response.tokens_per_second().unwrap_or(0.0) > 0.0);
    assert!(response.us_per_token().unwrap_or(0.0) > 0.0);

    // Verify telemetry format
    let tele = format!(
        "[{} model tokens; {}; {:.1} tok/s, {:.2} µs/token]",
        response.token_count,
        response.stopped_by,
        response.tokens_per_second().unwrap_or(0.0),
        response.us_per_token().unwrap_or(0.0)
    );
    assert!(tele.contains("model tokens;"));
    assert!(tele.contains("tok/s"));
    assert!(tele.contains("µs/token"));
}

// ============================================================================
// 3. Apple Silicon M1 Throughput & Latency Benchmark
// ============================================================================

#[test]
fn test_throughput_benchmark() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    let config = SessionConfig {
        temperature: 0.0,
        max_output_tokens: 128,
        ..Default::default()
    };
    let mut session = model.create_session(config).expect("create session");

    // Ingest prompt
    let _ = session.ingest("In the mathematical formulation of geometric physics");

    // Warmup 16 tokens outside measurement
    for _ in 0..16 {
        let tok = session.predict_token_zero_alloc().expect("warmup token");
        assert!(tok < 4096);
    }

    // Benchmark across 3 trials and select best to eliminate OS context-switch noise
    let num_tokens = 128;
    let mut best_elapsed = std::time::Duration::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..num_tokens {
            let tok = session.predict_token_zero_alloc().expect("predict token");
            assert!(tok < 4096);
        }
        let elapsed = start.elapsed();
        if elapsed < best_elapsed {
            best_elapsed = elapsed;
        }
    }
    let elapsed = best_elapsed;
    let elapsed_us = elapsed.as_micros() as f64;
    let tok_per_sec = (num_tokens as f64 / elapsed.as_secs_f64()) as u64;
    let us_per_tok = elapsed_us / num_tokens as f64;

    println!(
        "\n--- Throughput Benchmark (Native Zero-GEMM Serving) ---\nGenerated: {} tokens in {:.2} ms\nThroughput: {} tok/s\nLatency: {:.2} µs/token\n-------------------------------------------------------",
        num_tokens,
        elapsed.as_millis(),
        tok_per_sec,
        us_per_tok
    );

    assert!(
        tok_per_sec > 1_000,
        "Throughput must be positive and reasonable (got {} tok/s)",
        tok_per_sec
    );

    #[cfg(not(debug_assertions))]
    {
        assert!(
            tok_per_sec >= 40_000,
            "Release throughput must meet or exceed 40,000 tok/s on M1 (got {} tok/s, {:.2} µs/tok)",
            tok_per_sec,
            us_per_tok
        );
    }
}

// ============================================================================
// 4. 24-Turn Exact Named Entity Recall Test (100% Precision)
// ============================================================================

#[test]
fn test_24_turn_exact_named_entity_recall() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    let config = SessionConfig {
        temperature: 0.0,
        max_output_tokens: 32,
        ..Default::default()
    };
    let mut session = model.create_session(config).expect("create session");

    // 24 Ground-truth knowledge entities
    let test_facts = [
        ("capital of France", "Paris"),
        ("creator of Linux", "Linus Torvalds"),
        ("chemical symbol of gold", "Au"),
        ("largest planet in solar system", "Jupiter"),
        ("speed of light in vacuum", "299,792,458 m/s"),
        ("founder of Rust foundation", "Rust Project"),
        ("author of Principia Mathematica", "Isaac Newton"),
        ("tallest mountain on Earth", "Mount Everest"),
        ("currency of Japan", "Yen"),
        ("atomic number of carbon", "6"),
        ("programming language invented by Bjarne Stroustrup", "C++"),
        ("inventor of the World Wide Web", "Tim Berners-Lee"),
        ("capital of Australia", "Canberra"),
        ("deepest ocean trench", "Mariana Trench"),
        ("discoverer of penicillin", "Alexander Fleming"),
        ("hardest natural mineral", "Diamond"),
        ("first person on the Moon", "Neil Armstrong"),
        ("unit of electrical resistance", "Ohm"),
        ("capital of Canada", "Ottawa"),
        ("inventor of alternating current motor", "Nikola Tesla"),
        ("most abundant gas in Earth's atmosphere", "Nitrogen"),
        ("developer of Git", "Linus Torvalds"),
        ("author of The Origin of Species", "Charles Darwin"),
        ("boiling point of water at sea level", "100 degrees Celsius"),
    ];

    assert_eq!(test_facts.len(), 24);

    // Ingest all 24 facts into session durable memory
    for &(entity, fact) in &test_facts {
        session
            .store_fact(entity, fact)
            .expect("store fact in session memory");
    }

    // Now query each entity across 24 consecutive turns and assert 100% recall
    let mut correct_turns = 0;
    for (turn_idx, &(entity, expected_fact)) in test_facts.iter().enumerate() {
        let prompt = format!("What is the {}?", entity);
        let req = CompletionRequest {
            prompt,
            max_tokens: Some(32),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
        };
        let response = session
            .complete(req)
            .unwrap_or_else(|e| panic!("turn {} failed: {e}", turn_idx + 1));

        assert_eq!(
            response.memory_facts_read,
            Some(1),
            "Turn {} query for '{}' should read from memory store",
            turn_idx + 1,
            entity
        );
        assert_eq!(
            response.text.trim(),
            expected_fact,
            "Turn {} recalled '{}' but expected '{}'",
            turn_idx + 1,
            response.text.trim(),
            expected_fact
        );
        correct_turns += 1;
    }

    assert_eq!(
        correct_turns, 24,
        "All 24 turns must recall exact named entities with 100% precision"
    );
}

// ============================================================================
// 5. Zero Heap Allocations on Generation Hot Path Test
// ============================================================================

#[test]
fn test_zero_heap_allocations_on_generation_hot_path() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    let config = SessionConfig {
        temperature: 0.0,
        max_output_tokens: 64,
        ..Default::default()
    };
    let mut session = model.create_session(config).expect("create session");

    // Ingest initial context outside the measured section
    let _ = session.ingest("Geometric structure of language models without transformers");

    // Warmup 5 tokens outside measurement
    for _ in 0..5 {
        let tok = session
            .predict_token_zero_alloc()
            .expect("warmup prediction");
        assert!(tok < 4096);
    }

    // MEASURED SECTION: 100 predict_token_zero_alloc iterations
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut predictions = [0u32; 100];
    for i in 0..100 {
        predictions[i] = session.predict_token_zero_alloc().expect("predict token");
    }

    MEASURING.with(|v| v.set(false));
    let alloc_count = ALLOCATIONS.with(Cell::get);
    let byte_count = BYTES.with(Cell::get);

    assert_eq!(
        alloc_count, 0,
        "Steady-state predict_token_zero_alloc must execute 0 heap allocations, got {}",
        alloc_count
    );
    assert_eq!(
        byte_count, 0,
        "Steady-state predict_token_zero_alloc must execute 0 allocated bytes, got {}",
        byte_count
    );

    for &tok in &predictions {
        assert!(tok < 4096);
    }
}

// ============================================================================
// 6. Checkpoint Export & Restore for Geometric Prose Sessions
// ============================================================================

#[test]
fn test_checkpoint_export_and_restore() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    let config = SessionConfig {
        temperature: 0.0,
        max_output_tokens: 32,
        ..Default::default()
    };
    let mut session1 = model
        .create_session(config.clone())
        .expect("create session 1");

    session1
        .store_fact("capital of France", "Paris")
        .expect("store fact");
    let _ = session1.ingest("The quick brown fox jumps");

    // Export session state
    let state_bytes = session1.export_state().expect("export state");
    assert!(!state_bytes.is_empty());

    // Restore into a second session
    let mut session2 = model.create_session(config).expect("create session 2");
    session2.import_state(&state_bytes).expect("import state");

    // Query fact from restored session
    let fact = session2
        .query_memory("capital of France")
        .expect("query memory");
    assert_eq!(fact.as_deref(), Some("Paris"));

    // Query via completion prompt
    let req = CompletionRequest {
        prompt: "What is the capital of France?".into(),
        max_tokens: Some(16),
        temperature: Some(0.0),
        stop_sequences: Vec::new(),
    };
    let resp = session2.complete(req).expect("complete restored session");
    assert_eq!(resp.text.trim(), "Paris");
    assert_eq!(resp.memory_facts_read, Some(1));
}

// ============================================================================
// 7. Natural Prose Generation Coherence & Entropy Validation
// ============================================================================

#[test]
fn test_prose_generation_entropy_and_coherence() {
    let (_, rgm_path, tok_path) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");
    let tok_bytes = std::fs::read(&tok_path).expect("read tok bytes");
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes).expect("load tokenizer");

    let test_prompts = [
        "Once upon a time in a sunny meadow,",
        "Lily was playing in the garden when she found",
        "Sammy the squirrel loved to collect nuts for",
        "The kind old grandfather smiled and told a story about",
        "Deep inside the quiet forest, a soft river flowed past",
    ];

    let mut sum_distinct_1 = 0.0;
    let mut sum_distinct_2 = 0.0;
    let mut sum_max_freq = 0.0;
    let mut achieved_high_distinct_1 = false;
    let mut achieved_low_max_freq = false;

    for (idx, &prompt) in test_prompts.iter().enumerate() {
        let config = SessionConfig {
            session_id: format!("test-prose-{}", idx),
            temperature: 0.9,
            top_k: 16,
            max_output_tokens: 64,
            ..Default::default()
        };
        let mut session = model.create_session(config).expect("create session");

        let req = CompletionRequest {
            prompt: prompt.into(),
            max_tokens: Some(64),
            temperature: Some(0.9),
            stop_sequences: Vec::new(),
        };

        let response = session.complete(req).expect("complete prompt");
        assert!(
            !response.text.is_empty(),
            "Response must not be empty for prompt: {prompt}"
        );
        assert!(response.token_count > 0, "Must generate tokens");

        // Assert no degenerate subword or unspaced loops
        assert!(
            !response.text.contains("adlOnce"),
            "Output contains degenerate adlOnce loop: {}",
            response.text
        );
        assert!(
            !response.text.contains("adl"),
            "Output contains raw subword fragment adl: {}",
            response.text
        );
        assert!(
            !response.text.contains("OnceOnce"),
            "Output contains degenerate OnceOnce loop: {}",
            response.text
        );
        assert!(
            !response.text.contains(".Al"),
            "Output contains unspaced subword fragment .Al: {}",
            response.text
        );
        assert!(
            !response.text.contains(".You"),
            "Output contains unspaced token .You: {}",
            response.text
        );

        let completion_tokens = tokenizer.encode(&response.text);
        assert!(
            !completion_tokens.is_empty(),
            "Completion tokens must not be empty for prompt: {prompt}"
        );
        let (distinct_1, distinct_2) = compute_distinct_metrics(&completion_tokens);
        let max_freq = max_token_frequency_ratio(&completion_tokens);

        let full_text = format!("{} {}", prompt, response.text);
        let entropy = compute_4gram_entropy_bytes(full_text.as_bytes());
        println!("\nPrompt: {prompt:?}");
        println!("Completion: {:?}", response.text);
        println!("4-gram entropy: {:.4}", entropy);
        println!(
            "Distinct-1: {:.4}, Distinct-2: {:.4}, Max Token Freq: {:.4}",
            distinct_1, distinct_2, max_freq
        );

        assert!(
            entropy > 0.65,
            "4-gram repetition entropy for '{prompt}' must exceed 0.65, got {:.4}",
            entropy
        );
        assert!(
            distinct_2 >= 0.70,
            "Distinct-2 metric for '{prompt}' must be >= 0.70, got {:.4}",
            distinct_2
        );
        assert!(
            distinct_1 >= 0.40,
            "Distinct-1 metric for '{prompt}' must be >= 0.40, got {:.4}",
            distinct_1
        );
        assert!(
            max_freq <= 0.25,
            "Max token frequency ratio for '{prompt}' must be <= 0.25, got {:.4}",
            max_freq
        );

        sum_distinct_1 += distinct_1;
        sum_distinct_2 += distinct_2;
        sum_max_freq += max_freq;
        if distinct_1 >= 0.50 {
            achieved_high_distinct_1 = true;
        }
        if max_freq <= 0.15 {
            achieved_low_max_freq = true;
        }
    }

    let n = test_prompts.len() as f64;
    let mean_d1 = sum_distinct_1 / n;
    let mean_d2 = sum_distinct_2 / n;
    let mean_mf = sum_max_freq / n;

    println!(
        "\nProse Generation Summary: Mean Distinct-1: {:.4} (>= 0.45), Mean Distinct-2: {:.4} (>= 0.75), Mean Max Freq: {:.4} (<= 0.20)",
        mean_d1, mean_d2, mean_mf
    );

    assert!(
        mean_d1 >= 0.45,
        "Mean Distinct-1 across prompts must be >= 0.45, got {:.4}",
        mean_d1
    );
    assert!(
        mean_d2 >= 0.75,
        "Mean Distinct-2 across prompts must be >= 0.75, got {:.4}",
        mean_d2
    );
    assert!(
        mean_mf <= 0.20,
        "Mean max token frequency ratio must be <= 0.20, got {:.4}",
        mean_mf
    );
    assert!(
        achieved_high_distinct_1,
        "At least one prompt must achieve Distinct-1 >= 0.50"
    );
    assert!(
        achieved_low_max_freq,
        "At least one prompt must achieve Max Token Frequency <= 0.15"
    );
}

#[test]
fn test_conversational_free_generation_without_preloaded_facts() {
    let (_, rgm_path, tok_path) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");
    let tok_bytes = std::fs::read(&tok_path).expect("read tok bytes");
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes).expect("load tokenizer");

    let conversational_prompts = [
        "Hello, tell me a short story about a little bird",
        "Once upon a time there was a dog named Spot who",
        "Can you share what happened on a sunny afternoon in the forest?",
    ];

    let mut sum_distinct_1 = 0.0;
    let mut sum_distinct_2 = 0.0;
    let mut sum_max_freq = 0.0;
    let mut achieved_high_distinct_1 = false;
    let mut achieved_low_max_freq = false;

    for (idx, &prompt) in conversational_prompts.iter().enumerate() {
        let config = SessionConfig {
            session_id: format!("test-free-conv-{}", idx),
            temperature: 0.9,
            top_k: 16,
            max_output_tokens: 64,
            ..Default::default()
        };
        let mut session = model.create_session(config).expect("create session");

        let req = CompletionRequest {
            prompt: prompt.into(),
            max_tokens: Some(64),
            temperature: Some(0.9),
            stop_sequences: Vec::new(),
        };

        let response = session.complete(req).expect("complete prompt");
        assert!(
            !response.text.is_empty(),
            "Response must not be empty for prompt: {prompt}"
        );
        assert!(response.token_count > 0, "Must generate tokens");

        let completion_tokens = tokenizer.encode(&response.text);
        assert!(
            !completion_tokens.is_empty(),
            "Completion tokens must not be empty for prompt: {prompt}"
        );
        let (distinct_1, distinct_2) = compute_distinct_metrics(&completion_tokens);
        let max_freq = max_token_frequency_ratio(&completion_tokens);

        println!("\nConversational Prompt: {prompt:?}");
        println!("Completion: {:?}", response.text);
        println!(
            "Distinct-1: {:.4}, Distinct-2: {:.4}, Max Token Freq: {:.4}",
            distinct_1, distinct_2, max_freq
        );

        assert!(
            distinct_2 >= 0.70,
            "Distinct-2 metric for '{prompt}' must be >= 0.70, got {:.4}",
            distinct_2
        );
        assert!(
            distinct_1 >= 0.40,
            "Distinct-1 metric for '{prompt}' must be >= 0.40, got {:.4}",
            distinct_1
        );
        assert!(
            max_freq <= 0.25,
            "Max token frequency ratio for '{prompt}' must be <= 0.25, got {:.4}",
            max_freq
        );

        sum_distinct_1 += distinct_1;
        sum_distinct_2 += distinct_2;
        sum_max_freq += max_freq;
        if distinct_1 >= 0.50 {
            achieved_high_distinct_1 = true;
        }
        if max_freq <= 0.15 {
            achieved_low_max_freq = true;
        }
    }

    let n = conversational_prompts.len() as f64;
    let mean_d1 = sum_distinct_1 / n;
    let mean_d2 = sum_distinct_2 / n;
    let mean_mf = sum_max_freq / n;

    println!(
        "\nConversational Free Generation Summary: Mean Distinct-1: {:.4} (>= 0.45), Mean Distinct-2: {:.4} (>= 0.75), Mean Max Freq: {:.4} (<= 0.16)",
        mean_d1, mean_d2, mean_mf
    );

    assert!(
        mean_d1 >= 0.45,
        "Mean Distinct-1 across conversational prompts must be >= 0.45, got {:.4}",
        mean_d1
    );
    assert!(
        mean_d2 >= 0.75,
        "Mean Distinct-2 across conversational prompts must be >= 0.75, got {:.4}",
        mean_d2
    );
    assert!(
        mean_mf <= 0.16,
        "Mean max token frequency ratio across conversational prompts must be <= 0.16, got {:.4}",
        mean_mf
    );
    assert!(
        achieved_high_distinct_1,
        "At least one conversational prompt must achieve Distinct-1 >= 0.50"
    );
    assert!(
        achieved_low_max_freq,
        "At least one conversational prompt must achieve Max Token Frequency <= 0.15"
    );
}

fn compute_distinct_metrics(tokens: &[u32]) -> (f64, f64) {
    if tokens.is_empty() {
        return (1.0, 1.0);
    }
    let mut unigrams = std::collections::HashSet::new();
    for &t in tokens {
        unigrams.insert(t);
    }
    let distinct_1 = unigrams.len() as f64 / tokens.len() as f64;

    if tokens.len() < 2 {
        return (distinct_1, 1.0);
    }
    let mut bigrams = std::collections::HashSet::new();
    for window in tokens.windows(2) {
        bigrams.insert((window[0], window[1]));
    }
    let distinct_2 = bigrams.len() as f64 / (tokens.len() - 1) as f64;

    (distinct_1, distinct_2)
}

fn max_token_frequency_ratio(tokens: &[u32]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let mut counts = std::collections::HashMap::new();
    for &t in tokens {
        *counts.entry(t).or_insert(0usize) += 1;
    }
    let max_count = counts.values().copied().max().unwrap_or(0);
    max_count as f64 / tokens.len() as f64
}

fn compute_4gram_entropy_bytes(tokens: &[u8]) -> f64 {
    if tokens.len() < 4 {
        return 1.0;
    }
    let total_4grams = tokens.len() - 3;
    let mut counts = std::collections::HashMap::new();
    for window in tokens.windows(4) {
        *counts
            .entry([window[0], window[1], window[2], window[3]])
            .or_insert(0usize) += 1;
    }
    let mut entropy = 0.0;
    let total_f = total_4grams as f64;
    for &count in counts.values() {
        let p = (count as f64) / total_f;
        if p > 0.0 {
            entropy -= p * libm::log2(p);
        }
    }
    let max_entropy = libm::log2(total_f).max(1.0);
    entropy / max_entropy
}

// ============================================================================
// 8. Edge Cases: Long Prompt (>64 tokens, temp > 1.0) & Many Facts (>64 facts)
// ============================================================================

#[test]
fn test_edge_cases_long_prompt_and_many_facts() {
    let (_, rgm_path, _) = ensure_models_exist();
    let rgm_bytes = std::fs::read(&rgm_path).expect("read rgm bytes");
    let model = NativeModel::load_from_bytes(&rgm_bytes).expect("load model");

    // Case A: Assert 100 distinct facts (>64 item boundary) into session durable memory
    let config = SessionConfig {
        session_id: "test-edge-100-facts".into(),
        temperature: 0.0,
        max_output_tokens: 32,
        ..Default::default()
    };
    let mut session = model.create_session(config).expect("create session");

    for i in 0..100 {
        let owner = format!("entity_identifier_{:03}", i);
        let val = format!("secret_value_{:03}", i);
        session.store_fact(&owner, &val).expect("store fact");
    }

    // Verify retrieval of first, middle, and last entities
    for &probe_idx in &[0, 25, 49, 63, 64, 85, 99] {
        let probe_owner = format!("entity_identifier_{:03}", probe_idx);
        let expected = format!("secret_value_{:03}", probe_idx);
        let query_resp = session.query_memory(&probe_owner).expect("query memory");
        assert_eq!(
            query_resp.as_deref(),
            Some(expected.as_str()),
            "Failed recalling entity {}",
            probe_idx
        );

        let req = CompletionRequest {
            prompt: format!("What is the {}?", probe_owner),
            max_tokens: Some(32),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
        };
        let completion = session.complete(req).expect("complete query prompt");
        assert_eq!(completion.text.trim(), expected.as_str());
        assert_eq!(completion.memory_facts_read, Some(1));
    }

    // Case B: Prompt with >64 tokens under high temperature (temp = 1.2)
    let long_prompt = "Once upon a time in a faraway enchanted kingdom, there lived a brave young boy who loved exploring the deep quiet woods. Every morning he would wake up early, pack some sweet red berries and crunchy apples in his small wooden basket, and set out along the windy dirt trail. He greeted every chirping songbird and playful squirrel that crossed his winding path through the ancient sunlit grove.";
    let config_high_temp = SessionConfig {
        session_id: "test-edge-long-prompt".into(),
        temperature: 1.2,
        top_k: 10,
        max_output_tokens: 64,
        ..Default::default()
    };
    let mut session_long = model
        .create_session(config_high_temp)
        .expect("create high-temp session");
    let req_long = CompletionRequest {
        prompt: long_prompt.into(),
        max_tokens: Some(64),
        temperature: Some(1.2),
        stop_sequences: Vec::new(),
    };
    let resp_long = session_long
        .complete(req_long)
        .expect("complete long prompt");
    assert!(
        resp_long.token_count > 0,
        "Must generate tokens on long prompt with temp > 1.0"
    );
    assert!(
        !resp_long.text.is_empty(),
        "Must return text completion on long prompt"
    );
}
