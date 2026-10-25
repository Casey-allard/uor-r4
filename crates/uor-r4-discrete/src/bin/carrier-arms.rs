//! The eight arms of `carrier-credit-hard-selection` (#2819; prep sheet §2, §5, §7).
//!
//! carrier-arms plan BASE_CONFIG SHEET OUT
//!     Host: each arm's lineage SHA-256 and the sheet's, for the D67 release.
//! carrier-arms run BASE_CONFIG SHEET RELEASE FRESH_DIR SCHEDULE SEED BATCHES OUT_DIR
//!     CUDA: train one released arm and score it on DEV and the fresh slice.
//! carrier-arms grade OUT ARM_DIR...
//!     Host: the selection audit over each seed's receipts, then the sheet's two
//!     comparisons over the eight stored arm records.
//! Every output path must not exist; outputs are written once.

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use uor_r4_discrete::carrier_arms::{
    arm_config, audit_selections, grade, ArmScore, SCHEDULES, SEEDS,
};
use uor_r4_discrete::corpus;
use uor_r4_discrete::training::{DriverConfig, PreparedDriver, SchedulePolicy, StepReceipt};

fn read(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{path}: {e}"))
}

fn write_once(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "{} exists; outputs are written once",
            path.display()
        ));
    }
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn base(path: &str) -> Result<DriverConfig, String> {
    serde_json::from_slice(&read(path)?).map_err(|e| format!("{path}: {e}"))
}

fn schedule(name: &str) -> Result<SchedulePolicy, String> {
    serde_json::from_str(&format!("\"{name}\"")).map_err(|e| format!("schedule {name}: {e}"))
}

fn plan(args: &[String]) -> Result<(), String> {
    let [config, sheet, out] = args else {
        return Err("usage: carrier-arms plan BASE_CONFIG SHEET OUT".into());
    };
    let base = base(config)?;
    let mut arms = Vec::new();
    for seed in SEEDS {
        for schedule in SCHEDULES {
            let config = arm_config(&base, schedule, seed).map_err(|e| format!("{e:?}"))?;
            let driver = PreparedDriver::load(config).map_err(|e| format!("{e:?}"))?;
            let lineage = driver.lineage_sha256().map_err(|e| format!("{e:?}"))?;
            arms.push(
                serde_json::json!({"schedule": schedule, "seed": seed, "lineage_sha256": lineage}),
            );
        }
    }
    let record = serde_json::json!({
        "schema": "uor-r4.carrier-arms-plan/1",
        "sheet_sha256": corpus::sha256(&read(sheet)?),
        "arms": arms,
    });
    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    write_once(Path::new(out), &bytes)?;
    println!("{record}");
    Ok(())
}

#[cfg(feature = "cuda")]
fn run(args: &[String]) -> Result<(), String> {
    use uor_r4_discrete::carrier_arms::{run_arm, CarrierRelease};
    use uor_r4_discrete::corpus::{CorpusSource, FilePin, PreparedCorpus, Split};
    let [config, sheet, release, fresh_dir, name, seed, batches, out] = args else {
        return Err(
            "usage: carrier-arms run BASE_CONFIG SHEET RELEASE FRESH_DIR SCHEDULE SEED BATCHES OUT_DIR"
                .into(),
        );
    };
    let seed: u32 = seed.parse().map_err(|e| format!("SEED: {e}"))?;
    let batches: u64 = batches.parse().map_err(|e| format!("BATCHES: {e}"))?;
    let out = Path::new(out);
    if out.exists() {
        return Err(format!("{} exists; an arm is written once", out.display()));
    }
    let config = arm_config(&base(config)?, schedule(name)?, seed).map_err(|e| format!("{e:?}"))?;
    let release: CarrierRelease =
        serde_json::from_slice(&read(release)?).map_err(|e| format!("release: {e}"))?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&read(&format!("{fresh_dir}/manifest.json"))?)
            .map_err(|e| format!("fresh manifest: {e}"))?;
    let field = |key: &str| -> Result<String, String> {
        manifest[key]
            .as_str()
            .map(str::to_owned)
            .ok_or(format!("fresh manifest lacks {key}"))
    };
    let count = |key: &str| -> Result<u64, String> {
        manifest[key]
            .as_u64()
            .ok_or(format!("fresh manifest lacks {key}"))
    };
    let source = CorpusSource {
        split: Split::Fresh,
        tokens: FilePin {
            path: format!("{fresh_dir}/tokens.uort").into(),
            sha256: field("fresh_tokens_sha256")?,
        },
        mask: FilePin {
            path: format!("{fresh_dir}/mask.u8").into(),
            sha256: field("fresh_mask_sha256")?,
        },
        expected_payload_tokens: count("fresh_payload_tokens")?,
        expected_eligible_targets: count("kept_targets")?,
    };
    let fresh = PreparedCorpus::load(&source, config.spec.vocabulary, &config.bos)
        .map_err(|e| format!("fresh: {e:?}"))?;
    let driver = PreparedDriver::load(config).map_err(|e| format!("{e:?}"))?;
    fs::create_dir(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut receipts = std::io::BufWriter::new(
        fs::File::create(out.join("receipts.jsonl")).map_err(|e| e.to_string())?,
    );
    let record = run_arm(
        &driver,
        &release,
        &read(sheet)?,
        &fresh,
        batches,
        0,
        &mut receipts,
    )
    .map_err(|e| format!("{e:?}"))?;
    // A short receipts file would starve the selection audit, so a flush or
    // sync failure is an error, not a silent drop.
    receipts
        .into_inner()
        .map_err(|e| format!("receipts.jsonl: {e}"))?
        .sync_all()
        .map_err(|e| format!("receipts.jsonl: {e}"))?;
    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    write_once(&out.join("record.json"), &bytes)?;
    println!("{record}");
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn run(_: &[String]) -> Result<(), String> {
    Err("carrier-arms run needs the cuda feature".into())
}

fn receipts(dir: &str) -> Result<Vec<StepReceipt>, String> {
    let text =
        String::from_utf8(read(&format!("{dir}/receipts.jsonl"))?).map_err(|e| e.to_string())?;
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(|e| format!("{dir}/receipts.jsonl: {e}")))
        .collect()
}

fn grade_records(args: &[String]) -> Result<(), String> {
    let (out, dirs) = args
        .split_first()
        .ok_or("usage: carrier-arms grade OUT ARM_DIR...")?;
    let mut scores = Vec::new();
    for dir in dirs {
        let path = format!("{dir}/record.json");
        let r: serde_json::Value =
            serde_json::from_slice(&read(&path)?).map_err(|e| format!("{path}: {e}"))?;
        if r["scored"] != serde_json::Value::Bool(true) {
            return Err(format!(
                "{path}: not a scored full-dose record (a probe scores nothing)"
            ));
        }
        let number = |v: &serde_json::Value| v.as_u64().ok_or(format!("{path}: missing count"));
        scores.push((
            dir.clone(),
            ArmScore {
                schedule: serde_json::from_value(r["schedule"].clone())
                    .map_err(|e| e.to_string())?,
                seed: u32::try_from(number(&r["seed"])?).map_err(|e| e.to_string())?,
                fresh_top1: number(&r["fresh"]["top1_correct"])?,
                fresh_rows: number(&r["fresh"]["scored_rows"])?,
                dev_top1: number(&r["dev"]["top1_correct"])?,
                dev_rows: number(&r["dev"]["scored_rows"])?,
            },
        ));
    }
    let dir_of = |schedule: SchedulePolicy, seed: u32| {
        scores
            .iter()
            .find(|(_, s)| s.schedule == schedule && s.seed == seed)
            .map(|(d, _)| d.clone())
            .ok_or(format!("no arm {schedule:?} seed {seed}"))
    };
    let mut audits = Vec::new();
    for seed in SEEDS {
        let on = receipts(&dir_of(SchedulePolicy::CarrierCreditV1, seed)?)?;
        let cu = receipts(&dir_of(SchedulePolicy::CarrierUniformTargetV1, seed)?)?;
        let cf = receipts(&dir_of(SchedulePolicy::CarrierUniformRowV1, seed)?)?;
        audits.push(audit_selections(seed, &on, &cu, &cf).map_err(|e| format!("{e:?}"))?);
    }
    let scores: Vec<ArmScore> = scores.into_iter().map(|(_, s)| s).collect();
    let verdict = grade(&scores, &audits).map_err(|e| format!("{e:?}"))?;
    let bytes = serde_json::to_vec_pretty(&verdict).map_err(|e| e.to_string())?;
    write_once(Path::new(out), &bytes)?;
    println!("{verdict}");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.split_first() {
        Some((command, rest)) if command == "plan" => plan(rest),
        Some((command, rest)) if command == "run" => run(rest),
        Some((command, rest)) if command == "grade" => grade_records(rest),
        _ => Err("usage: carrier-arms plan|run|grade ...".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("carrier-arms: {message}");
            ExitCode::FAILURE
        }
    }
}
