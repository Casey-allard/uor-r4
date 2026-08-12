//! #605 replacement-ladder tests, certify side: deterministic
//! double-run of the ladder, one-head-vs-nulls under the PRE-REGISTERED
//! contract, anti-vacuity and the deliberately-broken fit, the
//! stop-at-first-failure exit rule, UNAVAILABLE stages with reasons,
//! and absent-as-absent serialization round-trips.
//!
//! The synthetic arm's fixture (teacher + mini-corpus + fit) is built
//! ONCE per process through the production #603/#605 surfaces and
//! shared read-only across tests. Everything measured is deterministic
//! (integer-seeded teacher and stream, no clock); the only
//! nondeterminism in this file is temp-directory naming, which never
//! reaches a measured byte.
//!
//! ## Measured values at pin time (2026-08-12 fixture, seed constants
//! as shipped)
//!
//! ```text
//! instrument_valid = true
//! null:        overlap 0.5718  n1 0.2019  n2 0.2353  top1 1.0000  ratio 1.0000  PASS
//! one-head:    overlap 0.5718  n1 0.2019  n2 0.2353  top1 0.9941  ratio 0.9999  PASS
//! one-layer:   overlap 0.5730  n1 0.2039  n2 0.2312  top1 0.9922  ratio 1.0000  PASS
//! layer-range: overlap 0.5872  n1 0.2064  n2 0.2537  top1 0.9951  ratio 1.0001  PASS
//! whole-model: overlap 0.5801  n1 0.2052  n2 0.2424  top1 0.9902  ratio 1.0001  PASS
//! real-teacher / real-corpus: UNAVAILABLE (prerequisites named)
//! teacher bits/token 4.9709; top-k agreement 1.0 at every scope
//! ```
//!
//! The pinned verdict assertions below reflect THESE measured numbers
//! against the pre-registered margins; the consistency assertions would
//! hold (and are what matters) under either outcome — a fitted arm that
//! missed the margins would be a valid negative result, pinned as FAIL.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use uor_r4_graph_certify::route_fit_report::{
    canonical_route_fit_report_bytes, preregistered_route_fit_contract, route_fit_report_kappa,
    run_route_fit_ladder, RealArmProbe, RouteFitReport, RunContract, StageRecord, StageVerdict,
    NULL_N1_ID, NULL_N2_ID, REPLACEMENT_SEMANTICS_ID, ROUTE_FIT_REPORT_SCHEMA,
    STAGE_KIND_SYNTHETIC, STAGE_LAYER_RANGE, STAGE_NULL, STAGE_ONE_HEAD, STAGE_ONE_LAYER,
    STAGE_REAL_CORPUS, STAGE_REAL_TEACHER, STAGE_WHOLE_MODEL,
};
use uor_r4_graph_compiler::route_fit::{
    fit_route_codes, generate_synthetic_route_trace, load_route_trace_corpus,
    synthetic_capture_geometry, synthetic_fit_manifest, FitManifest, FittedRouteCodes,
    RouteTraceCorpus, SyntheticRouteTeacher,
};
use uor_r4_model_source::conformance::ConformanceStatus;
use uor_r4_model_source::TeacherOracle;

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("uor-r4-{name}-{nanos}"))
}

struct Fixture {
    corpus: RouteTraceCorpus,
    fitted: FittedRouteCodes,
    manifest: FitManifest,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let dir = unique_path("route-fit-ladder-fixture");
        generate_synthetic_route_trace(&dir).expect("synthetic trace corpus");
        let teacher = SyntheticRouteTeacher::new();
        let corpus = load_route_trace_corpus(
            &dir,
            synthetic_capture_geometry(),
            teacher.bos_token() as u32,
        )
        .expect("corpus loads");
        let fitted = fit_route_codes(&corpus).expect("fit succeeds");
        let manifest = synthetic_fit_manifest(&corpus, &teacher.kappa()).expect("manifest");
        let _ = std::fs::remove_dir_all(&dir);
        Fixture {
            corpus,
            fitted,
            manifest,
        }
    })
}

fn run_ladder(contract: &RunContract, fitted: &FittedRouteCodes) -> RouteFitReport {
    let fixture = fixture();
    let mut teacher = SyntheticRouteTeacher::new();
    run_route_fit_ladder(
        &mut teacher,
        &fixture.corpus,
        fitted,
        &fixture.manifest,
        contract,
        &RealArmProbe::from_env(),
    )
    .expect("ladder runs")
}

/// The pre-registered-contract report, run once and shared.
fn preregistered_report() -> &'static RouteFitReport {
    static REPORT: OnceLock<RouteFitReport> = OnceLock::new();
    REPORT.get_or_init(|| run_ladder(&preregistered_route_fit_contract(), &fixture().fitted))
}

fn stage<'a>(report: &'a RouteFitReport, name: &str) -> &'a StageRecord {
    report
        .stages
        .iter()
        .find(|record| record.stage == name)
        .unwrap_or_else(|| panic!("stage {name} present"))
}

/// Verdict-vs-numbers consistency: every stage's verdict must be
/// derivable from its own recorded numbers and the report's contract.
/// This is the assertion that holds under ANY empirical outcome — a
/// report may never claim a verdict its numbers do not support.
fn assert_report_consistent(report: &RouteFitReport) {
    let gates = &report.contract.gates;
    let mut stopped = false;
    for record in &report.stages {
        match record.verdict {
            StageVerdict::NotRun => {
                assert!(
                    stopped,
                    "stage {} is NOT_RUN but no earlier stage failed",
                    record.stage
                );
                // Absent-as-absent: an unevaluated stage carries no
                // measurement at all.
                assert!(record.overlap.is_none());
                assert!(record.teacher.is_none());
                assert!(record.replaced_metrics.is_none());
                assert!(record.runtime.is_none());
                assert!(record.preflight.is_none());
                assert!(record.fit_manifest_kappa.is_none());
            }
            StageVerdict::Unavailable => {
                assert!(
                    !record.reason.is_empty(),
                    "UNAVAILABLE must name its missing prerequisite"
                );
                assert!(record.overlap.is_none());
                assert!(record.replaced_metrics.is_none());
            }
            StageVerdict::Pass | StageVerdict::Fail => {
                assert_eq!(record.kind, STAGE_KIND_SYNTHETIC);
                assert!(!stopped, "an evaluated stage after the ladder exited");
                let overlap = record.overlap.as_ref().expect("evaluated overlap");
                let replaced = record
                    .replaced_metrics
                    .as_ref()
                    .expect("evaluated replaced row");
                let runtime = record.runtime.as_ref().expect("evaluated runtime checks");
                let preflight = record.preflight.as_ref().expect("evaluated preflight");
                let ratio = record.bits_per_token_ratio.expect("evaluated bits ratio");
                let overlap_gate_holds = if record.replaced.is_empty() {
                    true // the null stage replaces nothing
                } else {
                    let bar =
                        (gates.overlap_null_factor * overlap.best_null).max(gates.overlap_floor);
                    overlap.fitted >= bar
                };
                let expect_pass = !overlap.vacuous
                    && preflight.status == ConformanceStatus::Pass
                    && runtime.pass
                    && overlap_gate_holds
                    && replaced.top1_agreement >= gates.min_top1_agreement
                    && ratio <= gates.max_bits_per_token_ratio;
                assert_eq!(
                    record.verdict == StageVerdict::Pass,
                    expect_pass,
                    "stage {} verdict {:?} disagrees with its recorded numbers",
                    record.stage,
                    record.verdict
                );
                if record.verdict == StageVerdict::Fail {
                    stopped = true;
                }
                // A vacuous scope must invalidate the whole run.
                if overlap.vacuous {
                    assert!(!report.instrument_valid);
                }
            }
        }
    }
    // instrument_valid is exactly "no evaluated scope was vacuous".
    let any_vacuous = report
        .stages
        .iter()
        .filter_map(|record| record.overlap.as_ref())
        .any(|overlap| overlap.vacuous);
    assert_eq!(report.instrument_valid, !any_vacuous);
}

/// Test 1 (task list), ladder half: two in-process ladder runs over the
/// same fitted artifact produce byte-identical canonical reports and
/// equal κs.
#[test]
fn ladder_double_run_is_byte_identical() {
    let first = preregistered_report();
    let second = run_ladder(&preregistered_route_fit_contract(), &fixture().fitted);
    assert_eq!(
        canonical_route_fit_report_bytes(first),
        canonical_route_fit_report_bytes(&second),
        "the ladder must be deterministic"
    );
    assert_eq!(
        route_fit_report_kappa(first),
        route_fit_report_kappa(&second)
    );
    assert_eq!(first.schema, ROUTE_FIT_REPORT_SCHEMA);
    assert_eq!(first.fit_manifest_kappa, fixture().manifest.kappa());
    assert_eq!(first.fitted_params_kappa, fixture().fitted.kappa());
}

/// Test 2 (task list): the one-head arm against both pre-registered
/// nulls. The consistency assertion (`assert_report_consistent`) is the
/// binding one — the verdict must match the measured numbers under the
/// pre-registered contract, whatever they are. The verdict pins below
/// record the EMPIRICAL outcome measured at pin time (module docs):
/// the fitted one-head overlap 0.5718 cleared max(2 x 0.2353, 0.5) =
/// 0.5, top-1 0.9941 cleared 0.90, bits ratio 0.9999 cleared 1.10, and
/// the instrument was valid (N2 0.2353 < 0.5 x 0.5718 = 0.2859). Had
/// the arm missed a margin, the honest pin here would be FAIL — a
/// valid negative result, not a bug.
#[test]
fn one_head_beats_both_nulls_under_the_preregistered_margins() {
    let report = preregistered_report();
    assert_report_consistent(report);
    assert!(
        report.instrument_valid,
        "instrument measured valid at pin time"
    );

    let contract = &report.contract;
    assert_eq!(contract.nulls.n1, NULL_N1_ID);
    assert_eq!(contract.nulls.n2, NULL_N2_ID);
    assert_eq!(contract.replacement_semantics, REPLACEMENT_SEMANTICS_ID);
    // The pre-registered margins, as data in the report.
    assert_eq!(contract.gates.overlap_null_factor, 2.0);
    assert_eq!(contract.gates.overlap_floor, 0.5);
    assert_eq!(contract.gates.min_top1_agreement, 0.90);
    assert_eq!(contract.gates.max_bits_per_token_ratio, 1.10);
    assert_eq!(contract.gates.n2_vacuity_fraction, 0.5);

    let one_head = stage(report, STAGE_ONE_HEAD);
    assert_eq!(
        one_head.verdict,
        StageVerdict::Pass,
        "empirical pin (module docs)"
    );
    let overlap = one_head.overlap.as_ref().expect("overlap record");
    assert!(overlap.fitted >= 2.0 * overlap.n1, "fitted beats 2 x N1");
    assert!(overlap.fitted >= 2.0 * overlap.n2, "fitted beats 2 x N2");
    assert!(overlap.fitted >= 0.5, "fitted clears the absolute floor");
    assert!(overlap.n2 < 0.5 * overlap.fitted, "anti-vacuity margin");
    assert!(!overlap.vacuous);
    assert!(overlap.eligible_steps > 0);

    // Compilation/runtime success is recorded SEPARATELY from fit
    // success: the runtime checks passed on their own evidence.
    let runtime = one_head.runtime.as_ref().expect("runtime checks");
    assert!(runtime.pass);
    assert!(runtime.witness_replay_pass, "witness replay ran and passed");
    assert!(runtime.census_closed_form_pass);
    assert!(runtime.reference_crosscheck_pass);
    assert!(runtime.state_epoch_pass);
    assert!(runtime.steps > 0);
    assert!(runtime.allocation_note.contains("allocation_census"));
    let preflight = one_head.preflight.as_ref().expect("preflight");
    assert_eq!(preflight.status, ConformanceStatus::Pass);
    assert_eq!(preflight.checks.len(), 4);

    // The embedded Gate C parity rows (existing #307 metric type).
    let teacher_row = one_head.teacher.as_ref().expect("teacher row");
    let replaced_row = one_head.replaced_metrics.as_ref().expect("replaced row");
    assert_eq!(teacher_row.positions, fixture().corpus.records);
    assert_eq!(replaced_row.positions, fixture().corpus.records);
    assert!(replaced_row.top1_agreement >= contract.gates.min_top1_agreement);
    assert!(teacher_row.bits_per_token > 0.0);
    assert!(
        one_head.bits_per_token_ratio.expect("ratio") <= contract.gates.max_bits_per_token_ratio
    );
    assert!(one_head.top_k_agreement.expect("top-k row") >= replaced_row.top1_agreement);

    // Every synthetic stage's verdict at pin time (module docs). A
    // margin miss at a wider scope would legitimately flip these to
    // FAIL + NOT_RUN tails; re-pin only with re-measured numbers.
    for name in [
        STAGE_NULL,
        STAGE_ONE_HEAD,
        STAGE_ONE_LAYER,
        STAGE_LAYER_RANGE,
        STAGE_WHOLE_MODEL,
    ] {
        assert_eq!(
            stage(report, name).verdict,
            StageVerdict::Pass,
            "empirical pin for {name} (module docs)"
        );
    }
    // The null stage's identity check: an empty replacement scope
    // reproduces the teacher (top-1 agreement exactly 1.0).
    let null_stage = stage(report, STAGE_NULL);
    let null_replaced = null_stage.replaced_metrics.as_ref().expect("null row");
    assert_eq!(null_replaced.top1_agreement, 1.0);

    // A positive synthetic result is NOT a model-quality claim and does
    // NOT clear the dormant lane's activation gate: the decision record
    // says so in as many words.
    assert!(report
        .decision
        .outcome
        .contains("positive synthetic result"));
    assert!(report
        .decision
        .outcome
        .contains("activation gate remains uncleared"));
    assert!(report.decision.positive_next.contains("#531"));
    assert!(report.decision.negative_next.contains("route-fit/2"));
}

/// Test 3 (task list): the instrument can FAIL. A deliberately-broken
/// fit (every route code zeroed — all distances tie, the kernel selects
/// the lowest indices) must not pass: its selections carry no
/// query-specific information, so either the anti-vacuity rule fires
/// (N2 not below the fitted fraction) or the overlap gate fails — and
/// the kernel-level runtime checks still PASS, proving compilation
/// success is never conflated with fit success.
#[test]
fn broken_fit_produces_fail_verdicts() {
    let mut broken = fixture().fitted.clone();
    for head in &mut broken.heads {
        for story in &mut head.query_codes {
            for code in story {
                *code = [0u8; 36];
            }
        }
        for story in &mut head.key_codes {
            for code in story {
                *code = [0u8; 36];
            }
        }
    }
    let report = run_ladder(&preregistered_route_fit_contract(), &broken);
    assert_report_consistent(&report);
    let first_fail = report
        .stages
        .iter()
        .find(|record| record.verdict == StageVerdict::Fail)
        .expect("a broken fit must fail a stage");
    // Kernel-level evidence still passes on the broken fit — the FAIL
    // is a fit verdict, not a compilation verdict.
    let runtime = first_fail.runtime.as_ref().expect("runtime checks");
    assert!(runtime.pass, "runtime checks pass while the fit fails");
    // Nothing after the first failure ran.
    let fail_index = report
        .stages
        .iter()
        .position(|record| record.verdict == StageVerdict::Fail)
        .expect("index");
    for record in &report.stages[fail_index + 1..] {
        assert_eq!(record.verdict, StageVerdict::NotRun);
    }
    // No synthetic stage may PASS at or after the failure; and the
    // decision record marks the run as not-positive.
    assert!(!report
        .decision
        .outcome
        .contains("positive synthetic result"));
}

/// Test 4 (task list): stop-at-first-failure. A TEST-LOCAL contract
/// with an impossible overlap floor (1.01 — Jaccard cannot exceed 1)
/// makes the one-head stage the first failure; every later stage —
/// including the real-arm stages — must carry NOT_RUN, and the null
/// stage (which the overlap threshold gate does not bind) must still
/// PASS. The pre-registered margins are untouched by this test.
#[test]
fn ladder_stops_at_first_failure_with_not_run_tail() {
    let mut contract = preregistered_route_fit_contract();
    contract.gates.overlap_floor = 1.01;
    let report = run_ladder(&contract, &fixture().fitted);
    assert_report_consistent(&report);
    assert_eq!(stage(&report, STAGE_NULL).verdict, StageVerdict::Pass);
    let one_head = stage(&report, STAGE_ONE_HEAD);
    assert_eq!(one_head.verdict, StageVerdict::Fail);
    assert!(one_head.reason.contains("below the pre-registered bar"));
    for name in [
        STAGE_ONE_LAYER,
        STAGE_LAYER_RANGE,
        STAGE_WHOLE_MODEL,
        STAGE_REAL_TEACHER,
        STAGE_REAL_CORPUS,
    ] {
        let record = stage(&report, name);
        assert_eq!(record.verdict, StageVerdict::NotRun, "{name} is NOT_RUN");
        assert!(record.reason.contains("exited at stage one-head"));
    }
    assert!(report
        .decision
        .outcome
        .contains("negative synthetic result"));
    assert!(report.decision.outcome.contains("one-head"));
}

/// Test 5 (task list): the real-arm stages are present as UNAVAILABLE
/// with their prerequisites named (never a vacuous pass, never a silent
/// skip); the report round-trips serialization byte-for-byte; and the
/// three absence-adjacent states — FAIL, UNAVAILABLE, NOT_RUN — are
/// distinct on the wire.
#[test]
fn unavailable_stages_have_reasons_and_states_round_trip() {
    let report = preregistered_report();
    let real_teacher = stage(report, STAGE_REAL_TEACHER);
    assert_eq!(real_teacher.verdict, StageVerdict::Unavailable);
    assert!(
        real_teacher.reason.contains("SmolLM2"),
        "names the missing snapshot: {}",
        real_teacher.reason
    );
    let real_corpus = stage(report, STAGE_REAL_CORPUS);
    assert_eq!(real_corpus.verdict, StageVerdict::Unavailable);
    assert!(
        real_corpus.reason.contains("#531"),
        "names the missing corpus: {}",
        real_corpus.reason
    );

    // Round-trip: canonical bytes -> typed -> canonical bytes.
    let bytes = canonical_route_fit_report_bytes(report);
    let back: RouteFitReport = ciborium::from_reader(bytes.as_slice()).expect("deserializes");
    assert_eq!(canonical_route_fit_report_bytes(&back), bytes);
    assert_eq!(&back, report);
    assert_eq!(
        route_fit_report_kappa(&back),
        route_fit_report_kappa(report)
    );

    // Absent-as-absent on the wire: the four verdicts serialize to four
    // distinct tokens, so NOT_RUN can never be read as UNAVAILABLE or
    // FAIL after a round trip.
    let tokens: Vec<String> = [
        StageVerdict::Pass,
        StageVerdict::Fail,
        StageVerdict::Unavailable,
        StageVerdict::NotRun,
    ]
    .iter()
    .map(|verdict| {
        let mut bytes = Vec::new();
        ciborium::into_writer(verdict, &mut bytes).expect("serializes");
        format!("{bytes:?}")
    })
    .collect();
    for (i, a) in tokens.iter().enumerate() {
        for b in tokens.iter().skip(i + 1) {
            assert_ne!(a, b, "verdict wire tokens must be distinct");
        }
    }
    let round: StageVerdict = {
        let mut bytes = Vec::new();
        ciborium::into_writer(&StageVerdict::NotRun, &mut bytes).expect("serializes");
        ciborium::from_reader(bytes.as_slice()).expect("deserializes")
    };
    assert_eq!(round, StageVerdict::NotRun);
    assert_ne!(round, StageVerdict::Unavailable);
    assert_ne!(round, StageVerdict::Fail);
}
