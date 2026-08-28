//! Frozen bounded-global exact stored-spin contrast for issue #973.
//!
//! This operator owns no candidate admission. It ranks only the exact
//! maximum-count tie exposed by the bound #953 overlay. An independently
//! supplied immutable global snapshot is folded in stored-S3/H4, fiber, and
//! torsion state; one query-specific result is evaluated per exact snapshot
//! class and reused across repeated immutable references.

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
const SPIN_MAP_DOMAIN: &str = "uor-r4.canonical-s3-spin-to-h4/1";
const SPIN_MAP_RULE_IDENTITY: &str = "exact-s3-q30-components-divisible-by-2^29; arithmetic-right-shift-29-to-scaled-zphi-rational-coefficients; phi-coefficients-zero; unique-coordinate-membership-in-canonical-120-root-h4-table; reject-nonmultiple-nonmember-and-alias; no-prime-hash-candidate-or-nearest-root-placement";
const GRAMMAR_IDENTITY_BYTES: &[u8] = b"uor-r4 bounded global exact spin grammar/1\nconstruction=<ENTITY> bound the <ANCHOR> class.\\n\\nThe bounded global code is <CANDIDATE>.\nactive-query=The bounded global code is\nprototype-bindings=bronze->helix,teal->prism\nevaluation-snapshots=Pavel,Pavel,helix,prism|Pavel,Pavel,prism,helix\nevaluation-snapshots-are-not-fitting-inputs=true";
const ROUTING_POLICY_IDENTITY: &str = "uor-r4 bounded global exact spin routing policy/1\nmap=exact-stored-s3-to-canonical-h4-membership; q30-to-h4-shift=29; phi-coefficients=0\nfold=left-to-right exact H4 product with wrapped Q29 fiber/torsion addition\nphase-law=canonical interval [-1686629713,1686629713) with modulus 3373259426\nclass-result=C^-1*G with the same wrapped phase law\ncache-key=global-root,global-epoch,operator,map,chart,root-and-product-inverse-table,exact-class\ncost=lexicographic(h4-s3-angular-shell,fiber-circular-abs-q29,torsion-circular-abs-q29)\nselection=unique-minimum-or-abstain\ncontrols=real,identity-disabled,class-operator-permuted\nidentity-disabled=compute-all-then-return-bound-953-fallback\nclass-operator-permuted=swap-two-prototype-class-results\nscore-firewall=no-token-id,payload,address,prime,digest,ordinal,spin-sector,adjacent-row-or-target-numeric-input";
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
struct ExactSpinState {
    h4: OrderedH4FoldState,
    fiber_q29: i64,
    torsion_q29: i64,
}

impl ExactSpinState {
    fn identity(table: &H4BinaryIcosahedralClosure) -> Result<Self, BoundedGlobalExactSpinError> {
        Ok(Self {
            h4: OrderedH4FoldState::identity(table)?,
            fiber_q29: 0,
            torsion_q29: 0,
        })
    }

    fn compose(
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

    fn inverse(
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

    fn trace(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<BoundedGlobalExactSpinStateTrace, BoundedGlobalExactSpinError> {
        Ok(BoundedGlobalExactSpinStateTrace {
            h4_coordinate: self.h4.root_coordinate(table)?,
            fiber_q29: self.fiber_q29,
            torsion_q29: self.torsion_q29,
        })
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
                .lexical_route_address(unit_id)?
                .ok_or_else(|| {
                    BoundedGlobalExactSpinError::Invalid(
                        "prototype anchor has no registered address".to_owned(),
                    )
                })?;
            let value = construction_artifact
                .lexical_route_value_for_address(&address)?
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
                anchor_address_kappa: address.canonical_kappa().map_err(|error| {
                    BoundedGlobalExactSpinError::Invalid(format!(
                        "prototype anchor address kappa failed: {error}"
                    ))
                })?,
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
        validate_snapshot_units(global_snapshot_units)?;
        let input = observed_global_input(active_query, global_snapshot_units)?;
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
            != observed_global_input(active_query, &snapshot_input.global_snapshot_units)?
            || snapshot_artifact.codec_kappa() != self.codec.codec_kappa()
            || snapshot_artifact.vocabulary_kappa() != self.codec.vocabulary_kappa()
        {
            return Err(BoundedGlobalExactSpinError::Invalid(
                "snapshot artifact is not an exact frozen global input".to_owned(),
            ));
        }
        validate_snapshot_units(&snapshot_input.global_snapshot_units)?;
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

        let real = self.evaluate_arm(&view, &local, BoundedGlobalExactSpinArm::Real, false)?;
        let disabled = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::IdentityDisabled,
            false,
        )?;
        let permuted = self.evaluate_arm(
            &view,
            &local,
            BoundedGlobalExactSpinArm::ClassOperatorPermuted,
            false,
        )?;
        let reversed = self.evaluate_arm(&view, &local, BoundedGlobalExactSpinArm::Real, true)?;

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
            operator_cid: self.artifact_cid()?,
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
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(BoundedGlobalExactSpinError::Invalid(format!(
                "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        let first_decision = self.predict_matched(
            table,
            base_overlay,
            base_artifact,
            snapshot_artifact,
            active_query,
        )?;
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
        while real.can_step(max_units)
            || disabled.can_step(max_units)
            || permuted.can_step(max_units)
        {
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

        let operator_cid = self.artifact_cid()?;
        let result_binding = ClassResultBinding {
            global_root_kappa: &view.global_root_kappa,
            global_epoch: &view.global_epoch,
            operator_cid: &operator_cid,
            spin_map_kappa: &self.spin_map_kappa,
            chart_profile_kappa: &self.chart_profile_kappa,
            table: &self.h4_table,
        };
        let mut class_results = Vec::<ClassResult>::with_capacity(unique.len());
        let mut class_traces = Vec::with_capacity(unique.len());
        for class in unique {
            let relative = class
                .state
                .inverse(&self.h4_table)?
                .compose(fold, &self.h4_table)?;
            work.h4_inverse_table_reads += 1;
            work.h4_product_table_reads += 1;
            work.phase_additions += 2;
            let cost = exact_cost(relative, &self.h4_table)?;
            work.angular_shell_reads += 1;
            work.phase_distance_reads += 2;
            work.unique_class_evaluations += 1;
            work.class_result_applications += u64::try_from(class.reference_entry_kappas.len())
                .map_err(|_| BoundedGlobalExactSpinError::ArithmeticOverflow)?;
            let cold = class
                .state
                .inverse(&self.h4_table)?
                .compose(fold, &self.h4_table)?;
            let cold_cost = exact_cost(cold, &self.h4_table)?;
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
        let selection = select_candidate_rows(&candidate_rows)?;
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

fn observed_global_input(
    active_query: &[u8],
    global_snapshot_units: &[Vec<u8>],
) -> Result<ConversationInput, BoundedGlobalExactSpinError> {
    validate_active_query(active_query)?;
    validate_snapshot_units(global_snapshot_units)?;
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

fn validate_snapshot_units(units: &[Vec<u8>]) -> Result<(), BoundedGlobalExactSpinError> {
    let left = LEFT_SNAPSHOT.map(<[u8]>::to_vec).to_vec();
    let right = RIGHT_SNAPSHOT.map(<[u8]>::to_vec).to_vec();
    if units != left && units != right {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "global snapshot differs from both frozen order controls".to_owned(),
        ));
    }
    Ok(())
}

fn exact_state_from_address(
    address: &GeometricAddress,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    Ok(ExactSpinState {
        h4: exact_s3_spin_to_h4(address.spin.s3.raw(), table)?,
        fiber_q29: i64::from(address.spin.fiber.raw()),
        torsion_q29: i64::from(address.spin.torsion.raw()),
    })
}

fn exact_state_from_entry(
    entry: &GlobalExactSpinSnapshotEntry,
    table: &H4BinaryIcosahedralClosure,
) -> Result<ExactSpinState, BoundedGlobalExactSpinError> {
    Ok(ExactSpinState {
        h4: exact_s3_spin_to_h4(entry.spin.s3_q30, table)?,
        fiber_q29: i64::from(entry.spin.fiber_q29),
        torsion_q29: i64::from(entry.spin.torsion_q29),
    })
}

fn exact_s3_spin_to_h4(
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

fn exact_cost(
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
        fiber_distance_q29: relative.fiber_q29.unsigned_abs(),
        torsion_distance_q29: relative.torsion_q29.unsigned_abs(),
    })
}

impl ExactSpinState {
    fn root_real(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<[i64; 2], BoundedGlobalExactSpinError> {
        Ok(self.h4.root_coordinate(table)?.scaled_zphi_quaternion[0])
    }
}

#[derive(Debug, Clone, Copy)]
struct SelectionOutcome {
    unique_minimum: Option<u32>,
    minimum_cost: Option<BoundedGlobalExactSpinCost>,
    comparisons: u64,
}

fn select_candidate_rows(
    rows: &[ArmCandidateRow],
) -> Result<SelectionOutcome, BoundedGlobalExactSpinError> {
    if rows.len() != BOUNDED_GLOBAL_EXACT_SPIN_CANDIDATES {
        return Err(BoundedGlobalExactSpinError::Invalid(
            "bounded-global candidate row count differs from the frozen support".to_owned(),
        ));
    }
    let mut minimum: Option<(u32, BoundedGlobalExactSpinCost)> = None;
    let mut tied = false;
    let mut comparisons = 0_u64;
    for row in rows {
        comparisons = comparisons
            .checked_add(1)
            .ok_or(BoundedGlobalExactSpinError::ArithmeticOverflow)?;
        let cost = row.measured_cost;
        match minimum {
            None => {
                minimum = Some((row.token, cost));
                tied = false;
            }
            Some((_, current)) if cost < current => {
                minimum = Some((row.token, cost));
                tied = false;
            }
            Some((_, current)) if cost == current => tied = true,
            Some(_) => {}
        }
    }
    Ok(SelectionOutcome {
        unique_minimum: (!tied).then_some(minimum.map(|value| value.0)).flatten(),
        minimum_cost: (!tied).then_some(minimum.map(|value| value.1)).flatten(),
        comparisons,
    })
}

fn decision_from_selection(
    arm: BoundedGlobalExactSpinArm,
    fallback: u32,
    selection: SelectionOutcome,
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
    BoundedGlobalExactSpinDecision {
        arm,
        token: selection.unique_minimum.unwrap_or(fallback),
        unique_minimum: selection.unique_minimum,
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
    let selection = select_candidate_rows(&relabeled)?;
    let relabeled_decision = decision_from_selection(
        BoundedGlobalExactSpinArm::Real,
        tokens[1],
        selection,
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

fn wrap_phase_q29(mut value: i64) -> i64 {
    while value >= PHASE_HALF_Q29 {
        value -= PHASE_MODULUS_Q29;
    }
    while value < -PHASE_HALF_Q29 {
        value += PHASE_MODULUS_Q29;
    }
    value
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
