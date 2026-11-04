use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, content_mapping_sha256, content_target_mapping,
    derive_content_readout, derive_turn_memory, evaluate_content_readout, shuffle_content_targets,
    validate_corpus_manifest, ContentPromptReading, ContentReadout, CpuConversationSample,
    CpuMaskedCorpus, MemoryWeightSample, CONTENT_MIN_TARGET_VALUES, CONTENT_MIN_TRAINING_ROWS,
};
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

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
     [--reply-cap N] [--seed N] [--select-only]"
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
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--reply-cap" => config.reply_cap = number(&value)?,
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
    {
        return Err("group or reply dimensions are out of bounds".to_owned());
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
    let training =
        derive_content_readout(&memory_artifact, &training_samples, config.reply_cap, true)
            .map_err(|error| error.to_string())?;
    let shuffled = shuffle_content_targets(&training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let true_mapping =
        content_target_mapping(&training.readout).map_err(|error| error.to_string())?;
    let shuffled_mapping = content_target_mapping(&shuffled).map_err(|error| error.to_string())?;
    let changed_mapping_assignments = true_mapping
        .iter()
        .filter(|(signature, target)| shuffled_mapping.get(*signature) != Some(*target))
        .count();
    let true_mapping_sha256 =
        content_mapping_sha256(&training.readout).map_err(|error| error.to_string())?;
    let shuffled_mapping_sha256 =
        content_mapping_sha256(&shuffled).map_err(|error| error.to_string())?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-content-readout-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "training": training,
            "shuffled_readout_sha256": readout_sha(&shuffled)?,
            "mapping": {
                "signature_entries": true_mapping.len(),
                "changed_assignments": changed_mapping_assignments,
                "differs": changed_mapping_assignments > 0
                    && true_mapping_sha256 != shuffled_mapping_sha256,
                "true_sha256": true_mapping_sha256,
                "shuffled_sha256": shuffled_mapping_sha256,
            },
        }));
    }
    create_fresh_output(&config.out)?;
    let (writer, writer_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        None,
    )
    .map_err(|error| error.to_string())?;
    let (true_arm, true_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        Some(&training.readout),
    )
    .map_err(|error| error.to_string())?;
    let (shuffled_arm, shuffled_prompts) = evaluate_content_readout(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        Some(&shuffled),
    )
    .map_err(|error| error.to_string())?;
    let true_readout_sha256 = readout_sha(&training.readout)?;
    let shuffled_readout_sha256 = readout_sha(&shuffled)?;
    let vs_writer_top1 = paired(&true_prompts, &writer_prompts, |row| row.top1)?;
    let vs_shuffled_top1 = paired(&true_prompts, &shuffled_prompts, |row| row.top1)?;
    let vs_writer_depth = paired(&true_prompts, &writer_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let vs_shuffled_depth = paired(&true_prompts, &shuffled_prompts, |row| {
        row.matched_prefix_depth as u64
    })?;
    let bar = serde_json::json!({
        "rows": training.rows,
        "rows_pass": training.rows >= CONTENT_MIN_TRAINING_ROWS as u64,
        "target_values": training.target_values,
        "target_diversity_pass": training.target_values >= CONTENT_MIN_TARGET_VALUES as u64,
        "max_distance": training.readout.max_distance,
        "covered_rows": training.covered_rows,
        "coverage_pass": training.covered_rows * 10 >= training.rows * 9,
        "leave_one_out_top1": training.leave_one_out_top1,
        "leave_one_out_ambiguous": training.leave_one_out_ambiguous,
        "readout_decisions": true_arm.readout_decisions,
        "writer_fallbacks": true_arm.writer_fallbacks,
        "ambiguous_readout_decisions": true_arm.ambiguous_readout_decisions,
        "exposure_pass": true_arm.readout_decisions > 0 && true_arm.writer_fallbacks > 0,
        "signature_entries": true_mapping.len(),
        "changed_mapping_assignments": changed_mapping_assignments,
        "shuffled_mapping_differs": changed_mapping_assignments > 0
            && true_mapping_sha256 != shuffled_mapping_sha256,
        "true_readout_sha256": true_readout_sha256,
        "shuffled_readout_sha256": shuffled_readout_sha256,
        "true_mapping_sha256": true_mapping_sha256,
        "shuffled_mapping_sha256": shuffled_mapping_sha256,
        "vs_writer_top1": vs_writer_top1,
        "vs_shuffled_top1": vs_shuffled_top1,
        "vs_writer_depth": vs_writer_depth,
        "vs_shuffled_depth": vs_shuffled_depth,
        "passes": training.rows >= CONTENT_MIN_TRAINING_ROWS as u64
            && training.target_values >= CONTENT_MIN_TARGET_VALUES as u64
            && training.covered_rows * 10 >= training.rows * 9
            && true_arm.readout_decisions > 0
            && true_arm.writer_fallbacks > 0
            && changed_mapping_assignments > 0
            && true_mapping_sha256 != shuffled_mapping_sha256
            && vs_writer_top1["passes"].as_bool() == Some(true)
            && vs_shuffled_top1["passes"].as_bool() == Some(true)
            && vs_writer_depth["passes"].as_bool() == Some(true)
            && vs_shuffled_depth["passes"].as_bool() == Some(true),
    });
    std::fs::write(
        config.out.join("true-readout.json"),
        serde_json::to_vec_pretty(&training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("shuffled-readout.json"),
        serde_json::to_vec_pretty(&shuffled).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-content-readout-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "CPU Hamming nearest-example content readout; no serving or language claim",
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
        "training": training,
        "shuffled_readout_sha256": readout_sha(&shuffled)?,
        "true_mapping_sha256": true_mapping_sha256,
        "shuffled_mapping_sha256": shuffled_mapping_sha256,
        "arms": {
            "writer": { "reading": writer, "prompts": writer_prompts },
            "true": { "reading": true_arm, "prompts": true_prompts },
            "shuffled": { "reading": shuffled_arm, "prompts": shuffled_prompts },
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

fn paired(
    candidate: &[ContentPromptReading],
    control: &[ContentPromptReading],
    value: impl Fn(&ContentPromptReading) -> u64,
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
    let threshold = ceil_three_sqrt(only.saturating_add(control_only));
    Ok(serde_json::json!({
        "candidate_only": only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && only >= threshold && threshold > 0,
    }))
}

fn ceil_three_sqrt(value: u64) -> u64 {
    let mut root = 0u64;
    while root.saturating_mul(root) < value.saturating_mul(9) {
        root += 1;
    }
    root
}

fn readout_sha(readout: &ContentReadout) -> Result<String, String> {
    Ok(uor_r4_discrete::corpus::sha256(
        &serde_json::to_vec(readout).map_err(|error| error.to_string())?,
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
