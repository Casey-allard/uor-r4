use super::{
    data, learning,
    runtime::{self, Control},
};
use crate::{
    native_geometric::{
        adaptive_attention::{self, runtime as adaptive},
        addressed_attention::artifact::BoundGeometry,
        dependent_attention,
        hamming_refinement::metric::Metric,
        relational_attention::{self, runtime as read},
    },
    report_output,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};
fn write(p: &Path, n: &str, v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(p.join(n), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn target(answer: &[u8]) -> Vec<u16> {
    answer.iter().map(|&b| u16::from(b)).chain([256]).collect()
}
#[test]
#[ignore = "bounded actual text fit; exclusive report and pinned parent required"]
fn text_attention_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_TEXT_REPORT")?);
    let parent_path = std::path::PathBuf::from(std::env::var("UOR_TEXT_PARENT")?);
    let one_path = std::path::PathBuf::from(std::env::var("UOR_TEXT_PRIOR_ONE")?);
    let adaptive_path = std::path::PathBuf::from(std::env::var("UOR_TEXT_PRIOR_ADAPTIVE")?);
    if [&path, &parent_path, &one_path, &adaptive_path]
        .iter()
        .any(|p| p.as_os_str().is_empty())
    {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let config = learning::Config::default();
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_FIT","development_total":64,"exact_min":61,"pairs_both_min":31,"longer_than_training_fraction_min":0.95,"improvement_over_initial_min":32,"each_disabled_or_reversed_max":0,"new_text_loop_prior_one_required":8256,"historical_adaptive_required":2240,"configuration":config,"scope":"grounded span text generation, supplied key/boundary, alternating teacher-position output-directed training; not novel prose or end-to-end gradients","final_holdout":"NOT_RUN"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(&parent_path)?;
        let parent_sha = hex::encode(sha2::Sha256::digest(&bytes));
        if parent_sha != "f0e1f2c35a68e6228777c1a662a2e6b00a398957444f29fc3e4f2f9b8d473d54" {
            return Err("parent hash".into());
        }
        let parent = adaptive::Artifact::decode(&bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate(&train, &dev)?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        let initial = learning::initialize(parent, &train)?;
        let mut sources = 0;
        for e in train.iter().chain(&dev) {
            sources += usize::from(
                runtime::select(&initial, &g, &m, &e.records, e.query, false)?
                    .is_some_and(|i| e.records[i].text == e.answer),
            );
        }
        write(
            &path,
            "environment.json",
            &json!({"correct_available_sources":sources,"total":train.len()+dev.len(),"pass":sources==train.len()+dev.len(),"exact_span_boundaries_supplied":true,"canonical_query_encoding_fixed":true,"state":"integer cursor with learned hold/advance after actual non-EOS event","not_content_conditioned_semantic_state":true}),
        )?;
        if sources != train.len() + dev.len() {
            return Err("prefit source failure".into());
        }
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, config)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let reloaded = runtime::Artifact::decode(&encoded, &g)?;
        let mut panels = Vec::new();
        let mut rows = Vec::new();
        let mut full = 0;
        let mut baseline = 0;
        let mut controls_ok = true;
        let mut pairs = BTreeMap::new();
        let mut long_total = 0;
        let mut long_correct = 0;
        let max_train = train
            .iter()
            .map(|e| e.answer.len())
            .max()
            .ok_or("emptytrain")?;
        let mut replay = 0;
        for (split, examples) in [("training", &train), ("development", &dev)] {
            for (name, a) in [("initial", &initial), ("candidate", &candidate)] {
                for control in [
                    Control::Full,
                    Control::ReadDisabled,
                    Control::FeedbackDisabled,
                    Control::PayloadDisabled,
                    Control::QueryReversed,
                    Control::EosDisabled,
                ] {
                    if name == "initial" && control != Control::Full {
                        continue;
                    }
                    let mut exact = 0;
                    let mut exhausted = 0;
                    let mut symbols_correct = 0;
                    let mut symbols_total = 0;
                    for e in examples {
                        let out = runtime::generate(a, &g, &m, &e.records, e.query, control)?;
                        let expected = target(&e.answer);
                        let yes = out.tokens == expected;
                        exact += usize::from(yes);
                        exhausted += usize::from(out.exhausted);
                        symbols_correct += out
                            .tokens
                            .iter()
                            .zip(&expected)
                            .filter(|(x, y)| x == y)
                            .count();
                        symbols_total += expected.len();
                        if name == "candidate" && control == Control::Full {
                            if out
                                != runtime::generate(
                                    &reloaded, &g, &m, &e.records, e.query, control,
                                )?
                            {
                                return Err("reload".into());
                            }
                            replay += 1;
                            if split == "development" {
                                if yes {
                                    *pairs.entry(e.family.clone()).or_insert(0usize) += 1;
                                }
                                if e.answer.len() > max_train {
                                    long_total += 1;
                                    long_correct += usize::from(yes);
                                }
                            }
                        }
                        rows.push(json!({"split":split,"artifact":name,"control":control,"id":e.id,"family":e.family,"query":e.query,"answer":String::from_utf8_lossy(&e.answer),"actual_text":String::from_utf8_lossy(&out.tokens.iter().filter_map(|&b|u8::try_from(b).ok()).collect::<Vec<_>>()),"actual":out,"exact":yes}));
                    }
                    panels.push(json!({"split":split,"artifact":name,"control":control,"exact":exact,"total":examples.len(),"exhausted":exhausted,"aligned_symbols_correct":symbols_correct,"target_symbols":symbols_total}));
                    if split == "development" {
                        if name == "initial" {
                            baseline = exact
                        } else if control == Control::Full {
                            full = exact
                        } else {
                            controls_ok &= exact == 0;
                        }
                    }
                }
            }
        }
        let old: Value = serde_json::from_slice(&std::fs::read(&one_path)?)?;
        let mut one_ok = 0;
        let mut one_total = 0;
        for split in ["training", "development"] {
            let examples: Vec<relational_attention::data::Example> =
                serde_json::from_value(old[split].clone())?;
            if examples.len() > 8192 {
                return Err("prior size".into());
            }
            for e in examples {
                let records = std::array::from_fn(|i| data::Record {
                    key: e.records[i].key,
                    text: vec![e.records[i].value],
                });
                let out = runtime::generate(&candidate, &g, &m, &records, e.query, Control::Full)?;
                one_ok += usize::from(out.tokens == [u16::from(e.answer), 256]);
                one_total += 1;
            }
        }
        let old: Value = serde_json::from_slice(&std::fs::read(&adaptive_path)?)?;
        let mut adaptive_ok = 0;
        let mut adaptive_total = 0;
        for split in ["training", "development"] {
            let examples: Vec<adaptive_attention::data::Example> =
                serde_json::from_value(old[split].clone())?;
            if examples.len() > 2048 {
                return Err("prior size".into());
            }
            for e in examples {
                let out = adaptive::generate(
                    &candidate.parent,
                    &g,
                    &m,
                    &e.records,
                    e.query,
                    adaptive::Control::Full,
                )?;
                adaptive_ok += usize::from(
                    out.tokens == [u16::from(e.answer), 256] && out.steps.len() == e.depth,
                );
                adaptive_total += 1;
            }
        }
        let both = pairs.values().filter(|&&n| n == 2).count();
        let pass = full >= 61
            && both >= 31
            && full >= baseline + 32
            && controls_ok
            && long_total > 0
            && long_correct * 100 >= long_total * 95
            && one_ok == 8256
            && one_total == 8256
            && adaptive_ok == 2240
            && adaptive_total == 2240
            && candidate.parent == initial.parent
            && fit.stopped == "completed_dose"
            && fit.epochs == fit.config.epochs;
        std::fs::write(path.join("responses.json"), serde_json::to_vec(&rows)?)?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_GROUNDED_VARIABLE_TEXT"}else{"FAIL_GROUNDED_VARIABLE_TEXT"},"panels":panels,"development_pairs_both_exact":both,"max_training_bytes":max_train,"long_development_correct":long_correct,"long_development_total":long_total,"candidate_sha256":hex::encode(sha2::Sha256::digest(&encoded)),"parent_sha256":parent_sha,"parent_unchanged":candidate.parent==initial.parent,"changed_writer_tables":candidate.tables.iter().zip(&initial.tables).filter(|(x,y)|x!=y).count(),"changed_advance_entries":candidate.advance.iter().zip(&initial.advance).filter(|(x,y)|x!=y).count(),"changed_reader_tables":candidate.reader.score_tables.iter().zip(&initial.reader.score_tables).filter(|(x,y)|x!=y).count(),"advance":candidate.advance,"new_text_prior_one_correct":one_ok,"new_text_prior_one_total":one_total,"historical_adaptive_correct":adaptive_ok,"historical_adaptive_total":adaptive_total,"reload_equal_rows":replay,"retention_scope":"new text loop adapts prior one-byte spans; historical adaptive mode retained separately, not claimed as text-loop multi-hop support","final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
fn text_artifact_state_bounds_and_training_shape() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    let (reader, _, _) = relational_attention::learning::initialized(&g, &[])?;
    let (dep, _) = dependent_attention::learning::initialize(reader, &[])?;
    let adaptive = adaptive_attention::learning::initialize(dep, &[])?;
    let a = learning::initialize(adaptive, &[])?;
    a.validate(&g)?;
    assert_eq!(a, runtime::Artifact::decode(&a.encode()?, &g)?);
    let records = std::array::from_fn(|_| data::Record {
        key: [1, 2],
        text: b"ab".to_vec(),
    });
    let out = runtime::generate(&a, &g, &m, &records, [1, 2], Control::Full)?;
    assert!(out.exhausted);
    assert_eq!(out.steps.len(), runtime::MAX_SYMBOLS);
    assert!(out.steps.iter().all(|s| s.cursor == 0));
    let mut bad = a.clone();
    bad.advance[0] = 2;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.writer.nodes[0].inputs = [8, 0];
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.parent.emit[0] ^= 1;
    assert!(bad.validate(&g).is_err());
    let (train, _) = data::corpus()?;
    let mut bad = learning::initialize(a.parent, &train)?;
    bad.reader.scorer.nodes = vec![relational_attention::circuit::Node { inputs: [0, 0] }];
    bad.reader.scorer.outputs = vec![read::FEATURES];
    bad.reader.score_tables = vec![0];
    bad.reader.validate(&g)?;
    assert!(learning::fit(&bad, &g, &m, &train, learning::Config::default()).is_err());
    Ok(())
}
