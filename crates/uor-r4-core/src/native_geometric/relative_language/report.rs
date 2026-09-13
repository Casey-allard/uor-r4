use super::{
    data, learning,
    runtime::{self, Control},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry,
        hamming_refinement::metric::Metric,
        language_relation, ordered_state,
        recurrent_text::{self, runtime as recurrent},
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
fn sha(b: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(b))
}
#[test]
fn corpus_and_lexical_boundaries_preserve_occurrences() -> Result<(), Box<dyn std::error::Error>> {
    data::validate()?;
    let g = BoundGeometry::canonical()?;
    let words = runtime::words(
        &g,
        b"mira did help nora.",
        ordered_state::runtime::CANONICAL,
    )?;
    assert_eq!(words.len(), 4);
    assert_eq!(
        &b"mira did help nora."[words[3].start..words[3].end],
        b"nora"
    );
    assert_ne!(words[0].geometry.occurrences, words[3].geometry.occurrences);
    assert!(runtime::words(&g, b"bad123", ordered_state::runtime::CANONICAL).is_err());
    assert!(runtime::words(&g, &[b'a'; 129], ordered_state::runtime::CANONICAL).is_err());
    Ok(())
}
#[test]
#[ignore = "output-supervised contextual language relation fit with pinned artifacts and exclusive report"]
fn relative_language_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_RELATIVE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_RELATIVE_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":2048,"development_rows":768,"minimum_exact_per_split":244,"minimum_occurrences_per_split":244,"minimum_complete_families_per_split":15,"required_controls_zero":["ReadDisabled","ScorerDisabled","CoverageDisabled","RelativePositionDisabled","CursorDisabled","StopDisabled"],"order_erased_exact_max":384,"match_order_disabled_exact_max":384,"endpoint_disabled_exact_max":384,"diagnostic_only":["ExactIdentity","FinalRootOnly","FeedbackDisabled"],"retention":"13248 old outputs,512recurrent control traces,640ordered controls,1408prior language controls and128old language through new selector","max_rule_literals":4,"max_rules":8,"max_proposals":4047,"final_holdout":"NOT_RUN","scope":"mixed-length active constructions and disjoint lexical/syntactic/joint development, generic lexical boundaries, output-supervised contextual alignment; not general prose or semantic metric advantage"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(evidence.join("language-relation-1/attempt-1/candidate.json"))?;
        if sha(&bytes) != "d1883e09fade8c5337fb604bf6af74556b0f18decdf745ba68bc832a9e4a74ce" {
            return Err("parent hash".into());
        }
        let parent = language_relation::runtime::Artifact::decode(&bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate()?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        let initial = learning::initialize(parent, &train)?;
        let unsupported = runtime::route(
            &initial,
            &g,
            &m,
            &train[0].records,
            b"who did mira help?abcdefghijklmnopq",
            Control::Full,
        )?;
        if unsupported.status != runtime::RouteStatus::UnsupportedWordWindow {
            return Err("unsupported feedback misclassified".into());
        }
        let prepared = learning::prepare(&initial, &g, &m, &train)?;
        let missing: Vec<_> = prepared
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.compatible_outputs.iter().any(|&x| x))
            .map(|(i, _)| train[i].id.clone())
            .collect();
        let mut feature_classes = BTreeMap::<u32, [usize; 2]>::new();
        for p in &prepared {
            for (c, &good) in p.candidates.iter().zip(&p.compatible_outputs) {
                feature_classes.entry(c.features).or_default()[usize::from(good)] += 1;
            }
        }
        let ambiguous_classes = feature_classes
            .values()
            .filter(|n| n[0] > 0 && n[1] > 0)
            .count();
        write(
            &path,
            "preparation.json",
            &json!({"training":prepared,"missing_output_compatible_examples":missing,"feature_classes":feature_classes,"ambiguous_candidate_feature_classes":ambiguous_classes,"credit":"all output-compatible occurrences remain latent; rule must select a unique occurrence and avoid wrong-output candidates; source/word labels excluded","admission":"all word occurrences from all4sentences; bounded16words each","features":runtime::FEATURE_NAMES,"all_authored_examples_retained":true}),
        )?;
        if !missing.is_empty() {
            write(
                &path,
                "summary.json",
                &json!({"gate":"FAIL_RELATIVE_LANGUAGE_REPRESENTATION","fit":"NOT_RUN","promotion":false}),
            )?;
            return Ok(());
        }
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, &prepared)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let reload = runtime::Artifact::decode(&encoded, &g)?;
        let mut malformed = candidate.clone();
        malformed.rules.push(1 << 31);
        if malformed.validate(&g).is_ok() {
            return Err("bad feature accepted".into());
        }
        let mut malformed = candidate.clone();
        malformed.parent.parent.operators.swap(0, 1);
        if malformed.validate(&g).is_ok() {
            return Err("parent corruption accepted".into());
        }
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut full = 0;
        let mut paths = 0;
        let mut families = BTreeMap::new();
        let mut controls_ok = true;
        let mut split_full = BTreeMap::<String, [usize; 3]>::new();
        for (name, a, c) in [
            ("initial", &initial, Control::Full),
            ("candidate", &candidate, Control::Full),
            ("candidate", &candidate, Control::ReadDisabled),
            ("candidate", &candidate, Control::ScorerDisabled),
            ("candidate", &candidate, Control::OrderErased),
            ("candidate", &candidate, Control::CoverageDisabled),
            ("candidate", &candidate, Control::RelativePositionDisabled),
            ("candidate", &candidate, Control::MatchOrderDisabled),
            ("candidate", &candidate, Control::EndpointDisabled),
            ("candidate", &candidate, Control::ExactIdentity),
            ("candidate", &candidate, Control::FinalRootOnly),
            ("candidate", &candidate, Control::FeedbackDisabled),
            ("candidate", &candidate, Control::CursorDisabled),
            ("candidate", &candidate, Control::StopDisabled),
        ] {
            let mut exact = 0;
            let mut occurrences = 0;
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, &e.question, c)?;
                let yes = !out.actual.exhausted && out.actual.tokens == learning::target(e);
                exact += usize::from(yes);
                let selected = out
                    .routes
                    .first()
                    .and_then(|r| r.selected.as_ref())
                    .map(|x| [x.source, x.word]);
                let occurrence = selected == Some([e.expected_source, e.expected_word]);
                occurrences += usize::from(occurrence);
                if name == "candidate" && c == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, &e.question, c)? {
                        return Err("reload differs".into());
                    }
                    let counts = split_full.entry(e.split.clone()).or_default();
                    counts[0] += usize::from(yes);
                    counts[1] += usize::from(occurrence);
                    counts[2] += 1;
                    if yes && occurrence {
                        *families.entry(e.family.clone()).or_insert(0) += 1;
                    }
                    if e.id == dev[0].id {
                        write(
                            &path,
                            "sample-trace.json",
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                }
                rows.push(json!({"id":e.id,"family":e.family,"split":e.split,"construction":e.construction,"artifact":name,"control":c,"question":String::from_utf8_lossy(&e.question),"answer":String::from_utf8_lossy(&e.answer),"actual_text":String::from_utf8_lossy(&out.actual.tokens.iter().filter_map(|&t|u8::try_from(t).ok()).collect::<Vec<_>>()),"tokens":out.actual.tokens,"exhausted":out.actual.exhausted,"exact":yes,"selected":selected,"expected":[e.expected_source,e.expected_word],"occurrence_correct":occurrence,"route_statuses":out.routes.iter().map(|r|r.status).collect::<Vec<_>>()}));
            }
            panels.push(json!({"artifact":name,"control":c,"exact":exact,"correct_occurrences":occurrences,"total":dev.len()}));
            if name == "candidate" {
                match c {
                    Control::Full => {
                        full = exact;
                        paths = occurrences;
                    }
                    Control::ExactIdentity | Control::FinalRootOnly | Control::FeedbackDisabled => {
                    }
                    Control::OrderErased
                    | Control::MatchOrderDisabled
                    | Control::EndpointDisabled => controls_ok &= exact <= 384,
                    _ => controls_ok &= exact == 0,
                }
            }
        }
        // Replay the predecessor's exact responses through the generalized transition.
        // Same artifact and byte-query adapter; no refit or relaxation of old controls.
        let prior_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/data.json"),
        )?)?;
        let prior_retained: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/retained-responses.json"),
        )?)?;
        let mut legacy: BTreeMap<String, recurrent_text::data::Example> = BTreeMap::new();
        for split in ["joint_training", "development"] {
            for e in serde_json::from_value::<Vec<recurrent_text::data::Example>>(
                prior_data[split].clone(),
            )? {
                legacy.insert(e.id.clone(), e);
            }
        }
        // Earlier development rows were not training inputs; recover their exact data.
        for (name, rel, is_text) in [
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
            for x in v["development"].as_array().ok_or("old development")? {
                let records = if is_text {
                    serde_json::from_value(x["records"].clone())?
                } else {
                    let r: [crate::native_geometric::relational_attention::data::Record; 4] =
                        serde_json::from_value(x["records"].clone())?;
                    r.map(|r| recurrent_text::data::Record {
                        key: r.key,
                        text: vec![r.value],
                    })
                };
                let e = recurrent_text::data::Example {
                    id: format!("{name}:{}", x["id"].as_str().ok_or("id")?),
                    family: name.into(),
                    records,
                    query: serde_json::from_value(x["query"].clone())?,
                    answer: if is_text {
                        serde_json::from_value(x["answer"].clone())?
                    } else {
                        vec![serde_json::from_value(x["answer"].clone())?]
                    },
                };
                legacy.insert(e.id.clone(), e);
            }
        }
        let mut retained_rows = Vec::new();
        let mut retained_ok = true;
        for x in prior_retained.as_array().ok_or("old responses")? {
            let id = x["id"].as_str().ok_or("id")?;
            let e = legacy.get(id).ok_or("retained input missing")?;
            let out = recurrent::generate(
                &candidate.recurrent(),
                &g,
                &m,
                &e.records,
                e.query,
                recurrent::Control::Full,
            )?;
            let equal = serde_json::to_value(&out.tokens)? == x["actual"] && !out.exhausted;
            retained_ok &= equal;
            retained_rows.push(json!({"id":id,"equal":equal,"tokens":out.tokens}));
        }
        let prior_rows: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/responses.json"),
        )?)?;
        let mut control_replays = 0;
        for x in prior_rows
            .as_array()
            .ok_or("old controls")?
            .iter()
            .filter(|x| x["artifact"] == "candidate")
        {
            let id = x["id"].as_str().ok_or("control id")?;
            let e = legacy.get(id).ok_or("control input")?;
            let c: recurrent::Control = serde_json::from_value(x["control"].clone())?;
            let out = recurrent::generate(&candidate.recurrent(), &g, &m, &e.records, e.query, c)?;
            let equal = serde_json::to_value(&out)? == x["actual"];
            retained_ok &= equal;
            control_replays += usize::from(equal);
            retained_rows.push(json!({"id":id,"control":c,"equal":equal}));
        }
        let old_ordered = candidate.ordered()?;
        let ordered_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("ordered-state-1/attempt-2/data.json"),
        )?)?;
        let ordered_dev: Vec<ordered_state::data::Example> =
            serde_json::from_value(ordered_data["development"].clone())?;
        let ordered_rows: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("ordered-state-1/attempt-2/responses.json"),
        )?)?;
        let mut ordered_equal = 0;
        for x in ordered_rows
            .as_array()
            .ok_or("ordered rows")?
            .iter()
            .filter(|x| x["artifact"] == "candidate")
        {
            let e = ordered_dev
                .iter()
                .find(|e| x["id"] == e.id)
                .ok_or("ordered input")?;
            let c: ordered_state::runtime::Control = serde_json::from_value(x["control"].clone())?;
            let out =
                ordered_state::runtime::generate(&old_ordered, &g, &m, &e.records, &e.query, c)?;
            let actual_path: Vec<_> = out
                .steps
                .iter()
                .filter(|s| s.before.cursor == 0)
                .filter_map(|s| s.observation.selected)
                .collect();
            let equal = serde_json::to_value(&out.tokens)? == x["tokens"]
                && serde_json::to_value(&out.exhausted)? == x["exhausted"]
                && serde_json::to_value(&actual_path)? == x["actual_sources"];
            retained_ok &= equal;
            ordered_equal += usize::from(equal);
            retained_rows.push(json!({"id":e.id,"ordered_control":c,"equal":equal}));
        }
        if prior_retained.as_array().ok_or("retained rows")?.len() != 13248
            || control_replays != 512
            || ordered_equal != 640
        {
            retained_ok = false;
        }
        let old_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("language-relation-1/attempt-1/data.json"),
        )?)?;
        let old_dev: Vec<language_relation::data::Example> =
            serde_json::from_value(old_data["development"].clone())?;
        let old_rows: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("language-relation-1/attempt-1/responses.json"),
        )?)?;
        let mut language_equal = 0;
        for x in old_rows
            .as_array()
            .ok_or("language rows")?
            .iter()
            .filter(|x| x["artifact"] == "candidate")
        {
            let e = old_dev
                .iter()
                .find(|e| x["id"] == e.id)
                .ok_or("language input")?;
            let c: language_relation::runtime::Control =
                serde_json::from_value(x["control"].clone())?;
            let out = language_relation::runtime::generate(
                &candidate.parent,
                &g,
                &m,
                &e.records,
                &e.question,
                c,
            )?;
            let selected = out
                .routes
                .first()
                .and_then(|r| r.selected.as_ref())
                .map(|x| [x.source, x.word]);
            let equal = serde_json::to_value(&out.actual.tokens)? == x["tokens"]
                && serde_json::to_value(out.actual.exhausted)? == x["exhausted"]
                && serde_json::to_value(selected)? == x["selected"];
            retained_ok &= equal;
            language_equal += usize::from(equal);
            retained_rows.push(json!({"id":e.id,"language_control":c,"equal":equal}));
        }
        let mut old_through_new = 0;
        for e in &old_dev {
            let out =
                runtime::generate(&candidate, &g, &m, &e.records, &e.question, Control::Full)?;
            let selected = out
                .routes
                .first()
                .and_then(|r| r.selected.as_ref())
                .map(|x| [x.source, x.word]);
            let equal = !out.actual.exhausted
                && out.actual.tokens == language_relation::learning::target(e)
                && selected == Some([e.expected_source, e.expected_word]);
            old_through_new += usize::from(equal);
            retained_ok &= equal;
            retained_rows.push(json!({"id":e.id,"new_selector":true,"equal":equal,"tokens":out.actual.tokens,"selected":selected}));
        }
        retained_ok &= language_equal == 1408 && old_through_new == 128;
        let mut split_families = BTreeMap::<String, usize>::new();
        for e in &dev {
            if families.get(&e.family) == Some(&16) {
                split_families.entry(e.split.clone()).or_insert_with(|| {
                    dev.iter()
                        .filter(|v| v.split == e.split)
                        .map(|v| v.family.clone())
                        .collect::<std::collections::BTreeSet<_>>()
                        .iter()
                        .filter(|f| families.get(*f) == Some(&16))
                        .count()
                });
            }
        }
        let complete = families.values().filter(|&&n| n == 16).count();
        let pass = split_full.len() == 3
            && split_full
                .values()
                .all(|n| n[0] >= 244 && n[1] >= 244 && n[2] == 256)
            && split_families.len() == 3
            && split_families.values().all(|&n| n >= 15)
            && controls_ok
            && retained_ok
            && fit.actual_training_exact == train.len();
        write(&path, "responses.json", &json!(rows))?;
        write(&path, "retained-responses.json", &json!(retained_rows))?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_RELATIVE_LANGUAGE_BINDING"}else{"FAIL_RELATIVE_LANGUAGE_BINDING"},"panels":panels,"split_full":split_full,"split_complete_families":split_families,"full_exact":full,"retained_language_controls":language_equal,"old_language_through_new":old_through_new,"complete_families":complete,"correct_occurrences":paths,"training_exact":fit.actual_training_exact,"learned_rules":candidate.rules,"changed_action_rows":fit.changed_actions,"retained_rows":prior_retained.as_array().ok_or("rows")?.len(),"retained_recurrent_traces":control_replays,"retained_ordered_controls":ordered_equal,"retention_equal":retained_ok,"candidate_sha256":sha(&encoded),"parent_sha256":sha(&bytes),"reload_equal_rows":dev.len(),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
