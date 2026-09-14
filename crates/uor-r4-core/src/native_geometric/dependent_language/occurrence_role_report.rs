//! Learned occurrence-role development report. Raw grammar is an evaluation
//! oracle only; direct fit receives records, prompt and final answer bytes.
use super::{
    completion, correspondence, occurrence_role as model, phrase_data::intervals,
    runtime::PayloadWindow, scheduling, span_data,
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
struct Selected {
    source: usize,
    span: [usize; 2],
    bounds: [usize; 2],
    value: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Expected {
    queries: Vec<Vec<u8>>,
    selected: Vec<Selected>,
    matching_sources: Vec<Vec<usize>>,
    outcome: completion::Outcome,
    tokens: Vec<u16>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Example {
    id: String,
    family: String,
    variant: String,
    records: [Vec<u8>; 4],
    prompt: Vec<u8>,
    expected: Expected,
}
fn words(raw: &[u8]) -> Vec<&[u8]> {
    intervals(raw).iter().map(|[a, b]| &raw[*a..*b]).collect()
}
fn verb(w: &[u8]) -> bool {
    matches!(w, b"call" | b"help" | b"visit")
}
fn oracle(records: &[Vec<u8>; 4], prompt: &[u8]) -> Result<Expected> {
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
        let w = words(&query);
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
            let (answer, endpoint) = if subject {
                ([0, a], [a + 2, r.len()])
            } else {
                ([a + 2, r.len()], [0, a])
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
fn fact(known: &str, auxiliary: &str, relation: &str, answer: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("{answer} {auxiliary} {relation} {known}.")
    } else {
        format!("{known} {auxiliary} {relation} {answer}.")
    }
    .into_bytes()
}
fn question(known: &str, auxiliary: &str, relation: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("who {auxiliary} {relation} {known}?")
    } else {
        format!("who {auxiliary} {known} {relation}?")
    }
    .into_bytes()
}
fn development() -> Result<Vec<Example>> {
    let mut rows = vec![];
    for endpoint in ["will", "will amber", "ruby will amber"] {
        for subject in [false, true] {
            for rotation in 0..4 {
                for variant in ["baseline", "active", "inactive"] {
                    let records0 = [
                        fact("alice", "will", "call", endpoint, subject),
                        fact(
                            endpoint,
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
                            "will",
                            "help",
                            if variant == "inactive" {
                                "ruby"
                            } else {
                                "dylan"
                            },
                            subject,
                        ),
                        fact("clara", "did", "visit", "bruno", subject),
                    ];
                    let records = std::array::from_fn(|s| records0[(s + rotation) % 4].clone());
                    let prompt = [
                        question("alice", "will", "call", subject),
                        question(
                            if subject { "them" } else { "they" },
                            "did",
                            "help",
                            subject,
                        ),
                    ]
                    .join(&b' ');
                    let expected = oracle(&records, &prompt)?;
                    if expected.outcome != completion::Outcome::Answered
                        || expected.selected[0].value != endpoint.as_bytes()
                    {
                        return Err("dual-role oracle mismatch".into());
                    }
                    let family = format!(
                        "{}-subject{}-rotation{rotation}",
                        endpoint.replace(' ', "_"),
                        usize::from(subject)
                    );
                    rows.push(Example {
                        id: format!("{family}-{variant}"),
                        family,
                        variant: variant.into(),
                        records,
                        prompt,
                        expected,
                    });
                }
            }
        }
    }
    for family in rows.chunks_exact(3) {
        for changed in &family[1..] {
            if family[0].prompt != changed.prompt
                || family[0]
                    .records
                    .iter()
                    .zip(&changed.records)
                    .filter(|(a, b)| a != b)
                    .count()
                    != 1
            {
                return Err("dual-role intervention changed more than one record".into());
            }
        }
    }
    Ok(rows)
}
fn auxiliary_recombinations(
    prefix: &str,
    answer: &str,
    family: &str,
) -> Result<Vec<span_data::Example>> {
    let mut rows = vec![];
    for known in [
        "alice", "bruno", "clara", "dylan", "felix", "helen", "oscar", "ruby", "amber", "cedar",
    ] {
        for relation in ["call", "help", "visit"] {
            for rotation in 0..4 {
                let records0 = [
                    fact(known, "will", relation, answer, false),
                    fact("birch", "did", "call", "ruby", false),
                    fact("birch", "did", "help", "cedar", false),
                    fact("birch", "did", "visit", "helen", false),
                ];
                let records = std::array::from_fn(|s| records0[(s + rotation) % 4].clone());
                let prompt = question(known, "will", relation, false);
                let expected = oracle(&records, &prompt)?;
                if expected.selected.len() != 1 || expected.selected[0].value != answer.as_bytes() {
                    return Err("auxiliary recombination raw oracle mismatch".into());
                }
                rows.push(span_data::Example {
                    id: format!("{prefix}-{}", rows.len()),
                    family: family.into(),
                    depth: 1,
                    variant: "direct-auxiliary-recombination".into(),
                    records,
                    prompt,
                    answer: answer.as_bytes().to_vec(),
                    expected_path: vec![],
                    expected_span: [0; 3],
                    expected_bounds: [0; 2],
                });
            }
        }
    }
    Ok(rows)
}
fn training() -> Result<Vec<span_data::Example>> {
    let mut rows = vec![];
    for subject in [false, true] {
        for rotation in 0..4 {
            for (auxiliary, answer) in [
                ("did", "will"),
                ("did", "will amber"),
                ("did", "amber will"),
                ("did", "will amber cedar"),
                ("did", "amber will cedar"),
                ("did", "amber cedar will"),
                ("did", "bruno"),
                ("will", "helen"),
            ] {
                for relation in ["call", "help", "visit"] {
                    for known in ["clara", "dylan"] {
                        let records0 = [
                            fact(known, auxiliary, relation, answer, subject),
                            fact("felix", "did", "call", "ruby", subject),
                            fact("alice", "will", "help", "oscar", subject),
                            fact("amber", "did", "visit", "cedar", subject),
                        ];
                        let records = std::array::from_fn(|s| records0[(s + rotation) % 4].clone());
                        let prompt = question(known, auxiliary, relation, subject);
                        let expected = oracle(&records, &prompt)?;
                        if expected.selected.len() != 1
                            || expected.selected[0].value != answer.as_bytes()
                        {
                            return Err("training raw oracle mismatch".into());
                        }
                        rows.push(span_data::Example {
                            id: format!("training-{}", rows.len()),
                            family: "direct-role".into(),
                            depth: 1,
                            variant: "direct".into(),
                            records,
                            prompt,
                            answer: answer.as_bytes().to_vec(),
                            expected_path: vec![],
                            expected_span: [0; 3],
                            expected_bounds: [0; 2],
                        });
                    }
                }
            }
        }
    }
    rows.extend(auxiliary_recombinations(
        "source-auxiliary",
        "violet",
        "direct-role",
    )?);
    Ok(rows)
}
fn query_training() -> Result<Vec<span_data::Example>> {
    let mut rows = vec![];
    for subject in [false, true] {
        for rotation in 0..4 {
            for (auxiliary, known) in [
                ("did", "will"),
                ("did", "will amber"),
                ("did", "ruby will amber"),
                ("will", "clara"),
            ] {
                for relation in ["call", "help", "visit"] {
                    for answer in ["bruno", "oscar"] {
                        let records0 = [
                            fact(known, auxiliary, relation, answer, subject),
                            fact("alice", "will", "call", "ruby", subject),
                            fact("dylan", "did", "help", "cedar", subject),
                            fact("amber", "did", "visit", "helen", subject),
                        ];
                        let records = std::array::from_fn(|s| records0[(s + rotation) % 4].clone());
                        let prompt = question(known, auxiliary, relation, subject);
                        let expected = oracle(&records, &prompt)?;
                        if expected.selected.len() != 1
                            || expected.selected[0].value != answer.as_bytes()
                        {
                            return Err("query training raw oracle mismatch".into());
                        }
                        rows.push(span_data::Example {
                            id: format!("query-training-{}", rows.len()),
                            family: "direct-query-role".into(),
                            depth: 1,
                            variant: "direct".into(),
                            records,
                            prompt,
                            answer: answer.as_bytes().to_vec(),
                            expected_path: vec![],
                            expected_span: [0; 3],
                            expected_bounds: [0; 2],
                        });
                    }
                }
            }
        }
    }
    rows.extend(auxiliary_recombinations(
        "query-auxiliary",
        "cobalt",
        "direct-query-role",
    )?);
    Ok(rows)
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn load_rows(evidence: &Path, relative: &str) -> Result<(Vec<Example>, Value)> {
    let root = evidence.join(relative);
    report_output::verify(&root)?;
    let bytes = std::fs::read(root.join("data.json"))?;
    let value: Value = serde_json::from_slice(&bytes)?;
    Ok((
        serde_json::from_value(value["development"].clone())?,
        json!({"root":root,"data_sha256":sha(&bytes)}),
    ))
}
fn assess(
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
    let observed = completion::observe_routed(
        &a.parent.parent.parent,
        g,
        m,
        &e.records,
        &clauses,
        &first.before,
        completion::Control::Full,
        PayloadWindow::Phrase,
        &|b: &[u8], _| Ok(b.to_vec()),
        &mut |q: &[u8], _| model::route(a, g, m, &e.records, q, c),
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
    let next_query_roles = model::query_context(a, g, m, &next.bytes, false)?;
    let next_correct = next == &expected_queries[1];
    let next_route = model::route(a, g, m, &e.records, &next.bytes, c)?;
    let actual_sources: Vec<_> = next_route.compatible.iter().map(|s| s[0]).collect();
    let sources_correct = actual_sources == e.expected.matching_sources[1];
    Ok(
        json!({"correct":outcome&&path&&continuity&&initial_correct&&next_correct&&sources_correct,"initial_selection_correct":initial_correct,"lookahead_query_geometry_correct":next_correct,"exact_compatible_sources_correct":sources_correct,"actual_initial_selection":initial,"actual_lookahead_route":next_route,"learned_context_of_actual_next_query":next_query_roles,"outcome_correct":outcome,"committed_path_queries_source_span_bounds_correct":path,"continuity":continuity,"actual_selected":selected,"generated":out}),
    )
}
fn retention(a: &model::Artifact, g: &BoundGeometry, m: &Metric, evidence: &Path) -> Result<Value> {
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
            .ok_or("retained development missing")?;
        if rows.len() != required {
            return Err("retention panel size differs".into());
        }
        let mut items = vec![];
        let mut panel_equal = 0;
        for row in rows {
            let records: [Vec<u8>; 4] = serde_json::from_value(row["records"].clone())?;
            let prompt: Vec<u8> = serde_json::from_value(row["prompt"].clone())?;
            let old = correspondence::generate(
                &a.parent,
                g,
                m,
                &records,
                &prompt,
                correspondence::Control::Full,
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
        json!({"rows":total,"equal_rows":equal,"typed_unresolved":unresolved,"equal":total==6688&&equal==6688&&unresolved==512,"panels":panels,"old_adapter_control_replay":"NOT_RUN; Full current-parent trace retention only"}),
    )
}
#[test]
fn occurrence_role_raw_corpus_has_mixed_occurrences_and_independent_training() -> Result<()> {
    let train = training()?;
    let query_train = query_training()?;
    let rows = development()?;
    assert_eq!(train.len(), 504);
    assert_eq!(query_train.len(), 312);
    assert_eq!(rows.len(), 72);
    let train_inputs: BTreeSet<_> = train
        .iter()
        .chain(&query_train)
        .map(|e| (&e.records, &e.prompt))
        .collect();
    assert_eq!(train_inputs.len(), train.len() + query_train.len());
    assert!(rows
        .iter()
        .all(|e| !train_inputs.contains(&(&e.records, &e.prompt))));
    assert!(rows.iter().all(|e| e.expected.selected.len() == 2));
    Ok(())
}
#[test]
#[ignore = "bounded learned occurrence role fit and actual generation qualification"]
fn learned_occurrence_roles() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_ROLE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_ROLE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty role paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let train = training()?;
        let query_train = query_training()?;
        let new = development()?;
        let (lexical, lexical_receipt) = load_rows(&evidence, "lexical-role-1/attempt-1")?;
        let (phrase, phrase_receipt) = load_rows(&evidence, "phrase-order-1/attempt-1")?;
        if lexical.len() != 48 || phrase.len() != 80 {
            return Err("prior role/order row count".into());
        }
        let train_inputs: BTreeSet<_> = train
            .iter()
            .chain(&query_train)
            .map(|e| (&e.records, &e.prompt))
            .collect();
        if train_inputs.len() != train.len() + query_train.len()
            || new
                .iter()
                .chain(&lexical)
                .chain(&phrase)
                .any(|e| train_inputs.contains(&(&e.records, &e.prompt)))
        {
            return Err("training/development duplicate".into());
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD_AND_FIT","Full":{"dual_role":72,"prior_lexical_role":48,"prior_phrase_order":80,"old_Full_trace_rows":6688,"correspondence_Full_trace_rows":384,"old_typed_unresolved":512},"training":{"source_rows":504,"query_rows":312,"original_source_rows_unchanged":384,"original_query_rows_unchanged":192,"appended_auxiliary_recombinations_each":120,"fits":1,"source_fits":0,"query_fits":0,"coverage_fits":1,"parameter_reuse":"sealed occurrence-role-1/attempt-2 source and query tables and anchors unchanged; only false/true available-query-coverage selection","supervision":"direct final answer/EOS only; occurrence alternatives remain latent","intermediate_metadata":"empty and zero, never training truth"},"controls":{"ExactIdentity":"all200 generated structures identical to Full","reload":"all200 generated structures identical","ReadDisabled":"no correct answered response","UpdateDisabled":"no correct dependent answered response","RoleDisabled":"lose recovered will name cases; ordinary prior successful paths remain","QueryRolesDisabled":"diagnostic: learned query role alignment disabled with coverage retained; no fixed success criterion","CoverageDisabled":"all200 complete generated structures equal frozen schema2 candidate Full"},"final_holdout":"NOT_RUN","promotion":false,"scope":"authored direct-training and dependent-development role reuse; no general prose claim"}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":new,"training":train,"query_training":query_train,"lexical_role":lexical,"phrase_order":phrase,"lineage":[lexical_receipt,phrase_receipt],"input_overlap":false,"oracle":"raw auxiliary-followed-by-authored-relation grammar, never model candidate-derived"}),
        )?;
        let prior = evidence.join("correspondence-runs-1/attempt-2");
        report_output::verify(&prior)?;
        let parent_bytes = std::fs::read(prior.join("candidate.json"))?;
        if sha(&parent_bytes) != "1838cc12bda863c769db9239d7b225e91dd82856c7e90bf7427abffb5a9e1b23"
        {
            return Err("parent identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let previous_root = evidence.join("occurrence-role-1/attempt-2");
        report_output::verify(&previous_root)?;
        let previous_bytes = std::fs::read(previous_root.join("candidate.json"))?;
        let previous_summary: Value =
            serde_json::from_slice(&std::fs::read(previous_root.join("summary.json"))?)?;
        let previous_sha256 = sha(&previous_bytes);
        if previous_summary["candidate_sha256"] != previous_sha256 {
            return Err("sealed occurrence role candidate identity".into());
        }
        let previous_data_bytes = std::fs::read(previous_root.join("data.json"))?;
        let previous_data: Value = serde_json::from_slice(&previous_data_bytes)?;
        if previous_data["training"] != serde_json::to_value(&train)?
            || previous_data["query_training"] != serde_json::to_value(&query_train)?
        {
            return Err("source or query training changed from sealed attempt2".into());
        }
        let previous = model::Artifact::decode(&previous_bytes, &g)?;
        if previous.schema != 2 || previous.parent.encode()? != parent_bytes {
            return Err("schema2 role parent changed".into());
        }
        let source_fit_table = previous.table.clone();
        let query_fit_table = previous.query_table.clone();
        let previous_anchors = previous.anchors.clone();
        let combined_train: Vec<_> = train.iter().chain(&query_train).cloned().collect();
        let (a, coverage_fit) = model::fit_coverage(previous.clone(), &g, &m, &combined_train)?;
        let prior_fit_bytes = std::fs::read(previous_root.join("fit.json"))?;
        let prior_fit_receipt = json!({"root":previous_root,"candidate_sha256":previous_sha256,"data_sha256":sha(&previous_data_bytes),"fit_sha256":sha(&prior_fit_bytes),"source_and_query_training_identical":true});
        a.validate(&g)?;
        let bytes = a.encode()?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        write(
            &output,
            "fit.json",
            &json!({"coverage":coverage_fit,"fits":1,"source_fits":0,"query_fits":0,"coverage_fits":1,"prior_source_query_fit_receipt":prior_fit_receipt,"source_table_before_coverage_fit":source_fit_table,"query_table_before_coverage_fit":query_fit_table}),
        )?;
        let reload = model::Artifact::decode(&bytes, &g)?;
        let mut invalid_query = a.clone();
        invalid_query.query_table = vec![model::Row {
            key: model::Key {
                center: 255,
                left: 65,
                right: 65,
            },
            context: false,
        }];
        let invalid_query_key_rejected =
            model::Artifact::decode(&invalid_query.encode()?, &g).is_err();
        write(
            &output,
            "serialization.json",
            &json!({"roundtrip_equal":reload==a,"out_of_inventory_query_center_rejected":invalid_query_key_rejected,"source_fit_table_unchanged":a.table==source_fit_table,"query_fit_table_unchanged":a.query_table==query_fit_table,"anchors_unchanged":a.anchors==previous_anchors}),
        )?;
        let mut training_correct = 0;
        let mut query_training_correct = 0;
        let mut training_responses = vec![];
        for e in train.iter().chain(&query_train) {
            let out = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            let correct = out.outcome == completion::Outcome::Answered
                && !out.trace.exhausted
                && out.trace.tokens == super::span_learning::target(e);
            if e.family == "direct-query-role" {
                query_training_correct += usize::from(correct);
            } else {
                training_correct += usize::from(correct);
            }
            training_responses.push(json!({"id":e.id,"correct":correct,"generated":out}));
        }
        write(
            &output,
            "training-responses.json",
            &json!(training_responses),
        )?;
        let mut role_observations = vec![];
        for e in &new {
            let source = e.expected.selected[0].source;
            let record = &e.records[source];
            role_observations.push(json!({"id":e.id,"source":source,"record":record,"expected_payload":e.expected.selected[0],"parent_context":correspondence::source_context(&a.parent,&g,&m,record,false)?,"learned_context":model::source_context(&a,&g,&m,record,false)?,"exact_context":model::source_context(&a,&g,&m,record,true)?}));
        }
        write(&output, "role-observations.json", &json!(role_observations))?;
        let mut responses = vec![];
        let mut counts = BTreeMap::<String, usize>::new();
        let mut exact_equal = 0;
        let mut reload_equal = 0;
        let mut phrase_parent_equal = 0;
        let mut role_parent_equal = 0;
        let mut coverage_previous_equal = 0;
        let mut role_reused_correct = 0;
        let mut control_answer_correct = 0;
        for (label, rows) in [
            ("dual_role", &new),
            ("lexical_role", &lexical),
            ("phrase_order", &phrase),
        ] {
            for e in rows {
                let full =
                    model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
                let exact = model::generate(
                    &a,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::ExactIdentity,
                )?;
                exact_equal += usize::from(full == exact);
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
                let old = correspondence::generate(
                    &a.parent,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    correspondence::Control::Full,
                )?;
                if label == "phrase_order" {
                    phrase_parent_equal += usize::from(full == old);
                }
                let role = model::generate(
                    &a,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::RoleDisabled,
                )?;
                role_parent_equal += usize::from(role == old);
                let coverage_disabled = model::generate(
                    &a,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::CoverageDisabled,
                )?;
                let previous_full = model::generate(
                    &previous,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::Full,
                )?;
                coverage_previous_equal += usize::from(coverage_disabled == previous_full);
                for (control, c, out) in [
                    ("Full", model::Control::Full, full),
                    ("ExactIdentity", model::Control::ExactIdentity, exact),
                    ("RoleDisabled", model::Control::RoleDisabled, role),
                    (
                        "CoverageDisabled",
                        model::Control::CoverageDisabled,
                        coverage_disabled,
                    ),
                    (
                        "QueryRolesDisabled",
                        model::Control::QueryRolesDisabled,
                        model::generate(
                            &a,
                            &g,
                            &m,
                            &e.records,
                            &e.prompt,
                            model::Control::QueryRolesDisabled,
                        )?,
                    ),
                    (
                        "ReadDisabled",
                        model::Control::ReadDisabled,
                        model::generate(
                            &a,
                            &g,
                            &m,
                            &e.records,
                            &e.prompt,
                            model::Control::ReadDisabled,
                        )?,
                    ),
                    (
                        "UpdateDisabled",
                        model::Control::UpdateDisabled,
                        model::generate(
                            &a,
                            &g,
                            &m,
                            &e.records,
                            &e.prompt,
                            model::Control::UpdateDisabled,
                        )?,
                    ),
                ] {
                    let assessed = assess(&a, &g, &m, e, c, &out)?;
                    let correct = assessed["correct"] == true;
                    *counts.entry(format!("{label}/{control}")).or_default() +=
                        usize::from(correct);
                    if label == "lexical_role"
                        && e.variant.starts_with("will/")
                        && c == model::Control::RoleDisabled
                    {
                        role_reused_correct += usize::from(correct);
                    }
                    if matches!(
                        c,
                        model::Control::ReadDisabled | model::Control::UpdateDisabled
                    ) && e.expected.outcome == completion::Outcome::Answered
                    {
                        control_answer_correct += usize::from(assessed["outcome_correct"] == true);
                    }
                    responses.push(
                        json!({"panel":label,"id":e.id,"control":control,"response":assessed}),
                    );
                }
            }
        }
        write(&output, "responses.json", &json!(responses))?;
        let retained = retention(&a, &g, &m, &evidence)?;
        write(&output, "retained-responses.json", &retained)?;
        report_output::verify(&previous_root)?;
        let unchanged = a.parent.encode()? == parent_bytes
            && std::fs::read(prior.join("candidate.json"))? == parent_bytes
            && std::fs::read(previous_root.join("candidate.json"))? == previous_bytes;
        let pass = training_correct == 504
            && query_training_correct == 312
            && invalid_query_key_rejected
            && a.table == source_fit_table
            && a.query_table == query_fit_table
            && a.anchors == previous_anchors
            && coverage_previous_equal == 200
            && counts["dual_role/Full"] == 72
            && counts["lexical_role/Full"] == 48
            && counts["phrase_order/Full"] == 80
            && exact_equal == 200
            && reload_equal == 200
            && phrase_parent_equal == 80
            && role_parent_equal == 200
            && role_reused_correct == 0
            && control_answer_correct == 0
            && retained["equal"] == true
            && unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_LEARNED_OCCURRENCE_ROLE"}else{"FAIL_LEARNED_OCCURRENCE_ROLE"},"panels":counts,"training_correct":training_correct,"training_rows":504,"query_training_correct":query_training_correct,"query_training_rows":312,"invalid_query_key_rejected":invalid_query_key_rejected,"source_fit_table_unchanged":a.table==source_fit_table,"query_fit_table_unchanged":a.query_table==query_fit_table,"anchors_unchanged":a.anchors==previous_anchors,"coverage_disabled_previous_full_equal":coverage_previous_equal,"require_available_query_coverage":a.require_available_query_coverage,"prior_fit_receipt":prior_fit_receipt,"exact_equal":exact_equal,"reload_equal":reload_equal,"phrase_order_parent_full_trace_equal":phrase_parent_equal,"role_disabled_parent_full_trace_equal":role_parent_equal,"role_disabled_reused_name_correct":role_reused_correct,"read_or_update_disabled_correct_answer":control_answer_correct,"retained_full_trace_equal":retained["equal_rows"],"retained_typed_unresolved":retained["typed_unresolved"],"prior_correspondence_full_trace_equal":retained["panels"].as_array().and_then(|ps|ps.iter().find(|p|p["source"]=="correspondence-runs-1")).map(|p|p["equal"].clone()),"table":a.table,"query_table":a.query_table,"candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"parent_parameters_unchanged":unchanged,"fits":1,"source_fits":0,"query_fits":0,"coverage_fits":1,"final_holdout":"NOT_RUN","promotion":false,"default_model":"15baec48 unchanged"}),
        )?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"gate":if pass{"PASS_LEARNED_OCCURRENCE_ROLE"}else{"FAIL_LEARNED_OCCURRENCE_ROLE"},"path":output})
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
