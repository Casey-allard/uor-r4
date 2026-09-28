//! Offline differentiable references for UOR-R4 research.
//!
//! The first reference reproduces the retained #1017 dense Llama checkpoint.
//! It is an explicitly floating-point transformer comparator, never a native
//! serving path. Its purpose is to establish a real language-loss gradient
//! through attention before building and measuring a discrete bridge.

#![forbid(unsafe_code)]

pub mod baseline_counts;
pub mod baseline_protocol;
pub mod joint_admission;
pub mod joint_bounded_campaign;
pub mod joint_campaign;
pub mod joint_comparison;
pub mod joint_evaluation;
pub mod joint_integer;
pub mod joint_integer_evaluation;
pub mod joint_integer_math;
pub mod joint_integer_tables;
pub mod joint_model;
pub mod joint_optimizer;
pub mod joint_parallel;
pub mod joint_quantization;
pub mod joint_rounding;
pub mod joint_rounding_campaign;
pub mod ngram;
pub mod reference_campaign;
pub mod reference_eval;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor, Var};
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_model_source::{BehaviorSource, ExactBackendReport, HuggingFaceLlamaOracle};

pub const CANDLE_VERSION: &str = "0.9.2";
pub const REFERENCE_WEIGHTS_SHA256: &str =
    "cf988a6b5f722b614740b0843807a41feff8276571c846bbee85d410a0030d84";
pub const MAX_INTEGRITY_TOKENS: usize = 64;
pub const PARITY_MAX_ABSOLUTE_DELTA: f64 = 0.005;
pub const FINITE_DIFFERENCE_EPSILON: f32 = 0.01;
pub const FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE: f64 = 0.0002;
pub const FINITE_DIFFERENCE_RELATIVE_TOLERANCE: f64 = 0.05;

#[derive(Debug)]
pub enum TrainingError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Tensor(candle_core::Error),
    Safetensors(safetensors::SafeTensorError),
    Optimizer(joint_optimizer::OptimizerError),
    Integer(uor_r4_integer::IntegerError),
    Invalid(String),
    Reference(String),
}
impl fmt::Display for TrainingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O: {error}"),
            Self::Json(error) => write!(f, "JSON: {error}"),
            Self::Tensor(error) => write!(f, "autodiff tensor: {error}"),
            Self::Safetensors(error) => write!(f, "safetensors: {error}"),
            Self::Optimizer(error) => write!(f, "optimizer: {error}"),
            Self::Integer(error) => write!(f, "{error}"),
            Self::Invalid(error) => write!(f, "invalid reference request: {error}"),
            Self::Reference(error) => write!(f, "existing Rust reference: {error}"),
        }
    }
}
impl std::error::Error for TrainingError {}
macro_rules! convert {
    ($source:ty, $variant:ident) => {
        impl From<$source> for TrainingError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}
convert!(std::io::Error, Io);
convert!(serde_json::Error, Json);
convert!(candle_core::Error, Tensor);
convert!(safetensors::SafeTensorError, Safetensors);
convert!(joint_optimizer::OptimizerError, Optimizer);
convert!(uor_r4_integer::IntegerError, Integer);
pub type Result<T> = std::result::Result<T, TrainingError>;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut digest = Sha256::new();
    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        digest.update(&buffer[..bytes]);
    }
    Ok(hex::encode(digest.finalize()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub max_position_embeddings: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub rope_interleaved: bool,
    pub tie_word_embeddings: bool,
    pub attention_bias: bool,
    pub mlp_bias: bool,
    pub hidden_act: String,
}
impl ReferenceConfig {
    fn validate(&self) -> Result<()> {
        if self.vocab_size != 4096
            || self.hidden_size != 288
            || self.intermediate_size != 768
            || self.num_hidden_layers != 6
            || self.num_attention_heads != 6
            || self.num_key_value_heads != 6
            || self.max_position_embeddings != 256
            || self.rms_norm_eps != 1e-5
            || self.rope_theta != 10000.0
            || self.rope_interleaved
            || !self.tie_word_embeddings
            || self.attention_bias
            || self.mlp_bias
            || self.hidden_act != "silu"
        {
            return Err(invalid(
                "configuration differs from the retained #1017 architecture",
            ));
        }
        Ok(())
    }

    fn expected_shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let width = self.hidden_size;
        let inner = self.intermediate_size;
        let mut shapes = BTreeMap::from([
            (
                "model.embed_tokens.weight".into(),
                vec![self.vocab_size, width],
            ),
            ("model.norm.weight".into(), vec![width]),
        ]);
        for layer in 0..self.num_hidden_layers {
            for name in ["input_layernorm", "post_attention_layernorm"] {
                shapes.insert(format!("model.layers.{layer}.{name}.weight"), vec![width]);
            }
            for name in ["q_proj", "k_proj", "v_proj", "o_proj"] {
                shapes.insert(
                    format!("model.layers.{layer}.self_attn.{name}.weight"),
                    vec![width, width],
                );
            }
            for name in ["gate_proj", "up_proj"] {
                shapes.insert(
                    format!("model.layers.{layer}.mlp.{name}.weight"),
                    vec![inner, width],
                );
            }
            shapes.insert(
                format!("model.layers.{layer}.mlp.down_proj.weight"),
                vec![width, inner],
            );
        }
        shapes
    }
}

/// Reusable differentiable dense comparator; no tokenizer or serving fallback.
pub struct ReferenceModel {
    pub config: ReferenceConfig,
    variables: BTreeMap<String, Var>,
    device: Device,
}
impl ReferenceModel {
    /// Safe in-memory safetensors deserialization. No pickle, mmap, network or unsafe code.
    pub fn load_pinned(snapshot: &Path, device: &Device) -> Result<Self> {
        let weights_path = snapshot.join("model.safetensors");
        let bytes = fs::read(&weights_path)?;
        if hex::encode(Sha256::digest(&bytes)) != REFERENCE_WEIGHTS_SHA256 {
            return Err(invalid("#1017 reference weights SHA-256 mismatch"));
        }
        let config: ReferenceConfig =
            serde_json::from_slice(&fs::read(snapshot.join("config.json"))?)?;
        config.validate()?;
        let tensors = SafeTensors::deserialize(&bytes)?;
        let shapes = config.expected_shapes();
        let names: BTreeSet<_> = tensors.names().into_iter().collect();
        let expected: BTreeSet<_> = shapes.keys().map(String::as_str).collect();
        if names != expected {
            return Err(invalid(
                "reference tensor names differ from all 56 expected tensors",
            ));
        }
        let mut variables = BTreeMap::new();
        for (name, shape) in shapes {
            let tensor = tensors.tensor(&name)?;
            if tensor.dtype() != SafeDtype::F32 || tensor.shape() != shape {
                return Err(invalid(format!(
                    "reference shape/dtype mismatch for {name}"
                )));
            }
            let values = tensor
                .data()
                .chunks_exact(4)
                .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .collect::<Vec<_>>();
            if values.iter().any(|value| !value.is_finite()) {
                return Err(invalid(format!(
                    "nonfinite reference coefficient in {name}"
                )));
            }
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        Ok(Self {
            config,
            variables,
            device: device.clone(),
        })
    }

    fn weight(&self, name: &str) -> Result<&Tensor> {
        self.variables
            .get(name)
            .map(Var::as_tensor)
            .ok_or_else(|| invalid(format!("missing reference variable {name}")))
    }

    fn parameter(&self, name: &str, detached: bool) -> Result<Tensor> {
        let tensor = self.weight(name)?;
        Ok(if detached {
            tensor.detach()
        } else {
            tensor.clone()
        })
    }

    fn linear(&self, input: &Tensor, name: &str, detached: bool) -> Result<Tensor> {
        Ok(input.matmul(&self.parameter(name, detached)?.t()?)?)
    }

    fn rms_norm(&self, input: &Tensor, name: &str, detached: bool) -> Result<Tensor> {
        // Deliberately primitive composition: fused inference-only RMSNorm can drop backward.
        let denominator = input
            .sqr()?
            .mean_keepdim(1)?
            .affine(1.0, self.config.rms_norm_eps)?
            .sqrt()?;
        Ok(input
            .broadcast_div(&denominator)?
            .broadcast_mul(&self.parameter(name, detached)?)?)
    }

    fn rotate_half(&self, input: &Tensor, cosine: &Tensor, sine: &Tensor) -> Result<Tensor> {
        let half = self.config.hidden_size / self.config.num_attention_heads / 2;
        let first = input.narrow(2, 0, half)?;
        let second = input.narrow(2, half, half)?;
        let a = first
            .broadcast_mul(cosine)?
            .sub(&second.broadcast_mul(sine)?)?;
        let b = second
            .broadcast_mul(cosine)?
            .add(&first.broadcast_mul(sine)?)?;
        Ok(Tensor::cat(&[&a, &b], 2)?)
    }

    /// One causal batch of token IDs, producing [time, vocabulary] floating logits.
    /// Standard causal softmax is explicit; there is no discrete-selection surrogate yet.
    pub fn forward(&self, ids: &[u32]) -> Result<Tensor> {
        Ok(self.forward_batch(ids, 1, ids.len(), false)?.squeeze(0)?)
    }

    /// Population inference uses the same arithmetic with parameters detached
    /// before any operation, so it does not retain an unused backward graph.
    pub fn forward_eval_batch(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.forward_batch(ids, batch, time, true)
    }

    pub fn forward_eval(&self, ids: &[u32]) -> Result<Tensor> {
        Ok(self.forward_eval_batch(ids, 1, ids.len())?.squeeze(0)?)
    }

    fn forward_batch(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        detached: bool,
    ) -> Result<Tensor> {
        if !(1..=16).contains(&batch)
            || time == 0
            || time > self.config.max_position_embeddings
            || ids.len() != batch * time
            || ids
                .iter()
                .any(|&token| token as usize >= self.config.vocab_size)
        {
            return Err(invalid("forward token/context bounds"));
        }
        let heads = self.config.num_attention_heads;
        let width = self.config.hidden_size;
        let head_dim = width / heads;
        let half = head_dim / 2;
        let mut cosine = Vec::with_capacity(time * half);
        let mut sine = Vec::with_capacity(time * half);
        for position in 0..time {
            for pair in 0..half {
                let inverse = 1.0f32
                    / (self.config.rope_theta as f32).powf((2 * pair) as f32 / head_dim as f32);
                let angle = position as f32 * inverse;
                cosine.push(angle.cos());
                sine.push(angle.sin());
            }
        }
        let cosine = Tensor::from_vec(cosine, (1, time, half), &self.device)?;
        let sine = Tensor::from_vec(sine, (1, time, half), &self.device)?;
        let causal: Vec<u8> = (0..time)
            .flat_map(|row| (0..time).map(move |column| u8::from(column > row)))
            .collect();
        let mask = Tensor::from_vec(causal, (1, time, time), &self.device)?.broadcast_as((
            batch * heads,
            time,
            time,
        ))?;
        let excluded = Tensor::full(f32::NEG_INFINITY, (batch * heads, time, time), &self.device)?;
        let ids = Tensor::new(ids, &self.device)?;
        let mut state = self
            .parameter("model.embed_tokens.weight", detached)?
            .index_select(&ids, 0)?;
        for layer in 0..self.config.num_hidden_layers {
            let prefix = format!("model.layers.{layer}");
            let normalized = self.rms_norm(
                &state,
                &format!("{prefix}.input_layernorm.weight"),
                detached,
            )?;
            let project = |name| -> Result<Tensor> {
                Ok(self
                    .linear(
                        &normalized,
                        &format!("{prefix}.self_attn.{name}.weight"),
                        detached,
                    )?
                    .reshape((batch, time, heads, head_dim))?
                    .transpose(1, 2)?
                    .contiguous()?
                    .reshape((batch * heads, time, head_dim))?)
            };
            let query = self.rotate_half(&project("q_proj")?, &cosine, &sine)?;
            let key = self.rotate_half(&project("k_proj")?, &cosine, &sine)?;
            let value = project("v_proj")?;
            let scores = query
                .matmul(&key.transpose(1, 2)?.contiguous()?)?
                .affine(1.0 / (head_dim as f64).sqrt(), 0.0)?;
            let masked = mask.where_cond(&excluded, &scores)?;
            // softmax_last_dim is an inference-only custom op in Candle; use this composition.
            let probability = candle_nn::ops::softmax(&masked, 2)?;
            let attended = probability
                .matmul(&value)?
                .reshape((batch, heads, time, head_dim))?
                .transpose(1, 2)?
                .contiguous()?
                .reshape((batch * time, width))?;
            state = state.add(&self.linear(
                &attended,
                &format!("{prefix}.self_attn.o_proj.weight"),
                detached,
            )?)?;
            let normalized = self.rms_norm(
                &state,
                &format!("{prefix}.post_attention_layernorm.weight"),
                detached,
            )?;
            let gate = self
                .linear(
                    &normalized,
                    &format!("{prefix}.mlp.gate_proj.weight"),
                    detached,
                )?
                .silu()?;
            let up = self.linear(
                &normalized,
                &format!("{prefix}.mlp.up_proj.weight"),
                detached,
            )?;
            state = state.add(&self.linear(
                &gate.mul(&up)?,
                &format!("{prefix}.mlp.down_proj.weight"),
                detached,
            )?)?;
        }
        let hidden = self.rms_norm(&state, "model.norm.weight", detached)?;
        Ok(self
            .linear(&hidden, "model.embed_tokens.weight", detached)?
            .reshape((batch, time, self.config.vocab_size))?)
    }

    pub fn next_token_loss(&self, sequence: &[u32]) -> Result<Tensor> {
        if sequence.len() < 2 {
            return Err(invalid("loss needs at least one input and target"));
        }
        let logits = self.forward(&sequence[..sequence.len() - 1])?;
        let targets = Tensor::new(&sequence[1..], &self.device)?.unsqueeze(1)?;
        let log_probability = candle_nn::ops::log_softmax(&logits, 1)?;
        Ok(log_probability.gather(&targets, 1)?.mean_all()?.neg()?)
    }
}

#[derive(Clone, Debug)]
pub struct IntegrityRequest {
    pub snapshot: PathBuf,
    pub tokens: PathBuf,
    pub device: String,
    pub window: usize,
}
impl IntegrityRequest {
    pub fn validate(&self) -> Result<()> {
        if !(8..=MAX_INTEGRITY_TOKENS).contains(&self.window) {
            return Err(invalid("integrity window must be 8..=64 tokens"));
        }
        if self.device != "cpu" && self.device != "metal" {
            return Err(invalid(
                "device must be cpu or metal; there is no silent fallback",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct InputIdentity {
    pub path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
}
fn identity(path: &Path) -> Result<InputIdentity> {
    Ok(InputIdentity {
        path: path.to_path_buf(),
        bytes: fs::metadata(path)?.len(),
        sha256: sha256_file(path)?,
    })
}

#[derive(Debug, Serialize)]
pub struct GradientSummary {
    pub tensor: String,
    pub elements: usize,
    pub present: bool,
    pub finite: bool,
    pub nonzero: usize,
    pub l2: f64,
    pub max_abs: f64,
}

#[derive(Debug, Serialize)]
pub struct FiniteDifference {
    pub tensor: String,
    pub flat_index: usize,
    pub epsilon: f32,
    pub analytic: f64,
    pub numerical: f64,
    pub plus_loss: f64,
    pub minus_loss: f64,
    pub absolute_error: f64,
    pub tolerance: f64,
    pub passes: bool,
}

#[derive(Debug, Serialize)]
pub struct Parity {
    pub positions: usize,
    pub compared_logits: usize,
    pub max_absolute_delta: f64,
    pub maximum_allowed_absolute_delta: f64,
    pub top1_matches: usize,
    pub per_position: Vec<PositionParity>,
    pub final_reference_logits: Vec<f32>,
    pub final_autodiff_logits: Vec<f32>,
    pub passes: bool,
}

#[derive(Debug, Serialize)]
pub struct PositionParity {
    pub position: usize,
    pub reference_top1: usize,
    pub autodiff_top1: usize,
    pub max_absolute_delta: f64,
}

#[derive(Debug, Serialize)]
pub struct IntegrityReport {
    pub schema: &'static str,
    pub scope: &'static str,
    pub source_commit: String,
    pub executable_sha256: String,
    pub candle_version: &'static str,
    pub requested_device: String,
    pub actual_device: String,
    pub reference_backend: ExactBackendReport,
    pub inputs: Vec<InputIdentity>,
    pub config: ReferenceConfig,
    pub tokens: Vec<u32>,
    pub parity: Parity,
    pub next_token_loss_nats: f64,
    pub gradients: Vec<GradientSummary>,
    pub all_qkv_gradients_present_finite_nonzero: bool,
    pub finite_differences: Vec<FiniteDifference>,
    pub finite_differences_pass: bool,
    pub restored_loss_delta: f64,
    pub optimizer_steps: u64,
    pub learned_hard_selection: bool,
    pub elapsed_ms: u128,
    pub status: &'static str,
}

fn top1(values: &[f32]) -> Result<usize> {
    let mut best = None;
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(invalid("nonfinite evaluated logit"));
        }
        if best.is_none_or(|previous| value > values[previous]) {
            best = Some(index);
        }
    }
    best.ok_or_else(|| invalid("empty logits"))
}

/// Bounded integrity evaluation. Temporary finite-difference perturbations are restored.
/// No optimizer update, saved candidate, corpus run or serving change occurs here.
pub fn run_integrity(request: &IntegrityRequest) -> Result<IntegrityReport> {
    request.validate()?;
    let started = Instant::now();
    let device = match request.device.as_str() {
        "cpu" => Device::Cpu,
        "metal" => {
            #[cfg(feature = "metal")]
            {
                Device::new_metal(0)?
            }
            #[cfg(not(feature = "metal"))]
            {
                return Err(invalid(
                    "metal device requested but metal feature was not compiled",
                ));
            }
        }
        _ => return Err(invalid("unsupported device")),
    };
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .unwrap_or("UNBOUND")
        .to_owned();
    if source_commit == "UNBOUND" {
        return Err(invalid(
            "build with UOR_BUILD_SOURCE_COMMIT for source-bound evidence",
        ));
    }
    let inputs = vec![
        identity(&request.snapshot.join("model.safetensors"))?,
        identity(&request.snapshot.join("config.json"))?,
        identity(&request.snapshot.join("tokenizer.json"))?,
        identity(&request.tokens)?,
    ];
    if inputs[3].bytes % 2 != 0 || inputs[3].bytes < ((request.window + 1) * 2) as u64 {
        return Err(invalid("retained u16 token store length"));
    }
    let mut token_bytes = vec![0u8; (request.window + 1) * 2];
    fs::File::open(&request.tokens)?.read_exact(&mut token_bytes)?;
    let tokens = token_bytes
        .chunks_exact(2)
        .map(|value| u32::from(u16::from_le_bytes([value[0], value[1]])))
        .collect::<Vec<_>>();
    let model = ReferenceModel::load_pinned(&request.snapshot, &device)?;
    let logits = model.forward(&tokens[..request.window])?.to_vec2::<f32>()?;
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(&request.snapshot, request.window)
            .map_err(|error| TrainingError::Reference(error.to_string()))?;
    let reference_backend = oracle.exact_backend_report();
    let mut reference = vec![0f32; model.config.vocab_size];
    let mut maximum = 0.0f64;
    let mut matches = 0usize;
    let mut per_position = Vec::with_capacity(request.window);
    for (position, &token) in tokens[..request.window].iter().enumerate() {
        oracle.step(token as usize, position, &mut reference);
        let mut position_maximum = 0.0f64;
        for (&a, &b) in logits[position].iter().zip(&reference) {
            position_maximum = position_maximum.max((f64::from(a) - f64::from(b)).abs());
        }
        maximum = maximum.max(position_maximum);
        let autodiff_top1 = top1(&logits[position])?;
        let reference_top1 = top1(&reference)?;
        matches += usize::from(autodiff_top1 == reference_top1);
        per_position.push(PositionParity {
            position,
            reference_top1,
            autodiff_top1,
            max_absolute_delta: position_maximum,
        });
    }
    let parity = Parity {
        positions: request.window,
        compared_logits: request.window * model.config.vocab_size,
        max_absolute_delta: maximum,
        maximum_allowed_absolute_delta: PARITY_MAX_ABSOLUTE_DELTA,
        top1_matches: matches,
        per_position,
        final_reference_logits: reference,
        final_autodiff_logits: logits
            .last()
            .cloned()
            .ok_or_else(|| invalid("missing final logits"))?,
        passes: maximum <= PARITY_MAX_ABSOLUTE_DELTA && matches == request.window,
    };
    drop(oracle);
    let loss = model.next_token_loss(&tokens)?;
    let loss_value = f64::from(loss.to_scalar::<f32>()?);
    if !loss_value.is_finite() {
        return Err(invalid("nonfinite next-token loss"));
    }
    let gradients = loss.backward()?;
    let mut summaries = Vec::new();
    let mut all_qkv = true;
    for (name, variable) in &model.variables {
        let (present, values) = match gradients.get(variable.as_tensor()) {
            Some(gradient) => (true, gradient.flatten_all()?.to_vec1::<f32>()?),
            None => (false, Vec::new()),
        };
        let finite = present && values.iter().all(|value| value.is_finite());
        let nonzero = values.iter().filter(|&&value| value != 0.0).count();
        let l2 = values
            .iter()
            .map(|&value| f64::from(value).powi(2))
            .sum::<f64>()
            .sqrt();
        let max_abs = values
            .iter()
            .map(|&value| f64::from(value).abs())
            .fold(0.0f64, f64::max);
        if ["q_proj", "k_proj", "v_proj"]
            .iter()
            .any(|part| name.contains(part))
        {
            all_qkv &= present && finite && nonzero > 0;
        }
        summaries.push(GradientSummary {
            tensor: name.clone(),
            elements: variable.elem_count(),
            present,
            finite,
            nonzero,
            l2,
            max_abs,
        });
    }
    let mut finite_differences = Vec::new();
    for projection in ["q_proj", "k_proj", "v_proj"] {
        let name = format!("model.layers.0.self_attn.{projection}.weight");
        let variable = model
            .variables
            .get(&name)
            .ok_or_else(|| invalid("missing checked variable"))?;
        let gradient = gradients
            .get(variable.as_tensor())
            .ok_or_else(|| invalid(format!("no gradient for {name}")))?;
        let values = gradient.flatten_all()?.to_vec1::<f32>()?;
        let index = values
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(index, _)| index)
            .ok_or_else(|| invalid("empty checked gradient"))?;
        let original = variable.flatten_all()?.to_vec1::<f32>()?;
        let original_tensor = Tensor::from_vec(original.clone(), variable.shape(), &device)?;
        let perturbed = (|| -> Result<FiniteDifference> {
            let mut plus = original.clone();
            let mut minus = original.clone();
            plus[index] += FINITE_DIFFERENCE_EPSILON;
            minus[index] -= FINITE_DIFFERENCE_EPSILON;
            let denominator = f64::from(plus[index]) - f64::from(minus[index]);
            variable.set(&Tensor::from_vec(plus, variable.shape(), &device)?)?;
            let plus_loss = f64::from(model.next_token_loss(&tokens)?.to_scalar::<f32>()?);
            variable.set(&Tensor::from_vec(minus, variable.shape(), &device)?)?;
            let minus_loss = f64::from(model.next_token_loss(&tokens)?.to_scalar::<f32>()?);
            let analytic = f64::from(values[index]);
            let numerical = (plus_loss - minus_loss) / denominator;
            let absolute_error = (analytic - numerical).abs();
            let tolerance = FINITE_DIFFERENCE_ABSOLUTE_TOLERANCE
                + FINITE_DIFFERENCE_RELATIVE_TOLERANCE * analytic.abs().max(numerical.abs());
            Ok(FiniteDifference {
                tensor: name,
                flat_index: index,
                epsilon: FINITE_DIFFERENCE_EPSILON,
                analytic,
                numerical,
                plus_loss,
                minus_loss,
                absolute_error,
                tolerance,
                passes: numerical.is_finite()
                    && analytic.is_finite()
                    && absolute_error <= tolerance,
            })
        })();
        variable.set(&original_tensor)?;
        finite_differences.push(perturbed?);
    }
    let restored_loss_delta =
        (f64::from(model.next_token_loss(&tokens)?.to_scalar::<f32>()?) - loss_value).abs();
    let finite_differences_pass = finite_differences.iter().all(|check| check.passes);
    let success =
        parity.passes && all_qkv && finite_differences_pass && restored_loss_delta <= 1e-6;
    Ok(IntegrityReport {
        schema: "uor-r4.training-reference-integrity/1",
        scope: "Pinned #1017 ordinary floating-point transformer reference: numerical parity and soft-attention gradient availability only; no native serving, discrete-selection bridge, corpus fit or language qualification",
        source_commit,
        executable_sha256: sha256_file(&std::env::current_exe()?)?,
        candle_version: CANDLE_VERSION,
        requested_device: request.device.clone(),
        actual_device: format!("{:?}", device.location()),
        reference_backend,
        inputs, config: model.config, tokens, parity, next_token_loss_nats: loss_value,
        gradients: summaries, all_qkv_gradients_present_finite_nonzero: all_qkv,
        finite_differences, finite_differences_pass, restored_loss_delta,
        optimizer_steps: 0, learned_hard_selection: false,
        elapsed_ms: started.elapsed().as_millis(),
        status: if success { "INTEGRITY_PASS" } else { "INTEGRITY_FAILED" },
    })
}
