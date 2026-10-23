//! Export the `address_structured_v1` (or matched `address_shuffled_v1_seed_S`)
//! codebook as a pinned integer table for `codebook_initialization:
//! pinned_table_v1`. TRAIN only: the DEV split is never loaded. Host data
//! preparation; nothing here trains, scores or runs a model.
//!
//! Usage: address-cells-codebook CONFIG_JSON OUTPUT [structured|shuffled:SEED]
//! OUTPUT must not exist; it is created exclusively.

#[allow(dead_code)]
#[path = "../prime_zeta.rs"]
mod prime_zeta;

use prime_zeta::{generate_codebook, CodebookMode};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::ExitCode;
use uor_r4_discrete::address_cells::{build, table_file_bytes, AddressCellsMode, BigramCounts};
use uor_r4_discrete::corpus::{sha256, PreparedCorpus};
use uor_r4_discrete::training::{CodebookTable, DriverConfig, CODEBOOK_TABLE_SCHEMA};

fn mode(arg: Option<&String>) -> Result<AddressCellsMode, String> {
    match arg.map(String::as_str) {
        None | Some("structured") => Ok(AddressCellsMode::StructuredV1),
        Some("full-vector") => Ok(AddressCellsMode::FullVectorV1),
        Some(other) => {
            if let Some(seed) = other.strip_prefix("shuffled:") {
                return Ok(AddressCellsMode::ShuffledV1 {
                    seed: seed.parse().map_err(|e| format!("shuffled seed: {e}"))?,
                });
            }
            if let Some(seed) = other.strip_prefix("full-vector-shuffled:") {
                return Ok(AddressCellsMode::FullVectorShuffledV1 {
                    seed: seed
                        .parse()
                        .map_err(|e| format!("full-vector-shuffled seed: {e}"))?,
                });
            }
            Err(format!("unknown mode {other}"))
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    if !(2..=3).contains(&args.len()) {
        return Err(
            "usage: address-cells-codebook CONFIG_JSON OUTPUT [structured|shuffled:SEED]".into(),
        );
    }
    let config: DriverConfig =
        serde_json::from_slice(&fs::read(&args[0]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mode = mode(args.get(2))?;
    let spec = config.spec;
    // TRAIN only; FilePin::read verifies the content SHA. DEV is never loaded.
    // The output is created only after the table is built, so a failed build
    // never leaves an empty file that would block an exclusive retry.
    let train = PreparedCorpus::load(&config.train, spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    let counts = BigramCounts::count(train.tokens(), train.eligible_positions(), spec.vocabulary)
        .map_err(|e| e.to_string())?;
    let prime = generate_codebook(spec, 1, CodebookMode::PrimeZeta).map_err(|e| e.to_string())?;
    let blocks = spec.blocks as usize;
    let fallback: Vec<u16> = (0..spec.vocabulary as usize)
        .map(|t| {
            let roots = &prime.root_indices[t * blocks..(t + 1) * blocks];
            u16::from(roots[0]) * 16 + u16::from(roots[blocks - 1])
        })
        .collect();
    let placed = build(spec, &counts, &fallback, mode).map_err(|e| e.to_string())?;
    let bytes = table_file_bytes(spec, &placed).map_err(|e| e.to_string())?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])
        .map_err(|e| format!("exclusive {}: {e}", args[1]))?;
    output
        .write_all(&bytes)
        .and_then(|_| output.sync_all())
        .map_err(|e| e.to_string())?;
    let train_tokens: Vec<u8> = train
        .tokens()
        .iter()
        .flat_map(|t| t.to_le_bytes())
        .collect();
    println!(
        "{}",
        serde_json::json!({
            "schema": CODEBOOK_TABLE_SCHEMA,
            "mode": placed.mode,
            "table_sha256": CodebookTable::words_sha256(&placed.table),
            "file_sha256": sha256(&bytes),
            "distinct_vectors": placed.distinct_vectors,
            "occupied_cells": placed.occupied_cells,
            "counts_sha256": sha256(&counts.canonical_bytes()),
            "train_tokens_sha256": sha256(&train_tokens),
            "model_execution": false,
        })
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
