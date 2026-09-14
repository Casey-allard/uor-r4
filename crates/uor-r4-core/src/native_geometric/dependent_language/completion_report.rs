//! Typed completion qualification with separately labelled legacy replay and
//! successful-task transfer through the new policy.
use super::{
    completion as runtime, completion_data as data, completion_learning as learning, depth_data,
    runtime as binding, schedule_data, scheduling,
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
fn legacy_record(out: &scheduling::Generated) -> Value {
    let sources: Vec<_> = out
        .steps
        .iter()
        .filter(|step| {
            step.before.core.cursor == 0 && matches!(step.action, Action::Read | Action::Emit)
        })
        .filter_map(|step| {
            step.observation
                .route
                .selected
                .as_ref()
                .map(|value| [value.source, value.word])
        })
        .collect();
    json!({"tokens":out.tokens,"exhausted":out.exhausted,"sources":sources,
        "actions":out.steps.iter().map(|step| step.action).collect::<Vec<_>>(),
        "policy_rows":out.steps.iter().map(|step| step.observation.core.row).collect::<Vec<_>>(),
        "final_reads":out.steps.last().map(|step| step.after.core.reads)})
}
fn selected_prefix(out: &runtime::Generated) -> Vec<[usize; 2]> {
    // An Unresolved decision contributes the current successfully selected value.
    out.decisions
        .iter()
        .filter(|decision| decision.before.core.cursor == 0)
        .filter_map(|decision| decision.selected)
        .collect()
}
pub(super) fn legacy_retention(
    parent: &scheduling::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let earlier = super::report::retained(&parent.parent, g, m, evidence)?;
    let pd: Value = serde_json::from_slice(&std::fs::read(
        evidence.join("dependent-language-1/attempt-2/data.json"),
    )?)?;
    let inputs: Vec<super::data::Example> = serde_json::from_value(pd["development"].clone())?;
    let previous: Vec<Value> = serde_json::from_slice(&std::fs::read(
        evidence.join("dependent-language-1/attempt-2/responses.json"),
    )?)?;
    let mut dependent_rows = Vec::new();
    let mut dependent_equal = 0;
    for row in previous.iter().filter(|row| row["artifact"] == "candidate") {
        let e = inputs
            .iter()
            .find(|e| row["id"] == e.id)
            .ok_or("dependent input")?;
        let control: binding::Control = serde_json::from_value(row["control"].clone())?;
        let out = binding::generate(&parent.parent, g, m, &e.records, &e.prompt, control)?;
        let first = out
            .first
            .selected
            .as_ref()
            .map(|value| [value.source, value.word]);
        let second = out
            .output
            .as_ref()
            .and_then(|output| output.routes.first())
            .and_then(|route| route.selected.as_ref())
            .map(|value| [value.source, value.word]);
        let equal = serde_json::to_value(out.tokens())? == row["tokens"]
            && serde_json::to_value(out.exhausted())? == row["exhausted"]
            && serde_json::to_value(first)? == row["first"]
            && serde_json::to_value(second)? == row["second"];
        dependent_equal += usize::from(equal);
        dependent_rows.push(json!({"id":e.id,"control":control,"equal":equal}));
    }
    let mut equal_all =
        earlier["equal"] == true && dependent_rows.len() == 9984 && dependent_equal == 9984;
    let mut scheduling_panels = Vec::new();
    for (previous, required) in [
        ("language-scheduling-1", 19968),
        ("language-credit-1", 19968),
        ("language-depth-1", 19200),
    ] {
        let pd: Value = serde_json::from_slice(&std::fs::read(
            evidence.join(previous).join("attempt-1/data.json"),
        )?)?;
        let cases: Vec<(String, [Vec<u8>; 4], Vec<u8>)> = if previous == "language-depth-1" {
            serde_json::from_value::<Vec<depth_data::Example>>(pd["development"].clone())?
                .into_iter()
                .map(|e| (e.id, e.records, e.prompt))
                .collect()
        } else {
            serde_json::from_value::<Vec<schedule_data::Example>>(pd["development"].clone())?
                .into_iter()
                .map(|e| (e.id, e.records, e.prompt))
                .collect()
        };
        let cases: BTreeMap<_, _> = cases.iter().map(|e| (e.0.as_str(), e)).collect();
        let rows: Vec<Value> = serde_json::from_slice(&std::fs::read(
            evidence.join(previous).join("attempt-1/responses.json"),
        )?)?;
        let mut items = Vec::new();
        let mut equal = 0;
        let mut identities = BTreeSet::new();
        for row in rows
            .iter()
            .filter(|row| previous == "language-depth-1" || row["artifact"] == "candidate")
        {
            let id = row["id"].as_str().ok_or("legacy id")?;
            let case = cases.get(id).ok_or("legacy input")?;
            if !identities.insert((id, row["control"].to_string())) {
                return Err("duplicate legacy control".into());
            }
            let control: scheduling::Control = serde_json::from_value(row["control"].clone())?;
            let out = scheduling::generate(parent, g, m, &case.1, &case.2, control)?;
            let actual = legacy_record(&out);
            let same = [
                "tokens",
                "exhausted",
                "sources",
                "actions",
                "policy_rows",
                "final_reads",
            ]
            .iter()
            .all(|field| actual[field] == row[field]);
            equal += usize::from(same);
            items.push(json!({"id":id,"control":control,"equal":same}));
        }
        equal_all &= items.len() == required && equal == required;
        scheduling_panels
            .push(json!({"previous":previous,"rows":items.len(),"equal":equal,"items":items}));
    }
    Ok(
        json!({"scope":"legacy adapters and frozen parent controls, not corrected negative outputs",
        "equal":equal_all,"earlier":earlier,"dependent_rows":dependent_rows,"dependent_equal":dependent_equal,"scheduling":scheduling_panels,
        "counts":{"old_outputs":earlier["old_outputs"],"recurrent":earlier["recurrent_controls"],"ordered":earlier["ordered_controls"],
            "language":earlier["language_controls"],"old_language_through_relative":earlier["old_language_through_relative"],"relative":earlier["relative_controls"],
            "dependent":dependent_equal,"scheduling":19968,"credit":19968,"depth":19200}}),
    )
}
#[test]
#[ignore = "Learn typed completion with frozen reader; exclusive report"]
fn typed_language_completion_report() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_COMPLETION_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_COMPLETION_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":896,"development_rows":896,
            "full_typed_targets":896,"full_resolved_prefixes":896,"answered":384,"unresolved":512,"unresolved_per_reason":256,
            "families":128,"family_size":7,"initial_valid_correct":384,"initial_unresolved_correct":0,"initialization":"both pending banks copied from retained scheduling table","controls":["PendingHidden","UnresolvedDisabled"],
            "control_valid_correct":384,"control_unresolved_correct":0,"collapsed_pending_conflicts_required":true,
            "extended_conflicts_allowed":0,"parent_parameters_unchanged":true,"reload_rows":896,
            "new_full_legacy_transfer":{"scheduling":1536,"depth":1280},
            "legacy_control_replay":{"dependent":9984,"scheduling":19968,"credit":19968,"depth":19200},
            "split":"name offsets0..3 training,4..7 development; overlapping familiar vocabulary and grammar",
            "scope":"learned completion versus typed unresolved outcome for authored pending continuations; not global absence or general prose",
            "final_holdout":"NOT_RUN","promotion":false}),
        )?;
        let frozen = evidence.join("language-completion-1/diagnostic-1");
        report_output::verify(&frozen)?;
        let frozen_bytes = std::fs::read(frozen.join("data.json"))?;
        let frozen_data: Value = serde_json::from_slice(&frozen_bytes)?;
        let diagnostic_responses: Vec<Value> =
            serde_json::from_slice(&std::fs::read(frozen.join("responses.json"))?)?;
        let diagnostic_by_id: BTreeMap<_, _> = diagnostic_responses
            .iter()
            .map(|row| Ok((row["id"].as_str().ok_or("diagnostic response id")?, row)))
            .collect::<Result<_, Box<dyn std::error::Error>>>()?;
        let frozen_rows: Vec<data::Example> =
            serde_json::from_value(frozen_data["examples"].clone())?;
        let all = data::corpus()?;
        if all != frozen_rows {
            return Err("diagnostic corpus changed before correction".into());
        }
        let (train, dev) = learning::split(&all);
        if train.len() != 896 || dev.len() != 896 {
            return Err("completion split sizes".into());
        }
        write(
            &output,
            "data.json",
            &json!({"training":train,"development":dev,"validation":data::validate(&all)?,
            "frozen_diagnostic_data_sha256":sha(&frozen_bytes),"corpus_unchanged_since_diagnostic":true}),
        )?;
        let parent_bytes =
            std::fs::read(evidence.join("language-depth-1/attempt-1/candidate.json"))?;
        if sha(&parent_bytes) != "62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95"
        {
            return Err("parent hash".into());
        }
        let geometry = BoundGeometry::canonical()?;
        let metric = Metric::new(&geometry)?;
        let parent = scheduling::Artifact::decode(&parent_bytes, &geometry)?;
        let initial = learning::initialize(parent.clone(), &train)?;
        if initial.actions.len() != 16
            || initial.actions[..8] != parent.actions
            || initial.actions[8..] != parent.actions
        {
            return Err("initial table does not preserve both parent policy banks".into());
        }
        std::fs::write(output.join("initial.json"), initial.encode()?)?;
        let prepared = learning::prepare(&initial, &geometry, &metric, &train)?;
        write(
            &output,
            "preparation.json",
            &serde_json::to_value(&prepared)?,
        )?;
        if !prepared.missing.is_empty() || !prepared.conflicts.is_empty() {
            write(
                &output,
                "summary.json",
                &json!({"gate":"FAIL_TYPED_LANGUAGE_COMPLETION_REPRESENTATION","fit":"NOT_RUN","promotion":false}),
            )?;
            return Ok(());
        }
        let (candidate, fit) = learning::fit(&initial, &geometry, &metric, &train, &prepared)?;
        let bytes = candidate.encode()?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        write(&output, "fit.json", &serde_json::to_value(&fit)?)?;
        if candidate.parent.encode()? != parent_bytes {
            return Err("frozen parent changed".into());
        }
        let reload = runtime::Artifact::decode(&bytes, &geometry)?;
        let mut bad = candidate.clone();
        bad.actions[0] = 4;
        if bad.validate(&geometry).is_ok() {
            return Err("invalid completion action accepted".into());
        }
        let mut bad = candidate.clone();
        bad.parent_digest[0] ^= 1;
        if bad.validate(&geometry).is_ok() {
            return Err("invalid parent digest accepted".into());
        }
        let mut responses = Vec::new();
        let mut panels = Vec::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut gate = fit.training_exact == 896 && !prepared.collapsed_conflicts.is_empty();
        let mut pending_hidden_diagnostic_equal = 0;
        for (name, a, control) in [
            ("initial", &initial, runtime::Control::Full),
            ("candidate", &candidate, runtime::Control::Full),
            ("candidate", &candidate, runtime::Control::PendingHidden),
            (
                "candidate",
                &candidate,
                runtime::Control::UnresolvedDisabled,
            ),
        ] {
            let mut correct = 0;
            let mut prefixes = 0;
            let mut answered = 0;
            let mut unresolved = 0;
            let mut reasons = BTreeMap::<String, usize>::new();
            for e in &dev {
                let out = runtime::generate(a, &geometry, &metric, &e.records, &e.prompt, control)?;
                let target_ok = learning::matches_target(&out, e);
                if control == runtime::Control::PendingHidden {
                    let before = diagnostic_by_id
                        .get(e.id.as_str())
                        .ok_or("missing frozen diagnostic response")?;
                    let same = serde_json::to_value(&out.trace.tokens)? == before["actual_tokens"]
                        && serde_json::to_value(out.trace.exhausted)? == before["actual_exhausted"]
                        && serde_json::to_value(
                            out.trace.steps.last().map(|step| step.after.clause),
                        )? == before["actual_final_clause"];
                    pending_hidden_diagnostic_equal += usize::from(same);
                    gate &= same;
                }
                let actual_prefix = selected_prefix(&out);
                let prefix_ok = actual_prefix == e.resolved_prefix;
                let clause_ok = match &out.outcome {
                    runtime::Outcome::Answered => e.blocked_at.is_none(),
                    runtime::Outcome::Unresolved { clause, .. } => Some(*clause) == e.blocked_at,
                    runtime::Outcome::Exhausted => false,
                };
                let un_emitted = !matches!(&out.outcome, runtime::Outcome::Unresolved { .. })
                    || (out.trace.tokens.is_empty() && !out.trace.exhausted);
                let yes = target_ok && clause_ok && un_emitted;
                correct += usize::from(yes);
                prefixes += usize::from(prefix_ok);
                if e.answer.is_some() {
                    answered += usize::from(yes);
                } else {
                    unresolved += usize::from(yes);
                    *reasons.entry(e.expected_reason.clone()).or_default() += usize::from(yes);
                }
                if name == "candidate" && control == runtime::Control::Full {
                    let second = runtime::generate(
                        &reload, &geometry, &metric, &e.records, &e.prompt, control,
                    )?;
                    if serde_json::to_value(&out)? != serde_json::to_value(second)? {
                        return Err("reload mismatch".into());
                    }
                    if out
                        .trace
                        .steps
                        .windows(2)
                        .any(|pair| pair[0].after != pair[1].before)
                    {
                        return Err("core state continuity".into());
                    }
                    if yes && prefix_ok {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    if e.family == dev[0].family {
                        write(
                            &output,
                            &format!("sample-depth{}-{}.json", e.depth, e.variant),
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                }
                responses.push(json!({"id":e.id,"family":e.family,"depth":e.depth,"variant":e.variant,"artifact":name,"control":control,
                    "correct":yes,"target_matches":target_ok,"resolved_prefix_correct":prefix_ok,"resolved_prefix":actual_prefix,"expected_prefix":e.resolved_prefix,
                    "outcome":out.outcome,"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"decisions":out.decisions}));
            }
            panels.push(json!({"artifact":name,"control":control,"correct":correct,"prefixes":prefixes,"answered":answered,"unresolved":unresolved,"reasons":reasons,"rows":dev.len()}));
            if name == "initial" {
                gate &= correct == 384 && answered == 384 && unresolved == 0;
            } else if control == runtime::Control::Full {
                gate &= correct == 896
                    && prefixes == 896
                    && answered == 384
                    && unresolved == 512
                    && reasons.get("NoCompatibleCandidate") == Some(&256)
                    && reasons.get("Ambiguous") == Some(&256);
            } else {
                gate &= answered == 384 && unresolved == 0;
            }
        }
        let complete_families = families.values().filter(|&&count| count == 7).count();
        gate &= families.len() == 128
            && complete_families == 128
            && pending_hidden_diagnostic_equal == 896;
        write(&output, "responses.json", &json!(responses))?;
        let legacy = legacy_retention(&parent, &geometry, &metric, &evidence)?;
        let mut transfer_panels = Vec::new();
        let mut transferred = 0;
        for (previous, required) in [("language-scheduling-1", 1536), ("language-depth-1", 1280)] {
            let pd: Value = serde_json::from_slice(&std::fs::read(
                evidence.join(previous).join("attempt-1/data.json"),
            )?)?;
            let cases: Vec<(String, [Vec<u8>; 4], Vec<u8>)> = if previous == "language-depth-1" {
                serde_json::from_value::<Vec<depth_data::Example>>(pd["development"].clone())?
                    .into_iter()
                    .map(|e| (e.id, e.records, e.prompt))
                    .collect()
            } else {
                serde_json::from_value::<Vec<schedule_data::Example>>(pd["development"].clone())?
                    .into_iter()
                    .map(|e| (e.id, e.records, e.prompt))
                    .collect()
            };
            let mut equal = 0;
            let mut items = Vec::new();
            for (id, records, prompt) in cases {
                let before = scheduling::generate(
                    &parent,
                    &geometry,
                    &metric,
                    &records,
                    &prompt,
                    scheduling::Control::Full,
                )?;
                let after = runtime::generate(
                    &candidate,
                    &geometry,
                    &metric,
                    &records,
                    &prompt,
                    runtime::Control::Full,
                )?;
                let same =
                    after.trace == before && matches!(after.outcome, runtime::Outcome::Answered);
                equal += usize::from(same);
                items.push(json!({"id":id,"full_core_trace_equal":same}));
            }
            gate &= items.len() == required && equal == required;
            transferred += equal;
            transfer_panels
                .push(json!({"previous":previous,"rows":items.len(),"equal":equal,"items":items}));
        }
        gate &= legacy["equal"] == true && transferred == 2816;
        write(
            &output,
            "retained-responses.json",
            &json!({"legacy_control_replay":legacy,
            "new_full_transfer":{"equal":transferred==2816,"rows":2816,"panels":transfer_panels},
            "scope":"legacy controls are replayed through the unchanged parent; successful Full traces transfer through the new completion policy"}),
        )?;
        write(
            &output,
            "summary.json",
            &json!({"gate":if gate {"PASS_TYPED_LANGUAGE_COMPLETION"} else {"FAIL_TYPED_LANGUAGE_COMPLETION"},
            "training_exact":fit.training_exact,"training_rows":train.len(),"development_rows":dev.len(),"panels":panels,"complete_families":complete_families,
            "initialization":"both banks copied from retained policy","initial_actions":initial.actions,
            "pending_hidden_frozen_diagnostic_equal":pending_hidden_diagnostic_equal,
            "actions":candidate.actions,"changed_rows":fit.changed_rows,"supervised_rows":fit.supervised_rows,
            "collapsed_conflicts":prepared.collapsed_conflicts,"extended_conflicts":prepared.conflicts,
            "candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"parent_parameters_unchanged":candidate.parent.encode()?==parent_bytes,
            "corpus_unchanged_since_diagnostic":true,"reload_equal":896,"legacy_control_replay_equal":legacy["equal"],
            "legacy_control_replay_counts":legacy["counts"],"new_full_transfer_equal":transferred==2816,"new_full_transfer_rows":transferred,
            "final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
        )?;
        Ok(())
    })();
    if let Err(error) = &result {
        std::fs::write(output.join("error.txt"), error.to_string())?;
    }
    report_output::seal(&output)?;
    report_output::verify(&output)?;
    result
}
