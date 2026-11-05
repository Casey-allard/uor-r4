use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_core::report_output;
use uor_r4_cpu_train::{
    anchor_writer_ports, random_port_gain_control, read_gain, replace_final_gain,
    validate_corpus_manifest, CpuConversationSample, CpuFbsForward, CpuFbsHead, CpuMaskedCorpus,
    CpuPanelEvaluator, CpuPortRootTrainer, PanelReading, PortRootConfig, PortRootReading,
    RankMarginSample,
};
use uor_r4_integer::stack::{read_hamming_uorlut01, read_memory_uorlut01, IntegerStackModel};

const STEPS: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum BaselineMode {
    #[default]
    Anchored,
    Raw,
}

impl BaselineMode {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "anchored" => Ok(Self::Anchored),
            "raw" => Ok(Self::Raw),
            _ => Err(format!("baseline mode must be anchored or raw: {value}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Anchored => "anchored",
            Self::Raw => "raw",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BehavioralSignature {
    token_digest: [u8; 32],
    kept_cell_digest: [u8; 32],
    samples: usize,
    target_admitted: u64,
    top1_correct: u64,
    top8_correct: u64,
    target_port_distance_sum: u64,
    target_port_abs_sum: u64,
    margin_loss_sum: i64,
    target_rank_sum: u64,
    query_coordinate_mismatches: u64,
    token_mismatches: u64,
    kept_cell_mismatches: u64,
}

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
    baseline_mode: BaselineMode,
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
     --expected-panel-mask-sha256 HEX --expected-panel-manifest-sha256 HEX \
     [--baseline-mode anchored|raw] [--seed N]"
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
        baseline_mode: BaselineMode::Anchored,
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
            "--baseline-mode" => config.baseline_mode = BaselineMode::parse(&value)?,
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
    let baseline = baseline_artifact(&artifact, config.baseline_mode)?;
    if config.baseline_mode == BaselineMode::Raw && baseline != artifact {
        return Err("raw baseline differs from the source artifact".to_owned());
    }
    let baseline_panel = panel_reading(&baseline, &panel_samples)?;
    let baseline_port = port_reading(&baseline, &panel_samples)?;

    let mut trained = CpuPortRootTrainer::from_artifact(
        &baseline,
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
    assert_only_port_gain_changed(&baseline, &trained_artifact)?;
    write_new(&config.out.join("trained.uorlut"), &trained_artifact)?;

    let mut shuffled = CpuPortRootTrainer::from_artifact(
        &baseline,
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
    assert_only_port_gain_changed(&baseline, &shuffled_artifact)?;
    write_new(&config.out.join("shuffled.uorlut"), &shuffled_artifact)?;
    let (random_artifact, random_moves) =
        random_port_gain_control(&baseline, accepted, config.seed)
            .map_err(|error| error.to_string())?;
    assert_only_port_gain_changed(&baseline, &random_artifact)?;
    write_new(&config.out.join("random.uorlut"), &random_artifact)?;
    let baseline_name = match config.baseline_mode {
        BaselineMode::Anchored => "table-anchored.uorlut",
        BaselineMode::Raw => "raw-baseline.uorlut",
    };
    write_new(&config.out.join(baseline_name), &baseline)?;

    let trained_panel = panel_reading(&trained_artifact, &panel_samples)?;
    let trained_port = port_reading(&trained_artifact, &panel_samples)?;
    let shuffled_panel = panel_reading(&shuffled_artifact, &panel_samples)?;
    let shuffled_port = port_reading(&shuffled_artifact, &panel_samples)?;
    let random_panel = panel_reading(&random_artifact, &panel_samples)?;
    let random_port = port_reading(&random_artifact, &panel_samples)?;

    let mut repeat = CpuPortRootTrainer::from_artifact(
        &baseline,
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
        &baseline_panel,
    ]
    .iter()
    .all(|reading| reading.export.is_exact());
    let trained_distance_pass = trained_port.target_port_distance_sum
        < baseline_port.target_port_distance_sum
        && trained_port.target_port_distance_sum < random_port.target_port_distance_sum
        && trained_port.target_port_distance_sum < shuffled_port.target_port_distance_sum;
    let admission_non_fall = trained_port.target_admitted >= baseline_port.target_admitted;
    let top1_non_fall = trained_port.top1_correct >= baseline_port.top1_correct;
    let top8_non_fall = trained_port.top8_correct >= baseline_port.top8_correct;
    let safety_pass = admission_non_fall
        && top1_non_fall
        && (config.baseline_mode != BaselineMode::Raw || top8_non_fall);
    let control_moves_matched = accepted == shuffled_accepted;
    let trained_behavior = behavior_signature(
        &trained_artifact,
        &panel_samples,
        &trained_panel,
        &trained_port,
    )?;
    let shuffled_behavior = behavior_signature(
        &shuffled_artifact,
        &panel_samples,
        &shuffled_panel,
        &shuffled_port,
    )?;
    let shuffled_arm_distinct = trained_behavior != shuffled_behavior;
    let status = run_status(
        parity_exact,
        accepted,
        shuffled_accepted,
        shuffled_arm_distinct,
        trained_distance_pass,
        safety_pass,
    );
    let mut arms = serde_json::Map::new();
    arms.insert(
        "raw_frozen".to_owned(),
        arm_json(&artifact, &frozen_panel, &frozen_port),
    );
    let baseline_arm = match config.baseline_mode {
        BaselineMode::Anchored => "table_anchor_baseline",
        BaselineMode::Raw => "raw_baseline",
    };
    arms.insert(
        baseline_arm.to_owned(),
        arm_json(&baseline, &baseline_panel, &baseline_port),
    );
    arms.insert(
        "trained".to_owned(),
        arm_json(&trained_artifact, &trained_panel, &trained_port),
    );
    arms.insert(
        "shuffled_cells".to_owned(),
        arm_json(&shuffled_artifact, &shuffled_panel, &shuffled_port),
    );
    arms.insert(
        "random_gain".to_owned(),
        arm_json(&random_artifact, &random_panel, &random_port),
    );
    let table_anchor_sha256 = match config.baseline_mode {
        BaselineMode::Anchored => Some(uor_r4_discrete::corpus::sha256(&baseline)),
        BaselineMode::Raw => None,
    };
    let distance_key = match config.baseline_mode {
        BaselineMode::Anchored => "target_port_distance_below_table_anchor_random_and_shuffled",
        BaselineMode::Raw => "target_port_distance_below_raw_random_and_shuffled",
    };
    let mut bar = serde_json::Map::new();
    bar.insert("parity_exact".to_owned(), serde_json::json!(parity_exact));
    bar.insert(
        "control_moves_matched".to_owned(),
        serde_json::json!(control_moves_matched),
    );
    bar.insert(
        "shuffled_arm_distinct".to_owned(),
        serde_json::json!(shuffled_arm_distinct),
    );
    bar.insert(
        "admission_non_fall".to_owned(),
        serde_json::json!(admission_non_fall),
    );
    bar.insert("top1_non_fall".to_owned(), serde_json::json!(top1_non_fall));
    bar.insert(
        distance_key.to_owned(),
        serde_json::json!(trained_distance_pass),
    );
    match config.baseline_mode {
        BaselineMode::Anchored => {
            bar.insert(
                "admission_and_top1_non_fall".to_owned(),
                serde_json::json!(safety_pass),
            );
        }
        BaselineMode::Raw => {
            bar.insert("top8_non_fall".to_owned(), serde_json::json!(top8_non_fall));
            bar.insert(
                "admission_top1_top8_non_fall".to_owned(),
                serde_json::json!(safety_pass),
            );
        }
    }
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
            "baseline_mode": config.baseline_mode.as_str(),
            "baseline_artifact_sha256": uor_r4_discrete::corpus::sha256(&baseline),
            "table_anchor_artifact_sha256": table_anchor_sha256,
        },
        "training": {
            "accepted": accepted,
            "steps": trained_steps,
            "shuffled_accepted": shuffled_accepted,
            "shuffled_attempts": shuffled_steps.len(),
            "shuffled_steps": shuffled_steps,
            "random_moves": random_moves,
        },
        "arms": arms,
        "bar": bar,
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

fn baseline_artifact(artifact: &[u8], mode: BaselineMode) -> Result<Vec<u8>, String> {
    match mode {
        BaselineMode::Anchored => anchor_writer_ports(artifact).map_err(|error| error.to_string()),
        BaselineMode::Raw => Ok(artifact.to_vec()),
    }
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
    let before_gain = read_gain(before).map_err(|error| error.to_string())?;
    let after_gain = read_gain(after).map_err(|error| error.to_string())?;
    if before_gain.len() != after_gain.len() {
        return Err("final-gain length changed".to_owned());
    }
    if before_gain
        .iter()
        .zip(&after_gain)
        .enumerate()
        .any(|(index, (left, right))| left != right && (8..before_gain.len() - 8).contains(&index))
    {
        return Err("a non-port final-gain coordinate changed".to_owned());
    }
    let restored = replace_final_gain(after, &before_gain).map_err(|error| error.to_string())?;
    if restored != before {
        return Err("bytes outside the final-gain table changed".to_owned());
    }
    Ok(())
}

fn panel_reading(artifact: &[u8], samples: &[RankMarginSample]) -> Result<PanelReading, String> {
    CpuPanelEvaluator::from_artifact(artifact, samples.to_vec(), 1)
        .and_then(|mut evaluator| evaluator.evaluate())
        .map_err(|error| error.to_string())
}

fn behavior_signature(
    artifact: &[u8],
    samples: &[RankMarginSample],
    panel: &PanelReading,
    port: &PortRootReading,
) -> Result<BehavioralSignature, String> {
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
    let mut token_bytes = Vec::with_capacity(samples.len() * 4);
    let mut kept_cell_bytes = Vec::new();
    for sample in samples {
        forward.reset();
        let mut step = None;
        for &token in &sample.input {
            step = Some(
                forward
                    .step(token, CpuFbsHead::Writer)
                    .map_err(|error| error.to_string())?,
            );
        }
        let step = step.ok_or_else(|| "behavior sample input is empty".to_owned())?;
        token_bytes.extend_from_slice(&step.token.to_le_bytes());
        for cell in step.kept_cells {
            kept_cell_bytes.extend_from_slice(&cell.to_le_bytes());
        }
    }
    behavior_signature_from(
        panel,
        port,
        digest32(&token_bytes),
        digest32(&kept_cell_bytes),
    )
}

fn behavior_signature_from(
    panel: &PanelReading,
    port: &PortRootReading,
    token_digest: [u8; 32],
    kept_cell_digest: [u8; 32],
) -> Result<BehavioralSignature, String> {
    Ok(BehavioralSignature {
        token_digest,
        kept_cell_digest,
        samples: panel.samples,
        target_admitted: u64::try_from(panel.target_admitted)
            .map_err(|_| "panel admitted count does not fit u64".to_owned())?,
        top1_correct: u64::try_from(panel.top1_correct)
            .map_err(|_| "panel top-1 count does not fit u64".to_owned())?,
        top8_correct: u64::try_from(panel.top8_correct)
            .map_err(|_| "panel top-8 count does not fit u64".to_owned())?,
        target_port_distance_sum: port.target_port_distance_sum,
        target_port_abs_sum: port.target_port_abs_sum,
        margin_loss_sum: panel.margin_loss_sum,
        target_rank_sum: panel.target_rank_sum,
        query_coordinate_mismatches: panel.export.query_coordinate_mismatches,
        token_mismatches: panel.export.token_mismatches,
        kept_cell_mismatches: panel.export.kept_cell_mismatches,
    })
}

fn digest32(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

fn port_reading(artifact: &[u8], samples: &[RankMarginSample]) -> Result<PortRootReading, String> {
    CpuPortRootTrainer::from_artifact(artifact, samples.to_vec(), PortRootConfig::default())
        .and_then(|trainer| trainer.reading())
        .map_err(|error| error.to_string())
}

fn arm_json(artifact: &[u8], panel: &PanelReading, port: &PortRootReading) -> serde_json::Value {
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
    shuffled_arm_distinct: bool,
    distance_pass: bool,
    safety_pass: bool,
) -> &'static str {
    if !parity_exact {
        "INSTRUMENT_FAULT"
    } else if accepted == 0 {
        "NO_MOVE"
    } else if shuffled_accepted != accepted || !shuffled_arm_distinct {
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
    use uor_r4_cpu_train::PanelExportReading;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

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
            "--baseline-mode",
        ] {
            assert!(usage().contains(name));
        }
    }

    #[test]
    fn baseline_mode_defaults_to_anchored_and_parses_raw() {
        assert_eq!(BaselineMode::default(), BaselineMode::Anchored);
        assert_eq!(
            BaselineMode::parse("anchored").unwrap(),
            BaselineMode::Anchored
        );
        assert_eq!(BaselineMode::parse("raw").unwrap(), BaselineMode::Raw);
        assert!(BaselineMode::parse("table").is_err());
    }

    fn synthetic_artifact() -> Vec<u8> {
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            194,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 64,
                layers: 2,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        ))
        .unwrap()
    }

    #[test]
    fn raw_mode_does_not_substitute_table_rows() {
        let artifact = synthetic_artifact();
        assert_eq!(
            baseline_artifact(&artifact, BaselineMode::Raw).unwrap(),
            artifact
        );
        assert_ne!(
            baseline_artifact(&artifact, BaselineMode::Anchored).unwrap(),
            artifact
        );
    }

    fn behavior_fixtures() -> (PanelReading, PortRootReading) {
        (
            PanelReading {
                artifact_digest: [1; 32],
                sample_fingerprint: [2; 32],
                margin_q12: 1,
                samples: 616,
                target_admitted: 10,
                top1_correct: 8,
                top8_correct: 9,
                margin_loss_sum: 100,
                target_rank_sum: 200,
                export: PanelExportReading {
                    query_coordinates_checked: 157_696,
                    query_coordinate_mismatches: 0,
                    token_mismatches: 0,
                    kept_cells_checked: 4_928,
                    kept_cell_mismatches: 0,
                },
            },
            PortRootReading {
                artifact_sha256: "00".repeat(32),
                samples: 616,
                target_admitted: 10,
                top1_correct: 8,
                top8_correct: 9,
                target_port_distance_sum: 1_000,
                target_port_abs_sum: 2_000,
                port_coordinates_checked: 9_856,
            },
        )
    }

    #[test]
    fn behavioral_signature_ignores_artifact_identity() {
        let (panel, port) = behavior_fixtures();
        let first = behavior_signature_from(&panel, &port, [3; 32], [4; 32]).unwrap();
        let second = behavior_signature_from(&panel, &port, [3; 32], [4; 32]).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn behavioral_signature_detects_decision_and_port_differences() {
        let (panel, port) = behavior_fixtures();
        let baseline = behavior_signature_from(&panel, &port, [3; 32], [4; 32]).unwrap();

        let decision_changed = behavior_signature_from(&panel, &port, [5; 32], [4; 32]).unwrap();
        assert_ne!(baseline, decision_changed);

        let mut port_changed = port.clone();
        port_changed.target_port_distance_sum -= 1;
        let port_changed =
            behavior_signature_from(&panel, &port_changed, [3; 32], [4; 32]).unwrap();
        assert_ne!(baseline, port_changed);
    }

    #[test]
    fn port_gain_guard_checks_full_artifact_identity() {
        let artifact = synthetic_artifact();
        let gain = read_gain(&artifact).unwrap();
        let restored = replace_final_gain(&artifact, &gain).unwrap();
        assert_only_port_gain_changed(&artifact, &restored).unwrap();

        let mut port = gain.clone();
        port[3] += 1;
        let port_changed = replace_final_gain(&artifact, &port).unwrap();
        assert_only_port_gain_changed(&artifact, &port_changed).unwrap();

        let mut interior = gain;
        interior[16] += 1;
        let interior_changed = replace_final_gain(&artifact, &interior).unwrap();
        assert!(assert_only_port_gain_changed(&artifact, &interior_changed).is_err());
    }

    #[test]
    fn status_distinguishes_no_move_from_an_unmatched_control() {
        assert_eq!(run_status(true, 0, 0, false, false, false), "NO_MOVE");
        assert_eq!(run_status(true, 2, 1, true, true, true), "CONTROL_FAULT");
        assert_eq!(run_status(true, 2, 2, false, true, true), "CONTROL_FAULT");
        assert_eq!(run_status(true, 2, 2, true, true, true), "KEEP");
        assert_eq!(run_status(true, 2, 2, true, false, true), "REJECT");
        assert_eq!(
            run_status(false, 2, 2, true, true, true),
            "INSTRUMENT_FAULT"
        );
    }
}
