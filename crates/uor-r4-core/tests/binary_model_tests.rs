//! Integration tests for Milestone 13: Compact Binary Model Serialization (.rgm) & Mmap Serving.
//!
//! Verifies:
//! 1. Full roundtrip serialization (`ExportedGeometricModel::to_binary` -> `from_binary`).
//! 2. Binary model file size is <= 3.0 MB with all components (L2-cache lattice, engram, codebook).
//! 3. Zero-copy `MmapGeometricModel` cold-start load time < 50 microseconds.
//! 4. Bit-exact numerical parity across 1,000 queries between JSON reference, binary model, and Mmap model.
//! 5. Blake3 checksum integrity verification and single-bit corruption detection.
//! 6. Zero heap allocations on `MmapGeometricModel` inference hot paths.
//! 7. Exact 64-byte `repr(C)` header and 8-byte aligned section layout.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;
use uor_r4_core::native_geometric::engram::{hash_bigram, hash_trigram, EngramTable};
use uor_r4_core::native_geometric::hopf_metric::UnitS2Q30;
use uor_r4_core::native_geometric::lattice_table::{
    HierarchicalLatticeTables, COARSE_TABLE_SIZE, MAX_FINE_CLUSTERS,
};
use uor_r4_core::native_geometric::learner::binary_model::{
    BinaryModelError, MmapGeometricModel, RgmHeader, RgmSectionHeader, RGM_HEADER_SIZE,
};
use uor_r4_core::native_geometric::learner::{DiscreteServingTable, ExportedGeometricModel};
use uor_r4_core::native_geometric::vsa::{Codebook, HierarchicalCodebook};

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

/// Deterministic LCG for reproducible test models without external dependencies.
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }

    fn next_i16(&mut self) -> i16 {
        (self.next_u32() as i32 % 65536 - 32768) as i16
    }
}

/// The declared VSA code mode must survive a serialize/reload round trip, remain
/// **size-compatible**, and rebuild the hierarchical codebook in the matching space without
/// moving a single token between buckets.
///
/// This is the invariant that makes mode `1` coherent: `HierarchicalCodebook` centroids are
/// bundles of token vectors, so a code-space change without a matching rebuild would leave the
/// router comparing a query against centroids from a different space.
#[test]
fn test_vsa_code_mode_round_trip_and_coherent_rebuild() {
    use uor_r4_core::native_geometric::learner::build_root_codebook;

    let mut model = create_test_model();
    assert_eq!(
        model.vsa_code_mode, 0,
        "fixtures default to the fixed-hash mode"
    );

    let mode0_bytes = model.to_binary().expect("serialize mode 0");
    let reloaded0 = ExportedGeometricModel::from_binary(&mode0_bytes).expect("reload mode 0");
    assert_eq!(reloaded0.vsa_code_mode, 0);

    // Snapshot structure and anchors before the rebuild.
    let before = model
        .hierarchical_codebook
        .as_ref()
        .expect("fixture has a hierarchical codebook")
        .clone();
    let buckets_before: usize = before.sectors.iter().map(|s| s.len()).sum();
    let tokens_before: Vec<Vec<Vec<u32>>> = before
        .sectors
        .iter()
        .map(|s| s.iter().map(|b| b.tokens.clone()).collect())
        .collect();

    model.vsa_code_mode = 1;
    model.prepare_vsa_code_mode().expect("prepare mode 1");

    let after = model
        .hierarchical_codebook
        .as_ref()
        .expect("prepared model keeps its codebook");
    let buckets_after: usize = after.sectors.iter().map(|s| s.len()).sum();
    let tokens_after: Vec<Vec<Vec<u32>>> = after
        .sectors
        .iter()
        .map(|s| s.iter().map(|b| b.tokens.clone()).collect())
        .collect();

    assert_eq!(
        buckets_before, buckets_after,
        "the rebuild must not change the bucket count"
    );
    assert_eq!(
        tokens_before, tokens_after,
        "the rebuild must not move any token between buckets: the partition depends on \
         token_to_root and the leaf-bucket capacity, not on the code space"
    );
    assert_ne!(
        before.root_anchors, after.root_anchors,
        "the learned-root code space must differ from the fixed token-id hash"
    );

    // The model's codebook must be the root-derived one, element for element.
    let expected = build_root_codebook(model.vocab_size, &model.token_to_root, model.vsa_seed);
    let actual = model.vsa_codebook();
    assert_eq!(actual.table, expected.table);

    // Size compatibility: the mode lives in a reserved header byte, not a new section.
    let mode1_bytes = model.to_binary().expect("serialize mode 1");
    assert_eq!(
        mode0_bytes.len(),
        mode1_bytes.len(),
        "declaring the code mode must not change the artifact size"
    );
    let reloaded1 = ExportedGeometricModel::from_binary(&mode1_bytes).expect("reload mode 1");
    assert_eq!(reloaded1.vsa_code_mode, 1);

    // An out-of-range mode must be rejected rather than silently reinterpreted. The header byte
    // is inside the blake3-protected region, so the digest is recomputed to isolate the check.
    let mut corrupt = mode1_bytes.clone();
    corrupt[26] = 7;
    let digest = blake3::hash(&corrupt[64..]);
    corrupt[32..64].copy_from_slice(digest.as_bytes());
    assert!(
        ExportedGeometricModel::from_binary(&corrupt).is_err(),
        "an undefined code mode must be rejected"
    );
}

/// Constructs a full-featured `ExportedGeometricModel` containing all 6 canonical sections.
fn create_test_model() -> ExportedGeometricModel {
    let mut rng = SimpleRng::new(2026_0918);
    let vocab_size = 257;
    let num_lanes = 4;

    // 1. Token to H4 root mapping
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|_tok| (rng.next_u32() as usize % 120) as u8)
        .collect();

    // 2. Discrete multi-lane tables (4 lanes x 14,400 i16 scores)
    let mut discrete_tables = Vec::with_capacity(num_lanes);
    for _ in 0..num_lanes {
        let scores: Vec<i16> = (0..14_400).map(|_| rng.next_i16() / 4).collect();
        discrete_tables.push(DiscreteServingTable {
            scores,
            scale_bits: (16384.0_f64).to_bits(),
        });
    }

    // 3. Discrete bias
    let discrete_bias: Vec<i32> = (0..vocab_size)
        .map(|_| (rng.next_i16() as i32) * 8)
        .collect();

    // 4. Discrete S2 readout [vocab_size x 5]
    let discrete_s2_readout: Vec<[i16; 5]> = (0..vocab_size)
        .map(|_| {
            [
                rng.next_i16() / 2,
                rng.next_i16() / 2,
                rng.next_i16() / 2,
                rng.next_i16() / 2,
                rng.next_i16() / 2,
            ]
        })
        .collect();

    // 5. JEPA transition weights
    let mut discrete_jepa_w_state = [0i16; 9];
    let mut discrete_jepa_w_token = [0i16; 9];
    for i in 0..9 {
        discrete_jepa_w_state[i] = rng.next_i16() / 4;
        discrete_jepa_w_token[i] = rng.next_i16() / 4;
    }
    let discrete_jepa_bias = [rng.next_i16() / 4, rng.next_i16() / 4, rng.next_i16() / 4];

    let mut discrete_jepa_fiber_w_state = [0i16; 4];
    let mut discrete_jepa_fiber_w_token = [0i16; 4];
    for i in 0..4 {
        discrete_jepa_fiber_w_state[i] = rng.next_i16() / 4;
        discrete_jepa_fiber_w_token[i] = rng.next_i16() / 4;
    }
    let discrete_jepa_fiber_bias = [rng.next_i16() / 4, rng.next_i16() / 4];

    // 6. VSA parameters
    let vsa_seed = 42_u64;
    let vsa_scale_q15 = 1638_i16; // ~0.1 in Q1.14 / Q1.15

    // 7. Hierarchical codebook
    let vsa_codebook = Codebook::<64>::new(vocab_size, vsa_seed);
    let hierarchical_codebook =
        HierarchicalCodebook::new(vocab_size, &token_to_root, &vsa_codebook);

    // 8. Engram collocation table
    let mut engram_table = EngramTable::with_capacity(128);
    let key_th = hash_bigram(b'T' as u32, b'h' as u32);
    engram_table.insert_conditioned(key_th, 0, &[(b'e' as u32, 1024)]);
    let key_the = hash_trigram(b'T' as u32, b'h' as u32, b'e' as u32);
    engram_table.insert_conditioned(key_the, 0, &[(b' ' as u32, 2048)]);

    // 9. 2-Level Hierarchical Lattice Tables (120^3 coarse + 480^2 fine)
    let num_clusters = MAX_FINE_CLUSTERS; // 480 clusters
    let token_to_cluster: Vec<u16> = (0..vocab_size)
        .map(|tok| (tok % num_clusters) as u16)
        .collect();
    let coarse_trigram: Vec<i8> = (0..COARSE_TABLE_SIZE)
        .map(|i| (((i * 7 + 13) % 251) as i32 - 125) as i8)
        .collect();
    let fine_residual: Vec<i16> = (0..num_clusters * num_clusters)
        .map(|i| (((i * 17 + 29) % 4096) as i32 - 2048) as i16)
        .collect();
    let hierarchical_lattice = HierarchicalLatticeTables::new(
        num_clusters,
        token_to_cluster,
        coarse_trigram,
        fine_residual,
    );

    ExportedGeometricModel {
        vocab_size,
        token_to_root,
        discrete_tables,
        discrete_bias,
        discrete_s2_readout,
        discrete_jepa_w_state,
        discrete_jepa_w_token,
        discrete_jepa_bias,
        discrete_jepa_fiber_w_state,
        discrete_jepa_fiber_w_token,
        discrete_jepa_fiber_bias,
        vsa_seed,
        vsa_scale_q15,
        // Fixture is the fixed-hash code space and builds its hierarchical codebook in that same
        // space, so the two are coherent at mode 0.
        vsa_code_mode: 0,
        hierarchical_codebook: Some(hierarchical_codebook),
        engram_table: Some(engram_table),
        hierarchical_lattice: Some(hierarchical_lattice),
    }
}

#[test]
fn test_rgm_header_and_section_layout() {
    assert_eq!(std::mem::size_of::<RgmHeader>(), 64);
    assert_eq!(std::mem::align_of::<RgmHeader>(), 8);
    assert_eq!(std::mem::size_of::<RgmSectionHeader>(), 16);
    assert_eq!(std::mem::align_of::<RgmSectionHeader>(), 4);
    assert_eq!(RGM_HEADER_SIZE as usize, 64);
}

#[test]
fn test_roundtrip_binary_serialization() {
    let model = create_test_model();
    let binary_bytes = model.to_binary().expect("serialization to binary");

    // Deserialization from binary
    let deserialized =
        ExportedGeometricModel::from_binary(&binary_bytes).expect("deserialization from binary");

    assert_eq!(model.vocab_size, deserialized.vocab_size);
    assert_eq!(model.token_to_root, deserialized.token_to_root);
    assert_eq!(model.discrete_tables, deserialized.discrete_tables);
    assert_eq!(model.discrete_bias, deserialized.discrete_bias);
    assert_eq!(model.discrete_s2_readout, deserialized.discrete_s2_readout);
    assert_eq!(
        model.discrete_jepa_w_state,
        deserialized.discrete_jepa_w_state
    );
    assert_eq!(
        model.discrete_jepa_w_token,
        deserialized.discrete_jepa_w_token
    );
    assert_eq!(model.discrete_jepa_bias, deserialized.discrete_jepa_bias);
    assert_eq!(
        model.discrete_jepa_fiber_w_state,
        deserialized.discrete_jepa_fiber_w_state
    );
    assert_eq!(
        model.discrete_jepa_fiber_w_token,
        deserialized.discrete_jepa_fiber_w_token
    );
    assert_eq!(
        model.discrete_jepa_fiber_bias,
        deserialized.discrete_jepa_fiber_bias
    );
    assert_eq!(model.vsa_seed, deserialized.vsa_seed);
    assert_eq!(model.vsa_scale_q15, deserialized.vsa_scale_q15);
    assert_eq!(model, deserialized);
}

#[test]
fn test_file_size_is_bounded_under_3mb() {
    let model = create_test_model();
    let binary_bytes = model.to_binary().expect("serialization");
    let size = binary_bytes.len();

    println!(
        "Full RGM binary model size: {} bytes ({:.3} MB)",
        size,
        size as f64 / 1_048_576.0
    );

    assert!(
        size <= 3 * 1024 * 1024,
        "RGM file size must be <= 3.0 MB, got {} bytes ({:.3} MB)",
        size,
        size as f64 / 1_048_576.0
    );

    // Verify it is non-trivial and contains the coarse trigram table
    assert!(
        size >= 2 * 1024 * 1024,
        "RGM file size with full lattice should be >= 2.0 MB, got {}",
        size
    );
}

#[test]
fn test_mmap_cold_start_under_50us() {
    let model = create_test_model();
    let dir = std::env::temp_dir();
    let file_path = dir.join(format!("test_model_bench_{}.rgm", std::process::id()));
    model.save_to_binary(&file_path).expect("save binary file");

    // Warmup 5 iterations to avoid OS page cache cold disk access dominating CPU test
    for _ in 0..5 {
        let _ = MmapGeometricModel::open(&file_path).expect("open mmap");
    }

    // Benchmark 100 runs
    let mut durations = Vec::with_capacity(100);
    for _ in 0..100 {
        let start = Instant::now();
        let mmap_model = MmapGeometricModel::open(&file_path).expect("open mmap");
        let elapsed = start.elapsed();
        std::hint::black_box(mmap_model.vocab_size());
        durations.push(elapsed);
    }

    durations.sort();
    let min_duration = durations[0];
    let median_duration = durations[durations.len() / 2];

    println!(
        "MmapGeometricModel::open cold start benchmark: min = {:?}, median = {:?}",
        min_duration, median_duration
    );

    let _ = std::fs::remove_file(&file_path);

    assert!(
        min_duration < std::time::Duration::from_micros(50),
        "Cold-start load time must be < 50 us (got min: {:?}, median: {:?})",
        min_duration,
        median_duration
    );
}

#[test]
fn test_bit_exact_numerical_parity_1000_queries() {
    let model = create_test_model();
    let binary_bytes = model.to_binary().expect("to_binary");

    // Deserialized binary model
    let bin_model = ExportedGeometricModel::from_binary(&binary_bytes).expect("from_binary");

    // Deserialized JSON model
    let json_str = serde_json::to_string(&model).expect("to_json");
    let json_model: ExportedGeometricModel = serde_json::from_str(&json_str).expect("from_json");

    // Save to file for MmapGeometricModel
    let dir = std::env::temp_dir();
    let file_path = dir.join(format!("test_parity_{}.rgm", std::process::id()));
    model.save_to_binary(&file_path).expect("save to binary");
    let mmap_model = MmapGeometricModel::open(&file_path).expect("open mmap");

    let contexts: Vec<Vec<usize>> = vec![
        vec![],
        vec![b'T' as usize],
        vec![b'T' as usize, b'h' as usize],
        vec![b'T' as usize, b'h' as usize, b'e' as usize],
        vec![
            b'T' as usize,
            b'h' as usize,
            b'e' as usize,
            b' ' as usize,
            b'q' as usize,
        ],
        vec![
            b'O' as usize,
            b'n' as usize,
            b'c' as usize,
            b'e' as usize,
            b' ' as usize,
            b'u' as usize,
            b'p' as usize,
            b'o' as usize,
            b'n' as usize,
            b' ' as usize,
            b'a' as usize,
            b' ' as usize,
            b't' as usize,
            b'i' as usize,
            b'm' as usize,
            b'e' as usize,
        ],
    ];

    let mut query_count = 0;
    let mut rng = SimpleRng::new(987654321);

    for (ctx_idx, ctx) in contexts.iter().enumerate() {
        // 1. Check context_hopf_fiber_q30 parity
        let fiber_orig = model.context_hopf_fiber_q30(ctx);
        let fiber_bin = bin_model.context_hopf_fiber_q30(ctx);
        let fiber_json = json_model.context_hopf_fiber_q30(ctx);
        let fiber_mmap = mmap_model.context_hopf_fiber_q30(ctx);

        assert_eq!(fiber_orig, fiber_bin, "Fiber mismatch on ctx {}", ctx_idx);
        assert_eq!(fiber_orig, fiber_json, "Fiber mismatch on ctx {}", ctx_idx);
        assert_eq!(fiber_orig, fiber_mmap, "Fiber mismatch on ctx {}", ctx_idx);

        // 2. Check predict_jepa_step_q30 parity
        let s2_in = fiber_orig.base;
        let u1_in = fiber_orig.fiber_u1;
        let tok_s2 = UnitS2Q30::NORTH_POLE;
        let tok_u1 = [1073741824, 0]; // Q1.30 1.0

        let (pred_s2_orig, pred_u1_orig) =
            model.predict_jepa_step_q30(s2_in, u1_in, tok_s2, tok_u1);
        let (pred_s2_bin, pred_u1_bin) =
            bin_model.predict_jepa_step_q30(s2_in, u1_in, tok_s2, tok_u1);
        let (pred_s2_json, pred_u1_json) =
            json_model.predict_jepa_step_q30(s2_in, u1_in, tok_s2, tok_u1);
        let (pred_s2_mmap, pred_u1_mmap) =
            mmap_model.predict_jepa_step_q30(s2_in, u1_in, tok_s2, tok_u1);

        assert_eq!(pred_s2_orig, pred_s2_bin);
        assert_eq!(pred_s2_orig, pred_s2_json);
        assert_eq!(pred_s2_orig, pred_s2_mmap);
        assert_eq!(pred_u1_orig, pred_u1_bin);
        assert_eq!(pred_u1_orig, pred_u1_json);
        assert_eq!(pred_u1_orig, pred_u1_mmap);

        // 3. Score context candidate parity across candidates
        for cand in 0..model.vocab_size {
            let score_orig = model.score_context_candidate(ctx, cand, fiber_orig);
            let score_bin = bin_model.score_context_candidate(ctx, cand, fiber_bin);
            let score_json = json_model.score_context_candidate(ctx, cand, fiber_json);
            let score_mmap = mmap_model.score_context_candidate(ctx, cand, fiber_mmap);

            assert_eq!(
                score_orig, score_bin,
                "score_context_candidate mismatch (orig vs bin) for ctx {} cand {}",
                ctx_idx, cand
            );
            assert_eq!(
                score_orig, score_json,
                "score_context_candidate mismatch (orig vs json) for ctx {} cand {}",
                ctx_idx, cand
            );
            assert_eq!(
                score_orig, score_mmap,
                "score_context_candidate mismatch (orig vs mmap) for ctx {} cand {}",
                ctx_idx, cand
            );

            // Also test score_compatibility parity
            let q_tok = cand;
            let k_tok = (rng.next_u32() as usize) % model.vocab_size;
            let comp_orig = model.score_compatibility(q_tok, k_tok);
            let comp_bin = bin_model.score_compatibility(q_tok, k_tok);
            let comp_json = json_model.score_compatibility(q_tok, k_tok);
            let comp_mmap = mmap_model.score_compatibility(q_tok, k_tok);

            assert_eq!(comp_orig, comp_bin);
            assert_eq!(comp_orig, comp_json);
            assert_eq!(comp_orig, comp_mmap);

            query_count += 1;
        }
    }

    let _ = std::fs::remove_file(&file_path);

    println!(
        "Successfully verified bit-exact parity across {} total queries!",
        query_count
    );
    assert!(
        query_count >= 1000,
        "Must test at least 1000 queries, ran {}",
        query_count
    );
}

#[test]
fn test_checksum_corruption_detection() {
    let model = create_test_model();
    let mut binary_bytes = model.to_binary().expect("to_binary");

    // 1. Valid deserialization succeeds
    assert!(ExportedGeometricModel::from_binary(&binary_bytes).is_ok());

    // 2. Corrupt one bit in payload (e.g. byte 200)
    binary_bytes[200] ^= 0x01;
    let err = ExportedGeometricModel::from_binary(&binary_bytes).expect_err("must fail");
    match err {
        BinaryModelError::IntegrityCheckFailed => {}
        other => panic!("Expected IntegrityCheckFailed, got {:?}", other),
    }

    // 3. Truncated header
    let truncated = &binary_bytes[..32];
    let err_trunc = ExportedGeometricModel::from_binary(truncated).expect_err("must fail");
    match err_trunc {
        BinaryModelError::TruncatedHeader { .. } => {}
        other => panic!("Expected TruncatedHeader, got {:?}", other),
    }

    // 4. Invalid magic
    let mut bad_magic = binary_bytes.clone();
    bad_magic[0] = b'X';
    let err_magic = ExportedGeometricModel::from_binary(&bad_magic).expect_err("must fail");
    match err_magic {
        BinaryModelError::InvalidMagic(_) => {}
        other => panic!("Expected InvalidMagic, got {:?}", other),
    }

    // 5. Mmap verify_integrity succeeds on valid file and fails on corrupted file
    let dir = std::env::temp_dir();
    let valid_path = dir.join(format!("test_integrity_valid_{}.rgm", std::process::id()));
    let corrupt_path = dir.join(format!("test_integrity_corrupt_{}.rgm", std::process::id()));
    let valid_bytes = model.to_binary().expect("to_binary");
    std::fs::write(&valid_path, &valid_bytes).expect("write valid");
    let mut corrupt_bytes = valid_bytes.clone();
    corrupt_bytes[500] ^= 0x02;
    std::fs::write(&corrupt_path, &corrupt_bytes).expect("write corrupt");

    let mmap_valid = MmapGeometricModel::open(&valid_path).expect("open valid");
    assert!(mmap_valid.verify_integrity().is_ok());

    let mmap_corrupt = MmapGeometricModel::open(&corrupt_path).expect("open corrupt");
    match mmap_corrupt
        .verify_integrity()
        .expect_err("must fail integrity")
    {
        BinaryModelError::IntegrityCheckFailed => {}
        other => panic!("Expected IntegrityCheckFailed, got {:?}", other),
    }

    let _ = std::fs::remove_file(&valid_path);
    let _ = std::fs::remove_file(&corrupt_path);
}

#[test]
fn test_mmap_serving_zero_allocations() {
    let model = create_test_model();
    let dir = std::env::temp_dir();
    let file_path = dir.join(format!("test_mmap_zero_alloc_{}.rgm", std::process::id()));
    model.save_to_binary(&file_path).expect("save binary");

    let mmap_model = MmapGeometricModel::open(&file_path).expect("open mmap");

    let context = [b'T' as usize, b'h' as usize, b'e' as usize, b' ' as usize];
    let fiber = mmap_model.context_hopf_fiber_q30(&context);

    // Measure allocations on hot paths
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut accum = 0_i64;
    for cand in 0..mmap_model.vocab_size() {
        accum += mmap_model.score_context_candidate(&context, cand, fiber) as i64;
        accum += mmap_model.score_compatibility(cand, cand) as i64;
        let s2_cand = UnitS2Q30::NORTH_POLE;
        let u1_cand = [1073741824, 0];
        let (pred_s2, pred_u1) =
            mmap_model.predict_jepa_step_q30(fiber.base, fiber.fiber_u1, s2_cand, u1_cand);
        accum += pred_s2.0[0] as i64 + pred_u1[0] as i64;

        if let Some(cands) = mmap_model.lookup_engram_bigram(b'T' as u32, b'h' as u32) {
            accum += cands.len() as i64;
        }
        if let Some(cands) = mmap_model.lookup_engram_trigram(b'T' as u32, b'h' as u32, b'e' as u32)
        {
            accum += cands.len() as i64;
        }
    }
    std::hint::black_box(accum);

    MEASURING.with(|v| v.set(false));

    let alloc_count = ALLOCATIONS.with(Cell::get);
    let byte_count = BYTES.with(Cell::get);

    let _ = std::fs::remove_file(&file_path);

    assert_eq!(
        alloc_count, 0,
        "MmapGeometricModel hot path must perform ZERO heap allocations (got {})",
        alloc_count
    );
    assert_eq!(
        byte_count, 0,
        "MmapGeometricModel hot path must allocate ZERO bytes (got {})",
        byte_count
    );
}
