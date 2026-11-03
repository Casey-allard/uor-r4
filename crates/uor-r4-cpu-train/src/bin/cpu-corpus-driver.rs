use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use uor_r4_cpu_train::{
    panel_admission_gate, validate_corpus_manifest, CpuCorpusIdentity, CpuMaskedCorpus,
    CpuPanelEvaluator, CpuRankMarginTrainer, PanelGateDecision, RankMarginConfig,
};
use uor_r4_integer::stack::IntegerStackModel;

const CHECKPOINT_BINDING_SCHEMA: &str = "uor-r4.cpu-corpus-checkpoint-binding/1";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointBinding {
    schema: String,
    source_artifact_sha256: String,
    tokens_sha256: String,
    mask_sha256: String,
    manifest_sha256: String,
    tokenizer_sha256: String,
    trainer_seed: u64,
    #[serde(default = "default_margin_q12")]
    margin_q12: i32,
    #[serde(default = "default_step_q12")]
    step_q12: i16,
    #[serde(default = "default_parameter_limit_q12")]
    parameter_limit_q12: i16,
    #[serde(default = "default_coordinates_per_step")]
    coordinates_per_step: usize,
    evaluation_stride: usize,
    max_evaluation: usize,
    completed_steps: u64,
    checkpoint_sha256: String,
}

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    seed: u64,
    rank_margin: RankMarginConfig,
    steps: usize,
    evaluation_stride: usize,
    max_evaluation: usize,
    checkpoint_every: usize,
    checkpoint_dir: PathBuf,
    resume_checkpoint: Option<PathBuf>,
    require_admission: bool,
}

fn default_margin_q12() -> i32 {
    RankMarginConfig::default().margin_q12
}

fn default_step_q12() -> i16 {
    RankMarginConfig::default().step_q12
}

fn default_parameter_limit_q12() -> i16 {
    RankMarginConfig::default().parameter_limit_q12
}

fn default_coordinates_per_step() -> usize {
    RankMarginConfig::default().coordinates_per_step
}

fn main() {
    match run() {
        Ok((report, required_gate_failed)) => {
            println!("{report}");
            if required_gate_failed {
                eprintln!("cpu-corpus-driver: required admission gate failed");
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("cpu-corpus-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(String, bool), String> {
    let config = parse_args()?;
    let report = execute(&config)?;
    Ok((
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?,
        required_gate_failed(&config, &report),
    ))
}

fn required_gate_failed(config: &Config, report: &serde_json::Value) -> bool {
    config.require_admission && report["panel_gate"]["decision"].as_str() != Some("accept")
}

fn accepted_coordinate_block(
    coordinates: &[usize],
    width: usize,
    limit: usize,
) -> Result<usize, String> {
    if coordinates.is_empty() || coordinates.len() > limit {
        return Err("an accepted table block has no legal coordinate count".to_owned());
    }
    let mut distinct = coordinates.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.len() != coordinates.len()
        || coordinates
            .iter()
            .any(|&coordinate| !(8..width.saturating_sub(8)).contains(&coordinate))
    {
        return Err("an accepted table block has invalid coordinates".to_owned());
    }
    Ok(coordinates.len())
}

fn execute(config: &Config) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact).map_err(|error| format!("artifact: {error}"))?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    let token_bytes = std::fs::read(&config.tokens).map_err(|error| format!("tokens: {error}"))?;
    let mask = std::fs::read(&config.mask).map_err(|error| format!("mask: {error}"))?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| format!("manifest: {error}"))?;
    let manifest_sha256 = uor_r4_discrete::corpus::sha256(&manifest);
    let identity = validate_corpus_manifest(&manifest, &token_bytes, &mask)
        .map_err(|error| error.to_string())?;
    if identity.split != "train" {
        return Err(format!(
            "corpus manifest split must be train, found {}",
            identity.split
        ));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    let shape = model.shape();
    if identity.vocab != shape.vocab || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the artifact".to_owned());
    }
    if identity.validate_context(shape.context).is_err() {
        return Err(format!(
            "corpus manifest max_tokens {} is below artifact context {}",
            identity.max_tokens, shape.context
        ));
    }
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask,
        shape.vocab,
        shape.context,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let split = corpus
        .split(config.evaluation_stride, config.max_evaluation)
        .map_err(|error| error.to_string())?;

    let mut baseline_panel = CpuPanelEvaluator::from_artifact(
        &artifact,
        split.evaluation.clone(),
        config.rank_margin.margin_q12,
    )
    .map_err(|error| error.to_string())?;
    let baseline = baseline_panel
        .evaluate()
        .map_err(|error| error.to_string())?;
    let (mut trainer, resume) = if let Some(checkpoint_path) = &config.resume_checkpoint {
        let (trainer, resume) = resume_checkpoint(
            checkpoint_path,
            &artifact_sha256,
            &identity,
            &manifest_sha256,
            config,
            split.training.clone(),
        )?;
        (trainer, serde_json::json!(resume))
    } else {
        let trainer = CpuRankMarginTrainer::from_artifact(
            &artifact,
            split.training.clone(),
            config.rank_margin,
            config.seed,
        )
        .map_err(|error| error.to_string())?;
        (trainer, serde_json::Value::Null)
    };
    let resumed_from_step = trainer.step_count();
    let target_steps =
        u64::try_from(config.steps).map_err(|_| "target step count exceeds u64".to_owned())?;
    if resumed_from_step > target_steps {
        return Err(format!(
            "resume checkpoint is already at step {resumed_from_step}, beyond target {target_steps}"
        ));
    }
    let executed_steps =
        usize::try_from(target_steps - resumed_from_step).map_err(|_| "step count differs")?;
    if executed_steps > 0 && config.checkpoint_every == 0 {
        return Err("checkpoint every must be positive for remaining steps".to_owned());
    }

    std::fs::create_dir_all(&config.checkpoint_dir)
        .map_err(|error| format!("checkpoint directory: {error}"))?;
    let mut step_elapsed = Duration::ZERO;
    let mut checkpoint_elapsed = Duration::ZERO;
    let mut accepted = 0usize;
    let mut refused = 0usize;
    let mut context_tokens = 0usize;
    let mut target_visits = 0usize;
    let mut accepted_coordinate_moves = 0usize;
    let mut accepted_coordinate_histogram = vec![0usize; 65];
    let mut checkpoints = Vec::new();
    for completed in resumed_from_step + 1..=target_steps {
        let step_started = Instant::now();
        let step = trainer.train_step().map_err(|error| error.to_string())?;
        step_elapsed += step_started.elapsed();
        target_visits = target_visits
            .checked_add(1)
            .ok_or_else(|| "target visit count overflow".to_owned())?;
        if step.updated {
            accepted += 1;
            let coordinate_count = accepted_coordinate_block(
                &step.coordinates,
                shape.width,
                config.rank_margin.coordinates_per_step,
            )?;
            accepted_coordinate_moves = accepted_coordinate_moves
                .checked_add(coordinate_count)
                .ok_or_else(|| "accepted coordinate count overflow".to_owned())?;
            accepted_coordinate_histogram[coordinate_count] += 1;
        } else {
            refused += 1;
        }
        context_tokens = context_tokens
            .checked_add(split.training[step.sample].input.len())
            .ok_or_else(|| "training context-token count overflow".to_owned())?;
        if completed % config.checkpoint_every as u64 == 0 {
            let checkpoint_started = Instant::now();
            let bytes = trainer.checkpoint().map_err(|error| error.to_string())?;
            let path = config
                .checkpoint_dir
                .join(format!("step-{completed:012}.ckpt"));
            let binding_path = write_checkpoint(
                &path,
                &bytes,
                &artifact_sha256,
                &identity,
                &manifest_sha256,
                config,
                completed,
            )?;
            checkpoints.push(serde_json::json!({
                "step": completed,
                "path": path,
                "sha256": uor_r4_discrete::corpus::sha256(&bytes),
                "binding_path": binding_path,
            }));
            checkpoint_elapsed += checkpoint_started.elapsed();
        }
    }

    let mut final_panel = CpuPanelEvaluator::from_artifact(
        trainer.artifact(),
        split.evaluation.clone(),
        config.rank_margin.margin_q12,
    )
    .map_err(|error| error.to_string())?;
    let final_reading = final_panel.evaluate().map_err(|error| error.to_string())?;
    let gate =
        panel_admission_gate(&baseline, &final_reading).map_err(|error| error.to_string())?;
    let mut gate = match gate {
        PanelGateDecision::Accept(delta) => serde_json::json!({
            "decision": "accept",
            "target_admitted": delta.target_admitted,
            "top1_correct": delta.top1_correct,
            "top8_correct": delta.top8_correct,
            "margin_loss_sum": delta.margin_loss_sum,
        }),
        PanelGateDecision::RejectAdmission(delta) => serde_json::json!({
            "decision": "reject_admission",
            "target_admitted": delta.target_admitted,
            "top1_correct": delta.top1_correct,
            "top8_correct": delta.top8_correct,
            "margin_loss_sum": delta.margin_loss_sum,
        }),
        PanelGateDecision::RejectExportGap(delta) => serde_json::json!({
            "decision": "reject_export_gap",
            "target_admitted": delta.target_admitted,
            "top1_correct": delta.top1_correct,
            "top8_correct": delta.top8_correct,
            "margin_loss_sum": delta.margin_loss_sum,
        }),
    };
    gate["panel_changed"] = serde_json::json!(baseline != final_reading);
    gate["interpretation"] = serde_json::json!(
        "gate admission is an acceptance constraint, not a capability gain; use the raw readings"
    );

    let final_checkpoint = trainer.checkpoint().map_err(|error| error.to_string())?;
    let final_checkpoint_path = config.checkpoint_dir.join("final.ckpt");
    let final_binding_path = write_checkpoint(
        &final_checkpoint_path,
        &final_checkpoint,
        &artifact_sha256,
        &identity,
        &manifest_sha256,
        config,
        target_steps,
    )?;
    let resumed = CpuRankMarginTrainer::resume(&final_checkpoint, split.training.clone())
        .map_err(|error| format!("final checkpoint reload: {error}"))?;
    let final_artifact = trainer.artifact().to_vec();
    if resumed.artifact() != final_artifact {
        return Err("final checkpoint artifact differs from the trainer artifact".to_owned());
    }
    if target_steps == 0 && final_artifact != artifact {
        return Err("zero-step run changed the source artifact".to_owned());
    }
    let final_artifact_path = config.checkpoint_dir.join("final.uorlut");
    write_verified_artifact(&final_artifact_path, &final_artifact)?;
    let final_artifact_sha256 = uor_r4_discrete::corpus::sha256(&final_artifact);
    if !baseline.export.is_exact() || !final_reading.export.is_exact() {
        return Err("CPU/serving query, token or kept-cell export mismatch".to_owned());
    }
    let step_seconds = step_elapsed.as_secs_f64();
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-masked-corpus-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "artifact": {
            "path": config.artifact,
            "sha256": artifact_sha256,
            "vocab": shape.vocab,
            "context": shape.context,
            "raw_gates": shape.raw_gates,
            "raw_transition": shape.raw_transition,
        },
        "corpus": {
            "manifest_schema": identity.schema,
            "manifest_split": identity.split,
            "tokens_sha256": identity.tokens_sha256,
            "mask_sha256": identity.mask_sha256,
            "tokenizer_sha256": identity.tokenizer_sha256,
            "tokens": identity.tokens,
            "token_bytes": identity.token_bytes,
            "mask_bytes": identity.mask_bytes,
            "response_targets": identity.response_targets,
            "max_tokens": identity.max_tokens,
            "documents": corpus.document_count(),
            "training_documents": split.training_documents,
            "evaluation_documents": split.evaluation_documents,
            "training_targets": split.training_targets,
            "evaluation_targets": split.evaluation_targets,
            "evaluation_stride": split.evaluation_stride,
        },
        "run": {
            "seed": config.seed,
            "rank_margin": config.rank_margin,
            "require_admission": config.require_admission,
            "final_epoch": trainer.epoch(),
            "steps": target_steps,
            "resumed_from_step": resumed_from_step,
            "executed_steps": executed_steps,
            "accepted_updates": accepted,
            "refused_steps": refused,
            "target_visits": target_visits,
            "accepted_coordinate_moves": accepted_coordinate_moves,
            "accepted_coordinate_histogram": accepted_coordinate_histogram,
            "context_tokens": context_tokens,
            "step_elapsed_ms": step_elapsed.as_secs_f64() * 1_000.0,
            "mean_step_ms": if executed_steps > 0 {
                step_elapsed.as_secs_f64() * 1_000.0 / executed_steps as f64
            } else {
                0.0
            },
            "checkpoint_elapsed_ms": checkpoint_elapsed.as_secs_f64() * 1_000.0,
            "context_tokens_per_second": if step_seconds > 0.0 {
                context_tokens as f64 / step_seconds
            } else {
                0.0
            },
            "target_visits_per_second": if step_seconds > 0.0 {
                target_visits as f64 / step_seconds
            } else {
                0.0
            },
            "steps_per_second": if step_seconds > 0.0 {
                executed_steps as f64 / step_seconds
            } else {
                0.0
            },
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        },
        "panel_baseline": panel_json(&baseline),
        "panel_final": panel_json(&final_reading),
        "panel_gate": gate,
        "resume": resume,
        "checkpoints": checkpoints,
        "final_checkpoint": {
            "path": final_checkpoint_path,
            "sha256": uor_r4_discrete::corpus::sha256(&final_checkpoint),
            "binding_path": final_binding_path,
        },
        "final_artifact": {
            "path": final_artifact_path,
            "sha256": final_artifact_sha256,
            "reloaded": true,
            "matches_source": final_artifact == artifact,
        },
        "scope": "CPU development-research component run; not GPU qualification, language quality or complete-reply evidence",
    });
    Ok(report)
}

fn binding_path(checkpoint: &Path) -> PathBuf {
    checkpoint.with_extension("binding.json")
}

fn checkpoint_binding(
    source_artifact_sha256: &str,
    identity: &CpuCorpusIdentity,
    manifest_sha256: &str,
    config: &Config,
    completed_steps: u64,
    checkpoint_sha256: String,
) -> CheckpointBinding {
    CheckpointBinding {
        schema: CHECKPOINT_BINDING_SCHEMA.to_owned(),
        source_artifact_sha256: source_artifact_sha256.to_owned(),
        tokens_sha256: identity.tokens_sha256.clone(),
        mask_sha256: identity.mask_sha256.clone(),
        manifest_sha256: manifest_sha256.to_owned(),
        tokenizer_sha256: identity.tokenizer_sha256.clone(),
        trainer_seed: config.seed,
        margin_q12: config.rank_margin.margin_q12,
        step_q12: config.rank_margin.step_q12,
        parameter_limit_q12: config.rank_margin.parameter_limit_q12,
        coordinates_per_step: config.rank_margin.coordinates_per_step,
        evaluation_stride: config.evaluation_stride,
        max_evaluation: config.max_evaluation,
        completed_steps,
        checkpoint_sha256,
    }
}

fn write_checkpoint(
    path: &Path,
    bytes: &[u8],
    source_artifact_sha256: &str,
    identity: &CpuCorpusIdentity,
    manifest_sha256: &str,
    config: &Config,
    completed_steps: u64,
) -> Result<PathBuf, String> {
    std::fs::write(path, bytes)
        .map_err(|error| format!("checkpoint {}: {error}", path.display()))?;
    let binding = checkpoint_binding(
        source_artifact_sha256,
        identity,
        manifest_sha256,
        config,
        completed_steps,
        uor_r4_discrete::corpus::sha256(bytes),
    );
    let binding_path = binding_path(path);
    let binding_bytes = serde_json::to_vec_pretty(&binding)
        .map_err(|error| format!("checkpoint binding {}: {error}", binding_path.display()))?;
    std::fs::write(&binding_path, binding_bytes)
        .map_err(|error| format!("checkpoint binding {}: {error}", binding_path.display()))?;
    Ok(binding_path)
}

fn resume_checkpoint(
    checkpoint_path: &Path,
    source_artifact_sha256: &str,
    identity: &CpuCorpusIdentity,
    manifest_sha256: &str,
    config: &Config,
    samples: Vec<uor_r4_cpu_train::RankMarginSample>,
) -> Result<(CpuRankMarginTrainer, serde_json::Value), String> {
    let checkpoint = std::fs::read(checkpoint_path)
        .map_err(|error| format!("resume checkpoint {}: {error}", checkpoint_path.display()))?;
    let checkpoint_sha256 = uor_r4_discrete::corpus::sha256(&checkpoint);
    let binding_path = binding_path(checkpoint_path);
    let binding_bytes = std::fs::read(&binding_path)
        .map_err(|error| format!("resume binding {}: {error}", binding_path.display()))?;
    let binding: CheckpointBinding = serde_json::from_slice(&binding_bytes)
        .map_err(|error| format!("resume binding {}: {error}", binding_path.display()))?;
    let expected = checkpoint_binding(
        source_artifact_sha256,
        identity,
        manifest_sha256,
        config,
        binding.completed_steps,
        checkpoint_sha256.clone(),
    );
    if binding.schema != expected.schema {
        return Err("resume binding schema differs".to_owned());
    }
    if binding.source_artifact_sha256 != expected.source_artifact_sha256 {
        return Err("resume binding source artifact differs".to_owned());
    }
    if binding.tokens_sha256 != expected.tokens_sha256
        || binding.mask_sha256 != expected.mask_sha256
        || binding.manifest_sha256 != expected.manifest_sha256
        || binding.tokenizer_sha256 != expected.tokenizer_sha256
        || binding.trainer_seed != expected.trainer_seed
        || binding.margin_q12 != expected.margin_q12
        || binding.step_q12 != expected.step_q12
        || binding.parameter_limit_q12 != expected.parameter_limit_q12
        || binding.coordinates_per_step != expected.coordinates_per_step
        || binding.evaluation_stride != expected.evaluation_stride
        || binding.max_evaluation != expected.max_evaluation
    {
        return Err("resume binding corpus, seed or split differs".to_owned());
    }
    if binding.checkpoint_sha256 != checkpoint_sha256 {
        return Err("resume binding checkpoint hash differs".to_owned());
    }
    let trainer = CpuRankMarginTrainer::resume(&checkpoint, samples)
        .map_err(|error| format!("resume checkpoint: {error}"))?;
    if trainer.step_count() != binding.completed_steps {
        return Err("resume binding completed step differs from checkpoint".to_owned());
    }
    Ok((
        trainer,
        serde_json::json!({
            "checkpoint_path": checkpoint_path,
            "checkpoint_sha256": checkpoint_sha256,
            "binding_path": binding_path,
            "binding_sha256": uor_r4_discrete::corpus::sha256(&binding_bytes),
            "completed_steps": binding.completed_steps,
            "verified": true,
        }),
    ))
}

fn write_verified_artifact(path: &Path, artifact: &[u8]) -> Result<(), String> {
    IntegerStackModel::parse(artifact)
        .map_err(|error| format!("final artifact does not parse: {error}"))?;
    let partial = path.with_extension("partial");
    let mut file = std::fs::File::create(&partial)
        .map_err(|error| format!("final artifact {}: {error}", partial.display()))?;
    file.write_all(artifact)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("final artifact {}: {error}", partial.display()))?;
    drop(file);
    std::fs::rename(&partial, path)
        .map_err(|error| format!("final artifact {}: {error}", path.display()))?;
    let reloaded = std::fs::read(path)
        .map_err(|error| format!("final artifact reload {}: {error}", path.display()))?;
    if reloaded != artifact {
        std::fs::remove_file(path)
            .map_err(|error| format!("failed final artifact {}: {error}", path.display()))?;
        return Err("final artifact reload differs from the trainer artifact".to_owned());
    }
    Ok(())
}

fn panel_json(reading: &uor_r4_cpu_train::PanelReading) -> serde_json::Value {
    serde_json::json!({
        "samples": reading.samples,
        "target_admitted": reading.target_admitted,
        "top1_correct": reading.top1_correct,
        "top8_correct": reading.top8_correct,
        "margin_loss_sum": reading.margin_loss_sum,
        "target_rank_sum": reading.target_rank_sum,
        "export": {
            "query_coordinates_checked": reading.export.query_coordinates_checked,
            "query_coordinate_mismatches": reading.export.query_coordinate_mismatches,
            "token_mismatches": reading.export.token_mismatches,
            "kept_cells_checked": reading.export.kept_cells_checked,
            "kept_cell_mismatches": reading.export.kept_cell_mismatches,
        },
    })
}

fn peak_rss_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmHWM:")
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse().ok())
    })
}

fn parse_args() -> Result<Config, String> {
    let values = std::env::args().skip(1).collect::<Vec<_>>();
    parse_args_from(&values)
}

fn parse_args_from(values: &[String]) -> Result<Config, String> {
    let mut positional = Vec::new();
    let mut rank_margin = RankMarginConfig::default();
    let mut require_admission = false;
    let mut at = 0;
    while at < values.len() {
        match values[at].as_str() {
            "--require-admission" => {
                require_admission = true;
                at += 1;
            }
            "--margin-q12" | "--step-q12" | "--parameter-limit-q12" | "--coordinates-per-step" => {
                let flag = values[at].as_str();
                let value = values
                    .get(at + 1)
                    .ok_or_else(|| format!("{flag} needs a positive integer"))?;
                match flag {
                    "--margin-q12" => {
                        rank_margin.margin_q12 = value
                            .parse::<i32>()
                            .ok()
                            .filter(|value| *value > 0)
                            .ok_or_else(|| "--margin-q12 needs a positive integer".to_owned())?;
                    }
                    "--step-q12" => {
                        rank_margin.step_q12 = value
                            .parse::<i16>()
                            .ok()
                            .filter(|value| *value > 0)
                            .ok_or_else(|| "--step-q12 needs a positive integer".to_owned())?;
                    }
                    "--parameter-limit-q12" => {
                        rank_margin.parameter_limit_q12 = value
                            .parse::<i16>()
                            .ok()
                            .filter(|value| *value > 0)
                            .ok_or_else(|| {
                                "--parameter-limit-q12 needs a positive integer".to_owned()
                            })?;
                    }
                    "--coordinates-per-step" => {
                        rank_margin.coordinates_per_step = value
                            .parse::<usize>()
                            .ok()
                            .filter(|value| (1..=64).contains(value))
                            .ok_or_else(|| {
                                "--coordinates-per-step needs an integer in 1..=64".to_owned()
                            })?;
                    }
                    _ => unreachable!(),
                }
                at += 2;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown option {flag}"));
            }
            value => {
                positional.push(value);
                at += 1;
            }
        }
    }
    if !matches!(positional.len(), 10 | 11) {
        return Err(
            "usage: cpu-corpus-driver <artifact> <tokens.u16> <mask.u8> <manifest.json> \
             <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> \
             <checkpoint-dir> [resume-checkpoint] \
             [--margin-q12 N] [--step-q12 N] [--parameter-limit-q12 N] \
             [--coordinates-per-step N] [--require-admission]"
                .to_owned(),
        );
    }
    let parse_usize = |index: usize, name: &str| -> Result<usize, String> {
        positional[index]
            .parse()
            .map_err(|_| format!("{name} differs"))
    };
    let steps = parse_usize(5, "steps")?;
    let checkpoint_every = parse_usize(8, "checkpoint every")?;
    if steps > 0 && checkpoint_every == 0 {
        return Err("checkpoint every must be positive".to_owned());
    }
    Ok(Config {
        artifact: Path::new(positional[0]).to_path_buf(),
        tokens: Path::new(positional[1]).to_path_buf(),
        mask: Path::new(positional[2]).to_path_buf(),
        manifest: Path::new(positional[3]).to_path_buf(),
        seed: positional[4]
            .parse()
            .map_err(|_| "seed differs".to_owned())?,
        rank_margin,
        steps,
        evaluation_stride: parse_usize(6, "evaluation stride")?,
        max_evaluation: parse_usize(7, "max evaluation")?,
        checkpoint_every,
        checkpoint_dir: Path::new(positional[9]).to_path_buf(),
        resume_checkpoint: positional.get(10).map(PathBuf::from),
        require_admission,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    struct CorpusFixture {
        root: PathBuf,
        artifact: PathBuf,
        tokens: PathBuf,
        mask: PathBuf,
        manifest: PathBuf,
    }

    fn artifact() -> Vec<u8> {
        let snapshot = synthetic_fbs_trunk_with(
            307,
            FbsSynth {
                vocab: 4096,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn uort(tokens: &[u16], vocab: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocab.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
        bytes
    }

    fn corpus_fixture(label: &str) -> CorpusFixture {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cpu-corpus-{label}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let artifact_path = root.join("source.uorlut");
        std::fs::write(&artifact_path, artifact()).unwrap();
        let mut tokens = Vec::new();
        let mut mask = Vec::new();
        for document in 0u16..4 {
            tokens.push(0);
            mask.push(0);
            for offset in 1u16..12 {
                tokens.push(10 + document * 20 + offset);
                mask.push(1);
            }
        }
        let token_bytes = uort(&tokens, 4096);
        let tokens_path = root.join("tokens.u16");
        let mask_path = root.join("response_mask.u8");
        let manifest_path = root.join("manifest.json");
        std::fs::write(&tokens_path, &token_bytes).unwrap();
        std::fs::write(&mask_path, &mask).unwrap();
        let manifest = serde_json::json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": "train",
            "tokens": tokens.len(),
            "tokens_bytes": token_bytes.len(),
            "tokens_sha256": uor_r4_discrete::corpus::sha256(&token_bytes),
            "mask_bytes": mask.len(),
            "mask_sha256": uor_r4_discrete::corpus::sha256(&mask),
            "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
            "max_tokens": 8,
            "tokenizer": {
                "vocab_size": 4096,
                "bos_id": 0,
                "sha256": "00".repeat(32)
            }
        });
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        CorpusFixture {
            root,
            artifact: artifact_path,
            tokens: tokens_path,
            mask: mask_path,
            manifest: manifest_path,
        }
    }

    fn config(
        fixture: &CorpusFixture,
        steps: usize,
        checkpoint_name: &str,
        resume_checkpoint: Option<PathBuf>,
    ) -> Config {
        Config {
            artifact: fixture.artifact.clone(),
            tokens: fixture.tokens.clone(),
            mask: fixture.mask.clone(),
            manifest: fixture.manifest.clone(),
            seed: 41,
            rank_margin: RankMarginConfig::default(),
            steps,
            evaluation_stride: 2,
            max_evaluation: 16,
            checkpoint_every: 2,
            checkpoint_dir: fixture.root.join(checkpoint_name),
            resume_checkpoint,
            require_admission: false,
        }
    }

    #[test]
    fn interval_checkpoint_resume_reproduces_the_uninterrupted_run() {
        let fixture = corpus_fixture("resume");
        let reference = execute(&config(&fixture, 6, "reference", None)).unwrap();
        execute(&config(&fixture, 2, "partial", None)).unwrap();
        let interval = fixture.root.join("partial/step-000000000002.ckpt");
        let resumed = execute(&config(&fixture, 6, "resumed", Some(interval.clone()))).unwrap();

        assert_eq!(
            reference["final_artifact"]["sha256"],
            resumed["final_artifact"]["sha256"]
        );
        assert_eq!(
            reference["final_checkpoint"]["sha256"],
            resumed["final_checkpoint"]["sha256"]
        );
        assert_eq!(reference["panel_final"], resumed["panel_final"]);
        assert_eq!(resumed["run"]["resumed_from_step"], 2);
        assert_eq!(resumed["run"]["executed_steps"], 4);
        assert_eq!(resumed["run"]["final_epoch"], 0);
        assert_eq!(resumed["resume"]["verified"], true);

        let binding_path = binding_path(&interval);
        let original_binding = std::fs::read(&binding_path).unwrap();
        let tamper_cases = [
            (
                "source_artifact_sha256",
                serde_json::json!("00".repeat(32)),
                "source artifact",
            ),
            (
                "tokens_sha256",
                serde_json::json!("00".repeat(32)),
                "corpus, seed or split",
            ),
            (
                "mask_sha256",
                serde_json::json!("00".repeat(32)),
                "corpus, seed or split",
            ),
            (
                "manifest_sha256",
                serde_json::json!("00".repeat(32)),
                "corpus, seed or split",
            ),
            (
                "tokenizer_sha256",
                serde_json::json!("11".repeat(32)),
                "corpus, seed or split",
            ),
            (
                "trainer_seed",
                serde_json::json!(99),
                "corpus, seed or split",
            ),
            ("margin_q12", serde_json::json!(2), "corpus, seed or split"),
            ("step_q12", serde_json::json!(2), "corpus, seed or split"),
            (
                "parameter_limit_q12",
                serde_json::json!(4096),
                "corpus, seed or split",
            ),
            (
                "coordinates_per_step",
                serde_json::json!(4),
                "corpus, seed or split",
            ),
            (
                "evaluation_stride",
                serde_json::json!(3),
                "corpus, seed or split",
            ),
            (
                "max_evaluation",
                serde_json::json!(8),
                "corpus, seed or split",
            ),
            (
                "checkpoint_sha256",
                serde_json::json!("00".repeat(32)),
                "checkpoint hash",
            ),
            ("completed_steps", serde_json::json!(1), "completed step"),
            ("schema", serde_json::json!("changed"), "schema"),
        ];
        for (field, value, expected_error) in tamper_cases {
            let mut binding: serde_json::Value = serde_json::from_slice(&original_binding).unwrap();
            binding[field] = value;
            std::fs::write(&binding_path, serde_json::to_vec_pretty(&binding).unwrap()).unwrap();
            let error = match execute(&config(&fixture, 6, "tampered", Some(interval.clone()))) {
                Ok(_) => panic!("tampered {field} resumed"),
                Err(error) => error,
            };
            assert!(
                error.contains(expected_error),
                "tampered {field} was not refused as expected: {error}"
            );
        }
        std::fs::remove_dir_all(fixture.root).unwrap();
    }

    #[test]
    fn a_table_block_run_reports_its_coordinate_invariants_and_fails_required_admission() {
        let fixture = corpus_fixture("table-block");
        let mut base_config = config(&fixture, 8, "block", None);
        base_config.rank_margin.coordinates_per_step = 4;
        let report = execute(&base_config).unwrap();
        let accepted = report["run"]["accepted_updates"].as_u64().unwrap();
        assert_eq!(report["run"]["rank_margin"]["coordinates_per_step"], 4);
        assert_eq!(
            report["run"]["accepted_coordinate_histogram"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_u64().unwrap())
                .sum::<u64>(),
            accepted
        );
        assert!(
            report["run"]["accepted_coordinate_moves"].as_u64().unwrap() >= accepted,
            "{report}"
        );
        let checkpoint = PathBuf::from(
            report["checkpoints"][1]["path"]
                .as_str()
                .expect("step-4 checkpoint path"),
        );
        let mut resumed_config = config(&fixture, 8, "resumed", Some(checkpoint));
        resumed_config.rank_margin.coordinates_per_step = 4;
        let resumed = execute(&resumed_config).unwrap();
        assert_eq!(
            resumed["final_artifact"]["sha256"],
            report["final_artifact"]["sha256"]
        );
        assert_eq!(resumed["resume"]["verified"], true);
        assert_eq!(resumed["resume"]["completed_steps"], 4);

        assert_eq!(accepted_coordinate_block(&[8, 10, 12, 64], 80, 4), Ok(4));
        assert!(accepted_coordinate_block(&[8, 8, 10, 12], 80, 4).is_err());
        assert!(accepted_coordinate_block(&[7, 10, 12, 64], 80, 4).is_err());
        assert!(accepted_coordinate_block(&[8, 10, 12, 64, 66], 80, 4).is_err());

        base_config.require_admission = true;
        let accepted_report = serde_json::json!({
            "panel_gate": {"decision": "accept"}
        });
        let rejected_report = serde_json::json!({
            "panel_gate": {"decision": "reject_admission"}
        });
        assert!(!required_gate_failed(&base_config, &accepted_report));
        assert!(required_gate_failed(&base_config, &rejected_report));
        std::fs::remove_dir_all(fixture.root).unwrap();
    }

    #[test]
    fn final_artifact_write_is_byte_identical_and_reloadable() {
        let artifact = artifact();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-cpu-corpus-artifact-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("final.uorlut");
        write_verified_artifact(&path, &artifact).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), artifact);
        IntegerStackModel::parse(&artifact).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn final_artifact_write_refuses_non_uorlut_bytes() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-cpu-corpus-invalid-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("final.uorlut");
        assert!(write_verified_artifact(&path, b"not a UORLUT01 artifact").is_err());
        assert!(!path.exists());
        assert!(!path.with_extension("partial").exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn driver_args(extra: &[&str]) -> Vec<String> {
        let mut values = [
            "artifact", "tokens", "mask", "manifest", "1", "8", "2", "16", "2", "out",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        values.extend(extra.iter().map(|value| (*value).to_owned()));
        values
    }

    #[test]
    fn update_geometry_arguments_are_positive_and_backward_compatible() {
        let default = parse_args_from(&driver_args(&[])).unwrap();
        assert_eq!(default.rank_margin, RankMarginConfig::default());
        assert!(!default.require_admission);
        let changed = parse_args_from(&driver_args(&[
            "--margin-q12",
            "2",
            "--step-q12",
            "16",
            "--parameter-limit-q12",
            "4096",
            "--coordinates-per-step",
            "4",
            "--require-admission",
        ]))
        .unwrap();
        assert_eq!(
            changed.rank_margin,
            RankMarginConfig {
                margin_q12: 2,
                step_q12: 16,
                parameter_limit_q12: 4096,
                coordinates_per_step: 4,
            }
        );
        assert!(changed.require_admission);
        for bad in [
            vec!["--step-q12", "0"],
            vec!["--margin-q12", "-1"],
            vec!["--parameter-limit-q12", "32768"],
            vec!["--coordinates-per-step", "0"],
            vec!["--coordinates-per-step", "65"],
            vec!["--step-q12"],
            vec!["--unknown", "1"],
        ] {
            assert!(parse_args_from(&driver_args(&bad)).is_err(), "{bad:?}");
        }
    }
}
