//! Verify an FBS teacher-forced query cache and evaluate centroid output rows on CPU.
//!
//! `fbs-query-cache evaluate CACHE [--train SPLIT] [--panel SPLIT ...]
//!  [--memory 0|1] [--minimum N] [--out FILE]`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_float_train::query_cache::evaluate_centroid_cache;
use uor_r4_float_train::{FloatTrainError, Result};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("evaluate") {
        return Err(FloatTrainError::Invalid(
            "usage: fbs-query-cache evaluate CACHE [--train SPLIT] [--panel SPLIT ...] [--memory 0|1] [--minimum N] [--out FILE]".to_owned(),
        ));
    }
    let cache = PathBuf::from(args.next().ok_or_else(|| {
        FloatTrainError::Invalid("fbs-query-cache evaluate needs CACHE DIR".to_owned())
    })?);
    let mut train = "train".to_owned();
    let mut panels = Vec::new();
    let mut memory = true;
    let mut minimum = 8usize;
    let mut out = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| FloatTrainError::Invalid(format!("missing {flag} value")))?;
        match flag.as_str() {
            "--train" => train = value,
            "--panel" => panels.push(value),
            "--memory" => {
                memory = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err(FloatTrainError::Invalid("--memory wants 0 or 1".to_owned())),
                }
            }
            "--minimum" => {
                minimum = value.parse::<usize>().map_err(|_| {
                    FloatTrainError::Invalid("--minimum wants a whole number".to_owned())
                })?
            }
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(FloatTrainError::Invalid(format!("unknown flag {flag}"))),
        }
    }
    if panels.is_empty() {
        return Err(FloatTrainError::Invalid(
            "fbs-query-cache evaluate needs at least one --panel SPLIT".to_owned(),
        ));
    }
    let report = evaluate_centroid_cache(&cache, &train, &panels, memory, minimum)?;
    let bytes = serde_json::to_vec_pretty(&report)?;
    match out {
        Some(path) => write_new(&path, &bytes),
        None => {
            std::io::stdout().write_all(&bytes)?;
            std::io::stdout().write_all(b"\n")?;
            Ok(())
        }
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| FloatTrainError::Invalid(format!("{}: {error}", path.display())))?;
    file.write_all(bytes)?;
    Ok(())
}
