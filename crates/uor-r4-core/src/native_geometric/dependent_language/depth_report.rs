//! Frozen-parameter third-read qualification. No fitting or model-based filtering.
use super::{
    depth_data as data, runtime as binding, schedule_data,
    scheduling::{self as runtime, Control},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        recurrent_text::runtime::Action,
    },
    report_output,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(p: &Path, n: &str, v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(p.join(n), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn path(o: &runtime::Generated) -> Vec<[usize; 2]> {
    o.steps
        .iter()
        .filter(|s| s.before.core.cursor == 0 && matches!(s.action, Action::Read | Action::Emit))
        .filter_map(|s| {
            s.observation
                .route
                .selected
                .as_ref()
                .map(|v| [v.source, v.word])
        })
        .collect()
}
fn record(o: &runtime::Generated) -> Value {
    json!({"tokens":o.tokens,"exhausted":o.exhausted,"sources":path(o),"actions":o.steps.iter().map(|s|s.action).collect::<Vec<_>>(),"policy_rows":o.steps.iter().map(|s|s.observation.core.row).collect::<Vec<_>>(),"final_reads":o.steps.last().map(|s|s.after.core.reads)})
}
#[test]
#[ignore = "Frozen-parameter third language read; exclusive report"]
fn third_language_read_report() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_DEPTH_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_DEPTH_EVIDENCE")?);
    if root.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&root)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &root,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training":false,"rows":1280,"full_answers":1280,"full_paths":1280,"families":64,"family_size":20,"reads":3,"silent_reads":2,"prefix_intervention_isolated_rows":1280,"required_zero":["PolicyDisabled","ReadDisabled","ContinuationDisabled","AlwaysRead","EmitInsteadOfRead","UpdateDisabled","PayloadReversed","ScorerDisabled","CursorDisabled","StopDisabled","SecondUpdateDisabled","StaleSecondPayload"],"diagnostic_only":["ExactIdentity","FeedbackDisabled"],"artifact_bytes_unchanged":true,"final_holdout":"NOT_RUN","scope":"frozen-parameter depth transfer with familiar vocabulary/grammar and explicit sequential question references"}),
        )?;
        let dev = data::corpus()?;
        let validation = data::validate(&dev)?;
        write(
            &root,
            "data.json",
            &json!({"training":[],"development":dev,"validation":validation}),
        )?;
        let bytes = std::fs::read(evidence.join("language-credit-1/attempt-1/candidate.json"))?;
        if sha(&bytes) != "62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95" {
            return Err("parent hash".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = runtime::Artifact::decode(&bytes, &g)?;
        if a.encode()? != bytes {
            return Err("artifact serialization changed".into());
        }
        std::fs::write(root.join("candidate.json"), &bytes)?;
        let reload = runtime::Artifact::decode(&bytes, &g)?;
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut full = BTreeMap::new();
        let mut gate = true;
        for c in [
            Control::Full,
            Control::SecondUpdateDisabled,
            Control::StaleSecondPayload,
            Control::PolicyDisabled,
            Control::ReadDisabled,
            Control::ContinuationDisabled,
            Control::AlwaysRead,
            Control::EmitInsteadOfRead,
            Control::UpdateDisabled,
            Control::PayloadReversed,
            Control::ScorerDisabled,
            Control::CursorDisabled,
            Control::StopDisabled,
            Control::ExactIdentity,
            Control::FeedbackDisabled,
        ] {
            let (mut exact, mut paths, mut isolated) = (0, 0, 0);
            let mut variants = BTreeMap::<String, usize>::new();
            for e in &dev {
                let out = runtime::generate(&a, &g, &m, &e.records, &e.prompt, c)?;
                let expected = e
                    .answer
                    .iter()
                    .map(|&b| u16::from(b))
                    .chain(std::iter::once(256))
                    .collect::<Vec<_>>();
                let yes = !out.exhausted && out.tokens == expected;
                let py = path(&out) == e.expected_path;
                exact += usize::from(yes);
                paths += usize::from(py);
                *variants.entry(e.variant.clone()).or_default() += usize::from(yes);
                if c == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, &e.prompt, c)? {
                        return Err("reload".into());
                    }
                    let reads: Vec<_> = out
                        .steps
                        .iter()
                        .filter(|s| s.action == Action::Read)
                        .collect();
                    let state_ok = reads.len() == 2
                        && reads.iter().enumerate().all(|(i, s)| {
                            s.token.is_none()
                                && s.before.clause == i
                                && s.after.clause == i + 1
                                && s.before.core.reads == i + 1
                                && s.after.core.reads == i + 2
                                && s.observation.route.selected.as_ref().map(|v| &v.bytes)
                                    == e.intermediates.get(i)
                        })
                        && out.steps.windows(2).all(|w| w[0].after == w[1].before)
                        && out
                            .steps
                            .iter()
                            .filter(|s| s.token.is_some())
                            .all(|s| s.before.clause == 2)
                        && out
                            .steps
                            .last()
                            .is_some_and(|s| s.after.core.reads == 3 && s.after.core.done);
                    gate &= state_ok;
                    if yes && py && state_ok {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    full.insert(e.id.clone(), out.clone());
                } else if matches!(
                    c,
                    Control::SecondUpdateDisabled | Control::StaleSecondPayload
                ) {
                    let baseline = full.get(&e.id).ok_or("missing baseline")?;
                    let prefix = out.steps.first() == baseline.steps.first();
                    let second = out.steps.get(1);
                    let bs = baseline.steps.get(1);
                    let changed = match (second, bs) {
                        (Some(s), Some(b)) => {
                            s.before == b.before
                                && s.before.clause == 1
                                && s.observation.route == b.observation.route
                                && s.action == Action::Read
                                && s.after.core.query != b.after.core.query
                        }
                        _ => false,
                    };
                    let ok = prefix && changed;
                    isolated += usize::from(ok);
                    gate &= ok;
                }
                let mut row = record(&out);
                row["id"] = json!(e.id);
                row["family"] = json!(e.family);
                row["variant"] = json!(e.variant);
                row["control"] = serde_json::to_value(c)?;
                row["exact"] = json!(yes);
                row["path_correct"] = json!(py);
                rows.push(row);
            }
            if c == Control::Full {
                gate &= exact == 1280 && paths == 1280;
            } else if !matches!(c, Control::ExactIdentity | Control::FeedbackDisabled) {
                gate &= exact == 0;
            }
            panels.push(json!({"control":c,"exact":exact,"paths":paths,"total":dev.len(),"isolated_second_update":isolated,"variants":variants}));
        }
        write(&root, "responses.json", &json!(rows))?;
        let complete = families.values().filter(|&&n| n == 20).count();
        gate &= dev.len() == 1280 && complete == 64;
        // Compare every previous scheduling row independently. Older primitive adapters
        // have identical serialized parameters and are exercised by retained().
        let old = super::report::retained(&a.parent, &g, &m, &evidence)?;
        let mut retained = Vec::new();
        let prior_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("dependent-language-1/attempt-2/data.json"),
        )?)?;
        let prior_dev: Vec<super::data::Example> =
            serde_json::from_value(prior_data["development"].clone())?;
        let prior_rows: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("dependent-language-1/attempt-2/responses.json"),
        )?)?;
        let mut dependent_rows = Vec::new();
        let mut dependent_equal = 0;
        for row in prior_rows
            .as_array()
            .ok_or("prior rows")?
            .iter()
            .filter(|r| r["artifact"] == "candidate")
        {
            let e = prior_dev
                .iter()
                .find(|e| row["id"] == e.id)
                .ok_or("prior input")?;
            let c: binding::Control = serde_json::from_value(row["control"].clone())?;
            let out = binding::generate(&a.parent, &g, &m, &e.records, &e.prompt, c)?;
            let first = out.first.selected.as_ref().map(|s| [s.source, s.word]);
            let second = out
                .output
                .as_ref()
                .and_then(|o| o.routes.first())
                .and_then(|r| r.selected.as_ref())
                .map(|s| [s.source, s.word]);
            let equal = serde_json::to_value(out.tokens())? == row["tokens"]
                && serde_json::to_value(out.exhausted())? == row["exhausted"]
                && serde_json::to_value(first)? == row["first"]
                && serde_json::to_value(second)? == row["second"];
            dependent_equal += usize::from(equal);
            dependent_rows.push(json!({"id":e.id,"control":c,"equal":equal}));
        }

        gate &= dependent_rows.len() == 9984 && dependent_equal == 9984;
        for previous in ["language-scheduling-1", "language-credit-1"] {
            let pd: Value = serde_json::from_slice(&std::fs::read(
                evidence.join(previous).join("attempt-1/data.json"),
            )?)?;
            let inputs: Vec<schedule_data::Example> =
                serde_json::from_value(pd["development"].clone())?;
            let inputs: BTreeMap<_, _> = inputs.iter().map(|e| (e.id.as_str(), e)).collect();
            let pr: Vec<Value> = serde_json::from_slice(&std::fs::read(
                evidence.join(previous).join("attempt-1/responses.json"),
            )?)?;
            let mut items = Vec::new();
            let mut equal = 0;
            for row in pr.iter().filter(|r| r["artifact"] == "candidate") {
                let e = inputs
                    .get(row["id"].as_str().ok_or("retained id")?)
                    .ok_or("retained input")?;
                let c: Control = serde_json::from_value(row["control"].clone())?;
                let out = runtime::generate(&a, &g, &m, &e.records, &e.prompt, c)?;
                let actual = record(&out);
                let same = [
                    "tokens",
                    "exhausted",
                    "sources",
                    "actions",
                    "policy_rows",
                    "final_reads",
                ]
                .iter()
                .all(|k| actual[k] == row[k]);
                equal += usize::from(same);
                items.push(json!({"id":e.id,"control":c,"equal":same}));
            }
            gate &= items.len() == 19968 && equal == 19968;
            retained
                .push(json!({"previous":previous,"rows":items.len(),"equal":equal,"items":items}));
        }
        gate &= old["equal"] == true;
        write(
            &root,
            "retained-responses.json",
            &json!({"earlier":old,"dependent_rows":dependent_rows,"dependent_equal":dependent_equal,"scheduling":retained}),
        )?;
        for variant in ["baseline", "first", "middle", "terminal", "inactive"] {
            let e = dev.iter().find(|e| e.variant == variant).ok_or("sample")?;
            let out = full.get(&e.id).ok_or("sample output")?;
            write(
                &root,
                &format!("sample-{variant}.json"),
                &json!({"example":e,"generated":out}),
            )?;
        }
        write(
            &root,
            "summary.json",
            &json!({"gate":if gate{"PASS_THIRD_LANGUAGE_READ"}else{"FAIL_THIRD_LANGUAGE_READ"},"panels":panels,"complete_families":complete,"development_rows":dev.len(),"training_rows":0,"fits":0,"candidate_sha256":sha(&bytes),"parent_sha256":sha(&bytes),"artifact_bytes_unchanged":a.encode()?==bytes,"reader_rules":a.parent.parent.rules,"update_rules":a.parent.rules,"actions":a.actions,"reload_equal":dev.len(),"retention_equal":old["equal"]==true&&dependent_equal==9984&&retained.iter().all(|x|x["equal"]==19968),"retained_counts":{"old_outputs":old["old_outputs"],"recurrent":old["recurrent_controls"],"ordered":old["ordered_controls"],"language":old["language_controls"],"old_language_through_relative":old["old_language_through_relative"],"relative":old["relative_controls"],"dependent":dependent_equal,"scheduling":19968,"credit":19968},"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
        )?;
        Ok(())
    })();
    if let Err(e) = &result {
        std::fs::write(root.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}
