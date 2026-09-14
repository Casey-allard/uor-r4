//! Frozen-family collision diagnostic. Only candidates reachable by the parent
//! rule can be reached by its supersets, so other candidates need no model call.
use super::{completion, occurrence, scheduling, span, span_data::Example, span_learning::target};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry,
        hamming_refinement::metric::Metric,
        language_relation::runtime::{Route, RouteStatus},
    },
    report_output,
};
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
fn bit(signature: &[u64], index: usize) -> bool {
    signature[index / 64] & (1u64 << (index % 64)) != 0
}
#[derive(Default)]
struct Pair {
    positive: Option<Value>,
    negative: Option<Value>,
    positive_count: usize,
    negative_count: usize,
}
struct Row {
    id: String,
    positive: Vec<Vec<u64>>,
    eligible: usize,
    zero: usize,
}
#[test]
#[ignore = "bounded actual-artifact proposal collision diagnostic"]
fn occurrence_proposal_collision_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_OCCURRENCE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty collision report path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let source = evidence.join("language-occurrence-1/attempt-2");
        report_output::verify(&source)?;
        let fit: Value = serde_json::from_slice(&std::fs::read(source.join("fit.json"))?)?;
        let options: Vec<u32> = serde_json::from_value(fit["proposal_masks"].clone())?;
        if options.len() != 137
            || options[0] != 327682
            || options
                .iter()
                .any(|r| r & 327682 != 327682 || r.count_ones() > 5)
        {
            return Err("frozen proposal family differs".into());
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","training_rows":2816,"proposal_masks":options,"fits":0,"only_force_parent_rule_reachable_candidates":true,"zero_signature_candidate_compatibility":"NOT_EVALUATED; unreachable by every declared proposal","scope":"exhaustive observed candidate-admission signatures over this training set and137 frozen monotone proposals, with zero incompatible admission policy; not a general geometric impossibility claim","first_negative_per_proposal":true,"nonzero_signature_positive_negative_pairs":true,"promotion":false}),
        )?;
        let train_bytes = std::fs::read(source.join("training-data.json"))?;
        let train_json: Value = serde_json::from_slice(&train_bytes)?;
        let train: Vec<Example> = serde_json::from_value(train_json["training"].clone())?;
        if train.len() != 2816
            || train.iter().any(|e| {
                scheduling::clauses(&e.prompt)
                    .map(|q| q.len() != 1)
                    .unwrap_or(true)
            })
        {
            return Err("frozen training rows/window differs".into());
        }
        write(
            &output,
            "inputs.json",
            &json!({"training_data_sha256":sha(&train_bytes),"fit_sha256":sha(&std::fs::read(source.join("fit.json"))?),"training_rows":train.len(),"source":source,"training":"existing frozen training-data.json; no new examples"}),
        )?;
        let parent = evidence.join("language-span-1/attempt-3");
        report_output::verify(&parent)?;
        let bytes = std::fs::read(parent.join("candidate.json"))?;
        if sha(&bytes) != "44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b" {
            return Err("parent artifact hash differs".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = span::Artifact::decode(&bytes, &g)?;
        if a.rules != [327682] {
            return Err("parent rule differs".into());
        }
        let mut signatures = BTreeMap::<Vec<u64>, Pair>::new();
        let mut rows = Vec::new();
        let mut negatives = BTreeMap::<u32, Value>::new();
        let mut proposal_positive = vec![0usize; options.len()];
        let mut proposal_negative = vec![0usize; options.len()];
        let mut enumerated = 0;
        let mut forced = 0;
        let mut zero_count = 0;
        let mut max_nodes = 0;
        for e in &train {
            let search = occurrence::candidates(
                &a,
                &g,
                &m,
                &e.records,
                &e.prompt,
                occurrence::Control::Full,
            )?;
            max_nodes = max_nodes.max(search.nodes);
            let mut row = Row {
                id: e.id.clone(),
                positive: vec![],
                eligible: 0,
                zero: 0,
            };
            for candidate in search.candidates {
                enumerated += 1;
                if !candidate
                    .witnesses
                    .iter()
                    .any(|w| w.features & 327682 == 327682)
                {
                    row.zero += 1;
                    zero_count += 1;
                    continue;
                }
                row.eligible += 1;
                forced += 1;
                let mut signature = vec![0u64; options.len().div_ceil(64)];
                let mut admitted = Vec::new();
                for (i, &rule) in options.iter().enumerate() {
                    if candidate
                        .witnesses
                        .iter()
                        .any(|w| w.features & rule == rule)
                    {
                        signature[i / 64] |= 1u64 << (i % 64);
                        admitted.push(rule);
                    }
                }
                let v = &candidate.span.value;
                let generated = completion::generate_routed(
                    &a.parent,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    completion::Control::Full,
                    scheduling::Control::Full,
                    |_, _| {
                        Ok(Route {
                            status: RouteStatus::Selected,
                            selected: Some(v.clone()),
                            compatible: vec![[v.source, v.word]],
                        })
                    },
                )?;
                let compatible = generated.trace.tokens == target(e)
                    && !generated.trace.exhausted
                    && generated.outcome == completion::Outcome::Answered;
                let mut complete_features: Vec<_> =
                    candidate.witnesses.iter().map(|w| w.features).collect();
                complete_features.sort_unstable();
                complete_features.dedup();
                let sample = json!({"id":e.id,"records":e.records,"prompt":e.prompt,"expected_answer_evaluation_only":e.answer,"candidate":candidate,"complete_witness_feature_set":complete_features,"actual_tokens":generated.trace.tokens,"actual_outcome":generated.outcome,"exhausted":generated.trace.exhausted,"compatible":compatible,"admitted_by_proposals":admitted,"signature":signature});
                for (i, &rule) in options
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| bit(&signature, *i))
                {
                    if compatible {
                        proposal_positive[i] += 1;
                    } else {
                        proposal_negative[i] += 1;
                        negatives.entry(rule).or_insert_with(|| sample.clone());
                    }
                }
                let pair = signatures.entry(signature.clone()).or_default();
                if compatible {
                    pair.positive_count += 1;
                    if pair.positive.is_none() {
                        pair.positive = Some(sample);
                    }
                    row.positive.push(signature);
                } else {
                    pair.negative_count += 1;
                    if pair.negative.is_none() {
                        pair.negative = Some(sample);
                    }
                }
            }
            rows.push(row);
        }
        let safe: Vec<_> = options
            .iter()
            .enumerate()
            .filter(|(_, r)| !negatives.contains_key(*r))
            .map(|(i, _)| i)
            .collect();
        let mut all_conflicted = 0;
        let mut no_safe = 0;
        let mut sole_conflicted = 0;
        let mut row_reports = Vec::new();
        for row in &rows {
            let conflicting = row
                .positive
                .iter()
                .filter(|s| signatures.get(*s).is_some_and(|p| p.negative_count > 0))
                .count();
            let all = !row.positive.is_empty() && conflicting == row.positive.len();
            let safely_reachable = row.positive.iter().any(|s| safe.iter().any(|i| bit(s, *i)));
            all_conflicted += usize::from(all);
            no_safe += usize::from(!safely_reachable);
            sole_conflicted += usize::from(all && row.positive.len() == 1);
            row_reports.push(json!({"id":row.id,"reachable_candidates":row.eligible,"zero_signature_candidates_not_executed":row.zero,"compatible_reachable_candidates":row.positive.len(),"compatible_reachable_signatures":row.positive,"conflicting_compatible_reachable_candidates":conflicting,"all_reachable_targets_have_conflicting_signature":all,"has_any_positive_candidate_admitted_by_globally_safe_proposal":safely_reachable,"sole_reachable_target_conflicted":all&&row.positive.len()==1}));
        }
        let pairs:Vec<_>=signatures.iter().filter(|(_,p)|p.positive_count>0&&p.negative_count>0).map(|(signature,p)|json!({"signature":signature,"nonzero":signature.iter().any(|x|*x!=0),"compatible_candidates":p.positive_count,"incompatible_candidates":p.negative_count,"complete_witness_feature_sets_equal":p.positive.as_ref().map(|v|&v["complete_witness_feature_set"])==p.negative.as_ref().map(|v|&v["complete_witness_feature_set"]),"positive":p.positive,"negative":p.negative})).collect();
        let proposal_rows:Vec<_>=options.iter().enumerate().map(|(i,r)|json!({"proposal":r,"active_features":span::feature_names().iter().enumerate().filter(|(j,_)|r&(1u32<<j)!=0).map(|(_,n)|n).collect::<Vec<_>>(),"compatible_candidates_admitted":proposal_positive[i],"incompatible_candidates_admitted":proposal_negative[i],"globally_safe_on_training":proposal_negative[i]==0,"first_rejecting_candidate":negatives.get(r)})).collect();
        write(&output, "proposal-rejections.json", &json!(proposal_rows))?;
        write(&output, "nonzero-conflict-pairs.json", &json!(pairs))?;
        write(&output, "row-collisions.json", &json!(row_reports))?;
        write(
            &output,
            "summary.json",
            &json!({"gate":if no_safe>0{"FROZEN_PROPOSAL_FAMILY_CANNOT_COVER_TRAINING_WITHOUT_INCOMPATIBLE_ADMISSIONS"}else{"NO_COVER_IMPOSSIBILITY_ESTABLISHED"},"training_rows":train.len(),"proposals":options.len(),"enumerated_candidates":enumerated,"forced_candidate_generations":forced,"zero_signature_candidates_not_executed":zero_count,"nonzero_signatures":signatures.len(),"nonzero_conflicting_signatures":pairs.len(),"globally_safe_proposals":safe.iter().map(|i|options[*i]).collect::<Vec<_>>(),"unsafe_proposals":negatives.len(),"rows_without_reachable_target":rows.iter().filter(|r|r.positive.is_empty()).count(),"rows_all_reachable_targets_conflicted":all_conflicted,"rows_sole_reachable_target_conflicted":sole_conflicted,"rows_without_any_safe_positive_proposal":no_safe,"upper_bound_rows_coverable_under_zero_incompatible_admission":train.len()-no_safe,"bound_ignores_additional_uniqueness_and_eight_rule_constraints":true,"max_search_nodes":max_nodes,"fits":0,"candidate_sha256":sha(&bytes),"artifact_parameters_unchanged":a.encode()?==bytes,"source_sha256":sha(concat!(include_str!("occurrence_collision.rs"),include_str!("occurrence.rs"),include_str!("span.rs"),include_str!("span_learning.rs")).as_bytes()),"scope":"finite declared137-proposal family on frozen training inputs; exact admission-signature collision and globally unsafe-proposal witnesses, not general architecture impossibility","zero_candidate_compatibility":"NOT_EVALUATED and irrelevant to this proposal family because none can be admitted","promotion":false}),
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
