//! #606 target-operator certificate tests: deterministic double-run of
//! the assembly, replay (serialize → parse → re-serialize) equality,
//! the pinned end-to-end certificate over the #605 synthetic-ladder
//! outputs (positive families + UNAVAILABLE real arms + NON-PASSING
//! overall quality), a failing-ladder composition (FAIL + BLOCKED
//! rows), the schema-registry refusal, the quality-derivation refusal
//! under every absence state, and tampered-κ / tampered-verdict
//! detection.
//!
//! The #605 fixture (synthetic teacher + mini-corpus + fit + ladder)
//! is built ONCE per process through the production #603/#605 surfaces
//! and shared read-only. Everything measured is deterministic
//! (integer-seeded, no clock); the only nondeterminism in this file is
//! temp-directory naming, which never reaches a measured byte.

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use uor_r4_graph_certify::route_fit_report::{
    preregistered_route_fit_contract, route_fit_report_kappa, run_route_fit_ladder, RealArmProbe,
    RouteFitReport, RunContract, RuntimeChecks, StageVerdict, STAGE_REAL_CORPUS,
    STAGE_REAL_TEACHER,
};
use uor_r4_graph_certify::target_operator_certificate::{
    assemble_target_operator_certificate, canonical_target_operator_certificate_bytes,
    certificate_spec, derive_overall_quality, route_attention_obligation_links,
    target_operator_certificate_kappa, verify_certificate_sources,
    verify_target_operator_certificate, CertificateIdentity, FamilyVerdict, OverallQuality,
    ProvenanceRow, RuntimeBoundsRow, ScopeRow, TargetOperatorCertificate,
    TargetOperatorCertificateSpec, CERTIFICATE_SCOPES, QUALITY_BEARING_SCOPES,
    QUALITY_STATE_NOT_PASSING, QUALITY_STATE_PASSING, SCOPE_HEAD, SCOPE_LAYER, SCOPE_MODEL,
    SCOPE_REAL_CORPUS, SCOPE_REAL_TEACHER, SURFACE_FITTED_PARAMS, SURFACE_FIT_MANIFEST,
    SURFACE_FIT_REPORT, TARGET_OPERATOR_CERTIFICATE_ID, TARGET_OPERATOR_CERTIFICATE_SCHEMA,
    TARGET_OPERATOR_CERTIFICATE_VERSION,
};
use uor_r4_graph_compiler::route_fit::{
    fit_route_codes, generate_synthetic_route_trace, load_route_trace_corpus,
    synthetic_capture_geometry, synthetic_fit_manifest, FitManifest, FittedRouteCodes,
    RouteTraceCorpus, SyntheticRouteTeacher,
};
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
        let dir = unique_path("target-operator-certificate-fixture");
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

fn run_ladder(contract: &RunContract) -> RouteFitReport {
    let fixture = fixture();
    let mut teacher = SyntheticRouteTeacher::new();
    run_route_fit_ladder(
        &mut teacher,
        &fixture.corpus,
        &fixture.fitted,
        &fixture.manifest,
        contract,
        &RealArmProbe::from_env(),
    )
    .expect("ladder runs")
}

/// The pre-registered-contract fit report, run once and shared.
fn preregistered_report() -> &'static RouteFitReport {
    static REPORT: OnceLock<RouteFitReport> = OnceLock::new();
    REPORT.get_or_init(|| run_ladder(&preregistered_route_fit_contract()))
}

fn scope_row<'a>(certificate: &'a TargetOperatorCertificate, scope: &str) -> &'a ScopeRow {
    certificate
        .scopes
        .iter()
        .find(|row| row.scope == scope)
        .unwrap_or_else(|| panic!("scope {scope} present"))
}

fn runtime_row<'a>(
    certificate: &'a TargetOperatorCertificate,
    scope: &str,
) -> &'a RuntimeBoundsRow {
    certificate
        .runtime_bounds
        .iter()
        .find(|row| row.scope == scope)
        .unwrap_or_else(|| panic!("runtime scope {scope} present"))
}

/// Test 1: two assemblies from identical inputs — with the inputs
/// themselves rebuilt by a second ladder run over the same fitted
/// artifact — produce byte-identical canonical certificates and equal
/// κs; and the certificate replays (serialize → parse → re-serialize)
/// byte-for-byte.
#[test]
fn certificate_double_run_is_byte_identical_and_replays() {
    let fixture = fixture();
    let first = assemble_target_operator_certificate(&fixture.manifest, preregistered_report())
        .expect("certificate assembles");
    let second_report = run_ladder(&preregistered_route_fit_contract());
    let second = assemble_target_operator_certificate(&fixture.manifest, &second_report)
        .expect("certificate assembles again");
    assert_eq!(
        canonical_target_operator_certificate_bytes(&first),
        canonical_target_operator_certificate_bytes(&second),
        "certificate assembly must be deterministic over identical inputs"
    );
    assert_eq!(
        target_operator_certificate_kappa(&first),
        target_operator_certificate_kappa(&second)
    );
    assert_eq!(first.schema, TARGET_OPERATOR_CERTIFICATE_SCHEMA);
    assert_eq!(
        first.spec_digest,
        TargetOperatorCertificateSpec::v1().declared_digest
    );

    // Replay equality: canonical bytes -> typed -> canonical bytes.
    let bytes = canonical_target_operator_certificate_bytes(&first);
    let back: TargetOperatorCertificate =
        ciborium::from_reader(bytes.as_slice()).expect("deserializes");
    assert_eq!(canonical_target_operator_certificate_bytes(&back), bytes);
    assert_eq!(back, first);
    assert_eq!(
        target_operator_certificate_kappa(&back),
        target_operator_certificate_kappa(&first)
    );
    // The parsed certificate still verifies against its sources.
    verify_certificate_sources(&back, &fixture.manifest, preregistered_report())
        .expect("replayed certificate verifies");
}

/// Test 2: the pinned end-to-end certificate over the #605
/// synthetic-ladder outputs. The synthetic scope rows report their
/// actual measured verdicts (source-parity / target-fit /
/// runtime-contract / witness-replay PASS at pin time — the #605
/// empirical pins), model-quality is UNAVAILABLE with the #605
/// real-arm reason strings verbatim, the real-teacher / real-corpus
/// rows are UNAVAILABLE with their prerequisites named, and the
/// overall quality verdict is NON-PASSING: a compiled artifact never
/// reads as a quality success.
#[test]
fn end_to_end_certificate_over_605_ladder_is_non_passing() {
    let fixture = fixture();
    let report = preregistered_report();
    let certificate = assemble_target_operator_certificate(&fixture.manifest, report)
        .expect("certificate assembles");

    // Identity block: every identity the synthetic arm has, present;
    // the tokenizer genuinely absent (typed None, never an empty
    // string); the three composed κs equal to what the sources
    // compute.
    let identity = &certificate.identity;
    assert!(identity.source_snapshot.is_some());
    assert!(identity.tokenizer.is_none(), "typed absence, not a value");
    assert_eq!(
        identity.adapter.as_deref(),
        Some("synthetic-route-teacher/1")
    );
    assert!(identity.trace.is_some());
    assert!(identity.geometry.is_some());
    assert_eq!(identity.operator_id.as_deref(), Some("r4-route-attention"));
    assert_eq!(identity.operator_version, Some(1));
    assert!(identity.operator.is_some());
    assert!(identity.corpus.is_some());
    assert!(identity.compiler.is_some());
    assert_eq!(
        identity.fit_manifest.as_deref(),
        Some(fixture.manifest.kappa().as_str())
    );
    assert_eq!(
        identity.fit_report.as_deref(),
        Some(route_fit_report_kappa(report).as_str())
    );
    assert_eq!(
        identity.fitted_params.as_deref(),
        Some(fixture.fitted.kappa().as_str())
    );
    assert!(certificate.instrument_valid, "measured valid at pin time");

    // The synthetic scope rows: measured family verdicts (empirical
    // pin, #605 module docs), model-quality UNAVAILABLE with the #605
    // real-teacher reason VERBATIM.
    let real_teacher_stage = report
        .stages
        .iter()
        .find(|stage| stage.stage == STAGE_REAL_TEACHER)
        .expect("real-teacher stage");
    for scope in [SCOPE_HEAD, SCOPE_LAYER, "layer-range", SCOPE_MODEL] {
        let row = scope_row(&certificate, scope);
        assert_eq!(row.source_parity, FamilyVerdict::Pass, "{scope} parity");
        assert_eq!(row.target_fit, FamilyVerdict::Pass, "{scope} fit");
        assert_eq!(row.runtime_contract, FamilyVerdict::Pass, "{scope} runtime");
        assert_eq!(row.witness_replay, FamilyVerdict::Pass, "{scope} witness");
        assert_eq!(
            row.model_quality,
            FamilyVerdict::Unavailable(real_teacher_stage.reason.clone()),
            "{scope} quality is UNAVAILABLE with the #605 reason verbatim"
        );
        // The embedded #307 Gate C parity rows travel with the row.
        let teacher = row.teacher.as_ref().expect("teacher parity row");
        let replaced = row.replaced_metrics.as_ref().expect("replaced parity row");
        assert_eq!(teacher.positions, fixture.corpus.records);
        assert_eq!(replaced.positions, fixture.corpus.records);
        assert!(row.top_k_agreement.is_some());
        assert!(row.bits_per_token_ratio.is_some());
    }

    // The real-arm rows: every family UNAVAILABLE with the #605 reason
    // strings (never a vacuous pass, never a silent skip).
    let real_teacher = scope_row(&certificate, SCOPE_REAL_TEACHER);
    for verdict in [
        &real_teacher.source_parity,
        &real_teacher.target_fit,
        &real_teacher.runtime_contract,
        &real_teacher.witness_replay,
        &real_teacher.model_quality,
    ] {
        match verdict {
            FamilyVerdict::Unavailable(reason) => {
                assert!(reason.contains("SmolLM2"), "names the snapshot: {reason}");
                assert_eq!(reason, &real_teacher_stage.reason);
            }
            other => panic!("real-teacher family is {other:?}, expected UNAVAILABLE"),
        }
    }
    let real_corpus = scope_row(&certificate, SCOPE_REAL_CORPUS);
    let real_corpus_stage = report
        .stages
        .iter()
        .find(|stage| stage.stage == STAGE_REAL_CORPUS)
        .expect("real-corpus stage");
    assert_eq!(real_corpus_stage.verdict, StageVerdict::Unavailable);
    match &real_corpus.model_quality {
        FamilyVerdict::Unavailable(reason) => {
            assert!(reason.contains("#531"), "names the corpus: {reason}");
            assert_eq!(reason, &real_corpus_stage.reason);
        }
        other => panic!("real-corpus quality is {other:?}, expected UNAVAILABLE"),
    }

    // Runtime-bounds rows: the declared format-crate bounds plus the
    // embedded #605 checks; the allocation claim stays with the census.
    for scope in [SCOPE_HEAD, SCOPE_LAYER, "layer-range", SCOPE_MODEL] {
        let row = runtime_row(&certificate, scope);
        assert_eq!(row.max_candidates, 64);
        assert_eq!(row.max_top_m, 8);
        assert!(row.census_note.contains("uor-r4-graph-format"));
        let checks = row.checks.as_ref().expect("embedded runtime checks");
        assert!(checks.pass);
        assert!(checks.steps > 0);
        assert!(checks.allocation_note.contains("allocation_census"));
    }
    for scope in [SCOPE_REAL_TEACHER, SCOPE_REAL_CORPUS] {
        assert!(
            runtime_row(&certificate, scope).checks.is_none(),
            "no runtime evidence exists for the {scope} arm"
        );
    }

    // Provenance rows reference the composed surfaces by their own κs.
    let by_surface = |surface: &str| -> &ProvenanceRow {
        certificate
            .provenance
            .iter()
            .find(|row| row.surface == surface)
            .unwrap_or_else(|| panic!("provenance surface {surface}"))
    };
    assert_eq!(
        by_surface(SURFACE_FIT_REPORT).kappa.as_deref(),
        Some(route_fit_report_kappa(report).as_str())
    );
    assert_eq!(
        by_surface(SURFACE_FIT_MANIFEST).kappa.as_deref(),
        Some(fixture.manifest.kappa().as_str())
    );
    assert_eq!(
        by_surface(SURFACE_FITTED_PARAMS).kappa.as_deref(),
        Some(fixture.fitted.kappa().as_str())
    );
    // Surfaces with no κ of their own are referenced by typed identity
    // and carry a typed absence — never an invented digest.
    for surface in [
        "gate-c-parity",
        "teacher-parity-harness",
        "proof-obligations",
    ] {
        let row = by_surface(surface);
        assert!(row.kappa.is_none(), "{surface} has no κ to reference");
        assert!(!row.type_identity.is_empty());
    }
    assert_eq!(certificate.obligations, route_attention_obligation_links());

    // The overall quality verdict is NON-PASSING (real arms
    // unavailable; the tokenizer identity of a real teacher is absent)
    // while the fit/runtime/witness families above report their
    // measured PASS verdicts — compilation success is not quality
    // success, structurally.
    assert_eq!(certificate.overall_quality_state, QUALITY_STATE_NOT_PASSING);
    assert!(
        certificate.overall_quality_reason.contains("tokenizer"),
        "the first refusal in the fixed order is the absent real-teacher tokenizer \
         identity: {}",
        certificate.overall_quality_reason
    );
    verify_target_operator_certificate(&certificate).expect("stored verdict is the derived one");
    verify_certificate_sources(&certificate, &fixture.manifest, report).expect("sources verify");

    // With the tokenizer hypothetically present, the refusal moves to
    // the real arms themselves: the quality claim stays impossible
    // while the real-teacher / real-corpus rows are UNAVAILABLE.
    let mut with_tokenizer = certificate.clone();
    with_tokenizer.identity.tokenizer = Some("hypothetical-tokenizer".to_owned());
    match derive_overall_quality(&with_tokenizer) {
        OverallQuality::NotPassing { reason } => {
            assert!(
                reason.contains(SCOPE_REAL_TEACHER),
                "the refusal names the unavailable real arm: {reason}"
            );
        }
        OverallQuality::Passing(_) => {
            panic!("a certificate with unavailable real arms may never derive PASSING")
        }
    }
}

/// Test 3: a failing ladder composes into FAIL and BLOCKED rows. Under
/// a test-local contract with an impossible overlap floor the one-head
/// stage fails, so the head row's target-fit family is FAIL while its
/// runtime-contract and witness-replay families stay PASS (compilation
/// success is not fit success), and every later scope is BLOCKED with
/// the #605 exit-rule reason — absence states, never vacuous passes.
#[test]
fn failing_ladder_composes_fail_and_blocked_rows() {
    let fixture = fixture();
    let mut contract = preregistered_route_fit_contract();
    contract.gates.overlap_floor = 1.01;
    let report = run_ladder(&contract);
    let certificate = assemble_target_operator_certificate(&fixture.manifest, &report)
        .expect("certificate assembles");

    let head = scope_row(&certificate, SCOPE_HEAD);
    assert_eq!(head.target_fit, FamilyVerdict::Fail);
    assert_eq!(head.runtime_contract, FamilyVerdict::Pass);
    assert_eq!(head.witness_replay, FamilyVerdict::Pass);
    assert_eq!(head.source_parity, FamilyVerdict::Pass);
    assert!(head.note.contains("below the pre-registered bar"));

    for scope in [SCOPE_LAYER, "layer-range", SCOPE_MODEL] {
        let row = scope_row(&certificate, scope);
        for (family, verdict) in [
            ("source_parity", &row.source_parity),
            ("target_fit", &row.target_fit),
            ("runtime_contract", &row.runtime_contract),
            ("witness_replay", &row.witness_replay),
        ] {
            match verdict {
                FamilyVerdict::Blocked(reason) => {
                    assert!(
                        reason.contains("exited at stage one-head"),
                        "{scope}/{family} names the exit: {reason}"
                    );
                }
                other => panic!("{scope}/{family} is {other:?}, expected BLOCKED"),
            }
        }
        assert!(row.model_quality.is_absent());
    }
    // The real arms behind the exit are BLOCKED too (distinct from the
    // preregistered run's UNAVAILABLE).
    for scope in QUALITY_BEARING_SCOPES {
        let row = scope_row(&certificate, scope);
        assert!(
            matches!(row.model_quality, FamilyVerdict::Blocked(_)),
            "{scope} is blocked by the exit rule"
        );
    }
    assert_eq!(certificate.overall_quality_state, QUALITY_STATE_NOT_PASSING);
    verify_target_operator_certificate(&certificate).expect("stored verdict is derived");
}

/// Test 4a: the schema registry refuses every unknown `(id, version)`
/// by name — never guesses, never resolves a "closest" schema.
#[test]
fn registry_refuses_unknown_schema_by_name() {
    let known = certificate_spec(
        TARGET_OPERATOR_CERTIFICATE_ID,
        TARGET_OPERATOR_CERTIFICATE_VERSION,
    )
    .expect("registered schema");
    assert_eq!(known, TargetOperatorCertificateSpec::v1());
    for (id, version) in [
        (TARGET_OPERATOR_CERTIFICATE_ID, 2u32),
        (TARGET_OPERATOR_CERTIFICATE_ID, 0),
        ("route-fit", 1),
        ("target-operator-certificate-2", 7),
    ] {
        let error =
            certificate_spec(id, version).expect_err("unknown (id, version) is not a product");
        assert!(error.reason.contains(id), "reason names the id: {error}");
        assert!(
            error.reason.contains(&version.to_string()),
            "reason names the version: {error}"
        );
    }
}

/// A synthetic certificate every quality prerequisite of which holds:
/// the fixture the refusal test perturbs. Hand-built — the derivation
/// is a pure function of the record, so the rule's reachability and
/// its refusals are both testable without a real-teacher run.
fn passing_fixture() -> TargetOperatorCertificate {
    let kappa = |tag: &str| format!("blake3:{}", blake3::hash(tag.as_bytes()).to_hex());
    let checks = RuntimeChecks {
        steps: 8,
        witness_replay_pass: true,
        witness_replay_detail: String::new(),
        census_closed_form_pass: true,
        reference_crosscheck_pass: true,
        state_epoch_pass: true,
        allocation_note: "owned by the repository allocation census".to_owned(),
        pass: true,
        cost: None,
    };
    let scopes = CERTIFICATE_SCOPES
        .iter()
        .map(|&scope| {
            let quality_bearing = QUALITY_BEARING_SCOPES.contains(&scope);
            ScopeRow {
                scope: scope.to_owned(),
                stage: scope.to_owned(),
                source_parity: FamilyVerdict::Pass,
                target_fit: FamilyVerdict::Pass,
                runtime_contract: FamilyVerdict::Pass,
                witness_replay: FamilyVerdict::Pass,
                model_quality: if quality_bearing {
                    FamilyVerdict::Pass
                } else {
                    FamilyVerdict::Unavailable(
                        "model-quality binds to the real-teacher and real-corpus rows".to_owned(),
                    )
                },
                note: "synthetic all-pass fixture".to_owned(),
                ..ScopeRow::default()
            }
        })
        .collect();
    let runtime_bounds = CERTIFICATE_SCOPES
        .iter()
        .map(|&scope| RuntimeBoundsRow {
            scope: scope.to_owned(),
            max_candidates: 64,
            max_top_m: 8,
            census_note: "closed forms owned by uor-r4-graph-format".to_owned(),
            checks: Some(checks.clone()),
        })
        .collect();
    let provenance = vec![
        ProvenanceRow {
            surface: SURFACE_FIT_REPORT.to_owned(),
            type_identity: "RouteFitReport".to_owned(),
            kappa: Some(kappa("report")),
            note: String::new(),
        },
        ProvenanceRow {
            surface: SURFACE_FIT_MANIFEST.to_owned(),
            type_identity: "FitManifest".to_owned(),
            kappa: Some(kappa("manifest")),
            note: String::new(),
        },
        ProvenanceRow {
            surface: SURFACE_FITTED_PARAMS.to_owned(),
            type_identity: "FittedRouteCodes".to_owned(),
            kappa: Some(kappa("params")),
            note: String::new(),
        },
    ];
    let mut certificate = TargetOperatorCertificate {
        schema: TARGET_OPERATOR_CERTIFICATE_SCHEMA.to_owned(),
        spec_digest: TargetOperatorCertificateSpec::v1().declared_digest,
        identity: CertificateIdentity {
            source_snapshot: Some(kappa("snapshot")),
            tokenizer: Some("tokenizer/1".to_owned()),
            adapter: Some("adapter/1".to_owned()),
            trace: Some(kappa("trace")),
            geometry: Some(kappa("geometry")),
            operator_id: Some("r4-route-attention".to_owned()),
            operator_version: Some(1),
            operator: Some(kappa("operator")),
            corpus: Some(kappa("corpus")),
            compiler: Some("uor-r4-graph-compiler/0.1.0".to_owned()),
            fit_manifest: Some(kappa("manifest")),
            fit_report: Some(kappa("report")),
            fitted_params: Some(kappa("params")),
        },
        instrument_valid: true,
        instrument_note: "synthetic all-pass fixture".to_owned(),
        scopes,
        runtime_bounds,
        provenance,
        obligations: route_attention_obligation_links(),
        overall_quality_state: String::new(),
        overall_quality_reason: String::new(),
    };
    let quality = derive_overall_quality(&certificate);
    certificate.overall_quality_state = quality.state_token().to_owned();
    certificate.overall_quality_reason = quality.reason_line();
    certificate
}

fn assert_not_passing(certificate: &TargetOperatorCertificate, what: &str) {
    match derive_overall_quality(certificate) {
        OverallQuality::NotPassing { .. } => {}
        OverallQuality::Passing(_) => panic!("{what} must refuse a passing quality claim"),
    }
}

/// Test 4b: the quality derivation is reachable (the all-pass fixture
/// derives PASSING) and refuses every absence: any family of any
/// required row set to NOT_MEASURED / BLOCKED / UNAVAILABLE (or FAIL)
/// makes a passing claim impossible, as do a missing row, a missing
/// identity, an invalid instrument, unchecked runtime rows, an
/// unverified obligation link, and a measured model-quality verdict on
/// a synthetic scope.
#[test]
fn quality_cannot_pass_with_missing_blocked_or_unavailable_prerequisites() {
    let passing = passing_fixture();
    assert!(
        matches!(derive_overall_quality(&passing), OverallQuality::Passing(_)),
        "the rule must be reachable, else the refusals below are vacuous: {}",
        passing.overall_quality_reason
    );
    assert_eq!(passing.overall_quality_state, QUALITY_STATE_PASSING);
    verify_target_operator_certificate(&passing).expect("stored verdict is derived");

    let absences = || {
        [
            FamilyVerdict::NotMeasured,
            FamilyVerdict::Blocked("blocked by test".to_owned()),
            FamilyVerdict::Unavailable("unavailable by test".to_owned()),
            FamilyVerdict::Fail,
        ]
    };
    // Every non-quality family on every row, plus model-quality on the
    // quality-bearing rows: each absence state refuses the claim.
    for index in 0..passing.scopes.len() {
        let quality_bearing =
            QUALITY_BEARING_SCOPES.contains(&passing.scopes[index].scope.as_str());
        for verdict in absences() {
            for family in 0..5usize {
                if family == 4 && !quality_bearing {
                    continue; // handled below: synthetic quality must stay absent
                }
                let mut tampered = passing.clone();
                let row = &mut tampered.scopes[index];
                match family {
                    0 => row.source_parity = verdict.clone(),
                    1 => row.target_fit = verdict.clone(),
                    2 => row.runtime_contract = verdict.clone(),
                    3 => row.witness_replay = verdict.clone(),
                    _ => row.model_quality = verdict.clone(),
                }
                assert_not_passing(
                    &tampered,
                    &format!(
                        "scope {} family {family} as {verdict:?}",
                        tampered.scopes[index].scope
                    ),
                );
            }
        }
    }
    // A synthetic scope carrying a MEASURED model-quality verdict is an
    // inconsistency: a compiled artifact must never read as a quality
    // success, even by tamper.
    for measured in [FamilyVerdict::Pass, FamilyVerdict::Fail] {
        let mut tampered = passing.clone();
        for row in &mut tampered.scopes {
            if !QUALITY_BEARING_SCOPES.contains(&row.scope.as_str()) {
                row.model_quality = measured.clone();
            }
        }
        assert_not_passing(&tampered, "a measured synthetic model-quality verdict");
    }
    // A missing required row refuses; so does a duplicate.
    for index in 0..CERTIFICATE_SCOPES.len() {
        let mut tampered = passing.clone();
        tampered.scopes.remove(index);
        assert_not_passing(&tampered, "a missing scope row");
        let mut duplicated = passing.clone();
        let row = duplicated.scopes[index].clone();
        duplicated.scopes.push(row);
        assert_not_passing(&duplicated, "a duplicated scope row");
    }
    // Every identity, removed one at a time, refuses.
    for field in 0..13usize {
        let mut tampered = passing.clone();
        {
            let identity = &mut tampered.identity;
            match field {
                0 => identity.source_snapshot = None,
                1 => identity.tokenizer = None,
                2 => identity.adapter = None,
                3 => identity.trace = None,
                4 => identity.geometry = None,
                5 => identity.operator_id = None,
                6 => identity.operator_version = None,
                7 => identity.operator = None,
                8 => identity.corpus = None,
                9 => identity.compiler = None,
                10 => identity.fit_manifest = None,
                11 => identity.fit_report = None,
                _ => identity.fitted_params = None,
            }
        }
        assert_not_passing(&tampered, "an absent identity");
    }
    // A claimed κ that is not one refuses.
    let mut bad_kappa = passing.clone();
    bad_kappa.identity.fit_report = Some("not-a-kappa".to_owned());
    assert_not_passing(&bad_kappa, "a non-κ fit-report identity");
    // An invalid instrument refuses.
    let mut vacuous = passing.clone();
    vacuous.instrument_valid = false;
    assert_not_passing(&vacuous, "an invalid instrument");
    // Unchecked or failed runtime rows refuse.
    let mut unchecked = passing.clone();
    unchecked.runtime_bounds[0].checks = None;
    assert_not_passing(&unchecked, "a runtime row with no checks");
    let mut failed_checks = passing.clone();
    if let Some(checks) = failed_checks.runtime_bounds[0].checks.as_mut() {
        checks.pass = false;
    }
    assert_not_passing(&failed_checks, "failed runtime checks");
    let mut zero_steps = passing.clone();
    if let Some(checks) = zero_steps.runtime_bounds[0].checks.as_mut() {
        checks.steps = 0;
    }
    assert_not_passing(&zero_steps, "zero measured steps");
    let mut wrong_bounds = passing.clone();
    wrong_bounds.runtime_bounds[0].max_candidates = 65;
    assert_not_passing(&wrong_bounds, "bounds other than the declared ones");
    // A provenance κ that disagrees with the identity block refuses.
    let mut skewed = passing.clone();
    skewed.provenance[0].kappa = Some(format!(
        "blake3:{}",
        blake3::hash(b"someone-elses-report").to_hex()
    ));
    assert_not_passing(
        &skewed,
        "a provenance κ disagreeing with the identity block",
    );
    // An obligation recorded Unverified refuses; so does an empty link
    // list.
    let mut unverified = passing.clone();
    unverified.obligations[0].status = "Unverified".to_owned();
    assert_not_passing(&unverified, "an Unverified obligation link");
    let mut unlinked = passing.clone();
    unlinked.obligations.clear();
    assert_not_passing(&unlinked, "no linked obligations");
    // A foreign schema or spec digest refuses.
    let mut foreign = passing.clone();
    foreign.spec_digest = "blake3:0000".to_owned();
    assert_not_passing(&foreign, "a foreign spec digest");
}

/// Test 4c: tamper detection. A certificate whose embedded κ reference
/// was edited is refused against its sources; a certificate whose
/// stored overall verdict was edited to PASSING is refused by
/// verification; a report/manifest pair that disagrees is refused at
/// assembly. Each refusal is the sanctioned typed failure.
#[test]
fn tampered_kappa_or_verdict_is_detected() {
    let fixture = fixture();
    let report = preregistered_report();
    let certificate =
        assemble_target_operator_certificate(&fixture.manifest, report).expect("assembles");

    // Tampered embedded κ reference.
    let mut tampered = certificate.clone();
    tampered.identity.fit_report = Some(format!(
        "blake3:{}",
        blake3::hash(b"tampered-report-kappa").to_hex()
    ));
    let error = verify_certificate_sources(&tampered, &fixture.manifest, report)
        .expect_err("a tampered κ reference is not a valid certificate");
    assert!(
        error.reason.contains("fit_report"),
        "names the tampered field: {error}"
    );
    assert!(
        error.reason.contains(&route_fit_report_kappa(report)),
        "names the recomputed κ: {error}"
    );

    // Tampered stored verdict: editing NOT_PASSING to PASSING cannot
    // survive verification, because the derivation is recomputed from
    // the rows.
    let mut forged = certificate.clone();
    forged.overall_quality_state = QUALITY_STATE_PASSING.to_owned();
    let error = verify_target_operator_certificate(&forged)
        .expect_err("a forged PASSING state is not a valid certificate");
    assert!(
        error.reason.contains("disagrees with the derivation"),
        "names the mismatch: {error}"
    );
    // ... and the same forgery embedded in serialized bytes is refused
    // after a round trip, so a forged document cannot parse-and-verify.
    let bytes = canonical_target_operator_certificate_bytes(&forged);
    let parsed: TargetOperatorCertificate =
        ciborium::from_reader(bytes.as_slice()).expect("deserializes");
    verify_target_operator_certificate(&parsed)
        .expect_err("a forged document does not verify after replay");

    // A mismatched manifest/report pair is refused at assembly.
    let mut foreign_manifest = fixture.manifest.clone();
    foreign_manifest.adapter = Some("some-other-adapter/1".to_owned());
    let error = assemble_target_operator_certificate(&foreign_manifest, report)
        .expect_err("a mismatched input pair is not certifiable");
    assert!(
        error.reason.contains("mismatch"),
        "names the mismatch: {error}"
    );
}

/// Test 5: the five family-verdict states are distinct on the wire and
/// round-trip; the defaulted family is NOT_MEASURED (absence, never a
/// vacuous pass). Positive, negative, absent, and blocked rows all
/// serialize losslessly.
#[test]
fn family_verdict_states_are_distinct_and_round_trip() {
    let states = [
        FamilyVerdict::Pass,
        FamilyVerdict::Fail,
        FamilyVerdict::NotMeasured,
        FamilyVerdict::Blocked("prerequisite blocked".to_owned()),
        FamilyVerdict::Unavailable("prerequisite absent".to_owned()),
    ];
    let tokens: Vec<String> = states
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
    for state in &states {
        let mut bytes = Vec::new();
        ciborium::into_writer(state, &mut bytes).expect("serializes");
        let back: FamilyVerdict = ciborium::from_reader(bytes.as_slice()).expect("deserializes");
        assert_eq!(&back, state);
    }
    assert_eq!(FamilyVerdict::default(), FamilyVerdict::NotMeasured);
    assert!(FamilyVerdict::default().is_absent());
    // A zeroed/absent state never equals a measured one: Blocked and
    // Unavailable carry their reasons and stay distinct from each
    // other and from NotMeasured even with equal reason text.
    assert_ne!(
        FamilyVerdict::Blocked("x".to_owned()),
        FamilyVerdict::Unavailable("x".to_owned())
    );
}
