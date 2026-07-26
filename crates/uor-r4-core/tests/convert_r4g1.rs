//! Migration-converter tests: TLA3/TLA4/TLA5 + TLS1 → R4G1
//! (`transformerless::convert_r4g1`, plan §5 Phase 1).
//!
//! Fixture path: the repo TLA5 fixture artifacts plus a deterministic
//! synthetic store built in-test via `runtime::add_evidence`, so the
//! expected edge set is computable by hand. The manual end-to-end test
//! over the real `.uor-models` pair (TLA4 + legacy-u16 TLS1) is
//! `#[ignore]`d and skips silently when the files are absent.

use std::collections::{BTreeMap, BTreeSet};

use uor_r4_core::transformerless::compiler::{
    self, HammingCalibrationReport, RegionHammingCalibration, K, SIG_BYTES, SIG_WORDS, STAGES,
};
use uor_r4_core::transformerless::convert_r4g1::{
    self, class_node_index, ConversionReport, NODE_COUNT, ROOT_NODE,
};
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_format::{GraphView, SectionId};

fn fixture_artifacts() -> (Vec<u8>, compiler::Compiled) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin"))
        .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&bytes).expect("fixture artifacts parse");
    (bytes, artifacts)
}

/// Deterministic synthetic store: fixed codes populate prefix keys at
/// every grade level; `add_evidence` also writes the level-0 root prior.
///
/// Expected refinement edges (parent → child), by level:
/// - L1 `[3] [7] [11]`        → root → (0,3), (0,7), (0,11)
/// - L2 `[3,1] [3,5] [7,5] [11,5]`
///   → (0,3)→(1,1), (0,3)→(1,5), (0,7)→(1,5), (0,11)→(1,5)
/// - L3 `[3,1,4] [3,5,9] [7,5,9] [7,5,8] [11,5,8]`
///   → (1,1)→(2,4), (1,5)→(2,9), (1,5)→(2,8)  (the two `[_,5,9]`
///   keys dedup to one edge)
/// - L4 the six full codes
///   → (2,4)→(3,1), (2,4)→(3,2), (2,9)→(3,2), (2,8)→(3,2), (2,8)→(3,7)
///
/// 15 observed edges; 11 distinct class nodes have a parent, so
/// 1024 − 11 = 1013 root fallbacks; 1028 edges total.
fn synthetic_store() -> Store {
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; STAGES]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (i, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, 100 + i as u32, 1);
        runtime::add_evidence(&mut store, code, 42, 2);
    }
    store
}

fn expected_edges() -> BTreeSet<(u32, u32)> {
    let class = class_node_index;
    let mut edges = BTreeSet::new();
    for c in [3u32, 7, 11] {
        edges.insert((ROOT_NODE, class(0, c as usize)));
    }
    edges.insert((class(0, 3), class(1, 1)));
    edges.insert((class(0, 3), class(1, 5)));
    edges.insert((class(0, 7), class(1, 5)));
    edges.insert((class(0, 11), class(1, 5)));
    edges.insert((class(1, 1), class(2, 4)));
    edges.insert((class(1, 5), class(2, 9)));
    edges.insert((class(1, 5), class(2, 8)));
    edges.insert((class(2, 4), class(3, 1)));
    edges.insert((class(2, 4), class(3, 2)));
    edges.insert((class(2, 9), class(3, 2)));
    edges.insert((class(2, 8), class(3, 2)));
    edges.insert((class(2, 8), class(3, 7)));
    edges
}

fn convert_fixture() -> (Vec<u8>, Vec<u8>, Vec<u8>, Store, ConversionReport) {
    let (artifact_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1, report) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None)
            .expect("fixture conversion succeeds");
    (artifact_bytes, r4g1, store_bytes, store, report)
}

#[test]
fn converted_artifact_parses_and_verifies() {
    let (_, r4g1, _, _, report) = convert_fixture();
    let view = GraphView::parse(&r4g1).expect("converted R4G1 must pass two-stage validation");
    view.verify_cids().expect("converted R4G1 CIDs verify");
    assert_eq!(view.node_count(), Some(NODE_COUNT));
    assert_eq!(report.node_count, NODE_COUNT);
    assert_eq!(report.edge_count, 1028);
    assert_eq!(report.observed_refinement_edges, 15);
    assert_eq!(report.root_fallback_edges, 1013);
    assert_eq!(report.observed_prefix_keys, 3 + 4 + 5 + 6);
    assert_eq!(report.root_prior_entries, 7); // tokens 42, 100..=105
    assert_eq!(report.calibrated_radii, 0);
    assert_eq!(report.max_frontier_width, 32); // max observed child_len is 2
    assert_eq!(report.artifact_bytes, r4g1.len());
}

#[test]
fn head_carries_the_migration_geometry() {
    let (artifact_bytes, r4g1, _, _, _) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let head = view.head().expect("HEAD present");
    assert_eq!(head.signature_words(), 5);
    assert_eq!(head.signature_bytes(), 36);
    assert_eq!(head.depth_count(), 5);
    assert_eq!(head.node_count(), NODE_COUNT);
    assert_eq!(head.vocab_size(), 32000); // fixture TLA5 vocab
    assert_eq!(head.max_frontier_width(), 32);
    assert_eq!(head.max_candidates(), 16);
    assert_eq!(head.shortlist_size(), 8);
    assert_eq!(head.max_emission_entries(), 64);
    assert_eq!(head.max_program_steps(), 64);
    assert_eq!(head.fallback_policies(), [0; 5]); // unset, documented
    assert_eq!(head.hf_revision(), &[0u8; 20]); // not recoverable, zeroed
    assert_eq!(head.tokenizer_cid().0, [0u8; 32]);
    assert_eq!(head.corpus_construction_cid().0, [0u8; 32]);
    assert_eq!(head.corpus_certification_cid().0, [0u8; 32]);
    // teacher_cid is blake3 of the source TLA container bytes.
    assert_eq!(
        head.teacher_cid().0,
        *blake3::hash(&artifact_bytes).as_bytes()
    );
}

#[test]
fn node_records_match_the_fixed_mapping() {
    let (_, r4g1, _, _, _) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let nodes: Vec<_> = view.nodes().collect();
    assert_eq!(nodes.len(), NODE_COUNT as usize);

    // The synthetic root: depth 0, radius 0, every range empty.
    let root = nodes[ROOT_NODE as usize];
    assert_eq!(root.depth.0, 0);
    assert_eq!(root.radius.0, 0);
    assert_eq!(root.child_len, 0);
    assert_eq!(root.forward_len, 0);
    assert_eq!(root.emission_len, 0);
    assert_eq!(root.prototype_word_start, 0);
    assert_eq!(root.mask_word_start, 0);
    assert_eq!(root.flags, 0);

    // Class node (stage k, class c) → index 1 + k*256 + c, depth k+1,
    // default radius 288 without calibration.
    for (i, node) in nodes.iter().enumerate().skip(1) {
        let stage = (i - 1) / K;
        assert_eq!(node.depth.0 as usize, stage + 1, "node {i} depth");
        assert_eq!(node.radius.0, 288, "node {i} default radius");
        assert_eq!(node.flags, 0);
        assert_eq!(node.emission_len, 0);
        let w = SIG_WORDS as u32;
        assert_eq!(node.prototype_word_start, 1 + (i as u32) * w);
        assert_eq!(node.mask_word_start, 1 + (NODE_COUNT + i as u32) * w);
        // Every class node keeps at least one incoming refinement edge.
        assert!(node.forward_len >= 1, "node {i} has no parent edge");
    }
}

#[test]
fn prototype_and_mask_words_carry_the_class_signatures() {
    let (artifact_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1, _) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let view = GraphView::parse(&r4g1).unwrap();
    let rout = view.section(SectionId::ROUT).expect("ROUT present");
    let word =
        |word_index: u32| -> &[u8] { &rout[word_index as usize * 8..word_index as usize * 8 + 8] };
    // Spot-check three class nodes: prototype words = 36-byte signature
    // zero-padded to 40; mask = 0xFF over 36 bytes zero-padded to 40.
    for (stage, class) in [(0usize, 0usize), (1, 5), (3, 255)] {
        let node = class_node_index(stage, class);
        let packed = view.node(node).unwrap();
        let proto: Vec<u8> = (0..SIG_WORDS)
            .flat_map(|w| word(packed.prototype_word_start + w as u32).to_vec())
            .collect();
        assert_eq!(
            &proto[..SIG_BYTES],
            &artifacts.class_sigs[stage][class * SIG_BYTES..(class + 1) * SIG_BYTES],
            "prototype words of (stage {stage}, class {class})"
        );
        assert_eq!(&proto[SIG_BYTES..], &[0u8; 4], "prototype padding zero");
        let mask: Vec<u8> = (0..SIG_WORDS)
            .flat_map(|w| word(packed.mask_word_start + w as u32).to_vec())
            .collect();
        assert_eq!(&mask[..SIG_BYTES], &[0xFF; SIG_BYTES]);
        assert_eq!(&mask[SIG_BYTES..], &[0u8; 4], "mask padding zero");
    }
}

#[test]
fn refinement_edges_match_store_adjacency() {
    let (_, r4g1, _, _, _) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let expected = expected_edges();
    let observed_with_parent: BTreeSet<(u32, u32)> = expected.iter().copied().collect();
    // Observed = expected store edges plus one root edge per class node
    // that has no store parent.
    let mut want = observed_with_parent.clone();
    let has_parent: BTreeSet<u32> = expected.iter().map(|&(_, dst)| dst).collect();
    for stage in 0..STAGES {
        for class in 0..K {
            let node = class_node_index(stage, class);
            if !has_parent.contains(&node) {
                want.insert((ROOT_NODE, node));
            }
        }
    }
    let got: BTreeSet<(u32, u32)> = view
        .edges()
        .map(|edge| {
            assert_eq!(edge.kind, 0, "E_r refinement kind");
            assert_eq!(edge.score_q.raw(), 0);
            assert_eq!(edge.flags, 0);
            assert_eq!(edge.reserved, 0);
            (edge.src.0, edge.dst.0)
        })
        .collect();
    assert_eq!(got, want);
    assert_eq!(got.len() as u32, view.edge_count().unwrap());
}

#[test]
fn every_class_node_has_a_refinement_path_from_root() {
    let (_, r4g1, _, _, _) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let edges: Vec<(u32, u32)> = view.edges().map(|e| (e.src.0, e.dst.0)).collect();
    // Parent walk per class node: the reverse-index range of a node
    // lists its incoming edges (forward fields); any parent of a
    // stage-k node is the root (k=0) or a stage-(k-1) node, so the walk
    // reaches the root within depth steps.
    for stage in 0..STAGES {
        // Spot-check the full parent walk for a sample of classes per
        // stage (all 256 would just re-run identical structure).
        for class in [0usize, 1, 5, 137, 255] {
            let mut node = class_node_index(stage, class);
            for _ in 0..=STAGES {
                if node == ROOT_NODE {
                    break;
                }
                let packed = view.node(node).unwrap();
                assert!(packed.forward_len >= 1, "node {node} has no incoming edge");
                let first = view.reverse_edge_id(packed.forward_start).unwrap();
                let parent = edges[first as usize].0;
                // The reverse range is consistent: it points at an edge
                // whose dst is this node.
                assert_eq!(edges[first as usize].1, node);
                node = parent;
            }
            assert_eq!(node, ROOT_NODE, "class ({stage}, {class}) reaches root");
        }
    }
}

#[test]
fn exct_carries_the_raw_tls1_bytes() {
    let (_, r4g1, store_bytes, _, _) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let exct = view.section(SectionId::EXCT).expect("EXCT present");
    assert_eq!(&exct[..4], &[2, 0, 0, 0], "EXCT storage descriptor");
    assert_eq!(
        &exct[4..],
        &store_bytes[..],
        "EXCT remainder is the TLS1 carryover"
    );
    // The carryover remains readable by the legacy parser.
    let reparsed = runtime::parse_store(&exct[4..]).expect("EXCT remainder parses as TLS1");
    assert_eq!(reparsed.len(), STAGES + 1);
}

#[test]
fn emit_root_prior_matches_store_level_zero() {
    let (_, r4g1, _, store, report) = convert_fixture();
    let view = GraphView::parse(&r4g1).unwrap();
    let emit = view.section(SectionId::EMIT).expect("EMIT present");
    assert_eq!(&emit[..4], &[2, 0, 0, 0], "EMIT storage descriptor");
    let root_dist: &BTreeMap<u32, u32> = store[0].get(&[][..]).expect("root prior present");
    assert_eq!(report.root_prior_entries as usize, root_dist.len());
    let remainder = &emit[4..];
    assert_eq!(remainder.len(), root_dist.len() * 8);
    for (entry, (&token, &count)) in remainder.chunks_exact(8).zip(root_dist) {
        let got_token = i32::from_le_bytes(entry[..4].try_into().unwrap());
        let got_count = i32::from_le_bytes(entry[4..].try_into().unwrap());
        assert_eq!(got_token as u32, token);
        assert_eq!(got_count as u32, count);
    }
}

#[test]
fn conversion_is_byte_deterministic() {
    let (artifact_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (first, _) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let (second, _) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    assert_eq!(first, second, "identical inputs must give identical bytes");
    // … and both runs must be internally valid, CIDs included.
    let view = GraphView::parse(&second).unwrap();
    view.verify_cids().unwrap();
}

#[test]
fn calibration_overrides_default_radii() {
    let (artifact_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let calibration = HammingCalibrationReport {
        signature_bits: 288,
        quantile_numerator: 95,
        quantile_denominator: 100,
        regions: vec![
            RegionHammingCalibration {
                stage: 2,
                class: 5,
                mask_bits: 288,
                sample_count: 10,
                acceptance_radius: 137,
                hamming_histogram: vec![0; 289],
            },
            RegionHammingCalibration {
                stage: 0,
                class: 3,
                mask_bits: 288,
                sample_count: 4,
                acceptance_radius: 200,
                hamming_histogram: vec![0; 289],
            },
        ],
    };
    let (r4g1, report) = convert_r4g1::convert(
        &artifact_bytes,
        &artifacts,
        &store,
        &store_bytes,
        Some(&calibration),
    )
    .unwrap();
    assert_eq!(report.calibrated_radii, 2);
    let view = GraphView::parse(&r4g1).unwrap();
    assert_eq!(view.node(class_node_index(2, 5)).unwrap().radius.0, 137);
    assert_eq!(view.node(class_node_index(0, 3)).unwrap().radius.0, 200);
    assert_eq!(view.node(class_node_index(2, 6)).unwrap().radius.0, 288);
}

/// Manual end-to-end run over the real `.uor-models` pair: TLA4
/// artifacts + the legacy-u16 TLS1 store. Skips silently when the files
/// are absent (bare checkout). Run with:
///
///   cargo test -p uor-r4-core --test convert_r4g1 -- --ignored
#[test]
#[ignore]
fn convert_real_models_pair() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let base = format!("{dir}/../../.uor-models/compiled/smollm2-135m-instruct");
    let artifact_path = format!("{base}/tless_artifacts.bin");
    let store_path = format!("{base}/tless_store.bin");
    let (Ok(artifact_bytes), Ok(store_bytes)) =
        (std::fs::read(&artifact_path), std::fs::read(&store_path))
    else {
        eprintln!("skipping: {base} not present");
        return;
    };
    let artifacts = compiler::parse_artifacts(&artifact_bytes).expect("real artifacts parse");
    // The on-disk store is the pre-u32 TLS1 variant; accept either era.
    #[allow(deprecated)]
    let store = runtime::parse_store(&store_bytes)
        .or_else(|| runtime::parse_store_legacy_u16(&store_bytes))
        .expect("real store parses under one TLS1 era");
    let (r4g1, report) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None)
            .expect("real conversion succeeds");
    let view = GraphView::parse(&r4g1).expect("real R4G1 passes two-stage validation");
    view.verify_cids().expect("real R4G1 CIDs verify");
    eprintln!("real conversion report: {report:?}");
    assert_eq!(view.node_count(), Some(NODE_COUNT));
    assert_eq!(report.edge_count as usize, view.edges().count());
    // Determinism on the real pair too.
    let (r4g1_second, _) =
        convert_r4g1::convert(&artifact_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    assert_eq!(r4g1, r4g1_second);
    // EXCT carryover round-trips through the legacy parser.
    let exct = view.section(SectionId::EXCT).unwrap();
    #[allow(deprecated)]
    let reparsed = runtime::parse_store_legacy_u16(&exct[4..]).expect("EXCT legacy parse");
    let keys: usize = reparsed.iter().map(|level| level.len()).sum();
    let want_keys: usize = store.iter().map(|level| level.len()).sum();
    assert_eq!(keys, want_keys);
}
