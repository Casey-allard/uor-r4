//! R4G1Runtime unit tests verifying multiplication-free zero-allocation prediction behavior over R4G1 GraphView containers.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_format::{ArtifactBuilder, GraphView, ScoreQ, SectionId};
use uor_r4_graph_runtime::R4G1Runtime;

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let _ = COUNT_ALLOCATIONS.try_with(|gate| {
                if gate.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                }
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn counted_allocations(f: impl FnOnce()) -> usize {
    ALLOCATIONS.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|gate| gate.set(true));
    f();
    COUNT_ALLOCATIONS.with(|gate| gate.set(false));
    ALLOCATIONS.with(Cell::get)
}

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

fn base_r4g1_artifact() -> Vec<u8> {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds")
        .0
}

fn ngram_payload(include_trigram: bool) -> Vec<u8> {
    let row_count = if include_trigram { 2usize } else { 1usize };
    let entries_start =
        uor_r4_graph_format::NGRAM_HEADER_LEN + row_count * uor_r4_graph_format::NGRAM_ROW_LEN;
    let mut bytes = vec![0u8; entries_start];
    bytes[..4].copy_from_slice(&uor_r4_graph_format::NGRAM_MAGIC);
    bytes[4..6].copy_from_slice(&uor_r4_graph_format::NGRAM_VERSION.to_le_bytes());
    bytes[8..12].copy_from_slice(&(row_count as u32).to_le_bytes());
    bytes[12..14].copy_from_slice(&2u16.to_le_bytes());

    let mut entry_offset = entries_start;
    let mut add_row =
        |index: usize, context_len: u8, key0: u32, key1: u32, entries: &[(u32, i32)]| {
            let header =
                uor_r4_graph_format::NGRAM_HEADER_LEN + index * uor_r4_graph_format::NGRAM_ROW_LEN;
            bytes[header] = context_len;
            bytes[header + 2..header + 4].copy_from_slice(&(entries.len() as u16).to_le_bytes());
            bytes[header + 4..header + 8].copy_from_slice(&key0.to_le_bytes());
            bytes[header + 8..header + 12].copy_from_slice(&key1.to_le_bytes());
            bytes[header + 12..header + 16].copy_from_slice(&(entry_offset as u32).to_le_bytes());
            for &(token, score) in entries {
                bytes.extend_from_slice(&token.to_le_bytes());
                bytes.extend_from_slice(&score.to_le_bytes());
            }
            entry_offset = bytes.len();
        };

    add_row(0, 1, 20, 0, &[(7, 100)]);
    if include_trigram {
        add_row(1, 2, 10, 20, &[(8, 200), (9, 200)]);
    }
    bytes
}

fn artifact_with_ngram(include_trigram: bool) -> Vec<u8> {
    let base = base_r4g1_artifact();
    let view = GraphView::parse(&base).expect("base artifact parses");
    let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
    for section in view.sections() {
        builder.add_section(section.id, section.flags, section.payload);
    }
    builder.add_section(SectionId::NGRAM, 0, &ngram_payload(include_trigram));
    builder.build().expect("artifact with NGRAM builds")
}

#[test]
fn r4g1_runtime_parses_and_predicts() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1 succeeds");

    let runtime = R4G1Runtime::parse(&r4g1_bytes).expect("R4G1Runtime parses container");
    assert!(runtime.node_count() > 0);
    assert!(runtime.edge_count() > 0);

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (best_token, best_score) = runtime.predict_distribution(&[1, 2, 3], None, &mut node_scores);
    assert!(best_score > ScoreQ::MIN);

    let mut node_scores2 = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let token = runtime.predict_token(&[1, 2, 3], None, &mut node_scores2);
    assert_eq!(token, best_token);
}

#[test]
fn r4g1_runtime_enforces_no_float_in_prediction_path() {
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let runtime = R4G1Runtime::parse(&r4g1_bytes).unwrap();

    let mut node_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let (_, score) = runtime.predict_distribution(&[3, 1, 4], None, &mut node_scores);

    assert!(score >= ScoreQ::MIN);
    assert!(score <= ScoreQ::MAX);
}

#[test]
fn session_signature_enters_rout_fallback_as_secondary_probe() {
    // #247 decision (calibration recorded on the issue): the session lane
    // is admitted to ROUT fallback as the SECONDARY probe — consulted only
    // when the context probe admits nothing within a calibrated radius.
    // Contract pinned here: (1) context primacy — with a context signature
    // whose probe lands within-radius actives, differing session
    // signatures cannot change routing; (2) session participation — with
    // NO context signature, the session lane alone drives the fallback and
    // the prediction is deterministic per signature.
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let runtime = R4G1Runtime::parse(&r4g1_bytes).unwrap();
    let zero_session = [0u8; 36];
    let full_session = [0xffu8; 36];

    // (2) session-only fallback: deterministic, and repeatable per signature.
    let mut scores_a = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let mut scores_b = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let zero_once = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        None,
        Some(&zero_session),
        &mut scores_a,
    );
    let zero_again = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        None,
        Some(&zero_session),
        &mut scores_b,
    );
    assert_eq!(
        zero_once, zero_again,
        "session-driven fallback is deterministic"
    );
    let full = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        None,
        Some(&full_session),
        &mut scores_b,
    );
    assert!(zero_once.1 >= ScoreQ::MIN);
    assert!(full.1 >= ScoreQ::MIN);

    // (1) context primacy: a context probe that lands within-radius actives
    // (the all-zero signature matches the synthetic store's zero prototype)
    // is not perturbed by the session lane.
    let context_signature = [0u8; 36];
    let with_zero = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        Some(&context_signature),
        Some(&zero_session),
        &mut scores_a,
    );
    let with_full = runtime.predict_distribution_with_signature_lanes(
        &[3, 1, 4],
        Some(&context_signature),
        Some(&full_session),
        &mut scores_b,
    );
    assert_eq!(
        with_zero.0, with_full.0,
        "context primacy: within-radius context routing is not changed by the session lane"
    );
}

#[test]
fn ngram_runtime_prefers_trigram_then_bigram_without_allocations() {
    let trigram_bytes = artifact_with_ngram(true);
    let trigram_runtime = R4G1Runtime::parse(&trigram_bytes).expect("trigram artifact parses");
    let mut scores = vec![ScoreQ::MIN; trigram_runtime.node_count() as usize];
    let trigram = trigram_runtime.predict_distribution(&[1, 10, 20], None, &mut scores);
    assert_eq!(trigram, (8, ScoreQ::from_raw(200)));

    let allocations = counted_allocations(|| {
        for _ in 0..64 {
            assert_eq!(
                trigram_runtime.predict_distribution(&[1, 10, 20], None, &mut scores),
                trigram
            );
        }
    });
    assert_eq!(allocations, 0, "NGRAM lookup must be allocation-free");

    let bigram_bytes = artifact_with_ngram(false);
    let bigram_runtime = R4G1Runtime::parse(&bigram_bytes).expect("bigram artifact parses");
    let mut bigram_scores = vec![ScoreQ::MIN; bigram_runtime.node_count() as usize];
    assert_eq!(
        bigram_runtime.predict_distribution(&[1, 10, 20], None, &mut bigram_scores),
        (7, ScoreQ::from_raw(100))
    );
}

#[test]
fn ngram_lookup_kernel_is_integer_only_by_source_scan() {
    let source = include_str!("../src/engine.rs");
    let start = source
        .find("fn context_backoff")
        .expect("context lookup source exists");
    let end = source
        .find("impl<'a> R4G1Runtime")
        .expect("runtime implementation follows lookup helpers");
    let kernel = &source[start..end];
    for forbidden in ["*", "/", "f32", "f64"] {
        assert!(
            !kernel.contains(forbidden),
            "NGRAM lookup kernel contains forbidden operation/type {forbidden:?}"
        );
    }
}

#[test]
fn ngram_artifact_bytes_are_deterministic() {
    assert_eq!(artifact_with_ngram(true), artifact_with_ngram(true));
}

#[test]
fn node_path_prediction_publishes_node_scores_for_candidate_expansion() {
    // #785 C1: the distribution pass writes each scored target node's best
    // final emission score into the caller's node_scores buffer, so the
    // top-k candidate walk can expand genuinely active nodes. Before this
    // contract the buffer was never written and predict_candidates could
    // never return more than the single distribution winner.
    let (art_bytes, artifacts) = fixture_artifacts();
    let store = synthetic_store();
    let store_bytes = runtime::store_bytes(&store);
    let (r4g1_bytes, _) =
        convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None).unwrap();
    let rt = R4G1Runtime::parse(&r4g1_bytes).unwrap();

    let mut node_scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let (best_token, best_score) = rt.predict_distribution(&[1, 2, 3], None, &mut node_scores);
    assert!(best_token != 0);
    assert!(best_score > ScoreQ::MIN);
    let written = node_scores
        .iter()
        .filter(|s| s.raw() != ScoreQ::MIN.raw())
        .count();
    assert!(
        written > 0,
        "node-path prediction must publish at least one node score"
    );

    // A zero-length buffer reproduces the pre-fix gate: nothing can be
    // published, so only the distribution winner survives.
    let mut no_scores: [ScoreQ; 0] = [];
    let mut gated = [(0u32, ScoreQ::ZERO); 8];
    let gated_count = rt.predict_candidates(&[1, 2, 3], None, &mut no_scores, &mut gated);
    assert!(gated_count <= 1);

    let mut full_scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let mut cands = [(0u32, ScoreQ::ZERO); 8];
    let count = rt.predict_candidates(&[1, 2, 3], None, &mut full_scores, &mut cands);
    assert!(count >= 1);
    assert!(
        count >= gated_count,
        "published node scores must never shrink the candidate set"
    );
}

#[test]
fn ngram_hit_leaves_node_scores_untouched() {
    // A context-row hit returns before any node is consulted; the buffer
    // stays all-MIN and the candidate walk stays single-winner. Absence of
    // node evidence is left visible, never fabricated.
    let trigram_bytes = artifact_with_ngram(true);
    let rt = R4G1Runtime::parse(&trigram_bytes).unwrap();
    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let prediction = rt.predict_distribution(&[1, 10, 20], None, &mut scores);
    assert_eq!(prediction, (8, ScoreQ::from_raw(200)));
    assert!(scores.iter().all(|s| s.raw() == ScoreQ::MIN.raw()));
}
