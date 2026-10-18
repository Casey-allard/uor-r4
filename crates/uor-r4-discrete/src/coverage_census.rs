//! Codebook coverage census (D39 item 2): distinct vectors, alias groups,
//! within-cell tie mass and the DEV alias ceiling for the G-Cap gate.
//!
//! Host-side grouping and counting over stored data only. Token cells come
//! from the GPU `attention-export-token-cells` receipt; this module never
//! reconstructs a cell from geometry (D24). Without that receipt the cell
//! metrics are reported as unavailable, not estimated.

use crate::corpus::{self, FilePin};
use crate::training::{DriverCheckpoint, DriverConfig, PreparedDriver};
use crate::{Error, Result, Spec};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

const CELL_COUNT: usize = 256;
const CELL_RECEIPT_SCHEMA: &str = "uor-r4.attention-token-cells/1";

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}

/// The fields of a token-cell export receipt this census binds to.
#[derive(Debug, Deserialize)]
struct CellReceipt {
    schema: String,
    driver_checkpoint_sha256: String,
    token_cells_file: FilePin,
    token_count: usize,
}

/// Alias group of every token: tokens with an identical full vector share a
/// group, named by its lowest token ID (the evaluation tie-break winner).
pub fn alias_representatives(table: &[i32], spec: Spec) -> Result<Vec<u32>> {
    let width = spec.blocks as usize * 8;
    let vocabulary = spec.vocabulary as usize;
    if width == 0 || vocabulary == 0 {
        return Err(invalid("codebook spec has no blocks or no vocabulary"));
    }
    if table.len() != vocabulary * width {
        return Err(invalid(format!(
            "codebook holds {} words, expected vocabulary {vocabulary} x {width}",
            table.len()
        )));
    }
    let mut first: HashMap<&[i32], u32> = HashMap::new();
    Ok(table
        .chunks_exact(width)
        .enumerate()
        .map(|(token, vector)| *first.entry(vector).or_insert(token as u32))
        .collect())
}

/// Coverage metrics from the codebook, DEV targets and (optionally) exported cells.
pub fn census(
    table: &[i32],
    spec: Spec,
    dev_targets: &[u32],
    cells: Option<&[u16]>,
    bar: Option<u64>,
) -> Result<Value> {
    let representative = alias_representatives(table, spec)?;
    let mut group_size: BTreeMap<u32, u64> = BTreeMap::new();
    for &r in &representative {
        *group_size.entry(r).or_default() += 1;
    }
    let mut size_histogram: BTreeMap<u64, u64> = BTreeMap::new();
    for &n in group_size.values() {
        *size_histogram.entry(n).or_default() += 1;
    }

    let mut distinct_targets: BTreeMap<u32, u64> = BTreeMap::new();
    let mut ceiling_rows = 0u64;
    for &target in dev_targets {
        let r = *representative
            .get(target as usize)
            .ok_or_else(|| invalid(format!("DEV target {target} outside the vocabulary")))?;
        *distinct_targets.entry(target).or_default() += 1;
        if r == target {
            ceiling_rows += 1;
        }
    }
    let reachable_targets = distinct_targets
        .keys()
        .filter(|&&t| representative[t as usize] == t)
        .count();

    let cell_metrics = match cells {
        None => json!({"status": "UNAVAILABLE_NO_GPU_TOKEN_CELL_RECEIPT"}),
        Some(cells) => cell_census(cells, &representative, dev_targets)?,
    };
    let gate = bar.map(|bar| {
        json!({"bar_rows": bar, "alias_ceiling_rows": ceiling_rows,
            "verdict": if ceiling_rows > bar { "ALIAS_CEILING_ABOVE_BAR" } else { "INSTRUMENT_UNREACHABLE" },
            "note": "G-Cap also nets out admission; that term comes from the admission receipt, not this census"})
    });
    Ok(json!({
        "vocabulary": spec.vocabulary,
        "blocks": spec.blocks,
        "distinct_vectors": group_size.len(),
        "alias_group_size_histogram": size_histogram,
        // Exact evaluation ties: identical vectors always share a cell and a
        // distance (table_block returns the row at evaluation), so only the
        // lowest-ID member can win. This is the within-cell tie mass of D39.
        "within_cell_tie_mass": {
            "tokens_in_alias_groups": representative.iter().filter(|r| group_size[r] > 1).count(),
            "dev_rows_targeting_non_representative": dev_targets.len() as u64 - ceiling_rows,
        },
        "dev_rows": dev_targets.len(),
        "dev_distinct_targets": distinct_targets.len(),
        "dev_targets_alias_representative": reachable_targets,
        "dev_alias_ceiling_rows": ceiling_rows,
        "cells": cell_metrics,
        "g_cap": gate,
    }))
}

fn cell_census(cells: &[u16], representative: &[u32], dev_targets: &[u32]) -> Result<Value> {
    if cells.len() != representative.len() {
        return Err(invalid(format!(
            "token-cell map has {} entries for {} tokens",
            cells.len(),
            representative.len()
        )));
    }
    let mut occupancy = [0u64; CELL_COUNT];
    let mut vectors: Vec<Vec<u32>> = vec![Vec::new(); CELL_COUNT];
    for (token, &cell) in cells.iter().enumerate() {
        let cell = cell as usize;
        if cell >= CELL_COUNT {
            return Err(invalid(format!(
                "token {token} has cell {cell} outside 0..255"
            )));
        }
        occupancy[cell] += 1;
        let r = representative[token];
        // An alias group must share one cell: the exporter and the evaluator
        // read the same row. A split group is an instrument fault (D31).
        if cells[r as usize] as usize != cell {
            return Err(invalid(format!(
                "token {token} and its alias representative {r} have different exported cells"
            )));
        }
        if !vectors[cell].contains(&r) {
            vectors[cell].push(r);
        }
    }
    let mut occupancy_histogram: BTreeMap<u64, u64> = BTreeMap::new();
    for &n in occupancy.iter().filter(|&&n| n > 0) {
        *occupancy_histogram.entry(n).or_default() += 1;
    }
    // Crowding, not ties: distinct vectors sharing a cell compete on token
    // distance; they tie only when that distance is also equal.
    let crowded = |cell: usize| vectors[cell].len() > 1;
    let tokens_in_crowded_cells: u64 = (0..CELL_COUNT)
        .filter(|&c| crowded(c))
        .map(|c| occupancy[c])
        .sum();
    let dev_rows_in_crowded_cells = dev_targets
        .iter()
        .filter(|&&t| crowded(cells[t as usize] as usize))
        .count();
    Ok(json!({
        "status": "FROM_GPU_TOKEN_CELL_RECEIPT",
        "occupied_cells": occupancy.iter().filter(|&&n| n > 0).count(),
        "cell_occupancy_histogram": occupancy_histogram,
        "max_cell_occupancy": occupancy.iter().max().copied().unwrap_or(0),
        "cell_collision_pairs": occupancy.iter().map(|&n| n * n.saturating_sub(1) / 2).sum::<u64>(),
        "crowded_cells_multiple_vectors": (0..CELL_COUNT).filter(|&c| crowded(c)).count(),
        "tokens_in_crowded_cells": tokens_in_crowded_cells,
        "dev_rows_in_crowded_cells": dev_rows_in_crowded_cells,
    }))
}

fn token_cells(receipt: &CellReceipt, checkpoint_sha256: &str) -> Result<Vec<u16>> {
    if receipt.schema != CELL_RECEIPT_SCHEMA {
        return Err(invalid(format!(
            "token-cell receipt schema {}",
            receipt.schema
        )));
    }
    if receipt.driver_checkpoint_sha256 != checkpoint_sha256 {
        return Err(invalid("token-cell receipt belongs to another checkpoint"));
    }
    let bytes = receipt.token_cells_file.read()?;
    if bytes.len() != receipt.token_count * 2 {
        return Err(invalid("token-cell file length differs from its receipt"));
    }
    Ok(bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect())
}

/// Validate config, corpus and checkpoint provenance, then run the census.
/// The checkpoint is the native `DriverCheckpoint` JSON (not an authentication
/// wrapper); its lineage must match the config exactly.
pub fn run(
    config: DriverConfig,
    checkpoint_bytes: &[u8],
    cell_receipt: Option<&[u8]>,
    bar: Option<u64>,
) -> Result<Value> {
    let driver = PreparedDriver::load(config)?;
    let checkpoint: DriverCheckpoint =
        serde_json::from_slice(checkpoint_bytes).map_err(|e| invalid(e.to_string()))?;
    driver.validate_checkpoint(&checkpoint)?;
    let checkpoint_sha256 =
        corpus::sha256(&serde_json::to_vec(&checkpoint).map_err(|e| invalid(e.to_string()))?);
    let spec: Spec = serde_json::from_value(checkpoint.model["spec"].clone())
        .map_err(|e| invalid(e.to_string()))?;
    if spec != driver.config().spec {
        return Err(invalid(
            "checkpoint model spec differs from the driver config",
        ));
    }
    let table: Vec<i32> = serde_json::from_value(checkpoint.model["table"].clone())
        .map_err(|e| invalid(e.to_string()))?;
    let cells = cell_receipt
        .map(|bytes| {
            let receipt: CellReceipt =
                serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
            token_cells(&receipt, &checkpoint_sha256)
        })
        .transpose()?;
    let dev_targets = driver.dev().eligible_targets();
    let mut report = census(&table, spec, &dev_targets, cells.as_deref(), bar)?;
    report["schema"] = json!("uor-r4.coverage-census/1");
    report["status"] = json!("COMPLETE");
    report["checkpoint_raw_sha256"] = json!(corpus::sha256(checkpoint_bytes));
    report["driver_checkpoint_sha256"] = json!(checkpoint_sha256);
    report["model_sha256"] = json!(checkpoint.model_sha256);
    report["completed_batches"] = json!(checkpoint.cursor.completed_batches);
    report["accepted_updates"] = json!(checkpoint.cursor.accepted_updates);
    report["host_cell_reconstruction"] = json!(false);
    report["model_execution"] = json!(false);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(vocabulary: u32) -> Spec {
        Spec {
            vocabulary,
            blocks: 2,
            context: 4,
            kept_cells: 8,
        }
    }
    /// Four tokens: 0 and 2 share a vector, 1 and 3 are distinct.
    fn table() -> Vec<i32> {
        let a = [1; 16];
        let b = [2; 16];
        let c = [3; 16];
        [a, b, a, c].concat()
    }

    #[test]
    fn alias_groups_take_the_lowest_token_id() {
        assert_eq!(
            alias_representatives(&table(), spec(4)).unwrap(),
            vec![0, 1, 0, 3]
        );
    }

    #[test]
    fn ceiling_counts_rows_whose_target_is_the_representative() {
        // Rows targeting 2 can never win the tie-break against 0.
        let report = census(&table(), spec(4), &[0, 2, 2, 1, 3], None, Some(3)).unwrap();
        assert_eq!(report["distinct_vectors"], 3);
        assert_eq!(report["dev_alias_ceiling_rows"], 3);
        assert_eq!(report["dev_distinct_targets"], 4);
        assert_eq!(report["dev_targets_alias_representative"], 3);
        assert_eq!(report["alias_group_size_histogram"]["1"], 2);
        assert_eq!(report["alias_group_size_histogram"]["2"], 1);
        assert_eq!(report["g_cap"]["verdict"], "INSTRUMENT_UNREACHABLE");
        assert_eq!(
            report["cells"]["status"],
            "UNAVAILABLE_NO_GPU_TOKEN_CELL_RECEIPT"
        );
    }

    #[test]
    fn cell_ties_count_distinct_vectors_not_tokens() {
        // Cell 5 holds tokens 0 and 2 (one vector): no tie. Cell 9 holds 1 and 3
        // (two vectors): a within-cell tie.
        let cells = [5u16, 9, 5, 9];
        let report = census(&table(), spec(4), &[0, 1, 3, 3], Some(&cells), None).unwrap();
        let c = &report["cells"];
        assert_eq!(c["occupied_cells"], 2);
        assert_eq!(c["crowded_cells_multiple_vectors"], 1);
        assert_eq!(c["tokens_in_crowded_cells"], 2);
        assert_eq!(c["dev_rows_in_crowded_cells"], 3);
        assert_eq!(c["cell_collision_pairs"], 2);
        assert!(report["g_cap"].is_null());
    }

    #[test]
    fn tie_mass_is_the_alias_mass() {
        // Tokens 0 and 2 tie exactly; two rows target the losing member 2.
        let report = census(&table(), spec(4), &[0, 2, 2, 1], None, None).unwrap();
        let t = &report["within_cell_tie_mass"];
        assert_eq!(t["tokens_in_alias_groups"], 2);
        assert_eq!(t["dev_rows_targeting_non_representative"], 2);
    }

    #[test]
    fn a_split_alias_group_is_an_instrument_fault() {
        // Tokens 0 and 2 have identical rows, so the exporter must give one cell.
        let error = census(&table(), spec(4), &[0], Some(&[5, 9, 6, 9]), None).unwrap_err();
        assert!(error.to_string().contains("different exported cells"));
    }

    #[test]
    fn malformed_inputs_are_refused() {
        assert!(alias_representatives(&table()[..10], spec(4)).is_err());
        let mut empty = spec(4);
        empty.blocks = 0;
        assert!(alias_representatives(&[], empty).is_err());
        assert!(census(&table(), spec(4), &[7], None, None).is_err());
        assert!(census(&table(), spec(4), &[0], Some(&[0, 1, 0]), None).is_err());
        assert!(census(&table(), spec(4), &[0], Some(&[0, 1, 0, 300]), None).is_err());
    }
}
