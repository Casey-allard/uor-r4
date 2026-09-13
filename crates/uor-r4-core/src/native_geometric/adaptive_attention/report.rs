use super::{
    data, learning,
    runtime::{self, Control, MAX_READS},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry,
        dependent_attention::{self, runtime as dependent},
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
#[test]
#[ignore = "single bounded mixed-depth fit; requires exclusive report and pinned parent"]
fn adaptive_attention_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_ADAPTIVE_REPORT")?);
    let parent_path = std::path::PathBuf::from(std::env::var("UOR_ADAPTIVE_PARENT")?);
    let dep_data = std::path::PathBuf::from(std::env::var("UOR_ADAPTIVE_DEPENDENT_DATA")?);
    let one_data = std::path::PathBuf::from(std::env::var("UOR_ADAPTIVE_ONE_READ_DATA")?);
    if [&path, &parent_path, &dep_data, &one_data]
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
            &json!({"status":"FROZEN_BEFORE_FIT","development_total":192,"exact_min":183,"correct_depth_min":183,"pairs_both_min":91,"per_depth_fraction_min":0.95,"improvement_over_initial_min":96,"improvement_over_always_emit_min":64,"fixed_two_max":96,"always_read_max":0,"read_disabled_max":0,"long_update_disabled_max":0,"long_payload_disabled_max":0,"retained_one_read_required":8256,"retained_dependent_required":2176,"retention_scope":"both frozen parent paths and new adaptive loop must reproduce all prior one/two-read outputs","configuration":config,"final_holdout":"NOT_RUN","scope":"content-action table learning under frozen reader/update/codec; typed byte-domain, not prose or joint learning"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(&parent_path)?;
        let parent_sha = hex::encode(sha2::Sha256::digest(&bytes));
        if parent_sha != "dcbca3d77bc7db8a30d962ac85450e821fa92f8d2bf5d8228c8c897f0d6e1d9d" {
            return Err("parent identity".into());
        }
        let parent = dependent::Artifact::decode(&bytes, &g)?;
        let (train, dev) = data::corpus()?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":data::validate(&train,&dev)?}),
        )?;
        let mut correct = 0;
        let mut visited_train = [false; 256];
        let mut visited_dev = [false; 256];
        let mut correct_sources = 0;
        let mut reads = 0;
        for (split, examples) in [(0, &train), (1, &dev)] {
            for e in examples {
                let mut query = e.query;
                for hop in 1..=MAX_READS {
                    let out = read::generate(
                        &parent.reader,
                        &g,
                        &m,
                        &e.records,
                        query,
                        read::Control::Full,
                    )?;
                    let slot = out.selected.ok_or("no source")?;
                    correct_sources += usize::from(e.records[slot].key == query);
                    reads += 1;
                    let payload = e.records[slot].value;
                    if split == 0 {
                        visited_train[usize::from(payload)] = true
                    } else {
                        visited_dev[usize::from(payload)] = true
                    }
                    if out.tokens == [u16::from(e.answer), 256] {
                        correct += usize::from(hop == e.depth);
                        break;
                    }
                    query = dependent::update_query(&parent, query, payload)?;
                }
            }
        }
        let unseen = visited_dev
            .iter()
            .zip(visited_train)
            .filter(|(d, t)| **d && !*t)
            .count();
        let env_ok = correct == train.len() + dev.len() && correct_sources == reads && unseen == 0;
        write(
            &path,
            "environment.json",
            &json!({"pass":env_ok,"correct_answer_depth":correct,"rows":train.len()+dev.len(),"correct_sources":correct_sources,"reads":reads,"unseen_development_action_bytes":unseen,"training_action_bytes":visited_train.iter().filter(|&&b|b).count(),"typed_byte_domains":true,"four_read_is_resource_cap_not_stop_rule":true}),
        )?;
        if !env_ok {
            return Err("prefit environment failed".into());
        }
        let initial = learning::initialize(parent, &train)?;
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, config)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let restored = runtime::Artifact::decode(&encoded, &g)?;
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut full = 0;
        let mut depth_correct = 0;
        let mut initial_exact = 0;
        let mut always_emit = 0;
        let mut controls_ok = true;
        let mut pairs = BTreeMap::new();
        let mut replayed = 0;
        let mut depth_gate = true;
        for (split, examples) in [("training", &train), ("development", &dev)] {
            for (name, a) in [("initial", &initial), ("candidate", &candidate)] {
                for control in [
                    Control::Full,
                    Control::AlwaysRead,
                    Control::AlwaysEmit,
                    Control::FixedTwo,
                    Control::UpdateDisabled,
                    Control::PayloadDisabled,
                    Control::ReadDisabled,
                ] {
                    if name == "initial" && control != Control::Full {
                        continue;
                    }
                    let mut exact = 0;
                    let mut correct_depth = 0;
                    let mut exhausted = 0;
                    let mut long_exact = 0;
                    let mut per_depth = BTreeMap::<usize, [usize; 2]>::new();
                    for e in examples {
                        let out = runtime::generate(a, &g, &m, &e.records, e.query, control)?;
                        let yes = out.tokens == [u16::from(e.answer), 256];
                        exact += usize::from(yes);
                        long_exact += usize::from(yes && e.depth > 1);
                        correct_depth += usize::from(!out.exhausted && out.steps.len() == e.depth);
                        exhausted += usize::from(out.exhausted);
                        let c = per_depth.entry(e.depth).or_default();
                        c[0] += usize::from(yes);
                        c[1] += 1;
                        if name == "candidate" && control == Control::Full {
                            if out
                                != runtime::generate(
                                    &restored, &g, &m, &e.records, e.query, control,
                                )?
                            {
                                return Err("reload differs".into());
                            }
                            replayed += 1;
                            if split == "development" && yes {
                                *pairs.entry(e.family.clone()).or_insert(0usize) += 1;
                            }
                        }
                        rows.push(json!({"split":split,"artifact":name,"control":control,"id":e.id,"family":e.family,"expected_depth":e.depth,"answer":e.answer,"actual":out,"exact":yes}));
                    }
                    panels.push(json!({"split":split,"artifact":name,"control":control,"total":examples.len(),"exact":exact,"correct_depth":correct_depth,"exhausted":exhausted,"long_exact":long_exact,"per_depth":per_depth}));
                    if split == "development" {
                        if name == "initial" {
                            initial_exact = exact
                        } else {
                            match control {
                                Control::Full => {
                                    full = exact;
                                    depth_correct = correct_depth;
                                    depth_gate =
                                        per_depth.values().all(|c| c[0] * 100 >= c[1] * 95);
                                }
                                Control::AlwaysEmit => always_emit = exact,
                                Control::FixedTwo => controls_ok &= exact <= 96,
                                Control::AlwaysRead | Control::ReadDisabled => {
                                    controls_ok &= exact == 0
                                }
                                Control::UpdateDisabled | Control::PayloadDisabled => {
                                    controls_ok &= long_exact == 0
                                }
                            }
                        }
                    }
                }
            }
        }
        let prior: Value = serde_json::from_slice(&std::fs::read(&dep_data)?)?;
        let mut dep_ok = 0;
        let mut dep_total = 0;
        let mut adaptive_dep = 0;
        for split in ["training", "development"] {
            let examples: Vec<dependent_attention::data::Example> =
                serde_json::from_value(prior[split].clone())?;
            if examples.len() > 2048 {
                return Err("prior dependent size".into());
            }
            for e in examples {
                let out = dependent::generate(
                    &candidate.parent,
                    &g,
                    &m,
                    &e.records,
                    e.query,
                    dependent::Control::Full,
                )?;
                dep_ok += usize::from(
                    out.second.tokens == [u16::from(e.answer), 256]
                        && out.next_query == e.next_query,
                );
                let current =
                    runtime::generate(&candidate, &g, &m, &e.records, e.query, Control::Full)?;
                adaptive_dep += usize::from(
                    current.tokens == out.second.tokens
                        && !current.exhausted
                        && current.steps.len() == 2,
                );
                dep_total += 1;
            }
        }
        let prior: Value = serde_json::from_slice(&std::fs::read(&one_data)?)?;
        let mut one_ok = 0;
        let mut one_total = 0;
        let mut adaptive_one = 0;
        for split in ["training", "development"] {
            let examples: Vec<relational_attention::data::Example> =
                serde_json::from_value(prior[split].clone())?;
            if examples.len() > 8192 {
                return Err("prior one size".into());
            }
            for e in examples {
                let out = read::generate(
                    &candidate.parent.reader,
                    &g,
                    &m,
                    &e.records,
                    e.query,
                    read::Control::Full,
                )?;
                one_ok += usize::from(
                    out.tokens == [u16::from(e.answer), 256]
                        && out.selected.is_some_and(|i| e.records[i].key == e.query),
                );
                let current =
                    runtime::generate(&candidate, &g, &m, &e.records, e.query, Control::Full)?;
                adaptive_one += usize::from(
                    current.tokens == out.tokens && !current.exhausted && current.steps.len() == 1,
                );
                one_total += 1;
            }
        }
        let both = pairs.values().filter(|&&n| n == 2).count();
        let pass = full >= 183
            && depth_correct >= 183
            && both >= 91
            && depth_gate
            && full >= initial_exact + 96
            && full >= always_emit + 64
            && controls_ok
            && adaptive_dep == 2176
            && adaptive_one == 8256
            && dep_ok == 2176
            && dep_total == 2176
            && one_ok == 8256
            && one_total == 8256
            && candidate.parent == initial.parent
            && fit.stopped == "completed_dose"
            && fit.completed_epochs == fit.config.epochs;
        std::fs::write(path.join("responses.json"), serde_json::to_vec(&rows)?)?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_TYPED_ADAPTIVE_READ_EMIT"}else{"FAIL_TYPED_ADAPTIVE_READ_EMIT"},"panels":panels,"development_pairs_both_exact":both,"reload_equal_rows":replayed,"candidate_sha256":hex::encode(sha2::Sha256::digest(&encoded)),"parent_sha256":parent_sha,"parent_unchanged":candidate.parent==initial.parent,"changed_action_entries":candidate.emit.iter().zip(&initial.emit).filter(|(a,b)|a!=b).count(),"emit_bytes":candidate.emit.iter().enumerate().filter(|(_,v)|**v!=0).map(|(i,_)|i).collect::<Vec<_>>(),"adaptive_retained_dependent_correct":adaptive_dep,"adaptive_retained_one_correct":adaptive_one,"retention_scope":"old paths plus new adaptive runner on both prior corpora","retained_dependent_correct":dep_ok,"retained_dependent_total":dep_total,"retained_one_correct":one_ok,"retained_one_total":one_total,"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
fn artifact_and_loop_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    let (reader, _, _) = relational_attention::learning::initialized(&g, &[])?;
    let (parent, _) = dependent_attention::learning::initialize(reader, &[])?;
    let a = learning::initialize(parent, &[])?;
    a.validate(&g)?;
    assert_eq!(a, runtime::Artifact::decode(&a.encode()?, &g)?);
    let records = [relational_attention::data::Record {
        key: [1, 2],
        value: 3,
    }; 4];
    let out = runtime::generate(&a, &g, &m, &records, [1, 2], Control::AlwaysRead)?;
    assert!(out.exhausted);
    assert!(out.tokens.is_empty());
    assert_eq!(out.steps.len(), MAX_READS);
    let mut bad = a.clone();
    bad.emit.pop();
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.emit[0] = 2;
    assert!(bad.validate(&g).is_err());
    let mut bad = a.clone();
    bad.parent.tables[0] ^= 1;
    assert!(bad.validate(&g).is_err());
    let (examples, _) = data::corpus()?;
    assert!(learning::fit(&a, &g, &m, &examples[..1], learning::Config::default()).is_err());
    Ok(())
}
