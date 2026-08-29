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
    CausalAttentionHeadContext, CausalAttentionSourceContext, CausalAttentionTransport,
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
        self.frames[position] = Some(frame);
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
        h4_left_quaternion_matrix(self.frame(position)?, &self.exact_route_table)
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
}
