use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use uor_r4_core::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec,
    CanonicalRouteArtifact, ConversationInput, H4RootCoordinate, ParagraphInput, TurnInput,
};
use uor_r4_core::local_geometric_generation::{
    LocalGenerationControl, LocalGenerationSourceCounts, LocalGenerationStopReason,
    LocalGeometricGenerationReport, LocalGeometricGenerator,
};
use uor_r4_core::prime_route_attention::GeometricAddress;
use uor_r4_core::prime_route_geometric_attention::{
    AttentionQueryPolicy, AttentionRowKey, AttentionRowSource, AttentionSourceCounts,
    AttentionSupportTrace, GeometricAttentionArtifact, H4S3AngularShell,
    LocalContextPlacementControl, LocalContextPlacementCost, LocalContextPlacementRelationCensus,
    LocalContextTrajectory, LocalSameObjectContextPlacementV1,
    EXACT_RECENT_SUFFIX_H4_SHELL_VECTOR_V1_IDENTITY, LOCAL_CONTEXT_PLACEMENT_FRAME_MISMATCH,
    LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH, LOCAL_CONTEXT_PLACEMENT_MAX_PROTOTYPES_PER_CANDIDATE,
    LOCAL_SAME_OBJECT_CONTEXT_PLACEMENT_V1_IDENTITY,
    PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY,
    RECENT_SUFFIX_ORDERED_H4_TRAJECTORY_V1_IDENTITY,
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

// Unlike FIXTURE_IDENTITY_BYTES, this selection-blind input identity contains
// no expected continuation or terminal that could attach a held-out label to
// construction, support, trajectory encoding, or relation enumeration.
const LOCAL_CONTEXT_PREFLIGHT_INPUT_BYTES: &[u8] =
    b"uor-r4.local-context-placement-preflight-input/1\
\nidentity_scope=issue-953/natural-agreement-v1\
\nturn_id=turn-0001\
\nglobal_snapshot=near\
\nconstruction_0001=athletes generally still run\
\nconstruction_0002=one athlete generally still runs\
\nregistered_surfaces=athlete|athletes|generally|near|one|run|runs|still\
\nleft_prompt=one athlete near athletes generally\
\nright_prompt=athletes near one athlete generally\
\nadmission_policy=uor-r4.attention-query-policy/primary-then-adjacent-spin-fallback-v1\
\nplacement_policy=uor-r4.local-same-object-context-placement/1\
\nencoder=uor-r4.recent-suffix-ordered-h4-trajectory/1\
\nrelation=uor-r4.exact-recent-suffix-h4-shell-vector/1\
\ncontrols=real-placement|placement-permuted|order-shuffled\
\nframe_width=4\
\npadding=typed-h4-identity-with-explicit-population-mask\
\nslot_order=recent-suffix-lengths-1-2-3-4\
\ncomparison_order=lexicographic-shortest-suffix-first\
\nprototype_cap_per_candidate=4";

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

// These #953 placement identities are frozen from the selection-blind
// relation census before its vectors are attached to the held-out expected
// continuations. The decoded generator is not invoked by either preflight.
const LOCAL_CONTEXT_PLACEMENT_POLICY_KAPPA: Option<&str> =
    Some("blake3:e09af2db10f41efaf02b24e075a97fe42dc966834c43566689579763ba95b49c");
const LOCAL_CONTEXT_PLACEMENT_OVERLAY_KAPPA: Option<&str> =
    Some("blake3:2be03c8acc1e97d1ba830805653ec1b16745065127ee1e00cb431b933a173ee1");
const LOCAL_CONTEXT_PREFLIGHT_INPUT_KAPPA: Option<&str> =
    Some("blake3:ce5430048c3789e85e18ed17a80f10f79d317a66769ad8be0fd28224668eb72e");
const LOCAL_CONTEXT_RELATION_CENSUS_KAPPA: Option<&str> =
    Some("blake3:50e67c087e1ec5e04aa47cf09d42e9b0857c15e4cafa07b1363d67faf96c6aeb");
const LOCAL_CONTEXT_PREFLIGHT_OUTCOME_KAPPA: Option<&str> =
    Some("blake3:d5e2f3614c8f1d3c6c629e2261ec42dc970fa5484982b2a86cb4f4b06b06a372");

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

fn local_context_preflight_input_kappa() -> String {
    format!(
        "blake3:{}",
        blake3::hash(LOCAL_CONTEXT_PREFLIGHT_INPUT_BYTES).to_hex()
    )
}

fn local_context_placement_generation_authorized() -> bool {
    false
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

fn decisive_history_from_singleton_support(
    bundle: &FrozenBundle,
    prompt: &[u8],
) -> (Vec<GeometricAddress>, AttentionSupportTrace) {
    let mut history = history_for_prompt(bundle, prompt);
    assert_eq!(history.len(), 5);
    let state = bundle
        .attention
        .causal_state_from_history(&history)
        .unwrap();
    let support = bundle.attention.query_support_only(&state).unwrap();
    assert_eq!(support.candidates.len(), 1);
    let observed = support.candidates[0].next.clone();
    // This is an audit of the construction-derived singleton output, never a
    // literal candidate supplied to the causal history.
    assert_eq!(payload_for_address(bundle, &observed), b"still");
    history.push(observed);
    (history, support)
}

fn preflight_arm(bundle: &FrozenBundle, prompt: &[u8]) -> RepairedPreflightArm {
    let (history, first_support) = decisive_history_from_singleton_support(bundle, prompt);
    let prompt_history = &history[..history.len() - 1];
    let mut state = bundle
        .attention
        .causal_state_from_history(prompt_history)
        .unwrap();

    // This method has no H4 table, state, cost, or selection input. The
    // projection records only bounded lookup keys, tier state, and support.
    let first = preflight_step(bundle, prompt_history.len(), first_support);
    bundle
        .attention
        .observe(&mut state, history.last().unwrap().clone())
        .unwrap();
    let second = preflight_step(
        bundle,
        history.len(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementTrajectoryRecord {
    populated: [bool; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
    suffix_states: [H4RootCoordinate; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
}

impl From<&LocalContextTrajectory> for PlacementTrajectoryRecord {
    fn from(trajectory: &LocalContextTrajectory) -> Self {
        Self {
            populated: trajectory.populated,
            suffix_states: trajectory.suffix_states,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementPrototypeRecord {
    candidate_payload_bytes: Vec<u8>,
    candidate_address_kappa: String,
    sentence_id: String,
    transition_ordinal: usize,
    predecessor_length: usize,
    predecessor_history_kappa: String,
    predecessor_address_kappas: Vec<String>,
    trajectory: PlacementTrajectoryRecord,
    order_shuffled_trajectory: PlacementTrajectoryRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementPrototypeRelationRecord {
    sentence_id: String,
    transition_ordinal: usize,
    predecessor_history_kappa: String,
    prototype_trajectory: PlacementTrajectoryRecord,
    population_match: [bool; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
    padding_identity_alias: [bool; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
    exact_slot_match: [bool; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
    relative_states: [H4RootCoordinate; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH],
    cost: LocalContextPlacementCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementCandidateRelationRecord {
    candidate_payload_bytes: Vec<u8>,
    candidate_address_kappa: String,
    source_counts: [u32; 5],
    prototype_source_payload_bytes: Option<Vec<u8>>,
    prototype_source_address_kappa: Option<String>,
    prototype_count: usize,
    prototype_relations: Vec<PlacementPrototypeRelationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementControlRecord {
    control: LocalContextPlacementControl,
    manifest_kappa: String,
    placement_policy: String,
    placement_policy_kappa: String,
    overlay_kappa: String,
    live_trajectory: PlacementTrajectoryRecord,
    query_policy: String,
    query_policy_kappa: String,
    fallback_active: bool,
    support_rows: usize,
    candidate_entries_available: usize,
    candidate_entries_examined: usize,
    candidate_entries_admitted: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    candidate_union: Vec<Vec<u8>>,
    inherited_path_keys_per_candidate: usize,
    inherited_path_comparisons: usize,
    complete_candidate_membership: bool,
    prototype_evaluations: usize,
    trajectory_slot_comparisons: usize,
    candidates: Vec<PlacementCandidateRelationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PlacementPromptRelationRecord {
    prompt_bytes: Vec<u8>,
    decisive_observed_routes: usize,
    real: PlacementControlRecord,
    placement_permuted: PlacementControlRecord,
    order_shuffled: PlacementControlRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalContextRelationCensusRecord {
    schema: u32,
    preflight_input_kappa: String,
    codec_kappa: String,
    vocabulary_kappa: String,
    construction_artifact_kappa: String,
    attention_manifest_kappa: String,
    query_policy: String,
    query_policy_kappa: String,
    overlay_schema: u32,
    overlay_policy: String,
    overlay_policy_kappa: String,
    encoder_identity: String,
    quantization_identity: String,
    overlay_kappa: String,
    h4_root_table_kappa: String,
    multiplication_table_kappa: String,
    frame_width: usize,
    padding_contract: String,
    slot_order: String,
    comparison_order: String,
    prototype_cap_per_candidate: usize,
    source_witnesses: usize,
    source_transitions: usize,
    retained_candidates: usize,
    retained_prototypes: usize,
    prototypes: Vec<PlacementPrototypeRecord>,
    compiler_query_reproductions: usize,
    exact_self_matches: usize,
    construction_class_collisions: usize,
    padding_identity_aliases: usize,
    decisive_order_shuffled_prototypes_distinct: usize,
    held_out_full_histories_absent_from_construction: bool,
    expected_continuation_inputs: usize,
    held_out_label_inputs: usize,
    future_route_inputs: usize,
    source_tensor_reads: usize,
    teacher_forwards: usize,
    provider_calls: usize,
    causal_singleton_support_observations: usize,
    selector_candidate_append_inputs: usize,
    frame_mismatch_terminal: String,
    left_support: RepairedPreflightArm,
    right_support: RepairedPreflightArm,
    left_relations: PlacementPromptRelationRecord,
    right_relations: PlacementPromptRelationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StrictMinimumRecord {
    winner_payload_bytes: Option<Vec<u8>>,
    tie: bool,
    unavailable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalContextPreflightOutcomeRecord {
    schema: u32,
    fixture_kappa: String,
    repaired_support_record_kappa: String,
    relation_census_kappa: String,
    frozen_left_expected: Vec<u8>,
    frozen_right_expected: Vec<u8>,
    left_real: StrictMinimumRecord,
    right_real: StrictMinimumRecord,
    left_placement_permuted: StrictMinimumRecord,
    right_placement_permuted: StrictMinimumRecord,
    left_order_shuffled: StrictMinimumRecord,
    right_order_shuffled: StrictMinimumRecord,
    real_intended_matches: usize,
    placement_permuted_intended_matches: usize,
    order_shuffled_intended_matches: usize,
    strict_selection_ceiling_numerator: usize,
    strict_selection_ceiling_denominator: usize,
    tie_rule: String,
    pass_local_context_placement_preflight: String,
    decoded_generation: String,
    failure: String,
    terminal: String,
}

fn trajectory_record(trajectory: &LocalContextTrajectory) -> PlacementTrajectoryRecord {
    trajectory.into()
}

fn placement_control_record(
    bundle: &FrozenBundle,
    history_len: usize,
    census: LocalContextPlacementRelationCensus,
) -> PlacementControlRecord {
    let LocalContextPlacementRelationCensus {
        manifest_kappa,
        policy_identity,
        policy_kappa,
        overlay_kappa,
        control,
        live_trajectory,
        support,
        complete_candidate_membership,
        prototype_evaluations,
        trajectory_slot_comparisons,
        candidates: census_candidates,
    } = census;
    assert_eq!(manifest_kappa, support.manifest_kappa);
    let mut candidate_union = support
        .candidates
        .iter()
        .map(|candidate| payload_for_address(bundle, &candidate.next))
        .collect::<Vec<_>>();
    candidate_union.sort();
    let candidates = census_candidates
        .into_iter()
        .map(|candidate| {
            let prototype_source_payload_bytes = Some(payload_for_address(
                bundle,
                &candidate.prototype_source_candidate,
            ));
            let prototype_source_address_kappa = Some(
                candidate
                    .prototype_source_candidate
                    .canonical_kappa()
                    .unwrap(),
            );
            let prototype_relations = candidate
                .prototype_relations
                .into_iter()
                .map(|relation| {
                    let population_match = [0usize, 1, 2, 3].map(|slot| {
                        relation.prototype_trajectory.populated[slot]
                            == live_trajectory.populated[slot]
                    });
                    let padding_identity_alias = [0usize, 1, 2, 3].map(|slot| {
                        !population_match[slot]
                            && relation.cost.suffix_shells[slot] == H4S3AngularShell::Coincident
                    });
                    let exact_slot_match = [0usize, 1, 2, 3].map(|slot| {
                        population_match[slot]
                            && relation.prototype_trajectory.suffix_states[slot]
                                == live_trajectory.suffix_states[slot]
                    });
                    PlacementPrototypeRelationRecord {
                        sentence_id: relation.sentence_id,
                        transition_ordinal: relation.transition_ordinal,
                        predecessor_history_kappa: relation.predecessor_history_kappa,
                        prototype_trajectory: trajectory_record(&relation.prototype_trajectory),
                        population_match,
                        padding_identity_alias,
                        exact_slot_match,
                        relative_states: relation.relative_states,
                        cost: relation.cost,
                    }
                })
                .collect::<Vec<_>>();
            PlacementCandidateRelationRecord {
                candidate_payload_bytes: payload_for_address(bundle, &candidate.candidate),
                candidate_address_kappa: candidate.candidate.canonical_kappa().unwrap(),
                source_counts: count_array(candidate.source_counts),
                prototype_source_payload_bytes,
                prototype_source_address_kappa,
                prototype_count: candidate.prototype_count,
                prototype_relations,
            }
        })
        .collect::<Vec<_>>();
    PlacementControlRecord {
        control,
        manifest_kappa,
        placement_policy: policy_identity,
        placement_policy_kappa: policy_kappa,
        overlay_kappa,
        live_trajectory: trajectory_record(&live_trajectory),
        query_policy: support.query_policy.identity().to_owned(),
        query_policy_kappa: support.query_policy_kappa,
        fallback_active: support.fallback_active,
        support_rows: support.rows_read.len(),
        candidate_entries_available: support.candidate_entries_available,
        candidate_entries_examined: support.candidate_entries_examined,
        candidate_entries_admitted: support.candidate_entries_admitted,
        candidate_entry_ceiling: support.candidate_entry_ceiling,
        unique_candidates_before_ceiling: support.unique_candidates_before_ceiling,
        candidate_ceiling: support.candidate_ceiling,
        candidate_union,
        inherited_path_keys_per_candidate: history_len,
        inherited_path_comparisons: history_len * candidates.len(),
        complete_candidate_membership,
        prototype_evaluations,
        trajectory_slot_comparisons,
        candidates,
    }
}

fn placement_prompt_record(
    bundle: &FrozenBundle,
    prompt: &[u8],
    overlay: &LocalSameObjectContextPlacementV1,
    table: &uor_r4_core::canonical_lexical_ingestion::H4BinaryIcosahedralClosure,
) -> PlacementPromptRelationRecord {
    let (history, _) = decisive_history_from_singleton_support(bundle, prompt);
    let history_len = history.len();
    let real = placement_control_record(
        bundle,
        history_len,
        overlay
            .relation_census_for_history(
                &bundle.attention,
                &history,
                table,
                LocalContextPlacementControl::RealPlacement,
            )
            .unwrap(),
    );
    let placement_permuted = placement_control_record(
        bundle,
        history_len,
        overlay
            .relation_census_for_history(
                &bundle.attention,
                &history,
                table,
                LocalContextPlacementControl::PlacementPermuted,
            )
            .unwrap(),
    );
    let order_shuffled = placement_control_record(
        bundle,
        history_len,
        overlay
            .relation_census_for_history(
                &bundle.attention,
                &history,
                table,
                LocalContextPlacementControl::OrderShuffled,
            )
            .unwrap(),
    );
    for control in [&placement_permuted, &order_shuffled] {
        assert_eq!(control.manifest_kappa, real.manifest_kappa);
        assert_eq!(control.placement_policy, real.placement_policy);
        assert_eq!(control.placement_policy_kappa, real.placement_policy_kappa);
        assert_eq!(control.overlay_kappa, real.overlay_kappa);
        assert_eq!(control.query_policy, real.query_policy);
        assert_eq!(control.query_policy_kappa, real.query_policy_kappa);
        assert_eq!(control.fallback_active, real.fallback_active);
        assert_eq!(control.support_rows, real.support_rows);
        assert_eq!(
            control.candidate_entries_available,
            real.candidate_entries_available
        );
        assert_eq!(
            control.candidate_entries_examined,
            real.candidate_entries_examined
        );
        assert_eq!(
            control.candidate_entries_admitted,
            real.candidate_entries_admitted
        );
        assert_eq!(
            control.candidate_entry_ceiling,
            real.candidate_entry_ceiling
        );
        assert_eq!(
            control.unique_candidates_before_ceiling,
            real.unique_candidates_before_ceiling
        );
        assert_eq!(control.candidate_ceiling, real.candidate_ceiling);
        assert_eq!(control.candidate_union, real.candidate_union);
        assert_eq!(
            control.inherited_path_keys_per_candidate,
            real.inherited_path_keys_per_candidate
        );
        assert_eq!(
            control.inherited_path_comparisons,
            real.inherited_path_comparisons
        );
    }
    PlacementPromptRelationRecord {
        prompt_bytes: prompt.to_vec(),
        decisive_observed_routes: history_len,
        real,
        placement_permuted,
        order_shuffled,
    }
}

fn strict_minimum(control: &PlacementControlRecord) -> StrictMinimumRecord {
    if !control.complete_candidate_membership
        || control.candidates.is_empty()
        || control
            .candidates
            .iter()
            .any(|candidate| candidate.prototype_relations.is_empty())
    {
        return StrictMinimumRecord {
            winner_payload_bytes: None,
            tie: false,
            unavailable: true,
        };
    }
    let minimum = control
        .candidates
        .iter()
        .filter_map(candidate_minimum_cost)
        .min()
        .unwrap();
    let winners = control
        .candidates
        .iter()
        .filter(|candidate| candidate_minimum_cost(candidate) == Some(minimum))
        .collect::<Vec<_>>();
    if winners.len() == 1 {
        StrictMinimumRecord {
            winner_payload_bytes: Some(winners[0].candidate_payload_bytes.clone()),
            tie: false,
            unavailable: false,
        }
    } else {
        StrictMinimumRecord {
            winner_payload_bytes: None,
            tie: true,
            unavailable: false,
        }
    }
}

fn candidate_minimum_cost(
    candidate: &PlacementCandidateRelationRecord,
) -> Option<LocalContextPlacementCost> {
    candidate
        .prototype_relations
        .iter()
        .map(|relation| relation.cost)
        .min()
}

fn placement_candidate<'a>(
    control: &'a PlacementControlRecord,
    payload: &[u8],
) -> &'a PlacementCandidateRelationRecord {
    control
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_payload_bytes == payload)
        .unwrap()
}

fn padding_identity_aliases(record: &PlacementPromptRelationRecord) -> usize {
    [
        &record.real,
        &record.placement_permuted,
        &record.order_shuffled,
    ]
    .into_iter()
    .flat_map(|control| &control.candidates)
    .flat_map(|candidate| &candidate.prototype_relations)
    .map(|relation| {
        relation
            .padding_identity_alias
            .iter()
            .filter(|alias| **alias)
            .count()
    })
    .sum()
}

fn relation_census_record(bundle: &FrozenBundle) -> LocalContextRelationCensusRecord {
    let manifest = bundle.artifact.embedded_spin_manifest().unwrap();
    let table = validate_h4_binary_icosahedral_closure().unwrap();
    let overlay =
        LocalSameObjectContextPlacementV1::compile_from_manifest_witnesses(&manifest, &table)
            .unwrap();
    let replay =
        LocalSameObjectContextPlacementV1::compile_from_manifest_witnesses(&manifest, &table)
            .unwrap();
    assert_eq!(overlay, replay);
    assert_eq!(
        overlay.canonical_bytes().unwrap(),
        replay.canonical_bytes().unwrap()
    );
    assert_eq!(
        overlay.overlay_kappa(),
        overlay.reproduce_overlay_kappa().unwrap()
    );

    let prototypes = overlay
        .prototype_sets()
        .iter()
        .flat_map(|set| {
            set.prototypes
                .iter()
                .map(|prototype| PlacementPrototypeRecord {
                    candidate_payload_bytes: payload_for_address(bundle, &set.candidate),
                    candidate_address_kappa: set.candidate.canonical_kappa().unwrap(),
                    sentence_id: prototype.sentence_id.clone(),
                    transition_ordinal: prototype.transition_ordinal,
                    predecessor_length: prototype.predecessor_history.len(),
                    predecessor_history_kappa: prototype.predecessor_history_kappa.clone(),
                    predecessor_address_kappas: prototype
                        .predecessor_history
                        .iter()
                        .map(GeometricAddress::canonical_kappa)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                    trajectory: trajectory_record(&prototype.trajectory),
                    order_shuffled_trajectory: trajectory_record(
                        &prototype.order_shuffled_trajectory,
                    ),
                })
        })
        .collect::<Vec<_>>();

    let compiler_query_reproductions = overlay
        .prototype_sets()
        .iter()
        .flat_map(|set| &set.prototypes)
        .filter(|prototype| {
            overlay
                .encode_query_history(&bundle.attention, &prototype.predecessor_history, &table)
                .is_ok_and(|trajectory| trajectory == prototype.trajectory)
        })
        .count();
    let all_prototypes = overlay.retained_prototypes();
    assert_eq!(compiler_query_reproductions, all_prototypes);
    for prototype in overlay
        .prototype_sets()
        .iter()
        .flat_map(|set| &set.prototypes)
    {
        let mut reversed_history = prototype.predecessor_history.clone();
        reversed_history.reverse();
        let mut original_multiset = prototype
            .predecessor_history
            .iter()
            .map(GeometricAddress::canonical_kappa)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let mut reversed_multiset = reversed_history
            .iter()
            .map(GeometricAddress::canonical_kappa)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        original_multiset.sort();
        reversed_multiset.sort();
        assert_eq!(original_multiset, reversed_multiset);
        assert_eq!(
            overlay
                .encode_query_history(&bundle.attention, &reversed_history, &table)
                .unwrap(),
            prototype.order_shuffled_trajectory
        );
    }

    let exact_self_matches = overlay
        .prototype_sets()
        .iter()
        .flat_map(|set| {
            set.prototypes.iter().map(|prototype| {
                let census = overlay
                    .relation_census_for_history(
                        &bundle.attention,
                        &prototype.predecessor_history,
                        &table,
                        LocalContextPlacementControl::RealPlacement,
                    )
                    .unwrap();
                census
                    .candidates
                    .iter()
                    .find(|candidate| candidate.candidate == set.candidate)
                    .and_then(|candidate| {
                        candidate
                            .prototype_relations
                            .iter()
                            .map(|relation| relation.cost)
                            .min()
                    })
                    .is_some_and(|cost| {
                        cost.suffix_shells
                            == [H4S3AngularShell::Coincident; LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH]
                    })
            })
        })
        .filter(|matched| *matched)
        .count();

    let mut construction_classes = BTreeMap::<Vec<u8>, BTreeSet<Vec<u8>>>::new();
    for prototype in &prototypes {
        construction_classes
            .entry(serde_json::to_vec(&prototype.trajectory).unwrap())
            .or_default()
            .insert(prototype.candidate_payload_bytes.clone());
    }
    let construction_class_collisions = construction_classes
        .values()
        .filter(|candidates| candidates.len() > 1)
        .count();

    let decisive_order_shuffled_prototypes_distinct = prototypes
        .iter()
        .filter(|prototype| {
            [b"run".as_slice(), b"runs".as_slice()]
                .contains(&prototype.candidate_payload_bytes.as_slice())
                && prototype.trajectory != prototype.order_shuffled_trajectory
        })
        .count();
    let (left_decisive_history, _) = decisive_history_from_singleton_support(bundle, LEFT_PROMPT);
    let (right_decisive_history, _) = decisive_history_from_singleton_support(bundle, RIGHT_PROMPT);
    let held_out_full_histories_absent_from_construction = overlay
        .prototype_sets()
        .iter()
        .flat_map(|set| &set.prototypes)
        .all(|prototype| {
            prototype.predecessor_history != left_decisive_history
                && prototype.predecessor_history != right_decisive_history
        });

    let mut mismatched_table = table.clone();
    mismatched_table.h4_root_table_kappa =
        "blake3:0000000000000000000000000000000000000000000000000000000000000000".to_owned();
    let mismatch = overlay
        .encode_query_history(&bundle.attention, &left_decisive_history, &mismatched_table)
        .unwrap_err();
    assert!(mismatch
        .to_string()
        .ends_with(LOCAL_CONTEXT_PLACEMENT_FRAME_MISMATCH));

    let left_relations = placement_prompt_record(bundle, LEFT_PROMPT, &overlay, &table);
    let right_relations = placement_prompt_record(bundle, RIGHT_PROMPT, &overlay, &table);
    let padding_identity_aliases =
        padding_identity_aliases(&left_relations) + padding_identity_aliases(&right_relations);

    LocalContextRelationCensusRecord {
        schema: 2,
        preflight_input_kappa: local_context_preflight_input_kappa(),
        codec_kappa: CODEC_KAPPA.to_owned(),
        vocabulary_kappa: VOCABULARY_KAPPA.to_owned(),
        construction_artifact_kappa: CONSTRUCTION_ARTIFACT_KAPPA.to_owned(),
        attention_manifest_kappa: ATTENTION_MANIFEST_KAPPA.to_owned(),
        query_policy: PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY.to_owned(),
        query_policy_kappa: QUERY_POLICY_KAPPA.to_owned(),
        overlay_schema: overlay.schema(),
        overlay_policy: overlay.policy_identity().to_owned(),
        overlay_policy_kappa: overlay.policy_kappa().to_owned(),
        encoder_identity: overlay.encoder_identity().to_owned(),
        quantization_identity: overlay.quantization_identity().to_owned(),
        overlay_kappa: overlay.overlay_kappa().to_owned(),
        h4_root_table_kappa: overlay.h4_root_table_kappa().to_owned(),
        multiplication_table_kappa: overlay.multiplication_table_kappa().to_owned(),
        frame_width: LOCAL_CONTEXT_PLACEMENT_FRAME_WIDTH,
        padding_contract: "typed-h4-identity-with-explicit-population-mask".to_owned(),
        slot_order: "recent-suffix-lengths-1-2-3-4".to_owned(),
        comparison_order: "lexicographic-shortest-suffix-first".to_owned(),
        prototype_cap_per_candidate: LOCAL_CONTEXT_PLACEMENT_MAX_PROTOTYPES_PER_CANDIDATE,
        source_witnesses: overlay.source_witnesses(),
        source_transitions: overlay.source_transitions(),
        retained_candidates: overlay.prototype_sets().len(),
        retained_prototypes: overlay.retained_prototypes(),
        prototypes,
        compiler_query_reproductions,
        exact_self_matches,
        construction_class_collisions,
        padding_identity_aliases,
        decisive_order_shuffled_prototypes_distinct,
        held_out_full_histories_absent_from_construction,
        expected_continuation_inputs: 0,
        held_out_label_inputs: 0,
        future_route_inputs: 0,
        source_tensor_reads: 0,
        teacher_forwards: 0,
        provider_calls: 0,
        causal_singleton_support_observations: 2,
        selector_candidate_append_inputs: 0,
        frame_mismatch_terminal: LOCAL_CONTEXT_PLACEMENT_FRAME_MISMATCH.to_owned(),
        left_support: preflight_arm(bundle, LEFT_PROMPT),
        right_support: preflight_arm(bundle, RIGHT_PROMPT),
        left_relations,
        right_relations,
    }
}

#[test]
fn freeze_local_context_placement_overlay_and_relation_census() {
    let bundle = frozen_bundle();
    let record = relation_census_record(&bundle);
    let bytes = serde_json::to_vec(&record).unwrap();
    let record_kappa = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    if let Some(expected) = LOCAL_CONTEXT_PREFLIGHT_INPUT_KAPPA {
        assert_eq!(record.preflight_input_kappa, expected);
    }
    if let Some(expected) = LOCAL_CONTEXT_RELATION_CENSUS_KAPPA {
        assert_eq!(record_kappa, expected);
    }
    assert_eq!(
        record.overlay_policy,
        LOCAL_SAME_OBJECT_CONTEXT_PLACEMENT_V1_IDENTITY
    );
    assert_eq!(
        record.encoder_identity,
        RECENT_SUFFIX_ORDERED_H4_TRAJECTORY_V1_IDENTITY
    );
    assert_eq!(
        record.quantization_identity,
        EXACT_RECENT_SUFFIX_H4_SHELL_VECTOR_V1_IDENTITY
    );
    assert_eq!(record.frame_width, 4);
    assert_eq!(record.prototype_cap_per_candidate, 4);
    assert_eq!(
        record.compiler_query_reproductions,
        record.retained_prototypes
    );
    assert_eq!(record.exact_self_matches, record.retained_prototypes);
    assert_eq!(record.construction_class_collisions, 0);
    assert_eq!(record.padding_identity_aliases, 0);
    assert_eq!(record.decisive_order_shuffled_prototypes_distinct, 2);
    assert!(record.held_out_full_histories_absent_from_construction);
    let prototype = |payload: &[u8]| {
        record
            .prototypes
            .iter()
            .find(|prototype| prototype.candidate_payload_bytes == payload)
            .unwrap()
    };
    assert_eq!(prototype(b"run").predecessor_length, 3);
    assert_eq!(
        prototype(b"run").trajectory.populated,
        [true, true, true, false]
    );
    assert_eq!(prototype(b"runs").predecessor_length, 4);
    assert_eq!(
        prototype(b"runs").trajectory.populated,
        [true, true, true, true]
    );
    assert!(preflight_matches_frozen_contract(&record.left_support));
    assert!(preflight_matches_frozen_contract(&record.right_support));
    assert!(preflight_arms_have_equal_support_and_work(
        &record.left_support,
        &record.right_support
    ));

    for prompt in [&record.left_relations, &record.right_relations] {
        for control in [
            &prompt.real,
            &prompt.placement_permuted,
            &prompt.order_shuffled,
        ] {
            assert_eq!(control.candidate_union, [b"run".to_vec(), b"runs".to_vec()]);
            assert_eq!(control.support_rows, 7);
            assert_eq!(control.candidate_entries_available, 11);
            assert_eq!(control.candidate_entries_examined, 6);
            assert_eq!(control.candidate_entries_admitted, 6);
            assert_eq!(control.manifest_kappa, ATTENTION_MANIFEST_KAPPA);
            assert_eq!(
                control.placement_policy,
                LOCAL_SAME_OBJECT_CONTEXT_PLACEMENT_V1_IDENTITY
            );
            assert_eq!(control.placement_policy_kappa, record.overlay_policy_kappa);
            assert_eq!(control.overlay_kappa, record.overlay_kappa);
            assert_eq!(
                control.query_policy,
                PRIMARY_THEN_ADJACENT_SPIN_FALLBACK_V1_IDENTITY
            );
            assert_eq!(control.query_policy_kappa, QUERY_POLICY_KAPPA);
            assert!(!control.fallback_active);
            assert_eq!(control.candidate_entry_ceiling, 56);
            assert_eq!(control.unique_candidates_before_ceiling, 2);
            assert_eq!(control.candidate_ceiling, 8);
            assert!(control.complete_candidate_membership);
            assert_eq!(control.inherited_path_keys_per_candidate, 6);
            assert_eq!(control.inherited_path_comparisons, 12);
            assert_eq!(control.prototype_evaluations, 2);
            assert_eq!(control.trajectory_slot_comparisons, 8);
        }
        assert_ne!(prompt.real.candidates, prompt.placement_permuted.candidates);
        assert_ne!(prompt.real.candidates, prompt.order_shuffled.candidates);
        assert_eq!(
            prompt.real.candidates.len(),
            prompt.placement_permuted.candidates.len()
        );
        for candidate_index in 0..prompt.real.candidates.len() {
            let real_candidate = &prompt.real.candidates[candidate_index];
            let permuted_candidate = &prompt.placement_permuted.candidates[candidate_index];
            let shuffled_candidate = &prompt.order_shuffled.candidates[candidate_index];
            let cyclic_source =
                &prompt.real.candidates[(candidate_index + 1) % prompt.real.candidates.len()];
            assert_eq!(
                real_candidate.candidate_address_kappa,
                permuted_candidate.candidate_address_kappa
            );
            assert_eq!(
                real_candidate.candidate_address_kappa,
                shuffled_candidate.candidate_address_kappa
            );
            assert_eq!(
                real_candidate.prototype_source_address_kappa.as_ref(),
                Some(&real_candidate.candidate_address_kappa)
            );
            assert_eq!(
                permuted_candidate.prototype_source_address_kappa.as_ref(),
                Some(&cyclic_source.candidate_address_kappa)
            );
            assert_eq!(
                shuffled_candidate.prototype_source_address_kappa.as_ref(),
                Some(&shuffled_candidate.candidate_address_kappa)
            );
        }
        assert_ne!(
            candidate_minimum_cost(&prompt.real.candidates[0]),
            candidate_minimum_cost(&prompt.real.candidates[1])
        );
    }

    println!(
        "local_context_placement_policy_kappa={}",
        record.overlay_policy_kappa
    );
    println!(
        "local_context_placement_overlay_kappa={}",
        record.overlay_kappa
    );
    println!(
        "local_context_preflight_input_kappa={}",
        record.preflight_input_kappa
    );
    println!("local_context_relation_census_kappa={record_kappa}");
    println!(
        "inventory=source_witnesses:{},source_transitions:{},retained_candidates:{},retained_prototypes:{},class_collisions:{},compiler_query_reproductions:{},self_matches:{}",
        record.source_witnesses,
        record.source_transitions,
        record.retained_candidates,
        record.retained_prototypes,
        record.construction_class_collisions,
        record.compiler_query_reproductions,
        record.exact_self_matches,
    );
    for prototype in record.prototypes.iter().filter(|prototype| {
        [b"run".as_slice(), b"runs".as_slice()]
            .contains(&prototype.candidate_payload_bytes.as_slice())
    }) {
        println!(
            "prototype={} predecessor_length={} populated={:?}",
            String::from_utf8_lossy(&prototype.candidate_payload_bytes),
            prototype.predecessor_length,
            prototype.trajectory.populated,
        );
    }
    for (prompt, control) in [
        ("left", &record.left_relations.real),
        ("right", &record.right_relations.real),
    ] {
        for candidate in &control.candidates {
            println!(
                "{prompt}_real_{}={:?}; exact_slot_match={:?}",
                String::from_utf8_lossy(&candidate.candidate_payload_bytes),
                candidate_minimum_cost(candidate).unwrap().suffix_shells,
                candidate.prototype_relations[0].exact_slot_match,
            );
        }
    }
    if let Some(expected) = LOCAL_CONTEXT_PLACEMENT_POLICY_KAPPA {
        assert_eq!(record.overlay_policy_kappa, expected);
    }
    if let Some(expected) = LOCAL_CONTEXT_PLACEMENT_OVERLAY_KAPPA {
        assert_eq!(record.overlay_kappa, expected);
    }
}

#[test]
fn local_context_placement_preflight_hard_stops_before_generation() {
    let bundle = frozen_bundle();
    let record = relation_census_record(&bundle);
    let relation_bytes = serde_json::to_vec(&record).unwrap();
    let relation_census_kappa = format!("blake3:{}", blake3::hash(&relation_bytes).to_hex());
    if let Some(expected) = LOCAL_CONTEXT_RELATION_CENSUS_KAPPA {
        assert_eq!(relation_census_kappa, expected);
    }
    assert_eq!(record.padding_identity_aliases, 0);

    // Selector semantics begin only after the raw relation census is hashed.
    // Equal exact minima abstain; canonical address order never rescues a tie.
    let mut tied = record.left_relations.real.clone();
    let tied_cost = candidate_minimum_cost(&tied.candidates[0]).unwrap();
    for relation in &mut tied.candidates[1].prototype_relations {
        relation.cost = tied_cost;
    }
    let tie = strict_minimum(&tied);
    assert!(tie.tie);
    assert!(!tie.unavailable);
    assert!(tie.winner_payload_bytes.is_none());

    let left_real = strict_minimum(&record.left_relations.real);
    let right_real = strict_minimum(&record.right_relations.real);
    let left_placement_permuted = strict_minimum(&record.left_relations.placement_permuted);
    let right_placement_permuted = strict_minimum(&record.right_relations.placement_permuted);
    let left_order_shuffled = strict_minimum(&record.left_relations.order_shuffled);
    let right_order_shuffled = strict_minimum(&record.right_relations.order_shuffled);
    let frozen_left_expected = LEFT_EXPECTED[1].to_vec();
    let frozen_right_expected = RIGHT_EXPECTED[1].to_vec();
    let intended_matches = |left: &StrictMinimumRecord, right: &StrictMinimumRecord| {
        usize::from(left.winner_payload_bytes.as_deref() == Some(frozen_left_expected.as_slice()))
            + usize::from(
                right.winner_payload_bytes.as_deref() == Some(frozen_right_expected.as_slice()),
            )
    };
    let real_intended_matches = intended_matches(&left_real, &right_real);
    let placement_permuted_intended_matches =
        intended_matches(&left_placement_permuted, &right_placement_permuted);
    let order_shuffled_intended_matches =
        intended_matches(&left_order_shuffled, &right_order_shuffled);

    assert_eq!(
        left_real.winner_payload_bytes.as_deref(),
        Some(b"run".as_slice())
    );
    assert_eq!(
        right_real.winner_payload_bytes.as_deref(),
        Some(b"runs".as_slice())
    );
    assert_eq!(
        left_placement_permuted.winner_payload_bytes.as_deref(),
        Some(b"runs".as_slice())
    );
    assert_eq!(
        right_placement_permuted.winner_payload_bytes.as_deref(),
        Some(b"run".as_slice())
    );
    assert_eq!(
        left_order_shuffled.winner_payload_bytes.as_deref(),
        Some(b"runs".as_slice())
    );
    assert_eq!(
        right_order_shuffled.winner_payload_bytes.as_deref(),
        Some(b"runs".as_slice())
    );
    let shells = |control: &PlacementControlRecord, payload: &[u8]| {
        placement_candidate(control, payload).prototype_relations[0]
            .cost
            .suffix_shells
    };
    assert_eq!(
        shells(&record.left_relations.real, b"run"),
        [
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Degrees120,
        ]
    );
    assert_eq!(
        shells(&record.left_relations.real, b"runs"),
        [
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Orthogonal,
            H4S3AngularShell::Degrees120,
        ]
    );
    assert_eq!(
        shells(&record.right_relations.real, b"run"),
        [
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Orthogonal,
            H4S3AngularShell::Degrees108,
        ]
    );
    assert_eq!(
        shells(&record.right_relations.real, b"runs"),
        [
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
            H4S3AngularShell::Coincident,
        ]
    );
    assert_eq!(
        shells(&record.left_relations.order_shuffled, b"run"),
        [
            H4S3AngularShell::Degrees144,
            H4S3AngularShell::Degrees144,
            H4S3AngularShell::Degrees60,
            H4S3AngularShell::Degrees120,
        ]
    );
    assert_eq!(
        shells(&record.left_relations.order_shuffled, b"runs"),
        [
            H4S3AngularShell::Degrees60,
            H4S3AngularShell::Degrees108,
            H4S3AngularShell::Orthogonal,
            H4S3AngularShell::Degrees120,
        ]
    );
    assert_eq!(
        shells(&record.right_relations.order_shuffled, b"run"),
        [
            H4S3AngularShell::Degrees144,
            H4S3AngularShell::Degrees144,
            H4S3AngularShell::Degrees120,
            H4S3AngularShell::Degrees108,
        ]
    );
    assert_eq!(
        shells(&record.right_relations.order_shuffled, b"runs"),
        [
            H4S3AngularShell::Degrees60,
            H4S3AngularShell::Degrees108,
            H4S3AngularShell::Orthogonal,
            H4S3AngularShell::Degrees60,
        ]
    );
    assert_eq!(
        placement_candidate(&record.left_relations.real, b"run").prototype_relations[0]
            .exact_slot_match,
        [true, true, true, false]
    );
    assert_eq!(
        placement_candidate(&record.left_relations.real, b"runs").prototype_relations[0]
            .exact_slot_match,
        [true, true, false, false]
    );
    assert_eq!(
        placement_candidate(&record.right_relations.real, b"run").prototype_relations[0]
            .exact_slot_match,
        [true, true, false, false]
    );
    assert_eq!(
        placement_candidate(&record.right_relations.real, b"runs").prototype_relations[0]
            .exact_slot_match,
        [true, true, true, true]
    );
    assert_eq!(real_intended_matches, 0);
    assert_eq!(placement_permuted_intended_matches, 2);
    assert_eq!(order_shuffled_intended_matches, 1);

    let outcome = LocalContextPreflightOutcomeRecord {
        schema: 2,
        fixture_kappa: FIXTURE_KAPPA.to_owned(),
        repaired_support_record_kappa: REPAIRED_SUPPORT_PREFLIGHT_RECORD_KAPPA.to_owned(),
        relation_census_kappa,
        frozen_left_expected,
        frozen_right_expected,
        left_real,
        right_real,
        left_placement_permuted,
        right_placement_permuted,
        left_order_shuffled,
        right_order_shuffled,
        real_intended_matches,
        placement_permuted_intended_matches,
        order_shuffled_intended_matches,
        strict_selection_ceiling_numerator: 0,
        strict_selection_ceiling_denominator: 2,
        tie_rule: "equal-exact-candidate-vectors-abstain".to_owned(),
        pass_local_context_placement_preflight: "UNAVAILABLE".to_owned(),
        decoded_generation: "NOT_RUN".to_owned(),
        failure: "REAL_PLACEMENT_INVERTED_AND_PERMUTED_CONTROL_OUTPERFORMED".to_owned(),
        terminal: REVISION_TERMINAL.to_owned(),
    };
    let outcome_bytes = serde_json::to_vec(&outcome).unwrap();
    let outcome_kappa = format!("blake3:{}", blake3::hash(&outcome_bytes).to_hex());
    println!("local_context_preflight_outcome_kappa={outcome_kappa}");
    println!("PASS_LOCAL_CONTEXT_PLACEMENT_PREFLIGHT=UNAVAILABLE");
    println!("strict_selection_ceiling=0/2");
    println!("decoded_generation=NOT_RUN");
    println!("terminal={REVISION_TERMINAL}");
    if let Some(expected) = LOCAL_CONTEXT_PREFLIGHT_OUTCOME_KAPPA {
        assert_eq!(outcome_kappa, expected);
    }
}

#[test]
fn failed_local_context_preflight_quarantines_historical_four_arm_generator() {
    assert!(LOCAL_CONTEXT_PREFLIGHT_OUTCOME_KAPPA.is_some());
    assert!(!local_context_placement_generation_authorized());
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
    // An explicit ignored-test invocation cannot bypass either the published
    // support gate or the later placement-preflight hard stop merely because
    // the historical four-arm record remains append-only below.
    let left_preflight = preflight_arm(&bundle, LEFT_PROMPT);
    let right_preflight = preflight_arm(&bundle, RIGHT_PROMPT);
    assert!(
        preflight_matches_frozen_contract(&left_preflight)
            && preflight_matches_frozen_contract(&right_preflight)
            && preflight_arms_have_equal_support_and_work(&left_preflight, &right_preflight),
        "NOT_RUN_SUPPORT_PREFLIGHT_HARD_STOP"
    );
    assert!(
        local_context_placement_generation_authorized(),
        "NOT_RUN_LOCAL_CONTEXT_PLACEMENT_PREFLIGHT_HARD_STOP"
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
