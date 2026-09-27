//! Offline complete-response learning from the retained native dialogue model.
//! Prefix policy changes conditioning, never selected response/EOS exposure.
use crate::{
    dialogue_artifact::{DatasetBinding, LegacyDialogueArtifact},
    dialogue_episodes::{EpisodeContract, EpisodeIndex, PrefixPolicy, SourceSpan, SAMPLER_ID},
    invalid,
    joint_evaluation::{self, JointGenerationStop},
    joint_model::{JointModel, ReadMode},
    joint_optimizer::{AdamConfig, NamedAdamW},
    joint_parallel::response_batch_gradients,
    sha256_file, Result,
};
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{native_geometric::mmap_corpus::MmapCorpusReader, report_output};
use uor_r4_tokenizer::dialogue::DialogueProtocol;

const SCHEMA: &str = "uor-r4.dialogue-prefix-campaign/1";
const CHECKPOINT: &str = "uor-r4.dialogue-prefix-checkpoint/1";
const CONTEXT: usize = 256;
const SCHEDULE_CHAIN: &str = "uor-r4.dialogue-response-schedule-chain/1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub schema: String,
    pub parent_fit_root: PathBuf,
    pub prepared_manifest: PathBuf,
    pub tokenizer_path: PathBuf,
    pub parent_parameter_sha256: String,
    pub prepared_manifest_sha256: String,
    pub tokenizer_sha256: String,
    pub policy: PrefixPolicy,
    pub data_seed: u64,
    pub total_steps: usize,
    pub batch: usize,
    pub cpu_gradient_shards: usize,
    pub expected_train_response_runs: usize,
    pub expected_eligible_train_responses: usize,
    pub optimizer: AdamConfig,
    pub development_seed: u64,
    pub development_episodes: usize,
    pub development_batch: usize,
    pub development_every_steps: usize,
    pub checkpoint_every_steps: usize,
    /// Includes preparation and periodic evaluation before admitting an update.
    /// The external supervisor separately reserves checkpoint/output closeout.
    pub max_process_seconds: f64,
    pub stop_file: Option<PathBuf>,
    pub resume_from: Option<PathBuf>,
    pub requests_path: PathBuf,
    pub requests_sha256: String,
    pub max_new_tokens: usize,
}
impl Campaign {
    fn validate(&self) -> Result<()> {
        if self.schema != SCHEMA
            || self.total_steps == 0
            || self.total_steps > 1_000_000
            || self.batch == 0
            || self.batch > 64
            || !matches!(self.cpu_gradient_shards, 1 | 2 | 4)
            || self.batch % self.cpu_gradient_shards != 0
            || self.expected_eligible_train_responses == 0
            || self.expected_train_response_runs < self.expected_eligible_train_responses
            || self.development_episodes == 0
            || self.development_episodes > 4096
            || self.development_batch == 0
            || self.development_batch > 16
            || !self.max_process_seconds.is_finite()
            || self.max_process_seconds <= 0.0
            || self.max_new_tokens == 0
            || self.max_new_tokens > 128
        {
            return Err(invalid("invalid bounded complete-prefix campaign"));
        }
        for h in [
            &self.parent_parameter_sha256,
            &self.prepared_manifest_sha256,
            &self.tokenizer_sha256,
            &self.requests_sha256,
        ] {
            if h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid("campaign SHA256 binding"));
            }
        }
        self.optimizer.validate()?;
        if !self.optimizer.allowed_missing_gradients.is_empty() {
            return Err(invalid("dialogue learning requires every named gradient"));
        }
        Ok(())
    }
    fn learning_contract(&self) -> Result<Value> {
        let mut v = serde_json::to_value(self)?;
        let object = v
            .as_object_mut()
            .ok_or_else(|| invalid("campaign object"))?;
        // Resource continuation cannot change the selected endpoint/objective.
        for key in ["resume_from", "max_process_seconds", "stop_file"] {
            object.remove(key);
        }
        Ok(v)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    id: String,
    category: String,
    user_turns: Vec<String>,
}

fn save(path: &Path, value: &Value) -> Result<()> {
    let mut f = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut f, value)?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    Ok(())
}
fn digest<T: Serialize + ?Sized>(value: &T) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(value)?)))
}

fn advance_schedule(
    previous: &str,
    step: usize,
    ids: &[usize],
    target_sha: &str,
) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(previous.as_bytes());
    hash.update(serde_json::to_vec(
        &json!({"step":step,"response_ids":ids,"targets_sha256":target_sha}),
    )?);
    Ok(hex::encode(hash.finalize()))
}
fn count(v: &Value, field: &str) -> Result<usize> {
    v[field]
        .as_u64()
        .and_then(|x| usize::try_from(x).ok())
        .ok_or_else(|| invalid(format!("missing bounded count {field}")))
}

struct Corpus {
    reader: MmapCorpusReader,
    masks: Vec<u8>,
    sources: Vec<SourceSpan>,
}
impl Corpus {
    fn load(binding: &DatasetBinding, prepared: &Value, manifest_path: &Path) -> Result<Self> {
        let split = &prepared[&binding.split];
        let root = manifest_path
            .parent()
            .ok_or_else(|| invalid("prepared manifest root"))?;
        let local: Value =
            serde_json::from_slice(&fs::read(root.join(&binding.split).join("manifest.json"))?)?;
        if &local != split
            || split["drops"]["special_token_occurrences"] != 0
            || sha256_file(&binding.tokens_path)? != binding.tokens_sha256
            || sha256_file(&binding.mask_path)? != binding.mask_sha256
        {
            return Err(invalid(
                "episode corpus manifest/content/special-token binding",
            ));
        }
        let reader = MmapCorpusReader::open(&binding.tokens_path)
            .map_err(|e| invalid(format!("dialogue corpus: {e}")))?;
        let masks = fs::read(&binding.mask_path)?;
        if reader.as_slice().len() != binding.tokens
            || masks.len() != binding.tokens
            || reader.vocab_size() != 4096
            || !cfg!(target_endian = "little")
        {
            return Err(invalid("episode corpus lengths/vocabulary/endian"));
        }
        let mut sources = Vec::new();
        let mut start = 0usize;
        for file in split["files"]
            .as_array()
            .ok_or_else(|| invalid("episode source manifest"))?
        {
            let end = start
                .checked_add(count(file, "tokens")?)
                .ok_or_else(|| invalid("source length overflow"))?;
            if file["special_token_occurrences"] != 0 {
                return Err(invalid("literal special token in source"));
            }
            sources.push(SourceSpan {
                label: file["label"]
                    .as_str()
                    .ok_or_else(|| invalid("source label"))?
                    .into(),
                start,
                end,
            });
            start = end;
        }
        Ok(Self {
            reader,
            masks,
            sources,
        })
    }
    fn index(&self, contract: EpisodeContract) -> Result<EpisodeIndex<'_>> {
        EpisodeIndex::new(self.reader.as_slice(), &self.masks, contract, &self.sources)
    }
}

fn evaluate(
    model: &JointModel,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch_size: usize,
) -> Result<Value> {
    let started = Instant::now();
    let mut loss_sum = 0.0;
    let mut targets = 0usize;
    let mut early_sum = 0.0;
    let mut early_targets = 0usize;
    for chunk in ids.chunks(batch_size) {
        // Both learned conditions are evaluated under the same true request.
        let batch = index.materialize(chunk, PrefixPolicy::FullPrefix)?;
        let n = batch.weights.iter().filter(|&&w| w == 1.0).count();
        let output = model.forward(
            &batch.inputs,
            batch.batch,
            batch.time,
            ReadMode::Enabled,
            false,
        )?;
        let loss = output
            .weighted_loss(&batch.targets, &batch.weights)?
            .to_scalar::<f32>()?;
        let mut early = vec![0.0; batch.weights.len()];
        for (row, mask) in batch.weights.chunks(batch.time).enumerate() {
            for (position, _) in mask.iter().enumerate().filter(|(_, w)| **w == 1.0).take(4) {
                early[row * batch.time + position] = 1.0;
            }
        }
        let en = early.iter().filter(|&&w| w == 1.0).count();
        let el = output
            .weighted_loss(&batch.targets, &early)?
            .to_scalar::<f32>()?;
        if !loss.is_finite() || !el.is_finite() {
            return Err(invalid("nonfinite dialogue evaluation"));
        }
        loss_sum += f64::from(loss) * n as f64;
        targets += n;
        early_sum += f64::from(el) * en as f64;
        early_targets += en;
    }
    if targets == 0 {
        return Err(invalid("empty dialogue evaluation population"));
    }
    Ok(
        json!({"response_mean_nll":loss_sum/targets as f64,"supervised_targets":targets,
        "first_four_response_targets_mean_nll":early_sum/early_targets as f64,"first_four_targets":early_targets,
        "response_ids":ids,"conditioning":"full_original_prefix","elapsed_seconds":started.elapsed().as_secs_f64(),
        "scope":"Fixed exposed development response population, not a fresh held-out capability test."}),
    )
}

fn parameter_fingerprint(model: &JointModel) -> Result<String> {
    let mut h = Sha256::new();
    for (name, variable) in model.variables() {
        h.update((name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
        for value in variable.flatten_all()?.to_vec1::<f32>()? {
            h.update(value.to_le_bytes());
        }
    }
    Ok(hex::encode(h.finalize()))
}

fn checkpoint(
    out: &Path,
    name: &str,
    model: &JointModel,
    optimizer: &NamedAdamW,
    cfg: &Campaign,
    source: &str,
    index_sha: &str,
    provenance: &Value,
    step: usize,
    supervised: u64,
    sample_digest: &str,
) -> Result<PathBuf> {
    let path = out.join(name);
    report_output::claim(&path)?;
    model.save(&path)?;
    let adam = optimizer.save(&path)?;
    save(
        &path.join("checkpoint.json"),
        &json!({"schema":CHECKPOINT,"source_commit":source,
        "learning_contract":cfg.learning_contract()?,"parent":provenance,"train_index_sha256":index_sha,
        "sampler":SAMPLER_ID,"schedule_chain":SCHEDULE_CHAIN,"next_data_step":step,"optimizer_step":optimizer.step_count(),
        "supervised_target_visits":supervised,"executed_schedule_sha256":sample_digest,
        "parameter_fingerprint":parameter_fingerprint(model)?,"optimizer":adam,
        "optimizer_fingerprint":optimizer.continuity_fingerprint()?,
        "lineage":"Independent retained parameter copy with fresh Adam at step0; later checkpoints preserve this new lineage."}),
    )?;
    report_output::seal(&path)?;
    report_output::verify(&path)?;
    Ok(path)
}

/// Claim once after argument validation, before loading any model. Failed
/// attempts are sealed; a retry always requires a different output root.
pub fn run(campaign_path: &Path, out: &Path, source: &str) -> Result<Value> {
    let bytes = fs::read(campaign_path)?;
    let cfg: Campaign = serde_json::from_slice(&bytes)?;
    cfg.validate()?;
    if source.len() != 40 || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("build with full UOR_BUILD_SOURCE_COMMIT"));
    }
    report_output::claim(out)?;
    let result = run_claimed(&cfg, &bytes, out, source);
    if let Err(error) = &result {
        save(
            &out.join("failed-attempt.json"),
            &json!({"status":"UNVERIFIED","source_commit":source,
            "error":error.to_string(),"scope":"Execution failure, not model-quality evidence; sealed child checkpoints remain separately recoverable."}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn run_claimed(cfg: &Campaign, cfg_bytes: &[u8], out: &Path, source: &str) -> Result<Value> {
    let clock = Instant::now();
    save(&out.join("campaign.json"), &serde_json::to_value(cfg)?)?;
    save(
        &out.join("execution-binding.json"),
        &json!({"source_commit":source,
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "campaign_sha256":hex::encode(Sha256::digest(cfg_bytes)),"cpu_accelerate_compiled":cfg!(feature="cpu-accelerate"),
        "deadline_scope":"max_process_seconds limits admission of new updates, including preparation/evaluation; external supervisor separately limits complete checkpoint/output closeout."}),
    )?;
    for (p, h) in [
        (&cfg.prepared_manifest, &cfg.prepared_manifest_sha256),
        (&cfg.tokenizer_path, &cfg.tokenizer_sha256),
        (&cfg.requests_path, &cfg.requests_sha256),
    ] {
        if sha256_file(p)? != *h {
            return Err(invalid(format!(
                "campaign file identity differs: {}",
                p.display()
            )));
        }
    }
    let requests: Vec<Request> = serde_json::from_slice(&fs::read(&cfg.requests_path)?)?;
    let unique_ids: std::collections::BTreeSet<_> = requests.iter().map(|r| &r.id).collect();
    if requests.is_empty()
        || requests.len() > 128
        || unique_ids.len() != requests.len()
        || requests.iter().any(|r| {
            !r.id.starts_with("dev-")
                || r.user_turns.is_empty()
                || r.user_turns.len() > 8
                || r.user_turns.iter().any(|s| s.trim().is_empty())
        })
    {
        return Err(invalid(
            "only explicitly bound open development requests are supported",
        ));
    }
    let parent = LegacyDialogueArtifact::load(
        &cfg.parent_fit_root,
        &cfg.prepared_manifest,
        &cfg.tokenizer_path,
        &Device::Cpu,
    )?;
    if parent.provenance().parameter_sha256 != cfg.parent_parameter_sha256 {
        return Err(invalid("parent parameter identity differs"));
    }
    let parts = parent.into_training_parts();
    let provenance = serde_json::to_value(&parts.provenance)?;
    let protocol: DialogueProtocol = parts.provenance.protocol.clone();
    let encoder = protocol
        .bind(&parts.tokenizer)
        .map_err(|e| invalid(e.to_string()))?;
    // Prove the complete reply packet fits even if every previous reply reaches
    // its cap and needs an explicit caller EOS, before spending any updates.
    for request in &requests {
        let mut longest_history = 1usize;
        for (turn, user) in request.user_turns.iter().enumerate() {
            let suffix = encoder.encode_user_prefix(user, turn != 0);
            if suffix.emitted_turns != 1 || suffix.special_token_occurrences != 0 {
                return Err(invalid("development request protocol"));
            }
            longest_history += suffix.tokens.len() + cfg.max_new_tokens;
            if longest_history > CONTEXT {
                return Err(invalid(
                    "complete generated history exceeds context; no truncation",
                ));
            }
            if turn + 1 < request.user_turns.len() {
                longest_history += 1;
            }
        }
    }
    let marker_prefix = encoder.encode_assistant_prefix(&[]);
    if marker_prefix.tokens.first() != Some(&protocol.bos_id) {
        return Err(invalid("empty assistant prefix missing BOS"));
    }
    let contract = EpisodeContract {
        context: CONTEXT,
        vocab_size: 4096,
        bos_id: protocol.bos_id,
        eos_id: protocol.eos_id,
        unk_id: protocol.unk_id,
        padding_id: protocol.unk_id,
        assistant_marker_ids: marker_prefix.tokens[1..].to_vec(),
    };
    let prepared: Value = serde_json::from_slice(&fs::read(&cfg.prepared_manifest)?)?;
    let binding = |split: &str| {
        parts
            .provenance
            .datasets
            .iter()
            .find(|x| x.split == split)
            .ok_or_else(|| invalid("missing bound dataset"))
    };
    let train = Corpus::load(binding("train")?, &prepared, &cfg.prepared_manifest)?;
    let dev = Corpus::load(binding("heldout")?, &prepared, &cfg.prepared_manifest)?;
    let train_index = train.index(contract.clone())?;
    let dev_index = dev.index(contract)?;
    if train_index.population().response_runs != cfg.expected_train_response_runs
        || train_index.episodes().len() != cfg.expected_eligible_train_responses
    {
        return Err(invalid(
            "bound training response population differs from campaign",
        ));
    }
    let inventory = json!({"contract":train_index.contract(),"sampler":SAMPLER_ID,
        "episodes":train_index.episodes(),"sources":train_index.sources(),"population":train_index.population(),
        "parent_token_mask_bindings":parts.provenance.datasets,"prepared_manifest_sha256":cfg.prepared_manifest_sha256,
        "protocol_identity":parts.provenance.protocol_identity});
    let index_sha = digest(&inventory)?;
    save(&out.join("train-episodes.json"), &inventory)?;
    let mut dev_ids = Vec::with_capacity(cfg.development_episodes);
    for step in 0..cfg.development_episodes.div_ceil(64) {
        let n = (cfg.development_episodes - dev_ids.len()).min(64);
        dev_ids.extend(dev_index.sample_ids(cfg.development_seed, step as u64, n)?);
    }
    save(
        &out.join("development-episodes.json"),
        &json!({"population":dev_index.population(),"sources":dev_index.sources(),
        "response_ids":dev_ids,"selected":dev_ids.iter().map(|&i|&dev_index.episodes()[i]).collect::<Vec<_>>(),
        "scope":"Previously exposed heldout-named corpus, used as open development; fresh request panel not opened."}),
    )?;
    let mut model = parts.model;
    let mut optimizer = NamedAdamW::new(model.variables(), cfg.optimizer.clone())?;
    let initial_parameters = parameter_fingerprint(&model)?;
    let mut step = 0usize;
    let mut supervised = 0u64;
    let mut schedule = hex::encode(Sha256::digest(SCHEDULE_CHAIN.as_bytes()));
    if let Some(path) = &cfg.resume_from {
        let resumed = JointModel::load_offline_dialogue_checkpoint(path, &Device::Cpu)?;
        let metadata: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
        if metadata["schema"] != CHECKPOINT
            || metadata["learning_contract"] != cfg.learning_contract()?
            || metadata["train_index_sha256"] != index_sha
            || metadata["parent"] != provenance
            || metadata["sampler"] != SAMPLER_ID
            || metadata["schedule_chain"] != SCHEDULE_CHAIN
            || metadata["source_commit"] != source
        {
            return Err(invalid("dialogue resume lineage/learning contract differs"));
        }
        step = count(&metadata, "next_data_step")?;
        supervised = metadata["supervised_target_visits"]
            .as_u64()
            .ok_or_else(|| invalid("resume response count"))?;
        optimizer = NamedAdamW::load(path, resumed.variables(), &cfg.optimizer)?;
        if step >= cfg.total_steps
            || optimizer.step_count() != step as u64
            || metadata["optimizer_step"] != step as u64
            || metadata["parameter_fingerprint"] != parameter_fingerprint(&resumed)?
            || metadata["optimizer_fingerprint"] != optimizer.continuity_fingerprint()?
        {
            return Err(invalid("resume clocks/parameter/optimizer state differs"));
        }
        // Each update extends the same chain, independent of checkpoint boundaries.
        schedule = metadata["executed_schedule_sha256"]
            .as_str()
            .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(|| invalid("resume schedule hash"))?
            .to_owned();
        model = resumed;
    }
    let start_step = step;
    let start_supervised = supervised;
    let initial_optimizer = optimizer.continuity_fingerprint()?;
    let initial_dev = evaluate(&model, &dev_index, &dev_ids, cfg.development_batch)?;
    let mut curve = fs::File::create_new(out.join("learning-curve.jsonl"))?;
    let mut last_ids = None;
    while step < cfg.total_steps {
        if clock.elapsed().as_secs_f64() >= cfg.max_process_seconds
            || cfg.stop_file.as_ref().is_some_and(|p| p.exists())
        {
            break;
        }
        let started = Instant::now();
        let ids = train_index.sample_ids(cfg.data_seed, step as u64, cfg.batch)?;
        let batch = train_index.materialize(&ids, cfg.policy)?;
        let grads = response_batch_gradients(
            &model,
            &batch.inputs,
            &batch.targets,
            &batch.weights,
            batch.batch,
            batch.time,
            cfg.cpu_gradient_shards,
        )?;
        let opt = optimizer.step(model.variables(), &grads.gradients)?;
        supervised = supervised
            .checked_add(grads.supervised_targets as u64)
            .ok_or_else(|| invalid("response exposure overflow"))?;
        let target_sha = digest(&batch.selected_target_ids)?;
        schedule = advance_schedule(&schedule, step, &ids, &target_sha)?;
        step += 1;
        let row = json!({"step":step,"response_ids":ids,"selected_target_ids_sha256":target_sha,
            "response_mean_nll":grads.mean_nll,"supervised_targets":grads.supervised_targets,
            "cumulative_supervised_targets":supervised,"counts":batch.counts,"source_visits":batch.source_visits,
            "optimizer":opt,"complete_update_seconds":started.elapsed().as_secs_f64()});
        serde_json::to_writer(&mut curve, &row)?;
        curve.write_all(b"\n")?;
        curve.flush()?;
        last_ids = Some(ids);
        if cfg.development_every_steps != 0
            && step % cfg.development_every_steps == 0
            && step < cfg.total_steps
        {
            save(
                &out.join(format!("development-{step:08}.json")),
                &evaluate(&model, &dev_index, &dev_ids, cfg.development_batch)?,
            )?;
        }
        if cfg.checkpoint_every_steps != 0
            && step % cfg.checkpoint_every_steps == 0
            && step < cfg.total_steps
        {
            checkpoint(
                out,
                &format!("checkpoint-{step:08}"),
                &model,
                &optimizer,
                cfg,
                source,
                &index_sha,
                &provenance,
                step,
                supervised,
                &schedule,
            )?;
        }
    }
    curve.sync_all()?;
    let final_fingerprint = parameter_fingerprint(&model)?;
    let optimizer_fingerprint = optimizer.continuity_fingerprint()?;
    let saved = checkpoint(
        out,
        "checkpoint-final",
        &model,
        &optimizer,
        cfg,
        source,
        &index_sha,
        &provenance,
        step,
        supervised,
        &schedule,
    )?;
    let last_batch = last_ids
        .as_ref()
        .map(|ids| train_index.materialize(ids, cfg.policy))
        .transpose()?;
    let pre_reload = last_batch
        .as_ref()
        .map(|b| -> Result<f32> {
            Ok(model
                .forward(&b.inputs, b.batch, b.time, ReadMode::Enabled, false)?
                .weighted_loss(&b.targets, &b.weights)?
                .to_scalar::<f32>()?)
        })
        .transpose()?;
    drop(model);
    drop(optimizer);
    let reloaded = JointModel::load_offline_dialogue_checkpoint(&saved, &Device::Cpu)?;
    let restored_optimizer = NamedAdamW::load(&saved, reloaded.variables(), &cfg.optimizer)?;
    if parameter_fingerprint(&reloaded)? != final_fingerprint
        || restored_optimizer.continuity_fingerprint()? != optimizer_fingerprint
    {
        return Err(invalid("saved dialogue parameter/optimizer reload differs"));
    }
    let post_reload = last_batch
        .as_ref()
        .map(|b| -> Result<f32> {
            Ok(reloaded
                .forward(&b.inputs, b.batch, b.time, ReadMode::Enabled, false)?
                .weighted_loss(&b.targets, &b.weights)?
                .to_scalar::<f32>()?)
        })
        .transpose()?;
    let reload_delta = pre_reload.zip(post_reload).map(|(a, b)| (a - b).abs());
    if reload_delta.is_some_and(|d| !d.is_finite() || d > 1e-5) {
        return Err(invalid("saved response prediction reload differs"));
    }
    let final_dev = evaluate(&reloaded, &dev_index, &dev_ids, cfg.development_batch)?;
    let mut replies = Vec::new();
    for request in &requests {
        let mut history = vec![protocol.bos_id];
        let mut turns = Vec::new();
        for (turn, user) in request.user_turns.iter().enumerate() {
            let suffix = encoder.encode_user_prefix(user, turn != 0);
            if suffix.emitted_turns != 1 || suffix.special_token_occurrences != 0 {
                return Err(invalid("development request protocol"));
            }
            history.extend(suffix.tokens);
            if history.len() + cfg.max_new_tokens > CONTEXT {
                return Err(invalid(
                    "complete generated history exceeds context; no truncation",
                ));
            }
            let generation = joint_evaluation::generate_tokens(
                &reloaded,
                &parts.generation_tokenizer,
                &history,
                ReadMode::Enabled,
                None,
                cfg.max_new_tokens,
            )?;
            let model_eos = matches!(generation.stop, JointGenerationStop::Eos);
            history.extend_from_slice(&generation.generated_token_ids);
            let caller_eos = !model_eos && turn + 1 < request.user_turns.len();
            if caller_eos {
                history.push(protocol.eos_id);
            }
            turns.push(json!({"turn":turn+1,"user":user,"generation":generation,"model_eos":model_eos,
                "caller_eos_inserted_before_next_request":caller_eos,"retained_history_ids":history}));
        }
        replies.push(json!({"id":request.id,"category":request.category,"turns":turns}));
    }
    save(
        &out.join("responses.json"),
        &json!({"rows":replies,"protocol":protocol,"requests_sha256":cfg.requests_sha256,
        "scope":"Actual greedy replies from reloaded continuous checkpoint; exact generated history, explicit caller closure, no serving/quality qualification."}),
    )?;
    let result = json!({"schema":"uor-r4.dialogue-prefix-fit/1","status":if step==cfg.total_steps{"TARGET_COMPLETE"}else{"RESOURCE_STOP"},
        "source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"campaign":cfg,
        "parent":provenance,"sampler":SAMPLER_ID,"schedule_chain":SCHEDULE_CHAIN,"executed_schedule_sha256":schedule,
        "train_index_sha256":index_sha,"selected_train_responses":train_index.episodes().len(),
        "starting_step":start_step,"completed_step":step,"new_optimizer_updates":step-start_step,
        "new_supervised_target_visits":supervised-start_supervised,"cumulative_supervised_target_visits":supervised,
        "new_tensor_positions":(step-start_step) as u64*cfg.batch as u64*CONTEXT as u64,
        "initial_parent_parameter_fingerprint":initial_parameters,"final_parameter_fingerprint":final_fingerprint,
        "starting_optimizer":initial_optimizer,"restored_optimizer":optimizer_fingerprint,"reload_response_nll_delta":reload_delta,
        "initial_development":initial_dev,"final_development":final_dev,"checkpoint":saved,
        "elapsed_seconds":clock.elapsed().as_secs_f64(),"scope":"Offline response-conditioned learning. Historical parent sampled_targets is nominal positions; current exposure counts actual response/EOS supervision. No integer export or automatic language qualification."});
    save(&out.join("result.json"), &result)?;
    Ok(result)
}
