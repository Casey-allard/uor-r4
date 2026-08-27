//! A1.0 ordered-state and value-reachability probe for recursive attention.
//!
//! This module intentionally stops before implementing an attention scorer.
//! It freezes a registration-only vocabulary, a construction partition, and
//! three evaluation contrasts before compiling any route rows or hierarchy
//! summaries. The evaluation contrasts differ only in earlier order. Candidate
//! support comes from the existing bounded [`GeometricAttentionArtifact`]
//! child-manifest path; no target is injected into a query.

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, AttentionLevelTrace,
    CanonicalLexicalCodec, CanonicalLexicalError, CanonicalRouteArtifact, ConversationInput,
    ParagraphInput, TurnInput,
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

const ISSUE_URL: &str = "https://github.com/UOR-Foundation/uor-r4/issues/952";
const S0_CONSUMER_CONTRACT_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434217921";
const A1_DECISION_CONTRACT_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434437267";
const A1_FROZEN_FIXTURE_URL: &str =
    "https://github.com/UOR-Foundation/uor-r4/issues/952#issuecomment-5434478600";
const FROZEN_S0_ARTIFACT_KAPPA: &str =
    "blake3:3f2043e15a32f6ef799c0073d0c714e3140449591b7d8a18069e39c5182662bd";

const CANDIDATE_CEILING: usize = 8;
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
        .lexical_route_address(target_unit)?
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
        .lexical_route_address(lexical_unit_id(codec, token)?)?
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
        .lexical_route_value_for_address(address)?
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
