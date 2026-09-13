//! Native count-fitted geometric language prototype.
//!
//! Learned, quantized conditional-score tables read an ordered H4 window,
//! prime-derived fixed-zeta phases and exact signed-root orientation. This is
//! an experimental finite-state language model, not an established general
//! reasoner. It has no neural matrix products or external-model fallback.
//! `runtime::{Session::observe, Session::predict}` is the integer/table kernel;
//! tokenization, serialization, fitting and diagnostic rendering are host work.

pub use relation::RelationWork;
pub use relation_admission::RelationAdmissionMode;
pub use relation_training::{RelationExample, RelationLabel};
mod dependent_read;
mod dependent_read_training;
pub use dependent_read_training::DependentReadExample;
#[cfg(test)]
mod dependent_read_tests;
#[cfg(test)]
mod general_prose_tests;
mod joint_admission;
#[cfg(test)]
mod joint_admission_tests;
mod joint_admission_training;
mod literal_refinement;
#[cfg(test)]
mod source_refinement_tests;
mod source_routing;
#[cfg(test)]
mod source_routing_tests;
mod source_routing_training;
mod source_span;
mod source_span_training;
pub use source_routing_training::SourceRoutingConfig;
mod action_binding;
mod action_emission;
#[cfg(test)]
mod action_emission_tests;
mod current_query_handoff;
mod historical_version_intent;
pub use historical_version_intent::HistoricalVersionExample;
mod current_source;
mod historical_field_composition;
pub use current_query_handoff::CurrentQueryExample;
mod historical_query_context;
pub use historical_query_context::HistoricalQueryExample;
mod historical_read;
mod historical_read_training;
pub use historical_read_training::HistoricalReadExample;
mod mixed_initial_training;
mod shared_operator_refinement;
pub use current_source::{CurrentSourceExample, CurrentSourceTarget};
mod writer_choice;
mod writer_lexical;
mod writer_role;
pub use writer_choice::{WriterChoiceExample, WriterChoiceOverride, WriterChoiceTarget};
pub use writer_lexical::WriterLexicalExample;
mod source_role_refinement;
pub use source_role_refinement::{SourceRoleRefinementExample, SourceRoleTarget};
mod field_composition;
mod field_composition_snapshot;
mod word_emission;
pub use action_emission::{ActionEmissionExample, ActionEmissionSegment};
pub use field_composition::{
    FieldAnchor, FieldCompositionExample, FieldDecision, FieldPiece, FieldRead,
};
pub use word_emission::WordEmissionExample;
mod mixed_operators;
pub use mixed_operators::{MixedOperatorExample, MixedOperatorTarget};
mod composed_output;
mod composed_output_training;
mod instruction_binding;
mod instruction_binding_training;
pub use instruction_binding_training::InstructionExample;
mod lexical_emission;
#[cfg(test)]
mod lexical_emission_tests;
pub use lexical_emission::{EmissionExample, EmissionPiece};
mod operation_transition;
#[cfg(test)]
mod operation_transition_tests;
pub use operation_transition::OperationTransitionExample;
mod typed_routing;
mod typed_routing_training;
pub use typed_routing_training::{TypedRoutingExample, TypedRoutingTurn};
mod anchors;
mod learned_routing;
#[cfg(test)]
mod learned_routing_tests;
mod learned_routing_training;
mod recurrent_routing;
#[cfg(test)]
mod recurrent_routing_tests;
mod recurrent_routing_training;
pub use learned_routing::{RoutingDecision, RoutingHeadDecision, RoutingMode, RoutingWork};
pub use learned_routing_training::{RoutingFitConfig, RoutingFitReport};
pub use recurrent_routing_training::RecurrentRoutingFitReport;
mod completion_runtime;
mod completion_training;
mod completion_types;
pub mod durable_memory;
#[cfg(test)]
mod durable_memory_tests;
pub mod groundedness;
#[cfg(test)]
mod groundedness_tests;
pub mod guarantees;
#[cfg(test)]
mod guarantees_tests;
pub mod m1_profiler;
#[cfg(test)]
mod m1_profiler_tests;
mod memory_runtime;
mod memory_training;
mod memory_types;
mod mixture;
pub mod multi_step_reasoning;
#[cfg(test)]
mod multi_step_reasoning_tests;
mod numeral;
mod relation;
mod relation_admission;
mod relation_span;
mod relation_start;
mod relation_start_refinement;
pub use relation_start_refinement::{RelationStartExample, RelationStartOverride};
mod relation_start_training;
#[cfg(test)]
mod relation_tests;
mod relation_training;
mod response_entry_runtime;
mod response_entry_training;
mod response_entry_types;
mod response_runtime;
mod role_read;
mod role_read_training;
mod runtime;
mod snapshot;
mod training;
mod value_lexemes;
mod value_runtime;
mod value_training;
mod value_types;
mod word_copy_runtime;
mod word_copy_training;
mod word_copy_types;
pub mod workspace_coding;
#[cfg(test)]
mod workspace_coding_tests;
mod writer_refinement;

use serde::{Deserialize, Serialize};

/// Isolated primitive experiment; no retained dispatch change.
pub mod addressed_attention;
/// Experimental bounded Hamming reads and contextual refinement.
pub mod hamming_policy;
pub mod hamming_refinement;
/// Experimental shared state/access/emission kernel; never an implicit fallback.
pub mod shared_core;

pub use completion_training::{ValueCompletionFitConfig, ValueCompletionFitReport};
pub use completion_types::{
    CompletionAction, CompletionDecision, CompletionStateView, CompletionWork,
};
pub use response_entry_training::{ResponseEntryFitConfig, ResponseEntryFitReport};
pub use response_entry_types::{
    ResponseEntryAction, ResponseEntryDecision, ResponseEntryStateView,
};
pub type ResponseEntryWork = CompletionWork;
pub use durable_memory::{
    DurableConsolidationReport, DurableFactRecord, DurableMemoryStore, DurableSession,
    IdentityScope,
};
pub use groundedness::{
    AbstentionReason, ConflictStatus, GroundedAbstention, GroundedAnswer, GroundedCase,
    GroundedCaseResult, GroundedClarification, GroundedConflict, GroundedOutcome,
    GroundedProvenance, GroundednessEvaluator, GroundednessReport, GROUNDEDNESS_SCHEMA,
};
pub use guarantees::{
    AllowedOp, ArtifactIntegrityWitness, BridgeBoundary, CensusReport, ChartAdapter, ChartKind,
    ChartWitness, ClaimClass, ClaimStatus, CodecRole, DossierAuditReport, EulerHopfBridge,
    ForbiddenOp, FormalClaim, FormalClaimDossier, IcosianQuaternion, PairedH4Icosian,
    ServingOperationCensus, ZPhi,
};
pub use m1_profiler::{
    HardwareMetrics, LatencyDistribution, M1EnergyModel, M1Profiler, StageBreakdown,
    TaskBenchmarkResult, TaskKind,
};
pub use memory_training::{
    MemoryReadDiagnostic, MemoryReadDocumentExposure, MemoryReadDocumentSupervision,
    MemoryReadResponseStateReport, MemoryReadSchedule, MemoryReadStreamProgress,
    MemoryReadStreamReport, MemoryReadSupervision, MemoryReadTokenSpan, MemoryReadTrainer,
};
pub use memory_types::{MemoryReadFitConfig, MemoryReadFitReport, MemoryStateView};
pub use memory_types::{ResponseAction, ResponseDecision, ResponseStateView};
pub use mixture::{ReadoutFitConfig, ReadoutFitReport};
pub use multi_step_reasoning::{
    ConstraintPolicy, MultiStepReasoningEngine, MultiStepReasoningReport, ReasoningChain,
    ReasoningStep, ReasoningStepKind, MULTI_STEP_REASONING_SCHEMA,
};
pub use runtime::{Session, StateView};
pub use training::Trainer;
pub use value_training::{ValueExample, ValueFitConfig, ValueFitReport};
pub use value_types::{
    ValueAction, ValueDecision, ValueDerivation, ValueRecord, ValueStateView, ValueWork,
};
pub use word_copy_training::ResponseEntryCopyFitReport;
pub use word_copy_types::{
    WordCopyAction, WordCopyDecision, WordCopyProgress, WordCopyStateView, WordCopyWork,
};
pub use workspace_coding::{
    CompileReport, CompilerDiagnostic, PatchOperation, RepairIteration, WorkspaceCodingEngine,
    WorkspaceCodingReport, WorkspaceEnvironment, WorkspaceRevision, WORKSPACE_CODING_SCHEMA,
};

pub const SCHEMA: &str = "uor-r4.native-geometric-language/1";
pub const BOS: u32 = 0;
pub const EOS: u32 = 1;
pub const PHASE_CHANNELS: usize = 8;
const LEXICAL_BASE: u32 = 258;
const SCORE_SCALE: f64 = 256.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub context_tokens: usize,
    pub candidate_limit: usize,
    pub max_lexical_pieces: usize,
    pub max_rows: usize,
    pub max_associations: usize,
    pub postings_per_row: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            context_tokens: 128,
            candidate_limit: 32,
            max_lexical_pieces: 4096,
            max_rows: 65_536,
            max_associations: 500_000,
            postings_per_row: 16,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(1..=4096).contains(&self.context_tokens)
            || !(1..=256).contains(&self.candidate_limit)
            || !(1..=65_536).contains(&self.max_lexical_pieces)
            || !(1..=1_000_000).contains(&self.max_rows)
            || !(1..=8_000_000).contains(&self.max_associations)
            || !(1..=256).contains(&self.postings_per_row)
        {
            return Err(Error(
                "native geometric config exceeds supported bounds".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentReceipt {
    pub id: String,
    pub text_cid: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Control {
    /// Diagnostic reachability intervention; the retained default is unchanged.
    CurrentRelationReadAll,
    HistoricalReadDisabled,
    HistoricalFieldCompositionDisabled,
    HistoricalQueryContextDisabled,
    CurrentQueryHandoffDisabled,
    CurrentQueryHandoffTransformDisabled,
    CurrentQueryHandoffScopeDisabled,
    HistoricalVersionIntentDisabled,
    HistoricalVersionIntentTransformDisabled,
    HistoricalVersionIntentScopeDisabled,
    /// Offer only each head's immediate previous record to the learned version selector.
    HistoricalVersionIntentAncestorDisabled,
    /// Withhold truncated-chain abstention candidates from the learned version selector.
    HistoricalVersionIntentAbstainDisabled,
    /// Follow explicit revision links only: withhold same-value reassertion links and
    /// the proven-eviction requirement of the versioned chain contract.
    HistoricalVersionIntentReassertionDisabled,
    /// Withhold the first hop through a same-value reassertion head (the head contract).
    HistoricalVersionIntentReassertionHeadDisabled,
    /// Keep the witness but let the frozen persistent reader scan only the eight most
    /// recent request words for the owner (withhold the reader-window contract).
    HistoricalVersionIntentReaderWindowDisabled,
    /// Keep the request-window scan but withhold the current-turn boundary (reproduces
    /// the post-fact-only scope of f3620cb7).
    HistoricalVersionIntentReaderTurnScopeDisabled,
    /// Keep the active historical router but expose only eight query words.
    HistoricalQueryWindowDisabled,
    RelationStartRefinementDisabled,
    CurrentSourceDisabled,
    CurrentSourceVersionDisabled,
    WriterRoleDisabled,
    WriterRoleContextDisabled,
    WriterChoiceDisabled,
    WriterChoiceBoundaryDisabled,
    FieldCompositionDisabled,
    FieldCompositionContextDisabled,
    FieldCompositionGeometryDisabled,
    FieldCompositionReadDisabled,
    SourceSpanDisabled,
    /// Remove following source identity from the fitted extent operator.
    SourceSpanContextDisabled,
    /// Keep next-word identity but remove its pair with the original source cue.
    SourceSpanPairDisabled,
    /// Disable the nonpositive lexical residual; retain the parent writer/cache.
    WriterLexicalDisabled,
    /// Restore the parent source router while retaining the current context law.
    SourceRoleRefinementDisabled,
    /// Restore the exact source router and feature law preceding context retention.
    SourceContextDisabled,
    /// Keep the fitted router but remove retained predecessors from its features.
    SourceContextWindowOnly,
    JointAdmissionDisabled,
    LiteralRefinementDisabled,
    InstructionBindingDisabled,
    ComposedOutputDisabled,
    MixedOperatorsDisabled,
    MixedInitialDisabled,
    MixedTransitionDisabled,
    WordEmissionDisabled,
    WordEmissionContextDisabled,
    WordEmissionPrefixContextDisabled,
    WordEmissionGeometryDisabled,
    SharedOperatorBindingDisabled,
    SharedOperatorTransitionDisabled,
    ActionTransitionDisabled,
    ActionBindingDisabled,
    ActionEmissionDisabled,
    LexicalActionContextDisabled,
    LexicalEmissionDisabled,
    LexicalRecordReadDisabled,
    LexicalEmissionGeometryDisabled,
    OperationTransitionDisabled,
    OperationTransitionIntermediateDisabled,
    #[default]
    Full,
    GeometryDisabled,
    ZetaDisabled,
    H4Disabled,
    OrientationDisabled,
    PairedDisabled,
    RadialDisabled,
    HeatmapDisabled,
    MemoryDisabled,
    ResponseStateDisabled,
    ValuesDisabled,
    ValueLexemesDisabled,
    ValueCompletionDisabled,
    ValueCompletionGeometryDisabled,
    ResponseEntryDisabled,
    ResponseEntryGeometryDisabled,
    WordCopyDisabled,
    WordCopyGeometryDisabled,
    WordCopyDispatchDisabled,
    LearnedRoutingDisabled,
    LearnedRoutingSelectionDisabled,
    LearnedRoutingTransformDisabled,
    LearnedRoutingChainDisabled,
}

/// Explicit feature addresses, never content digests. Kinds 0/1 are full
/// prime lexical addresses; 2/3 are exact H4 state/trajectory; 4 is signed
/// orientation; 8..16 are the eight fixed-zeta phase channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) struct Feature {
    pub kind: u8,
    pub value: u64,
}
// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_BEGIN
// Runtime feature routing and attenuation; included in the kernel source guard.
impl Feature {
    fn group(self) -> usize {
        match self.kind {
            0 | 1 => 0,
            2 | 3 | 6 => 1,
            4 | 24 | 25 => 2,
            5 => 3,
            7 => 4,
            8..=15 => 5,
            _ => 6,
        }
    }
    fn shift(self) -> u8 {
        if (8..=23).contains(&self.kind) {
            3
        } else if self.kind >= 2 {
            1
        } else {
            0
        }
    }
    fn admitted(self, control: Control) -> bool {
        match control {
            Control::Full
            | Control::HistoricalReadDisabled
            | Control::HistoricalQueryWindowDisabled
            | Control::CurrentQueryHandoffDisabled
            | Control::CurrentQueryHandoffTransformDisabled
            | Control::CurrentQueryHandoffScopeDisabled
            | Control::HistoricalVersionIntentDisabled
            | Control::HistoricalVersionIntentTransformDisabled
            | Control::HistoricalVersionIntentScopeDisabled
            | Control::HistoricalVersionIntentAncestorDisabled
            | Control::HistoricalVersionIntentAbstainDisabled
            | Control::HistoricalVersionIntentReassertionDisabled
            | Control::HistoricalVersionIntentReassertionHeadDisabled
            | Control::HistoricalVersionIntentReaderWindowDisabled
            | Control::HistoricalVersionIntentReaderTurnScopeDisabled
            | Control::HistoricalQueryContextDisabled
            | Control::HistoricalFieldCompositionDisabled
            | Control::RelationStartRefinementDisabled
            | Control::CurrentSourceDisabled
            | Control::CurrentSourceVersionDisabled
            | Control::CurrentRelationReadAll
            | Control::WriterRoleDisabled
            | Control::WriterRoleContextDisabled
            | Control::WriterChoiceDisabled
            | Control::WriterChoiceBoundaryDisabled
            | Control::FieldCompositionDisabled
            | Control::FieldCompositionContextDisabled
            | Control::FieldCompositionGeometryDisabled
            | Control::FieldCompositionReadDisabled
            | Control::SourceSpanDisabled
            | Control::SourceSpanContextDisabled
            | Control::SourceSpanPairDisabled
            | Control::WriterLexicalDisabled
            | Control::SourceRoleRefinementDisabled
            | Control::SourceContextDisabled
            | Control::SourceContextWindowOnly
            | Control::JointAdmissionDisabled
            | Control::OperationTransitionIntermediateDisabled
            | Control::WordEmissionDisabled
            | Control::WordEmissionContextDisabled
            | Control::WordEmissionPrefixContextDisabled
            | Control::WordEmissionGeometryDisabled
            | Control::SharedOperatorBindingDisabled
            | Control::SharedOperatorTransitionDisabled
            | Control::ActionTransitionDisabled
            | Control::ActionBindingDisabled
            | Control::ActionEmissionDisabled
            | Control::LexicalActionContextDisabled
            | Control::LexicalEmissionDisabled
            | Control::LexicalRecordReadDisabled
            | Control::LexicalEmissionGeometryDisabled
            | Control::OperationTransitionDisabled
            | Control::LiteralRefinementDisabled
            | Control::InstructionBindingDisabled
            | Control::ComposedOutputDisabled
            | Control::MixedOperatorsDisabled
            | Control::MixedInitialDisabled
            | Control::MixedTransitionDisabled
            | Control::MemoryDisabled
            | Control::ResponseStateDisabled
            | Control::ValuesDisabled
            | Control::ValueLexemesDisabled
            | Control::ValueCompletionDisabled
            | Control::ValueCompletionGeometryDisabled
            | Control::ResponseEntryDisabled
            | Control::ResponseEntryGeometryDisabled
            | Control::WordCopyDisabled
            | Control::WordCopyGeometryDisabled
            | Control::LearnedRoutingDisabled
            | Control::LearnedRoutingSelectionDisabled
            | Control::LearnedRoutingTransformDisabled
            | Control::LearnedRoutingChainDisabled
            | Control::WordCopyDispatchDisabled => true,
            Control::GeometryDisabled => self.kind < 2,
            Control::ZetaDisabled => !(8..=15).contains(&self.kind) && self.kind != 5,
            Control::H4Disabled => self.kind < 2 || (8..=15).contains(&self.kind),
            Control::OrientationDisabled => self.kind != 4,
            Control::PairedDisabled => self.kind != 6 && !(16..=23).contains(&self.kind),
            Control::RadialDisabled => self.kind != 7,
            Control::HeatmapDisabled => self.kind != 4 && self.kind != 24 && self.kind != 25,
        }
    }
}
// NATIVE_GEOMETRIC_INTEGER_FEATURE_METHODS_END

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenGeometry {
    prime: u32,
    leaf: u16,
    phases: [u16; PHASE_CHANNELS],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Geometry {
    root_cid: String,
    product_cid: String,
    zeta_grid: String,
    identity: u16,
    row_bases: Vec<usize>,
    products: Vec<u16>,
    inverses: Vec<u16>,
    /// Encoded signs of exact root coordinates (q0,q1): each -1/0/+1 -> 0/1/2.
    orientation: Vec<u8>,
    anchors: anchors::AnchorTable,
    square_offset: i64,
    squares: Vec<i64>,
    tokens: Vec<TokenGeometry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenScore {
    token: u32,
    score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScoreRow {
    feature: Feature,
    default_score: i32,
    scores: Vec<TokenScore>,
    postings: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrainingProgress {
    pub documents_completed: usize,
    pub target_positions: u64,
    pub feature_events: u64,
    pub dropped_feature_events: u64,
    pub learned_rows: usize,
    pub learned_associations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ModelWire")]
pub struct Model {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    historical_version_intent: Option<historical_version_intent::HistoricalVersionIntent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_query_handoff: Option<current_query_handoff::CurrentQueryHandoff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    historical_query_context: Option<historical_query_context::HistoricalQueryContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    historical_field_composition: Option<historical_field_composition::HistoricalFieldComposition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_start_refinement: Option<relation_start_refinement::RelationStartRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    historical_read: Option<historical_read::HistoricalRead>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    writer_role: Option<writer_role::WriterRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_source: Option<current_source::CurrentSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    writer_choice: Option<writer_choice::WriterChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    field_composition: Option<field_composition::FieldComposition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    writer_lexical: Option<writer_lexical::WriterLexical>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_role_refinement: Option<source_role_refinement::SourceRoleRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    word_emission: Option<word_emission::WordEmission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shared_operator_refinement: Option<shared_operator_refinement::SharedOperatorRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    action_emission: Option<action_emission::ActionEmission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mixed_operators: Option<mixed_operators::MixedOperators>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    composed_output: Option<composed_output::ComposedOutput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instruction_binding: Option<instruction_binding::InstructionBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lexical_emission: Option<lexical_emission::LexicalEmission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    operation_transition: Option<operation_transition::OperationTransition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_role_refinement: Option<typed_routing::TypedRoleRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_writer_refinement: Option<writer_refinement::WriterRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_start_context: Option<Vec<word_copy_types::WordCopyAddress>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_start: Option<source_routing::SourceRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_reverse_spans: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_spans: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_span_context: Option<Vec<word_copy_types::WordCopyAddress>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_span: Option<source_routing::SourceRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_context: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    literal_routing_refinement: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    joint_admission: Option<joint_admission::JointAdmission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    no_read_completion: Option<response_entry_types::ResponseEntryModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_routing: Option<typed_routing::TypedRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_roles: Option<typed_routing::TypedRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_literals: Option<typed_routing::TypedRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    relation_writer: Option<relation_training::WriterRevision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dependent_read: Option<dependent_read::DependentRead>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_routing: Option<source_routing::SourceRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_routing_refinement: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    learned_routing: Option<learned_routing::RoutingBlock>,
    schema: String,
    artifact_cid: String,
    uor_model_address: String,
    config: Config,
    training: TrainingProgress,
    construction: Vec<DocumentReceipt>,
    lexical_pieces: Vec<Vec<u8>>,
    geometry: Geometry,
    prior_scores: Vec<i32>,
    prior_postings: Vec<u32>,
    rows: Vec<ScoreRow>,
    readout: mixture::Readout,
    readout_training: Vec<DocumentReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memory_read: Option<memory_types::MemoryModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    values: Option<value_types::ValueModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    completion: Option<completion_types::CompletionModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_entry: Option<response_entry_types::ResponseEntryModel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelWire {
    #[serde(default)]
    historical_version_intent: Option<historical_version_intent::HistoricalVersionIntent>,
    #[serde(default)]
    current_query_handoff: Option<current_query_handoff::CurrentQueryHandoff>,
    #[serde(default)]
    historical_query_context: Option<historical_query_context::HistoricalQueryContext>,
    #[serde(default)]
    historical_field_composition: Option<historical_field_composition::HistoricalFieldComposition>,
    #[serde(default)]
    relation_start_refinement: Option<relation_start_refinement::RelationStartRefinement>,
    #[serde(default)]
    historical_read: Option<historical_read::HistoricalRead>,
    #[serde(default)]
    writer_role: Option<writer_role::WriterRole>,
    #[serde(default)]
    current_source: Option<current_source::CurrentSource>,
    #[serde(default)]
    writer_choice: Option<writer_choice::WriterChoice>,
    #[serde(default)]
    field_composition: Option<field_composition::FieldComposition>,
    #[serde(default)]
    writer_lexical: Option<writer_lexical::WriterLexical>,
    #[serde(default)]
    source_role_refinement: Option<source_role_refinement::SourceRoleRefinement>,
    #[serde(default)]
    word_emission: Option<word_emission::WordEmission>,
    #[serde(default)]
    shared_operator_refinement: Option<shared_operator_refinement::SharedOperatorRefinement>,
    #[serde(default)]
    action_emission: Option<action_emission::ActionEmission>,
    #[serde(default)]
    mixed_operators: Option<mixed_operators::MixedOperators>,
    #[serde(default)]
    composed_output: Option<composed_output::ComposedOutput>,
    #[serde(default)]
    instruction_binding: Option<instruction_binding::InstructionBinding>,
    #[serde(default)]
    lexical_emission: Option<lexical_emission::LexicalEmission>,
    #[serde(default)]
    operation_transition: Option<operation_transition::OperationTransition>,
    #[serde(default)]
    typed_role_refinement: Option<typed_routing::TypedRoleRefinement>,
    #[serde(default)]
    relation_writer_refinement: Option<writer_refinement::WriterRefinement>,
    #[serde(default)]
    relation_start_context: Option<Vec<word_copy_types::WordCopyAddress>>,
    #[serde(default)]
    relation_start: Option<source_routing::SourceRouting>,
    #[serde(default)]
    relation_reverse_spans: Option<String>,
    #[serde(default)]
    relation_spans: Option<String>,
    #[serde(default)]
    source_span_context: Option<Vec<word_copy_types::WordCopyAddress>>,
    #[serde(default)]
    source_span: Option<source_routing::SourceRouting>,
    #[serde(default)]
    source_context: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default)]
    literal_routing_refinement: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default)]
    joint_admission: Option<joint_admission::JointAdmission>,
    #[serde(default)]
    no_read_completion: Option<response_entry_types::ResponseEntryModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_routing: Option<typed_routing::TypedRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_roles: Option<typed_routing::TypedRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    typed_literals: Option<typed_routing::TypedRouting>,
    #[serde(default)]
    relation_writer: Option<relation_training::WriterRevision>,
    #[serde(default)]
    dependent_read: Option<dependent_read::DependentRead>,
    #[serde(default)]
    source_routing: Option<source_routing::SourceRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_routing_refinement: Option<source_routing::SourceRoutingRefinement>,
    #[serde(default)]
    learned_routing: Option<learned_routing::RoutingBlock>,
    schema: String,
    artifact_cid: String,
    uor_model_address: String,
    config: Config,
    training: TrainingProgress,
    construction: Vec<DocumentReceipt>,
    lexical_pieces: Vec<Vec<u8>>,
    geometry: Geometry,
    prior_scores: Vec<i32>,
    prior_postings: Vec<u32>,
    rows: Vec<ScoreRow>,
    readout: mixture::Readout,
    readout_training: Vec<DocumentReceipt>,
    #[serde(default)]
    memory_read: Option<memory_types::MemoryModel>,
    #[serde(default)]
    values: Option<value_types::ValueModel>,
    #[serde(default)]
    completion: Option<completion_types::CompletionModel>,
    #[serde(default)]
    response_entry: Option<response_entry_types::ResponseEntryModel>,
}
impl TryFrom<ModelWire> for Model {
    type Error = Error;
    fn try_from(wire: ModelWire) -> Result<Self> {
        let model = Self {
            historical_version_intent: wire.historical_version_intent,
            current_query_handoff: wire.current_query_handoff,
            historical_query_context: wire.historical_query_context,
            historical_field_composition: wire.historical_field_composition,
            relation_start_refinement: wire.relation_start_refinement,
            writer_role: wire.writer_role,
            current_source: wire.current_source,
            historical_read: wire.historical_read,
            writer_choice: wire.writer_choice,
            field_composition: wire.field_composition,
            writer_lexical: wire.writer_lexical,
            source_role_refinement: wire.source_role_refinement,
            word_emission: wire.word_emission,
            shared_operator_refinement: wire.shared_operator_refinement,
            action_emission: wire.action_emission,
            mixed_operators: wire.mixed_operators,
            composed_output: wire.composed_output,
            instruction_binding: wire.instruction_binding,
            lexical_emission: wire.lexical_emission,
            operation_transition: wire.operation_transition,
            typed_role_refinement: wire.typed_role_refinement,
            relation_writer_refinement: wire.relation_writer_refinement,
            relation_start_context: wire.relation_start_context,
            relation_start: wire.relation_start,
            relation_reverse_spans: wire.relation_reverse_spans,
            relation_spans: wire.relation_spans,
            source_span_context: wire.source_span_context,
            source_span: wire.source_span,
            source_context: wire.source_context,
            literal_routing_refinement: wire.literal_routing_refinement,
            joint_admission: wire.joint_admission,
            no_read_completion: wire.no_read_completion,
            typed_routing: wire.typed_routing,
            typed_roles: wire.typed_roles,
            typed_literals: wire.typed_literals,
            relation_writer: wire.relation_writer,
            dependent_read: wire.dependent_read,
            source_routing: wire.source_routing,
            source_routing_refinement: wire.source_routing_refinement,
            learned_routing: wire.learned_routing,
            schema: wire.schema,
            artifact_cid: wire.artifact_cid,
            uor_model_address: wire.uor_model_address,
            config: wire.config,
            training: wire.training,
            construction: wire.construction,
            lexical_pieces: wire.lexical_pieces,
            geometry: wire.geometry,
            prior_scores: wire.prior_scores,
            prior_postings: wire.prior_postings,
            rows: wire.rows,
            readout: wire.readout,
            readout_training: wire.readout_training,
            memory_read: wire.memory_read,
            values: wire.values,
            completion: wire.completion,
            response_entry: wire.response_entry,
        };
        model.validate()?;
        Ok(model)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Work {
    #[serde(default, skip_serializing_if = "RoutingWork::is_empty")]
    pub learned_routing: RoutingWork,
    #[serde(default, skip_serializing_if = "WordCopyWork::is_empty")]
    pub word_copy: WordCopyWork,
    #[serde(default, skip_serializing_if = "CompletionWork::is_empty")]
    pub response_entry: CompletionWork,
    #[serde(default, skip_serializing_if = "CompletionWork::is_empty")]
    pub completion: CompletionWork,
    #[serde(default, skip_serializing_if = "ValueWork::is_empty")]
    pub values: ValueWork,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_query_captures: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_commits: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_requeries: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_continuations: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_base_steps: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_stops: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_mismatches: u64,
    #[serde(default, skip_serializing_if = "zero_work")]
    pub response_reference_reads: u64,
    pub observed_tokens: u64,
    pub evictions: u64,
    pub h4_table_reads: u64,
    pub orientation_table_reads: u64,
    pub anchor_table_reads: u64,
    pub radial_square_reads: u64,
    pub phase_additions: u64,
    pub feature_queries: u64,
    pub matched_rows: u64,
    pub candidate_offers: u64,
    pub candidate_evaluations: u64,
    pub score_lookups: u64,
    pub mixture_gate_reads: u64,
    #[serde(default)]
    pub memory_index_reads: u64,
    #[serde(default)]
    pub memory_index_writes: u64,
    #[serde(default)]
    pub memory_stale_rejections: u64,
    #[serde(default)]
    pub memory_candidates: u64,
    #[serde(default)]
    pub memory_score_lookups: u64,
    #[serde(default)]
    pub memory_h4_reads: u64,
    #[serde(default)]
    pub memory_phase_updates: u64,
    #[serde(default)]
    pub memory_cue_reads: u64,
    #[serde(default)]
    pub memory_composed_candidates: u64,
    #[serde(default)]
    pub memory_composition_feature_offers: u64,
    #[serde(default)]
    pub memory_composition_duplicate_features: u64,
    #[serde(default)]
    pub memory_composition_comparisons: u64,
    #[serde(default)]
    pub memory_composition_feature_moves: u64,
}

fn zero_work(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub token: u32,
    pub score: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prediction {
    pub token: u32,
    pub score: i64,
    pub candidate_count: usize,
    pub geometric_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generation {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub word_copy_trace: Vec<WordCopyDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_entry_trace: Vec<ResponseEntryDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completion_trace: Vec<CompletionDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub value_trace: Vec<ValueDecision>,
    /// First at most 96 decisions, recorded outside the allocation-free kernel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_trace: Vec<ResponseDecision>,
    pub text: String,
    pub utf8_valid: bool,
    /// Exact output bytes are retained if byte fallback generates invalid UTF-8.
    pub bytes: Vec<u8>,
    pub token_ids: Vec<u32>,
    pub stop: String,
    pub work: Work,
    pub state: StateView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    pub documents: usize,
    pub positions: u64,
    pub correct: u64,
    pub candidate_hits: u64,
    pub geometric_row_positions: u64,
    pub top1: f64,
    pub candidate_coverage: f64,
    pub work: Work,
}

#[cfg(test)]
mod response_runtime_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod value_runtime_tests;

#[cfg(test)]
mod completion_runtime_tests;

#[cfg(test)]
mod response_entry_runtime_tests;
#[cfg(test)]
mod response_entry_training_tests;
#[cfg(test)]
mod role_read_tests;
#[cfg(test)]
mod word_copy_tests;
#[cfg(test)]
mod word_copy_value_tests;

/// Experimental relational attention learning; typed context, no prose qualification.
pub mod relational_attention;

/// Learned query update between two geometric reads.
pub mod dependent_attention;

/// Learned content-controlled geometric read loop.
pub mod adaptive_attention;

/// Grounded variable-length text with emitted-byte feedback.
pub mod text_attention;

/// Shared recurrent action learning with emitted-content query feedback.
pub mod recurrent_text;

pub mod language_relation;
pub mod ordered_state;

pub mod relative_language;
