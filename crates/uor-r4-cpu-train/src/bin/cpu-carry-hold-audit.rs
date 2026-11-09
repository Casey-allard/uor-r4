//! Read-only CPU audit of Claude-334's saved carry-hold checkpoint fields.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::ask_offset_audit::{AskOffsetRelationReading, TrainedAskOffsets};

const SCHEMA: &str = "uor-r4.cpu-carry-hold-audit/1";
const RELATION_TOKENS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-carry-hold-audit: {error}");
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
    let expected_rows = args["expected-hold-rows"]
        .parse::<usize>()
        .map_err(|error| format!("--expected-hold-rows: {error}"))?;
    if expected_rows == 0 {
        return Err("--expected-hold-rows must be positive".to_owned());
    }
    let out = PathBuf::from(&args["out"]);
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }

    let offsets = TrainedAskOffsets::from_bytes(&model, &hamming)?;
    let depths = offsets.depths().collect::<Vec<_>>();
    if depths != [2, 6] {
        return Err(format!(
            "held checkpoint read depths are {depths:?}, not the Claude-334 [2, 6]"
        ));
    }
    let hold = offsets
        .hold_summary()
        .ok_or_else(|| "checkpoint carries no saved d2.hold".to_owned())?;
    if hold.rows != expected_rows {
        return Err(format!(
            "checkpoint holds for {} rows, expected {expected_rows}",
            hold.rows
        ));
    }
    let relations = offsets.audit_relations(RELATION_TOKENS, QUERY_SHIFT)?;
    let ask = summarize_ask(&relations);
    let executable_sha256 = executable_sha256()?;
    let report = serde_json::json!({
        "schema": SCHEMA,
        "checkpoint": {
            "model_sha256": model_sha,
            "hamming_sha256": hamming_sha,
            "vocab": offsets.vocab(),
            "width": offsets.width(),
            "heads": offsets.heads(),
            "ask_heads": offsets.ask_head_count(),
            "depths": depths,
            "hold_rows": hold.rows,
        },
        "settings": {
            "relation_tokens": RELATION_TOKENS,
            "query_shift": QUERY_SHIFT,
            "expected_hold_rows": expected_rows,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256,
        },
        "hold": hold,
        "ask": ask,
        "relations": relations,
        "claim": "read-only held-checkpoint structure, gain and ask-selection audit; not B2, carry, capability or serving evidence"
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
            "hold": report["hold"],
            "ask": report["ask"],
        })
    );
    Ok(())
}

fn summarize_ask(relations: &[AskOffsetRelationReading]) -> serde_json::Value {
    let cells = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    serde_json::json!({
        "relations": relations.len(),
        "cells": cells.len(),
        "zero_exact_codes": cells.iter().filter(|cell| cell.zero_bits == 0).count(),
        "trained_exact_codes": cells.iter().filter(|cell| cell.trained_bits == 0).count(),
        "zero_total_bits": cells.iter().map(|cell| u64::from(cell.zero_bits)).sum::<u64>(),
        "trained_total_bits": cells.iter().map(|cell| u64::from(cell.trained_bits)).sum::<u64>(),
        "improvement_bits": cells.iter().map(|cell| i64::from(cell.improvement_bits)).sum::<i64>(),
        "improved_cells": cells.iter().filter(|cell| cell.improvement_bits > 0).count(),
        "regressed_cells": cells.iter().filter(|cell| cell.improvement_bits < 0).count(),
        "zero_target_nearest": cells.iter().filter(|cell| cell.zero_target_nearest).count(),
        "trained_target_nearest": cells.iter().filter(|cell| cell.trained_target_nearest).count(),
    })
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 5] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "expected-hold-rows",
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

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("--expected-{name}-sha256 is not a SHA-256"));
    }
    Ok(())
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
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(format!("{}: {error}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-carry-hold-audit-{name}-{}",
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
        let mut gain = vec![0.0f32; vocab];
        gain[1] = 0.5;
        gain[2] = -0.25;
        let mut weight = vec![0.0f32; 2 * width];
        weight[width + 1] = 1.5;
        let hamming = tensor_file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(&rotation),
            ),
            ("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(&ask)),
            ("d2.weight", Dtype::F32, vec![2, width], f32_bytes(&weight)),
            ("d2.hold", Dtype::F32, vec![vocab], f32_bytes(&gain)),
            ("hold_rows", Dtype::U32, vec![1], u32_bytes(&[8])),
            (
                "d6.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(&rotation),
            ),
            ("d6.ask", Dtype::F32, vec![vocab, width], f32_bytes(&ask)),
        ]);
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        (
            uor_r4_discrete::corpus::sha256(&model),
            uor_r4_discrete::corpus::sha256(&hamming),
        )
    }

    fn args(
        dir: &TestDir,
        checkpoint: &Path,
        model: &str,
        hamming: &str,
    ) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("checkpoint".to_owned(), checkpoint.display().to_string()),
            ("expected-model-sha256".to_owned(), model.to_owned()),
            ("expected-hamming-sha256".to_owned(), hamming.to_owned()),
            ("expected-hold-rows".to_owned(), "8".to_owned()),
            (
                "out".to_owned(),
                dir.0.join("report.json").display().to_string(),
            ),
        ])
    }

    #[test]
    fn synthetic_report_is_complete_and_deterministic() {
        let dir = TestDir::new("report");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model, hamming) = write_checkpoint(&checkpoint);
        let args = args(&dir, &checkpoint, &model, &hamming);
        run_with(&args).unwrap();
        let out = PathBuf::from(&args["out"]);
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(report["schema"], SCHEMA);
        assert_eq!(report["checkpoint"]["hold_rows"], 8);
        assert_eq!(report["hold"]["gain_nonzero"], 2);
        assert_eq!(report["hold"]["write_weight_nonzero"], 1);
        assert_eq!(report["ask"]["trained_target_nearest"], 28);
        assert_eq!(report["ask"]["regressed_cells"], 2);

        let mut repeat_args = args.clone();
        repeat_args.insert(
            "out".to_owned(),
            dir.0.join("repeat.json").display().to_string(),
        );
        run_with(&repeat_args).unwrap();
        assert_eq!(
            std::fs::read(&out).unwrap(),
            std::fs::read(&repeat_args["out"]).unwrap()
        );
        assert!(run_with(&args)
            .unwrap_err()
            .contains("reports are never replaced"));
        if let Ok(path) = std::env::var("UOR_KEEP_CARRY_HOLD_AUDIT_EVIDENCE") {
            std::fs::write(path, std::fs::read(&out).unwrap()).unwrap();
        }
    }

    #[test]
    fn hashes_and_expected_rows_are_refused() {
        let dir = TestDir::new("refuse");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model, hamming) = write_checkpoint(&checkpoint);

        let mut bad_hash = args(&dir, &checkpoint, &model, &hamming);
        bad_hash.insert("expected-model-sha256".to_owned(), "0".repeat(64));
        assert!(run_with(&bad_hash)
            .unwrap_err()
            .contains("model.safetensors SHA-256 differs"));

        let mut bad_hamming = args(&dir, &checkpoint, &model, &hamming);
        bad_hamming.insert("expected-hamming-sha256".to_owned(), "0".repeat(64));
        assert!(run_with(&bad_hamming)
            .unwrap_err()
            .contains("hamming_read.safetensors SHA-256 differs"));

        let mut bad_rows = args(&dir, &checkpoint, &model, &hamming);
        bad_rows.insert("expected-hold-rows".to_owned(), "7".to_owned());
        assert!(run_with(&bad_rows)
            .unwrap_err()
            .contains("checkpoint holds for 8 rows, expected 7"));
    }

    #[test]
    fn parser_requires_every_flag_once() {
        let base = [
            "--checkpoint",
            "c",
            "--expected-model-sha256",
            "m",
            "--expected-hamming-sha256",
            "h",
            "--expected-hold-rows",
            "8",
            "--out",
            "o",
        ];
        assert_eq!(
            parse_args(base.into_iter().map(str::to_owned))
                .unwrap()
                .len(),
            5
        );
        assert!(parse_args(["--checkpoint", "c"].into_iter().map(str::to_owned)).is_err());
    }
}
