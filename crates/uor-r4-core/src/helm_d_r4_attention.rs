//! HELM-D-derived dense causal attention and UOR R4 gauge transport.
//!
//! This module is the compiler-side softmax oracle for issue #973.  It copies
//! the small semantic core of HELM-D's full attention rather than importing its
//! CUDA/PyTorch training stack, then exposes the exact discrete Spin/H4 frame
//! atlas used to gauge-lift an already learned causal attention head into R4
//! blocks. Vector actions are an offline f64/f32 oracle, not an exact deployed
//! arithmetic claim.
//!
//! The first R4 rung is deliberately a gauge reparameterization.  A model-space
//! vector at position `j` is encoded in its cumulative Spin frame `F_j`, moved
//! into query frame `F_i` with `P_(j->i) = F_i^T F_j`, and decoded after value
//! aggregation.  Therefore coherent transport preserves the ordinary dot
//! product and weighted value sum up to declared floating-point tolerance.  It
//! establishes that causal attention can live in UOR's R4 frames; it does not
//! establish an R4 advantage or a transformerless serving path.

use serde::{Deserialize, Serialize};

use crate::bounded_global_exact_spin_attention::ExactSpinState;
use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, H4BinaryIcosahedralClosure,
};
use crate::corpus_induced_spin_placement::{compile_identity_leaves, leaf_for_token};
use uor_r4_model_source::attention::{
    CausalAttentionHeadContext, CausalAttentionProjectionContext, CausalAttentionSourceContext,
    CausalAttentionTransport,
};

const R4_WIDTH: usize = 4;
const EPSILON: f64 = 1.0e-12;
const GOLDEN_RATIO: f64 = 1.618_033_988_749_895;

pub const HELM_D_UPSTREAM_COMMIT: &str = "7501deca8f413848bfef804be64ce874b72a3cd7";

/// Literal semantic identity of the pinned upstream full-attention row.
pub const HELM_D_DENSE_REFERENCE_POLICY: &str = concat!(
    "upstream=Graph-and-Geometric-Learning/helm@",
    "7501deca8f413848bfef804be64ce874b72a3cd7\n",
    "source=helm/hypercore/nn/attention/lorentz_former_conv.py\n",
    "projection=(sqrt(sum(spatial^2)+c),spatial)\n",
    "logit=2*c+2*c*lorentz-cinner(query,key)\n",
    "scale=divide-by-learned-scale-plus-bias\n",
    "mask=causal-before-softmax\n",
    "selector=stable-ordinary-softmax\n",
    "aggregate=normalized-Lorentz-centroid\n",
    "license=MIT\n",
    "not-claimed=checkpoint-parity,paper-result-inheritance,R4-equivalence"
);

/// UOR's first copy-and-adapt contract.  The intrinsic R4 distance/centroid
/// experiment is a later rung and is intentionally absent here.
pub const HELM_D_R4_GAUGE_SOFTMAX_POLICY: &str = concat!(
    "schema=helm-d-r4-gauge-softmax/1\n",
    "scope=offline-full-prefix-causal-softmax-oracle\n",
    "head-layout=complete-consecutive-R4-blocks\n",
    "frame=exact-cumulative-UOR-Spin-H4-left-quaternion\n",
    "encode=F_position_transpose-times-model-vector\n",
    "transport=P_source_to_query=F_query_transpose-times-F_source\n",
    "transported-state=every-causal-key-and-value\n",
    "score=unchanged-scaled-dot-product-in-query-gauge\n",
    "selector=unchanged-stable-causal-softmax\n",
    "aggregate=unchanged-weighted-value-sum-in-query-gauge\n",
    "decode=F_query-times-query-gauge-output-before-Wo\n",
    "control=source-frame-permuted-with-identical-shape-and-work\n",
    "expected=ordinary-attention-numerical-and-behavioral-parity\n",
    "not-claimed=geometry-advantage,intrinsic-distance,transformerless-serving,",
    "softmax-removal,source-free-language-model"
);

/// Product-Hyperbolic-4 attention over transported four-lane R4 blocks.
///
/// `Lorentz` is the decision arm: every R4 block is the spatial chart of a
/// unit-radius four-dimensional Lorentz manifold. `Flat` has the same
/// parameter shape, causal support, and row-count budget, but uses squared
/// Euclidean distance and an arithmetic weighted centroid. Its cheaper
/// arithmetic is reported separately. It is the curvature-destroying learned
/// control, not a geometric-advantage claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntrinsicR4AttentionMetric {
    Lorentz,
    Flat,
}

/// Equal-arithmetic interventions within one intrinsic R4 metric arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntrinsicR4AttentionIntervention {
    Coherent,
    SourceFramePermuted,
    ValuePermuted,
}

/// Compiler-fitted parameters shared by the curved and flat arms.
///
/// Parameters are laid out in `(layer, head, R4 block)` order. A score row is
/// the sum of each block's non-positive distance feature times its nonnegative
/// coefficient. Each output centroid block is multiplied by its corresponding
/// positive output scale. Keeping the two vectors the same length makes it
/// impossible for the Lorentz arm to obtain extra placement capacity merely
/// by changing metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntrinsicLorentzR4AttentionParameters {
    layers: usize,
    heads: usize,
    blocks_per_head: usize,
    score_coefficients: Vec<f64>,
    output_block_scales: Vec<f64>,
}

impl IntrinsicLorentzR4AttentionParameters {
    pub fn new(
        layers: usize,
        heads: usize,
        blocks_per_head: usize,
        score_coefficients: Vec<f64>,
        output_block_scales: Vec<f64>,
    ) -> Result<Self, HelmDR4AttentionError> {
        let parameters = Self {
            layers,
            heads,
            blocks_per_head,
            score_coefficients,
            output_block_scales,
        };
        parameters.validate()?;
        Ok(parameters)
    }

    pub fn uniform(
        layers: usize,
        heads: usize,
        blocks_per_head: usize,
        score_coefficient: f64,
        output_block_scale: f64,
    ) -> Result<Self, HelmDR4AttentionError> {
        let parameter_count = Self::parameter_count(layers, heads, blocks_per_head)?;
        Self::new(
            layers,
            heads,
            blocks_per_head,
            vec![score_coefficient; parameter_count],
            vec![output_block_scale; parameter_count],
        )
    }

    pub const fn layers(&self) -> usize {
        self.layers
    }

    pub const fn heads(&self) -> usize {
        self.heads
    }

    pub const fn blocks_per_head(&self) -> usize {
        self.blocks_per_head
    }

    pub fn score_coefficients(&self) -> &[f64] {
        &self.score_coefficients
    }

    pub fn output_block_scales(&self) -> &[f64] {
        &self.output_block_scales
    }

    pub fn score_coefficient(
        &self,
        layer: usize,
        head: usize,
        block: usize,
    ) -> Result<f64, HelmDR4AttentionError> {
        self.score_coefficients
            .get(self.index(layer, head, block)?)
            .copied()
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "intrinsic R4 score coefficient is unavailable".to_owned(),
                )
            })
    }

    pub fn output_block_scale(
        &self,
        layer: usize,
        head: usize,
        block: usize,
    ) -> Result<f64, HelmDR4AttentionError> {
        self.output_block_scales
            .get(self.index(layer, head, block)?)
            .copied()
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "intrinsic R4 output block scale is unavailable".to_owned(),
                )
            })
    }

    pub fn validate(&self) -> Result<(), HelmDR4AttentionError> {
        let expected = Self::parameter_count(self.layers, self.heads, self.blocks_per_head)?;
        if self.score_coefficients.len() != expected || self.output_block_scales.len() != expected {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "intrinsic R4 parameter shape mismatch: expected {expected}, score={}, output={}",
                self.score_coefficients.len(),
                self.output_block_scales.len()
            )));
        }
        if self
            .score_coefficients
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(HelmDR4AttentionError::Invalid(
                "intrinsic R4 score coefficients must be finite and nonnegative".to_owned(),
            ));
        }
        if self
            .output_block_scales
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(HelmDR4AttentionError::Invalid(
                "intrinsic R4 output scales must be finite and positive".to_owned(),
            ));
        }
        Ok(())
    }

    fn parameter_count(
        layers: usize,
        heads: usize,
        blocks_per_head: usize,
    ) -> Result<usize, HelmDR4AttentionError> {
        if layers == 0 || heads == 0 || blocks_per_head == 0 {
            return Err(HelmDR4AttentionError::Invalid(
                "intrinsic R4 parameter dimensions must be positive".to_owned(),
            ));
        }
        layers
            .checked_mul(heads)
            .and_then(|count| count.checked_mul(blocks_per_head))
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "intrinsic R4 parameter dimensions overflow usize".to_owned(),
                )
            })
    }

    fn index(
        &self,
        layer: usize,
        head: usize,
        block: usize,
    ) -> Result<usize, HelmDR4AttentionError> {
        if layer >= self.layers || head >= self.heads || block >= self.blocks_per_head {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "intrinsic R4 parameter index ({layer},{head},{block}) exceeds shape ({},{},{})",
                self.layers, self.heads, self.blocks_per_head
            )));
        }
        Ok((layer * self.heads + head) * self.blocks_per_head + block)
    }
}

pub const INTRINSIC_LORENTZ_R4_ATTENTION_POLICY: &str = concat!(
    "schema=intrinsic-lorentz-r4-attention/1\n",
    "manifold=product-of-unit-radius-hyperbolic-4-blocks\n",
    "feature=negative-square-acosh-of-negative-lorentz-inner\n",
    "domain=fail-below-one-minus-1e-12-clamp-only-roundoff-to-one\n",
    "arithmetic=pinned-libm-f64-sqrt-log1p-exp-with-f32-weight-quantization\n",
    "selector=stable-causal-softmax\n",
    "aggregate=normalized-lorentz-barycenter-per-r4-block\n",
    "transport=exact-cumulative-uor-spin-h4-query-frame\n",
    "parameters=nonnegative-score-coefficients-and-positive-output-scales\n",
    "not-claimed=karcher-mean,source-free,softmax-free,transformerless"
);

pub const INTRINSIC_FLAT_R4_ATTENTION_POLICY: &str = concat!(
    "schema=intrinsic-flat-r4-control/1\n",
    "manifold=flat-r4-blocks\n",
    "feature=negative-squared-euclidean-distance\n",
    "arithmetic=ordered-f64-feature-and-pinned-libm-f64-softmax-exp-with-f32-weight-quantization\n",
    "selector=stable-causal-softmax\n",
    "aggregate=arithmetic-weighted-centroid-per-r4-block\n",
    "transport=exact-cumulative-uor-spin-h4-query-frame\n",
    "parameters=same-shape-as-intrinsic-lorentz-r4-attention\n",
    "claim=equal-capacity-curvature-destroying-control"
);

type Vector4 = [f64; R4_WIDTH];
type Matrix4 = [[f64; R4_WIDTH]; R4_WIDTH];

#[derive(Debug, Clone, PartialEq)]
pub enum HelmDR4AttentionError {
    Invalid(String),
    ExactRoute(String),
    Arithmetic(String),
}

impl std::fmt::Display for HelmDR4AttentionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::ExactRoute(reason) => write!(formatter, "exact route: {reason}"),
            Self::Arithmetic(reason) => write!(formatter, "arithmetic: {reason}"),
        }
    }
}

impl std::error::Error for HelmDR4AttentionError {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HelmDLorentzReferenceConfig {
    pub curvature: f64,
    pub learned_scale: f64,
    pub bias: f64,
}

impl Default for HelmDLorentzReferenceConfig {
    fn default() -> Self {
        Self {
            curvature: 1.0,
            learned_scale: 1.0,
            bias: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HelmDLorentzReferenceTrace {
    pub logits: Vec<f64>,
    pub weights: Vec<f64>,
    pub centroid: Vec<f64>,
    pub weight_sum: f64,
    pub lorentz_constraint_residual: f64,
}

/// One causal row of the pinned HELM-D full-attention equation.
///
/// Inputs are the spatial coordinates accepted by upstream `project`; the
/// time-like coordinate is reconstructed exactly as the pinned source does.
pub fn helm_d_lorentz_causal_row(
    query_spatial: &[f64],
    key_spatial: &[Vec<f64>],
    value_spatial: &[Vec<f64>],
    config: HelmDLorentzReferenceConfig,
) -> Result<HelmDLorentzReferenceTrace, HelmDR4AttentionError> {
    if query_spatial.is_empty() {
        return Err(HelmDR4AttentionError::Invalid(
            "HELM-D query must have at least one spatial coordinate".to_owned(),
        ));
    }
    if key_spatial.is_empty() || key_spatial.len() != value_spatial.len() {
        return Err(HelmDR4AttentionError::Invalid(
            "HELM-D causal keys and values must be nonempty and aligned".to_owned(),
        ));
    }
    if !config.curvature.is_finite()
        || config.curvature <= 0.0
        || !config.learned_scale.is_finite()
        || config.learned_scale.abs() <= EPSILON
        || !config.bias.is_finite()
    {
        return Err(HelmDR4AttentionError::Invalid(
            "HELM-D curvature/scale/bias configuration is invalid".to_owned(),
        ));
    }
    let width = query_spatial.len();
    if key_spatial
        .iter()
        .chain(value_spatial)
        .any(|row| row.len() != width || row.iter().any(|value| !value.is_finite()))
        || query_spatial.iter().any(|value| !value.is_finite())
    {
        return Err(HelmDR4AttentionError::Invalid(
            "HELM-D row dimensions differ or contain non-finite values".to_owned(),
        ));
    }

    let query = lorentz_project(query_spatial, config.curvature)?;
    let keys = key_spatial
        .iter()
        .map(|row| lorentz_project(row, config.curvature))
        .collect::<Result<Vec<_>, _>>()?;
    let values = value_spatial
        .iter()
        .map(|row| lorentz_project(row, config.curvature))
        .collect::<Result<Vec<_>, _>>()?;
    let logits = keys
        .iter()
        .map(|key| {
            (2.0 * config.curvature + 2.0 * config.curvature * lorentz_inner(&query, key))
                / config.learned_scale
                + config.bias
        })
        .collect::<Vec<_>>();
    let weights = stable_softmax(&logits)?;
    let mut average = vec![0.0; width + 1];
    for (value, weight) in values.iter().zip(&weights) {
        for (sum, coordinate) in average.iter_mut().zip(value) {
            *sum += *weight * *coordinate;
        }
    }
    let average_inner = lorentz_inner(&average, &average);
    let denominator = (-average_inner).abs().max(EPSILON).sqrt();
    let scale = config.curvature.sqrt() / denominator;
    let centroid = average
        .into_iter()
        .map(|coordinate| coordinate * scale)
        .collect::<Vec<_>>();
    let weight_sum = weights.iter().sum::<f64>();
    let lorentz_constraint_residual =
        (lorentz_inner(&centroid, &centroid) + config.curvature).abs();
    Ok(HelmDLorentzReferenceTrace {
        logits,
        weights,
        centroid,
        weight_sum,
        lorentz_constraint_residual,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum R4SpinTransportIntervention {
    Coherent,
    SourceFramePermuted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SpinTransportAudit {
    pub positions_prepared: u64,
    pub r4_blocks_encoded: u64,
    pub key_blocks_transported: u64,
    pub value_blocks_transported: u64,
    pub output_blocks_decoded: u64,
    pub future_position_reads: u64,
    pub source_frame_permutations: u64,
}

/// Canonical implementation-owned evidence emitted by the R4 transport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct R4SpinTransportEvidence {
    pub schema: String,
    pub policy_identity: String,
    pub intervention: R4SpinTransportIntervention,
    pub frame_table_offsets: Vec<u16>,
    pub audit: R4SpinTransportAudit,
}

/// Exact cumulative Spin/H4 frames for one causal decoder session.
#[derive(Debug, Clone)]
pub struct R4SpinFrameAtlas {
    maximum_token_id: u32,
    exact_route_table: H4BinaryIcosahedralClosure,
    exact_route_leaves: Vec<ExactSpinState>,
    frames: Vec<Option<ExactSpinState>>,
    frame_matrices: Vec<Option<Matrix4>>,
    tokens: Vec<Option<u32>>,
    next_position: usize,
    audit: R4SpinTransportAudit,
}

impl R4SpinFrameAtlas {
    pub fn new(
        maximum_token_id: u32,
        sequence_capacity: usize,
    ) -> Result<Self, HelmDR4AttentionError> {
        if sequence_capacity == 0 {
            return Err(HelmDR4AttentionError::Invalid(
                "R4 attention sequence capacity must be positive".to_owned(),
            ));
        }
        let exact_route_table = validate_h4_binary_icosahedral_closure()
            .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?;
        let exact_route_leaves = compile_identity_leaves(maximum_token_id, &exact_route_table)
            .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?;
        Ok(Self {
            maximum_token_id,
            exact_route_table,
            exact_route_leaves,
            frames: vec![None; sequence_capacity],
            frame_matrices: vec![None; sequence_capacity],
            tokens: vec![None; sequence_capacity],
            next_position: 0,
            audit: R4SpinTransportAudit::default(),
        })
    }

    pub const fn maximum_token_id(&self) -> u32 {
        self.maximum_token_id
    }

    pub fn sequence_capacity(&self) -> usize {
        self.frames.len()
    }

    pub const fn next_position(&self) -> usize {
        self.next_position
    }

    pub const fn audit(&self) -> R4SpinTransportAudit {
        self.audit
    }

    pub fn reset(&mut self) {
        self.frames.fill(None);
        self.frame_matrices.fill(None);
        self.tokens.fill(None);
        self.next_position = 0;
        self.audit = R4SpinTransportAudit::default();
    }

    pub fn begin_position(
        &mut self,
        token: u32,
        position: usize,
    ) -> Result<(), HelmDR4AttentionError> {
        if token > self.maximum_token_id {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "token {token} exceeds R4 attention namespace {}",
                self.maximum_token_id
            )));
        }
        if position != self.next_position || position >= self.frames.len() {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "R4 attention positions must be sequential: expected {}, received {position}",
                self.next_position
            )));
        }
        let leaf = leaf_for_token(&self.exact_route_leaves, token)
            .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?;
        let frame = if position == 0 {
            let identity = ExactSpinState::identity(&self.exact_route_table)
                .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?;
            identity
                .compose(leaf, &self.exact_route_table)
                .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?
        } else {
            let prior = self.frames[position - 1].ok_or_else(|| {
                HelmDR4AttentionError::Invalid("prior R4 attention frame is unavailable".to_owned())
            })?;
            prior
                .compose(leaf, &self.exact_route_table)
                .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?
        };
        let frame_matrix = h4_left_quaternion_matrix(frame, &self.exact_route_table)?;
        self.frames[position] = Some(frame);
        self.frame_matrices[position] = Some(frame_matrix);
        self.tokens[position] = Some(token);
        self.next_position += 1;
        self.audit.positions_prepared = self.audit.positions_prepared.saturating_add(1);
        Ok(())
    }

    pub fn frame_table_offset(&self, position: usize) -> Result<u16, HelmDR4AttentionError> {
        Ok(self.frame(position)?.table_index().table_offset())
    }

    /// Encode a model-space R4 block into the declared position's local gauge.
    pub fn encode_model_block(
        &mut self,
        position: usize,
        model: Vector4,
    ) -> Result<Vector4, HelmDR4AttentionError> {
        let frame = self.frame_matrix(position)?;
        self.audit.r4_blocks_encoded = self.audit.r4_blocks_encoded.saturating_add(1);
        checked_matrix_vector(transpose(frame), model, "R4 gauge encoding")
    }

    /// Move a locally encoded source block into the query position's gauge.
    pub fn transport_local_block(
        &mut self,
        source_position: usize,
        query_position: usize,
        source_local: Vector4,
        intervention: R4SpinTransportIntervention,
        value: bool,
    ) -> Result<Vector4, HelmDR4AttentionError> {
        if source_position > query_position {
            self.audit.future_position_reads = self.audit.future_position_reads.saturating_add(1);
            return Err(HelmDR4AttentionError::Invalid(
                "R4 causal transport cannot read a future position".to_owned(),
            ));
        }
        let source_frame_position = match intervention {
            R4SpinTransportIntervention::Coherent => source_position,
            R4SpinTransportIntervention::SourceFramePermuted => {
                if query_position == 0 {
                    source_position
                } else {
                    (source_position + 1) % (query_position + 1)
                }
            }
        };
        if source_frame_position != source_position {
            self.audit.source_frame_permutations =
                self.audit.source_frame_permutations.saturating_add(1);
        }
        let source_frame = self.frame_matrix(source_frame_position)?;
        let query_frame = self.frame_matrix(query_position)?;
        let connection = matrix_multiply(transpose(query_frame), source_frame);
        if value {
            self.audit.value_blocks_transported =
                self.audit.value_blocks_transported.saturating_add(1);
        } else {
            self.audit.key_blocks_transported = self.audit.key_blocks_transported.saturating_add(1);
        }
        checked_matrix_vector(connection, source_local, "R4 gauge transport")
    }

    /// Decode a query-gauge aggregate back to the learned model basis.
    pub fn decode_query_block(
        &mut self,
        query_position: usize,
        query_local: Vector4,
    ) -> Result<Vector4, HelmDR4AttentionError> {
        let query_frame = self.frame_matrix(query_position)?;
        self.audit.output_blocks_decoded = self.audit.output_blocks_decoded.saturating_add(1);
        checked_matrix_vector(query_frame, query_local, "R4 gauge decoding")
    }

    fn frame(&self, position: usize) -> Result<ExactSpinState, HelmDR4AttentionError> {
        if position >= self.next_position {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "R4 attention frame {position} has not been causally prepared"
            )));
        }
        self.frames
            .get(position)
            .and_then(|frame| *frame)
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(format!(
                    "R4 attention frame {position} is unavailable"
                ))
            })
    }

    fn frame_matrix(&self, position: usize) -> Result<Matrix4, HelmDR4AttentionError> {
        let _ = self.frame(position)?;
        self.frame_matrices
            .get(position)
            .and_then(|matrix| *matrix)
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(format!(
                    "R4 attention frame matrix {position} is unavailable"
                ))
            })
    }
}

/// Object-safe decoder transport implementing the first HELM-D-R4 rung.
///
/// The source decoder retains all learned projections and the softmax/value
/// owners. This type supplies exact cumulative Spin/H4 frame identities and
/// their compiler-side floating-point gauge actions.
#[derive(Debug, Clone)]
pub struct R4SpinCausalAttentionTransport {
    atlas: R4SpinFrameAtlas,
    intervention: R4SpinTransportIntervention,
    fault: Option<String>,
}

impl R4SpinCausalAttentionTransport {
    pub fn new(
        maximum_token_id: u32,
        sequence_capacity: usize,
        intervention: R4SpinTransportIntervention,
    ) -> Result<Self, HelmDR4AttentionError> {
        Ok(Self {
            atlas: R4SpinFrameAtlas::new(maximum_token_id, sequence_capacity)?,
            intervention,
            fault: None,
        })
    }

    pub const fn intervention(&self) -> R4SpinTransportIntervention {
        self.intervention
    }

    pub fn set_intervention(&mut self, intervention: R4SpinTransportIntervention) {
        self.intervention = intervention;
    }

    pub const fn audit(&self) -> R4SpinTransportAudit {
        self.atlas.audit()
    }

    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    pub fn frame_table_offset(&self, position: usize) -> Result<u16, HelmDR4AttentionError> {
        self.atlas.frame_table_offset(position)
    }

    pub fn policy_identity(&self) -> &'static str {
        HELM_D_R4_GAUGE_SOFTMAX_POLICY
    }

    pub fn evidence_snapshot(&self) -> Result<R4SpinTransportEvidence, HelmDR4AttentionError> {
        let frame_table_offsets = (0..self.atlas.next_position())
            .map(|position| self.atlas.frame_table_offset(position))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(R4SpinTransportEvidence {
            schema: "uor-r4.r4-spin-transport-evidence/1".to_owned(),
            policy_identity: HELM_D_R4_GAUGE_SOFTMAX_POLICY.to_owned(),
            intervention: self.intervention,
            frame_table_offsets,
            audit: self.atlas.audit(),
        })
    }

    fn fail_and_copy(&mut self, reason: impl Into<String>, input: &[f32], output: &mut [f32]) {
        if self.fault.is_none() {
            self.fault = Some(reason.into());
        }
        output.fill(0.0);
        for (target, source) in output.iter_mut().zip(input) {
            *target = *source;
        }
    }

    fn valid_block_slices(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        if input.len() == output.len() && !input.is_empty() && input.len().is_multiple_of(R4_WIDTH)
        {
            true
        } else {
            self.fail_and_copy(
                format!(
                    "R4 transport requires equal nonempty head slices in four-lane blocks; input={}, output={}",
                    input.len(),
                    output.len()
                ),
                input,
                output,
            );
            false
        }
    }

    fn read_block(input: &[f32], offset: usize) -> Vector4 {
        [
            f64::from(input[offset]),
            f64::from(input[offset + 1]),
            f64::from(input[offset + 2]),
            f64::from(input[offset + 3]),
        ]
    }

    fn write_block(output: &mut [f32], offset: usize, block: Vector4) -> bool {
        for (target, source) in output[offset..offset + R4_WIDTH].iter_mut().zip(block) {
            *target = source as f32;
            if !target.is_finite() {
                return false;
            }
        }
        true
    }
}

impl CausalAttentionTransport for R4SpinCausalAttentionTransport {
    fn reset(&mut self) {
        self.atlas.reset();
        self.fault = None;
    }

    fn policy_identity(&self) -> &str {
        HELM_D_R4_GAUGE_SOFTMAX_POLICY
    }

    fn implementation_evidence(&self) -> Result<Option<String>, String> {
        self.evidence_snapshot()
            .and_then(|evidence| {
                serde_json::to_string(&evidence)
                    .map_err(|error| HelmDR4AttentionError::Invalid(error.to_string()))
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn status(&self) -> Result<(), String> {
        match &self.fault {
            Some(reason) => Err(reason.clone()),
            None => Ok(()),
        }
    }

    fn begin_position(&mut self, token: usize, position: usize) {
        let result = u32::try_from(token)
            .map_err(|_| {
                HelmDR4AttentionError::Invalid(
                    "decoder token does not fit the UOR u32 namespace".to_owned(),
                )
            })
            .and_then(|token| self.atlas.begin_position(token, position));
        if let Err(error) = result {
            if self.fault.is_none() {
                self.fault = Some(error.to_string());
            }
        }
    }

    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let encoded = self
                .atlas
                .encode_model_block(context.query_position, Self::read_block(input, offset));
            match encoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("R4 query encoding overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        self.intervention,
                        false,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("R4 key transport overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        self.intervention,
                        true,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("R4 value transport overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn output_to_model_frame(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let decoded = self
                .atlas
                .decode_query_block(context.query_position, Self::read_block(input, offset));
            match decoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("R4 output decoding overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }
}

type LorentzVector5 = [f64; R4_WIDTH + 1];

/// Pure per-block compatibility feature used by the deterministic fitter and
/// by [`IntrinsicR4CausalAttentionTransport`]. The returned value is always
/// non-positive: zero means identical points.
pub fn intrinsic_r4_score_feature(
    metric: IntrinsicR4AttentionMetric,
    query: [f64; R4_WIDTH],
    key: [f64; R4_WIDTH],
) -> Result<f64, HelmDR4AttentionError> {
    intrinsic_r4_score_feature_with_clamp(metric, query, key).map(|(feature, _)| feature)
}

/// Pure geometric weighted centroid used by the deterministic fitter and the
/// live intrinsic attention row. The Lorentz arm returns the spatial chart of
/// a normalized future-sheet Lorentz barycenter; it is deliberately not named
/// or claimed as the iterative Karcher/Fréchet mean.
pub fn intrinsic_r4_weighted_centroid(
    metric: IntrinsicR4AttentionMetric,
    values: &[[f64; R4_WIDTH]],
    weights: &[f64],
) -> Result<[f64; R4_WIDTH], HelmDR4AttentionError> {
    if values.is_empty() || values.len() != weights.len() {
        return Err(HelmDR4AttentionError::Invalid(
            "intrinsic R4 centroid values and weights must be nonempty and aligned".to_owned(),
        ));
    }
    if values
        .iter()
        .flatten()
        .chain(weights)
        .any(|value| !value.is_finite())
        || weights.iter().any(|weight| *weight < 0.0)
    {
        return Err(HelmDR4AttentionError::Invalid(
            "intrinsic R4 centroid inputs must be finite with nonnegative weights".to_owned(),
        ));
    }
    let weight_sum = weights.iter().sum::<f64>();
    if !weight_sum.is_finite() || weight_sum <= EPSILON {
        return Err(HelmDR4AttentionError::Arithmetic(
            "intrinsic R4 centroid weight sum is not positive and finite".to_owned(),
        ));
    }

    match metric {
        IntrinsicR4AttentionMetric::Flat => {
            let mut centroid = [0.0; R4_WIDTH];
            for (value, weight) in values.iter().zip(weights) {
                let normalized_weight = *weight / weight_sum;
                for (coordinate, source) in centroid.iter_mut().zip(value) {
                    *coordinate += normalized_weight * *source;
                }
            }
            if centroid.iter().any(|coordinate| !coordinate.is_finite()) {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "flat R4 centroid is non-finite".to_owned(),
                ));
            }
            Ok(centroid)
        }
        IntrinsicR4AttentionMetric::Lorentz => {
            let mut average = [0.0; R4_WIDTH + 1];
            for (value, weight) in values.iter().zip(weights) {
                let projected = intrinsic_lorentz_r4_project(*value)?;
                let normalized_weight = *weight / weight_sum;
                for (coordinate, source) in average.iter_mut().zip(projected) {
                    *coordinate += normalized_weight * source;
                }
            }
            intrinsic_lorentz_r4_normalize_barycenter(average)
        }
    }
}

fn intrinsic_lorentz_r4_project(
    spatial: [f64; R4_WIDTH],
) -> Result<LorentzVector5, HelmDR4AttentionError> {
    if spatial.iter().any(|coordinate| !coordinate.is_finite()) {
        return Err(HelmDR4AttentionError::Invalid(
            "Lorentz R4 spatial coordinates must be finite".to_owned(),
        ));
    }
    let spatial_norm_squared = spatial
        .iter()
        .map(|coordinate| coordinate * coordinate)
        .sum::<f64>();
    let time = libm::sqrt(1.0 + spatial_norm_squared);
    if !time.is_finite() {
        return Err(HelmDR4AttentionError::Arithmetic(
            "Lorentz R4 projection is non-finite".to_owned(),
        ));
    }
    Ok([time, spatial[0], spatial[1], spatial[2], spatial[3]])
}

fn intrinsic_lorentz_r4_normalize_barycenter(
    average: LorentzVector5,
) -> Result<Vector4, HelmDR4AttentionError> {
    let spatial_norm_squared = average[1..]
        .iter()
        .map(|coordinate| coordinate * coordinate)
        .sum::<f64>();
    let timelike_norm_squared = average[0] * average[0] - spatial_norm_squared;
    if !timelike_norm_squared.is_finite() || timelike_norm_squared < EPSILON {
        return Err(HelmDR4AttentionError::Arithmetic(
            "Lorentz R4 barycenter is not future timelike".to_owned(),
        ));
    }
    let normalization = libm::sqrt(timelike_norm_squared).recip();
    let normalized_time = average[0] * normalization;
    let mut centroid = [0.0; R4_WIDTH];
    for (target, source) in centroid.iter_mut().zip(&average[1..]) {
        *target = *source * normalization;
    }
    if !normalized_time.is_finite()
        || normalized_time <= 0.0
        || centroid.iter().any(|coordinate| !coordinate.is_finite())
    {
        return Err(HelmDR4AttentionError::Arithmetic(
            "normalized Lorentz R4 barycenter is invalid".to_owned(),
        ));
    }
    let normalized_spatial_norm_squared = centroid
        .iter()
        .map(|coordinate| coordinate * coordinate)
        .sum::<f64>();
    let residual =
        (-normalized_time * normalized_time + normalized_spatial_norm_squared + 1.0).abs();
    let residual_scale = 1.0 + normalized_time * normalized_time + normalized_spatial_norm_squared;
    if !residual.is_finite() || residual > 1.0e-9 * residual_scale {
        return Err(HelmDR4AttentionError::Arithmetic(format!(
            "normalized Lorentz R4 barycenter residual {residual} exceeds tolerance"
        )));
    }
    Ok(centroid)
}

fn intrinsic_r4_score_feature_with_clamp(
    metric: IntrinsicR4AttentionMetric,
    query: [f64; R4_WIDTH],
    key: [f64; R4_WIDTH],
) -> Result<(f64, bool), HelmDR4AttentionError> {
    if query
        .iter()
        .chain(&key)
        .any(|coordinate| !coordinate.is_finite())
    {
        return Err(HelmDR4AttentionError::Invalid(
            "intrinsic R4 compatibility coordinates must be finite".to_owned(),
        ));
    }
    let (feature, clamped) = match metric {
        IntrinsicR4AttentionMetric::Flat => {
            let squared_distance = query
                .iter()
                .zip(key)
                .map(|(left, right)| {
                    let difference = *left - right;
                    difference * difference
                })
                .sum::<f64>();
            (-squared_distance, false)
        }
        IntrinsicR4AttentionMetric::Lorentz => {
            let query = intrinsic_lorentz_r4_project(query)?;
            let key = intrinsic_lorentz_r4_project(key)?;
            let negative_inner = query[0] * key[0]
                - query[1..]
                    .iter()
                    .zip(&key[1..])
                    .map(|(left, right)| left * right)
                    .sum::<f64>();
            if !negative_inner.is_finite() {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "Lorentz R4 compatibility inner product is non-finite".to_owned(),
                ));
            }
            if negative_inner < 1.0 - 1.0e-12 {
                return Err(HelmDR4AttentionError::Arithmetic(format!(
                    "Lorentz R4 compatibility domain violation: {negative_inner}"
                )));
            }
            let clamped = negative_inner < 1.0;
            let delta = negative_inner.max(1.0) - 1.0;
            // `acosh(1 + delta)` in a form that remains accurate near the
            // identity and avoids squaring a very large `delta` directly.
            let root = libm::sqrt(delta) * libm::sqrt(delta + 2.0);
            let distance = libm::log1p(delta + root);
            (-distance * distance, clamped)
        }
    };
    if !feature.is_finite() || feature > 1.0e-12 {
        return Err(HelmDR4AttentionError::Arithmetic(
            "intrinsic R4 compatibility feature is invalid".to_owned(),
        ));
    }
    Ok((feature.min(0.0), clamped))
}

/// Apply the exact stable-softmax arithmetic used by the live intrinsic R4
/// attention row.
///
/// The input logits are replaced by their shifted `libm::exp` values and the
/// normalized weights are deliberately rounded to `f32`, matching the model
/// attention seam. Compiler-side fitters use this helper so their value
/// aggregation cannot silently optimize a higher-precision selector than the
/// one evaluated by the live decoder.
pub fn intrinsic_stable_softmax_into(
    logits: &mut [f64],
    output_weights: &mut [f32],
) -> Result<(), HelmDR4AttentionError> {
    if logits.is_empty()
        || logits.len() != output_weights.len()
        || logits.iter().any(|value| !value.is_finite())
    {
        return Err(HelmDR4AttentionError::Arithmetic(
            "intrinsic R4 softmax logits are empty, misaligned, or non-finite".to_owned(),
        ));
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    for logit in logits.iter_mut() {
        *logit = libm::exp(*logit - maximum);
    }
    let denominator = logits.iter().sum::<f64>();
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(HelmDR4AttentionError::Arithmetic(
            "intrinsic R4 softmax denominator is invalid".to_owned(),
        ));
    }
    for (output, weight) in output_weights.iter_mut().zip(logits) {
        *output = (*weight / denominator) as f32;
        if !output.is_finite() {
            return Err(HelmDR4AttentionError::Arithmetic(
                "intrinsic R4 softmax overflowed f32".to_owned(),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntrinsicR4AttentionAudit {
    pub score_rows: u64,
    pub compatibility_pairs: u64,
    pub score_blocks: u64,
    pub centroid_rows: u64,
    pub centroid_source_pairs: u64,
    pub centroid_blocks: u64,
    pub lorentz_domain_clamps: u64,
    pub value_permutations: u64,
    pub arithmetic_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntrinsicR4AttentionEvidence {
    pub schema: String,
    pub policy_identity: String,
    pub parameter_identity: String,
    pub metric: IntrinsicR4AttentionMetric,
    pub intervention: IntrinsicR4AttentionIntervention,
    pub frame_table_offsets: Vec<u16>,
    pub transport_audit: R4SpinTransportAudit,
    pub intrinsic_audit: IntrinsicR4AttentionAudit,
}

/// Dense compiler-side intrinsic attention over product-Hyperbolic-4 blocks.
///
/// This implementation deliberately reuses [`R4SpinFrameAtlas`] rather than
/// introducing a second route/frame definition. It remains an O(T^2) floating
/// softmax oracle; recurrence and exact runtime lowering are later decisions.
#[derive(Debug, Clone)]
pub struct IntrinsicR4CausalAttentionTransport {
    atlas: R4SpinFrameAtlas,
    parameters: IntrinsicLorentzR4AttentionParameters,
    metric: IntrinsicR4AttentionMetric,
    intervention: IntrinsicR4AttentionIntervention,
    audit: IntrinsicR4AttentionAudit,
    logit_scratch: Vec<f64>,
    fault: Option<String>,
}

impl IntrinsicR4CausalAttentionTransport {
    pub fn new(
        maximum_token_id: u32,
        sequence_capacity: usize,
        parameters: IntrinsicLorentzR4AttentionParameters,
        metric: IntrinsicR4AttentionMetric,
        intervention: IntrinsicR4AttentionIntervention,
    ) -> Result<Self, HelmDR4AttentionError> {
        parameters.validate()?;
        Ok(Self {
            atlas: R4SpinFrameAtlas::new(maximum_token_id, sequence_capacity)?,
            parameters,
            metric,
            intervention,
            audit: IntrinsicR4AttentionAudit::default(),
            logit_scratch: vec![0.0; sequence_capacity],
            fault: None,
        })
    }

    pub const fn metric(&self) -> IntrinsicR4AttentionMetric {
        self.metric
    }

    pub const fn intervention(&self) -> IntrinsicR4AttentionIntervention {
        self.intervention
    }

    pub fn parameters(&self) -> &IntrinsicLorentzR4AttentionParameters {
        &self.parameters
    }

    pub const fn intrinsic_audit(&self) -> IntrinsicR4AttentionAudit {
        self.audit
    }

    pub const fn transport_audit(&self) -> R4SpinTransportAudit {
        self.atlas.audit()
    }

    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    pub fn policy_identity(&self) -> &'static str {
        match self.metric {
            IntrinsicR4AttentionMetric::Lorentz => INTRINSIC_LORENTZ_R4_ATTENTION_POLICY,
            IntrinsicR4AttentionMetric::Flat => INTRINSIC_FLAT_R4_ATTENTION_POLICY,
        }
    }

    pub fn parameter_identity(&self) -> Result<String, HelmDR4AttentionError> {
        let bytes = serde_json::to_vec(&self.parameters)
            .map_err(|error| HelmDR4AttentionError::Invalid(error.to_string()))?;
        Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
    }

    pub fn evidence_snapshot(&self) -> Result<IntrinsicR4AttentionEvidence, HelmDR4AttentionError> {
        let frame_table_offsets = (0..self.atlas.next_position())
            .map(|position| self.atlas.frame_table_offset(position))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(IntrinsicR4AttentionEvidence {
            schema: "uor-r4.intrinsic-r4-attention-evidence/1".to_owned(),
            policy_identity: self.policy_identity().to_owned(),
            parameter_identity: self.parameter_identity()?,
            metric: self.metric,
            intervention: self.intervention,
            frame_table_offsets,
            transport_audit: self.atlas.audit(),
            intrinsic_audit: self.audit,
        })
    }

    fn transport_intervention(&self) -> R4SpinTransportIntervention {
        match self.intervention {
            IntrinsicR4AttentionIntervention::SourceFramePermuted => {
                R4SpinTransportIntervention::SourceFramePermuted
            }
            IntrinsicR4AttentionIntervention::Coherent
            | IntrinsicR4AttentionIntervention::ValuePermuted => {
                R4SpinTransportIntervention::Coherent
            }
        }
    }

    fn record_fault(&mut self, reason: impl Into<String>) {
        if self.fault.is_none() {
            self.audit.arithmetic_failures = self.audit.arithmetic_failures.saturating_add(1);
            self.fault = Some(reason.into());
        }
    }

    fn fail_and_copy(&mut self, reason: impl Into<String>, input: &[f32], output: &mut [f32]) {
        self.record_fault(reason);
        output.fill(0.0);
        for (target, source) in output.iter_mut().zip(input) {
            *target = *source;
        }
    }

    fn valid_block_slices(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        if input.len() == output.len()
            && !input.is_empty()
            && input.len().is_multiple_of(R4_WIDTH)
            && input.len() / R4_WIDTH == self.parameters.blocks_per_head()
        {
            true
        } else {
            self.fail_and_copy(
                format!(
                    "intrinsic R4 transport requires {} complete four-lane blocks; input={}, output={}",
                    self.parameters.blocks_per_head(),
                    input.len(),
                    output.len()
                ),
                input,
                output,
            );
            false
        }
    }

    fn read_block(input: &[f32], offset: usize) -> Vector4 {
        [
            f64::from(input[offset]),
            f64::from(input[offset + 1]),
            f64::from(input[offset + 2]),
            f64::from(input[offset + 3]),
        ]
    }

    fn write_block(output: &mut [f32], offset: usize, block: Vector4) -> bool {
        for (target, source) in output[offset..offset + R4_WIDTH].iter_mut().zip(block) {
            *target = source as f32;
            if !target.is_finite() {
                return false;
            }
        }
        true
    }

    fn score_row(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
    ) -> Result<(), HelmDR4AttentionError> {
        let head_width = query.len();
        if head_width == 0
            || !head_width.is_multiple_of(R4_WIDTH)
            || head_width / R4_WIDTH != self.parameters.blocks_per_head()
            || output_weights.is_empty()
            || output_weights.len() > self.logit_scratch.len()
            || output_weights.len().checked_mul(head_width) != Some(packed_keys.len())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "intrinsic R4 score row has inconsistent query/key/weight shapes".to_owned(),
            ));
        }
        for (key_index, packed_key) in packed_keys.chunks_exact(head_width).enumerate() {
            let mut logit = 0.0;
            for block in 0..self.parameters.blocks_per_head() {
                let offset = block * R4_WIDTH;
                let (feature, clamped) = intrinsic_r4_score_feature_with_clamp(
                    self.metric,
                    Self::read_block(query, offset),
                    Self::read_block(packed_key, offset),
                )?;
                let coefficient =
                    self.parameters
                        .score_coefficient(context.layer, context.head, block)?;
                logit += coefficient * feature;
                self.audit.score_blocks = self.audit.score_blocks.saturating_add(1);
                if clamped {
                    self.audit.lorentz_domain_clamps =
                        self.audit.lorentz_domain_clamps.saturating_add(1);
                }
            }
            if !logit.is_finite() {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "intrinsic R4 row logit is non-finite".to_owned(),
                ));
            }
            self.logit_scratch[key_index] = logit;
            self.audit.compatibility_pairs = self.audit.compatibility_pairs.saturating_add(1);
        }
        intrinsic_stable_softmax_into(
            &mut self.logit_scratch[..output_weights.len()],
            output_weights,
        )?;
        self.audit.score_rows = self.audit.score_rows.saturating_add(1);
        Ok(())
    }

    fn centroid_row(
        &mut self,
        context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) -> Result<(), HelmDR4AttentionError> {
        let head_width = output.len();
        if head_width == 0
            || !head_width.is_multiple_of(R4_WIDTH)
            || head_width / R4_WIDTH != self.parameters.blocks_per_head()
            || weights.is_empty()
            || weights.len().checked_mul(head_width) != Some(packed_values.len())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "intrinsic R4 centroid row has inconsistent weight/value/output shapes".to_owned(),
            ));
        }
        let mut weight_sum = 0.0;
        for weight in weights {
            let weight = f64::from(*weight);
            if !weight.is_finite() || weight < 0.0 {
                return Err(HelmDR4AttentionError::Invalid(
                    "intrinsic R4 centroid weights must be finite and nonnegative".to_owned(),
                ));
            }
            weight_sum += weight;
        }
        if !weight_sum.is_finite() || weight_sum <= EPSILON {
            return Err(HelmDR4AttentionError::Arithmetic(
                "intrinsic R4 centroid weight sum is not positive and finite".to_owned(),
            ));
        }
        for block in 0..self.parameters.blocks_per_head() {
            let block_offset = block * R4_WIDTH;
            let mut flat_average = [0.0; R4_WIDTH];
            let mut lorentz_average = [0.0; R4_WIDTH + 1];
            for weight_index in 0..weights.len() {
                let value_index = match self.intervention {
                    IntrinsicR4AttentionIntervention::ValuePermuted if weights.len() > 1 => {
                        self.audit.value_permutations =
                            self.audit.value_permutations.saturating_add(1);
                        (weight_index + 1) % weights.len()
                    }
                    _ => weight_index,
                };
                let value =
                    Self::read_block(packed_values, value_index * head_width + block_offset);
                let normalized_weight = f64::from(weights[weight_index]) / weight_sum;
                match self.metric {
                    IntrinsicR4AttentionMetric::Flat => {
                        if value.iter().any(|coordinate| !coordinate.is_finite()) {
                            return Err(HelmDR4AttentionError::Invalid(
                                "flat R4 centroid coordinates must be finite".to_owned(),
                            ));
                        }
                        for (coordinate, source) in flat_average.iter_mut().zip(value) {
                            *coordinate += normalized_weight * source;
                        }
                    }
                    IntrinsicR4AttentionMetric::Lorentz => {
                        let projected = intrinsic_lorentz_r4_project(value)?;
                        for (coordinate, source) in lorentz_average.iter_mut().zip(projected) {
                            *coordinate += normalized_weight * source;
                        }
                    }
                }
            }
            let mut centroid = match self.metric {
                IntrinsicR4AttentionMetric::Flat => {
                    if flat_average
                        .iter()
                        .any(|coordinate| !coordinate.is_finite())
                    {
                        return Err(HelmDR4AttentionError::Arithmetic(
                            "flat R4 centroid is non-finite".to_owned(),
                        ));
                    }
                    flat_average
                }
                IntrinsicR4AttentionMetric::Lorentz => {
                    intrinsic_lorentz_r4_normalize_barycenter(lorentz_average)?
                }
            };
            let scale = self
                .parameters
                .output_block_scale(context.layer, context.head, block)?;
            for coordinate in &mut centroid {
                *coordinate *= scale;
            }
            if !Self::write_block(output, block_offset, centroid) {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "intrinsic R4 centroid overflowed f32".to_owned(),
                ));
            }
            self.audit.centroid_blocks = self.audit.centroid_blocks.saturating_add(1);
        }
        self.audit.centroid_source_pairs = self
            .audit
            .centroid_source_pairs
            .saturating_add(u64::try_from(weights.len()).unwrap_or(u64::MAX));
        self.audit.centroid_rows = self.audit.centroid_rows.saturating_add(1);
        Ok(())
    }
}

impl CausalAttentionTransport for IntrinsicR4CausalAttentionTransport {
    fn reset(&mut self) {
        self.atlas.reset();
        self.audit = IntrinsicR4AttentionAudit::default();
        self.fault = None;
    }

    fn policy_identity(&self) -> &str {
        IntrinsicR4CausalAttentionTransport::policy_identity(self)
    }

    fn implementation_evidence(&self) -> Result<Option<String>, String> {
        self.evidence_snapshot()
            .and_then(|evidence| {
                serde_json::to_string(&evidence)
                    .map_err(|error| HelmDR4AttentionError::Invalid(error.to_string()))
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn status(&self) -> Result<(), String> {
        match &self.fault {
            Some(reason) => Err(reason.clone()),
            None => Ok(()),
        }
    }

    fn begin_position(&mut self, token: usize, position: usize) {
        let result = u32::try_from(token)
            .map_err(|_| {
                HelmDR4AttentionError::Invalid(
                    "decoder token does not fit the UOR u32 namespace".to_owned(),
                )
            })
            .and_then(|token| self.atlas.begin_position(token, position));
        if let Err(error) = result {
            self.record_fault(error.to_string());
        }
    }

    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("intrinsic R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let encoded = self
                .atlas
                .encode_model_block(context.query_position, Self::read_block(input, offset));
            match encoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("intrinsic R4 query encoding overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("intrinsic R4 transport is already faulted", input, output);
            }
            return;
        }
        let intervention = self.transport_intervention();
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        intervention,
                        false,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy("intrinsic R4 key transport overflowed f32", input, output);
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("intrinsic R4 transport is already faulted", input, output);
            }
            return;
        }
        let intervention = self.transport_intervention();
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        intervention,
                        true,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "intrinsic R4 value transport overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn output_to_model_frame(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_block_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy("intrinsic R4 transport is already faulted", input, output);
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let decoded = self
                .atlas
                .decode_query_block(context.query_position, Self::read_block(input, offset));
            match decoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "intrinsic R4 output decoding overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn score_and_normalize(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        _canonical_math: bool,
    ) {
        if self.fault.is_some() {
            output_weights.fill(0.0);
            return;
        }
        if let Err(error) = self.score_row(context, query, packed_keys, output_weights) {
            output_weights.fill(0.0);
            self.record_fault(error.to_string());
        }
    }

    fn weighted_value_centroid(
        &mut self,
        context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() {
            output.fill(0.0);
            return;
        }
        if let Err(error) = self.centroid_row(context, weights, packed_values, output) {
            output.fill(0.0);
            self.record_fault(error.to_string());
        }
    }
}

/// The construction qualifier copied from HELM-D's dense Lorentz attention.
///
/// Both arms own the same learned R4-block affine adapters. `Lorentz` changes
/// only the compatibility relation and value centroid; `Euclidean` is the
/// equal-capacity curvature-destroying control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelmDLearnedManifoldMetric {
    Lorentz,
    Euclidean,
}

/// Equal-shape interventions used to test whether coherent R4 transport and
/// value binding are causally responsible for an observed construction gain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelmDLearnedManifoldIntervention {
    Coherent,
    SourceFramePermuted,
    ValuePermuted,
    OrderKeyShuffled,
}

/// One compact learned affine map over a canonical four-lane R4 block.
///
/// HELM-D upstream uses unrestricted dense Q/K/V maps. This block-diagonal
/// adapter is UOR's explicitly bounded construction parameterization over the
/// donor-projected Q/K/V; it is not a checkpoint-parity claim.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4AffineAdapter {
    pub matrix: Matrix4,
    pub bias: Vector4,
}

impl R4AffineAdapter {
    pub const fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            bias: [0.0; R4_WIDTH],
        }
    }

    pub fn apply(self, input: Vector4) -> Result<Vector4, HelmDR4AttentionError> {
        let mut output = checked_matrix_vector(self.matrix, input, "learned R4 affine adapter")?;
        for (coordinate, bias) in output.iter_mut().zip(self.bias) {
            *coordinate += bias;
        }
        if output.iter().any(|coordinate| !coordinate.is_finite()) {
            return Err(HelmDR4AttentionError::Arithmetic(
                "learned R4 affine adapter produced a non-finite coordinate".to_owned(),
            ));
        }
        Ok(output)
    }

    fn validate(self) -> Result<(), HelmDR4AttentionError> {
        if self
            .matrix
            .iter()
            .flatten()
            .chain(&self.bias)
            .any(|value| !value.is_finite())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned R4 affine adapter parameters must be finite".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Matched learned parameters for the construction-only HELM-D qualifier.
///
/// Query adapters follow query heads; key and value adapters follow the
/// checkpoint's grouped-query K/V heads. Each layer also owns the one positive
/// scale and uniform bias appearing in the pinned upstream attention logit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelmDLearnedManifoldParameters {
    layers: usize,
    query_heads: usize,
    key_value_heads: usize,
    blocks_per_head: usize,
    query_adapters: Vec<R4AffineAdapter>,
    key_adapters: Vec<R4AffineAdapter>,
    value_adapters: Vec<R4AffineAdapter>,
    learned_scales: Vec<f64>,
    learned_biases: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
enum LearnedProjectionRole {
    Query,
    Key,
    Value,
}

impl HelmDLearnedManifoldParameters {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layers: usize,
        query_heads: usize,
        key_value_heads: usize,
        blocks_per_head: usize,
        query_adapters: Vec<R4AffineAdapter>,
        key_adapters: Vec<R4AffineAdapter>,
        value_adapters: Vec<R4AffineAdapter>,
        learned_scales: Vec<f64>,
        learned_biases: Vec<f64>,
    ) -> Result<Self, HelmDR4AttentionError> {
        let parameters = Self {
            layers,
            query_heads,
            key_value_heads,
            blocks_per_head,
            query_adapters,
            key_adapters,
            value_adapters,
            learned_scales,
            learned_biases,
        };
        parameters.validate()?;
        Ok(parameters)
    }

    pub fn identity(
        layers: usize,
        query_heads: usize,
        key_value_heads: usize,
        blocks_per_head: usize,
        initial_scale: f64,
    ) -> Result<Self, HelmDR4AttentionError> {
        let query_count = Self::adapter_count(layers, query_heads, blocks_per_head)?;
        let key_value_count = Self::adapter_count(layers, key_value_heads, blocks_per_head)?;
        Self::new(
            layers,
            query_heads,
            key_value_heads,
            blocks_per_head,
            vec![R4AffineAdapter::identity(); query_count],
            vec![R4AffineAdapter::identity(); key_value_count],
            vec![R4AffineAdapter::identity(); key_value_count],
            vec![initial_scale; layers],
            vec![0.0; layers],
        )
    }

    pub const fn layers(&self) -> usize {
        self.layers
    }

    pub const fn query_heads(&self) -> usize {
        self.query_heads
    }

    pub const fn key_value_heads(&self) -> usize {
        self.key_value_heads
    }

    pub const fn blocks_per_head(&self) -> usize {
        self.blocks_per_head
    }

    pub const fn head_width(&self) -> usize {
        self.blocks_per_head * R4_WIDTH
    }

    /// Total scalar capacity, including each 4x4 matrix, four-vector bias,
    /// and the scale/bias pair per layer.
    pub fn scalar_parameter_count(&self) -> Result<usize, HelmDR4AttentionError> {
        let adapters = self
            .query_adapters
            .len()
            .checked_add(self.key_adapters.len())
            .and_then(|count| count.checked_add(self.value_adapters.len()))
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "learned-manifold adapter count overflows usize".to_owned(),
                )
            })?;
        adapters
            .checked_mul(R4_WIDTH * R4_WIDTH + R4_WIDTH)
            .and_then(|count| count.checked_add(self.layers.checked_mul(2)?))
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "learned-manifold scalar parameter count overflows usize".to_owned(),
                )
            })
    }

    /// Versioned, architecture-independent identity of the ordered learned
    /// parameter stream used by both checkpoints and live operator evidence.
    pub fn parameter_identity(&self) -> Result<String, HelmDR4AttentionError> {
        self.validate()?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4.helm-d-learned-manifold-parameters/2\0");
        for dimension in [
            self.layers,
            self.query_heads,
            self.key_value_heads,
            self.blocks_per_head,
        ] {
            let dimension = u64::try_from(dimension).map_err(|_| {
                HelmDR4AttentionError::Invalid(
                    "learned-manifold parameter dimension exceeds u64".to_owned(),
                )
            })?;
            hasher.update(&dimension.to_le_bytes());
        }
        for (role, adapters) in [
            (b"query".as_slice(), self.query_adapters.as_slice()),
            (b"key".as_slice(), self.key_adapters.as_slice()),
            (b"value".as_slice(), self.value_adapters.as_slice()),
        ] {
            hasher.update(role);
            let count = u64::try_from(adapters.len()).map_err(|_| {
                HelmDR4AttentionError::Invalid(
                    "learned-manifold adapter count exceeds u64".to_owned(),
                )
            })?;
            hasher.update(&count.to_le_bytes());
            for adapter in adapters {
                for value in adapter.matrix.iter().flatten().chain(&adapter.bias) {
                    hasher.update(&value.to_bits().to_le_bytes());
                }
            }
        }
        hasher.update(b"scale");
        for value in &self.learned_scales {
            hasher.update(&value.to_bits().to_le_bytes());
        }
        hasher.update(b"bias");
        for value in &self.learned_biases {
            hasher.update(&value.to_bits().to_le_bytes());
        }
        Ok(format!("blake3:{}", hasher.finalize().to_hex()))
    }

    pub fn query_adapters(&self) -> &[R4AffineAdapter] {
        &self.query_adapters
    }

    pub fn query_adapters_mut(&mut self) -> &mut [R4AffineAdapter] {
        &mut self.query_adapters
    }

    pub fn key_adapters(&self) -> &[R4AffineAdapter] {
        &self.key_adapters
    }

    pub fn key_adapters_mut(&mut self) -> &mut [R4AffineAdapter] {
        &mut self.key_adapters
    }

    pub fn value_adapters(&self) -> &[R4AffineAdapter] {
        &self.value_adapters
    }

    pub fn value_adapters_mut(&mut self) -> &mut [R4AffineAdapter] {
        &mut self.value_adapters
    }

    pub fn learned_scales(&self) -> &[f64] {
        &self.learned_scales
    }

    pub fn learned_scales_mut(&mut self) -> &mut [f64] {
        &mut self.learned_scales
    }

    pub fn learned_biases(&self) -> &[f64] {
        &self.learned_biases
    }

    pub fn learned_biases_mut(&mut self) -> &mut [f64] {
        &mut self.learned_biases
    }

    pub fn learned_scale(&self, layer: usize) -> Result<f64, HelmDR4AttentionError> {
        self.learned_scales.get(layer).copied().ok_or_else(|| {
            HelmDR4AttentionError::Invalid(format!(
                "learned-manifold scale layer {layer} exceeds {} layers",
                self.layers
            ))
        })
    }

    pub fn learned_bias(&self, layer: usize) -> Result<f64, HelmDR4AttentionError> {
        self.learned_biases.get(layer).copied().ok_or_else(|| {
            HelmDR4AttentionError::Invalid(format!(
                "learned-manifold bias layer {layer} exceeds {} layers",
                self.layers
            ))
        })
    }

    pub fn validate(&self) -> Result<(), HelmDR4AttentionError> {
        if self.layers == 0
            || self.query_heads == 0
            || self.key_value_heads == 0
            || self.blocks_per_head == 0
            || !self.query_heads.is_multiple_of(self.key_value_heads)
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold dimensions must be positive with query heads divisible by KV heads"
                    .to_owned(),
            ));
        }
        let expected_query =
            Self::adapter_count(self.layers, self.query_heads, self.blocks_per_head)?;
        let expected_key_value =
            Self::adapter_count(self.layers, self.key_value_heads, self.blocks_per_head)?;
        if self.query_adapters.len() != expected_query
            || self.key_adapters.len() != expected_key_value
            || self.value_adapters.len() != expected_key_value
            || self.learned_scales.len() != self.layers
            || self.learned_biases.len() != self.layers
        {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "learned-manifold parameter shape mismatch: q={} (expected {expected_query}), k={}, v={} (expected {expected_key_value}), scales={}, biases={}, layers={}",
                self.query_adapters.len(),
                self.key_adapters.len(),
                self.value_adapters.len(),
                self.learned_scales.len(),
                self.learned_biases.len(),
                self.layers
            )));
        }
        for adapter in self
            .query_adapters
            .iter()
            .chain(&self.key_adapters)
            .chain(&self.value_adapters)
        {
            adapter.validate()?;
        }
        if self
            .learned_scales
            .iter()
            .any(|scale| !scale.is_finite() || *scale <= 0.0)
            || self.learned_biases.iter().any(|bias| !bias.is_finite())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold scales must be finite and positive and biases finite".to_owned(),
            ));
        }
        Ok(())
    }

    fn adapter_count(
        layers: usize,
        heads: usize,
        blocks_per_head: usize,
    ) -> Result<usize, HelmDR4AttentionError> {
        if layers == 0 || heads == 0 || blocks_per_head == 0 {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold adapter dimensions must be positive".to_owned(),
            ));
        }
        layers
            .checked_mul(heads)
            .and_then(|count| count.checked_mul(blocks_per_head))
            .ok_or_else(|| {
                HelmDR4AttentionError::Invalid(
                    "learned-manifold adapter dimensions overflow usize".to_owned(),
                )
            })
    }

    fn adapter(
        &self,
        role: LearnedProjectionRole,
        layer: usize,
        head: usize,
        block: usize,
    ) -> Result<R4AffineAdapter, HelmDR4AttentionError> {
        let (heads, adapters, label) = match role {
            LearnedProjectionRole::Query => (self.query_heads, &self.query_adapters, "query"),
            LearnedProjectionRole::Key => (self.key_value_heads, &self.key_adapters, "key"),
            LearnedProjectionRole::Value => (self.key_value_heads, &self.value_adapters, "value"),
        };
        if layer >= self.layers || head >= heads || block >= self.blocks_per_head {
            return Err(HelmDR4AttentionError::Invalid(format!(
                "learned-manifold {label} adapter index ({layer},{head},{block}) exceeds ({},{heads},{})",
                self.layers, self.blocks_per_head
            )));
        }
        let index = (layer * heads + head) * self.blocks_per_head + block;
        adapters.get(index).copied().ok_or_else(|| {
            HelmDR4AttentionError::Invalid(format!(
                "learned-manifold {label} adapter index is unavailable"
            ))
        })
    }
}

pub const HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY: &str = concat!(
    "schema=helm-d-learned-manifold-r4-construction/2\n",
    "upstream=Graph-and-Geometric-Learning/helm@7501deca8f413848bfef804be64ce874b72a3cd7\n",
    "projection=learned-block-diagonal-r4-affine-adapter-over-donor-qkv-before-rope\n",
    "position=unchanged-donor-rope-on-query-and-key\n",
    "manifold=one-unit-lorentz-h64-point-per-64-spatial-lane-head\n",
    "score=(2+2*lorentz-inner)/learned-layer-scale+uniform-bias\n",
    "selector=stable-complete-prefix-causal-softmax\n",
    "aggregate=full-head-normalized-lorentz-centroid\n",
    "storage=sixteen-r4-blocks-per-head\n",
    "transport=coherent-exact-cumulative-spin-h4-query-frame\n",
    "output=unchanged-frozen-donor-wo\n",
    "numerical-adaptation=compensated-f64-sums-and-fail-closed-future-timelike-normalization\n",
    "not-claimed=full-dense-qkv,checkpoint-parity,paper-result-inheritance,softmax-free,source-free,transformerless-serving"
);

pub const HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY: &str = concat!(
    "schema=helm-d-learned-manifold-r4-euclidean-control/2\n",
    "projection=identical-learned-block-diagonal-r4-affine-capacity\n",
    "position=unchanged-donor-rope-on-query-and-key\n",
    "score=negative-full-head-squared-euclidean-distance/learned-layer-scale+uniform-bias\n",
    "selector=identical-stable-complete-prefix-causal-softmax\n",
    "aggregate=full-head-arithmetic-centroid\n",
    "storage-and-transport=identical-r4-block-layout-and-coherent-spin-h4-query-frame\n",
    "output=unchanged-frozen-donor-wo\n",
    "claim=equal-capacity-curvature-destroying-control"
);

/// Pure full-head logit used by both the fitter and the live decoder seam.
pub fn helm_d_learned_manifold_logit(
    metric: HelmDLearnedManifoldMetric,
    query: &[f64],
    key: &[f64],
    learned_scale: f64,
    bias: f64,
) -> Result<f64, HelmDR4AttentionError> {
    if query.is_empty()
        || query.len() != key.len()
        || query.iter().chain(key).any(|value| !value.is_finite())
        || !learned_scale.is_finite()
        || learned_scale <= 0.0
        || !bias.is_finite()
    {
        return Err(HelmDR4AttentionError::Invalid(
            "learned-manifold logit inputs are empty, misaligned, non-finite, or have invalid scale"
                .to_owned(),
        ));
    }
    let numerator = match metric {
        HelmDLearnedManifoldMetric::Lorentz => {
            let query_norm_squared = compensated_square_sum(query)?;
            let key_norm_squared = compensated_square_sum(key)?;
            let query_time = libm::sqrt(1.0 + query_norm_squared);
            let key_time = libm::sqrt(1.0 + key_norm_squared);
            let dot = compensated_dot(query, key)?;
            2.0 + 2.0 * (-query_time * key_time + dot)
        }
        HelmDLearnedManifoldMetric::Euclidean => {
            let differences = query
                .iter()
                .zip(key)
                .map(|(left, right)| *left - *right)
                .collect::<Vec<_>>();
            -compensated_square_sum(&differences)?
        }
    };
    let logit = numerator / learned_scale + bias;
    if !logit.is_finite() {
        return Err(HelmDR4AttentionError::Arithmetic(
            "learned-manifold logit is non-finite".to_owned(),
        ));
    }
    Ok(logit)
}

/// Pure whole-head aggregate. Lorentz values are lifted as one H^D point,
/// never as an independent product of H4 blocks.
pub fn helm_d_learned_manifold_centroid(
    metric: HelmDLearnedManifoldMetric,
    values: &[Vec<f64>],
    weights: &[f64],
) -> Result<Vec<f64>, HelmDR4AttentionError> {
    if values.is_empty()
        || values.len() != weights.len()
        || values[0].is_empty()
        || values.iter().any(|value| {
            value.len() != values[0].len() || value.iter().any(|coordinate| !coordinate.is_finite())
        })
        || weights
            .iter()
            .any(|weight| !weight.is_finite() || *weight < 0.0)
    {
        return Err(HelmDR4AttentionError::Invalid(
            "learned-manifold centroid values/weights are empty, misaligned, or non-finite"
                .to_owned(),
        ));
    }
    let weight_sum = compensated_sum(weights)?;
    if weight_sum <= EPSILON {
        return Err(HelmDR4AttentionError::Arithmetic(
            "learned-manifold centroid weight sum is not positive".to_owned(),
        ));
    }
    let width = values[0].len();
    let mut spatial_sum = vec![0.0; width];
    let mut spatial_correction = vec![0.0; width];
    let mut time_sum = 0.0;
    let mut time_correction = 0.0;
    for (value, weight) in values.iter().zip(weights) {
        let normalized_weight = *weight / weight_sum;
        for ((sum, correction), coordinate) in spatial_sum
            .iter_mut()
            .zip(&mut spatial_correction)
            .zip(value)
        {
            compensated_add(sum, correction, normalized_weight * *coordinate);
        }
        if metric == HelmDLearnedManifoldMetric::Lorentz {
            let time = libm::sqrt(1.0 + compensated_square_sum(value)?);
            compensated_add(
                &mut time_sum,
                &mut time_correction,
                normalized_weight * time,
            );
        }
    }
    for (sum, correction) in spatial_sum.iter_mut().zip(spatial_correction) {
        *sum += correction;
    }
    time_sum += time_correction;
    if spatial_sum.iter().any(|coordinate| !coordinate.is_finite()) {
        return Err(HelmDR4AttentionError::Arithmetic(
            "learned-manifold centroid spatial sum is non-finite".to_owned(),
        ));
    }
    if metric == HelmDLearnedManifoldMetric::Euclidean {
        return Ok(spatial_sum);
    }

    let spatial_norm = libm::sqrt(compensated_square_sum(&spatial_sum)?);
    let lower = time_sum - spatial_norm;
    let upper = time_sum + spatial_norm;
    let timelike_norm_squared = lower * upper;
    if !time_sum.is_finite()
        || !spatial_norm.is_finite()
        || lower <= 0.0
        || !timelike_norm_squared.is_finite()
        || timelike_norm_squared <= EPSILON
    {
        return Err(HelmDR4AttentionError::Arithmetic(
            "learned-manifold Lorentz centroid is not future timelike".to_owned(),
        ));
    }
    let normalization = libm::sqrt(timelike_norm_squared).recip();
    for coordinate in &mut spatial_sum {
        *coordinate *= normalization;
    }
    let normalized_time = time_sum * normalization;
    let residual =
        (-normalized_time * normalized_time + compensated_square_sum(&spatial_sum)? + 1.0).abs();
    let residual_scale =
        1.0 + normalized_time * normalized_time + compensated_square_sum(&spatial_sum)?;
    if spatial_sum.iter().any(|coordinate| !coordinate.is_finite())
        || !normalized_time.is_finite()
        || normalized_time <= 0.0
        || !residual.is_finite()
        || residual > 1.0e-9 * residual_scale
    {
        return Err(HelmDR4AttentionError::Arithmetic(format!(
            "learned-manifold Lorentz centroid residual {residual} exceeds tolerance"
        )));
    }
    Ok(spatial_sum)
}

fn compensated_add(sum: &mut f64, correction: &mut f64, value: f64) {
    let next = *sum + value;
    if sum.abs() >= value.abs() {
        *correction += (*sum - next) + value;
    } else {
        *correction += (value - next) + *sum;
    }
    *sum = next;
}

fn compensated_sum(values: &[f64]) -> Result<f64, HelmDR4AttentionError> {
    let mut sum = 0.0;
    let mut correction = 0.0;
    for value in values {
        compensated_add(&mut sum, &mut correction, *value);
    }
    let total = sum + correction;
    if !total.is_finite() {
        return Err(HelmDR4AttentionError::Arithmetic(
            "compensated sum is non-finite".to_owned(),
        ));
    }
    Ok(total)
}

fn compensated_square_sum(values: &[f64]) -> Result<f64, HelmDR4AttentionError> {
    let squares = values.iter().map(|value| value * value).collect::<Vec<_>>();
    compensated_sum(&squares)
}

fn compensated_dot(left: &[f64], right: &[f64]) -> Result<f64, HelmDR4AttentionError> {
    let products = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .collect::<Vec<_>>();
    compensated_sum(&products)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelmDLearnedManifoldAudit {
    pub projection_tuples: u64,
    pub projected_query_lanes: u64,
    pub projected_key_lanes: u64,
    pub projected_value_lanes: u64,
    pub score_rows: u64,
    pub compatibility_pairs: u64,
    pub centroid_rows: u64,
    pub centroid_source_pairs: u64,
    pub source_frame_permutations: u64,
    pub value_permutations: u64,
    pub order_key_permutations: u64,
    pub arithmetic_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelmDLearnedManifoldEvidence {
    pub schema: String,
    pub policy_identity: String,
    pub parameter_identity: String,
    pub scalar_parameter_count: usize,
    pub metric: HelmDLearnedManifoldMetric,
    pub intervention: HelmDLearnedManifoldIntervention,
    pub frame_table_offsets: Vec<u16>,
    pub transport_audit: R4SpinTransportAudit,
    pub learned_manifold_audit: HelmDLearnedManifoldAudit,
}

/// Live compiler-side construction operator retaining the donor decoder,
/// ordinary complete-prefix causal softmax, RoPE, and Wo around a copied
/// HELM-D Lorentz score/centroid core.
#[derive(Debug, Clone)]
pub struct HelmDLearnedManifoldR4Transport {
    atlas: R4SpinFrameAtlas,
    parameters: HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    intervention: HelmDLearnedManifoldIntervention,
    audit: HelmDLearnedManifoldAudit,
    logit_scratch: Vec<f64>,
    fault: Option<String>,
}

impl HelmDLearnedManifoldR4Transport {
    pub fn new(
        maximum_token_id: u32,
        sequence_capacity: usize,
        parameters: HelmDLearnedManifoldParameters,
        metric: HelmDLearnedManifoldMetric,
        intervention: HelmDLearnedManifoldIntervention,
    ) -> Result<Self, HelmDR4AttentionError> {
        parameters.validate()?;
        Ok(Self {
            atlas: R4SpinFrameAtlas::new(maximum_token_id, sequence_capacity)?,
            parameters,
            metric,
            intervention,
            audit: HelmDLearnedManifoldAudit::default(),
            logit_scratch: vec![0.0; sequence_capacity],
            fault: None,
        })
    }

    pub fn parameters(&self) -> &HelmDLearnedManifoldParameters {
        &self.parameters
    }

    pub const fn metric(&self) -> HelmDLearnedManifoldMetric {
        self.metric
    }

    pub const fn intervention(&self) -> HelmDLearnedManifoldIntervention {
        self.intervention
    }

    pub const fn audit(&self) -> HelmDLearnedManifoldAudit {
        self.audit
    }

    pub const fn transport_audit(&self) -> R4SpinTransportAudit {
        self.atlas.audit()
    }

    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    pub fn policy_identity(&self) -> &'static str {
        match self.metric {
            HelmDLearnedManifoldMetric::Lorentz => HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY,
            HelmDLearnedManifoldMetric::Euclidean => HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY,
        }
    }

    pub fn parameter_identity(&self) -> Result<String, HelmDR4AttentionError> {
        self.parameters.parameter_identity()
    }

    pub fn evidence_snapshot(&self) -> Result<HelmDLearnedManifoldEvidence, HelmDR4AttentionError> {
        let frame_table_offsets = (0..self.atlas.next_position())
            .map(|position| self.atlas.frame_table_offset(position))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(HelmDLearnedManifoldEvidence {
            schema: "uor-r4.helm-d-learned-manifold-r4-evidence/2".to_owned(),
            policy_identity: self.policy_identity().to_owned(),
            parameter_identity: self.parameter_identity()?,
            scalar_parameter_count: self.parameters.scalar_parameter_count()?,
            metric: self.metric,
            intervention: self.intervention,
            frame_table_offsets,
            transport_audit: self.atlas.audit(),
            learned_manifold_audit: self.audit,
        })
    }

    fn transport_intervention(&self) -> R4SpinTransportIntervention {
        match self.intervention {
            HelmDLearnedManifoldIntervention::SourceFramePermuted => {
                R4SpinTransportIntervention::SourceFramePermuted
            }
            HelmDLearnedManifoldIntervention::Coherent
            | HelmDLearnedManifoldIntervention::ValuePermuted
            | HelmDLearnedManifoldIntervention::OrderKeyShuffled => {
                R4SpinTransportIntervention::Coherent
            }
        }
    }

    fn record_fault(&mut self, reason: impl Into<String>) {
        if self.fault.is_none() {
            self.audit.arithmetic_failures = self.audit.arithmetic_failures.saturating_add(1);
            self.fault = Some(reason.into());
        }
    }

    fn fail_and_copy(&mut self, reason: impl Into<String>, input: &[f32], output: &mut [f32]) {
        self.record_fault(reason);
        output.fill(0.0);
        for (target, source) in output.iter_mut().zip(input) {
            *target = *source;
        }
    }

    fn read_block(input: &[f32], offset: usize) -> Vector4 {
        [
            f64::from(input[offset]),
            f64::from(input[offset + 1]),
            f64::from(input[offset + 2]),
            f64::from(input[offset + 3]),
        ]
    }

    fn write_block(output: &mut [f32], offset: usize, block: Vector4) -> bool {
        for (target, source) in output[offset..offset + R4_WIDTH].iter_mut().zip(block) {
            *target = source as f32;
            if !target.is_finite() {
                return false;
            }
        }
        true
    }

    fn valid_head_slices(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        if input.len() == self.parameters.head_width() && output.len() == input.len() {
            true
        } else {
            self.fail_and_copy(
                format!(
                    "learned-manifold transport requires one {}-lane head; input={}, output={}",
                    self.parameters.head_width(),
                    input.len(),
                    output.len()
                ),
                input,
                output,
            );
            false
        }
    }

    fn apply_projection_role(
        &self,
        role: LearnedProjectionRole,
        context: CausalAttentionProjectionContext,
        vectors: &mut [f32],
    ) -> Result<(), HelmDR4AttentionError> {
        let heads = match role {
            LearnedProjectionRole::Query => context.query_heads,
            LearnedProjectionRole::Key | LearnedProjectionRole::Value => context.key_value_heads,
        };
        if context.layer >= self.parameters.layers()
            || context.query_heads != self.parameters.query_heads()
            || context.key_value_heads != self.parameters.key_value_heads()
            || context.head_size != self.parameters.head_width()
            || heads
                .checked_mul(context.head_size)
                .filter(|expected| *expected == vectors.len())
                .is_none()
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold pre-RoPE projection shape differs from frozen parameters"
                    .to_owned(),
            ));
        }
        for head in 0..heads {
            let head_offset = head * context.head_size;
            for block in 0..self.parameters.blocks_per_head() {
                let offset = head_offset + block * R4_WIDTH;
                let adapter = self.parameters.adapter(role, context.layer, head, block)?;
                let output = adapter.apply(Self::read_block(vectors, offset))?;
                if !Self::write_block(vectors, offset, output) {
                    return Err(HelmDR4AttentionError::Arithmetic(
                        "learned-manifold pre-RoPE projection overflowed f32".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn score_row(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
    ) -> Result<(), HelmDR4AttentionError> {
        let width = self.parameters.head_width();
        if query.len() != width
            || context.head >= self.parameters.query_heads()
            || output_weights.is_empty()
            || output_weights.len() > self.logit_scratch.len()
            || output_weights.len().checked_mul(width) != Some(packed_keys.len())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold score row has inconsistent shapes".to_owned(),
            ));
        }
        let query = query
            .iter()
            .map(|value| f64::from(*value))
            .collect::<Vec<_>>();
        let scale = self.parameters.learned_scale(context.layer)?;
        let bias = self.parameters.learned_bias(context.layer)?;
        for source in 0..output_weights.len() {
            let key_source = match self.intervention {
                HelmDLearnedManifoldIntervention::OrderKeyShuffled if output_weights.len() > 1 => {
                    self.audit.order_key_permutations =
                        self.audit.order_key_permutations.saturating_add(1);
                    (source + 1) % output_weights.len()
                }
                _ => source,
            };
            let key = &packed_keys[key_source * width..(key_source + 1) * width];
            let key = key
                .iter()
                .map(|value| f64::from(*value))
                .collect::<Vec<_>>();
            self.logit_scratch[source] =
                helm_d_learned_manifold_logit(self.metric, &query, &key, scale, bias)?;
            self.audit.compatibility_pairs = self.audit.compatibility_pairs.saturating_add(1);
        }
        intrinsic_stable_softmax_into(
            &mut self.logit_scratch[..output_weights.len()],
            output_weights,
        )?;
        self.audit.score_rows = self.audit.score_rows.saturating_add(1);
        Ok(())
    }

    fn centroid_row(
        &mut self,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) -> Result<(), HelmDR4AttentionError> {
        let width = self.parameters.head_width();
        if output.len() != width
            || weights.is_empty()
            || weights.len().checked_mul(width) != Some(packed_values.len())
        {
            return Err(HelmDR4AttentionError::Invalid(
                "learned-manifold centroid row has inconsistent shapes".to_owned(),
            ));
        }
        let mut values = Vec::with_capacity(weights.len());
        for source in 0..weights.len() {
            let value_source = match self.intervention {
                HelmDLearnedManifoldIntervention::ValuePermuted if weights.len() > 1 => {
                    self.audit.value_permutations = self.audit.value_permutations.saturating_add(1);
                    (source + 1) % weights.len()
                }
                _ => source,
            };
            values.push(
                packed_values[value_source * width..(value_source + 1) * width]
                    .iter()
                    .map(|value| f64::from(*value))
                    .collect::<Vec<_>>(),
            );
        }
        let weights = weights
            .iter()
            .map(|weight| f64::from(*weight))
            .collect::<Vec<_>>();
        let centroid = helm_d_learned_manifold_centroid(self.metric, &values, &weights)?;
        for (target, source) in output.iter_mut().zip(centroid) {
            *target = source as f32;
            if !target.is_finite() {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "learned-manifold centroid overflowed f32".to_owned(),
                ));
            }
        }
        self.audit.centroid_source_pairs = self
            .audit
            .centroid_source_pairs
            .saturating_add(u64::try_from(weights.len()).unwrap_or(u64::MAX));
        self.audit.centroid_rows = self.audit.centroid_rows.saturating_add(1);
        Ok(())
    }
}

impl CausalAttentionTransport for HelmDLearnedManifoldR4Transport {
    fn reset(&mut self) {
        self.atlas.reset();
        self.audit = HelmDLearnedManifoldAudit::default();
        self.fault = None;
    }

    fn policy_identity(&self) -> &str {
        HelmDLearnedManifoldR4Transport::policy_identity(self)
    }

    fn implementation_evidence(&self) -> Result<Option<String>, String> {
        self.evidence_snapshot()
            .and_then(|evidence| {
                serde_json::to_string(&evidence)
                    .map_err(|error| HelmDR4AttentionError::Invalid(error.to_string()))
            })
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn status(&self) -> Result<(), String> {
        match &self.fault {
            Some(reason) => Err(reason.clone()),
            None => Ok(()),
        }
    }

    fn begin_position(&mut self, token: usize, position: usize) {
        let result = u32::try_from(token)
            .map_err(|_| {
                HelmDR4AttentionError::Invalid(
                    "decoder token does not fit the UOR u32 namespace".to_owned(),
                )
            })
            .and_then(|token| self.atlas.begin_position(token, position));
        if let Err(error) = result {
            self.record_fault(error.to_string());
        }
    }

    fn transform_projected_qkv_before_rope(
        &mut self,
        context: CausalAttentionProjectionContext,
        query: &mut [f32],
        key: &mut [f32],
        value: &mut [f32],
    ) {
        if self.fault.is_some() {
            return;
        }
        let result = self
            .apply_projection_role(LearnedProjectionRole::Query, context, query)
            .and_then(|()| self.apply_projection_role(LearnedProjectionRole::Key, context, key))
            .and_then(|()| {
                self.apply_projection_role(LearnedProjectionRole::Value, context, value)
            });
        match result {
            Ok(()) => {
                self.audit.projection_tuples = self.audit.projection_tuples.saturating_add(1);
                self.audit.projected_query_lanes = self
                    .audit
                    .projected_query_lanes
                    .saturating_add(u64::try_from(query.len()).unwrap_or(u64::MAX));
                self.audit.projected_key_lanes = self
                    .audit
                    .projected_key_lanes
                    .saturating_add(u64::try_from(key.len()).unwrap_or(u64::MAX));
                self.audit.projected_value_lanes = self
                    .audit
                    .projected_value_lanes
                    .saturating_add(u64::try_from(value.len()).unwrap_or(u64::MAX));
            }
            Err(error) => self.record_fault(error.to_string()),
        }
    }

    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_head_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy(
                    "learned-manifold transport is already faulted",
                    input,
                    output,
                );
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let encoded = self
                .atlas
                .encode_model_block(context.query_position, Self::read_block(input, offset));
            match encoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "learned-manifold query encoding overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_head_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy(
                    "learned-manifold transport is already faulted",
                    input,
                    output,
                );
            }
            return;
        }
        let intervention = self.transport_intervention();
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        intervention,
                        false,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "learned-manifold key transport overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
        if intervention == R4SpinTransportIntervention::SourceFramePermuted
            && context.query_position != 0
        {
            self.audit.source_frame_permutations = self
                .audit
                .source_frame_permutations
                .saturating_add(u64::try_from(input.len() / R4_WIDTH).unwrap_or(u64::MAX));
        }
    }

    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_head_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy(
                    "learned-manifold transport is already faulted",
                    input,
                    output,
                );
            }
            return;
        }
        let intervention = self.transport_intervention();
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let result = self
                .atlas
                .encode_model_block(context.source_position, Self::read_block(input, offset))
                .and_then(|local| {
                    self.atlas.transport_local_block(
                        context.source_position,
                        context.query_position,
                        local,
                        intervention,
                        true,
                    )
                });
            match result {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "learned-manifold value transport overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
        if intervention == R4SpinTransportIntervention::SourceFramePermuted
            && context.query_position != 0
        {
            self.audit.source_frame_permutations = self
                .audit
                .source_frame_permutations
                .saturating_add(u64::try_from(input.len() / R4_WIDTH).unwrap_or(u64::MAX));
        }
    }

    fn output_to_model_frame(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() || !self.valid_head_slices(input, output) {
            if self.fault.is_some() {
                self.fail_and_copy(
                    "learned-manifold transport is already faulted",
                    input,
                    output,
                );
            }
            return;
        }
        for offset in (0..input.len()).step_by(R4_WIDTH) {
            let decoded = self
                .atlas
                .decode_query_block(context.query_position, Self::read_block(input, offset));
            match decoded {
                Ok(block) if Self::write_block(output, offset, block) => {}
                Ok(_) => {
                    self.fail_and_copy(
                        "learned-manifold output decoding overflowed f32",
                        input,
                        output,
                    );
                    return;
                }
                Err(error) => {
                    self.fail_and_copy(error.to_string(), input, output);
                    return;
                }
            }
        }
    }

    fn score_and_normalize(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        _canonical_math: bool,
    ) {
        if self.fault.is_some() {
            output_weights.fill(0.0);
            return;
        }
        if let Err(error) = self.score_row(context, query, packed_keys, output_weights) {
            output_weights.fill(0.0);
            self.record_fault(error.to_string());
        }
    }

    fn weighted_value_centroid(
        &mut self,
        _context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) {
        if self.fault.is_some() {
            output.fill(0.0);
            return;
        }
        if let Err(error) = self.centroid_row(weights, packed_values, output) {
            output.fill(0.0);
            self.record_fault(error.to_string());
        }
    }
}

fn lorentz_project(spatial: &[f64], curvature: f64) -> Result<Vec<f64>, HelmDR4AttentionError> {
    let spatial_norm_squared = spatial.iter().map(|value| value * value).sum::<f64>();
    let time = (spatial_norm_squared + curvature).sqrt();
    if !time.is_finite() {
        return Err(HelmDR4AttentionError::Arithmetic(
            "HELM-D Lorentz projection is non-finite".to_owned(),
        ));
    }
    let mut projected = Vec::with_capacity(spatial.len() + 1);
    projected.push(time);
    projected.extend_from_slice(spatial);
    Ok(projected)
}

fn lorentz_inner(left: &[f64], right: &[f64]) -> f64 {
    -left[0] * right[0]
        + left[1..]
            .iter()
            .zip(&right[1..])
            .map(|(left, right)| left * right)
            .sum::<f64>()
}

fn stable_softmax(logits: &[f64]) -> Result<Vec<f64>, HelmDR4AttentionError> {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err(HelmDR4AttentionError::Arithmetic(
            "softmax logits are empty or non-finite".to_owned(),
        ));
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut weights = logits
        .iter()
        .map(|logit| (*logit - maximum).exp())
        .collect::<Vec<_>>();
    let denominator = weights.iter().sum::<f64>();
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(HelmDR4AttentionError::Arithmetic(
            "softmax denominator is invalid".to_owned(),
        ));
    }
    for weight in &mut weights {
        *weight /= denominator;
    }
    Ok(weights)
}

fn h4_left_quaternion_matrix(
    state: ExactSpinState,
    table: &H4BinaryIcosahedralClosure,
) -> Result<Matrix4, HelmDR4AttentionError> {
    let coordinate = state
        .root_coordinate(table)
        .map_err(|error| HelmDR4AttentionError::ExactRoute(error.to_string()))?;
    let mut quaternion = [0.0; R4_WIDTH];
    for (target, [integer, phi]) in quaternion.iter_mut().zip(coordinate.scaled_zphi_quaternion) {
        *target = (integer as f64 + (phi as f64) * GOLDEN_RATIO) * 0.5;
    }
    let norm = quaternion
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= EPSILON {
        return Err(HelmDR4AttentionError::Arithmetic(
            "H4 frame quaternion has invalid norm".to_owned(),
        ));
    }
    for value in &mut quaternion {
        *value /= norm;
    }
    let [w, x, y, z] = quaternion;
    let matrix = [[w, -x, -y, -z], [x, w, -z, y], [y, z, w, -x], [z, -y, x, w]];
    let orthogonality = matrix_multiply(matrix, transpose(matrix));
    for (row, values) in orthogonality.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            let expected = if row == column { 1.0 } else { 0.0 };
            if (*value - expected).abs() > 1.0e-9 {
                return Err(HelmDR4AttentionError::Arithmetic(
                    "H4 frame matrix is not numerically orthogonal".to_owned(),
                ));
            }
        }
    }
    Ok(matrix)
}

fn checked_matrix_vector(
    matrix: Matrix4,
    vector: Vector4,
    label: &str,
) -> Result<Vector4, HelmDR4AttentionError> {
    let mut output = [0.0; R4_WIDTH];
    for (row, result) in output.iter_mut().enumerate() {
        for (column, value) in vector.iter().enumerate() {
            *result += matrix[row][column] * value;
        }
    }
    if output.iter().any(|value| !value.is_finite()) {
        return Err(HelmDR4AttentionError::Arithmetic(format!(
            "{label} produced a non-finite coordinate"
        )));
    }
    Ok(output)
}

fn transpose(matrix: Matrix4) -> Matrix4 {
    let mut output = [[0.0; R4_WIDTH]; R4_WIDTH];
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            output[column][row] = *value;
        }
    }
    output
}

fn matrix_multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut output = [[0.0; R4_WIDTH]; R4_WIDTH];
    for (row, values) in output.iter_mut().enumerate() {
        for (column, value) in values.iter_mut().enumerate() {
            for inner in 0..R4_WIDTH {
                *value += left[row][inner] * right[inner][column];
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(left: Vector4, right: Vector4) -> f64 {
        left.iter()
            .zip(right)
            .map(|(left, right)| left * right)
            .sum()
    }

    #[test]
    fn pinned_helm_d_row_matches_the_audited_upstream_golden_fixture() {
        let trace = helm_d_lorentz_causal_row(
            &[0.25, -0.5, 0.75],
            &[vec![0.1, -0.2, 0.3], vec![-0.4, 0.2, 0.6]],
            &[vec![0.7, 0.1, -0.2], vec![-0.3, 0.8, 0.4]],
            HelmDLorentzReferenceConfig {
                curvature: 1.0,
                learned_scale: 2.5,
                bias: -0.125,
            },
        )
        .expect("pinned HELM-D row");

        // Direct float64 evaluation of the pinned upstream `project`,
        // `cinner`, causal `Softmax`, and `lorentzian_centroid` equations at
        // commit 7501deca. These values make translation drift observable;
        // normalization/manifold invariants alone would not.
        let expected_logits = [-0.214_615_321_377_075_56, -0.493_210_510_118_965_55];
        let expected_weights = [0.569_201_782_148_292_1, 0.430_798_217_851_707_9];
        let expected_centroid = [
            1.078_716_185_780_169_5,
            0.223_617_725_820_237_98,
            0.333_562_632_088_915_3,
            0.048_576_667_619_514_846,
        ];
        for (actual, expected) in trace.logits.iter().zip(expected_logits) {
            assert!((*actual - expected).abs() <= 1.0e-15);
        }
        for (actual, expected) in trace.weights.iter().zip(expected_weights) {
            assert!((*actual - expected).abs() <= 1.0e-15);
        }
        for (actual, expected) in trace.centroid.iter().zip(expected_centroid) {
            assert!((*actual - expected).abs() <= 1.0e-15);
        }
        assert!((trace.weight_sum - 1.0).abs() <= 1.0e-12);
        assert!(trace.lorentz_constraint_residual <= 1.0e-10);
    }

    #[test]
    fn coherent_spin_transport_is_a_gauge_reparameterization() {
        let mut atlas = R4SpinFrameAtlas::new(32, 4).expect("frame atlas");
        for (position, token) in [5, 9, 2, 17].into_iter().enumerate() {
            atlas.begin_position(token, position).expect("causal frame");
        }
        let query = [0.25, -0.5, 0.75, 0.125];
        let key = [-0.4, 0.2, 0.6, -0.3];
        let value = [0.7, 0.1, -0.2, 0.55];
        let query_local = atlas.encode_model_block(3, query).expect("encode query");
        let key_local = atlas.encode_model_block(1, key).expect("encode key");
        let key_query = atlas
            .transport_local_block(
                1,
                3,
                key_local,
                R4SpinTransportIntervention::Coherent,
                false,
            )
            .expect("transport key");
        assert!((dot(query, key) - dot(query_local, key_query)).abs() <= 1.0e-10);

        let value_local = atlas.encode_model_block(1, value).expect("encode value");
        let value_query = atlas
            .transport_local_block(
                1,
                3,
                value_local,
                R4SpinTransportIntervention::Coherent,
                true,
            )
            .expect("transport value");
        let decoded = atlas
            .decode_query_block(3, value_query)
            .expect("decode value");
        for (actual, expected) in decoded.into_iter().zip(value) {
            assert!((actual - expected).abs() <= 1.0e-10);
        }
        assert_eq!(atlas.audit().future_position_reads, 0);
        assert_eq!(atlas.audit().key_blocks_transported, 1);
        assert_eq!(atlas.audit().value_blocks_transported, 1);
    }

    #[test]
    fn source_frame_permutation_breaks_the_coherent_identity() {
        let mut atlas = R4SpinFrameAtlas::new(32, 3).expect("frame atlas");
        for (position, token) in [5, 9, 2].into_iter().enumerate() {
            atlas.begin_position(token, position).expect("causal frame");
        }
        let query = [0.25, -0.5, 0.75, 0.125];
        let key = [-0.4, 0.2, 0.6, -0.3];
        let query_local = atlas.encode_model_block(2, query).expect("encode query");
        let key_local = atlas.encode_model_block(0, key).expect("encode key");
        let permuted = atlas
            .transport_local_block(
                0,
                2,
                key_local,
                R4SpinTransportIntervention::SourceFramePermuted,
                false,
            )
            .expect("permuted transport");
        assert!((dot(query, key) - dot(query_local, permuted)).abs() > 1.0e-6);
        assert_eq!(atlas.audit().source_frame_permutations, 1);
    }

    #[test]
    fn cached_frame_matrix_is_bit_identical_to_direct_reconstruction() {
        let mut atlas = R4SpinFrameAtlas::new(32, 4).expect("frame atlas");
        for (position, token) in [5, 9, 2, 17].into_iter().enumerate() {
            atlas.begin_position(token, position).expect("causal frame");
            let cached = atlas.frame_matrix(position).expect("cached frame matrix");
            let direct = h4_left_quaternion_matrix(
                atlas.frame(position).expect("exact frame"),
                &atlas.exact_route_table,
            )
            .expect("direct frame matrix");
            assert_eq!(cached, direct);
        }
        atlas.reset();
        assert!(atlas.frame_matrix(0).is_err());
    }

    #[test]
    fn intrinsic_distance_has_identity_symmetry_and_nonnegativity() {
        let left = [0.25, -0.5, 0.75, 0.125];
        let right = [-0.4, 0.2, 0.6, -0.3];
        for metric in [
            IntrinsicR4AttentionMetric::Lorentz,
            IntrinsicR4AttentionMetric::Flat,
        ] {
            let identity = intrinsic_r4_score_feature(metric, left, left).expect("identity");
            let forward = intrinsic_r4_score_feature(metric, left, right).expect("forward");
            let reverse = intrinsic_r4_score_feature(metric, right, left).expect("reverse");
            assert!(identity.abs() <= 1.0e-12);
            assert!(forward <= 0.0);
            assert!((forward - reverse).abs() <= 1.0e-12);
            assert!(-forward >= 0.0);
        }
    }

    #[test]
    fn lorentz_distance_and_centroid_are_so4_equivariant() {
        fn rotate(vector: Vector4) -> Vector4 {
            [-vector[1], vector[0], -vector[3], vector[2]]
        }

        let query = [0.25, -0.5, 0.75, 0.125];
        let key = [-0.4, 0.2, 0.6, -0.3];
        let feature = intrinsic_r4_score_feature(IntrinsicR4AttentionMetric::Lorentz, query, key)
            .expect("Lorentz feature");
        let rotated_feature = intrinsic_r4_score_feature(
            IntrinsicR4AttentionMetric::Lorentz,
            rotate(query),
            rotate(key),
        )
        .expect("rotated Lorentz feature");
        assert!((feature - rotated_feature).abs() <= 1.0e-12);

        let values = [
            [0.7, 0.1, -0.2, 0.55],
            [-0.3, 0.8, 0.4, -0.1],
            [0.2, -0.6, 0.1, 0.45],
        ];
        let rotated_values = values.map(rotate);
        let weights = [0.2, 0.3, 0.5];
        let centroid =
            intrinsic_r4_weighted_centroid(IntrinsicR4AttentionMetric::Lorentz, &values, &weights)
                .expect("Lorentz centroid");
        let rotated_centroid = intrinsic_r4_weighted_centroid(
            IntrinsicR4AttentionMetric::Lorentz,
            &rotated_values,
            &weights,
        )
        .expect("rotated Lorentz centroid");
        for (actual, expected) in rotated_centroid.into_iter().zip(rotate(centroid)) {
            assert!((actual - expected).abs() <= 1.0e-12);
        }
    }

    #[test]
    fn lorentz_barycenter_respects_a_one_hot_weight() {
        let values = [
            [0.7, 0.1, -0.2, 0.55],
            [-0.3, 0.8, 0.4, -0.1],
            [0.2, -0.6, 0.1, 0.45],
        ];
        let centroid = intrinsic_r4_weighted_centroid(
            IntrinsicR4AttentionMetric::Lorentz,
            &values,
            &[0.0, 1.0, 0.0],
        )
        .expect("one-hot Lorentz centroid");
        for (actual, expected) in centroid.into_iter().zip(values[1]) {
            assert!((actual - expected).abs() <= 1.0e-12);
        }
    }

    #[test]
    fn lorentz_barycenter_accepts_the_frozen_timelike_floor() {
        let boundary_time = libm::sqrt(EPSILON);
        assert_eq!(boundary_time * boundary_time, EPSILON);
        let boundary =
            intrinsic_lorentz_r4_normalize_barycenter([boundary_time, 0.0, 0.0, 0.0, 0.0])
                .expect("the frozen timelike floor is inclusive");
        assert_eq!(boundary, [0.0; R4_WIDTH]);

        assert!(intrinsic_lorentz_r4_normalize_barycenter([
            boundary_time * 0.5,
            0.0,
            0.0,
            0.0,
            0.0,
        ])
        .is_err());
    }

    #[test]
    fn intrinsic_softmax_exposes_the_live_f32_weight_boundary() {
        let mut logits = [0.0, -1.0, -2.0];
        let mut weights = [0.0f32; 3];
        intrinsic_stable_softmax_into(&mut logits, &mut weights).expect("intrinsic softmax");

        let maximum = 0.0f64;
        let denominator = [0.0f64, -1.0, -2.0]
            .into_iter()
            .map(|logit| libm::exp(logit - maximum))
            .sum::<f64>();
        for (actual, logit) in weights.into_iter().zip([0.0f64, -1.0, -2.0]) {
            assert_eq!(actual, (libm::exp(logit - maximum) / denominator) as f32);
        }
        assert!(logits.iter().all(|value| value.is_finite() && *value > 0.0));
    }

    #[test]
    fn flat_control_is_squared_distance_and_arithmetic_centroid() {
        let left = [1.0, 2.0, 3.0, 4.0];
        let right = [2.0, 0.0, 5.0, 1.0];
        let feature = intrinsic_r4_score_feature(IntrinsicR4AttentionMetric::Flat, left, right)
            .expect("flat feature");
        assert_eq!(feature, -18.0);

        let centroid = intrinsic_r4_weighted_centroid(
            IntrinsicR4AttentionMetric::Flat,
            &[left, right],
            &[1.0, 3.0],
        )
        .expect("flat centroid");
        let expected = [1.75, 0.5, 4.5, 1.75];
        for (actual, expected) in centroid.into_iter().zip(expected) {
            assert!((actual - expected).abs() <= 1.0e-15);
        }
    }

    #[test]
    fn intrinsic_destructive_controls_are_live() {
        let parameters =
            IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, 1.0, 1.0).expect("parameters");
        let mut coherent = IntrinsicR4CausalAttentionTransport::new(
            32,
            3,
            parameters.clone(),
            IntrinsicR4AttentionMetric::Lorentz,
            IntrinsicR4AttentionIntervention::Coherent,
        )
        .expect("coherent transport");
        let mut source_permuted = IntrinsicR4CausalAttentionTransport::new(
            32,
            3,
            parameters.clone(),
            IntrinsicR4AttentionMetric::Lorentz,
            IntrinsicR4AttentionIntervention::SourceFramePermuted,
        )
        .expect("source-permuted transport");
        for (position, token) in [5, 9, 2].into_iter().enumerate() {
            CausalAttentionTransport::begin_position(&mut coherent, token, position);
            CausalAttentionTransport::begin_position(&mut source_permuted, token, position);
        }
        let context = CausalAttentionSourceContext {
            layer: 0,
            head: 0,
            query_position: 2,
            source_position: 0,
        };
        let key = [-0.4_f32, 0.2, 0.6, -0.3];
        let mut coherent_key = [0.0; R4_WIDTH];
        let mut permuted_key = [0.0; R4_WIDTH];
        coherent.transport_key(context, &key, &mut coherent_key);
        source_permuted.transport_key(context, &key, &mut permuted_key);
        assert_ne!(coherent_key, permuted_key);
        assert!(source_permuted.transport_audit().source_frame_permutations > 0);

        let mut value_permuted = IntrinsicR4CausalAttentionTransport::new(
            32,
            1,
            parameters,
            IntrinsicR4AttentionMetric::Lorentz,
            IntrinsicR4AttentionIntervention::ValuePermuted,
        )
        .expect("value-permuted transport");
        let head_context = CausalAttentionHeadContext {
            layer: 0,
            head: 0,
            query_position: 1,
        };
        let weights = [0.8_f32, 0.2];
        let packed_values = [0.7_f32, 0.1, -0.2, 0.55, -0.3, 0.8, 0.4, -0.1];
        let mut coherent_centroid = [0.0; R4_WIDTH];
        let mut permuted_centroid = [0.0; R4_WIDTH];
        coherent
            .centroid_row(
                head_context,
                &weights,
                &packed_values,
                &mut coherent_centroid,
            )
            .expect("coherent centroid");
        value_permuted
            .centroid_row(
                head_context,
                &weights,
                &packed_values,
                &mut permuted_centroid,
            )
            .expect("permuted centroid");
        assert_ne!(coherent_centroid, permuted_centroid);
        assert_eq!(value_permuted.intrinsic_audit().value_permutations, 2);
        assert_eq!(value_permuted.intrinsic_audit().centroid_source_pairs, 2);
    }

    #[test]
    fn intrinsic_parameters_allow_zero_scores_and_reject_invalid_values() {
        assert!(IntrinsicLorentzR4AttentionParameters::uniform(0, 1, 1, 1.0, 1.0).is_err());
        assert!(
            IntrinsicLorentzR4AttentionParameters::new(1, 1, 2, vec![1.0], vec![1.0, 1.0]).is_err()
        );
        assert!(IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, 0.0, 1.0).is_ok());
        assert!(IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, -1.0, 1.0).is_err());
        assert!(IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, 1.0, 0.0).is_err());
        assert!(IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, 1.0, f64::NAN).is_err());
    }

    #[test]
    fn intrinsic_operator_binds_policy_evidence_and_fails_closed() {
        let parameters =
            IntrinsicLorentzR4AttentionParameters::uniform(1, 1, 1, 1.0, 1.0).expect("parameters");
        let mut operator = IntrinsicR4CausalAttentionTransport::new(
            32,
            2,
            parameters,
            IntrinsicR4AttentionMetric::Lorentz,
            IntrinsicR4AttentionIntervention::Coherent,
        )
        .expect("operator");
        assert_eq!(
            CausalAttentionTransport::policy_identity(&operator),
            INTRINSIC_LORENTZ_R4_ATTENTION_POLICY
        );
        assert!(CausalAttentionTransport::implementation_evidence(&operator)
            .expect("evidence")
            .is_some());

        let context = CausalAttentionHeadContext {
            layer: 0,
            head: 0,
            query_position: 0,
        };
        let scratch_pointer = operator.logit_scratch.as_ptr();
        let mut weights = [0.0_f32; 2];
        operator.score_and_normalize(
            context,
            &[0.1, 0.2, 0.3, 0.4],
            &[0.1, 0.2, 0.3, 0.4, -0.4, 0.2, 0.6, -0.3],
            &mut weights,
            true,
        );
        assert!(CausalAttentionTransport::status(&operator).is_ok());
        assert_eq!(operator.logit_scratch.as_ptr(), scratch_pointer);
        assert!((weights.iter().sum::<f32>() - 1.0).abs() <= f32::EPSILON);

        let packed_values = [1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let mut centroid = [0.0_f32; R4_WIDTH];
        operator.weighted_value_centroid(context, &weights, &packed_values, &mut centroid);
        let expected = intrinsic_r4_weighted_centroid(
            IntrinsicR4AttentionMetric::Lorentz,
            &[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]],
            &[f64::from(weights[0]), f64::from(weights[1])],
        )
        .expect("pure Lorentz centroid");
        for (actual, expected) in centroid.into_iter().zip(expected) {
            assert!((f64::from(actual) - expected).abs() <= 1.0e-7);
        }
        assert_eq!(operator.logit_scratch.as_ptr(), scratch_pointer);
        assert_eq!(operator.intrinsic_audit().score_rows, 1);
        assert_eq!(operator.intrinsic_audit().compatibility_pairs, 2);
        assert_eq!(operator.intrinsic_audit().centroid_rows, 1);
        assert_eq!(operator.intrinsic_audit().centroid_source_pairs, 2);

        let mut no_weights = [];
        operator.score_and_normalize(context, &[0.1, 0.2, 0.3, 0.4], &[], &mut no_weights, true);
        assert!(CausalAttentionTransport::status(&operator).is_err());
        assert_eq!(operator.intrinsic_audit().arithmetic_failures, 1);
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_has_the_frozen_matched_capacity() {
        let parameters =
            HelmDLearnedManifoldParameters::identity(30, 9, 3, 16, 24.0).expect("parameters");
        assert_eq!(parameters.head_width(), 64);
        assert_eq!(parameters.scalar_parameter_count().expect("count"), 144_060);
        assert_eq!(parameters.query_adapters().len(), 30 * 9 * 16);
        assert_eq!(parameters.key_adapters().len(), 30 * 3 * 16);
        assert_eq!(parameters.value_adapters().len(), 30 * 3 * 16);

        let lorentz = HelmDLearnedManifoldR4Transport::new(
            32,
            2,
            parameters.clone(),
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldIntervention::Coherent,
        )
        .expect("Lorentz arm");
        let euclidean = HelmDLearnedManifoldR4Transport::new(
            32,
            2,
            parameters,
            HelmDLearnedManifoldMetric::Euclidean,
            HelmDLearnedManifoldIntervention::Coherent,
        )
        .expect("Euclidean arm");
        assert_eq!(
            lorentz
                .parameters()
                .scalar_parameter_count()
                .expect("Lorentz count"),
            euclidean
                .parameters()
                .scalar_parameter_count()
                .expect("Euclidean count")
        );
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_identity_projection_is_exact() {
        let parameters =
            HelmDLearnedManifoldParameters::identity(1, 9, 3, 16, 24.0).expect("parameters");
        let mut operator = HelmDLearnedManifoldR4Transport::new(
            64,
            2,
            parameters,
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldIntervention::Coherent,
        )
        .expect("operator");
        let mut query = (0..576)
            .map(|index| (index as f32 - 288.0) / 1024.0)
            .collect::<Vec<_>>();
        let mut key = (0..192)
            .map(|index| (index as f32 - 96.0) / 512.0)
            .collect::<Vec<_>>();
        let mut value = (0..192)
            .map(|index| (96.0 - index as f32) / 768.0)
            .collect::<Vec<_>>();
        let expected = (query.clone(), key.clone(), value.clone());
        CausalAttentionTransport::transform_projected_qkv_before_rope(
            &mut operator,
            CausalAttentionProjectionContext {
                layer: 0,
                query_position: 0,
                query_heads: 9,
                key_value_heads: 3,
                head_size: 64,
            },
            &mut query,
            &mut key,
            &mut value,
        );
        assert_eq!((query, key, value), expected);
        assert_eq!(operator.audit().projection_tuples, 1);
        assert_eq!(operator.audit().projected_query_lanes, 576);
        assert_eq!(operator.audit().projected_key_lanes, 192);
        assert_eq!(operator.audit().projected_value_lanes, 192);
        assert!(CausalAttentionTransport::status(&operator).is_ok());
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_matches_the_upstream_h64_row() {
        let query = (0..64)
            .map(|index| ((index as f64 + 1.0) * 0.03125).sin() * 0.4)
            .collect::<Vec<_>>();
        let keys = (0..3)
            .map(|source| {
                (0..64)
                    .map(|index| ((index + source * 7) as f64 * 0.023).cos() * 0.35)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let values = (0..3)
            .map(|source| {
                (0..64)
                    .map(|index| ((index * 3 + source * 11) as f64 * 0.017).sin() * 0.25)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let scale = 7.25;
        let bias = -0.375;
        let reference = helm_d_lorentz_causal_row(
            &query,
            &keys,
            &values,
            HelmDLorentzReferenceConfig {
                curvature: 1.0,
                learned_scale: scale,
                bias,
            },
        )
        .expect("upstream row");
        for (source, key) in keys.iter().enumerate() {
            let actual = helm_d_learned_manifold_logit(
                HelmDLearnedManifoldMetric::Lorentz,
                &query,
                key,
                scale,
                bias,
            )
            .expect("full-head logit");
            assert!((actual - reference.logits[source]).abs() <= 1.0e-12);
        }
        let centroid = helm_d_learned_manifold_centroid(
            HelmDLearnedManifoldMetric::Lorentz,
            &values,
            &reference.weights,
        )
        .expect("full-head centroid");
        for (actual, expected) in centroid.iter().zip(&reference.centroid[1..]) {
            assert!((*actual - *expected).abs() <= 1.0e-12);
        }
        let time = libm::sqrt(1.0 + compensated_square_sum(&centroid).expect("norm"));
        let residual =
            (-time * time + compensated_square_sum(&centroid).expect("normalized norm") + 1.0)
                .abs();
        assert!(residual <= 1.0e-12);
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_is_covariant_and_mismatch_is_live() {
        fn model_vector(seed: usize) -> Vec<f64> {
            (0..64)
                .map(|lane| ((seed * 17 + lane * 5) as f64 * 0.019).sin() * 0.3)
                .collect()
        }
        fn encode(atlas: &mut R4SpinFrameAtlas, position: usize, vector: &[f64]) -> Vec<f64> {
            let mut output = Vec::with_capacity(vector.len());
            for block in vector.chunks_exact(R4_WIDTH) {
                output.extend_from_slice(
                    &atlas
                        .encode_model_block(position, [block[0], block[1], block[2], block[3]])
                        .expect("encode"),
                );
            }
            output
        }
        fn transport(
            atlas: &mut R4SpinFrameAtlas,
            source: usize,
            query: usize,
            vector: &[f64],
            intervention: R4SpinTransportIntervention,
            value: bool,
        ) -> Vec<f64> {
            let mut output = Vec::with_capacity(vector.len());
            for block in vector.chunks_exact(R4_WIDTH) {
                let local = atlas
                    .encode_model_block(source, [block[0], block[1], block[2], block[3]])
                    .expect("source encode");
                output.extend_from_slice(
                    &atlas
                        .transport_local_block(source, query, local, intervention, value)
                        .expect("transport"),
                );
            }
            output
        }
        fn decode(atlas: &mut R4SpinFrameAtlas, position: usize, vector: &[f64]) -> Vec<f64> {
            let mut output = Vec::with_capacity(vector.len());
            for block in vector.chunks_exact(R4_WIDTH) {
                output.extend_from_slice(
                    &atlas
                        .decode_query_block(position, [block[0], block[1], block[2], block[3]])
                        .expect("decode"),
                );
            }
            output
        }

        let mut atlas = R4SpinFrameAtlas::new(64, 3).expect("atlas");
        for (position, token) in [5, 11, 23].into_iter().enumerate() {
            atlas.begin_position(token, position).expect("position");
        }
        let query = model_vector(3);
        let keys = [model_vector(7), model_vector(13), model_vector(19)];
        let values = [model_vector(29), model_vector(31), model_vector(37)];
        let query_gauge = encode(&mut atlas, 2, &query);
        let keys_gauge = keys
            .iter()
            .enumerate()
            .map(|(source, key)| {
                transport(
                    &mut atlas,
                    source,
                    2,
                    key,
                    R4SpinTransportIntervention::Coherent,
                    false,
                )
            })
            .collect::<Vec<_>>();
        let values_gauge = values
            .iter()
            .enumerate()
            .map(|(source, value)| {
                transport(
                    &mut atlas,
                    source,
                    2,
                    value,
                    R4SpinTransportIntervention::Coherent,
                    true,
                )
            })
            .collect::<Vec<_>>();
        let model_logits = keys
            .iter()
            .map(|key| {
                helm_d_learned_manifold_logit(
                    HelmDLearnedManifoldMetric::Lorentz,
                    &query,
                    key,
                    24.0,
                    0.0,
                )
                .expect("model logit")
            })
            .collect::<Vec<_>>();
        let gauge_logits = keys_gauge
            .iter()
            .map(|key| {
                helm_d_learned_manifold_logit(
                    HelmDLearnedManifoldMetric::Lorentz,
                    &query_gauge,
                    key,
                    24.0,
                    0.0,
                )
                .expect("gauge logit")
            })
            .collect::<Vec<_>>();
        for (model, gauge) in model_logits.iter().zip(&gauge_logits) {
            assert!((*model - *gauge).abs() <= 1.0e-12);
        }
        let weights = stable_softmax(&model_logits).expect("weights");
        let model_centroid = helm_d_learned_manifold_centroid(
            HelmDLearnedManifoldMetric::Lorentz,
            &values,
            &weights,
        )
        .expect("model centroid");
        let gauge_centroid = helm_d_learned_manifold_centroid(
            HelmDLearnedManifoldMetric::Lorentz,
            &values_gauge,
            &weights,
        )
        .expect("gauge centroid");
        let decoded = decode(&mut atlas, 2, &gauge_centroid);
        let maximum_error = decoded
            .iter()
            .zip(&model_centroid)
            .map(|(left, right)| (*left - *right).abs())
            .fold(0.0, f64::max);
        assert!(maximum_error <= 1.0e-8, "covariance error {maximum_error}");

        let mismatched = transport(
            &mut atlas,
            0,
            2,
            &keys[0],
            R4SpinTransportIntervention::SourceFramePermuted,
            false,
        );
        let mismatch_logit = helm_d_learned_manifold_logit(
            HelmDLearnedManifoldMetric::Lorentz,
            &query_gauge,
            &mismatched,
            24.0,
            0.0,
        )
        .expect("mismatch logit");
        assert!((mismatch_logit - model_logits[0]).abs() > 1.0e-6);
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_rejects_invalid_parameters() {
        let mut parameters =
            HelmDLearnedManifoldParameters::identity(1, 9, 3, 16, 24.0).expect("parameters");
        parameters.learned_scales_mut()[0] = 0.0;
        assert!(parameters.validate().is_err());

        let mut parameters =
            HelmDLearnedManifoldParameters::identity(1, 9, 3, 16, 24.0).expect("parameters");
        parameters.query_adapters_mut()[0].matrix[0][0] = f64::NAN;
        assert!(parameters.validate().is_err());
        assert!(HelmDLearnedManifoldParameters::identity(1, 8, 3, 16, 24.0).is_err());
    }

    #[test]
    fn helm_d_learned_manifold_r4_construction_order_key_control_is_live() {
        let parameters =
            HelmDLearnedManifoldParameters::identity(1, 1, 1, 16, 24.0).expect("parameters");
        let mut coherent = HelmDLearnedManifoldR4Transport::new(
            32,
            3,
            parameters.clone(),
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldIntervention::Coherent,
        )
        .expect("coherent");
        let mut shuffled = HelmDLearnedManifoldR4Transport::new(
            32,
            3,
            parameters,
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldIntervention::OrderKeyShuffled,
        )
        .expect("shuffled");
        let query = (0..64)
            .map(|lane| (lane as f32 * 0.013).sin())
            .collect::<Vec<_>>();
        let packed_keys = (0..3)
            .flat_map(|source| (0..64).map(move |lane| ((source * 19 + lane) as f32 * 0.021).cos()))
            .collect::<Vec<_>>();
        let context = CausalAttentionHeadContext {
            layer: 0,
            head: 0,
            query_position: 2,
        };
        let mut coherent_weights = [0.0_f32; 3];
        let mut shuffled_weights = [0.0_f32; 3];
        coherent.score_and_normalize(context, &query, &packed_keys, &mut coherent_weights, true);
        shuffled.score_and_normalize(context, &query, &packed_keys, &mut shuffled_weights, true);
        assert_ne!(coherent_weights, shuffled_weights);
        assert_eq!(shuffled.audit().order_key_permutations, 3);
        assert!(CausalAttentionTransport::status(&shuffled).is_ok());
    }
}
