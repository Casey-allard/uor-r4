//! Frozen paragraph-scope SpinTorsion path probe for issue #973.
//!
//! `ParagraphEntitySpinPathR4V1` owns no candidate admission. It can only
//! rank the exact maximum-count tie already exposed by the bound #953
//! `MultiscaleCountRadiusR4V1` result. A typed entity key resolves one of two
//! construction-recurrent descriptor paths. Those paths are folded in the
//! exact stored lexical `SpinTorsionState` frame: the S3 component is included
//! exactly in the fixed H4 root table, while fiber and torsion compose as
//! wrapped Q29 phases. Selection is the unique lexicographic minimum of
//! `(H4 angular shell, fiber distance, torsion distance)`.
//!
//! The bounded fixture deliberately leaves the H4 shell tied; fiber/torsion
//! phase is the load-bearing coordinate. This is therefore a narrow synthetic
//! phase-path recurrence mechanism, not semantic similarity, candidate-address
//! geometry, or general paragraph understanding.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::canonical_lexical_ingestion::{
    canonical_global_epoch, validate_h4_binary_icosahedral_closure, CanonicalLexicalCodec,
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

const OPERATOR_MAGIC: [u8; 8] = *b"PESPIN01";
const OPERATOR_SCHEMA: u32 = 1;
const OPERATOR_DOMAIN: &str = "uor-r4.paragraph-entity-spin-path/1";
const SPIN_MAP_DOMAIN: &str = "uor-r4.canonical-s3-spin-to-h4/1";
const SPIN_MAP_RULE_IDENTITY: &str = "exact-s3-q30-components-divisible-by-2^29; arithmetic-right-shift-29-to-scaled-zphi-rational-coefficients; phi-coefficients-zero; unique-coordinate-membership-in-canonical-120-root-h4-table; reject-nonmultiple-nonmember-and-alias; no-prime-hash-candidate-or-nearest-root-placement";
const GRAMMAR_IDENTITY_BYTES: &[u8] = b"uor-r4 paragraph entity spin path grammar/1\n<ENTITY> carried the <DESCRIPTOR> marker.\nFor <ENTITY> the registry code is";
const ROUTING_POLICY_IDENTITY: &str = "uor-r4 paragraph entity spin path routing policy/1\nphase=wrapped-q29[-1686629713,1686629713)\ncost=lexicographic(h4-s3-angular-shell,fiber-circular-abs-q29,torsion-circular-abs-q29)\nselection=unique-minimum-or-abstain\nparagraph-disabled=measure-but-do-not-rank\ncontrols=real,paragraph-disabled,entity-binding-permuted,parsed-fact-vector-reversed\nquery-h4=prevalidated-row-and-coordinate-table-reads\nwork=2-facts,2-entity-comparisons,2-descriptor-comparisons,4-leaves,6-products,2-inverses,12-phase-additions,4-phase-distances,2-shells,2-cost-comparisons,1-final-choice";
const IDENTITY_SCOPE: &str = "issue-973/paragraph-entity-spin-path-v1";
const TURN_ID: &str = "construction-turn-0001";
const FROZEN_CONSTRUCTION: [(&str, &[u8]); 2] = [
    (
        "20",
        b"Nora carried the striped marker.\n\nFor Nora the registry code is amber.",
    ),
    (
        "21",
        b"Owen carried the dotted marker.\n\nFor Owen the registry code is cobalt.",
    ),
];
const PHASE_HALF_Q29: i64 = 1_686_629_713;
const PHASE_MODULUS_Q29: i64 = 3_373_259_426;
const Q29_H4_SCALE_SHIFT: u32 = 29;
const Q29_H4_SCALE_MASK: i32 = (1_i32 << Q29_H4_SCALE_SHIFT) - 1;

pub const PARAGRAPH_ENTITY_FACTS: usize = 2;
pub const PARAGRAPH_ENTITY_CANDIDATES: usize = 2;
pub const PARAGRAPH_ENTITY_PATH_LEAVES: usize = 4;
pub const MAX_PARAGRAPH_ENTITY_UNITS: usize = 64;
pub const MAX_PARAGRAPH_ENTITY_BYTES: usize = 1024;
pub const MAX_PARAGRAPH_ENTITY_OPERATOR_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParagraphEntitySpinPathError {
    Invalid(String),
    SourceFree(String),
    CanonicalLexical(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for ParagraphEntitySpinPathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::SourceFree(reason) => write!(formatter, "source-free table: {reason}"),
            Self::CanonicalLexical(reason) => write!(formatter, "canonical lexical: {reason}"),
            Self::Serialization(reason) => write!(formatter, "serialization: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("paragraph entity spin-path arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for ParagraphEntitySpinPathError {}

impl From<SourceFreeTableError> for ParagraphEntitySpinPathError {
    fn from(error: SourceFreeTableError) -> Self {
        Self::SourceFree(error.to_string())
    }
}

impl From<CanonicalLexicalError> for ParagraphEntitySpinPathError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::CanonicalLexical(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParagraphEntitySpinPathArm {
    Real,
    ParagraphDisabled,
    EntityBindingPermuted,
    FactOrderReversed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParagraphEntitySpinPathAbstention {
    CostTie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ParagraphEntitySpinPathCost {
    pub angular_shell: H4S3AngularShell,
    pub fiber_distance_q29: u64,
    pub torsion_distance_q29: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ParagraphEntitySpinPathStateTrace {
    pub h4_coordinate: H4RootCoordinate,
    pub fiber_q29: i64,
    pub torsion_q29: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParagraphEntityRouteValueWitness {
    pub lexical_unit_id: u32,
    pub registry_address_index: u16,
    pub prime: u32,
    pub address_kappa: String,
    pub radial_zphi: [i64; 2],
    pub payload_cid: String,
    pub payload_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParagraphEntitySpinLeafTrace {
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
pub struct ParagraphEntitySpinPrototypeTrace {
    pub candidate_token: u32,
    pub candidate_hex: String,
    pub candidate_value: ParagraphEntityRouteValueWitness,
    pub candidate_geometry_used_for_ranking: bool,
    pub descriptor_hex: String,
    pub leaves: Vec<ParagraphEntitySpinLeafTrace>,
    pub path_state: ParagraphEntitySpinPathStateTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ParagraphEntitySpinRelationTrace {
    pub query_state: ParagraphEntitySpinPathStateTrace,
    pub relative_state: ParagraphEntitySpinPathStateTrace,
    pub measured_cost: ParagraphEntitySpinPathCost,
    pub ranking_cost: Option<ParagraphEntitySpinPathCost>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParagraphEntitySpinCandidateEvidence {
    pub token: u32,
    pub count: u64,
    pub candidate_hex: String,
    pub prototype_descriptor_hex: String,
    pub prototype_state: ParagraphEntitySpinPathStateTrace,
    pub real: ParagraphEntitySpinRelationTrace,
    pub paragraph_disabled: ParagraphEntitySpinRelationTrace,
    pub entity_binding_permuted: ParagraphEntitySpinRelationTrace,
    pub fact_order_reversed: ParagraphEntitySpinRelationTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ParagraphEntitySpinPathWork {
    pub local: MultiscaleCountRadiusWork,
    pub prior_fact_slots_scanned: u64,
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
pub struct ParagraphEntitySpinPathDecision {
    pub arm: ParagraphEntitySpinPathArm,
    pub token: u32,
    pub unique_minimum: Option<u32>,
    pub minimum_cost: Option<ParagraphEntitySpinPathCost>,
    pub support_tokens: Vec<u32>,
    pub work: ParagraphEntitySpinPathWork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedParagraphEntitySpinPathPrediction {
    pub local: MatchedGeometricPrediction,
    pub query_entity_hex: String,
    pub real_descriptor_hex: String,
    pub permuted_descriptor_hex: String,
    pub prior_candidate_occurrences: u32,
    pub prototypes: Vec<ParagraphEntitySpinPrototypeTrace>,
    pub candidate_evidence: Vec<ParagraphEntitySpinCandidateEvidence>,
    pub real: ParagraphEntitySpinPathDecision,
    pub paragraph_disabled: ParagraphEntitySpinPathDecision,
    pub entity_binding_permuted: ParagraphEntitySpinPathDecision,
    pub fact_order_reversed: ParagraphEntitySpinPathDecision,
    pub operator_abstention: Option<ParagraphEntitySpinPathAbstention>,
    pub support_matched: bool,
    pub work_matched: bool,
    pub teacher_calls: u64,
    pub provider_calls: u64,
    pub source_weight_reads: u64,
    pub future_unit_reads: u64,
    pub target_reads: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedParagraphEntitySpinPathContinuation {
    pub first_decision: MatchedParagraphEntitySpinPathPrediction,
    pub real: Continuation,
    pub paragraph_disabled: Continuation,
    pub entity_binding_permuted: Continuation,
    pub fact_order_reversed: Continuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpinPathState {
    h4: OrderedH4FoldState,
    fiber_q29: i64,
    torsion_q29: i64,
}

impl SpinPathState {
    fn identity(table: &H4BinaryIcosahedralClosure) -> Result<Self, ParagraphEntitySpinPathError> {
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
    ) -> Result<Self, ParagraphEntitySpinPathError> {
        Ok(Self {
            h4: self.h4.compose(right.h4, table)?,
            fiber_q29: wrap_phase_q29(
                self.fiber_q29
                    .checked_add(right.fiber_q29)
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                self.torsion_q29
                    .checked_add(right.torsion_q29)
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledSpinLeaf {
    trace: ParagraphEntitySpinLeafTrace,
    state: SpinPathState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledRouteValue {
    address: GeometricAddress,
    witness: ParagraphEntityRouteValueWitness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidatePrototype {
    candidate_token: u32,
    candidate_bytes: Vec<u8>,
    descriptor: Vec<u8>,
    candidate_value: ParagraphEntityRouteValueWitness,
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
struct ArmEvaluation {
    query_state: SpinPathState,
    relations: Vec<(SpinPathState, ParagraphEntitySpinPathCost)>,
    winner: Option<u32>,
    minimum_cost: Option<ParagraphEntitySpinPathCost>,
}

/// Construction-bound, candidate-admission-neutral paragraph path operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphEntitySpinPathR4V1 {
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
    spin_map_kappa: String,
    prototypes: Vec<CandidatePrototype>,
}

impl ParagraphEntitySpinPathR4V1 {
    pub fn compile(
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        construction: &[SourceDocument],
    ) -> Result<Self, ParagraphEntitySpinPathError> {
        if base_overlay.table_artifact_cid() != table.artifact_cid() {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "#953 overlay table binding mismatches".to_owned(),
            ));
        }
        if construction.len() != PARAGRAPH_ENTITY_CANDIDATES {
            return Err(ParagraphEntitySpinPathError::Invalid(format!(
                "paragraph spin-path construction requires exactly {PARAGRAPH_ENTITY_CANDIDATES} documents"
            )));
        }
        if construction
            .iter()
            .any(|document| d3_is_held_out(&document.id))
            || !table.is_bound_to_construction_documents(construction)
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "operator construction is not the exact D3-construction set bound to the table"
                    .to_owned(),
            ));
        }
        let mut sorted = construction.to_vec();
        sorted.sort_by(|left, right| left.id.cmp(&right.id));
        if sorted.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "construction document IDs are not unique".to_owned(),
            ));
        }
        for (document, (expected_id, expected_text)) in sorted.iter().zip(FROZEN_CONSTRUCTION) {
            if document.id != expected_id || document.text.as_slice() != expected_text {
                return Err(ParagraphEntitySpinPathError::Invalid(
                    "construction differs from the exact frozen #973 documents".to_owned(),
                ));
            }
        }
        let rebuilt_table = SourceFreeTable::compile(&sorted)?;
        if rebuilt_table.artifact_cid() != table.artifact_cid() {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "bound source-free table does not reproduce from the frozen construction"
                    .to_owned(),
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
            != PARAGRAPH_ENTITY_CANDIDATES
            || parsed
                .iter()
                .map(|row| row.descriptor.as_slice())
                .collect::<BTreeSet<_>>()
                .len()
                != PARAGRAPH_ENTITY_CANDIDATES
            || parsed
                .iter()
                .map(|row| row.candidate.as_slice())
                .collect::<BTreeSet<_>>()
                .len()
                != PARAGRAPH_ENTITY_CANDIDATES
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
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
                    .map_err(|_| ParagraphEntitySpinPathError::ArithmeticOverflow)?;
                let index =
                    OpaqueH4TableIndex::from_table_offset(offset, &h4_table).ok_or_else(|| {
                        ParagraphEntitySpinPathError::Invalid(
                            "H4 query cache index is outside the exact table".to_owned(),
                        )
                    })?;
                Ok(OrderedH4FoldState::from_table_index(index, &h4_table)?)
            })
            .collect::<Result<Vec<_>, ParagraphEntitySpinPathError>>()?;
        let h4_product_rows = h4_table
            .multiplication_indices
            .chunks_exact(h4_table.root_count)
            .map(<[u16]>::to_vec)
            .collect::<Vec<_>>();
        let h4_root_coordinates = h4_states
            .iter()
            .map(|state| Ok(state.root_coordinate(&h4_table)?))
            .collect::<Result<Vec<_>, ParagraphEntitySpinPathError>>()?;
        let grammar_kappa = blake3_label(GRAMMAR_IDENTITY_BYTES);
        let routing_policy_kappa = routing_policy_kappa()?;
        let spin_map_kappa = spin_map_kappa(&h4_table)?;

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
                return Err(ParagraphEntitySpinPathError::Invalid(
                    "construction candidate is not one exact fitted lexical token".to_owned(),
                ));
            }
            let leaves =
                compile_descriptor_leaves(&codec, &route_artifact, &h4_table, &row.descriptor)?;
            let state = fold_leaves(&leaves, &h4_table)?;
            let candidate_value =
                compile_route_value_witness(&codec, &route_artifact, row.candidate.as_slice())?;
            prototypes.push(CandidatePrototype {
                candidate_token: candidate_tokens[0],
                candidate_bytes: candidate_with_boundary,
                descriptor: row.descriptor,
                candidate_value,
                leaves,
                state,
            });
        }
        prototypes.sort_by_key(|prototype| prototype.candidate_token);
        let token_or_state_alias = prototypes.windows(2).any(|pair| {
            pair[0].candidate_token >= pair[1].candidate_token || pair[0].state == pair[1].state
        });
        if prototypes.len() != PARAGRAPH_ENTITY_CANDIDATES || token_or_state_alias {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "candidate prototype tokens or complete SpinTorsion paths alias".to_owned(),
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
    ) -> Result<Self, ParagraphEntitySpinPathError> {
        if bytes.len() < OPERATOR_MAGIC.len()
            || bytes.len() > MAX_PARAGRAPH_ENTITY_OPERATOR_BYTES
            || bytes[..OPERATOR_MAGIC.len()] != OPERATOR_MAGIC
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path operator magic/size is invalid".to_owned(),
            ));
        }
        let expected = Self::compile(table, base_overlay, construction)?;
        if expected.to_bytes()? != bytes {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path operator is noncanonical or binding-drifted".to_owned(),
            ));
        }
        Ok(expected)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ParagraphEntitySpinPathError> {
        self.validate_internal()?;
        let prototypes = self.prototype_traces()?;
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
            spin_map_kappa: self.spin_map_kappa.clone(),
            max_facts: PARAGRAPH_ENTITY_FACTS,
            max_candidates: PARAGRAPH_ENTITY_CANDIDATES,
            path_leaves: PARAGRAPH_ENTITY_PATH_LEAVES,
            max_units: MAX_PARAGRAPH_ENTITY_UNITS,
            max_bytes: MAX_PARAGRAPH_ENTITY_BYTES,
            prototypes,
        };
        let payload = serde_json::to_vec(&wire)
            .map_err(|error| ParagraphEntitySpinPathError::Serialization(error.to_string()))?;
        let mut bytes = Vec::with_capacity(OPERATOR_MAGIC.len() + payload.len());
        bytes.extend_from_slice(&OPERATOR_MAGIC);
        bytes.extend_from_slice(&payload);
        if bytes.len() > MAX_PARAGRAPH_ENTITY_OPERATOR_BYTES {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path operator exceeds its byte ceiling".to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn artifact_cid(&self) -> Result<String, ParagraphEntitySpinPathError> {
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

    pub fn h4_root_table_kappa(&self) -> &str {
        &self.h4_table.h4_root_table_kappa
    }

    pub fn h4_multiplication_table_kappa(&self) -> &str {
        &self.h4_table.multiplication_table_kappa
    }

    pub fn prototype_traces(
        &self,
    ) -> Result<Vec<ParagraphEntitySpinPrototypeTrace>, ParagraphEntitySpinPathError> {
        self.prototypes
            .iter()
            .map(|prototype| {
                Ok(ParagraphEntitySpinPrototypeTrace {
                    candidate_token: prototype.candidate_token,
                    candidate_hex: hex::encode(&prototype.candidate_bytes),
                    candidate_value: prototype.candidate_value.clone(),
                    candidate_geometry_used_for_ranking: false,
                    descriptor_hex: hex::encode(&prototype.descriptor),
                    leaves: prototype
                        .leaves
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
    ) -> Result<SpinPathState, ParagraphEntitySpinPathError> {
        let left_offset = usize::from(left.h4.table_index().table_offset());
        let right_offset = usize::from(right.h4.table_index().table_offset());
        let product_offset = self
            .h4_product_rows
            .get(left_offset)
            .and_then(|row| row.get(right_offset))
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "query H4 product addressed outside the cached exact table".to_owned(),
                )
            })?;
        let h4 = self
            .h4_states
            .get(usize::from(product_offset))
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "query H4 product returned an invalid cached state".to_owned(),
                )
            })?;
        Ok(SpinPathState {
            h4,
            fiber_q29: wrap_phase_q29(
                left.fiber_q29
                    .checked_add(right.fiber_q29)
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                left.torsion_q29
                    .checked_add(right.torsion_q29)
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }

    fn query_inverse(
        &self,
        state: SpinPathState,
    ) -> Result<SpinPathState, ParagraphEntitySpinPathError> {
        let offset = usize::from(state.h4.table_index().table_offset());
        let inverse_offset = self
            .h4_table
            .inverse_indices
            .get(offset)
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "query H4 inverse addressed outside the cached exact table".to_owned(),
                )
            })?;
        let h4 = self
            .h4_states
            .get(usize::from(inverse_offset))
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "query H4 inverse returned an invalid cached state".to_owned(),
                )
            })?;
        Ok(SpinPathState {
            h4,
            fiber_q29: wrap_phase_q29(
                state
                    .fiber_q29
                    .checked_neg()
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
            torsion_q29: wrap_phase_q29(
                state
                    .torsion_q29
                    .checked_neg()
                    .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?,
            ),
        })
    }

    fn query_trace(
        &self,
        state: SpinPathState,
    ) -> Result<ParagraphEntitySpinPathStateTrace, ParagraphEntitySpinPathError> {
        let offset = usize::from(state.h4.table_index().table_offset());
        let h4_coordinate = self
            .h4_root_coordinates
            .get(offset)
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "query H4 trace addressed outside the cached exact table".to_owned(),
                )
            })?;
        Ok(ParagraphEntitySpinPathStateTrace {
            h4_coordinate,
            fiber_q29: state.fiber_q29,
            torsion_q29: state.torsion_q29,
        })
    }

    fn query_shell(
        &self,
        state: SpinPathState,
    ) -> Result<H4S3AngularShell, ParagraphEntitySpinPathError> {
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
            other => Err(ParagraphEntitySpinPathError::Invalid(format!(
                "cached H4 relative state has noncanonical signed S3 real coordinate {other:?}"
            ))),
        }
    }

    fn query_fold_leaves(
        &self,
        leaves: &[CompiledSpinLeaf],
    ) -> Result<SpinPathState, ParagraphEntitySpinPathError> {
        if leaves.len() != PARAGRAPH_ENTITY_PATH_LEAVES {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "descriptor path has the wrong leaf count".to_owned(),
            ));
        }
        let identity = self
            .h4_states
            .get(usize::from(self.h4_table.identity_index))
            .copied()
            .ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
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
        prior_paragraph: &[u8],
        active_query: &[u8],
    ) -> Result<MatchedParagraphEntitySpinPathPrediction, ParagraphEntitySpinPathError> {
        self.ensure_bound(table, base_overlay)?;
        let prompt_bytes = prior_paragraph
            .len()
            .checked_add(2)
            .and_then(|value| value.checked_add(active_query.len()))
            .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?;
        if prompt_bytes > MAX_PARAGRAPH_ENTITY_BYTES {
            return Err(ParagraphEntitySpinPathError::Invalid(format!(
                "combined paragraph boundary and query exceed the {MAX_PARAGRAPH_ENTITY_BYTES}-byte bound"
            )));
        }
        let facts = parse_prior_facts(prior_paragraph)?;
        let query_entity = parse_active_query(active_query)?;
        let real_descriptor = resolve_descriptor(&facts, &query_entity)?;
        let permuted_descriptor = resolve_other_descriptor(&facts, &query_entity)?;
        let mut reversed_facts = facts.clone();
        reversed_facts.reverse();
        let reversed_descriptor = resolve_descriptor(&reversed_facts, &query_entity)?;
        if reversed_descriptor != real_descriptor {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "fact-order reversal changed the typed entity binding".to_owned(),
            ));
        }
        let mut prompt = Vec::with_capacity(prompt_bytes);
        prompt.extend_from_slice(prior_paragraph);
        prompt.extend_from_slice(b"\n\n");
        prompt.extend_from_slice(active_query);
        let mut context = vec![BOS_TOKEN];
        context.extend(table.encode_text(&prompt)?);
        if context.len().saturating_sub(1) > MAX_PARAGRAPH_ENTITY_UNITS {
            return Err(ParagraphEntitySpinPathError::Invalid(format!(
                "paragraph prompt exceeds the {MAX_PARAGRAPH_ENTITY_UNITS}-unit bound"
            )));
        }
        let local = table.predict_multiscale_count_radius(&context, base_overlay)?;
        self.validate_local_support(table, &context, &local)?;

        let mut prior_candidate_occurrences = 0_u32;
        for token in context.iter().copied() {
            for prototype in &self.prototypes {
                if token == prototype.candidate_token {
                    prior_candidate_occurrences = prior_candidate_occurrences
                        .checked_add(1)
                        .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?;
                }
            }
        }
        if prior_candidate_occurrences != 0 {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph prompt contains an admitted candidate token".to_owned(),
            ));
        }

        let real_eval = self.evaluate_descriptor(&real_descriptor)?;
        let disabled_eval = self.evaluate_descriptor(&real_descriptor)?;
        let permuted_eval = self.evaluate_descriptor(&permuted_descriptor)?;
        let reversed_eval = self.evaluate_descriptor(&reversed_descriptor)?;
        let work = query_work_schedule().with_local(local.geometric_work);
        let support = local.max_count_tie_tokens.clone();
        let fallback = local.geometric_token;
        let real = decision(
            ParagraphEntitySpinPathArm::Real,
            real_eval.winner.unwrap_or(fallback),
            real_eval.winner,
            real_eval.minimum_cost,
            support.clone(),
            work,
        );
        let paragraph_disabled = decision(
            ParagraphEntitySpinPathArm::ParagraphDisabled,
            fallback,
            None,
            None,
            support.clone(),
            work,
        );
        let entity_binding_permuted = decision(
            ParagraphEntitySpinPathArm::EntityBindingPermuted,
            permuted_eval.winner.unwrap_or(fallback),
            permuted_eval.winner,
            permuted_eval.minimum_cost,
            support.clone(),
            work,
        );
        let fact_order_reversed = decision(
            ParagraphEntitySpinPathArm::FactOrderReversed,
            reversed_eval.winner.unwrap_or(fallback),
            reversed_eval.winner,
            reversed_eval.minimum_cost,
            support.clone(),
            work,
        );
        let mut candidate_evidence = Vec::with_capacity(self.prototypes.len());
        for (index, prototype) in self.prototypes.iter().enumerate() {
            let local_candidate = local.tie_evidence.get(index).ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "#953 tie evidence omitted a bound prototype candidate".to_owned(),
                )
            })?;
            if local_candidate.token != prototype.candidate_token {
                return Err(ParagraphEntitySpinPathError::Invalid(
                    "#953 tie evidence order differs from the bound prototype order".to_owned(),
                ));
            }
            candidate_evidence.push(ParagraphEntitySpinCandidateEvidence {
                token: prototype.candidate_token,
                count: local_candidate.count,
                candidate_hex: hex::encode(&prototype.candidate_bytes),
                prototype_descriptor_hex: hex::encode(&prototype.descriptor),
                prototype_state: self.query_trace(prototype.state)?,
                real: self.relation_trace(&real_eval, index, true)?,
                paragraph_disabled: self.relation_trace(&disabled_eval, index, false)?,
                entity_binding_permuted: self.relation_trace(&permuted_eval, index, true)?,
                fact_order_reversed: self.relation_trace(&reversed_eval, index, true)?,
            });
        }
        let support_matched = real.support_tokens == paragraph_disabled.support_tokens
            && real.support_tokens == entity_binding_permuted.support_tokens
            && real.support_tokens == fact_order_reversed.support_tokens;
        let work_matched = real.work == paragraph_disabled.work
            && real.work == entity_binding_permuted.work
            && real.work == fact_order_reversed.work
            && local.baseline_work == local.geometric_work;
        let operator_abstention = (real_eval.winner.is_none()
            || permuted_eval.winner.is_none()
            || reversed_eval.winner.is_none())
        .then_some(ParagraphEntitySpinPathAbstention::CostTie);

        Ok(MatchedParagraphEntitySpinPathPrediction {
            local,
            query_entity_hex: hex::encode(query_entity),
            real_descriptor_hex: hex::encode(real_descriptor),
            permuted_descriptor_hex: hex::encode(permuted_descriptor),
            prior_candidate_occurrences,
            prototypes: self.prototype_traces()?,
            candidate_evidence,
            real,
            paragraph_disabled,
            entity_binding_permuted,
            fact_order_reversed,
            operator_abstention,
            support_matched,
            work_matched,
            teacher_calls: 0,
            provider_calls: 0,
            source_weight_reads: 0,
            future_unit_reads: 0,
            target_reads: 0,
        })
    }

    pub fn continue_matched(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
        prior_paragraph: &[u8],
        active_query: &[u8],
        max_units: usize,
    ) -> Result<MatchedParagraphEntitySpinPathContinuation, ParagraphEntitySpinPathError> {
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(ParagraphEntitySpinPathError::Invalid(format!(
                "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        let first_decision =
            self.predict_matched(table, base_overlay, prior_paragraph, active_query)?;
        if first_decision.operator_abstention.is_some()
            || !first_decision.support_matched
            || !first_decision.work_matched
            || first_decision.real.unique_minimum.is_none()
            || first_decision
                .entity_binding_permuted
                .unique_minimum
                .is_none()
            || first_decision.fact_order_reversed.unique_minimum.is_none()
            || first_decision.paragraph_disabled.unique_minimum.is_some()
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path hard gate stopped before decoding".to_owned(),
            ));
        }
        let mut prompt = Vec::new();
        prompt.extend_from_slice(prior_paragraph);
        prompt.extend_from_slice(b"\n\n");
        prompt.extend_from_slice(active_query);
        let mut initial_context = vec![BOS_TOKEN];
        initial_context.extend(table.encode_text(&prompt)?);
        let mut real = ContinuationState::new(initial_context.clone());
        let mut disabled = ContinuationState::new(initial_context.clone());
        let mut permuted = ContinuationState::new(initial_context.clone());
        let mut reversed = ContinuationState::new(initial_context);
        real.accept(first_decision.real.token);
        disabled.accept(first_decision.paragraph_disabled.token);
        permuted.accept(first_decision.entity_binding_permuted.token);
        reversed.accept(first_decision.fact_order_reversed.token);
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
        Ok(MatchedParagraphEntitySpinPathContinuation {
            first_decision,
            real: real.finish(table)?,
            paragraph_disabled: disabled.finish(table)?,
            entity_binding_permuted: permuted.finish(table)?,
            fact_order_reversed: reversed.finish(table)?,
        })
    }

    fn validate_internal(&self) -> Result<(), ParagraphEntitySpinPathError> {
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
        if self.prototypes.len() != PARAGRAPH_ENTITY_CANDIDATES
            || self.construction_ids.len() != PARAGRAPH_ENTITY_CANDIDATES
            || self.construction_text_cids.len() != PARAGRAPH_ENTITY_CANDIDATES
            || !construction_ids_match
            || !construction_cids_match
            || !h4_rows_match
            || !h4_states_match
            || self.grammar_kappa != blake3_label(GRAMMAR_IDENTITY_BYTES)
            || self.routing_policy_kappa != routing_policy_kappa()?
            || self.spin_map_kappa != spin_map_kappa(&self.h4_table)?
            || self.route_artifact.codec_kappa() != self.codec.codec_kappa()
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path operator binding does not reproduce".to_owned(),
            ));
        }
        for prototype in &self.prototypes {
            let candidate_payload =
                prototype
                    .candidate_bytes
                    .strip_prefix(b" ")
                    .ok_or_else(|| {
                        ParagraphEntitySpinPathError::Invalid(
                            "candidate token lacks its frozen lexical boundary".to_owned(),
                        )
                    })?;
            if prototype.leaves.len() != PARAGRAPH_ENTITY_PATH_LEAVES
                || fold_leaves(&prototype.leaves, &self.h4_table)? != prototype.state
                || prototype.candidate_value.payload_hex != hex::encode(candidate_payload)
            {
                return Err(ParagraphEntitySpinPathError::Invalid(
                    "paragraph spin-path prototype does not reproduce".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn ensure_bound(
        &self,
        table: &SourceFreeTable,
        base_overlay: &MultiscaleCountRadiusR4V1,
    ) -> Result<(), ParagraphEntitySpinPathError> {
        if self.table_artifact_cid != table.artifact_cid()
            || self.base_overlay_artifact_cid != base_overlay.artifact_cid()
            || base_overlay.table_artifact_cid() != table.artifact_cid()
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "paragraph spin-path table/overlay binding mismatches".to_owned(),
            ));
        }
        Ok(())
    }

    fn validate_local_support(
        &self,
        table: &SourceFreeTable,
        context: &[u32],
        local: &MatchedGeometricPrediction,
    ) -> Result<(), ParagraphEntitySpinPathError> {
        if local.order != BackoffOrder::Trigram
            || !local.geometry_reachable
            || local.max_count != 1
            || local.max_count_tie_tokens.len() != PARAGRAPH_ENTITY_CANDIDATES
            || local.tie_evidence.len() != PARAGRAPH_ENTITY_CANDIDATES
            || local.baseline_support_tokens != local.geometric_support_tokens
            || local.baseline_support_tokens != local.max_count_tie_tokens
            || local.baseline_work != local.geometric_work
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
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
                return Err(ParagraphEntitySpinPathError::Invalid(
                    "#953 tie evidence is not the exact two-candidate support".to_owned(),
                ));
            }
        };
        let fallback = expected_tokens.first().copied().ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "bound paragraph prototype support is empty".to_owned(),
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
            return Err(ParagraphEntitySpinPathError::Invalid(
                "#953 admitted candidates, coordinates, radii, or fallback drifted".to_owned(),
            ));
        }
        if context.len() < 2
            || table.decode_tokens(&context[context.len() - 2..context.len() - 1])? != b" code"
            || table.decode_tokens(&context[context.len() - 1..])? != b" is"
        {
            return Err(ParagraphEntitySpinPathError::Invalid(
                "active query is not the frozen code/is trigram frame".to_owned(),
            ));
        }
        Ok(())
    }

    fn evaluate_descriptor(
        &self,
        descriptor: &[u8],
    ) -> Result<ArmEvaluation, ParagraphEntitySpinPathError> {
        let mut query_leaves = None;
        for prototype in &self.prototypes {
            if prototype.descriptor == descriptor {
                if query_leaves.is_some() {
                    return Err(ParagraphEntitySpinPathError::Invalid(
                        "query descriptor aliases multiple compiled SpinTorsion paths".to_owned(),
                    ));
                }
                query_leaves = Some(prototype.leaves.as_slice());
            }
        }
        let query_leaves = query_leaves.ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "query descriptor has no compiled SpinTorsion path".to_owned(),
            )
        })?;
        let query_state = self.query_fold_leaves(query_leaves)?;
        let mut relations = Vec::with_capacity(self.prototypes.len());
        for prototype in &self.prototypes {
            let relative = self.query_compose(self.query_inverse(prototype.state)?, query_state)?;
            let cost = ParagraphEntitySpinPathCost {
                angular_shell: self.query_shell(relative)?,
                fiber_distance_q29: circular_abs_q29(relative.fiber_q29),
                torsion_distance_q29: circular_abs_q29(relative.torsion_q29),
            };
            relations.push((relative, cost));
        }
        let mut minimum = ParagraphEntitySpinPathCost {
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
                        .ok_or(ParagraphEntitySpinPathError::ArithmeticOverflow)?;
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
                        ParagraphEntitySpinPathError::Invalid(
                            "unique minimum index is outside the fixed support".to_owned(),
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
    ) -> Result<ParagraphEntitySpinRelationTrace, ParagraphEntitySpinPathError> {
        let (relative, measured_cost) =
            evaluation.relations.get(candidate_index).ok_or_else(|| {
                ParagraphEntitySpinPathError::Invalid(
                    "candidate relation index is outside the fixed support".to_owned(),
                )
            })?;
        Ok(ParagraphEntitySpinRelationTrace {
            query_state: self.query_trace(evaluation.query_state)?,
            relative_state: self.query_trace(*relative)?,
            measured_cost: *measured_cost,
            ranking_cost: ranking_enabled.then_some(*measured_cost),
        })
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
    routing_policy: RoutingPolicyWire,
    spin_map_kappa: String,
    max_facts: usize,
    max_candidates: usize,
    path_leaves: usize,
    max_units: usize,
    max_bytes: usize,
    prototypes: Vec<ParagraphEntitySpinPrototypeTrace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct RoutingPolicyWire {
    identity: &'static str,
    phase_lower_inclusive_q29: i64,
    phase_upper_exclusive_q29: i64,
    phase_modulus_q29: i64,
    cost_order: [&'static str; 3],
    unique_minimum_required: bool,
    paragraph_disabled_ranking_enabled: bool,
    fact_order_control: &'static str,
    query_h4_access: &'static str,
    work: QueryWorkSchedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct QueryWorkSchedule {
    prior_fact_slots_scanned: u64,
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
    fn with_local(self, local: MultiscaleCountRadiusWork) -> ParagraphEntitySpinPathWork {
        ParagraphEntitySpinPathWork {
            local,
            prior_fact_slots_scanned: self.prior_fact_slots_scanned,
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
        prior_fact_slots_scanned: 2,
        entity_key_comparisons: 2,
        descriptor_row_comparisons: 2,
        stored_spin_leaf_reads: 4,
        h4_product_table_reads: 6,
        h4_inverse_table_reads: 2,
        phase_additions: 12,
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
        cost_order: [
            "h4_s3_angular_shell",
            "fiber_circular_abs_q29",
            "torsion_circular_abs_q29",
        ],
        unique_minimum_required: true,
        paragraph_disabled_ranking_enabled: false,
        fact_order_control: "parsed_fact_vector_reversed",
        query_h4_access: "prevalidated_nested_row_and_coordinate_table_reads",
        work: query_work_schedule(),
    }
}

fn routing_policy_kappa() -> Result<String, ParagraphEntitySpinPathError> {
    let bytes = serde_json::to_vec(&routing_policy_wire())
        .map_err(|error| ParagraphEntitySpinPathError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
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

fn construction_geometry_input(
    construction: &[SourceDocument],
) -> Result<ConversationInput, ParagraphEntitySpinPathError> {
    let global_snapshot_units = vec![b"registry".to_vec()];
    let mut paragraphs = Vec::with_capacity(construction.len() * 2);
    for document in construction {
        let (first, second) = split_once_bytes(&document.text, b"\n\n").ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "construction document lacks the declared paragraph boundary".to_owned(),
            )
        })?;
        paragraphs.push(ParagraphInput {
            sentences: vec![first.to_vec()],
        });
        paragraphs.push(ParagraphInput {
            sentences: vec![second.to_vec()],
        });
    }
    Ok(ConversationInput {
        identity_scope: IDENTITY_SCOPE.to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: vec![TurnInput {
            turn_id: TURN_ID.to_owned(),
            paragraphs,
        }],
    })
}

fn parse_construction_document(
    document: &SourceDocument,
) -> Result<ParsedConstructionRow, ParagraphEntitySpinPathError> {
    let (fact, readout) = split_once_bytes(&document.text, b"\n\n").ok_or_else(|| {
        ParagraphEntitySpinPathError::Invalid(
            "construction document lacks one declared paragraph boundary".to_owned(),
        )
    })?;
    let binding = parse_fact(fact)?;
    let text = std::str::from_utf8(readout).map_err(|_| {
        ParagraphEntitySpinPathError::Invalid("construction readout is not valid UTF-8".to_owned())
    })?;
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    if words.len() != 7
        || words[0] != "For"
        || words[2] != "the"
        || words[3] != "registry"
        || words[4] != "code"
        || words[5] != "is"
        || !words[6].ends_with('.')
        || words[6].len() <= 1
        || words[1].as_bytes() != binding.entity
    {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "construction readout violates the frozen grammar".to_owned(),
        ));
    }
    let candidate = words[6].as_bytes()[..words[6].len() - 1].to_vec();
    validate_word(&candidate, "construction candidate")?;
    let expected = format!(
        "For {} the registry code is {}.",
        std::str::from_utf8(&binding.entity).map_err(|_| {
            ParagraphEntitySpinPathError::Invalid(
                "construction entity is not valid UTF-8".to_owned(),
            )
        })?,
        std::str::from_utf8(&candidate).map_err(|_| {
            ParagraphEntitySpinPathError::Invalid(
                "construction candidate is not valid UTF-8".to_owned(),
            )
        })?
    );
    if readout != expected.as_bytes() {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "construction readout spacing differs from the frozen grammar".to_owned(),
        ));
    }
    Ok(ParsedConstructionRow {
        entity: binding.entity,
        descriptor: binding.descriptor,
        candidate,
    })
}

fn parse_prior_facts(
    prior_paragraph: &[u8],
) -> Result<Vec<EntityBinding>, ParagraphEntitySpinPathError> {
    let text = std::str::from_utf8(prior_paragraph).map_err(|_| {
        ParagraphEntitySpinPathError::Invalid("prior paragraph is not valid UTF-8".to_owned())
    })?;
    let body = text.strip_suffix('.').ok_or_else(|| {
        ParagraphEntitySpinPathError::Invalid("prior paragraph lacks its final period".to_owned())
    })?;
    let parts = body.split(". ").collect::<Vec<_>>();
    if parts.len() != PARAGRAPH_ENTITY_FACTS {
        return Err(ParagraphEntitySpinPathError::Invalid(format!(
            "prior paragraph requires exactly {PARAGRAPH_ENTITY_FACTS} facts"
        )));
    }
    let mut facts = Vec::with_capacity(parts.len());
    for part in parts {
        let mut sentence = part.as_bytes().to_vec();
        sentence.push(b'.');
        facts.push(parse_fact(&sentence)?);
    }
    if facts[0].entity == facts[1].entity || facts[0].descriptor == facts[1].descriptor {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "prior facts require distinct entities and descriptors".to_owned(),
        ));
    }
    Ok(facts)
}

fn parse_fact(sentence: &[u8]) -> Result<EntityBinding, ParagraphEntitySpinPathError> {
    let text = std::str::from_utf8(sentence).map_err(|_| {
        ParagraphEntitySpinPathError::Invalid("fact sentence is not valid UTF-8".to_owned())
    })?;
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    if words.len() != 5 || words[1] != "carried" || words[2] != "the" || words[4] != "marker." {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "fact sentence violates the frozen grammar".to_owned(),
        ));
    }
    let entity = words[0].as_bytes().to_vec();
    let descriptor = words[3].as_bytes().to_vec();
    validate_word(&entity, "entity")?;
    validate_word(&descriptor, "descriptor")?;
    let expected = format!(
        "{} carried the {} marker.",
        std::str::from_utf8(&entity).map_err(|_| {
            ParagraphEntitySpinPathError::Invalid("entity is not valid UTF-8".to_owned())
        })?,
        std::str::from_utf8(&descriptor).map_err(|_| {
            ParagraphEntitySpinPathError::Invalid("descriptor is not valid UTF-8".to_owned())
        })?
    );
    if sentence != expected.as_bytes() {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "fact spacing differs from the frozen grammar".to_owned(),
        ));
    }
    Ok(EntityBinding { entity, descriptor })
}

fn parse_active_query(active_query: &[u8]) -> Result<Vec<u8>, ParagraphEntitySpinPathError> {
    let text = std::str::from_utf8(active_query).map_err(|_| {
        ParagraphEntitySpinPathError::Invalid("active query is not valid UTF-8".to_owned())
    })?;
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    if words.len() != 6
        || words[0] != "For"
        || words[2] != "the"
        || words[3] != "registry"
        || words[4] != "code"
        || words[5] != "is"
    {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "active query violates the frozen grammar".to_owned(),
        ));
    }
    let entity = words[1].as_bytes().to_vec();
    validate_word(&entity, "query entity")?;
    let expected = format!(
        "For {} the registry code is",
        std::str::from_utf8(&entity).map_err(|_| {
            ParagraphEntitySpinPathError::Invalid("query entity is not valid UTF-8".to_owned())
        })?
    );
    if active_query != expected.as_bytes() {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "active-query spacing differs from the frozen grammar".to_owned(),
        ));
    }
    Ok(entity)
}

fn resolve_descriptor(
    facts: &[EntityBinding],
    entity: &[u8],
) -> Result<Vec<u8>, ParagraphEntitySpinPathError> {
    let mut matches = Vec::new();
    for fact in facts {
        if fact.entity == entity {
            matches.push(fact.descriptor.clone());
        }
    }
    if matches.len() != 1 {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "query entity does not resolve exactly one prior fact".to_owned(),
        ));
    }
    Ok(matches.remove(0))
}

fn resolve_other_descriptor(
    facts: &[EntityBinding],
    entity: &[u8],
) -> Result<Vec<u8>, ParagraphEntitySpinPathError> {
    let mut matches = Vec::new();
    for fact in facts {
        if fact.entity != entity {
            matches.push(fact.descriptor.clone());
        }
    }
    if matches.len() != 1 {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "entity permutation does not have exactly one other binding".to_owned(),
        ));
    }
    Ok(matches.remove(0))
}

fn validate_word(bytes: &[u8], field: &str) -> Result<(), ParagraphEntitySpinPathError> {
    if bytes.is_empty()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return Err(ParagraphEntitySpinPathError::Invalid(format!(
            "{field} is not one bounded lexical word"
        )));
    }
    Ok(())
}

fn compile_descriptor_leaves(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
    descriptor: &[u8],
) -> Result<Vec<CompiledSpinLeaf>, ParagraphEntitySpinPathError> {
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

fn compile_leaf(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    table: &H4BinaryIcosahedralClosure,
    surface: &[u8],
) -> Result<CompiledSpinLeaf, ParagraphEntitySpinPathError> {
    let registered = compile_registered_route_value(codec, artifact, surface)?;
    leaf_from_address(surface, &registered.address, registered.witness, table)
}

fn compile_route_value_witness(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    surface: &[u8],
) -> Result<ParagraphEntityRouteValueWitness, ParagraphEntitySpinPathError> {
    Ok(compile_registered_route_value(codec, artifact, surface)?.witness)
}

fn compile_registered_route_value(
    codec: &CanonicalLexicalCodec,
    artifact: &CanonicalRouteArtifact,
    surface: &[u8],
) -> Result<CompiledRouteValue, ParagraphEntitySpinPathError> {
    let encoded = codec.encode(0, 0, surface)?;
    if encoded.units.len() != 1
        || !encoded.units[0].leading_bytes.is_empty()
        || !encoded.trailing_bytes.is_empty()
        || codec.decode(&encoded)? != surface
    {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "path surface is not one canonical lexical unit".to_owned(),
        ));
    }
    let address = artifact
        .lexical_route_address(encoded.units[0].unit_id)?
        .ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "path surface has no exact registered geometric address".to_owned(),
            )
        })?;
    let inverse = artifact
        .lexical_route_value_for_address(&address)?
        .ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "path address has no exact lexical inverse".to_owned(),
            )
        })?;
    if inverse.payload_bytes != surface {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "path address lexical inverse differs from its surface".to_owned(),
        ));
    }
    if inverse.lexical_unit_id != encoded.units[0].unit_id
        || inverse.address_kappa
            != address
                .canonical_kappa()
                .map_err(|error| ParagraphEntitySpinPathError::Invalid(error.to_string()))?
        || inverse.payload_cid != address.payload_cid
    {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "registered route address/value witness does not reproduce".to_owned(),
        ));
    }
    Ok(CompiledRouteValue {
        witness: ParagraphEntityRouteValueWitness {
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
    value: ParagraphEntityRouteValueWitness,
    table: &H4BinaryIcosahedralClosure,
) -> Result<CompiledSpinLeaf, ParagraphEntitySpinPathError> {
    let h4 = exact_s3_spin_to_h4(address.spin.s3.raw(), table)?;
    let mapped_h4_coordinate = h4.root_coordinate(table)?;
    let trace = ParagraphEntitySpinLeafTrace {
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
) -> Result<SpinPathState, ParagraphEntitySpinPathError> {
    if leaves.len() != PARAGRAPH_ENTITY_PATH_LEAVES {
        return Err(ParagraphEntitySpinPathError::Invalid(
            "descriptor path has the wrong leaf count".to_owned(),
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
) -> Result<OrderedH4FoldState, ParagraphEntitySpinPathError> {
    let mut coordinate = [[0_i64; 2]; 4];
    for (target, value) in coordinate.iter_mut().zip(raw) {
        if value & Q29_H4_SCALE_MASK != 0 {
            return Err(ParagraphEntitySpinPathError::Invalid(
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
        let offset =
            u16::try_from(offset).map_err(|_| ParagraphEntitySpinPathError::ArithmeticOverflow)?;
        let index = OpaqueH4TableIndex::from_table_offset(offset, table).ok_or_else(|| {
            ParagraphEntitySpinPathError::Invalid(
                "H4 map addressed outside the exact table".to_owned(),
            )
        })?;
        let state = OrderedH4FoldState::from_table_index(index, table)?;
        if state.root_coordinate(table)? == expected {
            matches.push(state);
        }
    }
    if matches.len() != 1 {
        return Err(ParagraphEntitySpinPathError::Invalid(format!(
            "S3 spin has {} exact H4 coordinate matches",
            matches.len()
        )));
    }
    Ok(matches[0])
}

fn spin_map_kappa(
    table: &H4BinaryIcosahedralClosure,
) -> Result<String, ParagraphEntitySpinPathError> {
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
        .collect::<Result<Vec<_>, ParagraphEntitySpinPathError>>()?;
    let bytes = serde_json::to_vec(&SpinMapWire {
        schema: 1,
        domain: SPIN_MAP_DOMAIN,
        exact_mapping_rule: SPIN_MAP_RULE_IDENTITY,
        h4_root_table_kappa: table.h4_root_table_kappa.clone(),
        rows,
    })
    .map_err(|error| ParagraphEntitySpinPathError::Serialization(error.to_string()))?;
    Ok(blake3_label(&bytes))
}

fn decision(
    arm: ParagraphEntitySpinPathArm,
    token: u32,
    unique_minimum: Option<u32>,
    minimum_cost: Option<ParagraphEntitySpinPathCost>,
    support_tokens: Vec<u32>,
    work: ParagraphEntitySpinPathWork,
) -> ParagraphEntitySpinPathDecision {
    ParagraphEntitySpinPathDecision {
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

    fn finish(self, table: &SourceFreeTable) -> Result<Continuation, ParagraphEntitySpinPathError> {
        Ok(Continuation {
            decoded: table.decode_tokens(&self.generated)?,
            tokens: self.generated,
            stop: self.stop,
        })
    }
}
