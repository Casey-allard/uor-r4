//! Aggregate the two seed reports from `cpu-query-cache-evaluate`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::{aggregate_query_cache_reports, CpuTrainError};

fn main() -> Result<(), CpuTrainError> {
    let mut args = std::env::args().skip(1);
    let first = PathBuf::from(args.next().ok_or_else(usage)?);
    let second = PathBuf::from(args.next().ok_or_else(usage)?);
    let mut out = None;
    if let Some(flag) = args.next() {
        if flag != "--out" {
            return Err(CpuTrainError::Invalid(format!("unknown argument {flag}")));
        }
        out = Some(PathBuf::from(args.next().ok_or_else(usage)?));
    }
    if args.next().is_some() {
        return Err(usage());
    }
    let reports =
        [&first, &second]
            .into_iter()
            .map(|path| {
                serde_json::from_slice(&fs::read(path).map_err(|error| {
                    CpuTrainError::Invalid(format!("{}: {error}", path.display()))
                })?)
                .map_err(|error| CpuTrainError::Invalid(format!("{}: {error}", path.display())))
            })
            .collect::<Result<Vec<serde_json::Value>, CpuTrainError>>()?;
    let report = aggregate_query_cache_reports(&reports)?;
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
        "usage: cpu-query-cache-aggregate SEED1.json SEED2.json [--out FILE]".to_owned(),
    )
}
