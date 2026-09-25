//! Causal recurrent memory learner and quantized emulator for D8 rungs 1/2.
//!
//! This is F32 offline training scaffolding, with dense learned affine maps and
//! a differentiable soft read over all earlier events. It is not integer
//! serving, a transformer, a physical Hamiltonian, or an exact arithmetic
//! geometric kernel. Quaternion signs remain distinct. The Householder arm
//! changes only the lane transport formula and its declared local scale.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use candle_core::{DType, Device, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::joint_quantization::{self, QuantizationSpec};
use crate::{invalid, Result};

pub const UNIFORM_MIXTURE: f64 = 1e-8;
pub const RMS_EPSILON: f64 = 1e-5;
pub const TRANSPORT_MIN_NORM: f64 = 1e-6;
pub const QUATERNION_DELTA_SCALE: f64 = 0.1;
pub const CHECKPOINT_SCHEMA: &str = "uor-r4.joint-recurrent-checkpoint/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Quaternion,
    HouseholderPair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadMode {
    Enabled,
    NoRead,
}

/// Forward-only precision interventions on one fully quantized floating parent.
/// The first letter selects parameter precision; the second selects every
/// declared recurrent/read/output interface. `Q` uses the existing fixed grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PrecisionMode {
    FF,
    QF,
    FQ,
    QQ,
}

impl PrecisionMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_uppercase().as_str() {
            "FF" => Ok(Self::FF),
            "QF" => Ok(Self::QF),
            "FQ" => Ok(Self::FQ),
            "QQ" => Ok(Self::QQ),
            _ => Err(invalid("precision mode must be FF, QF, FQ or QQ")),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::FF => "FF",
            Self::QF => "QF",
            Self::FQ => "FQ",
            Self::QQ => "QQ",
        }
    }

    pub const fn quantizes_parameters(self) -> bool {
        matches!(self, Self::QF | Self::QQ)
    }

    pub const fn quantizes_interfaces(self) -> bool {
        matches!(self, Self::FQ | Self::QQ)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JointConfig {
    pub vocab_size: usize,
    pub width: usize,
    pub read_width: usize,
    pub context: usize,
    pub transport: Transport,
    pub seed: u64,
}
impl Default for JointConfig {
    fn default() -> Self {
        Self {
            vocab_size: 4096,
            width: 256,
            read_width: 64,
            context: 256,
            transport: Transport::Quaternion,
            seed: 0,
        }
    }
}
impl JointConfig {
    pub fn validate(&self) -> Result<()> {
        if self.vocab_size != 4096
            || !matches!(self.width, 128 | 256)
            || self.read_width != 64
            || !(2..=256).contains(&self.context)
        {
            return Err(invalid(
                "joint config requires vocabulary4096, width128/256, read_width64, context2..256",
            ));
        }
        Ok(())
    }

    fn shapes(&self) -> BTreeMap<String, Vec<usize>> {
        let d = self.width;
        let r = self.read_width;
        BTreeMap::from([
            ("embedding.weight".into(), vec![self.vocab_size, d]),
            ("recurrent.input.weight".into(), vec![3 * d, d]),
            ("recurrent.state.weight".into(), vec![3 * d, d]),
            ("recurrent.bias".into(), vec![3 * d]),
            ("read.query.weight".into(), vec![r, d]),
            ("read.query.bias".into(), vec![r]),
            ("read.key.weight".into(), vec![r, d]),
            ("read.key.bias".into(), vec![r]),
            ("read.value.weight".into(), vec![d, d]),
            ("read.value.bias".into(), vec![d]),
            ("read.age".into(), vec![self.context - 1]),
            ("read.no_read.weight".into(), vec![1, d]),
            ("read.no_read.bias".into(), vec![1]),
            ("update.weight".into(), vec![d, 2 * d]),
            ("update.bias".into(), vec![d]),
            ("update.gate.weight".into(), vec![1, 2 * d]),
            ("update.gate.bias".into(), vec![1]),
            ("copy.gate.weight".into(), vec![1, 2 * d]),
            ("copy.gate.bias".into(), vec![1]),
            ("output.norm.weight".into(), vec![d]),
            ("output.bias".into(), vec![self.vocab_size]),
        ])
    }
}

pub struct JointModel {
    pub config: JointConfig,
    variables: BTreeMap<String, Var>,
    device: Device,
    age_order: Tensor,
    quantization: Option<QuantizedTrainingState>,
    // An in-memory evaluation intervention, never a checkpoint field. None
    // preserves the original continuous/QAT/packed behavior without changes.
    precision_mode: Option<PrecisionMode>,
    // Prepared once per full-window forward (or incremental session). The STE
    // tensors retain the original Var IDs, and never survive an optimizer step.
    prepared_parameters: Option<BTreeMap<String, Tensor>>,
    hard_only: bool,
    interface_audit: Option<Arc<Mutex<BTreeMap<String, InterfaceAudit>>>>,
}

/// Scales are calibrated once from the sealed parent and retained on resume.
/// Evaluation always uses the complete quantized path, even during the ramp.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantizedTrainingState {
    pub start_step: usize,
    pub ramp_steps: usize,
    pub completed_step: usize,
    pub spec: QuantizationSpec,
}

#[derive(Clone, Copy)]
enum Interface {
    State,
    Normalized,
    Affine,
    Unit,
    Gate,
}

#[derive(Default, Serialize)]
struct InterfaceAudit {
    count: u64,
    clipped: u64,
    nonfinite: u64,
    minimum: Option<f32>,
    maximum: Option<f32>,
}

impl Interface {
    fn name(self) -> &'static str {
        match self {
            Self::State => "state_transport_provisional_read",
            Self::Normalized => "rms_normalized_output_hidden",
            Self::Affine => "fused_affine_qk_scores_logits",
            Self::Unit => "candidate_update_value_unit_transport",
            Self::Gate => "sigmoid_gates",
        }
    }
    fn format(self) -> (i16, i32, i32) {
        match self {
            Self::State => (-11, -32767, 32767),
            Self::Normalized => (-10, -32767, 32767),
            Self::Affine => (-8, -32767, 32767),
            Self::Unit => (-14, -32767, 32767),
            Self::Gate => (-15, 0, 32768),
        }
    }
}

/// All output rows are normalized distributions over the same vocabulary.
pub struct JointOutput {
    pub probabilities: Tensor,
    pub no_read_mass: Tensor,
    pub read_masses: Tensor,
    pub copy_gate: Tensor,
    pub states: Tensor,
}
impl JointOutput {
    /// Population-mean next-token NLL. Targets enter only this loss boundary.
    pub fn loss(&self, targets: &[u32]) -> Result<Tensor> {
        let (batch, time, vocab) = self.probabilities.dims3()?;
        if targets.len() != batch * time || targets.iter().any(|&id| id as usize >= vocab) {
            return Err(invalid("joint next-token target shape or vocabulary"));
        }
        let targets = Tensor::from_vec(
            targets.to_vec(),
            (batch * time, 1),
            self.probabilities.device(),
        )?;
        Ok(self
            .probabilities
            .reshape((batch * time, vocab))?
            .gather(&targets, 1)?
            .log()?
            .mean_all()?
            .neg()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub occurrence: usize,
    /// Exact observed input token per batch lane, not a target or source label.
    pub tokens: Vec<u32>,
}

struct RecurrentState {
    state: Tensor,
    // Contiguous [batch, previous events, width] histories. Each append keeps
    // its graph edge during training, without restacking every earlier write.
    keys: Option<Tensor>,
    values: Option<Tensor>,
    events: Vec<MemoryEvent>,
}

/// Detached incremental state; the context bound is enforced without eviction.
pub struct JointSession {
    config: JointConfig,
    batch: usize,
    memory: RecurrentState,
    prepared: JointModel,
}
impl JointSession {
    pub fn len(&self) -> usize {
        self.memory.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.memory.events.is_empty()
    }
    pub fn events(&self) -> &[MemoryEvent] {
        &self.memory.events
    }
}

pub struct JointStep {
    pub probabilities: Tensor,
    pub no_read_mass: Tensor,
    pub read_masses: Tensor,
    pub copy_gate: Tensor,
    pub state: Tensor,
    pub read_occurrences: Vec<usize>,
    pub written_occurrence: usize,
}

struct CoreStep {
    state: Tensor,
    no_read_mass: Tensor,
    read_masses: Tensor,
    copy_gate: Tensor,
}

impl JointModel {
    pub fn new(config: JointConfig, device: &Device) -> Result<Self> {
        config.validate()?;
        let mut rng = Initializer(config.seed);
        let mut variables = BTreeMap::new();
        // Stable lexical parameter order and one seed give identical initial
        // arrays across the two transport labels and across CPU/Metal loading.
        for (name, shape) in config.shapes() {
            let count: usize = shape.iter().product();
            let mut values = vec![0f32; count];
            if name == "embedding.weight" {
                for value in &mut values {
                    *value = rng.symmetric() * (3f32).sqrt() * 0.02;
                }
            } else if name == "output.norm.weight" {
                values.fill(1.0);
            } else if shape.len() == 2 {
                let limit = (6.0 / (shape[0] + shape[1]) as f32).sqrt();
                for value in &mut values {
                    *value = rng.symmetric() * limit;
                }
            } else if name == "recurrent.bias" {
                let lanes = config.width / 4;
                for lane in 0..lanes {
                    let fraction = lane as f64 / (lanes - 1) as f64;
                    let tau = 4.0 * 16f64.powf(fraction);
                    let z = 1.0 / tau;
                    let bias = (z / (1.0 - z)).ln() as f32;
                    for component in 0..4 {
                        values[config.width + lane * 4 + component] = bias;
                    }
                }
            } else if name == "read.age" {
                for (index, value) in values.iter_mut().enumerate() {
                    *value = -((index + 1) as f32) / 64.0;
                }
            } else if name == "update.gate.bias" {
                values.fill(-0.5);
            } else if name == "copy.gate.bias" {
                values.fill(-1.0);
            }
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        Self::from_variables(config, variables, device)
    }

    fn from_variables(
        config: JointConfig,
        variables: BTreeMap<String, Var>,
        device: &Device,
    ) -> Result<Self> {
        let order: Vec<u32> = (0..(config.context - 1) as u32).rev().collect();
        let age_order = Tensor::from_vec(order, (config.context - 1,), device)?;
        Ok(Self {
            config,
            variables,
            device: device.clone(),
            age_order,
            quantization: None,
            precision_mode: None,
            prepared_parameters: None,
            hard_only: false,
            interface_audit: None,
        })
    }

    pub fn quantization(&self) -> Option<&QuantizedTrainingState> {
        self.quantization.as_ref()
    }

    pub fn precision_mode(&self) -> Option<PrecisionMode> {
        self.precision_mode
    }

    /// Preserve the stored shadows and frozen quantizers while independently
    /// selecting their use in evaluation. No parameters or clocks are changed.
    /// The caller binds the sealed source checkpoint in its diagnostic report.
    pub fn precision_view(&self, mode: PrecisionMode) -> Result<Self> {
        if self.hard_only || self.prepared_parameters.is_some() || self.precision_mode.is_some() {
            return Err(invalid(
                "precision views require an unprepared floating checkpoint, not a packed or diagnostic view",
            ));
        }
        let state = self
            .quantization
            .as_ref()
            .ok_or_else(|| invalid("precision views require a quantized floating checkpoint"))?;
        // training_strength describes the *next* update, so checking it alone
        // would admit the checkpoint immediately before the last ramp update.
        if state.ramp_steps == 0
            || state
                .completed_step
                .checked_sub(state.start_step)
                .is_none_or(|completed| completed < state.ramp_steps)
        {
            return Err(invalid(
                "precision views require the complete quantization ramp to have finished",
            ));
        }
        state.spec.validate(&self.variables)?;
        let mut model = self.detached_view()?;
        model.precision_mode = Some(mode);
        Ok(model)
    }

    pub fn configure_quantization(&mut self, start_step: usize, ramp_steps: usize) -> Result<()> {
        if self.quantization.is_some()
            || self.hard_only
            || self.precision_mode.is_some()
            || ramp_steps == 0
        {
            return Err(invalid(
                "quantization requires an unquantized parent and positive ramp",
            ));
        }
        self.quantization = Some(QuantizedTrainingState {
            start_step,
            ramp_steps,
            completed_step: start_step,
            spec: joint_quantization::calibrate(&self.variables)?,
        });
        Ok(())
    }

    pub fn set_completed_step(&mut self, step: usize) -> Result<()> {
        if self.prepared_parameters.is_some() || self.hard_only || self.precision_mode.is_some() {
            return Err(invalid(
                "cannot update a prepared, hard-only or precision-view model",
            ));
        }
        if let Some(state) = &mut self.quantization {
            if step < state.completed_step {
                return Err(invalid("quantization clock moved backwards"));
            }
            state.completed_step = step;
        }
        Ok(())
    }

    pub fn training_strength(&self) -> f64 {
        self.quantization.as_ref().map_or(0.0, |state| {
            (state
                .completed_step
                .saturating_sub(state.start_step)
                .saturating_add(1) as f64
                / state.ramp_steps as f64)
                .min(1.0)
        })
    }

    fn detached_view(&self) -> Result<Self> {
        let mut model =
            Self::from_variables(self.config.clone(), self.variables.clone(), &self.device)?;
        model.quantization = self.quantization.clone();
        model.precision_mode = self.precision_mode;
        model.hard_only = self.hard_only;
        model.interface_audit = self.interface_audit.clone();
        Ok(model)
    }

    pub fn without_quantization(&self) -> Result<Self> {
        if self.precision_mode.is_some() {
            return Err(invalid(
                "a precision view cannot discard its evaluation-only mode; request FF from its parent",
            ));
        }
        if self.hard_only {
            return Err(invalid("a packed model has no floating shadow parameters"));
        }
        let mut model = self.detached_view()?;
        model.quantization = None;
        Ok(model)
    }

    fn prepare(&self, training: bool) -> Result<Self> {
        if training && (self.hard_only || self.precision_mode.is_some()) {
            return Err(invalid(
                "packed models and precision views are evaluation-only",
            ));
        }
        let mut model = self.detached_view()?;
        let strength = if training {
            self.training_strength()
        } else {
            1.0
        };
        let parameter_quantization = self.quantization.as_ref().filter(|_| {
            self.precision_mode
                .is_none_or(PrecisionMode::quantizes_parameters)
        });
        let parameters = self
            .variables
            .iter()
            .map(|(name, variable)| {
                let tensor = if training {
                    variable.as_tensor().clone()
                } else {
                    variable.detach()
                };
                let tensor = if let Some(state) = parameter_quantization {
                    state.spec.parameter(name, &tensor, strength, training)?
                } else {
                    tensor
                };
                Ok((name.clone(), tensor))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        model.prepared_parameters = Some(parameters);
        Ok(model)
    }

    fn interface(&self, input: &Tensor, kind: Interface, training: bool) -> Result<Tensor> {
        if self.quantization.is_none()
            || self
                .precision_mode
                .is_some_and(|mode| !mode.quantizes_interfaces())
        {
            return Ok(input.clone());
        }
        let (exponent, low, high) = kind.format();
        if let Some(audit) = &self.interface_audit {
            let scale = 2f32.powi(i32::from(exponent));
            let values = input.detach().flatten_all()?.to_vec1::<f32>()?;
            let mut audit = audit
                .lock()
                .map_err(|_| invalid("interface audit lock poisoned"))?;
            let row = audit.entry(kind.name().into()).or_default();
            for value in values {
                row.count += 1;
                if !value.is_finite() {
                    row.nonfinite += 1;
                    continue;
                }
                row.minimum = Some(row.minimum.map_or(value, |old| old.min(value)));
                row.maximum = Some(row.maximum.map_or(value, |old| old.max(value)));
                if value < low as f32 * scale || value > high as f32 * scale {
                    row.clipped += 1;
                }
            }
        }
        joint_quantization::fake_quant(
            input,
            exponent,
            low,
            high,
            if training {
                self.training_strength()
            } else {
                1.0
            },
            training,
        )
    }

    /// Opt-in observation only. No full-tensor host reads occur in normal fit.
    pub fn enable_interface_audit(&mut self) {
        self.interface_audit = Some(Arc::new(Mutex::new(BTreeMap::new())));
    }

    pub fn interface_audit(&self) -> Result<Value> {
        let audit = self
            .interface_audit
            .as_ref()
            .ok_or_else(|| invalid("interface audit was not enabled"))?;
        let audit = audit
            .lock()
            .map_err(|_| invalid("interface audit lock poisoned"))?;
        Ok(serde_json::to_value(&*audit)?)
    }

    fn normalized(&self, input: &Tensor, training: bool) -> Result<Tensor> {
        self.interface(&rms(input)?, Interface::Normalized, training)
    }

    pub fn variables(&self) -> &BTreeMap<String, Var> {
        &self.variables
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn parameter_count(&self) -> usize {
        self.variables
            .values()
            .map(|value| value.elem_count())
            .sum()
    }

    fn weight(&self, name: &str, training: bool) -> Result<Tensor> {
        if let Some(parameters) = &self.prepared_parameters {
            return parameters
                .get(name)
                .cloned()
                .ok_or_else(|| invalid(format!("missing prepared parameter {name}")));
        }
        let variable = self
            .variables
            .get(name)
            .ok_or_else(|| invalid(format!("missing joint parameter {name}")))?;
        Ok(if training {
            variable.as_tensor().clone()
        } else {
            variable.detach()
        })
    }

    fn linear(&self, input: &Tensor, prefix: &str, training: bool) -> Result<Tensor> {
        let output = input.matmul(&self.weight(&format!("{prefix}.weight"), training)?.t()?)?;
        self.interface(
            &output.broadcast_add(&self.weight(&format!("{prefix}.bias"), training)?)?,
            Interface::Affine,
            training,
        )
    }

    fn initial_memory(&self, batch: usize) -> Result<RecurrentState> {
        if batch == 0 || batch > 64 {
            return Err(invalid("joint batch must be 1..64"));
        }
        Ok(RecurrentState {
            state: Tensor::zeros((batch, self.config.width), DType::F32, &self.device)?,
            keys: None,
            values: None,
            events: Vec::new(),
        })
    }

    /// Batch-major observed input IDs. No target or source-label input exists.
    /// Training preserves the entire unroll graph, including memory writes.
    pub fn forward(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        mode: ReadMode,
        training: bool,
    ) -> Result<JointOutput> {
        if training && self.precision_mode.is_some() {
            return Err(invalid("a precision view is evaluation-only"));
        }
        if self.prepared_parameters.is_none() {
            return self
                .prepare(training)?
                .forward(ids, batch, time, mode, training);
        }
        if time == 0
            || time > self.config.context
            || ids.len()
                != batch
                    .checked_mul(time)
                    .ok_or_else(|| invalid("joint shape overflow"))?
            || ids.iter().any(|&id| id as usize >= self.config.vocab_size)
        {
            return Err(invalid("joint input shape/context/vocabulary"));
        }
        let mut memory = self.initial_memory(batch)?;
        let embedding = self.weight("embedding.weight", training)?;
        let index = Tensor::from_vec(ids.to_vec(), (ids.len(),), &self.device)?;
        let token_affine = embedding
            .index_select(&index, 0)?
            .matmul(&self.weight("recurrent.input.weight", training)?.t()?)?
            .reshape((batch, time, 3 * self.config.width))?;
        let mut states = Vec::with_capacity(time);
        let mut no_reads = Vec::with_capacity(time);
        let mut reads = Vec::with_capacity(time);
        let mut gates = Vec::with_capacity(time);
        for position in 0..time {
            let input: Vec<u32> = (0..batch).map(|lane| ids[lane * time + position]).collect();
            let affine = token_affine
                .narrow(1, position, 1)?
                .squeeze(1)?
                .contiguous()?;
            let step = self.core_step(&mut memory, &input, &affine, mode, training)?;
            states.push(step.state);
            no_reads.push(step.no_read_mass.squeeze(1)?);
            gates.push(step.copy_gate.squeeze(1)?);
            let padded = if position == 0 {
                Tensor::zeros((batch, time), DType::F32, &self.device)?
            } else {
                Tensor::cat(
                    &[
                        &step.read_masses,
                        &Tensor::zeros((batch, time - position), DType::F32, &self.device)?,
                    ],
                    1,
                )?
            };
            reads.push(padded);
        }
        let states = Tensor::stack(&states, 1)?;
        let no_read_mass = Tensor::stack(&no_reads, 1)?;
        let read_masses = Tensor::stack(&reads, 1)?;
        let copy_gate = Tensor::stack(&gates, 1)?;
        let copy = self.training_copy(&read_masses, ids, batch, time)?;
        let probabilities = self
            .output_distribution(
                &states.reshape((batch * time, self.config.width))?,
                &no_read_mass.reshape((batch * time, 1))?,
                &copy_gate.reshape((batch * time, 1))?,
                &copy.reshape((batch * time, self.config.vocab_size))?,
                training,
            )?
            .reshape((batch, time, self.config.vocab_size))?;
        Ok(JointOutput {
            probabilities,
            no_read_mass,
            read_masses,
            copy_gate,
            states,
        })
    }

    pub fn new_session(&self, batch: usize) -> Result<JointSession> {
        Ok(JointSession {
            config: self.config.clone(),
            batch,
            memory: self.initial_memory(batch)?,
            prepared: self.prepare(false)?,
        })
    }

    /// Consume observed inputs once, then emit the next-token distribution.
    /// Every returned read identity predates the current write.
    pub fn step(
        &self,
        session: &mut JointSession,
        input_tokens: &[u32],
        mode: ReadMode,
    ) -> Result<JointStep> {
        if session.config != self.config
            || session.prepared.quantization != self.quantization
            || session.prepared.precision_mode != self.precision_mode
            || session.prepared.variables.iter().any(|(name, variable)| {
                self.variables
                    .get(name)
                    .is_none_or(|actual| actual.id() != variable.id())
            })
            || input_tokens.len() != session.batch
            || input_tokens
                .iter()
                .any(|&id| id as usize >= self.config.vocab_size)
            || session.len() >= self.config.context
        {
            return Err(invalid("joint session model/batch/context/token mismatch"));
        }
        let occurrence = session.len();
        let model = &session.prepared;
        let index = Tensor::from_vec(input_tokens.to_vec(), (session.batch,), &self.device)?;
        let affine = model
            .weight("embedding.weight", false)?
            .index_select(&index, 0)?
            .matmul(&model.weight("recurrent.input.weight", false)?.t()?)?;
        let core = model.core_step(&mut session.memory, input_tokens, &affine, mode, false)?;
        let copy = self.incremental_copy(
            &core.read_masses,
            &session.memory.events[..occurrence],
            session.batch,
        )?;
        let probabilities = model.output_distribution(
            &core.state,
            &core.no_read_mass,
            &core.copy_gate,
            &copy,
            false,
        )?;
        Ok(JointStep {
            probabilities,
            no_read_mass: core.no_read_mass.squeeze(1)?,
            read_masses: core.read_masses,
            copy_gate: core.copy_gate.squeeze(1)?,
            state: core.state,
            read_occurrences: (0..occurrence).collect(),
            written_occurrence: occurrence,
        })
    }

    fn core_step(
        &self,
        memory: &mut RecurrentState,
        input: &[u32],
        token_affine: &Tensor,
        mode: ReadMode,
        training: bool,
    ) -> Result<CoreStep> {
        let batch = input.len();
        let d = self.config.width;
        let previous = memory.events.len();
        let recurrent = self
            .normalized(&memory.state, training)?
            .matmul(&self.weight("recurrent.state.weight", training)?.t()?)?;
        let fused = token_affine
            .add(&recurrent)?
            .broadcast_add(&self.weight("recurrent.bias", training)?)?;
        let fused = self.interface(&fused, Interface::Affine, training)?;
        let candidate =
            self.interface(&fused.narrow(1, 0, d)?.tanh()?, Interface::Unit, training)?;
        let z = self.interface(
            &candle_nn::ops::sigmoid(&fused.narrow(1, d, d)?)?,
            Interface::Gate,
            training,
        )?;
        let raw = fused.narrow(1, 2 * d, d)?;
        let transported =
            transport_lanes_with_unit(&memory.state, &raw, self.config.transport, |unit| {
                self.interface(unit, Interface::Unit, training)
            })?;
        let transported = self.interface(&transported, Interface::State, training)?;
        let provisional = z
            .affine(-1.0, 1.0)?
            .mul(&transported)?
            .add(&z.mul(&candidate)?)?;
        let provisional = self.interface(&provisional, Interface::State, training)?;
        let normalized = self.normalized(&provisional, training)?;
        let (no_read_mass, read_masses, read) = if previous == 0 || mode == ReadMode::NoRead {
            (
                Tensor::ones((batch, 1), DType::F32, &self.device)?,
                Tensor::zeros((batch, previous), DType::F32, &self.device)?,
                Tensor::zeros((batch, d), DType::F32, &self.device)?,
            )
        } else {
            let query = self.linear(&normalized, "read.query", training)?;
            let keys = memory
                .keys
                .as_ref()
                .ok_or_else(|| invalid("missing prior key history"))?;
            let values = memory
                .values
                .as_ref()
                .ok_or_else(|| invalid("missing prior value history"))?;
            let scores = query
                .unsqueeze(1)?
                .matmul(&keys.transpose(1, 2)?.contiguous()?)?
                .squeeze(1)?
                .affine(1.0 / (self.config.read_width as f64).sqrt(), 0.0)?;
            let age_indices =
                self.age_order
                    .narrow(0, self.config.context - 1 - previous, previous)?;
            let age = self
                .weight("read.age", training)?
                .index_select(&age_indices, 0)?
                .unsqueeze(0)?;
            let scores =
                self.interface(&scores.broadcast_add(&age)?, Interface::Affine, training)?;
            let null = self.linear(&normalized, "read.no_read", training)?;
            let mass = candle_nn::ops::softmax(&Tensor::cat(&[&null, &scores], 1)?, 1)?;
            let no_read_mass = mass.narrow(1, 0, 1)?;
            let read_masses = mass.narrow(1, 1, previous)?.contiguous()?;
            let read = read_masses.unsqueeze(1)?.matmul(values)?.squeeze(1)?;
            let read = self.interface(&read, Interface::State, training)?;
            (no_read_mass, read_masses, read)
        };
        let update_input = Tensor::cat(&[&provisional, &read], 1)?;
        let update = self.interface(
            &self.linear(&update_input, "update", training)?.tanh()?,
            Interface::Unit,
            training,
        )?;
        let rho = self.interface(
            &candle_nn::ops::sigmoid(&self.linear(&update_input, "update.gate", training)?)?,
            Interface::Gate,
            training,
        )?;
        let state = provisional
            .broadcast_mul(&rho.affine(-1.0, 1.0)?)?
            .add(&update.broadcast_mul(&rho)?)?;
        let state = self.interface(&state, Interface::State, training)?;
        let copy_input = Tensor::cat(&[&state, &read], 1)?;
        let copy_gate = self.interface(
            &candle_nn::ops::sigmoid(&self.linear(&copy_input, "copy.gate", training)?)?,
            Interface::Gate,
            training,
        )?;
        // Writes happen only after read/update; nothing in this event can be
        // attended until the next call. The last unroll write is unused credit.
        let normalized_write = self.normalized(&state, training)?;
        let key = self.linear(&normalized_write, "read.key", training)?;
        let value = self
            .linear(&normalized_write, "read.value", training)?
            .tanh()?;
        let value = self.interface(&value, Interface::Unit, training)?;
        let keys = append_history(memory.keys.as_ref(), &key, training)?;
        let values = append_history(memory.values.as_ref(), &value, training)?;
        memory.state = if training {
            state.clone()
        } else {
            state.detach()
        };
        memory.keys = Some(keys);
        memory.values = Some(values);
        memory.events.push(MemoryEvent {
            occurrence: previous,
            tokens: input.to_vec(),
        });
        Ok(CoreStep {
            state,
            no_read_mass,
            read_masses,
            copy_gate,
        })
    }

    fn output_distribution(
        &self,
        states: &Tensor,
        no_read: &Tensor,
        gate: &Tensor,
        copy: &Tensor,
        training: bool,
    ) -> Result<Tensor> {
        let hidden = self
            .normalized(states, training)?
            .broadcast_mul(&self.weight("output.norm.weight", training)?)?;
        let hidden = self.interface(&hidden, Interface::Normalized, training)?;
        let logits = hidden
            .matmul(&self.weight("embedding.weight", training)?.t()?)?
            .broadcast_add(&self.weight("output.bias", training)?)?;
        let logits = self.interface(&logits, Interface::Affine, training)?;
        let vocabulary = candle_nn::ops::softmax(&logits, 1)?;
        let read_probability = no_read.affine(-1.0, 1.0)?;
        let vocabulary_fraction = gate.mul(&read_probability)?.affine(-1.0, 1.0)?;
        let mixture = vocabulary
            .broadcast_mul(&vocabulary_fraction)?
            .add(&copy.broadcast_mul(gate)?)?;
        Ok(mixture.affine(
            1.0 - UNIFORM_MIXTURE,
            UNIFORM_MIXTURE / self.config.vocab_size as f64,
        )?)
    }

    fn training_copy(
        &self,
        masses: &Tensor,
        ids: &[u32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        let mut copied = Vec::with_capacity(batch);
        for lane in 0..batch {
            let indexes = Tensor::from_vec(
                ids[lane * time..(lane + 1) * time].to_vec(),
                (time,),
                &self.device,
            )?;
            // Pinned Metal index_add parallelizes over non-indexed axes.
            // [vocabulary,query] therefore runs one lane per query, each
            // accumulating earlier occurrences in their original order.
            // Flattening all axes would serialize the complete B*T*T sum.
            let source = masses.get(lane)?.transpose(0, 1)?.contiguous()?;
            let zeros = Tensor::zeros((self.config.vocab_size, time), DType::F32, &self.device)?;
            copied.push(zeros.index_add(&indexes, &source, 0)?.transpose(0, 1)?);
        }
        // index_add has a source-gradient gather and a complete CPU/Metal
        // implementation in pinned Candle. Future/current masses are zero.
        Ok(Tensor::stack(&copied, 0)?)
    }

    fn incremental_copy(
        &self,
        masses: &Tensor,
        events: &[MemoryEvent],
        batch: usize,
    ) -> Result<Tensor> {
        let zeros = Tensor::zeros((batch * self.config.vocab_size,), DType::F32, &self.device)?;
        if events.is_empty() {
            return Ok(zeros.reshape((batch, self.config.vocab_size))?);
        }
        let mut indexes = Vec::with_capacity(batch * events.len());
        for lane in 0..batch {
            for event in events {
                indexes.push(
                    u32::try_from(lane * self.config.vocab_size + event.tokens[lane] as usize)
                        .map_err(|_| invalid("incremental copy destination exceeds u32"))?,
                );
            }
        }
        let indexes = Tensor::from_vec(indexes, (batch * events.len(),), &self.device)?;
        Ok(zeros
            .index_add(&indexes, &masses.flatten_all()?.contiguous()?, 0)?
            .reshape((batch, self.config.vocab_size))?)
    }

    /// Only parameter/config files are owned here. Caller claims the directory
    /// and saves optimizer, source, data and budget provenance alongside them.
    pub fn save(&self, directory: &Path) -> Result<()> {
        if self.hard_only || self.prepared_parameters.is_some() || self.precision_mode.is_some() {
            return Err(invalid(
                "save a training model, not a prepared, packed or precision view",
            ));
        }
        let mut buffers = BTreeMap::new();
        for (name, variable) in &self.variables {
            let values = variable.detach().flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|value| !value.is_finite()) {
                return Err(invalid(format!(
                    "nonfinite joint checkpoint parameter {name}"
                )));
            }
            let bytes: Vec<u8> = values.into_iter().flat_map(f32::to_le_bytes).collect();
            buffers.insert(name.clone(), (variable.dims().to_vec(), bytes));
        }
        let mut views = Vec::with_capacity(buffers.len());
        for (name, (shape, bytes)) in &buffers {
            views.push((
                name.as_str(),
                TensorView::new(SafeDtype::F32, shape.clone(), bytes)?,
            ));
        }
        let bytes = safetensors::serialize(views, None)?;
        let config = CheckpointConfig {
            schema: CHECKPOINT_SCHEMA.to_owned(),
            model: self.config.clone(),
            weights_sha256: hex::encode(Sha256::digest(&bytes)),
            numerical_contract: self.numerical_contract(),
            quantization: self.quantization.clone(),
        };
        let mut weights = File::create_new(directory.join("model.safetensors"))?;
        weights.write_all(&bytes)?;
        weights.sync_all()?;
        let mut config_file = File::create_new(directory.join("config.json"))?;
        serde_json::to_writer_pretty(&mut config_file, &config)?;
        config_file.write_all(b"\n")?;
        config_file.sync_all()?;
        Ok(())
    }

    pub fn load(directory: &Path, device: &Device) -> Result<Self> {
        let config: CheckpointConfig =
            serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
        config.model.validate()?;
        let expected_contract = if config.quantization.is_some() {
            quantized_numerical_contract()
        } else {
            numerical_contract()
        };
        if config.schema != CHECKPOINT_SCHEMA || config.numerical_contract != expected_contract {
            return Err(invalid(
                "joint checkpoint schema or numerical contract mismatch",
            ));
        }
        let bytes = fs::read(directory.join("model.safetensors"))?;
        if hex::encode(Sha256::digest(&bytes)) != config.weights_sha256 {
            return Err(invalid("joint checkpoint weights hash mismatch"));
        }
        let tensors = SafeTensors::deserialize(&bytes)?;
        let shapes = config.model.shapes();
        let observed: BTreeSet<_> = tensors.names().into_iter().collect();
        let expected: BTreeSet<_> = shapes.keys().map(String::as_str).collect();
        if observed != expected {
            return Err(invalid("joint checkpoint parameter names mismatch"));
        }
        let mut variables = BTreeMap::new();
        for (name, shape) in shapes {
            let view = tensors.tensor(&name)?;
            if view.dtype() != SafeDtype::F32 || view.shape() != shape {
                return Err(invalid(format!("joint checkpoint shape/dtype {name}")));
            }
            let values: Vec<f32> = view
                .data()
                .chunks_exact(4)
                .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .collect();
            if values.iter().any(|value| !value.is_finite()) {
                return Err(invalid(format!("nonfinite loaded joint parameter {name}")));
            }
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        let mut model = Self::from_variables(config.model, variables, device)?;
        if let Some(state) = &config.quantization {
            state.spec.validate(model.variables())?;
            if state.ramp_steps == 0 || state.completed_step < state.start_step {
                return Err(invalid("invalid quantization checkpoint clock"));
            }
        }
        model.quantization = config.quantization;
        Ok(model)
    }

    pub fn numerical_contract(&self) -> Value {
        let mut contract = if self.quantization.is_some() {
            quantized_numerical_contract()
        } else {
            numerical_contract()
        };
        if let Some(mode) = self.precision_mode {
            contract["precision_evaluation"] = json!({
                "mode":mode,
                "parameter_quantization":mode.quantizes_parameters(),
                "interface_quantization":mode.quantizes_interfaces(),
                "scope":"Evaluation-only intervention on the same stored shadows and frozen grids; no calibration, optimizer update, checkpoint selection or serving qualification",
                "quantization_contract_scope":"The retained grid definitions apply only where the corresponding precision switch is enabled"
            });
            contract["quantization"]["backward"] =
                json!("Disabled: precision views reject training and use full-strength grids only where enabled");
            contract["quantization"]["artifact"] =
                json!("In-memory diagnostic view of a floating checkpoint; cannot be saved as a training or packed model");
        }
        contract
    }

    /// Packed parameter export for the shared quantized F32 evaluator. This is
    /// deliberately not an integer execution kernel: the nonlinearities,
    /// accumulations, normalization and probability calculations remain F32.
    pub fn save_hard(&self, directory: &Path) -> Result<Value> {
        if self.precision_mode.is_some() {
            return Err(invalid("a precision view cannot be exported"));
        }
        let state = self
            .quantization
            .as_ref()
            .ok_or_else(|| invalid("hard export requires frozen quantization scales"))?;
        let parameters =
            joint_quantization::save_hard_parameters(&state.spec, &self.variables, directory)?;
        let manifest = json!({
            "schema":"uor-r4.joint-recurrent-packed-emulator/1",
            "model":self.config,
            "quantization":state,
            "numerical_contract":quantized_numerical_contract(),
            "parameter_manifest":parameters,
            "parameter_manifest_sha256":crate::sha256_file(&directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE))?,
            "scope":"Packed signed 4-bit multiplicative weights and signed 16-bit additive offsets; dyadic scales; quantized recurrent interfaces; F32 emulation, not D0-b integer serving"
        });
        let mut file = File::create_new(directory.join("hard-model.json"))?;
        serde_json::to_writer_pretty(&mut file, &manifest)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(manifest)
    }

    pub fn load_hard(directory: &Path, device: &Device) -> Result<Self> {
        let manifest: Value =
            serde_json::from_slice(&fs::read(directory.join("hard-model.json"))?)?;
        if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
            || manifest["numerical_contract"] != quantized_numerical_contract()
            || manifest["parameter_manifest_sha256"]
                != crate::sha256_file(
                    &directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE),
                )?
        {
            return Err(invalid(
                "packed model schema, numerical contract or parameter manifest hash",
            ));
        }
        let actual_manifest: Value = serde_json::from_slice(&fs::read(
            directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE),
        )?)?;
        if actual_manifest != manifest["parameter_manifest"] {
            return Err(invalid("packed model parameter manifest binding"));
        }
        let config: JointConfig = serde_json::from_value(manifest["model"].clone())?;
        config.validate()?;
        let state: QuantizedTrainingState =
            serde_json::from_value(manifest["quantization"].clone())?;
        let (spec, variables) = joint_quantization::load_hard_parameters(directory, device)?;
        if spec != state.spec
            || state.ramp_steps == 0
            || state.completed_step < state.start_step
            || variables
                .iter()
                .map(|(name, var)| (name.clone(), var.dims().to_vec()))
                .collect::<BTreeMap<_, _>>()
                != config.shapes()
        {
            return Err(invalid(
                "packed model parameter set, scale or clock mismatch",
            ));
        }
        let mut model = Self::from_variables(config, variables, device)?;
        model.quantization = Some(state);
        model.hard_only = true;
        Ok(model)
    }
}

#[derive(Serialize, Deserialize)]
struct CheckpointConfig {
    schema: String,
    model: JointConfig,
    weights_sha256: String,
    numerical_contract: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quantization: Option<QuantizedTrainingState>,
}

pub fn quantized_numerical_contract() -> Value {
    let mut contract = numerical_contract();
    contract["quantization"] = json!({
        "schema":"uor-r4.joint-recurrent-quantized-interfaces/1",
        "parameters":"All multiplicative parameters, including embedding and output norm, signed 4-bit [-7,7]; additive biases/age signed 16-bit [-32767,32767]; frozen parent-calibrated power-of-two per-output-row scales (one scale for vectors)",
        "calibration":"4-bit row reconstruction MSE chooses among bounded ceil(log2(maxabs/7)) minus 2, minus 1, and ceiling; ties prefer larger exponent; additive16 uses ceiling. Scales never learn or recalibrate.",
        "round":"nearest, ties away from zero; canonical positive zero; clip before round",
        "backward":"inclusive clipped straight-through gradient; identity inside representable range, zero outside at full strength; convex identity/hard ramp during training; hard path in every evaluation",
        "state_transport_provisional_read_state":{"exponent":-11,"codes":[-32767,32767]},
        "rms_normalized_and_output_hidden":{"exponent":-10,"codes":[-32767,32767]},
        "fused_affine_qk_null_read_scores_and_logits":{"exponent":-8,"codes":[-32767,32767]},
        "tanh_values_candidates_updates_and_unit_transport":{"exponent":-14,"codes":[-32767,32767]},
        "sigmoid_gates":{"exponent":-15,"codes":[0,32768]},
        "transport":"Quantize signed unit coordinates after normalization; do not renormalize off-grid. Approximate quaternion and Householder orthogonality only; no hemisphere folding or H4 codebook.",
        "remaining_float":"F32 matmul/elementwise products and accumulation, RMS and unit normalization, sigmoid/tanh/softmax, probability mixture and sampling; fixed scalars are not yet integer-lowered. State-state and attention operations remain dense/full-window.",
        "artifact":"Packed codes plus integer scale exponents, no floating shadow dependency; load decodes dyadic values to F32 for this emulator; training checkpoint separately retains F32 shadows and Adam moments",
        "cache":"Quantized parameter tensors prepared once per full-window shard; incremental sessions snapshot them once; graph-retaining STE tensors are not cached across optimizer updates"
    });
    contract
}

pub fn numerical_contract() -> Value {
    json!({
        "arithmetic":"F32 offline continuous dense affine and softmax computation; approximate floating transport, not exact Z[phi] or target serving",
        "transport":"PerR4 lane q=normalize(e0+alpha*raw), left Hamilton multiplication; ordinary control H(v)H(e0), v=normalize(e0+alpha/sqrt2*raw)",
        "quaternion_delta_scale":QUATERNION_DELTA_SCALE,
        "householder_delta_scale":QUATERNION_DELTA_SCALE/2f64.sqrt(),
        "quaternion_sign":"q and -q remain distinct; no hemisphere canonicalization",
        "transport_min_norm":TRANSPORT_MIN_NORM,
        "transport_zero_rule":"Clamp squared denominator before sqrt; at norm<=minimum select identity e0 with finite excluded-branch arithmetic",
        "rms_epsilon":RMS_EPSILON,
        "retention_initialization":"Each R4 lane has one constant initial update-gate bias with log-spaced tau4..64; learned coordinate gates may subsequently differ",
        "rho_initial_bias":-0.5,
        "copy_initial_bias":-1.0,
        "read":"Query/Key from RMS-normalized provisional/written states; Value=tanh(affine(normalized written state)); score=dot/sqrt(read_width)+learned age, competing with learned NoRead",
        "age_initialization":"-(occurrence age)/64; age starts1 for the immediately previous write",
        "output":"(1-g*(1-a0))*P_vocabulary + g*sum_previous(a_i*onehot(observed_token_i)), then declared uniform mixture",
        "uniform_mixture_epsilon":UNIFORM_MIXTURE,
        "write_order":"Read previous occurrences, update state, write current observed token Key/Value for next step",
        "training_credit":"Full differentiable unroll; no state or memory detach; labels only enter next-token NLL",
        "session":"Same core with detached parameters/state, exact token/occurrence tape, bounded context without eviction"
    })
}

/// Preserve all previous-write gradients with one two-input concatenation.
/// Candle's stack/cat otherwise encodes one copy and one gradient edge per
/// historical write on every step. Both layouts copy the same history bytes;
/// this layout reduces encoder and graph-node counts without mutable writes.
fn append_history(history: Option<&Tensor>, write: &Tensor, training: bool) -> Result<Tensor> {
    let write = write.unsqueeze(1)?;
    let appended = match history {
        Some(previous) => Tensor::cat(&[previous, &write], 1)?,
        None => write,
    };
    Ok(if training {
        appended
    } else {
        appended.detach()
    })
}

fn rms(input: &Tensor) -> Result<Tensor> {
    Ok(input.broadcast_div(
        &input
            .sqr()?
            .mean_keepdim(1)?
            .affine(1.0, RMS_EPSILON)?
            .sqrt()?,
    )?)
}

/// Matched four-coordinate lane transports. Identity-centered normalization
/// has three tangent degrees of freedom in either arm; the radial direction
/// is redundant. The alpha/sqrt2 control factor matches local Frobenius scale.
pub fn transport_lanes(input: &Tensor, raw: &Tensor, transport: Transport) -> Result<Tensor> {
    transport_lanes_with_unit(input, raw, transport, |unit| Ok(unit.clone()))
}

fn transport_lanes_with_unit(
    input: &Tensor,
    raw: &Tensor,
    transport: Transport,
    quantize_unit: impl FnOnce(&Tensor) -> Result<Tensor>,
) -> Result<Tensor> {
    let (batch, width) = input.dims2()?;
    if width == 0 || width % 4 != 0 || raw.dims() != input.dims() {
        return Err(invalid("transport lane shape mismatch"));
    }
    let lanes = width / 4;
    let raw = raw.reshape((batch, lanes, 4))?;
    let identity = Tensor::from_vec(vec![1f32, 0.0, 0.0, 0.0], (1, 1, 4), input.device())?
        .broadcast_as((batch, lanes, 4))?;
    let scale = match transport {
        Transport::Quaternion => QUATERNION_DELTA_SCALE,
        Transport::HouseholderPair => QUATERNION_DELTA_SCALE / 2f64.sqrt(),
    };
    let centered = raw.affine(scale, 0.0)?.add(&identity)?;
    let squared = centered.sqr()?.sum_keepdim(2)?;
    // A safe branch is essential: where_cond alone does not make division by
    // zero in its excluded backward graph safe.
    let denominator = squared
        .clamp((TRANSPORT_MIN_NORM * TRANSPORT_MIN_NORM) as f32, f32::MAX)?
        .sqrt()?;
    let normalized = centered.broadcast_div(&denominator)?;
    let active = squared
        .gt((TRANSPORT_MIN_NORM * TRANSPORT_MIN_NORM) as f32)?
        .broadcast_as((batch, lanes, 4))?;
    // Quantized coordinates are not renormalized off their declared grid.
    // Consequently the quantized quaternion/Householder map is approximately,
    // not exactly, orthogonal; both arms retain their signed coordinates.
    let unit = quantize_unit(&active.where_cond(&normalized, &identity)?)?;
    let x = input.reshape((batch, lanes, 4))?;
    let result = match transport {
        Transport::Quaternion => {
            let w = unit.narrow(2, 0, 1)?;
            let a = unit.narrow(2, 1, 1)?;
            let b = unit.narrow(2, 2, 1)?;
            let c = unit.narrow(2, 3, 1)?;
            let x0 = x.narrow(2, 0, 1)?;
            let x1 = x.narrow(2, 1, 1)?;
            let x2 = x.narrow(2, 2, 1)?;
            let x3 = x.narrow(2, 3, 1)?;
            let y0 = w
                .mul(&x0)?
                .sub(&a.mul(&x1)?)?
                .sub(&b.mul(&x2)?)?
                .sub(&c.mul(&x3)?)?;
            let y1 = w
                .mul(&x1)?
                .add(&a.mul(&x0)?)?
                .add(&b.mul(&x3)?)?
                .sub(&c.mul(&x2)?)?;
            let y2 = w
                .mul(&x2)?
                .sub(&a.mul(&x3)?)?
                .add(&b.mul(&x0)?)?
                .add(&c.mul(&x1)?)?;
            let y3 = w
                .mul(&x3)?
                .add(&a.mul(&x2)?)?
                .sub(&b.mul(&x1)?)?
                .add(&c.mul(&x0)?)?;
            Tensor::cat(&[&y0, &y1, &y2, &y3], 2)?
        }
        Transport::HouseholderPair => {
            let reflected = Tensor::cat(&[&x.narrow(2, 0, 1)?.neg()?, &x.narrow(2, 1, 3)?], 2)?;
            let dot = unit.mul(&reflected)?.sum_keepdim(2)?;
            reflected.sub(&unit.broadcast_mul(&dot)?.affine(2.0, 0.0)?)?
        }
    };
    Ok(result.reshape((batch, width))?)
}

struct Initializer(u64);
impl Initializer {
    fn symmetric(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^= value >> 31;
        ((value >> 40) as f32) * (2.0 / 16_777_216.0) - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(transport: Transport) -> JointConfig {
        JointConfig {
            width: 128,
            context: 8,
            transport,
            seed: 7,
            ..JointConfig::default()
        }
    }
    fn max_delta(a: &Tensor, b: &Tensor) -> Result<f32> {
        Ok(a.sub(b)?.abs()?.flatten_all()?.max(0)?.to_scalar::<f32>()?)
    }

    fn same_bits(a: &Tensor, b: &Tensor) -> Result<bool> {
        Ok(a.dims() == b.dims()
            && a.flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .map(|value| value.to_bits())
                .eq(b
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .map(|value| value.to_bits())))
    }

    fn precision_test_root(label: &str) -> Result<std::path::PathBuf> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock before epoch"))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-joint-precision-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        Ok(root)
    }

    #[test]
    fn precision_modes_apply_independent_frozen_quantizers() -> Result<()> {
        let mut parent = JointModel::new(small(Transport::Quaternion), &Device::Cpu)?;
        parent.configure_quantization(7, 4)?;
        parent.set_completed_step(11)?;
        let state = parent
            .quantization()
            .ok_or_else(|| invalid("test quantization missing"))?;
        let original = parent
            .variables()
            .iter()
            .map(|(name, value)| Ok((name.clone(), (value.id(), value.detach().copy()?))))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let probe = Tensor::from_vec(
            vec![-999f32, -0.1234567, -0.0, 0.0, 0.00012, 0.345, 999.0],
            (7,),
            &Device::Cpu,
        )?;
        for (mode, parameters, interfaces) in [
            (PrecisionMode::FF, false, false),
            (PrecisionMode::QF, true, false),
            (PrecisionMode::FQ, false, true),
            (PrecisionMode::QQ, true, true),
        ] {
            assert_eq!(PrecisionMode::parse(mode.name())?, mode);
            assert_eq!(
                PrecisionMode::parse(&mode.name().to_ascii_lowercase())?,
                mode
            );
            assert_eq!(serde_json::to_value(mode)?, json!(mode.name()));
            assert_eq!(
                serde_json::from_value::<PrecisionMode>(json!(mode.name()))?,
                mode
            );
            assert_eq!(mode.quantizes_parameters(), parameters);
            assert_eq!(mode.quantizes_interfaces(), interfaces);
            let view = parent.precision_view(mode)?;
            let prepared = view.prepare(false)?;
            assert_eq!(prepared.precision_mode(), Some(mode));
            assert_eq!(prepared.quantization(), parent.quantization());
            let contract = view.numerical_contract();
            assert_eq!(contract["precision_evaluation"]["mode"], mode.name());
            assert_eq!(
                contract["precision_evaluation"]["parameter_quantization"],
                parameters
            );
            assert_eq!(
                contract["precision_evaluation"]["interface_quantization"],
                interfaces
            );
            let mut rounding_witnessed = false;
            for (name, variable) in parent.variables() {
                let shadow = variable.detach();
                let hard = state.spec.parameter(name, &shadow, 1.0, false)?;
                rounding_witnessed |= !same_bits(&shadow, &hard)?;
                let actual = prepared.weight(name, false)?;
                assert!(!actual.track_op());
                assert!(
                    same_bits(&actual, if parameters { &hard } else { &shadow })?,
                    "{mode:?}: {name}"
                );
            }
            assert!(rounding_witnessed);
            for kind in [
                Interface::State,
                Interface::Normalized,
                Interface::Affine,
                Interface::Unit,
                Interface::Gate,
            ] {
                let (exponent, low, high) = kind.format();
                let hard = joint_quantization::fake_quant(&probe, exponent, low, high, 1.0, false)?;
                assert!(!same_bits(&probe, &hard)?);
                let actual = prepared.interface(&probe, kind, false)?;
                assert!(
                    same_bits(&actual, if interfaces { &hard } else { &probe })?,
                    "{mode:?}: {}",
                    kind.name()
                );
            }
        }
        assert!(PrecisionMode::parse("F").is_err());
        for (name, variable) in parent.variables() {
            let (id, value) = &original[name];
            assert_eq!(&variable.id(), id);
            assert!(same_bits(variable.as_tensor(), value)?, "{name}");
        }
        assert_eq!(parent.precision_mode(), None);
        assert_eq!(parent.numerical_contract(), quantized_numerical_contract());
        Ok(())
    }

    #[test]
    fn precision_views_match_endpoints_sessions_and_causal_prefix() -> Result<()> {
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        let mut edited = ids;
        edited[4] = 8;
        edited[9] = 5;
        for transport in [Transport::Quaternion, Transport::HouseholderPair] {
            let mut parent = JointModel::new(small(transport), &Device::Cpu)?;
            parent.configure_quantization(7, 4)?;
            parent.set_completed_step(11)?;
            let root = precision_test_root("endpoints")?;
            let floating = root.join("floating");
            let packed = root.join("packed");
            fs::create_dir(&floating)?;
            fs::create_dir(&packed)?;
            parent.save(&floating)?;
            parent.save_hard(&packed)?;
            let parent = JointModel::load(&floating, &Device::Cpu)?;
            let hard = JointModel::load_hard(&packed, &Device::Cpu)?;
            let shadow = parent.without_quantization()?;
            let modes = [
                PrecisionMode::FF,
                PrecisionMode::QF,
                PrecisionMode::FQ,
                PrecisionMode::QQ,
            ];
            for mode in modes {
                assert!(hard.precision_view(mode).is_err());
                let view = parent.precision_view(mode)?;
                for read_mode in [ReadMode::Enabled, ReadMode::NoRead] {
                    let whole = view.forward(&ids, 2, 5, read_mode, false)?;
                    assert!(!whole.probabilities.track_op());
                    for reference in match mode {
                        PrecisionMode::FF => vec![&shadow],
                        PrecisionMode::QQ => vec![&parent, &hard],
                        _ => vec![],
                    } {
                        let expected = reference.forward(&ids, 2, 5, read_mode, false)?;
                        for (actual, expected) in [
                            (&whole.probabilities, &expected.probabilities),
                            (&whole.states, &expected.states),
                            (&whole.read_masses, &expected.read_masses),
                            (&whole.no_read_mass, &expected.no_read_mass),
                            (&whole.copy_gate, &expected.copy_gate),
                        ] {
                            assert!(same_bits(actual, expected)?, "{transport:?} {mode:?}");
                        }
                    }
                    let mut session = view.new_session(2)?;
                    assert_eq!(session.prepared.precision_mode(), Some(mode));
                    assert!(parent.step(&mut session, &[3, 10], read_mode).is_err());
                    for other in modes.into_iter().filter(|other| *other != mode) {
                        assert!(parent
                            .precision_view(other)?
                            .step(&mut session, &[3, 10], read_mode)
                            .is_err());
                    }
                    assert!(session.is_empty());
                    for position in 0..5 {
                        let step = view.step(
                            &mut session,
                            &[ids[position], ids[5 + position]],
                            read_mode,
                        )?;
                        assert!(
                            max_delta(
                                &step.probabilities,
                                &whole.probabilities.narrow(1, position, 1)?.squeeze(1)?
                            )? < 3e-6,
                            "{transport:?} {mode:?} {read_mode:?} position {position}"
                        );
                        assert_eq!(step.read_occurrences, (0..position).collect::<Vec<_>>());
                        assert_eq!(
                            session.events()[position].tokens,
                            vec![ids[position], ids[5 + position]]
                        );
                    }
                    let future = view.forward(&edited, 2, 5, read_mode, false)?;
                    assert!(same_bits(
                        &whole.probabilities.narrow(1, 0, 4)?,
                        &future.probabilities.narrow(1, 0, 4)?
                    )?);
                    assert!(same_bits(
                        &whole.states.narrow(1, 0, 4)?,
                        &future.states.narrow(1, 0, 4)?
                    )?);
                }
            }
            fs::remove_dir_all(root)?;
        }
        Ok(())
    }

    #[test]
    fn precision_views_reject_training_export_and_incomplete_parents() -> Result<()> {
        let mut parent = JointModel::new(small(Transport::Quaternion), &Device::Cpu)?;
        assert!(parent.precision_view(PrecisionMode::FF).is_err());
        parent.configure_quantization(7, 4)?;
        assert!(parent.precision_view(PrecisionMode::QQ).is_err());
        parent.set_completed_step(10)?;
        assert_eq!(parent.training_strength(), 1.0);
        assert!(parent.precision_view(PrecisionMode::QQ).is_err());
        parent.set_completed_step(11)?;
        assert!(parent
            .prepare(false)?
            .precision_view(PrecisionMode::FF)
            .is_err());
        let root = precision_test_root("rejections")?;
        for mode in [
            PrecisionMode::FF,
            PrecisionMode::QF,
            PrecisionMode::FQ,
            PrecisionMode::QQ,
        ] {
            let mut view = parent.precision_view(mode)?;
            assert!(view
                .forward(&[3, 4], 1, 2, ReadMode::Enabled, true)
                .is_err());
            assert!(view.prepare(true).is_err());
            assert!(view
                .prepare(false)?
                .forward(&[3, 4], 1, 2, ReadMode::Enabled, true)
                .is_err());
            assert!(view.save(&root).is_err());
            assert!(view.save_hard(&root).is_err());
            assert!(view.configure_quantization(11, 4).is_err());
            assert!(view.set_completed_step(12).is_err());
            assert!(view.without_quantization().is_err());
            assert!(view.precision_view(PrecisionMode::FF).is_err());
            assert_eq!(view.precision_mode(), Some(mode));
            assert_eq!(view.quantization(), parent.quantization());
        }
        assert!(fs::read_dir(&root)?.next().is_none());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn quantized_shared_core_gradients_and_packed_reload() -> Result<()> {
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        let targets = [17, 4, 9, 2, 5, 6, 11, 7, 22, 8];
        for transport in [Transport::Quaternion, Transport::HouseholderPair] {
            let mut model = JointModel::new(small(transport), &Device::Cpu)?;
            model.configure_quantization(7, 4)?;
            assert_eq!(model.training_strength(), 0.25);
            model.set_completed_step(10)?;
            assert_eq!(model.training_strength(), 1.0);
            let trained = model.forward(&ids, 2, 5, ReadMode::Enabled, true)?;
            let evaluated = model.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
            assert_eq!(
                max_delta(&trained.probabilities, &evaluated.probabilities)?,
                0.0
            );
            let gradients = trained.loss(&targets)?.backward()?;
            for (name, variable) in model.variables() {
                let values = gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("missing STE gradient {name}")))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert!(values.iter().all(|value| value.is_finite()), "{name}");
                assert!(values.iter().any(|value| *value != 0.0), "{name}");
            }
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| invalid("test clock before epoch"))?
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("uor-joint-packed-{}-{nonce}", std::process::id()));
            std::fs::create_dir(&root)?;
            let saved = root.join("shadow");
            let packed = root.join("packed");
            std::fs::create_dir(&saved)?;
            std::fs::create_dir(&packed)?;
            model.save(&saved)?;
            model.save_hard(&packed)?;
            let reloaded = JointModel::load(&saved, &Device::Cpu)?;
            let hard = JointModel::load_hard(&packed, &Device::Cpu)?;
            assert_eq!(model.quantization(), reloaded.quantization());
            assert_eq!(model.quantization(), hard.quantization());
            assert!(!packed.join("model.safetensors").exists());
            assert!(hard.without_quantization().is_err());
            assert!(hard.forward(&ids, 2, 5, ReadMode::Enabled, true).is_err());
            for mode in [ReadMode::Enabled, ReadMode::NoRead] {
                let expected = model.forward(&ids, 2, 5, mode, false)?;
                let restored = reloaded.forward(&ids, 2, 5, mode, false)?;
                let packed_output = hard.forward(&ids, 2, 5, mode, false)?;
                assert_eq!(
                    max_delta(&expected.probabilities, &restored.probabilities)?,
                    0.0
                );
                assert_eq!(
                    max_delta(&expected.probabilities, &packed_output.probabilities)?,
                    0.0
                );
                let mut session = hard.new_session(2)?;
                for position in 0..5 {
                    let step =
                        hard.step(&mut session, &[ids[position], ids[5 + position]], mode)?;
                    assert!(
                        max_delta(
                            &step.probabilities,
                            &packed_output
                                .probabilities
                                .narrow(1, position, 1)?
                                .squeeze(1)?
                        )? < 3e-6
                    );
                    for value in step.state.flatten_all()?.to_vec1::<f32>()? {
                        assert_eq!(value * 2048.0, (value * 2048.0).round());
                    }
                    assert_eq!(step.read_occurrences, (0..position).collect::<Vec<_>>());
                }
            }
            let mut edited = ids;
            edited[4] = 8;
            edited[9] = 5;
            let future = hard.forward(&edited, 2, 5, ReadMode::Enabled, false)?;
            assert_eq!(
                max_delta(
                    &evaluated.probabilities.narrow(1, 0, 4)?,
                    &future.probabilities.narrow(1, 0, 4)?
                )?,
                0.0
            );
            std::fs::remove_dir_all(root)?;
        }
        Ok(())
    }

    #[test]
    fn accumulated_history_matches_stack_values_and_all_write_gradients() -> Result<()> {
        let device = Device::Cpu;
        let writes = (0..5)
            .map(|position| {
                Var::from_vec(
                    (0..6)
                        .map(|coordinate| (position * 6 + coordinate + 1) as f32 / 32.0)
                        .collect::<Vec<_>>(),
                    (2, 3),
                    &device,
                )
            })
            .collect::<candle_core::Result<Vec<_>>>()?;
        let mut history: Option<Tensor> = None;
        let mut accumulated_loss = Tensor::zeros((), DType::F32, &device)?;
        let mut stacked_loss = Tensor::zeros((), DType::F32, &device)?;
        for (position, write) in writes.iter().enumerate() {
            let appended = append_history(history.as_ref(), write.as_tensor(), true)?;
            let originals: Vec<_> = writes[..=position].iter().map(Var::as_tensor).collect();
            let stacked = Tensor::stack(&originals, 1)?;
            assert_eq!(max_delta(&appended, &stacked)?, 0.0);
            accumulated_loss = accumulated_loss.add(&appended.sqr()?.sum_all()?)?;
            stacked_loss = stacked_loss.add(&stacked.sqr()?.sum_all()?)?;
            history = Some(appended);
        }
        let accumulated_gradients = accumulated_loss.backward()?;
        let stacked_gradients = stacked_loss.backward()?;
        for write in &writes {
            let actual = accumulated_gradients
                .get(write.as_tensor())
                .ok_or_else(|| invalid("missing accumulated-history write gradient"))?;
            let expected = stacked_gradients
                .get(write.as_tensor())
                .ok_or_else(|| invalid("missing stacked-history write gradient"))?;
            assert!(max_delta(actual, expected)? < 1e-6);
        }
        let detached = append_history(history.as_ref(), writes[0].as_tensor(), false)?;
        assert!(!detached.track_op());
        Ok(())
    }

    #[test]
    fn transport_identity_norm_sign_and_local_scale() -> Result<()> {
        let device = Device::Cpu;
        let input = Tensor::from_vec(vec![1f32, 2., 3., 4., -2., 1., 0.5, -0.25], (2, 4), &device)?;
        let zero = Tensor::zeros((2, 4), DType::F32, &device)?;
        for kind in [Transport::Quaternion, Transport::HouseholderPair] {
            assert!(max_delta(&transport_lanes(&input, &zero, kind)?, &input)? < 1e-6);
            let raw = Tensor::from_vec(
                vec![0.2f32, -0.4, 0.7, 0.3, -0.1, 0.8, 0.2, -0.5],
                (2, 4),
                &device,
            )?;
            let transformed = transport_lanes(&input, &raw, kind)?;
            assert!(max_delta(&transformed.sqr()?.sum(1)?, &input.sqr()?.sum(1)?)? < 1e-5);
        }
        let negative =
            Tensor::from_vec(vec![-20f32, 0., 0., 0., -20., 0., 0., 0.], (2, 4), &device)?;
        assert!(
            max_delta(
                &transport_lanes(&input, &negative, Transport::Quaternion)?,
                &input.neg()?
            )? < 1e-6
        );
        let basis = Tensor::eye(4, DType::F32, &device)?;
        for direction in 1..4 {
            let mut values = vec![0f32; 16];
            for row in 0..4 {
                values[row * 4 + direction] = 0.001;
            }
            let delta = Tensor::from_vec(values, (4, 4), &device)?;
            let q = transport_lanes(&basis, &delta, Transport::Quaternion)?
                .sub(&basis)?
                .sqr()?
                .sum_all()?
                .to_scalar::<f32>()?;
            let h = transport_lanes(&basis, &delta, Transport::HouseholderPair)?
                .sub(&basis)?
                .sqr()?
                .sum_all()?
                .to_scalar::<f32>()?;
            assert!((q / h - 1.0).abs() < 1e-3);
        }
        Ok(())
    }

    #[test]
    fn causal_normalized_read_and_no_read_distributions() -> Result<()> {
        let model = JointModel::new(small(Transport::Quaternion), &Device::Cpu)?;
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        let output = model.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
        assert!(!output.probabilities.track_op());
        for sum in output
            .probabilities
            .sum(2)?
            .flatten_all()?
            .to_vec1::<f32>()?
        {
            assert!((sum - 1.0).abs() < 3e-6);
        }
        let mut edited = ids;
        edited[4] = 8;
        edited[9] = 5;
        let future = model.forward(&edited, 2, 5, ReadMode::Enabled, false)?;
        assert!(
            max_delta(
                &output.probabilities.narrow(1, 0, 4)?,
                &future.probabilities.narrow(1, 0, 4)?
            )? < 1e-6
        );
        let masses = output.read_masses.to_vec3::<f32>()?;
        for lane in masses {
            for (position, row) in lane.iter().enumerate() {
                assert!(row[position..].iter().all(|&mass| mass == 0.0));
            }
        }
        let off = model.forward(&ids, 2, 5, ReadMode::NoRead, false)?;
        assert!(off
            .no_read_mass
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|&a| a == 1.0));
        assert!(off
            .read_masses
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|&a| a == 0.0));
        Ok(())
    }

    #[test]
    fn both_arms_use_same_initial_arrays_and_incremental_core() -> Result<()> {
        let q = JointModel::new(small(Transport::Quaternion), &Device::Cpu)?;
        let h = JointModel::new(small(Transport::HouseholderPair), &Device::Cpu)?;
        for (name, value) in q.variables() {
            let other = h
                .variables()
                .get(name)
                .ok_or_else(|| invalid("matched parameter missing"))?;
            assert_eq!(
                value.flatten_all()?.to_vec1::<f32>()?,
                other.flatten_all()?.to_vec1::<f32>()?
            );
        }
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        for model in [&q, &h] {
            let whole = model.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
            let mut session = model.new_session(2)?;
            for position in 0..5 {
                let step = model.step(
                    &mut session,
                    &[ids[position], ids[5 + position]],
                    ReadMode::Enabled,
                )?;
                assert!(
                    max_delta(
                        &step.probabilities,
                        &whole.probabilities.narrow(1, position, 1)?.squeeze(1)?
                    )? < 2e-6
                );
                assert_eq!(step.read_occurrences, (0..position).collect::<Vec<_>>());
                assert_eq!(
                    session.events()[position].tokens,
                    vec![ids[position], ids[5 + position]]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn language_loss_reaches_query_key_value_and_recurrent_state() -> Result<()> {
        for kind in [Transport::Quaternion, Transport::HouseholderPair] {
            let model = JointModel::new(small(kind), &Device::Cpu)?;
            let output = model.forward(
                &[3, 17, 4, 9, 2, 10, 6, 11, 7, 22],
                2,
                5,
                ReadMode::Enabled,
                true,
            )?;
            let loss = output.loss(&[17, 4, 9, 2, 5, 6, 11, 7, 22, 8])?;
            assert!(loss.to_scalar::<f32>()?.is_finite());
            let gradients = loss.backward()?;
            for name in [
                "read.query.weight",
                "read.key.weight",
                "read.value.weight",
                "recurrent.state.weight",
                "recurrent.input.weight",
                "read.age",
                "read.no_read.weight",
            ] {
                let variable = model
                    .variables()
                    .get(name)
                    .ok_or_else(|| invalid("gradient parameter missing"))?;
                let gradient = gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("missing language gradient {name}")))?;
                let values = gradient.flatten_all()?.to_vec1::<f32>()?;
                assert!(values.iter().all(|value| value.is_finite()), "{name}");
                assert!(values.iter().any(|&value| value != 0.0), "{name}");
            }
        }
        Ok(())
    }
}
