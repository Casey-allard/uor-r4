//! One persistent offline recurrent-memory learning campaign.
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::baseline_protocol::{
    device, load_evaluator, read_tokens, save_json, verify_identity, Evaluator,
};
use crate::joint_evaluation::{self, STORY_PROBE_SCOPE, STORY_STOP_POLICY};
use crate::joint_model::{JointConfig, JointModel, PrecisionMode, ReadMode};
use crate::joint_optimizer::{AdamConfig, NamedAdamW};
use crate::joint_quantization::QuantizationSpec;
use crate::{invalid, sha256_file, Result};

/// A new window schedule starting from one exact, sealed parent checkpoint.
/// Hashes bind the parent's source, parameters, optimizer clock and data policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrainingWindowTransition {
    pub parent_checkpoint_sha256: String,
    pub parent_campaign_sha256: String,
    pub reason: String,
    pub old_batch: usize,
    pub old_context: usize,
    pub new_batch: usize,
    pub new_context: usize,
    pub parent_optimizer_step: usize,
    pub parent_sampled_target_visits: usize,
}

/// One fixed-scale quantization schedule anchored to an exact continuous parent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantizationTransition {
    pub parent_checkpoint_sha256: String,
    pub parent_campaign_sha256: String,
    pub parent_optimizer_step: usize,
    pub ramp_steps: usize,
    pub reason: String,
}

/// A stored-shadow optimizer policy; it does not change the fixed quantizer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPolicy {
    FixedRepresentableRange,
}

/// One explicit policy change from an exact, already fully quantized parent.
/// The compact typed-spec digest binds every shape, bit width and frozen scale.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionTransition {
    pub policy: ProjectionPolicy,
    pub parent_checkpoint_sha256: String,
    pub parent_campaign_sha256: String,
    pub parent_optimizer_step: usize,
    pub quantization_spec_sha256: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub schema: String,
    pub evaluator_path: PathBuf,
    pub model: JointConfig,
    pub optimizer: AdamConfig,
    pub data_seed: u64,
    pub batch: usize,
    pub context: usize,
    /// Execution-only CPU partition; global batch, horizon and optimizer stay fixed.
    #[serde(default = "default_cpu_gradient_shards")]
    pub cpu_gradient_shards: usize,
    pub total_steps: usize,
    pub development_every_steps: usize,
    pub development_blocks: usize,
    pub checkpoint_steps: Vec<usize>,
    /// Deadline for starting another update, measured from process setup.
    /// Final evaluation/checkpoint/reload needs separately budgeted closeout time.
    pub max_process_seconds: u64,
    /// Optional local supervisor request: stop between updates and checkpoint.
    #[serde(default)]
    pub stop_file: Option<PathBuf>,
    #[serde(default)]
    pub training_window_transition: Option<TrainingWindowTransition>,
    #[serde(default)]
    pub quantization_transition: Option<QuantizationTransition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_transition: Option<ProjectionTransition>,
    pub trial_scope: String,
}

impl Campaign {
    pub fn load(path: &Path) -> Result<Self> {
        let cfg: Self = serde_json::from_slice(&fs::read(path)?)?;
        if cfg.schema != "uor-r4.joint-recurrent-campaign/1"
            || !(1..=64).contains(&cfg.batch)
            || !(8..=256).contains(&cfg.context)
            || cfg.context > cfg.model.context
            || !matches!(cfg.cpu_gradient_shards, 1 | 2 | 4)
            || cfg.batch % cfg.cpu_gradient_shards != 0
            || cfg.model.vocab_size != 4096
            || cfg.total_steps == 0
            || cfg.total_steps > 100_000
            || cfg.development_blocks > 64
            || cfg.checkpoint_steps.len() > 3
            || cfg
                .checkpoint_steps
                .iter()
                .any(|&n| n == 0 || n >= cfg.total_steps)
            || cfg.checkpoint_steps.windows(2).any(|p| p[0] >= p[1])
            || cfg.max_process_seconds == 0
            || cfg.max_process_seconds > 86400
        {
            return Err(invalid("unsupported joint campaign configuration"));
        }
        cfg.model.validate()?;
        cfg.optimizer.validate()?;
        if let Some(transition) = &cfg.training_window_transition {
            if !is_hex_digest(&transition.parent_checkpoint_sha256, 64)
                || !is_hex_digest(&transition.parent_campaign_sha256, 64)
                || transition.reason.trim().is_empty()
                || !(1..=64).contains(&transition.old_batch)
                || !(8..=cfg.model.context).contains(&transition.old_context)
                || transition.new_batch != cfg.batch
                || transition.new_context != cfg.context
                || transition.old_batch * transition.old_context != cfg.batch * cfg.context
                || (transition.old_batch == cfg.batch && transition.old_context == cfg.context)
                || transition.parent_optimizer_step == 0
                || transition
                    .parent_optimizer_step
                    .checked_mul(cfg.batch * cfg.context)
                    != Some(transition.parent_sampled_target_visits)
            {
                return Err(invalid("invalid declared training-window transition"));
            }
        }
        if let Some(transition) = &cfg.quantization_transition {
            if !is_hex_digest(&transition.parent_checkpoint_sha256, 64)
                || !is_hex_digest(&transition.parent_campaign_sha256, 64)
                || transition.reason.trim().is_empty()
                || transition.parent_optimizer_step == 0
                || transition.parent_optimizer_step >= cfg.total_steps
                || transition.ramp_steps == 0
                || transition.ramp_steps > 100_000
                || cfg.batch != 16
                || cfg.context != 256
                || cfg.model.context != 256
            {
                return Err(invalid("invalid declared quantization transition"));
            }
        }
        validate_projection_declaration(&cfg)?;
        Ok(cfg)
    }
}

fn validate_projection_declaration(cfg: &Campaign) -> Result<()> {
    if let Some(transition) = &cfg.projection_transition {
        let quantization = cfg
            .quantization_transition
            .as_ref()
            .ok_or_else(|| invalid("projection requires an existing quantization transition"))?;
        let full_hard_step = quantization
            .parent_optimizer_step
            .checked_add(quantization.ramp_steps)
            .ok_or_else(|| invalid("quantization ramp clock overflow"))?;
        if !is_hex_digest(&transition.parent_checkpoint_sha256, 64)
            || !is_hex_digest(&transition.parent_campaign_sha256, 64)
            || !is_hex_digest(&transition.quantization_spec_sha256, 64)
            || transition.reason.trim().is_empty()
            || transition.parent_optimizer_step < full_hard_step
            || transition.parent_optimizer_step >= cfg.total_steps
            || cfg.batch != 16
            || cfg.context != 256
            || cfg.model.context != 256
        {
            return Err(invalid("invalid full-hard projection transition"));
        }
    }
    Ok(())
}

fn quantization_spec_digest(state: &Value) -> Result<String> {
    let spec: QuantizationSpec = serde_json::from_value(state["spec"].clone())?;
    spec.validate_structure()?;
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(&spec)?)))
}

fn projection_state_matches(cfg: &Campaign, state: &Value, step: usize) -> Result<()> {
    validate_projection_declaration(cfg)?;
    if let Some(transition) = &cfg.projection_transition {
        quantization_state_matches(cfg, state, step)?;
        if step < transition.parent_optimizer_step
            || transition.quantization_spec_sha256 != quantization_spec_digest(state)?
        {
            return Err(invalid(
                "projection clock or fixed quantization specification differs",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_projection_binding(
    cfg: &Campaign,
    checkpoint: &Value,
    state: &Value,
    step: usize,
) -> Result<()> {
    // Missing legacy fields and absent policies both read as JSON null.
    if checkpoint["projection_transition"] != json!(cfg.projection_transition) {
        return Err(invalid("checkpoint projection policy or lineage differs"));
    }
    projection_state_matches(cfg, state, step)
}

/// Only the first explicit transition may enable projection. Its lineage is
/// immutable on later resumes, including changes to execution shard count.
fn projection_resume_transition(
    cfg: &Campaign,
    old: &Campaign,
    checkpoint_sha: &str,
    campaign_sha: &str,
    state: &Value,
    step: usize,
) -> Result<bool> {
    projection_state_matches(cfg, state, step)?;
    match (&old.projection_transition, &cfg.projection_transition) {
        (None, None) => Ok(false),
        (Some(previous), Some(current)) if previous == current => {
            same_learning_configuration(cfg, old)?;
            Ok(false)
        }
        (None, Some(transition)) => {
            if transition.parent_checkpoint_sha256 != checkpoint_sha
                || transition.parent_campaign_sha256 != campaign_sha
                || transition.parent_optimizer_step != step
                || old.model != cfg.model
                || old.optimizer != cfg.optimizer
                || old.data_seed != cfg.data_seed
                || old.batch != cfg.batch
                || old.context != cfg.context
                || old.cpu_gradient_shards != cfg.cpu_gradient_shards
                || old.training_window_transition != cfg.training_window_transition
                || old.quantization_transition != cfg.quantization_transition
            {
                return Err(invalid(
                    "projection transition changes its exact full-hard parent",
                ));
            }
            Ok(true)
        }
        _ => Err(invalid(
            "resume changes persistent projection policy or lineage",
        )),
    }
}

fn quantization_state_matches(cfg: &Campaign, state: &Value, step: usize) -> Result<()> {
    match &cfg.quantization_transition {
        None if state.is_null() => Ok(()),
        Some(transition)
            if state["start_step"] == json!(transition.parent_optimizer_step)
                && state["ramp_steps"] == json!(transition.ramp_steps)
                && state["completed_step"] == json!(step)
                && step >= transition.parent_optimizer_step
                && state["spec"].is_object() =>
        {
            Ok(())
        }
        _ => Err(invalid(
            "model quantization schedule/specification/clock differs from campaign",
        )),
    }
}

fn validate_quantization_binding(
    cfg: &Campaign,
    checkpoint: &Value,
    state: &Value,
    step: usize,
) -> Result<()> {
    if checkpoint["quantization_transition"] != json!(cfg.quantization_transition)
        || checkpoint["quantization"] != *state
    {
        return Err(invalid(
            "checkpoint/model quantization state or transition differs",
        ));
    }
    if state["preparation"] == "calibrated_for_rounding" {
        let calibration = &checkpoint["shadow_calibration"];
        if calibration["schema"] != "uor-r4.calibrated-rounding-shadow/1"
            || calibration["model_step"] != json!(step)
            || calibration["optimizer_updates"] != 0
            || calibration["sampler_steps_advanced"] != 0
            || calibration["quantization_spec_sha256"] != quantization_spec_digest(state)?
        {
            return Err(invalid(
                "calibrated shadow provenance differs from its model clock/grids",
            ));
        }
    }
    quantization_state_matches(cfg, state, step)?;
    validate_projection_binding(cfg, checkpoint, state, step)
}

/// Returns true only for the first, declared continuous-to-quantized transition.
fn quantization_resume_transition(
    cfg: &Campaign,
    old: &Campaign,
    checkpoint_sha: &str,
    campaign_sha: &str,
    step: usize,
) -> Result<bool> {
    match (&old.quantization_transition, &cfg.quantization_transition) {
        (None, None) => Ok(false),
        (Some(previous), Some(current)) if previous == current => {
            if old.batch != cfg.batch
                || old.context != cfg.context
                || old.training_window_transition != cfg.training_window_transition
            {
                return Err(invalid(
                    "quantized resume changes the retained training horizon",
                ));
            }
            Ok(false)
        }
        (None, Some(transition)) => {
            if transition.parent_checkpoint_sha256 != checkpoint_sha
                || transition.parent_campaign_sha256 != campaign_sha
                || transition.parent_optimizer_step != step
                || old.batch != 16
                || cfg.batch != 16
                || old.context != 256
                || cfg.context != 256
                || old.model != cfg.model
                || old.optimizer != cfg.optimizer
                || old.data_seed != cfg.data_seed
                || old.cpu_gradient_shards != cfg.cpu_gradient_shards
                || old.training_window_transition != cfg.training_window_transition
            {
                return Err(invalid(
                    "quantization transition does not match its exact continuous parent",
                ));
            }
            Ok(true)
        }
        _ => Err(invalid(
            "ordinary resume changes quantization transition lineage",
        )),
    }
}

fn default_cpu_gradient_shards() -> usize {
    1
}

fn is_hex_digest(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn metadata(cfg: &Campaign, mode: &str, device_name: &str) -> Result<Value> {
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if source == "UNBOUND" {
        return Err(invalid("joint campaign requires source-bound build"));
    }
    Ok(
        json!({"schema":"uor-r4.joint-recurrent-report/1", "mode":mode,
        "source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "campaign":cfg,"requested_device":device_name,
        "cpu_gradient_shards":cfg.cpu_gradient_shards,
        "gradient_execution":"Independent full-window CPU shards share immutable Vars; ordered weighted mean F32 gradients; one global clip/AdamW update. Reduction order can differ from an unsplit batch.",
        "cpu_accelerate_compiled":cfg!(feature="cpu-accelerate"),
        "candle_source":"vendored0.9.2; four-line Accelerate operand slice correction; UPSTREAM.json",
        "deadline_scope":"max_process_seconds stops new updates; measured closeout allowance is budgeted separately",
        "scope":"Offline recurrent-memory learner or quantized numerical emulator; no transformer backbone, compliant integer serving kernel, geometry promotion or energy claim"}),
    )
}

fn splitmix(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d049bb133111eb);
    state ^ (state >> 31)
}

/// Stateless, reproducible random windows, uniform over valid starts in the two
/// source stores. No window crosses their boundary; every target is shifted once.
pub(crate) fn training_batch(
    stores: &[Vec<u16>],
    cfg: &Campaign,
    step: usize,
) -> Result<(Vec<u32>, Vec<u32>)> {
    let windows: Vec<_> = stores
        .iter()
        .map(|s| s.len().saturating_sub(cfg.context))
        .collect();
    let total: usize = windows.iter().sum();
    if total == 0 {
        return Err(invalid("no valid training windows"));
    }
    let mut inputs = Vec::with_capacity(cfg.batch * cfg.context);
    let mut targets = Vec::with_capacity(cfg.batch * cfg.context);
    for lane in 0..cfg.batch {
        let counter = cfg.data_seed
            ^ (step as u64).wrapping_mul(0xd1342543de82ef95)
            ^ (lane as u64).wrapping_mul(0x9e3779b97f4a7c15);
        let mut start = (splitmix(counter) % total as u64) as usize;
        let mut selected = None;
        for (index, &length) in windows.iter().enumerate() {
            if start < length {
                selected = Some(index);
                break;
            }
            start -= length;
        }
        let store = &stores[selected.ok_or_else(|| invalid("window sampler boundary"))?];
        inputs.extend(
            store[start..start + cfg.context]
                .iter()
                .map(|&n| u32::from(n)),
        );
        targets.extend(
            store[start + 1..start + cfg.context + 1]
                .iter()
                .map(|&n| u32::from(n)),
        );
    }
    Ok((inputs, targets))
}

fn quick_loss(
    model: &JointModel,
    tokens: &[u16],
    blocks: usize,
    batch: usize,
) -> Result<Option<f64>> {
    if blocks == 0 {
        return Ok(None);
    }
    if tokens.len() < blocks * 256 + 1 {
        return Err(invalid("development prefix too short"));
    }
    let mut weighted = 0.0;
    for first in (0..blocks).step_by(batch) {
        let count = (blocks - first).min(batch);
        let mut inputs = Vec::with_capacity(count * 256);
        let mut targets = Vec::with_capacity(count * 256);
        for block in first..first + count {
            let start = block * 256;
            inputs.extend(tokens[start..start + 256].iter().map(|&n| u32::from(n)));
            targets.extend(tokens[start + 1..start + 257].iter().map(|&n| u32::from(n)));
        }
        let value = model
            .forward(&inputs, count, 256, ReadMode::Enabled, false)?
            .loss(&targets)?
            .to_scalar::<f32>()?;
        if !value.is_finite() {
            return Err(invalid("nonfinite development likelihood"));
        }
        weighted += f64::from(value) * count as f64;
    }
    Ok(Some(weighted / blocks as f64))
}

fn shadow_loss(
    model: &JointModel,
    tokens: &[u16],
    blocks: usize,
    batch: usize,
) -> Result<Option<f64>> {
    if model.quantization().is_some() {
        quick_loss(&model.without_quantization()?, tokens, blocks, batch)
    } else {
        Ok(None)
    }
}

struct HardParameterSnapshot {
    coordinates: usize,
    out_of_range_coordinates: usize,
    values_and_codes: BTreeMap<String, (Vec<u32>, Vec<i32>)>,
    shadow_sha256: String,
    hard_values_sha256: String,
    codes_sha256: String,
}

/// The fixed dyadic scale maps each hard F32 value to one exact integer code.
/// Retain both for direct, all-coordinate equality; hashes are provenance only.
fn hard_parameter_snapshot(model: &JointModel) -> Result<HardParameterSnapshot> {
    let state = model
        .quantization()
        .ok_or_else(|| invalid("projection witness requires quantization"))?;
    state.spec.validate(model.variables())?;
    let mut values_and_codes = BTreeMap::new();
    let mut shadow_hash = Sha256::new();
    let mut hard_hash = Sha256::new();
    let mut code_hash = Sha256::new();
    let mut coordinates = 0usize;
    let mut out_of_range_coordinates = 0usize;
    for (name, variable) in model.variables() {
        let parameter = state
            .spec
            .parameters
            .get(name)
            .ok_or_else(|| invalid("projection parameter specification missing"))?;
        let header = serde_json::to_vec(&(name, variable.dims()))?;
        for digest in [&mut shadow_hash, &mut hard_hash, &mut code_hash] {
            digest.update((header.len() as u64).to_le_bytes());
            digest.update(&header);
        }
        let shadow = variable
            .as_detached_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let values = state
            .spec
            .parameter(name, variable.as_tensor(), 1.0, false)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        if shadow.len() != values.len() {
            return Err(invalid("projection hard/shadow coordinate counts differ"));
        }
        let row_width = if parameter.shape.len() == 2 {
            parameter.shape[1]
        } else {
            parameter.shape[0]
        };
        let qmax = if parameter.bits == 4 { 7 } else { 32767 };
        let mut bits = Vec::with_capacity(values.len());
        let mut codes = Vec::with_capacity(values.len());
        for (index, (&raw, &hard)) in shadow.iter().zip(&values).enumerate() {
            let exponent = *parameter
                .row_exponents
                .get(index / row_width)
                .ok_or_else(|| invalid("projection witness row scale missing"))?;
            let code = hard / 2_f32.powi(i32::from(exponent));
            let shadow_code = raw / 2_f32.powi(i32::from(exponent));
            if !raw.is_finite()
                || !hard.is_finite()
                || !code.is_finite()
                || code != code.round()
                || code.abs() > qmax as f32
            {
                return Err(invalid(
                    "projection witness has a nonfinite or off-grid parameter",
                ));
            }
            let code = code as i32;
            if shadow_code < -(qmax as f32) || shadow_code > qmax as f32 {
                out_of_range_coordinates += 1;
            }
            shadow_hash.update(raw.to_bits().to_le_bytes());
            hard_hash.update(hard.to_bits().to_le_bytes());
            code_hash.update(code.to_le_bytes());
            bits.push(hard.to_bits());
            codes.push(code);
        }
        coordinates = coordinates
            .checked_add(values.len())
            .ok_or_else(|| invalid("projection parameter count overflow"))?;
        values_and_codes.insert(name.clone(), (bits, codes));
    }
    Ok(HardParameterSnapshot {
        coordinates,
        out_of_range_coordinates,
        values_and_codes,
        shadow_sha256: hex::encode(shadow_hash.finalize()),
        hard_values_sha256: hex::encode(hard_hash.finalize()),
        codes_sha256: hex::encode(code_hash.finalize()),
    })
}

fn words_sha256(values: &[u32]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_le_bytes());
    }
    hex::encode(hash.finalize())
}

/// Run once before any fit/development loss. It is deliberately no-grad and
/// receives the optimizer immutably; a policy resume also witnesses idempotence.
fn witness_entry_projection(
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    stores: &[Vec<u16>],
    begin: usize,
    loaded_parent: &Value,
    out: &Path,
) -> Result<Value> {
    let Some(transition) = &cfg.projection_transition else {
        return Ok(Value::Null);
    };
    let state = model
        .quantization()
        .ok_or_else(|| invalid("projection requires a quantized model"))?;
    projection_state_matches(cfg, &serde_json::to_value(state)?, begin)?;
    let (inputs, targets) = training_batch(stores, cfg, begin)?;
    let probe = inputs
        .get(..256)
        .ok_or_else(|| invalid("entry projection requires one actual full context"))?;
    let before = hard_parameter_snapshot(model)?;
    let variable_ids: Vec<_> = model.variables().values().map(|v| v.id()).collect();
    let optimizer_before = optimizer.continuity_fingerprint()?;
    let before_probabilities = model
        .forward(probe, 1, 256, ReadMode::Enabled, false)?
        .probabilities
        .flatten_all()?
        .to_vec1::<f32>()?;
    let statistics = state.spec.project_parameters(model.variables())?;
    let after = hard_parameter_snapshot(model)?;
    let optimizer_after = optimizer.continuity_fingerprint()?;
    let after_probabilities = model
        .forward(probe, 1, 256, ReadMode::Enabled, false)?
        .probabilities
        .flatten_all()?
        .to_vec1::<f32>()?;
    let finite = before_probabilities.iter().all(|p| p.is_finite())
        && after_probabilities.iter().all(|p| p.is_finite());
    let probability_bits_equal = before_probabilities.len() == after_probabilities.len()
        && before_probabilities.len() == 256 * cfg.model.vocab_size
        && before_probabilities
            .iter()
            .zip(&after_probabilities)
            .all(|(a, b)| a.to_bits() == b.to_bits());
    let max_delta = before_probabilities
        .iter()
        .zip(&after_probabilities)
        .map(|(&a, &b)| (f64::from(a) - f64::from(b)).abs())
        .fold(0.0_f64, f64::max);
    let hard_equal = before.values_and_codes == after.values_and_codes;
    let ids_equal = variable_ids
        == model
            .variables()
            .values()
            .map(|v| v.id())
            .collect::<Vec<_>>();
    let sampler_equal = training_batch(stores, cfg, begin)? == (inputs.clone(), targets.clone());
    let optimizer_equal =
        optimizer_before == optimizer_after && optimizer.step_count() as usize == begin;
    let passed = hard_equal
        && ids_equal
        && sampler_equal
        && optimizer_equal
        && before.coordinates == after.coordinates
        && statistics.total_coordinates == before.coordinates
        && statistics.projected_coordinates == before.out_of_range_coordinates
        && after.out_of_range_coordinates == 0
        && finite
        && probability_bits_equal
        && max_delta == 0.0;
    let report = json!({
        "schema":"uor-r4.joint-projection-entry/1",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "status":if passed {"PRESERVED_HARD_MODEL_AND_CONTINUATION"} else {"FAILED_INTEGRITY"},
        "projection_transition":transition,"entry_projection":statistics,
        "loaded_parent":loaded_parent,
        "quantization_spec_sha256":quantization_spec_digest(&serde_json::to_value(state)?)?,
        "parameter_tensors":before.values_and_codes.len(),"parameter_coordinates":before.coordinates,
        "out_of_range_shadow_coordinates_before":before.out_of_range_coordinates,
        "out_of_range_shadow_coordinates_after":after.out_of_range_coordinates,
        "hard_parameter_values_and_codes_equal":hard_equal,"variable_ids_preserved":ids_equal,
        "shadow_parameters_sha256_before":before.shadow_sha256,
        "shadow_parameters_sha256_after":after.shadow_sha256,
        "hard_parameter_values_sha256_before":before.hard_values_sha256,
        "hard_parameter_values_sha256_after":after.hard_values_sha256,
        "hard_parameter_codes_sha256_before":before.codes_sha256,
        "hard_parameter_codes_sha256_after":after.codes_sha256,
        "parameter_hash_encoding":"BTreeMap name order; LE-u64 JSON header length; compact JSON (name,shape); per-coordinate LE-F32 bits or LE-i32 dyadic code",
        "optimizer_before":optimizer_before,"optimizer_after":optimizer_after,
        "optimizer_moments_and_clocks_equal":optimizer_equal,
        "optimizer_updates_during_entry":0,"sampler_advanced_steps":0,
        "next_data_step":begin,"sampled_target_visits":begin*cfg.batch*cfg.context,
        "next_training_inputs_and_targets_equal":sampler_equal,
        "next_training_inputs_sha256_le_u32":words_sha256(&inputs),
        "next_training_targets_sha256_le_u32":words_sha256(&targets),
        "probe":{"source":"lane0 of the next scheduled training batch; reset full prefix",
            "batch":1,"context":256,"input_ids":probe,"input_sha256_le_u32":words_sha256(probe)},
        "probability_coordinates":before_probabilities.len(),"probabilities_finite":finite,
        "hard_probability_bits_equal":probability_bits_equal,"hard_probability_max_absolute_delta":finite.then_some(max_delta),
        "scope":"Integrity of fixed-range projection at this loaded full-hard checkpoint. No optimizer, data-cursor or quantization-scale update; not a retention or stability gate."
    });
    // Preserve the witness even if integrity fails; the outer attempt is sealed.
    save_json(&out.join("projection-entry.json"), &report)?;
    if !passed {
        return Err(invalid(
            "entry projection changed hard values, codes, probabilities or continuation state",
        ));
    }
    Ok(report)
}

fn save_checkpoint(
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    directory: &Path,
    step: usize,
    status: &str,
    evaluator_sha: &str,
) -> Result<()> {
    save_checkpoint_with_calibration(
        model,
        optimizer,
        cfg,
        directory,
        step,
        status,
        evaluator_sha,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn save_checkpoint_with_calibration(
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    directory: &Path,
    step: usize,
    status: &str,
    evaluator_sha: &str,
    calibration: Option<&Value>,
) -> Result<()> {
    let quantization = serde_json::to_value(model.quantization())?;
    quantization_state_matches(cfg, &quantization, step)?;
    projection_state_matches(cfg, &quantization, step)?;
    if optimizer.step_count() as usize != step {
        return Err(invalid("checkpoint optimizer and model clocks differ"));
    }
    model.save(directory)?;
    optimizer.save(directory)?;
    save_json(&directory.join("campaign.json"), cfg)?;
    let mut binding = json!({
        "schema":"uor-r4.joint-recurrent-checkpoint/1","status":status,
        "optimizer_step":step,"sampled_target_visits":step*cfg.batch*cfg.context,
        "data_sampler":"splitmix64-counter-v1;valid-window-union;no-cross-store;step/lane-bound",
        "next_data_step":step,"data_seed":cfg.data_seed,
        "training_batch":cfg.batch,"training_context":cfg.context,
        "cpu_gradient_shards":cfg.cpu_gradient_shards,
        "sampled_targets_per_step":cfg.batch*cfg.context,
        "training_window_transition":cfg.training_window_transition,
        "quantization_transition":cfg.quantization_transition,
        "quantization":quantization,
        "evaluator_sha256":evaluator_sha,
        "model_sha256":sha256_file(&directory.join("model.safetensors"))?,
        "model_config_sha256":sha256_file(&directory.join("config.json"))?,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND")
    });
    if let Some(transition) = &cfg.projection_transition {
        binding["projection_transition"] = json!(transition);
    }
    if let Some(calibration) = calibration {
        binding["shadow_calibration"] = calibration.clone();
    }
    save_json(&directory.join("checkpoint.json"), &binding)?;
    report_output::seal(directory)?;
    report_output::verify(directory)?;
    Ok(())
}

pub fn fit(cfg: &Campaign, out: &Path, device_name: &str, resume: Option<&Path>) -> Result<()> {
    let started = Instant::now();
    let mut report = metadata(cfg, "joint-fit", device_name)?;
    if cfg.cpu_gradient_shards > 1 && device_name != "cpu" {
        return Err(invalid("multiple gradient shards require the CPU backend"));
    }
    if (cfg.training_window_transition.is_some()
        || cfg.quantization_transition.is_some()
        || cfg.projection_transition.is_some())
        && resume.is_none()
    {
        return Err(invalid(
            "a declared training transition requires its sealed parent",
        ));
    }
    report["training_window_transition"] = json!(cfg.training_window_transition);
    report["training_window_transition_applied"] = json!(false);
    report["quantization_transition"] = json!(cfg.quantization_transition);
    report["quantization_transition_applied"] = json!(false);
    report["projection_transition"] = json!(cfg.projection_transition);
    report["projection_transition_applied"] = json!(false);
    report["evaluation_numerics"] = json!(if cfg.quantization_transition.is_some() {
        "Full-strength quantized numerical emulator, including during the training ramp; F32-origin quick_loss reductions. Not an integer serving kernel."
    } else {
        "Continuous F32 model; F32-origin quick_loss reductions."
    });
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    // Reserve every checkpoint leaf before loading data or model. The final leaf
    // retains a clean resource-limited stop as well as a complete run.
    let final_path = out.join("checkpoint-final");
    report_output::claim(&final_path)?;
    for step in &cfg.checkpoint_steps {
        report_output::claim(&out.join(format!("checkpoint-{step:08}")))?;
    }
    save_json(&out.join("campaign.json"), cfg)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    save_json(
        &out.join("frozen-story-probes.json"),
        &json!({"scope":STORY_PROBE_SCOPE,
        "stop_policy":STORY_STOP_POLICY,"probes":joint_evaluation::story_probes()}),
    )?;
    let sources = evaluator.document["train_sources"]
        .as_array()
        .ok_or_else(|| invalid("training sources"))?;
    let stores = sources
        .iter()
        .map(read_tokens)
        .collect::<Result<Vec<_>>>()?;
    let dev = read_tokens(&evaluator.document["dev_source"])?;
    let selected = device(device_name)?;
    let (mut model, mut optimizer, begin) = if let Some(path) = resume {
        report_output::verify(path)?;
        let old = Campaign::load(&path.join("campaign.json"))?;
        let checkpoint: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
        let checkpoint_sha = sha256_file(&path.join("checkpoint.json"))?;
        let campaign_sha = sha256_file(&path.join("campaign.json"))?;
        let begin = usize::try_from(
            checkpoint["next_data_step"]
                .as_u64()
                .ok_or_else(|| invalid("resume cursor"))?,
        )
        .map_err(|_| invalid("resume cursor exceeds platform usize"))?;
        let parent_visits = begin
            .checked_mul(old.batch * old.context)
            .ok_or_else(|| invalid("resume target-visit count overflow"))?;
        // Time/target/checkpoint/report frequency may change on ordinary resume.
        // Window dimensions require a declaration bound to this exact parent.
        if serde_json::to_value(&old.model)? != serde_json::to_value(&cfg.model)?
            || old.optimizer != cfg.optimizer
            || old.data_seed != cfg.data_seed
            || checkpoint["evaluator_sha256"] != evaluator.sha256
            || checkpoint["model_sha256"] != sha256_file(&path.join("model.safetensors"))?
            || (checkpoint.get("model_config_sha256").is_some()
                && checkpoint["model_config_sha256"] != sha256_file(&path.join("config.json"))?)
            || checkpoint["schema"] != "uor-r4.joint-recurrent-checkpoint/1"
            || checkpoint["training_batch"] != json!(old.batch)
            || checkpoint["training_context"] != json!(old.context)
            || checkpoint["sampled_targets_per_step"] != json!(old.batch * old.context)
            || checkpoint
                .get("cpu_gradient_shards")
                .map_or(Some(1), Value::as_u64)
                != Some(old.cpu_gradient_shards as u64)
            || !checkpoint["source_commit"]
                .as_str()
                .is_some_and(|source| is_hex_digest(source, 40))
        {
            return Err(invalid(
                "resume changes model, optimizer, data, evaluator or bound weights/source",
            ));
        }
        let window_changed = old.batch != cfg.batch || old.context != cfg.context;
        if window_changed {
            let transition = cfg
                .training_window_transition
                .as_ref()
                .ok_or_else(|| invalid("resume changes windows without a declared transition"))?;
            if transition.parent_checkpoint_sha256 != checkpoint_sha
                || transition.parent_campaign_sha256 != campaign_sha
                || transition.old_batch != old.batch
                || transition.old_context != old.context
                || transition.new_batch != cfg.batch
                || transition.new_context != cfg.context
                || old.batch * old.context != cfg.batch * cfg.context
                || transition.parent_optimizer_step != begin
                || transition.parent_sampled_target_visits != parent_visits
            {
                return Err(invalid(
                    "training-window transition does not match its actual parent",
                ));
            }
        } else if cfg.training_window_transition != old.training_window_transition {
            return Err(invalid(
                "ordinary resume changes training-window transition lineage",
            ));
        }
        let mut model = JointModel::load(path, &selected)?;
        if model.is_rounding_calibrated() {
            return Err(invalid("calibrated shadow is reserved for alpha-only rounding; resume the continuous parent for language training"));
        }
        if model.config != cfg.model {
            return Err(invalid("loaded resume model differs from campaign"));
        }
        validate_quantization_binding(
            &old,
            &checkpoint,
            &serde_json::to_value(model.quantization())?,
            begin,
        )?;
        if checkpoint["training_window_transition"] != json!(old.training_window_transition) {
            return Err(invalid(
                "checkpoint training-window lineage differs from parent campaign",
            ));
        }
        let quantization_changed =
            quantization_resume_transition(cfg, &old, &checkpoint_sha, &campaign_sha, begin)?;
        let projection_changed = projection_resume_transition(
            cfg,
            &old,
            &checkpoint_sha,
            &campaign_sha,
            &serde_json::to_value(model.quantization())?,
            begin,
        )?;
        if quantization_changed {
            model.configure_quantization(
                begin,
                cfg.quantization_transition
                    .as_ref()
                    .ok_or_else(|| invalid("missing quantization transition"))?
                    .ramp_steps,
            )?;
        }
        quantization_state_matches(cfg, &serde_json::to_value(model.quantization())?, begin)?;
        let optimizer = NamedAdamW::load(path, model.variables(), &cfg.optimizer)?;
        if optimizer.step_count() as usize != begin
            || begin >= cfg.total_steps
            || checkpoint["optimizer_step"] != json!(begin)
            || checkpoint["data_seed"] != json!(cfg.data_seed)
            || checkpoint["sampled_target_visits"] != json!(parent_visits)
            || checkpoint["data_sampler"]
                != "splitmix64-counter-v1;valid-window-union;no-cross-store;step/lane-bound"
        {
            return Err(invalid(
                "resume optimizer/cursor mismatch or completed target",
            ));
        }
        report["resume_parent"] = json!({
            "path":path,"checkpoint_sha256":checkpoint_sha,"campaign_sha256":campaign_sha,
            "source_commit":checkpoint["source_commit"],"optimizer_step":begin,
            "sampled_target_visits":parent_visits,
        });
        report["training_window_transition_applied"] = json!(window_changed);
        report["quantization_transition_applied"] = json!(quantization_changed);
        report["projection_transition_applied"] = json!(projection_changed);
        report["resume_optimizer_files"] = json!({
            "metadata_sha256":sha256_file(&path.join(crate::joint_optimizer::METADATA_FILE))?,
            "moments_sha256":sha256_file(&path.join(crate::joint_optimizer::MOMENT_FILE))?,
        });
        report["cpu_gradient_shards_changed_on_resume"] =
            json!(old.cpu_gradient_shards != cfg.cpu_gradient_shards);
        report["window_sampler_continuation"] = json!(
            "Same counter algorithm and global step; a declared batch/context transition changes sampled windows and lane count, with no data-cursor or optimizer reset."
        );
        (model, optimizer, begin)
    } else {
        let model = JointModel::new(cfg.model.clone(), &selected)?;
        let optimizer = NamedAdamW::new(model.variables(), cfg.optimizer.clone())?;
        (model, optimizer, 0)
    };
    report["evaluator_sha256"] = json!(evaluator.sha256);
    report["training_sources"] = evaluator.document["train_sources"].clone();
    report["actual_device"] = json!(format!("{:?}", selected.location()));
    report["parameter_count"] = json!(model
        .variables()
        .values()
        .map(|v| v.elem_count())
        .sum::<usize>());
    report["starting_step"] = json!(begin);
    report["starting_sampled_target_visits"] = json!(begin * cfg.batch * cfg.context);
    report["initial_quantization"] = serde_json::to_value(model.quantization())?;
    report["initial_training_quantization_strength"] = json!(model.training_strength());
    let entry_projection = witness_entry_projection(
        &model,
        &optimizer,
        cfg,
        &stores,
        begin,
        &report["resume_parent"],
        out,
    )?;
    if cfg.projection_transition.is_some() {
        report["projection_entry"] = entry_projection;
        report["projection_entry_sha256"] = json!(sha256_file(&out.join("projection-entry.json"))?);
    }
    let (retained_inputs, retained_targets) = training_batch(&stores, cfg, 0)?;
    let initial_fit = f64::from(
        model
            .forward(
                &retained_inputs,
                cfg.batch,
                cfg.context,
                ReadMode::Enabled,
                false,
            )?
            .loss(&retained_targets)?
            .to_scalar::<f32>()?,
    );
    if !initial_fit.is_finite() {
        return Err(invalid("nonfinite initial retained loss"));
    }
    let initial_dev = quick_loss(&model, &dev, cfg.development_blocks, cfg.batch)?;
    report["initial_retained_batch_nll"] = json!(initial_fit);
    report["initial_development_nll"] = json!(initial_dev);
    report["initial_shadow_development_nll"] = json!(shadow_loss(
        &model,
        &dev,
        cfg.development_blocks,
        cfg.batch
    )?);
    let mut curve = BufWriter::new(File::create_new(out.join("learning-curve.jsonl"))?);
    let mut step_times = Vec::new();
    let mut complete = begin;
    let mut projection_updates = 0usize;
    let mut projected_coordinate_updates = 0usize;
    let mut maximum_projection_correction = 0.0_f64;
    for step in begin..cfg.total_steps {
        if started.elapsed().as_secs() >= cfg.max_process_seconds
            || cfg.stop_file.as_ref().is_some_and(|path| path.exists())
        {
            break;
        }
        let step_started = Instant::now();
        let quantization_strength = model.training_strength();
        let (inputs, targets) = training_batch(&stores, cfg, step)?;
        let gradients = crate::joint_parallel::batch_gradients(
            &model,
            &inputs,
            &targets,
            cfg.batch,
            cfg.context,
            cfg.cpu_gradient_shards,
        )?;
        let value = gradients.mean_nll;
        let update = optimizer.step(model.variables(), &gradients.gradients)?;
        let projection = if cfg.projection_transition.is_some() {
            let statistics = model
                .quantization()
                .ok_or_else(|| invalid("projection policy lost its quantization state"))?
                .spec
                .project_parameters(model.variables())?;
            projection_updates += 1;
            projected_coordinate_updates = projected_coordinate_updates
                .checked_add(statistics.projected_coordinates)
                .ok_or_else(|| invalid("projection coordinate-update count overflow"))?;
            maximum_projection_correction =
                maximum_projection_correction.max(statistics.maximum_absolute_correction);
            Some(statistics)
        } else {
            None
        };
        model.set_completed_step(step + 1)?;
        let elapsed = step_started.elapsed().as_secs_f64();
        step_times.push(elapsed);
        complete = step + 1;
        let mut row = json!({"step":complete,"sampled_target_visits":complete*cfg.batch*cfg.context,
            "batch_mean_nll":value,"complete_step_seconds":elapsed,"optimizer":update,
            "cpu_gradient_shards":cfg.cpu_gradient_shards,
            "training_quantization_strength":quantization_strength,
            "development_numerics":if model.quantization().is_some() { "full-strength quantized numerical emulator" } else { "continuous" }});
        if let Some(statistics) = projection {
            row["projection"] = json!(statistics);
        }
        drop(gradients);
        if cfg.development_every_steps > 0 && complete % cfg.development_every_steps == 0 {
            row["development_nll"] =
                json!(quick_loss(&model, &dev, cfg.development_blocks, cfg.batch)?);
            row["shadow_development_nll"] = json!(shadow_loss(
                &model,
                &dev,
                cfg.development_blocks,
                cfg.batch
            )?);
        }
        serde_json::to_writer(&mut curve, &row)?;
        writeln!(curve)?;
        curve.flush()?;
        if step == begin || complete % 10 == 0 {
            eprintln!(
                "joint step {complete}/{}: NLL {value:.5}, {elapsed:.4}s, {:.1} tokens/s",
                cfg.total_steps,
                (cfg.batch * cfg.context) as f64 / elapsed
            );
        }
        if cfg.checkpoint_steps.contains(&complete) {
            save_checkpoint(
                &model,
                &optimizer,
                cfg,
                &out.join(format!("checkpoint-{complete:08}")),
                complete,
                "INTERMEDIATE",
                &evaluator.sha256,
            )?;
        }
    }
    curve.flush()?;
    curve.get_ref().sync_all()?;
    let status = if complete == cfg.total_steps {
        "TARGET_COMPLETE"
    } else {
        "RESOURCE_CHECKPOINT"
    };
    let final_fit = f64::from(
        model
            .forward(
                &retained_inputs,
                cfg.batch,
                cfg.context,
                ReadMode::Enabled,
                false,
            )?
            .loss(&retained_targets)?
            .to_scalar::<f32>()?,
    );
    if !final_fit.is_finite() {
        return Err(invalid("nonfinite final retained loss"));
    }
    let final_dev = quick_loss(&model, &dev, cfg.development_blocks, cfg.batch)?;
    let final_shadow_dev = shadow_loss(&model, &dev, cfg.development_blocks, cfg.batch)?;
    save_checkpoint(
        &model,
        &optimizer,
        cfg,
        &final_path,
        complete,
        status,
        &evaluator.sha256,
    )?;
    // Actual artifact reload; generation/evaluation consume this same checkpoint.
    let restored = JointModel::load(&final_path, &selected)?;
    if restored.config != model.config || restored.quantization() != model.quantization() {
        return Err(invalid(
            "loaded checkpoint changed model configuration or quantization state",
        ));
    }
    let reloaded_fit = f64::from(
        restored
            .forward(
                &retained_inputs,
                cfg.batch,
                cfg.context,
                ReadMode::Enabled,
                false,
            )?
            .loss(&retained_targets)?
            .to_scalar::<f32>()?,
    );
    let reload_delta = (final_fit - reloaded_fit).abs();
    if !reloaded_fit.is_finite()
        || !reload_delta.is_finite()
        || reload_delta > 1e-6
        || (model.quantization().is_some() && final_fit.to_bits() != reloaded_fit.to_bits())
    {
        return Err(invalid("loaded recurrent checkpoint changed retained loss"));
    }
    let tokenizer = load_tokenizer(&evaluator.document)?;
    let smoke = joint_evaluation::generate(
        &restored,
        &tokenizer,
        "Once upon a time, there was a little girl who",
        ReadMode::Enabled,
        Some(240924),
        48,
    )?;
    save_json(&out.join("loaded-generation.json"), &smoke)?;
    let warm = &step_times[step_times.len().min(2)..];
    let total_seconds: f64 = step_times.iter().sum();
    let steady_seconds: f64 = warm.iter().sum();
    report["status"] = json!(status);
    report["supervisor_stop_requested"] =
        json!(cfg.stop_file.as_ref().is_some_and(|path| path.exists()));
    report["completed_step"] = json!(complete);
    report["new_optimizer_steps"] = json!(complete - begin);
    report["new_sampled_target_visits"] = json!((complete - begin) * cfg.batch * cfg.context);
    report["cumulative_sampled_target_visits"] = json!(complete * cfg.batch * cfg.context);
    report["final_retained_batch_nll"] = json!(final_fit);
    report["final_development_nll"] = json!(final_dev);
    report["final_shadow_development_nll"] = json!(final_shadow_dev);
    report["reloaded_retained_batch_nll"] = json!(reloaded_fit);
    report["reload_absolute_delta"] = json!(reload_delta);
    report["final_quantization"] = serde_json::to_value(model.quantization())?;
    if cfg.projection_transition.is_some() {
        report["projection_updates"] = json!({
            "complete_optimizer_updates_projected":projection_updates,
            "projected_coordinate_updates":projected_coordinate_updates,
            "maximum_absolute_correction":maximum_projection_correction,
            "scope":"Post-AdamW events; coordinate counts may count the same coordinate on multiple updates. Entry projection is reported separately."
        });
    }
    report["next_training_quantization_strength"] = json!(model.training_strength());
    report["training_complete_steps_seconds"] = json!(total_seconds);
    report["steady_complete_step_tokens_per_second"] = if steady_seconds > 0.0 {
        json!((warm.len() * cfg.batch * cfg.context) as f64 / steady_seconds)
    } else {
        Value::Null
    };
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    save_json(&out.join("fit-report.json"), &report)?;
    eprintln!("joint {status}: {complete} steps, retained {initial_fit:.5}->{final_fit:.5}, reload delta {reload_delta}");
    Ok(())
}

pub fn run_cli(args: &[String]) -> Result<()> {
    if args.first().map(String::as_str) == Some("joint-calibrate-shadow") {
        return calibrate_shadow_cli(args);
    }
    if args.first().map(String::as_str) == Some("joint-integer-tables") {
        if args.len() != 2 {
            return Err(invalid("joint-integer-tables NEW_REPORT_ROOT"));
        }
        return crate::joint_integer_tables::export(std::path::Path::new(&args[1]));
    }
    if crate::joint_integer_evaluation::run(args)? {
        return Ok(());
    }

    if args
        .first()
        .is_some_and(|s| matches!(s.as_str(), "joint-bound-fit" | "joint-evaluate-admission"))
    {
        return crate::joint_bounded_campaign::run_cli(args);
    }
    if args
        .first()
        .is_some_and(|s| matches!(s.as_str(), "joint-round-fit" | "joint-round-calibrate"))
    {
        return crate::joint_rounding_campaign::run_cli(args);
    }
    if args.first().map(String::as_str) == Some("joint-compare") {
        return crate::joint_comparison::run_cli(args);
    }
    if args.first().map(String::as_str) == Some("joint-evaluate-precision") {
        return evaluate_precision_cli(args);
    }
    if args.first().is_some_and(|mode| {
        matches!(
            mode.as_str(),
            "joint-evaluate" | "joint-evaluate-shadow" | "joint-evaluate-hard"
        )
    }) {
        return evaluate_cli(args);
    }
    if args.first().map(String::as_str) == Some("joint-export-hard") {
        return export_hard_cli(args);
    }
    if !(args.len() == 4 || args.len() == 5)
        || args.first().map(String::as_str) != Some("joint-fit")
    {
        return Err(invalid(
            "usage: joint-fit CAMPAIGN_JSON NEW_REPORT_ROOT {cpu|metal} [SEALED_RESUME_CHECKPOINT]",
        ));
    }
    if !["cpu", "metal"].contains(&args[3].as_str()) {
        return Err(invalid("joint device cpu|metal"));
    }
    let cfg = Campaign::load(Path::new(&args[1]))?;
    let out = Path::new(&args[2]);
    report_output::claim(out)?;
    let result = fit(&cfg, out, &args[3], args.get(4).map(Path::new));
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn load_tokenizer(evaluator: &Value) -> Result<HfBpeTokenizer> {
    let identities = evaluator["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("reference identity inventory"))?;
    let identity = identities
        .iter()
        .find(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("/tokenizer.json"))
        })
        .ok_or_else(|| invalid("tokenizer identity missing"))?;
    let path = verify_identity(identity)?;
    HfBpeTokenizer::from_dir(path.parent().ok_or_else(|| invalid("tokenizer parent"))?)
        .map_err(|e| invalid(format!("joint tokenizer: {e}")))
}

fn evaluate_cli(args: &[String]) -> Result<()> {
    if args.len() != 7 || !["cpu", "metal"].contains(&args[4].as_str()) {
        return Err(invalid("usage: joint-evaluate[-shadow] CAMPAIGN_JSON SEALED_CHECKPOINT NEW_REPORT_ROOT {cpu|metal} {read|no-read} BATCH; joint-evaluate-hard SEALED_CHECKPOINT_OR_EXPORT EVALUATOR_JSON NEW_REPORT_ROOT cpu {read|no-read} BATCH"));
    }
    let hard = args[0] == "joint-evaluate-hard";
    let shadow = args[0] == "joint-evaluate-shadow";
    if hard && args[4] != "cpu" {
        return Err(invalid("packed hard evaluation requires the CPU emulator"));
    }
    let mode = match args[5].as_str() {
        "read" => ReadMode::Enabled,
        "no-read" => ReadMode::NoRead,
        _ => return Err(invalid("evaluation read mode read|no-read")),
    };
    let batch: usize = args[6].parse().map_err(|_| invalid("evaluation batch"))?;
    if !(1..=joint_evaluation::MAX_EVALUATION_BATCH).contains(&batch) {
        return Err(invalid("evaluation batch1..32"));
    }
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = (|| {
        let selected = device(&args[4])?;
        if hard {
            let evaluator = load_evaluator(Path::new(&args[2]))?;
            let source = Path::new(&args[1]);
            let input = if source.join("hard-model.json").try_exists()? {
                load_hard_export(source, &evaluator)?
            } else {
                let parent = load_bound_checkpoint(source, &selected, &evaluator.sha256)?;
                if parent.model.quantization().is_none() {
                    return Err(invalid(
                        "continuous checkpoints require an explicit calibrated export campaign",
                    ));
                }
                let packed = out.join("packed-model");
                report_output::claim(&packed)?;
                let result = write_hard_export(&parent, source, &evaluator, &packed);
                finish_attempt(&packed, result)?
            };
            evaluate_loaded(
                &input, &evaluator, source, out, &args[0], &args[4], mode, batch,
            )
        } else {
            let cfg = Campaign::load(Path::new(&args[1]))?;
            let evaluator = load_evaluator(&cfg.evaluator_path)?;
            let source = Path::new(&args[2]);
            let mut input = load_bound_checkpoint(source, &selected, &evaluator.sha256)?;
            same_learning_configuration(&cfg, &input.campaign)?;
            input.campaign = cfg;
            if shadow {
                if input.model.quantization().is_none() {
                    return Err(invalid(
                        "shadow evaluation requires quantized training weights",
                    ));
                }
                input.model = input.model.without_quantization()?;
                input.artifact["numerical_path"] = json!(
                    "continuous shadow of the same quantized-training weights; quantizers disabled"
                );
            }
            evaluate_loaded(
                &input, &evaluator, source, out, &args[0], &args[4], mode, batch,
            )
        }
    })();
    finish_attempt(out, result)
}

/// A fixed-checkpoint numerical intervention, never a training or export mode.
fn evaluate_precision_cli(args: &[String]) -> Result<()> {
    if args.len() != 7 || args[4] != "cpu" {
        return Err(invalid("usage: joint-evaluate-precision SEALED_SHADOW_CHECKPOINT EVALUATOR_JSON NEW_REPORT_ROOT cpu {FF|QF|FQ|QQ} BATCH"));
    }
    let precision = PrecisionMode::parse(&args[5])?;
    let batch: usize = args[6].parse().map_err(|_| invalid("evaluation batch"))?;
    if !(1..=joint_evaluation::MAX_EVALUATION_BATCH).contains(&batch) {
        return Err(invalid("evaluation batch1..32"));
    }
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = (|| {
        let evaluator = load_evaluator(Path::new(&args[2]))?;
        let source = Path::new(&args[1]);
        let mut input =
            load_bound_checkpoint(source, &candle_core::Device::Cpu, &evaluator.sha256)?;
        input.model = input.model.precision_view(precision)?;
        input.artifact["precision_intervention"] = json!({
            "mode":precision,
            "quantize_parameters":precision.quantizes_parameters(),
            "quantize_interfaces":precision.quantizes_interfaces(),
            "optimizer_updates":0,
            "parameters_mutated":false,
            "scales_recalibrated":false,
            "scope":"Evaluation-only forward intervention on the same fully quantized-training shadows; mixed modes are not serving candidates."
        });
        input.artifact["numerical_path"] = json!(format!(
            "Evaluation-only {} view of the same floating checkpoint; parameter/interface switches are explicit in precision_intervention; execution remains F32",
            precision.name()
        ));
        save_json(&out.join("evaluator.json"), &evaluator.document)?;
        evaluate_loaded(
            &input,
            &evaluator,
            source,
            out,
            &args[0],
            "cpu",
            ReadMode::Enabled,
            batch,
        )
    })();
    finish_attempt(out, result)
}

pub(crate) struct BoundModel {
    pub(crate) campaign: Campaign,
    pub(crate) model: JointModel,
    pub(crate) binding: Value,
    pub(crate) artifact: Value,
}

pub(crate) fn finish_attempt<T>(out: &Path, result: Result<T>) -> Result<T> {
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn same_learning_configuration(a: &Campaign, b: &Campaign) -> Result<()> {
    if a.model != b.model
        || a.optimizer != b.optimizer
        || a.data_seed != b.data_seed
        || a.batch != b.batch
        || a.context != b.context
        || a.cpu_gradient_shards != b.cpu_gradient_shards
        || a.training_window_transition != b.training_window_transition
        || a.quantization_transition != b.quantization_transition
        || a.projection_transition != b.projection_transition
    {
        return Err(invalid(
            "evaluation campaign changes bound learning configuration",
        ));
    }
    Ok(())
}

pub(crate) fn load_bound_checkpoint(
    checkpoint: &Path,
    selected: &candle_core::Device,
    evaluator_sha: &str,
) -> Result<BoundModel> {
    report_output::verify(checkpoint)?;
    let cfg = Campaign::load(&checkpoint.join("campaign.json"))?;
    let binding: Value = serde_json::from_slice(&fs::read(checkpoint.join("checkpoint.json"))?)?;
    let step = binding["optimizer_step"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("checkpoint optimizer step"))?;
    if step > cfg.total_steps
        || binding["schema"] != "uor-r4.joint-recurrent-checkpoint/1"
        || binding["evaluator_sha256"] != evaluator_sha
        || binding["model_sha256"] != sha256_file(&checkpoint.join("model.safetensors"))?
        || (binding.get("model_config_sha256").is_some()
            && binding["model_config_sha256"] != sha256_file(&checkpoint.join("config.json"))?)
        || binding["next_data_step"] != json!(step)
        || binding["sampled_target_visits"] != json!(step.checked_mul(cfg.batch * cfg.context))
        || binding["data_seed"] != json!(cfg.data_seed)
        || binding["training_batch"] != json!(cfg.batch)
        || binding["training_context"] != json!(cfg.context)
        || binding["sampled_targets_per_step"] != json!(cfg.batch * cfg.context)
        || binding["training_window_transition"] != json!(cfg.training_window_transition)
        || binding["data_sampler"]
            != "splitmix64-counter-v1;valid-window-union;no-cross-store;step/lane-bound"
        || !binding["source_commit"]
            .as_str()
            .is_some_and(|s| is_hex_digest(s, 40))
        || binding
            .get("cpu_gradient_shards")
            .map_or(Some(1), Value::as_u64)
            != Some(cfg.cpu_gradient_shards as u64)
    {
        return Err(invalid("checkpoint/evaluator binding differs"));
    }
    let model = JointModel::load(checkpoint, selected)?;
    if model.config != cfg.model {
        return Err(invalid("evaluation campaign/model differs"));
    }
    validate_quantization_binding(
        &cfg,
        &binding,
        &serde_json::to_value(model.quantization())?,
        step,
    )?;
    let artifact = json!({"kind":"training_checkpoint", "parent_checkpoint":checkpoint,
        "checkpoint_sha256":sha256_file(&checkpoint.join("checkpoint.json"))?,
        "campaign_sha256":sha256_file(&checkpoint.join("campaign.json"))?,
        "model_config_sha256":sha256_file(&checkpoint.join("config.json"))?,
        "parent_file_set_verified":true,
        "numerical_path":if model.quantization().is_some() {"full-strength quantized numerical emulator from training checkpoint"} else {"continuous F32"}});
    Ok(BoundModel {
        campaign: cfg,
        model,
        binding,
        artifact,
    })
}

/// Calibrate from current continuous parameters, then clip only coordinates
/// outside legal intervals. Interior shadows retain their fractional choices.
fn prepare_rounding_shadow(
    model: &mut JointModel,
    optimizer: &NamedAdamW,
    step: usize,
) -> Result<Value> {
    if model.quantization().is_some()
        || model.admission_policy() != crate::joint_admission::AdmissionPolicy::Full
        || optimizer.step_count() as usize != step
        || step == 0
    {
        return Err(invalid("shadow preparation requires a continuous full-access parent at its actual optimizer clock"));
    }
    let optimizer_before = optimizer.continuity_fingerprint()?;
    let original = model
        .variables()
        .iter()
        .map(|(name, var)| {
            Ok((
                name.clone(),
                (var.id(), var.flatten_all()?.to_vec1::<f32>()?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    model.configure_rounding_calibration(step)?;
    let before = hard_parameter_snapshot(model)?;
    let state = model
        .quantization()
        .ok_or_else(|| invalid("missing fresh scales"))?;
    let spec_sha = quantization_spec_digest(&serde_json::to_value(state)?)?;
    let statistics = state.spec.project_parameters(model.variables())?;
    let after = hard_parameter_snapshot(model)?;
    let mut fractional_coordinates = 0usize;
    for (name, var) in model.variables() {
        let (id, initial) = &original[name];
        if *id != var.id() {
            return Err(invalid("calibration changed parameter identity"));
        }
        let parameter = &state.spec.parameters[name];
        let width = if parameter.shape.len() == 2 {
            parameter.shape[1]
        } else {
            parameter.shape[0]
        };
        let limit = if parameter.bits == 4 {
            7.0f32
        } else {
            32767.0f32
        };
        for (index, (&old, value)) in initial
            .iter()
            .zip(var.flatten_all()?.to_vec1::<f32>()?)
            .enumerate()
        {
            let scale = 2f32.powi(i32::from(parameter.row_exponents[index / width]));
            let bound = limit * scale;
            let expected = if old < -bound {
                -bound
            } else if old > bound {
                bound
            } else {
                old
            };
            if value.to_bits() != expected.to_bits() {
                return Err(invalid(
                    "calibration rounded an interior shadow or changed its legal projection",
                ));
            }
            if (value / scale).fract() != 0.0 {
                fractional_coordinates += 1;
            }
        }
    }
    if before.values_and_codes != after.values_and_codes
        || after.out_of_range_coordinates != 0
        || statistics.projected_coordinates != before.out_of_range_coordinates
        || optimizer.continuity_fingerprint()? != optimizer_before
    {
        return Err(invalid(
            "shadow calibration changed hard codes, moments or clocks",
        ));
    }
    Ok(json!({
        "schema":"uor-r4.calibrated-rounding-shadow/1",
        "preparation":"calibrated_for_rounding", "model_step":step,
        "optimizer_updates":0,"sampler_steps_advanced":0,"sampled_target_visits":0,
        "calibration_rule":crate::joint_quantization::CALIBRATION_RULE,
        "calibration_input":"Current continuous parameters only; no evaluation data or code selection",
        "quantization_spec_sha256":spec_sha,"projection":statistics,
        "fractional_shadow_coordinates":fractional_coordinates,
        "interior_shadow_bits_preserved":true,"nearest_hard_values_and_codes_preserved":true,
        "hard_codes_sha256":after.codes_sha256,
        "hard_values_sha256":after.hard_values_sha256,
        "continuous_shadow_sha256":before.shadow_sha256,
        "projected_shadow_sha256":after.shadow_sha256,
        "optimizer_moments_and_clocks":optimizer_before,
        "clock_scope":"start_step=completed_step=actual continuous parent step; ramp_steps=1 is unused compatibility metadata, not a completed QAT ramp; only alpha learning is permitted"
    }))
}

fn calibrate_shadow_cli(args: &[String]) -> Result<()> {
    if args.len() != 4 {
        return Err(invalid(
            "usage: joint-calibrate-shadow SEALED_CONTINUOUS_PARENT EVALUATOR_JSON NEW_REPORT_ROOT",
        ));
    }
    let source = Path::new(&args[1]);
    let evaluator_path = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = (|| {
        let started = Instant::now();
        let target = out.join("checkpoint-calibrated");
        report_output::claim(&target)?;
        let evaluator = load_evaluator(evaluator_path)?;
        let mut parent =
            load_bound_checkpoint(source, &candle_core::Device::Cpu, &evaluator.sha256)?;
        let step = parent.binding["optimizer_step"]
            .as_u64()
            .and_then(|step| usize::try_from(step).ok())
            .ok_or_else(|| invalid("calibration parent clock"))?;
        if step >= 100_000
            || parent.campaign.batch != 16
            || parent.campaign.context != 256
            || parent.model.config.context != 256
            || parent.campaign.quantization_transition.is_some()
            || parent.campaign.projection_transition.is_some()
        {
            return Err(invalid(
                "calibration requires the unchanged continuous B16/T256 campaign",
            ));
        }
        let optimizer =
            NamedAdamW::load(source, parent.model.variables(), &parent.campaign.optimizer)?;
        let mut calibration = prepare_rounding_shadow(&mut parent.model, &optimizer, step)?;
        calibration["original_continuous_artifact"] = parent.artifact.clone();
        calibration["original_continuous_binding"] = parent.binding.clone();
        calibration["original_optimizer_metadata_sha256"] = json!(sha256_file(
            &source.join(crate::joint_optimizer::METADATA_FILE)
        )?);
        calibration["original_optimizer_moments_sha256"] = json!(sha256_file(
            &source.join(crate::joint_optimizer::MOMENT_FILE)
        )?);
        let mut cfg = parent.campaign.clone();
        cfg.evaluator_path = evaluator_path.to_path_buf();
        cfg.total_steps = cfg.total_steps.max(step + 1);
        cfg.checkpoint_steps.clear();
        cfg.stop_file = None;
        cfg.quantization_transition = Some(QuantizationTransition {
            parent_checkpoint_sha256: sha256_file(&source.join("checkpoint.json"))?,
            parent_campaign_sha256: sha256_file(&source.join("campaign.json"))?,
            parent_optimizer_step: step,
            ramp_steps: 1,
            reason: "Explicit parameter-only calibration and legal-range shadow projection for alpha learning; zero model updates; one-step ramp field is unused preparation metadata".into(),
        });
        cfg.trial_scope = "CALIBRATED_SHADOW_NO_MODEL_UPDATES: original continuous parent retained; only alpha code learning may consume this checkpoint; total_steps retains unused schema headroom, not completed exposure".into();
        let mut report = metadata(&cfg, "joint-calibrate-shadow", "cpu")?;
        report["calibration"] = calibration.clone();
        save_checkpoint_with_calibration(
            &parent.model,
            &optimizer,
            &cfg,
            &target,
            step,
            "CALIBRATED_SHADOW_NO_MODEL_UPDATES",
            &evaluator.sha256,
            Some(&calibration),
        )?;
        let restored =
            load_bound_checkpoint(&target, &candle_core::Device::Cpu, &evaluator.sha256)?;
        let restored_optimizer =
            NamedAdamW::load(&target, restored.model.variables(), &cfg.optimizer)?;
        if hard_parameter_snapshot(&restored.model)?.shadow_sha256
            != calibration["projected_shadow_sha256"]
            || restored_optimizer.continuity_fingerprint()?
                != calibration["optimizer_moments_and_clocks"]
            || restored.binding["shadow_calibration"] != calibration
        {
            return Err(invalid(
                "calibrated shadow reload changed parameters, optimizer or provenance",
            ));
        }
        report["checkpoint"] = json!(target);
        report["checkpoint_sha256"] = json!(sha256_file(&target.join("checkpoint.json"))?);
        report["reload_shadow_optimizer_and_provenance_equal"] = json!(true);
        report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
        save_json(&out.join("calibration-report.json"), &report)?;
        Ok(())
    })();
    finish_attempt(out, result)
}

fn export_hard_cli(args: &[String]) -> Result<()> {
    if !(args.len() == 3 || args.len() == 4) {
        return Err(invalid("usage: joint-export-hard SEALED_CHECKPOINT NEW_REPORT_ROOT [INITIAL_QAT_CAMPAIGN_JSON]"));
    }
    let out = Path::new(&args[2]);
    report_output::claim(out)?;
    let result = (|| {
        let source = Path::new(&args[1]);
        report_output::verify(source)?;
        let cfg = Campaign::load(&source.join("campaign.json"))?;
        let evaluator = load_evaluator(&cfg.evaluator_path)?;
        let mut parent =
            load_bound_checkpoint(source, &candle_core::Device::Cpu, &evaluator.sha256)?;
        if let Some(campaign_path) = args.get(3) {
            let next = Campaign::load(Path::new(campaign_path))?;
            if next.projection_transition.is_some() {
                return Err(invalid(
                    "hard export cannot introduce a training projection policy",
                ));
            }
            if load_evaluator(&next.evaluator_path)?.sha256 != evaluator.sha256 {
                return Err(invalid("calibrated hard export changes evaluator"));
            }
            let step = parent.binding["optimizer_step"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| invalid("parent optimizer step"))?;
            if !quantization_resume_transition(
                &next,
                &parent.campaign,
                &sha256_file(&source.join("checkpoint.json"))?,
                &sha256_file(&source.join("campaign.json"))?,
                step,
            )? || parent.model.quantization().is_some()
            {
                return Err(invalid("optional export campaign only declares an initial continuous-parent calibration"));
            }
            let optimizer = NamedAdamW::load(source, parent.model.variables(), &next.optimizer)?;
            if optimizer.step_count() as usize != step {
                return Err(invalid("calibration parent optimizer clock differs"));
            }
            parent.model.configure_quantization(
                step,
                next.quantization_transition
                    .as_ref()
                    .ok_or_else(|| invalid("missing calibration transition"))?
                    .ramp_steps,
            )?;
            parent.campaign = next;
        }
        if parent.model.quantization().is_none() {
            return Err(invalid(
                "continuous export requires its declared quantization campaign",
            ));
        }
        write_hard_export(&parent, source, &evaluator, out).map(|_| ())
    })();
    finish_attempt(out, result)
}

pub(crate) fn write_hard_export(
    parent: &BoundModel,
    source: &Path,
    evaluator: &Evaluator,
    out: &Path,
) -> Result<BoundModel> {
    let mut report = metadata(&parent.campaign, "joint-export-hard", "cpu")?;
    let state = serde_json::to_value(parent.model.quantization())?;
    let step = state["completed_step"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("hard export requires a quantization clock"))?;
    quantization_state_matches(&parent.campaign, &state, step)?;
    projection_state_matches(&parent.campaign, &state, step)?;
    let manifest = parent.model.save_hard(out)?;
    let manifest_sha = sha256_file(&out.join("hard-model.json"))?;
    let mut binding = parent.binding.clone();
    binding["model_sha256"] = json!(manifest_sha);
    binding["quantization_transition"] = json!(parent.campaign.quantization_transition);
    binding["quantization"] = state;
    binding["artifact_kind"] = json!("packed_quantized_parameters_with_float_numerical_emulator");
    if let Some(object) = binding.as_object_mut() {
        object.remove("model_config_sha256");
    }
    let artifact = json!({"kind":"packed_hard_export", "hard_model_manifest_sha256":manifest_sha,
        "hard_model_manifest":manifest,"parent_checkpoint":source,
        "parent_checkpoint_sha256":sha256_file(&source.join("checkpoint.json"))?,
        "parent_campaign_sha256":sha256_file(&source.join("campaign.json"))?,
        "parent_checkpoint_binding":parent.binding,"parent_artifact":parent.artifact,
        "parent_file_set_verified_at_export":true,
        "optimizer_updates_during_export":0,
        "numerical_path":"Full-strength packed-parameter numerical emulator; floating matmul, normalization and nonlinearities remain. This is not an integer serving kernel."});
    let mut restored = JointModel::load_hard(out, &candle_core::Device::Cpu)?;
    if restored.config != parent.model.config
        || restored.quantization() != parent.model.quantization()
        || restored.admission_policy() != parent.model.admission_policy()
    {
        return Err(invalid(
            "hard export reload changed configuration or quantization state",
        ));
    }
    validate_quantization_binding(
        &parent.campaign,
        &binding,
        &serde_json::to_value(restored.quantization())?,
        step,
    )?;
    let tokenizer = load_tokenizer(&evaluator.document)?;
    let expected_smoke = joint_evaluation::generate(
        &parent.model,
        &tokenizer,
        "Once upon a time, there was a little girl who",
        ReadMode::Enabled,
        Some(240924),
        48,
    )?;
    let dev = read_tokens(&evaluator.document["dev_source"])?;
    if dev.len() < 256 {
        return Err(invalid("hard reload check requires a full context"));
    }
    let inputs: Vec<u32> = dev[..256].iter().map(|&id| u32::from(id)).collect();
    let expected_probabilities = parent
        .model
        .forward(&inputs, 1, 256, ReadMode::Enabled, false)?
        .probabilities;
    let loaded_probabilities = restored
        .forward(&inputs, 1, 256, ReadMode::Enabled, false)?
        .probabilities;
    let reload_delta = expected_probabilities
        .sub(&loaded_probabilities)?
        .abs()?
        .max_all()?
        .to_scalar::<f32>()?;
    if reload_delta != 0.0 {
        return Err(invalid(
            "packed reload changes actual full-context hard probabilities",
        ));
    }
    restored.enable_interface_audit();
    let smoke = joint_evaluation::generate(
        &restored,
        &tokenizer,
        "Once upon a time, there was a little girl who",
        ReadMode::Enabled,
        Some(240924),
        48,
    )?;
    if smoke.generated_token_ids != expected_smoke.generated_token_ids
        || smoke
            .decisions
            .iter()
            .map(|row| &row.probabilities_sha256_le_f32)
            .collect::<Vec<_>>()
            != expected_smoke
                .decisions
                .iter()
                .map(|row| &row.probabilities_sha256_le_f32)
                .collect::<Vec<_>>()
    {
        return Err(invalid(
            "packed reload changes generated IDs or probabilities",
        ));
    }
    save_json(&out.join("pre-export-generation.json"), &expected_smoke)?;
    save_json(&out.join("loaded-generation.json"), &smoke)?;
    save_json(&out.join("campaign.json"), &parent.campaign)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    report["status"] = json!("PACKED_EXPORT_RELOADED");
    report["checkpoint_binding"] = binding.clone();
    report["artifact"] = artifact.clone();
    report["evaluator_sha256"] = json!(evaluator.sha256);
    report["optimizer_steps"] = json!(0);
    report["hard_reload_full_context_probability_max_absolute_delta"] = json!(reload_delta);
    report["hard_reload_generated_ids_and_probability_hashes_equal"] = json!(true);
    report["interface_audit"] = restored.interface_audit()?;
    report["interface_audit_scope"] = json!("Only the recorded 48-token-cap loaded smoke generation; not a corpus-wide saturation bound. Audit observes pre-quantization interfaces of the hard numerical emulator.");
    report["quantized_training_updates"] = json!(
        step - parent
            .campaign
            .quantization_transition
            .as_ref()
            .ok_or_else(|| invalid("export transition missing"))?
            .parent_optimizer_step
    );
    save_json(&out.join("hard-export-report.json"), &report)?;
    // The smoke audit performs host observations. Do not retain it in the full
    // population evaluator returned to a checkpoint-to-packed evaluation call.
    let restored = JointModel::load_hard(out, &candle_core::Device::Cpu)?;
    Ok(BoundModel {
        campaign: parent.campaign.clone(),
        model: restored,
        binding,
        artifact,
    })
}

pub(crate) fn load_hard_export(source: &Path, evaluator: &Evaluator) -> Result<BoundModel> {
    report_output::verify(source)?;
    let report: Value = serde_json::from_slice(&fs::read(source.join("hard-export-report.json"))?)?;
    let cfg = Campaign::load(&source.join("campaign.json"))?;
    let binding = report["checkpoint_binding"].clone();
    if report["status"] != "PACKED_EXPORT_RELOADED"
        || report["mode"] != "joint-export-hard"
        || report["evaluator_sha256"] != evaluator.sha256
        || report["campaign"] != serde_json::to_value(&cfg)?
        || binding["evaluator_sha256"] != evaluator.sha256
        || binding["model_sha256"] != sha256_file(&source.join("hard-model.json"))?
        || report["artifact"]["hard_model_manifest_sha256"] != binding["model_sha256"]
    {
        return Err(invalid(
            "hard export campaign/evaluator/manifest binding differs",
        ));
    }
    let model = JointModel::load_hard(source, &candle_core::Device::Cpu)?;
    let step = binding["optimizer_step"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("hard export optimizer clock"))?;
    if model.config != cfg.model {
        return Err(invalid("hard export model differs from campaign"));
    }
    validate_quantization_binding(
        &cfg,
        &binding,
        &serde_json::to_value(model.quantization())?,
        step,
    )?;
    Ok(BoundModel {
        campaign: cfg,
        model,
        binding,
        artifact: report["artifact"].clone(),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_loaded(
    input: &BoundModel,
    evaluator: &Evaluator,
    checkpoint: &Path,
    out: &Path,
    operation: &str,
    device_name: &str,
    mode: ReadMode,
    batch: usize,
) -> Result<()> {
    let model = &input.model;
    let mut report = metadata(&input.campaign, operation, device_name)?;
    report["artifact"] = input.artifact.clone();
    report["training_projection_transition"] = json!(input.campaign.projection_transition);
    report["parameter_projection_during_evaluation"] = json!(false);
    report["executed_quantization"] = serde_json::to_value(model.quantization())?;
    report["evaluation_quantization_strength"] = json!(if model.quantization().is_some() {
        1.0
    } else {
        0.0
    });
    if let Some(precision) = model.precision_mode() {
        report["precision_intervention"] = input.artifact["precision_intervention"].clone();
        report["executed_numerical_contract"] = model.numerical_contract();
        report["executed_quantization_scope"] = json!(
            "Immutable retained quantizer metadata; the actual parameter/interface switches are recorded separately in precision_intervention."
        );
        match precision {
            PrecisionMode::FF => report["evaluation_quantization_strength"] = json!(0.0),
            PrecisionMode::QQ => report["evaluation_quantization_strength"] = json!(1.0),
            PrecisionMode::QF | PrecisionMode::FQ => {
                // One scalar cannot describe two independently selected grids.
                // Existing commands retain their original numerical field.
                if let Some(fields) = report.as_object_mut() {
                    fields.remove("evaluation_quantization_strength");
                }
            }
        }
    }
    let tokenizer = load_tokenizer(&evaluator.document)?;
    let prompts_path = verify_identity(&evaluator.document["prompt_source"])?;
    let prompts: Value = serde_json::from_slice(&fs::read(prompts_path)?)?;
    let mut generated = Vec::new();
    for (i, prompt) in prompts["prompts"]
        .as_array()
        .ok_or_else(|| invalid("prompts"))?
        .iter()
        .enumerate()
    {
        generated.push(joint_evaluation::generate(
            model,
            &tokenizer,
            prompt["text"]
                .as_str()
                .ok_or_else(|| invalid("prompt text"))?,
            mode,
            Some(2014 + i as u64),
            128,
        )?);
    }
    save_json(&out.join("generations.json"), &generated)?;
    save_json(
        &out.join("story-probes.json"),
        &joint_evaluation::run_story_probes(model, &tokenizer, mode)?,
    )?;
    let dev = read_tokens(&evaluator.document["dev_source"])?;
    let mut blocks = BufWriter::new(File::create_new(out.join("blocks.jsonl"))?);
    let mut rows = BufWriter::new(File::create_new(out.join("targets.bin"))?);
    let evaluation = joint_evaluation::evaluate(model, &dev, mode, batch, |block, predictions| {
        serde_json::to_writer(&mut blocks, block)?;
        writeln!(blocks)?;
        for row in predictions {
            rows.write_all(&(row.input_offset as u64).to_le_bytes())?;
            rows.write_all(&row.score.target_token.to_le_bytes())?;
            rows.write_all(&row.score.predicted_token.to_le_bytes())?;
            rows.write_all(&row.score.target_probability.to_le_bytes())?;
            rows.write_all(&row.score.nll_nats.to_le_bytes())?;
            rows.write_all(&(row.no_read_mass as f32).to_le_bytes())?;
            rows.write_all(&(row.copy_gate as f32).to_le_bytes())?;
            rows.write_all(&row.top_read_position.map_or(-1, |p| p as i32).to_le_bytes())?;
        }
        if block.block_index % 64 == 0 {
            eprintln!(
                "joint evaluate {:?} block {}/976",
                mode,
                block.block_index + 1
            );
        }
        Ok(())
    })?;
    blocks.flush()?;
    blocks.get_ref().sync_all()?;
    rows.flush()?;
    rows.get_ref().sync_all()?;
    report["evaluation"] = serde_json::to_value(evaluation)?;
    report["checkpoint"] = json!(checkpoint);
    report["checkpoint_binding"] = input.binding.clone();
    report["evaluator_sha256"] = json!(evaluator.sha256);
    report["targets_format"] = json!({"record_bytes":44,"endianness":"little",
        "fields":["input_offset:u64","target:u32","predicted:u32","target_probability:f64",
        "nll_nats:f64","no_read_mass:f32","copy_gate:f32","top_read_position:i32(-1=None)"],
        "order":"corpus order; target_offset=input_offset+1; same256context reset as evaluator v2"});
    report["targets_sha256"] = json!(sha256_file(&out.join("targets.bin"))?);
    report["admission_policy"] = json!(model.admission_policy());
    report["admission_audit"] = model.admission_audit()?;
    report["executed_numerical_contract"] = model.numerical_contract();
    save_json(&out.join("evaluation-report.json"), &report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrated_shadow_preserves_fractional_weights_clocks_and_packed_metadata() -> Result<()> {
        use crate::joint_rounding::{LearnedRounding, RoundingConfig};
        use candle_core::{Device, Tensor};
        let mut model = JointModel::new(
            JointConfig {
                width: 128,
                context: 256,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        let mut optimizer = NamedAdamW::new(model.variables(), AdamConfig::default())?;
        let terms = model
            .variables()
            .values()
            .map(|v| v.sqr()?.sum_all())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let loss = Tensor::stack(&terms, 0)?.sum_all()?;
        optimizer.step(model.variables(), &loss.backward()?)?;
        let embedding = &model.variables()["embedding.weight"];
        let mut values = embedding.flatten_all()?.to_vec1::<f32>()?;
        values[..128].fill(0.1);
        values[0] = 1.0;
        embedding.set(&Tensor::from_vec(values, embedding.dims(), &Device::Cpu)?)?;
        let moments = optimizer.continuity_fingerprint()?;
        let witness = prepare_rounding_shadow(&mut model, &optimizer, 1)?;
        assert!(witness["projection"]["projected_coordinates"]
            .as_u64()
            .is_some_and(|n| n > 0));
        assert!(witness["fractional_shadow_coordinates"]
            .as_u64()
            .is_some_and(|n| n > 0));
        assert_eq!(moments, optimizer.continuity_fingerprint()?);
        assert!(model.set_completed_step(2).is_err());
        assert!(model
            .forward(&[1, 2], 1, 2, ReadMode::Enabled, true)
            .is_err());
        let state = model
            .quantization()
            .ok_or_else(|| invalid("test calibration missing"))?;
        assert_eq!(
            (state.start_step, state.completed_step, state.ramp_steps),
            (1, 1, 1)
        );
        let learner = LearnedRounding::new(
            &state.spec,
            model.variables(),
            RoundingConfig {
                steps: 2,
                warmup_steps: 1,
                beta_start: 20.0,
                beta_end: 2.0,
                regularization: 0.0,
            },
        )?;
        let view = model
            .rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
        let alpha_loss = view
            .forward(&[1, 2], 1, 2, ReadMode::Enabled, true)?
            .loss(&[2, 3])?;
        assert!(alpha_loss.to_scalar::<f32>()?.is_finite());
        let alpha_gradients = alpha_loss.backward()?;
        let mut nonzero_alpha_gradient = false;
        for variable in learner.variables().values() {
            if let Some(gradient) = alpha_gradients.get(variable) {
                let magnitude = gradient.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(magnitude.is_finite());
                nonzero_alpha_gradient |= magnitude > 0.0;
            }
        }
        assert!(nonzero_alpha_gradient);
        let hard = model.materialize_rounding_codes(learner.parameters(true)?)?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock"))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-calibrated-shadow-{}-{nonce}",
            std::process::id()
        ));
        report_output::claim(&root)?;
        let packed = root.join("packed");
        report_output::claim(&packed)?;
        let manifest = hard.save_hard(&packed)?;
        report_output::seal(&packed)?;
        let loaded = JointModel::load_hard(&packed, &Device::Cpu)?;
        assert_eq!(loaded.quantization(), hard.quantization());
        // Exercise the exact metadata type and clock predicate used by integer loading.
        let integer_state: uor_r4_integer::config::QuantizedTrainingState =
            serde_json::from_value(manifest["quantization"].clone())?;
        assert_eq!(
            serde_json::to_value(&integer_state)?,
            manifest["quantization"]
        );
        assert!(uor_r4_integer::config::valid_quantization_clock(
            integer_state.start_step,
            integer_state.ramp_steps,
            integer_state.completed_step,
            integer_state.preparation
        ));
        let checkpoint = root.join("checkpoint");
        report_output::claim(&checkpoint)?;
        let mut cfg = transition_campaign();
        cfg.model = model.config.clone();
        cfg.total_steps = 2;
        cfg.quantization_transition = Some(QuantizationTransition {
            parent_checkpoint_sha256: "a".repeat(64),
            parent_campaign_sha256: "b".repeat(64),
            parent_optimizer_step: 1,
            ramp_steps: 1,
            reason: "test no-update calibration".into(),
        });
        save_checkpoint_with_calibration(
            &model,
            &optimizer,
            &cfg,
            &checkpoint,
            1,
            "CALIBRATED_SHADOW_NO_MODEL_UPDATES",
            &"c".repeat(64),
            Some(&witness),
        )?;
        let restored = load_bound_checkpoint(&checkpoint, &Device::Cpu, &"c".repeat(64))?;
        let restored_optimizer =
            NamedAdamW::load(&checkpoint, restored.model.variables(), &cfg.optimizer)?;
        assert_eq!(restored_optimizer.continuity_fingerprint()?, moments);
        assert_eq!(
            hard_parameter_snapshot(&restored.model)?.shadow_sha256,
            witness["projected_shadow_sha256"]
        );
        let mut missing = restored.binding.clone();
        missing
            .as_object_mut()
            .ok_or_else(|| invalid("test binding object"))?
            .remove("shadow_calibration");
        assert!(validate_quantization_binding(
            &cfg,
            &missing,
            &serde_json::to_value(restored.model.quantization())?,
            1
        )
        .is_err());
        report_output::seal(&root)?;
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn calibrated_shadow_mode_preserves_legacy_serialization_and_rejects_false_clocks() -> Result<()>
    {
        use uor_r4_integer::config::{valid_quantization_clock, QuantizationPreparation};
        let mut model = JointModel::new(
            JointConfig {
                width: 128,
                ..JointConfig::default()
            },
            &candle_core::Device::Cpu,
        )?;
        model.configure_quantization(7, 4)?;
        let state = serde_json::to_value(model.quantization())?;
        assert!(state.get("preparation").is_none());
        let roundtrip: crate::joint_model::QuantizedTrainingState =
            serde_json::from_value(state.clone())?;
        assert_eq!(serde_json::to_value(roundtrip)?, state);
        assert!(model.materialize_rounding_codes(BTreeMap::new()).is_err());
        assert!(valid_quantization_clock(7, 4, 7, None));
        assert!(valid_quantization_clock(
            7,
            1,
            7,
            Some(QuantizationPreparation::CalibratedForRounding)
        ));
        assert!(!valid_quantization_clock(
            7,
            1,
            8,
            Some(QuantizationPreparation::CalibratedForRounding)
        ));
        assert!(!valid_quantization_clock(
            7,
            2,
            7,
            Some(QuantizationPreparation::CalibratedForRounding)
        ));
        let mut invalid_mode = state;
        invalid_mode["preparation"] = json!("completed_without_updates");
        assert!(
            serde_json::from_value::<crate::joint_model::QuantizedTrainingState>(
                invalid_mode.clone()
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<uor_r4_integer::config::QuantizedTrainingState>(invalid_mode)
                .is_err()
        );
        Ok(())
    }

    fn transition_campaign() -> Campaign {
        Campaign {
            schema: "uor-r4.joint-recurrent-campaign/1".into(),
            evaluator_path: PathBuf::new(),
            model: JointConfig::default(),
            optimizer: AdamConfig::default(),
            data_seed: 42,
            batch: 16,
            context: 256,
            cpu_gradient_shards: 2,
            total_steps: 8348,
            development_every_steps: 0,
            development_blocks: 0,
            checkpoint_steps: vec![],
            max_process_seconds: 100,
            stop_file: None,
            training_window_transition: None,
            quantization_transition: None,
            projection_transition: None,
            trial_scope: "transition unit check".into(),
        }
    }

    #[test]
    fn quantization_transition_binds_parent_and_preserves_resume_lineage() {
        let old = transition_campaign();
        let mut next = old.clone();
        next.quantization_transition = Some(QuantizationTransition {
            parent_checkpoint_sha256: "a".repeat(64),
            parent_campaign_sha256: "b".repeat(64),
            parent_optimizer_step: 7324,
            ramp_steps: 256,
            reason: "fixed calibration".into(),
        });
        assert!(quantization_resume_transition(
            &next,
            &old,
            &"a".repeat(64),
            &"b".repeat(64),
            7324
        )
        .unwrap());
        assert!(quantization_resume_transition(
            &next,
            &old,
            &"c".repeat(64),
            &"b".repeat(64),
            7324
        )
        .is_err());
        assert!(quantization_resume_transition(
            &next,
            &old,
            &"a".repeat(64),
            &"b".repeat(64),
            7325
        )
        .is_err());
        assert!(!quantization_resume_transition(
            &next,
            &next,
            "later checkpoint",
            "later campaign",
            7340
        )
        .unwrap());
        let mut altered = next.clone();
        altered.quantization_transition.as_mut().unwrap().ramp_steps = 128;
        assert!(quantization_resume_transition(&altered, &next, "unused", "unused", 7340).is_err());
        assert!(quantization_resume_transition(&old, &next, "unused", "unused", 7340).is_err());
        let mut changed_horizon = next.clone();
        changed_horizon.batch = 64;
        changed_horizon.context = 64;
        assert!(
            quantization_resume_transition(&changed_horizon, &next, "unused", "unused", 7340)
                .is_err()
        );
    }

    #[test]
    fn quantization_checkpoint_binds_clock_and_complete_specification() {
        let mut cfg = transition_campaign();
        cfg.quantization_transition = Some(QuantizationTransition {
            parent_checkpoint_sha256: "a".repeat(64),
            parent_campaign_sha256: "b".repeat(64),
            parent_optimizer_step: 7324,
            ramp_steps: 256,
            reason: "fixed calibration".into(),
        });
        let state = json!({"start_step":7324,"ramp_steps":256,"completed_step":7340,
            "spec":{"schema":"unit-test","parameters":{"weight":{"row_exponents":[-7]}}}});
        let binding =
            json!({"quantization_transition":cfg.quantization_transition,"quantization":state});
        validate_quantization_binding(&cfg, &binding, &state, 7340).unwrap();
        assert!(validate_quantization_binding(&cfg, &binding, &state, 7341).is_err());
        let mut altered = state.clone();
        altered["spec"]["parameters"]["weight"]["row_exponents"] = json!([-6]);
        assert!(validate_quantization_binding(&cfg, &binding, &altered, 7340).is_err());
        assert!(
            validate_quantization_binding(&transition_campaign(), &binding, &state, 7340).is_err()
        );
        validate_quantization_binding(&transition_campaign(), &json!({}), &Value::Null, 7340)
            .unwrap();
    }

    fn projection_fixture() -> (Campaign, Campaign, Value) {
        let mut old = transition_campaign();
        old.quantization_transition = Some(QuantizationTransition {
            parent_checkpoint_sha256: "a".repeat(64),
            parent_campaign_sha256: "b".repeat(64),
            parent_optimizer_step: 7324,
            ramp_steps: 256,
            reason: "original frozen quantizer".into(),
        });
        let state = json!({"start_step":7324,"ramp_steps":256,"completed_step":7836,
            "spec":{"schema":crate::joint_quantization::SPEC_SCHEMA,
                "parameters":{"embedding.weight":{"shape":[1,2],"bits":4,"row_exponents":[0]}}}});
        let mut projected = old.clone();
        projected.projection_transition = Some(ProjectionTransition {
            policy: ProjectionPolicy::FixedRepresentableRange,
            parent_checkpoint_sha256: "c".repeat(64),
            parent_campaign_sha256: "d".repeat(64),
            parent_optimizer_step: 7836,
            quantization_spec_sha256: quantization_spec_digest(&state).unwrap(),
            reason: "fixed-range projection from the full-hard midpoint".into(),
        });
        (old, projected, state)
    }

    #[test]
    fn projection_transition_rejects_toggle_rebinding_ramp_and_learning_changes() {
        let (old, projected, state) = projection_fixture();
        let checkpoint_sha = "c".repeat(64);
        let campaign_sha = "d".repeat(64);
        assert!(projection_resume_transition(
            &projected,
            &old,
            &checkpoint_sha,
            &campaign_sha,
            &state,
            7836,
        )
        .unwrap());
        for (checkpoint, campaign) in [
            ("wrong", campaign_sha.as_str()),
            (checkpoint_sha.as_str(), "wrong"),
        ] {
            assert!(projection_resume_transition(
                &projected, &old, checkpoint, campaign, &state, 7836
            )
            .is_err());
        }
        let mut resumed_state = state.clone();
        resumed_state["completed_step"] = json!(7840);
        assert!(!projection_resume_transition(
            &projected,
            &projected,
            "later checkpoint",
            "same policy",
            &resumed_state,
            7840,
        )
        .unwrap());
        assert!(projection_resume_transition(
            &old,
            &projected,
            "unused",
            "unused",
            &resumed_state,
            7840
        )
        .is_err());
        let mut rebound = projected.clone();
        rebound
            .projection_transition
            .as_mut()
            .unwrap()
            .parent_checkpoint_sha256 = "e".repeat(64);
        assert!(projection_resume_transition(
            &rebound,
            &projected,
            "unused",
            "unused",
            &resumed_state,
            7840
        )
        .is_err());
        let mut wrong_scales = state.clone();
        wrong_scales["spec"]["parameters"]["embedding.weight"]["row_exponents"] = json!([-1]);
        assert!(projection_resume_transition(
            &projected,
            &old,
            &checkpoint_sha,
            &campaign_sha,
            &wrong_scales,
            7836
        )
        .is_err());
        let mut ramp = projected.clone();
        ramp.projection_transition
            .as_mut()
            .unwrap()
            .parent_optimizer_step = 7400;
        let mut ramp_state = state.clone();
        ramp_state["completed_step"] = json!(7400);
        assert!(projection_resume_transition(
            &ramp,
            &old,
            &checkpoint_sha,
            &campaign_sha,
            &ramp_state,
            7400
        )
        .is_err());
        for field in [
            "model",
            "optimizer",
            "data_seed",
            "batch",
            "context",
            "cpu_gradient_shards",
            "quantization_transition",
        ] {
            let mut altered = projected.clone();
            match field {
                "model" => altered.model.seed += 1,
                "optimizer" => altered.optimizer.learning_rate *= 2.0,
                "data_seed" => altered.data_seed += 1,
                "batch" => altered.batch = 32,
                "context" => altered.context = 128,
                "cpu_gradient_shards" => altered.cpu_gradient_shards = 1,
                "quantization_transition" => altered
                    .quantization_transition
                    .as_mut()
                    .unwrap()
                    .reason
                    .push_str(" altered"),
                _ => unreachable!(),
            }
            assert!(
                projection_resume_transition(
                    &altered,
                    &old,
                    &checkpoint_sha,
                    &campaign_sha,
                    &state,
                    7836
                )
                .is_err(),
                "initial {field}"
            );
            assert!(
                projection_resume_transition(
                    &altered,
                    &projected,
                    "unused",
                    "unused",
                    &resumed_state,
                    7840
                )
                .is_err(),
                "resume {field}"
            );
        }
        let binding = json!({"projection_transition":projected.projection_transition});
        validate_projection_binding(&projected, &binding, &state, 7836).unwrap();
        assert!(validate_projection_binding(&projected, &json!({}), &state, 7836).is_err());
        assert!(validate_projection_binding(&old, &binding, &state, 7836).is_err());
    }

    #[test]
    fn projection_config_is_explicit_and_legacy_serialization_stays_absent() {
        let (old, projected, _) = projection_fixture();
        let legacy = serde_json::to_value(&old).unwrap();
        assert!(legacy.get("projection_transition").is_none());
        let decoded: Campaign = serde_json::from_value(legacy.clone()).unwrap();
        assert!(decoded.projection_transition.is_none());
        assert_eq!(serde_json::to_value(decoded).unwrap(), legacy);
        let mut misspelled = legacy.clone();
        misspelled["projection_transiton"] = json!(projected.projection_transition);
        assert!(serde_json::from_value::<Campaign>(misspelled).is_err());
        let mut bad_policy = serde_json::to_value(&projected).unwrap();
        bad_policy["projection_transition"]["policy"] = json!("clip_and_reset_moments");
        assert!(serde_json::from_value::<Campaign>(bad_policy).is_err());
        let mut unknown = serde_json::to_value(projected).unwrap();
        unknown["projection_transition"]["reset_optimizer"] = json!(false);
        assert!(serde_json::from_value::<Campaign>(unknown).is_err());
    }

    #[test]
    fn projection_preserves_named_moments_clocks_ids_and_sampler_cursor() -> Result<()> {
        use crate::joint_quantization::{ParameterQuantization, SPEC_SCHEMA};
        use candle_core::{Device, Var};
        let variables = BTreeMap::from([
            (
                "embedding.weight".into(),
                Var::from_vec(vec![9_f32, -9.0], (1, 2), &Device::Cpu)?,
            ),
            (
                "read.age".into(),
                Var::from_vec(vec![40000_f32], (1,), &Device::Cpu)?,
            ),
        ]);
        let config = AdamConfig {
            allowed_missing_gradients: std::collections::BTreeSet::from(["read.age".into()]),
            ..AdamConfig::default()
        };
        let mut optimizer = NamedAdamW::new(&variables, config)?;
        let gradients = variables["embedding.weight"].sqr()?.sum_all()?.backward()?;
        optimizer.step(&variables, &gradients)?;
        let before = optimizer.continuity_fingerprint()?;
        assert_eq!(before["step"], 1);
        assert_eq!(before["variables"][0]["updates"], 1);
        assert_eq!(before["variables"][1]["updates"], 0);
        let spec = QuantizationSpec {
            schema: SPEC_SCHEMA.into(),
            parameters: BTreeMap::from([
                (
                    "embedding.weight".into(),
                    ParameterQuantization {
                        shape: vec![1, 2],
                        bits: 4,
                        row_exponents: vec![0],
                    },
                ),
                (
                    "read.age".into(),
                    ParameterQuantization {
                        shape: vec![1],
                        bits: 16,
                        row_exponents: vec![0],
                    },
                ),
            ]),
        };
        let cfg = transition_campaign();
        let stores = vec![(0..1024).collect::<Vec<u16>>()];
        let cursor = optimizer.step_count() as usize;
        let expected_next_batch = training_batch(&stores, &cfg, cursor)?;
        let ids: Vec<_> = variables.values().map(|v| v.id()).collect();
        let statistics = spec.project_parameters(&variables)?;
        assert_eq!(statistics.projected_coordinates, 3);
        assert_eq!(
            variables["embedding.weight"]
                .flatten_all()?
                .to_vec1::<f32>()?,
            vec![7.0, -7.0]
        );
        assert_eq!(variables["read.age"].to_vec1::<f32>()?, vec![32767.0]);
        assert_eq!(ids, variables.values().map(|v| v.id()).collect::<Vec<_>>());
        assert_eq!(before, optimizer.continuity_fingerprint()?);
        assert_eq!(
            expected_next_batch,
            training_batch(&stores, &cfg, optimizer.step_count() as usize)?
        );
        // The same named optimizer accepts the projected Vars on its next update.
        let loss = (variables["embedding.weight"].sqr()?.sum_all()?
            + variables["read.age"].sqr()?.sum_all()?)?;
        optimizer.step(&variables, &loss.backward()?)?;
        let continued = optimizer.continuity_fingerprint()?;
        assert_eq!(continued["step"], 2);
        assert_eq!(continued["variables"][0]["updates"], 2);
        assert_eq!(continued["variables"][1]["updates"], 1);
        Ok(())
    }

    #[test]
    fn counter_windows_resume_without_crossing_store_boundaries() {
        let mut cfg = Campaign {
            schema: "uor-r4.joint-recurrent-campaign/1".into(),
            evaluator_path: PathBuf::new(),
            model: JointConfig::default(),
            optimizer: AdamConfig::default(),
            data_seed: 42,
            batch: 4,
            context: 8,
            cpu_gradient_shards: 1,
            total_steps: 10,
            development_every_steps: 0,
            development_blocks: 0,
            checkpoint_steps: vec![],
            max_process_seconds: 100,
            stop_file: None,
            training_window_transition: None,
            quantization_transition: None,
            projection_transition: None,
            trial_scope: "sampler".into(),
        };
        let stores = vec![
            (0..64).collect::<Vec<u16>>(),
            (1000..1100).collect::<Vec<u16>>(),
        ];
        let (x, y) = training_batch(&stores, &cfg, 7).unwrap();
        assert_eq!(
            (x.clone(), y.clone()),
            training_batch(&stores, &cfg, 7).unwrap()
        );
        for i in 0..x.len() {
            assert_eq!(y[i], x[i] + 1);
        }
        cfg.data_seed += 1;
        assert_ne!((x, y), training_batch(&stores, &cfg, 7).unwrap());
    }
}
