//! FWDA forward-anchor section round-trip and infill-fusion tests
//! (issue #399): the compile loop matches the Gate C instrumentation
//! law, rows survive the artifact byte format exactly, and
//! `score_candidates_infill` reproduces the measured f64 product-fusion
//! reference on a hand-computed example while staying byte-identical to
//! `score_candidates_coded` whenever the channel is off. The two-pass
//! tests cover `two_pass_infill_generate`: the stride-four anchor grid,
//! the B′ confidence gate (fails closed), and the pass-2 pure re-rank.
//! The A-mode tests cover `infill_fill`: givens consumed verbatim, free
//! positions filled with the nearest in-range given as the forward
//! anchor, base-scoring fallback when the channel is off or the next
//! given is beyond the FWDA lookahead range.

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler::{self, Corpus, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    compile_forward_anchor_rows, emit_scored_r4g1, emit_scored_r4g1_with_bound_partition_cids,
    emit_scored_r4g1_with_tokenizer_cid, ContextRow, EmissionTables, ForwardAnchorRow,
    QuantizationErrorStats, ScoredGraphSections, Smoothing,
};
use uor_r4_graph_certify::score_runtime::{
    infill_fill, next_skeleton_anchor, two_pass_infill_generate, GraphScorer, RegionParams,
};
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
        trajectory_sig: [0; SIG_BYTES],
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

/// A minimal but fully valid scored artifact: one region, no edges, two
/// explicit unigram context rows (the deterministic base distributions
/// for the fusion and two-pass tests; the key-6 row makes greedy
/// generation alternate 6 ↔ 20 with every step an NGRAM ExactContext
/// hit), an empty exact-context store, and the given forward-anchor
/// rows.
fn tiny_artifact(fwd_rows: &[ForwardAnchorRow]) -> Vec<u8> {
    tiny_artifact_emitted(fwd_rows, None, None)
}

fn tiny_artifact_emitted(
    fwd_rows: &[ForwardAnchorRow],
    tokenizer_cid: Option<[u8; 32]>,
    partition_cids: Option<([u8; 32], [u8; 32])>,
) -> Vec<u8> {
    let regions = vec![RegionParams {
        node: 1,
        depth: 1,
        radius: 0,
        sig: [0; SIG_BYTES],
        trajectory_sig: None,
        trajectory_radius: None,
        parent: None,
    }];
    let context_rows = vec![
        ContextRow {
            context_len: 1,
            key0: 6,
            key1: 0,
            entries: vec![
                (7, ScoreQ::from_raw(-100_000)),
                (20, ScoreQ::from_raw(-80_000)),
            ],
        },
        ContextRow {
            context_len: 1,
            key0: 20,
            key1: 0,
            entries: vec![
                (4, ScoreQ::from_raw(-100_000)),
                (5, ScoreQ::from_raw(-120_000)),
                (6, ScoreQ::from_raw(-80_000)),
            ],
        },
    ];
    let store: runtime::Store = vec![BTreeMap::new(); STAGES + 1];
    let tls1 = runtime::store_bytes(&store);
    let emissions = emissions();
    let sections = ScoredGraphSections {
        regions: &regions,
        structural: &[],
        transitions: &[],
        transition_quantization: QuantizationErrorStats::default(),
        emissions: &emissions,
        context_rows: &context_rows,
        exct_tls1: &tls1,
        exct_top_x: 3,
        fwd_rows,
        skipmix_rows: &[],
        psi_bag_rows: &[],
    };
    let (bytes, info) = match partition_cids {
        Some((construction, certification)) => emit_scored_r4g1_with_bound_partition_cids(
            b"teacher-container",
            VOCAB,
            &sections,
            tokenizer_cid.unwrap_or([0; 32]),
            construction,
            certification,
        ),
        None => match tokenizer_cid {
            Some(tokenizer_cid) => emit_scored_r4g1_with_tokenizer_cid(
                b"teacher-container",
                (b"meta", b"recs"),
                VOCAB,
                &sections,
                tokenizer_cid,
            ),
            None => emit_scored_r4g1(b"teacher-container", (b"meta", b"recs"), VOCAB, &sections),
        },
    };
    assert_eq!(info.fwda_row_count as usize, fwd_rows.len());
    bytes
}

#[test]
fn scored_emitter_binds_tokenizer_and_preserves_legacy_wrapper_bytes() {
    let legacy = tiny_artifact_emitted(&[], None, None);
    let explicit_legacy = tiny_artifact_emitted(&[], Some([0; 32]), None);
    assert_eq!(legacy, explicit_legacy, "legacy wrapper bytes changed");

    let tokenizer = b"exact scored tokenizer.bin bytes";
    let tokenizer_cid = *blake3::hash(tokenizer).as_bytes();
    let bound = tiny_artifact_emitted(&[], Some(tokenizer_cid), None);
    assert_ne!(bound, legacy);
    let view = GraphView::parse(&bound).expect("bound scored graph parses");
    assert_eq!(view.head().expect("HEAD").tokenizer_cid().0, tokenizer_cid);
    view.verify_tokenizer_cid(tokenizer)
        .expect("exact tokenizer verifies");
    assert!(view.verify_tokenizer_cid(b"swapped").is_err());
}

#[test]
fn production_emitter_binds_both_exact_partition_cids() {
    let construction = [0x33; 32];
    let certification = [0x44; 32];
    let bytes = tiny_artifact_emitted(&[], Some([0x22; 32]), Some((construction, certification)));
    let view = GraphView::parse(&bytes).expect("production graph parses");
    let head = view.head().expect("HEAD");
    assert_eq!(head.corpus_construction_cid().0, construction);
    assert_eq!(head.corpus_certification_cid().0, certification);
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

/// A minimal `Compiled` for the two-pass generation tests: empty token
/// codes decode to all-zero rows, so every window bundles to zero, the
/// sig is all-zero (matching the tiny artifact's region signature), and
/// the graded code is all-zero. Token selection in these tests is
/// carried entirely by the artifact's NGRAM context rows through the
/// recent-token deque, which is exactly the surface the two-pass gate
/// depends on.
fn tiny_compiled() -> compiler::Compiled {
    compiler::Compiled {
        token_codes: Vec::new(),
        stage_books: Vec::new(),
        stage_shifts: Vec::new(),
        thresholds: vec![0i64; compiler::D],
        class_sigs: Vec::new(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb: Vec::new(),
        resid_cb: Vec::new(),
        resid_scale_shifts: Vec::new(),
        norm_fold_const: 0,
    }
}

/// Gated two-pass generation on the synthetic artifact: seed `[20]`
/// alternates the draft 6 ↔ 20 through the two NGRAM rows (every step
/// ExactContext), so the stride-four anchor grid and the B′ gate are
/// both fully live. With a `(distance 2, anchor 6)` forward row present,
/// exactly the two gated free positions at lookahead two get re-ranked
/// to token 5; every other position — anchors included — keeps its
/// pass-1 token.
#[test]
fn two_pass_reranks_gated_positions_only() {
    let row = ForwardAnchorRow {
        distance: 2,
        anchor: 6,
        total: 50,
        entries: vec![(5, 50)],
    };
    let bytes = tiny_artifact(std::slice::from_ref(&row));
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();
    let seed = [20u32];

    let two_pass =
        two_pass_infill_generate(&scorer, &artifacts, &rotations, &seed, 8).expect("two-pass runs");

    // Pass 1: the NGRAM rows alternate deterministically.
    assert_eq!(two_pass.draft, vec![6, 20, 6, 20, 6, 20, 6, 20]);
    // Anchor grid with seed_len = 1: generated index i is an anchor when
    // (1 + i + 1) is a multiple of four — indices 2 and 6.
    assert_eq!(two_pass.anchor_positions, 2);
    // Gated free positions: indices 0, 1, 3, 4, 5 have an in-stream next
    // anchor whose draft resolved ExactContext; index 7's next anchor
    // (index 10) falls beyond the stream.
    assert_eq!(two_pass.gated_positions, 5);
    // Only the lookahead-two positions (0 and 4) have a matching forward
    // row; the anchor evidence flips both to token 5.
    assert_eq!(two_pass.changed_positions, 2);
    assert_eq!(two_pass.final_tokens, vec![5, 20, 6, 20, 5, 20, 6, 20]);
    assert!(two_pass.changed_positions <= two_pass.gated_positions);
    // Anchors are kept verbatim from the draft.
    for index in [2usize, 6] {
        assert_eq!(two_pass.final_tokens[index], two_pass.draft[index]);
    }
}

/// Without a FWDA section pass 2 is inert: the gate still opens (every
/// draft anchor resolves ExactContext) but no forward row exists, so the
/// re-rank is the identity and the final stream equals the draft.
#[test]
fn two_pass_empty_fwda_final_equals_draft() {
    let bytes = tiny_artifact(&[]);
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();
    let seed = [20u32];

    let two_pass =
        two_pass_infill_generate(&scorer, &artifacts, &rotations, &seed, 8).expect("two-pass runs");
    assert_eq!(two_pass.draft, vec![6, 20, 6, 20, 6, 20, 6, 20]);
    assert_eq!(two_pass.final_tokens, two_pass.draft);
    assert_eq!(two_pass.changed_positions, 0);
    assert_eq!(two_pass.gated_positions, 5);
    assert_eq!(two_pass.anchor_positions, 2);
}

/// The confidence gate fails closed: seeded off the NGRAM rows the draft
/// resolves every position through the graph path (status Graph, token
/// one from the root prior), so no anchor is ExactContext and pass 2
/// changes nothing — even though a forward row keyed to the drafted
/// anchor carries evidence strong enough to flip the argmax if the gate
/// were ignored.
#[test]
fn two_pass_gate_fails_closed_without_exact_context_anchor() {
    // ln(2 * 1000 + 1) ≈ 7.6 nats of residual spread over the absent
    // floor — enough to overcome the root-floor entry gap (≈ 6.87
    // nats), so an open gate WOULD flip gated positions to token 5.
    let row = ForwardAnchorRow {
        distance: 2,
        anchor: 1,
        total: 1000,
        entries: vec![(5, 1000)],
    };
    let bytes = tiny_artifact(std::slice::from_ref(&row));
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();
    // Token 30 hits no NGRAM row, so every draft step selects the root
    // prior's token one with status Graph — never ExactContext.
    let seed = [30u32];

    let two_pass =
        two_pass_infill_generate(&scorer, &artifacts, &rotations, &seed, 8).expect("two-pass runs");
    assert_eq!(two_pass.draft, vec![1; 8]);
    assert_eq!(two_pass.anchor_positions, 2);
    assert_eq!(two_pass.gated_positions, 0);
    assert_eq!(two_pass.changed_positions, 0);
    assert_eq!(two_pass.final_tokens, two_pass.draft);
}

/// A-mode skeleton fill: given tokens are consumed verbatim, free
/// positions get filled, and a free position whose nearest in-range
/// given carries a live forward row is re-ranked by the anchor evidence
/// (the same `(distance 2, anchor 6)` flip the two-pass test measures),
/// while a free position whose anchor has no row at its distance stays
/// on the base selection. A fully given skeleton — tokens with no
/// scoring rows at all — comes back byte-identical.
#[test]
fn infill_fill_respects_givens_and_reranks_free() {
    let row = ForwardAnchorRow {
        distance: 2,
        anchor: 6,
        total: 50,
        entries: vec![(5, 50)],
    };
    let bytes = tiny_artifact(std::slice::from_ref(&row));
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();

    // Position one is free with the given anchor 6 at distance two (the
    // live row: base pick 6 flips to 5); position two is free with the
    // anchor at distance one (no row there: base pick, the root prior's
    // token one, survives).
    let skeleton = [Some(20), None, None, Some(6)];
    let filled = infill_fill(&scorer, &artifacts, &rotations, &skeleton).expect("fill runs");
    assert_eq!(filled, vec![20, 5, 1, 6]);
    assert_eq!(filled[0], 20, "given consumed verbatim");
    assert_eq!(filled[3], 6, "given consumed verbatim");

    // The anchor-selection rule the fill used, checked directly.
    assert_eq!(next_skeleton_anchor(&skeleton, 1), Some((6, 2)));
    assert_eq!(next_skeleton_anchor(&skeleton, 2), Some((6, 1)));
    assert_eq!(next_skeleton_anchor(&skeleton, 3), None, "nothing follows");

    // A fully given skeleton is returned verbatim even for tokens the
    // artifact carries no rows for.
    let all_given = [Some(9), Some(8), Some(7)];
    let verbatim = infill_fill(&scorer, &artifacts, &rotations, &all_given).expect("fill runs");
    assert_eq!(verbatim, vec![9, 8, 7]);
}

/// Without a FWDA section the fill IS base scoring: every free position
/// matches a manual greedy loop over `score_candidates_coded` that
/// consumes the same givens with the same window and recent-token
/// handling — even though the skeleton offers in-range anchors at three
/// free positions.
#[test]
fn infill_fill_absent_fwda_matches_base_scoring() {
    let bytes = tiny_artifact(&[]);
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    assert_eq!(scorer.forward_anchor_row_count(), 0);
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();

    let skeleton = [Some(20), None, None, None, None, Some(6)];
    let filled = infill_fill(&scorer, &artifacts, &rotations, &skeleton).expect("fill runs");

    // Manual base-scoring reference with identical context handling.
    let mut window = [0u32; compiler::WINDOW];
    let mut recent: Vec<u32> = Vec::new();
    let mut reference = Vec::new();
    for (w_len, slot) in skeleton.iter().enumerate() {
        let token = match slot {
            Some(given) => *given,
            None => {
                let bundle = runtime::bundle_window_plain(&artifacts, &rotations, &window[..w_len]);
                let sig = runtime::sig_plain(&artifacts, &bundle);
                let code = runtime::assign_for_bundle(&artifacts, &bundle);
                scorer
                    .score_candidates_coded(&sig, Some(&code), &recent)
                    .expect("base outcome")
                    .selected
            }
        };
        reference.push(token);
        window[w_len] = token;
        recent.push(token);
    }
    assert_eq!(filled, reference);
    // The base dynamics are the two-pass draft's alternation, so the
    // comparison is not vacuous.
    assert_eq!(filled, vec![20, 6, 20, 6, 20, 6]);
}

/// A next given farther than the FWDA lookahead range yields no anchor
/// and the position falls back to base scoring — position one keeps its
/// base pick although the `(distance 2, anchor 6)` row is loaded and a
/// given 6 sits at distance four, while position three (the same row at
/// its measured distance) demonstrably flips in the same run.
#[test]
fn infill_fill_far_anchor_falls_back_to_base() {
    let row = ForwardAnchorRow {
        distance: 2,
        anchor: 6,
        total: 50,
        entries: vec![(5, 50)],
    };
    let bytes = tiny_artifact(std::slice::from_ref(&row));
    let scorer = GraphScorer::from_artifact(&bytes, None, 3, 3).expect("scorer loads");
    let artifacts = tiny_compiled();
    let rotations = runtime::derive_rotations();

    let skeleton = [Some(20), None, None, None, None, Some(6)];
    // Position one sees only free slots within the lookahead range; the
    // nearest given is four positions ahead.
    assert_eq!(next_skeleton_anchor(&skeleton, 1), None);
    assert_eq!(next_skeleton_anchor(&skeleton, 2), Some((6, 3)));
    assert_eq!(next_skeleton_anchor(&skeleton, 3), Some((6, 2)));
    assert_eq!(next_skeleton_anchor(&skeleton, 4), Some((6, 1)));

    let filled = infill_fill(&scorer, &artifacts, &rotations, &skeleton).expect("fill runs");
    // Position one: no anchor in range, base pick 6 (identical to the
    // absent-FWDA fill at that position). Position two: anchor at
    // distance three has no row, base pick 20. Position three: anchor at
    // distance two hits the live row and flips to 5. Position four:
    // anchor at distance one has no row, base pick (root prior token).
    assert_eq!(filled, vec![20, 6, 20, 5, 1, 6]);
}
