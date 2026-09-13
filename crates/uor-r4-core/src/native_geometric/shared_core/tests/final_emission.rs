//! One exact emission recalibration on a sealed final-state parent. Test-only driver.
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-10;
const PARENT: &str = "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58";
const CONTROLS: [Intervention; 3] = [
    Intervention::Full,
    Intervention::ContextDisabled,
    Intervention::StateDisabled,
];
fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}
fn documents(value: &Value) -> RunResult<Vec<Vec<u8>>> {
    value
        .as_array()
        .ok_or("documents")?
        .iter()
        .map(|v| Ok(v.as_str().ok_or("document")?.as_bytes().to_vec()))
        .collect()
}
fn write(out: &Path, name: &str, value: &impl Serialize) -> RunResult<()> {
    fs::File::create_new(out.join(name))?.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn guard(start: Instant) -> RunResult<()> {
    if start.elapsed().as_secs() >= 90 {
        return Err("complete emission experiment time guard".into());
    }
    Ok(())
}
// Whitelist only the emitter and its normal provenance. Every other artifact
// field, including future added source fields, must remain equal.
fn fixed_fields_equal(mut parent: Value, mut candidate: Value) -> RunResult<()> {
    for value in [&mut parent, &mut candidate] {
        let object = value.as_object_mut().ok_or("artifact object")?;
        for key in [
            "calibrated",
            "implementation",
            "training_digest",
            "fit_config",
            "tied_fit_config",
            "training_parent",
        ] {
            object.remove(key);
        }
    }
    if parent != candidate {
        return Err("non-emission artifact field changed".into());
    }
    Ok(())
}
#[derive(Clone, Serialize)]
struct Row {
    document: usize,
    position: usize,
    target: u16,
    prediction: u16,
    nll: f64,
    roots: [u16; LANES],
    source: Option<u64>,
}
fn target_nll(session: &mut CoreSession<'_>, target: u16) -> f64 {
    let (mut lo, mut hi, mut node, mut depth) = (0_u16, EOS + 1, 0, 0);
    let mut loss = 0.0;
    while hi - lo > 1 {
        let mid = (lo + hi) >> 1;
        let right = target >= mid;
        let score = f64::from(session.branch_score(node, depth)) / 4.0;
        loss += libm::log1p(libm::exp(if right { -score } else { score }));
        if right {
            lo = mid;
        } else {
            hi = mid;
        }
        node = (node << 1) + 1 + usize::from(right);
        depth += 1;
    }
    loss
}
fn rows(model: &SharedCore, docs: &[Vec<u8>], control: Intervention) -> Result<Vec<Row>> {
    let mut result = Vec::new();
    for (document, doc) in docs.iter().enumerate() {
        let mut session = model.session(control);
        for (position, target) in doc
            .iter()
            .map(|&b| u16::from(b))
            .chain(std::iter::once(EOS))
            .enumerate()
        {
            let prediction = session.predict();
            let nll = target_nll(&mut session, target);
            result.push(Row {
                document,
                position,
                target,
                prediction,
                nll,
                roots: session.state_roots(),
                source: session.last_source(),
            });
            if target != EOS {
                session.observe(target as u8)?;
            }
        }
    }
    Ok(result)
}
fn compare(base: &[Row], changed: &[Row]) -> RunResult<Value> {
    if base.len() != changed.len() {
        return Err("row count differs".into());
    }
    let (mut lost, mut gained, mut predictions, mut sources, mut roots) = (0, 0, 0, 0, 0);
    let mut deltas = Vec::with_capacity(base.len());
    for (a, b) in base.iter().zip(changed) {
        if (a.document, a.position, a.target) != (b.document, b.position, b.target) {
            return Err("row alignment differs".into());
        }
        if !a.nll.is_finite() || !b.nll.is_finite() {
            return Err("nonfinite row loss".into());
        }
        let correct_to_wrong = a.prediction == a.target && b.prediction != b.target;
        let wrong_to_correct = a.prediction != a.target && b.prediction == b.target;
        let root_delta = a
            .roots
            .iter()
            .zip(b.roots)
            .filter(|(x, y)| **x != *y)
            .count();
        lost += usize::from(correct_to_wrong);
        gained += usize::from(wrong_to_correct);
        predictions += usize::from(a.prediction != b.prediction);
        sources += usize::from(a.source != b.source);
        roots += root_delta;
        deltas.push(json!({"document":a.document,"position":a.position,"target":a.target,"parent_prediction":a.prediction,"candidate_prediction":b.prediction,"parent_nll":a.nll,"candidate_nll":b.nll,"nll_delta":b.nll-a.nll,"correct_to_wrong":correct_to_wrong,"wrong_to_correct":wrong_to_correct,"root_identity_hamming":root_delta,"source_identity_hamming":usize::from(a.source!=b.source)}));
    }
    Ok(
        json!({"positions":base.len(),"correct_to_wrong":lost,"wrong_to_correct":gained,"prediction_hamming":predictions,"source_identity_hamming":sources,"root_identity_hamming":roots,"hamming_scope":"Aligned categorical identities, never bit Hamming or a geometric metric.","rows":deltas}),
    )
}
fn require_fixed_trace(comparison: &Value) -> RunResult<()> {
    if comparison["root_identity_hamming"] != 0 || comparison["source_identity_hamming"] != 0 {
        return Err("emitter-only intervention changed root/source trace".into());
    }
    Ok(())
}
fn verified_metrics(
    model: &SharedCore,
    docs: &[Vec<u8>],
    control: Intervention,
    rows: &[Row],
) -> RunResult<Metrics> {
    let metrics = model.evaluate(docs, control)?;
    if rows.is_empty()
        || (rows.iter().map(|r| r.nll).sum::<f64>() / rows.len() as f64 - metrics.mean_nll).abs()
            > EPS
        || rows.iter().filter(|r| r.prediction == r.target).count() != metrics.correct
    {
        return Err("row/evaluate mismatch".into());
    }
    Ok(metrics)
}
fn generate(
    model: &SharedCore,
    prompts: &[String],
    controls: &[Intervention],
) -> Result<Vec<Value>> {
    let mut outputs = Vec::new();
    for prompt in prompts {
        for &control in controls {
            let (bytes, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
            outputs.push(json!({"prompt":prompt,"control":control,"bytes":bytes,"text":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
        }
    }
    Ok(outputs)
}
#[test]
fn final_emission_accounting_detects_regressions_alignment_and_source_changes() {
    let base = vec![
        Row {
            document: 0,
            position: 0,
            target: 3,
            prediction: 3,
            nll: 1.0,
            roots: [1; LANES],
            source: Some(0),
        },
        Row {
            document: 0,
            position: 1,
            target: 4,
            prediction: 5,
            nll: 2.0,
            roots: [2; LANES],
            source: None,
        },
    ];
    let mut changed = base.clone();
    changed[0].prediction = 8;
    changed[1].prediction = 4;
    changed[0].nll = 1.5;
    changed[1].nll = 1.0;
    let comparison = compare(&base, &changed).unwrap();
    assert_eq!(comparison["correct_to_wrong"], 1);
    assert_eq!(comparison["wrong_to_correct"], 1);
    assert_eq!(comparison["prediction_hamming"], 2);
    assert_eq!(comparison["rows"][0]["nll_delta"], 0.5);
    require_fixed_trace(&comparison).unwrap();
    changed[0].source = Some(1);
    let comparison = compare(&base, &changed).unwrap();
    assert_eq!(comparison["root_identity_hamming"], 0);
    assert_eq!(comparison["source_identity_hamming"], 1);
    assert!(require_fixed_trace(&comparison).is_err());
    changed[0].source = base[0].source;
    changed[0].roots[0] = 9;
    let comparison = compare(&base, &changed).unwrap();
    assert_eq!(comparison["root_identity_hamming"], 1);
    assert_eq!(comparison["source_identity_hamming"], 0);
    assert!(require_fixed_trace(&comparison).is_err());
    changed[0].position = 5;
    assert!(compare(&base, &changed).is_err());
    assert!(compare(&base, &changed[..1]).is_err());
    let parent = json!({"parameters":[1,2],"geometry":{"identity":3},"angular":[4],"seed":5,"calibrated":{"branches":[1]},"training_parent":"a"});
    let mut candidate = parent.clone();
    candidate["calibrated"] = json!({"branches":[2]});
    candidate["training_parent"] = Value::Null;
    fixed_fields_equal(parent.clone(), candidate.clone()).unwrap();
    candidate["parameters"][0] = json!(9);
    assert!(fixed_fields_equal(parent.clone(), candidate).is_err());
    let mut candidate = parent.clone();
    candidate["geometry"]["identity"] = json!(7);
    assert!(fixed_fields_equal(parent, candidate).is_err());
}
#[test]
#[ignore = "requires sealed parent/corpus, frozen final-emission design and exclusive output"]
fn final_emission_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_EMISSION_INPUT")?)?;
    let corpus = fs::canonicalize(std::env::var("UOR_EMISSION_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_EMISSION_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_EMISSION_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    if out.starts_with(&input) || out.starts_with(&corpus) {
        return Err("output beneath sealed input".into());
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        let start = Instant::now();
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&corpus)?;
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design too large".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        if design["schema"] != "uor-r4.final-emission-experiment/1"
            || design["parent"] != PARENT
            || design["construction_documents"] != 12
            || design["construction_positions"] != 672
            || design["opened_positions"] != 381
            || design["prompts"] != 4
            || design["max_seconds"] != 60
            || design["experiment_seconds"] != 90
            || design["fit_calls"] != 1
            || design["gate"]
                != json!({"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_rows_across_all_panels_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true})
        {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let raw_parent = fs::read(input.join("candidate.json"))?;
        let parent = SharedCore::from_bytes(&raw_parent)?;
        if parent.to_bytes()? != raw_parent || parent.artifact_cid() != PARENT {
            return Err("parent bytes/CID mismatch".into());
        }
        let corpus_bytes = fs::read(corpus.join("design.json"))?;
        if design["corpus_design_sha256"] != format!("{:x}", sha2::Sha256::digest(&corpus_bytes)) {
            return Err("corpus design SHA256 mismatch".into());
        }
        let original: Value = serde_json::from_slice(&corpus_bytes)?;
        let training = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        if training.len() != 12
            || training.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || opened.iter().map(|d| d.len() + 1).sum::<usize>() != 381
        {
            return Err("corpus positions mismatch".into());
        }
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        if prompts.len() != 4 {
            return Err("prompt count".into());
        }
        guard(start)?;
        // The only fitting call in this attempt. No search/retry/second calibration.
        let (candidate, report) = parent.calibrate_emission_blocks(&training, 60)?;
        let candidate_bytes = candidate.to_bytes()?;
        fs::File::create_new(out.join("candidate.json"))?.write_all(&candidate_bytes)?;
        write(&out, "calibration.json", &report)?;
        let loaded = SharedCore::from_bytes(&candidate_bytes)?;
        if loaded.to_bytes()? != candidate_bytes
            || loaded.artifact_cid() != candidate.artifact_cid()
        {
            return Err("candidate bytes/CID roundtrip mismatch".into());
        }
        fixed_fields_equal(
            serde_json::from_slice(&raw_parent)?,
            serde_json::from_slice(&candidate_bytes)?,
        )?;
        if parent.artifact.parameters != loaded.artifact.parameters
            || parent.artifact.geometry != loaded.artifact.geometry
        {
            return Err("fixed parameter/geometry mismatch".into());
        }
        let old = parent
            .artifact
            .calibrated
            .as_ref()
            .ok_or("parent emitter")?;
        let new = loaded
            .artifact
            .calibrated
            .as_ref()
            .ok_or("candidate emitter")?;
        if new.parent != PARENT
            || new.calibration_data.as_ref() != Some(&report.data_digest)
            || new.calibration_passes != 1
            || new
                .block_config
                .as_ref()
                .ok_or("block configuration")?
                .max_seconds
                != 60
            || report.settings != report.nodes * 3_484_800
        {
            return Err("calibration provenance/search completeness mismatch".into());
        }
        let changed_branches: Vec<usize> = old
            .branches
            .iter()
            .zip(&new.branches)
            .enumerate()
            .filter_map(|(i, (a, b))| {
                ((a.landmarks, a.thresholds, a.union) != (b.landmarks, b.thresholds, b.union))
                    .then_some(i)
            })
            .collect();
        // Actual free generation comes immediately after construction/roundtrip,
        // before the construction, opened-development and intervention panels.
        let early = generate(&loaded, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"candidate":loaded.artifact_cid(),"outputs":early}),
        )?;
        guard(start)?;
        let mut parent_panels = Vec::new();
        let mut candidate_panels = Vec::new();
        let mut regressions = 0_u64;
        let mut construction = None;
        let mut development = None;
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                guard(start)?;
                let base = rows(&parent, docs, control)?;
                let changed = rows(&loaded, docs, control)?;
                let before = verified_metrics(&parent, docs, control, &base)?;
                let after = verified_metrics(&loaded, docs, control, &changed)?;
                let comparison = compare(&base, &changed)?;
                require_fixed_trace(&comparison)?;
                regressions += comparison["correct_to_wrong"]
                    .as_u64()
                    .ok_or("regression count")?;
                if control == Intervention::Full {
                    if name == "complete_construction" {
                        if (report.before.mean_nll - before.mean_nll).abs() > EPS
                            || (report.after.mean_nll - after.mean_nll).abs() > EPS
                            || report.before.correct != before.correct
                            || report.after.correct != after.correct
                        {
                            return Err("fit/panel replay mismatch".into());
                        }
                        construction = Some((before.clone(), after.clone()));
                    } else {
                        development = Some((before.clone(), after.clone()));
                    }
                }
                parent_panels
                    .push(json!({"panel":name,"control":control,"metrics":before,"rows":base}));
                candidate_panels.push(json!({"panel":name,"control":control,"metrics":after,"rows":changed,"comparison_to_parent":comparison}));
            }
        }
        let (before, after) = construction.ok_or("construction metrics")?;
        let (opened_before, opened_after) = development.ok_or("development metrics")?;
        let conditional_optimization =
            after.mean_nll + EPS < before.mean_nll && after.correct >= before.correct;
        let preservation = regressions == 0;
        let opened_gate = opened_after.mean_nll <= opened_before.mean_nll + EPS
            && opened_after.correct >= opened_before.correct;
        let pass = conditional_optimization && preservation && opened_gate;
        let parent_generation = generate(&parent, &prompts, &CONTROLS)?;
        let candidate_generation = generate(&loaded, &prompts, &CONTROLS)?;
        // Deterministic replay of the immediate Full generation also verifies
        // that metric evaluation has not altered model state.
        for (i, first) in early.iter().enumerate() {
            if first != &candidate_generation[i * 3] {
                return Err("early/final generation mismatch".into());
            }
        }
        write(
            &out,
            "behavior.json",
            &json!([
                {"artifact":parent.artifact_cid(),"panels":parent_panels,"generation":parent_generation},
                {"artifact":loaded.artifact_cid(),"panels":candidate_panels,"generation":candidate_generation}
            ]),
        )?;
        write(
            &out,
            "source.json",
            &json!({"parent_cid":parent.artifact_cid(),"candidate_cid":loaded.artifact_cid(),"parent_implementation":parent.artifact.implementation,"implementation":SharedCore::implementation_digest(),"driver":digest(include_bytes!("final_emission.rs")),"block_calibration_source":digest(include_bytes!("../block_calibration.rs")),"design":digest(&design_bytes),"corpus_design":digest(&corpus_bytes),"parent_sha256":format!("{:x}",sha2::Sha256::digest(&raw_parent)),"candidate_sha256":format!("{:x}",sha2::Sha256::digest(&candidate_bytes))}),
        )?;
        if parent.to_bytes()? != raw_parent {
            return Err("parent mutated".into());
        }
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&corpus)?;
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.final-emission-result/1","decision":if pass{"PASS_FINAL_EMISSION_DEVELOPMENT_GATE"}else{"FAIL_FINAL_EMISSION_DEVELOPMENT_GATE"},"parent":parent.artifact_cid(),"candidate":loaded.artifact_cid(),"before":before,"after":after,"opened_before":opened_before,"opened_after":opened_after,"conditional_optimization_gate":conditional_optimization,"preservation_gate":preservation,"opened_development_gate":opened_gate,"correct_to_wrong_across_panels":regressions,"changed_branch_count":changed_branches.len(),"changed_branches":changed_branches,"parameters_unchanged":true,"geometry_unchanged":true,"root_source_traces_identical":true,"fit_calls":1,"nodes":report.nodes,"settings":report.settings,"fit_elapsed_ms":report.elapsed_ms,"elapsed_ms":start.elapsed().as_millis(),"promoted":false,"scope":"One conditional emission optimization on the final fixed state trajectories, complete construction and already-open development controls. No fresh holdout, recurrent fit, candidate promotion, language or energy qualification."}),
        )?;
        guard(start)?;
        Ok(())
    })();
    if let Err(error) = &result {
        write(
            &out,
            "failure.json",
            &json!({"status":"INCOMPLETE","error":error.to_string()}),
        )?;
    }
    crate::report_output::seal(&out)?;
    crate::report_output::verify(&out)?;
    result
}
