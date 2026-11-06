//! Evaluate centroid and matched-control output tables on a sealed FBS query cache.
//!
//! `cpu-query-cache-evaluate CACHE --train SPLIT --panel SPLIT ... [options]`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{
    evaluate_registered_query_cache, CentroidEvaluationRequest, CpuTrainError,
    REGISTERED_EXPECTED_ROWS, REGISTERED_MINIMUM_OBSERVATIONS, REGISTERED_MIN_FITTED_ROWS,
    REGISTERED_MIN_FIT_OBSERVATIONS, REGISTERED_PANEL_SPLITS, REGISTERED_POOLED_ROW_IDENTITY,
    REGISTERED_TRAIN_MASK_SHA256, REGISTERED_TRAIN_SPLIT, REGISTERED_TRAIN_TOKENS_SHA256,
};

fn main() -> Result<(), CpuTrainError> {
    let mut args = std::env::args().skip(1);
    let cache = PathBuf::from(args.next().ok_or_else(usage)?);
    let mut request = CentroidEvaluationRequest {
        seed: None,
        train_split: REGISTERED_TRAIN_SPLIT.to_owned(),
        panel_splits: Vec::new(),
        memory: false,
        minimum_observations: REGISTERED_MINIMUM_OBSERVATIONS,
        minimum_fitted_rows: REGISTERED_MIN_FITTED_ROWS,
        minimum_fit_observations: REGISTERED_MIN_FIT_OBSERVATIONS,
        expected_train_tokens_sha256: Some(REGISTERED_TRAIN_TOKENS_SHA256.to_owned()),
        expected_train_mask_sha256: Some(REGISTERED_TRAIN_MASK_SHA256.to_owned()),
        expected_rows: Some(REGISTERED_EXPECTED_ROWS),
        expected_row_identity: Some(REGISTERED_POOLED_ROW_IDENTITY.to_owned()),
        checkpoint_root: None,
    };
    let mut out = None;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| CpuTrainError::Invalid(format!("missing value for {flag}")))?;
        match flag.as_str() {
            "--seed" => {
                request.seed = Some(
                    value
                        .parse::<u8>()
                        .map_err(|_| CpuTrainError::Invalid("--seed wants 1 or 2".to_owned()))?,
                )
            }
            "--train" => request.train_split = value,
            "--panel" => request.panel_splits.push(value),
            "--memory" => {
                request.memory = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err(CpuTrainError::Invalid("--memory wants 0 or 1".to_owned())),
                }
            }
            "--minimum" => request.minimum_observations = number(&flag, &value)?,
            "--min-fitted-rows" => request.minimum_fitted_rows = number(&flag, &value)?,
            "--min-fit-observations" => request.minimum_fit_observations = number(&flag, &value)?,
            "--expect-train-tokens" => request.expected_train_tokens_sha256 = Some(value),
            "--expect-train-mask" => request.expected_train_mask_sha256 = Some(value),
            "--expect-rows" => request.expected_rows = Some(number(&flag, &value)?),
            "--expect-identity" => request.expected_row_identity = Some(value),
            "--checkpoint-root" => request.checkpoint_root = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(CpuTrainError::Invalid(format!("unknown flag {flag}"))),
        }
    }
    if request.panel_splits.is_empty() {
        request.panel_splits = REGISTERED_PANEL_SPLITS
            .into_iter()
            .map(str::to_owned)
            .collect();
    }
    let report = evaluate_registered_query_cache(&cache, &request)?;
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
    match out {
        Some(path) => write_new(&path, &bytes),
        None => {
            std::io::stdout()
                .write_all(&bytes)
                .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
            std::io::stdout()
                .write_all(b"\n")
                .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
            Ok(())
        }
    }
}

fn number(flag: &str, value: &str) -> Result<usize, CpuTrainError> {
    value
        .parse()
        .map_err(|_| CpuTrainError::Invalid(format!("{flag} wants a positive integer")))
        .and_then(|value| {
            if value == 0 {
                Err(CpuTrainError::Invalid(format!(
                    "{flag} wants a positive integer"
                )))
            } else {
                Ok(value)
            }
        })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CpuTrainError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| CpuTrainError::Invalid(format!("{}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| CpuTrainError::Invalid(error.to_string()))?;
    Ok(())
}

fn usage() -> CpuTrainError {
    CpuTrainError::Invalid(
        "usage: cpu-query-cache-evaluate CACHE --seed 1|2 [--checkpoint-root DIR] [--out FILE]"
            .to_owned(),
    )
}
