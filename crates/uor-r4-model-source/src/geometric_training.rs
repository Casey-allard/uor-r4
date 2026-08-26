//! Mixer-specific bounded training for the one-layer geometric decoder (#951).
//!
//! This is deliberately not a general autograd or optimizer framework.  It
//! differentiates only the four projections, two coordinate biases, bounded
//! support softmax, and output gain already owned by [`GeometricMixer`].  Every
//! dense forward and gradient product is executed by the pinned `uor-matmul`
//! dependency. Source-model values are exposed only as immutable trace inputs.

use std::collections::HashSet;
use std::io;
use std::num::NonZeroUsize;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::geometric_decoder::{
    GeometricMixer, GeometricMixerParameters, GeometryIntervention, R4_COORDINATE_WIDTH,
};
use crate::{
    matmul, ExactExecutor, HuggingFaceLlamaOracle, SourceUnavailable, State,
    TeacherExecutionConfig, UOR_MATMUL_REVISION,
};

pub const CHECKPOINT_SCHEMA: &str = "uor-r4.geometric-mixer-checkpoint/2";
pub const PREFLIGHT_SCHEMA: &str = "uor-r4.geometric-mixer-preflight/1";
pub const LOSS_FORMULA: &str = "0.55 * mean_squared(operator-target)/(mean_square(target)+1e-6) + 0.25 * sampled_next_token_cross_entropy/ln(16) + 0.20 * support_cross_entropy/ln(candidate_count)";
pub const GRADIENT_CHECK_EPSILON: f32 = 1.0e-3;
pub const GRADIENT_CHECK_ABSOLUTE_TOLERANCE: f32 = 2.0e-3;
pub const GRADIENT_CHECK_RELATIVE_TOLERANCE: f32 = 2.0e-2;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingPrefixKind {
    Teacher,
    Student,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingSupportSource {
    Prefix,
    Memory,
}

impl TrainingSupportSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prefix => "prefix",
            Self::Memory => "memory",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrainingSupportTarget {
    pub source: TrainingSupportSource,
    pub index: usize,
    pub probability: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrainingMemoryInput {
    pub span_index: usize,
    pub mean_embedding: Vec<f32>,
    pub r4_coordinates: [f32; R4_COORDINATE_WIDTH],
}

/// One already-frozen source trace point. Large residual/embedding vectors are
/// kept in memory for fitting and are intentionally absent from the retained
/// report/checkpoint.
#[derive(Clone, Debug)]
pub struct MixerTrainingExample {
    pub id: String,
    pub prefix_kind: TrainingPrefixKind,
    /// Layer-normalized residuals for positions `0..=position`.
    pub normalized_prefix: Vec<Vec<f32>>,
    pub session_route_state: [f32; R4_COORDINATE_WIDTH],
    pub memories: Vec<TrainingMemoryInput>,
    pub target_attention_output: Vec<f32>,
    pub support_target: Vec<TrainingSupportTarget>,
    /// Row zero is the true next-token embedding; the remaining rows are the
    /// deterministic matched negatives.
    pub next_token_candidate_embeddings: Vec<Vec<f32>>,
}

impl MixerTrainingExample {
    pub fn validate(&self, source_width: usize) -> Result<(), GeometricTrainingError> {
        if self.id.trim().is_empty()
            || self.normalized_prefix.is_empty()
            || self.target_attention_output.len() != source_width
            || self
                .normalized_prefix
                .iter()
                .any(|residual| residual.len() != source_width)
            || self
                .memories
                .iter()
                .any(|memory| memory.mean_embedding.len() != source_width)
            || self.next_token_candidate_embeddings.len() < 2
            || self
                .next_token_candidate_embeddings
                .iter()
                .any(|embedding| embedding.len() != source_width)
        {
            return Err(GeometricTrainingError::InvalidExample(self.id.clone()));
        }
        let candidate_count = self.normalized_prefix.len() + self.memories.len();
        if candidate_count < 2 || self.support_target.is_empty() {
            return Err(GeometricTrainingError::InvalidSupportTarget(
                self.id.clone(),
            ));
        }
        let mut probability_sum = 0.0f32;
        let mut seen = HashSet::new();
        for target in &self.support_target {
            let valid_index = match target.source {
                TrainingSupportSource::Prefix => target.index < self.normalized_prefix.len(),
                TrainingSupportSource::Memory => target.index < self.memories.len(),
            };
            if !valid_index
                || !target.probability.is_finite()
                || target.probability <= 0.0
                || !seen.insert((target.source, target.index))
            {
                return Err(GeometricTrainingError::InvalidSupportTarget(
                    self.id.clone(),
                ));
            }
            probability_sum += target.probability;
        }
        if (probability_sum - 1.0).abs() > 1.0e-4 {
            return Err(GeometricTrainingError::InvalidSupportTarget(
                self.id.clone(),
            ));
        }
        if !self
            .normalized_prefix
            .iter()
            .flatten()
            .chain(self.target_attention_output.iter())
            .chain(self.session_route_state.iter())
            .chain(self.memories.iter().flat_map(|memory| {
                memory
                    .mean_embedding
                    .iter()
                    .chain(memory.r4_coordinates.iter())
            }))
            .chain(self.next_token_candidate_embeddings.iter().flatten())
            .all(|value| value.is_finite())
        {
            return Err(GeometricTrainingError::NonFiniteExample(self.id.clone()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixerTrainingConfig {
    pub seed: u64,
    pub learning_rate: f32,
    pub maximum_gradient_norm: f32,
    pub operator_weight: f32,
    pub next_token_weight: f32,
    pub support_weight: f32,
    pub sampled_token_candidates: usize,
    pub loss_formula: String,
}

impl MixerTrainingConfig {
    pub fn issue_951(seed: u64) -> Self {
        Self {
            seed,
            learning_rate: 0.035,
            maximum_gradient_norm: 2.0,
            operator_weight: 0.55,
            next_token_weight: 0.25,
            support_weight: 0.20,
            sampled_token_candidates: 16,
            loss_formula: LOSS_FORMULA.to_owned(),
        }
    }

    fn validate(&self) -> Result<(), GeometricTrainingError> {
        let total = self.operator_weight + self.next_token_weight + self.support_weight;
        if self.operator_weight.to_bits() != 0.55f32.to_bits()
            || self.next_token_weight.to_bits() != 0.25f32.to_bits()
            || self.support_weight.to_bits() != 0.20f32.to_bits()
            || !self.learning_rate.is_finite()
            || self.learning_rate <= 0.0
            || !self.maximum_gradient_norm.is_finite()
            || self.maximum_gradient_norm <= 0.0
            || (total - 1.0).abs() > 1.0e-6
            || self.sampled_token_candidates < 2
            || self.loss_formula != LOSS_FORMULA
        {
            return Err(GeometricTrainingError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricMixerCheckpointBinding {
    pub source_cid: String,
    pub tokenizer_cid: String,
    pub base_checkpoint_identity: String,
    pub dataset_cid: String,
    pub seed: u64,
    pub training_config: MixerTrainingConfig,
    pub projection_owner: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricMixerCheckpoint {
    pub schema: String,
    pub binding: GeometricMixerCheckpointBinding,
    pub mixer: GeometricMixer,
    pub content_digest: String,
}

#[derive(Serialize)]
struct CheckpointContent<'a> {
    schema: &'a str,
    binding: &'a GeometricMixerCheckpointBinding,
    mixer: &'a GeometricMixer,
}

impl GeometricMixerCheckpoint {
    pub fn new(
        binding: GeometricMixerCheckpointBinding,
        mixer: GeometricMixer,
    ) -> Result<Self, GeometricTrainingError> {
        mixer
            .validate()
            .map_err(|error| GeometricTrainingError::InvalidCheckpoint(error.to_string()))?;
        validate_checkpoint_binding(&binding)?;
        let content_digest = checkpoint_digest(&binding, &mixer)?;
        Ok(Self {
            schema: CHECKPOINT_SCHEMA.to_owned(),
            binding,
            mixer,
            content_digest,
        })
    }

    pub fn validate(&self) -> Result<(), GeometricTrainingError> {
        if self.schema != CHECKPOINT_SCHEMA {
            return Err(GeometricTrainingError::InvalidCheckpoint(format!(
                "schema {} != {CHECKPOINT_SCHEMA}",
                self.schema
            )));
        }
        let expected = checkpoint_digest(&self.binding, &self.mixer)?;
        if expected != self.content_digest {
            return Err(GeometricTrainingError::CheckpointDigestMismatch {
                expected,
                actual: self.content_digest.clone(),
            });
        }
        self.mixer
            .validate()
            .map_err(|error| GeometricTrainingError::InvalidCheckpoint(error.to_string()))?;
        validate_checkpoint_binding(&self.binding)
    }

    pub fn save(&self, path: &Path) -> Result<(), GeometricTrainingError> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, GeometricTrainingError> {
        let bytes = std::fs::read(path)?;
        let checkpoint: Self = serde_json::from_slice(&bytes)
            .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
        checkpoint.validate()?;
        Ok(checkpoint)
    }
}

fn validate_checkpoint_binding(
    binding: &GeometricMixerCheckpointBinding,
) -> Result<(), GeometricTrainingError> {
    binding.training_config.validate()?;
    if binding.source_cid.trim().is_empty()
        || binding.tokenizer_cid.trim().is_empty()
        || binding.base_checkpoint_identity.trim().is_empty()
        || binding.dataset_cid.trim().is_empty()
        || binding.projection_owner != format!("uor-matmul exact GEMM@{UOR_MATMUL_REVISION}")
    {
        return Err(GeometricTrainingError::InvalidCheckpoint(
            "checkpoint binding is incomplete or names the wrong projection owner".to_owned(),
        ));
    }
    Ok(())
}

fn checkpoint_digest(
    binding: &GeometricMixerCheckpointBinding,
    mixer: &GeometricMixer,
) -> Result<String, GeometricTrainingError> {
    let bytes = serde_json::to_vec(&CheckpointContent {
        schema: CHECKPOINT_SCHEMA,
        binding,
        mixer,
    })
    .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MixerLossSummary {
    pub examples: usize,
    pub total: f64,
    pub operator_alignment: f64,
    pub sampled_next_token: f64,
    pub support: f64,
    pub distinct_selected_prefix_positions: usize,
    pub distinct_selected_memories: usize,
    pub mean_target_memory_probability: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixerTrainingRound {
    pub requested_steps: usize,
    pub completed_steps: usize,
    pub wall_seconds: f64,
    pub wall_limit_seconds: f64,
    pub initial: MixerLossSummary,
    pub final_loss: MixerLossSummary,
    pub reduction_fraction: f64,
    pub maximum_observed_gradient_norm: f64,
    pub clipped_steps: usize,
    pub terminal_status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientCheckResult {
    pub parameter: String,
    pub analytical: f64,
    pub finite_difference: f64,
    pub absolute_error: f64,
    pub allowed_error: f64,
    pub epsilon: f64,
    pub verdict: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TinyOverfitResult {
    pub loss_name: String,
    pub examples: usize,
    pub maximum_examples: usize,
    pub requested_steps: usize,
    pub maximum_steps: usize,
    pub completed_steps: usize,
    pub initial_loss: f64,
    pub final_loss: f64,
    pub reduction_fraction: f64,
    pub required_reduction_fraction: f64,
    pub verdict: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRoundTripResult {
    pub digest_before: String,
    pub digest_after: String,
    pub digest_preserved: bool,
    pub focused_output_bit_identical: bool,
    pub verdict: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MixerPreflightReport {
    pub schema: String,
    pub seed: u64,
    pub source_trace_opened: bool,
    pub tiny_overfit: TinyOverfitResult,
    pub gradient_check: GradientCheckResult,
    pub checkpoint_round_trip: CheckpointRoundTripResult,
    pub verdict: String,
    pub report_digest: String,
}

#[derive(Debug)]
pub enum GeometricTrainingError {
    Io(io::Error),
    Source(String),
    InvalidConfiguration,
    InvalidExample(String),
    NonFiniteExample(String),
    InvalidSupportTarget(String),
    InvalidCheckpoint(String),
    CheckpointDigestMismatch { expected: String, actual: String },
    Serialization(String),
    MatrixProduct(String),
    EmptyDataset,
    WallClockExceeded,
    PreflightFailed(String),
}

impl std::fmt::Display for GeometricTrainingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Source(reason) => write!(formatter, "source trace unavailable: {reason}"),
            Self::InvalidConfiguration => {
                write!(formatter, "invalid bounded mixer training configuration")
            }
            Self::InvalidExample(id) => write!(formatter, "invalid mixer training example {id}"),
            Self::NonFiniteExample(id) => {
                write!(formatter, "non-finite mixer training example {id}")
            }
            Self::InvalidSupportTarget(id) => write!(formatter, "invalid support target for {id}"),
            Self::InvalidCheckpoint(reason) => {
                write!(formatter, "invalid mixer checkpoint: {reason}")
            }
            Self::CheckpointDigestMismatch { expected, actual } => write!(
                formatter,
                "checkpoint digest {actual} != recomputed {expected}"
            ),
            Self::Serialization(reason) => {
                write!(formatter, "mixer serialization failed: {reason}")
            }
            Self::MatrixProduct(reason) => write!(formatter, "uor-matmul product failed: {reason}"),
            Self::EmptyDataset => write!(formatter, "mixer training dataset is empty"),
            Self::WallClockExceeded => write!(
                formatter,
                "bounded mixer fitting reached its wall-clock limit"
            ),
            Self::PreflightFailed(reason) => write!(formatter, "mixer preflight failed: {reason}"),
        }
    }
}

impl std::error::Error for GeometricTrainingError {}

impl From<io::Error> for GeometricTrainingError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SourceUnavailable> for GeometricTrainingError {
    fn from(error: SourceUnavailable) -> Self {
        Self::Source(error.to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceLayerCapture {
    pub base_residual: Vec<f32>,
    pub normalized_residual: Vec<f32>,
    pub attention_output: Vec<f32>,
    pub q: Vec<f32>,
    pub k: Vec<f32>,
    pub v: Vec<f32>,
    pub mean_attention_support: Vec<f32>,
    pub logits: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct SourceTrainingTracePoint {
    pub position: usize,
    pub base_residual: Vec<f32>,
    pub normalized_residual: Vec<f32>,
    pub attention_output: Vec<f32>,
    pub mean_attention_support: Vec<f32>,
    pub logits: Vec<f32>,
    pub q_cid: String,
    pub k_cid: String,
    pub v_cid: String,
}

impl HuggingFaceLlamaOracle {
    /// Capture the already-versioned residual/Q/K/V/attention/logit surfaces
    /// plus the source attention output at one declared layer. The source
    /// state is private and newly allocated; no source parameter can be
    /// reached by the returned values.
    pub fn capture_geometric_training_sequence(
        &self,
        tokens: &[u32],
        layer: usize,
    ) -> Result<Vec<SourceTrainingTracePoint>, GeometricTrainingError> {
        if tokens.len() < 2 || layer >= self.model.cfg.n_layers {
            return Err(GeometricTrainingError::Source(
                "trace sequence or target layer is outside the source bounds".to_owned(),
            ));
        }
        let mut state = State::new_bounded(&self.model.cfg, tokens.len())
            .map_err(|error| GeometricTrainingError::Source(error.to_string()))?;
        let mut points = Vec::with_capacity(tokens.len() - 1);
        for (position, &token) in tokens[..tokens.len() - 1].iter().enumerate() {
            let token = usize::try_from(token).map_err(|_| {
                GeometricTrainingError::Source("token does not fit usize".to_owned())
            })?;
            if token >= self.model.cfg.vocab {
                return Err(GeometricTrainingError::Source(format!(
                    "token {token} is outside source vocabulary {}",
                    self.model.cfg.vocab
                )));
            }
            let mut capture = SourceLayerCapture::default();
            self.model.forward_capturing_geometric_source(
                &mut state,
                token,
                position,
                self.fast_matmul,
                layer,
                &mut capture,
            );
            points.push(SourceTrainingTracePoint {
                position,
                base_residual: capture.base_residual,
                normalized_residual: capture.normalized_residual,
                attention_output: capture.attention_output,
                mean_attention_support: capture.mean_attention_support,
                logits: capture.logits,
                q_cid: vector_cid("q", &capture.q),
                k_cid: vector_cid("k", &capture.k),
                v_cid: vector_cid("v", &capture.v),
            });
        }
        Ok(points)
    }

    /// Copy exact source-width embedding rows without applying the historical
    /// compiled-geometry projection.
    pub fn source_embedding_rows(
        &self,
        tokens: &[u32],
    ) -> Result<Vec<Vec<f32>>, GeometricTrainingError> {
        let dim = self.model.cfg.dim;
        let mut rows = Vec::with_capacity(tokens.len());
        for &token in tokens {
            let token = usize::try_from(token).map_err(|_| {
                GeometricTrainingError::Source("token does not fit usize".to_owned())
            })?;
            if token >= self.model.cfg.vocab {
                return Err(GeometricTrainingError::Source(format!(
                    "token {token} is outside source vocabulary {}",
                    self.model.cfg.vocab
                )));
            }
            rows.push(
                self.model.w[self.model.emb + token * dim..self.model.emb + (token + 1) * dim]
                    .to_vec(),
            );
        }
        Ok(rows)
    }
}

fn vector_cid(label: &str, values: &[f32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.geometric-training-trace-vector/1");
    hasher.update(label.as_bytes());
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

#[derive(Clone)]
struct CandidateCache {
    source: TrainingSupportSource,
    index: usize,
    input: Vec<f32>,
    raw_key: [f32; R4_COORDINATE_WIDTH],
    key: [f32; R4_COORDINATE_WIDTH],
    effective_key: [f32; R4_COORDINATE_WIDTH],
    score: f32,
    support_probability: f32,
    selected_weight: f32,
    value: Option<Vec<f32>>,
}

struct ForwardCache {
    raw_query: [f32; R4_COORDINATE_WIDTH],
    query: [f32; R4_COORDINATE_WIDTH],
    candidates: Vec<CandidateCache>,
    aggregate: Vec<f32>,
    ungained_output: Vec<f32>,
    output: Vec<f32>,
    token_probabilities: Vec<f32>,
    target_probabilities: Vec<f32>,
    operator_loss: f32,
    token_loss: f32,
    support_loss: f32,
    total_loss: f32,
}

#[derive(Clone)]
struct ParameterGradient {
    query_projection: Vec<f32>,
    key_projection: Vec<f32>,
    value_projection: Vec<f32>,
    output_projection: Vec<f32>,
    query_bias: [f32; R4_COORDINATE_WIDTH],
    key_bias: [f32; R4_COORDINATE_WIDTH],
    output_gain: f32,
}

impl ParameterGradient {
    fn zero(mixer: &GeometricMixer) -> Self {
        Self {
            query_projection: vec![0.0; mixer.parameters.query_projection.len()],
            key_projection: vec![0.0; mixer.parameters.key_projection.len()],
            value_projection: vec![0.0; mixer.parameters.value_projection.len()],
            output_projection: vec![0.0; mixer.parameters.output_projection.len()],
            query_bias: [0.0; R4_COORDINATE_WIDTH],
            key_bias: [0.0; R4_COORDINATE_WIDTH],
            output_gain: 0.0,
        }
    }

    fn scale(&mut self, scale: f32) {
        for value in self
            .query_projection
            .iter_mut()
            .chain(self.key_projection.iter_mut())
            .chain(self.value_projection.iter_mut())
            .chain(self.output_projection.iter_mut())
            .chain(self.query_bias.iter_mut())
            .chain(self.key_bias.iter_mut())
            .chain(std::iter::once(&mut self.output_gain))
        {
            *value *= scale;
        }
    }

    fn norm(&self) -> f32 {
        self.query_projection
            .iter()
            .chain(self.key_projection.iter())
            .chain(self.value_projection.iter())
            .chain(self.output_projection.iter())
            .chain(self.query_bias.iter())
            .chain(self.key_bias.iter())
            .chain(std::iter::once(&self.output_gain))
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt()
    }
}

/// Fixed, mixer-only trainer. It owns no source weights and exposes no generic
/// parameter graph, optimizer state, or arbitrary operation registration.
pub struct MixerSpecificTrainer {
    mixer: GeometricMixer,
    config: MixerTrainingConfig,
    executor: ExactExecutor,
}

impl MixerSpecificTrainer {
    pub fn new(
        mixer: GeometricMixer,
        config: MixerTrainingConfig,
        workers: NonZeroUsize,
    ) -> Result<Self, GeometricTrainingError> {
        mixer
            .validate()
            .map_err(|error| GeometricTrainingError::InvalidCheckpoint(error.to_string()))?;
        config.validate()?;
        let executor = ExactExecutor::new(TeacherExecutionConfig::fixed_workers(workers))?;
        Ok(Self {
            mixer,
            config,
            executor,
        })
    }

    pub fn mixer(&self) -> &GeometricMixer {
        &self.mixer
    }

    pub fn into_mixer(self) -> GeometricMixer {
        self.mixer
    }

    pub fn evaluate(
        &self,
        examples: &[MixerTrainingExample],
        intervention: GeometryIntervention,
    ) -> Result<MixerLossSummary, GeometricTrainingError> {
        if examples.is_empty() {
            return Err(GeometricTrainingError::EmptyDataset);
        }
        let mut accumulator = LossAccumulator::default();
        for example in examples {
            example.validate(self.mixer.source_width)?;
            if example.next_token_candidate_embeddings.len() != self.config.sampled_token_candidates
            {
                return Err(GeometricTrainingError::InvalidExample(example.id.clone()));
            }
            let cache = forward_example(
                &self.mixer,
                &self.config,
                example,
                intervention,
                &self.executor,
            )?;
            accumulator.push(example, &cache);
        }
        Ok(accumulator.finish())
    }

    /// Matched source-attention reference for the same sampled-token rows.
    /// Operator alignment and support error are zero by definition because
    /// this arm retains the exact source attention output/support target.
    pub fn evaluate_disabled_reference(
        &self,
        examples: &[MixerTrainingExample],
    ) -> Result<MixerLossSummary, GeometricTrainingError> {
        if examples.is_empty() {
            return Err(GeometricTrainingError::EmptyDataset);
        }
        let mut token_sum = 0.0f64;
        let mut prefix = HashSet::new();
        let mut memory = HashSet::new();
        for example in examples {
            example.validate(self.mixer.source_width)?;
            let mut matrix = Vec::with_capacity(
                example.next_token_candidate_embeddings.len() * self.mixer.source_width,
            );
            for embedding in &example.next_token_candidate_embeddings {
                matrix.extend_from_slice(embedding);
            }
            let mut logits = vec![0.0f32; example.next_token_candidate_embeddings.len()];
            matmul(
                &self.executor,
                &mut logits,
                &example.target_attention_output,
                &matrix,
                self.mixer.source_width,
                true,
            );
            let scale = (self.mixer.source_width as f32).sqrt().max(1.0);
            for logit in &mut logits {
                *logit /= scale;
            }
            let probabilities = softmax(&logits);
            token_sum += f64::from(
                -probabilities[0].max(f32::MIN_POSITIVE).ln()
                    / (probabilities.len() as f32).ln().max(1.0),
            );
            for target in &example.support_target {
                match target.source {
                    TrainingSupportSource::Prefix => {
                        prefix.insert(target.index);
                    }
                    TrainingSupportSource::Memory => {
                        memory.insert(target.index);
                    }
                }
            }
        }
        let divisor = examples.len() as f64;
        let token = token_sum / divisor;
        Ok(MixerLossSummary {
            examples: examples.len(),
            total: f64::from(self.config.next_token_weight) * token,
            operator_alignment: 0.0,
            sampled_next_token: token,
            support: 0.0,
            distinct_selected_prefix_positions: prefix.len(),
            distinct_selected_memories: memory.len(),
            mean_target_memory_probability: None,
        })
    }

    pub fn train(
        &mut self,
        examples: &[MixerTrainingExample],
        requested_steps: usize,
        wall_limit: Duration,
    ) -> Result<MixerTrainingRound, GeometricTrainingError> {
        if examples.is_empty() {
            return Err(GeometricTrainingError::EmptyDataset);
        }
        if requested_steps == 0 || wall_limit.is_zero() {
            return Err(GeometricTrainingError::InvalidConfiguration);
        }
        for example in examples {
            example.validate(self.mixer.source_width)?;
            if example.next_token_candidate_embeddings.len() != self.config.sampled_token_candidates
            {
                return Err(GeometricTrainingError::InvalidExample(example.id.clone()));
            }
        }
        let initial = self.evaluate(examples, GeometryIntervention::Real)?;
        let started = Instant::now();
        let mut completed_steps = 0usize;
        let mut maximum_observed_gradient_norm = 0.0f32;
        let mut clipped_steps = 0usize;
        let mut terminal_status = "COMPLETED".to_owned();
        for step in 0..requested_steps {
            if started.elapsed() >= wall_limit {
                terminal_status = "WALL_LIMIT_REACHED".to_owned();
                break;
            }
            let mut gradient = ParameterGradient::zero(&self.mixer);
            for example in examples {
                let cache = forward_example(
                    &self.mixer,
                    &self.config,
                    example,
                    GeometryIntervention::Real,
                    &self.executor,
                )?;
                backward_example(
                    &self.mixer,
                    &self.config,
                    example,
                    &cache,
                    &self.executor,
                    &mut gradient,
                )?;
            }
            gradient.scale(1.0 / examples.len() as f32);
            let norm = gradient.norm();
            maximum_observed_gradient_norm = maximum_observed_gradient_norm.max(norm);
            if norm > self.config.maximum_gradient_norm {
                gradient.scale(self.config.maximum_gradient_norm / norm);
                clipped_steps += 1;
            }
            let schedule = 1.0 / (1.0 + step as f32 / 64.0).sqrt();
            apply_gradient(
                &mut self.mixer.parameters,
                &gradient,
                self.config.learning_rate * schedule,
            );
            self.mixer
                .validate()
                .map_err(|error| GeometricTrainingError::InvalidCheckpoint(error.to_string()))?;
            completed_steps += 1;
        }
        let final_loss = self.evaluate(examples, GeometryIntervention::Real)?;
        let reduction_fraction = if initial.total > f64::EPSILON {
            (initial.total - final_loss.total) / initial.total
        } else {
            0.0
        };
        Ok(MixerTrainingRound {
            requested_steps,
            completed_steps,
            wall_seconds: started.elapsed().as_secs_f64(),
            wall_limit_seconds: wall_limit.as_secs_f64(),
            initial,
            final_loss,
            reduction_fraction,
            maximum_observed_gradient_norm: f64::from(maximum_observed_gradient_norm),
            clipped_steps,
            terminal_status,
        })
    }

    pub fn gradient_check_output_projection_zero(
        &self,
        example: &MixerTrainingExample,
    ) -> Result<GradientCheckResult, GeometricTrainingError> {
        example.validate(self.mixer.source_width)?;
        let cache = forward_example(
            &self.mixer,
            &self.config,
            example,
            GeometryIntervention::Real,
            &self.executor,
        )?;
        let mut gradient = ParameterGradient::zero(&self.mixer);
        backward_example(
            &self.mixer,
            &self.config,
            example,
            &cache,
            &self.executor,
            &mut gradient,
        )?;
        let analytical = gradient.output_projection[0];
        let mut plus = self.mixer.clone();
        plus.parameters.output_projection[0] += GRADIENT_CHECK_EPSILON;
        let plus_loss = forward_example(
            &plus,
            &self.config,
            example,
            GeometryIntervention::Real,
            &self.executor,
        )?
        .total_loss;
        let mut minus = self.mixer.clone();
        minus.parameters.output_projection[0] -= GRADIENT_CHECK_EPSILON;
        let minus_loss = forward_example(
            &minus,
            &self.config,
            example,
            GeometryIntervention::Real,
            &self.executor,
        )?
        .total_loss;
        let finite_difference = (plus_loss - minus_loss) / (2.0 * GRADIENT_CHECK_EPSILON);
        let absolute_error = (analytical - finite_difference).abs();
        let allowed_error = GRADIENT_CHECK_ABSOLUTE_TOLERANCE
            + GRADIENT_CHECK_RELATIVE_TOLERANCE * analytical.abs().max(finite_difference.abs());
        Ok(GradientCheckResult {
            parameter: "output_projection[0]".to_owned(),
            analytical: f64::from(analytical),
            finite_difference: f64::from(finite_difference),
            absolute_error: f64::from(absolute_error),
            allowed_error: f64::from(allowed_error),
            epsilon: f64::from(GRADIENT_CHECK_EPSILON),
            verdict: if absolute_error <= allowed_error {
                "PASS"
            } else {
                "FAIL"
            }
            .to_owned(),
        })
    }
}

#[derive(Default)]
struct LossAccumulator {
    examples: usize,
    total: f64,
    operator: f64,
    token: f64,
    support: f64,
    prefix: HashSet<usize>,
    memory: HashSet<usize>,
    memory_probability_sum: f64,
    memory_probability_count: usize,
}

impl LossAccumulator {
    fn push(&mut self, example: &MixerTrainingExample, cache: &ForwardCache) {
        self.examples += 1;
        self.total += f64::from(cache.total_loss);
        self.operator += f64::from(cache.operator_loss);
        self.token += f64::from(cache.token_loss);
        self.support += f64::from(cache.support_loss);
        for candidate in &cache.candidates {
            if candidate.selected_weight > 0.0 {
                match candidate.source {
                    TrainingSupportSource::Prefix => {
                        self.prefix.insert(candidate.index);
                    }
                    TrainingSupportSource::Memory => {
                        self.memory.insert(candidate.index);
                    }
                }
            }
        }
        for target in &example.support_target {
            if target.source == TrainingSupportSource::Memory {
                let probability = cache
                    .candidates
                    .iter()
                    .find(|candidate| {
                        candidate.source == target.source && candidate.index == target.index
                    })
                    .map(|candidate| candidate.support_probability)
                    .unwrap_or(0.0);
                self.memory_probability_sum += f64::from(probability);
                self.memory_probability_count += 1;
            }
        }
    }

    fn finish(self) -> MixerLossSummary {
        let divisor = self.examples.max(1) as f64;
        MixerLossSummary {
            examples: self.examples,
            total: self.total / divisor,
            operator_alignment: self.operator / divisor,
            sampled_next_token: self.token / divisor,
            support: self.support / divisor,
            distinct_selected_prefix_positions: self.prefix.len(),
            distinct_selected_memories: self.memory.len(),
            mean_target_memory_probability: (self.memory_probability_count > 0).then_some(
                self.memory_probability_sum / self.memory_probability_count.max(1) as f64,
            ),
        }
    }
}

fn apply_gradient(
    parameters: &mut GeometricMixerParameters,
    gradient: &ParameterGradient,
    learning_rate: f32,
) {
    for (value, derivative) in parameters
        .query_projection
        .iter_mut()
        .zip(&gradient.query_projection)
        .chain(
            parameters
                .key_projection
                .iter_mut()
                .zip(&gradient.key_projection),
        )
        .chain(
            parameters
                .value_projection
                .iter_mut()
                .zip(&gradient.value_projection),
        )
        .chain(
            parameters
                .output_projection
                .iter_mut()
                .zip(&gradient.output_projection),
        )
        .chain(parameters.query_bias.iter_mut().zip(&gradient.query_bias))
        .chain(parameters.key_bias.iter_mut().zip(&gradient.key_bias))
    {
        *value -= learning_rate * *derivative;
    }
    parameters.output_gain -= learning_rate * gradient.output_gain;
}

fn forward_example(
    mixer: &GeometricMixer,
    config: &MixerTrainingConfig,
    example: &MixerTrainingExample,
    intervention: GeometryIntervention,
    executor: &ExactExecutor,
) -> Result<ForwardCache, GeometricTrainingError> {
    let current_position = example.normalized_prefix.len() - 1;
    let current = &example.normalized_prefix[current_position];
    let mut raw_query = [0.0f32; R4_COORDINATE_WIDTH];
    matmul(
        executor,
        &mut raw_query,
        current,
        &mixer.parameters.query_projection,
        mixer.source_width,
        true,
    );
    for (lane, value) in raw_query.iter_mut().enumerate() {
        *value += mixer.parameters.query_bias[lane] + 0.10 * example.session_route_state[lane];
    }
    let query = normalize4(raw_query);

    let mut candidates = Vec::with_capacity(
        example
            .normalized_prefix
            .len()
            .saturating_add(example.memories.len()),
    );
    for (position, residual) in example.normalized_prefix.iter().enumerate() {
        let mut raw_key = [0.0f32; R4_COORDINATE_WIDTH];
        matmul(
            executor,
            &mut raw_key,
            residual,
            &mixer.parameters.key_projection,
            mixer.source_width,
            true,
        );
        let phase = position as f32 * 0.173_205_08;
        let positional = [
            phase.sin(),
            phase.cos(),
            (phase * 0.5).sin(),
            (phase * 0.5).cos(),
        ];
        for lane in 0..R4_COORDINATE_WIDTH {
            raw_key[lane] += mixer.parameters.key_bias[lane]
                + 0.10 * example.session_route_state[lane]
                + 0.05 * positional[lane];
        }
        let key = normalize4(raw_key);
        let effective_key = intervene_training_key(key, intervention, false);
        candidates.push(CandidateCache {
            source: TrainingSupportSource::Prefix,
            index: position,
            input: residual.clone(),
            raw_key,
            key,
            effective_key,
            score: compatibility(query, effective_key),
            support_probability: 0.0,
            selected_weight: 0.0,
            value: None,
        });
    }
    for (memory_index, memory) in example.memories.iter().enumerate() {
        let mut raw_key = [0.0f32; R4_COORDINATE_WIDTH];
        matmul(
            executor,
            &mut raw_key,
            &memory.mean_embedding,
            &mixer.parameters.key_projection,
            mixer.source_width,
            true,
        );
        for (lane, value) in raw_key.iter_mut().enumerate() {
            *value += mixer.parameters.key_bias[lane]
                + 0.25 * memory.r4_coordinates[lane]
                + 0.01 * ((memory_index + lane) as f32 + 1.0).sin();
        }
        let key = normalize4(raw_key);
        let effective_key = intervene_training_key(key, intervention, true);
        candidates.push(CandidateCache {
            source: TrainingSupportSource::Memory,
            index: memory_index,
            input: memory.mean_embedding.clone(),
            raw_key,
            key,
            effective_key,
            score: compatibility(query, effective_key),
            support_probability: 0.0,
            selected_weight: 0.0,
            value: None,
        });
    }

    let support_scores = candidates
        .iter()
        .map(|candidate| candidate.score)
        .collect::<Vec<_>>();
    let support_probabilities = softmax(&support_scores);
    for (candidate, probability) in candidates.iter_mut().zip(&support_probabilities) {
        candidate.support_probability = *probability;
    }
    let mut target_probabilities = vec![0.0f32; candidates.len()];
    for target in &example.support_target {
        let index = candidate_offset(target.source, target.index, example.normalized_prefix.len());
        target_probabilities[index] = target.probability;
    }
    let support_log_norm = (candidates.len() as f32).ln().max(1.0);
    let support_loss = candidates
        .iter()
        .zip(&target_probabilities)
        .filter(|(_, target)| **target > 0.0)
        .map(|(candidate, target)| {
            -*target * candidate.support_probability.max(f32::MIN_POSITIVE).ln()
        })
        .sum::<f32>()
        / support_log_norm;

    let mut order = (0..candidates.len()).collect::<Vec<_>>();
    order.sort_by(|&left, &right| {
        candidates[right]
            .score
            .total_cmp(&candidates[left].score)
            .then_with(|| {
                source_order(candidates[left].source).cmp(&source_order(candidates[right].source))
            })
            .then_with(|| candidates[left].index.cmp(&candidates[right].index))
    });
    order.truncate(mixer.support_budget.min(order.len()));
    let selected_scores = order
        .iter()
        .map(|&index| candidates[index].score)
        .collect::<Vec<_>>();
    let selected_weights = softmax(&selected_scores);
    let mut aggregate = vec![0.0f32; mixer.value_width];
    for (&candidate_index, &weight) in order.iter().zip(&selected_weights) {
        let mut value = vec![0.0f32; mixer.value_width];
        matmul(
            executor,
            &mut value,
            &candidates[candidate_index].input,
            &mixer.parameters.value_projection,
            mixer.source_width,
            true,
        );
        for (sum, item) in aggregate.iter_mut().zip(&value) {
            *sum += weight * *item;
        }
        candidates[candidate_index].selected_weight = weight;
        candidates[candidate_index].value = Some(value);
    }
    let mut ungained_output = vec![0.0f32; mixer.source_width];
    matmul(
        executor,
        &mut ungained_output,
        &aggregate,
        &mixer.parameters.output_projection,
        mixer.value_width,
        true,
    );
    let output = ungained_output
        .iter()
        .map(|value| value * mixer.parameters.output_gain)
        .collect::<Vec<_>>();

    let target_square_mean = example
        .target_attention_output
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        / mixer.source_width as f32;
    let operator_loss = output
        .iter()
        .zip(&example.target_attention_output)
        .map(|(actual, target)| {
            let delta = actual - target;
            delta * delta
        })
        .sum::<f32>()
        / mixer.source_width as f32
        / (target_square_mean + 1.0e-6);

    let mut token_embedding_matrix =
        Vec::with_capacity(example.next_token_candidate_embeddings.len() * mixer.source_width);
    for embedding in &example.next_token_candidate_embeddings {
        token_embedding_matrix.extend_from_slice(embedding);
    }
    let mut token_logits = vec![0.0f32; example.next_token_candidate_embeddings.len()];
    matmul(
        executor,
        &mut token_logits,
        &output,
        &token_embedding_matrix,
        mixer.source_width,
        true,
    );
    let token_scale = (mixer.source_width as f32).sqrt().max(1.0);
    for logit in &mut token_logits {
        *logit /= token_scale;
    }
    let token_probabilities = softmax(&token_logits);
    let token_log_norm = (token_probabilities.len() as f32).ln().max(1.0);
    let token_loss = -token_probabilities[0].max(f32::MIN_POSITIVE).ln() / token_log_norm;
    let total_loss = config.operator_weight * operator_loss
        + config.next_token_weight * token_loss
        + config.support_weight * support_loss;
    if !total_loss.is_finite() {
        return Err(GeometricTrainingError::NonFiniteExample(example.id.clone()));
    }
    Ok(ForwardCache {
        raw_query,
        query,
        candidates,
        aggregate,
        ungained_output,
        output,
        token_probabilities,
        target_probabilities,
        operator_loss,
        token_loss,
        support_loss,
        total_loss,
    })
}

#[allow(clippy::too_many_arguments)]
fn backward_example(
    mixer: &GeometricMixer,
    config: &MixerTrainingConfig,
    example: &MixerTrainingExample,
    cache: &ForwardCache,
    executor: &ExactExecutor,
    gradient: &mut ParameterGradient,
) -> Result<(), GeometricTrainingError> {
    let mut d_output = vec![0.0f32; mixer.source_width];
    let target_square_mean = example
        .target_attention_output
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        / mixer.source_width as f32;
    let operator_scale =
        config.operator_weight * 2.0 / mixer.source_width as f32 / (target_square_mean + 1.0e-6);
    for ((derivative, actual), target) in d_output
        .iter_mut()
        .zip(&cache.output)
        .zip(&example.target_attention_output)
    {
        *derivative += operator_scale * (actual - target);
    }

    let token_log_norm = (cache.token_probabilities.len() as f32).ln().max(1.0);
    let token_scale = (mixer.source_width as f32).sqrt().max(1.0);
    let mut d_token_logits = cache.token_probabilities.clone();
    d_token_logits[0] -= 1.0;
    for derivative in &mut d_token_logits {
        *derivative *= config.next_token_weight / token_log_norm / token_scale;
    }
    let mut token_embedding_matrix =
        Vec::with_capacity(example.next_token_candidate_embeddings.len() * mixer.source_width);
    for embedding in &example.next_token_candidate_embeddings {
        token_embedding_matrix.extend_from_slice(embedding);
    }
    let token_output_gradient = exact_transpose_matvec(
        executor,
        &token_embedding_matrix,
        example.next_token_candidate_embeddings.len(),
        mixer.source_width,
        &d_token_logits,
    )?;
    for (derivative, token_derivative) in d_output.iter_mut().zip(token_output_gradient) {
        *derivative += token_derivative;
    }

    gradient.output_gain += cache
        .ungained_output
        .iter()
        .zip(&d_output)
        .map(|(value, derivative)| value * derivative)
        .sum::<f32>();
    let d_ungained = d_output
        .iter()
        .map(|derivative| derivative * mixer.parameters.output_gain)
        .collect::<Vec<_>>();
    exact_outer_add(
        &d_ungained,
        &cache.aggregate,
        &mut gradient.output_projection,
    )?;
    let d_aggregate = exact_transpose_matvec(
        executor,
        &mixer.parameters.output_projection,
        mixer.source_width,
        mixer.value_width,
        &d_ungained,
    )?;

    let support_log_norm = (cache.candidates.len() as f32).ln().max(1.0);
    let mut d_scores = cache
        .candidates
        .iter()
        .zip(&cache.target_probabilities)
        .map(|(candidate, target)| {
            config.support_weight * (candidate.support_probability - target) / support_log_norm
        })
        .collect::<Vec<_>>();
    let selected_weighted_value_gradient = cache
        .candidates
        .iter()
        .filter_map(|candidate| {
            candidate.value.as_ref().map(|value| {
                candidate.selected_weight
                    * d_aggregate
                        .iter()
                        .zip(value)
                        .map(|(left, right)| left * right)
                        .sum::<f32>()
            })
        })
        .sum::<f32>();
    for (candidate_index, candidate) in cache.candidates.iter().enumerate() {
        let Some(value) = &candidate.value else {
            continue;
        };
        let d_weight = d_aggregate
            .iter()
            .zip(value)
            .map(|(left, right)| left * right)
            .sum::<f32>();
        d_scores[candidate_index] += candidate.selected_weight * d_weight
            - candidate.selected_weight * selected_weighted_value_gradient;
        let d_value = d_aggregate
            .iter()
            .map(|derivative| derivative * candidate.selected_weight)
            .collect::<Vec<_>>();
        exact_outer_add(&d_value, &candidate.input, &mut gradient.value_projection)?;
    }

    let mut d_query = [0.0f32; R4_COORDINATE_WIDTH];
    for (candidate, &d_score) in cache.candidates.iter().zip(&d_scores) {
        let (query_factor, key_factor) =
            compatibility_gradient(cache.query, candidate.effective_key, d_score);
        for lane in 0..R4_COORDINATE_WIDTH {
            d_query[lane] += query_factor[lane];
        }
        let d_key = undo_intervention_gradient(
            key_factor,
            candidate.source,
            cache_intervention_key_was_permuted(candidate),
        );
        let d_raw_key = normalize_gradient(candidate.raw_key, candidate.key, d_key);
        exact_outer_add(&d_raw_key, &candidate.input, &mut gradient.key_projection)?;
        for (bias, derivative) in gradient.key_bias.iter_mut().zip(d_raw_key) {
            *bias += derivative;
        }
    }
    let d_raw_query = normalize_gradient(cache.raw_query, cache.query, d_query);
    let current = example
        .normalized_prefix
        .last()
        .ok_or(GeometricTrainingError::EmptyDataset)?;
    exact_outer_add(&d_raw_query, current, &mut gradient.query_projection)?;
    for (bias, derivative) in gradient.query_bias.iter_mut().zip(d_raw_query) {
        *bias += derivative;
    }
    Ok(())
}

fn candidate_offset(source: TrainingSupportSource, index: usize, prefix_count: usize) -> usize {
    match source {
        TrainingSupportSource::Prefix => index,
        TrainingSupportSource::Memory => prefix_count + index,
    }
}

fn source_order(source: TrainingSupportSource) -> u8 {
    match source {
        TrainingSupportSource::Prefix => 0,
        TrainingSupportSource::Memory => 1,
    }
}

fn compatibility(query: [f32; 4], key: [f32; 4]) -> f32 {
    let angular = query
        .iter()
        .zip(key)
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .clamp(-1.0, 1.0);
    angular - 0.25 * angular.acos() / std::f32::consts::PI
}

fn compatibility_gradient(query: [f32; 4], key: [f32; 4], derivative: f32) -> ([f32; 4], [f32; 4]) {
    let angular = query
        .iter()
        .zip(key)
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .clamp(-0.999_9, 0.999_9);
    let angular_derivative = 1.0 + 0.25 / (std::f32::consts::PI * (1.0 - angular * angular).sqrt());
    let scale = derivative * angular_derivative;
    (
        key.map(|value| value * scale),
        query.map(|value| value * scale),
    )
}

fn normalize4(raw: [f32; 4]) -> [f32; 4] {
    let norm = raw.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > f32::MIN_POSITIVE {
        raw.map(|value| value / norm)
    } else {
        [1.0, 0.0, 0.0, 0.0]
    }
}

fn normalize_gradient(raw: [f32; 4], normalized: [f32; 4], derivative: [f32; 4]) -> [f32; 4] {
    let norm = raw.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm <= f32::MIN_POSITIVE {
        return [0.0; 4];
    }
    let projection = derivative
        .iter()
        .zip(normalized)
        .map(|(left, right)| left * right)
        .sum::<f32>();
    std::array::from_fn(|lane| (derivative[lane] - normalized[lane] * projection) / norm)
}

fn permute_key(key: [f32; 4]) -> [f32; 4] {
    [key[1], -key[3], key[0], -key[2]]
}

fn intervene_training_key(
    key: [f32; 4],
    intervention: GeometryIntervention,
    memory: bool,
) -> [f32; 4] {
    match intervention {
        GeometryIntervention::Real | GeometryIntervention::Disabled => key,
        GeometryIntervention::PermutedCoordinates => permute_key(key),
        GeometryIntervention::PermutedMemory if memory => permute_key(key),
        GeometryIntervention::PermutedMemory => key,
    }
}

fn cache_intervention_key_was_permuted(_candidate: &CandidateCache) -> bool {
    // Backpropagation is admitted only for the real arm. Permuted arms are
    // evaluation-only matched controls, so their coordinate transform never
    // participates in an update.
    false
}

fn undo_intervention_gradient(
    derivative: [f32; 4],
    _source: TrainingSupportSource,
    permuted: bool,
) -> [f32; 4] {
    if permuted {
        [derivative[2], derivative[0], -derivative[3], -derivative[1]]
    } else {
        derivative
    }
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let maximum = values.iter().copied().max_by(f32::total_cmp).unwrap_or(0.0);
    let mut probabilities = values
        .iter()
        .map(|value| (value - maximum).exp())
        .collect::<Vec<_>>();
    let sum = probabilities.iter().sum::<f32>().max(f32::MIN_POSITIVE);
    for probability in &mut probabilities {
        *probability /= sum;
    }
    probabilities
}

fn exact_outer_add(
    left: &[f32],
    right: &[f32],
    destination: &mut [f32],
) -> Result<(), GeometricTrainingError> {
    if destination.len() != left.len().saturating_mul(right.len()) {
        return Err(GeometricTrainingError::MatrixProduct(
            "outer-product destination shape mismatch".to_owned(),
        ));
    }
    let mut product = vec![0.0f32; destination.len()];
    uor_matmul::slice::gemm_float(
        left.len(),
        1,
        right.len(),
        left,
        right,
        &mut product,
        &mut [],
        &mut [],
    )
    .map_err(|error| GeometricTrainingError::MatrixProduct(error.to_string()))?;
    for (sum, value) in destination.iter_mut().zip(product) {
        *sum += value;
    }
    Ok(())
}

fn exact_transpose_matvec(
    executor: &ExactExecutor,
    matrix: &[f32],
    rows: usize,
    columns: usize,
    vector: &[f32],
) -> Result<Vec<f32>, GeometricTrainingError> {
    if matrix.len() != rows.saturating_mul(columns) || vector.len() != rows {
        return Err(GeometricTrainingError::MatrixProduct(
            "transpose-product input shape mismatch".to_owned(),
        ));
    }
    let mut transposed = vec![0.0f32; matrix.len()];
    for row in 0..rows {
        for column in 0..columns {
            transposed[column * rows + row] = matrix[row * columns + column];
        }
    }
    let mut output = vec![0.0f32; columns];
    matmul(executor, &mut output, vector, &transposed, rows, true);
    Ok(output)
}

fn focused_output_bits(
    trainer: &MixerSpecificTrainer,
    example: &MixerTrainingExample,
) -> Result<Vec<u32>, GeometricTrainingError> {
    Ok(forward_example(
        trainer.mixer(),
        &trainer.config,
        example,
        GeometryIntervention::Real,
        &trainer.executor,
    )?
    .output
    .iter()
    .map(|value| value.to_bits())
    .collect())
}

/// Run all three source-free hard gates. This function never accepts a source
/// path and records `source_trace_opened=false` in its retained report.
pub fn run_mixer_preflight(
    seed: u64,
    report_path: &Path,
    checkpoint_path: &Path,
) -> Result<MixerPreflightReport, GeometricTrainingError> {
    let workers = NonZeroUsize::new(1).ok_or(GeometricTrainingError::InvalidConfiguration)?;
    let config = MixerTrainingConfig {
        learning_rate: 0.055,
        ..MixerTrainingConfig::issue_951(seed)
    };
    let base = GeometricMixer::deterministic(0, 8, b"issue-951-preflight-student")
        .map_err(|error| GeometricTrainingError::InvalidCheckpoint(error.to_string()))?;
    // The preflight target is deliberately reachable by this exact bounded
    // parameterization: identical geometry/value factors with a different
    // output scale. This tests whether the implemented gradient/update can
    // overfit, without conflating that gate with 4-D representation capacity.
    let mut teacher = base.clone();
    teacher.parameters.output_gain = 0.20;
    let examples = synthetic_examples(seed, &teacher, &config, workers)?;
    if examples.len() > 64 {
        return Err(GeometricTrainingError::PreflightFailed(
            "synthetic example cap exceeded".to_owned(),
        ));
    }
    let gradient_trainer = MixerSpecificTrainer::new(base.clone(), config.clone(), workers)?;
    let gradient_check = gradient_trainer.gradient_check_output_projection_zero(&examples[0])?;
    let mut trainer = MixerSpecificTrainer::new(base.clone(), config.clone(), workers)?;
    let round = trainer.train(&examples, 320, Duration::from_secs(60))?;
    let tiny_overfit = TinyOverfitResult {
        loss_name: "normalized operator-output alignment loss on the fixed synthetic batch"
            .to_owned(),
        examples: examples.len(),
        maximum_examples: 64,
        requested_steps: 320,
        maximum_steps: 500,
        completed_steps: round.completed_steps,
        initial_loss: round.initial.operator_alignment,
        final_loss: round.final_loss.operator_alignment,
        reduction_fraction: if round.initial.operator_alignment > f64::EPSILON {
            (round.initial.operator_alignment - round.final_loss.operator_alignment)
                / round.initial.operator_alignment
        } else {
            0.0
        },
        required_reduction_fraction: 0.50,
        verdict: if round.initial.operator_alignment > f64::EPSILON
            && (round.initial.operator_alignment - round.final_loss.operator_alignment)
                / round.initial.operator_alignment
                >= 0.50
            && round.completed_steps <= 500
        {
            "PASS"
        } else {
            "FAIL"
        }
        .to_owned(),
    };
    let binding = GeometricMixerCheckpointBinding {
        source_cid: "synthetic:no-source-opened".to_owned(),
        tokenizer_cid: "synthetic:tokenizer".to_owned(),
        base_checkpoint_identity: base.checkpoint_identity(),
        dataset_cid: synthetic_dataset_cid(&examples),
        seed,
        training_config: config,
        projection_owner: format!("uor-matmul exact GEMM@{UOR_MATMUL_REVISION}"),
    };
    let checkpoint = GeometricMixerCheckpoint::new(binding, trainer.mixer().clone())?;
    let before_bits = focused_output_bits(&trainer, &examples[0])?;
    checkpoint.save(checkpoint_path)?;
    let loaded = GeometricMixerCheckpoint::load(checkpoint_path)?;
    let loaded_trainer = MixerSpecificTrainer::new(
        loaded.mixer.clone(),
        loaded.binding.training_config.clone(),
        workers,
    )?;
    let after_bits = focused_output_bits(&loaded_trainer, &examples[0])?;
    let checkpoint_round_trip = CheckpointRoundTripResult {
        digest_before: checkpoint.content_digest.clone(),
        digest_after: loaded.content_digest.clone(),
        digest_preserved: checkpoint.content_digest == loaded.content_digest,
        focused_output_bit_identical: before_bits == after_bits,
        verdict: if checkpoint.content_digest == loaded.content_digest && before_bits == after_bits
        {
            "PASS"
        } else {
            "FAIL"
        }
        .to_owned(),
    };
    let pass = tiny_overfit.verdict == "PASS"
        && gradient_check.verdict == "PASS"
        && checkpoint_round_trip.verdict == "PASS";
    let mut report = MixerPreflightReport {
        schema: PREFLIGHT_SCHEMA.to_owned(),
        seed,
        source_trace_opened: false,
        tiny_overfit,
        gradient_check,
        checkpoint_round_trip,
        verdict: if pass { "PASS" } else { "FAIL" }.to_owned(),
        report_digest: String::new(),
    };
    report.report_digest = preflight_digest(&report)?;
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(report_path, bytes)?;
    if !pass {
        return Err(GeometricTrainingError::PreflightFailed(format!(
            "report retained at {}",
            report_path.display()
        )));
    }
    Ok(report)
}

pub fn load_passing_preflight(path: &Path) -> Result<MixerPreflightReport, GeometricTrainingError> {
    let bytes = std::fs::read(path)?;
    let report: MixerPreflightReport = serde_json::from_slice(&bytes)
        .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
    let expected = preflight_digest(&report)?;
    let recomputed_reduction = if report.tiny_overfit.initial_loss > f64::EPSILON {
        (report.tiny_overfit.initial_loss - report.tiny_overfit.final_loss)
            / report.tiny_overfit.initial_loss
    } else {
        f64::NAN
    };
    let overfit_is_bounded = report.tiny_overfit.examples > 0
        && report.tiny_overfit.examples <= report.tiny_overfit.maximum_examples
        && report.tiny_overfit.maximum_examples <= 64
        && report.tiny_overfit.requested_steps > 0
        && report.tiny_overfit.requested_steps <= report.tiny_overfit.maximum_steps
        && report.tiny_overfit.completed_steps > 0
        && report.tiny_overfit.completed_steps <= report.tiny_overfit.requested_steps
        && report.tiny_overfit.maximum_steps <= 500
        && report.tiny_overfit.initial_loss.is_finite()
        && report.tiny_overfit.final_loss.is_finite()
        && report.tiny_overfit.reduction_fraction.is_finite()
        && report.tiny_overfit.required_reduction_fraction == 0.50
        && recomputed_reduction.is_finite()
        && (report.tiny_overfit.reduction_fraction - recomputed_reduction).abs() <= f64::EPSILON
        && report.tiny_overfit.reduction_fraction
            >= report.tiny_overfit.required_reduction_fraction;
    let recomputed_allowed_error = f64::from(
        GRADIENT_CHECK_ABSOLUTE_TOLERANCE
            + GRADIENT_CHECK_RELATIVE_TOLERANCE
                * (report.gradient_check.analytical as f32)
                    .abs()
                    .max((report.gradient_check.finite_difference as f32).abs()),
    );
    let gradient_is_within_tolerance = report.gradient_check.parameter == "output_projection[0]"
        && report.gradient_check.analytical.is_finite()
        && report.gradient_check.finite_difference.is_finite()
        && report.gradient_check.absolute_error.is_finite()
        && report.gradient_check.allowed_error.is_finite()
        && report.gradient_check.allowed_error >= 0.0
        && report.gradient_check.epsilon == f64::from(GRADIENT_CHECK_EPSILON)
        && report.gradient_check.allowed_error.to_bits() == recomputed_allowed_error.to_bits()
        && report.gradient_check.absolute_error <= report.gradient_check.allowed_error;
    let checkpoint_is_identical = report.checkpoint_round_trip.digest_preserved
        && report.checkpoint_round_trip.focused_output_bit_identical
        && !report.checkpoint_round_trip.digest_before.trim().is_empty()
        && report.checkpoint_round_trip.digest_before == report.checkpoint_round_trip.digest_after;
    if report.schema != PREFLIGHT_SCHEMA
        || report.source_trace_opened
        || report.verdict != "PASS"
        || report.tiny_overfit.verdict != "PASS"
        || !overfit_is_bounded
        || report.gradient_check.verdict != "PASS"
        || !gradient_is_within_tolerance
        || report.checkpoint_round_trip.verdict != "PASS"
        || !checkpoint_is_identical
        || report.report_digest != expected
    {
        return Err(GeometricTrainingError::PreflightFailed(format!(
            "{} is not a complete passing preflight",
            path.display()
        )));
    }
    Ok(report)
}

fn preflight_digest(report: &MixerPreflightReport) -> Result<String, GeometricTrainingError> {
    let mut content = report.clone();
    content.report_digest.clear();
    let bytes = serde_json::to_vec(&content)
        .map_err(|error| GeometricTrainingError::Serialization(error.to_string()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn synthetic_examples(
    seed: u64,
    teacher: &GeometricMixer,
    config: &MixerTrainingConfig,
    workers: NonZeroUsize,
) -> Result<Vec<MixerTrainingExample>, GeometricTrainingError> {
    let teacher_trainer = MixerSpecificTrainer::new(teacher.clone(), config.clone(), workers)?;
    let mut generator = TrainingGenerator::new(seed ^ 0x9510_0001);
    let mut examples = Vec::with_capacity(8);
    for example_index in 0..8usize {
        let prefix_len = 3 + example_index % 4;
        let normalized_prefix = (0..prefix_len)
            .map(|_| {
                let mut residual = (0..teacher.source_width)
                    .map(|_| generator.next_signed())
                    .collect::<Vec<_>>();
                normalize_vector(&mut residual);
                residual
            })
            .collect::<Vec<_>>();
        let memories = if example_index.is_multiple_of(2) {
            let mut mean_embedding = (0..teacher.source_width)
                .map(|_| generator.next_signed())
                .collect::<Vec<_>>();
            normalize_vector(&mut mean_embedding);
            vec![TrainingMemoryInput {
                span_index: 0,
                mean_embedding,
                r4_coordinates: std::array::from_fn(|_| generator.next_signed()),
            }]
        } else {
            Vec::new()
        };
        let provisional_target = TrainingSupportTarget {
            source: TrainingSupportSource::Prefix,
            index: prefix_len - 1,
            probability: 1.0,
        };
        let mut example = MixerTrainingExample {
            id: format!("synthetic-{example_index:02}"),
            prefix_kind: if example_index.is_multiple_of(3) {
                TrainingPrefixKind::Student
            } else {
                TrainingPrefixKind::Teacher
            },
            normalized_prefix,
            session_route_state: std::array::from_fn(|_| 0.2 * generator.next_signed()),
            memories,
            target_attention_output: vec![0.0; teacher.source_width],
            support_target: vec![provisional_target],
            next_token_candidate_embeddings: (0..config.sampled_token_candidates)
                .map(|_| {
                    let mut embedding = (0..teacher.source_width)
                        .map(|_| generator.next_signed())
                        .collect::<Vec<_>>();
                    normalize_vector(&mut embedding);
                    embedding
                })
                .collect(),
        };
        let teacher_cache = forward_example(
            teacher,
            config,
            &example,
            GeometryIntervention::Real,
            &teacher_trainer.executor,
        )?;
        example.target_attention_output = teacher_cache.output.clone();
        let mut support_order = (0..teacher_cache.candidates.len()).collect::<Vec<_>>();
        support_order.sort_by(|&left, &right| {
            teacher_cache.candidates[right]
                .score
                .total_cmp(&teacher_cache.candidates[left].score)
        });
        support_order.truncate(teacher.support_budget.min(2));
        let target_probability = 1.0 / support_order.len() as f32;
        example.support_target = support_order
            .into_iter()
            .map(|index| TrainingSupportTarget {
                source: teacher_cache.candidates[index].source,
                index: teacher_cache.candidates[index].index,
                probability: target_probability,
            })
            .collect();
        let mut true_embedding = example.target_attention_output.clone();
        normalize_vector(&mut true_embedding);
        example.next_token_candidate_embeddings[0] = true_embedding;
        examples.push(example);
    }
    Ok(examples)
}

fn synthetic_dataset_cid(examples: &[MixerTrainingExample]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.geometric-mixer-synthetic-dataset/1");
    for example in examples {
        hasher.update(example.id.as_bytes());
        for value in &example.target_attention_output {
            hasher.update(&value.to_bits().to_le_bytes());
        }
        for target in &example.support_target {
            hasher.update(target.source.as_str().as_bytes());
            hasher.update(&(target.index as u64).to_le_bytes());
            hasher.update(&target.probability.to_bits().to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn normalize_vector(values: &mut [f32]) {
    let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > f32::MIN_POSITIVE {
        for value in values {
            *value /= norm;
        }
    }
}

struct TrainingGenerator {
    state: u64,
}

impl TrainingGenerator {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_signed(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        let unit = ((self.state >> 40) as u32) as f32 / ((1u32 << 24) - 1) as f32;
        unit * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixer_preflight_passes_without_a_source_trace() {
        let directory =
            std::env::temp_dir().join(format!("uor-r4-issue-951-preflight-{}", std::process::id()));
        let report_path = directory.join("preflight.json");
        let checkpoint_path = directory.join("checkpoint.json");
        let report = run_mixer_preflight(951_202_608_26, &report_path, &checkpoint_path)
            .expect("focused preflight");
        assert_eq!(report.verdict, "PASS");
        assert!(!report.source_trace_opened);
        assert!(report.tiny_overfit.reduction_fraction >= 0.50);
        assert_eq!(load_passing_preflight(&report_path).unwrap(), report);

        let mut tampered = report;
        tampered.tiny_overfit.reduction_fraction = 0.49;
        tampered.report_digest = preflight_digest(&tampered).unwrap();
        std::fs::write(&report_path, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        assert!(load_passing_preflight(&report_path).is_err());

        let _ = std::fs::remove_file(report_path);
        let _ = std::fs::remove_file(checkpoint_path);
        let _ = std::fs::remove_dir(directory);
    }

    #[test]
    fn training_example_rejects_non_finite_geometry() {
        let example = MixerTrainingExample {
            id: "non-finite-memory".to_owned(),
            prefix_kind: TrainingPrefixKind::Student,
            normalized_prefix: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            session_route_state: [0.0; 4],
            memories: vec![TrainingMemoryInput {
                span_index: 0,
                mean_embedding: vec![0.5, 0.5],
                r4_coordinates: [f32::NAN, 0.0, 0.0, 0.0],
            }],
            target_attention_output: vec![0.25, -0.25],
            support_target: vec![TrainingSupportTarget {
                source: TrainingSupportSource::Memory,
                index: 0,
                probability: 1.0,
            }],
            next_token_candidate_embeddings: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        };
        assert!(matches!(
            example.validate(2),
            Err(GeometricTrainingError::NonFiniteExample(id)) if id == "non-finite-memory"
        ));
    }

    #[test]
    fn student_forward_is_causal_bounded_and_memory_permutation_is_scoped() {
        let workers = NonZeroUsize::new(1).unwrap();
        let config = MixerTrainingConfig::issue_951(951);
        let mixer = GeometricMixer::deterministic(0, 8, b"issue-951-focused-controls").unwrap();
        let examples = synthetic_examples(951, &mixer, &config, workers).unwrap();
        let example = examples
            .iter()
            .find(|example| {
                example.prefix_kind == TrainingPrefixKind::Student && !example.memories.is_empty()
            })
            .unwrap();
        let trainer = MixerSpecificTrainer::new(mixer.clone(), config.clone(), workers).unwrap();
        let real = forward_example(
            &mixer,
            &config,
            example,
            GeometryIntervention::Real,
            &trainer.executor,
        )
        .unwrap();
        let permuted = forward_example(
            &mixer,
            &config,
            example,
            GeometryIntervention::PermutedMemory,
            &trainer.executor,
        )
        .unwrap();

        let real_prefix = real
            .candidates
            .iter()
            .filter(|candidate| candidate.source == TrainingSupportSource::Prefix)
            .collect::<Vec<_>>();
        let permuted_prefix = permuted
            .candidates
            .iter()
            .filter(|candidate| candidate.source == TrainingSupportSource::Prefix)
            .collect::<Vec<_>>();
        assert_eq!(real_prefix.len(), example.normalized_prefix.len());
        assert!(real_prefix
            .iter()
            .all(|candidate| candidate.index < example.normalized_prefix.len()));
        assert!(
            real.candidates
                .iter()
                .filter(|candidate| candidate.selected_weight > 0.0)
                .count()
                <= mixer.support_budget
        );
        assert!(real_prefix
            .iter()
            .zip(permuted_prefix)
            .all(|(left, right)| left.effective_key == right.effective_key));

        let real_memory = real
            .candidates
            .iter()
            .find(|candidate| candidate.source == TrainingSupportSource::Memory)
            .unwrap();
        let permuted_memory = permuted
            .candidates
            .iter()
            .find(|candidate| candidate.source == TrainingSupportSource::Memory)
            .unwrap();
        assert_ne!(real_memory.effective_key, permuted_memory.effective_key);
        assert_ne!(
            real_memory.support_probability.to_bits(),
            permuted_memory.support_probability.to_bits()
        );

        let student_examples = examples
            .iter()
            .filter(|example| example.prefix_kind == TrainingPrefixKind::Student)
            .cloned()
            .collect::<Vec<_>>();
        let summary = trainer
            .evaluate(&student_examples, GeometryIntervention::Real)
            .unwrap();
        assert_eq!(summary.examples, student_examples.len());
        assert!(summary.total.is_finite());
    }
}
