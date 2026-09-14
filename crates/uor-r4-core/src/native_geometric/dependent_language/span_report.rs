//! Bounded direct-to-dependent exact-span qualification. Full traces are sampled;
//! per-row records retain only the response and decision-bearing identities.
use super::{
    completion, completion_data, completion_report, depth_data, schedule_data, scheduling,
    span as runtime, span_data as data, span_learning as learning,
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
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
fn write(path: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn source_path(out: &completion::Generated) -> Vec<[usize; 2]> {
    out.decisions
        .iter()
        .filter(|d| d.before.core.cursor == 0)
        .filter_map(|d| d.selected)
        .collect()
}
fn intervals(bytes: &[u8]) -> Vec<[usize; 2]> {
    let mut result = Vec::new();
    let mut start = None;
    for (i, b) in bytes
        .iter()
        .copied()
        .chain(std::iter::once(b'.'))
        .enumerate()
    {
        if b.is_ascii_alphabetic() {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(first) = start.take() {
            result.push([first, i]);
        }
    }
    result
}
fn actual_extent(
    out: &completion::Generated,
    records: &[Vec<u8>; 4],
) -> (Option<[usize; 3]>, Option<[usize; 2]>) {
    let selected = out
        .trace
        .steps
        .iter()
        .rev()
        .find_map(|step| step.observation.route.selected.as_ref());
    let Some(v) = selected else {
        return (None, None);
    };
    let words = intervals(&records[v.source]);
    let first = words.iter().position(|w| w[0] == v.start);
    let last = words.iter().position(|w| w[1] == v.end);
    (
        first.zip(last).map(|(a, b)| [v.source, a, b + 1]),
        Some([v.start, v.end]),
    )
}
fn target_matches(out: &completion::Generated, e: &data::Example) -> bool {
    out.trace.tokens == learning::target(e)
        && !out.trace.exhausted
        && out.outcome == completion::Outcome::Answered
}
fn compact(out: &completion::Generated, records: &[Vec<u8>; 4]) -> Value {
    let (span, bounds) = actual_extent(out, records);
    json!({"tokens":out.trace.tokens,"exhausted":out.trace.exhausted,"outcome":out.outcome,
        "source_path":source_path(out),"actual_word_span":span,"last_selected_bounds":bounds})
}
pub(super) fn retention(
    candidate: &runtime::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    evidence: &Path,
) -> Result<Value, Box<dyn std::error::Error>> {
    let legacy = completion_report::legacy_retention(&candidate.parent.parent, g, m, evidence)?;
    let mut panels = Vec::new();
    let mut transferred = 0;
    for (previous, required) in [("language-scheduling-1", 1536), ("language-depth-1", 1280)] {
        let root = evidence.join(previous).join("attempt-1");
        report_output::verify(&root)?;
        let pd: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
        let cases: Vec<(String, [Vec<u8>; 4], Vec<u8>)> = if previous == "language-depth-1" {
            serde_json::from_value::<Vec<depth_data::Example>>(pd["development"].clone())?
                .into_iter()
                .map(|e| (e.id, e.records, e.prompt))
                .collect()
        } else {
            serde_json::from_value::<Vec<schedule_data::Example>>(pd["development"].clone())?
                .into_iter()
                .map(|e| (e.id, e.records, e.prompt))
                .collect()
        };
        let mut items = Vec::new();
        let mut equal = 0;
        for (id, records, prompt) in cases {
            let old = scheduling::generate(
                &candidate.parent.parent,
                g,
                m,
                &records,
                &prompt,
                scheduling::Control::Full,
            )?;
            let parent = completion::generate(
                &candidate.parent,
                g,
                m,
                &records,
                &prompt,
                completion::Control::Full,
            )?;
            let new = match runtime::generate(
                candidate,
                g,
                m,
                &records,
                &prompt,
                runtime::Control::Full,
            ) {
                Ok(out) => out,
                Err(error) => {
                    items.push(json!({"id":id,"full_core_trace_and_completion_decisions_equal":false,"error":error.to_string()}));
                    continue;
                }
            };
            let same =
                new == parent && new.trace == old && new.outcome == completion::Outcome::Answered;
            equal += usize::from(same);
            items.push(json!({"id":id,"full_core_trace_and_completion_decisions_equal":same}));
        }
        panels.push(json!({"previous":previous,"rows":items.len(),"equal":equal,"required":required,"items":items}));
        transferred += equal;
    }
    let root = evidence.join("language-completion-1/attempt-1");
    report_output::verify(&root)?;
    let pd: Value = serde_json::from_slice(&std::fs::read(root.join("data.json"))?)?;
    let cases: Vec<completion_data::Example> = serde_json::from_value(pd["development"].clone())?;
    // Drop historical full decision frames after compacting the exact Full rows.
    let frozen: BTreeMap<String, Value> = {
        let rows: Vec<Value> =
            serde_json::from_slice(&std::fs::read(root.join("responses.json"))?)?;
        let mut found = BTreeMap::new();
        for row in rows
            .into_iter()
            .filter(|r| r["artifact"] == "candidate" && r["control"] == "Full")
        {
            let id = row["id"]
                .as_str()
                .ok_or("completion response ID")?
                .to_string();
            if found
                .insert(
                    id,
                    json!({"tokens":row["tokens"],"exhausted":row["exhausted"],
                "outcome":row["outcome"],"source_path":row["resolved_prefix"]}),
                )
                .is_some()
            {
                return Err("duplicate frozen completion Full response".into());
            }
        }
        found
    };
    let mut equal = 0;
    let mut frozen_equal = 0;
    let mut answered = 0;
    let mut unresolved = 0;
    let mut items = Vec::new();
    for e in &cases {
        let before = completion::generate(
            &candidate.parent,
            g,
            m,
            &e.records,
            &e.prompt,
            completion::Control::Full,
        )?;
        let after = match runtime::generate(
            candidate,
            g,
            m,
            &e.records,
            &e.prompt,
            runtime::Control::Full,
        ) {
            Ok(out) => out,
            Err(error) => {
                items.push(json!({"id":e.id,"full_core_trace_and_completion_decisions_equal":false,"sealed_response_equal":false,"error":error.to_string()}));
                continue;
            }
        };
        let actual = json!({"tokens":after.trace.tokens,"exhausted":after.trace.exhausted,
            "outcome":after.outcome,"source_path":source_path(&after)});
        let same = before == after;
        let sealed = frozen.get(&e.id).ok_or("missing frozen completion row")? == &actual;
        equal += usize::from(same);
        frozen_equal += usize::from(sealed);
        answered += usize::from(same && matches!(after.outcome, completion::Outcome::Answered));
        unresolved +=
            usize::from(same && matches!(after.outcome, completion::Outcome::Unresolved { .. }));
        items.push(json!({"id":e.id,"full_core_trace_and_completion_decisions_equal":same,"sealed_response_equal":sealed}));
    }
    transferred += equal;
    let transfer_ok = panels
        .iter()
        .all(|p| p["rows"] == p["required"] && p["equal"] == p["required"])
        && cases.len() == 896
        && frozen.len() == 896
        && equal == 896
        && frozen_equal == 896
        && answered == 384
        && unresolved == 512;
    panels.push(json!({"previous":"language-completion-1","rows":cases.len(),"equal":equal,
        "sealed_response_equal":frozen_equal,"answered":answered,"unresolved":unresolved,"items":items}));
    Ok(
        json!({"legacy_control_replay":legacy,"new_full_transfer":{"equal":transfer_ok,"rows":3712,
        "equal_rows":transferred,"panels":panels},
        "scope":"legacy controls replay through unchanged parent adapters; new span Full compares all core traces and completion decisions, including typed unresolved outcomes; historical completion Full responses independently compared to sealed evidence"}),
    )
}
#[test]
#[ignore = "Refine learned boundary evidence with frozen span rules; exclusive report"]
fn typed_language_span_report() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_SPAN_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_LANGUAGE_SPAN_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","artifact_schema":2,
            "boundary_learning":"intersect words outside every actual answer/EOS-compatible candidate source span per example; union those intersections and remove every training answer word; unknown words remain unclassified",
            "candidate_rules":"up to8 positive-conjunction rules,1..5of19 predicates; all1..3word intervals remain candidates",
            "training_rows":2624,"new_training_rows":576,"retained_direct_training_rows":2048,"training_depth":1,"development_rows":1728,"full_answers":1728,"full_paths":1728,"full_word_spans":1728,"full_byte_bounds":1728,
            "development_families":48,"rows_per_development_family":36,"initial_span_rules":[327682],"rule_fit":"NOT_RUN","boundary_refinement":1,
            "missing_training_targets_allowed":0,"ambiguous_training_targets":"DESCRIPTIVE; latent occurrences may emit the same final answer","all_positive_candidates_dominated_rows_allowed":0,
            "controls":{"ReadDisabled":0,"ScorerDisabled":0,"CursorDisabled":0,"StopDisabled":0,
                "SingleWordOnly":{"one_word_correct":576,"multiword_correct":0},
                "PayloadFirstWord":{"one_word_correct":576,"multiword_correct":0},
                "ExtentDisabled":{"multiword_correct":0,"one_word_correct":"DESCRIPTIVE; removing extent can make singleton starts ambiguous"},
                "BoundaryContextDisabled":{"styled_subject_multiword_rows":288,"correct_less_than":288,"other_cells":"DESCRIPTIVE"},
                "LocalBoundaryDisabled":{"multiword_correct_less_than":1152,"other_cells":"DESCRIPTIVE"},
                "ExactIdentity":1728},
            "reload_rows":1728,"completion_updater_writer_unchanged":true,
            "new_full_transfer":{"completion":896,"scheduling":1536,"depth":1280},
            "legacy_controls":"original adapters; separately scoped from new Full transfer",
            "split":"name offsets0-3 direct training;4-7 direct/two/three-read development; whole families, familiar overlapping vocabulary",
            "scope":"bounded authored composed-name spans and retained query protocol; not natural prose or held-out generalization",
            "final_holdout":"NOT_RUN","promotion":false}),
        )?;
        let all = data::corpus()?;
        let (new_train, dev) = data::split(&all);
        if new_train.len() != 576 || dev.len() != 1728 {
            return Err("span corpus split changed".into());
        }
        let train = data::training_with_retained(&new_train)?;
        let retained_root = evidence.join("relative-language-1/attempt-2");
        report_output::verify(&retained_root)?;
        let retained_bytes = std::fs::read(retained_root.join("data.json"))?;
        let retained_data: Value = serde_json::from_slice(&retained_bytes)?;
        let (retained_train, _) = crate::native_geometric::relative_language::data::corpus()?;
        if serde_json::to_value(&retained_train)? != retained_data["training"] {
            return Err("retained direct training differs from sealed evidence".into());
        }
        let prior_root = evidence.join("language-span-1/attempt-1");
        report_output::verify(&prior_root)?;
        let prior_bytes = std::fs::read(prior_root.join("data.json"))?;
        let prior_data: Value = serde_json::from_slice(&prior_bytes)?;
        if serde_json::to_value(&train)? != prior_data["training"]
            || serde_json::to_value(&dev)? != prior_data["development"]
            || serde_json::to_value(&new_train)? != prior_data["new_training"]
            || serde_json::to_value(&retained_train)? != prior_data["retained_training"]
        {
            return Err("corpus changed after the sealed representation diagnostic".into());
        }
        write(
            &output,
            "data.json",
            &json!({"training":train,"new_training":new_train,
            "retained_training":retained_train,"development":dev,"validation":data::validate(&all)?,
            "frozen_representation_diagnostic":{"report_root":prior_root,"data_sha256":sha(&prior_bytes),"corpus_equal":true},
            "retained_training_provenance":{"report_root":retained_root,"data_sha256":sha(&retained_bytes),
                "training_sha256":sha(&serde_json::to_vec(&retained_train)?),"sealed_training_equal":true},
            "training_counts":{"new_direct":576,"retained_direct":2048,"total":2624},
            "scope":"only earlier direct training examples retained; no earlier development labels train span rules"}),
        )?;
        let parent_root = evidence.join("language-completion-1/attempt-1");
        report_output::verify(&parent_root)?;
        let parent_bytes = std::fs::read(parent_root.join("candidate.json"))?;
        if sha(&parent_bytes) != "f9e1f5aeef918d0fe6b463cd75986847d551aa8049c9f1c85b1d6d46c55a1e1f"
        {
            return Err("frozen completion parent hash differs".into());
        }
        let geometry = BoundGeometry::canonical()?;
        let metric = Metric::new(&geometry)?;
        let parent = completion::Artifact::decode(&parent_bytes, &geometry)?;
        let previous_root = evidence.join("language-span-1/attempt-2");
        report_output::verify(&previous_root)?;
        let initial_bytes = std::fs::read(previous_root.join("candidate.json"))?;
        if sha(&initial_bytes) != "ddf9bea88413c458f3ba45c4cd0f0e89eb2089b0ed84f194ede798a9c195812e"
        {
            return Err("frozen negative span candidate hash differs".into());
        }
        let initial = runtime::Artifact::decode(&initial_bytes, &geometry)?;
        if initial.rules != [327682] || initial.parent.encode()? != parent_bytes {
            return Err("negative candidate rules or retained completion parent differ".into());
        }
        std::fs::write(output.join("initial.json"), &initial_bytes)?;
        let original_prepared = learning::prepare(&initial, &geometry, &metric, &train)?;
        let mut context_evidence = BTreeSet::new();
        let mut answer_words = BTreeSet::new();
        let mut per_example = Vec::new();
        for (e, p) in train.iter().zip(&original_prepared) {
            for [a, b] in intervals(&e.answer) {
                answer_words.insert(e.answer[a..b].to_vec());
            }
            if p.candidates.len() != p.compatible.len() {
                return Err("candidate compatibility shape differs".into());
            }
            let mut intersection: Option<BTreeSet<Vec<u8>>> = None;
            let mut compatible_count = 0;
            for (candidate, yes) in p.candidates.iter().zip(&p.compatible) {
                if !*yes {
                    continue;
                }
                compatible_count += 1;
                let source = &e.records[candidate.value.source];
                let outside: BTreeSet<_> = intervals(source)
                    .into_iter()
                    .filter(|[a, b]| *b <= candidate.value.start || *a >= candidate.value.end)
                    .map(|[a, b]| source[a..b].to_vec())
                    .collect();
                intersection = Some(match intersection {
                    None => outside,
                    Some(previous) => previous.intersection(&outside).cloned().collect(),
                });
            }
            let evidence = intersection.unwrap_or_default();
            per_example.push(json!({"id":e.id,"answer_compatible_candidates":compatible_count,"outside_intersection":evidence}));
            context_evidence.extend(evidence);
        }
        let expected_context: Vec<_> = context_evidence
            .difference(&answer_words)
            .cloned()
            .collect();
        let (candidate, prepared) =
            learning::refine_boundary(&initial, &geometry, &metric, &train, &original_prepared)?;
        if prepared.len() != original_prepared.len() {
            return Err("boundary refinement changed prepared example count".into());
        }
        for (before, after) in original_prepared.iter().zip(&prepared) {
            if before.compatible != after.compatible
                || before.candidates.len() != after.candidates.len()
                || before
                    .candidates
                    .iter()
                    .zip(&after.candidates)
                    .any(|(old, new)| old.value != new.value || old.last_word != new.last_word)
            {
                return Err("boundary refinement changed candidate payload, order or actual output compatibility".into());
            }
        }
        drop(original_prepared);
        let actual_context: Vec<_> = candidate
            .context_words
            .iter()
            .map(|word| word.bytes.clone())
            .collect();
        if actual_context != expected_context {
            return Err(
                "refined context words differ from compatible latent-source intersections".into(),
            );
        }
        let rules_unchanged = candidate.rules == initial.rules;
        if !rules_unchanged {
            return Err("boundary refinement changed frozen selection rules".into());
        }
        let bytes = candidate.encode()?;
        std::fs::write(output.join("candidate.json"), &bytes)?;
        write(
            &output,
            "boundary-learning.json",
            &json!({"boundary_refinement":1,"rule_fit":"NOT_RUN",
            "initial_candidate_sha256":sha(&initial_bytes),"initial_rules":initial.rules,"rules":candidate.rules,"rules_unchanged":rules_unchanged,
            "initial_context_word_count":initial.context_words.len(),"initial_context_words":initial.context_words,
            "context_word_count":candidate.context_words.len(),"context_words":candidate.context_words,
            "per_example_evidence":per_example,"outside_intersection_vocabulary_count":context_evidence.len(),"answer_vocabulary_count":answer_words.len(),
            "context_evidence_exact":true,"positive_answer_usage_overrides_context":true,"unknown_words":"unclassified",
            "training_sha256":sha(&serde_json::to_vec(&train)?),"context_words_sha256":sha(&serde_json::to_vec(&candidate.context_words)?),
            "candidate_payload_order_and_compatibility_unchanged":true,
            "inputs":"raw training records and actual final answer/EOS-compatible latent source spans; no gold expected span/path labels or development evidence",
            "scope":"finite observed word-role evidence; irrelevant records supply no negative evidence; not a universal lexical category"}),
        )?;
        let preparation: Vec<_> = train.iter().zip(&prepared).map(|(e, p)| json!({"id":e.id,
            "candidates":p.candidates.len(),"answer_eos_compatible_candidates":p.compatible.iter().filter(|&&x| x).count()})).collect();
        let missing = prepared
            .iter()
            .filter(|p| !p.compatible.iter().any(|&x| x))
            .count();
        let ambiguous = prepared
            .iter()
            .filter(|p| p.compatible.iter().filter(|&&x| x).count() > 1)
            .count();
        let mut dominated_rows = 0;
        let mut witnesses = Vec::new();
        for (e, p) in train.iter().zip(&prepared) {
            let positives: Vec<_> = p
                .candidates
                .iter()
                .zip(&p.compatible)
                .filter(|(_, yes)| **yes)
                .map(|(v, _)| v)
                .collect();
            let dominated: Vec<_> = positives
                .iter()
                .map(|positive| {
                    p.candidates
                        .iter()
                        .zip(&p.compatible)
                        .find(|(negative, yes)| {
                            !**yes && negative.features & positive.features == positive.features
                        })
                        .map(|(negative, _)| (*positive, negative))
                })
                .collect();
            if !positives.is_empty() && dominated.iter().all(Option::is_some) {
                dominated_rows += 1;
                if witnesses.len() < 8 {
                    witnesses.push(json!({"id":e.id,"records":e.records,"prompt":e.prompt,"answer":e.answer,
                        "pairs":dominated.into_iter().flatten().map(|(positive, negative)| json!({
                            "positive":{"source":positive.value.source,"first_word":positive.value.word,"last_word":positive.last_word,
                                "bounds":[positive.value.start,positive.value.end],"bytes":positive.value.bytes,"features":positive.features},
                            "negative":{"source":negative.value.source,"first_word":negative.value.word,"last_word":negative.last_word,
                                "bounds":[negative.value.start,negative.value.end],"bytes":negative.value.bytes,"features":negative.features}
                        })).collect::<Vec<_>>()}));
                }
            }
        }
        write(
            &output,
            "preparation.json",
            &json!({"rows":preparation,"missing_target_rows":missing,
            "ambiguous_target_rows":ambiguous,"all_positive_candidates_dominated_rows":dominated_rows,"domination_witnesses":witnesses,
            "domination_test":"same-example incorrect candidate feature superset of every correct candidate prevents unique correct selection by any positive-conjunction rule union",
            "training_targets":"final answer bytes and EOS via actual retained writer; no source/span labels"}),
        )?;
        if missing != 0 || dominated_rows != 0 {
            write(
                &output,
                "summary.json",
                &json!({"gate":"FAIL_TYPED_LANGUAGE_SPAN_REPRESENTATION",
                "rule_fit":"NOT_RUN","boundary_refinement":1,"candidate_sha256":sha(&bytes),"initial_candidate_sha256":sha(&initial_bytes),"artifact_schema":2,"context_word_count":candidate.context_words.len(),"corpus_unchanged_since_representation_diagnostic":true,"missing_target_rows":missing,"ambiguous_target_rows":ambiguous,"all_positive_candidates_dominated_rows":dominated_rows,"promotion":false}),
            )?;
            return Ok(());
        }
        let fit =
            learning::evaluate_frozen_rules(&candidate, &geometry, &metric, &train, &prepared)?;
        drop(prepared);
        write(&output, "fit.json", &serde_json::to_value(&fit)?)?;
        let unchanged = candidate.parent.encode()? == parent_bytes;
        let reload = runtime::Artifact::decode(&bytes, &geometry)?;
        let mut bad = candidate.clone();
        bad.rules.push(1 << runtime::FEATURES);
        if bad.validate(&geometry).is_ok() {
            return Err("invalid rule bits accepted".into());
        }
        let mut bad = candidate.clone();
        bad.parent_digest[0] ^= 1;
        if bad.validate(&geometry).is_ok() {
            return Err("invalid parent digest accepted".into());
        }
        let mut malformed_context = candidate.clone();
        let prefix = malformed_context
            .context_words
            .first_mut()
            .and_then(|word| word.geometry.prefixes.first_mut())
            .ok_or("missing learned context geometry for malformed-artifact check")?;
        prefix[0] ^= 1;
        if malformed_context.validate(&geometry).is_ok() {
            return Err("malformed learned context geometry accepted".into());
        }
        let mut gate = fit.training_exact == 2624
            && fit.missing_target_rows == 0
            && unchanged
            && rules_unchanged
            && fit.proposals == 0;
        let mut responses = Vec::new();
        let mut panels = Vec::new();
        let mut families = BTreeMap::<String, usize>::new();
        let mut samples = BTreeSet::new();
        let mut reload_equal = 0;
        for control in [
            runtime::Control::Full,
            runtime::Control::ReadDisabled,
            runtime::Control::ScorerDisabled,
            runtime::Control::CursorDisabled,
            runtime::Control::StopDisabled,
            runtime::Control::SingleWordOnly,
            runtime::Control::PayloadFirstWord,
            runtime::Control::ExtentDisabled,
            runtime::Control::BoundaryContextDisabled,
            runtime::Control::LocalBoundaryDisabled,
            runtime::Control::ExactIdentity,
        ] {
            let mut correct = 0;
            let mut paths = 0;
            let mut spans = 0;
            let mut bounds = 0;
            let mut one = 0;
            let mut multi = 0;
            let mut errors = 0;
            let mut styled_subject_multiword = 0;
            for e in &dev {
                let out = match runtime::generate(
                    &candidate, &geometry, &metric, &e.records, &e.prompt, control,
                ) {
                    Ok(out) => out,
                    Err(error) => {
                        errors += 1;
                        responses.push(json!({"id":e.id,"family":e.family,"depth":e.depth,"variant":e.variant,
                            "artifact":"candidate","control":control,"correct":false,"error":error.to_string()}));
                        continue;
                    }
                };
                let yes = target_matches(&out, e);
                let path_ok = source_path(&out) == e.expected_path;
                let (span, bound) = actual_extent(&out, &e.records);
                let span_ok = span == Some(e.expected_span);
                let bounds_ok = bound == Some(e.expected_bounds);
                let length = e.expected_span[2] - e.expected_span[1];
                correct += usize::from(yes);
                paths += usize::from(path_ok);
                spans += usize::from(span_ok);
                bounds += usize::from(bounds_ok);
                one += usize::from(yes && length == 1);
                multi += usize::from(yes && length > 1);
                styled_subject_multiword +=
                    usize::from(yes && length > 1 && e.family.contains("-v1-s1-"));
                if control == runtime::Control::Full {
                    let loaded = runtime::generate(
                        &reload, &geometry, &metric, &e.records, &e.prompt, control,
                    )?;
                    reload_equal += usize::from(out == loaded);
                    if out
                        .trace
                        .steps
                        .windows(2)
                        .any(|p| p[0].after != p[1].before)
                    {
                        return Err("span generation state continuity differs".into());
                    }
                    if yes && path_ok && span_ok && bounds_ok {
                        *families.entry(e.family.clone()).or_default() += 1;
                    }
                    let subject = e.family.contains("-v1-");
                    if e.variant == "baseline" && samples.insert((e.depth, length, subject)) {
                        write(
                            &output,
                            &format!(
                                "sample-depth{}-words{length}-subject{subject}.json",
                                e.depth
                            ),
                            &json!({"example":e,"generated":out}),
                        )?;
                    }
                }
                let mut record = compact(&out, &e.records);
                let fields = record.as_object_mut().ok_or("compact response object")?;
                fields.insert("id".into(), json!(e.id));
                fields.insert("family".into(), json!(e.family));
                fields.insert("depth".into(), json!(e.depth));
                fields.insert("variant".into(), json!(e.variant));
                fields.insert("artifact".into(), json!("candidate"));
                fields.insert("control".into(), json!(control));
                fields.insert("correct".into(), json!(yes));
                fields.insert("path_correct".into(), json!(path_ok));
                fields.insert("word_span_correct".into(), json!(span_ok));
                fields.insert("byte_bounds_correct".into(), json!(bounds_ok));
                responses.push(record);
            }
            panels.push(json!({"control":control,"rows":dev.len(),"correct":correct,"paths":paths,"word_spans":spans,"byte_bounds":bounds,
                "one_word_correct":one,"multiword_correct":multi,"errors":errors,
                "styled_subject_multiword_correct":styled_subject_multiword,"styled_subject_multiword_rows":288}));
            gate &= match control {
                runtime::Control::Full => {
                    correct == 1728 && paths == 1728 && spans == 1728 && bounds == 1728
                }
                runtime::Control::ExactIdentity => correct == 1728,
                runtime::Control::SingleWordOnly | runtime::Control::PayloadFirstWord => {
                    one == 576 && multi == 0
                }
                runtime::Control::ExtentDisabled => multi == 0,
                runtime::Control::BoundaryContextDisabled => styled_subject_multiword < 288,
                runtime::Control::LocalBoundaryDisabled => multi < 1152,
                _ => correct == 0,
            };
        }
        let complete_families = families.values().filter(|&&n| n == 36).count();
        gate &= families.len() == 48 && complete_families == 48 && reload_equal == 1728;
        write(&output, "responses.json", &json!(responses))?;
        let mut baseline_correct = 0;
        let mut baseline_rows = Vec::new();
        let mut baseline_errors = 0;
        for e in &dev {
            let out = match completion::generate(
                &parent,
                &geometry,
                &metric,
                &e.records,
                &e.prompt,
                completion::Control::Full,
            ) {
                Ok(out) => out,
                Err(error) => {
                    baseline_errors += 1;
                    baseline_rows
                        .push(json!({"id":e.id,"correct":false,"error":error.to_string()}));
                    continue;
                }
            };
            let yes = target_matches(&out, e);
            baseline_correct += usize::from(yes);
            baseline_rows
                .push(json!({"id":e.id,"correct":yes,"response":compact(&out, &e.records)}));
        }
        write(
            &output,
            "parent-baseline.json",
            &json!({"scope":"descriptive old reader on the new span corpus; not a frozen acceptance threshold",
            "rows":baseline_rows,"correct":baseline_correct,"errors":baseline_errors}),
        )?;
        let retained = retention(&candidate, &geometry, &metric, &evidence)?;
        gate &= retained["legacy_control_replay"]["equal"] == true
            && retained["new_full_transfer"]["equal"] == true;
        write(&output, "retained-responses.json", &retained)?;
        write(
            &output,
            "summary.json",
            &json!({"gate":if gate {"PASS_TYPED_LANGUAGE_SPAN"} else {"FAIL_TYPED_LANGUAGE_SPAN"},
            "artifact_schema":2,"corpus_unchanged_since_representation_diagnostic":true,
            "context_word_count":candidate.context_words.len(),"rules_unchanged_since_negative_candidate":rules_unchanged,"rule_fit":"NOT_RUN","boundary_refinement":1,
            "initial_candidate_sha256":sha(&initial_bytes),"initial_context_word_count":initial.context_words.len(),
            "context_words_sha256":sha(&serde_json::to_vec(&candidate.context_words)?),
            "training_exact":fit.training_exact,"training_rows":train.len(),"development_rows":dev.len(),"initial_rules":initial.rules,"rules":candidate.rules,
            "panels":panels,"complete_families":complete_families,"reload_equal":reload_equal,"parent_baseline_correct":baseline_correct,
            "candidate_sha256":sha(&bytes),"parent_sha256":sha(&parent_bytes),"parent_parameters_unchanged":unchanged,
            "legacy_control_replay_equal":retained["legacy_control_replay"]["equal"],"legacy_control_replay_counts":retained["legacy_control_replay"]["counts"],
            "new_full_transfer_equal":retained["new_full_transfer"]["equal"],"new_full_transfer_rows":retained["new_full_transfer"]["equal_rows"],
            "final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
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
