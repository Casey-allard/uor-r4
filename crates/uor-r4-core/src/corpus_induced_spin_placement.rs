//! Construction-induced exact document-spin placement for issue #973.
//!
//! `CorpusInducedDocumentSpinPlacementR4V1` is an additive, source-free
//! placement overlay over the unchanged `SFTBL001` table and #953
//! `MultiscaleCountRadiusR4V1` admission path.  Compilation may observe only
//! D3-construction causal prefix -> observed-next-route pairs.  The serialized
//! artifact retains one aggregate exact prototype per usable candidate and no
//! source text, prefix row, continuation, target, or anti-recall index.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::bounded_global_exact_spin_attention::{
    candidate_relative_exact_cost, exact_s3_spin_to_h4, select_unique_minimum_exact_costs,
    BoundedGlobalExactSpinCost, BoundedGlobalExactSpinError, BoundedGlobalExactSpinStateTrace,
    ExactSpinState,
};
use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, H4BinaryIcosahedralClosure, OpaqueH4TableIndex,
};
use crate::prime_route_geometric_attention::H4S3AngularShell;
use crate::source_free_table::{
    BackoffOrder, Continuation, ContinuationStop, MatchedGeometricPrediction,
    MultiscaleCountRadiusR4V1, MultiscaleCountRadiusWork, SourceDocument, SourceFreeTable,
    SourceFreeTableError, BOS_TOKEN, EOS_TOKEN, MAX_CONTINUATION_UNITS,
};

const ARTIFACT_MAGIC: [u8; 8] = *b"CIDSP001";
const ARTIFACT_SCHEMA: u32 = 1;
const ARTIFACT_DOMAIN: &str = "uor-r4.corpus-induced-document-spin-placement/1";
const LEAF_BASIS_DOMAIN: &str = "uor-r4.sftbl001-exact-spin-identity-leaf/1";
const INDUCTION_POLICY_DOMAIN: &str = "uor-r4.document-prefix-componentwise-frechet-placement/1";
const QUERY_POLICY_DOMAIN: &str = "uor-r4.document-spin-c-inverse-g-lexicographic-minimum/1";
const SCORE_FIREWALL_DOMAIN: &str = "uor-r4.document-spin-score-firewall/1";
const SCORE_FIREWALL_POLICY: &str = "schema=1\ncapability=causal-prefix-frame-only\nallowed-score-inputs=exact-query-state,aggregate-prototype-state\nforbidden=heldout-target,compiler-future,teacher,provider,source-weight,runtime-corpus,token,payload,prime,rank,digest,support,provenance\ncertificate=BLAKE3(policy,operator,prefix-units,candidate-count,four-executed-query-states,forbidden-mask)";
const ANTI_RECALL_DOMAIN: &str = "uor-r4.document-spin-operative-anti-recall/1";
const ANTI_RECALL_POLICY_IDENTITY: &str = "schema=1\nfull-prefix-cid=BLAKE3(PREFIX_DOMAIN || each causal SFTBL001 token as fixed-width u32 little-endian)\ncanonical-witness-order=(active-row,canonical-support-cid,full-prefix-cid,document-id,target-index)\nwitness=first-operative-real-non-EOS-real-differs-from-each-control";
const PREFIX_DOMAIN: &[u8] = b"uor-r4.document-spin-prefix/1\0";
const ROW_DOMAIN: &[u8] = b"uor-r4.document-spin-active-row/1\0";
const SUPPORT_DOMAIN: &[u8] = b"uor-r4.document-spin-support/1\0";
const PREDICTION_DOMAIN: &[u8] = b"uor-r4.document-spin-prediction/1\0";
const CONSTRUCTION_SET_DOMAIN: &[u8] = b"uor-r4.document-spin-construction-set/1\0";
const HELD_OUT_SET_DOMAIN: &[u8] = b"uor-r4.document-spin-heldout-set/1\0";
const FROZEN_CORPUS_CID: &str =
    "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const FROZEN_TABLE_CID: &str =
    "blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297";
const FROZEN_OVERLAY_CID: &str =
    "blake3:914126a311c3984d1482258a8f0a7fa2e34896540d502d19f1d9076fbd4a9b76";
const FROZEN_CONSTRUCTION_SET_KAPPA: &str =
    "blake3:af2a2d7d49db55279e7ea40947a3259ac0a100aa56e8d920951e7c27eaf6df5c";
const FROZEN_HELD_OUT_SET_KAPPA: &str =
    "blake3:7a7558e96aa86aa2d8965972b69ddce02222c6eccc8ca560df2141fc0ac4170e";
const PHASE_MODULUS_Q29: i64 = 3_373_259_426;
const PHASE_HALF_Q29: i64 = 1_686_629_713;
const MAX_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const MIN_PROTOTYPE_DOCUMENTS: u32 = 2;
const MIN_DISTINCT_PROTOTYPE_STATES: u32 = 2;
const FROZEN_HELD_OUT_DOCUMENTS: u64 = 596;
const FROZEN_TARGET_FREE_ADMISSIONS: u64 = 81_177;
#[cfg(not(target_arch = "wasm32"))]
const MAX_NATIVE_DOCUMENT_SCAN_WORKERS: usize = 8;
pub const MIN_OPERATIVE_ANTI_RECALL_POSITIONS: u64 = 1_024;
pub const POSITIVE_TERMINAL: &str =
    "RETAIN_CORPUS_INDUCED_DOCUMENT_SPIN_ATTENTION_CONTINUE_FINAL_973_REQUALIFICATION";
pub const NEGATIVE_TERMINAL: &str = "RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT";
pub const UNAVAILABLE_TERMINAL: &str = "UNAVAILABLE_OPERATIVE_ANTI_RECALL_OR_REACHABILITY";

const LEAF_BASIS_IDENTITY: &str = "schema=1\ndomain=uor-r4.sftbl001-exact-spin-identity-leaf/1\ntoken=SFTBL001-u32-token-id\nprime=zero-based-rth-prime,p_0=2\nh4-row=r-mod-4 over exact V2 roots [(1,0,0,0),(0,1,0,0),(1/2,1/2,1/2,1/2),(1/2,-1/2,1/2,-1/2)]\nfiber=wrap_M(p_r*1000003+r*17071)\ntorsion=wrap_M(-p_r*97409+r*7919)\nM=3373259426\nBOS=identity-reset";
const INDUCTION_POLICY_IDENTITY: &str = "schema=1\ndomain=uor-r4.document-prefix-componentwise-frechet-placement/1\nscope=document-only\nexamples=construction-only max-count trigram/bigram ties whose observed next route is admitted\ncap=earliest eligible occurrence per candidate/document\nminimum=two-distinct-documents-and-two-distinct-exact-prefix-states\nh4=minimum-summed-angular-shell-rank-over-all-120-roots\nphase=minimum-summed-circular-Q29-distance-over-observed-values\nties=canonical-H4-table-order-then-ascending-signed-Q29";
const QUERY_POLICY_IDENTITY: &str = "schema=1\ndomain=uor-r4.document-spin-c-inverse-g-lexicographic-minimum/1\nadmission=unchanged-953-max-count-tie\nrelative=C^-1*G\ncost=lexicographic(H4S3AngularShell,circular-abs-fiber-Q29,circular-abs-torsion-Q29)\nselection=unique-minimum-else-953-fallback\ncontrols=real,scope-disabled,reverse-order,cyclic-operator-permutation\nforbidden-score-inputs=token,payload,prime,rank,digest,support,provenance,target,future,teacher,provider,source-weight";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpusInducedDocumentSpinError {
    Invalid(String),
    SourceFree(String),
    ExactSpin(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for CorpusInducedDocumentSpinError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::SourceFree(reason) => write!(formatter, "source-free table: {reason}"),
            Self::ExactSpin(reason) => write!(formatter, "exact spin: {reason}"),
            Self::Serialization(reason) => write!(formatter, "serialization: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("corpus-induced document-spin arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for CorpusInducedDocumentSpinError {}

impl From<SourceFreeTableError> for CorpusInducedDocumentSpinError {
    fn from(error: SourceFreeTableError) -> Self {
        Self::SourceFree(error.to_string())
    }
}

impl From<BoundedGlobalExactSpinError> for CorpusInducedDocumentSpinError {
    fn from(error: BoundedGlobalExactSpinError) -> Self {
        Self::ExactSpin(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusInducedDocumentSpinArm {
    Real,
    ScopeDisabled,
    OrderShuffled,
    OperatorPermuted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusInducedDocumentSpinAbstention {
    NotMaximumCountTie,
    MissingPrototype,
    CostTie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct CorpusInducedDocumentSpinForbiddenReads {
    pub held_out_target_reads: u64,
    pub compiler_future_reads: u64,
    pub teacher_calls: u64,
    pub provider_calls: u64,
    pub source_weight_reads: u64,
    pub runtime_corpus_reads: u64,
    pub token_score_reads: u64,
    pub payload_score_reads: u64,
    pub prime_score_reads: u64,
    pub rank_score_reads: u64,
    pub digest_score_reads: u64,
    pub support_score_reads: u64,
    pub provenance_score_reads: u64,
}

impl CorpusInducedDocumentSpinForbiddenReads {
    pub fn total(self) -> u64 {
        [
            self.held_out_target_reads,
            self.compiler_future_reads,
            self.teacher_calls,
            self.provider_calls,
            self.source_weight_reads,
            self.runtime_corpus_reads,
            self.token_score_reads,
            self.payload_score_reads,
            self.prime_score_reads,
            self.rank_score_reads,
            self.digest_score_reads,
            self.support_score_reads,
            self.provenance_score_reads,
        ]
        .into_iter()
        .fold(0_u64, u64::saturating_add)
    }

    fn saturating_accumulate(&mut self, other: Self) {
        self.held_out_target_reads = self
            .held_out_target_reads
            .saturating_add(other.held_out_target_reads);
        self.compiler_future_reads = self
            .compiler_future_reads
            .saturating_add(other.compiler_future_reads);
        self.teacher_calls = self.teacher_calls.saturating_add(other.teacher_calls);
        self.provider_calls = self.provider_calls.saturating_add(other.provider_calls);
        self.source_weight_reads = self
            .source_weight_reads
            .saturating_add(other.source_weight_reads);
        self.runtime_corpus_reads = self
            .runtime_corpus_reads
            .saturating_add(other.runtime_corpus_reads);
        self.token_score_reads = self
            .token_score_reads
            .saturating_add(other.token_score_reads);
        self.payload_score_reads = self
            .payload_score_reads
            .saturating_add(other.payload_score_reads);
        self.prime_score_reads = self
            .prime_score_reads
            .saturating_add(other.prime_score_reads);
        self.rank_score_reads = self.rank_score_reads.saturating_add(other.rank_score_reads);
        self.digest_score_reads = self
            .digest_score_reads
            .saturating_add(other.digest_score_reads);
        self.support_score_reads = self
            .support_score_reads
            .saturating_add(other.support_score_reads);
        self.provenance_score_reads = self
            .provenance_score_reads
            .saturating_add(other.provenance_score_reads);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinScoreFirewallCertificate {
    pub schema: u32,
    pub domain: String,
    pub policy_kappa: String,
    pub operator_cid: String,
    pub causal_prefix_units: u64,
    pub candidate_count: u64,
    pub query_state_kappa: String,
    pub forbidden_dependency_mask: u16,
    pub certificate_cid: String,
}

impl CorpusInducedDocumentSpinScoreFirewallCertificate {
    pub fn validate(&self) -> bool {
        self.schema == ARTIFACT_SCHEMA
            && self.domain == SCORE_FIREWALL_DOMAIN
            && self.policy_kappa == identity_kappa(&[SCORE_FIREWALL_POLICY])
            && self.forbidden_dependency_mask == 0
            && self.certificate_cid == score_firewall_certificate_cid(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinWork {
    pub local: MultiscaleCountRadiusWork,
    pub prefix_leaf_reads: u64,
    pub prefix_h4_product_reads: u64,
    pub prefix_phase_additions: u64,
    pub prototype_reads: u64,
    pub prototype_inverse_reads: u64,
    pub relative_h4_product_reads: u64,
    pub relative_phase_additions: u64,
    pub angular_shell_reads: u64,
    pub phase_distance_reads: u64,
    pub cost_comparisons: u64,
    pub final_choice_operations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinDecision {
    pub arm: CorpusInducedDocumentSpinArm,
    pub token: u32,
    pub unique_minimum: Option<u32>,
    pub minimum_cost: Option<BoundedGlobalExactSpinCost>,
    pub abstention: Option<CorpusInducedDocumentSpinAbstention>,
    pub support_tokens: Vec<u32>,
    pub work: CorpusInducedDocumentSpinWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinCandidateEvidence {
    pub token: u32,
    pub prototype_state: BoundedGlobalExactSpinStateTrace,
    pub prototype_document_support: u32,
    pub prototype_distinct_state_support: u32,
    pub real_relative_state: BoundedGlobalExactSpinStateTrace,
    pub real_cost: BoundedGlobalExactSpinCost,
    pub order_shuffled_relative_state: BoundedGlobalExactSpinStateTrace,
    pub order_shuffled_cost: BoundedGlobalExactSpinCost,
    pub permuted_from_token: u32,
    pub permuted_prototype_state: BoundedGlobalExactSpinStateTrace,
    pub operator_permuted_relative_state: BoundedGlobalExactSpinStateTrace,
    pub operator_permuted_cost: BoundedGlobalExactSpinCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedCorpusInducedDocumentSpinPrediction {
    pub local: MatchedGeometricPrediction,
    pub operator_cid: String,
    pub natural_state: BoundedGlobalExactSpinStateTrace,
    pub reverse_state: BoundedGlobalExactSpinStateTrace,
    pub candidate_evidence: Vec<CorpusInducedDocumentSpinCandidateEvidence>,
    pub real: CorpusInducedDocumentSpinDecision,
    pub scope_disabled: CorpusInducedDocumentSpinDecision,
    pub order_shuffled: CorpusInducedDocumentSpinDecision,
    pub operator_permuted: CorpusInducedDocumentSpinDecision,
    pub prototype_complete: bool,
    pub natural_reverse_distinct: bool,
    pub permutation_cost_vector_changed: bool,
    pub support_matched: bool,
    pub work_matched: bool,
    pub score_firewall_certificate: CorpusInducedDocumentSpinScoreFirewallCertificate,
    pub forbidden_reads: CorpusInducedDocumentSpinForbiddenReads,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedCorpusInducedDocumentSpinContinuation {
    pub first_decision: MatchedCorpusInducedDocumentSpinPrediction,
    pub real: Continuation,
    pub scope_disabled: Continuation,
    pub order_shuffled: Continuation,
    pub operator_permuted: Continuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct CorpusInducedDocumentSpinArtifactStats {
    pub construction_documents: u64,
    pub eligible_construction_positions: u64,
    pub retained_document_exemplars: u64,
    pub observed_candidate_tokens: u64,
    pub usable_prototypes: u64,
    pub rejected_single_document_prototypes: u64,
    pub rejected_single_state_prototypes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinPrototypeTrace {
    pub token: u32,
    pub payload_cid: String,
    pub state: BoundedGlobalExactSpinStateTrace,
    pub construction_document_support: u32,
    pub distinct_state_support: u32,
    pub h4_minimum_objective_sum: u128,
    pub h4_minimizer_count: u16,
    pub fiber_minimum_objective_sum: u128,
    pub fiber_minimizer_count: u32,
    pub torsion_minimum_objective_sum: u128,
    pub torsion_minimizer_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinCensusPosition {
    pub document_id: String,
    pub target_index: u32,
    pub prefix_cid: String,
    pub row_cid: String,
    pub support_cid: String,
    pub prediction_cid: String,
    pub real_token: u32,
    pub scope_disabled_token: u32,
    pub order_shuffled_token: u32,
    pub operator_permuted_token: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusInducedDocumentSpinTargetFreeCensus {
    pub schema: u32,
    pub domain: String,
    pub operator_cid: String,
    pub anti_recall_index_kappa: String,
    pub held_out_set_kappa: String,
    pub held_out_documents: u64,
    /// Every target-blind trigram/bigram maximum-count-tie prefix.  The frozen
    /// corpus value is 81,177; the 76,641 known-target count is attached only
    /// by [`CorpusInducedDocumentSpinPlacementR4V1::evaluate_held_out`].
    pub admission_opportunities: u64,
    /// Admission positions whose active row occurs in at least two held-out
    /// documents (a position count, not a distinct-row count).
    pub rows_in_multiple_held_out_documents: u64,
    pub prototype_complete_positions: u64,
    pub full_prefix_construction_hits: u64,
    pub natural_state_construction_hits: u64,
    pub operative_signature_construction_hits: u64,
    pub natural_reverse_equal_positions: u64,
    pub permutation_inert_positions: u64,
    pub support_mismatches: u64,
    pub work_mismatches: u64,
    pub invalid_score_firewall_certificates: u64,
    pub score_firewall_policy_kappa: String,
    pub meets_frozen_preflight: bool,
    pub operative_positions: Vec<CorpusInducedDocumentSpinCensusPosition>,
    pub frozen_decoded_witness: Option<CorpusInducedDocumentSpinCensusPosition>,
    pub forbidden_reads: CorpusInducedDocumentSpinForbiddenReads,
}

impl CorpusInducedDocumentSpinTargetFreeCensus {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CorpusInducedDocumentSpinError> {
        serde_json::to_vec(self)
            .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))
    }

    pub fn artifact_cid(&self) -> Result<String, CorpusInducedDocumentSpinError> {
        Ok(blake3_label(&self.canonical_bytes()?))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorpusInducedDocumentSpinComparatorEvaluation {
    pub wins: u64,
    pub losses: u64,
    pub ties: u64,
    pub discordant: u64,
    pub one_sided_exact_sign_test: String,
    pub passes: bool,
    pub terminal: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CorpusInducedDocumentSpinEvaluation {
    pub schema: u32,
    pub domain: String,
    pub decision: String,
    pub operator_cid: String,
    pub census_cid: String,
    pub held_out_set_kappa: String,
    pub held_out_documents: u64,
    pub post_join_admission_opportunities: u64,
    pub post_join_known_target_admission_opportunities: u64,
    pub operative_positions: u64,
    pub operative_known_target_positions: u64,
    pub real_correct: u64,
    pub scope_disabled_correct: u64,
    pub order_shuffled_correct: u64,
    pub operator_permuted_correct: u64,
    pub witness_continuation_contrast: bool,
    pub witness_continuation: Option<MatchedCorpusInducedDocumentSpinContinuation>,
    pub witness_continuation_cid: Option<String>,
    pub versus_scope_disabled: CorpusInducedDocumentSpinComparatorEvaluation,
    pub versus_order_shuffled: CorpusInducedDocumentSpinComparatorEvaluation,
    pub versus_operator_permuted: CorpusInducedDocumentSpinComparatorEvaluation,
    pub document_blocked_versus_scope_disabled: CorpusInducedDocumentSpinComparatorEvaluation,
    pub document_blocked_versus_order_shuffled: CorpusInducedDocumentSpinComparatorEvaluation,
    pub document_blocked_versus_operator_permuted: CorpusInducedDocumentSpinComparatorEvaluation,
    pub target_reads: u64,
    pub forbidden_reads: CorpusInducedDocumentSpinForbiddenReads,
}

impl CorpusInducedDocumentSpinEvaluation {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CorpusInducedDocumentSpinError> {
        serde_json::to_vec(self)
            .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))
    }

    pub fn report_cid(&self) -> Result<String, CorpusInducedDocumentSpinError> {
        Ok(blake3_label(&self.canonical_bytes()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ExactStateWire {
    h4_table_offset: u16,
    h4_coordinate: [[i64; 2]; 4],
    fiber_q29: i64,
    torsion_q29: i64,
}

#[derive(Debug, Clone)]
struct Prototype {
    token: u32,
    payload_cid: String,
    state: ExactSpinState,
    construction_document_support: u32,
    distinct_state_support: u32,
    fit_audit: PrototypeFitAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct PrototypeFitAudit {
    h4_minimum_objective_sum: u128,
    h4_minimizer_count: u16,
    fiber_minimum_objective_sum: u128,
    fiber_minimizer_count: u32,
    torsion_minimum_objective_sum: u128,
    torsion_minimizer_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PrototypeWire {
    token: u32,
    payload_cid: String,
    state: ExactStateWire,
    construction_document_support: u32,
    distinct_state_support: u32,
    fit_audit: PrototypeFitAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactWire {
    schema: u32,
    domain: String,
    corpus_cid: String,
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_set_kappa: String,
    leaf_basis_kappa: String,
    induction_policy_kappa: String,
    query_policy_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    phase_modulus_q29: i64,
    phase_half_q29: i64,
    max_artifact_bytes: usize,
    stats: ArtifactStatsWire,
    prototypes: Vec<PrototypeWire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactStatsWire {
    construction_documents: u64,
    eligible_construction_positions: u64,
    retained_document_exemplars: u64,
    observed_candidate_tokens: u64,
    usable_prototypes: u64,
    rejected_single_document_prototypes: u64,
    rejected_single_state_prototypes: u64,
}

#[derive(Debug, Clone)]
pub struct CorpusInducedDocumentSpinPlacementR4V1 {
    operator_cid: String,
    corpus_cid: String,
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_set_kappa: String,
    leaf_basis_kappa: String,
    induction_policy_kappa: String,
    query_policy_kappa: String,
    h4_table: H4BinaryIcosahedralClosure,
    max_token_id: u32,
    leaves: Vec<ExactSpinState>,
    prototypes: BTreeMap<u32, Prototype>,
    stats: CorpusInducedDocumentSpinArtifactStats,
}

#[derive(Debug, Clone)]
pub struct CorpusInducedDocumentSpinAntiRecallIndex {
    operator_cid: String,
    construction_set_kappa: String,
    full_prefix_cids: BTreeSet<[u8; 32]>,
    natural_states: BTreeSet<ExactStateWire>,
    operative_signature_cids: BTreeSet<[u8; 32]>,
    index_kappa: String,
}

#[cfg(test)]
#[derive(Serialize)]
struct AntiRecallIndexWire<'a> {
    schema: u32,
    domain: &'static str,
    operator_cid: &'a str,
    construction_set_kappa: &'a str,
    full_prefix_cids: &'a BTreeSet<[u8; 32]>,
    natural_states: &'a BTreeSet<ExactStateWire>,
    operative_signature_cids: &'a BTreeSet<[u8; 32]>,
    index_kappa: &'a str,
}

#[derive(Debug, Default)]
struct AntiRecallDocumentScan {
    full_prefix_cids: BTreeSet<[u8; 32]>,
    natural_states: BTreeSet<ExactStateWire>,
    operative_signature_cids: BTreeSet<[u8; 32]>,
}

#[derive(Debug, Clone)]
struct ConstructionExemplar {
    document_id: String,
    state: ExactSpinState,
}

#[derive(Debug)]
struct DocumentCompilation {
    eligible_positions: u64,
    exemplars: Vec<(u32, ConstructionExemplar)>,
}

#[derive(Debug)]
enum CandidateCompilation {
    Usable(Prototype),
    RejectedSingleDocument,
    RejectedSingleState,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ActiveRowKey {
    order: u8,
    previous_two: u32,
    previous_one: u32,
}

#[derive(Debug, Clone)]
struct TargetFreeCandidate {
    position: CorpusInducedDocumentSpinCensusPosition,
    row: ActiveRowKey,
    prediction: MatchedCorpusInducedDocumentSpinPrediction,
    natural_state: ExactStateWire,
    omega_cid: [u8; 32],
    prefix_cid: [u8; 32],
}

#[derive(Debug)]
struct TargetFreeDocumentScan {
    document_id: String,
    admission_opportunities: u64,
    rows_seen: BTreeSet<ActiveRowKey>,
    candidates: Vec<TargetFreeCandidate>,
}

#[derive(Debug, Default)]
struct TargetFreeScanAggregate {
    admission_opportunities: u64,
    row_documents: BTreeMap<ActiveRowKey, BTreeSet<String>>,
    candidates: Vec<TargetFreeCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScoreFrameOrigin {
    CausalPrefix,
    #[cfg(test)]
    InjectedHeldOutTarget,
}

#[derive(Debug, Clone, Copy)]
struct ExecutedQueryState {
    state: ExactSpinState,
    prefix_leaf_reads: u64,
    prefix_h4_product_reads: u64,
    prefix_phase_additions: u64,
}

#[derive(Debug, Clone, Copy)]
struct CausalScoreFrame {
    origin: ScoreFrameOrigin,
    prefix_units: u64,
    real: ExecutedQueryState,
    scope_disabled: ExecutedQueryState,
    order_shuffled: ExecutedQueryState,
    operator_permuted: ExecutedQueryState,
}

#[derive(Debug, Clone, Copy)]
struct CausalPrefixCursor {
    frame: CausalScoreFrame,
}

impl CausalPrefixCursor {
    fn new(table: &H4BinaryIcosahedralClosure) -> Result<Self, CorpusInducedDocumentSpinError> {
        let identity = ExactSpinState::identity(table)?;
        let query = ExecutedQueryState {
            state: identity,
            prefix_leaf_reads: 0,
            prefix_h4_product_reads: 0,
            prefix_phase_additions: 0,
        };
        Ok(Self {
            frame: CausalScoreFrame {
                origin: ScoreFrameOrigin::CausalPrefix,
                prefix_units: 0,
                real: query,
                scope_disabled: query,
                order_shuffled: query,
                operator_permuted: query,
            },
        })
    }

    fn from_prefix(
        prefix: &[u32],
        leaves: &[ExactSpinState],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        let mut cursor = Self::new(table)?;
        for &token in prefix {
            cursor.advance(token, leaves, table)?;
        }
        Ok(cursor)
    }

    fn advance(
        &mut self,
        token: u32,
        leaves: &[ExactSpinState],
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        self.frame.prefix_units = checked_add_u64(self.frame.prefix_units, 1)?;
        advance_executed_query(&mut self.frame.real, token, leaves, table, false)?;
        advance_executed_query(&mut self.frame.scope_disabled, token, leaves, table, false)?;
        advance_executed_query(&mut self.frame.order_shuffled, token, leaves, table, true)?;
        advance_executed_query(
            &mut self.frame.operator_permuted,
            token,
            leaves,
            table,
            false,
        )?;
        Ok(())
    }

    const fn frame(self) -> CausalScoreFrame {
        self.frame
    }
}

fn advance_executed_query(
    query: &mut ExecutedQueryState,
    token: u32,
    leaves: &[ExactSpinState],
    table: &H4BinaryIcosahedralClosure,
    reverse: bool,
) -> Result<(), CorpusInducedDocumentSpinError> {
    let leaf = leaf_for_token(leaves, token)?;
    query.prefix_leaf_reads = checked_add_u64(query.prefix_leaf_reads, 1)?;
    query.state = if reverse {
        leaf.compose(query.state, table)?
    } else {
        query.state.compose(leaf, table)?
    };
    query.prefix_h4_product_reads = checked_add_u64(query.prefix_h4_product_reads, 1)?;
    query.prefix_phase_additions = checked_add_u64(query.prefix_phase_additions, 2)?;
    Ok(())
}

#[derive(Debug)]
struct ExecutedCandidateScore {
    support_token: u32,
    prototype_token: u32,
    prototype_state: ExactSpinState,
    relative_state: ExactSpinState,
    cost: BoundedGlobalExactSpinCost,
}

#[derive(Debug)]
struct ExecutedArm {
    decision: CorpusInducedDocumentSpinDecision,
    candidates: Vec<ExecutedCandidateScore>,
}

#[derive(Clone)]
struct PrefixDigestState(blake3::Hasher);

impl PrefixDigestState {
    fn new() -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(PREFIX_DOMAIN);
        Self(hasher)
    }

    fn append(&mut self, token: u32) {
        self.0.update(&token.to_le_bytes());
    }

    fn digest(&self) -> [u8; 32] {
        *self.0.clone().finalize().as_bytes()
    }
}

impl From<CorpusInducedDocumentSpinArtifactStats> for ArtifactStatsWire {
    fn from(stats: CorpusInducedDocumentSpinArtifactStats) -> Self {
        Self {
            construction_documents: stats.construction_documents,
            eligible_construction_positions: stats.eligible_construction_positions,
            retained_document_exemplars: stats.retained_document_exemplars,
            observed_candidate_tokens: stats.observed_candidate_tokens,
            usable_prototypes: stats.usable_prototypes,
            rejected_single_document_prototypes: stats.rejected_single_document_prototypes,
            rejected_single_state_prototypes: stats.rejected_single_state_prototypes,
        }
    }
}

impl From<ArtifactStatsWire> for CorpusInducedDocumentSpinArtifactStats {
    fn from(stats: ArtifactStatsWire) -> Self {
        Self {
            construction_documents: stats.construction_documents,
            eligible_construction_positions: stats.eligible_construction_positions,
            retained_document_exemplars: stats.retained_document_exemplars,
            observed_candidate_tokens: stats.observed_candidate_tokens,
            usable_prototypes: stats.usable_prototypes,
            rejected_single_document_prototypes: stats.rejected_single_document_prototypes,
            rejected_single_state_prototypes: stats.rejected_single_state_prototypes,
        }
    }
}

impl CorpusInducedDocumentSpinPlacementR4V1 {
    /// Compile the aggregate-only placement from the exact construction set
    /// that produced `table`.  Held-out documents are rejected by the table's
    /// binding check and are not accepted by this API.
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        Self::compile_with_worker_count(table, base_overlay, construction, None)
    }

    fn compile_with_worker_count(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
        worker_count: Option<usize>,
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        if !table.is_bound_to_construction_documents(construction) {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement compiler is not bound to the table's exact construction set".to_owned(),
            ));
        }
        if base_overlay.table_artifact_cid() != table.artifact_cid() {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "#953 overlay is not bound to the supplied source-free table".to_owned(),
            ));
        }
        if construction.is_empty() {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement construction set is empty".to_owned(),
            ));
        }

        let h4_table = validate_h4_binary_icosahedral_closure()
            .map_err(|error| CorpusInducedDocumentSpinError::ExactSpin(error.to_string()))?;
        let max_token_id = table.maximum_token_id();
        let leaves = compile_identity_leaves(max_token_id, &h4_table)?;
        let table_artifact_cid = table.artifact_cid();
        let base_overlay_artifact_cid = base_overlay.artifact_cid();
        let construction_set_kappa = document_set_kappa(CONSTRUCTION_SET_DOMAIN, construction)?;
        if table_artifact_cid == FROZEN_TABLE_CID
            && base_overlay_artifact_cid == FROZEN_OVERLAY_CID
            && construction_set_kappa != FROZEN_CONSTRUCTION_SET_KAPPA
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "frozen construction-set kappa does not reproduce".to_owned(),
            ));
        }
        let corpus_cid = if table_artifact_cid == FROZEN_TABLE_CID
            && base_overlay_artifact_cid == FROZEN_OVERLAY_CID
        {
            FROZEN_CORPUS_CID.to_owned()
        } else {
            construction_set_kappa.clone()
        };
        let leaf_basis_kappa = identity_kappa(&[
            LEAF_BASIS_DOMAIN,
            LEAF_BASIS_IDENTITY,
            table_artifact_cid.as_str(),
            h4_table.h4_root_table_kappa.as_str(),
            h4_table.multiplication_table_kappa.as_str(),
        ]);
        let induction_policy_kappa = identity_kappa(&[
            INDUCTION_POLICY_DOMAIN,
            INDUCTION_POLICY_IDENTITY,
            table_artifact_cid.as_str(),
            base_overlay_artifact_cid.as_str(),
        ]);
        let query_policy_kappa = identity_kappa(&[
            QUERY_POLICY_DOMAIN,
            QUERY_POLICY_IDENTITY,
            table_artifact_cid.as_str(),
            base_overlay_artifact_cid.as_str(),
        ]);

        let mut ordered_documents = construction.iter().collect::<Vec<_>>();
        ordered_documents.sort_by(|left, right| left.id.cmp(&right.id));
        if ordered_documents
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement construction contains duplicate document IDs".to_owned(),
            ));
        }

        let document_compilations = compile_construction_documents(
            table,
            base_overlay,
            &ordered_documents,
            &leaves,
            &h4_table,
            worker_count,
        )?;
        let mut exemplars = BTreeMap::<u32, Vec<ConstructionExemplar>>::new();
        let mut eligible_construction_positions = 0_u64;
        for compilation in document_compilations {
            eligible_construction_positions = checked_add_u64(
                eligible_construction_positions,
                compilation.eligible_positions,
            )?;
            for (token, exemplar) in compilation.exemplars {
                exemplars.entry(token).or_default().push(exemplar);
            }
        }

        let observed_candidate_tokens = usize_u64(exemplars.len())?;
        let retained_document_exemplars = exemplars.values().try_fold(0_u64, |total, rows| {
            checked_add_u64(total, usize_u64(rows.len())?)
        })?;
        let exemplar_rows = exemplars.into_iter().collect::<Vec<_>>();
        let candidate_compilations =
            compile_candidate_prototypes(table, &exemplar_rows, &h4_table, worker_count)?;
        let mut rejected_single_document_prototypes = 0_u64;
        let mut rejected_single_state_prototypes = 0_u64;
        let mut prototypes = BTreeMap::new();
        for compilation in candidate_compilations {
            match compilation {
                CandidateCompilation::Usable(prototype) => {
                    prototypes.insert(prototype.token, prototype);
                }
                CandidateCompilation::RejectedSingleDocument => {
                    rejected_single_document_prototypes =
                        checked_add_u64(rejected_single_document_prototypes, 1)?;
                }
                CandidateCompilation::RejectedSingleState => {
                    rejected_single_state_prototypes =
                        checked_add_u64(rejected_single_state_prototypes, 1)?;
                }
            }
        }

        let stats = CorpusInducedDocumentSpinArtifactStats {
            construction_documents: usize_u64(ordered_documents.len())?,
            eligible_construction_positions,
            retained_document_exemplars,
            observed_candidate_tokens,
            usable_prototypes: usize_u64(prototypes.len())?,
            rejected_single_document_prototypes,
            rejected_single_state_prototypes,
        };
        let mut operator = Self {
            operator_cid: String::new(),
            corpus_cid,
            table_artifact_cid,
            base_overlay_artifact_cid,
            construction_set_kappa,
            leaf_basis_kappa,
            induction_policy_kappa,
            query_policy_kappa,
            h4_table,
            max_token_id,
            leaves,
            prototypes,
            stats,
        };
        operator.validate_binding(table, base_overlay)?;
        let artifact_bytes = operator.to_bytes()?;
        if artifact_bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "corpus-induced placement artifact exceeds its byte ceiling".to_owned(),
            ));
        }
        operator.operator_cid = blake3_label(&artifact_bytes);
        Ok(operator)
    }

    pub fn from_bytes(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        expected_operator_cid: &str,
        bytes: &[u8],
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        if bytes.len() < ARTIFACT_MAGIC.len()
            || bytes.len() > MAX_ARTIFACT_BYTES
            || bytes[..ARTIFACT_MAGIC.len()] != ARTIFACT_MAGIC
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "corpus-induced placement artifact magic or size is invalid".to_owned(),
            ));
        }
        if blake3_label(bytes) != expected_operator_cid {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement artifact does not match its trusted expected CID".to_owned(),
            ));
        }
        let wire: ArtifactWire = serde_json::from_slice(&bytes[ARTIFACT_MAGIC.len()..])
            .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
        let h4_table = validate_h4_binary_icosahedral_closure()
            .map_err(|error| CorpusInducedDocumentSpinError::ExactSpin(error.to_string()))?;
        if wire.schema != ARTIFACT_SCHEMA
            || wire.domain != ARTIFACT_DOMAIN
            || wire.phase_modulus_q29 != PHASE_MODULUS_Q29
            || wire.phase_half_q29 != PHASE_HALF_Q29
            || wire.max_artifact_bytes != MAX_ARTIFACT_BYTES
            || wire.h4_root_table_kappa != h4_table.h4_root_table_kappa
            || wire.h4_multiplication_table_kappa != h4_table.multiplication_table_kappa
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement artifact schema, constants, or exact H4 table binding is invalid"
                    .to_owned(),
            ));
        }
        if wire
            .prototypes
            .windows(2)
            .any(|pair| pair[0].token >= pair[1].token)
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement prototype rows are not in unique canonical token order".to_owned(),
            ));
        }
        let max_token_id = table.maximum_token_id();
        let leaves = compile_identity_leaves(max_token_id, &h4_table)?;
        let mut prototypes = BTreeMap::new();
        for row in &wire.prototypes {
            if row.token > max_token_id
                || !table.is_fitted_lexical_token(row.token)
                || prototypes.contains_key(&row.token)
            {
                return Err(CorpusInducedDocumentSpinError::Invalid(
                    "placement prototype token is duplicated or out of range".to_owned(),
                ));
            }
            let state = exact_state_from_wire(&row.state, &h4_table)?;
            let payload_cid = blake3_label(&table.decode_tokens(&[row.token])?);
            if row.payload_cid != payload_cid
                || row.construction_document_support < MIN_PROTOTYPE_DOCUMENTS
                || row.distinct_state_support < MIN_DISTINCT_PROTOTYPE_STATES
                || row.fit_audit.h4_minimizer_count == 0
                || usize::from(row.fit_audit.h4_minimizer_count) > h4_table.root_count
                || row.fit_audit.fiber_minimizer_count == 0
                || row.fit_audit.fiber_minimizer_count > row.distinct_state_support
                || row.fit_audit.torsion_minimizer_count == 0
                || row.fit_audit.torsion_minimizer_count > row.distinct_state_support
            {
                return Err(CorpusInducedDocumentSpinError::Invalid(
                    "placement prototype payload or support witness is invalid".to_owned(),
                ));
            }
            prototypes.insert(
                row.token,
                Prototype {
                    token: row.token,
                    payload_cid: row.payload_cid.clone(),
                    state,
                    construction_document_support: row.construction_document_support,
                    distinct_state_support: row.distinct_state_support,
                    fit_audit: row.fit_audit,
                },
            );
        }
        let stats: CorpusInducedDocumentSpinArtifactStats = wire.stats.into();
        let classified = stats
            .usable_prototypes
            .checked_add(stats.rejected_single_document_prototypes)
            .and_then(|value| value.checked_add(stats.rejected_single_state_prototypes))
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        if stats.construction_documents == 0
            || stats.usable_prototypes != usize_u64(prototypes.len())?
            || stats.observed_candidate_tokens != classified
            || prototypes.values().any(|prototype| {
                u64::from(prototype.construction_document_support) > stats.construction_documents
                    || prototype.distinct_state_support > prototype.construction_document_support
            })
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement artifact aggregate statistics are inconsistent".to_owned(),
            ));
        }
        let expected_table_cid = table.artifact_cid();
        let expected_overlay_cid = base_overlay.artifact_cid();
        let expected_corpus_cid = if expected_table_cid == FROZEN_TABLE_CID
            && expected_overlay_cid == FROZEN_OVERLAY_CID
        {
            FROZEN_CORPUS_CID
        } else {
            wire.construction_set_kappa.as_str()
        };
        if wire.table_artifact_cid != expected_table_cid
            || wire.base_overlay_artifact_cid != expected_overlay_cid
            || wire.corpus_cid != expected_corpus_cid
            || (wire.table_artifact_cid == FROZEN_TABLE_CID
                && wire.base_overlay_artifact_cid == FROZEN_OVERLAY_CID
                && wire.construction_set_kappa != FROZEN_CONSTRUCTION_SET_KAPPA)
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement artifact corpus, table, or overlay identity is invalid".to_owned(),
            ));
        }
        let operator = Self {
            operator_cid: blake3_label(bytes),
            corpus_cid: wire.corpus_cid,
            table_artifact_cid: wire.table_artifact_cid,
            base_overlay_artifact_cid: wire.base_overlay_artifact_cid,
            construction_set_kappa: wire.construction_set_kappa,
            leaf_basis_kappa: wire.leaf_basis_kappa,
            induction_policy_kappa: wire.induction_policy_kappa,
            query_policy_kappa: wire.query_policy_kappa,
            h4_table,
            max_token_id,
            leaves,
            prototypes,
            stats,
        };
        operator.validate_binding(table, base_overlay)?;
        if operator.to_bytes()? != bytes {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement artifact is noncanonical or binding-drifted".to_owned(),
            ));
        }
        Ok(operator)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CorpusInducedDocumentSpinError> {
        let prototypes = self
            .prototypes
            .values()
            .map(|prototype| {
                Ok(PrototypeWire {
                    token: prototype.token,
                    payload_cid: prototype.payload_cid.clone(),
                    state: exact_state_wire(prototype.state, &self.h4_table)?,
                    construction_document_support: prototype.construction_document_support,
                    distinct_state_support: prototype.distinct_state_support,
                    fit_audit: prototype.fit_audit,
                })
            })
            .collect::<Result<Vec<_>, CorpusInducedDocumentSpinError>>()?;
        let wire = ArtifactWire {
            schema: ARTIFACT_SCHEMA,
            domain: ARTIFACT_DOMAIN.to_owned(),
            corpus_cid: self.corpus_cid.clone(),
            table_artifact_cid: self.table_artifact_cid.clone(),
            base_overlay_artifact_cid: self.base_overlay_artifact_cid.clone(),
            construction_set_kappa: self.construction_set_kappa.clone(),
            leaf_basis_kappa: self.leaf_basis_kappa.clone(),
            induction_policy_kappa: self.induction_policy_kappa.clone(),
            query_policy_kappa: self.query_policy_kappa.clone(),
            h4_root_table_kappa: self.h4_table.h4_root_table_kappa.clone(),
            h4_multiplication_table_kappa: self.h4_table.multiplication_table_kappa.clone(),
            phase_modulus_q29: PHASE_MODULUS_Q29,
            phase_half_q29: PHASE_HALF_Q29,
            max_artifact_bytes: MAX_ARTIFACT_BYTES,
            stats: self.stats.into(),
            prototypes,
        };
        let payload = serde_json::to_vec(&wire)
            .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
        let mut bytes = Vec::with_capacity(ARTIFACT_MAGIC.len() + payload.len());
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        bytes.extend_from_slice(&payload);
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "corpus-induced placement artifact exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn artifact_cid(&self) -> Result<String, CorpusInducedDocumentSpinError> {
        Ok(self.operator_cid.clone())
    }

    pub fn table_artifact_cid(&self) -> &str {
        &self.table_artifact_cid
    }

    pub fn base_overlay_artifact_cid(&self) -> &str {
        &self.base_overlay_artifact_cid
    }

    pub fn construction_set_kappa(&self) -> &str {
        &self.construction_set_kappa
    }

    pub fn leaf_basis_kappa(&self) -> &str {
        &self.leaf_basis_kappa
    }

    pub fn induction_policy_kappa(&self) -> &str {
        &self.induction_policy_kappa
    }

    pub fn query_policy_kappa(&self) -> &str {
        &self.query_policy_kappa
    }

    pub fn stats(&self) -> CorpusInducedDocumentSpinArtifactStats {
        self.stats
    }

    pub fn prototype_traces(
        &self,
    ) -> Result<Vec<CorpusInducedDocumentSpinPrototypeTrace>, CorpusInducedDocumentSpinError> {
        self.prototypes
            .values()
            .map(|prototype| {
                Ok(CorpusInducedDocumentSpinPrototypeTrace {
                    token: prototype.token,
                    payload_cid: prototype.payload_cid.clone(),
                    state: prototype.state.trace(&self.h4_table)?,
                    construction_document_support: prototype.construction_document_support,
                    distinct_state_support: prototype.distinct_state_support,
                    h4_minimum_objective_sum: prototype.fit_audit.h4_minimum_objective_sum,
                    h4_minimizer_count: prototype.fit_audit.h4_minimizer_count,
                    fiber_minimum_objective_sum: prototype.fit_audit.fiber_minimum_objective_sum,
                    fiber_minimizer_count: prototype.fit_audit.fiber_minimizer_count,
                    torsion_minimum_objective_sum: prototype
                        .fit_audit
                        .torsion_minimum_objective_sum,
                    torsion_minimizer_count: prototype.fit_audit.torsion_minimizer_count,
                })
            })
            .collect()
    }

    fn validate_binding(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        if self.table_artifact_cid != table.artifact_cid()
            || self.base_overlay_artifact_cid != base_overlay.artifact_cid()
            || base_overlay.table_artifact_cid() != table.artifact_cid()
            || self.max_token_id != table.maximum_token_id()
            || self.leaves.len()
                != usize::try_from(self.max_token_id)
                    .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?
                    .checked_add(1)
                    .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement table, overlay, token namespace, or leaf binding mismatches".to_owned(),
            ));
        }
        let expected_leaf_basis = identity_kappa(&[
            LEAF_BASIS_DOMAIN,
            LEAF_BASIS_IDENTITY,
            self.table_artifact_cid.as_str(),
            self.h4_table.h4_root_table_kappa.as_str(),
            self.h4_table.multiplication_table_kappa.as_str(),
        ]);
        let expected_induction = identity_kappa(&[
            INDUCTION_POLICY_DOMAIN,
            INDUCTION_POLICY_IDENTITY,
            self.table_artifact_cid.as_str(),
            self.base_overlay_artifact_cid.as_str(),
        ]);
        let expected_query = identity_kappa(&[
            QUERY_POLICY_DOMAIN,
            QUERY_POLICY_IDENTITY,
            self.table_artifact_cid.as_str(),
            self.base_overlay_artifact_cid.as_str(),
        ]);
        if self.leaf_basis_kappa != expected_leaf_basis
            || self.induction_policy_kappa != expected_induction
            || self.query_policy_kappa != expected_query
            || self.stats.usable_prototypes != usize_u64(self.prototypes.len())?
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "placement policy identity or prototype census does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parallel_construction_fixture() -> Vec<SourceDocument> {
        const CANDIDATES: usize = 12;
        const DOCUMENTS_PER_CANDIDATE: usize = 2;

        let mut documents = Vec::with_capacity(CANDIDATES * DOCUMENTS_PER_CANDIDATE);
        let mut next_id = 0_u64;
        for candidate in 0..CANDIDATES {
            for replica in 0..DOCUMENTS_PER_CANDIDATE {
                let id = loop {
                    let id = format!("worker-fixture-{next_id}");
                    next_id += 1;
                    if !crate::source_free_table::d3_is_held_out(&id) {
                        break id;
                    }
                };
                documents.push(SourceDocument::new(
                    id,
                    format!("Prelude{replica} common anchor choice{candidate}.").into_bytes(),
                ));
            }
        }
        documents
    }

    fn parallel_held_out_fixture() -> Vec<SourceDocument> {
        const DOCUMENTS: usize = 16;

        let mut held_out = Vec::with_capacity(DOCUMENTS);
        let mut next_id = 0_u64;
        while held_out.len() < DOCUMENTS {
            let id = format!("heldout-worker-fixture-{next_id}");
            next_id += 1;
            if crate::source_free_table::d3_is_held_out(&id) {
                let document = held_out.len();
                held_out.push(SourceDocument::new(
                    id,
                    format!("Heldout{document} common anchor choice{}.", document % 12)
                        .into_bytes(),
                ));
            }
        }
        held_out
    }

    fn target_mutation_fixture() -> [SourceDocument; 2] {
        let mut ids = Vec::with_capacity(2);
        let mut next_id = 0_u64;
        while ids.len() < 2 {
            let id = format!("heldout-target-mutation-{next_id}");
            next_id += 1;
            if crate::source_free_table::d3_is_held_out(&id) {
                ids.push(id);
            }
        }
        [
            SourceDocument::new(ids.remove(0), b"Probe common anchor choice0.".to_vec()),
            SourceDocument::new(ids.remove(0), b"Probe common anchor choice1.".to_vec()),
        ]
    }

    fn small_construction_fixture() -> Vec<SourceDocument> {
        [
            ("14", "The red fox rests."),
            ("657", "At noon the red fox rests."),
            ("4579", "The red fox runs."),
            ("5121", "At dusk the red fox runs."),
        ]
        .into_iter()
        .map(|(id, text)| SourceDocument::new(id, text.as_bytes().to_vec()))
        .collect()
    }

    fn brute_force_circular_medoid(values: &[i64]) -> (i64, u128, u32) {
        let mut candidates = values.to_vec();
        candidates.sort_unstable();
        candidates.dedup();
        let mut best = None;
        for candidate in candidates {
            let objective = values.iter().fold(0_u128, |sum, &value| {
                sum + u128::from(
                    crate::bounded_global_exact_spin_attention::circular_abs_q29(value - candidate),
                )
            });
            match best {
                None => best = Some((candidate, objective, 1_u32)),
                Some((_, current, _)) if objective < current => {
                    best = Some((candidate, objective, 1));
                }
                Some((selected, current, count)) if objective == current => {
                    best = Some((selected, current, count + 1));
                }
                Some(_) => {}
            }
        }
        best.expect("nonempty medoid fixture")
    }

    #[test]
    fn one_and_eight_worker_compiles_have_identical_canonical_bytes(
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        let construction = parallel_construction_fixture();
        let table = SourceFreeTable::compile(&construction)?;
        let overlay = MultiscaleCountRadiusR4V1::compile(&table)?;
        let serial = CorpusInducedDocumentSpinPlacementR4V1::compile_with_worker_count(
            &table,
            &overlay,
            &construction,
            Some(1),
        )?;
        let parallel = CorpusInducedDocumentSpinPlacementR4V1::compile_with_worker_count(
            &table,
            &overlay,
            &construction,
            Some(8),
        )?;

        assert!(
            serial.stats().usable_prototypes >= 8,
            "fixture must exercise candidate parallelism"
        );
        assert_eq!(serial.to_bytes()?, parallel.to_bytes()?);
        assert_eq!(serial.artifact_cid()?, parallel.artifact_cid()?);
        Ok(())
    }

    #[test]
    fn one_and_eight_worker_document_scans_are_canonical(
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        let construction = parallel_construction_fixture();
        let held_out = parallel_held_out_fixture();
        let table = SourceFreeTable::compile(&construction)?;
        let overlay = MultiscaleCountRadiusR4V1::compile(&table)?;
        let operator =
            CorpusInducedDocumentSpinPlacementR4V1::compile(&table, &overlay, &construction)?;
        let serial_index = CorpusInducedDocumentSpinAntiRecallIndex::compile_with_worker_count(
            &operator,
            &table,
            &overlay,
            &construction,
            Some(1),
        )?;
        let parallel_index = CorpusInducedDocumentSpinAntiRecallIndex::compile_with_worker_count(
            &operator,
            &table,
            &overlay,
            &construction,
            Some(8),
        )?;
        assert_eq!(serial_index.index_kappa(), parallel_index.index_kappa());
        assert_eq!(
            serial_index.canonical_bytes()?,
            parallel_index.canonical_bytes()?
        );

        let serial_census = operator.target_free_census_with_worker_count(
            &table,
            &overlay,
            &serial_index,
            &held_out,
            Some(1),
        )?;
        let parallel_census = operator.target_free_census_with_worker_count(
            &table,
            &overlay,
            &parallel_index,
            &held_out,
            Some(8),
        )?;
        assert!(serial_census.admission_opportunities > 0);
        assert_eq!(
            serial_census.canonical_bytes()?,
            parallel_census.canonical_bytes()?
        );
        assert_eq!(
            serial_census.artifact_cid()?,
            parallel_census.artifact_cid()?
        );
        assert_eq!(serial_census.forbidden_reads.held_out_target_reads, 0);
        assert_eq!(parallel_census.forbidden_reads.held_out_target_reads, 0);
        Ok(())
    }

    #[test]
    fn target_free_scan_freezes_prediction_before_current_target(
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        let construction = parallel_construction_fixture();
        let table = SourceFreeTable::compile(&construction)?;
        let overlay = MultiscaleCountRadiusR4V1::compile(&table)?;
        let operator =
            CorpusInducedDocumentSpinPlacementR4V1::compile(&table, &overlay, &construction)?;
        let documents = target_mutation_fixture();
        let left_stream = table.encode_document_stream(&documents[0])?;
        let right_stream = table.encode_document_stream(&documents[1])?;
        let target_index = left_stream
            .iter()
            .zip(&right_stream)
            .position(|(left, right)| left != right)
            .expect("mutation fixture must contain one distinct current target");
        assert_eq!(&left_stream[..target_index], &right_stream[..target_index]);

        let left = scan_one_target_free_document(&operator, &table, &overlay, &documents[0])?;
        let right = scan_one_target_free_document(&operator, &table, &overlay, &documents[1])?;
        let left_candidate = left
            .candidates
            .iter()
            .find(|candidate| candidate.position.target_index as usize == target_index)
            .expect("left mutation target must be an admitted maximum-count tie");
        let right_candidate = right
            .candidates
            .iter()
            .find(|candidate| candidate.position.target_index as usize == target_index)
            .expect("right mutation target must be an admitted maximum-count tie");

        assert_eq!(left_candidate.prefix_cid, right_candidate.prefix_cid);
        assert_eq!(left_candidate.natural_state, right_candidate.natural_state);
        assert_eq!(left_candidate.omega_cid, right_candidate.omega_cid);
        assert_eq!(left_candidate.prediction, right_candidate.prediction);
        assert_eq!(
            left_candidate.position.prediction_cid,
            right_candidate.position.prediction_cid
        );
        assert_eq!(
            left_candidate
                .prediction
                .forbidden_reads
                .held_out_target_reads,
            0
        );
        assert_eq!(
            right_candidate
                .prediction
                .forbidden_reads
                .held_out_target_reads,
            0
        );
        Ok(())
    }

    #[test]
    fn trusted_cid_and_single_byte_tamper_are_rejected(
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        let construction = small_construction_fixture();
        let table = SourceFreeTable::compile(&construction)?;
        let overlay = MultiscaleCountRadiusR4V1::compile(&table)?;
        let operator =
            CorpusInducedDocumentSpinPlacementR4V1::compile(&table, &overlay, &construction)?;
        let bytes = operator.to_bytes()?;
        let expected_cid = operator.artifact_cid()?;
        let wrong_cid = format!("blake3:{}", "00".repeat(32));

        let wrong_cid_error = CorpusInducedDocumentSpinPlacementR4V1::from_bytes(
            &table, &overlay, &wrong_cid, &bytes,
        )
        .expect_err("trusted CID mismatch must reject canonical bytes");
        assert!(wrong_cid_error.to_string().contains("trusted expected CID"));

        let mut tampered = bytes.clone();
        tampered[ARTIFACT_MAGIC.len()] ^= 1;
        let trusted_tamper_error = CorpusInducedDocumentSpinPlacementR4V1::from_bytes(
            &table,
            &overlay,
            &expected_cid,
            &tampered,
        )
        .expect_err("single-byte tamper must fail the trusted CID");
        assert!(trusted_tamper_error
            .to_string()
            .contains("trusted expected CID"));

        let tampered_cid = blake3_label(&tampered);
        assert!(CorpusInducedDocumentSpinPlacementR4V1::from_bytes(
            &table,
            &overlay,
            &tampered_cid,
            &tampered,
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn target_derived_score_frame_is_rejected() {
        let error = injected_target_score_firewall_control()
            .expect_err("target-derived query state must not receive a score certificate");
        assert!(error.to_string().contains("score firewall rejected"));
    }

    #[test]
    fn imbalanced_executed_prefix_work_is_rejected() {
        let error = imbalanced_work_score_firewall_control()
            .expect_err("imbalanced executed-prefix work must not receive a score certificate");
        assert!(error.to_string().contains("imbalanced executed-prefix"));
    }

    #[test]
    fn optimized_circular_medoid_matches_brute_force_at_boundaries(
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        let mut cases = vec![
            vec![0],
            vec![-PHASE_HALF_Q29, PHASE_HALF_Q29 - 1],
            vec![-PHASE_HALF_Q29, 0, PHASE_HALF_Q29 - 1],
            vec![-PHASE_HALF_Q29, -PHASE_HALF_Q29, PHASE_HALF_Q29 - 1],
            vec![-1, 0, 1],
            vec![-7, -7, 11, 11],
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for length in 1..=32_usize {
            let mut values = Vec::with_capacity(length);
            for _ in 0..length {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                values.push(
                    i64::try_from(state % PHASE_MODULUS_Q29 as u64)
                        .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?
                        - PHASE_HALF_Q29,
                );
            }
            cases.push(values);
        }

        for values in cases {
            assert_eq!(
                circular_medoid(values.iter().copied())?,
                brute_force_circular_medoid(&values),
                "optimized circular medoid drifted for {values:?}",
            );
        }
        Ok(())
    }
}

#[derive(Default)]
struct ComparatorAccumulator {
    wins: u64,
    losses: u64,
    ties: u64,
}

impl ComparatorAccumulator {
    fn observe(
        &mut self,
        real_correct: bool,
        comparator_correct: bool,
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        match (real_correct, comparator_correct) {
            (true, false) => self.wins = checked_add_u64(self.wins, 1)?,
            (false, true) => self.losses = checked_add_u64(self.losses, 1)?,
            _ => self.ties = checked_add_u64(self.ties, 1)?,
        }
        Ok(())
    }

    fn observe_counts(
        &mut self,
        real_correct: u64,
        comparator_correct: u64,
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        match real_correct.cmp(&comparator_correct) {
            std::cmp::Ordering::Greater => self.wins = checked_add_u64(self.wins, 1)?,
            std::cmp::Ordering::Less => self.losses = checked_add_u64(self.losses, 1)?,
            std::cmp::Ordering::Equal => self.ties = checked_add_u64(self.ties, 1)?,
        }
        Ok(())
    }

    fn finish(
        self,
    ) -> Result<CorpusInducedDocumentSpinComparatorEvaluation, CorpusInducedDocumentSpinError> {
        let discordant = checked_add_u64(self.wins, self.losses)?;
        let exact_threshold_passes = exact_one_sided_sign_test_passes(self.wins, self.losses)?;
        let passes = self.wins > self.losses && exact_threshold_passes;
        Ok(CorpusInducedDocumentSpinComparatorEvaluation {
            wins: self.wins,
            losses: self.losses,
            ties: self.ties,
            discordant,
            one_sided_exact_sign_test: format!(
                "20*sum_{{k=0}}^{{{}}} C({discordant},k) <= 2^{discordant}: {exact_threshold_passes}",
                self.losses,
            ),
            passes,
            terminal: if passes {
                "PASS_DIRECTIONAL_EXACT_SIGN_TEST"
            } else {
                "FAIL_DIRECTIONAL_EXACT_SIGN_TEST"
            }
            .to_owned(),
        })
    }
}

impl CorpusInducedDocumentSpinPlacementR4V1 {
    /// Perform the single authorized next-route join against an already-frozen
    /// target-free census.
    pub fn evaluate_held_out(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        anti_recall: &CorpusInducedDocumentSpinAntiRecallIndex,
        census: &CorpusInducedDocumentSpinTargetFreeCensus,
        held_out: &[SourceDocument],
    ) -> Result<CorpusInducedDocumentSpinEvaluation, CorpusInducedDocumentSpinError> {
        self.validate_binding(table, base_overlay)?;
        if !table.is_disjoint_d3_held_out_documents(held_out)
            || census.schema != ARTIFACT_SCHEMA
            || census.domain != ANTI_RECALL_DOMAIN
            || census.operator_cid != self.operator_cid
            || census.held_out_set_kappa != document_set_kappa(HELD_OUT_SET_DOMAIN, held_out)?
            || census.held_out_documents != usize_u64(held_out.len())?
            || census.invalid_score_firewall_certificates != 0
            || census.score_firewall_policy_kappa != identity_kappa(&[SCORE_FIREWALL_POLICY])
            || census.forbidden_reads.total() != 0
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "held-out evaluation is not bound to the frozen target-free census".to_owned(),
            ));
        }
        let reproduced = self.target_free_census(table, base_overlay, anti_recall, held_out)?;
        if reproduced.canonical_bytes()? != census.canonical_bytes()? {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "target-free census does not reproduce byte-identically before target join"
                    .to_owned(),
            ));
        }
        if !census.meets_frozen_preflight {
            let empty = ComparatorAccumulator::default().finish()?;
            return Ok(CorpusInducedDocumentSpinEvaluation {
                schema: ARTIFACT_SCHEMA,
                domain: "uor-r4.corpus-induced-document-spin-evaluation/1".to_owned(),
                decision: UNAVAILABLE_TERMINAL.to_owned(),
                operator_cid: self.operator_cid.clone(),
                census_cid: census.artifact_cid()?,
                held_out_set_kappa: census.held_out_set_kappa.clone(),
                held_out_documents: census.held_out_documents,
                post_join_admission_opportunities: 0,
                post_join_known_target_admission_opportunities: 0,
                operative_positions: usize_u64(census.operative_positions.len())?,
                operative_known_target_positions: 0,
                real_correct: 0,
                scope_disabled_correct: 0,
                order_shuffled_correct: 0,
                operator_permuted_correct: 0,
                witness_continuation_contrast: false,
                witness_continuation: None,
                witness_continuation_cid: None,
                versus_scope_disabled: empty.clone(),
                versus_order_shuffled: empty.clone(),
                versus_operator_permuted: empty.clone(),
                document_blocked_versus_scope_disabled: empty.clone(),
                document_blocked_versus_order_shuffled: empty.clone(),
                document_blocked_versus_operator_permuted: empty,
                target_reads: 0,
                forbidden_reads: census.forbidden_reads,
            });
        }
        let mut operative =
            BTreeMap::<(String, u32), &CorpusInducedDocumentSpinCensusPosition>::new();
        for position in &census.operative_positions {
            if operative
                .insert(
                    (position.document_id.clone(), position.target_index),
                    position,
                )
                .is_some()
            {
                return Err(CorpusInducedDocumentSpinError::Invalid(
                    "target-free census contains a duplicate operative position".to_owned(),
                ));
            }
        }
        let census_cid = census.artifact_cid()?;
        let witness_key = census
            .frozen_decoded_witness
            .as_ref()
            .map(|position| (position.document_id.clone(), position.target_index));
        let mut witness_prefix = None;
        let mut documents = held_out.iter().collect::<Vec<_>>();
        documents.sort_by(|left, right| left.id.cmp(&right.id));
        let mut post_join_admission_opportunities = 0_u64;
        let mut post_join_known_target_admission_opportunities = 0_u64;
        let mut operative_positions = 0_u64;
        let mut operative_known_target_positions = 0_u64;
        let mut real_correct = 0_u64;
        let mut scope_disabled_correct = 0_u64;
        let mut order_shuffled_correct = 0_u64;
        let mut operator_permuted_correct = 0_u64;
        let mut target_reads = 0_u64;
        let mut evaluation_forbidden_reads = CorpusInducedDocumentSpinForbiddenReads::default();
        let mut versus_scope_disabled = ComparatorAccumulator::default();
        let mut versus_order_shuffled = ComparatorAccumulator::default();
        let mut versus_operator_permuted = ComparatorAccumulator::default();
        let mut document_blocked_versus_scope_disabled = ComparatorAccumulator::default();
        let mut document_blocked_versus_order_shuffled = ComparatorAccumulator::default();
        let mut document_blocked_versus_operator_permuted = ComparatorAccumulator::default();
        for document in &documents {
            let mut document_real_correct = 0_u64;
            let mut document_scope_disabled_correct = 0_u64;
            let mut document_order_shuffled_correct = 0_u64;
            let mut document_operator_permuted_correct = 0_u64;
            let stream = table.encode_document_stream(document)?;
            let mut cursor = CausalPrefixCursor::new(&self.h4_table)?;
            cursor.advance(BOS_TOKEN, &self.leaves, &self.h4_table)?;
            let mut prefix_hash = PrefixDigestState::new();
            prefix_hash.append(BOS_TOKEN);
            for target_index in 1..stream.len() {
                let prefix = &stream[..target_index];
                let local = table.predict_multiscale_count_radius(prefix, base_overlay)?;
                let prediction = active_maximum_tie(&local)
                    .then(|| self.predict_from_frame(local, cursor.frame()))
                    .transpose()?;
                // This is the only label-bearing held-out read in this API.
                let target = stream[target_index];
                target_reads = checked_add_u64(target_reads, 1)?;
                if let Some(prediction) = prediction {
                    evaluation_forbidden_reads.saturating_accumulate(prediction.forbidden_reads);
                    post_join_admission_opportunities =
                        checked_add_u64(post_join_admission_opportunities, 1)?;
                    let fitted_target = table.is_fitted_lexical_token(target);
                    if fitted_target {
                        post_join_known_target_admission_opportunities =
                            checked_add_u64(post_join_known_target_admission_opportunities, 1)?;
                    }
                    let target_index_u32 = u32::try_from(target_index)
                        .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
                    if let Some(expected) = operative.get(&(document.id.clone(), target_index_u32))
                    {
                        let row = active_row_key(prefix, prediction.local.order)?;
                        let actual = CorpusInducedDocumentSpinCensusPosition {
                            document_id: document.id.clone(),
                            target_index: target_index_u32,
                            prefix_cid: digest_label(prefix_hash.digest()),
                            row_cid: row_label(&row),
                            support_cid: support_label(&prediction.real.support_tokens),
                            prediction_cid: prediction_label(&prediction)?,
                            real_token: prediction.real.token,
                            scope_disabled_token: prediction.scope_disabled.token,
                            order_shuffled_token: prediction.order_shuffled.token,
                            operator_permuted_token: prediction.operator_permuted.token,
                        };
                        if actual != **expected {
                            return Err(CorpusInducedDocumentSpinError::Invalid(
                                "operative prediction drifted after target attachment".to_owned(),
                            ));
                        }
                        if witness_key.as_ref() == Some(&(document.id.clone(), target_index_u32)) {
                            witness_prefix = Some(prefix.to_vec());
                        }
                        operative_positions = checked_add_u64(operative_positions, 1)?;
                        if fitted_target {
                            operative_known_target_positions =
                                checked_add_u64(operative_known_target_positions, 1)?;
                            let real_hit = prediction.real.token == target;
                            let scope_hit = prediction.scope_disabled.token == target;
                            let order_hit = prediction.order_shuffled.token == target;
                            let permuted_hit = prediction.operator_permuted.token == target;
                            if real_hit {
                                real_correct = checked_add_u64(real_correct, 1)?;
                                document_real_correct = checked_add_u64(document_real_correct, 1)?;
                            }
                            if scope_hit {
                                scope_disabled_correct =
                                    checked_add_u64(scope_disabled_correct, 1)?;
                                document_scope_disabled_correct =
                                    checked_add_u64(document_scope_disabled_correct, 1)?;
                            }
                            if order_hit {
                                order_shuffled_correct =
                                    checked_add_u64(order_shuffled_correct, 1)?;
                                document_order_shuffled_correct =
                                    checked_add_u64(document_order_shuffled_correct, 1)?;
                            }
                            if permuted_hit {
                                operator_permuted_correct =
                                    checked_add_u64(operator_permuted_correct, 1)?;
                                document_operator_permuted_correct =
                                    checked_add_u64(document_operator_permuted_correct, 1)?;
                            }
                            versus_scope_disabled.observe(real_hit, scope_hit)?;
                            versus_order_shuffled.observe(real_hit, order_hit)?;
                            versus_operator_permuted.observe(real_hit, permuted_hit)?;
                        }
                    }
                } else if operative.contains_key(&(
                    document.id.clone(),
                    u32::try_from(target_index)
                        .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
                )) {
                    return Err(CorpusInducedDocumentSpinError::Invalid(
                        "operative census position is no longer admitted".to_owned(),
                    ));
                }
                cursor.advance(target, &self.leaves, &self.h4_table)?;
                prefix_hash.append(target);
            }
            document_blocked_versus_scope_disabled
                .observe_counts(document_real_correct, document_scope_disabled_correct)?;
            document_blocked_versus_order_shuffled
                .observe_counts(document_real_correct, document_order_shuffled_correct)?;
            document_blocked_versus_operator_permuted
                .observe_counts(document_real_correct, document_operator_permuted_correct)?;
        }
        if operative_positions != usize_u64(census.operative_positions.len())?
            || post_join_admission_opportunities != census.admission_opportunities
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "post-join structural population differs from the target-free census".to_owned(),
            ));
        }
        if witness_key.is_some() && witness_prefix.is_none() {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "frozen decoded witness was not reproduced during target join".to_owned(),
            ));
        }
        let frozen_identity = self.corpus_cid == FROZEN_CORPUS_CID
            && self.table_artifact_cid == FROZEN_TABLE_CID
            && self.base_overlay_artifact_cid == FROZEN_OVERLAY_CID;
        if frozen_identity
            && (post_join_admission_opportunities != FROZEN_TARGET_FREE_ADMISSIONS
                || post_join_known_target_admission_opportunities != 76_641
                || usize_u64(documents.len())? != FROZEN_HELD_OUT_DOCUMENTS)
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "frozen 81,177/76,641 post-join population did not reproduce".to_owned(),
            ));
        }
        let versus_scope_disabled = versus_scope_disabled.finish()?;
        let versus_order_shuffled = versus_order_shuffled.finish()?;
        let versus_operator_permuted = versus_operator_permuted.finish()?;
        let document_blocked_versus_scope_disabled =
            document_blocked_versus_scope_disabled.finish()?;
        let document_blocked_versus_order_shuffled =
            document_blocked_versus_order_shuffled.finish()?;
        let document_blocked_versus_operator_permuted =
            document_blocked_versus_operator_permuted.finish()?;
        let witness_continuation = if let Some(prefix) = witness_prefix {
            Some(self.continue_matched(table, base_overlay, &prefix, MAX_CONTINUATION_UNITS)?)
        } else {
            None
        };
        let witness_continuation_contrast = if let Some(continuation) = &witness_continuation {
            continuation.real.decoded != continuation.scope_disabled.decoded
                && continuation.real.decoded != continuation.order_shuffled.decoded
                && continuation.real.decoded != continuation.operator_permuted.decoded
        } else {
            false
        };
        let witness_continuation_cid = witness_continuation
            .as_ref()
            .map(|continuation| {
                serde_json::to_vec(continuation)
                    .map(|bytes| blake3_label(&bytes))
                    .map_err(|error| {
                        CorpusInducedDocumentSpinError::Serialization(error.to_string())
                    })
            })
            .transpose()?;
        let decision = if versus_scope_disabled.passes
            && versus_order_shuffled.passes
            && versus_operator_permuted.passes
            && document_blocked_versus_scope_disabled.passes
            && document_blocked_versus_order_shuffled.passes
            && document_blocked_versus_operator_permuted.passes
            && witness_continuation_contrast
        {
            POSITIVE_TERMINAL
        } else {
            NEGATIVE_TERMINAL
        };
        Ok(CorpusInducedDocumentSpinEvaluation {
            schema: ARTIFACT_SCHEMA,
            domain: "uor-r4.corpus-induced-document-spin-evaluation/1".to_owned(),
            decision: decision.to_owned(),
            operator_cid: self.operator_cid.clone(),
            census_cid,
            held_out_set_kappa: census.held_out_set_kappa.clone(),
            held_out_documents: usize_u64(documents.len())?,
            post_join_admission_opportunities,
            post_join_known_target_admission_opportunities,
            operative_positions,
            operative_known_target_positions,
            real_correct,
            scope_disabled_correct,
            order_shuffled_correct,
            operator_permuted_correct,
            witness_continuation_contrast,
            witness_continuation,
            witness_continuation_cid,
            versus_scope_disabled,
            versus_order_shuffled,
            versus_operator_permuted,
            document_blocked_versus_scope_disabled,
            document_blocked_versus_order_shuffled,
            document_blocked_versus_operator_permuted,
            target_reads,
            forbidden_reads: evaluation_forbidden_reads,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnsignedLimbs(Vec<u32>);

impl UnsignedLimbs {
    fn one() -> Self {
        Self(vec![1])
    }

    fn power_of_two(exponent: u64) -> Result<Self, CorpusInducedDocumentSpinError> {
        let word = usize::try_from(exponent / 32)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let mut limbs = vec![
            0_u32;
            word.checked_add(1)
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?
        ];
        limbs[word] = 1_u32 << (exponent % 32);
        Ok(Self(limbs))
    }

    fn multiply_small(&mut self, factor: u64) -> Result<(), CorpusInducedDocumentSpinError> {
        let mut carry = 0_u128;
        for limb in &mut self.0 {
            let product = u128::from(*limb)
                .checked_mul(u128::from(factor))
                .and_then(|value| value.checked_add(carry))
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
            *limb = product as u32;
            carry = product >> 32;
        }
        while carry != 0 {
            self.0.push(carry as u32);
            carry >>= 32;
        }
        Ok(())
    }

    fn divide_small_exact(&mut self, divisor: u64) -> Result<(), CorpusInducedDocumentSpinError> {
        if divisor == 0 {
            return Err(CorpusInducedDocumentSpinError::ArithmeticOverflow);
        }
        let mut remainder = 0_u128;
        for limb in self.0.iter_mut().rev() {
            let dividend = (remainder << 32) | u128::from(*limb);
            let quotient = dividend / u128::from(divisor);
            remainder = dividend % u128::from(divisor);
            *limb = u32::try_from(quotient)
                .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        }
        if remainder != 0 {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "binomial recurrence division was not exact".to_owned(),
            ));
        }
        self.normalize();
        Ok(())
    }

    fn add_assign(&mut self, other: &Self) -> Result<(), CorpusInducedDocumentSpinError> {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0);
        }
        let mut carry = 0_u64;
        for index in 0..self.0.len() {
            let right = other.0.get(index).copied().unwrap_or(0);
            let sum = u64::from(self.0[index]) + u64::from(right) + carry;
            self.0[index] = sum as u32;
            carry = sum >> 32;
        }
        if carry != 0 {
            self.0.push(
                u32::try_from(carry)
                    .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
            );
        }
        Ok(())
    }

    fn normalize(&mut self) {
        while self.0.len() > 1 && self.0.last() == Some(&0) {
            self.0.pop();
        }
    }
}

impl PartialOrd for UnsignedLimbs {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnsignedLimbs {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

fn exact_one_sided_sign_test_passes(
    wins: u64,
    losses: u64,
) -> Result<bool, CorpusInducedDocumentSpinError> {
    let n = checked_add_u64(wins, losses)?;
    if n == 0 || wins <= losses {
        return Ok(false);
    }
    // By symmetry P[X >= wins] = sum_{k=0}^{losses} C(n,k) / 2^n.
    // Compare to 1/20 with an exact little-endian unsigned limb integer.
    let mut term = UnsignedLimbs::one();
    let mut cumulative = term.clone();
    for k in 1..=losses {
        term.multiply_small(n - k + 1)?;
        term.divide_small_exact(k)?;
        cumulative.add_assign(&term)?;
    }
    cumulative.multiply_small(20)?;
    Ok(cumulative <= UnsignedLimbs::power_of_two(n)?)
}

fn continue_arm(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    prefix: &[u32],
    max_units: usize,
    arm: CorpusInducedDocumentSpinArm,
) -> Result<Continuation, CorpusInducedDocumentSpinError> {
    let mut context = prefix.to_vec();
    let mut generated = Vec::new();
    let mut stop = ContinuationStop::Bound;
    while generated.len() < max_units {
        let prediction = operator.predict_matched_bound(table, base_overlay, &context)?;
        let token = match arm {
            CorpusInducedDocumentSpinArm::Real => prediction.real.token,
            CorpusInducedDocumentSpinArm::ScopeDisabled => prediction.scope_disabled.token,
            CorpusInducedDocumentSpinArm::OrderShuffled => prediction.order_shuffled.token,
            CorpusInducedDocumentSpinArm::OperatorPermuted => prediction.operator_permuted.token,
        };
        if token == EOS_TOKEN {
            stop = ContinuationStop::EndOfDocument;
            break;
        }
        if generated.last() == Some(&token) {
            stop = ContinuationStop::PeriodOneCycle;
            break;
        }
        if generated.len() >= 3
            && generated[generated.len() - 2] == token
            && generated[generated.len() - 3] == generated[generated.len() - 1]
        {
            stop = ContinuationStop::PeriodTwoCycle;
            break;
        }
        generated.push(token);
        context.push(token);
    }
    let decoded = table.decode_tokens(&generated)?;
    Ok(Continuation {
        tokens: generated,
        decoded,
        stop,
    })
}

#[derive(Serialize)]
struct OmegaCandidateWire<'a> {
    token: u32,
    relative_state: &'a BoundedGlobalExactSpinStateTrace,
    cost: BoundedGlobalExactSpinCost,
}

#[derive(Serialize)]
struct OmegaWire<'a> {
    order: u8,
    support: &'a [u32],
    natural_state: &'a BoundedGlobalExactSpinStateTrace,
    candidates: Vec<OmegaCandidateWire<'a>>,
}

fn digest_label(digest: [u8; 32]) -> String {
    format!("blake3:{}", hex::encode(digest))
}

fn active_row_key(
    prefix: &[u32],
    order: BackoffOrder,
) -> Result<ActiveRowKey, CorpusInducedDocumentSpinError> {
    match order {
        BackoffOrder::Trigram if prefix.len() >= 2 => Ok(ActiveRowKey {
            order: 2,
            previous_two: prefix[prefix.len() - 2],
            previous_one: prefix[prefix.len() - 1],
        }),
        BackoffOrder::Bigram if !prefix.is_empty() => Ok(ActiveRowKey {
            order: 1,
            previous_two: BOS_TOKEN,
            previous_one: prefix[prefix.len() - 1],
        }),
        _ => Err(CorpusInducedDocumentSpinError::Invalid(
            "active document-spin row lacks its causal context".to_owned(),
        )),
    }
}

fn row_label(row: &ActiveRowKey) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ROW_DOMAIN);
    hasher.update(&[row.order]);
    hasher.update(&row.previous_two.to_le_bytes());
    hasher.update(&row.previous_one.to_le_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn support_label(support: &[u32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SUPPORT_DOMAIN);
    hasher.update(&(support.len() as u64).to_le_bytes());
    for token in support {
        hasher.update(&token.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn prediction_label(
    prediction: &MatchedCorpusInducedDocumentSpinPrediction,
) -> Result<String, CorpusInducedDocumentSpinError> {
    let bytes = serde_json::to_vec(prediction)
        .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(PREDICTION_DOMAIN);
    hasher.update(&bytes);
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn omega_digest(
    prediction: &MatchedCorpusInducedDocumentSpinPrediction,
) -> Result<[u8; 32], CorpusInducedDocumentSpinError> {
    let candidates = prediction
        .candidate_evidence
        .iter()
        .map(|candidate| OmegaCandidateWire {
            token: candidate.token,
            relative_state: &candidate.real_relative_state,
            cost: candidate.real_cost,
        })
        .collect();
    let wire = OmegaWire {
        order: match prediction.local.order {
            BackoffOrder::Unigram => 0,
            BackoffOrder::Bigram => 1,
            BackoffOrder::Trigram => 2,
        },
        support: &prediction.real.support_tokens,
        natural_state: &prediction.natural_state,
        candidates,
    };
    let bytes = serde_json::to_vec(&wire)
        .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(ANTI_RECALL_DOMAIN.as_bytes());
    hasher.update(ANTI_RECALL_POLICY_IDENTITY.as_bytes());
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

fn anti_recall_kappa(
    operator_cid: &str,
    construction_set_kappa: &str,
    prefixes: &BTreeSet<[u8; 32]>,
    states: &BTreeSet<ExactStateWire>,
    signatures: &BTreeSet<[u8; 32]>,
) -> Result<String, CorpusInducedDocumentSpinError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ANTI_RECALL_DOMAIN.as_bytes());
    hasher.update(ANTI_RECALL_POLICY_IDENTITY.as_bytes());
    hasher.update(operator_cid.as_bytes());
    hasher.update(construction_set_kappa.as_bytes());
    hasher.update(&usize_u64(prefixes.len())?.to_le_bytes());
    for prefix in prefixes {
        hasher.update(prefix);
    }
    let states = serde_json::to_vec(states)
        .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
    hasher.update(&states);
    hasher.update(&usize_u64(signatures.len())?.to_le_bytes());
    for signature in signatures {
        hasher.update(signature);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn compile_identity_leaves(
    max_token_id: u32,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Vec<ExactSpinState>, CorpusInducedDocumentSpinError> {
    let count = usize::try_from(max_token_id)
        .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?
        .checked_add(1)
        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
    let primes = first_primes(count)?;
    let one = 1_i32 << 30;
    let half = 1_i32 << 29;
    let roots = [
        exact_s3_spin_to_h4([one, 0, 0, 0], table)?,
        exact_s3_spin_to_h4([0, one, 0, 0], table)?,
        exact_s3_spin_to_h4([half, half, half, half], table)?,
        exact_s3_spin_to_h4([half, -half, half, -half], table)?,
    ];
    let mut leaves = Vec::with_capacity(count);
    for token in 0..=max_token_id {
        if token == BOS_TOKEN {
            leaves.push(ExactSpinState::identity(table)?);
            continue;
        }
        let index = usize::try_from(token)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let prime = i64::try_from(primes[index])
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let rank = i64::from(token);
        let fiber = prime
            .checked_mul(1_000_003)
            .and_then(|value| {
                rank.checked_mul(17_071)
                    .and_then(|add| value.checked_add(add))
            })
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let torsion = prime
            .checked_mul(-97_409)
            .and_then(|value| {
                rank.checked_mul(7_919)
                    .and_then(|add| value.checked_add(add))
            })
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        leaves.push(ExactSpinState::from_parts(
            roots[index % roots.len()],
            wrap_phase_for_compile(fiber)?,
            wrap_phase_for_compile(torsion)?,
            table,
        )?);
    }
    Ok(leaves)
}

fn compile_one_construction_document(
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    document: &SourceDocument,
    leaves: &[ExactSpinState],
    h4_table: &H4BinaryIcosahedralClosure,
) -> Result<DocumentCompilation, CorpusInducedDocumentSpinError> {
    let stream = table.encode_document_stream(document)?;
    let mut natural = ExactSpinState::identity(h4_table)?;
    let mut retained_in_document = BTreeSet::<u32>::new();
    let mut eligible_positions = 0_u64;
    let mut exemplars = Vec::new();
    for target_index in 1..stream.len() {
        let local = table.predict_multiscale_count_radius(&stream[..target_index], base_overlay)?;
        let target = stream[target_index];
        if active_maximum_tie(&local)
            && table.is_fitted_lexical_token(target)
            && local.max_count_tie_tokens.binary_search(&target).is_ok()
        {
            eligible_positions = checked_add_u64(eligible_positions, 1)?;
            if retained_in_document.insert(target) {
                exemplars.push((
                    target,
                    ConstructionExemplar {
                        document_id: document.id.clone(),
                        state: natural,
                    },
                ));
            }
        }
        natural = natural.compose(leaf_for_token(leaves, target)?, h4_table)?;
    }
    Ok(DocumentCompilation {
        eligible_positions,
        exemplars,
    })
}

#[cfg(target_arch = "wasm32")]
fn compile_construction_documents(
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    leaves: &[ExactSpinState],
    h4_table: &H4BinaryIcosahedralClosure,
    _worker_count: Option<usize>,
) -> Result<Vec<DocumentCompilation>, CorpusInducedDocumentSpinError> {
    documents
        .iter()
        .map(|document| {
            compile_one_construction_document(table, base_overlay, document, leaves, h4_table)
        })
        .collect()
}

fn compile_one_candidate_prototype(
    table: &SourceFreeTable,
    token: u32,
    rows: &[ConstructionExemplar],
    h4_table: &H4BinaryIcosahedralClosure,
) -> Result<CandidateCompilation, CorpusInducedDocumentSpinError> {
    let document_support = rows
        .iter()
        .map(|row| row.document_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if document_support
        < usize::try_from(MIN_PROTOTYPE_DOCUMENTS)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?
    {
        return Ok(CandidateCompilation::RejectedSingleDocument);
    }
    let distinct_states = rows
        .iter()
        .map(|row| exact_state_wire(row.state, h4_table))
        .collect::<Result<BTreeSet<_>, _>>()?
        .len();
    if distinct_states
        < usize::try_from(MIN_DISTINCT_PROTOTYPE_STATES)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?
    {
        return Ok(CandidateCompilation::RejectedSingleState);
    }
    let states = rows.iter().map(|row| row.state).collect::<Vec<_>>();
    let (state, fit_audit) = fit_componentwise_frechet_prototype(&states, h4_table)?;
    let payload = table.decode_tokens(&[token])?;
    Ok(CandidateCompilation::Usable(Prototype {
        token,
        payload_cid: blake3_label(&payload),
        state,
        construction_document_support: u32::try_from(document_support)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
        distinct_state_support: u32::try_from(distinct_states)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
        fit_audit,
    }))
}

#[cfg(target_arch = "wasm32")]
fn compile_candidate_prototypes(
    table: &SourceFreeTable,
    rows: &[(u32, Vec<ConstructionExemplar>)],
    h4_table: &H4BinaryIcosahedralClosure,
    _worker_count: Option<usize>,
) -> Result<Vec<CandidateCompilation>, CorpusInducedDocumentSpinError> {
    rows.iter()
        .map(|(token, exemplars)| {
            compile_one_candidate_prototype(table, *token, exemplars, h4_table)
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn compile_candidate_prototypes(
    table: &SourceFreeTable,
    rows: &[(u32, Vec<ConstructionExemplar>)],
    h4_table: &H4BinaryIcosahedralClosure,
    worker_count: Option<usize>,
) -> Result<Vec<CandidateCompilation>, CorpusInducedDocumentSpinError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let workers = worker_count
        .unwrap_or(available)
        .min(MAX_NATIVE_DOCUMENT_SCAN_WORKERS)
        .min(rows.len())
        .max(1);
    let chunk_size = rows.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for (chunk_index, chunk) in rows.chunks(chunk_size).enumerate() {
            let handle = std::thread::Builder::new()
                .name(format!("uor-r4-candidate-compile-{chunk_index}"))
                .spawn_scoped(scope, move || {
                    chunk
                        .iter()
                        .map(|(token, exemplars)| {
                            compile_one_candidate_prototype(table, *token, exemplars, h4_table)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_err(|error| {
                    CorpusInducedDocumentSpinError::Invalid(format!(
                        "failed to spawn document-spin candidate worker: {error}"
                    ))
                })?;
            handles.push(handle);
        }
        let mut compiled = Vec::with_capacity(rows.len());
        for handle in handles {
            let mut chunk = handle.join().map_err(|_| {
                CorpusInducedDocumentSpinError::Invalid(
                    "document-spin candidate worker panicked".to_owned(),
                )
            })??;
            compiled.append(&mut chunk);
        }
        Ok(compiled)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn compile_construction_documents(
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    leaves: &[ExactSpinState],
    h4_table: &H4BinaryIcosahedralClosure,
    worker_count: Option<usize>,
) -> Result<Vec<DocumentCompilation>, CorpusInducedDocumentSpinError> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let workers = worker_count
        .unwrap_or(available)
        .min(MAX_NATIVE_DOCUMENT_SCAN_WORKERS)
        .min(documents.len())
        .max(1);
    let chunk_size = documents.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for (chunk_index, chunk) in documents.chunks(chunk_size).enumerate() {
            let handle = std::thread::Builder::new()
                .name(format!("uor-r4-construction-compile-{chunk_index}"))
                .spawn_scoped(scope, move || {
                    chunk
                        .iter()
                        .map(|document| {
                            compile_one_construction_document(
                                table,
                                base_overlay,
                                document,
                                leaves,
                                h4_table,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_err(|error| {
                    CorpusInducedDocumentSpinError::Invalid(format!(
                        "failed to spawn document-spin construction worker: {error}"
                    ))
                })?;
            handles.push(handle);
        }
        let mut compiled = Vec::with_capacity(documents.len());
        for handle in handles {
            let mut chunk = handle.join().map_err(|_| {
                CorpusInducedDocumentSpinError::Invalid(
                    "document-spin construction worker panicked".to_owned(),
                )
            })??;
            compiled.append(&mut chunk);
        }
        Ok(compiled)
    })
}

fn first_primes(count: usize) -> Result<Vec<u64>, CorpusInducedDocumentSpinError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut limit = 32_usize;
    loop {
        let mut composite = vec![
            false;
            limit
                .checked_add(1)
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow,)?
        ];
        let mut candidate = 2_usize;
        while candidate <= limit / candidate {
            if !composite[candidate] {
                let mut multiple = candidate
                    .checked_mul(candidate)
                    .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
                while multiple <= limit {
                    composite[multiple] = true;
                    multiple = multiple
                        .checked_add(candidate)
                        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
                }
            }
            candidate += 1;
        }
        let primes = (2..=limit)
            .filter(|&value| !composite[value])
            .map(|value| {
                u64::try_from(value).map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if primes.len() >= count {
            return Ok(primes.into_iter().take(count).collect());
        }
        limit = limit
            .checked_mul(2)
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
    }
}

fn wrap_phase_for_compile(value: i64) -> Result<i64, CorpusInducedDocumentSpinError> {
    value
        .checked_add(PHASE_HALF_Q29)
        .map(|shifted| shifted.rem_euclid(PHASE_MODULUS_Q29) - PHASE_HALF_Q29)
        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)
}

fn fit_componentwise_frechet_prototype(
    states: &[ExactSpinState],
    table: &H4BinaryIcosahedralClosure,
) -> Result<(ExactSpinState, PrototypeFitAudit), CorpusInducedDocumentSpinError> {
    if states.is_empty() {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "cannot fit an empty document-spin prototype".to_owned(),
        ));
    }
    let mut best_h4: Option<(u128, OpaqueH4TableIndex, u16)> = None;
    for offset in 0..table.root_count {
        let offset = u16::try_from(offset)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let index = OpaqueH4TableIndex::from_table_offset(offset, table).ok_or_else(|| {
            CorpusInducedDocumentSpinError::Invalid(
                "H4 Frechet search addressed outside the exact root table".to_owned(),
            )
        })?;
        let candidate = ExactSpinState::from_table_index_and_phases(index, 0, 0, table)?;
        let total = states.iter().try_fold(0_u128, |sum, &state| {
            let rank = u128::from(shell_rank(
                candidate_relative_exact_cost(candidate, state, table)?
                    .1
                    .angular_shell,
            ));
            sum.checked_add(rank)
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)
        })?;
        match best_h4 {
            None => best_h4 = Some((total, index, 1)),
            Some((best, _, _)) if total < best => best_h4 = Some((total, index, 1)),
            Some((best, selected, count)) if total == best => {
                best_h4 = Some((
                    best,
                    selected,
                    count
                        .checked_add(1)
                        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
                ));
            }
            Some(_) => {}
        }
    }
    let (h4_minimum_objective_sum, h4, h4_minimizer_count) = best_h4.ok_or_else(|| {
        CorpusInducedDocumentSpinError::Invalid("exact H4 root table is empty".to_owned())
    })?;
    let (fiber, fiber_minimum_objective_sum, fiber_minimizer_count) =
        circular_medoid(states.iter().map(|state| state.fiber_q29()))?;
    let (torsion, torsion_minimum_objective_sum, torsion_minimizer_count) =
        circular_medoid(states.iter().map(|state| state.torsion_q29()))?;
    Ok((
        ExactSpinState::from_table_index_and_phases(h4, fiber, torsion, table)?,
        PrototypeFitAudit {
            h4_minimum_objective_sum,
            h4_minimizer_count,
            fiber_minimum_objective_sum,
            fiber_minimizer_count,
            torsion_minimum_objective_sum,
            torsion_minimizer_count,
        },
    ))
}

fn circular_medoid(
    values: impl IntoIterator<Item = i64>,
) -> Result<(i64, u128, u32), CorpusInducedDocumentSpinError> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "circular medoid input is empty".to_owned(),
        ));
    }
    values.sort_unstable();
    let mut candidates = values.clone();
    candidates.dedup();
    let modulus = i128::from(PHASE_MODULUS_Q29);
    let half = i128::from(PHASE_HALF_Q29);
    let mut points = values
        .iter()
        .map(|&value| {
            let value = i128::from(value);
            if value < 0 {
                value + modulus
            } else {
                value
            }
        })
        .collect::<Vec<_>>();
    points.sort_unstable();
    let mut prefix_sums = Vec::with_capacity(points.len() + 1);
    prefix_sums.push(0_i128);
    for &point in &points {
        let next = prefix_sums
            .last()
            .copied()
            .and_then(|sum| sum.checked_add(point))
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        prefix_sums.push(next);
    }
    let total_sum = *prefix_sums.last().ok_or_else(|| {
        CorpusInducedDocumentSpinError::Invalid("circular medoid input is empty".to_owned())
    })?;
    let mut best: Option<(u128, i64, u32)> = None;
    for &candidate in &candidates {
        let candidate_unsigned = {
            let value = i128::from(candidate);
            if value < 0 {
                value + modulus
            } else {
                value
            }
        };
        let split = points.partition_point(|&point| point < candidate_unsigned);
        let split_i128 = i128::try_from(split)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let right_count = i128::try_from(points.len() - split)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let mut total = candidate_unsigned
            .checked_mul(split_i128)
            .and_then(|value| value.checked_sub(prefix_sums[split]))
            .and_then(|value| {
                total_sum
                    .checked_sub(prefix_sums[split])
                    .and_then(|right_sum| {
                        candidate_unsigned
                            .checked_mul(right_count)
                            .and_then(|right_center| right_sum.checked_sub(right_center))
                    })
                    .and_then(|right| value.checked_add(right))
            })
            .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        let left_boundary = candidate_unsigned - half;
        if left_boundary > 0 {
            let count = points.partition_point(|&point| point < left_boundary);
            let count_i128 = i128::try_from(count)
                .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
            let correction = modulus
                .checked_mul(count_i128)
                .and_then(|base| {
                    candidate_unsigned
                        .checked_mul(count_i128)
                        .and_then(|center| center.checked_sub(prefix_sums[count]))
                        .and_then(|distance| distance.checked_mul(2))
                        .and_then(|twice| base.checked_sub(twice))
                })
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
            total = total
                .checked_add(correction)
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        }
        let right_boundary = candidate_unsigned + half;
        if right_boundary < modulus {
            let start = points.partition_point(|&point| point <= right_boundary);
            let count = points.len() - start;
            let count_i128 = i128::try_from(count)
                .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
            let correction = modulus
                .checked_mul(count_i128)
                .and_then(|base| {
                    total_sum
                        .checked_sub(prefix_sums[start])
                        .and_then(|right_sum| {
                            candidate_unsigned
                                .checked_mul(count_i128)
                                .and_then(|center| right_sum.checked_sub(center))
                        })
                        .and_then(|distance| distance.checked_mul(2))
                        .and_then(|twice| base.checked_sub(twice))
                })
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
            total = total
                .checked_add(correction)
                .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        }
        let total = u128::try_from(total)
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        match best {
            None => best = Some((total, candidate, 1)),
            Some((current, _, _)) if total < current => best = Some((total, candidate, 1)),
            Some((current, selected, count)) if total == current => {
                best = Some((
                    current,
                    selected,
                    count
                        .checked_add(1)
                        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
                ));
            }
            Some(_) => {}
        }
    }
    best.map(|(objective, candidate, count)| (candidate, objective, count))
        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)
}

fn shell_rank(shell: H4S3AngularShell) -> u8 {
    match shell {
        H4S3AngularShell::Coincident => 0,
        H4S3AngularShell::Degrees36 => 1,
        H4S3AngularShell::Degrees60 => 2,
        H4S3AngularShell::Degrees72 => 3,
        H4S3AngularShell::Orthogonal => 4,
        H4S3AngularShell::Degrees108 => 5,
        H4S3AngularShell::Degrees120 => 6,
        H4S3AngularShell::Degrees144 => 7,
        H4S3AngularShell::Antipodal => 8,
    }
}

fn leaf_for_token(
    leaves: &[ExactSpinState],
    token: u32,
) -> Result<ExactSpinState, CorpusInducedDocumentSpinError> {
    let index =
        usize::try_from(token).map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
    leaves.get(index).copied().ok_or_else(|| {
        CorpusInducedDocumentSpinError::Invalid(
            "document-spin prefix token is outside the fitted namespace".to_owned(),
        )
    })
}

fn exact_state_wire(
    state: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactStateWire, CorpusInducedDocumentSpinError> {
    Ok(ExactStateWire {
        h4_table_offset: state.table_index().table_offset(),
        h4_coordinate: state.root_coordinate(table)?.scaled_zphi_quaternion,
        fiber_q29: state.fiber_q29(),
        torsion_q29: state.torsion_q29(),
    })
}

fn exact_state_from_wire(
    wire: &ExactStateWire,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, CorpusInducedDocumentSpinError> {
    if wire.fiber_q29 < -PHASE_HALF_Q29
        || wire.fiber_q29 >= PHASE_HALF_Q29
        || wire.torsion_q29 < -PHASE_HALF_Q29
        || wire.torsion_q29 >= PHASE_HALF_Q29
    {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "serialized document-spin phase is noncanonical".to_owned(),
        ));
    }
    let index =
        OpaqueH4TableIndex::from_table_offset(wire.h4_table_offset, table).ok_or_else(|| {
            CorpusInducedDocumentSpinError::Invalid("serialized H4 offset is invalid".to_owned())
        })?;
    let state = ExactSpinState::from_table_index_and_phases(
        index,
        wire.fiber_q29,
        wire.torsion_q29,
        table,
    )?;
    if state.root_coordinate(table)?.scaled_zphi_quaternion != wire.h4_coordinate {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "serialized H4 offset and coordinate disagree".to_owned(),
        ));
    }
    Ok(state)
}

fn active_maximum_tie(local: &MatchedGeometricPrediction) -> bool {
    local.geometry_reachable
        && local.max_count_tie_tokens.len() > 1
        && matches!(local.order, BackoffOrder::Trigram | BackoffOrder::Bigram)
}

fn work_from_executed_query(
    local: &MatchedGeometricPrediction,
    query: ExecutedQueryState,
) -> CorpusInducedDocumentSpinWork {
    CorpusInducedDocumentSpinWork {
        local: local.geometric_work,
        prefix_leaf_reads: query.prefix_leaf_reads,
        prefix_h4_product_reads: query.prefix_h4_product_reads,
        prefix_phase_additions: query.prefix_phase_additions,
        prototype_reads: 0,
        prototype_inverse_reads: 0,
        relative_h4_product_reads: 0,
        relative_phase_additions: 0,
        angular_shell_reads: 0,
        phase_distance_reads: 0,
        cost_comparisons: 0,
        final_choice_operations: 0,
    }
}

fn execute_arm(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    arm: CorpusInducedDocumentSpinArm,
    local: &MatchedGeometricPrediction,
    query: ExecutedQueryState,
    rotate_prototypes: bool,
) -> Result<ExecutedArm, CorpusInducedDocumentSpinError> {
    let mut support = Vec::with_capacity(local.max_count_tie_tokens.len());
    for &token in &local.max_count_tie_tokens {
        support.push(token);
    }
    let mut work = work_from_executed_query(local, query);
    if !active_maximum_tie(local) {
        work.final_choice_operations = checked_add_u64(work.final_choice_operations, 1)?;
        return Ok(ExecutedArm {
            decision: fallback_decision(
                arm,
                local.geometric_token,
                &support,
                work,
                CorpusInducedDocumentSpinAbstention::NotMaximumCountTie,
            ),
            candidates: Vec::new(),
        });
    }
    let mut assigned = Vec::with_capacity(support.len());
    let mut complete = true;
    for index in 0..support.len() {
        let prototype_token = if rotate_prototypes {
            support[(index + 1) % support.len()]
        } else {
            support[index]
        };
        work.prototype_reads = checked_add_u64(work.prototype_reads, 1)?;
        if !operator.prototypes.contains_key(&prototype_token) {
            complete = false;
        }
        assigned.push(prototype_token);
    }
    if !complete {
        work.final_choice_operations = checked_add_u64(work.final_choice_operations, 1)?;
        return Ok(ExecutedArm {
            decision: fallback_decision(
                arm,
                local.geometric_token,
                &support,
                work,
                CorpusInducedDocumentSpinAbstention::MissingPrototype,
            ),
            candidates: Vec::new(),
        });
    }
    let mut candidates = Vec::with_capacity(support.len());
    let mut costs = Vec::with_capacity(support.len());
    for (&support_token, &prototype_token) in support.iter().zip(&assigned) {
        let prototype = operator.prototypes.get(&prototype_token).ok_or_else(|| {
            CorpusInducedDocumentSpinError::Invalid(
                "independently admitted prototype disappeared during scoring".to_owned(),
            )
        })?;
        let (relative_state, cost) =
            candidate_relative_exact_cost(prototype.state, query.state, &operator.h4_table)?;
        work.prototype_inverse_reads = checked_add_u64(work.prototype_inverse_reads, 1)?;
        work.relative_h4_product_reads = checked_add_u64(work.relative_h4_product_reads, 1)?;
        work.relative_phase_additions = checked_add_u64(work.relative_phase_additions, 2)?;
        work.angular_shell_reads = checked_add_u64(work.angular_shell_reads, 1)?;
        work.phase_distance_reads = checked_add_u64(work.phase_distance_reads, 2)?;
        costs.push(cost);
        candidates.push(ExecutedCandidateScore {
            support_token,
            prototype_token,
            prototype_state: prototype.state,
            relative_state,
            cost,
        });
    }
    let decision = select_decision(arm, local.geometric_token, &support, &costs, work)?;
    Ok(ExecutedArm {
        decision,
        candidates,
    })
}

fn issue_score_firewall_certificate(
    operator_cid: &str,
    frame: CausalScoreFrame,
    table: &H4BinaryIcosahedralClosure,
    candidate_count: usize,
) -> Result<CorpusInducedDocumentSpinScoreFirewallCertificate, CorpusInducedDocumentSpinError> {
    let expected_phase_additions = frame
        .prefix_units
        .checked_mul(2)
        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
    let executed_queries = [
        frame.real,
        frame.scope_disabled,
        frame.order_shuffled,
        frame.operator_permuted,
    ];
    if executed_queries.iter().any(|query| {
        query.prefix_leaf_reads != frame.prefix_units
            || query.prefix_h4_product_reads != frame.prefix_units
            || query.prefix_phase_additions != expected_phase_additions
    }) || frame.real.state != frame.scope_disabled.state
        || frame.real.state != frame.operator_permuted.state
    {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "score firewall rejected an imbalanced executed-prefix capability".to_owned(),
        ));
    }
    let forbidden_dependency_mask = match frame.origin {
        ScoreFrameOrigin::CausalPrefix => 0,
        #[cfg(test)]
        ScoreFrameOrigin::InjectedHeldOutTarget => 1,
    };
    let state_bytes = serde_json::to_vec(
        &executed_queries
            .map(|query| {
                Ok::<_, CorpusInducedDocumentSpinError>((
                    exact_state_wire(query.state, table)?,
                    query.prefix_leaf_reads,
                    query.prefix_h4_product_reads,
                    query.prefix_phase_additions,
                ))
            })
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))?;
    let mut certificate = CorpusInducedDocumentSpinScoreFirewallCertificate {
        schema: ARTIFACT_SCHEMA,
        domain: SCORE_FIREWALL_DOMAIN.to_owned(),
        policy_kappa: identity_kappa(&[SCORE_FIREWALL_POLICY]),
        operator_cid: operator_cid.to_owned(),
        causal_prefix_units: frame.prefix_units,
        candidate_count: usize_u64(candidate_count)?,
        query_state_kappa: blake3_label(&state_bytes),
        forbidden_dependency_mask,
        certificate_cid: String::new(),
    };
    certificate.certificate_cid = score_firewall_certificate_cid(&certificate);
    if !certificate.validate() {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "score firewall rejected a forbidden or noncanonical input capability".to_owned(),
        ));
    }
    Ok(certificate)
}

#[cfg(test)]
fn injected_target_score_firewall_control() -> Result<(), CorpusInducedDocumentSpinError> {
    let table = validate_h4_binary_icosahedral_closure()
        .map_err(|error| CorpusInducedDocumentSpinError::ExactSpin(error.to_string()))?;
    let identity = ExactSpinState::identity(&table)?;
    let query = ExecutedQueryState {
        state: identity,
        prefix_leaf_reads: 1,
        prefix_h4_product_reads: 1,
        prefix_phase_additions: 2,
    };
    let frame = CausalScoreFrame {
        origin: ScoreFrameOrigin::InjectedHeldOutTarget,
        prefix_units: 1,
        real: query,
        scope_disabled: query,
        order_shuffled: query,
        operator_permuted: query,
    };
    issue_score_firewall_certificate("blake3:injected-target", frame, &table, 2).map(|_| ())
}

#[cfg(test)]
fn imbalanced_work_score_firewall_control() -> Result<(), CorpusInducedDocumentSpinError> {
    let table = validate_h4_binary_icosahedral_closure()
        .map_err(|error| CorpusInducedDocumentSpinError::ExactSpin(error.to_string()))?;
    let identity = ExactSpinState::identity(&table)?;
    let balanced = ExecutedQueryState {
        state: identity,
        prefix_leaf_reads: 1,
        prefix_h4_product_reads: 1,
        prefix_phase_additions: 2,
    };
    let mut imbalanced = balanced;
    imbalanced.prefix_h4_product_reads = 0;
    let frame = CausalScoreFrame {
        origin: ScoreFrameOrigin::CausalPrefix,
        prefix_units: 1,
        real: balanced,
        scope_disabled: balanced,
        order_shuffled: imbalanced,
        operator_permuted: balanced,
    };
    issue_score_firewall_certificate("blake3:imbalanced-work", frame, &table, 2).map(|_| ())
}

fn score_firewall_certificate_cid(
    certificate: &CorpusInducedDocumentSpinScoreFirewallCertificate,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SCORE_FIREWALL_DOMAIN.as_bytes());
    hasher.update(certificate.policy_kappa.as_bytes());
    hasher.update(certificate.operator_cid.as_bytes());
    hasher.update(&certificate.causal_prefix_units.to_le_bytes());
    hasher.update(&certificate.candidate_count.to_le_bytes());
    hasher.update(certificate.query_state_kappa.as_bytes());
    hasher.update(&certificate.forbidden_dependency_mask.to_le_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn forbidden_reads_from_mask(mask: u16) -> CorpusInducedDocumentSpinForbiddenReads {
    let set = |bit: u32| u64::from((mask & (1_u16 << bit)) != 0);
    CorpusInducedDocumentSpinForbiddenReads {
        held_out_target_reads: set(0),
        compiler_future_reads: set(1),
        teacher_calls: set(2),
        provider_calls: set(3),
        source_weight_reads: set(4),
        runtime_corpus_reads: set(5),
        token_score_reads: set(6),
        payload_score_reads: set(7),
        prime_score_reads: set(8),
        rank_score_reads: set(9),
        digest_score_reads: set(10),
        support_score_reads: set(11),
        provenance_score_reads: set(12),
    }
}

fn fallback_decision(
    arm: CorpusInducedDocumentSpinArm,
    fallback: u32,
    support: &[u32],
    work: CorpusInducedDocumentSpinWork,
    abstention: CorpusInducedDocumentSpinAbstention,
) -> CorpusInducedDocumentSpinDecision {
    CorpusInducedDocumentSpinDecision {
        arm,
        token: fallback,
        unique_minimum: None,
        minimum_cost: None,
        abstention: Some(abstention),
        support_tokens: support.to_vec(),
        work,
    }
}

fn select_decision(
    arm: CorpusInducedDocumentSpinArm,
    fallback: u32,
    support: &[u32],
    costs: &[BoundedGlobalExactSpinCost],
    mut work: CorpusInducedDocumentSpinWork,
) -> Result<CorpusInducedDocumentSpinDecision, CorpusInducedDocumentSpinError> {
    let selection = select_unique_minimum_exact_costs(costs)?;
    work.cost_comparisons = selection.comparisons;
    work.final_choice_operations = checked_add_u64(work.final_choice_operations, 1)?;
    let unique_minimum = selection.unique_minimum_index.map(|index| support[index]);
    Ok(CorpusInducedDocumentSpinDecision {
        arm,
        token: unique_minimum.unwrap_or(fallback),
        unique_minimum,
        minimum_cost: selection.minimum_cost,
        abstention: unique_minimum
            .is_none()
            .then_some(CorpusInducedDocumentSpinAbstention::CostTie),
        support_tokens: support.to_vec(),
        work,
    })
}

fn blake3_label(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn identity_kappa(parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn document_set_kappa(
    domain: &[u8],
    documents: &[SourceDocument],
) -> Result<String, CorpusInducedDocumentSpinError> {
    let mut documents = documents.iter().collect::<Vec<_>>();
    documents.sort_by(|left, right| left.id.cmp(&right.id));
    if documents.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(CorpusInducedDocumentSpinError::Invalid(
            "document set has duplicate IDs".to_owned(),
        ));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for document in documents {
        let id_len = u64::try_from(document.id.len())
            .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?;
        hasher.update(&id_len.to_le_bytes());
        hasher.update(document.id.as_bytes());
        hasher.update(&document.text_cid());
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn checked_add_u64(left: u64, right: u64) -> Result<u64, CorpusInducedDocumentSpinError> {
    left.checked_add(right)
        .ok_or(CorpusInducedDocumentSpinError::ArithmeticOverflow)
}

fn usize_u64(value: usize) -> Result<u64, CorpusInducedDocumentSpinError> {
    u64::try_from(value).map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)
}

impl AntiRecallDocumentScan {
    fn merge(&mut self, other: Self) {
        self.full_prefix_cids.extend(other.full_prefix_cids);
        self.natural_states.extend(other.natural_states);
        self.operative_signature_cids
            .extend(other.operative_signature_cids);
    }
}

impl TargetFreeScanAggregate {
    fn merge_document(
        &mut self,
        document: TargetFreeDocumentScan,
    ) -> Result<(), CorpusInducedDocumentSpinError> {
        self.admission_opportunities = checked_add_u64(
            self.admission_opportunities,
            document.admission_opportunities,
        )?;
        for row in document.rows_seen {
            self.row_documents
                .entry(row)
                .or_default()
                .insert(document.document_id.clone());
        }
        self.candidates.extend(document.candidates);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn merge(&mut self, other: Self) -> Result<(), CorpusInducedDocumentSpinError> {
        self.admission_opportunities =
            checked_add_u64(self.admission_opportunities, other.admission_opportunities)?;
        for (row, documents) in other.row_documents {
            self.row_documents.entry(row).or_default().extend(documents);
        }
        self.candidates.extend(other.candidates);
        Ok(())
    }
}

fn scan_one_anti_recall_document(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    document: &SourceDocument,
) -> Result<AntiRecallDocumentScan, CorpusInducedDocumentSpinError> {
    let stream = table.encode_document_stream(document)?;
    let mut cursor = CausalPrefixCursor::new(&operator.h4_table)?;
    cursor.advance(BOS_TOKEN, &operator.leaves, &operator.h4_table)?;
    let mut prefix_hash = PrefixDigestState::new();
    prefix_hash.append(BOS_TOKEN);
    let mut scan = AntiRecallDocumentScan::default();
    for target_index in 1..stream.len() {
        let prefix = &stream[..target_index];
        let local = table.predict_multiscale_count_radius(prefix, base_overlay)?;
        scan.full_prefix_cids.insert(prefix_hash.digest());
        scan.natural_states.insert(exact_state_wire(
            cursor.frame().real.state,
            &operator.h4_table,
        )?);
        if active_maximum_tie(&local) {
            let prediction = operator.predict_from_frame(local, cursor.frame())?;
            if prediction.prototype_complete {
                scan.operative_signature_cids
                    .insert(omega_digest(&prediction)?);
            }
        }
        cursor.advance(stream[target_index], &operator.leaves, &operator.h4_table)?;
        prefix_hash.append(stream[target_index]);
    }
    Ok(scan)
}

fn scan_one_target_free_document(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    document: &SourceDocument,
) -> Result<TargetFreeDocumentScan, CorpusInducedDocumentSpinError> {
    let stream = table.encode_document_stream(document)?;
    let mut cursor = CausalPrefixCursor::new(&operator.h4_table)?;
    cursor.advance(BOS_TOKEN, &operator.leaves, &operator.h4_table)?;
    let mut prefix_hash = PrefixDigestState::new();
    prefix_hash.append(BOS_TOKEN);
    let mut admission_opportunities = 0_u64;
    let mut rows_seen = BTreeSet::new();
    let mut candidates = Vec::new();
    for target_index in 1..stream.len() {
        let prefix = &stream[..target_index];
        let local = table.predict_multiscale_count_radius(prefix, base_overlay)?;
        if active_maximum_tie(&local) {
            admission_opportunities = checked_add_u64(admission_opportunities, 1)?;
            let row = active_row_key(prefix, local.order)?;
            rows_seen.insert(row.clone());
            let prediction = operator.predict_from_frame(local, cursor.frame())?;
            let prefix_cid = prefix_hash.digest();
            let omega_cid = omega_digest(&prediction)?;
            let position = CorpusInducedDocumentSpinCensusPosition {
                document_id: document.id.clone(),
                target_index: u32::try_from(target_index)
                    .map_err(|_| CorpusInducedDocumentSpinError::ArithmeticOverflow)?,
                prefix_cid: digest_label(prefix_cid),
                row_cid: row_label(&row),
                support_cid: support_label(&prediction.real.support_tokens),
                prediction_cid: prediction_label(&prediction)?,
                real_token: prediction.real.token,
                scope_disabled_token: prediction.scope_disabled.token,
                order_shuffled_token: prediction.order_shuffled.token,
                operator_permuted_token: prediction.operator_permuted.token,
            };
            candidates.push(TargetFreeCandidate {
                position,
                row,
                prediction,
                natural_state: exact_state_wire(cursor.frame().real.state, &operator.h4_table)?,
                omega_cid,
                prefix_cid,
            });
        }
        cursor.advance(stream[target_index], &operator.leaves, &operator.h4_table)?;
        prefix_hash.append(stream[target_index]);
    }
    Ok(TargetFreeDocumentScan {
        document_id: document.id.clone(),
        admission_opportunities,
        rows_seen,
        candidates,
    })
}

#[cfg(target_arch = "wasm32")]
fn scan_anti_recall_documents(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    _worker_count: Option<usize>,
) -> Result<AntiRecallDocumentScan, CorpusInducedDocumentSpinError> {
    let mut aggregate = AntiRecallDocumentScan::default();
    for document in documents {
        aggregate.merge(scan_one_anti_recall_document(
            operator,
            table,
            base_overlay,
            document,
        )?);
    }
    Ok(aggregate)
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_anti_recall_documents(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    worker_count: Option<usize>,
) -> Result<AntiRecallDocumentScan, CorpusInducedDocumentSpinError> {
    if documents.is_empty() {
        return Ok(AntiRecallDocumentScan::default());
    }
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let workers = worker_count
        .unwrap_or(available)
        .min(MAX_NATIVE_DOCUMENT_SCAN_WORKERS)
        .min(documents.len())
        .max(1);
    let chunk_size = documents.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for (chunk_index, chunk) in documents.chunks(chunk_size).enumerate() {
            let handle = std::thread::Builder::new()
                .name(format!("uor-r4-anti-recall-{chunk_index}"))
                .spawn_scoped(scope, move || {
                    let mut aggregate = AntiRecallDocumentScan::default();
                    for document in chunk {
                        aggregate.merge(scan_one_anti_recall_document(
                            operator,
                            table,
                            base_overlay,
                            document,
                        )?);
                    }
                    Ok::<_, CorpusInducedDocumentSpinError>(aggregate)
                })
                .map_err(|error| {
                    CorpusInducedDocumentSpinError::Invalid(format!(
                        "failed to spawn document-spin anti-recall worker: {error}"
                    ))
                })?;
            handles.push(handle);
        }
        let mut aggregate = AntiRecallDocumentScan::default();
        for handle in handles {
            let chunk = handle.join().map_err(|_| {
                CorpusInducedDocumentSpinError::Invalid(
                    "document-spin anti-recall worker panicked".to_owned(),
                )
            })??;
            aggregate.merge(chunk);
        }
        Ok(aggregate)
    })
}

#[cfg(target_arch = "wasm32")]
fn scan_target_free_documents(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    _worker_count: Option<usize>,
) -> Result<TargetFreeScanAggregate, CorpusInducedDocumentSpinError> {
    let mut aggregate = TargetFreeScanAggregate::default();
    for document in documents {
        aggregate.merge_document(scan_one_target_free_document(
            operator,
            table,
            base_overlay,
            document,
        )?)?;
    }
    Ok(aggregate)
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_target_free_documents(
    operator: &CorpusInducedDocumentSpinPlacementR4V1,
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    documents: &[&SourceDocument],
    worker_count: Option<usize>,
) -> Result<TargetFreeScanAggregate, CorpusInducedDocumentSpinError> {
    if documents.is_empty() {
        return Ok(TargetFreeScanAggregate::default());
    }
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let workers = worker_count
        .unwrap_or(available)
        .min(MAX_NATIVE_DOCUMENT_SCAN_WORKERS)
        .min(documents.len())
        .max(1);
    let chunk_size = documents.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for (chunk_index, chunk) in documents.chunks(chunk_size).enumerate() {
            let handle = std::thread::Builder::new()
                .name(format!("uor-r4-target-free-{chunk_index}"))
                .spawn_scoped(scope, move || {
                    let mut aggregate = TargetFreeScanAggregate::default();
                    for document in chunk {
                        aggregate.merge_document(scan_one_target_free_document(
                            operator,
                            table,
                            base_overlay,
                            document,
                        )?)?;
                    }
                    Ok::<_, CorpusInducedDocumentSpinError>(aggregate)
                })
                .map_err(|error| {
                    CorpusInducedDocumentSpinError::Invalid(format!(
                        "failed to spawn document-spin target-free worker: {error}"
                    ))
                })?;
            handles.push(handle);
        }
        let mut aggregate = TargetFreeScanAggregate::default();
        for handle in handles {
            let chunk = handle.join().map_err(|_| {
                CorpusInducedDocumentSpinError::Invalid(
                    "document-spin target-free worker panicked".to_owned(),
                )
            })??;
            aggregate.merge(chunk)?;
        }
        Ok(aggregate)
    })
}

impl CorpusInducedDocumentSpinAntiRecallIndex {
    /// Build the audit-only construction replay index. It is intentionally a
    /// separate value and has no serialization path into the runtime artifact.
    pub fn compile(
        operator: &CorpusInducedDocumentSpinPlacementR4V1,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        Self::compile_with_worker_count(operator, table, base_overlay, construction, None)
    }

    fn compile_with_worker_count(
        operator: &CorpusInducedDocumentSpinPlacementR4V1,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
        worker_count: Option<usize>,
    ) -> Result<Self, CorpusInducedDocumentSpinError> {
        operator.validate_binding(table, base_overlay)?;
        if !table.is_bound_to_construction_documents(construction)
            || document_set_kappa(CONSTRUCTION_SET_DOMAIN, construction)?
                != operator.construction_set_kappa
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "anti-recall replay is not the operator's exact construction set".to_owned(),
            ));
        }
        let mut documents = construction.iter().collect::<Vec<_>>();
        documents.sort_by(|left, right| left.id.cmp(&right.id));
        let scan =
            scan_anti_recall_documents(operator, table, base_overlay, &documents, worker_count)?;
        let index_kappa = anti_recall_kappa(
            &operator.operator_cid,
            &operator.construction_set_kappa,
            &scan.full_prefix_cids,
            &scan.natural_states,
            &scan.operative_signature_cids,
        )?;
        Ok(Self {
            operator_cid: operator.operator_cid.clone(),
            construction_set_kappa: operator.construction_set_kappa.clone(),
            full_prefix_cids: scan.full_prefix_cids,
            natural_states: scan.natural_states,
            operative_signature_cids: scan.operative_signature_cids,
            index_kappa,
        })
    }

    #[cfg(test)]
    fn canonical_bytes(&self) -> Result<Vec<u8>, CorpusInducedDocumentSpinError> {
        serde_json::to_vec(&AntiRecallIndexWire {
            schema: ARTIFACT_SCHEMA,
            domain: ANTI_RECALL_DOMAIN,
            operator_cid: &self.operator_cid,
            construction_set_kappa: &self.construction_set_kappa,
            full_prefix_cids: &self.full_prefix_cids,
            natural_states: &self.natural_states,
            operative_signature_cids: &self.operative_signature_cids,
            index_kappa: &self.index_kappa,
        })
        .map_err(|error| CorpusInducedDocumentSpinError::Serialization(error.to_string()))
    }

    pub fn index_kappa(&self) -> &str {
        &self.index_kappa
    }

    pub fn full_prefix_count(&self) -> usize {
        self.full_prefix_cids.len()
    }

    pub fn natural_state_count(&self) -> usize {
        self.natural_states.len()
    }

    pub fn operative_signature_count(&self) -> usize {
        self.operative_signature_cids.len()
    }
}

impl CorpusInducedDocumentSpinPlacementR4V1 {
    /// Census the held-out structural population without attaching any next
    /// route. The stream element at `target_index` is used only after that
    /// prefix has been frozen, when it becomes causal history for the next
    /// prefix.
    pub fn target_free_census(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        anti_recall: &CorpusInducedDocumentSpinAntiRecallIndex,
        held_out: &[SourceDocument],
    ) -> Result<CorpusInducedDocumentSpinTargetFreeCensus, CorpusInducedDocumentSpinError> {
        self.target_free_census_with_worker_count(table, base_overlay, anti_recall, held_out, None)
    }

    fn target_free_census_with_worker_count(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        anti_recall: &CorpusInducedDocumentSpinAntiRecallIndex,
        held_out: &[SourceDocument],
        worker_count: Option<usize>,
    ) -> Result<CorpusInducedDocumentSpinTargetFreeCensus, CorpusInducedDocumentSpinError> {
        self.validate_binding(table, base_overlay)?;
        if anti_recall.operator_cid != self.operator_cid
            || anti_recall.construction_set_kappa != self.construction_set_kappa
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "anti-recall index is not bound to this operator".to_owned(),
            ));
        }
        let reproduced_index_kappa = anti_recall_kappa(
            &anti_recall.operator_cid,
            &anti_recall.construction_set_kappa,
            &anti_recall.full_prefix_cids,
            &anti_recall.natural_states,
            &anti_recall.operative_signature_cids,
        )?;
        if reproduced_index_kappa != anti_recall.index_kappa {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "anti-recall index does not reproduce its canonical kappa".to_owned(),
            ));
        }
        if !table.is_disjoint_d3_held_out_documents(held_out) {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "target-free census requires a pristine disjoint D3 held-out set".to_owned(),
            ));
        }
        let held_out_set_kappa = document_set_kappa(HELD_OUT_SET_DOMAIN, held_out)?;
        if self.corpus_cid == FROZEN_CORPUS_CID && held_out_set_kappa != FROZEN_HELD_OUT_SET_KAPPA {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "frozen held-out-set kappa does not reproduce".to_owned(),
            ));
        }
        let mut documents = held_out.iter().collect::<Vec<_>>();
        documents.sort_by(|left, right| left.id.cmp(&right.id));
        let TargetFreeScanAggregate {
            admission_opportunities,
            row_documents,
            mut candidates,
        } = scan_target_free_documents(self, table, base_overlay, &documents, worker_count)?;
        candidates.sort_by(|left, right| {
            left.row
                .cmp(&right.row)
                .then_with(|| left.position.support_cid.cmp(&right.position.support_cid))
                .then_with(|| left.position.prefix_cid.cmp(&right.position.prefix_cid))
                .then_with(|| left.position.document_id.cmp(&right.position.document_id))
                .then_with(|| left.position.target_index.cmp(&right.position.target_index))
        });
        let mut rows_in_multiple_held_out_documents = 0_u64;
        let mut prototype_complete_positions = 0_u64;
        let mut full_prefix_construction_hits = 0_u64;
        let mut natural_state_construction_hits = 0_u64;
        let mut operative_signature_construction_hits = 0_u64;
        let mut natural_reverse_equal_positions = 0_u64;
        let mut permutation_inert_positions = 0_u64;
        let mut support_mismatches = 0_u64;
        let mut work_mismatches = 0_u64;
        let mut invalid_score_firewall_certificates = 0_u64;
        let mut forbidden_reads = CorpusInducedDocumentSpinForbiddenReads::default();
        let mut operative_positions = Vec::new();
        let mut frozen_decoded_witness = None;
        for candidate in candidates {
            let multi_document = row_documents
                .get(&candidate.row)
                .is_some_and(|documents| documents.len() >= 2);
            if multi_document {
                rows_in_multiple_held_out_documents =
                    checked_add_u64(rows_in_multiple_held_out_documents, 1)?;
            }
            if candidate.prediction.prototype_complete {
                prototype_complete_positions = checked_add_u64(prototype_complete_positions, 1)?;
            }
            let prefix_hit = anti_recall.full_prefix_cids.contains(&candidate.prefix_cid);
            let state_hit = anti_recall
                .natural_states
                .contains(&candidate.natural_state);
            let omega_hit = anti_recall
                .operative_signature_cids
                .contains(&candidate.omega_cid);
            if prefix_hit {
                full_prefix_construction_hits = checked_add_u64(full_prefix_construction_hits, 1)?;
            }
            if state_hit {
                natural_state_construction_hits =
                    checked_add_u64(natural_state_construction_hits, 1)?;
            }
            if omega_hit {
                operative_signature_construction_hits =
                    checked_add_u64(operative_signature_construction_hits, 1)?;
            }
            if !candidate.prediction.natural_reverse_distinct {
                natural_reverse_equal_positions =
                    checked_add_u64(natural_reverse_equal_positions, 1)?;
            }
            if !candidate.prediction.permutation_cost_vector_changed {
                permutation_inert_positions = checked_add_u64(permutation_inert_positions, 1)?;
            }
            if !candidate.prediction.support_matched {
                support_mismatches = checked_add_u64(support_mismatches, 1)?;
            }
            if !candidate.prediction.work_matched {
                work_mismatches = checked_add_u64(work_mismatches, 1)?;
            }
            let firewall_valid = candidate.prediction.score_firewall_certificate.validate();
            if !firewall_valid {
                invalid_score_firewall_certificates =
                    checked_add_u64(invalid_score_firewall_certificates, 1)?;
            }
            forbidden_reads.saturating_accumulate(candidate.prediction.forbidden_reads);
            let operative = multi_document
                && candidate.prediction.prototype_complete
                && !prefix_hit
                && !state_hit
                && !omega_hit
                && candidate.prediction.natural_reverse_distinct
                && candidate.prediction.permutation_cost_vector_changed
                && candidate.prediction.support_matched
                && candidate.prediction.work_matched
                && firewall_valid
                && candidate.prediction.forbidden_reads.total() == 0;
            if operative {
                let all_control_contrast = candidate.position.real_token != EOS_TOKEN
                    && candidate.position.real_token != candidate.position.scope_disabled_token
                    && candidate.position.real_token != candidate.position.order_shuffled_token
                    && candidate.position.real_token != candidate.position.operator_permuted_token;
                if frozen_decoded_witness.is_none() && all_control_contrast {
                    frozen_decoded_witness = Some(candidate.position.clone());
                }
                operative_positions.push(candidate.position);
            }
        }
        let frozen_identity = self.corpus_cid == FROZEN_CORPUS_CID
            && self.table_artifact_cid == FROZEN_TABLE_CID
            && self.base_overlay_artifact_cid == FROZEN_OVERLAY_CID;
        let frozen_population_reproduced = !frozen_identity
            || (usize_u64(documents.len())? == FROZEN_HELD_OUT_DOCUMENTS
                && admission_opportunities == FROZEN_TARGET_FREE_ADMISSIONS);
        let meets_frozen_preflight = frozen_population_reproduced
            && usize_u64(operative_positions.len())? >= MIN_OPERATIVE_ANTI_RECALL_POSITIONS
            && frozen_decoded_witness.is_some()
            && support_mismatches == 0
            && work_mismatches == 0
            && invalid_score_firewall_certificates == 0
            && forbidden_reads.total() == 0;
        Ok(CorpusInducedDocumentSpinTargetFreeCensus {
            schema: ARTIFACT_SCHEMA,
            domain: ANTI_RECALL_DOMAIN.to_owned(),
            operator_cid: self.operator_cid.clone(),
            anti_recall_index_kappa: anti_recall.index_kappa.clone(),
            held_out_set_kappa,
            held_out_documents: usize_u64(documents.len())?,
            admission_opportunities,
            rows_in_multiple_held_out_documents,
            prototype_complete_positions,
            full_prefix_construction_hits,
            natural_state_construction_hits,
            operative_signature_construction_hits,
            natural_reverse_equal_positions,
            permutation_inert_positions,
            support_mismatches,
            work_mismatches,
            invalid_score_firewall_certificates,
            score_firewall_policy_kappa: identity_kappa(&[SCORE_FIREWALL_POLICY]),
            meets_frozen_preflight,
            operative_positions,
            frozen_decoded_witness,
            forbidden_reads,
        })
    }
}

impl CorpusInducedDocumentSpinPlacementR4V1 {
    /// Predict all four frozen, equal-support arms from one causal prefix.
    pub fn predict_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        prefix: &[u32],
    ) -> Result<MatchedCorpusInducedDocumentSpinPrediction, CorpusInducedDocumentSpinError> {
        self.validate_binding(table, base_overlay)?;
        self.predict_matched_bound(table, base_overlay, prefix)
    }

    fn predict_matched_bound(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        prefix: &[u32],
    ) -> Result<MatchedCorpusInducedDocumentSpinPrediction, CorpusInducedDocumentSpinError> {
        if prefix.first().copied() != Some(BOS_TOKEN)
            || prefix.iter().any(|&token| token > self.max_token_id)
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "document-spin prediction requires a BOS-led in-range prefix".to_owned(),
            ));
        }
        let frame = CausalPrefixCursor::from_prefix(prefix, &self.leaves, &self.h4_table)?.frame();
        let local = table.predict_multiscale_count_radius(prefix, base_overlay)?;
        self.predict_from_frame(local, frame)
    }

    /// Continue each arm on its own causal history after the shared first
    /// decision. EOS and period-one/two sentinels are not appended.
    pub fn continue_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        prefix: &[u32],
        max_units: usize,
    ) -> Result<MatchedCorpusInducedDocumentSpinContinuation, CorpusInducedDocumentSpinError> {
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(CorpusInducedDocumentSpinError::Invalid(format!(
                "document-spin continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        self.validate_binding(table, base_overlay)?;
        let first_decision = self.predict_matched_bound(table, base_overlay, prefix)?;
        Ok(MatchedCorpusInducedDocumentSpinContinuation {
            first_decision,
            real: continue_arm(
                self,
                table,
                base_overlay,
                prefix,
                max_units,
                CorpusInducedDocumentSpinArm::Real,
            )?,
            scope_disabled: continue_arm(
                self,
                table,
                base_overlay,
                prefix,
                max_units,
                CorpusInducedDocumentSpinArm::ScopeDisabled,
            )?,
            order_shuffled: continue_arm(
                self,
                table,
                base_overlay,
                prefix,
                max_units,
                CorpusInducedDocumentSpinArm::OrderShuffled,
            )?,
            operator_permuted: continue_arm(
                self,
                table,
                base_overlay,
                prefix,
                max_units,
                CorpusInducedDocumentSpinArm::OperatorPermuted,
            )?,
        })
    }

    fn predict_from_frame(
        &self,
        local: MatchedGeometricPrediction,
        frame: CausalScoreFrame,
    ) -> Result<MatchedCorpusInducedDocumentSpinPrediction, CorpusInducedDocumentSpinError> {
        if local
            .max_count_tie_tokens
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(CorpusInducedDocumentSpinError::Invalid(
                "#953 maximum-count support is not canonical".to_owned(),
            ));
        }
        let score_firewall_certificate = issue_score_firewall_certificate(
            &self.operator_cid,
            frame,
            &self.h4_table,
            local.max_count_tie_tokens.len(),
        )?;
        let forbidden_reads =
            forbidden_reads_from_mask(score_firewall_certificate.forbidden_dependency_mask);
        let real_execution = execute_arm(
            self,
            CorpusInducedDocumentSpinArm::Real,
            &local,
            frame.real,
            false,
        )?;
        let mut scope_execution = execute_arm(
            self,
            CorpusInducedDocumentSpinArm::ScopeDisabled,
            &local,
            frame.scope_disabled,
            false,
        )?;
        scope_execution.decision.token = local.geometric_token;
        let order_execution = execute_arm(
            self,
            CorpusInducedDocumentSpinArm::OrderShuffled,
            &local,
            frame.order_shuffled,
            false,
        )?;
        let permuted_execution = execute_arm(
            self,
            CorpusInducedDocumentSpinArm::OperatorPermuted,
            &local,
            frame.operator_permuted,
            true,
        )?;
        let decisions = [
            &real_execution.decision,
            &scope_execution.decision,
            &order_execution.decision,
            &permuted_execution.decision,
        ];
        let support_matched = decisions
            .windows(2)
            .all(|pair| pair[0].support_tokens == pair[1].support_tokens);
        let work_matched = decisions
            .windows(2)
            .all(|pair| pair[0].work == pair[1].work);
        let prototype_complete = active_maximum_tie(&local)
            && decisions.iter().all(|decision| {
                decision.abstention != Some(CorpusInducedDocumentSpinAbstention::MissingPrototype)
            });
        let mut evidence = Vec::new();
        if active_maximum_tie(&local) && prototype_complete {
            for ((real, order), permuted) in real_execution
                .candidates
                .iter()
                .zip(&order_execution.candidates)
                .zip(&permuted_execution.candidates)
            {
                if real.support_token != order.support_token
                    || real.support_token != permuted.support_token
                {
                    return Err(CorpusInducedDocumentSpinError::Invalid(
                        "independently executed arm supports are misaligned".to_owned(),
                    ));
                }
                let prototype = self.prototypes.get(&real.prototype_token).ok_or_else(|| {
                    CorpusInducedDocumentSpinError::Invalid(
                        "executed real prototype disappeared".to_owned(),
                    )
                })?;
                evidence.push(CorpusInducedDocumentSpinCandidateEvidence {
                    token: real.support_token,
                    prototype_state: real.prototype_state.trace(&self.h4_table)?,
                    prototype_document_support: prototype.construction_document_support,
                    prototype_distinct_state_support: prototype.distinct_state_support,
                    real_relative_state: real.relative_state.trace(&self.h4_table)?,
                    real_cost: real.cost,
                    order_shuffled_relative_state: order.relative_state.trace(&self.h4_table)?,
                    order_shuffled_cost: order.cost,
                    permuted_from_token: permuted.prototype_token,
                    permuted_prototype_state: permuted.prototype_state.trace(&self.h4_table)?,
                    operator_permuted_relative_state: permuted
                        .relative_state
                        .trace(&self.h4_table)?,
                    operator_permuted_cost: permuted.cost,
                });
            }
        }
        let permutation_cost_vector_changed = real_execution
            .candidates
            .iter()
            .map(|candidate| candidate.cost)
            .ne(permuted_execution
                .candidates
                .iter()
                .map(|candidate| candidate.cost));
        Ok(MatchedCorpusInducedDocumentSpinPrediction {
            local,
            operator_cid: self.operator_cid.clone(),
            natural_state: frame.real.state.trace(&self.h4_table)?,
            reverse_state: frame.order_shuffled.state.trace(&self.h4_table)?,
            candidate_evidence: evidence,
            real: real_execution.decision,
            scope_disabled: scope_execution.decision,
            order_shuffled: order_execution.decision,
            operator_permuted: permuted_execution.decision,
            prototype_complete,
            natural_reverse_distinct: frame.real.state != frame.order_shuffled.state,
            permutation_cost_vector_changed,
            support_matched,
            work_matched,
            score_firewall_certificate,
            forbidden_reads,
        })
    }
}
