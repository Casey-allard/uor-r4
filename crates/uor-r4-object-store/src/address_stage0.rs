#![cfg(feature = "stage0-lab")]
//! Deterministic Stage 0 copy-probe manifest and exact-emission protocol.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

pub const STAGE0_SCHEMA: &str = "uor-r4.address-stage0-copy/1";
pub const STAGE0_SEED: &str = "m1-address-stage0-v1";

/// Stage 0's pinned inputs (sheet
/// `docs/harness/prep/m1-address-as-read-stage0-2026-10-26.md`).
pub const ARTIFACT_SHA256: &str =
    "cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd";
/// The sheet's original checkpoint file pin. The receipt-based CLI no longer
/// reads the checkpoint; identity records it as registered lineage only.
pub const CHECKPOINT_SHA256: &str =
    "0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4";
pub const TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
pub const MANIFEST_SHA256: &str =
    "ea36011870e3f6d0add87e9fc1d25115ffc77f2a9206d13bfe304c6785351fa5";
/// The sealed Stage T GPU receipt that replaces the external evaluator.
pub const RETAINED_RECEIPT_SHA256: &str =
    "a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e";
pub const RETAINED_RECEIPT_SCHEMA: &str = "uor-r4.discrete-corpus-driver/1";
/// The outer Stage T archive that retains the sealed receipt.
pub const STAGE_T_ARCHIVE_SHA256: &str =
    "c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6";
pub const DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
pub const DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
pub const TRAIN_TOKENS_SHA256: &str =
    "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8";
pub const TRAIN_MASK_SHA256: &str =
    "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3";
/// The original Stage T config retained in the repository, the staged input
/// after its absolute paths were rewritten locally, and the normalization
/// applied on the pod. The actual normalized file's SHA is recorded at runtime.
pub const ORIGINAL_CONFIG_SHA256: &str =
    "77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d";
pub const STAGED_INPUT_CONFIG_SHA256: &str =
    "43a316bc67bb45fe5b50bd2ceee57f6b919164c02fe45ee9e677f8819fea5d15";
pub const CONFIG_PATH_NORMALIZATION_NOTE: &str = "The retained Stage T config was rewritten locally with sed into the staged input config; a second sed path correction on the pod produced the actual normalized config. original_config_sha256, staged_input_config_sha256 and normalized_config_sha256 distinguish those three files.";
pub const DEV_ROWS: usize = 14_287;
/// Prompt tokens must leave room for at least one generated teacher token.
pub const PROMPT_TOKEN_LIMIT: usize = 192;
pub const RETAINED_TOP1: u64 = 455;
pub const RETAINED_TOP8: u64 = 1_484;
/// The registered strict non-vacuity floor: `d > 24 && d*d > 4*(b + c)`.
pub const PRIMARY_FLOOR: u64 = 24;
pub const STAGE0_HEAD_ENV: &str = "UOR_STAGE0_HEAD";
pub const IDENTITY_SCHEMA: &str = "uor-r4.address-stage0-identity/1";
pub const PARITY_SCHEMA: &str = "uor-r4.address-stage0-parity/1";
pub const PARITY_ROW_SCHEMA: &str = "uor-r4.address-stage0-parity-row/1";
pub const COPY_SCHEMA: &str = "uor-r4.address-stage0-copy-report/1";
pub const COPY_ROW_SCHEMA: &str = "uor-r4.address-stage0-copy-row/1";
/// `uor-r4-discrete::metadata::ABSENT`; pads a short top-8 for comparison.
pub const ABSENT_TOKEN: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyManifest {
    pub schema: String,
    pub seed: String,
    pub rows: Vec<CopyRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CopyRow {
    pub id: String,
    pub value: String,
    pub random_hint: String,
    pub value_tokens: Vec<u32>,
    pub random_tokens: Vec<u32>,
    pub prompt: String,
    pub random_prompt: String,
    pub no_hint_prompt: String,
    pub prompt_tokens: Vec<u32>,
    pub random_prompt_tokens: Vec<u32>,
    pub no_hint_prompt_tokens: Vec<u32>,
    pub prompt_sha256: String,
    pub random_prompt_sha256: String,
    pub no_hint_prompt_sha256: String,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The public hash used by the runner's identity records.
pub fn sha256_hex(bytes: &[u8]) -> String {
    sha(bytes)
}

/// Parses the pinned copy manifest and runs the existing validator. The
/// caller still owns the pin check (`MANIFEST_SHA256`), because a manifest's
/// digest is evidence, not an authorization signature.
pub fn parse_manifest(bytes: &[u8]) -> Result<CopyManifest, String> {
    let manifest: CopyManifest =
        serde_json::from_slice(bytes).map_err(|e| format!("copy manifest: {e}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn contains_sequence(haystack: &[u32], needle: &[u32]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Row ids whose exact value token sequence appears in `corpus`.
pub fn corpus_value_leaks(manifest: &CopyManifest, corpus: &[u32]) -> Vec<String> {
    manifest
        .rows
        .iter()
        .filter(|row| contains_sequence(corpus, &row.value_tokens))
        .map(|row| row.id.clone())
        .collect()
}

/// The two discordant counts of one paired comparison: `b` is a treatment-only
/// win, `c` a control-only win.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairedWins {
    pub b: u64,
    pub c: u64,
}

impl PairedWins {
    pub fn d(&self) -> i64 {
        self.b as i64 - self.c as i64
    }
}

/// The registered floor `b - c > max(24, 2*sqrt(b + c))`, decided exactly in
/// integers as `d > 24 && d*d > 4*(b + c)`. Equality never clears.
pub fn clears_primary(b: u64, c: u64) -> bool {
    if b <= c {
        return false;
    }
    let d = u128::from(b - c);
    d > u128::from(PRIMARY_FLOOR) && d * d > 4 * (u128::from(b) + u128::from(c))
}

/// Both primary comparisons must clear; teacher-forced top-8 is not a
/// substitute for either.
pub fn primary_pass(exact_vs_no_hint: PairedWins, exact_vs_random_hint: PairedWins) -> bool {
    clears_primary(exact_vs_no_hint.b, exact_vs_no_hint.c)
        && clears_primary(exact_vs_random_hint.b, exact_vs_random_hint.c)
}

/// Populates the three parity aggregate fields once the D11 aggregate exists.
/// The runner calls this before its per-row mismatch and top-1/top-8 stops, so
/// those reports cannot lose the aggregates. Returns the admission delta.
pub fn set_parity_aggregates(
    report: &mut serde_json::Value,
    d11: (u64, u64, u64),
    original: (u64, u64, u64),
    original_rows: u64,
) -> i64 {
    let (d11_top1, d11_top8, d11_admitted) = d11;
    let (original_top1, original_top8, original_admitted) = original;
    let d11_rows = report
        .get("rows")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let admission_delta = d11_admitted as i64 - original_admitted as i64;
    report["d11_aggregate"] = serde_json::json!({
        "rows": d11_rows,
        "top1": d11_top1,
        "top8": d11_top8,
        "target_admitted": d11_admitted,
    });
    report["original_aggregate"] = serde_json::json!({
        "rows": original_rows,
        "top1": original_top1,
        "top8": original_top8,
        "target_admitted": original_admitted,
    });
    report["position_strip"] = serde_json::json!({
        "d11_top1": d11_top1,
        "retained_top1": RETAINED_TOP1,
        "top1_delta": d11_top1 as i64 - RETAINED_TOP1 as i64,
        "d11_top8": d11_top8,
        "retained_top8": RETAINED_TOP8,
        "top8_delta": d11_top8 as i64 - RETAINED_TOP8 as i64,
        "d11_target_admitted": d11_admitted,
        "original_target_admitted": original_admitted,
        "admission_delta": admission_delta,
        "admission_aggregate_matches": admission_delta == 0,
        "note": "Aggregates only; no row-level baseline is assumed. A position-strip difference is recorded before copy results are interpreted.",
    });
    admission_delta
}

/// Writes the stop fields of a parity report. The runner's `stop_parity` and
/// the mismatch regression test use this one transition.
pub fn mark_parity_stop(report: &mut serde_json::Value, reason: &str) {
    report["pass"] = serde_json::json!(false);
    report["instrument_stop"] = serde_json::json!(reason);
}

/// A planted wrong answer holds only when at least one row is applicable and
/// every applicable random-hint emission differs from the exact value.
pub fn planted_wrong_holds(
    manifest: &CopyManifest,
    random_emissions: &[Option<String>],
) -> Result<bool, String> {
    if random_emissions.len() != manifest.rows.len() {
        return Err(format!(
            "{} random emissions for {} manifest rows",
            random_emissions.len(),
            manifest.rows.len()
        ));
    }
    let mut applicable = 0usize;
    for (row, emission) in manifest.rows.iter().zip(random_emissions) {
        if let Some(answer) = emission {
            applicable += 1;
            if answer == &row.value {
                return Ok(false);
            }
        }
    }
    Ok(applicable > 0)
}

/// One sealed Stage T evaluator row object, collected from the retained
/// `final-dev.json` receipt.
/// `admitted_cells` is that report's scalar count; `target_cell_rank` is the
/// target cell's rank among the admitted cells (1-based). The D11 scorer's row
/// record carries the cell IDs and is compared by its count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginalRow {
    pub target_position: u64,
    pub target_token: u32,
    pub top8: [u32; 8],
    pub top1_correct: u64,
    pub admitted_cells: u64,
    pub target_cell_rank: u64,
}

/// Recursively collects every object carrying all five original-report row
/// keys, in deterministic traversal order (arrays in order, then object keys).
/// Row objects are terminal; their contents are not traversed again.
pub fn collect_original_rows(report: &serde_json::Value) -> Result<Vec<OriginalRow>, String> {
    let mut rows = Vec::new();
    collect_rows_into(report, &mut rows)?;
    Ok(rows)
}

fn collect_rows_into(value: &serde_json::Value, rows: &mut Vec<OriginalRow>) -> Result<(), String> {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_rows_into(item, rows)?;
            }
        }
        serde_json::Value::Object(map) => {
            let keys = [
                "target_position",
                "target_token",
                "top8",
                "top1_correct",
                "admitted_cells",
            ];
            if keys.iter().all(|key| map.contains_key(*key)) {
                rows.push(parse_original_row(map)?);
            } else {
                for item in map.values() {
                    collect_rows_into(item, rows)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_original_row(
    map: &serde_json::Map<String, serde_json::Value>,
) -> Result<OriginalRow, String> {
    let target_position = map["target_position"]
        .as_u64()
        .ok_or("original report row has no target_position")?;
    let target_token = u32::try_from(
        map["target_token"]
            .as_u64()
            .ok_or("original report row has no target_token")?,
    )
    .map_err(|_| "original report row target_token exceeds u32".to_string())?;
    let top8 = map["top8"]
        .as_array()
        .ok_or("original report row top8 is not an array")?;
    if top8.len() != 8 {
        return Err(format!(
            "original report row has {} top8 entries, not 8",
            top8.len()
        ));
    }
    let mut values = [0u32; 8];
    for (slot, value) in values.iter_mut().zip(top8) {
        *slot = u32::try_from(
            value
                .as_u64()
                .ok_or("original report row top8 entry is not a u64")?,
        )
        .map_err(|_| "original report row top8 entry exceeds u32".to_string())?;
    }
    let top1_correct = map["top1_correct"]
        .as_u64()
        .ok_or("original report row has no top1_correct")?;
    if top1_correct > 1 {
        return Err("original report row top1_correct is not 0 or 1".into());
    }
    let admitted_cells = map["admitted_cells"]
        .as_u64()
        .ok_or("original report row has no admitted_cells")?;
    let target_cell_rank = map["target_cell_rank"]
        .as_u64()
        .ok_or("original report row has no target_cell_rank")?;
    Ok(OriginalRow {
        target_position,
        target_token,
        top8: values,
        top1_correct,
        admitted_cells,
        target_cell_rank,
    })
}

/// ABSENT-pads a short scorer top-8 to the receipt's fixed eight slots.
pub fn padded_top8(top8: &[u32]) -> Result<[u32; 8], String> {
    if top8.len() > 8 {
        return Err(format!("a scorer row has {} top-8 entries", top8.len()));
    }
    let mut padded = [ABSENT_TOKEN; 8];
    padded[..top8.len()].copy_from_slice(top8);
    Ok(padded)
}

/// One D11 parity row, independent of the GPU scorer types so the comparison
/// can be tested on the host. `top8` is already ABSENT-padded to eight slots;
/// `admitted_cells` carries the actual cell IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D11ParityRow {
    pub ordinal: u64,
    pub target_position: u64,
    pub target_token: u32,
    pub status: i32,
    pub prediction: u32,
    pub top8: [u32; 8],
    pub admitted_cells: Vec<u32>,
}

impl D11ParityRow {
    pub fn admitted_count(&self) -> u64 {
        self.admitted_cells.len() as u64
    }
}

/// Compact per-row parity records. The schema and `admitted_count` are part
/// of the recorded claim; `ordinal` is the D11 row ordinal.
pub fn parity_row_lines(rows: &[D11ParityRow]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    for row in rows {
        let line = serde_json::json!({
            "schema": PARITY_ROW_SCHEMA,
            "ordinal": row.ordinal,
            "target_position": row.target_position,
            "target_token": row.target_token,
            "status": row.status,
            "prediction": row.prediction,
            "top8": row.top8,
            "admitted_count": row.admitted_count(),
            "admitted_cells": row.admitted_cells,
        });
        out.extend(
            serde_json::to_vec(&line).map_err(|e| format!("parity row {}: {e}", row.ordinal))?,
        );
        out.push(b'\n');
    }
    Ok(out)
}

/// D11 top-1/top-8/target-admission counts. Nonzero-status rows are refused by
/// the runner but are skipped here so the aggregate does not hide them.
pub fn d11_aggregate_rows(
    rows: &[D11ParityRow],
    target_cells: &[u32],
) -> Result<(u64, u64, u64), String> {
    let (mut top1, mut top8, mut admitted) = (0u64, 0u64, 0u64);
    for row in rows {
        if row.status != 0 {
            continue;
        }
        if row.prediction == row.target_token {
            top1 += 1;
        }
        if row.top8.contains(&row.target_token) {
            top8 += 1;
        }
        let cell = *target_cells
            .get(row.target_token as usize)
            .ok_or_else(|| format!("target {} is outside the artifact cells", row.target_token))?;
        if row.admitted_cells.contains(&cell) {
            admitted += 1;
        }
    }
    Ok((top1, top8, admitted))
}

/// Compares D11 parity rows to the retained receipt rows. The join is by
/// traversal position only; the receipt has no ordinal field, so the public
/// checks do not claim one.
pub fn compare_parity_rows(
    d11: &[D11ParityRow],
    original: &[OriginalRow],
) -> Result<serde_json::Value, String> {
    if d11.len() != original.len() {
        return Err(format!(
            "{} D11 rows, {} retained receipt rows",
            d11.len(),
            original.len()
        ));
    }
    let mut mismatched = 0u64;
    let mut first = Vec::new();
    for (row_index, (d11, original)) in d11.iter().zip(original).enumerate() {
        let mut reasons: Vec<&'static str> = Vec::new();
        if original.target_position != d11.target_position {
            reasons.push("target_position");
        }
        if original.target_token != d11.target_token {
            reasons.push("target_token");
        }
        if original.top8[0] != d11.prediction {
            reasons.push("prediction");
        }
        if original.top8 != d11.top8 {
            reasons.push("top8");
        }
        if original.admitted_cells != d11.admitted_count() {
            reasons.push("admitted_count");
        }
        if original.top1_correct != u64::from(d11.prediction == d11.target_token) {
            reasons.push("top1_correct");
        }
        if !reasons.is_empty() {
            mismatched += 1;
            if first.len() < 20 {
                first.push(serde_json::json!({
                    "row_index": row_index,
                    "reasons": reasons,
                    "original": {
                        "target_position": original.target_position,
                        "target_token": original.target_token,
                        "top8": original.top8,
                        "top1_correct": original.top1_correct,
                        "admitted_cells": original.admitted_cells,
                        "target_cell_rank": original.target_cell_rank,
                    },
                    "d11": {
                        "target_position": d11.target_position,
                        "target_token": d11.target_token,
                        "status": d11.status,
                        "prediction": d11.prediction,
                        "top8": d11.top8,
                        "admitted_count": d11.admitted_count(),
                        "admitted_cells": d11.admitted_cells,
                    },
                }));
            }
        }
    }
    Ok(serde_json::json!({
        "rows_compared": d11.len(),
        "mismatched_rows": mismatched,
        "first_mismatches": first,
        "row_order_join": "D11 rows are joined to retained-receipt rows by traversal position; the receipt has no ordinal field.",
        "checks": ["target_position", "target_token", "prediction", "top8", "admitted_count", "top1_correct"],
    }))
}

/// The retained receipt's aggregate target-admitted count.
pub fn original_target_admitted(report: &serde_json::Value) -> Result<u64, String> {
    if let Some(value) = report
        .get("aggregate")
        .and_then(|aggregate| aggregate.get("target_admitted_rows"))
        .and_then(serde_json::Value::as_u64)
    {
        return Ok(value);
    }
    let mut total = 0u64;
    let mut seen = false;
    if let Some(batches) = report.get("batches").and_then(serde_json::Value::as_array) {
        for batch in batches {
            if let Some(value) = batch
                .get("aggregate")
                .and_then(|aggregate| aggregate.get("target_admitted_rows"))
                .and_then(serde_json::Value::as_u64)
            {
                total = total
                    .checked_add(value)
                    .ok_or("the original admission sum overflows")?;
                seen = true;
            }
        }
    }
    if seen {
        Ok(total)
    } else {
        Err("the original report has no target_admitted_rows aggregate".into())
    }
}

/// Retained-receipt top-1/top-8/admission aggregate. Admission is recomputed
/// from each row's `target_cell_rank <= admitted_cells` and cross-checked
/// against the receipt's aggregate field, not merely copied from it.
pub fn original_aggregate(
    report: &serde_json::Value,
    rows: &[OriginalRow],
) -> Result<(u64, u64, u64), String> {
    let top1 = rows.iter().filter(|row| row.top1_correct == 1).count() as u64;
    let top8 = rows
        .iter()
        .filter(|row| row.top8.contains(&row.target_token))
        .count() as u64;
    let admitted_rows = rows
        .iter()
        .filter(|row| row.target_cell_rank <= row.admitted_cells)
        .count() as u64;
    let aggregate_admitted = original_target_admitted(report)?;
    if aggregate_admitted != admitted_rows {
        return Err(format!(
            "the receipt's target_admitted_rows {aggregate_admitted} differs from per-row target_cell_rank <= admitted_cells ({admitted_rows})"
        ));
    }
    if let Some(aggregate) = report.get("aggregate") {
        if let Some(value) = aggregate
            .get("scored_rows")
            .and_then(serde_json::Value::as_u64)
        {
            if value != rows.len() as u64 {
                return Err(format!(
                    "the original report's scored_rows {value} differs from its {} rows",
                    rows.len()
                ));
            }
        }
        let check = |name: &str, computed: u64| -> Result<(), String> {
            if let Some(value) = aggregate.get(name).and_then(serde_json::Value::as_u64) {
                if value != computed {
                    return Err(format!(
                        "the original report's {name} {value} differs from its rows' {computed}"
                    ));
                }
            }
            Ok(())
        };
        check("top1_correct", top1)?;
        check("top8_correct", top8)?;
        check("target_admitted_rows", admitted_rows)?;
    }
    Ok((top1, top8, admitted_rows))
}

/// Decodes a `.u16` token file without model or corpus assembly. The canonical
/// corpus files carry a 64-byte `UORT` header; a headerless file is accepted
/// byte-for-byte, so the leak check never needs the corpus machinery.
pub fn u16_le_tokens(bytes: &[u8]) -> Result<Vec<u32>, String> {
    let payload = if bytes.starts_with(b"UORT") {
        if bytes.len() < 64 {
            return Err("UORT header is short".into());
        }
        let version = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|_| "UORT version field is short")?,
        );
        if version != 1 {
            return Err(format!("UORT version {version} is not 1"));
        }
        let count = u64::from_le_bytes(
            bytes[8..16]
                .try_into()
                .map_err(|_| "UORT count field is short")?,
        );
        let physical = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(64))
            .ok_or("UORT count overflows")?;
        if physical != bytes.len() as u64 {
            return Err(format!(
                "UORT count {count} disagrees with {} bytes",
                bytes.len()
            ));
        }
        &bytes[64..]
    } else {
        bytes
    };
    if payload.len() % 2 != 0 {
        return Err("token bytes are not a whole number of u16 values".into());
    }
    Ok(payload
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect())
}

/// The run's HEAD is recorded from the harness environment and must be a full
/// 40-lowercase-hex commit; a short or abbreviated HEAD is refused.
pub fn validate_head(value: &str) -> Result<&str, String> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(value)
    } else {
        Err("UOR_STAGE0_HEAD must be the full 40-lowercase-hex Stage 0 commit".into())
    }
}

/// Reads `git rev-parse HEAD` from the source checkout.
pub fn git_head(checkout: &std::path::Path) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("git rev-parse HEAD in {}: {e}", checkout.display()))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse HEAD failed in {}: {}",
            checkout.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let head = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    validate_head(&head)?;
    Ok(head)
}

/// Requires `UOR_STAGE0_HEAD` to equal the source checkout's actual HEAD, not
/// merely to have the right shape.
pub fn validate_head_against_git(value: &str, checkout: &std::path::Path) -> Result<(), String> {
    validate_head(value)?;
    let head = git_head(checkout)?;
    if head != value {
        return Err(format!(
            "UOR_STAGE0_HEAD {value} differs from git rev-parse HEAD {head}"
        ));
    }
    Ok(())
}

/// Runs a fallible device initializer before creating OUT_DIR, so a failed
/// device cannot leave an empty directory that makes a retry refuse.
pub fn create_out_dir_after<T>(
    out_dir: &std::path::Path,
    init: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let value = init()?;
    std::fs::create_dir(out_dir)
        .map_err(|e| format!("create {}: {e} (OUT_DIR must not exist)", out_dir.display()))?;
    Ok(value)
}

/// Validates the five registered corpus/tokenizer config pins and the DEV row
/// count before any corpus file is read or scanned.
pub fn validate_config_pins(config: &serde_json::Value) -> Result<(), String> {
    let pin = |pointer: &str, expected: &str, name: &str| -> Result<(), String> {
        let actual = config
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("config lacks {pointer}"))?;
        if actual != expected {
            return Err(format!("config {name} pin {actual} is not {expected}"));
        }
        Ok(())
    };
    pin("/dev/tokens/sha256", DEV_TOKENS_SHA256, "DEV tokens")?;
    pin("/dev/mask/sha256", DEV_MASK_SHA256, "DEV mask")?;
    pin("/train/tokens/sha256", TRAIN_TOKENS_SHA256, "TRAIN tokens")?;
    pin("/train/mask/sha256", TRAIN_MASK_SHA256, "TRAIN mask")?;
    pin("/tokenizer/sha256", TOKENIZER_SHA256, "tokenizer")?;
    let eligible = config
        .pointer("/dev/expected_eligible_targets")
        .and_then(serde_json::Value::as_u64);
    if eligible != Some(DEV_ROWS as u64) {
        return Err(format!(
            "config DEV declares {eligible:?} eligible targets, not {DEV_ROWS}"
        ));
    }
    Ok(())
}

/// The `manifest` CLI refuses a non-pinned tokenizer and a generated manifest
/// that is not the registered artifact, before creating the output file.
pub fn validate_manifest_cli_pins(
    tokenizer_sha256: &str,
    manifest_sha256: &str,
) -> Result<(), String> {
    if tokenizer_sha256 != TOKENIZER_SHA256 {
        return Err(format!(
            "manifest tokenizer {tokenizer_sha256} is not {TOKENIZER_SHA256}"
        ));
    }
    if manifest_sha256 != MANIFEST_SHA256 {
        return Err(format!(
            "generated manifest {manifest_sha256} is not {MANIFEST_SHA256}"
        ));
    }
    Ok(())
}

fn value(seed: &str) -> String {
    Sha256::digest(seed.as_bytes())
        .iter()
        .take(8)
        .map(|byte| char::from(b'a' + byte % 26))
        .collect()
}

fn spaced(value: &str) -> String {
    value
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn prompt(value: &str) -> String {
    format!(
        "supplied value is {}.\nQuestion: what is the supplied value?\nAnswer: ",
        spaced(value)
    )
}

fn query() -> &'static str {
    "Question: what is the supplied value?\nAnswer: "
}

pub fn build_manifest(tokenizer: &ByteBpeTokenizer) -> Result<CopyManifest, String> {
    let mut rows = Vec::new();
    for index in 0..40 {
        let exact_value = value(&format!("{STAGE0_SEED}|value-{index}"));
        let exact_tokens = tokenizer.encode(&exact_value);
        let exact_prompt = prompt(&exact_value);
        let exact_prompt_tokens = tokenizer.encode(&exact_prompt);
        let mut random_hint = String::new();
        let mut random_tokens = Vec::new();
        let mut matched = false;
        for candidate in 0..1024 {
            let hint = value(&format!("{STAGE0_SEED}|random-{index}-{candidate}"));
            let tokens = tokenizer.encode(&hint);
            let candidate_prompt_tokens = tokenizer.encode(&prompt(&hint));
            if hint != exact_value
                && tokens.len() == exact_tokens.len()
                && candidate_prompt_tokens.len() == exact_prompt_tokens.len()
            {
                random_hint = hint;
                random_tokens = tokens;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(format!(
                "no token-length-matched random hint at row {index}"
            ));
        }
        let random_prompt = prompt(&random_hint);
        let no_hint_prompt = query().to_owned();
        let row = CopyRow {
            id: format!("copy-{index:03}"),
            value_tokens: exact_tokens,
            random_tokens,
            prompt_tokens: tokenizer.encode(&exact_prompt),
            random_prompt_tokens: tokenizer.encode(&random_prompt),
            no_hint_prompt_tokens: tokenizer.encode(&no_hint_prompt),
            prompt_sha256: sha(exact_prompt.as_bytes()),
            random_prompt_sha256: sha(random_prompt.as_bytes()),
            no_hint_prompt_sha256: sha(no_hint_prompt.as_bytes()),
            value: exact_value,
            random_hint,
            prompt: exact_prompt,
            random_prompt,
            no_hint_prompt,
        };
        if row.prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
            || row.random_prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
            || row.no_hint_prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
        {
            return Err(format!(
                "prompt must be shorter than {PROMPT_TOKEN_LIMIT} tokens at row {index}"
            ));
        }
        for (arm, prompt_tokens) in [
            ("exact", &row.prompt_tokens),
            ("random", &row.random_prompt_tokens),
            ("no_hint", &row.no_hint_prompt_tokens),
        ] {
            if prompt_tokens.len() + row.value_tokens.len() > PROMPT_TOKEN_LIMIT {
                return Err(format!(
                    "prompt+value exceeds {PROMPT_TOKEN_LIMIT} tokens at row {index} ({arm})"
                ));
            }
        }
        if row.value_tokens.len() != row.random_tokens.len() {
            return Err(format!("hint token lengths differ at row {index}"));
        }
        if row.prompt_tokens.len() != row.random_prompt_tokens.len() {
            return Err(format!("prompt token lengths differ at row {index}"));
        }
        rows.push(row);
    }
    let manifest = CopyManifest {
        schema: STAGE0_SCHEMA.to_owned(),
        seed: STAGE0_SEED.to_owned(),
        rows,
    };
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &CopyManifest) -> Result<(), String> {
    if manifest.schema != STAGE0_SCHEMA || manifest.seed != STAGE0_SEED {
        return Err("manifest schema or seed differs".into());
    }
    if manifest.rows.len() != 40 {
        return Err("copy probe needs exactly 40 rows".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut values = std::collections::BTreeSet::new();
    for row in &manifest.rows {
        if !ids.insert(&row.id) || !values.insert(&row.value) {
            return Err("duplicate row id or value".into());
        }
        if row.value.len() != 8
            || row.random_hint.len() != 8
            || !row.value.bytes().all(|b| b.is_ascii_lowercase())
            || !row.random_hint.bytes().all(|b| b.is_ascii_lowercase())
            || row.value == row.random_hint
        {
            return Err(format!("invalid value pair at {}", row.id));
        }
        if row.value_tokens.len() != row.random_tokens.len()
            || row.prompt_tokens.len() != row.random_prompt_tokens.len()
            || row.prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
            || row.random_prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
            || row.no_hint_prompt_tokens.len() >= PROMPT_TOKEN_LIMIT
        {
            return Err(format!("token length mismatch at {}", row.id));
        }
        for (arm, prompt_tokens) in [
            ("exact", &row.prompt_tokens),
            ("random", &row.random_prompt_tokens),
            ("no_hint", &row.no_hint_prompt_tokens),
        ] {
            if prompt_tokens.len() + row.value_tokens.len() > PROMPT_TOKEN_LIMIT {
                return Err(format!(
                    "prompt+value exceeds {PROMPT_TOKEN_LIMIT} tokens at {} ({arm})",
                    row.id
                ));
            }
        }
        if row.prompt != prompt(&row.value)
            || row.random_prompt != prompt(&row.random_hint)
            || row.no_hint_prompt != query()
            || row.prompt_sha256 != sha(row.prompt.as_bytes())
            || row.random_prompt_sha256 != sha(row.random_prompt.as_bytes())
            || row.no_hint_prompt_sha256 != sha(row.no_hint_prompt.as_bytes())
        {
            return Err(format!("prompt or prompt hash mismatch at {}", row.id));
        }
        if row.no_hint_prompt.contains(&row.value) || row.random_prompt.contains(&row.value) {
            return Err(format!("answer leak at {}", row.id));
        }
    }
    Ok(())
}

pub fn values_missing_from_corpus(manifest: &CopyManifest, corpus: &[u32]) -> bool {
    manifest
        .rows
        .iter()
        .all(|row| !contains_sequence(corpus, &row.value_tokens))
}

pub fn normalize_emission(text: &str) -> Result<String, String> {
    let mut letters = text.chars().filter(|c| c.is_ascii_lowercase());
    let answer: String = letters.by_ref().take(8).collect();
    if answer.len() != 8 {
        return Err("emission has fewer than eight lowercase letters".into());
    }
    if letters.next().is_some() {
        return Err("emission has more than eight lowercase letters".into());
    }
    if text
        .chars()
        .any(|c| !(c.is_ascii_lowercase() || c.is_ascii_whitespace()))
    {
        return Err("emission contains punctuation or a non-ASCII character".into());
    }
    Ok(answer)
}

/// The Stage 0 GPU runner (`cuda` feature). It links the D11 scorer instrument
/// and `uor-r4-discrete` only behind this feature; the default host build and
/// the `manifest` command never compile the scorer.
#[cfg(feature = "cuda")]
pub mod runner {
    use super::{
        clears_primary, collect_original_rows, corpus_value_leaks, create_out_dir_after,
        normalize_emission, original_aggregate, padded_top8, planted_wrong_holds, primary_pass,
        sha256_hex, u16_le_tokens, validate_config_pins, validate_head_against_git, CopyManifest,
        CopyRow, D11ParityRow, PairedWins, ABSENT_TOKEN, ARTIFACT_SHA256, CHECKPOINT_SHA256,
        CONFIG_PATH_NORMALIZATION_NOTE, COPY_ROW_SCHEMA, COPY_SCHEMA, DEV_MASK_SHA256, DEV_ROWS,
        DEV_TOKENS_SHA256, IDENTITY_SCHEMA, MANIFEST_SHA256, ORIGINAL_CONFIG_SHA256, PARITY_SCHEMA,
        PROMPT_TOKEN_LIMIT, RETAINED_RECEIPT_SCHEMA, RETAINED_RECEIPT_SHA256, RETAINED_TOP1,
        RETAINED_TOP8, STAGE0_HEAD_ENV, STAGED_INPUT_CONFIG_SHA256, STAGE_T_ARCHIVE_SHA256,
        TOKENIZER_SHA256,
    };
    use m1_d11_artifact_scorer_instrument as inst;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use uor_r4_discrete::training::DriverConfig;
    use uor_r4_discrete::ReadSpec;
    use uor_r4_tokenizer::ByteBpeTokenizer;

    /// The exact inputs of the registered Stage 0 command.
    #[derive(Debug, Clone)]
    pub struct RunPaths {
        pub artifact: PathBuf,
        pub config: PathBuf,
        pub retained_receipt: PathBuf,
        pub tokenizer: PathBuf,
        pub manifest: PathBuf,
        pub out_dir: PathBuf,
    }

    #[derive(Debug, Clone)]
    pub struct Stage0Outcome {
        pub parity_pass: bool,
        pub copy_primary_pass: bool,
        pub decision: String,
        pub out_dir: PathBuf,
    }

    #[derive(Debug, Clone)]
    struct CopyOutcome {
        primary_pass: bool,
        decision: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Arm {
        Exact,
        NoHint,
        RandomHint,
    }

    impl Arm {
        const ALL: [Arm; 3] = [Arm::Exact, Arm::NoHint, Arm::RandomHint];

        fn name(self) -> &'static str {
            match self {
                Arm::Exact => "exact",
                Arm::NoHint => "no_hint",
                Arm::RandomHint => "random_hint",
            }
        }

        fn prompt<'a>(self, row: &'a CopyRow) -> &'a [u32] {
            match self {
                Arm::Exact => &row.prompt_tokens,
                Arm::NoHint => &row.no_hint_prompt_tokens,
                Arm::RandomHint => &row.random_prompt_tokens,
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    struct TeacherCounts {
        rows: u64,
        tokens: u64,
        token_top1: u64,
        token_top8: u64,
        rows_all_top1: u64,
        rows_all_top8: u64,
    }

    #[derive(Debug, Clone)]
    struct FreeRow {
        normalized: Option<String>,
        exact: bool,
    }

    /// Validated Stage 0 run. It writes `identity.json`, then either a parity
    /// stop report or both parity row records and `parity-report.json`, then
    /// the copy records and report. A failed primary copy gate is a completed
    /// measurement (`COPY_GATE_FAIL`), not an instrument error.
    pub fn run(
        paths: &RunPaths,
        cuda_driver_api_version: Result<i32, String>,
    ) -> Result<Stage0Outcome, String> {
        run_internal(paths, cuda_driver_api_version, true)
    }

    /// Run only the pinned artifact parity/determinism checks, leaving the
    /// output directory available for a composed instrument such as the oracle.
    pub fn run_parity(
        paths: &RunPaths,
        cuda_driver_api_version: Result<i32, String>,
    ) -> Result<Stage0Outcome, String> {
        run_internal(paths, cuda_driver_api_version, false)
    }

    fn run_internal(
        paths: &RunPaths,
        cuda_driver_api_version: Result<i32, String>,
        run_copy: bool,
    ) -> Result<Stage0Outcome, String> {
        let head = std::env::var(STAGE0_HEAD_ENV)
            .map_err(|_| format!("{STAGE0_HEAD_ENV} must be the full Stage 0 commit"))?;
        validate_head_against_git(&head, Path::new(env!("CARGO_MANIFEST_DIR")))?;
        if paths.out_dir.exists() {
            return Err(format!(
                "{} exists; Stage 0 outputs are never overwritten",
                paths.out_dir.display()
            ));
        }

        // Pinned bytes are checked before any GPU context exists.
        let artifact_bytes = read_pinned(&paths.artifact, ARTIFACT_SHA256, "artifact")?;
        let receipt_bytes = read_pinned(
            &paths.retained_receipt,
            RETAINED_RECEIPT_SHA256,
            "retained final-dev receipt",
        )?;
        let tokenizer_bytes = read_pinned(&paths.tokenizer, TOKENIZER_SHA256, "tokenizer")?;
        let manifest_bytes = read_pinned(&paths.manifest, MANIFEST_SHA256, "manifest")?;
        let config_bytes = read_file(&paths.config, "config")?;
        let config_sha256 = sha256_hex(&config_bytes);

        let receipt: Value = serde_json::from_slice(&receipt_bytes)
            .map_err(|e| format!("retained receipt {}: {e}", paths.retained_receipt.display()))?;
        if receipt.get("schema").and_then(Value::as_str) != Some(RETAINED_RECEIPT_SCHEMA) {
            return Err(format!(
                "retained receipt schema is not {RETAINED_RECEIPT_SCHEMA}"
            ));
        }
        let receipt_epoch = receipt
            .get("model_epoch")
            .and_then(Value::as_u64)
            .ok_or("retained receipt lacks model_epoch")?;
        let receipt_accepted = receipt
            .get("accepted_updates_in_model")
            .and_then(Value::as_u64)
            .ok_or("retained receipt lacks accepted_updates_in_model")?;
        if receipt_epoch != receipt_accepted {
            return Err(format!(
                "retained receipt model_epoch {receipt_epoch} differs from accepted_updates_in_model {receipt_accepted}"
            ));
        }
        let receipt_model_storage = receipt
            .get("model_storage_sha256")
            .and_then(Value::as_str)
            .ok_or("retained receipt lacks model_storage_sha256")?;
        let receipt_lineage = receipt
            .get("evaluation_lineage_sha256")
            .and_then(Value::as_str)
            .ok_or("retained receipt lacks evaluation_lineage_sha256")?;
        let original_rows =
            collect_original_rows(&receipt).map_err(|e| format!("retained receipt rows: {e}"))?;
        if original_rows.len() != DEV_ROWS {
            return Err(format!(
                "retained receipt has {} rows, not {DEV_ROWS}",
                original_rows.len()
            ));
        }
        let (original_top1, original_top8, original_admitted) =
            original_aggregate(&receipt, &original_rows)
                .map_err(|e| format!("retained receipt aggregate: {e}"))?;
        let original_descriptor = json!({
            "kind": "sealed_stage_t_final_dev_receipt",
            "path": paths.retained_receipt.display().to_string(),
            "sha256": RETAINED_RECEIPT_SHA256,
            "stage_t_archive_sha256": STAGE_T_ARCHIVE_SHA256,
            "schema": RETAINED_RECEIPT_SCHEMA,
            "model_epoch": receipt_epoch,
            "accepted_updates_in_model": receipt_accepted,
            "model_storage_sha256": receipt_model_storage,
            "evaluation_lineage_sha256": receipt_lineage,
            "rows": original_rows.len(),
        });

        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or_else(|| "the pinned tokenizer JSON is not a ByteBpeTokenizer".to_string())?;
        let manifest = super::parse_manifest(&manifest_bytes)?;
        for row in &manifest.rows {
            if row.value_tokens.is_empty()
                || row.prompt_tokens.is_empty()
                || row.random_prompt_tokens.is_empty()
                || row.no_hint_prompt_tokens.is_empty()
            {
                return Err(format!(
                    "manifest row {} has an empty token sequence; the copy probe would be vacuous",
                    row.id
                ));
            }
        }
        let config_json: Value = serde_json::from_slice(&config_bytes)
            .map_err(|e| format!("config {}: {e}", paths.config.display()))?;
        validate_config_pins(&config_json)?;
        let config: DriverConfig = serde_json::from_slice(&config_bytes)
            .map_err(|e| format!("config {}: {e}", paths.config.display()))?;
        config.validate().map_err(|e| format!("config: {e}"))?;

        let (tables, model) = inst::artifact::ArtifactTables::from_bytes(&artifact_bytes)?;
        if tables.artifact_sha256 != ARTIFACT_SHA256 {
            return Err(format!(
                "the integer reader's artifact hash {} is not the pin",
                tables.artifact_sha256
            ));
        }
        let shape = model.shape();
        if shape.vocabulary != config.spec.vocabulary
            || shape.blocks != config.spec.blocks
            || shape.context != config.spec.context
            || shape.kept_cells != config.spec.kept_cells
        {
            return Err(format!(
                "artifact shape {shape:?} differs from config spec {:?}",
                config.spec
            ));
        }
        let read_kept = u32::try_from(tables.shape[6]).unwrap_or(u32::MAX);
        let read_sweeps = u32::try_from(tables.shape[7]).unwrap_or(u32::MAX);
        let read_matches = match &config.read_spec {
            ReadSpec::Route => tables.shape[5] == inst::artifact::READ_ROUTE,
            ReadSpec::Flock { kept } => {
                tables.shape[5] == inst::artifact::READ_FLOCK && *kept == read_kept
            }
            ReadSpec::Relation { kept, sweeps } => {
                tables.shape[5] == inst::artifact::READ_RELATION
                    && *kept == read_kept
                    && *sweeps == read_sweeps
            }
        };
        if !read_matches {
            return Err("artifact read mode differs from the config's read_spec".into());
        }
        let context = shape.context as usize;

        let rows = inst::dev_rows(&config).map_err(|e| format!("DEV rows: {e}"))?;
        if rows.len() != DEV_ROWS {
            return Err(format!("DEV rows are {}, not {DEV_ROWS}", rows.len()));
        }

        // The leak check is direct u16 little-endian file work; no model runs.
        let train_bytes = config
            .train
            .tokens
            .read()
            .map_err(|e| format!("TRAIN tokens: {e}"))?;
        config
            .train
            .mask
            .read()
            .map_err(|e| format!("TRAIN mask: {e}"))?;
        let dev_bytes = config
            .dev
            .tokens
            .read()
            .map_err(|e| format!("DEV tokens: {e}"))?;
        config
            .dev
            .mask
            .read()
            .map_err(|e| format!("DEV mask: {e}"))?;
        let train_tokens = u16_le_tokens(&train_bytes).map_err(|e| format!("TRAIN tokens: {e}"))?;
        let dev_tokens = u16_le_tokens(&dev_bytes).map_err(|e| format!("DEV tokens: {e}"))?;
        let train_leaks = corpus_value_leaks(&manifest, &train_tokens);
        let dev_leaks = corpus_value_leaks(&manifest, &dev_tokens);
        if !train_leaks.is_empty() || !dev_leaks.is_empty() {
            return Err(format!(
                "exact value tokens leak into the corpus: TRAIN {train_leaks:?}, DEV {dev_leaks:?}"
            ));
        }

        let scorer = create_out_dir_after(&paths.out_dir, || {
            inst::device::DeviceScorer::new(0, &tables).map_err(|e| format!("device 0: {e}"))
        })?;
        let executable = std::env::current_exe().map_err(|e| format!("current exe: {e}"))?;
        let identity = json!({
            "schema": IDENTITY_SCHEMA,
            "HEAD": head.as_str(),
            "executable_sha256": file_sha256(&executable)?,
            "executable_path": executable.display().to_string(),
            "retained_receipt_path": paths.retained_receipt.display().to_string(),
            "retained_receipt_sha256": RETAINED_RECEIPT_SHA256,
            "stage_t_archive_sha256": STAGE_T_ARCHIVE_SHA256,
            "retained_receipt_schema": RETAINED_RECEIPT_SCHEMA,
            "retained_receipt_model_epoch": receipt_epoch,
            "retained_receipt_accepted_updates_in_model": receipt_accepted,
            "retained_receipt_model_storage_sha256": receipt_model_storage,
            "retained_receipt_evaluation_lineage_sha256": receipt_lineage,
            "retained_receipt_rows": original_rows.len(),
            "cubin_sha256": inst::device::cubin_sha256(),
            "ptx_sha256": inst::device::ptx_sha256(),
            "IMAGE_TARGET": inst::device::IMAGE_TARGET.trim(),
            "RESOURCES": inst::device::RESOURCES.trim(),
            "device_capability": [scorer.capability.0, scorer.capability.1],
            "cuda_driver_api_version": match cuda_driver_api_version {
                Ok(raw) => json!({
                    "raw": raw,
                    "major": raw / 1000,
                    "minor": (raw % 1000) / 10,
                    "source": "cudarc::driver::sys::cuDriverGetVersion",
                }),
                Err(error) => json!({
                    "available": false,
                    "error": error,
                    "source": "cudarc::driver::sys::cuDriverGetVersion",
                }),
            },
            "cuda_toolkit_version": nvcc_identity(),
            "compiled_identity": uor_r4_discrete::device::compiled_identity(),
            "driver_source_sha256": uor_r4_discrete::training::driver_source_sha256(),
            "artifact_sha256": ARTIFACT_SHA256,
            "checkpoint_sha256": CHECKPOINT_SHA256,
            "checkpoint_file_input": false,
            "tokenizer_sha256": TOKENIZER_SHA256,
            "manifest_sha256": MANIFEST_SHA256,
            "original_config_sha256": ORIGINAL_CONFIG_SHA256,
            "staged_input_config_sha256": STAGED_INPUT_CONFIG_SHA256,
            "normalized_config_sha256": config_sha256,
            "config_path_normalization": CONFIG_PATH_NORMALIZATION_NOTE,
            "dev_tokens_sha256": DEV_TOKENS_SHA256,
            "dev_mask_sha256": DEV_MASK_SHA256,
            "dev_rows": rows.len(),
            "UOR_NVCC_ARCH": std::env::var("UOR_NVCC_ARCH").ok(),
        });
        write_json(&paths.out_dir.join("identity.json"), &identity)?;

        let mut parity = json!({
            "schema": PARITY_SCHEMA,
            "HEAD": head.as_str(),
            "artifact_sha256": ARTIFACT_SHA256,
            "checkpoint_sha256": CHECKPOINT_SHA256,
            "checkpoint_file_input": false,
            "tokenizer_sha256": TOKENIZER_SHA256,
            "manifest_sha256": MANIFEST_SHA256,
            "original_config_sha256": ORIGINAL_CONFIG_SHA256,
            "staged_input_config_sha256": STAGED_INPUT_CONFIG_SHA256,
            "normalized_config_sha256": config_sha256,
            "config_path_normalization": CONFIG_PATH_NORMALIZATION_NOTE,
            "retained_receipt_path": paths.retained_receipt.display().to_string(),
            "retained_receipt_sha256": RETAINED_RECEIPT_SHA256,
            "stage_t_archive_sha256": STAGE_T_ARCHIVE_SHA256,
            "rows_expected": DEV_ROWS,
            "rows": rows.len(),
            "all_status_zero": null,
            "runs_byte_identical": null,
            "run1_sha256": null,
            "run2_sha256": null,
            "original": original_descriptor,
            "comparison": null,
            "d11_aggregate": null,
            "original_aggregate": null,
            "retained": {"top1": RETAINED_TOP1, "top8": RETAINED_TOP8},
            "position_strip": null,
            "pass": false,
            "instrument_stop": null,
        });

        let views: Vec<&[u32]> = rows.iter().map(|row| row.prefix.as_slice()).collect();
        let packed = inst::kernel::pack_rows(&views).map_err(|e| format!("pack DEV rows: {e}"))?;
        let run1 = match scorer.score(&packed) {
            Ok(scores) => scores,
            Err(e) => {
                return stop_parity(
                    &mut parity,
                    &paths.out_dir,
                    format!("D11 parity run 1: {e}"),
                )
            }
        };
        if run1.len() != rows.len() {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!(
                    "D11 parity run 1 scored {} of {} rows",
                    run1.len(),
                    rows.len()
                ),
            );
        }
        let d11_run1 = match parity_rows_from_scores(&rows, &run1) {
            Ok(rows) => rows,
            Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
        };
        let bytes1 = match super::parity_row_lines(&d11_run1) {
            Ok(bytes) => bytes,
            Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
        };
        write_new(&paths.out_dir.join("parity-rows-run1.jsonl"), &bytes1)?;

        let run2 = match scorer.score(&packed) {
            Ok(scores) => scores,
            Err(e) => {
                return stop_parity(
                    &mut parity,
                    &paths.out_dir,
                    format!("D11 parity run 2: {e}"),
                )
            }
        };
        if run2.len() != rows.len() {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!(
                    "D11 parity run 2 scored {} of {} rows",
                    run2.len(),
                    rows.len()
                ),
            );
        }
        let d11_run2 = match parity_rows_from_scores(&rows, &run2) {
            Ok(rows) => rows,
            Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
        };
        let bytes2 = match super::parity_row_lines(&d11_run2) {
            Ok(bytes) => bytes,
            Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
        };
        write_new(&paths.out_dir.join("parity-rows-run2.jsonl"), &bytes2)?;

        let all_status_zero = d11_run1
            .iter()
            .chain(&d11_run2)
            .all(|score| score.status == 0);
        let runs_byte_identical = bytes1 == bytes2;
        parity["run1_sha256"] = json!(sha256_hex(&bytes1));
        parity["run2_sha256"] = json!(sha256_hex(&bytes2));
        parity["all_status_zero"] = json!(all_status_zero);
        parity["runs_byte_identical"] = json!(runs_byte_identical);

        let (d11_top1, d11_top8, d11_admitted) =
            match super::d11_aggregate_rows(&d11_run1, &tables.cells) {
                Ok(values) => values,
                Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
            };
        // Every parity stop after the two runs, including nonzero status and
        // nondeterminism, must carry the aggregates.
        let admission_delta = super::set_parity_aggregates(
            &mut parity,
            (d11_top1, d11_top8, d11_admitted),
            (original_top1, original_top8, original_admitted),
            original_rows.len() as u64,
        );

        if !all_status_zero {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                "a D11 parity row has nonzero status".into(),
            );
        }
        if !runs_byte_identical {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                "the two D11 parity runs differ byte-for-byte".into(),
            );
        }

        if original_rows.len() != rows.len() {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!(
                    "the retained receipt has {} rows; the D11 scored {}",
                    original_rows.len(),
                    rows.len()
                ),
            );
        }
        let comparison = match super::compare_parity_rows(&d11_run1, &original_rows) {
            Ok(value) => value,
            Err(e) => return stop_parity(&mut parity, &paths.out_dir, e),
        };
        let mismatched = comparison["mismatched_rows"].as_u64().unwrap_or(u64::MAX);
        parity["comparison"] = comparison;
        if mismatched != 0 {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!("{mismatched} per-row parity mismatches"),
            );
        }
        if d11_top1 != original_top1 || d11_top8 != original_top8 {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!(
                    "aggregate parity disagrees: D11 {d11_top1}/{d11_top8}, original {original_top1}/{original_top8}"
                ),
            );
        }
        if admission_delta != 0 {
            return stop_parity(
                &mut parity,
                &paths.out_dir,
                format!(
                    "admission aggregate differs: D11 {d11_admitted}, original {original_admitted}"
                ),
            );
        }
        parity["pass"] = json!(true);
        write_json(&paths.out_dir.join("parity-report.json"), &parity)?;

        if run_copy {
            let copy = copy_probe(&scorer, &tokenizer, &manifest, context, &paths.out_dir)?;
            Ok(Stage0Outcome {
                parity_pass: true,
                copy_primary_pass: copy.primary_pass,
                decision: copy.decision,
                out_dir: paths.out_dir.clone(),
            })
        } else {
            Ok(Stage0Outcome {
                parity_pass: true,
                copy_primary_pass: false,
                decision: "PARITY_PASS".into(),
                out_dir: paths.out_dir.clone(),
            })
        }
    }

    fn read_file(path: &Path, label: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|e| format!("{label} {}: {e}", path.display()))
    }

    fn read_pinned(path: &Path, expected: &str, label: &str) -> Result<Vec<u8>, String> {
        let bytes = read_file(path, label)?;
        let actual = sha256_hex(&bytes);
        if actual != expected {
            return Err(format!(
                "{label} {} sha256 {actual} differs from pinned {expected}",
                path.display()
            ));
        }
        Ok(bytes)
    }

    fn file_sha256(path: &Path) -> Result<String, String> {
        Ok(sha256_hex(&read_file(path, "file")?))
    }

    fn json_bytes(value: &Value) -> Result<Vec<u8>, String> {
        let mut text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
        text.push('\n');
        Ok(text.into_bytes())
    }

    fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| format!("{}: {e} (outputs are never overwritten)", path.display()))?;
        file.write_all(bytes)
            .map_err(|e| format!("{}: {e}", path.display()))
    }

    fn write_json(path: &Path, value: &Value) -> Result<(), String> {
        write_new(path, &json_bytes(value)?)
    }

    fn stop_parity<T>(report: &mut Value, out_dir: &Path, reason: String) -> Result<T, String> {
        super::mark_parity_stop(report, &reason);
        write_json(&out_dir.join("parity-report.json"), report)?;
        Err(reason)
    }

    fn parity_rows_from_scores(
        rows: &[inst::DevRow],
        scores: &[inst::kernel::RowScore],
    ) -> Result<Vec<D11ParityRow>, String> {
        if rows.len() != scores.len() {
            return Err(format!(
                "{} DEV rows but {} D11 scores",
                rows.len(),
                scores.len()
            ));
        }
        rows.iter()
            .zip(scores)
            .map(|(row, score)| {
                let top8 = if score.status == 0 {
                    super::padded_top8(&score.top8)?
                } else {
                    [ABSENT_TOKEN; 8]
                };
                Ok(D11ParityRow {
                    ordinal: row.ordinal,
                    target_position: row.target_position,
                    target_token: row.target_token,
                    status: score.status,
                    prediction: score.prediction,
                    top8,
                    admitted_cells: score.admitted_cells.clone(),
                })
            })
            .collect()
    }

    fn truncate(text: &str) -> String {
        const LIMIT: usize = 16_384;
        if text.chars().count() <= LIMIT {
            text.to_owned()
        } else {
            let mut out: String = text.chars().take(LIMIT).collect();
            out.push_str("...[truncated]");
            out
        }
    }

    fn nvcc_identity() -> Value {
        let path = std::env::var("NVCC").unwrap_or_else(|_| "nvcc".to_owned());
        match Command::new(&path).arg("--version").output() {
            Ok(output) if output.status.success() => json!({
                "path": path,
                "available": true,
                "output": truncate(&String::from_utf8_lossy(&output.stdout)),
            }),
            Ok(output) => json!({
                "path": path,
                "available": false,
                "output": truncate(&String::from_utf8_lossy(&output.stdout)),
                "error": truncate(&String::from_utf8_lossy(&output.stderr)),
            }),
            Err(e) => json!({
                "path": path,
                "available": false,
                "error": e.to_string(),
            }),
        }
    }

    fn teacher_counts(
        scorer: &inst::device::DeviceScorer,
        manifest: &CopyManifest,
        arm: Arm,
        context: usize,
        lines: &mut Vec<u8>,
    ) -> Result<TeacherCounts, String> {
        let mut prefixes: Vec<Vec<u32>> = Vec::new();
        let mut meta: Vec<(usize, usize, u32)> = Vec::new();
        for (row_index, row) in manifest.rows.iter().enumerate() {
            let base = arm.prompt(row);
            if base.is_empty() || base.len() > context {
                return Err(format!(
                    "{} row {} prompt length {} is outside 1..={context}",
                    arm.name(),
                    row.id,
                    base.len()
                ));
            }
            for (target_index, target) in row.value_tokens.iter().enumerate() {
                let length = base.len() + target_index;
                if length > context {
                    return Err(format!(
                        "{} row {} teacher prefix length {length} exceeds context {context}",
                        arm.name(),
                        row.id
                    ));
                }
                let mut prefix = Vec::with_capacity(length);
                prefix.extend_from_slice(base);
                prefix.extend_from_slice(&row.value_tokens[..target_index]);
                prefixes.push(prefix);
                meta.push((row_index, target_index, *target));
            }
        }
        let views: Vec<&[u32]> = prefixes.iter().map(Vec::as_slice).collect();
        let packed = inst::kernel::pack_rows(&views)?;
        let scores = scorer.score(&packed)?;
        if scores.len() != meta.len() {
            return Err(format!(
                "teacher-forced {} scored {} of {} prefixes",
                arm.name(),
                scores.len(),
                meta.len()
            ));
        }
        let mut counts = TeacherCounts {
            rows: manifest.rows.len() as u64,
            ..TeacherCounts::default()
        };
        let mut all_top1 = vec![true; manifest.rows.len()];
        let mut all_top8 = vec![true; manifest.rows.len()];
        for (score, (row_index, target_index, target)) in scores.iter().zip(&meta) {
            let (top1, top8_correct, top8, status) = if score.status == 0 {
                (
                    score.prediction == *target,
                    score.top8.contains(target),
                    padded_top8(&score.top8)?,
                    score.status,
                )
            } else {
                (false, false, [ABSENT_TOKEN; 8], score.status)
            };
            let line = json!({
                "schema": COPY_ROW_SCHEMA,
                "row": manifest.rows[*row_index].id,
                "arm": arm.name(),
                "kind": "teacher",
                "target_index": target_index,
                "target_token": target,
                "status": status,
                "prediction": score.prediction,
                "top1_correct": top1,
                "top8_correct": top8_correct,
                "top8": top8,
            });
            lines.extend(serde_json::to_vec(&line).map_err(|e| format!("teacher row: {e}"))?);
            lines.push(b'\n');
            counts.tokens += 1;
            counts.token_top1 += u64::from(top1);
            counts.token_top8 += u64::from(top8_correct);
            all_top1[*row_index] &= top1;
            all_top8[*row_index] &= top8_correct;
            if status != 0 {
                return Err(format!(
                    "teacher-forced {} {} target {} status {status}",
                    arm.name(),
                    manifest.rows[*row_index].id,
                    target_index
                ));
            }
        }
        counts.rows_all_top1 = all_top1.iter().filter(|value| **value).count() as u64;
        counts.rows_all_top8 = all_top8.iter().filter(|value| **value).count() as u64;
        Ok(counts)
    }

    fn free_counts(
        scorer: &inst::device::DeviceScorer,
        tokenizer: &ByteBpeTokenizer,
        manifest: &CopyManifest,
        arm: Arm,
        context: usize,
        lines: &mut Vec<u8>,
    ) -> Result<Vec<FreeRow>, String> {
        let limit = PROMPT_TOKEN_LIMIT.min(context);
        if limit == 0 {
            return Err("the artifact context is zero".into());
        }
        let mut out = Vec::new();
        for row in &manifest.rows {
            let mut prefix = arm.prompt(row).to_vec();
            if prefix.is_empty() || prefix.len() >= PROMPT_TOKEN_LIMIT || prefix.len() > context {
                return Err(format!(
                    "{} row {} prompt length {} is outside 1..{PROMPT_TOKEN_LIMIT}",
                    arm.name(),
                    row.id,
                    prefix.len()
                ));
            }
            let mut generated: Vec<u32> = Vec::new();
            let mut status = 0;
            loop {
                let bytes = tokenizer.decode_bytes(&generated);
                let letters = bytes
                    .iter()
                    .filter(|byte| byte.is_ascii_lowercase())
                    .count();
                if letters >= 8 || prefix.len() >= limit {
                    break;
                }
                let packed = inst::kernel::pack_rows(&[prefix.as_slice()])?;
                let scores = scorer.score(&packed)?;
                if scores.len() != 1 {
                    return Err(format!(
                        "free-running {} {} scored {} rows, not 1",
                        arm.name(),
                        row.id,
                        scores.len()
                    ));
                }
                let score = &scores[0];
                if score.status != 0 {
                    status = score.status;
                    break;
                }
                generated.push(score.prediction);
                prefix.push(score.prediction);
            }
            let bytes = tokenizer.decode_bytes(&generated);
            let letters = bytes
                .iter()
                .filter(|byte| byte.is_ascii_lowercase())
                .count();
            let emission_hex = bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let emission = String::from_utf8(bytes).ok();
            let normalized = emission
                .as_deref()
                .and_then(|text| normalize_emission(text).ok());
            let exact = normalized.as_deref() == Some(row.value.as_str());
            let record = FreeRow {
                normalized: normalized.clone(),
                exact,
            };
            let line = json!({
                "schema": COPY_ROW_SCHEMA,
                "row": row.id,
                "arm": arm.name(),
                "kind": "free",
                "status": status,
                "steps": generated.len(),
                "generated_tokens": generated,
                "lowercase_letters": letters,
                "emission": emission,
                "emission_hex": emission_hex,
                "normalized": normalized,
                "exact": exact,
            });
            lines.extend(serde_json::to_vec(&line).map_err(|e| format!("free row: {e}"))?);
            lines.push(b'\n');
            out.push(record);
            if status != 0 {
                return Err(format!(
                    "free-running {} {} status {status}",
                    arm.name(),
                    row.id
                ));
            }
        }
        Ok(out)
    }

    fn arm_report(counts: &TeacherCounts, free: &[FreeRow]) -> Value {
        json!({
            "rows": counts.rows,
            "teacher": {
                "tokens": counts.tokens,
                "token_top1": counts.token_top1,
                "token_top8": counts.token_top8,
                "rows_all_top1": counts.rows_all_top1,
                "rows_all_top8": counts.rows_all_top8,
            },
            "free": {
                "rows": free.len(),
                "normalized": free.iter().filter(|row| row.normalized.is_some()).count(),
                "exact": free.iter().filter(|row| row.exact).count(),
            },
        })
    }

    fn stop_copy(
        report: Value,
        lines: &[u8],
        out_dir: &Path,
        reason: String,
    ) -> Result<CopyOutcome, String> {
        let mut report = report;
        report["instrument_stop"] = json!(reason);
        report["decision"] = json!("STOP");
        write_new(&out_dir.join("copy-rows.jsonl"), lines)?;
        write_json(&out_dir.join("copy-report.json"), &report)?;
        Err(reason)
    }

    fn copy_probe(
        scorer: &inst::device::DeviceScorer,
        tokenizer: &ByteBpeTokenizer,
        manifest: &CopyManifest,
        context: usize,
        out_dir: &Path,
    ) -> Result<CopyOutcome, String> {
        let mut lines = Vec::new();
        let mut report = json!({
            "schema": COPY_SCHEMA,
            "manifest_sha256": MANIFEST_SHA256,
            "rows": manifest.rows.len(),
            "arms": {},
            "primary": null,
            "planted_wrong": null,
            "teacher_note": "Teacher-forced top-8 is descriptive oracle assistance, not generation capability.",
            "instrument_stop": null,
            "decision": "INCOMPLETE",
        });
        let mut results: Vec<(Arm, TeacherCounts, Vec<FreeRow>)> = Vec::new();
        for arm in Arm::ALL {
            let counts = match teacher_counts(scorer, manifest, arm, context, &mut lines) {
                Ok(counts) => counts,
                Err(e) => return stop_copy(report, &lines, out_dir, e),
            };
            let free = match free_counts(scorer, tokenizer, manifest, arm, context, &mut lines) {
                Ok(free) => free,
                Err(e) => return stop_copy(report, &lines, out_dir, e),
            };
            report["arms"][arm.name()] = arm_report(&counts, &free);
            results.push((arm, counts, free));
        }

        if results.len() != Arm::ALL.len() {
            return stop_copy(
                report,
                &lines,
                out_dir,
                "the copy probe did not complete all three arms".into(),
            );
        }
        let exact = results
            .iter()
            .find(|(arm, _, _)| *arm == Arm::Exact)
            .ok_or_else(|| "the exact copy arm is missing".to_string())?;
        let no_hint = results
            .iter()
            .find(|(arm, _, _)| *arm == Arm::NoHint)
            .ok_or_else(|| "the no-hint copy arm is missing".to_string())?;
        let random_hint = results
            .iter()
            .find(|(arm, _, _)| *arm == Arm::RandomHint)
            .ok_or_else(|| "the random-hint copy arm is missing".to_string())?;
        for (arm, _, free) in &results {
            if free.len() != manifest.rows.len() {
                return stop_copy(
                    report,
                    &lines,
                    out_dir,
                    format!(
                        "{} arm has {} free-running rows, not {}",
                        arm.name(),
                        free.len(),
                        manifest.rows.len()
                    ),
                );
            }
        }

        let paired_wins = |treatment: &[FreeRow], control: &[FreeRow]| -> PairedWins {
            let mut b = 0u64;
            let mut c = 0u64;
            for (treatment, control) in treatment.iter().zip(control) {
                match (treatment.exact, control.exact) {
                    (true, false) => b += 1,
                    (false, true) => c += 1,
                    _ => {}
                }
            }
            PairedWins { b, c }
        };
        let wins_no_hint = paired_wins(&exact.2, &no_hint.2);
        let wins_random_hint = paired_wins(&exact.2, &random_hint.2);
        let pass = primary_pass(wins_no_hint, wins_random_hint);

        let random_norm: Vec<Option<String>> = random_hint
            .2
            .iter()
            .map(|row| row.normalized.clone())
            .collect();
        let planted = match planted_wrong_holds(manifest, &random_norm) {
            Ok(value) => value,
            Err(e) => return stop_copy(report, &lines, out_dir, e),
        };
        let mut rows_equal_to_exact = Vec::new();
        let mut applicable = 0u64;
        for (row, result) in manifest.rows.iter().zip(&random_hint.2) {
            if let Some(answer) = &result.normalized {
                applicable += 1;
                if answer == &row.value {
                    rows_equal_to_exact.push(row.id.clone());
                }
            }
        }
        report["primary"] = json!({
            "exact_vs_no_hint": {
                "b": wins_no_hint.b,
                "c": wins_no_hint.c,
                "d": wins_no_hint.d(),
                "clears": clears_primary(wins_no_hint.b, wins_no_hint.c),
            },
            "exact_vs_random_hint": {
                "b": wins_random_hint.b,
                "c": wins_random_hint.c,
                "d": wins_random_hint.d(),
                "clears": clears_primary(wins_random_hint.b, wins_random_hint.c),
            },
            "pass": pass,
            "rule": "d > 24 && d*d > 4*(b+c) for both primary comparisons; teacher-forced top-8 cannot replace either",
        });
        report["planted_wrong"] = json!({
            "arm": "random_hint",
            "applicable_rows": applicable,
            "rows_equal_to_exact": rows_equal_to_exact,
            "holds": planted,
            "rule": "every applicable random-hint normalized answer differs from the exact value; a vacuous arm does not hold",
        });
        if !planted {
            let reason = if applicable == 0 {
                "the random-hint arm produced no normalized emission (vacuous planted control)"
                    .to_string()
            } else {
                format!(
                    "the random-hint arm returned the exact value on {} applicable rows: {:?}",
                    rows_equal_to_exact.len(),
                    rows_equal_to_exact
                )
            };
            return stop_copy(report, &lines, out_dir, reason);
        }

        let decision = if pass { "PASS" } else { "COPY_GATE_FAIL" };
        report["decision"] = json!(decision);
        write_new(&out_dir.join("copy-rows.jsonl"), &lines)?;
        write_json(&out_dir.join("copy-report.json"), &report)?;
        Ok(CopyOutcome {
            primary_pass: pass,
            decision: decision.to_owned(),
        })
    }
}
