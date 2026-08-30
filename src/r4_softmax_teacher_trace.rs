//! Bounded construction-side traces from the established R4/Spin softmax seam.
//!
//! [`TracingR4SpinTransport`] is a transparent decorator around
//! [`R4SpinCausalAttentionTransport`]. It delegates every attention hook to the
//! established transport and records only the values already presented at
//! that seam. The caller retains an [`R4SoftmaxTeacherTraceHandle`] after the
//! transport is type-erased into the source decoder session; after each
//! successful decoder step, the caller seals the position with the resulting
//! logits and its observed construction target.
//!
//! This module is compiler-side instrumentation. It does not define a
//! source-free runtime, replace softmax, or establish a geometric advantage.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use uor_r4_core::helm_d_r4_attention::R4SpinCausalAttentionTransport;
use uor_r4_model_source::attention::{
    CausalAttentionHeadContext, CausalAttentionProjectionContext, CausalAttentionSourceContext,
    CausalAttentionTransport,
};

pub const R4_SOFTMAX_TEACHER_TRACE_SCHEMA: &str = "R4SoftmaxTeacherTraceV1";
pub const R4_SOFTMAX_TEACHER_TRACE_MAGIC: [u8; 8] = *b"R4STTR01";
pub const R4_SOFTMAX_TEACHER_TRACE_VERSION: u32 = 1;
pub const R4_SOFTMAX_TRACE_SUPPORT_CAP: usize = 8;
pub const R4_SOFTMAX_TRACE_LOGIT_CAP: usize = 32;

/// Immutable construction/source identity bound into the trace bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxTeacherTraceIdentity {
    pub source_cid: String,
    pub tokenizer_cid: String,
    pub attention_policy_cid: String,
    pub corpus_cid: String,
    pub construction_partition_id: String,
    pub document_id: String,
    pub document_text_cid: String,
}

/// Fixed shape and allocation ceiling of one traced causal session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct R4SoftmaxTeacherTraceBounds {
    pub maximum_positions: usize,
    pub layers: usize,
    pub query_heads: usize,
    pub key_value_heads: usize,
    pub head_size: usize,
    pub vocabulary: usize,
}

impl R4SoftmaxTeacherTraceBounds {
    fn validate(self) -> Result<(), R4SoftmaxTeacherTraceError> {
        if self.maximum_positions == 0
            || self.layers == 0
            || self.query_heads == 0
            || self.key_value_heads == 0
            || self.head_size == 0
            || self.vocabulary == 0
        {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "trace bounds must all be nonzero".to_owned(),
            ));
        }
        if !self.query_heads.is_multiple_of(self.key_value_heads) {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "query heads {} are not divisible by key/value heads {}",
                self.query_heads, self.key_value_heads
            )));
        }
        if !self.head_size.is_multiple_of(4) {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "head size {} is not a complete sequence of R4 blocks",
                self.head_size
            )));
        }
        for (label, value) in [
            ("maximum positions", self.maximum_positions),
            ("layers", self.layers),
            ("query heads", self.query_heads),
            ("key/value heads", self.key_value_heads),
            ("head size", self.head_size),
            ("vocabulary", self.vocabulary),
        ] {
            u32::try_from(value).map_err(|_| {
                R4SoftmaxTeacherTraceError::Invalid(format!(
                    "{label} exceeds the canonical u32 trace domain"
                ))
            })?;
        }
        Ok(())
    }

    fn query_width(self) -> Result<usize, R4SoftmaxTeacherTraceError> {
        self.query_heads
            .checked_mul(self.head_size)
            .ok_or_else(|| R4SoftmaxTeacherTraceError::Invalid("query width overflowed".to_owned()))
    }

    fn key_value_width(self) -> Result<usize, R4SoftmaxTeacherTraceError> {
        self.key_value_heads
            .checked_mul(self.head_size)
            .ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid("key/value width overflowed".to_owned())
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedQkvTrace {
    /// Post-inner, pre-RoPE checkpoint-projected query lanes as f32 bits.
    pub query_bits: Vec<u32>,
    /// Post-inner, pre-RoPE checkpoint-projected current key lanes as f32 bits.
    pub key_bits: Vec<u32>,
    /// Post-inner, pre-RoPE checkpoint-projected current value lanes as f32 bits.
    pub value_bits: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxTraceSupport {
    pub source_position: u32,
    pub weight_bits: u32,
    pub transported_key_bits: Vec<u32>,
    pub transported_value_bits: Vec<u32>,
}

impl R4SoftmaxTraceSupport {
    pub fn weight(&self) -> f32 {
        f32::from_bits(self.weight_bits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxHeadTrace {
    pub head: u32,
    pub query_gauge_bits: Vec<u32>,
    pub current_key_query_gauge_bits: Vec<u32>,
    pub current_value_query_gauge_bits: Vec<u32>,
    pub top_support: Vec<R4SoftmaxTraceSupport>,
    pub weighted_value_aggregate_query_gauge_bits: Vec<u32>,
    pub decoded_output_model_frame_bits: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxLayerTrace {
    pub layer: u32,
    pub projected_qkv: ProjectedQkvTrace,
    pub heads: Vec<R4SoftmaxHeadTrace>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct R4SoftmaxRankedLogit {
    pub token: u32,
    pub logit_bits: u32,
}

impl R4SoftmaxRankedLogit {
    pub fn logit(self) -> f32 {
        f32::from_bits(self.logit_bits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxLogitTrace {
    pub target_token: u32,
    pub target_logit_bits: u32,
    pub maximum_logit_bits: u32,
    pub logsumexp_bits: u64,
    pub target_nll_bits: u64,
    pub top_logits: Vec<R4SoftmaxRankedLogit>,
}

impl R4SoftmaxLogitTrace {
    pub fn target_logit(&self) -> f32 {
        f32::from_bits(self.target_logit_bits)
    }

    pub fn maximum_logit(&self) -> f32 {
        f32::from_bits(self.maximum_logit_bits)
    }

    pub fn logsumexp(&self) -> f64 {
        f64::from_bits(self.logsumexp_bits)
    }

    pub fn target_nll(&self) -> f64 {
        f64::from_bits(self.target_nll_bits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxPositionTrace {
    pub position: u32,
    pub input_token: u32,
    pub frame_table_offset: u16,
    pub layers: Vec<R4SoftmaxLayerTrace>,
    pub logits: R4SoftmaxLogitTrace,
}

/// Complete immutable trace for one construction document/session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R4SoftmaxTeacherTrace {
    pub identity: R4SoftmaxTeacherTraceIdentity,
    pub bounds: R4SoftmaxTeacherTraceBounds,
    pub positions: Vec<R4SoftmaxPositionTrace>,
}

impl R4SoftmaxTeacherTrace {
    pub fn validate(&self) -> Result<(), R4SoftmaxTeacherTraceError> {
        validate_identity(&self.identity)?;
        self.bounds.validate()?;
        if self.positions.is_empty() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "a complete trace must contain at least one position".to_owned(),
            ));
        }
        if self.positions.len() > self.bounds.maximum_positions {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "trace contains {} positions but its bound is {}",
                self.positions.len(),
                self.bounds.maximum_positions
            )));
        }
        for (expected_position, position) in self.positions.iter().enumerate() {
            validate_position(position, expected_position, self.bounds)?;
        }
        Ok(())
    }

    /// Canonical little-endian bytes. Local paths, timing, and allocation
    /// details are deliberately absent.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, R4SoftmaxTeacherTraceError> {
        self.validate()?;
        let mut output = Vec::new();
        output.extend_from_slice(&R4_SOFTMAX_TEACHER_TRACE_MAGIC);
        push_u32(&mut output, R4_SOFTMAX_TEACHER_TRACE_VERSION);
        push_string(&mut output, R4_SOFTMAX_TEACHER_TRACE_SCHEMA)?;
        for value in [
            self.identity.source_cid.as_str(),
            self.identity.tokenizer_cid.as_str(),
            self.identity.attention_policy_cid.as_str(),
            self.identity.corpus_cid.as_str(),
            self.identity.construction_partition_id.as_str(),
            self.identity.document_id.as_str(),
            self.identity.document_text_cid.as_str(),
        ] {
            push_string(&mut output, value)?;
        }
        for value in [
            self.bounds.maximum_positions,
            self.bounds.layers,
            self.bounds.query_heads,
            self.bounds.key_value_heads,
            self.bounds.head_size,
            self.bounds.vocabulary,
            R4_SOFTMAX_TRACE_SUPPORT_CAP,
            R4_SOFTMAX_TRACE_LOGIT_CAP,
        ] {
            push_usize(&mut output, value)?;
        }
        push_usize(&mut output, self.positions.len())?;
        for position in &self.positions {
            push_u32(&mut output, position.position);
            push_u32(&mut output, position.input_token);
            output.extend_from_slice(&position.frame_table_offset.to_le_bytes());
            push_usize(&mut output, position.layers.len())?;
            for layer in &position.layers {
                push_u32(&mut output, layer.layer);
                push_bits(&mut output, &layer.projected_qkv.query_bits)?;
                push_bits(&mut output, &layer.projected_qkv.key_bits)?;
                push_bits(&mut output, &layer.projected_qkv.value_bits)?;
                push_usize(&mut output, layer.heads.len())?;
                for head in &layer.heads {
                    push_u32(&mut output, head.head);
                    push_bits(&mut output, &head.query_gauge_bits)?;
                    push_bits(&mut output, &head.current_key_query_gauge_bits)?;
                    push_bits(&mut output, &head.current_value_query_gauge_bits)?;
                    push_usize(&mut output, head.top_support.len())?;
                    for support in &head.top_support {
                        push_u32(&mut output, support.source_position);
                        push_u32(&mut output, support.weight_bits);
                        push_bits(&mut output, &support.transported_key_bits)?;
                        push_bits(&mut output, &support.transported_value_bits)?;
                    }
                    push_bits(&mut output, &head.weighted_value_aggregate_query_gauge_bits)?;
                    push_bits(&mut output, &head.decoded_output_model_frame_bits)?;
                }
            }
            push_u32(&mut output, position.logits.target_token);
            push_u32(&mut output, position.logits.target_logit_bits);
            push_u32(&mut output, position.logits.maximum_logit_bits);
            output.extend_from_slice(&position.logits.logsumexp_bits.to_le_bytes());
            output.extend_from_slice(&position.logits.target_nll_bits.to_le_bytes());
            push_usize(&mut output, position.logits.top_logits.len())?;
            for ranked in &position.logits.top_logits {
                push_u32(&mut output, ranked.token);
                push_u32(&mut output, ranked.logit_bits);
            }
        }
        Ok(output)
    }

    pub fn trace_cid(&self) -> Result<String, R4SoftmaxTeacherTraceError> {
        Ok(format!(
            "blake3:{}",
            blake3::hash(&self.canonical_bytes()?).to_hex()
        ))
    }
}

#[derive(Clone, Debug)]
struct PendingHeadTrace {
    query_gauge_bits: Option<Vec<u32>>,
    transported_keys: BTreeMap<u32, Vec<u32>>,
    transported_values: BTreeMap<u32, Vec<u32>>,
    top_support: Option<Vec<(u32, u32)>>,
    weighted_value_aggregate_query_gauge_bits: Option<Vec<u32>>,
    decoded_output_model_frame_bits: Option<Vec<u32>>,
}

impl PendingHeadTrace {
    fn new() -> Self {
        Self {
            query_gauge_bits: None,
            transported_keys: BTreeMap::new(),
            transported_values: BTreeMap::new(),
            top_support: None,
            weighted_value_aggregate_query_gauge_bits: None,
            decoded_output_model_frame_bits: None,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingLayerTrace {
    projected_qkv: Option<ProjectedQkvTrace>,
    heads: BTreeMap<u32, PendingHeadTrace>,
}

impl PendingLayerTrace {
    fn new() -> Self {
        Self {
            projected_qkv: None,
            heads: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct PendingPositionTrace {
    position: u32,
    input_token: u32,
    frame_table_offset: u16,
    layers: BTreeMap<u32, PendingLayerTrace>,
}

/// Mutable owner behind the externally retained trace handle.
#[derive(Debug)]
pub struct R4SoftmaxTeacherTraceCollector {
    identity: R4SoftmaxTeacherTraceIdentity,
    bounds: R4SoftmaxTeacherTraceBounds,
    positions: Vec<R4SoftmaxPositionTrace>,
    pending: Option<PendingPositionTrace>,
    fault: Option<String>,
}

impl R4SoftmaxTeacherTraceCollector {
    fn new(
        identity: R4SoftmaxTeacherTraceIdentity,
        bounds: R4SoftmaxTeacherTraceBounds,
    ) -> Result<Self, R4SoftmaxTeacherTraceError> {
        validate_identity(&identity)?;
        bounds.validate()?;
        Ok(Self {
            identity,
            bounds,
            positions: Vec::with_capacity(bounds.maximum_positions),
            pending: None,
            fault: None,
        })
    }

    fn fail(&mut self, error: impl fmt::Display) {
        if self.fault.is_none() {
            self.fault = Some(error.to_string());
        }
    }

    fn status(&self) -> Result<(), R4SoftmaxTeacherTraceError> {
        match &self.fault {
            Some(reason) => Err(R4SoftmaxTeacherTraceError::Invalid(reason.clone())),
            None => Ok(()),
        }
    }

    fn reset(&mut self) {
        self.positions.clear();
        self.pending = None;
        self.fault = None;
    }

    fn begin_position(
        &mut self,
        token: usize,
        position: usize,
        frame_table_offset: u16,
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.status()?;
        if self.pending.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "a new trace position began before the prior position was sealed with logits"
                    .to_owned(),
            ));
        }
        if position != self.positions.len() || position >= self.bounds.maximum_positions {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "trace positions must be sequential and bounded: expected {}, received {position}, bound {}",
                self.positions.len(),
                self.bounds.maximum_positions
            )));
        }
        if token >= self.bounds.vocabulary {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "input token {token} exceeds vocabulary {}",
                self.bounds.vocabulary
            )));
        }
        self.pending = Some(PendingPositionTrace {
            position: usize_u32(position, "position")?,
            input_token: usize_u32(token, "input token")?,
            frame_table_offset,
            layers: BTreeMap::new(),
        });
        Ok(())
    }

    fn record_projected_qkv(
        &mut self,
        context: CausalAttentionProjectionContext,
        query: &[f32],
        key: &[f32],
        value: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_projection_context(context)?;
        let query_width = self.bounds.query_width()?;
        let key_value_width = self.bounds.key_value_width()?;
        require_width(query, query_width, "projected query")?;
        require_width(key, key_value_width, "projected key")?;
        require_width(value, key_value_width, "projected value")?;
        let layer = self.pending_layer_mut(context.layer)?;
        if layer.projected_qkv.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "layer {} projected QKV was presented more than once",
                context.layer
            )));
        }
        layer.projected_qkv = Some(ProjectedQkvTrace {
            query_bits: finite_f32_bits(query, "projected query")?,
            key_bits: finite_f32_bits(key, "projected key")?,
            value_bits: finite_f32_bits(value, "projected value")?,
        });
        Ok(())
    }

    fn record_query(
        &mut self,
        context: CausalAttentionHeadContext,
        output: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_head_context(context)?;
        require_width(output, self.bounds.head_size, "query-gauge query")?;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head.query_gauge_bits.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "query for layer {} head {} was presented more than once",
                context.layer, context.head
            )));
        }
        head.query_gauge_bits = Some(finite_f32_bits(output, "query-gauge query")?);
        Ok(())
    }

    fn record_key(
        &mut self,
        context: CausalAttentionSourceContext,
        output: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_source_context(context)?;
        require_width(output, self.bounds.head_size, "transported key")?;
        let bits = finite_f32_bits(output, "transported key")?;
        let source_position = usize_u32(context.source_position, "source position")?;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head
            .transported_keys
            .insert(source_position, bits)
            .is_some()
        {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "key for layer {} head {} source {} was presented more than once",
                context.layer, context.head, context.source_position
            )));
        }
        Ok(())
    }

    fn record_value(
        &mut self,
        context: CausalAttentionSourceContext,
        output: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_source_context(context)?;
        require_width(output, self.bounds.head_size, "transported value")?;
        let bits = finite_f32_bits(output, "transported value")?;
        let source_position = usize_u32(context.source_position, "source position")?;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head
            .transported_values
            .insert(source_position, bits)
            .is_some()
        {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "value for layer {} head {} source {} was presented more than once",
                context.layer, context.head, context.source_position
            )));
        }
        Ok(())
    }

    fn record_support(
        &mut self,
        context: CausalAttentionHeadContext,
        packed_keys: &[f32],
        weights: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_head_context(context)?;
        let prefix_len = context.query_position.checked_add(1).ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid("causal prefix length overflowed".to_owned())
        })?;
        require_width(weights, prefix_len, "attention weights")?;
        let packed_width = prefix_len
            .checked_mul(self.bounds.head_size)
            .ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid(
                    "packed transported-key width overflowed".to_owned(),
                )
            })?;
        require_width(packed_keys, packed_width, "packed transported keys")?;
        finite_f32_bits(packed_keys, "packed transported keys")?;
        let ranked = top_support_positions(weights)?;
        let head_size = self.bounds.head_size;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head.top_support.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "support for layer {} head {} was presented more than once",
                context.layer, context.head
            )));
        }
        for &(source_position, _) in &ranked {
            let source = source_position as usize;
            let start = source * head_size;
            let end = start + head_size;
            let recorded = head.transported_keys.get(&source_position).ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid(format!(
                    "top support source {source_position} has no recorded transported key"
                ))
            })?;
            if recorded.as_slice()
                != finite_f32_bits(&packed_keys[start..end], "top support transported key")?
            {
                return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                    "packed key differs from the transported key at source {source_position}"
                )));
            }
        }
        head.top_support = Some(ranked);
        Ok(())
    }

    fn record_weighted_aggregate(
        &mut self,
        context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_head_context(context)?;
        let prefix_len = context.query_position.checked_add(1).ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid("causal prefix length overflowed".to_owned())
        })?;
        require_width(weights, prefix_len, "centroid weights")?;
        let packed_width = prefix_len
            .checked_mul(self.bounds.head_size)
            .ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid(
                    "packed transported-value width overflowed".to_owned(),
                )
            })?;
        require_width(packed_values, packed_width, "packed transported values")?;
        require_width(output, self.bounds.head_size, "weighted value aggregate")?;
        finite_f32_bits(weights, "centroid weights")?;
        finite_f32_bits(packed_values, "packed transported values")?;
        let head_size = self.bounds.head_size;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head.weighted_value_aggregate_query_gauge_bits.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "weighted aggregate for layer {} head {} was presented more than once",
                context.layer, context.head
            )));
        }
        let support = head.top_support.as_ref().ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid(
                "weighted aggregate arrived before attention support".to_owned(),
            )
        })?;
        for &(source_position, weight_bits) in support {
            let source = source_position as usize;
            if weights[source].to_bits() != weight_bits {
                return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                    "centroid weight differs from the scored weight at source {source_position}"
                )));
            }
            let start = source * head_size;
            let end = start + head_size;
            let recorded = head
                .transported_values
                .get(&source_position)
                .ok_or_else(|| {
                    R4SoftmaxTeacherTraceError::Invalid(format!(
                        "top support source {source_position} has no recorded transported value"
                    ))
                })?;
            if recorded.as_slice()
                != finite_f32_bits(&packed_values[start..end], "top support transported value")?
            {
                return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                    "packed value differs from the transported value at source {source_position}"
                )));
            }
        }
        head.weighted_value_aggregate_query_gauge_bits =
            Some(finite_f32_bits(output, "weighted value aggregate")?);
        Ok(())
    }

    fn record_decoded_output(
        &mut self,
        context: CausalAttentionHeadContext,
        output: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_head_context(context)?;
        require_width(output, self.bounds.head_size, "decoded head output")?;
        let head = self.pending_head_mut(context.layer, context.head)?;
        if head.decoded_output_model_frame_bits.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "decoded output for layer {} head {} was presented more than once",
                context.layer, context.head
            )));
        }
        head.decoded_output_model_frame_bits =
            Some(finite_f32_bits(output, "decoded head output")?);
        Ok(())
    }

    fn complete_position(
        &mut self,
        position: usize,
        target_token: u32,
        logits: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.status()?;
        let pending = self.pending.clone().ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid(
                "no pending trace position is available to seal".to_owned(),
            )
        })?;
        if pending.position as usize != position {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "attempted to seal position {position} while position {} is pending",
                pending.position
            )));
        }
        if target_token as usize >= self.bounds.vocabulary {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "target token {target_token} exceeds vocabulary {}",
                self.bounds.vocabulary
            )));
        }
        let layers = finalize_layers(&pending, self.bounds)?;
        let logit_trace = build_logit_trace(logits, target_token, self.bounds.vocabulary)?;
        let completed = R4SoftmaxPositionTrace {
            position: pending.position,
            input_token: pending.input_token,
            frame_table_offset: pending.frame_table_offset,
            layers,
            logits: logit_trace,
        };
        validate_position(&completed, position, self.bounds)?;
        self.positions.push(completed);
        self.pending = None;
        Ok(())
    }

    fn snapshot(&self) -> Result<R4SoftmaxTeacherTrace, R4SoftmaxTeacherTraceError> {
        self.status()?;
        if self.pending.is_some() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "the final position has not been sealed with logits".to_owned(),
            ));
        }
        let trace = R4SoftmaxTeacherTrace {
            identity: self.identity.clone(),
            bounds: self.bounds,
            positions: self.positions.clone(),
        };
        trace.validate()?;
        Ok(trace)
    }

    fn require_projection_context(
        &self,
        context: CausalAttentionProjectionContext,
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_pending_position(context.query_position)?;
        if context.layer >= self.bounds.layers
            || context.query_heads != self.bounds.query_heads
            || context.key_value_heads != self.bounds.key_value_heads
            || context.head_size != self.bounds.head_size
        {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "projection context does not match trace bounds at layer {} position {}",
                context.layer, context.query_position
            )));
        }
        Ok(())
    }

    fn require_head_context(
        &self,
        context: CausalAttentionHeadContext,
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_pending_position(context.query_position)?;
        if context.layer >= self.bounds.layers || context.head >= self.bounds.query_heads {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "head context ({},{}) exceeds trace shape ({},{})",
                context.layer, context.head, self.bounds.layers, self.bounds.query_heads
            )));
        }
        Ok(())
    }

    fn require_source_context(
        &self,
        context: CausalAttentionSourceContext,
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.require_pending_position(context.query_position)?;
        if context.layer >= self.bounds.layers
            || context.head >= self.bounds.query_heads
            || context.source_position > context.query_position
        {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "source context ({},{},{},{}) is outside the bounded causal trace",
                context.layer, context.head, context.query_position, context.source_position
            )));
        }
        Ok(())
    }

    fn require_pending_position(
        &self,
        query_position: usize,
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        let pending = self.pending.as_ref().ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid(
                "attention hook arrived before begin_position".to_owned(),
            )
        })?;
        if pending.position as usize != query_position {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "attention hook position {query_position} does not match pending position {}",
                pending.position
            )));
        }
        Ok(())
    }

    fn pending_layer_mut(
        &mut self,
        layer: usize,
    ) -> Result<&mut PendingLayerTrace, R4SoftmaxTeacherTraceError> {
        let layer = usize_u32(layer, "layer")?;
        Ok(self
            .pending
            .as_mut()
            .ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid(
                    "attention hook arrived before begin_position".to_owned(),
                )
            })?
            .layers
            .entry(layer)
            .or_insert_with(PendingLayerTrace::new))
    }

    fn pending_head_mut(
        &mut self,
        layer: usize,
        head: usize,
    ) -> Result<&mut PendingHeadTrace, R4SoftmaxTeacherTraceError> {
        let head = usize_u32(head, "head")?;
        Ok(self
            .pending_layer_mut(layer)?
            .heads
            .entry(head)
            .or_insert_with(PendingHeadTrace::new))
    }
}

/// Cloneable completion/snapshot handle retained after the transport is moved
/// into a type-erased decoder session.
#[derive(Clone, Debug)]
pub struct R4SoftmaxTeacherTraceHandle {
    collector: Arc<Mutex<R4SoftmaxTeacherTraceCollector>>,
}

impl R4SoftmaxTeacherTraceHandle {
    pub fn complete_position(
        &self,
        position: usize,
        target_token: u32,
        logits: &[f32],
    ) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.lock()?
            .complete_position(position, target_token, logits)
    }

    pub fn snapshot(&self) -> Result<R4SoftmaxTeacherTrace, R4SoftmaxTeacherTraceError> {
        self.lock()?.snapshot()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, R4SoftmaxTeacherTraceError> {
        self.snapshot()?.canonical_bytes()
    }

    pub fn trace_cid(&self) -> Result<String, R4SoftmaxTeacherTraceError> {
        self.snapshot()?.trace_cid()
    }

    pub fn status(&self) -> Result<(), R4SoftmaxTeacherTraceError> {
        self.lock()?.status()
    }

    fn lock(
        &self,
    ) -> Result<MutexGuard<'_, R4SoftmaxTeacherTraceCollector>, R4SoftmaxTeacherTraceError> {
        self.collector
            .lock()
            .map_err(|_| R4SoftmaxTeacherTraceError::Poisoned)
    }
}

/// Transparent R4 transport decorator with externally reachable trace state.
#[derive(Debug)]
pub struct TracingR4SpinTransport {
    inner: R4SpinCausalAttentionTransport,
    collector: Arc<Mutex<R4SoftmaxTeacherTraceCollector>>,
    local_fault: Option<String>,
}

impl TracingR4SpinTransport {
    pub fn new(
        inner: R4SpinCausalAttentionTransport,
        identity: R4SoftmaxTeacherTraceIdentity,
        bounds: R4SoftmaxTeacherTraceBounds,
    ) -> Result<(Self, R4SoftmaxTeacherTraceHandle), R4SoftmaxTeacherTraceError> {
        let collector = Arc::new(Mutex::new(R4SoftmaxTeacherTraceCollector::new(
            identity, bounds,
        )?));
        let handle = R4SoftmaxTeacherTraceHandle {
            collector: Arc::clone(&collector),
        };
        Ok((
            Self {
                inner,
                collector,
                local_fault: None,
            },
            handle,
        ))
    }

    pub fn inner(&self) -> &R4SpinCausalAttentionTransport {
        &self.inner
    }

    fn record(
        &mut self,
        operation: impl FnOnce(
            &mut R4SoftmaxTeacherTraceCollector,
        ) -> Result<(), R4SoftmaxTeacherTraceError>,
    ) {
        if self.local_fault.is_some() {
            return;
        }
        let collector = Arc::clone(&self.collector);
        match collector.lock() {
            Ok(mut collector) => {
                if let Err(error) = operation(&mut collector) {
                    collector.fail(&error);
                }
            }
            Err(_) => {
                self.local_fault = Some("R4 softmax trace collector mutex was poisoned".to_owned())
            }
        };
    }

    fn trace_status(&self) -> Result<(), String> {
        if let Some(reason) = &self.local_fault {
            return Err(reason.clone());
        }
        let collector = self
            .collector
            .lock()
            .map_err(|_| "R4 softmax trace collector mutex was poisoned".to_owned())?;
        collector.status().map_err(|error| error.to_string())
    }
}

impl CausalAttentionTransport for TracingR4SpinTransport {
    fn reset(&mut self) {
        self.inner.reset();
        self.local_fault = None;
        let collector = Arc::clone(&self.collector);
        match collector.lock() {
            Ok(mut collector) => collector.reset(),
            Err(_) => {
                self.local_fault = Some("R4 softmax trace collector mutex was poisoned".to_owned())
            }
        };
    }

    fn policy_identity(&self) -> &str {
        CausalAttentionTransport::policy_identity(&self.inner)
    }

    fn implementation_evidence(&self) -> Result<Option<String>, String> {
        CausalAttentionTransport::implementation_evidence(&self.inner)
    }

    fn status(&self) -> Result<(), String> {
        CausalAttentionTransport::status(&self.inner)?;
        self.trace_status()
    }

    fn begin_position(&mut self, token: usize, position: usize) {
        CausalAttentionTransport::begin_position(&mut self.inner, token, position);
        let frame = self.inner.frame_table_offset(position);
        match frame {
            Ok(frame_table_offset) => self
                .record(|collector| collector.begin_position(token, position, frame_table_offset)),
            Err(error) => {
                self.local_fault = Some(format!(
                    "R4 trace could not bind frame offset for position {position}: {error}"
                ));
            }
        }
    }

    fn transform_projected_qkv_before_rope(
        &mut self,
        context: CausalAttentionProjectionContext,
        query: &mut [f32],
        key: &mut [f32],
        value: &mut [f32],
    ) {
        CausalAttentionTransport::transform_projected_qkv_before_rope(
            &mut self.inner,
            context,
            query,
            key,
            value,
        );
        self.record(|collector| collector.record_projected_qkv(context, query, key, value));
    }

    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        CausalAttentionTransport::transform_query(&mut self.inner, context, input, output);
        self.record(|collector| collector.record_query(context, output));
    }

    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        CausalAttentionTransport::transport_key(&mut self.inner, context, input, output);
        self.record(|collector| collector.record_key(context, output));
    }

    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        CausalAttentionTransport::transport_value(&mut self.inner, context, input, output);
        self.record(|collector| collector.record_value(context, output));
    }

    fn score_and_normalize(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        canonical_math: bool,
    ) {
        CausalAttentionTransport::score_and_normalize(
            &mut self.inner,
            context,
            query,
            packed_keys,
            output_weights,
            canonical_math,
        );
        self.record(|collector| collector.record_support(context, packed_keys, output_weights));
    }

    fn weighted_value_centroid(
        &mut self,
        context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) {
        CausalAttentionTransport::weighted_value_centroid(
            &mut self.inner,
            context,
            weights,
            packed_values,
            output,
        );
        self.record(|collector| {
            collector.record_weighted_aggregate(context, weights, packed_values, output)
        });
    }

    fn output_to_model_frame(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        CausalAttentionTransport::output_to_model_frame(&mut self.inner, context, input, output);
        self.record(|collector| collector.record_decoded_output(context, output));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum R4SoftmaxTeacherTraceError {
    Invalid(String),
    Poisoned,
}

impl fmt::Display for R4SoftmaxTeacherTraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => {
                write!(formatter, "invalid R4 softmax teacher trace: {reason}")
            }
            Self::Poisoned => {
                formatter.write_str("R4 softmax teacher trace collector was poisoned")
            }
        }
    }
}

impl std::error::Error for R4SoftmaxTeacherTraceError {}

fn finalize_layers(
    pending: &PendingPositionTrace,
    bounds: R4SoftmaxTeacherTraceBounds,
) -> Result<Vec<R4SoftmaxLayerTrace>, R4SoftmaxTeacherTraceError> {
    if pending.layers.len() != bounds.layers {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "position {} captured {} layers; {} required",
            pending.position,
            pending.layers.len(),
            bounds.layers
        )));
    }
    let mut layers = Vec::with_capacity(bounds.layers);
    for layer_index in 0..bounds.layers {
        let layer_key = usize_u32(layer_index, "layer")?;
        let pending_layer = pending.layers.get(&layer_key).ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid(format!(
                "position {} is missing layer {layer_index}",
                pending.position
            ))
        })?;
        let projected_qkv = pending_layer.projected_qkv.clone().ok_or_else(|| {
            R4SoftmaxTeacherTraceError::Invalid(format!(
                "position {} layer {layer_index} has no projected QKV",
                pending.position
            ))
        })?;
        if pending_layer.heads.len() != bounds.query_heads {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "position {} layer {layer_index} captured {} heads; {} required",
                pending.position,
                pending_layer.heads.len(),
                bounds.query_heads
            )));
        }
        let mut heads = Vec::with_capacity(bounds.query_heads);
        for head_index in 0..bounds.query_heads {
            let head_key = usize_u32(head_index, "head")?;
            let pending_head = pending_layer.heads.get(&head_key).ok_or_else(|| {
                R4SoftmaxTeacherTraceError::Invalid(format!(
                    "position {} layer {layer_index} is missing head {head_index}",
                    pending.position
                ))
            })?;
            let current = pending.position;
            let top_support = pending_head
                .top_support
                .as_ref()
                .ok_or_else(|| {
                    R4SoftmaxTeacherTraceError::Invalid(format!(
                        "position {} layer {layer_index} head {head_index} has no attention support",
                        pending.position
                    ))
                })?
                .iter()
                .map(|&(source_position, weight_bits)| {
                    Ok(R4SoftmaxTraceSupport {
                        source_position,
                        weight_bits,
                        transported_key_bits: pending_head
                            .transported_keys
                            .get(&source_position)
                            .cloned()
                            .ok_or_else(|| {
                                R4SoftmaxTeacherTraceError::Invalid(format!(
                                    "top support source {source_position} has no transported key"
                                ))
                            })?,
                        transported_value_bits: pending_head
                            .transported_values
                            .get(&source_position)
                            .cloned()
                            .ok_or_else(|| {
                                R4SoftmaxTeacherTraceError::Invalid(format!(
                                    "top support source {source_position} has no transported value"
                                ))
                            })?,
                    })
                })
                .collect::<Result<Vec<_>, R4SoftmaxTeacherTraceError>>()?;
            heads.push(R4SoftmaxHeadTrace {
                head: head_key,
                query_gauge_bits: pending_head.query_gauge_bits.clone().ok_or_else(|| {
                    R4SoftmaxTeacherTraceError::Invalid(format!(
                        "position {} layer {layer_index} head {head_index} has no query",
                        pending.position
                    ))
                })?,
                current_key_query_gauge_bits: pending_head
                    .transported_keys
                    .get(&current)
                    .cloned()
                    .ok_or_else(|| {
                        R4SoftmaxTeacherTraceError::Invalid(format!(
                            "position {} layer {layer_index} head {head_index} has no current key",
                            pending.position
                        ))
                    })?,
                current_value_query_gauge_bits: pending_head
                    .transported_values
                    .get(&current)
                    .cloned()
                    .ok_or_else(|| {
                        R4SoftmaxTeacherTraceError::Invalid(format!(
                            "position {} layer {layer_index} head {head_index} has no current value",
                            pending.position
                        ))
                    })?,
                top_support,
                weighted_value_aggregate_query_gauge_bits: pending_head
                    .weighted_value_aggregate_query_gauge_bits
                    .clone()
                    .ok_or_else(|| {
                        R4SoftmaxTeacherTraceError::Invalid(format!(
                            "position {} layer {layer_index} head {head_index} has no weighted aggregate",
                            pending.position
                        ))
                    })?,
                decoded_output_model_frame_bits: pending_head
                    .decoded_output_model_frame_bits
                    .clone()
                    .ok_or_else(|| {
                        R4SoftmaxTeacherTraceError::Invalid(format!(
                            "position {} layer {layer_index} head {head_index} has no decoded output",
                            pending.position
                        ))
                    })?,
            });
        }
        layers.push(R4SoftmaxLayerTrace {
            layer: layer_key,
            projected_qkv,
            heads,
        });
    }
    Ok(layers)
}

fn build_logit_trace(
    logits: &[f32],
    target_token: u32,
    vocabulary: usize,
) -> Result<R4SoftmaxLogitTrace, R4SoftmaxTeacherTraceError> {
    require_width(logits, vocabulary, "decoder logits")?;
    if target_token as usize >= vocabulary {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "target token {target_token} exceeds vocabulary {vocabulary}"
        )));
    }
    let mut maximum = f32::NEG_INFINITY;
    let mut top =
        Vec::<R4SoftmaxRankedLogit>::with_capacity(R4_SOFTMAX_TRACE_LOGIT_CAP.min(vocabulary));
    for (token, &logit) in logits.iter().enumerate() {
        if !logit.is_finite() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "decoder logit {token} is not finite"
            )));
        }
        maximum = maximum.max(logit);
        let candidate = R4SoftmaxRankedLogit {
            token: usize_u32(token, "logit token")?,
            logit_bits: logit.to_bits(),
        };
        let insert_at = top
            .iter()
            .position(|existing| ranked_logit_before(candidate, *existing))
            .unwrap_or(top.len());
        if insert_at < R4_SOFTMAX_TRACE_LOGIT_CAP {
            top.insert(insert_at, candidate);
            if top.len() > R4_SOFTMAX_TRACE_LOGIT_CAP {
                top.pop();
            }
        }
    }
    let maximum_f64 = f64::from(maximum);
    let shifted_sum = logits
        .iter()
        .map(|logit| (f64::from(*logit) - maximum_f64).exp())
        .sum::<f64>();
    if !shifted_sum.is_finite() || shifted_sum <= 0.0 {
        return Err(R4SoftmaxTeacherTraceError::Invalid(
            "decoder logsumexp denominator is invalid".to_owned(),
        ));
    }
    let logsumexp = maximum_f64 + shifted_sum.ln();
    let target_logit = logits[target_token as usize];
    let target_nll = logsumexp - f64::from(target_logit);
    if !logsumexp.is_finite() || !target_nll.is_finite() || target_nll < -1.0e-12 {
        return Err(R4SoftmaxTeacherTraceError::Invalid(
            "decoder logsumexp or target NLL is invalid".to_owned(),
        ));
    }
    Ok(R4SoftmaxLogitTrace {
        target_token,
        target_logit_bits: target_logit.to_bits(),
        maximum_logit_bits: maximum.to_bits(),
        logsumexp_bits: logsumexp.to_bits(),
        target_nll_bits: target_nll.max(0.0).to_bits(),
        top_logits: top,
    })
}

fn top_support_positions(weights: &[f32]) -> Result<Vec<(u32, u32)>, R4SoftmaxTeacherTraceError> {
    if weights.is_empty() {
        return Err(R4SoftmaxTeacherTraceError::Invalid(
            "attention support is empty".to_owned(),
        ));
    }
    let mut ranked = Vec::<(u32, u32)>::with_capacity(R4_SOFTMAX_TRACE_SUPPORT_CAP);
    for (position, &weight) in weights.iter().enumerate() {
        if !weight.is_finite() || weight < 0.0 {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "attention weight {position} is negative or non-finite"
            )));
        }
        let candidate = (
            usize_u32(position, "attention source position")?,
            weight.to_bits(),
        );
        let insert_at = ranked
            .iter()
            .position(|existing| ranked_support_before(candidate, *existing))
            .unwrap_or(ranked.len());
        if insert_at < R4_SOFTMAX_TRACE_SUPPORT_CAP {
            ranked.insert(insert_at, candidate);
            if ranked.len() > R4_SOFTMAX_TRACE_SUPPORT_CAP {
                ranked.pop();
            }
        }
    }
    Ok(ranked)
}

fn ranked_support_before(left: (u32, u32), right: (u32, u32)) -> bool {
    match f32::from_bits(left.1).total_cmp(&f32::from_bits(right.1)) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => left.0 < right.0,
    }
}

fn ranked_logit_before(left: R4SoftmaxRankedLogit, right: R4SoftmaxRankedLogit) -> bool {
    match left.logit().total_cmp(&right.logit()) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => left.token < right.token,
    }
}

fn validate_identity(
    identity: &R4SoftmaxTeacherTraceIdentity,
) -> Result<(), R4SoftmaxTeacherTraceError> {
    for (label, value) in [
        ("source CID", identity.source_cid.as_str()),
        ("tokenizer CID", identity.tokenizer_cid.as_str()),
        (
            "attention policy CID",
            identity.attention_policy_cid.as_str(),
        ),
        ("corpus CID", identity.corpus_cid.as_str()),
        (
            "construction partition id",
            identity.construction_partition_id.as_str(),
        ),
        ("document id", identity.document_id.as_str()),
        ("document text CID", identity.document_text_cid.as_str()),
    ] {
        if value.is_empty() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "{label} must not be empty"
            )));
        }
        u32::try_from(value.len()).map_err(|_| {
            R4SoftmaxTeacherTraceError::Invalid(format!(
                "{label} exceeds the canonical string domain"
            ))
        })?;
    }
    Ok(())
}

fn validate_position(
    position: &R4SoftmaxPositionTrace,
    expected_position: usize,
    bounds: R4SoftmaxTeacherTraceBounds,
) -> Result<(), R4SoftmaxTeacherTraceError> {
    if position.position as usize != expected_position {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "trace position {} appears at ordinal {expected_position}",
            position.position
        )));
    }
    if position.input_token as usize >= bounds.vocabulary
        || position.logits.target_token as usize >= bounds.vocabulary
    {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "position {expected_position} contains a token outside the vocabulary"
        )));
    }
    if position.layers.len() != bounds.layers {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "position {expected_position} has {} layers; {} required",
            position.layers.len(),
            bounds.layers
        )));
    }
    let query_width = bounds.query_width()?;
    let key_value_width = bounds.key_value_width()?;
    for (expected_layer, layer) in position.layers.iter().enumerate() {
        if layer.layer as usize != expected_layer || layer.heads.len() != bounds.query_heads {
            return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                "position {expected_position} layer layout is incomplete at {expected_layer}"
            )));
        }
        validate_bits(
            &layer.projected_qkv.query_bits,
            query_width,
            "projected query",
        )?;
        validate_bits(
            &layer.projected_qkv.key_bits,
            key_value_width,
            "projected key",
        )?;
        validate_bits(
            &layer.projected_qkv.value_bits,
            key_value_width,
            "projected value",
        )?;
        for (expected_head, head) in layer.heads.iter().enumerate() {
            if head.head as usize != expected_head {
                return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                    "position {expected_position} layer {expected_layer} head order is not canonical"
                )));
            }
            for (label, bits) in [
                ("query-gauge query", head.query_gauge_bits.as_slice()),
                (
                    "current query-gauge key",
                    head.current_key_query_gauge_bits.as_slice(),
                ),
                (
                    "current query-gauge value",
                    head.current_value_query_gauge_bits.as_slice(),
                ),
                (
                    "query-gauge weighted aggregate",
                    head.weighted_value_aggregate_query_gauge_bits.as_slice(),
                ),
                (
                    "decoded model-frame output",
                    head.decoded_output_model_frame_bits.as_slice(),
                ),
            ] {
                validate_bits(bits, bounds.head_size, label)?;
            }
            let expected_support = R4_SOFTMAX_TRACE_SUPPORT_CAP.min(expected_position + 1);
            if head.top_support.len() != expected_support {
                return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                    "position {expected_position} layer {expected_layer} head {expected_head} has {} support entries; {expected_support} required",
                    head.top_support.len()
                )));
            }
            for (index, support) in head.top_support.iter().enumerate() {
                if support.source_position as usize > expected_position {
                    return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
                        "support reads future position {} from query {expected_position}",
                        support.source_position
                    )));
                }
                let weight = support.weight();
                if !weight.is_finite() || weight < 0.0 {
                    return Err(R4SoftmaxTeacherTraceError::Invalid(
                        "support weight is negative or non-finite".to_owned(),
                    ));
                }
                validate_bits(
                    &support.transported_key_bits,
                    bounds.head_size,
                    "support transported key",
                )?;
                validate_bits(
                    &support.transported_value_bits,
                    bounds.head_size,
                    "support transported value",
                )?;
                if index > 0 {
                    let prior = &head.top_support[index - 1];
                    if ranked_support_before(
                        (support.source_position, support.weight_bits),
                        (prior.source_position, prior.weight_bits),
                    ) {
                        return Err(R4SoftmaxTeacherTraceError::Invalid(
                            "attention support is not in canonical weight/position order"
                                .to_owned(),
                        ));
                    }
                }
            }
        }
    }
    validate_logits(&position.logits, bounds.vocabulary)
}

fn validate_logits(
    logits: &R4SoftmaxLogitTrace,
    vocabulary: usize,
) -> Result<(), R4SoftmaxTeacherTraceError> {
    let target = logits.target_logit();
    let maximum = logits.maximum_logit();
    let logsumexp = logits.logsumexp();
    let nll = logits.target_nll();
    if !target.is_finite()
        || !maximum.is_finite()
        || !logsumexp.is_finite()
        || !nll.is_finite()
        || nll < 0.0
        || logsumexp < f64::from(maximum)
        || (logsumexp - f64::from(target) - nll).abs() > 1.0e-12
    {
        return Err(R4SoftmaxTeacherTraceError::Invalid(
            "logit summary is internally inconsistent".to_owned(),
        ));
    }
    let expected = R4_SOFTMAX_TRACE_LOGIT_CAP.min(vocabulary);
    if logits.top_logits.len() != expected {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "logit summary contains {} rows; {expected} required",
            logits.top_logits.len()
        )));
    }
    for (index, ranked) in logits.top_logits.iter().copied().enumerate() {
        if ranked.token as usize >= vocabulary || !ranked.logit().is_finite() {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "ranked logit is outside the vocabulary or non-finite".to_owned(),
            ));
        }
        if index > 0 && ranked_logit_before(ranked, logits.top_logits[index - 1]) {
            return Err(R4SoftmaxTeacherTraceError::Invalid(
                "top logits are not in canonical logit/token order".to_owned(),
            ));
        }
    }
    if logits.top_logits[0].logit_bits != logits.maximum_logit_bits {
        return Err(R4SoftmaxTeacherTraceError::Invalid(
            "maximum logit does not match the first ranked logit".to_owned(),
        ));
    }
    Ok(())
}

fn finite_f32_bits(values: &[f32], label: &str) -> Result<Vec<u32>, R4SoftmaxTeacherTraceError> {
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "{label} lane {index} is not finite"
        )));
    }
    Ok(values.iter().map(|value| value.to_bits()).collect())
}

fn validate_bits(
    bits: &[u32],
    expected: usize,
    label: &str,
) -> Result<(), R4SoftmaxTeacherTraceError> {
    if bits.len() != expected {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "{label} has {} lanes; {expected} required",
            bits.len()
        )));
    }
    if let Some(index) = bits
        .iter()
        .position(|bits| !f32::from_bits(*bits).is_finite())
    {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "{label} lane {index} is not finite"
        )));
    }
    Ok(())
}

fn require_width(
    values: &[f32],
    expected: usize,
    label: &str,
) -> Result<(), R4SoftmaxTeacherTraceError> {
    if values.len() != expected {
        return Err(R4SoftmaxTeacherTraceError::Invalid(format!(
            "{label} has {} lanes; {expected} required",
            values.len()
        )));
    }
    Ok(())
}

fn usize_u32(value: usize, label: &str) -> Result<u32, R4SoftmaxTeacherTraceError> {
    u32::try_from(value).map_err(|_| {
        R4SoftmaxTeacherTraceError::Invalid(format!("{label} exceeds the u32 trace domain"))
    })
}

fn push_usize(output: &mut Vec<u8>, value: usize) -> Result<(), R4SoftmaxTeacherTraceError> {
    push_u32(output, usize_u32(value, "canonical length")?);
    Ok(())
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_string(output: &mut Vec<u8>, value: &str) -> Result<(), R4SoftmaxTeacherTraceError> {
    push_usize(output, value.len())?;
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_bits(output: &mut Vec<u8>, bits: &[u32]) -> Result<(), R4SoftmaxTeacherTraceError> {
    push_usize(output, bits.len())?;
    for value in bits {
        push_u32(output, *value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::helm_d_r4_attention::R4SpinTransportIntervention;

    fn identity() -> R4SoftmaxTeacherTraceIdentity {
        R4SoftmaxTeacherTraceIdentity {
            source_cid: "blake3:source".to_owned(),
            tokenizer_cid: "blake3:tokenizer".to_owned(),
            attention_policy_cid: "blake3:policy".to_owned(),
            corpus_cid: "blake3:corpus".to_owned(),
            construction_partition_id: "d3-construction".to_owned(),
            document_id: "14".to_owned(),
            document_text_cid: "blake3:text".to_owned(),
        }
    }

    fn bounds() -> R4SoftmaxTeacherTraceBounds {
        R4SoftmaxTeacherTraceBounds {
            maximum_positions: 2,
            layers: 1,
            query_heads: 1,
            key_value_heads: 1,
            head_size: 4,
            vocabulary: 8,
        }
    }

    fn new_transport() -> (TracingR4SpinTransport, R4SoftmaxTeacherTraceHandle) {
        let inner = R4SpinCausalAttentionTransport::new(
            7,
            bounds().maximum_positions,
            R4SpinTransportIntervention::Coherent,
        )
        .expect("construct R4 transport");
        TracingR4SpinTransport::new(inner, identity(), bounds()).expect("construct trace")
    }

    fn drive_position_zero(
        transport: &mut TracingR4SpinTransport,
        handle: &R4SoftmaxTeacherTraceHandle,
        logits: &[f32],
    ) {
        CausalAttentionTransport::begin_position(transport, 1, 0);
        let projection = CausalAttentionProjectionContext {
            layer: 0,
            query_position: 0,
            query_heads: 1,
            key_value_heads: 1,
            head_size: 4,
        };
        let mut q = [0.25, -0.5, 0.75, 1.0];
        let mut k = [-0.5, 0.75, 0.25, -1.0];
        let mut v = [1.0, 2.0, 3.0, 4.0];
        CausalAttentionTransport::transform_projected_qkv_before_rope(
            transport, projection, &mut q, &mut k, &mut v,
        );
        let head = CausalAttentionHeadContext {
            layer: 0,
            head: 0,
            query_position: 0,
        };
        let source = CausalAttentionSourceContext {
            layer: 0,
            head: 0,
            query_position: 0,
            source_position: 0,
        };
        let mut query_gauge = [0.0; 4];
        let mut key_gauge = [0.0; 4];
        let mut value_gauge = [0.0; 4];
        CausalAttentionTransport::transform_query(transport, head, &q, &mut query_gauge);
        CausalAttentionTransport::transport_key(transport, source, &k, &mut key_gauge);
        CausalAttentionTransport::transport_value(transport, source, &v, &mut value_gauge);
        let mut weights = [0.0];
        CausalAttentionTransport::score_and_normalize(
            transport,
            head,
            &query_gauge,
            &key_gauge,
            &mut weights,
            true,
        );
        let mut aggregate = [0.0; 4];
        CausalAttentionTransport::weighted_value_centroid(
            transport,
            head,
            &weights,
            &value_gauge,
            &mut aggregate,
        );
        let mut decoded = [0.0; 4];
        CausalAttentionTransport::output_to_model_frame(transport, head, &aggregate, &mut decoded);
        CausalAttentionTransport::status(transport).expect("transport remains healthy");
        handle
            .complete_position(0, 2, logits)
            .expect("seal trace position");
    }

    #[test]
    fn direct_hooks_capture_complete_bounded_trace_and_canonical_identity() {
        let (mut transport, handle) = new_transport();
        let logits = [0.0, 0.5, 2.0, -1.0, 1.0, 0.25, -0.25, 0.75];
        drive_position_zero(&mut transport, &handle, &logits);

        let trace = handle.snapshot().expect("complete trace");
        assert_eq!(trace.positions.len(), 1);
        let position = &trace.positions[0];
        assert_eq!(position.position, 0);
        assert_eq!(position.input_token, 1);
        assert_eq!(position.layers.len(), 1);
        let head = &position.layers[0].heads[0];
        assert_eq!(head.query_gauge_bits.len(), 4);
        assert_eq!(head.current_key_query_gauge_bits.len(), 4);
        assert_eq!(head.current_value_query_gauge_bits.len(), 4);
        assert_eq!(head.top_support.len(), 1);
        assert_eq!(head.top_support[0].source_position, 0);
        assert_eq!(head.top_support[0].weight(), 1.0);
        assert_eq!(position.logits.top_logits[0].token, 2);
        assert_eq!(position.logits.target_token, 2);
        assert!(position.logits.target_nll() > 0.0);

        let first = trace.canonical_bytes().expect("canonical bytes");
        let replay = handle.canonical_bytes().expect("handle canonical bytes");
        assert_eq!(first, replay);
        assert_eq!(trace.trace_cid().unwrap(), handle.trace_cid().unwrap());
        assert_eq!(&first[..8], &R4_SOFTMAX_TEACHER_TRACE_MAGIC);
    }

    #[test]
    fn decorator_is_numerically_transparent_for_every_row_hook() {
        let mut raw = R4SpinCausalAttentionTransport::new(
            7,
            bounds().maximum_positions,
            R4SpinTransportIntervention::Coherent,
        )
        .unwrap();
        let (mut traced, handle) = new_transport();
        CausalAttentionTransport::begin_position(&mut raw, 1, 0);
        CausalAttentionTransport::begin_position(&mut traced, 1, 0);

        let projection = CausalAttentionProjectionContext {
            layer: 0,
            query_position: 0,
            query_heads: 1,
            key_value_heads: 1,
            head_size: 4,
        };
        let q = [0.25, -0.5, 0.75, 1.0];
        let k = [-0.5, 0.75, 0.25, -1.0];
        let v = [1.0, 2.0, 3.0, 4.0];
        let (mut raw_q, mut raw_k, mut raw_v) = (q, k, v);
        let (mut traced_q, mut traced_k, mut traced_v) = (q, k, v);
        CausalAttentionTransport::transform_projected_qkv_before_rope(
            &mut raw, projection, &mut raw_q, &mut raw_k, &mut raw_v,
        );
        CausalAttentionTransport::transform_projected_qkv_before_rope(
            &mut traced,
            projection,
            &mut traced_q,
            &mut traced_k,
            &mut traced_v,
        );
        assert_eq!(raw_q.map(f32::to_bits), traced_q.map(f32::to_bits));
        assert_eq!(raw_k.map(f32::to_bits), traced_k.map(f32::to_bits));
        assert_eq!(raw_v.map(f32::to_bits), traced_v.map(f32::to_bits));

        let head = CausalAttentionHeadContext {
            layer: 0,
            head: 0,
            query_position: 0,
        };
        let source = CausalAttentionSourceContext {
            layer: 0,
            head: 0,
            query_position: 0,
            source_position: 0,
        };
        let (mut raw_query, mut traced_query) = ([0.0; 4], [0.0; 4]);
        let (mut raw_key, mut traced_key) = ([0.0; 4], [0.0; 4]);
        let (mut raw_value, mut traced_value) = ([0.0; 4], [0.0; 4]);
        CausalAttentionTransport::transform_query(&mut raw, head, &q, &mut raw_query);
        CausalAttentionTransport::transform_query(&mut traced, head, &q, &mut traced_query);
        CausalAttentionTransport::transport_key(&mut raw, source, &k, &mut raw_key);
        CausalAttentionTransport::transport_key(&mut traced, source, &k, &mut traced_key);
        CausalAttentionTransport::transport_value(&mut raw, source, &v, &mut raw_value);
        CausalAttentionTransport::transport_value(&mut traced, source, &v, &mut traced_value);
        assert_eq!(raw_query.map(f32::to_bits), traced_query.map(f32::to_bits));
        assert_eq!(raw_key.map(f32::to_bits), traced_key.map(f32::to_bits));
        assert_eq!(raw_value.map(f32::to_bits), traced_value.map(f32::to_bits));

        let (mut raw_weights, mut traced_weights) = ([0.0], [0.0]);
        CausalAttentionTransport::score_and_normalize(
            &mut raw,
            head,
            &raw_query,
            &raw_key,
            &mut raw_weights,
            true,
        );
        CausalAttentionTransport::score_and_normalize(
            &mut traced,
            head,
            &traced_query,
            &traced_key,
            &mut traced_weights,
            true,
        );
        assert_eq!(
            raw_weights.map(f32::to_bits),
            traced_weights.map(f32::to_bits)
        );

        let (mut raw_aggregate, mut traced_aggregate) = ([0.0; 4], [0.0; 4]);
        CausalAttentionTransport::weighted_value_centroid(
            &mut raw,
            head,
            &raw_weights,
            &raw_value,
            &mut raw_aggregate,
        );
        CausalAttentionTransport::weighted_value_centroid(
            &mut traced,
            head,
            &traced_weights,
            &traced_value,
            &mut traced_aggregate,
        );
        assert_eq!(
            raw_aggregate.map(f32::to_bits),
            traced_aggregate.map(f32::to_bits)
        );

        let (mut raw_output, mut traced_output) = ([0.0; 4], [0.0; 4]);
        CausalAttentionTransport::output_to_model_frame(
            &mut raw,
            head,
            &raw_aggregate,
            &mut raw_output,
        );
        CausalAttentionTransport::output_to_model_frame(
            &mut traced,
            head,
            &traced_aggregate,
            &mut traced_output,
        );
        assert_eq!(
            raw_output.map(f32::to_bits),
            traced_output.map(f32::to_bits)
        );
        assert_eq!(
            CausalAttentionTransport::implementation_evidence(&raw).unwrap(),
            CausalAttentionTransport::implementation_evidence(&traced).unwrap()
        );
        handle
            .complete_position(0, 2, &[0.0, 0.5, 2.0, -1.0, 1.0, 0.25, -0.25, 0.75])
            .unwrap();
        handle.snapshot().unwrap();
    }

    #[test]
    fn top_support_is_bounded_and_uses_lower_position_for_exact_ties() {
        let weights = [0.1, 0.9, 0.3, 0.8, 0.8, 0.2, 0.7, 0.6, 0.5, 0.4];
        let support = top_support_positions(&weights).unwrap();
        assert_eq!(support.len(), R4_SOFTMAX_TRACE_SUPPORT_CAP);
        assert_eq!(support[0].0, 1);
        assert_eq!(support[1].0, 3);
        assert_eq!(support[2].0, 4);
        assert!(!support.iter().any(|(position, _)| *position == 0));
    }

    #[test]
    fn incomplete_or_nonfinite_rows_fail_closed() {
        let (mut transport, handle) = new_transport();
        CausalAttentionTransport::begin_position(&mut transport, 1, 0);
        let error = handle
            .complete_position(0, 2, &[0.0; 8])
            .expect_err("incomplete hooks must fail");
        assert!(error.to_string().contains("captured 0 layers"));

        let (_, fresh) = new_transport();
        let error = build_logit_trace(&[0.0, 1.0, f32::NAN, 0.0, 0.0, 0.0, 0.0, 0.0], 1, 8)
            .expect_err("non-finite logits must fail");
        assert!(error.to_string().contains("not finite"));
        assert!(fresh.snapshot().is_err());
    }
}
