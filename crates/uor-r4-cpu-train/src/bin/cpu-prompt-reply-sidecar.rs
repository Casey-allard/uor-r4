use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    normalized_reference, validate_corpus_manifest, CpuCorpusIdentity, CpuMaskedCorpus,
    MemoryWeightSample,
};
use uor_r4_integer::stack::{PromptReplyExample, PromptReplyReadout};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-prompt-reply-sidecar: {error}");
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
    let tokenizer_bytes = read(&config.tokenizer)?;
    let tokenizer_sha256 = uor_r4_discrete::corpus::sha256(&tokenizer_bytes);
    if tokenizer_sha256 != config.expected_tokenizer_sha256 {
        return Err(format!(
            "tokenizer sha256 {tokenizer_sha256} != expected {}",
            config.expected_tokenizer_sha256
        ));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "tokenizer.json is not a byte-level BPE tokenizer".to_owned())?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, config.protocol)
        .map_err(|error| error.to_string())?;
    let (identity, samples) = load_samples(&config.train)?;
    let entries = samples
        .iter()
        .map(|sample| {
            let reply = normalized_reference(&sample.reply).to_vec();
            PromptReplyExample {
                prompt: sample.prompt.clone(),
                reply,
            }
        })
        .collect::<Vec<_>>();
    let readout = PromptReplyReadout::from_entries(
        artifact_sha256.clone(),
        tokenizer_sha256.clone(),
        config.protocol,
        vec![protocol.bos_id, protocol.eos_id, protocol.unk_id],
        entries,
        identity.vocab,
    )
    .map_err(|error| error.to_string())?;
    let sidecar = readout
        .to_sidecar_bytes()
        .map_err(|error| error.to_string())?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&config.output)
        .map_err(|error| format!("{}: {error}", config.output.display()))?;
    output
        .write_all(&sidecar)
        .map_err(|error| format!("{}: {error}", config.output.display()))?;
    output
        .sync_all()
        .map_err(|error| format!("{}: {error}", config.output.display()))?;
    let written = read(&config.output)?;
    if written != sidecar {
        return Err("sidecar read-back differs".to_owned());
    }
    let reloaded = PromptReplyReadout::from_sidecar(
        &written,
        &artifact_sha256,
        &tokenizer_sha256,
        config.protocol,
        &[protocol.bos_id, protocol.eos_id, protocol.unk_id],
        identity.vocab,
    )
    .map_err(|error| error.to_string())?;
    let output = serde_json::json!({
        "schema": "uor-r4.cpu-prompt-reply-sidecar/1",
        "artifact_sha256": artifact_sha256,
        "tokenizer_sha256": tokenizer_sha256,
        "protocol": config.protocol,
        "entries": reloaded.entries(),
        "ignore_token_ids": reloaded.ignore_token_ids(),
        "sidecar_bytes": written.len(),
        "sidecar_sha256": uor_r4_discrete::corpus::sha256(&written),
        "output": config.output,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn load_samples(path: &Path) -> Result<(CpuCorpusIdentity, Vec<MemoryWeightSample>), String> {
    let tokens_path = path.join("tokens.u16");
    let mask_path = path.join("response_mask.u8");
    let manifest_path = path.join("manifest.json");
    let token_bytes = read(&tokens_path)?;
    let mask = read(&mask_path)?;
    let manifest = read(&manifest_path)?;
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

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

struct Config {
    train: PathBuf,
    artifact: PathBuf,
    tokenizer: PathBuf,
    output: PathBuf,
    protocol: u8,
    expected_artifact_sha256: String,
    expected_tokenizer_sha256: String,
}

fn parse_args() -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    let mut positional = Vec::new();
    let mut named = BTreeMap::new();
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
            "--out" => set(&mut named, "out", value, &flag)?,
            "--protocol" => {
                protocol = value
                    .parse()
                    .map_err(|_| format!("not a protocol version: {value}"))?
            }
            "--expected-artifact" => set(&mut named, "artifact", value, &flag)?,
            "--expected-tokenizer" => set(&mut named, "tokenizer", value, &flag)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    if positional.len() != 3 || !(1..=2).contains(&protocol) {
        return Err(usage().to_owned());
    }
    for key in ["out", "artifact", "tokenizer"] {
        if !named.contains_key(key) {
            return Err(format!("missing --{key}\n{}", usage()));
        }
    }
    Ok(Config {
        train: positional[0].clone(),
        artifact: positional[1].clone(),
        tokenizer: positional[2].clone(),
        output: PathBuf::from(&named["out"]),
        protocol,
        expected_artifact_sha256: named["artifact"].clone(),
        expected_tokenizer_sha256: named["tokenizer"].clone(),
    })
}

fn set(
    named: &mut BTreeMap<String, String>,
    key: &str,
    value: String,
    flag: &str,
) -> Result<(), String> {
    if named.insert(key.to_owned(), value).is_some() {
        return Err(format!("duplicate {flag}"));
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: cpu-prompt-reply-sidecar TRAIN_DIR ARTIFACT TOKENIZER --out FILE --protocol 1|2 --expected-artifact SHA --expected-tokenizer SHA"
}
