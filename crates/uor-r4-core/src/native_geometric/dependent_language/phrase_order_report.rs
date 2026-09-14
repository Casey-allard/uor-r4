//! Unchanged-artifact development test of exact ordered endpoints. The oracle
//! parses the authored relation protocol; it never consults model candidates.
use super::{
    completion, correspondence as model, phrase_data::intervals, runtime::PayloadWindow, scheduling,
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
    matches!(w, b"guide" | b"help")
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
    if clauses.len() != 2 || prompt.len() > 256 {
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
            || w[..2] != [b"who".as_slice(), b"did".as_slice()]
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
                .filter(|(_, w)| **w == b"did")
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
            if r[a + 1] == relation && r[endpoint[0]..endpoint[1]].join(&b' ') == known {
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
fn fact(known: &str, relation: &str, answer: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("{answer} did {relation} {known}.")
    } else {
        format!("{known} did {relation} {answer}.")
    }
    .into_bytes()
}
fn question(known: &str, relation: &str, subject: bool) -> Vec<u8> {
    if subject {
        format!("who did {relation} {known}?")
    } else {
        format!("who did {known} {relation}?")
    }
    .into_bytes()
}
fn corpus() -> Result<Vec<Example>> {
    let mut rows = vec![];
    for reverse in [false, true] {
        for subject in [false, true] {
            for rotation in 0..4 {
                let (known, other, gapped) = if reverse {
                    ("amber ruby", "ruby amber", "amber cedar ruby")
                } else {
                    ("ruby amber", "amber ruby", "ruby cedar amber")
                };
                let family = format!(
                    "order{}-subject{}-rotation{rotation}",
                    usize::from(reverse),
                    usize::from(subject)
                );
                for variant in ["baseline", "active", "inactive", "missing", "conflicting"] {
                    let logical = [
                        fact(
                            "alice",
                            "guide",
                            if variant == "active" { other } else { known },
                            subject,
                        ),
                        fact(
                            if variant == "missing" {
                                "felix birch"
                            } else {
                                known
                            },
                            "help",
                            "helen",
                            subject,
                        ),
                        fact(other, "help", "oscar", subject),
                        fact(
                            if variant == "conflicting" {
                                known
                            } else {
                                gapped
                            },
                            "help",
                            if variant == "inactive" {
                                "clara"
                            } else {
                                "bruno"
                            },
                            subject,
                        ),
                    ];
                    let records =
                        std::array::from_fn(|slot| logical[(slot + rotation) % 4].clone());
                    let prompt = [
                        question("alice", "guide", subject),
                        question(if subject { "them" } else { "they" }, "help", subject),
                    ]
                    .join(&b' ');
                    let expected = oracle(&records, &prompt)?;
                    let expected_answer = if variant == "active" {
                        b"oscar".as_slice()
                    } else {
                        b"helen".as_slice()
                    };
                    match variant {
                        "missing" => assert_eq!(
                            expected.outcome,
                            completion::Outcome::Unresolved {
                                clause: 1,
                                reason: completion::Reason::NoCompatibleCandidate
                            }
                        ),
                        "conflicting" => assert_eq!(
                            expected.outcome,
                            completion::Outcome::Unresolved {
                                clause: 1,
                                reason: completion::Reason::Ambiguous
                            }
                        ),
                        _ => {
                            assert_eq!(expected.outcome, completion::Outcome::Answered);
                            assert_eq!(
                                expected.tokens,
                                expected_answer
                                    .iter()
                                    .map(|b| u16::from(*b))
                                    .chain([256])
                                    .collect::<Vec<_>>()
                            );
                        }
                    }
                    rows.push(Example {
                        id: format!("{family}-{variant}"),
                        family: family.clone(),
                        variant: variant.into(),
                        records,
                        prompt,
                        expected,
                    });
                }
            }
        }
    }
    let unique: BTreeSet<_> = rows.iter().map(|e| (&e.records, &e.prompt)).collect();
    if rows.len() != 80 || unique.len() != 80 {
        return Err("duplicate or missing input".into());
    }
    for group in rows.chunks_exact(5) {
        for changed in &group[1..] {
            if group[0]
                .records
                .iter()
                .zip(&changed.records)
                .filter(|(a, b)| a != b)
                .count()
                != 1
                || group[0].prompt != changed.prompt
            {
                return Err("intervention changes more than one record".into());
            }
        }
        let first = &group[0].expected.selected[0].value;
        let other = &group[1].expected.selected[0].value;
        let mut a = words(first);
        let mut b = words(other);
        a.sort();
        b.sort();
        if first == other || a != b {
            return Err("active change not exact multiset permutation".into());
        }
    }
    Ok(rows)
}
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, v: &Value) -> Result<()> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn assess(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    c: model::Control,
    out: &completion::Generated,
) -> Result<Value> {
    let first = out.decisions.first().ok_or("no initial decision")?;
    let qs = scheduling::clauses(&e.prompt)?;
    let observed = completion::observe_routed(
        &a.parent.parent,
        g,
        m,
        &e.records,
        &qs,
        &first.before,
        completion::Control::Full,
        PayloadWindow::Phrase,
        &|b: &[u8], _| Ok(b.to_vec()),
        &mut |q: &[u8], _| model::route(a, g, m, &e.records, q, c),
    )?;
    let expected_first = e
        .expected
        .selected
        .first()
        .ok_or("oracle initial selection")?;
    let selected = observed.core.route.selected.as_ref();
    let first_ok = selected.is_some_and(|s| {
        s.source == expected_first.source
            && s.word == expected_first.span[0]
            && [s.start, s.end] == expected_first.bounds
            && s.bytes == expected_first.value
    });
    let next = &observed.core.core.next_query;
    let expected_next = lexical::Query::new(
        g,
        &e.expected.queries[1],
        a.reader().parent.parent.operators,
    )?;
    let next_ok = next == &expected_next;
    let next_route = model::route(a, g, m, &e.records, &next.bytes, c)?;
    let actual_sources: Vec<_> = next_route.compatible.iter().map(|v| v[0]).collect();
    let source_set_ok = actual_sources == e.expected.matching_sources[1];
    let actual: Vec<_> = out
        .trace
        .steps
        .iter()
        .filter(|s| s.before.core.cursor == 0)
        .filter_map(|s| s.observation.route.selected.as_ref())
        .collect();
    let actual_queries: Vec<_> = out
        .decisions
        .iter()
        .filter(|d| d.before.core.cursor == 0)
        .map(|d| &d.before.core.query)
        .collect();
    let expected_queries = e
        .expected
        .queries
        .iter()
        .map(|q| lexical::Query::new(g, q, a.reader().parent.parent.operators))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let answered = e.expected.outcome == completion::Outcome::Answered;
    let path_ok = if answered {
        actual.len() == e.expected.selected.len()
            && actual.iter().zip(&e.expected.selected).all(|(s, v)| {
                s.source == v.source
                    && s.word == v.span[0]
                    && [s.start, s.end] == v.bounds
                    && s.bytes == v.value
            })
            && actual_queries == expected_queries.iter().collect::<Vec<_>>()
    } else {
        out.trace.steps.is_empty()
            && out.decisions.len() == 1
            && actual_queries == vec![&expected_queries[0]]
    };
    let outcome_ok = out.outcome == e.expected.outcome
        && out.trace.tokens == e.expected.tokens
        && !out.trace.exhausted;
    let continuity = out
        .trace
        .steps
        .windows(2)
        .all(|s| s[0].after == s[1].before);
    Ok(
        json!({"correct":first_ok&&next_ok&&path_ok&&outcome_ok&&continuity&&source_set_ok,"outcome_correct":outcome_ok,"initial_selection_correct":first_ok,"lookahead_query_and_encoding_correct":next_ok,"committed_path_and_queries_correct":path_ok,"state_continuity":continuity,"actual_lookahead_route":next_route,"exact_endpoint_sources_correct":source_set_ok,"actual_initial_selection":selected,"actual_lookahead_query":next,"committed_steps":out.trace.steps.len(),"generated":out}),
    )
}
fn witnesses(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    queries: &[Vec<u8>],
) -> Result<(Value, usize)> {
    let mut stages = vec![];
    let mut max = 0;
    for q in queries {
        let context = model::query_context(a, g, m, q, false)?;
        let search = model::candidates(a, g, m, &e.records, q, model::Control::Full)?;
        max = max.max(search.nodes);
        let mut rows = vec![];
        for candidate in &search.candidates {
            let source_context =
                model::source_context(a, g, m, &e.records[candidate.span.value.source], false)?;
            for witness in &candidate.witnesses {
                rows.push(json!({"candidate":candidate.span,"assignment":witness,"run":model::run_signature(&witness.query_to_source,&context,&source_context)?,"admitted":a.matches(witness.features)}));
            }
        }
        stages.push(
            json!({"query":q,"search_nodes":search.nodes,"query_context":context,"witnesses":rows}),
        );
    }
    Ok((json!(stages), max))
}
#[test]
fn raw_oracle_distinguishes_order_gaps_and_conflicts() -> Result<()> {
    let rows = corpus()?;
    assert_eq!(
        rows.iter()
            .filter(|e| e.expected.outcome == completion::Outcome::Answered)
            .count(),
        48
    );
    for e in rows.iter().filter(|e| e.variant == "missing") {
        assert_eq!(e.expected.matching_sources[1].len(), 0);
    }
    for e in rows.iter().filter(|e| e.variant == "conflicting") {
        assert_eq!(e.expected.matching_sources[1].len(), 2);
    }
    Ok(())
}
#[test]
#[ignore = "bounded unchanged-artifact phrase-order diagnostic"]
fn phrase_order_identity() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_PHRASE_ORDER_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_PHRASE_ORDER_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let rows = corpus()?;
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","Full":80,"answered":48,"missing":16,"ambiguous":16,"complete_families":16,"Full_ExactIdentity_equal":80,"reload_equal":80,"StructureDisabled":"fewer than80 complete cases; removal disables order and adjacency jointly","ReadDisabled_correct_answers":0,"UpdateDisabled_correct_answers":0,"initial_payload_and_actual_lookahead":"all80 including unresolved before read commit","fits":0,"artifact_unchanged":true,"final_holdout":"NOT_RUN","scope":"familiar authored grammar; new competing reversed endpoints and content gaps; no metric advantage or general prose"}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"input_unique":80,"single_record_interventions":true,"active_change_same_word_multiset":true,"oracle":"raw full endpoint equality with separately typed missing/ambiguous; independent of routing/geometry"}),
        )?;
        let prior = evidence.join("correspondence-runs-1/attempt-2");
        report_output::verify(&prior)?;
        let bytes = std::fs::read(prior.join("candidate.json"))?;
        if sha(&bytes) != "1838cc12bda863c769db9239d7b225e91dd82856c7e90bf7427abffb5a9e1b23" {
            return Err("candidate identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = model::Artifact::decode(&bytes, &g)?;
        let reload = model::Artifact::decode(&a.encode()?, &g)?;
        let content_context = model::source_context(&a, &g, &m, b"ruby amber cedar", false)?;
        if content_context != [false, false, false] {
            return Err("content gap classified as learned context".into());
        }
        write(
            &output,
            "artifact.json",
            &json!({"source":prior,"sha256":sha(&bytes),"rules":a.rules,"schema":a.schema,"context_roles":a.parent.context_words.len(),"gap_words_context_roles":content_context,"artifact_and_runtime_unchanged":true}),
        )?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut families: BTreeMap<String, usize> = BTreeMap::new();
        let mut responses = vec![];
        let mut diagnostics = vec![];
        let mut exact_equal = 0;
        let mut reload_equal = 0;
        let mut max_nodes = 0;
        for e in &rows {
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
            let exact = model::generate(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::ExactIdentity,
            )?;
            exact_equal += usize::from(full == exact);
            // Diagnostics use actual observed queries, including uncommitted lookahead.
            let full_assess = assess(&a, &g, &m, e, model::Control::Full, &full)?;
            let actual_next: lexical::Query =
                serde_json::from_value(full_assess["actual_lookahead_query"].clone())?;
            let actual_queries = vec![
                full.decisions[0].before.core.query.bytes.clone(),
                actual_next.bytes,
            ];
            let (trace, nodes) = witnesses(&a, &g, &m, e, &actual_queries)?;
            max_nodes = max_nodes.max(nodes);
            diagnostics.push(json!({"id":e.id,"actual_query_witnesses":trace}));
            for (label, c, out) in [
                ("Full", model::Control::Full, full),
                ("ExactIdentity", model::Control::ExactIdentity, exact),
                (
                    "StructureDisabled",
                    model::Control::StructureDisabled,
                    model::generate(
                        &a,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::StructureDisabled,
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
                let response = assess(&a, &g, &m, e, c, &out)?;
                let yes = response["correct"] == true;
                *counts.entry(label.into()).or_default() += usize::from(yes);
                *counts.entry(format!("{label}/{}", e.variant)).or_default() += usize::from(yes);
                *counts
                    .entry(format!("{label}/answered_correct"))
                    .or_default() += usize::from(
                    response["outcome_correct"] == true
                        && e.expected.outcome == completion::Outcome::Answered,
                );
                if label == "Full" && yes {
                    *families.entry(e.family.clone()).or_default() += 1;
                }
                responses.push(json!({"id":e.id,"control":label,"response":response}));
            }
        }
        write(&output, "responses.json", &json!(responses))?;
        write(&output, "witnesses.json", &json!(diagnostics))?;
        let complete = families.values().filter(|n| **n == 5).count();
        let unchanged =
            std::fs::read(prior.join("candidate.json"))? == bytes && a.encode()? == bytes;
        let pass = counts["Full"] == 80
            && complete == 16
            && exact_equal == 80
            && reload_equal == 80
            && counts["StructureDisabled"] < 80
            && counts["ReadDisabled/answered_correct"] == 0
            && counts["UpdateDisabled/answered_correct"] == 0
            && unchanged
            && max_nodes <= model::MAX_SEARCH_NODES;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_PHRASE_ORDER_IDENTITY"}else{"FAIL_PHRASE_ORDER_IDENTITY"},"rows":80,"counts":counts,"complete_families":complete,"exact_identity_equal":exact_equal,"reload_equal":reload_equal,"candidate_sha256":sha(&bytes),"artifact_unchanged":unchanged,"runtime_unchanged":true,"fits":0,"max_search_nodes":max_nodes,"route_limit":model::MAX_SEARCH_NODES,"prior_controls":"NOT_RERUN; unchanged runtime/artifact verified against prior tested source","final_holdout":"NOT_RUN","promotion":false}),
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
