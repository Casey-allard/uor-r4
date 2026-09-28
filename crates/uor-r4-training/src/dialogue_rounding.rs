//! Response-aware choices on the selected dialogue child's immutable integer grid.
//! This is offline alpha learning, not another weight fit or a serving decoder.
//! The complete response/EOS objective is reduced before one global penalty and
//! one clipped Adam update. Model and alpha clocks are deliberately independent.
//! Alpha response shards execute sequentially with immediate detached named
//! reduction; other trainers retain their existing parallel execution.
use crate::{
    dialogue_artifact::DatasetBinding,
    dialogue_episodes::{EpisodeBatch, EpisodeContract, EpisodeIndex, PrefixPolicy, SAMPLER_ID},
    dialogue_learning::{advance_schedule, Corpus},
    dialogue_rounding_artifact::DialogueRoundingParent,
    invalid,
    joint_model::JointModel,
    joint_optimizer::{AdamConfig, NamedAdamW},
    joint_parallel::{sequential_response_batch_gradients, ResponseGradients},
    joint_rounding::{LearnedRounding, RoundingConfig},
    sha256_file, Result,
};
use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;

const SCHEMA: &str = "uor-r4.dialogue-rounding-campaign/1";
const CHECKPOINT: &str = "uor-r4.dialogue-rounding-checkpoint/1";
const CONTEXT: usize = 256;
const NORMALIZATION_BATCHES: usize = 8;

/// Every optimization/dose choice is supplied by the caller. No selected recipe
/// is hidden in the executable. Resource-only resume changes do not alter it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub schema: String,
    pub child_report_root: PathBuf,
    pub prepared_manifest: PathBuf,
    pub tokenizer_path: PathBuf,
    pub nearest_packed_root: PathBuf,
    pub child_parameter_sha256: String,
    pub prepared_manifest_sha256: String,
    pub tokenizer_sha256: String,
    pub nearest_hard_manifest_sha256: String,
    pub train_index_sha256: String,
    pub rounding: RoundingConfig,
    pub optimizer: AdamConfig,
    pub data_seed: u64,
    pub data_start_step: usize,
    pub batch: usize,
    pub cpu_gradient_shards: usize,
    pub checkpoint_steps: Vec<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalization_report: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalization_report_sha256: Option<String>,
    pub max_process_seconds: f64,
    pub stop_file: Option<PathBuf>,
    pub resume_from: Option<PathBuf>,
}
impl Campaign {
    pub(crate) fn validate(&self) -> Result<()> {
        self.rounding.validate()?;
        self.optimizer.validate()?;
        if self.schema != SCHEMA
            || self.rounding.steps > 100_000
            || self.batch == 0
            || self.batch > 64
            || !matches!(self.cpu_gradient_shards, 1 | 2 | 4)
            || self.batch % self.cpu_gradient_shards != 0
            || self
                .data_start_step
                .checked_add(self.rounding.steps.max(NORMALIZATION_BATCHES))
                .is_none()
            || self.checkpoint_steps.len() > 16
            || self
                .checkpoint_steps
                .iter()
                .any(|&n| n == 0 || n >= self.rounding.steps)
            || self.checkpoint_steps.windows(2).any(|a| a[0] >= a[1])
            || !self.max_process_seconds.is_finite()
            || self.max_process_seconds <= 0.0
            || self.max_process_seconds > 86_400.0
            || self.optimizer.weight_decay != 0.0
            || self.optimizer.parameter_abs_limit != 30.0
            || !self.optimizer.allowed_missing_gradients.is_empty()
            || self.normalization_report.is_some() != self.normalization_report_sha256.is_some()
        {
            return Err(invalid("unsupported bounded dialogue rounding campaign"));
        }
        for h in [
            &self.child_parameter_sha256,
            &self.prepared_manifest_sha256,
            &self.tokenizer_sha256,
            &self.nearest_hard_manifest_sha256,
            &self.train_index_sha256,
        ]
        .into_iter()
        .chain(self.normalization_report_sha256.as_ref())
        {
            if !is_sha(h) {
                return Err(invalid(
                    "dialogue rounding requires complete SHA256 bindings",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn learning_contract(&self) -> Result<Value> {
        let mut value = serde_json::to_value(self)?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| invalid("campaign object"))?;
        for field in ["resume_from", "max_process_seconds", "stop_file"] {
            fields.remove(field);
        }
        Ok(value)
    }
    fn normalization_contract(&self) -> Result<Value> {
        let mut value = self.learning_contract()?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| invalid("campaign object"))?;
        fields.remove("normalization_report");
        fields.remove("normalization_report_sha256");
        // These fields cannot affect the initial-soft, unit-beta2 probe.
        // A later fit still binds the complete caller-selected recipe.
        for key in ["rounding", "optimizer", "checkpoint_steps"] {
            fields.remove(key);
        }
        fields.insert(
            "normalization_rule".into(),
            json!("eight-training-batches/response-pooled-gradient-l2/unit-beta2/v1"),
        );
        Ok(value)
    }
}
fn is_sha(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn digest<T: Serialize + ?Sized>(v: &T) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(v)?)))
}
fn save(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
fn number(v: &Value, name: &str) -> Result<usize> {
    v[name]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid(format!("missing bounded count {name}")))
}
fn text<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v[name]
        .as_str()
        .ok_or_else(|| invalid(format!("missing string {name}")))
}
fn dataset(v: &Value) -> Result<DatasetBinding> {
    Ok(DatasetBinding {
        split: text(v, "split")?.into(),
        tokens_path: text(v, "tokens_path")?.into(),
        mask_path: text(v, "mask_path")?.into(),
        tokens_sha256: text(v, "tokens_sha256")?.into(),
        mask_sha256: text(v, "mask_sha256")?.into(),
        tokens: number(v, "tokens")?,
        response_tokens: number(v, "response_tokens")?,
    })
}
fn load_parent(cfg: &Campaign) -> Result<DialogueRoundingParent> {
    for (p, h) in [
        (&cfg.prepared_manifest, &cfg.prepared_manifest_sha256),
        (&cfg.tokenizer_path, &cfg.tokenizer_sha256),
        (
            &cfg.nearest_packed_root.join("hard-model.json"),
            &cfg.nearest_hard_manifest_sha256,
        ),
    ] {
        if sha256_file(p)? != *h {
            return Err(invalid("rounding campaign input hash mismatch"));
        }
    }
    let parent = DialogueRoundingParent::load(
        &cfg.child_report_root,
        &cfg.prepared_manifest,
        &cfg.tokenizer_path,
        &cfg.nearest_packed_root,
        &Device::Cpu,
    )?;
    let child = parent.child();
    if child.parameter_sha256 != cfg.child_parameter_sha256
        || child.train_index_sha256 != cfg.train_index_sha256
        || child.steps_completed != cfg.data_start_step
        || child.learning_contract["data_seed"] != cfg.data_seed
        || child.learning_contract["batch"] != cfg.batch
        || child.learning_contract["cpu_gradient_shards"] != cfg.cpu_gradient_shards
    {
        return Err(invalid("rounding child/data continuation binding mismatch"));
    }
    Ok(parent)
}
fn corpus_split(
    parent: &DialogueRoundingParent,
    cfg: &Campaign,
    split: &str,
) -> Result<(Corpus, EpisodeContract)> {
    let child = parent.child();
    let prepared: Value = serde_json::from_slice(&fs::read(&cfg.prepared_manifest)?)?;
    let binding = child.ancestor["datasets"]
        .as_array()
        .ok_or_else(|| invalid("ancestor datasets"))?
        .iter()
        .find(|v| v["split"] == split)
        .ok_or_else(|| invalid("training dataset"))?;
    let corpus = Corpus::load(&dataset(binding)?, &prepared, &cfg.prepared_manifest)?;
    let protocol = &child.protocol;
    let encoder = protocol
        .bind(parent.tokenizer())
        .map_err(|e| invalid(e.to_string()))?;
    let prefix = encoder.encode_assistant_prefix(&[]);
    if prefix.tokens.first() != Some(&protocol.bos_id) {
        return Err(invalid("assistant marker BOS"));
    }
    let contract = EpisodeContract {
        context: CONTEXT,
        vocab_size: 4096,
        bos_id: protocol.bos_id,
        eos_id: protocol.eos_id,
        unk_id: protocol.unk_id,
        padding_id: protocol.unk_id,
        assistant_marker_ids: prefix.tokens[1..].to_vec(),
    };
    if split == "train" {
        let index = corpus.index(contract.clone())?;
        // Reproduce the child's exact inventory identity, including both split bindings.
        let inventory = json!({"contract":index.contract(),"sampler":SAMPLER_ID,
        "episodes":index.episodes(),"sources":index.sources(),"population":index.population(),
        "parent_token_mask_bindings":child.ancestor["datasets"],
        "prepared_manifest_sha256":cfg.prepared_manifest_sha256,"protocol_identity":child.protocol_identity});
        if digest(&inventory)? != cfg.train_index_sha256 {
            return Err(invalid("rounding training inventory differs from child"));
        }
    }
    Ok((corpus, contract))
}
#[cfg(test)]
fn learner(parent: &JointModel, cfg: RoundingConfig) -> Result<LearnedRounding> {
    let spec = &parent
        .quantization()
        .ok_or_else(|| invalid("missing frozen rounding grid"))?
        .spec;
    LearnedRounding::new(spec, parent.variables(), cfg)
}

/// Penalty is combined after response-mass reduction, once, including connected
/// zero gradients during warm-up and for non-choosable coordinates.
fn add_penalty(
    learner: &LearnedRounding,
    step: usize,
    gradients: &mut ResponseGradients,
) -> Result<f32> {
    let penalty = learner.penalty(step)?;
    let value = penalty.to_scalar::<f32>()?;
    if !value.is_finite() {
        return Err(invalid("nonfinite rounding penalty"));
    }
    let extra = penalty.backward()?;
    for (name, var) in learner.variables() {
        let task = gradients
            .gradients
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("missing response alpha gradient {name}")))?;
        let reg = extra
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("missing penalty alpha gradient {name}")))?;
        gradients
            .gradients
            .insert(var.as_tensor(), task.add(reg)?.detach());
    }
    Ok(value)
}
fn response_gradients(
    parent: &JointModel,
    learner: &LearnedRounding,
    batch: &EpisodeBatch,
    shards: usize,
) -> Result<ResponseGradients> {
    let view =
        parent.rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
    sequential_response_batch_gradients(
        &view,
        &batch.inputs,
        &batch.targets,
        &batch.weights,
        batch.batch,
        batch.time,
        shards,
    )
}
fn target_hash(batch: &EpisodeBatch) -> String {
    let mut h = Sha256::new();
    for id in &batch.selected_target_ids {
        h.update(id.to_le_bytes());
    }
    hex::encode(h.finalize())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Progress {
    completed_updates: usize,
    next_data_step: usize,
    supervised_target_visits: usize,
    padded_position_visits: usize,
    executed_schedule_sha256: String,
}
impl Progress {
    fn initial(cfg: &Campaign) -> Self {
        Self {
            completed_updates: 0,
            next_data_step: cfg.data_start_step,
            supervised_target_visits: 0,
            padded_position_visits: 0,
            // Separate stage domain: the receipt also binds the previous child chain.
            executed_schedule_sha256: hex::encode(Sha256::digest(
                b"uor-r4.dialogue-rounding-response-schedule/1",
            )),
        }
    }
    fn advance(&mut self, cfg: &Campaign, batch: &EpisodeBatch) -> Result<()> {
        self.executed_schedule_sha256 = advance_schedule(
            &self.executed_schedule_sha256,
            self.next_data_step,
            &batch.response_ids,
            &target_hash(batch),
        )?;
        self.completed_updates += 1;
        self.next_data_step = cfg.data_start_step + self.completed_updates;
        self.supervised_target_visits = self
            .supervised_target_visits
            .checked_add(batch.counts.supervised_target_count)
            .ok_or_else(|| invalid("response counter overflow"))?;
        self.padded_position_visits += batch.batch * batch.time;
        Ok(())
    }
    fn validate(&self, cfg: &Campaign) -> Result<()> {
        if self.completed_updates > cfg.rounding.steps
            || self.next_data_step != cfg.data_start_step + self.completed_updates
            || self.padded_position_visits != self.completed_updates * cfg.batch * CONTEXT
            || self.supervised_target_visits > self.padded_position_visits
            || self.supervised_target_visits < self.completed_updates * cfg.batch
            || !is_sha(&self.executed_schedule_sha256)
        {
            return Err(invalid("rounding checkpoint exposure/clock mismatch"));
        }
        Ok(())
    }
}
fn checkpoint(
    path: &Path,
    cfg: &Campaign,
    prep: &Value,
    learner: &LearnedRounding,
    optimizer: &NamedAdamW,
    progress: &Progress,
    source: &str,
    executable: &str,
) -> Result<Value> {
    progress.validate(cfg)?;
    if optimizer.step_count() != progress.completed_updates as u64 {
        return Err(invalid("alpha optimizer clock mismatch"));
    }
    report_output::claim(path)?;
    let vars = learner
        .variables()
        .iter()
        .map(|(n, v)| (n.clone(), v.detach()))
        .collect::<HashMap<_, _>>();
    candle_core::safetensors::save(&vars, path.join("rounding-variables.safetensors"))?;
    let optimizer_receipt = optimizer.save(path)?;
    save(&path.join("campaign.json"), &serde_json::to_value(cfg)?)?;
    let mut binding = serde_json::to_value(progress)?;
    let fields = binding
        .as_object_mut()
        .ok_or_else(|| invalid("progress object"))?;
    fields.extend(serde_json::to_value(json!({"schema":CHECKPOINT,"preparation":prep,
        "campaign_sha256":sha256_file(&path.join("campaign.json"))?,"learning_contract":cfg.learning_contract()?,
        "variables_sha256":sha256_file(&path.join("rounding-variables.safetensors"))?,
        "optimizer":optimizer_receipt,"optimizer_fingerprint":optimizer.continuity_fingerprint()?,
        "source_commit":source,"executable_sha256":executable}))?.as_object()
        .ok_or_else(|| invalid("checkpoint object"))?.clone());
    save(&path.join("checkpoint.json"), &binding)?;
    report_output::seal(path)?;
    report_output::verify(path)?;
    Ok(binding)
}
fn resume(
    path: &Path,
    cfg: &Campaign,
    prep: &Value,
    learner: &LearnedRounding,
    source: &str,
    executable: &str,
) -> Result<(NamedAdamW, Progress)> {
    report_output::verify(path)?;
    if path.join("failed-attempt.json").exists() {
        return Err(invalid("failed rounding checkpoint"));
    }
    let stored: Campaign = serde_json::from_slice(&fs::read(path.join("campaign.json"))?)?;
    stored.validate()?;
    let binding: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
    let progress: Progress = serde_json::from_value(binding.clone())?;
    progress.validate(cfg)?;
    if binding["schema"] != CHECKPOINT
        || binding["preparation"] != *prep
        || stored.learning_contract()? != cfg.learning_contract()?
        || binding["learning_contract"] != cfg.learning_contract()?
        || binding["source_commit"] != source
        || binding["executable_sha256"] != executable
        || binding["campaign_sha256"] != sha256_file(&path.join("campaign.json"))?
        || binding["variables_sha256"] != sha256_file(&path.join("rounding-variables.safetensors"))?
    {
        return Err(invalid("rounding resume identity/recipe changed"));
    }
    let values =
        candle_core::safetensors::load(path.join("rounding-variables.safetensors"), &Device::Cpu)?;
    if values.len() != learner.variables().len() {
        return Err(invalid("alpha resume inventory"));
    }
    for (name, var) in learner.variables() {
        let value = values.get(name).ok_or_else(|| invalid("missing alpha"))?;
        if value.dims() != var.dims()
            || value.dtype() != candle_core::DType::F32
            || value
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite() || x.abs() > 30.0)
        {
            return Err(invalid("alpha resume shape/value"));
        }
    }
    for (name, var) in learner.variables() {
        var.set(values.get(name).ok_or_else(|| invalid("missing alpha"))?)?;
    }
    let optimizer = NamedAdamW::load(path, learner.variables(), &cfg.optimizer)?;
    if optimizer.step_count() != progress.completed_updates as u64
        || binding["optimizer"]["step"] != progress.completed_updates
        || binding["optimizer"]["moments_sha256"]
            != sha256_file(&path.join("optimizer.safetensors"))?
        || binding["optimizer_fingerprint"] != optimizer.continuity_fingerprint()?
    {
        return Err(invalid("alpha resume optimizer binding"));
    }
    Ok((optimizer, progress))
}

fn stopped(cfg: &Campaign, started: Instant) -> bool {
    started.elapsed().as_secs_f64() >= cfg.max_process_seconds
        || cfg.stop_file.as_ref().is_some_and(|p| p.exists())
}
fn fit(
    cfg: &Campaign,
    parent: &DialogueRoundingParent,
    index: &EpisodeIndex<'_>,
    out: &Path,
    source: &str,
    executable: &str,
    started: Instant,
) -> Result<Value> {
    let preparation = serde_json::to_value(parent.provenance())?;
    validate_normalization(cfg, &preparation, source, executable)?;
    let learner = parent.new_learner(cfg.rounding.clone())?;
    let (mut optimizer, mut progress) = match &cfg.resume_from {
        Some(path) => resume(path, cfg, &preparation, &learner, source, executable)?,
        None => (
            NamedAdamW::new(learner.variables(), cfg.optimizer.clone())?,
            Progress::initial(cfg),
        ),
    };
    let initial = progress.clone();
    let mut curve = fs::File::create_new(out.join("learning-curve.jsonl"))?;
    while progress.completed_updates < cfg.rounding.steps && !stopped(cfg, started) {
        let clock = Instant::now();
        let ids = index.sample_ids(cfg.data_seed, progress.next_data_step as u64, cfg.batch)?;
        let batch = index.materialize(&ids, PrefixPolicy::FullPrefix)?;
        let mut gradients =
            response_gradients(parent.model(), &learner, &batch, cfg.cpu_gradient_shards)?;
        if gradients.supervised_targets != batch.counts.supervised_target_count
            || !gradients.mean_nll.is_finite()
        {
            return Err(invalid("response loss/count mismatch"));
        }
        let penalty = add_penalty(&learner, progress.completed_updates, &mut gradients)?;
        let update = optimizer.step(learner.variables(), &gradients.gradients)?;
        let data_step = progress.next_data_step;
        progress.advance(cfg, &batch)?;
        let row = json!({"completed_updates":progress.completed_updates,"data_step":data_step,
            "response_ids":ids,"target_sha256_le_u32":target_hash(&batch),"counts":batch.counts,
            "source_visits":batch.source_visits,"supervised_target_visits":progress.supervised_target_visits,
            "padded_position_visits":progress.padded_position_visits,"executed_schedule_sha256":progress.executed_schedule_sha256,
            "batch_mean_nll":gradients.mean_nll,"weighted_rounding_penalty":penalty,
            "objective":f64::from(gradients.mean_nll)+f64::from(penalty),
            "beta":cfg.rounding.beta(progress.completed_updates-1),"optimizer":update,
            "complete_step_seconds":clock.elapsed().as_secs_f64()});
        serde_json::to_writer(&mut curve, &row)?;
        curve.write_all(b"\n")?;
        curve.flush()?;
        if progress.completed_updates == initial.completed_updates + 1
            || progress.completed_updates % 16 == 0
        {
            eprintln!(
                "dialogue rounding {}/{} NLL {:.6} penalty {:.6}",
                progress.completed_updates, cfg.rounding.steps, gradients.mean_nll, penalty
            );
        }
        drop(gradients);
        if cfg.checkpoint_steps.contains(&progress.completed_updates) {
            checkpoint(
                &out.join(format!("checkpoint-{:08}", progress.completed_updates)),
                cfg,
                &preparation,
                &learner,
                &optimizer,
                &progress,
                source,
                executable,
            )?;
        }
    }
    curve.sync_all()?;
    let final_root = out.join("checkpoint-final");
    let final_binding = checkpoint(
        &final_root,
        cfg,
        &preparation,
        &learner,
        &optimizer,
        &progress,
        source,
        executable,
    )?;
    let complete = progress.completed_updates == cfg.rounding.steps;
    let mut result = json!({"schema":"uor-r4.dialogue-rounding-result/1",
        "status":if complete {"COMPLETE_FIXED_RECIPE"} else {"STOPPED_CHECKPOINTED"},
        "source_commit":source,"executable_sha256":executable,"campaign":cfg,"preparation":preparation,
        "starting_alpha_step":initial.completed_updates,"completed_updates":progress.completed_updates,
        "new_updates_this_attempt":progress.completed_updates-initial.completed_updates,
        "new_supervised_targets_this_attempt":progress.supervised_target_visits-initial.supervised_target_visits,
        "new_padded_positions_this_attempt":progress.padded_position_visits-initial.padded_position_visits,
        "progress":progress,"final_checkpoint":final_binding,"statistics":learner.statistics()?,
        "parent_model_clock":parent.child().steps_completed,"parent_model_clock_unchanged":true,
        "normalization_backward_presentations_this_attempt":0,
        "training_seconds_including_preparation_and_closeout_before_export":started.elapsed().as_secs_f64(),
        "scope":"Training-only FullPrefix response/EOS code choices. No native generation, quality selection or held-out evaluation in this process."});
    if complete {
        let packed = out.join("packed-model");
        report_output::claim(&packed)?;
        let hashes = source_hashes();
        let exported = parent.save_learned_hard(
            &packed,
            &learner,
            &final_root,
            source,
            executable,
            &hashes,
        )?;
        report_output::seal(&packed)?;
        report_output::verify(&packed)?;
        result["packed_manifest"] = exported;
        result["packed_root"] = json!(packed);
    }
    result["wall_seconds"] = json!(started.elapsed().as_secs_f64());
    save(&out.join("result.json"), &result)?;
    Ok(result)
}

/// Pool detached named gradients by response count, not batch count or mean
/// norms. Each previous full graph is dropped before the next presentation.
fn accumulate(
    totals: &mut BTreeMap<String, Tensor>,
    learner: &LearnedRounding,
    gradients: &ResponseGradients,
) -> Result<()> {
    for (name, var) in learner.variables() {
        let value = gradients
            .gradients
            .get(var.as_tensor())
            .ok_or_else(|| invalid("normalization gradient"))?
            .detach()
            .affine(gradients.supervised_targets as f64, 0.0)?
            .detach();
        let value = if let Some(old) = totals.remove(name) {
            old.add(&value)?.detach()
        } else {
            value
        };
        totals.insert(name.clone(), value);
    }
    Ok(())
}
fn normalize(
    cfg: &Campaign,
    parent: &DialogueRoundingParent,
    index: &EpisodeIndex<'_>,
    out: &Path,
    source: &str,
    executable: &str,
    started: Instant,
) -> Result<Value> {
    let mut probe = cfg.rounding.clone();
    probe.warmup_steps = 0;
    probe.beta_start = 2.0;
    probe.beta_end = 2.0;
    probe.regularization = 1.0;
    let learner = parent.new_learner(probe)?;
    let mut totals = BTreeMap::new();
    let mut count = 0usize;
    let mut loss = 0f64;
    let mut presentations = Vec::new();
    let mut chain = Progress::initial(cfg).executed_schedule_sha256;
    for n in 0..NORMALIZATION_BATCHES {
        if stopped(cfg, started) {
            return Err(invalid(
                "normalization stopped before all eight presentations; no coefficient selected",
            ));
        }
        let clock = Instant::now();
        let step = cfg.data_start_step + n;
        let ids = index.sample_ids(cfg.data_seed, step as u64, cfg.batch)?;
        let batch = index.materialize(&ids, PrefixPolicy::FullPrefix)?;
        let gradients =
            response_gradients(parent.model(), &learner, &batch, cfg.cpu_gradient_shards)?;
        if !gradients.mean_nll.is_finite()
            || gradients.supervised_targets != batch.counts.supervised_target_count
        {
            return Err(invalid("normalization response loss/count"));
        }
        accumulate(&mut totals, &learner, &gradients)?;
        count += gradients.supervised_targets;
        loss += f64::from(gradients.mean_nll) * gradients.supervised_targets as f64;
        chain = advance_schedule(&chain, step, &ids, &target_hash(&batch))?;
        presentations.push(
            json!({"data_step":step,"response_ids":ids,"target_sha256_le_u32":target_hash(&batch),
            "counts":batch.counts,"source_visits":batch.source_visits,"mean_nll":gradients.mean_nll,
            "seconds":clock.elapsed().as_secs_f64()}),
        );
        drop(gradients);
        eprintln!(
            "dialogue rounding normalization {}/{NORMALIZATION_BATCHES}",
            n + 1
        );
    }
    let penalty = learner.penalty(0)?.backward()?;
    let mut task_square = 0f64;
    let mut penalty_square = 0f64;
    let mut by_parameter = serde_json::Map::new();
    for (name, var) in learner.variables() {
        let task = totals
            .get(name)
            .ok_or_else(|| invalid("pooled gradient missing"))?
            .affine(1.0 / count as f64, 0.0)?;
        let reg = penalty
            .get(var.as_tensor())
            .ok_or_else(|| invalid("unit penalty gradient missing"))?;
        let a = f64::from(task.sqr()?.sum_all()?.to_scalar::<f32>()?);
        let b = f64::from(reg.sqr()?.sum_all()?.to_scalar::<f32>()?);
        if !a.is_finite() || !b.is_finite() {
            return Err(invalid("nonfinite normalization norm"));
        }
        task_square += a;
        penalty_square += b;
        by_parameter.insert(
            name.clone(),
            json!({"task_gradient_l2":a.sqrt(),"unit_penalty_gradient_l2":b.sqrt()}),
        );
    }
    if task_square <= 0.0 || penalty_square <= 0.0 {
        return Err(invalid("zero normalization norm cannot select coefficient"));
    }
    let coefficient = (task_square / penalty_square).sqrt();
    let mut checked = cfg.rounding.clone();
    checked.regularization = coefficient;
    checked.validate()?;
    let report = json!({"schema":"uor-r4.dialogue-rounding-normalization/1","status":"NORMALIZED_NO_UPDATES",
        "source_commit":source,"executable_sha256":executable,"preparation":parent.provenance(),
        "normalization_contract":cfg.normalization_contract()?,"regularization":coefficient,
        "formula":"L2(sum(M_b * g_b) / sum(M_b)) / L2(global unit-coefficient beta2 penalty gradient), eight initial-soft training batches, no clipping or updates",
        "task_gradient_l2":task_square.sqrt(),"unit_beta2_penalty_gradient_l2":penalty_square.sqrt(),
        "by_parameter":by_parameter,"presentations":presentations,"executed_schedule_sha256":chain,
        "supervised_target_visits":count,"padded_position_visits":NORMALIZATION_BATCHES*cfg.batch*CONTEXT,
        "response_mean_nll":loss/count as f64,"optimizer_updates":0,"used_evaluation_data":false,
        "wall_seconds":started.elapsed().as_secs_f64(),
        "scope":"Initial gradient heuristic, not optimal regularization or future per-update gradient balance. These backward presentations are extra work, not optimizer exposure."});
    let report_path = out.join("normalization.json");
    save(&report_path, &report)?;
    // This receipt resolves only lambda. The principal freezes dose, schedule,
    // optimizer and checkpoint choices in a separately authored fit campaign.
    Ok(report)
}
fn validate_normalization(
    cfg: &Campaign,
    prep: &Value,
    source: &str,
    executable: &str,
) -> Result<()> {
    if let (Some(path), Some(hash)) = (&cfg.normalization_report, &cfg.normalization_report_sha256)
    {
        let root = path.parent().ok_or_else(|| invalid("normalization root"))?;
        report_output::verify(root)?;
        if root.join("failed-attempt.json").exists() || sha256_file(path)? != *hash {
            return Err(invalid("normalization receipt identity"));
        }
        let v: Value = serde_json::from_slice(&fs::read(path)?)?;
        if v["schema"] != "uor-r4.dialogue-rounding-normalization/1"
            || v["status"] != "NORMALIZED_NO_UPDATES"
            || v["preparation"] != *prep
            || v["normalization_contract"] != cfg.normalization_contract()?
            || v["regularization"] != json!(cfg.rounding.regularization)
            || v["optimizer_updates"] != 0
            || v["source_commit"] != source
            || v["executable_sha256"] != executable
        {
            return Err(invalid("normalization receipt differs from fitted recipe"));
        }
    }
    Ok(())
}
fn source_hashes() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "dialogue_rounding.rs".into(),
            hex::encode(Sha256::digest(include_bytes!("dialogue_rounding.rs"))),
        ),
        (
            "dialogue_rounding_artifact.rs".into(),
            hex::encode(Sha256::digest(include_bytes!(
                "dialogue_rounding_artifact.rs"
            ))),
        ),
        (
            "joint_rounding.rs".into(),
            hex::encode(Sha256::digest(include_bytes!("joint_rounding.rs"))),
        ),
        (
            "joint_parallel.rs".into(),
            hex::encode(Sha256::digest(include_bytes!("joint_parallel.rs"))),
        ),
    ])
}
/// Score only the original child's exposed development IDs. The same existing
/// scorer observes the frozen nearest and learned QQ packed-emulator artifacts;
/// native generation remains the separately bound observer's operation.
fn evaluate_endpoint(
    cfg: &Campaign,
    parent: &DialogueRoundingParent,
    packed: &Path,
    out: &Path,
    source: &str,
    executable: &str,
    started: Instant,
) -> Result<Value> {
    let child = parent.child();
    let contract = &child.learning_contract;
    let panel_path = child.report_root.join("development-panel.json");
    let panel_sha = sha256_file(&panel_path)?;
    if panel_sha != text(contract, "development_panel_sha256")? {
        return Err(invalid(
            "child saved panel differs from original frozen panel",
        ));
    }
    let panel: Value = serde_json::from_slice(&fs::read(&panel_path)?)?;
    let ids: Vec<usize> = serde_json::from_value(panel["response_ids"].clone())?;
    let (corpus, episode_contract) = corpus_split(parent, cfg, "heldout")?;
    let index = corpus.index(episode_contract)?;
    let binding = child.ancestor["datasets"]
        .as_array()
        .ok_or_else(|| invalid("child datasets"))?
        .iter()
        .find(|v| v["split"] == "heldout")
        .ok_or_else(|| invalid("development dataset binding"))?;
    let index_sha = digest(
        &json!({"contract":index.contract(),"episodes":index.episodes(),
        "sources":index.sources(),"population":index.population(),"binding":binding}),
    )?;
    if panel["schema"] != "uor-r4.dialogue-development-panel/1"
        || panel["selection"] != crate::dialogue_development::SELECTION_ID
        || panel["source_commit"] != child.training_source_commit
        || panel["development_index_sha256"] != index_sha
        || panel["train_index_sha256"] != cfg.train_index_sha256
        || panel["prepared_manifest_sha256"] != cfg.prepared_manifest_sha256
        || panel["tokenizer_sha256"] != cfg.tokenizer_sha256
        || panel["protocol_identity"] != child.protocol_identity
        || panel["parent_parameter_sha256"] != child.ancestor["parameter_sha256"]
        || panel["seed"] != contract["development_seed"]
        || panel["per_source"] != contract["development_per_source"]
        || ids.len() != number(contract, "development_episodes")?
    {
        return Err(invalid("original development panel identity differs"));
    }
    crate::dialogue_development::validate(
        &index,
        &ids,
        number(contract, "development_per_source")?,
    )?;
    let batch = number(contract, "development_batch")?;
    let model = parent.load_scoring_hard(packed)?;
    let manifest: Value = serde_json::from_slice(&fs::read(packed.join("hard-model.json"))?)?;
    save(&out.join("development-panel.json"), &panel)?;
    let evaluated = crate::dialogue_development::evaluate(&model, &index, &ids, batch)?;
    let report = json!({"schema":"uor-r4.dialogue-rounding-development/1","status":"COMPLETE",
        "source_commit":source,"executable_sha256":executable,"preparation":parent.provenance(),
        "child":child,"packed_root":packed,"packed_manifest_sha256":sha256_file(&packed.join("hard-model.json"))?,
        "packed_seal_sha256":sha256_file(&packed.join("manifest.json"))?,
        "conversion_stage":manifest["conversion_provenance"]["schema"],"precision":"QQ",
        "development_panel_sha256":panel_sha,"development_index_sha256":index_sha,"batch":batch,
        "selected_responses":ids.len(),"forward_batches":ids.len().div_ceil(batch),
        "evaluated_padded_positions":ids.len()*CONTEXT,"evaluation":evaluated,
        "optimizer_updates":0,"generation_calls":0,"wall_seconds":started.elapsed().as_secs_f64(),
        "scope":"Previously exposed heldout-named development, exact original child panel, no reselection. Fixed source/first-four response metrics from the unchanged scorer. QQ uses the hard F32 emulator, not native integer execution; no fresh final holdout or automatic acceptance."});
    save(&out.join("result.json"), &report)?;
    Ok(report)
}

/// Score the fixed development panel in a separate attempt after the fit.
pub fn evaluate(campaign_path: &Path, packed: &Path, out: &Path, source: &str) -> Result<Value> {
    run_operation(campaign_path, out, source, false, Some(packed))
}

/// Claim before parent loading; preserve and seal failed attempts. Each resume
/// has a new root and reuses only a separately sealed alpha checkpoint.
pub fn run(
    campaign_path: &Path,
    out: &Path,
    source: &str,
    normalization_only: bool,
) -> Result<Value> {
    run_operation(campaign_path, out, source, normalization_only, None)
}
fn run_operation(
    campaign_path: &Path,
    out: &Path,
    source: &str,
    normalization_only: bool,
    evaluation_packed: Option<&Path>,
) -> Result<Value> {
    let cfg: Campaign = serde_json::from_slice(&fs::read(campaign_path)?)?;
    cfg.validate()?;
    if source.len() != 40
        || !source.bytes().all(|b| b.is_ascii_hexdigit())
        || (normalization_only && (cfg.resume_from.is_some() || cfg.normalization_report.is_some()))
    {
        return Err(invalid("source binding or normalization mode arguments"));
    }
    report_output::claim(out)?;
    let started = Instant::now();
    let result = (|| {
        let executable = sha256_file(&std::env::current_exe()?)?;
        save(&out.join("campaign.json"), &serde_json::to_value(&cfg)?)?;
        save(
            &out.join("execution-binding.json"),
            &json!({"source_commit":source,"executable_sha256":executable,
            "source_sha256":source_hashes(),"cpu_accelerate_compiled":cfg!(feature="cpu-accelerate"),
            "response_gradient_execution":if evaluation_packed.is_some() {"not-run-evaluation-only"} else {"sequential-full-sequence-shards/immediate-named-reduction/v1"},
            "supplied_campaign_sha256":sha256_file(campaign_path)?,"saved_campaign_sha256":sha256_file(&out.join("campaign.json"))?,
            "normalization_only":normalization_only,"evaluation_packed":evaluation_packed}),
        )?;
        let parent = load_parent(&cfg)?;
        save(
            &out.join("preparation.json"),
            &serde_json::to_value(parent.provenance())?,
        )?;
        if let Some(packed) = evaluation_packed {
            return evaluate_endpoint(&cfg, &parent, packed, out, source, &executable, started);
        }
        let (corpus, contract) = corpus_split(&parent, &cfg, "train")?;
        let index = corpus.index(contract)?;
        if normalization_only {
            normalize(&cfg, &parent, &index, out, source, &executable, started)
        } else {
            fit(&cfg, &parent, &index, out, source, &executable, started)
        }
    })();
    if let Err(error) = &result {
        save(
            &out.join("failed-attempt.json"),
            &json!({"status":"UNVERIFIED","error":error.to_string(),
            "source_commit":source,"wall_seconds":started.elapsed().as_secs_f64(),
            "scope":"Execution failure, not model-quality evidence. Sealed checkpoints remain separately recoverable."}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_model::{JointConfig, ReadMode};
    fn recipe() -> RoundingConfig {
        // Fixture constants, never a runnable campaign default.
        RoundingConfig {
            steps: 3,
            warmup_steps: 0,
            beta_start: 2.0,
            beta_end: 2.0,
            regularization: 0.1,
        }
    }
    fn parent(width: usize) -> Result<JointModel> {
        let mut model = if width == 576 {
            let cfg = JointConfig {
                width,
                ..JointConfig::default()
            };
            let mut state = 0x293741a5u64;
            let arrays = cfg
                .shapes()
                .into_iter()
                .map(|(name, shape)| {
                    let values = (0..shape.iter().product::<usize>())
                        .map(|_| {
                            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                            ((state >> 40) as f32 / 16777216.0 - 0.5) * 0.12
                        })
                        .collect();
                    (name, values)
                })
                .collect();
            JointModel::from_offline_dialogue_parameters(cfg, arrays, &Device::Cpu)?
        } else {
            JointModel::new(
                JointConfig {
                    width,
                    context: 8,
                    seed: 77,
                    ..JointConfig::default()
                },
                &Device::Cpu,
            )?
        };
        model.configure_quantization(0, 1)?;
        model.set_completed_step(1)?;
        model
            .quantization()
            .ok_or_else(|| invalid("test grid"))?
            .spec
            .project_parameters(model.variables())?;
        Ok(model)
    }
    fn masked() -> (Vec<u32>, Vec<u32>, Vec<f32>) {
        (
            vec![0, 4, 8, 4, 9, 8, 7, 9, 0, 7, 4, 8, 4, 7, 9, 8],
            vec![4, 8, 4, 9, 8, 7, 9, 1, 7, 4, 8, 4, 7, 9, 8, 1],
            vec![
                0., 0., 0., 0., 0., 0., 1., 1., 0., 0., 1., 1., 1., 1., 1., 1.,
            ],
        )
    }
    fn maximum_difference(a: &Tensor, b: &Tensor) -> Result<f32> {
        Ok(a.sub(b)?.abs()?.max_all()?.to_scalar::<f32>()?)
    }
    #[test]
    fn dialogue_rounding_response_alpha_global_objective() -> Result<()> {
        let parent = parent(576)?;
        let original = crate::dialogue_child_artifact::parameter_bindings(&parent)?.0;
        let parallel_learner = learner(&parent, recipe())?;
        let learner = learner(&parent, recipe())?;
        let (ids, targets, weights) = masked();
        let view = parent
            .rounding_learning_view(learner.variables().clone(), learner.parameters(false)?)?;
        let output = view.forward(&ids, 2, 8, ReadMode::Enabled, true)?;
        let loss = output.weighted_loss(&targets, &weights)?;
        let direct = loss.add(&learner.penalty(0)?)?.backward()?;
        let mut reduced =
            sequential_response_batch_gradients(&view, &ids, &targets, &weights, 2, 8, 2)?;
        let parallel_view = parent.rounding_learning_view(
            parallel_learner.variables().clone(),
            parallel_learner.parameters(false)?,
        )?;
        let mut parallel = crate::joint_parallel::response_batch_gradients(
            &parallel_view,
            &ids,
            &targets,
            &weights,
            2,
            8,
            2,
        )?;
        assert_eq!(reduced.supervised_targets, 8);
        assert_eq!(parallel.supervised_targets, reduced.supervised_targets);
        assert_eq!(parallel.mean_nll.to_bits(), reduced.mean_nll.to_bits());
        assert!((reduced.mean_nll - loss.to_scalar::<f32>()?).abs() < 2e-5);
        // Identical two-shard shapes and reduction order change only graph
        // concurrency. Require exact equality on this pinned CPU fixture;
        // this is not a cross-backend bitwise-training guarantee.
        for (name, var) in learner.variables() {
            let reference_var = parallel_learner
                .variables()
                .get(name)
                .ok_or_else(|| invalid("parallel fixture alpha"))?;
            let actual = reduced
                .gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("sequential fixture gradient"))?;
            let expected = parallel
                .gradients
                .get(reference_var.as_tensor())
                .ok_or_else(|| invalid("parallel fixture gradient"))?;
            assert_eq!(maximum_difference(actual, expected)?, 0.0, "{name}");
        }
        let task_norms: [f32; 4] = [
            "embedding.weight",
            "recurrent.state.weight",
            "read.query.weight",
            "read.key.weight",
        ]
        .map(|name| -> Result<f32> {
            let var = learner
                .variables()
                .get(name)
                .ok_or_else(|| invalid("fixture alpha"))?;
            Ok(reduced
                .gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("fixture gradient"))?
                .sqr()?
                .sum_all()?
                .to_scalar::<f32>()?)
        })
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| invalid("fixture gradient count"))?;
        assert!(
            task_norms.iter().all(|v| v.is_finite() && *v > 0.0),
            "{task_norms:?}"
        );
        let task_before = learner
            .variables()
            .iter()
            .map(|(name, var)| {
                Ok((
                    name.clone(),
                    reduced
                        .gradients
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("task gradient before penalty"))?
                        .detach(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let unit_once = learner.penalty(0)?.backward()?;
        let reg = add_penalty(&learner, 0, &mut reduced)?;
        assert_eq!(
            reg.to_bits(),
            add_penalty(&parallel_learner, 0, &mut parallel)?.to_bits()
        );
        let mut penalty_error = 0f64;
        let mut penalty_norm = 0f64;
        assert!(reg > 0.0);
        for (name, var) in learner.variables() {
            let expected = direct
                .get(var.as_tensor())
                .ok_or_else(|| invalid("direct alpha gradient"))?;
            let actual = reduced
                .gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("reduced alpha gradient"))?;
            assert!(maximum_difference(actual, expected)? < 3e-5, "{name}");
            let once = unit_once
                .get(var.as_tensor())
                .ok_or_else(|| invalid("single penalty gradient"))?;
            let delta = actual.sub(
                task_before
                    .get(name)
                    .ok_or_else(|| invalid("task snapshot"))?,
            )?;
            penalty_error += f64::from(delta.sub(once)?.sqr()?.sum_all()?.to_scalar::<f32>()?);
            penalty_norm += f64::from(once.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
        // A loose per-coordinate task tolerance cannot detect duplicated tiny
        // global-mean penalty gradients; compare their aggregate residual too.
        assert!(penalty_norm > 0.0 && penalty_error / penalty_norm < 1e-5);
        let adam_config = AdamConfig {
            weight_decay: 0.,
            parameter_abs_limit: 30.,
            ..AdamConfig::default()
        };
        let mut adam = NamedAdamW::new(learner.variables(), adam_config.clone())?;
        let mut parallel_adam = NamedAdamW::new(parallel_learner.variables(), adam_config)?;
        adam.step(learner.variables(), &reduced.gradients)?;
        parallel_adam.step(parallel_learner.variables(), &parallel.gradients)?;
        assert_eq!(adam.step_count(), 1);
        assert_eq!(parallel_adam.step_count(), 1);
        for (name, var) in learner.variables() {
            let expected = parallel_learner
                .variables()
                .get(name)
                .ok_or_else(|| invalid("parallel updated alpha"))?;
            assert_eq!(maximum_difference(var, expected)?, 0.0, "{name}");
        }
        assert_eq!(
            crate::dialogue_child_artifact::parameter_bindings(&parent)?.0,
            original
        );
        assert_eq!(
            parent
                .quantization()
                .ok_or_else(|| invalid("fixture clock"))?
                .completed_step,
            1
        );
        Ok(())
    }
    fn fixture_campaign() -> Campaign {
        Campaign {
            schema: SCHEMA.into(),
            child_report_root: "unused".into(),
            prepared_manifest: "unused".into(),
            tokenizer_path: "unused".into(),
            nearest_packed_root: "unused".into(),
            child_parameter_sha256: "1".repeat(64),
            prepared_manifest_sha256: "2".repeat(64),
            tokenizer_sha256: "3".repeat(64),
            nearest_hard_manifest_sha256: "4".repeat(64),
            train_index_sha256: "5".repeat(64),
            rounding: recipe(),
            optimizer: AdamConfig {
                weight_decay: 0.,
                parameter_abs_limit: 30.,
                ..AdamConfig::default()
            },
            data_seed: 17,
            data_start_step: 1024,
            batch: 2,
            cpu_gradient_shards: 2,
            checkpoint_steps: vec![],
            normalization_report: None,
            normalization_report_sha256: None,
            max_process_seconds: 60.,
            stop_file: None,
            resume_from: None,
        }
    }
    #[test]
    fn dialogue_rounding_checkpoint_resume_recipe() -> Result<()> {
        let parent = parent(128)?;
        let cfg = fixture_campaign();
        cfg.validate()?;
        let learner = learner(&parent, cfg.rounding.clone())?;
        let mut adam = NamedAdamW::new(learner.variables(), cfg.optimizer.clone())?;
        // A penalty-only serialization fixture keeps this test small. The
        // separate width576 test exercises the complete response objective.
        adam.step(learner.variables(), &learner.penalty(0)?.backward()?)?;
        let progress = Progress {
            completed_updates: 1,
            next_data_step: 1025,
            supervised_target_visits: 8,
            padded_position_visits: 512,
            executed_schedule_sha256: "a".repeat(64),
        };
        let prep = json!({"scope":"synthetic immutable preparation binding"});
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("dialogue-rounding-{}-{stamp}", std::process::id()));
        checkpoint(
            &path,
            &cfg,
            &prep,
            &learner,
            &adam,
            &progress,
            &"b".repeat(40),
            &"c".repeat(64),
        )?;
        let restored = super::learner(&parent, cfg.rounding.clone())?;
        let (mut resumed, got) = resume(
            &path,
            &cfg,
            &prep,
            &restored,
            &"b".repeat(40),
            &"c".repeat(64),
        )?;
        assert_eq!(
            serde_json::to_value(&got)?,
            serde_json::to_value(&progress)?
        );
        assert_eq!(
            resumed.continuity_fingerprint()?,
            adam.continuity_fingerprint()?
        );
        adam.step(learner.variables(), &learner.penalty(1)?.backward()?)?;
        resumed.step(restored.variables(), &restored.penalty(1)?.backward()?)?;
        assert_eq!(
            resumed.continuity_fingerprint()?,
            adam.continuity_fingerprint()?
        );
        for (name, var) in learner.variables() {
            assert_eq!(
                var.flatten_all()?.to_vec1::<f32>()?,
                restored.variables()[name].flatten_all()?.to_vec1::<f32>()?
            );
        }
        let mut changed = cfg.clone();
        changed.rounding.beta_start = 3.;
        assert!(resume(
            &path,
            &changed,
            &prep,
            &restored,
            &"b".repeat(40),
            &"c".repeat(64)
        )
        .is_err());
        assert!(resume(
            &path,
            &cfg,
            &json!({"different":"parent"}),
            &restored,
            &"b".repeat(40),
            &"c".repeat(64)
        )
        .is_err());
        let mut extended = cfg.clone();
        extended.max_process_seconds = 90.;
        extended.resume_from = Some(path.clone());
        assert_eq!(extended.learning_contract()?, cfg.learning_contract()?);
        // Exposure/schedule state evolves identically across an attempt boundary.
        let mut a = progress.clone();
        let mut b = got;
        let batch = EpisodeBatch {
            policy: PrefixPolicy::FullPrefix,
            batch: 2,
            time: 256,
            inputs: vec![],
            targets: vec![],
            weights: vec![],
            response_ids: vec![3, 7],
            selected_target_ids: vec![4, 1, 9, 1],
            rows: vec![],
            source_visits: vec![],
            counts: crate::dialogue_episodes::EpisodeCounts {
                supervised_target_count: 4,
                ..Default::default()
            },
        };
        a.advance(&cfg, &batch)?;
        b.advance(&cfg, &batch)?;
        assert_eq!(serde_json::to_value(a)?, serde_json::to_value(b)?);
        Ok(())
    }
    #[test]
    fn dialogue_rounding_normalization_response_pool() -> Result<()> {
        let parent = parent(128)?;
        let learner = learner(&parent, recipe())?;
        let mut totals = BTreeMap::new();
        for (mass, value) in [(2, 1.), (6, 3.)] {
            let mut gradients = ResponseGradients {
                mean_nll: 0.,
                supervised_targets: mass,
                gradients: learner.penalty(0)?.backward()?,
            };
            for var in learner.variables().values() {
                gradients.gradients.insert(
                    var.as_tensor(),
                    var.zeros_like()?.affine(0., value)?.detach(),
                );
            }
            accumulate(&mut totals, &learner, &gradients)?;
        }
        for total in totals.values() {
            let pooled = total.affine(1. / 8., 0.)?;
            assert!(
                pooled
                    .affine(1., -2.5)?
                    .abs()?
                    .max_all()?
                    .to_scalar::<f32>()?
                    < 1e-6
            );
        }
        let cfg = fixture_campaign();
        let mut resolved = cfg.clone();
        resolved.rounding.regularization = 7.25;
        assert_eq!(
            resolved.normalization_contract()?,
            cfg.normalization_contract()?
        );
        resolved.rounding.steps += 3;
        resolved.rounding.beta_start = 4.;
        resolved.optimizer.learning_rate *= 2.;
        resolved.checkpoint_steps = vec![1];
        assert_eq!(
            resolved.normalization_contract()?,
            cfg.normalization_contract()?
        );
        resolved.data_seed += 1;
        assert_ne!(
            resolved.normalization_contract()?,
            cfg.normalization_contract()?
        );
        Ok(())
    }
}
