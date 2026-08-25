//! #933 deployed-quality schema and fail-closed production admission.

use uor_r4_api::{
    parse_deployed_quality_for_research, ActiveSectionIdentity, ActiveSectionSetIdentity,
    ArtifactIdentity, ComparatorIdentity, CompilerIdentity, CorpusIdentity, DecodeIdentity,
    DecodeMode, DeployedQualityBindings, DeployedQualityReport, DeployedQualityValidationError,
    EvaluationEvidence, EvaluationMode, ExactRate, ExactSignedRate, NegativeControlEvidence,
    NegativeControlVerdict, PairedComparison, PairedCounts, PairedInterval, PartitionIdentity,
    PositionSelectionMode, QualityMeasurements, QualityProfileIdentity, QualityTokenizerIdentity,
    QualityVerdict, ResearchDeployedQualityReport, SeedIdentity, SelectorIdentity,
    WitnessReplayEvidence, DEPLOYED_QUALITY_PROFILE_ID, DEPLOYED_QUALITY_PROFILE_VERSION,
    DEPLOYED_QUALITY_REPORT_SCHEMA, LABEL_SHUFFLED_CONTROL_ID, NORMATIVE_EXECUTION_SCOPE,
    NORMATIVE_SELECTOR_ID, SECTIONS_ABSENT_COMPARATOR_ID, SECTIONS_ABSENT_COMPARATOR_VERSION,
    TLA_COMPARATOR_ID, TLA_COMPARATOR_VERSION,
};

const N: u64 = 1_000;

fn cid(value: u8) -> String {
    format!("blake3:{value:064x}")
}

fn rate(numerator: u64, denominator: u64) -> ExactRate {
    ExactRate {
        numerator,
        denominator,
        ppm: ((u128::from(numerator) * 1_000_000) / u128::from(denominator)) as u32,
    }
}

fn signed_rate(numerator: i64, denominator: u64) -> ExactSignedRate {
    ExactSignedRate {
        numerator,
        denominator,
        ppm: ((i128::from(numerator) * 1_000_000) / i128::from(denominator)) as i64,
    }
}

fn comparison(
    id: &str,
    version: &str,
    both_correct: u64,
    selector_only_correct: u64,
    comparator_only_correct: u64,
    neither_correct: u64,
) -> PairedComparison {
    let selector_hits = both_correct + selector_only_correct;
    let comparator_hits = both_correct + comparator_only_correct;
    let delta_numerator = selector_only_correct as i64 - comparator_only_correct as i64;
    let delta = signed_rate(delta_numerator, N);
    let counts = PairedCounts {
        both_correct,
        selector_only_correct,
        comparator_only_correct,
        neither_correct,
    };
    PairedComparison {
        comparator: ComparatorIdentity {
            id: id.to_string(),
            version: version.to_string(),
            definition_cid: cid(20),
            positions_cid: cid(8),
        },
        counts,
        selector_rate: rate(selector_hits, N),
        comparator_rate: rate(comparator_hits, N),
        delta,
        interval: PairedInterval::from_counts(counts).expect("bounded paired interval"),
    }
}

fn bindings() -> DeployedQualityBindings {
    DeployedQualityBindings {
        selector: SelectorIdentity {
            id: NORMATIVE_SELECTOR_ID.to_string(),
            semantics_version: "1.0.0".to_string(),
            semantics_cid: cid(1),
        },
        graph: ArtifactIdentity {
            bytes_cid: cid(2),
            artifact_kappa: cid(3),
        },
        teacher_artifact: ArtifactIdentity {
            bytes_cid: cid(4),
            artifact_kappa: cid(5),
        },
        corpus: CorpusIdentity {
            meta_cid: cid(6),
            records_cid: cid(7),
            stream_cid: cid(8),
        },
        partition: PartitionIdentity {
            manifest_cid: cid(9),
            construction_cid: cid(10),
            certification_cid: cid(11),
            evaluated_positions_cid: cid(8),
            split_version: "story-disjoint-80-20/1".to_string(),
        },
        tokenizer: QualityTokenizerIdentity {
            bytes_cid: cid(12),
            adapter_id: "hf-byte-bpe".to_string(),
            adapter_version: "1".to_string(),
            adapter_config_cid: cid(13),
        },
        compiler: CompilerIdentity {
            revision: "0123456789abcdef0123456789abcdef01234567".to_string(),
            configuration_cid: cid(14),
        },
        serving_configuration_cid: cid(15),
        active_sections: ActiveSectionSetIdentity {
            set_cid: cid(16),
            sections: vec![
                ActiveSectionIdentity {
                    id: "EXCT".to_string(),
                    cid: cid(17),
                },
                ActiveSectionIdentity {
                    id: "PSIB".to_string(),
                    cid: cid(18),
                },
                ActiveSectionIdentity {
                    id: "SKMX".to_string(),
                    cid: cid(19),
                },
            ],
        },
        decode: DecodeIdentity {
            mode: DecodeMode::GreedyTop1,
            implementation: "normative-ranked-candidate-argmax/1".to_string(),
            configuration_cid: cid(21),
        },
        seed: SeedIdentity {
            mode: PositionSelectionMode::FullPopulation,
            algorithm: "ascending-certification-position/1".to_string(),
            seed: 0,
            selection_cid: cid(8),
        },
    }
}

fn valid_report() -> DeployedQualityReport {
    DeployedQualityReport {
        schema: DEPLOYED_QUALITY_REPORT_SCHEMA,
        profile: QualityProfileIdentity {
            id: DEPLOYED_QUALITY_PROFILE_ID.to_string(),
            version: DEPLOYED_QUALITY_PROFILE_VERSION,
            execution_scope: NORMATIVE_EXECUTION_SCOPE.to_string(),
        },
        bindings: bindings(),
        evaluation: EvaluationEvidence {
            mode: EvaluationMode::FullCensus,
            population_size: N,
            evaluated_positions: N,
            verdict: QualityVerdict::Pass,
            measurements: Some(QualityMeasurements {
                // Selector 350/1000, TLA 300/1000, positive paired lower bound.
                versus_tla: comparison(
                    TLA_COMPARATOR_ID,
                    TLA_COMPARATOR_VERSION,
                    250,
                    100,
                    50,
                    600,
                ),
                // Same selector 350/1000, base 280/1000, lower bound > +20permille.
                versus_sections_absent: comparison(
                    SECTIONS_ABSENT_COMPARATOR_ID,
                    SECTIONS_ABSENT_COMPARATOR_VERSION,
                    250,
                    100,
                    30,
                    620,
                ),
                internal_base_control_checks: N,
                internal_base_control_mismatches: 0,
                cross_surface_checks: N + 6,
                cross_surface_mismatches: 0,
                cross_surface_evidence_cid: cid(24),
            }),
        },
        witness_replay: WitnessReplayEvidence {
            sample_cid: cid(22),
            requested: 64,
            replayed: 64,
            failures: 0,
        },
        negative_controls: vec![NegativeControlEvidence {
            id: LABEL_SHUFFLED_CONTROL_ID.to_string(),
            identity_cid: cid(23),
            verdict: NegativeControlVerdict::Passed,
            // Shuffled labels have a negative paired effect (-10permille).
            comparison: Some(comparison(
                SECTIONS_ABSENT_COMPARATOR_ID,
                SECTIONS_ABSENT_COMPARATOR_VERSION,
                250,
                20,
                30,
                700,
            )),
        }],
    }
}

#[test]
fn full_census_pass_with_matching_loaded_identities_is_production_valid() {
    let report = valid_report();
    assert_eq!(report.validate_for_research(), None);
    assert_eq!(
        report.validate_for_production(&report.bindings.clone()),
        None
    );
}

#[test]
fn deterministic_serialization_and_cid_bind_exact_bytes() {
    let report = valid_report();
    let first = report.deterministic_json_bytes().expect("serialize");
    let second = report.deterministic_json_bytes().expect("serialize again");
    assert_eq!(first, second);
    assert_eq!(first.last(), Some(&b'\n'));
    assert_eq!(
        report.cid().expect("report cid"),
        format!("blake3:{}", blake3::hash(&first).to_hex())
    );
    let parsed: DeployedQualityReport = serde_json::from_slice(&first).expect("round trip");
    assert_eq!(parsed, report);
}

#[test]
fn unknown_top_level_and_nested_fields_are_hard_parse_errors() {
    let mut top = serde_json::to_value(valid_report()).expect("value");
    top.as_object_mut()
        .expect("object")
        .insert("surprise".to_string(), serde_json::json!(true));
    assert!(serde_json::from_value::<DeployedQualityReport>(top).is_err());

    let mut nested = serde_json::to_value(valid_report()).expect("value");
    nested["bindings"]["graph"]
        .as_object_mut()
        .expect("graph object")
        .insert("surprise".to_string(), serde_json::json!(true));
    assert!(serde_json::from_value::<DeployedQualityReport>(nested).is_err());
}

#[test]
fn research_valid_sample_is_never_production_admissible() {
    let mut report = valid_report();
    report.evaluation.mode = EvaluationMode::Sample;
    report.evaluation.population_size = 1_200;
    report.evaluation.verdict = QualityVerdict::Estimate {
        decision: "proceed to census".to_string(),
    };
    report.bindings.seed.mode = PositionSelectionMode::DeterministicSample;
    let loaded = report.bindings.clone();
    assert_eq!(report.validate_for_research(), None);
    let error = report
        .validate_for_production(&loaded)
        .expect("sample rejected");
    assert!(matches!(
        error,
        DeployedQualityValidationError::NotProductionAdmissible { .. }
    ));
}

#[test]
fn every_loaded_identity_is_bound_and_a_graph_mismatch_fails_closed() {
    let report = valid_report();
    let mut loaded = report.bindings.clone();
    loaded.graph.bytes_cid = cid(99);
    let error = report
        .validate_for_production(&loaded)
        .expect("mismatch rejected");
    assert!(matches!(
        error,
        DeployedQualityValidationError::IdentityMismatch { field: "graph", .. }
    ));
}

#[test]
fn unavailable_evidence_is_research_readable_but_rejected_for_production() {
    let mut report = valid_report();
    report.evaluation.verdict = QualityVerdict::Unavailable {
        reason: "canonical fixture missing".to_string(),
    };
    report.evaluation.evaluated_positions = 0;
    report.evaluation.measurements = None;
    report.witness_replay.requested = 0;
    report.witness_replay.replayed = 0;
    report.negative_controls.clear();
    let loaded = report.bindings.clone();
    assert_eq!(report.validate_for_research(), None);
    assert!(report.validate_for_production(&loaded).is_some());
}

#[test]
fn off_serving_selector_is_research_readable_but_rejected_for_production() {
    let mut report = valid_report();
    report.bindings.selector.id = "GraphScorer".to_string();
    let loaded = report.bindings.clone();
    assert_eq!(report.validate_for_research(), None);
    let error = report
        .validate_for_production(&loaded)
        .expect("wrong selector rejected");
    assert!(error.to_string().contains(NORMATIVE_SELECTOR_ID));
}

#[test]
fn paired_counts_rates_and_intervals_are_cross_checked() {
    let mut report = valid_report();
    report
        .evaluation
        .measurements
        .as_mut()
        .expect("measurements")
        .versus_tla
        .counts
        .neither_correct += 1;
    let error = report
        .validate_for_research()
        .expect("inconsistent paired census rejected");
    assert!(error.to_string().contains("paired counts total"));
}

#[test]
fn paired_interval_bounds_and_method_are_recomputed_not_trusted() {
    let mut tampered_bound = valid_report();
    tampered_bound
        .evaluation
        .measurements
        .as_mut()
        .expect("measurements")
        .versus_tla
        .interval
        .lower_delta_ppm -= 1;
    let error = tampered_bound
        .validate_for_research()
        .expect("tampered interval rejected");
    assert!(error.to_string().contains("recomputed"));

    let mut tampered_method = valid_report();
    tampered_method
        .evaluation
        .measurements
        .as_mut()
        .expect("measurements")
        .versus_tla
        .interval
        .method = "claimed-by-producer".to_string();
    assert!(tampered_method.validate_for_research().is_some());
}

#[test]
fn witness_failure_and_unavailable_control_each_block_production() {
    let mut witness_failure = valid_report();
    witness_failure.witness_replay.failures = 1;
    assert!(witness_failure
        .validate_for_production(&witness_failure.bindings.clone())
        .is_some());

    let mut unavailable_control = valid_report();
    unavailable_control.negative_controls[0].verdict = NegativeControlVerdict::Unavailable;
    assert!(unavailable_control
        .validate_for_production(&unavailable_control.bindings.clone())
        .is_some());
}

#[test]
fn internal_absent_identity_must_cover_every_production_position_exactly() {
    let mut missing = valid_report();
    let missing_measurements = missing
        .evaluation
        .measurements
        .as_mut()
        .expect("measurements");
    missing_measurements.internal_base_control_checks = 0;
    missing_measurements.cross_surface_checks = 6;
    assert_eq!(missing.validate_for_research(), None);
    let error = missing
        .validate_for_production(&missing.bindings.clone())
        .expect("external parity cannot replace the internal absent census");
    assert!(error.to_string().contains("internal sections-absent"));

    let mut divergent = valid_report();
    let divergent_measurements = divergent
        .evaluation
        .measurements
        .as_mut()
        .expect("measurements");
    divergent_measurements.internal_base_control_mismatches = 1;
    divergent_measurements.cross_surface_mismatches = 1;
    assert_eq!(divergent.validate_for_research(), None);
    let error = divergent
        .validate_for_production(&divergent.bindings.clone())
        .expect("one absent-identity divergence must fail closed");
    assert!(error.to_string().contains("internal sections-absent"));
}

#[test]
fn internal_absent_identity_fields_are_required_by_the_report_schema() {
    for field in [
        "internal_base_control_checks",
        "internal_base_control_mismatches",
    ] {
        let mut value = serde_json::to_value(valid_report()).expect("report value");
        value["evaluation"]["measurements"]
            .as_object_mut()
            .expect("measurements object")
            .remove(field)
            .expect("required field exists");
        assert!(
            serde_json::from_value::<DeployedQualityReport>(value).is_err(),
            "missing {field} must not default to vacuous evidence"
        );
    }
}

#[test]
fn legacy_gate_c_json_is_only_available_through_explicit_research_parse() {
    let bytes = br#"{"schema":26,"gate_c":{"rule12_precedence":{}}}"#;
    let parsed = parse_deployed_quality_for_research(bytes).expect("research parse");
    assert!(matches!(
        parsed,
        ResearchDeployedQualityReport::LegacyUnavailable {
            declared_schema: Some(26),
            ..
        }
    ));
    assert!(serde_json::from_slice::<DeployedQualityReport>(bytes).is_err());
}

#[test]
fn explicit_research_parse_preserves_a_current_typed_report() {
    let report = valid_report();
    let bytes = report.deterministic_json_bytes().expect("bytes");
    let parsed = parse_deployed_quality_for_research(&bytes).expect("research parse");
    assert!(matches!(
        parsed,
        ResearchDeployedQualityReport::Current(current) if *current == report
    ));
}
