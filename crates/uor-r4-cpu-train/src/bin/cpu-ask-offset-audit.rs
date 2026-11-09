//! Read-only CPU audit of Claude-330's trained ask-offset tables.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use uor_r4_cpu_train::ask_offset_audit::{AskOffsetCellReading, TrainedAskOffsets};

const SCHEMA: &str = "uor-r4.cpu-ask-offset-audit/1";
const RELATION_TOKENS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize)]
struct CellAggregate {
    depth: usize,
    head: usize,
    relations: usize,
    zero_exact_codes: usize,
    trained_exact_codes: usize,
    zero_total_bits: u64,
    trained_total_bits: u64,
    improvement_bits: i64,
    improved_relations: usize,
    regressed_relations: usize,
    zero_target_nearest: usize,
    trained_target_nearest: usize,
    zero_target_rank_sum: usize,
    trained_target_rank_sum: usize,
    relation_rows_with_offsets: usize,
    offset_abs_sum_q16: u64,
    offset_changed_coordinates: usize,
}

#[derive(Debug, Serialize)]
struct DepthAggregate {
    depth: usize,
    cells: usize,
    relations_per_cell: usize,
    zero_exact_codes: usize,
    trained_exact_codes: usize,
    zero_total_bits: u64,
    trained_total_bits: u64,
    improvement_bits: i64,
    improved_relations: usize,
    regressed_relations: usize,
    zero_target_nearest: usize,
    trained_target_nearest: usize,
    offset_abs_sum_q16: u64,
    offset_changed_coordinates: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-ask-offset-audit: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    run_with(&args)
}

fn run_with(args: &BTreeMap<String, String>) -> Result<()> {
    let checkpoint = Path::new(&args["checkpoint"]);
    let model_path = checkpoint.join("model.safetensors");
    let hamming_path = checkpoint.join("hamming_read.safetensors");
    let model =
        std::fs::read(&model_path).map_err(|error| format!("{}: {error}", model_path.display()))?;
    let hamming = std::fs::read(&hamming_path)
        .map_err(|error| format!("{}: {error}", hamming_path.display()))?;
    let model_sha = uor_r4_discrete::corpus::sha256(&model);
    let hamming_sha = uor_r4_discrete::corpus::sha256(&hamming);
    validate_sha256(&args["expected-model-sha256"], "model")?;
    validate_sha256(&args["expected-hamming-sha256"], "hamming")?;
    if model_sha != args["expected-model-sha256"] {
        return Err(format!(
            "model.safetensors SHA-256 differs: got {model_sha}"
        ));
    }
    if hamming_sha != args["expected-hamming-sha256"] {
        return Err(format!(
            "hamming_read.safetensors SHA-256 differs: got {hamming_sha}"
        ));
    }
    let out = PathBuf::from(&args["out"]);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }

    let trained = TrainedAskOffsets::from_bytes(&model, &hamming)?;
    let relations = trained.audit_relations(RELATION_TOKENS, QUERY_SHIFT)?;
    let cells = aggregate_cells(&relations);
    let depth_summaries = aggregate_depths(&cells);
    let all = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    let summary = serde_json::json!({
        "relations": relations.len(),
        "depths": trained.depths().collect::<Vec<_>>(),
        "ask_heads": trained.ask_head_count(),
        "cells": all.len(),
        "zero_exact_codes": all.iter().filter(|cell| cell.zero_bits == 0).count(),
        "trained_exact_codes": all.iter().filter(|cell| cell.trained_bits == 0).count(),
        "zero_total_bits": all.iter().map(|cell| u64::from(cell.zero_bits)).sum::<u64>(),
        "trained_total_bits": all.iter().map(|cell| u64::from(cell.trained_bits)).sum::<u64>(),
        "improvement_bits": all.iter().map(|cell| i64::from(cell.improvement_bits)).sum::<i64>(),
        "improved_cells": all.iter().filter(|cell| cell.improvement_bits > 0).count(),
        "regressed_cells": all.iter().filter(|cell| cell.improvement_bits < 0).count(),
        "zero_target_nearest": all.iter().filter(|cell| cell.zero_target_nearest).count(),
        "trained_target_nearest": all.iter().filter(|cell| cell.trained_target_nearest).count(),
    });
    let report = serde_json::json!({
        "schema": SCHEMA,
        "checkpoint": {
            "model_sha256": model_sha,
            "hamming_sha256": hamming_sha,
            "vocab": trained.vocab(),
            "width": trained.width(),
            "heads": trained.heads(),
            "ask_heads": trained.ask_head_count(),
            "depths": trained.depths().collect::<Vec<_>>(),
        },
        "settings": {
            "relation_tokens": RELATION_TOKENS,
            "query_shift": QUERY_SHIFT,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
        },
        "summary": summary,
        "depth_summaries": depth_summaries,
        "cells": cells,
        "relations": relations,
        "claim": "read-only float-checkpoint ask-offset geometry audit; not B2, retention, capability or serving evidence"
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "sha256": uor_r4_discrete::corpus::sha256(&bytes),
            "summary": report["summary"],
        })
    );
    Ok(())
}

fn aggregate_cells(
    relations: &[uor_r4_cpu_train::ask_offset_audit::AskOffsetRelationReading],
) -> Vec<CellAggregate> {
    let mut grouped: BTreeMap<(usize, usize), Vec<&AskOffsetCellReading>> = BTreeMap::new();
    for cell in relations.iter().flat_map(|relation| relation.cells.iter()) {
        grouped
            .entry((cell.depth, cell.head))
            .or_default()
            .push(cell);
    }
    grouped
        .into_iter()
        .map(|((depth, head), cells)| CellAggregate {
            depth,
            head,
            relations: cells.len(),
            zero_exact_codes: cells.iter().filter(|cell| cell.zero_bits == 0).count(),
            trained_exact_codes: cells.iter().filter(|cell| cell.trained_bits == 0).count(),
            zero_total_bits: cells.iter().map(|cell| u64::from(cell.zero_bits)).sum(),
            trained_total_bits: cells.iter().map(|cell| u64::from(cell.trained_bits)).sum(),
            improvement_bits: cells
                .iter()
                .map(|cell| i64::from(cell.improvement_bits))
                .sum(),
            improved_relations: cells
                .iter()
                .filter(|cell| cell.improvement_bits > 0)
                .count(),
            regressed_relations: cells
                .iter()
                .filter(|cell| cell.improvement_bits < 0)
                .count(),
            zero_target_nearest: cells.iter().filter(|cell| cell.zero_target_nearest).count(),
            trained_target_nearest: cells
                .iter()
                .filter(|cell| cell.trained_target_nearest)
                .count(),
            zero_target_rank_sum: cells.iter().map(|cell| cell.zero_target_rank).sum(),
            trained_target_rank_sum: cells.iter().map(|cell| cell.trained_target_rank).sum(),
            relation_rows_with_offsets: cells
                .iter()
                .filter(|cell| cell.offset_changed_coordinates > 0)
                .count(),
            offset_abs_sum_q16: cells.iter().map(|cell| cell.offset_abs_sum_q16).sum(),
            offset_changed_coordinates: cells
                .iter()
                .map(|cell| cell.offset_changed_coordinates)
                .sum(),
        })
        .collect()
}

fn aggregate_depths(cells: &[CellAggregate]) -> Vec<DepthAggregate> {
    let mut grouped: BTreeMap<usize, Vec<&CellAggregate>> = BTreeMap::new();
    for cell in cells {
        grouped.entry(cell.depth).or_default().push(cell);
    }
    grouped
        .into_iter()
        .map(|(depth, cells)| DepthAggregate {
            depth,
            cells: cells.len(),
            relations_per_cell: cells.first().map_or(0, |cell| cell.relations),
            zero_exact_codes: cells.iter().map(|cell| cell.zero_exact_codes).sum(),
            trained_exact_codes: cells.iter().map(|cell| cell.trained_exact_codes).sum(),
            zero_total_bits: cells.iter().map(|cell| cell.zero_total_bits).sum(),
            trained_total_bits: cells.iter().map(|cell| cell.trained_total_bits).sum(),
            improvement_bits: cells.iter().map(|cell| cell.improvement_bits).sum(),
            improved_relations: cells.iter().map(|cell| cell.improved_relations).sum(),
            regressed_relations: cells.iter().map(|cell| cell.regressed_relations).sum(),
            zero_target_nearest: cells.iter().map(|cell| cell.zero_target_nearest).sum(),
            trained_target_nearest: cells.iter().map(|cell| cell.trained_target_nearest).sum(),
            offset_abs_sum_q16: cells.iter().map(|cell| cell.offset_abs_sum_q16).sum(),
            offset_changed_coordinates: cells
                .iter()
                .map(|cell| cell.offset_changed_coordinates)
                .sum(),
        })
        .collect()
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("--expected-{name}-sha256 is not a SHA-256"));
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 4] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "out",
    ];
    let mut parsed = BTreeMap::new();
    let mut args = args;
    while let Some(argument) = args.next() {
        let name = argument
            .strip_prefix("--")
            .filter(|name| FLAGS.contains(name))
            .ok_or_else(|| format!("unknown argument {argument}"))?;
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        if parsed.insert(name.to_owned(), value).is_some() {
            return Err(format!("{argument} given twice"));
        }
    }
    for name in FLAGS {
        if !parsed.contains_key(name) {
            return Err(format!("--{name} is required"));
        }
    }
    Ok(parsed)
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-cpu-ask-offset-audit-{name}-{}",
                std::process::id()
            ));
            if path.exists() {
                std::fs::remove_dir_all(&path).unwrap();
            }
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, None).unwrap()
    }

    fn write_checkpoint(dir: &Path) -> (String, String) {
        let (vocab, width, heads) = (64usize, 8usize, 2usize);
        let mut table = vec![1.0f32; vocab * width];
        table[18 * width..19 * width].fill(-1.0);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut ask = vec![0.0f32; vocab * width];
        ask[33 * width..34 * width].fill(-2.0);
        let hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(&rotation),
            ),
            ("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(&ask)),
        ]);
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        (
            uor_r4_discrete::corpus::sha256(&model),
            uor_r4_discrete::corpus::sha256(&hamming),
        )
    }

    #[test]
    fn sha256_arguments_are_checked() {
        assert!(validate_sha256(&"a".repeat(64), "model").is_ok());
        assert!(validate_sha256(&"a".repeat(63), "model").is_err());
        assert!(validate_sha256(&"g".repeat(64), "hamming").is_err());
    }

    #[test]
    fn parse_requires_every_flag_once() {
        let ok = parse_args(
            [
                "--checkpoint",
                "c",
                "--expected-model-sha256",
                "m",
                "--expected-hamming-sha256",
                "h",
                "--out",
                "o",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(ok.len(), 4);
        assert!(parse_args(["--checkpoint", "c"].into_iter().map(str::to_owned)).is_err());
        assert!(parse_args(
            [
                "--checkpoint",
                "c",
                "--checkpoint",
                "d",
                "--expected-model-sha256",
                "m",
                "--expected-hamming-sha256",
                "h",
                "--out",
                "o",
            ]
            .into_iter()
            .map(str::to_owned)
        )
        .is_err());
    }

    #[test]
    fn synthetic_checkpoint_writes_a_complete_report_once() {
        let dir = TestDir::new("report");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model_sha, hamming_sha) = write_checkpoint(&checkpoint);
        let out = dir.0.join("report.json");
        let args = BTreeMap::from([
            ("checkpoint".to_owned(), checkpoint.display().to_string()),
            ("expected-model-sha256".to_owned(), model_sha),
            ("expected-hamming-sha256".to_owned(), hamming_sha),
            ("out".to_owned(), out.display().to_string()),
        ]);
        run_with(&args).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(report["schema"], SCHEMA);
        assert_eq!(report["summary"]["relations"], 16);
        assert_eq!(report["summary"]["regressed_cells"], 1);
        assert_eq!(report["depth_summaries"][0]["depth"], 2);
        assert_eq!(report["depth_summaries"][0]["regressed_relations"], 1);
        assert!(run_with(&args)
            .unwrap_err()
            .contains("reports are never replaced"));

        let repeat = dir.0.join("report-repeat.json");
        let mut repeat_args = args.clone();
        repeat_args.insert("out".to_owned(), repeat.display().to_string());
        run_with(&repeat_args).unwrap();
        assert_eq!(
            std::fs::read(&out).unwrap(),
            std::fs::read(&repeat).unwrap()
        );

        if let Ok(path) = std::env::var("UOR_KEEP_ASK_OFFSET_AUDIT_EVIDENCE") {
            std::fs::write(path, std::fs::read(&out).unwrap()).unwrap();
        }
    }

    #[test]
    fn both_wrong_file_hashes_are_refused() {
        let dir = TestDir::new("hashes");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model_sha, hamming_sha) = write_checkpoint(&checkpoint);

        let bad_model = BTreeMap::from([
            ("checkpoint".to_owned(), checkpoint.display().to_string()),
            ("expected-model-sha256".to_owned(), "0".repeat(64)),
            ("expected-hamming-sha256".to_owned(), hamming_sha.clone()),
            (
                "out".to_owned(),
                dir.0.join("bad-model.json").display().to_string(),
            ),
        ]);
        assert!(run_with(&bad_model)
            .unwrap_err()
            .contains("model.safetensors SHA-256 differs"));

        let bad_hamming = BTreeMap::from([
            ("checkpoint".to_owned(), checkpoint.display().to_string()),
            ("expected-model-sha256".to_owned(), model_sha),
            ("expected-hamming-sha256".to_owned(), "0".repeat(64)),
            (
                "out".to_owned(),
                dir.0.join("bad-hamming.json").display().to_string(),
            ),
        ]);
        assert!(run_with(&bad_hamming)
            .unwrap_err()
            .contains("hamming_read.safetensors SHA-256 differs"));
    }
}
