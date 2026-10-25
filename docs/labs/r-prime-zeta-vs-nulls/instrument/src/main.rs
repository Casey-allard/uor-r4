//! `pz-nulls-grade MANIFEST_JSON OUTPUT_JSON`
//!
//! The manifest names, for each arm in `ARMS` (`p8`, `q0`..`q2`, `r0`..`r2`),
//! its `attention-dev-b3` report (`dev_b3`), its `eval-corpus` report (`eval`)
//! and its `coverage-census --token-cells` report (`census`). Relative paths
//! are read from the manifest's folder. It refuses an existing output, and it
//! exits 2 on an instrument fault (no paired reading is made).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use r_prime_zeta_vs_nulls_instrument as inst;
use serde_json::Value;

fn read(path: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn load(manifest_path: &Path) -> Result<[inst::Arm; 7], String> {
    let manifest = read(manifest_path)?;
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    let report = |arm: &str, key: &str| -> Result<Value, String> {
        let rel = manifest[arm][key]
            .as_str()
            .ok_or_else(|| format!("manifest lacks {arm}.{key}"))?;
        read(&base.join(rel))
    };
    let mut arms = Vec::new();
    for (name, _) in inst::ARMS {
        let dev_b3 = report(name, "dev_b3")?;
        let eval = report(name, "eval")?;
        arms.push(inst::arm_from_reports(name, &dev_b3, &eval, report(name, "census")?)?);
    }
    arms.try_into().map_err(|_| "seven arms are required".to_string())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [manifest, output] = args.as_slice() else {
        eprintln!("usage: pz-nulls-grade MANIFEST_JSON OUTPUT_JSON");
        return ExitCode::from(1);
    };
    let output = PathBuf::from(output);
    if output.exists() {
        eprintln!("{} exists; refusing to overwrite a grade", output.display());
        return ExitCode::from(1);
    }
    let record = match load(Path::new(manifest)).and_then(|arms| inst::grade(&arms)) {
        Ok(record) => record,
        Err(e) => {
            eprintln!("refused: {e}");
            return ExitCode::from(1);
        }
    };
    let mut text = serde_json::to_string_pretty(&record).expect("a JSON value serializes");
    text.push('\n');
    if let Err(e) = std::fs::write(&output, text) {
        eprintln!("{}: {e}", output.display());
        return ExitCode::from(1);
    }
    println!("verdict: {}", record["verdict"].as_str().unwrap_or("?"));
    if record["verdict"] == "instrument fault" {
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
