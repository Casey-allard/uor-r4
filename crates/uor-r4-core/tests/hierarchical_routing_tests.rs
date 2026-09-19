//! Tests for Milestone 3: Hierarchical Lattice Codebook & Bounded Shortlist Routing.
//!
//! Validates:
//! 1. Partitioning: All tokens in 0..vocab_size are assigned to exactly one leaf bucket;
//!    all bucket sizes <= 32.
//! 2. Bounded candidate offering in `predict`: Candidate offers remain bounded (<= 64 offers).
//! 3. Zero-allocation routing: `route_shortlist` and hot paths execute with 0 heap allocations
//!    under `CountingAllocator`.
//! 4. Coarse & Fine routing precision: Correct sector and bucket selection by S3 dot product
//!    and VSA similarity.
//! 5. Serde roundtrip: Exact deserialization equivalence.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use uor_r4_core::native_geometric::learner::embedding::{
    canonical_h4_roots, canonical_h4_roots_q30, H4_ROOT_COUNT,
};
use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::native_geometric::vsa::{
    Codebook, HierarchicalCodebook, Hypervector4096, LeafBucket, Shortlist,
};
use uor_r4_core::native_geometric::{Config, Control, Document, Trainer};

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
fn test_partitioning_guarantees() {
    for vocab_size in [10, 50, 120, 256, 500, 1024, 2048] {
        let seed = 0x5653_415f_1234_5678;
        let base_codebook = Codebook::<64>::new(vocab_size, seed);

        // Deterministic pseudo-random token-to-root mapping
        let token_to_root: Vec<u8> = (0..vocab_size)
            .map(|t: usize| ((t.wrapping_mul(37).wrapping_add(13)) % 120) as u8)
            .collect();

        let hierarchical =
            HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

        assert_eq!(hierarchical.vocab_size, vocab_size);

        // 1. All tokens in 0..vocab_size are stored across all buckets
        assert_eq!(
            hierarchical.total_token_count(),
            vocab_size,
            "Total token count must match vocab_size for vocab {}",
            vocab_size
        );

        // 2. Every token in 0..vocab_size is assigned to exactly one leaf bucket
        let mut token_seen = vec![0usize; vocab_size];
        for (sec_idx, sector) in hierarchical.sectors.iter().enumerate() {
            for bucket in sector {
                // All bucket sizes <= 32 and > 0
                assert!(
                    bucket.tokens.len() <= LeafBucket::<64>::MAX_TOKENS,
                    "Bucket exceeds max capacity 32: {}",
                    bucket.tokens.len()
                );
                assert!(!bucket.tokens.is_empty(), "Leaf bucket must not be empty");

                for &token in &bucket.tokens {
                    assert!(
                        (token as usize) < vocab_size,
                        "Token ID {} out of vocab bounds {}",
                        token,
                        vocab_size
                    );
                    token_seen[token as usize] += 1;

                    // Verify token is in the correct root sector
                    let expected_root = token_to_root[token as usize] as usize % 120;
                    assert_eq!(
                        sec_idx, expected_root,
                        "Token {} assigned to sector {} but expected root is {}",
                        token, sec_idx, expected_root
                    );
                }

                // Verify centroid matches the bundle of its tokens
                let token_vecs: Vec<Hypervector4096> = bucket
                    .tokens
                    .iter()
                    .map(|&t| base_codebook.get(t))
                    .collect();
                let expected_centroid = Hypervector4096::bundle(&token_vecs);
                assert_eq!(
                    bucket.centroid, expected_centroid,
                    "Bucket centroid must match bundle of token vectors"
                );
            }
        }

        // Verify each token was seen exactly once
        for (token, &count) in token_seen.iter().enumerate() {
            assert_eq!(
                count, 1,
                "Token {} was assigned {} times (expected exactly 1) for vocab {}",
                token, count, vocab_size
            );
        }

        // 3. Verify root sector anchors
        for (sec_idx, sector) in hierarchical.sectors.iter().enumerate() {
            let sector_tokens: Vec<u32> = sector
                .iter()
                .flat_map(|b| b.tokens.iter().copied())
                .collect();
            if sector_tokens.is_empty() {
                let expected = base_codebook.get_atom(&format!("h4_root_{}", sec_idx));
                assert_eq!(hierarchical.root_anchors[sec_idx], expected);
            } else {
                let vecs: Vec<Hypervector4096> = sector_tokens
                    .iter()
                    .map(|&t| base_codebook.get(t))
                    .collect();
                let expected = Hypervector4096::bundle(&vecs);
                assert_eq!(hierarchical.root_anchors[sec_idx], expected);
            }
        }
    }
}

#[test]
fn test_zero_allocation_routing_under_counting_allocator() {
    let vocab_size = 512;
    let seed = 0x5653_415f_8765_4321;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(43).wrapping_add(19)) % 120) as u8)
        .collect();
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    // Warm up lazy statics (canonical_h4_roots) outside measured section
    let roots = canonical_h4_roots();
    let sample_s3 = roots[7];
    let sample_vsa = base_codebook.get(42);

    // Warm up route_shortlist
    let _ = hierarchical.route_shortlist::<64>(Some(sample_s3), Some(&sample_vsa));

    // Zero-allocation census on route_shortlist across various combinations
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    let mut checksum = 0usize;
    for iter in 0..1000 {
        let root_idx = iter % 120;
        let s3_query = Some(roots[root_idx]);
        let token_id = (iter * 17) % vocab_size;
        let vsa_query = Some(base_codebook.get(token_id as u32));

        // 1. Both S3 and VSA
        let sl1 = hierarchical.route_shortlist::<64>(s3_query, vsa_query.as_ref());
        checksum += sl1.len();
        assert!(sl1.len() <= 64);

        // 2. S3 only
        let sl2 = hierarchical.route_shortlist::<64>(s3_query, None);
        checksum += sl2.len();
        assert!(sl2.len() <= 64);

        // 3. VSA only
        let sl3 = hierarchical.route_shortlist::<64>(None, vsa_query.as_ref());
        checksum += sl3.len();
        assert!(sl3.len() <= 64);

        // 4. Neither
        let sl4 = hierarchical.route_shortlist::<64>(None, None);
        checksum += sl4.len();
        assert!(sl4.len() <= 64);

        // 5. Smaller MAX capacities (32, 16)
        let sl32 = hierarchical.route_shortlist::<32>(s3_query, vsa_query.as_ref());
        checksum += sl32.len();
        assert!(sl32.len() <= 32);

        let sl16 = hierarchical.route_shortlist::<16>(s3_query, vsa_query.as_ref());
        checksum += sl16.len();
        assert!(sl16.len() <= 16);
    }

    MEASURING.with(|v| v.set(false));

    std::hint::black_box(checksum);
    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "route_shortlist must execute with ZERO steady-state heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "route_shortlist must allocate ZERO bytes"
    );
}

#[test]
fn test_bounded_candidate_offering_in_predict() {
    let vocab_size = 256;
    let seed = 0x5653_415f_1122_3344;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(17).wrapping_add(5)) % 120) as u8)
        .collect();
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    // 1. Direct validation: route_shortlist strictly bounds candidate offers to <= MAX
    let sl64: Shortlist<64> = hierarchical.route_shortlist::<64>(None, None);
    assert!(
        sl64.len() <= 64,
        "Shortlist<64> candidate offers must be <= 64, got {}",
        sl64.len()
    );
    assert!(
        !sl64.is_empty(),
        "Shortlist should not be empty for non-empty vocab"
    );

    let sl32: Shortlist<32> = hierarchical.route_shortlist::<32>(None, None);
    assert!(
        sl32.len() <= 32,
        "Shortlist<32> candidate offers must be <= 32, got {}",
        sl32.len()
    );

    let sl16: Shortlist<16> = hierarchical.route_shortlist::<16>(None, None);
    assert!(
        sl16.len() <= 16,
        "Shortlist<16> candidate offers must be <= 16, got {}",
        sl16.len()
    );

    // 2. Predict integration: verify candidate offers from prose tables are strictly bounded
    let documents = [
        Document {
            id: "doc1".into(),
            text: "the quick brown fox jumps over the lazy dog. the fox is swift.".into(),
        },
        Document {
            id: "doc2".into(),
            text: "the lazy dog sleeps under the warm sunshine. sunshine is bright.".into(),
        },
    ];

    let config = Config {
        context_tokens: 32,
        candidate_limit: 64,
        postings_per_row: 8,
        ..Config::default()
    };

    let mut trainer = Trainer::new(config, &documents).unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();

    let model_vocab = 256;
    let model_codebook = Codebook::<64>::new(model_vocab, seed);
    let model_token_to_root: Vec<u8> = (0..model_vocab)
        .map(|t: usize| ((t.wrapping_mul(17).wrapping_add(5)) % 120) as u8)
        .collect();
    let model_hierarchical =
        HierarchicalCodebook::<64>::new(model_vocab, &model_token_to_root, &model_codebook);

    let exported = ExportedGeometricModel {
        vocab_size: model_vocab,
        token_to_root: model_token_to_root,
        discrete_tables: Vec::new(),
        discrete_bias: vec![0; model_vocab],
        discrete_s2_readout: vec![[0; 5]; model_vocab],
        discrete_jepa_w_state: [0; 9],
        discrete_jepa_w_token: [0; 9],
        discrete_jepa_bias: [0; 3],
        discrete_jepa_fiber_w_state: [0; 4],
        discrete_jepa_fiber_w_token: [0; 4],
        discrete_jepa_fiber_bias: [0; 2],
        vsa_seed: seed,
        vsa_scale_q15: 1000,
        hierarchical_codebook: Some(model_hierarchical),
        engram_table: None,
        hierarchical_lattice: None,
    };

    model.geometric_prose_tables = Some(exported);

    let mut session = model.session(Control::Full).unwrap();
    for ch in b"the quick brown fox" {
        session.observe(&model, *ch as u32).unwrap();
    }

    // Verify prose table shortlist is strictly <= 64 candidate offers
    let prose_offers = model
        .geometric_prose_tables
        .as_ref()
        .map(|t| {
            let hier = t.hierarchical_codebook.as_ref().unwrap();
            let sl = hier.route_shortlist::<64>(None, None);
            sl.len()
        })
        .unwrap();

    assert!(
        prose_offers <= 64,
        "Prose candidate offers must remain bounded <= 64, got {}",
        prose_offers
    );

    let prediction = session.predict(&model).unwrap();
    assert!(prediction.token <= 512);
    assert!(
        prediction.candidate_count <= 64,
        "Retained candidate count must remain bounded <= 64, got {}",
        prediction.candidate_count
    );
}

#[test]
fn test_session_observe_predict_zero_allocation_with_hierarchical() {
    let documents = [
        Document {
            id: "alpha".into(),
            text: "alpha beta gamma delta epsilon zeta eta theta iota kappa.".into(),
        },
        Document {
            id: "lambda".into(),
            text: "lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega.".into(),
        },
    ];

    let config = Config {
        context_tokens: 32,
        candidate_limit: 16,
        postings_per_row: 4,
        ..Config::default()
    };

    let mut trainer = Trainer::new(config, &documents).unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();

    let vocab_size = 256;
    let seed = 0x5653_415f_9988_7766;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(23).wrapping_add(7)) % 120) as u8)
        .collect();
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    let exported = ExportedGeometricModel {
        vocab_size,
        token_to_root,
        discrete_tables: Vec::new(),
        discrete_bias: vec![0; vocab_size],
        discrete_s2_readout: vec![[0; 5]; vocab_size],
        discrete_jepa_w_state: [0; 9],
        discrete_jepa_w_token: [0; 9],
        discrete_jepa_bias: [0; 3],
        discrete_jepa_fiber_w_state: [0; 4],
        discrete_jepa_fiber_w_token: [0; 4],
        discrete_jepa_fiber_bias: [0; 2],
        vsa_seed: seed,
        vsa_scale_q15: 500,
        hierarchical_codebook: Some(hierarchical),
        engram_table: None,
        hierarchical_lattice: None,
    };

    model.geometric_prose_tables = Some(exported);

    let mut session = model.session(Control::Full).unwrap();

    // Warm up lazy statics
    let _ = canonical_h4_roots();
    session.observe(&model, b'a' as u32).unwrap();
    let _ = session.predict(&model).unwrap();

    // Measure steady-state observe and predict
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|v| v.set(true));

    for &byte in b"lpha beta gamma delta" {
        session.observe(&model, byte as u32).unwrap();
        let _ = std::hint::black_box(session.predict(&model).unwrap());
    }

    MEASURING.with(|v| v.set(false));

    assert_eq!(
        ALLOCATIONS.with(Cell::get),
        0,
        "Steady-state session observe/predict with HierarchicalCodebook must have ZERO heap allocations"
    );
    assert_eq!(
        BYTES.with(Cell::get),
        0,
        "Steady-state session observe/predict must allocate ZERO bytes"
    );
}

#[test]
fn test_coarse_and_fine_routing_precision() {
    let vocab_size = 256;
    let seed = 0x5653_415f_c0de_cafe;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);

    // Put tokens 0..10 into sector 5, 10..20 into sector 12, etc.
    let mut token_to_root = vec![0u8; vocab_size];
    for t in 0..10 {
        token_to_root[t] = 5;
    }
    for t in 10..20 {
        token_to_root[t] = 12;
    }

    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    let roots = canonical_h4_roots();

    // 1. Query matching canonical root 5 exactly
    let query_s3 = roots[5];
    let shortlist_s3 = hierarchical.route_shortlist::<64>(Some(query_s3), None);

    // Sector 5 tokens (0..10) must be present in shortlist
    for t in 0..10u32 {
        assert!(
            shortlist_s3.as_slice().contains(&t),
            "Shortlist from S3 query must contain token {}",
            t
        );
    }

    // 2. Query matching canonical root 12 exactly
    let query_s3_12 = roots[12];
    let shortlist_s3_12 = hierarchical.route_shortlist::<64>(Some(query_s3_12), None);

    for t in 10..20u32 {
        assert!(
            shortlist_s3_12.as_slice().contains(&t),
            "Shortlist from S3 query must contain token {}",
            t
        );
    }

    // 3. Query matching VSA sector anchor 5
    let query_vsa_5 = hierarchical.root_anchors[5];
    let shortlist_vsa_5 = hierarchical.route_shortlist::<64>(None, Some(&query_vsa_5));

    for t in 0..10u32 {
        assert!(
            shortlist_vsa_5.as_slice().contains(&t),
            "Shortlist from VSA anchor query must contain token {}",
            t
        );
    }

    // 4. Query matching specific token vector 3 (in sector 5)
    let query_tok_3 = base_codebook.get(3);
    let shortlist_tok_3 = hierarchical.route_shortlist::<64>(None, Some(&query_tok_3));
    assert!(
        shortlist_tok_3.as_slice().contains(&3),
        "Shortlist must contain the exact query token"
    );
}

#[test]
fn test_serde_hierarchical_codebook_roundtrip() {
    let vocab_size = 128;
    let seed = 0x5653_415f_dead_beef;
    let base_codebook = Codebook::<64>::new(vocab_size, seed);
    let token_to_root: Vec<u8> = (0..vocab_size)
        .map(|t: usize| ((t.wrapping_mul(31).wrapping_add(11)) % 120) as u8)
        .collect();

    let original = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: HierarchicalCodebook<64> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original.vocab_size, deserialized.vocab_size);
    assert_eq!(original.sectors.len(), deserialized.sectors.len());
    for r in 0..120 {
        assert_eq!(original.sectors[r], deserialized.sectors[r]);
        assert_eq!(original.root_anchors[r], deserialized.root_anchors[r]);
    }
    assert_eq!(original, deserialized);
}

#[test]
fn test_antipodal_twins_distinguished_in_s3_routing() {
    let roots = canonical_h4_roots_q30();
    // Find an antipodal pair: root_a and root_b where root_b == -root_a
    let mut pair = None;
    for i in 0..H4_ROOT_COUNT {
        for j in (i + 1)..H4_ROOT_COUNT {
            if roots[i].0[0] == -roots[j].0[0]
                && roots[i].0[1] == -roots[j].0[1]
                && roots[i].0[2] == -roots[j].0[2]
                && roots[i].0[3] == -roots[j].0[3]
            {
                pair = Some((i, j));
                break;
            }
        }
        if pair.is_some() {
            break;
        }
    }
    let (root_pos, root_neg) = pair.expect("must find antipodal twin pair in H4 roots");

    let vocab_size = 64;
    let base_codebook = Codebook::<64>::new(vocab_size, 0x1234_5678);
    let mut token_to_root = vec![0u8; vocab_size];
    // Tokens 0..5 assigned to root_pos
    for t in 0..5 {
        token_to_root[t] = root_pos as u8;
    }
    // Tokens 5..10 assigned to root_neg (antipodal twin)
    for t in 5..10 {
        token_to_root[t] = root_neg as u8;
    }
    // Other tokens assigned to non-antipodal roots
    for t in 10..vocab_size {
        token_to_root[t] = ((t * 7) % 120) as u8;
    }

    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    // Query matching root_pos exactly
    let query_s3 = roots[root_pos];
    let shortlist = hierarchical.route_shortlist_q30_with_memory::<16>(Some(query_s3), None, &[]);

    // Sector root_pos tokens MUST be in shortlist
    for t in 0..5u32 {
        assert!(
            shortlist.as_slice().contains(&t),
            "Shortlist must contain positive root token {}",
            t
        );
    }

    // Antipodal twin tokens (sector root_neg) MUST NOT be in shortlist
    // because dot is -1.0, not +1.0
    for t in 5..10u32 {
        assert!(
            !shortlist.as_slice().contains(&t),
            "Antipodal twin token {} must NOT be in shortlist (antipodal collapse eliminated)",
            t
        );
    }
}

#[test]
fn test_balanced_multi_sector_shortlist_distribution() {
    let vocab_size = 300;
    let base_codebook = Codebook::<64>::new(vocab_size, 0x5653_415f_1111);
    // Put 20 tokens in each of sectors 0..15
    let mut token_to_root = vec![0u8; vocab_size];
    for t in 0..vocab_size {
        token_to_root[t] = (t / 20) as u8;
    }
    let hierarchical = HierarchicalCodebook::<64>::new(vocab_size, &token_to_root, &base_codebook);

    // 16 memory tokens
    let memory_tokens: Vec<u32> = (250..266).map(|t| t as u32).collect();

    let roots = canonical_h4_roots_q30();
    let query_s3 = roots[0];

    let sl: Shortlist<64> =
        hierarchical.route_shortlist_q30_with_memory(Some(query_s3), None, &memory_tokens);

    assert_eq!(
        sl.len(),
        64,
        "Shortlist must be fully populated to 64 slots"
    );

    // Check all 16 memory tokens are present
    for &m in &memory_tokens {
        assert!(
            sl.as_slice().contains(&m),
            "Shortlist must contain memory priority token {}",
            m
        );
    }

    // Count non-memory tokens per sector
    let mut sector_counts = [0usize; 120];
    for &t in sl.as_slice() {
        if !memory_tokens.contains(&t) {
            let sec = token_to_root[t as usize] as usize;
            sector_counts[sec] += 1;
        }
    }

    // No sector should have contributed more than MAX_PER_SECTOR = 8 tokens
    for (sec, &cnt) in sector_counts.iter().enumerate() {
        assert!(
            cnt <= uor_r4_core::native_geometric::vsa::MAX_PER_SECTOR,
            "Sector {} contributed {} tokens, exceeding MAX_PER_SECTOR (8)",
            sec,
            cnt
        );
    }

    // Exactly 6 sectors should have contributed 8 tokens each (6 * 8 = 48)
    let active_sector_count = sector_counts.iter().filter(|&&c| c > 0).count();
    assert_eq!(
        active_sector_count, 6,
        "Exactly 6 sectors must contribute candidates to break single-sector monopoly"
    );
    for (sec, &cnt) in sector_counts.iter().enumerate() {
        if cnt > 0 {
            assert_eq!(
                cnt, 8,
                "Each active sector must contribute exactly 8 tokens, got {} for sector {}",
                cnt, sec
            );
        }
    }
}
