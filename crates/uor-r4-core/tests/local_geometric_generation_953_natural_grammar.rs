use std::collections::BTreeSet;

use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, CanonicalLexicalCodec, CanonicalRouteArtifact, ConversationInput,
    ParagraphInput, TurnInput,
};
use uor_r4_core::local_geometric_generation::{
    LocalGenerationControl, LocalGenerationSourceCounts, LocalGenerationStopReason,
    LocalGeometricGenerationReport, LocalGeometricGenerator,
};
use uor_r4_core::prime_route_attention::GeometricAddress;
use uor_r4_core::prime_route_geometric_attention::{
    AttentionQueryPolicy, AttentionRowKey, AttentionRowSource, AttentionSourceCounts,
    AttentionSupportTrace, GeometricAttentionArtifact,
    PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY,
};

const IDENTITY_SCOPE: &str = "issue-953/natural-agreement-v1";
const TURN_ID: &str = "turn-0001";
const CONSTRUCTION_SENTENCES: [&[&str]; 2] = [
    &["athletes", "generally", "still", "run"],
    &["one", "athlete", "generally", "still", "runs"],
];
const GLOBAL_SNAPSHOT: &[u8] = b"near";
const REGISTERED_SURFACES: [&str; 8] = [
    "athlete",
    "athletes",
    "generally",
    "near",
    "one",
    "run",
    "runs",
    "still",
];
const LEFT_PROMPT: &[u8] = b"one athlete near athletes generally";
const RIGHT_PROMPT: &[u8] = b"athletes near one athlete generally";
const LEFT_EXPECTED: &[&[u8]] = &[b"still", b"runs"];
const RIGHT_EXPECTED: &[&[u8]] = &[b"still", b"run"];
const CONTINUATION_CAP: usize = 2;
const REVISION_TERMINAL: &str = "REVISE_I1_GENERATOR_IN_PLACE";
const POSITIVE_TERMINAL: &str = "PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION";

const FIXTURE_IDENTITY_BYTES: &[u8] = b"uor-r4.local-geometric-natural-agreement/1\
\nidentity_scope=issue-953/natural-agreement-v1\
\nturn_id=turn-0001\
\nglobal_snapshot=near\
\nobservation_0001=athletes generally still run\
\nobservation_0002=one athlete generally still runs\
\nregistered_surfaces=athlete|athletes|generally|near|one|run|runs|still\
\nleft_prompt=one athlete near athletes generally\
\nright_prompt=athletes near one athlete generally\
\nleft_expected=still runs\
\nright_expected=still run\
\ncontrols=full_path|state_disabled\
\ncontinuation_cap=2\
\npositive_terminal=PROCEED_TO_A1Q_H_WITH_BOUNDED_SOURCE_FREE_GEOMETRIC_GENERATION\
\nfailure_terminal=REVISE_I1_GENERATOR_IN_PLACE\
\nstep_1_union=still\
\nstep_1_support=still:last_one=2,last_two=1,ordered_sentence=0,divisor=2,adjacent_spin=0\
\nstep_1_work=rows=7,entries=3,unique=1,keys_per_candidate=5,h4_comparisons=5\
\nstep_2_union=run|runs\
\nstep_2_support=each:last_one=1,last_two=1,ordered_sentence=0,divisor=1,adjacent_spin=0\
\nstep_2_work=rows=7,entries=6,unique=2,keys_per_candidate=6,h4_comparisons=12\
\ntermination=cap\
\ncycle=period-1-through-4-requires-three-equal-trailing-periods";

// These five identities are filled only from the pre-selection freeze test,
// before the ignored four-arm selector witness is ever executed.
const FIXTURE_KAPPA: &str =
    "blake3:0e018c9bcd43a29ed6f043665b2646c9579dd31d881d331f198fb89543184259";
const CODEC_KAPPA: &str = "blake3:6db64540ef344562903e01adac102f7bcc96c65908d162b1deca9b83550b35ed";
const VOCABULARY_KAPPA: &str =
    "blake3:3b74f7ace425c039b4eab751b400f2603d92baf4ccfc9f4b8ac9409446291b58";
const CONSTRUCTION_ARTIFACT_KAPPA: &str =
    "blake3:b222510ccc01ed3257c8b38b743ca771f5e60c87ebf12c565f92fadbbd00332d";
const ATTENTION_MANIFEST_KAPPA: &str =
    "blake3:1c3baf432b9fdcf2f3d90014797a5cae5850c0acba2fda63e0d6b659d49562de";
const PRIOR_SUPPORT_PREFLIGHT_RECORD_KAPPA: &str =
    "blake3:70375921e267b5ceff2198f879356cfb42dd6907accc0c2b720fc8b89b59b271";
const QUERY_POLICY_KAPPA: &str =
    "blake3:18c514b74b7d3e0e8796d9834c74d84745f0eddc88be0ef87236474f97a83820";
const REPAIRED_SUPPORT_PREFLIGHT_RECORD_KAPPA: &str =
    "blake3:aab38fc513521cdd495bad74cc4a87754ec43ecdef5cb6e098b101412d3d7fe9";

// A post-run evidence binding, never an input to selection. This was `None` at
// the pre-selection checkpoint and was replaced only after the single
// four-arm run and its byte-identical replay.
const FOUR_ARM_RECORD_KAPPA: Option<&str> =
    Some("blake3:dfe03d4c56f7e5e9cf48d524f2f0b10482c4b3b85fae152dd29c64543caa0b79");

struct FrozenBundle {
    codec: CanonicalLexicalCodec,
    artifact: CanonicalRouteArtifact,
    attention: GeometricAttentionArtifact,
}

fn frozen_input() -> ConversationInput {
    let global_snapshot_units = vec![GLOBAL_SNAPSHOT.to_vec()];
    ConversationInput {
        identity_scope: IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units).unwrap(),
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: TURN_ID.to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: CONSTRUCTION_SENTENCES
                    .iter()
                    .map(|sentence| sentence.join(" ").into_bytes())
                    .collect(),
            }],
        }],
    }
}

fn frozen_bundle() -> FrozenBundle {
    // The same value is deliberately passed to compilation and ingestion so
    // registration cannot diverge from the construction/global observation.
    let input = frozen_input();
    let codec = CanonicalLexicalCodec::compile(&input).unwrap();
    let artifact = CanonicalRouteArtifact::ingest(&codec, &input).unwrap();
    let manifest = artifact.embedded_spin_manifest().unwrap();
    let attention = GeometricAttentionArtifact::compile_from_manifest_witnesses(&manifest).unwrap();
    FrozenBundle {
        codec,
        artifact,
        attention,
    }
}

fn fixture_kappa() -> String {
    format!("blake3:{}", blake3::hash(FIXTURE_IDENTITY_BYTES).to_hex())
}

fn address_for_surface(bundle: &FrozenBundle, surface: &[u8]) -> GeometricAddress {
    let encoded = bundle.codec.encode(0, 0, surface).unwrap();
    assert_eq!(bundle.codec.decode(&encoded).unwrap(), surface);
    assert_eq!(encoded.units.len(), 1);
    assert!(encoded.units[0].leading_bytes.is_empty());
    assert!(encoded.trailing_bytes.is_empty());
    bundle
        .artifact
        .lexical_route_address(encoded.units[0].unit_id)
        .unwrap()
        .unwrap()
}

fn history_for_prompt(bundle: &FrozenBundle, prompt: &[u8]) -> Vec<GeometricAddress> {
    let encoded = bundle.codec.encode(0, 0, prompt).unwrap();
    assert_eq!(bundle.codec.decode(&encoded).unwrap(), prompt);
    assert!(encoded.trailing_bytes.is_empty());
    encoded
        .units
        .iter()
        .map(|unit| {
            bundle
                .artifact
                .lexical_route_address(unit.unit_id)
                .unwrap()
                .unwrap()
        })
        .collect()
}

fn payload_for_address(bundle: &FrozenBundle, address: &GeometricAddress) -> Vec<u8> {
    bundle
        .artifact
        .lexical_route_value_for_address(address)
        .unwrap()
        .unwrap()
        .payload_bytes
}

fn count_array(counts: AttentionSourceCounts) -> [u32; 5] {
    [
        counts.last_one,
        counts.last_two,
        counts.ordered_sentence,
        counts.divisor,
        counts.adjacent_spin,
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PreflightCandidate {
    payload_bytes: Vec<u8>,
    source_counts: [u32; 5],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LegacyPreflightStep {
    observed_routes: usize,
    rows_queried: usize,
    candidate_entries: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    candidates: Vec<PreflightCandidate>,
    adjacent_spin_rows_hit: usize,
    keys_per_candidate: usize,
    declared_h4_comparisons: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LegacyPreflightArm {
    prompt_bytes: Vec<u8>,
    steps: Vec<LegacyPreflightStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LegacyPreflightRecord {
    schema: u32,
    fixture_kappa: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    construction_artifact_kappa: String,
    attention_manifest_kappa: String,
    registered_surfaces: usize,
    observed_routes_per_completed_arm: usize,
    left: LegacyPreflightArm,
    right: LegacyPreflightArm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum PreflightRowSource {
    LastOne,
    LastTwo,
    OrderedSentence,
    Divisor,
    AdjacentSpin,
}

impl From<AttentionRowSource> for PreflightRowSource {
    fn from(source: AttentionRowSource) -> Self {
        match source {
            AttentionRowSource::LastOne => Self::LastOne,
            AttentionRowSource::LastTwo => Self::LastTwo,
            AttentionRowSource::OrderedSentence => Self::OrderedSentence,
            AttentionRowSource::Divisor => Self::Divisor,
            AttentionRowSource::AdjacentSpin => Self::AdjacentSpin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PreflightRowKey {
    LastOne {
        address_kappa: String,
    },
    LastTwo {
        previous_address_kappa: String,
        last_address_kappa: String,
    },
    LastTwoUnavailable,
    OrderedSentence {
        route_kappa: String,
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
struct RepairedPreflightRow {
    slot_index: usize,
    source: PreflightRowSource,
    key: PreflightRowKey,
    consulted: bool,
    physical_row_present: bool,
    fallback_active: bool,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairedPreflightStep {
    observed_routes: usize,
    query_policy: String,
    query_policy_kappa: String,
    fallback_active: bool,
    rows: Vec<RepairedPreflightRow>,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    candidates: Vec<PreflightCandidate>,
    keys_per_candidate: usize,
    declared_h4_comparisons: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairedPreflightArm {
    prompt_bytes: Vec<u8>,
    steps: Vec<RepairedPreflightStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RepairedPreflightRecord {
    schema: u32,
    prior_support_record_kappa: String,
    fixture_kappa: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    construction_artifact_kappa: String,
    attention_manifest_kappa: String,
    query_policy: String,
    query_policy_kappa: String,
    registered_surfaces: usize,
    observed_routes_per_completed_arm: usize,
    left: RepairedPreflightArm,
    right: RepairedPreflightArm,
}

fn preflight_row(
    row: &uor_r4_core::prime_route_geometric_attention::AttentionRowRead,
) -> RepairedPreflightRow {
    let key = match &row.key {
        AttentionRowKey::LastOne(address) => PreflightRowKey::LastOne {
            address_kappa: address.canonical_kappa().unwrap(),
        },
        AttentionRowKey::LastTwo { previous, last } => PreflightRowKey::LastTwo {
            previous_address_kappa: previous.canonical_kappa().unwrap(),
            last_address_kappa: last.canonical_kappa().unwrap(),
        },
        AttentionRowKey::LastTwoUnavailable => PreflightRowKey::LastTwoUnavailable,
        AttentionRowKey::OrderedSentence(route_kappa) => PreflightRowKey::OrderedSentence {
            route_kappa: route_kappa.clone(),
        },
        AttentionRowKey::Divisor(atom) => PreflightRowKey::Divisor {
            prime: atom.value(),
        },
        AttentionRowKey::AdjacentSpin(sector) => PreflightRowKey::AdjacentSpin {
            hopf_octant: sector.hopf_octant,
            torsion_bin: sector.torsion_bin,
        },
    };
    RepairedPreflightRow {
        slot_index: row.slot_index,
        source: row.source.into(),
        key,
        consulted: row.consulted,
        physical_row_present: row.physical_row_present,
        fallback_active: row.fallback_active,
        candidate_entries_available: row.candidate_entries_available,
        candidate_entries_examined: row.candidate_entries_examined,
        candidate_entries_admitted: row.candidate_entries_admitted,
    }
}

fn preflight_step(
    bundle: &FrozenBundle,
    observed_routes: usize,
    trace: AttentionSupportTrace,
) -> RepairedPreflightStep {
    let mut candidates = trace
        .candidates
        .into_iter()
        .map(|candidate| PreflightCandidate {
            payload_bytes: payload_for_address(bundle, &candidate.next),
            source_counts: count_array(candidate.source_counts),
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.payload_bytes.cmp(&right.payload_bytes));
    let declared_h4_comparisons = candidates.len() * observed_routes;
    RepairedPreflightStep {
        observed_routes,
        query_policy: trace.query_policy.identity().to_owned(),
        query_policy_kappa: trace.query_policy_kappa,
        fallback_active: trace.fallback_active,
        rows: trace.rows_read.iter().map(preflight_row).collect(),
        candidate_entries_available: trace.candidate_entries_available,
        candidate_entries_examined: trace.candidate_entries_examined,
        candidate_entries_admitted: trace.candidate_entries_admitted,
        candidate_entry_ceiling: trace.candidate_entry_ceiling,
        unique_candidates_before_ceiling: trace.unique_candidates_before_ceiling,
        candidate_ceiling: trace.candidate_ceiling,
        candidates,
        keys_per_candidate: observed_routes,
        declared_h4_comparisons,
    }
}

fn preflight_arm(bundle: &FrozenBundle, prompt: &[u8]) -> RepairedPreflightArm {
    let history = history_for_prompt(bundle, prompt);
    assert_eq!(history.len(), 5);
    let mut state = bundle
        .attention
        .causal_state_from_history(&history)
        .unwrap();

    // This method has no H4 table, state, cost, or selection input. The
    // projection records only bounded lookup keys, tier state, and support.
    let first = preflight_step(
        bundle,
        history.len(),
        bundle.attention.query_support_only(&state).unwrap(),
    );
    let still = address_for_surface(bundle, b"still");
    bundle.attention.observe(&mut state, still).unwrap();
    let second = preflight_step(
        bundle,
        history.len() + 1,
        bundle.attention.query_support_only(&state).unwrap(),
    );
    RepairedPreflightArm {
        prompt_bytes: prompt.to_vec(),
        steps: vec![first, second],
    }
}

fn adjacent_rows(step: &RepairedPreflightStep) -> Vec<&RepairedPreflightRow> {
    step.rows
        .iter()
        .filter(|row| row.source == PreflightRowSource::AdjacentSpin)
        .collect()
}

fn preflight_matches_frozen_contract(arm: &RepairedPreflightArm) -> bool {
    let Some(first) = arm.steps.first() else {
        return false;
    };
    let Some(second) = arm.steps.get(1) else {
        return false;
    };
    let first_adjacent = adjacent_rows(first);
    let second_adjacent = adjacent_rows(second);
    arm.steps.len() == 2
        && first.observed_routes == 5
        && first.query_policy == PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY
        && first.query_policy_kappa == QUERY_POLICY_KAPPA
        && !first.fallback_active
        && first.rows.len() == 7
        && first.candidate_entries_available == 8
        && first.candidate_entries_examined == 3
        && first.candidate_entries_admitted == 3
        && first.unique_candidates_before_ceiling == 1
        && first_adjacent.len() == 3
        && first_adjacent
            .iter()
            .all(|row| row.consulted && !row.fallback_active)
        && first_adjacent
            .iter()
            .filter(|row| row.physical_row_present)
            .count()
            == 1
        && first_adjacent
            .iter()
            .map(|row| row.candidate_entries_available)
            .sum::<usize>()
            == 5
        && first_adjacent
            .iter()
            .all(|row| row.candidate_entries_examined == 0 && row.candidate_entries_admitted == 0)
        && first.keys_per_candidate == 5
        && first.declared_h4_comparisons == 5
        && first.candidates
            == vec![PreflightCandidate {
                payload_bytes: b"still".to_vec(),
                source_counts: [2, 1, 0, 2, 0],
            }]
        && second.observed_routes == 6
        && second.query_policy == PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY
        && second.query_policy_kappa == QUERY_POLICY_KAPPA
        && !second.fallback_active
        && second.rows.len() == 7
        && second.candidate_entries_available == 11
        && second.candidate_entries_examined == 6
        && second.candidate_entries_admitted == 6
        && second.unique_candidates_before_ceiling == 2
        && second_adjacent.len() == 3
        && second_adjacent
            .iter()
            .all(|row| row.consulted && !row.fallback_active)
        && second_adjacent
            .iter()
            .filter(|row| row.physical_row_present)
            .count()
            == 1
        && second_adjacent
            .iter()
            .map(|row| row.candidate_entries_available)
            .sum::<usize>()
            == 5
        && second_adjacent
            .iter()
            .all(|row| row.candidate_entries_examined == 0 && row.candidate_entries_admitted == 0)
        && second.keys_per_candidate == 6
        && second.declared_h4_comparisons == 12
        && second.candidates
            == vec![
                PreflightCandidate {
                    payload_bytes: b"run".to_vec(),
                    source_counts: [1, 1, 0, 1, 0],
                },
                PreflightCandidate {
                    payload_bytes: b"runs".to_vec(),
                    source_counts: [1, 1, 0, 1, 0],
                },
            ]
}

fn preflight_arms_have_equal_support_and_work(
    left: &RepairedPreflightArm,
    right: &RepairedPreflightArm,
) -> bool {
    left.steps.len() == right.steps.len()
        && left.steps.iter().zip(&right.steps).all(|(left, right)| {
            left.observed_routes == right.observed_routes
                && left.query_policy == right.query_policy
                && left.query_policy_kappa == right.query_policy_kappa
                && left.fallback_active == right.fallback_active
                && left.candidate_entries_available == right.candidate_entries_available
                && left.candidate_entries_examined == right.candidate_entries_examined
                && left.candidate_entries_admitted == right.candidate_entries_admitted
                && left.candidate_entry_ceiling == right.candidate_entry_ceiling
                && left.unique_candidates_before_ceiling == right.unique_candidates_before_ceiling
                && left.candidate_ceiling == right.candidate_ceiling
                && left.candidates == right.candidates
                && left.keys_per_candidate == right.keys_per_candidate
                && left.declared_h4_comparisons == right.declared_h4_comparisons
                && left.rows.len() == right.rows.len()
                && left.rows.iter().zip(&right.rows).all(|(left, right)| {
                    left.slot_index == right.slot_index
                        && left.source == right.source
                        && left.consulted == right.consulted
                        && left.physical_row_present == right.physical_row_present
                        && left.fallback_active == right.fallback_active
                        && left.candidate_entries_available == right.candidate_entries_available
                        && left.candidate_entries_examined == right.candidate_entries_examined
                        && left.candidate_entries_admitted == right.candidate_entries_admitted
                })
        })
}

fn prior_support_preflight_arm(prompt: &[u8]) -> LegacyPreflightArm {
    LegacyPreflightArm {
        prompt_bytes: prompt.to_vec(),
        steps: vec![
            LegacyPreflightStep {
                observed_routes: 5,
                rows_queried: 7,
                candidate_entries: 8,
                candidate_entry_ceiling: 56,
                unique_candidates_before_ceiling: 5,
                candidate_ceiling: 8,
                candidates: vec![
                    PreflightCandidate {
                        payload_bytes: b"athlete".to_vec(),
                        source_counts: [0, 0, 0, 0, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"generally".to_vec(),
                        source_counts: [0, 0, 0, 0, 2],
                    },
                    PreflightCandidate {
                        payload_bytes: b"run".to_vec(),
                        source_counts: [0, 0, 0, 0, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"runs".to_vec(),
                        source_counts: [0, 0, 0, 0, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"still".to_vec(),
                        source_counts: [2, 1, 0, 2, 2],
                    },
                ],
                adjacent_spin_rows_hit: 1,
                keys_per_candidate: 5,
                declared_h4_comparisons: 25,
            },
            LegacyPreflightStep {
                observed_routes: 6,
                rows_queried: 7,
                candidate_entries: 11,
                candidate_entry_ceiling: 56,
                unique_candidates_before_ceiling: 5,
                candidate_ceiling: 8,
                candidates: vec![
                    PreflightCandidate {
                        payload_bytes: b"athlete".to_vec(),
                        source_counts: [0, 0, 0, 0, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"generally".to_vec(),
                        source_counts: [0, 0, 0, 0, 2],
                    },
                    PreflightCandidate {
                        payload_bytes: b"run".to_vec(),
                        source_counts: [1, 1, 0, 1, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"runs".to_vec(),
                        source_counts: [1, 1, 0, 1, 1],
                    },
                    PreflightCandidate {
                        payload_bytes: b"still".to_vec(),
                        source_counts: [0, 0, 0, 0, 2],
                    },
                ],
                adjacent_spin_rows_hit: 1,
                keys_per_candidate: 6,
                declared_h4_comparisons: 30,
            },
        ],
    }
}

#[test]
fn freeze_natural_agreement_contract_and_identities() {
    let bundle = frozen_bundle();
    assert!(REGISTERED_SURFACES.windows(2).all(|pair| pair[0] < pair[1]));
    let registered_ids = REGISTERED_SURFACES
        .iter()
        .map(|surface| {
            let encoded = bundle.codec.encode(0, 0, surface.as_bytes()).unwrap();
            assert_eq!(encoded.units.len(), 1);
            encoded.units[0].unit_id
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(registered_ids.len(), 8);
    assert_eq!(history_for_prompt(&bundle, LEFT_PROMPT).len(), 5);
    assert_eq!(history_for_prompt(&bundle, RIGHT_PROMPT).len(), 5);

    let actual = [
        fixture_kappa(),
        bundle.codec.codec_kappa().to_owned(),
        bundle.codec.vocabulary_kappa().to_owned(),
        bundle.artifact.manifest_kappa().to_owned(),
        bundle.attention.manifest_kappa().to_owned(),
    ];
    let frozen = [
        FIXTURE_KAPPA.to_owned(),
        CODEC_KAPPA.to_owned(),
        VOCABULARY_KAPPA.to_owned(),
        CONSTRUCTION_ARTIFACT_KAPPA.to_owned(),
        ATTENTION_MANIFEST_KAPPA.to_owned(),
    ];
    assert_eq!(actual, frozen);
}

#[test]
fn prior_natural_agreement_support_hard_stop_record_is_append_only() {
    let left = prior_support_preflight_arm(LEFT_PROMPT);
    let right = prior_support_preflight_arm(RIGHT_PROMPT);
    let record = LegacyPreflightRecord {
        schema: 1,
        fixture_kappa: FIXTURE_KAPPA.to_owned(),
        codec_kappa: CODEC_KAPPA.to_owned(),
        vocabulary_kappa: VOCABULARY_KAPPA.to_owned(),
        construction_artifact_kappa: CONSTRUCTION_ARTIFACT_KAPPA.to_owned(),
        attention_manifest_kappa: ATTENTION_MANIFEST_KAPPA.to_owned(),
        registered_surfaces: 8,
        observed_routes_per_completed_arm: 7,
        left,
        right,
    };
    let bytes = serde_json::to_vec(&record).unwrap();
    let record_kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    assert_eq!(record_kappa, PRIOR_SUPPORT_PREFLIGHT_RECORD_KAPPA);
    assert_eq!(record.left.steps, record.right.steps);
    assert_eq!(record.left.steps[0].candidate_entries, 8);
    assert_eq!(record.left.steps[1].candidate_entries, 11);
}

#[test]
fn natural_agreement_support_preflight_qualifies_tiered_admission() {
    let bundle = frozen_bundle();
    assert_eq!(fixture_kappa(), FIXTURE_KAPPA);
    assert_eq!(bundle.codec.codec_kappa(), CODEC_KAPPA);
    assert_eq!(bundle.codec.vocabulary_kappa(), VOCABULARY_KAPPA);
    assert_eq!(
        bundle.artifact.manifest_kappa(),
        CONSTRUCTION_ARTIFACT_KAPPA
    );
    assert_eq!(bundle.attention.manifest_kappa(), ATTENTION_MANIFEST_KAPPA);

    let left = preflight_arm(&bundle, LEFT_PROMPT);
    let right = preflight_arm(&bundle, RIGHT_PROMPT);
    assert_eq!(
        AttentionQueryPolicy::PrimaryThenAdjacentSpinFallbackV1.identity(),
        PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY
    );
    assert_eq!(
        AttentionQueryPolicy::PrimaryThenAdjacentSpinFallbackV1.identity_kappa(),
        QUERY_POLICY_KAPPA
    );
    assert!(preflight_matches_frozen_contract(&left));
    assert!(preflight_matches_frozen_contract(&right));
    assert!(preflight_arms_have_equal_support_and_work(&left, &right));

    let record = RepairedPreflightRecord {
        schema: 2,
        prior_support_record_kappa: PRIOR_SUPPORT_PREFLIGHT_RECORD_KAPPA.to_owned(),
        fixture_kappa: FIXTURE_KAPPA.to_owned(),
        codec_kappa: CODEC_KAPPA.to_owned(),
        vocabulary_kappa: VOCABULARY_KAPPA.to_owned(),
        construction_artifact_kappa: CONSTRUCTION_ARTIFACT_KAPPA.to_owned(),
        attention_manifest_kappa: ATTENTION_MANIFEST_KAPPA.to_owned(),
        query_policy: PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY.to_owned(),
        query_policy_kappa: QUERY_POLICY_KAPPA.to_owned(),
        registered_surfaces: 8,
        observed_routes_per_completed_arm: 7,
        left,
        right,
    };
    let bytes = serde_json::to_vec(&record).unwrap();
    let record_kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    println!("repaired_support_preflight_record_kappa={record_kappa}");
    assert_eq!(record_kappa, REPAIRED_SUPPORT_PREFLIGHT_RECORD_KAPPA);
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerationSupportSignature {
    rows: usize,
    query_policy: String,
    query_policy_kappa: String,
    fallback_active: bool,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
    unique_candidates_before_ceiling: usize,
    memory_keys_per_candidate: usize,
    path_geometry_evaluations: usize,
    candidates: Vec<(Vec<u8>, [u32; 5])>,
}

fn support_signature(
    report: &LocalGeometricGenerationReport,
    step_index: usize,
) -> Option<GenerationSupportSignature> {
    let step = report.steps.get(step_index)?;
    let mut candidates = step
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
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    Some(GenerationSupportSignature {
        rows: step.support_rows.len(),
        query_policy: step.query_policy.clone(),
        query_policy_kappa: step.query_policy_kappa.clone(),
        fallback_active: step.fallback_active,
        candidate_entries_available: step.candidate_entries_available,
        candidate_entries_examined: step.candidate_entries_examined,
        candidate_entries_admitted: step.candidate_entries_admitted,
        unique_candidates_before_ceiling: step.unique_candidates_before_ceiling,
        memory_keys_per_candidate: step.memory_keys_per_candidate,
        path_geometry_evaluations: step.path_geometry_evaluations,
        candidates,
    })
}

fn exact_inversion_holds(
    artifact: &CanonicalRouteArtifact,
    report: &LocalGeometricGenerationReport,
) -> bool {
    report.steps.iter().all(|step| {
        step.candidates.iter().all(|candidate| {
            artifact
                .lexical_route_value_for_address(
                    &artifact
                        .lexical_route_address(candidate.lexical_unit_id)
                        .ok()
                        .flatten()
                        .unwrap(),
                )
                .ok()
                .flatten()
                .is_some_and(|value| {
                    value.address_kappa == candidate.address_kappa
                        && value.payload_cid == candidate.payload_cid
                        && value.payload_bytes == candidate.payload_bytes
                })
        }) && step.selected.as_ref().is_none_or(|selected| {
            artifact
                .lexical_route_value_for_address(
                    &artifact
                        .lexical_route_address(selected.lexical_unit_id)
                        .ok()
                        .flatten()
                        .unwrap(),
                )
                .ok()
                .flatten()
                .is_some_and(|value| {
                    value.address_kappa == selected.address_kappa
                        && value.payload_cid == selected.payload_cid
                        && value.payload_bytes == selected.payload_bytes
                })
        })
    })
}

fn bounded_closure_holds(report: &LocalGeometricGenerationReport) -> bool {
    report.stop_reason == LocalGenerationStopReason::ContinuationCap
        && report.steps.len() == CONTINUATION_CAP
        && report.emitted_lexical_unit_ids.len() == CONTINUATION_CAP
        && report.prompt_routes.len() + report.emitted_lexical_unit_ids.len() == 7
        && report.detected_cycle_period.is_none()
        && report.steps.iter().all(|step| {
            step.detected_cycle_period.is_none()
                && step.selected.as_ref().is_some_and(|selected| {
                    selected.observed_routes_after_append == step.observed_routes_after
                })
        })
}

fn source_boundary_holds(report: &LocalGeometricGenerationReport) -> bool {
    let boundary = &report.source_boundary;
    boundary.artifact_provenance_validated
        && boundary.artifact_input_reconstructed
        && boundary.schema2_rebuild_witnesses_compiled
        && boundary.source_weight_reads == 0
        && boundary.teacher_forwards == 0
        && boundary.provider_calls == 0
        && boundary.source_attention_calls == 0
        && boundary.learned_router_calls == 0
        && boundary.dense_matrix_operations == 0
        && boundary.selection_future_event_reads == 0
        && boundary.selection_paragraph_conversation_global_reads == 0
}

fn decisive_disabled_outcome(
    report: &LocalGeometricGenerationReport,
) -> Option<(bool, bool, Option<Vec<u8>>)> {
    let step = report.steps.get(1)?;
    Some((
        step.abstained,
        step.tie,
        step.selected
            .as_ref()
            .map(|selected| selected.payload_bytes.clone()),
    ))
}

#[derive(Serialize)]
struct FourArmRecord<'a> {
    schema: u32,
    fixture_kappa: &'a str,
    codec_kappa: &'a str,
    vocabulary_kappa: &'a str,
    construction_artifact_kappa: &'a str,
    attention_manifest_kappa: &'a str,
    query_policy: &'a str,
    query_policy_kappa: &'a str,
    support_preflight_record_kappa: &'a str,
    frozen_left_expected: &'a [&'a [u8]],
    frozen_right_expected: &'a [&'a [u8]],
    terminal: &'a str,
    left_full: &'a LocalGeometricGenerationReport,
    right_full: &'a LocalGeometricGenerationReport,
    left_state_disabled: &'a LocalGeometricGenerationReport,
    right_state_disabled: &'a LocalGeometricGenerationReport,
}

#[test]
#[ignore = "explicit frozen four-arm evidence; excluded from routine QA"]
fn natural_agreement_four_arm_witness() {
    let bundle = frozen_bundle();

    // Fail closed before constructing the generator or opening any H4 path.
    // An explicit ignored-test invocation cannot bypass the published support
    // gate merely because the freeze and negative record now exist.
    let left_preflight = preflight_arm(&bundle, LEFT_PROMPT);
    let right_preflight = preflight_arm(&bundle, RIGHT_PROMPT);
    assert!(
        preflight_matches_frozen_contract(&left_preflight)
            && preflight_matches_frozen_contract(&right_preflight)
            && preflight_arms_have_equal_support_and_work(&left_preflight, &right_preflight),
        "NOT_RUN_SUPPORT_PREFLIGHT_HARD_STOP"
    );

    let artifact_bytes = bundle.artifact.canonical_bytes().unwrap();
    let generator = LocalGeometricGenerator::from_canonical_bytes(&artifact_bytes).unwrap();

    // Exactly four experimental arms.
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

    let reports = [&left_full, &right_full, &left_disabled, &right_disabled];
    let first_emissions_are_still = reports.iter().all(|report| {
        report
            .steps
            .first()
            .and_then(|step| step.selected.as_ref())
            .is_some_and(|selected| selected.payload_bytes == b"still")
    });
    let left_full_matches = emitted_payloads(&left_full) == LEFT_EXPECTED;
    let right_full_matches = emitted_payloads(&right_full) == RIGHT_EXPECTED;
    let full_choices_are_distinct = left_full
        .steps
        .get(1)
        .and_then(|step| step.selected.as_ref())
        .zip(
            right_full
                .steps
                .get(1)
                .and_then(|step| step.selected.as_ref()),
        )
        .is_some_and(|(left, right)| left.address_kappa != right.address_kappa);
    let disabled_prompt_inert = decisive_disabled_outcome(&left_disabled)
        .zip(decisive_disabled_outcome(&right_disabled))
        .is_some_and(|(left, right)| left == right);
    let support_and_work_match = (0..CONTINUATION_CAP).all(|step_index| {
        let signatures = reports
            .iter()
            .map(|report| support_signature(report, step_index))
            .collect::<Vec<_>>();
        signatures
            .iter()
            .all(|signature| signature == &signatures[0])
    });
    let inversion_holds = reports
        .iter()
        .all(|report| exact_inversion_holds(&bundle.artifact, report));
    let bounded_closure = reports.iter().all(|report| bounded_closure_holds(report));
    let source_boundary = reports.iter().all(|report| source_boundary_holds(report));

    // The only repeat is a complete four-arm replay for deterministic bytes.
    let replay = [
        generator
            .generate(
                LEFT_PROMPT,
                LocalGenerationControl::FullPath,
                CONTINUATION_CAP,
            )
            .unwrap(),
        generator
            .generate(
                RIGHT_PROMPT,
                LocalGenerationControl::FullPath,
                CONTINUATION_CAP,
            )
            .unwrap(),
        generator
            .generate(
                LEFT_PROMPT,
                LocalGenerationControl::StateDisabled,
                CONTINUATION_CAP,
            )
            .unwrap(),
        generator
            .generate(
                RIGHT_PROMPT,
                LocalGenerationControl::StateDisabled,
                CONTINUATION_CAP,
            )
            .unwrap(),
    ];
    let replay_is_identical = reports.iter().zip(&replay).all(|(first, second)| {
        *first == second && first.canonical_bytes().unwrap() == second.canonical_bytes().unwrap()
    });

    let positive = first_emissions_are_still
        && left_full_matches
        && right_full_matches
        && full_choices_are_distinct
        && disabled_prompt_inert
        && support_and_work_match
        && inversion_holds
        && bounded_closure
        && source_boundary
        && replay_is_identical;
    let terminal = if positive {
        POSITIVE_TERMINAL
    } else {
        REVISION_TERMINAL
    };

    // Freeze the observed negative branch. Admission is now exact, but the
    // decisive full-path choice is still `run` for both prompt orders.
    assert!(first_emissions_are_still);
    assert!(!left_full_matches);
    assert!(right_full_matches);
    assert!(!full_choices_are_distinct);
    assert!(disabled_prompt_inert);
    assert!(support_and_work_match);
    assert!(inversion_holds);
    assert!(bounded_closure);
    assert!(source_boundary);
    assert!(replay_is_identical);
    assert_eq!(terminal, REVISION_TERMINAL);

    let record = FourArmRecord {
        schema: 2,
        fixture_kappa: FIXTURE_KAPPA,
        codec_kappa: CODEC_KAPPA,
        vocabulary_kappa: VOCABULARY_KAPPA,
        construction_artifact_kappa: CONSTRUCTION_ARTIFACT_KAPPA,
        attention_manifest_kappa: ATTENTION_MANIFEST_KAPPA,
        query_policy: PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY,
        query_policy_kappa: QUERY_POLICY_KAPPA,
        support_preflight_record_kappa: REPAIRED_SUPPORT_PREFLIGHT_RECORD_KAPPA,
        frozen_left_expected: LEFT_EXPECTED,
        frozen_right_expected: RIGHT_EXPECTED,
        terminal,
        left_full: &left_full,
        right_full: &right_full,
        left_state_disabled: &left_disabled,
        right_state_disabled: &right_disabled,
    };
    let record_bytes = serde_json::to_vec(&record).unwrap();
    let record_kappa = format!("blake3:{}", blake3::hash(&record_bytes).to_hex());

    println!("terminal={terminal}");
    println!("left_full={:?}", emitted_payloads(&left_full));
    println!("right_full={:?}", emitted_payloads(&right_full));
    println!("left_state_disabled={:?}", emitted_payloads(&left_disabled));
    println!(
        "right_state_disabled={:?}",
        emitted_payloads(&right_disabled)
    );
    println!("four_arm_record_kappa={record_kappa}");

    if let Some(expected) = FOUR_ARM_RECORD_KAPPA {
        assert_eq!(record_kappa, expected);
    }
}
