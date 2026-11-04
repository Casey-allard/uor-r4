//! Exact CPU execution of the no-Hamming-read SpiralCore/FBS forward.
//!
//! The tables are copied from the parsed serving artifact through `FbsImage`.
//! Arithmetic follows the integer session's order, rounding and saturation;
//! this module does not call the serving forward and does not write its files.

use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::stack::{
    read_hamming_uorlut01, read_memory_uorlut01, stack_snap_select, FbsImage, FbsLayerImage,
    FbsMapImage, Fixed, IntegerStackModel, GROUP, PHI_Q32,
};

use crate::hamming::CpuHamming;
use crate::memory::{CpuTurnMemory, MemoryAnchors, MemoryMove};

const RESIDUAL_EXP: i32 = FbsImage::RESIDUAL_EXP;
const PRODUCT_EXP: i32 = FbsImage::PRODUCT_EXP;
const STATE_EXP: i32 = FbsImage::STATE_EXP;
const NORM_FINE_EXP: i32 = FbsImage::NORM_FINE_EXP;
const Q12_EXP: i32 = FbsImage::Q12_EXP;
const GRID_EXP_BIAS: i32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FbsForwardError {
    Invalid(String),
}

impl std::fmt::Display for FbsForwardError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for FbsForwardError {}

type Result<T> = std::result::Result<T, FbsForwardError>;

fn invalid(message: impl Into<String>) -> FbsForwardError {
    FbsForwardError::Invalid(message.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuFbsHead {
    Nearest,
    Writer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuFbsStep {
    pub token: u32,
    pub kept_cells: Vec<u32>,
    pub query_q12: Vec<i32>,
}

#[derive(Clone, Debug)]
pub(crate) struct CpuServedTrace {
    pub(crate) norm_before_final_gain: Vec<i64>,
    pub(crate) norm_before_memory: Vec<i64>,
    pub(crate) memory: Option<MemoryMove>,
}

pub const CPU_FBS_SESSION_CHECKPOINT_SCHEMA: &str = "uor-r4.cpu-fbs-session-checkpoint/2";
const CPU_FBS_SESSION_SUBSTRATE: &str = "cpu-development-research";
const CPU_FBS_SESSION_QUALIFICATION: &str = "gpu-required";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuFbsSessionShape {
    vocab: usize,
    width: usize,
    mlp: usize,
    layers: usize,
    kept_cells: usize,
    context: usize,
    raw_gates: bool,
    raw_transition: bool,
    hamming: bool,
    memory: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuFbsSessionPayload {
    artifact_sha256: String,
    shape: CpuFbsSessionShape,
    position: usize,
    state: Vec<Vec<i64>>,
    history: Vec<Vec<i32>>,
    tokens: Vec<u32>,
    hamming_payloads: Vec<Vec<i64>>,
    memory_anchors: MemoryAnchors,
    reply_start: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CpuFbsSessionCheckpoint {
    schema: String,
    substrate: String,
    qualification: String,
    state_sha256: String,
    payload: CpuFbsSessionPayload,
}

#[derive(Clone)]
struct Map {
    atoms: Vec<u16>,
    codes: Vec<[u8; 8]>,
}

impl Map {
    fn from_image(image: &FbsMapImage<'_>) -> Self {
        Self {
            atoms: image.atoms.to_vec(),
            codes: image.codes.to_vec(),
        }
    }
}

#[derive(Clone)]
struct Layer {
    rec_gain: Vec<i16>,
    mlp_gain: Vec<i16>,
    rec_in: Map,
    rec_gate: Map,
    rec_out: Map,
    mlp_gate: Map,
    mlp_up: Map,
    mlp_down: Map,
    taps: Vec<i16>,
    conv_bias: Vec<i32>,
    gate_bias: Vec<i32>,
    decay_offset: Vec<i32>,
}

impl Layer {
    fn from_image(image: &FbsLayerImage<'_>) -> Self {
        Self {
            rec_gain: image.rec_gain.to_vec(),
            mlp_gain: image.mlp_gain.to_vec(),
            rec_in: Map::from_image(&image.rec_in),
            rec_gate: Map::from_image(&image.rec_gate),
            rec_out: Map::from_image(&image.rec_out),
            mlp_gate: Map::from_image(&image.mlp_gate),
            mlp_up: Map::from_image(&image.mlp_up),
            mlp_down: Map::from_image(&image.mlp_down),
            taps: image.taps.to_vec(),
            conv_bias: image.conv_bias.to_vec(),
            gate_bias: image.gate_bias.to_vec(),
            decay_offset: image.decay_offset.to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct CpuFbsForward {
    artifact_sha256: Option<String>,
    vocab: usize,
    width: usize,
    mlp: usize,
    rms_eps: Fixed,
    layers: Vec<Layer>,
    final_gain: Vec<i16>,
    writer_table: Vec<i16>,
    roots_q12: [[i32; 8]; 16],
    kept_cells: usize,
    context: usize,
    raw_gates: bool,
    raw_transition: bool,
    position: usize,
    state: Vec<Vec<i64>>,
    history: Vec<Vec<i32>>,
    tokens: Vec<u32>,
    hamming: Option<CpuHamming>,
    hamming_payloads: Vec<Vec<i64>>,
    memory: Option<CpuTurnMemory>,
    memory_anchors: MemoryAnchors,
    reply_start: Option<usize>,
}

impl CpuFbsForward {
    pub fn from_model(model: &IntegerStackModel) -> Result<Self> {
        validate_shape(model)?;
        if model.turn_memory().is_some() {
            return Err(invalid(
                "CPU FBS forward does not yet support exact turn memory",
            ));
        }
        let image = model.fbs_image().ok_or_else(|| {
            invalid("CPU FBS forward does not yet support Hamming reads or turn memory")
        })?;
        Self::from_image(
            image,
            model.rms_epsilon(),
            model.shape().context,
            model.shape().raw_gates,
            model.shape().raw_transition,
        )
    }

    /// Parse artifact bytes and attach the Hamming reads as well as the FBS
    /// trunk. A model without reads behaves exactly as [`Self::from_model`].
    pub fn from_artifact(bytes: &[u8]) -> Result<Self> {
        let parsed = IntegerStackModel::parse(bytes)
            .map_err(|error| invalid(format!("CPU FBS artifact does not parse: {error}")))?;
        validate_shape(&parsed)?;
        let hamming = read_hamming_uorlut01(bytes)
            .map_err(|error| invalid(format!("CPU Hamming tables do not parse: {error}")))?;
        let memory = read_memory_uorlut01(bytes)
            .map_err(|error| invalid(format!("CPU turn-memory tables do not parse: {error}")))?;
        let context = parsed.shape().context;
        let model = parsed.without_hamming_read();
        let image = model
            .fbs_trunk_image()
            .ok_or_else(|| invalid("CPU FBS forward could not recover the FBS trunk"))?;
        let mut forward = Self::from_image(
            image,
            model.rms_epsilon(),
            context,
            model.shape().raw_gates,
            model.shape().raw_transition,
        )?;
        if let Some(hamming) = hamming {
            let reads = CpuHamming::new(
                &hamming,
                &forward.writer_table,
                forward.vocab,
                forward.width,
                forward.layers.len(),
                context,
            )
            .map_err(invalid)?;
            forward.hamming_payloads = reads.payload_caches(context);
            forward.hamming = Some(reads);
        }
        if let Some(memory) = memory {
            forward.memory = Some(CpuTurnMemory::new(memory, forward.width).map_err(invalid)?);
        }
        forward.artifact_sha256 = Some(uor_r4_discrete::corpus::sha256(bytes));
        Ok(forward)
    }

    fn from_image(
        image: FbsImage<'_>,
        rms_eps: Fixed,
        context: usize,
        raw_gates: bool,
        raw_transition: bool,
    ) -> Result<Self> {
        let width = image.final_gain.len();
        if width == 0 || !width.is_multiple_of(GROUP) || !width.is_multiple_of(4) {
            return Err(invalid("FBS width must be a positive multiple of 32"));
        }
        if !image.writer_table.len().is_multiple_of(width) {
            return Err(invalid("writer table length is not a multiple of width"));
        }
        let vocab = image.writer_table.len() / width;
        if vocab == 0 {
            return Err(invalid("writer table is empty"));
        }
        let mlp = image
            .layers
            .first()
            .map(|layer| layer.mlp_gate.atoms.len() * 8)
            .ok_or_else(|| invalid("FBS model has no layers"))?;
        let layers = image
            .layers
            .iter()
            .map(Layer::from_image)
            .collect::<Vec<_>>();
        for layer in &layers {
            validate_layer(layer, width, mlp)?;
        }
        if image.kept_cells == 0 || image.kept_cells > 16 {
            return Err(invalid("kept cells must be in 1..=16"));
        }
        Ok(Self {
            artifact_sha256: None,
            vocab,
            width,
            mlp,
            rms_eps,
            state: layers.iter().map(|_| vec![0; width]).collect(),
            history: layers.iter().map(|_| vec![0; 3 * width]).collect(),
            layers,
            final_gain: image.final_gain.to_vec(),
            writer_table: image.writer_table.to_vec(),
            roots_q12: *image.roots_q12,
            kept_cells: image.kept_cells,
            context,
            raw_gates,
            raw_transition,
            position: 0,
            tokens: Vec::with_capacity(context),
            hamming: None,
            hamming_payloads: Vec::new(),
            memory: None,
            memory_anchors: MemoryAnchors::default(),
            reply_start: None,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn vocabulary(&self) -> usize {
        self.vocab
    }

    pub(crate) fn token_table(&self) -> &[i16] {
        &self.writer_table
    }

    pub(crate) fn writer_roots(&self) -> &[[i32; 8]; 16] {
        &self.roots_q12
    }

    pub(crate) fn kept_cell_count(&self) -> usize {
        self.kept_cells
    }

    pub(crate) fn context_length(&self) -> usize {
        self.context
    }

    /// Mark that the next step's row is the prompt's last row and the opening move runs there.
    pub fn mark_reply_start(&mut self) {
        self.reply_start = Some(self.position + 1);
    }

    /// Stop keying and moving rows, and discard the anchors from any prior reply.
    pub fn clear_reply_start(&mut self) {
        self.reply_start = None;
        self.memory_anchors = MemoryAnchors::default();
    }

    /// Disable the artifact's turn-memory and point-key reads while retaining
    /// its trunk tables and artifact identity.
    pub fn disable_memory(&mut self) {
        self.memory = None;
        self.clear_reply_start();
    }

    /// Start a new stream with the same loaded tables.
    pub fn reset(&mut self) {
        self.position = 0;
        self.tokens.clear();
        for state in &mut self.state {
            state.fill(0);
        }
        for history in &mut self.history {
            history.fill(0);
        }
        for payloads in &mut self.hamming_payloads {
            payloads.fill(0);
        }
        self.memory_anchors = MemoryAnchors::default();
        self.reply_start = None;
    }

    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>> {
        let artifact_sha256 = self
            .artifact_sha256
            .clone()
            .ok_or_else(|| invalid("CPU FBS session has no artifact identity"))?;
        let payload = CpuFbsSessionPayload {
            artifact_sha256,
            shape: self.session_shape(),
            position: self.position,
            state: self.state.clone(),
            history: self.history.clone(),
            tokens: self.tokens.clone(),
            hamming_payloads: self.hamming_payloads.clone(),
            memory_anchors: self.memory_anchors.clone(),
            reply_start: self.reply_start,
        };
        self.validate_session_payload(&payload)?;
        let state_sha256 = uor_r4_discrete::corpus::sha256(
            &serde_json::to_vec(&payload).map_err(|error| invalid(error.to_string()))?,
        );
        serde_json::to_vec(&CpuFbsSessionCheckpoint {
            schema: CPU_FBS_SESSION_CHECKPOINT_SCHEMA.to_owned(),
            substrate: CPU_FBS_SESSION_SUBSTRATE.to_owned(),
            qualification: CPU_FBS_SESSION_QUALIFICATION.to_owned(),
            state_sha256,
            payload,
        })
        .map_err(|error| invalid(error.to_string()))
    }

    pub fn resume(checkpoint_bytes: &[u8], artifact: &[u8]) -> Result<Self> {
        let checkpoint: CpuFbsSessionCheckpoint =
            serde_json::from_slice(checkpoint_bytes).map_err(|error| invalid(error.to_string()))?;
        if checkpoint.schema != CPU_FBS_SESSION_CHECKPOINT_SCHEMA
            || checkpoint.substrate != CPU_FBS_SESSION_SUBSTRATE
            || checkpoint.qualification != CPU_FBS_SESSION_QUALIFICATION
        {
            return Err(invalid(
                "CPU FBS session checkpoint schema, substrate or qualification differs",
            ));
        }
        uor_r4_discrete::corpus::validate_sha256(&checkpoint.state_sha256)
            .map_err(|error| invalid(error.to_string()))?;
        let state_bytes =
            serde_json::to_vec(&checkpoint.payload).map_err(|error| invalid(error.to_string()))?;
        if uor_r4_discrete::corpus::sha256(&state_bytes) != checkpoint.state_sha256 {
            return Err(invalid("CPU FBS session checkpoint state hash differs"));
        }
        let artifact_sha256 = uor_r4_discrete::corpus::sha256(artifact);
        if checkpoint.payload.artifact_sha256 != artifact_sha256 {
            return Err(invalid(
                "CPU FBS session checkpoint artifact identity differs",
            ));
        }
        let mut forward = Self::from_artifact(artifact)?;
        forward.validate_session_payload(&checkpoint.payload)?;
        let payload = checkpoint.payload;
        forward.position = payload.position;
        forward.state = payload.state;
        forward.history = payload.history;
        forward.tokens = payload.tokens;
        forward.hamming_payloads = payload.hamming_payloads;
        forward.memory_anchors = payload.memory_anchors;
        forward.reply_start = payload.reply_start;
        Ok(forward)
    }

    fn session_shape(&self) -> CpuFbsSessionShape {
        CpuFbsSessionShape {
            vocab: self.vocab,
            width: self.width,
            mlp: self.mlp,
            layers: self.layers.len(),
            kept_cells: self.kept_cells,
            context: self.context,
            raw_gates: self.raw_gates,
            raw_transition: self.raw_transition,
            hamming: self.hamming.is_some(),
            memory: self.memory.is_some(),
        }
    }

    fn validate_session_payload(&self, payload: &CpuFbsSessionPayload) -> Result<()> {
        let expected_sha256 = self
            .artifact_sha256
            .as_deref()
            .ok_or_else(|| invalid("CPU FBS session has no artifact identity"))?;
        if payload.artifact_sha256 != expected_sha256
            || payload.shape != self.session_shape()
            || payload.position > self.context
            || payload.position != payload.tokens.len()
            || payload.state.len() != self.layers.len()
            || payload.history.len() != self.layers.len()
            || payload.state.iter().any(|state| state.len() != self.width)
            || payload
                .history
                .iter()
                .any(|history| history.len() != 3 * self.width)
            || payload
                .tokens
                .iter()
                .any(|&token| token as usize >= self.vocab)
        {
            return Err(invalid(
                "CPU FBS session dynamic state differs from its artifact shape",
            ));
        }
        let expected_payloads = self
            .hamming
            .as_ref()
            .map(|hamming| hamming.payload_lengths(self.context))
            .unwrap_or_default();
        if payload.hamming_payloads.len() != expected_payloads.len()
            || payload
                .hamming_payloads
                .iter()
                .zip(&expected_payloads)
                .any(|(cache, &expected)| {
                    let future_start = payload.position.saturating_mul(self.width);
                    cache.len() != expected
                        || cache
                            .get(future_start..)
                            .is_some_and(|future| future.iter().any(|&value| value != 0))
                })
        {
            return Err(invalid(
                "CPU FBS Hamming caches differ from their artifact shape",
            ));
        }
        let anchors = &payload.memory_anchors;
        let anchor_tokens_valid = anchors
            .opening
            .iter()
            .chain(&anchors.reply)
            .all(|&token| (token as usize) < self.vocab);
        let anchor_limits_valid = self
            .memory
            .as_ref()
            .map(|memory| {
                let (opening, reply) = memory.anchor_limits();
                anchors.opening.len() <= opening && anchors.reply.len() <= reply
            })
            .unwrap_or_else(|| anchors == &MemoryAnchors::default());
        let reply_valid = match payload.reply_start {
            None => anchors == &MemoryAnchors::default(),
            Some(start) => {
                start > 0
                    && start <= payload.position.saturating_add(1)
                    && start <= self.context.saturating_add(1)
                    && self.memory.as_ref().is_none_or(|memory| {
                        // A pending opening has not overwritten the anchors from the
                        // previous reply yet. Their structurally valid prior values
                        // are inert and must round-trip exactly.
                        start > payload.position
                            || anchors == &memory.anchors_at(&payload.tokens, start)
                    })
            }
        };
        if !anchor_tokens_valid || !anchor_limits_valid || !reply_valid {
            return Err(invalid(
                "CPU FBS memory anchors or reply cursor differ from their artifact shape",
            ));
        }
        Ok(())
    }

    pub fn step(&mut self, token: u32, head: CpuFbsHead) -> Result<CpuFbsStep> {
        self.step_inner(token, head, false).map(|(step, _)| step)
    }

    pub(crate) fn step_with_trace(
        &mut self,
        token: u32,
        head: CpuFbsHead,
    ) -> Result<(CpuFbsStep, CpuServedTrace)> {
        let (step, trace) = self.step_inner(token, head, true)?;
        let trace = trace.ok_or_else(|| invalid("CPU FBS memory trace is absent"))?;
        Ok((step, trace))
    }

    pub(crate) fn supports_gain_replay(&self) -> bool {
        self.hamming.is_none()
    }

    pub(crate) fn query_from_norm(&self, norm: &[i64]) -> Result<Vec<i32>> {
        if norm.len() != self.width {
            return Err(invalid("CPU FBS traced normalization width differs"));
        }
        Ok(norm
            .iter()
            .map(|&value| to_exp_i32(value, NORM_FINE_EXP, Q12_EXP).clamp(-32768, 32767))
            .collect())
    }

    pub(crate) fn writer_token_from_query(
        &self,
        query: &[i32],
        opening_used_point: bool,
    ) -> Result<u32> {
        if query.len() != self.width {
            return Err(invalid("CPU FBS traced query width differs"));
        }
        let mut token = writer(
            query,
            &self.writer_table,
            self.vocab,
            &self.roots_q12,
            self.kept_cells,
        )
        .0;
        if opening_used_point {
            if let Some((name, _)) = self
                .memory
                .as_ref()
                .and_then(|memory| memory.point_name(query))
                .transpose()
                .map_err(invalid)?
            {
                token = name;
            }
        }
        Ok(token)
    }

    pub(crate) fn replay_final_gain(
        &self,
        norm_before_final_gain: &[i64],
        gain: &[i16],
        memory: Option<MemoryMove>,
        opening_weights: &[i32],
        reply_weights: Option<&[i32]>,
    ) -> Result<(Vec<i32>, u32)> {
        if norm_before_final_gain.len() != self.width
            || gain.len() != self.width
            || opening_weights.len() != self.width
            || reply_weights.is_some_and(|weights| weights.len() != self.width)
        {
            return Err(invalid("CPU FBS final-gain replay dimensions differ"));
        }
        let mut norm = norm_before_final_gain.to_vec();
        apply_gain(&mut norm, gain);
        if let Some(memory) = memory {
            if let Some(stored) = memory.stored {
                let weights = if memory.opening {
                    opening_weights
                } else {
                    reply_weights.ok_or_else(|| {
                        invalid("CPU FBS final-gain replay lost its reply weights")
                    })?
                };
                crate::memory::move_point_with_weight(
                    stored,
                    &self.writer_table,
                    weights,
                    &mut norm,
                );
            }
        }
        let query = self.query_from_norm(&norm)?;
        let opening_used_point =
            memory.is_some_and(|memory| memory.opening && memory.stored.is_none());
        let token = self.writer_token_from_query(&query, opening_used_point)?;
        Ok((query, token))
    }

    fn step_inner(
        &mut self,
        token: u32,
        head: CpuFbsHead,
        capture_memory_trace: bool,
    ) -> Result<(CpuFbsStep, Option<CpuServedTrace>)> {
        if token as usize >= self.vocab {
            return Err(invalid(format!(
                "token {token} is outside vocabulary {}",
                self.vocab
            )));
        }
        if self.position >= self.context {
            return Err(invalid(format!(
                "context is full at {} positions",
                self.context
            )));
        }
        self.tokens.push(token);
        let d = self.width;
        let row = token as usize * d;
        let mut x: Vec<i32> = self.writer_table[row..row + d]
            .iter()
            .map(|&value| to_exp_i32(i64::from(value), Q12_EXP, RESIDUAL_EXP))
            .collect();
        let mut scratch = vec![0i64; d.max(self.mlp)];
        let mut gate = vec![0i32; self.mlp];
        let mut up = vec![0i32; self.mlp];
        let mut act = vec![0i16; d.max(self.mlp)];

        for index in 0..self.layers.len() {
            let layer = &self.layers[index];
            rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
            apply_gain(&mut scratch[..d], &layer.rec_gain);
            let mut branches = vec![0i32; 2 * d];
            let gate_rows = d / 4 + d;
            let mut gate_out = vec![0i32; gate_rows];
            gather(&layer.rec_in, &scratch[..d], NORM_FINE_EXP, &mut branches);
            gather(&layer.rec_gate, &scratch[..d], NORM_FINE_EXP, &mut gate_out);
            let act_exp = recurrence(
                d,
                layer,
                self.raw_transition,
                self.raw_gates,
                &mut self.state[index],
                &mut self.history[index],
                &branches,
                &mut gate_out,
                &mut scratch,
                &mut act,
            );
            let mut projected = vec![0i32; d];
            gather(&layer.rec_out, &act[..d], act_exp, &mut projected);
            for (left, right) in x.iter_mut().zip(&projected) {
                *left = left.saturating_add(*right);
            }

            rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
            apply_gain(&mut scratch[..d], &layer.mlp_gain);
            gather(&layer.mlp_gate, &scratch[..d], NORM_FINE_EXP, &mut gate);
            gather(&layer.mlp_up, &scratch[..d], NORM_FINE_EXP, &mut up);
            hamilton_feed_forward(&gate, &up, &mut scratch[..self.mlp], self.raw_gates);
            let act_exp = quantize16(&scratch[..self.mlp], PRODUCT_EXP, &mut act[..self.mlp]);
            gather(&layer.mlp_down, &act[..self.mlp], act_exp, &mut projected);
            for (left, right) in x.iter_mut().zip(&projected) {
                *left = left.saturating_add(*right);
            }
            if let Some(hamming) = &self.hamming {
                hamming.stream(
                    index + 1,
                    &self.tokens,
                    &self.writer_table,
                    &mut self.hamming_payloads,
                    &mut x,
                );
            }
        }

        rms_norm_fine(&x, self.rms_eps, &mut scratch[..d]);
        let norm_before_final_gain = capture_memory_trace.then(|| scratch[..d].to_vec());
        apply_gain(&mut scratch[..d], &self.final_gain);
        if let Some(hamming) = &self.hamming {
            hamming.readout(
                &self.tokens,
                &self.writer_table,
                &mut self.hamming_payloads,
                &mut scratch[..d],
            );
        }
        let norm_before_memory = if capture_memory_trace {
            scratch[..d].to_vec()
        } else {
            Vec::new()
        };
        let memory_move: Option<MemoryMove> = match (&self.memory, self.reply_start) {
            (Some(memory), Some(reply_start)) => memory.apply(
                &self.tokens,
                self.position,
                reply_start,
                &mut self.memory_anchors,
                &self.writer_table,
                &mut scratch[..d],
            ),
            _ => None,
        };
        let query_q12: Vec<i32> = scratch[..d]
            .iter()
            .map(|&value| to_exp_i32(value, NORM_FINE_EXP, Q12_EXP).clamp(-32768, 32767))
            .collect();
        let (mut token, kept_cells) = match head {
            CpuFbsHead::Nearest => (
                nearest(&query_q12, &self.writer_table, self.vocab),
                vec![0; self.kept_cells],
            ),
            CpuFbsHead::Writer => writer(
                &query_q12,
                &self.writer_table,
                self.vocab,
                &self.roots_q12,
                self.kept_cells,
            ),
        };
        if memory_move
            .as_ref()
            .is_some_and(|memory| memory.opening && memory.stored.is_none())
        {
            if let Some((name, _)) = self
                .memory
                .as_ref()
                .and_then(|memory| memory.point_name(&query_q12))
                .transpose()
                .map_err(invalid)?
            {
                token = name;
            }
        }
        self.position += 1;
        let trace = norm_before_final_gain.map(|norm_before_final_gain| CpuServedTrace {
            norm_before_final_gain,
            norm_before_memory,
            memory: memory_move,
        });
        Ok((
            CpuFbsStep {
                token,
                kept_cells,
                query_q12,
            },
            trace,
        ))
    }
}

fn validate_shape(model: &IntegerStackModel) -> Result<()> {
    let shape = model.shape();
    if !shape.projection_fbs()
        || !shape.rec_geometric()
        || shape.rec_retention()
        || !shape.gate_hamilton()
        || !shape.rotation
        || shape.pointer.is_some()
        || shape.pattern.bytes().any(|kind| kind != b'r')
    {
        return Err(invalid(
            "CPU FBS forward requires a rotated, decay-rate, recurrence-only FBS model",
        ));
    }
    Ok(())
}

fn validate_layer(layer: &Layer, width: usize, mlp: usize) -> Result<()> {
    let lanes = width / 4;
    let gate_rows = lanes + width;
    if layer.rec_gain.len() != width
        || layer.mlp_gain.len() != width
        || layer.taps.len() != 4 * width
        || layer.conv_bias.len() != width
        || layer.gate_bias.len() != gate_rows
        || layer.decay_offset.len() != lanes
        || layer.rec_in.atoms.len() != 2 * width / 8
        || layer.rec_gate.atoms.len() != gate_rows / 8
        || layer.rec_out.atoms.len() != width / 8
        || layer.mlp_gate.atoms.len() != mlp / 8
        || layer.mlp_up.atoms.len() != mlp / 8
        || layer.mlp_down.atoms.len() != width / 8
    {
        return Err(invalid("FBS layer table dimensions do not match the trunk"));
    }
    for map in [
        &layer.rec_in,
        &layer.rec_gate,
        &layer.rec_out,
        &layer.mlp_gate,
        &layer.mlp_up,
        &layer.mlp_down,
    ] {
        if map.atoms.len() != map.codes.len() {
            return Err(invalid("FBS route atom/code lengths differ"));
        }
    }
    validate_atoms(&layer.rec_in, width / 8)?;
    validate_atoms(&layer.rec_gate, width / 8)?;
    validate_atoms(&layer.rec_out, width / 8)?;
    validate_atoms(&layer.mlp_gate, width / 8)?;
    validate_atoms(&layer.mlp_up, width / 8)?;
    validate_atoms(&layer.mlp_down, mlp / 8)?;
    Ok(())
}

fn validate_atoms(map: &Map, input_blocks: usize) -> Result<()> {
    if map
        .atoms
        .iter()
        .any(|&atom| usize::from(atom) >= input_blocks)
    {
        return Err(invalid("FBS route atom lies outside its input"));
    }
    Ok(())
}

fn gather<T: Copy + Into<i64>>(map: &Map, input: &[T], from_exp: i32, out: &mut [i32]) {
    for ((block, &atom), codes) in out.chunks_exact_mut(8).zip(&map.atoms).zip(&map.codes) {
        let base = usize::from(atom) * 8;
        for (slot, &code) in block.iter_mut().zip(codes) {
            let value: i64 = input[base + usize::from(code & 7)].into();
            let value = if code & 8 == 0 {
                value
            } else {
                value.saturating_neg()
            };
            *slot = to_exp_i32(value, from_exp, RESIDUAL_EXP);
        }
    }
}

fn apply_gain(values: &mut [i64], codes: &[i16]) {
    for (value, &code) in values.iter_mut().zip(codes) {
        *value = grid_apply(*value, code);
    }
}

#[allow(clippy::too_many_arguments)]
fn recurrence(
    width: usize,
    layer: &Layer,
    raw_transition: bool,
    raw_gates: bool,
    state: &mut [i64],
    history: &mut [i32],
    branches: &[i32],
    gate_out: &mut [i32],
    scratch: &mut [i64],
    act: &mut [i16],
) -> i32 {
    for (value, &bias) in gate_out.iter_mut().zip(&layer.gate_bias) {
        *value = value.saturating_add(bias);
    }
    let (drive, gate) = branches.split_at(width);
    let (tap0, rest) = layer.taps.split_at(width);
    let (tap1, rest) = rest.split_at(width);
    let (tap2, tap3) = rest.split_at(width);
    let (back1, rest) = history.split_at(width);
    let (back2, back3) = rest.split_at(width);
    for i in 0..width {
        scratch[i] = i64::from(layer.conv_bias[i])
            .wrapping_add(grid_apply(i64::from(drive[i]), tap0[i]))
            .wrapping_add(grid_apply(i64::from(back1[i]), tap1[i]))
            .wrapping_add(grid_apply(i64::from(back2[i]), tap2[i]))
            .wrapping_add(grid_apply(i64::from(back3[i]), tap3[i]));
    }
    history.copy_within(..2 * width, width);
    history[..width].copy_from_slice(drive);

    let lanes = width / 4;
    let (decays, rotations) = gate_out.split_at(lanes);
    for lane in 0..lanes {
        let rate = decays[lane].saturating_add(layer.decay_offset[lane]);
        let idx = ((i64::from(rate.clamp(0, 1 << 16)) >> 12) as usize).min(15);
        let lambda = 1u64 << (30 - idx);
        let raw = [
            rotations[lane * 4],
            rotations[lane * 4 + 1],
            rotations[lane * 4 + 2],
            rotations[lane * 4 + 3],
        ];
        let transition = if raw_transition {
            raw_rotation(raw, lambda, 4_295)
        } else {
            snap_rotation(raw, lambda)
        };
        let held = [
            state[lane * 4],
            state[lane * 4 + 1],
            state[lane * 4 + 2],
            state[lane * 4 + 3],
        ];
        let moved = hamilton(transition, held);
        let keep = FbsImage::GEOMETRIC_KEEP_Q31[idx];
        for channel in 0..4 {
            let drive = i128::from(keep) * i128::from(scratch[lane * 4 + channel]);
            state[lane * 4 + channel] = shift_wide(moved[channel].wrapping_add(drive << 16), 31);
        }
    }

    for lane in 0..lanes {
        let raw_gate = [
            gate[lane * 4],
            gate[lane * 4 + 1],
            gate[lane * 4 + 2],
            gate[lane * 4 + 3],
        ];
        let q = if raw_gates {
            raw_rotation(raw_gate, 1u64 << 31, 0)
        } else {
            snap_rotation(raw_gate, 1u64 << 31)
        };
        let rhs = [
            shift(state[lane * 4], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 1], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 2], RESIDUAL_EXP - STATE_EXP),
            shift(state[lane * 4 + 3], RESIDUAL_EXP - STATE_EXP),
        ];
        let product = hamilton(q, rhs);
        for channel in 0..4 {
            scratch[lane * 4 + channel] = shift_wide(product[channel], 15);
        }
    }
    quantize16(&scratch[..width], PRODUCT_EXP, &mut act[..width])
}

fn hamilton_feed_forward(gate: &[i32], up: &[i32], out: &mut [i64], raw_gates: bool) {
    for ((out_lane, gate_lane), up_lane) in out
        .chunks_exact_mut(4)
        .zip(gate.chunks_exact(4))
        .zip(up.chunks_exact(4))
    {
        let raw = [gate_lane[0], gate_lane[1], gate_lane[2], gate_lane[3]];
        let q = if raw_gates {
            raw_rotation(raw, 1u64 << 31, 0)
        } else {
            snap_rotation(raw, 1u64 << 31)
        };
        let product = hamilton(
            q,
            [
                i64::from(up_lane[0]),
                i64::from(up_lane[1]),
                i64::from(up_lane[2]),
                i64::from(up_lane[3]),
            ],
        );
        for (slot, value) in out_lane.iter_mut().zip(product) {
            *slot = shift_wide(value, 15);
        }
    }
}

fn nearest(query: &[i32], table: &[i16], vocab: usize) -> u32 {
    let width = query.len();
    let mut best = (u32::MAX, i64::MIN);
    for token in 0..vocab {
        let row = &table[token * width..(token + 1) * width];
        let score = query
            .iter()
            .zip(row)
            .fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
            })
            .wrapping_neg();
        if score > best.1 {
            best = (token as u32, score);
        }
    }
    best.0
}

fn writer(
    query: &[i32],
    table: &[i16],
    vocab: usize,
    roots: &[[i32; 8]; 16],
    kept_cells: usize,
) -> (u32, Vec<u32>) {
    let width = query.len();
    let score = |left: &[i32], right: &[i32]| -> i64 {
        left.iter()
            .zip(right)
            .fold(0i64, |sum, (&left, &right)| {
                sum.wrapping_add(i64::from(left.abs_diff(right)))
            })
            .wrapping_neg()
    };
    let mut cells = [0i64; 256];
    for cell in 0..256 {
        cells[cell] = score(&query[..8], &roots[cell / 16])
            .wrapping_add(score(&query[width - 8..], &roots[cell % 16]));
    }
    let mut kept = vec![0u32; kept_cells];
    for slot in &mut kept {
        let mut best = 0;
        for cell in 1..256 {
            if cells[cell] > cells[best] {
                best = cell;
            }
        }
        *slot = best as u32;
        cells[best] = i64::MIN;
    }
    let mut best = (u32::MAX, i64::MIN);
    for &cell in &kept {
        let mut token = cell as usize;
        while token < vocab {
            let row = &table[token * width..(token + 1) * width];
            let candidate = query
                .iter()
                .zip(row)
                .fold(0i64, |sum, (&left, &right)| {
                    sum.wrapping_add(i64::from(left.abs_diff(i32::from(right))))
                })
                .wrapping_neg();
            if candidate > best.1 || (candidate == best.1 && (token as u32) < best.0) {
                best = (token as u32, candidate);
            }
            token += 256;
        }
    }
    (best.0, kept)
}

fn quantize16(values: &[i64], exp: i32, out: &mut [i16]) -> i32 {
    let max = values
        .iter()
        .fold(0u64, |max, value| max.max(value.unsigned_abs()));
    let shift_count = (bit_length(max) - 15).max(0);
    for (out, &value) in out.iter_mut().zip(values) {
        *out = shift(value, shift_count).clamp(-32767, 32767) as i16;
    }
    exp + shift_count
}

fn rms_norm_fine(x: &[i32], eps: Fixed, out: &mut [i64]) {
    let max = x
        .iter()
        .fold(0u32, |max, value| max.max(value.unsigned_abs()));
    let shrink = (bit_length(u64::from(max)) - 24).max(0);
    let mut sum = 0u64;
    for (slot, &value) in out.iter_mut().zip(x) {
        let y = shift(i64::from(value), shrink);
        *slot = y;
        sum = sum.wrapping_add(square(y));
    }
    let mean = sum / x.len().max(1) as u64;
    let square_exp = (RESIDUAL_EXP + shrink) << 1;
    let eps_int = shift(eps.mantissa, square_exp - eps.exp).max(0);
    let mean_square = mean.wrapping_add(eps_int as u64).max(1) as u128;
    let root = isqrt(mean_square << 68).max(1);
    let reciprocal = (1u128 << 96) / root;
    for value in out {
        let magnitude = u128::from(value.unsigned_abs()).wrapping_mul(reciprocal);
        let product = if *value < 0 {
            magnitude.wrapping_neg() as i128
        } else {
            magnitude as i128
        };
        *value = (product.wrapping_add(1i128 << 37) >> 38).clamp(i64::MIN as i128, i64::MAX as i128)
            as i64;
    }
}

fn snap_rotation(raw: [i32; 4], lambda: u64) -> [i64; 4] {
    let root = &H4_ROOT_COEFFICIENTS[stack_snap_select(raw)];
    let lambda_phi = lambda.wrapping_add(
        lambda
            .wrapping_mul((PHI_Q32 - (1 << 32)) as u64)
            .wrapping_add(1 << 31)
            >> 32,
    );
    std::array::from_fn(|channel| {
        let sum = small_mul(lambda as i64, root[channel][0])
            .wrapping_add(small_mul(lambda_phi as i64, root[channel][1]));
        shift(sum, 1)
    })
}

fn raw_rotation(raw: [i32; 4], lambda: u64, epsilon: u128) -> [i64; 4] {
    let square = raw.iter().fold(0u128, |sum, &value| {
        sum.wrapping_add(u128::from(square(i64::from(value))))
    });
    let norm = isqrt(square.wrapping_add(epsilon).wrapping_shl(32));
    std::array::from_fn(|channel| {
        let magnitude = (u128::from(raw[channel].unsigned_abs()) << 46)
            .checked_div(norm)
            .unwrap_or(0) as i128;
        let unit = if raw[channel] < 0 {
            magnitude.wrapping_neg()
        } else {
            magnitude
        };
        (i128::from(lambda).wrapping_mul(unit) >> 30) as i64
    })
}

fn small_mul(value: i64, coefficient: i8) -> i64 {
    match coefficient {
        -2 => (value << 1).wrapping_neg(),
        -1 => value.wrapping_neg(),
        0 => 0,
        1 => value,
        2 => value << 1,
        _ => {
            let mut result = 0i64;
            let mut addend = value;
            let mut remaining = coefficient.unsigned_abs();
            while remaining != 0 {
                if remaining & 1 != 0 {
                    result = result.wrapping_add(addend);
                }
                remaining >>= 1;
                addend <<= 1;
            }
            if coefficient < 0 {
                result.wrapping_neg()
            } else {
                result
            }
        }
    }
}

fn hamilton(a: [i64; 4], b: [i64; 4]) -> [i128; 4] {
    let mult = |index: usize, other: usize| i128::from(a[index]).wrapping_mul(i128::from(b[other]));
    [
        mult(0, 0)
            .wrapping_sub(mult(1, 1))
            .wrapping_sub(mult(2, 2))
            .wrapping_sub(mult(3, 3)),
        mult(0, 1)
            .wrapping_add(mult(1, 0))
            .wrapping_add(mult(2, 3))
            .wrapping_sub(mult(3, 2)),
        mult(0, 2)
            .wrapping_sub(mult(1, 3))
            .wrapping_add(mult(2, 0))
            .wrapping_add(mult(3, 1)),
        mult(0, 3)
            .wrapping_add(mult(1, 2))
            .wrapping_sub(mult(2, 1))
            .wrapping_add(mult(3, 0)),
    ]
}

fn grid_apply(value: i64, code: i16) -> i64 {
    if code == 0 {
        return 0;
    }
    let negative = code < 0;
    let magnitude = code.unsigned_abs();
    let exponent = i32::from(magnitude >> 4) - GRID_EXP_BIAS;
    let scaled = shift(scale_16_plus(value, (magnitude & 15) as u8), 4 - exponent);
    if negative {
        scaled.saturating_neg()
    } else {
        scaled
    }
}

fn scale_16_plus(value: i64, remainder: u8) -> i64 {
    let select = |bit: u32| value & i64::from((remainder >> bit) & 1).wrapping_neg();
    let value = (value << 1).wrapping_add(select(3));
    let value = (value << 1).wrapping_add(select(2));
    let value = (value << 1).wrapping_add(select(1));
    (value << 1).wrapping_add(select(0))
}

fn shift(value: i64, amount: i32) -> i64 {
    if amount > 0 {
        if amount >= 63 {
            return if value < 0 { -1 } else { 0 };
        }
        value.saturating_add(1i64 << (amount - 1)) >> amount
    } else if amount < 0 {
        let amount = amount.unsigned_abs().min(63);
        let limit = i64::MAX >> amount;
        if value > limit {
            i64::MAX
        } else if value < -limit {
            i64::MIN
        } else {
            value << amount
        }
    } else {
        value
    }
}

fn shift_wide(value: i128, amount: u32) -> i64 {
    let rounded = if amount == 0 {
        value
    } else {
        value.wrapping_add(1i128 << (amount - 1)) >> amount
    };
    rounded.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn to_exp_i32(value: i64, from: i32, to: i32) -> i32 {
    shift(value, to - from).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn square(value: i64) -> u64 {
    let magnitude = value.unsigned_abs();
    magnitude.wrapping_mul(magnitude)
}

fn isqrt(value: u128) -> u128 {
    if value == 0 {
        return 0;
    }
    let mut remainder = value;
    let mut root = 0u128;
    let mut bit = 1u128 << ((127 - value.leading_zeros()) & !1);
    while bit != 0 {
        if remainder >= root + bit {
            remainder -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

fn bit_length(value: u64) -> i32 {
    64 - value.leading_zeros() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, export_fbs_trunk_uorlut01_with,
        FbsExportSnaps, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, IntegerStackModel,
        PointKeyImage, MEMORY_PAD,
    };

    fn fixture(seed: u64, layers: usize, nearest: bool) -> (IntegerStackModel, CpuFbsForward) {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = nearest;
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_model(&model).unwrap();
        (model, cpu)
    }

    fn assert_parity(seed: u64, layers: usize, head: CpuFbsHead) {
        let (model, mut cpu) = fixture(seed, layers, head == CpuFbsHead::Nearest);
        let mut session = model.session();
        for token in [0, 1, 17, 63, 255, 511, 33, 7] {
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(
                actual.query_q12, expected.query_q12,
                "seed {seed} token {token}"
            );
            assert_eq!(actual.token, expected.token, "seed {seed} token {token}");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "seed {seed} token {token}"
            );
        }
    }

    fn hamming_fixture(
        seed: u64,
        layers: usize,
        head: CpuFbsHead,
        copy_scale: f32,
        type_scale: f32,
        depths: &[usize],
    ) -> (IntegerStackModel, CpuFbsForward, IntegerStackModel) {
        hamming_fixture_with_snaps(
            seed,
            layers,
            head,
            copy_scale,
            type_scale,
            depths,
            FbsExportSnaps::SNAPPED,
        )
    }

    fn hamming_fixture_with_snaps(
        seed: u64,
        layers: usize,
        head: CpuFbsHead,
        copy_scale: f32,
        type_scale: f32,
        depths: &[usize],
        snaps: FbsExportSnaps,
    ) -> (IntegerStackModel, CpuFbsForward, IntegerStackModel) {
        hamming_fixture_with_snaps_and_copy_key(
            seed,
            layers,
            head,
            copy_scale,
            type_scale,
            depths,
            snaps,
            &[1, 1],
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn hamming_fixture_with_snaps_and_copy_key(
        seed: u64,
        layers: usize,
        head: CpuFbsHead,
        copy_scale: f32,
        type_scale: f32,
        depths: &[usize],
        snaps: FbsExportSnaps,
        copy_keys: &[usize],
    ) -> (IntegerStackModel, CpuFbsForward, IntegerStackModel) {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = head == CpuFbsHead::Nearest;
        let free_bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let free = IntegerStackModel::parse(&free_bytes).unwrap();

        let mut hamming = synthetic_hamming(snapshot.width, layers, 4, seed ^ 194);
        hamming.reads = hamming
            .depths
            .iter()
            .zip(hamming.reads)
            .filter(|(depth, _)| depths.contains(depth))
            .map(|(_, read)| read)
            .collect();
        hamming.depths = depths.to_vec();
        // One key for every copy head is `copy_key`; differing keys are `copy_keys` (claude-268).
        if copy_keys.windows(2).all(|w| w[0] == w[1]) {
            hamming.copy_key = copy_keys[0];
        } else {
            hamming.copy_keys = Some(copy_keys.to_vec());
        }
        for read in &mut hamming.reads {
            let (copy, types) = read.weight.split_at_mut(snapshot.width);
            copy.iter_mut().for_each(|weight| *weight *= copy_scale);
            types.iter_mut().for_each(|weight| *weight *= type_scale);
        }
        snapshot.hamming = Some(hamming);
        let bytes = export_fbs_trunk_uorlut01_with(&snapshot, snaps).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        (model, cpu, free)
    }

    fn repeating_tokens() -> [u32; 16] {
        std::array::from_fn(|row| {
            if row < 12 {
                ((row * 7 + 3) % 11) as u32
            } else {
                ((row - 12) * 5 + 1) as u32
            }
        })
    }

    fn assert_hamming_parity(
        seed: u64,
        depths: &[usize],
        copy_scale: f32,
        type_scale: f32,
        head: CpuFbsHead,
    ) -> usize {
        assert_hamming_parity_with_snaps(
            seed,
            depths,
            copy_scale,
            type_scale,
            head,
            FbsExportSnaps::SNAPPED,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn assert_hamming_parity_with_snaps(
        seed: u64,
        depths: &[usize],
        copy_scale: f32,
        type_scale: f32,
        head: CpuFbsHead,
        snaps: FbsExportSnaps,
    ) -> usize {
        let (model, mut cpu, free) =
            hamming_fixture_with_snaps(seed, 3, head, copy_scale, type_scale, depths, snaps);
        let mut session = model.session();
        let mut free_session = free.session();
        let mut changed = 0;
        for (row, token) in repeating_tokens().into_iter().enumerate() {
            let expected = session.step_native(token).unwrap();
            let free_step = free_session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(
                actual.query_q12, expected.query_q12,
                "row {row}: complete query"
            );
            assert_eq!(actual.token, expected.token, "row {row}: token");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "row {row}: kept cells"
            );
            changed += usize::from(actual.query_q12 != free_step.query_q12);
        }
        changed
    }

    fn assert_hamming_parity_for_copy_key(
        seed: u64,
        copy_keys: &[usize],
        head: CpuFbsHead,
    ) -> usize {
        let (model, mut cpu, free) = hamming_fixture_with_snaps_and_copy_key(
            seed,
            3,
            head,
            1.0,
            0.0,
            &[1, 3],
            FbsExportSnaps::SNAPPED,
            copy_keys,
        );
        let mut session = model.session();
        let mut free_session = free.session();
        let mut changed = 0;
        for (row, token) in repeating_tokens().into_iter().enumerate() {
            let expected = session.step_native(token).unwrap();
            let free_step = free_session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(
                actual.query_q12, expected.query_q12,
                "copy keys {copy_keys:?}, row {row}: complete query"
            );
            assert_eq!(
                actual.token, expected.token,
                "copy keys {copy_keys:?}, row {row}: token"
            );
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "copy keys {copy_keys:?}, row {row}: kept cells"
            );
            changed += usize::from(actual.query_q12 != free_step.query_q12);
        }
        changed
    }

    fn memory_script() -> [u32; 6] {
        [0, 7, 11, 42, 77, 99]
    }

    fn memory_fixture(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        point: bool,
        opening_hit: bool,
    ) -> (IntegerStackModel, CpuFbsForward) {
        memory_fixture_with(
            seed,
            head,
            hamming,
            opening_hit,
            1.0,
            point.then(|| vec![99]),
        )
    }

    fn memory_fixture_with(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        opening_hit: bool,
        memory_weight: f32,
        point_targets: Option<Vec<u32>>,
    ) -> (IntegerStackModel, CpuFbsForward) {
        let bytes = memory_artifact_with(
            seed,
            head,
            hamming,
            opening_hit,
            memory_weight,
            point_targets,
        );
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        (model, cpu)
    }

    fn memory_artifact_with(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        opening_hit: bool,
        memory_weight: f32,
        point_targets: Option<Vec<u32>>,
    ) -> Vec<u8> {
        memory_artifact_with_snaps(
            seed,
            head,
            hamming,
            opening_hit,
            memory_weight,
            point_targets,
            FbsExportSnaps::SNAPPED,
        )
    }

    fn memory_artifact_with_snaps(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        opening_hit: bool,
        memory_weight: f32,
        point_targets: Option<Vec<u32>>,
        snaps: FbsExportSnaps,
    ) -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = head == CpuFbsHead::Nearest;
        if hamming {
            let mut reads = synthetic_hamming(snapshot.width, 3, 4, seed ^ 351);
            reads
                .reads
                .iter_mut()
                .for_each(|read| read.weight.fill(1.0));
            snapshot.hamming = Some(reads);
        }
        let opening_key = if opening_hit {
            vec![7, 11]
        } else {
            vec![13, 17]
        };
        let weight = vec![memory_weight; snapshot.width];
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: 192,
            opening: vec![FbsMemoryEntry {
                key: opening_key,
                value: 42,
            }],
            opening_weight: weight.clone(),
            reply: Some(FbsReplyTable {
                turn: 2,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 1, MEMORY_PAD, 42],
                        value: 77,
                    },
                    FbsMemoryEntry {
                        key: vec![7, 11, MEMORY_PAD, 2, MEMORY_PAD, 77],
                        value: 99,
                    },
                ],
                weight,
            }),
        });
        let mut bytes = export_fbs_trunk_uorlut01_with(&snapshot, snaps).unwrap();
        if let Some(targets) = point_targets {
            bytes = add_point_key_uorlut01(
                &bytes,
                &PointKeyImage {
                    width: snapshot.width,
                    points: vec![0; snapshot.width * targets.len()],
                    targets,
                    vote_k: 1,
                },
            )
            .unwrap();
        }
        bytes
    }

    fn assert_memory_parity(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        point: bool,
        opening_hit: bool,
    ) -> Vec<u32> {
        assert_memory_parity_with_weight(seed, head, hamming, point, opening_hit, 1.0)
    }

    fn assert_memory_parity_with_weight(
        seed: u64,
        head: CpuFbsHead,
        hamming: bool,
        point: bool,
        opening_hit: bool,
        memory_weight: f32,
    ) -> Vec<u32> {
        let (model, mut cpu) = memory_fixture_with(
            seed,
            head,
            hamming,
            opening_hit,
            memory_weight,
            point.then(|| vec![99]),
        );
        run_memory_parity(&model, &mut cpu, head)
    }

    fn run_memory_parity(
        model: &IntegerStackModel,
        cpu: &mut CpuFbsForward,
        head: CpuFbsHead,
    ) -> Vec<u32> {
        let mut session = model.session();
        let mut served = Vec::new();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row + 1 == 3 {
                session.mark_reply_start();
                cpu.mark_reply_start();
            }
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, head).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12, "row {row}: query");
            assert_eq!(actual.token, expected.token, "row {row}: token");
            assert_eq!(
                actual.kept_cells, expected.kept_cells,
                "row {row}: kept cells"
            );
            served.push(actual.token);
        }
        served
    }

    fn window_memory_fixture() -> (IntegerStackModel, CpuFbsForward) {
        let mut snapshot = synthetic_fbs_trunk_with(
            107,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = true;
        let weight = vec![1.0; snapshot.width];
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 2,
            key_window: 4,
            opening: vec![FbsMemoryEntry {
                key: vec![20],
                value: 42,
            }],
            opening_weight: weight.clone(),
            reply: Some(FbsReplyTable {
                turn: 2,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![20, MEMORY_PAD, 1, MEMORY_PAD, 42],
                        value: 77,
                    },
                    FbsMemoryEntry {
                        key: vec![20, MEMORY_PAD, 2, MEMORY_PAD, 77],
                        value: 99,
                    },
                    FbsMemoryEntry {
                        key: vec![MEMORY_PAD, 1, MEMORY_PAD, 99],
                        value: 123,
                    },
                ],
                weight,
            }),
        });
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        (model, cpu)
    }

    #[test]
    fn one_layer_nearest_matches_serving_forward() {
        assert_parity(7, 1, CpuFbsHead::Nearest);
    }

    #[test]
    fn three_layer_nearest_matches_serving_forward() {
        assert_parity(11, 3, CpuFbsHead::Nearest);
    }

    #[test]
    fn writer_head_matches_serving_forward() {
        assert_parity(13, 2, CpuFbsHead::Writer);
    }

    #[test]
    fn six_layers_match_serving_forward() {
        assert_parity(17, 6, CpuFbsHead::Nearest);
    }

    #[test]
    fn copy_heads_at_both_hooks_match_serving() {
        let changed = assert_hamming_parity(41, &[1, 3], 1.0, 0.0, CpuFbsHead::Writer);
        assert!(changed > 0, "the copy reads must move the complete query");
    }

    #[test]
    fn copy_key_two_heads_match_serving_at_both_hooks() {
        let changed = assert_hamming_parity_for_copy_key(261, &[2, 2], CpuFbsHead::Writer);
        assert!(
            changed > 0,
            "the two-token copy reads must move the complete query"
        );
    }

    /// claude-269: a record with a copy key per head (`copy_keys`) reads each copy head by its own
    /// key at both hooks, as the integer session does, with either head taking the two-token key.
    #[test]
    fn mixed_copy_key_heads_match_serving_at_both_hooks() {
        for keys in [[2, 1], [1, 2]] {
            let changed = assert_hamming_parity_for_copy_key(261, &keys, CpuFbsHead::Writer);
            assert!(
                changed > 0,
                "{keys:?}: the copy reads must move the complete query"
            );
        }
    }

    #[test]
    fn type_heads_at_both_hooks_match_serving() {
        let changed = assert_hamming_parity(43, &[1, 3], 0.0, 1.0, CpuFbsHead::Nearest);
        assert!(changed > 0, "the type reads must move the complete query");
    }

    #[test]
    fn combined_copy_and_type_reads_match_serving() {
        let changed = assert_hamming_parity(47, &[1, 3], 1.0, 1.0, CpuFbsHead::Writer);
        assert!(
            changed > 0,
            "the combined reads must move the complete query"
        );
    }

    #[test]
    fn erased_copy_codes_change_the_complete_query() {
        let (_, mut expected, _) = hamming_fixture(49, 3, CpuFbsHead::Nearest, 1.0, 1.0, &[1, 3]);
        let mut corrupted = expected.clone();
        corrupted.hamming.as_mut().unwrap().erase_copy_codes();
        for token in repeating_tokens().into_iter().take(3) {
            expected.step(token, CpuFbsHead::Nearest).unwrap();
            corrupted.step(token, CpuFbsHead::Nearest).unwrap();
        }
        let expected = expected.step(7, CpuFbsHead::Nearest).unwrap();
        let corrupted = corrupted.step(7, CpuFbsHead::Nearest).unwrap();
        assert_ne!(expected.query_q12, corrupted.query_q12);
    }

    #[test]
    fn stream_only_and_readout_only_reads_match_serving() {
        assert!(assert_hamming_parity(53, &[1], 1.0, 1.0, CpuFbsHead::Nearest) > 0);
        assert!(assert_hamming_parity(59, &[3], 1.0, 1.0, CpuFbsHead::Nearest) > 0);
    }

    #[test]
    fn zero_weight_hamming_matches_the_read_free_trunk() {
        let changed = assert_hamming_parity(61, &[1, 3], 0.0, 0.0, CpuFbsHead::Writer);
        assert_eq!(changed, 0, "zero read weights must add nothing");
    }

    #[test]
    fn opening_and_reply_memory_moves_match_serving() {
        assert_memory_parity(71, CpuFbsHead::Writer, false, false, true);
        assert_memory_parity(73, CpuFbsHead::Nearest, false, false, true);
    }

    #[test]
    fn fractional_positive_and_negative_memory_weights_match_serving() {
        assert_memory_parity_with_weight(74, CpuFbsHead::Writer, false, false, true, 0.5);
        assert_memory_parity_with_weight(75, CpuFbsHead::Nearest, false, false, true, -0.25);
    }

    #[test]
    fn point_fallback_names_the_missed_opening() {
        let served = assert_memory_parity(79, CpuFbsHead::Writer, false, true, false);
        assert_eq!(served[2], 99);
    }

    #[test]
    fn point_fallback_counts_targets_then_breaks_ties_by_smaller_id() {
        let (model, mut cpu) = memory_fixture_with(
            81,
            CpuFbsHead::Nearest,
            false,
            false,
            1.0,
            Some(vec![90, 90, 70]),
        );
        let served = run_memory_parity(&model, &mut cpu, CpuFbsHead::Nearest);
        assert_eq!(served[2], 90);

        let (model, mut cpu) = memory_fixture_with(
            82,
            CpuFbsHead::Nearest,
            false,
            false,
            1.0,
            Some(vec![30, 20]),
        );
        let served = run_memory_parity(&model, &mut cpu, CpuFbsHead::Nearest);
        assert_eq!(served[2], 20);
    }

    #[test]
    fn memory_keys_cross_windows_and_restart_after_eos() {
        let (model, mut cpu) = window_memory_fixture();
        let tokens = [9, 10, 11, 12, 1, 20, 42, 77, 99];
        let mut session = model.session();
        let mut served = Vec::new();
        for (row, token) in tokens.into_iter().enumerate() {
            if row + 1 == 6 {
                session.mark_reply_start();
                cpu.mark_reply_start();
            }
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, CpuFbsHead::Nearest).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12, "row {row}");
            assert_eq!(actual.token, expected.token, "row {row}");
            assert_eq!(actual.kept_cells, expected.kept_cells, "row {row}");
            served.push(actual.token);
        }
        assert_eq!(&served[5..], &[42, 77, 99, 123]);

        let (model, mut cpu) = window_memory_fixture();
        let mut session = model.session();
        for (row, token) in [9, 10, 11, 12, 1].into_iter().enumerate() {
            if row + 1 == 5 {
                session.mark_reply_start();
                cpu.mark_reply_start();
            }
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, CpuFbsHead::Nearest).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }
    }

    #[test]
    fn combined_hamming_and_memory_match_serving() {
        assert_memory_parity(83, CpuFbsHead::Writer, true, false, true);
        assert_memory_parity(89, CpuFbsHead::Nearest, true, true, false);
    }

    #[test]
    fn raw_quaternions_compose_with_hamming_and_memory() {
        let raw = FbsExportSnaps {
            gates: false,
            transition: false,
        };
        assert!(
            assert_hamming_parity_with_snaps(331, &[1, 3], 1.0, 1.0, CpuFbsHead::Writer, raw,) > 0
        );
        let bytes = memory_artifact_with_snaps(
            337,
            CpuFbsHead::Writer,
            true,
            true,
            0.5,
            Some(vec![99]),
            raw,
        );
        let model = IntegerStackModel::parse(&bytes).unwrap();
        let mut cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
        assert_eq!(
            run_memory_parity(&model, &mut cpu, CpuFbsHead::Writer).len(),
            memory_script().len()
        );
    }

    #[test]
    fn memory_without_a_reply_signal_or_after_clear_adds_nothing() {
        let (model, mut cpu) = memory_fixture(97, CpuFbsHead::Writer, false, false, true);
        let mut session = model.session();
        for token in memory_script() {
            let expected = session.step_native(token).unwrap();
            let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }

        let (model, mut cpu) = memory_fixture(101, CpuFbsHead::Writer, false, false, true);
        let mut session = model.session();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row + 1 == 3 {
                session.mark_reply_start();
                session.clear_reply_start();
                cpu.mark_reply_start();
                cpu.clear_reply_start();
            }
            let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
            let expected = session.step_native(token).unwrap();
            assert_eq!(actual.query_q12, expected.query_q12);
            assert_eq!(actual.token, expected.token);
            assert_eq!(actual.kept_cells, expected.kept_cells);
        }
    }

    #[test]
    fn the_trunk_view_does_not_weaken_fbs_image_refusal() {
        let (model, _) = memory_fixture(103, CpuFbsHead::Writer, true, false, true);
        assert!(model.fbs_image().is_none());
        assert!(model.fbs_trunk_image().is_some());
    }

    #[test]
    fn reset_starts_a_fresh_memory_stream() {
        let (_, mut reused) = memory_fixture(104, CpuFbsHead::Writer, true, true, true);
        for token in memory_script() {
            reused.step(token, CpuFbsHead::Writer).unwrap();
        }
        reused.reset();

        let (_, mut fresh) = memory_fixture(104, CpuFbsHead::Writer, true, true, true);
        let mut expected = Vec::new();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row + 1 == 3 {
                reused.mark_reply_start();
                fresh.mark_reply_start();
            }
            let reset_step = reused.step(token, CpuFbsHead::Writer).unwrap();
            let fresh_step = fresh.step(token, CpuFbsHead::Writer).unwrap();
            assert_eq!(reset_step, fresh_step);
            expected.push(reset_step.token);
        }
        assert!(!expected.is_empty());
    }

    const SESSION_PROCESS_CHECKPOINT: &str = "UOR_CPU_FBS_SESSION_CHECKPOINT";
    const SESSION_PROCESS_ARTIFACT: &str = "UOR_CPU_FBS_SESSION_ARTIFACT";
    const SESSION_PROCESS_OUTPUT: &str = "UOR_CPU_FBS_SESSION_OUTPUT";

    fn session_result(step: &CpuFbsStep, checkpoint: &CpuFbsForward) -> serde_json::Value {
        serde_json::json!({
            "token": step.token,
            "kept_cells": step.kept_cells,
            "query_q12": step.query_q12,
            "checkpoint_sha256": uor_r4_discrete::corpus::sha256(
                &checkpoint.checkpoint_bytes().unwrap()
            ),
        })
    }

    fn session_checkpoint_fixture() -> (Vec<u8>, Vec<u8>, CpuFbsForward) {
        let artifact =
            memory_artifact_with(105, CpuFbsHead::Writer, true, false, 0.5, Some(vec![99]));
        let mut forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        for (row, token) in memory_script().into_iter().take(3).enumerate() {
            if row == 1 {
                forward.mark_reply_start();
            }
            forward.step(token, CpuFbsHead::Writer).unwrap();
        }
        let checkpoint = forward.checkpoint_bytes().unwrap();
        (artifact, checkpoint, forward)
    }

    #[test]
    fn cpu_fbs_session_checkpoint_process_helper() {
        let Ok(checkpoint_path) = std::env::var(SESSION_PROCESS_CHECKPOINT) else {
            return;
        };
        let artifact_path = std::env::var(SESSION_PROCESS_ARTIFACT).unwrap();
        let output_path = std::env::var(SESSION_PROCESS_OUTPUT).unwrap();
        let checkpoint = std::fs::read(checkpoint_path).unwrap();
        let artifact = std::fs::read(artifact_path).unwrap();
        let mut forward = CpuFbsForward::resume(&checkpoint, &artifact).unwrap();
        let step = forward.step(77, CpuFbsHead::Writer).unwrap();
        std::fs::write(
            output_path,
            serde_json::to_vec(&session_result(&step, &forward)).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn cpu_fbs_session_survives_process_restart() {
        let (artifact, checkpoint, mut uninterrupted) = session_checkpoint_fixture();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "uor-r4-cpu-fbs-session-{}-{nonce}",
            std::process::id()
        ));
        let checkpoint_path = base.with_extension("checkpoint");
        let artifact_path = base.with_extension("d11");
        let output_path = base.with_extension("result");
        std::fs::write(&checkpoint_path, &checkpoint).unwrap();
        std::fs::write(&artifact_path, &artifact).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("fbs::tests::cpu_fbs_session_checkpoint_process_helper")
            .arg("--nocapture")
            .env(SESSION_PROCESS_CHECKPOINT, &checkpoint_path)
            .env(SESSION_PROCESS_ARTIFACT, &artifact_path)
            .env(SESSION_PROCESS_OUTPUT, &output_path)
            .env("RUST_TEST_THREADS", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "session helper failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let child: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output_path).unwrap()).unwrap();
        let expected_step = uninterrupted.step(77, CpuFbsHead::Writer).unwrap();
        assert_eq!(child, session_result(&expected_step, &uninterrupted));
        std::fs::remove_file(checkpoint_path).unwrap();
        std::fs::remove_file(artifact_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn pending_second_reply_round_trips_with_prior_anchors() {
        let artifact =
            memory_artifact_with(111, CpuFbsHead::Writer, true, true, 1.0, Some(vec![99]));
        let mut uninterrupted = CpuFbsForward::from_artifact(&artifact).unwrap();
        for (row, token) in memory_script().into_iter().take(5).enumerate() {
            if row == 1 {
                uninterrupted.mark_reply_start();
            }
            uninterrupted.step(token, CpuFbsHead::Writer).unwrap();
        }
        uninterrupted.mark_reply_start();
        let checkpoint = uninterrupted.checkpoint_bytes().unwrap();
        let mut restored = CpuFbsForward::resume(&checkpoint, &artifact).unwrap();
        assert_eq!(restored.checkpoint_bytes().unwrap(), checkpoint);
        assert_eq!(
            restored.step(99, CpuFbsHead::Writer).unwrap(),
            uninterrupted.step(99, CpuFbsHead::Writer).unwrap()
        );
        assert_eq!(
            restored.checkpoint_bytes().unwrap(),
            uninterrupted.checkpoint_bytes().unwrap()
        );
    }

    #[test]
    fn restored_hamming_and_memory_session_resets_like_a_fresh_stream() {
        let (artifact, checkpoint, _) = session_checkpoint_fixture();
        let mut restored = CpuFbsForward::resume(&checkpoint, &artifact).unwrap();
        restored.reset();
        let mut fresh = CpuFbsForward::from_artifact(&artifact).unwrap();
        for (row, token) in memory_script().into_iter().enumerate() {
            if row == 1 {
                restored.mark_reply_start();
                fresh.mark_reply_start();
            }
            assert_eq!(
                restored.step(token, CpuFbsHead::Writer).unwrap(),
                fresh.step(token, CpuFbsHead::Writer).unwrap()
            );
        }
        assert_eq!(
            restored.checkpoint_bytes().unwrap(),
            fresh.checkpoint_bytes().unwrap()
        );
    }

    #[test]
    fn cpu_fbs_session_checkpoint_refuses_identity_shape_and_state_corruption() {
        let (artifact, checkpoint, forward) = session_checkpoint_fixture();
        let restored = CpuFbsForward::resume(&checkpoint, &artifact).unwrap();
        assert_eq!(restored.checkpoint_bytes().unwrap(), checkpoint);
        let value: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
        let resealed = |value: &mut serde_json::Value| {
            let state = serde_json::to_vec(&value["payload"]).unwrap();
            value["state_sha256"] = serde_json::json!(uor_r4_discrete::corpus::sha256(&state));
            serde_json::to_vec(value).unwrap()
        };
        for mutate in [
            |value: &mut serde_json::Value| value["schema"] = serde_json::json!("wrong"),
            |value: &mut serde_json::Value| value["substrate"] = serde_json::json!("gpu"),
            |value: &mut serde_json::Value| value["qualification"] = serde_json::json!("qualified"),
            |value: &mut serde_json::Value| value["unexpected"] = serde_json::json!(true),
            |value: &mut serde_json::Value| {
                value["payload"]["artifact_sha256"] = serde_json::json!("00".repeat(32))
            },
            |value: &mut serde_json::Value| {
                value["payload"]["shape"]["width"] = serde_json::json!(96)
            },
            |value: &mut serde_json::Value| value["payload"]["position"] = serde_json::json!(2),
            |value: &mut serde_json::Value| {
                value["payload"]["state"][0].as_array_mut().unwrap().pop();
            },
            |value: &mut serde_json::Value| {
                value["payload"]["history"][0].as_array_mut().unwrap().pop();
            },
            |value: &mut serde_json::Value| {
                value["payload"]["hamming_payloads"][0]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            },
            |value: &mut serde_json::Value| {
                value["payload"]["hamming_payloads"][0][192] = serde_json::json!(1)
            },
            |value: &mut serde_json::Value| value["payload"]["tokens"][0] = serde_json::json!(9999),
            |value: &mut serde_json::Value| {
                value["payload"]["memory_anchors"]["opening"] = serde_json::json!([9999])
            },
            |value: &mut serde_json::Value| {
                value["payload"]["memory_anchors"]["opening"] = serde_json::json!([7])
            },
            |value: &mut serde_json::Value| {
                value["payload"]["reply_start"] = serde_json::Value::Null
            },
            |value: &mut serde_json::Value| value["payload"]["reply_start"] = serde_json::json!(99),
        ] {
            let mut bad = value.clone();
            mutate(&mut bad);
            assert!(CpuFbsForward::resume(&resealed(&mut bad), &artifact).is_err());
        }
        let mut bad_hash = value.clone();
        bad_hash["payload"]["position"] = serde_json::json!(0);
        assert!(CpuFbsForward::resume(&serde_json::to_vec(&bad_hash).unwrap(), &artifact).is_err());
        let mut other_artifact = artifact.clone();
        *other_artifact.last_mut().unwrap() ^= 1;
        assert!(CpuFbsForward::resume(&checkpoint, &other_artifact).is_err());
        let mut truncated = checkpoint.clone();
        truncated.pop();
        assert!(CpuFbsForward::resume(&truncated, &artifact).is_err());
        let mut trailing = checkpoint.clone();
        trailing.push(b'0');
        assert!(CpuFbsForward::resume(&trailing, &artifact).is_err());
        assert!(forward.checkpoint_bytes().is_ok());
    }

    #[test]
    fn a_route_sign_fault_changes_the_query() {
        let (_, mut forward) = fixture(19, 1, true);
        let mut corrupted = forward.clone();
        corrupted.layers[0].rec_in.codes[0][0] ^= 8;
        let expected = forward.step(5, CpuFbsHead::Nearest).unwrap();
        let altered = corrupted.step(5, CpuFbsHead::Nearest).unwrap();
        assert_eq!(expected.query_q12.len(), altered.query_q12.len());
        assert_ne!(expected.query_q12, altered.query_q12);
    }

    #[test]
    fn every_snapped_and_raw_gate_transition_combination_matches_serving() {
        let snapshot = synthetic_fbs_trunk_with(
            23,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        let tokens = [0, 1, 17, 63, 255, 511, 33, 7, 129, 256, 9, 44];
        let mut traces = Vec::new();
        for (raw_gates, raw_transition) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let bytes = export_fbs_trunk_uorlut01_with(
                &snapshot,
                FbsExportSnaps {
                    gates: !raw_gates,
                    transition: !raw_transition,
                },
            )
            .unwrap();
            let model = IntegerStackModel::parse(&bytes).unwrap();
            let mut cpu = CpuFbsForward::from_artifact(&bytes).unwrap();
            let mut serving = model.session();
            let mut trace = Vec::new();
            for token in tokens {
                let expected = serving.step_native(token).unwrap();
                let actual = cpu.step(token, CpuFbsHead::Writer).unwrap();
                let context = format!(
                    "raw gates {raw_gates}, raw transition {raw_transition}, token {token}"
                );
                assert_eq!(actual.query_q12, expected.query_q12, "{context}");
                assert_eq!(actual.token, expected.token, "{context}");
                assert_eq!(actual.kept_cells, expected.kept_cells, "{context}");
                trace.push(actual);
            }
            let checkpoint = cpu.checkpoint_bytes().unwrap();
            let mut resumed = CpuFbsForward::resume(&checkpoint, &bytes).unwrap();
            assert_eq!(resumed.checkpoint_bytes().unwrap(), checkpoint);
            let expected = serving.step_native(101).unwrap();
            let restored = resumed.step(101, CpuFbsHead::Writer).unwrap();
            let uninterrupted = cpu.step(101, CpuFbsHead::Writer).unwrap();
            assert_eq!(restored, uninterrupted);
            assert_eq!(restored.query_q12, expected.query_q12);
            assert_eq!(restored.token, expected.token);
            assert_eq!(restored.kept_cells, expected.kept_cells);
            traces.push(trace);
        }
        assert_ne!(traces[0], traces[1], "raw transition was ignored");
        assert_ne!(traces[0], traces[2], "raw gates were ignored");
    }

    #[test]
    fn retention_rate_artifacts_require_their_other_rate_tables() {
        let mut snapshot = synthetic_fbs_trunk_with(
            31,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.retain_rates = true;
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let model = IntegerStackModel::parse(&bytes).unwrap();
        assert!(CpuFbsForward::from_model(&model).is_err());
    }

    #[test]
    fn context_exhaustion_matches_serving() {
        let (model, mut cpu) = fixture(29, 1, true);
        let mut session = model.session();
        for step in 0..16 {
            let token = (step % 512) as u32;
            session.step_native(token).unwrap();
            cpu.step(token, CpuFbsHead::Nearest).unwrap();
        }
        assert!(session.step_native(0).is_err());
        assert!(cpu.step(0, CpuFbsHead::Nearest).is_err());
    }
}
