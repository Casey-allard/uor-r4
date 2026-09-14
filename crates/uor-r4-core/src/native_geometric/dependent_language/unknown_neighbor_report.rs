//! Training-only unknown-neighbor role learning with frozen open-development controls.
//! Raw grammar labels and exposed development vocabulary never enter the learner.
use super::{
    completion, occurrence_role, query_participation as model,
    query_participation_report::{assess, oracle, sha, words, write, Example},
    span_data, span_learning,
    styled_role_report::{retained_200, retained_6688},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const PARENT_SHA: &str = "0175ad86850a77017fdc41b7b4da9c22fe41bc8bcbf348e1352967dd29e43a94";
const CREDIT_SHA: &str = "d0860ccca3c9c038e9454ac53cbb198f0f20cc57b2070aa6d320788b83ac43c2";
const SECOND_SHA: &str = "d2c119f90f90275f9995728dcdd5fe248b4c53278046807a0ffe912bc283786e";
const OLD_SHA: &str = "8055a53c7801f1e6e11f808a6c15ddb090a5befe8d9d587b4ca65c74b7698793";

fn install(parent: &model::Artifact, roles: occurrence_role::Artifact) -> Result<model::Artifact> {
    let mut a = parent.clone();
    a.parent_digest = *blake3::hash(&roles.encode()?).as_bytes();
    a.parent = roles;
    Ok(a)
}

fn collision_key(
    a: &occurrence_role::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    raw: &[u8],
    mask_left: bool,
) -> Result<occurrence_role::Key> {
    let tokens = words(raw);
    let index = tokens
        .iter()
        .position(|word| *word == b"will")
        .ok_or("audit will")?;
    let keys = occurrence_role::observations(&a.parent, &a.anchors, g, m, raw, false)?;
    let mut key = keys.get(index).ok_or("audit key")?.clone();
    if mask_left {
        if index == 0 || tokens[index - 1] != b"amber" {
            return Err("audit masking is restricted to the training amber neighbor".into());
        }
        key.left = 64;
    }
    Ok(key)
}

fn representation_audit(
    before: &occurrence_role::Artifact,
    expanded: &occurrence_role::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<Value> {
    let auxiliary = b"who will help oscar?";
    let name = b"who did amber will help?";
    let auxiliary_count = train.iter().filter(|e| e.prompt == auxiliary).count();
    let name_count = train.iter().filter(|e| e.prompt == name).count();
    if auxiliary_count == 0 || name_count == 0 {
        return Err("collision audit queries must be present in frozen training".into());
    }
    let old_aux = collision_key(before, g, m, auxiliary, false)?;
    let old_name = collision_key(before, g, m, name, true)?;
    let new_aux = collision_key(expanded, g, m, auxiliary, false)?;
    let new_name = collision_key(expanded, g, m, name, true)?;
    let who_old = before.anchors.iter().any(|w| w.bytes == b"who");
    let who_new = expanded.anchors.iter().any(|w| w.bytes == b"who");
    let pass = old_aux == old_name
        && new_aux != new_name
        && !who_old
        && who_new
        && old_aux.left == 64
        && new_aux.left < 64
        && new_name.left == 64;
    Ok(
        json!({"status":"BEFORE_FIT","pass":pass,"auxiliary_raw":auxiliary.as_slice(),"name_raw":name.as_slice(),"auxiliary_training_rows":auxiliary_count,"name_training_rows":name_count,"old_auxiliary_key":old_aux,"old_name_masked_key":old_name,"expanded_auxiliary_key":new_aux,"expanded_name_masked_key":new_name,"old_anchor_count":before.anchors.len(),"expanded_anchor_count":expanded.anchors.len(),"question_only_who_added":!who_old&&who_new,"mask_scope":"Only the finite left-neighbor observation of training amber is set to unknown64; raw words and canonical matching identities remain unchanged.","scope":"Observed opposite-role collision is distinguishable after training-question anchors. This is not a learned-role or complete-generation result."}),
    )
}

fn source_roles_preserved(
    before: &occurrence_role::Artifact,
    after: &occurrence_role::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    training: &[span_data::Example],
    development: &[Example],
) -> Result<Value> {
    let raw: BTreeSet<_> = training
        .iter()
        .flat_map(|e| e.records.iter())
        .chain(development.iter().flat_map(|e| e.records.iter()))
        .collect();
    let mut changed = vec![];
    for record in &raw {
        let old = occurrence_role::source_context(before, g, m, record, false)?;
        let new = occurrence_role::source_context(after, g, m, record, false)?;
        if old != new {
            changed.push(json!({"record":record,"before":old,"after":new}));
        }
    }
    Ok(json!({"rows":raw.len(),"changed":changed.len(),"items":changed,"pass":changed.is_empty()}))
}

fn inventory(value: &Value, out: &mut BTreeSet<Vec<u8>>) -> Result<()> {
    match value {
        Value::Object(map) => {
            if map.contains_key("geometry") && map.contains_key("bytes") {
                out.insert(serde_json::from_value(map["bytes"].clone())?);
            }
            for child in map.values() {
                inventory(child, out)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                inventory(child, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn stage_observations(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
) -> Result<Value> {
    let mut rows = vec![];
    for (query, raw) in e
        .records
        .iter()
        .map(|r| (false, r))
        .chain(e.expected.queries.iter().map(|q| (true, q)))
    {
        let keys =
            occurrence_role::observations(&a.parent.parent, &a.parent.anchors, g, m, raw, false)?;
        let roles = if query {
            occurrence_role::query_context(&a.parent, g, m, raw, false)?
        } else {
            occurrence_role::source_context(&a.parent, g, m, raw, false)?
        };
        rows.push(json!({"query":query,"raw":raw,"keys":keys,"context":roles}));
    }
    Ok(json!(rows))
}

/// Replay only the sealed preceding candidate's divergent historical cases,
/// before broad training/retention replay. These are open development controls,
/// not fresh held-out evidence or inputs to rule induction.
fn divergent_probe(
    cached_root: &std::path::Path,
    candidate: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
) -> Result<Value> {
    let bytes = std::fs::read(cached_root.join("retained-6688.json"))?;
    let report: Value = serde_json::from_slice(&bytes)?;
    let mut count = 0;
    let mut equal = 0;
    let mut results = vec![];
    for panel in report["panels"]
        .as_array()
        .ok_or("cached retained panels")?
    {
        let divergent: Vec<_> = panel["items"]
            .as_array()
            .ok_or("cached retained items")?
            .iter()
            .filter(|row| row["equal"] == false)
            .collect();
        if divergent.is_empty() {
            continue;
        }
        let root = std::path::PathBuf::from(panel["root"].as_str().ok_or("cached raw panel root")?);
        report_output::verify(&root)?;
        let data_bytes = std::fs::read(root.join("data.json"))?;
        if panel["data_sha256"] != sha(&data_bytes) {
            return Err("cached divergent raw data identity".into());
        }
        let data: Value = serde_json::from_slice(&data_bytes)?;
        let raw = data["development"]
            .as_array()
            .ok_or("cached raw development")?;
        for prior in divergent {
            let matching: Vec<_> = raw.iter().filter(|e| e["id"] == prior["id"]).collect();
            if matching.len() != 1 {
                return Err("divergent raw row identity".into());
            }
            let e = matching[0];
            let records: [Vec<u8>; 4] = serde_json::from_value(e["records"].clone())?;
            let prompt: Vec<u8> = serde_json::from_value(e["prompt"].clone())?;
            let expected: completion::Generated =
                serde_json::from_value(prior["parent_if_changed"].clone())?;
            let actual = model::generate(candidate, g, m, &records, &prompt, model::Control::Full)?;
            let same = actual == expected;
            count += 1;
            equal += usize::from(same);
            results.push(json!({"root":root,"id":e["id"],"full_equal":same,"actual_tokens":actual.trace.tokens,"actual_outcome":actual.outcome,"actual_sha256":sha(&serde_json::to_vec(&actual)?),"expected_sealed8055_sha256":sha(&serde_json::to_vec(&expected)?),"actual_if_changed":if same {Value::Null} else {json!(actual)},"previous_rejected_root":cached_root}));
        }
    }
    if count != 320 {
        return Err("frozen cached divergent row count".into());
    }
    Ok(
        json!({"rows":count,"equal":equal,"pass":equal==320,"cached_retained_sha256":sha(&bytes),"items":results,"scope":"Actual corrected candidate versus sealed8055 outputs for the preceding candidate's320 divergent historical controls; no model selection occurs after this check."}),
    )
}

#[test]
#[ignore = "bounded training-only unknown-neighbor fit, actual generation and retained controls"]
fn unknown_neighbor_learning_report() -> Result<()> {
    let output = std::path::PathBuf::from(std::env::var("UOR_UNKNOWN_NEIGHBOR_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_UNKNOWN_NEIGHBOR_EVIDENCE")?);
    let credit_root =
        std::env::var_os("UOR_UNKNOWN_NEIGHBOR_CREDIT_REPORT").map(std::path::PathBuf::from);
    if output.as_os_str().is_empty()
        || evidence.as_os_str().is_empty()
        || credit_root
            .as_ref()
            .is_some_and(|p| p.as_os_str().is_empty())
    {
        return Err("empty report paths".into());
    }
    report_output::claim(&output)?;
    let result = (|| -> Result<()> {
        let root = evidence.join("styled-role-1/attempt-3");
        let neighbor_root = evidence.join("neighbor-transfer-1/attempt-1");
        report_output::verify(&root)?;
        report_output::verify(&neighbor_root)?;
        let data_bytes = std::fs::read(root.join("data.json"))?;
        let response_bytes = std::fs::read(root.join("responses.json"))?;
        let neighbor_data_bytes = std::fs::read(neighbor_root.join("data.json"))?;
        let neighbor_response_bytes = std::fs::read(neighbor_root.join("responses.json"))?;
        let data: Value = serde_json::from_slice(&data_bytes)?;
        let neighbor_data: Value = serde_json::from_slice(&neighbor_data_bytes)?;
        let sealed: Vec<Value> = serde_json::from_slice(&response_bytes)?;
        let neighbor_sealed: Vec<Value> = serde_json::from_slice(&neighbor_response_bytes)?;
        let training: Vec<span_data::Example> = serde_json::from_value(data["training"].clone())?;
        let development: Vec<Example> =
            serde_json::from_value(neighbor_data["development"].clone())?;
        let mut retained: Vec<Example> = serde_json::from_value(data["development"].clone())?;
        retained.extend(serde_json::from_value::<Vec<Example>>(
            data["collision"].clone(),
        )?);
        if training.len() != 2976 || development.len() != 492 || retained.len() != 300 {
            return Err("frozen panel sizes".into());
        }
        let training_inputs: BTreeSet<_> = training
            .iter()
            .map(|e| serde_json::to_vec(&(&e.records, &e.prompt)))
            .collect::<std::result::Result<_, _>>()?;
        let training_tokens: BTreeSet<Vec<u8>> = training
            .iter()
            .flat_map(|e| e.records.iter().chain([&e.prompt, &e.answer]))
            .flat_map(|raw| words(raw).into_iter().map(Vec::from))
            .collect();
        let mut development_tokens = BTreeSet::new();
        let mut development_inputs = BTreeSet::new();
        for e in &development {
            if oracle(&e.records, &e.prompt)? != e.expected {
                return Err("raw development oracle mismatch".into());
            }
            let input = serde_json::to_vec(&(&e.records, &e.prompt))?;
            if training_inputs.contains(&input) || !development_inputs.insert(input) {
                return Err("development overlap or duplicate raw case".into());
            }
            for raw in e.records.iter().chain([&e.prompt]) {
                development_tokens.extend(words(raw).into_iter().map(Vec::from));
            }
        }
        let absent_training_tokens: BTreeSet<_> = development_tokens
            .difference(&training_tokens)
            .cloned()
            .collect();
        let prior_environment_bytes = std::fs::read(neighbor_root.join("environment.json"))?;
        let prior_environment: Value = serde_json::from_slice(&prior_environment_bytes)?;
        let novel: BTreeSet<Vec<u8>> =
            serde_json::from_value(prior_environment["novel_names"].clone())?;
        if novel.len() != 13 || !novel.is_disjoint(&training_tokens) {
            return Err("declared thirteen development names overlap training".into());
        }
        write(
            &output,
            "environment.json",
            &json!({"declared_novel_names":novel,"all_development_tokens_absent_role_training":absent_training_tokens,"source_environment_sha256":sha(&prior_environment_bytes),"distinction":"Declared name substitutions include an unused third control-name component; other previously retained question/predicate tokens can also be absent from the direct role-training corpus. Only the sealed declared name set is required absent from all candidate inventories."}),
        )?;
        let retry = 2 * usize::from(credit_root.is_some());
        let witness_fits = usize::from(credit_root.is_none());
        let reused_credit_extraction_fits = usize::from(credit_root.is_some());
        let second_root = evidence.join("unknown-neighbor-1/attempt-3");
        let mut second_responses: Vec<Value> = vec![];
        let mut cached_responses: Vec<Value> = vec![];
        let mut cached_fit_bytes = vec![];
        let mut cached_candidate_bytes = vec![];
        if let Some(cached_root) = &credit_root {
            report_output::verify(cached_root)?;
            let cached_data_bytes = std::fs::read(cached_root.join("data.json"))?;
            let cached_data: Value = serde_json::from_slice(&cached_data_bytes)?;
            if cached_data["training"] != serde_json::to_value(&training)?
                || cached_data["development"] != serde_json::to_value(&development)?
                || cached_data["retained"] != serde_json::to_value(&retained)?
            {
                return Err("cached credit data differ from frozen current inputs".into());
            }
            cached_fit_bytes = std::fs::read(cached_root.join("fit.json"))?;
            cached_candidate_bytes = std::fs::read(cached_root.join("candidate.json"))?;
            if sha(&cached_candidate_bytes) != CREDIT_SHA {
                return Err("cached credit artifact identity".into());
            }
            let cached_response_bytes = std::fs::read(cached_root.join("responses.json"))?;
            cached_responses = serde_json::from_slice(&cached_response_bytes)?;
            report_output::verify(&second_root)?;
            let second_candidate_bytes = std::fs::read(second_root.join("candidate.json"))?;
            if sha(&second_candidate_bytes) != SECOND_SHA {
                return Err("second negative candidate identity".into());
            }
            let second_data_bytes = std::fs::read(second_root.join("data.json"))?;
            let second_data: Value = serde_json::from_slice(&second_data_bytes)?;
            if second_data["training"] != serde_json::to_value(&training)?
                || second_data["development"] != serde_json::to_value(&development)?
                || second_data["retained"] != serde_json::to_value(&retained)?
            {
                return Err("second negative candidate data changed".into());
            }
            let second_response_bytes = std::fs::read(second_root.join("responses.json"))?;
            second_responses = serde_json::from_slice(&second_response_bytes)?;
            if second_responses.len() != 492
                || second_responses
                    .iter()
                    .any(|r| r["controls"]["Full"]["assessment"]["correct"] != true)
            {
                return Err("second negative successful492 set".into());
            }
            write(
                &output,
                "second-negative-lineage.json",
                &json!({"root":second_root,"candidate_sha256":SECOND_SHA,"data_sha256":sha(&second_data_bytes),"responses_sha256":sha(&second_response_bytes),"successful_full_preservation_required":492}),
            )?;
            if cached_responses.len() != 492
                || cached_responses
                    .iter()
                    .any(|r| r["controls"]["Full"]["assessment"]["correct"] != true)
            {
                return Err("cached492 successful response set".into());
            }
            write(
                &output,
                "credit-lineage.json",
                &json!({"root":cached_root,"candidate_sha256":CREDIT_SHA,"fit_sha256":sha(&cached_fit_bytes),"data_sha256":sha(&cached_data_bytes),"responses_sha256":sha(&cached_response_bytes),"retry":2,"witness_fits":0,"rule_induction_fits":1,"reused_credit_extraction_fits":1,"training_and_development_unchanged":true,"scope":"Reinduce the identical sealed credit/blocker set under a corrected general unknown-neighbor rule guard; no witness search, fresh labels, data changes, or development-specific mask rule."}),
            )?;
        }
        write(
            &output,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD_AND_FIT","parent_sha256":PARENT_SHA,"training_rows":2976,"development_rows":492,"expected_parent_correct":468,"candidate_complete_correct":492,"prior_successes_lost":0,"training_correct":2976,"retained_sealed_full":300,"retained_200":200,"retained_6688":6688,"controls":{"ExactIdentity":"all492 identical Full","reload":"all492 identical Full","OriginalRolesRestored":"all492 identical actual parent and sealed parent","ReadDisabled":"zero correct answered outputs","UpdateDisabled":"zero correct answered outputs","AnchorsOnly":"measured, no required improvement or failure count","NoProjection":"full-known witnessed query fit before projected credits; measured all492, no effect mandated"},"fits":1,"rule_induction_fits":1,"witness_fits":witness_fits,"reused_credit_extraction_fits":reused_credit_extraction_fits,"retries":retry,"previous_candidate_success_exact_required":if retry>0{492}else{0},"participation_fits":0,"source_role_fits":0,"training_observation_views":"Predetermined eligible training-name anchors withheld only from finite role observations; exact canonical matching unchanged. No exposed development names or raw-oracle role labels enter learning.","limits":{"records":4,"record_words":16,"record_bytes":128,"prompt_bytes":256,"clauses":2,"payload_words":3,"payload_bytes":50,"steps":96},"scope":"open authored lexical-neighbor development and retained controls","final_holdout":"NOT_RUN","promotion":false}),
        )?;
        write(
            &output,
            "data.json",
            &json!({"training":training,"development":development,"retained":retained,"training_source":root,"training_data_sha256":sha(&data_bytes),"retained_responses_sha256":sha(&response_bytes),"development_source":neighbor_root,"development_data_sha256":sha(&neighbor_data_bytes),"development_responses_sha256":sha(&neighbor_response_bytes),"novel_names":novel,"training_development_raw_overlap":0,"raw_oracle_rebuilt":true}),
        )?;
        let parent_bytes = std::fs::read(root.join("candidate.json"))?;
        if sha(&parent_bytes) != PARENT_SHA {
            return Err("parent artifact identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let parent = model::Artifact::decode(&parent_bytes, &g)?;
        let expanded_roles =
            occurrence_role::expand_training_anchors(parent.parent.clone(), &g, &training)?;
        let expanded_roles =
            occurrence_role::inherit_query_anchors(expanded_roles, &g, &parent.anchors)?;
        write(
            &output,
            "inherited-query-vocabulary.json",
            &json!({"source_parent_sha256":PARENT_SHA,"source_parent_data_digest":parent.data_digest,"already_trained_query_anchors":parent.anchors,"already_trained_query_anchors_sha256":sha(&serde_json::to_vec(&parent.anchors)?),"expanded_role_anchors":expanded_roles.anchors,"scope":"Reuse canonical identities already bound to the retained outer query-participation artifact; no new evaluation names, examples, or labels."}),
        )?;
        let audit = representation_audit(&parent.parent, &expanded_roles, &g, &m, &training)?;
        write(&output, "representation-audit.json", &audit)?;
        if audit["pass"] != true {
            return Err("training-only representation audit failed before fit".into());
        }
        let anchors_only = install(&parent, expanded_roles.clone())?;
        anchors_only.validate(&g)?;
        let (roles, fit) = if credit_root.is_some() {
            let mut cached = model::Artifact::decode(&cached_candidate_bytes, &g)?;
            let old_anchors = cached.parent.anchors.clone();
            cached.parent =
                occurrence_role::inherit_query_anchors(cached.parent, &g, &parent.anchors)?;
            let migrated_fit = occurrence_role::migrate_unknown_credit(
                serde_json::from_slice::<occurrence_role::UnknownNeighborFit>(&cached_fit_bytes)?,
                &g,
                &old_anchors,
                &cached.parent.anchors,
            )?;
            write(
                &output,
                "credit-namespace-migration.json",
                &json!({"source_fit_sha256":sha(&cached_fit_bytes),"migrated_fit_sha256":sha(&serde_json::to_vec(&migrated_fit)?),"old_anchors":old_anchors,"new_anchors":cached.parent.anchors,"inherited_query_anchor_source_sha256":PARENT_SHA,"inherited_query_anchors_sha256":sha(&serde_json::to_vec(&parent.anchors)?),"scope":"Exact Word index migration of the sealed credits, blockers, known tables and eligible IDs. Center identities and label counts unchanged; unknown remains unknown. No repeated witness fit."}),
            )?;
            if cached.parent.anchors != expanded_roles.anchors
                || cached.parent.table != expanded_roles.table
                || cached.parent.parent != expanded_roles.parent
                || cached.parent.parent_digest != expanded_roles.parent_digest
                || cached.parent.require_available_query_coverage
                    != expanded_roles.require_available_query_coverage
                || cached.parent.schema != expanded_roles.schema
                || cached.anchors != parent.anchors
                || cached.optional != parent.optional
                || cached.replacement != parent.replacement
            {
                return Err(
                    "cached credit candidate differs beyond query-role fitting seam".into(),
                );
            }
            occurrence_role::reinduce_unknown_neighbors(cached.parent, &g, migrated_fit)?
        } else {
            occurrence_role::fit_unknown_neighbors(expanded_roles, &g, &m, &training)?
        };
        write(&output, "fit.json", &json!(fit))?;
        let known_query_table = fit.known_query_table.clone();
        let mut candidate = install(&parent, roles)?;
        candidate.data_digest = *blake3::hash(
            &[
                parent.data_digest.as_slice(),
                serde_json::to_vec(&training)?.as_slice(),
            ]
            .concat(),
        )
        .as_bytes();
        candidate.source_digest = *blake3::hash(
            concat!(
                include_str!("unknown_neighbor_report.rs"),
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
        candidate.training.push_str(" Training-only role-anchor expansion from records and questions with exact Word remapping, then one unknown-neighbor query-role fit. Source-role table semantics and all participation parameters are frozen. Open authored492 evaluation vocabulary does not enter learning; final independent evaluation is not run.");
        if retry > 0 {
            candidate.training.push_str(" A second development-informed correction inherits already-trained query vocabulary into the role observation namespace, migrates the original sealed credits and blockers by exact Word identity, and reinduces without repeating witness extraction. The first retry's conservative opposite-neighbor guard remains.");
        }
        candidate.validate(&g)?;
        let candidate_bytes = candidate.encode()?;
        let mut known = BTreeSet::new();
        inventory(&serde_json::from_slice(&candidate_bytes)?, &mut known)?;
        if !novel.is_disjoint(&known) {
            return Err("exposed development name entered candidate inventory".into());
        }
        std::fs::write(output.join("candidate.json"), &candidate_bytes)?;
        let reload = model::Artifact::decode(&candidate_bytes, &g)?;
        // Cheap actual-artifact multi-turn check immediately after construction.
        // Include the previously failing lexical form; no fit or retry follows.
        let sample_rows: Vec<_> = development
            .iter()
            .filter(|e| {
                e.family.starts_with("styled")
                    && e.expected.queries.iter().any(|q| {
                        words(q)
                            .windows(2)
                            .any(|w| w == [b"selka".as_slice(), b"will"])
                    })
            })
            .take(3)
            .collect();
        if sample_rows.len() != 3 {
            return Err("immediate sample identity".into());
        }
        let mut sample = vec![];
        for e in sample_rows {
            let full =
                model::generate(&reload, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            sample.push(json!({"id":e.id,"Full":full,"assessment":assess(&reload,&g,&m,e,model::Control::Full,&full)?}));
        }
        write(&output, "immediate-multi-turn.json", &json!(sample))?;
        let divergent = if let Some(cached_root) = &credit_root {
            divergent_probe(cached_root, &candidate, &g, &m)?
        } else {
            json!({"status":"NOT_APPLICABLE","rows":0,"pass":true})
        };
        write(&output, "immediate-divergent-controls.json", &divergent)?;
        let source_invariance = source_roles_preserved(
            &parent.parent,
            &candidate.parent,
            &g,
            &m,
            &training,
            &development,
        )?;
        write(&output, "source-role-invariance.json", &source_invariance)?;
        let mut restored = candidate.clone();
        restored.parent = parent.parent.clone();
        restored.parent_digest = parent.parent_digest;
        restored.validate(&g)?;
        let mut no_projection = candidate.clone();
        no_projection.parent.query_table = known_query_table;
        no_projection.parent_digest = *blake3::hash(&no_projection.parent.encode()?).as_bytes();
        no_projection.validate(&g)?;
        let mut counts = BTreeMap::<String, usize>::new();
        let (
            mut correct,
            mut parent_correct,
            mut gained,
            mut lost,
            mut exact,
            mut reloaded,
            mut restored_equal,
            mut parent_sealed_equal,
            mut disabled_answer_correct,
        ) = (0, 0, 0, 0, 0, 0, 0, 0, 0);
        let mut prior_success_full_equal = 0;
        let mut previous_candidate_success_equal = 0;
        let mut second_candidate_success_equal = 0;
        let mut second_comparison = vec![];
        let mut previous_comparison = vec![];
        let mut responses = vec![];
        let mut failures = vec![];
        for e in &development {
            let matching: Vec<_> = neighbor_sealed.iter().filter(|r| r["id"] == e.id).collect();
            if matching.len() != 1 {
                return Err("sealed492 row identity".into());
            }
            let sealed_full: completion::Generated =
                serde_json::from_value(matching[0]["Full"].clone())?;
            let old =
                model::generate(&parent, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            parent_sealed_equal += usize::from(old == sealed_full);
            let old_assessment = assess(&parent, &g, &m, e, model::Control::Full, &old)?;
            let old_yes = old_assessment["correct"] == true;
            parent_correct += usize::from(old_yes);
            let full = model::generate(
                &candidate,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::Full,
            )?;
            if retry > 0 {
                let matching_second: Vec<_> = second_responses
                    .iter()
                    .filter(|r| r["id"] == e.id)
                    .collect();
                if matching_second.len() != 1 {
                    return Err("second negative492 row identity".into());
                }
                let second: completion::Generated =
                    serde_json::from_value(matching_second[0]["Full"].clone())?;
                let equal_second = second == full;
                second_candidate_success_equal += usize::from(equal_second);
                second_comparison.push(json!({"id":e.id,"full_equal":equal_second,"actual_if_changed":if equal_second {Value::Null} else {json!(full)},"previous_if_changed":if equal_second {Value::Null} else {json!(second)}}));
                let matching: Vec<_> = cached_responses
                    .iter()
                    .filter(|r| r["id"] == e.id)
                    .collect();
                if matching.len() != 1 {
                    return Err("cached492 unique row identity".into());
                }
                let previous: completion::Generated =
                    serde_json::from_value(matching[0]["Full"].clone())?;
                let equal = previous == full;
                previous_candidate_success_equal += usize::from(equal);
                previous_comparison.push(json!({"id":e.id,"full_equal":equal,"actual_if_changed":if equal {Value::Null} else {json!(full)},"previous_if_changed":if equal {Value::Null} else {json!(previous)}}));
            }
            let full_assessment = assess(&candidate, &g, &m, e, model::Control::Full, &full)?;
            let yes = full_assessment["correct"] == true;
            correct += usize::from(yes);
            gained += usize::from(yes && !old_yes);
            lost += usize::from(old_yes && !yes);
            prior_success_full_equal += usize::from(old_yes && full == old);
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
            let restored_full = model::generate(
                &restored,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::Full,
            )?;
            restored_equal += usize::from(restored_full == old);
            let anchors_full = model::generate(
                &anchors_only,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::Full,
            )?;
            let panel = if e.family.starts_with("styled-missing-") {
                "styled_missing"
            } else if e.family.starts_with("styled-conflict-") {
                "styled_conflict"
            } else {
                e.family.split('-').next().ok_or("family")?
            };
            let mut controls = serde_json::Map::new();
            for (name, a, c, out) in [
                ("Full", &candidate, model::Control::Full, full.clone()),
                ("Parent", &parent, model::Control::Full, old.clone()),
                (
                    "OriginalRolesRestored",
                    &restored,
                    model::Control::Full,
                    restored_full,
                ),
                (
                    "AnchorsOnly",
                    &anchors_only,
                    model::Control::Full,
                    anchors_full,
                ),
                (
                    "NoProjection",
                    &no_projection,
                    model::Control::Full,
                    model::generate(
                        &no_projection,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::Full,
                    )?,
                ),
                (
                    "ExactIdentity",
                    &candidate,
                    model::Control::ExactIdentity,
                    model::generate(
                        &candidate,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::ExactIdentity,
                    )?,
                ),
                (
                    "ReadDisabled",
                    &candidate,
                    model::Control::ReadDisabled,
                    model::generate(
                        &candidate,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::ReadDisabled,
                    )?,
                ),
                (
                    "UpdateDisabled",
                    &candidate,
                    model::Control::UpdateDisabled,
                    model::generate(
                        &candidate,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::UpdateDisabled,
                    )?,
                ),
            ] {
                let assessment = assess(a, &g, &m, e, c, &out)?;
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
                    disabled_answer_correct += usize::from(assessment["outcome_correct"] == true);
                }
                controls.insert(name.into(),json!({"assessment":assessment,"outcome":out.outcome,"tokens":out.trace.tokens,"equal_full":out==full,"equal_parent":out==old}));
            }
            if !yes {
                failures.push(json!({"id":e.id,"records":e.records,"prompt":e.prompt,"expected":e.expected,"actual":full,"parent":old,"candidate_role_observations":stage_observations(&candidate,&g,&m,e)?}));
            }
            responses.push(json!({"id":e.id,"family":e.family,"variant":e.variant,"Full":full,"controls":controls,"parent_sealed_equal":old==sealed_full,"gained":yes&&!old_yes,"lost":old_yes&&!yes}));
        }
        write(&output, "responses.json", &json!(responses))?;
        write(
            &output,
            "previous-candidate-492.json",
            &json!({"root":credit_root,"rows":cached_responses.len(),"full_equal":previous_candidate_success_equal,"items":previous_comparison}),
        )?;
        write(
            &output,
            "failure-observations.json",
            &json!({"rows":failures.len(),"items":failures}),
        )?;
        write(
            &output,
            "second-negative-492.json",
            &json!({"root":second_root,"rows":second_responses.len(),"full_equal":second_candidate_success_equal,"items":second_comparison}),
        )?;
        let mut training_correct = 0;
        let mut parent_training_correct = 0;
        let mut anchors_training_correct = 0;
        let mut training_lost = 0;
        let mut training_results = vec![];
        for e in &training {
            let target = span_learning::target(e);
            let mut outputs = serde_json::Map::new();
            let mut flags = vec![];
            for (name, a) in [
                ("Full", &candidate),
                ("Parent", &parent),
                ("AnchorsOnly", &anchors_only),
            ] {
                let full = model::generate(a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
                let correct = full.outcome == completion::Outcome::Answered
                    && !full.trace.exhausted
                    && full.trace.tokens == target;
                flags.push(correct);
                outputs.insert(
                    name.into(),
                    json!({"correct":correct,"outcome":full.outcome,"tokens":full.trace.tokens}),
                );
            }
            training_correct += usize::from(flags[0]);
            parent_training_correct += usize::from(flags[1]);
            anchors_training_correct += usize::from(flags[2]);
            training_lost += usize::from(flags[1] && !flags[0]);
            training_results.push(json!({"id":e.id,"controls":outputs}));
        }
        write(
            &output,
            "training-responses.json",
            &json!({"rows":training.len(),"correct":training_correct,"parent_correct":parent_training_correct,"anchors_only_correct":anchors_training_correct,"prior_successes_lost":training_lost,"items":training_results}),
        )?;
        let mut retained_results = vec![];
        let mut retained_equal = 0;
        let mut retained_parent_equal = 0;
        for e in &retained {
            let matching: Vec<_> = sealed.iter().filter(|r| r["id"] == e.id).collect();
            if matching.len() != 1
                || matching[0]["controls"]["Full"]["assessment"]["correct"] != true
            {
                return Err("sealed300 row identity".into());
            }
            let expected: completion::Generated =
                serde_json::from_value(matching[0]["Full"].clone())?;
            let old =
                model::generate(&parent, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
            let actual = model::generate(
                &candidate,
                &g,
                &m,
                &e.records,
                &e.prompt,
                model::Control::Full,
            )?;
            let equal = actual == expected;
            retained_equal += usize::from(equal);
            retained_parent_equal += usize::from(old == expected);
            retained_results.push(json!({"id":e.id,"full_equal":equal,"parent_sealed_equal":old==expected,"actual_if_changed":if equal {Value::Null} else {json!(actual)}}));
        }
        write(
            &output,
            "retained-300.json",
            &json!({"source_root":root,"source_responses_sha256":sha(&response_bytes),"rows":300,"full_equal":retained_equal,"parent_sealed_equal":retained_parent_equal,"items":retained_results}),
        )?;
        let old_root = evidence.join("query-participation-1/attempt-1");
        report_output::verify(&old_root)?;
        let old_bytes = std::fs::read(old_root.join("candidate.json"))?;
        if sha(&old_bytes) != OLD_SHA {
            return Err("earlier8055 identity".into());
        }
        let old = model::Artifact::decode(&old_bytes, &g)?;
        let r200 = retained_200(&candidate, &old, &g, &m, &evidence)?;
        write(&output, "retained-200.json", &r200)?;
        let r6688 = retained_6688(&candidate, &old, &g, &m, &evidence)?;
        write(&output, "retained-6688.json", &r6688)?;
        let participation_unchanged = candidate.anchors == parent.anchors
            && candidate.optional == parent.optional
            && candidate.replacement == parent.replacement;
        let source_table_frozen = candidate.parent.table == anchors_only.parent.table
            && candidate.parent.parent == parent.parent.parent;
        let parent_unchanged = std::fs::read(root.join("candidate.json"))? == parent_bytes;
        report_output::verify(&root)?;
        report_output::verify(&neighbor_root)?;
        if let Some(cached_root) = &credit_root {
            report_output::verify(cached_root)?;
        }
        if retry > 0 {
            report_output::verify(&second_root)?;
        }
        let pass = correct == 492
            && parent_correct == 468
            && prior_success_full_equal == 468
            && lost == 0
            && parent_sealed_equal == 492
            && exact == 492
            && reloaded == 492
            && restored_equal == 492
            && disabled_answer_correct == 0
            && training_correct == 2976
            && parent_training_correct == 2976
            && training_lost == 0
            && retained_equal == 300
            && retained_parent_equal == 300
            && r200["pass"] == true
            && r6688["pass"] == true
            && source_invariance["pass"] == true
            && participation_unchanged
            && source_table_frozen
            && parent_unchanged
            && divergent["pass"] == true
            && (retry == 0
                || (previous_candidate_success_equal == 492
                    && second_candidate_success_equal == 492));
        let gate = if pass {
            "PASS_LEARNED_UNKNOWN_NEIGHBORS"
        } else {
            "FAIL_LEARNED_UNKNOWN_NEIGHBORS"
        };
        let mut summary = json!({"gate":gate,"previous_candidate_success_equal":previous_candidate_success_equal,"second_candidate_success_equal":second_candidate_success_equal,"immediate_divergent":{"rows":divergent["rows"],"equal":divergent["equal"],"pass":divergent["pass"]},"rows":492,"correct":correct,"parent_correct":parent_correct,"prior_success_full_equal":prior_success_full_equal,"gained":gained,"lost":lost,"panels":counts,"parent_sealed_equal":parent_sealed_equal,"exact_equal":exact,"reload_equal":reloaded,"original_roles_restored_equal":restored_equal,"read_or_update_disabled_correct_answer":disabled_answer_correct});
        let detail = json!({"training_rows":2976,"training_correct":training_correct,"parent_training_correct":parent_training_correct,"anchors_only_training_correct":anchors_training_correct,"training_prior_successes_lost":training_lost,"retained_300_equal":retained_equal,"retained_300_parent_equal":retained_parent_equal,"retained_200_equal":r200["equal"],"retained_6688_equal":r6688["equal"],"retained_typed_unresolved":r6688["typed_unresolved"],"source_roles_preserved":source_invariance["pass"],"source_table_frozen_after_remap":source_table_frozen,"participation_parameters_unchanged":participation_unchanged,"parent_artifact_unchanged":parent_unchanged,"parent_sha256":PARENT_SHA,"candidate_sha256":sha(&candidate_bytes),"fits":1,"rule_induction_fits":1,"witness_fits":witness_fits,"reused_credit_extraction_fits":reused_credit_extraction_fits,"source_role_fits":0,"participation_fits":0,"retries":retry,"final_holdout":"NOT_RUN","promotion":false,"default_model":"15baec48 unchanged","scope":"open authored lexical-neighbor training and two-clause development at four records; not general-language qualification"});
        summary
            .as_object_mut()
            .ok_or("summary object")?
            .extend(detail.as_object().ok_or("summary detail")?.clone());
        write(&output, "summary.json", &summary)?;
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
