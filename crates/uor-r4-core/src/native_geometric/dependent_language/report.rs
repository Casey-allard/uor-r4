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
        relative_language::{self, runtime as reader},
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

fn retained(
    candidate: &runtime::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
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
        for e in
            serde_json::from_value::<Vec<recurrent_text::data::Example>>(prior_data[split].clone())?
        {
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
        let out = ordered_state::runtime::generate(&old_ordered, &g, &m, &e.records, &e.query, c)?;
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
        let c: language_relation::runtime::Control = serde_json::from_value(x["control"].clone())?;
        let out = language_relation::runtime::generate(
            &candidate.parent.parent,
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
        let out = reader::generate(
            &candidate.parent,
            &g,
            &m,
            &e.records,
            &e.question,
            reader::Control::Full,
        )?;
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

    let relative_data: Value = serde_json::from_slice(&std::fs::read(
        evidence.join("relative-language-1/attempt-2/data.json"),
    )?)?;
    let relative_dev: Vec<relative_language::data::Example> =
        serde_json::from_value(relative_data["development"].clone())?;
    let relative_rows: Value = serde_json::from_slice(&std::fs::read(
        evidence.join("relative-language-1/attempt-2/responses.json"),
    )?)?;
    let mut relative_equal = 0;
    for x in relative_rows
        .as_array()
        .ok_or("relative rows")?
        .iter()
        .filter(|x| x["artifact"] == "candidate")
    {
        let e = relative_dev
            .iter()
            .find(|e| x["id"] == e.id)
            .ok_or("relative input")?;
        let c: reader::Control = serde_json::from_value(x["control"].clone())?;
        let out = reader::generate(&candidate.parent, g, m, &e.records, &e.question, c)?;
        let selected = out
            .routes
            .first()
            .and_then(|r| r.selected.as_ref())
            .map(|v| [v.source, v.word]);
        let equal = serde_json::to_value(&out.actual.tokens)? == x["tokens"]
            && serde_json::to_value(out.actual.exhausted)? == x["exhausted"]
            && serde_json::to_value(selected)? == x["selected"];
        relative_equal += usize::from(equal);
        retained_ok &= equal;
        retained_rows.push(json!({"id":e.id,"relative_control":c,"equal":equal}));
    }
    retained_ok &= relative_equal == 9984;
    Ok(
        json!({"equal":retained_ok,"old_outputs":13248,"recurrent_controls":control_replays,"ordered_controls":ordered_equal,"language_controls":language_equal,"old_language_through_relative":old_through_new,"relative_controls":relative_equal,"rows":retained_rows}),
    )
}

#[test]
#[ignore = "Final-output-trained query update with pinned parent and exclusive evidence"]
fn dependent_language_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_DEPENDENT_LANGUAGE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_DEPENDENT_LANGUAGE_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":2048,"development_rows":768,"minimum_exact_per_split":244,"minimum_paths_per_split":244,"minimum_complete_families_per_split":15,"required_controls_zero":["FirstReadDisabled","SecondReadDisabled","ScorerDisabled","UpdateDisabled","PayloadReversed","PositionDisabled","ContextMatchDisabled","SwapQuestions","CursorDisabled","StopDisabled"],"diagnostic_only":["ExactIdentity","FeedbackDisabled"],"max_rules":8,"max_literals":3,"proposals":92,"final_holdout":"NOT_RUN","scope":"two punctuated raw questions, one-token anaphoric substitution, fixed Read boundary; final-output-supervised update, not learned scheduling or general prose"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(evidence.join("relative-language-1/attempt-2/candidate.json"))?;
        if sha(&bytes) != "4e3d4da1fb39425b30007b1e00c16a520f860c8dd8d3257a0787f99144ada9e9" {
            return Err("parent hash".into());
        }
        let parent = reader::Artifact::decode(&bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate()?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        let initial = learning::initialize(parent, &train)?;
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let prepared = learning::prepare(&initial, &g, &m, &train)?;
        let missing: Vec<_> = prepared
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.compatible_outputs.iter().any(|&x| x))
            .map(|(i, _)| train[i].id.clone())
            .collect();
        write(
            &path,
            "preparation.json",
            &json!({"training":prepared,"missing_output_compatible_examples":missing,"credit":"Each candidate replaces one word of second raw question with actual first-read content; full suffix emission determines compatibility. Gold intermediate/path excluded.","all_authored_examples_retained":true,"feature_names":runtime::FEATURE_NAMES}),
        )?;
        if !missing.is_empty() {
            write(
                &path,
                "summary.json",
                &json!({"gate":"FAIL_DEPENDENT_LANGUAGE_REPRESENTATION","fit":"NOT_RUN","promotion":false}),
            )?;
            return Ok(());
        }
        let (candidate, fit) = if let Some(prior) =
            std::env::var_os("UOR_DEPENDENT_LANGUAGE_RESUME")
        {
            let prior = std::path::PathBuf::from(prior);
            report_output::verify(&prior)?;
            if std::fs::read(prior.join("initial.json"))? != initial.encode()?
                || std::fs::read(prior.join("data.json"))? != std::fs::read(path.join("data.json"))?
            {
                return Err("resume source, parent or corpus differs".into());
            }
            let bytes = std::fs::read(prior.join("candidate.json"))?;
            let candidate = runtime::Artifact::decode(&bytes, &g)?;
            if candidate.source_digest != initial.source_digest
                || candidate.data_digest != initial.data_digest
                || candidate.parent != initial.parent
            {
                return Err("resume candidate identity differs".into());
            }
            let fit: learning::Fit =
                serde_json::from_slice(&std::fs::read(prior.join("fit.json"))?)?;
            write(
                &path,
                "resume.json",
                &json!({"prior_report":prior,"candidate_sha256":sha(&bytes),"fit_reused":true,"reason":"Correct read-count assertion: initial selection counts as read 1; dependent transition commits read 2. Model source, data and acceptance unchanged."}),
            )?;
            (candidate, fit)
        } else {
            learning::fit(&initial, &g, &m, &train, &prepared)?
        };
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
        malformed.parent.rules.clear();
        if malformed.validate(&g).is_ok() {
            return Err("parent corruption accepted".into());
        }
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut splits = BTreeMap::<String, [usize; 3]>::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut controls_ok = true;
        let mut initial_zero = false;
        for (name, a, c) in [
            ("initial", &initial, Control::Full),
            ("candidate", &candidate, Control::Full),
            ("candidate", &candidate, Control::FirstReadDisabled),
            ("candidate", &candidate, Control::SecondReadDisabled),
            ("candidate", &candidate, Control::ScorerDisabled),
            ("candidate", &candidate, Control::UpdateDisabled),
            ("candidate", &candidate, Control::PayloadReversed),
            ("candidate", &candidate, Control::PositionDisabled),
            ("candidate", &candidate, Control::ContextMatchDisabled),
            ("candidate", &candidate, Control::SwapQuestions),
            ("candidate", &candidate, Control::ExactIdentity),
            ("candidate", &candidate, Control::FeedbackDisabled),
            ("candidate", &candidate, Control::CursorDisabled),
            ("candidate", &candidate, Control::StopDisabled),
        ] {
            let mut exact = 0;
            let mut paths = 0;
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, &e.prompt, c)?;
                let tokens = out.tokens();
                let yes = !out.exhausted() && tokens == learning::target(e);
                let first = out.first.selected.as_ref().map(|v| [v.source, v.word]);
                let second = out
                    .output
                    .as_ref()
                    .and_then(|o| o.routes.first())
                    .and_then(|r| r.selected.as_ref())
                    .map(|v| [v.source, v.word]);
                let good_path = first == Some([e.expected_sources[0], e.expected_words[0]])
                    && second == Some([e.expected_sources[1], e.expected_words[1]])
                    && out.first.selected.as_ref().map(|x| x.bytes.as_slice())
                        == Some(e.intermediate.as_slice());
                exact += usize::from(yes);
                paths += usize::from(good_path);
                if name == "candidate" && c == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, &e.prompt, c)? {
                        return Err("reload differs".into());
                    }
                    if let (Some(t), Some(o)) = (&out.transition, &out.output) {
                        if t.token.is_some()
                            || t.before.reads != 1
                            || t.after.reads != 2
                            || o.actual.steps.first().is_none_or(|s| s.before != t.after)
                        {
                            return Err("silent Read state was not continued".into());
                        }
                    }
                    let n = splits.entry(e.split.clone()).or_default();
                    n[0] += usize::from(yes);
                    n[1] += usize::from(good_path);
                    n[2] += 1;
                    if yes && good_path {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    if e.id == dev[0].id {
                        write(
                            &path,
                            "sample-trace.json",
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                }
                rows.push(json!({"id":e.id,"family":e.family,"split":e.split,"variant":e.variant,"artifact":name,"control":c,"prompt":String::from_utf8_lossy(&e.prompt),"answer":String::from_utf8_lossy(&e.answer),"actual_text":String::from_utf8_lossy(&tokens.iter().filter_map(|&t|u8::try_from(t).ok()).collect::<Vec<_>>()),"tokens":tokens,"exhausted":out.exhausted(),"exact":yes,"first":first,"second":second,"path_correct":good_path,"intermediate":out.first.selected.as_ref().map(|v|String::from_utf8_lossy(&v.bytes).into_owned()),"updated_question":out.transition.as_ref().map(|t|String::from_utf8_lossy(&t.after.query.bytes).into_owned()),"update_word":out.update.as_ref().map(|u|u.word),"status":out.status}));
            }
            panels.push(json!({"artifact":name,"control":c,"exact":exact,"correct_paths":paths,"total":dev.len()}));
            if name == "initial" {
                initial_zero = exact == 0;
            } else if !matches!(
                c,
                Control::Full | Control::ExactIdentity | Control::FeedbackDisabled
            ) {
                controls_ok &= exact == 0;
            }
        }
        let retained = retained(&candidate, &g, &m, &evidence)?;
        write(&path, "retained-responses.json", &retained)?;
        let mut split_families = BTreeMap::new();
        for split in ["lexical", "syntactic", "joint"] {
            let n = dev
                .iter()
                .filter(|e| e.split == split)
                .map(|e| e.family.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .iter()
                .filter(|f| families.get(*f) == Some(&16))
                .count();
            split_families.insert(split, n);
        }
        let pass = initial_zero
            && splits.len() == 3
            && splits
                .values()
                .all(|n| n[0] >= 244 && n[1] >= 244 && n[2] == 256)
            && split_families.values().all(|&n| n >= 15)
            && controls_ok
            && retained["equal"] == true
            && fit.actual_training_exact == train.len();
        write(&path, "responses.json", &json!(rows))?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_DEPENDENT_LANGUAGE_BINDING"}else{"FAIL_DEPENDENT_LANGUAGE_BINDING"},"panels":panels,"split_full":splits,"split_complete_families":split_families,"training_exact":fit.actual_training_exact,"learned_rules":candidate.rules,"changed_action_rows":fit.changed_actions,"retained_counts":{"old_outputs":retained["old_outputs"],"recurrent":retained["recurrent_controls"],"ordered":retained["ordered_controls"],"language":retained["language_controls"],"old_language_through_relative":retained["old_language_through_relative"],"relative":retained["relative_controls"]},"retention_equal":retained["equal"],"candidate_sha256":sha(&encoded),"parent_sha256":sha(&bytes),"reload_equal_rows":dev.len(),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
