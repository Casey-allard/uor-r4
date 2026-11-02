use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    panel_admission_gate, validate_corpus_manifest, CounterfactualConfig, CounterfactualGroup,
    CounterfactualStep, CpuCounterfactualTrainer, CpuMaskedCorpus, CpuPanelEvaluator,
    PanelGateDecision,
};
use uor_r4_integer::stack::IntegerStackModel;

const SCHEMA: &str = "uor-r4.cpu-counterfactual-corpus-driver/1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Arm {
    Baseline,
    Group(CounterfactualGroup),
}

impl Arm {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "baseline" => Ok(Self::Baseline),
            "normalization" => Ok(Self::Group(CounterfactualGroup::Normalization)),
            "bias" => Ok(Self::Group(CounterfactualGroup::Bias)),
            "decay" => Ok(Self::Group(CounterfactualGroup::Decay)),
            "route" => Ok(Self::Group(CounterfactualGroup::Route)),
            _ => Err(format!(
                "GROUP must be baseline, normalization, bias, decay or route; found {value}"
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Group(CounterfactualGroup::Normalization) => "normalization",
            Self::Group(CounterfactualGroup::Bias) => "bias",
            Self::Group(CounterfactualGroup::Decay) => "decay",
            Self::Group(CounterfactualGroup::Route) => "route",
        }
    }
}

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out_dir: PathBuf,
    seed: u64,
    arm: Arm,
    steps: usize,
    margin_q12: i32,
    delta: i32,
    coordinates_per_step: usize,
    evaluation_stride: usize,
    max_evaluation: usize,
}

fn main() {
    match parse_args().and_then(|config| run(&config)) {
        Ok(report) => println!("{report}"),
        Err(error) => {
            eprintln!("cpu-counterfactual-corpus-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let artifact = args.next().map(PathBuf::from).ok_or_else(usage_error)?;
    let tokens = args.next().map(PathBuf::from).ok_or_else(usage_error)?;
    let mask = args.next().map(PathBuf::from).ok_or_else(usage_error)?;
    let manifest = args.next().map(PathBuf::from).ok_or_else(usage_error)?;
    let out_dir = args.next().map(PathBuf::from).ok_or_else(usage_error)?;
    let mut config = Config {
        artifact,
        tokens,
        mask,
        manifest,
        out_dir,
        seed: 1,
        arm: Arm::Baseline,
        steps: 0,
        margin_q12: 1,
        delta: 1,
        coordinates_per_step: 4,
        evaluation_stride: 2,
        max_evaluation: 1_024,
    };
    let mut group_seen = false;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--seed" => config.seed = parse_number(&flag, &value)?,
            "--group" => {
                config.arm = Arm::parse(&value)?;
                group_seen = true;
            }
            "--steps" => config.steps = parse_number(&flag, &value)?,
            "--margin-q12" => config.margin_q12 = parse_number(&flag, &value)?,
            "--delta" => config.delta = parse_number(&flag, &value)?,
            "--coordinates-per-step" => config.coordinates_per_step = parse_number(&flag, &value)?,
            "--evaluation-stride" => config.evaluation_stride = parse_number(&flag, &value)?,
            "--max-evaluation" => config.max_evaluation = parse_number(&flag, &value)?,
            _ => return Err(format!("unknown flag {flag}\n{}", usage())),
        }
    }
    if !group_seen {
        return Err(format!("--group is required\n{}", usage()));
    }
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &Config) -> Result<(), String> {
    match config.arm {
        Arm::Baseline if config.steps != 0 => {
            return Err("baseline requires --steps 0".to_owned());
        }
        Arm::Group(_) if config.steps == 0 => {
            return Err("a trained group requires --steps greater than zero".to_owned());
        }
        _ => {}
    }
    if config.margin_q12 <= 0
        || config.delta <= 0
        || !(1..=64).contains(&config.coordinates_per_step)
        || config.evaluation_stride < 2
        || config.max_evaluation == 0
    {
        return Err(
            "margin, delta, coordinate block and evaluation controls are outside their bounds"
                .to_owned(),
        );
    }
    Ok(())
}

fn usage() -> String {
    "usage: cpu-counterfactual-corpus-driver ARTIFACT TOKENS MASK MANIFEST OUT_DIR --group \
     baseline|normalization|bias|decay|route [--steps N] [--seed N] [--margin-q12 N] [--delta N] \
     [--coordinates-per-step N] [--evaluation-stride N] [--max-evaluation N]"
        .to_owned()
}

fn usage_error() -> String {
    usage()
}

fn parse_number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} has an invalid value: {value}"))
}

fn run(config: &Config) -> Result<String, String> {
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact)
        .map_err(|error| format!("artifact {}: {error}", config.artifact.display()))?;
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
    identity
        .validate_context(shape.context)
        .map_err(|error| error.to_string())?;
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

    let mut baseline_evaluator =
        CpuPanelEvaluator::from_artifact(&artifact, split.evaluation.clone(), config.margin_q12)
            .map_err(|error| error.to_string())?;
    let baseline = baseline_evaluator
        .evaluate()
        .map_err(|error| error.to_string())?;
    if !baseline.export.is_exact() {
        return Err("the zero-update control has a CPU/serving export mismatch".to_owned());
    }

    let mut training = serde_json::Value::Null;
    let final_artifact;
    let final_reading;
    let gate;
    let checkpoint_sha256;
    let artifact_changed;
    match config.arm {
        Arm::Baseline => {
            final_artifact = artifact.clone();
            final_reading = baseline.clone();
            gate = serde_json::json!({
                "decision": "control",
                "delta": {
                    "target_admitted": 0,
                    "top1_correct": 0,
                    "top8_correct": 0,
                    "margin_loss_sum": 0,
                }
            });
            checkpoint_sha256 = serde_json::Value::Null;
            artifact_changed = false;
        }
        Arm::Group(group) => {
            let counterfactual = CounterfactualConfig {
                group,
                margin_q12: config.margin_q12,
                delta: config.delta,
                coordinates_per_step: config.coordinates_per_step,
            };
            let mut trainer = CpuCounterfactualTrainer::from_artifact(
                &artifact,
                split.training.clone(),
                counterfactual,
                config.seed,
            )
            .map_err(|error| error.to_string())?;
            let steps = trainer
                .train_steps(config.steps)
                .map_err(|error| error.to_string())?;
            training = training_summary(&steps);
            final_artifact = trainer.artifact().to_vec();
            artifact_changed = final_artifact != artifact;
            let checkpoint = trainer.checkpoint().map_err(|error| error.to_string())?;
            let resumed = CpuCounterfactualTrainer::resume(&checkpoint, split.training.clone())
                .map_err(|error| error.to_string())?;
            if resumed.artifact() != final_artifact || resumed.step_count() != config.steps as u64 {
                return Err("the final counterfactual checkpoint did not reload exactly".to_owned());
            }
            write_new(&config.out_dir.join("final.ckpt"), &checkpoint)?;
            checkpoint_sha256 = serde_json::json!(uor_r4_discrete::corpus::sha256(&checkpoint));

            let mut final_evaluator = CpuPanelEvaluator::from_artifact(
                &final_artifact,
                split.evaluation.clone(),
                config.margin_q12,
            )
            .map_err(|error| error.to_string())?;
            final_reading = final_evaluator
                .evaluate()
                .map_err(|error| error.to_string())?;
            gate = gate_summary(
                &panel_admission_gate(&baseline, &final_reading)
                    .map_err(|error| error.to_string())?,
            );
        }
    }
    write_new(&config.out_dir.join("final.uorlut"), &final_artifact)?;

    let report = serde_json::json!({
        "schema": SCHEMA,
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "arm": config.arm.name(),
        "seed": config.seed,
        "config": {
            "steps": config.steps,
            "margin_q12": config.margin_q12,
            "delta": config.delta,
            "coordinates_per_step": config.coordinates_per_step,
            "evaluation_stride": config.evaluation_stride,
            "max_evaluation": config.max_evaluation,
        },
        "inputs": {
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&artifact),
            "tokens_sha256": identity.tokens_sha256,
            "mask_sha256": identity.mask_sha256,
            "manifest_sha256": uor_r4_discrete::corpus::sha256(&manifest),
            "tokenizer_sha256": identity.tokenizer_sha256,
        },
        "split": {
            "training_documents": split.training_documents,
            "evaluation_documents": split.evaluation_documents,
            "training_targets": split.training_targets,
            "evaluation_targets": split.evaluation_targets,
        },
        "baseline": baseline,
        "training": training,
        "final": final_reading,
        "gate": gate,
        "artifact_changed": artifact_changed,
        "final_artifact_sha256": uor_r4_discrete::corpus::sha256(&final_artifact),
        "final_checkpoint_sha256": checkpoint_sha256,
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "peak_rss_kib": peak_rss_kib(),
    });
    serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
}

fn training_summary(steps: &[CounterfactualStep]) -> serde_json::Value {
    let accepted = steps.iter().filter(|step| step.accepted.is_some()).count();
    serde_json::json!({
        "steps": steps.len(),
        "accepted": accepted,
        "refused": steps.len() - accepted,
        "joint_updates": steps.iter().filter(|step| step.accepted.as_ref().is_some_and(|bundle| bundle.joint)).count(),
        "single_updates": steps.iter().filter(|step| step.accepted.as_ref().is_some_and(|bundle| !bundle.joint)).count(),
        "candidates_tried": steps.iter().map(|step| step.tried).sum::<usize>(),
        "eligible_coordinates": steps.iter().map(|step| step.eligible).sum::<usize>(),
        "coordinates_scanned": steps.iter().map(|step| step.coordinates).sum::<usize>(),
        "accepted_members": steps.iter().filter_map(|step| step.accepted.as_ref()).map(|bundle| bundle.members.len()).sum::<usize>(),
    })
}

fn gate_summary(gate: &PanelGateDecision) -> serde_json::Value {
    let (decision, delta) = match gate {
        PanelGateDecision::Accept(delta) => ("accept", delta),
        PanelGateDecision::RejectAdmission(delta) => ("reject_admission", delta),
        PanelGateDecision::RejectExportGap(delta) => ("reject_export_gap", delta),
    };
    serde_json::json!({
        "decision": decision,
        "delta": {
            "target_admitted": delta.target_admitted,
            "top1_correct": delta.top1_correct,
            "top8_correct": delta.top8_correct,
            "margin_loss_sum": delta.margin_loss_sum,
            "query_coordinate_mismatches": delta.query_coordinate_mismatches,
            "token_mismatches": delta.token_mismatches,
            "kept_cell_mismatches": delta.kept_cell_mismatches,
        }
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| format!("{} has no parent", path.display()))?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arms_parse_exactly_and_step_ownership_is_enforced() {
        assert_eq!(Arm::parse("baseline").unwrap(), Arm::Baseline);
        assert_eq!(
            Arm::parse("normalization").unwrap(),
            Arm::Group(CounterfactualGroup::Normalization)
        );
        assert_eq!(
            Arm::parse("route").unwrap(),
            Arm::Group(CounterfactualGroup::Route)
        );
        assert!(Arm::parse("token").is_err());

        let mut config = Config {
            artifact: PathBuf::new(),
            tokens: PathBuf::new(),
            mask: PathBuf::new(),
            manifest: PathBuf::new(),
            out_dir: PathBuf::new(),
            seed: 1,
            arm: Arm::Baseline,
            steps: 0,
            margin_q12: 1,
            delta: 1,
            coordinates_per_step: 4,
            evaluation_stride: 2,
            max_evaluation: 1_024,
        };
        assert!(validate_config(&config).is_ok());
        config.steps = 1;
        assert!(validate_config(&config).is_err());
        config.arm = Arm::Group(CounterfactualGroup::Normalization);
        config.steps = 0;
        assert!(validate_config(&config).is_err());
        config.steps = 1;
        assert!(validate_config(&config).is_ok());
    }
}
