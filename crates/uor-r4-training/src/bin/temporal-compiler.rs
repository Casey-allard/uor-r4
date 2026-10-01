//! Fit or score the learned query-view adapter for the real grounded session.
//!
//! ```text
//! temporal-compiler fit out=NEW_ROOT base=COMPILER.json train=ROWS.jsonl \
//!   [steps=300] [rate=0.5] [l2=0.001]
//! temporal-compiler score out=NEW_ROOT compiler=COMPILER.json data=ROWS.jsonl
//! ```
//!
//! Each labelled row contains `text`, `relation` (a base label, or null for
//! unresolved), `view` (`current`, `initial`, `previous_assertion`,
//! `previous_distinct_value`, `unresolved`) and `source_group`.
//! Labels describe recorded memory events, not presumed real-world chronology.
//! Fit reads only the supplied training file; score reads an explicitly supplied
//! evaluation file. Source groups are reported, not automatically split. Users
//! must separate evaluation groups before fitting to measure that transfer axis.
//! No evaluation label enters compilation or a session. The result's complete
//! `compiler.json` can be supplied to `grounded-session init`.
//!
//! This is a float lexical development interface, not D11 integer serving,
//! general temporal language or a geometric-advantage result.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::json;
use uor_r4_core::report_output;
use uor_r4_training::relation_compiler::SavedCompiler;
use uor_r4_training::sha256_file;
use uor_r4_training::temporal_compiler::{TemporalCompiler, TemporalExample, TemporalSettings};

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("temporal-compiler: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], keys: &[&str]) -> Result<Self, Error> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got {argument}"))?;
            if !keys.contains(&key) || pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("unknown or repeated argument {key}").into());
            }
        }
        Ok(Self(pairs))
    }

    fn path(&self, key: &str) -> Result<PathBuf, Error> {
        Ok(PathBuf::from(
            self.0.get(key).ok_or_else(|| format!("missing {key}="))?,
        ))
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, Error> {
        self.0.get(key).map_or(Ok(default), |value| {
            value
                .parse()
                .map_err(|_| format!("invalid {key}={value}").into())
        })
    }
}

fn rows(content: &[u8], path: &Path) -> Result<Vec<TemporalExample>, Error> {
    let content = std::str::from_utf8(content)?;
    let mut rows = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str(line)
                .map_err(|error| format!("{} row {}: {error}", path.display(), index + 1))?,
        );
    }
    if rows.is_empty() {
        return Err("labelled input has no rows".into());
    }
    Ok(rows)
}

fn run() -> Result<(), Error> {
    let started = Instant::now();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((mode, rest)) = arguments.split_first() else {
        return Err("usage: temporal-compiler fit|score key=value ...".into());
    };
    let keys: &[&str] = match mode.as_str() {
        "fit" => &["out", "base", "train", "steps", "rate", "l2"],
        "score" => &["out", "compiler", "data"],
        other => return Err(format!("unknown mode {other}").into()),
    };
    let args = Args::parse(rest, keys)?;
    let out = args.path("out")?;
    let data_path = args.path(if mode == "fit" { "train" } else { "data" })?;
    let compiler_path = args.path(if mode == "fit" { "base" } else { "compiler" })?;
    let settings = TemporalSettings {
        steps: args.number("steps", 300)?,
        rate: args.number("rate", 0.5)?,
        l2: args.number("l2", 0.001)?,
    };
    if mode == "fit"
        && (settings.steps == 0
            || !settings.rate.is_finite()
            || settings.rate <= 0.0
            || !settings.l2.is_finite()
            || settings.l2 < 0.0
            || !(settings.rate * settings.l2).is_finite()
            || settings.rate * settings.l2 > 1.0)
    {
        return Err("fit needs positive steps/rate and nonnegative finite l2".into());
    }
    // Claim exclusively after argument validation and before loading artifacts.
    report_output::claim(&out)?;
    let result = (|| -> Result<(), Error> {
        let data_bytes = fs::read(&data_path)?;
        let input = rows(&data_bytes, &data_path)?;
        let data_sha256 =
            uor_r4_core::native_geometric::learner::realtext_support::sha256_hex(&data_bytes);
        let groups: BTreeSet<&str> = input.iter().map(|row| row.source_group.as_str()).collect();
        let executable_sha256 = sha256_file(&std::env::current_exe()?)?;
        let source_revision = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE");
        let compiler = if mode == "fit" {
            TemporalCompiler::fit(
                SavedCompiler::from_bytes(fs::read(&compiler_path)?)?,
                &input,
                json!({
                    "file_sha256": data_sha256,
                    "file": data_path.display().to_string(),
                    "rows": input.len(),
                    "source_groups": groups,
                    "split": "caller-supplied training only; no automatic split",
                    "executable_sha256": executable_sha256,
                    "source_revision": source_revision,
                }),
                settings,
            )?
        } else {
            TemporalCompiler::from_bytes(fs::read(&compiler_path)?)?
        };
        let score = compiler.score(&input)?;
        if mode == "fit" {
            fs::write(out.join("compiler.json"), compiler.bytes())?;
        }
        fs::write(
            out.join("report.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.temporal-compiler-report/1",
                "mode": mode,
                "executable_sha256": executable_sha256,
                "source_revision": source_revision,
                "input_sha256": data_sha256,
                "input_file": data_path.display().to_string(),
                "source_groups": groups,
                "evaluation_scope": if mode == "fit" { "in-sample construction diagnostic" }
                    else { "caller-supplied rows; independence is not inferred" },
                "compiler_sha256": uor_r4_core::native_geometric::learner::realtext_support::sha256_hex(compiler.bytes()),
                "training": compiler.diagnostics(),
                "score": score,
                "wall_seconds": started.elapsed().as_secs_f64(),
            }))?,
        )?;
        Ok(())
    })();
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    println!(
        "{}",
        json!({"out": out.display().to_string(), "mode": mode})
    );
    Ok(())
}
