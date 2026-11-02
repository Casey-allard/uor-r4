use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use uor_r4_cpu_train::{
    panel_admission_gate, validate_corpus_manifest, CpuMaskedCorpus, CpuPanelEvaluator,
    CpuRankMarginTrainer, PanelGateDecision, RankMarginConfig,
};
use uor_r4_integer::stack::IntegerStackModel;

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
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-corpus-driver: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact).map_err(|error| format!("artifact: {error}"))?;
    let token_bytes = std::fs::read(&config.tokens).map_err(|error| format!("tokens: {error}"))?;
    let mask = std::fs::read(&config.mask).map_err(|error| format!("mask: {error}"))?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| format!("manifest: {error}"))?;
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
        RankMarginConfig::default().margin_q12,
    )
    .map_err(|error| error.to_string())?;
    let baseline = baseline_panel
        .evaluate()
        .map_err(|error| error.to_string())?;
    let mut trainer = CpuRankMarginTrainer::from_artifact(
        &artifact,
        split.training.clone(),
        RankMarginConfig::default(),
        config.seed,
    )
    .map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&config.checkpoint_dir)
        .map_err(|error| format!("checkpoint directory: {error}"))?;
    let mut step_elapsed = Duration::ZERO;
    let mut checkpoint_elapsed = Duration::ZERO;
    let mut accepted = 0usize;
    let mut refused = 0usize;
    let mut context_tokens = 0usize;
    let mut target_visits = 0usize;
    let mut checkpoints = Vec::new();
    for completed in 1..=config.steps {
        let step_started = Instant::now();
        let step = trainer.train_step().map_err(|error| error.to_string())?;
        step_elapsed += step_started.elapsed();
        target_visits = target_visits
            .checked_add(1)
            .ok_or_else(|| "target visit count overflow".to_owned())?;
        if step.updated {
            accepted += 1;
        } else {
            refused += 1;
        }
        context_tokens = context_tokens
            .checked_add(split.training[step.sample].input.len())
            .ok_or_else(|| "training context-token count overflow".to_owned())?;
        if completed % config.checkpoint_every == 0 {
            let checkpoint_started = Instant::now();
            let bytes = trainer.checkpoint().map_err(|error| error.to_string())?;
            let path = config
                .checkpoint_dir
                .join(format!("step-{completed:012}.ckpt"));
            std::fs::write(&path, &bytes)
                .map_err(|error| format!("checkpoint {}: {error}", path.display()))?;
            checkpoints.push(serde_json::json!({
                "step": completed,
                "path": path,
                "sha256": uor_r4_discrete::corpus::sha256(&bytes),
            }));
            checkpoint_elapsed += checkpoint_started.elapsed();
        }
    }

    let mut final_panel = CpuPanelEvaluator::from_artifact(
        trainer.artifact(),
        split.evaluation.clone(),
        RankMarginConfig::default().margin_q12,
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
    std::fs::write(&final_checkpoint_path, &final_checkpoint)
        .map_err(|error| format!("final checkpoint: {error}"))?;
    let step_seconds = step_elapsed.as_secs_f64();
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-masked-corpus-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "artifact": {
            "path": config.artifact,
            "sha256": uor_r4_discrete::corpus::sha256(&artifact),
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
            "steps": config.steps,
            "accepted_updates": accepted,
            "refused_steps": refused,
            "target_visits": target_visits,
            "context_tokens": context_tokens,
            "step_elapsed_ms": step_elapsed.as_secs_f64() * 1_000.0,
            "mean_step_ms": if config.steps > 0 {
                step_elapsed.as_secs_f64() * 1_000.0 / config.steps as f64
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
                config.steps as f64 / step_seconds
            } else {
                0.0
            },
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        },
        "panel_baseline": panel_json(&baseline),
        "panel_final": panel_json(&final_reading),
        "panel_gate": gate,
        "checkpoints": checkpoints,
        "final_checkpoint": {
            "path": final_checkpoint_path,
            "sha256": uor_r4_discrete::corpus::sha256(&final_checkpoint),
        },
        "scope": "CPU development-research component run; not GPU qualification, language quality or complete-reply evidence",
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    if !baseline.export.is_exact() || !final_reading.export.is_exact() {
        return Err("CPU/serving query, token or kept-cell export mismatch".to_owned());
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
    if values.len() != 10 {
        return Err(
            "usage: cpu-corpus-driver <artifact> <tokens.u16> <mask.u8> <manifest.json> \
             <seed> <steps> <evaluation-stride> <max-evaluation> <checkpoint-every> \
             <checkpoint-dir>"
                .to_owned(),
        );
    }
    let parse_usize = |index: usize, name: &str| -> Result<usize, String> {
        values[index].parse().map_err(|_| format!("{name} differs"))
    };
    let steps = parse_usize(5, "steps")?;
    let checkpoint_every = parse_usize(8, "checkpoint every")?;
    if steps > 0 && checkpoint_every == 0 {
        return Err("checkpoint every must be positive".to_owned());
    }
    Ok(Config {
        artifact: Path::new(&values[0]).to_path_buf(),
        tokens: Path::new(&values[1]).to_path_buf(),
        mask: Path::new(&values[2]).to_path_buf(),
        manifest: Path::new(&values[3]).to_path_buf(),
        seed: values[4].parse().map_err(|_| "seed differs".to_owned())?,
        steps,
        evaluation_stride: parse_usize(6, "evaluation stride")?,
        max_evaluation: parse_usize(7, "max evaluation")?,
        checkpoint_every,
        checkpoint_dir: Path::new(&values[9]).to_path_buf(),
    })
}
