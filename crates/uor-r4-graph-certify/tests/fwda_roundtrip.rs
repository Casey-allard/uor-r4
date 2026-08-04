//! FWDA forward-anchor section round-trip and infill-fusion tests
//! (issue #399): the compile loop matches the Gate C instrumentation
//! law, rows survive the artifact byte format exactly, and
//! `score_candidates_infill` reproduces the measured f64 product-fusion
//! reference on a hand-computed example while staying byte-identical to
//! `score_candidates_coded` whenever the channel is off.

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler::{Corpus, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    compile_forward_anchor_rows, emit_scored_r4g1, ContextRow, EmissionTables, ForwardAnchorRow,
    QuantizationErrorStats, ScoredGraphSections, Smoothing,
};
use uor_r4_graph_certify::score_runtime::{GraphScorer, RegionParams};
use uor_r4_graph_compiler::induction::Observation;
use uor_r4_graph_format::{GraphView, ScoreQ};

const VOCAB: u32 = 100;
const ROOT_FLOOR_RAW: i32 = -500_000;

fn observation(position: usize, corpus: &Corpus) -> Observation {
    Observation {
        position: position as u32,
        sample: [0; 32],
        vector: Vec::new(),
        sig: [0; SIG_BYTES],
        prev: corpus.input[position],
        next: corpus.next[position],
    }
}

/// Two identical eight-token stories: story-relative positions run zero
/// through seven, so the stride-four anchors sit at story positions
/// three and seven in each story.
fn two_story_corpus() -> Corpus {
    let n = 16;
    let story: Vec<u32> = (0..n).map(|i| (i / 8) as u32).collect();
    let input: Vec<u32> = (0..n).map(|i| 10 + (i % 8) as u32).collect();
    let next: Vec<u32> = (0..n).map(|i| 11 + (i % 8) as u32).collect();
    Corpus {
        n,
        stories: 2,
        story,
        input,
        next,
        t_argmax: vec![0; n],
        top_tokens: vec![[0; 8]; n],
        top_weights: vec![[0; 8]; n],
        span_start: vec![0; n],
        span_end: vec![0; n],
        byte_start: vec![0; n],
        byte_end: vec![0; n],
        hidden: None,
    }
}

/// The compile loop reproduces the Gate C instrumentation law on the
/// construction split: stride-four anchors keyed by EMITTED token,
/// same-story construction-split predecessors at each lookahead
/// distance, and rows below the minimum total dropped.
#[test]
fn compile_matches_gate_c_table_law() {
    let corpus = two_story_corpus();
    // Construction split: every position except 2 — this starves the
    // (distance 1, anchor 14) row down to a single observation, which
    // the minimum-total gate must then drop.
    let train: Vec<Observation> = (0..corpus.n)
        .filter(|&i| i != 2)
        .map(|i| observation(i, &corpus))
        .collect();
    let rows = compile_forward_anchor_rows(&corpus, &train);
    let expected = vec![
        ForwardAnchorRow {
            distance: 1,
            anchor: 18,
            total: 2,
            entries: vec![(17, 2)],
        },
        ForwardAnchorRow {
            distance: 2,
            anchor: 14,
            total: 2,
            entries: vec![(12, 2)],
        },
        ForwardAnchorRow {
            distance: 2,
            anchor: 18,
            total: 2,
            entries: vec![(16, 2)],
        },
        ForwardAnchorRow {
            distance: 3,
            anchor: 14,
            total: 2,
            entries: vec![(11, 2)],
        },
        ForwardAnchorRow {
            distance: 3,
            anchor: 18,
            total: 2,
            entries: vec![(15, 2)],
        },
    ];
    assert_eq!(rows, expected);
}

fn emissions() -> EmissionTables {
    let mut root_prior = BTreeMap::new();
    root_prior.insert(1u32, ScoreQ::from_raw(-50_000));
    EmissionTables {
        root_prior,
        root_floor: ScoreQ::from_raw(ROOT_FLOOR_RAW),
        root_total: 10,
        region_lists: vec![Vec::new()],
        smoothing: Smoothing::AddOne,
        root_prior_quantization: QuantizationErrorStats::default(),
        emission_quantization: QuantizationErrorStats::default(),
        selection_stats: Default::default(),
    }
}

/// A minimal but fully valid scored artifact: one region, no edges, one
/// explicit bigram context row (the deterministic base distribution for
/// the fusion tests), an empty exact-context store, and the given
/// forward-anchor rows.
fn tiny_artifact(fwd_rows: &[ForwardAnchorRow]) -> Vec<u8> {
    let regions = vec![RegionParams {
        node: 1,
        depth: 1,
        radius: 0,
        sig: [0; SIG_BYTES],
        parent: None,
    }];
    let context_rows = vec![ContextRow {
        context_len: 1,
        key0: 20,
        key1: 0,
        entries: vec![
            (4, ScoreQ::from_raw(-100_000)),
            (5, ScoreQ::from_raw(-120_000)),
            (6, ScoreQ::from_raw(-80_000)),
        ],
    }];
    let store: runtime::Store = vec![BTreeMap::new(); STAGES + 1];
    let tls1 = runtime::store_bytes(&store);
    let emissions = emissions();
    let (bytes, info) = emit_scored_r4g1(
        b"teacher-container",
        (b"meta", b"recs"),
        VOCAB,
        &ScoredGraphSections {
            regions: &regions,
            structural: &[],
            transitions: &[],
            transition_quantization: QuantizationErrorStats::default(),
            emissions: &emissions,
            context_rows: &context_rows,
            exct_tls1: &tls1,
            exct_top_x: 3,
            fmm_section: None,
            fwd_rows,
        },
    )
    .expect("tiny scored artifact");
    assert_eq!(info.fwda_row_count as usize, fwd_rows.len());
    bytes
}

fn fusion_rows() -> Vec<ForwardAnchorRow> {
    vec![
        ForwardAnchorRow {
            distance: 1,
            anchor: 99,
            total: 14,
            entries: vec![(5, 10), (6, 1), (7, 3)],
        },
        ForwardAnchorRow {
            distance: 2,
            anchor: 42,
            total: 2,
            entries: vec![(3, 2)],
        },
    ]
}

/// Rows written into the FWDA section come back byte-exact through the
/// validated view: distance, anchor, full total, and raw count entries.
#[test]
fn fwda_rows_roundtrip_exactly() {
    let rows = fusion_rows();
    let bytes = tiny_artifact(&rows);
    let view = GraphView::parse(&bytes).expect("emitted artifact re-validates");
    let table = view
        .fwda_table()
        .expect("FWDA section parses")
        .expect("FWDA section is present");
    let recovered: Vec<ForwardAnchorRow> = table
        .rows()
        .map(|row| ForwardAnchorRow {
            distance: row.distance(),
            anchor: row.anchor(),
            total: row.total(),
            entries: row
                .entries()
                .map(|entry| (entry.token, entry.count))
                .collect(),
        })
        .collect();
    assert_eq!(recovered, rows);
    // Canonical binary-search lookup agrees.
    let found = table.find(1, 99).expect("row (1, 99) is found");
    assert_eq!(found.total(), 14);
    assert!(table.find(1, 98).is_none());
    assert!(table.find(3, 99).is_none());
}

/// The quantized integer fusion selects the same token as the measured
/// f64 reference law on a three-candidate example — and the selection
/// differs from the base scorer's, so the channel demonstrably re-ranks.
#[test]
fn infill_fusion_matches_f64_reference() {
    let bytes = tiny_artifact(&fusion_rows());
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    assert_eq!(scorer.forward_anchor_row_count(), 2);
    let sig = [0u8; SIG_BYTES];
    let recent = [10u32, 20];

    let base = scorer
        .score_candidates_coded(&sig, None, &recent)
        .expect("base outcome");
    assert_eq!(base.selected, 6, "base argmax is the context row's");

    let fused = scorer
        .score_candidates_infill(&sig, None, &recent, Some((99, 1)))
        .expect("fused outcome");

    // f64 reference: fused_ln(t) = base_q(t)/65536 + ln((c+0.5)/(total +
    // vocab*0.5)); base absentees enter at (root_floor +
    // transition_offset)/65536; argmax with lowest-token tie-break.
    let counts: BTreeMap<u32, u32> = [(5u32, 10u32), (6, 1), (7, 3)].into_iter().collect();
    let smooth = 14.0f64 + f64::from(VOCAB) * 0.5;
    let ln_fwd = |t: u32| ((f64::from(counts.get(&t).copied().unwrap_or(0)) + 0.5) / smooth).ln();
    let base_ln: BTreeMap<u32, f64> = base
        .candidates
        .iter()
        .map(|&(t, s)| (t, f64::from(s.raw()) / 65536.0))
        .collect();
    let floor_ln = f64::from(ROOT_FLOOR_RAW) / 65536.0; // transition offset is zero here
    let mut reference: BTreeMap<u32, f64> = BTreeMap::new();
    for (&t, &ln12) in &base_ln {
        reference.insert(t, ln12 + ln_fwd(t));
    }
    for &t in counts.keys() {
        reference.entry(t).or_insert(floor_ln + ln_fwd(t));
    }
    let (mut ref_token, mut ref_score) = (u32::MAX, f64::NEG_INFINITY);
    for (&t, &score) in &reference {
        if score > ref_score {
            ref_token = t;
            ref_score = score;
        }
    }

    assert_eq!(fused.selected, ref_token, "quantized argmax == f64 argmax");
    assert_eq!(fused.selected, 5, "the anchor evidence flips the pick");
    assert_ne!(fused.selected, base.selected);

    // The fused candidate set is the base set plus the row-only token,
    // and the row-only token entered at the offset-shifted root floor
    // plus its quantized residual.
    let fused_map: BTreeMap<u32, ScoreQ> = fused.candidates.iter().copied().collect();
    assert_eq!(
        fused_map.keys().copied().collect::<Vec<_>>(),
        vec![4, 5, 6, 7]
    );
    let residual_7 = ScoreQ::from_logprob(((3.0f64 + 0.5) / smooth).ln() as f32);
    assert_eq!(
        fused_map[&7],
        ScoreQ::from_raw(ROOT_FLOOR_RAW).saturating_add(residual_7)
    );

    // Post-witness re-ranking: the witness and the components keep the
    // base outcome's contents.
    assert_eq!(fused.witness, base.witness);
    assert_eq!(fused.candidate_components, base.candidate_components);
}

/// Channel-off paths are byte-identical to `score_candidates_coded`: a
/// `None` anchor, an anchor without a matching row, and an artifact
/// without any FWDA section.
#[test]
fn infill_off_paths_match_coded_exactly() {
    let with_rows = tiny_artifact(&fusion_rows());
    let without_rows = tiny_artifact(&[]);
    let sig = [0u8; SIG_BYTES];
    let recent = [10u32, 20];

    let view = GraphView::parse(&without_rows).expect("artifact without FWDA re-validates");
    assert!(view.fwda_table().expect("parses").is_none());

    for bytes in [&with_rows, &without_rows] {
        let scorer = GraphScorer::from_artifact(bytes, None, 3, 3).expect("scorer loads");
        let base = scorer
            .score_candidates_coded(&sig, None, &recent)
            .expect("base outcome");
        let none_anchor = scorer
            .score_candidates_infill(&sig, None, &recent, None)
            .expect("infill without anchor");
        let missing_row = scorer
            .score_candidates_infill(&sig, None, &recent, Some((77, 2)))
            .expect("infill with unmatched anchor");
        for outcome in [&none_anchor, &missing_row] {
            assert_eq!(outcome.selected, base.selected);
            assert_eq!(outcome.selected_score, base.selected_score);
            assert_eq!(outcome.candidates, base.candidates);
            assert_eq!(outcome.candidate_components, base.candidate_components);
            assert_eq!(outcome.witness, base.witness);
            assert_eq!(outcome.exact_context_source, base.exact_context_source);
        }
    }

    let scorer = GraphScorer::from_artifact(&without_rows, None, 3, 3).expect("scorer loads");
    assert_eq!(scorer.forward_anchor_row_count(), 0);
    let base = scorer
        .score_candidates_coded(&sig, None, &recent)
        .expect("base outcome");
    let fused = scorer
        .score_candidates_infill(&sig, None, &recent, Some((99, 1)))
        .expect("infill on a channel-less artifact");
    assert_eq!(fused.selected, base.selected);
    assert_eq!(fused.candidates, base.candidates);
    assert_eq!(fused.witness, base.witness);
}
