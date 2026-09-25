//! One persistent offline recurrent-memory learning campaign.
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::baseline_protocol::{
    device, load_evaluator, read_tokens, save_json, verify_identity, Evaluator,
};
use crate::joint_evaluation::{self, STORY_PROBE_SCOPE, STORY_STOP_POLICY};
use crate::joint_model::{JointConfig, JointModel, ReadMode};
use crate::joint_optimizer::{AdamConfig, NamedAdamW};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        Ok(cfg)
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
    quantization_state_matches(cfg, state, step)
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

fn metadata(cfg: &Campaign, mode: &str, device_name: &str) -> Result<Value> {
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
fn training_batch(
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

fn save_checkpoint(
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    directory: &Path,
    step: usize,
    status: &str,
    evaluator_sha: &str,
) -> Result<()> {
    let quantization = serde_json::to_value(model.quantization())?;
    quantization_state_matches(cfg, &quantization, step)?;
    if optimizer.step_count() as usize != step {
        return Err(invalid("checkpoint optimizer and model clocks differ"));
    }
    model.save(directory)?;
    optimizer.save(directory)?;
    save_json(&directory.join("campaign.json"), cfg)?;
    save_json(
        &directory.join("checkpoint.json"),
        &json!({
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
        }),
    )?;
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
    if (cfg.training_window_transition.is_some() || cfg.quantization_transition.is_some())
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
        model.set_completed_step(step + 1)?;
        let elapsed = step_started.elapsed().as_secs_f64();
        step_times.push(elapsed);
        complete = step + 1;
        let mut row = json!({"step":complete,"sampled_target_visits":complete*cfg.batch*cfg.context,
            "batch_mean_nll":value,"complete_step_seconds":elapsed,"optimizer":update,
            "cpu_gradient_shards":cfg.cpu_gradient_shards,
            "training_quantization_strength":quantization_strength,
            "development_numerics":if model.quantization().is_some() { "full-strength quantized numerical emulator" } else { "continuous" }});
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
    if args.first().map(String::as_str) == Some("joint-compare") {
        return crate::joint_comparison::run_cli(args);
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

struct BoundModel {
    campaign: Campaign,
    model: JointModel,
    binding: Value,
    artifact: Value,
}

fn finish_attempt<T>(out: &Path, result: Result<T>) -> Result<T> {
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
    {
        return Err(invalid(
            "evaluation campaign changes bound learning configuration",
        ));
    }
    Ok(())
}

fn load_bound_checkpoint(
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

fn write_hard_export(
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

fn load_hard_export(source: &Path, evaluator: &Evaluator) -> Result<BoundModel> {
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
fn evaluate_loaded(
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
    report["executed_quantization"] = serde_json::to_value(model.quantization())?;
    report["evaluation_quantization_strength"] = json!(if model.quantization().is_some() {
        1.0
    } else {
        0.0
    });
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
    save_json(&out.join("evaluation-report.json"), &report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
