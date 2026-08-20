//! #832 — CID-bound capability suites and per-token resolution
//! attribution: fail-closed evidence for the evaluation constitution.
//!
//! Record: `docs/capability_suites_832.md`. Decision it binds to:
//! ADR-0001 (`docs/adr/0001-normative-r4g1-scorer.md`, #831).
//!
//! These tests exercise the committed constitution
//! (`uor_r4_api::capability_suite`) as the acceptance evidence for #832:
//!   * the committed manifests and constitution parse and validate;
//!   * a small committed fixture replays through the **reference**
//!     (`R4G1Runtime` / `GraphScorer`) and the **production**
//!     (`R4Engine`) paths, and every production token is attributed to a
//!     normative [`ResolutionPath`] bound to the normative scorer id;
//!   * planted document leakage, a CID mismatch, a degenerate control,
//!     and a wrong path attribution are each rejected;
//!   * an absent fixture is visible as `Unavailable`, never a value.

use std::collections::BTreeMap;

use uor_r4_api::capability_suite::{
    builtin_constitution, builtin_manifests, compute_cid, detect_document_leakage,
    is_degenerate_control, verify_cid, AttributionHistogram, CapabilityReport, ControlKind,
    ControlReport, MetricReport, MetricStatus, ResolutionPath, ScoringMode, Stage, SuiteIdentities,
    SuiteManifest, TokenAttribution, Workload, CAPABILITY_REPORT_SCHEMA, NORMATIVE_SCORER_ID,
};
use uor_r4_api::{EngineParts, PredictDecision, R4Engine};

use uor_r4_graph_certify::{GraphScorer, DEFAULT_EXCT_TOP_X, DEFAULT_ROOT_TOP_B};
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::R4G1Runtime;

/// A small, self-contained R4G1 bundle both scorers read — the synthetic
/// store recipe shared with `normative_scorer_831.rs`. Returns
/// `(r4g1_bytes, teacher_bytes)`; the teacher container carries the
/// pinned CID the reference scorer's EXCT path verifies fail-closed.
fn synthetic_bundle() -> (Vec<u8>, Vec<u8>) {
    use uor_r4_core::transformerless::compiler::{self, STAGES};
    use uor_r4_core::transformerless::{convert_r4g1, runtime};

    let dir = env!("CARGO_MANIFEST_DIR");
    let art_bytes = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture artifacts present");
    let artifacts = compiler::parse_artifacts(&art_bytes).expect("artifacts parse");

    let mut store: runtime::Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
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
    let store_bytes = runtime::store_bytes(&store);
    let r4g1 = convert_r4g1::convert(&art_bytes, &artifacts, &store, &store_bytes, None)
        .expect("convert to R4G1")
        .0;
    (r4g1, art_bytes)
}

/// The token windows the fixture is replayed on (small in-vocab ids).
const WINDOWS: &[&[u32]] = &[
    &[1, 2, 3],
    &[3, 1, 4],
    &[1, 4, 1],
    &[5, 9, 2],
    &[7, 5, 9],
    &[7, 5, 8],
    &[2, 3, 1],
    &[1, 1, 2],
];

// --- the committed constitution parses, validates, and is complete -----------

#[test]
fn committed_constitution_validates_and_names_a_primary_suite_per_stage() {
    let manifests = builtin_manifests();
    for m in &manifests {
        assert_eq!(m.validate(), None, "manifest {:?} validates", m.id);
    }
    let constitution = builtin_constitution();
    assert_eq!(
        constitution.validate(&manifests),
        None,
        "the constitution validates against the committed manifests"
    );
    // Acceptance criterion 1: every stage names a frozen primary suite,
    // split, control set, report schema, and promotion statistic.
    for stage in Stage::ALL {
        let entry = &constitution.stages[&stage];
        let m = manifests
            .iter()
            .find(|m| m.id == entry.primary_suite)
            .expect("primary suite has a manifest");
        assert!(!m.primary_metric.is_empty() && !m.promotion_statistic.is_empty());
        assert!(!m.controls.is_empty());
        assert_eq!(m.report_schema, CAPABILITY_REPORT_SCHEMA);
        assert!(m.split.leakage_check && m.split.axis_count() > 0);
    }
}

// --- the reference path replays the same committed fixture --------------------

#[test]
fn reference_path_replays_the_committed_fixture_deterministically() {
    let (r4g1, teacher) = synthetic_bundle();

    // Deployed normative runtime is reachable and deterministic on the
    // committed fixture bytes.
    let runtime = R4G1Runtime::parse(&r4g1).expect("normative runtime parses the fixture");
    assert!(runtime.node_count() > 0);
    for window in WINDOWS {
        let mut a = vec![ScoreQ::MIN; runtime.node_count() as usize];
        let mut b = vec![ScoreQ::MIN; runtime.node_count() as usize];
        assert_eq!(
            runtime.predict_distribution(window, None, &mut a),
            runtime.predict_distribution(window, None, &mut b),
            "the reference runtime is deterministic on {window:?}"
        );
    }

    // The reference/certifier scorer is reachable from the SAME fixture
    // bytes (with the pinned teacher container its EXCT path checks).
    let scorer = GraphScorer::from_artifact(
        &r4g1,
        Some(&teacher),
        DEFAULT_ROOT_TOP_B,
        DEFAULT_EXCT_TOP_X,
    );
    assert!(
        scorer.is_some(),
        "the reference scorer constructs from the same committed fixture"
    );
}

// --- helper: build a production report from real R4Engine decisions ----------

/// Replay `WINDOWS` through the deployed [`R4Engine`] and build a
/// [`CapabilityReport`] whose per-token attribution is derived from real
/// deployed decisions, bound to the normative scorer identity.
fn replay_production_report(r4g1: &[u8], teacher: &[u8]) -> CapabilityReport {
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: r4g1,
        signature_artifact: teacher,
        tokenizer: None,
        score_report: None,
    })
    .expect("R4Engine loads the committed fixture");

    let mut attribution = AttributionHistogram::default();
    let mut per_token = Vec::new();
    for (position, window) in WINDOWS.iter().enumerate() {
        let decision: PredictDecision = engine
            .predict_decision(window)
            .expect("in-vocab window decides");
        let attr = TokenAttribution::from_decision(position as u32, &decision);
        attribution.record(attr.path);
        per_token.push(attr);
    }

    let slice_cid = compute_cid(
        &WINDOWS
            .iter()
            .flat_map(|w| w.iter().flat_map(|t| t.to_le_bytes()))
            .collect::<Vec<u8>>(),
    );
    let identities = SuiteIdentities {
        teacher: Some(compute_cid(teacher)),
        tokenizer: Some(compute_cid(teacher)),
        corpus: Some(slice_cid.clone()),
        compiler: Some("synthetic-convert-r4g1".to_string()),
        artifact: Some(compute_cid(r4g1)),
        decoder: Some("argmax".to_string()),
        ..Default::default()
    };
    let served = attribution.served();
    let total = attribution.total();

    CapabilityReport {
        schema: CAPABILITY_REPORT_SCHEMA,
        suite_id: "s0-broad-text".to_string(),
        stage: Stage::S0,
        workload: Workload::BroadText,
        mode: ScoringMode::TeacherForced,
        execution_scope: "offline replay + measured R4Engine reachability".to_string(),
        slice_partition_cid: Some(slice_cid),
        identities,
        metrics: vec![
            // The manifest's primary is not scored here (this replay
            // attributes tokens; it does not measure held-out top-1),
            // so it is honestly NotRun rather than a fabricated value.
            MetricReport {
                name: "held-out-top1".to_string(),
                mode: ScoringMode::TeacherForced,
                status: MetricStatus::NotRun,
                sample_n: total,
                ci_low_permille: None,
                ci_high_permille: None,
                primary: true,
            },
            MetricReport {
                name: "served-fraction".to_string(),
                mode: ScoringMode::TeacherForced,
                status: MetricStatus::Measured {
                    numerator: served,
                    denominator: total,
                },
                sample_n: total,
                ci_low_permille: None,
                ci_high_permille: None,
                primary: false,
            },
        ],
        controls: vec![
            ControlReport {
                kind: ControlKind::ExctDisabled,
                status: MetricStatus::NotRun,
                note: None,
            },
            ControlReport {
                kind: ControlKind::SuffixOnly,
                status: MetricStatus::NotRun,
                note: None,
            },
            ControlReport {
                kind: ControlKind::TrivialPrior,
                status: MetricStatus::NotRun,
                note: None,
            },
        ],
        attribution,
        per_token,
        notes: Some("attribution replay of the committed synthetic fixture".to_string()),
    }
}

fn broad_text_manifest() -> SuiteManifest {
    builtin_manifests()
        .into_iter()
        .find(|m| m.id == "s0-broad-text")
        .expect("s0 broad-text manifest is committed")
}

// --- production replay: every token attributed to the normative scorer -------

#[test]
fn production_replay_attributes_every_token_to_the_normative_scorer() {
    let (r4g1, teacher) = synthetic_bundle();
    let report = replay_production_report(&r4g1, &teacher);

    assert_eq!(report.validate(), None, "the production report validates");
    assert_eq!(
        report.validate_against(&broad_text_manifest()),
        None,
        "the production report agrees with its manifest"
    );
    // Non-vacuous: real decisions were produced and attributed.
    assert_eq!(report.attribution.total(), WINDOWS.len() as u64);
    assert!(report.primary_metric().is_some());

    // Acceptance criterion 2: every reported production token is
    // attributable to a normative resolution path and scorer identity.
    for t in &report.per_token {
        assert_eq!(
            t.validate(),
            None,
            "token {} attributes cleanly",
            t.position
        );
        assert_eq!(
            t.scorer_id, NORMATIVE_SCORER_ID,
            "every production token binds the normative scorer id"
        );
    }
}

#[test]
fn production_report_is_byte_deterministic() {
    let (r4g1_a, teacher_a) = synthetic_bundle();
    let (r4g1_b, teacher_b) = synthetic_bundle();
    let a = replay_production_report(&r4g1_a, &teacher_a).to_canonical_json();
    let b = replay_production_report(&r4g1_b, &teacher_b).to_canonical_json();
    assert_eq!(
        a, b,
        "identical committed inputs produce identical report bytes"
    );
}

// --- CID tamper, leakage, degenerate control ---------------------------------

#[test]
fn committed_fixture_cid_detects_a_tampered_byte() {
    let (mut r4g1, _teacher) = synthetic_bundle();
    let cid = compute_cid(&r4g1);
    assert_eq!(verify_cid(&cid, &r4g1), None);
    r4g1[0] ^= 0x01;
    assert!(
        verify_cid(&cid, &r4g1).is_some(),
        "a single flipped byte fails the content check"
    );
}

#[test]
fn planted_document_leakage_is_rejected() {
    assert!(
        detect_document_leakage(&["doc-a", "doc-b"], &["doc-c", "doc-d"]).is_none(),
        "document-disjoint partitions pass"
    );
    assert!(
        detect_document_leakage(&["doc-a", "doc-b"], &["doc-b", "doc-c"]).is_some(),
        "a document in both partitions is rejected"
    );
}

#[test]
fn degenerate_control_is_flagged_and_a_separated_one_is_not() {
    let primary = MetricStatus::Measured {
        numerator: 500,
        denominator: 1000,
    };
    let degenerate = MetricStatus::Measured {
        numerator: 502,
        denominator: 1000,
    };
    let separated = MetricStatus::Measured {
        numerator: 100,
        denominator: 1000,
    };
    assert!(is_degenerate_control(&primary, &degenerate, 5));
    assert!(!is_degenerate_control(&primary, &separated, 5));
}

// --- path-attribution and fixture-absence negatives --------------------------

#[test]
fn a_served_token_bound_to_a_wrong_scorer_is_rejected() {
    let served = TokenAttribution {
        position: 0,
        token: 5,
        path: ResolutionPath::Graph,
        scorer_id: "an-alternate-scorer".to_string(),
        widened: false,
    };
    assert!(
        served.validate().is_some(),
        "a served token must bind the normative scorer, not an alternate"
    );

    // A histogram that disagrees with the embedded per-token tally is a
    // report-level path-attribution error.
    let (r4g1, teacher) = synthetic_bundle();
    let mut report = replay_production_report(&r4g1, &teacher);
    report.attribution.graph += 1; // no longer matches per_token
    assert!(
        report.validate().is_some(),
        "a miscounted attribution histogram is rejected"
    );
}

#[test]
fn an_absent_required_fixture_is_unavailable_never_a_value() {
    let (r4g1, teacher) = synthetic_bundle();
    let manifest = broad_text_manifest();

    // Baseline: with the primary NotRun and every identity present, valid.
    let report = replay_production_report(&r4g1, &teacher);
    assert_eq!(report.validate_against(&manifest), None);

    // Drop a required identity and (dishonestly) report a Measured
    // primary -> rejected: an absent fixture may not become a value.
    let mut measured_absent = replay_production_report(&r4g1, &teacher);
    measured_absent.identities.corpus = None;
    measured_absent.metrics[0].status = MetricStatus::Measured {
        numerator: 100,
        denominator: 1000,
    };
    assert!(
        measured_absent.validate_against(&manifest).is_some(),
        "a Measured primary with an absent required identity is rejected"
    );

    // The honest form: the primary is Unavailable when the fixture is
    // absent, which validates.
    let mut unavailable = replay_production_report(&r4g1, &teacher);
    unavailable.identities.corpus = None;
    unavailable.metrics[0].status = MetricStatus::Unavailable {
        reason: "corpus partition fixture absent".to_string(),
    };
    assert_eq!(
        unavailable.validate_against(&manifest),
        None,
        "an Unavailable primary is the honest encoding of an absent fixture"
    );
}
