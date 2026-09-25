//! One persistent offline recurrent-memory learning campaign.
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::baseline_protocol::{device, load_evaluator, read_tokens, save_json, verify_identity};
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
        Ok(cfg)
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
        "scope":"Offline continuous recurrent-memory learner; no transformer backbone, hard integer export, geometry promotion or energy claim"}),
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

fn save_checkpoint(
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    directory: &Path,
    step: usize,
    status: &str,
    evaluator_sha: &str,
) -> Result<()> {
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
            "evaluator_sha256":evaluator_sha,
            "model_sha256":sha256_file(&directory.join("model.safetensors"))?,
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
    if cfg.training_window_transition.is_some() && resume.is_none() {
        return Err(invalid(
            "a training-window transition requires its sealed parent",
        ));
    }
    report["training_window_transition"] = json!(cfg.training_window_transition);
    report["training_window_transition_applied"] = json!(false);
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
    let (model, mut optimizer, begin) = if let Some(path) = resume {
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
        let model = JointModel::load(path, &selected)?;
        if model.config != cfg.model {
            return Err(invalid("loaded resume model differs from campaign"));
        }
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
        let elapsed = step_started.elapsed().as_secs_f64();
        step_times.push(elapsed);
        complete = step + 1;
        let mut row = json!({"step":complete,"sampled_target_visits":complete*cfg.batch*cfg.context,
            "batch_mean_nll":value,"complete_step_seconds":elapsed,"optimizer":update,
            "cpu_gradient_shards":cfg.cpu_gradient_shards});
        drop(gradients);
        if cfg.development_every_steps > 0 && complete % cfg.development_every_steps == 0 {
            row["development_nll"] =
                json!(quick_loss(&model, &dev, cfg.development_blocks, cfg.batch)?);
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
    if !reloaded_fit.is_finite() || !reload_delta.is_finite() || reload_delta > 1e-6 {
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
    report["reloaded_retained_batch_nll"] = json!(reloaded_fit);
    report["reload_absolute_delta"] = json!(reload_delta);
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
    if args.first().map(String::as_str) == Some("joint-evaluate") {
        return evaluate_cli(args);
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
        return Err(invalid("usage: joint-evaluate CAMPAIGN_JSON SEALED_CHECKPOINT NEW_REPORT_ROOT {cpu|metal} {read|no-read} BATCH"));
    }
    let cfg = Campaign::load(Path::new(&args[1]))?;
    let mode = match args[5].as_str() {
        "read" => ReadMode::Enabled,
        "no-read" => ReadMode::NoRead,
        _ => return Err(invalid("evaluation read mode read|no-read")),
    };
    let batch: usize = args[6].parse().map_err(|_| invalid("evaluation batch"))?;
    if !(1..=joint_evaluation::MAX_EVALUATION_BATCH).contains(&batch) {
        return Err(invalid("evaluation batch1..32"));
    }
    let checkpoint = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = evaluate_checkpoint(&cfg, checkpoint, out, &args[4], mode, batch);
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

fn evaluate_checkpoint(
    cfg: &Campaign,
    checkpoint: &Path,
    out: &Path,
    device_name: &str,
    mode: ReadMode,
    batch: usize,
) -> Result<()> {
    let mut report = metadata(cfg, "joint-evaluate", device_name)?;
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    report_output::verify(checkpoint)?;
    let binding: Value = serde_json::from_slice(&fs::read(checkpoint.join("checkpoint.json"))?)?;
    if binding["evaluator_sha256"] != evaluator.sha256
        || binding["model_sha256"] != sha256_file(&checkpoint.join("model.safetensors"))?
        || binding
            .get("cpu_gradient_shards")
            .map_or(Some(1), Value::as_u64)
            != Some(cfg.cpu_gradient_shards as u64)
    {
        return Err(invalid("checkpoint/evaluator binding differs"));
    }
    let selected = device(device_name)?;
    let model = JointModel::load(checkpoint, &selected)?;
    if model.config != cfg.model {
        return Err(invalid("evaluation campaign/model differs"));
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
            &model,
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
        &joint_evaluation::run_story_probes(&model, &tokenizer, mode)?,
    )?;
    let dev = read_tokens(&evaluator.document["dev_source"])?;
    let mut blocks = BufWriter::new(File::create_new(out.join("blocks.jsonl"))?);
    let mut rows = BufWriter::new(File::create_new(out.join("targets.bin"))?);
    let evaluation =
        joint_evaluation::evaluate(&model, &dev, mode, batch, |block, predictions| {
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
    report["checkpoint_binding"] = binding;
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
