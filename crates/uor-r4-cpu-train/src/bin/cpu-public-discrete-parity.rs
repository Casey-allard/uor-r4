use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use uor_r4_cpu_train::{CpuDiscreteTrunkReader, VOCABULARY};
use uor_r4_discrete::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

const ARTIFACT_SHA256: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TOKENS_SHA256: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const MASK_SHA256: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
const DRIVER_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const MODEL_TOKENIZER_SHA256: &str =
    "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const MANIFEST_SHA256: &str = "ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab";
const PAYLOAD_TOKENS: u64 = 42_856;
const ELIGIBLE_TARGETS: u64 = 14_287;
const EXPECTED_ADMITTED: u64 = 9_012;
const EXPECTED_TOP1: u64 = 7_002;
const EXPECTED_TOP8: u64 = 7_485;

fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: cpu-public-discrete-parity <fixture-dir>".to_owned())?;
    let artifact = read_hashed(
        &root.join("t16-frozen-zero-seed1-trunk.d11"),
        ARTIFACT_SHA256,
    )?;
    authenticate_manifest(&root)?;
    authenticate_tokenizers(&root)?;
    let cpu = CpuDiscreteTrunkReader::from_bytes(&artifact).map_err(|error| error.to_string())?;
    let model = IntegerDiscreteModel::from_bytes(&artifact)
        .map_err(|error| format!("public artifact: {error}"))?;
    let corpus = PreparedCorpus::load(
        &CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: root.join("dev-tokens.u16"),
                sha256: TOKENS_SHA256.to_owned(),
            },
            mask: FilePin {
                path: root.join("dev-response-mask.u8"),
                sha256: MASK_SHA256.to_owned(),
            },
            expected_payload_tokens: PAYLOAD_TOKENS,
            expected_eligible_targets: ELIGIBLE_TARGETS,
        },
        VOCABULARY as u32,
        &BosIdentity {
            token: "<|bos|>".to_owned(),
            id: 0,
        },
    )
    .map_err(|error| format!("public DEV panel: {error}"))?;

    let mut result = PanelResult::default();
    let started = Instant::now();
    let mut cursor = 0u64;
    while cursor < ELIGIBLE_TARGETS {
        let batch = corpus
            .batch(cursor, 192, 64)
            .map_err(|error| format!("DEV batch at {cursor}: {error}"))?;
        for (window, &target) in batch.windows.iter().zip(&batch.targets) {
            let prefix: Vec<u32> = window.iter().map(|identity| identity.token).collect();
            let evaluation = model
                .evaluate_prefix(&prefix)
                .map_err(|error| format!("model row {}: {error}", result.rows))?;
            let (decision, target_admitted, target_top1, target_top8) = cpu
                .observe_target(&prefix, target)
                .map_err(|error| format!("CPU row {}: {error}", result.rows))?;
            let mut served_top8 = [u32::MAX; 8];
            for (slot, &token) in evaluation.top8.iter().take(8).enumerate() {
                served_top8[slot] = token;
            }
            result.rows += 1;
            result.query_coordinates_checked += decision.query.len() as u64;
            result.query_mismatches += decision
                .query
                .iter()
                .zip(&evaluation.read_vector)
                .filter(|(left, right)| left != right)
                .count() as u64;
            result.cell_mismatches +=
                u64::from(evaluation.admitted_cells != decision.admitted_cells);
            result.top8_mismatches += u64::from(served_top8 != decision.top8);
            result.token_mismatches += u64::from(evaluation.prediction != decision.token);
            result.admitted += u64::from(target_admitted);
            result.top1 += u64::from(target_top1);
            result.top8 += u64::from(target_top8);
            for &value in &decision.query {
                result.checksum = result
                    .checksum
                    .rotate_left(7)
                    .wrapping_add(value as u32 as u64);
            }
            for &token in &decision.top8 {
                result.checksum = result
                    .checksum
                    .rotate_left(7)
                    .wrapping_add(u64::from(token));
            }
        }
        cursor = batch.next_target;
    }
    let elapsed = started.elapsed().as_secs_f64();
    if result.rows != ELIGIBLE_TARGETS
        || result.admitted != EXPECTED_ADMITTED
        || result.top1 != EXPECTED_TOP1
        || result.top8 != EXPECTED_TOP8
        || result.query_mismatches != 0
        || result.cell_mismatches != 0
        || result.top8_mismatches != 0
        || result.token_mismatches != 0
    {
        return Err(format!("parity failure: {result:?}"));
    }
    println!(
        "{}",
        json!({
            "schema": "uor-r4.cpu-public-discrete-parity/1",
            "artifact_sha256": ARTIFACT_SHA256,
            "tokens_sha256": TOKENS_SHA256,
            "mask_sha256": MASK_SHA256,
            "driver_tokenizer_sha256": DRIVER_TOKENIZER_SHA256,
            "model_tokenizer_sha256": MODEL_TOKENIZER_SHA256,
            "manifest_sha256": MANIFEST_SHA256,
            "rows": result.rows,
            "threads": 1,
            "elapsed_ms": elapsed * 1_000.0,
            "rows_per_second": result.rows as f64 / elapsed,
            "query_coordinates_checked": result.query_coordinates_checked,
            "query_coordinate_mismatches": result.query_mismatches,
            "cell_mismatches": result.cell_mismatches,
            "top8_mismatches": result.top8_mismatches,
            "token_mismatches": result.token_mismatches,
            "target_admitted": result.admitted,
            "top1": result.top1,
            "top8": result.top8,
            "checksum": format!("{:016x}", result.checksum),
            "peak_rss_kib": peak_rss_kib(),
        })
    );
    Ok(())
}

#[derive(Debug, Default)]
struct PanelResult {
    rows: u64,
    query_coordinates_checked: u64,
    query_mismatches: u64,
    cell_mismatches: u64,
    top8_mismatches: u64,
    token_mismatches: u64,
    admitted: u64,
    top1: u64,
    top8: u64,
    checksum: u64,
}

fn read_hashed(path: &Path, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let actual = sha256(&bytes);
    if actual != expected {
        return Err(format!(
            "{} SHA-256 differs: {actual} != {expected}",
            path.display()
        ));
    }
    Ok(bytes)
}

fn authenticate_manifest(root: &Path) -> Result<(), String> {
    let bytes = read_hashed(&root.join("dev-manifest.json"), MANIFEST_SHA256)?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("DEV manifest: {error}"))?;
    if manifest["tokens"].as_u64() != Some(PAYLOAD_TOKENS)
        || manifest["response_tokens"].as_u64() != Some(ELIGIBLE_TARGETS)
        || manifest["tokens_sha256"].as_str() != Some(TOKENS_SHA256)
        || manifest["mask_sha256"].as_str() != Some(MASK_SHA256)
        || manifest["tokenizer"]["sha256"].as_str() != Some(DRIVER_TOKENIZER_SHA256)
    {
        return Err("DEV manifest inventory differs".to_owned());
    }
    Ok(())
}

fn authenticate_tokenizers(root: &Path) -> Result<(), String> {
    read_hashed(&root.join("driver-tokenizer.json"), DRIVER_TOKENIZER_SHA256)?;
    read_hashed(&root.join("tokenizer-seed-1.json"), MODEL_TOKENIZER_SHA256)?;
    Ok(())
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}
