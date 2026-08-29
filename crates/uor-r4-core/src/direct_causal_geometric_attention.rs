//! Dense, compiler-side reference attention in transported R4 tangent frames.
//!
//! `DirectCausalGeometricAttentionR4V1` deliberately implements the literal
//! one-head causal attention operator before any bounded-state factorization:
//! token-local Q/K/V/O placements are projected into their exact H4 leaf
//! tangent spaces, prior K and V vectors are transported into the current
//! cumulative frame, logits receive a stable causal softmax, and the aggregated
//! value is scored by a candidate-relative O placement over the caller's
//! unchanged admitted support.  Floating point, multiplication, allocation,
//! and softmax are allowed here because this is an offline mechanism-discovery
//! oracle, not the deployed table-native runtime.
//! The causal row contract is inclusive (`i <= t`): Q comes from the observed
//! current token, while each contextual memory row uses K from its predecessor
//! and V from its own observed token, including the current row.
//!
//! The emitted causal logits, normalized weights, and value aggregate are a
//! parity target for the intended `MultiResonanceSieve` replacement.  The
//! deployed goal is band-limited S3/SU(2), or S2 plus fiber/torsion, resonance
//! accumulation; it is not to carry dense softmax into the runtime.
//!
//! The H4 left-quaternion action below is named `H4FrameConnection`.  It is an
//! exact-route-derived orthogonal frame connection; this module does not claim
//! that it is a fitted or uniquely induced Levi-Civita connection.

use serde::Serialize;
use std::collections::BTreeSet;

use crate::bounded_global_exact_spin_attention::ExactSpinState;
use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, H4BinaryIcosahedralClosure,
};
use crate::corpus_induced_spin_placement::{compile_identity_leaves, leaf_for_token};
use crate::geometric_gated_delta_retention::{
    GeometricRetentionConstructionSequence, GeometricRetentionSupportBinding,
};

const ARTIFACT_MAGIC: &[u8; 8] = b"DCGA0001";
const ARTIFACT_SCHEMA: u32 = 1;
const DIMENSION: usize = 4;
const TANGENT_DIMENSION: f64 = 3.0;
const EPSILON: f64 = 1.0e-12;
const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;
const FIXED_EUCLIDEAN_FRAME_BASE: Vector4 = [1.0, 0.0, 0.0, 0.0];

type Vector4 = [f64; DIMENSION];
type Matrix4 = [[f64; DIMENSION]; DIMENSION];

pub const DIRECT_CAUSAL_GEOMETRIC_ATTENTION_POLICY: &str = concat!(
    "schema=1\n",
    "scope=offline-dense-compiler-reference-not-serving-runtime\n",
    "operator=one-head-literal-causal-Q-K-V-O-attention\n",
    "frames=exact-token-local-H4-leaves-plus-cumulative-query-H4-frame\n",
    "geometry-scope=H4-only;paired-H4-E8-hierarchy-input=NOT_RUN\n",
    "connection=H4-left-quaternion-orthogonal-frame-connection\n",
    "connection-not-claimed=Levi-Civita-or-independent-Q29-phase-action\n",
    "tangent=ambient-R4-projection-orthogonal-to-unit-H4-frame-base\n",
    "logits=metric-dot(transported-K,Q)/sqrt(3)/temperature\n",
    "causal-mask=standard-inclusive-i-less-than-or-equal-t\n",
    "weights=stable-prefix-only-softmax\n",
    "planned-lowering=MultiResonanceSieve-band-limited-S3-SU2-or-S2-plus-phases\n",
    "read=sum(weight*transported-V)\n",
    "score=metric-dot(candidate-relative-O,read)\n",
    "support=caller-supplied-unchanged-admitted-support\n",
    "optimizer=deterministic-sorted-construction-local-contrastive-surrogate\n",
    "raw-parameters=unit-normalized-R4-Q-K-V-O;three-effective-dof-per-vector\n",
    "geometric-operator=raw-R4-projected-to-route-dependent-tangent-frames\n",
    "plain-current-operator=raw-R4-projected-to-one-fixed-R4-tangent-frame\n",
    "plain-current-gradient=fixed-frame-tangent-projection-before-unit-R4-retraction\n",
    "controls=full-geometric,plain-euclidean,alternative-connection,",
    "key-tangent-isometry-permuted,order-shuffled,value-permuted,",
    "separately-trained-geometric-seed-disabled,separately-trained-current-token-only\n",
    "not-claimed=geometry-advantage,heldout-language-model,integer-lowering,",
    "bounded-state-runtime,autonomous-generation"
);

/// Frozen definition of the separately trained V3 parameter arms and their
/// deterministic initialization domains.  This identity is bound into the
/// construction population and artifact so an initialization or degree-of-
/// freedom change necessarily creates a new experiment rung.
pub const DIRECT_CAUSAL_GEOMETRIC_ATTENTION_V3_ARM_POLICY: &str = concat!(
    "version=dcga-973-equal-effective-dof-v3\n",
    "all-raw-vectors=normalized-R4-S3-three-effective-dof\n",
    "full-geometric=route-H4-seed-plus-route-dependent-tangent-projection-and-H4-transport\n",
    "plain-euclidean=independent-seed-plus-fixed-e0-tangent-projection-no-transport\n",
    "geometric-seed-disabled=independent-nongeometric-seed-plus-route-tangent-and-H4-transport\n",
    "current-token-only=independent-seed-plus-fixed-e0-tangent-projection-no-prefix-memory\n",
    "fixed-frame-base=(1,0,0,0)\n",
    "seed-domain-full-q=uor-r4.dcga.geom-q/3\n",
    "seed-domain-full-k=uor-r4.dcga.geom-k/3\n",
    "seed-domain-full-v=uor-r4.dcga.geom-v/3\n",
    "seed-domain-full-o=uor-r4.dcga.geom-o/3\n",
    "seed-domain-plain-q=uor-r4.dcga.plain-q/3\n",
    "seed-domain-plain-k=uor-r4.dcga.plain-k/3\n",
    "seed-domain-plain-v=uor-r4.dcga.plain-v/3\n",
    "seed-domain-plain-o=uor-r4.dcga.plain-o/3\n",
    "seed-domain-disabled-q=uor-r4.dcga.seed-disabled-q/3\n",
    "seed-domain-disabled-k=uor-r4.dcga.seed-disabled-k/3\n",
    "seed-domain-disabled-v=uor-r4.dcga.seed-disabled-v/3\n",
    "seed-domain-disabled-o=uor-r4.dcga.seed-disabled-o/3\n",
    "seed-domain-current-q=uor-r4.dcga.current-only-q/3\n",
    "seed-domain-current-k=uor-r4.dcga.current-only-k/3\n",
    "seed-domain-current-v=uor-r4.dcga.current-only-v/3\n",
    "seed-domain-current-o=uor-r4.dcga.current-only-o/3\n"
);

#[derive(Debug, Clone, PartialEq)]
pub enum DirectCausalGeometricAttentionError {
    Invalid(String),
    ExactRoute(String),
    Arithmetic(String),
}

impl std::fmt::Display for DirectCausalGeometricAttentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::ExactRoute(reason) => write!(formatter, "exact route: {reason}"),
            Self::Arithmetic(reason) => write!(formatter, "arithmetic: {reason}"),
        }
    }
}

impl std::error::Error for DirectCausalGeometricAttentionError {}

/// Frozen parameters for the dense compiler-side reference and its local
/// deterministic contrastive optimizer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DirectCausalGeometricAttentionConfig {
    pub epochs: u32,
    pub learning_rate: f64,
    pub temperature: f64,
}

impl Default for DirectCausalGeometricAttentionConfig {
    fn default() -> Self {
        Self {
            epochs: 12,
            learning_rate: 0.08,
            temperature: 0.35,
        }
    }
}

impl DirectCausalGeometricAttentionConfig {
    fn validate(self) -> Result<Self, DirectCausalGeometricAttentionError> {
        if self.epochs == 0 || self.epochs > 1_024 {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention epochs must be in 1..=1024".to_owned(),
            ));
        }
        if !self.learning_rate.is_finite() || self.learning_rate <= 0.0 || self.learning_rate > 1.0
        {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention learning rate must be finite and in (0,1]".to_owned(),
            ));
        }
        if !self.temperature.is_finite() || self.temperature <= 0.0 {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention temperature must be finite and positive".to_owned(),
            ));
        }
        Ok(self)
    }
}

/// Equal-budget mechanism and intervention arms.  Every trained arm uses one
/// unit-normalized raw R4 Q, K, V, and O vector per token: three effective
/// learnable degrees of freedom per vector and exactly one dense causal head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectCausalGeometricAttentionControl {
    FullGeometric,
    PlainEuclidean,
    /// Replace the H4 connection with a deterministic orthonormal
    /// trivialization that still maps source tangent to destination tangent.
    AlternativeConnection,
    /// Apply a deterministic token-conditioned tangent reflection to each K
    /// before its otherwise exact H4 transport.  It preserves norm, tangency,
    /// dimensions, and the work ledger while destroying learned Q/K alignment.
    KeyTangentIsometryPermuted,
    OrderShuffled,
    ValuePermuted,
    /// Disable only the H4-derived Q/K/V/O initialization.  Exact H4 frame
    /// transport remains active; no paired-H4/E8 input exists in this V1.
    GeometricSeedDisabled,
    /// Separately trained, equal-effective-DOF Q/K/V/O arm whose score sees
    /// only the current token and therefore cannot read an earlier binding.
    CurrentTokenOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlacementArm {
    Geometric,
    PlainEuclidean,
    GeometricSeedDisabled,
    CurrentTokenOnly,
}

impl PlacementArm {
    const fn index(self) -> usize {
        match self {
            Self::Geometric => 0,
            Self::PlainEuclidean => 1,
            Self::GeometricSeedDisabled => 2,
            Self::CurrentTokenOnly => 3,
        }
    }
}

fn placement_arm(control: DirectCausalGeometricAttentionControl) -> PlacementArm {
    match control {
        DirectCausalGeometricAttentionControl::PlainEuclidean => PlacementArm::PlainEuclidean,
        DirectCausalGeometricAttentionControl::GeometricSeedDisabled => {
            PlacementArm::GeometricSeedDisabled
        }
        DirectCausalGeometricAttentionControl::CurrentTokenOnly => PlacementArm::CurrentTokenOnly,
        _ => PlacementArm::Geometric,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectAttentionTransportKind {
    H4FrameConnection,
    AlternativeOrthonormalTrivialization,
    EuclideanIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TokenQkvoPlacement {
    query: Vector4,
    key: Vector4,
    value: Vector4,
    output: Vector4,
}

/// Public, inspectable orthogonal transport between two exact H4 frames.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct H4FrameConnection {
    relative_h4_table_offset: u16,
    matrix: Matrix4,
}

impl H4FrameConnection {
    pub const fn relative_h4_table_offset(&self) -> u16 {
        self.relative_h4_table_offset
    }

    pub const fn matrix(&self) -> Matrix4 {
        self.matrix
    }

    pub fn apply(&self, vector: Vector4) -> Vector4 {
        matrix_vector(self.matrix, vector)
    }

    pub fn inverse_apply(&self, vector: Vector4) -> Vector4 {
        matrix_vector(transpose(self.matrix), vector)
    }

    /// Compose numerical representations in path order: `self` after
    /// `earlier`.  Exact route composition is used when constructing each
    /// connection; this helper is exposed for representation audits.
    pub fn after(&self, earlier: &Self) -> Matrix4 {
        matrix_multiply(self.matrix, earlier.matrix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AttentionFrameTransport {
    kind: DirectAttentionTransportKind,
    exact_h4_table_offset: Option<u16>,
    matrix: Matrix4,
}

impl AttentionFrameTransport {
    fn apply(self, vector: Vector4) -> Vector4 {
        matrix_vector(self.matrix, vector)
    }

    fn inverse_apply(self, vector: Vector4) -> Vector4 {
        matrix_vector(transpose(self.matrix), vector)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DirectAttentionPositionTrace {
    pub attended_position: usize,
    pub observed_token: u32,
    pub key_source_token: u32,
    pub value_source_token: u32,
    pub source_h4_table_offset: u16,
    pub connection_h4_table_offset: Option<u16>,
    pub value_connection_h4_table_offset: Option<u16>,
    pub key_transport_kind: DirectAttentionTransportKind,
    pub value_transport_kind: DirectAttentionTransportKind,
    pub key_tangent_isometry_permuted: bool,
    pub attention_logit: f64,
    pub attention_weight: f64,
    pub transported_key_tangent_residual: f64,
    pub transported_value_tangent_residual: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DirectAttentionCandidateScore {
    pub token: u32,
    pub score: f64,
    pub output_connection_h4_table_offset: Option<u16>,
    pub output_transport_kind: DirectAttentionTransportKind,
    pub output_tangent_residual: f64,
}

/// Complete pre-observation ledger for one prediction.  The public API takes
/// a full buffer plus a query index so a test can mutate future suffix bytes;
/// implementation reads token values only through `query_position`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DirectCausalGeometricAttentionTrace {
    pub control: DirectCausalGeometricAttentionControl,
    pub input_position_count: usize,
    pub query_position: usize,
    pub causal_prefix_position_count: usize,
    pub masked_future_position_count: usize,
    pub maximum_position_read: usize,
    pub future_token_reads: u64,
    pub causal_token_value_reads: u64,
    pub query_token: u32,
    pub query_h4_table_offset: u16,
    pub query_tangent_residual: f64,
    pub admitted_support: Vec<u32>,
    pub positions: Vec<DirectAttentionPositionTrace>,
    pub aggregate_value: Vector4,
    pub scores: Vec<DirectAttentionCandidateScore>,
    pub selected_token: u32,
    pub softmax_weight_sum: f64,
    pub q_projections: u64,
    pub k_projections: u64,
    pub v_projections: u64,
    pub o_projections: u64,
    pub key_transports: u64,
    pub value_transports: u64,
    pub output_transports: u64,
    /// Physical f64 slots retained by the offline artifact.
    pub stored_scalar_parameter_count: usize,
    /// Effective learnable degrees of freedom after unit-R4 normalization.
    pub learned_effective_degree_count: usize,
}

#[derive(Debug, Clone)]
struct ForwardPosition {
    position: usize,
    token: u32,
    key_source_token: u32,
    value_source_token: u32,
    key_frame: ExactSpinState,
    key_frame_base: Vector4,
    value_frame_base: Vector4,
    key_connection: AttentionFrameTransport,
    value_connection: AttentionFrameTransport,
    key_current: Vector4,
    value_current: Vector4,
    attention_logit: f64,
    attention_weight: f64,
}

#[derive(Debug, Clone)]
struct ForwardPass {
    trace: DirectCausalGeometricAttentionTrace,
    effective_tokens: Vec<u32>,
    query_source_token: u32,
    query_source_base: Vector4,
    query_connection: AttentionFrameTransport,
    query_current: Vector4,
    outputs: Vec<ForwardOutput>,
    positions: Vec<ForwardPosition>,
}

#[derive(Debug, Clone)]
struct ForwardOutput {
    token: u32,
    frame_base: Vector4,
    connection: AttentionFrameTransport,
    output_current: Vector4,
}

/// Learned dense reference artifact.  It intentionally retains the full
/// prefix at prediction time and is not a candidate serving architecture.
#[derive(Debug, Clone)]
pub struct DirectCausalGeometricAttentionR4V1 {
    maximum_token_id: u32,
    config: DirectCausalGeometricAttentionConfig,
    exact_route_table: H4BinaryIcosahedralClosure,
    exact_route_leaves: Vec<ExactSpinState>,
    geometric_placements: Vec<TokenQkvoPlacement>,
    plain_placements: Vec<TokenQkvoPlacement>,
    seed_disabled_placements: Vec<TokenQkvoPlacement>,
    current_only_placements: Vec<TokenQkvoPlacement>,
    support_binding: GeometricRetentionSupportBinding,
    construction_document_ids: Vec<String>,
    construction_event_count: u64,
    construction_population_kappa: String,
    learning_update_counts: [[u64; 4]; 4],
}

impl DirectCausalGeometricAttentionR4V1 {
    /// Deterministically compile Q/K/V/O placements from construction-only
    /// events.  Documents are sorted by ID and prefix order remains causal.
    pub fn compile(
        maximum_token_id: u32,
        construction_sequences: &[GeometricRetentionConstructionSequence],
        config: DirectCausalGeometricAttentionConfig,
        support_binding: GeometricRetentionSupportBinding,
    ) -> Result<Self, DirectCausalGeometricAttentionError> {
        let config = config.validate()?;
        if construction_sequences.is_empty() {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention construction population is empty".to_owned(),
            ));
        }
        let mut ordered = construction_sequences.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.document_id.cmp(&right.document_id));
        let mut document_ids = BTreeSet::new();
        let mut event_count = 0_u64;
        for sequence in &ordered {
            validate_sequence(sequence, maximum_token_id)?;
            if !document_ids.insert(sequence.document_id.clone()) {
                return Err(DirectCausalGeometricAttentionError::Invalid(format!(
                    "duplicate construction document id {}",
                    sequence.document_id
                )));
            }
            event_count = event_count
                .checked_add(u64::try_from(sequence.steps.len()).map_err(|_| {
                    DirectCausalGeometricAttentionError::Arithmetic(
                        "construction event count does not fit u64".to_owned(),
                    )
                })?)
                .ok_or_else(|| {
                    DirectCausalGeometricAttentionError::Arithmetic(
                        "construction event count overflow".to_owned(),
                    )
                })?;
        }
        if event_count == 0 {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention construction population has no events".to_owned(),
            ));
        }

        let exact_route_table = validate_h4_binary_icosahedral_closure()
            .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))?;
        let exact_route_leaves = compile_identity_leaves(maximum_token_id, &exact_route_table)
            .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))?;
        let mut geometric_placements = Vec::with_capacity(exact_route_leaves.len());
        let mut plain_placements = Vec::with_capacity(exact_route_leaves.len());
        let mut seed_disabled_placements = Vec::with_capacity(exact_route_leaves.len());
        let mut current_only_placements = Vec::with_capacity(exact_route_leaves.len());
        for (token_index, leaf) in exact_route_leaves.iter().copied().enumerate() {
            let token = u32::try_from(token_index).map_err(|_| {
                DirectCausalGeometricAttentionError::Arithmetic(
                    "token placement index does not fit u32".to_owned(),
                )
            })?;
            geometric_placements.push(geometric_seed_placement(token, leaf, &exact_route_table)?);
            plain_placements.push(plain_seed_placement(token));
            seed_disabled_placements.push(seed_disabled_seed_placement(token));
            current_only_placements.push(current_only_seed_placement(token));
        }
        let construction_population_kappa =
            construction_population_kappa(&ordered, &support_binding);
        let mut model = Self {
            maximum_token_id,
            config,
            exact_route_table,
            exact_route_leaves,
            geometric_placements,
            plain_placements,
            seed_disabled_placements,
            current_only_placements,
            support_binding,
            construction_document_ids: document_ids.into_iter().collect(),
            construction_event_count: event_count,
            construction_population_kappa,
            learning_update_counts: [[0; 4]; 4],
        };
        for _epoch in 0..config.epochs {
            for sequence in &ordered {
                model.train_sequence(
                    sequence,
                    DirectCausalGeometricAttentionControl::FullGeometric,
                )?;
                model.train_sequence(
                    sequence,
                    DirectCausalGeometricAttentionControl::PlainEuclidean,
                )?;
                model.train_sequence(
                    sequence,
                    DirectCausalGeometricAttentionControl::GeometricSeedDisabled,
                )?;
                model.train_sequence(
                    sequence,
                    DirectCausalGeometricAttentionControl::CurrentTokenOnly,
                )?;
            }
        }
        model.validate_learned_state()?;
        Ok(model)
    }

    pub const fn maximum_token_id(&self) -> u32 {
        self.maximum_token_id
    }

    pub const fn construction_event_count(&self) -> u64 {
        self.construction_event_count
    }

    pub fn construction_document_ids(&self) -> &[String] {
        &self.construction_document_ids
    }

    pub fn construction_population_kappa(&self) -> &str {
        &self.construction_population_kappa
    }

    pub fn support_binding(&self) -> &GeometricRetentionSupportBinding {
        &self.support_binding
    }

    pub fn policy_identity(&self) -> &'static str {
        DIRECT_CAUSAL_GEOMETRIC_ATTENTION_POLICY
    }

    /// `[Q,K,V,O]` non-zero optimizer steps for full-geometric, plain
    /// Euclidean, geometric-seed-disabled, and current-token-only arms.
    pub const fn learning_update_counts(&self) -> [[u64; 4]; 4] {
        self.learning_update_counts
    }

    pub fn stored_scalar_parameter_count_per_arm(&self) -> usize {
        self.geometric_placements
            .len()
            .saturating_mul(4)
            .saturating_mul(DIMENSION)
    }

    pub fn learned_effective_degree_count_per_arm(&self) -> usize {
        self.geometric_placements
            .len()
            .saturating_mul(4)
            .saturating_mul(TANGENT_DIMENSION as usize)
    }

    /// Predict at one position using only tokens at positions `0..=query`.
    /// Token values in the suffix are neither validated nor read.
    pub fn predict_at(
        &self,
        token_buffer: &[u32],
        query_position: usize,
        admitted_support: &[u32],
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<DirectCausalGeometricAttentionTrace, DirectCausalGeometricAttentionError> {
        Ok(self
            .forward(token_buffer, query_position, admitted_support, control)?
            .trace)
    }

    /// Convenience wrapper when the caller already owns exactly the causal
    /// prefix and wants the final prefix position as the query.
    pub fn predict_prefix(
        &self,
        causal_prefix: &[u32],
        admitted_support: &[u32],
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<DirectCausalGeometricAttentionTrace, DirectCausalGeometricAttentionError> {
        let query_position = causal_prefix.len().checked_sub(1).ok_or_else(|| {
            DirectCausalGeometricAttentionError::Invalid(
                "direct attention requires a nonempty causal prefix".to_owned(),
            )
        })?;
        self.predict_at(causal_prefix, query_position, admitted_support, control)
    }

    /// Exact H4-derived representation between two cumulative prefix frames.
    /// This is exposed so composition and norm preservation can be audited
    /// independently of attention scores.
    pub fn h4_connection_between_prefix_positions(
        &self,
        causal_prefix: &[u32],
        source_position: usize,
        destination_position: usize,
    ) -> Result<H4FrameConnection, DirectCausalGeometricAttentionError> {
        if causal_prefix.is_empty() {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "connection audit requires a nonempty prefix".to_owned(),
            ));
        }
        validate_prefix(causal_prefix, self.maximum_token_id)?;
        let frames = self.cumulative_frames(
            causal_prefix,
            DirectCausalGeometricAttentionControl::FullGeometric,
        )?;
        let source = *frames.get(source_position).ok_or_else(|| {
            DirectCausalGeometricAttentionError::Invalid(
                "source prefix position is outside the causal prefix".to_owned(),
            )
        })?;
        let destination = *frames.get(destination_position).ok_or_else(|| {
            DirectCausalGeometricAttentionError::Invalid(
                "destination prefix position is outside the causal prefix".to_owned(),
            )
        })?;
        self.connection(destination, source)
    }

    /// Canonical compiler artifact bytes.  Every float is bound by its IEEE
    /// bit pattern in ascending token order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(ARTIFACT_MAGIC);
        bytes.extend_from_slice(&ARTIFACT_SCHEMA.to_le_bytes());
        push_bytes(
            &mut bytes,
            DIRECT_CAUSAL_GEOMETRIC_ATTENTION_POLICY.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            DIRECT_CAUSAL_GEOMETRIC_ATTENTION_V3_ARM_POLICY.as_bytes(),
        );
        bytes.extend_from_slice(&self.maximum_token_id.to_le_bytes());
        bytes.extend_from_slice(&self.config.epochs.to_le_bytes());
        bytes.extend_from_slice(&self.config.learning_rate.to_bits().to_le_bytes());
        bytes.extend_from_slice(&self.config.temperature.to_bits().to_le_bytes());
        push_bytes(
            &mut bytes,
            self.exact_route_table.h4_root_table_kappa.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.exact_route_table.multiplication_table_kappa.as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.support_binding.table_artifact_cid().as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.support_binding.overlay_artifact_cid().as_bytes(),
        );
        push_bytes(
            &mut bytes,
            self.support_binding
                .construction_partition_identity()
                .as_bytes(),
        );
        push_bytes(&mut bytes, self.construction_population_kappa.as_bytes());
        bytes.extend_from_slice(&self.construction_event_count.to_le_bytes());
        for arm in self.learning_update_counts {
            for count in arm {
                bytes.extend_from_slice(&count.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(self.construction_document_ids.len() as u64).to_le_bytes());
        for document_id in &self.construction_document_ids {
            push_bytes(&mut bytes, document_id.as_bytes());
        }
        bytes.extend_from_slice(&(self.geometric_placements.len() as u64).to_le_bytes());
        for (((geometric, plain), seed_disabled), current_only) in self
            .geometric_placements
            .iter()
            .zip(&self.plain_placements)
            .zip(&self.seed_disabled_placements)
            .zip(&self.current_only_placements)
        {
            push_placement(&mut bytes, *geometric);
            push_placement(&mut bytes, *plain);
            push_placement(&mut bytes, *seed_disabled);
            push_placement(&mut bytes, *current_only);
        }
        bytes
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    fn train_sequence(
        &mut self,
        sequence: &GeometricRetentionConstructionSequence,
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<(), DirectCausalGeometricAttentionError> {
        let mut prefix = vec![sequence.initial_token];
        for step in &sequence.steps {
            if step.admitted_support.len() > 1 {
                let query_position = prefix.len() - 1;
                let forward =
                    self.forward(&prefix, query_position, &step.admitted_support, control)?;
                let negative = forward
                    .trace
                    .scores
                    .iter()
                    .filter(|candidate| candidate.token != step.observed_token)
                    .max_by(|left, right| {
                        left.score
                            .total_cmp(&right.score)
                            .then_with(|| right.token.cmp(&left.token))
                    })
                    .map(|candidate| candidate.token)
                    .ok_or_else(|| {
                        DirectCausalGeometricAttentionError::Invalid(
                            "contrastive attention event has no distractor".to_owned(),
                        )
                    })?;
                self.local_contrastive_update(&forward, step.observed_token, negative, control)?;
            }
            prefix.push(step.observed_token);
        }
        Ok(())
    }

    fn local_contrastive_update(
        &mut self,
        forward: &ForwardPass,
        target: u32,
        negative: u32,
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<(), DirectCausalGeometricAttentionError> {
        let arm = placement_arm(control);
        let target_output = forward
            .outputs
            .iter()
            .find(|output| output.token == target)
            .ok_or_else(|| {
                DirectCausalGeometricAttentionError::Invalid(
                    "target output placement is absent from admitted support".to_owned(),
                )
            })?;
        let negative_output = forward
            .outputs
            .iter()
            .find(|output| output.token == negative)
            .ok_or_else(|| {
                DirectCausalGeometricAttentionError::Invalid(
                    "negative output placement is absent from admitted support".to_owned(),
                )
            })?;
        let output_contrast =
            subtract(target_output.output_current, negative_output.output_current);
        let rate = self.config.learning_rate;

        if arm == PlacementArm::CurrentTokenOnly {
            let feature_gradient = tangent_project(
                FIXED_EUCLIDEAN_FRAME_BASE,
                scale(output_contrast, 1.0 / TANGENT_DIMENSION),
            );
            let query_index = checked_token_index(
                *forward.effective_tokens.last().ok_or_else(|| {
                    DirectCausalGeometricAttentionError::Invalid(
                        "current-token baseline has no current token".to_owned(),
                    )
                })?,
                self.current_only_placements.len(),
            )?;
            let target_index = checked_token_index(target, self.current_only_placements.len())?;
            let negative_index = checked_token_index(negative, self.current_only_placements.len())?;
            let placements = &mut self.current_only_placements;
            placements[query_index].query = normalize(add(
                placements[query_index].query,
                scale(feature_gradient, rate),
            ))?;
            placements[query_index].key = normalize(add(
                placements[query_index].key,
                scale(feature_gradient, rate),
            ))?;
            placements[query_index].value = normalize(add(
                placements[query_index].value,
                scale(feature_gradient, rate),
            ))?;
            let output_gradient =
                tangent_project(FIXED_EUCLIDEAN_FRAME_BASE, forward.trace.aggregate_value);
            placements[target_index].output = normalize(add(
                placements[target_index].output,
                scale(output_gradient, rate),
            ))?;
            placements[negative_index].output = normalize(subtract(
                placements[negative_index].output,
                scale(output_gradient, rate),
            ))?;
            for component in 0..4 {
                self.learning_update_counts[arm.index()][component] = self.learning_update_counts
                    [arm.index()][component]
                    .checked_add(1)
                    .ok_or_else(update_count_overflow)?;
            }
            return Ok(());
        }

        let mut query_gradient = zero_vector();
        let mut token_key_gradients = vec![zero_vector(); self.exact_route_leaves.len()];
        let mut token_value_gradients = vec![zero_vector(); self.exact_route_leaves.len()];

        for position in &forward.positions {
            let value_centered = subtract(position.value_current, forward.trace.aggregate_value);
            let logit_gradient = position.attention_weight * dot(output_contrast, value_centered);
            query_gradient = add(
                query_gradient,
                scale(
                    position.key_current,
                    logit_gradient / (TANGENT_DIMENSION.sqrt() * self.config.temperature),
                ),
            );
            let key_gradient_current = scale(
                forward.query_current,
                logit_gradient / (TANGENT_DIMENSION.sqrt() * self.config.temperature),
            );
            let value_gradient_current = scale(output_contrast, position.attention_weight);
            let key_gradient_source = position.key_connection.inverse_apply(key_gradient_current);
            let value_gradient_source = position
                .value_connection
                .inverse_apply(value_gradient_current);
            let key_gradient_raw = tangent_project(position.key_frame_base, key_gradient_source);
            let value_gradient_raw =
                tangent_project(position.value_frame_base, value_gradient_source);
            let key_index =
                checked_token_index(position.key_source_token, token_key_gradients.len())?;
            let value_index =
                checked_token_index(position.value_source_token, token_value_gradients.len())?;
            token_key_gradients[key_index] = add(token_key_gradients[key_index], key_gradient_raw);
            token_value_gradients[value_index] =
                add(token_value_gradients[value_index], value_gradient_raw);
        }

        let query_gradient_raw = tangent_project(
            forward.query_source_base,
            forward.query_connection.inverse_apply(query_gradient),
        );
        let target_output_gradient_current = forward.trace.aggregate_value;
        let negative_output_gradient_current = scale(target_output_gradient_current, -1.0);
        let target_output_gradient = tangent_project(
            target_output.frame_base,
            target_output
                .connection
                .inverse_apply(target_output_gradient_current),
        );
        let negative_output_gradient = tangent_project(
            negative_output.frame_base,
            negative_output
                .connection
                .inverse_apply(negative_output_gradient_current),
        );

        let arm_index = arm.index();
        let placements = match arm {
            PlacementArm::Geometric => &mut self.geometric_placements,
            PlacementArm::PlainEuclidean => &mut self.plain_placements,
            PlacementArm::GeometricSeedDisabled => &mut self.seed_disabled_placements,
            PlacementArm::CurrentTokenOnly => unreachable!("handled above"),
        };
        let query_index = checked_token_index(forward.query_source_token, placements.len())?;
        if norm(query_gradient_raw) > EPSILON {
            placements[query_index].query = normalize(add(
                placements[query_index].query,
                scale(query_gradient_raw, rate),
            ))?;
            self.learning_update_counts[arm_index][0] = self.learning_update_counts[arm_index][0]
                .checked_add(1)
                .ok_or_else(update_count_overflow)?;
        }
        let mut any_key = false;
        let mut any_value = false;
        for (token_index, (key_gradient, value_gradient)) in token_key_gradients
            .into_iter()
            .zip(token_value_gradients)
            .enumerate()
        {
            if norm(key_gradient) > EPSILON {
                placements[token_index].key =
                    normalize(add(placements[token_index].key, scale(key_gradient, rate)))?;
                any_key = true;
            }
            if norm(value_gradient) > EPSILON {
                placements[token_index].value = normalize(add(
                    placements[token_index].value,
                    scale(value_gradient, rate),
                ))?;
                any_value = true;
            }
        }
        if any_key {
            self.learning_update_counts[arm_index][1] = self.learning_update_counts[arm_index][1]
                .checked_add(1)
                .ok_or_else(update_count_overflow)?;
        }
        if any_value {
            self.learning_update_counts[arm_index][2] = self.learning_update_counts[arm_index][2]
                .checked_add(1)
                .ok_or_else(update_count_overflow)?;
        }
        let target_index = checked_token_index(target, placements.len())?;
        let negative_index = checked_token_index(negative, placements.len())?;
        if norm(target_output_gradient) > EPSILON {
            placements[target_index].output = normalize(add(
                placements[target_index].output,
                scale(target_output_gradient, rate),
            ))?;
            placements[negative_index].output = normalize(add(
                placements[negative_index].output,
                scale(negative_output_gradient, rate),
            ))?;
            self.learning_update_counts[arm_index][3] = self.learning_update_counts[arm_index][3]
                .checked_add(1)
                .ok_or_else(update_count_overflow)?;
        }
        Ok(())
    }

    fn forward(
        &self,
        token_buffer: &[u32],
        query_position: usize,
        admitted_support: &[u32],
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<ForwardPass, DirectCausalGeometricAttentionError> {
        if token_buffer.is_empty() {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct attention requires a nonempty token buffer".to_owned(),
            ));
        }
        if query_position >= token_buffer.len() {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "query position is outside the token buffer".to_owned(),
            ));
        }
        validate_support(admitted_support, self.maximum_token_id)?;
        // This is the causal boundary: do not inspect any token value after
        // query_position.  Only the total suffix length enters the ledger.
        let causal_prefix = &token_buffer[..=query_position];
        let causal_prefix_input_len = causal_prefix.len();
        let arm = placement_arm(control);
        let current_only = arm == PlacementArm::CurrentTokenOnly;
        let mut effective_tokens = if current_only {
            let current = *causal_prefix.last().ok_or_else(|| {
                DirectCausalGeometricAttentionError::Invalid(
                    "current-token baseline requires a current token".to_owned(),
                )
            })?;
            if current > self.maximum_token_id {
                return Err(DirectCausalGeometricAttentionError::Invalid(
                    "current token is outside the fitted namespace".to_owned(),
                ));
            }
            vec![current]
        } else {
            validate_prefix(causal_prefix, self.maximum_token_id)?;
            causal_prefix.to_vec()
        };
        if control == DirectCausalGeometricAttentionControl::OrderShuffled
            && effective_tokens.len() > 2
        {
            let prior_len = effective_tokens.len() - 1;
            effective_tokens[..prior_len].reverse();
        }
        let frames = self.cumulative_frames(&effective_tokens, control)?;
        let query_frame = *frames.last().ok_or_else(|| {
            DirectCausalGeometricAttentionError::Invalid(
                "direct attention has no query frame".to_owned(),
            )
        })?;
        let query_frame_base = self.frame_base(query_frame)?;
        let query_token = *effective_tokens.last().ok_or_else(|| {
            DirectCausalGeometricAttentionError::Invalid(
                "direct attention has no query token".to_owned(),
            )
        })?;
        let plain_operator = matches!(
            arm,
            PlacementArm::PlainEuclidean | PlacementArm::CurrentTokenOnly
        );
        let query_source_token = query_token;
        let query_leaf = self.route_leaf(query_source_token, control)?;
        let query_route_base = self.frame_base(query_leaf)?;
        let query_source_base = if plain_operator {
            FIXED_EUCLIDEAN_FRAME_BASE
        } else {
            query_route_base
        };
        let query_operator_base = if plain_operator {
            FIXED_EUCLIDEAN_FRAME_BASE
        } else {
            query_frame_base
        };
        let query_placement = self.placement(query_source_token, arm)?;
        let query_connection = self.attention_transport(
            control,
            query_frame,
            query_operator_base,
            query_leaf,
            query_source_base,
        )?;
        let query_source = safe_tangent_project(query_source_base, query_placement.query)?;
        let query_current = query_connection.apply(query_source);

        let permutation_modulus = self.maximum_token_id.checked_add(1).ok_or_else(|| {
            DirectCausalGeometricAttentionError::Arithmetic(
                "value-token permutation modulus overflow".to_owned(),
            )
        })?;
        let position_start = if current_only {
            effective_tokens.len() - 1
        } else {
            0
        };
        // Standard inclusive causal attention: every row i <= t is visible,
        // including the current query row.  Contextual K_i still comes from
        // the causal predecessor while V_i is the observed token at i.
        let position_end = effective_tokens.len();
        let mut positions = Vec::with_capacity(position_end - position_start);
        for position in position_start..position_end {
            let token = effective_tokens[position];
            let key_source_token = if current_only || position == 0 {
                token
            } else {
                effective_tokens[position - 1]
            };
            let key_frame = self.route_leaf(key_source_token, control)?;
            let key_route_base = self.frame_base(key_frame)?;
            let key_frame_base = if plain_operator {
                FIXED_EUCLIDEAN_FRAME_BASE
            } else {
                key_route_base
            };
            let key_placement = self.placement(key_source_token, arm)?;
            let value_source_token =
                if control == DirectCausalGeometricAttentionControl::ValuePermuted {
                    token.checked_add(1).unwrap_or(0) % permutation_modulus
                } else {
                    token
                };
            let value_placement = self.placement(value_source_token, arm)?;
            let value_frame = self.route_leaf(value_source_token, control)?;
            let value_route_base = self.frame_base(value_frame)?;
            let value_frame_base = if plain_operator {
                FIXED_EUCLIDEAN_FRAME_BASE
            } else {
                value_route_base
            };
            let mut key_source = safe_tangent_project(key_frame_base, key_placement.key)?;
            if control == DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted {
                key_source =
                    tangent_isometry_permutation(key_frame_base, key_source, key_source_token)?;
            }
            let value_source = safe_tangent_project(value_frame_base, value_placement.value)?;
            let key_connection = self.attention_transport(
                control,
                query_frame,
                query_operator_base,
                key_frame,
                key_frame_base,
            )?;
            let value_connection = self.attention_transport(
                control,
                query_frame,
                query_operator_base,
                value_frame,
                value_frame_base,
            )?;
            let key_current = key_connection.apply(key_source);
            let value_current = value_connection.apply(value_source);
            require_finite_vector(key_current, "transported attention key")?;
            require_finite_vector(value_current, "transported attention value")?;
            let attention_logit = dot(query_current, key_current)
                / (TANGENT_DIMENSION.sqrt() * self.config.temperature);
            require_finite_scalar(attention_logit, "attention logit")?;
            positions.push(ForwardPosition {
                position,
                token,
                key_source_token,
                value_source_token,
                key_frame,
                key_frame_base,
                value_frame_base,
                key_connection,
                value_connection,
                key_current,
                value_current,
                attention_logit,
                attention_weight: 0.0,
            });
        }
        let logits = positions
            .iter()
            .map(|position| position.attention_logit)
            .collect::<Vec<_>>();
        let weights = stable_softmax(&logits)?;
        let mut aggregate_value = zero_vector();
        for (position, weight) in positions.iter_mut().zip(weights) {
            position.attention_weight = weight;
            aggregate_value = add(aggregate_value, scale(position.value_current, weight));
        }
        if current_only {
            let current = positions.first().ok_or_else(|| {
                DirectCausalGeometricAttentionError::Invalid(
                    "current-token baseline has no current feature".to_owned(),
                )
            })?;
            aggregate_value = scale(
                add(
                    add(query_current, current.key_current),
                    current.value_current,
                ),
                1.0 / TANGENT_DIMENSION,
            );
        }
        require_finite_vector(aggregate_value, "attention value aggregation")?;

        let mut scores = Vec::with_capacity(admitted_support.len());
        let mut outputs = Vec::with_capacity(admitted_support.len());
        let mut selected_token = admitted_support[0];
        let mut selected_score = f64::NEG_INFINITY;
        for &candidate in admitted_support {
            let placement = self.placement(candidate, arm)?;
            let candidate_leaf = self.route_leaf(candidate, control)?;
            let candidate_route_base = self.frame_base(candidate_leaf)?;
            let candidate_frame_base = if plain_operator {
                FIXED_EUCLIDEAN_FRAME_BASE
            } else {
                candidate_route_base
            };
            let output_connection = self.attention_transport(
                control,
                query_frame,
                query_operator_base,
                candidate_leaf,
                candidate_frame_base,
            )?;
            let output_source = safe_tangent_project(candidate_frame_base, placement.output)?;
            let output = output_connection.apply(output_source);
            let score = dot(output, aggregate_value);
            require_finite_scalar(score, "candidate-relative O score")?;
            outputs.push(ForwardOutput {
                token: candidate,
                frame_base: candidate_frame_base,
                connection: output_connection,
                output_current: output,
            });
            scores.push(DirectAttentionCandidateScore {
                token: candidate,
                score,
                output_connection_h4_table_offset: output_connection.exact_h4_table_offset,
                output_transport_kind: output_connection.kind,
                output_tangent_residual: dot(output, query_operator_base).abs(),
            });
            if score > selected_score + EPSILON {
                selected_score = score;
                selected_token = candidate;
            }
        }
        let softmax_weight_sum = positions
            .iter()
            .map(|position| position.attention_weight)
            .sum::<f64>();
        let position_traces = positions
            .iter()
            .map(|position| DirectAttentionPositionTrace {
                attended_position: position.position,
                observed_token: position.token,
                key_source_token: position.key_source_token,
                value_source_token: position.value_source_token,
                source_h4_table_offset: position.key_frame.table_index().table_offset(),
                connection_h4_table_offset: position.key_connection.exact_h4_table_offset,
                value_connection_h4_table_offset: position.value_connection.exact_h4_table_offset,
                key_transport_kind: position.key_connection.kind,
                value_transport_kind: position.value_connection.kind,
                key_tangent_isometry_permuted: control
                    == DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted,
                attention_logit: position.attention_logit,
                attention_weight: position.attention_weight,
                transported_key_tangent_residual: dot(position.key_current, query_operator_base)
                    .abs(),
                transported_value_tangent_residual: dot(
                    position.value_current,
                    query_operator_base,
                )
                .abs(),
            })
            .collect();
        let attended_len_u64 = u64::try_from(positions.len()).map_err(|_| {
            DirectCausalGeometricAttentionError::Arithmetic(
                "attended prefix length does not fit work ledger".to_owned(),
            )
        })?;
        let support_len_u64 = u64::try_from(admitted_support.len()).map_err(|_| {
            DirectCausalGeometricAttentionError::Arithmetic(
                "support length does not fit work ledger".to_owned(),
            )
        })?;
        let trace = DirectCausalGeometricAttentionTrace {
            control,
            input_position_count: token_buffer.len(),
            query_position,
            causal_prefix_position_count: causal_prefix_input_len,
            masked_future_position_count: token_buffer.len() - query_position - 1,
            maximum_position_read: query_position,
            future_token_reads: 0,
            causal_token_value_reads: u64::try_from(effective_tokens.len()).map_err(|_| {
                DirectCausalGeometricAttentionError::Arithmetic(
                    "causal token read count does not fit work ledger".to_owned(),
                )
            })?,
            query_token,
            query_h4_table_offset: query_frame.table_index().table_offset(),
            query_tangent_residual: dot(query_current, query_operator_base).abs(),
            admitted_support: admitted_support.to_vec(),
            positions: position_traces,
            aggregate_value,
            scores,
            selected_token,
            softmax_weight_sum,
            q_projections: 1,
            k_projections: attended_len_u64,
            v_projections: attended_len_u64,
            o_projections: support_len_u64,
            key_transports: if plain_operator { 0 } else { attended_len_u64 },
            value_transports: if plain_operator { 0 } else { attended_len_u64 },
            output_transports: if plain_operator { 0 } else { support_len_u64 },
            stored_scalar_parameter_count: self.stored_scalar_parameter_count_per_arm(),
            learned_effective_degree_count: self.learned_effective_degree_count_per_arm(),
        };
        Ok(ForwardPass {
            trace,
            effective_tokens,
            query_source_token,
            query_source_base,
            query_connection,
            query_current,
            outputs,
            positions,
        })
    }

    fn cumulative_frames(
        &self,
        tokens: &[u32],
        control: DirectCausalGeometricAttentionControl,
    ) -> Result<Vec<ExactSpinState>, DirectCausalGeometricAttentionError> {
        let mut frame = ExactSpinState::identity(&self.exact_route_table)
            .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))?;
        let mut frames = Vec::with_capacity(tokens.len());
        for &token in tokens {
            let leaf = self.route_leaf(token, control)?;
            frame = frame
                .compose(leaf, &self.exact_route_table)
                .map_err(|error| {
                    DirectCausalGeometricAttentionError::ExactRoute(error.to_string())
                })?;
            frames.push(frame);
        }
        Ok(frames)
    }

    fn connection(
        &self,
        destination: ExactSpinState,
        source: ExactSpinState,
    ) -> Result<H4FrameConnection, DirectCausalGeometricAttentionError> {
        let relative = source
            .inverse(&self.exact_route_table)
            .and_then(|inverse| destination.compose(inverse, &self.exact_route_table))
            .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))?;
        let matrix = h4_left_quaternion_matrix(relative, &self.exact_route_table)?;
        Ok(H4FrameConnection {
            relative_h4_table_offset: relative.table_index().table_offset(),
            matrix,
        })
    }

    fn frame_base(
        &self,
        frame: ExactSpinState,
    ) -> Result<Vector4, DirectCausalGeometricAttentionError> {
        route_quaternion(frame, &self.exact_route_table)
    }

    fn route_leaf(
        &self,
        token: u32,
        _control: DirectCausalGeometricAttentionControl,
    ) -> Result<ExactSpinState, DirectCausalGeometricAttentionError> {
        leaf_for_token(&self.exact_route_leaves, token)
            .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))
    }

    fn attention_transport(
        &self,
        control: DirectCausalGeometricAttentionControl,
        destination: ExactSpinState,
        destination_base: Vector4,
        source: ExactSpinState,
        source_base: Vector4,
    ) -> Result<AttentionFrameTransport, DirectCausalGeometricAttentionError> {
        if matches!(
            control,
            DirectCausalGeometricAttentionControl::PlainEuclidean
                | DirectCausalGeometricAttentionControl::CurrentTokenOnly
        ) {
            return Ok(AttentionFrameTransport {
                kind: DirectAttentionTransportKind::EuclideanIdentity,
                exact_h4_table_offset: None,
                matrix: identity_matrix(),
            });
        }
        if control == DirectCausalGeometricAttentionControl::AlternativeConnection {
            return Ok(AttentionFrameTransport {
                kind: DirectAttentionTransportKind::AlternativeOrthonormalTrivialization,
                exact_h4_table_offset: None,
                matrix: alternative_frame_connection(destination_base, source_base)?,
            });
        }
        let exact = self.connection(destination, source)?;
        Ok(AttentionFrameTransport {
            kind: DirectAttentionTransportKind::H4FrameConnection,
            exact_h4_table_offset: Some(exact.relative_h4_table_offset),
            matrix: exact.matrix,
        })
    }

    fn placement(
        &self,
        token: u32,
        arm: PlacementArm,
    ) -> Result<TokenQkvoPlacement, DirectCausalGeometricAttentionError> {
        let placements = match arm {
            PlacementArm::Geometric => &self.geometric_placements,
            PlacementArm::PlainEuclidean => &self.plain_placements,
            PlacementArm::GeometricSeedDisabled => &self.seed_disabled_placements,
            PlacementArm::CurrentTokenOnly => &self.current_only_placements,
        };
        let index = checked_token_index(token, placements.len())?;
        Ok(placements[index])
    }

    fn validate_learned_state(&self) -> Result<(), DirectCausalGeometricAttentionError> {
        if self.geometric_placements.len() != self.plain_placements.len()
            || self.geometric_placements.len() != self.seed_disabled_placements.len()
            || self.geometric_placements.len() != self.current_only_placements.len()
            || self.geometric_placements.len() != self.exact_route_leaves.len()
        {
            return Err(DirectCausalGeometricAttentionError::Invalid(
                "direct-attention Q/K/V/O namespaces differ".to_owned(),
            ));
        }
        for placement in self
            .geometric_placements
            .iter()
            .chain(&self.plain_placements)
            .chain(&self.seed_disabled_placements)
            .chain(&self.current_only_placements)
        {
            for vector in [
                placement.query,
                placement.key,
                placement.value,
                placement.output,
            ] {
                require_finite_vector(vector, "learned direct-attention placement")?;
                if (norm(vector) - 1.0).abs() > 1.0e-9 {
                    return Err(DirectCausalGeometricAttentionError::Arithmetic(
                        "learned direct-attention placement is not unit normalized".to_owned(),
                    ));
                }
            }
        }
        for (arm, counts) in [
            ("full geometric", self.learning_update_counts[0]),
            ("plain Euclidean", self.learning_update_counts[1]),
            ("geometric seed disabled", self.learning_update_counts[2]),
            ("current token only", self.learning_update_counts[3]),
        ] {
            if counts.contains(&0) {
                return Err(DirectCausalGeometricAttentionError::Invalid(format!(
                    "{arm} arm did not apply non-zero Q/K/V/O learning updates"
                )));
            }
        }
        Ok(())
    }
}

fn validate_sequence(
    sequence: &GeometricRetentionConstructionSequence,
    maximum_token_id: u32,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if sequence.document_id.is_empty() || sequence.document_id.contains(['\n', '\r', '\0']) {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "construction document id is empty or contains a control character".to_owned(),
        ));
    }
    if sequence.initial_token > maximum_token_id {
        return Err(DirectCausalGeometricAttentionError::Invalid(format!(
            "initial token {} exceeds fitted maximum {}",
            sequence.initial_token, maximum_token_id
        )));
    }
    for step in &sequence.steps {
        validate_support(&step.admitted_support, maximum_token_id)?;
        if step.observed_token > maximum_token_id {
            return Err(DirectCausalGeometricAttentionError::Invalid(format!(
                "observed token {} exceeds fitted maximum {}",
                step.observed_token, maximum_token_id
            )));
        }
        if step
            .admitted_support
            .binary_search(&step.observed_token)
            .is_err()
        {
            return Err(DirectCausalGeometricAttentionError::Invalid(format!(
                "observed target {} is not in its admitted support",
                step.observed_token
            )));
        }
    }
    Ok(())
}

fn validate_prefix(
    prefix: &[u32],
    maximum_token_id: u32,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if prefix.is_empty() {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "direct attention requires a nonempty prefix".to_owned(),
        ));
    }
    if prefix.iter().any(|token| *token > maximum_token_id) {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "causal prefix contains a token outside the fitted namespace".to_owned(),
        ));
    }
    Ok(())
}

fn validate_support(
    support: &[u32],
    maximum_token_id: u32,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if support.is_empty() {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "direct attention requires nonempty admitted support".to_owned(),
        ));
    }
    if support.iter().any(|token| *token > maximum_token_id) {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "admitted support contains a token outside the fitted namespace".to_owned(),
        ));
    }
    if support.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "admitted support must be strictly ascending and duplicate-free".to_owned(),
        ));
    }
    Ok(())
}

fn geometric_seed_placement(
    token: u32,
    leaf: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<TokenQkvoPlacement, DirectCausalGeometricAttentionError> {
    let route = route_quaternion(leaf, table)?;
    let query = normalize(add(
        [-route[1], route[0], route[3], -route[2]],
        scale(
            deterministic_unit_vector(b"uor-r4.dcga.geom-q/3", token),
            0.0625,
        ),
    ))?;
    let key = normalize(add(
        [route[2], -route[3], -route[0], route[1]],
        scale(
            deterministic_unit_vector(b"uor-r4.dcga.geom-k/3", token),
            0.0625,
        ),
    ))?;
    let value = normalize(add(
        [route[3], route[2], -route[1], -route[0]],
        scale(
            deterministic_unit_vector(b"uor-r4.dcga.geom-v/3", token),
            0.0625,
        ),
    ))?;
    let output = normalize(add(
        [-route[2], route[3], route[0], -route[1]],
        scale(
            deterministic_unit_vector(b"uor-r4.dcga.geom-o/3", token),
            0.0625,
        ),
    ))?;
    Ok(TokenQkvoPlacement {
        query,
        key,
        value,
        output,
    })
}

fn plain_seed_placement(token: u32) -> TokenQkvoPlacement {
    TokenQkvoPlacement {
        query: deterministic_unit_vector(b"uor-r4.dcga.plain-q/3", token),
        key: deterministic_unit_vector(b"uor-r4.dcga.plain-k/3", token),
        value: deterministic_unit_vector(b"uor-r4.dcga.plain-v/3", token),
        output: deterministic_unit_vector(b"uor-r4.dcga.plain-o/3", token),
    }
}

fn seed_disabled_seed_placement(token: u32) -> TokenQkvoPlacement {
    TokenQkvoPlacement {
        query: deterministic_unit_vector(b"uor-r4.dcga.seed-disabled-q/3", token),
        key: deterministic_unit_vector(b"uor-r4.dcga.seed-disabled-k/3", token),
        value: deterministic_unit_vector(b"uor-r4.dcga.seed-disabled-v/3", token),
        output: deterministic_unit_vector(b"uor-r4.dcga.seed-disabled-o/3", token),
    }
}

fn current_only_seed_placement(token: u32) -> TokenQkvoPlacement {
    TokenQkvoPlacement {
        query: deterministic_unit_vector(b"uor-r4.dcga.current-only-q/3", token),
        key: deterministic_unit_vector(b"uor-r4.dcga.current-only-k/3", token),
        value: deterministic_unit_vector(b"uor-r4.dcga.current-only-v/3", token),
        output: deterministic_unit_vector(b"uor-r4.dcga.current-only-o/3", token),
    }
}

fn construction_population_kappa(
    sequences: &[&GeometricRetentionConstructionSequence],
    binding: &GeometricRetentionSupportBinding,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.direct-causal-geometric-attention-construction/1\0");
    hasher.update(DIRECT_CAUSAL_GEOMETRIC_ATTENTION_POLICY.as_bytes());
    hasher.update(DIRECT_CAUSAL_GEOMETRIC_ATTENTION_V3_ARM_POLICY.as_bytes());
    hash_length_prefixed(&mut hasher, binding.table_artifact_cid().as_bytes());
    hash_length_prefixed(&mut hasher, binding.overlay_artifact_cid().as_bytes());
    hash_length_prefixed(
        &mut hasher,
        binding.construction_partition_identity().as_bytes(),
    );
    hasher.update(&(sequences.len() as u64).to_le_bytes());
    for sequence in sequences {
        hash_length_prefixed(&mut hasher, sequence.document_id.as_bytes());
        hasher.update(&sequence.initial_token.to_le_bytes());
        hasher.update(&(sequence.steps.len() as u64).to_le_bytes());
        for step in &sequence.steps {
            hasher.update(&(step.admitted_support.len() as u64).to_le_bytes());
            for token in &step.admitted_support {
                hasher.update(&token.to_le_bytes());
            }
            hasher.update(&step.observed_token.to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn route_quaternion(
    route: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Vector4, DirectCausalGeometricAttentionError> {
    let coordinate = route
        .root_coordinate(table)
        .map_err(|error| DirectCausalGeometricAttentionError::ExactRoute(error.to_string()))?;
    let mut quaternion = [0.0; DIMENSION];
    for (target, [integer, phi]) in quaternion.iter_mut().zip(coordinate.scaled_zphi_quaternion) {
        *target = (integer as f64 + phi as f64 * GOLDEN_RATIO) * 0.5;
    }
    normalize(quaternion)
}

fn h4_left_quaternion_matrix(
    relative: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Matrix4, DirectCausalGeometricAttentionError> {
    let [w, x, y, z] = route_quaternion(relative, table)?;
    let matrix = [[w, -x, -y, -z], [x, w, -z, y], [y, z, w, -x], [z, -y, x, w]];
    require_finite_matrix(matrix, "H4 frame connection")?;
    let orthogonality = matrix_multiply(matrix, transpose(matrix));
    for (row, values) in orthogonality.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            let expected = if row == column { 1.0 } else { 0.0 };
            if (*value - expected).abs() > 1.0e-9 {
                return Err(DirectCausalGeometricAttentionError::Arithmetic(
                    "H4 frame connection is not numerically orthogonal".to_owned(),
                ));
            }
        }
    }
    Ok(matrix)
}

/// Deterministic flat trivialization control.  Unlike a wrong-source H4
/// product, this is a coherent orthogonal map: it sends the source base to the
/// destination base and therefore maps the complete source tangent space into
/// the destination tangent space while preserving norm and work shape.
fn alternative_frame_connection(
    destination_base: Vector4,
    source_base: Vector4,
) -> Result<Matrix4, DirectCausalGeometricAttentionError> {
    let destination_basis = deterministic_orthonormal_frame(destination_base)?;
    let source_basis = deterministic_orthonormal_frame(source_base)?;
    let matrix = matrix_multiply(destination_basis, transpose(source_basis));
    require_finite_matrix(matrix, "alternative frame connection")?;
    Ok(matrix)
}

/// Columns form one deterministic orthonormal frame with the supplied unit
/// base as column zero and a Gram-Schmidt tangent basis in columns one to three.
fn deterministic_orthonormal_frame(
    base: Vector4,
) -> Result<Matrix4, DirectCausalGeometricAttentionError> {
    let mut columns = [zero_vector(); DIMENSION];
    columns[0] = normalize(base)?;
    let mut count = 1;
    for axis in 0..DIMENSION {
        let mut candidate = zero_vector();
        candidate[axis] = 1.0;
        for existing in columns.iter().take(count) {
            candidate = subtract(candidate, scale(*existing, dot(*existing, candidate)));
        }
        if norm(candidate) > 1.0e-10 {
            columns[count] = normalize(candidate)?;
            count += 1;
            if count == DIMENSION {
                break;
            }
        }
    }
    if count != DIMENSION {
        return Err(DirectCausalGeometricAttentionError::Arithmetic(
            "could not construct deterministic tangent frame".to_owned(),
        ));
    }
    let mut matrix = zero_matrix();
    for (column_index, column) in columns.into_iter().enumerate() {
        for (row_index, value) in column.into_iter().enumerate() {
            matrix[row_index][column_index] = value;
        }
    }
    Ok(matrix)
}

fn tangent_project(base: Vector4, ambient: Vector4) -> Vector4 {
    subtract(ambient, scale(base, dot(base, ambient)))
}

fn tangent_isometry_permutation(
    base: Vector4,
    tangent: Vector4,
    token: u32,
) -> Result<Vector4, DirectCausalGeometricAttentionError> {
    let axis = normalize(safe_tangent_project(
        base,
        deterministic_unit_vector(b"uor-r4.dcga.key-tangent-isometry/1", token),
    )?)?;
    let reflected = subtract(tangent, scale(axis, 2.0 * dot(axis, tangent)));
    require_finite_vector(reflected, "key tangent isometry permutation")?;
    Ok(reflected)
}

fn safe_tangent_project(
    base: Vector4,
    ambient: Vector4,
) -> Result<Vector4, DirectCausalGeometricAttentionError> {
    let tangent = tangent_project(base, ambient);
    require_finite_vector(tangent, "R4 tangent projection")?;
    if norm(tangent) <= EPSILON {
        // Deterministic fallback: choose the ambient basis least aligned with
        // the base, then project it.  This remains a genuine tangent vector.
        let mut basis_index = 0;
        for index in 1..DIMENSION {
            if base[index].abs() < base[basis_index].abs() {
                basis_index = index;
            }
        }
        let mut basis = zero_vector();
        basis[basis_index] = 1.0;
        return normalize(tangent_project(base, basis));
    }
    Ok(tangent)
}

fn stable_softmax(logits: &[f64]) -> Result<Vec<f64>, DirectCausalGeometricAttentionError> {
    if logits.is_empty() {
        return Err(DirectCausalGeometricAttentionError::Invalid(
            "causal softmax requires at least one prefix logit".to_owned(),
        ));
    }
    if logits.iter().any(|value| !value.is_finite()) {
        return Err(DirectCausalGeometricAttentionError::Arithmetic(
            "causal softmax contains a non-finite logit".to_owned(),
        ));
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut exponentials = logits
        .iter()
        .map(|logit| (*logit - maximum).exp())
        .collect::<Vec<_>>();
    let denominator = exponentials.iter().sum::<f64>();
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(DirectCausalGeometricAttentionError::Arithmetic(
            "causal softmax denominator is invalid".to_owned(),
        ));
    }
    for weight in &mut exponentials {
        *weight /= denominator;
    }
    Ok(exponentials)
}

fn deterministic_unit_vector(domain: &[u8], token: u32) -> Vector4 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&token.to_le_bytes());
    let mut reader = hasher.finalize_xof();
    let mut bytes = [0_u8; 32];
    reader.fill(&mut bytes);
    let mut vector = [0.0; DIMENSION];
    for (index, target) in vector.iter_mut().enumerate() {
        let offset = index * 8;
        let raw = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
        let unit = raw as f64 / u64::MAX as f64;
        *target = unit.mul_add(2.0, -1.0);
    }
    normalize(vector).unwrap_or([1.0, 0.0, 0.0, 0.0])
}

fn checked_token_index(
    token: u32,
    placement_len: usize,
) -> Result<usize, DirectCausalGeometricAttentionError> {
    let index = usize::try_from(token).map_err(|_| {
        DirectCausalGeometricAttentionError::Arithmetic(
            "token identifier does not fit platform index".to_owned(),
        )
    })?;
    if index >= placement_len {
        return Err(DirectCausalGeometricAttentionError::Invalid(format!(
            "token {token} is outside the direct-attention namespace"
        )));
    }
    Ok(index)
}

fn update_count_overflow() -> DirectCausalGeometricAttentionError {
    DirectCausalGeometricAttentionError::Arithmetic(
        "direct-attention learning update count overflow".to_owned(),
    )
}

fn zero_vector() -> Vector4 {
    [0.0; DIMENSION]
}

fn zero_matrix() -> Matrix4 {
    [[0.0; DIMENSION]; DIMENSION]
}

fn identity_matrix() -> Matrix4 {
    let mut matrix = zero_matrix();
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    matrix
}

fn add(left: Vector4, right: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = left[index] + right[index];
    }
    result
}

fn subtract(left: Vector4, right: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = left[index] - right[index];
    }
    result
}

fn scale(vector: Vector4, scalar: f64) -> Vector4 {
    let mut result = zero_vector();
    for index in 0..DIMENSION {
        result[index] = vector[index] * scalar;
    }
    result
}

fn dot(left: Vector4, right: Vector4) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn norm(vector: Vector4) -> f64 {
    dot(vector, vector).sqrt()
}

fn normalize(vector: Vector4) -> Result<Vector4, DirectCausalGeometricAttentionError> {
    require_finite_vector(vector, "vector normalization input")?;
    let magnitude = norm(vector);
    if !magnitude.is_finite() || magnitude <= f64::EPSILON {
        return Err(DirectCausalGeometricAttentionError::Arithmetic(
            "cannot normalize a zero or non-finite vector".to_owned(),
        ));
    }
    Ok(scale(vector, magnitude.recip()))
}

fn matrix_vector(matrix: Matrix4, vector: Vector4) -> Vector4 {
    let mut result = zero_vector();
    for row in 0..DIMENSION {
        result[row] = dot(matrix[row], vector);
    }
    result
}

fn transpose(matrix: Matrix4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            result[row][column] = matrix[column][row];
        }
    }
    result
}

fn matrix_multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut result = zero_matrix();
    for row in 0..DIMENSION {
        for column in 0..DIMENSION {
            for inner in 0..DIMENSION {
                result[row][column] += left[row][inner] * right[inner][column];
            }
        }
    }
    result
}

fn require_finite_scalar(
    value: f64,
    label: &str,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(DirectCausalGeometricAttentionError::Arithmetic(format!(
            "{label} is non-finite"
        )))
    }
}

fn require_finite_vector(
    vector: Vector4,
    label: &str,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if vector.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(DirectCausalGeometricAttentionError::Arithmetic(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn require_finite_matrix(
    matrix: Matrix4,
    label: &str,
) -> Result<(), DirectCausalGeometricAttentionError> {
    if matrix.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(DirectCausalGeometricAttentionError::Arithmetic(format!(
            "{label} contains a non-finite value"
        )))
    }
}

fn hash_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn push_bytes(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}

fn push_placement(target: &mut Vec<u8>, placement: TokenQkvoPlacement) {
    for vector in [
        placement.query,
        placement.key,
        placement.value,
        placement.output,
    ] {
        for value in vector {
            target.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_gated_delta_retention::GeometricRetentionConstructionStep;

    fn fixture_binding() -> GeometricRetentionSupportBinding {
        GeometricRetentionSupportBinding::new(
            format!("blake3:{}", blake3::hash(b"dcga-table").to_hex()),
            format!("blake3:{}", blake3::hash(b"dcga-overlay").to_hex()),
            "dcga-unit-construction/1",
        )
        .expect("binding")
    }

    fn fixture_sequences() -> Vec<GeometricRetentionConstructionSequence> {
        let document = |document_id: &str, left_value: u32, right_value: u32| {
            GeometricRetentionConstructionSequence {
                document_id: document_id.to_owned(),
                initial_token: 1,
                steps: vec![
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![left_value],
                        observed_token: left_value,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![2],
                        observed_token: 2,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![right_value],
                        observed_token: right_value,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![9],
                        observed_token: 9,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![1],
                        observed_token: 1,
                    },
                    GeometricRetentionConstructionStep {
                        admitted_support: vec![5, 6],
                        observed_token: left_value,
                    },
                ],
            }
        };
        vec![
            document("unit-construction-a", 5, 6),
            document("unit-construction-b", 6, 5),
        ]
    }

    fn fixture_model() -> DirectCausalGeometricAttentionR4V1 {
        DirectCausalGeometricAttentionR4V1::compile(
            12,
            &fixture_sequences(),
            DirectCausalGeometricAttentionConfig {
                epochs: 80,
                learning_rate: 0.04,
                temperature: 0.30,
            },
            fixture_binding(),
        )
        .expect("fixture model")
    }

    #[test]
    fn causal_mask_never_reads_a_future_token_value() {
        let model = fixture_model();
        let left = model
            .predict_at(
                &[1, 5, 2, 6, 9, 1, 7, 8],
                5,
                &[5, 6],
                DirectCausalGeometricAttentionControl::FullGeometric,
            )
            .expect("left");
        let right = model
            .predict_at(
                &[1, 5, 2, 6, 9, 1, u32::MAX, u32::MAX],
                5,
                &[5, 6],
                DirectCausalGeometricAttentionControl::FullGeometric,
            )
            .expect("right");
        assert_eq!(left, right);
        assert_eq!(left.positions.len(), 6);
        assert_eq!(left.maximum_position_read, 5);
        assert_eq!(left.masked_future_position_count, 2);
        assert_eq!(left.future_token_reads, 0);
        assert_eq!(left.query_token, 1);
        let inclusive_query_row = left.positions.last().expect("inclusive query row");
        assert_eq!(inclusive_query_row.attended_position, 5);
        assert_eq!(inclusive_query_row.observed_token, 1);
        assert_eq!(inclusive_query_row.key_source_token, 9);
        assert_eq!(inclusive_query_row.value_source_token, 1);
    }

    #[test]
    fn h4_connection_preserves_norm_tangency_and_composes() {
        let model = fixture_model();
        let prefix = [1, 2, 3, 4];
        let p_ba = model
            .h4_connection_between_prefix_positions(&prefix, 0, 1)
            .expect("P(b,a)");
        let p_cb = model
            .h4_connection_between_prefix_positions(&prefix, 1, 2)
            .expect("P(c,b)");
        let p_ca = model
            .h4_connection_between_prefix_positions(&prefix, 0, 2)
            .expect("P(c,a)");
        let composed = p_cb.after(&p_ba);
        for (composed_row, expected_row) in composed.iter().zip(p_ca.matrix()) {
            for (actual, expected) in composed_row.iter().zip(expected_row) {
                assert!((*actual - expected).abs() <= 1.0e-12);
            }
        }

        let frames = model
            .cumulative_frames(
                &prefix,
                DirectCausalGeometricAttentionControl::FullGeometric,
            )
            .expect("frames");
        let source_base = model.frame_base(frames[0]).expect("source base");
        let destination_base = model.frame_base(frames[2]).expect("destination base");
        let tangent = safe_tangent_project(source_base, [0.3, -0.2, 0.7, 0.5]).expect("tangent");
        let transported = p_ca.apply(tangent);
        assert!((norm(tangent) - norm(transported)).abs() <= 1.0e-12);
        assert!(dot(source_base, tangent).abs() <= 1.0e-12);
        assert!(dot(destination_base, transported).abs() <= 1.0e-12);
    }

    #[test]
    fn value_aggregation_is_load_bearing() {
        let model = fixture_model();
        let baseline = model
            .predict_prefix(
                &[1, 5, 2, 6, 9, 1],
                &[5, 6],
                DirectCausalGeometricAttentionControl::FullGeometric,
            )
            .expect("baseline");
        let mut perturbed = model.clone();
        let token_index =
            checked_token_index(5, perturbed.geometric_placements.len()).expect("token index");
        perturbed.geometric_placements[token_index].value =
            deterministic_unit_vector(b"uor-r4.dcga.value-intervention/2", 5);
        let intervention = perturbed
            .predict_prefix(
                &[1, 5, 2, 6, 9, 1],
                &[5, 6],
                DirectCausalGeometricAttentionControl::FullGeometric,
            )
            .expect("intervention");
        assert_ne!(baseline.aggregate_value, intervention.aggregate_value);
        assert_ne!(baseline.scores, intervention.scores);
    }

    #[test]
    fn compiles_are_deterministic_and_qkvo_all_learn() {
        let left = fixture_model();
        let right = fixture_model();
        assert_eq!(left.to_bytes(), right.to_bytes());
        assert_eq!(left.artifact_cid(), right.artifact_cid());
        for counts in left.learning_update_counts() {
            assert!(counts.iter().all(|count| *count > 0));
        }
    }

    #[test]
    fn every_arm_uses_unit_r4_raw_vectors_with_three_effective_dof() {
        let model = fixture_model();
        assert_eq!(
            model.stored_scalar_parameter_count_per_arm(),
            model.geometric_placements.len() * 4 * DIMENSION
        );
        assert_eq!(
            model.learned_effective_degree_count_per_arm(),
            model.geometric_placements.len() * 4 * TANGENT_DIMENSION as usize
        );
        for placements in [
            &model.geometric_placements,
            &model.plain_placements,
            &model.seed_disabled_placements,
            &model.current_only_placements,
        ] {
            for placement in placements {
                for vector in [
                    placement.query,
                    placement.key,
                    placement.value,
                    placement.output,
                ] {
                    assert!((norm(vector) - 1.0).abs() <= 1.0e-9);
                }
            }
        }
        assert!(model
            .plain_placements
            .iter()
            .chain(&model.current_only_placements)
            .flat_map(|placement| {
                [
                    placement.query,
                    placement.key,
                    placement.value,
                    placement.output,
                ]
            })
            .any(|vector| vector[3].abs() > 1.0e-6));
    }

    #[test]
    fn construction_same_query_dynamic_binding_is_load_bearing() {
        let model = fixture_model();
        let cases = [([1, 5, 2, 6, 9, 1], 5), ([1, 6, 2, 5, 9, 1], 6)];
        for control in [
            DirectCausalGeometricAttentionControl::FullGeometric,
            DirectCausalGeometricAttentionControl::PlainEuclidean,
            DirectCausalGeometricAttentionControl::GeometricSeedDisabled,
            DirectCausalGeometricAttentionControl::AlternativeConnection,
            DirectCausalGeometricAttentionControl::KeyTangentIsometryPermuted,
            DirectCausalGeometricAttentionControl::OrderShuffled,
            DirectCausalGeometricAttentionControl::ValuePermuted,
        ] {
            let correct = cases
                .iter()
                .filter(|(prefix, target)| {
                    model
                        .predict_prefix(prefix, &[5, 6], control)
                        .expect("construction replay")
                        .selected_token
                        == *target
                })
                .count();
            eprintln!("construction {control:?}: {correct}/{}", cases.len());
        }
        let current_only = cases
            .iter()
            .filter(|(prefix, target)| {
                model
                    .predict_prefix(
                        prefix,
                        &[5, 6],
                        DirectCausalGeometricAttentionControl::CurrentTokenOnly,
                    )
                    .expect("current-only replay")
                    .selected_token
                    == *target
            })
            .count();
        assert_eq!(
            current_only, 1,
            "balanced same-query labels cap lookup at chance"
        );
    }
}
