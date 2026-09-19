//! Tests for Milestone 6: 2-Level Hierarchical Lattice Tables ($120^3 + 480^2$, L2 Cache Resident).
//!
//! Verifies:
//! 1. Coarse 120^3 root trigram and fine 480^2 cluster bigram table indexing and scoring.
//! 2. Empirical n-gram statistics fitting and Laplace smoothing.
//! 3. Continuous forward, backward, and Adam optimization updates.
//! 4. Quantization to fixed-point integers (i8 scaled by 32, i16 scaled by 2048) with Q-scale bit shift.
//! 5. HierarchicalCodebook token-to-cluster partitioning.
//! 6. Zero heap allocations on serving hot path lookups under `CountingAllocator`.
//! 7. Serde serialization/deserialization roundtrip.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use uor_r4_core::native_geometric::lattice_table::{
    self_transition_surprisal, ContinuousLatticeTables, HierarchicalLatticeTables,
    COARSE_ROOT_COUNT, COARSE_SCALE, COARSE_TABLE_SIZE, FINE_SCALE, MAX_FINE_CLUSTERS,
    SELF_TRANSITION_RESIDUAL,
};
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

#[test]
fn test_hierarchical_lattice_tables_empty_and_memory_size() {
    let empty = HierarchicalLatticeTables::empty();
    assert_eq!(empty.num_clusters, 0);
    assert_eq!(empty.cluster_of(10), 0);
    assert_eq!(empty.coarse_score(5, 10, 15), 0);
    assert_eq!(empty.fine_score(2, 3), 0);
    assert_eq!(empty.score(5, 10, 15, 2, 3), 0);
    assert_eq!(empty.memory_bytes(), 0);

    let token_to_cluster = vec![0u16; 256];
    let coarse = vec![1i8; COARSE_TABLE_SIZE];
    let fine = vec![2i16; 100]; // 10x10
    let tables = HierarchicalLatticeTables::new(10, token_to_cluster, coarse, fine);
    assert_eq!(tables.num_clusters, 10);
    assert_eq!(tables.coarse_score(0, 0, 0), 1);
    assert_eq!(tables.fine_score(1, 1), 2);
    // Combined score: (1 << 6) + 2 = 64 + 2 = 66
    assert_eq!(tables.score(0, 0, 0, 1, 1), 66);
    assert_eq!(tables.memory_bytes(), 256 * 2 + COARSE_TABLE_SIZE + 100 * 2);
}

#[test]
fn test_continuous_lattice_tables_empirical_fitting() {
    let vocab_size = 64;
    let num_clusters = 8;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t| (t % COARSE_ROOT_COUNT) as u8)
        .collect();

    let mut continuous = ContinuousLatticeTables::new(num_clusters, token_to_cluster);

    // Sequence repeating (r0, r1, r2) = (1, 2, 3) frequently
    let mut sequences = Vec::new();
    for _ in 0..50 {
        sequences.push(vec![1, 2, 3, 1, 2, 3]);
    }
    // Sequences with other transitions
    for _ in 0..5 {
        sequences.push(vec![1, 2, 4]);
    }

    continuous.fit_empirical_ngram_statistics(&sequences, &token_to_root);

    // Transition (1, 2) -> 3 is very frequent, so logit should be positive
    let r1 = token_to_root[1] as usize;
    let r2 = token_to_root[2] as usize;
    let r3 = token_to_root[3] as usize;
    let r4 = token_to_root[4] as usize;

    let _c1 = continuous.cluster_of(1);
    let c2 = continuous.cluster_of(2);
    let c3 = continuous.cluster_of(3);

    let score_frequent = continuous.score_continuous(r1, r2, r3, c2, c3);
    let score_rare = continuous.score_continuous(r1, r2, r4, c2, c3);

    assert!(
        score_frequent > score_rare,
        "Frequent transition ({r1},{r2})->{r3} score {} must exceed rare ({r1},{r2})->{r4} score {}",
        score_frequent,
        score_rare
    );
}

#[test]
fn test_continuous_lattice_forward_backward_adam() {
    let vocab_size = 32;
    let num_clusters = 4;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let mut continuous = ContinuousLatticeTables::new(num_clusters, token_to_cluster);

    let r_prev = 5;
    let r_curr = 10;
    let r_cand = 15;
    let c_curr = 1;
    let c_cand = 2;

    let initial_score = continuous.score_continuous(r_prev, r_curr, r_cand, c_curr, c_cand);
    assert_eq!(initial_score, 0.0);

    // Backward pass with negative gradient (loss decreases as logit increases)
    continuous.backward(r_prev, r_curr, r_cand, c_curr, c_cand, -1.0);
    assert_eq!(continuous.active_coarse.len(), 1);
    assert_eq!(continuous.active_fine.len(), 1);

    // Step Adam
    continuous.step_adam(0.1, 0.9, 0.999, 0.0, 1);

    let updated_score = continuous.score_continuous(r_prev, r_curr, r_cand, c_curr, c_cand);
    assert!(
        updated_score > initial_score,
        "Adam step with negative gradient should increase logit score, got {}",
        updated_score
    );

    // Verify gradients are zeroed after step
    assert!(continuous.active_coarse.is_empty());
    assert!(continuous.active_fine.is_empty());
}

#[test]
fn test_lattice_quantization_precision() {
    let vocab_size = 32;
    let num_clusters = 4;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let mut continuous = ContinuousLatticeTables::new(num_clusters, token_to_cluster);

    let r_prev = 1;
    let r_curr = 2;
    let r_cand = 3;
    let c_curr = 0;
    let c_cand = 1;

    let coarse_idx = r_prev * 14400 + r_curr * 120 + r_cand;
    let fine_idx = c_curr * num_clusters + c_cand;

    // Set coarse logit = 1.0, fine logit = 0.5
    continuous.coarse_logits[coarse_idx] = 1.0;
    continuous.fine_logits[fine_idx] = 0.5;

    let discrete = continuous.quantize();

    assert_eq!(
        discrete.coarse_score(r_prev, r_curr, r_cand),
        (1.0 * COARSE_SCALE).round() as i8 // 32
    );
    assert_eq!(
        discrete.fine_score(c_curr, c_cand),
        (0.5 * FINE_SCALE).round() as i16 // 1024
    );

    // Combined score: (32 << 6) + 1024 = 2048 + 1024 = 3072
    let expected = (32i32 << 6) + 1024;
    assert_eq!(
        discrete.score(r_prev, r_curr, r_cand, c_curr, c_cand),
        expected
    );
}

#[test]
fn test_hierarchical_codebook_build_token_to_cluster() {
    let vocab_size = 256;
    let seed = 0x1234_5678_9abc_def0;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t| (t % COARSE_ROOT_COUNT) as u8)
        .collect();

    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);
    let (token_to_cluster, num_clusters) = hierarchical.build_token_to_cluster();

    assert_eq!(token_to_cluster.len(), vocab_size);
    assert!(num_clusters >= 1);
    assert!(
        num_clusters <= MAX_FINE_CLUSTERS,
        "num_clusters {} must not exceed MAX_FINE_CLUSTERS {}",
        num_clusters,
        MAX_FINE_CLUSTERS
    );

    for (token, &cluster) in token_to_cluster.iter().enumerate() {
        assert!(
            (cluster as usize) < num_clusters,
            "token {token} assigned cluster {cluster} >= num_clusters {num_clusters}"
        );
    }
}

#[test]
fn test_hierarchical_lattice_zero_allocations_under_counting_allocator() {
    let vocab_size = 256;
    let num_clusters = 16;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let coarse = vec![5i8; COARSE_TABLE_SIZE];
    let fine = vec![12i16; num_clusters * num_clusters];
    let tables = HierarchicalLatticeTables::new(num_clusters, token_to_cluster, coarse, fine);

    // Warm up
    let _ = tables.score(1, 2, 3, 0, 1);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut accum = 0i64;
    for i in 0..1024 {
        let r_prev = (i * 7) % 120;
        let r_curr = (i * 13) % 120;
        let r_cand = (i * 23) % 120;
        let c_curr = (i * 3) % num_clusters;
        let c_cand = (i * 5) % num_clusters;
        accum += tables.score(r_prev, r_curr, r_cand, c_curr, c_cand) as i64;
    }
    std::hint::black_box(accum);

    MEASURING.with(|v| v.set(false));

    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "HierarchicalLatticeTables serving lookups must perform ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "HierarchicalLatticeTables serving lookups must allocate ZERO bytes"
    );
}

#[test]
fn test_serde_hierarchical_lattice_roundtrip() {
    let vocab_size = 32;
    let num_clusters = 4;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let coarse = vec![7i8; 1000]; // Small slice for JSON roundtrip test
    let fine = vec![42i16; num_clusters * num_clusters];
    let original = HierarchicalLatticeTables::new(num_clusters, token_to_cluster, coarse, fine);

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: HierarchicalLatticeTables = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_self_transition_surprisal_mathematical_derivation() {
    // 1. Degenerate and boundary cases
    assert_eq!(self_transition_surprisal(0), 0);
    assert_eq!(self_transition_surprisal(1), 0);

    // 2. Exact powers of two
    assert_eq!(self_transition_surprisal(2), -2048);
    assert_eq!(self_transition_surprisal(4), -4096);
    assert_eq!(self_transition_surprisal(256), -16384);
    assert_eq!(self_transition_surprisal(1024), -20480);
    assert_eq!(self_transition_surprisal(4096), -24576);
    assert_eq!(self_transition_surprisal(4096), SELF_TRANSITION_RESIDUAL);
    assert_eq!(self_transition_surprisal(32768), -30720);
    assert_eq!(self_transition_surprisal(65536), -32768);

    // 3. Non-powers of two: verified against reference -round(log2(v) * 2048.0)
    let test_vocab_sizes = [3, 5, 10, 50, 100, 500, 1000, 3000, 5000, 32000, 50257];
    for &v in &test_vocab_sizes {
        let actual = self_transition_surprisal(v);
        let expected_f = -(libm::log2(v as f64) * 2048.0).round() as i32;
        let diff = (actual - expected_f).abs();
        assert!(
            diff <= 1,
            "self_transition_surprisal({v}) = {actual} deviated from reference {expected_f} by {diff} (must be <= 1 LSB)"
        );
    }
}

#[test]
fn test_score_token_dynamic_vocabulary_scaling() {
    // A: Vocab size 4096 (Canonical)
    let v4096 = 4096;
    let c16 = 16;
    let t2c_4096: Vec<u16> = (0..v4096).map(|t| (t % c16) as u16).collect();
    let coarse = vec![1i8; COARSE_TABLE_SIZE];
    let fine = vec![100i16; c16 * c16];
    let table_4096 = HierarchicalLatticeTables::new(c16, t2c_4096, coarse.clone(), fine.clone());

    // coarse is 1 -> (1 << 6) = 64
    // fine for normal is 100 -> 64 + 100 = 164
    assert_eq!(table_4096.score_token(0, 0, 0, 1, 1, false), 164);
    // fine for self transition is -24576 -> 64 - 24576 = -24512
    assert_eq!(table_4096.score_token(0, 0, 0, 1, 1, true), 64 - 24576);
    assert_eq!(table_4096.self_transition_residual(), -24576);

    // B: Vocab size 256 (Small byte-level vocab)
    let v256 = 256;
    let t2c_256: Vec<u16> = (0..v256).map(|t| (t % c16) as u16).collect();
    let table_256 = HierarchicalLatticeTables::new(c16, t2c_256, coarse.clone(), fine.clone());

    assert_eq!(table_256.score_token(0, 0, 0, 1, 1, false), 164);
    // fine for self transition is -16384 -> 64 - 16384 = -16320
    assert_eq!(table_256.score_token(0, 0, 0, 1, 1, true), 64 - 16384);
    assert_eq!(table_256.self_transition_residual(), -16384);

    // C: Vocab size 32
    let v32 = 32;
    let t2c_32: Vec<u16> = (0..v32).map(|t| (t % c16) as u16).collect();
    let table_32 = HierarchicalLatticeTables::new(c16, t2c_32, coarse, fine);

    assert_eq!(table_32.score_token(0, 0, 0, 1, 1, false), 164);
    // log2(32) = 5 -> -5 * 2048 = -10240 -> 64 - 10240 = -10176
    assert_eq!(table_32.score_token(0, 0, 0, 1, 1, true), 64 - 10240);
    assert_eq!(table_32.self_transition_residual(), -10240);
}

#[test]
fn test_score_token_zero_allocations_under_counting_allocator() {
    let vocab_size = 4096;
    let num_clusters = 64;
    let token_to_cluster: Vec<u16> = (0..vocab_size).map(|t| (t % num_clusters) as u16).collect();
    let coarse = vec![3i8; COARSE_TABLE_SIZE];
    let fine = vec![25i16; num_clusters * num_clusters];
    let table = HierarchicalLatticeTables::new(num_clusters, token_to_cluster, coarse, fine);

    // Warm up
    let _ = table.score_token(1, 2, 3, 0, 1, true);

    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut accum = 0i64;
    for i in 0..10_000 {
        let is_self = (i % 2) == 0;
        let r_prev = (i * 7) % 120;
        let r_curr = (i * 13) % 120;
        let r_cand = (i * 23) % 120;
        let c_curr = (i * 3) % num_clusters;
        let c_cand = (i * 5) % num_clusters;
        accum += table.score_token(r_prev, r_curr, r_cand, c_curr, c_cand, is_self) as i64;
        accum += self_transition_surprisal((i % 50000) + 2) as i64;
    }
    std::hint::black_box(accum);

    MEASURING.with(|v| v.set(false));

    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "score_token and self_transition_surprisal must perform ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "score_token and self_transition_surprisal must allocate ZERO bytes"
    );
}
