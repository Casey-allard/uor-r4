//! Frozen conversation-scope stored-spin path probe for issue #973.
//!
//! The operator owns no candidate admission. It ranks only the exact
//! maximum-count tie exposed by the bound #953 overlay. A typed entity binding
//! is resolved across two completed turns, then the construction-recurrent
//! descriptor path is compared with each already-admitted candidate prototype
//! in exact stored S3/H4, fiber, and torsion state.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    canonical_global_epoch, canonical_lexical_piece_bytes, validate_h4_binary_icosahedral_closure,
    AttentionHierarchyView, AttentionOrderedFoldLevel, CanonicalLexicalCodec,
    CanonicalLexicalError, CanonicalRouteArtifact, ConversationInput, H4BinaryIcosahedralClosure,
    H4RootCoordinate, OpaqueH4TableIndex, OrderedH4FoldState, ParagraphInput, TurnInput,
};
use crate::prime_route_attention::GeometricAddress;
use crate::prime_route_geometric_attention::H4S3AngularShell;
use crate::source_free_table::{
    d3_is_held_out, BackoffOrder, Continuation, ContinuationStop, MatchedGeometricPrediction,
    MultiscaleCountRadiusR4V1, MultiscaleCountRadiusWork, SourceDocument, SourceFreeTable,
    SourceFreeTableError, BOS_TOKEN, EOS_TOKEN, MAX_CONTINUATION_UNITS,
};

const OPERATOR_MAGIC: [u8; 8] = *b"CESPIN01";
const OPERATOR_SCHEMA: u32 = 1;
const OPERATOR_DOMAIN: &str = "uor-r4.conversation-entity-spin-path/1";
const SPIN_MAP_DOMAIN: &str = "uor-r4.canonical-s3-spin-to-h4/1";
const SPIN_MAP_RULE_IDENTITY: &str = "exact-s3-q30-components-divisible-by-2^29; arithmetic-right-shift-29-to-scaled-zphi-rational-coefficients; phi-coefficients-zero; unique-coordinate-membership-in-canonical-120-root-h4-table; reject-nonmultiple-nonmember-and-alias; no-prime-hash-candidate-or-nearest-root-placement";
const GRAMMAR_IDENTITY_BYTES: &[u8] = b"uor-r4 conversation entity spin path grammar/1\nheldout-binding=<ENTITY> carried the <DESCRIPTOR> marker. <ENTITY> carried the <DESCRIPTOR> marker.\nheldout-focus=<ENTITY> opened the registry. <ENTITY> waited.\nheldout-active-query=The active registry code is\nheldout-boundaries=structured binding-to-focus,focus-to-active rendered as exactly two blank-line separators in binding,focus,active order\nconstruction-binding=<ENTITY> carried the <DESCRIPTOR> marker.\nconstruction-focus=<ENTITY> opened the registry. <ENTITY> waited.\nconstruction-readout=The active registry code is <CANDIDATE>.\nconstruction-boundaries=exactly two blank-line separators in binding,focus,active order:binding-to-focus,focus-to-active";
const ROUTING_POLICY_IDENTITY: &str = "uor-r4 conversation entity spin path routing policy/1\nphase=wrapped-q29[-1686629713,1686629713)\npath=B(descriptor)*O; B=carried,the,descriptor,marker; O=opened,the,registry\ncost=lexicographic(h4-s3-angular-shell,fiber-circular-abs-q29,torsion-circular-abs-q29)\nselection=unique-minimum-or-abstain\nconversation-disabled=measure-but-do-not-rank\ncontrols=real,conversation-disabled,cross-turn-binding-permuted,binding-rows-reversed\nquery-h4=prevalidated-row-and-coordinate-table-reads\nwork=2-turns,2-binding-facts,2-focus-roles,2-entity-comparisons,2-descriptor-comparisons,7-leaves,9-products,2-inverses,18-phase-additions,4-phase-distances,2-shells,2-cost-comparisons,1-final-choice";
const HIERARCHY_AUDIT_POLICY_IDENTITY: &str = "uor-r4 conversation hierarchy isolation audit/1\nshared-target-free-vocabulary=true\nidentity-scope=issue-973/conversation-entity-spin-path-heldout-v1\nglobal-snapshot=registry\nturn-ids=binding-turn-0001,focus-turn-0002,active-turn-0003\nrequire-equal=current,previous,last-two,sentence,paragraph,global\nrequire-distinct=conversation\nscore-input=false";
const CONSTRUCTION_IDENTITY_SCOPE: &str = "issue-973/conversation-entity-spin-path-construction-v1";
const HELDOUT_IDENTITY_SCOPE: &str = "issue-973/conversation-entity-spin-path-heldout-v1";
const BINDING_TURN_ID: &str = "binding-turn-0001";
const FOCUS_TURN_ID: &str = "focus-turn-0002";
const ACTIVE_TURN_ID: &str = "active-turn-0003";
const GLOBAL_SNAPSHOT_UNIT: &[u8] = b"registry";
const ACTIVE_QUERY_BYTES: &[u8] = b"The active registry code is";
const FROZEN_CONSTRUCTION: [(&str, &[u8]); 2] = [
    (
        "27",
        b"Nora carried the spiral marker.\n\nNora opened the registry. Owen waited.\n\nThe active registry code is silver.",
    ),
    (
        "28",
        b"Owen carried the faceted marker.\n\nOwen opened the registry. Nora waited.\n\nThe active registry code is violet.",
    ),
];
const PHASE_HALF_Q29: i64 = 1_686_629_713;
const PHASE_MODULUS_Q29: i64 = 3_373_259_426;
const Q29_H4_SCALE_SHIFT: u32 = 29;
const Q29_H4_SCALE_MASK: i32 = (1_i32 << Q29_H4_SCALE_SHIFT) - 1;

pub const CONVERSATION_ENTITY_COMPLETED_TURNS: usize = 2;
pub const CONVERSATION_ENTITY_BINDING_FACTS: usize = 2;
pub const CONVERSATION_ENTITY_FOCUS_ROLES: usize = 2;
pub const CONVERSATION_ENTITY_CANDIDATES: usize = 2;
pub const CONVERSATION_ENTITY_PATH_LEAVES: usize = 7;
pub const MAX_CONVERSATION_ENTITY_UNITS: usize = 96;
pub const MAX_CONVERSATION_ENTITY_BYTES: usize = 1536;
pub const MAX_CONVERSATION_ENTITY_OPERATOR_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationEntitySpinPathError {
    Invalid(String),
    SourceFree(String),
    CanonicalLexical(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for ConversationEntitySpinPathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::SourceFree(reason) => write!(formatter, "source-free table: {reason}"),
            Self::CanonicalLexical(reason) => write!(formatter, "canonical lexical: {reason}"),
            Self::Serialization(reason) => write!(formatter, "serialization: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("conversation entity spin-path arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for ConversationEntitySpinPathError {}

impl From<SourceFreeTableError> for ConversationEntitySpinPathError {
    fn from(error: SourceFreeTableError) -> Self {
        Self::SourceFree(error.to_string())
    }
}

impl From<CanonicalLexicalError> for ConversationEntitySpinPathError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::CanonicalLexical(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationEntitySpinPathArm {
    Real,
    ConversationDisabled,
    CrossTurnBindingPermuted,
    BindingRowsReversed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationEntitySpinPathAbstention {
    CostTie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ConversationEntitySpinPathCost {
    pub angular_shell: H4S3AngularShell,
    pub fiber_distance_q29: u64,
    pub torsion_distance_q29: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinPathStateTrace {
    pub h4_coordinate: H4RootCoordinate,
    pub fiber_q29: i64,
    pub torsion_q29: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntityRouteValueWitness {
    pub lexical_unit_id: u32,
    pub registry_address_index: u16,
    pub prime: u32,
    pub address_kappa: String,
    pub radial_zphi: [i64; 2],
    pub payload_cid: String,
    pub payload_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinLeafTrace {
    pub surface_hex: String,
    pub lexical_unit_id: u32,
    pub registry_address_index: u16,
    pub prime: u32,
    pub address_kappa: String,
    pub radial_zphi: [i64; 2],
    pub payload_cid: String,
    pub s3_q30: [i32; 4],
    pub hopf_q30: [i32; 3],
    pub fiber_q29: i32,
    pub torsion_q29: i32,
    pub mapped_h4_coordinate: H4RootCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinPrototypeTrace {
    pub candidate_token: u32,
    pub candidate_hex: String,
    pub candidate_value: ConversationEntityRouteValueWitness,
    pub candidate_geometry_used_for_ranking: bool,
    pub descriptor_hex: String,
    pub binding_leaves: Vec<ConversationEntitySpinLeafTrace>,
    pub focus_leaves: Vec<ConversationEntitySpinLeafTrace>,
    pub path_state: ConversationEntitySpinPathStateTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinRelationTrace {
    pub query_state: ConversationEntitySpinPathStateTrace,
    pub relative_state: ConversationEntitySpinPathStateTrace,
    pub measured_cost: ConversationEntitySpinPathCost,
    pub ranking_cost: Option<ConversationEntitySpinPathCost>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinCandidateEvidence {
    pub token: u32,
    pub count: u64,
    pub candidate_hex: String,
    pub prototype_descriptor_hex: String,
    pub prototype_state: ConversationEntitySpinPathStateTrace,
    pub real: ConversationEntitySpinRelationTrace,
    pub conversation_disabled: ConversationEntitySpinRelationTrace,
    pub cross_turn_binding_permuted: ConversationEntitySpinRelationTrace,
    pub binding_rows_reversed: ConversationEntitySpinRelationTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinPathWork {
    pub local: MultiscaleCountRadiusWork,
    pub completed_turn_slots_scanned: u64,
    pub binding_fact_slots_scanned: u64,
    pub focus_role_slots_scanned: u64,
    pub entity_key_comparisons: u64,
    pub descriptor_row_comparisons: u64,
    pub stored_spin_leaf_reads: u64,
    pub h4_product_table_reads: u64,
    pub h4_inverse_table_reads: u64,
    pub phase_additions: u64,
    pub phase_distance_reads: u64,
    pub angular_shell_reads: u64,
    pub cost_comparisons: u64,
    pub final_choice_operations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntitySpinPathDecision {
    pub arm: ConversationEntitySpinPathArm,
    pub token: u32,
    pub unique_minimum: Option<u32>,
    pub minimum_cost: Option<ConversationEntitySpinPathCost>,
    pub support_tokens: Vec<u32>,
    pub work: ConversationEntitySpinPathWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedConversationEntitySpinPathPrediction {
    pub local: MatchedGeometricPrediction,
    pub opener_entity_hex: String,
    pub waiter_entity_hex: String,
    pub real_descriptor_hex: String,
    pub permuted_descriptor_hex: String,
    pub prior_candidate_occurrences: u32,
    pub prototypes: Vec<ConversationEntitySpinPrototypeTrace>,
    pub candidate_evidence: Vec<ConversationEntitySpinCandidateEvidence>,
    pub real: ConversationEntitySpinPathDecision,
    pub conversation_disabled: ConversationEntitySpinPathDecision,
    pub cross_turn_binding_permuted: ConversationEntitySpinPathDecision,
    pub binding_rows_reversed: ConversationEntitySpinPathDecision,
    pub operator_abstention: Option<ConversationEntitySpinPathAbstention>,
    pub support_matched: bool,
    pub work_matched: bool,
    pub teacher_calls: u64,
    pub provider_calls: u64,
    pub source_weight_reads: u64,
    pub future_unit_reads: u64,
    pub target_reads: u64,
    pub partition_id_reads: u64,
    pub full_history_key_reads: u64,
    pub global_operator_reads: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedConversationEntitySpinPathContinuation {
    pub first_decision: MatchedConversationEntitySpinPathPrediction,
    pub real: Continuation,
    pub conversation_disabled: Continuation,
    pub cross_turn_binding_permuted: Continuation,
    pub binding_rows_reversed: Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntityHierarchyLevelAudit {
    pub level: String,
    pub left_identity_kappa: String,
    pub right_identity_kappa: String,
    pub identity_equal: bool,
    pub left_observed_routes: u32,
    pub right_observed_routes: u32,
    pub left_state: OrderedH4FoldState,
    pub right_state: OrderedH4FoldState,
    pub left_root_coordinate: H4RootCoordinate,
    pub right_root_coordinate: H4RootCoordinate,
    pub ordered_state_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationEntityHierarchyAudit {
    pub schema: u32,
    pub domain: String,
    pub policy_kappa: String,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub left_route_manifest_kappa: String,
    pub right_route_manifest_kappa: String,
    pub global_epoch: String,
    pub left_global_snapshot_kappa: String,
    pub right_global_snapshot_kappa: String,
    pub lexical_multiset_equal: bool,
    pub lower_scope_identities_equal: bool,
    pub lower_scope_ordered_states_equal: bool,
    pub conversation_identity_distinct: bool,
    pub global_identity_equal: bool,
    pub global_ordered_state_equal: bool,
    pub levels: Vec<ConversationEntityHierarchyLevelAudit>,
    pub score_input_used: bool,
    pub global_operator_reads: u64,
    pub target_reads: u64,
    pub partition_id_reads: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpinPathState {
    h4: OrderedH4FoldState,
    fiber_q29: i64,
    torsion_q29: i64,
}

impl SpinPathState {
    fn identity(
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, ConversationEntitySpinPathError> {
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
    ) -> Result<Self, ConversationEntitySpinPathError> {
        Ok(Self {
            h4: self.h4.compose(right.h4, table)?,
            fiber_q29: wrap_phase_q29(
                self.fiber_q29
                    .checked_add(right.fiber_q29)
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                self.torsion_q29
                    .checked_add(right.torsion_q29)
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledSpinLeaf {
    trace: ConversationEntitySpinLeafTrace,
    state: SpinPathState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledRouteValue {
    address: GeometricAddress,
    witness: ConversationEntityRouteValueWitness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidatePrototype {
    candidate_token: u32,
    candidate_bytes: Vec<u8>,
    descriptor: Vec<u8>,
    candidate_value: ConversationEntityRouteValueWitness,
    binding_leaves: Vec<CompiledSpinLeaf>,
    focus_leaves: Vec<CompiledSpinLeaf>,
    leaves: Vec<CompiledSpinLeaf>,
    state: SpinPathState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedConstructionRow {
    entity: Vec<u8>,
    descriptor: Vec<u8>,
    candidate: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityBinding {
    entity: Vec<u8>,
    descriptor: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FocusRoles {
    opener: Vec<u8>,
    waiter: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArmEvaluation {
    query_state: SpinPathState,
    relations: Vec<(SpinPathState, ConversationEntitySpinPathCost)>,
    winner: Option<u32>,
    minimum_cost: Option<ConversationEntitySpinPathCost>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationEntitySpinPathR4V1 {
    table_artifact_cid: String,
    base_overlay_artifact_cid: String,
    construction_ids: Vec<String>,
    construction_text_cids: Vec<[u8; 32]>,
    codec: CanonicalLexicalCodec,
    route_artifact: CanonicalRouteArtifact,
    h4_table: H4BinaryIcosahedralClosure,
    h4_states: Vec<OrderedH4FoldState>,
    h4_product_rows: Vec<Vec<u16>>,
    h4_root_coordinates: Vec<H4RootCoordinate>,
    grammar_kappa: String,
    routing_policy_kappa: String,
    hierarchy_audit_policy_kappa: String,
    spin_map_kappa: String,
    prototypes: Vec<CandidatePrototype>,
}

impl ConversationEntitySpinPathR4V1 {
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, ConversationEntitySpinPathError> {
        if base_overlay.table_artifact_cid() != table.artifact_cid() {
            return Err(ConversationEntitySpinPathError::Invalid(
                "#953 overlay table binding mismatches".to_owned(),
            ));
        }
        if construction.len() != CONVERSATION_ENTITY_CANDIDATES {
            return Err(ConversationEntitySpinPathError::Invalid(format!(
                "conversation spin-path construction requires exactly {CONVERSATION_ENTITY_CANDIDATES} documents"
            )));
        }
        if construction
            .iter()
            .any(|document| d3_is_held_out(&document.id))
            || !table.is_bound_to_construction_documents(construction)
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "operator construction is not the exact D3-construction set bound to the table"
                    .to_owned(),
            ));
        }
        let mut sorted = construction.to_vec();
        sorted.sort_by(|left, right| left.id.cmp(&right.id));
        if sorted.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(ConversationEntitySpinPathError::Invalid(
                "construction document IDs are not unique".to_owned(),
            ));
        }
        for (document, (expected_id, expected_text)) in sorted.iter().zip(FROZEN_CONSTRUCTION) {
            if document.id != expected_id || document.text.as_slice() != expected_text {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "construction differs from the exact frozen #973 conversation documents"
                        .to_owned(),
                ));
            }
        }
        let rebuilt_table = SourceFreeTable::compile(&sorted)?;
        if rebuilt_table.artifact_cid() != table.artifact_cid() {
            return Err(ConversationEntitySpinPathError::Invalid(
                "bound source-free table does not reproduce from frozen construction".to_owned(),
            ));
        }
        let parsed = sorted
            .iter()
            .map(parse_construction_document)
            .collect::<Result<Vec<_>, _>>()?;
        if parsed
            .iter()
            .map(|row| row.entity.as_slice())
            .collect::<BTreeSet<_>>()
            .len()
            != CONVERSATION_ENTITY_CANDIDATES
            || parsed
                .iter()
                .map(|row| row.descriptor.as_slice())
                .collect::<BTreeSet<_>>()
                .len()
                != CONVERSATION_ENTITY_CANDIDATES
            || parsed
                .iter()
                .map(|row| row.candidate.as_slice())
                .collect::<BTreeSet<_>>()
                .len()
                != CONVERSATION_ENTITY_CANDIDATES
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "construction entities, descriptors, and candidates must each be unique".to_owned(),
            ));
        }

        let geometry_input = construction_geometry_input(&sorted)?;
        let codec = CanonicalLexicalCodec::compile(&geometry_input)?;
        let route_artifact = CanonicalRouteArtifact::ingest(&codec, &geometry_input)?;
        let h4_table = validate_h4_binary_icosahedral_closure()?;
        let h4_states = (0..h4_table.root_count)
            .map(|offset| {
                let offset = u16::try_from(offset)
                    .map_err(|_| ConversationEntitySpinPathError::ArithmeticOverflow)?;
                let index =
                    OpaqueH4TableIndex::from_table_offset(offset, &h4_table).ok_or_else(|| {
                        ConversationEntitySpinPathError::Invalid(
                            "H4 query cache index is outside the exact table".to_owned(),
                        )
                    })?;
                Ok(OrderedH4FoldState::from_table_index(index, &h4_table)?)
            })
            .collect::<Result<Vec<_>, ConversationEntitySpinPathError>>()?;
        let h4_product_rows = h4_table
            .multiplication_indices
            .chunks_exact(h4_table.root_count)
            .map(<[u16]>::to_vec)
            .collect::<Vec<_>>();
        let h4_root_coordinates = h4_states
            .iter()
            .map(|state| Ok(state.root_coordinate(&h4_table)?))
            .collect::<Result<Vec<_>, ConversationEntitySpinPathError>>()?;
        let grammar_kappa = blake3_label(GRAMMAR_IDENTITY_BYTES);
        let routing_policy_kappa = routing_policy_kappa()?;
        let hierarchy_audit_policy_kappa = blake3_label(HIERARCHY_AUDIT_POLICY_IDENTITY.as_bytes());
        let spin_map_kappa = spin_map_kappa(&h4_table)?;

        let focus_leaves = compile_focus_leaves(&codec, &route_artifact, &h4_table)?;
        let mut prototypes = Vec::with_capacity(parsed.len());
        for row in parsed {
            let mut candidate_with_boundary = Vec::with_capacity(row.candidate.len() + 1);
            candidate_with_boundary.push(b' ');
            candidate_with_boundary.extend_from_slice(&row.candidate);
            let candidate_tokens = table.encode_text(&candidate_with_boundary)?;
            if candidate_tokens.len() != 1
                || !table.is_fitted_lexical_token(candidate_tokens[0])
                || table.decode_tokens(&candidate_tokens)? != candidate_with_boundary
            {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "construction candidate is not one exact fitted lexical token".to_owned(),
                ));
            }
            let binding_leaves =
                compile_binding_leaves(&codec, &route_artifact, &h4_table, &row.descriptor)?;
            let mut leaves = binding_leaves.clone();
            leaves.extend(focus_leaves.clone());
            let state = fold_leaves(&leaves, &h4_table)?;
            let candidate_value =
                compile_route_value_witness(&codec, &route_artifact, &row.candidate)?;
            prototypes.push(CandidatePrototype {
                candidate_token: candidate_tokens[0],
                candidate_bytes: candidate_with_boundary,
                descriptor: row.descriptor,
                candidate_value,
                binding_leaves,
                focus_leaves: focus_leaves.clone(),
                leaves,
                state,
            });
        }
        prototypes.sort_by_key(|prototype| prototype.candidate_token);
        let token_or_state_alias = prototypes.windows(2).any(|pair| {
            pair[0].candidate_token >= pair[1].candidate_token || pair[0].state == pair[1].state
        });
        if prototypes.len() != CONVERSATION_ENTITY_CANDIDATES || token_or_state_alias {
            return Err(ConversationEntitySpinPathError::Invalid(
                "candidate prototype tokens or complete conversation SpinTorsion paths alias"
                    .to_owned(),
            ));
        }

        let operator = Self {
            table_artifact_cid: table.artifact_cid(),
            base_overlay_artifact_cid: base_overlay.artifact_cid(),
            construction_ids: sorted.iter().map(|document| document.id.clone()).collect(),
            construction_text_cids: sorted.iter().map(SourceDocument::text_cid).collect(),
            codec,
            route_artifact,
            h4_table,
            h4_states,
            h4_product_rows,
            h4_root_coordinates,
            grammar_kappa,
            routing_policy_kappa,
            hierarchy_audit_policy_kappa,
            spin_map_kappa,
            prototypes,
        };
        operator.validate_internal()?;
        Ok(operator)
    }

    pub fn from_bytes(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
        bytes: &[u8],
    ) -> Result<Self, ConversationEntitySpinPathError> {
        if bytes.len() < OPERATOR_MAGIC.len()
            || bytes.len() > MAX_CONVERSATION_ENTITY_OPERATOR_BYTES
            || bytes[..OPERATOR_MAGIC.len()] != OPERATOR_MAGIC
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path operator magic/size is invalid".to_owned(),
            ));
        }
        let expected = Self::compile(table, base_overlay, construction)?;
        if expected.to_bytes()? != bytes {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path operator is noncanonical or binding-drifted".to_owned(),
            ));
        }
        Ok(expected)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ConversationEntitySpinPathError> {
        self.validate_internal()?;
        let wire = OperatorWire {
            schema: OPERATOR_SCHEMA,
            domain: OPERATOR_DOMAIN,
            table_artifact_cid: self.table_artifact_cid(),
            base_overlay_artifact_cid: self.base_overlay_artifact_cid(),
            construction_ids: self.construction_ids.clone(),
            construction_text_cids: self
                .construction_text_cids
                .iter()
                .map(hex::encode)
                .collect(),
            codec_kappa: self.route_artifact.codec_kappa().to_owned(),
            vocabulary_kappa: self.route_artifact.vocabulary_kappa().to_owned(),
            route_manifest_kappa: self.route_artifact.manifest_kappa().to_owned(),
            h4_root_table_kappa: self.h4_table.h4_root_table_kappa.clone(),
            h4_multiplication_table_kappa: self.h4_table.multiplication_table_kappa.clone(),
            grammar_kappa: self.grammar_kappa.clone(),
            routing_policy_kappa: self.routing_policy_kappa.clone(),
            routing_policy: routing_policy_wire(),
            hierarchy_audit_policy_kappa: self.hierarchy_audit_policy_kappa.clone(),
            hierarchy_audit_policy: HIERARCHY_AUDIT_POLICY_IDENTITY,
            construction_identity_scope: CONSTRUCTION_IDENTITY_SCOPE,
            held_out_identity_scope: HELDOUT_IDENTITY_SCOPE,
            binding_turn_id: BINDING_TURN_ID,
            focus_turn_id: FOCUS_TURN_ID,
            active_turn_id: ACTIVE_TURN_ID,
            global_snapshot_unit_hex: hex::encode(GLOBAL_SNAPSHOT_UNIT),
            global_epoch: frozen_global_epoch()?,
            spin_map_kappa: self.spin_map_kappa.clone(),
            completed_turns: CONVERSATION_ENTITY_COMPLETED_TURNS,
            binding_facts: CONVERSATION_ENTITY_BINDING_FACTS,
            focus_roles: CONVERSATION_ENTITY_FOCUS_ROLES,
            candidates: CONVERSATION_ENTITY_CANDIDATES,
            path_leaves: CONVERSATION_ENTITY_PATH_LEAVES,
            max_units: MAX_CONVERSATION_ENTITY_UNITS,
            max_bytes: MAX_CONVERSATION_ENTITY_BYTES,
            max_operator_bytes: MAX_CONVERSATION_ENTITY_OPERATOR_BYTES,
            prototypes: self.prototype_traces()?,
        };
        let payload = serde_json::to_vec(&wire)
            .map_err(|error| ConversationEntitySpinPathError::Serialization(error.to_string()))?;
        let mut bytes = Vec::with_capacity(OPERATOR_MAGIC.len() + payload.len());
        bytes.extend_from_slice(&OPERATOR_MAGIC);
        bytes.extend_from_slice(&payload);
        if bytes.len() > MAX_CONVERSATION_ENTITY_OPERATOR_BYTES {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn artifact_cid(&self) -> Result<String, ConversationEntitySpinPathError> {
        Ok(blake3_label(&self.to_bytes()?))
    }

    pub fn table_artifact_cid(&self) -> String {
        self.table_artifact_cid.clone()
    }

    pub fn base_overlay_artifact_cid(&self) -> String {
        self.base_overlay_artifact_cid.clone()
    }

    pub fn codec_kappa(&self) -> &str {
        self.route_artifact.codec_kappa()
    }

    pub fn vocabulary_kappa(&self) -> &str {
        self.route_artifact.vocabulary_kappa()
    }

    pub fn route_manifest_kappa(&self) -> &str {
        self.route_artifact.manifest_kappa()
    }

    pub fn spin_map_kappa(&self) -> &str {
        &self.spin_map_kappa
    }

    pub fn grammar_kappa(&self) -> &str {
        &self.grammar_kappa
    }

    pub fn routing_policy_kappa(&self) -> &str {
        &self.routing_policy_kappa
    }

    pub fn hierarchy_audit_policy_kappa(&self) -> &str {
        &self.hierarchy_audit_policy_kappa
    }

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_table.h4_root_table_kappa
    }

    pub fn h4_multiplication_table_kappa(&self) -> &str {
        &self.h4_table.multiplication_table_kappa
    }

    pub fn global_epoch(&self) -> Result<String, ConversationEntitySpinPathError> {
        frozen_global_epoch()
    }

    pub fn prototype_traces(
        &self,
    ) -> Result<Vec<ConversationEntitySpinPrototypeTrace>, ConversationEntitySpinPathError> {
        self.prototypes
            .iter()
            .map(|prototype| {
                Ok(ConversationEntitySpinPrototypeTrace {
                    candidate_token: prototype.candidate_token,
                    candidate_hex: hex::encode(&prototype.candidate_bytes),
                    candidate_value: prototype.candidate_value.clone(),
                    candidate_geometry_used_for_ranking: false,
                    descriptor_hex: hex::encode(&prototype.descriptor),
                    binding_leaves: prototype
                        .binding_leaves
                        .iter()
                        .map(|leaf| leaf.trace.clone())
                        .collect(),
                    focus_leaves: prototype
                        .focus_leaves
                        .iter()
                        .map(|leaf| leaf.trace.clone())
                        .collect(),
                    path_state: self.query_trace(prototype.state)?,
                })
            })
            .collect()
    }

    fn query_compose(
        &self,
        left: SpinPathState,
        right: SpinPathState,
    ) -> Result<SpinPathState, ConversationEntitySpinPathError> {
        let left_offset = usize::from(left.h4.table_index().table_offset());
        let right_offset = usize::from(right.h4.table_index().table_offset());
        let product_offset = self
            .h4_product_rows
            .get(left_offset)
            .and_then(|row| row.get(right_offset))
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 product addressed outside the cached exact table".to_owned(),
                )
            })?;
        let h4 = self
            .h4_states
            .get(usize::from(product_offset))
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 product returned an invalid cached state".to_owned(),
                )
            })?;
        Ok(SpinPathState {
            h4,
            fiber_q29: wrap_phase_q29(
                left.fiber_q29
                    .checked_add(right.fiber_q29)
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                left.torsion_q29
                    .checked_add(right.torsion_q29)
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }

    fn query_inverse(
        &self,
        state: SpinPathState,
    ) -> Result<SpinPathState, ConversationEntitySpinPathError> {
        let offset = usize::from(state.h4.table_index().table_offset());
        let inverse_offset = self
            .h4_table
            .inverse_indices
            .get(offset)
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 inverse addressed outside the cached exact table".to_owned(),
                )
            })?;
        let h4 = self
            .h4_states
            .get(usize::from(inverse_offset))
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 inverse returned an invalid cached state".to_owned(),
                )
            })?;
        Ok(SpinPathState {
            h4,
            fiber_q29: wrap_phase_q29(
                state
                    .fiber_q29
                    .checked_neg()
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                state
                    .torsion_q29
                    .checked_neg()
                    .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }

    fn query_trace(
        &self,
        state: SpinPathState,
    ) -> Result<ConversationEntitySpinPathStateTrace, ConversationEntitySpinPathError> {
        let offset = usize::from(state.h4.table_index().table_offset());
        let h4_coordinate = self
            .h4_root_coordinates
            .get(offset)
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 trace addressed outside the cached exact table".to_owned(),
                )
            })?;
        Ok(ConversationEntitySpinPathStateTrace {
            h4_coordinate,
            fiber_q29: state.fiber_q29,
            torsion_q29: state.torsion_q29,
        })
    }

    fn query_shell(
        &self,
        state: SpinPathState,
    ) -> Result<H4S3AngularShell, ConversationEntitySpinPathError> {
        let real = self
            .query_trace(state)?
            .h4_coordinate
            .scaled_zphi_quaternion[0];
        match real {
            [2, 0] => Ok(H4S3AngularShell::Coincident),
            [0, 1] => Ok(H4S3AngularShell::Degrees36),
            [1, 0] => Ok(H4S3AngularShell::Degrees60),
            [-1, 1] => Ok(H4S3AngularShell::Degrees72),
            [0, 0] => Ok(H4S3AngularShell::Orthogonal),
            [1, -1] => Ok(H4S3AngularShell::Degrees108),
            [-1, 0] => Ok(H4S3AngularShell::Degrees120),
            [0, -1] => Ok(H4S3AngularShell::Degrees144),
            [-2, 0] => Ok(H4S3AngularShell::Antipodal),
            other => Err(ConversationEntitySpinPathError::Invalid(format!(
                "cached H4 relative state has noncanonical signed S3 real coordinate {other:?}"
            ))),
        }
    }

    fn query_fold_leaves(
        &self,
        leaves: &[CompiledSpinLeaf],
    ) -> Result<SpinPathState, ConversationEntitySpinPathError> {
        if leaves.len() != CONVERSATION_ENTITY_PATH_LEAVES {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation descriptor path has the wrong leaf count".to_owned(),
            ));
        }
        let identity = self
            .h4_states
            .get(usize::from(self.h4_table.identity_index))
            .copied()
            .ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "query H4 identity is absent from the cached exact table".to_owned(),
                )
            })?;
        let mut state = SpinPathState {
            h4: identity,
            fiber_q29: 0,
            torsion_q29: 0,
        };
        for leaf in leaves {
            state = self.query_compose(state, leaf.state)?;
        }
        Ok(state)
    }

    pub fn predict_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        binding_turn: &[u8],
        focus_turn: &[u8],
        active_query: &[u8],
    ) -> Result<MatchedConversationEntitySpinPathPrediction, ConversationEntitySpinPathError> {
        self.ensure_bound(table, base_overlay)?;
        let prompt = render_prompt(binding_turn, focus_turn, active_query)?;
        if prompt.len() > MAX_CONVERSATION_ENTITY_BYTES {
            return Err(ConversationEntitySpinPathError::Invalid(format!(
                "combined conversation exceeds the {MAX_CONVERSATION_ENTITY_BYTES}-byte bound"
            )));
        }
        let facts = parse_binding_turn(binding_turn)?;
        let focus = parse_focus_turn(focus_turn)?;
        parse_active_query(active_query)?;
        let real_descriptor = resolve_descriptor(&facts, &focus.opener)?;
        let disabled_descriptor = resolve_descriptor(&facts, &focus.opener)?;
        if disabled_descriptor != real_descriptor {
            return Err(ConversationEntitySpinPathError::Invalid(
                "disabled arm descriptor resolution differs from the real arm".to_owned(),
            ));
        }
        let permuted_descriptor = resolve_descriptor(&facts, &focus.waiter)?;
        if real_descriptor == permuted_descriptor {
            return Err(ConversationEntitySpinPathError::Invalid(
                "focus roles resolve the same descriptor".to_owned(),
            ));
        }
        let mut reversed_facts = facts.clone();
        reversed_facts.reverse();
        let reversed_descriptor = resolve_descriptor(&reversed_facts, &focus.opener)?;
        if reversed_descriptor != real_descriptor {
            return Err(ConversationEntitySpinPathError::Invalid(
                "binding-row reversal changed the typed cross-turn binding".to_owned(),
            ));
        }

        let mut context = vec![BOS_TOKEN];
        context.extend(table.encode_text(&prompt)?);
        if context.len().saturating_sub(1) > MAX_CONVERSATION_ENTITY_UNITS {
            return Err(ConversationEntitySpinPathError::Invalid(format!(
                "conversation prompt exceeds the {MAX_CONVERSATION_ENTITY_UNITS}-unit bound"
            )));
        }
        let local = table.predict_multiscale_count_radius(&context, base_overlay)?;
        self.validate_local_support(table, &context, &local)?;

        let mut prior_candidate_occurrences = 0_u32;
        for prototype in &self.prototypes {
            if contains_bytes(&prompt, &prototype.candidate_bytes) {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "observed conversation contains an admitted candidate payload".to_owned(),
                ));
            }
        }
        for token in context.iter().copied() {
            for prototype in &self.prototypes {
                if token == prototype.candidate_token {
                    prior_candidate_occurrences = prior_candidate_occurrences
                        .checked_add(1)
                        .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?;
                }
            }
        }
        if prior_candidate_occurrences != 0 {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation prompt contains an admitted candidate token".to_owned(),
            ));
        }

        let real_eval = self.evaluate_descriptor(&real_descriptor)?;
        let disabled_eval = self.evaluate_descriptor(&disabled_descriptor)?;
        let permuted_eval = self.evaluate_descriptor(&permuted_descriptor)?;
        let reversed_eval = self.evaluate_descriptor(&reversed_descriptor)?;
        let work = query_work_schedule().with_local(local.geometric_work);
        let support = local.max_count_tie_tokens.clone();
        let fallback = local.geometric_token;
        let real = decision(
            ConversationEntitySpinPathArm::Real,
            real_eval.winner.unwrap_or(fallback),
            real_eval.winner,
            real_eval.minimum_cost,
            support.clone(),
            work,
        );
        let conversation_disabled = decision(
            ConversationEntitySpinPathArm::ConversationDisabled,
            fallback,
            None,
            None,
            support.clone(),
            work,
        );
        let cross_turn_binding_permuted = decision(
            ConversationEntitySpinPathArm::CrossTurnBindingPermuted,
            permuted_eval.winner.unwrap_or(fallback),
            permuted_eval.winner,
            permuted_eval.minimum_cost,
            support.clone(),
            work,
        );
        let binding_rows_reversed = decision(
            ConversationEntitySpinPathArm::BindingRowsReversed,
            reversed_eval.winner.unwrap_or(fallback),
            reversed_eval.winner,
            reversed_eval.minimum_cost,
            support.clone(),
            work,
        );
        let mut candidate_evidence = Vec::with_capacity(self.prototypes.len());
        for (index, prototype) in self.prototypes.iter().enumerate() {
            let local_candidate = local.tie_evidence.get(index).ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "#953 tie evidence omitted a bound prototype candidate".to_owned(),
                )
            })?;
            if local_candidate.token != prototype.candidate_token {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "#953 tie evidence order differs from bound prototype order".to_owned(),
                ));
            }
            candidate_evidence.push(ConversationEntitySpinCandidateEvidence {
                token: prototype.candidate_token,
                count: local_candidate.count,
                candidate_hex: hex::encode(&prototype.candidate_bytes),
                prototype_descriptor_hex: hex::encode(&prototype.descriptor),
                prototype_state: self.query_trace(prototype.state)?,
                real: self.relation_trace(&real_eval, index, true)?,
                conversation_disabled: self.relation_trace(&disabled_eval, index, false)?,
                cross_turn_binding_permuted: self.relation_trace(&permuted_eval, index, true)?,
                binding_rows_reversed: self.relation_trace(&reversed_eval, index, true)?,
            });
        }
        let support_matched = real.support_tokens == conversation_disabled.support_tokens
            && real.support_tokens == cross_turn_binding_permuted.support_tokens
            && real.support_tokens == binding_rows_reversed.support_tokens;
        let work_matched = real.work == conversation_disabled.work
            && real.work == cross_turn_binding_permuted.work
            && real.work == binding_rows_reversed.work
            && local.baseline_work == local.geometric_work;
        let operator_abstention = (real_eval.winner.is_none()
            || permuted_eval.winner.is_none()
            || reversed_eval.winner.is_none())
        .then_some(ConversationEntitySpinPathAbstention::CostTie);

        Ok(MatchedConversationEntitySpinPathPrediction {
            local,
            opener_entity_hex: hex::encode(focus.opener),
            waiter_entity_hex: hex::encode(focus.waiter),
            real_descriptor_hex: hex::encode(real_descriptor),
            permuted_descriptor_hex: hex::encode(permuted_descriptor),
            prior_candidate_occurrences,
            prototypes: self.prototype_traces()?,
            candidate_evidence,
            real,
            conversation_disabled,
            cross_turn_binding_permuted,
            binding_rows_reversed,
            operator_abstention,
            support_matched,
            work_matched,
            teacher_calls: 0,
            provider_calls: 0,
            source_weight_reads: 0,
            future_unit_reads: 0,
            target_reads: 0,
            partition_id_reads: 0,
            full_history_key_reads: 0,
            global_operator_reads: 0,
        })
    }

    pub fn continue_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        binding_turn: &[u8],
        focus_turn: &[u8],
        active_query: &[u8],
        max_units: usize,
    ) -> Result<MatchedConversationEntitySpinPathContinuation, ConversationEntitySpinPathError>
    {
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(ConversationEntitySpinPathError::Invalid(format!(
                "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        let first_decision =
            self.predict_matched(table, base_overlay, binding_turn, focus_turn, active_query)?;
        if first_decision.operator_abstention.is_some()
            || !first_decision.support_matched
            || !first_decision.work_matched
            || first_decision.real.unique_minimum.is_none()
            || first_decision
                .cross_turn_binding_permuted
                .unique_minimum
                .is_none()
            || first_decision
                .binding_rows_reversed
                .unique_minimum
                .is_none()
            || first_decision
                .conversation_disabled
                .unique_minimum
                .is_some()
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path hard gate stopped before decoding".to_owned(),
            ));
        }
        let prompt = render_prompt(binding_turn, focus_turn, active_query)?;
        let mut initial_context = vec![BOS_TOKEN];
        initial_context.extend(table.encode_text(&prompt)?);
        let mut real = ContinuationState::new(initial_context.clone());
        let mut disabled = ContinuationState::new(initial_context.clone());
        let mut permuted = ContinuationState::new(initial_context.clone());
        let mut reversed = ContinuationState::new(initial_context);
        real.accept(first_decision.real.token);
        disabled.accept(first_decision.conversation_disabled.token);
        permuted.accept(first_decision.cross_turn_binding_permuted.token);
        reversed.accept(first_decision.binding_rows_reversed.token);
        while real.can_step(max_units)
            || disabled.can_step(max_units)
            || permuted.can_step(max_units)
            || reversed.can_step(max_units)
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
            if reversed.can_step(max_units) {
                reversed.accept(
                    table
                        .predict_multiscale_count_radius(&reversed.context, base_overlay)?
                        .geometric_token,
                );
            }
        }
        Ok(MatchedConversationEntitySpinPathContinuation {
            first_decision,
            real: real.finish(table)?,
            conversation_disabled: disabled.finish(table)?,
            cross_turn_binding_permuted: permuted.finish(table)?,
            binding_rows_reversed: reversed.finish(table)?,
        })
    }

    pub fn audit_hierarchy(
        &self,
        left_binding_turn: &[u8],
        right_binding_turn: &[u8],
        focus_turn: &[u8],
        active_query: &[u8],
    ) -> Result<ConversationEntityHierarchyAudit, ConversationEntitySpinPathError> {
        self.audit_hierarchy_pair(
            left_binding_turn,
            right_binding_turn,
            focus_turn,
            active_query,
        )
    }

    pub fn audit_hierarchy_pair(
        &self,
        left_binding_turn: &[u8],
        right_binding_turn: &[u8],
        focus_turn: &[u8],
        active_query: &[u8],
    ) -> Result<ConversationEntityHierarchyAudit, ConversationEntitySpinPathError> {
        parse_binding_turn(left_binding_turn)?;
        parse_binding_turn(right_binding_turn)?;
        parse_focus_turn(focus_turn)?;
        parse_active_query(active_query)?;
        let left_prompt = render_prompt(left_binding_turn, focus_turn, active_query)?;
        let right_prompt = render_prompt(right_binding_turn, focus_turn, active_query)?;
        if left_prompt.len() > MAX_CONVERSATION_ENTITY_BYTES
            || right_prompt.len() > MAX_CONVERSATION_ENTITY_BYTES
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "hierarchy-audit conversation exceeds the frozen byte bound".to_owned(),
            ));
        }
        for prototype in &self.prototypes {
            if contains_bytes(&left_prompt, &prototype.candidate_bytes)
                || contains_bytes(&right_prompt, &prototype.candidate_bytes)
            {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "hierarchy-audit input contains an admitted candidate".to_owned(),
                ));
            }
        }
        let mut left_multiset = canonical_lexical_piece_bytes(&left_prompt)?;
        let mut right_multiset = canonical_lexical_piece_bytes(&right_prompt)?;
        left_multiset.sort();
        right_multiset.sort();
        let lexical_multiset_equal = left_multiset == right_multiset;
        if !lexical_multiset_equal {
            return Err(ConversationEntitySpinPathError::Invalid(
                "held-out hierarchy inputs do not share one lexical multiset".to_owned(),
            ));
        }

        let left_input = observed_conversation_input(left_binding_turn, focus_turn, active_query)?;
        let right_input =
            observed_conversation_input(right_binding_turn, focus_turn, active_query)?;
        let left_codec = CanonicalLexicalCodec::compile(&left_input)?;
        let right_codec = CanonicalLexicalCodec::compile(&right_input)?;
        if left_codec.codec_kappa() != right_codec.codec_kappa()
            || left_codec.vocabulary_kappa() != right_codec.vocabulary_kappa()
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "independently compiled held-out hierarchy codecs differ".to_owned(),
            ));
        }
        let left_artifact = CanonicalRouteArtifact::ingest(&left_codec, &left_input)?;
        let right_artifact = CanonicalRouteArtifact::ingest(&left_codec, &right_input)?;
        let left_view = left_artifact.attention_hierarchy_view();
        let right_view = right_artifact.attention_hierarchy_view();
        let left_ordered =
            left_artifact.attention_consumer_trace_with_ordered_h4(&self.h4_table)?;
        let right_ordered =
            right_artifact.attention_consumer_trace_with_ordered_h4(&self.h4_table)?;
        let left_consumer = left_artifact.attention_consumer_trace()?;
        let right_consumer = right_artifact.attention_consumer_trace()?;
        if left_ordered.ordered_levels.len() != 7 || right_ordered.ordered_levels.len() != 7 {
            return Err(ConversationEntitySpinPathError::Invalid(
                "hierarchy audit did not expose the exact seven levels".to_owned(),
            ));
        }
        let left_identities = hierarchy_identities(&left_view);
        let right_identities = hierarchy_identities(&right_view);
        let mut levels = Vec::with_capacity(7);
        for index in 0..7 {
            let left_level = left_ordered.ordered_levels.get(index).ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "left hierarchy ordered level is absent".to_owned(),
                )
            })?;
            let right_level = right_ordered.ordered_levels.get(index).ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "right hierarchy ordered level is absent".to_owned(),
                )
            })?;
            let expected_level = HIERARCHY_LEVELS[index];
            if left_level.level != expected_level || right_level.level != expected_level {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "hierarchy audit level order differs from the fixed consumer order".to_owned(),
                ));
            }
            levels.push(hierarchy_level_audit(
                expected_level,
                left_identities[index],
                right_identities[index],
                left_level,
                right_level,
            ));
        }
        let lower_scope_identities_equal = levels[..5].iter().all(|level| level.identity_equal);
        let lower_scope_ordered_states_equal =
            levels[..5].iter().all(|level| level.ordered_state_equal);
        let conversation_identity_distinct = !levels[5].identity_equal;
        let global_identity_equal = levels[6].identity_equal;
        let global_ordered_state_equal = levels[6].ordered_state_equal;
        let global_epoch = frozen_global_epoch()?;
        if left_input.global_epoch != global_epoch
            || right_input.global_epoch != global_epoch
            || left_consumer.global_snapshot_kappa != right_consumer.global_snapshot_kappa
            || !lower_scope_identities_equal
            || !lower_scope_ordered_states_equal
            || !conversation_identity_distinct
            || !global_identity_equal
            || !global_ordered_state_equal
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "INVALID_CONVERSATION_SCOPE_CONTRACT: hierarchy isolation did not reproduce"
                    .to_owned(),
            ));
        }
        Ok(ConversationEntityHierarchyAudit {
            schema: 1,
            domain: "uor-r4.conversation-entity-spin-path-hierarchy-audit/1".to_owned(),
            policy_kappa: self.hierarchy_audit_policy_kappa.clone(),
            codec_kappa: left_codec.codec_kappa().to_owned(),
            vocabulary_kappa: left_codec.vocabulary_kappa().to_owned(),
            left_route_manifest_kappa: left_artifact.manifest_kappa().to_owned(),
            right_route_manifest_kappa: right_artifact.manifest_kappa().to_owned(),
            global_epoch,
            left_global_snapshot_kappa: left_consumer.global_snapshot_kappa,
            right_global_snapshot_kappa: right_consumer.global_snapshot_kappa,
            lexical_multiset_equal,
            lower_scope_identities_equal,
            lower_scope_ordered_states_equal,
            conversation_identity_distinct,
            global_identity_equal,
            global_ordered_state_equal,
            levels,
            score_input_used: false,
            global_operator_reads: 0,
            target_reads: 0,
            partition_id_reads: 0,
        })
    }

    fn validate_internal(&self) -> Result<(), ConversationEntitySpinPathError> {
        let construction_ids_match = self
            .construction_ids
            .iter()
            .map(String::as_str)
            .eq(FROZEN_CONSTRUCTION.iter().map(|(id, _)| *id));
        let construction_cids_match = self
            .construction_text_cids
            .iter()
            .zip(FROZEN_CONSTRUCTION)
            .all(|(actual, (id, text))| {
                *actual == SourceDocument::new(id, text.to_vec()).text_cid()
            });
        let h4_rows_match = self.h4_product_rows.len() == self.h4_table.root_count
            && self
                .h4_product_rows
                .iter()
                .all(|row| row.len() == self.h4_table.root_count)
            && self.h4_product_rows.iter().flatten().copied().eq(self
                .h4_table
                .multiplication_indices
                .iter()
                .copied());
        let h4_states_match = self.h4_states.len() == self.h4_table.root_count
            && self.h4_root_coordinates.len() == self.h4_table.root_count
            && self
                .h4_states
                .iter()
                .zip(&self.h4_root_coordinates)
                .enumerate()
                .all(|(offset, (state, coordinate))| {
                    usize::from(state.table_index().table_offset()) == offset
                        && state.root_coordinate(&self.h4_table).ok().as_ref() == Some(coordinate)
                });
        if self.prototypes.len() != CONVERSATION_ENTITY_CANDIDATES
            || self.construction_ids.len() != CONVERSATION_ENTITY_CANDIDATES
            || self.construction_text_cids.len() != CONVERSATION_ENTITY_CANDIDATES
            || !construction_ids_match
            || !construction_cids_match
            || !h4_rows_match
            || !h4_states_match
            || self.grammar_kappa != blake3_label(GRAMMAR_IDENTITY_BYTES)
            || self.routing_policy_kappa != routing_policy_kappa()?
            || self.hierarchy_audit_policy_kappa
                != blake3_label(HIERARCHY_AUDIT_POLICY_IDENTITY.as_bytes())
            || self.spin_map_kappa != spin_map_kappa(&self.h4_table)?
            || self.route_artifact.codec_kappa() != self.codec.codec_kappa()
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path operator binding does not reproduce".to_owned(),
            ));
        }
        for prototype in &self.prototypes {
            let candidate_payload =
                prototype
                    .candidate_bytes
                    .strip_prefix(b" ")
                    .ok_or_else(|| {
                        ConversationEntitySpinPathError::Invalid(
                            "candidate token lacks its frozen lexical boundary".to_owned(),
                        )
                    })?;
            if prototype.binding_leaves.len() != 4
                || prototype.focus_leaves.len() != 3
                || prototype.leaves.len() != CONVERSATION_ENTITY_PATH_LEAVES
                || fold_leaves(&prototype.leaves, &self.h4_table)? != prototype.state
                || prototype.candidate_value.payload_hex != hex::encode(candidate_payload)
            {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "conversation spin-path prototype does not reproduce".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn ensure_bound(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), ConversationEntitySpinPathError> {
        if self.table_artifact_cid != table.artifact_cid()
            || self.base_overlay_artifact_cid != base_overlay.artifact_cid()
            || base_overlay.table_artifact_cid() != table.artifact_cid()
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "conversation spin-path table/overlay binding mismatches".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_local_support(
        &self,
        table: &SourceFreeTable,
        context: &[u32],
        local: &MatchedGeometricPrediction,
    ) -> Result<(), ConversationEntitySpinPathError> {
        if local.order != BackoffOrder::Trigram
            || !local.geometry_reachable
            || local.max_count != 1
            || local.max_count_tie_tokens.len() != CONVERSATION_ENTITY_CANDIDATES
            || local.tie_evidence.len() != CONVERSATION_ENTITY_CANDIDATES
            || local.baseline_support_tokens != local.geometric_support_tokens
            || local.baseline_support_tokens != local.max_count_tie_tokens
            || local.baseline_work != local.geometric_work
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "#953 local tie/support/work contract is unavailable".to_owned(),
            ));
        }
        let expected_tokens = self
            .prototypes
            .iter()
            .map(|prototype| prototype.candidate_token)
            .collect::<Vec<_>>();
        let (first_candidate, second_candidate) = match local.tie_evidence.as_slice() {
            [first, second] => (first, second),
            _ => {
                return Err(ConversationEntitySpinPathError::Invalid(
                    "#953 tie evidence is not the exact two-candidate support".to_owned(),
                ));
            }
        };
        let fallback = expected_tokens.first().copied().ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "bound conversation prototype support is empty".to_owned(),
            )
        })?;
        if local.max_count_tie_tokens != expected_tokens
            || local
                .tie_evidence
                .iter()
                .map(|candidate| candidate.token)
                .ne(expected_tokens.iter().copied())
            || local
                .tie_evidence
                .iter()
                .any(|candidate| candidate.count != 1)
            || first_candidate.coordinates != second_candidate.coordinates
            || first_candidate.radius != second_candidate.radius
            || local.geometric_token != fallback
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "#953 admitted candidates, coordinates, radii, or fallback drifted".to_owned(),
            ));
        }
        if context.len() < 2
            || table.decode_tokens(&context[context.len() - 2..context.len() - 1])? != b" code"
            || table.decode_tokens(&context[context.len() - 1..])? != b" is"
        {
            return Err(ConversationEntitySpinPathError::Invalid(
                "active query is not the frozen code/is trigram frame".to_owned(),
            ));
        }
        Ok(())
    }

    fn evaluate_descriptor(
        &self,
        descriptor: &[u8],
    ) -> Result<ArmEvaluation, ConversationEntitySpinPathError> {
        let mut query_leaves = None;
        for prototype in &self.prototypes {
            if prototype.descriptor == descriptor {
                if query_leaves.is_some() {
                    return Err(ConversationEntitySpinPathError::Invalid(
                        "query descriptor aliases multiple compiled SpinTorsion paths".to_owned(),
                    ));
                }
                query_leaves = Some(prototype.leaves.as_slice());
            }
        }
        let query_leaves = query_leaves.ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "query descriptor has no compiled conversation SpinTorsion path".to_owned(),
            )
        })?;
        let query_state = self.query_fold_leaves(query_leaves)?;
        let mut relations = Vec::with_capacity(self.prototypes.len());
        for prototype in &self.prototypes {
            let relative = self.query_compose(self.query_inverse(prototype.state)?, query_state)?;
            let cost = ConversationEntitySpinPathCost {
                angular_shell: self.query_shell(relative)?,
                fiber_distance_q29: circular_abs_q29(relative.fiber_q29),
                torsion_distance_q29: circular_abs_q29(relative.torsion_q29),
            };
            relations.push((relative, cost));
        }
        let mut minimum = ConversationEntitySpinPathCost {
            angular_shell: H4S3AngularShell::Antipodal,
            fiber_distance_q29: u64::MAX,
            torsion_distance_q29: u64::MAX,
        };
        let mut minimum_count = 0_u8;
        let mut winner_index = 0_usize;
        for (index, (_, cost)) in relations.iter().enumerate() {
            match cost.cmp(&minimum) {
                std::cmp::Ordering::Less => {
                    minimum = *cost;
                    minimum_count = 1;
                    winner_index = index;
                }
                std::cmp::Ordering::Equal => {
                    minimum_count = minimum_count
                        .checked_add(1)
                        .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?;
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        let minimum_cost = (minimum_count != 0).then_some(minimum);
        let winner = if minimum_count == 1 {
            Some(
                self.prototypes
                    .get(winner_index)
                    .ok_or_else(|| {
                        ConversationEntitySpinPathError::Invalid(
                            "unique minimum index is outside fixed support".to_owned(),
                        )
                    })?
                    .candidate_token,
            )
        } else {
            None
        };
        Ok(ArmEvaluation {
            query_state,
            relations,
            winner,
            minimum_cost,
        })
    }

    fn relation_trace(
        &self,
        evaluation: &ArmEvaluation,
        candidate_index: usize,
        ranking_enabled: bool,
    ) -> Result<ConversationEntitySpinRelationTrace, ConversationEntitySpinPathError> {
        let (relative, measured_cost) =
            evaluation.relations.get(candidate_index).ok_or_else(|| {
                ConversationEntitySpinPathError::Invalid(
                    "candidate relation index is outside fixed support".to_owned(),
                )
            })?;
        Ok(ConversationEntitySpinRelationTrace {
            query_state: self.query_trace(evaluation.query_state)?,
            relative_state: self.query_trace(*relative)?,
            measured_cost: *measured_cost,
            ranking_cost: ranking_enabled.then_some(*measured_cost),
        })
    }
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
    routing_policy: RoutingPolicyWire,
    hierarchy_audit_policy_kappa: String,
    hierarchy_audit_policy: &'static str,
    construction_identity_scope: &'static str,
    held_out_identity_scope: &'static str,
    binding_turn_id: &'static str,
    focus_turn_id: &'static str,
    active_turn_id: &'static str,
    global_snapshot_unit_hex: String,
    global_epoch: String,
    spin_map_kappa: String,
    completed_turns: usize,
    binding_facts: usize,
    focus_roles: usize,
    candidates: usize,
    path_leaves: usize,
    max_units: usize,
    max_bytes: usize,
    max_operator_bytes: usize,
    prototypes: Vec<ConversationEntitySpinPrototypeTrace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct RoutingPolicyWire {
    identity: &'static str,
    phase_lower_inclusive_q29: i64,
    phase_upper_exclusive_q29: i64,
    phase_modulus_q29: i64,
    composition_order: [&'static str; 2],
    cost_order: [&'static str; 3],
    unique_minimum_required: bool,
    conversation_disabled_ranking_enabled: bool,
    binding_order_control: &'static str,
    query_h4_access: &'static str,
    work: QueryWorkSchedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct QueryWorkSchedule {
    completed_turn_slots_scanned: u64,
    binding_fact_slots_scanned: u64,
    focus_role_slots_scanned: u64,
    entity_key_comparisons: u64,
    descriptor_row_comparisons: u64,
    stored_spin_leaf_reads: u64,
    h4_product_table_reads: u64,
    h4_inverse_table_reads: u64,
    phase_additions: u64,
    phase_distance_reads: u64,
    angular_shell_reads: u64,
    cost_comparisons: u64,
    final_choice_operations: u64,
}

impl QueryWorkSchedule {
    fn with_local(self, local: MultiscaleCountRadiusWork) -> ConversationEntitySpinPathWork {
        ConversationEntitySpinPathWork {
            local,
            completed_turn_slots_scanned: self.completed_turn_slots_scanned,
            binding_fact_slots_scanned: self.binding_fact_slots_scanned,
            focus_role_slots_scanned: self.focus_role_slots_scanned,
            entity_key_comparisons: self.entity_key_comparisons,
            descriptor_row_comparisons: self.descriptor_row_comparisons,
            stored_spin_leaf_reads: self.stored_spin_leaf_reads,
            h4_product_table_reads: self.h4_product_table_reads,
            h4_inverse_table_reads: self.h4_inverse_table_reads,
            phase_additions: self.phase_additions,
            phase_distance_reads: self.phase_distance_reads,
            angular_shell_reads: self.angular_shell_reads,
            cost_comparisons: self.cost_comparisons,
            final_choice_operations: self.final_choice_operations,
        }
    }
}

const fn query_work_schedule() -> QueryWorkSchedule {
    QueryWorkSchedule {
        completed_turn_slots_scanned: 2,
        binding_fact_slots_scanned: 2,
        focus_role_slots_scanned: 2,
        entity_key_comparisons: 2,
        descriptor_row_comparisons: 2,
        stored_spin_leaf_reads: 7,
        h4_product_table_reads: 9,
        h4_inverse_table_reads: 2,
        phase_additions: 18,
        phase_distance_reads: 4,
        angular_shell_reads: 2,
        cost_comparisons: 2,
        final_choice_operations: 1,
    }
}

const fn routing_policy_wire() -> RoutingPolicyWire {
    RoutingPolicyWire {
        identity: ROUTING_POLICY_IDENTITY,
        phase_lower_inclusive_q29: -PHASE_HALF_Q29,
        phase_upper_exclusive_q29: PHASE_HALF_Q29,
        phase_modulus_q29: PHASE_MODULUS_Q29,
        composition_order: ["binding_descriptor_path", "focus_opened_registry_path"],
        cost_order: [
            "h4_s3_angular_shell",
            "fiber_circular_abs_q29",
            "torsion_circular_abs_q29",
        ],
        unique_minimum_required: true,
        conversation_disabled_ranking_enabled: false,
        binding_order_control: "parsed_binding_rows_reversed",
        query_h4_access: "prevalidated_nested_row_and_coordinate_table_reads",
        work: query_work_schedule(),
    }
}

fn routing_policy_kappa() -> Result<String, ConversationEntitySpinPathError> {
    let bytes = serde_json::to_vec(&routing_policy_wire())
        .map_err(|error| ConversationEntitySpinPathError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
}

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
) -> ConversationEntityHierarchyLevelAudit {
    ConversationEntityHierarchyLevelAudit {
        level: level.to_owned(),
        left_identity_kappa: left_identity.to_owned(),
        right_identity_kappa: right_identity.to_owned(),
        identity_equal: left_identity == right_identity,
        left_observed_routes: left.observed_routes,
        right_observed_routes: right.observed_routes,
        left_state: left.state,
        right_state: right.state,
        left_root_coordinate: left.root_coordinate,
        right_root_coordinate: right.root_coordinate,
        ordered_state_equal: left.state == right.state
            && left.observed_routes == right.observed_routes
            && left.root_coordinate == right.root_coordinate,
    }
}

fn frozen_global_epoch() -> Result<String, ConversationEntitySpinPathError> {
    Ok(canonical_global_epoch(&[GLOBAL_SNAPSHOT_UNIT.to_vec()])?)
}

fn construction_geometry_input(
    construction: &[SourceDocument],
) -> Result<ConversationInput, ConversationEntitySpinPathError> {
    let mut turns = Vec::with_capacity(construction.len() * 3);
    for document in construction {
        let (binding, focus, active) = split_construction_document(&document.text)?;
        parse_construction_binding(binding)?;
        parse_focus_turn(focus)?;
        parse_construction_readout(active)?;
        turns.push(TurnInput {
            turn_id: format!("construction-{}-binding", document.id),
            paragraphs: vec![ParagraphInput {
                sentences: vec![binding.to_vec()],
            }],
        });
        turns.push(TurnInput {
            turn_id: format!("construction-{}-focus", document.id),
            paragraphs: vec![ParagraphInput {
                sentences: split_two_sentences(focus)?,
            }],
        });
        turns.push(TurnInput {
            turn_id: format!("construction-{}-active", document.id),
            paragraphs: vec![ParagraphInput {
                sentences: vec![active.to_vec()],
            }],
        });
    }
    let global_snapshot_units = vec![GLOBAL_SNAPSHOT_UNIT.to_vec()];
    Ok(ConversationInput {
        identity_scope: CONSTRUCTION_IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns,
    })
}

fn observed_conversation_input(
    binding_turn: &[u8],
    focus_turn: &[u8],
    active_query: &[u8],
) -> Result<ConversationInput, ConversationEntitySpinPathError> {
    parse_binding_turn(binding_turn)?;
    parse_focus_turn(focus_turn)?;
    parse_active_query(active_query)?;
    let global_snapshot_units = vec![GLOBAL_SNAPSHOT_UNIT.to_vec()];
    Ok(ConversationInput {
        identity_scope: HELDOUT_IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: vec![
            TurnInput {
                turn_id: BINDING_TURN_ID.to_owned(),
                paragraphs: vec![ParagraphInput {
                    sentences: split_two_sentences(binding_turn)?,
                }],
            },
            TurnInput {
                turn_id: FOCUS_TURN_ID.to_owned(),
                paragraphs: vec![ParagraphInput {
                    sentences: split_two_sentences(focus_turn)?,
                }],
            },
            TurnInput {
                turn_id: ACTIVE_TURN_ID.to_owned(),
                paragraphs: vec![ParagraphInput {
                    sentences: vec![active_query.to_vec()],
                }],
            },
        ],
    })
}

fn parse_construction_document(
    document: &SourceDocument,
) -> Result<ParsedConstructionRow, ConversationEntitySpinPathError> {
    let (binding_turn, focus_turn, active_readout) = split_construction_document(&document.text)?;
    let binding = parse_construction_binding(binding_turn)?;
    let focus = parse_focus_turn(focus_turn)?;
    if focus.opener != binding.entity || focus.waiter == binding.entity {
        return Err(ConversationEntitySpinPathError::Invalid(
            "construction focus roles do not bind exactly the construction entity".to_owned(),
        ));
    }
    let candidate = parse_construction_readout(active_readout)?;
    Ok(ParsedConstructionRow {
        entity: binding.entity,
        descriptor: binding.descriptor,
        candidate,
    })
}

fn split_construction_document(
    text: &[u8],
) -> Result<(&[u8], &[u8], &[u8]), ConversationEntitySpinPathError> {
    let (binding, rest) = split_once_bytes(text, b"\n\n").ok_or_else(|| {
        ConversationEntitySpinPathError::Invalid(
            "construction document lacks the first completed-turn boundary".to_owned(),
        )
    })?;
    let (focus, active) = split_once_bytes(rest, b"\n\n").ok_or_else(|| {
        ConversationEntitySpinPathError::Invalid(
            "construction document lacks the active-turn boundary".to_owned(),
        )
    })?;
    if contains_bytes(active, b"\n\n") {
        return Err(ConversationEntitySpinPathError::Invalid(
            "construction document contains an extra turn boundary".to_owned(),
        ));
    }
    Ok((binding, focus, active))
}

fn parse_construction_binding(
    binding_turn: &[u8],
) -> Result<EntityBinding, ConversationEntitySpinPathError> {
    if contains_bytes(binding_turn, b". ") {
        return Err(ConversationEntitySpinPathError::Invalid(
            "construction binding turn must contain exactly one fact".to_owned(),
        ));
    }
    parse_fact(binding_turn)
}

fn parse_binding_turn(
    binding_turn: &[u8],
) -> Result<Vec<EntityBinding>, ConversationEntitySpinPathError> {
    let sentences = split_two_sentences(binding_turn)?;
    if sentences.len() != CONVERSATION_ENTITY_BINDING_FACTS {
        return Err(ConversationEntitySpinPathError::Invalid(
            "binding turn requires exactly two facts".to_owned(),
        ));
    }
    let facts = sentences
        .iter()
        .map(|sentence| parse_fact(sentence))
        .collect::<Result<Vec<_>, _>>()?;
    if facts[0].entity == facts[1].entity || facts[0].descriptor == facts[1].descriptor {
        return Err(ConversationEntitySpinPathError::Invalid(
            "binding turn requires distinct entities and descriptors".to_owned(),
        ));
    }
    Ok(facts)
}

fn parse_fact(sentence: &[u8]) -> Result<EntityBinding, ConversationEntitySpinPathError> {
    let grammar_sentence = sentence.strip_suffix(b" ").unwrap_or(sentence);
    let text = std::str::from_utf8(grammar_sentence).map_err(|_| {
        ConversationEntitySpinPathError::Invalid("binding fact is not valid UTF-8".to_owned())
    })?;
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    if words.len() != 5 || words[1] != "carried" || words[2] != "the" || words[4] != "marker." {
        return Err(ConversationEntitySpinPathError::Invalid(
            "binding fact violates the frozen grammar".to_owned(),
        ));
    }
    let entity = words[0].as_bytes().to_vec();
    let descriptor = words[3].as_bytes().to_vec();
    validate_word(&entity, "binding entity")?;
    validate_word(&descriptor, "binding descriptor")?;
    let expected = format!(
        "{} carried the {} marker.",
        std::str::from_utf8(&entity).map_err(|_| {
            ConversationEntitySpinPathError::Invalid("binding entity is not valid UTF-8".to_owned())
        })?,
        std::str::from_utf8(&descriptor).map_err(|_| {
            ConversationEntitySpinPathError::Invalid(
                "binding descriptor is not valid UTF-8".to_owned(),
            )
        })?
    );
    if grammar_sentence != expected.as_bytes() {
        return Err(ConversationEntitySpinPathError::Invalid(
            "binding fact spacing differs from frozen grammar".to_owned(),
        ));
    }
    Ok(EntityBinding { entity, descriptor })
}

fn parse_focus_turn(focus_turn: &[u8]) -> Result<FocusRoles, ConversationEntitySpinPathError> {
    let sentences = split_two_sentences(focus_turn)?;
    let first = std::str::from_utf8(&sentences[0]).map_err(|_| {
        ConversationEntitySpinPathError::Invalid("focus opener is not valid UTF-8".to_owned())
    })?;
    let second = std::str::from_utf8(&sentences[1]).map_err(|_| {
        ConversationEntitySpinPathError::Invalid("focus waiter is not valid UTF-8".to_owned())
    })?;
    let opener_words = first.split_ascii_whitespace().collect::<Vec<_>>();
    let waiter_words = second.split_ascii_whitespace().collect::<Vec<_>>();
    if opener_words.len() != 4
        || opener_words[1] != "opened"
        || opener_words[2] != "the"
        || opener_words[3] != "registry."
        || waiter_words.len() != 2
        || waiter_words[1] != "waited."
    {
        return Err(ConversationEntitySpinPathError::Invalid(
            "focus turn violates the frozen grammar".to_owned(),
        ));
    }
    let opener = opener_words[0].as_bytes().to_vec();
    let waiter = waiter_words[0].as_bytes().to_vec();
    validate_word(&opener, "focus opener")?;
    validate_word(&waiter, "focus waiter")?;
    if opener == waiter {
        return Err(ConversationEntitySpinPathError::Invalid(
            "focus opener and waiter must be distinct".to_owned(),
        ));
    }
    let expected = format!(
        "{} opened the registry. {} waited.",
        std::str::from_utf8(&opener).map_err(|_| {
            ConversationEntitySpinPathError::Invalid("focus opener is not valid UTF-8".to_owned())
        })?,
        std::str::from_utf8(&waiter).map_err(|_| {
            ConversationEntitySpinPathError::Invalid("focus waiter is not valid UTF-8".to_owned())
        })?
    );
    if focus_turn != expected.as_bytes() {
        return Err(ConversationEntitySpinPathError::Invalid(
            "focus turn spacing differs from frozen grammar".to_owned(),
        ));
    }
    Ok(FocusRoles { opener, waiter })
}

fn parse_active_query(active_query: &[u8]) -> Result<(), ConversationEntitySpinPathError> {
    if active_query != ACTIVE_QUERY_BYTES {
        return Err(ConversationEntitySpinPathError::Invalid(
            "active query differs from the frozen grammar".to_owned(),
        ));
    }
    Ok(())
}

fn parse_construction_readout(readout: &[u8]) -> Result<Vec<u8>, ConversationEntitySpinPathError> {
    let text = std::str::from_utf8(readout).map_err(|_| {
        ConversationEntitySpinPathError::Invalid(
            "construction readout is not valid UTF-8".to_owned(),
        )
    })?;
    let prefix = "The active registry code is ";
    let candidate_with_period = text.strip_prefix(prefix).ok_or_else(|| {
        ConversationEntitySpinPathError::Invalid(
            "construction readout violates the frozen grammar".to_owned(),
        )
    })?;
    let candidate = candidate_with_period
        .strip_suffix('.')
        .ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "construction readout lacks its final period".to_owned(),
            )
        })?
        .as_bytes()
        .to_vec();
    validate_word(&candidate, "construction candidate")?;
    let expected = format!(
        "The active registry code is {}.",
        std::str::from_utf8(&candidate).map_err(|_| {
            ConversationEntitySpinPathError::Invalid(
                "construction candidate is not valid UTF-8".to_owned(),
            )
        })?
    );
    if readout != expected.as_bytes() {
        return Err(ConversationEntitySpinPathError::Invalid(
            "construction readout spacing differs from frozen grammar".to_owned(),
        ));
    }
    Ok(candidate)
}

fn resolve_descriptor(
    facts: &[EntityBinding],
    entity: &[u8],
) -> Result<Vec<u8>, ConversationEntitySpinPathError> {
    let mut matches = Vec::new();
    for fact in facts {
        if fact.entity == entity {
            matches.push(fact.descriptor.clone());
        }
    }
    if matches.len() != 1 {
        return Err(ConversationEntitySpinPathError::Invalid(
            "focus entity does not resolve exactly one earlier binding".to_owned(),
        ));
    }
    Ok(matches.remove(0))
}

fn split_two_sentences(bytes: &[u8]) -> Result<Vec<Vec<u8>>, ConversationEntitySpinPathError> {
    let (left, right) = split_once_bytes(bytes, b". ").ok_or_else(|| {
        ConversationEntitySpinPathError::Invalid(
            "completed turn does not contain exactly two sentence slots".to_owned(),
        )
    })?;
    if left.is_empty() || right.is_empty() || contains_bytes(right, b". ") || !right.ends_with(b".")
    {
        return Err(ConversationEntitySpinPathError::Invalid(
            "completed turn has malformed or extra sentence slots".to_owned(),
        ));
    }
    let mut first = left.to_vec();
    first.extend_from_slice(b". ");
    Ok(vec![first, right.to_vec()])
}

fn render_prompt(
    binding_turn: &[u8],
    focus_turn: &[u8],
    active_query: &[u8],
) -> Result<Vec<u8>, ConversationEntitySpinPathError> {
    let capacity = binding_turn
        .len()
        .checked_add(2)
        .and_then(|value| value.checked_add(focus_turn.len()))
        .and_then(|value| value.checked_add(2))
        .and_then(|value| value.checked_add(active_query.len()))
        .ok_or(ConversationEntitySpinPathError::ArithmeticOverflow)?;
    let mut prompt = Vec::with_capacity(capacity);
    prompt.extend_from_slice(binding_turn);
    prompt.extend_from_slice(b"\n\n");
    prompt.extend_from_slice(focus_turn);
    prompt.extend_from_slice(b"\n\n");
    prompt.extend_from_slice(active_query);
    Ok(prompt)
}

fn validate_word(bytes: &[u8], field: &str) -> Result<(), ConversationEntitySpinPathError> {
    if bytes.is_empty()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return Err(ConversationEntitySpinPathError::Invalid(format!(
            "{field} is not one bounded lexical word"
        )));
    }
    Ok(())
}

fn compile_binding_leaves(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
    descriptor: &[u8],
) -> Result<Vec<CompiledSpinLeaf>, ConversationEntitySpinPathError> {
    [
        b"carried".as_slice(),
        b"the".as_slice(),
        descriptor,
        b"marker".as_slice(),
    ]
    .into_iter()
    .map(|surface| compile_leaf(codec, artifact, table, surface))
    .collect()
}

fn compile_focus_leaves(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Vec<CompiledSpinLeaf>, ConversationEntitySpinPathError> {
    [
        b"opened".as_slice(),
        b"the".as_slice(),
        b"registry".as_slice(),
    ]
    .into_iter()
    .map(|surface| compile_leaf(codec, artifact, table, surface))
    .collect()
}

fn compile_leaf(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
    surface: &[u8],
) -> Result<CompiledSpinLeaf, ConversationEntitySpinPathError> {
    let registered = compile_registered_route_value(codec, artifact, surface)?;
    leaf_from_address(surface, &registered.address, registered.witness, table)
}

fn compile_route_value_witness(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    surface: &[u8],
) -> Result<ConversationEntityRouteValueWitness, ConversationEntitySpinPathError> {
    Ok(compile_registered_route_value(codec, artifact, surface)?.witness)
}

fn compile_registered_route_value(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    surface: &[u8],
) -> Result<CompiledRouteValue, ConversationEntitySpinPathError> {
    let encoded = codec.encode(0, 0, surface)?;
    if encoded.units.len() != 1
        || !encoded.units[0].leading_bytes.is_empty()
        || !encoded.trailing_bytes.is_empty()
        || codec.decode(&encoded)? != surface
    {
        return Err(ConversationEntitySpinPathError::Invalid(
            "path surface is not one canonical lexical unit".to_owned(),
        ));
    }
    let address = artifact
        .lexical_route_address(encoded.units[0].unit_id)?
        .ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "path surface has no exact registered geometric address".to_owned(),
            )
        })?;
    let inverse = artifact
        .lexical_route_value_for_address(&address)?
        .ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "path address has no exact lexical inverse".to_owned(),
            )
        })?;
    if inverse.payload_bytes != surface {
        return Err(ConversationEntitySpinPathError::Invalid(
            "path address lexical inverse differs from its surface".to_owned(),
        ));
    }
    if inverse.lexical_unit_id != encoded.units[0].unit_id
        || inverse.address_kappa
            != address
                .canonical_kappa()
                .map_err(|error| ConversationEntitySpinPathError::Invalid(error.to_string()))?
        || inverse.payload_cid != address.payload_cid
    {
        return Err(ConversationEntitySpinPathError::Invalid(
            "registered route address/value witness does not reproduce".to_owned(),
        ));
    }
    Ok(CompiledRouteValue {
        witness: ConversationEntityRouteValueWitness {
            lexical_unit_id: inverse.lexical_unit_id,
            registry_address_index: inverse.registry_address_index,
            prime: inverse.prime,
            address_kappa: inverse.address_kappa,
            radial_zphi: [address.radial.a, address.radial.b],
            payload_cid: inverse.payload_cid,
            payload_hex: hex::encode(inverse.payload_bytes),
        },
        address,
    })
}

fn leaf_from_address(
    surface: &[u8],
    address: &GeometricAddress,
    value: ConversationEntityRouteValueWitness,
    table: &H4BinaryIcosahedralClosure,
) -> Result<CompiledSpinLeaf, ConversationEntitySpinPathError> {
    let h4 = exact_s3_spin_to_h4(address.spin.s3.raw(), table)?;
    let mapped_h4_coordinate = h4.root_coordinate(table)?;
    let trace = ConversationEntitySpinLeafTrace {
        surface_hex: hex::encode(surface),
        lexical_unit_id: value.lexical_unit_id,
        registry_address_index: value.registry_address_index,
        prime: value.prime,
        address_kappa: value.address_kappa,
        radial_zphi: value.radial_zphi,
        payload_cid: value.payload_cid,
        s3_q30: address.spin.s3.raw(),
        hopf_q30: address.spin.hopf.raw(),
        fiber_q29: address.spin.fiber.raw(),
        torsion_q29: address.spin.torsion.raw(),
        mapped_h4_coordinate,
    };
    Ok(CompiledSpinLeaf {
        state: SpinPathState {
            h4,
            fiber_q29: i64::from(address.spin.fiber.raw()),
            torsion_q29: i64::from(address.spin.torsion.raw()),
        },
        trace,
    })
}

fn fold_leaves(
    leaves: &[CompiledSpinLeaf],
    table: &H4BinaryIcosahedralClosure,
) -> Result<SpinPathState, ConversationEntitySpinPathError> {
    if leaves.len() != CONVERSATION_ENTITY_PATH_LEAVES {
        return Err(ConversationEntitySpinPathError::Invalid(
            "conversation descriptor path has the wrong leaf count".to_owned(),
        ));
    }
    leaves
        .iter()
        .try_fold(SpinPathState::identity(table)?, |state, leaf| {
            state.compose(leaf.state, table)
        })
}

fn exact_s3_spin_to_h4(
    raw: [i32; 4],
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, ConversationEntitySpinPathError> {
    let mut coordinate = [[0_i64; 2]; 4];
    for (target, value) in coordinate.iter_mut().zip(raw) {
        if value & Q29_H4_SCALE_MASK != 0 {
            return Err(ConversationEntitySpinPathError::Invalid(
                "S3 spin is not an exact scaled H4 coordinate".to_owned(),
            ));
        }
        target[0] = i64::from(value >> Q29_H4_SCALE_SHIFT);
    }
    let expected = H4RootCoordinate {
        scaled_zphi_quaternion: coordinate,
    };
    let mut matches = Vec::new();
    for offset in 0..table.root_count {
        let offset = u16::try_from(offset)
            .map_err(|_| ConversationEntitySpinPathError::ArithmeticOverflow)?;
        let index = OpaqueH4TableIndex::from_table_offset(offset, table).ok_or_else(|| {
            ConversationEntitySpinPathError::Invalid(
                "H4 map addressed outside the exact table".to_owned(),
            )
        })?;
        let state = OrderedH4FoldState::from_table_index(index, table)?;
        if state.root_coordinate(table)? == expected {
            matches.push(state);
        }
    }
    if matches.len() != 1 {
        return Err(ConversationEntitySpinPathError::Invalid(format!(
            "S3 spin has {} exact H4 coordinate matches",
            matches.len()
        )));
    }
    Ok(matches[0])
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

fn spin_map_kappa(
    table: &H4BinaryIcosahedralClosure,
) -> Result<String, ConversationEntitySpinPathError> {
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
        .collect::<Result<Vec<_>, ConversationEntitySpinPathError>>()?;
    let bytes = serde_json::to_vec(&SpinMapWire {
        schema: 1,
        domain: SPIN_MAP_DOMAIN,
        exact_mapping_rule: SPIN_MAP_RULE_IDENTITY,
        h4_root_table_kappa: table.h4_root_table_kappa.clone(),
        rows,
    })
    .map_err(|error| ConversationEntitySpinPathError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
}

fn decision(
    arm: ConversationEntitySpinPathArm,
    token: u32,
    unique_minimum: Option<u32>,
    minimum_cost: Option<ConversationEntitySpinPathCost>,
    support_tokens: Vec<u32>,
    work: ConversationEntitySpinPathWork,
) -> ConversationEntitySpinPathDecision {
    ConversationEntitySpinPathDecision {
        arm,
        token,
        unique_minimum,
        minimum_cost,
        support_tokens,
        work,
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

fn circular_abs_q29(value: i64) -> u64 {
    wrap_phase_q29(value).unsigned_abs()
}

fn split_once_bytes<'a>(bytes: &'a [u8], needle: &[u8]) -> Option<(&'a [u8], &'a [u8])> {
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

    fn finish(
        self,
        table: &SourceFreeTable,
    ) -> Result<Continuation, ConversationEntitySpinPathError> {
        Ok(Continuation {
            decoded: table.decode_tokens(&self.generated)?,
            tokens: self.generated,
            stop: self.stop,
        })
    }
}
