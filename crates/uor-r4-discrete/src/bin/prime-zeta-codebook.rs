//! Export a qualified PrimeZeta (or matched null) codebook as a pinned integer
//! table for `codebook_initialization: pinned_table_v1` (D39 item 3).
//!
//! Authority: the PrimeZeta construction qualification (D32 October 14
//! addendum #2398; sheet `m1-prime-zeta-complex-routing-2026-10-14.md`, FIT #2433).
//! Host `f64` arithmetic stays inside the separate `prime_zeta` module, up to its
//! integer boundary, exactly as in `prime-zeta-qualify`. This binary writes only
//! the resulting integer table; nothing here trains, scores or runs a model.
//!
//! Usage: prime-zeta-codebook SPEC_JSON SEED OUTPUT [prime_zeta|tangent_null|normal_null:SEED|random_injective:SEED|quota_only_injective:SEED]
//! OUTPUT must not exist; it is created exclusively.

#[path = "../prime_zeta.rs"]
mod prime_zeta;

use prime_zeta::{generate_codebook, writer_cell_occupancy, CodebookMode, TANGENT_NULL_DELTA};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::ExitCode;
use uor_r4_discrete::training::{CodebookTable, CODEBOOK_TABLE_SCHEMA};
use uor_r4_discrete::Spec;

fn mode(arg: Option<&String>) -> Result<CodebookMode, String> {
    match arg.map(String::as_str) {
        None | Some("prime_zeta") => Ok(CodebookMode::PrimeZeta),
        Some("tangent_null") => Ok(CodebookMode::TangentNull {
            delta: TANGENT_NULL_DELTA,
        }),
        Some(other) => {
            if let Some(seed) = other.strip_prefix("normal_null:") {
                Ok(CodebookMode::NormalNull {
                    seed: seed.parse().map_err(|e| format!("normal_null seed: {e}"))?,
                })
            } else if let Some(seed) = other.strip_prefix("random_injective:") {
                Ok(CodebookMode::RandomInjective {
                    seed: seed
                        .parse()
                        .map_err(|e| format!("random_injective seed: {e}"))?,
                })
            } else if let Some(seed) = other.strip_prefix("quota_only_injective:") {
                Ok(CodebookMode::QuotaOnlyInjective {
                    seed: seed
                        .parse()
                        .map_err(|e| format!("quota_only_injective seed: {e}"))?,
                })
            } else {
                Err(format!("unknown mode {other}"))
            }
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    if !(3..=4).contains(&args.len()) {
        return Err("usage: prime-zeta-codebook SPEC_JSON SEED OUTPUT [prime_zeta|tangent_null|normal_null:SEED|random_injective:SEED|quota_only_injective:SEED]".into());
    }
    let spec: Spec = serde_json::from_slice(&fs::read(&args[0]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let seed: u32 = args[1].parse().map_err(|e| format!("seed: {e}"))?;
    let mode = mode(args.get(3))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])
        .map_err(|e| format!("exclusive {}: {e}", args[2]))?;
    let codebook = generate_codebook(spec, seed, mode).map_err(|e| e.to_string())?;
    let occupancy = writer_cell_occupancy(&codebook);
    let min_cell_occupancy = occupancy.iter().copied().min().unwrap_or(0);
    let max_cell_occupancy = occupancy.iter().copied().max().unwrap_or(0);
    let cell_collision_pairs: u64 = occupancy
        .iter()
        .map(|&n| (n as u64) * (n.saturating_sub(1) as u64) / 2)
        .sum();
    // The constructor's own digest and the driver's pinned-table digest agree.
    let words_sha256 = CodebookTable::words_sha256(&codebook.table);
    if words_sha256 != codebook.table_sha256 {
        return Err("constructor table digest differs from the pinned-table digest".into());
    }
    if CodebookTable::count_distinct(&codebook.table, spec.blocks) != codebook.unique_table_vectors
    {
        return Err("constructor distinct-vector count differs from the recount".into());
    }
    let table = CodebookTable {
        schema: CODEBOOK_TABLE_SCHEMA.into(),
        spec,
        mode: codebook.mode.clone(),
        distinct_vectors: CodebookTable::count_distinct(&codebook.table, spec.blocks),
        table: codebook.table,
        table_sha256: words_sha256,
    };
    let bytes = serde_json::to_vec(&table).map_err(|e| e.to_string())?;
    output
        .write_all(&bytes)
        .and_then(|_| output.sync_all())
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "schema": CODEBOOK_TABLE_SCHEMA,
            "mode": table.mode,
            "seed": seed,
            "table_sha256": table.table_sha256,
            "file_sha256": uor_r4_discrete::corpus::sha256(&bytes),
            "unique_table_vectors": codebook.unique_table_vectors,
            "occupied_writer_cells": codebook.occupied_writer_cells,
            "min_cell_occupancy": min_cell_occupancy,
            "max_cell_occupancy": max_cell_occupancy,
            "cell_collision_pairs": cell_collision_pairs,
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

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn cli_mode_parser_accepts_all_five_modes_and_rejects_unknown() {
        assert_eq!(mode(None).unwrap(), CodebookMode::PrimeZeta);
        assert_eq!(
            mode(Some(&"prime_zeta".to_string())).unwrap(),
            CodebookMode::PrimeZeta
        );
        assert_eq!(
            mode(Some(&"tangent_null".to_string())).unwrap(),
            CodebookMode::TangentNull {
                delta: TANGENT_NULL_DELTA
            }
        );
        assert_eq!(
            mode(Some(&"normal_null:2".to_string())).unwrap(),
            CodebookMode::NormalNull { seed: 2 }
        );
        assert_eq!(
            mode(Some(&"random_injective:1".to_string())).unwrap(),
            CodebookMode::RandomInjective { seed: 1 }
        );
        assert_eq!(
            mode(Some(&"quota_only_injective:0".to_string())).unwrap(),
            CodebookMode::QuotaOnlyInjective { seed: 0 }
        );
        assert!(mode(Some(&"unknown_mode".to_string())).is_err());
        assert!(mode(Some(&"random_injective:not_a_u32".to_string())).is_err());
        assert!(mode(Some(&"quota_only_injective:not_a_u32".to_string())).is_err());
    }
}
