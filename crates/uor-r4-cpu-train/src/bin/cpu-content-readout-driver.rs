use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, content_mapping_sha256, content_target_mapping,
    derive_content_readout, derive_turn_memory, evaluate_content_readout, shuffle_content_targets,
    validate_corpus_manifest, ContentPromptReading, ContentReadout, ContentSignatureMode,
    CpuConversationSample, CpuMaskedCorpus, MemoryWeightSample, CONTENT_MIN_TARGET_VALUES,
    CONTENT_MIN_TRAINING_ROWS,
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
    let sign_shuffled = shuffle_content_targets(&sign_training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let code_shuffled = shuffle_content_targets(&code_training.readout, config.seed)
        .map_err(|error| error.to_string())?;
    let sign_changes = mapping_changes(&sign_training.readout, &sign_shuffled)?;
    let code_changes = mapping_changes(&code_training.readout, &code_shuffled)?;
    let sign_mapping = mapping_json(&sign_training.readout, &sign_shuffled, sign_changes)?;
    let code_mapping = mapping_json(&code_training.readout, &code_shuffled, code_changes)?;
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
            },
        }));
    }
    create_fresh_output(&config.out)?;
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
    let code_non_inferior = code_vs_sign_top1["candidate_sum"].as_u64()
        >= code_vs_sign_top1["control_sum"].as_u64()
        && code_vs_sign_depth["candidate_sum"].as_u64()
            >= code_vs_sign_depth["control_sum"].as_u64()
        && (code_vs_sign_top1["candidate_sum"] != code_vs_sign_top1["control_sum"]
            || code_vs_sign_depth["candidate_sum"] != code_vs_sign_depth["control_sum"]);
    let sign_structural = mode_structural_pass(&sign_training, &sign_mapping);
    let code_structural = mode_structural_pass(&code_training, &code_mapping);
    let code_exposure = code_arm.readout_decisions > 0 && code_arm.writer_fallbacks > 0;
    let bar = serde_json::json!({
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
        "passes": sign_structural
            && code_structural
            && code_exposure
            && code_non_inferior
            && code_training.leave_one_out_top1 > sign_training.leave_one_out_top1
            && code_vs_writer_top1["passes"].as_bool() == Some(true)
            && code_vs_shuffled_top1["passes"].as_bool() == Some(true)
            && code_vs_writer_depth["passes"].as_bool() == Some(true)
            && code_vs_shuffled_depth["passes"].as_bool() == Some(true),
    });
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
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-content-readout-run/2",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "CPU sign and modulo-256 nearest-example content readout; no serving or language claim",
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
        },
        "mapping": {
            "sign": sign_mapping_sha256,
            "code": code_mapping_sha256,
        },
        "arms": {
            "writer": { "reading": writer, "prompts": writer_prompts },
            "sign": { "reading": sign_arm, "prompts": sign_prompts },
            "sign_shuffled": { "reading": sign_shuffled_arm, "prompts": sign_shuffled_prompts },
            "code": { "reading": code_arm, "prompts": code_prompts },
            "code_shuffled": { "reading": code_shuffled_arm, "prompts": code_shuffled_prompts },
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
