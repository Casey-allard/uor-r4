//! A reviewed derivative of raw teacher paraphrases (#1552).
//!
//! ```text
//! paraphrase-review out=NEW_ROOT raw=P1.jsonl,P2.jsonl decisions=DECISIONS.tsv rubric=RUBRIC.md
//! ```
//!
//! The raw files are named `p1`, `p2`, … in the order given. The decisions
//! and their checks are those of `uor_r4_training::paraphrase_review`. Every
//! input is read and checked before the root is claimed. The sealed root
//! holds `paraphrases.jsonl` (kept and relabelled rows, with their source row
//! and file digest), `review.json` (digests and tallies) and copies of the
//! decisions and the rubric. Raw files are never changed.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_training::paraphrase_review::{apply, parse_decisions, raw_rows};
use uor_r4_training::sha256_file;

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("paraphrase-review: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Error> {
    const KEYS: [&str; 4] = ["out", "raw", "decisions", "rubric"];
    let mut args = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {argument}"))?;
        if !KEYS.contains(&key) || args.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("unknown or repeated argument {key}").into());
        }
    }
    let path = |key: &str| -> Result<PathBuf, Error> {
        Ok(PathBuf::from(
            args.get(key).ok_or_else(|| format!("missing {key}="))?,
        ))
    };
    let out = path("out")?;
    let decisions_path = path("decisions")?;
    let rubric_path = path("rubric")?;
    let raw_paths: Vec<PathBuf> = args
        .get("raw")
        .ok_or("missing raw=")?
        .split(',')
        .filter(|p| !p.trim().is_empty())
        .map(|p| PathBuf::from(p.trim()))
        .collect();
    if raw_paths.is_empty() {
        return Err("raw= names no file".into());
    }
    let mut raws = Vec::new();
    let mut files = Vec::new();
    for (index, raw_path) in raw_paths.iter().enumerate() {
        let name = format!("p{}", index + 1);
        let content = fs::read_to_string(raw_path)?;
        let digest = sha256_file(raw_path)?;
        let rows = raw_rows(&name, &content, &digest)?;
        files.push(json!({
            "name": name,
            "path": raw_path.display().to_string(),
            "sha256": digest,
            "rows": rows.len(),
        }));
        raws.extend(rows);
    }
    let review = apply(
        &raws,
        &parse_decisions(&fs::read_to_string(&decisions_path)?)?,
    )?;
    let mut lines = String::new();
    for row in &review.rows {
        lines.push_str(&serde_json::to_string(row)?);
        lines.push('\n');
    }
    let record = json!({
        "schema": "uor-r4.paraphrase-review/1",
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "raw": files,
        "decisions_sha256": sha256_file(&decisions_path)?,
        "rubric_sha256": sha256_file(&rubric_path)?,
        "rows": raws.len(),
        "decisions": review.decisions,
        "relabels": review.relabels,
        "output_rows": review.rows.len(),
        "output_by_label": review.by_label,
        "relations_covered": review
            .rows
            .iter()
            .filter_map(|row| row["relation"].as_str())
            .collect::<BTreeSet<_>>(),
    });
    report_output::claim(&out)?;
    let result = write_root(&out, &lines, &decisions_path, &rubric_path, record);
    if let Err(error) = &result {
        let _ = fs::write(
            out.join("error.json"),
            json!({"error": error.to_string()}).to_string(),
        );
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    println!(
        "{} raw rows: {:?}; {} reviewed rows written",
        raws.len(),
        review.decisions,
        review.rows.len()
    );
    Ok(())
}

fn write_root(
    out: &Path,
    lines: &str,
    decisions: &Path,
    rubric: &Path,
    mut record: Value,
) -> Result<(), Error> {
    fs::write(out.join("paraphrases.jsonl"), lines)?;
    fs::copy(decisions, out.join("decisions.tsv"))?;
    fs::copy(rubric, out.join("rubric.md"))?;
    record["paraphrases_sha256"] = json!(sha256_file(&out.join("paraphrases.jsonl"))?);
    fs::write(out.join("review.json"), serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}
