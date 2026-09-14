use super::{
    joint_learning as learning, runtime as binding, schedule_data as data, schedule_learning,
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
#[ignore = "Shared final-output reader and update credit; exclusive source-bound report"]
fn shared_language_output_credit_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_CREDIT_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_CREDIT_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty path".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":4096,"development_rows":1536,"required_full_exact":1536,"required_exact_paths":1536,"required_complete_families":96,"per_kind_split_rows":256,"required_zero":["PolicyDisabled","ReadDisabled","AlwaysRead","CursorDisabled","StopDisabled","ClearReader"],"required_direct_only":["ContinuationDisabled","EmitInsteadOfRead","UpdateDisabled","PayloadReversed","ScorerDisabled","ClearUpdater"],"diagnostic_only":["ExactIdentity","FeedbackDisabled","BootstrapComparison"],"required_preserved_blocks":4,"initial_reader_and_updater_cleared":true,"writer_and_scheduling_frozen":true,"mixed_reader_credit_sites":6144,"mixed_reader_credit_covered":6144,"nonlocal_positive_minimum":1,"bootstrap_rowwise_nonregression":true,"incremental_improvement_required":false,"policy_rows":8,"features":runtime::FEATURES,"max_steps":runtime::MAX_STEPS,"final_holdout":"NOT_RUN","scope":"discrete block-coordinate final-output reader/update credit with explicit direct bootstrap; shared scheduling and writer frozen; no joint gradient or general-language claim"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent_bytes =
            std::fs::read(evidence.join("language-scheduling-1/attempt-1/candidate.json"))?;
        if sha(&parent_bytes) != "65c28a3ae06148bf7defd0acdc6c1b25928f35febc1f964520309831e209ed22"
        {
            return Err("parent hash".into());
        }
        let parent = runtime::Artifact::decode(&parent_bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate()?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        let outcome = learning::run(parent.clone(), &g, &m, &train, &path)?;
        let initial = outcome.initial;
        let bootstrap = outcome.bootstrap;
        let candidate = outcome.candidate;
        let blocks = outcome.blocks;
        let training_exact = blocks.last().map(|block| block.after_exact).unwrap_or(0);
        let mixed = blocks
            .iter()
            .find(|block| block.name == "reader-mixed-credit");
        let mixed_credit_ok = mixed.is_some_and(|block| {
            block.sites == 6144 && block.covered == 6144 && block.nonlocal_positive > 0
        });
        let bytes = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &bytes)?;
        std::fs::write(path.join("bootstrap.json"), bootstrap.encode()?)?;
        write(&path, "blocks.json", &serde_json::to_value(&blocks)?)?;
        let mut clear_reader = candidate.clone();
        clear_reader.parent.parent.rules.clear();
        learning::rebind(&mut clear_reader, &train)?;
        let mut clear_updater = candidate.clone();
        clear_updater.parent.rules.clear();
        learning::rebind(&mut clear_updater, &train)?;
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
        let mut bootstrap_rows = BTreeMap::<String, [bool; 2]>::new();
        let mut bootstrap_exact = 0;
        let mut final_exact = 0;
        let mut final_not_worse_than_bootstrap = true;
        for (name, a, c) in [
            ("initial", &initial, Control::Full),
            ("bootstrap", &bootstrap, Control::Full),
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
            ("clear-reader", &clear_reader, Control::Full),
            ("clear-updater", &clear_updater, Control::Full),
        ] {
            let mut exact = 0;
            let mut paths = 0;
            let mut kinds = BTreeMap::<String, usize>::new();
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, &e.prompt, c)?;
                let path_actual = source_path(&out);
                let yes = !out.exhausted && out.tokens == schedule_learning::target(e);
                let path_yes = path_actual == e.expected_path;
                exact += usize::from(yes);
                paths += usize::from(path_yes);
                *kinds.entry(e.kind.clone()).or_default() += usize::from(yes);
                if name == "bootstrap" {
                    bootstrap_rows.insert(e.id.clone(), [yes, path_yes]);
                }
                if name == "candidate" && c == Control::Full {
                    let before = bootstrap_rows.get(&e.id).ok_or("bootstrap input missing")?;
                    final_not_worse_than_bootstrap &=
                        (!before[0] || yes) && (!before[1] || path_yes);
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
                    full_rows.push((e.id.clone(), out.clone()));
                }
                let label = match name {
                    "clear-reader" => json!("ClearReader"),
                    "clear-updater" => json!("ClearUpdater"),
                    _ => serde_json::to_value(c)?,
                };
                rows.push(json!({"id":e.id,"family":e.family,"kind":e.kind,"split":e.split,"artifact":name,"control":label,"exact":yes,"path_correct":path_yes,"sources":path_actual,"tokens":out.tokens,"exhausted":out.exhausted,"actions":out.steps.iter().map(|s|s.action).collect::<Vec<_>>(),"policy_rows":out.steps.iter().map(|s|s.observation.core.row).collect::<Vec<_>>(),"final_reads":out.steps.last().map(|s|s.after.core.reads)}));
            }
            let label = match name {
                "clear-reader" => json!("ClearReader"),
                "clear-updater" => json!("ClearUpdater"),
                _ => serde_json::to_value(c)?,
            };
            panels.push(json!({"artifact":name,"control":label,"exact":exact,"paths":paths,"kinds":kinds,"total":dev.len()}));
            if name == "initial" {
                gate &= exact == 0;
            } else if name == "bootstrap" {
                bootstrap_exact = exact;
            } else if name == "clear-reader" {
                gate &= exact == 0;
            } else if name == "clear-updater" {
                gate &= kinds.get("direct") == Some(&768) && kinds.get("dependent") == Some(&0);
            } else {
                match c {
                    Control::Full => {
                        final_exact = exact;
                        gate &= exact == 1536 && paths == 1536;
                    }
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
        let scheduler_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("language-scheduling-1/attempt-1/data.json"),
        )?)?;
        let scheduler_dev: Vec<data::Example> =
            serde_json::from_value(scheduler_data["development"].clone())?;
        let scheduler_responses: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("language-scheduling-1/attempt-1/responses.json"),
        )?)?;
        let mut scheduling_rows = Vec::new();
        let mut scheduling_equal = 0;
        let mut scheduling_all_equal = true;
        let mut scheduling_ids = BTreeSet::new();
        for row in scheduler_responses
            .as_array()
            .ok_or("scheduling rows")?
            .iter()
            .filter(|row| row["artifact"] == "candidate")
        {
            let e = scheduler_dev
                .iter()
                .find(|e| row["id"] == e.id)
                .ok_or("scheduling input")?;
            let c: Control = serde_json::from_value(row["control"].clone())?;
            if !scheduling_ids.insert((e.id.clone(), row["control"].to_string())) {
                return Err("duplicate retained scheduling control".into());
            }
            let out = runtime::generate(&candidate, &g, &m, &e.records, &e.prompt, c)?;
            let actions: Vec<_> = out.steps.iter().map(|step| step.action).collect();
            let policy_rows: Vec<_> = out
                .steps
                .iter()
                .map(|step| step.observation.core.row)
                .collect();
            let final_reads = out.steps.last().map(|step| step.after.core.reads);
            let equal = serde_json::to_value(&out.tokens)? == row["tokens"]
                && serde_json::to_value(out.exhausted)? == row["exhausted"]
                && serde_json::to_value(source_path(&out))? == row["sources"]
                && serde_json::to_value(actions)? == row["actions"]
                && serde_json::to_value(policy_rows)? == row["policy_rows"]
                && serde_json::to_value(final_reads)? == row["final_reads"];
            scheduling_equal += usize::from(equal);
            scheduling_all_equal &= equal;
            scheduling_rows.push(json!({"id":e.id,"control":c,"equal":equal}));
        }
        write(
            &path,
            "retained-responses.json",
            &json!({"earlier":old,
            "dependent_rows":dependent_rows,"dependent_equal":dependent_equal,
            "scheduling_rows":scheduling_rows,"scheduling_equal":scheduling_equal,
            "scheduling_all_equal":scheduling_all_equal}),
        )?;
        let complete = families.values().filter(|&&n| n == 16).count();
        gate &= training_exact == 4096
            && initial.parent.parent.rules.is_empty()
            && initial.parent.rules.is_empty()
            && candidate.parent.parent.parent == parent.parent.parent.parent
            && blocks.last().is_some_and(|block| {
                block.reader_rules == candidate.parent.parent.rules
                    && block.update_rules == candidate.parent.rules
            })
            && blocks.len() == 4
            && blocks.iter().all(|block| block.preserved)
            && mixed_credit_ok
            && !candidate.parent.parent.rules.is_empty()
            && !candidate.parent.rules.is_empty()
            && candidate.actions == parent.actions
            && bootstrap_rows.len() == 1536
            && final_not_worse_than_bootstrap
            && complete == 96
            && splits.len() == 6
            && splits.values().all(|x| *x == [256, 256, 256])
            && old["equal"] == true
            && dependent_rows.len() == 9984
            && dependent_equal == 9984
            && scheduling_rows.len() == 19968
            && scheduling_equal == 19968
            && scheduling_all_equal;
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
            &json!({"gate":if gate{"PASS_SHARED_LANGUAGE_OUTPUT_CREDIT"}else{"FAIL_SHARED_LANGUAGE_OUTPUT_CREDIT"},
                "training_exact":training_exact,"panels":panels,"splits":splits,"complete_families":complete,
                "blocks":blocks,"mixed_credit_ok":mixed_credit_ok,
                "initial_reader_rules":initial.parent.parent.rules,"initial_update_rules":initial.parent.rules,
                "original_reader_rules":parent.parent.parent.rules,"original_update_rules":parent.parent.rules,
                "bootstrap_reader_rules":bootstrap.parent.parent.rules,"bootstrap_update_rules":bootstrap.parent.rules,
                "reader_rules":candidate.parent.parent.rules,"update_rules":candidate.parent.rules,
                "reader_rules_changed_from_original":candidate.parent.parent.rules!=parent.parent.parent.rules,
                "update_rules_changed_from_original":candidate.parent.rules!=parent.parent.rules,
                "bootstrap_exact":bootstrap_exact,"candidate_exact":final_exact,
                "final_not_worse_than_bootstrap_rowwise":final_not_worse_than_bootstrap,
                "mixed_credit_incremental_improvement":final_exact>bootstrap_exact,
                "learned_actions":candidate.actions,"scheduling_actions_unchanged":candidate.actions==parent.actions,
                "writer_artifact_unchanged":candidate.parent.parent.parent==parent.parent.parent.parent,
                "development_feature_rows":seen_rows,"reload_equal":1536,
                "retention_equal":old["equal"]==true&&dependent_equal==9984&&scheduling_all_equal&&scheduling_equal==19968,
                "retained_counts":{"old_outputs":old["old_outputs"],"recurrent":old["recurrent_controls"],"ordered":old["ordered_controls"],"language":old["language_controls"],"old_language_through_relative":old["old_language_through_relative"],"relative":old["relative_controls"],"dependent":dependent_equal,"scheduling":scheduling_equal},
                "candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
