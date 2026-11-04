use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_termination_point_gates,
    derive_turn_memory, evaluate_memory_target_replies, evaluate_memory_weight_samples,
    evaluate_replies, evaluate_replies_memory_off, evaluate_termination_diagnostics,
    evaluate_termination_gate_replies, memory_key_coverage, memory_target_shuffle_audit,
    normalized_reference, replace_memory_weight_table, shuffle_memory_targets,
    validate_corpus_manifest, CpuConversationSample, CpuMaskedCorpus, CpuMemoryWeightTrainer,
    MemoryWeightConfig, MemoryWeightSample, MemoryWeightTable, TimedReplyReading,
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
    reply_cap: usize,
    termination_diagnostic: bool,
    termination_gate: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
    expected_trained_sha256: Option<String>,
    expected_no_gate_reply_sha256: Option<String>,
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
     [--expected-trained-sha256 HEX] [--expected-no-gate-reply-sha256 HEX] \
     [--steps N] [--keys N] [--per-key N] [--key-tokens N] \
     [--reply-turn N] [--reply-cap N] [--seed N] [--select-only] [--reference] \
     [--termination-diagnostic] [--termination-gate]"
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
        reply_cap: 0,
        termination_diagnostic: false,
        termination_gate: false,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
        expected_trained_sha256: None,
        expected_no_gate_reply_sha256: None,
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
        if flag == "--termination-diagnostic" {
            config.termination_diagnostic = true;
            continue;
        }
        if flag == "--termination-gate" {
            config.termination_gate = true;
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
            "--reply-cap" => config.reply_cap = number(&value)?,
            "--seed" => config.seed = number::<u64>(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            "--expected-trained-sha256" => config.expected_trained_sha256 = Some(hex(&value)?),
            "--expected-no-gate-reply-sha256" => {
                config.expected_no_gate_reply_sha256 = Some(hex(&value)?);
            }
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
        || config.reply_cap > 64
        || ((config.termination_diagnostic || config.termination_gate) && config.reply_cap == 0)
        || (config.termination_gate != config.expected_no_gate_reply_sha256.is_some())
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
            "evaluation_reference_shape": reference_shape(
                &evaluation_samples,
                config.reply_cap,
                shape.context,
            ),
            "memory_key_coverage": memory_key_coverage(
                &training_samples,
                &evaluation_samples,
                config.key_tokens,
                shape.context,
                config.reply_turn,
            ).map_err(|error| error.to_string())?,
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
    let trained_sha256 = uor_r4_discrete::corpus::sha256(&trained_artifact);
    if config
        .expected_trained_sha256
        .as_ref()
        .is_some_and(|expected| expected != &trained_sha256)
    {
        return Err(format!(
            "trained artifact SHA-256 differs: got {trained_sha256}"
        ));
    }
    std::fs::write(config.out.join("trained.uorlut"), &trained_artifact)
        .map_err(|error| error.to_string())?;
    let random_artifact =
        random_control(&memory_artifact, accepted.len(), config.seed, shape.width)
            .map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("random.uorlut"), &random_artifact)
        .map_err(|error| error.to_string())?;
    let (shuffled_artifact, shuffle_audit) = if config.reply_cap == 0 {
        (None, None)
    } else {
        let shuffled_memory =
            shuffle_memory_targets(&memory, config.seed).map_err(|error| error.to_string())?;
        let audit = memory_target_shuffle_audit(
            &memory,
            &shuffled_memory,
            &evaluation_samples,
            config.key_tokens,
            shape.context,
            config.reply_turn,
        )
        .map_err(|error| error.to_string())?;
        if audit.opening_changed < audit.opening_exercised.div_ceil(2)
            || audit.reply_changed < audit.reply_exercised.div_ceil(2)
        {
            return Err(format!(
                "shuffled target control changed too few exercised mappings: {audit:?}"
            ));
        }
        let mut shuffled = add_turn_memory_uorlut01(&artifact, &shuffled_memory)
            .map_err(|error| error.to_string())?;
        let trained_memory = read_memory_uorlut01(&trained_artifact)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "trained artifact lost turn memory".to_owned())?;
        shuffled = replace_memory_weight_table(
            &shuffled,
            MemoryWeightTable::Opening,
            &trained_memory.opening.weight,
        )
        .map_err(|error| error.to_string())?;
        if let Some(reply) = &trained_memory.reply {
            shuffled =
                replace_memory_weight_table(&shuffled, MemoryWeightTable::Reply, &reply.weight)
                    .map_err(|error| error.to_string())?;
        }
        std::fs::write(config.out.join("target-shuffled.uorlut"), &shuffled)
            .map_err(|error| error.to_string())?;
        (Some(shuffled), Some(audit))
    };
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
    let complete_replies = if config.reply_cap == 0 {
        BTreeMap::new()
    } else {
        let mut complete_replies = [
            ("frozen", memory_artifact.as_slice(), true),
            ("trained", trained_artifact.as_slice(), true),
            ("random", random_artifact.as_slice(), true),
            ("memory_off", artifact.as_slice(), false),
            ("trained_memory_off", trained_artifact.as_slice(), false),
        ]
        .into_iter()
        .map(|(name, bytes, memory_enabled)| {
            let reading = if memory_enabled {
                evaluate_replies(bytes, &evaluation_samples, config.reply_cap)
            } else {
                evaluate_replies_memory_off(bytes, &evaluation_samples, config.reply_cap)
            };
            reading
                .map_err(|error| error.to_string())
                .map(|reading| (name, reading))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
        for (name, bytes) in [
            ("target_frozen", memory_artifact.as_slice()),
            ("target_trained", trained_artifact.as_slice()),
            (
                "target_shuffled",
                shuffled_artifact.as_deref().ok_or_else(|| {
                    "reply evaluation lost its shuffled target artifact".to_owned()
                })?,
            ),
        ] {
            complete_replies.insert(
                name,
                evaluate_memory_target_replies(bytes, &evaluation_samples, config.reply_cap)
                    .map_err(|error| error.to_string())?,
            );
        }
        complete_replies
    };
    let paired_reply_comparisons = if complete_replies.is_empty() {
        BTreeMap::new()
    } else {
        let comparison = |candidate: &str, control: &str| -> Result<_, String> {
            paired_reply_comparison(
                complete_replies
                    .get(candidate)
                    .ok_or_else(|| format!("missing reply arm {candidate}"))?,
                complete_replies
                    .get(control)
                    .ok_or_else(|| format!("missing reply arm {control}"))?,
            )
        };
        [
            ("trained_vs_frozen", "trained", "frozen"),
            ("trained_vs_random", "trained", "random"),
            ("trained_vs_memory_off", "trained", "memory_off"),
            ("target_trained_vs_trained", "target_trained", "trained"),
            (
                "target_trained_vs_target_shuffled",
                "target_trained",
                "target_shuffled",
            ),
        ]
        .into_iter()
        .map(|(name, candidate, control)| {
            comparison(candidate, control).map(|reading| (name, reading))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?
    };
    let termination_diagnostics = if !config.termination_diagnostic {
        BTreeMap::new()
    } else {
        [
            ("frozen", memory_artifact.as_slice(), true),
            ("trained", trained_artifact.as_slice(), true),
            ("memory_off", artifact.as_slice(), false),
            ("trained_memory_off", trained_artifact.as_slice(), false),
        ]
        .into_iter()
        .map(|(name, bytes, memory_enabled)| {
            evaluate_termination_diagnostics(
                bytes,
                &evaluation_samples,
                config.reply_cap,
                memory_enabled,
            )
            .map(|reading| (name, reading))
            .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?
    };
    let termination_gate = if !config.termination_gate {
        serde_json::Value::Null
    } else {
        let derivation =
            derive_termination_point_gates(&trained_artifact, &training_samples, true, config.seed)
                .map_err(|error| error.to_string())?;
        let no_gate = complete_replies
            .get("trained")
            .ok_or_else(|| "termination gate lost the trained no-gate arm".to_owned())?;
        let expected_no_gate = config
            .expected_no_gate_reply_sha256
            .as_ref()
            .ok_or_else(|| "termination gate lost its no-gate reply pin".to_owned())?;
        if &no_gate.reply_sha256 != expected_no_gate {
            return Err(format!(
                "no-gate reply SHA-256 differs: got {}",
                no_gate.reply_sha256
            ));
        }
        let trained_gate = evaluate_termination_gate_replies(
            &trained_artifact,
            &evaluation_samples,
            config.reply_cap,
            true,
            &derivation.trained,
        )
        .map_err(|error| error.to_string())?;
        let random_gate = evaluate_termination_gate_replies(
            &trained_artifact,
            &evaluation_samples,
            config.reply_cap,
            true,
            &derivation.random,
        )
        .map_err(|error| error.to_string())?;
        let trained_vs_no_gate = termination_gate_comparison(&trained_gate, no_gate)?;
        let trained_vs_random = termination_gate_comparison(&trained_gate, &random_gate)?;
        serde_json::json!({
            "random_seed": config.seed,
            "artifact_sha256": trained_sha256,
            "derivation": derivation,
            "trained_gate_sha256": termination_gate_sha256(&derivation.trained)?,
            "random_gate_sha256": termination_gate_sha256(&derivation.random)?,
            "no_gate": no_gate,
            "trained_gate": trained_gate,
            "random_gate": random_gate,
            "paired": {
                "trained_vs_no_gate": trained_vs_no_gate,
                "trained_vs_random": trained_vs_random,
            },
        })
    };
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-memory-panel-run/2",
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
        "complete_reply_artifacts": {
            "frozen": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "trained": uor_r4_discrete::corpus::sha256(&trained_artifact),
            "random": uor_r4_discrete::corpus::sha256(&random_artifact),
            "memory_off": uor_r4_discrete::corpus::sha256(&artifact),
            "trained_memory_off": uor_r4_discrete::corpus::sha256(&trained_artifact),
            "target_frozen": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "target_trained": uor_r4_discrete::corpus::sha256(&trained_artifact),
            "target_shuffled": shuffled_artifact
                .as_deref()
                .map(uor_r4_discrete::corpus::sha256),
        },
        "complete_replies": complete_replies,
        "paired_reply_comparisons": paired_reply_comparisons,
        "termination_diagnostics": termination_diagnostics,
        "termination_gate": termination_gate,
        "target_shuffle_audit": shuffle_audit,
        "reply_cap": config.reply_cap,
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

fn reference_shape(
    samples: &[MemoryWeightSample],
    cap: usize,
    context: usize,
) -> serde_json::Value {
    let mut histogram = BTreeMap::<usize, usize>::new();
    let mut token_total = 0usize;
    let mut max = 0usize;
    let mut within_cap = 0usize;
    let mut within_cap_and_context = 0usize;
    for sample in samples {
        let length = normalized_reference(&sample.reply).len();
        *histogram.entry(length).or_default() += 1;
        token_total += length;
        max = max.max(length);
        within_cap += usize::from(cap > 0 && length <= cap);
        within_cap_and_context += usize::from(
            cap > 0
                && length <= cap
                && sample
                    .prompt
                    .len()
                    .checked_add(length)
                    .is_some_and(|total| total <= context),
        );
    }
    serde_json::json!({
        "replies": samples.len(),
        "token_total": token_total,
        "max_tokens": max,
        "within_reply_cap": within_cap,
        "within_reply_cap_and_context": within_cap_and_context,
        "length_histogram": histogram,
    })
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

fn paired_reply_comparison(
    candidate: &TimedReplyReading,
    control: &TimedReplyReading,
) -> Result<serde_json::Value, String> {
    if candidate.rows.len() != control.rows.len() {
        return Err("paired reply arms report different row counts".to_owned());
    }
    let (mut rows, mut candidate_exact, mut control_exact, mut b, mut c) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    for (left, right) in candidate.rows.iter().zip(&control.rows) {
        if left.row != right.row {
            return Err(format!(
                "paired reply row identity differs: {} vs {}",
                left.row, right.row
            ));
        }
        if left.reachable != right.reachable {
            return Err(format!(
                "paired reply reachability differs at row {}",
                left.row
            ));
        }
        if !left.reachable {
            continue;
        }
        rows = rows
            .checked_add(1)
            .ok_or_else(|| "paired reply row count overflows".to_owned())?;
        candidate_exact = candidate_exact
            .checked_add(u64::from(left.exact))
            .ok_or_else(|| "paired candidate count overflows".to_owned())?;
        control_exact = control_exact
            .checked_add(u64::from(right.exact))
            .ok_or_else(|| "paired control count overflows".to_owned())?;
        b = b
            .checked_add(u64::from(left.exact && !right.exact))
            .ok_or_else(|| "paired candidate-only count overflows".to_owned())?;
        c = c
            .checked_add(u64::from(!left.exact && right.exact))
            .ok_or_else(|| "paired control-only count overflows".to_owned())?;
    }
    let discordant = b
        .checked_add(c)
        .ok_or_else(|| "paired discordant count overflows".to_owned())?;
    let threshold = 2u64.max(ceil_sqrt(
        discordant
            .checked_mul(4)
            .ok_or_else(|| "paired margin input overflows".to_owned())?,
    ));
    let gain = i128::from(candidate_exact) - i128::from(control_exact);
    let passes_margin = gain >= i128::from(threshold);
    let passes_bos = candidate.reading.bos_complete >= control.reading.bos_complete;
    Ok(serde_json::json!({
        "rows": rows,
        "candidate_exact": candidate_exact,
        "control_exact": control_exact,
        "candidate_only": b,
        "control_only": c,
        "discordant": discordant,
        "gain": gain,
        "threshold": threshold,
        "candidate_bos": candidate.reading.bos_complete,
        "control_bos": control.reading.bos_complete,
        "passes_margin": passes_margin,
        "passes_bos": passes_bos,
        "passes": passes_margin && passes_bos,
    }))
}

fn termination_gate_comparison(
    candidate: &TimedReplyReading,
    control: &TimedReplyReading,
) -> Result<serde_json::Value, String> {
    let paired = paired_reply_comparison(candidate, control)?;
    let candidate_bos = candidate.reading.bos_complete;
    let control_bos = control.reading.bos_complete;
    let candidate_non_premature_bos = candidate
        .reading
        .bos_complete
        .checked_sub(candidate.reading.premature_stop)
        .ok_or_else(|| "candidate premature stops exceed BOS completions".to_owned())?;
    let control_non_premature_bos = control
        .reading
        .bos_complete
        .checked_sub(control.reading.premature_stop)
        .ok_or_else(|| "control premature stops exceed BOS completions".to_owned())?;
    let non_premature_bos_strictly_higher = candidate_non_premature_bos > control_non_premature_bos;
    let exact_not_lower = candidate.reading.exact_reference >= control.reading.exact_reference;
    Ok(serde_json::json!({
        "paired_exact": paired,
        "candidate_bos": candidate_bos,
        "control_bos": control_bos,
        "candidate_non_premature_bos": candidate_non_premature_bos,
        "control_non_premature_bos": control_non_premature_bos,
        "candidate_exact": candidate.reading.exact_reference,
        "control_exact": control.reading.exact_reference,
        "candidate_premature_stop": candidate.reading.premature_stop,
        "control_premature_stop": control.reading.premature_stop,
        "non_premature_bos_strictly_higher": non_premature_bos_strictly_higher,
        "exact_not_lower": exact_not_lower,
        "passes": non_premature_bos_strictly_higher && exact_not_lower,
    }))
}

fn termination_gate_sha256(
    gate: &uor_r4_cpu_train::TerminationPointGate,
) -> Result<String, String> {
    serde_json::to_vec(gate)
        .map(|bytes| uor_r4_discrete::corpus::sha256(&bytes))
        .map_err(|error| error.to_string())
}

fn ceil_sqrt(value: u64) -> u64 {
    let mut root = (value as f64).sqrt() as u64;
    while u128::from(root) * u128::from(root) < u128::from(value) {
        root += 1;
    }
    while root > 0 && u128::from(root - 1) * u128::from(root - 1) >= u128::from(value) {
        root -= 1;
    }
    root
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

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_cpu_train::{ReplyReading, ReplyRowReading};
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{
        export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, POINT_KEY_BOS,
    };

    fn bare_artifact() -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            931,
            FbsSynth {
                vocab: 4096,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        snapshot.memory = None;
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    #[test]
    fn reference_shape_normalizes_bos_and_counts_reachability() {
        let samples = vec![
            MemoryWeightSample {
                prompt: vec![10, 11],
                reply: vec![20, 21, 22, POINT_KEY_BOS],
            },
            MemoryWeightSample {
                prompt: vec![10, 11],
                reply: vec![30, 31, 32, 33, 34],
            },
        ];
        let shape = reference_shape(&samples, 4, 6);
        assert_eq!(shape["replies"], 2);
        assert_eq!(shape["token_total"], 8);
        assert_eq!(shape["max_tokens"], 5);
        assert_eq!(shape["within_reply_cap"], 1);
        assert_eq!(shape["within_reply_cap_and_context"], 1);
        assert_eq!(shape["length_histogram"]["3"], 1);
        assert_eq!(shape["length_histogram"]["5"], 1);
    }

    #[test]
    fn reference_shape_normalizes_corpus_eos() {
        let samples = vec![MemoryWeightSample {
            prompt: vec![10, 11],
            reply: vec![20, 21, 22, 1],
        }];
        let shape = reference_shape(&samples, 3, 5);
        assert_eq!(shape["token_total"], 3);
        assert_eq!(shape["max_tokens"], 3);
        assert_eq!(shape["within_reply_cap"], 1);
        assert_eq!(shape["within_reply_cap_and_context"], 1);
        assert_eq!(shape["length_histogram"]["3"], 1);
    }

    #[test]
    fn paired_reply_margin_uses_asymmetric_disagreements_and_skips_unreachable() {
        let rows = vec![
            ReplyRowReading {
                row: 0,
                reachable: true,
                exact: true,
            },
            ReplyRowReading {
                row: 1,
                reachable: true,
                exact: false,
            },
            ReplyRowReading {
                row: 2,
                reachable: true,
                exact: true,
            },
            ReplyRowReading {
                row: 3,
                reachable: false,
                exact: false,
            },
            ReplyRowReading {
                row: 4,
                reachable: true,
                exact: true,
            },
        ];
        let control_rows = vec![
            ReplyRowReading {
                row: 0,
                reachable: true,
                exact: false,
            },
            ReplyRowReading {
                row: 1,
                reachable: true,
                exact: true,
            },
            ReplyRowReading {
                row: 2,
                reachable: true,
                exact: true,
            },
            ReplyRowReading {
                row: 3,
                reachable: false,
                exact: true,
            },
            ReplyRowReading {
                row: 4,
                reachable: true,
                exact: false,
            },
        ];
        let reading = |rows: Vec<ReplyRowReading>, bos: u64| TimedReplyReading {
            rows,
            reading: ReplyReading {
                replies: 5,
                bos_complete: bos,
                ..ReplyReading::default()
            },
            reply_sha256: String::new(),
            wall_elapsed_us: 0,
            generated_tokens_per_second: 0.0,
        };
        let comparison = paired_reply_comparison(&reading(rows, 2), &reading(control_rows, 1))
            .expect("the paired reading is valid");
        assert_eq!(comparison["rows"], 4);
        assert_eq!(comparison["candidate_exact"], 3);
        assert_eq!(comparison["control_exact"], 2);
        assert_eq!(comparison["candidate_only"], 2);
        assert_eq!(comparison["control_only"], 1);
        assert_eq!(comparison["discordant"], 3);
        assert_eq!(comparison["threshold"], 4);
        assert_eq!(comparison["gain"], 1);
        assert_eq!(comparison["passes_margin"], false);
        assert_eq!(comparison["passes_bos"], true);
        assert_eq!(comparison["passes"], false);
    }

    #[test]
    fn termination_gate_requires_strict_bos_gain_without_exact_loss() {
        let rows = vec![ReplyRowReading {
            row: 0,
            reachable: true,
            exact: true,
        }];
        let reading = |bos: u64| TimedReplyReading {
            rows: rows.clone(),
            reading: ReplyReading {
                replies: 1,
                bos_complete: bos,
                exact_reference: 1,
                premature_stop: 0,
                ..ReplyReading::default()
            },
            reply_sha256: String::new(),
            wall_elapsed_us: 0,
            generated_tokens_per_second: 0.0,
        };
        let wins = termination_gate_comparison(&reading(1), &reading(0)).unwrap();
        assert_eq!(wins["passes"], true);
        assert_eq!(wins["non_premature_bos_strictly_higher"], true);
        assert_eq!(wins["exact_not_lower"], true);

        let tie = termination_gate_comparison(&reading(1), &reading(1)).unwrap();
        assert_eq!(tie["passes"], false);
        assert_eq!(tie["non_premature_bos_strictly_higher"], false);

        let premature = TimedReplyReading {
            rows: rows.clone(),
            reading: ReplyReading {
                replies: 1,
                bos_complete: 1,
                premature_stop: 1,
                ..ReplyReading::default()
            },
            reply_sha256: String::new(),
            wall_elapsed_us: 0,
            generated_tokens_per_second: 0.0,
        };
        let all_premature = termination_gate_comparison(&premature, &reading(0)).unwrap();
        assert_eq!(all_premature["candidate_non_premature_bos"], 0);
        assert_eq!(all_premature["passes"], false);
    }

    #[test]
    fn shuffled_target_control_changes_values_but_not_keys() {
        let memory = FbsMemorySnapshot {
            k: 3,
            key_window: 16,
            opening: vec![
                FbsMemoryEntry {
                    key: vec![10, 11, 12],
                    value: 1,
                },
                FbsMemoryEntry {
                    key: vec![10, 11, 13],
                    value: 2,
                },
                FbsMemoryEntry {
                    key: vec![10, 11, 14],
                    value: 3,
                },
                FbsMemoryEntry {
                    key: vec![10, 11, 15],
                    value: 4,
                },
            ],
            opening_weight: vec![0.0; 64],
            reply: Some(FbsReplyTable {
                turn: 3,
                entries: vec![
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, u32::MAX, 1, u32::MAX, 20],
                        value: 5,
                    },
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, u32::MAX, 2, u32::MAX, 21],
                        value: 6,
                    },
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, u32::MAX, 3, u32::MAX, 22],
                        value: 7,
                    },
                    FbsMemoryEntry {
                        key: vec![10, 11, 12, u32::MAX, 4, u32::MAX, 23],
                        value: 8,
                    },
                ],
                weight: vec![0.0; 64],
            }),
        };
        let shuffled = shuffle_memory_targets(&memory, 17).unwrap();
        let keys = |table: &FbsMemorySnapshot| {
            table
                .opening
                .iter()
                .map(|entry| entry.key.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(&shuffled), keys(&memory));
        let mut before = memory
            .opening
            .iter()
            .map(|entry| entry.value)
            .collect::<Vec<_>>();
        let mut after = shuffled
            .opening
            .iter()
            .map(|entry| entry.value)
            .collect::<Vec<_>>();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(before, after);
        assert_ne!(shuffled.opening, memory.opening);
        assert_ne!(
            shuffled.reply.as_ref().unwrap().entries,
            memory.reply.as_ref().unwrap().entries
        );

        let artifact = bare_artifact();
        let opening_weight = (0..64)
            .map(|coordinate| (coordinate - 32) << 12)
            .collect::<Vec<_>>();
        let reply_weight = (0..64)
            .map(|coordinate| (31 - coordinate) << 12)
            .collect::<Vec<_>>();
        let mut original = add_turn_memory_uorlut01(&artifact, &memory).unwrap();
        original =
            replace_memory_weight_table(&original, MemoryWeightTable::Opening, &opening_weight)
                .unwrap();
        original = replace_memory_weight_table(&original, MemoryWeightTable::Reply, &reply_weight)
            .unwrap();
        let mut control = add_turn_memory_uorlut01(&artifact, &shuffled).unwrap();
        control =
            replace_memory_weight_table(&control, MemoryWeightTable::Opening, &opening_weight)
                .unwrap();
        control =
            replace_memory_weight_table(&control, MemoryWeightTable::Reply, &reply_weight).unwrap();
        let original = read_memory_uorlut01(&original).unwrap().unwrap();
        let control = read_memory_uorlut01(&control).unwrap().unwrap();
        assert_eq!(original.record, control.record);
        assert_eq!(original.opening.keys, control.opening.keys);
        assert_eq!(original.opening.weight, control.opening.weight);
        assert_ne!(original.opening.values, control.opening.values);
        assert_eq!(original.opening.weight, opening_weight);
        assert_eq!(
            original.reply.as_ref().unwrap().keys,
            control.reply.as_ref().unwrap().keys
        );
        assert_eq!(
            original.reply.as_ref().unwrap().weight,
            control.reply.as_ref().unwrap().weight
        );
        assert_ne!(
            original.reply.as_ref().unwrap().values,
            control.reply.as_ref().unwrap().values
        );
        assert_eq!(original.reply.as_ref().unwrap().weight, reply_weight);
        let audit = memory_target_shuffle_audit(
            &memory,
            &shuffled,
            &[MemoryWeightSample {
                prompt: vec![10, 11, 12],
                reply: vec![20, 21, 22],
            }],
            3,
            16,
            3,
        )
        .unwrap();
        assert_eq!(audit.opening_exercised, 1);
        assert_eq!(audit.opening_changed, 1);
        assert_eq!(audit.reply_exercised, 2);
    }
}
