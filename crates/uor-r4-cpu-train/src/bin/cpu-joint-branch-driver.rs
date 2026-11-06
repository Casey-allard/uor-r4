use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    compare_joint_branch_rows, derive_prompt_retrieval_memory, derive_reply_boundary_memory,
    evaluate_joint_branch_selection, evaluate_prompt_retrieval, fit_joint_branch_selector,
    fit_joint_branch_selector_on_panels, validate_corpus_manifest, CopyKeyReplyMetric,
    CpuConversationSample, CpuCorpusIdentity, CpuMaskedCorpus, JointBranchArm,
    JointBranchEvaluation, MemoryWeightSample, PromptRetrievalArm, TimedJointBranchReading,
};
use uor_r4_cpu_train::{
    derive_full_sequence_suffix_content_memory, JOINT_BRANCH_EPOCHS, JOINT_BRANCH_FEATURE_NAMES,
    JOINT_BRANCH_FOLDS, JOINT_BRANCH_LEARNING_RATE, JOINT_BRANCH_MIN_OBSERVATIONS,
    JOINT_BRANCH_SHUFFLE_SEED, JOINT_BRANCH_TAIL, JOINT_BRANCH_WEIGHT_LIMIT,
};

struct Panel {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    manifest_sha256: String,
    identity: CpuCorpusIdentity,
    conversations: Vec<CpuConversationSample>,
    samples: Vec<MemoryWeightSample>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-joint-branch-driver: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let train = load_panel(&config.train)?;
    let calibrations = config
        .calibrations
        .iter()
        .map(load_calibration)
        .collect::<Result<Vec<_>, _>>()?;
    let artifact = read(&config.artifact)?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    if artifact_sha256 != config.expected_artifact_sha256 {
        return Err(format!(
            "artifact sha256 {artifact_sha256} != expected {}",
            config.expected_artifact_sha256
        ));
    }

    let suffix = derive_full_sequence_suffix_content_memory(&train.samples)
        .map_err(|error| error.to_string())?;
    let suffix_census = suffix.census();
    if suffix_census.bos_transitions != 0 {
        return Err("full-sequence suffix table contains BOS targets".to_owned());
    }
    let boundary =
        derive_reply_boundary_memory(&train.samples).map_err(|error| error.to_string())?;
    let boundary_census = boundary.census();
    let training = if calibrations.is_empty() {
        fit_joint_branch_selector(&train.conversations, config.cap, config.seed)
            .map_err(|error| error.to_string())?
    } else {
        let calibration_samples = calibrations
            .iter()
            .flat_map(|panel| panel.samples.iter().cloned())
            .collect::<Vec<_>>();
        fit_joint_branch_selector_on_panels(
            &artifact,
            &calibration_samples,
            &suffix,
            &boundary,
            config.cap,
            233,
        )
        .map_err(|error| error.to_string())?
    };
    let eligible = training.supervised_groups >= 96 && training.baseline_wrong_groups > 0;
    if config.preflight || !eligible {
        let report = serde_json::json!({
            "schema": "uor-r4.cpu-joint-branch-preflight/1",
            "substrate": "cpu-development-research",
            "qualification": "development-only",
            "mode": if config.preflight { "preflight" } else { "auto_stop" },
            "inputs": {
                "train": panel_input(&train),
                "calibrations": calibrations.iter().map(panel_input).collect::<Vec<_>>(),
                "panel_plan": panel_path_input(&config.panel),
                "artifact": config.artifact,
                "artifact_sha256": artifact_sha256,
            },
            "tables": {
                "suffix": suffix_census,
                "boundary": boundary_census,
            },
            "training": training,
            "bar": {
                "minimum_supervised_groups": 96,
                "eligible": eligible,
                "outcome": if eligible && config.preflight {
                    "GO"
                } else {
                    "INSUFFICIENT_POWER"
                },
            },
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
        return Ok(());
    }

    let panel = load_panel(&config.panel)?;
    if calibrations.iter().any(|calibration| {
        calibration.identity.tokens_sha256 == panel.identity.tokens_sha256
            || calibration.identity.mask_sha256 == panel.identity.mask_sha256
            || calibration.manifest_sha256 == panel.manifest_sha256
    }) {
        return Err("a calibration panel overlaps the evaluation panel".to_owned());
    }

    let mut readings = BTreeMap::new();
    for arm in JointBranchArm::EVALUATION_ARMS {
        let reading = evaluate_joint_branch_selection(
            &artifact,
            JointBranchEvaluation {
                samples: &panel.samples,
                suffix_memory: &suffix,
                boundary: &boundary,
                training: &training,
                cap: config.cap,
                arm,
                seed: config.seed,
            },
        )
        .map_err(|error| error.to_string())?;
        readings.insert(arm.name(), reading);
    }
    let learned = readings
        .get(JointBranchArm::Learned.name())
        .ok_or_else(|| "learned reading is missing".to_owned())?;
    let shuffled = readings
        .get(JointBranchArm::ShuffledLabels.name())
        .ok_or_else(|| "shuffled reading is missing".to_owned())?;
    let first = readings
        .get(JointBranchArm::FirstBoundary.name())
        .ok_or_else(|| "first-boundary reading is missing".to_owned())?;
    let retrieval = derive_prompt_retrieval_memory(&artifact, &train.samples, config.cap)
        .map_err(|error| error.to_string())?;
    let full_retrieval = evaluate_prompt_retrieval(
        &artifact,
        &retrieval,
        &retrieval,
        &panel.samples,
        config.cap,
        PromptRetrievalArm::UnigramJaccardOrdinalTie,
    )
    .map_err(|error| error.to_string())?;

    let exact_vs_shuffled = compare(learned, shuffled, CopyKeyReplyMetric::ExactReference)?;
    let body_vs_shuffled = compare(learned, shuffled, CopyKeyReplyMetric::CorrectBodyPositions)?;
    let exact_vs_first = compare(learned, first, CopyKeyReplyMetric::ExactReference)?;
    let body_vs_first = compare(learned, first, CopyKeyReplyMetric::CorrectBodyPositions)?;

    let reachable = learned.reading.reference_reachable;
    let all_emitted = learned.reading.bos_complete == reachable
        && learned.reading.selected_candidates == reachable;
    let exact_retained = learned.reading.exact_reference >= first.reading.exact_reference;
    let body_retained =
        learned.reading.correct_body_positions >= first.reading.correct_body_positions;
    let changed_from_first = learned.reading.changed_from_first > 0;
    let changed_from_shuffled = complete_rows_changed(learned, shuffled) > 0;
    let exact_shuffled = exact_vs_shuffled["passes"].as_bool().unwrap_or(false);
    let exact_first = exact_vs_first["passes"].as_bool().unwrap_or(false);
    let shuffle_active =
        training.shuffled_label_changes > 0 && training.trained != training.shuffled;
    let outcome = if training.supervised_groups < 96
        || reachable < 200
        || learned.reading.changed_from_first < 100
        || learned.reading.exact_candidate_rows < 20
    {
        "INSUFFICIENT_POWER"
    } else if !shuffle_active
        || learned.reading.selected_candidates == 0
        || !changed_from_first
        || !changed_from_shuffled
    {
        "CONTROL_FAULT"
    } else if all_emitted
        && learned.reading.premature_bos == 0
        && exact_retained
        && body_retained
        && exact_shuffled
        && exact_first
    {
        "KEEP"
    } else {
        "REJECT"
    };

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-joint-branch-selection/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "claim": "integer pairwise selection over complete boundary branches cut from one full-sequence geometric content spine",
        "inputs": {
            "train": panel_input(&train),
            "calibrations": calibrations.iter().map(panel_input).collect::<Vec<_>>(),
            "panel": panel_input(&panel),
            "artifact": config.artifact,
            "artifact_sha256": artifact_sha256,
        },
        "settings": {
            "cap": config.cap,
            "seed": config.seed,
            "features": JOINT_BRANCH_FEATURE_NAMES,
            "epochs": JOINT_BRANCH_EPOCHS,
            "learning_rate": JOINT_BRANCH_LEARNING_RATE,
            "weight_limit": JOINT_BRANCH_WEIGHT_LIMIT,
            "minimum_observations": JOINT_BRANCH_MIN_OBSERVATIONS,
            "tail": JOINT_BRANCH_TAIL,
            "shuffle_seed": JOINT_BRANCH_SHUFFLE_SEED,
            "folds": JOINT_BRANCH_FOLDS,
            "calibration_generation_seed": (!calibrations.is_empty()).then_some(233),
        },
        "tables": {
            "suffix": suffix_census,
            "boundary": boundary_census,
        },
        "training": training,
        "readings": readings
            .iter()
            .map(|(name, reading)| ((*name).to_owned(), serde_json::to_value(&reading.reading).unwrap()))
            .collect::<BTreeMap<_, _>>(),
        "full_retrieval": full_retrieval.reading,
        "comparisons": {
            "exact_vs_shuffled": exact_vs_shuffled,
            "body_vs_shuffled": body_vs_shuffled,
            "exact_vs_first": exact_vs_first,
            "body_vs_first": body_vs_first,
        },
        "hashes": readings
            .iter()
            .map(|(name, reading)| {
                (
                    (*name).to_owned(),
                    serde_json::json!({
                        "reply": reading.reply_sha256,
                        "decision": reading.decision_sha256,
                        "wall_elapsed_us": reading.wall_elapsed_us,
                        "generated_tokens_per_second": reading.generated_tokens_per_second,
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>(),
        "bar": {
            "reachable": reachable,
            "all_emitted": all_emitted,
            "exact_retained": exact_retained,
            "body_retained": body_retained,
            "exact_vs_shuffled": exact_shuffled,
            "exact_vs_first": exact_first,
            "changed_from_first": changed_from_first,
            "changed_from_shuffled": changed_from_shuffled,
            "shuffle_active": shuffle_active,
            "outcome": outcome,
        },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn compare(
    candidate: &TimedJointBranchReading,
    control: &TimedJointBranchReading,
    metric: CopyKeyReplyMetric,
) -> Result<serde_json::Value, String> {
    serde_json::to_value(
        compare_joint_branch_rows(&candidate.rows, &control.rows, metric)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn complete_rows_changed(
    candidate: &TimedJointBranchReading,
    control: &TimedJointBranchReading,
) -> u64 {
    candidate
        .rows
        .iter()
        .zip(&control.rows)
        .filter(|(candidate, control)| {
            candidate.reply_sha256 != control.reply_sha256 || candidate.stop != control.stop
        })
        .count() as u64
}

fn panel_input(panel: &Panel) -> serde_json::Value {
    serde_json::json!({
        "tokens": panel.tokens,
        "tokens_sha256": panel.identity.tokens_sha256,
        "mask": panel.mask,
        "mask_sha256": panel.identity.mask_sha256,
        "manifest": panel.manifest,
        "manifest_sha256": panel.manifest_sha256,
        "manifest_split": panel.identity.split,
    })
}

fn load_panel(paths: &PanelPaths) -> Result<Panel, String> {
    let token_bytes = read(&paths.tokens)?;
    let mask_bytes = read(&paths.mask)?;
    let manifest_bytes = read(&paths.manifest)?;
    let manifest_sha256 = uor_r4_discrete::corpus::sha256(&manifest_bytes);
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    if identity.tokens_sha256 != paths.expected_tokens_sha256
        || identity.mask_sha256 != paths.expected_mask_sha256
        || manifest_sha256 != paths.expected_manifest_sha256
    {
        return Err(format!(
            "{}: panel identity differs",
            paths.tokens.display()
        ));
    }
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask_bytes,
        identity.vocab,
        identity.max_tokens,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let conversations = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let samples = conversations
        .iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt.clone(),
            reply: sample.reply.clone(),
        })
        .collect();
    Ok(Panel {
        tokens: paths.tokens.clone(),
        mask: paths.mask.clone(),
        manifest: paths.manifest.clone(),
        manifest_sha256,
        identity,
        conversations,
        samples,
    })
}

fn load_calibration(path: &CalibrationPath) -> Result<Panel, String> {
    let tokens = path.directory.join("tokens.u16");
    let mask = path.directory.join("response_mask.u8");
    let manifest = path.directory.join("manifest.json");
    let manifest_bytes = read(&manifest)?;
    let manifest_sha256 = uor_r4_discrete::corpus::sha256(&manifest_bytes);
    if manifest_sha256 != path.expected_manifest_sha256 {
        return Err(format!(
            "{}: calibration manifest differs",
            manifest.display()
        ));
    }
    let token_bytes = read(&tokens)?;
    let mask_bytes = read(&mask)?;
    let identity = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask_bytes)
        .map_err(|error| error.to_string())?;
    load_panel(&PanelPaths {
        tokens,
        mask,
        manifest,
        expected_tokens_sha256: identity.tokens_sha256,
        expected_mask_sha256: identity.mask_sha256,
        expected_manifest_sha256: manifest_sha256,
    })
}

fn panel_path_input(paths: &PanelPaths) -> serde_json::Value {
    serde_json::json!({
        "tokens": paths.tokens,
        "tokens_sha256": paths.expected_tokens_sha256,
        "mask": paths.mask,
        "mask_sha256": paths.expected_mask_sha256,
        "manifest": paths.manifest,
        "manifest_sha256": paths.expected_manifest_sha256,
    })
}

struct PanelPaths {
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

struct Config {
    train: PanelPaths,
    panel: PanelPaths,
    calibrations: Vec<CalibrationPath>,
    artifact: PathBuf,
    expected_artifact_sha256: String,
    cap: usize,
    seed: u64,
    preflight: bool,
}

struct CalibrationPath {
    directory: PathBuf,
    expected_manifest_sha256: String,
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut cap = 64usize;
    let mut seed = 231u64;
    let mut preflight = false;
    let mut calibration_directories = Vec::new();
    let mut calibration_manifests = Vec::new();
    let mut expected = BTreeMap::<String, String>::new();
    while let Some(flag) = args.next() {
        if !flag.starts_with("--") {
            positional.push(PathBuf::from(flag));
            continue;
        }
        if flag == "--preflight" {
            preflight = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))?;
        match flag.as_str() {
            "--cap" => {
                cap = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--seed" => {
                seed = value
                    .parse()
                    .map_err(|_| format!("not a number: {value}"))?
            }
            "--calibration" => calibration_directories.push(PathBuf::from(value)),
            "--expected-calibration-manifest" => calibration_manifests.push(value),
            "--expected-train-tokens" => set_expected(&mut expected, "train_tokens", value, &flag)?,
            "--expected-train-mask" => set_expected(&mut expected, "train_mask", value, &flag)?,
            "--expected-train-manifest" => {
                set_expected(&mut expected, "train_manifest", value, &flag)?
            }
            "--expected-panel-tokens" => set_expected(&mut expected, "panel_tokens", value, &flag)?,
            "--expected-panel-mask" => set_expected(&mut expected, "panel_mask", value, &flag)?,
            "--expected-panel-manifest" => {
                set_expected(&mut expected, "panel_manifest", value, &flag)?
            }
            "--expected-artifact" => set_expected(&mut expected, "artifact", value, &flag)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if positional.len() != 7 || !(1..=64).contains(&cap) {
        return Err(usage());
    }
    if calibration_directories.len() != calibration_manifests.len() {
        return Err("--calibration and --expected-calibration-manifest counts differ".to_owned());
    }
    if calibration_manifests
        .iter()
        .any(|value| value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err("--expected-calibration-manifest is not a SHA-256".to_owned());
    }
    for key in [
        "train_tokens",
        "train_mask",
        "train_manifest",
        "panel_tokens",
        "panel_mask",
        "panel_manifest",
        "artifact",
    ] {
        let value = expected
            .get(key)
            .ok_or_else(|| format!("--expected-{} is required", key.replace('_', "-")))?;
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!("--expected-{key} is not a SHA-256"));
        }
    }
    let panel_paths = |start: usize, prefix: &str| PanelPaths {
        tokens: positional[start].clone(),
        mask: positional[start + 1].clone(),
        manifest: positional[start + 2].clone(),
        expected_tokens_sha256: expected[&format!("{prefix}_tokens")].to_ascii_lowercase(),
        expected_mask_sha256: expected[&format!("{prefix}_mask")].to_ascii_lowercase(),
        expected_manifest_sha256: expected[&format!("{prefix}_manifest")].to_ascii_lowercase(),
    };
    Ok(Config {
        train: panel_paths(0, "train"),
        panel: panel_paths(3, "panel"),
        calibrations: calibration_directories
            .into_iter()
            .zip(calibration_manifests)
            .map(|(directory, expected_manifest_sha256)| CalibrationPath {
                directory,
                expected_manifest_sha256: expected_manifest_sha256.to_ascii_lowercase(),
            })
            .collect(),
        artifact: positional[6].clone(),
        expected_artifact_sha256: expected["artifact"].to_ascii_lowercase(),
        cap,
        seed,
        preflight,
    })
}

fn set_expected(
    expected: &mut BTreeMap<String, String>,
    key: &str,
    value: String,
    flag: &str,
) -> Result<(), String> {
    if expected.insert(key.to_owned(), value).is_some() {
        return Err(format!("{flag} given more than once"));
    }
    Ok(())
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn usage() -> String {
    "usage: cpu-joint-branch-driver TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
     PANEL_TOKENS PANEL_MASK PANEL_MANIFEST ARTIFACT \
     --expected-train-tokens HEX --expected-train-mask HEX \
     --expected-train-manifest HEX --expected-panel-tokens HEX \
     --expected-panel-mask HEX --expected-panel-manifest HEX \
     --expected-artifact HEX [--calibration DIR \
     --expected-calibration-manifest HEX]... [--cap N] [--seed N] [--preflight]"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_every_required_input() {
        let usage = usage();
        for word in [
            "TRAIN_TOKENS",
            "TRAIN_MASK",
            "TRAIN_MANIFEST",
            "PANEL_TOKENS",
            "PANEL_MASK",
            "PANEL_MANIFEST",
            "ARTIFACT",
            "--expected-artifact",
            "--calibration",
            "--expected-calibration-manifest",
            "--preflight",
        ] {
            assert!(usage.contains(word), "{usage}");
        }
    }
}
