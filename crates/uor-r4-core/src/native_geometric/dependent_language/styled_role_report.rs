//! Paired output-credit occurrence roles, with frozen query participation.
//! Authored endpoint parsing and expected paths are report-only supervision.
use super::{
    completion, occurrence_role, query_participation as model,
    query_participation_report::{
        assess, oracle, prior_examples, question, sha, verb, words, write, Example,
    },
    span_data,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn endpoints(raw: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
    let w = words(raw);
    let positions: Vec<_> = w
        .iter()
        .enumerate()
        .filter(|(i, x)| matches!(**x, b"did" | b"will") && w.get(i + 1).is_some_and(|v| verb(v)))
        .map(|(i, _)| i)
        .collect();
    if positions.len() != 1 {
        return Err("training fact auxiliary is not unique".into());
    }
    let a = positions[0];
    let begin = usize::from(w.first() == Some(&b"today".as_slice()));
    let end = w.len() - usize::from(w.last() == Some(&b"tomorrow".as_slice()));
    if a <= begin || a + 2 >= end {
        return Err("training fact endpoints".into());
    }
    Ok((
        w[begin..a].join(&b' '),
        w[a].to_vec(),
        w[a + 1].to_vec(),
        w[a + 2..end].join(&b' '),
    ))
}

fn direct(records: &[Vec<u8>; 4], prompt: Vec<u8>, answer: Vec<u8>) -> Result<span_data::Example> {
    let expected = oracle(records, &prompt)?;
    if expected.outcome != completion::Outcome::Answered
        || expected.selected.len() != 1
        || expected.selected[0].value != answer
    {
        return Err("independent direct answer is not uniquely supported".into());
    }
    Ok(span_data::Example {
        id: String::new(),
        family: "paired-styled-direct".into(),
        depth: 1,
        variant: "authored".into(),
        records: records.clone(),
        prompt,
        answer,
        expected_path: vec![],
        expected_span: [0; 3],
        expected_bounds: [0; 2],
    })
}

fn add_pair(
    rows: &mut BTreeMap<Vec<u8>, span_data::Example>,
    records: &[Vec<u8>; 4],
    source: usize,
) -> Result<()> {
    let (subject, auxiliary, relation, object) = endpoints(&records[source])?;
    let aux = std::str::from_utf8(&auxiliary)?;
    let rel = std::str::from_utf8(&relation)?;
    for (known, answer, asks_subject) in [(&subject, &object, false), (&object, &subject, true)] {
        let e = direct(
            records,
            question(std::str::from_utf8(known)?, aux, rel, asks_subject),
            answer.clone(),
        )?;
        let key = serde_json::to_vec(&(&e.records, &e.prompt))?;
        if let Some(old) = rows.insert(key, e.clone()) {
            if old.answer != e.answer {
                return Err("contradictory direct answers".into());
            }
        }
    }
    Ok(())
}

fn paired_training(evidence: &Path) -> Result<(Vec<span_data::Example>, Value)> {
    let root = evidence.join("occurrence-role-1/attempt-3");
    report_output::verify(&root)?;
    let bytes = std::fs::read(root.join("data.json"))?;
    let old: Value = serde_json::from_slice(&bytes)?;
    let mut rows = BTreeMap::new();
    let mut historical = 0;
    for key in ["training", "query_training"] {
        let previous: Vec<span_data::Example> = serde_json::from_value(old[key].clone())?;
        historical += previous.len();
        for e in previous {
            let expected = oracle(&e.records, &e.prompt)?;
            if expected.selected.len() != 1 || expected.selected[0].value != e.answer {
                return Err("historical raw answer drift".into());
            }
            add_pair(&mut rows, &e.records, expected.selected[0].source)?;
        }
    }
    let historical_paired = rows.len();
    for leading in [false, true] {
        for trailing in [false, true] {
            for auxiliary in ["did", "will"] {
                for relation in ["call", "help", "visit"] {
                    let other = if relation == "visit" { "call" } else { "visit" };
                    for known in ["clara", "dylan"] {
                        for endpoint in [
                            "will",
                            "will amber",
                            "amber will",
                            "will amber cedar",
                            "amber will cedar",
                            "amber cedar will",
                            "bruno",
                        ] {
                            for subject in [false, true] {
                                let (left, right) = if subject {
                                    (endpoint, known)
                                } else {
                                    (known, endpoint)
                                };
                                let style = |s: String| {
                                    format!(
                                        "{}{}{}.",
                                        if leading { "today " } else { "" },
                                        s,
                                        if trailing { " tomorrow" } else { "" }
                                    )
                                    .into_bytes()
                                };
                                let records = [
                                    style(format!("{left} {auxiliary} {relation} {right}")),
                                    style(format!("felix {auxiliary} {relation} ruby")),
                                    style(format!("alice {auxiliary} {other} oscar")),
                                    style(format!("helen {auxiliary} {other} tell")),
                                ];
                                add_pair(&mut rows, &records, 0)?;
                            }
                        }
                    }
                }
            }
        }
    }
    let mut result: Vec<_> = rows.into_values().collect();
    for (i, e) in result.iter_mut().enumerate() {
        e.id = format!("paired-training-{i}");
    }
    if historical != 816
        || result.is_empty()
        || result.len() > 8192
        || result.iter().any(|e| {
            e.records
                .iter()
                .any(|r| r.len() > 128 || words(r).len() > 16)
                || e.prompt.len() > 128
                || e.answer.len() > 50
                || words(&e.answer).len() > 3
                || !e.expected_path.is_empty()
                || e.expected_span != [0; 3]
                || e.expected_bounds != [0; 2]
        })
    {
        return Err("paired training bounds or metadata".into());
    }
    Ok((
        result,
        json!({"historical_rows":historical,"historical_unique_reciprocal_rows":historical_paired,"source":root,"source_data_sha256":sha(&bytes),"construction":"Each uniquely supported raw fact is questioned in both endpoint directions in exactly the same four-record environment; only direct byte/EOS answers enter fitting. New styled raw examples include both names and auxiliaries, both record edges, both directions, and different-answer same-predicate distractors."}),
    ))
}

fn audit(
    parent: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    training: &[span_data::Example],
) -> Result<Value> {
    let anchors = occurrence_role::learn_anchors(&parent.parent.parent, g, training)?;
    let mut rows = vec![];
    let mut pass = true;
    for (raw, content, context) in [
        (b"today will will call clara tomorrow.".as_slice(), 1, 2),
        (
            b"today amber will will call clara tomorrow.".as_slice(),
            2,
            3,
        ),
        (b"clara will call amber will tomorrow.".as_slice(), 4, 1),
    ] {
        let keys =
            occurrence_role::observations(&parent.parent.parent, &anchors, g, m, raw, false)?;
        let exact =
            occurrence_role::observations(&parent.parent.parent, &anchors, g, m, raw, true)?;
        let distinct =
            keys[content].center == keys[context].center && keys[content] != keys[context];
        pass &= distinct && keys == exact;
        rows.push(json!({"raw":raw,"content_occurrence":content,"context_occurrence":context,"content_key":keys[content],"context_key":keys[context],"same_identity_distinct_context":distinct,"exact_geometric_equal":keys==exact}));
    }
    Ok(
        json!({"status":"BEFORE_FIT","pass":pass,"pairs":rows,"anchors":anchors.len(),"scope":"The exact observed contexts distinguish the three motivating name/auxiliary pairs; this does not establish a learned role or global separability."}),
    )
}

pub(super) fn retained_200(
    a: &model::Artifact,
    parent: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value> {
    let root = evidence.join("occurrence-role-1/attempt-3");
    report_output::verify(&root)?;
    let data_bytes = std::fs::read(root.join("data.json"))?;
    let response_bytes = std::fs::read(root.join("responses.json"))?;
    let data: Value = serde_json::from_slice(&data_bytes)?;
    let responses: Vec<Value> = serde_json::from_slice(&response_bytes)?;
    let mut count = 0;
    let mut equal = 0;
    let mut parent_equal = 0;
    let mut items = vec![];
    for (key, label, size) in [
        ("development", "dual_role", 72),
        ("lexical_role", "lexical_role", 48),
        ("phrase_order", "phrase_order", 80),
    ] {
        let rows: Vec<Example> = serde_json::from_value(data[key].clone())?;
        if rows.len() != size {
            return Err("retained200 panel size".into());
        }
        for e in rows {
            let matching: Vec<_> = responses
                .iter()
                .filter(|v| v["panel"] == label && v["id"] == e.id && v["control"] == "Full")
                .collect();
            if matching.len() != 1 {
                return Err("retained200 response identity".into());
            }
            let sealed: completion::Generated =
                serde_json::from_value(matching[0]["response"]["generated"].clone())?;
            let previous =
                model::generate(parent, g, m, &e.records, &e.prompt, model::Control::Full)?;
            let actual = model::generate(a, g, m, &e.records, &e.prompt, model::Control::Full)?;
            let same = actual == sealed;
            count += 1;
            equal += usize::from(same);
            parent_equal += usize::from(previous == sealed);
            items.push(json!({"id":e.id,"panel":label,"candidate_sealed_equal":same,"parent_sealed_equal":previous==sealed,"actual_if_changed":if same {Value::Null} else {json!(actual)}}));
        }
    }
    Ok(
        json!({"rows":count,"equal":equal,"parent_equal":parent_equal,"pass":count==200&&equal==200&&parent_equal==200,"data_sha256":sha(&data_bytes),"responses_sha256":sha(&response_bytes),"items":items}),
    )
}

pub(super) fn retained_6688(
    a: &model::Artifact,
    parent: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value> {
    let mut count = 0;
    let mut equal = 0;
    let mut unresolved = 0;
    let mut panels = vec![];
    for (name, attempt, expected) in [
        ("language-phrase-update-1", "attempt-1", 864),
        ("language-span-1", "attempt-3", 1728),
        ("language-scheduling-1", "attempt-1", 1536),
        ("language-depth-1", "attempt-1", 1280),
        ("language-completion-1", "attempt-1", 896),
        ("correspondence-runs-1", "attempt-2", 384),
    ] {
        let root = evidence.join(name).join(attempt);
        report_output::verify(&root)?;
        let bytes = std::fs::read(root.join("data.json"))?;
        let data: Value = serde_json::from_slice(&bytes)?;
        let rows = data["development"]
            .as_array()
            .ok_or("retained development")?;
        if rows.len() != expected {
            return Err("retained panel count".into());
        }
        let mut same_count = 0;
        let mut items = vec![];
        for e in rows {
            let records: [Vec<u8>; 4] = serde_json::from_value(e["records"].clone())?;
            let prompt: Vec<u8> = serde_json::from_value(e["prompt"].clone())?;
            let old = model::generate(parent, g, m, &records, &prompt, model::Control::Full)?;
            let actual = model::generate(a, g, m, &records, &prompt, model::Control::Full)?;
            let same = old == actual;
            same_count += usize::from(same);
            unresolved += usize::from(
                same && matches!(actual.outcome, completion::Outcome::Unresolved { .. }),
            );
            items.push(json!({"id":e["id"],"equal":same,"actual_if_changed":if same {Value::Null} else {json!(actual)},"parent_if_changed":if same {Value::Null} else {json!(old)}}));
        }
        count += rows.len();
        equal += same_count;
        panels.push(json!({"root":root,"data_sha256":sha(&bytes),"rows":rows.len(),"equal":same_count,"items":items}));
    }
    Ok(
        json!({"rows":count,"equal":equal,"typed_unresolved":unresolved,"pass":count==6688&&equal==6688&&unresolved==512,"comparison":"candidate versus actual frozen query-participation8055 Full generation; earlier sealed qualifications establish its prior lineage","panels":panels,"historical_adapter_controls":"NOT_RUN"}),
    )
}

#[test]
fn paired_styled_fact_endpoints_keep_names_separate_from_style() -> Result<()> {
    let a = endpoints(b"today will amber will call clara tomorrow.")?;
    assert_eq!(a.0, b"will amber");
    assert_eq!(a.1, b"will");
    assert_eq!(a.3, b"clara");
    let b = endpoints(b"clara did help amber will tomorrow.")?;
    assert_eq!(b.3, b"amber will");
    Ok(())
}

#[test]
#[ignore = "bounded paired source/query role fits and retained-path qualification"]
fn learned_styled_role_report() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_STYLED_ROLE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_STYLED_ROLE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let (training, construction) = paired_training(&evidence)?;
        let prior_attempt = evidence.join("styled-role-1/attempt-1");
        report_output::verify(&prior_attempt)?;
        let prior_data: Value =
            serde_json::from_slice(&std::fs::read(prior_attempt.join("data.json"))?)?;
        if prior_data["training"] != serde_json::to_value(&training)? {
            return Err("retry training changed".into());
        }
        let prior_responses: Vec<Value> =
            serde_json::from_slice(&std::fs::read(prior_attempt.join("responses.json"))?)?;
        let second_attempt = evidence.join("styled-role-1/attempt-2");
        report_output::verify(&second_attempt)?;
        let second_data: Value =
            serde_json::from_slice(&std::fs::read(second_attempt.join("data.json"))?)?;
        if second_data["training"] != serde_json::to_value(&training)? {
            return Err("second retry training changed".into());
        }
        let second_responses: Vec<Value> =
            serde_json::from_slice(&std::fs::read(second_attempt.join("responses.json"))?)?;
        let parent_root = evidence.join("query-participation-1/attempt-1");
        let collision_root = evidence.join("query-participation-1/collision-probe-1");
        let (development, previous, data_sha, responses_sha) = prior_examples(&parent_root)?;
        let (collision, collision_previous, collision_data_sha, collision_responses_sha) =
            prior_examples(&collision_root)?;
        if development.len() != 204
            || previous.len() != 204
            || collision.len() != 96
            || collision_previous.len() != 96
        {
            return Err("frozen development counts".into());
        }
        let inputs: BTreeSet<_> = training
            .iter()
            .map(|e| serde_json::to_vec(&(&e.records, &e.prompt)))
            .collect::<std::result::Result<_, _>>()?;
        if development.iter().chain(&collision).any(|e| {
            oracle(&e.records, &e.prompt).map_or(true, |o| o != e.expected)
                || serde_json::to_vec(&(&e.records, &e.prompt))
                    .map_or(true, |k| inputs.contains(&k))
        }) {
            return Err("raw development oracle or training overlap".into());
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD_AND_FIT","styled":96,"coverage":60,"predicate":48,"collision":96,"prior_success_exact":168,"collision_prior_exact":96,"sealed200_exact":200,"earlier_full_exact":6688,"earlier_unresolved":512,"training_exact":"all authored reciprocal direct byte/EOS answers","controls":{"ExactIdentity":"all300 full structures equal","reload":"all300 full structures equal","OriginalRolesRestored":"all300 complete structures equal sealed8055","QueryRefinementDisabled":"must lose actual direct-training answers with corrected source roles retained","ReadDisabled":"zero correct dependent answered responses","UpdateDisabled":"zero correct dependent answered responses"},"fits":3,"participation_fits":0,"previous_attempt_successful_full_exact":300,"second_attempt_successful_full_exact":288,"retry":"unchanged data; actual-output source-rule removal requires strict gain with zero prior correct rows lost, then constrained query witness refinement","limits":{"records":4,"record_words":16,"record_bytes":128,"query_bytes":128,"prompt_bytes":256,"clauses":2,"payload_words":3,"payload_bytes":50,"steps":96,"training_rows":8192},"scope":"authored open development and retained historical controls","final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"training":training,"construction":construction,"development":development,"collision":collision,"source_development_sha256":data_sha,"source_responses_sha256":responses_sha,"source_collision_data_sha256":collision_data_sha,"source_collision_responses_sha256":collision_responses_sha,"training_development_raw_overlap":0,"training_path_span_metadata_used":false}),
        )?;
        let parent_bytes = std::fs::read(parent_root.join("candidate.json"))?;
        if sha(&parent_bytes) != "8055a53c7801f1e6e11f808a6c15ddb090a5befe8d9d587b4ca65c74b7698793"
        {
            return Err("parent identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent = model::Artifact::decode(&parent_bytes, &g)?;
        let representation = audit(&parent, &g, &m, &training)?;
        write(&output, "representation-audit.json", &representation)?;
        if representation["pass"] != true {
            return Err("representation cannot distinguish motivating roles".into());
        }
        let (mut source, source_fit) =
            occurrence_role::fit_paired(parent.parent.parent.clone(), &g, &m, &training)?;
        std::fs::write(output.join("source-candidate.json"), source.encode()?)?;
        write(&output, "source-fit.json", &json!(source_fit))?;
        source.table = source_fit.candidate_table.clone();
        let remap = |id: u8| -> Result<u8> {
            if id >= 64 {
                return Ok(id);
            }
            let word = parent
                .parent
                .anchors
                .get(usize::from(id))
                .ok_or("old query anchor")?;
            let index = source
                .anchors
                .iter()
                .position(|w| w == word)
                .ok_or("missing exact query anchor")?;
            Ok(u8::try_from(index)?)
        };
        source.query_table = parent
            .parent
            .query_table
            .iter()
            .map(|row| {
                let mut r = row.clone();
                r.key.left = remap(r.key.left)?;
                r.key.right = remap(r.key.right)?;
                Ok(r)
            })
            .collect::<Result<Vec<_>>>()?;
        source.query_table.sort_by(|a, b| a.key.cmp(&b.key));
        source.schema = parent.parent.schema;
        source.require_available_query_coverage = parent.parent.require_available_query_coverage;
        source.validate(&g)?;
        std::fs::write(
            output.join("unselected-source-candidate.json"),
            source.encode()?,
        )?;
        let (source, source_selection) =
            occurrence_role::select_source_rules(source, &training, |roles, e| {
                let mut trial = parent.clone();
                trial.parent = roles.clone();
                trial.parent_digest = *blake3::hash(&roles.encode()?).as_bytes();
                model::generate(&trial, &g, &m, &e.records, &e.prompt, model::Control::Full)
            })?;
        write(&output, "source-selection.json", &json!(source_selection))?;
        std::fs::write(
            output.join("selected-source-candidate.json"),
            source.encode()?,
        )?;
        let (mut roles, query_fit) =
            occurrence_role::fit_queries_witnessed(source, &g, &m, &training)?;
        roles.schema = parent.parent.schema;
        roles.require_available_query_coverage = parent.parent.require_available_query_coverage;
        roles.validate(&g)?;
        write(&output, "query-fit.json", &json!(query_fit))?;
        let mut a = parent.clone();
        a.parent = roles;
        a.parent_digest = *blake3::hash(&a.parent.encode()?).as_bytes();
        a.data_digest = *blake3::hash(
            &[
                parent.data_digest.as_slice(),
                serde_json::to_vec(&training)?.as_slice(),
            ]
            .concat(),
        )
        .as_bytes();
        a.source_digest = *blake3::hash(
            concat!(
                include_str!("styled_role_report.rs"),
                include_str!("occurrence_role.rs"),
                include_str!("query_participation.rs"),
                include_str!("occurrence.rs"),
                include_str!("span_learning.rs"),
                include_str!("completion.rs"),
                include_str!("scheduling.rs")
            )
            .as_bytes(),
        )
        .as_bytes();
        a.training.push_str(" Paired reciprocal output-credit source role fit and bounded actual-output safe rule removal followed by query-role credit constrained by frozen first-pass content/context patterns and unanimous compatible witnesses. Participation anchors, optional requirements and replacement patterns are byte-identical to the prior artifact; no participation fit. Authored open-development styled combinations only.");
        a.validate(&g)?;
        let bytes = a.encode()?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        let reload = model::Artifact::decode(&bytes, &g)?;
        let mut restored = a.clone();
        restored.parent = parent.parent.clone();
        restored.parent_digest = parent.parent_digest;
        restored.validate(&g)?;
        let mut sample = vec![];
        for e in development
            .iter()
            .filter(|e| e.family.starts_with("styled"))
            .take(3)
        {
            let actual = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            sample.push(json!({"id":e.id,"assessment":assess(&a,&g,&m,e,model::Control::Full,&actual)?,"Full":actual}));
        }
        write(&output, "immediate-multi-turn.json", &json!(sample))?;
        let mut no_query_refinement = a.clone();
        no_query_refinement.parent.query_table = query_fit.initial_table.clone();
        no_query_refinement.parent_digest =
            *blake3::hash(&no_query_refinement.parent.encode()?).as_bytes();
        no_query_refinement.validate(&g)?;
        let mut refinement_disabled_correct = 0;
        let mut train_correct = 0;
        let mut train_results = vec![];
        for e in &training {
            let out = model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            let correct = out.outcome == completion::Outcome::Answered
                && !out.trace.exhausted
                && out.trace.tokens
                    == e.answer
                        .iter()
                        .copied()
                        .map(u16::from)
                        .chain([256])
                        .collect::<Vec<_>>();
            train_correct += usize::from(correct);
            let disabled = model::generate(
                &no_query_refinement,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::Full,
            )?;
            let disabled_correct = disabled.outcome == completion::Outcome::Answered
                && !disabled.trace.exhausted
                && disabled.trace.tokens
                    == e.answer
                        .iter()
                        .copied()
                        .map(u16::from)
                        .chain([256])
                        .collect::<Vec<_>>();
            refinement_disabled_correct += usize::from(disabled_correct);
            train_results.push(json!({"id":e.id,"correct":correct,"outcome":out.outcome,"tokens":out.trace.tokens,"query_refinement_disabled_correct":disabled_correct}));
        }
        write(
            &output,
            "training-responses.json",
            &json!({"rows":training.len(),"correct":train_correct,"query_refinement_disabled_correct":refinement_disabled_correct,"responses":train_results}),
        )?;
        let mut counts = BTreeMap::<String, usize>::new();
        let mut responses = vec![];
        let mut exact = 0;
        let mut reloaded = 0;
        let mut original_roles = 0;
        let mut successes = 0;
        let mut successes_exact = 0;
        let mut collision_exact = 0;
        let mut disabled_answer_correct = 0;
        for (rows, old_rows, collision_panel) in [
            (&development, &previous, false),
            (&collision, &collision_previous, true),
        ] {
            for e in rows {
                let panel = if collision_panel {
                    "collision"
                } else {
                    e.family.split('-').next().ok_or("panel")?
                };
                let matching: Vec<_> = old_rows.iter().filter(|v| v["id"] == e.id).collect();
                if matching.len() != 1 {
                    return Err("prior response identity".into());
                }
                let old = matching[0];
                let sealed: completion::Generated = serde_json::from_value(old["Full"].clone())?;
                let old_success = old["controls"]["Full"]["assessment"]["correct"] == true;
                let full =
                    model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
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
                let restored_output = model::generate(
                    &restored,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    model::Control::Full,
                )?;
                original_roles += usize::from(restored_output == sealed);
                if collision_panel {
                    collision_exact += usize::from(full == sealed);
                } else {
                    successes += usize::from(old_success);
                    successes_exact += usize::from(old_success && full == sealed);
                }
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
                    *counts.entry(format!("{panel}/{name}")).or_default() +=
                        usize::from(assessment["correct"] == true);
                    if c == model::Control::ExactIdentity {
                        exact += usize::from(out == full);
                    }
                    if matches!(
                        c,
                        model::Control::ReadDisabled | model::Control::UpdateDisabled
                    ) && e.expected.outcome == completion::Outcome::Answered
                    {
                        disabled_answer_correct +=
                            usize::from(assessment["outcome_correct"] == true);
                    }
                    controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"equal_full":out==full}));
                }
                controls.insert("OriginalRolesRestored".into(),json!({"equal_sealed_parent":restored_output==sealed,"equal_full":restored_output==full,"outcome":restored_output.outcome,"tokens":restored_output.trace.tokens}));
                responses.push(json!({"id":e.id,"panel":panel,"family":e.family,"variant":e.variant,"Full":full,"controls":controls,"previous_success":old_success,"previous_success_exact":old_success&&full==sealed}));
            }
        }
        write(&output, "responses.json", &json!(responses))?;
        let mut previous_success_exact = 0;
        let mut prior_comparison = vec![];
        for row in &responses {
            let matches: Vec<_> = prior_responses
                .iter()
                .filter(|v| v["id"] == row["id"] && v["panel"] == row["panel"])
                .collect();
            if matches.len() != 1 || matches[0]["controls"]["Full"]["assessment"]["correct"] != true
            {
                return Err("previous success identity".into());
            }
            let equal = matches[0]["Full"] == row["Full"];
            previous_success_exact += usize::from(equal);
            prior_comparison.push(json!({"id":row["id"],"panel":row["panel"],"full_exact":equal}));
        }
        write(
            &output,
            "previous-attempt.json",
            &json!({"root":prior_attempt,"rows":300,"full_exact":previous_success_exact,"items":prior_comparison}),
        )?;
        let mut second_successes = 0;
        let mut second_success_exact = 0;
        let mut second_comparison = vec![];
        for row in &second_responses {
            let matching: Vec<_> = responses
                .iter()
                .filter(|v| v["id"] == row["id"] && v["panel"] == row["panel"])
                .collect();
            if matching.len() != 1 {
                return Err("second attempt response identity".into());
            }
            let success = row["controls"]["Full"]["assessment"]["correct"] == true;
            let equal = row["Full"] == matching[0]["Full"];
            second_successes += usize::from(success);
            second_success_exact += usize::from(success && equal);
            second_comparison.push(json!({"id":row["id"],"panel":row["panel"],"previous_success":success,"full_exact":equal}));
        }
        write(
            &output,
            "second-attempt.json",
            &json!({"root":second_attempt,"successes":second_successes,"success_exact":second_success_exact,"items":second_comparison}),
        )?;
        let r200 = retained_200(&a, &parent, &g, &m, &evidence)?;
        write(&output, "retained-200.json", &r200)?;
        let r6688 = retained_6688(&a, &parent, &g, &m, &evidence)?;
        write(&output, "retained-6688.json", &r6688)?;
        report_output::verify(&parent_root)?;
        report_output::verify(&collision_root)?;
        let participation_unchanged = a.anchors == parent.anchors
            && a.optional == parent.optional
            && a.replacement == parent.replacement;
        let prior_unchanged = std::fs::read(parent_root.join("candidate.json"))? == parent_bytes;
        let pass = counts.get("styled/Full") == Some(&96)
            && counts.get("coverage/Full") == Some(&60)
            && counts.get("predicate/Full") == Some(&48)
            && counts.get("collision/Full") == Some(&96)
            && successes == 168
            && successes_exact == 168
            && collision_exact == 96
            && exact == 300
            && reloaded == 300
            && original_roles == 300
            && disabled_answer_correct == 0
            && train_correct == training.len()
            && previous_success_exact == 300
            && second_successes == 288
            && second_success_exact == 288
            && source_selection.after_correct > source_selection.before_correct
            && refinement_disabled_correct < train_correct
            && r200["pass"] == true
            && r6688["pass"] == true
            && participation_unchanged
            && prior_unchanged;
        write(
            &output,
            "summary.json",
            &json!({"gate":if pass{"PASS_LEARNED_STYLED_ROLES"}else{"FAIL_LEARNED_STYLED_ROLES"},"rows":300,"panels":counts,"prior_successes":successes,"prior_success_exact":successes_exact,"collision_prior_exact":collision_exact,"exact_equal":exact,"reload_equal":reloaded,"original_roles_restored_equal":original_roles,"read_or_update_disabled_correct_answer":disabled_answer_correct,"training_rows":training.len(),"training_correct":train_correct,"previous_attempt_full_exact":previous_success_exact,"query_refinement_disabled_training_correct":refinement_disabled_correct,"source_fit":source_fit,"source_selection":source_selection,"second_attempt_success_exact":second_success_exact,"query_fit":query_fit,"retained_200_equal":r200["equal"],"retained_6688_equal":r6688["equal"],"retained_typed_unresolved":r6688["typed_unresolved"],"participation_parameters_unchanged":participation_unchanged,"prior_artifact_unchanged":prior_unchanged,"candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"fits":3,"source_selection_fits":1,"source_role_fits":1,"query_role_fits":1,"participation_fits":0,"final_holdout":"NOT_RUN","promotion":false,"default_model":"15baec48 unchanged"}),
        )?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"gate":if pass{"PASS_LEARNED_STYLED_ROLES"}else{"FAIL_LEARNED_STYLED_ROLES"},"path":output})
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
