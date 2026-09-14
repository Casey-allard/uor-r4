//! Candidate-specific occurrence correspondence qualification against frozen
//! negative cases and retained outputs after one bounded rule-extension fit.
use super::{
    completion, occurrence as runtime, occurrence_data as data, occurrence_learning, phrase,
    phrase_data::intervals, runtime::PayloadWindow, scheduling, span, span_data,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        language_relation::runtime as lexical,
    },
    report_output,
};
use serde::Serialize;
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
            let words = intervals(&records[v.source]);
            if let Some((first, last)) = words
                .iter()
                .position(|w| w[0] == v.start)
                .zip(words.iter().position(|w| w[1] == v.end))
            {
                spans.push([v.source, first, last + 1]);
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
fn correct(out: &completion::Generated, e: &data::Example) -> bool {
    out.trace.tokens
        == e.answer
            .iter()
            .map(|b| u16::from(*b))
            .chain([256])
            .collect::<Vec<_>>()
        && !out.trace.exhausted
        && out.outcome == completion::Outcome::Answered
}
#[derive(Default, Serialize)]
struct Counts {
    rows: usize,
    correct: usize,
    paths: usize,
    spans: usize,
    bounds: usize,
    queries: usize,
    payloads: usize,
    query_geometry: usize,
    one_read_correct: usize,
    two_read_correct: usize,
    errors: usize,
}
impl Counts {
    fn add(
        &mut self,
        out: Option<&completion::Generated>,
        e: &data::Example,
        g: &BoundGeometry,
        a: &span::Artifact,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.rows += 1;
        let Some(out) = out else {
            self.errors += 1;
            return Ok(false);
        };
        let (spans, bounds, payloads) = selections(out, &e.records);
        let yes = correct(out, e);
        let path_ok = path(out) == e.expected_path;
        let span_ok = spans == e.expected_spans;
        let bounds_ok = bounds == e.expected_bounds;
        let query_ok = queries(out) == e.expected_queries;
        let payload_ok = payloads == e.expected_payloads;
        let actual: Vec<_> = out
            .decisions
            .iter()
            .filter(|d| d.before.core.cursor == 0)
            .map(|d| &d.before.core.query)
            .collect();
        let expected: Vec<_> = e
            .expected_queries
            .iter()
            .map(|q| lexical::Query::new(g, q, a.reader().parent.parent.operators))
            .collect::<Result<_, _>>()?;
        let geometry_ok = actual == expected.iter().collect::<Vec<_>>();
        self.correct += usize::from(yes);
        self.paths += usize::from(path_ok);
        self.spans += usize::from(span_ok);
        self.bounds += usize::from(bounds_ok);
        self.queries += usize::from(query_ok);
        self.payloads += usize::from(payload_ok);
        self.query_geometry += usize::from(geometry_ok);
        self.one_read_correct += usize::from(yes && e.depth == 1);
        self.two_read_correct += usize::from(yes && e.depth == 2);
        Ok(yes && path_ok && span_ok && bounds_ok && query_ok && payload_ok && geometry_ok)
    }
}
fn input(row: &Value) -> Result<([Vec<u8>; 4], Vec<u8>), Box<dyn std::error::Error>> {
    Ok((
        serde_json::from_value(row["records"].clone())?,
        serde_json::from_value(row["prompt"].clone())?,
    ))
}
fn compare_compact(old: &Value, now: &Value) -> bool {
    [
        "tokens",
        "exhausted",
        "outcome",
        "source_path",
        "selected_spans",
        "selected_bounds",
        "selected_payloads",
        "actual_queries",
    ]
    .iter()
    .all(|k| old[k] == now[k])
}
fn retention(
    parent_artifact: &span::Artifact,
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut panels = Vec::new();
    let mut total = 0;
    let mut total_equal = 0;
    let mut unresolved = 0;
    for (name, attempt, required) in [
        ("language-phrase-update-1", "attempt-1", 864),
        ("language-span-1", "attempt-3", 1728),
        ("language-scheduling-1", "attempt-1", 1536),
        ("language-depth-1", "attempt-1", 1280),
        ("language-completion-1", "attempt-1", 896),
    ] {
        let root = evidence.join(name).join(attempt);
        report_output::verify(&root)?;
        let frozen: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
        let rows = frozen["development"]
            .as_array()
            .ok_or("retained development rows")?;
        let mut items = Vec::new();
        let mut equal = 0;
        for row in rows {
            let (records, prompt) = input(row)?;
            let old = phrase::generate(
                parent_artifact,
                g,
                m,
                &records,
                &prompt,
                phrase::Control::Full,
            )?;
            let new = runtime::generate(a, g, m, &records, &prompt, runtime::Control::Full);
            let (same, response) = match new {
                Ok(out) => {
                    let same = old == out;
                    if same && matches!(out.outcome, completion::Outcome::Unresolved { .. }) {
                        unresolved += 1;
                    }
                    (same, compact(&out, &records))
                }
                Err(error) => (false, json!({"error":error.to_string()})),
            };
            equal += usize::from(same);
            items.push(json!({"id":row["id"],"full_trace_and_decisions_equal":same,"actual":if same {Value::Null} else {response},"previous":if same {Value::Null}else{compact(&old,&records)}}));
        }
        total += rows.len();
        total_equal += equal;
        panels.push(json!({"source":name,"required":required,"rows":rows.len(),"equal":equal,"items":items}));
    }
    let root = evidence.join("language-phrase-update-1/attempt-1");
    let frozen: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
    let rows = frozen["development"].as_array().ok_or("phrase rows")?;
    let by_id: BTreeMap<_, _> = rows
        .iter()
        .map(|e| Ok((e["id"].as_str().ok_or("phrase id")?, e)))
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let responses: Vec<Value> =
        serde_json::from_slice(&std::fs::read(root.join("responses.json"))?)?;
    let stale: Value = serde_json::from_slice(&std::fs::read(root.join("stale-baselines.json"))?)?;
    let mut replay = Vec::new();
    let mut replay_equal = 0;
    let mut seen = BTreeSet::new();
    for response in &responses {
        let id = response["id"].as_str().ok_or("phrase response id")?;
        let e = by_id.get(id).ok_or("missing phrase input")?;
        let control: phrase::Control = serde_json::from_value(response["control"].clone())?;
        if !seen.insert((id, response["control"].to_string())) {
            return Err("duplicate phrase control".into());
        }
        let (records, prompt) = input(e)?;
        let result = if control == phrase::Control::StalePayload {
            let variant = e["variant"].as_str().ok_or("phrase variant")?;
            let key = id.strip_suffix(variant).ok_or("stale baseline identity")?;
            let payloads: Vec<Vec<u8>> = serde_json::from_value(stale["payloads"][key].clone())?;
            phrase::generate_with_stale(parent_artifact, g, m, &records, &prompt, &payloads)
        } else {
            phrase::generate(parent_artifact, g, m, &records, &prompt, control)
        };
        let equal = match result {
            Ok(out) => {
                response.get("error").is_none()
                    && compare_compact(response, &compact(&out, &records))
            }
            Err(error) => response["error"] == error.to_string(),
        };
        replay_equal += usize::from(equal);
        replay.push(json!({"id":id,"control":control,"equal":equal}));
    }
    Ok(
        json!({"new_full_transfer":{"rows":total,"equal_rows":total_equal,"typed_unresolved":unresolved,"equal":total==6304&&total_equal==6304&&unresolved==512,"panels":panels},"prior_phrase_control_replay":{"rows":responses.len(),"equal_rows":replay_equal,"equal":responses.len()==6048&&replay_equal==6048,"items":replay},"older_legacy_adapter_replay":"NOT_RUN; retained source and sealed evidence verified separately"}),
    )
}
fn witness_trace(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    out: &completion::Generated,
) -> Result<(Value, usize), Box<dyn std::error::Error>> {
    let mut stages = Vec::new();
    let mut max_nodes = 0;
    for query in queries(out) {
        let search = runtime::candidates(a, g, m, records, &query, runtime::Control::Full)?;
        max_nodes = max_nodes.max(search.nodes);
        let candidates:Vec<_>=search.candidates.iter().map(|c|json!({"source":c.span.value.source,"first_word":c.span.value.word,"last_word_exclusive":c.span.last_word,"bounds":[c.span.value.start,c.span.value.end],"bytes":c.span.value.bytes,"admitted":c.witnesses.iter().any(|w|a.matches(w.features)),"witnesses":c.witnesses.iter().map(|w|json!({"query_to_source":w.query_to_source,"features":w.features,"barriers":w.barriers,"admitted":a.matches(w.features)})).collect::<Vec<_>>()})).collect();
        stages.push(
            json!({"actual_query":query,"search_nodes":search.nodes,"candidates":candidates}),
        );
    }
    Ok((json!(stages), max_nodes))
}
fn training(
    evidence: &Path,
    rows: &[data::Example],
) -> Result<Vec<span_data::Example>, Box<dyn std::error::Error>> {
    let root = evidence.join("language-span-1/attempt-3");
    report_output::verify(&root)?;
    let prior: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
    let mut train: Vec<span_data::Example> = serde_json::from_value(prior["training"].clone())?;
    if train.len() != 2624 {
        return Err("old direct training count".into());
    }
    // A bijection of non-grammar word identities preserves the independently
    // checked relation truth and repetition; no model selects training examples.
    let names = [
        "alice", "ruby", "amber", "helen", "birch", "oscar", "cedar", "clara", "dylan", "felix",
        "bruno",
    ];
    let rename = |raw: &[u8]| -> Vec<u8> {
        let mut out = Vec::new();
        let mut last = 0;
        for [start, end] in intervals(raw) {
            out.extend_from_slice(&raw[last..start]);
            if let Some(i) = names.iter().position(|w| w.as_bytes() == &raw[start..end]) {
                out.extend_from_slice(names[(i + 1) % names.len()].as_bytes());
            } else {
                out.extend_from_slice(&raw[start..end]);
            }
            last = end;
        }
        out.extend_from_slice(&raw[last..]);
        out
    };
    for e in rows.iter().filter(|e| e.depth == 1) {
        train.push(span_data::Example {
            id: format!("renamed-{}", e.id),
            family: format!("renamed-{}", e.family),
            depth: 1,
            variant: e.variant.clone(),
            records: std::array::from_fn(|i| rename(&e.records[i])),
            prompt: rename(&e.prompt),
            answer: rename(&e.answer),
            expected_path: vec![],
            expected_span: [0; 3],
            expected_bounds: [0; 2],
        });
    }
    if train.len() != 2816 {
        return Err("training balance".into());
    }
    for e in &mut train {
        e.expected_path.clear();
        e.expected_span = [0; 3];
        e.expected_bounds = [0; 2];
        if scheduling::clauses(&e.prompt)?.len() != 1
            || rows
                .iter()
                .any(|d| d.records == e.records && d.prompt == e.prompt)
        {
            return Err("training/development input overlap or non-direct training".into());
        }
    }
    Ok(train)
}
#[test]
#[ignore = "bounded actual-artifact occurrence qualification"]
fn occurrence_correspondence_qualification() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty occurrence qualification path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","examples":384,"complete_families":32,"rows_per_family":12,"Full":{"answers":384,"paths":384,"spans":384,"bounds":384,"queries":384,"payloads":384,"query_geometry":384,"reload_equal":384},"controls":{"ParentUnionMatches":"all384 compact responses exactly equal unchanged diagnostic","UnionMatches":"same refined artifact with union matches; report overlap failures","ExactIdentity":"all384 full generated traces equal Full","ReadDisabled":0,"UpdateDisabled":{"one_read_correct":192,"two_read_correct":0},"NonInjective":"no positive valid-panel requirement; must fail multiplicity guard"},"multiplicity_guard":{"Full":"Unresolved clause1 NoCompatibleCandidate","ExactIdentity":"equal Full","NonInjective":"must produce an incorrect Answered outcome rather than typed unresolved"},"retention":{"new_full_trace_rows":6304,"typed_unresolved":512,"prior_phrase_control_rows":6048},"fits":1,"parent_parameters_unchanged":true,"training_rows":2816,"proposal_family":"all supersets of parent rule adding0..2 existing bits, at most8rules","all_control_errors":0,"final_holdout":"NOT_RUN","promotion":false,"scope":"candidate-specific occurrence correspondence in authored one/two-read familiar relation grammar; no general prose claim"}),
        )?;
        let diagnostic = evidence.join("language-occurrence-1/diagnostic-1");
        report_output::verify(&diagnostic)?;
        let frozen: Value = serde_json::from_slice(&std::fs::read(diagnostic.join("data.json"))?)?;
        let rows: Vec<data::Example> = serde_json::from_value(frozen["development"].clone())?;
        if rows != data::corpus()? {
            return Err("diagnostic corpus changed".into());
        }
        let validation = data::validate(&rows)?;
        let guard = data::multiplicity_guard()?;
        let old_guard: Value =
            serde_json::from_slice(&std::fs::read(diagnostic.join("multiplicity-guard.json"))?)?;
        if guard != old_guard {
            return Err("multiplicity guard changed".into());
        }
        write(
            &output,
            "data.json",
            &json!({"development":rows,"validation":validation,"training":"separate training-data.json, direct renamed plus retained rows","diagnostic_data_sha256":sha(&std::fs::read(diagnostic.join("data.json"))?)}),
        )?;
        write(&output, "multiplicity-guard.json", &guard)?;
        let old: Vec<Value> =
            serde_json::from_slice(&std::fs::read(diagnostic.join("responses.json"))?)?;
        let old_full: BTreeMap<_, _> = old
            .iter()
            .filter(|r| r["control"] == "Full")
            .map(|r| Ok((r["id"].as_str().ok_or("diagnostic id")?, r)))
            .collect::<Result<_, Box<dyn std::error::Error>>>()?;
        if old_full.len() != 384 {
            return Err("diagnostic Full row count".into());
        }
        let train = training(&evidence, &rows)?;
        write(
            &output,
            "training-data.json",
            &json!({"training":train,"retained_rows":2624,"new_direct_rows":192,"renaming":"cyclic bijection of alice,ruby,amber,helen,birch,oscar,cedar,clara,dylan,felix,bruno","development_input_overlap":false,"evaluation_metadata":"removed before fit","fits":1}),
        )?;
        let parent = evidence.join("language-span-1/attempt-3");
        report_output::verify(&parent)?;
        let bytes = std::fs::read(parent.join("candidate.json"))?;
        if sha(&bytes) != "44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b" {
            return Err("retained artifact hash differs".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent_artifact = span::Artifact::decode(&bytes, &g)?;
        let (a, fit) = occurrence_learning::fit(&parent_artifact, &g, &m, &train)?;
        write(&output, "fit.json", &serde_json::to_value(&fit)?)?;
        let candidate_bytes = a.encode()?;
        let reload = span::Artifact::decode(&candidate_bytes, &g)?;
        std::fs::write(output.join("candidate.json"), &candidate_bytes)?;
        let previous_attempt = evidence.join("language-occurrence-1/attempt-1");
        report_output::verify(&previous_attempt)?;
        let previous_responses: Vec<Value> =
            serde_json::from_slice(&std::fs::read(previous_attempt.join("responses.json"))?)?;
        let previous_success: BTreeSet<String> = previous_responses
            .iter()
            .filter(|r| r["control"] == "Full" && r["correct"] == true)
            .map(|r| r["id"].as_str().ok_or("previous id").map(str::to_string))
            .collect::<Result<_, _>>()?;
        if previous_success.len() != 296 {
            return Err("first candidate success count".into());
        }
        let mut prior_success_equal = 0;
        let mut panels = BTreeMap::<String, Counts>::new();
        let mut groups = BTreeMap::<String, Counts>::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut responses = Vec::new();
        let mut search_nodes = Vec::new();
        let mut reloaded = 0;
        let mut exact_equal = 0;
        let mut union_equal = 0;
        let mut max_nodes = 0;
        for e in &rows {
            let full = runtime::generate(&a, &g, &m, &e.records, &e.prompt, runtime::Control::Full);
            let exact = runtime::generate(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                runtime::Control::ExactIdentity,
            );
            exact_equal += usize::from(match (&full, &exact) {
                (Ok(x), Ok(y)) => x == y,
                _ => false,
            });
            if let Ok(out) = &full {
                reloaded += usize::from(
                    runtime::generate(
                        &reload,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        runtime::Control::Full,
                    )
                    .as_ref()
                    .is_ok_and(|x| x == out),
                );
                if out
                    .trace
                    .steps
                    .windows(2)
                    .any(|p| p[0].after != p[1].before)
                {
                    return Err("occurrence state continuity differs".into());
                }
                let (witnesses, nodes) = witness_trace(&a, &g, &m, &e.records, out)?;
                max_nodes = max_nodes.max(nodes);
                search_nodes.push(json!({"id":e.id,"stages":witnesses.as_array().ok_or("witness stages")?.iter().map(|s|json!({"actual_query":s["actual_query"],"search_nodes":s["search_nodes"]})).collect::<Vec<_>>()}));
                if e.rotation == 0 && e.variant == "baseline" {
                    write(
                        &output,
                        &format!(
                            "sample-{}-subject{}-depth{}.json",
                            e.pattern,
                            usize::from(e.subject_answer),
                            e.depth
                        ),
                        &json!({"example":e,"generated":out,"occurrence_witnesses":witnesses}),
                    )?;
                }
            }
            let union = runtime::generate(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                runtime::Control::UnionMatches,
            );
            let previous = old_full
                .get(e.id.as_str())
                .ok_or("diagnostic input missing")?;
            let parent_union = runtime::generate(
                &parent_artifact,
                &g,
                &m,
                &e.records,
                &e.prompt,
                runtime::Control::UnionMatches,
            );
            if previous_success.contains(&e.id) {
                let prior = runtime::generate(
                    &parent_artifact,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    runtime::Control::Full,
                )?;
                prior_success_equal += usize::from(full.as_ref().is_ok_and(|out| *out == prior));
            }
            let same = match &parent_union {
                Ok(out) => {
                    previous.get("error").is_none()
                        && compare_compact(previous, &compact(out, &e.records))
                        && previous["read_observations"]
                            == json!(out
                                .trace
                                .steps
                                .iter()
                                .filter(|s| s.before.core.cursor == 0)
                                .map(|s| &s.observation.route)
                                .collect::<Vec<_>>())
                }
                Err(error) => previous["error"] == error.to_string(),
            };
            union_equal += usize::from(same);
            for (label, result) in [
                ("Full", full),
                ("ExactIdentity", exact),
                ("UnionMatches", union),
                ("ParentUnionMatches", parent_union),
                (
                    "NonInjective",
                    runtime::generate(
                        &a,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        runtime::Control::NonInjective,
                    ),
                ),
                (
                    "ReadDisabled",
                    runtime::generate(
                        &a,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        runtime::Control::ReadDisabled,
                    ),
                ),
                (
                    "UpdateDisabled",
                    runtime::generate(
                        &a,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        runtime::Control::UpdateDisabled,
                    ),
                ),
            ] {
                let complete =
                    panels
                        .entry(label.into())
                        .or_default()
                        .add(result.as_ref().ok(), e, &g, &a)?;
                if label == "Full" && complete {
                    *families.entry(e.family.clone()).or_default() += 1;
                }
                for group in [
                    format!("pattern/{}", e.pattern),
                    format!("role/{}", usize::from(e.subject_answer)),
                    format!("depth/{}", e.depth),
                    format!("variant/{}", e.variant),
                ] {
                    groups.entry(format!("{label}/{group}")).or_default().add(
                        result.as_ref().ok(),
                        e,
                        &g,
                        &a,
                    )?;
                }
                let mut response = match &result {
                    Ok(out) => compact(out, &e.records),
                    Err(error) => json!({"error":error.to_string()}),
                };
                let object = response.as_object_mut().ok_or("response object")?;
                for (k, v) in [
                    ("id", json!(e.id)),
                    ("control", json!(label)),
                    ("pattern", json!(e.pattern)),
                    ("variant", json!(e.variant)),
                    (
                        "correct",
                        json!(result.as_ref().is_ok_and(|out| correct(out, e))),
                    ),
                ] {
                    object.insert(k.into(), v);
                }
                responses.push(response);
            }
        }
        write(&output, "responses.json", &json!(responses))?;
        write(&output, "search-nodes.json", &json!(search_nodes))?;
        let (guard_records, guard_prompt) = input(&guard)?;
        let expected_first: Vec<u8> =
            serde_json::from_value(guard["expected_first_payload"].clone())?;
        let expected_second: Vec<u8> =
            serde_json::from_value(guard["expected_second_query"].clone())?;
        let mut guard_rows = Vec::new();
        let mut guard_full = None;
        let mut guard_exact_equal = false;
        let mut guard_full_correct = false;
        let mut guard_noninjective_correct = false;
        let mut guard_noninjective_answered = false;
        for (label, control) in [
            ("Full", runtime::Control::Full),
            ("ExactIdentity", runtime::Control::ExactIdentity),
            ("UnionMatches", runtime::Control::UnionMatches),
            ("NonInjective", runtime::Control::NonInjective),
        ] {
            let response = match runtime::generate(
                &a,
                &g,
                &m,
                &guard_records,
                &guard_prompt,
                control,
            ) {
                Ok(out) => {
                    let first = out.decisions.first().ok_or("guard decision missing")?;
                    let qs = scheduling::clauses(&guard_prompt)?;
                    let observed = completion::observe_routed(
                        &a.parent,
                        &g,
                        &m,
                        &guard_records,
                        &qs,
                        &first.before,
                        completion::Control::Full,
                        PayloadWindow::Phrase,
                        &|bytes: &[u8], _| Ok(bytes.to_vec()),
                        &mut |query: &[u8], _| {
                            runtime::route(&a, &g, &m, &guard_records, query, control)
                        },
                    )?;
                    let actual_payload = observed
                        .core
                        .route
                        .selected
                        .as_ref()
                        .map(|v| v.bytes.clone());
                    let actual_next_query = observed.core.core.next_query.bytes.clone();
                    let prefix_equal = actual_payload.as_ref() == Some(&expected_first)
                        && actual_next_query == expected_second;
                    let yes = prefix_equal
                        && matches!(
                            out.outcome,
                            completion::Outcome::Unresolved {
                                clause: 1,
                                reason: completion::Reason::NoCompatibleCandidate
                            }
                        )
                        && !out.trace.exhausted
                        && out.trace.tokens.is_empty();
                    if label == "Full" {
                        guard_full_correct = yes;
                        guard_full = Some(out.clone());
                    }
                    if label == "ExactIdentity" {
                        guard_exact_equal = guard_full.as_ref() == Some(&out);
                    }
                    if label == "NonInjective" {
                        guard_noninjective_correct = yes;
                        guard_noninjective_answered =
                            prefix_equal && out.outcome == completion::Outcome::Answered;
                    }
                    let (witnesses, nodes) = witness_trace(&a, &g, &m, &guard_records, &out)?;
                    max_nodes = max_nodes.max(nodes);
                    json!({"control":label,"correct_typed_unresolved":yes,"selected_payload_and_lookahead_query_correct":prefix_equal,"actual_selected_payload":actual_payload,"actual_lookahead_query":actual_next_query,"read_committed":!out.trace.steps.is_empty(),"generated":out,"Full_correspondence_diagnostic_on_actual_queries":witnesses})
                }
                Err(error) => {
                    json!({"control":label,"correct_typed_unresolved":false,"error":error.to_string()})
                }
            };
            guard_rows.push(response);
        }
        write(
            &output,
            "multiplicity-guard-responses.json",
            &json!(guard_rows),
        )?;
        let retained = retention(&parent_artifact, &a, &g, &m, &evidence)?;
        write(&output, "retained-responses.json", &retained)?;
        let complete = families.values().filter(|n| **n == 12).count();
        let unchanged = a.parent == parent_artifact.parent
            && a.parent_digest == parent_artifact.parent_digest
            && a.context_words == parent_artifact.context_words
            && a.feature_names == parent_artifact.feature_names;
        let full = panels.get("Full").ok_or("Full panel")?;
        let disabled = panels.get("ReadDisabled").ok_or("disabled panel")?;
        let update = panels.get("UpdateDisabled").ok_or("update panel")?;
        let passed = full.correct == 384
            && complete == 32
            && reloaded == 384
            && exact_equal == 384
            && union_equal == 384
            && disabled.correct == 0
            && update.one_read_correct == 192
            && update.two_read_correct == 0
            && guard_full_correct
            && guard_exact_equal
            && !guard_noninjective_correct
            && guard_noninjective_answered
            && retained["new_full_transfer"]["equal"] == true
            && retained["prior_phrase_control_replay"]["equal"] == true
            && unchanged
            && max_nodes <= runtime::MAX_SEARCH_NODES
            && prior_success_equal == 296
            && fit.training_exact == 2816
            && panels.values().all(|p| p.errors == 0);
        write(
            &output,
            "summary.json",
            &json!({"gate":if passed{"PASS_OCCURRENCE_CORRESPONDENCE"}else{"FAIL_OCCURRENCE_CORRESPONDENCE"},"development_rows":384,"panels":panels,"groups":groups,"complete_families":complete,"reload_equal":reloaded,"full_exact_identity_equal":exact_equal,"union_matches_diagnostic_equal":union_equal,"multiplicity_guard":{"Full_correct":guard_full_correct,"ExactIdentity_equal":guard_exact_equal,"NonInjective_correct":guard_noninjective_correct,"NonInjective_answered":guard_noninjective_answered},"max_observed_search_nodes":max_nodes,"route_search_limit":runtime::MAX_SEARCH_NODES,"new_full_transfer_equal":retained["new_full_transfer"]["equal"],"new_full_transfer_rows":retained["new_full_transfer"]["equal_rows"],"typed_unresolved_retained":retained["new_full_transfer"]["typed_unresolved"],"prior_phrase_control_replay_equal":retained["prior_phrase_control_replay"]["equal"],"prior_phrase_control_replay_rows":retained["prior_phrase_control_replay"]["equal_rows"],"fits":1,"fit":fit,"rules_before":parent_artifact.rules,"rules_after":a.rules,"previous_success_full_trace_equal":prior_success_equal,"candidate_sha256":sha(&candidate_bytes),"parent_sha256":sha(&bytes),"parent_parameters_unchanged":unchanged,"source_sha256":sha(concat!(include_str!("occurrence.rs"),include_str!("occurrence_learning.rs"),include_str!("occurrence_data.rs"),include_str!("occurrence_qualification.rs"),include_str!("phrase.rs"),include_str!("span.rs")).as_bytes()),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
