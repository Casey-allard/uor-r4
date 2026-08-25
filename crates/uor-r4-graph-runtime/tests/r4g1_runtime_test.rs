//! R4G1Runtime unit tests verifying multiplication-free zero-allocation prediction behavior over R4G1 GraphView containers.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_core::transformerless::convert_r4g1;
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_format::{
    ArtifactBuilder, FormatError, GraphView, ScoreQ, SectionId, build_psi_bag_table,
    build_skipmix_table,
};
use uor_r4_graph_runtime::{
    R4G1Runtime, ServedCandidate, ServedCandidateSource, SignatureRoutingSource,
};

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

fn artifact_with_serving_sections(skmx: Option<&[u8]>, psib: Option<&[u8]>) -> Vec<u8> {
    let base = artifact_with_ngram(true);
    let view = GraphView::parse(&base).expect("NGRAM artifact parses");
    let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
    for section in view.sections() {
        builder.add_section(section.id, section.flags, section.payload);
    }
    if let Some(payload) = skmx {
        builder.add_section(SectionId::SKMX, 0, payload);
    }
    if let Some(payload) = psib {
        builder.add_section(SectionId::PSIB, 0, payload);
    }
    builder
        .build()
        .expect("artifact with serving sections builds")
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
    let (_, session_trace) = runtime.predict_distribution_with_signature_lanes_traced(
        &[3, 1, 4],
        None,
        Some(&zero_session),
        &mut scores_a,
    );
    assert!(session_trace.session_probe_attempted);
    assert!(session_trace.session_admitted_nodes > 0);
    assert_eq!(
        session_trace.selected_source,
        SignatureRoutingSource::SessionSignature
    );

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
    let (_, context_trace) = runtime.predict_distribution_with_signature_lanes_traced(
        &[3, 1, 4],
        Some(&context_signature),
        Some(&full_session),
        &mut scores_a,
    );
    assert!(context_trace.context_probe_attempted);
    assert!(context_trace.context_admitted_nodes > 0);
    assert!(!context_trace.session_probe_attempted);
    assert_eq!(
        context_trace.selected_source,
        SignatureRoutingSource::ContextSignature
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
    // #787 E-c: the naive per-file scan this test carried was consolidated
    // onto the shared canonical scanner (comment- and literal-aware value
    // `*` `/` `%` detection, method-form needles, float tokens). The
    // kernel region sanctions no allowances.
    let outcome = uor_r4_core::transformerless::source_scan::scan_for_forbidden_arith_and_floats(
        &source[start..end],
    );
    assert!(
        outcome.offenders.is_empty(),
        "NGRAM lookup kernel contains forbidden operations/types:\n{}",
        outcome.offenders.join("\n")
    );
    assert!(
        outcome.allowed.is_empty(),
        "no p4-allow markers are sanctioned in the NGRAM kernel:\n{}",
        outcome.allowed.join("\n")
    );
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

#[test]
fn ngram_hit_surfaces_row_alternatives_as_candidates() {
    // #785 C1: on a context-row hit the winning row's alternative entries
    // become candidates, so multi-candidate decoding exists on the n-gram
    // tier too. The fixture trigram row holds (8, 200) and (9, 200); the
    // single-winner distribution keeps 8 (tie broken by lower token), and
    // the candidate walk must now surface 9 as well.
    let trigram_bytes = artifact_with_ngram(true);
    let rt = R4G1Runtime::parse(&trigram_bytes).unwrap();
    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let mut cands = [(0u32, ScoreQ::ZERO); 8];
    let count = rt.predict_candidates(&[1, 10, 20], None, &mut scores, &mut cands);
    assert!(count >= 2, "row alternatives must be surfaced, got {count}");
    assert_eq!(cands[0], (8, ScoreQ::from_raw(200)));
    assert!(
        cands[..count].iter().any(|&(token, _)| token == 9),
        "the row's alternative continuation must appear"
    );
    // The buffer contract is unchanged: a row hit still consults no nodes.
    assert!(scores.iter().all(|s| s.raw() == ScoreQ::MIN.raw()));
}

#[test]
fn absent_skipmix_sections_preserve_base_candidate_identity() {
    let bytes = artifact_with_ngram(true);
    let runtime = R4G1Runtime::parse(&bytes).expect("base artifact parses");
    assert_eq!(runtime.skipmix_tables_present(), (false, false));

    let mut base_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let mut base = [(0u32, ScoreQ::MIN); 8];
    let base_len = runtime.predict_candidates(&[1, 10, 20], None, &mut base_scores, &mut base);

    let mut served_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let served = runtime.predict_served_candidates(&[1, 10, 20], None, &mut served_scores);
    assert_eq!(served.len(), base_len);
    assert_eq!(served.attribution(), None);
    for (served, &(token, score)) in served.ranked().iter().zip(&base[..base_len]) {
        assert_eq!((served.token, served.score), (token, score));
        assert_eq!(served.source, ServedCandidateSource::Base);
        assert!(!served.skmx_contributed);
        assert!(!served.psib_contributed);
    }
}

#[test]
fn skmx_candidate_injection_reaches_the_normative_winner_without_allocations() {
    let skmx = build_skipmix_table(&[(10, 20, vec![(42, 1_000)])]).expect("SKMX builds");
    let bytes = artifact_with_serving_sections(Some(&skmx), None);
    let runtime = R4G1Runtime::parse(&bytes).expect("SKMX artifact parses");
    assert_eq!(runtime.skipmix_tables_present(), (true, false));
    let mut scores = vec![ScoreQ::MIN; runtime.node_count() as usize];

    let served = runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
    let winner = served.winner().expect("candidate exists");
    assert_eq!(winner.token, 42);
    assert_eq!(winner.score, ScoreQ::from_raw(1_000));
    assert_eq!(winner.source, ServedCandidateSource::Skipmix);
    assert!(winner.skmx_contributed);
    assert!(!winner.psib_contributed);
    let attribution = served.attribution().expect("promotion is attributed");
    assert_eq!(attribution.base_token, 8);
    assert_eq!(attribution.promoted_token, 42);
    assert_eq!(attribution.contribution, ScoreQ::from_raw(1_000));
    assert!(attribution.skmx_contributed);
    assert!(!attribution.psib_contributed);

    let allocations = counted_allocations(|| {
        scores.fill(ScoreQ::MIN);
        let repeated = runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
        assert_eq!(repeated, served);
    });
    assert_eq!(allocations, 0, "served selection must not allocate");
}

#[test]
fn joint_row_presence_gates_psib_and_absent_joint_row_uses_it() {
    let joint = build_skipmix_table(&[(10, 20, vec![(42, 5)])]).expect("SKMX builds");
    let psib = build_psi_bag_table(&[(10, vec![(43, 5_000)])]).expect("PSIB builds");
    let joint_bytes = artifact_with_serving_sections(Some(&joint), Some(&psib));
    let joint_runtime = R4G1Runtime::parse(&joint_bytes).expect("joint artifact parses");
    let mut scores = vec![ScoreQ::MIN; joint_runtime.node_count() as usize];
    let served = joint_runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
    assert_eq!(served.winner().map(|candidate| candidate.token), Some(42));
    assert!(
        served
            .ranked()
            .iter()
            .all(|candidate| candidate.token != 43),
        "a present joint row must suppress PSIB even for a candidate absent from that row"
    );

    let empty_joint = build_skipmix_table(&[]).expect("empty SKMX builds");
    let fallback_bytes = artifact_with_serving_sections(Some(&empty_joint), Some(&psib));
    let fallback_runtime = R4G1Runtime::parse(&fallback_bytes).expect("fallback artifact parses");
    let mut scores = vec![ScoreQ::MIN; fallback_runtime.node_count() as usize];
    let fallback = fallback_runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
    let fallback_winner = fallback.winner().expect("PSIB supplies a candidate");
    assert_eq!(fallback_winner.token, 43);
    assert_eq!(fallback_winner.source, ServedCandidateSource::Skipmix);
    assert!(!fallback_winner.skmx_contributed);
    assert!(fallback_winner.psib_contributed);
    let attribution = fallback.attribution().expect("PSIB promotion attribution");
    assert!(!attribution.skmx_contributed);
    assert!(attribution.psib_contributed);
}

#[test]
fn skipmix_uses_distinct_tokens_in_the_newest_eight_token_compiler_window() {
    let skmx = build_skipmix_table(&[(99, 20, vec![(42, 1_000)])]).expect("SKMX builds");
    let bytes = artifact_with_serving_sections(Some(&skmx), None);
    let runtime = R4G1Runtime::parse(&bytes).expect("SKMX artifact parses");

    let mut excluded_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let excluded = runtime.predict_served_candidates(
        &[99, 30, 31, 32, 33, 34, 35, 10, 20],
        None,
        &mut excluded_scores,
    );
    assert_eq!(excluded.winner().map(|candidate| candidate.token), Some(8));
    assert_eq!(excluded.attribution(), None);

    let mut included_scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let included = runtime.predict_served_candidates(
        &[99, 30, 31, 32, 33, 34, 10, 20],
        None,
        &mut included_scores,
    );
    assert_eq!(included.winner().map(|candidate| candidate.token), Some(42));
}

#[test]
fn runtime_skipmix_winner_and_attribution_match_an_independent_window_combinator() {
    type JointRow = (u32, u32, Vec<(u32, i32)>);
    type PsiRow = (u32, Vec<(u32, i32)>);

    fn independent_winner(
        base: &[ServedCandidate],
        context: &[u32],
        joint: &[JointRow],
        psi: &[PsiRow],
    ) -> (u32, i32) {
        let window = &context[context.len().saturating_sub(compiler::WINDOW)..];
        let mut unique = window.to_vec();
        unique.sort_unstable();
        unique.dedup();
        let last = *context.last().expect("representative context is nonempty");
        let mut candidates: Vec<u32> = base.iter().map(|candidate| candidate.token).collect();
        for &content in &unique {
            if let Some((_, _, entries)) = joint
                .iter()
                .find(|&&(row_content, row_last, _)| row_content == content && row_last == last)
            {
                candidates.extend(entries.iter().map(|&(token, _)| token));
            } else if let Some((_, entries)) =
                psi.iter().find(|&&(row_content, _)| row_content == content)
            {
                candidates.extend(entries.iter().map(|&(token, _)| token));
            }
        }
        candidates.sort_unstable();
        candidates.dedup();

        let contribution = |candidate: u32| {
            unique.iter().fold(0i32, |sum, &content| {
                let score = if let Some((_, _, entries)) = joint
                    .iter()
                    .find(|&&(row_content, row_last, _)| row_content == content && row_last == last)
                {
                    entries
                        .iter()
                        .find_map(|&(token, score)| (token == candidate).then_some(score))
                } else {
                    psi.iter()
                        .find(|&&(row_content, _)| row_content == content)
                        .and_then(|(_, entries)| {
                            entries
                                .iter()
                                .find_map(|&(token, score)| (token == candidate).then_some(score))
                        })
                };
                sum.saturating_add(score.unwrap_or(0))
            })
        };

        candidates
            .into_iter()
            .filter_map(|token| {
                let score = contribution(token);
                (score > 0).then_some((token, score))
            })
            .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
            .unwrap_or_else(|| (base[0].token, base[0].score.raw()))
    }

    let joint: Vec<JointRow> = vec![
        (10, 20, vec![(42, 300), (43, 100)]),
        (11, 20, vec![(42, 200)]),
        (12, 20, vec![(44, 50)]),
        (99, 20, vec![(45, 9_999)]),
    ];
    let psi: Vec<PsiRow> = vec![(10, vec![(43, 900)]), (13, vec![(43, 700)])];
    let context = [99, 10, 10, 11, 12, 13, 14, 10, 20];

    let base_bytes = artifact_with_ngram(true);
    let base_runtime = R4G1Runtime::parse(&base_bytes).expect("base artifact parses");
    let mut base_scores = vec![ScoreQ::MIN; base_runtime.node_count() as usize];
    let base = base_runtime.predict_served_candidates(&context, None, &mut base_scores);
    let expected = independent_winner(base.ranked(), &context, &joint, &psi);

    let skmx = build_skipmix_table(&joint).expect("representative SKMX builds");
    let psib = build_psi_bag_table(&psi).expect("representative PSIB builds");
    let bytes = artifact_with_serving_sections(Some(&skmx), Some(&psib));
    let runtime = R4G1Runtime::parse(&bytes).expect("representative artifact parses");
    let mut scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let served = runtime.predict_served_candidates(&context, None, &mut scores);
    let winner = served.winner().expect("runtime winner");
    assert_eq!((winner.token, winner.score.raw()), expected);
    assert_eq!(winner.source, ServedCandidateSource::Skipmix);
    assert!(winner.skmx_contributed);
    assert!(winner.psib_contributed);
    let attribution = served.attribution().expect("promotion attribution");
    assert_eq!(
        attribution.base_token,
        base.winner().expect("base winner").token
    );
    assert_eq!(attribution.promoted_token, expected.0);
    assert_eq!(attribution.contribution.raw(), expected.1);
    assert!(attribution.skmx_contributed);
    assert!(attribution.psib_contributed);
    assert!(
        served
            .ranked()
            .iter()
            .all(|candidate| candidate.token != 45),
        "content outside the newest compiler window must not inject a candidate"
    );
}

#[test]
fn served_candidate_ties_break_by_ascending_token() {
    let skmx = build_skipmix_table(&[(10, 20, vec![(42, 500)]), (20, 20, vec![(41, 500)])])
        .expect("SKMX builds");
    let bytes = artifact_with_serving_sections(Some(&skmx), None);
    let runtime = R4G1Runtime::parse(&bytes).expect("SKMX artifact parses");
    let mut scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let served = runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
    assert_eq!(served.ranked()[0].token, 41);
    assert_eq!(served.ranked()[1].token, 42);
    assert_eq!(served.ranked()[0].score, served.ranked()[1].score);
}

#[test]
fn learned_special_token_is_a_normative_candidate_not_silently_dropped() {
    let skmx = build_skipmix_table(&[(10, 20, vec![(2, 500)])]).expect("SKMX builds");
    let bytes = artifact_with_serving_sections(Some(&skmx), None);
    let runtime = R4G1Runtime::parse(&bytes).expect("SKMX artifact parses");
    let mut scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let served = runtime.predict_served_candidates(&[1, 10, 20], None, &mut scores);
    let winner = served.winner().expect("a learned winner");
    assert_eq!(
        winner.token, 2,
        "EOS remains a valid teacher-argmax outcome"
    );
    assert_eq!(winner.source, ServedCandidateSource::Skipmix);
}

#[test]
fn runtime_parse_rejects_malformed_or_overwide_serving_tables() {
    let malformed_skmx = artifact_with_serving_sections(Some(&[0u8; 20]), None);
    let error = R4G1Runtime::parse(&malformed_skmx).expect_err("malformed SKMX must fail closed");
    assert_eq!(error.reason, FormatError::SkipmixBadMagic);

    let malformed_psib = artifact_with_serving_sections(None, Some(&[0u8; 16]));
    let error = R4G1Runtime::parse(&malformed_psib).expect_err("malformed PSIB must fail closed");
    assert_eq!(error.reason, FormatError::PsiBagBadMagic);

    let entries: Vec<(u32, i32)> = (3..68).map(|token| (token, 1)).collect();
    let overwide = build_psi_bag_table(&[(10, entries)]).expect("wide PSIB is format-valid");
    let overwide_bytes = artifact_with_serving_sections(None, Some(&overwide));
    let error = R4G1Runtime::parse(&overwide_bytes)
        .expect_err("serving row above the fixed work bound must fail closed");
    assert_eq!(error.reason, FormatError::PsiBagInvalidRow);

    let mut overprobe =
        build_skipmix_table(&[(10, 20, vec![(42, 1)])]).expect("bounded SKMX is format-valid");
    overprobe[12..14].copy_from_slice(&33u16.to_le_bytes());
    let overprobe_bytes = artifact_with_serving_sections(Some(&overprobe), None);
    let error = R4G1Runtime::parse(&overprobe_bytes)
        .expect_err("serving probe above the fixed work bound must fail closed");
    assert_eq!(error.reason, FormatError::SkipmixProbeExceeded);
}

#[test]
fn transitive_skipmix_lookup_kernel_is_integer_only_by_source_scan() {
    let source = include_str!("../../uor-r4-graph-format/src/skipmix.rs");
    let entry_start = source
        .find("impl<'a> SkipEntries<'a>")
        .expect("entry lookup implementation exists");
    let entry_end = source[entry_start..]
        .find("pub struct SkipEntriesIter")
        .map(|offset| entry_start + offset)
        .expect("entry iterator follows lookup implementation");

    let psi_start = source
        .find("    fn row(&self, index: usize)")
        .expect("PSIB lookup implementation exists");
    let psi_end = source[psi_start..]
        .find("// SKMX")
        .map(|offset| psi_start + offset)
        .expect("SKMX follows PSIB lookup implementation");

    let joint_start = source
        .find("    fn slot_row(&self, index: u32)")
        .expect("SKMX lookup implementation exists");
    let joint_end = source[joint_start..]
        .find("// Compiler-side builders")
        .map(|offset| joint_start + offset)
        .expect("builders follow SKMX lookup implementation");

    for (name, region) in [
        ("entries", &source[entry_start..entry_end]),
        ("PSIB", &source[psi_start..psi_end]),
        ("SKMX", &source[joint_start..joint_end]),
    ] {
        let outcome =
            uor_r4_core::transformerless::source_scan::scan_for_forbidden_arith_and_floats(region);
        assert!(
            outcome.offenders.is_empty(),
            "{name} lookup contains forbidden operations or types:\n{}",
            outcome.offenders.join("\n")
        );
        assert!(
            outcome.allowed.is_empty(),
            "{name} lookup sanctions no source-scan allowances"
        );
    }

    // The lookup tables are only the leaves of the normative hot path. Scan
    // the complete new fixed-capacity contribution/ranking layer as well, so
    // a multiply, divide, or float cannot hide between a clean lookup and the
    // served token.
    let runtime_source = include_str!("../src/engine.rs");
    let combinator_start = runtime_source
        .find("fn unique_tokens_in_newest_compiler_window")
        .expect("served-candidate combinator exists");
    let combinator_end = runtime_source[combinator_start..]
        .find("impl<'a> R4G1Runtime<'a>")
        .map(|offset| combinator_start + offset)
        .expect("runtime impl follows served-candidate helpers");
    let selector_start = runtime_source
        .find("    pub fn predict_served_candidates(")
        .expect("normative served-candidate selector exists");
    let selector_end = runtime_source[selector_start..]
        .find("\n}\n\n#[cfg(test)]")
        .map(|offset| selector_start + offset)
        .expect("tests follow normative served-candidate selector");
    for (name, region) in [
        (
            "served-candidate contribution and ranking",
            &runtime_source[combinator_start..combinator_end],
        ),
        (
            "normative served-candidate selector",
            &runtime_source[selector_start..selector_end],
        ),
    ] {
        let outcome =
            uor_r4_core::transformerless::source_scan::scan_for_forbidden_arith_and_floats(region);
        assert!(
            outcome.offenders.is_empty(),
            "{name} contains forbidden operations or types:\n{}",
            outcome.offenders.join("\n")
        );
        assert!(
            outcome.allowed.is_empty(),
            "{name} sanctions no source-scan allowances"
        );
    }
}

/// Rebuild the base converter artifact with selected section payloads
/// replaced, preserving section order and alignment.
fn artifact_with_replaced_sections(replacements: &[(SectionId, Vec<u8>)]) -> Vec<u8> {
    let base = base_r4g1_artifact();
    let view = GraphView::parse(&base).expect("base artifact parses");
    let mut builder = ArtifactBuilder::new(view.header().alignment_log2);
    for section in view.sections() {
        match replacements.iter().find(|(id, _)| *id == section.id) {
            Some((_, payload)) => {
                builder.add_section(section.id, section.flags, payload);
            }
            None => {
                builder.add_section(section.id, section.flags, section.payload);
            }
        }
    }
    builder.build().expect("rebuilt artifact builds")
}

#[test]
fn exct_container_bytes_are_never_scored_as_emission_pairs() {
    // #785 C1c seeded violation: EXCT is a storage descriptor plus a raw
    // store container, never a (token, score_q) pair list. The old
    // node-0 fallback preferred EXCT and pair-scanned it, so a byte
    // pattern that happens to decode as a valid token id with a
    // near-saturated score would win every step — exactly the failure
    // observed live on converter bundles carrying a multi-megabyte TLS1
    // container. Seed one such pattern and prove it can never surface.
    let mut poisoned_exct = vec![2u8, 0, 0, 0];
    poisoned_exct.extend_from_slice(&4242u32.to_le_bytes());
    poisoned_exct.extend_from_slice(&(i32::MAX - 1).to_le_bytes());
    let bytes = artifact_with_replaced_sections(&[(SectionId::EXCT, poisoned_exct)]);
    let rt = R4G1Runtime::parse(&bytes).expect("poisoned-EXCT artifact parses");

    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let (token, score) = rt.predict_distribution(&[1, 2, 3], None, &mut scores);
    assert_ne!(token, 4242, "EXCT container bytes must never be emitted");
    assert!(
        score.raw() < 1_000_000,
        "no near-saturated garbage score may survive, got {}",
        score.raw()
    );

    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let mut cands = [(0u32, ScoreQ::ZERO); 8];
    let count = rt.predict_candidates(&[1, 2, 3], None, &mut scores, &mut cands);
    assert!(
        cands[..count].iter().all(|&(t, _)| t != 4242),
        "EXCT container bytes must never enter the candidate walk"
    );
}

#[test]
fn scored_flavor_reads_only_per_node_emission_ranges() {
    // #785 C1c seeded violation, scored flavor: when any node carries a
    // per-node emission range, the EMIT remainder is a root-prior block
    // plus per-region lists — whole-remainder reads (old node-0 and
    // last-resort fallbacks) would score arbitrary header bytes. Craft a
    // remainder whose first pseudo-pair is a valid-looking token at a
    // near-saturated score, wire node 1's range past it, and prove only
    // the ranged entry is ever read.
    let base = base_r4g1_artifact();
    let base_view = GraphView::parse(&base).expect("base artifact parses");
    let mut node_payload = base_view
        .section(SectionId::NODE)
        .expect("NODE section present")
        .to_vec();
    // Node 1's record starts at PACKED_NODE_LEN; emission_start is at
    // record offset 12 (u32), emission_len at 16 (u16).
    let record = uor_r4_graph_format::PACKED_NODE_LEN;
    node_payload[record + 12..record + 16].copy_from_slice(&8u32.to_le_bytes());
    node_payload[record + 16..record + 18].copy_from_slice(&8u16.to_le_bytes());

    let mut emit = vec![2u8, 0, 0, 0];
    emit.extend_from_slice(&49000u32.to_le_bytes()); // poison pseudo-pair
    emit.extend_from_slice(&(i32::MAX - 1).to_le_bytes());
    emit.extend_from_slice(&7u32.to_le_bytes()); // node 1's real entry
    emit.extend_from_slice(&500i32.to_le_bytes());

    let mut poisoned_exct = vec![2u8, 0, 0, 0];
    poisoned_exct.extend_from_slice(&49000u32.to_le_bytes());
    poisoned_exct.extend_from_slice(&(i32::MAX - 1).to_le_bytes());

    let bytes = artifact_with_replaced_sections(&[
        (SectionId::NODE, node_payload),
        (SectionId::EMIT, emit),
        (SectionId::EXCT, poisoned_exct),
    ]);
    let rt = R4G1Runtime::parse(&bytes).expect("scored-flavor artifact parses");

    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let (token, score) = rt.predict_distribution(&[1, 2, 3], None, &mut scores);
    assert_ne!(token, 49000, "whole-remainder reads must be gated off");
    assert!(
        score.raw() < 1_000_000,
        "no near-saturated garbage score may survive, got {}",
        score.raw()
    );

    let mut scores = vec![ScoreQ::MIN; rt.node_count() as usize];
    let mut cands = [(0u32, ScoreQ::ZERO); 8];
    let count = rt.predict_candidates(&[1, 2, 3], None, &mut scores, &mut cands);
    assert!(
        cands[..count].iter().all(|&(t, _)| t != 49000),
        "poison pseudo-pair must never enter the candidate walk"
    );
}
