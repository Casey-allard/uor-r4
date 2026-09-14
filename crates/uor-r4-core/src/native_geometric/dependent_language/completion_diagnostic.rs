//! Frozen-parameter diagnostic of final versus unusable pending continuation.
//! This report measures existing behavior without changing model or acceptance.
use super::{completion_data as data, runtime as binding, scheduling};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        relative_language::runtime as reader,
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
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
#[test]
#[ignore = "Measure frozen continuation completion; no fit or correction"]
fn continuation_completion_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_COMPLETION_DIAGNOSTIC_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_COMPLETION_DIAGNOSTIC_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty diagnostic path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "scope.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","mode":"descriptive diagnostic",
            "rows":1792,"fits":0,"model_correction":false,"outcomes":"actual emitted tokens, EOS, exhaustion, clause completion and next-route status",
            "reproduction_criterion":"authored unresolved row emits EOS without exhaustion before its last requested clause",
            "passing_behavior_threshold":null,"data_filtering":false,"promotion":false}),
        )?;
        let rows = data::corpus()?;
        write(
            &output,
            "data.json",
            &json!({"examples":rows,"validation":data::validate(&rows)?}),
        )?;
        let bytes = std::fs::read(evidence.join("language-depth-1/attempt-1/candidate.json"))?;
        if sha(&bytes) != "62da0bfaddb37452a5ec7eb1aa220fbda7b33c79ef3dbcd4da1dcb11293e7a95" {
            return Err("frozen diagnostic parent hash".into());
        }
        let geometry = BoundGeometry::canonical()?;
        let metric = Metric::new(&geometry)?;
        let artifact = scheduling::Artifact::decode(&bytes, &geometry)?;
        if artifact.encode()? != bytes {
            return Err("frozen artifact serialization changed".into());
        }
        let mut responses = Vec::new();
        let mut counts: BTreeMap<String, [usize; 4]> = BTreeMap::new();
        let mut valid_exact = 0;
        let mut unresolved = 0;
        let mut premature_completions = 0;
        let mut reason_counts: BTreeMap<String, usize> = BTreeMap::new();
        for row in &rows {
            let qs = scheduling::clauses(&row.prompt)?;
            let actual = scheduling::generate(
                &artifact,
                &geometry,
                &metric,
                &row.records,
                &row.prompt,
                scheduling::Control::Full,
            )?;
            let eos = actual.tokens.last() == Some(&256);
            let final_clause = actual.steps.last().map(|step| step.after.clause);
            let declared_complete = !actual.exhausted && eos;
            let expected = row.answer.as_ref().map(|answer| {
                answer
                    .iter()
                    .map(|&b| u16::from(b))
                    .chain([256])
                    .collect::<Vec<_>>()
            });
            let exact = expected.as_ref().is_some_and(|target| {
                !actual.exhausted && &actual.tokens == target && final_clause == Some(row.depth - 1)
            });
            let premature = row.blocked_at.is_some()
                && declared_complete
                && final_clause.is_some_and(|clause| clause < row.depth - 1);
            valid_exact += usize::from(exact);
            unresolved += usize::from(row.blocked_at.is_some());
            premature_completions += usize::from(premature);
            let n = counts
                .entry(format!("depth{}/{}", row.depth, row.variant))
                .or_default();
            n[0] += 1;
            n[1] += usize::from(exact);
            n[2] += usize::from(declared_complete);
            n[3] += usize::from(premature);
            let mut trace = Vec::new();
            for step in &actual.steps {
                if step.before.core.cursor != 0 && row.family != rows[0].family {
                    continue;
                }
                let next = qs.get(step.before.clause + 1);
                let pending = next.is_some();
                let mut updater_candidates = None;
                if let (Some(next), Some(selected)) =
                    (next, step.observation.route.selected.as_ref())
                {
                    let candidates = binding::updates(
                        &artifact.parent,
                        &geometry,
                        &metric,
                        &row.records,
                        next,
                        &selected.bytes,
                        binding::Control::Full,
                    )?;
                    updater_candidates = Some(
                        candidates
                            .iter()
                            .filter(|candidate| artifact.parent.matches(candidate.features))
                            .count(),
                    );
                }
                let next_route = if pending && step.observation.update.is_some() {
                    Some(reader::route(
                        &artifact.parent.parent,
                        &geometry,
                        &metric,
                        &row.records,
                        &step.observation.core.next_query.bytes,
                        reader::Control::Full,
                    )?)
                } else {
                    None
                };
                let reason = if !pending {
                    "NotRequested".into()
                } else if let Some(route) = &next_route {
                    serde_json::to_value(route.status)?
                        .as_str()
                        .ok_or("route status string")?
                        .to_string()
                } else if step.observation.route.selected.is_none() {
                    "CurrentRouteUnavailable".into()
                } else {
                    "NoUniqueUpdate".into()
                };
                if step.before.core.cursor == 0 {
                    *reason_counts.entry(reason.clone()).or_default() += 1;
                }
                trace.push(json!({"before_clause":step.before.clause,"after_clause":step.after.clause,
                    "cursor":step.before.core.cursor,"pending_clause":pending,"usable_continuation":step.observation.core.next_available,
                    "updater_accepted_candidates":updater_candidates,"next_route_reason":reason,"next_route":next_route,
                    "current_route":step.observation.route,"row":step.observation.core.row,"action":step.action,"token":step.token,
                    "committed_query":step.after.core.query.bytes,"done":step.after.core.done,"exhausted":step.after.core.exhausted}));
            }
            let mut response = json!({"id":row.id,"family":row.family,"depth":row.depth,"variant":row.variant,
                "expected_answer":row.answer,"expected_reason":row.expected_reason,"blocked_at":row.blocked_at,
                "resolved_prefix":row.resolved_prefix,"actual_tokens":actual.tokens,"actual_eos":eos,"actual_exhausted":actual.exhausted,
                "actual_final_clause":final_clause,"declared_complete":declared_complete,"valid_exact":exact,
                "premature_intermediate_completion":premature});
            if row.family == rows[0].family {
                response["trace"] = json!(trace);
                write(
                    &output,
                    &format!("sample-depth{}-{}.json", row.depth, row.variant),
                    &json!({"example":row,"response":response}),
                )?;
            }
            responses.push(response);
        }
        write(&output, "responses.json", &json!(responses))?;
        write(
            &output,
            "summary.json",
            &json!({"status":"DIAGNOSTIC_COMPLETE",
            "finding":if premature_completions > 0 {"INTERMEDIATE_COMPLETION_REPRODUCED"} else {"INTERMEDIATE_COMPLETION_NOT_OBSERVED"},
            "rows":rows.len(),"valid_rows":rows.len()-unresolved,"valid_exact":valid_exact,"unresolved_rows":unresolved,
            "premature_intermediate_completions":premature_completions,"counts_columns":["rows","valid_exact","declared_complete","premature_intermediate_completion"],"counts":counts,
            "next_route_reasons_at_boundaries":reason_counts,"candidate_sha256":sha(&bytes),"artifact_bytes_unchanged":artifact.encode()?==bytes,
            "fits":0,"model_correction":false,"promotion":false,"scope":"frozen authored completion diagnostic; unavailable/ambiguous routing does not prove global fact absence"}),
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
