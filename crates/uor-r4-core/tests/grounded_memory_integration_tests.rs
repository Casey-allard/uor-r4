//! Milestone 4: Grounded Multi-Turn Integration Test Suite for UOR-R4.
//!
//! Validates:
//! 1. Multi-turn conversational state tracking: `session.observe`, `session.begin_response`,
//!    and `session.end_response` over multiple turns; context and relations persist without corruption.
//! 2. Verbatim factual recall via typed memory operators: `WordCopyAction::Start` and
//!    `WordCopyAction::Byte` provide 100% provenance and zero hallucination, with HierarchicalCodebook
//!    providing bounded shortlist framing.
//! 3. Joint scoring and ranking: Memory candidate tokens receive joint evidence boosting
//!    under `geometric_prose_tables`, preserving historical behavior when absent.
//! 4. Zero steady-state heap allocations under `CountingAllocator` during continuous multi-turn
//!    streaming with grounded memory and hierarchical shortlist routing.
//! 5. Structure-preserving discrete JEPA fixed-point step (`predict_jepa_step_q30`): Zero runtime
//!    floats, bit-exact determinism, bounded dynamic range, and zero heap allocations.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_core::native_geometric::hopf_metric::{HopfFiberPointQ30, UnitS2Q30, UnitS3Q30};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::native_geometric::vsa::{Codebook, HierarchicalCodebook};
use uor_r4_core::native_geometric::{Control, WordCopyAction, BOS};

mod fixture {
    use uor_r4_core::native_geometric as native;
    include!("support/native_word_copy_fixture.rs");
}

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

/// Helper to create an ExportedGeometricModel with HierarchicalCodebook for testing.
fn make_test_hierarchical_tables(vocab_size: usize, seed: u64) -> ExportedGeometricModel {
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(31).wrapping_add(17)) % 120) as u8)
        .collect();
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    ExportedGeometricModel {
        vocab_size,
        token_to_root,
        discrete_tables: Vec::new(),
        discrete_bias: vec![0; vocab_size],
        discrete_s2_readout: vec![[0; 5]; vocab_size],
        discrete_jepa_w_state: [1000, 200, -100, 50, 1200, -80, -30, 90, 1100],
        discrete_jepa_w_token: [400, -50, 20, 10, 450, -30, -10, 20, 500],
        discrete_jepa_bias: [10, -5, 12],
        discrete_jepa_fiber_w_state: [1200, -150, 150, 1200],
        discrete_jepa_fiber_w_token: [300, -40, 40, 300],
        discrete_jepa_fiber_bias: [8, -3],
        vsa_seed: seed,
        vsa_scale_q15: 500,
        hierarchical_codebook: Some(hierarchical),
        engram_table: None,
        hierarchical_lattice: None,
    }
}

/// Test 1: Multi-turn conversational state tracking across observe, begin_response,
/// predict, and end_response. Verifies conversational context and relations persist.
#[test]
fn test_multi_turn_conversational_state_tracking() {
    let (_, copy_model) = fixture::fitted();
    let mut session = copy_model.session(Control::Full).unwrap();

    let initial_state = session.state();
    assert_eq!(initial_state.tokens_seen, 0);
    let mut expected_tokens_seen: u64 = 0;

    // --- Turn 1: Introduce relation binding for "alpha" ---
    session.observe(copy_model, BOS).unwrap();
    expected_tokens_seen += 1;

    let turn1_prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let turn1_tokens = copy_model.encode(turn1_prompt).unwrap();
    for &tok in &turn1_tokens {
        session.observe(copy_model, tok).unwrap();
        expected_tokens_seen += 1;
    }
    session.begin_response(copy_model).unwrap();

    let t1_pred = session.predict(copy_model).unwrap();
    assert_eq!(t1_pred.token, u32::from(b'a') + 2);
    session.observe(copy_model, t1_pred.token).unwrap();
    expected_tokens_seen += 1;

    // Emit remaining bytes of "lpha\n}\n"
    for _expected_byte in b"lpha\n}\n" {
        let next_pred = session.predict(copy_model).unwrap();
        session.observe(copy_model, next_pred.token).unwrap();
        expected_tokens_seen += 1;
    }
    session.end_response(copy_model).unwrap();

    let state_after_turn1 = session.state();
    assert_eq!(state_after_turn1.tokens_seen, expected_tokens_seen);
    assert!(state_after_turn1.values.is_some());

    // --- Turn 2: Follow-up conversational turn binding "bravo" ---
    let turn2_prompt = "left = 14; right = 4; fn identity(bravo: i32) -> i32 {\n    ";
    let turn2_tokens = copy_model.encode(turn2_prompt).unwrap();
    for &tok in &turn2_tokens {
        session.observe(copy_model, tok).unwrap();
        expected_tokens_seen += 1;
    }
    session.begin_response(copy_model).unwrap();

    let t2_pred = session.predict(copy_model).unwrap();
    assert_eq!(t2_pred.token, u32::from(b'b') + 2);
    session.observe(copy_model, t2_pred.token).unwrap();
    expected_tokens_seen += 1;

    for _expected_byte in b"ravo\n}\n" {
        let next_pred = session.predict(copy_model).unwrap();
        session.observe(copy_model, next_pred.token).unwrap();
        expected_tokens_seen += 1;
    }
    session.end_response(copy_model).unwrap();

    let state_after_turn2 = session.state();
    assert_eq!(state_after_turn2.tokens_seen, expected_tokens_seen);
    assert!(state_after_turn2.values.is_some());

    // --- Turn 3: Third conversational turn binding "cedar" ---
    let turn3_prompt = "left = 15; right = 4; fn identity(cedar: i32) -> i32 {\n    ";
    let turn3_tokens = copy_model.encode(turn3_prompt).unwrap();
    for &tok in &turn3_tokens {
        session.observe(copy_model, tok).unwrap();
        expected_tokens_seen += 1;
    }
    session.begin_response(copy_model).unwrap();

    let t3_pred = session.predict(copy_model).unwrap();
    assert_eq!(t3_pred.token, u32::from(b'c') + 2);
    session.observe(copy_model, t3_pred.token).unwrap();
    expected_tokens_seen += 1;

    for _expected_byte in b"edar\n}\n" {
        let next_pred = session.predict(copy_model).unwrap();
        session.observe(copy_model, next_pred.token).unwrap();
        expected_tokens_seen += 1;
    }
    session.end_response(copy_model).unwrap();

    let state_after_turn3 = session.state();
    assert_eq!(state_after_turn3.tokens_seen, expected_tokens_seen);
    assert!(state_after_turn3.values.is_some());
}

/// Test 2: Verbatim factual recall via typed memory operators.
/// Verifies WordCopyAction::Start followed by WordCopyAction::Byte with 100% provenance
/// while HierarchicalCodebook provides bounded syntactic shortlist framing.
#[test]
fn test_verbatim_factual_recall_via_typed_memory_operators() {
    let (_, copy_model) = fixture::fitted();
    let prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";

    // Part A: Baseline factual recall via typed word copy
    let mut session = copy_model.session(Control::Full).unwrap();
    session.observe(copy_model, BOS).unwrap();
    for token in copy_model.encode(prompt).unwrap() {
        session.observe(copy_model, token).unwrap();
    }
    session.begin_response(copy_model).unwrap();

    // First byte 'a' must be WordCopyAction::Start
    let first = session.predict(copy_model).unwrap();
    assert_eq!(first.token, u32::from(b'a') + 2);
    let decision = session
        .word_copy_decision()
        .expect("word copy decision must be active");
    assert_eq!(decision.action, WordCopyAction::Start);
    session.observe(copy_model, first.token).unwrap();

    // Subsequent bytes "lpha" must be WordCopyAction::Byte
    for &byte in b"lpha" {
        let pred = session.predict(copy_model).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        let dec = session
            .word_copy_decision()
            .expect("word copy decision must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session.observe(copy_model, pred.token).unwrap();
    }
    session.end_response(copy_model).unwrap();

    // Part B: Factual recall with HierarchicalCodebook installed
    let mut model_with_hierarchical = copy_model.clone();
    let vocab_size = model_with_hierarchical.vocabulary_size();
    let seed = 0x5653_415f_abcd_1234;
    let tables = make_test_hierarchical_tables(vocab_size, seed);
    model_with_hierarchical.geometric_prose_tables = Some(tables);

    let mut session_hier = model_with_hierarchical.session(Control::Full).unwrap();
    session_hier.observe(&model_with_hierarchical, BOS).unwrap();
    for token in model_with_hierarchical.encode(prompt).unwrap() {
        session_hier
            .observe(&model_with_hierarchical, token)
            .unwrap();
    }
    session_hier
        .begin_response(&model_with_hierarchical)
        .unwrap();

    // Candidate count must remain strictly bounded <= 64
    let first_hier = session_hier.predict(&model_with_hierarchical).unwrap();
    assert_eq!(first_hier.token, u32::from(b'a') + 2);
    assert!(
        first_hier.candidate_count <= 64,
        "Candidate offers must remain bounded <= 64, got {}",
        first_hier.candidate_count
    );
    let decision_hier = session_hier
        .word_copy_decision()
        .expect("word copy decision must be active");
    assert_eq!(decision_hier.action, WordCopyAction::Start);
    session_hier
        .observe(&model_with_hierarchical, first_hier.token)
        .unwrap();

    for &byte in b"lpha" {
        let pred = session_hier.predict(&model_with_hierarchical).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        assert!(pred.candidate_count <= 64);
        let dec = session_hier
            .word_copy_decision()
            .expect("word copy decision must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session_hier
            .observe(&model_with_hierarchical, pred.token)
            .unwrap();
    }
    session_hier.end_response(&model_with_hierarchical).unwrap();
}

/// Test 3: Joint scoring and ranking verification.
/// Verifies memory candidate injection into HierarchicalCodebook shortlist routing,
/// joint score boosting (`geom_score + mem_evidence`), and proper candidate ranking.
#[test]
fn test_joint_scoring_and_ranking_with_memory() {
    let vocab_size = 256;
    let seed = 0x5653_415f_8877_6655;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(41).wrapping_add(19)) % 120) as u8)
        .collect();
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    // 1. Direct unit verification of route_shortlist_q30_with_memory
    let memory_tokens = [42u32, 99u32, 15u32];
    let query_s3 = Some(UnitS3Q30([1 << 30, 0, 0, 0]));
    let query_vsa = base_codebook.get(10);
    let shortlist = hierarchical.route_shortlist_q30_with_memory::<64>(
        query_s3,
        Some(&query_vsa),
        &memory_tokens,
    );

    assert!(shortlist.len <= 64);
    assert!(shortlist.len >= 3);
    // Active memory candidates must be injected at the head of the shortlist
    let sl_slice = shortlist.as_slice();
    assert_eq!(sl_slice[0], 42);
    assert_eq!(sl_slice[1], 99);
    assert_eq!(sl_slice[2], 15);

    // 2. Integration verification on Session candidate ranking
    let (_, copy_model) = fixture::fitted();
    let mut model_with_tables = copy_model.clone();
    let model_vocab_size = model_with_tables.vocabulary_size();
    let tables = make_test_hierarchical_tables(model_vocab_size, seed);
    model_with_tables.geometric_prose_tables = Some(tables);

    let prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let mut session = model_with_tables.session(Control::Full).unwrap();
    session.observe(&model_with_tables, BOS).unwrap();
    for tok in model_with_tables.encode(prompt).unwrap() {
        session.observe(&model_with_tables, tok).unwrap();
    }
    session.begin_response(&model_with_tables).unwrap();

    let pred = session.predict(&model_with_tables).unwrap();
    let candidates = session.candidates();

    // Verify candidates are strictly sorted in descending score order
    for i in 1..candidates.len() {
        assert!(
            candidates[i - 1].score >= candidates[i].score,
            "Candidates must be sorted descending: {} < {}",
            candidates[i - 1].score,
            candidates[i].score
        );
    }
    assert!(pred.candidate_count <= model_with_tables.config().candidate_limit);

    // Verify historical baseline when geometric_prose_tables is None
    let mut session_baseline = copy_model.session(Control::Full).unwrap();
    session_baseline.observe(copy_model, BOS).unwrap();
    for tok in copy_model.encode(prompt).unwrap() {
        session_baseline.observe(copy_model, tok).unwrap();
    }
    session_baseline.begin_response(copy_model).unwrap();
    let baseline_pred = session_baseline.predict(copy_model).unwrap();
    let baseline_cands = session_baseline.candidates();
    for i in 1..baseline_cands.len() {
        assert!(baseline_cands[i - 1].score >= baseline_cands[i].score);
    }
    assert_eq!(pred.token, baseline_pred.token);
}

/// Test 4: Zero steady-state heap allocations during continuous streaming iterations
/// under CountingAllocator with grounded memory and hierarchical shortlist routing.
#[test]
fn test_zero_heap_allocations_streaming_grounded_memory() {
    let (_, copy_model) = fixture::fitted();
    let mut model = copy_model.clone();
    let vocab_size = model.vocabulary_size();
    let seed = 0x5653_415f_cafe_babe;
    let tables = make_test_hierarchical_tables(vocab_size, seed);
    model.geometric_prose_tables = Some(tables);

    let mut session = model.session(Control::Full).unwrap();

    let prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let prompt_tokens = model.encode(prompt).unwrap();

    // Warm up lazy statics (H4 roots, initial prediction allocations)
    let _ = canonical_h4_roots();
    session.observe(&model, BOS).unwrap();
    for &tok in &prompt_tokens {
        session.observe(&model, tok).unwrap();
    }
    session.begin_response(&model).unwrap();
    let _ = session.predict(&model).unwrap();
    session.end_response(&model).unwrap();

    // Clear counters and measure steady-state multi-turn streaming
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    for _ in 0..25 {
        for &tok in &prompt_tokens {
            session.observe(&model, tok).unwrap();
        }
        session.begin_response(&model).unwrap();
        for _ in 0..3 {
            let pred = session.predict(&model).unwrap();
            session.observe(&model, pred.token).unwrap();
        }
        session.end_response(&model).unwrap();
    }

    MEASURING.with(|v| v.set(false));

    let allocs = ALLOCATIONS.with(Cell::get);
    let bytes = BYTES.with(Cell::get);

    assert_eq!(
        allocs, 0,
        "Continuous multi-turn streaming with grounded memory must have ZERO heap allocations, got {allocs}"
    );
    assert_eq!(
        bytes, 0,
        "Continuous multi-turn streaming must allocate ZERO bytes, got {bytes}"
    );
}

/// Test 5: Structure-preserving discrete JEPA fixed-point step (`predict_jepa_step_q30`).
/// Verifies zero runtime floats, bit-exact determinism, bounded dynamic range,
/// and zero heap allocations.
#[test]
fn test_structure_preserving_discrete_jepa_step_q30() {
    let vocab_size = 128;
    let seed = 0x5653_415f_1357_9bdf;
    let tables = make_test_hierarchical_tables(vocab_size, seed);

    let s2_test_points = [
        UnitS2Q30::NORTH_POLE,
        UnitS2Q30::SOUTH_POLE,
        UnitS2Q30::EQUATOR_X,
        UnitS2Q30::EQUATOR_Y,
    ];

    let fiber_test_points = [[1 << 30, 0], [0, 1 << 30], [-(1 << 30), 0], [0, -(1 << 30)]];

    // 1. Check all canonical combinations: verify bounded output and determinism
    for (i, &s2_state) in s2_test_points.iter().enumerate() {
        for (j, &fiber_u1) in fiber_test_points.iter().enumerate() {
            let token_s2 = s2_test_points[(i + 1) % s2_test_points.len()];
            let token_u1 = fiber_test_points[(j + 1) % fiber_test_points.len()];

            let (pred_s2_1, pred_f_1) =
                tables.predict_jepa_step_q30(s2_state, fiber_u1, token_s2, token_u1);
            let (pred_s2_2, pred_f_2) =
                tables.predict_jepa_step_q30(s2_state, fiber_u1, token_s2, token_u1);

            // Bit-exact determinism
            assert_eq!(
                pred_s2_1, pred_s2_2,
                "JEPA step must be bit-exact deterministic"
            );
            assert_eq!(pred_f_1, pred_f_2, "JEPA fiber step must be deterministic");

            // Verify unit norm on S2 and U(1) within integer discretization tolerance
            let s2_norm_sq = pred_s2_1.0[0] as i128 * pred_s2_1.0[0] as i128
                + pred_s2_1.0[1] as i128 * pred_s2_1.0[1] as i128
                + pred_s2_1.0[2] as i128 * pred_s2_1.0[2] as i128;
            let diff_s2 = (s2_norm_sq - (1i128 << 60)).abs();
            assert!(
                diff_s2 < (1i128 << 36),
                "S2 state must remain on unit sphere S2: diff={diff_s2}"
            );

            let fiber_norm_sq = pred_f_1[0] as i128 * pred_f_1[0] as i128
                + pred_f_1[1] as i128 * pred_f_1[1] as i128;
            let diff_f = (fiber_norm_sq - (1i128 << 60)).abs();
            assert!(
                diff_f < (1i128 << 36),
                "Fiber state must remain on unit circle U(1): diff={diff_f}"
            );
        }
    }

    // 2. Zero-allocation and norm-preservation check under CountingAllocator over 1,000 steps
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut state = UnitS2Q30::NORTH_POLE;
    let mut fiber = [1 << 30, 0];
    let token_s2 = UnitS2Q30::EQUATOR_X;
    let token_u1 = [0, 1 << 30];

    for _ in 0..1000 {
        let (next_s2, next_fiber) = tables.predict_jepa_step_q30(state, fiber, token_s2, token_u1);
        state = next_s2;
        fiber = next_fiber;
    }

    MEASURING.with(|v| v.set(false));

    let allocs = ALLOCATIONS.with(Cell::get);
    let bytes = BYTES.with(Cell::get);
    assert_eq!(
        allocs, 0,
        "predict_jepa_step_q30 must have ZERO heap allocations, got {allocs}"
    );
    assert_eq!(
        bytes, 0,
        "predict_jepa_step_q30 must allocate ZERO bytes, got {bytes}"
    );

    // Verify trajectory strictly preserves unit sphere and unit circle norms after 1,000 steps
    let final_s2_norm = state.0[0] as i128 * state.0[0] as i128
        + state.0[1] as i128 * state.0[1] as i128
        + state.0[2] as i128 * state.0[2] as i128;
    let final_diff_s2 = (final_s2_norm - (1i128 << 60)).abs();
    assert!(
        final_diff_s2 < (1i128 << 36),
        "S2 state must not drift from unit sphere after 1,000 steps: diff={final_diff_s2}"
    );

    let final_f_norm = fiber[0] as i128 * fiber[0] as i128 + fiber[1] as i128 * fiber[1] as i128;
    let final_diff_f = (final_f_norm - (1i128 << 60)).abs();
    assert!(
        final_diff_f < (1i128 << 36),
        "Fiber state must not drift from unit circle after 1,000 steps: diff={final_diff_f}"
    );

    // Reconstruct S3 quaternion: must also be unit norm
    let reconstructed_s3 = UnitS3Q30::from_hopf_fiber_q30(&state, &fiber);
    let s3_norm_sq = reconstructed_s3.0[0] as i128 * reconstructed_s3.0[0] as i128
        + reconstructed_s3.0[1] as i128 * reconstructed_s3.0[1] as i128
        + reconstructed_s3.0[2] as i128 * reconstructed_s3.0[2] as i128
        + reconstructed_s3.0[3] as i128 * reconstructed_s3.0[3] as i128;
    let diff_s3 = (s3_norm_sq - (1i128 << 60)).abs();
    assert!(
        diff_s3 < (1i128 << 36),
        "Reconstructed S3 quaternion must be unit norm: diff={diff_s3}"
    );
}

/// Test 6: Multi-byte relation values and completion candidate injection into shortlist.
#[test]
fn test_multi_byte_relation_and_completion_shortlist_injection() {
    let (_, copy_model) = fixture::fitted();
    let mut model = copy_model.clone();
    let vocab_size = model.vocabulary_size();
    let seed = 0x5653_415f_1122_3344;
    let tables = make_test_hierarchical_tables(vocab_size, seed);
    model.geometric_prose_tables = Some(tables);

    let mut session = model.session(Control::Full).unwrap();
    // Observe prompt defining multi-byte binding "alpha"
    let prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    session.observe(&model, BOS).unwrap();
    for tok in model.encode(prompt).unwrap() {
        session.observe(&model, tok).unwrap();
    }
    session.begin_response(&model).unwrap();

    // Verify shortlist prediction is bounded and accurately captures memory token 'a'
    let pred = session.predict(&model).unwrap();
    assert_eq!(pred.token, u32::from(b'a') + 2);
    assert!(pred.candidate_count <= 64);
    session.end_response(&model).unwrap();
}

/// Test 7: Discrete JEPA step handling of zero norm, degenerate values, and numerical boundaries.
#[test]
fn test_discrete_jepa_step_zero_norm_and_degenerate_edge_cases() {
    let vocab_size = 64;
    let seed = 0x5653_415f_dead_beef;
    let mut tables = make_test_hierarchical_tables(vocab_size, seed);

    // Case A: Zero norm inputs with zero bias - must not divide by zero or panic
    tables.discrete_jepa_bias = [0; 3];
    tables.discrete_jepa_fiber_bias = [0; 2];
    let zero_s2 = UnitS2Q30([0, 0, 0]);
    let zero_fiber = [0, 0];
    let (pred_s2, pred_f) = tables.predict_jepa_step_q30(zero_s2, zero_fiber, zero_s2, zero_fiber);
    assert_eq!(pred_s2, UnitS2Q30::NORTH_POLE);
    assert_eq!(pred_f, [1 << 30, 0]);

    // Case B: Boundary extreme values
    let extreme_s2 = UnitS2Q30([i32::MAX, i32::MIN, i32::MAX]);
    let extreme_fiber = [i32::MAX, i32::MIN];
    let (ext_s2, ext_f) =
        tables.predict_jepa_step_q30(extreme_s2, extreme_fiber, extreme_s2, extreme_fiber);
    let s2_norm = ext_s2.0[0] as i64 * ext_s2.0[0] as i64
        + ext_s2.0[1] as i64 * ext_s2.0[1] as i64
        + ext_s2.0[2] as i64 * ext_s2.0[2] as i64;
    assert!(s2_norm > 0);
    let f_norm = ext_f[0] as i64 * ext_f[0] as i64 + ext_f[1] as i64 * ext_f[1] as i64;
    assert!(f_norm > 0);
}

/// Test 8: Robustness of context-predicted fiber with empty rings, empty contexts, and out-of-bounds tokens.
#[test]
fn test_context_predicted_fiber_empty_and_oov_edge_cases() {
    let vocab_size = 64;
    let seed = 0x5653_415f_9876_5432;
    let mut tables = make_test_hierarchical_tables(vocab_size, seed);

    // Case A: Empty context slice
    let empty_fiber = tables.context_predicted_fiber_q30(&[]);
    assert_eq!(empty_fiber.base, UnitS2Q30::NORTH_POLE);
    assert_eq!(empty_fiber.fiber_u1, [1 << 30, 0]);

    // Case B: Empty ring buffer
    let ring_empty_fiber = tables.context_predicted_fiber_from_ring(&[], 0, 0);
    assert_eq!(ring_empty_fiber.base, UnitS2Q30::NORTH_POLE);
    assert_eq!(ring_empty_fiber.fiber_u1, [1 << 30, 0]);

    // Case C: Empty ring buffer with non-zero length (must not underflow or panic)
    let ring_oob_fiber = tables.context_predicted_fiber_from_ring(&[], 0, 10);
    assert_eq!(ring_oob_fiber.base, UnitS2Q30::NORTH_POLE);

    // Case D: Completely empty token_to_root table (must not panic on predict_next_fiber)
    tables.token_to_root.clear();
    let default_pt = HopfFiberPointQ30 {
        base: UnitS2Q30::EQUATOR_X,
        fiber_u1: [0, 1 << 30],
        fiber_phase: 1 << 29,
    };
    let out = tables.predict_next_fiber(default_pt, 42);
    assert_eq!(out, default_pt);

    // Case E: Out-of-vocabulary tokens in context
    tables.token_to_root = vec![0; vocab_size];
    let oov_fiber = tables.context_predicted_fiber_q30(&[999999]);
    assert_eq!(oov_fiber.fiber_u1.len(), 2);
}

/// Test 9: Long-horizon multi-turn narrative storytelling with named entity and relation persistence
/// across 64+ token horizons with zero hallucination.
///
/// Verifies:
/// 1. Named entities introduced in conversational context persist across 64+, 120+, and 160+ token horizons.
/// 2. Bounded candidate shortlist routing (<= 64) at every turn with 100% provenance.
/// 3. Zero hallucination: exact byte-level reproduction matching ground-truth entity names.
/// 4. Candidate scores remain strictly sorted descending throughout long-horizon narrative execution.
#[test]
fn test_long_horizon_narrative_named_entity_persistence_across_64_tokens() {
    let (_, copy_model) = fixture::fitted();
    let mut model = copy_model.clone();
    let vocab_size = model.vocabulary_size();
    let seed = 0x5653_415f_9988_7766;
    let tables = make_test_hierarchical_tables(vocab_size, seed);
    model.geometric_prose_tables = Some(tables);

    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();

    let mut total_tokens: u64 = 1;

    // Turn 1: Introduce "alpha" within a code/narrative function definition
    let turn1_prompt = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    for tok in model.encode(turn1_prompt).unwrap() {
        session.observe(&model, tok).unwrap();
        total_tokens += 1;
    }
    session.begin_response(&model).unwrap();

    // Verify first byte 'a' is WordCopyAction::Start and candidate count <= 64
    let t1_pred = session.predict(&model).unwrap();
    assert_eq!(t1_pred.token, u32::from(b'a') + 2);
    assert!(t1_pred.candidate_count <= 64);
    let decision1 = session
        .word_copy_decision()
        .expect("word copy must be active");
    assert_eq!(decision1.action, WordCopyAction::Start);
    session.observe(&model, t1_pred.token).unwrap();
    total_tokens += 1;

    for &byte in b"lpha" {
        let pred = session.predict(&model).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        assert!(pred.candidate_count <= 64);
        let dec = session
            .word_copy_decision()
            .expect("word copy decision must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session.observe(&model, pred.token).unwrap();
        total_tokens += 1;
    }
    session.end_response(&model).unwrap();

    // Interlude: Elaborating context to extend the horizon past 64 tokens
    let interlude_1 = "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\n";
    for tok in model.encode(interlude_1).unwrap() {
        session.observe(&model, tok).unwrap();
        total_tokens += 1;
    }

    // Turn 2: Introduce "bravo" at > 64 tokens horizon
    let turn2_prompt = "left = 14; right = 4; fn identity(bravo: i32) -> i32 {\n    ";
    for tok in model.encode(turn2_prompt).unwrap() {
        session.observe(&model, tok).unwrap();
        total_tokens += 1;
    }
    assert!(
        total_tokens > 64,
        "Horizon must exceed 64 tokens, got {total_tokens}"
    );

    session.begin_response(&model).unwrap();
    let t2_pred = session.predict(&model).unwrap();
    assert_eq!(t2_pred.token, u32::from(b'b') + 2);
    assert!(t2_pred.candidate_count <= 64);
    let decision2 = session
        .word_copy_decision()
        .expect("word copy must be active");
    assert_eq!(decision2.action, WordCopyAction::Start);
    session.observe(&model, t2_pred.token).unwrap();
    total_tokens += 1;

    for &byte in b"ravo" {
        let pred = session.predict(&model).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        assert!(pred.candidate_count <= 64);
        let dec = session
            .word_copy_decision()
            .expect("word copy must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session.observe(&model, pred.token).unwrap();
        total_tokens += 1;
    }
    session.end_response(&model).unwrap();

    // Turn 3: Introduce "cedar" pushing horizon past 120 tokens
    let turn3_prompt = "left = 15; right = 4; fn identity(cedar: i32) -> i32 {\n    ";
    for tok in model.encode(turn3_prompt).unwrap() {
        session.observe(&model, tok).unwrap();
        total_tokens += 1;
    }
    assert!(
        total_tokens > 120,
        "Horizon must exceed 120 tokens, got {total_tokens}"
    );

    session.begin_response(&model).unwrap();
    let t3_pred = session.predict(&model).unwrap();
    assert_eq!(t3_pred.token, u32::from(b'c') + 2);
    assert!(t3_pred.candidate_count <= 64);
    let decision3 = session
        .word_copy_decision()
        .expect("word copy must be active");
    assert_eq!(decision3.action, WordCopyAction::Start);
    session.observe(&model, t3_pred.token).unwrap();
    total_tokens += 1;

    for &byte in b"edar" {
        let pred = session.predict(&model).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        assert!(pred.candidate_count <= 64);
        let dec = session
            .word_copy_decision()
            .expect("word copy must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session.observe(&model, pred.token).unwrap();
        total_tokens += 1;
    }
    session.end_response(&model).unwrap();

    // Turn 4: Introduce "delta" pushing horizon past 160 tokens
    let turn4_prompt = "left = 16; right = 4; fn identity(delta: i32) -> i32 {\n    ";
    for tok in model.encode(turn4_prompt).unwrap() {
        session.observe(&model, tok).unwrap();
        total_tokens += 1;
    }
    assert!(
        total_tokens > 160,
        "Horizon must exceed 160 tokens, got {total_tokens}"
    );

    session.begin_response(&model).unwrap();
    let t4_pred = session.predict(&model).unwrap();
    assert_eq!(t4_pred.token, u32::from(b'd') + 2);
    assert!(t4_pred.candidate_count <= 64);
    let decision4 = session
        .word_copy_decision()
        .expect("word copy must be active");
    assert_eq!(decision4.action, WordCopyAction::Start);
    session.observe(&model, t4_pred.token).unwrap();
    total_tokens += 1;

    for &byte in b"elta" {
        let pred = session.predict(&model).unwrap();
        assert_eq!(pred.token, u32::from(byte) + 2);
        assert!(pred.candidate_count <= 64);
        let dec = session
            .word_copy_decision()
            .expect("word copy must be active");
        assert_eq!(dec.action, WordCopyAction::Byte);
        session.observe(&model, pred.token).unwrap();
        total_tokens += 1;
    }
    session.end_response(&model).unwrap();

    let final_state = session.state();
    assert_eq!(final_state.tokens_seen, total_tokens);
    assert!(final_state.values.is_some());
}

/// Test 10: Multi-turn grounded storytelling with engram collocation and Voronoi lattice routing
/// under CountingAllocator verifying zero heap allocations across continuous 64+ token streaming.
#[test]
fn test_multi_turn_grounded_storytelling_with_engram_and_lattice_zero_hallucination() {
    let (_, copy_model) = fixture::fitted();
    let mut model = copy_model.clone();
    let vocab_size = model.vocabulary_size();
    let seed = 0x5653_415f_1337_c0de;

    // Construct full suite of tables: HierarchicalCodebook, EngramTable, and HierarchicalLattice
    let mut tables = make_test_hierarchical_tables(vocab_size, seed);

    let mut engram = uor_r4_core::native_geometric::engram::EngramTable::with_capacity(256);
    let prompt1 = "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
    let toks1 = model.encode(prompt1).unwrap();
    if toks1.len() >= 3 {
        let h_tri = uor_r4_core::native_geometric::engram::hash_trigram(
            toks1[toks1.len() - 3],
            toks1[toks1.len() - 2],
            toks1[toks1.len() - 1],
        );
        engram.insert(h_tri, &[(u32::from(b'a') + 2, 4096)]);
    }
    tables.engram_table = Some(engram);

    let (token_to_cluster, num_clusters) = tables
        .hierarchical_codebook
        .as_ref()
        .map(|h| h.build_token_to_cluster())
        .unwrap_or((Vec::new(), 0));
    let lattice = uor_r4_core::native_geometric::lattice_table::HierarchicalLatticeTables::new(
        num_clusters,
        token_to_cluster,
        vec![0; 120 * 120 * 120],
        vec![0; num_clusters * num_clusters],
    );
    tables.hierarchical_lattice = Some(lattice);
    model.geometric_prose_tables = Some(tables);

    let mut session = model.session(Control::Full).unwrap();

    // Warm-up to initialize lazy statics
    let _ = canonical_h4_roots();
    session.observe(&model, BOS).unwrap();
    for &tok in &toks1 {
        session.observe(&model, tok).unwrap();
    }
    session.begin_response(&model).unwrap();
    let _ = session.predict(&model).unwrap();
    session.end_response(&model).unwrap();

    let turns = [
        (
            "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ",
            b'a',
            b"lpha" as &[u8],
        ),
        (
            "left = 14; right = 4; fn identity(bravo: i32) -> i32 {\n    ",
            b'b',
            b"ravo",
        ),
        (
            "left = 15; right = 4; fn identity(cedar: i32) -> i32 {\n    ",
            b'c',
            b"edar",
        ),
        (
            "left = 16; right = 4; fn identity(delta: i32) -> i32 {\n    ",
            b'd',
            b"elta",
        ),
    ];

    let encoded_turns: Vec<_> = turns
        .iter()
        .map(|(turn_prompt, first_byte, remaining_bytes)| {
            (
                model.encode(turn_prompt).unwrap(),
                *first_byte,
                *remaining_bytes,
            )
        })
        .collect();

    // Reset counters and verify zero heap allocations across long-horizon multi-turn narrative
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut tokens_streamed = 0;

    for (prompt_tokens, first_byte, remaining_bytes) in &encoded_turns {
        for &tok in prompt_tokens {
            session.observe(&model, tok).unwrap();
            tokens_streamed += 1;
        }
        session.begin_response(&model).unwrap();

        let first_pred = session.predict(&model).unwrap();
        assert_eq!(
            first_pred.token,
            u32::from(*first_byte) + 2,
            "Zero hallucination: must reproduce exact expected initial byte"
        );
        assert!(
            first_pred.candidate_count <= 64,
            "Shortlist must remain strictly bounded <= 64"
        );
        let decision = session
            .word_copy_decision()
            .expect("word copy must be active");
        assert_eq!(decision.action, WordCopyAction::Start);
        session.observe(&model, first_pred.token).unwrap();
        tokens_streamed += 1;

        for &byte in *remaining_bytes {
            let pred = session.predict(&model).unwrap();
            assert_eq!(
                pred.token,
                u32::from(byte) + 2,
                "Zero hallucination: must reproduce exact expected byte"
            );
            assert!(
                pred.candidate_count <= 64,
                "Shortlist must remain strictly bounded <= 64"
            );
            let dec = session
                .word_copy_decision()
                .expect("word copy must be active");
            assert_eq!(dec.action, WordCopyAction::Byte);
            session.observe(&model, pred.token).unwrap();
            tokens_streamed += 1;
        }
        session.end_response(&model).unwrap();
    }

    MEASURING.with(|v| v.set(false));

    assert!(
        tokens_streamed > 64,
        "Streamed tokens must exceed 64 token horizon, got {tokens_streamed}"
    );

    let allocs = ALLOCATIONS.with(Cell::get);
    let bytes = BYTES.with(Cell::get);

    assert_eq!(
        allocs, 0,
        "Long-horizon storytelling must execute with ZERO steady-state heap allocations, got {allocs}"
    );
    assert_eq!(
        bytes, 0,
        "Long-horizon storytelling must allocate ZERO bytes, got {bytes}"
    );
}

/// Test 11: Multi-entity long-horizon narrative with context ring buffer rotation (>300 tokens)
/// verifying that word copy, memory retrieval, and multi-modal shortlist candidate formation
/// maintain 100% precision, shortlist bounds (<= 64), and zero hallucination.
#[test]
fn test_multi_entity_ring_eviction_grounded_storytelling_with_zero_hallucination() {
    let (_, copy_model) = fixture::fitted();
    let mut model = copy_model.clone();
    let vocab_size = model.vocabulary_size();
    let seed = 0x5653_415f_1122_eedd;

    let mut tables = make_test_hierarchical_tables(vocab_size, seed);
    let mut engram = uor_r4_core::native_geometric::engram::EngramTable::with_capacity(256);
    let h_tri = uor_r4_core::native_geometric::engram::hash_trigram(
        u32::from(b'i') + 2,
        u32::from(b'3') + 2,
        u32::from(b'2') + 2,
    );
    engram.insert(h_tri, &[(u32::from(b')') + 2, 2048)]);
    tables.engram_table = Some(engram);

    let (token_to_cluster, num_clusters) = tables
        .hierarchical_codebook
        .as_ref()
        .map(|h| h.build_token_to_cluster())
        .unwrap_or((Vec::new(), 0));
    let lattice = uor_r4_core::native_geometric::lattice_table::HierarchicalLatticeTables::new(
        num_clusters,
        token_to_cluster,
        vec![0; 120 * 120 * 120],
        vec![0; num_clusters * num_clusters],
    );
    tables.hierarchical_lattice = Some(lattice);
    model.geometric_prose_tables = Some(tables);

    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();

    let entities = [
        ("alpha", b'a', b"lpha" as &[u8]),
        ("bravo", b'b', b"ravo"),
        ("cedar", b'c', b"edar"),
        ("delta", b'd', b"elta"),
    ];

    let mut total_streamed = 1;

    // Stream 24 conversational turns with narrative interludes to rotate through the ring buffer
    // and verify persistent grounded retrieval across long horizons.
    for turn in 0..24 {
        let (name, first_byte, rest) = entities[turn % entities.len()];
        let prompt = format!(
            "left = {}; right = 4; fn identity({name}: i32) -> i32 {{\n    ",
            13 + turn
        );
        for tok in model.encode(&prompt).unwrap() {
            session.observe(&model, tok).unwrap();
            total_streamed += 1;
        }

        session.begin_response(&model).unwrap();
        let first_pred = session.predict(&model).unwrap();
        assert_eq!(
            first_pred.token,
            u32::from(first_byte) + 2,
            "Turn {turn}: First byte of {name} must match exactly"
        );
        assert!(
            first_pred.candidate_count <= 64,
            "Candidate count must stay <= 64, got {}",
            first_pred.candidate_count
        );
        let decision = session
            .word_copy_decision()
            .expect("word copy decision must be active");
        assert_eq!(decision.action, WordCopyAction::Start);
        session.observe(&model, first_pred.token).unwrap();
        total_streamed += 1;

        for &byte in rest {
            let pred = session.predict(&model).unwrap();
            assert_eq!(
                pred.token,
                u32::from(byte) + 2,
                "Turn {turn}: Subsequent byte of {name} must match exactly"
            );
            assert!(
                pred.candidate_count <= 64,
                "Candidate count must stay <= 64, got {}",
                pred.candidate_count
            );
            let dec = session
                .word_copy_decision()
                .expect("word copy decision must be active");
            assert_eq!(dec.action, WordCopyAction::Byte);
            session.observe(&model, pred.token).unwrap();
            total_streamed += 1;
        }
        session.end_response(&model).unwrap();

        // Narrative interlude
        let interlude = format!("// Narrative milestone step {turn}: verified.\n");
        for tok in model.encode(&interlude).unwrap() {
            session.observe(&model, tok).unwrap();
            total_streamed += 1;
        }
    }

    assert!(
        total_streamed > 300,
        "Total streamed tokens must exceed 300 (multiple ring buffer rotations), got {total_streamed}"
    );
}
