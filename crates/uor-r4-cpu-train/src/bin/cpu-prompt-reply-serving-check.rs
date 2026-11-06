use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    derive_prompt_retrieval_memory, evaluate_prompt_retrieval, normalized_reference,
    shuffle_prompt_retrieval_replies, validate_corpus_manifest, CpuCorpusIdentity, CpuMaskedCorpus,
    MemoryWeightSample, PromptRetrievalArm,
};
use uor_r4_integer::stack::{IntegerStackModel, PromptReplyReadout};

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-prompt-reply-serving-check: {error}");
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
    let (sidecar_bytes, readout) = PromptReplyReadout::read_sidecar(
        &config.sidecar,
        &artifact_sha256,
        &tokenizer_sha256,
        config.protocol,
        &[0, 1, 2],
        IntegerStackModel::parse(&artifact)
            .map_err(|error| error.to_string())?
            .shape()
            .vocab,
    )
    .map_err(|error| error.to_string())?;
    if readout.entries() != 453 {
        return Err(format!(
            "prompt-reply sidecar has {} entries, expected 453",
            readout.entries()
        ));
    }
    let (_, train_samples) = load_panel(&config.train)?;
    let cpu_memory = derive_prompt_retrieval_memory(&artifact, &train_samples, 64)
        .map_err(|error| error.to_string())?;
    let (cpu_shuffled, _) =
        shuffle_prompt_retrieval_replies(&cpu_memory, 207).map_err(|error| error.to_string())?;
    let (identity, samples) = load_panel(&config.panel)?;
    let cpu = evaluate_prompt_retrieval(
        &artifact,
        &cpu_memory,
        &cpu_shuffled,
        &samples,
        64,
        PromptRetrievalArm::UnigramJaccardOrdinalTie,
    )
    .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut rows = Vec::with_capacity(samples.len());
    for (row, sample) in samples.iter().enumerate() {
        rows.push(evaluate_row(&readout, sample, row, identity.vocab)?);
    }
    let elapsed_us = started.elapsed().as_micros().try_into().unwrap_or(u64::MAX);
    let reading = summarize(&rows);
    if reading.reference_reachable < 200 || reading.emitted < reading.reference_reachable {
        return Err(format!(
            "serving retrieval power or coverage failed: {reading:?}"
        ));
    }
    if reading.exact_reference != 39
        || reading.opening_correct != 131
        || reading.correct_body_positions != 948
        || reading.premature_bos != 0
    {
        return Err(format!(
            "serving retrieval differs from sealed CPU result: {reading:?}"
        ));
    }
    if reading.exact_reference != cpu.reading.exact_reference
        || reading.opening_correct != cpu.reading.opening_correct
        || reading.correct_body_positions != cpu.reading.correct_body_positions
        || cpu.steps.len() != rows.len()
    {
        return Err(format!(
            "serving aggregate differs from CPU arm: serving={reading:?}, cpu={:?}",
            cpu.reading
        ));
    }
    for (row, step) in rows.iter().zip(&cpu.steps) {
        if row.prompt_ordinal != Some(step.selected_ordinal)
            || cpu.rows[row.row].reply_sha256 != row.reply_sha256
        {
            return Err(format!(
                "serving decision differs from CPU arm at row {}",
                row.row
            ));
        }
    }
    let first_trace = selection_trace(&rows);
    for (row, sample) in samples.iter().enumerate() {
        let again = evaluate_row(&readout, sample, row, identity.vocab)?;
        if again != rows[row] {
            return Err(format!("serving retrieval repeat differs at row {row}"));
        }
    }
    if selection_trace(&rows) != first_trace {
        return Err("serving retrieval decision trace differs".to_owned());
    }
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-prompt-reply-serving-check/1",
        "substrate": "cpu-development-research",
        "qualification": "development-only",
        "artifact_sha256": artifact_sha256,
        "tokenizer_sha256": tokenizer_sha256,
        "sidecar_sha256": uor_r4_discrete::corpus::sha256(&sidecar_bytes),
        "sidecar_bytes": sidecar_bytes.len(),
        "entries": readout.entries(),
        "protocol": readout.protocol_version(),
        "panel": panel_input(&config.panel),
        "train": panel_input(&config.train),
        "cpu_arm": cpu.reading,
        "reading": reading,
        "selection_elapsed_us": elapsed_us,
        "prompts_per_second": samples.len() as f64 * 1_000_000.0 / elapsed_us as f64,
        "decision_sha256": selection_trace(&rows),
        "repeat_exact": true,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
struct Row {
    row: usize,
    reachable: bool,
    emitted: bool,
    exact_reference: bool,
    opening_correct: bool,
    correct_body_positions: u64,
    reply_tokens: u64,
    reference_tokens: u64,
    prompt_ordinal: Option<usize>,
    similarity_numerator: u64,
    similarity_denominator: u64,
    reply_sha256: String,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
struct Reading {
    replies: u64,
    reference_reachable: u64,
    emitted: u64,
    exact_reference: u64,
    opening_correct: u64,
    correct_body_positions: u64,
    generated_tokens: u64,
    reference_tokens: u64,
    premature_bos: u64,
}

fn evaluate_row(
    readout: &PromptReplyReadout,
    sample: &MemoryWeightSample,
    row: usize,
    _vocab: usize,
) -> Result<Row, String> {
    let reference = normalized_reference(&sample.reply);
    let reachable = reference.len() < 64;
    let selection = readout.select(&sample.prompt);
    let emitted = selection.is_some();
    let reply = selection
        .as_ref()
        .map(|selection| selection.reply.as_slice())
        .unwrap_or(&[]);
    Ok(Row {
        row,
        reachable,
        emitted,
        exact_reference: reply == reference,
        opening_correct: matches!(
            (reply.first(), reference.first()),
            (Some(left), Some(right)) if left == right
        ),
        correct_body_positions: reply
            .iter()
            .zip(reference)
            .filter(|(left, right)| left == right)
            .count() as u64,
        reply_tokens: reply.len() as u64,
        reference_tokens: reference.len() as u64,
        prompt_ordinal: selection.as_ref().map(|selection| selection.prompt_ordinal),
        similarity_numerator: selection
            .as_ref()
            .map_or(0, |selection| selection.similarity_numerator),
        similarity_denominator: selection
            .as_ref()
            .map_or(0, |selection| selection.similarity_denominator),
        reply_sha256: token_hash(reply),
    })
}

fn summarize(rows: &[Row]) -> Reading {
    let mut reading = Reading {
        replies: rows.len() as u64,
        ..Reading::default()
    };
    for row in rows {
        reading.reference_reachable += u64::from(row.reachable);
        reading.emitted += u64::from(row.emitted);
        reading.exact_reference += u64::from(row.exact_reference);
        reading.opening_correct += u64::from(row.opening_correct);
        reading.correct_body_positions += row.correct_body_positions;
        reading.generated_tokens += row.reply_tokens;
        reading.reference_tokens += row.reference_tokens;
    }
    reading
}

fn selection_trace(rows: &[Row]) -> String {
    let mut hasher = blake3::Hasher::new();
    for row in rows {
        hasher.update(&(row.row as u64).to_le_bytes());
        hasher.update(&(row.prompt_ordinal.unwrap_or(usize::MAX) as u64).to_le_bytes());
        hasher.update(&row.similarity_numerator.to_le_bytes());
        hasher.update(&row.similarity_denominator.to_le_bytes());
        hasher.update(row.reply_sha256.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn token_hash(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
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
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut expected = BTreeMap::new();
    let mut protocol = 2u8;
    while let Some(flag) = args.next() {
        if !flag.starts_with("--") {
            positional.push(PathBuf::from(flag));
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
    "usage: cpu-prompt-reply-serving-check SIDECAR TRAIN_DIR PANEL_DIR ARTIFACT TOKENIZER --protocol 1|2 --expected-artifact SHA --expected-tokenizer SHA"
}
