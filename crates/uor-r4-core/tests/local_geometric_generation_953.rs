use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, CanonicalLexicalCodec, CanonicalRouteArtifact, ConversationInput,
    ParagraphInput, TurnInput,
};
use uor_r4_core::local_geometric_generation::{
    LocalGenerationControl, LocalGenerationRowSource, LocalGenerationSourceCounts,
    LocalGenerationStepTrace, LocalGenerationStopReason, LocalGeometricGenerationReport,
    LocalGeometricGenerator,
};
use uor_r4_core::prime_route_geometric_attention::{H4S3AngularShell, PathLeaseCost};

const REGISTERED_UNITS: [&str; 10] = [
    "active",
    "agile",
    "alert",
    "athletes",
    "brave",
    "carefully",
    "run",
    "slowly",
    "train",
    "walk",
];
const CONSTRUCTION_SENTENCES: [&[&str]; 7] = [
    &["train", "carefully"],
    &["walk", "slowly"],
    &["active"],
    &["agile"],
    &["alert"],
    &["athletes"],
    &["run"],
];
const LEFT_PROMPT: &[u8] = b"active agile athletes run";
const RIGHT_PROMPT: &[u8] = b"agile active athletes run";
const CONTINUATION_CAP: usize = 2;
const REVISION_TERMINAL: &str = "REVISE_I1_GENERATOR_IN_PLACE";
const SMOKE_FIXTURE_IDENTITY_BYTES: &[u8] = b"uor-r4.local-geometric-generation-smoke/1\nconstruction=train carefully|walk slowly|active|agile|alert|athletes|run;global=brave\nprompts=active agile athletes run|agile active athletes run\ncontrols=full_path|state_disabled\ncontinuation_cap=2\ntermination=cap\ncycle=period-1-through-4-requires-three-equal-trailing-periods";

// Frozen before the generator smoke was first executed. The terminal audit
// subsequently retained this identity as revision evidence, not as a positive
// natural-language fixture.
const SMOKE_FIXTURE_KAPPA: &str =
    "blake3:b6e1f5a0665fb4e8a329c1b769697ced0b0f7b16ff20ff5a9288b279b1880409";
const CONSTRUCTION_ARTIFACT_KAPPA: &str =
    "blake3:411f091f9455dd401711861db6db534482780f4b07645454c6bc1579072cc0ad";
const ATTENTION_MANIFEST_KAPPA: &str =
    "blake3:55465770d59b8e27cc232e09511c59654b4c93acd074ee3f26652e4a03eb76d2";
const CODEC_KAPPA: &str = "blake3:71aa5e35465be4da1847bbfdbb7a836a4a21194f289fd638b79b4bfe576c8c09";
const SMOKE_RECORD_KAPPA: &str =
    "blake3:f8738ae16585b5817108ad6c8bc1ec7aee93f9d5a6cacffaa3aa084bb643cf72";

fn conversation_input(scope: &str, sentences: &[&[&str]]) -> ConversationInput {
    let global_snapshot_units = vec![b"brave".to_vec()];
    ConversationInput {
        identity_scope: scope.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units).unwrap(),
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: "turn-0001".to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: sentences
                    .iter()
                    .map(|sentence| sentence.join(" ").into_bytes())
                    .collect(),
            }],
        }],
    }
}

fn registered_input() -> ConversationInput {
    let owned = REGISTERED_UNITS
        .iter()
        .map(|unit| vec![*unit])
        .collect::<Vec<_>>();
    let sentences = owned.iter().map(Vec::as_slice).collect::<Vec<_>>();
    conversation_input("issue-953/natural-registration-v1", &sentences)
}

fn frozen_artifact() -> CanonicalRouteArtifact {
    let codec = CanonicalLexicalCodec::compile(&registered_input()).unwrap();
    CanonicalRouteArtifact::ingest(
        &codec,
        &conversation_input("issue-953/natural-construction-v1", &CONSTRUCTION_SENTENCES),
    )
    .unwrap()
}

fn fixture_kappa() -> String {
    format!(
        "blake3:{}",
        blake3::hash(SMOKE_FIXTURE_IDENTITY_BYTES).to_hex()
    )
}

#[test]
fn freeze_relabel_fixture_identity_before_generation() {
    assert!(REGISTERED_UNITS.windows(2).all(|pair| pair[0] < pair[1]));
    let rank = |surface: &str| REGISTERED_UNITS.binary_search(&surface).unwrap();
    assert_eq!(
        CONSTRUCTION_SENTENCES
            .iter()
            .map(|sentence| sentence.iter().map(|unit| rank(unit)).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec![
            vec![8, 5],
            vec![9, 7],
            vec![0],
            vec![1],
            vec![2],
            vec![3],
            vec![6],
        ],
        "the construction is rank-isomorphic to #969's symbolic fixture"
    );
    assert_eq!(
        ["active", "agile", "athletes", "run"].map(rank),
        [0, 1, 3, 6]
    );
    assert_eq!(
        ["agile", "active", "athletes", "run"].map(rank),
        [1, 0, 3, 6]
    );
    let artifact = frozen_artifact();
    assert_eq!(fixture_kappa(), SMOKE_FIXTURE_KAPPA);
    assert_eq!(artifact.manifest_kappa(), CONSTRUCTION_ARTIFACT_KAPPA);
    assert_eq!(
        artifact.embedded_spin_manifest_kappa(),
        ATTENTION_MANIFEST_KAPPA
    );
    assert_eq!(artifact.codec_kappa(), CODEC_KAPPA);
}

fn support_signature(step: &LocalGenerationStepTrace) -> Vec<(Vec<u8>, [u32; 5])> {
    let mut signature = step
        .candidates
        .iter()
        .map(|candidate| {
            let LocalGenerationSourceCounts {
                last_one,
                last_two,
                ordered_sentence,
                divisor,
                adjacent_spin,
            } = candidate.source_counts;
            (
                candidate.payload_bytes.clone(),
                [last_one, last_two, ordered_sentence, divisor, adjacent_spin],
            )
        })
        .collect::<Vec<_>>();
    signature.sort();
    signature
}

fn emitted_payloads(report: &LocalGeometricGenerationReport) -> Vec<Vec<u8>> {
    report
        .steps
        .iter()
        .filter_map(|step| {
            step.selected
                .as_ref()
                .map(|selected| selected.payload_bytes.clone())
        })
        .collect()
}

fn cost_signature(step: &LocalGenerationStepTrace) -> Vec<(Vec<u8>, PathLeaseCost)> {
    step.candidates
        .iter()
        .map(|candidate| (candidate.payload_bytes.clone(), candidate.cost))
        .collect()
}

fn assert_exact_inversion(
    artifact: &CanonicalRouteArtifact,
    report: &LocalGeometricGenerationReport,
) {
    for step in &report.steps {
        for candidate in &step.candidates {
            let address = artifact
                .lexical_route_address(candidate.lexical_unit_id)
                .unwrap()
                .unwrap();
            let value = artifact
                .lexical_route_value_for_address(&address)
                .unwrap()
                .unwrap();
            assert_eq!(value.address_kappa, candidate.address_kappa);
            assert_eq!(value.payload_cid, candidate.payload_cid);
            assert_eq!(value.payload_bytes, candidate.payload_bytes);
        }
        if let Some(selected) = &step.selected {
            let address = artifact
                .lexical_route_address(selected.lexical_unit_id)
                .unwrap()
                .unwrap();
            let value = artifact
                .lexical_route_value_for_address(&address)
                .unwrap()
                .unwrap();
            assert_eq!(value.address_kappa, selected.address_kappa);
            assert_eq!(value.payload_cid, selected.payload_cid);
            assert_eq!(value.payload_bytes, selected.payload_bytes);
        }
    }
}

fn assert_bounded_source_free_report(report: &LocalGeometricGenerationReport) {
    assert_eq!(report.prompt_routes.len(), 4);
    assert_eq!(report.steps.len(), 2);
    assert_eq!(report.emitted_lexical_unit_ids.len(), 2);
    assert_eq!(report.prompt_routes.len() + report.steps.len(), 6);
    assert_eq!(report.continuation_cap, CONTINUATION_CAP);
    assert_eq!(
        report.stop_reason,
        LocalGenerationStopReason::ContinuationCap
    );
    assert_eq!(report.detected_cycle_period, None);
    assert!(report.source_boundary.artifact_provenance_validated);
    assert!(report.source_boundary.artifact_input_reconstructed);
    assert!(report.source_boundary.schema2_rebuild_witnesses_compiled);
    assert_eq!(report.source_boundary.source_weight_reads, 0);
    assert_eq!(report.source_boundary.teacher_forwards, 0);
    assert_eq!(report.source_boundary.provider_calls, 0);
    assert_eq!(report.source_boundary.source_attention_calls, 0);
    assert_eq!(report.source_boundary.learned_router_calls, 0);
    assert_eq!(report.source_boundary.dense_matrix_operations, 0);
    assert_eq!(report.source_boundary.selection_future_event_reads, 0);
    assert_eq!(
        report
            .source_boundary
            .selection_paragraph_conversation_global_reads,
        0
    );

    for (step_index, step) in report.steps.iter().enumerate() {
        assert_eq!(step.support_rows.len(), 7);
        assert_eq!(step.candidate_entries_examined, 2);
        assert_eq!(step.unique_candidates_before_ceiling, 2);
        assert_eq!(step.memory_keys_per_candidate, 4 + step_index);
        assert_eq!(step.path_geometry_evaluations, 8 + 2 * step_index);
        assert_eq!(step.observed_routes_before, 4 + step_index);
        assert_eq!(step.observed_routes_after, 5 + step_index);
        assert_eq!(
            step.selected.as_ref().unwrap().observed_routes_after_append,
            step.observed_routes_after
        );
        assert_eq!(step.detected_cycle_period, None);
        assert!(!step.abstained);
        assert!(!step.tie);
        for row in &step.support_rows {
            if matches!(
                row.source,
                LocalGenerationRowSource::LastOne
                    | LocalGenerationRowSource::LastTwo
                    | LocalGenerationRowSource::OrderedSentence
            ) {
                assert!(!row.hit, "no exact I1/I2/IS continuation row may serve");
            }
        }
        assert_eq!(
            support_signature(step),
            vec![
                (b"carefully".to_vec(), [0, 0, 0, 0, 1]),
                (b"slowly".to_vec(), [0, 0, 0, 0, 1]),
            ]
        );
    }
}

#[derive(Serialize)]
struct SmokeRecord<'a> {
    schema: u32,
    fixture_kappa: &'a str,
    terminal: &'a str,
    left_full: &'a LocalGeometricGenerationReport,
    right_full: &'a LocalGeometricGenerationReport,
    left_state_disabled: &'a LocalGeometricGenerationReport,
    right_state_disabled: &'a LocalGeometricGenerationReport,
}

#[test]
fn relabelled_natural_smoke_requires_generator_revision() {
    assert_eq!(fixture_kappa(), SMOKE_FIXTURE_KAPPA);
    let artifact = frozen_artifact();
    assert_eq!(artifact.manifest_kappa(), CONSTRUCTION_ARTIFACT_KAPPA);
    assert_eq!(
        artifact.embedded_spin_manifest_kappa(),
        ATTENTION_MANIFEST_KAPPA
    );
    assert_eq!(artifact.codec_kappa(), CODEC_KAPPA);
    let artifact_bytes = artifact.canonical_bytes().unwrap();
    let generator = LocalGeometricGenerator::from_canonical_bytes(&artifact_bytes).unwrap();

    let trailing_whitespace = generator
        .generate(
            b"active agile athletes run ",
            LocalGenerationControl::FullPath,
            CONTINUATION_CAP,
        )
        .unwrap_err();
    assert!(
        trailing_whitespace
            .to_string()
            .contains("prompt trailing whitespace is unsupported"),
        "trailing whitespace must fail closed before boundary rendering: {trailing_whitespace}"
    );

    let left_full = generator
        .generate(
            LEFT_PROMPT,
            LocalGenerationControl::FullPath,
            CONTINUATION_CAP,
        )
        .unwrap();
    let right_full = generator
        .generate(
            RIGHT_PROMPT,
            LocalGenerationControl::FullPath,
            CONTINUATION_CAP,
        )
        .unwrap();
    let left_disabled = generator
        .generate(
            LEFT_PROMPT,
            LocalGenerationControl::StateDisabled,
            CONTINUATION_CAP,
        )
        .unwrap();
    let right_disabled = generator
        .generate(
            RIGHT_PROMPT,
            LocalGenerationControl::StateDisabled,
            CONTINUATION_CAP,
        )
        .unwrap();

    for report in [&left_full, &right_full, &left_disabled, &right_disabled] {
        assert_bounded_source_free_report(report);
        assert_exact_inversion(&artifact, report);
    }

    assert_eq!(
        emitted_payloads(&left_full),
        [b"slowly".to_vec(), b"carefully".to_vec()]
    );
    assert_eq!(
        emitted_payloads(&right_full),
        [b"carefully".to_vec(), b"slowly".to_vec()]
    );
    assert_eq!(
        emitted_payloads(&left_disabled),
        [b"slowly".to_vec(), b"slowly".to_vec()]
    );
    assert_eq!(
        emitted_payloads(&right_disabled),
        [b"slowly".to_vec(), b"slowly".to_vec()]
    );
    assert_eq!(left_full.continuation_bytes, b" slowly carefully");
    assert_eq!(right_full.continuation_bytes, b" carefully slowly");
    assert_ne!(left_full.continuation_bytes, right_full.continuation_bytes);
    assert_eq!(
        left_disabled.continuation_bytes,
        right_disabled.continuation_bytes
    );
    assert_eq!(
        cost_signature(&left_full.steps[0]),
        vec![
            (
                b"slowly".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 4,
                },
            ),
            (
                b"carefully".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees60,
                    lease_age: 2,
                },
            ),
        ]
    );
    assert_eq!(
        cost_signature(&right_full.steps[0]),
        vec![
            (
                b"carefully".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 4,
                },
            ),
            (
                b"slowly".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees60,
                    lease_age: 2,
                },
            ),
        ]
    );
    assert_eq!(
        cost_signature(&left_full.steps[1]),
        vec![
            (
                b"carefully".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 6,
                },
            ),
            (
                b"slowly".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Orthogonal,
                    lease_age: 3,
                },
            ),
        ]
    );
    assert_eq!(
        cost_signature(&right_full.steps[1]),
        vec![
            (
                b"slowly".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 5,
                },
            ),
            (
                b"carefully".to_vec(),
                PathLeaseCost {
                    angular_shell: H4S3AngularShell::Degrees36,
                    lease_age: 6,
                },
            ),
        ]
    );
    let disabled_costs = vec![
        (
            b"slowly".to_vec(),
            PathLeaseCost {
                angular_shell: H4S3AngularShell::Degrees72,
                lease_age: 1,
            },
        ),
        (
            b"carefully".to_vec(),
            PathLeaseCost {
                angular_shell: H4S3AngularShell::Degrees120,
                lease_age: 1,
            },
        ),
    ];
    for report in [&left_disabled, &right_disabled] {
        assert_eq!(cost_signature(&report.steps[0]), disabled_costs);
        assert_eq!(cost_signature(&report.steps[1]), disabled_costs);
    }

    let decisive_support = support_signature(&left_full.steps[0]);
    for report in [&right_full, &left_disabled, &right_disabled] {
        assert_eq!(support_signature(&report.steps[0]), decisive_support);
        assert_eq!(
            report.steps[0].path_geometry_evaluations,
            left_full.steps[0].path_geometry_evaluations
        );
    }
    assert_ne!(
        left_full.steps[0].selected.as_ref().unwrap().address_kappa,
        right_full.steps[0].selected.as_ref().unwrap().address_kappa
    );
    assert_ne!(
        right_full.steps[0].selected.as_ref().unwrap().address_kappa,
        right_disabled.steps[0]
            .selected
            .as_ref()
            .unwrap()
            .address_kappa
    );
    assert_eq!(
        left_disabled.steps[0]
            .selected
            .as_ref()
            .unwrap()
            .address_kappa,
        right_disabled.steps[0]
            .selected
            .as_ref()
            .unwrap()
            .address_kappa
    );

    for (prompt, control, expected) in [
        (LEFT_PROMPT, LocalGenerationControl::FullPath, &left_full),
        (RIGHT_PROMPT, LocalGenerationControl::FullPath, &right_full),
        (
            LEFT_PROMPT,
            LocalGenerationControl::StateDisabled,
            &left_disabled,
        ),
        (
            RIGHT_PROMPT,
            LocalGenerationControl::StateDisabled,
            &right_disabled,
        ),
    ] {
        let replay = generator
            .generate(prompt, control, CONTINUATION_CAP)
            .unwrap();
        assert_eq!(&replay, expected);
        assert_eq!(
            replay.canonical_bytes().unwrap(),
            expected.canonical_bytes().unwrap()
        );
        assert_eq!(
            replay.report_kappa().unwrap(),
            expected.report_kappa().unwrap()
        );
    }

    let record = SmokeRecord {
        schema: 1,
        fixture_kappa: SMOKE_FIXTURE_KAPPA,
        terminal: REVISION_TERMINAL,
        left_full: &left_full,
        right_full: &right_full,
        left_state_disabled: &left_disabled,
        right_state_disabled: &right_disabled,
    };
    let record_bytes = serde_json::to_vec(&record).unwrap();
    let record_kappa = format!("blake3:{}", blake3::hash(&record_bytes).to_hex());
    assert_eq!(record_kappa, SMOKE_RECORD_KAPPA);
}
