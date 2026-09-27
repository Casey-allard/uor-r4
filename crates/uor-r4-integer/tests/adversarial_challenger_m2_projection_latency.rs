//! Empirical Challenger Test Suite for Milestone M2:
//! Zero-MatMul 4K Vocab Un-Embedding Numerical Equivalence & Latency Benchmarks.
//!
//! File: crates/uor-r4-integer/tests/adversarial_challenger_m2_projection_latency.rs

use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::format::load_hard_codes;
use uor_r4_integer::math::scale_pow2;
use uor_r4_integer::model::SlotTarget;
use uor_r4_integer::session::ChatSession;
use uor_r4_integer::ReadMode;

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";
const REAL_HOUSEHOLDER_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";

const WORK_BITS: i32 = 40;

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Golden scalar reference implementation for project_vocab.
/// Uses pure scalar i64 multiply-accumulate and checked math, with no table-unrolling or low_bit_dot_4x.
fn golden_project_vocab(
    hidden: &[i32],
    codes: &[i16],
    embedding_row_exponents: &[i16],
    bias_codes: &[i16],
    bias_exponent: i16,
) -> Vec<i32> {
    let vocab_size = embedding_row_exponents.len();
    assert_eq!(vocab_size, 4096);
    assert_eq!(codes.len(), 4096 * 256);
    assert_eq!(hidden.len(), 256);

    let bias_shift = WORK_BITS + i32::from(bias_exponent);
    let mut golden_logits = Vec::with_capacity(vocab_size);

    for r in 0..vocab_size {
        // Pure scalar reference dot product
        let mut dot = 0i64;
        let row_offset = r * 256;
        for i in 0..256 {
            let h = i64::from(hidden[i]);
            let w = i64::from(codes[row_offset + i]);
            dot += h * w;
        }

        let shift = WORK_BITS - 10 + i32::from(embedding_row_exponents[r]);
        let val = scale_pow2(i128::from(dot), shift).expect("golden scale val");
        let bias = scale_pow2(i128::from(bias_codes[r]), bias_shift).expect("golden scale bias");
        let rounded = scale_pow2(val + bias, 8 - WORK_BITS).expect("golden quantize");
        let logit = rounded.clamp(-32767, 32767) as i32;
        golden_logits.push(logit);
    }
    golden_logits
}

/// Simple deterministic PRNG for generating reproducible test vectors.
struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_i32_range(&mut self, min: i32, max: i32) -> i32 {
        let span = (max as i64 - min as i64 + 1) as u64;
        let val = (self.next_u32() as u64) % span;
        (min as i64 + val as i64) as i32
    }
}

#[test]
fn test_challenger_m2_numerical_equivalence_random_and_extreme_inputs() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    let model_dir = bundle_path.join("model");
    let (spec, codes) = load_hard_codes(&model_dir).expect("load hard codes");

    let emb_spec = &spec.parameters["embedding.weight"];
    let emb_codes = &codes["embedding.weight"];
    let bias_spec = &spec.parameters["output.bias"];
    let bias_codes = &codes["output.bias"];

    let bias_exp = bias_spec.row_exponents[0];

    // 1. Extreme inputs
    let mut test_vectors: Vec<(&str, Vec<i32>)> = vec![
        ("all_zeros", vec![0i32; 256]),
        ("all_ones", vec![1i32; 256]),
        ("all_minus_ones", vec![-1i32; 256]),
        ("all_32767", vec![32767i32; 256]),
        ("all_minus_32767", vec![-32767i32; 256]),
    ];

    let alternating: Vec<i32> = (0..256)
        .map(|i| if i % 2 == 0 { 1000 } else { -1000 })
        .collect();
    test_vectors.push(("alternating_1000", alternating));

    let sparse: Vec<i32> = (0..256)
        .map(|i| if i == 42 || i == 128 { 5000 } else { 0 })
        .collect();
    test_vectors.push(("sparse_impulses", sparse));

    // 2. 50 Pseudo-random test vectors
    let mut lcg = SimpleLcg::new(9876543210);
    for _ in 0..50 {
        let rand_vec: Vec<i32> = (0..256)
            .map(|_| lcg.next_i32_range(-32767, 32767))
            .collect();
        test_vectors.push(("random_vector", rand_vec));
    }

    println!(
        "Testing numerical equivalence across {} test vectors against golden scalar reference...",
        test_vectors.len()
    );

    let mut total_logits_tested = 0usize;
    let mut max_abs_diff = 0i64;

    for (name, hidden) in &test_vectors {
        let actual_logits = model.project_vocab(hidden).expect("project_vocab failed");
        let golden_logits = golden_project_vocab(
            hidden,
            emb_codes,
            &emb_spec.row_exponents,
            bias_codes,
            bias_exp,
        );

        assert_eq!(
            actual_logits.len(),
            golden_logits.len(),
            "Logit count mismatch for {}",
            name
        );
        assert_eq!(actual_logits.len(), 4096);

        for (r, (&act, &gold)) in actual_logits.iter().zip(&golden_logits).enumerate() {
            total_logits_tested += 1;
            let diff = (i64::from(act) - i64::from(gold)).abs();
            if diff > max_abs_diff {
                max_abs_diff = diff;
            }
            assert_eq!(
                act, gold,
                "Numerical discrepancy at vector '{}', row {}: actual = {}, golden = {}",
                name, r, act, gold
            );
        }
    }

    println!(
        "Numerical equivalence PASS: verified {} logits across {} vectors. Max abs diff = {}",
        total_logits_tested,
        test_vectors.len(),
        max_abs_diff
    );
    assert_eq!(
        max_abs_diff, 0,
        "Max abs diff must be 0 for exact equivalence"
    );
}

#[test]
fn test_challenger_m2_numerical_equivalence_real_model_hidden_states() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    let model_dir = bundle_path.join("model");
    let (spec, codes) = load_hard_codes(&model_dir).expect("load hard codes");

    let emb_spec = &spec.parameters["embedding.weight"];
    let emb_codes = &codes["embedding.weight"];
    let bias_spec = &spec.parameters["output.bias"];
    let bias_codes = &codes["output.bias"];
    let bias_exp = bias_spec.row_exponents[0];

    let mut session = model.new_conversational_session();

    println!(
        "Stepping model through real conversation tokens and checking project_vocab equivalence..."
    );

    // Step 50 real tokens into session
    let mut total_real_logits_tested = 0usize;
    for i in 0..50 {
        let token = 100 + (i as u32);
        let _ = model
            .step_conversational(&mut session, token, SlotTarget::Dialogue, ReadMode::Enabled)
            .expect("step_conversational");

        // The current recurrent state in session.state represents real live model states
        let state_vec = &session.state;
        assert_eq!(state_vec.len(), 256);

        let actual = model.project_vocab(state_vec).expect("project_vocab");
        let golden = golden_project_vocab(
            state_vec,
            emb_codes,
            &emb_spec.row_exponents,
            bias_codes,
            bias_exp,
        );

        for (r, (&act, &gold)) in actual.iter().zip(&golden).enumerate() {
            total_real_logits_tested += 1;
            assert_eq!(
                act, gold,
                "Real model state discrepancy at step {}, row {}: actual={}, golden={}",
                i, r, act, gold
            );
        }
    }

    println!(
        "Real model hidden state equivalence PASS: {} logits verified across 50 dialogue steps",
        total_real_logits_tested
    );
}

#[test]
fn test_challenger_m2_unembedding_latency_100_plus_tokens() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    let hidden = vec![123i32; model.config().width];

    // Warmup 25 passes
    for _ in 0..25 {
        let _ = model.project_vocab(&hidden).unwrap();
    }

    let iters = 200;
    let mut latencies_us = Vec::with_capacity(iters);

    for _ in 0..iters {
        let t0 = Instant::now();
        let _ = model.project_vocab(&hidden).unwrap();
        latencies_us.push(t0.elapsed().as_micros() as f64);
    }

    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let avg_ms = (latencies_us.iter().sum::<f64>() / iters as f64) / 1000.0;
    let min_ms = latencies_us[0] / 1000.0;
    let max_ms = latencies_us[iters - 1] / 1000.0;
    let p50_ms = latencies_us[iters / 2] / 1000.0;
    let p90_ms = latencies_us[(iters as f64 * 0.90) as usize] / 1000.0;
    let p95_ms = latencies_us[(iters as f64 * 0.95) as usize] / 1000.0;
    let p99_ms = latencies_us[(iters as f64 * 0.99) as usize] / 1000.0;

    println!(
        "Challenger project_vocab latency across {} calls: min={:.3} ms, p50={:.3} ms, avg={:.3} ms, p90={:.3} ms, p95={:.3} ms, p99={:.3} ms, max={:.3} ms",
        iters, min_ms, p50_ms, avg_ms, p90_ms, p95_ms, p99_ms, max_ms
    );

    // Objective: does un-embedding projection latency stay well below 1.0 ms?
    println!(
        "Un-embedding projection latency assessment: avg = {:.3} ms (target: well below 1.0 ms)",
        avg_ms
    );
}

#[test]
fn test_challenger_m2_live_4k_streaming_generation_latency() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    for (name, bundle_path_str) in [
        ("quaternion", REAL_QUATERNION_BUNDLE),
        ("householder_pair", REAL_HOUSEHOLDER_BUNDLE),
    ] {
        let bundle_path = Path::new(bundle_path_str);
        let bundle = Bundle::load(bundle_path).expect("load bundle");
        let mut session = ChatSession::new(&bundle, None, 42).expect("chat session");

        let prompt = "Explain the geometric structure of Riemannian manifolds.";
        session.ingest_user_turn(prompt).expect("ingest_user_turn");

        let mut pending = Vec::new();
        // Warmup 5 tokens
        for i in 0..5 {
            session
                .step_stream(10 + (i as u32), &mut pending)
                .expect("warmup");
        }
        pending.clear();

        let num_tokens = 100;
        let mut latencies_us = Vec::with_capacity(num_tokens);
        let mut raw_latencies = Vec::with_capacity(num_tokens);

        for i in 0..num_tokens {
            let token = 50 + (i as u32);
            let t0 = Instant::now();
            session
                .step_stream(token, &mut pending)
                .expect("step_stream");
            let el = t0.elapsed();
            let us = el.as_micros() as f64;
            latencies_us.push(us);
            raw_latencies.push(us / 1000.0);
        }

        // Print deciles
        for d in 0..10 {
            let start = d * 10;
            let end = start + 10;
            let decile_avg: f64 = raw_latencies[start..end].iter().sum::<f64>() / 10.0;
            println!(
                "  [Decile {}-{}] avg latency = {:.3} ms/token",
                start, end, decile_avg
            );
        }

        latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_ms = (latencies_us.iter().sum::<f64>() / num_tokens as f64) / 1000.0;
        let min_ms = latencies_us[0] / 1000.0;
        let max_ms = latencies_us[num_tokens - 1] / 1000.0;
        let p50_ms = latencies_us[num_tokens / 2] / 1000.0;
        let p90_ms = latencies_us[(num_tokens as f64 * 0.90) as usize] / 1000.0;
        let p95_ms = latencies_us[(num_tokens as f64 * 0.95) as usize] / 1000.0;
        let p99_ms = latencies_us[(num_tokens as f64 * 0.99) as usize] / 1000.0;

        println!(
            "[{}] Live 4K generation latency across {} tokens: min={:.3} ms, p50={:.3} ms, avg={:.3} ms, p90={:.3} ms, p95={:.3} ms, p99={:.3} ms, max={:.3} ms",
            name, num_tokens, min_ms, p50_ms, avg_ms, p90_ms, p95_ms, p99_ms, max_ms
        );
    }
}

#[test]
fn test_challenger_m2_memory_scaling_breakdown() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    // 1. Benchmark step_conversational with ReadMode::Disabled (pure recurrent + project_vocab)
    let mut session_noread = model.new_conversational_session();
    let iters = 100;
    let t0 = Instant::now();
    for i in 0..iters {
        let _ = model
            .step_conversational(
                &mut session_noread,
                (10 + i) as u32,
                SlotTarget::Dialogue,
                ReadMode::NoRead,
            )
            .unwrap();
    }
    let noread_avg_ms = t0.elapsed().as_secs_f64() * 1000.0 / iters as f64;
    println!(
        "step_conversational (ReadMode::NoRead, 0 memory overhead): {:.3} ms/step",
        noread_avg_ms
    );

    // 2. Benchmark step_conversational with ReadMode::Enabled at slot milestones
    for slots in [0, 20, 50, 100, 200] {
        let mut session = model.new_conversational_session();
        for i in 0..slots {
            let _ = model
                .step_conversational(
                    &mut session,
                    (10 + i) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap();
        }
        let t_slots = Instant::now();
        let eval_iters = 30;
        for i in 0..eval_iters {
            let _ = model
                .step_conversational(
                    &mut session,
                    (1000 + i) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap();
        }
        let slot_avg_ms = t_slots.elapsed().as_secs_f64() * 1000.0 / eval_iters as f64;
        println!(
            "step_conversational (ReadMode::Enabled with {} dialogue slots): {:.3} ms/step",
            slots, slot_avg_ms
        );
    }
}
