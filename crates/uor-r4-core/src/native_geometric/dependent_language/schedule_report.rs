use super::{
    runtime as binding, schedule_data as data, schedule_learning as learning,
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
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(path: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn source_path(out: &runtime::Generated) -> Vec<[usize; 2]> {
    out.steps
        .iter()
        .filter(|s| s.before.core.cursor == 0 && matches!(s.action, Action::Read | Action::Emit))
        .filter_map(|s| {
            s.observation
                .route
                .selected
                .as_ref()
                .map(|x| [x.source, x.word])
        })
        .collect()
}
#[test]
#[ignore = "Learn shared raw-language action policy; exclusive source-bound report"]
fn shared_language_scheduling_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_SCHEDULING_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_SCHEDULING_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":4096,"development_rows":1536,"required_full_exact":1536,"required_exact_paths":1536,"required_complete_families":96,"per_kind_split_rows":256,"required_zero":["PolicyDisabled","ReadDisabled","AlwaysRead","CursorDisabled","StopDisabled"],"required_direct_only":["ContinuationDisabled","EmitInsteadOfRead","UpdateDisabled","PayloadReversed","ScorerDisabled"],"diagnostic_only":["ExactIdentity","FeedbackDisabled"],"policy_rows":8,"features":runtime::FEATURES,"search_node_limit":512,"max_steps":runtime::MAX_STEPS,"final_holdout":"NOT_RUN","scope":"one shared loop for existing direct/dependent language; explicit punctuation and usable-continuation feature; frozen primitives"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent_bytes =
            std::fs::read(evidence.join("dependent-language-1/attempt-2/candidate.json"))?;
        if sha(&parent_bytes) != "b44de7fa42ddb1e2728d4e2d950bc91d1ad2320f85fcb67a615b057f3c3612e2"
        {
            return Err("parent hash".into());
        }
        let parent = binding::Artifact::decode(&parent_bytes, &g)?;
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
        write(&path, "preparation.json", &serde_json::to_value(&prepared)?)?;
        if !prepared.missing.is_empty() || !prepared.conflicts.is_empty() {
            write(
                &path,
                "summary.json",
                &json!({"gate":"FAIL_SHARED_LANGUAGE_REPRESENTATION","fit":"NOT_RUN","promotion":false}),
            )?;
            return Ok(());
        }
        let (candidate, fit) = learning::fit(&initial, &g, &m, &train, &prepared)?;
        let bytes = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &bytes)?;
        write(&path, "fit.json", &serde_json::to_value(&fit)?)?;
        let reload = runtime::Artifact::decode(&bytes, &g)?;
        let mut bad = candidate.clone();
        bad.actions[0] = 3;
        if bad.validate(&g).is_ok() {
            return Err("invalid action accepted".into());
        }
        let mut bad = candidate.clone();
        bad.parent.rules.clear();
        if bad.validate(&g).is_ok() {
            return Err("changed parent accepted".into());
        }
        let mut rows = Vec::new();
        let mut panels = Vec::new();
        let mut splits = BTreeMap::<String, [usize; 3]>::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut seen_rows = BTreeSet::new();
        let mut full_rows = Vec::new();
        let mut gate = true;
        for (name, a, c) in [
            ("initial", &initial, Control::Full),
            ("candidate", &candidate, Control::Full),
            ("candidate", &candidate, Control::PolicyDisabled),
            ("candidate", &candidate, Control::ReadDisabled),
            ("candidate", &candidate, Control::ContinuationDisabled),
            ("candidate", &candidate, Control::AlwaysRead),
            ("candidate", &candidate, Control::EmitInsteadOfRead),
            ("candidate", &candidate, Control::UpdateDisabled),
            ("candidate", &candidate, Control::PayloadReversed),
            ("candidate", &candidate, Control::ScorerDisabled),
            ("candidate", &candidate, Control::CursorDisabled),
            ("candidate", &candidate, Control::StopDisabled),
            ("candidate", &candidate, Control::ExactIdentity),
            ("candidate", &candidate, Control::FeedbackDisabled),
        ] {
            let mut exact = 0;
            let mut paths = 0;
            let mut kinds = BTreeMap::<String, usize>::new();
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, &e.prompt, c)?;
                let path_actual = source_path(&out);
                let yes = !out.exhausted && out.tokens == learning::target(e);
                let path_yes = path_actual == e.expected_path;
                exact += usize::from(yes);
                paths += usize::from(path_yes);
                *kinds.entry(e.kind.clone()).or_default() += usize::from(yes);
                if name == "candidate" && c == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, &e.prompt, c)? {
                        return Err("reload mismatch".into());
                    }
                    if out.steps.windows(2).any(|s| s[0].after != s[1].before)
                        || out.steps.iter().any(|s| {
                            s.action == Action::Read
                                && (s.token.is_some()
                                    || s.after.core.reads != s.before.core.reads + 1
                                    || s.after.clause != s.before.clause + 1)
                        })
                    {
                        return Err("state continuity".into());
                    }
                    for s in &out.steps {
                        seen_rows.insert(s.observation.core.row);
                    }
                    let n = splits.entry(format!("{}:{}", e.kind, e.split)).or_default();
                    n[0] += usize::from(yes);
                    n[1] += usize::from(path_yes);
                    n[2] += 1;
                    if yes && path_yes {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    if e.id == "direct:joint-000-00-base" || e.id == "dependent:joint-000-00-base" {
                        write(
                            &path,
                            &format!("sample-{}.json", e.kind),
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                    full_rows.push((e.id.clone(), out.clone()));
                }
                rows.push(json!({"id":e.id,"family":e.family,"kind":e.kind,"split":e.split,"artifact":name,"control":c,"exact":yes,"path_correct":path_yes,"sources":path_actual,"tokens":out.tokens,"exhausted":out.exhausted,"actions":out.steps.iter().map(|s|s.action).collect::<Vec<_>>(),"policy_rows":out.steps.iter().map(|s|s.observation.core.row).collect::<Vec<_>>(),"final_reads":out.steps.last().map(|s|s.after.core.reads)}));
            }
            panels.push(json!({"artifact":name,"control":c,"exact":exact,"paths":paths,"kinds":kinds,"total":dev.len()}));
            if name == "initial" {
                gate &= exact == 0;
            } else {
                match c {
                    Control::Full => gate &= exact == 1536 && paths == 1536,
                    Control::PolicyDisabled
                    | Control::ReadDisabled
                    | Control::AlwaysRead
                    | Control::CursorDisabled
                    | Control::StopDisabled => gate &= exact == 0,
                    Control::ContinuationDisabled
                    | Control::EmitInsteadOfRead
                    | Control::UpdateDisabled
                    | Control::PayloadReversed
                    | Control::ScorerDisabled => {
                        gate &=
                            kinds.get("direct") == Some(&768) && kinds.get("dependent") == Some(&0)
                    }
                    _ => {}
                }
            }
        }
        let old = super::report::retained(&candidate.parent, &g, &m, &evidence)?;
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
            let out = binding::generate(&candidate.parent, &g, &m, &e.records, &e.prompt, c)?;
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
        write(
            &path,
            "retained-responses.json",
            &json!({"earlier":old,"dependent_rows":dependent_rows,"dependent_equal":dependent_equal}),
        )?;
        let complete = families.values().filter(|&&n| n == 16).count();
        gate &= fit.training_exact == 4096
            && complete == 96
            && splits.len() == 6
            && splits.values().all(|x| *x == [256, 256, 256])
            && old["equal"] == true
            && dependent_equal == 9984;
        write(&path, "responses.json", &json!(rows))?;
        // Trace samples selected by kind rather than any particular source ID.
        for kind in ["direct", "dependent"] {
            let e = dev
                .iter()
                .find(|e| e.kind == kind && e.split == "joint")
                .ok_or("sample")?;
            let out = full_rows
                .iter()
                .find(|(id, _)| id == &e.id)
                .ok_or("sample output")?;
            write(
                &path,
                &format!("sample-{kind}.json"),
                &json!({"example":e,"generated":out.1}),
            )?;
        }
        write(
            &path,
            "summary.json",
            &json!({"gate":if gate{"PASS_SHARED_LANGUAGE_SCHEDULING"}else{"FAIL_SHARED_LANGUAGE_SCHEDULING"},"training_exact":fit.training_exact,"panels":panels,"splits":splits,"complete_families":complete,"learned_actions":candidate.actions,"supervised_rows":fit.supervised_rows,"development_feature_rows":seen_rows,"all_development_rows_seen_in_training":seen_rows.iter().all(|r|fit.supervised_rows.contains(r)),"reload_equal":1536,"retention_equal":old["equal"]==true&&dependent_equal==9984,"retained_counts":{"old_outputs":old["old_outputs"],"recurrent":old["recurrent_controls"],"ordered":old["ordered_controls"],"language":old["language_controls"],"old_language_through_relative":old["old_language_through_relative"],"relative":old["relative_controls"],"dependent":dependent_equal},"candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
