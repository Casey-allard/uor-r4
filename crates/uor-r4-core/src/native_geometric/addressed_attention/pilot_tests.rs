use super::super::{
    artifact::Model,
    learner::{batch, sgd, Checkpoint},
    pilot_data,
    policy::{InterpretedPolicy, PARAMETER_COUNT},
};
use super::*;
use crate::report_output;
use std::{fs, path::Path, time::Instant};
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn config() -> serde_json::Value {
    serde_json::json!({"schema":"uor-r4.addressed-learning-pilot/1","parameter_seed":7341,"event_seed":973,"max_updates":64,"rate":0.05,"optimizer":"plain SGD; no clipping/momentum/decay","particles":4,"window_max":64,"rng_document_id":"update index0..63","training_example":"update index modulo16","objective":"all prompt+response+terminalEOS positions; complete mean CE plus all hard-event LOO","prefill":"full predict/discard-offer/observe, matching training","train_digest":hex::encode(pilot_data::corpus_digest(&pilot_data::training())),"development_digest":hex::encode(pilot_data::corpus_digest(&pilot_data::development())),"pilot_source":blake3::hash(include_bytes!("pilot.rs")).to_hex().to_string(),"data_source":blake3::hash(include_bytes!("pilot_data.rs")).to_hex().to_string(),"driver_source":blake3::hash(include_bytes!("pilot_tests.rs")).to_hex().to_string(),"controls":["Full","ReadDisabled: Null=query; scan retained, no exact reads","ExactPayloadMasked: bits776..800,804,805 zero; geometry/metadata/legal actions retained","StateTransportDisabled: Root4..7 replaced by identity after base choice; zeta remains"],"acceptance":{"dev_response_ce":"strictly lower than initialized Full","dev_correct_symbols":"strictly more","retention":"zero initialized-correct response symbols lost","generation":"at least one exact answer+EOS in each memory/coding family and one same-family changed-source pair both exact with different answers","controls":"all three strictly worse response CE and each loses at least one Full-exact response","generated_code_semantics":"at least one actual saved generated program executes with correct stdout; no exact code answer has compiler/runtime failure","promotion":false},"fit_limit_ms":40000,"fit_stop_scheduling_ms":36000,"generation_limit":64,"checkpoint_steps":[0,64],"final_holdout":"NOT_RUN; authored development only"})
}
fn write_json(root: &Path, name: &str, value: &impl Serialize) -> TestResult {
    fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn claim_output(root: &Path) -> TestResult {
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
fn seal_result(root: &Path, result: TestResult) -> TestResult {
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
fn load_checkpoint(
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
fn controls_disable_reads_mask_only_declared_fields_and_preserve_export_interpretation(
) -> TestResult {
    let params = Parameters::seeded(7341)?;
    let compiled = params.compile()?;
    let geo = BoundGeometry::canonical()?;
    let mut session = RuntimeSession::new(params.digest(), &geo, 1)?;
    let mut p = Controlled::new(&compiled, Control::ReadDisabled, geo.identity());
    for &byte in b"a=19;b=7;" {
        let offer = session.predict(&geo, &mut p)?;
        assert!(offer.trace.selected.iter().all(Option::is_none));
        session.observe(Symbol::Byte(byte), offer.offer.id, &geo, &mut p)?;
    }
    let mut a = Controlled::new(&compiled, Control::ExactPayloadMasked, geo.identity());
    let mut b = Controlled::new(&compiled, Control::Full, geo.identity());
    let input = [true; 1024];
    let mut masked = input;
    masked[776..800].fill(false);
    masked[804] = false;
    masked[805] = false;
    assert_eq!(
        a.context(Phase::Emit, &input)?,
        b.context(Phase::Emit, &masked)?
    );
    let mut s = RuntimeSession::new(params.digest(), &geo, 2)?;
    let mut p = Controlled::new(&compiled, Control::StateTransportDisabled, geo.identity());
    for &byte in b"xyz" {
        let o = s.predict(&geo, &mut p)?;
        s.observe(Symbol::Byte(byte), o.offer.id, &geo, &mut p)?;
        assert_eq!(s.roots(), [geo.identity(); 4]);
    }
    let mut a = RuntimeSession::new(params.digest(), &geo, 3)?;
    let mut b = a.clone();
    let mut cp = CompiledPolicy {
        compiled: &compiled,
    };
    let mut ip = InterpretedPolicy {
        parameters: &params,
    };
    for target in [65u16, 66, 256] {
        let x = a.predict(&geo, &mut cp)?;
        let y = b.predict(&geo, &mut ip)?;
        assert_eq!(x, y);
        assert_eq!(
            a.observe(sym(target)?, x.offer.id, &geo, &mut cp)?,
            b.observe(sym(target)?, y.offer.id, &geo, &mut ip)?
        );
    }
    Ok(())
}
#[test]
#[ignore = "one capped pilot fit; exclusive report root and projected allowance required"]
fn pilot_fit_report() -> TestResult {
    let path = std::env::var("UOR_PILOT_REPORT")?;
    let parent = std::env::var("UOR_PILOT_PARENT")?;
    let root = Path::new(&path);
    claim_output(root)?;
    let attempt_started = Instant::now();
    let result = (|| -> TestResult {
        pilot_data::validate()?;
        let cfg = config();
        let cfg_bytes = serde_json::to_vec_pretty(&cfg)?;
        fs::write(root.join("config.json"), &cfg_bytes)?;
        let train = pilot_data::training();
        let dev = pilot_data::development();
        write_json(root, "training.json", &train)?;
        write_json(root, "development.json", &dev)?;
        let parent_bytes = fs::read(parent)?;
        let model = Model::decode(&parent_bytes)?;
        let mut params = Parameters::seeded(7341)?;
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
            let draw = match batch(
                &params,
                model.geometry(),
                &document(example),
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
            for (i, &v) in draw.gradient.iter().enumerate() {
                norms[usize::from(i >= super::super::circuit::GATE_LOGITS)] += v * v;
            }
            let next = match sgd(&params, &draw.gradient, 0.05) {
                Ok(v) => v,
                Err(e) => {
                    loop_error = Some(e.to_string());
                    break;
                }
            };
            steps.push(serde_json::json!({"update":update+1,"example_id":example.id,"mean_ce":draw.mean_ce,"batch_us":draw.elapsed_us,"counts":draw.counts,"gate_gradient_l2":libm::sqrt(norms[0]),"head_gradient_l2":libm::sqrt(norms[1]),"before":hex::encode(params.digest()),"after":hex::encode(next.digest())}));
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
            &serde_json::json!({"status":if updates==64{"COMPLETED_64_UPDATE_PILOT_FIT"}else{"STOPPED_AT_FIT_LIMIT"},"updates":updates,"fit_us":fit_us,"attempt_us":attempt_started.elapsed().as_micros(),"checkpoint_stop_scheduling_ms":36000,"loop_error":loop_error,"parameter_count":PARAMETER_COUNT,"initial_digest":hex::encode(before),"final_digest":hex::encode(params.digest()),"changed_compiled_bytes":changed,"parameter_delta_l2":delta_l2,"finite":params.values().iter().all(|x|x.is_finite()),"normal_trained_model_artifact":"NOT_EXPORTED; checkpoint plus source-bound primitive witness","parent_model_id":hex::encode(model.id()),"geometry_digest":hex::encode(model.geometry().identity_digest()),"config_digest":hex::encode(blake3::hash(&cfg_bytes).as_bytes()),"fit_calls":1,"promotion":false}),
        )?;
        println!("pilot fit updates={updates}, fit_us={fit_us}, changed_compiled_bytes={changed}");
        if let Some(e) = loop_error {
            return Err(e.into());
        }
        Ok(())
    })();
    seal_result(root, result)
}
#[test]
#[ignore = "evaluation of the one sealed pilot fit; exclusive new report root required"]
fn pilot_evaluation_report() -> TestResult {
    let path = std::env::var("UOR_PILOT_REPORT")?;
    let fit_path = std::env::var("UOR_PILOT_FIT")?;
    let root = Path::new(&path);
    claim_output(root)?;
    let result = (|| -> TestResult {
        let started = Instant::now();
        let fit = Path::new(&fit_path);
        report_output::verify(fit)?;
        let summary: serde_json::Value =
            serde_json::from_slice(&fs::read(fit.join("fit-summary.json"))?)?;
        if summary["updates"] != 64 {
            return Err("pilot64updates not completed; no qualification".into());
        }
        let initial = load_checkpoint(fit, "initial-parameters.bin", 0)?;
        let final_cp = load_checkpoint(fit, "final-parameters.bin", 64)?;
        let base = Model::decode(&fs::read(fit.join("parent-initialized-model.bin"))?)?;
        let train: Vec<Example> = serde_json::from_slice(&fs::read(fit.join("training.json"))?)?;
        let dev: Vec<Example> = serde_json::from_slice(&fs::read(fit.join("development.json"))?)?;
        if train != pilot_data::training() || dev != pilot_data::development() {
            return Err("frozen data mismatch".into());
        }
        let initial_export = PrimitiveExport::decode(&fs::read(fit.join("initial-compiled.bin"))?)?;
        let final_export = PrimitiveExport::decode(&fs::read(fit.join("final-compiled.bin"))?)?;
        if initial_export != initial.parameters.compile()?
            || final_export != final_cp.parameters.compile()?
        {
            return Err("compiled/checkpoint binding".into());
        }
        let mut a = RuntimeSession::new(final_cp.parameters.digest(), base.geometry(), 1)?;
        let mut b = a.clone();
        let mut cp = CompiledPolicy {
            compiled: &final_export,
        };
        let mut ip = InterpretedPolicy {
            parameters: &final_cp.parameters,
        };
        for target in document(&dev[0]) {
            let x = a.predict(base.geometry(), &mut cp)?;
            let y = b.predict(base.geometry(), &mut ip)?;
            if x != y
                || a.observe(sym(target)?, x.offer.id, base.geometry(), &mut cp)?
                    != b.observe(sym(target)?, y.offer.id, base.geometry(), &mut ip)?
            {
                return Err("learned full interpreter/export parity".into());
            }
        }
        let init_train = evaluate(
            &initial.parameters,
            &initial_export,
            base.geometry(),
            &train,
            Control::Full,
        )?;
        let final_train = evaluate(
            &final_cp.parameters,
            &final_export,
            base.geometry(),
            &train,
            Control::Full,
        )?;
        let init_dev = evaluate(
            &initial.parameters,
            &initial_export,
            base.geometry(),
            &dev,
            Control::Full,
        )?;
        let final_dev = evaluate(
            &final_cp.parameters,
            &final_export,
            base.geometry(),
            &dev,
            Control::Full,
        )?;
        let mut arms = Vec::new();
        for control in &CONTROLS[1..] {
            arms.push(evaluate(
                &final_cp.parameters,
                &final_export,
                base.geometry(),
                &dev,
                *control,
            )?);
        }
        let mut result = decision(&init_dev, &final_dev, &arms)?;
        let metric = serde_json::json!({"initial_training":metrics(&init_train),"final_training":metrics(&final_train),"initial_development":metrics(&init_dev),"final_development":metrics(&final_dev),"controls":arms.iter().map(|r|serde_json::json!({"control":r[0].control,"metrics":metrics(r)})).collect::<Vec<_>>()});
        write_json(root, "initial-training.json", &init_train)?;
        write_json(root, "final-training.json", &final_train)?;
        write_json(root, "initial-development.json", &init_dev)?;
        write_json(root, "final-development.json", &final_dev)?;
        write_json(root, "controls.json", &arms)?;
        write_json(root, "metrics.json", &metric)?;
        let mut programs = Vec::new();
        for (index, row) in final_dev
            .iter()
            .enumerate()
            .filter(|(_, r)| r.expected_scalar.is_some())
        {
            let actual: Vec<u8> = row
                .generated
                .iter()
                .copied()
                .take_while(|&v| v != 256)
                .map(|v| v as u8)
                .collect();
            let filename = format!("generated-{index}.rs");
            fs::write(root.join(&filename), &actual)?;
            let mut rec = serde_json::json!({"id":row.id,"source":filename,"status":"NOT_EXECUTED_OUTSIDE_FROZEN_PURE_PROGRAM_GRAMMAR","expected":row.expected_scalar});
            // Only the frozen pure println program is admitted to local compilation.
            // The generated file itself is compiled, with no repair or response replacement.
            if actual == row.expected && row.generated_eos {
                let bin = root.join(format!("generated-{index}"));
                let output = std::process::Command::new("/Users/casey.allard/.cargo/bin/rustc")
                    .arg(root.join(&filename))
                    .args(["--crate-name", "pilot_generated", "-o"])
                    .arg(&bin)
                    .output()?;
                fs::write(
                    root.join(format!("generated-{index}-compile.stderr")),
                    &output.stderr,
                )?;
                if output.status.success() {
                    let run = std::process::Command::new(&bin).output()?;
                    let expected = format!("{}\n", row.expected_scalar.ok_or("scalar intent")?);
                    rec["status"] = if run.status.success() && run.stdout == expected.as_bytes() {
                        "PASS_GENERATED_PROGRAM_SEMANTICS".into()
                    } else {
                        "FAIL_GENERATED_PROGRAM_SEMANTICS".into()
                    };
                    rec["stdout"] = String::from_utf8_lossy(&run.stdout).to_string().into();
                } else {
                    rec["status"] = "FAIL_GENERATED_PROGRAM_COMPILATION".into();
                }
            }
            programs.push(rec);
        }
        let code_semantics = programs
            .iter()
            .any(|p| p["status"] == "PASS_GENERATED_PROGRAM_SEMANTICS")
            && !programs.iter().any(|p| {
                p["status"] == "FAIL_GENERATED_PROGRAM_SEMANTICS"
                    || p["status"] == "FAIL_GENERATED_PROGRAM_COMPILATION"
            });
        if !code_semantics {
            result.status = "FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT";
        }
        write_json(root, "program-semantics.json", &programs)?;
        write_json(
            root,
            "summary.json",
            &serde_json::json!({"status":result.status,"decision":result,"generated_code_semantics":code_semantics,"metrics":metric,"initial_parameter_digest":hex::encode(initial.parameters.digest()),"final_parameter_digest":hex::encode(final_cp.parameters.digest()),"loaded_checkpoint_export_parity":true,"evaluation_us":started.elapsed().as_micros(),"fit_calls":0,"promotion":false,"general_language_coding":"NOT_QUALIFIED","source_fit_root":fit_path}),
        )?;
        println!("{}", result.status);
        Ok(())
    })();
    seal_result(root, result)
}

#[test]
fn pilot_gate_rejects_a_retained_symbol_regression_despite_aggregate_improvement() -> TestResult {
    fn rows(final_arm: bool, disabled: bool) -> Vec<EvaluationRow> {
        pilot_data::development()
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let targets: Vec<u16> = e
                    .answer
                    .iter()
                    .map(|&b| u16::from(b))
                    .chain([256])
                    .collect();
                let mut predictions = if final_arm {
                    targets.clone()
                } else {
                    vec![999; targets.len()]
                };
                if i == 0 {
                    predictions[0] = if final_arm { 999 } else { targets[0] };
                }
                EvaluationRow {
                    id: e.id.clone(),
                    family: e.family.clone(),
                    control: Control::Full,
                    prompt: e.prompt.clone(),
                    expected: e.answer.clone(),
                    expected_scalar: e.expected_scalar,
                    correct_symbols: predictions
                        .iter()
                        .zip(&targets)
                        .filter(|(a, b)| a == b)
                        .count(),
                    positions: targets.len(),
                    predictions,
                    generated: targets,
                    response_ce: if final_arm && !disabled { 3.0 } else { 5.0 },
                    exact_response_and_eos: final_arm && !disabled,
                    generated_eos: true,
                    candidate_scores: 0,
                    circuit_calls: 0,
                    selected_reads: 0,
                    trace_digest: String::new(),
                    first_generated_state_digest: String::new(),
                }
            })
            .collect()
    }
    let initial = rows(false, false);
    let final_arm = rows(true, false);
    let controls = vec![rows(true, true), rows(true, true), rows(true, true)];
    let d = decision(&initial, &final_arm, &controls)?;
    assert!(d.predictive_improvement);
    assert!(d.both_families_exact);
    assert!(d.controls_weaker.iter().all(|&v| v));
    assert_eq!(d.lost_initial_correct_symbols, 1);
    assert_eq!(d.status, "FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT");
    Ok(())
}
