use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory,
    evaluate_memory_weight_samples, evaluate_self_prefix, evaluate_self_prefix_prompts,
    validate_corpus_manifest, CpuConversationSample, CpuMaskedCorpus, CpuSelfPrefixTrainer,
    MemoryWeightSample, RankMarginConfig, SelfPrefixConfig, SelfPrefixMode, SelfPrefixTargets,
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
    attempts: usize,
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

#[derive(Clone, Copy)]
struct Arm {
    name: &'static str,
    mode: SelfPrefixMode,
    targets: SelfPrefixTargets,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-generated-prefix-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-generated-prefix-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--keys N] [--per-key N] [--key-tokens N] [--reply-turn N] \
     [--reply-cap N] [--attempts N] [--seed N] [--select-only]"
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
        attempts: 80,
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
            "--attempts" => config.attempts = number(&value)?,
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
        || !(1..=80).contains(&config.attempts)
    {
        return Err("group, reply or attempt dimensions are out of bounds".to_owned());
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
    let training_corpus = groups
        .iter()
        .flat_map(|group| group.training.iter())
        .cloned()
        .collect::<Vec<_>>();
    let evaluation_corpus = groups
        .iter()
        .flat_map(|group| group.evaluation.iter())
        .cloned()
        .collect::<Vec<_>>();
    let training_samples = training_corpus
        .iter()
        .map(memory_sample)
        .collect::<Vec<_>>();
    let evaluation_samples = evaluation_corpus
        .iter()
        .map(memory_sample)
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
    let memory_artifact =
        add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    let base = SelfPrefixConfig {
        rank_margin: RankMarginConfig::default(),
        reply_cap: config.reply_cap,
        memory_enabled: true,
        mode: SelfPrefixMode::DynamicGenerated,
        targets: SelfPrefixTargets::TrueLabels,
        shuffle_seed: config.seed,
    };
    let preflight_trainer =
        CpuSelfPrefixTrainer::from_artifact(&memory_artifact, training_samples.clone(), base)
            .map_err(|error| error.to_string())?;
    let preflight = preflight_trainer
        .eligibility()
        .map_err(|error| error.to_string())?;
    let shuffled_config = SelfPrefixConfig {
        targets: SelfPrefixTargets::SameCellShuffled,
        ..base
    };
    let shuffled_preflight = CpuSelfPrefixTrainer::from_artifact(
        &memory_artifact,
        training_samples.clone(),
        shuffled_config,
    )
    .map_err(|error| error.to_string())?
    .eligibility()
    .map_err(|error| error.to_string())?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-generated-prefix-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "preflight": preflight,
            "shuffled_preflight": shuffled_preflight,
            "frozen_training": preflight_trainer.reading().map_err(|error| error.to_string())?,
        }));
    }
    let preflight_pass = preflight.eligible_decisions >= 640
        && preflight.prompts >= 16
        && preflight.target_cells >= 8
        && preflight.same_cell_shuffle_pairs >= 64
        && shuffled_preflight.eligible_decisions >= 640
        && shuffled_preflight.prompts >= 16
        && shuffled_preflight.target_cells >= 8
        && shuffled_preflight.same_cell_shuffle_pairs >= 64;
    if !preflight_pass {
        return Err(format!(
            "self-prefix eligibility gate failed: {}",
            serde_json::to_string(&preflight).expect("preflight serializes")
        ));
    }
    create_fresh_output(&config.out)?;
    std::fs::write(config.out.join("memory-baseline.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;
    let arms = [
        Arm {
            name: "reference-true",
            mode: SelfPrefixMode::Reference,
            targets: SelfPrefixTargets::TrueLabels,
        },
        Arm {
            name: "static-true",
            mode: SelfPrefixMode::StaticGenerated,
            targets: SelfPrefixTargets::TrueLabels,
        },
        Arm {
            name: "static-shuffled",
            mode: SelfPrefixMode::StaticGenerated,
            targets: SelfPrefixTargets::SameCellShuffled,
        },
        Arm {
            name: "dynamic-true",
            mode: SelfPrefixMode::DynamicGenerated,
            targets: SelfPrefixTargets::TrueLabels,
        },
        Arm {
            name: "dynamic-shuffled",
            mode: SelfPrefixMode::DynamicGenerated,
            targets: SelfPrefixTargets::SameCellShuffled,
        },
    ];
    let frozen_eval = evaluate_self_prefix(&memory_artifact, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let frozen_prompts = evaluate_self_prefix_prompts(&memory_artifact, &evaluation_samples, base)
        .map_err(|error| error.to_string())?;
    let frozen_teacher =
        evaluate_memory_weight_samples(&memory_artifact, &evaluation_samples, true)
            .map_err(|error| error.to_string())?;
    let frozen_parity = audit_memory_weight_samples(&memory_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    let mut all_parity_exact = frozen_parity.is_exact();
    let mut arm_reports = serde_json::Map::new();
    arm_reports.insert(
        "frozen".to_owned(),
        arm_json(
            &memory_artifact,
            frozen_eval.clone(),
            frozen_prompts.clone(),
            frozen_teacher.clone(),
            frozen_parity,
            0,
            0,
        ),
    );
    let mut dynamic_prompts = None;
    let mut dynamic_reading = None;
    let mut dynamic_teacher = None;
    let mut static_shuffled_accepted = 0u64;
    let mut shuffled_prompts = None;
    let mut shuffled_reading = None;
    let mut dynamic_shuffled_accepted = 0u64;
    for arm in arms {
        let arm_config = SelfPrefixConfig {
            mode: arm.mode,
            targets: arm.targets,
            ..base
        };
        let mut trainer = CpuSelfPrefixTrainer::from_artifact(
            &memory_artifact,
            training_samples.clone(),
            arm_config,
        )
        .map_err(|error| error.to_string())?;
        let steps = trainer
            .train_steps(config.attempts)
            .map_err(|error| error.to_string())?;
        let trained = trainer.artifact().to_vec();
        let reading = evaluate_self_prefix(&trained, &evaluation_samples, base)
            .map_err(|error| error.to_string())?;
        let prompts = evaluate_self_prefix_prompts(&trained, &evaluation_samples, base)
            .map_err(|error| error.to_string())?;
        let teacher = evaluate_memory_weight_samples(&trained, &evaluation_samples, true)
            .map_err(|error| error.to_string())?;
        let parity = audit_memory_weight_samples(&trained, &evaluation_samples, true)
            .map_err(|error| error.to_string())?;
        all_parity_exact &= parity.is_exact();
        std::fs::write(config.out.join(format!("{}.uorlut", arm.name)), &trained)
            .map_err(|error| error.to_string())?;
        arm_reports.insert(
            arm.name.to_owned(),
            arm_json(
                &trained,
                reading.clone(),
                prompts.clone(),
                teacher.clone(),
                parity,
                trainer.attempts(),
                trainer.accepted(),
            ),
        );
        if arm.name == "dynamic-true" {
            dynamic_prompts = Some(prompts.clone());
            dynamic_reading = Some(reading.clone());
            dynamic_teacher = Some(teacher);
        }
        if arm.name == "dynamic-shuffled" {
            shuffled_prompts = Some(prompts.clone());
            shuffled_reading = Some(reading.clone());
            dynamic_shuffled_accepted = trainer.accepted();
        }
        if arm.name == "static-shuffled" {
            static_shuffled_accepted = trainer.accepted();
        }
        let _ = steps;
    }
    let dynamic_prompts = dynamic_prompts.ok_or("dynamic-true prompts missing")?;
    let dynamic_reading = dynamic_reading.ok_or("dynamic-true reading missing")?;
    let dynamic_teacher = dynamic_teacher.ok_or("dynamic-true teacher reading missing")?;
    let shuffled_prompts = shuffled_prompts.ok_or("dynamic-shuffled prompts missing")?;
    let shuffled_reading = shuffled_reading.ok_or("dynamic-shuffled reading missing")?;
    let vs_frozen = paired_depth(&dynamic_prompts, &frozen_prompts)?;
    let vs_shuffled = paired_depth(&dynamic_prompts, &shuffled_prompts)?;
    let teacher_top1 = frozen_teacher.top1_correct;
    let bar = serde_json::json!({
        "vs_frozen": vs_frozen,
        "vs_shuffled": vs_shuffled,
        "generated_top1_above_frozen": dynamic_reading.generated_top1 > frozen_eval.generated_top1,
        "generated_top1_above_shuffled": dynamic_reading.generated_top1 > shuffled_reading.generated_top1,
        "teacher_top1_frozen": teacher_top1,
        "teacher_top1_dynamic": dynamic_teacher.top1_correct,
        "frozen_admitted_only_writer_top1": admitted_only_top1(&frozen_eval),
        "dynamic_admitted_only_writer_top1": admitted_only_top1(&dynamic_reading),
        "all_parity_exact": all_parity_exact,
        "static_shuffled_accepted": static_shuffled_accepted,
        "dynamic_shuffled_accepted": dynamic_shuffled_accepted,
        "shuffle_controls_active": static_shuffled_accepted > 0 && dynamic_shuffled_accepted > 0,
        "passes": vs_frozen["passes"].as_bool() == Some(true)
            && vs_shuffled["passes"].as_bool() == Some(true)
            && dynamic_reading.generated_top1 > frozen_eval.generated_top1
            && dynamic_reading.generated_top1 > shuffled_reading.generated_top1
            && dynamic_teacher.top1_correct >= teacher_top1
            && all_parity_exact
            && static_shuffled_accepted > 0
            && dynamic_shuffled_accepted > 0,
    });
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-generated-prefix-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "self-generated-prefix writer ranking; no serving-contract or language claim",
        "inputs": {
            "artifact": config.artifact,
            "artifact_sha256": config.expected_artifact_sha256,
            "tokens": config.tokens,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask": config.mask,
            "mask_sha256": config.expected_mask_sha256,
            "manifest": config.manifest,
            "manifest_sha256": config.expected_manifest_sha256,
            "memory_artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
        },
        "selection": selection_json(&groups, &training_samples, &evaluation_samples),
        "preflight": preflight,
        "attempts": config.attempts,
        "reply_cap": config.reply_cap,
        "arms": arm_reports,
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

fn arm_json(
    artifact: &[u8],
    reading: uor_r4_cpu_train::SelfPrefixReading,
    prompts: Vec<uor_r4_cpu_train::SelfPrefixPromptReading>,
    teacher: uor_r4_cpu_train::MemoryWeightReading,
    parity: uor_r4_cpu_train::MemoryWeightParityReading,
    attempts: u64,
    accepted: u64,
) -> serde_json::Value {
    serde_json::json!({
        "artifact_sha256": uor_r4_discrete::corpus::sha256(artifact),
        "attempts": attempts,
        "accepted": accepted,
        "reading": reading,
        "prompts": prompts,
        "teacher_forced": teacher,
        "parity": parity,
    })
}

fn paired_depth(
    candidate: &[uor_r4_cpu_train::SelfPrefixPromptReading],
    control: &[uor_r4_cpu_train::SelfPrefixPromptReading],
) -> Result<serde_json::Value, String> {
    if candidate.len() != control.len() {
        return Err(format!(
            "paired prompt counts differ: {} and {}",
            candidate.len(),
            control.len()
        ));
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
    let threshold = ceil_three_sqrt(candidate_only.saturating_add(control_only));
    Ok(serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "ties": ties,
        "candidate_depth": candidate_depth,
        "control_depth": control_depth,
        "threshold": threshold,
        "passes": candidate_depth > control_depth && candidate_only >= threshold && threshold > 0,
    }))
}

fn ceil_three_sqrt(value: u64) -> u64 {
    let target = value.saturating_mul(9);
    let mut root = 0u64;
    while root.saturating_mul(root) < target {
        root += 1;
    }
    root
}

fn admitted_only_top1(reading: &uor_r4_cpu_train::SelfPrefixReading) -> f64 {
    if reading.target_admitted == 0 {
        0.0
    } else {
        reading.writer_top1 as f64 / reading.target_admitted as f64
    }
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

fn create_fresh_output(path: &Path) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(())
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

    #[test]
    fn paired_threshold_uses_three_sigma_integer_rule() {
        assert_eq!(ceil_three_sqrt(0), 0);
        assert_eq!(ceil_three_sqrt(1), 3);
        assert_eq!(ceil_three_sqrt(2), 5);
        assert_eq!(ceil_three_sqrt(4), 6);
    }

    #[test]
    fn paired_depth_refuses_unequal_or_reordered_panels() {
        let reading = |sample, depth| uor_r4_cpu_train::SelfPrefixPromptReading {
            sample,
            matched_prefix_depth: depth,
            generated_top1: 0,
            observed_decisions: 0,
            opening_correct: false,
            exact_reference: false,
            stop: "cap".to_owned(),
            generated_tokens: 0,
        };
        assert!(paired_depth(&[reading(0, 1)], &[]).is_err());
        assert!(paired_depth(&[reading(0, 1)], &[reading(1, 1)]).is_err());
    }
}
