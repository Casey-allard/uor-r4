//! Training-only adaptive choices between fixed neighboring integer codes.
//! Parent weights/scales and recurrent architecture stay fixed. This is an
//! offline F32 learning graph followed by the existing packed emulator codec.
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;

use crate::baseline_protocol::{load_evaluator, read_tokens, save_json};
use crate::joint_campaign::{self, Campaign};
use crate::joint_model::{JointModel, ReadMode};
use crate::joint_optimizer::{AdamConfig, NamedAdamW};
use crate::joint_rounding::{LearnedRounding, RoundingConfig};
use crate::{invalid, sha256_file, Result};

const SCHEMA: &str = "uor-r4.joint-learned-rounding-campaign/1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundingCampaign {
    pub schema: String,
    pub parent_checkpoint: PathBuf,
    pub parent_checkpoint_sha256: String,
    pub parent_model_sha256: String,
    pub quantization_spec_sha256: String,
    pub evaluator_path: PathBuf,
    pub evaluator_sha256: String,
    pub rounding: RoundingConfig,
    pub optimizer: AdamConfig,
    pub data_seed: u64,
    pub data_start_step: usize,
    pub batch: usize,
    pub context: usize,
    pub cpu_gradient_shards: usize,
    pub checkpoint_steps: Vec<usize>,
    pub max_process_seconds: u64,
    pub stop_file: Option<PathBuf>,
    pub scope: String,
}

impl RoundingCampaign {
    fn validate(&self) -> Result<()> {
        self.rounding.validate()?;
        self.optimizer.validate()?;
        if self.schema != SCHEMA
            || self.batch == 0
            || self.batch > 64
            || self.context < 8
            || self.context > 256
            || !matches!(self.cpu_gradient_shards, 1 | 2 | 4)
            || self.batch % self.cpu_gradient_shards != 0
            || self.rounding.steps > 100_000
            || self
                .data_start_step
                .checked_add(self.rounding.steps)
                .is_none()
            || self.checkpoint_steps.len() > 2
            || self
                .checkpoint_steps
                .iter()
                .any(|&s| s == 0 || s >= self.rounding.steps)
            || self.checkpoint_steps.windows(2).any(|p| p[0] >= p[1])
            || self.max_process_seconds == 0
            || self.max_process_seconds > 86400
            || self.optimizer.weight_decay != 0.0
            || !self.optimizer.allowed_missing_gradients.is_empty()
            || self.scope.is_empty()
        {
            return Err(invalid("unsupported learned-rounding campaign"));
        }
        for digest in [
            &self.parent_checkpoint_sha256,
            &self.parent_model_sha256,
            &self.quantization_spec_sha256,
            &self.evaluator_sha256,
        ] {
            if digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(invalid(
                    "rounding campaign requires complete SHA256 bindings",
                ));
            }
        }
        Ok(())
    }
}

fn checkpoint(
    path: &Path,
    cfg: &RoundingCampaign,
    learner: &LearnedRounding,
    optimizer: &NamedAdamW,
    complete: usize,
    sampled_sha: &str,
) -> Result<Value> {
    if optimizer.step_count() != complete as u64 {
        return Err(invalid("rounding optimizer and data clocks differ"));
    }
    let file = path.join("rounding-variables.safetensors");
    if file.try_exists()? {
        return Err(invalid("rounding checkpoint already exists"));
    }
    let variables = learner
        .variables()
        .iter()
        .map(|(name, var)| (name.clone(), var.detach()))
        .collect::<HashMap<_, _>>();
    candle_core::safetensors::save(&variables, &file)?;
    let moments = optimizer.save(path)?;
    save_json(&path.join("rounding-campaign.json"), cfg)?;
    let receipt = json!({"schema":"uor-r4.joint-rounding-checkpoint/1",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "completed_updates":complete,"next_data_step":cfg.data_start_step+complete,
        "sampled_target_visits":complete*cfg.batch*cfg.context,
        "sampled_input_target_sha256_le_u32":sampled_sha,
        "sample_hash_scope":"Ordered inputs then targets for each update in this attempt; resumed segments bind their predecessor separately.",
        "rounding_variables_sha256":sha256_file(&file)?,"optimizer":moments,
        "rounding_campaign_sha256":sha256_file(&path.join("rounding-campaign.json"))?,
        "statistics":learner.statistics()?});
    save_json(&path.join("rounding-checkpoint.json"), &receipt)?;
    report_output::seal(path)?;
    report_output::verify(path)?;
    Ok(receipt)
}

fn resume(
    path: &Path,
    cfg: &RoundingCampaign,
    learner: &LearnedRounding,
) -> Result<(NamedAdamW, usize, Value)> {
    report_output::verify(path)?;
    let stored: RoundingCampaign =
        serde_json::from_slice(&fs::read(path.join("rounding-campaign.json"))?)?;
    let binding: Value = serde_json::from_slice(&fs::read(path.join("rounding-checkpoint.json"))?)?;
    let complete = binding["completed_updates"]
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| invalid("rounding resume clock"))?;
    if stored != *cfg
        || complete > cfg.rounding.steps
        || binding["schema"] != "uor-r4.joint-rounding-checkpoint/1"
        || binding["next_data_step"] != json!(cfg.data_start_step + complete)
        || binding["sampled_target_visits"] != json!(complete * cfg.batch * cfg.context)
        || binding["rounding_campaign_sha256"] != sha256_file(&path.join("rounding-campaign.json"))?
        || binding["rounding_variables_sha256"]
            != sha256_file(&path.join("rounding-variables.safetensors"))?
    {
        return Err(invalid("rounding resume identity or configuration differs"));
    }
    let values =
        candle_core::safetensors::load(path.join("rounding-variables.safetensors"), &Device::Cpu)?;
    if values.len() != learner.variables().len() {
        return Err(invalid("rounding resume inventory"));
    }
    // Validate the complete set before any write. A copy failure discards this
    // newly constructed learner; it never modifies the original parent.
    for (name, var) in learner.variables() {
        let value = values
            .get(name)
            .ok_or_else(|| invalid("missing rounding variable"))?;
        let maximum = value.abs()?.max_all()?.to_scalar::<f32>()?;
        if value.dims() != var.dims()
            || !maximum.is_finite()
            || f64::from(maximum) > cfg.optimizer.parameter_abs_limit
        {
            return Err(invalid("rounding resume variable shape or numeric bound"));
        }
    }
    for (name, var) in learner.variables() {
        var.set(&values[name])?;
    }
    let optimizer = NamedAdamW::load(path, learner.variables(), &cfg.optimizer)?;
    if optimizer.step_count() != complete as u64 {
        return Err(invalid("rounding resume optimizer clock"));
    }
    Ok((
        optimizer,
        complete,
        json!({"path":path,
        "checkpoint_sha256":sha256_file(&path.join("rounding-checkpoint.json"))?,"binding":binding}),
    ))
}

fn fit(cfg: &RoundingCampaign, out: &Path, resume_path: Option<&Path>) -> Result<()> {
    let started = Instant::now();
    let final_path = out.join("checkpoint-final");
    report_output::claim(&final_path)?;
    for step in &cfg.checkpoint_steps {
        report_output::claim(&out.join(format!("checkpoint-{step:08}")))?;
    }
    let packed_path = out.join("packed-model");
    report_output::claim(&packed_path)?;
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    if evaluator.sha256 != cfg.evaluator_sha256
        || sha256_file(&cfg.parent_checkpoint.join("checkpoint.json"))?
            != cfg.parent_checkpoint_sha256
        || sha256_file(&cfg.parent_checkpoint.join("model.safetensors"))? != cfg.parent_model_sha256
    {
        return Err(invalid("rounding parent/evaluator identity mismatch"));
    }
    let mut parent = joint_campaign::load_bound_checkpoint(
        &cfg.parent_checkpoint,
        &Device::Cpu,
        &evaluator.sha256,
    )?;
    let state = parent
        .model
        .quantization()
        .ok_or_else(|| invalid("rounding parent is not quantized"))?;
    let spec_sha = format!("{:x}", Sha256::digest(serde_json::to_vec(&state.spec)?));
    if spec_sha != cfg.quantization_spec_sha256
        || cfg.context != parent.model.config.context
        || cfg.context != parent.campaign.context
        || cfg.batch != parent.campaign.batch
        || cfg.data_start_step != state.completed_step
        || cfg.data_seed != parent.campaign.data_seed
    {
        return Err(invalid(
            "rounding changes fixed scale/context/batch or training continuation identity",
        ));
    }
    let learner =
        LearnedRounding::new(&state.spec, parent.model.variables(), cfg.rounding.clone())?;
    let initial_stats = learner.statistics()?;
    let (mut optimizer, begin, resume_binding) = match resume_path {
        Some(path) => resume(path, cfg, &learner)?,
        None => (
            NamedAdamW::new(learner.variables(), cfg.optimizer.clone())?,
            0,
            Value::Null,
        ),
    };
    let mut sample_cfg: Campaign = parent.campaign.clone();
    sample_cfg.cpu_gradient_shards = cfg.cpu_gradient_shards;
    // The shared sampler uses the retained data seed, B/T and an explicit
    // continuation counter. No development/source-answer record enters the fit.
    let sources = evaluator.document["train_sources"]
        .as_array()
        .ok_or_else(|| invalid("training sources"))?;
    let stores = sources
        .iter()
        .map(read_tokens)
        .collect::<Result<Vec<_>>>()?;
    save_json(&out.join("rounding-campaign.json"), cfg)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    let mut report = joint_campaign::metadata(&parent.campaign, "joint-round-fit", "cpu")?;
    report["rounding_campaign"] = json!(cfg);
    report["rounding_campaign_sha256"] = json!(sha256_file(&out.join("rounding-campaign.json"))?);
    report["parent_artifact"] = parent.artifact.clone();
    report["parent_checkpoint_binding"] = parent.binding.clone();
    report["resume"] = resume_binding;
    report["initial_statistics"] = initial_stats;
    report["parameter_optimizer_scope"] = json!("Only new alpha code-choice variables are optimized. Original weights, scales, parent Adam moments/clocks and architecture are unchanged.");
    let (probe_inputs, probe_targets) =
        joint_campaign::training_batch(&stores, &sample_cfg, cfg.data_start_step)?;
    let initial_hard = parent
        .model
        .materialize_rounding_codes(learner.parameters(true)?)?;
    if begin == 0 {
        let expected = parent
            .model
            .forward(
                &probe_inputs,
                cfg.batch,
                cfg.context,
                ReadMode::Enabled,
                false,
            )?
            .probabilities;
        let actual = initial_hard
            .forward(
                &probe_inputs,
                cfg.batch,
                cfg.context,
                ReadMode::Enabled,
                false,
            )?
            .probabilities;
        let delta = expected
            .sub(&actual)?
            .abs()?
            .max_all()?
            .to_scalar::<f32>()?;
        if delta != 0.0 {
            return Err(invalid(
                "initial rounding hard path differs from retained parent",
            ));
        }
        report["initial_hard_parent_probability_max_delta"] = json!(delta);
    }
    report["initial_hard_retained_training_batch_nll"] = json!(initial_hard
        .forward(
            &probe_inputs,
            cfg.batch,
            cfg.context,
            ReadMode::Enabled,
            false
        )?
        .loss(&probe_targets)?
        .to_scalar::<f32>()?);
    drop(initial_hard);
    let mut curve = BufWriter::new(File::create_new(out.join("learning-curve.jsonl"))?);
    let mut complete = begin;
    let mut samples = Sha256::new();
    for step in begin..cfg.rounding.steps {
        if started.elapsed().as_secs() >= cfg.max_process_seconds
            || cfg.stop_file.as_ref().is_some_and(|p| p.exists())
        {
            break;
        }
        let update_started = Instant::now();
        let (inputs, targets) =
            joint_campaign::training_batch(&stores, &sample_cfg, cfg.data_start_step + step)?;
        for token in inputs.iter().chain(targets.iter()) {
            samples.update(token.to_le_bytes());
        }
        let model = parent
            .model
            .rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
        let mut gradients = crate::joint_parallel::batch_gradients(
            &model,
            &inputs,
            &targets,
            cfg.batch,
            cfg.context,
            cfg.cpu_gradient_shards,
        )?;
        let penalty = learner.penalty(step)?;
        let penalty_value = penalty.to_scalar::<f32>()?;
        let penalty_gradients = penalty.backward()?;
        for (name, variable) in learner.variables() {
            if let Some(extra) = penalty_gradients.get(variable.as_tensor()) {
                let nll_gradient =
                    gradients
                        .gradients
                        .get(variable.as_tensor())
                        .ok_or_else(|| {
                            invalid(format!("missing code-choice language gradient {name}"))
                        })?;
                gradients
                    .gradients
                    .insert(variable.as_tensor(), nll_gradient.add(extra)?.detach());
            }
        }
        if !penalty_value.is_finite() {
            return Err(invalid("nonfinite rounding penalty"));
        }
        let update = optimizer.step(learner.variables(), &gradients.gradients)?;
        complete = step + 1;
        let row = json!({"completed_updates":complete,"data_step":cfg.data_start_step+step,
            "sampled_target_visits":complete*cfg.batch*cfg.context,"batch_mean_nll":gradients.mean_nll,
            "weighted_rounding_penalty":penalty_value,"objective":gradients.mean_nll+penalty_value,
            "beta":cfg.rounding.beta(step),"optimizer":update,"complete_step_seconds":update_started.elapsed().as_secs_f64()});
        serde_json::to_writer(&mut curve, &row)?;
        writeln!(curve)?;
        curve.flush()?;
        if step == begin || complete % 16 == 0 {
            eprintln!(
                "rounding {complete}/{}: NLL {:.5}, penalty {penalty_value:.6}, {:.3}s",
                cfg.rounding.steps,
                gradients.mean_nll,
                update_started.elapsed().as_secs_f64()
            );
        }
        drop(gradients);
        drop(model);
        if cfg.checkpoint_steps.contains(&complete) {
            checkpoint(
                &out.join(format!("checkpoint-{complete:08}")),
                cfg,
                &learner,
                &optimizer,
                complete,
                &format!("{:x}", samples.clone().finalize()),
            )?;
        }
    }
    let final_binding = checkpoint(
        &final_path,
        cfg,
        &learner,
        &optimizer,
        complete,
        &format!("{:x}", samples.finalize()),
    )?;
    report["final_checkpoint"] = final_binding.clone();
    report["completed_updates"] = json!(complete);
    report["new_updates_this_attempt"] = json!(complete - begin);
    report["new_target_visits_this_attempt"] = json!((complete - begin) * cfg.batch * cfg.context);
    report["status"] = json!(if complete == cfg.rounding.steps {
        "COMPLETE_FIXED_RECIPE"
    } else {
        "STOPPED_CHECKPOINTED"
    });
    if complete == cfg.rounding.steps {
        let calibration = json!({"schema":"uor-r4.joint-rounding-lineage/1",
            "rounding_campaign":cfg,"rounding_campaign_sha256":report["rounding_campaign_sha256"],
            "checkpoint_path":final_path,"checkpoint_sha256":sha256_file(&final_path.join("rounding-checkpoint.json"))?,
            "checkpoint":final_binding,"parent_model_clock_unchanged":true,
            "rounding_optimizer_steps":complete,"source_commit":report["source_commit"],
            "scope":"Full-trajectory training-only code choices. Parent weights/scales/architecture fixed; no held-out selection; packed execution remains F32."});
        let hard_parameters = learner.parameters(true)?;
        let mut changes = serde_json::Map::new();
        let mut changed_total = 0usize;
        let mut committed_total = 0usize;
        let spec = &parent
            .model
            .quantization()
            .ok_or_else(|| invalid("lost frozen grids"))?
            .spec;
        for (name, value) in &hard_parameters {
            let old = spec
                .parameter(name, parent.model.variables()[name].as_tensor(), 1.0, false)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            let new = value.flatten_all()?.to_vec1::<f32>()?;
            let changed = old.iter().zip(&new).filter(|(a, b)| a != b).count();
            let alpha = learner.variables()[name].flatten_all()?.to_vec1::<f32>()?;
            let committed = alpha
                .iter()
                .filter(|a| f64::from(**a).abs() >= 11f64.ln())
                .count();
            changed_total += changed;
            committed_total += committed;
            changes.insert(
                name.clone(),
                json!({"elements":old.len(),"changed_hard_codes_from_nearest_parent":changed,
                "alpha_at_or_beyond_relaxation_clamp":committed}),
            );
        }
        report["hard_code_changes"] = json!({"total_changed":changed_total,
            "alpha_at_or_beyond_relaxation_clamp":committed_total,"by_parameter":changes});
        parent.model = parent.model.materialize_rounding_codes(hard_parameters)?;
        // Persist calibration in both binding and artifact, so the old parent
        // clock cannot be mistaken for zero learning of the new hard codes.
        parent.binding["rounding_calibration"] = calibration.clone();
        parent.artifact["rounding_calibration"] = calibration;
        let restored = joint_campaign::write_hard_export(
            &parent,
            &cfg.parent_checkpoint,
            &evaluator,
            &packed_path,
        );
        joint_campaign::finish_attempt(&packed_path, restored)?;
        report["packed_model"] = json!(packed_path);
    } else {
        save_json(
            &packed_path.join("not-run.json"),
            &json!({"status":"NOT_RUN","reason":"Fixed recipe incomplete; alpha/optimizer checkpoint retained."}),
        )?;
        report_output::seal(&packed_path)?;
    }
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["full_evaluation_during_fit"] = json!(false);
    report["training_samples_only"] = json!(true);
    save_json(&out.join("fit-report.json"), &report)?;
    eprintln!(
        "rounding complete status {} updates {complete}",
        report["status"]
    );
    Ok(())
}

/// A fixed training-only normalization rule, not a hyperparameter search.
/// At the initial soft parameters, balance the L2 norm of the eight-batch mean
/// language gradient with the global-mean binary penalty gradient at beta=2.
/// The chosen scalar and every sampled input/target are bound before any update.
fn calibrate(cfg: &RoundingCampaign, out: &Path) -> Result<()> {
    const BATCHES: usize = 8;
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    if evaluator.sha256 != cfg.evaluator_sha256
        || sha256_file(&cfg.parent_checkpoint.join("checkpoint.json"))?
            != cfg.parent_checkpoint_sha256
        || sha256_file(&cfg.parent_checkpoint.join("model.safetensors"))? != cfg.parent_model_sha256
    {
        return Err(invalid("rounding calibration parent/evaluator mismatch"));
    }
    let parent = joint_campaign::load_bound_checkpoint(
        &cfg.parent_checkpoint,
        &Device::Cpu,
        &evaluator.sha256,
    )?;
    let state = parent
        .model
        .quantization()
        .ok_or_else(|| invalid("rounding calibration quantization"))?;
    if format!("{:x}", Sha256::digest(serde_json::to_vec(&state.spec)?))
        != cfg.quantization_spec_sha256
        || cfg.batch != parent.campaign.batch
        || cfg.context != parent.campaign.context
        || cfg.context != parent.model.config.context
        || cfg.data_start_step != state.completed_step
        || cfg.data_seed != parent.campaign.data_seed
    {
        return Err(invalid(
            "rounding calibration fixed representation/data mismatch",
        ));
    }
    let mut probe_config = cfg.rounding.clone();
    probe_config.warmup_steps = 0;
    probe_config.beta_start = 2.0;
    probe_config.beta_end = 2.0;
    probe_config.regularization = 1.0;
    let learner = LearnedRounding::new(&state.spec, parent.model.variables(), probe_config)?;
    let stores = evaluator.document["train_sources"]
        .as_array()
        .ok_or_else(|| invalid("calibration train sources"))?
        .iter()
        .map(read_tokens)
        .collect::<Result<Vec<_>>>()?;
    let mut totals = std::collections::BTreeMap::<String, Tensor>::new();
    let mut losses = Vec::new();
    let mut sample_hash = Sha256::new();
    for batch in 0..BATCHES {
        let (inputs, targets) =
            joint_campaign::training_batch(&stores, &parent.campaign, cfg.data_start_step + batch)?;
        for token in inputs.iter().chain(targets.iter()) {
            sample_hash.update(token.to_le_bytes());
        }
        let view = parent
            .model
            .rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
        let gradients = crate::joint_parallel::batch_gradients(
            &view,
            &inputs,
            &targets,
            cfg.batch,
            cfg.context,
            cfg.cpu_gradient_shards,
        )?;
        losses.push(gradients.mean_nll);
        for (name, var) in learner.variables() {
            let value = gradients
                .gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("missing calibration gradient"))?
                .detach()
                .affine(1.0 / BATCHES as f64, 0.0)?;
            let total = match totals.remove(name) {
                Some(old) => old.add(&value)?,
                None => value,
            };
            totals.insert(name.clone(), total);
        }
        eprintln!(
            "rounding training-only normalization {}/{BATCHES}",
            batch + 1
        );
    }
    let penalty = learner.penalty(0)?;
    let penalty_gradients = penalty.backward()?;
    let mut task_square = 0.0_f64;
    let mut penalty_square = 0.0_f64;
    let mut by_name = serde_json::Map::new();
    for (name, var) in learner.variables() {
        let task = f64::from(totals[name].sqr()?.sum_all()?.to_scalar::<f32>()?);
        let reg = f64::from(
            penalty_gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("missing normalization penalty gradient"))?
                .sqr()?
                .sum_all()?
                .to_scalar::<f32>()?,
        );
        if !task.is_finite() || !reg.is_finite() {
            return Err(invalid("nonfinite normalization gradient"));
        }
        task_square += task;
        penalty_square += reg;
        by_name.insert(name.clone(),json!({"mean_task_gradient_l2":task.sqrt(),"unit_beta2_penalty_gradient_l2":reg.sqrt()}));
    }
    if task_square <= 0.0 || penalty_square <= 0.0 {
        return Err(invalid(
            "zero-gradient normalization cannot choose a coefficient",
        ));
    }
    let coefficient = (task_square / penalty_square).sqrt();
    let mut resolved = cfg.clone();
    resolved.rounding.regularization = coefficient;
    resolved.validate()?;
    save_json(&out.join("rounding-campaign.json"), &resolved)?;
    save_json(
        &out.join("calibration-report.json"),
        &json!({
        "schema":"uor-r4.rounding-gradient-normalization/1","status":"CALIBRATED_NO_UPDATES",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,"parent_artifact":parent.artifact,
        "input_recipe":cfg,"resolved_recipe_sha256":sha256_file(&out.join("rounding-campaign.json"))?,
        "formula":"lambda = L2(mean of 8 full-batch language alpha gradients) / L2(global-mean unit-coefficient rounding penalty alpha gradient at beta=2); ratio=1; no sweep or validation selection",
        "language_loss_scope":"Initial soft neighboring parameters with all fixed interface quantizers and their retained STE",
        "training_batches":BATCHES,"target_visits":BATCHES*cfg.batch*cfg.context,
        "data_steps":[cfg.data_start_step,cfg.data_start_step+BATCHES],"sample_hash_order":"inputs then targets each batch, little-endian u32",
        "sampled_input_target_sha256_le_u32":format!("{:x}",sample_hash.finalize()),"batch_mean_nll":losses,
        "task_gradient_l2":task_square.sqrt(),"unit_beta2_penalty_gradient_l2":penalty_square.sqrt(),
        "regularization":coefficient,"by_parameter":by_name,"optimizer_updates":0,"used_evaluation_data":false,
        "scope":"Recipe initialization on training only. Gradient norm balancing is a declared heuristic, not proof of optimal regularization or expected code recovery."}),
    )?;
    Ok(())
}

pub fn run_cli(args: &[String]) -> Result<()> {
    if !(args.len() == 3 || args.len() == 4)
        || !matches!(
            args[0].as_str(),
            "joint-round-fit" | "joint-round-calibrate"
        )
        || (args[0] == "joint-round-calibrate" && args.len() != 3)
    {
        return Err(invalid("usage: joint-round-fit ROUNDING_CAMPAIGN_JSON NEW_REPORT_ROOT [SEALED_ROUNDING_RESUME_CHECKPOINT]"));
    }
    let cfg: RoundingCampaign = serde_json::from_slice(&fs::read(&args[1])?)?;
    cfg.validate()?;
    let out = Path::new(&args[2]);
    report_output::claim(out)?;
    let result = if args[0] == "joint-round-calibrate" {
        calibrate(&cfg, out)
    } else {
        fit(&cfg, out, args.get(3).map(Path::new))
    };
    joint_campaign::finish_attempt(out, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_model::{JointConfig, Transport};

    fn config() -> RoundingConfig {
        RoundingConfig {
            steps: 8,
            warmup_steps: 2,
            beta_start: 20.0,
            beta_end: 2.0,
            regularization: 0.1,
        }
    }

    #[test]
    fn learned_rounding_language_gradients_shards_and_hard_reload() -> Result<()> {
        for transport in [Transport::Quaternion, Transport::HouseholderPair] {
            let mut parent = JointModel::new(
                JointConfig {
                    width: 128,
                    context: 8,
                    transport,
                    seed: 77,
                    ..JointConfig::default()
                },
                &Device::Cpu,
            )?;
            parent.configure_quantization(0, 1)?;
            parent.set_completed_step(1)?;
            let spec = &parent
                .quantization()
                .ok_or_else(|| invalid("test quantization"))?
                .spec;
            spec.project_parameters(parent.variables())?;
            let original = parent
                .variables()
                .iter()
                .map(|(n, v)| Ok((n.clone(), v.flatten_all()?.to_vec1::<f32>()?)))
                .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
            let learner = LearnedRounding::new(spec, parent.variables(), config())?;
            let ids = [4, 8, 4, 9, 8, 7, 9, 4, 7, 4, 8, 4, 7, 9, 8, 4];
            let targets = [8, 4, 9, 8, 7, 9, 4, 7, 4, 8, 4, 7, 9, 8, 4, 9];
            let view = parent
                .rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
            assert!(view.forward(&ids, 2, 8, ReadMode::Enabled, false).is_err());
            assert!(view.without_quantization().is_err());
            assert!(view.new_session(1).is_err());
            let one = crate::joint_parallel::batch_gradients(&view, &ids, &targets, 2, 8, 1)?;
            let two = crate::joint_parallel::batch_gradients(&view, &ids, &targets, 2, 8, 2)?;
            assert!((one.mean_nll - two.mean_nll).abs() < 2e-5);
            for (name, var) in learner.variables() {
                let a = one
                    .gradients
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("missing alpha gradient {name}")))?;
                let b = two
                    .gradients
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("missing sharded alpha gradient {name}")))?;
                assert!(
                    a.sub(b)?.abs()?.max_all()?.to_scalar::<f32>()? < 2e-5,
                    "{name}"
                );
            }
            for name in [
                "embedding.weight",
                "recurrent.state.weight",
                "read.query.weight",
                "read.key.weight",
            ] {
                let var = &learner.variables()[name];
                assert!(
                    one.gradients
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("language gradient"))?
                        .sqr()?
                        .sum_all()?
                        .to_scalar::<f32>()?
                        > 0.0,
                    "{name}"
                );
            }
            let adam_cfg = AdamConfig {
                learning_rate: 0.01,
                weight_decay: 0.0,
                ..AdamConfig::default()
            };
            let mut optimizer = NamedAdamW::new(learner.variables(), adam_cfg)?;
            optimizer.step(learner.variables(), &one.gradients)?;
            assert_eq!(optimizer.step_count(), 1);
            for (name, var) in parent.variables() {
                assert_eq!(
                    var.flatten_all()?.to_vec1::<f32>()?,
                    original[name],
                    "parent changed {name}"
                );
            }
            let hard = parent.materialize_rounding_codes(learner.parameters(true)?)?;
            assert_eq!(hard.quantization(), parent.quantization());
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "uor-rounding-reload-{}-{stamp}",
                std::process::id()
            ));
            fs::create_dir(&root)?;
            assert!(view.save_hard(&root).is_err());
            hard.save_hard(&root)?;
            let loaded = JointModel::load_hard(&root, &Device::Cpu)?;
            for mode in [ReadMode::Enabled, ReadMode::NoRead] {
                let a = hard.forward(&ids, 2, 8, mode, false)?.probabilities;
                let b = loaded.forward(&ids, 2, 8, mode, false)?.probabilities;
                assert_eq!(a.sub(&b)?.abs()?.max_all()?.to_scalar::<f32>()?, 0.0);
            }
            fs::remove_dir_all(root)?;
        }
        Ok(())
    }

    #[test]
    fn learned_rounding_checkpoint_binds_alpha_optimizer_and_recipe() -> Result<()> {
        let mut parent = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                seed: 91,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        parent.configure_quantization(0, 1)?;
        parent.set_completed_step(1)?;
        let spec = &parent
            .quantization()
            .ok_or_else(|| invalid("quantization"))?
            .spec;
        spec.project_parameters(parent.variables())?;
        let learner = LearnedRounding::new(spec, parent.variables(), config())?;
        let mut optimizer = NamedAdamW::new(
            learner.variables(),
            AdamConfig {
                weight_decay: 0.0,
                ..AdamConfig::default()
            },
        )?;
        let gradients = learner.penalty(3)?.backward()?;
        optimizer.step(learner.variables(), &gradients)?;
        let cfg = RoundingCampaign {
            schema: SCHEMA.into(),
            parent_checkpoint: "unused-test-parent".into(),
            parent_checkpoint_sha256: "0".repeat(64),
            parent_model_sha256: "0".repeat(64),
            quantization_spec_sha256: "0".repeat(64),
            evaluator_path: "unused-test-evaluator".into(),
            evaluator_sha256: "0".repeat(64),
            rounding: config(),
            optimizer: AdamConfig {
                weight_decay: 0.0,
                ..AdamConfig::default()
            },
            data_seed: 91,
            data_start_step: 1,
            batch: 2,
            context: 8,
            cpu_gradient_shards: 2,
            checkpoint_steps: vec![],
            max_process_seconds: 60,
            stop_file: None,
            scope: "Serialization fixture, not model capability".into(),
        };
        cfg.validate()?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-rounding-checkpoint-{}-{stamp}",
            std::process::id()
        ));
        report_output::claim(&root)?;
        checkpoint(&root, &cfg, &learner, &optimizer, 1, &"0".repeat(64))?;
        let restored = LearnedRounding::new(spec, parent.variables(), config())?;
        let (restored_optimizer, step, _) = resume(&root, &cfg, &restored)?;
        assert_eq!(step, 1);
        assert_eq!(restored_optimizer.step_count(), 1);
        for (name, var) in learner.variables() {
            assert_eq!(
                var.flatten_all()?.to_vec1::<f32>()?,
                restored.variables()[name].flatten_all()?.to_vec1::<f32>()?
            );
        }
        let mut changed = cfg.clone();
        changed.data_seed += 1;
        assert!(resume(&root, &changed, &restored).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
