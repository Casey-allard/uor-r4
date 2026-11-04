use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory,
    evaluate_memory_weight_samples, validate_corpus_manifest, CpuConversationSample, CpuFbsForward,
    CpuFbsHead, CpuMaskedCorpus, MemoryWeightSample,
};
use uor_r4_integer::stack::{
    add_point_key_uorlut01, build_point_key, read_memory_uorlut01, IntegerStackModel,
    PointKeyImage, PointKeyWindow,
};

const SCHEMA: &str = "uor-r4.cpu-point-transfer-run/1";

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    train_keys: usize,
    evaluation_keys: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    random_seed: u64,
    neighbor_vote: usize,
    select_only: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyGroup {
    key: Vec<u32>,
    samples: Vec<CpuConversationSample>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct OpeningReading {
    decisions: u64,
    top1_correct: u64,
    target_distance_sum: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct NeighborReading {
    decisions: u64,
    nearest_top1: u64,
    vote_top1: u64,
    random_vote_top1: u64,
    nearest_matches_served: u64,
    voted_matches_served: u64,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-point-transfer-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-point-transfer-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--train-keys N] [--evaluation-keys N] [--per-key N] \
     [--key-tokens N] [--reply-turn N] [--random-seed N] \
     [--neighbor-vote N] [--select-only]"
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
        train_keys: 8,
        evaluation_keys: 8,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        random_seed: 1,
        neighbor_vote: 0,
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
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--train-keys" => config.train_keys = number(&value)?,
            "--evaluation-keys" => config.evaluation_keys = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--random-seed" => config.random_seed = number(&value)?,
            "--neighbor-vote" => config.neighbor_vote = number(&value)?,
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
    if config.train_keys == 0
        || config.evaluation_keys == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
    {
        return Err("group dimensions must be positive and bounded".to_owned());
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
    if config.out.exists() {
        return Err(format!(
            "output root already exists: {}",
            config.out.display()
        ));
    }
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
        return Err("the public source artifact already carries memory".to_owned());
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&tokens, &mask, shape.vocab, shape.context, identity.bos)
            .map_err(|error| error.to_string())?;
    let examples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let (training, evaluation) = select_transfer_groups(&examples, &config)?;
    let training_samples = training
        .iter()
        .flat_map(|group| group.samples.iter())
        .cloned()
        .collect::<Vec<_>>();
    let evaluation_samples = evaluation
        .iter()
        .flat_map(|group| group.samples.iter())
        .cloned()
        .collect::<Vec<_>>();
    let training_keys = training
        .iter()
        .map(|group| group.key.clone())
        .collect::<BTreeSet<_>>();
    if evaluation
        .iter()
        .any(|group| training_keys.contains(&group.key))
    {
        return Err("a held-out opening key overlaps training".to_owned());
    }
    if training_samples.len() < config.train_keys * 4
        || evaluation_samples.len() < config.evaluation_keys * 4
    {
        return Err("selected key groups have too few examples".to_owned());
    }
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-point-transfer-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "training_keys": training_keys,
            "evaluation_keys": evaluation.iter().map(|group| group.key.clone()).collect::<Vec<_>>(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "training_groups": training.iter().map(group_json).collect::<Vec<_>>(),
            "evaluation_groups": evaluation.iter().map(group_json).collect::<Vec<_>>(),
        }));
    }

    let weight_samples = training_samples
        .iter()
        .map(memory_sample)
        .collect::<Vec<_>>();
    let held_out = evaluation_samples
        .iter()
        .map(memory_sample)
        .collect::<Vec<_>>();
    let memory = derive_turn_memory(
        &weight_samples,
        shape.width,
        config.key_tokens,
        shape.context,
        config.reply_turn,
    )
    .map_err(|error| error.to_string())?;
    let memory_artifact =
        add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    assert_held_out_exact_keys_miss(&memory_artifact, &held_out)?;

    let point = derive_point_table(
        &memory_artifact,
        &training_samples,
        shape.width,
        shape.vocab,
    )?;
    let point_artifact =
        add_point_key_uorlut01(&memory_artifact, &point).map_err(|error| error.to_string())?;
    let voted_point = (config.neighbor_vote > 1).then(|| PointKeyImage {
        width: point.width,
        points: point.points.clone(),
        targets: point.targets.clone(),
        vote_k: config.neighbor_vote,
    });
    let voted_artifact = voted_point
        .as_ref()
        .map(|image| add_point_key_uorlut01(&memory_artifact, image))
        .transpose()
        .map_err(|error| error.to_string())?;
    let random_point = PointKeyImage {
        width: point.width,
        points: point.points.clone(),
        targets: shuffled_targets(&point.targets, config.random_seed)?,
        vote_k: 1,
    };
    let random_artifact = add_point_key_uorlut01(&memory_artifact, &random_point)
        .map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&config.out).map_err(|error| error.to_string())?;
    let mut outputs = vec![
        ("memory-only.uorlut", memory_artifact.as_slice()),
        ("point.uorlut", point_artifact.as_slice()),
        ("random-point.uorlut", random_artifact.as_slice()),
    ];
    if let Some(bytes) = &voted_artifact {
        outputs.push(("point-vote.uorlut", bytes.as_slice()));
    }
    for (name, bytes) in outputs {
        std::fs::write(config.out.join(name), bytes).map_err(|error| error.to_string())?;
    }

    let mut arms = vec![
        ("point_off", memory_artifact.as_slice()),
        ("point", point_artifact.as_slice()),
        ("random", random_artifact.as_slice()),
    ];
    if let Some(bytes) = &voted_artifact {
        arms.push(("point_vote", bytes.as_slice()));
    }
    let opening = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            evaluate_openings(bytes, &evaluation_samples, shape.width)
                .map(|reading| (name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let opening_distances = opening
        .values()
        .map(|reading| reading.target_distance_sum)
        .collect::<BTreeSet<_>>();
    if opening_distances.len() != 1 {
        return Err("point fallback changed the opening query distance".to_owned());
    }
    let neighbor = if config.neighbor_vote == 0 {
        None
    } else {
        if config.neighbor_vote > point.rows() {
            return Err("neighbor vote exceeds the point-table row count".to_owned());
        }
        Some(
            evaluate_neighbor_vote(
                &memory_artifact,
                &point_artifact,
                voted_artifact.as_deref(),
                &point,
                &random_point,
                &evaluation_samples,
                config.neighbor_vote,
            )
            .map_err(|error| error.to_string())?,
        )
    };
    if neighbor.is_some_and(|reading| {
        reading.nearest_matches_served != reading.decisions
            || reading.voted_matches_served != reading.decisions
    }) {
        return Err("point extraction differs from the served point key".to_owned());
    }
    let full = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            evaluate_memory_weight_samples(bytes, &held_out, true)
                .map(|reading| (name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let parity = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            audit_memory_weight_samples(bytes, &held_out, true)
                .map(|reading| (name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    for (name, reading) in &parity {
        if !reading.is_exact() {
            return Err(format!("{name} lost CPU/serving parity"));
        }
    }
    let baseline_hits = opening["point_off"].top1_correct;
    let n = opening["point_off"].decisions;
    let p = baseline_hits as f64 / n as f64;
    let sigma = 3.0 * (n as f64 * p * (1.0 - p)).sqrt();
    let threshold = sigma.ceil() as u64;
    let neighbor_three_sigma = neighbor.map(|reading| {
        let p = reading.nearest_top1 as f64 / reading.decisions as f64;
        let threshold = (3.0 * (reading.decisions as f64 * p * (1.0 - p)).sqrt()).ceil() as u64;
        serde_json::json!({
            "baseline_nearest": reading.nearest_top1,
            "decisions": reading.decisions,
            "vote_delta": reading.vote_top1 as i64 - reading.nearest_top1 as i64,
            "threshold": threshold,
        })
    });
    let report = serde_json::json!({
        "schema": SCHEMA,
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
            "per_key": config.per_key,
            "training_keys": training_keys,
            "evaluation_keys": evaluation.iter().map(|group| group.key.clone()).collect::<Vec<_>>(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "training_documents": training_samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>().len(),
            "evaluation_documents": evaluation_samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>().len(),
        },
        "memory": {
            "k": memory.k,
            "key_window": memory.key_window,
            "opening_rows": memory.opening.len(),
            "reply_rows": memory.reply.as_ref().map_or(0, |reply| reply.entries.len()),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "held_out_exact_key_hits": 0,
        },
        "point": {
            "rows": point.rows(),
            "width": point.width,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&point_artifact),
            "voted_artifact_sha256": voted_artifact
                .as_ref()
                .map(|bytes| uor_r4_discrete::corpus::sha256(bytes)),
            "random_artifact_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
            "random_seed": config.random_seed,
        },
        "neighbor_vote": {
            "k": config.neighbor_vote,
            "reading": neighbor,
        },
        "opening": opening,
        "full_teacher_forced": full,
        "parity": parity,
        "three_sigma": {
            "baseline_top1": baseline_hits,
            "decisions": n,
            "point_delta": opening["point"].top1_correct as i64 - baseline_hits as i64,
            "random_delta": opening["random"].top1_correct as i64 - baseline_hits as i64,
            "threshold": threshold,
        },
        "neighbor_three_sigma": neighbor_three_sigma,
        "threads": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib(),
        "claim": "CPU development-research transfer test; not GPU qualification or language capability",
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

fn group_json(group: &KeyGroup) -> serde_json::Value {
    serde_json::json!({
        "key": group.key,
        "samples": group.samples.len(),
        "documents": group.samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>(),
    })
}

fn memory_sample(sample: &CpuConversationSample) -> MemoryWeightSample {
    MemoryWeightSample {
        prompt: sample.prompt.clone(),
        reply: sample.reply.clone(),
    }
}

fn opening_key(sample: &CpuConversationSample, key_tokens: usize) -> Vec<u32> {
    sample.prompt.iter().copied().take(key_tokens).collect()
}

fn select_transfer_groups(
    examples: &[CpuConversationSample],
    config: &Config,
) -> Result<(Vec<KeyGroup>, Vec<KeyGroup>), String> {
    let mut training = BTreeMap::<Vec<u32>, Vec<CpuConversationSample>>::new();
    let mut evaluation = BTreeMap::<Vec<u32>, Vec<CpuConversationSample>>::new();
    for sample in examples {
        let key = opening_key(sample, config.key_tokens);
        if sample.document_ordinal % 2 == 0 {
            training.entry(key).or_default().push(sample.clone());
        } else {
            evaluation.entry(key).or_default().push(sample.clone());
        }
    }
    let mut training = select_groups(training, config.train_keys, config.per_key);
    let training_keys = training
        .iter()
        .map(|group| group.key.clone())
        .collect::<BTreeSet<_>>();
    let evaluation = evaluation
        .into_iter()
        .filter(|(key, samples)| !training_keys.contains(key) && samples.len() >= 4)
        .collect::<BTreeMap<_, _>>();
    let evaluation = select_groups(evaluation, config.evaluation_keys, config.per_key);
    if training.is_empty() || evaluation.is_empty() {
        return Err("no disjoint training/evaluation opening-key groups were found".to_owned());
    }
    training.sort_by(|left, right| left.key.cmp(&right.key));
    Ok((training, evaluation))
}

fn select_groups(
    groups: BTreeMap<Vec<u32>, Vec<CpuConversationSample>>,
    count: usize,
    per_key: usize,
) -> Vec<KeyGroup> {
    let mut groups = groups
        .into_iter()
        .filter_map(|(key, mut samples)| {
            if samples.len() < 4 {
                return None;
            }
            samples.truncate(per_key);
            Some(KeyGroup { key, samples })
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .samples
            .len()
            .cmp(&left.samples.len())
            .then_with(|| right.key.cmp(&left.key))
    });
    groups.truncate(count);
    groups
}

fn derive_point_table(
    memory_artifact: &[u8],
    training: &[CpuConversationSample],
    width: usize,
    vocab: usize,
) -> Result<PointKeyImage, String> {
    let model = IntegerStackModel::parse(memory_artifact).map_err(|error| error.to_string())?;
    let windows = training
        .iter()
        .map(|sample| {
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "point training sample has no reply".to_owned())?;
            let at = sample
                .prompt
                .len()
                .checked_sub(1)
                .ok_or_else(|| "point training prompt is empty".to_owned())?;
            Ok(PointKeyWindow {
                input: sample.prompt.clone(),
                openings: vec![(at, target)],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let built = build_point_key(&model, &windows).map_err(|error| error.to_string())?;
    let mut targets = BTreeMap::<Vec<i16>, BTreeMap<u32, usize>>::new();
    for (point, &target) in built.points.chunks_exact(width).zip(&built.targets) {
        let point = point.to_vec();
        *targets.entry(point).or_default().entry(target).or_default() += 1;
    }
    let rows = targets
        .into_iter()
        .map(|(point, counts)| {
            let target = counts
                .into_iter()
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                .map(|(target, _)| target)
                .expect("a point row has a target");
            (point, target)
        })
        .collect::<Vec<_>>();
    let image = PointKeyImage {
        width,
        points: rows
            .iter()
            .flat_map(|(point, _)| point.iter().copied())
            .collect(),
        targets: rows.into_iter().map(|(_, target)| target).collect(),
        vote_k: 1,
    };
    image.check(vocab).map_err(|error| error.to_string())?;
    Ok(image)
}

fn shuffled_targets(targets: &[u32], seed: u64) -> Result<Vec<u32>, String> {
    let mut out = targets.to_vec();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for index in (1..out.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        out.swap(index, (draw as usize) % (index + 1));
    }
    if out == targets {
        out.rotate_left(1);
    }
    if out == targets {
        return Err("random target control is identical to the point table".to_owned());
    }
    Ok(out)
}

fn assert_held_out_exact_keys_miss(
    memory_artifact: &[u8],
    held_out: &[MemoryWeightSample],
) -> Result<(), String> {
    let memory = read_memory_uorlut01(memory_artifact)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "attached memory disappeared".to_owned())?;
    let keys = memory
        .opening
        .keys
        .iter()
        .map(Vec::as_slice)
        .collect::<BTreeSet<_>>();
    for sample in held_out {
        let key = &sample.prompt[..memory.record.k.min(sample.prompt.len())];
        if keys.contains(key) {
            return Err(format!(
                "held-out opening key {:?} remains in the exact memory table",
                key
            ));
        }
    }
    Ok(())
}

fn evaluate_neighbor_vote(
    memory_artifact: &[u8],
    point_artifact: &[u8],
    voted_artifact: Option<&[u8]>,
    point: &PointKeyImage,
    random_point: &PointKeyImage,
    samples: &[CpuConversationSample],
    neighbors: usize,
) -> Result<NeighborReading, String> {
    let readings = samples
        .par_iter()
        .map(|sample| {
            let (query, _) = opening_query_and_token(memory_artifact, &sample.prompt)?;
            let (_, served) = opening_query_and_token(point_artifact, &sample.prompt)?;
            let nearest = point_vote(point, &query, 1)?;
            let vote = point_vote(point, &query, neighbors)?;
            let random_vote = point_vote(random_point, &query, neighbors)?;
            let voted_served = voted_artifact
                .map(|bytes| opening_query_and_token(bytes, &sample.prompt).map(|(_, token)| token))
                .transpose()?
                .unwrap_or(nearest);
            if nearest != served {
                return Err(format!(
                    "nearest extraction named {nearest}, while the served point key named {served}"
                ));
            }
            if vote != voted_served {
                return Err(format!(
                    "eight-row extraction named {vote}, while the served vote_k={neighbors} artifact named {voted_served}"
                ));
            }
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "evaluation reply is empty".to_owned())?;
            Ok((
                u64::from(nearest == target),
                u64::from(vote == target),
                u64::from(random_vote == target),
                u64::from(nearest == served),
                u64::from(vote == voted_served),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut reading = NeighborReading {
        decisions: readings.len() as u64,
        ..NeighborReading::default()
    };
    for (nearest, vote, random_vote, nearest_matches_served, voted_matches_served) in readings {
        reading.nearest_top1 += nearest;
        reading.vote_top1 += vote;
        reading.random_vote_top1 += random_vote;
        reading.nearest_matches_served += nearest_matches_served;
        reading.voted_matches_served += voted_matches_served;
    }
    Ok(reading)
}

fn opening_query_and_token(artifact: &[u8], prompt: &[u32]) -> Result<(Vec<i32>, u32), String> {
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
    let mut result = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| error.to_string())?;
        if index + 1 == prompt.len() {
            result = Some((step.query_q12, step.token));
        }
    }
    result.ok_or_else(|| "evaluation prompt is empty".to_owned())
}

fn point_vote(image: &PointKeyImage, query: &[i32], neighbors: usize) -> Result<u32, String> {
    if query.len() != image.width || image.rows() == 0 || neighbors == 0 || neighbors > image.rows()
    {
        return Err("invalid point-neighborhood query or size".to_owned());
    }
    let mut distances = image
        .points
        .chunks_exact(image.width)
        .enumerate()
        .map(|(row, point)| {
            let distance = query
                .iter()
                .zip(point)
                .try_fold(0u64, |sum, (&query, &point)| {
                    sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                        .ok_or_else(|| "point-neighborhood distance overflows".to_owned())
                })?;
            Ok((distance, row))
        })
        .collect::<Result<Vec<_>, String>>()?;
    distances.sort_unstable();
    let mut counts = BTreeMap::<u32, usize>::new();
    for &(_, row) in distances.iter().take(neighbors) {
        *counts.entry(image.targets[row]).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(target, _)| target)
        .ok_or_else(|| "point neighborhood names no target".to_owned())
}

fn evaluate_openings(
    artifact: &[u8],
    samples: &[CpuConversationSample],
    width: usize,
) -> Result<OpeningReading, String> {
    let model = IntegerStackModel::parse(artifact).map_err(|error| error.to_string())?;
    let table = model
        .fbs_writer_table()
        .ok_or_else(|| "the artifact has no FBS writer table".to_owned())?
        .to_vec();
    if table.len() != model.shape().vocab * width {
        return Err("writer table shape differs".to_owned());
    }
    let readings = samples
        .par_iter()
        .map(|sample| {
            let mut forward =
                CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
            let mut step = None;
            for (index, &token) in sample.prompt.iter().enumerate() {
                if index + 1 == sample.prompt.len() {
                    forward.mark_reply_start();
                }
                step = Some(
                    forward
                        .step(token, CpuFbsHead::Writer)
                        .map_err(|error| error.to_string())?,
                );
            }
            let step = step.ok_or_else(|| "evaluation prompt is empty".to_owned())?;
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "evaluation reply is empty".to_owned())?;
            let row = &table[target as usize * width..][..width];
            let distance =
                step.query_q12
                    .iter()
                    .zip(row)
                    .try_fold(0u64, |sum, (&query, &point)| {
                        sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                            .ok_or_else(|| "opening distance overflows".to_owned())
                    })?;
            Ok((u64::from(step.token == target), distance))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut reading = OpeningReading {
        decisions: readings.len() as u64,
        ..OpeningReading::default()
    };
    for (top1, distance) in readings {
        reading.top1_correct += top1;
        reading.target_distance_sum = reading
            .target_distance_sum
            .checked_add(distance)
            .ok_or_else(|| "opening total overflows".to_owned())?;
    }
    Ok(reading)
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
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot};

    fn sample(document: usize, key: u32, reply: u32) -> CpuConversationSample {
        CpuConversationSample {
            document_ordinal: document,
            prompt: vec![
                key,
                key + 1,
                key + 2,
                key + 3,
                key + 4,
                key + 5,
                key + 6,
                key + 7,
            ],
            reply: vec![reply, 1],
        }
    }

    fn config() -> Config {
        Config {
            artifact: PathBuf::new(),
            tokens: PathBuf::new(),
            mask: PathBuf::new(),
            manifest: PathBuf::new(),
            out: PathBuf::new(),
            train_keys: 2,
            evaluation_keys: 2,
            per_key: 4,
            key_tokens: 8,
            reply_turn: 8,
            random_seed: 1,
            neighbor_vote: 0,
            select_only: true,
            expected_artifact_sha256: String::new(),
            expected_tokens_sha256: String::new(),
            expected_mask_sha256: String::new(),
            expected_manifest_sha256: String::new(),
        }
    }

    #[test]
    fn transfer_selection_is_deterministic_disjoint_and_bounded() {
        let mut examples = Vec::new();
        for key in [10, 20] {
            for document in [0, 2, 4, 6] {
                examples.push(sample(document, key, 100 + key));
            }
        }
        for key in [30, 40] {
            for document in [1, 3, 5, 7] {
                examples.push(sample(document, key, 200 + key));
            }
        }
        let first = select_transfer_groups(&examples, &config()).unwrap();
        let second = select_transfer_groups(&examples, &config()).unwrap();
        assert_eq!(first, second);
        let keys = first
            .0
            .iter()
            .map(|group| group.key.clone())
            .collect::<BTreeSet<_>>();
        assert!(first.1.iter().all(|group| !keys.contains(&group.key)));
        assert_eq!(first.0.len(), 2);
        assert_eq!(first.1.len(), 2);
    }

    #[test]
    fn random_control_permutes_targets_without_changing_points() {
        let points = PointKeyImage {
            width: 2,
            points: vec![1, 2, 3, 4, 5, 6],
            targets: vec![7, 8, 9],
            vote_k: 1,
        };
        let targets = shuffled_targets(&points.targets, 17).unwrap();
        let mut sorted = targets.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, points.targets);
        assert_ne!(targets, points.targets);
        assert_eq!(points.points.len(), points.rows() * points.width);
        assert!(shuffled_targets(&[7], 17).is_err());
    }

    #[test]
    fn eight_row_vote_can_override_a_wrong_nearest_row() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            targets: vec![9, 7, 7, 7, 7, 7, 7, 7, 8],
            vote_k: 1,
        };
        assert_eq!(point_vote(&points, &[0], 1).unwrap(), 9);
        assert_eq!(point_vote(&points, &[0], 8).unwrap(), 7);
    }

    #[test]
    fn point_vote_breaks_equal_counts_to_the_smaller_target() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 1, 2, 2],
            targets: vec![9, 5, 7, 8],
            vote_k: 1,
        };
        assert_eq!(point_vote(&points, &[0], 2).unwrap(), 5);
    }

    #[test]
    fn point_vote_refuses_an_empty_or_oversized_neighborhood() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 2],
            targets: vec![3, 4],
            vote_k: 1,
        };
        assert!(point_vote(&points, &[0], 0).is_err());
        assert!(point_vote(&points, &[0], 3).is_err());
    }

    fn opening_token(artifact: &[u8], prompt: &[u32]) -> u32 {
        let mut forward = CpuFbsForward::from_artifact(artifact).unwrap();
        let mut token = None;
        for (index, &value) in prompt.iter().enumerate() {
            if index + 1 == prompt.len() {
                forward.mark_reply_start();
            }
            token = Some(forward.step(value, CpuFbsHead::Writer).unwrap().token);
        }
        token.unwrap()
    }

    #[test]
    fn unseen_exact_key_uses_the_training_derived_point_target() {
        let mut snapshot = synthetic_fbs_trunk_with(
            104,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = false;
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 8,
            key_window: 24,
            opening: vec![FbsMemoryEntry {
                key: vec![1, 2, 3],
                value: 7,
            }],
            opening_weight: vec![0.0; snapshot.width],
            reply: None,
        });
        let memory_artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let prompt = vec![30, 31, 32, 33];
        let base = opening_token(&memory_artifact, &prompt);
        let target = (base + 1) % 512;
        let training = CpuConversationSample {
            document_ordinal: 0,
            prompt: vec![10, 11, 12, 13],
            reply: vec![target, 1],
        };
        let second_target = (target + 1) % 512;
        let second_training = CpuConversationSample {
            document_ordinal: 2,
            prompt: vec![20, 21, 22, 23],
            reply: vec![second_target, 1],
        };
        let held_out = CpuConversationSample {
            document_ordinal: 1,
            prompt,
            reply: vec![target, 1],
        };
        let point = derive_point_table(
            &memory_artifact,
            &[training, second_training],
            snapshot.width,
            snapshot.vocab,
        )
        .unwrap();
        let point_artifact = add_point_key_uorlut01(&memory_artifact, &point).unwrap();
        assert_ne!(opening_token(&memory_artifact, &held_out.prompt), target);
        let point_token = opening_token(&point_artifact, &held_out.prompt);
        assert!([target, second_target].contains(&point_token));
        let random_point = PointKeyImage {
            width: point.width,
            points: point.points.clone(),
            targets: shuffled_targets(&point.targets, 17).unwrap(),
            vote_k: 1,
        };
        let random_artifact = add_point_key_uorlut01(&memory_artifact, &random_point).unwrap();
        assert_ne!(
            opening_token(&random_artifact, &held_out.prompt),
            point_token
        );
        let neighbors = evaluate_neighbor_vote(
            &memory_artifact,
            &point_artifact,
            None,
            &point,
            &random_point,
            std::slice::from_ref(&held_out),
            1,
        )
        .unwrap();
        assert_eq!(neighbors.decisions, 1);
        assert_eq!(neighbors.nearest_matches_served, 1);
        assert_eq!(neighbors.voted_matches_served, 1);
        assert_held_out_exact_keys_miss(&memory_artifact, &[memory_sample(&held_out)]).unwrap();
        let hit = CpuConversationSample {
            document_ordinal: 1,
            prompt: vec![1, 2, 3],
            reply: vec![target, 1],
        };
        assert!(
            assert_held_out_exact_keys_miss(&memory_artifact, &[memory_sample(&hit)],).is_err()
        );
    }

    #[test]
    fn overlapping_transfer_keys_are_refused() {
        let mut examples = Vec::new();
        for document in [0, 2, 4, 6, 1, 3, 5, 7] {
            examples.push(sample(document, 10, 100));
        }
        assert!(select_transfer_groups(&examples, &config()).is_err());
    }

    #[test]
    fn malformed_point_rows_are_refused() {
        let malformed = PointKeyImage {
            width: 2,
            points: vec![1, 2, 3],
            targets: vec![4, 5],
            vote_k: 1,
        };
        assert!(malformed.check(512).is_err());
        let empty = PointKeyImage {
            width: 2,
            points: Vec::new(),
            targets: Vec::new(),
            vote_k: 1,
        };
        assert!(empty.check(512).is_err());
    }
}
