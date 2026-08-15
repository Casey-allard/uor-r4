//! #661/#722 Octeract route-trace screen conformance.
//!
//! These fixtures are deliberately small, deterministic, and certifier-only.
//! They exercise the locked screen and its null instruments without presenting
//! synthetic observations as evidence about a real checkpoint.

use std::collections::BTreeMap;

use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;
use uor_r4_graph_certify::octeract_trace_screen::{
    canonical_octeract_trace_report_bytes, classify_candidate_arm, collision_oracle,
    deranged_supports, exhaustive_anchor_relabel_control, occupancy_matched_fold_null,
    octeract_trace_payload_kappa, octeract_trace_report_kappa,
    preregistered_octeract_trace_contract, registered_trace_evidence, run_octeract_trace_screen,
    score_octeract_step, shuffled_block_permutation, CandidateGate, CollisionGroup,
    OcteractTraceInput, ScoredCandidate, ScreenDisposition, TraceKind, ANCHOR_RELABELINGS,
    INSTRUMENT_CONFORMANCE_EVIDENCE_ID, INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
    OCCUPANCY_MATCHED_FOLD_SEED, OCTERACT_TRACE_CONTRACT_FORMAT, OCTERACT_TRACE_REPORT_FORMAT,
    SHUFFLED_BLOCK_SEED,
};
use uor_r4_graph_certify::route_fit_report::StageVerdict;
use uor_r4_graph_compiler::observation::ObservationManifest;
use uor_r4_graph_compiler::route_fit::{
    synthetic_fit_manifest, FittedRouteCodes, HeadCodes, RouteFitMethod, RouteTraceCorpus,
    StepTrace, StoryTrace,
};
use uor_r4_graph_compiler::trace_profile::TraceProfile;
use uor_r4_graph_format::route_attention::ROUTE_CODE_BYTES;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::geometry::GeometryProjection;
use uor_r4_model_source::TraceCaptureGeometry;

const TEST_STEPS: usize = 12;
const TEST_TOP_M: u32 = 2;

struct Fixture {
    corpus: RouteTraceCorpus,
    fitted: FittedRouteCodes,
    manifest: uor_r4_graph_compiler::route_fit::FitManifest,
}

fn digest(label: &str) -> String {
    format!("blake3:{}", blake3::hash(label.as_bytes()).to_hex())
}

fn code_with(first: u8, second: u8) -> [u8; ROUTE_CODE_BYTES] {
    let mut code = [0u8; ROUTE_CODE_BYTES];
    code[0] = first;
    code[1] = second;
    code
}

/// One 12-step, one-head `full/1` fixture. Key XOR deltas include exact ties
/// and complement-fold pairs (1/7 and 2/6). The nonuniform shared query base
/// preserves those distances while making the shuffled-block null observable.
/// Teacher supports follow the exact V1 winners on eligible steps, which gives
/// the frame and derangement controls a non-vacuous signal.
fn fixture() -> Fixture {
    let profile = TraceProfile::full(&[0], TEST_TOP_M);
    let geometry = TraceCaptureGeometry {
        layers: 1,
        heads: 1,
        kv_heads: 1,
        residual_width: 288,
    };
    let query_code: [u8; ROUTE_CODE_BYTES] =
        core::array::from_fn(|block| (block as u8).wrapping_mul(37).wrapping_add(11));
    let distance_codes = [
        code_with(0x00, 0x00), // D=0, F=0
        code_with(0x01, 0x00), // D=1, F=1
        code_with(0x02, 0x00), // D=1, F=1 (V1 tie, index loses)
        code_with(0xfe, 0x00), // D=7, F=1 (complement-fold collision)
        code_with(0x03, 0x00), // D=2, F=2
        code_with(0xfc, 0x00), // D=6, F=2 (complement-fold collision)
        code_with(0x0f, 0x00), // D=4, F=4 (equator)
        code_with(0x00, 0x01), // D=1 in a second block
        code_with(0x00, 0x02), // D=1 in a second block (tie)
        code_with(0xff, 0x00), // D=8, F=0
        code_with(0x00, 0xff), // D=8 in a second block, F=0
        code_with(0x55, 0xaa), // D=8, F=8
    ];
    let keys = distance_codes
        .map(|distance| core::array::from_fn(|block| query_code[block] ^ distance[block]));
    let queries = vec![vec![query_code; TEST_STEPS]];
    let key_codes = vec![keys.to_vec()];

    let mut steps = Vec::with_capacity(TEST_STEPS);
    for pos in 0..TEST_STEPS {
        let support = match pos {
            0 => vec![(0, 1.0)],
            1 => vec![(0, 0.75), (1, 0.25)],
            _ => vec![(0, 0.6), (1, 0.4)],
        };
        steps.push(StepTrace {
            pos: pos as u32,
            input_token: pos as u32,
            next: (pos + 1) as u32,
            top_tokens: [0, 1, 2, 3, 4, 5, 6, 7],
            target_logprob_nats: -1.0,
            q_rows: vec![vec![0.0; 288]],
            k_rows: vec![vec![0.0; 288]],
            supports: vec![vec![support]],
        });
    }
    let corpus = RouteTraceCorpus {
        geometry,
        declared_layers: vec![0],
        support_size: TEST_TOP_M,
        trace_profile: profile,
        stories: vec![StoryTrace {
            story: 0,
            tokens: (0..TEST_STEPS as u32).collect(),
            steps,
        }],
        records: TEST_STEPS,
        records_kappa: digest("#722-test-records"),
        trace_kappa: digest("#722-test-trace"),
        identity_bundle_digest: digest("#722-test-observation-identities"),
    };
    let fitted = FittedRouteCodes {
        method: RouteFitMethod::route_fit_v1(),
        top_m: TEST_TOP_M,
        heads: vec![HeadCodes {
            layer: 0,
            head: 0,
            thresholds: vec![0.0; 288],
            query_codes: queries,
            key_codes,
        }],
    };
    let mut manifest = synthetic_fit_manifest(&corpus, &digest("#722-test-source"))
        .expect("registered route-fit/operator identities");
    // The fixture can be offered as a fully identified pinned-real shape in
    // negative/control tests. TraceKind still decides whether rows are
    // empirical or instrument-conformance; identity spelling alone cannot
    // promote a synthetic row.
    manifest.tokenizer = Some(digest("#722-test-tokenizer"));
    Fixture {
        corpus,
        fitted,
        manifest,
    }
}

/// A second shape in which the full bounded screen has four eligible steps
/// (`N=9..=12`, `M=8`) but the locked `P=floor(3N/4)` prefilter can run only
/// for `N=11,12`. This makes row-domain accounting observably different from
/// copying the shared counts into the prefilter row.
fn prefilter_subset_fixture() -> Fixture {
    let mut fixture = fixture();
    fixture.corpus.support_size = 8;
    fixture.corpus.trace_profile = TraceProfile::full(&[0], 8);
    fixture.fitted.top_m = 8;
    for (position, step) in fixture.corpus.stories[0].steps.iter_mut().enumerate() {
        let candidates: Vec<u32> = if position < 8 {
            (0..=position as u32).collect()
        } else {
            vec![0, 1, 2, 4, 5, 6, 7, 8]
        };
        let support = candidates
            .iter()
            .map(|&candidate| (candidate, 1.0 / candidates.len() as f32))
            .collect();
        step.supports = vec![vec![support]];
    }
    let source_snapshot = fixture
        .manifest
        .source_snapshot
        .clone()
        .expect("synthetic teacher snapshot identity");
    fixture.manifest = synthetic_fit_manifest(&fixture.corpus, &source_snapshot)
        .expect("registered route-fit/operator identities");
    fixture
}

fn screen_input<'a>(
    fixture: &'a Fixture,
    observation_manifest: Option<&'a ObservationManifest>,
) -> OcteractTraceInput<'a> {
    OcteractTraceInput {
        corpus: &fixture.corpus,
        fitted: &fixture.fitted,
        fit_manifest: &fixture.manifest,
        observation_manifest,
        evidence: registered_trace_evidence(
            INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
            INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
        )
        .expect("registered structural evidence"),
    }
}

fn bound_observation(fixture: &mut Fixture) -> ObservationManifest {
    let mut tokenizer = TokenizerAdapter {
        family: TokenizerAdapter::HF_BYTE_BPE_FAMILY.to_owned(),
        version: TokenizerAdapter::HF_BYTE_BPE_VERSION,
        tokenizer_cid: digest("#722-test-tokenizer-definition"),
        ..TokenizerAdapter::default()
    };
    tokenizer.adapter_digest = tokenizer.declared_digest();

    let mut observation = ObservationManifest::new(0);
    observation.input_cid = Some(digest("#722-test-input-corpus"));
    observation.source_manifest_kappa = Some(digest("#722-test-source-manifest"));
    observation.geometry = Some(GeometryProjection::bucket_average(288, 288));
    observation.tokenizer_adapter = Some(tokenizer.clone());
    observation.attention_operator = Some(AttentionOperatorSpec::standard());
    observation.trace_profile = Some(fixture.corpus.trace_profile.clone());
    observation.total_records = fixture.corpus.records as u64;

    fixture.corpus.identity_bundle_digest = observation.identity_bundle_digest();
    fixture.manifest.tokenizer = Some(tokenizer.declared_digest());
    observation
}

fn real_shaped_fixture() -> (Fixture, ObservationManifest) {
    let mut fixture = fixture();
    fixture.manifest.adapter = Some("pinned-real-test-adapter/1".to_owned());
    let observation = bound_observation(&mut fixture);
    (fixture, observation)
}

fn assert_unavailable(report: &uor_r4_graph_certify::octeract_trace_screen::OcteractTraceReport) {
    assert_eq!(report.disposition, ScreenDisposition::Unavailable);
    assert_eq!(report.arms[0].verdict, StageVerdict::Unavailable);
    assert!(report.arms[1..]
        .iter()
        .all(|arm| arm.verdict == StageVerdict::NotRun));
    assert!(report
        .arms
        .iter()
        .all(|arm| !matches!(arm.verdict, StageVerdict::Pass | StageVerdict::Fail)));
}

#[test]
fn verdict_classifier_keeps_all_four_states_distinct_and_synthetic_nonempirical() {
    let gate = CandidateGate {
        controls_pass: true,
        frame_score: 1.0,
        deranged_distinct: true,
        gate_pass: true,
    };
    assert_eq!(
        classify_candidate_arm(TraceKind::PinnedReal, gate, false),
        StageVerdict::Pass
    );
    assert_eq!(
        classify_candidate_arm(
            TraceKind::PinnedReal,
            CandidateGate {
                gate_pass: false,
                ..gate
            },
            false,
        ),
        StageVerdict::Fail
    );
    assert_eq!(
        classify_candidate_arm(
            TraceKind::PinnedReal,
            CandidateGate {
                frame_score: 0.0,
                ..gate
            },
            false,
        ),
        StageVerdict::Unavailable,
        "a frame mismatch is absence, never an empirical negative"
    );
    assert_eq!(
        classify_candidate_arm(TraceKind::PinnedReal, gate, true),
        StageVerdict::NotRun
    );
    for gate_pass in [false, true] {
        assert_eq!(
            classify_candidate_arm(
                TraceKind::InstrumentConformance,
                CandidateGate { gate_pass, ..gate },
                false,
            ),
            StageVerdict::NotRun,
            "synthetic evidence can exercise the instrument but cannot become PASS/FAIL"
        );
    }
}

#[test]
fn contract_is_frozen_before_labels_and_only_two_arms_can_advance() {
    let first = preregistered_octeract_trace_contract();
    let second = preregistered_octeract_trace_contract();
    assert_eq!(first, second);
    assert_eq!(first.format, OCTERACT_TRACE_CONTRACT_FORMAT);
    assert_eq!((first.layer, first.head), (0, 0));
    assert_eq!((first.code_bits, first.blocks), (288, 36));
    assert_eq!(first.mask, vec![0xff; ROUTE_CODE_BYTES]);
    assert_eq!(first.occupancy_seed, OCCUPANCY_MATCHED_FOLD_SEED);
    assert_eq!(first.shuffled_block_seed, SHUFFLED_BLOCK_SEED);
    assert_eq!(first.thresholds.direct_jaccard_margin, 0.03);
    assert_eq!(first.thresholds.prefilter_v1_recall, 0.95);
    assert_eq!(first.thresholds.prefilter_refinement_fraction, 0.75);
    assert!(first
        .aggregation
        .contains("row-counts-own-consumed-domain; shared-counts-are-matched-base-domain"));
    let evidence = registered_trace_evidence(
        INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
        INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
    )
    .expect("registered structural evidence");
    assert_eq!(first.evidence_registry.len(), 1);
    assert_eq!(first.evidence_registry[0].id, evidence.id());
    assert_eq!(first.evidence_registry[0].version, evidence.version());
    assert_eq!(first.evidence_registry[0].kind, evidence.kind());
    assert_eq!(
        first.evidence_registry[0].declared_digest,
        evidence.declared_digest()
    );
    assert!(first
        .evidence_registry_rule
        .contains("registry-expansion-requires-new-contract-and-report-format-version"));
    let advancing: Vec<&str> = first
        .arms
        .iter()
        .filter(|arm| arm.can_advance)
        .map(|arm| arm.id.as_str())
        .collect();
    assert_eq!(advancing, vec!["octeract-fold5", "octeract-prefilter"]);
    assert!(first.nulls.iter().all(|null| !null.can_advance));

    let mut first_bytes = Vec::new();
    ciborium::into_writer(&first, &mut first_bytes).expect("contract serializes");
    let mut second_bytes = Vec::new();
    ciborium::into_writer(&second, &mut second_bytes).expect("contract serializes");
    assert_eq!(first_bytes, second_bytes);
}

#[test]
fn absent_real_trace_is_canonical_typed_unavailable_and_never_searched() {
    let first = run_octeract_trace_screen(None, TraceKind::PinnedReal);
    let second = run_octeract_trace_screen(None, TraceKind::PinnedReal);
    assert_eq!(first, second);
    assert_eq!(first.format, OCTERACT_TRACE_REPORT_FORMAT);
    assert_eq!(first.disposition, ScreenDisposition::Unavailable);
    assert!(first.disposition_reason.contains("explicitly supplied"));
    assert_eq!(first.arms[0].verdict, StageVerdict::Unavailable);
    assert!(first.arms[1..]
        .iter()
        .all(|arm| arm.verdict == StageVerdict::NotRun));
    assert!(first
        .nulls
        .iter()
        .all(|null| null.verdict == StageVerdict::NotRun));
    assert_eq!(first.controls.frame, "UNAVAILABLE");
    assert_eq!(first.identities.observation_identity_bundle_digest, None);
    assert_eq!(first.identities.source_attention_operator_digest, None);
    assert_eq!(first.payload_kappa, octeract_trace_payload_kappa(&first));
    assert_eq!(
        canonical_octeract_trace_report_bytes(&first),
        canonical_octeract_trace_report_bytes(&second)
    );
    assert_eq!(
        octeract_trace_report_kappa(&first),
        octeract_trace_report_kappa(&second)
    );
    assert_eq!(canonical_octeract_trace_report_bytes(&first).len(), 6_587);
    assert_eq!(
        first.payload_kappa,
        "blake3:8b8c3bdc41f04ac2d6b9a15ef843f5064fae92e8b6bfe57cafbc6803eca7c5a2"
    );
    assert_eq!(
        octeract_trace_report_kappa(&first),
        "blake3:eab7b1bb12d9508d9815da0c4fbac248eab8b93b8258e717829542e41ac75e5e"
    );

    // The final byte function recomputes the non-self-referential payload
    // identity, so mutating only the cached derived field cannot fork bytes.
    let mut stale = first.clone();
    stale.payload_kappa = "blake3:stale-caller-cache".to_owned();
    assert_eq!(
        canonical_octeract_trace_report_bytes(&stale),
        canonical_octeract_trace_report_bytes(&first)
    );
}

#[test]
fn instrument_report_is_deterministic_comprehensive_and_nonpromoting() {
    let mut fixture = fixture();
    let observation = bound_observation(&mut fixture);
    let first = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::InstrumentConformance,
    );
    let second = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::InstrumentConformance,
    );
    assert_eq!(first, second);
    assert_eq!(first.disposition, ScreenDisposition::Unavailable);
    assert!(first.disposition_reason.contains("instrument-conformance"));
    assert_eq!(first.payload_kappa, octeract_trace_payload_kappa(&first));
    assert_eq!(
        canonical_octeract_trace_report_bytes(&first),
        canonical_octeract_trace_report_bytes(&second)
    );
    assert_eq!(
        octeract_trace_report_kappa(&first),
        octeract_trace_report_kappa(&second)
    );

    assert!(first.controls.instrument_valid);
    assert!(first.controls.v1_scalar_operator_exact);
    assert!(first.controls.weight9_exact);
    assert!(first.controls.oriented_exact);
    assert!(first.controls.lower_bound_exact);
    assert!(first.controls.anchor_relabel_invariant);
    assert_eq!(first.controls.anchor_relabels_checked, ANCHOR_RELABELINGS);
    assert!(first.controls.deranged_support_transformation_distinct);
    assert!(first.controls.deranged_support_observably_distinct);
    assert!(first.controls.occupancy_null_transformation_distinct);
    assert!(first.controls.occupancy_null_result_distinct);
    assert!(first.controls.shuffled_block_null_transformation_distinct);
    assert!(first.controls.shuffled_block_null_result_distinct);
    assert_eq!(first.controls.frame, "FRAMED");

    assert_eq!(first.arms.len(), 6);
    assert_eq!(first.nulls.len(), 3);
    assert!(first.arms.iter().all(|arm| {
        arm.verdict == StageVerdict::NotRun && arm.reason.contains("instrument-conformance")
    }));
    assert!(first.nulls.iter().all(|null| {
        !null.can_advance
            && null.verdict == StageVerdict::NotRun
            && null.reason.contains("instrument-conformance")
    }));

    let shared = &first.shared_metrics;
    assert_eq!(shared.counts.eligible_stories, 1);
    assert_eq!(shared.counts.eligible_steps, 10);
    assert_eq!(shared.counts.candidates, 75);
    assert_eq!(shared.counts.teacher_support_entries, 20);
    assert!(first
        .arms
        .iter()
        .chain(&first.nulls)
        .all(|row| row.metrics.counts.is_some()));
    for row in [
        &first.arms[0],
        &first.arms[1],
        &first.arms[2],
        &first.arms[3],
        &first.arms[5],
        &first.nulls[0],
        &first.nulls[1],
    ] {
        assert_eq!(
            row.metrics.counts.as_ref(),
            Some(&shared.counts),
            "{}",
            row.id
        );
    }
    let prefilter_counts = first.arms[4]
        .metrics
        .counts
        .as_ref()
        .expect("prefilter owns its work-eligible domain");
    assert_eq!(prefilter_counts.eligible_stories, 1);
    assert_eq!(prefilter_counts.eligible_steps, 10);
    assert_eq!(prefilter_counts.candidates, 75);
    assert_eq!(prefilter_counts.teacher_support_entries, 20);
    let deranged_counts = first.nulls[2]
        .metrics
        .counts
        .as_ref()
        .expect("deranged-support null owns its transformed-label domain");
    assert_eq!(deranged_counts.eligible_stories, 1);
    assert_eq!(deranged_counts.eligible_steps, 10);
    assert_eq!(deranged_counts.candidates, 75);
    assert_eq!(
        deranged_counts.teacher_support_entries, 19,
        "the cyclic derangement ends with the one-entry position-0 support"
    );
    assert_eq!(shared.occupancy.exact_weight.iter().sum::<u64>(), 75 * 36);
    assert_eq!(shared.occupancy.folded_shell.iter().sum::<u64>(), 75 * 36);
    assert_eq!(shared.occupancy.exact_weight_per_block.len(), 36);
    assert_eq!(shared.occupancy.folded_shell_per_block.len(), 36);
    assert!(shared.occupancy.exact_entropy_bits.is_finite());
    assert!(shared.occupancy.folded_entropy_bits.is_finite());
    assert!(shared.occupancy.exact_entropy_bits > 0.0);
    assert!(shared.occupancy.folded_entropy_bits > 0.0);

    let v1 = &first.arms[0].metrics;
    assert!(first
        .arms
        .iter()
        .all(|row| row.metrics.fold_occupancy.is_none()));
    let v1_selection = v1.selection.as_ref().expect("V1 owns selection metrics");
    assert!(v1_selection.mean_teacher_jaccard > 0.0);
    assert!(v1_selection.score_support_mi_bits.is_finite());
    assert!(v1_selection.pooled_block_class_support_mi_bits.is_finite());
    assert_eq!(v1_selection.per_block_class_support_mi_bits.len(), 36);
    assert!(v1_selection
        .per_block_class_support_mi_bits
        .iter()
        .all(|value| value.is_finite()));
    assert!(v1.occupancy.is_none());
    assert!(v1.collisions.is_none());
    assert!(v1.shortlist.is_none());
    assert!(v1.prefilter.is_none());
    assert!(v1.lower_bound.is_none());
    assert!(v1.oracle.is_none());

    let weight9 = &first.arms[1].metrics;
    assert!(weight9.selection.is_some());
    assert!(weight9.occupancy.is_none());
    assert!(weight9.collisions.is_none());
    assert!(weight9.shortlist.is_none());
    assert!(weight9.prefilter.is_none());
    assert!(weight9.lower_bound.is_none());
    assert!(weight9.oracle.is_none());

    let fold5 = &first.arms[2].metrics;
    assert!(fold5.selection.is_some());
    let collisions = fold5
        .collisions
        .as_ref()
        .expect("fold5 owns collision metrics");
    assert!(collisions.exact_collision_pairs > 0);
    assert!(collisions.complement_collision_pairs > 0);
    assert!(collisions.complete_fold_signature_groups > 0);
    assert!(collisions.lossy_signature_groups > 0);
    assert!(!collisions.group_size_distribution.is_empty());
    let oracle = fold5.oracle.as_ref().expect("fold5 owns oracle metrics");
    assert!(oracle.mean_max_recall.is_finite());
    assert!(oracle.mean_max_jaccard.is_finite());
    assert!(fold5.occupancy.is_none());
    assert!(fold5.shortlist.is_none());
    assert!(fold5.prefilter.is_none());
    assert!(fold5.lower_bound.is_none());

    let oriented = &first.arms[3].metrics;
    assert!(oriented.selection.is_some());
    assert!(oriented.occupancy.is_none());
    assert!(oriented.collisions.is_none());
    assert!(oriented.shortlist.is_none());
    assert!(oriented.prefilter.is_none());
    assert!(oriented.lower_bound.is_none());
    assert!(oriented.oracle.is_none());

    let prefilter = &first.arms[4].metrics;
    assert!(prefilter.selection.is_some());
    let shortlist = prefilter
        .shortlist
        .as_ref()
        .expect("prefilter owns shortlist metrics");
    assert!(shortlist.false_positives > 0);
    assert_eq!(
        shortlist.false_negatives, 0,
        "the main fixture retains V1 inside P; the separate adversary pins the failure case"
    );
    let prefilter_metrics = prefilter
        .prefilter
        .as_ref()
        .expect("prefilter owns work/fidelity metrics");
    assert!(prefilter_metrics.work_eligible_steps > 0);
    assert!(prefilter_metrics.exact_refinement_candidates_avoided > 0);
    assert!(prefilter_metrics.exact_refinement_fraction.is_finite());
    assert!(prefilter_metrics.mean_v1_recall.is_finite());
    assert!(prefilter_metrics.mean_v1_jaccard.is_finite());
    assert!(prefilter.occupancy.is_none());
    assert!(prefilter.collisions.is_none());
    assert!(prefilter.lower_bound.is_none());
    assert!(prefilter.oracle.is_none());

    let lower = &first.arms[5].metrics;
    assert!(lower.selection.is_some());
    let lower_metrics = lower
        .lower_bound
        .as_ref()
        .expect("lower-bound control owns work metrics");
    assert!(lower_metrics.exact_evaluations_avoided > 0);
    assert!(lower_metrics.tight_fraction.is_finite());
    assert!(lower.occupancy.is_none());
    assert!(lower.collisions.is_none());
    assert!(lower.shortlist.is_none());
    assert!(lower.prefilter.is_none());
    assert!(lower.oracle.is_none());

    let occupancy_null = &first.nulls[0].metrics;
    let occupancy_null_occupancy = occupancy_null
        .fold_occupancy
        .as_ref()
        .expect("occupancy null owns its realized occupancy");
    assert_eq!(
        occupancy_null_occupancy.folded_shell.iter().sum::<u64>(),
        75 * 36
    );
    assert_eq!(
        occupancy_null_occupancy.folded_shell_per_block.len(),
        ROUTE_CODE_BYTES
    );
    assert!(occupancy_null.occupancy.is_none());
    assert!(occupancy_null.selection.is_some());
    assert!(occupancy_null.collisions.is_none());
    assert!(occupancy_null.shortlist.is_none());
    assert!(occupancy_null.prefilter.is_none());
    assert!(occupancy_null.lower_bound.is_none());
    assert!(occupancy_null.oracle.is_none());

    let shuffled_null = &first.nulls[1].metrics;
    let shuffled_occupancy = shuffled_null
        .occupancy
        .as_ref()
        .expect("shuffled-block null owns its realized occupancy");
    assert_eq!(shuffled_occupancy.exact_weight.iter().sum::<u64>(), 75 * 36);
    assert_eq!(shuffled_occupancy.folded_shell.iter().sum::<u64>(), 75 * 36);
    assert!(shuffled_null.selection.is_some());
    assert!(shuffled_null.fold_occupancy.is_none());
    assert!(shuffled_null.collisions.is_none());
    assert!(shuffled_null.shortlist.is_none());
    assert!(shuffled_null.prefilter.is_none());
    assert!(shuffled_null.lower_bound.is_none());
    assert!(shuffled_null.oracle.is_none());

    let deranged_null = &first.nulls[2].metrics;
    assert!(deranged_null.selection.is_some());
    assert!(deranged_null.occupancy.is_none());
    assert!(deranged_null.fold_occupancy.is_none());
    assert!(deranged_null.collisions.is_none());
    assert!(deranged_null.shortlist.is_none());
    assert!(deranged_null.prefilter.is_none());
    assert!(deranged_null.lower_bound.is_none());
    assert!(deranged_null.oracle.is_none());

    assert_eq!(
        first
            .identities
            .observation_identity_bundle_digest
            .as_deref(),
        Some(fixture.corpus.identity_bundle_digest.as_str())
    );
    assert_eq!(
        first.identities.records_kappa.as_deref(),
        Some(fixture.corpus.records_kappa.as_str())
    );
    assert_eq!(
        first.identities.trace_kappa.as_deref(),
        Some(fixture.corpus.trace_kappa.as_str())
    );
    assert_eq!(
        first.identities.route_fit_digest.as_deref(),
        Some(fixture.fitted.method.declared_digest().as_str())
    );
    assert_eq!(
        first.identities.fitted_params_kappa.as_deref(),
        Some(fixture.fitted.kappa().as_str())
    );
    assert_eq!(
        first.identities.fit_manifest_kappa.as_deref(),
        Some(fixture.manifest.kappa().as_str())
    );
    assert_eq!(
        first.identities.source_snapshot.as_deref(),
        fixture.manifest.source_snapshot.as_deref()
    );
    assert_eq!(
        first.identities.source_manifest_kappa.as_deref(),
        observation.source_manifest_kappa.as_deref()
    );
    assert_ne!(
        first.identities.source_snapshot, first.identities.source_manifest_kappa,
        "teacher weight snapshot kappa and #597 source-manifest binding are distinct identities"
    );
    assert_eq!(
        first.identities.observation_input_cid.as_deref(),
        observation.input_cid.as_deref()
    );
    assert_eq!(
        first.identities.source_geometry_digest.as_deref(),
        observation
            .geometry
            .as_ref()
            .map(GeometryProjection::declared_digest)
            .as_deref()
    );
    assert_eq!(
        first.identities.tokenizer_cid.as_deref(),
        observation
            .tokenizer_adapter
            .as_ref()
            .map(|adapter| adapter.tokenizer_cid.as_str())
    );
    assert_eq!(
        first.identities.tokenizer_adapter_digest.as_deref(),
        observation
            .tokenizer_adapter
            .as_ref()
            .map(|adapter| adapter.adapter_digest.as_str())
    );
    assert_eq!(
        first.identities.target_operator_digest,
        fixture.manifest.operator_identity
    );
    assert_eq!(
        first.identities.source_attention_operator_digest.as_deref(),
        Some(
            observation
                .attention_operator
                .as_ref()
                .expect("source operator")
                .declared_digest()
                .as_str()
        )
    );
    assert_ne!(
        first.identities.target_operator_digest, first.identities.source_attention_operator_digest,
        "the target dormant operator must never be relabeled as the teacher source operator"
    );
    let evidence = registered_trace_evidence(
        INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
        INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
    )
    .expect("registered structural evidence");
    assert_eq!(
        first.identities.trace_evidence_id.as_deref(),
        Some(evidence.id())
    );
    assert_eq!(
        first.identities.trace_evidence_version,
        Some(evidence.version())
    );
    assert_eq!(first.identities.trace_evidence_kind, Some(evidence.kind()));
    assert_eq!(
        first.identities.trace_evidence_digest,
        Some(evidence.declared_digest())
    );
}

#[test]
fn prefilter_counts_only_its_work_eligible_domain() {
    let fixture = prefilter_subset_fixture();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, None)),
        TraceKind::InstrumentConformance,
    );
    assert!(report.controls.instrument_valid);
    assert_eq!(report.shared_metrics.counts.eligible_stories, 1);
    assert_eq!(report.shared_metrics.counts.eligible_steps, 4);
    assert_eq!(report.shared_metrics.counts.candidates, 42);
    assert_eq!(report.shared_metrics.counts.teacher_support_entries, 32);

    assert!(report
        .arms
        .iter()
        .chain(&report.nulls)
        .all(|row| row.metrics.counts.is_some()));
    for row in [
        &report.arms[0],
        &report.arms[1],
        &report.arms[2],
        &report.arms[3],
        &report.arms[5],
        &report.nulls[0],
        &report.nulls[1],
    ] {
        assert_eq!(
            row.metrics.counts.as_ref(),
            Some(&report.shared_metrics.counts),
            "{}",
            row.id
        );
    }

    let prefilter = &report.arms[4].metrics;
    let counts = prefilter
        .counts
        .as_ref()
        .expect("prefilter owns a narrowed row domain");
    assert_eq!(counts.eligible_stories, 1);
    assert_eq!(counts.eligible_steps, 2);
    assert_eq!(counts.candidates, 23);
    assert_eq!(counts.teacher_support_entries, 16);
    let work = prefilter
        .prefilter
        .as_ref()
        .expect("prefilter work metrics");
    assert_eq!(work.work_eligible_steps, counts.eligible_steps);
    assert_eq!(work.total_candidates, counts.candidates);

    let deranged = report.nulls[2]
        .metrics
        .counts
        .as_ref()
        .expect("deranged-support null owns transformed counts");
    assert_eq!(deranged.eligible_stories, 1);
    assert_eq!(deranged.eligible_steps, 4);
    assert_eq!(deranged.candidates, 42);
    assert_eq!(deranged.teacher_support_entries, 25);
}

#[test]
fn known_synthetic_adapter_cannot_be_relabelled_pinned_real() {
    let mut fixture = fixture();
    let observation = bound_observation(&mut fixture);
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("evidence-kind"));
    assert!(report.disposition_reason.contains("cannot relabel"));
}

#[test]
fn closed_evidence_registry_prevents_arbitrary_synthetic_identity_relabeling() {
    let evidence = registered_trace_evidence(
        INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
        INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION,
    )
    .expect("the structural evidence record is registered");
    assert_eq!(evidence.id(), INSTRUMENT_CONFORMANCE_EVIDENCE_ID);
    assert_eq!(evidence.version(), INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION);
    assert_eq!(evidence.kind(), TraceKind::InstrumentConformance);
    assert!(evidence.declared_digest().starts_with("blake3:"));
    assert!(registered_trace_evidence("pinned-real", 1).is_none());
    assert!(registered_trace_evidence(
        INSTRUMENT_CONFORMANCE_EVIDENCE_ID,
        INSTRUMENT_CONFORMANCE_EVIDENCE_VERSION + 1
    )
    .is_none());
    assert!(registered_trace_evidence("invented-real-corpus", 1).is_none());

    let mut fixture = fixture();
    fixture.manifest.source_snapshot = Some(digest("relabelled-teacher-weight-snapshot"));
    fixture.manifest.adapter = Some("arbitrarily-relabelled-real-adapter/999".to_owned());
    fixture.manifest.compiler = Some("arbitrarily-relabelled-real-compiler/999".to_owned());
    fixture.corpus.records_kappa = digest("relabelled-records");
    fixture.corpus.trace_kappa = digest("relabelled-trace");
    fixture.manifest.corpus = Some(fixture.corpus.records_kappa.clone());
    fixture.manifest.trace = Some(fixture.corpus.trace_kappa.clone());

    let mut observation = bound_observation(&mut fixture);
    observation.input_cid = Some(digest("relabelled-input-cid"));
    observation.source_manifest_kappa = Some(digest("relabelled-source-manifest"));
    let tokenizer = observation
        .tokenizer_adapter
        .as_mut()
        .expect("fixture tokenizer");
    tokenizer.tokenizer_cid = digest("relabelled-tokenizer-definition");
    tokenizer.adapter_digest = tokenizer.declared_digest();
    fixture.manifest.tokenizer = Some(tokenizer.declared_digest());
    fixture.corpus.identity_bundle_digest = observation.identity_bundle_digest();

    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("evidence-kind"));
    assert!(report.disposition_reason.contains("cannot relabel"));
    assert!(report
        .arms
        .iter()
        .chain(&report.nulls)
        .all(|row| !matches!(row.verdict, StageVerdict::Pass | StageVerdict::Fail)));
    assert_eq!(
        report.identities.trace_evidence_kind,
        Some(TraceKind::InstrumentConformance)
    );
    assert_eq!(
        report.identities.trace_evidence_id.as_deref(),
        Some(evidence.id())
    );
    assert_eq!(
        report.identities.trace_evidence_version,
        Some(evidence.version())
    );
    assert_eq!(
        report.identities.trace_evidence_digest.as_deref(),
        Some(evidence.declared_digest().as_str())
    );
}

#[test]
fn pinned_real_requires_the_authoritative_manifest_and_exact_bundle_binding() {
    let (fixture, observation) = real_shaped_fixture();
    let missing =
        run_octeract_trace_screen(Some(screen_input(&fixture, None)), TraceKind::PinnedReal);
    assert_unavailable(&missing);
    assert!(missing
        .disposition_reason
        .contains("authoritative observation manifest"));

    let mut mismatched = observation.clone();
    mismatched.input_cid = Some(digest("different-corpus"));
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&mismatched))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("does not bind"));

    let (mut fixture, mut wrong_profile) = real_shaped_fixture();
    wrong_profile.trace_profile = Some(TraceProfile::full(&[0], TEST_TOP_M + 1));
    fixture.corpus.identity_bundle_digest = wrong_profile.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&wrong_profile))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("does not bind"));

    let (fixture, mut wrong_records) = real_shaped_fixture();
    wrong_records.total_records += 1;
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&wrong_records))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("does not bind"));
}

#[test]
fn pinned_real_rejects_unknown_or_tampered_source_provenance_records() {
    let (mut fixture, mut tampered_operator) = real_shaped_fixture();
    tampered_operator
        .attention_operator
        .as_mut()
        .expect("source operator")
        .params
        .score_scale = "tampered-after-capture".to_owned();
    fixture.corpus.identity_bundle_digest = tampered_operator.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&tampered_operator))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("tampered"));

    let (mut fixture, mut unknown_operator) = real_shaped_fixture();
    let operator = unknown_operator
        .attention_operator
        .as_mut()
        .expect("source operator");
    operator.id = "unregistered-source-attention".to_owned();
    operator.version = 99;
    operator.implementation_digest = operator.declared_digest();
    fixture.corpus.identity_bundle_digest = unknown_operator.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&unknown_operator))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report
        .disposition_reason
        .contains("not a registered record"));

    let (mut fixture, mut tampered_tokenizer) = real_shaped_fixture();
    tampered_tokenizer
        .tokenizer_adapter
        .as_mut()
        .expect("tokenizer adapter")
        .adapter_digest = digest("wrong-adapter-digest");
    fixture.corpus.identity_bundle_digest = tampered_tokenizer.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&tampered_tokenizer))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(
        report.disposition_reason.contains("tokenizer adapter"),
        "{}",
        report.disposition_reason
    );

    let (mut fixture, mut unknown_tokenizer) = real_shaped_fixture();
    let tokenizer = unknown_tokenizer
        .tokenizer_adapter
        .as_mut()
        .expect("tokenizer adapter");
    tokenizer.family = "unregistered-tokenizer".to_owned();
    tokenizer.version = 99;
    tokenizer.adapter_digest = tokenizer.declared_digest();
    fixture.corpus.identity_bundle_digest = unknown_tokenizer.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&unknown_tokenizer))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(
        report.disposition_reason.contains("tokenizer adapter"),
        "{}",
        report.disposition_reason
    );

    let (mut fixture, mut tampered_geometry) = real_shaped_fixture();
    tampered_geometry
        .geometry
        .as_mut()
        .expect("source geometry")
        .id = "unregistered-geometry".to_owned();
    fixture.corpus.identity_bundle_digest = tampered_geometry.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&tampered_geometry))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(
        report.disposition_reason.contains("geometry"),
        "{}",
        report.disposition_reason
    );
}

#[test]
fn supplied_observation_requires_input_cid_and_matching_source_geometry() {
    let mut input_fixture = fixture();
    let mut missing_input = bound_observation(&mut input_fixture);
    missing_input.input_cid = None;
    input_fixture.corpus.identity_bundle_digest = missing_input.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&input_fixture, Some(&missing_input))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(
        !report.disposition_reason.contains("evidence-kind"),
        "{}",
        report.disposition_reason
    );

    let mut geometry_fixture = fixture();
    let mut mismatched_geometry = bound_observation(&mut geometry_fixture);
    mismatched_geometry.geometry = Some(GeometryProjection::bucket_average(576, 288));
    geometry_fixture.corpus.identity_bundle_digest = mismatched_geometry.identity_bundle_digest();
    let report = run_octeract_trace_screen(
        Some(screen_input(&geometry_fixture, Some(&mismatched_geometry))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(
        report.disposition_reason.contains("geometry"),
        "{}",
        report.disposition_reason
    );
}

#[test]
fn teacher_snapshot_and_source_manifest_kappas_are_distinct_and_canonical() {
    let mut valid_fixture = fixture();
    let observation = bound_observation(&mut valid_fixture);
    assert_ne!(
        valid_fixture.manifest.source_snapshot,
        observation.source_manifest_kappa
    );
    let report = run_octeract_trace_screen(
        Some(screen_input(&valid_fixture, Some(&observation))),
        TraceKind::InstrumentConformance,
    );
    assert!(report.controls.instrument_valid);
    assert_eq!(
        report.identities.source_snapshot,
        valid_fixture.manifest.source_snapshot
    );
    assert_eq!(
        report.identities.source_manifest_kappa,
        observation.source_manifest_kappa
    );

    for malformed in [None, Some("not-a-canonical-kappa".to_owned())] {
        let mut snapshot_fixture = fixture();
        let observation = bound_observation(&mut snapshot_fixture);
        snapshot_fixture.manifest.source_snapshot = malformed.clone();
        let report = run_octeract_trace_screen(
            Some(screen_input(&snapshot_fixture, Some(&observation))),
            TraceKind::PinnedReal,
        );
        assert_unavailable(&report);
        assert!(
            !report.disposition_reason.contains("evidence-kind"),
            "{}",
            report.disposition_reason
        );

        let mut manifest_fixture = fixture();
        let mut observation = bound_observation(&mut manifest_fixture);
        observation.source_manifest_kappa = malformed.clone();
        manifest_fixture.corpus.identity_bundle_digest = observation.identity_bundle_digest();
        let report = run_octeract_trace_screen(
            Some(screen_input(&manifest_fixture, Some(&observation))),
            TraceKind::PinnedReal,
        );
        assert_unavailable(&report);
        assert!(
            !report.disposition_reason.contains("evidence-kind"),
            "{}",
            report.disposition_reason
        );
    }
}

#[test]
fn target_operator_identity_is_separate_and_tamper_evident() {
    let (mut fixture, observation) = real_shaped_fixture();
    fixture
        .manifest
        .operator
        .as_mut()
        .expect("target operator")
        .params
        .score_scale = "tampered-target-score".to_owned();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report
        .disposition_reason
        .contains("target attention operator"));
    assert_eq!(report.identities.source_attention_operator_digest, None);
    assert_ne!(
        fixture.manifest.operator_identity,
        observation
            .attention_operator
            .as_ref()
            .map(AttentionOperatorSpec::declared_digest)
    );
}

#[test]
fn pinned_real_rejects_fit_manifest_format_and_provenance_label_drift() {
    let (mut fixture, observation) = real_shaped_fixture();
    fixture.manifest.format = "uor-r4-route-fit-manifest/99".to_owned();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("route-fit identities"));

    let (mut fixture, observation) = real_shaped_fixture();
    fixture.manifest.parameters[0].class = "source".to_owned();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("route-fit identities"));
}

#[test]
fn malformed_capture_geometry_is_unavailable_without_panicking() {
    let mut cases = vec![
        (
            "zero layers",
            TraceCaptureGeometry {
                layers: 0,
                heads: 1,
                kv_heads: 1,
                residual_width: 288,
            },
        ),
        (
            "zero heads",
            TraceCaptureGeometry {
                layers: 1,
                heads: 0,
                kv_heads: 1,
                residual_width: 288,
            },
        ),
        (
            "zero kv heads",
            TraceCaptureGeometry {
                layers: 1,
                heads: 1,
                kv_heads: 0,
                residual_width: 288,
            },
        ),
        (
            "zero residual width",
            TraceCaptureGeometry {
                layers: 1,
                heads: 1,
                kv_heads: 1,
                residual_width: 0,
            },
        ),
        (
            "nondivisible kv width",
            TraceCaptureGeometry {
                layers: 1,
                heads: 2,
                kv_heads: 1,
                residual_width: 289,
            },
        ),
        (
            "kv width multiplication overflow",
            TraceCaptureGeometry {
                layers: 1,
                heads: 1,
                kv_heads: 2,
                residual_width: usize::MAX,
            },
        ),
    ];
    #[cfg(target_pointer_width = "64")]
    cases.push((
        "u32-truncating source width",
        TraceCaptureGeometry {
            layers: 1,
            heads: 1,
            kv_heads: 1,
            residual_width: u32::MAX as usize + 1,
        },
    ));

    for (case, geometry) in cases {
        let mut fixture = fixture();
        fixture.corpus.geometry = geometry;
        let report = run_octeract_trace_screen(
            Some(screen_input(&fixture, None)),
            TraceKind::InstrumentConformance,
        );
        assert_unavailable(&report);
        assert!(
            !report.disposition_reason.is_empty(),
            "{case}: {}",
            report.disposition_reason
        );
    }
}

#[test]
fn fully_bound_unregistered_shape_still_cannot_claim_an_empirical_decision() {
    let (fixture, observation) = real_shaped_fixture();
    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, Some(&observation))),
        TraceKind::PinnedReal,
    );
    assert_unavailable(&report);
    assert!(report.disposition_reason.contains("evidence-kind"));
    assert!(report.disposition_reason.contains("cannot relabel"));
    assert!(report
        .arms
        .iter()
        .chain(&report.nulls)
        .all(|row| !matches!(row.verdict, StageVerdict::Pass | StageVerdict::Fail)));
    assert_ne!(
        report.identities.target_operator_digest,
        report.identities.source_attention_operator_digest
    );
    assert_ne!(
        report.identities.source_snapshot,
        report.identities.source_manifest_kappa
    );
}

#[test]
fn scalar_v1_ties_weight9_and_oriented_controls_are_exact() {
    let query = [0u8; ROUTE_CODE_BYTES];
    let keys = [
        code_with(0x01, 0),
        code_with(0x02, 0),
        code_with(0x03, 0),
        code_with(0x07, 0),
    ];
    let scores = score_octeract_step(&query, &keys, 2).expect("bounded step");
    assert_eq!(scores.exact_scores, vec![1, 1, 2, 3]);
    assert_eq!(scores.weight9_scores, scores.exact_scores);
    assert_eq!(scores.oriented_scores, scores.exact_scores);
    assert_eq!(scores.weight9_selected, scores.v1_selected);
    assert_eq!(scores.oriented_selected, scores.v1_selected);
    assert_eq!(
        scores.v1_selected,
        vec![
            ScoredCandidate {
                candidate: 0,
                score: 1,
            },
            ScoredCandidate {
                candidate: 1,
                score: 1,
            },
        ],
        "equal distances retain ascending candidate order"
    );
    assert_eq!(scores.lower_bound.selected, scores.v1_selected);

    assert!(score_octeract_step(&query, &[], 1).is_none());
    assert!(score_octeract_step(&query, &keys, 0).is_none());
    assert!(score_octeract_step(&query, &keys, 5).is_none());
}

#[test]
fn weight9_materializes_blocks_and_scores_independently_of_fold5() {
    let query = [0u8; ROUTE_CODE_BYTES];
    let mut distributed = [0u8; ROUTE_CODE_BYTES];
    distributed[..8].fill(0x01);
    let concentrated = code_with(0xff, 0x00);

    // Both candidates have exact/weight9 score 8, but their block values and
    // fold5 scores differ. Placing the distributed candidate first makes the
    // stable exact tie select candidate 0 while fold5 selects candidate 1.
    // This catches a weight9 control implemented by reusing fold5 values.
    let scored = score_octeract_step(&query, &[distributed, concentrated], 1)
        .expect("bounded discriminating adversary");
    assert_eq!(scored.exact_scores, vec![8, 8]);
    assert_eq!(scored.weight9_scores, vec![8, 8]);
    assert_eq!(scored.folded_scores, vec![8, 0]);
    assert_eq!(scored.weight9_blocks[0][..8], [1u8; 8]);
    assert!(scored.weight9_blocks[0][8..]
        .iter()
        .all(|&value| value == 0));
    assert_eq!(scored.weight9_blocks[1][0], 8);
    assert!(scored.weight9_blocks[1][1..]
        .iter()
        .all(|&value| value == 0));
    assert_eq!(
        scored.weight9_scores,
        scored
            .weight9_blocks
            .iter()
            .map(|blocks| blocks.iter().map(|&value| u32::from(value)).sum())
            .collect::<Vec<u32>>()
    );
    assert_eq!(scored.weight9_selected, scored.v1_selected);
    assert_eq!(scored.weight9_selected[0].candidate, 0);
    assert_eq!(scored.folded_selected[0].candidate, 1);
}

#[test]
fn fold_complement_adversaries_and_prefilter_outcomes_are_explicit() {
    let fixture = fixture();
    let head = fixture.fitted.head(0, 0).expect("fixture head");
    let query = &head.query_codes[0][0];
    let keys = &head.key_codes[0];
    let scores = score_octeract_step(query, keys, TEST_TOP_M as usize).expect("bounded step");

    // Distance 1 and distance 7 are the same fold shell while the oriented
    // representation reconstructs both exact weights.
    assert_eq!(scores.fold_classes[1], scores.fold_classes[3]);
    assert_ne!(scores.exact_scores[1], scores.exact_scores[3]);
    assert_eq!(scores.oriented_weights[1][0], 1);
    assert_eq!(scores.oriented_weights[3][0], 7);
    assert_eq!(scores.oriented_scores, scores.exact_scores);
    assert_ne!(scores.folded_selected, scores.v1_selected);

    // N=12 => P=floor(3N/4)=9. The true exact winners survive this
    // shortlist, so refinement reproduces V1 at exactly the 0.75 work cap.
    assert_eq!(scores.prefilter.shortlist.len(), 9);
    assert_eq!(scores.prefilter.selected, scores.v1_selected);
    assert_eq!(scores.prefilter.exact_refinement_candidates, 9);
    assert_eq!(scores.prefilter.exact_refinement_candidates_avoided, 3);

    // Adversarial tie: six early distance-8 keys have fold score zero and
    // crowd the exact distance-0/1 winners out of P=6. This is a valid
    // fidelity failure, not a reason to tune P after inspection.
    let key_from_delta =
        |delta: [u8; ROUTE_CODE_BYTES]| core::array::from_fn(|block| query[block] ^ delta[block]);
    let mut bad_keys = vec![key_from_delta(code_with(0xff, 0)); 6];
    bad_keys.push(key_from_delta(code_with(0x00, 0)));
    bad_keys.push(key_from_delta(code_with(0x01, 0)));
    let bad = score_octeract_step(query, &bad_keys, 2).expect("bounded adversary");
    assert_eq!(
        bad.v1_selected
            .iter()
            .map(|entry| entry.candidate)
            .collect::<Vec<_>>(),
        vec![6, 7]
    );
    assert_eq!(bad.prefilter.shortlist, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(
        bad.prefilter
            .selected
            .iter()
            .map(|entry| entry.candidate)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let no_work = score_octeract_step(query, &bad_keys[..2], 2).expect("bounded no-work step");
    assert!(no_work.prefilter.shortlist.is_empty());
    assert_eq!(no_work.prefilter.exact_refinement_candidates, 0);
}

#[test]
fn safe_lower_bound_skips_equal_later_ties_without_changing_v1() {
    let query = [0u8; ROUTE_CODE_BYTES];
    let keys = [
        code_with(0x00, 0),
        code_with(0x01, 0),
        code_with(0x02, 0),
        code_with(0x03, 0),
    ];
    let scores = score_octeract_step(&query, &keys, 2).expect("bounded step");
    assert_eq!(scores.lower_bound.selected, scores.v1_selected);
    assert_eq!(scores.lower_bound.lower_bounds, scores.exact_scores);
    assert_eq!(scores.lower_bound.exact_evaluations, 2);
    assert_eq!(scores.lower_bound.exact_evaluations_avoided, 2);
    assert_eq!(
        scores.lower_bound.lower_bounds[2], scores.v1_selected[1].score,
        "candidate 2 is an equal-distance later tie and may be skipped safely"
    );
}

#[test]
fn nulls_are_deterministic_occupancy_preserving_and_nonidentity() {
    let fixture = fixture();
    let head = fixture.fitted.head(0, 0).expect("fixture head");
    let scores = score_octeract_step(
        &head.query_codes[0][0],
        &head.key_codes[0],
        TEST_TOP_M as usize,
    )
    .expect("bounded step");
    assert!(exhaustive_anchor_relabel_control(
        &scores,
        TEST_TOP_M as usize
    ));
    let first = occupancy_matched_fold_null(&scores.fold_classes, OCCUPANCY_MATCHED_FOLD_SEED);
    let second = occupancy_matched_fold_null(&scores.fold_classes, OCCUPANCY_MATCHED_FOLD_SEED);
    assert_eq!(first, second);
    assert_ne!(first, scores.fold_classes);
    for block in 0..ROUTE_CODE_BYTES {
        let mut original: Vec<u8> = scores.fold_classes.iter().map(|row| row[block]).collect();
        let mut permuted: Vec<u8> = first.iter().map(|row| row[block]).collect();
        original.sort_unstable();
        permuted.sort_unstable();
        assert_eq!(permuted, original, "block {block} occupancy");
    }

    let permutation = shuffled_block_permutation(SHUFFLED_BLOCK_SEED);
    assert_eq!(permutation, shuffled_block_permutation(SHUFFLED_BLOCK_SEED));
    assert!(
        permutation
            .iter()
            .enumerate()
            .any(|(index, &value)| index != usize::from(value)),
        "the block null is required to be nonidentity"
    );
    let mut sorted = permutation;
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        core::array::from_fn::<_, ROUTE_CODE_BYTES, _>(|index| index as u8)
    );
    let query: [u8; ROUTE_CODE_BYTES] = core::array::from_fn(|index| index as u8);
    let aligned = query;
    let shuffled: [u8; ROUTE_CODE_BYTES] =
        core::array::from_fn(|index| aligned[usize::from(permutation[index])]);
    let aligned_score = score_octeract_step(&query, &[aligned], 1).expect("aligned step");
    let shuffled_score = score_octeract_step(&query, &[shuffled], 1).expect("shuffled step");
    assert_eq!(aligned_score.exact_scores, vec![0]);
    assert!(shuffled_score.exact_scores[0] > 0);

    let supports = vec![vec![0, 1], vec![2, 3], vec![4, 5]];
    assert_eq!(
        deranged_supports(&supports),
        vec![vec![2, 3], vec![4, 5], vec![0, 1]]
    );
    assert_ne!(deranged_supports(&supports), supports);
}

#[test]
fn a_vacuous_occupancy_null_makes_the_screen_unavailable() {
    let mut fixture = fixture();
    let query = fixture.fitted.heads[0].query_codes[0][0];
    fixture.fitted.heads[0].key_codes[0].fill(query);

    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, None)),
        TraceKind::InstrumentConformance,
    );
    assert!(!report.controls.occupancy_null_transformation_distinct);
    assert!(!report.controls.occupancy_null_result_distinct);
    assert_unavailable(&report);
    assert!(report.arms[1..]
        .iter()
        .all(|row| row.verdict == StageVerdict::NotRun));
    assert!(report
        .nulls
        .iter()
        .all(|row| row.verdict == StageVerdict::NotRun));
}

#[test]
fn a_vacuous_shuffled_block_null_makes_the_screen_unavailable() {
    let mut fixture = fixture();
    fixture.fitted.heads[0].query_codes[0].fill([0u8; ROUTE_CODE_BYTES]);
    let uniform_bytes = [
        0x00, 0x01, 0x03, 0x05, 0x06, 0x09, 0x0a, 0x0c, 0x0f, 0x17, 0x1b, 0x1d,
    ];
    for (key, value) in fixture.fitted.heads[0].key_codes[0]
        .iter_mut()
        .zip(uniform_bytes)
    {
        key.fill(value);
    }

    let report = run_octeract_trace_screen(
        Some(screen_input(&fixture, None)),
        TraceKind::InstrumentConformance,
    );
    assert!(!report.controls.shuffled_block_null_transformation_distinct);
    assert!(!report.controls.shuffled_block_null_result_distinct);
    assert_unavailable(&report);
    assert!(report.arms[1..]
        .iter()
        .all(|row| row.verdict == StageVerdict::NotRun));
    assert!(report
        .nulls
        .iter()
        .all(|row| row.verdict == StageVerdict::NotRun));
}

#[test]
fn every_bijective_anchor_relabel_preserves_fold_signature_groups() {
    let fixture = fixture();
    let head = fixture.fitted.head(0, 0).expect("fixture head");
    let scores = score_octeract_step(
        &head.query_codes[0][0],
        &head.key_codes[0],
        TEST_TOP_M as usize,
    )
    .expect("bounded step");
    fn next_permutation(values: &mut [u8]) -> bool {
        let Some(pivot) = (0..values.len().saturating_sub(1))
            .rev()
            .find(|&index| values[index] < values[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..values.len())
            .rev()
            .find(|&index| values[pivot] < values[index])
            .expect("a pivot has a successor");
        values.swap(pivot, successor);
        values[pivot + 1..].reverse();
        true
    }

    fn selected(scores: &[u32], m: usize) -> Vec<ScoredCandidate> {
        let mut ranked: Vec<_> = scores
            .iter()
            .enumerate()
            .map(|(candidate, &score)| ScoredCandidate {
                candidate: candidate as u32,
                score,
            })
            .collect();
        ranked.sort_unstable_by_key(|entry| (entry.score, entry.candidate));
        ranked.truncate(m);
        ranked
    }

    fn signature_groups(
        rows: &[[u8; ROUTE_CODE_BYTES]],
    ) -> BTreeMap<[u8; ROUTE_CODE_BYTES], Vec<u32>> {
        let mut groups = BTreeMap::new();
        for (candidate, row) in rows.iter().copied().enumerate() {
            groups
                .entry(row)
                .or_insert_with(Vec::new)
                .push(candidate as u32);
        }
        groups
    }

    let expected_scores: Vec<u32> = scores
        .fold_classes
        .iter()
        .map(|row| row.iter().map(|&class| u32::from(class)).sum())
        .collect();
    assert_eq!(expected_scores, scores.folded_scores);
    let expected_selected = selected(&expected_scores, TEST_TOP_M as usize);
    assert_eq!(expected_selected, scores.folded_selected);
    let expected_groups = signature_groups(&scores.fold_classes);

    let symbols = [0u8, 127, 189, 217, 225];
    let mut permutation = [0u8, 1, 2, 3, 4];
    let mut checked = 0u32;
    loop {
        let encoded: Vec<[u8; ROUTE_CODE_BYTES]> = scores
            .fold_classes
            .iter()
            .map(|row| row.map(|class| symbols[permutation[class as usize] as usize]))
            .collect();
        let mut inverse = [0u8; 256];
        for (class, &permuted_class) in permutation.iter().enumerate() {
            inverse[symbols[permuted_class as usize] as usize] = class as u8;
        }
        let decoded: Vec<[u8; ROUTE_CODE_BYTES]> = encoded
            .iter()
            .map(|row| row.map(|label| inverse[label as usize]))
            .collect();
        let decoded_scores: Vec<u32> = decoded
            .iter()
            .map(|row| row.iter().map(|&class| u32::from(class)).sum())
            .collect();

        assert_eq!(decoded, scores.fold_classes);
        assert_eq!(decoded_scores, scores.folded_scores);
        assert_eq!(
            selected(&decoded_scores, TEST_TOP_M as usize),
            expected_selected
        );
        assert_eq!(signature_groups(&decoded), expected_groups);
        checked += 1;
        if !next_permutation(&mut permutation) {
            break;
        }
    }
    assert_eq!(checked, 120, "all 5! anchor bijections are exercised");
}

fn brute_force_oracle(groups: &[CollisionGroup], m: usize) -> u32 {
    fn visit(
        groups: &[CollisionGroup],
        m: usize,
        order: &mut Vec<usize>,
        used: &mut [bool],
        best: &mut u32,
    ) {
        if order.len() == groups.len() {
            let hits = order
                .iter()
                .flat_map(|&index| groups[index].teacher_membership.iter().copied())
                .take(m)
                .filter(|&member| member)
                .count() as u32;
            *best = (*best).max(hits);
            return;
        }
        for index in 0..groups.len() {
            if !used[index] {
                used[index] = true;
                order.push(index);
                visit(groups, m, order, used, best);
                order.pop();
                used[index] = false;
            }
        }
    }

    let mut best = 0;
    visit(
        groups,
        m,
        &mut Vec::new(),
        &mut vec![false; groups.len()],
        &mut best,
    );
    best
}

fn groups_from_masks(total: usize, boundaries: usize, memberships: usize) -> Vec<CollisionGroup> {
    let mut groups = Vec::new();
    let mut start = 0;
    for candidate in 0..total {
        let ends_group = candidate + 1 == total || (boundaries & (1 << candidate)) != 0;
        if ends_group {
            groups.push(CollisionGroup {
                candidate_ids: (start..=candidate).map(|value| value as u32).collect(),
                teacher_membership: (start..=candidate)
                    .map(|value| (memberships & (1 << value)) != 0)
                    .collect(),
            });
            start = candidate + 1;
        }
    }
    groups
}

#[test]
fn collision_oracle_matches_bruteforce_over_exhaustive_small_domains() {
    for total in 1..=6usize {
        for boundaries in 0..(1usize << total.saturating_sub(1)) {
            for memberships in 0..(1usize << total) {
                let groups = groups_from_masks(total, boundaries, memberships);
                for m in 1..=total.min(3) {
                    let oracle = collision_oracle(&groups, m).expect("bounded oracle domain");
                    assert_eq!(oracle.selected_slots, m as u32);
                    assert_eq!(
                        oracle.max_teacher_hits,
                        brute_force_oracle(&groups, m),
                        "total={total} boundaries={boundaries:#x} memberships={memberships:#x} m={m}"
                    );
                }
            }
        }
    }

    let malformed = CollisionGroup {
        candidate_ids: vec![1, 0],
        teacher_membership: vec![true, false],
    };
    assert!(collision_oracle(std::slice::from_ref(&malformed), 1).is_none());
    let duplicate_across_groups = [
        CollisionGroup {
            candidate_ids: vec![0, 2],
            teacher_membership: vec![true, false],
        },
        CollisionGroup {
            candidate_ids: vec![1, 2],
            teacher_membership: vec![false, true],
        },
    ];
    assert!(
        collision_oracle(&duplicate_across_groups, 2).is_none(),
        "one candidate cannot occupy two complete-signature groups"
    );
    assert!(collision_oracle(&[], 0).is_none());
}
