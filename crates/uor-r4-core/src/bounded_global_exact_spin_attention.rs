//! Frozen bounded-global exact stored-spin contrast for issue #973.
//!
//! This operator owns no candidate admission. It ranks only the exact
//! maximum-count tie exposed by the bound #953 overlay. An independently
//! supplied immutable global snapshot is folded in stored-S3/H4, fiber, and
//! torsion state; one query-specific result is evaluated per exact snapshot
//! class and reused across repeated immutable references.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    canonical_global_epoch, shared_class_kappa, validate_h4_binary_icosahedral_closure,
    AttentionHierarchyView, AttentionOrderedFoldLevel, CanonicalLexicalCodec,
    CanonicalLexicalError, CanonicalRouteArtifact, ConversationInput, GlobalExactSpinSnapshotEntry,
    GlobalExactSpinSnapshotView, H4BinaryIcosahedralClosure, H4RootCoordinate, OpaqueH4TableIndex,
    OrderedH4FoldState, ParagraphInput, SpinTorsionStateTrace, TurnInput,
};
use crate::prime_route_attention::GeometricAddress;
use crate::prime_route_geometric_attention::H4S3AngularShell;
use crate::source_free_table::{
    d3_is_held_out, BackoffOrder, Continuation, ContinuationStop, MatchedGeometricPrediction,
    MultiscaleCountRadiusR4V1, MultiscaleCountRadiusWork, SourceDocument, SourceFreeTable,
    SourceFreeTableError, BOS_TOKEN, EOS_TOKEN, MAX_CONTINUATION_UNITS,
};

const OPERATOR_MAGIC: [u8; 8] = *b"BGESP001";
const OPERATOR_SCHEMA: u32 = 1;
const OPERATOR_DOMAIN: &str = "uor-r4.bounded-global-exact-spin-left-fold/1";
const NONCOMMUTING_OPERATOR_MAGIC: [u8; 8] = *b"BGESP002";
const NONCOMMUTING_OPERATOR_SCHEMA: u32 = 2;
const NONCOMMUTING_OPERATOR_DOMAIN: &str =
    "uor-r4.bounded-global-noncommuting-exact-spin-left-fold/2";
const SPIN_MAP_DOMAIN: &str = "uor-r4.canonical-s3-spin-to-h4/1";
const SPIN_MAP_RULE_IDENTITY: &str = "exact-s3-q30-components-divisible-by-2^29; arithmetic-right-shift-29-to-scaled-zphi-rational-coefficients; phi-coefficients-zero; unique-coordinate-membership-in-canonical-120-root-h4-table; reject-nonmultiple-nonmember-and-alias; no-prime-hash-candidate-or-nearest-root-placement";
const GRAMMAR_IDENTITY_BYTES: &[u8] = b"uor-r4 bounded global exact spin grammar/1\nconstruction=<ENTITY> bound the <ANCHOR> class.\\n\\nThe bounded global code is <CANDIDATE>.\nactive-query=The bounded global code is\nprototype-bindings=bronze->helix,teal->prism\nevaluation-snapshots=Pavel,Pavel,helix,prism|Pavel,Pavel,prism,helix\nevaluation-snapshots-are-not-fitting-inputs=true";
const ROUTING_POLICY_IDENTITY: &str = "uor-r4 bounded global exact spin routing policy/1\nmap=exact-stored-s3-to-canonical-h4-membership; q30-to-h4-shift=29; phi-coefficients=0\nfold=left-to-right exact H4 product with wrapped Q29 fiber/torsion addition\nphase-law=canonical interval [-1686629713,1686629713) with modulus 3373259426\nclass-result=C^-1*G with the same wrapped phase law\ncache-key=global-root,global-epoch,operator,map,chart,root-and-product-inverse-table,exact-class\ncost=lexicographic(h4-s3-angular-shell,fiber-circular-abs-q29,torsion-circular-abs-q29)\nselection=unique-minimum-or-abstain\ncontrols=real,identity-disabled,class-operator-permuted\nidentity-disabled=compute-all-then-return-bound-953-fallback\nclass-operator-permuted=swap-two-prototype-class-results\nscore-firewall=no-token-id,payload,address,prime,digest,ordinal,spin-sector,adjacent-row-or-target-numeric-input";
const NONCOMMUTING_GRAMMAR_IDENTITY_BYTES: &[u8] = b"uor-r4 bounded global noncommuting exact spin grammar/2\nconstruction=<ENTITY> bound the <ANCHOR> class.\\n\\nThe bounded global code is <CANDIDATE>.\nactive-query=The bounded global code is\nprototype-bindings=bronze->helix,teal->prism\nevaluation-snapshots=Lena,Lena,helix,prism|Lena,helix,Lena,prism\nevaluation-snapshots-are-not-fitting-inputs=true\nheldout-document-identities-are-evaluation-only=true";
const NONCOMMUTING_ROUTING_POLICY_IDENTITY: &str = "uor-r4 bounded global noncommuting exact spin routing policy/2\nmap=exact-stored-s3-to-canonical-h4-membership; q30-to-h4-shift=29; phi-coefficients=0\nfold=left-to-right exact H4 product with wrapped Q29 fiber/torsion addition\nphase-law=canonical interval [-1686629713,1686629713) with modulus 3373259426; phase-factors-are-central\nclass-result=C^-1*G with the same wrapped phase law\nnoncommutation-gate=direct exact H4 A*B!=B*A plus distinct nonidentity complete folds\ncache-key=global-root,global-epoch,operator,map,chart,root-and-product-inverse-table,exact-class\ncost=lexicographic(h4-s3-angular-shell,fiber-circular-abs-q29,torsion-circular-abs-q29)\nselection=unique-minimum-or-abstain\ncontrols=real,identity-disabled,class-operator-permuted\nidentity-disabled=compute-all-then-return-bound-953-fallback\nclass-operator-permuted=swap-two-prototype-class-results\nscore-firewall=no-token-id,payload,address,prime,digest,ordinal,spin-sector,adjacent-row-or-target-numeric-input";
const NONCOMMUTING_POPULATION_POLICY_IDENTITY: &str = "uor-r4 bounded global noncommuting population policy/1\npool=exact registered one-unit noncandidate nonanchor construction lexemes ordered by canonical lexical-unit-id\npool-surfaces=.,Lena,Pavel,The,bound,bounded,class,code,global,is,the\nmultiset=[D,D,helix,prism]\npermutations=unique lexicographic lexical-unit-id vectors\npairs=lexicographic left-index,right-index with left-index<right-index\nrequirements=same-exact-multiset,one-exact-transposition,three-exact-classes,one-same-address-reuse,direct-noncommutation,distinct-nonidentity-complete-folds,incompatible-unique-C^-1G-prototype-winners\nselection=first qualifying pair; no target,continuation,partition-digest,candidate-id,payload,prime,address-ordinal-or-class-digest input";
const CONSTRUCTION_IDENTITY_SCOPE: &str = "issue-973/bounded-global-exact-spin-construction-v1";
const HELD_OUT_IDENTITY_SCOPE: &str = "issue-973/bounded-global-exact-spin-heldout-v1";
const ACTIVE_TURN_ID: &str = "active-turn-0001";
const ACTIVE_QUERY_BYTES: &[u8] = b"The bounded global code is";
const CONSTRUCTION_GLOBAL_UNIT: &[u8] = b"global";
const FROZEN_CONSTRUCTION: [(&str, &[u8]); 2] = [
    (
        "51",
        b"Lena bound the helix class.\n\nThe bounded global code is bronze.",
    ),
    (
        "52",
        b"Pavel bound the prism class.\n\nThe bounded global code is teal.",
    ),
];
const LEFT_SNAPSHOT: [&[u8]; 4] = [b"Pavel", b"Pavel", b"helix", b"prism"];
const RIGHT_SNAPSHOT: [&[u8]; 4] = [b"Pavel", b"Pavel", b"prism", b"helix"];
const NONCOMMUTING_LEFT_SNAPSHOT: [&[u8]; 4] = [b"Lena", b"Lena", b"helix", b"prism"];
const NONCOMMUTING_RIGHT_SNAPSHOT: [&[u8]; 4] = [b"Lena", b"helix", b"Lena", b"prism"];
const NONCOMMUTING_DUPLICATE_POOL: [&[u8]; 11] = [
    b".", b"Lena", b"Pavel", b"The", b"bound", b"bounded", b"class", b"code", b"global", b"is",
    b"the",
];
const PROTOTYPE_BINDINGS: [(&[u8], &[u8]); 2] = [(b"bronze", b"helix"), (b"teal", b"prism")];
const PHASE_HALF_Q29: i64 = 1_686_629_713;
const PHASE_MODULUS_Q29: i64 = 3_373_259_426;
const Q29_H4_SCALE_SHIFT: u32 = 29;
const Q29_H4_SCALE_MASK: i32 = (1_i32 << Q29_H4_SCALE_SHIFT) - 1;
const TORSION_BINS: i64 = 8;

pub const BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES: usize = 2;
pub const BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES: usize = 4;
pub const BOUNDED_GLOBAL_EXACT_SPIN_CLASSES: usize = 3;
pub const BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS: u64 = 1;
pub const MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES: usize = 1024 * 1024;
pub const MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedGlobalExactSpinError {
    Invalid(String),
    SourceFree(String),
    CanonicalLexical(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for BoundedGlobalExactSpinError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::SourceFree(reason) => write!(formatter, "source-free table: {reason}"),
            Self::CanonicalLexical(reason) => write!(formatter, "canonical lexical: {reason}"),
            Self::Serialization(reason) => write!(formatter, "serialization: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("bounded-global exact-spin arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for BoundedGlobalExactSpinError {}

impl From<SourceFreeTableError> for BoundedGlobalExactSpinError {
    fn from(error: SourceFreeTableError) -> Self {
        Self::SourceFree(error.to_string())
    }
}

impl From<CanonicalLexicalError> for BoundedGlobalExactSpinError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::CanonicalLexical(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundedGlobalExactSpinArm {
    Real,
    IdentityDisabled,
    ClassOperatorPermuted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundedGlobalExactSpinAbstention {
    CostTie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct BoundedGlobalExactSpinCost {
    pub angular_shell: H4S3AngularShell,
    pub fiber_distance_q29: u64,
    pub torsion_distance_q29: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinStateTrace {
    pub h4_coordinate: H4RootCoordinate,
    pub fiber_q29: i64,
    pub torsion_q29: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalSpinSectorTrace {
    pub hopf_octant: u8,
    pub torsion_bin: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinPrototypeTrace {
    pub candidate_token: u32,
    pub candidate_hex: String,
    pub anchor_hex: String,
    pub anchor_lexical_unit_id: u32,
    pub anchor_address_kappa: String,
    pub anchor_payload_cid: String,
    pub anchor_spin: SpinTorsionStateTrace,
    pub anchor_class_kappa: String,
    pub anchor_state: BoundedGlobalExactSpinStateTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinSnapshotEntryTrace {
    pub ordinal: u16,
    pub entry_kappa: String,
    pub lexical_unit_id: u32,
    pub address_index: u16,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_hex: String,
    pub spin: SpinTorsionStateTrace,
    pub shared_class_kappa: String,
    pub mapped_state: BoundedGlobalExactSpinStateTrace,
    pub diagnostic_sector: BoundedGlobalSpinSectorTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinFoldStepTrace {
    pub ordinal: u16,
    pub entry_kappa: String,
    pub before: BoundedGlobalExactSpinStateTrace,
    pub entry: BoundedGlobalExactSpinStateTrace,
    pub after: BoundedGlobalExactSpinStateTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinClassEvaluationTrace {
    pub shared_class_kappa: String,
    pub reference_entry_kappas: Vec<String>,
    pub reference_count: u64,
    pub evaluation_count: u64,
    pub reuse_count: u64,
    pub class_state: BoundedGlobalExactSpinStateTrace,
    pub relative_result: BoundedGlobalExactSpinStateTrace,
    pub cost: BoundedGlobalExactSpinCost,
    pub result_cid: String,
    pub cold_result_cid: String,
    pub cold_recomputation_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalNoncommutingPoolRowTrace {
    pub duplicate_hex: String,
    pub duplicate_lexical_unit_id: u32,
    pub duplicate_address_kappa: String,
    pub duplicate_class_kappa: String,
    pub duplicate_state: BoundedGlobalExactSpinStateTrace,
    pub direct_noncommutation: bool,
    pub unique_permutations: u32,
    pub permutation_pairs_examined: u32,
    pub selected_pair_indices: Option<[u32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalNoncommutingWitnessTrace {
    pub left_operand_hex: String,
    pub right_operand_hex: String,
    pub left_operand: BoundedGlobalExactSpinStateTrace,
    pub right_operand: BoundedGlobalExactSpinStateTrace,
    pub left_then_right: BoundedGlobalExactSpinStateTrace,
    pub right_then_left: BoundedGlobalExactSpinStateTrace,
    pub products_distinct: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalNoncommutingCandidateCostTrace {
    pub prototype_anchor_hex: String,
    pub prototype_class_kappa: String,
    pub relative_state: BoundedGlobalExactSpinStateTrace,
    pub cost: BoundedGlobalExactSpinCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalNoncommutingPopulationAudit {
    pub schema: u32,
    pub domain: String,
    pub population_policy_kappa: String,
    pub duplicate_pool_hex: Vec<String>,
    pub rows_examined: Vec<BoundedGlobalNoncommutingPoolRowTrace>,
    pub selected_duplicate_hex: String,
    pub selected_duplicate_lexical_unit_id: u32,
    pub selected_duplicate_class_kappa: String,
    pub selected_pair_indices: [u32; 2],
    pub left_snapshot_hex: [String; 4],
    pub right_snapshot_hex: [String; 4],
    pub one_transposition: bool,
    pub transposed_ordinals: [u16; 2],
    pub noncommutation: BoundedGlobalNoncommutingWitnessTrace,
    pub left_fold: BoundedGlobalExactSpinStateTrace,
    pub right_fold: BoundedGlobalExactSpinStateTrace,
    pub distinct_nonidentity_folds: bool,
    pub complete_phase_totals_equal: bool,
    pub left_candidate_costs: Vec<BoundedGlobalNoncommutingCandidateCostTrace>,
    pub right_candidate_costs: Vec<BoundedGlobalNoncommutingCandidateCostTrace>,
    pub left_winner_anchor_hex: String,
    pub right_winner_anchor_hex: String,
    pub incompatible_unique_winners: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinCandidateEvidence {
    pub token: u32,
    pub count: u64,
    pub candidate_hex: String,
    pub prototype_anchor_hex: String,
    pub prototype_class_kappa: String,
    pub base_coordinates: crate::source_free_table::MultiscaleCountRadiusCoordinates,
    pub base_radius: u128,
    pub real_class_result_cid: String,
    pub real_relative_state: BoundedGlobalExactSpinStateTrace,
    pub real_measured_cost: BoundedGlobalExactSpinCost,
    pub real_ranking_cost: Option<BoundedGlobalExactSpinCost>,
    pub identity_disabled_measured_cost: BoundedGlobalExactSpinCost,
    pub identity_disabled_ranking_cost: Option<BoundedGlobalExactSpinCost>,
    pub permuted_class_result_cid: String,
    pub permuted_relative_state: BoundedGlobalExactSpinStateTrace,
    pub permuted_measured_cost: BoundedGlobalExactSpinCost,
    pub permuted_ranking_cost: Option<BoundedGlobalExactSpinCost>,
    pub candidate_state_kappa: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinWork {
    pub local: MultiscaleCountRadiusWork,
    pub snapshot_entry_reads: u64,
    pub exact_class_comparisons: u64,
    pub unique_class_evaluations: u64,
    pub class_reuse_hits: u64,
    pub class_result_applications: u64,
    pub h4_product_table_reads: u64,
    pub h4_inverse_table_reads: u64,
    pub phase_additions: u64,
    pub phase_distance_reads: u64,
    pub angular_shell_reads: u64,
    pub candidate_class_lookups: u64,
    pub cost_comparisons: u64,
    pub final_choice_operations: u64,
}

/// Structural score-firewall witness.
///
/// The exact scorer and winner selectors accept only exact geometric state or
/// exact costs; the focused #973 source invariant rejects forbidden score
/// capabilities. These zero fields record that boundary in the prediction
/// trace; they are not dynamic machine-instruction counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct BoundedGlobalExactSpinForbiddenReads {
    pub target_reads: u64,
    pub future_unit_reads: u64,
    pub teacher_calls: u64,
    pub provider_calls: u64,
    pub corpus_reads: u64,
    pub partition_id_reads: u64,
    pub full_history_key_reads: u64,
    pub candidate_identity_score_reads: u64,
    pub payload_score_reads: u64,
    pub address_score_reads: u64,
    pub prime_score_reads: u64,
    pub digest_score_reads: u64,
    pub class_kappa_score_reads: u64,
    pub ordinal_score_reads: u64,
    pub spin_sector_score_reads: u64,
    pub adjacent_row_score_reads: u64,
    pub construction_identity_score_reads: u64,
    pub prompt_identity_score_reads: u64,
    pub table_identity_score_reads: u64,
    pub base_evidence_score_reads: u64,
    pub lower_state_score_reads: u64,
    pub declared_work_score_reads: u64,
    pub global_summary_score_reads: u64,
    pub hierarchy_h4_score_reads: u64,
}

impl BoundedGlobalExactSpinForbiddenReads {
    pub fn total(self) -> u64 {
        self.target_reads
            + self.future_unit_reads
            + self.teacher_calls
            + self.provider_calls
            + self.corpus_reads
            + self.partition_id_reads
            + self.full_history_key_reads
            + self.candidate_identity_score_reads
            + self.payload_score_reads
            + self.address_score_reads
            + self.prime_score_reads
            + self.digest_score_reads
            + self.class_kappa_score_reads
            + self.ordinal_score_reads
            + self.spin_sector_score_reads
            + self.adjacent_row_score_reads
            + self.construction_identity_score_reads
            + self.prompt_identity_score_reads
            + self.table_identity_score_reads
            + self.base_evidence_score_reads
            + self.lower_state_score_reads
            + self.declared_work_score_reads
            + self.global_summary_score_reads
            + self.hierarchy_h4_score_reads
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinDecision {
    pub arm: BoundedGlobalExactSpinArm,
    pub token: u32,
    pub unique_minimum: Option<u32>,
    pub minimum_cost: Option<BoundedGlobalExactSpinCost>,
    pub support_tokens: Vec<u32>,
    pub work: BoundedGlobalExactSpinWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedBoundedGlobalExactSpinPrediction {
    pub local: MatchedGeometricPrediction,
    pub base_lower_artifact_manifest_kappa: String,
    pub source_snapshot_artifact_manifest_kappa: String,
    pub global_epoch: String,
    pub global_snapshot_kappa: String,
    pub global_root_kappa: String,
    pub operator_cid: String,
    pub spin_map_kappa: String,
    pub chart_profile_kappa: String,
    pub h4_root_table_kappa: String,
    pub h4_multiplication_table_kappa: String,
    pub snapshot_entries: Vec<BoundedGlobalExactSpinSnapshotEntryTrace>,
    pub fold_steps: Vec<BoundedGlobalExactSpinFoldStepTrace>,
    pub global_result: BoundedGlobalExactSpinStateTrace,
    pub class_evaluations: Vec<BoundedGlobalExactSpinClassEvaluationTrace>,
    pub candidate_evidence: Vec<BoundedGlobalExactSpinCandidateEvidence>,
    pub real: BoundedGlobalExactSpinDecision,
    pub identity_disabled: BoundedGlobalExactSpinDecision,
    pub class_operator_permuted: BoundedGlobalExactSpinDecision,
    pub support_reversed_real_token: u32,
    pub support_reversal_invariant: bool,
    pub coherent_relabel_equivariant: bool,
    pub support_matched: bool,
    pub work_matched: bool,
    pub operator_abstention: Option<BoundedGlobalExactSpinAbstention>,
    pub forbidden_reads: BoundedGlobalExactSpinForbiddenReads,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedBoundedGlobalExactSpinContinuation {
    pub first_decision: MatchedBoundedGlobalExactSpinPrediction,
    pub real: Continuation,
    pub identity_disabled: Continuation,
    pub class_operator_permuted: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedBoundedGlobalNoncommutingPairPrediction {
    pub population_audit: BoundedGlobalNoncommutingPopulationAudit,
    pub left: MatchedBoundedGlobalExactSpinPrediction,
    pub right: MatchedBoundedGlobalExactSpinPrediction,
    pub exact_fold_distinct: bool,
    pub real_winners_incompatible: bool,
    pub permuted_winners_incompatible: bool,
    pub common_lower_artifact: bool,
    pub support_matched_between_cases: bool,
    pub work_matched_between_cases: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedBoundedGlobalNoncommutingPairContinuation {
    pub first_pair: MatchedBoundedGlobalNoncommutingPairPrediction,
    pub left: MatchedBoundedGlobalExactSpinContinuation,
    pub right: MatchedBoundedGlobalExactSpinContinuation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinHierarchyLevelAudit {
    pub level: String,
    pub left_identity_kappa: String,
    pub right_identity_kappa: String,
    pub identity_equal: bool,
    pub left_state: OrderedH4FoldState,
    pub right_state: OrderedH4FoldState,
    pub ordered_state_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundedGlobalExactSpinHierarchyAudit {
    pub levels: Vec<BoundedGlobalExactSpinHierarchyLevelAudit>,
    pub lower_through_conversation_identity_equal: bool,
    pub lower_through_conversation_ordered_state_equal: bool,
    pub global_identity_distinct: bool,
    pub global_ordered_state_distinct: bool,
    pub left_global_snapshot_kappa: String,
    pub right_global_snapshot_kappa: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExactSpinState {
    h4: OrderedH4FoldState,
    fiber_q29: i64,
    torsion_q29: i64,
}

impl ExactSpinState {
    pub(crate) fn from_parts(
        h4: OrderedH4FoldState,
        fiber_q29: i64,
        torsion_q29: i64,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        h4.root_coordinate(table)?;
        Ok(Self {
            h4,
            fiber_q29: wrap_phase_q29(fiber_q29),
            torsion_q29: wrap_phase_q29(torsion_q29),
        })
    }

    pub(crate) fn from_table_index_and_phases(
        table_index: OpaqueH4TableIndex,
        fiber_q29: i64,
        torsion_q29: i64,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        let h4 = OrderedH4FoldState::from_table_index(table_index, table)?;
        Self::from_parts(h4, fiber_q29, torsion_q29, table)
    }

    pub(crate) fn from_spin_trace(
        spin: SpinTorsionStateTrace,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        let h4 = exact_s3_spin_to_h4(spin.s3_q30, table)?;
        Self::from_parts(
            h4,
            i64::from(spin.fiber_q29),
            i64::from(spin.torsion_q29),
            table,
        )
    }

    pub(crate) fn identity(
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        Ok(Self {
            h4: OrderedH4FoldState::identity(table)?,
            fiber_q29: 0,
            torsion_q29: 0,
        })
    }

    pub(crate) fn compose(
        self,
        right: Self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        Ok(Self {
            h4: self.h4.compose(right.h4, table)?,
            fiber_q29: wrap_phase_q29(
                self.fiber_q29
                    .checked_add(right.fiber_q29)
                    .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                self.torsion_q29
                    .checked_add(right.torsion_q29)
                    .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            ),
        })
    }

    pub(crate) fn inverse(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        Ok(Self {
            h4: self.h4.inverse(table)?,
            fiber_q29: wrap_phase_q29(
                self.fiber_q29
                    .checked_neg()
                    .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                self.torsion_q29
                    .checked_neg()
                    .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            ),
        })
    }

    pub(crate) fn trace(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<BoundedGlobalExactSpinStateTrace, BoundedGlobalExactSpinError> {
        Ok(BoundedGlobalExactSpinStateTrace {
            h4_coordinate: self.h4.root_coordinate(table)?,
            fiber_q29: self.fiber_q29,
            torsion_q29: self.torsion_q29,
        })
    }

    pub(crate) const fn table_index(self) -> OpaqueH4TableIndex {
        self.h4.table_index()
    }

    pub(crate) fn root_coordinate(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<H4RootCoordinate, BoundedGlobalExactSpinError> {
        Ok(self.h4.root_coordinate(table)?)
    }

    pub(crate) const fn fiber_q29(self) -> i64 {
        self.fiber_q29
    }

    pub(crate) const fn torsion_q29(self) -> i64 {
        self.torsion_q29
    }

    pub(crate) fn root_real(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<[i64; 2], BoundedGlobalExactSpinError> {
        Ok(self.root_coordinate(table)?.scaled_zphi_quaternion[0])
    }
}

#[derive(Debug, Clone)]
struct CandidatePrototype {
    candidate_token: u32,
    candidate_bytes: Vec<u8>,
    anchor_bytes: Vec<u8>,
    anchor_lexical_unit_id: u32,
    anchor_address: GeometricAddress,
    anchor_address_kappa: String,
    anchor_payload_cid: String,
    anchor_class_kappa: String,
    anchor_state: ExactSpinState,
}

#[derive(Debug, Clone)]
pub struct BoundedGlobalExactSpinR4V1 {
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_ids: Vec<String>,
    construction_text_cids: Vec<[u8; 32]>,
    codec: CanonicalLexicalCodec,
    construction_artifact: CanonicalRouteArtifact,
    h4_table: H4BinaryIcosahedralClosure,
    grammar_kappa: String,
    routing_policy_kappa: String,
    spin_map_kappa: String,
    chart_profile_kappa: String,
    prototypes: Vec<CandidatePrototype>,
}

/// Versioned successor to the target-free V1 relation failure.
///
/// V2 reuses the exact construction-bound codec, address registry, stored-spin
/// map, prototype bindings, and `C^-1 * G` least-cost relation. Its additional
/// population audit proves that the two detached global carriers differ by an
/// actual noncommuting stored-H4 transposition before either case may decode.
#[derive(Debug, Clone)]
pub struct BoundedGlobalNoncommutingExactSpinR4V2 {
    core: BoundedGlobalExactSpinR4V1,
    grammar_kappa: String,
    routing_policy_kappa: String,
    population_policy_kappa: String,
    population_audit: BoundedGlobalNoncommutingPopulationAudit,
}

#[derive(Debug, Clone)]
struct ArmEvaluation {
    entries: Vec<BoundedGlobalExactSpinSnapshotEntryTrace>,
    fold_steps: Vec<BoundedGlobalExactSpinFoldStepTrace>,
    global_result: ExactSpinState,
    classes: Vec<BoundedGlobalExactSpinClassEvaluationTrace>,
    candidate_rows: Vec<ArmCandidateRow>,
    decision: BoundedGlobalExactSpinDecision,
}

#[derive(Debug, Clone)]
struct ArmCandidateRow {
    token: u32,
    class_result_cid: String,
    relative_state: BoundedGlobalExactSpinStateTrace,
    measured_cost: BoundedGlobalExactSpinCost,
    ranking_cost: Option<BoundedGlobalExactSpinCost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapshotContract {
    FrozenCommutingV1,
    FrozenNoncommutingV2,
}

#[derive(Debug, Clone)]
struct PopulationLexeme {
    bytes: Vec<u8>,
    lexical_unit_id: u32,
    address_kappa: String,
    class_kappa: String,
    spin: SpinTorsionStateTrace,
    state: ExactSpinState,
}

#[derive(Debug, Clone)]
struct PopulationSelection {
    duplicate: PopulationLexeme,
    left_index: usize,
    right_index: usize,
    left_ids: [u32; 4],
    right_ids: [u32; 4],
    left_fold: ExactSpinState,
    right_fold: ExactSpinState,
    left_costs: [BoundedGlobalNoncommutingCandidateCostTrace; 2],
    right_costs: [BoundedGlobalNoncommutingCandidateCostTrace; 2],
    left_winner: usize,
    right_winner: usize,
    left_operand: PopulationLexeme,
    right_operand: PopulationLexeme,
    left_then_right: ExactSpinState,
    right_then_left: ExactSpinState,
}

impl BoundedGlobalExactSpinR4V1 {
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        if base_overlay.table_artifact_cid() != table.artifact_cid() {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "#953 overlay table binding mismatches".to_owned(),
            ));
        }
        if construction.len() != FROZEN_CONSTRUCTION.len()
            || construction
                .iter()
                .any(|document| d3_is_held_out(&document.id))
            || !table.is_bound_to_construction_documents(construction)
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "operator is not bound to the exact D3 construction set".to_owned(),
            ));
        }
        let mut sorted = construction.to_vec();
        sorted.sort_by(|left, right| left.id.cmp(&right.id));
        for (document, (expected_id, expected_text)) in sorted.iter().zip(FROZEN_CONSTRUCTION) {
            if document.id != expected_id || document.text.as_slice() != expected_text {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "construction differs from the frozen bounded-global documents".to_owned(),
                ));
            }
        }
        if SourceFreeTable::compile(&sorted)?.artifact_cid() != table.artifact_cid() {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "source-free table does not reproduce from frozen construction".to_owned(),
            ));
        }

        let geometry_input = construction_geometry_input(&sorted)?;
        let codec = CanonicalLexicalCodec::compile(&geometry_input)?;
        let construction_artifact = CanonicalRouteArtifact::ingest(&codec, &geometry_input)?;
        let chart_profile_kappa = construction_artifact
            .attention_consumer_trace()?
            .chart_profile_kappa;
        let h4_table = validate_h4_binary_icosahedral_closure()?;
        let grammar_kappa = blake3_label(GRAMMAR_IDENTITY_BYTES);
        let routing_policy_kappa = blake3_label(ROUTING_POLICY_IDENTITY.as_bytes());
        let spin_map_kappa = spin_map_kappa(&h4_table)?;

        let mut prototypes = Vec::with_capacity(PROTOTYPE_BINDINGS.len());
        for (candidate, anchor) in PROTOTYPE_BINDINGS {
            let mut candidate_bytes = Vec::with_capacity(candidate.len() + 1);
            candidate_bytes.push(b' ');
            candidate_bytes.extend_from_slice(candidate);
            let candidate_tokens = table.encode_text(&candidate_bytes)?;
            if candidate_tokens.len() != 1
                || !table.is_fitted_lexical_token(candidate_tokens[0])
                || table.decode_tokens(&candidate_tokens)? != candidate_bytes
            {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "prototype candidate is not one exact fitted lexical unit".to_owned(),
                ));
            }
            let encoded_anchor = codec.encode(0, 0, anchor)?;
            if encoded_anchor.units.len() != 1 || !encoded_anchor.trailing_bytes.is_empty() {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "prototype anchor is not one exact canonical lexical unit".to_owned(),
                ));
            }
            let unit_id = encoded_anchor.units[0].unit_id;
            let address = construction_artifact
                .lexical_route_address_from_validated_artifact(unit_id)?
                .ok_or_else(|| {
                    BoundedGlobalExactSpinError::Invalid(
                        "prototype anchor has no registered address".to_owned(),
                    )
                })?;
            let value = construction_artifact
                .lexical_route_value_for_address_from_validated_artifact(&address)?
                .ok_or_else(|| {
                    BoundedGlobalExactSpinError::Invalid(
                        "prototype anchor address has no payload inversion".to_owned(),
                    )
                })?;
            if value.payload_bytes != anchor {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "prototype anchor payload inversion mismatches".to_owned(),
                ));
            }
            let state = exact_state_from_address(&address, &h4_table)?;
            prototypes.push(CandidatePrototype {
                candidate_token: candidate_tokens[0],
                candidate_bytes,
                anchor_bytes: anchor.to_vec(),
                anchor_lexical_unit_id: unit_id,
                anchor_address_kappa: value.address_kappa,
                anchor_payload_cid: address.payload_cid.clone(),
                anchor_class_kappa: shared_class_kappa(address.spin)?,
                anchor_address: address,
                anchor_state: state,
            });
        }
        prototypes.sort_by_key(|prototype| prototype.candidate_token);
        if prototypes.len() != BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES
            || prototypes.windows(2).any(|pair| {
                pair[0].candidate_token >= pair[1].candidate_token
                    || pair[0].anchor_class_kappa == pair[1].anchor_class_kappa
            })
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "prototype tokens or exact anchor classes alias".to_owned(),
            ));
        }

        let operator = Self {
            table_artifact_cid: table.artifact_cid(),
            base_overlay_artifact_cid: base_overlay.artifact_cid(),
            construction_ids: sorted.iter().map(|document| document.id.clone()).collect(),
            construction_text_cids: sorted.iter().map(SourceDocument::text_cid).collect(),
            codec,
            construction_artifact,
            h4_table,
            grammar_kappa,
            routing_policy_kappa,
            spin_map_kappa,
            chart_profile_kappa,
            prototypes,
        };
        operator.validate_binding(table, base_overlay)?;
        if operator.to_bytes()?.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(operator)
    }

    pub fn from_bytes(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
        bytes: &[u8],
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        if bytes.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator exceeds its byte ceiling".to_owned(),
            ));
        }
        if bytes.len() < OPERATOR_MAGIC.len() || bytes[..OPERATOR_MAGIC.len()] != OPERATOR_MAGIC {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator magic is invalid".to_owned(),
            ));
        }
        let expected = Self::compile(table, base_overlay, construction)?;
        if expected.to_bytes()? != bytes {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator is noncanonical or binding-drifted".to_owned(),
            ));
        }
        Ok(expected)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, BoundedGlobalExactSpinError> {
        let wire = OperatorWire {
            schema: OPERATOR_SCHEMA,
            domain: OPERATOR_DOMAIN,
            table_artifact_cid: self.table_artifact_cid.clone(),
            base_overlay_artifact_cid: self.base_overlay_artifact_cid.clone(),
            construction_ids: self.construction_ids.clone(),
            construction_text_cids: self
                .construction_text_cids
                .iter()
                .map(hex::encode)
                .collect(),
            codec_kappa: self.construction_artifact.codec_kappa().to_owned(),
            vocabulary_kappa: self.construction_artifact.vocabulary_kappa().to_owned(),
            route_manifest_kappa: self.construction_artifact.manifest_kappa().to_owned(),
            h4_root_table_kappa: self.h4_table.h4_root_table_kappa.clone(),
            h4_multiplication_table_kappa: self.h4_table.multiplication_table_kappa.clone(),
            grammar_kappa: self.grammar_kappa.clone(),
            routing_policy_kappa: self.routing_policy_kappa.clone(),
            spin_map_kappa: self.spin_map_kappa.clone(),
            chart_profile_kappa: self.chart_profile_kappa.clone(),
            construction_identity_scope: CONSTRUCTION_IDENTITY_SCOPE,
            held_out_identity_scope: HELD_OUT_IDENTITY_SCOPE,
            active_turn_id: ACTIVE_TURN_ID,
            active_query_hex: hex::encode(ACTIVE_QUERY_BYTES),
            construction_global_unit_hex: hex::encode(CONSTRUCTION_GLOBAL_UNIT),
            left_snapshot_hex: LEFT_SNAPSHOT.map(hex::encode),
            right_snapshot_hex: RIGHT_SNAPSHOT.map(hex::encode),
            candidates: BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES,
            snapshot_entries: BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES,
            snapshot_classes: BOUNDED_GLOBAL_EXACT_SPIN_CLASSES,
            reuse_hits: BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS,
            max_query_bytes: MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES,
            max_operator_bytes: MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES,
            prototypes: self.prototype_traces()?,
        };
        let payload = serde_json::to_vec(&wire)
            .map_err(|error| BoundedGlobalExactSpinError::Serialization(error.to_string()))?;
        let mut bytes = Vec::with_capacity(OPERATOR_MAGIC.len() + payload.len());
        bytes.extend_from_slice(&OPERATOR_MAGIC);
        bytes.extend_from_slice(&payload);
        if bytes.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn artifact_cid(&self) -> Result<String, BoundedGlobalExactSpinError> {
        Ok(blake3_label(&self.to_bytes()?))
    }

    pub fn table_artifact_cid(&self) -> &str {
        &self.table_artifact_cid
    }

    pub fn base_overlay_artifact_cid(&self) -> &str {
        &self.base_overlay_artifact_cid
    }

    pub fn codec_kappa(&self) -> &str {
        self.construction_artifact.codec_kappa()
    }

    pub fn vocabulary_kappa(&self) -> &str {
        self.construction_artifact.vocabulary_kappa()
    }

    pub fn route_manifest_kappa(&self) -> &str {
        self.construction_artifact.manifest_kappa()
    }

    pub fn spin_map_kappa(&self) -> &str {
        &self.spin_map_kappa
    }

    pub fn chart_profile_kappa(&self) -> &str {
        &self.chart_profile_kappa
    }

    pub fn grammar_kappa(&self) -> &str {
        &self.grammar_kappa
    }

    pub fn routing_policy_kappa(&self) -> &str {
        &self.routing_policy_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_table.h4_root_table_kappa
    }

    pub fn h4_multiplication_table_kappa(&self) -> &str {
        &self.h4_table.multiplication_table_kappa
    }

    pub fn prototype_traces(
        &self,
    ) -> Result<Vec<BoundedGlobalExactSpinPrototypeTrace>, BoundedGlobalExactSpinError> {
        self.prototypes
            .iter()
            .map(|prototype| {
                Ok(BoundedGlobalExactSpinPrototypeTrace {
                    candidate_token: prototype.candidate_token,
                    candidate_hex: hex::encode(&prototype.candidate_bytes),
                    anchor_hex: hex::encode(&prototype.anchor_bytes),
                    anchor_lexical_unit_id: prototype.anchor_lexical_unit_id,
                    anchor_address_kappa: prototype.anchor_address_kappa.clone(),
                    anchor_payload_cid: prototype.anchor_payload_cid.clone(),
                    anchor_spin: spin_trace(prototype.anchor_address.spin),
                    anchor_class_kappa: prototype.anchor_class_kappa.clone(),
                    anchor_state: prototype.anchor_state.trace(&self.h4_table)?,
                })
            })
            .collect()
    }

    pub fn build_query_artifact(
        &self,
        active_query: &[u8],
    ) -> Result<CanonicalRouteArtifact, BoundedGlobalExactSpinError> {
        validate_active_query(active_query)?;
        let input = observed_base_input(active_query)?;
        Ok(CanonicalRouteArtifact::ingest(&self.codec, &input)?)
    }

    pub fn build_snapshot_artifact(
        &self,
        active_query: &[u8],
        global_snapshot_units: &[Vec<u8>],
    ) -> Result<CanonicalRouteArtifact, BoundedGlobalExactSpinError> {
        validate_active_query(active_query)?;
        validate_snapshot_units_for(global_snapshot_units, SnapshotContract::FrozenCommutingV1)?;
        let input = observed_global_input_for(
            active_query,
            global_snapshot_units,
            SnapshotContract::FrozenCommutingV1,
        )?;
        Ok(CanonicalRouteArtifact::ingest(&self.codec, &input)?)
    }

    pub fn predict_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        base_artifact: &CanonicalRouteArtifact,
        snapshot_artifact: &CanonicalRouteArtifact,
        active_query: &[u8],
    ) -> Result<MatchedBoundedGlobalExactSpinPrediction, BoundedGlobalExactSpinError> {
        let operator_cid = self.artifact_cid()?;
        self.predict_matched_for(
            table,
            base_overlay,
            base_artifact,
            snapshot_artifact,
            active_query,
            SnapshotContract::FrozenCommutingV1,
            &operator_cid,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn predict_matched_for(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        base_artifact: &CanonicalRouteArtifact,
        snapshot_artifact: &CanonicalRouteArtifact,
        active_query: &[u8],
        snapshot_contract: SnapshotContract,
        operator_cid: &str,
    ) -> Result<MatchedBoundedGlobalExactSpinPrediction, BoundedGlobalExactSpinError> {
        self.validate_binding(table, base_overlay)?;
        validate_active_query(active_query)?;
        let base_input = base_artifact.reconstruct_input()?;
        if base_input != observed_base_input(active_query)?
            || base_artifact.codec_kappa() != self.codec.codec_kappa()
            || base_artifact.vocabulary_kappa() != self.codec.vocabulary_kappa()
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "base query artifact is not the exact frozen lower input".to_owned(),
            ));
        }
        let snapshot_input = snapshot_artifact.reconstruct_input()?;
        if snapshot_input
            != observed_global_input_for(
                active_query,
                &snapshot_input.global_snapshot_units,
                snapshot_contract,
            )?
            || snapshot_artifact.codec_kappa() != self.codec.codec_kappa()
            || snapshot_artifact.vocabulary_kappa() != self.codec.vocabulary_kappa()
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "snapshot artifact is not an exact frozen global input".to_owned(),
            ));
        }
        validate_snapshot_units_for(&snapshot_input.global_snapshot_units, snapshot_contract)?;
        let view = snapshot_artifact.global_exact_spin_snapshot_view()?;
        if view.global_epoch != snapshot_input.global_epoch
            || view.snapshot_kappa != snapshot_input.global_epoch
            || view.entries.len() != BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "global exact-spin view mismatches the frozen snapshot".to_owned(),
            ));
        }

        for prototype in &self.prototypes {
            if contains_bytes(active_query, &prototype.candidate_bytes)
                || view
                    .entries
                    .iter()
                    .any(|entry| entry.payload_bytes == prototype.candidate_bytes[1..])
            {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "observed prompt or global snapshot contains an admitted candidate".to_owned(),
                ));
            }
        }

        let mut context = vec![BOS_TOKEN];
        context.extend(table.encode_text(active_query)?);
        let local = table.predict_multiscale_count_radius(&context, base_overlay)?;
        let prototype_tokens = self
            .prototypes
            .iter()
            .map(|prototype| prototype.candidate_token)
            .collect::<Vec<_>>();
        if local.order != BackoffOrder::Trigram
            || local.max_count != 1
            || local.max_count_tie_tokens != prototype_tokens
            || local.baseline_support_tokens != local.geometric_support_tokens
            || local.baseline_work != local.geometric_work
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "#953 admission is not the frozen exact count-one tie".to_owned(),
            ));
        }

        let real = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::Real,
            false,
            operator_cid,
        )?;
        let disabled = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::IdentityDisabled,
            false,
            operator_cid,
        )?;
        let permuted = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::ClassOperatorPermuted,
            false,
            operator_cid,
        )?;
        let reversed = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::Real,
            true,
            operator_cid,
        )?;

        if real.entries != disabled.entries
            || real.entries != permuted.entries
            || real.fold_steps != disabled.fold_steps
            || real.fold_steps != permuted.fold_steps
            || real.classes != disabled.classes
            || real.classes != permuted.classes
            || real.decision.work != disabled.decision.work
            || real.decision.work != permuted.decision.work
            || real.decision.support_tokens != disabled.decision.support_tokens
            || real.decision.support_tokens != permuted.decision.support_tokens
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "matched global arms differ in inputs, class results, support, or work".to_owned(),
            ));
        }

        let mut candidate_evidence = Vec::with_capacity(self.prototypes.len());
        for (index, prototype) in self.prototypes.iter().enumerate() {
            let base = local.tie_evidence.get(index).ok_or_else(|| {
                BoundedGlobalExactSpinError::Invalid(
                    "#953 tie evidence omitted a prototype candidate".to_owned(),
                )
            })?;
            if base.token != prototype.candidate_token
                || real.candidate_rows[index].token != prototype.candidate_token
                || disabled.candidate_rows[index].token != prototype.candidate_token
                || permuted.candidate_rows[index].token != prototype.candidate_token
            {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "candidate evidence order differs from bound prototype order".to_owned(),
                ));
            }
            let real_row = &real.candidate_rows[index];
            let disabled_row = &disabled.candidate_rows[index];
            let permuted_row = &permuted.candidate_rows[index];
            candidate_evidence.push(BoundedGlobalExactSpinCandidateEvidence {
                token: prototype.candidate_token,
                count: base.count,
                candidate_hex: hex::encode(&prototype.candidate_bytes),
                prototype_anchor_hex: hex::encode(&prototype.anchor_bytes),
                prototype_class_kappa: prototype.anchor_class_kappa.clone(),
                base_coordinates: base.coordinates,
                base_radius: base.radius,
                real_class_result_cid: real_row.class_result_cid.clone(),
                real_relative_state: real_row.relative_state,
                real_measured_cost: real_row.measured_cost,
                real_ranking_cost: real_row.ranking_cost,
                identity_disabled_measured_cost: disabled_row.measured_cost,
                identity_disabled_ranking_cost: disabled_row.ranking_cost,
                permuted_class_result_cid: permuted_row.class_result_cid.clone(),
                permuted_relative_state: permuted_row.relative_state,
                permuted_measured_cost: permuted_row.measured_cost,
                permuted_ranking_cost: permuted_row.ranking_cost,
                candidate_state_kappa: candidate_state_kappa(
                    real.global_result,
                    prototype.anchor_state,
                    real_row.measured_cost,
                    &self.h4_table,
                )?,
            });
        }

        let support_reversal_invariant = reversed.decision.token == real.decision.token;
        let coherent_relabel_equivariant = coherent_relabel_equivariant(
            &real.candidate_rows,
            real.decision.token,
            &prototype_tokens,
        )?;
        let work_matched = real.decision.work == disabled.decision.work
            && real.decision.work == permuted.decision.work;
        let support_matched = real.decision.support_tokens == disabled.decision.support_tokens
            && real.decision.support_tokens == permuted.decision.support_tokens;
        let operator_abstention = real
            .decision
            .unique_minimum
            .is_none()
            .then_some(BoundedGlobalExactSpinAbstention::CostTie);

        Ok(MatchedBoundedGlobalExactSpinPrediction {
            local,
            base_lower_artifact_manifest_kappa: base_artifact.manifest_kappa().to_owned(),
            source_snapshot_artifact_manifest_kappa: view.source_artifact_manifest_kappa,
            global_epoch: view.global_epoch,
            global_snapshot_kappa: view.snapshot_kappa,
            global_root_kappa: view.global_root_kappa,
            operator_cid: operator_cid.to_owned(),
            spin_map_kappa: self.spin_map_kappa.clone(),
            chart_profile_kappa: self.chart_profile_kappa.clone(),
            h4_root_table_kappa: self.h4_table.h4_root_table_kappa.clone(),
            h4_multiplication_table_kappa: self.h4_table.multiplication_table_kappa.clone(),
            snapshot_entries: real.entries,
            fold_steps: real.fold_steps,
            global_result: real.global_result.trace(&self.h4_table)?,
            class_evaluations: real.classes,
            candidate_evidence,
            real: real.decision,
            identity_disabled: disabled.decision,
            class_operator_permuted: permuted.decision,
            support_reversed_real_token: reversed.decision.token,
            support_reversal_invariant,
            coherent_relabel_equivariant,
            support_matched,
            work_matched,
            operator_abstention,
            forbidden_reads: BoundedGlobalExactSpinForbiddenReads::default(),
        })
    }

    pub fn continue_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        base_artifact: &CanonicalRouteArtifact,
        snapshot_artifact: &CanonicalRouteArtifact,
        active_query: &[u8],
        max_units: usize,
    ) -> Result<MatchedBoundedGlobalExactSpinContinuation, BoundedGlobalExactSpinError> {
        validate_continuation_bound(max_units)?;
        let first_decision = self.predict_matched(
            table,
            base_overlay,
            base_artifact,
            snapshot_artifact,
            active_query,
        )?;
        continue_from_prediction(table, base_overlay, active_query, max_units, first_decision)
    }

    pub fn audit_hierarchy_pair(
        &self,
        left: &CanonicalRouteArtifact,
        right: &CanonicalRouteArtifact,
    ) -> Result<BoundedGlobalExactSpinHierarchyAudit, BoundedGlobalExactSpinError> {
        let left_view = left.attention_hierarchy_view();
        let right_view = right.attention_hierarchy_view();
        let left_ordered = left.attention_consumer_trace_with_ordered_h4(&self.h4_table)?;
        let right_ordered = right.attention_consumer_trace_with_ordered_h4(&self.h4_table)?;
        if left_ordered.ordered_levels.len() != 7 || right_ordered.ordered_levels.len() != 7 {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "hierarchy audit did not expose seven ordered levels".to_owned(),
            ));
        }
        let left_ids = hierarchy_identities(&left_view);
        let right_ids = hierarchy_identities(&right_view);
        let mut levels = Vec::with_capacity(7);
        for index in 0..7 {
            let left_level = &left_ordered.ordered_levels[index];
            let right_level = &right_ordered.ordered_levels[index];
            if left_level.level != HIERARCHY_LEVELS[index]
                || right_level.level != HIERARCHY_LEVELS[index]
            {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "hierarchy level order differs from the fixed consumer order".to_owned(),
                ));
            }
            levels.push(hierarchy_level_audit(
                HIERARCHY_LEVELS[index],
                left_ids[index],
                right_ids[index],
                left_level,
                right_level,
            ));
        }
        Ok(BoundedGlobalExactSpinHierarchyAudit {
            lower_through_conversation_identity_equal: levels[..6]
                .iter()
                .all(|level| level.identity_equal),
            lower_through_conversation_ordered_state_equal: levels[..6]
                .iter()
                .all(|level| level.ordered_state_equal),
            global_identity_distinct: !levels[6].identity_equal,
            global_ordered_state_distinct: !levels[6].ordered_state_equal,
            left_global_snapshot_kappa: left.global_exact_spin_snapshot_view()?.snapshot_kappa,
            right_global_snapshot_kappa: right.global_exact_spin_snapshot_view()?.snapshot_kappa,
            levels,
        })
    }

    fn validate_binding(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), BoundedGlobalExactSpinError> {
        if self.table_artifact_cid != table.artifact_cid()
            || self.base_overlay_artifact_cid != base_overlay.artifact_cid()
            || base_overlay.table_artifact_cid() != table.artifact_cid()
            || self.codec.codec_kappa() != self.construction_artifact.codec_kappa()
            || self.codec.vocabulary_kappa() != self.construction_artifact.vocabulary_kappa()
            || self.h4_table.reproduce_multiplication_table_kappa()?
                != self.h4_table.multiplication_table_kappa
            || spin_map_kappa(&self.h4_table)? != self.spin_map_kappa
            || blake3_label(GRAMMAR_IDENTITY_BYTES) != self.grammar_kappa
            || blake3_label(ROUTING_POLICY_IDENTITY.as_bytes()) != self.routing_policy_kappa
            || self
                .construction_artifact
                .attention_consumer_trace()?
                .chart_profile_kappa
                != self.chart_profile_kappa
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global operator binding does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }

    fn evaluate_arm(
        &self,
        view: &GlobalExactSpinSnapshotView,
        local: &MatchedGeometricPrediction,
        arm: BoundedGlobalExactSpinArm,
        reverse_support_iteration: bool,
        operator_cid: &str,
    ) -> Result<ArmEvaluation, BoundedGlobalExactSpinError> {
        let mut work = WorkCounter::new(local.geometric_work);
        let mut entries = Vec::with_capacity(view.entries.len());
        let mut states = Vec::with_capacity(view.entries.len());
        for entry in &view.entries {
            work.snapshot_entry_reads += 1;
            let state = exact_state_from_entry(entry, &self.h4_table)?;
            states.push(state);
            entries.push(BoundedGlobalExactSpinSnapshotEntryTrace {
                ordinal: entry.ordinal,
                entry_kappa: entry.entry_kappa.clone(),
                lexical_unit_id: entry.lexical_unit_id,
                address_index: entry.address_index,
                address_kappa: entry.address_kappa.clone(),
                payload_cid: entry.payload_cid.clone(),
                payload_hex: hex::encode(&entry.payload_bytes),
                spin: entry.spin,
                shared_class_kappa: entry.shared_class_kappa.clone(),
                mapped_state: state.trace(&self.h4_table)?,
                diagnostic_sector: diagnostic_sector(entry.spin),
            });
        }

        let mut fold = ExactSpinState::identity(&self.h4_table)?;
        let mut fold_steps = Vec::with_capacity(states.len());
        for (entry, state) in view.entries.iter().zip(states.iter().copied()) {
            let before = fold;
            fold = fold.compose(state, &self.h4_table)?;
            work.h4_product_table_reads += 1;
            work.phase_additions += 2;
            fold_steps.push(BoundedGlobalExactSpinFoldStepTrace {
                ordinal: entry.ordinal,
                entry_kappa: entry.entry_kappa.clone(),
                before: before.trace(&self.h4_table)?,
                entry: state.trace(&self.h4_table)?,
                after: fold.trace(&self.h4_table)?,
            });
        }
        if fold == ExactSpinState::identity(&self.h4_table)? {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "global stored-spin fold is identity".to_owned(),
            ));
        }

        let mut unique = Vec::<ClassAccumulator>::new();
        for ((entry, trace), state) in view
            .entries
            .iter()
            .zip(entries.iter())
            .zip(states.iter().copied())
        {
            let mut found = None;
            for (index, class) in unique.iter().enumerate() {
                work.exact_class_comparisons += 1;
                if class.class_kappa == entry.shared_class_kappa {
                    found = Some(index);
                    break;
                }
            }
            if let Some(index) = found {
                let class = &mut unique[index];
                if class.state != state
                    || class.address_kappa != trace.address_kappa
                    || class.payload_cid != trace.payload_cid
                {
                    return Err(BoundedGlobalExactSpinError::Invalid(
                        "one exact class aliases different state/address/payload".to_owned(),
                    ));
                }
                class.reference_entry_kappas.push(entry.entry_kappa.clone());
                work.class_reuse_hits += 1;
            } else {
                unique.push(ClassAccumulator {
                    class_kappa: entry.shared_class_kappa.clone(),
                    address_kappa: trace.address_kappa.clone(),
                    payload_cid: trace.payload_cid.clone(),
                    state,
                    reference_entry_kappas: vec![entry.entry_kappa.clone()],
                });
            }
        }
        if unique.len() != BOUNDED_GLOBAL_EXACT_SPIN_CLASSES
            || work.class_reuse_hits != BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "snapshot exact-class population differs from the frozen census".to_owned(),
            ));
        }

        let result_binding = ClassResultBinding {
            global_root_kappa: &view.global_root_kappa,
            global_epoch: &view.global_epoch,
            operator_cid,
            spin_map_kappa: &self.spin_map_kappa,
            chart_profile_kappa: &self.chart_profile_kappa,
            table: &self.h4_table,
        };
        let mut class_results = Vec::<ClassResult>::with_capacity(unique.len());
        let mut class_traces = Vec::with_capacity(unique.len());
        for class in unique {
            let (relative, cost) =
                candidate_relative_exact_cost(class.state, fold, &self.h4_table)?;
            work.h4_inverse_table_reads += 1;
            work.h4_product_table_reads += 1;
            work.phase_additions += 2;
            work.angular_shell_reads += 1;
            work.phase_distance_reads += 2;
            work.unique_class_evaluations += 1;
            work.class_result_applications += u64::try_from(class.reference_entry_kappas.len())
                .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?;
            let (cold, cold_cost) =
                candidate_relative_exact_cost(class.state, fold, &self.h4_table)?;
            work.h4_inverse_table_reads += 1;
            work.h4_product_table_reads += 1;
            work.phase_additions += 2;
            work.angular_shell_reads += 1;
            work.phase_distance_reads += 2;
            let result_bytes =
                class_result_bytes(&result_binding, &class.class_kappa, relative, cost)?;
            let cold_result_bytes =
                class_result_bytes(&result_binding, &class.class_kappa, cold, cold_cost)?;
            let result_cid = blake3_label(&result_bytes);
            let cold_result_cid = blake3_label(&cold_result_bytes);
            let trace = BoundedGlobalExactSpinClassEvaluationTrace {
                shared_class_kappa: class.class_kappa.clone(),
                reference_count: u64::try_from(class.reference_entry_kappas.len())
                    .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
                evaluation_count: 1,
                reuse_count: u64::try_from(class.reference_entry_kappas.len().saturating_sub(1))
                    .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
                reference_entry_kappas: class.reference_entry_kappas,
                class_state: class.state.trace(&self.h4_table)?,
                relative_result: relative.trace(&self.h4_table)?,
                cost,
                result_cid: result_cid.clone(),
                cold_result_cid: cold_result_cid.clone(),
                cold_recomputation_equal: result_bytes == cold_result_bytes,
            };
            class_results.push(ClassResult {
                class_kappa: class.class_kappa,
                relative,
                cost,
                result_cid,
            });
            class_traces.push(trace);
        }

        let mut candidate_rows = Vec::with_capacity(self.prototypes.len());
        for (index, prototype) in self.prototypes.iter().enumerate() {
            work.candidate_class_lookups += 1;
            let source_index = match arm {
                BoundedGlobalExactSpinArm::ClassOperatorPermuted => {
                    (index + 1) % self.prototypes.len()
                }
                BoundedGlobalExactSpinArm::Real | BoundedGlobalExactSpinArm::IdentityDisabled => {
                    index
                }
            };
            let source_class = &self.prototypes[source_index].anchor_class_kappa;
            let result = class_results
                .iter()
                .find(|result| result.class_kappa == *source_class)
                .ok_or_else(|| {
                    BoundedGlobalExactSpinError::Invalid(
                        "snapshot omitted one admitted prototype exact class".to_owned(),
                    )
                })?;
            candidate_rows.push(ArmCandidateRow {
                token: prototype.candidate_token,
                class_result_cid: result.result_cid.clone(),
                relative_state: result.relative.trace(&self.h4_table)?,
                measured_cost: result.cost,
                ranking_cost: (arm != BoundedGlobalExactSpinArm::IdentityDisabled)
                    .then_some(result.cost),
            });
        }
        if reverse_support_iteration {
            candidate_rows.reverse();
        }
        let support_tokens = self
            .prototypes
            .iter()
            .map(|prototype| prototype.candidate_token)
            .collect::<Vec<_>>();
        let measured_costs = candidate_rows
            .iter()
            .map(|row| row.measured_cost)
            .collect::<Vec<_>>();
        let ranked_tokens = candidate_rows
            .iter()
            .map(|row| row.token)
            .collect::<Vec<_>>();
        let selection = select_exact_costs(&measured_costs)?;
        work.cost_comparisons = work
            .cost_comparisons
            .checked_add(selection.comparisons)
            .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?;
        work.final_choice_operations = work
            .final_choice_operations
            .checked_add(1)
            .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?;
        let decision = decision_from_selection(
            arm,
            local.geometric_token,
            selection,
            &ranked_tokens,
            support_tokens,
            work.finish(),
        );
        Ok(ArmEvaluation {
            entries,
            fold_steps,
            global_result: fold,
            classes: class_traces,
            candidate_rows,
            decision,
        })
    }
}

impl BoundedGlobalNoncommutingExactSpinR4V2 {
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        let core = BoundedGlobalExactSpinR4V1::compile(table, base_overlay, construction)?;
        let grammar_kappa = blake3_label(NONCOMMUTING_GRAMMAR_IDENTITY_BYTES);
        let routing_policy_kappa = blake3_label(NONCOMMUTING_ROUTING_POLICY_IDENTITY.as_bytes());
        let population_policy_kappa =
            blake3_label(NONCOMMUTING_POPULATION_POLICY_IDENTITY.as_bytes());
        let population_audit =
            build_noncommuting_population_audit(&core, &population_policy_kappa)?;
        let operator = Self {
            core,
            grammar_kappa,
            routing_policy_kappa,
            population_policy_kappa,
            population_audit,
        };
        operator.validate_binding(table, base_overlay)?;
        let audit = operator.population_audit()?;
        if audit.left_snapshot_hex != NONCOMMUTING_LEFT_SNAPSHOT.map(hex::encode)
            || audit.right_snapshot_hex != NONCOMMUTING_RIGHT_SNAPSHOT.map(hex::encode)
            || !audit.noncommutation.products_distinct
            || !audit.distinct_nonidentity_folds
            || !audit.complete_phase_totals_equal
            || !audit.incompatible_unique_winners
            || !audit.one_transposition
            || !frozen_noncommuting_population_matches(&audit)
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "canonical noncommuting population does not reproduce the frozen V2 pair"
                    .to_owned(),
            ));
        }
        if operator.to_bytes()?.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(operator)
    }

    pub fn from_bytes(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
        bytes: &[u8],
    ) -> Result<Self, BoundedGlobalExactSpinError> {
        if bytes.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator exceeds its byte ceiling".to_owned(),
            ));
        }
        if bytes.len() < NONCOMMUTING_OPERATOR_MAGIC.len()
            || bytes[..NONCOMMUTING_OPERATOR_MAGIC.len()] != NONCOMMUTING_OPERATOR_MAGIC
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator magic is invalid".to_owned(),
            ));
        }
        let expected = Self::compile(table, base_overlay, construction)?;
        if expected.to_bytes()? != bytes {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator is noncanonical or binding-drifted"
                    .to_owned(),
            ));
        }
        Ok(expected)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, BoundedGlobalExactSpinError> {
        let wire = NoncommutingOperatorWire {
            schema: NONCOMMUTING_OPERATOR_SCHEMA,
            domain: NONCOMMUTING_OPERATOR_DOMAIN,
            table_artifact_cid: self.core.table_artifact_cid.clone(),
            base_overlay_artifact_cid: self.core.base_overlay_artifact_cid.clone(),
            construction_ids: self.core.construction_ids.clone(),
            construction_text_cids: self
                .core
                .construction_text_cids
                .iter()
                .map(hex::encode)
                .collect(),
            codec_kappa: self.core.codec_kappa().to_owned(),
            vocabulary_kappa: self.core.vocabulary_kappa().to_owned(),
            route_manifest_kappa: self.core.route_manifest_kappa().to_owned(),
            h4_root_table_kappa: self.core.h4_root_table_kappa().to_owned(),
            h4_multiplication_table_kappa: self.core.h4_multiplication_table_kappa().to_owned(),
            grammar_kappa: self.grammar_kappa.clone(),
            routing_policy_kappa: self.routing_policy_kappa.clone(),
            spin_map_kappa: self.core.spin_map_kappa().to_owned(),
            chart_profile_kappa: self.core.chart_profile_kappa().to_owned(),
            population_policy_kappa: self.population_policy_kappa.clone(),
            construction_identity_scope: CONSTRUCTION_IDENTITY_SCOPE,
            held_out_identity_scope: HELD_OUT_IDENTITY_SCOPE,
            active_turn_id: ACTIVE_TURN_ID,
            active_query_hex: hex::encode(ACTIVE_QUERY_BYTES),
            construction_global_unit_hex: hex::encode(CONSTRUCTION_GLOBAL_UNIT),
            duplicate_pool_hex: NONCOMMUTING_DUPLICATE_POOL.map(hex::encode),
            left_snapshot_hex: NONCOMMUTING_LEFT_SNAPSHOT.map(hex::encode),
            right_snapshot_hex: NONCOMMUTING_RIGHT_SNAPSHOT.map(hex::encode),
            candidates: BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES,
            snapshot_entries: BOUNDED_GLOBAL_EXACT_SPIN_ENTRIES,
            snapshot_classes: BOUNDED_GLOBAL_EXACT_SPIN_CLASSES,
            reuse_hits: BOUNDED_GLOBAL_EXACT_SPIN_REUSE_HITS,
            max_query_bytes: MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES,
            max_operator_bytes: MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES,
            prototypes: self.core.prototype_traces()?,
            population_audit: self.population_audit.clone(),
        };
        let payload = serde_json::to_vec(&wire)
            .map_err(|error| BoundedGlobalExactSpinError::Serialization(error.to_string()))?;
        let mut bytes = Vec::with_capacity(NONCOMMUTING_OPERATOR_MAGIC.len() + payload.len());
        bytes.extend_from_slice(&NONCOMMUTING_OPERATOR_MAGIC);
        bytes.extend_from_slice(&payload);
        if bytes.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_OPERATOR_BYTES {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn artifact_cid(&self) -> Result<String, BoundedGlobalExactSpinError> {
        Ok(blake3_label(&self.to_bytes()?))
    }

    pub fn table_artifact_cid(&self) -> &str {
        self.core.table_artifact_cid()
    }

    pub fn base_overlay_artifact_cid(&self) -> &str {
        self.core.base_overlay_artifact_cid()
    }

    pub fn codec_kappa(&self) -> &str {
        self.core.codec_kappa()
    }

    pub fn vocabulary_kappa(&self) -> &str {
        self.core.vocabulary_kappa()
    }

    pub fn route_manifest_kappa(&self) -> &str {
        self.core.route_manifest_kappa()
    }

    pub fn spin_map_kappa(&self) -> &str {
        self.core.spin_map_kappa()
    }

    pub fn chart_profile_kappa(&self) -> &str {
        self.core.chart_profile_kappa()
    }

    pub fn grammar_kappa(&self) -> &str {
        &self.grammar_kappa
    }

    pub fn routing_policy_kappa(&self) -> &str {
        &self.routing_policy_kappa
    }

    pub fn population_policy_kappa(&self) -> &str {
        &self.population_policy_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        self.core.h4_root_table_kappa()
    }

    pub fn h4_multiplication_table_kappa(&self) -> &str {
        self.core.h4_multiplication_table_kappa()
    }

    pub fn prototype_traces(
        &self,
    ) -> Result<Vec<BoundedGlobalExactSpinPrototypeTrace>, BoundedGlobalExactSpinError> {
        self.core.prototype_traces()
    }

    pub fn population_audit(
        &self,
    ) -> Result<BoundedGlobalNoncommutingPopulationAudit, BoundedGlobalExactSpinError> {
        Ok(self.population_audit.clone())
    }

    pub fn build_query_artifact(
        &self,
        active_query: &[u8],
    ) -> Result<CanonicalRouteArtifact, BoundedGlobalExactSpinError> {
        self.core.build_query_artifact(active_query)
    }

    pub fn build_snapshot_artifact(
        &self,
        active_query: &[u8],
        global_snapshot_units: &[Vec<u8>],
    ) -> Result<CanonicalRouteArtifact, BoundedGlobalExactSpinError> {
        validate_active_query(active_query)?;
        validate_snapshot_units_for(
            global_snapshot_units,
            SnapshotContract::FrozenNoncommutingV2,
        )?;
        let input = observed_global_input_for(
            active_query,
            global_snapshot_units,
            SnapshotContract::FrozenNoncommutingV2,
        )?;
        Ok(CanonicalRouteArtifact::ingest(&self.core.codec, &input)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn predict_pair_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        base_artifact: &CanonicalRouteArtifact,
        left_snapshot_artifact: &CanonicalRouteArtifact,
        right_snapshot_artifact: &CanonicalRouteArtifact,
        active_query: &[u8],
    ) -> Result<MatchedBoundedGlobalNoncommutingPairPrediction, BoundedGlobalExactSpinError> {
        self.validate_binding(table, base_overlay)?;
        let population_audit = self.population_audit()?;
        let operator_cid = self.artifact_cid()?;
        let left = self.core.predict_matched_for(
            table,
            base_overlay,
            base_artifact,
            left_snapshot_artifact,
            active_query,
            SnapshotContract::FrozenNoncommutingV2,
            &operator_cid,
        )?;
        let right = self.core.predict_matched_for(
            table,
            base_overlay,
            base_artifact,
            right_snapshot_artifact,
            active_query,
            SnapshotContract::FrozenNoncommutingV2,
            &operator_cid,
        )?;

        let left_payloads = left
            .snapshot_entries
            .iter()
            .map(|entry| entry.payload_hex.as_str())
            .collect::<Vec<_>>();
        let right_payloads = right
            .snapshot_entries
            .iter()
            .map(|entry| entry.payload_hex.as_str())
            .collect::<Vec<_>>();
        let expected_left = NONCOMMUTING_LEFT_SNAPSHOT
            .iter()
            .map(hex::encode)
            .collect::<Vec<_>>();
        let expected_right = NONCOMMUTING_RIGHT_SNAPSHOT
            .iter()
            .map(hex::encode)
            .collect::<Vec<_>>();
        if left_payloads != expected_left.iter().map(String::as_str).collect::<Vec<_>>()
            || right_payloads
                != expected_right
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "paired global carriers are not in the canonical selected orientation".to_owned(),
            ));
        }

        let left_winner =
            self.candidate_token_for_anchor_hex(&population_audit.left_winner_anchor_hex)?;
        let right_winner =
            self.candidate_token_for_anchor_hex(&population_audit.right_winner_anchor_hex)?;
        let left_permuted = self.other_candidate_token(left_winner)?;
        let right_permuted = self.other_candidate_token(right_winner)?;
        let exact_fold_distinct = left.global_result == population_audit.left_fold
            && right.global_result == population_audit.right_fold
            && left.global_result != right.global_result;
        let real_winners_incompatible = left.real.token == left_winner
            && right.real.token == right_winner
            && left_winner != right_winner;
        let permuted_winners_incompatible = left.class_operator_permuted.token == left_permuted
            && right.class_operator_permuted.token == right_permuted
            && left_permuted != right_permuted;
        let common_lower_artifact =
            left.base_lower_artifact_manifest_kappa == right.base_lower_artifact_manifest_kappa;
        let support_matched_between_cases = left.real.support_tokens == right.real.support_tokens
            && left.identity_disabled.support_tokens == right.identity_disabled.support_tokens
            && left.class_operator_permuted.support_tokens
                == right.class_operator_permuted.support_tokens;
        let work_matched_between_cases = left.real.work == right.real.work
            && left.identity_disabled.work == right.identity_disabled.work
            && left.class_operator_permuted.work == right.class_operator_permuted.work;
        let left_costs_reproduced =
            prediction_reproduces_population_costs(&left, &population_audit.left_candidate_costs);
        let right_costs_reproduced =
            prediction_reproduces_population_costs(&right, &population_audit.right_candidate_costs);
        let hard_gate = population_audit.noncommutation.products_distinct
            && population_audit.distinct_nonidentity_folds
            && population_audit.complete_phase_totals_equal
            && population_audit.incompatible_unique_winners
            && population_audit.one_transposition
            && exact_fold_distinct
            && real_winners_incompatible
            && permuted_winners_incompatible
            && common_lower_artifact
            && support_matched_between_cases
            && work_matched_between_cases
            && left_costs_reproduced
            && right_costs_reproduced
            && left.source_snapshot_artifact_manifest_kappa
                != right.source_snapshot_artifact_manifest_kappa
            && left.global_epoch != right.global_epoch
            && left.global_root_kappa != right.global_root_kappa
            && left.identity_disabled.token == right.identity_disabled.token
            && left.support_matched
            && right.support_matched
            && left.work_matched
            && right.work_matched
            && left.support_reversal_invariant
            && right.support_reversal_invariant
            && left.coherent_relabel_equivariant
            && right.coherent_relabel_equivariant
            && left.forbidden_reads.total() == 0
            && right.forbidden_reads.total() == 0
            && left.operator_abstention.is_none()
            && right.operator_abstention.is_none()
            && left.real.unique_minimum == Some(left.real.token)
            && right.real.unique_minimum == Some(right.real.token)
            && left.class_operator_permuted.unique_minimum
                == Some(left.class_operator_permuted.token)
            && right.class_operator_permuted.unique_minimum
                == Some(right.class_operator_permuted.token)
            && left.identity_disabled.unique_minimum.is_none()
            && right.identity_disabled.unique_minimum.is_none();
        if !hard_gate {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "paired noncommuting exact-spin hard gate stopped before decoding".to_owned(),
            ));
        }

        Ok(MatchedBoundedGlobalNoncommutingPairPrediction {
            population_audit,
            left,
            right,
            exact_fold_distinct,
            real_winners_incompatible,
            permuted_winners_incompatible,
            common_lower_artifact,
            support_matched_between_cases,
            work_matched_between_cases,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn continue_pair_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        base_artifact: &CanonicalRouteArtifact,
        left_snapshot_artifact: &CanonicalRouteArtifact,
        right_snapshot_artifact: &CanonicalRouteArtifact,
        active_query: &[u8],
        max_units: usize,
    ) -> Result<MatchedBoundedGlobalNoncommutingPairContinuation, BoundedGlobalExactSpinError> {
        validate_continuation_bound(max_units)?;
        let first_pair = self.predict_pair_matched(
            table,
            base_overlay,
            base_artifact,
            left_snapshot_artifact,
            right_snapshot_artifact,
            active_query,
        )?;
        let left = continue_from_prediction(
            table,
            base_overlay,
            active_query,
            max_units,
            first_pair.left.clone(),
        )?;
        let right = continue_from_prediction(
            table,
            base_overlay,
            active_query,
            max_units,
            first_pair.right.clone(),
        )?;
        let continuations = [
            &left.real,
            &left.identity_disabled,
            &left.class_operator_permuted,
            &right.real,
            &right.identity_disabled,
            &right.class_operator_permuted,
        ];
        for continuation in continuations {
            let exact_period = continuation.tokens.get(1).is_some_and(|token| {
                table
                    .decode_tokens(&[*token])
                    .is_ok_and(|bytes| bytes == b".")
            });
            if continuation.stop != ContinuationStop::EndOfDocument
                || continuation.tokens.len() != 2
                || continuation.tokens[0] == continuation.tokens[1]
                || !exact_period
            {
                return Err(BoundedGlobalExactSpinError::Invalid(
                    "paired noncommuting exact-spin continuation did not append period and reach EOS"
                        .to_owned(),
                ));
            }
        }
        Ok(MatchedBoundedGlobalNoncommutingPairContinuation {
            first_pair,
            left,
            right,
        })
    }

    fn validate_binding(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), BoundedGlobalExactSpinError> {
        self.core.validate_binding(table, base_overlay)?;
        if self.grammar_kappa != blake3_label(NONCOMMUTING_GRAMMAR_IDENTITY_BYTES)
            || self.routing_policy_kappa
                != blake3_label(NONCOMMUTING_ROUTING_POLICY_IDENTITY.as_bytes())
            || self.population_policy_kappa
                != blake3_label(NONCOMMUTING_POPULATION_POLICY_IDENTITY.as_bytes())
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "bounded-global noncommuting operator binding does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }

    fn candidate_token_for_anchor_hex(
        &self,
        anchor_hex: &str,
    ) -> Result<u32, BoundedGlobalExactSpinError> {
        self.core
            .prototypes
            .iter()
            .find(|prototype| hex::encode(&prototype.anchor_bytes) == anchor_hex)
            .map(|prototype| prototype.candidate_token)
            .ok_or_else(|| {
                BoundedGlobalExactSpinError::Invalid(
                    "population audit winner is not a bound prototype anchor".to_owned(),
                )
            })
    }

    fn other_candidate_token(&self, token: u32) -> Result<u32, BoundedGlobalExactSpinError> {
        let other = self
            .core
            .prototypes
            .iter()
            .filter(|prototype| prototype.candidate_token != token)
            .map(|prototype| prototype.candidate_token)
            .collect::<Vec<_>>();
        if other.len() != 1 {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "noncommuting permutation requires exactly one other candidate".to_owned(),
            ));
        }
        Ok(other[0])
    }
}

fn prediction_reproduces_population_costs(
    prediction: &MatchedBoundedGlobalExactSpinPrediction,
    expected: &[BoundedGlobalNoncommutingCandidateCostTrace],
) -> bool {
    if prediction.candidate_evidence.len() != BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES
        || expected.len() != BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES
    {
        return false;
    }
    prediction.candidate_evidence.iter().all(|evidence| {
        let Some(real) = expected
            .iter()
            .find(|row| row.prototype_anchor_hex == evidence.prototype_anchor_hex)
        else {
            return false;
        };
        let Some(permuted) = expected
            .iter()
            .find(|row| row.prototype_anchor_hex != evidence.prototype_anchor_hex)
        else {
            return false;
        };
        evidence.real_relative_state == real.relative_state
            && evidence.real_measured_cost == real.cost
            && evidence.real_ranking_cost == Some(real.cost)
            && evidence.identity_disabled_measured_cost == real.cost
            && evidence.identity_disabled_ranking_cost.is_none()
            && evidence.permuted_relative_state == permuted.relative_state
            && evidence.permuted_measured_cost == permuted.cost
            && evidence.permuted_ranking_cost == Some(permuted.cost)
    })
}

#[derive(Debug, Clone)]
struct ClassAccumulator {
    class_kappa: String,
    address_kappa: String,
    payload_cid: String,
    state: ExactSpinState,
    reference_entry_kappas: Vec<String>,
}

#[derive(Debug, Clone)]
struct ClassResult {
    class_kappa: String,
    relative: ExactSpinState,
    cost: BoundedGlobalExactSpinCost,
    result_cid: String,
}

#[derive(Debug, Clone, Copy)]
struct WorkCounter {
    local: MultiscaleCountRadiusWork,
    snapshot_entry_reads: u64,
    exact_class_comparisons: u64,
    unique_class_evaluations: u64,
    class_reuse_hits: u64,
    class_result_applications: u64,
    h4_product_table_reads: u64,
    h4_inverse_table_reads: u64,
    phase_additions: u64,
    phase_distance_reads: u64,
    angular_shell_reads: u64,
    candidate_class_lookups: u64,
    cost_comparisons: u64,
    final_choice_operations: u64,
}

impl WorkCounter {
    const fn new(local: MultiscaleCountRadiusWork) -> Self {
        Self {
            local,
            snapshot_entry_reads: 0,
            exact_class_comparisons: 0,
            unique_class_evaluations: 0,
            class_reuse_hits: 0,
            class_result_applications: 0,
            h4_product_table_reads: 0,
            h4_inverse_table_reads: 0,
            phase_additions: 0,
            phase_distance_reads: 0,
            angular_shell_reads: 0,
            candidate_class_lookups: 0,
            cost_comparisons: 0,
            final_choice_operations: 0,
        }
    }

    const fn finish(self) -> BoundedGlobalExactSpinWork {
        BoundedGlobalExactSpinWork {
            local: self.local,
            snapshot_entry_reads: self.snapshot_entry_reads,
            exact_class_comparisons: self.exact_class_comparisons,
            unique_class_evaluations: self.unique_class_evaluations,
            class_reuse_hits: self.class_reuse_hits,
            class_result_applications: self.class_result_applications,
            h4_product_table_reads: self.h4_product_table_reads,
            h4_inverse_table_reads: self.h4_inverse_table_reads,
            phase_additions: self.phase_additions,
            phase_distance_reads: self.phase_distance_reads,
            angular_shell_reads: self.angular_shell_reads,
            candidate_class_lookups: self.candidate_class_lookups,
            cost_comparisons: self.cost_comparisons,
            final_choice_operations: self.final_choice_operations,
        }
    }
}

#[derive(Serialize)]
struct OperatorWire {
    schema: u32,
    domain: &'static str,
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_ids: Vec<String>,
    construction_text_cids: Vec<String>,
    codec_kappa: String,
    vocabulary_kappa: String,
    route_manifest_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    grammar_kappa: String,
    routing_policy_kappa: String,
    spin_map_kappa: String,
    chart_profile_kappa: String,
    construction_identity_scope: &'static str,
    held_out_identity_scope: &'static str,
    active_turn_id: &'static str,
    active_query_hex: String,
    construction_global_unit_hex: String,
    left_snapshot_hex: [String; 4],
    right_snapshot_hex: [String; 4],
    candidates: usize,
    snapshot_entries: usize,
    snapshot_classes: usize,
    reuse_hits: u64,
    max_query_bytes: usize,
    max_operator_bytes: usize,
    prototypes: Vec<BoundedGlobalExactSpinPrototypeTrace>,
}

#[derive(Serialize)]
struct NoncommutingOperatorWire {
    schema: u32,
    domain: &'static str,
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_ids: Vec<String>,
    construction_text_cids: Vec<String>,
    codec_kappa: String,
    vocabulary_kappa: String,
    route_manifest_kappa: String,
    h4_root_table_kappa: String,
    h4_multiplication_table_kappa: String,
    grammar_kappa: String,
    routing_policy_kappa: String,
    spin_map_kappa: String,
    chart_profile_kappa: String,
    population_policy_kappa: String,
    construction_identity_scope: &'static str,
    held_out_identity_scope: &'static str,
    active_turn_id: &'static str,
    active_query_hex: String,
    construction_global_unit_hex: String,
    duplicate_pool_hex: [String; 11],
    left_snapshot_hex: [String; 4],
    right_snapshot_hex: [String; 4],
    candidates: usize,
    snapshot_entries: usize,
    snapshot_classes: usize,
    reuse_hits: u64,
    max_query_bytes: usize,
    max_operator_bytes: usize,
    prototypes: Vec<BoundedGlobalExactSpinPrototypeTrace>,
    population_audit: BoundedGlobalNoncommutingPopulationAudit,
}

#[derive(Serialize)]
struct SpinMapWire {
    schema: u32,
    domain: &'static str,
    exact_mapping_rule: &'static str,
    h4_root_table_kappa: String,
    rows: Vec<SpinMapRow>,
}

#[derive(Serialize)]
struct SpinMapRow {
    s3_q30: [i32; 4],
    h4_coordinate: H4RootCoordinate,
}

#[derive(Serialize)]
struct ClassResultCidWire<'a> {
    schema: u32,
    domain: &'static str,
    global_root_kappa: &'a str,
    global_epoch: &'a str,
    operator_cid: &'a str,
    spin_map_kappa: &'a str,
    chart_profile_kappa: &'a str,
    h4_root_table_kappa: &'a str,
    h4_multiplication_table_kappa: &'a str,
    shared_class_kappa: &'a str,
    relative_result: BoundedGlobalExactSpinStateTrace,
    cost: BoundedGlobalExactSpinCost,
}

#[derive(Serialize)]
struct CandidateStateKappaWire {
    schema: u32,
    domain: &'static str,
    global: BoundedGlobalExactSpinStateTrace,
    prototype: BoundedGlobalExactSpinStateTrace,
    cost: BoundedGlobalExactSpinCost,
}

fn construction_geometry_input(
    construction: &[SourceDocument],
) -> Result<ConversationInput, BoundedGlobalExactSpinError> {
    let mut turns = Vec::with_capacity(construction.len());
    for document in construction {
        let (binding, active) = split_once_exact(&document.text, b"\n\n").ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "construction document lacks the frozen paragraph boundary".to_owned(),
            )
        })?;
        turns.push(TurnInput {
            turn_id: format!("construction-turn-{}", document.id),
            paragraphs: vec![
                ParagraphInput {
                    sentences: vec![binding.to_vec()],
                },
                ParagraphInput {
                    sentences: vec![active.to_vec()],
                },
            ],
        });
    }
    let global_snapshot_units = vec![CONSTRUCTION_GLOBAL_UNIT.to_vec()];
    Ok(ConversationInput {
        identity_scope: CONSTRUCTION_IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns,
    })
}

fn observed_global_input_for(
    active_query: &[u8],
    global_snapshot_units: &[Vec<u8>],
    contract: SnapshotContract,
) -> Result<ConversationInput, BoundedGlobalExactSpinError> {
    validate_active_query(active_query)?;
    validate_snapshot_units_for(global_snapshot_units, contract)?;
    Ok(ConversationInput {
        identity_scope: HELD_OUT_IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(global_snapshot_units)?,
        global_snapshot_units: global_snapshot_units.to_vec(),
        turns: vec![TurnInput {
            turn_id: ACTIVE_TURN_ID.to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: vec![active_query.to_vec()],
            }],
        }],
    })
}

fn observed_base_input(
    active_query: &[u8],
) -> Result<ConversationInput, BoundedGlobalExactSpinError> {
    validate_active_query(active_query)?;
    let global_snapshot_units = vec![CONSTRUCTION_GLOBAL_UNIT.to_vec()];
    Ok(ConversationInput {
        identity_scope: HELD_OUT_IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: ACTIVE_TURN_ID.to_owned(),
            paragraphs: vec![ParagraphInput {
                sentences: vec![active_query.to_vec()],
            }],
        }],
    })
}

fn validate_active_query(active_query: &[u8]) -> Result<(), BoundedGlobalExactSpinError> {
    if active_query != ACTIVE_QUERY_BYTES
        || active_query.len() > MAX_BOUNDED_GLOBAL_EXACT_SPIN_QUERY_BYTES
    {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "active query differs from the frozen bounded-global prompt".to_owned(),
        ));
    }
    Ok(())
}

fn validate_snapshot_units_for(
    units: &[Vec<u8>],
    contract: SnapshotContract,
) -> Result<(), BoundedGlobalExactSpinError> {
    let (left, right) = match contract {
        SnapshotContract::FrozenCommutingV1 => (LEFT_SNAPSHOT, RIGHT_SNAPSHOT),
        SnapshotContract::FrozenNoncommutingV2 => {
            (NONCOMMUTING_LEFT_SNAPSHOT, NONCOMMUTING_RIGHT_SNAPSHOT)
        }
    };
    let left = left.map(<[u8]>::to_vec).to_vec();
    let right = right.map(<[u8]>::to_vec).to_vec();
    if units != left && units != right {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "global snapshot differs from both frozen order controls".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn exact_state_from_address(
    address: &GeometricAddress,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    ExactSpinState::from_parts(
        exact_s3_spin_to_h4(address.spin.s3.raw(), table)?,
        i64::from(address.spin.fiber.raw()),
        i64::from(address.spin.torsion.raw()),
        table,
    )
}

pub(crate) fn exact_state_from_entry(
    entry: &GlobalExactSpinSnapshotEntry,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    ExactSpinState::from_spin_trace(entry.spin, table)
}

fn build_noncommuting_population_audit(
    core: &BoundedGlobalExactSpinR4V1,
    population_policy_kappa: &str,
) -> Result<BoundedGlobalNoncommutingPopulationAudit, BoundedGlobalExactSpinError> {
    if population_policy_kappa != blake3_label(NONCOMMUTING_POPULATION_POLICY_IDENTITY.as_bytes()) {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "noncommuting population policy identity does not reproduce".to_owned(),
        ));
    }

    let helix = prototype_population_lexeme(core, b"helix")?;
    let prism = prototype_population_lexeme(core, b"prism")?;
    let mut pool = Vec::with_capacity(NONCOMMUTING_DUPLICATE_POOL.len());
    for bytes in NONCOMMUTING_DUPLICATE_POOL {
        if PROTOTYPE_BINDINGS
            .iter()
            .any(|(_, anchor)| *anchor == bytes)
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "noncommuting duplicate pool contains a prototype anchor".to_owned(),
            ));
        }
        pool.push(resolve_population_lexeme(core, bytes)?);
    }
    if pool
        .windows(2)
        .any(|pair| pair[0].lexical_unit_id >= pair[1].lexical_unit_id)
    {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "noncommuting duplicate pool is not in canonical lexical-unit order".to_owned(),
        ));
    }

    let mut lexemes = BTreeMap::<u32, PopulationLexeme>::new();
    for lexeme in pool.iter().chain([&helix, &prism]) {
        if lexemes
            .insert(lexeme.lexical_unit_id, lexeme.clone())
            .is_some()
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "noncommuting population lexeme IDs alias".to_owned(),
            ));
        }
    }

    let mut rows_examined = Vec::new();
    let mut selected = None;
    for duplicate in &pool {
        let direct_noncommutation =
            population_has_direct_noncommutation([duplicate, &helix, &prism], &core.h4_table)?;
        let permutations = unique_permutations_4([
            duplicate.lexical_unit_id,
            duplicate.lexical_unit_id,
            helix.lexical_unit_id,
            prism.lexical_unit_id,
        ]);
        let mut pairs_examined = 0_u32;
        let mut selected_pair_indices = None;

        if duplicate.spin != helix.spin
            && duplicate.spin != prism.spin
            && helix.spin != prism.spin
            && direct_noncommutation
        {
            'pairs: for left_index in 0..permutations.len() {
                for right_index in left_index + 1..permutations.len() {
                    pairs_examined = pairs_examined
                        .checked_add(1)
                        .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?;
                    let left_ids = permutations[left_index];
                    let right_ids = permutations[right_index];
                    let Some(transposed_ordinals) = one_transposition(left_ids, right_ids) else {
                        continue;
                    };
                    let left_operand = lexemes
                        .get(&left_ids[usize::from(transposed_ordinals[0])])
                        .ok_or_else(|| {
                            BoundedGlobalExactSpinError::Invalid(
                                "population transposition references an unknown left operand"
                                    .to_owned(),
                            )
                        })?;
                    let right_operand = lexemes
                        .get(&left_ids[usize::from(transposed_ordinals[1])])
                        .ok_or_else(|| {
                            BoundedGlobalExactSpinError::Invalid(
                                "population transposition references an unknown right operand"
                                    .to_owned(),
                            )
                        })?;
                    let left_then_right = left_operand
                        .state
                        .compose(right_operand.state, &core.h4_table)?;
                    let right_then_left = right_operand
                        .state
                        .compose(left_operand.state, &core.h4_table)?;
                    let identity = ExactSpinState::identity(&core.h4_table)?;
                    if left_operand.state.h4 == identity.h4
                        || right_operand.state.h4 == identity.h4
                        || left_then_right.h4 == right_then_left.h4
                    {
                        continue;
                    }
                    let left_fold = fold_population_ids(left_ids, &lexemes, &core.h4_table)?;
                    let right_fold = fold_population_ids(right_ids, &lexemes, &core.h4_table)?;
                    if left_fold == identity || right_fold == identity || left_fold == right_fold {
                        continue;
                    }
                    let left_costs = [
                        population_candidate_cost(core, &core.prototypes, b"helix", left_fold)?,
                        population_candidate_cost(core, &core.prototypes, b"prism", left_fold)?,
                    ];
                    let right_costs = [
                        population_candidate_cost(core, &core.prototypes, b"helix", right_fold)?,
                        population_candidate_cost(core, &core.prototypes, b"prism", right_fold)?,
                    ];
                    let Some(left_winner) =
                        unique_exact_cost_winner(&[left_costs[0].cost, left_costs[1].cost])?
                    else {
                        continue;
                    };
                    let Some(right_winner) =
                        unique_exact_cost_winner(&[right_costs[0].cost, right_costs[1].cost])?
                    else {
                        continue;
                    };
                    if left_winner == right_winner {
                        continue;
                    }
                    selected_pair_indices = Some([
                        u32::try_from(left_index)
                            .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
                        u32::try_from(right_index)
                            .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
                    ]);
                    selected = Some(PopulationSelection {
                        duplicate: duplicate.clone(),
                        left_index,
                        right_index,
                        left_ids,
                        right_ids,
                        left_fold,
                        right_fold,
                        left_costs,
                        right_costs,
                        left_winner,
                        right_winner,
                        left_operand: left_operand.clone(),
                        right_operand: right_operand.clone(),
                        left_then_right,
                        right_then_left,
                    });
                    break 'pairs;
                }
            }
        }

        rows_examined.push(BoundedGlobalNoncommutingPoolRowTrace {
            duplicate_hex: hex::encode(&duplicate.bytes),
            duplicate_lexical_unit_id: duplicate.lexical_unit_id,
            duplicate_address_kappa: duplicate.address_kappa.clone(),
            duplicate_class_kappa: duplicate.class_kappa.clone(),
            duplicate_state: duplicate.state.trace(&core.h4_table)?,
            direct_noncommutation,
            unique_permutations: u32::try_from(permutations.len())
                .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            permutation_pairs_examined: pairs_examined,
            selected_pair_indices,
        });
        if selected.is_some() {
            break;
        }
    }

    let selected = selected.ok_or_else(|| {
        BoundedGlobalExactSpinError::Invalid(
            "canonical construction pool has no noncommuting exact-spin population".to_owned(),
        )
    })?;
    let left_snapshot = population_ids_to_hex(selected.left_ids, &lexemes)?;
    let right_snapshot = population_ids_to_hex(selected.right_ids, &lexemes)?;
    let transposed_ordinals =
        one_transposition(selected.left_ids, selected.right_ids).ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "selected noncommuting pair is not one exact transposition".to_owned(),
            )
        })?;
    let left_winner_anchor_hex = selected.left_costs[selected.left_winner]
        .prototype_anchor_hex
        .clone();
    let right_winner_anchor_hex = selected.right_costs[selected.right_winner]
        .prototype_anchor_hex
        .clone();

    Ok(BoundedGlobalNoncommutingPopulationAudit {
        schema: 1,
        domain: "uor-r4.bounded-global-noncommuting-population-audit/1".to_owned(),
        population_policy_kappa: population_policy_kappa.to_owned(),
        duplicate_pool_hex: NONCOMMUTING_DUPLICATE_POOL.map(hex::encode).to_vec(),
        rows_examined,
        selected_duplicate_hex: hex::encode(&selected.duplicate.bytes),
        selected_duplicate_lexical_unit_id: selected.duplicate.lexical_unit_id,
        selected_duplicate_class_kappa: selected.duplicate.class_kappa,
        selected_pair_indices: [
            u32::try_from(selected.left_index)
                .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
            u32::try_from(selected.right_index)
                .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?,
        ],
        left_snapshot_hex: left_snapshot,
        right_snapshot_hex: right_snapshot,
        one_transposition: true,
        transposed_ordinals,
        noncommutation: BoundedGlobalNoncommutingWitnessTrace {
            left_operand_hex: hex::encode(&selected.left_operand.bytes),
            right_operand_hex: hex::encode(&selected.right_operand.bytes),
            left_operand: selected.left_operand.state.trace(&core.h4_table)?,
            right_operand: selected.right_operand.state.trace(&core.h4_table)?,
            left_then_right: selected.left_then_right.trace(&core.h4_table)?,
            right_then_left: selected.right_then_left.trace(&core.h4_table)?,
            products_distinct: selected.left_then_right.h4 != selected.right_then_left.h4,
        },
        left_fold: selected.left_fold.trace(&core.h4_table)?,
        right_fold: selected.right_fold.trace(&core.h4_table)?,
        distinct_nonidentity_folds: selected.left_fold != selected.right_fold
            && selected.left_fold != ExactSpinState::identity(&core.h4_table)?
            && selected.right_fold != ExactSpinState::identity(&core.h4_table)?,
        complete_phase_totals_equal: selected.left_fold.fiber_q29 == selected.right_fold.fiber_q29
            && selected.left_fold.torsion_q29 == selected.right_fold.torsion_q29,
        left_candidate_costs: selected.left_costs.to_vec(),
        right_candidate_costs: selected.right_costs.to_vec(),
        left_winner_anchor_hex: left_winner_anchor_hex.clone(),
        right_winner_anchor_hex: right_winner_anchor_hex.clone(),
        incompatible_unique_winners: left_winner_anchor_hex != right_winner_anchor_hex,
    })
}

fn frozen_noncommuting_population_matches(
    audit: &BoundedGlobalNoncommutingPopulationAudit,
) -> bool {
    let state =
        |scaled: [i64; 4], fiber_q29: i64, torsion_q29: i64| BoundedGlobalExactSpinStateTrace {
            h4_coordinate: H4RootCoordinate {
                scaled_zphi_quaternion: [
                    [scaled[0], 0],
                    [scaled[1], 0],
                    [scaled[2], 0],
                    [scaled[3], 0],
                ],
            },
            fiber_q29,
            torsion_q29,
        };
    let cost =
        |angular_shell, fiber_distance_q29, torsion_distance_q29| BoundedGlobalExactSpinCost {
            angular_shell,
            fiber_distance_q29,
            torsion_distance_q29,
        };
    let cost_matches = |row: &BoundedGlobalNoncommutingCandidateCostTrace,
                        anchor: &[u8],
                        relative_state: BoundedGlobalExactSpinStateTrace,
                        expected_cost: BoundedGlobalExactSpinCost| {
        row.prototype_anchor_hex == hex::encode(anchor)
            && row.relative_state == relative_state
            && row.cost == expected_cost
    };

    let left_costs_match = audit.left_candidate_costs.len() == 2
        && cost_matches(
            &audit.left_candidate_costs[0],
            b"helix",
            state([-2, 0, 0, 0], 61_239_177, -5_831_083),
            cost(H4S3AngularShell::Antipodal, 61_239_177, 5_831_083),
        )
        && cost_matches(
            &audit.left_candidate_costs[1],
            b"prism",
            state([-1, -1, -1, -1], 55_205_017, -5_262_467),
            cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467),
        );
    let right_costs_match = audit.right_candidate_costs.len() == 2
        && cost_matches(
            &audit.right_candidate_costs[0],
            b"helix",
            state([0, 0, 2, 0], 61_239_177, -5_831_083),
            cost(H4S3AngularShell::Orthogonal, 61_239_177, 5_831_083),
        )
        && cost_matches(
            &audit.right_candidate_costs[1],
            b"prism",
            state([-1, -1, 1, 1], 55_205_017, -5_262_467),
            cost(H4S3AngularShell::Degrees120, 55_205_017, 5_262_467),
        );
    let rows_match = audit.rows_examined.len() == 2
        && audit.rows_examined[0].duplicate_hex == hex::encode(b".")
        && audit.rows_examined[0]
            .duplicate_state
            .h4_coordinate
            .scaled_zphi_quaternion
            == [[2, 0], [0, 0], [0, 0], [0, 0]]
        && !audit.rows_examined[0].direct_noncommutation
        && audit.rows_examined[0].selected_pair_indices.is_none()
        && audit.rows_examined[1].duplicate_hex == hex::encode(b"Lena")
        && audit.rows_examined[1].direct_noncommutation
        && audit.rows_examined[1].selected_pair_indices == Some([0, 2]);

    rows_match
        && audit.selected_duplicate_hex == hex::encode(b"Lena")
        && audit.selected_pair_indices == [0, 2]
        && audit.left_snapshot_hex == NONCOMMUTING_LEFT_SNAPSHOT.map(hex::encode)
        && audit.right_snapshot_hex == NONCOMMUTING_RIGHT_SNAPSHOT.map(hex::encode)
        && audit.one_transposition
        && audit.transposed_ordinals == [1, 2]
        && audit.noncommutation.left_operand_hex == hex::encode(b"Lena")
        && audit.noncommutation.right_operand_hex == hex::encode(b"helix")
        && audit.noncommutation.left_operand == state([0, 2, 0, 0], 7_017_092, -673_944)
        && audit.noncommutation.right_operand == state([1, 1, 1, 1], 41_170_833, -3_914_579)
        && audit.noncommutation.left_then_right == state([-1, 1, -1, 1], 48_187_925, -4_588_523)
        && audit.noncommutation.right_then_left == state([-1, 1, 1, -1], 48_187_925, -4_588_523)
        && audit.noncommutation.products_distinct
        && audit.left_fold == state([-1, -1, -1, -1], 102_410_010, -9_745_662)
        && audit.right_fold == state([-1, -1, 1, 1], 102_410_010, -9_745_662)
        && audit.distinct_nonidentity_folds
        && audit.complete_phase_totals_equal
        && left_costs_match
        && right_costs_match
        && audit.left_winner_anchor_hex == hex::encode(b"prism")
        && audit.right_winner_anchor_hex == hex::encode(b"helix")
        && audit.incompatible_unique_winners
}

fn resolve_population_lexeme(
    core: &BoundedGlobalExactSpinR4V1,
    bytes: &[u8],
) -> Result<PopulationLexeme, BoundedGlobalExactSpinError> {
    let encoded = core.codec.encode(0, 0, bytes)?;
    if encoded.units.len() != 1 || !encoded.trailing_bytes.is_empty() {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "population pool surface is not one exact canonical lexical unit".to_owned(),
        ));
    }
    let lexical_unit_id = encoded.units[0].unit_id;
    let address = core
        .construction_artifact
        .lexical_route_address_from_validated_artifact(lexical_unit_id)?
        .ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "population pool surface has no registered address".to_owned(),
            )
        })?;
    let value = core
        .construction_artifact
        .lexical_route_value_for_address_from_validated_artifact(&address)?
        .ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "population pool address has no payload inversion".to_owned(),
            )
        })?;
    if value.payload_bytes != bytes {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "population pool address payload inversion mismatches".to_owned(),
        ));
    }
    Ok(PopulationLexeme {
        bytes: bytes.to_vec(),
        lexical_unit_id,
        address_kappa: value.address_kappa,
        class_kappa: shared_class_kappa(address.spin)?,
        spin: spin_trace(address.spin),
        state: exact_state_from_address(&address, &core.h4_table)?,
    })
}

fn prototype_population_lexeme(
    core: &BoundedGlobalExactSpinR4V1,
    anchor: &[u8],
) -> Result<PopulationLexeme, BoundedGlobalExactSpinError> {
    let prototype = core
        .prototypes
        .iter()
        .find(|prototype| prototype.anchor_bytes == anchor)
        .ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "noncommuting population omitted a prototype anchor".to_owned(),
            )
        })?;
    Ok(PopulationLexeme {
        bytes: prototype.anchor_bytes.clone(),
        lexical_unit_id: prototype.anchor_lexical_unit_id,
        address_kappa: prototype.anchor_address_kappa.clone(),
        class_kappa: prototype.anchor_class_kappa.clone(),
        spin: spin_trace(prototype.anchor_address.spin),
        state: prototype.anchor_state,
    })
}

fn population_has_direct_noncommutation(
    factors: [&PopulationLexeme; 3],
    table: &H4BinaryIcosahedralClosure,
) -> Result<bool, BoundedGlobalExactSpinError> {
    let identity = ExactSpinState::identity(table)?;
    for left_index in 0..factors.len() {
        for right_index in left_index + 1..factors.len() {
            let left = factors[left_index].state;
            let right = factors[right_index].state;
            if left.h4 == identity.h4 || right.h4 == identity.h4 {
                continue;
            }
            if left.compose(right, table)?.h4 != right.compose(left, table)?.h4 {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn unique_permutations_4(mut values: [u32; 4]) -> Vec<[u32; 4]> {
    values.sort_unstable();
    let mut permutations = vec![values];
    while next_lexicographic_permutation(&mut values) {
        permutations.push(values);
    }
    permutations
}

fn next_lexicographic_permutation(values: &mut [u32]) -> bool {
    let Some(pivot) = (0..values.len().saturating_sub(1))
        .rev()
        .find(|index| values[*index] < values[index + 1])
    else {
        return false;
    };
    let Some(successor) = (pivot + 1..values.len())
        .rev()
        .find(|index| values[*index] > values[pivot])
    else {
        return false;
    };
    values.swap(pivot, successor);
    values[pivot + 1..].reverse();
    true
}

fn one_transposition(left: [u32; 4], right: [u32; 4]) -> Option<[u16; 2]> {
    let changed = (0..left.len())
        .filter(|index| left[*index] != right[*index])
        .collect::<Vec<_>>();
    if changed.len() != 2
        || left[changed[0]] != right[changed[1]]
        || left[changed[1]] != right[changed[0]]
    {
        return None;
    }
    Some([
        u16::try_from(changed[0]).ok()?,
        u16::try_from(changed[1]).ok()?,
    ])
}

fn fold_population_ids(
    ids: [u32; 4],
    lexemes: &BTreeMap<u32, PopulationLexeme>,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    let states = ids
        .into_iter()
        .map(|id| {
            lexemes.get(&id).map(|state| state.state).ok_or_else(|| {
                BoundedGlobalExactSpinError::Invalid(
                    "population permutation references an unknown lexical unit".to_owned(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    fold_exact_spin_states(states, table)
}

fn population_candidate_cost(
    core: &BoundedGlobalExactSpinR4V1,
    prototypes: &[CandidatePrototype],
    anchor: &[u8],
    fold: ExactSpinState,
) -> Result<BoundedGlobalNoncommutingCandidateCostTrace, BoundedGlobalExactSpinError> {
    let prototype = prototypes
        .iter()
        .find(|prototype| prototype.anchor_bytes == anchor)
        .ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "population cost omitted one prototype anchor".to_owned(),
            )
        })?;
    let (relative, cost) =
        candidate_relative_exact_cost(prototype.anchor_state, fold, &core.h4_table)?;
    Ok(BoundedGlobalNoncommutingCandidateCostTrace {
        prototype_anchor_hex: hex::encode(&prototype.anchor_bytes),
        prototype_class_kappa: prototype.anchor_class_kappa.clone(),
        relative_state: relative.trace(&core.h4_table)?,
        cost,
    })
}

fn unique_exact_cost_winner(
    costs: &[BoundedGlobalExactSpinCost; 2],
) -> Result<Option<usize>, BoundedGlobalExactSpinError> {
    Ok(select_unique_minimum_exact_costs(costs)?.unique_minimum_index)
}

fn population_ids_to_hex(
    ids: [u32; 4],
    lexemes: &BTreeMap<u32, PopulationLexeme>,
) -> Result<[String; 4], BoundedGlobalExactSpinError> {
    let values = ids.map(|id| {
        lexemes
            .get(&id)
            .map(|lexeme| hex::encode(&lexeme.bytes))
            .ok_or_else(|| {
                BoundedGlobalExactSpinError::Invalid(
                    "population selection references an unknown lexical unit".to_owned(),
                )
            })
    });
    let [first, second, third, fourth] = values;
    Ok([first?, second?, third?, fourth?])
}

pub(crate) fn exact_s3_spin_to_h4(
    raw: [i32; 4],
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, BoundedGlobalExactSpinError> {
    let mut coordinate = [[0_i64; 2]; 4];
    for (target, value) in coordinate.iter_mut().zip(raw) {
        if value & Q29_H4_SCALE_MASK != 0 {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "stored S3 spin is not an exact scaled H4 coordinate".to_owned(),
            ));
        }
        target[0] = i64::from(value >> Q29_H4_SCALE_SHIFT);
    }
    let expected = H4RootCoordinate {
        scaled_zphi_quaternion: coordinate,
    };
    let mut matched = None;
    let mut matches = 0usize;
    for offset in 0..table.root_count {
        let offset =
            u16::try_from(offset).map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?;
        let index = OpaqueH4TableIndex::from_table_offset(offset, table).ok_or_else(|| {
            BoundedGlobalExactSpinError::Invalid(
                "stored-spin H4 map addressed outside the exact table".to_owned(),
            )
        })?;
        let state = OrderedH4FoldState::from_table_index(index, table)?;
        if state.root_coordinate(table)? == expected {
            matches += 1;
            matched = Some(state);
        }
    }
    if matches != 1 {
        return Err(BoundedGlobalExactSpinError::Invalid(format!(
            "stored S3 spin has {matches} exact H4 coordinate matches"
        )));
    }
    matched.ok_or_else(|| {
        BoundedGlobalExactSpinError::Invalid("stored S3 spin has no exact H4 match".to_owned())
    })
}

fn spin_map_kappa(
    table: &H4BinaryIcosahedralClosure,
) -> Result<String, BoundedGlobalExactSpinError> {
    let half = 1_i32 << 29;
    let one = 1_i32 << 30;
    let raw_rows = [
        [one, 0, 0, 0],
        [0, one, 0, 0],
        [half, half, half, half],
        [half, -half, half, -half],
    ];
    let rows = raw_rows
        .into_iter()
        .map(|raw| {
            let state = exact_s3_spin_to_h4(raw, table)?;
            Ok(SpinMapRow {
                s3_q30: raw,
                h4_coordinate: state.root_coordinate(table)?,
            })
        })
        .collect::<Result<Vec<_>, BoundedGlobalExactSpinError>>()?;
    let bytes = serde_json::to_vec(&SpinMapWire {
        schema: 1,
        domain: SPIN_MAP_DOMAIN,
        exact_mapping_rule: SPIN_MAP_RULE_IDENTITY,
        h4_root_table_kappa: table.h4_root_table_kappa.clone(),
        rows,
    })
    .map_err(|error| BoundedGlobalExactSpinError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
}

pub(crate) fn fold_exact_spin_states(
    states: impl IntoIterator<Item = ExactSpinState>,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    let mut fold = ExactSpinState::identity(table)?;
    for state in states {
        fold = fold.compose(state, table)?;
    }
    Ok(fold)
}

pub(crate) fn candidate_relative_exact_cost(
    class_state: ExactSpinState,
    global_state: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<(ExactSpinState, BoundedGlobalExactSpinCost), BoundedGlobalExactSpinError> {
    let relative = class_state.inverse(table)?.compose(global_state, table)?;
    let cost = exact_cost(relative, table)?;
    Ok((relative, cost))
}

pub(crate) fn exact_cost(
    relative: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<BoundedGlobalExactSpinCost, BoundedGlobalExactSpinError> {
    let real = relative.root_real(table)?;
    let angular_shell = match real {
        [2, 0] => H4S3AngularShell::Coincident,
        [0, 1] => H4S3AngularShell::Degrees36,
        [1, 0] => H4S3AngularShell::Degrees60,
        [-1, 1] => H4S3AngularShell::Degrees72,
        [0, 0] => H4S3AngularShell::Orthogonal,
        [1, -1] => H4S3AngularShell::Degrees108,
        [-1, 0] => H4S3AngularShell::Degrees120,
        [0, -1] => H4S3AngularShell::Degrees144,
        [-2, 0] => H4S3AngularShell::Antipodal,
        other => {
            return Err(BoundedGlobalExactSpinError::Invalid(format!(
                "relative H4 state has noncanonical signed real coordinate {other:?}"
            )))
        }
    };
    Ok(BoundedGlobalExactSpinCost {
        angular_shell,
        fiber_distance_q29: circular_abs_q29(relative.fiber_q29),
        torsion_distance_q29: circular_abs_q29(relative.torsion_q29),
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SelectionOutcome {
    pub(crate) unique_minimum_index: Option<usize>,
    pub(crate) minimum_cost: Option<BoundedGlobalExactSpinCost>,
    pub(crate) comparisons: u64,
}

pub(crate) fn select_unique_minimum_exact_costs(
    costs: &[BoundedGlobalExactSpinCost],
) -> Result<SelectionOutcome, BoundedGlobalExactSpinError> {
    if costs.is_empty() {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "exact-spin cost selection requires nonempty support".to_owned(),
        ));
    }
    let mut minimum: Option<(usize, BoundedGlobalExactSpinCost)> = None;
    let mut tied = false;
    let mut comparisons = 0_u64;
    for (index, cost) in costs.iter().copied().enumerate() {
        comparisons = comparisons
            .checked_add(1)
            .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?;
        match minimum {
            None => {
                minimum = Some((index, cost));
                tied = false;
            }
            Some((_, current)) if cost < current => {
                minimum = Some((index, cost));
                tied = false;
            }
            Some((_, current)) if cost == current => tied = true,
            Some(_) => {}
        }
    }
    Ok(SelectionOutcome {
        unique_minimum_index: (!tied).then_some(minimum.map(|value| value.0)).flatten(),
        minimum_cost: (!tied).then_some(minimum.map(|value| value.1)).flatten(),
        comparisons,
    })
}

fn select_exact_costs(
    costs: &[BoundedGlobalExactSpinCost],
) -> Result<SelectionOutcome, BoundedGlobalExactSpinError> {
    if costs.len() != BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "bounded-global exact-cost count differs from the frozen support".to_owned(),
        ));
    }
    select_unique_minimum_exact_costs(costs)
}

fn decision_from_selection(
    arm: BoundedGlobalExactSpinArm,
    fallback: u32,
    selection: SelectionOutcome,
    ranked_tokens: &[u32],
    support_tokens: Vec<u32>,
    work: BoundedGlobalExactSpinWork,
) -> BoundedGlobalExactSpinDecision {
    if arm == BoundedGlobalExactSpinArm::IdentityDisabled {
        return BoundedGlobalExactSpinDecision {
            arm,
            token: fallback,
            unique_minimum: None,
            minimum_cost: None,
            support_tokens,
            work,
        };
    }
    let unique_minimum = selection
        .unique_minimum_index
        .and_then(|index| ranked_tokens.get(index).copied());
    BoundedGlobalExactSpinDecision {
        arm,
        token: unique_minimum.unwrap_or(fallback),
        unique_minimum,
        minimum_cost: selection.minimum_cost,
        support_tokens,
        work,
    }
}

fn coherent_relabel_equivariant(
    rows: &[ArmCandidateRow],
    original_winner: u32,
    tokens: &[u32],
) -> Result<bool, BoundedGlobalExactSpinError> {
    if rows.len() != 2 || tokens.len() != 2 {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "coherent relabel requires exactly two candidates".to_owned(),
        ));
    }
    let swapped_winner = if original_winner == tokens[0] {
        tokens[1]
    } else if original_winner == tokens[1] {
        tokens[0]
    } else {
        return Ok(false);
    };
    let relabeled = [
        ArmCandidateRow {
            token: tokens[1],
            ..rows[0].clone()
        },
        ArmCandidateRow {
            token: tokens[0],
            ..rows[1].clone()
        },
    ];
    let measured_costs = [relabeled[0].measured_cost, relabeled[1].measured_cost];
    let ranked_tokens = [relabeled[0].token, relabeled[1].token];
    let selection = select_exact_costs(&measured_costs)?;
    let relabeled_decision = decision_from_selection(
        BoundedGlobalExactSpinArm::Real,
        tokens[1],
        selection,
        &ranked_tokens,
        tokens.to_vec(),
        BoundedGlobalExactSpinWork {
            local: MultiscaleCountRadiusWork::default(),
            snapshot_entry_reads: 0,
            exact_class_comparisons: 0,
            unique_class_evaluations: 0,
            class_reuse_hits: 0,
            class_result_applications: 0,
            h4_product_table_reads: 0,
            h4_inverse_table_reads: 0,
            phase_additions: 0,
            phase_distance_reads: 0,
            angular_shell_reads: 0,
            candidate_class_lookups: 0,
            cost_comparisons: 2,
            final_choice_operations: 1,
        },
    );
    Ok(relabeled_decision.token == swapped_winner)
}

struct ClassResultBinding<'a> {
    global_root_kappa: &'a str,
    global_epoch: &'a str,
    operator_cid: &'a str,
    spin_map_kappa: &'a str,
    chart_profile_kappa: &'a str,
    table: &'a H4BinaryIcosahedralClosure,
}

fn class_result_bytes(
    binding: &ClassResultBinding<'_>,
    class_kappa: &str,
    relative: ExactSpinState,
    cost: BoundedGlobalExactSpinCost,
) -> Result<Vec<u8>, BoundedGlobalExactSpinError> {
    serde_json::to_vec(&ClassResultCidWire {
        schema: 1,
        domain: "uor-r4.bounded-global-exact-spin-class-result/1",
        global_root_kappa: binding.global_root_kappa,
        global_epoch: binding.global_epoch,
        operator_cid: binding.operator_cid,
        spin_map_kappa: binding.spin_map_kappa,
        chart_profile_kappa: binding.chart_profile_kappa,
        h4_root_table_kappa: &binding.table.h4_root_table_kappa,
        h4_multiplication_table_kappa: &binding.table.multiplication_table_kappa,
        shared_class_kappa: class_kappa,
        relative_result: relative.trace(binding.table)?,
        cost,
    })
    .map_err(|error| BoundedGlobalExactSpinError::Serialization(error.to_string()))
}

fn candidate_state_kappa(
    global: ExactSpinState,
    prototype: ExactSpinState,
    cost: BoundedGlobalExactSpinCost,
    table: &H4BinaryIcosahedralClosure,
) -> Result<String, BoundedGlobalExactSpinError> {
    let bytes = serde_json::to_vec(&CandidateStateKappaWire {
        schema: 1,
        domain: "uor-r4.bounded-global-candidate-state-class/1",
        global: global.trace(table)?,
        prototype: prototype.trace(table)?,
        cost,
    })
    .map_err(|error| BoundedGlobalExactSpinError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
}

fn diagnostic_sector(spin: SpinTorsionStateTrace) -> BoundedGlobalSpinSectorTrace {
    let hopf = spin.hopf_q30;
    let hopf_octant =
        u8::from(hopf[0] >= 0) | (u8::from(hopf[1] >= 0) << 1) | (u8::from(hopf[2] >= 0) << 2);
    let shifted = i64::from(spin.torsion_q29) + PHASE_HALF_Q29;
    let bin = (shifted * TORSION_BINS) / PHASE_MODULUS_Q29;
    BoundedGlobalSpinSectorTrace {
        hopf_octant,
        torsion_bin: bin.clamp(0, TORSION_BINS - 1) as u8,
    }
}

fn spin_trace(spin: crate::prime_route_attention::SpinTorsionState) -> SpinTorsionStateTrace {
    SpinTorsionStateTrace {
        s3_q30: spin.s3.raw(),
        hopf_q30: spin.hopf.raw(),
        fiber_q29: spin.fiber.raw(),
        torsion_q29: spin.torsion.raw(),
    }
}

pub(crate) fn wrap_phase_q29(mut value: i64) -> i64 {
    while value >= PHASE_HALF_Q29 {
        value -= PHASE_MODULUS_Q29;
    }
    while value < -PHASE_HALF_Q29 {
        value += PHASE_MODULUS_Q29;
    }
    value
}

pub(crate) fn circular_abs_q29(value: i64) -> u64 {
    wrap_phase_q29(value).unsigned_abs()
}

const HIERARCHY_LEVELS: [&str; 7] = [
    "current",
    "previous",
    "last-two",
    "sentence",
    "paragraph",
    "conversation",
    "global",
];

fn hierarchy_identities(view: &AttentionHierarchyView) -> [&str; 7] {
    [
        view.current.as_str(),
        view.previous.as_str(),
        view.last_two.as_str(),
        view.sentence.as_str(),
        view.paragraph.as_str(),
        view.conversation.as_str(),
        view.global.as_str(),
    ]
}

fn hierarchy_level_audit(
    level: &str,
    left_identity: &str,
    right_identity: &str,
    left: &AttentionOrderedFoldLevel,
    right: &AttentionOrderedFoldLevel,
) -> BoundedGlobalExactSpinHierarchyLevelAudit {
    BoundedGlobalExactSpinHierarchyLevelAudit {
        level: level.to_owned(),
        left_identity_kappa: left_identity.to_owned(),
        right_identity_kappa: right_identity.to_owned(),
        identity_equal: left_identity == right_identity,
        left_state: left.state,
        right_state: right.state,
        ordered_state_equal: left.state == right.state
            && left.observed_routes == right.observed_routes
            && left.root_coordinate == right.root_coordinate,
    }
}

fn split_once_exact<'a>(bytes: &'a [u8], needle: &[u8]) -> Option<(&'a [u8], &'a [u8])> {
    let index = bytes
        .windows(needle.len())
        .position(|window| window == needle)?;
    let right = index.checked_add(needle.len())?;
    Some((&bytes[..index], &bytes[right..]))
}

fn contains_bytes(bytes: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && bytes.windows(needle.len()).any(|window| window == needle)
}

fn blake3_label(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn validate_continuation_bound(max_units: usize) -> Result<(), BoundedGlobalExactSpinError> {
    if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
        return Err(BoundedGlobalExactSpinError::Invalid(format!(
            "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
        )));
    }
    Ok(())
}

fn continue_from_prediction(
    table: &SourceFreeTable,
    base_overlay: &MultiscaleCountRadiusR4V1,
    active_query: &[u8],
    max_units: usize,
    first_decision: MatchedBoundedGlobalExactSpinPrediction,
) -> Result<MatchedBoundedGlobalExactSpinContinuation, BoundedGlobalExactSpinError> {
    validate_continuation_bound(max_units)?;
    if first_decision.operator_abstention.is_some()
        || !first_decision.support_matched
        || !first_decision.work_matched
        || !first_decision.support_reversal_invariant
        || !first_decision.coherent_relabel_equivariant
        || first_decision.real.unique_minimum.is_none()
        || first_decision
            .class_operator_permuted
            .unique_minimum
            .is_none()
        || first_decision.identity_disabled.unique_minimum.is_some()
        || first_decision.forbidden_reads.total() != 0
    {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "bounded-global hard gate stopped before decoding".to_owned(),
        ));
    }
    let mut initial_context = vec![BOS_TOKEN];
    initial_context.extend(table.encode_text(active_query)?);
    let mut real = ContinuationState::new(initial_context.clone());
    let mut disabled = ContinuationState::new(initial_context.clone());
    let mut permuted = ContinuationState::new(initial_context);
    real.accept(first_decision.real.token);
    disabled.accept(first_decision.identity_disabled.token);
    permuted.accept(first_decision.class_operator_permuted.token);
    while real.can_step(max_units) || disabled.can_step(max_units) || permuted.can_step(max_units) {
        if real.can_step(max_units) {
            real.accept(
                table
                    .predict_multiscale_count_radius(&real.context, base_overlay)?
                    .geometric_token,
            );
        }
        if disabled.can_step(max_units) {
            disabled.accept(
                table
                    .predict_multiscale_count_radius(&disabled.context, base_overlay)?
                    .geometric_token,
            );
        }
        if permuted.can_step(max_units) {
            permuted.accept(
                table
                    .predict_multiscale_count_radius(&permuted.context, base_overlay)?
                    .geometric_token,
            );
        }
    }
    Ok(MatchedBoundedGlobalExactSpinContinuation {
        first_decision,
        real: real.finish(table)?,
        identity_disabled: disabled.finish(table)?,
        class_operator_permuted: permuted.finish(table)?,
    })
}

#[derive(Debug, Clone)]
struct ContinuationState {
    context: Vec<u32>,
    generated: Vec<u32>,
    stop: ContinuationStop,
}

impl ContinuationState {
    fn new(context: Vec<u32>) -> Self {
        Self {
            context,
            generated: Vec::new(),
            stop: ContinuationStop::Bound,
        }
    }

    fn can_step(&self, max_units: usize) -> bool {
        self.stop == ContinuationStop::Bound && self.generated.len() < max_units
    }

    fn accept(&mut self, token: u32) {
        if token == EOS_TOKEN {
            self.stop = ContinuationStop::EndOfDocument;
            return;
        }
        if self.generated.last() == Some(&token) {
            self.stop = ContinuationStop::PeriodOneCycle;
            return;
        }
        if self.generated.len() >= 3
            && self.generated[self.generated.len() - 2] == token
            && self.generated[self.generated.len() - 3] == self.generated[self.generated.len() - 1]
        {
            self.stop = ContinuationStop::PeriodTwoCycle;
            return;
        }
        self.generated.push(token);
        self.context.push(token);
    }

    fn finish(self, table: &SourceFreeTable) -> Result<Continuation, BoundedGlobalExactSpinError> {
        Ok(Continuation {
            decoded: table.decode_tokens(&self.generated)?,
            tokens: self.generated,
            stop: self.stop,
        })
    }
}
