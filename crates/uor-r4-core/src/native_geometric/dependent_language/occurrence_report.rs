//! Unchanged-artifact diagnostic; a reproduced negative is sealed evidence,
//! not a qualification pass or an instruction to fit around lost occurrences.
use super::{
    completion, occurrence_data as data, phrase, phrase_data::intervals, span, span_boundary,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        ordered_state::runtime as ordered, relative_language::runtime as reader,
    },
    report_output,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};

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
            if let Some((a, b)) = words
                .iter()
                .position(|w| w[0] == v.start)
                .zip(words.iter().position(|w| w[1] == v.end))
            {
                spans.push([v.source, a, b + 1]);
            }
            bounds.push([v.start, v.end]);
            payloads.push(v.bytes.clone());
        }
    }
    (spans, bounds, payloads)
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
fn compact(out: &completion::Generated, e: &data::Example) -> Value {
    let (spans, bounds, payloads) = selections(out, &e.records);
    json!({"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"outcome":out.outcome,"source_path":path(out),"selected_spans":spans,"selected_bounds":bounds,"selected_payloads":payloads,"actual_queries":queries(out),"read_observations":out.trace.steps.iter().filter(|s| s.before.core.cursor==0).map(|s| &s.observation.route).collect::<Vec<_>>()})
}
#[derive(Default, Serialize)]
struct Counts {
    rows: usize,
    correct: usize,
    paths: usize,
    spans: usize,
    bounds: usize,
    queries: usize,
    errors: usize,
}
impl Counts {
    fn add(&mut self, out: Option<&completion::Generated>, e: &data::Example) {
        self.rows += 1;
        if let Some(out) = out {
            let (s, b, _) = selections(out, &e.records);
            self.correct += usize::from(correct(out, e));
            self.paths += usize::from(path(out) == e.expected_path);
            self.spans += usize::from(s == e.expected_spans);
            self.bounds += usize::from(b == e.expected_bounds);
            self.queries += usize::from(queries(out) == e.expected_queries);
        } else {
            self.errors += 1;
        }
    }
}
/// This only observes the actual generated query. Oracle spans annotate the
/// resulting candidates after enumeration; they never alter or admit a candidate.
fn diagnose(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &data::Example,
    out: &completion::Generated,
) -> Result<(Value, usize), Box<dyn std::error::Error>> {
    let mut stages = Vec::new();
    let mut expected_blocked = 0;
    for (stage, query) in queries(out).iter().enumerate() {
        let q = reader::words(g, query, a.reader().parent.parent.operators)?;
        let candidates = span::candidates(a, g, m, &e.records, query, span::Control::Full)?;
        let exact_candidates =
            span::candidates(a, g, m, &e.records, query, span::Control::ExactIdentity)?;
        let mut records = Vec::new();
        for (source, raw) in e.records.iter().enumerate() {
            let r = reader::words(g, raw, ordered::CANONICAL)?;
            let mut matches = vec![vec![]; q.len()];
            let mut exact_matches = vec![vec![]; q.len()];
            for (i, qw) in q.iter().enumerate() {
                for (j, rw) in r.iter().enumerate() {
                    if ordered::distance(m, &qw.geometry, &rw.geometry, false)? == 0 {
                        matches[i].push(j);
                    }
                    if qw.geometry.occurrences == rw.geometry.occurrences {
                        exact_matches[i].push(j);
                    }
                }
            }
            let query_barriers: Vec<_> = (0..r.len())
                .map(|j| matches.iter().any(|p| p.contains(&j)))
                .collect();
            let context_barriers: Vec<_> = r
                .iter()
                .map(|w| span_boundary::contains(m, &a.context_words, &w.geometry, false))
                .collect::<Result<_, _>>()?;
            let barriers: Vec<_> = query_barriers
                .iter()
                .zip(&context_barriers)
                .map(|(x, y)| *x || *y)
                .collect();
            let expected = e
                .expected_spans
                .get(stage)
                .filter(|p| p[0] == source && e.expected_queries.get(stage) == Some(query));
            let blocked = expected.is_some_and(|p| query_barriers[p[1]..p[2]].iter().any(|b| *b));
            expected_blocked += usize::from(blocked);
            let items:Vec<_>=candidates.iter().filter(|c| c.value.source==source).map(|c| {
                let expected=expected.is_some_and(|p| p[1]==c.value.word && p[2]==c.last_word);
                json!({"first_word":c.value.word,"last_word_exclusive":c.last_word,"start":c.value.start,"end":c.value.end,"bytes":c.value.bytes,"features":c.features,"active_features":span::feature_names().iter().enumerate().filter(|(i,_)| c.features&(1u32<<i)!=0).map(|(_,n)| n).collect::<Vec<_>>(),"admitted":a.matches(c.features),"oracle_span_evaluation_only":expected})
            }).collect();
            records.push(json!({"source":source,"word_occurrences":r.iter().enumerate().map(|(i,w)| json!({"word":i,"bytes":w.bytes,"bounds":[w.start,w.end]})).collect::<Vec<_>>(),"geometric_query_to_source_occurrences":matches,"exact_query_to_source_occurrences":exact_matches,"match_relation_equal":matches==exact_matches,"query_barriers":query_barriers,"context_barriers":context_barriers,"union_barriers":barriers,"oracle_answer_has_query_barrier_evaluation_only":blocked,"candidates":items}));
        }
        stages.push(json!({"stage":stage,"actual_query":query,"query_word_occurrences":q.iter().enumerate().map(|(i,w)|json!({"word":i,"bytes":w.bytes,"bounds":[w.start,w.end]})).collect::<Vec<_>>(),"candidate_geometry_exact_equal":candidates==exact_candidates,"records":records}));
    }
    Ok((
        json!({"id":e.id,"stages":stages,"oracle_blocked_spans_evaluation_only":expected_blocked}),
        expected_blocked,
    ))
}
#[test]
#[ignore = "bounded actual-artifact occurrence diagnostic"]
fn occurrence_correspondence_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty occurrence report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","examples":384,"families":32,"disjoint_rows":144,"overlap_rows":240,"controls":["Full","ExactIdentity","ReadDisabled"],"fits":0,"artifact_parameters_unchanged":true,"diagnostic_gate":"OVERLAPPING_OCCURRENCE_FAILURE_REPRODUCED requires at least one successful disjoint row, at least one failed overlap row whose correct span contains an actual query-match barrier, complete Full/ExactIdentity response equality and zero correct ReadDisabled answers; all counts are reported, no row is filtered","qualification":"NOT_RUN","retained_replay":"NOT_RUN","final_holdout":"NOT_RUN","promotion":false,"scope":"authored one/two-read full phrase endpoints, repeated word occurrences and both answer roles; not general prose"}),
        )?;
        let rows = data::corpus()?;
        let validation = data::validate(&rows)?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"validation":validation,"training":"NOT_RUN"}),
        )?;
        let guard = data::multiplicity_guard()?;
        write(&output, "multiplicity-guard.json", &guard)?;
        let parent = evidence.join("language-span-1/attempt-3");
        report_output::verify(&parent)?;
        let bytes = std::fs::read(parent.join("candidate.json"))?;
        if sha(&bytes) != "44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b" {
            return Err("retained span artifact differs".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = span::Artifact::decode(&bytes, &g)?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        let mut panels = BTreeMap::<String, Counts>::new();
        let mut groups = BTreeMap::<String, Counts>::new();
        let mut responses = Vec::new();
        let mut diagnostics = Vec::new();
        let mut exact_equal = 0;
        let mut overlap_failed_with_barrier = 0;
        let mut disjoint_correct = 0;
        let mut overlap_correct = 0;
        let mut read_disabled_correct = 0;
        for e in &rows {
            let full = phrase::generate(&a, &g, &m, &e.records, &e.prompt, phrase::Control::Full);
            let exact = phrase::generate(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                phrase::Control::ExactIdentity,
            );
            exact_equal += usize::from(match (&full, &exact) {
                (Ok(x), Ok(y)) => x == y,
                (Err(x), Err(y)) => x.to_string() == y.to_string(),
                _ => false,
            });
            if let Ok(out) = &full {
                let (diagnostic, blocked) = diagnose(&a, &g, &m, e, out)?;
                diagnostics.push(diagnostic);
                let yes = correct(out, e);
                overlap_failed_with_barrier += usize::from(e.overlapping && !yes && blocked > 0);
                disjoint_correct += usize::from(!e.overlapping && yes);
                overlap_correct += usize::from(e.overlapping && yes);
                if e.rotation == 0 && e.variant == "baseline" {
                    write(
                        &output,
                        &format!(
                            "sample-{}-subject{}-depth{}.json",
                            e.pattern,
                            usize::from(e.subject_answer),
                            e.depth
                        ),
                        &json!({"example":e,"generated":out}),
                    )?;
                }
            }
            let disabled = phrase::generate(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                phrase::Control::ReadDisabled,
            );
            read_disabled_correct +=
                usize::from(disabled.as_ref().is_ok_and(|out| correct(out, e)));
            for (control, result) in [
                ("Full", full),
                ("ExactIdentity", exact),
                ("ReadDisabled", disabled),
            ] {
                panels
                    .entry(control.into())
                    .or_default()
                    .add(result.as_ref().ok(), e);
                for group in [
                    format!("pattern/{}", e.pattern),
                    format!("role/{}", usize::from(e.subject_answer)),
                    format!("depth/{}", e.depth),
                    format!("variant/{}", e.variant),
                    format!(
                        "cell/{}/role{}/depth{}/{}",
                        e.pattern,
                        usize::from(e.subject_answer),
                        e.depth,
                        e.variant
                    ),
                ] {
                    groups
                        .entry(format!("{control}/{group}"))
                        .or_default()
                        .add(result.as_ref().ok(), e);
                }
                let mut response = match &result {
                    Ok(out) => compact(out, e),
                    Err(error) => json!({"error":error.to_string()}),
                };
                let object = response.as_object_mut().ok_or("response object")?;
                for (k, v) in [
                    ("id", json!(e.id)),
                    ("pattern", json!(e.pattern)),
                    ("overlapping", json!(e.overlapping)),
                    ("subject_answer", json!(e.subject_answer)),
                    ("depth", json!(e.depth)),
                    ("variant", json!(e.variant)),
                    ("control", json!(control)),
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
        write(&output, "occurrence-diagnostics.json", &json!(diagnostics))?;
        let guard_records: [Vec<u8>; 4] = serde_json::from_value(guard["records"].clone())?;
        let guard_prompt: Vec<u8> = serde_json::from_value(guard["prompt"].clone())?;
        let mut guard_responses = Vec::new();
        for control in [phrase::Control::Full, phrase::Control::ExactIdentity] {
            let response = match phrase::generate(
                &a,
                &g,
                &m,
                &guard_records,
                &guard_prompt,
                control,
            ) {
                Ok(out) => {
                    json!({"control":control,"correct_typed_unresolved":matches!(out.outcome,completion::Outcome::Unresolved{clause:1,reason:completion::Reason::NoCompatibleCandidate}),"generated":out})
                }
                Err(error) => {
                    json!({"control":control,"error":error.to_string(),"correct_typed_unresolved":false})
                }
            };
            guard_responses.push(response);
        }
        write(
            &output,
            "multiplicity-guard-responses.json",
            &json!(guard_responses),
        )?;
        let unchanged = a.encode()? == bytes;
        let reproduced = disjoint_correct > 0
            && overlap_failed_with_barrier > 0
            && exact_equal == rows.len()
            && read_disabled_correct == 0
            && unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if reproduced {"OVERLAPPING_OCCURRENCE_FAILURE_REPRODUCED"} else {"OCCURRENCE_DIAGNOSTIC_INCONCLUSIVE"},"development_rows":rows.len(),"panels":panels,"groups":groups,"disjoint_correct":disjoint_correct,"disjoint_rows":144,"overlap_correct":overlap_correct,"overlap_rows":240,"overlap_failed_with_actual_query_barrier":overlap_failed_with_barrier,"full_exact_identity_equal":exact_equal,"read_disabled_correct":read_disabled_correct,"multiplicity_guard_correct":guard_responses.iter().all(|r|r["correct_typed_unresolved"]==true),"fits":0,"candidate_sha256":sha(&bytes),"parent_sha256":sha(&bytes),"artifact_parameters_unchanged":unchanged,"source_sha256":sha(concat!(include_str!("occurrence_data.rs"),include_str!("occurrence_report.rs"),include_str!("phrase.rs"),include_str!("span.rs"),include_str!("span_boundary.rs")).as_bytes()),"qualification":"NOT_RUN","retained_replay":"NOT_RUN","final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
