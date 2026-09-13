//! Small tied-parameter learning experiment; the oracle is opened after all arms.
use super::super::tied_training::{search, SearchMode};
use super::*;
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path, time::Instant};
type RunResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn config(seed: u64) -> TiedFitConfig {
    TiedFitConfig {
        indices: [1176, 1930],
        seed,
        batches: 32,
        batch_size: 64,
        step_size: 40.0,
        max_seconds: 120,
    }
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
fn generate(model: &SharedCore, prompts: &[String]) -> Result<Vec<Value>> {
    let mut outputs = Vec::new();
    for prompt in prompts {
        for control in [
            Intervention::Full,
            Intervention::ContextDisabled,
            Intervention::StateDisabled,
        ] {
            let (bytes, eos, work) = model.generate(prompt.as_bytes(), 96, control)?;
            outputs.push(json!({"prompt":prompt,"control":control,"bytes":bytes,
                "text":String::from_utf8_lossy(&bytes),"eos":eos,"work":work}));
        }
    }
    Ok(outputs)
}
fn expectation(probabilities: &[Vec<f64>; 2], costs: &[f64]) -> f64 {
    let mut result = 0.0;
    for a in 0..ROOTS {
        for b in 0..ROOTS {
            result += probabilities[0][a] * probabilities[1][b] * costs[a * ROOTS + b];
        }
    }
    result
}

#[test]
fn tied_artifact_roundtrip_and_exclusive_training_metadata() {
    let parent = model().upgrade_calibrated().unwrap();
    let before = parent.to_bytes().unwrap();
    let mut cfg = config(41);
    cfg.batches = 2;
    cfg.batch_size = 4;
    let docs = vec![b"ababa".to_vec()];
    let (candidate, report) = parent.fit_tied(&docs, cfg).unwrap();
    assert_eq!(parent.to_bytes().unwrap(), before);
    assert_eq!(report.search.calls, 10);
    assert!(report.after.mean_nll <= report.before.mean_nll + 1e-12);
    let bytes = candidate.to_bytes().unwrap();
    let loaded = SharedCore::from_bytes(&bytes).unwrap();
    assert_eq!(loaded.to_bytes().unwrap(), bytes);
    assert_eq!(loaded.artifact_cid(), candidate.artifact_cid());
    for (i, (&a, &b)) in parent
        .artifact
        .parameters
        .iter()
        .zip(&loaded.artifact.parameters)
        .enumerate()
    {
        if !cfg.indices.contains(&i) {
            assert_eq!(a, b);
        }
    }
    assert_eq!(parent.artifact.geometry, loaded.artifact.geometry);
    assert_eq!(
        serde_json::to_value(&parent.artifact.calibrated).unwrap(),
        serde_json::to_value(&loaded.artifact.calibrated).unwrap()
    );
    assert_eq!(
        loaded.generate(b"ab", 12, Intervention::Full).unwrap(),
        candidate.generate(b"ab", 12, Intervention::Full).unwrap()
    );
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["fit_config"] = json!({"seed":1,"max_proposals":1,"max_seconds":1});
    assert!(SharedCore::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    value["fit_config"] = Value::Null;
    value["tied_fit_config"]["indices"] = json!([1176, 1176]);
    assert!(SharedCore::from_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    // Existing training replaces provenance, never leaves two learner configs.
    let (old_method, _) = candidate
        .fit(
            &docs,
            FitConfig {
                seed: 3,
                max_proposals: 1,
                max_seconds: 1,
            },
        )
        .unwrap();
    assert!(old_method.artifact.tied_fit_config.is_none());
    assert!(SharedCore::from_bytes(&old_method.to_bytes().unwrap()).is_ok());
}

#[test]
fn tied_search_counts_real_callbacks_for_all_arms() {
    for mode in [
        SearchMode::Learned,
        SearchMode::Fixed,
        SearchMode::Coordinate,
    ] {
        let mut cfg = config(19);
        cfg.batches = 2;
        cfg.batch_size = 4;
        let mut calls = 0;
        let report = search([119, 119], cfg, mode, |roots| {
            calls += 1;
            Ok(f64::from(roots[0]) + f64::from(roots[1]))
        })
        .unwrap();
        assert_eq!(calls, 10);
        assert_eq!(report.calls, calls);
        assert!(report.after_cost <= report.before_cost);
    }
}

#[test]
#[ignore = "requires sealed actual parent, frozen design and exclusive output"]
fn tied_run_saved_experiment() -> RunResult<()> {
    let input = fs::canonicalize(std::env::var("UOR_TIED_INPUT")?)?;
    let raw = std::path::PathBuf::from(std::env::var("UOR_TIED_OUTPUT")?);
    let out = fs::canonicalize(raw.parent().ok_or("output parent")?)?
        .join(raw.file_name().ok_or("output name")?);
    let path = fs::canonicalize(std::env::var("UOR_TIED_DESIGN")?)?;
    let corpus = fs::canonicalize(std::env::var("UOR_TIED_CORPUS")?)?;
    if out.starts_with(&input) || out.starts_with(&corpus) {
        return Err("output beneath sealed input".into());
    }
    crate::report_output::claim(&out)?;
    let result = (|| -> RunResult<()> {
        crate::report_output::verify(&input)?;
        crate::report_output::verify(&corpus)?;
        if fs::metadata(&path)?.len() > 65536 {
            return Err("design too large".into());
        }
        let design_bytes = fs::read(&path)?;
        let design: Value = serde_json::from_slice(&design_bytes)?;
        if design["schema"] != "uor-r4.tied-categorical-experiment/1"
            || design["indices"] != json!([1176, 1930])
            || design["seeds"] != json!([7341, 7342, 7343, 7344])
            || design["config"]
                != json!({"indices":[1176,1930],"batches":32,"batch_size":64,"step_size":40,"max_seconds":120})
            || design["calls_per_arm"] != 2050
            || design["oracle_settings"] != 14400
            || design["gate"]
                != json!({"distribution_expected_loss_improves_seeds_at_least":3,"beats_each_baseline_seeds_at_least":3,"mean_oracle_gain_fraction_at_least":0.5})
        {
            return Err("frozen design mismatch".into());
        }
        write(&out, "design.json", &design)?;
        let raw_parent = fs::read(input.join("candidate.json"))?;
        let parent = SharedCore::from_bytes(&raw_parent)?;
        if parent.to_bytes()? != raw_parent || design["parent"] != parent.artifact_cid() {
            return Err("parent byte/CID mismatch".into());
        }
        let docs = documents(&design["documents"])?;
        if docs.len() != 4 || docs.iter().any(|d| d.len() != 32) {
            return Err("small block data bound".into());
        }
        let original: Value = serde_json::from_slice(&fs::read(corpus.join("design.json"))?)?;
        let training = documents(&original["training"])?;
        let opened = documents(&original["holdout"])?;
        let expected: Vec<_> = [0, 3, 7, 9]
            .iter()
            .map(|&i| {
                training
                    .get(i)
                    .and_then(|d| d.get(..32))
                    .map(|d| d.to_vec())
                    .ok_or("construction prefix missing")
            })
            .collect::<std::result::Result<_, _>>()?;
        if docs != expected {
            return Err("data changed from construction prefixes".into());
        }
        let prompts: Vec<String> = serde_json::from_value(original["prompts"].clone())?;
        if prompts.len() != 4 {
            return Err("prompt count".into());
        }
        let initial = [
            parent.artifact.parameters[1176],
            parent.artifact.parameters[1930],
        ];
        let mut arms = Vec::new();
        let mut candidates = Vec::new();
        let start = Instant::now();
        for seed in [7341, 7342, 7343, 7344] {
            let cfg = config(seed);
            let (candidate, learned) = parent.fit_tied(&docs, cfg)?;
            if learned.search.calls != 2050 {
                return Err("learned call budget".into());
            }
            let bytes = candidate.to_bytes()?;
            let loaded = SharedCore::from_bytes(&bytes)?;
            if bytes != loaded.to_bytes()? {
                return Err("candidate roundtrip".into());
            }
            fs::File::create_new(out.join(format!("candidate-{seed}.json")))?.write_all(&bytes)?;
            // Actual early generation immediately follows construction/roundtrip.
            let early = loaded.generate(prompts[0].as_bytes(), 96, Intervention::Full)?;
            write(
                &out,
                &format!("early-generation-{seed}.json"),
                &json!({"bytes":early.0,"eos":early.1,"work":early.2}),
            )?;
            let mut baseline_model = parent.clone();
            let mut baseline = |roots: [u16; 2]| {
                baseline_model.artifact.parameters[1176] = roots[0];
                baseline_model.artifact.parameters[1930] = roots[1];
                Ok(baseline_model.evaluate(&docs, Intervention::Full)?.mean_nll)
            };
            let fixed = search(initial, cfg, SearchMode::Fixed, &mut baseline)?;
            let coordinate = search(initial, cfg, SearchMode::Coordinate, &mut baseline)?;
            if fixed.calls != 2050 || coordinate.calls != 2050 {
                return Err("baseline call budget".into());
            }
            arms.push(json!({"seed":seed,"candidate":candidate.artifact_cid(),"learned":learned,"fixed":fixed,"coordinate":coordinate}));
            candidates.push(loaded);
            if start.elapsed().as_secs() > 100 {
                return Err("experiment time guard before oracle".into());
            }
        }
        // All learners and comparators are finished before any joint oracle loss.
        let mut oracle_model = parent.clone();
        let mut costs = Vec::with_capacity(ROOTS * ROOTS);
        for a in 0..ROOTS {
            for b in 0..ROOTS {
                oracle_model.artifact.parameters[1176] = a as u16;
                oracle_model.artifact.parameters[1930] = b as u16;
                costs.push(oracle_model.evaluate(&docs, Intervention::Full)?.mean_nll);
            }
            if start.elapsed().as_secs() > 110 {
                return Err("experiment oracle time guard".into());
            }
        }
        let before = parent.evaluate(&docs, Intervention::Full)?.mean_nll;
        let best = costs.iter().copied().fold(f64::INFINITY, f64::min);
        let mut gains = Vec::new();
        let (mut distribution_wins, mut fixed_wins, mut coordinate_wins) = (0, 0, 0);
        for arm in &mut arms {
            let search = &arm["learned"]["search"];
            let probabilities: [Vec<f64>; 2] =
                serde_json::from_value(search["final_probabilities"].clone())?;
            let initial_p: [Vec<f64>; 2] =
                serde_json::from_value(search["initial_probabilities"].clone())?;
            let expected_before = expectation(&initial_p, &costs);
            let expected_after = expectation(&probabilities, &costs);
            let selected = search["after_cost"].as_f64().ok_or("learned loss")?;
            let roots: [u16; 2] = serde_json::from_value(search["best_roots"].clone())?;
            if (selected - costs[usize::from(roots[0]) * ROOTS + usize::from(roots[1])]).abs()
                > 1e-10
            {
                return Err("selected/oracle witness mismatch".into());
            }
            let fw = selected + 1e-10 < arm["fixed"]["after_cost"].as_f64().ok_or("fixed loss")?;
            let cw = selected + 1e-10
                < arm["coordinate"]["after_cost"]
                    .as_f64()
                    .ok_or("coordinate loss")?;
            distribution_wins += usize::from(expected_after + 1e-10 < expected_before);
            fixed_wins += usize::from(fw);
            coordinate_wins += usize::from(cw);
            let gain = if before > best + 1e-10 {
                Some((before - selected) / (before - best))
            } else {
                None
            };
            if let Some(g) = gain {
                gains.push(g);
            }
            arm["assessment"] = json!({"expected_loss_before":expected_before,"expected_loss_after":expected_after,"beats_fixed":fw,"beats_coordinate":cw,"oracle_gain_fraction":gain});
        }
        let fraction = if gains.len() == 4 {
            Some(gains.iter().sum::<f64>() / 4.0)
        } else {
            None
        };
        let pass = distribution_wins >= 3
            && fixed_wins >= 3
            && coordinate_wins >= 3
            && fraction.is_some_and(|x| x >= 0.5);
        let mut behavior = Vec::new();
        for model in std::iter::once(&parent).chain(&candidates) {
            let mut panels = Vec::new();
            for (name, docs) in [
                ("complete_construction", &training),
                ("opened_development", &opened),
            ] {
                for control in [
                    Intervention::Full,
                    Intervention::ContextDisabled,
                    Intervention::StateDisabled,
                ] {
                    panels.push(json!({"panel":name,"control":control,"metrics":model.evaluate(docs,control)?}));
                }
            }
            behavior.push(json!({"artifact":model.artifact_cid(),"panels":panels,"generation":generate(model,&prompts)?}));
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
            &json!({"indices":[1176,1930],"costs_row_major":costs,"before":before,"best":best,"settings":14400,"opened_after_all_arms":true}),
        )?;
        write(&out, "behavior.json", &behavior)?;
        write(
            &out,
            "source.json",
            &json!({"parent_cid":parent.artifact_cid(),"design_blake3":blake3::hash(&design_bytes).to_hex().to_string(),"learner_source_blake3":blake3::hash(include_bytes!("../tied_training.rs")).to_hex().to_string(),"driver_source_blake3":blake3::hash(include_bytes!("tied_learning.rs")).to_hex().to_string()}),
        )?;
        write(
            &out,
            "result.json",
            &json!({"schema":"uor-r4.tied-categorical-result/1","decision":if pass {"PASS_TIED_CATEGORICAL_METHOD_SMOKE"}else{"FAIL_TIED_CATEGORICAL_METHOD_SMOKE"},"arms":arms,"distribution_improving_seeds":distribution_wins,"beats_fixed_seeds":fixed_wins,"beats_coordinate_seeds":coordinate_wins,"mean_oracle_gain_fraction":fraction,"elapsed_ms":start.elapsed().as_millis(),"sampled_model_calls":4*3*2050,"oracle_model_calls":14400,"promoted":false,"scope":"Two-parameter development learner; no fresh draw, full-model learning, language or energy qualification."}),
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
