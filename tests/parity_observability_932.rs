#[path = "support/parity_observability.rs"]
mod parity_observability;

use parity_observability::{
    adaptive_decode_checkpoints, adaptive_decode_decision,
    apply_teacher_free_preflight_failure_metadata, classify_exact_probe_artifact,
    configured_exact_probe_report_path, configured_fixture_dir, configured_preflight_report_path,
    deterministic_evidence_identities, deterministic_teacher_execution, estimate_eta,
    events_path_for_report, heartbeat_progress_units, mark_finalization_failed,
    prepare_final_reports, publish_atomic_preflight_outcome, sample_host_resources,
    seconds_per_forward_from_rate, take_run_report_ownership, validate_binding_host_shape,
    validate_full_width_exact_heartbeat, validate_in_flight_heartbeat_cadence,
    validate_nonqualified_probe_prepublication, validate_private_multistream_evidence,
    write_atomic_json, write_final_reports, AdaptiveDecodeDecision, CancellableStartGate,
    ConfigError, DeterministicEvidence, EtaInput, EtaStatus, EventKind, ExactProbeArtifact,
    ExactProgressObservation, FixtureStatus, FixtureVerdict, HeartbeatEvent, HeartbeatLog,
    HeartbeatWorker, Measurement, ObservabilityMode, ParityConfig, PlanInstallError, ProgressUnit,
    QueueSnapshot, ResourceAvailability, RunMetadata, RunReport, RunStatus, SchedulerSnapshot,
    SharedProgress, StreamProgress, StreamState, TeacherFreeGraphFailureStage, WorkCounters,
    WorkPlan,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uor_r4_model_source::TeacherExecutionSnapshot;

fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test value is nonzero")
}

fn assert_measured_or_reason<T>(measurement: &Measurement<T>) {
    if let Measurement::Unavailable { reason } = measurement {
        assert!(!reason.trim().is_empty());
    }
}

fn unique_dir(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "uor-r4-parity-observability-932-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn lookup(values: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<String> {
    move |name| values.get(name).map(|value| (*value).to_owned())
}

#[test]
fn exact_probe_state_discriminator_projects_truthful_nonqualification_and_rejects_unknown_shapes() {
    const SCHEMA: &str = "uor-r4.exact-multicore-probe/2";
    for (event, status, expected_run, expected_fixture, expected_prefix) in [
        (
            "NOT_RUN",
            "REFUSE_FULL_RUN",
            RunStatus::NotRun,
            FixtureVerdict::NotRun,
            "NOT_RUN / REFUSED:",
        ),
        (
            "UNAVAILABLE",
            "UNAVAILABLE",
            RunStatus::Unavailable,
            FixtureVerdict::Unavailable,
            "UNAVAILABLE:",
        ),
        (
            "FAIL",
            "FAIL",
            RunStatus::Fail,
            FixtureVerdict::Failed,
            "FAILED:",
        ),
        (
            "ABORTED",
            "ABORTED",
            RunStatus::Aborted,
            FixtureVerdict::NotRun,
            "ABORTED:",
        ),
    ] {
        let bytes = serde_json::to_vec(&json!({
            "schema": SCHEMA,
            "record": "EXACT_MULTICORE_PROBE_STATE",
            "event": event,
            "status": status,
            "qualifies_full_run": false,
            "reason": "planted direct-tuner preflight refusal",
        }))
        .expect("state JSON");
        let ExactProbeArtifact::NonQualified(state) =
            classify_exact_probe_artifact(&bytes, SCHEMA).expect("recognized non-qualified state")
        else {
            panic!("state artifact must not be treated as a qualified report")
        };
        assert_eq!(state.run_status, expected_run);
        assert!(state.outcome_reason().starts_with(expected_prefix));
        let fixture = state.fixture_status("blake3:state-artifact");
        assert_eq!(fixture.verdict, expected_fixture);
        assert_eq!(fixture.cid.as_deref(), Some("blake3:state-artifact"));
        assert!(fixture
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("planted")));
    }

    let running = serde_json::to_vec(&json!({
        "schema": SCHEMA,
        "record": "EXACT_MULTICORE_PROBE_STATE",
        "event": "RUNNING",
        "status": "NOT_QUALIFIED",
        "qualifies_full_run": false,
    }))
    .expect("running state JSON");
    let ExactProbeArtifact::NonQualified(running) =
        classify_exact_probe_artifact(&running, SCHEMA).expect("running state is non-qualified")
    else {
        panic!("running state must not be treated as a qualified report")
    };
    assert_eq!(running.run_status, RunStatus::NotRun);
    assert!(running.reason.contains("still running"));

    let candidate = serde_json::to_vec(&json!({
        "schema": SCHEMA,
        "executor_contract_cid": "blake3:typed-owner-validates-the-rest",
    }))
    .expect("candidate JSON");
    assert!(matches!(
        classify_exact_probe_artifact(&candidate, SCHEMA),
        Ok(ExactProbeArtifact::QualifiedCandidate(_))
    ));

    for invalid in [
        b"not-json".to_vec(),
        serde_json::to_vec(&json!({
            "schema": SCHEMA,
            "record": "UNKNOWN_RECORD",
        }))
        .expect("unknown record JSON"),
        serde_json::to_vec(&json!({
            "schema": SCHEMA,
            "record": "EXACT_MULTICORE_PROBE_STATE",
            "event": "FINAL",
            "status": "QUALIFIED",
            "qualifies_full_run": false,
            "reason": "contradictory qualified state",
        }))
        .expect("unknown state JSON"),
        serde_json::to_vec(&json!({
            "schema": SCHEMA,
            "record": "EXACT_MULTICORE_PROBE_STATE",
            "event": "NOT_RUN",
            "status": "REFUSE_FULL_RUN",
            "qualifies_full_run": true,
            "reason": "state may never qualify",
        }))
        .expect("qualified state JSON"),
        serde_json::to_vec(&json!({
            "schema": SCHEMA,
            "record": "EXACT_MULTICORE_PROBE_STATE",
            "event": "FAIL",
            "status": "FAIL",
            "qualifies_full_run": false,
        }))
        .expect("reasonless terminal state JSON"),
    ] {
        let error = classify_exact_probe_artifact(&invalid, SCHEMA)
            .expect_err("malformed or unknown artifacts fail closed");
        assert!(error.starts_with("FAILED:"), "{error}");
    }

    let bdd = include_str!("bdd.rs");
    let validator_start = bdd
        .find("fn validate_parity_probe(")
        .expect("BDD probe validator exists");
    let validator_end = bdd[validator_start..]
        .find("fn parity_teacher_observer(")
        .map(|offset| validator_start + offset)
        .expect("BDD probe validator section is bounded");
    let validator = &bdd[validator_start..validator_end];
    assert!(validator.contains("classify_exact_probe_artifact"));
    assert!(validator.contains("project_parity_probe_state"));
    assert!(validator.contains("FixtureStatus::failed_with_cid"));
}

#[test]
fn exact_probe_prepublication_binds_nonqualified_state_to_metadata_and_nonpass_outcome() {
    const SCHEMA: &str = "uor-r4.exact-multicore-probe/2";
    let bytes = serde_json::to_vec(&json!({
        "schema": SCHEMA,
        "record": "EXACT_MULTICORE_PROBE_STATE",
        "event": "NOT_RUN",
        "status": "REFUSE_FULL_RUN",
        "qualifies_full_run": false,
        "reason": "bounded preflight did not qualify the full run",
    }))
    .expect("state JSON");
    let cid = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    let ExactProbeArtifact::NonQualified(state) =
        classify_exact_probe_artifact(&bytes, SCHEMA).expect("recognized state")
    else {
        panic!("state artifact must remain non-qualified")
    };
    let fixture = state.fixture_status(cid.clone());
    let initial_fixture = FixtureStatus::not_run("probe admission has not been evaluated");
    assert!(validate_nonqualified_probe_prepublication(
        RunStatus::Fail,
        Some(&initial_fixture),
        &state,
        &cid,
    )
    .is_err());
    validate_nonqualified_probe_prepublication(RunStatus::NotRun, Some(&fixture), &state, &cid)
        .expect("truthful refusal may finalize as non-PASS");
    validate_nonqualified_probe_prepublication(RunStatus::Fail, Some(&fixture), &state, &cid)
        .expect("an independent failure may dominate the refusal");

    for (label, overall, planted_fixture) in [
        ("pass", RunStatus::Pass, fixture.clone()),
        (
            "wrong CID",
            RunStatus::NotRun,
            FixtureStatus::not_run_with_cid("blake3:stale", state.reason.clone()),
        ),
        (
            "wrong verdict",
            RunStatus::NotRun,
            FixtureStatus::failed_with_cid(cid.clone(), state.reason.clone()),
        ),
        (
            "wrong reason",
            RunStatus::NotRun,
            FixtureStatus::not_run_with_cid(cid.clone(), "stale reason"),
        ),
    ] {
        let error = validate_nonqualified_probe_prepublication(
            overall,
            Some(&planted_fixture),
            &state,
            &cid,
        )
        .expect_err(label);
        assert!(error.starts_with("FAIL:"), "{label}: {error}");
    }
    assert!(
        validate_nonqualified_probe_prepublication(RunStatus::NotRun, None, &state, &cid,).is_err()
    );

    let bdd = include_str!("bdd.rs");
    let validator_start = bdd
        .find("fn validate_parity_prepublication(")
        .expect("BDD prepublication validator exists");
    let validator_end = bdd[validator_start..]
        .find("fn append_operational_failure(")
        .map(|offset| validator_start + offset)
        .expect("BDD prepublication validator is bounded");
    let validator = &bdd[validator_start..validator_end];
    assert!(validator.contains("classify_exact_probe_artifact"));
    assert!(validator.contains("validate_nonqualified_probe_prepublication"));
    assert!(validator.contains("ExactMulticoreProbeReport = serde_json::from_value"));

    let early_failure_start = bdd
        .find("fn load_parity_fixtures()")
        .expect("BDD fixture loader exists");
    let early_failure_end = bdd[early_failure_start..]
        .find("let execution = TeacherExecutionConfig")
        .map(|offset| early_failure_start + offset)
        .expect("BDD early preflight section is bounded");
    let early_failure = &bdd[early_failure_start..early_failure_end];
    assert!(early_failure.contains("present_nonqualified_probe_fixture_status"));
    assert!(early_failure.contains("\"exact_multicore_probe\".to_owned(), fixture"));
}

#[test]
fn config_defaults_to_all_available_workers_and_eight_independent_streams() {
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let available = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1);

    assert_eq!(config.workers.get(), available);
    assert_eq!(config.streams, nonzero(8));
    assert_eq!(config.batch_per_worker, nonzero(4));
    assert_eq!(config.progress_every, NonZeroU64::new(10).unwrap());
    assert_eq!(config.max_wall, NonZeroU64::new(28_800).unwrap());
    assert_eq!(config.positions, nonzero(256));
    assert_eq!(config.gen_tokens, nonzero(8));
    assert_eq!(config.runs, nonzero(1));
    assert_eq!(config.corpus_positions, nonzero(1_000));
    assert_eq!(config.fmm_positions, nonzero(256));
    assert_eq!(config.probe_positions, nonzero(1));
    assert_eq!(config.mode, ObservabilityMode::Enabled);
    assert_eq!(
        config.report_path,
        PathBuf::from("target/teacher-parity/parity-report.json")
    );
}

#[test]
fn configured_wall_ceiling_cannot_exceed_eight_hours() {
    let error = ParityConfig::from_lookup(|name| {
        (name == "R4_PARITY_MAX_WALL_SECS").then(|| "28801".to_owned())
    })
    .expect_err("a longer live dispatch ceiling must fail closed");
    assert!(matches!(
        error,
        ConfigError::OutOfRange {
            name: "R4_PARITY_MAX_WALL_SECS",
            value: 28_801,
            minimum: 1,
            maximum: 28_800,
        }
    ));
}

#[test]
fn binding_shape_requires_canonical_streams_and_one_run_not_equal_workers() {
    let mut config = ParityConfig::from_lookup(|_| None).expect("default config");
    config.workers = nonzero(8);
    config.streams = nonzero(8);
    validate_binding_host_shape(&config, 8).expect("canonical S8/W8 shape");

    config.streams = nonzero(16);
    assert!(validate_binding_host_shape(&config, 8).is_err());
    config.workers = nonzero(4);
    config.streams = nonzero(8);
    validate_binding_host_shape(&config, 8).expect("S8 and W4 are independent");
    config.runs = nonzero(2);
    assert!(validate_binding_host_shape(&config, 8).is_err());
}

#[test]
fn diagnostic_worker_and_stream_bounds_remain_independently_configurable() {
    let available = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1);
    if available < 2 {
        return;
    }
    let workers = (available / 2).max(1);
    let streams = 8;
    let worker_text = workers.to_string();
    let default_streams = ParityConfig::from_lookup(|name| {
        (name == "R4_PARITY_WORKERS").then(|| worker_text.clone())
    })
    .expect("worker-only diagnostic config");
    assert_eq!(default_streams.workers.get(), workers);
    assert_eq!(default_streams.streams.get(), 8);
    let config = ParityConfig::from_lookup(|name| match name {
        "R4_PARITY_WORKERS" => Some(workers.to_string()),
        "R4_PARITY_STREAMS" => Some(streams.to_string()),
        _ => None,
    })
    .expect("diagnostic asymmetric shape parses");
    assert_eq!(config.workers.get(), workers);
    assert_eq!(config.streams.get(), streams);
    validate_binding_host_shape(&config, available)
        .expect("canonical streams admit W independently");
}

#[test]
fn planted_fan_out_lane_evidence_is_not_private_multistream_evidence() {
    let duplicate_seeds = vec!["blake3:same".to_owned(); 8];
    let outputs = (0..8)
        .map(|lane| format!("blake3:out-{lane}"))
        .collect::<Vec<_>>();
    let error = validate_private_multistream_evidence(&duplicate_seeds, &outputs, 8)
        .expect_err("one seed fanned to eight lanes must be refused");
    assert!(error.to_string().contains("fan-out"));

    let distinct_seeds = (0..8)
        .map(|lane| format!("blake3:seed-{lane}"))
        .collect::<Vec<_>>();
    validate_private_multistream_evidence(&distinct_seeds, &outputs, 8)
        .expect("complete ordered private lane evidence");
}

#[test]
fn config_rejects_zero_invalid_and_out_of_range_values_without_coupling_streams_to_workers() {
    let zero_workers =
        ParityConfig::from_lookup(lookup(HashMap::from([("R4_PARITY_WORKERS", "0")])));
    assert!(matches!(
        zero_workers,
        Err(ConfigError::NonPositive {
            name: "R4_PARITY_WORKERS",
            ..
        })
    ));

    let invalid_interval = ParityConfig::from_lookup(lookup(HashMap::from([(
        "R4_PARITY_PROGRESS_EVERY_SECS",
        "soon",
    )])));
    assert!(matches!(
        invalid_interval,
        Err(ConfigError::InvalidInteger {
            name: "R4_PARITY_PROGRESS_EVERY_SECS",
            ..
        })
    ));

    let available = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1);
    let workers = available.to_string();
    let independent_streams = ParityConfig::from_lookup(|name| match name {
        "R4_PARITY_WORKERS" => Some(workers.clone()),
        "R4_PARITY_STREAMS" => Some("1".to_owned()),
        _ => None,
    })
    .expect("diagnostic stream width is independent of row workers");
    assert_eq!(independent_streams.workers.get(), available);
    assert_eq!(independent_streams.streams.get(), 1);

    let zero_batch = ParityConfig::from_lookup(lookup(HashMap::from([
        ("R4_PARITY_WORKERS", "1"),
        ("R4_PARITY_STREAMS", "1"),
        ("R4_PARITY_BATCH_PER_WORKER", "0"),
    ])));
    assert!(matches!(
        zero_batch,
        Err(ConfigError::NonPositive {
            name: "R4_PARITY_BATCH_PER_WORKER",
            ..
        })
    ));

    let above_available = available.saturating_add(1).to_string();
    let too_many_workers = ParityConfig::from_lookup(|name| {
        (name == "R4_PARITY_WORKERS").then(|| above_available.clone())
    });
    assert!(matches!(
        too_many_workers,
        Err(ConfigError::WorkersAboveAvailable {
            requested,
            available: actual_available,
        }) if requested == available + 1 && actual_available == available
    ));

    let blank_report =
        ParityConfig::from_lookup(lookup(HashMap::from([("R4_PARITY_REPORT", "   ")])));
    assert!(matches!(
        blank_report,
        Err(ConfigError::InvalidPath {
            name: "R4_PARITY_REPORT",
            ..
        })
    ));

    for name in [
        "R4_PARITY_POSITIONS",
        "R4_PARITY_GEN_TOKENS",
        "R4_PARITY_RUNS",
        "R4_PARITY_CORPUS_POSITIONS",
        "R4_FMM_POSITIONS",
        "R4_EXACT_PROBE_POSITIONS",
    ] {
        let zero_budget =
            ParityConfig::from_lookup(|candidate| (candidate == name).then(|| "0".to_owned()));
        assert!(matches!(
            zero_budget,
            Err(ConfigError::NonPositive { name: actual, .. }) if actual == name
        ));
        let malformed_budget =
            ParityConfig::from_lookup(|candidate| (candidate == name).then(|| "many".to_owned()));
        assert!(matches!(
            malformed_budget,
            Err(ConfigError::InvalidInteger { name: actual, .. }) if actual == name
        ));
    }
    let too_many_probe_positions = ParityConfig::from_lookup(|name| {
        (name == "R4_EXACT_PROBE_POSITIONS").then(|| "9".to_owned())
    });
    assert!(matches!(
        too_many_probe_positions,
        Err(ConfigError::OutOfRange {
            name: "R4_EXACT_PROBE_POSITIONS",
            value: 9,
            minimum: 1,
            maximum: 8,
        })
    ));
    let too_many_generation_tokens =
        ParityConfig::from_lookup(|name| (name == "R4_PARITY_GEN_TOKENS").then(|| "9".to_owned()));
    assert!(matches!(
        too_many_generation_tokens,
        Err(ConfigError::OutOfRange {
            name: "R4_PARITY_GEN_TOKENS",
            value: 9,
            minimum: 1,
            maximum: 8,
        })
    ));
    let non_power_generation_tokens =
        ParityConfig::from_lookup(|name| (name == "R4_PARITY_GEN_TOKENS").then(|| "3".to_owned()));
    assert!(matches!(
        non_power_generation_tokens,
        Err(ConfigError::InvalidAdaptiveMaximum { value: 3 })
    ));
}

#[test]
fn every_present_nonunicode_parity_control_fails_instead_of_defaulting() {
    for name in [
        "R4_PARITY_WORKERS",
        "R4_PARITY_STREAMS",
        "R4_PARITY_BATCH_PER_WORKER",
        "R4_PARITY_PROGRESS_EVERY_SECS",
        "R4_PARITY_MAX_WALL_SECS",
        "R4_PARITY_POSITIONS",
        "R4_PARITY_GEN_TOKENS",
        "R4_PARITY_RUNS",
        "R4_PARITY_CORPUS_POSITIONS",
        "R4_FMM_POSITIONS",
        "R4_EXACT_PROBE_POSITIONS",
        "R4_PARITY_TELEMETRY",
        "R4_PARITY_REPORT",
    ] {
        let error = ParityConfig::from_env_lookup(|candidate| {
            if candidate == name {
                Err(std::env::VarError::NotUnicode(std::ffi::OsString::from(
                    "planted-nonunicode",
                )))
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .expect_err("a present non-Unicode control must never select its default");
        assert!(matches!(
            error,
            ConfigError::NonUnicode { name: actual } if actual == name
        ));
    }
}

#[test]
fn counters_distinguish_logical_work_from_batches_padding_cache_and_worker_tiles() {
    let plan = WorkPlan {
        logical_forwards: 15,
        tokens: 15,
        physical_batches: 2,
        matrix_calls: 10,
        batched_matrix_calls: 8,
        max_matrix_batch_width: 8,
        padded_forwards: 1,
        cache_hits: 7,
        streams: 8,
        worker_tasks: 24,
        row_tiles: 48,
        output_cells: 100,
        scalar_terms: 1_000,
    };
    let counters = WorkCounters::new(plan);
    counters.record_batch(8, 0, 4, 12, 24);
    counters.record_batch(7, 1, 3, 12, 24);
    counters.record_tokens(15);
    counters.record_matrix_calls(10);
    counters.record_batched_matrix_calls(8);
    counters.record_max_matrix_batch_width(8);
    counters.record_output_cells(100);
    counters.record_scalar_terms(1_000);
    for _ in 0..8 {
        counters.record_stream_completed();
    }

    let snapshot = counters.snapshot();
    assert_eq!(snapshot.logical_forwards, 15);
    assert_eq!(snapshot.physical_batches, 2);
    assert_eq!(snapshot.padded_forwards, 1);
    assert_eq!(snapshot.cache_hits, 7);
    assert_eq!(snapshot.streams, 8);
    assert_eq!(snapshot.worker_tasks, 24);
    assert_eq!(snapshot.row_tiles, 48);
    assert_eq!(snapshot.failed_worker_tasks, 0);
    assert_eq!(
        counters.completion_status(RunStatus::Pass).status,
        RunStatus::Pass
    );
}

#[test]
fn planted_worker_failure_and_incomplete_accounting_cannot_be_reported_as_pass() {
    let plan = WorkPlan {
        logical_forwards: 8,
        physical_batches: 1,
        padded_forwards: 0,
        cache_hits: 0,
        streams: 8,
        worker_tasks: 16,
        row_tiles: 32,
        ..WorkPlan::default()
    };
    let counters = WorkCounters::new(plan);
    counters.record_batch(8, 0, 0, 15, 31);
    counters.record_worker_task_failed();

    let verdict = counters.completion_status(RunStatus::Pass);
    assert_eq!(verdict.status, RunStatus::Fail);
    let detail = verdict.detail.expect("failure detail");
    assert!(detail.contains("worker_tasks 15/16"), "{detail}");
    assert!(detail.contains("row_tiles 31/32"), "{detail}");
    assert!(detail.contains("failed_worker_tasks 1"), "{detail}");
}

#[test]
fn work_plan_is_install_once_and_cannot_be_rewritten_after_work_starts() {
    let plan = WorkPlan {
        logical_forwards: 1,
        physical_batches: 1,
        ..WorkPlan::default()
    };
    let counters = WorkCounters::unplanned();
    counters.set_plan(plan).expect("first plan installation");
    assert_eq!(
        counters.set_plan(plan),
        Err(PlanInstallError::AlreadyInstalled)
    );

    let late = WorkCounters::unplanned();
    late.record_logical_forwards(1);
    assert_eq!(
        late.set_plan(plan),
        Err(PlanInstallError::WorkAlreadyStarted)
    );
    let verdict = late.completion_status(RunStatus::Pass);
    assert_eq!(verdict.status, RunStatus::Fail);
    assert!(verdict
        .detail
        .expect("fail-closed detail")
        .contains("before the plan was installed"));
}

#[test]
fn adaptive_plan_can_only_close_down_to_observed_work() {
    let ceiling = WorkPlan {
        logical_forwards: 100,
        tokens: 100,
        physical_batches: 14,
        max_matrix_batch_width: 8,
        streams: 16,
        ..WorkPlan::default()
    };
    let counters = WorkCounters::new(ceiling);
    counters.record_logical_forwards(44);
    let reduced = WorkPlan {
        logical_forwards: 44,
        tokens: 44,
        physical_batches: 14,
        max_matrix_batch_width: 8,
        streams: 16,
        ..WorkPlan::default()
    };
    counters
        .reduce_plan(reduced)
        .expect("adaptive upper bound closes to selected work");
    let increase = WorkPlan {
        logical_forwards: 45,
        ..reduced
    };
    assert_eq!(
        counters.reduce_plan(increase),
        Err(PlanInstallError::ReductionWouldIncrease)
    );
    let below_observed = WorkPlan {
        logical_forwards: 43,
        ..reduced
    };
    assert_eq!(
        counters.reduce_plan(below_observed),
        Err(PlanInstallError::ReductionBelowObserved)
    );
}

#[test]
fn adaptive_decode_stops_early_only_above_margin_and_is_truthful_at_maximum() {
    assert_eq!(
        adaptive_decode_decision(1, 8, 1.10, 9.0),
        AdaptiveDecodeDecision::ExtendConservativeMarginNotCleared,
        "the conservative early margin is strict"
    );
    assert_eq!(
        adaptive_decode_decision(2, 8, 1.11, 1.11),
        AdaptiveDecodeDecision::StopEarlyConservativeMarginCleared
    );
    assert_eq!(
        adaptive_decode_decision(8, 8, 1.01, 1.01),
        AdaptiveDecodeDecision::StopAtMaximumAcceptanceCleared
    );
    assert_eq!(
        adaptive_decode_decision(8, 8, 1.0, 20.0),
        AdaptiveDecodeDecision::StopAtMaximumNotEstablished,
        "the >1.0 acceptance boundary is strict and requires both engines"
    );
}

#[test]
fn adaptive_checkpoints_are_one_cumulative_causal_wave_not_repeat_counts() {
    assert_eq!(adaptive_decode_checkpoints(1), vec![1]);
    assert_eq!(adaptive_decode_checkpoints(2), vec![1, 2]);
    assert_eq!(adaptive_decode_checkpoints(4), vec![1, 2, 4]);
    assert_eq!(adaptive_decode_checkpoints(8), vec![1, 2, 4, 8]);
}

#[test]
fn bdd_source_keeps_one_time_compiled_cohorts_before_teacher_continuation() {
    // This is deliberately a source-contract regression test. Fixture-backed
    // execution counters remain the behavioral evidence; the source check
    // prevents the expensive rebuild/reload loop from being reintroduced in
    // environments where those fixtures are unavailable.
    let source = include_str!("bdd.rs");
    assert!(!source.contains("timed_legacy_generate"));
    assert!(!source.contains("timed_graph_generate"));

    let s4_start = source
        .find("fn parity_time_generation(")
        .expect("S4 timing step exists");
    let s4_end = source[s4_start..]
        .find("#[then(\"the measured concurrent token rate")
        .map(|offset| s4_start + offset)
        .expect("S4 timing step has a bounded source section");
    let s4 = &source[s4_start..s4_end];
    assert_eq!(s4.matches("timed_legacy_causal_wave(").count(), 1);
    assert_eq!(s4.matches("timed_graph_causal_wave(").count(), 1);
    let legacy = s4.find("timed_legacy_causal_wave(").unwrap();
    let graph = s4.find("timed_graph_causal_wave(").unwrap();
    let teacher_prepare = s4.find("prepare_teacher_generation(").unwrap();
    let teacher_decode = s4.find("timed_teacher_decode_to(").unwrap();
    assert!(legacy < graph && graph < teacher_prepare && teacher_prepare < teacher_decode);
    assert!(s4.contains("legacy_samples[stage].aggregate_tps"));
    assert!(s4.contains("graph_samples[stage].aggregate_tps"));

    let graph_wave_start = source
        .find("fn timed_graph_causal_wave(")
        .expect("graph causal wave exists");
    let graph_wave_end = source[graph_wave_start..]
        .find("fn generation_wave_json(")
        .map(|offset| graph_wave_start + offset)
        .expect("graph causal wave has a bounded source section");
    let graph_wave = &source[graph_wave_start..graph_wave_end];
    assert_eq!(graph_wave.matches("load_r4g1(").count(), 1);
    let lane_loop = graph_wave.find("for stream in 0..streams").unwrap();
    let lane_state_load = graph_wave.find("state: load_r4g1(").unwrap();
    assert!(lane_loop < lane_state_load && lane_state_load < graph_wave.find("for _step").unwrap());
    let graph_trajectory_start = source
        .find("struct GraphGenerationTrajectory")
        .expect("graph trajectory owns its state");
    let graph_trajectory_end = source[graph_trajectory_start..]
        .find("fn parity_source_dir")
        .map(|offset| graph_trajectory_start + offset)
        .expect("graph trajectory declaration has a bounded source section");
    let graph_trajectory = &source[graph_trajectory_start..graph_trajectory_end];
    assert!(graph_trajectory.contains("state: R4g1State"));
    assert!(!source.contains("struct GraphGenerationWorker"));
    assert!(
        graph_wave.contains("trajectory\n                                            .state"),
        "each prediction must use the lane-owned mutable policy state"
    );

    let loader_start = source
        .find("fn load_parity_fixtures(")
        .expect("fixture loader exists");
    let loader_end = source[loader_start..]
        .find("fn load_fmm_candidate(")
        .map(|offset| loader_start + offset)
        .expect("fixture loader has a bounded source section");
    let loader = &source[loader_start..loader_end];
    assert!(
        !loader.contains("Tokenizer::try_load"),
        "parity admission must not consult unbound sibling or hard-coded vocab.json files"
    );
    assert_eq!(
        loader
            .matches("Tokenizer::from_bytes(&tokenizer_bytes)")
            .count(),
        1,
        "fixture loading must parse only the content-bound tokenizer.bin bytes"
    );
    assert!(
        loader.find("teacher_free_parity_preflight()").unwrap()
            < loader.find("validate_parity_probe(").unwrap(),
        "compiled structural preflight must gate the source-only tuner"
    );

    let adoption_start = source
        .find("fn adopt_probe_execution(")
        .expect("probe adoption exists");
    let adoption_end = source[adoption_start..]
        .find("fn validate_parity_probe(")
        .map(|offset| adoption_start + offset)
        .expect("probe adoption has a bounded source section");
    let adoption = &source[adoption_start..adoption_end];
    assert!(
        !adoption.contains("requested_workers ="),
        "probe selection must not rewrite the operator-requested worker count"
    );
    assert!(adoption.contains("effective_workers = shape.selected_workers"));
    assert!(adoption.contains("probe_selected_execution"));

    let preflight_start = source
        .find("fn teacher_free_parity_preflight(")
        .expect("teacher-free preflight entry exists");
    let preflight_end = source[preflight_start..]
        .find("struct PromptWork")
        .map(|offset| preflight_start + offset)
        .expect("teacher-free preflight has a bounded source section");
    let preflight = &source[preflight_start..preflight_end];
    for forbidden in [
        "parity_run(",
        "parity_progress(",
        "validate_parity_probe(",
        "SmolLm2Oracle",
    ] {
        assert!(
            !preflight.contains(forbidden),
            "standalone preflight must not initialize live teacher/run machinery: {forbidden}"
        );
    }
    assert!(preflight.contains("\"teacher_forwards\": 0"));
    assert!(preflight.contains("\"authorizing_contract_cid\": exact_executor_contract_cid()"));
    assert!(preflight.contains("\"selected_source_dir\": source.display().to_string()"));
    assert!(!preflight.contains("Tokenizer::try_load"));
    assert!(preflight.contains("Tokenizer::from_bytes(&tokenizer_bytes)"));
    assert!(preflight.contains("let graph_states = (0..lane_seeds.len())"));
    assert!(preflight.contains("\"graph_state_preparations\": graph_states.len()"));
    let ordinary_input_start = source
        .find("fn teacher_free_input_evidence(")
        .expect("ordinary teacher-free input evidence exists");
    let production_input_start = source
        .find("fn production_admission_input_evidence(")
        .expect("production admission input evidence exists");
    let ordinary_input = &source[ordinary_input_start..production_input_start];
    assert!(ordinary_input.contains("std::fs::metadata(path)"));
    assert!(!ordinary_input.contains("symlink_metadata"));
    let production_input_end = source[production_input_start..]
        .find("fn teacher_free_preflight_failure_report(")
        .map(|offset| production_input_start + offset)
        .expect("production evidence helper has a bounded source section");
    let production_input = &source[production_input_start..production_input_end];
    assert!(production_input.contains("std::fs::symlink_metadata(path)"));
    assert!(production_input.contains("regular non-symlink file"));
    let refusal_start = source
        .find("fn teacher_free_preflight_failure_report(")
        .expect("teacher-free refusal evidence builder exists");
    let refusal_end = source[refusal_start..]
        .find("fn bind_teacher_free_preflight_report_path(")
        .map(|offset| refusal_start + offset)
        .expect("teacher-free refusal builder has a bounded source section");
    let refusal = &source[refusal_start..refusal_end];
    for required in [
        "\"reason\": reason",
        "\"authorizing_contract_cid\": exact_executor_contract_cid()",
        "\"teacher_source_opened\": false",
        "\"teacher_forwards\": 0",
        "\"selected_source_dir\"",
        "\"selected_bundle_dir\"",
        "\"teacher_model\"",
        "\"legacy_artifact\"",
        "\"graph\"",
        "\"graph_report\"",
        "\"production_admission\"",
        "production_admission_input_evidence",
    ] {
        assert!(
            refusal.contains(required),
            "missing refusal field {required}"
        );
    }
    assert!(refusal.contains("reason.starts_with(\"UNAVAILABLE:\")"));
    assert!(refusal.contains("\"FAILED\""));

    let automatic_start = source
        .find("fn load_parity_fixtures(")
        .expect("automatic fixture loader exists");
    let automatic_end = source[automatic_start..]
        .find("fn load_fmm_candidate(")
        .map(|offset| automatic_start + offset)
        .expect("automatic fixture loader has a bounded source section");
    let automatic = &source[automatic_start..automatic_end];
    assert!(automatic.contains("run_teacher_free_parity_preflight()"));
    assert!(automatic.contains("file_kappa(&teacher_free_preflight_path)?.0"));
    assert!(!automatic.contains("serde_json::to_vec(&teacher_free_preflight)"));
    let final_preflight_rehash = automatic
        .rfind("file_kappa(&teacher_free_preflight_path)?.0")
        .expect("preflight token is rehashed immediately before teacher load");
    let final_generation_rehash = automatic
        .rfind("production_admission_component_cids(&bundle)")
        .expect("complete schema-2 generation is rehashed before teacher load");
    let teacher_load = automatic
        .find("SmolLm2Oracle::load_with_sequence_length_and_execution(")
        .expect("live teacher load exists");
    assert!(final_preflight_rehash < final_generation_rehash);
    assert!(final_generation_rehash < teacher_load);
    assert!(automatic.contains("current_preflight_cid != preflight_cid"));
    assert!(automatic.contains("current_production_generation != admitted_production_generation"));
    assert!(
        automatic
            .find("run_teacher_free_parity_preflight()")
            .unwrap()
            < automatic.find("validate_parity_probe(").unwrap()
    );
    let main_start = source.find("async fn main()").expect("BDD main exists");
    let main_end = source[main_start..]
        .find("// =========================================================================")
        .map(|offset| main_start + offset)
        .expect("BDD main has a bounded source section");
    let main = &source[main_start..main_end];
    assert!(main.contains("run_teacher_free_parity_preflight()"));
    assert!(main.contains("teacher-free preflight refusal artifact"));
    assert!(main.contains("panic!(\"teacher-free parity preflight: {}\", failure.reason)"));
}

#[test]
fn bdd_source_preserves_all_variable_length_prompt_histories_without_extra_teacher_work() {
    let source = include_str!("bdd.rs");
    assert!(source.contains(
        "S4_REGISTERED_PROMPT_TOKEN_LENGTHS: [usize; S4_CANONICAL_STREAMS] = [6, 7, 6, 5, 4, 5, 6, 5]"
    ));

    let seeds_start = source
        .find("fn generation_lane_seeds(")
        .expect("generation seed policy exists");
    let seeds_end = source[seeds_start..]
        .find("fn generation_output_identities(")
        .map(|offset| seeds_start + offset)
        .expect("generation seed policy has a bounded source section");
    let seeds = &source[seeds_start..seeds_end];
    assert!(seeds.contains("seed.pop();"));
    assert!(!seeds.contains("seed.truncate("));
    assert!(seeds.contains("S4_REGISTERED_PROMPT_TOKEN_LENGTHS[lane]"));

    let transcript_start = source
        .find("fn build_teacher_transcript(")
        .expect("transcript builder exists");
    let transcript_end = source[transcript_start..]
        .find("fn teacher_transcript(")
        .map(|offset| transcript_start + offset)
        .expect("transcript builder has a bounded source section");
    let transcript = &source[transcript_start..transcript_end];
    assert!(transcript.contains("position + 1 == work.positions"));
    assert!(transcript.contains("generation_retained_prefix_tokens_per_lane"));
    assert!(!transcript.contains("common_prefix_tokens"));

    let adaptive_start = source
        .find("fn timed_teacher_decode_to(")
        .expect("adaptive teacher decode exists");
    let adaptive_end = source[adaptive_start..]
        .find("fn record_compiled_failure(")
        .map(|offset| adaptive_start + offset)
        .expect("adaptive teacher decode has a bounded source section");
    let adaptive = &source[adaptive_start..adaptive_end];
    assert!(adaptive.contains("cohort.retained_prefix_tokens_per_lane[lane]"));
    assert!(!adaptive.contains("positions.fill("));

    let preflight_start = source
        .find("fn teacher_free_parity_preflight(")
        .expect("teacher-free preflight exists");
    let preflight_end = source[preflight_start..]
        .find("struct PromptWork")
        .map(|offset| preflight_start + offset)
        .expect("teacher-free preflight has a bounded source section");
    let preflight = &source[preflight_start..preflight_end];
    assert!(preflight.contains("each lane's final teacher-forced prompt prefix"));
    assert!(preflight.contains("\"retained_prefix_tokens_per_lane\""));
    assert!(preflight.contains("\"seed_tokens_per_lane\""));
    assert!(preflight.contains("\"teacher_forwards\": 0"));
}

#[test]
fn bdd_source_binds_per_forward_overlap_and_zero_growth_after_excluded_preparation() {
    let source = include_str!("bdd.rs");
    let observer_start = source
        .find("fn parity_teacher_observer()")
        .expect("teacher observer exists");
    let observer_end = source[observer_start..]
        .find("fn teacher_execution_delta(")
        .map(|offset| observer_start + offset)
        .expect("observer source section is bounded");
    assert!(source[observer_start..observer_end].contains("snapshot.forward_max_active_workers"));

    let forward_start = source
        .find("fn teacher_forward_accounted(")
        .expect("accounted forward exists");
    let forward_end = source[forward_start..]
        .find("fn with_parity_fixtures")
        .map(|offset| forward_start + offset)
        .expect("accounted forward source section is bounded");
    let forward = &source[forward_start..forward_end];
    assert!(forward.contains("delta.multiworker_forward_calls != delta.forward_calls"));
    assert!(forward.contains("delta.forward_max_active_workers <= 1"));
    assert!(forward.contains("observed_workers != delta.forward_max_active_workers"));

    let loader_start = source
        .find("fn load_parity_fixtures(")
        .expect("fixture loader exists");
    let loader_end = source[loader_start..]
        .find("fn load_fmm_candidate(")
        .map(|offset| loader_start + offset)
        .expect("fixture loader source section is bounded");
    let loader = &source[loader_start..loader_end];
    let preparation = loader.find(".prepare_exact_execution(").unwrap();
    let measured = loader.find(".begin_measured_execution(").unwrap();
    assert!(preparation < measured);
    assert!(loader.contains("measured_start.workspace_growth_events != 0"));

    let transcript_start = source
        .find("fn build_teacher_transcript(")
        .expect("transcript builder exists");
    let transcript_end = source[transcript_start..]
        .find("fn teacher_transcript(")
        .map(|offset| transcript_start + offset)
        .expect("transcript source section is bounded");
    let transcript = &source[transcript_start..transcript_end];
    assert!(transcript.contains("forward.workspace_growth_events != 0"));

    let adaptive_start = source
        .find("fn timed_teacher_decode_to(")
        .expect("adaptive teacher decode exists");
    let adaptive_end = source[adaptive_start..]
        .find("fn record_compiled_failure(")
        .map(|offset| adaptive_start + offset)
        .expect("adaptive decode source section is bounded");
    let adaptive = &source[adaptive_start..adaptive_end];
    assert!(adaptive.contains("execution_delta.multiworker_forward_calls"));
    assert!(adaptive.contains("execution_delta.workspace_growth_events != 0"));
}

#[test]
fn standalone_json_publication_is_atomic_and_replaces_by_readback_value() {
    let dir = unique_dir("standalone-json");
    let path = dir.join("teacher-free-preflight.json");
    let first = json!({"schema": "preflight/1", "value": 1});
    assert_eq!(
        write_atomic_json(&path, &first).expect("first atomic publication"),
        path
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&path).expect("first bytes"))
            .expect("first JSON"),
        first
    );
    let second = json!({"schema": "preflight/1", "value": 2});
    write_atomic_json(&path, &second).expect("replacement atomic publication");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&path).expect("second bytes"))
            .expect("second JSON"),
        second
    );
    assert_eq!(
        fs::read_dir(&dir)
            .expect("list standalone directory")
            .count(),
        1,
        "successful publication must leave no temporary siblings"
    );
    fs::remove_dir_all(dir).expect("remove standalone JSON output");
}

#[test]
fn planted_graph_quality_refusal_is_atomically_published_before_non_pass_return() {
    let path = unique_dir("graph-quality-refusal").join("preflight.json");
    let reason =
        "FAILED: R4G1 quality gate failed: graph runtime top-1 25.65% is below TLA baseline 30.12%";
    let outcome = publish_atomic_preflight_outcome(&path, Err(reason.to_owned()), |reason| {
        json!({
            "schema": "uor-r4.teacher-parity-preflight/1",
            "status": "FAILED",
            "reason": reason,
            "report_path": path.display().to_string(),
            "selected_source_dir": "/fixture/source",
            "selected_bundle_dir": "/fixture/bundle",
            "teacher_source_opened": false,
            "teacher_forwards": 0,
            "inputs": {
                "graph": {
                    "path": "/fixture/bundle/graph/score.r4g1",
                    "presence": "PRESENT",
                    "cid": "blake3:graph",
                },
                "graph_report": {
                    "path": "/fixture/bundle/graph/score_report.json",
                    "presence": "PRESENT",
                    "cid": "blake3:report",
                },
            },
        })
    });
    assert_eq!(outcome.expect_err("quality refusal stays non-PASS"), reason);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("durable refusal bytes"))
            .expect("durable refusal JSON");
    assert_eq!(report["status"], "FAILED");
    assert_eq!(report["reason"], reason);
    assert_eq!(report["report_path"], path.display().to_string());
    assert_eq!(report["teacher_source_opened"], false);
    assert_eq!(report["teacher_forwards"], 0);
    assert_eq!(report["inputs"]["graph"]["presence"], "PRESENT");
}

#[test]
fn planted_graph_quality_refusal_projects_exact_fixture_evidence_into_final_metadata() {
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let mut metadata = RunMetadata::new(
        SchedulerSnapshot::from_config(&config),
        "exact",
        "exact-gemm",
        "test-isa",
    );
    let report_path = unique_dir("preflight-final-metadata").join("preflight.json");
    let reason = "FAILED: R4G1 load: R4G1 quality gate failed: graph runtime top-1 25.65% is below TLA baseline 30.12%";
    let report = json!({
        "teacher_source_opened": false,
        "teacher_forwards": 0,
        "inputs": {
            "teacher_model": {"presence": "PRESENT", "cid": null, "cid_status": "NOT_READ_TEACHER_FREE"},
            "teacher_config": {"presence": "PRESENT", "cid": null, "cid_status": "NOT_READ_TEACHER_FREE"},
            "tokenizer": {"presence": "PRESENT", "cid": "blake3:tokenizer"},
            "legacy_artifact": {"presence": "PRESENT", "cid": "blake3:tla"},
            "legacy_store": {"presence": "PRESENT", "cid": "blake3:tls"},
            "graph": {"presence": "PRESENT", "cid": "blake3:graph"},
            "graph_report": {"presence": "PRESENT", "cid": "blake3:graph-report"},
        },
    });
    apply_teacher_free_preflight_failure_metadata(
        &mut metadata,
        &report_path,
        Some("blake3:preflight-report"),
        &report,
        FixtureStatus::failed(reason),
        true,
        TeacherFreeGraphFailureStage::GraphLoadAttempted,
    );

    assert_eq!(
        metadata.paths.get("teacher_free_preflight_report"),
        Some(&report_path.display().to_string())
    );
    for fixture in ["tokenizer", "tla_artifact", "tls_store"] {
        assert_eq!(
            metadata.fixtures[fixture].verdict,
            FixtureVerdict::Available
        );
        assert!(metadata.fixtures[fixture].cid.is_some());
    }
    for fixture in ["teacher_weights", "teacher_config"] {
        assert_eq!(metadata.fixtures[fixture].verdict, FixtureVerdict::NotRun);
        assert!(metadata.fixtures[fixture].cid.is_none());
        assert!(metadata.fixtures[fixture]
            .reason
            .as_deref()
            .is_some_and(|detail| detail.contains("not opened")));
    }
    assert_eq!(
        metadata.fixtures["r4g1_graph"].verdict,
        FixtureVerdict::Failed
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph"].cid.as_deref(),
        Some("blake3:graph")
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph"].reason.as_deref(),
        Some(reason)
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph_report"].verdict,
        FixtureVerdict::Available
    );
    assert_eq!(
        metadata
            .identities
            .get("r4g1_graph_report")
            .map(String::as_str),
        Some("blake3:graph-report")
    );
    assert_eq!(
        metadata.fixtures["teacher_free_s4_preflight"]
            .cid
            .as_deref(),
        Some("blake3:preflight-report")
    );

    let report_reason =
        "FAILED: parse /fixture/bundle/graph/score_report.json: expected value at line 1";
    apply_teacher_free_preflight_failure_metadata(
        &mut metadata,
        &report_path,
        Some("blake3:preflight-report"),
        &report,
        FixtureStatus::failed(report_reason),
        true,
        TeacherFreeGraphFailureStage::ReportFailed,
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph"].verdict,
        FixtureVerdict::NotRun
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph"].cid.as_deref(),
        Some("blake3:graph")
    );
    assert!(metadata.fixtures["r4g1_graph"]
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains(report_reason)));
    assert_eq!(
        metadata.fixtures["r4g1_graph_report"].verdict,
        FixtureVerdict::Failed
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph_report"].cid.as_deref(),
        Some("blake3:graph-report")
    );
    assert_eq!(
        metadata.fixtures["r4g1_graph_report"].reason.as_deref(),
        Some(report_reason)
    );
}

#[test]
fn planted_missing_input_refusal_retains_paths_presence_and_unavailable_status() {
    let path = unique_dir("missing-input-refusal").join("preflight.json");
    let reason = "UNAVAILABLE: graph artifact/report absent";
    let outcome = publish_atomic_preflight_outcome(&path, Err(reason.to_owned()), |reason| {
        json!({
            "schema": "uor-r4.teacher-parity-preflight/1",
            "status": "UNAVAILABLE",
            "reason": reason,
            "report_path": path.display().to_string(),
            "selected_source_dir": "/fixture/source",
            "selected_bundle_dir": "/fixture/bundle",
            "teacher_source_opened": false,
            "teacher_forwards": 0,
            "inputs": {
                "graph": {
                    "path": "/fixture/bundle/graph/score.r4g1",
                    "presence": "ABSENT",
                    "cid": null,
                },
            },
        })
    });
    assert_eq!(outcome.expect_err("missing input stays non-PASS"), reason);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("durable refusal bytes"))
            .expect("durable refusal JSON");
    assert_eq!(report["status"], "UNAVAILABLE");
    assert_eq!(report["reason"], reason);
    assert_eq!(report["inputs"]["graph"]["presence"], "ABSENT");
    assert!(report["inputs"]["graph"]["cid"].is_null());

    let report_cid = format!(
        "blake3:{}",
        blake3::hash(&fs::read(&path).expect("published refusal bytes")).to_hex()
    );
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let mut metadata = RunMetadata::new(
        SchedulerSnapshot::from_config(&config),
        "exact",
        "exact-gemm",
        "test-isa",
    );
    apply_teacher_free_preflight_failure_metadata(
        &mut metadata,
        &path,
        Some(&report_cid),
        &report,
        FixtureStatus::unavailable(reason),
        true,
        TeacherFreeGraphFailureStage::NotReached,
    );
    assert_eq!(
        metadata.fixtures["teacher_free_s4_preflight"].verdict,
        FixtureVerdict::Unavailable
    );
    assert_eq!(
        metadata.fixtures["teacher_free_s4_preflight"]
            .cid
            .as_deref(),
        Some(report_cid.as_str())
    );
    assert_eq!(
        metadata
            .identities
            .get("teacher_free_s4_preflight")
            .map(String::as_str),
        Some(report_cid.as_str())
    );
    assert!(!metadata
        .identities
        .contains_key("teacher_free_preflight_report"));
}

#[test]
fn planted_preflight_report_write_failure_retains_original_refusal_and_never_passes() {
    let dir = unique_dir("preflight-write-failure");
    fs::create_dir_all(&dir).expect("test directory");
    let blocking_file = dir.join("not-a-directory");
    fs::write(&blocking_file, b"block").expect("blocking file");
    let path = blocking_file.join("preflight.json");
    let reason = "FAILED: planted graph quality refusal";
    let error = publish_atomic_preflight_outcome(&path, Err(reason.to_owned()), |reason| {
        json!({
            "schema": "uor-r4.teacher-parity-preflight/1",
            "status": "FAILED",
            "reason": reason,
            "report_path": path.display().to_string(),
            "teacher_source_opened": false,
            "teacher_forwards": 0,
        })
    })
    .expect_err("unwritable refusal report must fail closed");
    assert!(error.contains(reason));
    assert!(error.contains("invalidate stale report"));
    assert!(error.contains(&path.display().to_string()));
    assert!(!path.exists());
}

#[test]
fn planted_preflight_publication_failure_cannot_leave_a_stale_available_token() {
    let dir = unique_dir("preflight-stale-available");
    fs::create_dir_all(&dir).expect("test directory");
    let path = dir.join("preflight.json");
    fs::write(
        &path,
        b"{\"schema\":\"uor-r4.teacher-parity-preflight/1\",\"status\":\"AVAILABLE\"}\n",
    )
    .expect("stale AVAILABLE artifact");

    let publication = std::panic::catch_unwind(|| {
        let _ = publish_atomic_preflight_outcome(
            &path,
            Err("FAILED: planted current refusal".to_owned()),
            |_| panic!("planted failure after stale-token invalidation"),
        );
    });
    assert!(publication.is_err());
    assert!(
        !path.exists(),
        "a stale AVAILABLE admission token must be removed before current publication"
    );
    fs::remove_dir_all(dir).expect("remove stale-artifact test directory");
}

#[test]
fn standalone_preflight_path_has_a_stable_default_and_rejects_empty_override() {
    assert_eq!(
        configured_preflight_report_path(None).expect("default preflight path"),
        PathBuf::from("target/teacher-parity/teacher-free-preflight.json")
    );
    assert_eq!(
        configured_preflight_report_path(Some("target/custom-preflight.json".to_owned()))
            .expect("custom preflight path"),
        PathBuf::from("target/custom-preflight.json")
    );
    assert!(matches!(
        configured_preflight_report_path(Some("   ".to_owned())),
        Err(ConfigError::InvalidPath {
            name: "R4_PARITY_PREFLIGHT_REPORT",
            ..
        })
    ));
}

#[test]
fn fixture_directory_overrides_are_explicit_and_fail_closed() {
    let default = PathBuf::from("/registered/default");
    assert_eq!(
        configured_fixture_dir("R4_PARITY_SOURCE", Ok(None), default.clone())
            .expect("missing override uses the registered default"),
        default
    );
    assert_eq!(
        configured_fixture_dir(
            "R4_PARITY_BUNDLE",
            Ok(Some("../shared/bundle".to_owned())),
            PathBuf::from("/unused"),
        )
        .expect("explicit bundle path"),
        PathBuf::from("../shared/bundle")
    );
    assert!(configured_fixture_dir(
        "R4_PARITY_SOURCE",
        Ok(Some(" \t".to_owned())),
        PathBuf::from("/unused"),
    )
    .unwrap_err()
    .contains("R4_PARITY_SOURCE is empty"));
    assert!(configured_fixture_dir(
        "R4_PARITY_BUNDLE",
        Err("is not valid Unicode".to_owned()),
        PathBuf::from("/unused"),
    )
    .unwrap_err()
    .contains("R4_PARITY_BUNDLE is not valid Unicode"));
}

#[test]
fn standalone_json_write_failure_leaves_no_temporary_artifact() {
    let dir = unique_dir("standalone-json-failure");
    let blocked = dir.join("blocked.json");
    fs::create_dir_all(&blocked).expect("create blocking destination directory");
    let error = write_atomic_json(&blocked, &json!({"status": "AVAILABLE"}))
        .expect_err("a directory cannot be replaced by the standalone report");
    assert!(error.to_string().contains("telemetry I/O"));
    assert_eq!(
        fs::read_dir(&dir)
            .expect("list failed standalone directory")
            .count(),
        1,
        "failed publication must remove its temporary sibling"
    );
    fs::remove_dir_all(dir).expect("remove failed standalone JSON output");
}

#[test]
fn eta_reports_warming_estimated_unavailable_and_stall_without_guessing() {
    let warming = estimate_eta(EtaInput {
        completed: 1,
        total: 100,
        elapsed: Duration::from_secs(10),
        last_progress_age: Duration::ZERO,
        stall_after: Duration::from_secs(60),
        minimum_samples: 3,
    });
    assert_eq!(warming.status, EtaStatus::WarmingUp);
    assert_eq!(warming.remaining_seconds, None);

    let estimated = estimate_eta(EtaInput {
        completed: 25,
        total: 100,
        elapsed: Duration::from_secs(50),
        last_progress_age: Duration::from_secs(2),
        stall_after: Duration::from_secs(60),
        minimum_samples: 3,
    });
    assert_eq!(estimated.status, EtaStatus::Estimated);
    assert_eq!(estimated.remaining_seconds, Some(150));

    let unavailable = estimate_eta(EtaInput {
        completed: 0,
        total: 0,
        elapsed: Duration::from_secs(50),
        last_progress_age: Duration::ZERO,
        stall_after: Duration::from_secs(60),
        minimum_samples: 3,
    });
    assert_eq!(unavailable.status, EtaStatus::Unavailable);

    let stalled = estimate_eta(EtaInput {
        completed: 25,
        total: 100,
        elapsed: Duration::from_secs(500),
        last_progress_age: Duration::from_secs(61),
        stall_after: Duration::from_secs(60),
        minimum_samples: 3,
    });
    assert_eq!(stalled.status, EtaStatus::Stall);
    assert_eq!(stalled.remaining_seconds, None);
}

#[test]
fn heartbeat_jsonl_is_parseable_and_visible_before_the_log_is_dropped() {
    let dir = unique_dir("heartbeat");
    fs::create_dir_all(&dir).expect("create test output");
    let path = dir.join("run.events.jsonl");
    let mut log = HeartbeatLog::create(&path).expect("create heartbeat log");
    let event = HeartbeatEvent::new(
        "run-932",
        RunStatus::NotRun,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test sampler".to_owned(),
        },
    );

    log.append(event.clone()).expect("durably append heartbeat");
    let first = fs::read_to_string(&path).expect("flushed heartbeat is immediately readable");
    let parsed: HeartbeatEvent = serde_json::from_str(first.trim()).expect("heartbeat JSON");
    assert_eq!(parsed.sequence, 0);
    assert_eq!(parsed.run_id, "run-932");

    log.append(event).expect("append second heartbeat");
    let second = fs::read_to_string(&path).expect("read two heartbeats");
    let events: Vec<HeartbeatEvent> = second
        .lines()
        .map(|line| serde_json::from_str(line).expect("heartbeat line JSON"))
        .collect();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].sequence, 1);
    fs::remove_dir_all(&dir).expect("remove isolated test output");
}

#[test]
fn empirical_run_changes_cannot_change_supplied_deterministic_evidence_bytes() {
    let enabled_dir = unique_dir("enabled");
    let disabled_dir = unique_dir("disabled");
    fs::create_dir_all(&enabled_dir).expect("create enabled output");
    fs::create_dir_all(&disabled_dir).expect("create disabled output");

    let identity = BTreeMap::from([
        ("teacher_kappa".to_owned(), "blake3:teacher".to_owned()),
        ("artifact_kappa".to_owned(), "blake3:artifact".to_owned()),
    ]);
    let evidence = DeterministicEvidence::new(
        RunStatus::Pass,
        identity,
        json!({"tokens": [3, 1, 4], "logit_bits": [1065353216u32]}),
    );
    let plan = WorkPlan {
        logical_forwards: 1,
        physical_batches: 1,
        padded_forwards: 0,
        cache_hits: 0,
        streams: 1,
        worker_tasks: 1,
        row_tiles: 1,
        ..WorkPlan::default()
    };
    let counters = WorkCounters::new(plan);
    counters.record_batch(1, 0, 0, 1, 1);
    counters.record_stream_completed();
    let snapshot = counters.snapshot();

    let enabled = RunReport::new(
        "enabled-run",
        ObservabilityMode::Enabled,
        RunStatus::Pass,
        snapshot,
        estimate_eta(EtaInput {
            completed: 1,
            total: 1,
            elapsed: Duration::from_secs(1),
            last_progress_age: Duration::ZERO,
            stall_after: Duration::from_secs(60),
            minimum_samples: 1,
        }),
        ResourceAvailability::Unavailable {
            reason: "enabled test".to_owned(),
        },
    );
    let disabled = RunReport::new(
        "disabled-run",
        ObservabilityMode::Disabled,
        RunStatus::Pass,
        snapshot,
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "disabled".to_owned(),
        },
    );

    let enabled_report = enabled_dir.join("same-evidence.json");
    let disabled_report = disabled_dir.join("same-evidence.json");
    let enabled_paths =
        write_final_reports(&enabled_report, &enabled, &evidence).expect("write enabled reports");
    let disabled_paths = write_final_reports(&disabled_report, &disabled, &evidence)
        .expect("write disabled reports");

    assert_eq!(enabled_paths.run, enabled_report);
    assert!(enabled_paths
        .evidence
        .ends_with("same-evidence.evidence.json"));
    assert!(events_path_for_report(&enabled_paths.run)
        .expect("event path")
        .ends_with("same-evidence.events.jsonl"));
    assert_ne!(
        fs::read(&enabled_paths.run).expect("enabled run bytes"),
        fs::read(&disabled_paths.run).expect("disabled run bytes")
    );
    assert_eq!(
        fs::read(&enabled_paths.evidence).expect("enabled evidence bytes"),
        fs::read(&disabled_paths.evidence).expect("disabled evidence bytes"),
        "timing/resource/instrumentation state must never enter deterministic evidence"
    );
    assert_eq!(
        evidence.output,
        json!({"tokens": [3, 1, 4], "logit_bits": [1065353216u32]})
    );
    write_final_reports(&enabled_report, &disabled, &evidence)
        .expect("a new run atomically replaces the current report");
    assert_eq!(
        fs::read(&enabled_paths.run).expect("replaced empirical run"),
        fs::read(&disabled_paths.run).expect("expected replacement bytes")
    );

    fs::remove_dir_all(&enabled_dir).expect("remove enabled output");
    fs::remove_dir_all(&disabled_dir).expect("remove disabled output");
}

#[test]
fn deterministic_identity_projection_excludes_path_and_timing_bound_admission_cids() {
    let scheduler = SchedulerSnapshot::from_config(
        &ParityConfig::from_lookup(|_| None).expect("default config"),
    );
    let mut first = RunMetadata::new(scheduler.clone(), "exact", "kernel", "isa")
        .with_identity("model", "blake3:model")
        .with_identity("teacher_free_s4_preflight", "blake3:absolute-path-a")
        .with_identity("probe_selected_execution", "workers=8,tiles_per_worker=4");
    let output = json!({
        "S0_teacher_free_preflight": {
            "schema": "uor-r4.teacher-parity-preflight/1",
            "status": "UNAVAILABLE",
            "production_admission": {"release_manifest": {"presence": "ABSENT", "cid": null}}
        }
    });
    let projected_first =
        deterministic_evidence_identities(&first, &output).expect("first projection");
    first.identities.insert(
        "teacher_free_s4_preflight".to_owned(),
        "blake3:absolute-path-b-and-new-elapsed".to_owned(),
    );
    first.identities.insert(
        "probe_selected_execution".to_owned(),
        "workers=4,tiles_per_worker=4".to_owned(),
    );
    let projected_second =
        deterministic_evidence_identities(&first, &output).expect("second projection");
    assert_eq!(projected_first, projected_second);
    assert_eq!(projected_first["model"], "blake3:model");
    assert!(projected_first["teacher_free_s4_preflight"].starts_with("blake3:"));
    assert!(!projected_first.contains_key("probe_selected_execution"));

    let bdd = include_str!("bdd.rs");
    let project_start = bdd
        .find("fn project_parity_probe_state<T>(")
        .expect("nonqualified probe projection exists");
    let project_end = bdd[project_start..]
        .find("struct ParityAdmissionShape")
        .map(|offset| project_start + offset)
        .expect("nonqualified probe projection is bounded");
    let project = &bdd[project_start..project_end];
    let deterministic_start = project
        .find("parity_record_output(")
        .expect("nonqualified deterministic record exists");
    let deterministic_end = project[deterministic_start..]
        .find("project_parity_probe_nonpass")
        .map(|offset| deterministic_start + offset)
        .expect("nonqualified deterministic record is bounded");
    let deterministic = &project[deterministic_start..deterministic_end];
    assert!(!deterministic.contains("state.reason"));
    assert!(!deterministic.contains("artifact_cid"));
}

#[test]
fn zero_work_refusal_final_report_round_trips_without_nonfinite_rates() {
    let dir = unique_dir("zero-work-refusal-final-report");
    fs::create_dir_all(&dir).expect("create refusal output");
    let report_path = dir.join("parity-report.json");
    let work = WorkCounters::new(WorkPlan::default()).snapshot();
    let zero_rate = work.logical_forwards as f64 / 1.0;
    let report = RunReport::new(
        "graph-quality-refusal",
        ObservabilityMode::Enabled,
        RunStatus::Fail,
        work,
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "graph quality gate refused before teacher work".to_owned(),
        },
    )
    .with_rates(parity_observability::RateSnapshot {
        cumulative_forwards_per_second: Some(zero_rate),
        seconds_per_forward: seconds_per_forward_from_rate(Some(zero_rate)),
        ..parity_observability::RateSnapshot::default()
    });
    let evidence = DeterministicEvidence::new(
        RunStatus::Fail,
        BTreeMap::new(),
        json!({"teacher_forwards": 0, "refusal": "graph_quality"}),
    );

    let paths = write_final_reports(&report_path, &report, &evidence)
        .expect("zero-work refusal companions remain serializable");
    let decoded: RunReport =
        serde_json::from_slice(&fs::read(&paths.run).expect("read zero-work refusal report"))
            .expect("decode zero-work refusal report");
    assert_eq!(decoded.status, RunStatus::Fail);
    assert_eq!(decoded.work.logical_forwards, 0);
    assert_eq!(decoded.rates.cumulative_forwards_per_second, Some(0.0));
    assert_eq!(decoded.rates.seconds_per_forward, None);

    fs::remove_dir_all(&dir).expect("remove refusal output");
}

#[test]
fn explicit_nonfinite_final_rate_remains_rejected_by_semantic_readback() {
    let dir = unique_dir("nonfinite-final-rate");
    fs::create_dir_all(&dir).expect("create rejection output");
    let report_path = dir.join("parity-report.json");
    let report = RunReport::new(
        "invalid-nonfinite-rate",
        ObservabilityMode::Enabled,
        RunStatus::Fail,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test".to_owned(),
        },
    )
    .with_rates(parity_observability::RateSnapshot {
        cumulative_forwards_per_second: Some(0.0),
        seconds_per_forward: Some(f64::INFINITY),
        ..parity_observability::RateSnapshot::default()
    });
    let evidence = DeterministicEvidence::new(RunStatus::Fail, BTreeMap::new(), json!({}));

    let error = write_final_reports(&report_path, &report, &evidence)
        .expect_err("strict readback rejects an explicit nonfinite rate");
    assert!(error
        .to_string()
        .contains("temporary report readback changed serialized content"));
    assert!(!report_path.exists());

    fs::remove_dir_all(&dir).expect("remove rejection output");
}

#[test]
fn companion_publication_failure_cannot_leave_a_stale_canonical_pass_report() {
    let dir = unique_dir("publication-failure");
    fs::create_dir_all(&dir).expect("create publication output");
    let report_path = dir.join("parity-report.json");
    fs::write(&report_path, br#"{"status":"PASS","run_id":"stale"}"#).expect("seed stale report");
    let evidence_path = dir.join("parity-report.evidence.json");
    fs::create_dir(&evidence_path).expect("plant non-replaceable evidence directory");

    let report = RunReport::new(
        "current-run",
        ObservabilityMode::Enabled,
        RunStatus::Pass,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test".to_owned(),
        },
    );
    let evidence = DeterministicEvidence::new(
        RunStatus::Pass,
        BTreeMap::new(),
        json!({"output": "current"}),
    );
    assert!(write_final_reports(&report_path, &report, &evidence).is_err());
    assert!(
        !report_path.exists(),
        "a stale canonical PASS must not survive companion publication failure"
    );
    fs::remove_dir_all(&dir).expect("remove publication output");
}

#[test]
fn new_run_start_invalidates_a_prior_canonical_pass_before_heartbeat_work() {
    let dir = unique_dir("startup-stale-pass");
    fs::create_dir_all(&dir).expect("create startup output");
    let report_path = dir.join("parity-report.json");
    let evidence_path = dir.join("parity-report.evidence.json");
    fs::write(&report_path, br#"{"status":"PASS","run_id":"prior"}"#)
        .expect("seed prior PASS report");
    fs::write(&evidence_path, br#"{"status":"PASS"}"#).expect("seed prior PASS evidence");

    let owned = take_run_report_ownership(&dir, Some("parity-report.json".to_owned()))
        .expect("new run takes ownership of canonical companions");
    assert_eq!(owned, report_path);
    assert!(configured_exact_probe_report_path(Some("   ".to_owned())).is_err());
    assert!(!report_path.exists());
    assert!(!evidence_path.exists());

    let bdd = include_str!("bdd.rs");
    let init = bdd
        .split("fn initialize_parity_run()")
        .nth(1)
        .and_then(|tail| tail.split("fn parity_run()").next())
        .expect("bounded parity initialization source");
    let invalidate = init
        .find("take_run_report_ownership")
        .expect("initialization takes canonical companion ownership");
    let config = init
        .find("ParityConfig::from_env")
        .expect("initialization parses full config");
    let probe = init
        .find("parity_probe_report_path()")
        .expect("initialization resolves probe path");
    let preflight = init
        .find("parity_preflight_report_path()")
        .expect("initialization resolves preflight path");
    let fixtures = init
        .find("parity_fixture_dirs()")
        .expect("initialization resolves fixture directories");
    let heartbeat = init
        .find("HeartbeatWorker::spawn_with_stall_after")
        .expect("initialization starts heartbeat");
    assert!(invalidate < config);
    assert!(invalidate < probe);
    assert!(invalidate < preflight);
    assert!(invalidate < fixtures);
    assert!(invalidate < heartbeat);

    let support = include_str!("support/parity_observability.rs");
    let ownership_start = support
        .find("pub fn take_run_report_ownership(")
        .expect("ownership helper exists");
    let ownership_end = support[ownership_start..]
        .find("pub fn events_path_for_report(")
        .map(|offset| ownership_start + offset)
        .expect("ownership helper has a bounded source section");
    assert!(support[ownership_start..ownership_end]
        .contains("invalidate_final_reports_checked(&report_path)?"));

    fs::remove_dir_all(dir).expect("remove startup output");
}

#[test]
fn post_terminal_commit_failure_invalidates_every_prepared_pass_companion() {
    let dir = unique_dir("post-terminal-commit-failure");
    fs::create_dir_all(&dir).expect("create publication output");
    let report_path = dir.join("parity-report.json");
    let evidence_path = dir.join("parity-report.evidence.json");
    let report = RunReport::new(
        "current-run",
        ObservabilityMode::Enabled,
        RunStatus::Pass,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test".to_owned(),
        },
    );
    let evidence =
        DeterministicEvidence::new(RunStatus::Pass, BTreeMap::new(), json!({"exact": true}));
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let progress = SharedProgress::new(
        "current-run",
        RunMetadata::new(
            SchedulerSnapshot::from_config(&config),
            "exact",
            "exact-gemm",
            "test-isa",
        ),
    );
    progress
        .update(|live| live.status = RunStatus::Pass)
        .expect("candidate PASS progress");
    let prepared = prepare_final_reports(&report_path, &report, &evidence)
        .expect("PASS companions prepare before the terminal boundary");

    // Model a failure in the only post-terminal operation: the canonical run
    // path becomes non-replaceable after both temporary companions are synced.
    fs::create_dir(&report_path).expect("plant non-replaceable commit marker path");
    assert!(prepared.commit().is_err());
    mark_finalization_failed(&progress, "telemetry_commit_failed")
        .expect("failed commit downgrades retry/readback status");
    assert_eq!(
        progress.snapshot().expect("retry progress").live.status,
        RunStatus::Fail
    );
    assert!(
        !report_path.is_file(),
        "a failed post-terminal rename must not publish a PASS commit marker"
    );
    assert!(
        !evidence_path.exists(),
        "a partially published PASS evidence sidecar must be invalidated"
    );
    fs::remove_dir_all(&dir).expect("remove publication output");
}

#[test]
fn late_companion_validation_failure_invalidates_a_stale_canonical_pass() {
    let dir = unique_dir("late-validation-failure");
    fs::create_dir_all(&dir).expect("create publication output");
    let report_path = dir.join("parity-report.json");
    fs::write(&report_path, br#"{"status":"PASS","run_id":"stale"}"#).expect("seed stale report");
    let report = RunReport::new(
        "current-run",
        ObservabilityMode::Enabled,
        RunStatus::Pass,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test".to_owned(),
        },
    );
    let mismatched_evidence =
        DeterministicEvidence::new(RunStatus::Fail, BTreeMap::new(), json!({}));

    assert!(write_final_reports(&report_path, &report, &mismatched_evidence).is_err());
    assert!(
        !report_path.exists(),
        "a rejected late PASS candidate must invalidate the stale canonical PASS"
    );
    fs::remove_dir_all(&dir).expect("remove publication output");
}

#[test]
fn every_terminal_state_has_an_explicit_stable_machine_spelling() {
    for (status, spelling) in [
        (RunStatus::Pass, "PASS"),
        (RunStatus::Fail, "FAIL"),
        (RunStatus::Unavailable, "UNAVAILABLE"),
        (RunStatus::Aborted, "ABORTED"),
        (RunStatus::NotRun, "NOT_RUN"),
    ] {
        assert_eq!(
            serde_json::to_string(&status).expect("status JSON"),
            format!("\"{spelling}\"")
        );
    }
}

#[test]
fn resource_sampling_is_measured_or_explicitly_unavailable_never_zero_filled() {
    match sample_host_resources() {
        ResourceAvailability::Available(sample) => {
            assert!(!sample.architecture.trim().is_empty());
            assert!(!sample.operating_system.trim().is_empty());
            assert_measured_or_reason(&sample.cpu_model);
            assert_measured_or_reason(&sample.operating_system_version);
            assert_measured_or_reason(&sample.physical_cores);
            assert_measured_or_reason(&sample.logical_cores);
            assert_measured_or_reason(&sample.performance_cores);
            assert_measured_or_reason(&sample.efficiency_cores);
            assert_measured_or_reason(&sample.total_memory_bytes);
            assert_measured_or_reason(&sample.resident_set_bytes);
            assert_measured_or_reason(&sample.virtual_memory_bytes);
            assert_measured_or_reason(&sample.cpu_percent);
            assert_measured_or_reason(&sample.process_cpu_time_seconds);
            assert_measured_or_reason(&sample.thread_count);
            assert_measured_or_reason(&sample.peak_resident_set_bytes);
            #[cfg(target_os = "macos")]
            {
                assert!(matches!(
                    sample.resident_set_bytes,
                    Measurement::Available { .. }
                ));
                assert!(matches!(
                    sample.virtual_memory_bytes,
                    Measurement::Available { .. }
                ));
                assert!(matches!(sample.cpu_percent, Measurement::Available { .. }));
                assert!(matches!(
                    sample.process_cpu_time_seconds,
                    Measurement::Available { .. }
                ));
                match &sample.thread_count {
                    Measurement::Unavailable { reason } => assert!(reason.contains("thcount")),
                    Measurement::Available { .. } => {
                        panic!("macOS ps thread count must not be fabricated")
                    }
                }
            }
        }
        ResourceAvailability::Unavailable { reason } => {
            assert!(!reason.trim().is_empty());
        }
    }
}

#[test]
fn exact_live_occupancy_ignores_out_of_order_observer_callbacks() {
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let progress = SharedProgress::new(
        "run-932-epoch",
        RunMetadata::new(
            SchedulerSnapshot::from_config(&config),
            "exact",
            "exact-gemm",
            "test-isa",
        ),
    );
    progress
        .update(|state| state.queue.queue_depth = 8)
        .expect("publish queued lanes");
    progress.publish_exact(ExactProgressObservation {
        observer_epoch: 2,
        streams_started: 8,
        streams_completed: 0,
        active_streams: 8,
        peak_active_streams: 8,
        active_row_workers: 6,
        peak_active_row_workers: 6,
        matrix_calls: 5,
        batched_matrix_calls: 5,
        max_matrix_batch_width: 8,
        completed_worker_tasks: 12,
        output_cells_completed: 120,
        scalar_terms_completed: 1_200,
        effective_workers: 8,
    });
    progress.publish_exact(ExactProgressObservation {
        observer_epoch: 1,
        streams_started: 1,
        streams_completed: 0,
        active_streams: 1,
        peak_active_streams: 1,
        active_row_workers: 1,
        peak_active_row_workers: 1,
        matrix_calls: 1,
        batched_matrix_calls: 1,
        max_matrix_batch_width: 1,
        completed_worker_tasks: 1,
        output_cells_completed: 1,
        scalar_terms_completed: 1,
        effective_workers: 1,
    });
    let snapshot = progress.snapshot().expect("epoch-safe snapshot");
    assert_eq!(snapshot.live.queue.active_streams, 8);
    assert_eq!(snapshot.live.queue.queue_depth, 0);
    assert_eq!(snapshot.live.queue.active_row_workers, 6);
    assert_eq!(snapshot.live.queue.peak_active_streams, 8);
    assert_eq!(snapshot.live.queue.peak_active_row_workers, 6);
    assert_eq!(
        snapshot.live.metadata.scheduler.effective_row_workers.get(),
        8
    );

    progress.finish_exact_forward();
    let finished = progress.snapshot().expect("finished snapshot");
    assert_eq!(finished.live.queue.active_streams, 0);
    assert_eq!(finished.live.queue.active_row_workers, 0);
}

#[test]
fn deterministic_s2_and_probe_evidence_ignore_schedule_geometry() {
    let stable_work = TeacherExecutionSnapshot {
        observer_epoch: 10,
        requested_workers: 8,
        effective_workers: 8,
        active_workers: 0,
        max_active_workers: 5,
        forward_max_active_workers: 4,
        multiworker_forward_calls: 7,
        forward_calls: 8,
        streams_started: 64,
        streams_completed: 64,
        active_streams: 0,
        max_active_streams: 8,
        matrix_calls: 144,
        batched_matrix_calls: 144,
        max_matrix_batch_width: 8,
        tiles_completed: 576,
        output_cells_completed: 2_048,
        scalar_terms_completed: 65_536,
        workspace_growth_events: 0,
        workspace_growth_bytes: 0,
    };
    let scheduler_variant = TeacherExecutionSnapshot {
        observer_epoch: 9_999,
        requested_workers: 16,
        effective_workers: 4,
        active_workers: 3,
        max_active_workers: 16,
        forward_max_active_workers: 2,
        multiworker_forward_calls: 1,
        active_streams: 6,
        max_active_streams: 64,
        tiles_completed: 1_152,
        workspace_growth_events: 3,
        workspace_growth_bytes: 32_768,
        ..stable_work
    };

    let reference = deterministic_teacher_execution(stable_work);
    let actual = deterministic_teacher_execution(scheduler_variant);
    let reference_bytes = serde_json::to_vec(&reference).expect("reference exact-work JSON");
    let actual_bytes = serde_json::to_vec(&actual).expect("variant exact-work JSON");
    assert_eq!(actual_bytes, reference_bytes);

    let value = serde_json::to_value(actual).expect("deterministic exact-work JSON");
    for empirical_field in [
        "observer_epoch",
        "requested_workers",
        "effective_workers",
        "active_workers",
        "max_active_workers",
        "forward_max_active_workers",
        "multiworker_forward_calls",
        "active_streams",
        "max_active_streams",
        "tiles_completed",
        "workspace_growth_events",
        "workspace_growth_bytes",
    ] {
        assert!(
            value.get(empirical_field).is_none(),
            "scheduler field {empirical_field} leaked into deterministic S2 evidence"
        );
    }

    let different_work = TeacherExecutionSnapshot {
        scalar_terms_completed: stable_work.scalar_terms_completed + 1,
        ..stable_work
    };
    assert_ne!(
        deterministic_teacher_execution(different_work),
        reference,
        "a logical exact-work change must remain evidence-visible"
    );

    let bdd = include_str!("bdd.rs");
    let deterministic_start = bdd
        .find("\"S2_transcript\",")
        .expect("S2 deterministic evidence record exists");
    let deterministic_end = bdd[deterministic_start..]
        .find("\n    );\n}")
        .map(|offset| deterministic_start + offset)
        .expect("S2 deterministic evidence record is bounded");
    let deterministic_record = &bdd[deterministic_start..deterministic_end];
    assert!(deterministic_record
        .contains("\"exact_work\": deterministic_teacher_execution(evidence.execution)"));
    for empirical_field in [
        "max_active_streams",
        "peak_active_row_workers",
        "execution_preparation",
        "workspace_growth_events",
        "workspace_growth_bytes",
        "owner_plan",
        "worker_tasks",
        "row_tiles",
        "tiles_completed",
        "\"execution\": evidence.execution",
    ] {
        assert!(
            !deterministic_record.contains(empirical_field),
            "empirical field {empirical_field} leaked into the S2 deterministic record"
        );
    }
    assert!(
        bdd[..deterministic_start].contains("\"S2_execution_observability\"")
            && bdd[..deterministic_start].contains("\"execution_snapshot\": evidence.execution"),
        "the full S2 execution snapshot must remain available as empirical telemetry"
    );

    let probe_start = bdd
        .find("\"exact_probe_trace\",")
        .expect("exact-probe deterministic record exists");
    let probe_end = bdd[probe_start..]
        .find("\n    );")
        .map(|offset| probe_start + offset)
        .expect("exact-probe deterministic record is bounded");
    let probe_record = &bdd[probe_start..probe_end];
    for empirical_field in ["worker_rows", "workers", "forward_plan"] {
        assert!(
            !probe_record.contains(empirical_field),
            "probe worker field {empirical_field} leaked into deterministic evidence"
        );
    }
    assert!(
        bdd[..probe_start].contains("\"exact_probe_worker_rows\"")
            && bdd[..probe_start].contains("\"forward_plan\": run.forward_plan"),
        "per-worker exact-probe plans must remain available as empirical telemetry"
    );
}

#[test]
fn host_resource_schema_round_trips_explicit_performance_and_efficiency_topology() {
    let sample = sample_host_resources();
    let bytes = serde_json::to_vec(&sample).expect("resource JSON");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("resource value");
    if value["availability"] == "AVAILABLE" {
        assert!(value.get("performance_cores").is_some());
        assert!(value.get("efficiency_cores").is_some());
        assert!(value.get("max_sampled_resident_set_bytes").is_some());
        assert!(value.get("thread_count").is_some());
    }
    let decoded: ResourceAvailability =
        serde_json::from_slice(&bytes).expect("resource schema round trip");
    assert_eq!(decoded, sample);
}

#[test]
fn heartbeat_schema_carries_scheduler_queue_stream_identity_budget_and_fixture_state() {
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let metadata = RunMetadata::new(
        SchedulerSnapshot::from_config(&config),
        "exact-uor-matmul",
        "uor-matmul-exact-gemm",
        "aarch64-neon",
    )
    .with_identity("teacher", "blake3:teacher")
    .with_model_geometry("layers", 30)
    .with_budget("positions", 256)
    .with_fixture("teacher", FixtureStatus::available("blake3:teacher"))
    .with_path("run_report", "/tmp/teacher-parity/parity-report.json")
    .with_path(
        "deterministic_evidence",
        "/tmp/teacher-parity/parity-report.evidence.json",
    )
    .with_path(
        "event_jsonl",
        "/tmp/teacher-parity/parity-report.events.jsonl",
    )
    .with_path(
        "exact_probe_report",
        "/tmp/teacher-parity/exact-multicore-probe.json",
    );
    let queue = QueueSnapshot {
        queue_depth: 3,
        active_streams: 2,
        peak_active_streams: 6,
        active_row_workers: 4,
        peak_active_row_workers: 8,
        completed_streams: 4,
        failed_streams: 0,
        active_worker_tasks: 4,
        completed_worker_tasks: 12,
        failed_worker_tasks: 0,
        longest_active_millis: 75,
    };
    let streams = vec![StreamProgress {
        stream_id: "prompt-0/run-0".to_owned(),
        phase: "teacher_generate".to_owned(),
        state: StreamState::Active,
        logical_forwards_completed: 7,
        logical_forwards_total: 16,
        tokens_completed: 7,
        tokens_total: 16,
        active_forward_age_millis: 75,
    }];
    let event = HeartbeatEvent::new(
        "run-932",
        RunStatus::NotRun,
        WorkCounters::new(WorkPlan::default()).snapshot(),
        estimate_eta(EtaInput::default()),
        ResourceAvailability::Unavailable {
            reason: "test sampler".to_owned(),
        },
    )
    .with_event_kind(EventKind::Heartbeat)
    .with_metadata(metadata)
    .with_queue(queue)
    .with_streams(streams)
    .with_phase("teacher_generate")
    .with_elapsed(Duration::from_secs(2));

    let value = serde_json::to_value(&event).expect("event JSON");
    assert_eq!(value["event_kind"], "HEARTBEAT");
    assert_eq!(value["metadata"]["backend"], "exact-uor-matmul");
    assert_eq!(value["metadata"]["model_geometry"]["layers"], 30);
    assert_eq!(
        value["metadata"]["paths"]["run_report"],
        "/tmp/teacher-parity/parity-report.json"
    );
    assert_eq!(
        value["metadata"]["scheduler"]["configured_trajectory_workers"],
        config.workers.get()
    );
    assert_eq!(
        value["metadata"]["scheduler"]["configured_row_workers"],
        config.workers.get()
    );
    assert_eq!(value["queue"]["queue_depth"], 3);
    assert_eq!(value["queue"]["peak_active_streams"], 6);
    assert_eq!(value["queue"]["active_row_workers"], 4);
    assert_eq!(value["queue"]["peak_active_row_workers"], 8);
    assert_eq!(value["streams"][0]["logical_forwards_completed"], 7);
    let human = event.human_summary();
    assert!(human.contains("forwards=0/0"), "{human}");
    assert!(human.contains("queue=3"), "{human}");
    assert!(human.contains("active_streams=2"), "{human}");
    assert!(human.contains("active_row_workers=4"), "{human}");
    assert!(human.contains("exact_live_worker_tasks=12/0"), "{human}");
}

#[test]
fn cadence_evidence_rejects_lifecycle_idle_and_cross_phase_pairs() {
    fn event(
        kind: EventKind,
        phase: &str,
        active_streams: u64,
        active_row_workers: u64,
        timestamp: u64,
    ) -> HeartbeatEvent {
        let mut event = HeartbeatEvent::new(
            "run-cadence",
            RunStatus::NotRun,
            WorkCounters::new(WorkPlan::default()).snapshot(),
            estimate_eta(EtaInput::default()),
            ResourceAvailability::Unavailable {
                reason: "test".to_owned(),
            },
        )
        .with_event_kind(kind)
        .with_phase(phase)
        .with_queue(QueueSnapshot {
            active_streams,
            active_row_workers,
            ..QueueSnapshot::default()
        })
        .with_streams(
            (0..active_streams)
                .map(|lane| StreamProgress {
                    stream_id: format!("lane-{lane}"),
                    phase: "exact_forward".to_owned(),
                    state: StreamState::Active,
                    logical_forwards_completed: 0,
                    logical_forwards_total: 1,
                    tokens_completed: 0,
                    tokens_total: 1,
                    active_forward_age_millis: 0,
                })
                .collect(),
        );
        event.timestamp_unix_millis = timestamp;
        event
    }

    let cadence = Duration::from_millis(100);
    let insufficient = vec![
        event(EventKind::WorkStarted, "active-forward", 8, 8, 1_000),
        event(EventKind::Heartbeat, "active-forward", 8, 8, 1_050),
        event(EventKind::Heartbeat, "loading", 0, 0, 1_100),
        event(EventKind::Heartbeat, "loading", 0, 0, 1_150),
    ];
    assert!(validate_in_flight_heartbeat_cadence(&insufficient, cadence, 8, 8).is_err());

    let compiled_only = vec![
        event(EventKind::Heartbeat, "compiled-decode", 8, 0, 1_050),
        event(EventKind::Heartbeat, "compiled-decode", 8, 0, 1_150),
    ];
    assert!(
        validate_in_flight_heartbeat_cadence(&compiled_only, cadence, 8, 8).is_err(),
        "compiled trajectory activity cannot masquerade as an in-flight exact forward"
    );

    let established = vec![
        event(EventKind::WorkStarted, "active-forward", 8, 8, 1_000),
        event(EventKind::Heartbeat, "active-forward", 8, 3, 1_050),
        event(EventKind::Heartbeat, "active-forward", 8, 5, 1_150),
    ];
    validate_in_flight_heartbeat_cadence(&established, cadence, 8, 8)
        .expect("two same-phase bounded active periodic rows establish cadence");
}

#[test]
fn binding_heartbeat_requires_distinct_full_width_streams_with_bounded_worker_diagnostics() {
    fn heartbeat(stream_ids: &[&str], active_streams: u64, row_workers: u64) -> HeartbeatEvent {
        HeartbeatEvent::new(
            "run-full-width",
            RunStatus::NotRun,
            WorkCounters::new(WorkPlan::default()).snapshot(),
            estimate_eta(EtaInput::default()),
            ResourceAvailability::Unavailable {
                reason: "test sampler".to_owned(),
            },
        )
        .with_event_kind(EventKind::Heartbeat)
        .with_queue(QueueSnapshot {
            active_streams,
            active_row_workers: row_workers,
            ..QueueSnapshot::default()
        })
        .with_streams(
            stream_ids
                .iter()
                .map(|stream_id| StreamProgress {
                    stream_id: (*stream_id).to_owned(),
                    phase: "exact_forward".to_owned(),
                    state: StreamState::Active,
                    logical_forwards_completed: 0,
                    logical_forwards_total: 1,
                    tokens_completed: 0,
                    tokens_total: 1,
                    active_forward_age_millis: 25,
                })
                .collect(),
        )
        .with_phase("exact_forward")
    }

    let distinct = (0..8)
        .map(|lane| format!("lane-{lane}"))
        .collect::<Vec<_>>();
    let distinct_refs = distinct.iter().map(String::as_str).collect::<Vec<_>>();
    let full_width = vec![heartbeat(&distinct_refs, 8, 4)];
    validate_full_width_exact_heartbeat(&full_width, 8, 8)
        .expect("one durable row binds eight private streams to bounded row activity");

    let duplicate = vec!["lane-0"; 8];
    assert!(validate_full_width_exact_heartbeat(&[heartbeat(&duplicate, 8, 8)], 8, 8).is_err());
    validate_full_width_exact_heartbeat(&[heartbeat(&distinct_refs, 8, 7)], 8, 8)
        .expect("worker saturation is diagnostic rather than binding");
    assert!(validate_full_width_exact_heartbeat(&[heartbeat(&distinct_refs, 8, 0)], 8, 8).is_err());
    assert!(validate_full_width_exact_heartbeat(&[heartbeat(&distinct_refs, 8, 9)], 8, 8).is_err());
    assert!(validate_full_width_exact_heartbeat(&[heartbeat(&distinct_refs, 7, 8)], 8, 8).is_err());
}

#[test]
fn cancelled_worker_start_gate_releases_a_partial_spawn_cohort() {
    let gate = Arc::new(CancellableStartGate::new());
    let waiting_gate = Arc::clone(&gate);
    let worker = thread::spawn(move || waiting_gate.wait());
    thread::sleep(Duration::from_millis(10));
    gate.cancel("planted later worker creation failure");
    let reason = worker
        .join()
        .expect("partial worker exits without panic")
        .expect_err("partial worker is cancelled instead of entering a fixed barrier");
    assert!(reason.contains("planted later worker creation failure"));
}

#[test]
fn independent_heartbeat_worker_flushes_while_a_forward_is_still_in_flight() {
    let dir = unique_dir("heartbeat-worker");
    fs::create_dir_all(&dir).expect("create heartbeat worker output");
    let path = dir.join("parity-report.events.jsonl");
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let metadata = RunMetadata::new(
        SchedulerSnapshot::from_config(&config),
        "exact",
        "exact-gemm",
        "test-isa",
    );
    let progress = SharedProgress::new("run-932-worker", metadata);
    progress
        .update(|state| {
            state.phase = "long_exact_forward".to_owned();
            state.queue.queue_depth = 0;
            state.queue.active_streams = 2;
            state.queue.peak_active_streams = 2;
            state.queue.active_row_workers = 2;
            state.queue.peak_active_row_workers = 2;
            state.streams = (0..2)
                .map(|lane| StreamProgress {
                    stream_id: format!("stream-{lane}"),
                    phase: "long_exact_forward".to_owned(),
                    state: StreamState::Active,
                    logical_forwards_completed: 0,
                    logical_forwards_total: 1,
                    tokens_completed: 0,
                    tokens_total: 1,
                    active_forward_age_millis: 0,
                })
                .collect();
        })
        .expect("seed live progress");
    progress.publish_exact(ExactProgressObservation {
        observer_epoch: 1,
        streams_started: 2,
        streams_completed: 0,
        active_streams: 2,
        peak_active_streams: 2,
        active_row_workers: 2,
        peak_active_row_workers: 2,
        matrix_calls: 0,
        batched_matrix_calls: 0,
        max_matrix_batch_width: 0,
        completed_worker_tasks: 0,
        output_cells_completed: 0,
        scalar_terms_completed: 0,
        effective_workers: 2,
    });
    let counters = Arc::new(WorkCounters::new(WorkPlan {
        logical_forwards: 2,
        tokens: 2,
        physical_batches: 1,
        padded_forwards: 0,
        cache_hits: 0,
        streams: 2,
        worker_tasks: 2,
        row_tiles: 2,
        ..WorkPlan::default()
    }));
    let worker = HeartbeatWorker::spawn(
        &path,
        Duration::from_millis(50),
        Arc::clone(&counters),
        progress.clone(),
    )
    .expect("spawn heartbeat worker");
    worker
        .emit(EventKind::WorkStarted)
        .expect("durably emit lifecycle event");

    thread::sleep(Duration::from_millis(320));
    let in_flight = fs::read_to_string(&path).expect("read in-flight heartbeat stream");
    let in_flight_events: Vec<HeartbeatEvent> = in_flight
        .lines()
        .map(|line| serde_json::from_str(line).expect("heartbeat JSON line"))
        .collect();
    let periodic_heartbeats = in_flight_events
        .iter()
        .filter(|event| event.event_kind == EventKind::Heartbeat)
        .count();
    assert!(
        periodic_heartbeats >= 2,
        "expected repeated periodic HEARTBEAT rows, got {periodic_heartbeats}"
    );
    assert!(
        in_flight_events
            .iter()
            .all(|event| event.work.logical_forwards == 0),
        "the forward must still be visibly in flight"
    );
    validate_in_flight_heartbeat_cadence(&in_flight_events, Duration::from_millis(50), 2, 2)
        .expect("same active forward receives repeated cadence heartbeats");

    counters.record_batch(2, 0, 0, 2, 2);
    counters.record_tokens(2);
    counters.record_stream_completed();
    counters.record_stream_completed();
    progress.finish_exact_forward();
    progress
        .update(|state| {
            state.status = RunStatus::Pass;
            state.queue.queue_depth = 0;
            state.queue.active_streams = 0;
            state.queue.active_row_workers = 0;
            state.queue.completed_streams = 2;
            for stream in &mut state.streams {
                stream.state = StreamState::Completed;
                stream.logical_forwards_completed = 1;
                stream.tokens_completed = 1;
            }
        })
        .expect("complete live progress");
    progress
        .update(|state| state.streams.clear())
        .expect("clear completed stream rows");
    worker
        .emit(EventKind::WorkCompleted)
        .expect("emit completion after stream rows are cleared");
    let completed_events = fs::read_to_string(&path).expect("read completion event");
    let completed: HeartbeatEvent = serde_json::from_str(
        completed_events
            .lines()
            .last()
            .expect("completion JSONL row"),
    )
    .expect("completion event JSON");
    assert_eq!(completed.work.tokens, 2);
    #[cfg(target_os = "macos")]
    assert!(worker.max_sampled_rss_bytes().is_some());
    worker
        .emit_and_stop(EventKind::SuiteCompleted)
        .expect("terminal append and writer stop are one ordered operation");
    let terminal_events = fs::read_to_string(&path).expect("read terminal event stream");
    let terminal: HeartbeatEvent =
        serde_json::from_str(terminal_events.lines().last().expect("terminal JSONL row"))
            .expect("terminal event JSON");
    assert_eq!(terminal.event_kind, EventKind::SuiteCompleted);
    thread::sleep(Duration::from_millis(150));
    assert_eq!(
        fs::read_to_string(&path).expect("read stopped terminal event stream"),
        terminal_events,
        "no periodic progress may be appended after a terminal event"
    );
    fs::remove_dir_all(&dir).expect("remove heartbeat worker output");
}

#[test]
fn advancing_inflight_exact_work_updates_eta_and_never_false_stalls() {
    let dir = unique_dir("inflight-exact-eta");
    fs::create_dir_all(&dir).expect("create in-flight ETA output");
    let path = dir.join("parity-report.events.jsonl");
    let config = ParityConfig::from_lookup(|_| None).expect("default config");
    let progress = SharedProgress::new(
        "run-932-inflight-eta",
        RunMetadata::new(
            SchedulerSnapshot::from_config(&config),
            "exact",
            "exact-gemm",
            "test-isa",
        ),
    );
    progress
        .update(|state| {
            state.phase = "long_exact_forward".to_owned();
            state.queue.active_streams = 8;
            state.streams = (0..8)
                .map(|lane| StreamProgress {
                    stream_id: format!("stream-{lane}"),
                    phase: "long_exact_forward".to_owned(),
                    state: StreamState::Active,
                    logical_forwards_completed: 0,
                    logical_forwards_total: 1,
                    tokens_completed: 0,
                    tokens_total: 1,
                    active_forward_age_millis: 0,
                })
                .collect();
        })
        .expect("seed exact in-flight state");
    let counters = Arc::new(WorkCounters::new(WorkPlan {
        logical_forwards: 8,
        tokens: 8,
        physical_batches: 1,
        matrix_calls: 20,
        batched_matrix_calls: 20,
        max_matrix_batch_width: 8,
        worker_tasks: 100,
        row_tiles: 100,
        output_cells: 10_000,
        scalar_terms: 1_000_000,
        streams: 8,
        ..WorkPlan::default()
    }));
    let worker = HeartbeatWorker::spawn_with_stall_after(
        &path,
        Duration::from_secs(3_600),
        Duration::from_millis(20),
        Arc::clone(&counters),
        progress.clone(),
    )
    .expect("spawn planted-stall heartbeat worker");
    thread::sleep(Duration::from_millis(30));
    progress.publish_exact(ExactProgressObservation {
        observer_epoch: 1,
        streams_started: 8,
        streams_completed: 0,
        active_streams: 8,
        peak_active_streams: 8,
        active_row_workers: 4,
        peak_active_row_workers: 4,
        matrix_calls: 3,
        batched_matrix_calls: 3,
        max_matrix_batch_width: 8,
        completed_worker_tasks: 5,
        output_cells_completed: 500,
        scalar_terms_completed: 50_000,
        effective_workers: 8,
    });
    worker
        .emit(EventKind::Heartbeat)
        .expect("publish advancing exact work after planted stall age");
    let rows = fs::read_to_string(&path).expect("read in-flight ETA event");
    let event: HeartbeatEvent = serde_json::from_str(rows.lines().last().expect("heartbeat row"))
        .expect("parse heartbeat row");
    assert_eq!(event.work.logical_forwards, 0);
    assert_eq!(event.work.matrix_calls, 3);
    assert_eq!(event.work.batched_matrix_calls, 3);
    assert_eq!(event.work.max_matrix_batch_width, 8);
    assert_eq!(event.work.worker_tasks, 5);
    assert_eq!(event.work.row_tiles, 5);
    assert_eq!(event.work.output_cells, 500);
    assert_eq!(event.work.scalar_terms, 50_000);
    assert_eq!(event.queue.completed_worker_tasks, 5);
    assert_eq!(event.rates.eta_progress_unit, ProgressUnit::ScalarTerms);
    assert_eq!(event.rates.eta_progress_completed, 50_000);
    assert_eq!(event.rates.eta_progress_total, 1_000_000);
    assert_eq!(
        heartbeat_progress_units(&event.work),
        (ProgressUnit::ScalarTerms, 50_000, 1_000_000)
    );
    assert_eq!(event.eta.status, EtaStatus::Estimated);
    let human = event.human_summary();
    assert!(human.contains("worker_tasks=5/100"), "{human}");
    assert!(human.contains("scalar_terms=50000/1000000"), "{human}");
    assert!(
        human.contains("eta_progress=ScalarTerms:50000/1000000"),
        "{human}"
    );
    worker.stop().expect("stop planted-stall heartbeat worker");
    fs::remove_dir_all(&dir).expect("remove in-flight ETA output");
}

#[test]
fn disabled_mode_is_parseable_for_negative_evidence_but_not_a_binding_run() {
    let workers = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1)
        .min(4);
    let streams = workers.max(8);
    let worker_text = workers.to_string();
    let stream_text = streams.to_string();
    let config = ParityConfig::from_lookup(|name| match name {
        "R4_PARITY_WORKERS" => Some(worker_text.clone()),
        "R4_PARITY_STREAMS" => Some(stream_text.clone()),
        "R4_PARITY_BATCH_PER_WORKER" => Some("4".to_owned()),
        "R4_PARITY_TELEMETRY" => Some("0".to_owned()),
        "R4_PARITY_PROGRESS_EVERY_SECS" => Some("5".to_owned()),
        "R4_PARITY_MAX_WALL_SECS" => Some("600".to_owned()),
        "R4_PARITY_REPORT" => Some("target/custom-parity.json".to_owned()),
        _ => None,
    })
    .expect("explicit config");

    assert_eq!(config.workers, nonzero(workers));
    assert_eq!(config.streams, nonzero(streams));
    assert_eq!(config.batch_per_worker, nonzero(4));
    assert_eq!(config.mode, ObservabilityMode::Disabled);
    assert_eq!(config.progress_every, NonZeroU64::new(5).unwrap());
    assert_eq!(config.stall_after, NonZeroU64::new(120).unwrap());
    assert_eq!(config.max_wall, NonZeroU64::new(600).unwrap());
    assert_eq!(
        config.report_path,
        PathBuf::from("target/custom-parity.json")
    );
}
