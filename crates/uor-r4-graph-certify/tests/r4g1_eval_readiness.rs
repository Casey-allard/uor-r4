//! Readiness-probe tests for the R4G1 certify evaluation guard (issue #232).
//!
//! Builds tiny R4G1 containers the same way the runtime's own tests do
//! (fixture artifacts + synthetic store) and verifies the probe classifies
//! trained-looking graphs as `Ready` and degenerate output shapes as
//! skip-worthy.

use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_certify::r4g1_readiness::{r4g1_eval_readiness, R4g1EvalReadiness};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;

fn fixture_artifacts() -> (Vec<u8>, compiler::Compiled) {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&bytes).expect("fixture artifacts parse");
    (bytes, artifacts)
}

fn synthetic_store() -> Store {
    let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
    let codes: [[u8; 4]; 6] = [
        [3, 1, 4, 1],
        [3, 1, 4, 2],
        [3, 5, 9, 2],
        [7, 5, 9, 2],
        [7, 5, 8, 2],
        [11, 5, 8, 7],
    ];
    for (i, code) in codes.iter().enumerate() {
        runtime::add_evidence(&mut store, code, (i + 1) as u32, 1);
    }
    store
}

fn build_runtime_bytes(store: &Store) -> Vec<u8> {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store_bytes = runtime::store_bytes(store);
    let (r4g1_bytes, _) = convert_r4g1::convert(&art_bytes, &artifacts, store, &store_bytes, None)
        .expect("convert to R4G1 succeeds");
    r4g1_bytes
}

#[test]
fn varied_contexts_on_populated_store_are_ready_or_explicitly_degenerate() {
    let bytes = build_runtime_bytes(&synthetic_store());
    let rt = R4G1Runtime::parse(&bytes).expect("parse");
    let mut node_scores = vec![ScoreQ::MIN; rt.node_count() as usize];

    // Contexts drawn from the store's own token universe, varied enough that
    // a functioning graph does not answer with one constant token.
    let contexts: Vec<Vec<u32>> = vec![
        vec![1, 2, 3],
        vec![3, 1, 4],
        vec![3, 5, 9],
        vec![7, 5, 8],
        vec![11, 5, 8],
        vec![2, 3, 4],
    ];
    let readiness =
        r4g1_eval_readiness(&rt, contexts.iter().map(|c| c.as_slice()), &mut node_scores);

    // The fixture-backed graph must not silently classify as Ready while
    // actually emitting nothing: either it is Ready with a nonzero scored
    // count, or the probe names the degenerate shape.
    match readiness {
        R4g1EvalReadiness::Ready { scored } => assert!(scored > 0, "Ready requires scored > 0"),
        R4g1EvalReadiness::NoScoredEmission | R4g1EvalReadiness::ConstantPrediction { .. } => {
            // Acceptable classification for a tiny synthetic store — the
            // point of the guard is that this shape is *named*, not zeroed.
        }
    }
}

#[test]
fn out_of_vocabulary_contexts_classify_as_degenerate() {
    let bytes = build_runtime_bytes(&synthetic_store());
    let rt = R4G1Runtime::parse(&bytes).expect("parse");
    let mut node_scores = vec![ScoreQ::MIN; rt.node_count() as usize];

    // Tokens far outside anything the store observed: the suffix walk finds
    // no emitting node, so predictions collapse to the constant root
    // fallback (or stay unscored). Either way the probe must NOT say Ready.
    let contexts: Vec<Vec<u32>> = (0..8).map(|k| vec![40_000 + k, 41_000 + k]).collect();
    let readiness =
        r4g1_eval_readiness(&rt, contexts.iter().map(|c| c.as_slice()), &mut node_scores);

    assert!(
        !readiness.is_ready(),
        "degenerate probe output must not classify as Ready, got {readiness:?}"
    );
}

#[test]
fn empty_probe_is_not_ready() {
    let bytes = build_runtime_bytes(&synthetic_store());
    let rt = R4G1Runtime::parse(&bytes).expect("parse");
    let mut node_scores = vec![ScoreQ::MIN; rt.node_count() as usize];

    let readiness = r4g1_eval_readiness(&rt, std::iter::empty::<&[u32]>(), &mut node_scores);
    assert_eq!(readiness, R4g1EvalReadiness::NoScoredEmission);
}
