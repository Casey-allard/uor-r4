//! Test-only matched causal-credit fit driver; retained source/artifacts unchanged.
use super::{
    artifact::Model, causal_credit::paired_batch, causal_learner::Checkpoint,
    circuit::PrimitiveExport, learner::sgd, pilot::document, pilot_data, policy::PARAMETER_COUNT,
};
use crate::report_output;
use serde::Serialize;
use std::{fs, path::Path, time::Instant};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn config() -> serde_json::Value {
    serde_json::json!({"schema":"uor-r4.addressed-causal-learning-pilot/1","credit":"causal suffix-LOO, independent other-particle baseline at same target boundary","additional_retention":"zero lost correct symbols or exact responses against both saved prior endpoints; original acceptance below unchanged","evaluation_source":blake3::hash(include_bytes!("causal_pilot_eval.rs")).to_hex().to_string(),"causal_source":blake3::hash(include_bytes!("causal_credit.rs")).to_hex().to_string(),"causal_checkpoint_source":hex::encode(super::causal_learner::implementation_digest()),"parameter_seed":7341,"event_seed":973,"max_updates":64,"rate":0.05,"optimizer":"plain SGD; no clipping/momentum/decay","particles":4,"window_max":64,"rng_document_id":"update index0..63","training_example":"update index modulo16","objective":"all prompt+response+terminalEOS positions; complete mean CE; causal downstream hard-event LOO","prefill":"full predict/discard-offer/observe, matching training","train_digest":hex::encode(pilot_data::corpus_digest(&pilot_data::training())),"development_digest":hex::encode(pilot_data::corpus_digest(&pilot_data::development())),"pilot_source":blake3::hash(include_bytes!("pilot.rs")).to_hex().to_string(),"data_source":blake3::hash(include_bytes!("pilot_data.rs")).to_hex().to_string(),"driver_source":blake3::hash(include_bytes!("causal_pilot.rs")).to_hex().to_string(),"controls":["Full","ReadDisabled: Null=query; scan retained, no exact reads","ExactPayloadMasked: bits776..800,804,805 zero; geometry/metadata/legal actions retained","StateTransportDisabled: Root4..7 replaced by identity after base choice; zeta remains"],"acceptance":{"dev_response_ce":"strictly lower than initialized Full","dev_correct_symbols":"strictly more","retention":"zero initialized-correct response symbols lost","generation":"at least one exact answer+EOS in each memory/coding family and one same-family changed-source pair both exact with different answers","controls":"all three strictly worse response CE and each loses at least one Full-exact response","generated_code_semantics":"at least one actual saved generated program executes with correct stdout; no exact code answer has compiler/runtime failure","promotion":false},"fit_limit_ms":40000,"fit_stop_scheduling_ms":36000,"generation_limit":64,"checkpoint_steps":[0,64],"final_holdout":"NOT_RUN; authored development only"})
}
pub(super) fn write_json(root: &Path, name: &str, value: &impl Serialize) -> TestResult {
    fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
pub(super) fn claim_output(root: &Path) -> TestResult {
    if !root.is_absolute()
        || root
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("report must be an absolute path without parent traversal".into());
    }
    let parent = root.parent().ok_or("report parent")?.canonicalize()?;
    for ancestor in parent.ancestors() {
        if ancestor.join("manifest.json").exists() {
            return Err("report output is beneath a sealed or manifested root".into());
        }
    }
    report_output::claim(root)?;
    Ok(())
}
pub(super) fn seal_result(root: &Path, result: TestResult) -> TestResult {
    if let Err(e) = &result {
        write_json(
            root,
            "incomplete.json",
            &serde_json::json!({"status":"INCOMPLETE_PILOT_ATTEMPT","error":e.to_string(),"no_automatic_retry":true}),
        )?;
    }
    report_output::seal(root)?;
    report_output::verify(root)?;
    result
}
pub(super) fn load_checkpoint(
    root: &Path,
    name: &str,
    expected_updates: u64,
) -> std::result::Result<Checkpoint, Box<dyn std::error::Error>> {
    let bytes = fs::read(root.join("config.json"))?;
    let cp = Checkpoint::decode(
        &fs::read(root.join(name))?,
        pilot_data::corpus_digest(&pilot_data::training()),
        *blake3::hash(&bytes).as_bytes(),
        973,
    )?;
    if cp.completed_updates != expected_updates || cp.parameters.seed() != 7341 {
        return Err("pilot checkpoint step/seed".into());
    }
    Ok(cp)
}
#[test]
#[ignore = "one capped pilot fit; exclusive report root and projected allowance required"]
fn causal_pilot_fit_report() -> TestResult {
    let path = std::env::var("UOR_CAUSAL_PILOT_REPORT")?;
    let parent = std::env::var("UOR_CAUSAL_PILOT_PARENT_FIT")?;
    let root = Path::new(&path);
    claim_output(root)?;
    let attempt_started = Instant::now();
    let result = (|| -> TestResult {
        pilot_data::validate()?;
        let old_fit = Path::new(&parent);
        report_output::verify(old_fit)?;
        let old_config_bytes = fs::read(old_fit.join("config.json"))?;
        let old_config: serde_json::Value = serde_json::from_slice(&old_config_bytes)?;
        let mut cfg = config();
        for key in [
            "parameter_seed",
            "event_seed",
            "max_updates",
            "rate",
            "particles",
            "window_max",
            "checkpoint_steps",
            "acceptance",
        ] {
            if cfg[key] != old_config[key] {
                return Err(format!("changed matched recipe field {key}").into());
            }
        }
        let old_train: Vec<pilot_data::Example> =
            serde_json::from_slice(&fs::read(old_fit.join("training.json"))?)?;
        let old_dev: Vec<pilot_data::Example> =
            serde_json::from_slice(&fs::read(old_fit.join("development.json"))?)?;
        if old_train != pilot_data::training() || old_dev != pilot_data::development() {
            return Err("old data/order changed".into());
        }
        cfg["parent_checkpoint_blake3"] =
            blake3::hash(&fs::read(old_fit.join("initial-parameters.bin"))?)
                .to_hex()
                .to_string()
                .into();
        cfg["parent_fit_config_blake3"] =
            blake3::hash(&old_config_bytes).to_hex().to_string().into();
        let cfg_bytes = serde_json::to_vec_pretty(&cfg)?;
        fs::write(root.join("config.json"), &cfg_bytes)?;
        let train = pilot_data::training();
        let dev = pilot_data::development();
        write_json(root, "training.json", &train)?;
        write_json(root, "development.json", &dev)?;
        let parent_bytes = fs::read(old_fit.join("parent-initialized-model.bin"))?;
        let model = Model::decode(&parent_bytes)?;
        let original = super::learner::Checkpoint::decode(
            &fs::read(old_fit.join("initial-parameters.bin"))?,
            pilot_data::corpus_digest(&train),
            *blake3::hash(&old_config_bytes).as_bytes(),
            973,
        )?;
        if original.completed_updates != 0 || original.parameters.seed() != 7341 {
            return Err("original initialization identity".into());
        }
        let mut params = original.parameters;
        let parent_checkpoint_digest = hex::encode(params.digest());
        if model.compiled() != &params.compile()?
            || model.provenance().parameter_digest != params.digest()
        {
            return Err("preserved initialized payload mismatch".into());
        }
        fs::write(root.join("parent-initialized-model.bin"), parent_bytes)?;
        let initial = Checkpoint::new(
            params.clone(),
            pilot_data::corpus_digest(&train),
            *blake3::hash(&cfg_bytes).as_bytes(),
            973,
            0,
        )?;
        fs::write(root.join("initial-parameters.bin"), initial.encode()?)?;
        let reloaded = load_checkpoint(root, "initial-parameters.bin", 0)?;
        if reloaded.parameters.digest() != params.digest() {
            return Err("initial checkpoint reload".into());
        }
        let before = params.digest();
        let initial_compiled = params.compile()?.encode();
        fs::write(root.join("initial-compiled.bin"), &initial_compiled)?;
        let timer = Instant::now();
        let mut steps = Vec::new();
        let mut updates = 0;
        let mut loop_error: Option<String> = None;
        for update in 0..64 {
            if attempt_started.elapsed().as_millis() >= 36000 {
                break;
            }
            let example = &train[update % train.len()];
            let draw = match paired_batch(
                &params,
                model.geometry(),
                &document(example),
                example.prompt.len(),
                973,
                update as u64,
            ) {
                Ok(v) => v,
                Err(e) => {
                    loop_error = Some(e.to_string());
                    break;
                }
            };
            let mut norms = [0.0f64; 2];
            for (i, &v) in draw.causal_gradient.iter().enumerate() {
                norms[usize::from(i >= super::circuit::GATE_LOGITS)] += v * v;
            }
            let next = match sgd(&params, &draw.causal_gradient, 0.05) {
                Ok(v) => v,
                Err(e) => {
                    loop_error = Some(e.to_string());
                    break;
                }
            };
            steps.push(serde_json::json!({"update":update+1,"example_id":example.id,"mean_ce":draw.mean_ce,"batch_us":draw.elapsed_us,"tape_bytes":draw.tape_bytes,"counts":draw.counts,"gate_gradient_l2":libm::sqrt(norms[0]),"head_gradient_l2":libm::sqrt(norms[1]),"before":hex::encode(params.digest()),"after":hex::encode(next.digest())}));
            params = next;
            updates += 1;
        }
        let fit_us = timer.elapsed().as_micros();
        let final_cp = Checkpoint::new(
            params.clone(),
            pilot_data::corpus_digest(&train),
            *blake3::hash(&cfg_bytes).as_bytes(),
            973,
            updates,
        )?;
        fs::write(root.join("final-parameters.bin"), final_cp.encode()?)?;
        let restored = load_checkpoint(root, "final-parameters.bin", updates)?;
        if restored.parameters.digest() != params.digest() {
            return Err("final checkpoint reload".into());
        }
        let compiled = params.compile()?;
        let encoded = compiled.encode();
        fs::write(root.join("final-compiled.bin"), &encoded)?;
        if PrimitiveExport::decode(&fs::read(root.join("final-compiled.bin"))?)? != compiled {
            return Err("final actual export reload".into());
        }
        let changed = initial_compiled
            .iter()
            .zip(&encoded)
            .filter(|(a, b)| a != b)
            .count();
        let deltas: Vec<f64> = params
            .values()
            .iter()
            .zip(initial.parameters.values())
            .map(|(a, b)| a - b)
            .collect();
        let delta_l2 = libm::sqrt(deltas.iter().map(|v| v * v).sum::<f64>());
        write_json(root, "steps.json", &steps)?;
        write_json(
            root,
            "fit-summary.json",
            &serde_json::json!({"status":if updates==64{"COMPLETED_64_UPDATE_CAUSAL_PILOT_FIT"}else{"STOPPED_AT_FIT_LIMIT"},"updates":updates,"fit_us":fit_us,"attempt_us":attempt_started.elapsed().as_micros(),"checkpoint_stop_scheduling_ms":36000,"loop_error":loop_error,"parameter_count":PARAMETER_COUNT,"initial_digest":hex::encode(before),"final_digest":hex::encode(params.digest()),"changed_compiled_bytes":changed,"parameter_delta_l2":delta_l2,"finite":params.values().iter().all(|x|x.is_finite()),"normal_trained_model_artifact":"NOT_EXPORTED; checkpoint plus source-bound primitive witness","parent_model_id":hex::encode(model.id()),"geometry_digest":hex::encode(model.geometry().identity_digest()),"config_digest":hex::encode(blake3::hash(&cfg_bytes).as_bytes()),"fit_calls":1,"runtime_replays":0,"matched_old_fit_root":parent,"parent_checkpoint_digest":parent_checkpoint_digest,"causal_checkpoint_identity":hex::encode(super::causal_learner::implementation_digest()),"promotion":false}),
        )?;
        println!("pilot fit updates={updates}, fit_us={fit_us}, changed_compiled_bytes={changed}");
        if let Some(e) = loop_error {
            return Err(e.into());
        }
        Ok(())
    })();
    seal_result(root, result)
}
