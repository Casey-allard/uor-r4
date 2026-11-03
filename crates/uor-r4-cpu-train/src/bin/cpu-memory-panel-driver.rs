use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory,
    evaluate_memory_weight_samples, replace_memory_weight_table, validate_corpus_manifest,
    CpuConversationSample, CpuMaskedCorpus, CpuMemoryWeightTrainer, MemoryWeightConfig,
    MemoryWeightSample, MemoryWeightTable,
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
    key_count: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    seed: u64,
    select_only: bool,
    reference: bool,
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
            eprintln!("cpu-memory-panel-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-memory-panel-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--steps N] [--keys N] [--per-key N] [--key-tokens N] \
     [--reply-turn N] [--seed N] [--select-only] [--reference]"
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
        key_count: 8,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        seed: 1,
        select_only: false,
        reference: false,
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
        if flag == "--reference" {
            config.reference = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--steps" => config.steps = number(&value)?,
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--seed" => config.seed = number::<u64>(&value)?,
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
    if config.steps == 0
        || config.key_count == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
    {
        return Err("training and group dimensions must be positive and bounded".to_owned());
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
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-memory-panel-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "key_tokens": config.key_tokens,
            "key_count": groups.len(),
            "per_key": config.per_key,
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "keys": groups.iter().map(|group| serde_json::json!({
                "key": group.key,
                "training": group.training.len(),
                "evaluation": group.evaluation.len(),
            })).collect::<Vec<_>>(),
        }));
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
    std::fs::create_dir_all(&config.out).map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("memory-baseline.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;

    let weight_config = MemoryWeightConfig {
        step_q24: 1 << 20,
        parameter_limit_q24: 1 << 24,
        train_opening: true,
        train_reply: true,
    };
    let mut trainer = if config.reference {
        CpuMemoryWeightTrainer::from_artifact(
            &memory_artifact,
            training_samples.clone(),
            weight_config,
        )
    } else {
        CpuMemoryWeightTrainer::from_artifact_cached(
            &memory_artifact,
            training_samples.clone(),
            weight_config,
        )
    }
    .map_err(|error| error.to_string())?;
    let train_before = trainer.reading().map_err(|error| error.to_string())?;
    let frozen = evaluate_memory_weight_samples(&memory_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let memory_off = evaluate_memory_weight_samples(&artifact, &evaluation_samples, false)
        .map_err(|error| error.to_string())?;
    let mut accepted = Vec::new();
    for _ in 0..config.steps {
        let step = trainer.train_step().map_err(|error| error.to_string())?;
        if step.updated {
            accepted.push(step);
        }
    }
    let train_after = trainer.reading().map_err(|error| error.to_string())?;
    let trained_artifact = trainer.artifact().to_vec();
    std::fs::write(config.out.join("trained.uorlut"), &trained_artifact)
        .map_err(|error| error.to_string())?;
    let random_artifact =
        random_control(&memory_artifact, accepted.len(), config.seed, shape.width)
            .map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("random.uorlut"), &random_artifact)
        .map_err(|error| error.to_string())?;
    let trained = evaluate_memory_weight_samples(&trained_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let random = evaluate_memory_weight_samples(&random_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let trained_off = evaluate_memory_weight_samples(&trained_artifact, &evaluation_samples, false)
        .map_err(|error| error.to_string())?;

    let parity = [
        ("frozen", &memory_artifact, true),
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
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-memory-panel-run/1",
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
            "training_documents": groups.iter().flat_map(|g| g.training.iter()).map(|s| s.document_ordinal).collect::<std::collections::BTreeSet<_>>().len(),
            "evaluation_documents": groups.iter().flat_map(|g| g.evaluation.iter()).map(|s| s.document_ordinal).collect::<std::collections::BTreeSet<_>>().len(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "keys": groups.iter().map(|group| serde_json::json!({
                "key": group.key,
                "training": group.training.len(),
                "evaluation": group.evaluation.len(),
            })).collect::<Vec<_>>(),
        },
        "memory": {
            "k": memory.k,
            "key_window": memory.key_window,
            "opening_rows": memory.opening.len(),
            "reply_rows": memory.reply.as_ref().map_or(0, |reply| reply.entries.len()),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
        },
        "training": {
            "seed": config.seed,
            "threads": rayon::current_num_threads(),
            "search_mode": if config.reference {
                "full_forward"
            } else {
                "cached_trajectory"
            },
            "steps": config.steps,
            "accepted_updates": accepted.len(),
            "accepted": accepted,
            "before": train_before,
            "after": train_after,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&trained_artifact),
        },
        "evaluation": {
            "frozen": frozen,
            "trained": trained,
            "random": random,
            "memory_off": memory_off,
            "trained_memory_off": trained_off,
        },
        "parity": parity,
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

fn random_control(
    artifact: &[u8],
    moves: usize,
    seed: u64,
    width: usize,
) -> Result<Vec<u8>, String> {
    let mut out = artifact.to_vec();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for _ in 0..moves {
        let memory = read_memory_uorlut01(&out)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "random control lost memory".to_owned())?;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let use_reply = memory.reply.is_some() && draw & 1 == 1;
        let table = if use_reply {
            MemoryWeightTable::Reply
        } else {
            MemoryWeightTable::Opening
        };
        let mut weights = match table {
            MemoryWeightTable::Opening => memory.opening.weight,
            MemoryWeightTable::Reply => memory.reply.expect("checked").weight,
        };
        let coordinate = ((draw >> 1) as usize) % width;
        let delta = if draw & 2 == 0 { 1 << 20 } else { -(1 << 20) };
        weights[coordinate] = weights[coordinate]
            .saturating_add(delta)
            .clamp(-(1 << 24), 1 << 24);
        out = replace_memory_weight_table(&out, table, &weights)
            .map_err(|error| error.to_string())?;
    }
    Ok(out)
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
