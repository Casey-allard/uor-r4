//! Experimental one-layer geometric decoder seam (issue #950).
//!
//! This module deliberately owns only the smallest trainable operator needed
//! for the G0 reachability spike.  It is a host-side floating-point path: the
//! frozen R4G1 P-4, `no_std`, and allocation-free contracts do not apply.
//! Dense query/key/value/output projections are executed by the same pinned
//! `uor-matmul` exact owner as the Llama source runtime.  The mixer selects a
//! bounded causal support set before aggregating values and never constructs a
//! dense full-prefix Q·K matrix or calls the source-attention operator.

use serde::{Deserialize, Serialize};

use crate::{expf, matmul, ExactExecutor, State, UOR_MATMUL_REVISION};

/// Width of the learned R4 coordinate used for compatibility.
pub const R4_COORDINATE_WIDTH: usize = 4;
/// Deliberately small value bottleneck for the G0 spike.
pub const DEFAULT_VALUE_WIDTH: usize = 16;
/// Maximum selected prefix/memory entries per token.
pub const DEFAULT_SUPPORT_BUDGET: usize = 4;
/// Hard bound on memory spans admitted to one decoder context.
pub const MAX_MEMORY_SPANS: usize = 16;
/// Hard bound on source-tokenizer ids admitted across one context.
pub const MAX_MEMORY_TOKENS: usize = 256;

/// Controlled intervention at the one-layer seam.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryIntervention {
    /// Execute the learned R4 mixer with the recorded coordinates.
    Real,
    /// Permute candidate coordinates while leaving the query unchanged.
    PermutedCoordinates,
    /// Permute only persistent-memory coordinates. Prefix geometry and every
    /// trainable value remain unchanged, making this the matched G1 memory
    /// ablation rather than a second model.
    PermutedMemory,
    /// Bypass the mixer and execute ordinary source attention exactly.
    Disabled,
}

impl GeometryIntervention {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::PermutedCoordinates => "permuted_coordinates",
            Self::PermutedMemory => "permuted_memory",
            Self::Disabled => "disabled",
        }
    }
}

/// Content/provenance binding carried by every [`GeometryContext`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeometryProvenance {
    /// Content identity of the exact source weights.
    pub source_cid: String,
    /// Content identity of the router state used to construct the context.
    pub router_state_cid: String,
    /// Human-readable source for the retained memory spans.
    pub memory_source: String,
}

/// One ordered persistent memory span encoded by the real source tokenizer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryMemorySpan {
    pub sequence: u64,
    pub role: String,
    pub text: String,
    pub token_ids: Vec<u32>,
    pub tokenizer_cid: String,
    pub adapter_identity: String,
    pub r4_coordinates: [f32; R4_COORDINATE_WIDTH],
    pub provenance: String,
}

/// Runtime R4 state retained for one causal token position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricPositionState {
    pub position: usize,
    pub query_coordinates: [f32; R4_COORDINATE_WIDTH],
    pub key_coordinates: [f32; R4_COORDINATE_WIDTH],
    pub selected_support_cid: String,
}

/// Minimal tokenizer-bound input to the experimental mixer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryContext {
    pub identity: String,
    pub tokenizer_cid: String,
    pub adapter_identity: String,
    pub session_route_state: [f32; R4_COORDINATE_WIDTH],
    pub memory_spans: Vec<GeometryMemorySpan>,
    pub provenance: GeometryProvenance,
    /// Filled deterministically as the causal session advances.
    pub position_states: Vec<GeometricPositionState>,
}

impl GeometryContext {
    /// Construct and validate one bounded context.  A binding mismatch fails
    /// closed instead of silently re-tokenizing or re-projecting a memory.
    pub fn new(
        identity: impl Into<String>,
        tokenizer_cid: impl Into<String>,
        adapter_identity: impl Into<String>,
        session_route_state: [f32; R4_COORDINATE_WIDTH],
        memory_spans: Vec<GeometryMemorySpan>,
        provenance: GeometryProvenance,
    ) -> Result<Self, GeometricDecoderError> {
        let context = Self {
            identity: identity.into(),
            tokenizer_cid: tokenizer_cid.into(),
            adapter_identity: adapter_identity.into(),
            session_route_state,
            memory_spans,
            provenance,
            position_states: Vec::new(),
        };
        context.validate()?;
        Ok(context)
    }

    fn validate(&self) -> Result<(), GeometricDecoderError> {
        if self.identity.trim().is_empty() {
            return Err(GeometricDecoderError::EmptyIdentity);
        }
        if self.tokenizer_cid.trim().is_empty() || self.adapter_identity.trim().is_empty() {
            return Err(GeometricDecoderError::EmptyBinding);
        }
        if self.memory_spans.len() > MAX_MEMORY_SPANS {
            return Err(GeometricDecoderError::MemorySpanLimit {
                requested: self.memory_spans.len(),
                maximum: MAX_MEMORY_SPANS,
            });
        }
        let mut memory_tokens = 0usize;
        let mut previous_sequence = None;
        for span in &self.memory_spans {
            if previous_sequence.is_some_and(|previous| span.sequence <= previous) {
                return Err(GeometricDecoderError::MemoryOrder);
            }
            previous_sequence = Some(span.sequence);
            if span.tokenizer_cid != self.tokenizer_cid
                || span.adapter_identity != self.adapter_identity
            {
                return Err(GeometricDecoderError::MemoryBindingMismatch {
                    sequence: span.sequence,
                });
            }
            if span.token_ids.is_empty() {
                return Err(GeometricDecoderError::EmptyMemorySpan {
                    sequence: span.sequence,
                });
            }
            memory_tokens = memory_tokens
                .checked_add(span.token_ids.len())
                .ok_or(GeometricDecoderError::ArithmeticOverflow)?;
        }
        if memory_tokens > MAX_MEMORY_TOKENS {
            return Err(GeometricDecoderError::MemoryTokenLimit {
                requested: memory_tokens,
                maximum: MAX_MEMORY_TOKENS,
            });
        }
        if !self
            .session_route_state
            .iter()
            .chain(
                self.memory_spans
                    .iter()
                    .flat_map(|span| span.r4_coordinates.iter()),
            )
            .all(|value| value.is_finite())
        {
            return Err(GeometricDecoderError::NonFiniteCoordinates);
        }
        Ok(())
    }
}

/// All mutable trainable values of the G0 mixer.  #951 may update this exact
/// bounded parameter set; no generic autograd/optimizer substrate is implied.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricMixerParameters {
    /// Row-major `[4, source_width]`.
    pub query_projection: Vec<f32>,
    /// Row-major `[4, source_width]`.
    pub key_projection: Vec<f32>,
    /// Row-major `[value_width, source_width]`.
    pub value_projection: Vec<f32>,
    /// Row-major `[source_width, value_width]`.
    pub output_projection: Vec<f32>,
    pub query_bias: [f32; R4_COORDINATE_WIDTH],
    pub key_bias: [f32; R4_COORDINATE_WIDTH],
    /// Small residual scale keeps the untrained G0 reachability treatment
    /// decodable; quality fitting is explicitly owned by #951.
    pub output_gain: f32,
}

/// One deterministic, trainable mixer checkpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricMixer {
    pub layer: usize,
    pub source_width: usize,
    pub value_width: usize,
    pub support_budget: usize,
    pub parameters: GeometricMixerParameters,
}

impl GeometricMixer {
    /// Deterministically initialize the small G0 checkpoint from a
    /// content-bound seed.  The returned vectors are ordinary trainable f32
    /// parameters; deterministic initialization is an identity property, not
    /// a claim that the untrained operator has a quality advantage.
    pub fn deterministic(
        layer: usize,
        source_width: usize,
        seed: &[u8],
    ) -> Result<Self, GeometricDecoderError> {
        if source_width == 0 {
            return Err(GeometricDecoderError::InvalidSourceWidth(0));
        }
        let mut generator = DeterministicGenerator::new(seed);
        let coordinate_scale = 0.12 / (source_width as f32).sqrt();
        let value_scale = 0.10 / (source_width as f32).sqrt();
        let output_scale = 0.10 / (DEFAULT_VALUE_WIDTH as f32).sqrt();
        let mut values = |length: usize, scale: f32| {
            (0..length)
                .map(|_| generator.next_signed() * scale)
                .collect::<Vec<_>>()
        };
        let mixer = Self {
            layer,
            source_width,
            value_width: DEFAULT_VALUE_WIDTH,
            support_budget: DEFAULT_SUPPORT_BUDGET,
            parameters: GeometricMixerParameters {
                query_projection: values(R4_COORDINATE_WIDTH * source_width, coordinate_scale),
                key_projection: values(R4_COORDINATE_WIDTH * source_width, coordinate_scale),
                value_projection: values(DEFAULT_VALUE_WIDTH * source_width, value_scale),
                output_projection: values(source_width * DEFAULT_VALUE_WIDTH, output_scale),
                query_bias: [0.011, -0.017, 0.023, -0.029],
                key_bias: [-0.019, 0.013, -0.031, 0.007],
                output_gain: 0.05,
            },
        };
        mixer.validate()?;
        Ok(mixer)
    }

    pub fn validate(&self) -> Result<(), GeometricDecoderError> {
        if self.source_width == 0 {
            return Err(GeometricDecoderError::InvalidSourceWidth(0));
        }
        if self.value_width == 0 || self.support_budget == 0 {
            return Err(GeometricDecoderError::InvalidMixerShape);
        }
        let expected = [
            (
                self.parameters.query_projection.len(),
                R4_COORDINATE_WIDTH * self.source_width,
            ),
            (
                self.parameters.key_projection.len(),
                R4_COORDINATE_WIDTH * self.source_width,
            ),
            (
                self.parameters.value_projection.len(),
                self.value_width * self.source_width,
            ),
            (
                self.parameters.output_projection.len(),
                self.source_width * self.value_width,
            ),
        ];
        if expected.iter().any(|(actual, expected)| actual != expected)
            || !self
                .parameters
                .query_projection
                .iter()
                .chain(self.parameters.key_projection.iter())
                .chain(self.parameters.value_projection.iter())
                .chain(self.parameters.output_projection.iter())
                .chain(self.parameters.query_bias.iter())
                .chain(self.parameters.key_bias.iter())
                .chain(std::iter::once(&self.parameters.output_gain))
                .all(|value| value.is_finite())
        {
            return Err(GeometricDecoderError::InvalidMixerShape);
        }
        Ok(())
    }

    /// Content identity of every trainable value and declared bound.
    pub fn checkpoint_identity(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4.geometric-mixer-checkpoint/1");
        hash_usize(&mut hasher, self.layer);
        hash_usize(&mut hasher, self.source_width);
        hash_usize(&mut hasher, self.value_width);
        hash_usize(&mut hasher, self.support_budget);
        for values in [
            self.parameters.query_projection.as_slice(),
            self.parameters.key_projection.as_slice(),
            self.parameters.value_projection.as_slice(),
            self.parameters.output_projection.as_slice(),
            self.parameters.query_bias.as_slice(),
            self.parameters.key_bias.as_slice(),
            std::slice::from_ref(&self.parameters.output_gain),
        ] {
            hash_f32s(&mut hasher, values);
        }
        format!("blake3:{}", hasher.finalize().to_hex())
    }

    /// Deterministic memory-to-layer adapter identity.  It binds the exact
    /// tokenizer/source, target layer, projection owner, and the key/value
    /// parameters that turn source-token ids into mixer support entries.
    pub fn memory_adapter_identity(&self, source_cid: &str, tokenizer_cid: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4.memory-to-layer-adapter/1");
        hash_bytes(&mut hasher, source_cid.as_bytes());
        hash_bytes(&mut hasher, tokenizer_cid.as_bytes());
        hash_bytes(&mut hasher, UOR_MATMUL_REVISION.as_bytes());
        hash_usize(&mut hasher, self.layer);
        hash_usize(&mut hasher, self.source_width);
        hash_usize(&mut hasher, self.value_width);
        hash_f32s(&mut hasher, &self.parameters.key_projection);
        hash_f32s(&mut hasher, &self.parameters.value_projection);
        hash_f32s(&mut hasher, &self.parameters.key_bias);
        format!("blake3:{}", hasher.finalize().to_hex())
    }
}

/// One selected support entry in the compact operator trace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricSupportTrace {
    pub source: String,
    pub index: usize,
    pub score: f32,
    pub weight: f32,
}

/// Compact per-token evidence from the target layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometricOperatorTrace {
    pub layer: usize,
    pub position: usize,
    pub intervention: GeometryIntervention,
    pub projection_owner: String,
    pub source_attention_calls: u8,
    pub dense_full_prefix_qk: bool,
    pub prefix_candidates: usize,
    pub memory_candidates: usize,
    pub support_budget: usize,
    pub selected_support: Vec<GeometricSupportTrace>,
    pub support_cid: String,
    pub output_l2: f32,
}

#[derive(Clone)]
struct PreparedMemory {
    span_index: usize,
    key: [f32; R4_COORDINATE_WIDTH],
    value: Vec<f32>,
}

#[derive(Clone)]
struct PrefixEntry {
    key: [f32; R4_COORDINATE_WIDTH],
    value: Vec<f32>,
}

#[derive(Clone)]
struct Candidate {
    source: &'static str,
    index: usize,
    score: f32,
    value: Vec<f32>,
}

/// Mutable state of one geometric sequence.  It is cloneable so a controlled
/// intervention can start from exactly the same causal prefix bits.
#[derive(Clone)]
pub(crate) struct GeometricRuntime {
    mixer: GeometricMixer,
    context: GeometryContext,
    intervention: GeometryIntervention,
    prepared_memory: Vec<PreparedMemory>,
    prefix: Vec<Option<PrefixEntry>>,
    traces: Vec<GeometricOperatorTrace>,
}

impl GeometricRuntime {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        mixer: GeometricMixer,
        context: GeometryContext,
        intervention: GeometryIntervention,
        sequence_capacity: usize,
        executor: &ExactExecutor,
        embedding_rows: &[f32],
        vocabulary: usize,
    ) -> Result<Self, GeometricDecoderError> {
        mixer.validate()?;
        context.validate()?;
        if embedding_rows.len() < vocabulary.saturating_mul(mixer.source_width) {
            return Err(GeometricDecoderError::InvalidEmbeddingShape);
        }
        let mut prepared_memory = Vec::with_capacity(context.memory_spans.len());
        for (span_index, span) in context.memory_spans.iter().enumerate() {
            let mut mean = vec![0.0f32; mixer.source_width];
            for &token in &span.token_ids {
                let token = usize::try_from(token)
                    .map_err(|_| GeometricDecoderError::TokenOutOfRange(u32::MAX as usize))?;
                if token >= vocabulary {
                    return Err(GeometricDecoderError::TokenOutOfRange(token));
                }
                let row =
                    &embedding_rows[token * mixer.source_width..(token + 1) * mixer.source_width];
                for (sum, value) in mean.iter_mut().zip(row) {
                    *sum += *value;
                }
            }
            let reciprocal = 1.0 / span.token_ids.len() as f32;
            for value in &mut mean {
                *value *= reciprocal;
            }
            let mut key = [0.0; R4_COORDINATE_WIDTH];
            matmul(
                executor,
                &mut key,
                &mean,
                &mixer.parameters.key_projection,
                mixer.source_width,
                true,
            );
            for lane in 0..R4_COORDINATE_WIDTH {
                key[lane] += mixer.parameters.key_bias[lane]
                    + 0.25 * span.r4_coordinates[lane]
                    + 0.01 * ((span_index + lane) as f32 + 1.0).sin();
            }
            normalize4(&mut key);
            let mut value = vec![0.0; mixer.value_width];
            matmul(
                executor,
                &mut value,
                &mean,
                &mixer.parameters.value_projection,
                mixer.source_width,
                true,
            );
            prepared_memory.push(PreparedMemory {
                span_index,
                key,
                value,
            });
        }
        Ok(Self {
            mixer,
            context,
            intervention,
            prepared_memory,
            prefix: vec![None; sequence_capacity],
            traces: Vec::new(),
        })
    }

    pub(crate) fn target_layer(&self) -> usize {
        self.mixer.layer
    }

    pub(crate) fn intervention(&self) -> GeometryIntervention {
        self.intervention
    }

    pub(crate) fn set_intervention(&mut self, intervention: GeometryIntervention) {
        self.intervention = intervention;
    }

    pub(crate) fn context(&self) -> &GeometryContext {
        &self.context
    }

    pub(crate) fn traces(&self) -> &[GeometricOperatorTrace] {
        &self.traces
    }

    pub(crate) fn clear_traces(&mut self) {
        self.traces.clear();
    }

    pub(crate) fn checkpoint_identity(&self) -> String {
        self.mixer.checkpoint_identity()
    }

    pub(crate) fn mix(
        &mut self,
        executor: &ExactExecutor,
        normalized_residual: &[f32],
        position: usize,
        output: &mut [f32],
        canonical_math: bool,
    ) {
        debug_assert_eq!(normalized_residual.len(), self.mixer.source_width);
        debug_assert_eq!(output.len(), self.mixer.source_width);
        debug_assert!(position < self.prefix.len());

        let mut query = [0.0; R4_COORDINATE_WIDTH];
        let mut key = [0.0; R4_COORDINATE_WIDTH];
        matmul(
            executor,
            &mut query,
            normalized_residual,
            &self.mixer.parameters.query_projection,
            self.mixer.source_width,
            true,
        );
        matmul(
            executor,
            &mut key,
            normalized_residual,
            &self.mixer.parameters.key_projection,
            self.mixer.source_width,
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
            query[lane] += self.mixer.parameters.query_bias[lane]
                + 0.10 * self.context.session_route_state[lane];
            key[lane] += self.mixer.parameters.key_bias[lane]
                + 0.10 * self.context.session_route_state[lane]
                + 0.05 * positional[lane];
        }
        normalize4(&mut query);
        normalize4(&mut key);
        let mut value = vec![0.0; self.mixer.value_width];
        matmul(
            executor,
            &mut value,
            normalized_residual,
            &self.mixer.parameters.value_projection,
            self.mixer.source_width,
            true,
        );
        self.prefix[position] = Some(PrefixEntry {
            key,
            value: value.clone(),
        });

        let mut candidates = Vec::with_capacity(position + 1 + self.prepared_memory.len());
        for prefix_position in 0..=position {
            let Some(entry) = &self.prefix[prefix_position] else {
                continue;
            };
            let candidate_key = intervene_key(entry.key, self.intervention, false);
            candidates.push(Candidate {
                source: "prefix",
                index: prefix_position,
                score: compatibility(query, candidate_key),
                value: entry.value.clone(),
            });
        }
        for memory in &self.prepared_memory {
            let candidate_key = intervene_key(memory.key, self.intervention, true);
            candidates.push(Candidate {
                source: "memory",
                index: memory.span_index,
                score: compatibility(query, candidate_key),
                value: memory.value.clone(),
            });
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| source_order(left.source).cmp(&source_order(right.source)))
                .then_with(|| left.index.cmp(&right.index))
        });
        candidates.truncate(self.mixer.support_budget.min(candidates.len()));

        let max_score = candidates
            .iter()
            .map(|candidate| candidate.score)
            .max_by(f32::total_cmp)
            .unwrap_or(0.0);
        let mut weights = candidates
            .iter()
            .map(|candidate| expf(candidate.score - max_score, canonical_math))
            .collect::<Vec<_>>();
        let weight_sum = weights.iter().sum::<f32>().max(f32::MIN_POSITIVE);
        for weight in &mut weights {
            *weight /= weight_sum;
        }
        let mut aggregate = vec![0.0f32; self.mixer.value_width];
        for (candidate, &weight) in candidates.iter().zip(&weights) {
            for (sum, value) in aggregate.iter_mut().zip(&candidate.value) {
                *sum += weight * *value;
            }
        }
        matmul(
            executor,
            output,
            &aggregate,
            &self.mixer.parameters.output_projection,
            self.mixer.value_width,
            true,
        );
        for value in output.iter_mut() {
            *value *= self.mixer.parameters.output_gain;
        }

        let selected_support = candidates
            .iter()
            .zip(&weights)
            .map(|(candidate, &weight)| GeometricSupportTrace {
                source: candidate.source.to_owned(),
                index: candidate.index,
                score: candidate.score,
                weight,
            })
            .collect::<Vec<_>>();
        let support_cid = support_cid(&selected_support);
        let output_l2 = output.iter().map(|value| value * value).sum::<f32>().sqrt();
        self.context.position_states.push(GeometricPositionState {
            position,
            query_coordinates: query,
            key_coordinates: key,
            selected_support_cid: support_cid.clone(),
        });
        self.traces.push(GeometricOperatorTrace {
            layer: self.mixer.layer,
            position,
            intervention: self.intervention,
            projection_owner: format!("uor-matmul exact GEMM@{UOR_MATMUL_REVISION}"),
            source_attention_calls: 0,
            dense_full_prefix_qk: false,
            prefix_candidates: position + 1,
            memory_candidates: self.prepared_memory.len(),
            support_budget: self.mixer.support_budget,
            selected_support,
            support_cid,
            output_l2,
        });
    }
}

/// Cloneable source state plus geometric runtime for one independent arm.
#[derive(Clone)]
pub struct GeometricDecoderSession {
    pub(crate) state: State,
    pub(crate) runtime: GeometricRuntime,
}

impl GeometricDecoderSession {
    pub fn intervention(&self) -> GeometryIntervention {
        self.runtime.intervention()
    }

    pub fn set_intervention(&mut self, intervention: GeometryIntervention) {
        self.runtime.set_intervention(intervention);
    }

    pub fn context(&self) -> &GeometryContext {
        self.runtime.context()
    }

    pub fn traces(&self) -> &[GeometricOperatorTrace] {
        self.runtime.traces()
    }

    pub fn clear_traces(&mut self) {
        self.runtime.clear_traces();
    }

    pub fn checkpoint_identity(&self) -> String {
        self.runtime.checkpoint_identity()
    }

    pub fn persistent_state_cid(&self) -> String {
        self.state.persistent_state_cid()
    }

    pub fn sequence_capacity(&self) -> usize {
        self.state.sequence_capacity()
    }
}

/// Focused typed failures at the experimental library boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeometricDecoderError {
    EmptyIdentity,
    EmptyBinding,
    MemorySpanLimit { requested: usize, maximum: usize },
    MemoryTokenLimit { requested: usize, maximum: usize },
    EmptyMemorySpan { sequence: u64 },
    MemoryBindingMismatch { sequence: u64 },
    MemoryOrder,
    NonFiniteCoordinates,
    InvalidSourceWidth(usize),
    InvalidMixerShape,
    InvalidEmbeddingShape,
    SourceBindingMismatch,
    AdapterBindingMismatch,
    TargetLayerOutOfRange { requested: usize, layers: usize },
    SequenceCapacity(String),
    PositionOutOfRange { position: usize, capacity: usize },
    TokenOutOfRange(usize),
    LogitShape { requested: usize, expected: usize },
    ArithmeticOverflow,
}

impl std::fmt::Display for GeometricDecoderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "geometric decoder unavailable: {self:?}")
    }
}

impl std::error::Error for GeometricDecoderError {}

fn compatibility(query: [f32; 4], key: [f32; 4]) -> f32 {
    let angular = query
        .iter()
        .zip(key.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>()
        .clamp(-1.0, 1.0);
    let geodesic = angular.acos() / std::f32::consts::PI;
    angular - 0.25 * geodesic
}

fn intervene_key(key: [f32; 4], intervention: GeometryIntervention, memory: bool) -> [f32; 4] {
    match intervention {
        GeometryIntervention::Real | GeometryIntervention::Disabled => key,
        GeometryIntervention::PermutedCoordinates => [key[1], -key[3], key[0], -key[2]],
        GeometryIntervention::PermutedMemory if memory => [key[1], -key[3], key[0], -key[2]],
        GeometryIntervention::PermutedMemory => key,
    }
}

fn normalize4(values: &mut [f32; 4]) {
    let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > f32::MIN_POSITIVE {
        for value in values {
            *value /= norm;
        }
    } else {
        *values = [1.0, 0.0, 0.0, 0.0];
    }
}

fn source_order(source: &str) -> u8 {
    if source == "prefix" {
        0
    } else {
        1
    }
}

fn support_cid(entries: &[GeometricSupportTrace]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.geometric-support/1");
    for entry in entries {
        hash_bytes(&mut hasher, entry.source.as_bytes());
        hash_usize(&mut hasher, entry.index);
        hasher.update(&entry.score.to_bits().to_le_bytes());
        hasher.update(&entry.weight.to_bits().to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn hash_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(bytes);
}

fn hash_usize(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&u64::try_from(value).unwrap_or(u64::MAX).to_le_bytes());
}

fn hash_f32s(hasher: &mut blake3::Hasher, values: &[f32]) {
    hash_usize(hasher, values.len());
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
}

struct DeterministicGenerator {
    state: u64,
}

impl DeterministicGenerator {
    fn new(seed: &[u8]) -> Self {
        let digest = blake3::hash(seed);
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest.as_bytes()[..8]);
        let state = u64::from_le_bytes(bytes).max(1);
        Self { state }
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

    fn memory(tokenizer: &str, adapter: &str) -> GeometryMemorySpan {
        GeometryMemorySpan {
            sequence: 0,
            role: "user".to_owned(),
            text: "remember blue".to_owned(),
            token_ids: vec![1, 2],
            tokenizer_cid: tokenizer.to_owned(),
            adapter_identity: adapter.to_owned(),
            r4_coordinates: [0.1, 0.2, 0.3, 0.4],
            provenance: "blake3:memory".to_owned(),
        }
    }

    #[test]
    fn context_rejects_cross_tokenizer_memory() {
        let error = GeometryContext::new(
            "alice",
            "blake3:tokenizer-a",
            "blake3:adapter",
            [0.0; 4],
            vec![memory("blake3:tokenizer-b", "blake3:adapter")],
            GeometryProvenance {
                source_cid: "blake3:source".to_owned(),
                router_state_cid: "blake3:router".to_owned(),
                memory_source: "test".to_owned(),
            },
        )
        .expect_err("mismatched memory must fail closed");
        assert_eq!(
            error,
            GeometricDecoderError::MemoryBindingMismatch { sequence: 0 }
        );
    }

    #[test]
    fn deterministic_checkpoint_and_adapter_identities_are_content_bound() {
        let first = GeometricMixer::deterministic(0, 8, b"seed").expect("mixer");
        let second = GeometricMixer::deterministic(0, 8, b"seed").expect("mixer");
        assert_eq!(first, second);
        assert_eq!(first.checkpoint_identity(), second.checkpoint_identity());
        assert_ne!(
            first.memory_adapter_identity("blake3:source-a", "blake3:tokenizer"),
            first.memory_adapter_identity("blake3:source-b", "blake3:tokenizer")
        );
    }

    #[test]
    fn context_rejects_out_of_order_memory_spans() {
        let mut later = memory("blake3:tokenizer", "blake3:adapter");
        later.sequence = 2;
        let mut earlier = later.clone();
        earlier.sequence = 1;
        let error = GeometryContext::new(
            "alice",
            "blake3:tokenizer",
            "blake3:adapter",
            [0.0; 4],
            vec![later, earlier],
            GeometryProvenance {
                source_cid: "blake3:source".to_owned(),
                router_state_cid: "blake3:router".to_owned(),
                memory_source: "test".to_owned(),
            },
        )
        .expect_err("out-of-order memory must fail closed");
        assert_eq!(error, GeometricDecoderError::MemoryOrder);
    }
}
