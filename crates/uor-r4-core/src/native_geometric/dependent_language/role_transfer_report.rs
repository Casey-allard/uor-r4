//! Unchanged-artifact role-transfer diagnostic. The raw authored grammar is
//! evaluation-only and never supplied to routing, fitting, or inference.
use super::{
    completion, occurrence_role as model, phrase_data::intervals, runtime::PayloadWindow,
    scheduling,
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
    matches!(w, b"call" | b"help" | b"visit" | b"trust")
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
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
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
        json!({"correct":outcome&&path&&continuity&&initial_correct&&next_correct&&sources_correct,"initial_selection_correct":initial_correct,"lookahead_query_geometry_correct":next_correct,"exact_compatible_sources_correct":sources_correct,"actual_initial_selection":initial,"actual_lookahead_route":next_route,"learned_context_of_actual_next_query":next_query_roles,"outcome_correct":outcome,"committed_path_queries_source_span_bounds_correct":path,"continuity":continuity,"actual_selected":selected}),
    )
}
fn push(
    rows: &mut Vec<Example>,
    family: String,
    variant: &str,
    records0: [Vec<u8>; 4],
    prompt: Vec<u8>,
    rotation: usize,
) -> Result<()> {
    let records = std::array::from_fn(|s| records0[(s + rotation) % 4].clone());
    let expected = oracle(&records, &prompt)?;
    if expected.selected.is_empty() || expected.queries.len() != 2 {
        return Err("transfer corpus must reach an oracle second clause".into());
    }
    rows.push(Example {
        id: format!("{family}-{variant}"),
        family,
        variant: variant.into(),
        records,
        prompt,
        expected,
    });
    Ok(())
}
fn styled(known: &str, auxiliary: &str, relation: &str, answer: &str, subject: bool) -> Vec<u8> {
    let raw = fact(known, auxiliary, relation, answer, subject);
    [b"today ".as_slice(), &raw[..raw.len() - 1], b" tomorrow."].concat()
}
fn dependent_prompt(auxiliary: &str, first_relation: &str, subject: bool, prefix: bool) -> Vec<u8> {
    let first = question("alice", auxiliary, first_relation, subject);
    let second = question(
        if subject { "them" } else { "they" },
        "did",
        "help",
        subject,
    );
    if prefix {
        [
            b"please tell me ".as_slice(),
            &first,
            b" please tell me ",
            &second,
        ]
        .concat()
    } else {
        [first, second].join(&b' ')
    }
}
fn development() -> Result<Vec<Example>> {
    let mut rows = vec![];
    for outcome in ["valid", "missing", "conflicting"] {
        for rotation in 0..4 {
            let family = format!("coverage-{outcome}-rotation{rotation}");
            for irrelevant in ["clara", "please", "tell", "me", "who"] {
                push(
                    &mut rows,
                    family.clone(),
                    if irrelevant == "clara" {
                        "baseline"
                    } else {
                        irrelevant
                    },
                    [
                        fact("alice", "did", "call", "bruno", false),
                        fact(
                            if outcome == "missing" {
                                "felix"
                            } else {
                                "bruno"
                            },
                            "did",
                            "help",
                            "helen",
                            false,
                        ),
                        fact(
                            if outcome == "conflicting" {
                                "bruno"
                            } else {
                                "dylan"
                            },
                            "did",
                            "help",
                            "oscar",
                            false,
                        ),
                        fact(irrelevant, "did", "visit", "amber", false),
                    ],
                    dependent_prompt("did", "call", false, true),
                    rotation,
                )?;
            }
        }
    }
    for endpoint in ["bruno", "will", "will amber", "amber will"] {
        for subject in [false, true] {
            for rotation in 0..4 {
                let family = format!(
                    "styled-{}-subject{}-rotation{rotation}",
                    endpoint.replace(' ', "_"),
                    usize::from(subject)
                );
                for variant in ["baseline", "active", "inactive"] {
                    push(
                        &mut rows,
                        family.clone(),
                        variant,
                        [
                            styled("alice", "will", "call", endpoint, subject),
                            styled(
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
                            styled(
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
                            styled("clara", "did", "visit", "cedar", subject),
                        ],
                        dependent_prompt("will", "call", subject, true),
                        rotation,
                    )?;
                }
            }
        }
    }
    for relation in ["help", "trust"] {
        for endpoint in ["bruno", "will"] {
            for rotation in 0..4 {
                let family = format!("predicate-{relation}-{endpoint}-rotation{rotation}");
                for variant in ["baseline", "active", "inactive"] {
                    push(
                        &mut rows,
                        family.clone(),
                        variant,
                        [
                            fact("alice", "will", relation, endpoint, false),
                            fact(
                                endpoint,
                                "did",
                                "help",
                                if variant == "active" {
                                    "oscar"
                                } else {
                                    "helen"
                                },
                                false,
                            ),
                            fact(
                                "felix",
                                "will",
                                relation,
                                if variant == "inactive" {
                                    "ruby"
                                } else {
                                    "dylan"
                                },
                                false,
                            ),
                            fact("clara", "did", "visit", "cedar", false),
                        ],
                        dependent_prompt("will", relation, false, false),
                        rotation,
                    )?;
                }
            }
        }
    }
    validate(&rows)?;
    Ok(rows)
}
fn validate(rows: &[Example]) -> Result<()> {
    if rows.len() != 204 {
        return Err("transfer matrix size".into());
    }
    let mut families = BTreeMap::<&str, Vec<&Example>>::new();
    let mut ids = BTreeSet::new();
    for row in rows {
        if !ids.insert(&row.id)
            || row.prompt.len() > 256
            || row
                .records
                .iter()
                .any(|r| r.len() > 128 || words(r).len() > 16)
        {
            return Err("transfer bounds or duplicate ID".into());
        }
        families.entry(&row.family).or_default().push(row);
    }
    for group in families.values() {
        let base = group
            .iter()
            .find(|e| e.variant == "baseline")
            .ok_or("missing paired baseline")?;
        for row in group.iter().filter(|e| e.variant != "baseline") {
            if row.prompt != base.prompt
                || row
                    .records
                    .iter()
                    .zip(&base.records)
                    .filter(|(a, b)| a != b)
                    .count()
                    != 1
            {
                return Err("source intervention is not a one-record replacement".into());
            }
            let changed = row.expected.tokens != base.expected.tokens;
            if changed != (row.variant == "active") || row.expected.outcome != base.expected.outcome
            {
                return Err("independent source intervention oracle mismatch".into());
            }
        }
    }
    Ok(())
}
fn retained(a: &model::Artifact, g: &BoundGeometry, m: &Metric, root: &Path) -> Result<Value> {
    report_output::verify(root)?;
    let data_bytes = std::fs::read(root.join("data.json"))?;
    let response_bytes = std::fs::read(root.join("responses.json"))?;
    let data: Value = serde_json::from_slice(&data_bytes)?;
    let responses: Vec<Value> = serde_json::from_slice(&response_bytes)?;
    let mut total = 0;
    let mut equal = 0;
    let mut items = vec![];
    for (key, label, required) in [
        ("development", "dual_role", 72),
        ("lexical_role", "lexical_role", 48),
        ("phrase_order", "phrase_order", 80),
    ] {
        let examples: Vec<Example> = serde_json::from_value(data[key].clone())?;
        if examples.len() != required {
            return Err("prior role transfer panel count".into());
        }
        for e in examples {
            let matches: Vec<_> = responses
                .iter()
                .filter(|v| v["panel"] == label && v["id"] == e.id && v["control"] == "Full")
                .collect();
            if matches.len() != 1 {
                return Err("prior Full response identity".into());
            }
            let old: completion::Generated =
                serde_json::from_value(matches[0]["response"]["generated"].clone())?;
            let actual = model::generate(a, g, m, &e.records, &e.prompt, model::Control::Full)?;
            let same = old == actual;
            total += 1;
            equal += usize::from(same);
            items.push(json!({"panel":label,"id":e.id,"full_trace_equal":same,"actual_if_changed":if same {Value::Null} else {serde_json::to_value(actual)?}}));
        }
    }
    Ok(
        json!({"root":root,"data_sha256":sha(&data_bytes),"responses_sha256":sha(&response_bytes),"rows":total,"equal":equal,"pass":total==200&&equal==200,"items":items,"older_6688_campaign":"NOT_RERUN; unchanged artifact and runtime, sealed prior result retained"}),
    )
}
#[test]
fn role_transfer_raw_corpus_is_independent_and_source_interventions_are_bounded() -> Result<()> {
    let rows = development()?;
    assert_eq!(
        rows.iter()
            .filter(|e| e.family.starts_with("coverage-"))
            .count(),
        60
    );
    assert_eq!(
        rows.iter()
            .filter(|e| e.family.starts_with("styled-"))
            .count(),
        96
    );
    assert_eq!(
        rows.iter()
            .filter(|e| e.family.starts_with("predicate-"))
            .count(),
        48
    );
    assert_eq!(
        rows.iter()
            .filter(|e| matches!(e.expected.outcome, completion::Outcome::Unresolved { .. }))
            .count(),
        40
    );
    assert!(rows.iter().all(|e| e.expected.selected[0].value == b"bruno"
        || e.expected.selected[0].value == b"will"
        || e.expected.selected[0].value == b"will amber"
        || e.expected.selected[0].value == b"amber will"));
    Ok(())
}
#[test]
#[ignore = "bounded unchanged-artifact role transfer and coverage diagnostic"]
fn unchanged_role_transfer_report() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_ROLE_TRANSFER_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_ROLE_TRANSFER_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty transfer paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let rows = development()?;
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","Full":{"coverage":60,"styled":96,"predicate":48,"all":204,"typed_unresolved":40},"criteria":"exact answer/EOS or typed unresolved; first payload and actual lookahead query/source; committed source/occurrence/span/byte bounds and continuity","paired_interventions":"exactly one changed record and identical prompt; active changes oracle answer, irrelevant changes preserve it","controls":{"ExactIdentity":"204 complete structures equal Full","reload":"204 complete structures equal Full","ReadDisabled":"zero correct answered responses","UpdateDisabled":"zero correct dependent answered responses","CoverageDisabled":"diagnostic, no fixed success count","QueryRolesDisabled":"diagnostic, no fixed success count","RoleDisabled":"diagnostic, no fixed success count"},"retained_Full":200,"limits":{"records":4,"record_words":16,"record_bytes":128,"prompt_bytes":256,"clauses":2,"steps":96},"fits":0,"runtime_changes":false,"final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"oracle":"independent raw authored subject/auxiliary/relation/object grammar; please tell me question prefix and today/tomorrow source modifiers stripped only inside oracle; runtime receives original bytes","matrix":{"coverage":60,"styled":96,"predicate":48},"training":[],"no_training":true}),
        )?;
        let prior = evidence.join("occurrence-role-1/attempt-3");
        report_output::verify(&prior)?;
        let bytes = std::fs::read(prior.join("candidate.json"))?;
        let summary: Value = serde_json::from_slice(&std::fs::read(prior.join("summary.json"))?)?;
        if summary["candidate_sha256"] != sha(&bytes)
            || summary["gate"] != "PASS_LEARNED_OCCURRENCE_ROLE"
        {
            return Err("qualified candidate identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = model::Artifact::decode(&bytes, &g)?;
        if a.schema != 3 || !a.require_available_query_coverage {
            return Err("expected schema3 coverage candidate".into());
        }
        let reload = model::Artifact::decode(&a.encode()?, &g)?;
        write(
            &output,
            "artifact.json",
            &json!({"path":prior.join("candidate.json"),"sha256":sha(&bytes),"schema":a.schema,"unchanged":true,"source_table":a.table,"query_table":a.query_table,"anchors":a.anchors}),
        )?;
        let mut responses = vec![];
        let mut counts = BTreeMap::<String, usize>::new();
        let mut exact_equal = 0;
        let mut reload_equal = 0;
        let mut disabled_answer_correct = 0;
        let mut family_results = BTreeMap::<String, Vec<Value>>::new();
        for e in &rows {
            let panel = e.family.split('-').next().ok_or("panel")?;
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
            let mut controls = serde_json::Map::new();
            let mut full_assessment = Value::Null;
            for (name, c) in [
                ("Full", model::Control::Full),
                ("ExactIdentity", model::Control::ExactIdentity),
                ("CoverageDisabled", model::Control::CoverageDisabled),
                ("QueryRolesDisabled", model::Control::QueryRolesDisabled),
                ("RoleDisabled", model::Control::RoleDisabled),
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
                let assessment = assess(&a, &g, &m, e, c, &out)?;
                let correct = assessment["correct"] == true;
                *counts.entry(format!("{panel}/{name}")).or_default() += usize::from(correct);
                if matches!(
                    c,
                    model::Control::ReadDisabled | model::Control::UpdateDisabled
                ) && e.expected.outcome == completion::Outcome::Answered
                {
                    disabled_answer_correct += usize::from(assessment["outcome_correct"] == true);
                }
                if c == model::Control::Full {
                    full_assessment = assessment.clone();
                }
                controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"steps":out.trace.steps.len(),"equal_full":out==full}));
            }
            let source_roles = e
                .records
                .iter()
                .map(|r| model::source_context(&a, &g, &m, r, false))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            family_results.entry(e.family.clone()).or_default().push(json!({"id":e.id,"variant":e.variant,"correct":full_assessment["correct"],"outcome":full.outcome,"tokens":full.trace.tokens,"initial_selection_correct":full_assessment["initial_selection_correct"],"CoverageDisabled_correct":controls["CoverageDisabled"]["assessment"]["correct"],"QueryRolesDisabled_correct":controls["QueryRolesDisabled"]["assessment"]["correct"]}));
            responses.push(json!({"id":e.id,"panel":panel,"family":e.family,"variant":e.variant,"Full":full,"controls":controls,"source_context":source_roles}));
        }
        write(&output, "responses.json", &json!(responses))?;
        write(
            &output,
            "localization.json",
            &json!({"families":family_results,"scope":"actual unchanged-artifact generation; ablations are controls, never substituted outputs","candidate_selection":"none","training":"none"}),
        )?;
        let retention = retained(&a, &g, &m, &prior)?;
        write(&output, "retained-responses.json", &retention)?;
        report_output::verify(&prior)?;
        let unchanged = std::fs::read(prior.join("candidate.json"))? == bytes;
        let pass = counts["coverage/Full"] == 60
            && counts["styled/Full"] == 96
            && counts["predicate/Full"] == 48
            && exact_equal == 204
            && reload_equal == 204
            && disabled_answer_correct == 0
            && retention["pass"] == true
            && unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_UNCHANGED_ROLE_TRANSFER"}else{"FAIL_UNCHANGED_ROLE_TRANSFER"},"panels":counts,"rows":204,"families":family_results.len(),"exact_equal":exact_equal,"reload_equal":reload_equal,"read_or_update_disabled_correct_answer":disabled_answer_correct,"retained_Full_equal":retention["equal"],"candidate_sha256":sha(&bytes),"artifact_unchanged":unchanged,"fits":0,"runtime_changes":false,"final_holdout":"NOT_RUN","promotion":false,"default_model":"15baec48 unchanged"}),
        )?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"gate":if pass{"PASS_UNCHANGED_ROLE_TRANSFER"}else{"FAIL_UNCHANGED_ROLE_TRANSFER"},"path":output})
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
