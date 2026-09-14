//! Unchanged-parameter whole-phrase query-update qualification.
use super::{
    completion, completion_data, depth_data, phrase as runtime, phrase_data as data, schedule_data,
    span, span_data, span_report,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        language_relation::runtime as lexical,
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
fn write(root: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn path(out: &completion::Generated) -> Vec<[usize; 2]> {
    out.decisions
        .iter()
        .filter(|d| d.before.core.cursor == 0)
        .filter_map(|d| d.selected)
        .collect()
}
fn queries(out: &completion::Generated) -> Vec<Vec<u8>> {
    out.decisions
        .iter()
        .filter(|d| d.before.core.cursor == 0)
        .map(|d| d.before.core.query.bytes.clone())
        .collect()
}
fn selections(
    out: &completion::Generated,
    records: &[Vec<u8>; 4],
) -> (Vec<[usize; 3]>, Vec<[usize; 2]>, Vec<Vec<u8>>) {
    let mut spans = Vec::new();
    let mut bounds = Vec::new();
    let mut payloads = Vec::new();
    for step in out.trace.steps.iter().filter(|s| s.before.core.cursor == 0) {
        if let Some(v) = &step.observation.route.selected {
            let words = data::intervals(&records[v.source]);
            let first = words.iter().position(|b| b[0] == v.start);
            let last = words.iter().position(|b| b[1] == v.end);
            if let Some((a, b)) = first.zip(last) {
                spans.push([v.source, a, b + 1]);
            }
            bounds.push([v.start, v.end]);
            payloads.push(v.bytes.clone());
        }
    }
    (spans, bounds, payloads)
}
fn compact(out: &completion::Generated, records: &[Vec<u8>; 4]) -> Value {
    let (spans, bounds, payloads) = selections(out, records);
    json!({"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"outcome":out.outcome,"source_path":path(out),"selected_spans":spans,"selected_bounds":bounds,"selected_payloads":payloads,"actual_queries":queries(out)})
}
fn answer(out: &completion::Generated, e: &data::Example) -> bool {
    out.trace.tokens
        == e.answer
            .iter()
            .map(|&b| u16::from(b))
            .chain([256])
            .collect::<Vec<_>>()
        && !out.trace.exhausted
        && out.outcome == completion::Outcome::Answered
}
fn span_compact(out: &completion::Generated, records: &[Vec<u8>; 4]) -> Value {
    let selected = out
        .trace
        .steps
        .iter()
        .rev()
        .find_map(|s| s.observation.route.selected.as_ref());
    let bounds = selected.map(|v| [v.start, v.end]);
    let actual = selected.and_then(|v| {
        let words = data::intervals(&records[v.source]);
        words
            .iter()
            .position(|w| w[0] == v.start)
            .zip(words.iter().position(|w| w[1] == v.end))
            .map(|(a, b)| [v.source, a, b + 1])
    });
    json!({"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"outcome":out.outcome,"source_path":path(out),"actual_word_span":actual,"last_selected_bounds":bounds})
}
fn retain(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let earlier = span_report::retention(a, g, m, evidence)?;
    let root = evidence.join("language-span-1/attempt-3");
    report_output::verify(&root)?;
    let pd: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
    let cases: Vec<span_data::Example> = serde_json::from_value(pd["development"].clone())?;
    let by_id: BTreeMap<_, _> = cases.iter().map(|e| (e.id.as_str(), e)).collect();
    let previous: Vec<Value> =
        serde_json::from_slice(&std::fs::read(root.join("responses.json"))?)?;
    let mut identity = BTreeSet::new();
    let mut replay = Vec::new();
    let mut replay_equal = 0;
    for row in &previous {
        let id = row["id"].as_str().ok_or("span response id")?;
        let e = by_id.get(id).ok_or("span response case")?;
        let control: span::Control = serde_json::from_value(row["control"].clone())?;
        if !identity.insert((id, row["control"].to_string())) {
            return Err("duplicate retained span response".into());
        }
        let actual = span::generate(a, g, m, &e.records, &e.prompt, control);
        let equal = match actual {
            Ok(out) => {
                let now = span_compact(&out, &e.records);
                row.get("error").is_none()
                    && [
                        "tokens",
                        "exhausted",
                        "outcome",
                        "source_path",
                        "actual_word_span",
                        "last_selected_bounds",
                    ]
                    .iter()
                    .all(|field| now[field] == row[field])
            }
            Err(error) => row["error"] == error.to_string(),
        };
        replay_equal += usize::from(equal);
        replay.push(json!({"id":id,"control":control,"equal":equal}));
    }
    let mut transfer = Vec::new();
    let mut total_equal = 0;
    for (previous, required) in [
        ("language-span-1", 1728),
        ("language-scheduling-1", 1536),
        ("language-depth-1", 1280),
        ("language-completion-1", 896),
    ] {
        let root = evidence
            .join(previous)
            .join(if previous == "language-span-1" {
                "attempt-3"
            } else {
                "attempt-1"
            });
        report_output::verify(&root)?;
        let pd: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
        let rows: Vec<(String, [Vec<u8>; 4], Vec<u8>)> = match previous {
            "language-span-1" => {
                serde_json::from_value::<Vec<span_data::Example>>(pd["development"].clone())?
                    .into_iter()
                    .map(|e| (e.id, e.records, e.prompt))
                    .collect()
            }
            "language-depth-1" => {
                serde_json::from_value::<Vec<depth_data::Example>>(pd["development"].clone())?
                    .into_iter()
                    .map(|e| (e.id, e.records, e.prompt))
                    .collect()
            }
            "language-completion-1" => {
                serde_json::from_value::<Vec<completion_data::Example>>(pd["development"].clone())?
                    .into_iter()
                    .map(|e| (e.id, e.records, e.prompt))
                    .collect()
            }
            _ => serde_json::from_value::<Vec<schedule_data::Example>>(pd["development"].clone())?
                .into_iter()
                .map(|e| (e.id, e.records, e.prompt))
                .collect(),
        };
        let mut items = Vec::new();
        let mut equal = 0;
        let mut unresolved = 0;
        for (id, records, prompt) in rows {
            let old = span::generate(a, g, m, &records, &prompt, span::Control::Full)?;
            let new = runtime::generate(a, g, m, &records, &prompt, runtime::Control::Full);
            let (same, error) = match new {
                Ok(out) => {
                    unresolved += usize::from(
                        out == old && matches!(out.outcome, completion::Outcome::Unresolved { .. }),
                    );
                    (out == old, None)
                }
                Err(error) => (false, Some(error.to_string())),
            };
            equal += usize::from(same);
            items.push(json!({"id":id,"full_trace_and_decisions_equal":same,"error":error}));
        }
        total_equal += equal;
        transfer.push(json!({"previous":previous,"rows":items.len(),"required":required,"equal":equal,"unresolved":unresolved,"items":items}));
    }
    let full_equal = transfer
        .iter()
        .all(|p| p["rows"] == p["required"] && p["equal"] == p["required"])
        && transfer
            .iter()
            .find(|p| p["previous"] == "language-completion-1")
            .is_some_and(|p| p["unresolved"] == 512);
    Ok(
        json!({"earlier":earlier,"span_control_replay":{"rows":replay.len(),"equal_rows":replay_equal,"equal":replay.len()==19008&&replay_equal==19008,"items":replay},
        "new_full_transfer":{"rows":5440,"equal_rows":total_equal,"equal":full_equal,"panels":transfer},
        "scope":"old span controls and earlier adapters replay through the retained span artifact; new whole-phrase Full separately preserves all prior span and earlier Full traces/decisions"}),
    )
}
#[test]
#[ignore = "Execute whole-phrase update with frozen artifact; exclusive report"]
fn whole_phrase_query_update_report() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_PHRASE_UPDATE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_PHRASE_UPDATE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","examples":864,"families":72,"rows_per_family":12,
            "Full":{"answers":864,"paths":864,"spans":864,"bounds":864,"queries":864,"query_geometry":864},
            "controls":{"LegacyWordOnly":{"one_word_correct":288,"multiword_correct":0},"UpdateFirstWord":{"one_word_correct":288,"multiword_correct":0},
                "StalePayload":{"baseline_correct":288,"inactive_correct":288,"active_correct":0},"UpdateDisabled":0,"ReadDisabled":0,"ExactIdentity":864},
            "stale_source":"actual matched baseline Full selected bytes indexed by clause; never an oracle payload",
            "reload_rows":864,"fits":0,"artifact_parameters_unchanged":true,"old_span_control_rows":19008,
            "new_full_transfer":{"span":1728,"scheduling":1536,"depth":1280,"completion":896,"typed_unresolved":512},
            "scope":"bounded sequential phrase-reference and same-first-word alias branching; familiar authored vocabulary and grammar; not general prose",
            "final_holdout":"NOT_RUN","promotion":false}),
        )?;
        let rows = data::corpus()?;
        let validation = data::validate(&rows)?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"validation":validation,"training":"NOT_RUN"}),
        )?;
        let parent_root = evidence.join("language-span-1/attempt-3");
        report_output::verify(&parent_root)?;
        let bytes = std::fs::read(parent_root.join("candidate.json"))?;
        if sha(&bytes) != "44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b" {
            return Err("retained span artifact hash differs".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = span::Artifact::decode(&bytes, &g)?;
        let reload = span::Artifact::decode(&bytes, &g)?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        let mut baseline_payloads = BTreeMap::<String, Vec<Vec<u8>>>::new();
        let mut baseline_errors = Vec::new();
        // This baseline pass supplies actual stale intervention values. It is not
        // fitted, and no expected payload/path determines the intervention.
        for e in rows.iter().filter(|e| e.variant == "baseline") {
            match runtime::generate(&a, &g, &m, &e.records, &e.prompt, runtime::Control::Full) {
                Ok(out) => {
                    let (_, _, payloads) = selections(&out, &e.records);
                    if payloads.len() == e.depth {
                        baseline_payloads
                            .insert(e.id.trim_end_matches("baseline").to_string(), payloads);
                    } else {
                        baseline_errors.push(json!({"id":e.id,"error":"incomplete actual baseline payloads","response":compact(&out,&e.records)}));
                    }
                }
                Err(error) => baseline_errors.push(json!({"id":e.id,"error":error.to_string()})),
            }
        }
        write(
            &output,
            "stale-baselines.json",
            &json!({"payloads":baseline_payloads,"errors":baseline_errors,"source":"actual Full trajectories"}),
        )?;
        let mut gate = baseline_payloads.len() == 288;
        let mut panels = Vec::new();
        let mut responses = Vec::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut samples = BTreeSet::new();
        let mut reloaded = 0;
        for control in [
            runtime::Control::Full,
            runtime::Control::LegacyWordOnly,
            runtime::Control::UpdateFirstWord,
            runtime::Control::StalePayload,
            runtime::Control::UpdateDisabled,
            runtime::Control::ReadDisabled,
            runtime::Control::ExactIdentity,
        ] {
            let (
                mut correct,
                mut paths,
                mut spans,
                mut bounds,
                mut query_equal,
                mut geometry_equal,
                mut one,
                mut multi,
                mut errors,
            ) = (0, 0, 0, 0, 0, 0, 0, 0, 0);
            let mut variants = BTreeMap::<String, usize>::new();
            for e in &rows {
                let result = if control == runtime::Control::StalePayload {
                    let suffix = format!("{}", e.variant);
                    let key = e.id.strip_suffix(&suffix).ok_or("variant id suffix")?;
                    match baseline_payloads.get(key) {
                        Some(p) => {
                            runtime::generate_with_stale(&a, &g, &m, &e.records, &e.prompt, p)
                        }
                        None => Err(
                            crate::native_geometric::relational_attention::runtime::Error::State,
                        ),
                    }
                } else {
                    runtime::generate(&a, &g, &m, &e.records, &e.prompt, control)
                };
                let out = match result {
                    Ok(out) => out,
                    Err(error) => {
                        errors += 1;
                        responses.push(json!({"id":e.id,"family":e.family,"placement":e.placement,"length":e.length,"variant":e.variant,"control":control,"correct":false,"error":error.to_string()}));
                        continue;
                    }
                };
                let yes = answer(&out, e);
                let path_ok = path(&out) == e.expected_path;
                let (actual_spans, actual_bounds, payloads) = selections(&out, &e.records);
                let span_ok = actual_spans == e.expected_spans;
                let bounds_ok = actual_bounds == e.expected_bounds;
                let query_ok = queries(&out) == e.expected_queries;
                let actual_queries: Vec<_> = out
                    .decisions
                    .iter()
                    .filter(|d| d.before.core.cursor == 0)
                    .map(|d| &d.before.core.query)
                    .collect();
                let expected_queries: Vec<_> = e
                    .expected_queries
                    .iter()
                    .map(|q| lexical::Query::new(&g, q, a.reader().parent.parent.operators))
                    .collect::<Result<_, _>>()?;
                let geometry_ok = actual_queries == expected_queries.iter().collect::<Vec<_>>();
                correct += usize::from(yes);
                paths += usize::from(path_ok);
                spans += usize::from(span_ok);
                bounds += usize::from(bounds_ok);
                query_equal += usize::from(query_ok);
                geometry_equal += usize::from(geometry_ok);
                one += usize::from(yes && e.length == 1);
                multi += usize::from(yes && e.length > 1);
                *variants.entry(e.variant.clone()).or_default() += usize::from(yes);
                if control == runtime::Control::Full {
                    let second =
                        runtime::generate(&reload, &g, &m, &e.records, &e.prompt, control)?;
                    reloaded += usize::from(out == second);
                    if out
                        .trace
                        .steps
                        .windows(2)
                        .any(|p| p[0].after != p[1].before)
                    {
                        return Err("query-update state continuity differs".into());
                    }
                    if yes
                        && path_ok
                        && span_ok
                        && bounds_ok
                        && query_ok
                        && geometry_ok
                        && payloads == e.expected_payloads
                    {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    let role = e.family.contains("-v1-");
                    if e.variant == "baseline"
                        && samples.insert((e.placement.clone(), e.length, role))
                    {
                        write(
                            &output,
                            &format!(
                                "sample-{}-words{}-subject{role}.json",
                                e.placement, e.length
                            ),
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                }
                let mut record = compact(&out, &e.records);
                let obj = record.as_object_mut().ok_or("response object")?;
                for (k, v) in [
                    ("id", json!(e.id)),
                    ("family", json!(e.family)),
                    ("placement", json!(e.placement)),
                    ("length", json!(e.length)),
                    ("variant", json!(e.variant)),
                    ("control", json!(control)),
                    ("correct", json!(yes)),
                    ("path_correct", json!(path_ok)),
                    ("spans_correct", json!(span_ok)),
                    ("bounds_correct", json!(bounds_ok)),
                    ("queries_correct", json!(query_ok)),
                    ("query_geometry_correct", json!(geometry_ok)),
                ] {
                    obj.insert(k.into(), v);
                }
                responses.push(record);
            }
            panels.push(json!({"control":control,"rows":rows.len(),"correct":correct,"paths":paths,"spans":spans,"bounds":bounds,"queries":query_equal,"query_geometry":geometry_equal,"one_word_correct":one,"multiword_correct":multi,"variants":variants,"errors":errors}));
            gate &= match control {
                runtime::Control::Full => {
                    correct == 864
                        && paths == 864
                        && spans == 864
                        && bounds == 864
                        && query_equal == 864
                        && geometry_equal == 864
                }
                runtime::Control::LegacyWordOnly | runtime::Control::UpdateFirstWord => {
                    one == 288 && multi == 0
                }
                runtime::Control::StalePayload => {
                    variants.get("baseline") == Some(&288)
                        && variants.get("inactive") == Some(&288)
                        && variants.get("active").copied().unwrap_or(0) == 0
                }
                runtime::Control::UpdateDisabled | runtime::Control::ReadDisabled => correct == 0,
                runtime::Control::ExactIdentity => correct == 864,
            };
        }
        let complete = families.values().filter(|&&n| n == 12).count();
        gate &= families.len() == 72 && complete == 72 && reloaded == 864;
        write(&output, "responses.json", &json!(responses))?;
        let retained = retain(&a, &g, &m, &evidence)?;
        gate &= retained["earlier"]["legacy_control_replay"]["equal"] == true
            && retained["earlier"]["new_full_transfer"]["equal"] == true
            && retained["span_control_replay"]["equal"] == true
            && retained["new_full_transfer"]["equal"] == true;
        write(&output, "retained-responses.json", &retained)?;
        let unchanged = a.encode()? == bytes;
        gate &= unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if gate{"PASS_WHOLE_PHRASE_QUERY_UPDATE"}else{"FAIL_WHOLE_PHRASE_QUERY_UPDATE"},"development_rows":rows.len(),"panels":panels,"complete_families":complete,"reload_equal":reloaded,
            "fits":0,"candidate_sha256":sha(&bytes),"parent_sha256":sha(&bytes),"artifact_parameters_unchanged":unchanged,"source_sha256":sha(concat!(include_str!("phrase.rs"),include_str!("phrase_data.rs"),include_str!("phrase_report.rs"),include_str!("runtime.rs"),include_str!("completion.rs"),include_str!("scheduling.rs")).as_bytes()),
            "legacy_control_replay_equal":retained["earlier"]["legacy_control_replay"]["equal"],"legacy_control_replay_counts":retained["earlier"]["legacy_control_replay"]["counts"],"prior_span_control_replay_equal":retained["span_control_replay"]["equal"],"prior_span_control_replay_rows":retained["span_control_replay"]["equal_rows"],
            "new_full_transfer_equal":retained["new_full_transfer"]["equal"],"new_full_transfer_rows":retained["new_full_transfer"]["equal_rows"],"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
