use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_core::report_output;
use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory,
    evaluate_memory_weight_samples, random_gain_control, read_gain, validate_corpus_manifest,
    CpuConversationSample, CpuFbsForward, CpuFbsHead, CpuMaskedCorpus, CpuMemoryWeightTrainer,
    CpuServedGainTrainer, MemoryWeightConfig, MemoryWeightSample, ServedGainConfig,
};
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    steps: usize,
    memory_steps: usize,
    key_count: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    seed: u64,
    reference: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
    expected_memory_sha256: String,
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
            eprintln!("cpu-gain-panel-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-gain-panel-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     --expected-memory-sha256 HEX [--steps N] [--memory-steps N] [--keys N] \
     [--per-key N] [--key-tokens N] [--reply-turn N] [--seed N] [--reference]"
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
        steps: 8,
        memory_steps: 8,
        key_count: 8,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        seed: 1,
        reference: false,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
        expected_memory_sha256: String::new(),
    };
    while let Some(flag) = values.next() {
        if flag == "--reference" {
            config.reference = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--steps" => config.steps = number(&value)?,
            "--memory-steps" => config.memory_steps = number(&value)?,
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--seed" => config.seed = number::<u64>(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            "--expected-memory-sha256" => config.expected_memory_sha256 = hex(&value)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("artifact", &config.expected_artifact_sha256),
        ("tokens", &config.expected_tokens_sha256),
        ("mask", &config.expected_mask_sha256),
        ("manifest", &config.expected_manifest_sha256),
        ("memory", &config.expected_memory_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
    }
    if config.steps != 8
        || config.memory_steps != 8
        || config.key_count == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
    {
        return Err("registered training requires eight memory and eight gain steps".to_owned());
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
            "schema": "uor-r4.cpu-gain-panel-failure/1",
            "status": "INCOMPLETE_OR_UNVERIFIED",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "error": error,
            "inputs": {
                "artifact": config.artifact,
                "artifact_sha256": config.expected_artifact_sha256,
                "tokens": config.tokens,
                "tokens_sha256": config.expected_tokens_sha256,
                "mask": config.mask,
                "mask_sha256": config.expected_mask_sha256,
                "manifest": config.manifest,
                "manifest_sha256": config.expected_manifest_sha256,
                "memory_baseline_sha256": config.expected_memory_sha256,
            },
            "execution": {
                "threads": rayon::current_num_threads(),
                "reference": config.reference,
                "seed": config.seed,
            },
            "scope": "Preserved execution or instrument failure; not a capability verdict. No automatic retry.",
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        });
        let bytes = serde_json::to_vec_pretty(&failure)
            .map_err(|error| format!("serialize failed-attempt report: {error}"))?;
        write_new(&config.out.join("failed-attempt.json"), &bytes)?;
    }
    report_output::seal(&config.out)
        .map_err(|error| format!("seal report directory {}: {error}", config.out.display()))?;
    report_output::verify(&config.out)
        .map_err(|error| format!("verify sealed report directory: {error}"))?;
    result
}

fn run_claimed(config: &Config, started: Instant) -> Result<serde_json::Value, String> {
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
    let groups = select_groups(&examples, config);
    let training_samples = groups
        .iter()
        .flat_map(|group| group.training.iter())
        .map(memory_sample)
        .collect::<Vec<_>>();
    let evaluation_samples = groups
        .iter()
        .flat_map(|group| group.evaluation.iter())
        .map(memory_sample)
        .collect::<Vec<_>>();
    if training_samples.len() < config.key_count * 4
        || evaluation_samples.len() < config.key_count * 4
    {
        return Err("selected key groups have too few train/evaluation examples".to_owned());
    }
    if groups.len() != 8 || training_samples.len() != 58 || evaluation_samples.len() != 56 {
        return Err(format!(
            "registered repeated-key selection differs: {} keys, {} training, {} evaluation",
            groups.len(),
            training_samples.len(),
            evaluation_samples.len()
        ));
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
    let mut memory_trainer = CpuMemoryWeightTrainer::from_artifact_cached(
        &memory_artifact,
        training_samples.clone(),
        MemoryWeightConfig {
            step_q24: 1 << 20,
            parameter_limit_q24: 1 << 24,
            train_opening: true,
            train_reply: true,
        },
    )
    .map_err(|error| error.to_string())?;
    let memory_started = Instant::now();
    let mut memory_steps = Vec::new();
    for _ in 0..config.memory_steps {
        memory_steps.push(
            memory_trainer
                .train_step()
                .map_err(|error| error.to_string())?,
        );
        check_stop(memory_started, 20 * 60, "memory preconditioning")?;
    }
    if memory_steps.iter().filter(|step| step.updated).count() != config.memory_steps {
        return Err("memory preconditioning did not accept every registered step".to_owned());
    }
    let gain_baseline = memory_trainer.artifact().to_vec();
    let gain_baseline_sha256 = uor_r4_discrete::corpus::sha256(&gain_baseline);
    if gain_baseline_sha256 != config.expected_memory_sha256 {
        return Err(format!(
            "memory baseline SHA-256 differs: got {gain_baseline_sha256}"
        ));
    }
    write_new(&config.out.join("memory-baseline.uorlut"), &memory_artifact)?;
    write_new(&config.out.join("gain-baseline.uorlut"), &gain_baseline)?;

    let gain_config = ServedGainConfig { step_code: 1 };
    let mut trainer = if config.reference {
        CpuServedGainTrainer::from_artifact(&gain_baseline, training_samples.clone(), gain_config)
    } else {
        CpuServedGainTrainer::from_artifact_cached(
            &gain_baseline,
            training_samples.clone(),
            gain_config,
        )
    }
    .map_err(|error| error.to_string())?;
    let train_before = trainer.reading().map_err(|error| error.to_string())?;
    let frozen = evaluate_memory_weight_samples(&gain_baseline, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let memory_off = evaluate_memory_weight_samples(&artifact, &evaluation_samples, false)
        .map_err(|error| error.to_string())?;
    let gain_started = Instant::now();
    let mut accepted = Vec::new();
    for _ in 0..config.steps {
        let step = trainer.train_step().map_err(|error| error.to_string())?;
        if step.updated {
            accepted.push(step);
        }
        check_stop(gain_started, 15 * 60, "final-gain candidate sweep")?;
        check_stop(started, 20 * 60, "registered run")?;
    }
    let train_after = trainer.reading().map_err(|error| error.to_string())?;
    let trained_artifact = trainer.artifact().to_vec();
    write_new(&config.out.join("trained.uorlut"), &trained_artifact)?;
    let (random_artifact, random_moves) =
        random_gain_control(&gain_baseline, accepted.len(), config.seed)
            .map_err(|error| error.to_string())?;
    write_new(&config.out.join("random.uorlut"), &random_artifact)?;
    let trained = evaluate_memory_weight_samples(&trained_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let random = evaluate_memory_weight_samples(&random_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let trained_off = evaluate_memory_weight_samples(&trained_artifact, &evaluation_samples, false)
        .map_err(|error| error.to_string())?;

    let parity = [
        ("frozen", &gain_baseline, true),
        ("trained", &trained_artifact, true),
        ("random", &random_artifact, true),
        ("memory_off", &artifact, false),
        ("trained_memory_off", &trained_artifact, false),
    ]
    .into_iter()
    .map(|(name, bytes, enabled)| {
        audit_memory_weight_samples(bytes, &evaluation_samples, enabled)
            .map(|reading| (name, reading))
            .map_err(|error| error.to_string())
    })
    .collect::<Result<BTreeMap<_, _>, _>>()?;
    for (name, reading) in &parity {
        if !reading.is_exact() {
            return Err(format!("{name} lost CPU/serving parity"));
        }
    }
    let frozen_rows = decision_correctness(&gain_baseline, &evaluation_samples, true)?;
    let trained_rows = decision_correctness(&trained_artifact, &evaluation_samples, true)?;
    let random_rows = decision_correctness(&random_artifact, &evaluation_samples, true)?;
    let frozen_only = frozen_rows
        .iter()
        .zip(&trained_rows)
        .filter(|(frozen, trained)| **frozen && !**trained)
        .count();
    let trained_only = frozen_rows
        .iter()
        .zip(&trained_rows)
        .filter(|(frozen, trained)| !**frozen && **trained)
        .count();
    let top1_delta = trained_only as i64 - frozen_only as i64;
    let three_sigma = 3.0 * ((frozen_only + trained_only) as f64).sqrt();
    let frozen_top1 = frozen_rows.iter().filter(|&&correct| correct).count() as u64;
    let random_top1 = random_rows.iter().filter(|&&correct| correct).count() as u64;
    let trained_top1 = trained.top1_correct;
    let distance_pass = trained.target_distance_sum < frozen.target_distance_sum
        && trained.target_distance_sum < random.target_distance_sum;
    let top1_nonfall = trained_top1 >= frozen_top1 && trained_top1 >= random_top1;
    let ranking_pass = top1_delta as f64 > three_sigma;
    let activity_pass = !accepted.is_empty() && random_moves.len() == accepted.len();
    let instrument_pass = parity.values().all(|reading| reading.is_exact());
    let pass = instrument_pass && activity_pass && distance_pass && top1_nonfall && ranking_pass;
    check_stop(started, 20 * 60, "registered run")?;
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-gain-panel-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
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
        "selection": {
            "key_tokens": config.key_tokens,
            "key_count": groups.len(),
            "per_key": config.per_key,
            "training_documents": groups.iter().flat_map(|g| g.training.iter()).map(|s| s.document_ordinal).collect::<BTreeSet<_>>().len(),
            "evaluation_documents": groups.iter().flat_map(|g| g.evaluation.iter()).map(|s| s.document_ordinal).collect::<BTreeSet<_>>().len(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "keys": groups.iter().map(|group| serde_json::json!({
                "key": group.key,
                "training": group.training.len(),
                "evaluation": group.evaluation.len(),
            })).collect::<Vec<_>>(),
        },
        "memory_training": {
            "steps": config.memory_steps,
            "accepted_updates": memory_steps.iter().filter(|step| step.updated).count(),
            "accepted": memory_steps,
            "memory_artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "gain_baseline_sha256": gain_baseline_sha256,
        },
        "gain_training": {
            "threads": rayon::current_num_threads(),
            "search_mode": if config.reference { "full_forward" } else { "cached_final_gain" },
            "step_code": gain_config.step_code,
            "steps": config.steps,
            "accepted_updates": accepted.len(),
            "accepted": accepted,
            "before": train_before,
            "after": train_after,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&trained_artifact),
            "final_gain": read_gain(&trained_artifact).map_err(|error| error.to_string())?,
        },
        "random_control": {
            "seed": config.seed,
            "moves": random_moves,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
        },
        "evaluation": {
            "frozen": frozen,
            "trained": trained,
            "random": random,
            "memory_off": memory_off,
            "trained_memory_off": trained_off,
        },
        "parity": parity,
        "bar": {
            "instrument": if instrument_pass { "pass" } else { "fail" },
            "training_activity": if activity_pass { "pass" } else { "fail" },
            "distance_below_frozen_and_random": if distance_pass { "pass" } else { "fail" },
            "top1_nonfall": if top1_nonfall { "pass" } else { "fail" },
            "paired_top1_delta": top1_delta,
            "discordant_frozen_only": frozen_only,
            "discordant_trained_only": trained_only,
            "three_sigma_threshold": three_sigma,
            "three_sigma_ranking_gain": if ranking_pass { "pass" } else { "fail" },
            "verdict": if pass { "PASS" } else { "NEGATIVE" }
        },
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib(),
    });
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    write_new(&config.out.join("report.json"), &bytes)?;
    Ok(report)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::File::create_new(path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("write {}: {error}", path.display()))
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

fn decision_correctness(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<Vec<bool>, String> {
    let mut correctness = Vec::new();
    for sample in samples {
        let mut forward =
            CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        for index in 0..tokens.len() - 1 {
            if memory_enabled && index + 1 == sample.prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward
                .step(tokens[index], CpuFbsHead::Writer)
                .map_err(|error| error.to_string())?;
            correctness.push(step.token == tokens[index + 1]);
        }
    }
    Ok(correctness)
}

fn check_stop(started: Instant, limit_seconds: u64, name: &str) -> Result<(), String> {
    if started.elapsed().as_secs() > limit_seconds {
        return Err(format!("{name} exceeded its registered stop"));
    }
    if peak_rss_kib() > 1024 * 1024 {
        return Err(format!("{name} exceeded the 1 GiB RSS stop"));
    }
    Ok(())
}

fn peak_rss_kib() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}
