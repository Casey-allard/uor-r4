//! Conditional block calibration and one separately admitted joint fit.
//! Uses the frozen successor design; never upgrades or repeats old calibration.
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};
use uor_r4_core::{
    native_geometric::shared_core::{FitConfig, Intervention, SharedCore},
    report_output,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn write(out: &Path, name: &str, value: &impl serde::Serialize) -> Result<()> {
    bytes(out, name, &serde_json::to_vec_pretty(value)?)
}
fn bytes(out: &Path, name: &str, data: &[u8]) -> Result<()> {
    fs::File::create_new(out.join(name))?.write_all(data)?;
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
fn load(input: &Path, name: &str) -> Result<SharedCore> {
    let raw = fs::read(input.join(name))?;
    let model = SharedCore::from_bytes(&raw)?;
    if model.to_bytes()? != raw {
        return Err("Preserved artifact byte identity mismatch".into());
    }
    Ok(model)
}
fn unchanged(input: &Path, name: &str, model: &SharedCore) -> Result<()> {
    if model.to_bytes()? != fs::read(input.join(name))? {
        return Err("Source artifact changed during execution".into());
    }
    Ok(())
}
fn validate_design(design: &Value) -> Result<FitConfig> {
    let fit: FitConfig = serde_json::from_value(design["fit"].clone())?;
    let gate = &design["learning_gate"];
    let representation = &design["representation_gate"];
    if design["schema"] != "uor-r4.block-calibration-experiment/1"
        || fit.seed != 443
        || fit.max_proposals != 100000
        || fit.max_seconds != 120
        || design["block_calibration"]["max_seconds"] != 120
        || design["block_calibration"]["all_observed_nodes"] != true
        || design["block_calibration"]["selection_uses_error_gate"] != false
        || design["generation_limit"] != 96
        || gate["nll_vs_legacy_fraction"].as_f64() != Some(0.98)
        || gate["nll_vs_calibrated_fraction"].as_f64() != Some(0.98)
        || gate["byte_accuracy_not_lower_than_either"] != true
        || gate["context_disabled_nll_fraction"].as_f64() != Some(1.01)
        || gate["state_disabled_nll_fraction"].as_f64() != Some(1.01)
        || representation["root_positions"] != 672
        || representation["ascii_positions"] != 660
        || representation["root_errors_at_most"] != 12
        || representation["ascii_errors_less_than"] != 216
        || design["prompts"].as_array().is_none_or(|v| v.len() != 4)
    {
        return Err("Frozen successor design differs from authorized bounds/gates".into());
    }
    Ok(fit)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || !["calibrate", "learn"].contains(&args[1].as_str()) {
        return Err(
            "usage: native_block_calibration calibrate|learn SEALED_INPUT NEW_OUTPUT DESIGN_JSON"
                .into(),
        );
    }
    let input = fs::canonicalize(&args[2])?;
    let raw = Path::new(&args[3]);
    let out = if raw.exists() {
        fs::canonicalize(raw)?
    } else {
        fs::canonicalize(raw.parent().ok_or("output parent")?)?
            .join(raw.file_name().ok_or("output directory name")?)
    };
    if out.starts_with(&input) {
        return Err("Output must be outside sealed input".into());
    }
    let design_path = fs::canonicalize(&args[4])?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        report_output::verify(&input)?;
        if fs::metadata(&design_path)?.len() > 128 * 1024 {
            return Err("Design byte bound".into());
        }
        let design_bytes = fs::read(&design_path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        let config = validate_design(&design)?;
        bytes(&out, "design.json", &design_bytes)?;
        write(
            &out,
            "design-source.json",
            &json!({"path":design_path,
            "blake3":format!("blake3:{}",blake3::hash(&design_bytes))}),
        )?;
        if args[1] == "calibrate" {
            calibrate(&input, &out, &design)?;
        } else {
            if fs::read(input.join("design.json"))? != design_bytes {
                return Err("Learning design differs from admitted calibration design".into());
            }
            learn(&input, &out, &design, config)?;
        }
        if fs::read(&design_path)? != design_bytes {
            return Err("Frozen design changed".into());
        }
        report_output::verify(&input)?;
        Ok(())
    })();
    if let Err(error) = &result {
        write(&out, "failure.json", &json!({"error":error.to_string()}))?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}
fn calibrate(input: &Path, out: &Path, design: &Value) -> Result<()> {
    let admission = read(&input.join("result.json"))?;
    if admission["decision"] != "FAIL_CALIBRATION_DOSE_NO_JOINT_FIT"
        || admission["joint_fit"] != "NOT_RUN"
        || admission["fresh_evaluation"] != "NOT_RUN"
    {
        return Err("Expected the preserved failed three-pass calibration".into());
    }
    let old_design = read(&input.join("design.json"))?;
    for key in ["training", "holdout", "prompts"] {
        if old_design[key] != design[key] {
            return Err("Original frozen population changed".into());
        }
    }
    let legacy = load(input, "legacy.json")?;
    let upgraded = load(input, "upgraded.json")?;
    let previous = load(input, "calibrated.json")?;
    if admission["legacy"] != legacy.artifact_cid()
        || admission["upgraded"] != upgraded.artifact_cid()
        || admission["calibrated"] != previous.artifact_cid()
        || design["parent"] != previous.artifact_cid()
    {
        return Err("Parent report/design artifact bindings disagree".into());
    }
    bytes(out, "legacy.json", &legacy.to_bytes()?)?;
    bytes(out, "previous.json", &previous.to_bytes()?)?;
    let train = data(&design["training"])?;
    let (calibrated, report) = previous.calibrate_emission_blocks(&train, 120)?;
    bytes(out, "calibrated.json", &calibrated.to_bytes()?)?;
    write(out, "calibration.json", &report)?;
    let gate = report.root_positions == 672
        && report.ascii_positions == 660
        && report.root_branch_errors <= 12
        && report.ascii_branch_errors < 216;
    unchanged(input, "legacy.json", &legacy)?;
    unchanged(input, "upgraded.json", &upgraded)?;
    unchanged(input, "calibrated.json", &previous)?;
    write(
        out,
        "result.json",
        &json!({
        "decision":if gate {"PASS_BLOCK_CALIBRATION"} else {"FAIL_BLOCK_CALIBRATION_NO_JOINT_FIT"},
        "source_attempt":input,"legacy":legacy.artifact_cid(),"previous":previous.artifact_cid(),
        "original_upgraded":upgraded.artifact_cid(),"calibrated":calibrated.artifact_cid(),
        "parent_bytes_preserved":true,"joint_fit":"NOT_RUN","fresh_evaluation":"NOT_RUN",
        "promoted":false,"selection_objective":"Whole occupied-node blocks minimized by logistic NLL; error gates evaluated after calibration."}),
    )?;
    Ok(())
}
fn learn(input: &Path, out: &Path, design: &Value, config: FitConfig) -> Result<()> {
    let admission = read(&input.join("result.json"))?;
    if admission["decision"] != "PASS_BLOCK_CALIBRATION"
        || admission["joint_fit"] != "NOT_RUN"
        || admission["fresh_evaluation"] != "NOT_RUN"
    {
        return Err("Block calibration gate did not authorize a joint fit".into());
    }
    let legacy = load(input, "legacy.json")?;
    let previous = load(input, "previous.json")?;
    let calibrated = load(input, "calibrated.json")?;
    if admission["legacy"] != legacy.artifact_cid()
        || admission["previous"] != previous.artifact_cid()
        || admission["calibrated"] != calibrated.artifact_cid()
        || design["parent"] != previous.artifact_cid()
    {
        return Err("Admitted block calibration lineage disagrees".into());
    }
    let train = data(&design["training"])?;
    let (candidate, fit) = calibrated.fit(&train, config)?;
    bytes(out, "candidate.json", &candidate.to_bytes()?)?;
    write(out, "fit.json", &fit)?;
    let fresh = data(&design["holdout"])?;
    let prior = legacy.evaluate(&fresh, Intervention::Full)?;
    let failed = previous.evaluate(&fresh, Intervention::Full)?;
    let block = calibrated.evaluate(&fresh, Intervention::Full)?;
    let full = candidate.evaluate(&fresh, Intervention::Full)?;
    let context = candidate.evaluate(&fresh, Intervention::ContextDisabled)?;
    let state = candidate.evaluate(&fresh, Intervention::StateDisabled)?;
    let zeta = candidate.evaluate(&fresh, Intervention::ZetaDisabled)?;
    let transport = candidate.evaluate(&fresh, Intervention::TransportDisabled)?;
    let pass = full.mean_nll < prior.mean_nll * 0.98
        && full.mean_nll < block.mean_nll * 0.98
        && full.correct >= prior.correct
        && full.correct >= block.correct
        && context.mean_nll > full.mean_nll * 1.01
        && state.mean_nll > full.mean_nll * 1.01;
    let mut generations = Vec::new();
    for (label, model) in [
        ("legacy", &legacy),
        ("previous", &previous),
        ("block_calibrated", &calibrated),
        ("candidate", &candidate),
    ] {
        for prompt in design["prompts"].as_array().ok_or("prompts")? {
            let prompt = prompt.as_str().ok_or("prompt")?;
            for control in [
                Intervention::Full,
                Intervention::ContextDisabled,
                Intervention::StateDisabled,
            ] {
                let (output, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
                generations.push(json!({"model":label,"prompt":prompt,"control":control,
                    "bytes":output,"utf8_lossy":String::from_utf8_lossy(&output),"eos":eos,"work":work}));
            }
        }
    }
    write(out, "generation.json", &generations)?;
    unchanged(input, "legacy.json", &legacy)?;
    unchanged(input, "previous.json", &previous)?;
    unchanged(input, "calibrated.json", &calibrated)?;
    write(
        out,
        "result.json",
        &json!({
        "decision":if pass {"PASS_PREDICTIVE_SMOKE_ONLY"} else {"FAIL_CONTEXTUAL_TRANSFER_SMOKE"},
        "source_attempt":input,"candidate":candidate.artifact_cid(),
        "block_parent":calibrated.artifact_cid(),"previous_artifact":previous.artifact_cid(),
        "legacy_artifact":legacy.artifact_cid(),"legacy":prior,"previous":failed,
        "block_calibrated":block,"full":full,"context_disabled":context,"state_disabled":state,
        "zeta_disabled":zeta,"transport_disabled":transport,"promoted":false,
        "parent_bytes_preserved":true,"language_qualification":"NOT_ESTABLISHED",
        "generation_review":"REQUIRED_SEPARATELY",
        "scope":"One new joint fit; frozen authored fresh comparison. No retained-model replacement, automatic promotion or repeat."}),
    )?;
    Ok(())
}
