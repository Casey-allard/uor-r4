//! Run the hash-pinned real-checkpoint ask-selection regression.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_cpu_train::ask_offset_regression::{
    run_ask_arm_regression, AskArmRegressionReport, ASK_ARM_REGRESSION_DEFAULT_WORKERS,
    ASK_ARM_REGRESSION_MAX_WORKERS,
};

type Result<T> = std::result::Result<T, String>;

fn main() {
    match run() {
        Ok(report) => {
            let source_failures = report.source_failures();
            let regression_failures = report.regression_failures();
            println!(
                "{}",
                serde_json::json!({
                    "schema": report.schema,
                    "passed": report.passed,
                    "arms": report.arms.len(),
                    "failures": report.failures(),
                    "source_failures": source_failures,
                    "regression_failures": regression_failures,
                    "workers": report.workers,
                    "manifest_sha256": report.manifest_sha256,
                })
            );
            if source_failures > 0 {
                std::process::exit(1);
            }
            if regression_failures > 0 {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("cpu-ask-arm-regression: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<AskArmRegressionReport> {
    let args = parse_args(std::env::args().skip(1))?;
    let manifest_path = Path::new(&args["manifest"]);
    let bytes = std::fs::read(manifest_path)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    let workers = args
        .get("workers")
        .map_or(Ok(ASK_ARM_REGRESSION_DEFAULT_WORKERS), |value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("--workers: {error}"))
        })?;
    if workers == 0 || workers > ASK_ARM_REGRESSION_MAX_WORKERS {
        return Err(format!(
            "--workers must be within 1..={ASK_ARM_REGRESSION_MAX_WORKERS}"
        ));
    }
    let out = PathBuf::from(&args["out"]);
    if out.exists() {
        return Err(format!(
            "{} exists; regression reports are never replaced",
            out.display()
        ));
    }
    let executable =
        std::env::current_exe().map_err(|error| format!("current executable: {error}"))?;
    let executable_bytes =
        std::fs::read(&executable).map_err(|error| format!("{}: {error}", executable.display()))?;
    let executable_sha256 = uor_r4_discrete::corpus::sha256(&executable_bytes);
    let report = run_ask_arm_regression(
        Path::new(&args["root"]),
        &bytes,
        workers,
        &executable_sha256,
    )?;
    let mut encoded =
        serde_json::to_vec_pretty(&report).map_err(|error| format!("report JSON: {error}"))?;
    encoded.push(b'\n');
    write_new(&out, &encoded)?;
    Ok(report)
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 4] = ["manifest", "root", "out", "workers"];
    const REQUIRED: [&str; 3] = ["manifest", "root", "out"];
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

    #[test]
    fn parse_requires_three_paths_and_accepts_workers() {
        let base = ["--manifest", "m", "--root", "r", "--out", "o"];
        assert_eq!(
            parse_args(base.into_iter().map(str::to_owned))
                .unwrap()
                .len(),
            3
        );
        assert!(parse_args(
            base.into_iter()
                .chain(["--workers", "4"])
                .map(str::to_owned)
        )
        .unwrap()
        .contains_key("workers"));
        assert!(parse_args(["--manifest", "m"].into_iter().map(str::to_owned)).is_err());
    }

    #[test]
    fn parse_refuses_unknown_duplicate_and_missing_values() {
        assert!(parse_args(
            [
                "--manifest",
                "m",
                "--root",
                "r",
                "--out",
                "o",
                "--extra",
                "x"
            ]
            .into_iter()
            .map(str::to_owned)
        )
        .is_err());
        assert!(parse_args(
            [
                "--manifest",
                "m",
                "--manifest",
                "m2",
                "--root",
                "r",
                "--out",
                "o"
            ]
            .into_iter()
            .map(str::to_owned)
        )
        .is_err());
        assert!(parse_args(
            ["--manifest", "m", "--root", "r", "--out"]
                .into_iter()
                .map(str::to_owned)
        )
        .is_err());
    }

    #[test]
    fn output_is_created_once_and_refuses_replacement() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-ask-arm-regression-output-{}",
            std::process::id()
        ));
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        write_new(&path, b"first\n").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"first\n");
        assert!(write_new(&path, b"second\n").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"first\n");
        std::fs::remove_file(path).unwrap();
    }
}
