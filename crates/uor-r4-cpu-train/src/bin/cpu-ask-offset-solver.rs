//! Offline CPU solver for Claude-330's checkpoint-compatible ask tables.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use uor_r4_cpu_train::ask_offset_audit::TrainedAskOffsets;
use uor_r4_cpu_train::ask_offset_solver::{AskOffsetSolveBasis, AskPullCorpus};

const SCHEMA: &str = "uor-r4.cpu-ask-offset-solver/1";
const DEFAULT_MAX_EVALUATIONS: u64 = 200_000;
const RELATIONS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;
static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-ask-offset-solver: {error}");
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
    let out = resolve_output_path(Path::new(&args["out"]))?;
    let hamming_out = resolve_output_path(Path::new(&args["hamming-out"]))?;
    if out == hamming_out {
        return Err("--out and --hamming-out must be distinct".to_owned());
    }
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    if hamming_out.exists() {
        return Err(format!(
            "{} exists; candidate sidecars are never replaced",
            hamming_out.display()
        ));
    }
    let (corpus, pull_source, pull_path, pull_sha) = match args.get("pulls") {
        Some(path) => {
            let bytes = std::fs::read(path).map_err(|error| format!("{path}: {error}"))?;
            let corpus = AskPullCorpus::from_json(&bytes)?;
            let sha = uor_r4_discrete::corpus::sha256(&bytes);
            (corpus, "file", Some(path.clone()), sha)
        }
        None => {
            let corpus = AskPullCorpus::b2_relations(RELATIONS, QUERY_SHIFT)?;
            let bytes = serde_json::to_vec(&corpus).map_err(|error| error.to_string())?;
            (
                corpus,
                "default-b2",
                None,
                uor_r4_discrete::corpus::sha256(&bytes),
            )
        }
    };
    if let Some(expected) = args.get("expected-pulls-sha256") {
        validate_sha256(expected, "pulls")?;
        if pull_sha != *expected {
            return Err(format!("pull corpus SHA-256 differs: got {pull_sha}"));
        }
    }
    let max_evaluations = args
        .get("max-evaluations")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "--max-evaluations wants a positive integer".to_owned())
        })
        .transpose()?
        .unwrap_or(DEFAULT_MAX_EVALUATIONS);
    let basis = AskOffsetSolveBasis::from_bytes(&model, &hamming)?;
    let solved = basis.solve(&corpus, max_evaluations)?;
    let corpus_audit = basis.audit_candidate(&solved.hamming_bytes, &corpus)?;
    let audited = TrainedAskOffsets::from_bytes(&model, &solved.hamming_bytes)?;
    let relations = audited.audit_relations(RELATIONS, QUERY_SHIFT)?;
    let audit_cells = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    let audit_trained_total_bits = checked_sum(
        audit_cells.iter().map(|cell| u64::from(cell.trained_bits)),
        "audited trained bits",
    )?;
    let audit_zero_total_bits = checked_sum(
        audit_cells.iter().map(|cell| u64::from(cell.zero_bits)),
        "audited zero bits",
    )?;
    let audit_summary = serde_json::json!({
        "cells": audit_cells.len(),
        "trained_exact_codes": audit_cells.iter().filter(|cell| cell.trained_bits == 0).count(),
        "trained_target_nearest": audit_cells.iter().filter(|cell| cell.trained_target_nearest).count(),
        "zero_target_nearest": audit_cells.iter().filter(|cell| cell.zero_target_nearest).count(),
        "trained_total_bits": audit_trained_total_bits,
        "zero_total_bits": audit_zero_total_bits,
    });
    let candidate_sha = uor_r4_discrete::corpus::sha256(&solved.hamming_bytes);
    let zero_weighted_bits = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.zero_weighted_bits),
        "zero weighted bits",
    )?;
    let solved_weighted_bits = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.solved_weighted_bits),
        "solved weighted bits",
    )?;
    let improvement_bits = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.improvement_bits),
        "improvement bits",
    )?;
    let candidate_evaluations = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.candidate_evaluations),
        "candidate evaluations",
    )?;
    let coordinate_evaluations = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.coordinate_evaluations),
        "coordinate evaluations",
    )?;
    let total_evaluations = checked_sum(
        solved
            .readings
            .iter()
            .map(|reading| reading.total_evaluations),
        "total evaluations",
    )?;
    let search_starts = checked_sum(
        solved.readings.iter().map(|reading| reading.search_starts),
        "search starts",
    )?;
    let summary = serde_json::json!({
        "questions": solved.readings.iter().map(|reading| reading.question).collect::<std::collections::BTreeSet<_>>().len(),
        "depths": solved.readings.iter().map(|reading| reading.depth).collect::<std::collections::BTreeSet<_>>().len(),
        "readings": solved.readings.len(),
        "zero_weighted_bits": zero_weighted_bits,
        "solved_weighted_bits": solved_weighted_bits,
        "improvement_bits": improvement_bits,
        "improved_readings": solved.readings.iter().filter(|reading| reading.improvement_bits > 0).count(),
        "candidate_evaluations": candidate_evaluations,
        "coordinate_evaluations": coordinate_evaluations,
        "total_evaluations": total_evaluations,
        "search_starts": search_starts,
        "budget_exhausted_readings": solved.readings.iter().filter(|reading| reading.budget_exhausted).count(),
    });
    let corpus_zero_weighted_bits = checked_sum(
        corpus_audit
            .iter()
            .map(|reading| reading.zero_weighted_bits),
        "corpus-audit zero weighted bits",
    )?;
    let corpus_candidate_weighted_bits = checked_sum(
        corpus_audit
            .iter()
            .map(|reading| reading.candidate_weighted_bits),
        "corpus-audit candidate weighted bits",
    )?;
    let corpus_improvement_bits = checked_sum(
        corpus_audit.iter().map(|reading| reading.improvement_bits),
        "corpus-audit improvement bits",
    )?;
    let corpus_audit_summary = serde_json::json!({
        "readings": corpus_audit.len(),
        "zero_weighted_bits": corpus_zero_weighted_bits,
        "candidate_weighted_bits": corpus_candidate_weighted_bits,
        "improvement_bits": corpus_improvement_bits,
        "improved_readings": corpus_audit.iter().filter(|reading| reading.improvement_bits > 0).count(),
    });
    let report = serde_json::json!({
        "schema": SCHEMA,
        "checkpoint": {
            "model_sha256": model_sha,
            "hamming_sha256": hamming_sha,
            "vocab": audited.vocab(),
            "width": audited.width(),
            "heads": audited.heads(),
            "ask_heads": audited.ask_head_count(),
            "depths": audited.depths().collect::<Vec<_>>(),
        },
        "pull_corpus": {
            "source": pull_source,
            "path": pull_path,
            "sha256": pull_sha,
            "schema": corpus.schema,
        },
        "settings": {
            "max_evaluations_per_question_depth": max_evaluations,
            "audit_relations": RELATIONS,
            "query_shift": QUERY_SHIFT,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256()?,
        },
        "summary": summary,
        "audit_after_solve": audit_summary,
        "corpus_audit_after_solve": corpus_audit_summary,
        "corpus_audit_readings": corpus_audit,
        "candidate_hamming_sha256": candidate_sha,
        "readings": solved.readings,
        "claim": "offline checkpoint ask-offset initializer; not a trained ask-head model, B2 result, artifact schema or serving result"
    });
    let summary = report["summary"].clone();
    let mut report_bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    report_bytes.push(b'\n');
    write_new_pair(&hamming_out, &solved.hamming_bytes, &out, &report_bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "hamming_out": hamming_out,
            "report_sha256": uor_r4_discrete::corpus::sha256(&report_bytes),
            "candidate_hamming_sha256": candidate_sha,
            "pull_corpus_sha256": pull_sha,
            "summary": summary,
        })
    );
    Ok(())
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("--expected-{name}-sha256 is not a SHA-256"));
    }
    Ok(())
}

fn checked_sum(values: impl IntoIterator<Item = u64>, what: &str) -> Result<u64> {
    values
        .into_iter()
        .try_fold(0u64, u64::checked_add)
        .ok_or_else(|| format!("{what} overflow"))
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 8] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "expected-pulls-sha256",
        "out",
        "hamming-out",
        "pulls",
        "max-evaluations",
    ];
    const REQUIRED: [&str; 5] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "out",
        "hamming-out",
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
    for name in REQUIRED {
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

fn resolve_output_path(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no output file name", path.display()))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent =
        std::fs::canonicalize(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    Ok(parent.join(name))
}

fn write_temp_new(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} has no output file name", path.display()))?;
    for _ in 0..100 {
        let serial = TEMP_SERIAL.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(name);
        temporary_name.push(format!(".uor-tmp-{}-{serial}", std::process::id()));
        let temporary = path.with_file_name(temporary_name);
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{}: {error}", temporary.display())),
        };
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            let _ = std::fs::remove_file(&temporary);
            return Err(format!("{}: {error}", temporary.display()));
        }
        return Ok(temporary);
    }
    Err(format!(
        "{}: could not allocate a temporary sibling",
        path.display()
    ))
}

fn promote_new(temporary: &Path, final_path: &Path) -> Result<()> {
    std::fs::hard_link(temporary, final_path)
        .map_err(|error| format!("{}: {error}", final_path.display()))?;
    if let Err(error) = std::fs::remove_file(temporary) {
        let _ = std::fs::remove_file(final_path);
        return Err(format!("{}: {error}", temporary.display()));
    }
    Ok(())
}

fn write_new_pair(
    first_path: &Path,
    first_bytes: &[u8],
    second_path: &Path,
    second_bytes: &[u8],
) -> Result<()> {
    let first_temporary = write_temp_new(first_path, first_bytes)?;
    let second_temporary = match write_temp_new(second_path, second_bytes) {
        Ok(path) => path,
        Err(error) => {
            let _ = std::fs::remove_file(&first_temporary);
            return Err(error);
        }
    };
    if let Err(error) = promote_new(&first_temporary, first_path) {
        let _ = std::fs::remove_file(&first_temporary);
        let _ = std::fs::remove_file(&second_temporary);
        return Err(error);
    }
    if let Err(error) = promote_new(&second_temporary, second_path) {
        let _ = std::fs::remove_file(&second_temporary);
        let _ = std::fs::remove_file(first_path);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
fn retain_evidence(root: &Path, hamming: &[u8], report: &[u8]) -> Result<()> {
    std::fs::create_dir(root).map_err(|error| format!("{}: {error}", root.display()))?;
    write_new_pair(
        &root.join("synthetic-hamming.safetensors"),
        hamming,
        &root.join("synthetic-report.json"),
        report,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-cpu-ask-offset-solver-{name}-{}",
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
        table[17 * width..18 * width].fill(-1.0);
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut tensors = vec![("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32]))];
        for depth in [2usize, 6] {
            for part in ["query_rotation", "key_rotation", "value_rotation"] {
                tensors.push((
                    match (depth, part) {
                        (2, "query_rotation") => "d2.query_rotation",
                        (2, "key_rotation") => "d2.key_rotation",
                        (2, "value_rotation") => "d2.value_rotation",
                        (6, "query_rotation") => "d6.query_rotation",
                        (6, "key_rotation") => "d6.key_rotation",
                        (6, "value_rotation") => "d6.value_rotation",
                        _ => unreachable!(),
                    },
                    Dtype::F32,
                    vec![width / 4, 4],
                    f32_bytes(&rotation),
                ));
            }
            tensors.push((
                match depth {
                    2 => "d2.margin",
                    6 => "d6.margin",
                    _ => unreachable!(),
                },
                Dtype::F32,
                vec![heads],
                f32_bytes(&vec![2.0; heads]),
            ));
            tensors.push((
                match depth {
                    2 => "d2.weight",
                    6 => "d6.weight",
                    _ => unreachable!(),
                },
                Dtype::F32,
                vec![2, width],
                f32_bytes(&vec![0.0; 2 * width]),
            ));
        }
        let hamming = tensor_file(tensors);
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        (
            uor_r4_discrete::corpus::sha256(&model),
            uor_r4_discrete::corpus::sha256(&hamming),
        )
    }

    #[test]
    fn parse_requires_outputs_and_hashes() {
        let base = [
            "--checkpoint",
            "c",
            "--expected-model-sha256",
            "m",
            "--expected-hamming-sha256",
            "h",
            "--out",
            "o",
            "--hamming-out",
            "s",
        ];
        let parsed = parse_args(base.into_iter().map(str::to_owned)).unwrap();
        assert_eq!(parsed.len(), 5);
        assert!(parse_args(["--checkpoint", "c"].into_iter().map(str::to_owned)).is_err());
    }

    #[test]
    fn sha_and_default_evaluations_are_validated() {
        assert!(validate_sha256(&"a".repeat(64), "model").is_ok());
        assert!(validate_sha256(&"a".repeat(63), "model").is_err());
        assert_eq!(DEFAULT_MAX_EVALUATIONS, 200_000);
        assert_eq!(checked_sum([1, 2, 3], "test").unwrap(), 6);
        assert!(checked_sum([u64::MAX, 1], "test").is_err());
    }

    #[test]
    fn synthetic_checkpoint_writes_solved_sidecar_report_and_repeat() {
        let dir = TestDir::new("report");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model_sha, hamming_sha) = write_checkpoint(&checkpoint);
        let args_for = |suffix: &str| {
            BTreeMap::from([
                ("checkpoint".to_owned(), checkpoint.display().to_string()),
                ("expected-model-sha256".to_owned(), model_sha.clone()),
                ("expected-hamming-sha256".to_owned(), hamming_sha.clone()),
                (
                    "out".to_owned(),
                    dir.0
                        .join(format!("report-{suffix}.json"))
                        .display()
                        .to_string(),
                ),
                (
                    "hamming-out".to_owned(),
                    dir.0
                        .join(format!("hamming-{suffix}.safetensors"))
                        .display()
                        .to_string(),
                ),
            ])
        };
        let args = args_for("one");
        let mut collision = args.clone();
        collision.insert("hamming-out".to_owned(), args["out"].clone());
        assert!(run_with(&collision)
            .err()
            .expect("colliding outputs")
            .contains("must be distinct"));
        let alias_parent = dir.0.join("alias-parent");
        std::fs::create_dir(&alias_parent).unwrap();
        let mut alias = args.clone();
        alias.insert(
            "out".to_owned(),
            alias_parent
                .join("..")
                .join("alias-output")
                .display()
                .to_string(),
        );
        alias.insert(
            "hamming-out".to_owned(),
            dir.0.join("alias-output").display().to_string(),
        );
        assert!(run_with(&alias)
            .err()
            .expect("aliased outputs")
            .contains("must be distinct"));
        let mut wrong_corpus_hash = args.clone();
        wrong_corpus_hash.insert("expected-pulls-sha256".to_owned(), "0".repeat(64));
        assert!(run_with(&wrong_corpus_hash)
            .err()
            .expect("wrong corpus hash")
            .contains("pull corpus SHA-256 differs"));
        run_with(&args).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&args["out"]).unwrap()).unwrap();
        assert_eq!(report["schema"], SCHEMA);
        assert_eq!(report["summary"]["questions"], 16);
        assert_eq!(report["summary"]["solved_weighted_bits"], 0);
        assert_eq!(report["summary"]["budget_exhausted_readings"], 0);
        assert_eq!(report["pull_corpus"]["source"], "default-b2");
        assert_eq!(
            report["corpus_audit_after_solve"]["candidate_weighted_bits"],
            report["summary"]["solved_weighted_bits"]
        );
        assert_eq!(
            report["audit_after_solve"]["trained_target_nearest"],
            report["audit_after_solve"]["cells"]
        );

        let repeat = args_for("two");
        run_with(&repeat).unwrap();
        assert_eq!(
            std::fs::read(&args["hamming-out"]).unwrap(),
            std::fs::read(&repeat["hamming-out"]).unwrap()
        );
        assert_eq!(
            std::fs::read(&args["out"]).unwrap(),
            std::fs::read(&repeat["out"]).unwrap()
        );
        assert!(run_with(&args)
            .unwrap_err()
            .contains("reports are never replaced"));

        if let Ok(root) = std::env::var("UOR_KEEP_ASK_OFFSET_SOLVER_EVIDENCE") {
            retain_evidence(
                Path::new(&root),
                &std::fs::read(&args["hamming-out"]).unwrap(),
                &std::fs::read(&args["out"]).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn pair_write_rolls_back_when_the_second_final_was_created() {
        let dir = TestDir::new("pair-write");
        let first = dir.0.join("candidate.safetensors");
        let second = dir.0.join("report.json");
        std::fs::write(&second, b"existing").unwrap();
        assert!(write_new_pair(&first, b"candidate", &second, b"report").is_err());
        assert!(!first.exists());
        assert_eq!(std::fs::read(&second).unwrap(), b"existing");
        assert!(std::fs::read_dir(&dir.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".uor-tmp-")
        }));

        let retained = dir.0.join("retained");
        retain_evidence(&retained, b"candidate", b"report").unwrap();
        assert!(retain_evidence(&retained, b"changed", b"changed").is_err());
        assert_eq!(
            std::fs::read(retained.join("synthetic-hamming.safetensors")).unwrap(),
            b"candidate"
        );
        assert_eq!(
            std::fs::read(retained.join("synthetic-report.json")).unwrap(),
            b"report"
        );
    }
}
