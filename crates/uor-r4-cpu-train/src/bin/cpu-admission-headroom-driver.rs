use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_self_prefix_generated, derive_turn_memory,
    evaluate_memory_weight_samples, evaluate_self_prefix, evaluate_self_prefix_admission,
    evaluate_self_prefix_prompts, patch_kept_cells, validate_corpus_manifest,
    CpuConversationSample, CpuMaskedCorpus, MemoryWeightSample, RankMarginConfig,
    SelfPrefixAdmissionCensus, SelfPrefixConfig, SelfPrefixMode, SelfPrefixPromptReading,
    SelfPrefixTargets,
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
            eprintln!("cpu-admission-headroom-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-admission-headroom-driver ARTIFACT TOKENS MASK MANIFEST OUT \
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
        return Err(format!(
            "selected split differs: {} training, {} evaluation",
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
    let k8 = add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    if IntegerStackModel::parse(&k8)
        .map_err(|error| error.to_string())?
        .shape()
        .kept_cells
        != Some(8)
    {
        return Err("source artifact is not K=8".to_owned());
    }
    let k16 = patch_kept_cells(&k8, 16).map_err(|error| error.to_string())?;
    if artifact_payload(&k8) != artifact_payload(&k16) {
        return Err("K=16 patch changed table payload bytes".to_owned());
    }
    let base = SelfPrefixConfig {
        rank_margin: RankMarginConfig::default(),
        reply_cap: config.reply_cap,
        memory_enabled: true,
        mode: SelfPrefixMode::DynamicGenerated,
        targets: SelfPrefixTargets::TrueLabels,
        shuffle_seed: config.seed,
    };
    let reference = SelfPrefixConfig {
        mode: SelfPrefixMode::Reference,
        ..base
    };
    let training_generated = evaluate_self_prefix_admission(&k8, &training_samples, base)
        .map_err(|error| error.to_string())?;
    let training_generated_k16 = evaluate_self_prefix_admission(&k16, &training_samples, base)
        .map_err(|error| error.to_string())?;
    let training_reference = evaluate_self_prefix_admission(&k8, &training_samples, reference)
        .map_err(|error| error.to_string())?;
    let evaluation_generated = evaluate_self_prefix_admission(&k8, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let evaluation_generated_k16 = evaluate_self_prefix_admission(&k16, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let evaluation_reference = evaluate_self_prefix_admission(&k8, &evaluation_samples, reference)
        .map_err(|error| error.to_string())?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-admission-headroom-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "census": {
                "training_generated": training_generated,
                "training_generated_k16": training_generated_k16,
                "training_reference": training_reference,
                "evaluation_generated": evaluation_generated,
                "evaluation_generated_k16": evaluation_generated_k16,
                "evaluation_reference": evaluation_reference,
            },
            "artifacts": {
                "k8_sha256": uor_r4_discrete::corpus::sha256(&k8),
                "k16_sha256": uor_r4_discrete::corpus::sha256(&k16),
                "payload_identical": true,
            },
        }));
    }
    create_fresh_output(&config.out)?;
    std::fs::write(config.out.join("k8.uorlut"), &k8).map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("k16.uorlut"), &k16).map_err(|error| error.to_string())?;
    let k8_eval =
        evaluate_self_prefix(&k8, &evaluation_samples, base).map_err(|error| error.to_string())?;
    let k16_eval =
        evaluate_self_prefix(&k16, &evaluation_samples, base).map_err(|error| error.to_string())?;
    let k8_prompts = evaluate_self_prefix_prompts(&k8, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let k16_prompts = evaluate_self_prefix_prompts(&k16, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let k8_teacher = evaluate_memory_weight_samples(&k8, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let k16_teacher = evaluate_memory_weight_samples(&k16, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let k8_parity = audit_self_prefix_generated(&k8, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let k16_parity = audit_self_prefix_generated(&k16, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let paired = paired_depth(&k16_prompts, &k8_prompts)?;
    let k8_reading = k_reading(&training_generated, 8)?;
    let k16_reading = k_reading(&training_generated_k16, 16)?;
    let k8_heldout = k_reading(&evaluation_generated, 8)?;
    let k16_heldout = k_reading(&evaluation_generated_k16, 16)?;
    let threshold_pass = k16_reading.positive_margin >= 640
        && k16_reading.positive_margin >= k8_reading.positive_margin + 610
        && k16_reading.positive_margin_pairs >= 64;
    let heldout_added = k16_heldout
        .target_admitted
        .saturating_sub(k8_heldout.target_admitted);
    let bar = serde_json::json!({
        "training_positive_margin_k8": k8_reading.positive_margin,
        "training_positive_margin_k16": k16_reading.positive_margin,
        "training_positive_margin_gain": k16_reading.positive_margin.saturating_sub(k8_reading.positive_margin),
        "training_positive_margin_cells": k16_reading.positive_margin_cells,
        "training_positive_margin_pairs": k16_reading.positive_margin_pairs,
        "training_positive_margin_pass": threshold_pass,
        "heldout_admitted_k8": k8_heldout.target_admitted,
        "heldout_admitted_k16": k16_heldout.target_admitted,
        "heldout_admitted_gain": heldout_added,
        "heldout_admitted_pass": heldout_added >= 128,
        "depth_nonloss": k16_eval.matched_prefix_depth_sum >= k8_eval.matched_prefix_depth_sum,
        "teacher_top1_nonloss": k16_teacher.top1_correct >= k8_teacher.top1_correct,
        "random_extra_target_admitted": evaluation_generated_k16.random_extra.target_admitted,
        "random_extra_positive_margin": evaluation_generated_k16.random_extra.positive_margin,
        "random_extra_positive_margin_cells": evaluation_generated_k16.random_extra.positive_margin_cells,
        "k16_above_random_extra": k16_heldout.target_admitted > evaluation_generated_k16.random_extra.target_admitted,
        "k16_above_random_extra_cells": k16_heldout.positive_margin_cells > evaluation_generated_k16.random_extra.positive_margin_cells,
        "payload_identical": artifact_payload(&k8) == artifact_payload(&k16),
        "k8_parity": k8_parity,
        "k16_parity": k16_parity,
        "all_parity_exact": k8_parity.is_exact() && k16_parity.is_exact(),
        "passes": threshold_pass
            && heldout_added >= 128
            && k16_eval.matched_prefix_depth_sum >= k8_eval.matched_prefix_depth_sum
            && k16_teacher.top1_correct >= k8_teacher.top1_correct
            && k16_heldout.target_admitted > evaluation_generated_k16.random_extra.target_admitted
            && k16_heldout.positive_margin_cells
                > evaluation_generated_k16.random_extra.positive_margin_cells
            && artifact_payload(&k8) == artifact_payload(&k16)
            && k8_parity.is_exact()
            && k16_parity.is_exact(),
    });
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-admission-headroom-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "K=8 versus K=16 writer admission headroom; no training update or language claim",
        "inputs": {
            "artifact": config.artifact,
            "artifact_sha256": config.expected_artifact_sha256,
            "tokens": config.tokens,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask": config.mask,
            "mask_sha256": config.expected_mask_sha256,
            "manifest": config.manifest,
            "manifest_sha256": config.expected_manifest_sha256,
            "k8_sha256": uor_r4_discrete::corpus::sha256(&k8),
            "k16_sha256": uor_r4_discrete::corpus::sha256(&k16),
        },
        "selection": selection_json(&groups, &training_samples, &evaluation_samples),
        "census": {
            "training_generated": training_generated,
            "training_generated_k16": training_generated_k16,
            "training_reference": training_reference,
            "evaluation_generated": evaluation_generated,
            "evaluation_generated_k16": evaluation_generated_k16,
            "evaluation_reference": evaluation_reference,
        },
        "evaluation": {
            "k8": k8_eval,
            "k16": k16_eval,
            "paired_depth": paired,
            "k8_teacher": k8_teacher,
            "k16_teacher": k16_teacher,
            "k8_prompts": k8_prompts,
            "k16_prompts": k16_prompts,
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

fn k_reading(
    census: &SelfPrefixAdmissionCensus,
    kept_cells: usize,
) -> Result<uor_r4_cpu_train::CellKReading, String> {
    census
        .readings
        .iter()
        .find(|reading| reading.kept_cells == kept_cells)
        .cloned()
        .ok_or_else(|| format!("K={kept_cells} reading is missing"))
}

fn paired_depth(
    candidate: &[SelfPrefixPromptReading],
    control: &[SelfPrefixPromptReading],
) -> Result<serde_json::Value, String> {
    if candidate.len() != control.len() {
        return Err("paired prompt counts differ".to_owned());
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    let mut ties = 0u64;
    let mut candidate_depth = 0u64;
    let mut control_depth = 0u64;
    for (candidate, control) in candidate.iter().zip(control) {
        if candidate.sample != control.sample {
            return Err("paired prompt identity differs".to_owned());
        }
        candidate_depth += candidate.matched_prefix_depth as u64;
        control_depth += control.matched_prefix_depth as u64;
        match candidate
            .matched_prefix_depth
            .cmp(&control.matched_prefix_depth)
        {
            std::cmp::Ordering::Greater => candidate_only += 1,
            std::cmp::Ordering::Less => control_only += 1,
            std::cmp::Ordering::Equal => ties += 1,
        }
    }
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "ties": ties,
        "candidate_depth": candidate_depth,
        "control_depth": control_depth,
    }))
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
        "keys": groups.iter().map(|group| serde_json::json!({
            "key": group.key,
            "training": group.training.len(),
            "evaluation": group.evaluation.len(),
        })).collect::<Vec<_>>(),
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

fn artifact_payload(artifact: &[u8]) -> &[u8] {
    let header_len = u64::from_le_bytes(artifact[8..16].try_into().unwrap()) as usize;
    let data_start = (16 + header_len).div_ceil(64) * 64;
    &artifact[data_start..]
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
    use uor_r4_cpu_train::{CellKReading, RandomCellReading, SelfPrefixAdmissionCensus};

    fn prompt(sample: usize, depth: usize) -> SelfPrefixPromptReading {
        SelfPrefixPromptReading {
            sample,
            matched_prefix_depth: depth,
            generated_top1: 0,
            observed_decisions: 0,
            opening_correct: false,
            exact_reference: false,
            stop: "cap".to_owned(),
            generated_tokens: 0,
        }
    }

    #[test]
    fn k_reading_selects_the_registered_budget() {
        let row = |kept_cells| CellKReading {
            kept_cells,
            target_admitted: 0,
            positive_margin: 0,
            writer_top1: 0,
            positive_margin_cells: 0,
            positive_margin_pairs: 0,
        };
        let census = SelfPrefixAdmissionCensus {
            fixed_horizon_decisions: 0,
            observed_decisions: 0,
            missing_decisions: 0,
            rank_histogram: BTreeMap::new(),
            minimum_k_histogram: BTreeMap::new(),
            readings: vec![row(8), row(16)],
            random_extra: RandomCellReading {
                kept_cells: 16,
                target_admitted: 0,
                positive_margin: 0,
                writer_top1: 0,
                positive_margin_cells: 0,
                positive_margin_pairs: 0,
            },
        };
        assert_eq!(k_reading(&census, 8).unwrap().kept_cells, 8);
        assert_eq!(k_reading(&census, 16).unwrap().kept_cells, 16);
        assert!(k_reading(&census, 12).is_err());
    }

    #[test]
    fn paired_depth_refuses_unequal_or_reordered_panels() {
        assert!(paired_depth(&[prompt(0, 1)], &[]).is_err());
        assert!(paired_depth(&[prompt(0, 1)], &[prompt(1, 1)]).is_err());
        assert!(paired_depth(&[prompt(0, 2)], &[prompt(0, 1)]).is_ok());
    }
}
