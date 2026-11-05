use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_content_stop_phase_content_prefix,
    audit_content_stop_trajectory_content_prefix, content_continuation_loo_prediction,
    content_mapping_sha256, content_stop_density_gate_fires,
    content_stop_density_predictions_changed, content_stop_gate_fires, content_target_mapping,
    derive_content_continuation_rows, derive_content_readout, derive_content_stop_phase_rows,
    derive_content_stop_readout, derive_content_stop_rows, derive_content_stop_trajectory_rows,
    derive_turn_memory, evaluate_content_continuation_gate, evaluate_content_readout,
    evaluate_content_stop_density_gate, evaluate_content_stop_gate,
    evaluate_content_stop_phase_density_gate, evaluate_content_stop_trajectory_density_gate,
    fit_content_continuation_gate, fit_content_stop_density_gate, fit_content_stop_gate,
    shuffle_content_continuation_rows, shuffle_content_stop_rows, shuffle_content_targets,
    validate_corpus_manifest, verify_content_readout_serving, ContentContinuationRow,
    ContentContinuationTraining, ContentPromptReading, ContentReadout, ContentReadoutReading,
    ContentSignatureMode, ContentStopDensityGate, ContentStopGate, ContentStopRow,
    CpuConversationSample, CpuMaskedCorpus, MemoryWeightSample, CONTENT_MIN_TARGET_VALUES,
    CONTENT_MIN_TRAINING_ROWS,
};
use uor_r4_integer::stack::{
    read_memory_uorlut01, IntegerStackModel, STACK_CONTENT_READOUT_SCHEMA,
};

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    key_count: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    reply_cap: usize,
    stop_prototypes: usize,
    stop_density_neighbors: Option<usize>,
    stop_phase_neighbors: Option<usize>,
    stop_trajectory_neighbors: Option<usize>,
    stop_density_offset_radius: u16,
    stop_horizon_neighbors: Option<usize>,
    trajectory_readout: bool,
    seed: u64,
    select_only: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

#[derive(Clone)]
struct KeyGroup {
    key: Vec<u32>,
    training: Vec<CpuConversationSample>,
    evaluation: Vec<CpuConversationSample>,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-content-readout-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-content-readout-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--keys N] [--per-key N] [--key-tokens N] [--reply-turn N] \
     [--reply-cap N] [--stop-prototypes N] [--stop-density-neighbors N] \
     [--stop-density-offset-radius N] [--stop-phase-neighbors N] \
     [--stop-trajectory-neighbors N] \
     [--stop-horizon-neighbors N] \
     [--trajectory-readout] [--seed N] [--select-only]"
        .to_owned()
}

fn parse_args() -> Result<Config, String> {
    let mut values = std::env::args().skip(1);
    let artifact = PathBuf::from(values.next().ok_or_else(usage)?);
    let tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let out = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut config = Config {
        artifact,
        tokens,
        mask,
        manifest,
        out,
        key_count: 16,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        reply_cap: 16,
        stop_prototypes: 8,
        stop_density_neighbors: None,
        stop_phase_neighbors: None,
        stop_trajectory_neighbors: None,
        stop_density_offset_radius: 2,
        stop_horizon_neighbors: None,
        trajectory_readout: false,
        seed: 1,
        select_only: false,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
    };
    while let Some(flag) = values.next() {
        if flag == "--select-only" {
            config.select_only = true;
            continue;
        }
        if flag == "--trajectory-readout" {
            config.trajectory_readout = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--reply-cap" => config.reply_cap = number(&value)?,
            "--stop-prototypes" => config.stop_prototypes = number(&value)?,
            "--stop-density-neighbors" => {
                config.stop_density_neighbors = Some(number(&value)?);
            }
            "--stop-density-offset-radius" => {
                config.stop_density_offset_radius = number(&value)?;
            }
            "--stop-phase-neighbors" => {
                config.stop_phase_neighbors = Some(number(&value)?);
            }
            "--stop-trajectory-neighbors" => {
                config.stop_trajectory_neighbors = Some(number(&value)?);
            }
            "--stop-horizon-neighbors" => {
                config.stop_horizon_neighbors = Some(number(&value)?);
            }
            "--seed" => config.seed = number(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("artifact", &config.expected_artifact_sha256),
        ("tokens", &config.expected_tokens_sha256),
        ("mask", &config.expected_mask_sha256),
        ("manifest", &config.expected_manifest_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
    }
    if config.key_count == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
        || !(1..=64).contains(&config.reply_cap)
        || !(1..=1024).contains(&config.stop_prototypes)
        || config
            .stop_density_neighbors
            .is_some_and(|neighbors| !(1..=16).contains(&neighbors))
        || config
            .stop_phase_neighbors
            .is_some_and(|neighbors| !(1..=16).contains(&neighbors))
        || config
            .stop_trajectory_neighbors
            .is_some_and(|neighbors| !(1..=16).contains(&neighbors))
        || config
            .stop_horizon_neighbors
            .is_some_and(|neighbors| !(1..=16).contains(&neighbors))
        || config.stop_density_offset_radius > 16
    {
        return Err("group or reply dimensions are out of bounds".to_owned());
    }
    let density_modes = [
        config.stop_density_neighbors.is_some(),
        config.stop_phase_neighbors.is_some(),
        config.stop_trajectory_neighbors.is_some(),
    ];
    if density_modes.into_iter().filter(|enabled| *enabled).count() > 1 {
        return Err("--stop-density-neighbors, --stop-phase-neighbors and \
             --stop-trajectory-neighbors are exclusive"
            .to_owned());
    }
    Ok(config)
}

fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("not a number: {value}"))
}

fn hex(value: &str) -> Result<String, String> {
    let value = value.to_ascii_lowercase();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(format!("not a SHA-256: {value}"))
    }
}

fn run() -> Result<serde_json::Value, String> {
    let config = parse_args()?;
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact).map_err(|error| error.to_string())?;
    let tokens = std::fs::read(&config.tokens).map_err(|error| error.to_string())?;
    let mask = std::fs::read(&config.mask).map_err(|error| error.to_string())?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| error.to_string())?;
    for (name, bytes, expected) in [
        (
            "artifact",
            artifact.as_slice(),
            &config.expected_artifact_sha256,
        ),
        ("tokens", tokens.as_slice(), &config.expected_tokens_sha256),
        ("mask", mask.as_slice(), &config.expected_mask_sha256),
        (
            "manifest",
            manifest.as_slice(),
            &config.expected_manifest_sha256,
        ),
    ] {
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        if &actual != expected {
            return Err(format!("{name} SHA-256 differs: got {actual}"));
        }
    }
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    let shape = model.shape();
    if identity.vocab != shape.vocab || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the artifact".to_owned());
    }
    if shape.width != 256 {
        return Err("content readout requires width 256".to_owned());
    }
    if read_memory_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("the source artifact already carries memory".to_owned());
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&tokens, &mask, shape.vocab, shape.context, identity.bos)
            .map_err(|error| error.to_string())?;
    let examples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let groups = select_groups(&examples, &config);
    if groups.len() != config.key_count {
        return Err(format!(
            "selected {} keys, expected {}",
            groups.len(),
            config.key_count
        ));
    }
    let training_samples = groups
        .iter()
        .flat_map(|group| group.training.iter().map(memory_sample))
        .collect::<Vec<_>>();
    let evaluation_samples = groups
        .iter()
        .flat_map(|group| group.evaluation.iter().map(memory_sample))
        .collect::<Vec<_>>();
    if training_samples.len() != 98 || evaluation_samples.len() != 103 {
        return Err("selected content-readout split differs".to_owned());
    }
    let memory = derive_turn_memory(
        &training_samples,
        shape.width,
        config.key_tokens,
        shape.context,
        config.reply_turn,
    )
    .map_err(|error| error.to_string())?;
    let memory_artifact =
        add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    let sign_training = derive_content_readout(
        &memory_artifact,
        &training_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Sign,
    )
    .map_err(|error| error.to_string())?;
    let code_training = derive_content_readout(
        &memory_artifact,
        &training_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Code,
    )
    .map_err(|error| error.to_string())?;
    let trajectory_training = config
        .trajectory_readout
        .then(|| {
            derive_content_readout(
                &memory_artifact,
                &training_samples,
                config.reply_cap,
                true,
                ContentSignatureMode::Trajectory,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let trajectory_shuffled = trajectory_training
        .as_ref()
        .map(|training| shuffle_content_targets(&training.readout, config.seed))
        .transpose()
        .map_err(|error| error.to_string())?;
    let trajectory_changes = trajectory_training
        .as_ref()
        .zip(trajectory_shuffled.as_ref())
        .map(|(training, shuffled)| mapping_changes(&training.readout, shuffled))
        .transpose()?
        .unwrap_or(0);
    let trajectory_mapping = trajectory_training
        .as_ref()
        .zip(trajectory_shuffled.as_ref())
        .map(|(training, shuffled)| mapping_json(&training.readout, shuffled, trajectory_changes))
        .transpose()?;
    let trajectory_shuffled_sha = trajectory_shuffled.as_ref().map(readout_sha).transpose()?;
    let code_stop_training =
        derive_content_stop_readout(&memory_artifact, &training_samples, config.reply_cap, true)
            .map_err(|error| error.to_string())?;
    let gate_rows = derive_content_stop_rows(
        &memory_artifact,
        &training_samples,
        config.reply_cap,
        true,
        &code_training.readout,
    )
    .map_err(|error| error.to_string())?;
    let gate_shuffled_rows =
        shuffle_content_stop_rows(&gate_rows, config.seed).map_err(|error| error.to_string())?;
    let phase_gate_rows = config
        .stop_phase_neighbors
        .map(|_| {
            derive_content_stop_phase_rows(
                &memory_artifact,
                &training_samples,
                config.reply_cap,
                true,
                &code_training.readout,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let trajectory_gate_rows = config
        .stop_trajectory_neighbors
        .map(|_| {
            derive_content_stop_trajectory_rows(
                &memory_artifact,
                &training_samples,
                config.reply_cap,
                true,
                &code_training.readout,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let density_rows = trajectory_gate_rows
        .as_ref()
        .or(phase_gate_rows.as_ref())
        .unwrap_or(&gate_rows);
    let density_shuffled_rows = if config.stop_phase_neighbors.is_some()
        || config.stop_trajectory_neighbors.is_some()
    {
        shuffle_content_stop_rows(density_rows, config.seed).map_err(|error| error.to_string())?
    } else {
        gate_shuffled_rows.clone()
    };
    let stop_density_neighbors = config
        .stop_trajectory_neighbors
        .or(config.stop_phase_neighbors)
        .or(config.stop_density_neighbors);
    let gate_training = fit_content_stop_gate(&gate_rows, config.stop_prototypes)
        .map_err(|error| error.to_string())?;
    let gate_shuffled_training = fit_content_stop_gate(&gate_shuffled_rows, config.stop_prototypes)
        .map_err(|error| error.to_string())?;
    let gate_sha256 = json_sha(&gate_training.gate)?;
    let gate_shuffled_sha256 = json_sha(&gate_shuffled_training.gate)?;
    let gate_density_training = stop_density_neighbors
        .map(|neighbors| {
            fit_content_stop_density_gate(
                density_rows,
                neighbors,
                config.stop_density_offset_radius,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let gate_density_shuffled_training = stop_density_neighbors
        .map(|neighbors| {
            fit_content_stop_density_gate(
                &density_shuffled_rows,
                neighbors,
                config.stop_density_offset_radius,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let gate_density_sha256 = gate_density_training
        .as_ref()
        .map(|training| json_sha(&training.gate))
        .transpose()?;
    let gate_density_shuffled_sha256 = gate_density_shuffled_training
        .as_ref()
        .map(|training| json_sha(&training.gate))
        .transpose()?;
    let gate_density_prediction_changes = gate_density_training
        .as_ref()
        .zip(gate_density_shuffled_training.as_ref())
        .map(|(training, shuffled)| {
            content_stop_density_predictions_changed(
                density_rows,
                &density_shuffled_rows,
                &training.gate,
                &shuffled.gate,
            )
            .map_err(|error| error.to_string())
        })
        .transpose()?;
    let training_trajectory_content_audit = if config.stop_trajectory_neighbors.is_some() {
        Some(
            audit_content_stop_trajectory_content_prefix(
                &memory_artifact,
                &training_samples,
                config.reply_cap,
                true,
                &code_training.readout,
                &gate_density_training
                    .as_ref()
                    .ok_or_else(|| "trajectory density training is missing".to_owned())?
                    .gate,
            )
            .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    let horizon_rows = config
        .stop_horizon_neighbors
        .map(|_| {
            derive_content_continuation_rows(
                &memory_artifact,
                &training_samples,
                config.reply_cap,
                true,
                &code_training.readout,
            )
        })
        .transpose()
        .map_err(|error| error.to_string())?;
    let horizon_shuffled_rows = horizon_rows
        .as_ref()
        .map(|rows| shuffle_content_continuation_rows(rows, config.seed))
        .transpose()
        .map_err(|error| error.to_string())?;
    let horizon_training = config
        .stop_horizon_neighbors
        .map(|neighbors| {
            fit_content_continuation_gate(
                horizon_rows
                    .as_ref()
                    .ok_or_else(|| "continuation rows are missing".to_owned())?,
                neighbors,
            )
            .map_err(|error| error.to_string())
        })
        .transpose()?;
    let horizon_shuffled_training = config
        .stop_horizon_neighbors
        .map(|neighbors| {
            fit_content_continuation_gate(
                horizon_shuffled_rows
                    .as_ref()
                    .ok_or_else(|| "shuffled continuation rows are missing".to_owned())?,
                neighbors,
            )
            .map_err(|error| error.to_string())
        })
        .transpose()?;
    let horizon_sha256 = horizon_training
        .as_ref()
        .map(|training| json_sha(&training.gate))
        .transpose()?;
    let horizon_shuffled_sha256 = horizon_shuffled_training
        .as_ref()
        .map(|training| json_sha(&training.gate))
        .transpose()?;
    let horizon_prediction_changes = match (
        horizon_rows.as_ref(),
        horizon_training.as_ref(),
        horizon_shuffled_rows.as_ref(),
        horizon_shuffled_training.as_ref(),
    ) {
        (Some(rows), Some(training), Some(shuffled_rows), Some(shuffled)) => {
            Some(horizon_predictions_changed(
                rows,
                training.gate.neighbors,
                shuffled_rows,
                shuffled.gate.neighbors,
            )?)
        }
        _ => None,
    };
    let sign_shuffled = shuffle_content_targets(&sign_training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let code_shuffled = shuffle_content_targets(&code_training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let code_stop_shuffled = shuffle_content_targets(&code_stop_training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let sign_changes = mapping_changes(&sign_training.readout, &sign_shuffled)?;
    let code_changes = mapping_changes(&code_training.readout, &code_shuffled)?;
    let code_stop_changes = mapping_changes(&code_stop_training.readout, &code_stop_shuffled)?;
    let sign_mapping = mapping_json(&sign_training.readout, &sign_shuffled, sign_changes)?;
    let code_mapping = mapping_json(&code_training.readout, &code_shuffled, code_changes)?;
    let code_stop_mapping = mapping_json(
        &code_stop_training.readout,
        &code_stop_shuffled,
        code_stop_changes,
    )?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-content-readout-selection/2",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "modes": {
                "sign": {
                    "training": sign_training,
                    "shuffled_readout_sha256": readout_sha(&sign_shuffled)?,
                    "mapping": sign_mapping,
                },
                "code": {
                    "training": code_training,
                    "shuffled_readout_sha256": readout_sha(&code_shuffled)?,
                    "mapping": code_mapping,
                },
                "trajectory": trajectory_training.as_ref().map(|training| {
                    serde_json::json!({
                        "training": training,
                        "shuffled_readout_sha256": trajectory_shuffled_sha,
                        "mapping": trajectory_mapping,
                    })
                }),
                "code_stop": {
                    "training": code_stop_training,
                    "shuffled_readout_sha256": readout_sha(&code_stop_shuffled)?,
                    "mapping": code_stop_mapping,
                },
                "gate": {
                    "rows": gate_rows.len(),
                    "stop_prototypes": config.stop_prototypes,
                    "training": gate_training,
                    "shuffled_training": gate_shuffled_training,
                    "true_sha256": gate_sha256,
                    "shuffled_sha256": gate_shuffled_sha256,
                },
                "gate_density": gate_density_training.as_ref().map(|training| {
                    serde_json::json!({
                        "rows": density_rows.len(),
                        "signature_mode": if config.stop_trajectory_neighbors.is_some() {
                            "current_code_phase_transition"
                        } else if config.stop_phase_neighbors.is_some() {
                            "phase_transition"
                        } else {
                            "current_code"
                        },
                        "max_neighbors": stop_density_neighbors,
                        "offset_radius": config.stop_density_offset_radius,
                        "training": training,
                        "shuffled_training": gate_density_shuffled_training,
                        "true_sha256": gate_density_sha256,
                        "shuffled_sha256": gate_density_shuffled_sha256,
                        "predictions_changed": gate_density_prediction_changes,
                        "training_content_audit": training_trajectory_content_audit,
                    })
                }),
                "gate_horizon": horizon_training.as_ref().map(|training| {
                    serde_json::json!({
                        "rows": horizon_rows.as_ref().map(Vec::len),
                        "max_neighbors": config.stop_horizon_neighbors,
                        "training": training,
                        "shuffled_training": horizon_shuffled_training,
                        "true_sha256": horizon_sha256,
                        "shuffled_sha256": horizon_shuffled_sha256,
                        "predictions_changed": horizon_prediction_changes,
                    })
                }),
            },
        }));
    }
    create_fresh_output(&config.out)?;
    let serving_artifact_sha256 = uor_r4_discrete::corpus::sha256(&memory_artifact);
    let serving_sidecar = serde_json::json!({
        "schema": STACK_CONTENT_READOUT_SCHEMA,
        "artifact_sha256": serving_artifact_sha256,
        "readout": &code_training.readout,
    });
    let serving_sidecar_bytes =
        serde_json::to_vec_pretty(&serving_sidecar).map_err(|error| error.to_string())?;
    let serving_sidecar_sha256 = uor_r4_discrete::corpus::sha256(&serving_sidecar_bytes);
    std::fs::write(config.out.join("serving.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("content-readout-sidecar.json"),
        &serving_sidecar_bytes,
    )
    .map_err(|error| error.to_string())?;
    let (writer, writer_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Sign,
        None,
    )
    .map_err(|error| error.to_string())?;
    let (sign_arm, sign_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Sign,
        Some(&sign_training.readout),
    )
    .map_err(|error| error.to_string())?;
    let (sign_shuffled_arm, sign_shuffled_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Sign,
        Some(&sign_shuffled),
    )
    .map_err(|error| error.to_string())?;
    let (code_arm, code_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Code,
        Some(&code_training.readout),
    )
    .map_err(|error| error.to_string())?;
    let (code_shuffled_arm, code_shuffled_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Code,
        Some(&code_shuffled),
    )
    .map_err(|error| error.to_string())?;
    let trajectory_arms =
        if let (Some(training), Some(shuffled)) = (&trajectory_training, &trajectory_shuffled) {
            let (arm, prompts) = evaluate_content_readout(
                &memory_artifact,
                &evaluation_samples,
                config.reply_cap,
                true,
                ContentSignatureMode::Trajectory,
                Some(&training.readout),
            )
            .map_err(|error| error.to_string())?;
            let (shuffled_arm, shuffled_prompts) = evaluate_content_readout(
                &memory_artifact,
                &evaluation_samples,
                config.reply_cap,
                true,
                ContentSignatureMode::Trajectory,
                Some(shuffled),
            )
            .map_err(|error| error.to_string())?;
            Some((arm, prompts, shuffled_arm, shuffled_prompts))
        } else {
            None
        };
    let (code_stop_arm, code_stop_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Code,
        Some(&code_stop_training.readout),
    )
    .map_err(|error| error.to_string())?;
    let (code_stop_shuffled_arm, code_stop_shuffled_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        ContentSignatureMode::Code,
        Some(&code_stop_shuffled),
    )
    .map_err(|error| error.to_string())?;
    let (gate_arm, gate_prompts) = evaluate_content_stop_gate(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        &code_training.readout,
        &gate_training.gate,
    )
    .map_err(|error| error.to_string())?;
    let (gate_shuffled_arm, gate_shuffled_prompts) = evaluate_content_stop_gate(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        &code_training.readout,
        &gate_shuffled_training.gate,
    )
    .map_err(|error| error.to_string())?;
    let (gate_density_arms, gate_density_bar) =
        if let (Some(density_training), Some(density_shuffled_training)) =
            (&gate_density_training, &gate_density_shuffled_training)
        {
            let evaluate_density = |gate| {
                if config.stop_trajectory_neighbors.is_some() {
                    evaluate_content_stop_trajectory_density_gate(
                        &memory_artifact,
                        &evaluation_samples,
                        config.reply_cap,
                        true,
                        &code_training.readout,
                        gate,
                    )
                } else if config.stop_phase_neighbors.is_some() {
                    evaluate_content_stop_phase_density_gate(
                        &memory_artifact,
                        &evaluation_samples,
                        config.reply_cap,
                        true,
                        &code_training.readout,
                        gate,
                    )
                } else {
                    evaluate_content_stop_density_gate(
                        &memory_artifact,
                        &evaluation_samples,
                        config.reply_cap,
                        true,
                        &code_training.readout,
                        gate,
                    )
                }
            };
            let (density_arm, density_prompts) =
                evaluate_density(&density_training.gate).map_err(|error| error.to_string())?;
            let (density_shuffled_arm, density_shuffled_prompts) =
                evaluate_density(&density_shuffled_training.gate)
                    .map_err(|error| error.to_string())?;
            let phase_content_audit = if config.stop_trajectory_neighbors.is_some() {
                Some(
                    audit_content_stop_trajectory_content_prefix(
                        &memory_artifact,
                        &evaluation_samples,
                        config.reply_cap,
                        true,
                        &code_training.readout,
                        &density_training.gate,
                    )
                    .map_err(|error| error.to_string())?,
                )
            } else if config.stop_phase_neighbors.is_some() {
                Some(
                    audit_content_stop_phase_content_prefix(
                        &memory_artifact,
                        &evaluation_samples,
                        config.reply_cap,
                        true,
                        &code_training.readout,
                        &density_training.gate,
                    )
                    .map_err(|error| error.to_string())?,
                )
            } else {
                None
            };
            let vs_code_top1 = paired(&density_prompts, &code_prompts, |row| row.top1)?;
            let vs_code_depth = paired(&density_prompts, &code_prompts, |row| {
                row.matched_prefix_depth as u64
            })?;
            let vs_code_bos = paired_two_sqrt(&density_prompts, &code_prompts, |row| {
                u64::from(row.stop == "bos")
            })?;
            let vs_shuffled_bos =
                paired_two_sqrt(&density_prompts, &density_shuffled_prompts, |row| {
                    u64::from(row.stop == "bos")
                })?;
            let non_inferior = non_inferior(
                vs_code_top1["candidate_sum"]
                    .as_u64()
                    .ok_or_else(|| "density top-1 sum differs".to_owned())?,
                vs_code_top1["control_sum"]
                    .as_u64()
                    .ok_or_else(|| "code top-1 sum differs".to_owned())?,
                vs_code_depth["candidate_sum"]
                    .as_u64()
                    .ok_or_else(|| "density depth sum differs".to_owned())?,
                vs_code_depth["control_sum"]
                    .as_u64()
                    .ok_or_else(|| "code depth sum differs".to_owned())?,
            );
            let lengths = bos_length_counts(&density_prompts);
            let terminal_comparison = paired_terminal_density_rows(
                density_rows,
                &density_training.gate,
                &density_shuffled_training.gate,
            )?;
            let hashes_differ = gate_density_sha256 != gate_density_shuffled_sha256;
            let predictions_changed = gate_density_prediction_changes.unwrap_or(0);
            let rows_match = density_training.gate.terminal_rows.len() as u64
                == density_training.terminal_rows
                && density_training.gate.nonterminal_rows.len() as u64
                    == density_training.nonterminal_rows;
            let structural = density_training.terminal_rows >= 84
                && density_training.nonterminal_rows >= 1_484
                && density_training.terminal_stops >= 26
                && density_training.nonterminal_stops <= density_training.nonterminal_stop_limit
                && hashes_differ
                && (config.stop_phase_neighbors.is_none()
                    && config.stop_trajectory_neighbors.is_none()
                    || predictions_changed > 0)
                && rows_match
                && phase_content_audit
                    .as_ref()
                    .is_none_or(|audit| audit.decisions > 0 && audit.mismatches == 0);
            let exact_reference_floor = if config.stop_phase_neighbors.is_some()
                || config.stop_trajectory_neighbors.is_some()
            {
                9
            } else {
                8
            };
            let passes = structural
                && non_inferior
                && density_arm.ungated_token_mismatches == 0
                && density_arm.bos_stopped >= 9
                && lengths.within_reference() >= 8
                && density_arm.exact_reference >= exact_reference_floor
                && vs_code_bos["passes"].as_bool() == Some(true)
                && vs_shuffled_bos["passes"].as_bool() == Some(true)
                && terminal_comparison["passes"].as_bool() == Some(true);
            (
                Some(serde_json::json!({
                    "reading": density_arm,
                    "prompts": density_prompts,
                    "shuffled_reading": density_shuffled_arm,
                    "shuffled_prompts": density_shuffled_prompts,
                })),
                Some(serde_json::json!({
                    "training": density_training,
                    "shuffled_training": density_shuffled_training,
                    "true_sha256": gate_density_sha256.as_ref(),
                    "shuffled_sha256": gate_density_shuffled_sha256.as_ref(),
                    "structural_pass": structural,
                    "hashes_differ": hashes_differ,
                    "predictions_changed": predictions_changed,
                    "gate_rows_match_training": rows_match,
                    "phase_content_audit": phase_content_audit,
                    "terminal_rows_vs_shuffled": terminal_comparison,
                    "exact_reference": density_arm.exact_reference,
                    "bos_complete": density_arm.bos_stopped,
                    "bos_within_reference": lengths.within_reference(),
                    "bos_at_reference_length": lengths.at_reference_length,
                    "bos_shorter": lengths.shorter,
                    "bos_longer": lengths.longer,
                    "gate_fires": density_arm.gate_fires,
                    "ungated_token_mismatches": density_arm.ungated_token_mismatches,
                    "non_inferior_to_code": non_inferior,
                    "vs_code_top1": vs_code_top1,
                    "vs_code_depth": vs_code_depth,
                    "vs_code_bos": vs_code_bos,
                    "vs_shuffled_bos": vs_shuffled_bos,
                    "passes": passes,
                })),
            )
        } else {
            (None, None)
        };
    let (gate_horizon_arms, gate_horizon_bar) =
        if let (Some(horizon_rows), Some(horizon_training), Some(horizon_shuffled_training)) = (
            horizon_rows.as_ref(),
            horizon_training.as_ref(),
            horizon_shuffled_training.as_ref(),
        ) {
            let (horizon_arm, horizon_prompts) = evaluate_content_continuation_gate(
                &memory_artifact,
                &evaluation_samples,
                config.reply_cap,
                true,
                &code_training.readout,
                &horizon_training.gate,
            )
            .map_err(|error| error.to_string())?;
            let (horizon_shuffled_arm, horizon_shuffled_prompts) =
                evaluate_content_continuation_gate(
                    &memory_artifact,
                    &evaluation_samples,
                    config.reply_cap,
                    true,
                    &code_training.readout,
                    &horizon_shuffled_training.gate,
                )
                .map_err(|error| error.to_string())?;
            let vs_code_top1 = paired(&horizon_prompts, &code_prompts, |row| row.top1)?;
            let vs_code_depth = paired(&horizon_prompts, &code_prompts, |row| {
                row.matched_prefix_depth as u64
            })?;
            let vs_code_bos = paired(&horizon_prompts, &code_prompts, |row| {
                u64::from(row.stop == "bos")
            })?;
            let vs_shuffled_bos = paired(&horizon_prompts, &horizon_shuffled_prompts, |row| {
                u64::from(row.stop == "bos")
            })?;
            let non_inferior = non_inferior(
                vs_code_top1["candidate_sum"]
                    .as_u64()
                    .ok_or_else(|| "horizon top-1 sum differs".to_owned())?,
                vs_code_top1["control_sum"]
                    .as_u64()
                    .ok_or_else(|| "code top-1 sum differs".to_owned())?,
                vs_code_depth["candidate_sum"]
                    .as_u64()
                    .ok_or_else(|| "horizon depth sum differs".to_owned())?,
                vs_code_depth["control_sum"]
                    .as_u64()
                    .ok_or_else(|| "code depth sum differs".to_owned())?,
            );
            let lengths = bos_length_counts(&horizon_prompts);
            let terminal_comparison = paired_terminal_continuation_rows(
                horizon_rows,
                horizon_training,
                horizon_shuffled_rows
                    .as_ref()
                    .ok_or_else(|| "shuffled continuation rows are missing".to_owned())?,
                horizon_shuffled_training,
            )?;
            let prediction_changes = horizon_prediction_changes.unwrap_or(0);
            let hashes_differ = horizon_sha256 != horizon_shuffled_sha256;
            let rows_match = horizon_training.gate.rows.len() as u64 == horizon_training.rows;
            let horizon_error_better =
                horizon_training.horizon_abs_error < horizon_shuffled_training.horizon_abs_error;
            let structural = horizon_training.rows == 1_568
                && horizon_training.terminal_rows == 84
                && horizon_training.nonterminal_rows == 1_484
                && horizon_training.terminal_stops >= 26
                && horizon_training.nonterminal_stops <= horizon_training.nonterminal_stop_limit
                && horizon_error_better
                && hashes_differ
                && prediction_changes > 0
                && rows_match;
            let passes = structural
                && non_inferior
                && horizon_arm.ungated_token_mismatches == 0
                && horizon_arm.bos_stopped >= 9
                && lengths.within_reference() >= 8
                && horizon_arm.exact_reference >= 8
                && vs_code_bos["passes"].as_bool() == Some(true)
                && vs_shuffled_bos["passes"].as_bool() == Some(true)
                && terminal_comparison["passes"].as_bool() == Some(true);
            (
                Some(serde_json::json!({
                    "reading": horizon_arm,
                    "prompts": horizon_prompts,
                    "shuffled_reading": horizon_shuffled_arm,
                    "shuffled_prompts": horizon_shuffled_prompts,
                })),
                Some(serde_json::json!({
                    "training": horizon_training,
                    "shuffled_training": horizon_shuffled_training,
                    "true_sha256": horizon_sha256.as_ref(),
                    "shuffled_sha256": horizon_shuffled_sha256.as_ref(),
                    "prediction_changes": prediction_changes,
                    "structural_pass": structural,
                    "hashes_differ": hashes_differ,
                    "gate_rows_match_training": rows_match,
                    "horizon_error_better": horizon_error_better,
                    "terminal_rows_vs_shuffled": terminal_comparison,
                    "exact_reference": horizon_arm.exact_reference,
                    "bos_complete": horizon_arm.bos_stopped,
                    "bos_within_reference": lengths.within_reference(),
                    "bos_at_reference_length": lengths.at_reference_length,
                    "bos_shorter": lengths.shorter,
                    "bos_longer": lengths.longer,
                    "gate_fires": horizon_arm.gate_fires,
                    "ungated_token_mismatches": horizon_arm.ungated_token_mismatches,
                    "non_inferior_to_code": non_inferior,
                    "vs_code_top1": vs_code_top1,
                    "vs_code_depth": vs_code_depth,
                    "vs_code_bos": vs_code_bos,
                    "vs_shuffled_bos": vs_shuffled_bos,
                    "passes": passes,
                })),
            )
        } else {
            (None, None)
        };
    let trajectory_bar = trajectory_arms
        .as_ref()
        .map(
            |(arm, prompts, shuffled_arm, shuffled_prompts)| -> Result<serde_json::Value, String> {
                let mapping = trajectory_mapping
                    .as_ref()
                    .ok_or_else(|| "trajectory mapping is missing".to_owned())?;
                let training = trajectory_training
                    .as_ref()
                    .ok_or_else(|| "trajectory training is missing".to_owned())?;
                let vs_code_top1 = paired(prompts, &code_prompts, |row| row.top1)?;
                let vs_code_depth = paired_two_sqrt(prompts, &code_prompts, |row| {
                    row.matched_prefix_depth as u64
                })?;
                let vs_shuffled_top1 = paired(prompts, shuffled_prompts, |row| row.top1)?;
                let structural = mode_structural_pass(training, mapping);
                let fallback_accounting = readout_fallback_accounting(arm);
                let passes = structural
                    && fallback_accounting
                    && non_inferior(
                        arm.top1,
                        code_arm.top1,
                        arm.matched_prefix_depth_sum,
                        code_arm.matched_prefix_depth_sum,
                    )
                    && vs_code_top1["passes"].as_bool() == Some(true)
                    && vs_code_depth["passes"].as_bool() == Some(true)
                    && vs_shuffled_top1["passes"].as_bool() == Some(true);
                Ok(serde_json::json!({
                "reading": arm,
                "prompts": prompts,
                "shuffled_reading": shuffled_arm,
                "shuffled_prompts": shuffled_prompts,
                "structural_pass": structural,
                "readout_fallback_accounting": fallback_accounting,
                "vs_code_top1": vs_code_top1,
                "vs_code_depth": vs_code_depth,
                "vs_shuffled_top1": vs_shuffled_top1,
                "passes": passes,
                }))
            },
        )
        .transpose()?;
    let serving_parity = verify_content_readout_serving(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        &code_training.readout,
        &serving_sidecar_bytes,
    )
    .map_err(|error| error.to_string())?;
    let sign_vs_writer_top1 = paired(&sign_prompts, &writer_prompts, |row| row.top1)?;
    let sign_vs_shuffled_top1 = paired(&sign_prompts, &sign_shuffled_prompts, |row| row.top1)?;
    let code_vs_writer_top1 = paired(&code_prompts, &writer_prompts, |row| row.top1)?;
    let code_vs_shuffled_top1 = paired(&code_prompts, &code_shuffled_prompts, |row| row.top1)?;
    let code_vs_sign_top1 = paired(&code_prompts, &sign_prompts, |row| row.top1)?;
    let sign_vs_writer_depth = paired(&sign_prompts, &writer_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let sign_vs_shuffled_depth = paired(&sign_prompts, &sign_shuffled_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let code_vs_writer_depth = paired(&code_prompts, &writer_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let code_vs_shuffled_depth = paired(&code_prompts, &code_shuffled_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let code_vs_sign_depth = paired(&code_prompts, &sign_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let code_stop_vs_code_top1 = paired(&code_stop_prompts, &code_prompts, |row| row.top1)?;
    let code_stop_vs_code_depth = paired(&code_stop_prompts, &code_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let code_stop_vs_code_bos = paired(&code_stop_prompts, &code_prompts, |row| {
        u64::from(row.stop == "bos")
    })?;
    let code_stop_vs_shuffled_bos =
        paired(&code_stop_prompts, &code_stop_shuffled_prompts, |row| {
            u64::from(row.stop == "bos")
        })?;
    let code_stop_bos_lengths = bos_length_counts(&code_stop_prompts);
    let code_stop_non_inferior = non_inferior(
        code_stop_vs_code_top1["candidate_sum"]
            .as_u64()
            .ok_or_else(|| "code-stop top-1 sum differs".to_owned())?,
        code_stop_vs_code_top1["control_sum"]
            .as_u64()
            .ok_or_else(|| "code top-1 sum differs".to_owned())?,
        code_stop_vs_code_depth["candidate_sum"]
            .as_u64()
            .ok_or_else(|| "code-stop depth sum differs".to_owned())?,
        code_stop_vs_code_depth["control_sum"]
            .as_u64()
            .ok_or_else(|| "code depth sum differs".to_owned())?,
    );
    let gate_vs_code_top1 = paired(&gate_prompts, &code_prompts, |row| row.top1)?;
    let gate_vs_code_depth = paired(&gate_prompts, &code_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let gate_vs_code_bos = paired(&gate_prompts, &code_prompts, |row| {
        u64::from(row.stop == "bos")
    })?;
    let gate_vs_shuffled_bos = paired(&gate_prompts, &gate_shuffled_prompts, |row| {
        u64::from(row.stop == "bos")
    })?;
    let gate_non_inferior = non_inferior(
        gate_vs_code_top1["candidate_sum"]
            .as_u64()
            .ok_or_else(|| "gate top-1 sum differs".to_owned())?,
        gate_vs_code_top1["control_sum"]
            .as_u64()
            .ok_or_else(|| "code top-1 sum differs".to_owned())?,
        gate_vs_code_depth["candidate_sum"]
            .as_u64()
            .ok_or_else(|| "gate depth sum differs".to_owned())?,
        gate_vs_code_depth["control_sum"]
            .as_u64()
            .ok_or_else(|| "code depth sum differs".to_owned())?,
    );
    let gate_bos_lengths = bos_length_counts(&gate_prompts);
    let gate_terminal_comparison = paired_terminal_rows(
        &gate_rows,
        &gate_training.gate,
        &gate_shuffled_training.gate,
    )?;
    let code_non_inferior = code_vs_sign_top1["candidate_sum"].as_u64()
        >= code_vs_sign_top1["control_sum"].as_u64()
        && code_vs_sign_depth["candidate_sum"].as_u64()
            >= code_vs_sign_depth["control_sum"].as_u64()
        && (code_vs_sign_top1["candidate_sum"] != code_vs_sign_top1["control_sum"]
            || code_vs_sign_depth["candidate_sum"] != code_vs_sign_depth["control_sum"]);
    let sign_structural = mode_structural_pass(&sign_training, &sign_mapping);
    let code_structural = mode_structural_pass(&code_training, &code_mapping);
    let code_stop_structural = mode_structural_pass(&code_stop_training, &code_stop_mapping);
    let gate_structural = gate_training.terminal_rows >= 84
        && gate_training.nonterminal_rows > 0
        && gate_training.terminal_stops >= 26
        && gate_training.nonterminal_stops <= gate_training.nonterminal_stop_limit;
    let code_exposure = code_arm.readout_decisions > 0 && code_arm.writer_fallbacks > 0;
    let mut bar = serde_json::json!({
        "sign": {
            "structural_pass": sign_structural,
            "vs_writer_top1": sign_vs_writer_top1,
            "vs_shuffled_top1": sign_vs_shuffled_top1,
            "vs_writer_depth": sign_vs_writer_depth,
            "vs_shuffled_depth": sign_vs_shuffled_depth,
        },
        "code": {
            "structural_pass": code_structural,
            "exposure_pass": code_exposure,
            "readout_decisions": code_arm.readout_decisions,
            "writer_fallbacks": code_arm.writer_fallbacks,
            "vs_writer_top1": code_vs_writer_top1,
            "vs_shuffled_top1": code_vs_shuffled_top1,
            "vs_writer_depth": code_vs_writer_depth,
            "vs_shuffled_depth": code_vs_shuffled_depth,
            "vs_sign_top1": code_vs_sign_top1,
            "vs_sign_depth": code_vs_sign_depth,
            "non_inferior_to_sign": code_non_inferior,
            "loo_above_sign": code_training.leave_one_out_top1
                > sign_training.leave_one_out_top1,
        },
        "code_stop": {
            "structural_pass": code_stop_structural,
            "exact_reference": code_stop_arm.exact_reference,
            "bos_complete": code_stop_arm.bos_stopped,
            "bos_within_reference": code_stop_bos_lengths.within_reference(),
            "bos_at_reference_length": code_stop_bos_lengths.at_reference_length,
            "bos_shorter": code_stop_bos_lengths.shorter,
            "bos_longer": code_stop_bos_lengths.longer,
            "non_inferior_to_code": code_stop_non_inferior,
            "vs_code_top1": code_stop_vs_code_top1,
            "vs_code_depth": code_stop_vs_code_depth,
            "vs_code_bos": code_stop_vs_code_bos,
            "vs_shuffled_bos": code_stop_vs_shuffled_bos,
        },
        "serving": {
            "artifact_sha256": serving_artifact_sha256,
            "sidecar_sha256": serving_sidecar_sha256,
            "rows": code_training.readout.rows.len(),
            "target_values": code_training.target_values,
            "max_distance": code_training.readout.max_distance,
            "parity": serving_parity,
            "passes": serving_parity.prompts == evaluation_samples.len() as u64
                && serving_parity.decisions > 0
                && serving_parity.readout_fires > 0
                && serving_parity.token_mismatches == 0
                && serving_parity.decision_mismatches == 0
                && serving_parity.cpu_decision_sha256 == serving_parity.serving_decision_sha256
                && code_training.rows >= CONTENT_MIN_TRAINING_ROWS as u64
                && code_training.target_values >= CONTENT_MIN_TARGET_VALUES as u64
                && code_training.readout.max_distance
                    <= uor_r4_cpu_train::CONTENT_CODE_SIGNATURE_BITS as u32,
        },
        "gate": {
            "training": gate_training,
            "shuffled_training": gate_shuffled_training,
            "true_sha256": gate_sha256,
            "shuffled_sha256": gate_shuffled_sha256,
            "structural_pass": gate_structural,
            "terminal_rows_vs_shuffled": gate_terminal_comparison,
            "exact_reference": gate_arm.exact_reference,
            "bos_complete": gate_arm.bos_stopped,
            "bos_within_reference": gate_bos_lengths.within_reference(),
            "bos_at_reference_length": gate_bos_lengths.at_reference_length,
            "bos_shorter": gate_bos_lengths.shorter,
            "bos_longer": gate_bos_lengths.longer,
            "gate_fires": gate_arm.gate_fires,
            "ungated_token_mismatches": gate_arm.ungated_token_mismatches,
            "non_inferior_to_code": gate_non_inferior,
            "vs_code_top1": gate_vs_code_top1,
            "vs_code_depth": gate_vs_code_depth,
            "vs_code_bos": gate_vs_code_bos,
            "vs_shuffled_bos": gate_vs_shuffled_bos,
        },
        "passes": sign_structural
            && code_structural
            && gate_structural
            && code_exposure
            && code_non_inferior
            && gate_non_inferior
            && code_training.leave_one_out_top1 > sign_training.leave_one_out_top1
            && code_vs_writer_top1["passes"].as_bool() == Some(true)
            && code_vs_shuffled_top1["passes"].as_bool() == Some(true)
            && code_vs_writer_depth["passes"].as_bool() == Some(true)
            && code_vs_shuffled_depth["passes"].as_bool() == Some(true)
            && gate_terminal_comparison["passes"].as_bool() == Some(true)
            && gate_arm.bos_stopped >= 9
            && gate_bos_lengths.within_reference() >= 8
            && gate_arm.exact_reference >= 8
            && gate_arm.ungated_token_mismatches == 0
            && gate_vs_code_bos["passes"].as_bool() == Some(true)
            && gate_vs_shuffled_bos["passes"].as_bool() == Some(true),
    });
    if let Some(density) = gate_density_bar.as_ref() {
        bar["gate_density"] = density.clone();
    }
    if let Some(horizon) = gate_horizon_bar.as_ref() {
        bar["gate_horizon"] = horizon.clone();
    }
    if let Some(trajectory) = trajectory_bar.as_ref() {
        bar["trajectory"] = trajectory.clone();
    }
    let sign_mapping_sha256 = mapping_json(&sign_training.readout, &sign_shuffled, sign_changes)?;
    let code_mapping_sha256 = mapping_json(&code_training.readout, &code_shuffled, code_changes)?;
    std::fs::write(
        config.out.join("sign-true-readout.json"),
        serde_json::to_vec_pretty(&sign_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("sign-shuffled-readout.json"),
        serde_json::to_vec_pretty(&sign_shuffled).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("code-true-readout.json"),
        serde_json::to_vec_pretty(&code_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("code-shuffled-readout.json"),
        serde_json::to_vec_pretty(&code_shuffled).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if let (Some(training), Some(shuffled)) = (&trajectory_training, &trajectory_shuffled) {
        std::fs::write(
            config.out.join("trajectory-true-readout.json"),
            serde_json::to_vec_pretty(training).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            config.out.join("trajectory-shuffled-readout.json"),
            serde_json::to_vec_pretty(shuffled).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }
    std::fs::write(
        config.out.join("code-stop-readout.json"),
        serde_json::to_vec_pretty(&code_stop_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("code-stop-shuffled-readout.json"),
        serde_json::to_vec_pretty(&code_stop_shuffled).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("content-stop-rows.json"),
        serde_json::to_vec_pretty(&gate_rows).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("content-stop-gate.json"),
        serde_json::to_vec_pretty(&gate_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("content-stop-gate-shuffled.json"),
        serde_json::to_vec_pretty(&gate_shuffled_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if let (Some(training), Some(shuffled)) =
        (&gate_density_training, &gate_density_shuffled_training)
    {
        std::fs::write(
            config.out.join("content-stop-density-gate.json"),
            serde_json::to_vec_pretty(training).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            config.out.join("content-stop-density-gate-shuffled.json"),
            serde_json::to_vec_pretty(shuffled).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }
    if let (Some(training), Some(shuffled)) = (&horizon_training, &horizon_shuffled_training) {
        std::fs::write(
            config.out.join("content-continuation-rows.json"),
            serde_json::to_vec_pretty(
                horizon_rows
                    .as_ref()
                    .ok_or_else(|| "continuation rows are missing".to_owned())?,
            )
            .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            config.out.join("content-continuation-gate.json"),
            serde_json::to_vec_pretty(training).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            config.out.join("content-continuation-gate-shuffled.json"),
            serde_json::to_vec_pretty(shuffled).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-content-readout-run/2",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "CPU generated-state content, trajectory and termination research with exact integer-serving parity; no language claim",
        "inputs": {
            "artifact": config.artifact,
            "artifact_sha256": config.expected_artifact_sha256,
            "tokens": config.tokens,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask": config.mask,
            "mask_sha256": config.expected_mask_sha256,
            "manifest": config.manifest,
            "manifest_sha256": config.expected_manifest_sha256,
        },
        "selection": selection_json(&groups, &training_samples, &evaluation_samples),
        "training": {
            "sign": sign_training,
            "code": code_training,
            "trajectory": trajectory_training.as_ref(),
            "code_stop": code_stop_training,
            "gate": gate_training,
            "gate_shuffled": gate_shuffled_training,
            "gate_density": gate_density_training.as_ref(),
            "gate_density_shuffled": gate_density_shuffled_training.as_ref(),
            "gate_horizon": horizon_training.as_ref(),
            "gate_horizon_shuffled": horizon_shuffled_training.as_ref(),
        },
        "serving": {
            "artifact": "serving.uorlut",
            "artifact_sha256": serving_artifact_sha256,
            "sidecar": "content-readout-sidecar.json",
            "sidecar_sha256": serving_sidecar_sha256,
            "parity": serving_parity,
        },
        "mapping": {
            "sign": sign_mapping_sha256,
            "code": code_mapping_sha256,
            "trajectory": trajectory_mapping,
            "code_stop": code_stop_mapping,
        },
        "arms": {
            "writer": { "reading": writer, "prompts": writer_prompts },
            "sign": { "reading": sign_arm, "prompts": sign_prompts },
            "sign_shuffled": { "reading": sign_shuffled_arm, "prompts": sign_shuffled_prompts },
            "code": { "reading": code_arm, "prompts": code_prompts },
            "code_shuffled": { "reading": code_shuffled_arm, "prompts": code_shuffled_prompts },
            "trajectory": trajectory_arms.as_ref().map(|(reading, prompts, _, _)| {
                serde_json::json!({ "reading": reading, "prompts": prompts })
            }),
            "trajectory_shuffled": trajectory_arms.as_ref().map(|(_, _, reading, prompts)| {
                serde_json::json!({ "reading": reading, "prompts": prompts })
            }),
            "code_stop": { "reading": code_stop_arm, "prompts": code_stop_prompts },
            "code_stop_shuffled": {
                "reading": code_stop_shuffled_arm,
                "prompts": code_stop_shuffled_prompts
            },
            "gate": { "reading": gate_arm, "prompts": gate_prompts },
            "gate_shuffled": {
                "reading": gate_shuffled_arm,
                "prompts": gate_shuffled_prompts
            },
            "gate_density": gate_density_arms,
            "gate_horizon": gate_horizon_arms,
        },
        "bar": bar,
        "rayon_workers": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib(),
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

fn mode_structural_pass(
    training: &uor_r4_cpu_train::ContentReadoutTraining,
    mapping: &serde_json::Value,
) -> bool {
    training.rows >= CONTENT_MIN_TRAINING_ROWS as u64
        && training.target_values >= CONTENT_MIN_TARGET_VALUES as u64
        && training.covered_rows * 10 >= training.rows * 9
        && mapping["differs"].as_bool() == Some(true)
}

fn mapping_changes(
    true_readout: &ContentReadout,
    shuffled: &ContentReadout,
) -> Result<usize, String> {
    let true_mapping = content_target_mapping(true_readout).map_err(|error| error.to_string())?;
    let shuffled_mapping = content_target_mapping(shuffled).map_err(|error| error.to_string())?;
    Ok(true_mapping
        .iter()
        .filter(|(signature, target)| shuffled_mapping.get(*signature) != Some(*target))
        .count())
}

fn mapping_json(
    true_readout: &ContentReadout,
    shuffled: &ContentReadout,
    changed_assignments: usize,
) -> Result<serde_json::Value, String> {
    let true_mapping = content_target_mapping(true_readout).map_err(|error| error.to_string())?;
    let true_sha256 = content_mapping_sha256(true_readout).map_err(|error| error.to_string())?;
    let shuffled_sha256 = content_mapping_sha256(shuffled).map_err(|error| error.to_string())?;
    Ok(serde_json::json!({
        "mode": true_readout.mode,
        "signature_bits": true_readout.signature_bits,
        "signature_entries": true_mapping.len(),
        "changed_assignments": changed_assignments,
        "differs": changed_assignments > 0 && true_sha256 != shuffled_sha256,
        "true_sha256": true_sha256,
        "shuffled_sha256": shuffled_sha256,
    }))
}

fn non_inferior(
    candidate_top1: u64,
    control_top1: u64,
    candidate_depth: u64,
    control_depth: u64,
) -> bool {
    candidate_top1 >= control_top1 && candidate_depth >= control_depth
}

fn readout_fallback_accounting(reading: &ContentReadoutReading) -> bool {
    reading.generated_decisions == reading.readout_decisions + reading.writer_fallbacks
}

fn paired(
    candidate: &[ContentPromptReading],
    control: &[ContentPromptReading],
    value: impl Fn(&ContentPromptReading) -> u64,
) -> Result<serde_json::Value, String> {
    paired_with(candidate, control, value, ceil_three_sqrt)
}

fn paired_two_sqrt(
    candidate: &[ContentPromptReading],
    control: &[ContentPromptReading],
    value: impl Fn(&ContentPromptReading) -> u64,
) -> Result<serde_json::Value, String> {
    paired_with(candidate, control, value, ceil_two_sqrt)
}

fn paired_with(
    candidate: &[ContentPromptReading],
    control: &[ContentPromptReading],
    value: impl Fn(&ContentPromptReading) -> u64,
    threshold: fn(u64) -> u64,
) -> Result<serde_json::Value, String> {
    if candidate.len() != control.len() {
        return Err("content-readout paired counts differ".to_owned());
    }
    let mut only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for (candidate, control) in candidate.iter().zip(control) {
        if candidate.sample != control.sample {
            return Err("content-readout paired identity differs".to_owned());
        }
        let left = value(candidate);
        let right = value(control);
        candidate_sum += left;
        control_sum += right;
        only += u64::from(left > right);
        control_only += u64::from(right > left);
    }
    let threshold = threshold(control_only).max(2);
    Ok(serde_json::json!({
        "candidate_only": only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && only >= threshold,
    }))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BosLengthCounts {
    at_reference_length: u64,
    shorter: u64,
    longer: u64,
}

impl BosLengthCounts {
    fn within_reference(self) -> u64 {
        self.at_reference_length + self.shorter
    }
}

fn bos_length_counts(prompts: &[ContentPromptReading]) -> BosLengthCounts {
    let mut counts = BosLengthCounts::default();
    for row in prompts.iter().filter(|row| row.stop == "bos") {
        match row.generated_tokens.cmp(&row.reference_tokens) {
            std::cmp::Ordering::Less => counts.shorter += 1,
            std::cmp::Ordering::Equal => counts.at_reference_length += 1,
            std::cmp::Ordering::Greater => counts.longer += 1,
        }
    }
    counts
}

fn paired_terminal_rows(
    rows: &[ContentStopRow],
    candidate: &ContentStopGate,
    control: &ContentStopGate,
) -> Result<serde_json::Value, String> {
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for row in rows.iter().filter(|row| row.terminal) {
        let left = u64::from(
            content_stop_gate_fires(candidate, &row.signature)
                .map_err(|error| error.to_string())?,
        );
        let right = u64::from(
            content_stop_gate_fires(control, &row.signature).map_err(|error| error.to_string())?,
        );
        candidate_sum += left;
        control_sum += right;
        candidate_only += u64::from(left > right);
        control_only += u64::from(right > left);
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum
            && candidate_only >= threshold,
    }))
}

fn paired_terminal_density_rows(
    rows: &[ContentStopRow],
    candidate: &ContentStopDensityGate,
    control: &ContentStopDensityGate,
) -> Result<serde_json::Value, String> {
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for row in rows.iter().filter(|row| row.terminal) {
        let left = u64::from(
            content_stop_density_gate_fires(candidate, &row.signature, row.offset)
                .map_err(|error| error.to_string())?,
        );
        let right = u64::from(
            content_stop_density_gate_fires(control, &row.signature, row.offset)
                .map_err(|error| error.to_string())?,
        );
        candidate_sum += left;
        control_sum += right;
        candidate_only += u64::from(left > right);
        control_only += u64::from(right > left);
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    }))
}

fn paired_terminal_continuation_rows(
    rows: &[ContentContinuationRow],
    candidate: &ContentContinuationTraining,
    shuffled_rows: &[ContentContinuationRow],
    control: &ContentContinuationTraining,
) -> Result<serde_json::Value, String> {
    if rows.len() != shuffled_rows.len() {
        return Err("continuation terminal row counts differ".to_owned());
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut candidate_sum = 0u64;
    let mut control_sum = 0u64;
    for (index, _row) in rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.remaining == 0)
    {
        let left = u64::from(
            content_continuation_loo_prediction(rows, index, candidate.gate.neighbors)
                .map_err(|error| error.to_string())?
                == Some(0),
        );
        let right = u64::from(
            content_continuation_loo_prediction(shuffled_rows, index, control.gate.neighbors)
                .map_err(|error| error.to_string())?
                == Some(0),
        );
        candidate_sum += left;
        control_sum += right;
        candidate_only += u64::from(left > right);
        control_only += u64::from(right > left);
    }
    let threshold = ceil_two_sqrt(control_only).max(2);
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    }))
}

fn horizon_predictions_changed(
    rows: &[ContentContinuationRow],
    candidate_neighbors: usize,
    comparison_rows: &[ContentContinuationRow],
    control_neighbors: usize,
) -> Result<u64, String> {
    if rows.len() != comparison_rows.len() {
        return Err("continuation prediction row counts differ".to_owned());
    }
    rows.iter()
        .zip(comparison_rows)
        .enumerate()
        .try_fold(0u64, |changed, (index, _)| {
            let left = content_continuation_loo_prediction(rows, index, candidate_neighbors)
                .map_err(|error| error.to_string())?;
            let right =
                content_continuation_loo_prediction(comparison_rows, index, control_neighbors)
                    .map_err(|error| error.to_string())?;
            Ok(changed + u64::from(left != right))
        })
}

fn ceil_three_sqrt(value: u64) -> u64 {
    let mut root = 0u64;
    while root.saturating_mul(root) < value.saturating_mul(9) {
        root += 1;
    }
    root
}

fn ceil_two_sqrt(value: u64) -> u64 {
    let mut root = 0u64;
    while root.saturating_mul(root) < value.saturating_mul(4) {
        root += 1;
    }
    root
}

fn readout_sha(readout: &ContentReadout) -> Result<String, String> {
    Ok(uor_r4_discrete::corpus::sha256(
        &serde_json::to_vec(readout).map_err(|error| error.to_string())?,
    ))
}

fn json_sha(value: &impl serde::Serialize) -> Result<String, String> {
    Ok(uor_r4_discrete::corpus::sha256(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn selection_json(
    groups: &[KeyGroup],
    training: &[MemoryWeightSample],
    evaluation: &[MemoryWeightSample],
) -> serde_json::Value {
    serde_json::json!({
        "groups": groups.len(),
        "training_samples": training.len(),
        "evaluation_samples": evaluation.len(),
        "training_content_decisions": training.iter().map(|sample| sample.reply.len().saturating_sub(1)).sum::<usize>(),
        "evaluation_content_decisions": evaluation.iter().map(|sample| sample.reply.len().saturating_sub(1)).sum::<usize>(),
    })
}

fn memory_sample(sample: &CpuConversationSample) -> MemoryWeightSample {
    MemoryWeightSample {
        prompt: sample.prompt.clone(),
        reply: sample.reply.clone(),
    }
}

fn select_groups(examples: &[CpuConversationSample], config: &Config) -> Vec<KeyGroup> {
    let mut groups =
        BTreeMap::<Vec<u32>, (Vec<CpuConversationSample>, Vec<CpuConversationSample>)>::new();
    for sample in examples {
        let key = sample
            .prompt
            .iter()
            .copied()
            .take(config.key_tokens)
            .collect::<Vec<_>>();
        let entry = groups.entry(key).or_default();
        if sample.document_ordinal % 2 == 0 {
            entry.0.push(sample.clone());
        } else {
            entry.1.push(sample.clone());
        }
    }
    let mut groups = groups
        .into_iter()
        .filter_map(|(key, (mut training, mut evaluation))| {
            if training.len() < 4 || evaluation.len() < 4 {
                return None;
            }
            training.truncate(config.per_key);
            evaluation.truncate(config.per_key);
            Some(KeyGroup {
                key,
                training,
                evaluation,
            })
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .training
            .len()
            .min(right.evaluation.len())
            .cmp(&left.training.len().min(left.evaluation.len()))
            .then_with(|| right.evaluation.len().cmp(&left.evaluation.len()))
            .then_with(|| right.training.len().cmp(&left.training.len()))
            .then_with(|| right.key.cmp(&left.key))
    });
    groups.truncate(config.key_count);
    groups
}

fn create_fresh_output(path: &Path) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(bit: usize) -> Vec<u64> {
        let mut signature = vec![0; uor_r4_cpu_train::CONTENT_CODE_SIGNATURE_BITS / 64];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn gate(terminal: Vec<u64>, nonterminal: Vec<u64>) -> ContentStopGate {
        ContentStopGate {
            signature_bits: uor_r4_cpu_train::CONTENT_CODE_SIGNATURE_BITS,
            terminal_prototypes: vec![terminal],
            nonterminal_prototypes: vec![nonterminal],
            margin: 1,
        }
    }

    fn density_gate(terminal: Vec<u64>, nonterminal: Vec<u64>) -> ContentStopDensityGate {
        ContentStopDensityGate {
            signature_bits: uor_r4_cpu_train::CONTENT_CODE_SIGNATURE_BITS,
            terminal_rows: vec![uor_r4_cpu_train::ContentStopDensityPoint {
                signature: terminal,
                offset: 4,
            }],
            nonterminal_rows: vec![uor_r4_cpu_train::ContentStopDensityPoint {
                signature: nonterminal,
                offset: 4,
            }],
            neighbors: 1,
            offset_radius: 0,
            margin: 1,
        }
    }

    #[test]
    fn content_non_inferiority_uses_both_ordered_metrics() {
        assert!(non_inferior(73, 73, 64, 64));
        assert!(non_inferior(74, 73, 64, 64));
        assert!(non_inferior(73, 73, 65, 64));
        assert!(!non_inferior(72, 73, 65, 64));
        assert!(!non_inferior(74, 73, 63, 64));
    }

    #[test]
    fn trajectory_readout_fallback_accounts_for_every_decision() {
        let reading = ContentReadoutReading {
            decisions: 2,
            generated_decisions: 10,
            readout_decisions: 3,
            writer_fallbacks: 7,
            ..ContentReadoutReading::default()
        };
        assert!(readout_fallback_accounting(&reading));
        assert!(!readout_fallback_accounting(&ContentReadoutReading {
            decisions: 2,
            generated_decisions: 10,
            readout_decisions: 3,
            writer_fallbacks: 6,
            ..ContentReadoutReading::default()
        }));
    }

    #[test]
    fn terminal_pairing_requires_the_two_sigma_disagreement() {
        let terminal_signature = signature(10);
        let rows = (0..4)
            .map(|_| ContentStopRow {
                signature: terminal_signature.clone(),
                terminal: true,
                writer_token: 7,
                offset: 4,
            })
            .collect::<Vec<_>>();
        let candidate = gate(terminal_signature.clone(), signature(100));
        let control = gate(signature(100), terminal_signature);
        let paired = paired_terminal_rows(&rows, &candidate, &control).unwrap();
        assert_eq!(paired["candidate_only"], 4);
        assert_eq!(paired["control_only"], 0);
        assert_eq!(paired["threshold"], 2);
        assert_eq!(paired["passes"], true);
        assert_eq!(ceil_two_sqrt(0), 0);
        assert_eq!(ceil_two_sqrt(1), 2);
        assert_eq!(ceil_two_sqrt(4), 4);
    }

    #[test]
    fn density_terminal_pairing_uses_the_same_control_rule() {
        let terminal_signature = signature(10);
        let rows = (0..4)
            .map(|_| ContentStopRow {
                signature: terminal_signature.clone(),
                terminal: true,
                writer_token: 7,
                offset: 4,
            })
            .collect::<Vec<_>>();
        let candidate = density_gate(terminal_signature.clone(), signature(100));
        let control = density_gate(signature(100), terminal_signature);
        let paired = paired_terminal_density_rows(&rows, &candidate, &control).unwrap();
        assert_eq!(paired["candidate_only"], 4);
        assert_eq!(paired["control_only"], 0);
        assert_eq!(paired["threshold"], 2);
        assert_eq!(paired["passes"], true);
    }

    #[test]
    fn continuation_terminal_pairing_uses_the_same_control_rule() {
        let rows = vec![
            ContentContinuationRow {
                signature: signature(0),
                remaining: 0,
                writer_token: 7,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(0),
                remaining: 0,
                writer_token: 7,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(100),
                remaining: 1,
                writer_token: 8,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(100),
                remaining: 1,
                writer_token: 8,
                offset: 4,
            },
        ];
        let shuffled = shuffle_content_continuation_rows(&rows, 1).unwrap();
        let candidate = fit_content_continuation_gate(&rows, 1).unwrap();
        let control = fit_content_continuation_gate(&shuffled, 1).unwrap();
        let paired =
            paired_terminal_continuation_rows(&rows, &candidate, &shuffled, &control).unwrap();
        assert_eq!(paired["candidate_only"], 2);
        assert_eq!(paired["control_only"], 0);
        assert_eq!(paired["threshold"], 2);
        assert_eq!(paired["passes"], true);
        assert_eq!(
            horizon_predictions_changed(
                &rows,
                candidate.gate.neighbors,
                &shuffled,
                control.gate.neighbors,
            )
            .unwrap(),
            4
        );
    }

    #[test]
    fn continuation_pairing_does_not_count_a_self_signature() {
        let rows = vec![
            ContentContinuationRow {
                signature: signature(0),
                remaining: 0,
                writer_token: 7,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(2),
                remaining: 0,
                writer_token: 7,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(100),
                remaining: 1,
                writer_token: 8,
                offset: 4,
            },
            ContentContinuationRow {
                signature: signature(201),
                remaining: 1,
                writer_token: 8,
                offset: 4,
            },
        ];
        let candidate = fit_content_continuation_gate(&rows, 1).unwrap();
        assert_eq!(
            uor_r4_cpu_train::content_continuation_gate_prediction(
                &candidate.gate,
                &signature(0),
                4
            )
            .unwrap(),
            Some(0)
        );
        assert_eq!(
            content_continuation_loo_prediction(&rows, 0, candidate.gate.neighbors).unwrap(),
            Some(1)
        );
        let shuffled = shuffle_content_continuation_rows(&rows, 1).unwrap();
        let control = fit_content_continuation_gate(&shuffled, 1).unwrap();
        let paired =
            paired_terminal_continuation_rows(&rows, &candidate, &shuffled, &control).unwrap();
        assert_eq!(paired["candidate_sum"], 0);
    }

    #[test]
    fn terminal_pairing_threshold_uses_control_only_disagreements() {
        let mut rows = (0..5)
            .map(|_| ContentStopRow {
                signature: signature(10),
                terminal: true,
                writer_token: 7,
                offset: 4,
            })
            .collect::<Vec<_>>();
        rows.extend((0..4).map(|_| ContentStopRow {
            signature: signature(100),
            terminal: true,
            writer_token: 8,
            offset: 4,
        }));
        let candidate = gate(signature(10), signature(100));
        let control = gate(signature(100), signature(10));

        let paired = paired_terminal_rows(&rows, &candidate, &control).unwrap();
        assert_eq!(paired["candidate_only"], 5);
        assert_eq!(paired["control_only"], 4);
        assert_eq!(paired["threshold"], 4);
        assert_eq!(paired["passes"], true);
    }

    #[test]
    fn paired_threshold_uses_control_only_disagreements() {
        let reading = |sample, value| ContentPromptReading {
            sample,
            matched_prefix_depth: 0,
            top1: value,
            observed_decisions: 0,
            exact_reference: false,
            stop: "cap".to_owned(),
            generated_tokens: 16,
            reference_tokens: 16,
            generated_ids: Vec::new(),
            decision_sha256: String::new(),
        };
        let candidate = (0..19)
            .map(|sample| reading(sample, u64::from(sample < 10)))
            .collect::<Vec<_>>();
        let control = (0..19)
            .map(|sample| reading(sample, u64::from((10..19).contains(&sample))))
            .collect::<Vec<_>>();

        let paired = paired(&candidate, &control, |row| row.top1).unwrap();
        assert_eq!(paired["candidate_only"], 10);
        assert_eq!(paired["control_only"], 9);
        assert_eq!(paired["threshold"], 9);
        assert_eq!(paired["passes"], true);
    }

    #[test]
    fn two_and_three_sigma_pairs_use_distinct_thresholds() {
        let reading = |sample, value| ContentPromptReading {
            sample,
            matched_prefix_depth: value,
            top1: 0,
            observed_decisions: 0,
            exact_reference: false,
            stop: "cap".to_owned(),
            generated_tokens: 16,
            reference_tokens: 16,
            generated_ids: Vec::new(),
            decision_sha256: String::new(),
        };
        let candidate = [1, 1, 0, 1]
            .into_iter()
            .enumerate()
            .map(|(sample, value)| reading(sample, value))
            .collect::<Vec<_>>();
        let control = [1, 0, 1, 0]
            .into_iter()
            .enumerate()
            .map(|(sample, value)| reading(sample, value))
            .collect::<Vec<_>>();
        let two =
            paired_two_sqrt(&candidate, &control, |row| row.matched_prefix_depth as u64).unwrap();
        let three = paired(&candidate, &control, |row| row.matched_prefix_depth as u64).unwrap();
        assert_eq!(two["threshold"], 2);
        assert_eq!(two["passes"], true);
        assert_eq!(three["threshold"], 3);
        assert_eq!(three["passes"], false);
    }

    #[test]
    fn bos_length_accounting_separates_exact_shorter_and_longer_stops() {
        let prompt = |stop: &str, generated_tokens, reference_tokens| ContentPromptReading {
            sample: 0,
            matched_prefix_depth: 0,
            top1: 0,
            observed_decisions: 0,
            exact_reference: false,
            stop: stop.to_owned(),
            generated_tokens,
            reference_tokens,
            generated_ids: Vec::new(),
            decision_sha256: String::new(),
        };
        let prompts = vec![
            prompt("bos", 2, 3),
            prompt("bos", 3, 3),
            prompt("bos", 4, 3),
            prompt("cap", 3, 3),
            prompt("context", 1, 3),
        ];

        assert_eq!(
            bos_length_counts(&prompts),
            BosLengthCounts {
                at_reference_length: 1,
                shorter: 1,
                longer: 1,
            }
        );
        assert_eq!(bos_length_counts(&prompts).within_reference(), 2);
    }
}
