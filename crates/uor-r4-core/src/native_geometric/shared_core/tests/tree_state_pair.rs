//! Exhaustive causal state-pair search with a fixed decoder; test-only witnesses.
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const EPS: f64 = 1e-10;
const PARENT: &str = "blake3:8f8e5c342c83b062274744033a96668e9c079dc47a9c4d1b138baa7ea4889b6a";
const PRIOR_TREE: &str = "blake3:1198ef4788397b9bf8cae20b8df4b23573bc9861eb812a51baf359f2545618b1";
const CAP: &str = "blake3:0f82b7286bb1699b9ea3bf4193f152702f3b22e702cf31bcd0a0f56596bbf8b8";
const OLDER: &str = "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58";
const INDICES: [usize; 2] = [1176, 1930];
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

fn saved_rows_equal(saved: &Value, actual: &[Row], before: &Metrics) -> RunResult<()> {
    let metrics: Metrics = serde_json::from_value(saved["metrics"].clone())?;
    if metrics.positions != before.positions
        || metrics.correct != before.correct
        || (metrics.mean_nll - before.mean_nll).abs() > EPS
    {
        return Err("saved artifact metrics changed".into());
    }
    let saved = saved["rows"].as_array().ok_or("saved rows")?;
    if saved.len() != actual.len() {
        return Err("saved row count".into());
    }
    for (a, b) in saved.iter().zip(actual) {
        if a["document"] != b.document
            || a["position"] != b.position
            || a["prediction"] != b.prediction
            || a["target"] != b.target
            || a["roots"] != json!(b.roots)
            || a["source"] != json!(b.source)
            || (a["nll"].as_f64().ok_or("saved NLL")? - b.nll).abs() > EPS
        {
            return Err("saved artifact row changed".into());
        }
    }
    Ok(())
}
fn driver_digest() -> String {
    digest(include_bytes!("tree_state_pair.rs"))
}
fn select(costs: &[f64], parent_index: usize) -> RunResult<usize> {
    if parent_index >= costs.len() || costs.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err("invalid exhaustive costs".into());
    }
    let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
    if costs[parent_index] <= minimum + EPS {
        return Ok(parent_index);
    }
    costs
        .iter()
        .position(|&v| v <= minimum + EPS)
        .ok_or_else(|| "no minimum".into())
}
fn require_only_pair_changed(parent: &SharedCore, changed: &SharedCore) -> RunResult<()> {
    let mut actual = serde_json::to_value(&changed.artifact)?;
    let expected = serde_json::to_value(&parent.artifact)?;
    for index in INDICES {
        actual["parameters"][index] = expected["parameters"][index].clone();
    }
    if actual != expected {
        return Err("state-pair witness altered another artifact field".into());
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    schema: String,
    parent: String,
    implementation: String,
    driver: String,
    design: String,
    corpus_design: String,
    indices: [usize; 2],
    selected_roots: [u16; 2],
    evaluated_settings: usize,
    parent_nll: f64,
    selected_nll: f64,
}
impl Witness {
    fn replay(&self, parent: &SharedCore, design: &str, corpus: &str) -> RunResult<SharedCore> {
        if self.schema != "uor-r4.tree-state-pair-witness/1"
            || self.parent != parent.artifact_cid()
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus_design != corpus
            || self.indices != INDICES
            || self.selected_roots.iter().any(|&r| usize::from(r) >= ROOTS)
            || self.evaluated_settings != 14400
            || !self.parent_nll.is_finite()
            || !self.selected_nll.is_finite()
            || self.parent_nll < 0.0
            || self.selected_nll < 0.0
            || parent.artifact.angular_tree.is_none()
        {
            return Err("unbound or invalid state-pair witness".into());
        }
        let mut changed = parent.clone();
        for (i, r) in INDICES.into_iter().zip(self.selected_roots) {
            changed.artifact.parameters[i] = r;
        }
        require_only_pair_changed(parent, &changed)?;
        // The inherited CID is not an identity for this in-memory intervention.
        // Only the serialized witness is exported and named in model evidence.
        Ok(changed)
    }
}
#[test]
fn tree_state_pair_witness_preserves_decoder_and_selects_exact_ties() -> RunResult<()> {
    let mut parent = SharedCore::initialized(745)?.upgrade_calibrated()?;
    let cap_cid = parent.artifact_cid().to_string();
    parent.artifact.schema = ANGULAR_TREE_SCHEMA.into();
    let mut branches = vec![None; 512];
    branches[0] = Some(vec![AngularNode::Leaf { score: 3 }]);
    parent.artifact.angular_tree = Some(AngularEmission {
        config: AngularTreeConfig {
            max_depth: 1,
            min_leaf: 4,
            max_seconds: 60,
        },
        parent: cap_cid,
        data_digest: format!("blake3:{}", "a".repeat(64)),
        branches,
    });
    parent.refresh_identity()?;
    let original = parent.to_bytes()?;
    let witness = Witness {
        schema: "uor-r4.tree-state-pair-witness/1".into(),
        parent: parent.artifact_cid().into(),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "design".into(),
        corpus_design: "corpus".into(),
        indices: INDICES,
        selected_roots: INDICES.map(|i| (parent.artifact.parameters[i] + 1) % 120),
        evaluated_settings: 14400,
        parent_nll: 2.0,
        selected_nll: 1.0,
    };
    let loaded: Witness = serde_json::from_slice(&serde_json::to_vec(&witness)?)?;
    let changed = loaded.replay(&parent, "design", "corpus")?;
    for (i, (&a, &b)) in parent
        .artifact
        .parameters
        .iter()
        .zip(&changed.artifact.parameters)
        .enumerate()
    {
        if INDICES.contains(&i) {
            assert_ne!(a, b);
        } else {
            assert_eq!(a, b);
        }
    }
    assert_eq!(
        serde_json::to_value(&parent.artifact.angular_tree)?,
        serde_json::to_value(&changed.artifact.angular_tree)?
    );
    assert_eq!(original, parent.to_bytes()?);
    for field in [
        "parent",
        "implementation",
        "driver",
        "design",
        "corpus_design",
        "schema",
    ] {
        let mut bad = serde_json::to_value(&witness)?;
        bad[field] = json!("changed");
        let bad: Witness = serde_json::from_value(bad)?;
        assert!(bad.replay(&parent, "design", "corpus").is_err());
    }
    let mut bad = witness.clone();
    bad.indices = [1176, 1176];
    assert!(bad.replay(&parent, "design", "corpus").is_err());
    let mut bad = witness.clone();
    bad.selected_roots[0] = 120;
    assert!(bad.replay(&parent, "design", "corpus").is_err());
    let mut bad_model = changed.clone();
    bad_model.artifact.parameters[0] = (bad_model.artifact.parameters[0] + 1) % 120;
    assert!(require_only_pair_changed(&parent, &bad_model).is_err());
    let mut bad_model = changed;
    bad_model.artifact.angular_tree = None;
    assert!(require_only_pair_changed(&parent, &bad_model).is_err());
    assert_eq!(select(&[1.0, 1.0, 1.0], 2)?, 2);
    assert_eq!(select(&[1.0, 1.0, 2.0], 2)?, 0);
    assert_eq!(select(&[1.0 + 0.5 * EPS, 1.0, 1.0 + 1.5 * EPS], 2)?, 0); // Global minimum, not a chain of pairwise near-ties.
    assert_eq!(select(&[1.0, 1.0 + 0.5 * EPS], 1)?, 1);
    assert!(select(&[f64::NAN], 0).is_err());
    Ok(())
}
#[test]
#[ignore = "requires sealed decoder/priors/corpus and frozen tree-state-pair design"]
fn tree_state_pair_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_TREE_PAIR_INPUT")?)?;
    let evidence = input
        .parent()
        .and_then(Path::parent)
        .ok_or("evidence parent")?;
    let prior_input = fs::canonicalize(evidence.join("angular-tree-1/attempt-1"))?;
    let cap_input = fs::canonicalize(evidence.join("final-emission-1/attempt-1"))?;
    let older_input = fs::canonicalize(evidence.join("loss-frontier-1/joint-1"))?;
    let corpus = fs::canonicalize(std::env::var("UOR_TREE_PAIR_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_TREE_PAIR_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_TREE_PAIR_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let roots = [&input, &prior_input, &cap_input, &older_input, &corpus];
    for root in roots {
        if out.starts_with(root) {
            return Err("output beneath sealed input".into());
        }
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        let start = Instant::now();
        for root in roots {
            crate::report_output::verify(root)?;
        }
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design size".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        let expected = json!({"schema":"uor-r4.tree-state-pair-design/1","parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"corpus_sha256":"541a094ae83a56adbcabb0cd157f0a7a14aef23b6db29f82f2d794975d420adf","indices":INDICES,"root_choices":120,"enumerated_settings":14400,"selection":"Lowest complete causal construction mean NLL; ties within 1e-10 of global minimum prefer unchanged parent, then lexicographic roots. Opened and disabled-control labels excluded.","experiment_seconds":90,"construction_documents":12,"construction_positions":672,"opened_positions":381,"generation_prompts":4,"generation_bytes":96,"witness":"Source-bound test-only parameter witness; no standard changed-model artifact or inherited candidate CID.","decoder_refits":0,"gates":{"construction_loss_strictly_improves":true,"construction_accuracy_retained":true,"correct_to_wrong_each_prior_max":0,"opened_full_loss_nonincreasing":true,"opened_full_accuracy_retained":true},"promotion":false});
        if design != expected {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let mut prior_bytes = Vec::new();
        let mut priors = Vec::new();
        for (root, cid) in [
            (&input, PARENT),
            (&prior_input, PRIOR_TREE),
            (&cap_input, CAP),
            (&older_input, OLDER),
        ] {
            let bytes = fs::read(root.join("candidate.json"))?;
            let model = SharedCore::from_bytes(&bytes)?;
            if model.artifact_cid() != cid || model.to_bytes()? != bytes {
                return Err("prior artifact identity".into());
            }
            prior_bytes.push(bytes);
            priors.push(model);
        }
        let parent = &priors[0];
        let previous_source: Value = serde_json::from_slice(&fs::read(input.join("source.json"))?)?;
        if previous_source["candidate"] != PARENT
            || previous_source["prior_tree"] != PRIOR_TREE
            || previous_source["parent"] != CAP
            || previous_source["older"] != OLDER
            || previous_source["implementation"] != SharedCore::implementation_digest()
        {
            return Err("decoder source bindings".into());
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
            return Err("corpus sizes".into());
        }
        let parent_roots = INDICES.map(|i| parent.artifact.parameters[i]);
        let parent_index = usize::from(parent_roots[0]) * ROOTS + usize::from(parent_roots[1]);
        let mut scoring = parent.clone();
        let mut costs = Vec::with_capacity(14400);
        let mut correct_counts = Vec::with_capacity(14400);
        for first in 0..ROOTS {
            for second in 0..ROOTS {
                guard(start)?;
                scoring.artifact.parameters[INDICES[0]] = first as u16;
                scoring.artifact.parameters[INDICES[1]] = second as u16;
                // Every call rebuilds all causal trajectories from each document's
                // initial state. Saved state rows never substitute for recurrence.
                let metric = scoring.evaluate(&training, Intervention::Full)?;
                if metric.positions != 672 || !metric.mean_nll.is_finite() {
                    return Err("oracle metric bounds".into());
                }
                costs.push(metric.mean_nll);
                correct_counts.push(metric.correct);
            }
        }
        require_only_pair_changed(parent, &scoring)?;
        guard(start)?;
        let selected = select(&costs, parent_index)?;
        let selected_roots = [(selected / ROOTS) as u16, (selected % ROOTS) as u16];
        let minimum = costs.iter().copied().fold(f64::INFINITY, f64::min);
        let next_best = costs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != selected)
            .map(|(_, v)| *v)
            .fold(f64::INFINITY, f64::min);
        let next_strict = costs
            .iter()
            .copied()
            .filter(|v| *v > minimum + EPS)
            .reduce(f64::min);
        let improving = costs
            .iter()
            .filter(|&&v| v + EPS < costs[parent_index])
            .count();
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        let witness = Witness {
            schema: "uor-r4.tree-state-pair-witness/1".into(),
            parent: PARENT.into(),
            implementation: SharedCore::implementation_digest(),
            driver: driver_digest(),
            design: design_digest.clone(),
            corpus_design: corpus_digest.clone(),
            indices: INDICES,
            selected_roots,
            evaluated_settings: costs.len(),
            parent_nll: costs[parent_index],
            selected_nll: costs[selected],
        };
        let witness_bytes = serde_json::to_vec_pretty(&witness)?;
        let witness_id = digest(&witness_bytes);
        fs::File::create_new(out.join("witness.json"))?.write_all(&witness_bytes)?;
        write(
            &out,
            "oracle.json",
            &json!({"indices":INDICES,"parent_roots":parent_roots,"selected_roots":selected_roots,"settings":costs.len(),"minimum_nll":minimum,"selected_nll":costs[selected],"parent_nll":costs[parent_index],"next_best_excluding_selected_nll":next_best,"next_nll_strictly_above_minimum_tolerance":next_strict,"settings_within_minimum_tolerance":costs.iter().filter(|&&v|v<=minimum+EPS).count(),"strictly_improving_settings":improving,"costs_row_major":costs,"correct_counts_row_major":correct_counts,"construction_positions":672,"all_causal_trajectories_recomputed":true,"opened_and_disabled_labels_used":false,"decoder_refits":0}),
        )?;
        let replayed: Witness = serde_json::from_slice(&witness_bytes)?;
        let candidate = replayed.replay(parent, &design_digest, &corpus_digest)?;
        let early = generate(&candidate, &prompts, &[Intervention::Full])?;
        write(
            &out,
            "early-generation.json",
            &json!({"witness":witness_id,"outputs":early}),
        )?;
        guard(start)?;
        // Selection is fixed and the witness is exported before any opened or
        // disabled-control labels are inspected.
        let previous: Value = serde_json::from_slice(&fs::read(input.join("behavior.json"))?)?;
        let mut behavior = Vec::new();
        let mut lost = [0_u64; 4];
        let mut gained = [0_u64; 4];
        let mut root_hamming = [0_u64; 4];
        let mut source_hamming = [0_u64; 4];
        let mut prediction_hamming = [0_u64; 4];
        let mut metrics = Vec::new();
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                guard(start)?;
                let changed = rows(&candidate, docs, control)?;
                let after = verified_metrics(&candidate, docs, control, &changed)?;
                let saved_panel = previous["panels"]
                    .as_array()
                    .ok_or("prior panels")?
                    .iter()
                    .find(|v| {
                        v["panel"] == name
                            && v["control"] == serde_json::to_value(control).unwrap_or(Value::Null)
                    })
                    .ok_or("prior panel")?;
                let mut comparisons = Vec::new();
                for (index, model) in priors.iter().enumerate() {
                    let base = rows(model, docs, control)?;
                    let before = verified_metrics(model, docs, control, &base)?;
                    let saved = if index == 0 {
                        json!({"metrics":saved_panel["candidate_metrics"],"rows":saved_panel["candidate_rows"]})
                    } else {
                        saved_panel["comparisons"]
                            .as_array()
                            .ok_or("prior comparisons")?
                            .iter()
                            .find(|v| v["parent"] == model.artifact_cid())
                            .ok_or("prior comparison")?
                            .clone()
                    };
                    saved_rows_equal(&saved, &base, &before)?;
                    let comparison = compare(&base, &changed)?;
                    lost[index] += comparison["correct_to_wrong"]
                        .as_u64()
                        .ok_or("lost count")?;
                    gained[index] += comparison["wrong_to_correct"]
                        .as_u64()
                        .ok_or("gained count")?;
                    root_hamming[index] += comparison["root_identity_hamming"]
                        .as_u64()
                        .ok_or("root count")?;
                    source_hamming[index] += comparison["source_identity_hamming"]
                        .as_u64()
                        .ok_or("source count")?;
                    prediction_hamming[index] += comparison["prediction_hamming"]
                        .as_u64()
                        .ok_or("prediction count")?;
                    if control == Intervention::Full {
                        metrics.push(json!({"panel":name,"parent":model.artifact_cid(),"before":before,"after":after}));
                    }
                    comparisons.push(json!({"parent":model.artifact_cid(),"metrics":before,"rows":base,"comparison":comparison}));
                }
                behavior.push(json!({"panel":name,"control":control,"witness":witness_id,"witness_metrics":after,"witness_rows":changed,"comparisons":comparisons}));
            }
        }
        let mut generations = Vec::new();
        for model in &priors {
            let outputs = generate(model, &prompts, &CONTROLS)?;
            let saved = previous["generations"]
                .as_array()
                .ok_or("saved generations")?
                .iter()
                .find(|v| v["artifact"] == model.artifact_cid())
                .ok_or("saved artifact generation")?;
            if serde_json::to_value(&outputs)? != saved["outputs"] {
                return Err("prior generation changed".into());
            }
            generations.push(json!({"artifact":model.artifact_cid(),"outputs":outputs}));
        }
        let outputs = generate(&candidate, &prompts, &CONTROLS)?;
        for (i, first) in early.iter().enumerate() {
            if first != &outputs[i * 3] {
                return Err("early generation replay".into());
            }
        }
        generations.push(json!({"witness":witness_id,"outputs":outputs}));
        let construction = metrics
            .iter()
            .find(|m| m["panel"] == "complete_construction" && m["parent"] == PARENT)
            .ok_or("construction metrics")?;
        let before: Metrics = serde_json::from_value(construction["before"].clone())?;
        let after: Metrics = serde_json::from_value(construction["after"].clone())?;
        if (before.mean_nll - replayed.parent_nll).abs() > EPS
            || (after.mean_nll - replayed.selected_nll).abs() > EPS
            || before.correct != correct_counts[parent_index]
            || after.correct != correct_counts[selected]
        {
            return Err("oracle/witness metric replay".into());
        }
        let optimization =
            after.mean_nll + EPS < before.mean_nll && after.correct >= before.correct;
        let preservation = lost == [0, 0, 0, 0];
        let mut development = true;
        for item in metrics
            .iter()
            .filter(|m| m["panel"] == "opened_development")
        {
            let base: Metrics = serde_json::from_value(item["before"].clone())?;
            let changed: Metrics = serde_json::from_value(item["after"].clone())?;
            development &=
                changed.mean_nll <= base.mean_nll + EPS && changed.correct >= base.correct;
        }
        guard(start)?;
        let mut session = candidate.session(Intervention::Full);
        let mut max_comparisons = 0;
        let mut census_predictions = Vec::with_capacity(512);
        for step in 0..512 {
            guard(start)?;
            let before = session.work();
            let prediction = session.predict();
            let comparisons = session.work().products - before.products;
            if comparisons > 27 {
                return Err("witness prediction angular comparison bound".into());
            }
            max_comparisons = max_comparisons.max(comparisons);
            census_predictions.push(prediction);
            session.observe((step & 255) as u8)?;
        }
        write(
            &out,
            "runtime-census.json",
            &json!({"witness":witness_id,"steps":512,"max_angular_comparisons_per_predict":max_comparisons,"bound":27,"work":session.work(),"predictions":census_predictions,"allocation_measurement":"NOT_RUN_IN_THIS_DRIVER"}),
        )?;
        write(
            &out,
            "behavior.json",
            &json!({"panels":behavior,"generations":generations}),
        )?;
        write(
            &out,
            "source.json",
            &json!({"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"tree_source":digest(include_bytes!("../angular_tree.rs")),"design":design_digest,"corpus":corpus_digest,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"witness":witness_id,"prior_artifact_sha256":prior_bytes.iter().map(|b|format!("{:x}",sha2::Sha256::digest(b))).collect::<Vec<_>>(),"prior_report_roots":[input,prior_input,cap_input,older_input],"standard_changed_model_artifact_exported":false}),
        )?;
        for (model, bytes) in priors.iter().zip(&prior_bytes) {
            if model.to_bytes()? != *bytes {
                return Err("prior mutated".into());
            }
        }
        require_only_pair_changed(parent, &candidate)?;
        for root in roots {
            crate::report_output::verify(root)?;
        }
        guard(start)?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.tree-state-pair-result/1","decision":if optimization&&preservation&&development{"PASS_TREE_STATE_PAIR_DEVELOPMENT_GATE"}else{"FAIL_TREE_STATE_PAIR_DEVELOPMENT_GATE"},"witness":witness_id,"parent":PARENT,"prior_tree":PRIOR_TREE,"cap":CAP,"older":OLDER,"selected_roots":selected_roots,"parent_roots":parent_roots,"enumerated_settings":14400,"strictly_improving_settings":improving,"minimum_nll":minimum,"parent_is_selected":selected==parent_index,"conditional_optimization_gate":optimization,"preservation_gate":preservation,"opened_development_gate":development,"correct_to_wrong_each_prior":lost,"wrong_to_correct_each_prior":gained,"root_identity_hamming_each_prior":root_hamming,"source_identity_hamming_each_prior":source_hamming,"prediction_hamming_each_prior":prediction_hamming,"metrics":metrics,"before":before,"after":after,"elapsed_ms":start.elapsed().as_millis(),"only_selected_parameter_indices_mutable":true,"changed_parameter_count":parent.artifact.parameters.iter().zip(&candidate.artifact.parameters).filter(|(a,b)|a!=b).count(),"decoder_unchanged":true,"decoder_refits":0,"promoted":false,"scope":"Exact two-parameter conditional construction search under fixed angular decoder, with separately retained prior controls and actual generation. Test-only source-bound witness; no standard changed artifact, fresh holdout, decoder refit, promotion, language or energy qualification."}),
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
