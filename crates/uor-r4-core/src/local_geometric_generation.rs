//! Bounded, source-free lexical generation over the #969 causal R4/S3 selector.
//!
//! This module is deliberately a thin decoded loop. It reconstructs the
//! canonical lexical codec from a validated schema-1 route artifact, reuses
//! that artifact's natural schema-2 admission rows, and delegates selection
//! unchanged to [`GeometricAttentionArtifact::select_path_or_abstain`]. It
//! introduces no provider, learned weight, target row, hierarchy lookup, or
//! semantic scoring layer.

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec, CanonicalLexicalError,
    CanonicalRouteArtifact, EncodedLexicalUnit, H4BinaryIcosahedralClosure, LexicalRouteValueView,
};
use crate::prime_route_attention::{GeometricAddress, PrimeRouteError};
use crate::prime_route_geometric_attention::{
    AttentionRowKey, AttentionRowRead, AttentionRowSource, AttentionSourceCounts,
    AttentionSupportAdmission, GeometricAttentionArtifact, GeometricAttentionError,
    PathLeaseAttentionTrace, PathLeaseControl, PathLeaseCost, LOCAL_PATH_ATTENTION_MAX_UNITS,
};

pub const LOCAL_GEOMETRIC_GENERATION_REPORT_SCHEMA: u32 = 1;
pub const LOCAL_GEOMETRIC_GENERATION_REPORT_DOMAIN: &str =
    "uor-r4.local-geometric-generation-report/1";
pub const LOCAL_GEOMETRIC_GENERATION_MIN_PROMPT_UNITS: usize = 2;

#[derive(Debug)]
pub enum LocalGeometricGenerationError {
    Canonical(CanonicalLexicalError),
    Attention(GeometricAttentionError),
    Route(PrimeRouteError),
    Serialization(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for LocalGeometricGenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "canonical lexical boundary: {error}"),
            Self::Attention(error) => write!(formatter, "causal R4/S3 selection: {error}"),
            Self::Route(error) => write!(formatter, "geometric route identity: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "generation report serialization: {error}")
            }
            Self::Invalid(reason) => {
                write!(formatter, "invalid local geometric generation: {reason}")
            }
        }
    }
}

impl std::error::Error for LocalGeometricGenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Canonical(error) => Some(error),
            Self::Attention(error) => Some(error),
            Self::Route(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<CanonicalLexicalError> for LocalGeometricGenerationError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<GeometricAttentionError> for LocalGeometricGenerationError {
    fn from(error: GeometricAttentionError) -> Self {
        Self::Attention(error)
    }
}

impl From<PrimeRouteError> for LocalGeometricGenerationError {
    fn from(error: PrimeRouteError) -> Self {
        Self::Route(error)
    }
}

impl From<serde_json::Error> for LocalGeometricGenerationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

/// The two arms frozen by #953. `LastOnly` remains available on the underlying
/// #969 diagnostic API but is intentionally not carried into this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalGenerationControl {
    FullPath,
    StateDisabled,
}

impl LocalGenerationControl {
    const fn path_control(self) -> PathLeaseControl {
        match self {
            Self::FullPath => PathLeaseControl::FullPath,
            Self::StateDisabled => PathLeaseControl::StateDisabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalGenerationRowSource {
    LastOne,
    LastTwo,
    OrderedSentence,
    Divisor,
    AdjacentSpin,
}

impl From<AttentionRowSource> for LocalGenerationRowSource {
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
pub enum LocalGenerationRowKey {
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
pub struct LocalGenerationRowTrace {
    pub source: LocalGenerationRowSource,
    pub key: LocalGenerationRowKey,
    pub hit: bool,
    pub candidate_entries_examined: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct LocalGenerationSourceCounts {
    pub last_one: u32,
    pub last_two: u32,
    pub ordered_sentence: u32,
    pub divisor: u32,
    pub adjacent_spin: u32,
}

impl From<AttentionSourceCounts> for LocalGenerationSourceCounts {
    fn from(counts: AttentionSourceCounts) -> Self {
        Self {
            last_one: counts.last_one,
            last_two: counts.last_two,
            ordered_sentence: counts.ordered_sentence,
            divisor: counts.divisor,
            adjacent_spin: counts.adjacent_spin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGenerationPromptRouteTrace {
    pub occurrence_index: usize,
    pub lexical_unit_id: u32,
    pub leading_bytes: Vec<u8>,
    pub span_start: u32,
    pub span_end: u32,
    pub prime: u32,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGenerationCandidateTrace {
    pub lexical_unit_id: u32,
    pub prime: u32,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
    pub source_counts: LocalGenerationSourceCounts,
    pub best_prefix_index: u8,
    pub cost: PathLeaseCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGenerationSelectionTrace {
    pub lexical_unit_id: u32,
    pub prime: u32,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
    pub cost: PathLeaseCost,
    pub emitted_boundary_bytes: Vec<u8>,
    pub rendered_bytes: Vec<u8>,
    pub observed_routes_after_append: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGenerationStepTrace {
    pub step_index: usize,
    pub observed_routes_before: usize,
    pub support_admission: String,
    pub support_rows: Vec<LocalGenerationRowTrace>,
    pub candidate_entries_examined: usize,
    pub candidate_entry_ceiling: usize,
    pub unique_candidates_before_ceiling: usize,
    pub candidate_ceiling: usize,
    pub memory_keys_per_candidate: usize,
    pub path_geometry_evaluations: usize,
    pub candidates: Vec<LocalGenerationCandidateTrace>,
    pub minimum_cost: Option<PathLeaseCost>,
    pub tie: bool,
    pub abstained: bool,
    pub selected: Option<LocalGenerationSelectionTrace>,
    pub observed_routes_after: usize,
    pub detected_cycle_period: Option<usize>,
}

/// Auditable closure of the inputs reachable from this generation loop.
/// Artifact reconstruction and schema-2 compilation are disclosed separately;
/// the zero counters are structural properties of selection-time execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGenerationSourceBoundary {
    pub serving_inputs: String,
    pub artifact_provenance_validated: bool,
    pub artifact_input_reconstructed: bool,
    pub schema2_rebuild_witnesses_compiled: bool,
    pub source_weight_reads: usize,
    pub teacher_forwards: usize,
    pub provider_calls: usize,
    pub source_attention_calls: usize,
    pub learned_router_calls: usize,
    pub dense_matrix_operations: usize,
    pub selection_future_event_reads: usize,
    pub selection_paragraph_conversation_global_reads: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LocalGenerationStopReason {
    Abstained { tie: bool },
    TerminalPunctuation,
    ShortCycle { period: usize },
    ContinuationCap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalGeometricGenerationReport {
    pub schema: u32,
    pub domain: String,
    pub issue: u32,
    pub artifact_manifest_kappa: String,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub attention_manifest_kappa: String,
    pub h4_root_table_kappa: String,
    pub h4_multiplication_table_kappa: String,
    pub source_boundary: LocalGenerationSourceBoundary,
    pub control: LocalGenerationControl,
    pub prompt_bytes: Vec<u8>,
    pub prompt_routes: Vec<LocalGenerationPromptRouteTrace>,
    pub prompt_trailing_bytes: Vec<u8>,
    pub continuation_cap: usize,
    pub steps: Vec<LocalGenerationStepTrace>,
    pub emitted_lexical_unit_ids: Vec<u32>,
    pub emitted_address_kappas: Vec<String>,
    /// Exact bytes to append to an accepted `prompt_bytes`; prompts with
    /// trailing whitespace fail closed before selection.
    pub continuation_bytes: Vec<u8>,
    pub detected_cycle_period: Option<usize>,
    pub stop_reason: LocalGenerationStopReason,
}

impl LocalGeometricGenerationReport {
    /// Deterministic JSON bytes. The report contains only ordered structures,
    /// fixed-field records, and exact integer/byte/string values.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, LocalGeometricGenerationError> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn report_kappa(&self) -> Result<String, LocalGeometricGenerationError> {
        Ok(format!(
            "blake3:{}",
            blake3::hash(&self.canonical_bytes()?).to_hex()
        ))
    }
}

/// Immutable construction bundle for the bounded decoded loop.
pub struct LocalGeometricGenerator {
    artifact: CanonicalRouteArtifact,
    codec: CanonicalLexicalCodec,
    attention: GeometricAttentionArtifact,
    h4_table: H4BinaryIcosahedralClosure,
}

impl LocalGeometricGenerator {
    /// Decode a canonical schema-1 artifact and reconstruct every serving
    /// dependency from its own provider-free bytes. Codec reconstruction is
    /// accepted only when both stored identities reproduce exactly.
    pub fn from_canonical_bytes(
        artifact_bytes: &[u8],
    ) -> Result<Self, LocalGeometricGenerationError> {
        let artifact = CanonicalRouteArtifact::decode_canonical(artifact_bytes)?;
        let reconstructed_input = artifact.reconstruct_input()?;
        let codec = CanonicalLexicalCodec::compile(&reconstructed_input)?;
        if codec.codec_kappa() != artifact.codec_kappa()
            || codec.vocabulary_kappa() != artifact.vocabulary_kappa()
        {
            return Err(LocalGeometricGenerationError::Invalid(
                "reconstructed codec/vocabulary identities do not match the artifact; the artifact does not carry a complete reconstructible registration input"
                    .to_owned(),
            ));
        }
        let manifest = artifact.embedded_spin_manifest()?;
        let attention = GeometricAttentionArtifact::compile_from_manifest_witnesses(&manifest)?;
        if attention.manifest_kappa() != artifact.embedded_spin_manifest_kappa() {
            return Err(LocalGeometricGenerationError::Invalid(
                "compiled attention identity does not match the embedded schema-2 manifest"
                    .to_owned(),
            ));
        }
        let h4_table = validate_h4_binary_icosahedral_closure()?;
        Ok(Self {
            artifact,
            codec,
            attention,
            h4_table,
        })
    }

    pub fn artifact_manifest_kappa(&self) -> &str {
        self.artifact.manifest_kappa()
    }

    pub fn attention_manifest_kappa(&self) -> &str {
        self.attention.manifest_kappa()
    }

    /// Generate at most `continuation_cap` lexical units. The prompt must
    /// contain at least two units and prompt plus cap must fit #969's eight-unit
    /// causal-state bound.
    pub fn generate(
        &self,
        prompt: &[u8],
        control: LocalGenerationControl,
        continuation_cap: usize,
    ) -> Result<LocalGeometricGenerationReport, LocalGeometricGenerationError> {
        if continuation_cap == 0 {
            return Err(LocalGeometricGenerationError::Invalid(
                "continuation cap must be at least one lexical unit".to_owned(),
            ));
        }
        let encoded = self.codec.encode(0, 0, prompt)?;
        if self.codec.decode(&encoded)? != prompt {
            return Err(LocalGeometricGenerationError::Invalid(
                "prompt bytes do not round-trip through the reconstructed canonical codec"
                    .to_owned(),
            ));
        }
        if !encoded.trailing_bytes.is_empty() {
            return Err(LocalGeometricGenerationError::Invalid(
                "prompt trailing whitespace is unsupported because appendable closing punctuation must attach to the final lexical unit"
                    .to_owned(),
            ));
        }
        let prompt_units = encoded.units.len();
        if prompt_units < LOCAL_GEOMETRIC_GENERATION_MIN_PROMPT_UNITS {
            return Err(LocalGeometricGenerationError::Invalid(format!(
                "prompt requires at least {LOCAL_GEOMETRIC_GENERATION_MIN_PROMPT_UNITS} canonical lexical units"
            )));
        }
        let total_bound = prompt_units.checked_add(continuation_cap).ok_or_else(|| {
            LocalGeometricGenerationError::Invalid(
                "prompt plus continuation cap overflows the local unit bound".to_owned(),
            )
        })?;
        if total_bound > LOCAL_PATH_ATTENTION_MAX_UNITS {
            return Err(LocalGeometricGenerationError::Invalid(format!(
                "prompt ({prompt_units}) plus continuation cap ({continuation_cap}) exceeds the {LOCAL_PATH_ATTENTION_MAX_UNITS}-unit local path bound"
            )));
        }

        let mut history = Vec::with_capacity(prompt_units);
        let mut prompt_routes = Vec::with_capacity(prompt_units);
        for (occurrence_index, unit) in encoded.units.iter().enumerate() {
            let address = self.address_for_unit(unit.unit_id)?;
            let value = self.invert_address(&address)?;
            verify_unit_binding(unit.unit_id, &address, &value)?;
            prompt_routes.push(prompt_route_trace(occurrence_index, unit, &address, value)?);
            history.push(address);
        }

        let last_payload = prompt_routes
            .last()
            .map(|route| route.payload_bytes.as_slice())
            .ok_or_else(|| {
                LocalGeometricGenerationError::Invalid(
                    "prompt route trace is unexpectedly empty".to_owned(),
                )
            })?;
        let prior_boundary_class = classify_boundary(last_payload)?;
        let mut renderer = BoundaryRenderer {
            previous: Some(prior_boundary_class),
        };
        let mut state = self
            .attention
            .causal_path_state_from_history(&history, &self.h4_table)?;
        let mut steps = Vec::with_capacity(continuation_cap);
        let mut emitted_lexical_unit_ids = Vec::with_capacity(continuation_cap);
        let mut emitted_address_kappas = Vec::with_capacity(continuation_cap);
        let mut continuation_bytes = Vec::new();
        let mut detected_cycle_period = None;
        let mut stop_reason = LocalGenerationStopReason::ContinuationCap;

        for step_index in 0..continuation_cap {
            let observed_routes_before = state.observed_routes();
            let path_trace = self.attention.select_path_or_abstain(
                &state,
                &self.h4_table,
                control.path_control(),
            )?;
            let support_rows = path_trace
                .support
                .rows_read
                .iter()
                .map(row_trace)
                .collect::<Result<Vec<_>, LocalGeometricGenerationError>>()?;
            let candidates = path_trace
                .candidates
                .iter()
                .map(|candidate| {
                    let value = self.invert_address(&candidate.next)?;
                    verify_unit_binding(value.lexical_unit_id, &candidate.next, &value)?;
                    Ok(LocalGenerationCandidateTrace {
                        lexical_unit_id: value.lexical_unit_id,
                        prime: value.prime,
                        address_kappa: value.address_kappa,
                        payload_cid: value.payload_cid,
                        payload_bytes: value.payload_bytes,
                        source_counts: candidate.source_counts.into(),
                        best_prefix_index: candidate.best_prefix_index,
                        cost: candidate.cost,
                    })
                })
                .collect::<Result<Vec<_>, LocalGeometricGenerationError>>()?;

            let candidate_entries_examined = path_trace.support.candidate_entries_examined;
            let candidate_entry_ceiling = path_trace.support.candidate_entry_ceiling;
            let unique_candidates_before_ceiling =
                path_trace.support.unique_candidates_before_ceiling;
            let candidate_ceiling = path_trace.support.candidate_ceiling;
            let memory_keys_per_candidate = path_trace.memory_keys_per_candidate;
            let path_geometry_evaluations = path_trace.path_geometry_evaluations;
            let minimum_cost = path_trace.minimum_cost;
            let tie = path_trace.tie;
            let abstained = path_trace.abstained;

            let mut selected_trace = None;
            let mut observed_routes_after = observed_routes_before;
            let mut step_cycle_period = None;
            let mut terminal_punctuation = false;
            if let Some(selected) = path_trace.selected.as_ref() {
                let value = self.invert_address(&selected.next)?;
                verify_unit_binding(value.lexical_unit_id, &selected.next, &value)?;
                let boundary_class = classify_boundary(&value.payload_bytes)?;
                let emitted_boundary_bytes = renderer.boundary_before(boundary_class);
                let mut rendered_bytes = emitted_boundary_bytes.clone();
                rendered_bytes.extend_from_slice(&value.payload_bytes);
                let selected_address_kappa = value.address_kappa.clone();
                let selected_unit_id = value.lexical_unit_id;

                // State changes only after admission, exact inversion, and
                // boundary rendering have all succeeded.
                self.attention
                    .observe_path(&mut state, selected.next.clone(), &self.h4_table)?;
                observed_routes_after = state.observed_routes();
                emitted_lexical_unit_ids.push(selected_unit_id);
                emitted_address_kappas.push(selected_address_kappa.clone());
                continuation_bytes.extend_from_slice(&rendered_bytes);
                renderer.observe(boundary_class);

                step_cycle_period = short_cycle_period(&emitted_lexical_unit_ids);
                detected_cycle_period = detected_cycle_period.or(step_cycle_period);
                terminal_punctuation = boundary_class.is_terminal_punctuation();
                selected_trace = Some(LocalGenerationSelectionTrace {
                    lexical_unit_id: selected_unit_id,
                    prime: value.prime,
                    address_kappa: selected_address_kappa,
                    payload_cid: value.payload_cid,
                    payload_bytes: value.payload_bytes,
                    cost: selected.cost,
                    emitted_boundary_bytes,
                    rendered_bytes,
                    observed_routes_after_append: observed_routes_after,
                });
            }

            steps.push(step_trace(
                step_index,
                observed_routes_before,
                &path_trace,
                support_rows,
                candidate_entries_examined,
                candidate_entry_ceiling,
                unique_candidates_before_ceiling,
                candidate_ceiling,
                memory_keys_per_candidate,
                path_geometry_evaluations,
                candidates,
                minimum_cost,
                tie,
                abstained,
                selected_trace,
                observed_routes_after,
                step_cycle_period,
            ));

            if abstained {
                stop_reason = LocalGenerationStopReason::Abstained { tie };
                break;
            }
            if terminal_punctuation {
                stop_reason = LocalGenerationStopReason::TerminalPunctuation;
                break;
            }
            if let Some(period) = step_cycle_period {
                stop_reason = LocalGenerationStopReason::ShortCycle { period };
                break;
            }
        }

        Ok(LocalGeometricGenerationReport {
            schema: LOCAL_GEOMETRIC_GENERATION_REPORT_SCHEMA,
            domain: LOCAL_GEOMETRIC_GENERATION_REPORT_DOMAIN.to_owned(),
            issue: 953,
            artifact_manifest_kappa: self.artifact.manifest_kappa().to_owned(),
            codec_kappa: self.codec.codec_kappa().to_owned(),
            vocabulary_kappa: self.codec.vocabulary_kappa().to_owned(),
            attention_manifest_kappa: self.attention.manifest_kappa().to_owned(),
            h4_root_table_kappa: self.h4_table.h4_root_table_kappa.clone(),
            h4_multiplication_table_kappa: self.h4_table.multiplication_table_kappa.clone(),
            source_boundary: LocalGenerationSourceBoundary {
                serving_inputs:
                    "validated canonical route artifact, embedded schema-2 manifest, fixed exact H4 table, and prompt bytes"
                        .to_owned(),
                artifact_provenance_validated: true,
                artifact_input_reconstructed: true,
                schema2_rebuild_witnesses_compiled: true,
                source_weight_reads: 0,
                teacher_forwards: 0,
                provider_calls: 0,
                source_attention_calls: 0,
                learned_router_calls: 0,
                dense_matrix_operations: 0,
                selection_future_event_reads: 0,
                selection_paragraph_conversation_global_reads: 0,
            },
            control,
            prompt_bytes: prompt.to_vec(),
            prompt_routes,
            prompt_trailing_bytes: encoded.trailing_bytes,
            continuation_cap,
            steps,
            emitted_lexical_unit_ids,
            emitted_address_kappas,
            continuation_bytes,
            detected_cycle_period,
            stop_reason,
        })
    }

    fn address_for_unit(
        &self,
        lexical_unit_id: u32,
    ) -> Result<GeometricAddress, LocalGeometricGenerationError> {
        self.artifact
            .lexical_route_address_from_validated_artifact(lexical_unit_id)?
            .ok_or_else(|| {
                LocalGeometricGenerationError::Invalid(format!(
                    "canonical lexical unit {lexical_unit_id} has no registered route address"
                ))
            })
    }

    fn invert_address(
        &self,
        address: &GeometricAddress,
    ) -> Result<LexicalRouteValueView, LocalGeometricGenerationError> {
        self.artifact
            .lexical_route_value_for_address_from_validated_artifact(address)?
            .ok_or_else(|| {
                LocalGeometricGenerationError::Invalid(
                    "naturally admitted geometric address has no exact lexical inverse".to_owned(),
                )
            })
    }
}

fn verify_unit_binding(
    lexical_unit_id: u32,
    address: &GeometricAddress,
    value: &LexicalRouteValueView,
) -> Result<(), LocalGeometricGenerationError> {
    let address_kappa = address.canonical_kappa()?;
    if value.lexical_unit_id != lexical_unit_id
        || value.prime != address.atom.value()
        || value.payload_cid != address.payload_cid
        || value.address_kappa != address_kappa
    {
        return Err(LocalGeometricGenerationError::Invalid(
            "route-to-payload inversion did not reproduce the selected exact address binding"
                .to_owned(),
        ));
    }
    Ok(())
}

fn prompt_route_trace(
    occurrence_index: usize,
    unit: &EncodedLexicalUnit,
    address: &GeometricAddress,
    value: LexicalRouteValueView,
) -> Result<LocalGenerationPromptRouteTrace, LocalGeometricGenerationError> {
    Ok(LocalGenerationPromptRouteTrace {
        occurrence_index,
        lexical_unit_id: unit.unit_id,
        leading_bytes: unit.leading_bytes.clone(),
        span_start: unit.span_start,
        span_end: unit.span_end,
        prime: value.prime,
        address_kappa: address.canonical_kappa()?,
        payload_cid: value.payload_cid,
        payload_bytes: value.payload_bytes,
    })
}

fn row_trace(
    row: &AttentionRowRead,
) -> Result<LocalGenerationRowTrace, LocalGeometricGenerationError> {
    let key = match &row.key {
        AttentionRowKey::LastOne(address) => LocalGenerationRowKey::LastOne {
            address_kappa: address.canonical_kappa()?,
        },
        AttentionRowKey::LastTwo { previous, last } => LocalGenerationRowKey::LastTwo {
            previous_address_kappa: previous.canonical_kappa()?,
            last_address_kappa: last.canonical_kappa()?,
        },
        AttentionRowKey::LastTwoUnavailable => LocalGenerationRowKey::LastTwoUnavailable,
        AttentionRowKey::OrderedSentence(route_kappa) => LocalGenerationRowKey::OrderedSentence {
            route_kappa: route_kappa.clone(),
        },
        AttentionRowKey::Divisor(atom) => LocalGenerationRowKey::Divisor {
            prime: atom.value(),
        },
        AttentionRowKey::AdjacentSpin(sector) => LocalGenerationRowKey::AdjacentSpin {
            hopf_octant: sector.hopf_octant,
            torsion_bin: sector.torsion_bin,
        },
    };
    Ok(LocalGenerationRowTrace {
        source: row.source.into(),
        key,
        hit: row.hit,
        candidate_entries_examined: row.candidate_entries_examined,
    })
}

#[allow(clippy::too_many_arguments)]
fn step_trace(
    step_index: usize,
    observed_routes_before: usize,
    path_trace: &PathLeaseAttentionTrace,
    support_rows: Vec<LocalGenerationRowTrace>,
    candidate_entries_examined: usize,
    candidate_entry_ceiling: usize,
    unique_candidates_before_ceiling: usize,
    candidate_ceiling: usize,
    memory_keys_per_candidate: usize,
    path_geometry_evaluations: usize,
    candidates: Vec<LocalGenerationCandidateTrace>,
    minimum_cost: Option<PathLeaseCost>,
    tie: bool,
    abstained: bool,
    selected: Option<LocalGenerationSelectionTrace>,
    observed_routes_after: usize,
    detected_cycle_period: Option<usize>,
) -> LocalGenerationStepTrace {
    debug_assert_eq!(path_trace.observed_routes as usize, observed_routes_before);
    LocalGenerationStepTrace {
        step_index,
        observed_routes_before,
        support_admission: support_admission_contract(path_trace.support.support_admission),
        support_rows,
        candidate_entries_examined,
        candidate_entry_ceiling,
        unique_candidates_before_ceiling,
        candidate_ceiling,
        memory_keys_per_candidate,
        path_geometry_evaluations,
        candidates,
        minimum_cost,
        tie,
        abstained,
        selected,
        observed_routes_after,
        detected_cycle_period,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryClass {
    WordRun,
    OpeningPunctuation,
    ClosingPunctuation { terminal: bool },
}

impl BoundaryClass {
    const fn is_terminal_punctuation(self) -> bool {
        matches!(self, Self::ClosingPunctuation { terminal: true })
    }
}

fn classify_boundary(payload: &[u8]) -> Result<BoundaryClass, LocalGeometricGenerationError> {
    let surface = std::str::from_utf8(payload).map_err(|error| {
        LocalGeometricGenerationError::Invalid(format!(
            "lexical payload is not valid UTF-8 at the rendering boundary: {error}"
        ))
    })?;
    if !surface.is_empty()
        && surface
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
    {
        return Ok(BoundaryClass::WordRun);
    }
    let mut characters = surface.chars();
    let character = characters.next().ok_or_else(|| {
        LocalGeometricGenerationError::Invalid(
            "empty lexical payload has no deterministic boundary class".to_owned(),
        )
    })?;
    if characters.next().is_some() {
        return Err(LocalGeometricGenerationError::Invalid(
            "non-word lexical payload contains multiple scalars and has no deterministic boundary class"
                .to_owned(),
        ));
    }
    match character {
        '(' | '[' | '{' => Ok(BoundaryClass::OpeningPunctuation),
        '.' | '!' | '?' => Ok(BoundaryClass::ClosingPunctuation { terminal: true }),
        ',' | ';' | ':' | ')' | ']' | '}' => {
            Ok(BoundaryClass::ClosingPunctuation { terminal: false })
        }
        _ => Err(LocalGeometricGenerationError::Invalid(format!(
            "unsupported or ambiguous lexical boundary scalar U+{:04X}",
            u32::from(character)
        ))),
    }
}

struct BoundaryRenderer {
    previous: Option<BoundaryClass>,
}

impl BoundaryRenderer {
    fn boundary_before(&self, current: BoundaryClass) -> Vec<u8> {
        if self.previous.is_none()
            || matches!(current, BoundaryClass::ClosingPunctuation { .. })
            || matches!(self.previous, Some(BoundaryClass::OpeningPunctuation))
        {
            Vec::new()
        } else {
            vec![b' ']
        }
    }

    fn observe(&mut self, emitted: BoundaryClass) {
        self.previous = Some(emitted);
    }
}

fn support_admission_contract(admission: AttentionSupportAdmission) -> String {
    match admission {
        AttentionSupportAdmission::SourceBreadthThenTotalCountThenCanonicalAddress => {
            "source-breadth-then-total-count-then-canonical-address".to_owned()
        }
    }
}

/// Match the existing project gate: a period of one through four units is a
/// short cycle only after three identical trailing periods.
fn short_cycle_period(units: &[u32]) -> Option<usize> {
    for period in 1..=4 {
        let span = period * 3;
        if units.len() < span {
            continue;
        }
        let tail = &units[units.len() - span..];
        if tail[..period] == tail[period..period * 2] && tail[..period] == tail[period * 2..] {
            return Some(period);
        }
    }
    None
}
