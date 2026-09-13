use super::{
    data, learning,
    runtime::{self, Control},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry,
        hamming_refinement::metric::Metric,
        relational_attention::{self, runtime as read},
    },
    report_output,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};
fn write(path: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
#[test]
#[ignore = "bounded actual dependent-query fit; requires exclusive report and pinned reader paths"]
fn dependent_attention_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_DEPENDENT_ATTENTION_REPORT")?);
    let reader_path = std::path::PathBuf::from(std::env::var("UOR_DEPENDENT_READER")?);
    let prior_data = std::path::PathBuf::from(std::env::var("UOR_DEPENDENT_PRIOR_DATA")?);
    if path.as_os_str().is_empty()
        || reader_path.as_os_str().is_empty()
        || prior_data.as_os_str().is_empty()
    {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let config = learning::Config::default();
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_FIT","development_total":128,"exact_min":122,"second_selection_min":122,"next_query_min":122,"first_selection_required":128,"pairs_both_exact_min":61,"improvement_over_initial_min":64,"restricted_controls_max":32,"retained_one_read_required":8256,"configuration":config,"scope":"learned query update under frozen reader and codec, typed composition curriculum; not joint recurrent language learning","final_holdout":"NOT_RUN"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let reader_bytes = std::fs::read(&reader_path)?;
        let reader_sha = hex::encode(sha2::Sha256::digest(&reader_bytes));
        if reader_sha != "9901cc50d36d32b4c41f24db2c5f3af41b7cb1f2eaa77ff6e7dcde339e66a582" {
            return Err("unexpected reader identity".into());
        }
        let reader = read::Artifact::decode(&reader_bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate(&train, &dev)?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        // Verify availability and the frozen reader before spending any training dose.
        let mut first_correct = 0;
        let mut next_correct = 0;
        let mut unique_suffix = 0;
        for e in train.iter().chain(&dev) {
            let first = read::generate(&reader, &g, &m, &e.records, e.query, read::Control::Full)?;
            first_correct +=
                usize::from(first.selected.is_some_and(|i| e.records[i].key == e.query));
            let second = read::generate(
                &reader,
                &g,
                &m,
                &e.records,
                e.next_query,
                read::Control::Full,
            )?;
            next_correct += usize::from(
                second.tokens == [u16::from(e.answer), 256]
                    && second
                        .selected
                        .is_some_and(|i| e.records[i].key == e.next_query),
            );
            let mut useful = 0;
            for r in &e.records {
                useful += usize::from(
                    read::generate(&reader, &g, &m, &e.records, r.key, read::Control::Full)?.tokens
                        == [u16::from(e.answer), 256],
                );
            }
            unique_suffix += usize::from(useful == 1);
        }
        let total = train.len() + dev.len();
        let environment_ok =
            first_correct == total && next_correct == total && unique_suffix == total;
        write(
            &path,
            "environment.json",
            &json!({"total":total,"first_correct":first_correct,"available_correct_suffix":next_correct,"unique_successful_suffix":unique_suffix,"pass":environment_ok,"typed_records":true,"next_query_is_offline_oracle_only":true,"candidate_admission":4,"learner_target":"unique actual generated suffix answer match; no transition-oracle call"}),
        )?;
        if !environment_ok {
            return Err("dependent environment failed before fit".into());
        }
        let (initial, _) = learning::initialize(reader, &train)?;
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, config)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let reloaded = runtime::Artifact::decode(&encoded, &g)?;
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut dev_full = 0;
        let mut dev_initial = 0;
        let mut selected_dev = 0;
        let mut query_dev = 0;
        let mut first_dev = 0;
        let mut controls_ok = true;
        let mut pairs = BTreeMap::new();
        let mut replayed = 0;
        for (split, examples) in [("training", &train), ("development", &dev)] {
            for (name, a) in [("initial", &initial), ("candidate", &candidate)] {
                for control in [
                    Control::Full,
                    Control::UpdateDisabled,
                    Control::PayloadDisabled,
                    Control::PriorQueryDisabled,
                    Control::ReadDisabled,
                    Control::SecondReadDisabled,
                ] {
                    if name == "initial" && control != Control::Full {
                        continue;
                    }
                    let mut exact = 0;
                    let mut first_selected = 0;
                    let mut second_selected = 0;
                    let mut query_exact = 0;
                    for e in examples {
                        let out = runtime::generate(a, &g, &m, &e.records, e.query, control)?;
                        let correct = out.second.tokens == [u16::from(e.answer), 256];
                        exact += usize::from(correct);
                        first_selected += usize::from(
                            out.first
                                .selected
                                .is_some_and(|i| e.records[i].key == e.query),
                        );
                        second_selected += usize::from(
                            out.second
                                .selected
                                .is_some_and(|i| e.records[i].key == e.next_query),
                        );
                        query_exact += usize::from(out.next_query == e.next_query);
                        if name == "candidate" && control == Control::Full {
                            if out
                                != runtime::generate(
                                    &reloaded, &g, &m, &e.records, e.query, control,
                                )?
                            {
                                return Err("reload divergence".into());
                            }
                            replayed += 1;
                            if split == "development" && correct {
                                *pairs.entry(e.family.clone()).or_insert(0usize) += 1;
                            }
                        }
                        rows.push(json!({"split":split,"artifact":name,"control":control,"id":e.id,"family":e.family,"query":e.query,"expected_next_query":e.next_query,"answer":e.answer,"first_selected":out.first.selected,"first_scores":out.first.scores,"next_query":out.next_query,"second_selected":out.second.selected,"second_scores":out.second.scores,"tokens":out.second.tokens,"exact":correct}));
                    }
                    panels.push(json!({"split":split,"artifact":name,"control":control,"exact":exact,"first_selected":first_selected,"second_selected":second_selected,"query_exact":query_exact,"total":examples.len()}));
                    if split == "development" {
                        if name == "initial" {
                            dev_initial = exact;
                        } else if control == Control::Full {
                            dev_full = exact;
                            first_dev = first_selected;
                            selected_dev = second_selected;
                            query_dev = query_exact;
                        } else {
                            controls_ok &= exact <= 32;
                        }
                    }
                }
            }
        }
        // Replay the retained learned one-read task, not historical memory-repair panels.
        let prior: Value = serde_json::from_slice(&std::fs::read(&prior_data)?)?;
        let mut retained = 0;
        let mut retained_total = 0;
        for split in ["training", "development"] {
            let examples: Vec<relational_attention::data::Example> =
                serde_json::from_value(prior[split].clone())?;
            if examples.len() > 8192 {
                return Err("unexpected prior size".into());
            }
            for e in examples {
                let a = read::generate(
                    &candidate.reader,
                    &g,
                    &m,
                    &e.records,
                    e.query,
                    read::Control::Full,
                )?;
                retained += usize::from(
                    a.tokens == [u16::from(e.answer), 256]
                        && a.selected.is_some_and(|i| e.records[i].key == e.query),
                );
                retained_total += 1;
            }
        }
        let pair_count = pairs.values().filter(|&&n| n == 2).count();
        let pass = dev_full >= 122
            && selected_dev >= 122
            && query_dev >= 122
            && first_dev == 128
            && pair_count >= 61
            && dev_full >= dev_initial + 64
            && controls_ok
            && retained == 8256
            && retained_total == 8256
            && fit.stopped == "completed_dose"
            && fit.completed_epochs == fit.config.epochs
            && candidate.reader == initial.reader;
        std::fs::write(path.join("responses.json"), serde_json::to_vec(&rows)?)?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_TYPED_DEPENDENT_QUERY_LEARNING"}else{"FAIL_TYPED_DEPENDENT_QUERY_LEARNING"},"panels":panels,"development_pairs_both_exact":pair_count,"reader_sha256":reader_sha,"reader_unchanged":candidate.reader==initial.reader,"candidate_sha256":hex::encode(sha2::Sha256::digest(&encoded)),"changed_update_tables":candidate.tables.iter().zip(&initial.tables).filter(|(a,b)|a!=b).count(),"tables":candidate.tables,"reload_equal_rows":replayed,"retained_one_read_correct":retained,"retained_one_read_total":retained_total,"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48","source":{"runtime":blake3::hash(include_bytes!("runtime.rs")).to_hex().to_string(),"learning":blake3::hash(include_bytes!("learning.rs")).to_hex().to_string(),"data":blake3::hash(include_bytes!("data.rs")).to_hex().to_string()}}),
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
fn update_artifact_rejects_corruption() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let (train, _) = relational_attention::data::corpus()?;
    let (reader, _, _) = relational_attention::learning::initialized(&g, &train)?;
    let (a, _) = learning::initialize(reader, &[])?;
    a.validate(&g)?;
    assert_eq!(a, runtime::Artifact::decode(&a.encode()?, &g)?);
    let mut bad = a.clone();
    bad.reader.score_tables[0] ^= 1;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.tables[0] = 255;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.update.nodes[0].inputs[0] = 24;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.update
        .nodes
        .push(relational_attention::circuit::Node { inputs: [0, 1] });
    bad.tables.push(0);
    assert!(bad.update.validate().is_ok());
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.update.nodes[1].inputs[0] = 24;
    assert!(bad.update.validate().is_ok());
    assert!(bad.validate(&g).is_err());
    let (examples, _) = data::corpus()?;
    let m = Metric::new(&g)?;
    assert!(learning::fit(&a, &g, &m, &examples[..1], learning::Config::default()).is_err());
    let mut bad = a;
    bad.update.outputs.pop();
    assert!(bad.validate(&g).is_err());
    Ok(())
}
