use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uor_r4_cpu_train::{
    compare_prompt_reply_rank_rows, derive_prompt_reply_rank_memory, evaluate_prompt_reply_ranking,
    fit_prompt_reply_ranker, normalized_reference, validate_corpus_manifest, CpuCorpusIdentity,
    CpuMaskedCorpus, MemoryWeightSample, PromptReplyRankArm, PROMPT_REPLY_RANK_FEATURE_NAMES,
    PROMPT_REPLY_RANK_SHUFFLE_SEED, PROMPT_REPLY_RANK_TOP_K,
};
use uor_r4_integer::stack::{PromptReplyExample, MAX_PROMPT_REPLY_SIDECAR_BYTES};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-prompt-reply-ranker: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let artifact = read(&config.artifact)?;
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    if artifact_sha256 != config.expected_artifact_sha256 {
        return Err(format!(
            "artifact sha256 {artifact_sha256} != expected {}",
            config.expected_artifact_sha256
        ));
    }
    let tokenizer = read(&config.tokenizer)?;
    let tokenizer_sha256 = uor_r4_discrete::corpus::sha256(&tokenizer);
    if tokenizer_sha256 != config.expected_tokenizer_sha256 {
        return Err(format!(
            "tokenizer sha256 {tokenizer_sha256} != expected {}",
            config.expected_tokenizer_sha256
        ));
    }
    let sidecar_bytes = read(&config.sidecar)?;
    if sidecar_bytes.len() as u64 > MAX_PROMPT_REPLY_SIDECAR_BYTES {
        return Err("sidecar exceeds its byte limit".to_owned());
    }
    let sidecar: Sidecar =
        serde_json::from_slice(&sidecar_bytes).map_err(|error| format!("sidecar JSON: {error}"))?;
    if sidecar.schema != "uor-r4.stack-prompt-reply-readout/1"
        || sidecar.artifact_sha256 != artifact_sha256
        || sidecar.tokenizer_sha256 != tokenizer_sha256
        || sidecar.protocol_version != config.protocol
        || sidecar.ignore_token_ids != [0, 1, 2]
    {
        return Err("sidecar identity does not match the artifact/protocol".to_owned());
    }
    if sidecar.entries.len() != 453 {
        return Err(format!(
            "sidecar bytes/entries differ: {} / {}",
            sidecar_bytes.len(),
            sidecar.entries.len()
        ));
    }
    let (_, train_samples) = load_panel(&config.train)?;
    if train_samples.len() != sidecar.entries.len()
        || train_samples
            .iter()
            .zip(&sidecar.entries)
            .any(|(sample, entry)| {
                sample.prompt.iter().copied().collect::<BTreeSet<_>>()
                    != entry.prompt.iter().copied().collect::<BTreeSet<_>>()
                    || normalized_reference(&sample.reply) != entry.reply
            })
    {
        return Err("sidecar prompt-set/reply identity differs from TRAIN panel".to_owned());
    }
    let train_examples = train_samples
        .iter()
        .map(|sample| MemoryWeightSample {
            prompt: sample.prompt.clone(),
            reply: normalized_reference(&sample.reply).to_vec(),
        })
        .collect::<Vec<_>>();
    let memory = derive_prompt_reply_rank_memory(&artifact, &train_examples, 64)
        .map_err(|error| error.to_string())?;
    let training = fit_prompt_reply_ranker(&memory).map_err(|error| error.to_string())?;
    if config.train_only {
        let output = serde_json::json!({
            "schema": "uor-r4.cpu-prompt-reply-ranker-preflight/1",
            "artifact_sha256": artifact_sha256,
            "tokenizer_sha256": tokenizer_sha256,
            "sidecar_sha256": uor_r4_discrete::corpus::sha256(&sidecar_bytes),
            "train": panel_input(&config.train),
            "training": training,
            "feature_names": PROMPT_REPLY_RANK_FEATURE_NAMES,
            "top_k": PROMPT_REPLY_RANK_TOP_K,
            "shuffle_seed": PROMPT_REPLY_RANK_SHUFFLE_SEED,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
        );
        return Ok(());
    }
    let (_, samples) = load_panel(&config.panel)?;
    let mut outputs = BTreeMap::new();
    for arm in PromptReplyRankArm::EVALUATION_ARMS {
        let first = evaluate_prompt_reply_ranking(&memory, &training, &samples, 64, arm)
            .map_err(|error| error.to_string())?;
        let second = evaluate_prompt_reply_ranking(&memory, &training, &samples, 64, arm)
            .map_err(|error| error.to_string())?;
        if first.reading != second.reading
            || first.reply_sha256 != second.reply_sha256
            || first.decision_sha256 != second.decision_sha256
        {
            return Err(format!("{arm:?} repeat differed"));
        }
        outputs.insert(arm.name().to_owned(), first);
    }
    let learned = outputs
        .get("learned_ranker")
        .ok_or_else(|| "learned output is missing".to_owned())?;
    let lexical = outputs
        .get("learned_lexical")
        .ok_or_else(|| "lexical output is missing".to_owned())?;
    let raw = outputs
        .get("raw_unigram_jaccard")
        .ok_or_else(|| "raw output is missing".to_owned())?;
    let shuffled = outputs
        .get("shuffled_ranker")
        .ok_or_else(|| "shuffled output is missing".to_owned())?;
    let shuffled_lexical = outputs
        .get("shuffled_lexical")
        .ok_or_else(|| "shuffled lexical output is missing".to_owned())?;
    let lexical_vs_raw_exact = compare_prompt_reply_rank_rows(
        &lexical.rows,
        &raw.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|error| error.to_string())?;
    let lexical_vs_raw_body = compare_prompt_reply_rank_rows(
        &lexical.rows,
        &raw.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::CorrectBodyPositions,
    )
    .map_err(|error| error.to_string())?;
    let lexical_vs_shuffled = compare_prompt_reply_rank_rows(
        &lexical.rows,
        &shuffled_lexical.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|error| error.to_string())?;
    let exact_vs_raw = compare_prompt_reply_rank_rows(
        &learned.rows,
        &raw.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|error| error.to_string())?;
    let body_vs_raw = compare_prompt_reply_rank_rows(
        &learned.rows,
        &raw.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::CorrectBodyPositions,
    )
    .map_err(|error| error.to_string())?;
    let exact_vs_shuffled = compare_prompt_reply_rank_rows(
        &learned.rows,
        &shuffled.rows,
        uor_r4_cpu_train::CopyKeyReplyMetric::ExactReference,
    )
    .map_err(|error| error.to_string())?;
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-prompt-reply-ranker/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "artifact_sha256": artifact_sha256,
        "tokenizer_sha256": tokenizer_sha256,
        "sidecar_sha256": uor_r4_discrete::corpus::sha256(&sidecar_bytes),
        "sidecar_bytes": sidecar_bytes.len(),
        "protocol": sidecar.protocol_version,
        "top_k": PROMPT_REPLY_RANK_TOP_K,
        "feature_names": PROMPT_REPLY_RANK_FEATURE_NAMES,
        "shuffle_seed": PROMPT_REPLY_RANK_SHUFFLE_SEED,
        "train": panel_input(&config.train),
        "panel": panel_input(&config.panel),
        "training": training,
        "arms": outputs,
        "comparisons": {
            "lexical_vs_raw_exact": lexical_vs_raw_exact,
            "lexical_vs_raw_body": lexical_vs_raw_body,
            "lexical_vs_shuffled_exact": lexical_vs_shuffled,
            "learned_vs_raw_exact": exact_vs_raw,
            "learned_vs_raw_body": body_vs_raw,
            "learned_vs_shuffled_exact": exact_vs_shuffled,
        },
        "repeat_exact": true,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sidecar {
    schema: String,
    artifact_sha256: String,
    tokenizer_sha256: String,
    protocol_version: u8,
    ignore_token_ids: Vec<u32>,
    entries: Vec<PromptReplyExample>,
}

fn load_panel(path: &Path) -> Result<(CpuCorpusIdentity, Vec<MemoryWeightSample>), String> {
    let token_bytes = read(&path.join("tokens.u16"))?;
    let mask = read(&path.join("response_mask.u8"))?;
    let manifest = read(&path.join("manifest.json"))?;
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
    let manifest = fs::read(path.join("manifest.json")).unwrap_or_default();
    let tokens = fs::read(path.join("tokens.u16")).unwrap_or_default();
    let mask = fs::read(path.join("response_mask.u8")).unwrap_or_default();
    serde_json::json!({
        "path": path,
        "manifest_sha256": uor_r4_discrete::corpus::sha256(&manifest),
        "tokens_sha256": uor_r4_discrete::corpus::sha256(&tokens),
        "mask_sha256": uor_r4_discrete::corpus::sha256(&mask),
    })
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

struct Config {
    sidecar: PathBuf,
    train: PathBuf,
    panel: PathBuf,
    artifact: PathBuf,
    tokenizer: PathBuf,
    protocol: u8,
    expected_artifact_sha256: String,
    expected_tokenizer_sha256: String,
    train_only: bool,
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut expected = BTreeMap::new();
    let mut protocol = 2u8;
    let mut train_only = false;
    while let Some(flag) = args.next() {
        if !flag.starts_with("--") {
            positional.push(PathBuf::from(flag));
            continue;
        }
        if flag == "--train-only" {
            train_only = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))?;
        match flag.as_str() {
            "--protocol" => {
                protocol = value
                    .parse()
                    .map_err(|_| format!("not a protocol version: {value}"))?
            }
            "--expected-artifact" => set(&mut expected, "artifact", value, &flag)?,
            "--expected-tokenizer" => set(&mut expected, "tokenizer", value, &flag)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if positional.len() != 5
        || !(1..=2).contains(&protocol)
        || !expected.contains_key("artifact")
        || !expected.contains_key("tokenizer")
    {
        return Err(usage().to_owned());
    }
    Ok(Config {
        sidecar: positional[0].clone(),
        train: positional[1].clone(),
        panel: positional[2].clone(),
        artifact: positional[3].clone(),
        tokenizer: positional[4].clone(),
        protocol,
        expected_artifact_sha256: expected["artifact"].clone(),
        expected_tokenizer_sha256: expected["tokenizer"].clone(),
        train_only,
    })
}

fn set(
    expected: &mut BTreeMap<String, String>,
    key: &str,
    value: String,
    flag: &str,
) -> Result<(), String> {
    if expected.insert(key.to_owned(), value).is_some() {
        return Err(format!("duplicate {flag}"));
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: cpu-prompt-reply-ranker SIDECAR TRAIN_DIR PANEL_DIR ARTIFACT TOKENIZER --protocol 1|2 --expected-artifact SHA --expected-tokenizer SHA [--train-only]"
}
