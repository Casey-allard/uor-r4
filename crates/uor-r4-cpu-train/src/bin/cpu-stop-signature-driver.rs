use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, derive_stop_signature_rows, derive_turn_memory, evaluate_replies,
    evaluate_stop_signature_no_gate, evaluate_stop_signature_replies, evaluate_stop_signature_rows,
    fit_stop_signature_gate, shuffle_stop_signature_rows, validate_corpus_manifest,
    CpuConversationSample, CpuMaskedCorpus, MemoryWeightSample, StopSignatureEvaluation,
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
    prototypes: usize,
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
            eprintln!("cpu-stop-signature-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-stop-signature-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--keys N] [--per-key N] [--key-tokens N] [--reply-turn N] \
     [--prototypes N] [--reply-cap N] [--seed N] [--select-only]"
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
        prototypes: 8,
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
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--prototypes" => config.prototypes = number(&value)?,
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
        || !(1..=32).contains(&config.prototypes)
        || !(1..=64).contains(&config.reply_cap)
    {
        return Err("group, prototype and reply dimensions are out of bounds".to_owned());
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
    let training_rows = derive_stop_signature_rows(&memory_artifact, &training_samples, true)
        .map_err(|error| error.to_string())?;
    let evaluation_rows = derive_stop_signature_rows(&memory_artifact, &evaluation_samples, true)
        .map_err(|error| error.to_string())?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-stop-signature-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "rows": {
                "training": rows_json(&training_rows),
                "evaluation": rows_json(&evaluation_rows),
            },
        }));
    }

    create_fresh_output(&config.out)?;
    std::fs::write(config.out.join("memory-baseline.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;
    let shuffled_rows = shuffle_stop_signature_rows(&training_rows, config.seed)
        .map_err(|error| error.to_string())?;
    let true_training = fit_stop_signature_gate(&training_rows, config.prototypes)
        .map_err(|error| error.to_string())?;
    let shuffled_training = fit_stop_signature_gate(&shuffled_rows, config.prototypes)
        .map_err(|error| error.to_string())?;
    let true_evaluation = evaluate_stop_signature_rows(&evaluation_rows, &true_training.gate)
        .map_err(|error| error.to_string())?;
    let shuffled_evaluation =
        evaluate_stop_signature_rows(&evaluation_rows, &shuffled_training.gate)
            .map_err(|error| error.to_string())?;
    let no_gate_evaluation =
        evaluate_stop_signature_no_gate(&evaluation_rows).map_err(|error| error.to_string())?;
    let vs_no_gate = paired_terminal_comparison(&true_evaluation, &no_gate_evaluation)?;
    let vs_shuffled = paired_terminal_comparison(&true_evaluation, &shuffled_evaluation)?;
    let terminal_required = true_evaluation.reading.terminal_decisions.div_ceil(4);
    let nonterminal_limit = true_evaluation.reading.nonterminal_decisions / 100;
    let free_no_gate = evaluate_replies(&memory_artifact, &evaluation_samples, config.reply_cap)
        .map_err(|error| error.to_string())?;
    let true_gate = evaluate_stop_signature_replies(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        &true_training.gate,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_gate = evaluate_stop_signature_replies(
        &memory_artifact,
        &evaluation_samples,
        config.reply_cap,
        true,
        &shuffled_training.gate,
    )
    .map_err(|error| error.to_string())?;
    let bar = serde_json::json!({
        "terminal_required": terminal_required,
        "terminal_stops": true_evaluation.reading.terminal_stops,
        "terminal_pass": true_evaluation.reading.terminal_stops >= terminal_required,
        "nonterminal_limit": nonterminal_limit,
        "nonterminal_stops": true_evaluation.reading.nonterminal_stops,
        "nonterminal_pass": true_evaluation.reading.nonterminal_stops <= nonterminal_limit,
        "gate_exposed": true_gate.gated_decisions > 0,
        "gate_fired": true_gate.fired > 0,
        "vs_no_gate": vs_no_gate,
        "vs_shuffled": vs_shuffled,
        "ungated_token_mismatches": true_evaluation.reading.ungated_token_mismatches,
        "passes": true_evaluation.reading.terminal_stops >= terminal_required
            && true_evaluation.reading.nonterminal_stops <= nonterminal_limit
            && true_gate.gated_decisions > 0
            && true_gate.fired > 0
            && vs_no_gate["passes"].as_bool() == Some(true)
            && vs_shuffled["passes"].as_bool() == Some(true)
            && true_evaluation.reading.ungated_token_mismatches == 0,
    });
    let true_gate_sha256 = serialized_sha(&true_training.gate)?;
    let shuffled_gate_sha256 = serialized_sha(&shuffled_training.gate)?;
    std::fs::write(
        config.out.join("true-gate.json"),
        serde_json::to_vec_pretty(&true_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        config.out.join("shuffled-gate.json"),
        serde_json::to_vec_pretty(&shuffled_training).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-stop-signature-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "CPU-only admission-external stop candidate; not an integer-serving change or language capability",
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
        "memory": {
            "k": memory.k,
            "key_window": memory.key_window,
            "opening_rows": memory.opening.len(),
            "reply_rows": memory.reply.as_ref().map_or(0, |reply| reply.entries.len()),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
        },
        "training": {
            "prototypes_requested": config.prototypes,
            "true": true_training,
            "true_gate_sha256": true_gate_sha256,
            "label_shuffled": shuffled_training,
            "label_shuffled_gate_sha256": shuffled_gate_sha256,
        },
        "evaluation": {
            "no_gate": no_gate_evaluation,
            "true": true_evaluation,
            "label_shuffled": shuffled_evaluation,
        },
        "bar": bar,
        "free_generation": {
            "no_gate": free_no_gate,
            "true_gate": true_gate,
            "label_shuffled_gate": shuffled_gate,
        },
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

fn rows_json(rows: &[uor_r4_cpu_train::StopSignatureRow]) -> serde_json::Value {
    let terminal = rows.iter().filter(|row| row.terminal).count();
    serde_json::json!({
        "decisions": rows.len(),
        "terminal": terminal,
        "nonterminal": rows.len() - terminal,
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

fn paired_terminal_comparison(
    candidate: &StopSignatureEvaluation,
    control: &StopSignatureEvaluation,
) -> Result<serde_json::Value, String> {
    if candidate.terminal_rows.len() != control.terminal_rows.len() {
        return Err("terminal stop arms report different row counts".to_owned());
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

fn serialized_sha<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| uor_r4_discrete::corpus::sha256(&bytes))
        .map_err(|error| error.to_string())
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
    use uor_r4_cpu_train::{StopSignatureEvaluation, StopSignatureReading, STOP_SIGNATURE_WORDS};

    fn evaluation(rows: Vec<bool>, nonterminal_stops: u64) -> StopSignatureEvaluation {
        let terminal_stops = rows.iter().filter(|row| **row).count() as u64;
        StopSignatureEvaluation {
            reading: StopSignatureReading {
                decisions: 10,
                terminal_decisions: rows.len() as u64,
                nonterminal_decisions: 9,
                terminal_stops,
                nonterminal_stops,
                writer_fallbacks: 0,
                gated_stops: terminal_stops + nonterminal_stops,
                ungated_token_mismatches: 0,
            },
            terminal_rows: rows,
            decisions: Vec::new(),
        }
    }

    #[test]
    fn terminal_pairing_uses_discordant_margin() {
        let candidate = evaluation(vec![true, false, false, false], 0);
        let control = evaluation(vec![false, false, false, false], 0);
        let paired = paired_terminal_comparison(&candidate, &control).unwrap();
        assert_eq!(paired["gain"], 1);
        assert_eq!(paired["threshold"], 2);
        assert_eq!(paired["passes"], false);
    }

    #[test]
    fn terminal_pairing_rejects_different_rows() {
        assert!(
            paired_terminal_comparison(&evaluation(vec![true], 0), &evaluation(vec![], 0)).is_err()
        );
    }

    #[test]
    fn output_root_must_be_fresh() {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-stop-signature-{}-{}",
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

    #[test]
    fn signature_words_are_256_bits() {
        assert_eq!(STOP_SIGNATURE_WORDS * 64, 256);
    }
}
