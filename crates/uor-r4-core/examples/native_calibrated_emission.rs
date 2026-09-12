//! Two exclusive phases: development representability, then one frozen fit.
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};
use uor_r4_core::{
    native_geometric::shared_core::{FitConfig, Intervention, SharedCore},
    report_output,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn write(out: &Path, name: &str, value: &impl serde::Serialize) -> Result<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn artifact(out: &Path, name: &str, m: &SharedCore) -> Result<()> {
    fs::File::create_new(out.join(name))?.write_all(&m.to_bytes()?)?;
    Ok(())
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn data(value: &Value) -> Result<Vec<Vec<u8>>> {
    value
        .as_array()
        .ok_or("documents")?
        .iter()
        .map(|v| Ok(v.as_str().ok_or("document")?.as_bytes().to_vec()))
        .collect()
}
fn model(input: &Path, name: &str) -> Result<SharedCore> {
    Ok(SharedCore::from_bytes(&fs::read(input.join(name))?)?)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 || !["represent", "learn"].contains(&args[1].as_str()) {
        return Err(
            "usage: native_calibrated_emission represent|learn SEALED_INPUT NEW_OUTPUT".into(),
        );
    }
    let input = fs::canonicalize(&args[2])?;
    let raw = Path::new(&args[3]);
    let out = if raw.exists() {
        fs::canonicalize(raw)?
    } else {
        fs::canonicalize(raw.parent().ok_or("output parent")?)?
            .join(raw.file_name().ok_or("output name")?)
    };
    if out.starts_with(&input) {
        return Err("Output must be outside sealed input".into());
    }
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        report_output::verify(&input)?;
        if args[1] == "represent" {
            represent(&input, &out)?;
        } else {
            learn(&input, &out)?;
        }
        report_output::verify(&input)?;
        Ok(())
    })();
    if let Err(e) = &result {
        write(&out, "failure.json", &json!({"error":e.to_string()}))?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
fn represent(input: &Path, out: &Path) -> Result<()> {
    let old = read(&input.join("design.json"))?;
    // New acceptance is frozen before calibration/model loading; never used by it.
    let design = json!({"schema":"uor-r4.calibrated-emission-experiment/1","training":old["training"],
        "holdout":[
            "A small red boat rested on the dry soil. Rain made the boat wet.\n",
            "Ben lives in Oslo. Ben now lives in Lima. The blue book is in Lima.\n",
            "User: Tell me about the garden.\nAssistant: The garden is calm.\n",
            "The bird flew across the lake. The red cup was on the branch.\n",
            "fn fourth(b: i32) -> i32 { b + 2 }\nlet total = 4;\n",
            "User: Where is the small boat?\nAssistant: The boat is on the lake.\n"],
        "prompts":["The red boat ","Ben now lives in Lima. Ben lives in ","User: Tell me about the garden.\nAssistant:","fn fourth(b: i32) -> i32 { "],
        "calibration_passes":3,"fit":{"seed":443,"max_proposals":100000,"max_seconds":120},"generation_limit":96,
        "representation_gate":{"root_positions":672,"ascii_positions":660,"root_errors_at_most":12,"ascii_errors_less_than":216},
        "learning_gate":{"nll_vs_legacy_fraction":0.98,"nll_vs_calibrated_fraction":0.98,"byte_accuracy_not_lower_than_either":true,"context_disabled_nll_fraction":1.01,"state_disabled_nll_fraction":1.01},
        "scope":"Authored byte-corpus smoke only. Three fixed-state calibration passes; at most one joint fit. Useful generation required separately. No promotion or fresh retry."});
    write(out, "design.json", &design)?;
    let legacy = model(input, "candidate.json")?;
    if legacy.to_bytes()? != fs::read(input.join("candidate.json"))? {
        return Err("Legacy identity mismatch".into());
    }
    let train = data(&design["training"])?;
    let expected = read(&input.join("fit.json"))?;
    let replay = legacy.evaluate(&train, Intervention::Full)?;
    if replay.correct as u64 != expected["after"]["correct"].as_u64().ok_or("prior count")?
        || (replay.mean_nll - expected["after"]["mean_nll"].as_f64().ok_or("prior loss")?).abs()
            > 1e-10
    {
        return Err("Legacy construction changed".into());
    }
    let previous_holdout = legacy.evaluate(&data(&old["holdout"])?, Intervention::Full)?;
    let previous = read(&input.join("result.json"))?;
    if previous_holdout.correct as u64
        != previous["full"]["correct"].as_u64().ok_or("prior count")?
        || (previous_holdout.mean_nll
            - previous["full"]["mean_nll"].as_f64().ok_or("prior loss")?)
        .abs()
            > 1e-10
    {
        return Err("Legacy holdout changed".into());
    }
    let upgraded = legacy.upgrade_calibrated()?;
    let (calibrated, report) = upgraded.calibrate_emission(&train)?;
    let gate = report.root_positions == 672
        && report.ascii_positions == 660
        && report.root_branch_errors <= 12
        && report.ascii_branch_errors < 216;
    artifact(out, "legacy.json", &legacy)?;
    artifact(out, "upgraded.json", &upgraded)?;
    artifact(out, "calibrated.json", &calibrated)?;
    write(out, "calibration.json", &report)?;
    write(
        out,
        "result.json",
        &json!({"decision":if gate{"PASS_DEVELOPMENT_REPRESENTATION"}else{"FAIL_CALIBRATION_DOSE_NO_JOINT_FIT"},
        "source_attempt":input,"legacy":legacy.artifact_cid(),"upgraded":upgraded.artifact_cid(),"calibrated":calibrated.artifact_cid(),
        "legacy_metrics_reproduced":true,"promoted":false,"fresh_evaluation":"NOT_RUN","joint_fit":"NOT_RUN"}),
    )?;
    Ok(())
}
fn learn(input: &Path, out: &Path) -> Result<()> {
    let admission = read(&input.join("result.json"))?;
    if admission["decision"] != "PASS_DEVELOPMENT_REPRESENTATION" {
        return Err("Representation gate failed; joint fit prohibited".into());
    }
    let design = read(&input.join("design.json"))?;
    write(out, "design.json", &design)?;
    let config: FitConfig = serde_json::from_value(design["fit"].clone())?;
    let train = data(&design["training"])?;
    let fresh = data(&design["holdout"])?;
    let legacy = model(input, "legacy.json")?;
    let upgraded = model(input, "upgraded.json")?;
    let calibrated = model(input, "calibrated.json")?;
    let (candidate, fit) = calibrated.fit(&train, config)?;
    artifact(out, "candidate.json", &candidate)?;
    write(out, "fit.json", &fit)?;
    let prior = legacy.evaluate(&fresh, Intervention::Full)?;
    let uncal = upgraded.evaluate(&fresh, Intervention::Full)?;
    let calibration = calibrated.evaluate(&fresh, Intervention::Full)?;
    let full = candidate.evaluate(&fresh, Intervention::Full)?;
    let context = candidate.evaluate(&fresh, Intervention::ContextDisabled)?;
    let state = candidate.evaluate(&fresh, Intervention::StateDisabled)?;
    let zeta = candidate.evaluate(&fresh, Intervention::ZetaDisabled)?;
    let transport = candidate.evaluate(&fresh, Intervention::TransportDisabled)?;
    let pass = full.mean_nll < prior.mean_nll * 0.98
        && full.mean_nll < calibration.mean_nll * 0.98
        && full.correct >= prior.correct
        && full.correct >= calibration.correct
        && context.mean_nll > full.mean_nll * 1.01
        && state.mean_nll > full.mean_nll * 1.01;
    let mut generations = Vec::new();
    for (label, m) in [
        ("legacy", &legacy),
        ("upgraded", &upgraded),
        ("calibrated", &calibrated),
        ("candidate", &candidate),
    ] {
        for prompt in design["prompts"].as_array().ok_or("prompts")? {
            let prompt = prompt.as_str().ok_or("prompt")?;
            for control in [
                Intervention::Full,
                Intervention::ContextDisabled,
                Intervention::StateDisabled,
            ] {
                let (bytes, eos, work) = m.generate(prompt.as_bytes(), 96, control)?;
                generations.push(json!({"model":label,"prompt":prompt,"control":control,"bytes":bytes,"utf8_lossy":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
            }
        }
    }
    write(out, "generation.json", &generations)?;
    write(
        out,
        "result.json",
        &json!({"decision":if pass{"PASS_PREDICTIVE_SMOKE_ONLY"}else{"FAIL_CONTEXTUAL_TRANSFER_SMOKE"},
        "promoted":false,"candidate":candidate.artifact_cid(),"calibrated":calibrated.artifact_cid(),"source_attempt":input,
        "legacy":prior,"upgraded":uncal,"calibration":calibration,"full":full,"context_disabled":context,"state_disabled":state,"zeta_disabled":zeta,"transport_disabled":transport,
        "language_qualification":"NOT_ESTABLISHED","scope":"Fresh authored smoke; no repeat, retained-model comparison or serving promotion. Generation assessed separately."}),
    )?;
    Ok(())
}
