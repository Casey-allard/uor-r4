//! Continuous causal recurrent memory learner for D8 rung 1.
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

use candle_core::{DType, Device, Tensor, Var};
use safetensors::{tensor::TensorView, Dtype as SafeDtype, SafeTensors};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

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
        })
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
        Ok(output.broadcast_add(&self.weight(&format!("{prefix}.bias"), training)?)?)
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
            || input_tokens.len() != session.batch
            || input_tokens
                .iter()
                .any(|&id| id as usize >= self.config.vocab_size)
            || session.len() >= self.config.context
        {
            return Err(invalid("joint session model/batch/context/token mismatch"));
        }
        let occurrence = session.len();
        let index = Tensor::from_vec(input_tokens.to_vec(), (session.batch,), &self.device)?;
        let affine = self
            .weight("embedding.weight", false)?
            .index_select(&index, 0)?
            .matmul(&self.weight("recurrent.input.weight", false)?.t()?)?;
        let core = self.core_step(&mut session.memory, input_tokens, &affine, mode, false)?;
        let copy = self.incremental_copy(
            &core.read_masses,
            &session.memory.events[..occurrence],
            session.batch,
        )?;
        let probabilities = self.output_distribution(
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
        let recurrent =
            rms(&memory.state)?.matmul(&self.weight("recurrent.state.weight", training)?.t()?)?;
        let fused = token_affine
            .add(&recurrent)?
            .broadcast_add(&self.weight("recurrent.bias", training)?)?;
        let candidate = fused.narrow(1, 0, d)?.tanh()?;
        let z = candle_nn::ops::sigmoid(&fused.narrow(1, d, d)?)?;
        let raw = fused.narrow(1, 2 * d, d)?;
        let transported = transport_lanes(&memory.state, &raw, self.config.transport)?;
        let provisional = z
            .affine(-1.0, 1.0)?
            .mul(&transported)?
            .add(&z.mul(&candidate)?)?;
        let normalized = rms(&provisional)?;
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
            let scores = scores.broadcast_add(&age)?;
            let null = self.linear(&normalized, "read.no_read", training)?;
            let mass = candle_nn::ops::softmax(&Tensor::cat(&[&null, &scores], 1)?, 1)?;
            let no_read_mass = mass.narrow(1, 0, 1)?;
            let read_masses = mass.narrow(1, 1, previous)?.contiguous()?;
            let read = read_masses.unsqueeze(1)?.matmul(values)?.squeeze(1)?;
            (no_read_mass, read_masses, read)
        };
        let update_input = Tensor::cat(&[&provisional, &read], 1)?;
        let update = self.linear(&update_input, "update", training)?.tanh()?;
        let rho = candle_nn::ops::sigmoid(&self.linear(&update_input, "update.gate", training)?)?;
        let state = provisional
            .broadcast_mul(&rho.affine(-1.0, 1.0)?)?
            .add(&update.broadcast_mul(&rho)?)?;
        let copy_input = Tensor::cat(&[&state, &read], 1)?;
        let copy_gate =
            candle_nn::ops::sigmoid(&self.linear(&copy_input, "copy.gate", training)?)?;
        // Writes happen only after read/update; nothing in this event can be
        // attended until the next call. The last unroll write is unused credit.
        let normalized_write = rms(&state)?;
        let key = self.linear(&normalized_write, "read.key", training)?;
        let value = self
            .linear(&normalized_write, "read.value", training)?
            .tanh()?;
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
        let hidden = rms(states)?.broadcast_mul(&self.weight("output.norm.weight", training)?)?;
        let logits = hidden
            .matmul(&self.weight("embedding.weight", training)?.t()?)?
            .broadcast_add(&self.weight("output.bias", training)?)?;
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
            numerical_contract: numerical_contract(),
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
        if config.schema != CHECKPOINT_SCHEMA || config.numerical_contract != numerical_contract() {
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
        Self::from_variables(config.model, variables, device)
    }
}

#[derive(Serialize, Deserialize)]
struct CheckpointConfig {
    schema: String,
    model: JointConfig,
    weights_sha256: String,
    numerical_contract: Value,
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
    let unit = active.where_cond(&normalized, &identity)?;
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
