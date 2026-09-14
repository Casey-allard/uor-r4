//! No-fit representation check on frozen training inputs and the recorded collision.
use super::{
    completion, correspondence as runtime, scheduling, span, span_data, span_learning::target,
};
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
use std::{collections::BTreeSet, path::Path};
fn sha(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
#[test]
#[ignore = "bounded no-fit correspondence representation diagnostic"]
fn correspondence_representation_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::path::PathBuf::from(std::env::var("UOR_CORRESPONDENCE_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_CORRESPONDENCE_EVIDENCE")?);
    if output.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty report/evidence path".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","fits":0,"training_rows":2816,"base_rule":327682,"hypothesis_probe_rule":851970,"require_all_training_rows_exactly_one_compatible_offer":true,"require_all912_prior_sole_target_conflict_rows_separated":true,"record_actual_output_for_every_base_reachable_candidate":true,"other_candidates":"NOT_EVALUATED; no hypothesis rule can admit them","recorded_pair":"same source/span identity; correct offer admitted, incorrect offer excluded","scope":"representation hypothesis on known training inputs; no final holdout or promotion"}),
        )?;
        let prior = evidence.join("language-occurrence-1/attempt-2");
        report_output::verify(&prior)?;
        let train_bytes = std::fs::read(prior.join("training-data.json"))?;
        let data: Value = serde_json::from_slice(&train_bytes)?;
        let train: Vec<span_data::Example> = serde_json::from_value(data["training"].clone())?;
        if train.len() != 2816 {
            return Err("training count differs".into());
        }
        let previous = evidence.join("language-occurrence-1/collision-1");
        report_output::verify(&previous)?;
        let collision_rows: Vec<Value> =
            serde_json::from_slice(&std::fs::read(previous.join("row-collisions.json"))?)?;
        let conflict_ids: BTreeSet<String> = collision_rows
            .iter()
            .filter(|r| r["sole_reachable_target_conflicted"] == true)
            .map(|r| r["id"].as_str().ok_or("collision id").map(str::to_owned))
            .collect::<Result<_, _>>()?;
        if conflict_ids.len() != 912 {
            return Err("prior conflict count differs".into());
        }
        let parent = evidence.join("language-span-1/attempt-3");
        report_output::verify(&parent)?;
        let bytes = std::fs::read(parent.join("candidate.json"))?;
        if sha(&bytes) != "44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b" {
            return Err("parent hash differs".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = runtime::Artifact::from_parent(span::Artifact::decode(&bytes, &g)?)?;
        let wrapper_bytes = a.encode()?;
        if runtime::Artifact::decode(&wrapper_bytes, &g)? != a {
            return Err("wrapper roundtrip differs".into());
        }
        let mut tampered = a.clone();
        tampered.parent_digest[0] ^= 1;
        if runtime::Artifact::decode(&tampered.encode()?, &g).is_ok() {
            return Err("parent digest tamper accepted".into());
        }
        write(
            &output,
            "artifact-validation.json",
            &json!({"roundtrip":true,"parent_digest_tamper_rejected":true,"wrapper_sha256":sha(&wrapper_bytes)}),
        )?;
        write(
            &output,
            "inputs.json",
            &json!({"training_data_sha256":sha(&train_bytes),"parent_sha256":sha(&bytes),"context_words":a.parent.context_words,"source_training":prior,"source_collision":previous}),
        )?;
        let mut forced = 0;
        let mut enumerated = 0;
        let mut excluded = 0;
        let mut exact = 0;
        let mut conflicts_fixed = 0;
        let mut negative_admissions = 0;
        let mut max_nodes = 0;
        let mut rows = Vec::new();
        let mut first_negative = None;
        let mut evaluated_candidates = Vec::new();
        for e in &train {
            let search =
                runtime::candidates(&a, &g, &m, &e.records, &e.prompt, runtime::Control::Full)?;
            max_nodes = max_nodes.max(search.nodes);
            let mut positives = 0;
            let mut negatives = 0;
            let mut reachable_positive = 0;
            let mut accepted = Vec::new();
            for c in search.candidates {
                enumerated += 1;
                if !c.witnesses.iter().any(|w| w.features & 327682 == 327682) {
                    excluded += 1;
                    continue;
                }
                let value = &c.span.value;
                forced += 1;
                let out = completion::generate_routed(
                    &a.parent.parent,
                    &g,
                    &m,
                    &e.records,
                    &e.prompt,
                    completion::Control::Full,
                    scheduling::Control::Full,
                    |_, _| {
                        Ok(Route {
                            status: RouteStatus::Selected,
                            selected: Some(value.clone()),
                            compatible: vec![[value.source, value.word]],
                        })
                    },
                )?;
                let yes = out.trace.tokens == target(e)
                    && !out.trace.exhausted
                    && out.outcome == completion::Outcome::Answered;
                reachable_positive += usize::from(yes);
                evaluated_candidates.push(json!({"id":e.id,"source":c.span.value.source,"first_word":c.span.value.word,"last_word_exclusive":c.span.last_word,"witness_features":c.witnesses.iter().map(|w|w.features).collect::<Vec<_>>(),"actual_tokens":out.trace.tokens,"actual_outcome":out.outcome,"exhausted":out.trace.exhausted,"compatible":yes,"hypothesis_admitted":c.witnesses.iter().any(|w|w.features&851970==851970)}));
                if c.witnesses.iter().any(|w| w.features & 851970 == 851970) {
                    positives += usize::from(yes);
                    negatives += usize::from(!yes);
                    let sample = json!({"candidate":c,"actual_tokens":out.trace.tokens,"actual_outcome":out.outcome,"compatible":yes});
                    if !yes && first_negative.is_none() {
                        first_negative = Some(json!({"example":e,"admission":sample}));
                    }
                    accepted.push(sample);
                }
            }
            let passes = positives == 1 && negatives == 0;
            exact += usize::from(passes);
            negative_admissions += negatives;
            conflicts_fixed += usize::from(passes && conflict_ids.contains(&e.id));
            rows.push(json!({"id":e.id,"prior_sole_target_conflict":conflict_ids.contains(&e.id),"base_reachable_compatible_candidates":reachable_positive,"hypothesis_compatible_offers":positives,"hypothesis_incompatible_offers":negatives,"exactly_one_compatible_offer":passes,"failed_row_details":if passes{Value::Null}else{json!({"example":e,"accepted":accepted})}}));
        }
        let pairs: Vec<Value> = serde_json::from_slice(&std::fs::read(
            previous.join("nonzero-conflict-pairs.json"),
        )?)?;
        let mut pair_reports = Vec::new();
        let mut pairs_separated = 0;
        for pair in &pairs {
            let mut values = Vec::new();
            let mut good = true;
            for (label, should_admit) in [("positive", true), ("negative", false)] {
                let sample = &pair[label];
                let records: [Vec<u8>; 4] = serde_json::from_value(sample["records"].clone())?;
                let prompt: Vec<u8> = serde_json::from_value(sample["prompt"].clone())?;
                let old = &sample["candidate"]["span"];
                let source = old["value"]["source"].as_u64().ok_or("source")? as usize;
                let first = old["value"]["word"].as_u64().ok_or("word")? as usize;
                let end = old["last_word"].as_u64().ok_or("last")? as usize;
                let search =
                    runtime::candidates(&a, &g, &m, &records, &prompt, runtime::Control::Full)?;
                let c = search
                    .candidates
                    .iter()
                    .find(|c| {
                        c.span.value.source == source
                            && c.span.value.word == first
                            && c.span.last_word == end
                    })
                    .ok_or("recorded candidate missing")?;
                let admitted = c.witnesses.iter().any(|w| w.features & 851970 == 851970);
                good &= admitted == should_admit;
                values.push(json!({"label":label,"records":records,"prompt":prompt,"old_candidate":old,"new_candidate":c,"hypothesis_admitted":admitted,"expected_admission":should_admit}));
            }
            pairs_separated += usize::from(good);
            pair_reports.push(json!({"separated":good,"cases":values}));
        }
        write(
            &output,
            "evaluated-candidates.json",
            &json!(evaluated_candidates),
        )?;
        write(&output, "rows.json", &json!(rows))?;
        write(&output, "recorded-pairs.json", &json!(pair_reports))?;
        write(
            &output,
            "summary.json",
            &json!({"gate":if exact==2816&&conflicts_fixed==912&&negative_admissions==0&&!pairs.is_empty()&&pairs_separated==pairs.len(){"PASS_CORRESPONDENCE_REPRESENTATION"}else{"FAIL_CORRESPONDENCE_REPRESENTATION"},"training_rows":train.len(),"exactly_one_compatible_offer_rows":exact,"prior_sole_target_conflict_rows":912,"prior_conflict_rows_separated":conflicts_fixed,"negative_admissions":negative_admissions,"first_negative":first_negative,"recorded_pairs":pairs.len(),"recorded_pairs_separated":pairs_separated,"enumerated_candidates":enumerated,"forced_candidate_generations":forced,"unreachable_candidates_not_evaluated":excluded,"max_search_nodes":max_nodes,"fits":0,"parent_sha256":sha(&bytes),"parent_parameters_unchanged":a.parent.encode()?==bytes,"source_sha256":sha(concat!(include_str!("correspondence_diagnostic.rs"),include_str!("correspondence.rs"),include_str!("occurrence.rs")).as_bytes()),"final_holdout":"NOT_RUN","promotion":false}),
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
