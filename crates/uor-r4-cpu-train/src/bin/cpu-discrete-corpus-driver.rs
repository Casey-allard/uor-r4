//! Bounded CPU driver over the exact discrete proposal/apply path.
//!
//! This is development evidence under D84, not GPU qualification.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::json;
use uor_r4_cpu_train::{
    validate_corpus_manifest, CpuDiscreteCheckpointDriver, CpuDiscretePanelReading,
    CpuDiscreteProposalSample, CpuDiscreteProposalScorer, CpuDiscreteSchedule, CpuMaskedCorpus,
};
use uor_r4_discrete::training::SchedulePolicy;
use uor_r4_integer::discrete_d11::{DiscreteReadMode, IntegerDiscreteModel};

const BINDING_SCHEMA: &str = "uor-r4.cpu-discrete-corpus-binding/1";
const BATCH_ROWS: u32 = 64;

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    seed: u64,
    steps: usize,
    evaluation_stride: usize,
    max_evaluation: usize,
    checkpoint_every: usize,
    checkpoint_dir: PathBuf,
    resume: Option<PathBuf>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointBinding {
    schema: String,
    source_artifact_sha256: String,
    tokens_sha256: String,
    mask_sha256: String,
    manifest_sha256: String,
    seed: u64,
    batch_rows: u32,
    evaluation_stride: usize,
    max_evaluation: usize,
    completed_steps: u64,
    checkpoint_sha256: String,
    artifact_sha256: String,
}

fn main() {
    match run() {
        Ok(report) => println!("{report}"),
        Err(error) => {
            eprintln!("cpu-discrete-corpus-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<String, String> {
    let config = parse_args()?;
    let report = execute(&config)?;
    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

fn execute(config: &Config) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact).map_err(|error| format!("artifact: {error}"))?;
    let artifact_sha256 = sha256(&artifact);
    let token_bytes = std::fs::read(&config.tokens).map_err(|error| format!("tokens: {error}"))?;
    let mask = std::fs::read(&config.mask).map_err(|error| format!("mask: {error}"))?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| format!("manifest: {error}"))?;
    let manifest_sha256 = sha256(&manifest);
    let identity = validate_corpus_manifest(&manifest, &token_bytes, &mask)
        .map_err(|error| error.to_string())?;
    if identity.split != "train" {
        return Err(format!(
            "corpus manifest split must be train, found {}",
            identity.split
        ));
    }

    let serving = IntegerDiscreteModel::from_bytes(&artifact)
        .map_err(|error| format!("discrete artifact: {error}"))?;
    let shape = serving.shape();
    if identity.vocab != shape.vocabulary as usize || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the artifact".to_owned());
    }
    if config.seed != u64::from(shape.seed) {
        return Err(format!(
            "driver seed {} differs from artifact seed {}",
            config.seed, shape.seed
        ));
    }
    identity
        .validate_context(shape.context as usize)
        .map_err(|error| error.to_string())?;
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask,
        shape.vocabulary as usize,
        shape.context as usize,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let split = corpus
        .split(config.evaluation_stride, config.max_evaluation)
        .map_err(|error| error.to_string())?;
    let training = proposal_samples(&split.training);
    let evaluation = proposal_samples(&split.evaluation);
    if training.is_empty() || evaluation.is_empty() {
        return Err("discrete corpus split needs training and evaluation rows".to_owned());
    }

    let schedule_policy = match serving.read_mode() {
        DiscreteReadMode::Route => SchedulePolicy::CanonicalVectorThenReadV1,
        DiscreteReadMode::Flock { .. } | DiscreteReadMode::Relation { .. } => {
            SchedulePolicy::InterleavedReadEvery8V1
        }
    };
    let schedule = CpuDiscreteSchedule::new(
        schedule_policy,
        1,
        BATCH_ROWS,
        u64::try_from(training.len()).map_err(|_| "training target count overflows".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    let (mut driver, resume) = if let Some(checkpoint_path) = &config.resume {
        let checkpoint = std::fs::read(checkpoint_path)
            .map_err(|error| format!("resume checkpoint: {error}"))?;
        let binding = validate_resume_binding(
            checkpoint_path,
            &checkpoint,
            &artifact_sha256,
            &identity.tokens_sha256,
            &identity.mask_sha256,
            &manifest_sha256,
            config,
        )?;
        let driver = CpuDiscreteCheckpointDriver::resume_with_artifact(
            &checkpoint,
            &artifact_sha256,
            &artifact,
        )
        .map_err(|error| error.to_string())?;
        if driver.cursor().completed_batches != binding.completed_steps {
            return Err(format!(
                "resume checkpoint cursor is at step {}, binding says {}",
                driver.cursor().completed_batches,
                binding.completed_steps
            ));
        }
        if driver.schedule().batch_rows() != BATCH_ROWS
            || driver.schedule().coordinate_block() != 1
            || driver.schedule().policy() != schedule_policy
            || driver.schedule().eligible_targets()
                != u64::try_from(training.len()).unwrap_or(u64::MAX)
        {
            return Err("resume schedule differs from the requested corpus driver".to_owned());
        }
        (
            driver,
            json!({
                "path": checkpoint_path,
                "checkpoint_sha256": sha256(&checkpoint),
                "completed_steps": binding.completed_steps,
            }),
        )
    } else {
        (
            CpuDiscreteCheckpointDriver::from_artifact(&artifact, schedule)
                .map_err(|error| error.to_string())?,
            serde_json::Value::Null,
        )
    };

    let source_scorer =
        CpuDiscreteProposalScorer::from_bytes(&artifact).map_err(|error| error.to_string())?;
    let baseline = source_scorer
        .evaluate_samples(&evaluation)
        .map_err(|error| error.to_string())?;
    let baseline_export = source_scorer
        .export_parity(&artifact, &evaluation)
        .map_err(|error| error.to_string())?;
    if !baseline_export.is_exact() {
        return Err(format!(
            "source artifact CPU/serving parity differs: {baseline_export:?}"
        ));
    }

    std::fs::create_dir_all(&config.checkpoint_dir)
        .map_err(|error| format!("checkpoint directory: {error}"))?;
    let completed = driver.cursor().completed_batches;
    let target_steps =
        u64::try_from(config.steps).map_err(|_| "target steps exceed u64".to_owned())?;
    if completed > target_steps {
        return Err(format!(
            "resume checkpoint is at step {completed}, beyond target {target_steps}"
        ));
    }
    if completed < target_steps && config.checkpoint_every == 0 {
        return Err("checkpoint every must be positive for remaining steps".to_owned());
    }

    let mut step_elapsed = Duration::ZERO;
    let mut accepted = 0u64;
    let mut refused = 0u64;
    let mut context_tokens = 0u64;
    let mut target_visits = 0u64;
    let mut checkpoints = Vec::new();
    for step in completed + 1..=target_steps {
        let block = driver.next_block().map_err(|error| error.to_string())?;
        if block.indices.len() != 1 {
            return Err(format!(
                "bounded driver currently requires coordinate block 1, found {}",
                block.indices.len()
            ));
        }
        let batch = training_batch(&training, driver.cursor().next_target, BATCH_ROWS)?;
        context_tokens = context_tokens
            .checked_add(
                batch
                    .iter()
                    .map(|sample| sample.prefix.len() as u64)
                    .sum::<u64>(),
            )
            .ok_or_else(|| "context token count overflows".to_owned())?;
        let step_started = Instant::now();
        let scoring = driver
            .score(block.kind, block.indices[0], &batch)
            .map_err(|error| error.to_string())?;
        target_visits = target_visits
            .checked_add(
                u64::try_from(scoring.eligible_rows)
                    .map_err(|_| "eligible row count overflows".to_owned())?,
            )
            .ok_or_else(|| "target visit count overflows".to_owned())?;
        let receipt = driver
            .applier_mut()
            .apply(block.kind, block.indices[0], scoring.proposal)
            .map_err(|error| error.to_string())?;
        driver
            .advance_after_batch(receipt.applied, u64::from(receipt.applied))
            .map_err(|error| error.to_string())?;
        step_elapsed += step_started.elapsed();
        if receipt.applied {
            accepted += 1;
        } else {
            refused += 1;
        }

        if step % config.checkpoint_every as u64 == 0 {
            checkpoints.push(write_checkpoint(
                &driver,
                config,
                &identity,
                &manifest_sha256,
                step,
                &artifact_sha256,
            )?);
        }
    }

    let final_checkpoint = match checkpoints.last() {
        Some(checkpoint) if checkpoint["step"].as_u64() == Some(target_steps) => checkpoint.clone(),
        _ => write_checkpoint(
            &driver,
            config,
            &identity,
            &manifest_sha256,
            target_steps,
            &artifact_sha256,
        )?,
    };
    let final_artifact = driver.artifact_bytes().map_err(|error| error.to_string())?;
    write_new(&config.checkpoint_dir.join("final.uorlut"), &final_artifact)
        .map_err(|error| format!("final artifact: {error}"))?;
    let final_sha256 = sha256(&final_artifact);
    let final_scorer = CpuDiscreteProposalScorer::from_bytes(&final_artifact)
        .map_err(|error| error.to_string())?;
    let final_panel = final_scorer
        .evaluate_samples(&evaluation)
        .map_err(|error| error.to_string())?;
    let final_export = final_scorer
        .export_parity(&final_artifact, &evaluation)
        .map_err(|error| error.to_string())?;
    if !final_export.is_exact() {
        return Err(format!(
            "trained artifact CPU/serving parity differs: {final_export:?}"
        ));
    }
    if final_panel.target_admitted < baseline.target_admitted {
        return Err(format!(
            "training lost target admission: {} -> {}",
            baseline.target_admitted, final_panel.target_admitted
        ));
    }

    let step_seconds = step_elapsed.as_secs_f64();
    let elapsed = started.elapsed().as_secs_f64();
    Ok(json!({
        "schema": "uor-r4.cpu-discrete-corpus-driver/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "source_artifact_sha256": artifact_sha256,
        "final_artifact_sha256": final_sha256,
        "source_epoch": serving.shape().epoch,
        "final_epoch": driver.applier().epoch(),
        "inputs": {
            "tokens_sha256": identity.tokens_sha256,
            "mask_sha256": identity.mask_sha256,
            "manifest_sha256": manifest_sha256,
            "seed": config.seed,
            "documents": corpus.document_count(),
            "training_documents": split.training_documents,
            "evaluation_documents": split.evaluation_documents,
            "training_targets": split.training_targets,
            "evaluation_targets": split.evaluation_targets,
        },
        "schedule": {
            "policy": schedule_policy,
            "coordinate_block": 1,
            "batch_rows": BATCH_ROWS,
            "source_epoch": serving.shape().epoch,
        },
        "training": {
            "resumed_from_step": completed,
            "completed_steps": driver.cursor().completed_batches,
            "executed_steps_this_invocation": target_steps - completed,
            "accepted_updates": driver.cursor().accepted_updates,
            "refused_updates": driver.cursor().completed_batches
                - driver.cursor().accepted_updates,
            "accepted_updates_this_invocation": accepted,
            "refused_updates_this_invocation": refused,
            "context_tokens_this_invocation": context_tokens,
            "target_visits_this_invocation": target_visits,
            "step_seconds_this_invocation": step_seconds,
            "context_tokens_per_second_this_invocation": context_tokens as f64 / step_seconds,
            "target_visits_per_second_this_invocation": target_visits as f64 / step_seconds,
        },
        "panel": panel_json(&baseline, &final_panel),
        "export": {
            "baseline": baseline_export,
            "final": final_export,
        },
        "resume": resume,
        "checkpoints": checkpoints,
        "final_checkpoint": final_checkpoint,
        "wall_seconds": elapsed,
        "peak_rss_kib": peak_rss_kib(),
        "notes": [
            "The driver executes a bounded sequence of exact integer counterfactual updates on CPU.",
            "The CPU candidate scorer is exact at the tested scope; this is not GPU training or language qualification.",
            "Final artifacts are four-table uor-r4.lut-discrete/1 writebacks; the 19-table M1 trunk is refused by this driver."
        ]
    }))
}

fn proposal_samples(
    samples: &[uor_r4_cpu_train::RankMarginSample],
) -> Vec<CpuDiscreteProposalSample> {
    samples
        .iter()
        .map(|sample| CpuDiscreteProposalSample {
            prefix: sample.input.clone(),
            target: sample.target,
        })
        .collect()
}

fn training_batch(
    training: &[CpuDiscreteProposalSample],
    next_target: u64,
    rows: u32,
) -> Result<Vec<CpuDiscreteProposalSample>, String> {
    let start = usize::try_from(next_target)
        .map_err(|_| "target cursor exceeds address space".to_owned())?;
    if start >= training.len() {
        return Err(format!(
            "target cursor {start} is outside the {} eligible rows",
            training.len()
        ));
    }
    let end = start.saturating_add(rows as usize).min(training.len());
    Ok(training[start..end].to_vec())
}

fn panel_json(
    before: &CpuDiscretePanelReading,
    after: &CpuDiscretePanelReading,
) -> serde_json::Value {
    json!({
        "before": before,
        "after": after,
        "delta": {
            "target_admitted": after.target_admitted as i64 - before.target_admitted as i64,
            "top1_correct": after.top1_correct as i64 - before.top1_correct as i64,
            "top8_correct": after.top8_correct as i64 - before.top8_correct as i64,
            "margin_loss_sum": after.margin_loss_sum as i128 - before.margin_loss_sum as i128,
            "target_rank_sum": after.target_rank_sum as i128 - before.target_rank_sum as i128,
        }
    })
}

fn write_checkpoint(
    driver: &CpuDiscreteCheckpointDriver,
    config: &Config,
    identity: &uor_r4_cpu_train::CpuCorpusIdentity,
    manifest_sha256: &str,
    step: u64,
    source_artifact_sha256: &str,
) -> Result<serde_json::Value, String> {
    let checkpoint = driver
        .checkpoint_bytes()
        .map_err(|error| error.to_string())?;
    let artifact = driver.artifact_bytes().map_err(|error| error.to_string())?;
    let checkpoint_path = config.checkpoint_dir.join(format!("step-{step:012}.ckpt"));
    let artifact_path = config
        .checkpoint_dir
        .join(format!("step-{step:012}.uorlut"));
    let binding_path = config
        .checkpoint_dir
        .join(format!("step-{step:012}.binding.json"));
    let checkpoint_sha256 = sha256(&checkpoint);
    let artifact_sha256 = sha256(&artifact);
    let binding = CheckpointBinding {
        schema: BINDING_SCHEMA.into(),
        source_artifact_sha256: source_artifact_sha256.to_owned(),
        tokens_sha256: identity.tokens_sha256.clone(),
        mask_sha256: identity.mask_sha256.clone(),
        manifest_sha256: manifest_sha256.to_owned(),
        seed: config.seed,
        batch_rows: BATCH_ROWS,
        evaluation_stride: config.evaluation_stride,
        max_evaluation: config.max_evaluation,
        completed_steps: step,
        checkpoint_sha256: checkpoint_sha256.clone(),
        artifact_sha256: artifact_sha256.clone(),
    };
    write_new(&checkpoint_path, &checkpoint).map_err(|error| format!("checkpoint: {error}"))?;
    write_new(&artifact_path, &artifact).map_err(|error| format!("artifact: {error}"))?;
    write_new(
        &binding_path,
        &serde_json::to_vec_pretty(&binding).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("binding: {error}"))?;
    Ok(json!({
        "step": step,
        "checkpoint": checkpoint_path,
        "checkpoint_sha256": checkpoint_sha256,
        "artifact": artifact_path,
        "artifact_sha256": artifact_sha256,
        "binding": binding_path,
    }))
}

fn validate_resume_binding(
    checkpoint_path: &Path,
    checkpoint: &[u8],
    artifact_sha256: &str,
    tokens_sha256: &str,
    mask_sha256: &str,
    manifest_sha256: &str,
    config: &Config,
) -> Result<CheckpointBinding, String> {
    let completed_steps = checkpoint_steps(checkpoint_path)?;
    let binding_path = checkpoint_path.with_extension("binding.json");
    let binding: CheckpointBinding = serde_json::from_slice(
        &std::fs::read(&binding_path).map_err(|error| format!("binding: {error}"))?,
    )
    .map_err(|error| format!("binding: {error}"))?;
    if binding.schema != BINDING_SCHEMA
        || binding.source_artifact_sha256 != artifact_sha256
        || binding.tokens_sha256 != tokens_sha256
        || binding.mask_sha256 != mask_sha256
        || binding.manifest_sha256 != manifest_sha256
        || binding.seed != config.seed
        || binding.batch_rows != BATCH_ROWS
        || binding.evaluation_stride != config.evaluation_stride
        || binding.max_evaluation != config.max_evaluation
        || binding.completed_steps != completed_steps
        || binding.checkpoint_sha256 != sha256(checkpoint)
    {
        return Err("resume binding differs from the requested run".to_owned());
    }
    let artifact_path = checkpoint_path.with_extension("uorlut");
    let artifact = std::fs::read(&artifact_path)
        .map_err(|error| format!("{}: {error}", artifact_path.display()))?;
    if binding.artifact_sha256 != sha256(&artifact) {
        return Err("resume binding artifact bytes differ".to_owned());
    }
    Ok(binding)
}

fn checkpoint_steps(path: &Path) -> Result<u64, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("step-"))
        .and_then(|name| name.strip_suffix(".ckpt"))
        .and_then(|step| step.parse().ok())
        .ok_or_else(|| format!("{}: checkpoint name is not step-NNNN.ckpt", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn parse_args() -> Result<Config, String> {
    let values = std::env::args().skip(1).collect::<Vec<_>>();
    if !(10..=11).contains(&values.len()) {
        return Err("usage: cpu-discrete-corpus-driver <artifact> <tokens.u16> <mask.u8> <manifest.json> <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> <checkpoint-dir> [resume-checkpoint]".to_owned());
    }
    let number = |name: &str, value: &str| -> Result<u64, String> {
        value
            .parse()
            .map_err(|_| format!("{name}: expected a nonnegative integer"))
    };
    Ok(Config {
        artifact: PathBuf::from(&values[0]),
        tokens: PathBuf::from(&values[1]),
        mask: PathBuf::from(&values[2]),
        manifest: PathBuf::from(&values[3]),
        seed: number("seed", &values[4])?,
        steps: usize::try_from(number("steps", &values[5])?)
            .map_err(|_| "steps exceeds usize".to_owned())?,
        evaluation_stride: usize::try_from(number("evaluation stride", &values[6])?)
            .map_err(|_| "evaluation stride exceeds usize".to_owned())?,
        max_evaluation: usize::try_from(number("max evaluation", &values[7])?)
            .map_err(|_| "max evaluation exceeds usize".to_owned())?,
        checkpoint_every: usize::try_from(number("checkpoint every", &values[8])?)
            .map_err(|_| "checkpoint every exceeds usize".to_owned())?,
        checkpoint_dir: PathBuf::from(&values[9]),
        resume: values.get(10).map(PathBuf::from),
    })
}

fn sha256(bytes: &[u8]) -> String {
    uor_r4_discrete::corpus::sha256(bytes)
}

fn peak_rss_kib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmHWM:")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn training_batch_takes_the_short_final_batch() {
        let training = (0u32..5)
            .map(|target| CpuDiscreteProposalSample {
                prefix: vec![0, target],
                target,
            })
            .collect::<Vec<_>>();
        let batch = training_batch(&training, 4, 4).unwrap();
        assert_eq!(
            batch.iter().map(|sample| sample.target).collect::<Vec<_>>(),
            vec![4]
        );
        assert!(training_batch(&training, 5, 4).is_err());
    }

    #[test]
    fn interval_checkpoint_step_is_read_from_its_name() {
        assert_eq!(
            checkpoint_steps(Path::new("step-000000000064.ckpt")).unwrap(),
            64
        );
        assert!(checkpoint_steps(Path::new("final.ckpt")).is_err());
    }

    #[test]
    fn resume_binding_checks_the_step_name_and_sibling_artifact() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cpu-discrete-resume-binding-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let checkpoint_path = root.join("step-000000000002.ckpt");
        let checkpoint = b"checkpoint";
        let artifact = b"artifact";
        std::fs::write(&checkpoint_path, checkpoint).unwrap();
        std::fs::write(checkpoint_path.with_extension("uorlut"), artifact).unwrap();
        let config = Config {
            artifact: PathBuf::new(),
            tokens: PathBuf::new(),
            mask: PathBuf::new(),
            manifest: PathBuf::new(),
            seed: 1,
            steps: 4,
            evaluation_stride: 2,
            max_evaluation: 128,
            checkpoint_every: 2,
            checkpoint_dir: PathBuf::new(),
            resume: None,
        };
        let binding = CheckpointBinding {
            schema: BINDING_SCHEMA.into(),
            source_artifact_sha256: "00".repeat(32),
            tokens_sha256: "11".repeat(32),
            mask_sha256: "22".repeat(32),
            manifest_sha256: "33".repeat(32),
            seed: 1,
            batch_rows: BATCH_ROWS,
            evaluation_stride: 2,
            max_evaluation: 128,
            completed_steps: 2,
            checkpoint_sha256: sha256(checkpoint),
            artifact_sha256: sha256(artifact),
        };
        std::fs::write(
            checkpoint_path.with_extension("binding.json"),
            serde_json::to_vec(&binding).unwrap(),
        )
        .unwrap();
        let validate = || {
            validate_resume_binding(
                &checkpoint_path,
                checkpoint,
                &binding.source_artifact_sha256,
                &binding.tokens_sha256,
                &binding.mask_sha256,
                &binding.manifest_sha256,
                &config,
            )
        };
        assert_eq!(validate().unwrap().completed_steps, 2);

        let mut wrong_step = binding.clone();
        wrong_step.completed_steps = 3;
        std::fs::write(
            checkpoint_path.with_extension("binding.json"),
            serde_json::to_vec(&wrong_step).unwrap(),
        )
        .unwrap();
        assert!(validate()
            .unwrap_err()
            .contains("binding differs from the requested run"));
        std::fs::write(
            checkpoint_path.with_extension("binding.json"),
            serde_json::to_vec(&binding).unwrap(),
        )
        .unwrap();

        std::fs::write(checkpoint_path.with_extension("uorlut"), b"changed").unwrap();
        assert!(validate().unwrap_err().contains("artifact bytes differ"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn evidence_files_are_never_replaced() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cpu-discrete-write-new-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("record.json");
        write_new(&path, b"first").unwrap();
        assert!(write_new(&path, b"second").is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"first");
        std::fs::remove_dir_all(root).unwrap();
    }
}
