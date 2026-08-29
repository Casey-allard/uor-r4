use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use rayon::prelude::*;
use serde::Serialize;
use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec,
    CanonicalRouteArtifact, ConversationInput, ParagraphInput, TurnInput,
};
use uor_r4_core::construction_causal_return_attention::{
    construction_causal_return_policy_kappa, construction_causal_return_policy_report,
    ConstructionCausalReturnConstructionPartition, ConstructionCausalReturnControlledEncoder,
    ConstructionCausalReturnControlledRawQuery, ConstructionCausalReturnControlledSlotKind,
    ConstructionCausalReturnError, ConstructionCausalReturnFrame,
    ConstructionCausalReturnNegativeControl, ConstructionCausalReturnPopulationRole,
    ConstructionCausalReturnPrimePlacementPermutation, ConstructionCausalReturnRawQuery,
    ConstructionCausalReturnRawQueryReport, ConstructionCausalReturnTransitionBindingInput,
    ConstructionCausalReturnV1, ConstructionCausalReturnWorkReport,
};
use uor_r4_core::prime_route_attention::GeometricAddress;
use uor_r4_core::prime_route_geometric_attention::{
    AttentionSupportTrace, GeometricAttentionArtifact,
};

#[path = "support/construction_causal_return_gate0.rs"]
#[allow(dead_code)]
mod gate0;

use gate0::{
    cyclic_compiled_key_shuffle, cyclic_construction_label_pairing,
    incoherent_candidate_representation_swap, label_free_structural_coverage,
    strict_post_label_ceiling, Gate0ClassLookup, Gate0ExactRecallMap, Gate0GeometricClassMap,
    Gate0ObservationRow, Gate0QueryCandidateRow, Gate0RepresentationKeys, Gate0StrictCeilingReport,
    Gate0StructuralCoverageReport, Gate0UnavailableClassMap, Gate0ValidationLabel,
    GATE0_CLASS_LOOKUP_SHAPE_IDENTITY,
};

const IDENTITY_SCOPE: &str = "issue-983/construction-causal-return-v1";
const TURN_ID: &str = "turn-construction";
const FIXTURE_PARTITION_KAPPA: &str =
    "blake3:289cab4b5e22d45a61324137f2cc229c473570e6b9d3358dec16652dbfc84f83";
const CODEC_KAPPA: &str = "blake3:b1e8baf2ad3e6b9eb58f8d8c06809e76f37bdf91bfcce462a4e868e9952d654a";
const VOCABULARY_KAPPA: &str =
    "blake3:bd8c3c458f0c92fbbf9003b722417335678a1adfdfd7a7c0c68580dc129ffb75";
const CODEC_VOCABULARY_RECORD_KAPPA: &str =
    "blake3:0c45cd205ab0b14197ae178a668dbbbc893fc5c4e7beae97aeaf8a0be4b8be61";
const CONSTRUCTION_ARTIFACT_KAPPA: &str =
    "blake3:45c2475983fb047a9cedda6ef28edee832326cf730faeb5b590b7afc305505db";
const ATTENTION_MANIFEST_KAPPA: &str =
    "blake3:3e47dc0475c8f9da017ec1df456485d0c4957fe2203022dce7dcb537576b659a";
const CONSTRUCTION_ARTIFACT_RECORD_KAPPA: &str =
    "blake3:c4bca03f4c06d2ce58ed7167842b72491f4c9275c25fa9599f7bf30f54b1a670";
const MECHANISM_POLICY_KAPPA: &str =
    "blake3:0ab5118269a6aacbb4293ad876edcc82bf8f4ecca8b2121409b2bd8e0ff887c0";
const LABEL_FREE_VALIDATION_INPUT_KAPPA: &str =
    "blake3:17246f23c14a81ea83d388e5592af634027d07625ad7bdb1fdf743c1b562712a";
const SEALED_VALIDATION_LABEL_JOIN_KAPPA: &str =
    "blake3:5aa4b1c5880f660bc463a0650ebf21603d63170252b56e03e9a93e0a07e28c76";
const CONSTRUCTION_PARTITION_KAPPA: &str =
    "blake3:e9eaaa021752335149cd6cbd5fd17faac9e31f61db6741532292e6c6d1ef58ba";
const COMPILER_QUERY_FRAME_KAPPA: &str =
    "blake3:fe0fa1d5f56e97ba6508bf290c644b059caf4d23a96296e078b78dc522767c1a";
const H4_ROOT_TABLE_KAPPA: &str =
    "blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76";
const H4_MULTIPLICATION_TABLE_KAPPA: &str =
    "blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759";
const RAW_CENSUS_KAPPA: &str =
    "blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017";
const OUTCOME_KAPPA: &str =
    "blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587";
const CONSTRUCTION_SENTENCES: [&str; 12] = [
    "pilot near painters at station before noon is ready",
    "baker beside sailors by harbor before noon is calm",
    "artists near mechanic at station before noon are ready",
    "nurses beside driver by harbor before noon are calm",
    "captain near sailors at harbor today already has departed",
    "doctor beside nurses at clinic today already has arrived",
    "workers near manager at factory today already have gathered",
    "pilots beside mechanic at hangar today already have departed",
    "teacher near students at school early yesterday was absent",
    "driver beside workers at depot early yesterday was late",
    "students near teacher at school early yesterday were absent",
    "workers beside driver at depot early yesterday were late",
];
const CONSTRUCTION_IDS: [&str; 12] = [
    "A-is-1", "A-is-2", "A-are-1", "A-are-2", "B-has-1", "B-has-2", "B-have-1", "B-have-2",
    "C-was-1", "C-was-2", "C-were-1", "C-were-2",
];
const CONSTRUCTION_FAMILIES: [&str; 12] = [
    "is-are", "is-are", "is-are", "is-are", "has-have", "has-have", "has-have", "has-have",
    "was-were", "was-were", "was-were", "was-were",
];
const VALIDATION_ONLY_SURFACES: [&str; 10] = [
    "architect",
    "authors",
    "builders",
    "curator",
    "dancers",
    "editor",
    "gallery",
    "office",
    "quietly",
    "site",
];

#[derive(Debug, Clone, Copy)]
struct ValidationInput {
    id: &'static str,
    family: &'static str,
    prompt: &'static str,
    candidates: [&'static str; 2],
    trailing_four: [&'static str; 4],
}

const VALIDATION_INPUTS: [ValidationInput; 6] = [
    ValidationInput {
        id: "V-3f2c9a71",
        family: "is-are",
        prompt: "curator beside dancers by gallery at noon",
        candidates: ["are", "is"],
        trailing_four: ["by", "gallery", "at", "noon"],
    },
    ValidationInput {
        id: "V-8d04e6b5",
        family: "is-are",
        prompt: "dancers beside curator by gallery at noon",
        candidates: ["are", "is"],
        trailing_four: ["by", "gallery", "at", "noon"],
    },
    ValidationInput {
        id: "V-51ab7c90",
        family: "has-have",
        prompt: "editor beside authors quietly at office already",
        candidates: ["has", "have"],
        trailing_four: ["quietly", "at", "office", "already"],
    },
    ValidationInput {
        id: "V-c7e2384d",
        family: "has-have",
        prompt: "authors beside editor quietly at office already",
        candidates: ["has", "have"],
        trailing_four: ["quietly", "at", "office", "already"],
    },
    ValidationInput {
        id: "V-046fd1a3",
        family: "was-were",
        prompt: "architect beside builders quietly at site yesterday",
        candidates: ["was", "were"],
        trailing_four: ["quietly", "at", "site", "yesterday"],
    },
    ValidationInput {
        id: "V-b9a572ce",
        family: "was-were",
        prompt: "builders beside architect quietly at site yesterday",
        candidates: ["was", "were"],
        trailing_four: ["quietly", "at", "site", "yesterday"],
    },
];

#[derive(Debug, Clone, Copy, Serialize)]
struct ValidationLabelRow<'a> {
    id: &'a str,
    expected_candidate: &'a str,
}

#[derive(Serialize)]
struct SealedValidationLabelJoin<'a> {
    schema: u32,
    identity: &'a str,
    rows: [ValidationLabelRow<'a>; 6],
}

static SEALED_VALIDATION_LABEL_JOIN_LOADS: AtomicUsize = AtomicUsize::new(0);

fn sealed_validation_label_join() -> SealedValidationLabelJoin<'static> {
    SEALED_VALIDATION_LABEL_JOIN_LOADS.fetch_add(1, Ordering::SeqCst);
    SealedValidationLabelJoin {
        schema: 1,
        identity: "uor-r4.construction-causal-return-validation-label-join/1",
        rows: [
            ValidationLabelRow {
                id: "V-3f2c9a71",
                expected_candidate: "is",
            },
            ValidationLabelRow {
                id: "V-8d04e6b5",
                expected_candidate: "are",
            },
            ValidationLabelRow {
                id: "V-51ab7c90",
                expected_candidate: "has",
            },
            ValidationLabelRow {
                id: "V-c7e2384d",
                expected_candidate: "have",
            },
            ValidationLabelRow {
                id: "V-046fd1a3",
                expected_candidate: "was",
            },
            ValidationLabelRow {
                id: "V-b9a572ce",
                expected_candidate: "were",
            },
        ],
    }
}

#[derive(Serialize)]
struct ConstructionFreezeRow<'a> {
    id: &'a str,
    family: &'a str,
    source: &'a str,
    candidate_ordinal: usize,
}

#[derive(Serialize)]
struct ValidationInputRow<'a> {
    id: &'a str,
    family: &'a str,
    prompt: &'a str,
    candidates: [&'a str; 2],
    trailing_four: [&'a str; 4],
}

#[derive(Serialize)]
struct FixturePartitionFreeze<'a> {
    schema: u32,
    identity: &'a str,
    identity_scope: &'a str,
    construction: Vec<ConstructionFreezeRow<'a>>,
    validation_ids: Vec<&'a str>,
    validation_labels_record: &'a str,
    validation_labels_kappa: &'a str,
    construction_transitions_per_candidate: usize,
    validation_decisions: usize,
    known_fixture_exclusions: [&'a str; 3],
}

#[derive(Serialize)]
struct CodecVocabularyFreeze<'a> {
    schema: u32,
    identity: &'a str,
    codec_kappa: &'a str,
    vocabulary_kappa: &'a str,
    validation_only_surfaces: [&'a str; 10],
}

#[derive(Serialize)]
struct ConstructionArtifactFreeze<'a> {
    schema: u32,
    identity: &'a str,
    fixture_partition_kappa: &'a str,
    codec_vocabulary_record_kappa: &'a str,
    artifact_kappa: &'a str,
    attention_manifest_kappa: &'a str,
    rebuild_witnesses: usize,
    transition_bearing_construction_witnesses: usize,
    construction_transition_ordinal: usize,
}

#[derive(Serialize)]
struct LabelFreeValidationFreeze<'a> {
    schema: u32,
    identity: &'a str,
    codec_record_kappa: &'a str,
    construction_artifact_record_kappa: &'a str,
    mechanism_policy_kappa: &'a str,
    rows: Vec<ValidationInputRow<'a>>,
    forbidden_inputs: [&'a str; 5],
}

struct FrozenBundle {
    codec: CanonicalLexicalCodec,
    artifact: CanonicalRouteArtifact,
    attention: GeometricAttentionArtifact,
    address_by_surface: BTreeMap<String, GeometricAddress>,
    surface_commitment_by_address_kappa: BTreeMap<String, String>,
    rebuild_witnesses: usize,
    transition_bearing_construction_witnesses: usize,
}

fn population_address_book(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
) -> BTreeMap<String, GeometricAddress> {
    let surfaces = CONSTRUCTION_SENTENCES
        .iter()
        .flat_map(|sentence| sentence.split_ascii_whitespace())
        .chain(
            VALIDATION_INPUTS
                .iter()
                .flat_map(|input| input.prompt.split_ascii_whitespace()),
        )
        .chain(VALIDATION_INPUTS.iter().flat_map(|input| input.candidates))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unit_ids = surfaces
        .iter()
        .map(|surface| {
            let encoded = codec.encode(0, 0, surface.as_bytes()).unwrap();
            assert!(encoded.trailing_bytes.is_empty());
            assert_eq!(encoded.units.len(), 1, "surface {surface} must be one unit");
            encoded.units[0].unit_id
        })
        .collect::<Vec<_>>();
    let addresses = artifact.lexical_route_addresses(&unit_ids).unwrap();
    surfaces
        .into_iter()
        .zip(addresses)
        .map(|(surface, address)| (surface, address.unwrap()))
        .collect()
}

fn build_frozen_bundle() -> FrozenBundle {
    let codec = CanonicalLexicalCodec::compile(&codec_registration_input()).unwrap();
    let artifact = CanonicalRouteArtifact::ingest(&codec, &construction_input()).unwrap();
    let manifest = artifact.embedded_spin_manifest().unwrap();
    let rebuild_witnesses = manifest.rebuild_witnesses.len();
    let transition_bearing_construction_witnesses = manifest
        .rebuild_witnesses
        .iter()
        .filter(|witness| witness.address_indices.len() > 1)
        .count();
    let attention = GeometricAttentionArtifact::compile_from_manifest_witnesses(&manifest).unwrap();
    let address_by_surface = population_address_book(&codec, &artifact);
    let surface_commitment_by_address_kappa = address_by_surface
        .iter()
        .map(|(surface, address)| (address.canonical_kappa().unwrap(), surface.clone()))
        .collect();
    FrozenBundle {
        codec,
        artifact,
        attention,
        address_by_surface,
        surface_commitment_by_address_kappa,
        rebuild_witnesses,
        transition_bearing_construction_witnesses,
    }
}

fn input_with_global_snapshot(global_snapshot_units: Vec<Vec<u8>>) -> ConversationInput {
    ConversationInput {
        identity_scope: IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units).unwrap(),
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: TURN_ID.to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: CONSTRUCTION_SENTENCES
                    .iter()
                    .map(|sentence| sentence.as_bytes().to_vec())
                    .collect(),
            }],
        }],
    }
}

fn codec_registration_input() -> ConversationInput {
    input_with_global_snapshot(
        VALIDATION_ONLY_SURFACES
            .iter()
            .map(|surface| surface.as_bytes().to_vec())
            .collect(),
    )
}

fn construction_input() -> ConversationInput {
    let global_snapshot_units = vec![b"pilot".to_vec()];
    ConversationInput {
        identity_scope: IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units).unwrap(),
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: TURN_ID.to_owned(),
            paragraphs: vec![
                ParagraphInput {
                    sentences: CONSTRUCTION_SENTENCES
                        .iter()
                        .map(|sentence| sentence.as_bytes().to_vec())
                        .collect(),
                },
                // These singleton witnesses bind validation-only vocabulary
                // into the exact address registry. They contain no transition
                // and are excluded from every construction prototype.
                ParagraphInput {
                    sentences: VALIDATION_ONLY_SURFACES
                        .iter()
                        .map(|surface| surface.as_bytes().to_vec())
                        .collect(),
                },
            ],
        }],
    }
}

fn frozen_bundle() -> &'static FrozenBundle {
    static BUNDLE: OnceLock<FrozenBundle> = OnceLock::new();
    BUNDLE.get_or_init(build_frozen_bundle)
}

fn record_kappa<T: Serialize + ?Sized>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap();
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

fn history_for_prompt(bundle: &FrozenBundle, prompt: &str) -> Vec<GeometricAddress> {
    let encoded = bundle.codec.encode(0, 0, prompt.as_bytes()).unwrap();
    assert_eq!(bundle.codec.decode(&encoded).unwrap(), prompt.as_bytes());
    assert!(encoded.trailing_bytes.is_empty());
    prompt
        .split_ascii_whitespace()
        .map(|surface| bundle.address_by_surface.get(surface).unwrap().clone())
        .collect()
}

#[derive(Clone)]
struct PreparedConstruction {
    id: &'static str,
    predecessor_history: Vec<GeometricAddress>,
    observed_next: GeometricAddress,
    candidate_union: [GeometricAddress; 2],
}

#[derive(Clone)]
struct PreparedValidation {
    input: ValidationInput,
    observed_history: Vec<GeometricAddress>,
}

fn family_candidate_surfaces(family: &str) -> [&'static str; 2] {
    match family {
        "is-are" => ["are", "is"],
        "has-have" => ["has", "have"],
        "was-were" => ["was", "were"],
        other => panic!("unknown candidate family {other}"),
    }
}

fn address_for_surface(bundle: &FrozenBundle, surface: &str) -> GeometricAddress {
    bundle.address_by_surface.get(surface).unwrap().clone()
}

fn prepared_construction(bundle: &FrozenBundle) -> Vec<PreparedConstruction> {
    CONSTRUCTION_SENTENCES
        .iter()
        .zip(CONSTRUCTION_IDS)
        .zip(CONSTRUCTION_FAMILIES)
        .map(|((source, id), family)| {
            let units = source.split_ascii_whitespace().collect::<Vec<_>>();
            assert_eq!(units.len(), 9, "{id} construction width");
            let predecessor_text = units[..7].join(" ");
            let observed_next_surface = units[7];
            let candidates = family_candidate_surfaces(family);
            assert!(candidates.contains(&observed_next_surface));
            PreparedConstruction {
                id,
                predecessor_history: history_for_prompt(bundle, &predecessor_text),
                observed_next: address_for_surface(bundle, observed_next_surface),
                candidate_union: [
                    address_for_surface(bundle, candidates[0]),
                    address_for_surface(bundle, candidates[1]),
                ],
            }
        })
        .collect()
}

fn prepared_validation(bundle: &FrozenBundle) -> Vec<PreparedValidation> {
    VALIDATION_INPUTS
        .iter()
        .copied()
        .map(|input| PreparedValidation {
            input,
            observed_history: history_for_prompt(bundle, input.prompt),
        })
        .collect()
}

fn construction_partition(
    rows: &[PreparedConstruction],
) -> ConstructionCausalReturnConstructionPartition {
    let inputs = rows
        .iter()
        .map(|row| ConstructionCausalReturnTransitionBindingInput {
            transition_id: row.id.to_owned(),
            predecessor_history: row.predecessor_history.clone(),
            observed_next: row.observed_next.clone(),
            candidate_union: row.candidate_union.clone(),
        })
        .collect::<Vec<_>>();
    ConstructionCausalReturnConstructionPartition::compile(&inputs).unwrap()
}

fn construction_frame(
    bundle: &FrozenBundle,
    partition: &ConstructionCausalReturnConstructionPartition,
) -> ConstructionCausalReturnFrame {
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    ConstructionCausalReturnFrame::from_canonical_artifacts(
        &bundle.codec,
        &bundle.artifact,
        &bundle.attention,
        partition,
        &table,
    )
    .unwrap()
}

fn registered_surface_for_address(bundle: &FrozenBundle, address: &GeometricAddress) -> String {
    bundle
        .surface_commitment_by_address_kappa
        .get(&address.canonical_kappa().unwrap())
        .unwrap()
        .clone()
}

fn support_for_decision(bundle: &FrozenBundle, decision: ValidationInput) -> AttentionSupportTrace {
    let history = history_for_prompt(bundle, decision.prompt);
    assert_eq!(history.len(), 7, "{} prompt width", decision.id);
    let state = bundle
        .attention
        .causal_state_from_history(&history)
        .unwrap();
    bundle.attention.query_support_only(&state).unwrap()
}

fn prompt_multiset(prompt: &str) -> Vec<&str> {
    let mut units = prompt.split_ascii_whitespace().collect::<Vec<_>>();
    units.sort_unstable();
    units
}

#[test]
fn freeze_pre_geometry_record_identities() {
    let bundle = frozen_bundle();
    let validation_label_join = sealed_validation_label_join();
    let sealed_validation_label_join_kappa = record_kappa(&validation_label_join);
    let fixture = FixturePartitionFreeze {
        schema: 1,
        identity: "uor-r4.construction-causal-return-fixture/1",
        identity_scope: IDENTITY_SCOPE,
        construction: CONSTRUCTION_IDS
            .iter()
            .zip(CONSTRUCTION_FAMILIES)
            .zip(CONSTRUCTION_SENTENCES)
            .map(|((id, family), source)| ConstructionFreezeRow {
                id,
                family,
                source,
                candidate_ordinal: 7,
            })
            .collect(),
        validation_ids: VALIDATION_INPUTS.iter().map(|row| row.id).collect(),
        validation_labels_record: "uor-r4.construction-causal-return-validation-label-join/1",
        validation_labels_kappa: &sealed_validation_label_join_kappa,
        construction_transitions_per_candidate: 2,
        validation_decisions: 6,
        known_fixture_exclusions: ["issue-953", "run-runs", "issue-970"],
    };
    let fixture_partition_kappa = record_kappa(&fixture);

    let codec = CodecVocabularyFreeze {
        schema: 1,
        identity: "uor-r4.construction-causal-return-codec/1",
        codec_kappa: bundle.codec.codec_kappa(),
        vocabulary_kappa: bundle.codec.vocabulary_kappa(),
        validation_only_surfaces: VALIDATION_ONLY_SURFACES,
    };
    let codec_vocabulary_kappa = record_kappa(&codec);

    let construction = ConstructionArtifactFreeze {
        schema: 1,
        identity: "uor-r4.construction-causal-return-artifact/1",
        fixture_partition_kappa: &fixture_partition_kappa,
        codec_vocabulary_record_kappa: &codec_vocabulary_kappa,
        artifact_kappa: bundle.artifact.manifest_kappa(),
        attention_manifest_kappa: bundle.attention.manifest_kappa(),
        rebuild_witnesses: bundle.rebuild_witnesses,
        transition_bearing_construction_witnesses: bundle.transition_bearing_construction_witnesses,
        construction_transition_ordinal: 7,
    };
    let construction_artifact_record_kappa = record_kappa(&construction);

    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let policy = construction_causal_return_policy_report();
    let mechanism_policy_kappa = construction_causal_return_policy_kappa().unwrap();

    let validation = LabelFreeValidationFreeze {
        schema: 1,
        identity: "uor-r4.construction-causal-return-validation-input/1",
        codec_record_kappa: &codec_vocabulary_kappa,
        construction_artifact_record_kappa: &construction_artifact_record_kappa,
        mechanism_policy_kappa: &mechanism_policy_kappa,
        rows: VALIDATION_INPUTS
            .iter()
            .map(|row| ValidationInputRow {
                id: row.id,
                family: row.family,
                prompt: row.prompt,
                candidates: row.candidates,
                trailing_four: row.trailing_four,
            })
            .collect(),
        forbidden_inputs: [
            "validation-label",
            "actual-next-route",
            "teacher-output",
            "provider-text",
            "source-tensor",
        ],
    };
    let label_free_validation_input_kappa = record_kappa(&validation);
    println!("fixture_partition_kappa={fixture_partition_kappa}");
    println!("codec_kappa={}", bundle.codec.codec_kappa());
    println!("vocabulary_kappa={}", bundle.codec.vocabulary_kappa());
    println!("codec_vocabulary_record_kappa={codec_vocabulary_kappa}");
    println!(
        "construction_artifact_kappa={}",
        bundle.artifact.manifest_kappa()
    );
    println!(
        "attention_manifest_kappa={}",
        bundle.attention.manifest_kappa()
    );
    println!("construction_artifact_record_kappa={construction_artifact_record_kappa}");
    println!("mechanism_policy_kappa={mechanism_policy_kappa}");
    println!("label_free_validation_input_kappa={label_free_validation_input_kappa}");
    println!("sealed_validation_label_join_kappa={sealed_validation_label_join_kappa}");

    assert_eq!(fixture.construction.len(), 12);
    assert_eq!(validation.rows.len(), 6);
    for pair in validation_label_join.rows.chunks_exact(2) {
        assert_ne!(pair[0].expected_candidate, pair[1].expected_candidate);
    }
    assert_eq!(bundle.rebuild_witnesses, 23);
    assert_eq!(bundle.transition_bearing_construction_witnesses, 12);
    assert_eq!(
        bundle.rebuild_witnesses - bundle.transition_bearing_construction_witnesses,
        VALIDATION_ONLY_SURFACES.len() + 1
    );
    assert_eq!(policy.maximum_observed_prefixes, 8);
    assert_eq!(policy.retained_prototypes_per_candidate, 2);
    assert_eq!(
        policy.selector_rule,
        "select iff exactly one admitted candidate is SELECT and the other is REJECT"
    );
    assert!(table.h4_root_table_kappa.starts_with("blake3:"));
    assert!(table.multiplication_table_kappa.starts_with("blake3:"));
    assert_eq!(fixture_partition_kappa, FIXTURE_PARTITION_KAPPA);
    assert_eq!(bundle.codec.codec_kappa(), CODEC_KAPPA);
    assert_eq!(bundle.codec.vocabulary_kappa(), VOCABULARY_KAPPA);
    assert_eq!(codec_vocabulary_kappa, CODEC_VOCABULARY_RECORD_KAPPA);
    assert_eq!(
        bundle.artifact.manifest_kappa(),
        CONSTRUCTION_ARTIFACT_KAPPA
    );
    assert_eq!(bundle.attention.manifest_kappa(), ATTENTION_MANIFEST_KAPPA);
    assert_eq!(
        construction_artifact_record_kappa,
        CONSTRUCTION_ARTIFACT_RECORD_KAPPA
    );
    assert_eq!(mechanism_policy_kappa, MECHANISM_POLICY_KAPPA);
    assert_eq!(
        label_free_validation_input_kappa,
        LABEL_FREE_VALIDATION_INPUT_KAPPA
    );
    assert_eq!(
        sealed_validation_label_join_kappa,
        SEALED_VALIDATION_LABEL_JOIN_KAPPA
    );
}

#[test]
fn frozen_population_has_independent_matched_support() {
    assert!(CONSTRUCTION_SENTENCES.iter().all(|sentence| !sentence
        .split_ascii_whitespace()
        .any(|unit| unit == "run" || unit == "runs")));
    assert_eq!(CONSTRUCTION_SENTENCES.len(), 12);
    assert_eq!(VALIDATION_INPUTS.len(), 6);

    for pair in VALIDATION_INPUTS.chunks_exact(2) {
        assert_eq!(pair[0].family, pair[1].family);
        assert_eq!(
            prompt_multiset(pair[0].prompt),
            prompt_multiset(pair[1].prompt)
        );
        assert_eq!(pair[0].trailing_four, pair[1].trailing_four);
    }

    let construction_units = CONSTRUCTION_SENTENCES
        .iter()
        .flat_map(|sentence| sentence.split_ascii_whitespace())
        .collect::<BTreeSet<_>>();
    for surface in VALIDATION_ONLY_SURFACES {
        assert!(!construction_units.contains(surface));
    }
    for decision in VALIDATION_INPUTS {
        let suffix = decision.trailing_four.join(" ");
        assert!(!CONSTRUCTION_SENTENCES
            .iter()
            .any(|sentence| sentence.contains(&suffix)));
    }

    let bundle = frozen_bundle();
    let mut paired_work = Vec::new();
    for decision in VALIDATION_INPUTS {
        let support = support_for_decision(bundle, decision);
        let mut candidates = support
            .candidates
            .iter()
            .map(|candidate| registered_surface_for_address(bundle, &candidate.next))
            .collect::<Vec<_>>();
        candidates.sort();
        assert_eq!(candidates, decision.candidates, "{} support", decision.id);
        assert_eq!(support.candidates.len(), 2);
        assert_eq!(support.unique_candidates_before_ceiling, 2);
        assert!(support.candidate_ceiling >= 2);
        paired_work.push((
            decision.family,
            support.rows_read.len(),
            support.candidate_entries_available,
            support.candidate_entries_examined,
            support.candidate_entries_admitted,
        ));
    }
    for pair in paired_work.chunks_exact(2) {
        assert_eq!(pair[0], pair[1]);
    }
}

#[test]
fn lexical_route_address_batch_preserves_order_duplicates_and_validation() {
    let bundle = frozen_bundle();
    let unit_ids = ["pilot", "noon", "pilot"]
        .iter()
        .map(|surface| {
            let encoded = bundle.codec.encode(0, 0, surface.as_bytes()).unwrap();
            assert_eq!(encoded.units.len(), 1);
            assert!(encoded.trailing_bytes.is_empty());
            encoded.units[0].unit_id
        })
        .collect::<Vec<_>>();
    let batch = bundle.artifact.lexical_route_addresses(&unit_ids).unwrap();
    let individual = unit_ids
        .iter()
        .map(|unit_id| bundle.artifact.lexical_route_address(*unit_id).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(batch, individual);
    assert_eq!(batch.len(), unit_ids.len());
    assert_eq!(batch[0], batch[2]);
    assert_ne!(batch[0], batch[1]);
    let absent_batch = bundle
        .artifact
        .lexical_route_addresses(&[u32::MAX])
        .unwrap();
    let absent_individual = bundle.artifact.lexical_route_address(u32::MAX).unwrap();
    assert_eq!(absent_batch, vec![absent_individual]);
    assert_eq!(absent_batch, vec![None]);
}

#[test]
fn freeze_compiler_query_frame_identity() {
    let bundle = frozen_bundle();
    let construction = prepared_construction(bundle);
    let partition = construction_partition(&construction);
    let frame = construction_frame(bundle, &partition);
    assert_eq!(partition.transitions().len(), 12);
    assert_eq!(partition.canonical_report().candidate_count, 6);
    assert_eq!(
        partition.reproduce_partition_kappa().unwrap(),
        partition.partition_kappa()
    );
    assert_eq!(frame.reproduce_frame_kappa().unwrap(), frame.frame_kappa());
    assert_eq!(
        frame.construction_partition_kappa(),
        partition.partition_kappa()
    );
    assert_eq!(frame.policy_kappa(), MECHANISM_POLICY_KAPPA);
    assert_eq!(partition.partition_kappa(), CONSTRUCTION_PARTITION_KAPPA);
    assert_eq!(frame.frame_kappa(), COMPILER_QUERY_FRAME_KAPPA);
    assert_eq!(frame.h4_root_table_kappa(), H4_ROOT_TABLE_KAPPA);
    assert_eq!(
        frame.multiplication_table_kappa(),
        H4_MULTIPLICATION_TABLE_KAPPA
    );
    println!(
        "construction_partition_kappa={}",
        partition.partition_kappa()
    );
    println!("compiler_query_frame_kappa={}", frame.frame_kappa());
    println!("h4_root_table_kappa={}", frame.h4_root_table_kappa());
    println!(
        "multiplication_table_kappa={}",
        frame.multiplication_table_kappa()
    );

    let mismatch_inputs = construction
        .iter()
        .enumerate()
        .map(
            |(index, row)| ConstructionCausalReturnTransitionBindingInput {
                transition_id: if index == 0 {
                    "A-is-1-frame-mismatch".to_owned()
                } else {
                    row.id.to_owned()
                },
                predecessor_history: row.predecessor_history.clone(),
                observed_next: row.observed_next.clone(),
                candidate_union: row.candidate_union.clone(),
            },
        )
        .collect::<Vec<_>>();
    let mismatch_partition =
        ConstructionCausalReturnConstructionPartition::compile(&mismatch_inputs).unwrap();
    let mismatch_frame = construction_frame(bundle, &mismatch_partition);
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let mismatch_raw = mismatch_frame
        .raw_query(
            &bundle.attention,
            &construction[0].predecessor_history,
            &table,
        )
        .unwrap();
    let error = partition
        .authorize_label_join(
            &mismatch_raw,
            construction[0].id,
            &construction[0].observed_next,
        )
        .unwrap_err();
    assert_eq!(
        error,
        ConstructionCausalReturnError::UnavailableFrameMismatch
    );
}

#[derive(Clone)]
enum FrozenGate0Map {
    Geometric(Gate0GeometricClassMap),
    ExactRecall(Gate0ExactRecallMap),
    Unavailable(Gate0UnavailableClassMap),
}

impl FrozenGate0Map {
    fn kind(&self) -> &'static str {
        match self {
            Self::Geometric(_) => "geometric_r_min_then_r_full",
            Self::ExactRecall(_) => "exact_recall_only",
            Self::Unavailable(_) => "typed_unavailable",
        }
    }

    fn map_kappa(&self) -> &str {
        match self {
            Self::Geometric(map) => map.map_kappa(),
            Self::ExactRecall(map) => map.map_kappa(),
            Self::Unavailable(map) => map.map_kappa(),
        }
    }

    fn inventory_value(&self) -> serde_json::Value {
        match self {
            Self::Geometric(map) => serde_json::to_value(map).unwrap(),
            Self::ExactRecall(map) => serde_json::to_value(map).unwrap(),
            Self::Unavailable(map) => serde_json::to_value(map).unwrap(),
        }
    }

    fn construction_rows(&self) -> usize {
        match self {
            Self::Geometric(map) => map.observation_rows,
            Self::ExactRecall(map) => map.observation_rows,
            Self::Unavailable(map) => map.observation_rows,
        }
    }

    fn all_selection_classes_pure(&self) -> bool {
        match self {
            Self::Geometric(map) => map.all_selection_classes_pure,
            Self::ExactRecall(map) => map.all_classes_pure,
            Self::Unavailable(_) => false,
        }
    }

    fn promotion(&self) -> (usize, usize, usize) {
        match self {
            Self::Geometric(map) => (
                map.promoted_minimum_classes,
                map.promoted_rate_numerator(),
                map.promoted_rate_denominator(),
            ),
            Self::ExactRecall(_) | Self::Unavailable(_) => (0, 0, self.construction_rows()),
        }
    }

    fn coverage(&self, rows: &[Gate0QueryCandidateRow]) -> Gate0StructuralCoverageReport {
        match self {
            Self::Geometric(map) => label_free_structural_coverage(map, rows).unwrap(),
            Self::ExactRecall(map) => label_free_structural_coverage(map, rows).unwrap(),
            Self::Unavailable(map) => label_free_structural_coverage(map, rows).unwrap(),
        }
    }

    fn ceiling(
        &self,
        rows: &[Gate0QueryCandidateRow],
        labels: &[Gate0ValidationLabel],
    ) -> Gate0StrictCeilingReport {
        match self {
            Self::Geometric(map) => strict_post_label_ceiling(map, rows, labels).unwrap(),
            Self::ExactRecall(map) => strict_post_label_ceiling(map, rows, labels).unwrap(),
            Self::Unavailable(map) => strict_post_label_ceiling(map, rows, labels).unwrap(),
        }
    }

    fn reproduce_kappa(&self) -> String {
        match self {
            Self::Geometric(map) => map.reproduce_artifact_kappa().unwrap(),
            Self::ExactRecall(map) => map.reproduce_artifact_kappa().unwrap(),
            Self::Unavailable(map) => map.reproduce_artifact_kappa().unwrap(),
        }
    }
}

#[derive(Clone)]
struct Gate0EvaluationArm {
    name: &'static str,
    map: FrozenGate0Map,
    queries: Vec<Gate0QueryCandidateRow>,
}

#[derive(Debug, Clone, Serialize)]
struct Gate0ArmRawRecord {
    name: &'static str,
    negative_control: Option<ConstructionCausalReturnNegativeControl>,
    raw_basis: &'static str,
    map_kind: &'static str,
    map_kappa: String,
    reproduced_map_kappa: String,
    construction_observation_rows_kappa: String,
    construction_rows: usize,
    all_selection_classes_pure: bool,
    promoted_minimum_classes: usize,
    promoted_rows: usize,
    promotion_denominator: usize,
    inventory: serde_json::Value,
    label_free_coverage: Gate0StructuralCoverageReport,
    construction_raw_queries: Vec<serde_json::Value>,
    validation_raw_queries: Vec<serde_json::Value>,
    construction_support_reports: Vec<CanonicalAttentionSupportReport>,
    validation_support_reports: Vec<CanonicalAttentionSupportReport>,
    support_reports_kappa: String,
    transformation: Option<serde_json::Value>,
    raw_report_frame_kappa: String,
    raw_report_frame_and_control_match_arm: bool,
    support_equal_to_real: bool,
    declared_work_equal_to_real: bool,
    class_lookup_shape_exact: bool,
    class_lookup_shape_equal_to_real: bool,
    typed_populated_padding_aliases: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CanonicalAttentionRowKey {
    LastOne {
        address_kappa: String,
    },
    LastTwo {
        previous_address_kappa: String,
        last_address_kappa: String,
    },
    LastTwoUnavailable,
    OrderedSentence {
        sentence_kappa: String,
    },
    Divisor {
        prime: u32,
    },
    AdjacentSpin {
        hopf_octant: u8,
        torsion_bin: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CanonicalAttentionRowRead {
    slot_index: usize,
    source: &'static str,
    key: CanonicalAttentionRowKey,
    consulted: bool,
    hit: bool,
    physical_row_present: bool,
    fallback_active: bool,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CanonicalAttentionSupportCandidate {
    candidate_address_kappa: String,
    last_one: u32,
    last_two: u32,
    ordered_sentence: u32,
    divisor: u32,
    adjacent_spin: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CanonicalAttentionSupportReport {
    manifest_kappa: String,
    query_policy_identity: String,
    query_policy_kappa: String,
    fallback_active: bool,
    rows_read: Vec<CanonicalAttentionRowRead>,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    support_admission: &'static str,
    candidates: Vec<CanonicalAttentionSupportCandidate>,
    support_kappa: String,
}

#[derive(Debug, Clone, Serialize)]
struct OperativeAntiRecallReport {
    construction_histories: usize,
    validation_histories: usize,
    exact_raw_history_overlaps: usize,
    exact_trailing_four_suffix_overlaps: usize,
    exact_ordered_route_witness_overlaps: usize,
    exact_complete_candidate_representation_overlaps: usize,
    operative_raw_prototype_recall: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationWorkAggregate {
    decisions: usize,
    support_rows_read: usize,
    relation_slots: usize,
    declared_prototype_class_slot_reads: usize,
    performed_prototype_class_slot_reads: usize,
    declared_payload_inversions: usize,
    performed_payload_inversions: usize,
    source_inputs: usize,
    provider_inputs: usize,
    teacher_inputs: usize,
    future_route_inputs: usize,
    validation_label_inputs: usize,
}

#[derive(Debug, Clone, Serialize)]
struct CountOnlyAnchorClass {
    anchor_address_kappa: String,
    candidate_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct CountOnlyValidationDecision {
    decision_id: String,
    anchor_address_kappa: String,
    candidate_address_kappas: Vec<String>,
    selected_candidate_address_kappa: Option<String>,
    tied: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CountOnlyLastAnchorReport {
    identity: &'static str,
    construction_condition: &'static str,
    selection_rule: &'static str,
    classes: Vec<CountOnlyAnchorClass>,
    validation: Vec<CountOnlyValidationDecision>,
    admits_candidates: bool,
    supplies_geometric_selector: bool,
    attention_claim: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CoherentRelabelEntry {
    source_candidate_address_kappa: String,
    relabeled_candidate_address_kappa: String,
    source_registered_surface_commitment: String,
    registered_surface_at_relabeled_candidate_before: String,
    registered_surface_at_relabeled_candidate_after: String,
}

#[derive(Debug, Clone, Serialize)]
struct CoherentRelabelRawReport {
    identity: &'static str,
    scope: &'static str,
    native_codec_or_placement_rebuild: bool,
    availability: &'static str,
    entries: Vec<CoherentRelabelEntry>,
    source_construction_rows_kappa: String,
    relabeled_construction_rows_kappa: String,
    source_validation_support_rows_kappa: String,
    relabeled_validation_support_rows_kappa: String,
    construction_rows_relabelled: usize,
    validation_rows_relabelled: usize,
    candidate_mapping_bijective: bool,
    partition_candidate_identity_correspondence: bool,
    support_candidate_identity_and_counts_correspondence: bool,
    raw_candidate_identity_representation_and_work_correspondence: bool,
    construction_action_correspondence: bool,
    validation_query_correspondence: bool,
    surface_commitment_correspondence: bool,
    exact_row_correspondence: bool,
    registered_surface_association_reproduced: bool,
    map_kappa_reproduced: bool,
    structural_coverage_reproduced: bool,
    support_prototypes_surface_commitments_and_construction_actions_move_together: bool,
    original_qualification_record_kappa: String,
    relabeled_qualification_record_kappa: String,
    twice_relabeled_qualification_record_kappa: String,
    candidate_identity_occurrences: usize,
    candidate_identity_occurrences_changed: usize,
    no_candidate_identity_occurrence_left_unmapped: bool,
    involution_twice_reproduces_original_bytes: bool,
    qualification_record_alpha_equivariance_without_payload: bool,
    complete_preselector_qualification_record_alpha_equivariance: bool,
    payload_association_status: &'static str,
    validation_label_commitment_loaded: bool,
    report_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CandidateSurfaceCommitment {
    candidate_address_kappa: String,
    registered_surface: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlphaQualificationRecord {
    partition: AlphaPartitionRecord,
    construction_support_reports: Vec<AlphaSupportReport>,
    validation_support_reports: Vec<AlphaSupportReport>,
    construction_raw_queries: Vec<ConstructionCausalReturnRawQueryReport>,
    validation_raw_queries: Vec<ConstructionCausalReturnRawQueryReport>,
    construction_observation_rows: Vec<Gate0ObservationRow>,
    validation_query_rows: Vec<Gate0QueryCandidateRow>,
    surface_commitments: Vec<CandidateSurfaceCommitment>,
    compiled_class_map_domain: &'static str,
    map_kappa: String,
    reproduced_map_kappa: String,
    structural_coverage: Gate0StructuralCoverageReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlphaPartitionTransition {
    transition_id: String,
    predecessor_history_kappa: String,
    predecessor_address_kappas: Vec<String>,
    observed_next_address_kappa: String,
    candidate_union_address_kappas: [String; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlphaPartitionRecord {
    transition_count: usize,
    construction_row_count: usize,
    candidate_count: usize,
    transitions: Vec<AlphaPartitionTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AlphaSupportReport {
    manifest_kappa: String,
    query_policy_identity: String,
    query_policy_kappa: String,
    fallback_active: bool,
    rows_read: Vec<CanonicalAttentionRowRead>,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    support_admission: &'static str,
    candidates: Vec<CanonicalAttentionSupportCandidate>,
}

#[derive(Debug, Clone, Serialize)]
struct IncrementalReproductionReport {
    query_count: usize,
    full_vs_incremental_fresh_support_byte_matches: usize,
    full_vs_incremental_frozen_support_byte_matches: usize,
    exact: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SelectionBlindExperimentRecord {
    schema: u32,
    identity: &'static str,
    fixture_partition_kappa: &'static str,
    codec_vocabulary_record_kappa: &'static str,
    construction_artifact_record_kappa: &'static str,
    mechanism_policy_kappa: &'static str,
    label_free_validation_input_kappa: &'static str,
    construction_partition_kappa: String,
    compiler_query_frame_kappa: String,
    construction_label_join_kappa: String,
    core_artifact: serde_json::Value,
    real: Gate0ArmRawRecord,
    negative_controls: Vec<Gate0ArmRawRecord>,
    anti_recall: OperativeAntiRecallReport,
    real_validation_work: ValidationWorkAggregate,
    all_arms_support_equal: bool,
    all_arms_declared_work_equal: bool,
    all_arms_raw_report_provenance_exact: bool,
    all_arms_class_lookup_shape_exact_and_equal: bool,
    all_arms_complete: bool,
    all_arms_source_provider_teacher_future_and_label_inputs_zero: bool,
    populated_padding_aliases: usize,
    count_only_last_anchor: CountOnlyLastAnchorReport,
    coherent_full_artifact_candidate_relabeling: CoherentRelabelRawReport,
    incremental_reproduction: IncrementalReproductionReport,
    selector_type_present: bool,
    validation_label_join_loads_observed: usize,
    validation_labels_loaded: bool,
    issue_953_fixture_loaded: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeterministicRebuildReport {
    complete_builds: usize,
    independent_complete_build_inputs: bool,
    codec_kappa_equal: bool,
    vocabulary_kappa_equal: bool,
    route_artifact_bytes_equal: bool,
    attention_manifest_kappa_equal: bool,
    construction_partition_bytes_equal: bool,
    frame_bytes_equal: bool,
    core_artifact_bytes_equal: bool,
    selection_blind_experiment_bytes_equal: bool,
    exact: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Gate0RawCensusRecord {
    schema: u32,
    identity: &'static str,
    experiment: SelectionBlindExperimentRecord,
    deterministic_rebuild: DeterministicRebuildReport,
    validation_label_join_status: &'static str,
    selector_status: &'static str,
}

struct RealGate0Basis {
    partition: ConstructionCausalReturnConstructionPartition,
    frame: ConstructionCausalReturnFrame,
    core: ConstructionCausalReturnV1,
    generic_map: Gate0GeometricClassMap,
    construction: Vec<PreparedConstruction>,
    validation: Vec<PreparedValidation>,
    construction_raw: Vec<ConstructionCausalReturnRawQuery>,
    validation_raw: Vec<ConstructionCausalReturnRawQuery>,
    observations:
        Vec<uor_r4_core::construction_causal_return_attention::ConstructionCausalReturnObservation>,
    observation_rows: Vec<Gate0ObservationRow>,
    query_rows: Vec<Gate0QueryCandidateRow>,
}

struct SelectionBlindExperiment {
    record: SelectionBlindExperimentRecord,
    arms: Vec<Gate0EvaluationArm>,
    coherent_arm: Gate0EvaluationArm,
    coherent_candidate_map: BTreeMap<String, String>,
    count_only: CountOnlyLastAnchorReport,
    route_artifact_bytes: Vec<u8>,
    attention_manifest_kappa: String,
    partition_bytes: Vec<u8>,
    frame_bytes: Vec<u8>,
    core_bytes: Vec<u8>,
}

struct Gate0RawCensus {
    record: Gate0RawCensusRecord,
    primary: SelectionBlindExperiment,
    replay: SelectionBlindExperiment,
    address_by_surface: BTreeMap<String, GeometricAddress>,
    raw_census_kappa: String,
}

fn arm_record_is_complete(record: &Gate0ArmRawRecord) -> bool {
    record.construction_rows == 24
        && record.map_kappa == record.reproduced_map_kappa
        && !record.raw_report_frame_kappa.is_empty()
        && record.raw_report_frame_and_control_match_arm
        && record.class_lookup_shape_exact
        && record.label_free_coverage.decision_count == 6
        && record.label_free_coverage.candidate_count == 12
        && record.construction_raw_queries.len() == 12
        && record.validation_raw_queries.len() == 6
        && record.construction_support_reports.len() == 12
        && record.validation_support_reports.len() == 6
        && record
            .construction_support_reports
            .iter()
            .chain(&record.validation_support_reports)
            .all(|support| {
                support.rows_read.len() == 7
                    && support.candidates.len() == 2
                    && support.unique_candidates_before_ceiling == 2
                    && support.candidate_ceiling >= 2
            })
}

fn canonical_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}

fn raw_query_values(queries: &[ConstructionCausalReturnRawQuery]) -> Vec<serde_json::Value> {
    queries
        .iter()
        .map(|query| canonical_value(&query.canonical_report()))
        .collect()
}

fn controlled_query_values(
    queries: &[ConstructionCausalReturnControlledRawQuery],
) -> Vec<serde_json::Value> {
    queries
        .iter()
        .map(|query| canonical_value(&query.canonical_report()))
        .collect()
}

fn canonical_support_report(support: &AttentionSupportTrace) -> CanonicalAttentionSupportReport {
    use uor_r4_core::prime_route_geometric_attention::{AttentionRowKey, AttentionRowSource};

    let rows_read = support
        .rows_read
        .iter()
        .map(|row| {
            let source = match row.source {
                AttentionRowSource::LastOne => "last_one",
                AttentionRowSource::LastTwo => "last_two",
                AttentionRowSource::OrderedSentence => "ordered_sentence",
                AttentionRowSource::Divisor => "divisor",
                AttentionRowSource::AdjacentSpin => "adjacent_spin",
            };
            let key = match &row.key {
                AttentionRowKey::LastOne(address) => CanonicalAttentionRowKey::LastOne {
                    address_kappa: address.canonical_kappa().unwrap(),
                },
                AttentionRowKey::LastTwo { previous, last } => CanonicalAttentionRowKey::LastTwo {
                    previous_address_kappa: previous.canonical_kappa().unwrap(),
                    last_address_kappa: last.canonical_kappa().unwrap(),
                },
                AttentionRowKey::LastTwoUnavailable => CanonicalAttentionRowKey::LastTwoUnavailable,
                AttentionRowKey::OrderedSentence(sentence_kappa) => {
                    CanonicalAttentionRowKey::OrderedSentence {
                        sentence_kappa: sentence_kappa.clone(),
                    }
                }
                AttentionRowKey::Divisor(prime) => CanonicalAttentionRowKey::Divisor {
                    prime: prime.value(),
                },
                AttentionRowKey::AdjacentSpin(sector) => CanonicalAttentionRowKey::AdjacentSpin {
                    hopf_octant: sector.hopf_octant,
                    torsion_bin: sector.torsion_bin,
                },
            };
            CanonicalAttentionRowRead {
                slot_index: row.slot_index,
                source,
                key,
                consulted: row.consulted,
                hit: row.hit,
                physical_row_present: row.physical_row_present,
                fallback_active: row.fallback_active,
                candidate_entries_available: row.candidate_entries_available,
                candidate_entries_examined: row.candidate_entries_examined,
                candidate_entries_admitted: row.candidate_entries_admitted,
            }
        })
        .collect::<Vec<_>>();
    let candidates = support
        .candidates
        .iter()
        .map(|candidate| CanonicalAttentionSupportCandidate {
            candidate_address_kappa: candidate.next.canonical_kappa().unwrap(),
            last_one: candidate.source_counts.last_one,
            last_two: candidate.source_counts.last_two,
            ordered_sentence: candidate.source_counts.ordered_sentence,
            divisor: candidate.source_counts.divisor,
            adjacent_spin: candidate.source_counts.adjacent_spin,
        })
        .collect::<Vec<_>>();
    let support_admission = "source_breadth_then_total_count_then_canonical_address";
    let support_kappa = record_kappa(&(
        &support.manifest_kappa,
        support.query_policy.identity(),
        &support.query_policy_kappa,
        support.fallback_active,
        &rows_read,
        support.candidate_entries_available,
        support.candidate_entries_examined,
        support.candidate_entries_admitted,
        support.candidate_entry_ceiling,
        support.unique_candidates_before_ceiling,
        support.candidate_ceiling,
        support_admission,
        &candidates,
    ));
    CanonicalAttentionSupportReport {
        manifest_kappa: support.manifest_kappa.clone(),
        query_policy_identity: support.query_policy.identity().to_owned(),
        query_policy_kappa: support.query_policy_kappa.clone(),
        fallback_active: support.fallback_active,
        rows_read,
        candidate_entries_available: support.candidate_entries_available,
        candidate_entries_examined: support.candidate_entries_examined,
        candidate_entries_admitted: support.candidate_entries_admitted,
        candidate_entry_ceiling: support.candidate_entry_ceiling,
        unique_candidates_before_ceiling: support.unique_candidates_before_ceiling,
        candidate_ceiling: support.candidate_ceiling,
        support_admission,
        candidates,
        support_kappa,
    }
}

fn real_support_reports(
    queries: &[ConstructionCausalReturnRawQuery],
) -> Vec<CanonicalAttentionSupportReport> {
    queries
        .iter()
        .map(|query| canonical_support_report(query.support()))
        .collect()
}

fn controlled_support_reports(
    queries: &[ConstructionCausalReturnControlledRawQuery],
) -> Vec<CanonicalAttentionSupportReport> {
    queries
        .iter()
        .map(|query| canonical_support_report(query.support()))
        .collect()
}

fn query_rows_from_real(
    decision_id: &str,
    raw: &ConstructionCausalReturnRawQuery,
) -> Vec<Gate0QueryCandidateRow> {
    raw.candidates()
        .iter()
        .map(|candidate| Gate0QueryCandidateRow::from_real(decision_id, candidate))
        .collect()
}

fn query_rows_from_controlled(
    decision_id: &str,
    raw: &ConstructionCausalReturnControlledRawQuery,
) -> Vec<Gate0QueryCandidateRow> {
    raw.candidates()
        .iter()
        .map(|candidate| Gate0QueryCandidateRow::from_controlled(decision_id, candidate))
        .collect()
}

fn support_candidate_kappas(raw: &ConstructionCausalReturnRawQuery) -> Vec<String> {
    let mut candidates = raw
        .candidates()
        .iter()
        .map(|candidate| candidate.candidate_address_kappa().to_owned())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
}

fn build_real_gate0_basis(bundle: &FrozenBundle) -> RealGate0Basis {
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let construction = prepared_construction(bundle);
    let validation = prepared_validation(bundle);
    let partition = construction_partition(&construction);
    let frame = construction_frame(bundle, &partition);

    let mut construction_raw = Vec::with_capacity(construction.len());
    let mut observations = Vec::with_capacity(construction.len() * 2);
    for row in &construction {
        let raw = frame
            .raw_query(&bundle.attention, &row.predecessor_history, &table)
            .unwrap();
        let mut expected = row
            .candidate_union
            .iter()
            .map(|candidate| candidate.canonical_kappa().unwrap())
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(
            support_candidate_kappas(&raw),
            expected,
            "{} support",
            row.id
        );
        observations.extend(
            partition
                .authorize_label_join(&raw, row.id, &row.observed_next)
                .unwrap(),
        );
        construction_raw.push(raw);
    }

    let mut validation_raw = Vec::with_capacity(validation.len());
    let mut query_rows = Vec::with_capacity(validation.len() * 2);
    for row in &validation {
        let raw = frame
            .raw_query(&bundle.attention, &row.observed_history, &table)
            .unwrap();
        let mut expected = row
            .input
            .candidates
            .iter()
            .map(|surface| {
                address_for_surface(bundle, surface)
                    .canonical_kappa()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(
            support_candidate_kappas(&raw),
            expected,
            "{} support",
            row.input.id
        );
        query_rows.extend(query_rows_from_real(row.input.id, &raw));
        validation_raw.push(raw);
    }

    let observation_rows = observations
        .iter()
        .map(Gate0ObservationRow::from_real)
        .collect::<Vec<_>>();
    let generic_map = Gate0GeometricClassMap::compile(&observation_rows).unwrap();
    let core = ConstructionCausalReturnV1::compile(frame.clone(), &table, &observations).unwrap();
    assert_eq!(core.construction_rows(), generic_map.observation_rows);
    assert_eq!(
        core.reproduce_artifact_kappa().unwrap(),
        core.artifact_kappa()
    );
    assert_eq!(
        generic_map.reproduce_artifact_kappa().unwrap(),
        generic_map.artifact_kappa
    );
    for raw in construction_raw.iter().chain(&validation_raw) {
        for candidate in raw.candidates() {
            let core_lookup = core.lookup_action(candidate.representation()).unwrap();
            let generic_lookup = generic_map
                .lookup(&Gate0RepresentationKeys::from_real(
                    candidate.representation(),
                ))
                .lookup;
            assert_eq!(
                serde_json::to_value(core_lookup).unwrap(),
                serde_json::to_value(generic_lookup).unwrap(),
                "core and selection-blind generic map diverged"
            );
        }
    }

    RealGate0Basis {
        partition,
        frame,
        core,
        generic_map,
        construction,
        validation,
        construction_raw,
        validation_raw,
        observations,
        observation_rows,
        query_rows,
    }
}

fn assert_exact_policy_orientation_and_prefix_boundary(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
) {
    let policy = construction_causal_return_policy_report();
    assert_eq!(
        policy.h4_product_orientation,
        "row-major left * right; quaternion basis (1,i,j,k); right-handed"
    );
    assert_eq!(policy.prefix_formula, "P_0=identity; P_i=P_{i-1}*L(x_i)");
    assert_eq!(policy.suffix_formula, "S_i=P_i^-1*P_t");
    assert_eq!(policy.relation_formula, "R_i=((S_i*L(c))*S_i^-1)*L(c)^-1");
    assert_eq!(policy.excluded_prefix, "i=t");
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let pairs = real
        .construction
        .iter()
        .map(|row| row.predecessor_history.as_slice())
        .zip(&real.construction_raw)
        .chain(
            real.validation
                .iter()
                .map(|row| row.observed_history.as_slice())
                .zip(&real.validation_raw),
        );
    for (history, raw) in pairs {
        let path = bundle
            .attention
            .causal_path_state_from_history(history, &table)
            .unwrap();
        assert_eq!(path.prefix_states().len(), history.len() + 1);
        for candidate in raw.candidates() {
            let representation = candidate.representation();
            assert_eq!(usize::from(representation.observed_routes()), history.len());
            assert_eq!(
                representation
                    .slots()
                    .iter()
                    .filter(|slot| slot.class_event.occupied)
                    .count(),
                history.len()
            );
            for (index, slot) in representation.slots().iter().enumerate() {
                assert_eq!(usize::from(slot.slot_index), index);
                if index < history.len() {
                    // Slot i is exactly P_i for 0 <= i < t. P_t exists only
                    // as the observed endpoint and is intentionally excluded.
                    assert_eq!(slot.prefix_state, path.prefix_states()[index]);
                    assert_eq!(
                        slot.class_event.observed_lease_age,
                        u8::try_from(history.len() - index).unwrap()
                    );
                    assert_eq!(slot.prefix_state, path.prefix_states()[index]);
                } else {
                    assert!(!slot.class_event.occupied);
                }
            }
        }
    }
}

fn controlled_typed_aliases(raw: &ConstructionCausalReturnControlledRawQuery) -> usize {
    raw.candidates()
        .iter()
        .map(|candidate| {
            let slots = candidate.representation().slots();
            slots
                .iter()
                .filter(|slot| slot.kind == ConstructionCausalReturnControlledSlotKind::Operative)
                .flat_map(|operative| {
                    slots
                        .iter()
                        .filter(|slot| {
                            slot.kind == ConstructionCausalReturnControlledSlotKind::Padding
                        })
                        .map(move |padding| (operative, padding))
                })
                .filter(|(operative, padding)| {
                    operative.witness.class_event == padding.witness.class_event
                })
                .count()
        })
        .sum()
}

fn real_typed_aliases(raw: &ConstructionCausalReturnRawQuery) -> usize {
    raw.populated_padding_aliases()
}

fn aggregate_validation_work(
    queries: &[ConstructionCausalReturnRawQuery],
    coverage: &Gate0StructuralCoverageReport,
) -> ValidationWorkAggregate {
    let mut aggregate = ValidationWorkAggregate {
        decisions: queries.len(),
        support_rows_read: 0,
        relation_slots: 0,
        declared_prototype_class_slot_reads: 0,
        performed_prototype_class_slot_reads: 0,
        declared_payload_inversions: 0,
        performed_payload_inversions: 0,
        source_inputs: 0,
        provider_inputs: 0,
        teacher_inputs: 0,
        future_route_inputs: 0,
        validation_label_inputs: 0,
    };
    for query in queries {
        let work = query.work();
        aggregate.support_rows_read += work.support_rows_read;
        aggregate.relation_slots += work.relation_slots;
        aggregate.declared_prototype_class_slot_reads += work.declared_prototype_class_slots;
        aggregate.performed_prototype_class_slot_reads += work.performed_prototype_class_slot_reads;
        aggregate.declared_payload_inversions += work.declared_payload_inversions;
        aggregate.performed_payload_inversions += work.performed_payload_inversions;
        aggregate.source_inputs += work.source_inputs;
        aggregate.provider_inputs += work.provider_inputs;
        aggregate.teacher_inputs += work.teacher_inputs;
        aggregate.future_route_inputs += work.future_route_inputs;
        aggregate.validation_label_inputs += work.validation_label_inputs;
    }
    assert_eq!(
        aggregate.declared_prototype_class_slot_reads,
        coverage.declared_class_reads
    );
    aggregate.performed_prototype_class_slot_reads = coverage.performed_class_reads;
    aggregate
}

fn work_is_source_free(work: ConstructionCausalReturnWorkReport) -> bool {
    work.source_inputs == 0
        && work.provider_inputs == 0
        && work.teacher_inputs == 0
        && work.future_route_inputs == 0
        && work.validation_label_inputs == 0
}

fn raw_report_provenance(
    negative_control: Option<ConstructionCausalReturnNegativeControl>,
    construction_raw_queries: &[serde_json::Value],
    validation_raw_queries: &[serde_json::Value],
) -> (String, bool) {
    let expected_control = negative_control.map(|control| canonical_value(&control));
    let mut frame_kappa = None::<String>;
    let mut exact = true;
    for (expected_population, reports) in [
        ("construction", construction_raw_queries),
        ("validation", validation_raw_queries),
    ] {
        for report in reports {
            let Some(report_frame) = report
                .get("frame_kappa")
                .and_then(serde_json::Value::as_str)
            else {
                exact = false;
                continue;
            };
            match &frame_kappa {
                Some(expected) => exact &= expected == report_frame,
                None => frame_kappa = Some(report_frame.to_owned()),
            }
            if let Some(control) = report.get("control") {
                exact &= expected_control
                    .as_ref()
                    .is_some_and(|expected| expected == control);
                exact &= report
                    .get("population_role")
                    .and_then(serde_json::Value::as_str)
                    == Some(expected_population);
            }
        }
    }
    (
        frame_kappa.unwrap_or_else(|| "UNAVAILABLE_NO_RAW_REPORT".to_owned()),
        exact,
    )
}

fn class_lookup_shape_exact(coverage: &Gate0StructuralCoverageReport) -> bool {
    let ledgers = coverage
        .decisions
        .iter()
        .flat_map(|decision| &decision.candidates)
        .map(|candidate| candidate.class_reads)
        .collect::<Vec<_>>();
    !ledgers.is_empty()
        && ledgers.len() == coverage.candidate_count
        && coverage.declared_class_reads == ledgers.len() * 2
        && coverage.performed_class_reads == ledgers.len() * 2
        && ledgers.iter().all(|reads| {
            reads.lookup_shape_identity == GATE0_CLASS_LOOKUP_SHAPE_IDENTITY
                && reads.unified_typed_slot_table
                && reads.declared_class_reads == 2
                && reads.performed_class_reads == 2
                && reads.minimum_or_exact_reads == 1
                && reads.rich_or_typed_noop_reads == 1
        })
}

fn raw_work_values(record: &Gate0ArmRawRecord) -> Option<Vec<serde_json::Value>> {
    record
        .construction_raw_queries
        .iter()
        .chain(&record.validation_raw_queries)
        .map(|report| report.get("work").cloned())
        .collect()
}

fn class_lookup_shape_values(record: &Gate0ArmRawRecord) -> Vec<serde_json::Value> {
    record
        .label_free_coverage
        .decisions
        .iter()
        .flat_map(|decision| &decision.candidates)
        .map(|candidate| canonical_value(&candidate.class_reads))
        .collect()
}

fn set_arm_equalities_from_serialized_reports(
    record: &mut Gate0ArmRawRecord,
    real: &Gate0ArmRawRecord,
) {
    record.support_equal_to_real = record.support_reports_kappa == real.support_reports_kappa
        && serde_json::to_vec(&(
            &record.construction_support_reports,
            &record.validation_support_reports,
        ))
        .unwrap()
            == serde_json::to_vec(&(
                &real.construction_support_reports,
                &real.validation_support_reports,
            ))
            .unwrap();
    record.declared_work_equal_to_real =
        raw_work_values(record).is_some() && raw_work_values(record) == raw_work_values(real);
    record.class_lookup_shape_equal_to_real =
        class_lookup_shape_values(record) == class_lookup_shape_values(real);
}

// The constructor mirrors the frozen arm-record schema field for field.
#[allow(clippy::too_many_arguments)]
fn gate0_arm_record(
    name: &'static str,
    negative_control: Option<ConstructionCausalReturnNegativeControl>,
    raw_basis: &'static str,
    map: &FrozenGate0Map,
    construction_observation_rows: &[Gate0ObservationRow],
    queries: &[Gate0QueryCandidateRow],
    construction_raw_queries: Vec<serde_json::Value>,
    validation_raw_queries: Vec<serde_json::Value>,
    construction_support_reports: Vec<CanonicalAttentionSupportReport>,
    validation_support_reports: Vec<CanonicalAttentionSupportReport>,
    transformation: Option<serde_json::Value>,
    typed_populated_padding_aliases: usize,
) -> Gate0ArmRawRecord {
    let (promoted_minimum_classes, promoted_rows, promotion_denominator) = map.promotion();
    let support_reports_kappa =
        record_kappa(&(&construction_support_reports, &validation_support_reports));
    let label_free_coverage = map.coverage(queries);
    let class_lookup_shape_exact = class_lookup_shape_exact(&label_free_coverage);
    let (raw_report_frame_kappa, raw_report_frame_and_control_match_arm) = raw_report_provenance(
        negative_control,
        &construction_raw_queries,
        &validation_raw_queries,
    );
    Gate0ArmRawRecord {
        name,
        negative_control,
        raw_basis,
        map_kind: map.kind(),
        map_kappa: map.map_kappa().to_owned(),
        reproduced_map_kappa: map.reproduce_kappa(),
        construction_observation_rows_kappa: record_kappa(&construction_observation_rows),
        construction_rows: map.construction_rows(),
        all_selection_classes_pure: map.all_selection_classes_pure(),
        promoted_minimum_classes,
        promoted_rows,
        promotion_denominator,
        inventory: map.inventory_value(),
        label_free_coverage,
        construction_raw_queries,
        validation_raw_queries,
        construction_support_reports,
        validation_support_reports,
        support_reports_kappa,
        transformation,
        raw_report_frame_kappa,
        raw_report_frame_and_control_match_arm,
        support_equal_to_real: false,
        declared_work_equal_to_real: false,
        class_lookup_shape_exact,
        class_lookup_shape_equal_to_real: false,
        typed_populated_padding_aliases,
    }
}

fn operative_anti_recall(real: &RealGate0Basis) -> OperativeAntiRecallReport {
    let construction_histories = real
        .construction_raw
        .iter()
        .map(|query| query.observed_history_kappa().to_owned())
        .collect::<BTreeSet<_>>();
    let validation_histories = real
        .validation_raw
        .iter()
        .map(|query| query.observed_history_kappa().to_owned())
        .collect::<BTreeSet<_>>();
    let exact_raw_history_overlaps = construction_histories
        .intersection(&validation_histories)
        .count();

    let construction_suffixes = real
        .construction
        .iter()
        .map(|row| {
            record_kappa(
                &row.predecessor_history[row.predecessor_history.len() - 4..]
                    .iter()
                    .map(|address| address.canonical_kappa().unwrap())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeSet<_>>();
    let validation_suffixes = real
        .validation
        .iter()
        .map(|row| {
            record_kappa(
                &row.observed_history[row.observed_history.len() - 4..]
                    .iter()
                    .map(|address| address.canonical_kappa().unwrap())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeSet<_>>();
    let exact_trailing_four_suffix_overlaps = construction_suffixes
        .intersection(&validation_suffixes)
        .count();

    let construction_witnesses = real
        .construction_raw
        .iter()
        .flat_map(|query| query.candidates())
        .map(|candidate| record_kappa(candidate.representation().slots()))
        .collect::<BTreeSet<_>>();
    let validation_witnesses = real
        .validation_raw
        .iter()
        .flat_map(|query| query.candidates())
        .map(|candidate| record_kappa(candidate.representation().slots()))
        .collect::<BTreeSet<_>>();
    let exact_ordered_route_witness_overlaps = construction_witnesses
        .intersection(&validation_witnesses)
        .count();

    let construction_representations = real
        .construction_raw
        .iter()
        .flat_map(|query| query.candidates())
        .map(|candidate| record_kappa(candidate.representation()))
        .collect::<BTreeSet<_>>();
    let validation_representations = real
        .validation_raw
        .iter()
        .flat_map(|query| query.candidates())
        .map(|candidate| record_kappa(candidate.representation()))
        .collect::<BTreeSet<_>>();
    let exact_complete_candidate_representation_overlaps = construction_representations
        .intersection(&validation_representations)
        .count();
    let operative_raw_prototype_recall = exact_raw_history_overlaps != 0
        || exact_trailing_four_suffix_overlaps != 0
        || exact_ordered_route_witness_overlaps != 0
        || exact_complete_candidate_representation_overlaps != 0;

    OperativeAntiRecallReport {
        construction_histories: construction_histories.len(),
        validation_histories: validation_histories.len(),
        exact_raw_history_overlaps,
        exact_trailing_four_suffix_overlaps,
        exact_ordered_route_witness_overlaps,
        exact_complete_candidate_representation_overlaps,
        operative_raw_prototype_recall,
    }
}

fn count_only_last_anchor(real: &RealGate0Basis) -> CountOnlyLastAnchorReport {
    let mut counts = BTreeMap::<String, BTreeMap<String, usize>>::new();
    for row in &real.construction {
        let anchor = row
            .predecessor_history
            .last()
            .unwrap()
            .canonical_kappa()
            .unwrap();
        let candidate = row.observed_next.canonical_kappa().unwrap();
        *counts
            .entry(anchor)
            .or_default()
            .entry(candidate)
            .or_default() += 1;
    }
    let classes = counts
        .iter()
        .map(
            |(anchor_address_kappa, candidate_counts)| CountOnlyAnchorClass {
                anchor_address_kappa: anchor_address_kappa.clone(),
                candidate_counts: candidate_counts.clone(),
            },
        )
        .collect::<Vec<_>>();
    let validation = real
        .validation
        .iter()
        .zip(&real.validation_raw)
        .map(|(row, raw)| {
            let anchor_address_kappa = row
                .observed_history
                .last()
                .unwrap()
                .canonical_kappa()
                .unwrap();
            let candidate_counts = counts.get(&anchor_address_kappa).unwrap();
            let mut candidate_address_kappas = support_candidate_kappas(raw);
            candidate_address_kappas.sort();
            let maximum = candidate_address_kappas
                .iter()
                .map(|candidate| candidate_counts.get(candidate).copied().unwrap_or(0))
                .max()
                .unwrap();
            let winners = candidate_address_kappas
                .iter()
                .filter(|candidate| {
                    candidate_counts.get(*candidate).copied().unwrap_or(0) == maximum
                })
                .cloned()
                .collect::<Vec<_>>();
            CountOnlyValidationDecision {
                decision_id: row.input.id.to_owned(),
                anchor_address_kappa,
                candidate_address_kappas,
                selected_candidate_address_kappa: (winners.len() == 1).then(|| winners[0].clone()),
                tied: winners.len() != 1,
            }
        })
        .collect();
    CountOnlyLastAnchorReport {
        identity: "uor-r4.count-only-last-anchor-comparator/1",
        construction_condition: "exact last observed anchor route",
        selection_rule: "select only a unique maximum construction count; ties abstain",
        classes,
        validation,
        admits_candidates: false,
        supplies_geometric_selector: false,
        attention_claim: false,
    }
}

fn incremental_reproduction(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
) -> IncrementalReproductionReport {
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let pairs = real
        .construction
        .iter()
        .map(|row| row.predecessor_history.as_slice())
        .zip(&real.construction_raw)
        .chain(
            real.validation
                .iter()
                .map(|row| row.observed_history.as_slice())
                .zip(&real.validation_raw),
        )
        .collect::<Vec<_>>();
    let mut fresh_matches = 0usize;
    let mut frozen_matches = 0usize;
    for (history, raw) in &pairs {
        let mut incremental = bundle
            .attention
            .causal_path_state_from_history(&history[..1], &table)
            .unwrap();
        for observed in &history[1..] {
            bundle
                .attention
                .observe_path(&mut incremental, observed.clone(), &table)
                .unwrap();
        }
        let from_incremental = real
            .frame
            .raw_query_from_path_state(&bundle.attention, history, &incremental, &table)
            .unwrap();
        let from_incremental_frozen = real
            .frame
            .raw_query_from_path_state_and_frozen_raw(raw, history, &incremental, &table)
            .unwrap();
        if from_incremental.canonical_bytes().unwrap() == raw.canonical_bytes().unwrap() {
            fresh_matches += 1;
        }
        if from_incremental_frozen.canonical_bytes().unwrap() == raw.canonical_bytes().unwrap() {
            frozen_matches += 1;
        }
    }
    IncrementalReproductionReport {
        query_count: pairs.len(),
        full_vs_incremental_fresh_support_byte_matches: fresh_matches,
        full_vs_incremental_frozen_support_byte_matches: frozen_matches,
        exact: fresh_matches == pairs.len() && frozen_matches == pairs.len(),
    }
}

fn candidate_relabel_map(bundle: &FrozenBundle) -> BTreeMap<String, String> {
    let mut mapping = BTreeMap::new();
    for family in ["is-are", "has-have", "was-were"] {
        let candidates = family_candidate_surfaces(family);
        let left = address_for_surface(bundle, candidates[0])
            .canonical_kappa()
            .unwrap();
        let right = address_for_surface(bundle, candidates[1])
            .canonical_kappa()
            .unwrap();
        mapping.insert(left.clone(), right.clone());
        mapping.insert(right, left);
    }
    mapping
}

fn alpha_support_report(report: &CanonicalAttentionSupportReport) -> AlphaSupportReport {
    AlphaSupportReport {
        manifest_kappa: report.manifest_kappa.clone(),
        query_policy_identity: report.query_policy_identity.clone(),
        query_policy_kappa: report.query_policy_kappa.clone(),
        fallback_active: report.fallback_active,
        rows_read: report.rows_read.clone(),
        candidate_entries_available: report.candidate_entries_available,
        candidate_entries_examined: report.candidate_entries_examined,
        candidate_entries_admitted: report.candidate_entries_admitted,
        candidate_entry_ceiling: report.candidate_entry_ceiling,
        unique_candidates_before_ceiling: report.unique_candidates_before_ceiling,
        candidate_ceiling: report.candidate_ceiling,
        support_admission: report.support_admission,
        candidates: report.candidates.clone(),
    }
}

fn rename_candidate_identity(
    identity: &mut String,
    mapping: &BTreeMap<String, String>,
) -> (usize, usize) {
    match mapping.get(identity) {
        Some(relabeled) => {
            let changed = usize::from(relabeled != identity);
            *identity = relabeled.clone();
            (1, changed)
        }
        None => (0, 0),
    }
}

fn assert_no_candidate_in_row_key(
    key: &CanonicalAttentionRowKey,
    mapping: &BTreeMap<String, String>,
) {
    let absent = match key {
        CanonicalAttentionRowKey::LastOne { address_kappa } => !mapping.contains_key(address_kappa),
        CanonicalAttentionRowKey::LastTwo {
            previous_address_kappa,
            last_address_kappa,
        } => {
            !mapping.contains_key(previous_address_kappa)
                && !mapping.contains_key(last_address_kappa)
        }
        CanonicalAttentionRowKey::LastTwoUnavailable
        | CanonicalAttentionRowKey::OrderedSentence { .. }
        | CanonicalAttentionRowKey::Divisor { .. }
        | CanonicalAttentionRowKey::AdjacentSpin { .. } => true,
    };
    assert!(absent, "candidate identity entered an observed support key");
}

fn typed_alpha_relabel(
    source: &AlphaQualificationRecord,
    mapping: &BTreeMap<String, String>,
) -> (AlphaQualificationRecord, usize, usize) {
    let mut relabeled = source.clone();
    let mut occurrences = 0usize;
    let mut changed = 0usize;
    for transition in &mut relabeled.partition.transitions {
        assert!(transition
            .predecessor_address_kappas
            .iter()
            .all(|address| !mapping.contains_key(address)));
        let counts =
            rename_candidate_identity(&mut transition.observed_next_address_kappa, mapping);
        occurrences += counts.0;
        changed += counts.1;
        for candidate in &mut transition.candidate_union_address_kappas {
            let counts = rename_candidate_identity(candidate, mapping);
            occurrences += counts.0;
            changed += counts.1;
        }
        transition.candidate_union_address_kappas.sort();
    }
    for support in relabeled
        .construction_support_reports
        .iter_mut()
        .chain(&mut relabeled.validation_support_reports)
    {
        for row in &support.rows_read {
            assert_no_candidate_in_row_key(&row.key, mapping);
        }
        for candidate in &mut support.candidates {
            let counts = rename_candidate_identity(&mut candidate.candidate_address_kappa, mapping);
            occurrences += counts.0;
            changed += counts.1;
        }
        support.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    for raw in relabeled
        .construction_raw_queries
        .iter_mut()
        .chain(&mut relabeled.validation_raw_queries)
    {
        assert!(raw
            .observed_history_address_kappas
            .iter()
            .all(|address| !mapping.contains_key(address)));
        for candidate in &mut raw.candidates {
            let counts = rename_candidate_identity(&mut candidate.candidate_address_kappa, mapping);
            occurrences += counts.0;
            changed += counts.1;
        }
        raw.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    for row in &mut relabeled.construction_observation_rows {
        let counts = rename_candidate_identity(&mut row.candidate_address_kappa, mapping);
        occurrences += counts.0;
        changed += counts.1;
    }
    relabeled.construction_observation_rows.sort();
    for row in &mut relabeled.validation_query_rows {
        let counts = rename_candidate_identity(&mut row.candidate_address_kappa, mapping);
        occurrences += counts.0;
        changed += counts.1;
    }
    relabeled.validation_query_rows.sort();
    for commitment in &mut relabeled.surface_commitments {
        let counts = rename_candidate_identity(&mut commitment.candidate_address_kappa, mapping);
        occurrences += counts.0;
        changed += counts.1;
    }
    relabeled.surface_commitments.sort_by(|left, right| {
        left.candidate_address_kappa
            .cmp(&right.candidate_address_kappa)
    });
    let compiled_map = Gate0GeometricClassMap::compile(&relabeled.construction_observation_rows)
        .expect("typed alpha-renaming must retain a compilable construction map");
    relabeled.compiled_class_map_domain = compiled_map.domain;
    relabeled.map_kappa = compiled_map.artifact_kappa.clone();
    relabeled.reproduced_map_kappa = compiled_map
        .reproduce_artifact_kappa()
        .expect("typed alpha-renamed map must reproduce");
    relabeled.structural_coverage =
        label_free_structural_coverage(&compiled_map, &relabeled.validation_query_rows)
            .expect("typed alpha-renamed validation rows must remain structurally auditable");
    (relabeled, occurrences, changed)
}

fn alpha_partition_corresponds(
    source: &AlphaPartitionRecord,
    relabeled: &AlphaPartitionRecord,
    mapping: &BTreeMap<String, String>,
) -> bool {
    let mut expected = source.clone();
    for transition in &mut expected.transitions {
        let Some(observed_next) = mapping.get(&transition.observed_next_address_kappa) else {
            return false;
        };
        transition.observed_next_address_kappa = observed_next.clone();
        for candidate in &mut transition.candidate_union_address_kappas {
            let Some(mapped) = mapping.get(candidate) else {
                return false;
            };
            *candidate = mapped.clone();
        }
        transition.candidate_union_address_kappas.sort();
    }
    expected == *relabeled
}

fn alpha_support_corresponds(
    source: &[AlphaSupportReport],
    relabeled: &[AlphaSupportReport],
    mapping: &BTreeMap<String, String>,
) -> bool {
    source.len() == relabeled.len()
        && source.iter().zip(relabeled).all(|(source, relabeled)| {
            let mut expected = source.clone();
            for candidate in &mut expected.candidates {
                let Some(mapped) = mapping.get(&candidate.candidate_address_kappa) else {
                    return false;
                };
                candidate.candidate_address_kappa = mapped.clone();
            }
            expected.candidates.sort_by(|left, right| {
                left.candidate_address_kappa
                    .cmp(&right.candidate_address_kappa)
            });
            expected == *relabeled
        })
}

fn alpha_raw_corresponds(
    source: &[ConstructionCausalReturnRawQueryReport],
    relabeled: &[ConstructionCausalReturnRawQueryReport],
    mapping: &BTreeMap<String, String>,
) -> bool {
    source.len() == relabeled.len()
        && source.iter().zip(relabeled).all(|(source, relabeled)| {
            let mut expected = source.clone();
            for candidate in &mut expected.candidates {
                let Some(mapped) = mapping.get(&candidate.candidate_address_kappa) else {
                    return false;
                };
                candidate.candidate_address_kappa = mapped.clone();
            }
            expected.candidates.sort_by(|left, right| {
                left.candidate_address_kappa
                    .cmp(&right.candidate_address_kappa)
            });
            expected == *relabeled
        })
}

fn alpha_structural_coverage_corresponds(
    source: &Gate0StructuralCoverageReport,
    relabeled: &Gate0StructuralCoverageReport,
    mapping: &BTreeMap<String, String>,
) -> bool {
    let mut expected = source.clone();
    for decision in &mut expected.decisions {
        for candidate in &mut decision.candidates {
            let Some(mapped) = mapping.get(&candidate.candidate_address_kappa) else {
                return false;
            };
            candidate.candidate_address_kappa = mapped.clone();
        }
        decision.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    // The target kappa is independently recomputed by the support helper from
    // the renamed candidate rows; all other fields are compared exactly.
    expected.coverage_kappa = relabeled.coverage_kappa.clone();
    !relabeled.coverage_kappa.is_empty() && expected == *relabeled
}

fn coherent_relabel(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
) -> (
    Gate0EvaluationArm,
    BTreeMap<String, String>,
    CoherentRelabelRawReport,
) {
    let fresh = fresh_real_raw_arm(bundle, real);
    let mapping = candidate_relabel_map(bundle);
    let relabel = |candidate: &str| mapping.get(candidate).unwrap().clone();
    let mut construction_rows = fresh
        .observation_rows
        .iter()
        .cloned()
        .map(|mut row| {
            row.candidate_address_kappa = relabel(&row.candidate_address_kappa);
            row
        })
        .collect::<Vec<_>>();
    construction_rows.sort();
    let mut query_rows = fresh
        .query_rows
        .iter()
        .cloned()
        .map(|mut row| {
            row.candidate_address_kappa = relabel(&row.candidate_address_kappa);
            row
        })
        .collect::<Vec<_>>();
    query_rows.sort();
    let source_map = Gate0GeometricClassMap::compile(&fresh.observation_rows).unwrap();
    assert_eq!(source_map.artifact_kappa, real.generic_map.artifact_kappa);
    let map = Gate0GeometricClassMap::compile(&construction_rows).unwrap();
    let original_coverage = label_free_structural_coverage(&source_map, &fresh.query_rows).unwrap();
    let relabelled_coverage = label_free_structural_coverage(&map, &query_rows).unwrap();
    let entries = mapping
        .iter()
        .map(|(source, target)| CoherentRelabelEntry {
            source_candidate_address_kappa: source.clone(),
            relabeled_candidate_address_kappa: target.clone(),
            source_registered_surface_commitment: bundle
                .surface_commitment_by_address_kappa
                .get(source)
                .unwrap()
                .clone(),
            registered_surface_at_relabeled_candidate_before: bundle
                .surface_commitment_by_address_kappa
                .get(target)
                .unwrap()
                .clone(),
            // Full-artifact alpha-renaming moves the already-frozen surface
            // association commitment. No route-value payload is inverted.
            registered_surface_at_relabeled_candidate_after: bundle
                .surface_commitment_by_address_kappa
                .get(source)
                .unwrap()
                .clone(),
        })
        .collect::<Vec<_>>();
    let candidate_mapping_bijective = mapping.len() == 6
        && mapping.values().cloned().collect::<BTreeSet<_>>().len() == mapping.len()
        && mapping.iter().all(|(source, target)| {
            source != target && mapping.get(target).is_some_and(|back| back == source)
        });
    let mut expected_construction_rows = fresh.observation_rows.clone();
    for row in &mut expected_construction_rows {
        row.candidate_address_kappa = relabel(&row.candidate_address_kappa);
    }
    expected_construction_rows.sort();
    let exact_construction_correspondence = expected_construction_rows == construction_rows;
    let mut expected_query_rows = fresh.query_rows.clone();
    for row in &mut expected_query_rows {
        row.candidate_address_kappa = relabel(&row.candidate_address_kappa);
    }
    expected_query_rows.sort();
    let exact_query_correspondence = expected_query_rows == query_rows;
    let exact_row_correspondence = exact_construction_correspondence
        && exact_query_correspondence
        && construction_rows.len() == fresh.observation_rows.len()
        && query_rows.len() == fresh.query_rows.len();
    let registered_surface_association_reproduced = entries.iter().all(|entry| {
        entry.registered_surface_at_relabeled_candidate_after
            == entry.source_registered_surface_commitment
    });
    let surface_commitments = mapping
        .keys()
        .map(|candidate_address_kappa| CandidateSurfaceCommitment {
            candidate_address_kappa: candidate_address_kappa.clone(),
            registered_surface: bundle
                .surface_commitment_by_address_kappa
                .get(candidate_address_kappa)
                .unwrap()
                .clone(),
        })
        .collect::<Vec<_>>();
    let partition_report = real.partition.canonical_report();
    let mut original_qualification_record = AlphaQualificationRecord {
        partition: AlphaPartitionRecord {
            transition_count: partition_report.transition_count,
            construction_row_count: partition_report.construction_row_count,
            candidate_count: partition_report.candidate_count,
            transitions: partition_report
                .transitions
                .iter()
                .map(|transition| AlphaPartitionTransition {
                    transition_id: transition.transition_id.clone(),
                    predecessor_history_kappa: transition.predecessor_history_kappa.clone(),
                    predecessor_address_kappas: transition.predecessor_address_kappas.clone(),
                    observed_next_address_kappa: transition.observed_next_address_kappa.clone(),
                    candidate_union_address_kappas: transition
                        .candidate_union_address_kappas
                        .clone(),
                })
                .collect(),
        },
        construction_support_reports: real_support_reports(&fresh.construction_raw)
            .iter()
            .map(alpha_support_report)
            .collect(),
        validation_support_reports: real_support_reports(&fresh.validation_raw)
            .iter()
            .map(alpha_support_report)
            .collect(),
        construction_raw_queries: fresh
            .construction_raw
            .iter()
            .map(ConstructionCausalReturnRawQuery::canonical_report)
            .collect(),
        validation_raw_queries: fresh
            .validation_raw
            .iter()
            .map(ConstructionCausalReturnRawQuery::canonical_report)
            .collect(),
        construction_observation_rows: fresh.observation_rows.clone(),
        validation_query_rows: fresh.query_rows.clone(),
        surface_commitments,
        compiled_class_map_domain: source_map.domain,
        map_kappa: source_map.artifact_kappa.clone(),
        reproduced_map_kappa: source_map.reproduce_artifact_kappa().unwrap(),
        structural_coverage: original_coverage.clone(),
    };
    for transition in &mut original_qualification_record.partition.transitions {
        transition.candidate_union_address_kappas.sort();
    }
    for support in original_qualification_record
        .construction_support_reports
        .iter_mut()
        .chain(&mut original_qualification_record.validation_support_reports)
    {
        support.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    for raw in original_qualification_record
        .construction_raw_queries
        .iter_mut()
        .chain(&mut original_qualification_record.validation_raw_queries)
    {
        raw.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    original_qualification_record
        .construction_observation_rows
        .sort();
    original_qualification_record.validation_query_rows.sort();
    original_qualification_record
        .surface_commitments
        .sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    let (relabeled_qualification_record, candidate_identity_occurrences, changed_once) =
        typed_alpha_relabel(&original_qualification_record, &mapping);
    let (twice_relabeled_qualification_record, occurrences_twice, changed_twice) =
        typed_alpha_relabel(&relabeled_qualification_record, &mapping);
    assert_eq!(
        relabeled_qualification_record.construction_observation_rows,
        construction_rows
    );
    assert_eq!(
        relabeled_qualification_record.validation_query_rows,
        query_rows
    );
    let expected_candidate_identity_occurrences =
        original_qualification_record.partition.transitions.len() * 3
            + original_qualification_record
                .construction_support_reports
                .iter()
                .chain(&original_qualification_record.validation_support_reports)
                .map(|report| report.candidates.len())
                .sum::<usize>()
            + original_qualification_record
                .construction_raw_queries
                .iter()
                .chain(&original_qualification_record.validation_raw_queries)
                .map(|report| report.candidates.len())
                .sum::<usize>()
            + original_qualification_record
                .construction_observation_rows
                .len()
            + original_qualification_record.validation_query_rows.len()
            + original_qualification_record.surface_commitments.len();
    let no_candidate_identity_occurrence_left_unmapped = candidate_identity_occurrences
        == expected_candidate_identity_occurrences
        && candidate_identity_occurrences == changed_once
        && occurrences_twice == expected_candidate_identity_occurrences
        && occurrences_twice == changed_twice;
    let original_qualification_bytes = serde_json::to_vec(&original_qualification_record).unwrap();
    let twice_relabeled_qualification_bytes =
        serde_json::to_vec(&twice_relabeled_qualification_record).unwrap();
    let involution_twice_reproduces_original_bytes =
        original_qualification_bytes == twice_relabeled_qualification_bytes;
    let original_qualification_record_kappa = record_kappa(&original_qualification_record);
    let relabeled_qualification_record_kappa = record_kappa(&relabeled_qualification_record);
    let twice_relabeled_qualification_record_kappa =
        record_kappa(&twice_relabeled_qualification_record);
    let partition_candidate_identity_correspondence = alpha_partition_corresponds(
        &original_qualification_record.partition,
        &relabeled_qualification_record.partition,
        &mapping,
    );
    let support_candidate_identity_and_counts_correspondence = alpha_support_corresponds(
        &original_qualification_record.construction_support_reports,
        &relabeled_qualification_record.construction_support_reports,
        &mapping,
    ) && alpha_support_corresponds(
        &original_qualification_record.validation_support_reports,
        &relabeled_qualification_record.validation_support_reports,
        &mapping,
    );
    let raw_candidate_identity_representation_and_work_correspondence = alpha_raw_corresponds(
        &original_qualification_record.construction_raw_queries,
        &relabeled_qualification_record.construction_raw_queries,
        &mapping,
    ) && alpha_raw_corresponds(
        &original_qualification_record.validation_raw_queries,
        &relabeled_qualification_record.validation_raw_queries,
        &mapping,
    );
    let construction_action_correspondence = exact_construction_correspondence;
    let validation_query_correspondence = exact_query_correspondence;
    let surface_commitment_correspondence = original_qualification_record
        .surface_commitments
        .iter()
        .all(|source| {
            relabeled_qualification_record
                .surface_commitments
                .iter()
                .any(|relabeled| {
                    relabeled.candidate_address_kappa == relabel(&source.candidate_address_kappa)
                        && relabeled.registered_surface == source.registered_surface
                })
        });
    let map_kappa_reproduced = original_qualification_record.map_kappa
        == original_qualification_record.reproduced_map_kappa
        && relabeled_qualification_record.map_kappa
            == relabeled_qualification_record.reproduced_map_kappa
        && relabeled_qualification_record.map_kappa == original_qualification_record.map_kappa
        && map.artifact_kappa == relabeled_qualification_record.map_kappa;
    let structural_coverage_reproduced = alpha_structural_coverage_corresponds(
        &original_qualification_record.structural_coverage,
        &relabeled_qualification_record.structural_coverage,
        &mapping,
    ) && relabelled_coverage
        == relabeled_qualification_record.structural_coverage;
    let native_codec_or_placement_rebuild = false;
    let validation_label_commitment_loaded =
        SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst) != 0;
    let qualification_record_alpha_equivariance_without_payload = !native_codec_or_placement_rebuild
        && !validation_label_commitment_loaded
        && candidate_mapping_bijective
        && partition_candidate_identity_correspondence
        && support_candidate_identity_and_counts_correspondence
        && raw_candidate_identity_representation_and_work_correspondence
        && construction_action_correspondence
        && validation_query_correspondence
        && surface_commitment_correspondence
        && exact_row_correspondence
        && registered_surface_association_reproduced
        && map_kappa_reproduced
        && structural_coverage_reproduced
        && no_candidate_identity_occurrence_left_unmapped
        && involution_twice_reproduces_original_bytes;
    // The complete Gate0 qualification artifact ends at the registered
    // address<->surface commitment. Native payload/CID association and exact
    // inversion belong only to the later selector stage and remain NOT_RUN.
    let complete_preselector_qualification_record_alpha_equivariance =
        qualification_record_alpha_equivariance_without_payload;
    let mut report = CoherentRelabelRawReport {
        identity: "uor-r4.coherent-full-artifact-candidate-relabeling/1",
        scope: "typed qualification-record alpha-renaming through partition, support, raw candidate reports, observation actions, validation queries, registered address-surface commitments, compiled class map, and label-free coverage; native codec/placement rebuild and payload association are outside this Gate0 record",
        native_codec_or_placement_rebuild,
        availability: "AVAILABLE_COMPLETE_GATE0_QUALIFICATION_ARTIFACT_ALPHA_EQUIVARIANCE",
        entries,
        source_construction_rows_kappa: record_kappa(&fresh.observation_rows),
        relabeled_construction_rows_kappa: record_kappa(&construction_rows),
        source_validation_support_rows_kappa: record_kappa(&fresh.query_rows),
        relabeled_validation_support_rows_kappa: record_kappa(&query_rows),
        construction_rows_relabelled: construction_rows.len(),
        validation_rows_relabelled: query_rows.len(),
        candidate_mapping_bijective,
        partition_candidate_identity_correspondence,
        support_candidate_identity_and_counts_correspondence,
        raw_candidate_identity_representation_and_work_correspondence,
        construction_action_correspondence,
        validation_query_correspondence,
        surface_commitment_correspondence,
        exact_row_correspondence,
        registered_surface_association_reproduced,
        map_kappa_reproduced,
        structural_coverage_reproduced,
        support_prototypes_surface_commitments_and_construction_actions_move_together:
            candidate_mapping_bijective
                && partition_candidate_identity_correspondence
                && support_candidate_identity_and_counts_correspondence
                && raw_candidate_identity_representation_and_work_correspondence
                && construction_action_correspondence
                && validation_query_correspondence
                && surface_commitment_correspondence
                && exact_row_correspondence
                && registered_surface_association_reproduced,
        original_qualification_record_kappa,
        relabeled_qualification_record_kappa,
        twice_relabeled_qualification_record_kappa,
        candidate_identity_occurrences,
        candidate_identity_occurrences_changed: changed_once,
        no_candidate_identity_occurrence_left_unmapped,
        involution_twice_reproduces_original_bytes,
        qualification_record_alpha_equivariance_without_payload,
        complete_preselector_qualification_record_alpha_equivariance,
        payload_association_status: "NOT_EXERCISED_SELECTOR_ONLY_NO_GATE0_CID_CLAIM",
        validation_label_commitment_loaded,
        report_kappa: String::new(),
    };
    report.report_kappa = record_kappa(&report);
    (
        Gate0EvaluationArm {
            name: "coherent_full_artifact_candidate_relabeling",
            map: FrozenGate0Map::Geometric(map),
            queries: query_rows,
        },
        mapping,
        report,
    )
}

struct FreshRealRawArm {
    construction_raw: Vec<ConstructionCausalReturnRawQuery>,
    validation_raw: Vec<ConstructionCausalReturnRawQuery>,
    observation_rows: Vec<Gate0ObservationRow>,
    query_rows: Vec<Gate0QueryCandidateRow>,
}

fn fresh_real_raw_arm(bundle: &FrozenBundle, real: &RealGate0Basis) -> FreshRealRawArm {
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let mut construction_raw = Vec::with_capacity(real.construction.len());
    let mut observation_rows = Vec::with_capacity(real.construction.len() * 2);
    for row in &real.construction {
        let raw = real
            .frame
            .raw_query(&bundle.attention, &row.predecessor_history, &table)
            .unwrap();
        assert_eq!(
            raw.support(),
            real.construction_raw[construction_raw.len()].support()
        );
        assert_eq!(
            raw.work(),
            real.construction_raw[construction_raw.len()].work()
        );
        observation_rows.extend(
            real.partition
                .authorize_label_join(&raw, row.id, &row.observed_next)
                .unwrap()
                .iter()
                .map(Gate0ObservationRow::from_real),
        );
        construction_raw.push(raw);
    }
    let mut validation_raw = Vec::with_capacity(real.validation.len());
    let mut query_rows = Vec::with_capacity(real.validation.len() * 2);
    for row in &real.validation {
        let raw = real
            .frame
            .raw_query(&bundle.attention, &row.observed_history, &table)
            .unwrap();
        assert_eq!(
            raw.support(),
            real.validation_raw[validation_raw.len()].support()
        );
        assert_eq!(raw.work(), real.validation_raw[validation_raw.len()].work());
        query_rows.extend(query_rows_from_real(row.input.id, &raw));
        validation_raw.push(raw);
    }
    FreshRealRawArm {
        construction_raw,
        validation_raw,
        observation_rows,
        query_rows,
    }
}

#[derive(Clone, Copy)]
enum ControlledMapKind {
    Geometric,
    ExactRecall,
    Unavailable,
}

fn controlled_arm(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
    name: &'static str,
    control: ConstructionCausalReturnNegativeControl,
    construction_encoders: Vec<ConstructionCausalReturnControlledEncoder>,
    validation_encoders: Vec<ConstructionCausalReturnControlledEncoder>,
    map_kind: ControlledMapKind,
) -> (Gate0EvaluationArm, Gate0ArmRawRecord) {
    assert_eq!(construction_encoders.len(), real.construction.len());
    assert_eq!(validation_encoders.len(), real.validation.len());
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let mut construction_raw = Vec::with_capacity(real.construction.len());
    let mut controlled_observations = Vec::with_capacity(real.construction.len() * 2);
    for (index, (row, encoder)) in real
        .construction
        .iter()
        .zip(&construction_encoders)
        .enumerate()
    {
        assert_eq!(encoder.control(), control);
        let raw = real
            .frame
            .controlled_raw_query(
                &bundle.attention,
                &row.predecessor_history,
                ConstructionCausalReturnPopulationRole::Construction,
                encoder,
                &table,
            )
            .unwrap();
        assert_eq!(raw.frame_kappa(), real.frame.frame_kappa());
        assert_eq!(raw.control(), control);
        assert_eq!(
            raw.population_role(),
            ConstructionCausalReturnPopulationRole::Construction
        );
        assert_eq!(raw.support(), real.construction_raw[index].support());
        assert_eq!(raw.work(), real.construction_raw[index].work());
        let report = raw.canonical_report();
        assert!(!report.support_reused_from_frozen_raw);
        assert_eq!(report.support_admission_queries_performed, 1);
        controlled_observations.extend(
            real.partition
                .authorize_controlled_label_join(&raw, row.id, &row.observed_next)
                .unwrap(),
        );
        construction_raw.push(raw);
    }

    let mut validation_raw = Vec::with_capacity(real.validation.len());
    let mut query_rows = Vec::with_capacity(real.validation.len() * 2);
    for (index, (row, encoder)) in real.validation.iter().zip(&validation_encoders).enumerate() {
        assert_eq!(encoder.control(), control);
        let raw = real
            .frame
            .controlled_raw_query(
                &bundle.attention,
                &row.observed_history,
                ConstructionCausalReturnPopulationRole::Validation,
                encoder,
                &table,
            )
            .unwrap();
        assert_eq!(raw.frame_kappa(), real.frame.frame_kappa());
        assert_eq!(raw.control(), control);
        assert_eq!(
            raw.population_role(),
            ConstructionCausalReturnPopulationRole::Validation
        );
        assert_eq!(raw.support(), real.validation_raw[index].support());
        assert_eq!(raw.work(), real.validation_raw[index].work());
        let report = raw.canonical_report();
        assert!(!report.support_reused_from_frozen_raw);
        assert_eq!(report.support_admission_queries_performed, 1);
        query_rows.extend(query_rows_from_controlled(row.input.id, &raw));
        validation_raw.push(raw);
    }

    let observation_rows = controlled_observations
        .iter()
        .map(Gate0ObservationRow::from_controlled)
        .collect::<Vec<_>>();
    if control == ConstructionCausalReturnNegativeControl::StateDisabled {
        assert!(observation_rows
            .iter()
            .all(|row| row.keys == Gate0RepresentationKeys::Unavailable));
        assert!(query_rows
            .iter()
            .all(|row| row.keys == Gate0RepresentationKeys::Unavailable));
    }
    let map = match map_kind {
        ControlledMapKind::Geometric => {
            FrozenGate0Map::Geometric(Gate0GeometricClassMap::compile(&observation_rows).unwrap())
        }
        ControlledMapKind::ExactRecall => {
            FrozenGate0Map::ExactRecall(Gate0ExactRecallMap::compile(&observation_rows).unwrap())
        }
        ControlledMapKind::Unavailable => FrozenGate0Map::Unavailable(
            Gate0UnavailableClassMap::compile(&controlled_observations).unwrap(),
        ),
    };
    let aliases = construction_raw
        .iter()
        .chain(&validation_raw)
        .map(controlled_typed_aliases)
        .sum();
    let record = gate0_arm_record(
        name,
        Some(control),
        "fresh natural admission followed by one controlled representation",
        &map,
        &observation_rows,
        &query_rows,
        controlled_query_values(&construction_raw),
        controlled_query_values(&validation_raw),
        controlled_support_reports(&construction_raw),
        controlled_support_reports(&validation_raw),
        None,
        aliases,
    );
    (
        Gate0EvaluationArm {
            name,
            map,
            queries: query_rows,
        },
        record,
    )
}

fn post_raw_arm(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
    name: &'static str,
    control: ConstructionCausalReturnNegativeControl,
    transform: impl FnOnce(
        &FreshRealRawArm,
    ) -> (
        FrozenGate0Map,
        Vec<Gate0ObservationRow>,
        Vec<Gate0QueryCandidateRow>,
        serde_json::Value,
    ),
) -> (Gate0EvaluationArm, Gate0ArmRawRecord) {
    let fresh = fresh_real_raw_arm(bundle, real);
    let (map, observation_rows, query_rows, transformation) = transform(&fresh);
    let aliases = fresh
        .construction_raw
        .iter()
        .chain(&fresh.validation_raw)
        .map(real_typed_aliases)
        .sum();
    let record = gate0_arm_record(
        name,
        Some(control),
        "fresh real natural admission followed by a frozen post-raw transformation",
        &map,
        &observation_rows,
        &query_rows,
        raw_query_values(&fresh.construction_raw),
        raw_query_values(&fresh.validation_raw),
        real_support_reports(&fresh.construction_raw),
        real_support_reports(&fresh.validation_raw),
        Some(transformation),
        aliases,
    );
    (
        Gate0EvaluationArm {
            name,
            map,
            queries: query_rows,
        },
        record,
    )
}

fn candidate_prototype_placement_arm(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
) -> (Gate0EvaluationArm, Gate0ArmRawRecord) {
    let name = "candidate_prototype_placement_permutation";
    let control = ConstructionCausalReturnNegativeControl::CandidatePrototypePlacementPermutation;
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let mut construction_raw = Vec::with_capacity(real.construction.len());
    let mut controlled_observations = Vec::with_capacity(real.construction.len() * 2);
    for (index, row) in real.construction.iter().enumerate() {
        let raw = real
            .frame
            .controlled_raw_query(
                &bundle.attention,
                &row.predecessor_history,
                ConstructionCausalReturnPopulationRole::Construction,
                &ConstructionCausalReturnControlledEncoder::CandidatePrototypePlacementPermutation,
                &table,
            )
            .unwrap();
        assert_eq!(raw.frame_kappa(), real.frame.frame_kappa());
        assert_eq!(raw.control(), control);
        assert_eq!(raw.support(), real.construction_raw[index].support());
        assert_eq!(raw.work(), real.construction_raw[index].work());
        let report = raw.canonical_report();
        assert!(!report.support_reused_from_frozen_raw);
        assert_eq!(report.support_admission_queries_performed, 1);
        controlled_observations.extend(
            real.partition
                .authorize_controlled_label_join(&raw, row.id, &row.observed_next)
                .unwrap(),
        );
        construction_raw.push(raw);
    }
    let fresh_validation = real
        .validation
        .iter()
        .map(|row| {
            real.frame
                .raw_query(&bundle.attention, &row.observed_history, &table)
                .unwrap()
        })
        .collect::<Vec<_>>();
    for (fresh, baseline) in fresh_validation.iter().zip(&real.validation_raw) {
        assert_eq!(fresh.support(), baseline.support());
        assert_eq!(fresh.work(), baseline.work());
    }
    let observation_rows = controlled_observations
        .iter()
        .map(Gate0ObservationRow::from_controlled)
        .collect::<Vec<_>>();
    let query_rows = real
        .validation
        .iter()
        .zip(&fresh_validation)
        .flat_map(|(row, raw)| query_rows_from_real(row.input.id, raw))
        .collect::<Vec<_>>();
    let map =
        FrozenGate0Map::Geometric(Gate0GeometricClassMap::compile(&observation_rows).unwrap());
    let aliases = construction_raw
        .iter()
        .map(controlled_typed_aliases)
        .sum::<usize>()
        + fresh_validation
            .iter()
            .map(real_typed_aliases)
            .sum::<usize>();
    let record = gate0_arm_record(
        name,
        Some(control),
        "fresh controlled construction placement plus fresh real validation admission",
        &map,
        &observation_rows,
        &query_rows,
        controlled_query_values(&construction_raw),
        raw_query_values(&fresh_validation),
        controlled_support_reports(&construction_raw),
        real_support_reports(&fresh_validation),
        None,
        aliases,
    );
    (
        Gate0EvaluationArm {
            name,
            map,
            queries: query_rows,
        },
        record,
    )
}

fn content_swap_arm(
    bundle: &FrozenBundle,
    real: &RealGate0Basis,
) -> (Gate0EvaluationArm, Gate0ArmRawRecord) {
    let name = "content_swap";
    let control = ConstructionCausalReturnNegativeControl::ContentSwap;
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let fresh_construction = real
        .construction
        .iter()
        .map(|row| {
            real.frame
                .raw_query(&bundle.attention, &row.predecessor_history, &table)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let mut observation_rows = Vec::with_capacity(real.construction.len() * 2);
    for ((row, raw), baseline) in real
        .construction
        .iter()
        .zip(&fresh_construction)
        .zip(&real.construction_raw)
    {
        assert_eq!(raw.support(), baseline.support());
        assert_eq!(raw.work(), baseline.work());
        observation_rows.extend(
            real.partition
                .authorize_label_join(raw, row.id, &row.observed_next)
                .unwrap()
                .iter()
                .map(Gate0ObservationRow::from_real),
        );
    }
    let mut validation_raw = Vec::with_capacity(real.validation.len());
    let mut query_rows = Vec::with_capacity(real.validation.len() * 2);
    for (pair_index, pair) in real.validation.chunks_exact(2).enumerate() {
        for within_pair in 0..2 {
            let row = &pair[within_pair];
            let baseline_index = pair_index * 2 + within_pair;
            let swapped = pair[1 - within_pair].observed_history.clone();
            let raw = real
                .frame
                .controlled_raw_query(
                    &bundle.attention,
                    &row.observed_history,
                    ConstructionCausalReturnPopulationRole::Validation,
                    &ConstructionCausalReturnControlledEncoder::ContentSwap {
                        swapped_observed_history: swapped,
                    },
                    &table,
                )
                .unwrap();
            assert_eq!(raw.frame_kappa(), real.frame.frame_kappa());
            assert_eq!(raw.control(), control);
            assert_eq!(raw.support(), real.validation_raw[baseline_index].support());
            assert_eq!(raw.work(), real.validation_raw[baseline_index].work());
            let report = raw.canonical_report();
            assert!(!report.support_reused_from_frozen_raw);
            assert_eq!(report.support_admission_queries_performed, 1);
            query_rows.extend(query_rows_from_controlled(row.input.id, &raw));
            validation_raw.push(raw);
        }
    }
    let map =
        FrozenGate0Map::Geometric(Gate0GeometricClassMap::compile(&observation_rows).unwrap());
    let aliases = fresh_construction
        .iter()
        .map(real_typed_aliases)
        .sum::<usize>()
        + validation_raw
            .iter()
            .map(controlled_typed_aliases)
            .sum::<usize>();
    let record = gate0_arm_record(
        name,
        Some(control),
        "fresh real construction admission plus fresh validation content transposition",
        &map,
        &observation_rows,
        &query_rows,
        raw_query_values(&fresh_construction),
        controlled_query_values(&validation_raw),
        real_support_reports(&fresh_construction),
        controlled_support_reports(&validation_raw),
        None,
        aliases,
    );
    (
        Gate0EvaluationArm {
            name,
            map,
            queries: query_rows,
        },
        record,
    )
}

fn repeated_encoder(
    encoder: ConstructionCausalReturnControlledEncoder,
    count: usize,
) -> Vec<ConstructionCausalReturnControlledEncoder> {
    std::iter::repeat_n(encoder, count).collect()
}

fn build_selection_blind_experiment(bundle: &FrozenBundle) -> SelectionBlindExperiment {
    let label_loads_before = SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst);
    let real = build_real_gate0_basis(bundle);
    assert_exact_policy_orientation_and_prefix_boundary(bundle, &real);
    let real_map = FrozenGate0Map::Geometric(real.generic_map.clone());
    let real_aliases = real
        .construction_raw
        .iter()
        .chain(&real.validation_raw)
        .map(real_typed_aliases)
        .sum();
    let mut real_record = gate0_arm_record(
        "real",
        None,
        "fresh natural schema-2 admission and frozen causal-return representation",
        &real_map,
        &real.observation_rows,
        &real.query_rows,
        raw_query_values(&real.construction_raw),
        raw_query_values(&real.validation_raw),
        real_support_reports(&real.construction_raw),
        real_support_reports(&real.validation_raw),
        None,
        real_aliases,
    );
    let real_record_for_equality = real_record.clone();
    set_arm_equalities_from_serialized_reports(&mut real_record, &real_record_for_equality);

    let mut arms = vec![Gate0EvaluationArm {
        name: "real",
        map: real_map,
        queries: real.query_rows.clone(),
    }];
    let mut negative_records = Vec::with_capacity(11);

    let controlled_specs = [
        (
            "state_disabled",
            ConstructionCausalReturnNegativeControl::StateDisabled,
            ConstructionCausalReturnControlledEncoder::StateDisabled,
            ControlledMapKind::Unavailable,
        ),
        (
            "last_only",
            ConstructionCausalReturnNegativeControl::LastOnly,
            ConstructionCausalReturnControlledEncoder::LastOnly,
            ControlledMapKind::Geometric,
        ),
        (
            "order_shuffled_history",
            ConstructionCausalReturnNegativeControl::OrderShuffledHistory,
            ConstructionCausalReturnControlledEncoder::OrderShuffledHistory,
            ControlledMapKind::Geometric,
        ),
        (
            "causal_return_lease_disabled",
            ConstructionCausalReturnNegativeControl::CausalReturnLeaseDisabled,
            ConstructionCausalReturnControlledEncoder::CausalReturnLeaseDisabled,
            ControlledMapKind::Geometric,
        ),
        (
            "exact_recall_only",
            ConstructionCausalReturnNegativeControl::ExactRecallOnly,
            ConstructionCausalReturnControlledEncoder::ExactRecallOnly,
            ControlledMapKind::ExactRecall,
        ),
    ];
    let controlled_results = controlled_specs
        .into_par_iter()
        .map(|(name, control, encoder, map_kind)| {
            controlled_arm(
                bundle,
                &real,
                name,
                control,
                repeated_encoder(encoder.clone(), real.construction.len()),
                repeated_encoder(encoder, real.validation.len()),
                map_kind,
            )
        })
        .collect::<Vec<_>>();
    for (arm, record) in controlled_results {
        arms.push(arm);
        negative_records.push(record);
    }

    let pairing_cycles = vec![
        vec!["A-is-1".to_owned(), "A-are-1".to_owned()],
        vec!["A-is-2".to_owned(), "A-are-2".to_owned()],
        vec!["B-has-1".to_owned(), "B-have-1".to_owned()],
        vec!["B-has-2".to_owned(), "B-have-2".to_owned()],
        vec!["C-was-1".to_owned(), "C-were-1".to_owned()],
        vec!["C-was-2".to_owned(), "C-were-2".to_owned()],
    ];
    // These six controls have no data dependency on one another.  Rayon uses
    // the host-sized shared pool; indexed collection preserves the frozen
    // control order and the subsequent canonical reduction remains serial.
    let secondary_results = (0_u8..6)
        .into_par_iter()
        .map(|control_index| match control_index {
            0 => post_raw_arm(
                bundle,
                &real,
                "construction_content_current_pairing_shuffle",
                ConstructionCausalReturnNegativeControl::ConstructionContentCurrentPairingShuffle,
                |fresh| {
                    let transformed =
                        cyclic_construction_label_pairing(&fresh.observation_rows, &pairing_cycles)
                            .unwrap();
                    assert_eq!(transformed.report.bindings.len(), 24);
                    assert!(transformed.report.bindings.iter().all(|binding| {
                        binding.target_action_before.is_some()
                            && binding.source_action.is_some()
                            && binding.target_action_before != binding.source_action
                    }));
                    let map = Gate0GeometricClassMap::compile(&transformed.value).unwrap();
                    (
                        FrozenGate0Map::Geometric(map),
                        transformed.value,
                        fresh.query_rows.clone(),
                        canonical_value(&transformed.report),
                    )
                },
            ),
            1 => candidate_prototype_placement_arm(bundle, &real),
            2 => {
                let table = validate_h4_binary_icosahedral_closure().unwrap();
                let population_addresses = bundle
                    .address_by_surface
                    .values()
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                let prime_permutation =
                    ConstructionCausalReturnPrimePlacementPermutation::canonical_one_step(
                        &population_addresses,
                        &table,
                    )
                    .unwrap();
                let prime_encoder =
                    ConstructionCausalReturnControlledEncoder::PrimePlacementPermutation(
                        prime_permutation,
                    );
                controlled_arm(
                    bundle,
                    &real,
                    "prime_placement_permutation",
                    ConstructionCausalReturnNegativeControl::PrimePlacementPermutation,
                    repeated_encoder(prime_encoder.clone(), real.construction.len()),
                    repeated_encoder(prime_encoder, real.validation.len()),
                    ControlledMapKind::Geometric,
                )
            }
            3 => content_swap_arm(bundle, &real),
            4 => post_raw_arm(
                bundle,
                &real,
                "construction_key_shuffle",
                ConstructionCausalReturnNegativeControl::ConstructionKeyShuffle,
                |fresh| {
                    let source_map =
                        Gate0GeometricClassMap::compile(&fresh.observation_rows).unwrap();
                    let transformed = cyclic_compiled_key_shuffle(&source_map).unwrap();
                    let transformed_rows = transformed.value.source_rows().to_vec();
                    (
                        FrozenGate0Map::Geometric(transformed.value),
                        transformed_rows,
                        fresh.query_rows.clone(),
                        canonical_value(&transformed.report),
                    )
                },
            ),
            5 => post_raw_arm(
                bundle,
                &real,
                "incoherent_candidate_relabeling",
                ConstructionCausalReturnNegativeControl::IncoherentCandidateRelabeling,
                |fresh| {
                    let map = Gate0GeometricClassMap::compile(&fresh.observation_rows).unwrap();
                    let transformed =
                        incoherent_candidate_representation_swap(&fresh.query_rows).unwrap();
                    (
                        FrozenGate0Map::Geometric(map),
                        fresh.observation_rows.clone(),
                        transformed.value,
                        canonical_value(&transformed.report),
                    )
                },
            ),
            _ => unreachable!("frozen secondary control index"),
        })
        .collect::<Vec<_>>();
    for (arm, record) in secondary_results {
        arms.push(arm);
        negative_records.push(record);
    }
    assert_eq!(negative_records.len(), 11);
    assert_eq!(arms.len(), 12);

    for record in &mut negative_records {
        set_arm_equalities_from_serialized_reports(record, &real_record);
    }
    assert_eq!(real_record.raw_report_frame_kappa, real.frame.frame_kappa());
    assert!(real_record.raw_report_frame_and_control_match_arm);
    assert!(real_record.support_equal_to_real);
    assert!(real_record.declared_work_equal_to_real);
    assert!(real_record.class_lookup_shape_exact);
    assert!(real_record.class_lookup_shape_equal_to_real);

    let (coherent_arm, coherent_candidate_map, coherent_report) = coherent_relabel(bundle, &real);
    let anti_recall = operative_anti_recall(&real);
    let real_validation_work =
        aggregate_validation_work(&real.validation_raw, &real_record.label_free_coverage);
    assert_eq!(real_validation_work.support_rows_read, 42);
    assert_eq!(real_validation_work.relation_slots, 96);
    assert_eq!(real_validation_work.declared_prototype_class_slot_reads, 24);
    assert_eq!(
        real_validation_work.performed_prototype_class_slot_reads,
        24
    );
    assert_eq!(real_validation_work.declared_payload_inversions, 12);
    assert_eq!(real_validation_work.performed_payload_inversions, 0);
    let all_arms_support_equal = real_record.support_equal_to_real
        && negative_records
            .iter()
            .all(|record| record.support_equal_to_real);
    let all_arms_declared_work_equal = real_record.declared_work_equal_to_real
        && negative_records
            .iter()
            .all(|record| record.declared_work_equal_to_real);
    let all_arms_raw_report_provenance_exact = real_record.raw_report_frame_and_control_match_arm
        && negative_records.iter().all(|record| {
            record.raw_report_frame_and_control_match_arm
                && record.raw_report_frame_kappa == real_record.raw_report_frame_kappa
        });
    let all_arms_class_lookup_shape_exact_and_equal = real_record.class_lookup_shape_exact
        && real_record.class_lookup_shape_equal_to_real
        && negative_records.iter().all(|record| {
            record.class_lookup_shape_exact && record.class_lookup_shape_equal_to_real
        });
    let all_arms_complete =
        arm_record_is_complete(&real_record) && negative_records.iter().all(arm_record_is_complete);
    assert!(all_arms_complete);
    let real_source_free = real
        .construction_raw
        .iter()
        .chain(&real.validation_raw)
        .all(|query| work_is_source_free(query.work()));
    let all_arms_source_provider_teacher_future_and_label_inputs_zero =
        real_source_free && all_arms_declared_work_equal;
    let incremental_reproduction = incremental_reproduction(bundle, &real);
    let count_only = count_only_last_anchor(&real);
    let construction_label_join_kappa = record_kappa(&(
        "uor-r4.construction-causal-return-construction-label-join/1",
        &real.observations,
    ));
    let label_loads_after = SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst);
    assert_eq!(label_loads_after, label_loads_before);
    let record = SelectionBlindExperimentRecord {
        schema: 1,
        identity: "uor-r4.construction-causal-return-raw-census/1",
        fixture_partition_kappa: FIXTURE_PARTITION_KAPPA,
        codec_vocabulary_record_kappa: CODEC_VOCABULARY_RECORD_KAPPA,
        construction_artifact_record_kappa: CONSTRUCTION_ARTIFACT_RECORD_KAPPA,
        mechanism_policy_kappa: MECHANISM_POLICY_KAPPA,
        label_free_validation_input_kappa: LABEL_FREE_VALIDATION_INPUT_KAPPA,
        construction_partition_kappa: real.partition.partition_kappa().to_owned(),
        compiler_query_frame_kappa: real.frame.frame_kappa().to_owned(),
        construction_label_join_kappa,
        core_artifact: canonical_value(&real.core.canonical_report()),
        real: real_record,
        negative_controls: negative_records,
        anti_recall,
        real_validation_work,
        all_arms_support_equal,
        all_arms_declared_work_equal,
        all_arms_raw_report_provenance_exact,
        all_arms_class_lookup_shape_exact_and_equal,
        all_arms_complete,
        all_arms_source_provider_teacher_future_and_label_inputs_zero,
        populated_padding_aliases: real_aliases,
        count_only_last_anchor: count_only.clone(),
        coherent_full_artifact_candidate_relabeling: coherent_report,
        incremental_reproduction,
        selector_type_present: false,
        validation_label_join_loads_observed: label_loads_after,
        validation_labels_loaded: label_loads_after != 0,
        issue_953_fixture_loaded: false,
    };
    SelectionBlindExperiment {
        record,
        arms,
        coherent_arm,
        coherent_candidate_map,
        count_only,
        route_artifact_bytes: bundle.artifact.canonical_bytes().unwrap(),
        attention_manifest_kappa: bundle.attention.manifest_kappa().to_owned(),
        partition_bytes: real.partition.canonical_bytes().unwrap(),
        frame_bytes: real.frame.canonical_bytes().unwrap(),
        core_bytes: real.core.canonical_bytes().unwrap(),
    }
}

fn build_gate0_raw_census() -> Gate0RawCensus {
    let label_loads_before = SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst);
    let ((primary_bundle, primary), (replay_bundle, replay)) = rayon::join(
        || {
            let bundle = build_frozen_bundle();
            let experiment = build_selection_blind_experiment(&bundle);
            (bundle, experiment)
        },
        || {
            let bundle = build_frozen_bundle();
            let experiment = build_selection_blind_experiment(&bundle);
            (bundle, experiment)
        },
    );
    let address_by_surface = primary_bundle.address_by_surface.clone();
    let primary_experiment_bytes = serde_json::to_vec(&primary.record).unwrap();
    let replay_experiment_bytes = serde_json::to_vec(&replay.record).unwrap();
    let independent_complete_build_inputs = !std::ptr::eq(&primary_bundle, &replay_bundle);
    let deterministic_rebuild = DeterministicRebuildReport {
        complete_builds: 2,
        codec_kappa_equal: primary_bundle.codec.codec_kappa() == replay_bundle.codec.codec_kappa(),
        vocabulary_kappa_equal: primary_bundle.codec.vocabulary_kappa()
            == replay_bundle.codec.vocabulary_kappa(),
        route_artifact_bytes_equal: primary.route_artifact_bytes == replay.route_artifact_bytes,
        attention_manifest_kappa_equal: primary.attention_manifest_kappa
            == replay.attention_manifest_kappa,
        construction_partition_bytes_equal: primary.partition_bytes == replay.partition_bytes,
        frame_bytes_equal: primary.frame_bytes == replay.frame_bytes,
        core_artifact_bytes_equal: primary.core_bytes == replay.core_bytes,
        selection_blind_experiment_bytes_equal: primary_experiment_bytes == replay_experiment_bytes,
        independent_complete_build_inputs,
        exact: false,
    };
    let exact = deterministic_rebuild.independent_complete_build_inputs
        && deterministic_rebuild.complete_builds == 2
        && deterministic_rebuild.codec_kappa_equal
        && deterministic_rebuild.vocabulary_kappa_equal
        && deterministic_rebuild.route_artifact_bytes_equal
        && deterministic_rebuild.attention_manifest_kappa_equal
        && deterministic_rebuild.construction_partition_bytes_equal
        && deterministic_rebuild.frame_bytes_equal
        && deterministic_rebuild.core_artifact_bytes_equal
        && deterministic_rebuild.selection_blind_experiment_bytes_equal;
    let deterministic_rebuild = DeterministicRebuildReport {
        exact,
        ..deterministic_rebuild
    };
    assert!(deterministic_rebuild.exact);
    let label_loads_after = SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst);
    assert_eq!(label_loads_after, label_loads_before);
    let record = Gate0RawCensusRecord {
        schema: 1,
        identity: "uor-r4.construction-causal-return-raw-census/1",
        experiment: primary.record.clone(),
        deterministic_rebuild,
        validation_label_join_status: if label_loads_after == 0 {
            "NOT_LOADED"
        } else {
            "PREVIOUSLY_LOADED_OUTSIDE_RAW_CENSUS"
        },
        selector_status: "DEPLOYED_SELECTOR_NOT_COMPILED_NOT_RUN_LABEL_FREE_COVERAGE_LOOKUP_ONLY",
    };
    let raw_census_kappa = record_kappa(&record);
    assert_eq!(raw_census_kappa, RAW_CENSUS_KAPPA);
    Gate0RawCensus {
        record,
        primary,
        replay,
        address_by_surface,
        raw_census_kappa,
    }
}

#[derive(Debug, Clone, Serialize)]
struct CountOnlyOutcome {
    decisions: usize,
    strict_ceiling_hits: usize,
    abstentions: usize,
    comparison_only: bool,
    attention_claim: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Gate0ArmCeilingRecord {
    name: &'static str,
    strict_ceiling: Gate0StrictCeilingReport,
}

#[derive(Debug, Clone, Serialize)]
struct LabelAttachedGate0Evaluation {
    raw_census_kappa: String,
    sealed_validation_label_join_kappa: String,
    real: Gate0ArmCeilingRecord,
    negative_controls: Vec<Gate0ArmCeilingRecord>,
    coherent_full_artifact_candidate_relabeling: Gate0ArmCeilingRecord,
    coherent_expected_ids_mapped_by_same_bijection: bool,
    coherent_post_label_ceiling_equivariance: bool,
    count_only_last_anchor: CountOnlyOutcome,
}

#[derive(Debug, Clone, Serialize)]
struct Gate0OutcomeRecord {
    schema: u32,
    identity: &'static str,
    raw_census_kappa: String,
    sealed_validation_label_join_kappa: String,
    evaluation: LabelAttachedGate0Evaluation,
    deterministic_outcome_replay_byte_identical: bool,
    construction_classes_pure: bool,
    all_six_decisions_structurally_covered: bool,
    real_strict_ceiling_is_six_of_six: bool,
    operative_anti_recall_clean: bool,
    populated_padding_aliases_zero: bool,
    real_strictly_above_every_negative_control: bool,
    support_and_declared_work_equal: bool,
    positive_metamorphic_controls_exact: bool,
    hard_gate_passed: bool,
    selector_status: &'static str,
    issue_953_generation_status: &'static str,
    payload_inversion_status: &'static str,
    source_provider_teacher_future_inputs: usize,
    terminal: &'static str,
}

fn gate0_validation_labels(
    address_by_surface: &BTreeMap<String, GeometricAddress>,
    join: &SealedValidationLabelJoin<'_>,
) -> Vec<Gate0ValidationLabel> {
    join.rows
        .iter()
        .map(|row| Gate0ValidationLabel {
            decision_id: row.id.to_owned(),
            expected_candidate_address_kappa: address_by_surface
                .get(row.expected_candidate)
                .unwrap()
                .canonical_kappa()
                .unwrap(),
        })
        .collect()
}

fn alpha_ceiling_corresponds(
    source: &Gate0StrictCeilingReport,
    relabeled: &Gate0StrictCeilingReport,
    mapping: &BTreeMap<String, String>,
) -> bool {
    let mut expected = source.clone();
    for decision in &mut expected.decisions {
        let Some(expected_candidate) = mapping.get(&decision.expected_candidate_address_kappa)
        else {
            return false;
        };
        decision.expected_candidate_address_kappa = expected_candidate.clone();
        if let Some(selected) = &mut decision.selected_candidate_address_kappa {
            let Some(mapped) = mapping.get(selected) else {
                return false;
            };
            *selected = mapped.clone();
        }
        for candidate in &mut decision.candidates {
            let Some(mapped) = mapping.get(&candidate.candidate_address_kappa) else {
                return false;
            };
            candidate.candidate_address_kappa = mapped.clone();
        }
        decision.candidates.sort_by(|left, right| {
            left.candidate_address_kappa
                .cmp(&right.candidate_address_kappa)
        });
    }
    // The target ceiling kappa is independently recomputed from renamed
    // labels, rows, and the rebuilt map; every remaining field is exact.
    expected.ceiling_kappa = relabeled.ceiling_kappa.clone();
    !relabeled.ceiling_kappa.is_empty() && expected == *relabeled
}

fn label_attached_evaluation(
    experiment: &SelectionBlindExperiment,
    labels: &[Gate0ValidationLabel],
    sealed_validation_label_join_kappa: &str,
    raw_census_kappa: &str,
) -> LabelAttachedGate0Evaluation {
    let real = Gate0ArmCeilingRecord {
        name: experiment.arms[0].name,
        strict_ceiling: experiment.arms[0]
            .map
            .ceiling(&experiment.arms[0].queries, labels),
    };
    let negative_controls = experiment
        .arms
        .iter()
        .skip(1)
        .map(|arm| Gate0ArmCeilingRecord {
            name: arm.name,
            strict_ceiling: arm.map.ceiling(&arm.queries, labels),
        })
        .collect::<Vec<_>>();
    let relabeled_labels = labels
        .iter()
        .map(|label| Gate0ValidationLabel {
            decision_id: label.decision_id.clone(),
            expected_candidate_address_kappa: experiment
                .coherent_candidate_map
                .get(&label.expected_candidate_address_kappa)
                .unwrap()
                .clone(),
        })
        .collect::<Vec<_>>();
    let coherent_expected_ids_mapped_by_same_bijection = labels.len() == 6
        && relabeled_labels.len() == labels.len()
        && relabeled_labels
            .iter()
            .map(|label| label.expected_candidate_address_kappa.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            == 6
        && labels
            .iter()
            .zip(&relabeled_labels)
            .all(|(source, relabeled)| {
                source.decision_id == relabeled.decision_id
                    && experiment
                        .coherent_candidate_map
                        .get(&source.expected_candidate_address_kappa)
                        == Some(&relabeled.expected_candidate_address_kappa)
            });
    let coherent_full_artifact_candidate_relabeling = Gate0ArmCeilingRecord {
        name: experiment.coherent_arm.name,
        strict_ceiling: experiment
            .coherent_arm
            .map
            .ceiling(&experiment.coherent_arm.queries, &relabeled_labels),
    };
    let coherent_post_label_ceiling_equivariance = alpha_ceiling_corresponds(
        &real.strict_ceiling,
        &coherent_full_artifact_candidate_relabeling.strict_ceiling,
        &experiment.coherent_candidate_map,
    );
    let label_by_decision = labels
        .iter()
        .map(|label| {
            (
                label.decision_id.as_str(),
                label.expected_candidate_address_kappa.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let strict_ceiling_hits = experiment
        .count_only
        .validation
        .iter()
        .filter(|decision| {
            decision.selected_candidate_address_kappa.as_deref()
                == label_by_decision
                    .get(decision.decision_id.as_str())
                    .copied()
        })
        .count();
    let abstentions = experiment
        .count_only
        .validation
        .iter()
        .filter(|decision| decision.selected_candidate_address_kappa.is_none())
        .count();
    LabelAttachedGate0Evaluation {
        raw_census_kappa: raw_census_kappa.to_owned(),
        sealed_validation_label_join_kappa: sealed_validation_label_join_kappa.to_owned(),
        real,
        negative_controls,
        coherent_full_artifact_candidate_relabeling,
        coherent_expected_ids_mapped_by_same_bijection,
        coherent_post_label_ceiling_equivariance,
        count_only_last_anchor: CountOnlyOutcome {
            decisions: experiment.count_only.validation.len(),
            strict_ceiling_hits,
            abstentions,
            comparison_only: true,
            attention_claim: false,
        },
    }
}

fn terminal_for_gate0(
    record: &SelectionBlindExperimentRecord,
    real_coverage: usize,
    hard_gate_passed: bool,
) -> &'static str {
    if hard_gate_passed {
        "GATE0_AUTHORIZES_ONE_SELECTOR_RUN_BUT_SELECTOR_NOT_PRESENT_IN_THIS_HARNESS"
    } else if !record
        .coherent_full_artifact_candidate_relabeling
        .complete_preselector_qualification_record_alpha_equivariance
    {
        "UNAVAILABLE_REQUIRED_COMPLETE_GATE0_ARTIFACT_ALPHA_EQUIVARIANCE"
    } else if record.anti_recall.operative_raw_prototype_recall {
        "UNAVAILABLE_NO_OPERATIVE_ANTI_RECALL"
    } else if real_coverage == 0 {
        "UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER"
    } else {
        "REDESIGN_CANDIDATE_CONDITIONED_CAUSAL_RETURN_REPRESENTATION"
    }
}

#[test]
#[ignore = "bounded issue #983 raw Gate 0 census; one entry with internally parallel ordered work"]
fn freeze_raw_selection_blind_gate0_census() {
    assert_eq!(SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst), 0);
    let census = build_gate0_raw_census();
    assert_eq!(SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst), 0);
    assert_eq!(
        census
            .record
            .experiment
            .validation_label_join_loads_observed,
        0
    );
    assert!(!census.record.experiment.validation_labels_loaded);
    assert!(census.record.experiment.all_arms_complete);
    assert!(census.record.experiment.all_arms_support_equal);
    assert!(census.record.experiment.all_arms_declared_work_equal);
    assert!(
        census
            .record
            .experiment
            .all_arms_raw_report_provenance_exact
    );
    assert!(
        census
            .record
            .experiment
            .all_arms_class_lookup_shape_exact_and_equal
    );
    let raw_json = serde_json::to_string(&census.record).unwrap();
    assert!(!raw_json.contains("singular"));
    assert!(!raw_json.contains("plural"));
    assert!(!raw_json.contains("expected_candidate"));
    println!("raw_census_kappa={}", census.raw_census_kappa);
    println!(
        "real_coverage={}/{} real_classes_pure={} promoted_rows={}/{} anti_recall={} aliases={}",
        census
            .record
            .experiment
            .real
            .label_free_coverage
            .covered_decisions,
        census
            .record
            .experiment
            .real
            .label_free_coverage
            .decision_count,
        census.record.experiment.real.all_selection_classes_pure,
        census.record.experiment.real.promoted_rows,
        census.record.experiment.real.promotion_denominator,
        census
            .record
            .experiment
            .anti_recall
            .operative_raw_prototype_recall,
        census.record.experiment.populated_padding_aliases,
    );
    let real_inventory = &census.record.experiment.real.inventory;
    println!(
        "real_inventory minimum_classes={} rich_classes={} promoted_minimum_classes={}",
        real_inventory["minimum_class_count"].as_u64().unwrap(),
        real_inventory["rich_class_count"].as_u64().unwrap(),
        real_inventory["promoted_minimum_classes"].as_u64().unwrap(),
    );
    for (index, class) in real_inventory["inventory"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let rich_classes = class["rich_classes"].as_array().unwrap();
        let rich_full_kappas = rich_classes
            .iter()
            .map(|rich| record_kappa(&rich["r_full"]))
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "real_minimum_class[{index}] r_min_kappa={} class_kappa={} rows={} select={} reject={} promoted={} direct={} rich_count={} rich_full_kappas={}",
            record_kappa(&class["r_min"]),
            record_kappa(class),
            class["construction_rows"].as_u64().unwrap(),
            class["select_rows"].as_u64().unwrap(),
            class["reject_rows"].as_u64().unwrap(),
            class["promoted_to_r_full"].as_bool().unwrap(),
            class["direct_action"].as_str().unwrap_or("PROMOTED"),
            rich_classes.len(),
            rich_full_kappas,
        );
    }
    println!(
        "construction_label_join_kappa={} real_map_kappa={}",
        census.record.experiment.construction_label_join_kappa,
        census.record.experiment.real.map_kappa,
    );
    println!(
        "anti_recall histories={}/{} raw={} suffix4={} ordered_route={} complete_representation={}",
        census.record.experiment.anti_recall.construction_histories,
        census.record.experiment.anti_recall.validation_histories,
        census
            .record
            .experiment
            .anti_recall
            .exact_raw_history_overlaps,
        census
            .record
            .experiment
            .anti_recall
            .exact_trailing_four_suffix_overlaps,
        census
            .record
            .experiment
            .anti_recall
            .exact_ordered_route_witness_overlaps,
        census
            .record
            .experiment
            .anti_recall
            .exact_complete_candidate_representation_overlaps,
    );
    println!(
        "real_validation_work decisions={} rows={} relation_slots={} declared_class_reads={} performed_class_reads={} declared_payload_inversions={} performed_payload_inversions={}",
        census.record.experiment.real_validation_work.decisions,
        census
            .record
            .experiment
            .real_validation_work
            .support_rows_read,
        census.record.experiment.real_validation_work.relation_slots,
        census
            .record
            .experiment
            .real_validation_work
            .declared_prototype_class_slot_reads,
        census
            .record
            .experiment
            .real_validation_work
            .performed_prototype_class_slot_reads,
        census
            .record
            .experiment
            .real_validation_work
            .declared_payload_inversions,
        census
            .record
            .experiment
            .real_validation_work
            .performed_payload_inversions,
    );
    println!(
        "metamorphic coherent_alpha={} incremental={} rebuild={} source_free={}",
        census
            .record
            .experiment
            .coherent_full_artifact_candidate_relabeling
            .complete_preselector_qualification_record_alpha_equivariance,
        census.record.experiment.incremental_reproduction.exact,
        census.record.deterministic_rebuild.exact,
        census
            .record
            .experiment
            .all_arms_source_provider_teacher_future_and_label_inputs_zero,
    );
    for control in &census.record.experiment.negative_controls {
        println!(
            "control={} structural_coverage={}/{} map_kappa={}",
            control.name,
            control.label_free_coverage.covered_decisions,
            control.label_free_coverage.decision_count,
            control.map_kappa,
        );
    }
    println!("raw_census_json_bytes={}", raw_json.len());
}

#[test]
#[ignore = "bounded issue #983 sealed post-raw Gate 0 outcome; one entry with internally parallel ordered work"]
fn attach_sealed_labels_and_freeze_gate0_outcome_without_selector() {
    assert_eq!(SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst), 0);
    let census = build_gate0_raw_census();
    assert_eq!(SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst), 0);
    let sealed = sealed_validation_label_join();
    assert_eq!(SEALED_VALIDATION_LABEL_JOIN_LOADS.load(Ordering::SeqCst), 1);
    let sealed_validation_label_join_kappa = record_kappa(&sealed);
    let labels = gate0_validation_labels(&census.address_by_surface, &sealed);
    let primary = label_attached_evaluation(
        &census.primary,
        &labels,
        &sealed_validation_label_join_kappa,
        &census.raw_census_kappa,
    );
    let replay = label_attached_evaluation(
        &census.replay,
        &labels,
        &sealed_validation_label_join_kappa,
        &census.raw_census_kappa,
    );
    let deterministic_outcome_replay_byte_identical =
        serde_json::to_vec(&primary).unwrap() == serde_json::to_vec(&replay).unwrap();
    let real_hits = primary.real.strict_ceiling.strict_ceiling_hits;
    let real_structural_coverage = census
        .record
        .experiment
        .real
        .label_free_coverage
        .covered_decisions;
    let real_strictly_above_every_negative_control = primary
        .negative_controls
        .iter()
        .all(|control| real_hits > control.strict_ceiling.strict_ceiling_hits);
    let coherent_raw = &census
        .record
        .experiment
        .coherent_full_artifact_candidate_relabeling;
    let coherent_exact = primary
        .coherent_full_artifact_candidate_relabeling
        .strict_ceiling
        .strict_ceiling_hits
        == real_hits
        && primary.coherent_expected_ids_mapped_by_same_bijection
        && primary.coherent_post_label_ceiling_equivariance
        && !coherent_raw.native_codec_or_placement_rebuild
        && coherent_raw.entries.len() == 6
        && coherent_raw.construction_rows_relabelled == 24
        && coherent_raw.validation_rows_relabelled == 12
        && coherent_raw.candidate_mapping_bijective
        && coherent_raw.partition_candidate_identity_correspondence
        && coherent_raw.support_candidate_identity_and_counts_correspondence
        && coherent_raw.raw_candidate_identity_representation_and_work_correspondence
        && coherent_raw.construction_action_correspondence
        && coherent_raw.validation_query_correspondence
        && coherent_raw.surface_commitment_correspondence
        && coherent_raw.exact_row_correspondence
        && coherent_raw.registered_surface_association_reproduced
        && coherent_raw.map_kappa_reproduced
        && coherent_raw.structural_coverage_reproduced
        && coherent_raw
            .support_prototypes_surface_commitments_and_construction_actions_move_together
        && coherent_raw.no_candidate_identity_occurrence_left_unmapped
        && coherent_raw.involution_twice_reproduces_original_bytes
        && coherent_raw.qualification_record_alpha_equivariance_without_payload
        && coherent_raw.complete_preselector_qualification_record_alpha_equivariance
        && !coherent_raw.validation_label_commitment_loaded
        && !coherent_raw.report_kappa.is_empty();
    let construction_classes_pure = census.record.experiment.real.all_selection_classes_pure;
    let all_six_decisions_structurally_covered = real_structural_coverage == 6;
    let real_strict_ceiling_is_six_of_six = real_hits == 6;
    let operative_anti_recall_clean = !census
        .record
        .experiment
        .anti_recall
        .operative_raw_prototype_recall;
    let populated_padding_aliases_zero = census.record.experiment.populated_padding_aliases == 0;
    let support_and_declared_work_equal = census.record.experiment.all_arms_support_equal
        && census.record.experiment.all_arms_declared_work_equal
        && census
            .record
            .experiment
            .all_arms_raw_report_provenance_exact
        && census
            .record
            .experiment
            .all_arms_class_lookup_shape_exact_and_equal
        && census.record.experiment.all_arms_complete;
    let positive_metamorphic_controls_exact = coherent_exact
        && census.record.experiment.incremental_reproduction.exact
        && census.record.deterministic_rebuild.exact
        && deterministic_outcome_replay_byte_identical;
    let hard_gate_passed = construction_classes_pure
        && all_six_decisions_structurally_covered
        && real_strict_ceiling_is_six_of_six
        && operative_anti_recall_clean
        && populated_padding_aliases_zero
        && real_strictly_above_every_negative_control
        && support_and_declared_work_equal
        && census
            .record
            .experiment
            .all_arms_source_provider_teacher_future_and_label_inputs_zero
        && positive_metamorphic_controls_exact;
    let terminal = terminal_for_gate0(
        &census.record.experiment,
        real_structural_coverage,
        hard_gate_passed,
    );
    let source_provider_teacher_future_inputs =
        census.record.experiment.real_validation_work.source_inputs
            + census
                .record
                .experiment
                .real_validation_work
                .provider_inputs
            + census.record.experiment.real_validation_work.teacher_inputs
            + census
                .record
                .experiment
                .real_validation_work
                .future_route_inputs;
    let outcome = Gate0OutcomeRecord {
        schema: 1,
        identity: "uor-r4.construction-causal-return-outcome/1",
        raw_census_kappa: census.raw_census_kappa.clone(),
        sealed_validation_label_join_kappa,
        evaluation: primary,
        deterministic_outcome_replay_byte_identical,
        construction_classes_pure,
        all_six_decisions_structurally_covered,
        real_strict_ceiling_is_six_of_six,
        operative_anti_recall_clean,
        populated_padding_aliases_zero,
        real_strictly_above_every_negative_control,
        support_and_declared_work_equal,
        positive_metamorphic_controls_exact,
        hard_gate_passed,
        selector_status: "NOT_RUN_DEPLOYED_SELECTOR_ABSENT_OFFLINE_STRICT_CEILING_LOOKUP_ONLY",
        issue_953_generation_status: "NOT_RUN",
        payload_inversion_status: "NOT_RUN_SELECTOR_ONLY_NO_GATE0_ARTIFACT_INVERSION",
        source_provider_teacher_future_inputs,
        terminal,
    };
    assert!(!outcome.hard_gate_passed);
    assert_eq!(outcome.terminal, "UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER");
    assert_eq!(
        outcome.evaluation.real.strict_ceiling.strict_ceiling_hits,
        0
    );
    assert_eq!(outcome.evaluation.real.strict_ceiling.decision_count, 6);
    assert_eq!(outcome.evaluation.real.strict_ceiling.abstentions, 6);
    assert_eq!(
        outcome.evaluation.real.strict_ceiling.declared_class_reads,
        24
    );
    assert_eq!(
        outcome.evaluation.real.strict_ceiling.performed_class_reads,
        24
    );
    assert!(outcome.evaluation.negative_controls.iter().all(|control| {
        control.strict_ceiling.strict_ceiling_hits == 0
            && control.strict_ceiling.decision_count == 6
            && control.strict_ceiling.abstentions == 6
            && control.strict_ceiling.declared_class_reads == 24
            && control.strict_ceiling.performed_class_reads == 24
    }));
    assert_eq!(
        outcome.selector_status,
        "NOT_RUN_DEPLOYED_SELECTOR_ABSENT_OFFLINE_STRICT_CEILING_LOOKUP_ONLY"
    );
    assert_eq!(outcome.issue_953_generation_status, "NOT_RUN");
    assert_eq!(
        outcome.payload_inversion_status,
        "NOT_RUN_SELECTOR_ONLY_NO_GATE0_ARTIFACT_INVERSION"
    );
    assert_eq!(outcome.source_provider_teacher_future_inputs, 0);
    assert!(outcome.deterministic_outcome_replay_byte_identical);
    assert!(outcome.support_and_declared_work_equal);
    assert!(outcome.positive_metamorphic_controls_exact);
    assert_eq!(
        outcome
            .evaluation
            .count_only_last_anchor
            .strict_ceiling_hits,
        0
    );
    let outcome_kappa = record_kappa(&outcome);
    assert_eq!(outcome_kappa, OUTCOME_KAPPA);
    println!("raw_census_kappa={}", census.raw_census_kappa);
    println!(
        "sealed_validation_label_join_kappa={}",
        outcome.sealed_validation_label_join_kappa
    );
    println!("outcome_kappa={outcome_kappa}");
    println!(
        "real_strict_ceiling={}/{} terminal={} selector={} count_only={}/{}",
        outcome.evaluation.real.strict_ceiling.strict_ceiling_hits,
        outcome.evaluation.real.strict_ceiling.decision_count,
        outcome.terminal,
        outcome.selector_status,
        outcome
            .evaluation
            .count_only_last_anchor
            .strict_ceiling_hits,
        outcome.evaluation.count_only_last_anchor.decisions,
    );
    for control in &outcome.evaluation.negative_controls {
        println!(
            "control={} strict_ceiling={}/{} abstentions={} ties={}",
            control.name,
            control.strict_ceiling.strict_ceiling_hits,
            control.strict_ceiling.decision_count,
            control.strict_ceiling.abstentions,
            control.strict_ceiling.ties_or_multiply_selected,
        );
    }
}
