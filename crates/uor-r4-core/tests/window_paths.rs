//! Fast integration witnesses for the migration seams: the corpus-free
//! window path (kernel and plain forms identical to the corpus path), the
//! artifact container roundtrip (PROOF.md P5(a) as a unit test; TLA7
//! since the #327 re-pin, 2026-08-01), and the TLS1 store container.

use uor_r4_core::transformerless::compiler::{self, Corpus, SIG_BYTES, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime::{self, OpKernel, Store};

fn fixture() -> (compiler::Compiled, Corpus) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).unwrap();
    let art = compiler::parse_artifacts(&bytes).expect("fixture container parses");
    let c = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("fixture corpus loads");
    (art, c)
}

#[test]
fn container_roundtrip_byte_identical() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin")).unwrap();
    let art = compiler::parse_artifacts(&bytes).expect("parse");
    assert_eq!(
        compiler::artifact_bytes(&art),
        bytes,
        "parse → serialize is byte-identical (P5(a) save → load → save)"
    );
    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("{dir}/tests/fixtures/baseline_kappa.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        format!("blake3:{}", blake3::hash(&bytes).to_hex()),
        baseline["bundle_derived_macos"]["container"]["kappa"]
            .as_str()
            .unwrap(),
        "fixture container matches the baseline κ"
    );
}

#[test]
fn window_path_matches_corpus_path() {
    let (art, c) = fixture();
    let rot = compiler::derive_rotations();
    let mut k = OpKernel::default();
    let mut rt = runtime::Runtime::new(&art);
    let mut checked = 0usize;
    for i in WINDOW - 1..c.n {
        // positions whose full WINDOW history is in-story
        if (1..WINDOW).any(|back| c.story[i - back] != c.story[i]) {
            continue;
        }
        let window: Vec<u32> = ((i + 1 - WINDOW)..=i).map(|idx| c.input[idx]).collect();
        assert_eq!(
            runtime::bundle_window_plain(&art, &rot, &window),
            runtime::bundle_plain(&art, &rot, &c, i),
            "plain bundle at position {i}"
        );
        assert_eq!(
            runtime::bundle_window_kernel(&mut k, &art, &rot, &window),
            runtime::bundle_plain(&art, &rot, &c, i),
            "kernel bundle at position {i}"
        );
        assert_eq!(
            rt.assign_window(&window),
            runtime::code_plain(&art, &rot, &c, i),
            "graded class code at position {i}"
        );
        checked += 1;
        if checked == 256 {
            break;
        }
    }
    assert_eq!(checked, 256, "enough full-history positions sampled");
}

#[test]
fn store_container_roundtrip() {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    store[0].entry(vec![]).or_default().insert(7, 42);
    store[0].entry(vec![]).or_default().insert(9, 3);
    store[2].entry(vec![5, 6]).or_default().insert(11, 2);
    store[4].entry(vec![1, 2, 3, 4]).or_default().insert(12, 1);
    let bytes = runtime::store_bytes(&store);
    assert!(bytes.starts_with(b"TLS1"));
    let back = runtime::parse_store(&bytes).expect("TLS1 parses");
    assert_eq!(back, store, "store bytes → parse is the identity");
    assert!(
        runtime::parse_store(&bytes[..bytes.len() - 1]).is_none(),
        "truncated container rejected"
    );
    assert!(
        runtime::parse_store(b"badmagic").is_none(),
        "bad magic rejected"
    );
}

#[test]
fn predict_witness_depth_and_count() {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    store[0].entry(vec![]).or_default().insert(1, 10);
    store[1].entry(vec![9]).or_default().insert(2, 5);
    let hit = runtime::predict_witness_plain(&store, &[9, 0, 0, 0]);
    assert_eq!(
        (hit.token, hit.depth, hit.count),
        (2, 1, 5),
        "deepest populated class answers with its evidence count"
    );
    let miss = runtime::predict_witness_plain(&store, &[7, 0, 0, 0]);
    assert_eq!(
        (miss.token, miss.depth, miss.count),
        (1, 0, 10),
        "backoff reaches level 0"
    );
}

#[test]
fn predict_witness_handles_empty_store_without_panic() {
    use std::collections::HashMap;

    let store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    let code = [0u8; STAGES];
    let priors: HashMap<u32, u32> = HashMap::new();

    let pred_plain = runtime::predict_witness_plain(&store, &code);
    assert_eq!(pred_plain, runtime::Prediction::default());

    let pred_plain_with_priors = runtime::predict_witness_plain_with_priors(&store, &code, &priors);
    assert_eq!(pred_plain_with_priors, runtime::Prediction::default());
}

#[test]
fn test_generative_priors() {
    use std::collections::HashMap;

    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    // Token 1 has count 10 (higher default)
    store[0].entry(vec![]).or_default().insert(1, 10);
    // Token 2 has count 5 (lower default)
    store[0].entry(vec![]).or_default().insert(2, 5);

    let code = [0u8; STAGES];

    // 1. Default prediction: selects token 1
    let default_pred = runtime::predict_witness_plain(&store, &code);
    assert_eq!(default_pred.token, 1);

    // 2. Prior prediction: boost token 2 by prior score of 1 (equivalent to +100 in score)
    let mut priors = HashMap::new();
    priors.insert(2, 1u32);

    let prior_pred = runtime::predict_witness_plain_with_priors(&store, &code, &priors);
    assert_eq!(
        prior_pred.token, 2,
        "Semantic prior successfully shifted argmax to token 2"
    );
}

#[test]
fn assign_memberships_plain_is_bounded_and_keeps_nearest_floor() {
    let (art, c) = fixture();
    let rot = compiler::derive_rotations();
    let sig = runtime::sig_plain(&art, &runtime::bundle_plain(&art, &rot, &c, WINDOW));
    let (nearest, by_depth) = runtime::assign_memberships_plain(&art, &sig);

    assert_eq!(nearest, runtime::assign_plain(&art, &sig));
    assert_eq!(by_depth.len(), STAGES + 1);
    assert_eq!(by_depth[0], vec![Vec::<u8>::new()]);

    for depth in 1..=STAGES {
        assert!(
            !by_depth[depth].is_empty(),
            "depth {depth} must have memberships"
        );
        assert!(
            by_depth[depth].len() <= 3,
            "depth {depth} exceeds top-M bound"
        );
        assert!(
            by_depth[depth].iter().all(|key| key.len() == depth),
            "depth {depth} membership key shape"
        );
        assert!(
            by_depth[depth]
                .iter()
                .any(|key| key.as_slice() == &nearest[..depth]),
            "nearest-class prefix must remain as fallback floor at depth {depth}"
        );
    }

    let zero_sig = [0u8; SIG_BYTES];
    let (nearest_zero, by_depth_zero) = runtime::assign_memberships_plain(&art, &zero_sig);
    for depth in 1..=STAGES {
        assert!(
            by_depth_zero[depth]
                .iter()
                .any(|key| key.as_slice() == &nearest_zero[..depth]),
            "fallback floor also holds for synthetic signatures at depth {depth}"
        );
    }
}

#[test]
fn add_evidence_multi_uses_fallback_when_depth_memberships_missing() {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    let code = [1u8, 2, 3, 4];
    let by_depth: Vec<Vec<Vec<u8>>> = (0..=STAGES).map(|_| Vec::new()).collect();

    runtime::add_evidence_multi(&mut store, &by_depth, &code, 11, 7);

    assert_eq!(store[0][&vec![]][&11], 7);
    for depth in 1..=STAGES {
        let key = code[..depth].to_vec();
        assert_eq!(
            store[depth][&key][&11], 7,
            "depth {depth} fallback key used"
        );
    }
}
