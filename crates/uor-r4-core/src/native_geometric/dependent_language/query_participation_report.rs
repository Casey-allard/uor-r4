//! Output-supervised query participation fit and retained-path qualification.
//! Raw authored grammar and expected paths are evaluation-only.
use super::{
    completion, occurrence_role, phrase_data::intervals, query_participation as model,
    runtime::PayloadWindow, scheduling,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
        language_relation::runtime as lexical,
    },
    report_output,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Digest;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Selected {
    pub(super) source: usize,
    pub(super) span: [usize; 2],
    pub(super) bounds: [usize; 2],
    pub(super) value: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Expected {
    pub(super) queries: Vec<Vec<u8>>,
    pub(super) selected: Vec<Selected>,
    pub(super) matching_sources: Vec<Vec<usize>>,
    pub(super) outcome: completion::Outcome,
    pub(super) tokens: Vec<u16>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Example {
    pub(super) id: String,
    pub(super) family: String,
    pub(super) variant: String,
    pub(super) records: [Vec<u8>; 4],
    pub(super) prompt: Vec<u8>,
    pub(super) expected: Expected,
}
pub(super) fn words(raw: &[u8]) -> Vec<&[u8]> {
    intervals(raw).iter().map(|[a, b]| &raw[*a..*b]).collect()
}
pub(super) fn verb(w: &[u8]) -> bool {
    matches!(w, b"call" | b"help" | b"visit" | b"trust")
}
pub(super) fn oracle(records: &[Vec<u8>; 4], prompt: &[u8]) -> Result<Expected> {
    let mut e = Expected {
        queries: vec![],
        selected: vec![],
        matching_sources: vec![],
        outcome: completion::Outcome::Answered,
        tokens: vec![],
    };
    let clauses: Vec<_> = prompt
        .split_inclusive(|b| *b == b'?')
        .map(|q| q.strip_prefix(b" ").unwrap_or(q))
        .collect();
    if clauses.is_empty() || clauses.len() > 2 || prompt.len() > 256 {
        return Err("outside oracle clause bound".into());
    }
    for (clause, q) in clauses.iter().enumerate() {
        let refs: Vec<_> = intervals(q)
            .into_iter()
            .filter(|[a, b]| matches!(&q[*a..*b], b"they" | b"them"))
            .collect();
        let query = match (e.selected.last(), refs.as_slice()) {
            (None, []) => q.to_vec(),
            (Some(previous), [[a, b]]) => [&q[..*a], previous.value.as_slice(), &q[*b..]].concat(),
            _ => return Err("oracle reference shape".into()),
        };
        let raw_words = words(&query);
        let w = if raw_words.starts_with(&[b"please".as_slice(), b"tell", b"me"]) {
            &raw_words[3..]
        } else {
            &raw_words[..]
        };
        if w.len() < 4
            || w[0] != b"who"
            || !matches!(w[1], b"did" | b"will")
            || query.last() != Some(&b'?')
        {
            return Err("oracle question grammar".into());
        }
        let subject = verb(w[2]);
        let (relation, known) = if subject {
            (w[2], w[3..].join(&b' '))
        } else {
            (w[w.len() - 1], w[2..w.len() - 1].join(&b' '))
        };
        if !verb(relation) {
            return Err("oracle verb".into());
        }
        let mut hits = vec![];
        for (source, record) in records.iter().enumerate() {
            let r = words(record);
            let spans = intervals(record);
            let aux: Vec<_> = r
                .iter()
                .enumerate()
                .filter(|(i, w)| {
                    matches!(**w, b"did" | b"will") && r.get(i + 1).is_some_and(|v| verb(v))
                })
                .map(|(i, _)| i)
                .collect();
            if record.len() > 128 || r.len() > 16 || record.last() != Some(&b'.') || aux.len() != 1
            {
                return Err("oracle fact grammar".into());
            }
            let a = aux[0];
            if a == 0 || a + 2 >= r.len() || !verb(r[a + 1]) {
                return Err("oracle fact bounds".into());
            }
            let begin = usize::from(r.first() == Some(&b"today".as_slice()));
            let end = r.len() - usize::from(r.last() == Some(&b"tomorrow".as_slice()));
            if a <= begin || a + 2 >= end {
                return Err("oracle styled fact bounds".into());
            }
            let (answer, endpoint) = if subject {
                ([begin, a], [a + 2, end])
            } else {
                ([a + 2, end], [begin, a])
            };
            if r[a] == w[1]
                && r[a + 1] == relation
                && r[endpoint[0]..endpoint[1]].join(&b' ') == known
            {
                let bounds = [spans[answer[0]][0], spans[answer[1] - 1][1]];
                hits.push(Selected {
                    source,
                    span: answer,
                    bounds,
                    value: record[bounds[0]..bounds[1]].to_vec(),
                });
            }
        }
        e.queries.push(query);
        e.matching_sources
            .push(hits.iter().map(|v| v.source).collect());
        if hits.len() != 1 {
            e.outcome = completion::Outcome::Unresolved {
                clause,
                reason: if hits.is_empty() {
                    completion::Reason::NoCompatibleCandidate
                } else {
                    completion::Reason::Ambiguous
                },
            };
            return Ok(e);
        }
        e.selected.push(hits.remove(0));
    }
    e.tokens = e
        .selected
        .last()
        .ok_or("oracle missing answer")?
        .value
        .iter()
        .map(|b| u16::from(*b))
        .chain([256])
        .collect();
    Ok(e)
}
pub(super) fn fact(
    known: &str,
    auxiliary: &str,
    relation: &str,
    answer: &str,
    subject: bool,
) -> Vec<u8> {
    if subject {
        format!("{answer} {auxiliary} {relation} {known}.")
    } else {
        format!("{known} {auxiliary} {relation} {answer}.")
    }
    .into_bytes()
}
pub(super) fn question(known: &str, auxiliary: &str, relation: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("who {auxiliary} {relation} {known}?")
    } else {
        format!("who {auxiliary} {known} {relation}?")
    }
    .into_bytes()
}
pub(super) fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
pub(super) fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
pub(super) fn assess(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    c: model::Control,
    out: &completion::Generated,
) -> Result<Value> {
    let expected_queries = e
        .expected
        .queries
        .iter()
        .map(|q| lexical::Query::new(g, q, a.parent.reader().parent.parent.operators))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let queries: Vec<_> = out
        .decisions
        .iter()
        .filter(|d| d.before.core.cursor == 0)
        .map(|d| &d.before.core.query)
        .collect();
    let mut selected = vec![];
    for step in out.trace.steps.iter().filter(|s| s.before.core.cursor == 0) {
        if let Some(value) = &step.observation.route.selected {
            let ranges = intervals(&e.records[value.source]);
            let first = ranges
                .iter()
                .position(|w| w[0] == value.start)
                .ok_or("selection start is not a word boundary")?;
            let last = ranges
                .iter()
                .position(|w| w[1] == value.end)
                .ok_or("selection end is not a word boundary")?;
            selected.push(Selected {
                source: value.source,
                span: [first, last + 1],
                bounds: [value.start, value.end],
                value: value.bytes.clone(),
            });
        }
    }
    let outcome = out.outcome == e.expected.outcome
        && out.trace.tokens == e.expected.tokens
        && !out.trace.exhausted;
    let answered = e.expected.outcome == completion::Outcome::Answered;
    let path = if answered {
        selected == e.expected.selected && queries == expected_queries.iter().collect::<Vec<_>>()
    } else {
        out.trace.steps.is_empty()
            && out.decisions.len() == 1
            && queries == vec![&expected_queries[0]]
    };
    let continuity = out
        .trace
        .steps
        .windows(2)
        .all(|s| s[0].after == s[1].before);
    let first = out.decisions.first().ok_or("missing first decision")?;
    let clauses = scheduling::clauses(&e.prompt)?;
    let observed = completion::observe_routed_updates(
        &a.parent.parent.parent.parent,
        g,
        m,
        &e.records,
        &clauses,
        &first.before,
        completion::Control::Full,
        PayloadWindow::Phrase,
        &|b: &[u8], _| Ok(b.to_vec()),
        &mut |q: &[u8], _| model::route(a, g, m, &e.records, q, c),
        &|q, update| model::allow_update(a, g, m, q, update, c),
    )?;
    let initial = observed.core.route.selected.as_ref();
    let intended = &e.expected.selected[0];
    let initial_correct = initial.is_some_and(|s| {
        s.source == intended.source
            && s.word == intended.span[0]
            && [s.start, s.end] == intended.bounds
            && s.bytes == intended.value
    });
    let next = &observed.core.core.next_query;
    let next_query_roles = occurrence_role::query_context(&a.parent, g, m, &next.bytes, false)?;
    let next_correct = next == &expected_queries[1];
    let next_route = model::route(a, g, m, &e.records, &next.bytes, c)?;
    let actual_sources: Vec<_> = next_route.compatible.iter().map(|s| s[0]).collect();
    let sources_correct = actual_sources == e.expected.matching_sources[1];
    Ok(
        json!({"correct":outcome&&path&&continuity&&initial_correct&&next_correct&&sources_correct,"initial_selection_correct":initial_correct,"lookahead_query_geometry_correct":next_correct,"exact_compatible_sources_correct":sources_correct,"actual_initial_selection":initial,"actual_lookahead_route":next_route,"learned_context_of_actual_next_query":next_query_roles,"outcome_correct":outcome,"committed_path_queries_source_span_bounds_correct":path,"continuity":continuity,"actual_selected":selected,"lookahead_is_recomputed_from_actual_first_frame":true,"committed_steps":out.trace.steps.len()}),
    )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ReplacementTemplate {
    id: String,
    records: [Vec<u8>; 4],
    first_question: Vec<u8>,
    question: Vec<u8>,
    answer: Vec<u8>,
}
fn prefix(q: Vec<u8>, yes: bool) -> Vec<u8> {
    if yes {
        [b"please tell me ".as_slice(), &q].concat()
    } else {
        q
    }
}
fn training() -> Result<(Vec<model::DirectExample>, Vec<ReplacementTemplate>)> {
    let mut direct = vec![];
    let mut replacement = vec![];
    for auxiliary in ["did", "will"] {
        for relation in ["call", "help", "visit"] {
            let other = if relation == "visit" { "call" } else { "visit" };
            for subject in [false, true] {
                for known in ["alice", "bruno", "please", "tell", "me", "who"] {
                    for prefixed in [false, true] {
                        if prefixed && matches!(known, "please" | "tell" | "me" | "who") {
                            continue;
                        }
                        for irrelevant in ["cedar", "please"] {
                            // The alternate question introduction is authored only here.
                            // No runtime parser or word role label is supplied to the fit.
                            let mut q = question(known, auxiliary, relation, subject);
                            if known == "who" {
                                q = [b"which person ".as_slice(), &q[4..]].concat();
                            }
                            direct.push(model::DirectExample {
                                records: [
                                    fact(known, auxiliary, relation, "helen", subject),
                                    fact("felix", auxiliary, relation, "oscar", subject),
                                    fact(irrelevant, auxiliary, other, "amber", subject),
                                    fact("ruby", auxiliary, other, "dylan", subject),
                                ],
                                question: prefix(q, prefixed),
                                answer: b"helen".to_vec(),
                            });
                        }
                    }
                }
                for prefixed in [false, true] {
                    for irrelevant in ["please", "tell", "me", "who"] {
                        replacement.push(ReplacementTemplate {
                            id: format!("replacement-{auxiliary}-{relation}-{subject}-{prefixed}-{irrelevant}"),
                            records: [
                                fact("alice", auxiliary, other, "bruno", subject),
                                fact("bruno", auxiliary, relation, "helen", subject),
                                fact("felix", auxiliary, relation, "oscar", subject),
                                fact(irrelevant, auxiliary, other, "amber", subject),
                            ],
                            first_question: question("alice", auxiliary, other, subject),
                            question: prefix(question(if subject { "them" } else { "they" }, auxiliary, relation, subject), prefixed),
                            answer: b"helen".to_vec(),
                        });
                    }
                }
            }
        }
    }
    if direct.len() != 192 || replacement.len() != 96 {
        return Err("training count".into());
    }
    let mut ids = BTreeSet::new();
    for e in &direct {
        if e.records
            .iter()
            .any(|r| r.len() > 128 || words(r).len() > 16)
            || e.question.len() > 128
            || !ids.insert(serde_json::to_vec(&(
                e.records.clone(),
                e.question.clone(),
            ))?)
        {
            return Err("direct bounds or duplicate".into());
        }
        if e.records
            .iter()
            .filter(|r| words(r).contains(&b"helen".as_slice()))
            .count()
            != 1
        {
            return Err("direct answer identity must be unique".into());
        }
    }
    for e in &replacement {
        if e.records
            .iter()
            .any(|r| r.len() > 128 || words(r).len() > 16)
            || e.question.len() + e.first_question.len() + 1 > 256
        {
            return Err("replacement bounds".into());
        }
        let actual_oracle = oracle(
            &e.records,
            &[e.first_question.clone(), e.question.clone()].join(&b' '),
        )?;
        if actual_oracle.selected.len() != 2 || actual_oracle.selected[1].value != e.answer {
            return Err("replacement final answer oracle".into());
        }
        // This validation does not give the first selection or splice site to fitting.
    }
    Ok((direct, replacement))
}
pub(super) fn prior_examples(root: &Path) -> Result<(Vec<Example>, Vec<Value>, String, String)> {
    report_output::verify(root)?;
    let data = std::fs::read(root.join("data.json"))?;
    let responses = std::fs::read(root.join("responses.json"))?;
    let parsed: Value = serde_json::from_slice(&data)?;
    let rows = serde_json::from_value(parsed["development"].clone())?;
    Ok((
        rows,
        serde_json::from_slice(&responses)?,
        sha(&data),
        sha(&responses),
    ))
}
pub(super) fn earlier_200(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    root: &Path,
) -> Result<Value> {
    report_output::verify(root)?;
    let data_bytes = std::fs::read(root.join("data.json"))?;
    let response_bytes = std::fs::read(root.join("responses.json"))?;
    let data: Value = serde_json::from_slice(&data_bytes)?;
    let responses: Vec<Value> = serde_json::from_slice(&response_bytes)?;
    let mut total = 0;
    let mut equal = 0;
    let mut parent_equal = 0;
    let mut items = vec![];
    for (key, label, required) in [
        ("development", "dual_role", 72),
        ("lexical_role", "lexical_role", 48),
        ("phrase_order", "phrase_order", 80),
    ] {
        let examples: Vec<Example> = serde_json::from_value(data[key].clone())?;
        if examples.len() != required {
            return Err("prior 200 count".into());
        }
        for e in examples {
            let matches: Vec<_> = responses
                .iter()
                .filter(|v| v["panel"] == label && v["id"] == e.id && v["control"] == "Full")
                .collect();
            if matches.len() != 1 {
                return Err("prior Full identity".into());
            }
            let old: completion::Generated =
                serde_json::from_value(matches[0]["response"]["generated"].clone())?;
            let parent = occurrence_role::generate(
                &a.parent,
                g,
                m,
                &e.records,
                &e.prompt,
                occurrence_role::Control::Full,
            )?;
            let actual = model::generate(a, g, m, &e.records, &e.prompt, model::Control::Full)?;
            let same = old == actual;
            total += 1;
            equal += usize::from(same);
            parent_equal += usize::from(parent == old);
            items.push(json!({"panel":label,"id":e.id,"full_trace_equal":same,"parent_full_trace_equal":parent==old,"actual_if_changed":if same {Value::Null} else {serde_json::to_value(actual)?}}));
        }
    }
    Ok(
        json!({"root":root,"data_sha256":sha(&data_bytes),"responses_sha256":sha(&response_bytes),"rows":total,"equal":equal,"parent_equal":parent_equal,"pass":total==200&&equal==200&&parent_equal==200,"items":items}),
    )
}
pub(super) fn earlier_6688(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value> {
    let mut total = 0;
    let mut equal = 0;
    let mut unresolved = 0;
    let mut panels = vec![];
    for (name, attempt, required) in [
        ("language-phrase-update-1", "attempt-1", 864),
        ("language-span-1", "attempt-3", 1728),
        ("language-scheduling-1", "attempt-1", 1536),
        ("language-depth-1", "attempt-1", 1280),
        ("language-completion-1", "attempt-1", 896),
        ("correspondence-runs-1", "attempt-2", 384),
    ] {
        let root = evidence.join(name).join(attempt);
        report_output::verify(&root)?;
        let data: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
        let rows = data["development"]
            .as_array()
            .ok_or("retained development")?;
        if rows.len() != required {
            return Err("retained panel count".into());
        }
        let mut items = vec![];
        let mut panel_equal = 0;
        for row in rows {
            let records: [Vec<u8>; 4] = serde_json::from_value(row["records"].clone())?;
            let prompt: Vec<u8> = serde_json::from_value(row["prompt"].clone())?;
            let old = occurrence_role::generate(
                &a.parent,
                g,
                m,
                &records,
                &prompt,
                occurrence_role::Control::Full,
            )?;
            let actual = model::generate(a, g, m, &records, &prompt, model::Control::Full)?;
            let same = old == actual;
            panel_equal += usize::from(same);
            unresolved += usize::from(
                same && matches!(actual.outcome, completion::Outcome::Unresolved { .. }),
            );
            items.push(json!({"id":row["id"],"full_trace_and_decisions_equal":same,"actual":if same{Value::Null}else{serde_json::to_value(actual)?},"previous":if same{Value::Null}else{serde_json::to_value(old)?}}));
        }
        total += rows.len();
        equal += panel_equal;
        panels.push(json!({"source":name,"rows":rows.len(),"equal":panel_equal,"items":items}));
    }
    Ok(
        json!({"rows":total,"equal_rows":equal,"typed_unresolved":unresolved,"pass":total==6688&&equal==6688&&unresolved==512,"panels":panels,"comparison":"actual current parent path versus candidate; prior sealed report establishes these parent Full paths retained their earlier correspondences","historical_adapter_control_replay":"NOT_RUN"}),
    )
}
#[test]
fn query_participation_training_has_independent_answers_and_causal_distractors() -> Result<()> {
    let (direct, replacement) = training()?;
    assert_eq!(direct.len(), 192);
    assert_eq!(replacement.len(), 96);
    assert!(direct
        .iter()
        .any(|e| words(&e.question).contains(&b"who".as_slice())
            && words(&e.records[0]).contains(&b"who".as_slice())));
    for e in replacement {
        assert_eq!(words(&e.records[1])[1], words(&e.records[2])[1]);
        assert_eq!(words(&e.records[1])[2], words(&e.records[2])[2]);
        assert_ne!(e.records[1], e.records[2]);
    }
    Ok(())
}
fn representation_audit(a: &model::Artifact, g: &BoundGeometry, m: &Metric) -> Result<Value> {
    let optional = b"please tell me who did they help?";
    let observed = model::observations(a, g, m, optional, false)?;
    let exact = model::observations(a, g, m, optional, true)?;
    let mut pairs = vec![];
    let mut pass = observed == exact;
    for (position, name) in ["please", "tell", "me", "who"].into_iter().enumerate() {
        let required = if name == "who" {
            b"which person did who call?".to_vec()
        } else {
            question(name, "did", "call", false)
        };
        let index = words(&required)
            .iter()
            .position(|w| *w == name.as_bytes())
            .ok_or("audit required position")?;
        let keys = model::observations(a, g, m, &required, false)?;
        let same_identity = observed[position].center == keys[index].center;
        let distinct_context = observed[position] != keys[index];
        pass &= same_identity && distinct_context && observed[position].center < 64;
        pairs.push(json!({"word":name,"optional_query":optional.to_vec(),"required_query":required,"optional_key":observed[position],"required_key":keys[index],"same_canonical_identity":same_identity,"distinct_occurrence_context":distinct_context}));
    }
    let distinct_centers: BTreeSet<_> = observed[..4].iter().map(|k| k.center).collect();
    let old = &a.parent.parent.parent.context_words;
    let previously_collapsed = ["please", "tell", "me", "who", "they", "them"]
        .iter()
        .all(|name| !old.iter().any(|w| w.bytes == name.as_bytes()));
    pass &= distinct_centers.len() == 4 && previously_collapsed;
    Ok(
        json!({"pass":pass,"status":"BEFORE_FIT","old_context_center_collapsed_these_identities":previously_collapsed,"distinct_new_prefix_centers":distinct_centers.len(),"geometric_exact_observations_equal":observed==exact,"same_identity_optional_required_pairs":pairs,"scope":"representation preserves these authored distinctions; this is not learned correctness or general grammar evidence"}),
    )
}
#[test]
#[ignore = "bounded output-credit fit and query-participation qualification"]
fn learned_query_participation_report() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_QUERY_PARTICIPATION_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_QUERY_PARTICIPATION_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let (direct, templates) = training()?;
        let transfer_root = evidence.join("role-transfer-1/attempt-1");
        let (rows, previous, data_sha, response_sha) = prior_examples(&transfer_root)?;
        if rows.len() != 204 || previous.len() != 204 {
            return Err("frozen transfer count".into());
        }
        // Recompute the authored oracle before the candidate or parent is loaded.
        if rows
            .iter()
            .any(|e| oracle(&e.records, &e.prompt).map_or(true, |v| v != e.expected))
        {
            return Err("frozen raw oracle drift".into());
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD_AND_FIT","coverage":60,"prior_successes_exact":120,"prior_200_exact":200,"earlier_Full_exact":6688,"earlier_typed_unresolved":512,"styled_prior_failures":36,"styled_boundary":"improvement not required; every previous success must retain its complete path","training":{"direct":192,"replacement_templates":96,"fits":2,"supervision":"final direct byte/EOS answer only; replacement payload derived from actual first route after requirements fit; no oracle span or splice index"},"controls":{"ExactIdentity":"all204 full structures equal","reload":"all204 full structures equal","BothDisabled":"all204 complete structures equal sealed parent","RequirementsDisabled":"must lose coverage improvements","ReplacementDisabled":"must lose coverage improvements","ReadDisabled":"zero correct dependent answered responses","UpdateDisabled":"zero correct dependent answered responses"},"limits":{"records":4,"record_words":16,"record_bytes":128,"prompt_bytes":256,"clauses":2,"payload_words":3,"payload_bytes":50,"steps":96},"final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"direct_training":direct,"replacement_templates":templates,"development":rows,"source_development":transfer_root,"source_data_sha256":data_sha,"source_responses_sha256":response_sha,"training_construction":"authored direct relations and final answers, with same-relation different-endpoint distractor; optional prefix identities also occur as required names; evaluation expected paths never enter either fit"}),
        )?;
        let prior = evidence.join("occurrence-role-1/attempt-3");
        report_output::verify(&prior)?;
        let parent_bytes = std::fs::read(prior.join("candidate.json"))?;
        if sha(&parent_bytes) != "712de3158ca14241b549043f743709911ca6bbc1bf8adc271d990d8616c8df42"
        {
            return Err("parent identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent = occurrence_role::Artifact::decode(&parent_bytes, &g)?;
        let training_queries: Vec<Vec<u8>> = direct
            .iter()
            .map(|e| e.question.clone())
            .chain(
                templates
                    .iter()
                    .flat_map(|e| [e.first_question.clone(), e.question.clone()]),
            )
            .collect();
        let source_digest = *blake3::hash(
            concat!(
                include_str!("query_participation.rs"),
                include_str!("occurrence.rs"),
                include_str!("occurrence_role.rs"),
                include_str!("scheduling.rs"),
                include_str!("completion.rs"),
                include_str!("query_participation_report.rs")
            )
            .as_bytes(),
        )
        .as_bytes();
        let initial = model::initialize(parent, &g, &training_queries, source_digest)?;
        let audit = representation_audit(&initial, &g, &m)?;
        write(&output, "representation-audit.json", &audit)?;
        if audit["pass"] != true {
            return Err("representation loses required authored distinction".into());
        }
        let (required, required_fit) = model::fit_required(initial, &g, &m, &direct)?;
        std::fs::write(
            output.join("requirements-candidate.json"),
            required.encode()?,
        )?;
        write(&output, "requirements-fit.json", &json!(required_fit))?;
        let mut replacements = vec![];
        let mut preparation = vec![];
        for e in &templates {
            let route = model::route(
                &required,
                &g,
                &m,
                &e.records,
                &e.first_question,
                model::Control::Full,
            )?;
            if let Some(selected) = &route.selected {
                replacements.push(model::ReplacementExample {
                    records: e.records.clone(),
                    question: e.question.clone(),
                    payload: selected.bytes.clone(),
                    answer: e.answer.clone(),
                });
            }
            preparation.push(
                json!({"id":e.id,"actual_first_route":route,"included":route.selected.is_some()}),
            );
        }
        write(
            &output,
            "replacement-inputs.json",
            &json!({"actual_first_reads":preparation,"training":replacements,"expected_intermediate_metadata_used":false}),
        )?;
        if replacements.len() != templates.len() {
            return Err("actual first route unavailable for replacement training; checkpoint before second fit".into());
        }
        let (a, replacement_fit) = model::fit_replacements(required, &g, &m, &replacements)?;
        write(&output, "replacement-fit.json", &json!(replacement_fit))?;
        let bytes = a.encode()?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        let reload = model::Artifact::decode(&bytes, &g)?;
        let mut invalid = serde_json::to_value(&a)?;
        invalid["schema"] = json!(65535);
        let invalid_schema_rejected =
            model::Artifact::decode(&serde_json::to_vec(&invalid)?, &g).is_err();
        let mut direct_correct = 0;
        let mut direct_results = vec![];
        for (i, e) in direct.iter().enumerate() {
            let out = model::generate(&a, &g, &m, &e.records, &e.question, model::Control::Full)?;
            let correct = out.outcome == completion::Outcome::Answered
                && !out.trace.exhausted
                && out.trace.tokens
                    == e.answer
                        .iter()
                        .copied()
                        .map(u16::from)
                        .chain([256])
                        .collect::<Vec<_>>();
            direct_correct += usize::from(correct);
            direct_results.push(
                json!({"row":i,"correct":correct,"outcome":out.outcome,"tokens":out.trace.tokens}),
            );
        }
        write(
            &output,
            "training-responses.json",
            &json!({"direct_correct":direct_correct,"direct_rows":direct.len(),"direct":direct_results}),
        )?;
        let mut responses = vec![];
        let mut counts = BTreeMap::<String, usize>::new();
        let mut exact_equal = 0;
        let mut reload_equal = 0;
        let mut disabled_answer_correct = 0;
        let mut prior_successes = 0;
        let mut prior_success_equal = 0;
        let mut both_disabled_equal = 0;
        for e in &rows {
            let panel = e.family.split('-').next().ok_or("panel")?;
            let old = previous
                .iter()
                .find(|r| r["id"] == e.id)
                .ok_or("prior row identity")?;
            let parent_out: completion::Generated = serde_json::from_value(old["Full"].clone())?;
            let old_success = old["controls"]["Full"]["assessment"]["correct"] == true;
            let full = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            reload_equal += usize::from(
                full == model::generate(
                    &reload,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::Full,
                )?,
            );
            prior_successes += usize::from(old_success);
            prior_success_equal += usize::from(old_success && full == parent_out);
            let mut controls = serde_json::Map::new();
            for (name, c) in [
                ("Full", model::Control::Full),
                ("ExactIdentity", model::Control::ExactIdentity),
                ("RequirementsDisabled", model::Control::RequirementsDisabled),
                ("ReplacementDisabled", model::Control::ReplacementDisabled),
                ("BothDisabled", model::Control::BothDisabled),
                ("ReadDisabled", model::Control::ReadDisabled),
                ("UpdateDisabled", model::Control::UpdateDisabled),
            ] {
                let out = if c == model::Control::Full {
                    full.clone()
                } else {
                    model::generate(&a, &g, &m, &e.records, &e.prompt, c)?
                };
                if c == model::Control::ExactIdentity {
                    exact_equal += usize::from(out == full);
                }
                if c == model::Control::BothDisabled {
                    both_disabled_equal += usize::from(out == parent_out);
                }
                let assessment = assess(&a, &g, &m, e, c, &out)?;
                *counts.entry(format!("{panel}/{name}")).or_default() +=
                    usize::from(assessment["correct"] == true);
                if matches!(
                    c,
                    model::Control::ReadDisabled | model::Control::UpdateDisabled
                ) && e.expected.outcome == completion::Outcome::Answered
                {
                    disabled_answer_correct += usize::from(assessment["outcome_correct"] == true);
                }
                controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"steps":out.trace.steps.len(),"equal_full":out==full,"equal_parent":out==parent_out}));
            }
            let clauses = scheduling::clauses(&e.prompt)?;
            let observations = clauses.iter().map(|q| Ok(json!({"query":q,"keys":model::observations(&a,&g,&m,q,false)?,"required_mask":a.required_mask(&g,&m,q,false)?,"replacement_mask":a.replacement_mask(&g,&m,q,false)?}))).collect::<Result<Vec<Value>>>()?;
            responses.push(json!({"id":e.id,"panel":panel,"family":e.family,"variant":e.variant,"Full":full,"controls":controls,"query_participation":observations,"previous_success":old_success,"previous_success_exact":old_success&&full==parent_out}));
        }
        write(&output, "responses.json", &json!(responses))?;
        let retained_200 = earlier_200(&a, &g, &m, &prior)?;
        write(&output, "retained-200.json", &retained_200)?;
        let retained_6688 = earlier_6688(&a, &g, &m, &evidence)?;
        write(&output, "retained-6688.json", &retained_6688)?;
        let parent_unchanged = a.parent.encode()? == parent_bytes
            && std::fs::read(prior.join("candidate.json"))? == parent_bytes;
        let pass = counts["coverage/Full"] == 60
            && prior_successes == 120
            && prior_success_equal == 120
            && exact_equal == 204
            && reload_equal == 204
            && both_disabled_equal == 204
            && counts["coverage/RequirementsDisabled"] < 60
            && counts["coverage/ReplacementDisabled"] < 60
            && disabled_answer_correct == 0
            && retained_200["pass"] == true
            && retained_6688["pass"] == true
            && direct_correct == 192
            && invalid_schema_rejected
            && parent_unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_LEARNED_QUERY_PARTICIPATION"}else{"FAIL_LEARNED_QUERY_PARTICIPATION"},"panels":counts,"rows":204,"prior_successes":prior_successes,"prior_success_exact":prior_success_equal,"exact_equal":exact_equal,"reload_equal":reload_equal,"both_disabled_parent_equal":both_disabled_equal,"read_or_update_disabled_correct_answer":disabled_answer_correct,"direct_training_correct":direct_correct,"direct_training_rows":192,"replacement_training_rows":96,"required_fit":required_fit,"replacement_fit":replacement_fit,"retained_200_equal":retained_200["equal"],"parent_200_equal":retained_200["parent_equal"],"retained_6688_equal":retained_6688["equal_rows"],"retained_typed_unresolved":retained_6688["typed_unresolved"],"candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"parent_parameters_unchanged":parent_unchanged,"invalid_schema_rejected":invalid_schema_rejected,"fits":2,"requirements_fits":1,"replacement_fits":1,"source_role_fits":0,"query_role_fits":0,"final_holdout":"NOT_RUN","promotion":false,"default_model":"15baec48 unchanged"}),
        )?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"gate":if pass{"PASS_LEARNED_QUERY_PARTICIPATION"}else{"FAIL_LEARNED_QUERY_PARTICIPATION"},"path":output})
            )?
        );
        Ok(())
    })();
    if let Err(error) = &result {
        std::fs::write(output.join("error.txt"), error.to_string())?;
    }
    report_output::seal(&output)?;
    report_output::verify(&output)?;
    result
}

#[test]
#[ignore = "unchanged fitted artifact: optional and required same-identity occurrences"]
fn same_identity_participation_probe() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_QUERY_PARTICIPATION_PROBE")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_QUERY_PARTICIPATION_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty probe paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let mut rows = vec![];
        for known in ["please", "tell", "me", "who"] {
            for subject in [false, true] {
                for rotation in 0..4 {
                    for variant in ["baseline", "active", "inactive"] {
                        let original = [
                            fact(known, "did", "call", "bruno", subject),
                            fact(
                                "bruno",
                                "did",
                                "help",
                                if variant == "active" {
                                    "oscar"
                                } else {
                                    "helen"
                                },
                                subject,
                            ),
                            fact(
                                "felix",
                                "did",
                                "help",
                                if variant == "inactive" {
                                    "cedar"
                                } else {
                                    "amber"
                                },
                                subject,
                            ),
                            fact("ruby", "did", "visit", "dylan", subject),
                        ];
                        let records = std::array::from_fn(|i| original[(i + rotation) % 4].clone());
                        let prompt = [
                            prefix(question(known, "did", "call", subject), true),
                            prefix(
                                question(
                                    if subject { "them" } else { "they" },
                                    "did",
                                    "help",
                                    subject,
                                ),
                                true,
                            ),
                        ]
                        .join(&b' ');
                        let expected = oracle(&records, &prompt)?;
                        if expected.outcome != completion::Outcome::Answered
                            || expected.selected.len() != 2
                            || expected.selected[0].value != b"bruno"
                        {
                            return Err("same-identity raw oracle".into());
                        }
                        rows.push(Example {
                            id: format!("collision-{known}-{subject}-{rotation}-{variant}"),
                            family: format!("collision-{known}-{subject}-{rotation}"),
                            variant: variant.into(),
                            records,
                            prompt,
                            expected,
                        });
                    }
                }
            }
        }
        if rows.len() != 96 {
            return Err("probe rows".into());
        }
        for e in rows.iter().filter(|e| e.variant != "baseline") {
            let baseline = rows
                .iter()
                .find(|b| b.family == e.family && b.variant == "baseline")
                .ok_or("probe baseline")?;
            if e.prompt != baseline.prompt
                || e.records
                    .iter()
                    .zip(&baseline.records)
                    .filter(|(a, b)| a != b)
                    .count()
                    != 1
                || (e.expected.tokens != baseline.expected.tokens) != (e.variant == "active")
            {
                return Err("probe intervention".into());
            }
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_CANDIDATE_LOAD","rows":96,"Full":"exact answers, two source/spans/bounds, payload, next-query and EOS", "ExactIdentity":"96 complete structures equal Full","reload":"96 equal Full","BothDisabled":"96 equal original parent structures","RequirementsDisabled":"loses every Full success","ReplacementDisabled":"diagnostic, no fixed success count","ReadDisabled":"zero correct answered responses","UpdateDisabled":"zero correct answered responses","fits":0,"scope":"open development same-identity recombination; no independent final holdout"}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"source":"authored independent raw oracle; same identities occupy optional question prefix and required known endpoint; one-record active/inactive variants","training":[]}),
        )?;
        let root = evidence.join("query-participation-1/attempt-1");
        report_output::verify(&root)?;
        let bytes = std::fs::read(root.join("candidate.json"))?;
        if sha(&bytes) != "8055a53c7801f1e6e11f808a6c15ddb090a5befe8d9d587b4ca65c74b7698793" {
            return Err("probe artifact identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = model::Artifact::decode(&bytes, &g)?;
        let reload = model::Artifact::decode(&a.encode()?, &g)?;
        let mut counts = BTreeMap::<String, usize>::new();
        let mut responses = vec![];
        let mut exact = 0;
        let mut reloaded = 0;
        let mut parent_equal = 0;
        let mut disabled_correct = 0;
        for e in &rows {
            let full = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            reloaded += usize::from(
                full == model::generate(
                    &reload,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::Full,
                )?,
            );
            let parent = occurrence_role::generate(
                &a.parent,
                &g,
                &m,
                &e.records,
                &e.prompt,
                occurrence_role::Control::Full,
            )?;
            let mut controls = serde_json::Map::new();
            for (name, c) in [
                ("Full", model::Control::Full),
                ("ExactIdentity", model::Control::ExactIdentity),
                ("RequirementsDisabled", model::Control::RequirementsDisabled),
                ("ReplacementDisabled", model::Control::ReplacementDisabled),
                ("BothDisabled", model::Control::BothDisabled),
                ("ReadDisabled", model::Control::ReadDisabled),
                ("UpdateDisabled", model::Control::UpdateDisabled),
            ] {
                let out = if c == model::Control::Full {
                    full.clone()
                } else {
                    model::generate(&a, &g, &m, &e.records, &e.prompt, c)?
                };
                let assessment = assess(&a, &g, &m, e, c, &out)?;
                *counts.entry(name.into()).or_default() +=
                    usize::from(assessment["correct"] == true);
                if c == model::Control::ExactIdentity {
                    exact += usize::from(out == full);
                }
                if c == model::Control::BothDisabled {
                    parent_equal += usize::from(out == parent);
                }
                if matches!(
                    c,
                    model::Control::ReadDisabled | model::Control::UpdateDisabled
                ) {
                    disabled_correct += usize::from(assessment["outcome_correct"] == true);
                }
                controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"equal_full":out==full}));
            }
            responses.push(json!({"id":e.id,"Full":full,"controls":controls}));
        }
        write(&output, "responses.json", &json!(responses))?;
        report_output::verify(&root)?;
        let unchanged = std::fs::read(root.join("candidate.json"))? == bytes;
        let pass = counts["Full"] == 96
            && exact == 96
            && reloaded == 96
            && parent_equal == 96
            && counts["RequirementsDisabled"] == 0
            && disabled_correct == 0
            && unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_SAME_IDENTITY_PARTICIPATION"}else{"FAIL_SAME_IDENTITY_PARTICIPATION"},"rows":96,"counts":counts,"exact_equal":exact,"reload_equal":reloaded,"both_disabled_parent_equal":parent_equal,"disabled_correct_answer":disabled_correct,"candidate_sha256":sha(&bytes),"artifact_unchanged":unchanged,"fits":0,"final_holdout":"NOT_RUN","promotion":false}),
        )?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"gate":if pass{"PASS_SAME_IDENTITY_PARTICIPATION"}else{"FAIL_SAME_IDENTITY_PARTICIPATION"},"path":output})
            )?
        );
        Ok(())
    })();
    if let Err(e) = &result {
        std::fs::write(output.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&output)?;
    report_output::verify(&output)?;
    result
}
