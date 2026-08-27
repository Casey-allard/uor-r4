//! A1.0 ordered-state and value-reachability probe for recursive attention.
//!
//! A1.0 intentionally stops before scoring when reusable state erases order.
//! A1R adds only a bounded candidate-relative falsifier after repairing that
//! representation; it does not qualify recursive attention. Both probes freeze
//! a registration-only vocabulary, construction partition, and three earlier-
//! order contrasts before compiling route rows or hierarchy summaries.
//! Candidate support comes from the existing bounded
//! [`GeometricAttentionArtifact`] child-manifest path; no target is injected.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::canonical_lexical_ingestion::{
    canonical_global_epoch, h4_leaf_state_for_address, validate_h4_binary_icosahedral_closure,
    AttentionLevelTrace, AttentionOrderedFoldTrace, CanonicalLexicalCodec, CanonicalLexicalError,
    CanonicalRouteArtifact, ConversationInput, H4BinaryIcosahedralClosure, H4RootCoordinate,
    OrderedH4FoldState, ParagraphInput, TurnInput,
};
use crate::prime_route_attention::{GeometricAddress, PrimeRouteError};
use crate::prime_route_geometric_attention::{
    AttentionControl, AttentionRowKey, AttentionRowSource, AttentionSourceCounts,
    CausalAttentionState, GeometricAttentionArtifact, GeometricAttentionError,
    GeometricAttentionTrace,
};
use crate::spiralcore_operator::{cl06_finite_composition_table, SpiralCoreOperatorError};

pub const A1_0_ORDERED_STATE_PROBE_SCHEMA: u32 = 1;
pub const A1_0_ORDERED_STATE_PROBE_DOMAIN: &str =
    "uor-r4.a1-ordered-state-value-reachability-probe/1";
pub const REDESIGN_ORDERED_ROUTE_SUMMARY: &str = "REDESIGN_ORDERED_ROUTE_SUMMARY";
pub const A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_SCHEMA: u32 = 1;
pub const A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_DOMAIN: &str =
    "uor-r4.a1r-associative-ordered-summary-probe/1";

pub const REDESIGN_ROUTE_PLACEMENT: &str = "REDESIGN_ROUTE_PLACEMENT";
pub const RETAIN_STATE_ONLY: &str = "RETAIN_STATE_ONLY";
pub const REVISE_A1R: &str = "REVISE_A1R";
pub const PROMOTE_TO_A1Q: &str = "PROMOTE_TO_A1Q";

const ISSUE_URL: &str = "https://github.com/UOR-Foundation/uor-r4/issues/952";
const S0_CONSUMER_CONTRACT_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434217921";
const A1_DECISION_CONTRACT_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434437267";
const A1_FROZEN_FIXTURE_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434478600";
const FROZEN_S0_ARTIFACT_KAPPA: &str =
    "blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd";
const A1R_ISSUE_URL: &str = "https://github.com/UOR-Foundation/uor-r4/issues/967";
const A1R_SUCCESSOR_URL: &str = "https://github.com/UOR-Foundation/uor-r4/issues/969";
const A1R_DECISION_CONTRACT_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/967#issuecomment-5434971151";
const FROZEN_A1_PARTITION_KAPPA: &str =
    "blake3:d008b82eda9b16b102cf4c7ffa4a47a40ad514b30f0763ed3f46c0ebae3e277b";
const FROZEN_A1_CONSTRUCTION_ARTIFACT_KAPPA: &str =
    "blake3:2b70588d654c8e8bb2d8ab063f41853d45a21487d742ff7567f93a42cfb9011b";
const FROZEN_A1_ATTENTION_MANIFEST_KAPPA: &str =
    "blake3:1c77c4103732964af6776f1dfcabc8b2a9191eea875a8ba205c36ebbf5618a99";
const FROZEN_A1_REPORT_KAPPA: &str =
    "blake3:23e07a17e897abb701c09354d35a3113a1b075ba1d60ee634067fa3ed3fd1904";

const CANDIDATE_CEILING: usize = 8;
const ROWS_PER_QUERY_CEILING: usize = 7;
const CONTROL_ARMS_PER_QUERY: usize = 8;
const A1R_REQUIRED_CONTROLS: [&str; 8] = [
    "full-ordered-hierarchy",
    "current-only",
    "existing-additive-summary",
    "factor-count",
    "deterministic-conjugation",
    "hierarchy-disabled",
    "exact-recall-only",
    "inverse-h4-intervention",
];
const GLOBAL_TOKEN: &str = "gg";
const REQUIRED_LEVELS: [&str; 7] = [
    "current",
    "previous",
    "last-two",
    "sentence",
    "paragraph",
    "conversation",
    "global",
];

// Equal-length two-byte units keep the boundary and position controls matched.
// These constants are the complete fixed registration population; no token is
// selected after inspecting a compiled address, sector, row, or candidate.
const REGISTERED_TOKENS: [&str; 10] = ["aa", "bb", "cc", "dd", "gg", "ll", "qq", "rr", "uu", "vv"];

const CONSTRUCTION_SENTENCES: [&[&str]; 7] = [
    &["uu", "ll"],
    &["vv", "rr"],
    &["aa"],
    &["bb"],
    &["cc"],
    &["dd"],
    &["qq"],
];

struct FixedContrast {
    id: &'static str,
    left_history: &'static [&'static str],
    right_history: &'static [&'static str],
    left_target: &'static str,
    right_target: &'static str,
}

const FIXED_CONTRASTS: [FixedContrast; 3] = [
    FixedContrast {
        id: "early-swap-aa-bb",
        left_history: &["aa", "bb", "dd", "cc", "qq"],
        right_history: &["bb", "aa", "dd", "cc", "qq"],
        left_target: "ll",
        right_target: "rr",
    },
    FixedContrast {
        id: "early-permutation-aa-dd-bb",
        left_history: &["aa", "dd", "bb", "cc", "qq"],
        right_history: &["dd", "aa", "bb", "cc", "qq"],
        left_target: "ll",
        right_target: "rr",
    },
    FixedContrast {
        id: "early-permutation-bb-dd-aa",
        left_history: &["bb", "dd", "aa", "cc", "qq"],
        right_history: &["dd", "bb", "aa", "cc", "qq"],
        left_target: "ll",
        right_target: "rr",
    },
];

const NON_DIGEST_ATTENTION_LEVEL_FIELDS: [&str; 46] = [
    "level",
    "identity_kind",
    "occurrence",
    "turn",
    "paragraph",
    "sentence",
    "ordinal_in_sentence",
    "lexical_unit_id",
    "prime",
    "address_index",
    "boundary_kind",
    "boundary_identity",
    "chain_identity",
    "direct_child_count",
    "observed_descendant_routes",
    "window_start",
    "window_end",
    "session_hypersphere_q30",
    "winding_turns",
    "projection_energy_q30",
    "shared_prime_factors",
    "cosine_resonance_q30",
    "accumulated_hopf_phase_q29",
    "zeta_phase_signature_q29",
    "s3_spin_q30",
    "s2_hopf_observation_q30",
    "fiber_q29",
    "torsion_q29",
    "radial_zphi",
    "bridge_mode",
    "active_chart",
    "selected_adapter",
    "chart_sin_q30",
    "chart_cos_q30",
    "chart_activation_q30",
    "chart_chirality",
    "chart_cosine_polarity",
    "quarter_turn_orientation",
    "phase_shift_q29",
    "torsion_shift_q29",
    "transported_fiber_q29",
    "transported_torsion_q29",
    "inverse_fiber_q29",
    "inverse_torsion_q29",
    "chart_inverse_exact",
    "paired_h4_e8_coordinate_sum",
];

const EXCLUDED_DIGEST_IDENTITY_FIELDS: [&str; 10] = [
    "identity_kappa",
    "exact_chain_kappa",
    "geometric_summary_kappa",
    "previous_identity_kappa",
    "ordered_child_kappa",
    "payload_cid",
    "address_kappa",
    "shared_class_kappa",
    "paired_h4_e8_coordinate_kappa",
    "transported_trajectory_kappa",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursiveGeometricAttentionError {
    CanonicalLexical(String),
    GeometricAttention(String),
    PrimeRoute(String),
    SpiralCore(String),
    Serialization(String),
    Addressing(String),
    InvalidProbe(String),
}

impl std::fmt::Display for RecursiveGeometricAttentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CanonicalLexical(reason) => {
                write!(formatter, "canonical lexical state: {reason}")
            }
            Self::GeometricAttention(reason) => {
                write!(formatter, "geometric candidate path: {reason}")
            }
            Self::PrimeRoute(reason) => write!(formatter, "prime-route identity: {reason}"),
            Self::SpiralCore(reason) => write!(formatter, "SpiralCore finite control: {reason}"),
            Self::Serialization(reason) => write!(formatter, "A1.0 serialization: {reason}"),
            Self::Addressing(reason) => write!(formatter, "A1.0 content address: {reason}"),
            Self::InvalidProbe(reason) => write!(formatter, "invalid A1.0 probe: {reason}"),
        }
    }
}

impl std::error::Error for RecursiveGeometricAttentionError {}

impl From<CanonicalLexicalError> for RecursiveGeometricAttentionError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::CanonicalLexical(error.to_string())
    }
}

impl From<GeometricAttentionError> for RecursiveGeometricAttentionError {
    fn from(error: GeometricAttentionError) -> Self {
        Self::GeometricAttention(error.to_string())
    }
}

impl From<PrimeRouteError> for RecursiveGeometricAttentionError {
    fn from(error: PrimeRouteError) -> Self {
        Self::PrimeRoute(error.to_string())
    }
}

impl From<SpiralCoreOperatorError> for RecursiveGeometricAttentionError {
    fn from(error: SpiralCoreOperatorError) -> Self {
        Self::SpiralCore(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10OrderedStateProbeReport {
    pub schema: u32,
    pub domain: String,
    pub report_kappa: String,
    pub body: A10OrderedStateProbeBody,
}

impl A10OrderedStateProbeReport {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RecursiveGeometricAttentionError> {
        if self.schema != A1_0_ORDERED_STATE_PROBE_SCHEMA
            || self.domain != A1_0_ORDERED_STATE_PROBE_DOMAIN
        {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(
                "report schema/domain is unsupported".to_owned(),
            ));
        }
        let expected = report_identity_kappa(&self.body)?;
        if expected != self.report_kappa {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(
                "report kappa does not reproduce".to_owned(),
            ));
        }
        canonical_json(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10OrderedStateProbeBody {
    pub probe_status: String,
    pub terminal_verdict: String,
    pub provenance: A10ProbeProvenance,
    pub fixed_partition: A10FrozenPartition,
    pub codec_registration: A10CodecRegistration,
    pub construction_artifact: A10ConstructionArtifact,
    pub exact_h4_closure: A10H4ClosureControl,
    pub spiralcore_control: A10SpiralCoreControl,
    pub contrasts: Vec<A10ContrastReport>,
    pub required_contrasts: usize,
    pub colliding_contrasts: usize,
    pub all_required_contrasts_collide: bool,
    pub serving_boundary: A10ServingBoundary,
    pub scorer_boundary: A10ScorerBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10ProbeProvenance {
    pub issue_url: String,
    pub s0_consumer_contract_url: String,
    pub decision_contract_url: String,
    pub frozen_fixture_url: String,
    pub frozen_s0_artifact_kappa: String,
    pub fixed_partition_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10FrozenPartition {
    pub declaration: String,
    pub registered_vocabulary: Vec<String>,
    pub construction_sentences: Vec<Vec<String>>,
    pub evaluation_contrasts: Vec<A10EvaluationContract>,
    pub frozen_before_codec_compile: bool,
    pub frozen_before_candidate_compile: bool,
    pub frozen_before_summary_compile: bool,
    pub targets_selected_from_observed_rows: bool,
    pub evaluation_histories_enter_construction_manifest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10EvaluationContract {
    pub contrast_id: String,
    pub left_history: Vec<String>,
    pub right_history: Vec<String>,
    pub left_target: String,
    pub right_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10CodecRegistration {
    pub compile_population: String,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub registered_units: usize,
    pub evaluation_target_independent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10ConstructionArtifact {
    pub canonical_route_manifest_kappa: String,
    pub embedded_spin_manifest_kappa: String,
    pub attention_manifest_kappa: String,
    pub maximum_candidates_per_row: u16,
    pub rows_per_query: usize,
    pub candidate_entries_per_query: usize,
    pub retained_candidate_ceiling: usize,
    pub child_manifest_addresses: usize,
    pub construction_only: bool,
    pub one_worker_child_manifest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10SpiralCoreControl {
    pub semantic_status: String,
    pub operator_kappa: String,
    pub composition_table_kappa: String,
    pub composition_table_kappa_reproduces: bool,
    pub unique_states: usize,
    pub composition_entries: usize,
    pub associativity_checks: usize,
    pub two_sided_inverses: usize,
    pub noncommuting_ordered_pairs: usize,
    pub identity_index: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10H4ClosureControl {
    pub semantic_status: String,
    pub h4_root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub multiplication_table_kappa_reproduces: bool,
    pub root_count: usize,
    pub product_count: usize,
    pub associativity_checks: usize,
    pub identity_index: u16,
    pub inverse_count: usize,
    pub unique_closure_exact: bool,
    pub identity_exact: bool,
    pub inverses_exact: bool,
    pub associativity_exact: bool,
    pub integer_only_no_rounding: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10ScorerBoundary {
    pub attention_scorer_implemented: bool,
    pub geometry_coefficients_tuned: bool,
    pub scorer_controls_exercised: bool,
    pub scorer_status: String,
    pub h4_status: String,
    pub digest_distance_used_as_geometry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10ServingBoundary {
    pub source_model_weights_opened: bool,
    pub teacher_forwards: u32,
    pub transformer_calls: u32,
    pub moe_calls: u32,
    pub learned_router_calls: u32,
    pub dense_intelligence_matrix_calls: u32,
    pub ollama_calls: u32,
    pub hosted_provider_calls: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10ContrastReport {
    pub contrast_id: String,
    pub invariants: A10MatchedHistoryInvariants,
    pub collision_census: A10CollisionCensus,
    pub left_candidate_path: A10CandidatePath,
    pub right_candidate_path: A10CandidatePath,
    pub natural_candidate_union_equal: bool,
    pub candidate_support_counts_equal: bool,
    pub both_competing_targets_in_each_union: bool,
    pub exact_direct_rows_miss_both_sides: bool,
    pub ordered_state_collides: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10MatchedHistoryInvariants {
    pub left_history: Vec<String>,
    pub right_history: Vec<String>,
    pub left_target: String,
    pub right_target: String,
    pub same_length: bool,
    pub same_multiset: bool,
    pub same_boundary_shape: bool,
    pub same_current: bool,
    pub same_previous: bool,
    pub same_last_two_suffix: bool,
    pub earlier_order_differs: bool,
    pub competing_targets_differ: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10CollisionCensus {
    pub included_non_digest_fields: Vec<String>,
    pub excluded_digest_identity_fields: Vec<String>,
    pub included_field_count: usize,
    pub excluded_field_count: usize,
    pub left_levels: Vec<A10AttentionLevelNonDigest>,
    pub right_levels: Vec<A10AttentionLevelNonDigest>,
    pub per_level_equal: Vec<A10LevelCollision>,
    pub all_required_levels_present: bool,
    pub all_non_digest_fields_collide: bool,
    pub digest_identities_differ: bool,
    pub digest_identity_used_for_verdict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10LevelCollision {
    pub level: String,
    pub all_non_digest_fields_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10AttentionLevelNonDigest {
    pub level: String,
    pub identity_kind: String,
    pub occurrence: Option<u32>,
    pub turn: Option<u16>,
    pub paragraph: Option<u16>,
    pub sentence: Option<u16>,
    pub ordinal_in_sentence: Option<u16>,
    pub lexical_unit_id: Option<u32>,
    pub prime: Option<u32>,
    pub address_index: Option<u16>,
    pub boundary_kind: Option<String>,
    pub boundary_identity: Option<String>,
    pub chain_identity: Option<String>,
    pub direct_child_count: u32,
    pub observed_descendant_routes: u32,
    pub window_start: u32,
    pub window_end: u32,
    pub session_hypersphere_q30: [i64; 4],
    pub winding_turns: i64,
    pub projection_energy_q30: u64,
    pub shared_prime_factors: Vec<A10PrimeFactor>,
    pub cosine_resonance_q30: [i64; 8],
    pub accumulated_hopf_phase_q29: i32,
    pub zeta_phase_signature_q29: [i32; 8],
    pub s3_spin_q30: [i32; 4],
    pub s2_hopf_observation_q30: [i32; 3],
    pub fiber_q29: i32,
    pub torsion_q29: i32,
    pub radial_zphi: [i64; 2],
    pub bridge_mode: String,
    pub active_chart: String,
    pub selected_adapter: String,
    pub chart_sin_q30: i32,
    pub chart_cos_q30: i32,
    pub chart_activation_q30: u32,
    pub chart_chirality: i8,
    pub chart_cosine_polarity: i8,
    pub quarter_turn_orientation: i8,
    pub phase_shift_q29: i32,
    pub torsion_shift_q29: i32,
    pub transported_fiber_q29: i32,
    pub transported_torsion_q29: i32,
    pub inverse_fiber_q29: i32,
    pub inverse_torsion_q29: i32,
    pub chart_inverse_exact: bool,
    pub paired_h4_e8_coordinate_sum: [i64; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10PrimeFactor {
    pub prime: u32,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10CandidatePath {
    pub intended_target_token: String,
    pub intended_target_address_kappa: String,
    pub rows: Vec<A10RowOrigin>,
    pub exclusions: Vec<A10LeakageExclusion>,
    pub candidate_entries_examined: usize,
    pub candidate_entry_ceiling: usize,
    pub unique_candidates_before_admission: usize,
    pub unique_candidates_after_admission: usize,
    pub retained_candidate_ceiling: usize,
    pub admission_truncated_union: bool,
    pub full_pre_admission_union_observed: bool,
    pub candidates: Vec<A10CandidateOrigin>,
    pub intended_target_pre_admission_reachable: Option<bool>,
    pub intended_target_post_admission_reachable: bool,
    pub intended_target_truncated_before_geometry: Option<bool>,
    pub exact_direct_rows_miss: bool,
    pub target_injected: bool,
    pub future_events_visible: bool,
    pub incremental_next_state: Option<A10IncrementalNextState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10RowOrigin {
    pub source: String,
    pub key_kind: String,
    pub key_identity: String,
    pub hit: bool,
    pub candidate_entries_examined: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10LeakageExclusion {
    pub scope: String,
    pub status: String,
    pub candidate_entries_contributed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct A10SourceCounts {
    pub last_one: u32,
    pub last_two: u32,
    pub ordered_sentence: u32,
    pub divisor: u32,
    pub adjacent_spin: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10CandidateOrigin {
    pub address_value: A10AddressValue,
    pub source_counts: A10SourceCounts,
    pub contributing_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10AddressValue {
    pub lexical_unit_id: u32,
    /// Stable index in the complete parent codec registry.
    pub registry_address_index: u16,
    /// Exact index in the frozen schema-2 child manifest address vector.
    pub child_manifest_address_index: u16,
    pub address_kappa: String,
    pub prime: u32,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
    pub s3_spin_q30: [i32; 4],
    pub hopf_observation_q30: [i32; 3],
    pub fiber_q29: i32,
    pub torsion_q29: i32,
    pub radial_zphi: [i64; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A10IncrementalNextState {
    pub target_address_kappa: String,
    pub target_payload_bytes: Vec<u8>,
    pub incremental_previous_address_kappa: Option<String>,
    pub rebuilt_previous_address_kappa: Option<String>,
    pub incremental_last_address_kappa: String,
    pub rebuilt_last_address_kappa: String,
    pub incremental_ordered_sentence_kappa: String,
    pub rebuilt_ordered_sentence_kappa: String,
    pub incremental_observed_routes: u32,
    pub rebuilt_observed_routes: u32,
    pub exact_reproduction: bool,
}

/// Execute only the predeclared A1.0 ordered-state and value-reachability gate.
///
/// The vocabulary, construction rows, histories, and targets are compile-time
/// constants. A negative ordered-state result returns a report with the exact
/// `REDESIGN_ORDERED_ROUTE_SUMMARY` verdict; it does not construct a scorer.
pub fn run_a1_0_ordered_state_probe(
) -> Result<A10OrderedStateProbeReport, RecursiveGeometricAttentionError> {
    // Materialize every partition and target before compiling the codec, any
    // child-manifest candidate row, or any evaluation hierarchy summary.
    let fixed_partition = frozen_partition();
    validate_frozen_partition(&fixed_partition)?;
    let provenance = A10ProbeProvenance {
        issue_url: ISSUE_URL.to_owned(),
        s0_consumer_contract_url: S0_CONSUMER_CONTRACT_URL.to_owned(),
        decision_contract_url: A1_DECISION_CONTRACT_URL.to_owned(),
        frozen_fixture_url: A1_FROZEN_FIXTURE_URL.to_owned(),
        frozen_s0_artifact_kappa: FROZEN_S0_ARTIFACT_KAPPA.to_owned(),
        fixed_partition_kappa: frozen_partition_kappa(&fixed_partition)?,
    };

    let registry_input = registration_input(&fixed_partition)?;
    let codec = CanonicalLexicalCodec::compile(&registry_input)?;
    let codec_registration = A10CodecRegistration {
        compile_population: "registration-only; not ingested as construction evidence".to_owned(),
        codec_kappa: codec.codec_kappa().to_owned(),
        vocabulary_kappa: codec.vocabulary_kappa().to_owned(),
        registered_units: fixed_partition.registered_vocabulary.len(),
        evaluation_target_independent: true,
    };

    let construction_input = construction_input(&fixed_partition)?;
    let construction_artifact = CanonicalRouteArtifact::ingest(&codec, &construction_input)?;
    let embedded_manifest = construction_artifact.embedded_spin_manifest()?;
    let attention =
        GeometricAttentionArtifact::compile_from_manifest_witnesses(&embedded_manifest)?;
    let compile_stats = attention.compile_stats();
    let lookup_bounds = attention.lookup_bounds();
    if usize::from(compile_stats.maximum_candidates_per_row) > CANDIDATE_CEILING
        || lookup_bounds.unique_candidates_after_ceiling > CANDIDATE_CEILING
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
            "candidate ceiling exceeds the predeclared maximum of {CANDIDATE_CEILING}"
        )));
    }
    let construction = A10ConstructionArtifact {
        canonical_route_manifest_kappa: construction_artifact.manifest_kappa().to_owned(),
        embedded_spin_manifest_kappa: construction_artifact
            .embedded_spin_manifest_kappa()
            .to_owned(),
        attention_manifest_kappa: attention.manifest_kappa().to_owned(),
        maximum_candidates_per_row: compile_stats.maximum_candidates_per_row,
        rows_per_query: lookup_bounds.rows_per_query,
        candidate_entries_per_query: lookup_bounds.candidate_entries_per_query,
        retained_candidate_ceiling: lookup_bounds.unique_candidates_after_ceiling,
        child_manifest_addresses: embedded_manifest.addresses.len(),
        construction_only: true,
        one_worker_child_manifest: true,
    };

    // This is an exact finite-table closure audit, not a semantic score or an
    // approximate H4 projection.
    let exact_h4_table = validate_h4_binary_icosahedral_closure()?;
    let exact_h4_closure = A10H4ClosureControl {
        semantic_status: "EXACT_CLOSURE_CONTROL_ONLY_NO_SEMANTIC_CLAIM".to_owned(),
        h4_root_table_kappa: exact_h4_table.h4_root_table_kappa.clone(),
        multiplication_table_kappa: exact_h4_table.multiplication_table_kappa.clone(),
        multiplication_table_kappa_reproduces: exact_h4_table
            .reproduce_multiplication_table_kappa()?
            == exact_h4_table.multiplication_table_kappa,
        root_count: exact_h4_table.root_count,
        product_count: exact_h4_table.product_count,
        associativity_checks: exact_h4_table.root_count.pow(3),
        identity_index: exact_h4_table.identity_index,
        inverse_count: exact_h4_table.inverse_indices.len(),
        unique_closure_exact: exact_h4_table.unique_closure_exact,
        identity_exact: exact_h4_table.identity_exact,
        inverses_exact: exact_h4_table.inverses_exact,
        associativity_exact: exact_h4_table.associativity_exact,
        integer_only_no_rounding: exact_h4_table.integer_only_no_rounding,
    };
    let spiralcore_table = cl06_finite_composition_table()?;
    let spiralcore_validation = spiralcore_table.validate()?;
    let reproduced_spiralcore_kappa = spiralcore_table.reproduce_kappa()?;
    let spiralcore_control = A10SpiralCoreControl {
        semantic_status: "OPTIONAL_CONTROL_PENDING_NO_SEMANTIC_CLAIM".to_owned(),
        operator_kappa: spiralcore_validation.operator_kappa,
        composition_table_kappa: spiralcore_table.composition_kappa().to_owned(),
        composition_table_kappa_reproduces: reproduced_spiralcore_kappa
            == spiralcore_table.composition_kappa(),
        unique_states: spiralcore_validation.unique_states,
        composition_entries: spiralcore_validation.composition_entries,
        associativity_checks: spiralcore_validation.associativity_checks,
        two_sided_inverses: spiralcore_validation.two_sided_inverses,
        noncommuting_ordered_pairs: spiralcore_validation.noncommuting_ordered_pairs,
        identity_index: spiralcore_validation.identity_index,
    };

    let mut contrasts = Vec::with_capacity(fixed_partition.evaluation_contrasts.len());
    for contract in &fixed_partition.evaluation_contrasts {
        contrasts.push(evaluate_contrast(
            &codec,
            &construction_artifact,
            &attention,
            &embedded_manifest.addresses,
            contract,
        )?);
    }
    let colliding_contrasts = contrasts
        .iter()
        .filter(|contrast| contrast.ordered_state_collides)
        .count();
    let all_required_contrasts_collide = !contrasts.is_empty()
        && contrasts.len() == fixed_partition.evaluation_contrasts.len()
        && colliding_contrasts == contrasts.len();
    let any_admission_truncation = contrasts.iter().any(|contrast| {
        [
            &contrast.left_candidate_path,
            &contrast.right_candidate_path,
        ]
        .into_iter()
        .any(|path| {
            path.admission_truncated_union
                || path.intended_target_truncated_before_geometry == Some(true)
        })
    });
    let any_candidate_absence = contrasts
        .iter()
        .any(|contrast| !contrast.both_competing_targets_in_each_union);
    let mechanics_invalid = contrasts.iter().any(|contrast| {
        !contrast.exact_direct_rows_miss_both_sides
            || !contrast.natural_candidate_union_equal
            || !contrast.candidate_support_counts_equal
            || [
                &contrast.left_candidate_path,
                &contrast.right_candidate_path,
            ]
            .into_iter()
            .any(|path| path.target_injected || path.future_events_visible)
    });
    if mechanics_invalid {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "candidate support, exact-row leakage, or causal visibility violated the frozen A1.0 contract"
                .to_owned(),
        ));
    }
    let terminal_verdict = if any_admission_truncation {
        "REDESIGN_ADMISSION_OR_DECLARED_CEILING"
    } else if any_candidate_absence {
        "REDESIGN_CANDIDATE_INDEXES"
    } else if all_required_contrasts_collide {
        REDESIGN_ORDERED_ROUTE_SUMMARY
    } else {
        "ORDER_OBSERVABLE_CONTINUE_A1_QUALIFICATION"
    };
    let body = A10OrderedStateProbeBody {
        probe_status: "EXERCISED_FIXED_A1_0_GATE".to_owned(),
        terminal_verdict: terminal_verdict.to_owned(),
        provenance,
        fixed_partition,
        codec_registration,
        construction_artifact: construction,
        exact_h4_closure,
        spiralcore_control,
        required_contrasts: contrasts.len(),
        colliding_contrasts,
        all_required_contrasts_collide,
        contrasts,
        serving_boundary: A10ServingBoundary {
            source_model_weights_opened: false,
            teacher_forwards: 0,
            transformer_calls: 0,
            moe_calls: 0,
            learned_router_calls: 0,
            dense_intelligence_matrix_calls: 0,
            ollama_calls: 0,
            hosted_provider_calls: 0,
        },
        scorer_boundary: A10ScorerBoundary {
            attention_scorer_implemented: false,
            geometry_coefficients_tuned: false,
            scorer_controls_exercised: false,
            scorer_status: if terminal_verdict == REDESIGN_ORDERED_ROUTE_SUMMARY {
                "NOT_IMPLEMENTED_PREDECLARED_ORDERED_STATE_STOP"
            } else if terminal_verdict.starts_with("REDESIGN_") {
                "NOT_IMPLEMENTED_PREDECLARED_A1_0_MECHANICS_STOP"
            } else {
                "NOT_IMPLEMENTED_A1_0_REACHABILITY_ONLY"
            }
            .to_owned(),
            h4_status: "EXACT_CLOSURE_CONTROL_ONLY_NO_SEMANTIC_CLAIM".to_owned(),
            digest_distance_used_as_geometry: false,
        },
    };
    let report_kappa = report_identity_kappa(&body)?;
    let report = A10OrderedStateProbeReport {
        schema: A1_0_ORDERED_STATE_PROBE_SCHEMA,
        domain: A1_0_ORDERED_STATE_PROBE_DOMAIN.to_owned(),
        report_kappa,
        body,
    };
    report.canonical_bytes()?;
    Ok(report)
}

/// Short CLI-facing alias for the fixed A1.0 gate.
pub fn run_a1_0_probe() -> Result<A10OrderedStateProbeReport, RecursiveGeometricAttentionError> {
    run_a1_0_ordered_state_probe()
}

/// Canonical A1R evidence envelope. This consumer-side report is versioned
/// independently from the frozen S0 and A1.0 artifacts it cites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RAssociativeOrderedSummaryProbeReport {
    pub schema: u32,
    pub domain: String,
    pub report_kappa: String,
    pub body: A1RAssociativeOrderedSummaryProbeBody,
}

impl A1RAssociativeOrderedSummaryProbeReport {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RecursiveGeometricAttentionError> {
        if self.schema != A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_SCHEMA
            || self.domain != A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_DOMAIN
        {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(
                "A1R report schema/domain is unsupported".to_owned(),
            ));
        }
        if a1r_report_identity_kappa(&self.body)? != self.report_kappa {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(
                "A1R report kappa does not reproduce".to_owned(),
            ));
        }
        canonical_json(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RAssociativeOrderedSummaryProbeBody {
    pub probe_status: String,
    pub terminal_verdict: String,
    pub successor_effect: String,
    pub provenance: A1RProvenance,
    pub work_contract: A1RWorkContract,
    pub fold_contract: A1RFoldContract,
    pub collision_census: A1RPermutationCollisionCensus,
    pub scope_contrasts: Vec<A1RScopeContrast>,
    pub global_order_fixture: A1RGlobalOrderFixture,
    pub fold_laws: A1RFoldLawEvidence,
    pub incremental_checks: Vec<A1RIncrementalCheck>,
    pub candidate_queries: Vec<A1RCandidateQuery>,
    pub support_pair_checks: Vec<A1RSupportPairCheck>,
    pub transition_readout_summary: A1RTransitionReadoutSummary,
    pub scoring_summary: A1RScoringSummary,
    pub support_invariants_exact: bool,
    pub excluded_digest_fields: Vec<String>,
    pub serving_boundary: A10ServingBoundary,
    pub claim_boundary: A1RClaimBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RProvenance {
    pub issue_url: String,
    pub decision_contract_url: String,
    pub successor_a1q_url: String,
    pub frozen_s0_artifact_kappa: String,
    pub frozen_partition_kappa: String,
    pub frozen_partition_kappa_reproduces: bool,
    pub frozen_construction_artifact_kappa: String,
    pub construction_artifact_kappa_reproduces: bool,
    pub frozen_attention_manifest_kappa: String,
    pub attention_manifest_kappa_reproduces: bool,
    pub frozen_a1_0_report_kappa: String,
    pub frozen_inputs_modified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RWorkContract {
    pub expected_contrasts: usize,
    pub exercised_contrasts: usize,
    pub expected_candidate_queries: usize,
    pub exercised_candidate_queries: usize,
    pub expected_candidate_payload_inversions: usize,
    pub exercised_candidate_payload_inversions: usize,
    pub exact_candidate_payload_inversions: usize,
    pub expected_incremental_checks: usize,
    pub exercised_incremental_checks: usize,
    pub exact_incremental_checks: usize,
    pub rows_per_query_ceiling: usize,
    pub expected_row_reads: usize,
    pub exercised_row_reads: usize,
    pub candidate_entry_ceiling_per_query: usize,
    pub candidate_ceiling: usize,
    pub maximum_admitted_candidates_observed: usize,
    pub control_arms_per_query: usize,
    pub expected_permutation_census: usize,
    pub exercised_permutation_census: usize,
    pub expected_associativity_checks: usize,
    pub exercised_associativity_checks: usize,
    pub external_corpus_population_scan_performed: bool,
    pub source_model_or_teacher_run_performed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RFoldContract {
    pub semantic_status: String,
    pub leaf_assignment: String,
    pub composition_order: String,
    pub opaque_index_distance_forbidden: bool,
    pub h4_root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub root_count: usize,
    pub identity: A1RStateWitness,
    pub construction_leaf_states: Vec<A1RNamedStateWitness>,
    pub symmetric_generators: Vec<A1RStateWitness>,
    pub cayley_metric_kappa: String,
    pub cayley_distances: Vec<A1RDistanceEntry>,
    pub all_states_reachable: bool,
    pub deterministic_conjugator: A1RStateWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RStateWitness {
    pub opaque_table_offset: u16,
    pub root_coordinate: H4RootCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RNamedStateWitness {
    pub token: String,
    pub state: A1RStateWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RDistanceEntry {
    pub state: A1RStateWitness,
    pub distance_to_identity: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RPermutationCollisionCensus {
    pub population: Vec<String>,
    pub expected_permutations: usize,
    pub examined_permutations: usize,
    pub unique_states: usize,
    pub collision_free: bool,
    pub largest_collision_bucket_size: usize,
    pub outcomes: Vec<A1RPermutationOutcome>,
    pub collision_buckets: Vec<A1RCollisionBucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RPermutationOutcome {
    pub ordered_tokens: Vec<String>,
    pub fold_state: A1RStateWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RCollisionBucket {
    pub fold_state: A1RStateWitness,
    pub ordered_token_sequences: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RScopeContrast {
    pub contrast_id: String,
    pub left_overlay_kappa: String,
    pub right_overlay_kappa: String,
    pub levels: Vec<A1RScopeLevelComparison>,
    pub scope_mask_exact: bool,
    pub legacy_non_digest_summary_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RScopeLevelComparison {
    pub level: String,
    pub expected_equal: bool,
    pub observed_equal: bool,
    pub left_observed_routes: u32,
    pub right_observed_routes: u32,
    pub left_state: A1RStateWitness,
    pub right_state: A1RStateWitness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RGlobalOrderFixture {
    pub declaration: String,
    pub left_global_snapshot_units: Vec<String>,
    pub right_global_snapshot_units: Vec<String>,
    pub left_global_epoch: String,
    pub right_global_epoch: String,
    pub global_epoch_is_derived_identity: bool,
    pub lower_scope_inputs_equal: bool,
    pub candidate_rows_equal: bool,
    pub candidate_support_equal: bool,
    pub candidate_work_budget_equal: bool,
    pub left_support_denominator_kappa: String,
    pub right_support_denominator_kappa: String,
    pub levels: Vec<A1RScopeLevelComparison>,
    pub only_global_state_differs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RFoldLawEvidence {
    pub exact_table_identity: bool,
    pub exact_table_inverses: bool,
    pub exact_table_associativity: bool,
    pub exact_table_closure: bool,
    pub identity_checks: usize,
    pub inverse_checks: usize,
    pub associativity_checks: usize,
    pub grouping_checks: Vec<A1RGroupingCheck>,
    pub recursive_hierarchy_fixture: A1RRecursiveHierarchyFixture,
    pub all_grouping_checks_exact: bool,
    pub all_laws_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RGroupingCheck {
    pub contrast_id: String,
    pub side: String,
    pub flat_state: A1RStateWitness,
    pub regrouped_state: A1RStateWitness,
    pub current_matches_leaf: bool,
    pub previous_matches_leaf: bool,
    pub last_two_matches_direct_fold: bool,
    pub sentence_matches_flat_fold: bool,
    pub paragraph_matches_sentence: bool,
    pub conversation_matches_paragraph: bool,
    pub regrouping_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RRecursiveHierarchyFixture {
    pub declaration: String,
    pub turn_count: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub lexical_unit_count: usize,
    pub flat_state: A1RStateWitness,
    pub sentence_regrouped_state: A1RStateWitness,
    pub paragraph_regrouped_state: A1RStateWitness,
    pub conversation_state: A1RStateWitness,
    pub flat_equals_sentence_regrouped: bool,
    pub flat_equals_paragraph_regrouped: bool,
    pub flat_equals_recursive_conversation: bool,
    pub all_regroupings_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RIncrementalCheck {
    pub contrast_id: String,
    pub side: String,
    pub intended_target: String,
    pub prefix_state: A1RStateWitness,
    pub incremental_state: A1RStateWitness,
    pub rebuilt_state: A1RStateWitness,
    pub incremental_observed_routes: u32,
    pub rebuilt_observed_routes: u32,
    pub prefix_clone_unchanged: bool,
    pub exact_reproduction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RCandidateQuery {
    pub contrast_id: String,
    pub side: String,
    pub intended_target: String,
    pub history: Vec<String>,
    pub candidate_entries_examined: usize,
    pub candidate_entry_ceiling: usize,
    pub unique_candidates_before_admission: usize,
    pub unique_candidates_after_admission: usize,
    pub retained_candidate_ceiling: usize,
    pub full_pre_admission_union_observed: bool,
    pub anchored_candidates_after_admission: usize,
    pub required_anchored_candidates: usize,
    pub admission_truncated_union: bool,
    pub exact_direct_rows_miss: bool,
    pub divisor_rows_miss: bool,
    pub adjacent_spin_only_support: bool,
    pub target_injected: bool,
    pub future_events_visible: bool,
    pub support_denominator_kappa: String,
    pub rows: Vec<A10RowOrigin>,
    pub candidate_support: Vec<A1RCandidateSupport>,
    pub exact_candidate_payload_inversions: usize,
    pub excluded_candidates: Vec<A1RExcludedCandidate>,
    pub controls: Vec<A1RControlResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RCandidateSupport {
    pub token: String,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
    pub exact_payload_inversion: bool,
    pub source_counts: A10SourceCounts,
    pub contributing_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RSupportPairCheck {
    pub contrast_id: String,
    pub left_support_denominator_kappa: String,
    pub right_support_denominator_kappa: String,
    pub natural_candidate_union_equal: bool,
    pub candidate_source_counts_equal: bool,
    pub candidate_origins_equal: bool,
    pub row_source_outcomes_equal: bool,
    pub row_and_candidate_budgets_equal: bool,
    pub both_competing_targets_in_each_union: bool,
    pub exact_direct_rows_miss_both_sides: bool,
    pub legacy_additive_summary_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RExcludedCandidate {
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RControlResult {
    pub control: String,
    pub status: String,
    pub exercised: bool,
    pub energy_kind: String,
    pub hierarchy_state: Option<A1RStateWitness>,
    pub legacy_additive_state: Option<A10AttentionLevelNonDigest>,
    pub candidates: Vec<A1RCandidateInteraction>,
    pub minimum_energy: Option<u32>,
    pub minimum_energy_candidates: Vec<String>,
    pub canonical_address_tiebreak_rule: String,
    pub canonical_address_tiebreak_token: Option<String>,
    pub selected_token: Option<String>,
    pub tie: bool,
    pub abstained: bool,
    pub intended_target_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RCandidateInteraction {
    pub token: String,
    pub address_kappa: String,
    pub payload_cid: String,
    pub source_counts: A10SourceCounts,
    pub candidate_state: A1RStateWitness,
    pub predecessor_token: String,
    pub predecessor_state: A1RStateWitness,
    pub interaction_state: Option<A1RStateWitness>,
    pub predecessor_interaction_state: Option<A1RStateWitness>,
    pub relative_state: Option<A1RStateWitness>,
    pub energy: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RScoringSummary {
    pub status: String,
    pub required_queries: usize,
    pub exercised_queries: usize,
    pub full_strict_correct: usize,
    pub full_ties: usize,
    pub control_strict_correct: Vec<A1RControlAggregate>,
    pub all_required_controls_present: bool,
    pub all_required_controls_exercised: bool,
    pub any_exercised_control_not_weaker: bool,
    pub every_control_weaker: bool,
    pub positive_contract_satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RTransitionReadoutSummary {
    pub interaction: String,
    pub scalar_readout: String,
    pub exercised_queries: usize,
    pub queries_with_distinct_candidate_relative_states: usize,
    pub queries_with_distinct_relative_states_but_equal_energy: usize,
    pub paired_same_candidate_comparisons: usize,
    pub paired_same_candidate_relative_state_differences: usize,
    pub scalar_readout_degeneracy_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RControlAggregate {
    pub control: String,
    pub exercised_queries: usize,
    pub strict_correct: usize,
    pub ties: usize,
    pub abstentions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct A1RClaimBoundary {
    pub representation_repair_only: bool,
    pub full_recursive_attention_qualified: bool,
    pub generation_unblocked: bool,
    pub correctness_established: bool,
    pub reasoning_established: bool,
    pub digest_distance_used_as_geometry: bool,
    pub all_identity_and_provenance_bits_excluded_from_geometry: bool,
    pub candidate_support_or_admission_modified: bool,
}

/// Execute the frozen #967 A1R representation and candidate-relative probe.
///
/// This function never changes the frozen #961/#952 artifact bytes, candidate
/// indexes, or admission rules. A positive report can activate A1Q/#969 only;
/// it cannot qualify attention or unblock generation.
pub fn run_a1r_associative_ordered_summary_probe(
) -> Result<A1RAssociativeOrderedSummaryProbeReport, RecursiveGeometricAttentionError> {
    let fixed_partition = frozen_partition();
    validate_frozen_partition(&fixed_partition)?;
    let partition_kappa = frozen_partition_kappa(&fixed_partition)?;
    let registry_input = registration_input(&fixed_partition)?;
    let codec = CanonicalLexicalCodec::compile(&registry_input)?;
    let construction_input = construction_input(&fixed_partition)?;
    let construction_artifact = CanonicalRouteArtifact::ingest(&codec, &construction_input)?;
    let embedded_manifest = construction_artifact.embedded_spin_manifest()?;
    let attention =
        GeometricAttentionArtifact::compile_from_manifest_witnesses(&embedded_manifest)?;
    let table = validate_h4_binary_icosahedral_closure()?;

    let frozen_partition_reproduces = partition_kappa == FROZEN_A1_PARTITION_KAPPA;
    let frozen_construction_reproduces =
        construction_artifact.manifest_kappa() == FROZEN_A1_CONSTRUCTION_ARTIFACT_KAPPA;
    let frozen_attention_reproduces =
        attention.manifest_kappa() == FROZEN_A1_ATTENTION_MANIFEST_KAPPA;
    if !frozen_partition_reproduces
        || !frozen_construction_reproduces
        || !frozen_attention_reproduces
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "the frozen #952 partition or construction identity drifted before A1R".to_owned(),
        ));
    }

    let predecessors = a1r_candidate_predecessors(&fixed_partition)?;
    let metric = a1r_cayley_metric(
        &codec,
        &construction_artifact,
        &embedded_manifest.addresses,
        &table,
    )?;
    let permutation_collision_census =
        a1r_permutation_collision_census(&codec, &construction_artifact, &table)?;

    let mut scope_contrasts = Vec::new();
    let mut grouping_checks = Vec::new();
    let mut incremental_checks = Vec::new();
    let mut prepared_candidate_queries = Vec::new();
    let mut support_pair_checks = Vec::new();
    let mut paired_support_exact = true;
    for contract in &fixed_partition.evaluation_contrasts {
        let left_artifact = CanonicalRouteArtifact::ingest(
            &codec,
            &evaluation_input(&contract.contrast_id, &contract.left_history)?,
        )?;
        let right_artifact = CanonicalRouteArtifact::ingest(
            &codec,
            &evaluation_input(&contract.contrast_id, &contract.right_history)?,
        )?;
        let left_trace = left_artifact.attention_consumer_trace_with_ordered_h4(&table)?;
        let right_trace = right_artifact.attention_consumer_trace_with_ordered_h4(&table)?;
        let legacy_left = left_artifact.attention_consumer_trace()?;
        let legacy_right = right_artifact.attention_consumer_trace()?;
        let legacy_collision = collision_census(
            &legacy_left.ordered_levels,
            &legacy_right.ordered_levels,
            legacy_left.artifact_manifest_kappa != legacy_right.artifact_manifest_kappa,
        )?;
        scope_contrasts.push(a1r_scope_contrast(
            contract,
            &left_trace,
            &right_trace,
            legacy_collision.all_non_digest_fields_collide,
            &table,
        )?);

        let left_history =
            a1r_history_addresses(&codec, &construction_artifact, &contract.left_history)?;
        let right_history =
            a1r_history_addresses(&codec, &construction_artifact, &contract.right_history)?;
        grouping_checks.push(a1r_grouping_check(
            &contract.contrast_id,
            "left",
            &left_history,
            &left_trace,
            &table,
        )?);
        grouping_checks.push(a1r_grouping_check(
            &contract.contrast_id,
            "right",
            &right_history,
            &right_trace,
            &table,
        )?);

        let left_path = candidate_path(
            &codec,
            &construction_artifact,
            &attention,
            &embedded_manifest.addresses,
            &contract.left_history,
            &contract.left_target,
        )?;
        let right_path = candidate_path(
            &codec,
            &construction_artifact,
            &attention,
            &embedded_manifest.addresses,
            &contract.right_history,
            &contract.right_target,
        )?;
        paired_support_exact &=
            candidate_support_signature(&left_path) == candidate_support_signature(&right_path);
        let both_competing_targets_in_each_union = [
            (&left_path, &contract.left_target),
            (&left_path, &contract.right_target),
            (&right_path, &contract.left_target),
            (&right_path, &contract.right_target),
        ]
        .into_iter()
        .all(|(path, target)| a1r_path_contains_token(path, target));
        support_pair_checks.push(A1RSupportPairCheck {
            contrast_id: contract.contrast_id.clone(),
            left_support_denominator_kappa: a1r_support_denominator_kappa(&left_path)?,
            right_support_denominator_kappa: a1r_support_denominator_kappa(&right_path)?,
            natural_candidate_union_equal: candidate_support_signature(&left_path)
                .iter()
                .map(|(address, _)| address)
                .eq(candidate_support_signature(&right_path)
                    .iter()
                    .map(|(address, _)| address)),
            candidate_source_counts_equal: candidate_support_signature(&left_path)
                == candidate_support_signature(&right_path),
            candidate_origins_equal: a1r_candidate_origin_signature(&left_path)
                == a1r_candidate_origin_signature(&right_path),
            row_source_outcomes_equal: a1r_row_source_outcome_signature(&left_path)
                == a1r_row_source_outcome_signature(&right_path),
            row_and_candidate_budgets_equal: a1r_path_budget_signature(&left_path)
                == a1r_path_budget_signature(&right_path),
            both_competing_targets_in_each_union,
            exact_direct_rows_miss_both_sides: left_path.exact_direct_rows_miss
                && right_path.exact_direct_rows_miss,
            legacy_additive_summary_equal: legacy_collision.all_non_digest_fields_collide,
        });
        incremental_checks.push(a1r_incremental_check(
            &contract.contrast_id,
            "left",
            &contract.left_target,
            &left_history,
            &attention,
            &codec,
            &construction_artifact,
            &table,
        )?);
        incremental_checks.push(a1r_incremental_check(
            &contract.contrast_id,
            "right",
            &contract.right_target,
            &right_history,
            &attention,
            &codec,
            &construction_artifact,
            &table,
        )?);
        prepared_candidate_queries.push(A1RPreparedCandidateQuery {
            contract: contract.clone(),
            side: "left",
            history: contract.left_history.clone(),
            path: left_path,
            trace: left_trace,
        });
        prepared_candidate_queries.push(A1RPreparedCandidateQuery {
            contract: contract.clone(),
            side: "right",
            history: contract.right_history.clone(),
            path: right_path,
            trace: right_trace,
        });
    }

    let global_order_fixture = a1r_global_order_fixture(
        &codec,
        &construction_artifact,
        &attention,
        &embedded_manifest.addresses,
        &table,
    )?;
    let recursive_hierarchy_fixture =
        a1r_recursive_hierarchy_fixture(&codec, &construction_artifact, &table)?;
    let all_grouping_checks_exact = grouping_checks.iter().all(|check| {
        check.current_matches_leaf
            && check.previous_matches_leaf
            && check.last_two_matches_direct_fold
            && check.sentence_matches_flat_fold
            && check.paragraph_matches_sentence
            && check.conversation_matches_paragraph
            && check.regrouping_exact
    }) && recursive_hierarchy_fixture.all_regroupings_exact;
    let fold_laws = A1RFoldLawEvidence {
        exact_table_identity: table.identity_exact,
        exact_table_inverses: table.inverses_exact,
        exact_table_associativity: table.associativity_exact,
        exact_table_closure: table.unique_closure_exact,
        identity_checks: table.root_count.saturating_mul(2),
        inverse_checks: table.root_count.saturating_mul(2),
        associativity_checks: table.root_count.pow(3),
        grouping_checks,
        recursive_hierarchy_fixture,
        all_grouping_checks_exact,
        all_laws_exact: table.identity_exact
            && table.inverses_exact
            && table.associativity_exact
            && table.unique_closure_exact
            && all_grouping_checks_exact,
    };
    let frozen_equal_scope_exact = scope_contrasts.iter().all(|contrast| {
        contrast.levels.iter().all(|level| {
            !matches!(
                level.level.as_str(),
                "current" | "previous" | "last-two" | "global"
            ) || level.observed_equal
        })
    });
    let required_order_scopes_distinct = scope_contrasts.iter().all(|contrast| {
        contrast.levels.iter().all(|level| {
            !matches!(
                level.level.as_str(),
                "sentence" | "paragraph" | "conversation"
            ) || !level.observed_equal
        })
    });
    let incremental_exact = incremental_checks
        .iter()
        .all(|check| check.exact_reproduction && check.prefix_clone_unchanged);
    let support_invariants_exact = paired_support_exact
        && support_pair_checks.iter().all(|check| {
            check.left_support_denominator_kappa == check.right_support_denominator_kappa
                && check.natural_candidate_union_equal
                && check.candidate_source_counts_equal
                && check.candidate_origins_equal
                && check.row_source_outcomes_equal
                && check.row_and_candidate_budgets_equal
                && check.both_competing_targets_in_each_union
                && check.exact_direct_rows_miss_both_sides
                && check.legacy_additive_summary_equal
        })
        && prepared_candidate_queries
            .iter()
            .all(|prepared| a1r_candidate_path_contract_exact(&prepared.path));
    let census_complete = permutation_collision_census.examined_permutations
        == permutation_collision_census.expected_permutations;
    let structural_gate_exact = fold_laws.all_laws_exact
        && incremental_exact
        && frozen_equal_scope_exact
        && required_order_scopes_distinct
        && global_order_fixture.only_global_state_differs
        && global_order_fixture.candidate_rows_equal
        && global_order_fixture.candidate_support_equal
        && global_order_fixture.candidate_work_budget_equal
        && census_complete;
    let score_controls =
        structural_gate_exact && support_invariants_exact && metric.all_states_reachable;
    let candidate_queries = prepared_candidate_queries
        .iter()
        .map(|prepared| {
            a1r_candidate_query(
                &prepared.contract,
                prepared.side,
                &prepared.history,
                &prepared.path,
                &prepared.trace,
                &codec,
                &construction_artifact,
                &predecessors,
                &metric,
                &table,
                score_controls,
            )
        })
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let support_invariants_exact = support_invariants_exact
        && candidate_queries.iter().all(|query| {
            query.exact_candidate_payload_inversions == query.candidate_support.len()
                && query.anchored_candidates_after_admission == query.required_anchored_candidates
        });
    let scoring_summary = a1r_scoring_summary(&candidate_queries, score_controls);
    let transition_readout_summary = a1r_transition_readout_summary(&candidate_queries);
    let mechanics_exact =
        structural_gate_exact && metric.all_states_reachable && support_invariants_exact;
    let terminal_verdict = if !fold_laws.all_laws_exact
        || !incremental_exact
        || !frozen_equal_scope_exact
        || !global_order_fixture.only_global_state_differs
        || !census_complete
    {
        REVISE_A1R
    } else if !required_order_scopes_distinct {
        REDESIGN_ORDERED_ROUTE_SUMMARY
    } else if !support_invariants_exact || !metric.all_states_reachable {
        REVISE_A1R
    } else if scoring_summary.positive_contract_satisfied {
        PROMOTE_TO_A1Q
    } else if scoring_summary.full_ties == scoring_summary.exercised_queries
        || scoring_summary.any_exercised_control_not_weaker
    {
        RETAIN_STATE_ONLY
    } else {
        REDESIGN_ROUTE_PLACEMENT
    };

    let fold_contract = A1RFoldContract {
        semantic_status: "EXACT_FINITE_STATE_REPRESENTATION_NO_ATTENTION_CLAIM".to_owned(),
        leaf_assignment: "S(route) = H4[prime mod 120] in the canonical root order".to_owned(),
        composition_order: "left-to-right S(A || B) = S(A) * S(B)".to_owned(),
        opaque_index_distance_forbidden: true,
        h4_root_table_kappa: table.h4_root_table_kappa.clone(),
        multiplication_table_kappa: table.multiplication_table_kappa.clone(),
        root_count: table.root_count,
        identity: a1r_state_witness(OrderedH4FoldState::identity(&table)?, &table)?,
        construction_leaf_states: metric.construction_leaf_states.clone(),
        symmetric_generators: metric.generators.clone(),
        cayley_metric_kappa: metric.metric_kappa.clone(),
        cayley_distances: metric.distances.clone(),
        all_states_reachable: metric.all_states_reachable,
        deterministic_conjugator: a1r_state_witness(metric.conjugator, &table)?,
    };
    let body = A1RAssociativeOrderedSummaryProbeBody {
        probe_status: if mechanics_exact && scoring_summary.all_required_controls_exercised {
            "EXERCISED_FIXED_A1R_GATE_ALL_CONTROLS"
        } else if mechanics_exact && scoring_summary.all_required_controls_present {
            "EXERCISED_FIXED_A1R_NEGATIVE_WITH_UNAVAILABLE_MATCHED_CONTROL"
        } else {
            "EXERCISED_A1R_WITH_LOCALIZED_MECHANICS_DEFECT"
        }
        .to_owned(),
        terminal_verdict: terminal_verdict.to_owned(),
        successor_effect: if terminal_verdict == PROMOTE_TO_A1Q {
            "A1Q_969_MAY_BECOME_ELIGIBLE_GENERATION_953_REMAINS_BLOCKED"
        } else {
            "A1Q_969_MUST_REMAIN_BLOCKED_BY_AN_EXACT_A1R_SUCCESSOR"
        }
        .to_owned(),
        provenance: A1RProvenance {
            issue_url: A1R_ISSUE_URL.to_owned(),
            decision_contract_url: A1R_DECISION_CONTRACT_URL.to_owned(),
            successor_a1q_url: A1R_SUCCESSOR_URL.to_owned(),
            frozen_s0_artifact_kappa: FROZEN_S0_ARTIFACT_KAPPA.to_owned(),
            frozen_partition_kappa: partition_kappa,
            frozen_partition_kappa_reproduces: frozen_partition_reproduces,
            frozen_construction_artifact_kappa: construction_artifact.manifest_kappa().to_owned(),
            construction_artifact_kappa_reproduces: frozen_construction_reproduces,
            frozen_attention_manifest_kappa: attention.manifest_kappa().to_owned(),
            attention_manifest_kappa_reproduces: frozen_attention_reproduces,
            frozen_a1_0_report_kappa: FROZEN_A1_REPORT_KAPPA.to_owned(),
            frozen_inputs_modified: false,
        },
        work_contract: A1RWorkContract {
            expected_contrasts: FIXED_CONTRASTS.len(),
            exercised_contrasts: scope_contrasts.len(),
            expected_candidate_queries: FIXED_CONTRASTS.len().saturating_mul(2),
            exercised_candidate_queries: candidate_queries.len(),
            expected_candidate_payload_inversions: FIXED_CONTRASTS.len().saturating_mul(4),
            exercised_candidate_payload_inversions: candidate_queries
                .iter()
                .map(|query| query.candidate_support.len())
                .sum(),
            exact_candidate_payload_inversions: candidate_queries
                .iter()
                .map(|query| query.exact_candidate_payload_inversions)
                .sum(),
            expected_incremental_checks: FIXED_CONTRASTS.len().saturating_mul(2),
            exercised_incremental_checks: incremental_checks.len(),
            exact_incremental_checks: incremental_checks
                .iter()
                .filter(|check| check.exact_reproduction && check.prefix_clone_unchanged)
                .count(),
            rows_per_query_ceiling: ROWS_PER_QUERY_CEILING,
            expected_row_reads: FIXED_CONTRASTS
                .len()
                .saturating_mul(2)
                .saturating_mul(ROWS_PER_QUERY_CEILING),
            exercised_row_reads: candidate_queries.iter().map(|query| query.rows.len()).sum(),
            candidate_entry_ceiling_per_query: ROWS_PER_QUERY_CEILING
                .saturating_mul(CANDIDATE_CEILING),
            candidate_ceiling: CANDIDATE_CEILING,
            maximum_admitted_candidates_observed: candidate_queries
                .iter()
                .map(|query| query.unique_candidates_after_admission)
                .max()
                .unwrap_or(0),
            control_arms_per_query: CONTROL_ARMS_PER_QUERY,
            expected_permutation_census: 120,
            exercised_permutation_census: permutation_collision_census.examined_permutations,
            expected_associativity_checks: table.root_count.pow(3),
            exercised_associativity_checks: table.root_count.pow(3),
            external_corpus_population_scan_performed: false,
            source_model_or_teacher_run_performed: false,
        },
        fold_contract,
        collision_census: permutation_collision_census,
        scope_contrasts,
        global_order_fixture,
        fold_laws,
        incremental_checks,
        candidate_queries,
        support_pair_checks,
        transition_readout_summary,
        scoring_summary,
        support_invariants_exact,
        excluded_digest_fields: EXCLUDED_DIGEST_IDENTITY_FIELDS
            .into_iter()
            .map(str::to_owned)
            .collect(),
        serving_boundary: A10ServingBoundary {
            source_model_weights_opened: false,
            teacher_forwards: 0,
            transformer_calls: 0,
            moe_calls: 0,
            learned_router_calls: 0,
            dense_intelligence_matrix_calls: 0,
            ollama_calls: 0,
            hosted_provider_calls: 0,
        },
        claim_boundary: A1RClaimBoundary {
            representation_repair_only: true,
            full_recursive_attention_qualified: false,
            generation_unblocked: false,
            correctness_established: false,
            reasoning_established: false,
            digest_distance_used_as_geometry: false,
            all_identity_and_provenance_bits_excluded_from_geometry: true,
            candidate_support_or_admission_modified: false,
        },
    };
    let report_kappa = a1r_report_identity_kappa(&body)?;
    let report = A1RAssociativeOrderedSummaryProbeReport {
        schema: A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_SCHEMA,
        domain: A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_DOMAIN.to_owned(),
        report_kappa,
        body,
    };
    report.canonical_bytes()?;
    Ok(report)
}

#[derive(Debug, Clone)]
struct A1RPreparedCandidateQuery {
    contract: A10EvaluationContract,
    side: &'static str,
    history: Vec<String>,
    path: A10CandidatePath,
    trace: AttentionOrderedFoldTrace,
}

#[derive(Debug, Clone)]
struct A1RCayleyMetric {
    construction_leaf_states: Vec<A1RNamedStateWitness>,
    generators: Vec<A1RStateWitness>,
    distances: Vec<A1RDistanceEntry>,
    distance_by_index: Vec<Option<u16>>,
    all_states_reachable: bool,
    metric_kappa: String,
    conjugator: OrderedH4FoldState,
}

#[derive(Serialize)]
struct A1RCayleyMetricIdentityWire<'a> {
    schema: u32,
    domain: &'static str,
    h4_root_table_kappa: &'a str,
    multiplication_table_kappa: &'a str,
    generators: &'a [A1RStateWitness],
    distances: &'a [A1RDistanceEntry],
}

fn a1r_cayley_metric(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    child_manifest_addresses: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RCayleyMetric, RecursiveGeometricAttentionError> {
    let child_address_set = child_manifest_addresses.iter().collect::<BTreeSet<_>>();
    let mut construction_leaf_states = Vec::new();
    let mut generator_offsets = BTreeSet::new();
    for token in REGISTERED_TOKENS {
        let address = lexical_address(codec, construction_artifact, token)?;
        if !child_address_set.contains(&address) {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                "construction token {token:?} is absent from the frozen child manifest"
            )));
        }
        let state = h4_leaf_state_for_address(&address, table)?;
        construction_leaf_states.push(A1RNamedStateWitness {
            token: token.to_owned(),
            state: a1r_state_witness(state, table)?,
        });
        if state.table_index().table_offset() != table.identity_index {
            generator_offsets.insert(state.table_index().table_offset());
            generator_offsets.insert(state.inverse(table)?.table_index().table_offset());
        }
    }
    let generator_states = generator_offsets
        .iter()
        .map(|offset| a1r_state_from_offset(*offset, table))
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let generators = generator_states
        .iter()
        .copied()
        .map(|state| a1r_state_witness(state, table))
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    if generator_states.is_empty() {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "construction produced no nonidentity Cayley generator".to_owned(),
        ));
    }

    let mut distance_by_index: Vec<Option<u16>> = vec![None; table.root_count];
    let identity_offset = usize::from(table.identity_index);
    distance_by_index[identity_offset] = Some(0);
    let mut queue = VecDeque::from([table.identity_index]);
    while let Some(current_offset) = queue.pop_front() {
        let current_distance = distance_by_index[usize::from(current_offset)].ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(
                "Cayley traversal dequeued an unvisited state".to_owned(),
            )
        })?;
        for generator in &generator_states {
            let next = table
                .product_index(current_offset, generator.table_index().table_offset())
                .ok_or_else(|| {
                    RecursiveGeometricAttentionError::InvalidProbe(
                        "Cayley traversal addressed outside the H4 table".to_owned(),
                    )
                })?;
            let entry = &mut distance_by_index[usize::from(next)];
            if entry.is_none() {
                *entry = Some(current_distance.checked_add(1).ok_or_else(|| {
                    RecursiveGeometricAttentionError::InvalidProbe(
                        "Cayley distance overflowed u16".to_owned(),
                    )
                })?);
                queue.push_back(next);
            }
        }
    }
    let mut distances = Vec::with_capacity(table.root_count);
    for offset in 0..table.root_count {
        let offset = u16::try_from(offset).map_err(|_| {
            RecursiveGeometricAttentionError::InvalidProbe(
                "H4 root count exceeds the opaque u16 table key".to_owned(),
            )
        })?;
        distances.push(A1RDistanceEntry {
            state: a1r_state_witness(a1r_state_from_offset(offset, table)?, table)?,
            distance_to_identity: distance_by_index[usize::from(offset)],
        });
    }
    let conjugator = (0..table.root_count)
        .filter_map(|offset| u16::try_from(offset).ok())
        .map(|offset| a1r_state_from_offset(offset, table))
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?
        .into_iter()
        .find(|candidate| {
            (0..table.root_count).any(|offset| {
                let Ok(offset) = u16::try_from(offset) else {
                    return false;
                };
                table.product_index(candidate.table_index().table_offset(), offset)
                    != table.product_index(offset, candidate.table_index().table_offset())
            })
        })
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(
                "the canonical H4 root order has no noncentral conjugator".to_owned(),
            )
        })?;
    let metric_kappa = canonical_kappa(&canonical_json(&A1RCayleyMetricIdentityWire {
        schema: 1,
        domain: "uor-r4.a1r-symmetric-construction-cayley-metric/1",
        h4_root_table_kappa: &table.h4_root_table_kappa,
        multiplication_table_kappa: &table.multiplication_table_kappa,
        generators: &generators,
        distances: &distances,
    })?)?;
    Ok(A1RCayleyMetric {
        construction_leaf_states,
        generators,
        distances,
        all_states_reachable: distance_by_index.iter().all(Option::is_some),
        distance_by_index,
        metric_kappa,
        conjugator,
    })
}

fn a1r_state_from_offset(
    offset: u16,
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, RecursiveGeometricAttentionError> {
    let index =
        crate::canonical_lexical_ingestion::OpaqueH4TableIndex::from_table_offset(offset, table)
            .ok_or_else(|| {
                RecursiveGeometricAttentionError::InvalidProbe(format!(
                    "H4 table offset {offset} is outside the frozen table"
                ))
            })?;
    Ok(OrderedH4FoldState::from_table_index(index, table)?)
}

fn a1r_state_witness(
    state: OrderedH4FoldState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RStateWitness, RecursiveGeometricAttentionError> {
    Ok(A1RStateWitness {
        opaque_table_offset: state.table_index().table_offset(),
        root_coordinate: state.root_coordinate(table)?,
    })
}

fn a1r_fold_states(
    states: &[OrderedH4FoldState],
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, RecursiveGeometricAttentionError> {
    let mut fold = OrderedH4FoldState::identity(table)?;
    for state in states {
        fold = fold.compose(*state, table)?;
    }
    Ok(fold)
}

fn a1r_fold_addresses(
    addresses: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, RecursiveGeometricAttentionError> {
    let states = addresses
        .iter()
        .map(|address| h4_leaf_state_for_address(address, table))
        .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
    a1r_fold_states(&states, table)
}

fn a1r_history_addresses(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    history: &[String],
) -> Result<Vec<GeometricAddress>, RecursiveGeometricAttentionError> {
    history
        .iter()
        .map(|token| lexical_address(codec, construction_artifact, token))
        .collect()
}

fn a1r_permutation_collision_census(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RPermutationCollisionCensus, RecursiveGeometricAttentionError> {
    const POPULATION: [&str; 5] = ["aa", "bb", "cc", "dd", "qq"];
    let mut permutations = Vec::with_capacity(120);
    a1r_permute_tokens(
        &POPULATION,
        &mut Vec::new(),
        &mut [false; 5],
        &mut permutations,
    );
    if permutations.len() != 120 {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
            "five-token collision census produced {} rather than 120 permutations",
            permutations.len()
        )));
    }
    let mut outcomes = Vec::with_capacity(permutations.len());
    let mut buckets = BTreeMap::<u16, Vec<Vec<String>>>::new();
    for permutation in permutations {
        let addresses = permutation
            .iter()
            .map(|token| lexical_address(codec, construction_artifact, token))
            .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
        let state = a1r_fold_addresses(&addresses, table)?;
        let ordered_tokens: Vec<String> = permutation
            .iter()
            .map(|token| (*token).to_owned())
            .collect();
        buckets
            .entry(state.table_index().table_offset())
            .or_default()
            .push(ordered_tokens.clone());
        outcomes.push(A1RPermutationOutcome {
            ordered_tokens,
            fold_state: a1r_state_witness(state, table)?,
        });
    }
    let collision_buckets = buckets
        .iter()
        .filter(|(_, sequences)| sequences.len() > 1)
        .map(|(offset, sequences)| {
            Ok(A1RCollisionBucket {
                fold_state: a1r_state_witness(a1r_state_from_offset(*offset, table)?, table)?,
                ordered_token_sequences: sequences.clone(),
            })
        })
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let largest_collision_bucket_size = buckets.values().map(Vec::len).max().unwrap_or(0);
    Ok(A1RPermutationCollisionCensus {
        population: POPULATION.into_iter().map(str::to_owned).collect(),
        expected_permutations: 120,
        examined_permutations: outcomes.len(),
        unique_states: buckets.len(),
        collision_free: buckets.len() == outcomes.len(),
        largest_collision_bucket_size,
        outcomes,
        collision_buckets,
    })
}

fn a1r_permute_tokens<'a>(
    population: &'a [&'a str; 5],
    prefix: &mut Vec<&'a str>,
    used: &mut [bool; 5],
    permutations: &mut Vec<Vec<&'a str>>,
) {
    if prefix.len() == population.len() {
        permutations.push(prefix.clone());
        return;
    }
    for index in 0..population.len() {
        if used[index] {
            continue;
        }
        used[index] = true;
        prefix.push(population[index]);
        a1r_permute_tokens(population, prefix, used, permutations);
        prefix.pop();
        used[index] = false;
    }
}

fn a1r_scope_contrast(
    contract: &A10EvaluationContract,
    left: &AttentionOrderedFoldTrace,
    right: &AttentionOrderedFoldTrace,
    legacy_non_digest_summary_equal: bool,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RScopeContrast, RecursiveGeometricAttentionError> {
    let expected = [true, true, true, false, false, false, true];
    let levels = a1r_compare_scope_levels(left, right, &expected, table)?;
    Ok(A1RScopeContrast {
        contrast_id: contract.contrast_id.clone(),
        left_overlay_kappa: left.overlay_kappa.clone(),
        right_overlay_kappa: right.overlay_kappa.clone(),
        scope_mask_exact: levels
            .iter()
            .all(|level| level.expected_equal == level.observed_equal),
        legacy_non_digest_summary_equal,
        levels,
    })
}

fn a1r_compare_scope_levels(
    left: &AttentionOrderedFoldTrace,
    right: &AttentionOrderedFoldTrace,
    expected_equal: &[bool; 7],
    table: &H4BinaryIcosahedralClosure,
) -> Result<Vec<A1RScopeLevelComparison>, RecursiveGeometricAttentionError> {
    if left.ordered_levels.len() != REQUIRED_LEVELS.len()
        || right.ordered_levels.len() != REQUIRED_LEVELS.len()
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "ordered H4 overlay did not expose seven required levels".to_owned(),
        ));
    }
    left.ordered_levels
        .iter()
        .zip(&right.ordered_levels)
        .zip(REQUIRED_LEVELS.into_iter().zip(expected_equal))
        .map(|((left, right), (required, expected_equal))| {
            if left.level != required || right.level != required {
                return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                    "ordered H4 overlay level order diverged at {required}"
                )));
            }
            Ok(A1RScopeLevelComparison {
                level: required.to_owned(),
                expected_equal: *expected_equal,
                observed_equal: left.state == right.state,
                left_observed_routes: left.observed_routes,
                right_observed_routes: right.observed_routes,
                left_state: a1r_state_witness(left.state, table)?,
                right_state: a1r_state_witness(right.state, table)?,
            })
        })
        .collect()
}

fn a1r_ordered_level_state(
    trace: &AttentionOrderedFoldTrace,
    level: &str,
) -> Result<OrderedH4FoldState, RecursiveGeometricAttentionError> {
    trace
        .ordered_levels
        .iter()
        .find(|candidate| candidate.level == level)
        .map(|candidate| candidate.state)
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(format!(
                "ordered H4 overlay is missing {level}"
            ))
        })
}

fn a1r_grouping_check(
    contrast_id: &str,
    side: &str,
    history: &[GeometricAddress],
    trace: &AttentionOrderedFoldTrace,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RGroupingCheck, RecursiveGeometricAttentionError> {
    if history.len() < 2 {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "A1R grouping check requires at least two observed routes".to_owned(),
        ));
    }
    let flat = a1r_fold_addresses(history, table)?;
    let split = 2.min(history.len());
    let regrouped = a1r_fold_addresses(&history[..split], table)?
        .compose(a1r_fold_addresses(&history[split..], table)?, table)?;
    let current = a1r_ordered_level_state(trace, "current")?;
    let previous = a1r_ordered_level_state(trace, "previous")?;
    let last_two = a1r_ordered_level_state(trace, "last-two")?;
    let sentence = a1r_ordered_level_state(trace, "sentence")?;
    let paragraph = a1r_ordered_level_state(trace, "paragraph")?;
    let conversation = a1r_ordered_level_state(trace, "conversation")?;
    Ok(A1RGroupingCheck {
        contrast_id: contrast_id.to_owned(),
        side: side.to_owned(),
        flat_state: a1r_state_witness(flat, table)?,
        regrouped_state: a1r_state_witness(regrouped, table)?,
        current_matches_leaf: current
            == h4_leaf_state_for_address(
                history.last().ok_or_else(|| {
                    RecursiveGeometricAttentionError::InvalidProbe(
                        "A1R history unexpectedly became empty".to_owned(),
                    )
                })?,
                table,
            )?,
        previous_matches_leaf: previous
            == h4_leaf_state_for_address(&history[history.len() - 2], table)?,
        last_two_matches_direct_fold: last_two
            == a1r_fold_addresses(&history[history.len() - 2..], table)?,
        sentence_matches_flat_fold: sentence == flat,
        paragraph_matches_sentence: paragraph == sentence,
        conversation_matches_paragraph: conversation == paragraph,
        regrouping_exact: regrouped == flat,
    })
}

fn a1r_recursive_hierarchy_fixture(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RRecursiveHierarchyFixture, RecursiveGeometricAttentionError> {
    let turns = vec![
        vec![vec![vec!["aa", "bb"], vec!["dd"]], vec![vec!["cc", "qq"]]],
        vec![vec![vec!["bb", "aa"], vec!["dd", "cc"]]],
    ];
    let global_snapshot_units = vec![GLOBAL_TOKEN.as_bytes().to_vec()];
    let input = ConversationInput {
        identity_scope: "issue-967/a1r-recursive-hierarchy-fixture".to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: turns
            .iter()
            .enumerate()
            .map(|(turn_index, paragraphs)| TurnInput {
                turn_id: format!("turn-{turn_index:04}"),
                paragraphs: paragraphs
                    .iter()
                    .map(|sentences| ParagraphInput {
                        sentences: sentences
                            .iter()
                            .map(|tokens| tokens.join(" ").into_bytes())
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let trace = CanonicalRouteArtifact::ingest(codec, &input)?
        .attention_consumer_trace_with_ordered_h4(table)?;

    let mut flat_addresses = Vec::new();
    let mut sentence_states = Vec::new();
    let mut paragraph_states = Vec::new();
    let mut paragraph_count = 0usize;
    let mut sentence_count = 0usize;
    for paragraphs in &turns {
        for sentences in paragraphs {
            paragraph_count = paragraph_count.saturating_add(1);
            let mut states_in_paragraph = Vec::new();
            for tokens in sentences {
                sentence_count = sentence_count.saturating_add(1);
                let addresses = tokens
                    .iter()
                    .map(|token| lexical_address(codec, construction_artifact, token))
                    .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
                flat_addresses.extend(addresses.iter().cloned());
                let sentence_state = a1r_fold_addresses(&addresses, table)?;
                sentence_states.push(sentence_state);
                states_in_paragraph.push(sentence_state);
            }
            paragraph_states.push(a1r_fold_states(&states_in_paragraph, table)?);
        }
    }
    let flat = a1r_fold_addresses(&flat_addresses, table)?;
    let sentence_regrouped = a1r_fold_states(&sentence_states, table)?;
    let paragraph_regrouped = a1r_fold_states(&paragraph_states, table)?;
    let conversation = a1r_ordered_level_state(&trace, "conversation")?;
    let flat_equals_sentence_regrouped = flat == sentence_regrouped;
    let flat_equals_paragraph_regrouped = flat == paragraph_regrouped;
    let flat_equals_recursive_conversation = flat == conversation;
    Ok(A1RRecursiveHierarchyFixture {
        declaration: "two turns, three paragraphs, and five sentences independently regrouped from the same nine lexical units".to_owned(),
        turn_count: turns.len(),
        paragraph_count,
        sentence_count,
        lexical_unit_count: flat_addresses.len(),
        flat_state: a1r_state_witness(flat, table)?,
        sentence_regrouped_state: a1r_state_witness(sentence_regrouped, table)?,
        paragraph_regrouped_state: a1r_state_witness(paragraph_regrouped, table)?,
        conversation_state: a1r_state_witness(conversation, table)?,
        flat_equals_sentence_regrouped,
        flat_equals_paragraph_regrouped,
        flat_equals_recursive_conversation,
        all_regroupings_exact: flat_equals_sentence_regrouped
            && flat_equals_paragraph_regrouped
            && flat_equals_recursive_conversation,
    })
}

fn a1r_global_order_fixture(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    attention: &GeometricAttentionArtifact,
    child_manifest_addresses: &[GeometricAddress],
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RGlobalOrderFixture, RecursiveGeometricAttentionError> {
    let history = vec!["aa".to_owned(), "bb".to_owned(), "cc".to_owned()];
    let left_units = vec![b"aa".to_vec(), b"bb".to_vec(), b"dd".to_vec()];
    let right_units = vec![b"bb".to_vec(), b"aa".to_vec(), b"dd".to_vec()];
    let left_input = a1r_global_fixture_input(&history, left_units.clone())?;
    let right_input = a1r_global_fixture_input(&history, right_units.clone())?;
    let left_epoch = left_input.global_epoch.clone();
    let right_epoch = right_input.global_epoch.clone();
    let left = CanonicalRouteArtifact::ingest(codec, &left_input)?
        .attention_consumer_trace_with_ordered_h4(table)?;
    let right = CanonicalRouteArtifact::ingest(codec, &right_input)?
        .attention_consumer_trace_with_ordered_h4(table)?;
    let left_path = candidate_path(
        codec,
        construction_artifact,
        attention,
        child_manifest_addresses,
        &history,
        "ll",
    )?;
    let right_path = candidate_path(
        codec,
        construction_artifact,
        attention,
        child_manifest_addresses,
        &history,
        "ll",
    )?;
    let expected = [true, true, true, true, true, true, false];
    let levels = a1r_compare_scope_levels(&left, &right, &expected, table)?;
    let candidate_rows_equal = a1r_row_source_outcome_signature(&left_path)
        == a1r_row_source_outcome_signature(&right_path);
    let candidate_support_equal =
        a1r_candidate_origin_signature(&left_path) == a1r_candidate_origin_signature(&right_path);
    let candidate_work_budget_equal =
        a1r_path_budget_signature(&left_path) == a1r_path_budget_signature(&right_path);
    let lower_scope_inputs_equal = left_input.identity_scope == right_input.identity_scope
        && left_input.turns == right_input.turns;
    Ok(A1RGlobalOrderFixture {
        declaration: "only construction-independent global snapshot order differs".to_owned(),
        left_global_snapshot_units: left_units.iter().map(hex::encode).collect(),
        right_global_snapshot_units: right_units.iter().map(hex::encode).collect(),
        left_global_epoch: left_epoch,
        right_global_epoch: right_epoch,
        global_epoch_is_derived_identity: true,
        lower_scope_inputs_equal,
        candidate_rows_equal,
        candidate_support_equal,
        candidate_work_budget_equal,
        left_support_denominator_kappa: a1r_support_denominator_kappa(&left_path)?,
        right_support_denominator_kappa: a1r_support_denominator_kappa(&right_path)?,
        only_global_state_differs: levels
            .iter()
            .all(|level| level.expected_equal == level.observed_equal)
            && lower_scope_inputs_equal
            && candidate_rows_equal
            && candidate_support_equal
            && candidate_work_budget_equal,
        levels,
    })
}

fn a1r_global_fixture_input(
    history: &[String],
    global_snapshot_units: Vec<Vec<u8>>,
) -> Result<ConversationInput, RecursiveGeometricAttentionError> {
    Ok(ConversationInput {
        identity_scope: "issue-967/a1r-global-order-fixture".to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: "turn-0001".to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: vec![history.join(" ").into_bytes()],
            }],
        }],
    })
}

fn a1r_candidate_predecessors(
    partition: &A10FrozenPartition,
) -> Result<BTreeMap<String, String>, RecursiveGeometricAttentionError> {
    let mut predecessors = BTreeMap::new();
    for sentence in &partition.construction_sentences {
        if let [predecessor, candidate] = sentence.as_slice() {
            if let Some(existing) = predecessors.insert(candidate.clone(), predecessor.clone()) {
                if existing != *predecessor {
                    return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                        "candidate {candidate:?} has multiple construction predecessors"
                    )));
                }
            }
        }
    }
    if predecessors.get("ll").map(String::as_str) != Some("uu")
        || predecessors.get("rr").map(String::as_str) != Some("vv")
        || predecessors.len() != 2
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "frozen construction predecessor map is not ll<-uu and rr<-vv".to_owned(),
        ));
    }
    Ok(predecessors)
}

fn a1r_incremental_check(
    contrast_id: &str,
    side: &str,
    intended_target: &str,
    history: &[GeometricAddress],
    attention: &GeometricAttentionArtifact,
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RIncrementalCheck, RecursiveGeometricAttentionError> {
    let mut incremental = attention.causal_ordered_state_from_history(history, table)?;
    let prefix = incremental.clone();
    let prefix_state = incremental.fold_state();
    let target = lexical_address(codec, construction_artifact, intended_target)?;
    attention.observe_ordered(&mut incremental, &target, table)?;
    let mut extended = history.to_vec();
    extended.push(target);
    let rebuilt = attention.causal_ordered_state_from_history(&extended, table)?;
    Ok(A1RIncrementalCheck {
        contrast_id: contrast_id.to_owned(),
        side: side.to_owned(),
        intended_target: intended_target.to_owned(),
        prefix_state: a1r_state_witness(prefix_state, table)?,
        incremental_state: a1r_state_witness(incremental.fold_state(), table)?,
        rebuilt_state: a1r_state_witness(rebuilt.fold_state(), table)?,
        incremental_observed_routes: incremental.observed_routes(),
        rebuilt_observed_routes: rebuilt.observed_routes(),
        prefix_clone_unchanged: prefix.fold_state() == prefix_state
            && prefix.observed_routes()
                == u32::try_from(history.len()).map_err(|_| {
                    RecursiveGeometricAttentionError::InvalidProbe(
                        "A1R history length exceeds u32".to_owned(),
                    )
                })?,
        exact_reproduction: incremental == rebuilt,
    })
}

#[derive(Debug, Clone)]
struct A1RAnchoredCandidate {
    support: A1RCandidateSupport,
    candidate_state: OrderedH4FoldState,
    predecessor_token: String,
    predecessor_state: OrderedH4FoldState,
}

#[derive(Debug, Clone)]
enum A1RControlMode {
    Geometry(OrderedH4FoldState),
    LegacyAdditive(A10AttentionLevelNonDigest),
    FactorCount,
    ExactRecall,
}

#[allow(clippy::too_many_arguments)]
fn a1r_candidate_query(
    contract: &A10EvaluationContract,
    side: &str,
    history: &[String],
    path: &A10CandidatePath,
    trace: &AttentionOrderedFoldTrace,
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    predecessors: &BTreeMap<String, String>,
    metric: &A1RCayleyMetric,
    table: &H4BinaryIcosahedralClosure,
    score_controls: bool,
) -> Result<A1RCandidateQuery, RecursiveGeometricAttentionError> {
    let intended_target = match side {
        "left" => &contract.left_target,
        "right" => &contract.right_target,
        _ => {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                "unsupported A1R query side {side:?}"
            )))
        }
    };
    if path.intended_target_token != *intended_target {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
            "candidate path target {} disagrees with {side} contract target {intended_target}",
            path.intended_target_token
        )));
    }
    let mut candidate_support = Vec::with_capacity(path.candidates.len());
    let mut anchored = Vec::new();
    let mut excluded_candidates = Vec::new();
    for candidate in &path.candidates {
        let token = std::str::from_utf8(&candidate.address_value.payload_bytes)
            .map_err(|error| {
                RecursiveGeometricAttentionError::InvalidProbe(format!(
                    "candidate payload is not a fixed UTF-8 token: {error}"
                ))
            })?
            .to_owned();
        let candidate_address = lexical_address(codec, construction_artifact, &token)?;
        let reproduced_value = construction_artifact
            .lexical_route_value_for_address_from_validated_artifact(&candidate_address)?
            .ok_or_else(|| {
                RecursiveGeometricAttentionError::InvalidProbe(format!(
                    "candidate {token:?} did not invert through the frozen lexical artifact"
                ))
            })?;
        let exact_payload_inversion = candidate_address.canonical_kappa()?
            == candidate.address_value.address_kappa
            && reproduced_value.address_kappa == candidate.address_value.address_kappa
            && reproduced_value.payload_cid == candidate.address_value.payload_cid
            && reproduced_value.payload_bytes == candidate.address_value.payload_bytes;
        let support = A1RCandidateSupport {
            token: token.clone(),
            address_kappa: candidate.address_value.address_kappa.clone(),
            payload_cid: candidate.address_value.payload_cid.clone(),
            payload_bytes: candidate.address_value.payload_bytes.clone(),
            exact_payload_inversion,
            source_counts: candidate.source_counts,
            contributing_sources: candidate.contributing_sources.clone(),
        };
        if let Some(predecessor_token) = predecessors.get(&token) {
            if !exact_payload_inversion {
                return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                    "candidate {token:?} address and payload did not reproduce independently"
                )));
            }
            let predecessor_address =
                lexical_address(codec, construction_artifact, predecessor_token)?;
            anchored.push(A1RAnchoredCandidate {
                support: support.clone(),
                candidate_state: h4_leaf_state_for_address(&candidate_address, table)?,
                predecessor_token: predecessor_token.clone(),
                predecessor_state: h4_leaf_state_for_address(&predecessor_address, table)?,
            });
        } else {
            excluded_candidates.push(A1RExcludedCandidate {
                address_kappa: support.address_kappa.clone(),
                payload_cid: support.payload_cid.clone(),
                payload_bytes: support.payload_bytes.clone(),
                reason: "NO_UNIQUE_FROZEN_CONSTRUCTION_PREDECESSOR".to_owned(),
            });
        }
        candidate_support.push(support);
    }
    anchored.sort_by(|left, right| left.support.token.cmp(&right.support.token));
    excluded_candidates.sort_by(|left, right| left.address_kappa.cmp(&right.address_kappa));

    let full = a1r_ordered_level_state(trace, "sentence")?;
    let current = a1r_ordered_level_state(trace, "current")?;
    let legacy_additive = trace
        .ordered_levels
        .iter()
        .find(|level| level.level == "sentence")
        .map(|level| non_digest_level(&level.consumer_level))
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(
                "A1R trace is missing the existing additive sentence summary".to_owned(),
            )
        })?;
    let conjugator_inverse = metric.conjugator.inverse(table)?;
    let conjugated = metric
        .conjugator
        .compose(full, table)?
        .compose(conjugator_inverse, table)?;
    let identity = OrderedH4FoldState::identity(table)?;
    let inverse = full.inverse(table)?;
    let controls = if score_controls {
        vec![
            a1r_control_result(
                "full-ordered-hierarchy",
                "symmetric-cayley-word-distance-to-identity",
                A1RControlMode::Geometry(full),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "current-only",
                "symmetric-cayley-word-distance-to-identity",
                A1RControlMode::Geometry(current),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "existing-additive-summary",
                "no-predeclared-candidate-relative-scorer-for-frozen-additive-state",
                A1RControlMode::LegacyAdditive(legacy_additive),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "factor-count",
                "inverse-total-frozen-source-count",
                A1RControlMode::FactorCount,
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "deterministic-conjugation",
                "symmetric-cayley-word-distance-to-identity",
                A1RControlMode::Geometry(conjugated),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "hierarchy-disabled",
                "symmetric-cayley-word-distance-to-identity",
                A1RControlMode::Geometry(identity),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "exact-recall-only",
                "exact-row-hit-binary-energy",
                A1RControlMode::ExactRecall,
                &anchored,
                intended_target,
                metric,
                table,
            )?,
            a1r_control_result(
                "inverse-h4-intervention",
                "symmetric-cayley-word-distance-to-identity",
                A1RControlMode::Geometry(inverse),
                &anchored,
                intended_target,
                metric,
                table,
            )?,
        ]
    } else {
        Vec::new()
    };
    Ok(A1RCandidateQuery {
        contrast_id: contract.contrast_id.clone(),
        side: side.to_owned(),
        intended_target: intended_target.clone(),
        history: history.to_vec(),
        candidate_entries_examined: path.candidate_entries_examined,
        candidate_entry_ceiling: path.candidate_entry_ceiling,
        unique_candidates_before_admission: path.unique_candidates_before_admission,
        unique_candidates_after_admission: path.unique_candidates_after_admission,
        retained_candidate_ceiling: path.retained_candidate_ceiling,
        full_pre_admission_union_observed: path.full_pre_admission_union_observed,
        anchored_candidates_after_admission: anchored.len(),
        required_anchored_candidates: predecessors.len(),
        admission_truncated_union: path.admission_truncated_union,
        exact_direct_rows_miss: path.exact_direct_rows_miss,
        divisor_rows_miss: path
            .rows
            .iter()
            .filter(|row| row.source == "divisor")
            .all(|row| !row.hit),
        adjacent_spin_only_support: candidate_support.iter().all(|candidate| {
            candidate.source_counts.last_one == 0
                && candidate.source_counts.last_two == 0
                && candidate.source_counts.ordered_sentence == 0
                && candidate.source_counts.divisor == 0
                && candidate.source_counts.adjacent_spin > 0
                && candidate.contributing_sources.len() == 1
                && candidate.contributing_sources.first().map(String::as_str)
                    == Some("adjacent-spin")
        }),
        target_injected: path.target_injected,
        future_events_visible: path.future_events_visible,
        support_denominator_kappa: a1r_support_denominator_kappa(path)?,
        rows: path.rows.clone(),
        exact_candidate_payload_inversions: candidate_support
            .iter()
            .filter(|candidate| candidate.exact_payload_inversion)
            .count(),
        candidate_support,
        excluded_candidates,
        controls,
    })
}

fn a1r_support_denominator_kappa(
    path: &A10CandidatePath,
) -> Result<String, RecursiveGeometricAttentionError> {
    canonical_kappa(&canonical_json(&candidate_support_signature(path))?)
}

fn a1r_path_contains_token(path: &A10CandidatePath, token: &str) -> bool {
    path.candidates
        .iter()
        .any(|candidate| candidate.address_value.payload_bytes == token.as_bytes())
}

fn a1r_commutator(
    left: OrderedH4FoldState,
    right: OrderedH4FoldState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, RecursiveGeometricAttentionError> {
    Ok(left
        .compose(right, table)?
        .compose(left.inverse(table)?, table)?
        .compose(right.inverse(table)?, table)?)
}

fn a1r_control_result(
    control: &str,
    energy_kind: &str,
    mode: A1RControlMode,
    candidates: &[A1RAnchoredCandidate],
    intended_target: &str,
    metric: &A1RCayleyMetric,
    table: &H4BinaryIcosahedralClosure,
) -> Result<A1RControlResult, RecursiveGeometricAttentionError> {
    let hierarchy_state = match &mode {
        A1RControlMode::Geometry(state) => Some(a1r_state_witness(*state, table)?),
        A1RControlMode::LegacyAdditive(_)
        | A1RControlMode::FactorCount
        | A1RControlMode::ExactRecall => None,
    };
    let legacy_additive_state = match &mode {
        A1RControlMode::LegacyAdditive(state) => Some(state.clone()),
        A1RControlMode::Geometry(_) | A1RControlMode::FactorCount | A1RControlMode::ExactRecall => {
            None
        }
    };
    let mut interactions = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let (interaction, predecessor_interaction, relative, energy) = match &mode {
            A1RControlMode::Geometry(hierarchy) => {
                let interaction = a1r_commutator(*hierarchy, candidate.candidate_state, table)?;
                let predecessor_interaction = a1r_commutator(
                    candidate.predecessor_state,
                    candidate.candidate_state,
                    table,
                )?;
                let relative =
                    interaction.compose(predecessor_interaction.inverse(table)?, table)?;
                let distance = metric.distance_by_index
                    [usize::from(relative.table_index().table_offset())]
                .map(u32::from)
                .unwrap_or(u32::MAX);
                (
                    Some(a1r_state_witness(interaction, table)?),
                    Some(a1r_state_witness(predecessor_interaction, table)?),
                    Some(a1r_state_witness(relative, table)?),
                    Some(distance),
                )
            }
            A1RControlMode::LegacyAdditive(_) => (None, None, None, None),
            A1RControlMode::FactorCount => {
                let counts = candidate.support.source_counts;
                let total = counts
                    .last_one
                    .saturating_add(counts.last_two)
                    .saturating_add(counts.ordered_sentence)
                    .saturating_add(counts.divisor)
                    .saturating_add(counts.adjacent_spin);
                (None, None, None, Some(u32::MAX.saturating_sub(total)))
            }
            A1RControlMode::ExactRecall => {
                let counts = candidate.support.source_counts;
                let exact_hit =
                    counts.last_one > 0 || counts.last_two > 0 || counts.ordered_sentence > 0;
                (None, None, None, exact_hit.then_some(0))
            }
        };
        interactions.push(A1RCandidateInteraction {
            token: candidate.support.token.clone(),
            address_kappa: candidate.support.address_kappa.clone(),
            payload_cid: candidate.support.payload_cid.clone(),
            source_counts: candidate.support.source_counts,
            candidate_state: a1r_state_witness(candidate.candidate_state, table)?,
            predecessor_token: candidate.predecessor_token.clone(),
            predecessor_state: a1r_state_witness(candidate.predecessor_state, table)?,
            interaction_state: interaction,
            predecessor_interaction_state: predecessor_interaction,
            relative_state: relative,
            energy,
        });
    }
    let minimum_energy = interactions
        .iter()
        .filter_map(|candidate| candidate.energy)
        .min();
    let mut minimum_energy_candidates = minimum_energy.map_or_else(Vec::new, |minimum| {
        interactions
            .iter()
            .filter(|candidate| candidate.energy == Some(minimum))
            .map(|candidate| candidate.token.clone())
            .collect::<Vec<_>>()
    });
    minimum_energy_candidates.sort();
    let canonical_address_tiebreak_token = interactions
        .iter()
        .filter(|candidate| minimum_energy_candidates.contains(&candidate.token))
        .min_by(|left, right| left.address_kappa.cmp(&right.address_kappa))
        .map(|candidate| candidate.token.clone());
    let exercised = !matches!(&mode, A1RControlMode::LegacyAdditive(_));
    let tie = exercised && minimum_energy_candidates.len() > 1;
    let selected_token = (exercised && minimum_energy_candidates.len() == 1)
        .then(|| minimum_energy_candidates[0].clone());
    let abstained = selected_token.is_none();
    let status = if !exercised {
        "NOT_EXERCISED_NO_PREDECLARED_ADDITIVE_SCORER"
    } else if minimum_energy_candidates.is_empty() && matches!(&mode, A1RControlMode::ExactRecall) {
        "EXERCISED_NO_EXACT_HIT_ABSTAIN"
    } else if minimum_energy_candidates.is_empty() {
        "EXERCISED_EMPTY_CANDIDATE_ABSTAIN"
    } else if tie {
        "EXERCISED_TIE_ABSTAIN"
    } else {
        "EXERCISED_STRICT_SELECTION"
    };
    let intended_target_selected = selected_token
        .as_deref()
        .is_some_and(|token| token == intended_target);
    Ok(A1RControlResult {
        control: control.to_owned(),
        status: status.to_owned(),
        exercised,
        energy_kind: energy_kind.to_owned(),
        hierarchy_state,
        legacy_additive_state,
        candidates: interactions,
        minimum_energy,
        minimum_energy_candidates,
        canonical_address_tiebreak_rule: "lexicographically-smallest-address-kappa-diagnostic-only"
            .to_owned(),
        canonical_address_tiebreak_token,
        selected_token,
        tie,
        abstained,
        intended_target_selected,
    })
}

fn a1r_scoring_summary(queries: &[A1RCandidateQuery], score_controls: bool) -> A1RScoringSummary {
    if !score_controls {
        return A1RScoringSummary {
            status: "NOT_EXERCISED_STRUCTURAL_OR_SUPPORT_GATE".to_owned(),
            required_queries: FIXED_CONTRASTS.len().saturating_mul(2),
            exercised_queries: 0,
            full_strict_correct: 0,
            full_ties: 0,
            control_strict_correct: Vec::new(),
            all_required_controls_present: false,
            all_required_controls_exercised: false,
            any_exercised_control_not_weaker: false,
            every_control_weaker: false,
            positive_contract_satisfied: false,
        };
    }
    let mut aggregates = BTreeMap::<String, (usize, usize, usize, usize)>::new();
    let mut full_strict_correct = 0usize;
    let mut full_ties = 0usize;
    let mut full_exercised_queries = 0usize;
    let required_controls = A1R_REQUIRED_CONTROLS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let all_required_controls_present = queries.iter().all(|query| {
        query.controls.len() == CONTROL_ARMS_PER_QUERY
            && query
                .controls
                .iter()
                .map(|control| control.control.clone())
                .collect::<BTreeSet<_>>()
                == required_controls
    });
    for query in queries {
        for control in &query.controls {
            let aggregate = aggregates.entry(control.control.clone()).or_default();
            if control.exercised {
                aggregate.0 += 1;
            }
            if control.intended_target_selected {
                aggregate.1 += 1;
            }
            if control.tie {
                aggregate.2 += 1;
            }
            if control.abstained {
                aggregate.3 += 1;
            }
            if control.control == "full-ordered-hierarchy" {
                if control.exercised {
                    full_exercised_queries += 1;
                }
                if control.intended_target_selected {
                    full_strict_correct += 1;
                }
                if control.tie {
                    full_ties += 1;
                }
            }
        }
    }
    let control_strict_correct = aggregates
        .iter()
        .filter(|(control, _)| control.as_str() != "full-ordered-hierarchy")
        .map(
            |(control, (exercised_queries, strict_correct, ties, abstentions))| {
                A1RControlAggregate {
                    control: control.clone(),
                    exercised_queries: *exercised_queries,
                    strict_correct: *strict_correct,
                    ties: *ties,
                    abstentions: *abstentions,
                }
            },
        )
        .collect::<Vec<_>>();
    let all_required_controls_exercised = all_required_controls_present
        && aggregates
            .iter()
            .all(|(_, (exercised, _, _, _))| *exercised == queries.len());
    let every_control_weaker = all_required_controls_exercised
        && control_strict_correct
            .iter()
            .all(|control| control.strict_correct < full_strict_correct);
    let any_exercised_control_not_weaker = control_strict_correct.iter().any(|control| {
        control.exercised_queries == queries.len() && control.strict_correct >= full_strict_correct
    });
    let status = if !all_required_controls_present {
        "INVALID_REQUIRED_CONTROL_SET"
    } else if !all_required_controls_exercised {
        "EXERCISED_WITH_UNAVAILABLE_MATCHED_CONTROL"
    } else {
        "EXERCISED_ALL_MATCHED_CONTROLS"
    };
    A1RScoringSummary {
        status: status.to_owned(),
        required_queries: FIXED_CONTRASTS.len().saturating_mul(2),
        exercised_queries: full_exercised_queries,
        full_strict_correct,
        full_ties,
        all_required_controls_present,
        all_required_controls_exercised,
        any_exercised_control_not_weaker,
        every_control_weaker,
        positive_contract_satisfied: all_required_controls_present
            && all_required_controls_exercised
            && full_exercised_queries == FIXED_CONTRASTS.len().saturating_mul(2)
            && full_strict_correct == queries.len()
            && every_control_weaker,
        control_strict_correct,
    }
}

fn a1r_transition_readout_summary(queries: &[A1RCandidateQuery]) -> A1RTransitionReadoutSummary {
    let mut exercised_queries = 0usize;
    let mut distinct_candidate_relative_states = 0usize;
    let mut distinct_states_equal_energy = 0usize;
    for query in queries {
        let Some(control) = a1r_full_control(query) else {
            continue;
        };
        if !control.exercised {
            continue;
        }
        exercised_queries = exercised_queries.saturating_add(1);
        let relative_states = control
            .candidates
            .iter()
            .filter_map(|candidate| candidate.relative_state.as_ref())
            .collect::<Vec<_>>();
        let relative_states_distinct = relative_states.len() == control.candidates.len()
            && relative_states.iter().enumerate().all(|(index, state)| {
                relative_states[..index]
                    .iter()
                    .all(|prior| *prior != *state)
            });
        if relative_states_distinct {
            distinct_candidate_relative_states =
                distinct_candidate_relative_states.saturating_add(1);
        }
        let energies = control
            .candidates
            .iter()
            .filter_map(|candidate| candidate.energy)
            .collect::<Vec<_>>();
        let equal_energy = energies.len() == control.candidates.len()
            && energies
                .first()
                .is_some_and(|first| energies.iter().all(|energy| energy == first));
        if relative_states_distinct && equal_energy {
            distinct_states_equal_energy = distinct_states_equal_energy.saturating_add(1);
        }
    }

    let mut paired_same_candidate_comparisons = 0usize;
    let mut paired_same_candidate_relative_state_differences = 0usize;
    for contrast in &FIXED_CONTRASTS {
        let left = queries
            .iter()
            .find(|query| query.contrast_id == contrast.id && query.side == "left")
            .and_then(a1r_full_control);
        let right = queries
            .iter()
            .find(|query| query.contrast_id == contrast.id && query.side == "right")
            .and_then(a1r_full_control);
        let (Some(left), Some(right)) = (left, right) else {
            continue;
        };
        for token in ["ll", "rr"] {
            let left_state = left
                .candidates
                .iter()
                .find(|candidate| candidate.token == token)
                .and_then(|candidate| candidate.relative_state.as_ref());
            let right_state = right
                .candidates
                .iter()
                .find(|candidate| candidate.token == token)
                .and_then(|candidate| candidate.relative_state.as_ref());
            if let (Some(left_state), Some(right_state)) = (left_state, right_state) {
                paired_same_candidate_comparisons =
                    paired_same_candidate_comparisons.saturating_add(1);
                if left_state != right_state {
                    paired_same_candidate_relative_state_differences =
                        paired_same_candidate_relative_state_differences.saturating_add(1);
                }
            }
        }
    }
    A1RTransitionReadoutSummary {
        interaction: "D(H,c)=C(H,c)*C(P_c,c)^-1".to_owned(),
        scalar_readout: "shortest symmetric construction-Cayley word distance to identity"
            .to_owned(),
        exercised_queries,
        queries_with_distinct_candidate_relative_states: distinct_candidate_relative_states,
        queries_with_distinct_relative_states_but_equal_energy: distinct_states_equal_energy,
        paired_same_candidate_comparisons,
        paired_same_candidate_relative_state_differences,
        scalar_readout_degeneracy_observed: exercised_queries > 0
            && distinct_states_equal_energy == exercised_queries,
    }
}

fn a1r_full_control(query: &A1RCandidateQuery) -> Option<&A1RControlResult> {
    query
        .controls
        .iter()
        .find(|control| control.control == "full-ordered-hierarchy")
}

fn frozen_partition() -> A10FrozenPartition {
    A10FrozenPartition {
        declaration: "compile-time fixed before codec, candidate rows, summaries, or statistics"
            .to_owned(),
        registered_vocabulary: REGISTERED_TOKENS.into_iter().map(str::to_owned).collect(),
        construction_sentences: CONSTRUCTION_SENTENCES
            .into_iter()
            .map(|sentence| sentence.iter().map(|token| (*token).to_owned()).collect())
            .collect(),
        evaluation_contrasts: FIXED_CONTRASTS
            .iter()
            .map(|contrast| A10EvaluationContract {
                contrast_id: contrast.id.to_owned(),
                left_history: contrast
                    .left_history
                    .iter()
                    .map(|token| (*token).to_owned())
                    .collect(),
                right_history: contrast
                    .right_history
                    .iter()
                    .map(|token| (*token).to_owned())
                    .collect(),
                left_target: contrast.left_target.to_owned(),
                right_target: contrast.right_target.to_owned(),
            })
            .collect(),
        frozen_before_codec_compile: true,
        frozen_before_candidate_compile: true,
        frozen_before_summary_compile: true,
        targets_selected_from_observed_rows: false,
        evaluation_histories_enter_construction_manifest: false,
    }
}

fn validate_frozen_partition(
    partition: &A10FrozenPartition,
) -> Result<(), RecursiveGeometricAttentionError> {
    if partition.evaluation_contrasts.len() < 3 {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "the fixed population requires at least three matched contrasts".to_owned(),
        ));
    }
    let registered = partition
        .registered_vocabulary
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    if registered.len() != partition.registered_vocabulary.len() {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "registered vocabulary repeats a unit".to_owned(),
        ));
    }
    for contract in &partition.evaluation_contrasts {
        let invariants = matched_history_invariants(contract);
        if !invariants.same_length
            || !invariants.same_multiset
            || !invariants.same_boundary_shape
            || !invariants.same_current
            || !invariants.same_previous
            || !invariants.same_last_two_suffix
            || !invariants.earlier_order_differs
            || !invariants.competing_targets_differ
        {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                "contrast {} violates its predeclared matching contract",
                contract.contrast_id
            )));
        }
        if contract
            .left_history
            .iter()
            .chain(&contract.right_history)
            .chain([&contract.left_target, &contract.right_target])
            .any(|token| !registered.contains(token.as_str()))
        {
            return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
                "contrast {} references an unregistered unit",
                contract.contrast_id
            )));
        }
    }
    Ok(())
}

fn registration_input(
    partition: &A10FrozenPartition,
) -> Result<ConversationInput, RecursiveGeometricAttentionError> {
    let sentences = partition
        .registered_vocabulary
        .iter()
        .map(|token| vec![token.clone()])
        .collect::<Vec<_>>();
    conversation_input("issue-952/a1-registration-only", &sentences)
}

fn construction_input(
    partition: &A10FrozenPartition,
) -> Result<ConversationInput, RecursiveGeometricAttentionError> {
    conversation_input(
        "issue-952/a1-construction-only",
        &partition.construction_sentences,
    )
}

fn evaluation_input(
    contrast_id: &str,
    history: &[String],
) -> Result<ConversationInput, RecursiveGeometricAttentionError> {
    conversation_input(
        &format!("issue-952/a1-evaluation/{contrast_id}"),
        &[history.to_vec()],
    )
}

fn conversation_input(
    identity_scope: &str,
    sentences: &[Vec<String>],
) -> Result<ConversationInput, RecursiveGeometricAttentionError> {
    let global_snapshot_units = vec![GLOBAL_TOKEN.as_bytes().to_vec()];
    Ok(ConversationInput {
        identity_scope: identity_scope.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
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
    })
}

fn evaluate_contrast(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    attention: &GeometricAttentionArtifact,
    child_manifest_addresses: &[GeometricAddress],
    contract: &A10EvaluationContract,
) -> Result<A10ContrastReport, RecursiveGeometricAttentionError> {
    let invariants = matched_history_invariants(contract);
    let left_artifact = CanonicalRouteArtifact::ingest(
        codec,
        &evaluation_input(&contract.contrast_id, &contract.left_history)?,
    )?;
    let right_artifact = CanonicalRouteArtifact::ingest(
        codec,
        &evaluation_input(&contract.contrast_id, &contract.right_history)?,
    )?;
    let left_consumer = left_artifact.attention_consumer_trace()?;
    let right_consumer = right_artifact.attention_consumer_trace()?;
    let collision_census = collision_census(
        &left_consumer.ordered_levels,
        &right_consumer.ordered_levels,
        left_consumer.artifact_manifest_kappa != right_consumer.artifact_manifest_kappa,
    )?;

    let left_candidate_path = candidate_path(
        codec,
        construction_artifact,
        attention,
        child_manifest_addresses,
        &contract.left_history,
        &contract.left_target,
    )?;
    let right_candidate_path = candidate_path(
        codec,
        construction_artifact,
        attention,
        child_manifest_addresses,
        &contract.right_history,
        &contract.right_target,
    )?;
    let left_support = candidate_support_signature(&left_candidate_path);
    let right_support = candidate_support_signature(&right_candidate_path);
    let natural_candidate_union_equal = left_support
        .iter()
        .map(|(address, _)| address)
        .eq(right_support.iter().map(|(address, _)| address));
    let candidate_support_counts_equal = left_support == right_support;
    let both_competing_targets_in_each_union = [
        (&left_candidate_path, &contract.left_target),
        (&left_candidate_path, &contract.right_target),
        (&right_candidate_path, &contract.left_target),
        (&right_candidate_path, &contract.right_target),
    ]
    .into_iter()
    .all(|(path, target)| {
        path.candidates
            .iter()
            .any(|candidate| candidate.address_value.payload_bytes == target.as_bytes())
    });
    let exact_direct_rows_miss_both_sides =
        left_candidate_path.exact_direct_rows_miss && right_candidate_path.exact_direct_rows_miss;
    let ordered_state_collides = collision_census.all_required_levels_present
        && collision_census.all_non_digest_fields_collide
        && invariants.same_length
        && invariants.same_multiset
        && invariants.same_boundary_shape
        && invariants.same_current
        && invariants.same_previous
        && invariants.same_last_two_suffix
        && invariants.earlier_order_differs;
    Ok(A10ContrastReport {
        contrast_id: contract.contrast_id.clone(),
        invariants,
        collision_census,
        left_candidate_path,
        right_candidate_path,
        natural_candidate_union_equal,
        candidate_support_counts_equal,
        both_competing_targets_in_each_union,
        exact_direct_rows_miss_both_sides,
        ordered_state_collides,
    })
}

fn matched_history_invariants(contract: &A10EvaluationContract) -> A10MatchedHistoryInvariants {
    let mut left_multiset = contract.left_history.clone();
    let mut right_multiset = contract.right_history.clone();
    left_multiset.sort();
    right_multiset.sort();
    let left_len = contract.left_history.len();
    let right_len = contract.right_history.len();
    let same_length = left_len == right_len;
    let same_current = contract.left_history.last() == contract.right_history.last();
    let same_previous = left_len >= 2
        && right_len >= 2
        && contract.left_history.get(left_len - 2) == contract.right_history.get(right_len - 2);
    let same_last_two_suffix = left_len >= 2
        && right_len >= 2
        && contract.left_history[left_len - 2..] == contract.right_history[right_len - 2..];
    let earlier_order_differs = left_len >= 2
        && right_len >= 2
        && contract.left_history[..left_len - 2] != contract.right_history[..right_len - 2];
    let equal_token_widths = contract
        .left_history
        .iter()
        .chain(&contract.right_history)
        .all(|token| token.len() == 2);
    A10MatchedHistoryInvariants {
        left_history: contract.left_history.clone(),
        right_history: contract.right_history.clone(),
        left_target: contract.left_target.clone(),
        right_target: contract.right_target.clone(),
        same_length,
        same_multiset: left_multiset == right_multiset,
        same_boundary_shape: same_length && equal_token_widths,
        same_current,
        same_previous,
        same_last_two_suffix,
        earlier_order_differs,
        competing_targets_differ: contract.left_target != contract.right_target,
    }
}

fn collision_census(
    left: &[AttentionLevelTrace],
    right: &[AttentionLevelTrace],
    artifact_identities_differ: bool,
) -> Result<A10CollisionCensus, RecursiveGeometricAttentionError> {
    let left_levels = left.iter().map(non_digest_level).collect::<Vec<_>>();
    let right_levels = right.iter().map(non_digest_level).collect::<Vec<_>>();
    let all_required_levels_present = left_levels.len() == REQUIRED_LEVELS.len()
        && right_levels.len() == REQUIRED_LEVELS.len()
        && left_levels
            .iter()
            .map(|level| level.level.as_str())
            .eq(REQUIRED_LEVELS)
        && right_levels
            .iter()
            .map(|level| level.level.as_str())
            .eq(REQUIRED_LEVELS);
    if !all_required_levels_present {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "attention consumer did not expose the required seven-level order".to_owned(),
        ));
    }
    let per_level_equal = left_levels
        .iter()
        .zip(&right_levels)
        .map(|(left, right)| A10LevelCollision {
            level: left.level.clone(),
            all_non_digest_fields_equal: left == right,
        })
        .collect::<Vec<_>>();
    let digest_identities_differ = artifact_identities_differ
        || left.iter().zip(right).any(|(left, right)| {
            left.identity_kappa != right.identity_kappa
                || left.exact_chain_kappa != right.exact_chain_kappa
                || left.geometric_summary_kappa != right.geometric_summary_kappa
                || left.ordered_child_kappa != right.ordered_child_kappa
                || left.transported_trajectory_kappa != right.transported_trajectory_kappa
        });
    Ok(A10CollisionCensus {
        included_non_digest_fields: NON_DIGEST_ATTENTION_LEVEL_FIELDS
            .into_iter()
            .map(str::to_owned)
            .collect(),
        excluded_digest_identity_fields: EXCLUDED_DIGEST_IDENTITY_FIELDS
            .into_iter()
            .map(str::to_owned)
            .collect(),
        included_field_count: NON_DIGEST_ATTENTION_LEVEL_FIELDS.len(),
        excluded_field_count: EXCLUDED_DIGEST_IDENTITY_FIELDS.len(),
        all_required_levels_present,
        all_non_digest_fields_collide: per_level_equal
            .iter()
            .all(|level| level.all_non_digest_fields_equal),
        digest_identities_differ,
        digest_identity_used_for_verdict: false,
        left_levels,
        right_levels,
        per_level_equal,
    })
}

fn non_digest_level(level: &AttentionLevelTrace) -> A10AttentionLevelNonDigest {
    A10AttentionLevelNonDigest {
        level: level.level.clone(),
        identity_kind: level.identity_kind.clone(),
        occurrence: level.occurrence,
        turn: level.turn,
        paragraph: level.paragraph,
        sentence: level.sentence,
        ordinal_in_sentence: level.ordinal_in_sentence,
        lexical_unit_id: level.lexical_unit_id,
        prime: level.prime,
        address_index: level.address_index,
        boundary_kind: level.boundary_kind.clone(),
        boundary_identity: level.boundary_identity.clone(),
        chain_identity: level.chain_identity.clone(),
        direct_child_count: level.direct_child_count,
        observed_descendant_routes: level.observed_descendant_routes,
        window_start: level.window_start,
        window_end: level.window_end,
        session_hypersphere_q30: level.session_hypersphere_q30,
        winding_turns: level.winding_turns,
        projection_energy_q30: level.projection_energy_q30,
        shared_prime_factors: level
            .shared_prime_factors
            .iter()
            .map(|factor| A10PrimeFactor {
                prime: factor.prime,
                count: factor.count,
            })
            .collect(),
        cosine_resonance_q30: level.cosine_resonance_q30,
        accumulated_hopf_phase_q29: level.accumulated_hopf_phase_q29,
        zeta_phase_signature_q29: level.zeta_phase_signature_q29,
        s3_spin_q30: level.s3_spin_q30,
        s2_hopf_observation_q30: level.s2_hopf_observation_q30,
        fiber_q29: level.fiber_q29,
        torsion_q29: level.torsion_q29,
        radial_zphi: level.radial_zphi,
        bridge_mode: level.bridge_mode.clone(),
        active_chart: level.active_chart.clone(),
        selected_adapter: level.selected_adapter.clone(),
        chart_sin_q30: level.chart_sin_q30,
        chart_cos_q30: level.chart_cos_q30,
        chart_activation_q30: level.chart_activation_q30,
        chart_chirality: level.chart_chirality,
        chart_cosine_polarity: level.chart_cosine_polarity,
        quarter_turn_orientation: level.quarter_turn_orientation,
        phase_shift_q29: level.phase_shift_q29,
        torsion_shift_q29: level.torsion_shift_q29,
        transported_fiber_q29: level.transported_fiber_q29,
        transported_torsion_q29: level.transported_torsion_q29,
        inverse_fiber_q29: level.inverse_fiber_q29,
        inverse_torsion_q29: level.inverse_torsion_q29,
        chart_inverse_exact: level.chart_inverse_exact,
        paired_h4_e8_coordinate_sum: level.paired_h4_e8_coordinate_sum,
    }
}

fn candidate_path(
    codec: &CanonicalLexicalCodec,
    construction_artifact: &CanonicalRouteArtifact,
    attention: &GeometricAttentionArtifact,
    child_manifest_addresses: &[GeometricAddress],
    history_tokens: &[String],
    target_token: &str,
) -> Result<A10CandidatePath, RecursiveGeometricAttentionError> {
    let history = history_tokens
        .iter()
        .map(|token| lexical_address(codec, construction_artifact, token))
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let target_unit = lexical_unit_id(codec, target_token)?;
    let target = construction_artifact
        .lexical_route_address_from_validated_artifact(target_unit)?
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(format!(
                "target {target_token} has no registered geometric address"
            ))
        })?;
    let target_address_kappa = target.canonical_kappa()?;
    let state = attention.causal_state_from_history(&history)?;
    // CountOnly still executes the production row union and admission path.
    // No ranking energy contributes to this reachability report.
    let trace = attention.query(&state, AttentionControl::CountOnly)?;
    let rows = trace
        .rows_read
        .iter()
        .map(row_origin)
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let exact_direct_rows_miss = trace.rows_read.iter().all(|row| {
        !matches!(
            row.source,
            AttentionRowSource::LastOne
                | AttentionRowSource::LastTwo
                | AttentionRowSource::OrderedSentence
        ) || !row.hit
    });
    let exclusions = leakage_exclusions(&trace);
    let candidates = trace
        .candidates
        .iter()
        .map(|candidate| {
            let counts = source_counts(candidate.source_counts);
            Ok(A10CandidateOrigin {
                address_value: address_value(
                    construction_artifact,
                    child_manifest_addresses,
                    &candidate.next,
                )?,
                contributing_sources: contributing_sources(counts),
                source_counts: counts,
            })
        })
        .collect::<Result<Vec<_>, RecursiveGeometricAttentionError>>()?;
    let intended_target_post_admission_reachable = trace
        .candidates
        .iter()
        .any(|candidate| candidate.next == target);
    let admission_truncated_union = trace.unique_candidates_before_ceiling > trace.candidates.len();
    let full_pre_admission_union_observed = !admission_truncated_union;
    let intended_target_pre_admission_reachable = if intended_target_post_admission_reachable {
        Some(true)
    } else if full_pre_admission_union_observed {
        Some(false)
    } else {
        None
    };
    let intended_target_truncated_before_geometry = intended_target_pre_admission_reachable
        .map(|reachable| reachable && !intended_target_post_admission_reachable);
    let incremental_next_state = intended_target_post_admission_reachable
        .then(|| {
            incremental_next_state(
                attention,
                &state,
                &history,
                &target,
                construction_artifact,
                child_manifest_addresses,
            )
        })
        .transpose()?;
    Ok(A10CandidatePath {
        intended_target_token: target_token.to_owned(),
        intended_target_address_kappa: target_address_kappa,
        rows,
        exclusions,
        candidate_entries_examined: trace.candidate_entries_examined,
        candidate_entry_ceiling: trace.candidate_entry_ceiling,
        unique_candidates_before_admission: trace.unique_candidates_before_ceiling,
        unique_candidates_after_admission: trace.candidates.len(),
        retained_candidate_ceiling: trace.candidate_ceiling,
        admission_truncated_union,
        full_pre_admission_union_observed,
        candidates,
        intended_target_pre_admission_reachable,
        intended_target_post_admission_reachable,
        intended_target_truncated_before_geometry,
        exact_direct_rows_miss,
        target_injected: false,
        future_events_visible: false,
        incremental_next_state,
    })
}

fn lexical_unit_id(
    codec: &CanonicalLexicalCodec,
    token: &str,
) -> Result<u32, RecursiveGeometricAttentionError> {
    let encoded = codec.encode(0, 0, token.as_bytes())?;
    if encoded.units.len() != 1
        || !encoded.units[0].leading_bytes.is_empty()
        || !encoded.trailing_bytes.is_empty()
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(format!(
            "fixed token {token:?} does not encode as one boundary-free lexical unit"
        )));
    }
    Ok(encoded.units[0].unit_id)
}

fn lexical_address(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    token: &str,
) -> Result<GeometricAddress, RecursiveGeometricAttentionError> {
    artifact
        .lexical_route_address_from_validated_artifact(lexical_unit_id(codec, token)?)?
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(format!(
                "fixed token {token:?} has no registered route address"
            ))
        })
}

fn row_origin(
    row: &crate::prime_route_geometric_attention::AttentionRowRead,
) -> Result<A10RowOrigin, RecursiveGeometricAttentionError> {
    let (key_kind, key_identity) = match &row.key {
        AttentionRowKey::LastOne(address) => (
            "last-one-address",
            format!("address:{}", address.canonical_kappa()?),
        ),
        AttentionRowKey::LastTwo { previous, last } => (
            "last-two-addresses",
            format!(
                "previous:{};last:{}",
                previous.canonical_kappa()?,
                last.canonical_kappa()?
            ),
        ),
        AttentionRowKey::LastTwoUnavailable => ("last-two-unavailable", "none".to_owned()),
        AttentionRowKey::OrderedSentence(identity) => {
            ("ordered-sentence", format!("ordered-sentence:{identity}"))
        }
        AttentionRowKey::Divisor(atom) => ("prime-divisor", format!("prime:{}", atom.value())),
        AttentionRowKey::AdjacentSpin(sector) => (
            "adjacent-spin-sector",
            format!(
                "hopf-octant:{};torsion-bin:{}",
                sector.hopf_octant, sector.torsion_bin
            ),
        ),
    };
    Ok(A10RowOrigin {
        source: row_source_name(row.source).to_owned(),
        key_kind: key_kind.to_owned(),
        key_identity,
        hit: row.hit,
        candidate_entries_examined: row.candidate_entries_examined,
    })
}

fn row_source_name(source: AttentionRowSource) -> &'static str {
    match source {
        AttentionRowSource::LastOne => "last-one",
        AttentionRowSource::LastTwo => "last-two",
        AttentionRowSource::OrderedSentence => "ordered-sentence",
        AttentionRowSource::Divisor => "divisor",
        AttentionRowSource::AdjacentSpin => "adjacent-spin",
    }
}

fn leakage_exclusions(trace: &GeometricAttentionTrace) -> Vec<A10LeakageExclusion> {
    let direct = |source| {
        trace
            .rows_read
            .iter()
            .find(|row| row.source == source)
            .map_or((false, 0), |row| (row.hit, row.candidate_entries_examined))
    };
    let (local_hit, local_entries) = direct(AttentionRowSource::LastOne);
    let (last_two_hit, last_two_entries) = direct(AttentionRowSource::LastTwo);
    let (sentence_hit, sentence_entries) = direct(AttentionRowSource::OrderedSentence);
    vec![
        A10LeakageExclusion {
            scope: "exact-local".to_owned(),
            status: if local_hit {
                "LEAKED_ROW_HIT"
            } else {
                "EXCLUDED_EXACT_MISS"
            }
            .to_owned(),
            candidate_entries_contributed: local_entries,
        },
        A10LeakageExclusion {
            scope: "exact-last-two".to_owned(),
            status: if last_two_hit {
                "LEAKED_ROW_HIT"
            } else {
                "EXCLUDED_EXACT_MISS"
            }
            .to_owned(),
            candidate_entries_contributed: last_two_entries,
        },
        A10LeakageExclusion {
            scope: "exact-sentence".to_owned(),
            status: if sentence_hit {
                "LEAKED_ROW_HIT"
            } else {
                "EXCLUDED_EXACT_MISS"
            }
            .to_owned(),
            candidate_entries_contributed: sentence_entries,
        },
        A10LeakageExclusion {
            scope: "exact-paragraph".to_owned(),
            status: "EXCLUDED_NO_CANDIDATE_INDEX".to_owned(),
            candidate_entries_contributed: 0,
        },
        A10LeakageExclusion {
            scope: "exact-conversation".to_owned(),
            status: "EXCLUDED_NO_CANDIDATE_INDEX".to_owned(),
            candidate_entries_contributed: 0,
        },
        A10LeakageExclusion {
            scope: "exact-global".to_owned(),
            status: "EXCLUDED_NO_CANDIDATE_INDEX".to_owned(),
            candidate_entries_contributed: 0,
        },
    ]
}

fn source_counts(counts: AttentionSourceCounts) -> A10SourceCounts {
    A10SourceCounts {
        last_one: counts.last_one,
        last_two: counts.last_two,
        ordered_sentence: counts.ordered_sentence,
        divisor: counts.divisor,
        adjacent_spin: counts.adjacent_spin,
    }
}

fn contributing_sources(counts: A10SourceCounts) -> Vec<String> {
    [
        ("last-one", counts.last_one),
        ("last-two", counts.last_two),
        ("ordered-sentence", counts.ordered_sentence),
        ("divisor", counts.divisor),
        ("adjacent-spin", counts.adjacent_spin),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(source, _)| source.to_owned())
    .collect()
}

fn address_value(
    artifact: &CanonicalRouteArtifact,
    child_manifest_addresses: &[GeometricAddress],
    address: &GeometricAddress,
) -> Result<A10AddressValue, RecursiveGeometricAttentionError> {
    let value = artifact
        .lexical_route_value_for_address_from_validated_artifact(address)?
        .ok_or_else(|| {
            RecursiveGeometricAttentionError::InvalidProbe(
                "admitted candidate cannot resolve to an exact lexical value".to_owned(),
            )
        })?;
    let address_kappa = address.canonical_kappa()?;
    if value.address_kappa != address_kappa
        || value.prime != address.atom.value()
        || value.payload_cid != address.payload_cid
    {
        return Err(RecursiveGeometricAttentionError::InvalidProbe(
            "selected address and exact lexical value view disagree".to_owned(),
        ));
    }
    let child_manifest_address_index =
        child_manifest_addresses
            .binary_search(address)
            .map_err(|_| {
                RecursiveGeometricAttentionError::InvalidProbe(
                    "admitted candidate is absent from the frozen child manifest address vector"
                        .to_owned(),
                )
            })?;
    Ok(A10AddressValue {
        lexical_unit_id: value.lexical_unit_id,
        registry_address_index: value.registry_address_index,
        child_manifest_address_index: u16::try_from(child_manifest_address_index).map_err(
            |_| {
                RecursiveGeometricAttentionError::InvalidProbe(
                    "child manifest address index exceeds u16".to_owned(),
                )
            },
        )?,
        address_kappa,
        prime: value.prime,
        payload_cid: value.payload_cid,
        payload_bytes: value.payload_bytes,
        s3_spin_q30: address.spin.s3.raw(),
        hopf_observation_q30: address.spin.hopf.raw(),
        fiber_q29: address.spin.fiber.raw(),
        torsion_q29: address.spin.torsion.raw(),
        radial_zphi: [address.radial.a, address.radial.b],
    })
}

fn incremental_next_state(
    attention: &GeometricAttentionArtifact,
    initial: &CausalAttentionState,
    history: &[GeometricAddress],
    target: &GeometricAddress,
    artifact: &CanonicalRouteArtifact,
    child_manifest_addresses: &[GeometricAddress],
) -> Result<A10IncrementalNextState, RecursiveGeometricAttentionError> {
    let target_value = address_value(artifact, child_manifest_addresses, target)?;
    let mut incremental = initial.clone();
    attention.observe(&mut incremental, target.clone())?;
    let mut extended = history.to_vec();
    extended.push(target.clone());
    let rebuilt = attention.causal_state_from_history(&extended)?;
    let incremental_previous_address_kappa = incremental
        .previous()
        .map(GeometricAddress::canonical_kappa)
        .transpose()?;
    let rebuilt_previous_address_kappa = rebuilt
        .previous()
        .map(GeometricAddress::canonical_kappa)
        .transpose()?;
    let incremental_last_address_kappa = incremental.last().canonical_kappa()?;
    let rebuilt_last_address_kappa = rebuilt.last().canonical_kappa()?;
    let incremental_ordered_sentence_kappa = incremental.sentence_key()?.as_str().to_owned();
    let rebuilt_ordered_sentence_kappa = rebuilt.sentence_key()?.as_str().to_owned();
    let exact_reproduction = incremental_previous_address_kappa == rebuilt_previous_address_kappa
        && incremental_last_address_kappa == rebuilt_last_address_kappa
        && incremental_ordered_sentence_kappa == rebuilt_ordered_sentence_kappa
        && incremental.observed_routes() == rebuilt.observed_routes();
    Ok(A10IncrementalNextState {
        target_address_kappa: target_value.address_kappa,
        target_payload_bytes: target_value.payload_bytes,
        incremental_previous_address_kappa,
        rebuilt_previous_address_kappa,
        incremental_last_address_kappa,
        rebuilt_last_address_kappa,
        incremental_ordered_sentence_kappa,
        rebuilt_ordered_sentence_kappa,
        incremental_observed_routes: incremental.observed_routes(),
        rebuilt_observed_routes: rebuilt.observed_routes(),
        exact_reproduction,
    })
}

fn candidate_support_signature(path: &A10CandidatePath) -> Vec<(String, A10SourceCounts)> {
    let mut signature = path
        .candidates
        .iter()
        .map(|candidate| {
            (
                candidate.address_value.address_kappa.clone(),
                candidate.source_counts,
            )
        })
        .collect::<Vec<_>>();
    signature.sort();
    signature
}

fn a1r_candidate_origin_signature(
    path: &A10CandidatePath,
) -> Vec<(String, String, Vec<u8>, A10SourceCounts, Vec<String>)> {
    let mut signature = path
        .candidates
        .iter()
        .map(|candidate| {
            (
                candidate.address_value.address_kappa.clone(),
                candidate.address_value.payload_cid.clone(),
                candidate.address_value.payload_bytes.clone(),
                candidate.source_counts,
                candidate.contributing_sources.clone(),
            )
        })
        .collect::<Vec<_>>();
    signature.sort();
    signature
}

fn a1r_row_source_outcome_signature(path: &A10CandidatePath) -> Vec<(String, String, bool, usize)> {
    path.rows
        .iter()
        .map(|row| {
            (
                row.source.clone(),
                row.key_kind.clone(),
                row.hit,
                row.candidate_entries_examined,
            )
        })
        .collect()
}

fn a1r_path_budget_signature(
    path: &A10CandidatePath,
) -> (usize, usize, usize, usize, usize, usize, bool, bool) {
    (
        path.rows.len(),
        path.candidate_entries_examined,
        path.candidate_entry_ceiling,
        path.unique_candidates_before_admission,
        path.unique_candidates_after_admission,
        path.retained_candidate_ceiling,
        path.admission_truncated_union,
        path.full_pre_admission_union_observed,
    )
}

fn a1r_candidate_path_contract_exact(path: &A10CandidatePath) -> bool {
    let source_count = |source: &str| path.rows.iter().filter(|row| row.source == source).count();
    let divisor_rows_miss = path
        .rows
        .iter()
        .filter(|row| row.source == "divisor")
        .all(|row| !row.hit);
    let candidates_are_exact_adjacent_spin_pair = path.candidates.len() == 2
        && path
            .candidates
            .iter()
            .map(|candidate| candidate.address_value.payload_bytes.as_slice())
            .collect::<BTreeSet<_>>()
            == BTreeSet::from([b"ll".as_slice(), b"rr".as_slice()])
        && path.candidates.iter().all(|candidate| {
            candidate.source_counts.last_one == 0
                && candidate.source_counts.last_two == 0
                && candidate.source_counts.ordered_sentence == 0
                && candidate.source_counts.divisor == 0
                && candidate.source_counts.adjacent_spin > 0
                && candidate.contributing_sources.len() == 1
                && candidate.contributing_sources.first().map(String::as_str)
                    == Some("adjacent-spin")
        });
    path.rows.len() == ROWS_PER_QUERY_CEILING
        && source_count("last-one") == 1
        && source_count("last-two") == 1
        && source_count("ordered-sentence") == 1
        && source_count("divisor") == 1
        && source_count("adjacent-spin") == 3
        && path.candidate_entries_examined == 2
        && path.candidate_entry_ceiling == ROWS_PER_QUERY_CEILING.saturating_mul(CANDIDATE_CEILING)
        && path.unique_candidates_before_admission == 2
        && path.unique_candidates_after_admission == 2
        && path.retained_candidate_ceiling == CANDIDATE_CEILING
        && !path.admission_truncated_union
        && path.full_pre_admission_union_observed
        && path.intended_target_pre_admission_reachable == Some(true)
        && path.intended_target_post_admission_reachable
        && path.intended_target_truncated_before_geometry == Some(false)
        && path.exact_direct_rows_miss
        && divisor_rows_miss
        && candidates_are_exact_adjacent_spin_pair
        && path.exclusions.iter().all(|exclusion| {
            exclusion.status.starts_with("EXCLUDED_")
                && exclusion.candidate_entries_contributed == 0
        })
        && !path.target_injected
        && !path.future_events_visible
        && path
            .incremental_next_state
            .as_ref()
            .is_some_and(|state| state.exact_reproduction)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, RecursiveGeometricAttentionError> {
    serde_json::to_vec(value)
        .map_err(|error| RecursiveGeometricAttentionError::Serialization(error.to_string()))
}

#[derive(Serialize)]
struct ByteIdentityWire {
    schema: u32,
    domain: &'static str,
    bytes_hex: String,
}

#[derive(Serialize)]
struct ReportIdentityWire<'a> {
    schema: u32,
    domain: &'static str,
    report_kappa: &'static str,
    body: &'a A10OrderedStateProbeBody,
}

#[derive(Serialize)]
struct A1RReportIdentityWire<'a> {
    schema: u32,
    domain: &'static str,
    report_kappa: &'static str,
    body: &'a A1RAssociativeOrderedSummaryProbeBody,
}

#[derive(Serialize)]
struct FrozenPartitionIdentityWire<'a> {
    schema: u32,
    domain: &'static str,
    partition: &'a A10FrozenPartition,
}

fn frozen_partition_kappa(
    partition: &A10FrozenPartition,
) -> Result<String, RecursiveGeometricAttentionError> {
    canonical_kappa(&canonical_json(&FrozenPartitionIdentityWire {
        schema: A1_0_ORDERED_STATE_PROBE_SCHEMA,
        domain: "uor-r4.a1-frozen-registration-construction-evaluation-partition/1",
        partition,
    })?)
}

fn report_identity_kappa(
    body: &A10OrderedStateProbeBody,
) -> Result<String, RecursiveGeometricAttentionError> {
    canonical_kappa(&canonical_json(&ReportIdentityWire {
        schema: A1_0_ORDERED_STATE_PROBE_SCHEMA,
        domain: A1_0_ORDERED_STATE_PROBE_DOMAIN,
        report_kappa: "",
        body,
    })?)
}

fn a1r_report_identity_kappa(
    body: &A1RAssociativeOrderedSummaryProbeBody,
) -> Result<String, RecursiveGeometricAttentionError> {
    canonical_kappa(&canonical_json(&A1RReportIdentityWire {
        schema: A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_SCHEMA,
        domain: A1R_ASSOCIATIVE_ORDERED_SUMMARY_PROBE_DOMAIN,
        report_kappa: "",
        body,
    })?)
}

fn canonical_kappa(bytes: &[u8]) -> Result<String, RecursiveGeometricAttentionError> {
    let identity = canonical_json(&ByteIdentityWire {
        schema: 1,
        domain: "uor-r4.canonical-byte-identity/1",
        bytes_hex: hex::encode(bytes),
    })?;
    uor_addr::json::address_blake3(&identity)
        .map(|outcome| outcome.address.to_string())
        .map_err(|error| RecursiveGeometricAttentionError::Addressing(format!("{error:?}")))
}
