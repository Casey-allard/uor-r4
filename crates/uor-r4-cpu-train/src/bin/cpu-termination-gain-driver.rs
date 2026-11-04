use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory, evaluate_replies,
    evaluate_termination_gain_samples, random_gain_control, shuffle_termination_gain_labels,
    termination_gain_labels, validate_corpus_manifest, CpuConversationSample, CpuMaskedCorpus,
    CpuTerminationGainTrainer, MemoryWeightSample, TerminationGainConfig,
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
    reply_cap: usize,
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
            eprintln!("cpu-termination-gain-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-termination-gain-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--steps N] [--keys N] [--per-key N] [--key-tokens N] \
     [--reply-turn N] [--reply-cap N] [--seed N] [--select-only]"
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
        key_count: 16,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        seed: 1,
        reply_cap: 16,
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
            "--steps" => config.steps = number(&value)?,
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
    if config.steps == 0
        || config.key_count == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
        || !(1..=64).contains(&config.reply_cap)
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
    let training_labels =
        termination_gain_labels(&training_samples).map_err(|error| error.to_string())?;
    let evaluation_labels =
        termination_gain_labels(&evaluation_samples).map_err(|error| error.to_string())?;
    let shuffled_labels = shuffle_termination_gain_labels(&training_labels, config.seed)
        .map_err(|error| error.to_string())?;
    let shuffled_changed_terminal_rows = training_labels
        .iter()
        .zip(&shuffled_labels)
        .filter(|(left, right)| left != right)
        .count();
    if shuffled_changed_terminal_rows == 0 {
        return Err("shuffled termination labels did not change the subset".to_owned());
    }
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-termination-gain-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "labels": {
                "training_terminal": training_labels.iter().filter(|terminal| **terminal).count(),
                "training_nonterminal": training_labels.iter().filter(|terminal| !**terminal).count(),
                "evaluation_terminal": evaluation_labels.iter().filter(|terminal| **terminal).count(),
                "evaluation_nonterminal": evaluation_labels.iter().filter(|terminal| !**terminal).count(),
                "shuffled_terminal": shuffled_labels.iter().filter(|terminal| **terminal).count(),
                "shuffled_changed_terminal_rows": shuffled_changed_terminal_rows,
            },
        }));
    }

    create_fresh_output(&config.out)?;
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
    std::fs::write(config.out.join("memory-baseline.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;

    let trainer_config = TerminationGainConfig::default();
    let mut true_trainer = CpuTerminationGainTrainer::from_artifact_cached(
        &memory_artifact,
        training_samples.clone(),
        training_labels.clone(),
        trainer_config,
    )
    .map_err(|error| error.to_string())?;
    let mut shuffled_trainer = CpuTerminationGainTrainer::from_artifact_cached(
        &memory_artifact,
        training_samples.clone(),
        shuffled_labels.clone(),
        trainer_config,
    )
    .map_err(|error| error.to_string())?;
    let true_before = true_trainer.reading().map_err(|error| error.to_string())?;
    let shuffled_before = shuffled_trainer
        .reading()
        .map_err(|error| error.to_string())?;
    let mut true_steps = Vec::new();
    let mut shuffled_steps = Vec::new();
    for _ in 0..config.steps {
        let true_step = true_trainer
            .train_step()
            .map_err(|error| error.to_string())?;
        let shuffled_step = shuffled_trainer
            .train_step()
            .map_err(|error| error.to_string())?;
        if true_step.updated {
            true_steps.push(true_step);
        }
        if shuffled_step.updated {
            shuffled_steps.push(shuffled_step);
        }
    }
    if true_steps.is_empty() {
        return Err("true-label termination training accepted no move".to_owned());
    }
    if shuffled_steps.is_empty() {
        return Err("shuffled-label termination training accepted no move".to_owned());
    }
    let true_after = true_trainer.reading().map_err(|error| error.to_string())?;
    let shuffled_after = shuffled_trainer
        .reading()
        .map_err(|error| error.to_string())?;
    let trained_artifact = true_trainer.artifact().to_vec();
    let shuffled_artifact = shuffled_trainer.artifact().to_vec();
    let (random_artifact, random_moves) =
        random_gain_control(&memory_artifact, true_steps.len(), config.seed)
            .map_err(|error| error.to_string())?;
    if random_moves.len() != true_steps.len() {
        return Err("random-gain move count differs from the true trainer".to_owned());
    }
    if trained_artifact == memory_artifact {
        return Err("true-label training returned the frozen artifact".to_owned());
    }
    if shuffled_artifact == memory_artifact {
        return Err("shuffled-label training returned the frozen artifact".to_owned());
    }
    if random_artifact == memory_artifact {
        return Err("random-gain control returned the frozen artifact".to_owned());
    }
    for (name, bytes) in [
        ("trained.uorlut", trained_artifact.as_slice()),
        ("label-shuffled.uorlut", shuffled_artifact.as_slice()),
        ("random.uorlut", random_artifact.as_slice()),
    ] {
        std::fs::write(config.out.join(name), bytes).map_err(|error| error.to_string())?;
    }

    let arms = [
        ("frozen", memory_artifact.as_slice()),
        ("trained", trained_artifact.as_slice()),
        ("label_shuffled", shuffled_artifact.as_slice()),
        ("random", random_artifact.as_slice()),
    ];
    let evaluations = arms
        .iter()
        .map(|(name, bytes)| {
            evaluate_termination_gain_samples(bytes, &evaluation_samples, &evaluation_labels, true)
                .map(|reading| (*name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let frozen = evaluations
        .get("frozen")
        .ok_or_else(|| "missing frozen arm".to_owned())?;
    let trained = evaluations
        .get("trained")
        .ok_or_else(|| "missing trained arm".to_owned())?;
    let shuffled = evaluations
        .get("label_shuffled")
        .ok_or_else(|| "missing shuffled arm".to_owned())?;
    let random = evaluations
        .get("random")
        .ok_or_else(|| "missing random arm".to_owned())?;
    let vs_shuffled = paired_terminal_comparison(trained, shuffled)?;
    let vs_random = paired_terminal_comparison(trained, random)?;
    let target_terminal = ceil_div(trained.reading.terminal_decisions, 4);
    let nonterminal_limit = trained.reading.nonterminal_decisions / 100;
    let bar = serde_json::json!({
        "terminal_bos_required": target_terminal,
        "terminal_bos_target": trained.reading.terminal_bos_top1 >= target_terminal,
        "vs_shuffled": vs_shuffled,
        "vs_random": vs_random,
        "nonterminal_bos_limit": nonterminal_limit,
        "nonterminal_bos": trained.reading.nonterminal_bos_top1,
        "nonterminal_bos_pass": trained.reading.nonterminal_bos_top1 <= nonterminal_limit
            && trained.reading.nonterminal_bos_top1 <= frozen.reading.nonterminal_bos_top1,
        "nonterminal_content_pass": trained.reading.nonterminal_content_top1
            >= frozen.reading.nonterminal_content_top1,
        "passes": trained.reading.terminal_bos_top1 >= target_terminal
            && vs_shuffled["passes"].as_bool() == Some(true)
            && vs_random["passes"].as_bool() == Some(true)
            && trained.reading.nonterminal_bos_top1 <= nonterminal_limit
            && trained.reading.nonterminal_bos_top1 <= frozen.reading.nonterminal_bos_top1
            && trained.reading.nonterminal_content_top1 >= frozen.reading.nonterminal_content_top1,
    });

    let parity = arms
        .iter()
        .map(|(name, bytes)| {
            audit_memory_weight_samples(bytes, &evaluation_samples, true)
                .map(|reading| (*name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    for (name, reading) in &parity {
        if !reading.is_exact() {
            return Err(format!("{name} lost complete CPU/serving parity"));
        }
    }
    let complete_replies = arms
        .iter()
        .map(|(name, bytes)| {
            evaluate_replies(bytes, &evaluation_samples, config.reply_cap)
                .map(|reading| (*name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-termination-gain-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "direct final-gain BOS training on a bounded CPU development panel; not GPU qualification or language capability",
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
        "labels": {
            "training_terminal": training_labels.iter().filter(|terminal| **terminal).count(),
            "training_nonterminal": training_labels.iter().filter(|terminal| !**terminal).count(),
            "evaluation_terminal": evaluation_labels.iter().filter(|terminal| **terminal).count(),
            "evaluation_nonterminal": evaluation_labels.iter().filter(|terminal| !**terminal).count(),
            "shuffled_terminal": shuffled_labels.iter().filter(|terminal| **terminal).count(),
            "shuffled_changed_terminal_rows": shuffled_changed_terminal_rows,
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
            "steps": config.steps,
            "true": {
                "accepted_moves": true_steps.len(),
                "before": true_before.reading,
                "after": true_after.reading,
                "steps": true_steps,
                "artifact_sha256": uor_r4_discrete::corpus::sha256(&trained_artifact),
            },
            "label_shuffled": {
                "accepted_moves": shuffled_steps.len(),
                "before": shuffled_before.reading,
                "after": shuffled_after.reading,
                "steps": shuffled_steps,
                "artifact_sha256": uor_r4_discrete::corpus::sha256(&shuffled_artifact),
            },
            "random": {
                "moves": random_moves,
                "artifact_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
            },
        },
        "evaluation": evaluations,
        "bar": bar,
        "parity": parity,
        "complete_replies": complete_replies,
        "reply_cap": config.reply_cap,
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

fn selection_json(
    groups: &[KeyGroup],
    training: &[MemoryWeightSample],
    evaluation: &[MemoryWeightSample],
) -> serde_json::Value {
    serde_json::json!({
        "groups": groups.len(),
        "training_samples": training.len(),
        "evaluation_samples": evaluation.len(),
        "training_decisions": training.iter().map(|sample| sample.prompt.len() + sample.reply.len() - 1).sum::<usize>(),
        "evaluation_decisions": evaluation.iter().map(|sample| sample.prompt.len() + sample.reply.len() - 1).sum::<usize>(),
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

fn paired_terminal_comparison(
    candidate: &uor_r4_cpu_train::TerminationGainEvaluation,
    control: &uor_r4_cpu_train::TerminationGainEvaluation,
) -> Result<serde_json::Value, String> {
    if candidate.terminal_rows.len() != control.terminal_rows.len() {
        return Err("terminal BOS arms report different row counts".to_owned());
    }
    let mut rows = 0u64;
    let mut candidate_top1 = 0u64;
    let mut control_top1 = 0u64;
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    for (&left, &right) in candidate.terminal_rows.iter().zip(&control.terminal_rows) {
        rows += 1;
        candidate_top1 += u64::from(left);
        control_top1 += u64::from(right);
        candidate_only += u64::from(left && !right);
        control_only += u64::from(!left && right);
    }
    let discordant = candidate_only + control_only;
    let threshold = 2u64.max(ceil_sqrt(discordant * 4));
    let gain = i128::from(candidate_top1) - i128::from(control_top1);
    Ok(serde_json::json!({
        "rows": rows,
        "candidate_top1": candidate_top1,
        "control_top1": control_top1,
        "candidate_only": candidate_only,
        "control_only": control_only,
        "discordant": discordant,
        "gain": gain,
        "threshold": threshold,
        "passes": gain >= i128::from(threshold),
    }))
}

fn ceil_div(value: u64, divisor: u64) -> u64 {
    value.div_ceil(divisor)
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

fn create_fresh_output(out: &Path) -> Result<(), String> {
    if out.exists() {
        return Err(format!("output root already exists: {}", out.display()));
    }
    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::create_dir(out).map_err(|error| format!("{}: {error}", out.display()))
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
    use uor_r4_cpu_train::{TerminationGainEvaluation, TerminationGainReading};

    fn evaluation(
        rows: Vec<bool>,
        nonterminal_bos: u64,
        content: u64,
    ) -> TerminationGainEvaluation {
        let terminal = rows.iter().filter(|row| **row).count() as u64;
        TerminationGainEvaluation {
            reading: TerminationGainReading {
                artifact_sha256: "00".repeat(32),
                samples: 1,
                decisions: 10,
                terminal_decisions: rows.len() as u64,
                nonterminal_decisions: 9,
                terminal_bos_top1: terminal,
                terminal_bos_admitted: terminal,
                terminal_bos_top8: terminal,
                terminal_bos_rank_sum: 0,
                terminal_bos_distance_sum: 0,
                terminal_content_top1: 0,
                nonterminal_bos_top1: nonterminal_bos,
                nonterminal_content_top1: content,
                nonterminal_bos_distance_sum: 0,
            },
            terminal_rows: rows,
        }
    }

    #[test]
    fn terminal_pairing_uses_discordant_margin() {
        let candidate = evaluation(vec![true, true, false, false], 0, 8);
        let control = evaluation(vec![false, false, true, false], 0, 8);
        let paired = paired_terminal_comparison(&candidate, &control).unwrap();
        assert_eq!(paired["gain"], 1);
        assert_eq!(paired["threshold"], 4);
        assert_eq!(paired["passes"], false);
    }

    #[test]
    fn terminal_pairing_rejects_different_row_counts() {
        assert!(paired_terminal_comparison(
            &evaluation(vec![true], 0, 8),
            &evaluation(vec![true, false], 0, 8),
        )
        .is_err());
    }

    #[test]
    fn output_root_must_be_fresh() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-termination-gain-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let out = root.join("run");
        create_fresh_output(&out).unwrap();
        assert!(create_fresh_output(&out).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
