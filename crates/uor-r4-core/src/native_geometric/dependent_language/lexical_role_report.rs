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
    for subject in [false, true] {
        for rotation in 0..4 {
            for name in ["bruno", "will"] {
                for variant in ["baseline", "active", "inactive"] {
                    let records0 = [
                        fact("alice", "call", name, subject),
                        fact(
                            name,
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
                            "help",
                            if variant == "inactive" {
                                "ruby"
                            } else {
                                "dylan"
                            },
                            subject,
                        ),
                        fact("clara", "visit", "amber", subject),
                    ];
                    let records =
                        std::array::from_fn(|slot| records0[(slot + rotation) % 4].clone());
                    let prompt = [
                        question("alice", "call", subject),
                        question(if subject { "them" } else { "they" }, "help", subject),
                    ]
                    .join(&b' ');
                    let expected = oracle(&records, &prompt)?;
                    let want = if variant == "active" {
                        b"oscar".as_slice()
                    } else {
                        b"helen".as_slice()
                    };
                    if expected.outcome != completion::Outcome::Answered
                        || expected.selected[0].value != name.as_bytes()
                        || expected.selected[1].value != want
                    {
                        return Err("raw role oracle mismatch".into());
                    }
                    let family = format!("subject{}-rotation{rotation}", usize::from(subject));
                    rows.push(Example {
                        id: format!("{family}-{name}-{variant}"),
                        family,
                        variant: format!("{name}/{variant}"),
                        records,
                        prompt,
                        expected,
                    });
                }
            }
        }
    }
    if rows.len() != 48
        || rows
            .iter()
            .map(|e| (&e.records, &e.prompt))
            .collect::<BTreeSet<_>>()
            .len()
            != 48
    {
        return Err("role corpus count/duplicates".into());
    }
    for family in rows.chunks_exact(6) {
        for group in [&family[..3], &family[3..]] {
            for e in &group[1..] {
                if group[0].prompt != e.prompt
                    || group[0]
                        .records
                        .iter()
                        .zip(&e.records)
                        .filter(|(a, b)| a != b)
                        .count()
                        != 1
                {
                    return Err("answer/distractor change scope".into());
                }
            }
        }
        for (base, reused) in family[..3].iter().zip(&family[3..]) {
            if base.prompt != reused.prompt
                || base
                    .records
                    .iter()
                    .zip(&reused.records)
                    .filter(|(a, b)| a != b)
                    .count()
                    != 2
            {
                return Err("consistent identity substitution scope".into());
            }
            for (a, b) in base.records.iter().zip(&reused.records) {
                if String::from_utf8(a.clone())?
                    .replace("bruno", "will")
                    .as_bytes()
                    != b
                {
                    return Err("nonidentity difference".into());
                }
            }
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
fn localize(a: &model::Artifact, g: &BoundGeometry, m: &Metric, e: &Example) -> Result<Value> {
    let q = &e.expected.queries[0]; // The first prompt clause is actual input, never synthesized feedback.
    let intended = &e.expected.selected[0];
    let search = model::candidates(a, g, m, &e.records, q, model::Control::Full)?;
    let target = search.candidates.iter().find(|c| {
        c.span.value.source == intended.source
            && [c.span.value.start, c.span.value.end] == intended.bounds
    });
    let context = model::source_context(a, g, m, &e.records[intended.source], false)?;
    let query_context = model::query_context(a, g, m, q, false)?;
    let mut witnesses = vec![];
    if let Some(target) = target {
        for witness in &target.witnesses {
            witnesses.push(json!({"witness":witness,"missing_required_bits":a.rules.iter().map(|rule|rule & !witness.features).collect::<Vec<_>>(),"admitted":a.matches(witness.features),"run":model::run_signature(&witness.query_to_source,&query_context,&context)?}));
        }
    }
    let direct = model::generate(a, g, m, &e.records, q, model::Control::Full)?;
    let direct_expected = intended
        .value
        .iter()
        .map(|b| u16::from(*b))
        .chain([256])
        .collect::<Vec<_>>();
    let direct_correct = direct.outcome == completion::Outcome::Answered
        && direct.trace.tokens == direct_expected
        && !direct.trace.exhausted;
    // These are explicitly supplied-payload diagnostics, not actual generated continuation.
    let clauses = scheduling::clauses(&e.prompt)?;
    let updater = &a.parent.parent.parent.parent;
    let updates = super::runtime::updates_with_window(
        updater,
        g,
        m,
        &e.records,
        &clauses[1],
        &intended.value,
        super::runtime::Control::Full,
        PayloadWindow::Phrase,
    )?;
    let admitted: Vec<_> = updates
        .iter()
        .filter(|u| updater.matches(u.features))
        .collect();
    let supplied_update_correct =
        admitted.len() == 1 && admitted[0].question == e.expected.queries[1];
    let counterfactual = model::route(
        a,
        g,
        m,
        &e.records,
        &e.expected.queries[1],
        model::Control::Full,
    )?;
    let expected = &e.expected.selected[1];
    let counterfactual_route_correct = counterfactual.selected.as_ref().is_some_and(|s| {
        s.source == expected.source
            && [s.start, s.end] == expected.bounds
            && s.bytes == expected.value
    });
    Ok(
        json!({"actual_first_query":q,"intended_target_span":intended,"target_enumerated":target.is_some(),"target_witness_count":witnesses.len(),"target_source_context":context,"query_context":query_context,"target_witnesses":witnesses,"search_nodes":search.nodes,"direct_first_clause_generated":direct,"direct_first_clause_correct":direct_correct,"supplied_payload_probe":{"scope":"COUNTERFACTUAL: independent oracle payload supplied after actual first read may have failed; no native generation credit","payload":intended.value,"update_candidates":updates,"admitted_update_count":admitted.len(),"unique_correct_update":supplied_update_correct,"oracle_expanded_query":e.expected.queries[1],"route_on_oracle_query":counterfactual,"correct_downstream_route":counterfactual_route_correct}}),
    )
}
#[test]
fn role_oracle_preserves_consistent_identity_and_answer_interventions() -> Result<()> {
    let rows = corpus()?;
    assert_eq!(rows.len(), 48);
    assert_eq!(
        rows.iter()
            .filter(|e| e.variant.starts_with("will/"))
            .count(),
        24
    );
    Ok(())
}
#[test]
#[ignore = "bounded unchanged-artifact lexical role diagnostic"]
fn lexical_role_reuse() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_LEXICAL_ROLE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LEXICAL_ROLE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty diagnostic paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let rows = corpus()?;
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","capability_gate":"PASS only if Full48/48,24ordinary and24reused-name,correct actual firstpayload and nextquery/trace/EOS,48exact parity/reload,read/update controls lose all correct answers","diagnosis":"On failure report enumeration,witness bits,barriers,actual direct read; supplied-payload update and oracle-query route are COUNTERFACTUAL, never generation success","identity_substitution":"two linked records consistently renamed; active/inactive each change one record","fits":0,"artifact_runtime_unchanged":true,"final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"raw_oracle":"independent exact authored relation protocol; answer strings from raw records, no geometric candidate/gold model input","rows":48}),
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
        let roles = model::source_context(&a, &g, &m, b"bruno will", false)?;
        if roles != [false, true] {
            return Err("frozen lexical roles differ".into());
        }
        write(
            &output,
            "artifact.json",
            &json!({"source":prior,"sha256":sha(&bytes),"rules":a.rules,"roles":a.parent.context_words,"bruno_will_context":roles,"runtime_artifact_unchanged":true}),
        )?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut responses = vec![];
        let mut localizations = vec![];
        let mut exact_equal = 0;
        let mut reload_equal = 0;
        for e in &rows {
            let full = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
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
            let diagnostic = localize(&a, &g, &m, e)?;
            localizations.push(json!({"id":e.id,"variant":e.variant,"diagnostic":diagnostic}));
            for (label, c, out) in [
                ("Full", model::Control::Full, full),
                ("ExactIdentity", model::Control::ExactIdentity, exact),
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
                for key in [
                    label.to_string(),
                    format!("{label}/{}", e.variant),
                    format!(
                        "{label}/{}",
                        if e.variant.starts_with("will/") {
                            "reused_name"
                        } else {
                            "ordinary_name"
                        }
                    ),
                ] {
                    *counts.entry(key).or_default() += usize::from(yes);
                }
                *counts.entry(format!("{label}/correct_answer")).or_default() +=
                    usize::from(response["outcome_correct"] == true);
                responses.push(json!({"id":e.id,"control":label,"response":response}));
            }
        }
        write(&output, "responses.json", &json!(responses))?;
        write(&output, "localization.json", &json!(localizations))?;
        let mut diagnosis = BTreeMap::<String, Value>::new();
        for name in ["bruno", "will"] {
            let selected: Vec<_> = localizations
                .iter()
                .filter(|e| e["variant"].as_str().is_some_and(|v| v.starts_with(name)))
                .map(|e| &e["diagnostic"])
                .collect();
            let only_boundary = selected
                .iter()
                .filter(|r| {
                    r["target_witnesses"].as_array().is_some_and(|ws| {
                        !ws.is_empty()
                            && ws
                                .iter()
                                .all(|w| w["missing_required_bits"] == json!([1u32 << 18]))
                    })
                })
                .count();
            diagnosis.insert(name.into(),json!({"rows":selected.len(),"target_enumerated":selected.iter().filter(|r|r["target_enumerated"]==true).count(),"every_target_witness_missing_only_boundary_bit18":only_boundary,"direct_first_read_correct":selected.iter().filter(|r|r["direct_first_clause_correct"]==true).count(),"supplied_payload_unique_correct_update":selected.iter().filter(|r|r["supplied_payload_probe"]["unique_correct_update"]==true).count(),"oracle_query_correct_downstream_route":selected.iter().filter(|r|r["supplied_payload_probe"]["correct_downstream_route"]==true).count()}));
        }
        let unchanged =
            a.encode()? == bytes && std::fs::read(prior.join("candidate.json"))? == bytes;
        let pass = counts["Full"] == 48
            && exact_equal == 48
            && reload_equal == 48
            && counts["ReadDisabled/correct_answer"] == 0
            && counts["UpdateDisabled/correct_answer"] == 0
            && unchanged;
        let boundary_localized = counts["Full/ordinary_name"] == 24
            && counts["Full/reused_name"] == 0
            && diagnosis["will"]["every_target_witness_missing_only_boundary_bit18"] == 24;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_LEXICAL_ROLE_REUSE"}else{"FAIL_LEXICAL_ROLE_REUSE"},"diagnosis":if boundary_localized{"GLOBAL_CONTEXT_PAYLOAD_BARRIER_LOCALIZED"}else{"SEE_LOCALIZATION"},"rows":48,"counts":counts,"localization":diagnosis,"exact_identity_equal":exact_equal,"reload_equal":reload_equal,"candidate_sha256":sha(&bytes),"artifact_unchanged":unchanged,"runtime_unchanged":true,"fits":0,"counterfactuals":"not generated behavior or qualification credit","older_controls":"NOT_RERUN; runtime/artifact unchanged and prior evidence preserved","final_holdout":"NOT_RUN","promotion":false}),
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
