//! Test-only, full-construction comparison. Parameter witnesses are not model artifacts.
use super::super::tied_training::{search, SearchMode, SearchReport};
use super::*;
use serde_json::{json, Value};
use sha2::Digest as _;
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const INDICES: [usize; 2] = [1176, 1930];
const SEEDS: [u64; 4] = [7341, 7342, 7343, 7344];
const EPS: f64 = 1e-10;
const CONTROLS: [Intervention; 3] = [
    Intervention::Full,
    Intervention::ContextDisabled,
    Intervention::StateDisabled,
];
fn config(seed: u64) -> TiedFitConfig {
    TiedFitConfig {
        indices: INDICES,
        seed,
        batches: 32,
        batch_size: 64,
        step_size: 40.0,
        max_seconds: 120,
    }
}
fn digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}
fn driver_digest() -> String {
    digest(include_bytes!("full_objective.rs"))
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
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    schema: String,
    parent: String,
    implementation: String,
    driver: String,
    design: String,
    corpus_design: String,
    mode: String,
    config: TiedFitConfig,
    selected_roots: [u16; 2],
    search: SearchReport,
}
impl Witness {
    fn replay(&self, parent: &SharedCore, design: &str, corpus: &str) -> RunResult<SharedCore> {
        self.config.validate()?;
        if self.schema != "uor-r4.test-only-parameter-witness/1"
            || self.parent != parent.artifact_cid()
            || self.implementation != SharedCore::implementation_digest()
            || self.driver != driver_digest()
            || self.design != design
            || self.corpus_design != corpus
            || !SEEDS.contains(&self.config.seed)
            || serde_json::to_value(self.config)? != serde_json::to_value(config(self.config.seed))?
            || ![
                "fixed_initial_distribution_joint",
                "greedy_uniform_coordinate",
            ]
            .contains(&self.mode.as_str())
            || self.search.mode != self.mode
            || self.search.calls != 2050
            || self.search.batches != 32
            || self.search.best_roots != self.selected_roots
            || self.selected_roots.iter().any(|&r| usize::from(r) >= ROOTS)
            || !self.search.before_cost.is_finite()
            || !self.search.after_cost.is_finite()
        {
            return Err("invalid or unbound parameter witness".into());
        }
        let mut model = parent.clone();
        for (index, root) in INDICES.into_iter().zip(self.selected_roots) {
            model.artifact.parameters[index] = root;
        }
        // This clone is used in memory only. Its inherited CID/provenance is never
        // presented as identifying the changed model or serialized as an artifact.
        Ok(model)
    }
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
    let (mut lost, mut gained, mut prediction_hamming, mut source_hamming, mut root_hamming) =
        (0, 0, 0, 0, 0);
    let mut row_deltas = Vec::with_capacity(base.len());
    for (a, b) in base.iter().zip(changed) {
        if (a.document, a.position, a.target) != (b.document, b.position, b.target) {
            return Err("row alignment differs".into());
        }
        let correct_to_wrong = a.prediction == a.target && b.prediction != b.target;
        let wrong_to_correct = a.prediction != a.target && b.prediction == b.target;
        lost += usize::from(correct_to_wrong);
        gained += usize::from(wrong_to_correct);
        prediction_hamming += usize::from(a.prediction != b.prediction);
        source_hamming += usize::from(a.source != b.source);
        root_hamming += a
            .roots
            .iter()
            .zip(b.roots)
            .filter(|(x, y)| **x != *y)
            .count();
        row_deltas.push(json!({"document":a.document,"position":a.position,"target":a.target,
            "parent_prediction":a.prediction,"witness_prediction":b.prediction,"parent_nll":a.nll,"witness_nll":b.nll,
            "nll_delta":b.nll-a.nll,"correct_to_wrong":correct_to_wrong,"wrong_to_correct":wrong_to_correct}));
    }
    Ok(
        json!({"positions":base.len(),"correct_to_wrong":lost,"wrong_to_correct":gained,
        "prediction_hamming":prediction_hamming,"source_identity_hamming":source_hamming,"root_identity_hamming":root_hamming,
        "hamming_scope":"Aligned categorical identities, never bit Hamming or a geometric metric.","rows":row_deltas}),
    )
}
fn generate(model: &SharedCore, prompts: &[String]) -> Result<Vec<Value>> {
    let mut outputs = Vec::new();
    for prompt in prompts {
        for control in CONTROLS {
            let (bytes, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
            outputs.push(json!({"prompt":prompt,"control":control,"bytes":bytes,"text":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
        }
    }
    Ok(outputs)
}
#[test]
fn full_objective_witness_binds_parent_config_and_replays_only_selected_parameters() {
    let parent = model().upgrade_calibrated().unwrap();
    let original = parent.to_bytes().unwrap();
    let cfg = config(SEEDS[0]);
    let report = search([119, 119], cfg, SearchMode::Fixed, |r| {
        Ok(f64::from(r[0]) + f64::from(r[1]))
    })
    .unwrap();
    let witness = Witness {
        schema: "uor-r4.test-only-parameter-witness/1".into(),
        parent: parent.artifact_cid().into(),
        implementation: SharedCore::implementation_digest(),
        driver: driver_digest(),
        design: "design".into(),
        corpus_design: "corpus".into(),
        mode: report.mode.clone(),
        config: cfg,
        selected_roots: report.best_roots,
        search: report,
    };
    let loaded: Witness = serde_json::from_slice(&serde_json::to_vec(&witness).unwrap()).unwrap();
    let replay = loaded.replay(&parent, "design", "corpus").unwrap();
    let mut direct = parent.clone();
    for (i, r) in INDICES.into_iter().zip(loaded.selected_roots) {
        direct.artifact.parameters[i] = r;
    }
    for (i, (&a, &b)) in parent
        .artifact
        .parameters
        .iter()
        .zip(&replay.artifact.parameters)
        .enumerate()
    {
        if !INDICES.contains(&i) {
            assert_eq!(a, b);
        }
    }
    assert_eq!(
        direct.generate(b"ab", 24, Intervention::Full).unwrap(),
        replay.generate(b"ab", 24, Intervention::Full).unwrap()
    );
    assert_eq!(parent.to_bytes().unwrap(), original);
    let mut bad = loaded.clone();
    bad.parent = "wrong".into();
    assert!(bad.replay(&parent, "design", "corpus").is_err());
    let mut bad = loaded.clone();
    bad.selected_roots[0] = ROOTS as u16;
    bad.search.best_roots = bad.selected_roots;
    assert!(bad.replay(&parent, "design", "corpus").is_err());
    let mut bad = loaded.clone();
    bad.config.indices = [1176, 1176];
    assert!(bad.replay(&parent, "design", "corpus").is_err());
    assert!(loaded.replay(&parent, "changed", "corpus").is_err());
    let docs = vec![b"aba".to_vec()];
    let baseline = rows(&parent, &docs, Intervention::Full).unwrap();
    assert_eq!(baseline.len(), 4);
    assert_eq!(baseline[3].target, EOS);
    let mut altered = baseline.clone();
    altered[0].prediction = (baseline[0].prediction + 1) % 257;
    let comparison = compare(&baseline, &altered).unwrap();
    assert_eq!(comparison["prediction_hamming"], 1);
    let mut known = baseline.clone();
    known[0].prediction = known[0].target;
    known[1].prediction = (known[1].target + 1) % 257;
    let mut changed = known.clone();
    changed[0].prediction = (changed[0].target + 1) % 257;
    changed[1].prediction = changed[1].target;
    changed[0].roots[0] = (changed[0].roots[0] + 1) % 120;
    changed[0].source = Some(999);
    let counted = compare(&known, &changed).unwrap();
    assert_eq!(counted["correct_to_wrong"], 1);
    assert_eq!(counted["wrong_to_correct"], 1);
    assert_eq!(counted["root_identity_hamming"], 1);
    assert_eq!(counted["source_identity_hamming"], 1);
    altered[0].position = 3;
    assert!(compare(&baseline, &altered).is_err());
}
#[test]
#[ignore = "requires sealed parent/corpus, frozen full-objective design and exclusive output"]
fn full_objective_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_FULL_INPUT")?)?;
    let corpus = fs::canonicalize(std::env::var("UOR_FULL_CORPUS")?)?;
    let path = fs::canonicalize(std::env::var("UOR_FULL_DESIGN")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_FULL_OUTPUT")?);
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
        if design["schema"] != "uor-r4.full-objective-experiment/1"
            || design["indices"] != json!(INDICES)
            || design["seeds"] != json!(SEEDS)
            || design["config"]
                != json!({"indices":INDICES,"batches":32,"batch_size":64,"step_size":40,"max_seconds":120})
            || design["calls_per_arm"] != 2050
            || design["oracle_settings"] != 14400
            || design["construction_positions"] != 672
            || design["opened_positions"] != 381
            || design["gate"]
                != json!({"fixed_beats_coordinate_seeds_at_least":3,"fixed_accuracy_retained_and_loss_improves_seeds_at_least":3})
            || design["witness_gate"]
                != json!({"full_construction_loss_strictly_improves":true,"correct_to_wrong_rows_across_all_panels_max":0})
        {
            return Err("frozen design mismatch".into());
        }
        fs::File::create_new(out.join("design.json"))?.write_all(&design_bytes)?;
        let raw_parent = fs::read(input.join("candidate.json"))?;
        let parent = SharedCore::from_bytes(&raw_parent)?;
        if parent.to_bytes()? != raw_parent
            || design["parent"] != parent.artifact_cid()
            || parent.artifact_cid()
                != "blake3:ade9a1cfbf39f0828ec1ec2b53775c05fcbaaf53fa2189b6a37ab7a636ce5e58"
        {
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
        let design_digest = digest(&design_bytes);
        let corpus_digest = digest(&corpus_bytes);
        let initial = [
            parent.artifact.parameters[INDICES[0]],
            parent.artifact.parameters[INDICES[1]],
        ];
        let before = parent.evaluate(&training, Intervention::Full)?;
        let mut witnesses = Vec::new();
        let mut replayed = Vec::new();
        let mut arms = Vec::new();
        for seed in SEEDS {
            for mode in [SearchMode::Fixed, SearchMode::Coordinate] {
                let mut scoring = parent.clone();
                let report = search(initial, config(seed), mode, |roots| {
                    if start.elapsed().as_secs() >= 120 {
                        return Err(CoreError::InvalidInput("complete experiment time guard"));
                    }
                    for (i, r) in INDICES.into_iter().zip(roots) {
                        scoring.artifact.parameters[i] = r;
                    }
                    Ok(scoring.evaluate(&training, Intervention::Full)?.mean_nll)
                })?;
                if report.calls != 2050 || (report.before_cost - before.mean_nll).abs() > EPS {
                    return Err("search call count mismatch".into());
                }
                let witness = Witness {
                    schema: "uor-r4.test-only-parameter-witness/1".into(),
                    parent: parent.artifact_cid().into(),
                    implementation: SharedCore::implementation_digest(),
                    driver: driver_digest(),
                    design: design_digest.clone(),
                    corpus_design: corpus_digest.clone(),
                    mode: report.mode.clone(),
                    config: config(seed),
                    selected_roots: report.best_roots,
                    search: report,
                };
                let bytes = serde_json::to_vec_pretty(&witness)?;
                let witness_id = digest(&bytes);
                let loaded: Witness = serde_json::from_slice(&bytes)?;
                let model = loaded.replay(&parent, &design_digest, &corpus_digest)?;
                let metrics = model.evaluate(&training, Intervention::Full)?;
                if (metrics.mean_nll - loaded.search.after_cost).abs() > EPS {
                    return Err("witness replay cost mismatch".into());
                }
                let name = format!("witness-{seed}-{}.json", loaded.mode);
                fs::File::create_new(out.join(&name))?.write_all(&bytes)?;
                let (bytes, eos, work) =
                    model.generate(prompts[0].as_bytes(), 96, Intervention::Full)?;
                write(
                    &out,
                    &format!("early-generation-{seed}-{}.json", loaded.mode),
                    &json!({"witness":witness_id,"prompt":prompts[0],"bytes":bytes,"text":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}),
                )?;
                arms.push(json!({"seed":seed,"mode":loaded.mode,"witness":witness_id,"witness_file":name,"metrics":metrics,"search":loaded.search}));
                witnesses.push((witness_id, loaded));
                replayed.push(model);
            }
        }
        // All eight search arms are complete. Only now open the full pair oracle.
        let mut oracle_model = parent.clone();
        let mut costs = Vec::with_capacity(ROOTS * ROOTS);
        let mut best = f64::INFINITY;
        let mut best_pair = [0_u16; 2];
        let mut improving = 0;
        for a in 0..ROOTS {
            for b in 0..ROOTS {
                if start.elapsed().as_secs() >= 120 {
                    return Err("oracle experiment time guard".into());
                }
                oracle_model.artifact.parameters[INDICES[0]] = a as u16;
                oracle_model.artifact.parameters[INDICES[1]] = b as u16;
                let cost = oracle_model
                    .evaluate(&training, Intervention::Full)?
                    .mean_nll;
                if cost < best {
                    best = cost;
                    best_pair = [a as u16, b as u16];
                }
                improving += usize::from(cost + EPS < before.mean_nll);
                costs.push(cost);
            }
        }
        let mut fixed_wins = 0;
        let mut fixed_retained_improved = 0;
        for pair in arms.chunks_exact(2) {
            let fixed: Metrics = serde_json::from_value(pair[0]["metrics"].clone())?;
            let coordinate: Metrics = serde_json::from_value(pair[1]["metrics"].clone())?;
            fixed_wins += usize::from(fixed.mean_nll + EPS < coordinate.mean_nll);
            fixed_retained_improved += usize::from(
                fixed.correct >= before.correct && fixed.mean_nll + EPS < before.mean_nll,
            );
        }
        for (_, w) in &witnesses {
            let pair = w.selected_roots;
            if (costs[usize::from(pair[0]) * ROOTS + usize::from(pair[1])] - w.search.after_cost)
                .abs()
                > EPS
            {
                return Err("oracle selected witness mismatch".into());
            }
        }
        let mut base_rows = Vec::new();
        let mut parent_panels = Vec::new();
        for (name, docs) in [
            ("complete_construction", &training),
            ("opened_development", &opened),
        ] {
            for control in CONTROLS {
                let r = rows(&parent, docs, control)?;
                parent_panels.push(json!({"panel":name,"control":control,"metrics":parent.evaluate(docs,control)?,"rows":r}));
                base_rows.push(r);
            }
        }
        let mut behavior = vec![
            json!({"parent_artifact":parent.artifact_cid(),"panels":parent_panels,"generation":generate(&parent,&prompts)?}),
        ];
        for (index, ((id, w), model)) in witnesses.iter().zip(&replayed).enumerate() {
            let mut panels = Vec::new();
            let mut regressions = 0_u64;
            let mut base_index = 0;
            for (name, docs) in [
                ("complete_construction", &training),
                ("opened_development", &opened),
            ] {
                for control in CONTROLS {
                    let r = rows(model, docs, control)?;
                    let comparison = compare(&base_rows[base_index], &r)?;
                    base_index += 1;
                    regressions += comparison["correct_to_wrong"]
                        .as_u64()
                        .ok_or("regression count")?;
                    let metrics = model.evaluate(docs, control)?;
                    let row_mean = r.iter().map(|r| r.nll).sum::<f64>() / r.len() as f64;
                    let row_correct = r.iter().filter(|r| r.prediction == r.target).count();
                    if (row_mean - metrics.mean_nll).abs() > EPS || row_correct != metrics.correct {
                        return Err("row/evaluate mismatch".into());
                    }
                    panels.push(json!({"panel":name,"control":control,"metrics":metrics,"comparison_to_parent":comparison}));
                }
            }
            let eligible = w.search.after_cost + EPS < before.mean_nll && regressions == 0;
            arms[index]["comparison_eligible"] = json!(eligible);
            arms[index]["correct_to_wrong_across_panels"] = json!(regressions);
            arms[index]["oracle_gain_fraction"] = if before.mean_nll > best + EPS {
                json!((before.mean_nll - w.search.after_cost) / (before.mean_nll - best))
            } else {
                Value::Null
            };
            behavior.push(json!({"witness":id,"mode":w.mode,"seed":w.config.seed,"panels":panels,"generation":generate(model,&prompts)?,"comparison_eligible":eligible,"promoted":false}));
        }
        if start.elapsed().as_secs() >= 120 {
            return Err("complete experiment time guard".into());
        }
        if parent.to_bytes()? != raw_parent {
            return Err("parent mutated".into());
        }
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&corpus)?;
        write(
            &out,
            "oracle.json",
            &json!({"indices":INDICES,"before":before.mean_nll,"best":best,"best_pair":best_pair,"strictly_improving_settings":improving,"settings":14400,"costs_row_major":costs,"opened_after_all_arms":true,"installed":false}),
        )?;
        write(&out, "behavior.json", &behavior)?;
        write(
            &out,
            "source.json",
            &json!({"parent_cid":parent.artifact_cid(),"parent_implementation":parent.artifact.implementation,"implementation":SharedCore::implementation_digest(),"driver":driver_digest(),"design":design_digest,"corpus_design":corpus_digest,"search_source":digest(include_bytes!("../tied_training.rs"))}),
        )?;
        if start.elapsed().as_secs() >= 120 {
            return Err("report-writing experiment time guard".into());
        }
        let pass = fixed_wins >= 3 && fixed_retained_improved >= 3;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.full-objective-result/1","decision":if pass{"PASS_FULL_OBJECTIVE_JOINT_METHOD_SMOKE"}else{"FAIL_FULL_OBJECTIVE_JOINT_METHOD_SMOKE"},"parent":parent.artifact_cid(),"before":before,"arms":arms,"fixed_beats_coordinate_seeds":fixed_wins,"fixed_accuracy_retained_and_loss_improves_seeds":fixed_retained_improved,"sampled_model_calls":8*2050,"oracle_model_calls":14400,"elapsed_ms":start.elapsed().as_millis(),"promoted":false,"scope":"Two-parameter full construction selection, previously opened development and retained parent row comparisons. Test-only witnesses; no new standard model artifact, fresh holdout, full-model fit, language or energy qualification."}),
        )?;
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
