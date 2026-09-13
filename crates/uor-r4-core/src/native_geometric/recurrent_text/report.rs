use super::{
    data::{self, Example, Record},
    learning,
    runtime::{self, Action, Control},
};
use crate::{
    native_geometric::{
        adaptive_attention, addressed_attention::artifact::BoundGeometry, dependent_attention,
        hamming_refinement::metric::Metric, relational_attention, text_attention,
    },
    report_output,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};
fn write(root: &Path, name: &str, v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
fn old_bytes(v: &Value, label: &str) -> Result<Vec<Example>, Box<dyn std::error::Error>> {
    v.as_array()
        .ok_or("old array")?
        .iter()
        .map(|x| {
            let r: [relational_attention::data::Record; 4] =
                serde_json::from_value(x["records"].clone())?;
            Ok(Example {
                id: format!("{label}:{}", x["id"].as_str().ok_or("old id")?),
                family: format!("{label}:{}", x["family"].as_str().ok_or("old family")?),
                query: serde_json::from_value(x["query"].clone())?,
                answer: vec![serde_json::from_value(x["answer"].clone())?],
                records: r.map(|r| Record {
                    key: r.key,
                    text: vec![r.value],
                }),
            })
        })
        .collect()
}
fn old_text(v: &Value, label: &str) -> Result<Vec<Example>, Box<dyn std::error::Error>> {
    v.as_array()
        .ok_or("old array")?
        .iter()
        .map(|x| {
            Ok(Example {
                id: format!("{label}:{}", x["id"].as_str().ok_or("old id")?),
                family: format!("{label}:{}", x["family"].as_str().ok_or("old family")?),
                query: serde_json::from_value(x["query"].clone())?,
                answer: serde_json::from_value(x["answer"].clone())?,
                records: serde_json::from_value(x["records"].clone())?,
            })
        })
        .collect()
}
#[test]
#[ignore = "bounded recurrent integration fit; exclusive report and pinned evidence required"]
fn recurrent_text_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_RECURRENT_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_RECURRENT_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","development_total":64,"exact_min":61,"complete_pairs_min":31,"each_depth_fraction_min":0.95,"correct_new_source_paths_min":61,"long_control_exact_max":0,"retention_all_required":true,"training_dose_epochs":1,"rate":0.1,"fit_wall_seconds":300,"search_states_per_example":512,"scope":"shared action learning across supplied typed bytes and multi-span text; retained learned primitives frozen","final_holdout":"NOT_RUN"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(evidence.join("text-attention-1/attempt-1/candidate.json"))?;
        let parent_sha = hex::encode(sha2::Sha256::digest(&bytes));
        if parent_sha != "22b1df1c6b5721ea71d0bad40ab968434b63397c7819213aec046c7def6af63e" {
            return Err("parent hash".into());
        }
        let parent = text_attention::runtime::Artifact::decode(&bytes, &g)?;
        let (new_train, dev) = data::corpus()?;
        let validation = data::validate(&new_train, &dev)?;
        let mut train = new_train.clone();
        let mut retained = Vec::new();
        for (name, rel, text) in [
            ("one", "relational-attention-1/attempt-4/data.json", false),
            (
                "dependent",
                "dependent-attention-1/attempt-1/data.json",
                false,
            ),
            (
                "adaptive",
                "adaptive-attention-1/attempt-1/data.json",
                false,
            ),
            ("text", "text-attention-1/attempt-1/data.json", true),
        ] {
            let v: Value = serde_json::from_slice(&std::fs::read(evidence.join(rel))?)?;
            let mut all = Vec::new();
            for split in ["training", "development"] {
                let examples = if text {
                    old_text(&v[split], name)?
                } else {
                    old_bytes(&v[split], name)?
                };
                if split == "training" {
                    train.extend(examples.clone());
                }
                all.extend(examples);
            }
            retained.push((name, all));
        }
        write(
            &path,
            "data.json",
            &json!({"new_training":new_train,"development":dev,"joint_training":train,"validation":validation,"retained_counts":retained.iter().map(|(n,e)|json!({"name":n,"rows":e.len()})).collect::<Vec<_>>(),"adapter":"old byte records become one-byte spans without class tags or output overrides"}),
        )?;
        let initial = learning::initialize(parent, &train)?;
        let prepared = learning::prepare(&initial, &g, &m, &train)?;
        let dev_prepared = learning::prepare(&initial, &g, &m, &dev)?;
        let unseen: Vec<_> = dev_prepared
            .counts
            .iter()
            .enumerate()
            .filter(|(i, c)| {
                c.iter().any(|&v| v > 0) && prepared.counts[*i].iter().all(|&v| v == 0)
            })
            .map(|(i, _)| i)
            .collect();
        write(
            &path,
            "preparation.json",
            &json!({"training":prepared,"development":dev_prepared,"unseen_development_policy_rows":unseen,"development_labels_excluded_from_fit":true,"credit":"successful bounded output-constrained trajectories through actual writer/update/geometric reads; no expected source/depth labels"}),
        )?;
        if !prepared.missing.is_empty()
            || !prepared.conflicts.is_empty()
            || !dev_prepared.missing.is_empty()
            || !unseen.is_empty()
        {
            write(
                &path,
                "summary.json",
                &json!({"gate":"FAIL_RECURRENT_REPRESENTATION_PREFIT","fit":"NOT_RUN","training_missing":prepared.missing.len(),"training_conflicts":prepared.conflicts.len(),"development_missing":dev_prepared.missing.len(),"unseen_development_rows":unseen,"promotion":false,"retained_model":"15baec48"}),
            )?;
            return Ok(());
        }
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, &prepared)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let reload = runtime::Artifact::decode(&encoded, &g)?;
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut full = 0;
        let mut both = BTreeMap::new();
        let mut depth = BTreeMap::<usize, (usize, usize)>::new();
        let mut paths_correct = 0;
        let mut controls_ok = true;
        let mut replays = 0;
        for (name, a, control) in [
            ("initial", &initial, Control::Full),
            ("candidate", &candidate, Control::Full),
            ("candidate", &candidate, Control::ReadDisabled),
            ("candidate", &candidate, Control::FeedbackDisabled),
            ("candidate", &candidate, Control::UpdateDisabled),
            ("candidate", &candidate, Control::CursorDisabled),
            ("candidate", &candidate, Control::BoundaryReadDisabled),
            ("candidate", &candidate, Control::QueryReversed),
            ("candidate", &candidate, Control::StopDisabled),
        ] {
            let mut exact = 0;
            let mut long_exact = 0;
            let mut exhausted = 0;
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, e.query, control)?;
                let yes = out.tokens == target(e) && !out.exhausted;
                let expected_path = data::trace(e)?;
                exact += usize::from(yes);
                exhausted += usize::from(out.exhausted);
                if expected_path.len() > 1 {
                    long_exact += usize::from(yes);
                }
                if name == "candidate" && control == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, e.query, control)? {
                        return Err("reload mismatch".into());
                    }
                    replays += 1;
                    if yes {
                        *both.entry(&e.family).or_insert(0) += 1;
                    }
                    let d = depth.entry(expected_path.len()).or_default();
                    d.0 += usize::from(yes);
                    d.1 += 1;
                    let actual_path: Vec<_> = out
                        .steps
                        .iter()
                        .filter(|s| s.before.cursor == 0)
                        .filter_map(|s| s.observation.selected)
                        .collect();
                    paths_correct += usize::from(actual_path == expected_path);
                }
                rows.push(json!({"id":e.id,"family":e.family,"artifact":name,"control":control,"answer":String::from_utf8_lossy(&e.answer),"actual_text":String::from_utf8_lossy(&out.tokens.iter().filter_map(|&t|u8::try_from(t).ok()).collect::<Vec<_>>()),"exact":yes,"expected_source_path":expected_path,"actual":out}));
            }
            panels.push(json!({"artifact":name,"control":control,"exact":exact,"total":dev.len(),"long_exact":long_exact,"exhausted":exhausted}));
            if name == "candidate" && control == Control::Full {
                full = exact;
            } else if name == "candidate" {
                controls_ok &= long_exact == 0;
            }
        }
        let mut retention = Vec::new();
        let mut retained_ok = true;
        let mut retained_rows = Vec::new();
        for (name, examples) in &retained {
            let mut exact = 0;
            for e in examples {
                let out =
                    runtime::generate(&candidate, &g, &m, &e.records, e.query, Control::Full)?;
                let yes = out.tokens == target(e) && !out.exhausted;
                exact += usize::from(yes);
                retained_rows.push(json!({"panel":name,"id":e.id,"actual":out.tokens,"expected":target(e),"exact":yes}));
            }
            retained_ok &= exact == examples.len();
            retention.push(json!({"name":name,"exact":exact,"total":examples.len(),"path":"same recurrent runtime; no legacy dispatch"}));
        }
        let pairs = both.values().filter(|&&v| v == 2).count();
        let depth_ok = depth.values().all(|&(ok, n)| ok * 100 >= n * 95);
        let pass = full >= 61
            && pairs >= 31
            && paths_correct >= 61
            && depth_ok
            && controls_ok
            && retained_ok
            && fit.stopped == "completed_dose";
        write(&path, "responses.json", &json!(rows))?;
        write(&path, "retained-responses.json", &json!(retained_rows))?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_RECURRENT_GROUNDED_TEXT"}else{"FAIL_RECURRENT_GROUNDED_TEXT"},"panels":panels,"development_pairs_both_exact":pairs,"depths":depth,"correct_source_paths":paths_correct,"retention":retention,"reload_equal_rows":replays,"candidate_sha256":hex::encode(sha2::Sha256::digest(&encoded)),"parent_sha256":parent_sha,"parent_unchanged":candidate.parent==initial.parent,"changed_action_rows":fit.changed_rows,"training_policy_rows":prepared.rows,"unseen_development_rows":unseen,"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
        )?;
        Ok(())
    })();
    if let Err(e) = &result {
        std::fs::write(path.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&path)?;
    report_output::verify(&path)?;
    result
}
#[test]
fn actual_emission_updates_pending_query_and_export_rejects_corruption(
) -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let (reader, _, _) = relational_attention::learning::initialized(&g, &[])?;
    let (mut dep, _) = dependent_attention::learning::initialize(reader, &[])?;
    dep.tables.fill(relational_attention::circuit::XOR);
    let adaptive = adaptive_attention::learning::initialize(dep, &[])?;
    let mut text = text_attention::learning::initialize(adaptive, &[])?;
    text.tables = vec![10; 9];
    text.tables[8] = 3;
    text.advance[3] = 1;
    let a = learning::initialize(text, &[])?;
    a.validate(&g)?;
    assert_eq!(a, runtime::Artifact::decode(&a.encode()?, &g)?);
    let o = runtime::Observation {
        selected: Some(0),
        byte: b'x',
        present: true,
        next_query: [0, 0],
        next_available: false,
        row: 0,
    };
    let mut full = runtime::State::new([13, 21]);
    let mut disabled = full.clone();
    assert_eq!(
        runtime::execute(&a, &o, &mut full, Action::Emit, Control::Full)?,
        Some(u16::from(b'x'))
    );
    runtime::execute(
        &a,
        &o,
        &mut disabled,
        Action::Emit,
        Control::FeedbackDisabled,
    )?;
    assert_eq!(full.pending, [13 ^ b'x', 21 ^ b'x']);
    assert_eq!(disabled.pending, [13, 21]);
    assert_eq!(full.cursor, 1);
    assert_eq!(full.query, [13, 21]);
    let mut bad = a.clone();
    bad.actions[0] = 3;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.parent.advance[3] ^= 1;
    assert!(bad.validate(&g).is_err());
    full.reads = runtime::MAX_READS;
    runtime::execute(&a, &o, &mut full, Action::Read, Control::Full)?;
    assert!(full.exhausted);
    assert!(!full.done);
    assert!(Action::from_byte(3).is_err());
    Ok(())
}
