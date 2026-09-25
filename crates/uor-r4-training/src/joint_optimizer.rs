//! Named, resumable AdamW for offline F32 learning, independent of serving.
//!
//! The update follows Candle 0.9.2's `candle-nn/src/optim.rs`, with decoupled
//! weight decay, named moments, explicit missing-gradient policy, and global
//! norm clipping. A skipped parameter receives no decay or moment update; its
//! own update count determines bias correction when it next has a gradient.
//!
//! All update arithmetic is detached. Gradient norms and bounds are reduced on
//! the device; one compact F32 vector crosses to the host per step. No parameter
//! is changed until all proposed updates pass validation. A device write failure
//! can still partially apply `Var::set`; that poisons this optimizer and requires
//! reloading the previous complete model/optimizer checkpoint.
//!
//! Saving writes only optimizer state. The caller must save matching model
//! weights, sampler/RNG position, recurrent state if carried, and provenance in
//! the same already claimed checkpoint leaf. Files are created exclusively; an
//! incomplete save is not a resumable checkpoint and must not be overwritten.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use candle_core::backprop::GradStore;
use candle_core::{DType, Tensor, TensorId, Var};
use safetensors::tensor::TensorView;
use safetensors::{Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const BETA1: f64 = 0.9;
pub const BETA2: f64 = 0.999;
pub const EPSILON: f64 = 1e-8;
pub const CLIP_NORM: f64 = 1.0;
// Clipped coordinates have |g| <= 1. Convex moment updates preserve this bound;
// the small tolerance admits F32 reduction and roundoff at the clipping edge.
pub const MOMENT_ABS_LIMIT: f64 = 1.00001;
pub const MOMENT_FILE: &str = "optimizer.safetensors";
pub const METADATA_FILE: &str = "optimizer.json";
const FORMAT_VERSION: u32 = 1;
const MAX_METADATA_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum OptimizerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Tensor(candle_core::Error),
    Safetensors(safetensors::SafeTensorError),
    InvalidConfig(String),
    InvalidVariables(String),
    MissingGradient(String),
    InvalidNumericState(String),
    InvalidCheckpoint(String),
    Poisoned,
}

impl fmt::Display for OptimizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "optimizer I/O: {error}"),
            Self::Json(error) => write!(f, "optimizer metadata: {error}"),
            Self::Tensor(error) => write!(f, "optimizer tensor operation: {error}"),
            Self::Safetensors(error) => write!(f, "optimizer tensor format: {error}"),
            Self::InvalidConfig(reason) => write!(f, "invalid Adam configuration: {reason}"),
            Self::InvalidVariables(reason) => write!(f, "invalid named variables: {reason}"),
            Self::MissingGradient(name) => write!(f, "missing required gradient for {name}"),
            Self::InvalidNumericState(reason) => write!(f, "invalid optimizer numbers: {reason}"),
            Self::InvalidCheckpoint(reason) => write!(f, "invalid optimizer checkpoint: {reason}"),
            Self::Poisoned => write!(
                f,
                "optimizer device commit failed; reload a complete checkpoint"
            ),
        }
    }
}

impl std::error::Error for OptimizerError {}

macro_rules! convert_error {
    ($source:ty, $variant:ident) => {
        impl From<$source> for OptimizerError {
            fn from(error: $source) -> Self {
                Self::$variant(error)
            }
        }
    };
}
convert_error!(std::io::Error, Io);
convert_error!(serde_json::Error, Json);
convert_error!(candle_core::Error, Tensor);
convert_error!(safetensors::SafeTensorError, Safetensors);

pub type Result<T> = std::result::Result<T, OptimizerError>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdamConfig {
    pub learning_rate: f64,
    pub weight_decay: f64,
    pub parameter_abs_limit: f64,
    pub allowed_missing_gradients: BTreeSet<String>,
}

impl Default for AdamConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-3,
            weight_decay: 0.01,
            parameter_abs_limit: 1e6,
            allowed_missing_gradients: BTreeSet::new(),
        }
    }
}

impl AdamConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.learning_rate.is_finite()
            || self.learning_rate <= 0.0
            || !(self.learning_rate as f32).is_finite()
            || self.learning_rate as f32 == 0.0
        {
            return Err(OptimizerError::InvalidConfig(
                "learning rate must be positive finite F32-representable".into(),
            ));
        }
        if !self.weight_decay.is_finite()
            || self.weight_decay < 0.0
            || !((self.learning_rate * self.weight_decay) as f32).is_finite()
            || self.learning_rate * self.weight_decay > 1.0
        {
            return Err(OptimizerError::InvalidConfig(
                "decay must be nonnegative and lr * decay must lie in [0,1]".into(),
            ));
        }
        if !self.parameter_abs_limit.is_finite()
            || self.parameter_abs_limit <= 0.0
            || !(self.parameter_abs_limit as f32).is_finite()
            || self.parameter_abs_limit as f32 == 0.0
        {
            return Err(OptimizerError::InvalidConfig(
                "parameter bound must be positive finite F32-representable".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepReport {
    pub step: u64,
    pub global_grad_norm: f64,
    pub applied_clip_scale: f64,
    pub updated_variables: usize,
    pub missing_gradients: Vec<String>,
    pub gradient_l2_by_name: BTreeMap<String, f64>,
    pub parameter_max_abs: f64,
    pub first_moment_max_abs: f64,
    pub second_moment_max_abs: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerCheckpoint {
    pub step: u64,
    pub variables: usize,
    pub moments_bytes: u64,
    pub moments_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VariableMetadata {
    name: String,
    shape: Vec<usize>,
    updates: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointMetadata {
    format_version: u32,
    candle_version: String,
    beta1: f64,
    beta2: f64,
    epsilon: f64,
    clip_norm: f64,
    moment_abs_limit: f64,
    config: AdamConfig,
    step: u64,
    variables: Vec<VariableMetadata>,
    moments_bytes: u64,
    moments_sha256: String,
}

#[derive(Debug)]
struct MomentState {
    variable_id: TensorId,
    first: Tensor,
    second: Tensor,
    updates: u64,
}

struct ProposedUpdate {
    parameter: Tensor,
    first: Tensor,
    second: Tensor,
    updates: u64,
}

#[derive(Debug)]
pub struct NamedAdamW {
    config: AdamConfig,
    states: BTreeMap<String, MomentState>,
    step: u64,
    poisoned: bool,
}

impl NamedAdamW {
    pub fn new(variables: &BTreeMap<String, Var>, config: AdamConfig) -> Result<Self> {
        validate_variables(variables, &config)?;
        validate_initial_parameters(variables, config.parameter_abs_limit)?;
        let mut states = BTreeMap::new();
        for (name, variable) in variables {
            states.insert(
                name.clone(),
                MomentState {
                    variable_id: variable.id(),
                    first: Tensor::zeros(variable.shape(), DType::F32, variable.device())?,
                    second: Tensor::zeros(variable.shape(), DType::F32, variable.device())?,
                    updates: 0,
                },
            );
        }
        Ok(Self {
            config,
            states,
            step: 0,
            poisoned: false,
        })
    }

    pub fn config(&self) -> &AdamConfig {
        &self.config
    }

    pub fn step_count(&self) -> u64 {
        self.step
    }

    pub fn step(
        &mut self,
        variables: &BTreeMap<String, Var>,
        gradients: &GradStore,
    ) -> Result<StepReport> {
        if self.poisoned {
            return Err(OptimizerError::Poisoned);
        }
        if variables.len() != self.states.len() {
            return Err(OptimizerError::InvalidVariables(
                "variable inventory changed".into(),
            ));
        }
        let next_step = self
            .step
            .checked_add(1)
            .ok_or_else(|| OptimizerError::InvalidNumericState("step counter exhausted".into()))?;
        let mut present = Vec::new();
        let mut missing = Vec::new();
        let mut gradient_squares = Vec::new();
        let mut bounds = Vec::new();
        let mut parameter_maxima = Vec::new();
        let mut first_maxima = Vec::new();
        let mut second_maxima = Vec::new();
        for (name, state) in &self.states {
            let variable = variables.get(name).ok_or_else(|| {
                OptimizerError::InvalidVariables(format!("missing variable {name}"))
            })?;
            if variable.id() != state.variable_id
                || variable.dtype() != DType::F32
                || variable.dims() != state.first.dims()
                || !variable.device().same_device(state.first.device())
            {
                return Err(OptimizerError::InvalidVariables(format!(
                    "binding, shape, dtype or device changed for {name}"
                )));
            }
            bounds.push(bounded(
                &variable.as_detached_tensor(),
                self.config.parameter_abs_limit,
            )?);
            let Some(gradient) = gradients.get(variable) else {
                if !self.config.allowed_missing_gradients.contains(name) {
                    return Err(OptimizerError::MissingGradient(name.clone()));
                }
                missing.push(name.clone());
                parameter_maxima.push(variable.as_detached_tensor().abs()?.max_all()?);
                first_maxima.push(state.first.abs()?.max_all()?);
                second_maxima.push(state.second.max_all()?);
                continue;
            };
            if gradient.dtype() != DType::F32
                || gradient.dims() != variable.dims()
                || !gradient.device().same_device(variable.device())
            {
                return Err(OptimizerError::InvalidVariables(format!(
                    "gradient shape, dtype or device differs for {name}"
                )));
            }
            let gradient = gradient.detach();
            gradient_squares.push(gradient.sqr()?.sum_all()?);
            present.push((name.clone(), gradient));
        }
        if present.is_empty() {
            return Err(OptimizerError::InvalidNumericState(
                "no parameter has a gradient; no step taken".into(),
            ));
        }
        // Keep clipping on device. A nonfinite sum or overflow is rejected below,
        // even if intermediate clipping would happen to produce finite values.
        let norm = Tensor::stack(&gradient_squares, 0)?.sum_all()?.sqrt()?;
        let clip = norm.clamp(CLIP_NORM, f32::MAX as f64)?.recip()?;
        let mut proposed = BTreeMap::new();
        for (name, gradient) in &present {
            let state = self.states.get(name).ok_or_else(|| {
                OptimizerError::InvalidVariables(format!("missing moment state for {name}"))
            })?;
            let variable = variables.get(name).ok_or_else(|| {
                OptimizerError::InvalidVariables(format!("missing variable {name}"))
            })?;
            let updates = state.updates.checked_add(1).ok_or_else(|| {
                OptimizerError::InvalidNumericState(format!("update count exhausted for {name}"))
            })?;
            let gradient = gradient.broadcast_mul(&clip)?;
            let first = ((&state.first * BETA1)? + (&gradient * (1.0 - BETA1))?)?.detach();
            let second = ((&state.second * BETA2)? + (gradient.sqr()? * (1.0 - BETA2))?)?.detach();
            let first_hat = (&first * (1.0 / (1.0 - BETA1.powf(updates as f64))))?;
            let second_hat = (&second * (1.0 / (1.0 - BETA2.powf(updates as f64))))?;
            let direction = (first_hat / (second_hat.sqrt()? + EPSILON)?)?;
            let decayed = (variable.as_detached_tensor()
                * (1.0 - self.config.learning_rate * self.config.weight_decay))?;
            let parameter = (decayed - (direction * self.config.learning_rate)?)?.detach();
            bounds.push(bounded(&parameter, self.config.parameter_abs_limit)?);
            bounds.push(bounded(&first, MOMENT_ABS_LIMIT)?);
            bounds.push(bounded(&second, MOMENT_ABS_LIMIT)?);
            bounds.push(second.ge(0f64)?.to_dtype(DType::F32)?.min_all()?);
            parameter_maxima.push(parameter.abs()?.max_all()?);
            first_maxima.push(first.abs()?.max_all()?);
            second_maxima.push(second.max_all()?);
            proposed.insert(
                name.clone(),
                ProposedUpdate {
                    parameter,
                    first,
                    second,
                    updates,
                },
            );
        }
        let mut diagnostics = vec![
            norm,
            clip,
            Tensor::stack(&parameter_maxima, 0)?.max_all()?,
            Tensor::stack(&first_maxima, 0)?.max_all()?,
            Tensor::stack(&second_maxima, 0)?.max_all()?,
            Tensor::stack(&bounds, 0)?.min_all()?,
        ];
        for square in &gradient_squares {
            diagnostics.push(square.sqrt()?);
        }
        // The sole host transfer in a step: six aggregate scalars plus one norm
        // per present named gradient. Full parameter/moment vectors stay local.
        let diagnostics = Tensor::stack(&diagnostics, 0)?.to_vec1::<f32>()?;
        if diagnostics.iter().any(|value| !value.is_finite())
            || diagnostics[5] != 1.0
            || diagnostics[2] as f64 > self.config.parameter_abs_limit
            || diagnostics[3] as f64 > MOMENT_ABS_LIMIT
            || diagnostics[4] as f64 > MOMENT_ABS_LIMIT
        {
            return Err(OptimizerError::InvalidNumericState(
                "gradient norm is nonfinite/overflowed, or a parameter/moment violates its finite bound; no step taken".into()));
        }
        let gradient_l2_by_name = present
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), diagnostics[6 + i] as f64))
            .collect();
        // Validation errors above cannot mutate weights or moments. A backend
        // copy failure below is explicitly unrecoverable without checkpoint load.
        self.poisoned = true;
        for (name, update) in &proposed {
            variables
                .get(name)
                .ok_or_else(|| {
                    OptimizerError::InvalidVariables(format!("missing variable {name}"))
                })?
                .set(&update.parameter)?;
        }
        for (name, update) in proposed {
            let state = self.states.get_mut(&name).ok_or_else(|| {
                OptimizerError::InvalidVariables(format!("missing moment state for {name}"))
            })?;
            state.first = update.first;
            state.second = update.second;
            state.updates = update.updates;
        }
        self.step = next_step;
        self.poisoned = false;
        Ok(StepReport {
            step: self.step,
            global_grad_norm: diagnostics[0] as f64,
            applied_clip_scale: diagnostics[1] as f64,
            updated_variables: present.len(),
            missing_gradients: missing,
            gradient_l2_by_name,
            parameter_max_abs: diagnostics[2] as f64,
            first_moment_max_abs: diagnostics[3] as f64,
            second_moment_max_abs: diagnostics[4] as f64,
        })
    }

    /// The destination directory must already exist. Neither file is overwritten.
    pub fn save(&self, directory: impl AsRef<Path>) -> Result<OptimizerCheckpoint> {
        if self.poisoned {
            return Err(OptimizerError::Poisoned);
        }
        let directory = directory.as_ref();
        if !directory.is_dir() {
            return Err(OptimizerError::InvalidCheckpoint(
                "checkpoint leaf must already exist".into(),
            ));
        }
        let mut encoded = BTreeMap::new();
        let mut variables = Vec::with_capacity(self.states.len());
        for (name, state) in &self.states {
            variables.push(VariableMetadata {
                name: name.clone(),
                shape: state.first.dims().to_vec(),
                updates: state.updates,
            });
            encoded.insert(
                format!("m/{name}"),
                (
                    state.first.dims().to_vec(),
                    encode_moment(&state.first, false, state.updates)?,
                ),
            );
            encoded.insert(
                format!("v/{name}"),
                (
                    state.second.dims().to_vec(),
                    encode_moment(&state.second, true, state.updates)?,
                ),
            );
        }
        // Avoid Candle's Tensor-as-View implementation, whose device extraction
        // calls unwrap. Explicit checked extraction gives ordinary Result errors.
        let views = encoded
            .iter()
            .map(|(name, (shape, data))| {
                Ok((
                    name.as_str(),
                    TensorView::new(SafeDtype::F32, shape.clone(), data)?,
                ))
            })
            .collect::<std::result::Result<Vec<_>, safetensors::SafeTensorError>>()?;
        let bytes = safetensors::serialize(views, None)?;
        let receipt = OptimizerCheckpoint {
            step: self.step,
            variables: self.states.len(),
            moments_bytes: bytes.len() as u64,
            moments_sha256: hex::encode(Sha256::digest(&bytes)),
        };
        let metadata = CheckpointMetadata {
            format_version: FORMAT_VERSION,
            candle_version: "0.9.2".into(),
            beta1: BETA1,
            beta2: BETA2,
            epsilon: EPSILON,
            clip_norm: CLIP_NORM,
            moment_abs_limit: MOMENT_ABS_LIMIT,
            config: self.config.clone(),
            step: self.step,
            variables,
            moments_bytes: receipt.moments_bytes,
            moments_sha256: receipt.moments_sha256.clone(),
        };
        let mut json = serde_json::to_vec_pretty(&metadata)?;
        json.push(b'\n');
        if json.len() as u64 > MAX_METADATA_BYTES {
            return Err(OptimizerError::InvalidCheckpoint(
                "metadata exceeds 8 MiB".into(),
            ));
        }
        // Claim both names before writing. A failed claim/write leaves a partial
        // leaf for the caller to retain; it never changes an existing file.
        let mut tensor_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(MOMENT_FILE))?;
        let mut json_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(METADATA_FILE))?;
        tensor_file.write_all(&bytes)?;
        tensor_file.sync_all()?;
        json_file.write_all(&json)?;
        json_file.sync_all()?;
        Ok(receipt)
    }

    /// Load moments for already restored model weights. Config, names, shapes,
    /// moment hash and algorithm constants must match exactly. This cannot prove
    /// the caller restored matching weights or its sampler; the outer checkpoint
    /// manifest owns that association.
    pub fn load(
        directory: impl AsRef<Path>,
        variables: &BTreeMap<String, Var>,
        expected_config: &AdamConfig,
    ) -> Result<Self> {
        validate_variables(variables, expected_config)?;
        validate_initial_parameters(variables, expected_config.parameter_abs_limit)?;
        let directory = directory.as_ref();
        let mut json = Vec::new();
        fs::File::open(directory.join(METADATA_FILE))?
            .take(MAX_METADATA_BYTES + 1)
            .read_to_end(&mut json)?;
        if json.len() as u64 > MAX_METADATA_BYTES {
            return Err(OptimizerError::InvalidCheckpoint(
                "metadata exceeds 8 MiB".into(),
            ));
        }
        let metadata: CheckpointMetadata = serde_json::from_slice(&json)?;
        if metadata.format_version != FORMAT_VERSION
            || metadata.candle_version != "0.9.2"
            || metadata.beta1 != BETA1
            || metadata.beta2 != BETA2
            || metadata.epsilon != EPSILON
            || metadata.clip_norm != CLIP_NORM
            || metadata.moment_abs_limit != MOMENT_ABS_LIMIT
            || metadata.config != *expected_config
            || metadata.variables.len() != variables.len()
        {
            return Err(OptimizerError::InvalidCheckpoint(
                "algorithm, config or variable inventory differs".into(),
            ));
        }
        for (saved, (name, variable)) in metadata.variables.iter().zip(variables) {
            if saved.name != *name
                || saved.shape != variable.dims()
                || saved.updates > metadata.step
                || (!expected_config.allowed_missing_gradients.contains(name)
                    && saved.updates != metadata.step)
            {
                return Err(OptimizerError::InvalidCheckpoint(format!(
                    "name, shape or update count differs for {name}"
                )));
            }
        }
        let element_count: u64 = variables
            .values()
            .map(|variable| variable.elem_count() as u64)
            .sum();
        let limit = element_count
            .checked_mul(8)
            .and_then(|n| n.checked_add(MAX_METADATA_BYTES))
            .ok_or_else(|| OptimizerError::InvalidCheckpoint("moment size overflow".into()))?;
        let moment_path = directory.join(MOMENT_FILE);
        let length = fs::metadata(&moment_path)?.len();
        if length != metadata.moments_bytes || length > limit {
            return Err(OptimizerError::InvalidCheckpoint(
                "moment file size differs or exceeds shape bound".into(),
            ));
        }
        let mut bytes = Vec::new();
        fs::File::open(moment_path)?
            .take(length + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != length {
            return Err(OptimizerError::InvalidCheckpoint(
                "moment file changed length during read".into(),
            ));
        }
        if hex::encode(Sha256::digest(&bytes)) != metadata.moments_sha256 {
            return Err(OptimizerError::InvalidCheckpoint(
                "moment SHA-256 differs".into(),
            ));
        }
        let tensors = SafeTensors::deserialize(&bytes)?;
        let expected_names: BTreeSet<_> = variables
            .keys()
            .flat_map(|name| [format!("m/{name}"), format!("v/{name}")])
            .collect();
        let actual_names: BTreeSet<_> = tensors.names().into_iter().map(str::to_owned).collect();
        if actual_names != expected_names {
            return Err(OptimizerError::InvalidCheckpoint(
                "moment tensor names differ".into(),
            ));
        }
        let mut states = BTreeMap::new();
        for (saved, (name, variable)) in metadata.variables.iter().zip(variables) {
            let first = decode_moment(
                &tensors.tensor(&format!("m/{name}"))?,
                variable,
                false,
                saved.updates,
            )?;
            let second = decode_moment(
                &tensors.tensor(&format!("v/{name}"))?,
                variable,
                true,
                saved.updates,
            )?;
            states.insert(
                name.clone(),
                MomentState {
                    variable_id: variable.id(),
                    first,
                    second,
                    updates: saved.updates,
                },
            );
        }
        Ok(Self {
            config: expected_config.clone(),
            states,
            step: metadata.step,
            poisoned: false,
        })
    }
}

fn validate_variables(variables: &BTreeMap<String, Var>, config: &AdamConfig) -> Result<()> {
    config.validate()?;
    let first = variables
        .values()
        .next()
        .ok_or_else(|| OptimizerError::InvalidVariables("no trainable variables".into()))?;
    let mut ids = HashSet::new();
    for (name, variable) in variables {
        if name.is_empty()
            || name.len() > 1024
            || variable.dtype() != DType::F32
            || variable.elem_count() == 0
            || !variable.is_contiguous()
            || !variable.device().same_device(first.device())
            || !ids.insert(variable.id())
        {
            return Err(OptimizerError::InvalidVariables(format!(
                "{name}: requires a nonempty bounded name, unique contiguous nonempty F32 variable and common device")));
        }
    }
    if let Some(name) = config
        .allowed_missing_gradients
        .iter()
        .find(|name| !variables.contains_key(*name))
    {
        return Err(OptimizerError::InvalidConfig(format!(
            "missing-gradient allowance names unknown parameter {name}"
        )));
    }
    Ok(())
}

fn bounded(tensor: &Tensor, limit: f64) -> Result<Tensor> {
    // NaN <= limit is false, unlike max reductions which may ignore NaNs.
    Ok(tensor.abs()?.le(limit)?.to_dtype(DType::F32)?.min_all()?)
}

fn validate_initial_parameters(variables: &BTreeMap<String, Var>, limit: f64) -> Result<()> {
    let flags = variables
        .values()
        .map(|variable| bounded(&variable.as_detached_tensor(), limit))
        .collect::<Result<Vec<_>>>()?;
    if Tensor::stack(&flags, 0)?.min_all()?.to_scalar::<f32>()? != 1.0 {
        return Err(OptimizerError::InvalidNumericState(
            "initial parameter is nonfinite or outside its bound".into(),
        ));
    }
    Ok(())
}

fn valid_moment(value: f32, second: bool, updates: u64) -> bool {
    value.is_finite()
        && (value as f64).abs() <= MOMENT_ABS_LIMIT
        && (!second || value >= 0.0)
        && (updates != 0 || value == 0.0)
}

fn encode_moment(tensor: &Tensor, second: bool, updates: u64) -> Result<Vec<u8>> {
    let values = tensor.flatten_all()?.to_vec1::<f32>()?;
    if values
        .iter()
        .any(|&value| !valid_moment(value, second, updates))
    {
        return Err(OptimizerError::InvalidNumericState(
            "cannot save invalid moment values".into(),
        ));
    }
    Ok(values.into_iter().flat_map(f32::to_le_bytes).collect())
}

fn decode_moment(
    view: &TensorView<'_>,
    variable: &Var,
    second: bool,
    updates: u64,
) -> Result<Tensor> {
    if view.dtype() != SafeDtype::F32 || view.shape() != variable.dims() {
        return Err(OptimizerError::InvalidCheckpoint(
            "moment shape or dtype differs".into(),
        ));
    }
    let values: Vec<f32> = view
        .data()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    if values
        .iter()
        .any(|&value| !valid_moment(value, second, updates))
    {
        return Err(OptimizerError::InvalidCheckpoint(
            "moment is nonfinite, out of bounds, negative variance or nonzero before first update"
                .into(),
        ));
    }
    Ok(Tensor::from_vec(
        values,
        variable.shape(),
        variable.device(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn loss(variables: &BTreeMap<String, Var>) -> Result<Tensor> {
        let weight = variables
            .get("weight")
            .ok_or_else(|| OptimizerError::InvalidVariables("test weight".into()))?;
        let bias = variables
            .get("bias")
            .ok_or_else(|| OptimizerError::InvalidVariables("test bias".into()))?;
        Ok(((weight.sqr()?.sum_all()? * 8.0)? + bias.sqr()?.sum_all()?)?)
    }

    fn bits(tensor: &Tensor) -> Result<Vec<u32>> {
        Ok(tensor
            .flatten_all()?
            .to_vec1::<f32>()?
            .into_iter()
            .map(f32::to_bits)
            .collect())
    }

    #[test]
    fn adamw_checkpoint_continuation_matches_uninterrupted_next_step() -> Result<()> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "uor-named-adam-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| OptimizerError::InvalidCheckpoint(error.to_string()))?
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory)?;
        let mut variables = BTreeMap::new();
        variables.insert(
            "weight".into(),
            Var::from_vec(vec![0.5f32, -0.25], (2,), &Device::Cpu)?,
        );
        variables.insert(
            "bias".into(),
            Var::from_vec(vec![0.1f32], (1,), &Device::Cpu)?,
        );
        variables.insert(
            "optional".into(),
            Var::from_vec(vec![0.2f32], (1,), &Device::Cpu)?,
        );
        let config = AdamConfig {
            allowed_missing_gradients: BTreeSet::from(["optional".into()]),
            ..AdamConfig::default()
        };
        let mut strict = NamedAdamW::new(&variables, AdamConfig::default())?;
        assert!(matches!(
            strict.step(&variables, &loss(&variables)?.backward()?),
            Err(OptimizerError::MissingGradient(_))
        ));
        assert_eq!(strict.step_count(), 0);
        let mut uninterrupted = NamedAdamW::new(&variables, config.clone())?;
        let first = uninterrupted.step(&variables, &loss(&variables)?.backward()?)?;
        assert!(first.global_grad_norm > 1.0 && first.applied_clip_scale < 1.0);
        assert_eq!(first.missing_gradients, ["optional"]);
        uninterrupted.save(&directory)?;
        assert!(uninterrupted.save(&directory).is_err());
        let mut restored_variables = BTreeMap::new();
        for (name, variable) in &variables {
            restored_variables.insert(
                name.clone(),
                Var::from_tensor(&variable.as_detached_tensor().copy()?)?,
            );
        }
        let mut resumed = NamedAdamW::load(&directory, &restored_variables, &config)?;
        // The optional parameter first receives a gradient after the checkpoint.
        // Its moment clock must start at one even though global step becomes two.
        let next_loss = (loss(&variables)? + variables["optional"].sqr()?.sum_all()?)?;
        let resumed_loss =
            (loss(&restored_variables)? + restored_variables["optional"].sqr()?.sum_all()?)?;
        let next = uninterrupted.step(&variables, &next_loss.backward()?)?;
        let resumed_next = resumed.step(&restored_variables, &resumed_loss.backward()?)?;
        assert_eq!(next.step, 2);
        assert_eq!(
            next.global_grad_norm.to_bits(),
            resumed_next.global_grad_norm.to_bits()
        );
        for (name, variable) in &variables {
            assert_eq!(bits(variable)?, bits(&restored_variables[name])?);
            let left = &uninterrupted.states[name];
            let right = &resumed.states[name];
            assert_eq!(left.updates, right.updates);
            assert_eq!(bits(&left.first)?, bits(&right.first)?);
            assert_eq!(bits(&left.second)?, bits(&right.second)?);
        }
        assert_eq!(resumed.states["optional"].updates, 1);
        // A nonfinite gradient rejects before any weight, moment or step changes.
        let before = bits(&variables["weight"])?;
        let mut bad = loss(&variables)?.backward()?;
        bad.insert(
            &variables["weight"],
            Tensor::from_vec(vec![f32::NAN, 0.0], (2,), &Device::Cpu)?,
        );
        assert!(uninterrupted.step(&variables, &bad).is_err());
        assert_eq!(uninterrupted.step_count(), 2);
        assert_eq!(bits(&variables["weight"])?, before);
        let mut wrong = config.clone();
        wrong.learning_rate *= 2.0;
        assert!(NamedAdamW::load(&directory, &restored_variables, &wrong).is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
