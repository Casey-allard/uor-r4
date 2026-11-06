use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use serde::Deserialize;
use uor_r4_cpu_train::{
    compare_copy_key_reply_rows, generate_reply, generate_reply_with_prefix, normalized_reference,
    validate_corpus_manifest, CopyKeyReplyMetric, CopyKeyReplyRow, CpuMaskedCorpus,
    MemoryWeightSample,
};
use uor_r4_integer::stack::PromptReplyExample;

const CAP: usize = 64;
const PREFIXES: [usize; 3] = [1, 2, 4];
const LENGTH_PREFIX: usize = 4;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-prompt-prefix-continuation: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let artifact = std::fs::read(&config.artifact).map_err(|e| e.to_string())?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    if artifact_sha256 != config.expected_artifact {
        return Err("artifact hash differs".to_owned());
    }
    let tokenizer = std::fs::read(&config.tokenizer).map_err(|e| e.to_string())?;
    let tokenizer_sha256 = uor_r4_discrete::corpus::sha256(&tokenizer);
    if tokenizer_sha256 != config.expected_tokenizer {
        return Err("tokenizer hash differs".to_owned());
    }
    let sidecar_bytes = std::fs::read(&config.sidecar).map_err(|e| e.to_string())?;
    let sidecar: Sidecar =
        serde_json::from_slice(&sidecar_bytes).map_err(|e| format!("sidecar: {e}"))?;
    if sidecar.artifact_sha256 != artifact_sha256
        || sidecar.tokenizer_sha256 != tokenizer_sha256
        || sidecar.protocol_version != config.protocol
        || sidecar.ignore_token_ids != [0, 1, 2]
    {
        return Err("sidecar identity differs".to_owned());
    }
    let (_, train) = load_panel(&config.train)?;
    if train.len() != sidecar.entries.len()
        || train.iter().zip(&sidecar.entries).any(|(sample, entry)| {
            sample
                .prompt
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                != entry
                    .prompt
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                || normalized_reference(&sample.reply) != entry.reply
        })
    {
        return Err("sidecar does not match TRAIN".to_owned());
    }
    let memory = Memory::new(
        train
            .iter()
            .map(|sample| {
                (
                    sample.prompt.clone(),
                    normalized_reference(&sample.reply).to_vec(),
                )
            })
            .collect(),
    );
    let (_, panel) = load_panel(&config.panel)?;
    let output = if config.length_completion {
        length_completion_output(
            &artifact,
            &artifact_sha256,
            &tokenizer_sha256,
            &sidecar_bytes,
            &config,
            &memory,
            &panel,
        )?
    } else {
        prefix_continuation_output(
            &artifact,
            &artifact_sha256,
            &tokenizer_sha256,
            &sidecar_bytes,
            &config,
            &memory,
            &panel,
        )?
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn prefix_continuation_output(
    artifact: &[u8],
    artifact_sha256: &str,
    tokenizer_sha256: &str,
    sidecar_bytes: &[u8],
    config: &Config,
    memory: &Memory,
    panel: &[MemoryWeightSample],
) -> Result<serde_json::Value, String> {
    let selected_k = choose_prefix(memory)?;
    let arms = evaluate_arms(artifact, memory, panel)?;
    let selected = &arms[&format!("retrieved_prefix_{selected_k}")];
    let model = &arms["model_only"];
    let shuffled = &arms[&format!("shuffled_prefix_{selected_k}")];
    let exact_vs_model = compare_copy_key_reply_rows(
        &selected.rows,
        &model.rows,
        CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|e| e.to_string())?;
    let body_vs_model = compare_copy_key_reply_rows(
        &selected.rows,
        &model.rows,
        CopyKeyReplyMetric::CorrectBodyPositions,
    )
    .map_err(|e| e.to_string())?;
    let exact_vs_shuffled = compare_copy_key_reply_rows(
        &selected.rows,
        &shuffled.rows,
        CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "schema": "uor-r4.cpu-prompt-prefix-continuation/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "artifact_sha256": artifact_sha256,
        "tokenizer_sha256": tokenizer_sha256,
        "sidecar_sha256": uor_r4_discrete::corpus::sha256(sidecar_bytes),
        "train": panel_input(&config.train),
        "panel": panel_input(&config.panel),
        "selected_prefix": selected_k,
        "arms": arms.into_iter().map(|(name, result)| (name, serde_json::json!({
            "reading": result.reading,
            "reply_sha256": result.reply_sha256,
            "decision_sha256": result.decision_sha256,
        }))).collect::<BTreeMap<_, _>>(),
        "comparisons": {
            "selected_vs_model_exact": exact_vs_model,
            "selected_vs_model_body": body_vs_model,
            "selected_vs_shuffled_exact": exact_vs_shuffled,
        },
    }))
}

fn length_completion_output(
    artifact: &[u8],
    artifact_sha256: &str,
    tokenizer_sha256: &str,
    sidecar_bytes: &[u8],
    config: &Config,
    memory: &Memory,
    panel: &[MemoryWeightSample],
) -> Result<serde_json::Value, String> {
    let arms = evaluate_length_arms(artifact, memory, panel)?;
    let selected = &arms[&format!("retrieved_prefix_{LENGTH_PREFIX}_length")];
    let model = &arms["model_only_retrieved_length"];
    let shuffled = &arms[&format!("shuffled_prefix_{LENGTH_PREFIX}_length")];
    let exact_vs_model = compare_copy_key_reply_rows(
        &selected.rows,
        &model.rows,
        CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|e| e.to_string())?;
    let body_vs_model = compare_copy_key_reply_rows(
        &selected.rows,
        &model.rows,
        CopyKeyReplyMetric::CorrectBodyPositions,
    )
    .map_err(|e| e.to_string())?;
    let exact_vs_shuffled = compare_copy_key_reply_rows(
        &selected.rows,
        &shuffled.rows,
        CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "schema": "uor-r4.cpu-prompt-prefix-length/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "artifact_sha256": artifact_sha256,
        "tokenizer_sha256": tokenizer_sha256,
        "sidecar_sha256": uor_r4_discrete::corpus::sha256(sidecar_bytes),
        "train": panel_input(&config.train),
        "panel": panel_input(&config.panel),
        "selected_prefix": LENGTH_PREFIX,
        "length_signal": "nearest stored reply length, capped at 64",
        "arms": arms.into_iter().map(|(name, result)| (name, serde_json::json!({
            "reading": result.reading,
            "reply_sha256": result.reply_sha256,
            "decision_sha256": result.decision_sha256,
        }))).collect::<BTreeMap<_, _>>(),
        "comparisons": {
            "selected_vs_model_exact": exact_vs_model,
            "selected_vs_model_body": body_vs_model,
            "selected_vs_shuffled_exact": exact_vs_shuffled,
        },
    }))
}

#[derive(Clone)]
struct Memory {
    prompts: Vec<std::collections::BTreeSet<u32>>,
    replies: Vec<Vec<u32>>,
}

impl Memory {
    fn new(entries: Vec<(Vec<u32>, Vec<u32>)>) -> Self {
        Self {
            prompts: entries
                .iter()
                .map(|(prompt, _)| prompt.iter().copied().collect())
                .collect(),
            replies: entries.into_iter().map(|(_, reply)| reply).collect(),
        }
    }

    fn select(&self, prompt: &[u32], exclude: Option<usize>) -> usize {
        let query = prompt
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        self.prompts
            .iter()
            .enumerate()
            .filter(|(index, _)| exclude != Some(*index))
            .min_by(|(left_index, left), (right_index, right)| {
                let left_n = query.intersection(left).count() as u128;
                let left_d = query.union(left).count() as u128;
                let right_n = query.intersection(right).count() as u128;
                let right_d = query.union(right).count() as u128;
                (right_n * left_d)
                    .cmp(&(left_n * right_d))
                    .then(left_index.cmp(right_index))
            })
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

#[derive(Clone, serde::Serialize)]
struct Reading {
    replies: u64,
    reachable: u64,
    emitted: u64,
    exact: u64,
    opening: u64,
    body: u64,
    premature_bos: u64,
    bos_complete: u64,
    length_complete: u64,
    cap_stopped: u64,
    context_stopped: u64,
    generated_tokens: u64,
}

struct ResultRows {
    reading: Reading,
    rows: Vec<CopyKeyReplyRow>,
    reply_sha256: String,
    decision_sha256: String,
}

fn choose_prefix(memory: &Memory) -> Result<usize, String> {
    let scores = PREFIXES
        .iter()
        .map(|&k| {
            let matches = memory
                .replies
                .iter()
                .enumerate()
                .map(|(row, reply)| {
                    let selected = memory.select(
                        &memory.prompts[row].iter().copied().collect::<Vec<_>>(),
                        Some(row),
                    );
                    memory.replies[selected]
                        .iter()
                        .zip(reply)
                        .take(k)
                        .filter(|(left, right)| left == right)
                        .count()
                })
                .sum::<usize>();
            (matches, k)
        })
        .max_by_key(|&(matches, k)| (matches, usize::MAX - k))
        .ok_or_else(|| "no prefix candidates".to_owned())?;
    Ok(scores.1)
}

fn evaluate_arms(
    artifact: &[u8],
    memory: &Memory,
    samples: &[MemoryWeightSample],
) -> Result<BTreeMap<String, ResultRows>, String> {
    let mut arms = vec![
        ("model_only".to_owned(), Arm::ModelOnly),
        ("full_retrieval".to_owned(), Arm::FullRetrieval),
    ];
    for k in PREFIXES {
        arms.push((format!("retrieved_prefix_{k}"), Arm::Retrieved(k)));
        arms.push((format!("shuffled_prefix_{k}"), Arm::Shuffled(k)));
        arms.push((format!("reference_prefix_{k}"), Arm::Reference(k)));
    }
    arms.into_iter()
        .map(|(name, arm)| Ok((name, evaluate(artifact, memory, samples, arm)?)))
        .collect()
}

fn evaluate_length_arms(
    artifact: &[u8],
    memory: &Memory,
    samples: &[MemoryWeightSample],
) -> Result<BTreeMap<String, ResultRows>, String> {
    let arms = vec![
        ("model_only_cap".to_owned(), LengthArm::ModelOnlyCap),
        (
            "model_only_retrieved_length".to_owned(),
            LengthArm::ModelOnlyRetrievedLength,
        ),
        (
            format!("retrieved_prefix_{LENGTH_PREFIX}_length"),
            LengthArm::RetrievedLength,
        ),
        (
            format!("shuffled_prefix_{LENGTH_PREFIX}_length"),
            LengthArm::ShuffledLength,
        ),
        ("full_retrieval".to_owned(), LengthArm::FullRetrieval),
        (
            "reference_length_oracle".to_owned(),
            LengthArm::ReferenceLength,
        ),
    ];
    arms.into_iter()
        .map(|(name, arm)| Ok((name, evaluate_length(artifact, memory, samples, arm)?)))
        .collect()
}

#[derive(Clone, Copy)]
enum LengthArm {
    ModelOnlyCap,
    ModelOnlyRetrievedLength,
    RetrievedLength,
    ShuffledLength,
    FullRetrieval,
    ReferenceLength,
}

fn evaluate_length(
    artifact: &[u8],
    memory: &Memory,
    samples: &[MemoryWeightSample],
    arm: LengthArm,
) -> Result<ResultRows, String> {
    let started = Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply);
            let ordinal = memory.select(&sample.prompt, None);
            let retrieved = &memory.replies[ordinal];
            if retrieved.is_empty() {
                return Err(format!("row {row}: retrieved reply is empty"));
            }
            let retrieved_limit = retrieved.len().min(CAP);
            let (tokens, raw_stop, requested_limit, exact_without_bos) = match arm {
                LengthArm::ModelOnlyCap => {
                    let reply =
                        generate_reply(artifact, &sample.prompt, CAP).map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop, CAP, false)
                }
                LengthArm::ModelOnlyRetrievedLength => {
                    let reply = generate_reply(artifact, &sample.prompt, retrieved_limit)
                        .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop, retrieved_limit, true)
                }
                LengthArm::RetrievedLength => {
                    let prefix = &retrieved[..retrieved.len().min(LENGTH_PREFIX)];
                    let reply = generate_reply_with_prefix(
                        artifact,
                        &sample.prompt,
                        prefix,
                        retrieved_limit,
                    )
                    .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop, retrieved_limit, true)
                }
                LengthArm::ShuffledLength => {
                    let shuffled = next_distinct(memory, ordinal, LENGTH_PREFIX);
                    let prefix = &memory.replies[shuffled]
                        [..memory.replies[shuffled].len().min(LENGTH_PREFIX)];
                    let reply = generate_reply_with_prefix(
                        artifact,
                        &sample.prompt,
                        prefix,
                        retrieved_limit,
                    )
                    .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop, retrieved_limit, true)
                }
                LengthArm::FullRetrieval => (retrieved.clone(), "retrieval", 0, true),
                LengthArm::ReferenceLength => {
                    if reference.is_empty() {
                        return Err(format!("row {row}: reference reply is empty"));
                    }
                    let reference_limit = reference.len().min(CAP);
                    let reply = generate_reply(artifact, &sample.prompt, reference_limit)
                        .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop, reference_limit, true)
                }
            };
            Ok(make_reply_row(
                row,
                sample,
                reference,
                &tokens,
                raw_stop,
                requested_limit,
                exact_without_bos,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    summarize_rows(evaluated, started)
}

#[derive(Clone, Copy)]
enum Arm {
    ModelOnly,
    FullRetrieval,
    Retrieved(usize),
    Shuffled(usize),
    Reference(usize),
}

fn evaluate(
    artifact: &[u8],
    memory: &Memory,
    samples: &[MemoryWeightSample],
    arm: Arm,
) -> Result<ResultRows, String> {
    let started = Instant::now();
    let evaluated = samples
        .par_iter()
        .enumerate()
        .map(|(row, sample)| {
            let reference = normalized_reference(&sample.reply);
            let ordinal = memory.select(&sample.prompt, None);
            let (tokens, stop) = match arm {
                Arm::ModelOnly => {
                    let reply =
                        generate_reply(artifact, &sample.prompt, CAP).map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop)
                }
                Arm::FullRetrieval => (memory.replies[ordinal].clone(), "bos"),
                Arm::Retrieved(k) => {
                    let reply = generate_reply_with_prefix(
                        artifact,
                        &sample.prompt,
                        &memory.replies[ordinal][..memory.replies[ordinal].len().min(k)],
                        CAP,
                    )
                    .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop)
                }
                Arm::Shuffled(k) => {
                    let shuffled = next_distinct(memory, ordinal, k);
                    let reply = generate_reply_with_prefix(
                        artifact,
                        &sample.prompt,
                        &memory.replies[shuffled][..memory.replies[shuffled].len().min(k)],
                        CAP,
                    )
                    .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop)
                }
                Arm::Reference(k) => {
                    let reply = generate_reply_with_prefix(
                        artifact,
                        &sample.prompt,
                        &reference[..reference.len().min(k)],
                        CAP,
                    )
                    .map_err(|e| e.to_string())?;
                    (reply.tokens, reply.stop)
                }
            };
            Ok(make_reply_row(
                row, sample, reference, &tokens, stop, 0, false,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    summarize_rows(evaluated, started)
}

fn make_reply_row(
    row: usize,
    sample: &MemoryWeightSample,
    reference: &[u32],
    tokens: &[u32],
    raw_stop: &str,
    requested_limit: usize,
    exact_without_bos: bool,
) -> CopyKeyReplyRow {
    let stop = if raw_stop == "retrieval" {
        "retrieval"
    } else if exact_without_bos && raw_stop == "cap" && tokens.len() == requested_limit {
        "length"
    } else {
        raw_stop
    };
    let reachable = reference.len() < CAP && sample.prompt.len() + reference.len() < 256;
    let exact = if exact_without_bos {
        tokens == reference
    } else {
        stop == "bos" && tokens == reference
    };
    let opening = tokens.first() == reference.first() && !tokens.is_empty();
    let body = tokens
        .iter()
        .zip(reference)
        .filter(|(left, right)| left == right)
        .count() as u64;
    let complete = matches!(stop, "bos" | "length" | "retrieval");
    CopyKeyReplyRow {
        row,
        reachable,
        exact_reference: exact,
        bos_complete: stop == "bos",
        opening_correct: opening,
        premature_bos: stop == "bos" && tokens.len() < reference.len(),
        non_premature_bos: stop == "bos" && tokens.len() >= reference.len(),
        generated_tokens: tokens.len() as u64,
        reference_tokens: reference.len() as u64,
        correct_body_positions: body,
        copy_decisions: u64::from(complete),
        fallback_decisions: u64::from(!complete),
        stop: stop.to_owned(),
        reply_sha256: token_hash(tokens),
    }
}

fn summarize_rows(evaluated: Vec<CopyKeyReplyRow>, started: Instant) -> Result<ResultRows, String> {
    let reading = Reading {
        replies: evaluated.len() as u64,
        reachable: evaluated.iter().filter(|row| row.reachable).count() as u64,
        emitted: evaluated
            .iter()
            .filter(|row| matches!(row.stop.as_str(), "bos" | "length" | "retrieval"))
            .count() as u64,
        exact: evaluated.iter().filter(|row| row.exact_reference).count() as u64,
        opening: evaluated.iter().filter(|row| row.opening_correct).count() as u64,
        body: evaluated.iter().map(|row| row.correct_body_positions).sum(),
        premature_bos: evaluated.iter().filter(|row| row.premature_bos).count() as u64,
        bos_complete: evaluated.iter().filter(|row| row.bos_complete).count() as u64,
        length_complete: evaluated.iter().filter(|row| row.stop == "length").count() as u64,
        cap_stopped: evaluated.iter().filter(|row| row.stop == "cap").count() as u64,
        context_stopped: evaluated.iter().filter(|row| row.stop == "context").count() as u64,
        generated_tokens: evaluated.iter().map(|row| row.generated_tokens).sum(),
    };
    let mut hasher = blake3::Hasher::new();
    for row in &evaluated {
        hasher.update(&(row.row as u64).to_le_bytes());
        hasher.update(&row.generated_tokens.to_le_bytes());
        hasher.update(row.stop.as_bytes());
        hasher.update(row.reply_sha256.as_bytes());
    }
    let decision_sha256 = hasher.finalize().to_hex().to_string();
    let mut reply_hasher = blake3::Hasher::new();
    for row in &evaluated {
        reply_hasher.update(row.reply_sha256.as_bytes());
    }
    let _elapsed = started.elapsed();
    Ok(ResultRows {
        reading,
        reply_sha256: reply_hasher.finalize().to_hex().to_string(),
        decision_sha256,
        rows: evaluated,
    })
}

fn next_distinct(memory: &Memory, ordinal: usize, k: usize) -> usize {
    for step in 1..memory.replies.len() {
        let candidate = (ordinal + step) % memory.replies.len();
        let left = &memory.replies[ordinal][..memory.replies[ordinal].len().min(k)];
        let right = &memory.replies[candidate][..memory.replies[candidate].len().min(k)];
        if left != right {
            return candidate;
        }
    }
    ordinal
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn load_panel(
    path: &Path,
) -> Result<(uor_r4_cpu_train::CpuCorpusIdentity, Vec<MemoryWeightSample>), String> {
    let token_bytes = std::fs::read(path.join("tokens.u16")).map_err(|e| e.to_string())?;
    let mask = std::fs::read(path.join("response_mask.u8")).map_err(|e| e.to_string())?;
    let manifest = std::fs::read(path.join("manifest.json")).map_err(|e| e.to_string())?;
    let identity =
        validate_corpus_manifest(&manifest, &token_bytes, &mask).map_err(|e| e.to_string())?;
    let corpus = CpuMaskedCorpus::from_bytes(
        &token_bytes,
        &mask,
        identity.vocab,
        identity.max_tokens,
        identity.bos,
    )
    .map_err(|e| e.to_string())?;
    let samples = corpus
        .conversation_examples()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt,
            reply: sample.reply,
        })
        .collect();
    Ok((identity, samples))
}

fn panel_input(path: &Path) -> serde_json::Value {
    let manifest = std::fs::read(path.join("manifest.json")).unwrap_or_default();
    let tokens = std::fs::read(path.join("tokens.u16")).unwrap_or_default();
    let mask = std::fs::read(path.join("response_mask.u8")).unwrap_or_default();
    serde_json::json!({
        "path": path,
        "manifest_sha256": uor_r4_discrete::corpus::sha256(&manifest),
        "tokens_sha256": uor_r4_discrete::corpus::sha256(&tokens),
        "mask_sha256": uor_r4_discrete::corpus::sha256(&mask),
    })
}

#[derive(Deserialize)]
struct Sidecar {
    artifact_sha256: String,
    tokenizer_sha256: String,
    protocol_version: u8,
    ignore_token_ids: Vec<u32>,
    entries: Vec<PromptReplyExample>,
}

struct Config {
    sidecar: PathBuf,
    train: PathBuf,
    panel: PathBuf,
    artifact: PathBuf,
    tokenizer: PathBuf,
    protocol: u8,
    expected_artifact: String,
    expected_tokenizer: String,
    length_completion: bool,
}

fn parse_args() -> Result<Config, String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 5 {
        return Err("usage: cpu-prompt-prefix-continuation SIDECAR TRAIN PANEL ARTIFACT TOKENIZER --expected-artifact SHA --expected-tokenizer SHA [--protocol N] [--length-completion]".to_owned());
    }
    let mut named = BTreeMap::new();
    let mut protocol = 2;
    let mut length_completion = false;
    let mut index = 5;
    while index < args.len() {
        match args[index].as_str() {
            "--length-completion" => {
                length_completion = true;
                index += 1;
                continue;
            }
            "--protocol" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--protocol needs value".to_owned())?;
                protocol = value.parse().map_err(|_| "bad protocol")?;
            }
            "--expected-artifact" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--expected-artifact needs value".to_owned())?;
                named.insert("artifact", value.clone());
            }
            "--expected-tokenizer" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--expected-tokenizer needs value".to_owned())?;
                named.insert("tokenizer", value.clone());
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 2;
    }
    Ok(Config {
        sidecar: args[0].clone().into(),
        train: args[1].clone().into(),
        panel: args[2].clone().into(),
        artifact: args[3].clone().into(),
        tokenizer: args[4].clone().into(),
        protocol,
        expected_artifact: named.remove("artifact").ok_or("missing artifact hash")?,
        expected_tokenizer: named.remove("tokenizer").ok_or("missing tokenizer hash")?,
        length_completion,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MemoryWeightSample {
        MemoryWeightSample {
            prompt: vec![10, 11, 12],
            reply: vec![20, 21, 22],
        }
    }

    #[test]
    fn retrieved_length_marks_an_identity_matched_length_stop_exact() {
        let sample = sample();
        let row = make_reply_row(7, &sample, &[20, 21, 22], &[20, 21, 22], "cap", 3, true);
        assert_eq!(row.stop, "length");
        assert!(row.reachable);
        assert!(row.exact_reference);
        assert!(!row.bos_complete);
        assert_eq!(row.copy_decisions, 1);
        assert_eq!(row.fallback_decisions, 0);
    }

    #[test]
    fn ordinary_bos_behavior_is_unchanged_without_length_exactness() {
        let sample = sample();
        let bos = make_reply_row(7, &sample, &[20, 21, 22], &[20, 21, 22], "bos", 0, false);
        assert_eq!(bos.stop, "bos");
        assert!(bos.exact_reference);
        assert!(bos.bos_complete);

        let length = make_reply_row(7, &sample, &[20, 21, 22], &[20, 21, 22], "cap", 3, false);
        assert_eq!(length.stop, "cap");
        assert!(!length.exact_reference);
        assert_eq!(length.fallback_decisions, 1);
    }

    #[test]
    fn retrieved_limit_is_positive_bounded_and_never_borrows_reference_length() {
        let stored_reply_length = 47;
        let reference_length = 3;
        assert_eq!(stored_reply_length.min(CAP), 47);
        assert_ne!(stored_reply_length.min(CAP), reference_length);
        assert_eq!(usize::MAX.min(CAP), 64);
    }

    #[test]
    fn premature_and_nonpremature_bos_remain_distinct() {
        let sample = sample();
        let premature = make_reply_row(1, &sample, &[20, 21, 22], &[20], "bos", 0, true);
        assert!(premature.premature_bos);
        assert!(!premature.non_premature_bos);

        let complete = make_reply_row(2, &sample, &[20, 21, 22], &[20, 21, 22], "bos", 0, true);
        assert!(!complete.premature_bos);
        assert!(complete.non_premature_bos);
    }
}
