//! Causal recurrent memory learner and quantized emulator for D8 rungs 1/2.
//!
//! This is F32 offline training scaffolding, with dense learned affine maps and
//! a differentiable soft read over all earlier events. It is not integer
//! serving, a transformer, a physical Hamiltonian, or an exact arithmetic
//! geometric kernel. Quaternion signs remain distinct. The Householder arm
//! changes only the lane transport formula and its declared local scale. The
//! optional Lorentz read geometry changes only the raw query/key read score.
//! LorentzAffine is an offline radius-preserving control of distance spacing;
//! the merged Lorentz training and integer contracts remain unchanged.

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

use crate::joint_admission::{AdmissionIndex, AdmissionPolicy, AdmissionWork};
use crate::joint_quantization::{self, QuantizationSpec};
use crate::{invalid, Result};

pub use uor_r4_integer::config::{ReadGeometry, LORENTZ_LOG_BETA, LORENTZ_OFFSET};

pub const UNIFORM_MIXTURE: f64 = 1e-8;
pub const RMS_EPSILON: f64 = 1e-5;
pub const TRANSPORT_MIN_NORM: f64 = 1e-6;
/// Lorentz products are clamped to this before arcosh: exact arithmetic gives
/// z >= 1, F32 cancellation can undershoot, and arcosh'(1) is infinite. The
/// clamp runs in F32, so the effective bound is this value rounded to F32.
pub const LORENTZ_MIN_INNER: f64 = 1.0 + 1e-6;
pub const QUATERNION_DELTA_SCALE: f64 = 0.1;
pub const CHECKPOINT_SCHEMA: &str = "uor-r4.joint-recurrent-checkpoint/1";

use uor_r4_integer::config::{valid_quantization_clock, QuantizationPreparation};
pub use uor_r4_integer::{JointConfig, ReadMode, Transport};

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
    // Alpha variables and prepared weights exist only in offline learning.
    rounding_learning: bool,
    interface_audit: Option<Arc<Mutex<BTreeMap<String, InterfaceAudit>>>>,
    admission: AdmissionPolicy,
    admission_audit: Option<Arc<Mutex<Value>>>,
}

/// Scales are calibrated once from the sealed parent and retained on resume.
/// Evaluation always uses the complete quantized path, even during the ramp.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantizedTrainingState {
    pub start_step: usize,
    pub ramp_steps: usize,
    pub completed_step: usize,
    pub spec: QuantizationSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preparation: Option<QuantizationPreparation>,
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

    /// Weighted population-mean next-token NLL: `Σ(w_t · −ln p_t) / Σ(w_t)`.
    /// Targets and weights share the `[batch × time]` order of `loss`, and the
    /// denominator is the weight sum, so weighting a target cannot rescale the
    /// objective by the target count. The optional training objective and its
    /// reported `batch_mean_nll` use this value. Ordinary `loss` and development
    /// evaluation remain unweighted; callers that shard weights must account
    /// for their local denominators when combining results.
    pub fn weighted_loss(&self, targets: &[u32], weights: &[f32]) -> Result<Tensor> {
        let (batch, time, vocab) = self.probabilities.dims3()?;
        if targets.len() != batch * time
            || weights.len() != targets.len()
            || targets.iter().any(|&id| id as usize >= vocab)
            || weights
                .iter()
                .any(|weight| !weight.is_finite() || *weight < 0.0)
        {
            return Err(invalid(
                "joint weighted next-token target/weight shape, vocabulary or weight validity",
            ));
        }
        let weight_sum: f32 = weights.iter().sum();
        if !(weight_sum > 0.0) || !weight_sum.is_finite() {
            return Err(invalid(
                "joint weighted next-token loss requires a positive finite weight sum",
            ));
        }
        let device = self.probabilities.device();
        let targets = Tensor::from_vec(targets.to_vec(), (batch * time, 1), device)?;
        let weights = Tensor::from_vec(weights.to_vec(), (batch * time, 1), device)?;
        let weighted_sum = self
            .probabilities
            .reshape((batch * time, vocab))?
            .gather(&targets, 1)?
            .log()?
            .mul(&weights)?
            .sum_all()?
            .neg()?;
        Ok(weighted_sum.affine(1.0 / f64::from(weight_sum), 0.0)?)
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
    indexes: Vec<AdmissionIndex>,
    key_events: Vec<Tensor>,
    value_events: Vec<Tensor>,
    dense_batch_history: bool,
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
    /// Column identities for the first lane; use the per-lane mapping for batches.
    pub read_occurrences: Vec<usize>,
    pub read_occurrences_by_lane: Vec<Vec<usize>>,
    pub written_occurrence: usize,
}

struct CoreStep {
    occurrences: Vec<Vec<usize>>,
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
            } else if name == LORENTZ_OFFSET {
                values.fill(lorentz_initial_offset(&config) as f32);
            } else if name == LORENTZ_LOG_BETA {
                values.fill(lorentz_initial_log_beta(&config) as f32);
            }
            variables.insert(name, Var::from_vec(values, shape.as_slice(), device)?);
        }
        Self::from_variables(config, variables, device)
    }

    /// Import only the retained R1d continuous dialogue cell. This does not
    /// relax canonical checkpoint, training-campaign or integer loader widths.
    pub(crate) fn from_offline_dialogue_parameters(
        config: JointConfig,
        parameters: BTreeMap<String, Vec<f32>>,
        device: &Device,
    ) -> Result<Self> {
        crate::dialogue_artifact::validate_model_config(&config)?;
        let shapes = config.shapes();
        if parameters.keys().ne(shapes.keys()) {
            return Err(invalid("dialogue parameter inventory mismatch"));
        }
        for (name, shape) in &shapes {
            let values = parameters
                .get(name)
                .ok_or_else(|| invalid("missing dialogue parameter"))?;
            if values.len() != shape.iter().product::<usize>()
                || values.iter().any(|value| !value.is_finite())
            {
                return Err(invalid(format!("dialogue parameter shape/value: {name}")));
            }
        }
        let mut variables = BTreeMap::new();
        for (name, values) in parameters {
            let shape = shapes
                .get(&name)
                .ok_or_else(|| invalid("unknown dialogue parameter"))?;
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
            rounding_learning: false,
            interface_audit: None,
            admission: AdmissionPolicy::Full,
            admission_audit: None,
        })
    }

    pub fn admission_policy(&self) -> AdmissionPolicy {
        self.admission
    }

    pub fn set_admission_policy(&mut self, policy: AdmissionPolicy) -> Result<()> {
        if self.prepared_parameters.is_some() {
            return Err(invalid("admission cannot change inside a prepared graph"));
        }
        self.admission = policy;
        Ok(())
    }

    /// New offline optimizer starts from the actual packed code values. Parent
    /// clock/scales remain immutable; campaign lineage records all new updates.
    pub(crate) fn packed_training_start(&self) -> Result<Self> {
        if !self.hard_only || self.prepared_parameters.is_some() {
            return Err(invalid("bounded continuation requires a packed parent"));
        }
        let mut model = self.detached_view()?;
        model.variables = self
            .variables
            .iter()
            .map(|(n, v)| Ok((n.clone(), Var::from_tensor(&v.detach())?)))
            .collect::<Result<_>>()?;
        model.hard_only = false;
        Ok(model)
    }

    pub fn enable_admission_audit(&mut self) {
        self.admission_audit = Some(Arc::new(Mutex::new(json!({
            "queries":0u64,"causally_available":0u64,"scored_candidates":0u64,
            "max_scored_candidates":0u64,"older_than_recent32":0u64,
            "work":{},"insert_work":{},"max_index_logical_bytes_per_lane":0u64,"posting_evictions":0u64,"policy":self.admission.name(),
            "scope":"Actual index query and selected ranking. Offline training may concatenate histories and expand masses; bounded incremental read/value/copy uses selected event tensors. Dense recurrent/output parameter access, host copies, allocations and fixed context ceiling remain."
        }))));
    }

    pub fn admission_audit(&self) -> Result<Value> {
        match &self.admission_audit {
            Some(a) => Ok(a
                .lock()
                .map_err(|_| invalid("admission audit lock"))?
                .clone()),
            None => Ok(Value::Null),
        }
    }

    fn record_admission(
        &self,
        previous: usize,
        occurrences: &[Vec<usize>],
        work: &[AdmissionWork],
    ) -> Result<()> {
        if let Some(a) = &self.admission_audit {
            let mut a = a.lock().map_err(|_| invalid("admission audit lock"))?;
            a["queries"] = json!(a["queries"].as_u64().unwrap_or(0) + occurrences.len() as u64);
            a["causally_available"] = json!(
                a["causally_available"].as_u64().unwrap_or(0)
                    + (previous * occurrences.len()) as u64
            );
            for row in occurrences {
                a["scored_candidates"] =
                    json!(a["scored_candidates"].as_u64().unwrap_or(0) + row.len() as u64);
                a["max_scored_candidates"] = json!(a["max_scored_candidates"]
                    .as_u64()
                    .unwrap_or(0)
                    .max(row.len() as u64));
                a["older_than_recent32"] = json!(
                    a["older_than_recent32"].as_u64().unwrap_or(0)
                        + row.iter().filter(|&&i| previous - i > 32).count() as u64
                );
            }
            for row in work {
                if let Some(fields) = serde_json::to_value(row)?.as_object() {
                    for (key, value) in fields {
                        if let Some(count) = value.as_u64() {
                            a["work"][key] = json!(a["work"][key].as_u64().unwrap_or(0) + count);
                        }
                    }
                }
            }
        }
        Ok(())
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
        require_quantized_read(self.config.read_geometry)?;
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

    /// Offline full-trajectory learning with shared alpha variables. All five
    /// interfaces keep their existing full-strength quantizers and surrogates.
    /// This view cannot be evaluated, serialized or used for generation.
    pub(crate) fn rounding_learning_view(
        &self,
        variables: BTreeMap<String, Var>,
        parameters: BTreeMap<String, Tensor>,
    ) -> Result<Self> {
        self.validate_rounding_parent()?;
        self.validate_parameter_tensors(&parameters)?;
        self.quantization
            .as_ref()
            .ok_or_else(|| invalid("missing quantizers"))?
            .spec
            .validate(&variables)?;
        if variables
            .values()
            .any(|v| !v.device().same_device(&self.device))
        {
            return Err(invalid("rounding alpha device differs"));
        }
        let mut model = self.detached_view()?;
        model.variables = variables;
        model.prepared_parameters = Some(parameters);
        model.rounding_learning = true;
        Ok(model)
    }

    /// Independent legal dyadic values for the unchanged hard codec. Parent
    /// model/quantizer clocks remain; calibration owns its separate update clock.
    pub(crate) fn materialize_rounding_codes(
        &self,
        parameters: BTreeMap<String, Tensor>,
    ) -> Result<Self> {
        self.validate_rounding_parent()?;
        self.validate_parameter_tensors(&parameters)?;
        let state = self
            .quantization
            .as_ref()
            .ok_or_else(|| invalid("missing quantizers"))?;
        let mut variables = BTreeMap::new();
        for (name, value) in parameters {
            let quantized = state.spec.parameter(&name, &value, 1.0, false)?;
            if value
                .sub(&quantized)?
                .abs()?
                .max_all()?
                .to_scalar::<f32>()?
                != 0.0
            {
                return Err(invalid(format!(
                    "nonfinite or off-grid rounding result {name}"
                )));
            }
            variables.insert(name, Var::from_tensor(&value.detach())?);
        }
        let mut model = Self::from_variables(self.config.clone(), variables, &self.device)?;
        model.quantization = self.quantization.clone();
        model.admission = self.admission;
        Ok(model)
    }

    fn validate_rounding_parent(&self) -> Result<()> {
        require_quantized_read(self.config.read_geometry)?;
        if self.hard_only
            || self.prepared_parameters.is_some()
            || self.precision_mode.is_some()
            || self.rounding_learning
        {
            return Err(invalid("rounding requires an unprepared floating parent"));
        }
        let state = self
            .quantization
            .as_ref()
            .ok_or_else(|| invalid("rounding requires frozen grids"))?;
        if !valid_quantization_clock(
            state.start_step,
            state.ramp_steps,
            state.completed_step,
            state.preparation,
        ) || (state.preparation.is_none()
            && state
                .completed_step
                .checked_sub(state.start_step)
                .is_none_or(|steps| steps < state.ramp_steps))
        {
            return Err(invalid("rounding requires a completed quantization ramp"));
        }
        state.spec.validate(&self.variables)
    }

    fn validate_parameter_tensors(&self, parameters: &BTreeMap<String, Tensor>) -> Result<()> {
        if !parameters.keys().eq(self.variables.keys()) {
            return Err(invalid("rounding parameter inventory differs"));
        }
        for (name, tensor) in parameters {
            if tensor.dims() != self.variables[name].dims()
                || tensor.dtype() != DType::F32
                || !tensor.device().same_device(&self.device)
            {
                return Err(invalid(format!(
                    "rounding tensor binding differs for {name}"
                )));
            }
        }
        Ok(())
    }

    pub fn configure_quantization(&mut self, start_step: usize, ramp_steps: usize) -> Result<()> {
        require_quantized_read(self.config.read_geometry)?;
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
            preparation: None,
        });
        Ok(())
    }

    /// Fresh parameter-only calibration for alpha learning, with no model step.
    /// The one-step ramp field is unused compatibility metadata in this mode.
    pub(crate) fn configure_rounding_calibration(&mut self, step: usize) -> Result<()> {
        self.configure_quantization(step, 1)?;
        let state = self
            .quantization
            .as_mut()
            .ok_or_else(|| invalid("missing fresh quantization state"))?;
        state.preparation = Some(QuantizationPreparation::CalibratedForRounding);
        Ok(())
    }

    pub(crate) fn is_rounding_calibrated(&self) -> bool {
        self.quantization
            .as_ref()
            .is_some_and(|state| state.preparation.is_some())
    }

    pub fn set_completed_step(&mut self, step: usize) -> Result<()> {
        if self.prepared_parameters.is_some() || self.hard_only || self.precision_mode.is_some() {
            return Err(invalid(
                "cannot update a prepared, hard-only or precision-view model",
            ));
        }
        if let Some(state) = &mut self.quantization {
            if state.preparation.is_some() && step != state.completed_step {
                return Err(invalid(
                    "calibrated rounding parent cannot advance the model clock",
                ));
            }
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
        model.rounding_learning = self.rounding_learning;
        model.interface_audit = self.interface_audit.clone();
        model.admission = self.admission;
        model.admission_audit = self.admission_audit.clone();
        Ok(model)
    }

    pub fn without_quantization(&self) -> Result<Self> {
        if self.rounding_learning {
            return Err(invalid(
                "rounding learning views cannot change numerical policy",
            ));
        }
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
        if self.quantization.is_some() {
            require_quantized_read(self.config.read_geometry)?;
        }
        if self.rounding_learning {
            return Err(invalid(
                "rounding tensors must stay in their prepared learning graph",
            ));
        }
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
            indexes: (0..batch).map(|_| AdmissionIndex::new()).collect(),
            key_events: Vec::new(),
            value_events: Vec::new(),
            dense_batch_history: false,
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
        if self.quantization.is_some() {
            require_quantized_read(self.config.read_geometry)?;
        }
        if training && self.is_rounding_calibrated() && !self.rounding_learning {
            return Err(invalid(
                "calibrated shadow permits alpha-only learning, not model training",
            ));
        }
        if self.rounding_learning && !training {
            return Err(invalid(
                "rounding learning views require the training graph",
            ));
        }
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
        memory.dense_batch_history = true;
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
            let dense_masses = if self.admission == AdmissionPolicy::Full {
                step.read_masses
            } else {
                self.expand_read_masses(&step.read_masses, &step.occurrences, position, batch)?
            };
            let padded = if position == 0 {
                Tensor::zeros((batch, time), DType::F32, &self.device)?
            } else {
                Tensor::cat(
                    &[
                        &dense_masses,
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
        if self.rounding_learning {
            return Err(invalid("rounding learning views cannot generate"));
        }
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
        if session.prepared.admission != self.admission
            || session.config != self.config
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
        let copy = if self.admission == AdmissionPolicy::Full {
            self.incremental_copy(
                &core.read_masses,
                &session.memory.events[..occurrence],
                session.batch,
            )?
        } else {
            self.selected_copy(
                &core.read_masses,
                &core.occurrences,
                &session.memory.events,
                session.batch,
            )?
        };
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
            read_occurrences: core.occurrences.first().cloned().unwrap_or_default(),
            read_occurrences_by_lane: core.occurrences,
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
        let mut occurrences = vec![Vec::new(); batch];
        let mut admission_work = Vec::new();
        let (no_read_mass, read_masses, read) = if previous == 0 || mode == ReadMode::NoRead {
            (
                Tensor::ones((batch, 1), DType::F32, &self.device)?,
                Tensor::zeros(
                    (
                        batch,
                        if self.admission == AdmissionPolicy::Full {
                            previous
                        } else {
                            0
                        },
                    ),
                    DType::F32,
                    &self.device,
                )?,
                Tensor::zeros((batch, d), DType::F32, &self.device)?,
            )
        } else if self.admission != AdmissionPolicy::Full {
            let query = self.linear(&normalized, "read.query", training)?;
            let observed_queries = query.detach().to_vec2::<f32>()?;
            for lane in 0..batch {
                let selected = memory.indexes[lane].query(
                    self.admission,
                    &observed_queries[lane],
                    input[lane],
                )?;
                occurrences[lane] = selected.occurrences;
                admission_work.push(selected.work);
            }
            self.selected_read(memory, &query, &normalized, &occurrences, training)?
        } else {
            occurrences = vec![(0..previous).collect(); batch];
            let query = self.linear(&normalized, "read.query", training)?;
            let keys = memory
                .keys
                .as_ref()
                .ok_or_else(|| invalid("missing prior key history"))?;
            let values = memory
                .values
                .as_ref()
                .ok_or_else(|| invalid("missing prior value history"))?;
            let scores = self.read_scores(&query, keys, training)?;
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
        if self.admission == AdmissionPolicy::Full && mode == ReadMode::NoRead {
            occurrences = vec![(0..previous).collect(); batch];
        }
        if mode == ReadMode::Enabled {
            self.record_admission(previous, &occurrences, &admission_work)?;
        }
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
        if self.admission == AdmissionPolicy::Full || memory.dense_batch_history {
            memory.keys = Some(append_history(memory.keys.as_ref(), &key, training)?);
            memory.values = Some(append_history(memory.values.as_ref(), &value, training)?);
        }
        if self.admission != AdmissionPolicy::Full {
            let key_rows = key.detach().to_vec2::<f32>()?;
            for lane in 0..batch {
                let before = memory.indexes[lane].insert_work();
                let old_evictions = memory.indexes[lane].footprint().posting_evictions;
                memory.indexes[lane].insert(&key_rows[lane], input[lane])?;
                if let Some(audit) = &self.admission_audit {
                    let mut audit = audit.lock().map_err(|_| invalid("admission audit lock"))?;
                    let after = serde_json::to_value(memory.indexes[lane].insert_work())?;
                    let before = serde_json::to_value(before)?;
                    if let Some(fields) = after.as_object() {
                        for (name, value) in fields {
                            let delta =
                                value.as_u64().unwrap_or(0) - before[name].as_u64().unwrap_or(0);
                            audit["insert_work"][name] =
                                json!(audit["insert_work"][name].as_u64().unwrap_or(0) + delta);
                        }
                    }
                    let footprint = memory.indexes[lane].footprint();
                    audit["posting_evictions"] = json!(
                        audit["posting_evictions"].as_u64().unwrap_or(0)
                            + (footprint.posting_evictions - old_evictions) as u64
                    );
                    audit["max_index_logical_bytes_per_lane"] = json!(audit
                        ["max_index_logical_bytes_per_lane"]
                        .as_u64()
                        .unwrap_or(0)
                        .max(footprint.logical_bytes as u64));
                }
            }
            if !memory.dense_batch_history {
                memory.key_events.push(key.detach());
                memory.value_events.push(value.detach());
            }
        }
        memory.state = if training {
            state.clone()
        } else {
            state.detach()
        };
        memory.events.push(MemoryEvent {
            occurrence: previous,
            tokens: input.to_vec(),
        });
        Ok(CoreStep {
            occurrences,
            state,
            no_read_mass,
            read_masses,
            copy_gate,
        })
    }

    /// Raw read scores [batch, candidates] before the shared learned age bias,
    /// interface, NoRead competition and value mixing. `Dot` keeps the retained
    /// operations and order; `Lorentz` is
    /// exp(read.lorentz_log_beta)*(read.lorentz_offset - arcosh(z)).
    fn read_scores(&self, query: &Tensor, keys: &Tensor, training: bool) -> Result<Tensor> {
        match self.config.read_geometry {
            ReadGeometry::Dot => Ok(query
                .unsqueeze(1)?
                .matmul(&keys.transpose(1, 2)?.contiguous()?)?
                .squeeze(1)?
                .affine(1.0 / (self.config.read_width as f64).sqrt(), 0.0)?),
            ReadGeometry::LorentzAffine => {
                let inner = lorentz_inner(query, keys)?;
                let distance = radial_distance(&inner, &self.config)?;
                let beta = self.weight(LORENTZ_LOG_BETA, training)?.exp()?;
                let offset = self.weight(LORENTZ_OFFSET, training)?;
                Ok(distance
                    .broadcast_sub(&offset)?
                    .broadcast_mul(&beta)?
                    .neg()?)
            }
            ReadGeometry::Lorentz => {
                let beta = self.weight(LORENTZ_LOG_BETA, training)?.exp()?;
                let offset = self.weight(LORENTZ_OFFSET, training)?;
                Ok(lorentz_distance(query, keys)?
                    .broadcast_sub(&offset)?
                    .broadcast_mul(&beta)?
                    .neg()?)
            }
        }
    }

    fn selected_read(
        &self,
        memory: &RecurrentState,
        query: &Tensor,
        normalized: &Tensor,
        occurrences: &[Vec<usize>],
        training: bool,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let batch = occurrences.len();
        let count = occurrences.iter().map(Vec::len).max().unwrap_or(0);
        if count == 0 || count > 64 {
            return Err(invalid("bounded read requires 1..64 candidate columns"));
        }
        let previous = memory.events.len();
        let mut masks = Vec::with_capacity(batch * count);
        let mut ages = Vec::with_capacity(batch * count);
        for row in occurrences {
            for slot in 0..count {
                if let Some(&id) = row.get(slot) {
                    if id >= previous {
                        return Err(invalid("noncausal admitted occurrence"));
                    }
                    masks.push(0f32);
                    ages.push((previous - id - 1) as u32);
                } else {
                    masks.push(f32::NEG_INFINITY);
                    ages.push(0);
                }
            }
        }
        let keys = self.selected_history(
            memory.keys.as_ref(),
            &memory.key_events,
            occurrences,
            count,
            self.config.read_width,
            memory.dense_batch_history,
        )?;
        let values = self.selected_history(
            memory.values.as_ref(),
            &memory.value_events,
            occurrences,
            count,
            self.config.width,
            memory.dense_batch_history,
        )?;
        let scores = self.read_scores(query, &keys, training)?;
        let age = self
            .weight("read.age", training)?
            .index_select(&Tensor::from_vec(ages, (batch * count,), &self.device)?, 0)?
            .reshape((batch, count))?;
        let scores = self
            .interface(&scores.add(&age)?, Interface::Affine, training)?
            .add(&Tensor::from_vec(masks, (batch, count), &self.device)?)?;
        let null = self.linear(normalized, "read.no_read", training)?;
        let masses = candle_nn::ops::softmax(&Tensor::cat(&[&null, &scores], 1)?, 1)?;
        let no_read = masses.narrow(1, 0, 1)?;
        let reads = masses.narrow(1, 1, count)?.contiguous()?;
        let read = reads.unsqueeze(1)?.matmul(&values)?.squeeze(1)?;
        Ok((
            no_read,
            reads,
            self.interface(&read, Interface::State, training)?,
        ))
    }

    /// Training gathers selected rows from a differentiable contiguous history.
    /// Incremental execution visits only selected separately stored events.
    fn selected_history(
        &self,
        history: Option<&Tensor>,
        events: &[Tensor],
        occurrences: &[Vec<usize>],
        count: usize,
        width: usize,
        training: bool,
    ) -> Result<Tensor> {
        let batch = occurrences.len();
        if training {
            let history = history.ok_or_else(|| invalid("missing bounded training history"))?;
            let indexes: Vec<u32> = occurrences
                .iter()
                .flat_map(|row| {
                    (0..count).flat_map(move |slot| {
                        std::iter::repeat_n(row.get(slot).copied().unwrap_or(0) as u32, width)
                    })
                })
                .collect();
            Ok(history.gather(
                &Tensor::from_vec(indexes, (batch, count, width), &self.device)?,
                1,
            )?)
        } else {
            let mut lanes = Vec::with_capacity(batch);
            for (lane, row) in occurrences.iter().enumerate() {
                let mut selected = Vec::with_capacity(count);
                for slot in 0..count {
                    if let Some(&id) = row.get(slot) {
                        selected.push(
                            events
                                .get(id)
                                .ok_or_else(|| invalid("missing exact event tensor"))?
                                .get(lane)?,
                        );
                    } else {
                        selected.push(Tensor::zeros((width,), DType::F32, &self.device)?);
                    }
                }
                lanes.push(Tensor::stack(&selected, 0)?);
            }
            Ok(Tensor::stack(&lanes, 0)?)
        }
    }

    /// Offline batch diagnostics/copy retain the old dense occurrence layout.
    /// This expansion is never used by the bounded incremental serving emulator.
    fn expand_read_masses(
        &self,
        masses: &Tensor,
        rows: &[Vec<usize>],
        previous: usize,
        batch: usize,
    ) -> Result<Tensor> {
        let count = masses.dim(1)?;
        if count == 0 {
            return Ok(Tensor::zeros((batch, previous), DType::F32, &self.device)?);
        }
        let mut expanded = Vec::with_capacity(batch);
        for (lane, row) in rows.iter().enumerate() {
            let indexes: Vec<u32> = (0..count)
                .map(|i| row.get(i).copied().unwrap_or(0) as u32)
                .collect();
            expanded.push(
                Tensor::zeros((previous,), DType::F32, &self.device)?.index_add(
                    &Tensor::from_vec(indexes, (count,), &self.device)?,
                    &masses.get(lane)?.contiguous()?,
                    0,
                )?,
            );
        }
        Ok(Tensor::stack(&expanded, 0)?)
    }

    fn selected_copy(
        &self,
        masses: &Tensor,
        rows: &[Vec<usize>],
        events: &[MemoryEvent],
        batch: usize,
    ) -> Result<Tensor> {
        let count = masses.dim(1)?;
        let zeros = Tensor::zeros((batch * self.config.vocab_size,), DType::F32, &self.device)?;
        if count == 0 {
            return Ok(zeros.reshape((batch, self.config.vocab_size))?);
        }
        let mut indexes = Vec::with_capacity(batch * count);
        for (lane, row) in rows.iter().enumerate() {
            for slot in 0..count {
                let token = match row.get(slot) {
                    Some(&id) => events
                        .get(id)
                        .and_then(|e| e.tokens.get(lane))
                        .copied()
                        .ok_or_else(|| invalid("missing exact copy token"))?,
                    None => 0,
                };
                indexes.push((lane * self.config.vocab_size + token as usize) as u32);
            }
        }
        Ok(zeros
            .index_add(
                &Tensor::from_vec(indexes, (batch * count,), &self.device)?,
                &masses.flatten_all()?.contiguous()?,
                0,
            )?
            .reshape((batch, self.config.vocab_size))?)
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

    /// Verbatim vocabulary softmax of `output_distribution` (read-only; no
    /// parameter, gate or sampler state changes). The emission trace rebuilds
    /// the mixture split from this row, so the block above must stay identical.
    pub(crate) fn output_vocabulary(&self, states: &Tensor) -> Result<Tensor> {
        let hidden = self
            .normalized(states, false)?
            .broadcast_mul(&self.weight("output.norm.weight", false)?)?;
        let hidden = self.interface(&hidden, Interface::Normalized, false)?;
        let logits = hidden
            .matmul(&self.weight("embedding.weight", false)?.t()?)?
            .broadcast_add(&self.weight("output.bias", false)?)?;
        let logits = self.interface(&logits, Interface::Affine, false)?;
        Ok(candle_nn::ops::softmax(&logits, 1)?)
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

    pub(crate) fn incremental_copy(
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
        if self.quantization.is_some() {
            require_quantized_read(self.config.read_geometry)?;
        }
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
            admission: self.admission,
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
        if config.quantization.is_some() {
            require_quantized_read(config.model.read_geometry)?;
        }
        let expected_contract = if config.quantization.is_some() {
            quantized_numerical_contract()
        } else {
            numerical_contract()
        };
        let expected_contract = admission_contract(
            read_geometry_contract(expected_contract, config.model.read_geometry),
            config.admission,
        );
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
            if !valid_quantization_clock(
                state.start_step,
                state.ramp_steps,
                state.completed_step,
                state.preparation,
            ) {
                return Err(invalid("invalid quantization checkpoint clock"));
            }
        }
        model.quantization = config.quantization;
        model.admission = config.admission;
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
        admission_contract(
            read_geometry_contract(contract, self.config.read_geometry),
            self.admission,
        )
    }

    /// Packed parameter export for the shared quantized F32 evaluator. This is
    /// deliberately not an integer execution kernel: the nonlinearities,
    /// accumulations, normalization and probability calculations remain F32.
    pub fn save_hard(&self, directory: &Path) -> Result<Value> {
        require_quantized_read(self.config.read_geometry)?;
        if self.prepared_parameters.is_some() || self.rounding_learning {
            return Err(invalid("materialize hard codes before export"));
        }
        if self.precision_mode.is_some() {
            return Err(invalid("a precision view cannot be exported"));
        }
        let state = self
            .quantization
            .as_ref()
            .ok_or_else(|| invalid("hard export requires frozen quantization scales"))?;
        let parameters =
            joint_quantization::save_hard_parameters(&state.spec, &self.variables, directory)?;
        let mut manifest = json!({
            "schema":"uor-r4.joint-recurrent-packed-emulator/1",
            "model":self.config,
            "quantization":state,
            "numerical_contract":admission_contract(
                read_geometry_contract(quantized_numerical_contract(), self.config.read_geometry),
                self.admission,
            ),
            "parameter_manifest":parameters,
            "parameter_manifest_sha256":crate::sha256_file(&directory.join(joint_quantization::HARD_PARAMETERS_MANIFEST_FILE))?,
            "scope":"Packed signed 4-bit multiplicative weights and signed 16-bit additive offsets; dyadic scales; quantized recurrent interfaces; F32 emulation, not D0-b integer serving"
        });
        if self.admission != AdmissionPolicy::Full {
            manifest["admission"] = json!(self.admission);
        }
        let mut file = File::create_new(directory.join("hard-model.json"))?;
        serde_json::to_writer_pretty(&mut file, &manifest)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(manifest)
    }

    pub fn load_hard(directory: &Path, device: &Device) -> Result<Self> {
        let manifest: Value =
            serde_json::from_slice(&fs::read(directory.join("hard-model.json"))?)?;
        let admission: AdmissionPolicy = manifest
            .get("admission")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?
            .unwrap_or_default();
        let config: JointConfig = serde_json::from_value(manifest["model"].clone())?;
        require_quantized_read(config.read_geometry)?;
        if manifest["schema"] != "uor-r4.joint-recurrent-packed-emulator/1"
            || manifest["numerical_contract"]
                != admission_contract(
                    read_geometry_contract(quantized_numerical_contract(), config.read_geometry),
                    admission,
                )
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
        config.validate()?;
        let state: QuantizedTrainingState =
            serde_json::from_value(manifest["quantization"].clone())?;
        let (spec, variables) = joint_quantization::load_hard_parameters(directory, device)?;
        if spec != state.spec
            || !valid_quantization_clock(
                state.start_step,
                state.ramp_steps,
                state.completed_step,
                state.preparation,
            )
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
        model.admission = admission;
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
    #[serde(default, skip_serializing_if = "admission_is_full")]
    admission: AdmissionPolicy,
}

fn require_quantized_read(geometry: ReadGeometry) -> Result<()> {
    if geometry == ReadGeometry::LorentzAffine {
        return Err(uor_r4_integer::IntegerError::UnsupportedReadGeometry(geometry).into());
    }
    Ok(())
}

fn admission_is_full(policy: &AdmissionPolicy) -> bool {
    *policy == AdmissionPolicy::Full
}

fn admission_contract(mut contract: Value, policy: AdmissionPolicy) -> Value {
    if policy != AdmissionPolicy::Full {
        contract["training_credit"] = json!("Full trajectory conditional on a stopped-gradient causal admission mask; selected query/key/value/state paths remain differentiable. No cross-bucket membership estimator.");
        if contract.get("quantization").is_some() {
            contract["quantization"]["remaining_float"] = json!("F32 recurrent/output dense maps, selected key/value scoring, normalization, nonlinearities and sampling remain. Bounded incremental admission/read/copy visit at most64 selected events; offline batch storage and diagnostic masses may be dense.");
        }
        contract["admission"] = json!({"policy":policy,"maximum_scored_candidates":64,"backward":"Hard causal index mask is stopped-gradient; selected query/key/value/state/output retain full-trajectory language credit. No estimator of cross-bucket membership credit.","scope":"Selected retrieval/ranking/value/copy in the shared recurrent graph; full-context and exact-cache controls. Not integer execution, unbounded context, parameter sparsity or measured energy."});
    }
    contract
}

/// Declare a Lorentz read in the bound contract. `Dot` returns the retained
/// contract unchanged, so existing checkpoints keep their exact identity. A
/// quantized contract takes the integer runtime's quantized Lorentz declaration,
/// so quantized training, packed export and integer serving bind one contract.
fn read_geometry_contract(mut contract: Value, geometry: ReadGeometry) -> Value {
    if geometry == ReadGeometry::LorentzAffine {
        contract["read"] = json!("Query/Key from RMS-normalized provisional/written states; common Lorentz lift x0=sqrt(1+|x|^2), z=max(q0*k0-<q,k>,1+1e-6); score=exp(read.lorentz_log_beta)*(read.lorentz_offset-distance(z))+learned age, competing with unchanged learned NoRead; Value=tanh(affine(normalized written state))");
        contract["read_geometry"] = json!({
            "schema":"uor-r4.offline-radial-read/1",
            "geometry":geometry,
            "minimum_inner_product":LORENTZ_MIN_INNER,
            "distance":"d_ref+(z-cosh(d_ref))/sinh(d_ref); fixed d_ref=arcosh(1+2*r*d/(r+d)), independent of learned offset",
            "initial_offset":"arcosh(1+2*r*d/(r+d))",
            "initial_log_beta":"ln(sinh(initial_offset)/sqrt(r))",
            "scalars":"Both radial arms learn read.lorentz_log_beta and read.lorentz_offset; their initialization consumes no random draws",
            "admission":"Unchanged; this operator scores only the candidates already admitted",
            "scope":"F32 offline training/evaluation only; quantization, packed export and integer serving are rejected"
        });
        return contract;
    }
    if geometry == ReadGeometry::Lorentz && contract.get("quantization").is_some() {
        return uor_r4_integer::config::with_lorentz_quantized_read(contract);
    }
    if geometry == ReadGeometry::Lorentz {
        contract["read"] = json!("Query/Key from RMS-normalized provisional/written states, each lifted to the hyperboloid x0=sqrt(1+|x|^2); Value=tanh(affine(normalized written state)); score=exp(read.lorentz_log_beta)*(read.lorentz_offset-arcosh(max(q0*k0-<q,k>,1+1e-6)))+learned age, competing with learned NoRead");
        contract["read_geometry"] = json!({
            "geometry":geometry,
            "minimum_inner_product":LORENTZ_MIN_INNER,
            "distance":"arcosh(z)=ln(z+sqrt((z-1)(z+1))) after the F32 clamp",
            "scale":"exp(read.lorentz_log_beta); learned scalar initialized ln(sinh(offset0)/sqrt(r)), matching the Dot read scale to first order",
            "offset":"read.lorentz_offset; learned scalar radius initialized arcosh(1+2*r*d/(r+d)), the distance between independent initial query/key rows",
            "admission":"Bounded candidate admission is unchanged; this score ranks admitted candidates only",
            "scope":"F32 offline training and evaluation only; no quantized interface, packed export or integer arcosh kernel"
        });
    }
    contract
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

/// Initial Lorentz read radius: the geodesic distance between independent
/// initial query and key rows. RMS-normalized inputs have unit mean square and
/// a Glorot-uniform [r, d] map gives each coordinate variance 2d/(r+d), so
/// E|q|^2 = 2rd/(r+d), and orthogonal rows of that norm have z = 1 + E|q|^2.
/// Initial Lorentz scores then start near zero, as the Dot arm's do, instead of
/// ceding almost all read mass to NoRead (all distances start near this value).
pub fn lorentz_initial_offset(config: &JointConfig) -> f64 {
    let (d, r) = (config.width as f64, config.read_width as f64);
    (1.0 + 2.0 * r * d / (r + d)).acosh()
}

/// Initial Lorentz log scale, matching the Dot read to first order. Near the
/// initial z0 = cosh(offset), arcosh(z) ~ offset - <q,k>/sinh(offset) for
/// fixed norms, so beta0 = sinh(offset)/sqrt(r) makes the initial score
/// approximately <q,k>/sqrt(r) plus a per-key radius term. With beta 1 the
/// candidate scores started about ten times flatter than Dot's.
pub fn lorentz_initial_log_beta(config: &JointConfig) -> f64 {
    let r = config.read_width as f64;
    (lorentz_initial_offset(config).sinh() / r.sqrt()).ln()
}

/// Shared lift of queries [batch, width] and keys [batch, count, width]:
/// x -> (sqrt(1+|x|^2), x), with z = q0*k0 - <q,k> clamped to
/// LORENTZ_MIN_INNER. Factoring this out preserves the Lorentz operation order.
fn lorentz_inner(query: &Tensor, keys: &Tensor) -> Result<Tensor> {
    let query_time = query.sqr()?.sum_keepdim(1)?.affine(1.0, 1.0)?.sqrt()?;
    let key_time = keys
        .sqr()?
        .sum_keepdim(2)?
        .affine(1.0, 1.0)?
        .sqrt()?
        .squeeze(2)?;
    let spatial = query
        .unsqueeze(1)?
        .matmul(&keys.transpose(1, 2)?.contiguous()?)?
        .squeeze(1)?;
    Ok(key_time
        .broadcast_mul(&query_time)?
        .sub(&spatial)?
        .clamp(LORENTZ_MIN_INNER as f32, f32::MAX)?)
}

fn lorentz_distance(query: &Tensor, keys: &Tensor) -> Result<Tensor> {
    lorentz_distance_from_inner(&lorentz_inner(query, keys)?)
}

fn lorentz_distance_from_inner(z: &Tensor) -> Result<Tensor> {
    // Retained acosh expression: (z-1)(z+1) preserves precision near z=1.
    let root = z.affine(1.0, -1.0)?.mul(&z.affine(1.0, 1.0)?)?.sqrt()?;
    Ok(z.add(&root)?.log()?)
}

fn radial_distance(inner: &Tensor, config: &JointConfig) -> Result<Tensor> {
    match config.read_geometry {
        ReadGeometry::Lorentz => lorentz_distance_from_inner(inner),
        ReadGeometry::LorentzAffine => {
            let reference = lorentz_initial_offset(config);
            // Fixed reference: the learned offset remains the same additive
            // competition with NoRead in both radial arms.
            Ok(inner
                .affine(1.0, -reference.cosh())?
                .affine(1.0 / reference.sinh(), reference)?)
        }
        ReadGeometry::Dot => Err(invalid("Dot read does not use a radial distance")),
    }
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

    fn radial_config(geometry: ReadGeometry) -> JointConfig {
        JointConfig {
            width: 128,
            seed: 7,
            read_geometry: geometry,
            ..JointConfig::default()
        }
    }

    #[test]
    fn radial_read_preserves_dot_identity_and_shared_initial_arrays() -> Result<()> {
        let legacy = r#"{"vocab_size":4096,"width":256,"read_width":64,"context":256,"transport":"quaternion","seed":0}"#;
        let config: JointConfig = serde_json::from_str(legacy)?;
        assert_eq!(serde_json::to_string(&config)?, legacy);
        assert_eq!(config.read_geometry, ReadGeometry::Dot);
        let dot = JointModel::new(radial_config(ReadGeometry::Dot), &Device::Cpu)?;
        assert_eq!(dot.numerical_contract(), numerical_contract());
        let query = Tensor::from_vec(vec![1f32, -2., 0.5, 3.], (2, 2), &Device::Cpu)?;
        let keys = Tensor::from_vec(
            vec![1f32, 2., -1., 0., 3., 1., 0., 2.],
            (2, 2, 2),
            &Device::Cpu,
        )?;
        let retained = query
            .unsqueeze(1)?
            .matmul(&keys.transpose(1, 2)?.contiguous()?)?
            .squeeze(1)?
            .affine(1.0 / 8.0, 0.0)?;
        assert!(same_bits(
            &retained,
            &dot.read_scores(&query, &keys, false)?
        )?);
        for geometry in [ReadGeometry::Lorentz, ReadGeometry::LorentzAffine] {
            let model = JointModel::new(radial_config(geometry), &Device::Cpu)?;
            assert_eq!(model.variables().len(), dot.variables().len() + 2);
            for (name, variable) in dot.variables() {
                assert!(
                    same_bits(variable, &model.variables()[name])?,
                    "{geometry:?} {name}"
                );
            }
            assert_eq!(
                model.variables()[LORENTZ_OFFSET].to_vec1::<f32>()?,
                [lorentz_initial_offset(&model.config) as f32]
            );
            assert_eq!(
                model.variables()[LORENTZ_LOG_BETA].to_vec1::<f32>()?,
                [lorentz_initial_log_beta(&model.config) as f32]
            );
            let ids = [3, 17, 4, 9, 2];
            assert!(same_bits(
                &model
                    .forward(&ids, 1, 5, ReadMode::NoRead, false)?
                    .probabilities,
                &dot.forward(&ids, 1, 5, ReadMode::NoRead, false)?
                    .probabilities,
            )?);
        }
        Ok(())
    }

    #[test]
    fn radial_read_affine_control_keeps_radii_and_matches_reference_tangent() -> Result<()> {
        let device = Device::Cpu;
        let query = Tensor::from_vec(vec![1f32, 0.], (1, 2), &device)?;
        // Equal spatial dots, different key radii: neither arm may discard
        // their different lifted time coordinates.
        let keys = Tensor::from_vec(vec![0f32, 1., 0., 2.], (1, 2, 2), &device)?;
        let inner = lorentz_inner(&query, &keys)?;
        let expected_inner = [2f64, 10f64.sqrt()];
        for (&actual, expected) in inner.to_vec2::<f32>()?[0].iter().zip(expected_inner) {
            assert!((f64::from(actual) - expected).abs() < 1e-6);
        }
        let mut tangent_values = Vec::new();
        for geometry in [ReadGeometry::Lorentz, ReadGeometry::LorentzAffine] {
            let model = JointModel::new(radial_config(geometry), &device)?;
            model.variables()[LORENTZ_LOG_BETA].set(&Tensor::from_vec(
                vec![0.5f32],
                (1,),
                &device,
            )?)?;
            model.variables()[LORENTZ_OFFSET].set(&Tensor::from_vec(
                vec![1.25f32],
                (1,),
                &device,
            )?)?;
            let reference = lorentz_initial_offset(&model.config);
            let scores = model.read_scores(&query, &keys, false)?.to_vec2::<f32>()?;
            assert!(scores[0][0] > scores[0][1]);
            for (&score, z) in scores[0].iter().zip(expected_inner) {
                let distance = if geometry == ReadGeometry::Lorentz {
                    z.acosh()
                } else {
                    reference + (z - reference.cosh()) / reference.sinh()
                };
                let expected = 0.5f64.exp() * (1.25 - distance);
                assert!((f64::from(score) - expected).abs() < 2e-6 * (1.0 + expected.abs()));
            }
            let z = Var::from_vec(vec![reference.cosh() as f32], (1, 1), &device)?;
            let distance = radial_distance(&z, &model.config)?.sum_all()?;
            let gradients = distance.backward()?;
            let slope = gradients
                .get(z.as_tensor())
                .ok_or_else(|| invalid("missing tangent gradient"))?
                .to_vec2::<f32>()?[0][0];
            assert!((f64::from(slope) - 1.0 / reference.sinh()).abs() < 1e-7);
            tangent_values.push(distance.to_scalar::<f32>()?);
            let epsilon = 0.125f32;
            let evaluate = |delta: f32| -> Result<f32> {
                Ok(radial_distance(
                    &Tensor::from_vec(vec![reference.cosh() as f32 + delta], (1, 1), &device)?,
                    &model.config,
                )?
                .sum_all()?
                .to_scalar::<f32>()?)
            };
            let numerical_slope = (evaluate(epsilon)? - evaluate(-epsilon)?) / (2.0 * epsilon);
            assert!((slope - numerical_slope).abs() < 3e-6);
        }
        assert!((tangent_values[0] - tangent_values[1]).abs() < 1e-6);
        let identical = query.unsqueeze(1)?;
        assert_eq!(
            lorentz_inner(&query, &identical)?.to_vec2::<f32>()?[0][0],
            LORENTZ_MIN_INNER as f32
        );
        Ok(())
    }

    #[test]
    fn radial_read_language_gradients_reach_both_shared_read_paths() -> Result<()> {
        let ids = [3, 17, 4, 9, 2];
        let targets = [17, 4, 9, 2, 5];
        for geometry in [ReadGeometry::Lorentz, ReadGeometry::LorentzAffine] {
            let mut model = JointModel::new(radial_config(geometry), &Device::Cpu)?;
            for admission in [AdmissionPolicy::Full, AdmissionPolicy::Recent64] {
                model.set_admission_policy(admission)?;
                let output = model.forward(&ids, 1, 5, ReadMode::Enabled, true)?;
                let loss = output.loss(&targets)?;
                assert!(loss.to_scalar::<f32>()?.is_finite());
                let gradients = loss.backward()?;
                for name in [
                    "read.query.weight",
                    "read.key.weight",
                    "read.value.weight",
                    "read.no_read.weight",
                    LORENTZ_LOG_BETA,
                    LORENTZ_OFFSET,
                ] {
                    let gradient = gradients
                        .get(model.variables()[name].as_tensor())
                        .ok_or_else(|| invalid(format!("missing {geometry:?} {name} gradient")))?;
                    let values = gradient.flatten_all()?.to_vec1::<f32>()?;
                    assert!(
                        values.iter().all(|value| value.is_finite()),
                        "{geometry:?} {admission:?} {name}"
                    );
                    assert!(
                        values.iter().any(|&value| value != 0.0),
                        "{geometry:?} {admission:?} {name}"
                    );
                }
                let changed_future =
                    model.forward(&[3, 17, 4, 9, 88], 1, 5, ReadMode::Enabled, false)?;
                assert!(
                    max_delta(
                        &output.probabilities.narrow(1, 0, 4)?,
                        &changed_future.probabilities.narrow(1, 0, 4)?
                    )? < 2e-6
                );
                let mut session = model.new_session(1)?;
                for (position, &token) in ids.iter().enumerate() {
                    let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
                    assert!(
                        max_delta(
                            &step.probabilities,
                            &output.probabilities.narrow(1, position, 1)?.squeeze(1)?
                        )? < 2e-6
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn radial_read_checkpoint_identity_and_integer_boundaries() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock"))?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-r4-radial-read-{}-{nonce}", std::process::id()));
        fs::create_dir(&root)?;
        for geometry in [ReadGeometry::Lorentz, ReadGeometry::LorentzAffine] {
            let directory = root.join(geometry.name());
            fs::create_dir(&directory)?;
            let mut model = JointModel::new(radial_config(geometry), &Device::Cpu)?;
            let unsupported = |result: Result<()>| {
                assert!(
                    matches!(result, Err(crate::TrainingError::Integer(uor_r4_integer::IntegerError::UnsupportedReadGeometry(actual))) if actual == geometry)
                );
            };
            if geometry == ReadGeometry::LorentzAffine {
                unsupported(model.configure_quantization(0, 1));
                unsupported(model.configure_rounding_calibration(0));
                unsupported(model.save_hard(&directory).map(|_| ()));
                unsupported(model.precision_view(PrecisionMode::FF).map(|_| ()));
                assert!(matches!(
                    uor_r4_integer::config::packed_numerical_contract(geometry),
                    Err(uor_r4_integer::IntegerError::UnsupportedReadGeometry(actual)) if actual == geometry
                ));
            } else {
                let mut quantized = JointModel::new(radial_config(geometry), &Device::Cpu)?;
                quantized.configure_quantization(0, 1)?;
                let written: Value =
                    serde_json::from_str(&serde_json::to_string(&quantized.numerical_contract())?)?;
                assert_eq!(
                    written,
                    uor_r4_integer::config::packed_numerical_contract(geometry)?
                );
            }
            assert!(fs::read_dir(&directory)?.next().is_none());
            model.save(&directory)?;
            if geometry == ReadGeometry::Lorentz {
                // Source-derived compatibility expectation from dcf979df's
                // continuous read_geometry_contract (lines 1721–1730), not a
                // replay of any trained artifact. Keep this literal separate
                // from the implementation so contract edits cannot pass merely
                // by writing and reloading the same incompatible declaration.
                let mut pinned = numerical_contract();
                pinned["read"] = json!("Query/Key from RMS-normalized provisional/written states, each lifted to the hyperboloid x0=sqrt(1+|x|^2); Value=tanh(affine(normalized written state)); score=exp(read.lorentz_log_beta)*(read.lorentz_offset-arcosh(max(q0*k0-<q,k>,1+1e-6)))+learned age, competing with learned NoRead");
                pinned["read_geometry"] = serde_json::from_str(
                    r#"{
                    "geometry":"lorentz",
                    "minimum_inner_product":1.000001,
                    "distance":"arcosh(z)=ln(z+sqrt((z-1)(z+1))) after the F32 clamp",
                    "scale":"exp(read.lorentz_log_beta); learned scalar initialized ln(sinh(offset0)/sqrt(r)), matching the Dot read scale to first order",
                    "offset":"read.lorentz_offset; learned scalar radius initialized arcosh(1+2*r*d/(r+d)), the distance between independent initial query/key rows",
                    "admission":"Bounded candidate admission is unchanged; this score ranks admitted candidates only",
                    "scope":"F32 offline training and evaluation only; no quantized interface, packed export or integer arcosh kernel"
                }"#,
                )?;
                assert_eq!(model.numerical_contract(), pinned);
                let saved: Value =
                    serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
                assert_eq!(saved["numerical_contract"], pinned);
            }
            let reloaded = JointModel::load(&directory, &Device::Cpu)?;
            assert_eq!(reloaded.config, model.config);
            assert_eq!(
                reloaded.numerical_contract()["read_geometry"]["geometry"],
                json!(geometry)
            );
            let ids = [3, 17, 4, 9, 2];
            assert!(same_bits(
                &model
                    .forward(&ids, 1, 5, ReadMode::Enabled, false)?
                    .probabilities,
                &reloaded
                    .forward(&ids, 1, 5, ReadMode::Enabled, false)?
                    .probabilities
            )?);
            if geometry == ReadGeometry::LorentzAffine {
                let manifest = json!({"model":model.config});
                fs::write(
                    directory.join("hard-model.json"),
                    serde_json::to_vec(&manifest)?,
                )?;
                unsupported(JointModel::load_hard(&directory, &Device::Cpu).map(|_| ()));
                assert!(
                    matches!(uor_r4_integer::IntegerModel::load_with_tables(&directory, &directory), Err(uor_r4_integer::IntegerError::UnsupportedReadGeometry(actual)) if actual == geometry)
                );
            }
            let mut manifest: Value =
                serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
            manifest["model"]["read_geometry"] = json!(if geometry == ReadGeometry::Lorentz {
                ReadGeometry::LorentzAffine
            } else {
                ReadGeometry::Lorentz
            });
            fs::write(
                directory.join("config.json"),
                serde_json::to_vec(&manifest)?,
            )?;
            assert!(JointModel::load(&directory, &Device::Cpu).is_err());
            // Even a caller mutating the public config of a quantized Dot
            // shadow cannot obtain an affine quantized evaluation or export.
            if geometry == ReadGeometry::LorentzAffine {
                let mut quantized =
                    JointModel::new(radial_config(ReadGeometry::Dot), &Device::Cpu)?;
                quantized.configure_quantization(0, 1)?;
                quantized.config.read_geometry = geometry;
                unsupported(
                    quantized
                        .forward(&ids, 1, 5, ReadMode::Enabled, false)
                        .map(|_| ()),
                );
                unsupported(quantized.new_session(1).map(|_| ()));
                unsupported(quantized.save(&directory));
                manifest["model"]["read_geometry"] = json!(geometry);
                manifest["quantization"] = serde_json::to_value(quantized.quantization())?;
                fs::write(
                    directory.join("config.json"),
                    serde_json::to_vec(&manifest)?,
                )?;
                unsupported(JointModel::load(&directory, &Device::Cpu).map(|_| ()));
            }
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn small(transport: Transport) -> JointConfig {
        JointConfig {
            width: 128,
            context: 8,
            transport,
            seed: 7,
            ..JointConfig::default()
        }
    }

    #[test]
    fn bounded_shared_graph_session_causality_and_gradients() -> Result<()> {
        for transport in [Transport::Quaternion, Transport::HouseholderPair] {
            let mut config = small(transport);
            config.context = 96;
            let base = JointModel::new(config, &Device::Cpu)?;
            let ids: Vec<u32> = (0..192)
                .map(|i| ((i * 17 + i / 7) % 4095 + 1) as u32)
                .collect();
            for policy in [
                AdmissionPolicy::Recent64,
                AdmissionPolicy::Orthant64,
                AdmissionPolicy::ExactCache64,
            ] {
                let mut model = base.detached_view()?;
                model.set_admission_policy(policy)?;
                let full = model.forward(&ids, 2, 96, ReadMode::Enabled, false)?;
                let mut session = model.new_session(2)?;
                for position in 0..96 {
                    let step = model.step(
                        &mut session,
                        &[ids[position], ids[96 + position]],
                        ReadMode::Enabled,
                    )?;
                    let expected = full.probabilities.narrow(1, position, 1)?.squeeze(1)?;
                    assert!(
                        max_delta(&expected, &step.probabilities)? < 2e-6,
                        "{policy:?} at {position}"
                    );
                    assert!(step
                        .read_occurrences_by_lane
                        .iter()
                        .all(|row| row.len() <= 64 && row.iter().all(|&id| id < position)));
                }
                assert!(
                    session.memory.keys.is_none() && session.memory.values.is_none(),
                    "bounded incremental path must not concatenate history"
                );
                assert_eq!(session.memory.key_events.len(), 96);
                let prefix: Vec<u32> = ids[..48].iter().chain(&ids[96..144]).copied().collect();
                let prefix_output = model.forward(&prefix, 2, 48, ReadMode::Enabled, false)?;
                assert!(
                    max_delta(
                        &prefix_output.probabilities,
                        &full.probabilities.narrow(1, 0, 48)?
                    )? < 2e-6
                );
                let no_read = model.forward(&ids, 2, 96, ReadMode::NoRead, false)?;
                let full_no_read = base.forward(&ids, 2, 96, ReadMode::NoRead, false)?;
                assert_eq!(
                    max_delta(&no_read.probabilities, &full_no_read.probabilities)?,
                    0.0
                );
                let trained = model.forward(&ids, 2, 96, ReadMode::Enabled, true)?;
                assert!(max_delta(&trained.probabilities, &full.probabilities)? < 2e-6);
                let targets: Vec<u32> = ids.iter().map(|x| (x + 1) % 4096).collect();
                let gradients = trained.loss(&targets)?.backward()?;
                for name in [
                    "read.query.weight",
                    "read.key.weight",
                    "read.value.weight",
                    "recurrent.state.weight",
                    "embedding.weight",
                ] {
                    let gradient = gradients
                        .get(model.variables()[name].as_tensor())
                        .ok_or_else(|| {
                            invalid(format!("missing bounded language gradient {name}"))
                        })?;
                    let norm = gradient.abs()?.sum_all()?.to_scalar::<f32>()?;
                    assert!(norm.is_finite() && norm > 0.0, "{policy:?} {name}");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn bounded_packed_policy_roundtrip_and_session_policy_binding() -> Result<()> {
        let mut config = small(Transport::Quaternion);
        config.context = 80;
        let mut model = JointModel::new(config, &Device::Cpu)?;
        model.configure_quantization(0, 1)?;
        model.set_completed_step(1)?;
        model
            .quantization()
            .ok_or_else(|| invalid("quantizers"))?
            .spec
            .project_parameters(model.variables())?;
        model.set_admission_policy(AdmissionPolicy::Orthant64)?;
        let mut session = model.new_session(1)?;
        model.set_admission_policy(AdmissionPolicy::Recent64)?;
        assert!(model.step(&mut session, &[1], ReadMode::Enabled).is_err());
        model.set_admission_policy(AdmissionPolicy::Orthant64)?;
        let root = precision_test_root("bounded-packed")?;
        model.save_hard(&root)?;
        let loaded = JointModel::load_hard(&root, &Device::Cpu)?;
        assert_eq!(loaded.admission_policy(), AdmissionPolicy::Orthant64);
        let ids: Vec<u32> = (1..=80).collect();
        assert_eq!(
            max_delta(
                &model
                    .forward(&ids, 1, 80, ReadMode::Enabled, false)?
                    .probabilities,
                &loaded
                    .forward(&ids, 1, 80, ReadMode::Enabled, false)?
                    .probabilities
            )?,
            0.0
        );
        let training = loaded.packed_training_start()?;
        assert_eq!(
            max_delta(
                &training
                    .forward(&ids, 1, 80, ReadMode::Enabled, false)?
                    .probabilities,
                &loaded
                    .forward(&ids, 1, 80, ReadMode::Enabled, false)?
                    .probabilities
            )?,
            0.0
        );
        assert!(training
            .forward(&ids, 1, 80, ReadMode::Enabled, true)?
            .loss(&ids)?
            .backward()?
            .get(training.variables()["read.query.weight"].as_tensor())
            .is_some());
        fs::remove_dir_all(root)?;
        Ok(())
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

    fn lorentz(transport: Transport) -> JointConfig {
        JointConfig {
            read_geometry: ReadGeometry::Lorentz,
            ..small(transport)
        }
    }

    fn finite_nonzero_gradient(
        gradients: &candle_core::backprop::GradStore,
        model: &JointModel,
        name: &str,
    ) -> Result<()> {
        let values = gradients
            .get(model.variables()[name].as_tensor())
            .ok_or_else(|| invalid(format!("missing Lorentz language gradient {name}")))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(values.iter().all(|value| value.is_finite()), "{name}");
        assert!(values.iter().any(|&value| value != 0.0), "{name}");
        Ok(())
    }

    /// (a) Full-read Lorentz arm: shares the Dot arm's initial arrays, adds
    /// only the initial log beta and offset, yields finite normalized
    /// causal distributions that differ from Dot, matches its incremental core,
    /// and trains query/key, log beta and offset with finite gradients.
    #[test]
    fn lorentz_read_forward_is_finite_and_trains_query_key_and_log_beta() -> Result<()> {
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        let targets = [17, 4, 9, 2, 5, 6, 11, 7, 22, 8];
        for transport in [Transport::Quaternion, Transport::HouseholderPair] {
            let model = JointModel::new(lorentz(transport), &Device::Cpu)?;
            let dot = JointModel::new(small(transport), &Device::Cpu)?;
            for (name, value) in model.variables() {
                match dot.variables().get(name) {
                    Some(other) => {
                        assert!(same_bits(value.as_tensor(), other.as_tensor())?, "{name}")
                    }
                    None if name == LORENTZ_LOG_BETA => {
                        let log_beta = lorentz_initial_log_beta(&model.config) as f32;
                        assert_eq!(value.to_vec1::<f32>()?, [log_beta]);
                    }
                    None => {
                        assert_eq!(name, LORENTZ_OFFSET);
                        let offset = lorentz_initial_offset(&model.config) as f32;
                        assert_eq!(value.to_vec1::<f32>()?, [offset]);
                    }
                }
            }
            assert_eq!(model.variables().len(), dot.variables().len() + 2);
            let evaluated = model.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
            for tensor in [
                &evaluated.probabilities,
                &evaluated.read_masses,
                &evaluated.no_read_mass,
                &evaluated.states,
            ] {
                let values = tensor.flatten_all()?.to_vec1::<f32>()?;
                assert!(values.iter().all(|value| value.is_finite()));
            }
            // Summing the 4096 F32 masses in F64 leaves only the model's own F32
            // normalization error (about 2e-6 here for either geometry); an F32
            // check sum adds comparable noise, so it is not used for this bound.
            for lane in evaluated.probabilities.to_vec3::<f32>()? {
                for row in lane {
                    let sum: f64 = row.iter().map(|&mass| f64::from(mass)).sum();
                    assert!((sum - 1.0).abs() < 1e-5, "{transport:?} row sum {sum:e}");
                }
            }
            for lane in evaluated.read_masses.to_vec3::<f32>()? {
                for (position, row) in lane.iter().enumerate() {
                    assert!(row[position..].iter().all(|&mass| mass == 0.0));
                }
            }
            let dot_output = dot.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
            assert!(max_delta(&evaluated.read_masses, &dot_output.read_masses)? > 0.0);
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
                        &evaluated.probabilities.narrow(1, position, 1)?.squeeze(1)?
                    )? < 2e-6
                );
            }
            let trained = model.forward(&ids, 2, 5, ReadMode::Enabled, true)?;
            let loss = trained.loss(&targets)?;
            assert!(loss.to_scalar::<f32>()?.is_finite());
            let gradients = loss.backward()?;
            for name in [
                "read.query.weight",
                "read.query.bias",
                "read.key.weight",
                "read.key.bias",
                LORENTZ_LOG_BETA,
                LORENTZ_OFFSET,
                "read.value.weight",
                "read.age",
                "read.no_read.weight",
            ] {
                finite_nonzero_gradient(&gradients, &model, name)?;
            }
            for (name, variable) in model.variables() {
                if let Some(gradient) = gradients.get(variable.as_tensor()) {
                    let values = gradient.flatten_all()?.to_vec1::<f32>()?;
                    assert!(values.iter().all(|value| value.is_finite()), "{name}");
                }
            }
        }
        Ok(())
    }

    /// (a) Bounded-admission Lorentz arm: admission is unchanged, admitted and
    /// padded candidates stay finite, the incremental core matches the full
    /// window, and query/key/log-beta receive finite language gradients.
    #[test]
    fn lorentz_bounded_read_matches_sessions_and_trains() -> Result<()> {
        let mut config = lorentz(Transport::Quaternion);
        config.context = 72;
        let ids: Vec<u32> = (0..144)
            .map(|i| ((i * 17 + i / 7) % 4095 + 1) as u32)
            .collect();
        let targets: Vec<u32> = ids.iter().map(|id| (id + 1) % 4096).collect();
        for policy in [AdmissionPolicy::Recent64, AdmissionPolicy::Orthant64] {
            let mut model = JointModel::new(config.clone(), &Device::Cpu)?;
            model.set_admission_policy(policy)?;
            let full = model.forward(&ids, 2, 72, ReadMode::Enabled, false)?;
            let mut session = model.new_session(2)?;
            let mut padded = false;
            for position in 0..72 {
                let step = model.step(
                    &mut session,
                    &[ids[position], ids[72 + position]],
                    ReadMode::Enabled,
                )?;
                let expected = full.probabilities.narrow(1, position, 1)?.squeeze(1)?;
                assert!(
                    max_delta(&expected, &step.probabilities)? < 2e-6,
                    "{policy:?} at {position}"
                );
                let rows = &step.read_occurrences_by_lane;
                assert!(rows
                    .iter()
                    .all(|row| row.len() <= 64 && row.iter().all(|&id| id < position)));
                padded |= rows.iter().any(|row| row.len() != rows[0].len());
            }
            // Unequal per-lane admission leaves masked padding slots, which
            // must also pass through the Lorentz score without a NaN.
            assert!(policy != AdmissionPolicy::Orthant64 || padded, "{policy:?}");
            let trained = model.forward(&ids, 2, 72, ReadMode::Enabled, true)?;
            let gradients = trained.loss(&targets)?.backward()?;
            for name in [
                "read.query.weight",
                "read.key.weight",
                LORENTZ_LOG_BETA,
                LORENTZ_OFFSET,
            ] {
                finite_nonzero_gradient(&gradients, &model, name)?;
            }
        }
        Ok(())
    }

    /// (b) Lift, Minkowski sign, clamp and arcosh against an F64 reference on
    /// the same F32 inputs. Row 0 has exact lifts (z = 1 clamped, 2, 7); row 1
    /// is generic, including a near-equal pair whose F32 cancellation is
    /// bounded by the product's conditioning. The model composes
    /// exp(log beta)*(offset - d), and the Dot branch is bitwise the retained
    /// expression.
    #[test]
    fn lorentz_score_matches_f64_reference() -> Result<()> {
        let device = Device::Cpu;
        let queries = [[1f32, 1.0, 1.0], [0.3, -1.7, 2.2]];
        let keys = [
            [[1f32, 1.0, 1.0], [2.0, 2.0, 0.0], [-1.0, -1.0, -1.0]],
            [[0.0, 0.0, 0.0], [-0.4, 0.9, 1.1], [0.3, -1.7, 2.1]],
        ];
        let query = Tensor::from_vec(queries.concat(), (2, 3), &device)?;
        let key_values: Vec<f32> = keys.iter().flatten().flatten().copied().collect();
        let key = Tensor::from_vec(key_values, (2, 3, 3), &device)?;
        let distance = lorentz_distance(&query, &key)?.to_vec2::<f32>()?;
        let minimum = f64::from(LORENTZ_MIN_INNER as f32);
        let lift =
            |x: &[f32; 3]| (1.0 + x.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>()).sqrt();
        let mut references = Vec::new();
        for (row, (q, row_keys)) in queries.iter().zip(&keys).enumerate() {
            for (column, k) in row_keys.iter().enumerate() {
                let inner: f64 = q
                    .iter()
                    .zip(k)
                    .map(|(&a, &b)| f64::from(a) * f64::from(b))
                    .sum();
                let product = lift(q) * lift(k);
                let z = (product - inner).max(minimum);
                let reference = z.acosh();
                let conditioning = (product + inner.abs()) / (z * z - 1.0).sqrt();
                let bound = 1e-6 + 1e-5 * reference + 8.0 * f64::from(f32::EPSILON) * conditioning;
                let actual = f64::from(distance[row][column]);
                assert!(
                    (actual - reference).abs() <= bound,
                    "({row},{column}) {actual} vs {reference}"
                );
                if row == 0 {
                    // Exact F32 lifts and products: only arcosh rounding remains.
                    assert!((actual - reference).abs() <= 1e-6, "({row},{column})");
                }
                references.push(reference);
            }
        }
        assert_eq!(references[0], minimum.acosh());
        assert!((references[1] - 2f64.acosh()).abs() < 1e-12);
        assert!((references[2] - 7f64.acosh()).abs() < 1e-12);

        let model = JointModel::new(lorentz(Transport::Quaternion), &device)?;
        model.variables()[LORENTZ_LOG_BETA].set(&Tensor::from_vec(vec![0.5f32], (1,), &device)?)?;
        model.variables()[LORENTZ_OFFSET].set(&Tensor::from_vec(vec![1.25f32], (1,), &device)?)?;
        let scores = model.read_scores(&query, &key, false)?.to_vec2::<f32>()?;
        for (score_row, distance_row) in scores.iter().zip(&distance) {
            for (&score, &d) in score_row.iter().zip(distance_row) {
                let expected = 0.5f64.exp() * (1.25 - f64::from(d));
                assert!((f64::from(score) - expected).abs() <= 1e-6 * (1.0 + expected.abs()));
            }
        }
        let dot = JointModel::new(small(Transport::Quaternion), &device)?;
        let dot_scores = dot.read_scores(&query, &key, false)?;
        let retained = query
            .unsqueeze(1)?
            .matmul(&key.transpose(1, 2)?.contiguous()?)?
            .squeeze(1)?
            .affine(1.0 / (dot.config.read_width as f64).sqrt(), 0.0)?;
        assert!(same_bits(&dot_scores, &retained)?);
        let dot_scores = dot_scores.to_vec2::<f32>()?;
        for (q, (row_keys, score_row)) in queries.iter().zip(keys.iter().zip(&dot_scores)) {
            for (k, &score) in row_keys.iter().zip(score_row) {
                let inner: f64 = q
                    .iter()
                    .zip(k)
                    .map(|(&a, &b)| f64::from(a) * f64::from(b))
                    .sum();
                assert!((f64::from(score) - inner / 8.0).abs() <= 1e-6 * (1.0 + inner.abs()));
            }
        }
        Ok(())
    }

    /// (c) Retained metadata without the field is `Dot`. Dot keeps its exact
    /// JSON, parameter inventory and numerical contracts (the quantized one as
    /// the frozen integer import contract), and its saved checkpoint stays in
    /// the retained format and reloads with bit-identical outputs.
    #[test]
    fn absent_read_geometry_is_dot_with_retained_identity() -> Result<()> {
        let retained_json = r#"{"vocab_size":4096,"width":128,"read_width":64,"context":8,"transport":"quaternion","seed":7}"#;
        let config: JointConfig = serde_json::from_str(retained_json)?;
        assert_eq!(config.read_geometry, ReadGeometry::Dot);
        assert_eq!(config, small(Transport::Quaternion));
        assert_eq!(serde_json::to_string(&config)?, retained_json);
        let explicit: JointConfig = serde_json::from_str(
            &retained_json.replace(r#""seed":7}"#, r#""seed":7,"read_geometry":"dot"}"#),
        )?;
        assert_eq!(explicit, config);
        let model = JointModel::new(config.clone(), &Device::Cpu)?;
        let retained_names = BTreeSet::from([
            "copy.gate.bias",
            "copy.gate.weight",
            "embedding.weight",
            "output.bias",
            "output.norm.weight",
            "read.age",
            "read.key.bias",
            "read.key.weight",
            "read.no_read.bias",
            "read.no_read.weight",
            "read.query.bias",
            "read.query.weight",
            "read.value.bias",
            "read.value.weight",
            "recurrent.bias",
            "recurrent.input.weight",
            "recurrent.state.weight",
            "update.bias",
            "update.gate.bias",
            "update.gate.weight",
            "update.weight",
        ]);
        let names: BTreeSet<&str> = model.variables().keys().map(String::as_str).collect();
        assert_eq!(names, retained_names);
        assert_eq!(model.numerical_contract(), numerical_contract());
        let mut quantized = JointModel::new(config.clone(), &Device::Cpu)?;
        quantized.configure_quantization(0, 1)?;
        assert_eq!(
            quantized.numerical_contract(),
            quantized_numerical_contract()
        );
        let written: Value =
            serde_json::from_str(&serde_json::to_string(&quantized.numerical_contract())?)?;
        assert_eq!(
            written,
            uor_r4_integer::config::quantized_numerical_contract()?
        );
        let root = precision_test_root("retained-dot")?;
        model.save(&root)?;
        let text = fs::read_to_string(root.join("config.json"))?;
        assert!(!text.contains("read_geometry") && !text.contains("lorentz"));
        let reloaded = JointModel::load(&root, &Device::Cpu)?;
        assert_eq!(reloaded.config, config);
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        for mode in [ReadMode::Enabled, ReadMode::NoRead] {
            let expected = model.forward(&ids, 2, 5, mode, false)?;
            let actual = reloaded.forward(&ids, 2, 5, mode, false)?;
            for (actual, expected) in [
                (&actual.probabilities, &expected.probabilities),
                (&actual.states, &expected.states),
                (&actual.read_masses, &expected.read_masses),
                (&actual.no_read_mass, &expected.no_read_mass),
                (&actual.copy_gate, &expected.copy_gate),
            ] {
                assert!(same_bits(actual, expected)?, "{mode:?}");
            }
        }
        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// (d) Lorentz metadata round-trips; its checkpoint binds the geometry,
    /// contract and learned scale and reloads bit-identically. Relabeling it
    /// as Dot fails the contract and then the parameter inventory. The float
    /// model has no packed export; quantizing it binds the quantized Lorentz
    /// contract shared with the integer runtime.
    #[test]
    fn lorentz_config_and_checkpoint_round_trip() -> Result<()> {
        let config = lorentz(Transport::HouseholderPair);
        let text = serde_json::to_string(&config)?;
        assert!(text.ends_with(r#","read_geometry":"lorentz"}"#), "{text}");
        assert_eq!(serde_json::from_str::<JointConfig>(&text)?, config);
        let model = JointModel::new(config.clone(), &Device::Cpu)?;
        model.variables()[LORENTZ_LOG_BETA].set(&Tensor::from_vec(
            vec![0.375f32],
            (1,),
            &Device::Cpu,
        )?)?;
        model.variables()[LORENTZ_OFFSET].set(&Tensor::from_vec(
            vec![4.5f32],
            (1,),
            &Device::Cpu,
        )?)?;
        let contract = model.numerical_contract();
        assert_eq!(contract["read_geometry"]["geometry"], "lorentz");
        assert_ne!(contract["read"], numerical_contract()["read"]);
        let root = precision_test_root("lorentz-checkpoint")?;
        assert!(model.save_hard(&root).is_err());
        model.save(&root)?;
        let reloaded = JointModel::load(&root, &Device::Cpu)?;
        assert_eq!(reloaded.config, config);
        assert_eq!(reloaded.numerical_contract(), contract);
        assert_eq!(
            reloaded.variables()[LORENTZ_LOG_BETA].to_vec1::<f32>()?,
            [0.375]
        );
        assert_eq!(
            reloaded.variables()[LORENTZ_OFFSET].to_vec1::<f32>()?,
            [4.5]
        );
        let ids = [3, 17, 4, 9, 2, 10, 6, 11, 7, 22];
        let expected = model.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
        let actual = reloaded.forward(&ids, 2, 5, ReadMode::Enabled, false)?;
        assert!(same_bits(&expected.probabilities, &actual.probabilities)?);
        assert!(same_bits(&expected.read_masses, &actual.read_masses)?);
        let path = root.join("config.json");
        let mut relabeled: Value = serde_json::from_slice(&fs::read(&path)?)?;
        relabeled["model"]
            .as_object_mut()
            .ok_or_else(|| invalid("checkpoint model object"))?
            .remove("read_geometry");
        fs::write(&path, serde_json::to_vec(&relabeled)?)?;
        let Err(error) = JointModel::load(&root, &Device::Cpu) else {
            return Err(invalid("Lorentz checkpoint relabeled as Dot was accepted"));
        };
        assert!(error.to_string().contains("numerical contract"), "{error}");
        relabeled["numerical_contract"] = numerical_contract();
        fs::write(&path, serde_json::to_vec(&relabeled)?)?;
        let Err(error) = JointModel::load(&root, &Device::Cpu) else {
            return Err(invalid("Lorentz parameters loaded as a Dot model"));
        };
        assert!(error.to_string().contains("parameter names"), "{error}");
        fs::remove_dir_all(root)?;

        let mut quantized = model.without_quantization()?;
        quantized.configure_quantization(0, 1)?;
        let spec = quantized.quantization().map(|state| &state.spec);
        for name in [LORENTZ_LOG_BETA, LORENTZ_OFFSET] {
            let parameter = spec
                .and_then(|spec| spec.parameters.get(name))
                .ok_or_else(|| invalid("quantized Lorentz scalar"))?;
            assert_eq!((parameter.bits, parameter.shape.as_slice()), (16, &[1][..]));
        }
        let written: Value =
            serde_json::from_str(&serde_json::to_string(&quantized.numerical_contract())?)?;
        assert_eq!(
            written,
            uor_r4_integer::config::packed_numerical_contract(ReadGeometry::Lorentz)?
        );
        assert_ne!(written["read"], contract["read"]);
        Ok(())
    }

    /// (e) The initial Lorentz read matches the Dot read's scale. Every initial
    /// distance sits near the initial offset, so scores start centered instead
    /// of ceding the read mass to NoRead; the initial scale makes their spread
    /// and ranking follow the Dot scores to first order, and the remainder is a
    /// per-key radius term. With beta 1 the scores were about ten times flatter.
    #[test]
    fn lorentz_initial_read_matches_dot_scale() -> Result<()> {
        let device = Device::Cpu;
        let statistics = |values: &[f32]| {
            let n = values.len() as f64;
            let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
            let variance = values
                .iter()
                .map(|&v| (f64::from(v) - mean).powi(2))
                .sum::<f64>()
                / n;
            (mean, variance.sqrt())
        };
        for width in [128, 256] {
            let config = JointConfig {
                width,
                ..lorentz(Transport::Quaternion)
            };
            let model = JointModel::new(config.clone(), &device)?;
            let dot = JointModel::new(
                JointConfig {
                    width,
                    ..small(Transport::Quaternion)
                },
                &device,
            )?;
            let offset = lorentz_initial_offset(&config);
            let expected = (1.0 + 2.0 * 64.0 * width as f64 / (64.0 + width as f64)).acosh();
            assert!((offset - expected).abs() < 1e-12);
            let beta = lorentz_initial_log_beta(&config).exp();
            assert!((beta - offset.sinh() / 8.0).abs() < 1e-9);
            let mut rng = Initializer(11);
            let mut draw =
                |count: usize| -> Vec<f32> { (0..count).map(|_| rng.symmetric()).collect() };
            let states = Tensor::from_vec(draw(4 * width), (4, width), &device)?;
            let written = Tensor::from_vec(draw(4 * 32 * width), (4 * 32, width), &device)?;
            // The arms share every initial array, so one query/key set serves both.
            let query = model.linear(&model.normalized(&states, false)?, "read.query", false)?;
            let keys = model
                .linear(&model.normalized(&written, false)?, "read.key", false)?
                .reshape((4, 32, 64))?;
            let distances = lorentz_distance(&query, &keys)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(
                (statistics(&distances).0 - offset).abs() < 0.5,
                "width {width}"
            );
            let lorentz_rows = model.read_scores(&query, &keys, false)?.to_vec2::<f32>()?;
            let dot_rows = dot.read_scores(&query, &keys, false)?.to_vec2::<f32>()?;
            for (lorentz_row, dot_row) in lorentz_rows.iter().zip(&dot_rows) {
                let (lorentz_mean, lorentz_spread) = statistics(lorentz_row);
                let (dot_mean, dot_spread) = statistics(dot_row);
                assert!(
                    lorentz_mean.abs() < 1.5,
                    "width {width}: mean {lorentz_mean}"
                );
                let ratio = lorentz_spread / dot_spread;
                assert!(
                    (0.5..2.0).contains(&ratio),
                    "width {width}: spread ratio {ratio}"
                );
                let covariance = lorentz_row
                    .iter()
                    .zip(dot_row)
                    .map(|(&a, &b)| (f64::from(a) - lorentz_mean) * (f64::from(b) - dot_mean))
                    .sum::<f64>()
                    / lorentz_row.len() as f64;
                let correlation = covariance / (lorentz_spread * dot_spread);
                assert!(
                    correlation > 0.5,
                    "width {width}: correlation {correlation}"
                );
            }
        }
        Ok(())
    }
}
