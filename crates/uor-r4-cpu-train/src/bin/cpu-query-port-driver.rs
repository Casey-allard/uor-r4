use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_core::report_output;
use uor_r4_cpu_train::{
    anchor_writer_ports, random_port_gain_control, read_gain, validate_corpus_manifest,
    CpuConversationSample, CpuMaskedCorpus, CpuPanelEvaluator, CpuPortRootTrainer, PortRootConfig,
    PortRootReading, RankMarginSample,
};
use uor_r4_integer::stack::{read_hamming_uorlut01, read_memory_uorlut01, IntegerStackModel};

const STEPS: usize = 8;

struct Config {
    artifact: PathBuf,
    train_tokens: PathBuf,
    train_mask: PathBuf,
    train_manifest: PathBuf,
    panel_tokens: PathBuf,
    panel_mask: PathBuf,
    panel_manifest: PathBuf,
    out: PathBuf,
    seed: u64,
    expected_artifact_sha256: String,
    expected_train_tokens_sha256: String,
    expected_train_mask_sha256: String,
    expected_train_manifest_sha256: String,
    expected_panel_tokens_sha256: String,
    expected_panel_mask_sha256: String,
    expected_panel_manifest_sha256: String,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-query-port-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-query-port-driver ARTIFACT TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST \
     PANEL_TOKENS PANEL_MASK PANEL_MANIFEST OUT --expected-artifact-sha256 HEX \
     --expected-train-tokens-sha256 HEX --expected-train-mask-sha256 HEX \
     --expected-train-manifest-sha256 HEX --expected-panel-tokens-sha256 HEX \
     --expected-panel-mask-sha256 HEX --expected-panel-manifest-sha256 HEX [--seed N]"
        .to_owned()
}

fn parse_args() -> Result<Config, String> {
    let mut values = std::env::args().skip(1);
    let mut path = || values.next().map(PathBuf::from).ok_or_else(usage);
    let mut config = Config {
        artifact: path()?,
        train_tokens: path()?,
        train_mask: path()?,
        train_manifest: path()?,
        panel_tokens: path()?,
        panel_mask: path()?,
        panel_manifest: path()?,
        out: path()?,
        seed: 183,
        expected_artifact_sha256: String::new(),
        expected_train_tokens_sha256: String::new(),
        expected_train_mask_sha256: String::new(),
        expected_train_manifest_sha256: String::new(),
        expected_panel_tokens_sha256: String::new(),
        expected_panel_mask_sha256: String::new(),
        expected_panel_manifest_sha256: String::new(),
    };
    while let Some(flag) = values.next() {
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--seed" => config.seed = number(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-train-tokens-sha256" => config.expected_train_tokens_sha256 = hex(&value)?,
            "--expected-train-mask-sha256" => config.expected_train_mask_sha256 = hex(&value)?,
            "--expected-train-manifest-sha256" => {
                config.expected_train_manifest_sha256 = hex(&value)?
            }
            "--expected-panel-tokens-sha256" => config.expected_panel_tokens_sha256 = hex(&value)?,
            "--expected-panel-mask-sha256" => config.expected_panel_mask_sha256 = hex(&value)?,
            "--expected-panel-manifest-sha256" => {
                config.expected_panel_manifest_sha256 = hex(&value)?
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("artifact", &config.expected_artifact_sha256),
        ("train-tokens", &config.expected_train_tokens_sha256),
        ("train-mask", &config.expected_train_mask_sha256),
        ("train-manifest", &config.expected_train_manifest_sha256),
        ("panel-tokens", &config.expected_panel_tokens_sha256),
        ("panel-mask", &config.expected_panel_mask_sha256),
        ("panel-manifest", &config.expected_panel_manifest_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
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
    report_output::claim(&config.out).map_err(|error| {
        format!(
            "claim exclusive report directory {}: {error}",
            config.out.display()
        )
    })?;
    let result = run_claimed(&config, started);
    if let Err(error) = &result {
        let failure = serde_json::json!({
            "schema": "uor-r4.cpu-query-port-failure/1",
            "status": "INCOMPLETE_OR_UNVERIFIED",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "error": error,
            "inputs": inputs_json(&config),
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        });
        write_new(
            &config.out.join("failed-attempt.json"),
            &serde_json::to_vec_pretty(&failure).map_err(|error| error.to_string())?,
        )?;
    }
    report_output::seal(&config.out)
        .map_err(|error| format!("seal report directory {}: {error}", config.out.display()))?;
    report_output::verify(&config.out)
        .map_err(|error| format!("verify sealed report directory: {error}"))?;
    result
}

fn run_claimed(config: &Config, started: Instant) -> Result<serde_json::Value, String> {
    let artifact = std::fs::read(&config.artifact).map_err(|error| error.to_string())?;
    let train_tokens = std::fs::read(&config.train_tokens).map_err(|error| error.to_string())?;
    let train_mask = std::fs::read(&config.train_mask).map_err(|error| error.to_string())?;
    let train_manifest =
        std::fs::read(&config.train_manifest).map_err(|error| error.to_string())?;
    let panel_tokens = std::fs::read(&config.panel_tokens).map_err(|error| error.to_string())?;
    let panel_mask = std::fs::read(&config.panel_mask).map_err(|error| error.to_string())?;
    let panel_manifest =
        std::fs::read(&config.panel_manifest).map_err(|error| error.to_string())?;
    for (name, bytes, expected) in [
        (
            "artifact",
            artifact.as_slice(),
            &config.expected_artifact_sha256,
        ),
        (
            "train-tokens",
            train_tokens.as_slice(),
            &config.expected_train_tokens_sha256,
        ),
        (
            "train-mask",
            train_mask.as_slice(),
            &config.expected_train_mask_sha256,
        ),
        (
            "train-manifest",
            train_manifest.as_slice(),
            &config.expected_train_manifest_sha256,
        ),
        (
            "panel-tokens",
            panel_tokens.as_slice(),
            &config.expected_panel_tokens_sha256,
        ),
        (
            "panel-mask",
            panel_mask.as_slice(),
            &config.expected_panel_mask_sha256,
        ),
        (
            "panel-manifest",
            panel_manifest.as_slice(),
            &config.expected_panel_manifest_sha256,
        ),
    ] {
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        if &actual != expected {
            return Err(format!("{name} SHA-256 differs: got {actual}"));
        }
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    let shape = model.shape();
    let train = corpus(
        &train_tokens,
        &train_mask,
        &train_manifest,
        shape.vocab,
        shape.context,
        "train",
    )?;
    let panel = corpus(
        &panel_tokens,
        &panel_mask,
        &panel_manifest,
        shape.vocab,
        shape.context,
        "dev",
    )?;
    if read_hamming_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("port-root source artifact must have no Hamming read".to_owned());
    }
    if read_memory_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("port-root source artifact must be memory-free".to_owned());
    }
    let training_samples = response_samples(
        &train
            .conversation_examples()
            .map_err(|error| error.to_string())?,
    )?;
    let panel_samples = response_samples(
        &panel
            .conversation_examples()
            .map_err(|error| error.to_string())?,
    )?;
    if training_samples.len() < 200 || panel_samples.len() < 200 {
        return Err(format!(
            "port-root power floor differs: {} training, {} panel rows",
            training_samples.len(),
            panel_samples.len()
        ));
    }

    let frozen_panel = panel_reading(&artifact, &panel_samples)?;
    let frozen_port = port_reading(&artifact, &panel_samples)?;
    let anchored = anchor_writer_ports(&artifact).map_err(|error| error.to_string())?;
    let anchored_panel = panel_reading(&anchored, &panel_samples)?;
    let anchored_port = port_reading(&anchored, &panel_samples)?;

    let mut trained = CpuPortRootTrainer::from_artifact(
        &anchored,
        training_samples.clone(),
        PortRootConfig::default(),
    )
    .map_err(|error| error.to_string())?;
    let mut trained_steps = Vec::with_capacity(STEPS);
    for _ in 0..STEPS {
        trained_steps.push(trained.train_step().map_err(|error| error.to_string())?);
        check_stop(started)?;
    }
    let accepted = trained_steps.iter().filter(|step| step.updated).count();
    let trained_artifact = trained.artifact().to_vec();
    assert_only_port_gain_changed(&anchored, &trained_artifact)?;
    write_new(&config.out.join("trained.uorlut"), &trained_artifact)?;

    let mut shuffled = CpuPortRootTrainer::from_artifact(
        &anchored,
        training_samples.clone(),
        PortRootConfig {
            cell_shift: 1,
            ..PortRootConfig::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let mut shuffled_steps = Vec::new();
    let mut shuffled_accepted = 0usize;
    while shuffled_accepted < accepted && shuffled_steps.len() < 32 {
        let step = shuffled.train_step().map_err(|error| error.to_string())?;
        shuffled_accepted += usize::from(step.updated);
        shuffled_steps.push(step);
        check_stop(started)?;
    }
    let shuffled_artifact = shuffled.artifact().to_vec();
    assert_only_port_gain_changed(&anchored, &shuffled_artifact)?;
    write_new(&config.out.join("shuffled.uorlut"), &shuffled_artifact)?;
    let (random_artifact, random_moves) =
        random_port_gain_control(&anchored, accepted, config.seed)
            .map_err(|error| error.to_string())?;
    assert_only_port_gain_changed(&anchored, &random_artifact)?;
    write_new(&config.out.join("random.uorlut"), &random_artifact)?;
    write_new(&config.out.join("table-anchored.uorlut"), &anchored)?;

    let trained_panel = panel_reading(&trained_artifact, &panel_samples)?;
    let trained_port = port_reading(&trained_artifact, &panel_samples)?;
    let shuffled_panel = panel_reading(&shuffled_artifact, &panel_samples)?;
    let shuffled_port = port_reading(&shuffled_artifact, &panel_samples)?;
    let random_panel = panel_reading(&random_artifact, &panel_samples)?;
    let random_port = port_reading(&random_artifact, &panel_samples)?;

    let mut repeat = CpuPortRootTrainer::from_artifact(
        &anchored,
        training_samples.clone(),
        PortRootConfig::default(),
    )
    .map_err(|error| error.to_string())?;
    let mut repeat_steps = Vec::with_capacity(STEPS);
    for _ in 0..STEPS {
        repeat_steps.push(repeat.train_step().map_err(|error| error.to_string())?);
    }
    if repeat.artifact() != trained_artifact.as_slice() || repeat_steps != trained_steps {
        return Err("port-root repeat did not reproduce artifact and steps".to_owned());
    }

    let parity_exact = [
        &frozen_panel,
        &trained_panel,
        &shuffled_panel,
        &random_panel,
        &anchored_panel,
    ]
    .iter()
    .all(|reading| reading.export.is_exact());
    let trained_distance_pass = trained_port.target_port_distance_sum
        < anchored_port.target_port_distance_sum
        && trained_port.target_port_distance_sum < random_port.target_port_distance_sum
        && trained_port.target_port_distance_sum < shuffled_port.target_port_distance_sum;
    let safety_pass = trained_port.target_admitted >= anchored_port.target_admitted
        && trained_port.top1_correct >= anchored_port.top1_correct;
    let control_moves_matched = accepted == shuffled_accepted;
    let status = run_status(
        parity_exact,
        accepted,
        shuffled_accepted,
        trained_distance_pass,
        safety_pass,
    );

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-query-port-driver/1",
        "status": status,
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": inputs_json(config),
        "execution": {
            "threads": rayon::current_num_threads(),
            "seed": config.seed,
            "steps": STEPS,
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        },
        "power": {
            "training_rows": training_samples.len(),
            "panel_rows": panel_samples.len(),
        },
        "preconditioning": {
            "table_anchor_artifact_sha256": uor_r4_discrete::corpus::sha256(&anchored),
        },
        "training": {
            "accepted": accepted,
            "steps": trained_steps,
            "shuffled_accepted": shuffled_accepted,
            "shuffled_attempts": shuffled_steps.len(),
            "shuffled_steps": shuffled_steps,
            "random_moves": random_moves,
        },
        "arms": {
            "raw_frozen": arm_json(&artifact, &frozen_panel, &frozen_port),
            "table_anchor_baseline": arm_json(&anchored, &anchored_panel, &anchored_port),
            "trained": arm_json(&trained_artifact, &trained_panel, &trained_port),
            "shuffled_cells": arm_json(&shuffled_artifact, &shuffled_panel, &shuffled_port),
            "random_gain": arm_json(&random_artifact, &random_panel, &random_port),
        },
        "bar": {
            "parity_exact": parity_exact,
            "control_moves_matched": control_moves_matched,
            "target_port_distance_below_table_anchor_random_and_shuffled": trained_distance_pass,
            "admission_and_top1_non_fall": safety_pass,
        },
    });
    write_new(
        &config.out.join("report.json"),
        &serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )?;
    Ok(report)
}

fn corpus(
    tokens: &[u8],
    mask: &[u8],
    manifest: &[u8],
    vocab: usize,
    context: usize,
    split: &str,
) -> Result<CpuMaskedCorpus, String> {
    let identity =
        validate_corpus_manifest(manifest, tokens, mask).map_err(|error| error.to_string())?;
    if identity.vocab != vocab || identity.bos != 0 || identity.split != split {
        return Err("corpus vocabulary, BOS or split differs from registration".to_owned());
    }
    CpuMaskedCorpus::from_bytes(tokens, mask, vocab, context, identity.bos)
        .map_err(|error| error.to_string())
}

fn response_samples(examples: &[CpuConversationSample]) -> Result<Vec<RankMarginSample>, String> {
    let mut samples = Vec::new();
    for example in examples {
        let tokens = example
            .prompt
            .iter()
            .chain(&example.reply)
            .copied()
            .collect::<Vec<_>>();
        for target_index in example.prompt.len()..tokens.len() {
            if target_index == 0 {
                continue;
            }
            samples.push(RankMarginSample {
                input: tokens[..target_index].to_vec(),
                target: tokens[target_index],
            });
        }
    }
    if samples.is_empty() {
        return Err("corpus has no response-token targets".to_owned());
    }
    Ok(samples)
}

fn assert_only_port_gain_changed(before: &[u8], after: &[u8]) -> Result<(), String> {
    let before = read_gain(before).map_err(|error| error.to_string())?;
    let after = read_gain(after).map_err(|error| error.to_string())?;
    if before.len() != after.len() {
        return Err("final-gain length changed".to_owned());
    }
    if before
        .iter()
        .zip(&after)
        .enumerate()
        .any(|(index, (left, right))| left != right && (8..before.len() - 8).contains(&index))
    {
        return Err("a non-port final-gain coordinate changed".to_owned());
    }
    Ok(())
}

fn panel_reading(
    artifact: &[u8],
    samples: &[RankMarginSample],
) -> Result<uor_r4_cpu_train::PanelReading, String> {
    CpuPanelEvaluator::from_artifact(artifact, samples.to_vec(), 1)
        .and_then(|mut evaluator| evaluator.evaluate())
        .map_err(|error| error.to_string())
}

fn port_reading(artifact: &[u8], samples: &[RankMarginSample]) -> Result<PortRootReading, String> {
    CpuPortRootTrainer::from_artifact(artifact, samples.to_vec(), PortRootConfig::default())
        .and_then(|trainer| trainer.reading())
        .map_err(|error| error.to_string())
}

fn arm_json(
    artifact: &[u8],
    panel: &uor_r4_cpu_train::PanelReading,
    port: &PortRootReading,
) -> serde_json::Value {
    serde_json::json!({
        "artifact_sha256": uor_r4_discrete::corpus::sha256(artifact),
        "panel": panel,
        "port": port,
    })
}

fn inputs_json(config: &Config) -> serde_json::Value {
    serde_json::json!({
        "artifact": config.artifact,
        "artifact_sha256": config.expected_artifact_sha256,
        "train_tokens": config.train_tokens,
        "train_tokens_sha256": config.expected_train_tokens_sha256,
        "train_mask": config.train_mask,
        "train_mask_sha256": config.expected_train_mask_sha256,
        "train_manifest": config.train_manifest,
        "train_manifest_sha256": config.expected_train_manifest_sha256,
        "panel_tokens": config.panel_tokens,
        "panel_tokens_sha256": config.expected_panel_tokens_sha256,
        "panel_mask": config.panel_mask,
        "panel_mask_sha256": config.expected_panel_mask_sha256,
        "panel_manifest": config.panel_manifest,
        "panel_manifest_sha256": config.expected_panel_manifest_sha256,
    })
}

fn check_stop(started: Instant) -> Result<(), String> {
    if started.elapsed().as_secs() > 20 * 60 {
        return Err("port-root run exceeded its 20-minute stop".to_owned());
    }
    if peak_rss_kib() > 1024 * 1024 {
        return Err("port-root run exceeded its 1 GiB RSS stop".to_owned());
    }
    Ok(())
}

fn run_status(
    parity_exact: bool,
    accepted: usize,
    shuffled_accepted: usize,
    distance_pass: bool,
    safety_pass: bool,
) -> &'static str {
    if !parity_exact {
        "INSTRUMENT_FAULT"
    } else if accepted == 0 {
        "NO_MOVE"
    } else if shuffled_accepted != accepted {
        "CONTROL_FAULT"
    } else if distance_pass && safety_pass {
        "KEEP"
    } else {
        "REJECT"
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    std::fs::write(path, bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_samples_cover_each_reply_target_after_the_prompt() {
        let examples = vec![CpuConversationSample {
            document_ordinal: 0,
            prompt: vec![10, 11],
            reply: vec![20, 21, 22],
        }];
        let samples = response_samples(&examples).unwrap();
        assert_eq!(
            samples,
            vec![
                RankMarginSample {
                    input: vec![10, 11],
                    target: 20,
                },
                RankMarginSample {
                    input: vec![10, 11, 20],
                    target: 21,
                },
                RankMarginSample {
                    input: vec![10, 11, 20, 21],
                    target: 22,
                },
            ]
        );
        assert!(response_samples(&[CpuConversationSample {
            document_ordinal: 0,
            prompt: vec![10],
            reply: Vec::new(),
        }])
        .is_err());
    }

    #[test]
    fn usage_names_every_required_input() {
        for name in [
            "ARTIFACT",
            "TRAIN_TOKENS",
            "TRAIN_MASK",
            "TRAIN_MANIFEST",
            "PANEL_TOKENS",
            "PANEL_MASK",
            "PANEL_MANIFEST",
            "OUT",
        ] {
            assert!(usage().contains(name));
        }
    }

    #[test]
    fn status_distinguishes_no_move_from_an_unmatched_control() {
        assert_eq!(run_status(true, 0, 0, false, false), "NO_MOVE");
        assert_eq!(run_status(true, 2, 1, true, true), "CONTROL_FAULT");
        assert_eq!(run_status(true, 2, 2, true, true), "KEEP");
        assert_eq!(run_status(true, 2, 2, false, true), "REJECT");
        assert_eq!(run_status(false, 2, 2, true, true), "INSTRUMENT_FAULT");
    }
}
