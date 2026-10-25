//! Grade the address-metric pre-flight (#2834) from stored GPU outputs (D24).
//!
//! Usage: address-key-grade MANIFEST_JSON OUTPUT_JSON
//!
//! The manifest names, for each K8 arm (`a8`, `fv8_env`, `fv8_two_scale`,
//! `fvs8_env`, `fvs8_two_scale`), its `attention-dev-b3` report (`dev_b3`) and
//! its `eval-corpus` report (`eval`); and `fv_census` and `fvs_census`, the two
//! tables' `coverage-census --token-cells` reports. Each treatment is graded
//! against A8-Outer and against the shuffled arm with the same metric.
use serde_json::{json, Value};
use std::fs;
use uor_r4_discrete::address_key_grade::{grade, Arm, ARM_CONFIGS};

/// The pod log line that G-Q passed (sheet Bar 2).
const G_Q_PASS: &str = "device_admission_matches_the_host_oracles_for_each_metric ... ok";

fn read(path: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn field<'a>(manifest: &'a Value, arm: &str, key: &str) -> Result<&'a str, String> {
    manifest[arm][key]
        .as_str()
        .ok_or_else(|| format!("manifest lacks {arm}.{key}"))
}

fn arm(manifest: &Value, name: &str) -> Result<Arm, String> {
    let dev_b3 = read(field(manifest, name, "dev_b3")?)?;
    if dev_b3["schema"] != "uor-r4.attention-preflight-dev-b3/1" || dev_b3["updates_applied"] != 0 {
        return Err(format!("{name}: not a zero-update attention-dev-b3 report"));
    }
    // The report must be this arm's: it records the config bytes it read.
    let pinned = ARM_CONFIGS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, sha)| *sha)
        .ok_or_else(|| format!("{name}: not a registered arm"))?;
    if dev_b3["config_file"]["sha256"] != pinned {
        return Err(format!("{name}: its dev-b3 report read another config"));
    }
    let b3 = serde_json::from_value(dev_b3["b3"].clone()).map_err(|e| format!("{name} b3: {e}"))?;
    let eval = read(field(manifest, name, "eval")?)?;
    if eval["schema"] != "uor-r4.discrete-corpus-driver/1"
        || eval["updates_applied_by_evaluation"] != 0
    {
        return Err(format!("{name}: not a zero-update eval-corpus report"));
    }
    let top1 = eval["aggregate"]["top1_correct"]
        .as_u64()
        .ok_or_else(|| format!("{name}: eval report lacks aggregate.top1_correct"))?;
    Ok(Arm { b3, top1 })
}

fn run(args: &[String]) -> Result<(), String> {
    let [_, manifest_path, output] = args else {
        return Err("usage: address-key-grade MANIFEST_JSON OUTPUT_JSON".into());
    };
    let manifest = read(manifest_path)?;
    let g_q = manifest["g_q"].as_str().ok_or("manifest lacks g_q")?;
    let log = fs::read_to_string(g_q).map_err(|e| format!("{g_q}: {e}"))?;
    if !log.contains(G_Q_PASS) {
        return Err(format!("{g_q}: G-Q did not pass; nothing is graded"));
    }
    let a8 = arm(&manifest, "a8")?;
    let fv_census = read(
        manifest["fv_census"]
            .as_str()
            .ok_or("manifest lacks fv_census")?,
    )?;
    let fvs_census = read(
        manifest["fvs_census"]
            .as_str()
            .ok_or("manifest lacks fvs_census")?,
    )?;
    let env = grade(
        &a8,
        &arm(&manifest, "fv8_env")?,
        &arm(&manifest, "fvs8_env")?,
        &fv_census,
        &fvs_census,
        "FV8-Env",
    )
    .map_err(|e| e.to_string())?;
    let two = grade(
        &a8,
        &arm(&manifest, "fv8_two_scale")?,
        &arm(&manifest, "fvs8_two_scale")?,
        &fv_census,
        &fvs_census,
        "FV8-TwoScale",
    )
    .map_err(|e| e.to_string())?;
    let outcomes = [env["outcome"].clone(), two["outcome"].clone()];
    // The sheet's outcome: an instrument fault in either decides first, then
    // a PASS by either treatment, then a PARTIAL, else FAIL.
    let overall = ["INSTRUMENT", "PASS", "PARTIAL"]
        .into_iter()
        .find(|o| outcomes.iter().any(|x| x == o))
        .unwrap_or("FAIL");
    let record = json!({
        "schema": "uor-r4.address-key-grade-run/1",
        "manifest": manifest,
        "treatments": [env, two],
        "outcome": overall,
    });
    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(output, bytes).map_err(|e| format!("{output}: {e}"))?;
    println!("{overall}");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(error) = run(&args) {
        eprintln!("address-key-grade: {error}");
        std::process::exit(1);
    }
}
