//! One bounded learned angular emission experiment. Test-only driver.
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-10;
const PARENT: &str = "blake3:0f82b7286bb1699b9ea3bf4193f152702f3b22e702cf31bcd0a0f56596bbf8b8";
const OLDER: &str = "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58";
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
            "angular_tree",
            "schema",
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
#[ignore = "requires sealed parents, corpus, frozen tree design and exclusive output"]
fn angular_tree_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_TREE_INPUT")?)?;
    let older_input = fs::canonicalize(std::env::var("UOR_TREE_OLDER")?)?;
    let corpus = fs::canonicalize(std::env::var("UOR_TREE_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_TREE_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_TREE_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    for root in [&input, &older_input, &corpus] {
        if out.starts_with(root) {
            return Err("output beneath sealed input".into());
        }
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        let start = Instant::now();
        for root in [&input, &older_input, &corpus] {
            crate::report_output::verify(root)?;
        }
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design size".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        if design["schema"] != "uor-r4.angular-tree-design/1"
            || design["parent"] != PARENT
            || design["other_control"] != OLDER
            || design["construction_documents"] != 12
            || design["construction_positions"] != 672
            || design["opened_positions"] != 381
            || design["generation_prompts"] != 4
            || design["generation_bytes"] != 96
            || design["fit_calls"] != 1
            || design["experiment_seconds"] != 90
            || design["config"] != json!({"max_depth":3,"min_leaf":4,"max_seconds":60})
            || design["max_nodes_per_branch"] != 15
            || design["max_emission_nodes"] != 7680
            || design["max_angular_comparisons_per_token"] != 27
            || design["max_extra_serialized_bytes"] != 1048576
            || design["leaf_score_min"] != -10
            || design["leaf_score_max"] != 10
            || design["gates"]
                != json!({"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_rows_each_parent_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true})
        {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let parent_bytes = fs::read(input.join("candidate.json"))?;
        let older_bytes = fs::read(older_input.join("candidate.json"))?;
        let legacy_behavior: Value =
            serde_json::from_slice(&fs::read(input.join("behavior.json"))?)?;
        let parent = SharedCore::from_bytes(&parent_bytes)?;
        let older = SharedCore::from_bytes(&older_bytes)?;
        if parent.artifact_cid() != PARENT
            || older.artifact_cid() != OLDER
            || parent.to_bytes()? != parent_bytes
            || older.to_bytes()? != older_bytes
        {
            return Err("parent identity".into());
        }
        let corpus_bytes = fs::read(corpus.join("design.json"))?;
        if design["corpus_sha256"] != format!("{:x}", sha2::Sha256::digest(&corpus_bytes)) {
            return Err("corpus identity".into());
        }
        let original: Value = serde_json::from_slice(&corpus_bytes)?;
        let training = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        if training.len() != 12
            || training.iter().map(|d| d.len() + 1).sum::<usize>() != 672
            || opened.iter().map(|d| d.len() + 1).sum::<usize>() != 381
            || prompts.len() != 4
        {
            return Err("data size".into());
        }
        guard(start)?;
        let (candidate, report) = parent.fit_angular_emission(
            &training,
            AngularTreeConfig {
                max_depth: 3,
                min_leaf: 4,
                max_seconds: 60,
            },
        )?;
        let candidate_bytes = candidate.to_bytes()?;
        fs::File::create_new(out.join("candidate.json"))?.write_all(&candidate_bytes)?;
        write(&out, "fit.json", &report)?;
        let loaded = SharedCore::from_bytes(&candidate_bytes)?;
        if loaded.to_bytes()? != candidate_bytes
            || loaded.artifact_cid() != candidate.artifact_cid()
        {
            return Err("candidate roundtrip".into());
        }
        if candidate_bytes.len() > parent_bytes.len() + 1048576 {
            return Err("candidate storage bound".into());
        }
        fixed_fields_equal(
            serde_json::from_slice(&parent_bytes)?,
            serde_json::from_slice(&candidate_bytes)?,
        )?;
        if parent.artifact.parameters != loaded.artifact.parameters
            || parent.artifact.geometry != loaded.artifact.geometry
        {
            return Err("fixed fields".into());
        }
        let report_json = serde_json::to_value(&report)?;
        if report_json["parent"] != PARENT || report_json["config"] != design["config"] {
            return Err("fit binding".into());
        }
        let early = generate(&loaded, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"candidate":loaded.artifact_cid(),"outputs":early}),
        )?;
        guard(start)?;
        let mut behavior = Vec::new();
        let mut lost = [0u64; 2];
        let mut metrics = Vec::new();
        // Every earlier control artifact is compared separately; improvements over
        // the immediate experimental parent cannot hide older lost predictions.
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                guard(start)?;
                let changed = rows(&loaded, docs, control)?;
                let after = verified_metrics(&loaded, docs, control, &changed)?;
                let mut comparisons = Vec::new();
                for (index, base_model) in [&parent, &older].into_iter().enumerate() {
                    let base = rows(base_model, docs, control)?;
                    let before = verified_metrics(base_model, docs, control, &base)?;
                    let saved = legacy_behavior
                        .as_array()
                        .ok_or("saved artifacts")?
                        .iter()
                        .find(|v| v["artifact"] == base_model.artifact_cid())
                        .ok_or("saved artifact")?;
                    let saved_panel = saved["panels"]
                        .as_array()
                        .ok_or("saved panels")?
                        .iter()
                        .find(|v| {
                            v["panel"] == name
                                && v["control"]
                                    == serde_json::to_value(control).unwrap_or(Value::Null)
                        })
                        .ok_or("saved panel")?;
                    let saved_metrics: Metrics =
                        serde_json::from_value(saved_panel["metrics"].clone())?;
                    if (saved_metrics.mean_nll - before.mean_nll).abs() > EPS
                        || saved_metrics.correct != before.correct
                    {
                        return Err("legacy metric changed across implementation".into());
                    }
                    let saved_rows = saved_panel["rows"].as_array().ok_or("saved rows")?;
                    if saved_rows.len() != base.len() {
                        return Err("legacy row count".into());
                    }
                    for (a, b) in saved_rows.iter().zip(&base) {
                        if a["document"] != b.document
                            || a["position"] != b.position
                            || a["prediction"] != b.prediction
                            || a["target"] != b.target
                            || a["roots"] != json!(b.roots)
                            || a["source"] != json!(b.source)
                            || (a["nll"].as_f64().ok_or("saved loss")? - b.nll).abs() > EPS
                        {
                            return Err("legacy row changed across implementation".into());
                        }
                    }
                    let comparison = compare(&base, &changed)?;
                    require_fixed_trace(&comparison)?;
                    lost[index] += comparison["correct_to_wrong"].as_u64().ok_or("lost rows")?;
                    if control == Intervention::Full {
                        metrics.push(json!({"panel":name,"parent":base_model.artifact_cid(),"before":before,"after":after}));
                    }
                    comparisons.push(json!({"parent":base_model.artifact_cid(),"metrics":before,"rows":base,"comparison":comparison}));
                }
                behavior.push(json!({"panel":name,"control":control,"candidate_metrics":after,"candidate_rows":changed,"comparisons":comparisons}));
            }
        }
        let mut generations = Vec::new();
        for model in [&parent, &older, &loaded] {
            generations.push(json!({"artifact":model.artifact_cid(),"outputs":generate(model,&prompts,&CONTROLS)?}));
        }
        for (i, first) in early.iter().enumerate() {
            if first != &generations[2]["outputs"][i * 3] {
                return Err("early generation replay".into());
            }
        }
        let construction = metrics
            .iter()
            .find(|m| m["panel"] == "complete_construction" && m["parent"] == PARENT)
            .ok_or("construction")?;
        let before: Metrics = serde_json::from_value(construction["before"].clone())?;
        let after: Metrics = serde_json::from_value(construction["after"].clone())?;
        if (report.before.mean_nll - before.mean_nll).abs() > EPS
            || (report.after.mean_nll - after.mean_nll).abs() > EPS
            || report.before.correct != before.correct
            || report.after.correct != after.correct
        {
            return Err("fit replay metrics".into());
        }
        let optimization =
            after.mean_nll + EPS < before.mean_nll && after.correct >= before.correct;
        let preservation = lost == [0, 0];
        let development = metrics
            .iter()
            .filter(|m| m["panel"] == "opened_development")
            .all(|m| {
                m["after"]["mean_nll"].as_f64().unwrap_or(f64::INFINITY)
                    <= m["before"]["mean_nll"]
                        .as_f64()
                        .unwrap_or(f64::NEG_INFINITY)
                        + EPS
                    && m["after"]["correct"].as_u64() >= m["before"]["correct"].as_u64()
            });
        write(
            &out,
            "behavior.json",
            &json!({"panels":behavior,"generations":generations}),
        )?;
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":digest(include_bytes!("angular_tree_experiment.rs")),"tree_source":digest(include_bytes!("../angular_tree.rs")),"design":digest(&design_bytes),"corpus":digest(&corpus_bytes),"parent":PARENT,"older":OLDER,"candidate":loaded.artifact_cid(),"candidate_sha256":format!("{:x}",sha2::Sha256::digest(&candidate_bytes))}),
        )?;
        for root in [&input, &older_input, &corpus] {
            crate::report_output::verify(root)?;
        }
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.angular-tree-result/1","decision":if optimization&&preservation&&development{"PASS_ANGULAR_TREE_DEVELOPMENT_GATE"}else{"FAIL_ANGULAR_TREE_DEVELOPMENT_GATE"},"candidate":loaded.artifact_cid(),"parent":PARENT,"older":OLDER,"conditional_optimization_gate":optimization,"preservation_gate":preservation,"opened_development_gate":development,"correct_to_wrong_each_parent":lost,"metrics":metrics,"before":before,"after":after,"fit":report_json,"fit_calls":1,"elapsed_ms":start.elapsed().as_millis(),"candidate_bytes":candidate_bytes.len(),"extra_bytes":candidate_bytes.len().saturating_sub(parent_bytes.len()),"fixed_root_source_traces":true,"promoted":false,"scope":"One greedy bounded angular tree fit on construction only. No global tree optimum, fresh holdout, recurrence fit, promotion, language or energy qualification."}),
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
