//! Frozen-artifact lexical-neighbor transfer. The raw grammar oracle is report-only.
use super::{
    completion, occurrence_role,
    phrase_data::intervals,
    query_participation as model,
    query_participation_report::{assess, oracle, sha, verb, words, write, Example},
    span_data,
    styled_role_report::{retained_200, retained_6688},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ARTIFACT_SHA: &str = "0175ad86850a77017fdc41b7b4da9c22fe41bc8bcbf348e1352967dd29e43a94";
const OLD_SHA: &str = "8055a53c7801f1e6e11f808a6c15ddb090a5befe8d9d587b4ca65c74b7698793";
const NAMES: [(&str, &str); 10] = [
    ("alice", "navor"),
    ("amber", "selka"),
    ("bruno", "tavin"),
    ("cedar", "lumet"),
    ("clara", "orvik"),
    ("dylan", "renza"),
    ("felix", "pelin"),
    ("helen", "dorva"),
    ("oscar", "kesin"),
    ("ruby", "mavri"),
];
const ALTERNATE: [&str; 3] = ["zelun", "fexor", "bexil"];

fn substitute(raw: &[u8]) -> Vec<u8> {
    let mut out = vec![];
    let mut cursor = 0;
    for [start, end] in intervals(raw) {
        out.extend_from_slice(&raw[cursor..start]);
        let token = &raw[start..end];
        out.extend_from_slice(
            NAMES
                .iter()
                .find(|(old, _)| old.as_bytes() == token)
                .map_or(token, |(_, new)| new.as_bytes()),
        );
        cursor = end;
    }
    out.extend_from_slice(&raw[cursor..]);
    out
}

fn novel_tokens() -> BTreeSet<Vec<u8>> {
    NAMES
        .iter()
        .map(|(_, new)| new.as_bytes().to_vec())
        .chain(ALTERNATE.iter().map(|s| s.as_bytes().to_vec()))
        .collect()
}

fn transferred(e: &Example) -> Result<Example> {
    let records = e.records.clone().map(|r| substitute(&r));
    let prompt = substitute(&e.prompt);
    let expected = oracle(&records, &prompt)?;
    if expected.outcome != e.expected.outcome
        || expected.matching_sources != e.expected.matching_sources
        || expected.queries
            != e.expected
                .queries
                .iter()
                .map(|q| substitute(q))
                .collect::<Vec<_>>()
        || expected.selected.len() != e.expected.selected.len()
        || expected
            .selected
            .iter()
            .zip(&e.expected.selected)
            .any(|(a, b)| {
                a.source != b.source || a.span != b.span || a.value != substitute(&b.value)
            })
    {
        return Err("lexical substitution changed independent path semantics".into());
    }
    Ok(Example {
        id: format!("novel-{}", e.id),
        family: e.family.clone(),
        variant: e.variant.clone(),
        records,
        prompt,
        expected,
    })
}

fn alternate(n: usize) -> Result<Vec<u8>> {
    if n == 0 || n > 3 {
        return Err("alternate endpoint word bound".into());
    }
    Ok(ALTERNATE[..n].join(" ").into_bytes())
}

fn splice(raw: &[u8], bounds: [usize; 2], replacement: &[u8]) -> Vec<u8> {
    [&raw[..bounds[0]], replacement, &raw[bounds[1]..]].concat()
}

fn continuation(e: &Example, conflict: bool) -> Result<Example> {
    if e.expected.outcome != completion::Outcome::Answered || e.expected.selected.len() != 2 {
        return Err("continuation requires a supported two-clause baseline".into());
    }
    let selected = &e.expected.selected[1];
    let raw = &e.records[selected.source];
    let ranges = intervals(raw);
    let w = words(raw);
    let aux: Vec<_> = w
        .iter()
        .enumerate()
        .filter(|(i, t)| matches!(**t, b"did" | b"will") && w.get(i + 1).is_some_and(|t| verb(t)))
        .map(|(i, _)| i)
        .collect();
    if aux.len() != 1 {
        return Err("continuation source grammar".into());
    }
    let a = aux[0];
    let begin = usize::from(w.first() == Some(&b"today".as_slice()));
    let end = w.len() - usize::from(w.last() == Some(&b"tomorrow".as_slice()));
    let other = if selected.span == [begin, a] {
        [a + 2, end]
    } else if selected.span == [a + 2, end] {
        [begin, a]
    } else {
        return Err("oracle answer endpoint".into());
    };
    let mut records = e.records.clone();
    if conflict {
        // Frozen styled rows rotate [first, continuation, inactive, spare].
        // Replace the unchanged spare, preserving the separately mutated inactive row.
        let spare = (e.expected.selected[0].source + 3) % 4;
        if e.expected.selected.iter().any(|s| s.source == spare) {
            return Err("no independent conflict slot".into());
        }
        records[spare] = splice(
            raw,
            selected.bounds,
            &alternate(selected.span[1] - selected.span[0])?,
        );
    } else {
        records[selected.source] = splice(
            raw,
            [ranges[other[0]][0], ranges[other[1] - 1][1]],
            &alternate(other[1] - other[0])?,
        );
    }
    let expected = oracle(&records, &e.prompt)?;
    let reason = if conflict {
        completion::Reason::Ambiguous
    } else {
        completion::Reason::NoCompatibleCandidate
    };
    if expected.outcome != (completion::Outcome::Unresolved { clause: 1, reason })
        || expected.selected != e.expected.selected[..1]
        || expected.queries != e.expected.queries
        || expected.matching_sources[0] != e.expected.matching_sources[0]
        || expected.matching_sources[1].len() != if conflict { 2 } else { 0 }
    {
        return Err("continuation manipulation did not isolate second-hop availability".into());
    }
    let mode = if conflict { "conflict" } else { "missing" };
    Ok(Example {
        id: format!("{}-{mode}", e.id),
        family: format!("styled-{mode}-{}", e.family),
        variant: e.variant.clone(),
        records,
        prompt: e.prompt.clone(),
        expected,
    })
}

fn panel(source: &[Example]) -> Result<(Vec<Example>, Value)> {
    let mut rows = vec![];
    let mut mapping = vec![];
    for old in source {
        if oracle(&old.records, &old.prompt)? != old.expected {
            return Err("sealed raw oracle changed".into());
        }
        let e = transferred(old)?;
        mapping.push(json!({"source_id":old.id,"new_id":e.id,"records_and_question_bijective_token_substitution_only":true,"matching_sources_and_word_spans_preserved":true}));
        if e.family.starts_with("styled-") {
            for conflict in [false, true] {
                let c = continuation(&e, conflict)?;
                mapping.push(json!({"source_id":old.id,"new_id":c.id,"valid_match_id":e.id,"question_preserved":true,"second_hop_only":true,"mode":if conflict{"conflict"}else{"missing"}}));
                rows.push(c);
            }
        }
        rows.push(e);
    }
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    for e in &rows {
        if !ids.insert(&e.id)
            || !inputs.insert(serde_json::to_vec(&(&e.records, &e.prompt))?)
            || e.records
                .iter()
                .any(|r| r.len() > 128 || words(r).len() > 16)
            || e.prompt.len() > 256
            || e.expected.queries.len() != 2
            || e.expected
                .selected
                .iter()
                .any(|s| s.value.len() > 50 || words(&s.value).len() > 3)
        {
            return Err("novel panel identity or machine window bounds".into());
        }
    }
    if source.len() != 300
        || rows.len() != 492
        || rows
            .iter()
            .filter(|e| e.expected.outcome == completion::Outcome::Answered)
            .count()
            != 260
    {
        return Err("frozen novel panel count".into());
    }
    Ok((rows, json!(mapping)))
}

// Collect every serialized canonical word inventory, including nested parents.
fn inventory(value: &Value, out: &mut BTreeSet<Vec<u8>>) -> Result<()> {
    match value {
        Value::Object(map) => {
            if map.contains_key("geometry") && map.contains_key("bytes") {
                out.insert(serde_json::from_value(map["bytes"].clone())?);
            }
            for v in map.values() {
                inventory(v, out)?;
            }
        }
        Value::Array(values) => {
            for v in values {
                inventory(v, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn key_matches(pattern: &occurrence_role::Key, observed: &occurrence_role::Key) -> bool {
    pattern.center == observed.center
        && (pattern.left == 66 || pattern.left == observed.left)
        && (pattern.right == 66 || pattern.right == observed.right)
}

fn observations(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    raw: &[u8],
    query: bool,
    credit_sets: &[(&str, Vec<occurrence_role::Credit>)],
) -> Result<Value> {
    let keys =
        occurrence_role::observations(&a.parent.parent, &a.parent.anchors, g, m, raw, false)?;
    let exact =
        occurrence_role::observations(&a.parent.parent, &a.parent.anchors, g, m, raw, true)?;
    let roles = if query {
        occurrence_role::query_context(&a.parent, g, m, raw, false)?
    } else {
        occurrence_role::source_context(&a.parent, g, m, raw, false)?
    };
    let table = if query {
        &a.parent.query_table
    } else {
        &a.parent.table
    };
    let w = words(raw);
    let mut items = vec![];
    for (index, key) in keys.iter().enumerate() {
        let credits: Vec<_> = credit_sets.iter().map(|(name, credit)| {
            let matching: Vec<_> = credit.iter().filter(|c| c.key == *key).collect();
            json!({"stage":name,"exact_key_rows":matching.len(),"content_credit":matching.iter().map(|c| c.content).sum::<usize>(),"context_credit":matching.iter().map(|c| c.context).sum::<usize>()})
        }).collect();
        items.push(json!({"index":index,"word":w[index],"key":key,"context":roles[index],"unknown_center":key.center==64,"unknown_left":key.left==64,"unknown_right":key.right==64,"matching_runtime_rules":table.iter().filter(|r| key_matches(&r.key,key)).collect::<Vec<_>>(),"training_credit":credits}));
    }
    let participation = if query {
        json!({"keys":model::observations(a,g,m,raw,false)?,"required":a.required_mask(g,m,raw,false)?,"replacement":a.replacement_mask(g,m,raw,false)?,"inventory":"outer query-participation anchors; indices are distinct from occurrence-role anchor indices"})
    } else {
        Value::Null
    };
    Ok(
        json!({"raw":raw,"query":query,"exact_geometric_keys_equal":keys==exact,"items":items,"participation":participation,"zero_credit_means":"No exact observed-key credit in the cited fit stage; no grammatical label or impossibility is inferred."}),
    )
}

fn failure_observations(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    out: &completion::Generated,
    source_credit: &[(&str, Vec<occurrence_role::Credit>)],
    query_credit: &[(&str, Vec<occurrence_role::Credit>)],
) -> Result<Value> {
    let sources = e
        .records
        .iter()
        .map(|r| observations(a, g, m, r, false, source_credit))
        .collect::<Result<Vec<_>>>()?;
    let expected = e
        .expected
        .queries
        .iter()
        .map(|q| observations(a, g, m, q, true, query_credit))
        .collect::<Result<Vec<_>>>()?;
    let actual_queries: BTreeSet<_> = out
        .decisions
        .iter()
        .map(|d| d.before.core.query.bytes.clone())
        .collect();
    let actual = actual_queries
        .iter()
        .map(|q| observations(a, g, m, q, true, query_credit))
        .collect::<Result<Vec<_>>>()?;
    Ok(
        json!({"id":e.id,"sources":sources,"expected_queries":expected,"actual_committed_queries":actual}),
    )
}

fn observation_transfer(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    source: &[Example],
    development: &[Example],
    training: &[span_data::Example],
) -> Result<Value> {
    let mut pairs = vec![];
    let mut witnesses = std::collections::BTreeMap::<
        (bool, occurrence_role::Key),
        std::collections::BTreeMap<bool, Vec<Value>>,
    >::new();
    let mut unique = BTreeSet::new();
    for e in source {
        let id = format!("novel-{}", e.id);
        let novel = development
            .iter()
            .find(|n| n.id == id)
            .ok_or("paired novel row")?;
        let mut rows = vec![];
        for (query, old, new) in e
            .records
            .iter()
            .zip(&novel.records)
            .map(|(a, b)| (false, a, b))
            .chain(
                e.expected
                    .queries
                    .iter()
                    .zip(&novel.expected.queries)
                    .map(|(a, b)| (true, a, b)),
            )
        {
            let old_keys = occurrence_role::observations(
                &a.parent.parent,
                &a.parent.anchors,
                g,
                m,
                old,
                false,
            )?;
            let new_keys = occurrence_role::observations(
                &a.parent.parent,
                &a.parent.anchors,
                g,
                m,
                new,
                false,
            )?;
            let old_roles = if query {
                occurrence_role::query_context(&a.parent, g, m, old, false)?
            } else {
                occurrence_role::source_context(&a.parent, g, m, old, false)?
            };
            let new_roles = if query {
                occurrence_role::query_context(&a.parent, g, m, new, false)?
            } else {
                occurrence_role::source_context(&a.parent, g, m, new, false)?
            };
            let participation = if query {
                json!({"old_keys":model::observations(a,g,m,old,false)?,"new_keys":model::observations(a,g,m,new,false)?,"old_required":a.required_mask(g,m,old,false)?,"new_required":a.required_mask(g,m,new,false)?})
            } else {
                Value::Null
            };
            rows.push(json!({"query":query,"old_raw":old,"new_raw":new,"old_keys":old_keys,"new_keys":new_keys,"old_context":old_roles,"new_context":new_roles,"roles_equal":old_roles==new_roles,"participation":participation}));
        }
        pairs.push(json!({"source_id":e.id,"new_id":novel.id,"rows":rows}));
    }
    let mut raw_rows = vec![];
    for e in development {
        for (query, raw) in e
            .records
            .iter()
            .map(|r| (false, r))
            .chain(e.expected.queries.iter().map(|q| (true, q)))
        {
            raw_rows.push(("development", e.id.as_str(), query, raw));
        }
    }
    for e in training {
        raw_rows.push(("training", e.id.as_str(), true, &e.prompt));
    }
    for (origin, id, query, raw) in raw_rows {
        if !unique.insert((query, raw.clone())) {
            continue;
        }
        let w = words(raw);
        let aux = if query {
            usize::from(w.starts_with(&[b"please".as_slice(), b"tell", b"me"])) * 3 + 1
        } else {
            w.iter()
                .enumerate()
                .find(|(i, t)| {
                    matches!(**t, b"did" | b"will") && w.get(i + 1).is_some_and(|t| verb(t))
                })
                .map(|(i, _)| i)
                .ok_or("role audit auxiliary")?
        };
        let keys =
            occurrence_role::observations(&a.parent.parent, &a.parent.anchors, g, m, raw, false)?;
        let roles = if query {
            occurrence_role::query_context(&a.parent, g, m, raw, false)?
        } else {
            occurrence_role::source_context(&a.parent, g, m, raw, false)?
        };
        for (i, word) in w.iter().enumerate().filter(|(_, word)| **word == b"will") {
            let _ = word;
            let expected_context = i == aux;
            let bucket = witnesses
                .entry((query, keys[i].clone()))
                .or_default()
                .entry(expected_context)
                .or_default();
            if bucket.len() < 2 {
                bucket.push(json!({"origin":origin,"id":id,"raw":raw,"index":i,"expected_context_from_authored_auxiliary_position":expected_context,"actual_context":roles[i]}));
            }
        }
    }
    let collisions:Vec<_>=witnesses.iter().filter(|(_,roles)|roles.len()==2).map(|((query,key),roles)|json!({"query":query,"key":key,"context_witnesses":roles.get(&true),"content_witnesses":roles.get(&false)})).collect();
    Ok(
        json!({"paired_rows":pairs,"opposite_authored_will_roles_same_observed_key":collisions,"collision_count":collisions.len(),"scope":"Exact same observed role key assigned both name-content and auxiliary-context by the independent raw authored grammar. Source and query keys are separate. A collision does not by itself establish complete-model failure.","triple_name_internal_will":"NOT_TESTED: inherited styled endpoint forms are one or two words."}),
    )
}

#[test]
fn neighbor_substitution_preserves_role_tokens_and_punctuation() {
    assert_eq!(
        substitute(
            b"today amber will will call alice tomorrow. please tell me who will amber will help?"
        ),
        b"today selka will will call navor tomorrow. please tell me who will selka will help?"
    );
}

#[test]
fn neighbor_continuation_changes_only_second_hop_availability() -> Result<()> {
    let records = [
        b"today navor will call will selka tomorrow.".to_vec(),
        b"today will selka did help dorva tomorrow.".to_vec(),
        b"today pelin will help renza tomorrow.".to_vec(),
        b"today orvik did visit lumet tomorrow.".to_vec(),
    ];
    let prompt = b"please tell me who will navor call? please tell me who did they help?".to_vec();
    let e = Example {
        id: "probe".into(),
        family: "styled-probe".into(),
        variant: "baseline".into(),
        expected: oracle(&records, &prompt)?,
        records,
        prompt,
    };
    assert_eq!(e.expected.outcome, completion::Outcome::Answered);
    assert_eq!(
        continuation(&e, false)?.expected.matching_sources[1],
        Vec::<usize>::new()
    );
    assert_eq!(
        continuation(&e, true)?.expected.matching_sources[1].len(),
        2
    );
    Ok(())
}

#[test]
#[ignore = "bounded unchanged-artifact lexical-neighbor transfer and retained controls"]
fn unchanged_neighbor_transfer_report() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_NEIGHBOR_TRANSFER_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_NEIGHBOR_TRANSFER_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let root = evidence.join("styled-role-1/attempt-3");
        report_output::verify(&root)?;
        let data_bytes = std::fs::read(root.join("data.json"))?;
        let response_bytes = std::fs::read(root.join("responses.json"))?;
        let data: Value = serde_json::from_slice(&data_bytes)?;
        let sealed: Vec<Value> = serde_json::from_slice(&response_bytes)?;
        let mut source: Vec<Example> = serde_json::from_value(data["development"].clone())?;
        source.extend(serde_json::from_value::<Vec<Example>>(
            data["collision"].clone(),
        )?);
        let (development, mapping) = panel(&source)?;
        let training: Vec<span_data::Example> = serde_json::from_value(data["training"].clone())?;
        let mut training_tokens = BTreeSet::new();
        let mut training_inputs = BTreeSet::new();
        for e in &training {
            for raw in e.records.iter().chain([&e.prompt, &e.answer]) {
                training_tokens.extend(words(raw).into_iter().map(Vec::from));
            }
            training_inputs.insert(serde_json::to_vec(&(&e.records, &e.prompt))?);
        }
        let novel = novel_tokens();
        let mut overlap = 0;
        for e in &development {
            overlap += usize::from(
                training_inputs.contains(&serde_json::to_vec(&(&e.records, &e.prompt))?),
            );
        }
        if novel.len() != 13 || !novel.is_disjoint(&training_tokens) || overlap != 0 {
            return Err("declared novel vocabulary overlaps training".into());
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","artifact_sha256":ARTIFACT_SHA,"new_rows":492,"answered":260,"typed_unresolved":232,"all_complete_structures_correct":492,"retained_sealed_full":300,"retained_latest":200,"retained_earlier":6688,"earlier_typed_unresolved":512,"controls":{"ExactIdentity":"all492 equal Full","reload":"all492 equal Full","ReadDisabled":"zero correct answered outputs","UpdateDisabled":"zero correct answered outputs"},"fits":0,"runtime_changes":0,"new_name_mapping":NAMES,"additional_control_names":ALTERNATE,"limits":{"records":4,"record_words":16,"record_bytes":128,"prompt_bytes":256,"clauses":2,"payload_words":3,"payload_bytes":50,"steps":96},"scope":"open development lexical-neighbor transfer at fixed authored grammar and depth","final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"development":development,"mapping":mapping,"source_root":root,"source_data_sha256":sha(&data_bytes),"source_responses_sha256":sha(&response_bytes),"names_absent_training":true,"training_rows":training.len(),"novel_raw_training_overlap":0,"independent_expected_from_raw":true}),
        )?;
        let bytes = std::fs::read(root.join("candidate.json"))?;
        if sha(&bytes) != ARTIFACT_SHA {
            return Err("frozen artifact identity".into());
        }
        let mut all_words = BTreeSet::new();
        inventory(&serde_json::from_slice(&bytes)?, &mut all_words)?;
        if !novel.is_disjoint(&all_words) {
            return Err("novel word appears in bound nested inventory".into());
        }
        write(
            &output,
            "environment.json",
            &json!({"pass":true,"frozen_before_generation":true,"artifact_sha256":ARTIFACT_SHA,"novel_names":novel,"nested_canonical_words":all_words.len(),"artifact_inventory_overlap":0,"training_token_overlap":0,"unknown_marker":64,"edge_marker":65,"wildcard_marker":66,"unknown_is_not_semantic_position":true}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = model::Artifact::decode(&bytes, &g)?;
        let reload = model::Artifact::decode(&a.encode()?, &g)?;
        write(
            &output,
            "observation-transfer.json",
            &observation_transfer(&a, &g, &m, &source, &development, &training)?,
        )?;
        let source_fit: Value =
            serde_json::from_slice(&std::fs::read(root.join("source-fit.json"))?)?;
        let query_fit: Value =
            serde_json::from_slice(&std::fs::read(root.join("query-fit.json"))?)?;
        let source_credit = [(
            "paired_source",
            serde_json::from_value::<Vec<occurrence_role::Credit>>(
                source_fit["fit"]["credits"].clone(),
            )?,
        )];
        let query_credit = [
            (
                "initial_query",
                serde_json::from_value::<Vec<occurrence_role::Credit>>(
                    query_fit["initial"]["credits"].clone(),
                )?,
            ),
            (
                "refined_query",
                serde_json::from_value::<Vec<occurrence_role::Credit>>(
                    query_fit["refinement"]["credits"].clone(),
                )?,
            ),
        ];
        let mut counts = std::collections::BTreeMap::<String, usize>::new();
        let mut responses = vec![];
        let mut failures = vec![];
        let mut exact = 0;
        let mut reloaded = 0;
        let mut correct = 0;
        let mut disabled_answer_correct = 0;
        for e in &development {
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
            let mut controls = serde_json::Map::new();
            for (name, c) in [
                ("Full", model::Control::Full),
                ("ExactIdentity", model::Control::ExactIdentity),
                ("ReadDisabled", model::Control::ReadDisabled),
                ("UpdateDisabled", model::Control::UpdateDisabled),
            ] {
                let out = if c == model::Control::Full {
                    full.clone()
                } else {
                    model::generate(&a, &g, &m, &e.records, &e.prompt, c)?
                };
                let assessment = assess(&a, &g, &m, e, c, &out)?;
                let yes = assessment["correct"] == true;
                let panel = if e.family.starts_with("styled-missing-") {
                    "styled_missing"
                } else if e.family.starts_with("styled-conflict-") {
                    "styled_conflict"
                } else {
                    e.family.split('-').next().ok_or("family")?
                };
                *counts.entry(format!("{panel}/{name}")).or_default() += usize::from(yes);
                if c == model::Control::Full {
                    correct += usize::from(yes);
                    if !yes {
                        failures.push(failure_observations(
                            &a,
                            &g,
                            &m,
                            e,
                            &out,
                            &source_credit,
                            &query_credit,
                        )?);
                    }
                }
                if c == model::Control::ExactIdentity {
                    exact += usize::from(out == full);
                }
                if matches!(
                    c,
                    model::Control::ReadDisabled | model::Control::UpdateDisabled
                ) && e.expected.outcome == completion::Outcome::Answered
                {
                    disabled_answer_correct += usize::from(assessment["outcome_correct"] == true);
                }
                controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"equal_full":out==full}));
            }
            responses.push(json!({"id":e.id,"family":e.family,"variant":e.variant,"Full":full,"controls":controls}));
        }
        write(&output, "responses.json", &json!(responses))?;
        write(
            &output,
            "failure-observations.json",
            &json!({"rows":failures.len(),"source_fit_sha256":sha(&std::fs::read(root.join("source-fit.json"))?),"query_fit_sha256":sha(&std::fs::read(root.join("query-fit.json"))?),"source_anchors":a.parent.anchors,"context_inventory":a.parent.parent.parent.context_words,"items":failures}),
        )?;
        let mut retained = vec![];
        let mut retained_equal = 0;
        for e in &source {
            let matching: Vec<_> = sealed.iter().filter(|r| r["id"] == e.id).collect();
            if matching.len() != 1
                || matching[0]["controls"]["Full"]["assessment"]["correct"] != true
            {
                return Err("sealed retained row identity".into());
            }
            let expected: completion::Generated =
                serde_json::from_value(matching[0]["Full"].clone())?;
            let actual = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            let equal = actual == expected;
            retained_equal += usize::from(equal);
            retained.push(json!({"id":e.id,"full_equal":equal,"actual_if_changed":if equal{Value::Null}else{json!(actual)}}));
        }
        write(
            &output,
            "retained-300.json",
            &json!({"source_root":root,"source_responses_sha256":sha(&response_bytes),"rows":300,"full_equal":retained_equal,"items":retained}),
        )?;
        let old_root = evidence.join("query-participation-1/attempt-1");
        report_output::verify(&old_root)?;
        let old_bytes = std::fs::read(old_root.join("candidate.json"))?;
        if sha(&old_bytes) != OLD_SHA {
            return Err("retained8055 artifact identity".into());
        }
        let old = model::Artifact::decode(&old_bytes, &g)?;
        let r200 = retained_200(&a, &old, &g, &m, &evidence)?;
        write(&output, "retained-200.json", &r200)?;
        let r6688 = retained_6688(&a, &old, &g, &m, &evidence)?;
        write(&output, "retained-6688.json", &r6688)?;
        let artifact_unchanged = std::fs::read(root.join("candidate.json"))? == bytes;
        report_output::verify(&root)?;
        let pass = correct == 492
            && exact == 492
            && reloaded == 492
            && disabled_answer_correct == 0
            && retained_equal == 300
            && r200["pass"] == true
            && r6688["pass"] == true
            && artifact_unchanged;
        let gate = if pass {
            "PASS_UNCHANGED_NEIGHBOR_TRANSFER"
        } else {
            "FAIL_UNCHANGED_NEIGHBOR_TRANSFER"
        };
        write(
            &output,
            "summary.json",
            &json!({"gate":gate,"rows":492,"correct":correct,"panels":counts,"exact_equal":exact,"reload_equal":reloaded,"read_or_update_disabled_correct_answer":disabled_answer_correct,"retained_300_equal":retained_equal,"retained_200_equal":r200["equal"],"retained_6688_equal":r6688["equal"],"retained_typed_unresolved":r6688["typed_unresolved"],"artifact_sha256":ARTIFACT_SHA,"artifact_unchanged":artifact_unchanged,"fits":0,"runtime_changes":0,"final_holdout":"NOT_RUN","promotion":false,"scope":"fixed authored two-clause grammar, four records, novel ordinary name neighbors; no general-language qualification"}),
        )?;
        println!(
            "{}",
            serde_json::to_string(&json!({"gate":gate,"path":output}))?
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
