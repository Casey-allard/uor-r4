//! Kneser-Ney 5-gram gate on real held-out text.
//!
//! The transition plan sets an n-gram/Kneser-Ney acceptance gate for native mechanisms. This
//! example measures that gate on the same token stream and the same byte accounting the model
//! evaluations use, so the two numbers are comparable:
//!
//! ```text
//! kn-ngram-gate --train <train.u16> --eval <heldout.u16> --tokenizer <tokenizer.json>
//!               [--train-tokens N] [--eval-tokens N] [--out report.json]
//! ```
//!
//! Bits per byte is computed over the tokenizer's own byte lengths, matching `ablate-prose` and
//! `lut4-prose`. No model is loaded; this is a statistical comparator, not native serving.

use std::path::PathBuf;
use std::time::Instant;

use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::ngram::{KneserNey5Gram, NgramConfig};

struct Args {
    train: PathBuf,
    eval: PathBuf,
    tokenizer: PathBuf,
    train_tokens: usize,
    eval_tokens: usize,
    /// Retained row caps for orders two to five. The default is KN's own capacity; a tiny value
    /// models a count table at the scale the served artifact's engram term actually carries
    /// (about ten thousand contexts), which is what isolates capacity from estimator quality.
    row_caps: [usize; 4],
    out: Option<PathBuf>,
}

fn usage() -> String {
    "kn-ngram-gate --train <train.u16> --eval <heldout.u16> --tokenizer <tokenizer.json>\n\
     \x20 [--train-tokens N] [--eval-tokens N] [--row-caps a,b,c,d] [--out report.json]"
        .to_string()
}

fn parse_args() -> Result<Args, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args {
        train: PathBuf::new(),
        eval: PathBuf::new(),
        tokenizer: PathBuf::new(),
        train_tokens: 20_000_000,
        eval_tokens: 200_000,
        row_caps: [4_000_000, 4_000_000, 8_000_000, 8_000_000],
        out: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let value = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("missing value for {}", argv[i]))
        };
        match argv[i].as_str() {
            "--train" => {
                args.train = PathBuf::from(value(i)?);
                i += 2;
            }
            "--eval" => {
                args.eval = PathBuf::from(value(i)?);
                i += 2;
            }
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(value(i)?);
                i += 2;
            }
            "--train-tokens" => {
                args.train_tokens = value(i)?
                    .parse()
                    .map_err(|e| format!("train-tokens: {e}"))?;
                i += 2;
            }
            "--eval-tokens" => {
                args.eval_tokens = value(i)?.parse().map_err(|e| format!("eval-tokens: {e}"))?;
                i += 2;
            }
            "--row-caps" => {
                let raw = value(i)?;
                let parts: Vec<&str> = raw.split(',').collect();
                if parts.len() != 4 {
                    return Err("--row-caps needs four comma-separated values".to_string());
                }
                for (slot, part) in parts.iter().enumerate() {
                    args.row_caps[slot] =
                        part.trim().parse().map_err(|e| format!("row-caps: {e}"))?;
                }
                i += 2;
            }
            "--out" => {
                args.out = Some(PathBuf::from(value(i)?));
                i += 2;
            }
            "--help" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    if args.train.as_os_str().is_empty() || args.eval.as_os_str().is_empty() {
        return Err(format!("--train and --eval are required\n\n{}", usage()));
    }
    Ok(args)
}

fn read_tokens(path: &PathBuf) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let raw = std::fs::read(path)?;
    Ok(raw
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect())
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    if let Err(error) = run(&args) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let train = read_tokens(&args.train)?;
    let eval = read_tokens(&args.eval)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&std::fs::read(&args.tokenizer)?)
        .ok_or("failed to parse tokenizer")?;
    let token_bytes = tokenizer.token_byte_lengths();

    let train_len = args.train_tokens.min(train.len());
    let vocab_size = 4096usize;
    // Fit on document-sized chunks so a context never crosses an arbitrary file boundary, and
    // split the stream around any out-of-vocabulary token so no window is rejected whole. The
    // streams carry roughly one such token per million, so this is lossless in practice.
    const CHUNK: usize = 4096;
    let mut chunks: Vec<&[u16]> = Vec::new();
    {
        let mut start = 0usize;
        let mut index = 0usize;
        while index <= train_len {
            let boundary = index == train_len || (train[index] as usize) >= vocab_size;
            if boundary {
                if index > start {
                    chunks.extend(train[start..index].chunks(CHUNK));
                }
                start = index + 1;
            }
            index += 1;
        }
    }
    let config = NgramConfig {
        vocab_size: 4096,
        row_caps: args.row_caps,
        ..NgramConfig::default()
    };
    config.validate()?;
    let (model, fit) = KneserNey5Gram::fit(&chunks, config)?;

    let positions = args.eval_tokens.min(eval.len().saturating_sub(5));
    let mut bits = 0.0f64;
    let mut bytes = 0u64;
    let mut scored = 0u64;
    let mut skipped_oov = 0u64;
    for end in 5..=positions {
        let context = &eval[end - 4..end];
        let next = eval[end];
        // Keep every scored position inside the declared vocabulary, in the context and the
        // target alike; a stream carrying a handful of larger ids is otherwise rejected whole.
        if next as usize >= vocab_size || context.iter().any(|t| *t as usize >= vocab_size) {
            skipped_oov += 1;
            continue;
        }
        let p = model.probability(context, next)?;
        bits += -p.max(1e-12).log2();
        bytes += token_bytes.get(next as usize).copied().unwrap_or(1).max(1) as u64;
        scored += 1;
    }

    let report = serde_json::json!({
        "schema": "uor-r4.kn-ngram-gate/1",
        "status": "COMPLETED",
        "algorithm": uor_r4_training::ngram::ALGORITHM,
        "train": args.train.display().to_string(),
        "eval": args.eval.display().to_string(),
        "train_tokens": train_len,
        "train_windows": chunks.len(),
        "eval_positions": scored,
        "skipped_out_of_vocabulary": skipped_oov,
        "eval_bytes": bytes,
        "row_caps": args.row_caps,
        "bits_per_byte": bits / bytes.max(1) as f64,
        "model_bytes": model.model_bytes(),
        "fit": {
            "orders": fit.orders.iter().map(|o| serde_json::json!({
                "order": o.order, "raw_types": o.raw_types, "retained_types": o.retained_types,
            })).collect::<Vec<_>>(),
        },
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "scope": "Statistical Kneser-Ney comparator on real held-out text with the same token byte accounting the model evaluations use. This is the acceptance gate the native mechanisms must beat, not a native result.",
    });
    let text = serde_json::to_string_pretty(&report)?;
    match &args.out {
        Some(path) => std::fs::write(path, text)?,
        None => println!("{text}"),
    }
    eprintln!(
        "Kneser-Ney 5-gram bits/byte {:.6} over {} positions",
        bits / bytes.max(1) as f64,
        scored
    );
    Ok(())
}
