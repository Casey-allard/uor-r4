//! Host-only tokenizer data preparation. Input files are separate documents.
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde_json::json;
use uor_r4_tokenizer::training::{BpeTrainer, TrainingLimits};

const USAGE: &str = "train-bpe --input UTF8_FILE [--input UTF8_FILE ...] --output NEW_TOKENIZER_JSON\n  [--vocab-size 16384] [--min-frequency 2] [--pre-tokenizer PRETOKENIZER_JSON]\n  [--max-input-bytes 67108864] [--max-unique-pretokens 1000000] [--max-pretoken-bytes 16384]\nThe optional JSON is a ByteLevel/Digits pre-tokenizer object, not an entire tokenizer.\nReserves BOS/EOS/UNK IDs0/1/2; actual size may be smaller if pairs are exhausted.";

fn main() {
    if let Err(error) = run(std::env::args_os().skip(1).collect()) {
        eprintln!("train-bpe: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<OsString>) -> Result<(), String> {
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let mut inputs = Vec::new();
    let mut output = None;
    let mut policy_file = None;
    let mut size = 16_384;
    let mut frequency = 2;
    let mut limits = TrainingLimits::default();
    let mut seen = std::collections::HashSet::new();
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].to_str().ok_or("option names must be UTF-8")?;
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {flag}\n{USAGE}"))?;
        if flag != "--input" && !seen.insert(flag.to_owned()) {
            return Err(format!("duplicate option {flag}"));
        }
        let number = || {
            value
                .to_str()
                .ok_or("numeric option must be UTF-8")?
                .parse::<usize>()
                .map_err(|_| "invalid nonnegative integer".to_string())
        };
        match flag {
            "--input" => inputs.push(PathBuf::from(value)),
            "--output" => output = Some(PathBuf::from(value)),
            "--pre-tokenizer" => policy_file = Some(PathBuf::from(value)),
            "--vocab-size" => size = number()?,
            "--min-frequency" => {
                frequency = u64::try_from(number()?).map_err(|_| "frequency too large")?
            }
            "--max-input-bytes" => limits.max_input_bytes = number()?,
            "--max-unique-pretokens" => limits.max_unique_pretokens = number()?,
            "--max-pretoken-bytes" => limits.max_pretoken_bytes = number()?,
            _ => return Err(format!("unknown option {flag}\n{USAGE}")),
        }
        i += 2;
    }
    if inputs.is_empty() {
        return Err(format!("at least one --input is required\n{USAGE}"));
    }
    let output = output.ok_or_else(|| format!("--output is required\n{USAGE}"))?;
    if output.try_exists().map_err(|error| error.to_string())? {
        return Err("output already exists; choose a fresh path".into());
    }
    let policy = match policy_file {
        Some(path) => serde_json::from_slice(&read_bounded(&path, 64 * 1024)?)
            .map_err(|error| format!("invalid pre-tokenizer JSON: {error}"))?,
        None => json!({"type":"ByteLevel","add_prefix_space":false}),
    };
    let mut trainer = BpeTrainer::new(size, frequency, policy)?.with_limits(limits)?;
    let mut input_bytes = 0usize;
    // Finish validation and training before creating the output. Never truncate
    // or overwrite an existing corpus/tokenizer, including symlinks/hardlinks.
    for path in &inputs {
        let remaining = limits
            .max_input_bytes
            .checked_sub(input_bytes)
            .ok_or("input byte limit exceeded")?;
        let bytes = read_bounded(path, remaining)?;
        input_bytes = input_bytes
            .checked_add(bytes.len())
            .ok_or("input size overflow")?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| format!("{} is not UTF-8: {error}", path.display()))?;
        trainer.add_text(text)?;
    }
    let result = trainer.finish()?;
    let mut bytes = serde_json::to_vec(&result).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|error| format!("create {}: {error}", output.display()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("write {}: {error}; output may be partial", output.display()))?;
    eprintln!(
        "trained {} files / {} bytes; vocabulary {} of {}; merges {}; blake3:{}",
        inputs.len(),
        input_bytes,
        result["model"]["vocab"].as_object().map_or(0, |v| v.len()),
        size,
        result["model"]["merges"].as_array().map_or(0, |m| m.len()),
        blake3::hash(&bytes).to_hex()
    );
    Ok(())
}

fn read_bounded(path: &std::path::Path, max_bytes: usize) -> Result<Vec<u8>, String> {
    // Opening a FIFO itself waits for a writer; screen local paths before
    // opening, then recheck the descriptor. Inputs must not be replaced by
    // another process during this local data-preparation command.
    if !std::fs::metadata(path)
        .map_err(|error| format!("stat {}: {error}", path.display()))?
        .is_file()
    {
        return Err(format!("{} must be a regular file", path.display()));
    }
    let file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err(format!("{} must be a regular file", path.display()));
    }
    let limit = u64::try_from(max_bytes)
        .map_err(|_| "input limit too large")?
        .checked_add(1)
        .ok_or("input limit too large")?;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err(format!("{} exceeds input byte limit", path.display()));
    }
    Ok(bytes)
}
