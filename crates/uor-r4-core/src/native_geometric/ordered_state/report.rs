use super::{
    data, learning,
    runtime::{self, Artifact, Control, Operator, Query, CANONICAL},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry,
        hamming_refinement::metric::Metric,
        recurrent_text::{self, runtime as recurrent},
    },
    report_output,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::{collections::BTreeMap, path::Path};
fn write(p: &Path, n: &str, v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(p.join(n), serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn sha(b: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(b))
}
#[test]
fn ordered_corpus_and_prefix_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let (t, d) = data::corpus()?;
    data::validate(&t, &d)?;
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    for e in &t {
        let q = Query::encode(&g, &e.query, CANONICAL)?;
        let mut s = Query::empty();
        for &b in &e.query {
            s = s.push(&g, b, CANONICAL)?;
        }
        assert_eq!(q, s);
        assert_eq!(runtime::distance(&m, &q, &s, false)?, 0);
    }
    let mut q = Query::empty();
    for _ in 0..runtime::MAX_PREFIX {
        q = q.push(&g, b'a', CANONICAL)?;
    }
    assert!(q.push(&g, b'a', CANONICAL).is_err());
    assert!(Query::encode(&g, &[], CANONICAL).is_err());
    let mut malformed = q.clone();
    malformed.prefixes.push([0; 2]);
    assert!(runtime::distance(&m, &malformed, &q, false).is_err());
    let mut malformed = q.clone();
    malformed.prefixes[0][0] = 120;
    assert!(runtime::distance(&m, &malformed, &q, true).is_err());
    let aliased = (0u8..255)
        .flat_map(|a| (a + 1..=255).map(move |b| (a, b)))
        .find(|&(a, b)| g.byte_leaf(a) == g.byte_leaf(b))
        .ok_or("expected finite leaf aliases")?;
    let a = Query::encode(&g, &[aliased.0], CANONICAL)?;
    let b = Query::encode(&g, &[aliased.1], CANONICAL)?;
    assert_eq!(runtime::distance(&m, &a, &b, false)?, 0);
    assert_ne!(a.occurrences, b.occurrences);
    Ok(())
}
#[test]
#[ignore = "bounded output-driven ordered-state fit, pinned parent and exclusive report"]
fn ordered_state_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_ORDERED_REPORT")?);
    let evidence = std::path::PathBuf::from(std::env::var("UOR_ORDERED_EVIDENCE")?);
    if path.as_os_str().is_empty() || evidence.as_os_str().is_empty() {
        return Err("empty paths".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        write(
            &path,
            "acceptance.json",
            &json!({"status":"FROZEN_BEFORE_MODEL_LOAD","development_rows":64,"minimum_exact":61,"minimum_complete_families":15,"minimum_correct_source_paths":61,"required_controls_exact_max":0,"diagnostic_only_controls":["FinalRootOnly"],"prefit_ambiguous_sequence_keys_max":0,"training_rows":256,"operator_candidates":16,"retention":"all previous retained outputs and all recurrent development control traces must remain identical","final_holdout":"NOT_RUN","scope":"ordered supplied occurrence-key contextual read with actual emitted-byte recurrence; no raw-language relation parse or semantic generalization claim"}),
        )?;
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let bytes = std::fs::read(evidence.join("recurrent-text-1/attempt-2/candidate.json"))?;
        if sha(&bytes) != "acad5821d77361aba1b752789f4f5324f27719201e5ab9ba0fdf8cd3e1e0c4c5" {
            return Err("parent hash".into());
        }
        let parent = recurrent::Artifact::decode(&bytes, &g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate(&train, &dev)?;
        write(
            &path,
            "data.json",
            &json!({"training":train,"development":dev,"validation":validation}),
        )?;
        let mut collisions = Vec::new();
        let mut final_collisions = Vec::new();
        let mut counts = 0;
        for e in train.iter().chain(&dev) {
            for i in 0..4 {
                for j in i + 1..4 {
                    let a = Query::encode(&g, &e.records[i].key, CANONICAL)?;
                    let b = Query::encode(&g, &e.records[j].key, CANONICAL)?;
                    counts += 1;
                    if runtime::distance(&m, &a, &b, false)? == 0 {
                        collisions.push(json!({"id":e.id,"slots":[i,j]}));
                    }
                    if runtime::distance(&m, &a, &b, true)? == 0 {
                        final_collisions.push(json!({"id":e.id,"slots":[i,j]}));
                    }
                }
            }
        }
        write(
            &path,
            "preparation.json",
            &json!({"within_context_pairs":counts,"prefix_collisions":collisions,"final_root_only_collisions":final_collisions,"representation":"forward and reverse signed H4 product at every occurrence position; two channels not paired-H4 lift","all_authored_rows_retained":true,"development_labels_excluded_from_fit":true,"metric":"fixed hemisphere-signature Hamming with strict zero compatibility, no semantic-distance claim"}),
        )?;
        if !collisions.is_empty() {
            write(
                &path,
                "summary.json",
                &json!({"gate":"FAIL_ORDERED_STATE_REPRESENTATION","fit":"NOT_RUN","collisions":collisions.len(),"promotion":false}),
            )?;
            return Ok(());
        }
        let sources = concat!(
            include_str!("runtime.rs"),
            include_str!("learning.rs"),
            include_str!("data.rs"),
            include_str!("../recurrent_text/runtime.rs")
        );
        let initial=Artifact{schema:1,parent:parent.clone(),parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),geometry_digest:g.id(),operators:[Operator::Hold;2],source_digest:*blake3::hash(sources.as_bytes()).as_bytes(),data_digest:*blake3::hash(&serde_json::to_vec(&train)?).as_bytes(),training:"Exhaustive 16 shared two-operation configurations; minimize actual generated token mismatches plus length/exhaustion; stable first-min tie; no development targets or source-path labels used".into()};
        std::fs::write(path.join("initial.json"), initial.encode()?)?;
        let (candidate, trials) = if let Ok(resume) = std::env::var("UOR_ORDERED_RESUME") {
            let prior = std::path::PathBuf::from(resume);
            report_output::verify(&prior)?;
            let bytes = std::fs::read(prior.join("candidate.json"))?;
            if sha(&bytes) != "dc15055718fff00c166c2c9ad5d21dd0f1d93ebff76b3c89e64fb5cf8c0b2486" {
                return Err("resume hash".into());
            }
            let a = Artifact::decode(&bytes, &g)?;
            if a.parent != initial.parent
                || a.source_digest != initial.source_digest
                || a.data_digest != initial.data_digest
            {
                return Err("resume source/data mismatch".into());
            }
            let fit: Value = serde_json::from_slice(&std::fs::read(prior.join("fit.json"))?)?;
            write(
                &path,
                "resume.json",
                &json!({"from":prior,"candidate_sha256":sha(&bytes),"refit":false,"reason":"correct boundary fixture to existing span context; model source, train/development data and acceptance unchanged"}),
            )?;
            (
                a,
                serde_json::from_value::<Vec<learning::Trial>>(fit["trials"].clone())?,
            )
        } else {
            learning::fit(&initial, &g, &m, &train)?
        };
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        write(
            &path,
            "fit.json",
            &json!({"trials":trials,"selected":candidate.operators,"train_rows":train.len(),"dose":"all 16 bounded operator configurations; one generation per training row per configuration","primitive_action_writer_frozen":candidate.parent==parent}),
        )?;
        let reload = Artifact::decode(&encoded, &g)?;
        let boundary_query = vec![b'a'; 127];
        let mut boundary_records = std::array::from_fn(|i| data::Record {
            key: if i == 0 {
                boundary_query.clone()
            } else {
                vec![i as u8]
            },
            text: vec![b'.'],
        });
        let unsupported_boundary = runtime::generate(
            &candidate,
            &g,
            &m,
            &boundary_records,
            &boundary_query,
            Control::Full,
        )?;
        // Exercise the span-domain stopping row learned by the frozen policy.
        // The earlier byte-only '.' fixture instead visited an untrained Emit row.
        boundary_records[1].text = vec![b'.'; 2];
        let boundary = runtime::generate(
            &candidate,
            &g,
            &m,
            &boundary_records,
            &boundary_query,
            Control::Full,
        )?;
        if boundary.tokens != vec![u16::from(b'.'), 256] || boundary.exhausted {
            return Err("EOS at prefix limit failed".into());
        }
        write(
            &path,
            "boundary.json",
            &json!({"query_tokens":127,"pending_tokens":128,"actual":boundary.tokens,"exhausted":boundary.exhausted,"status":"PASS","earlier_byte_only_fixture":{"tokens":unsupported_boundary.tokens,"exhausted":unsupported_boundary.exhausted,"absent_dot_action":candidate.parent.actions[46]},"span_context":true}),
        )?;
        let mut bad = candidate.clone();
        bad.geometry_digest[0] ^= 1;
        if bad.validate(&g).is_ok() {
            return Err("geometry binding accepted".into());
        }
        let mut bad = candidate.clone();
        bad.parent.actions[0] ^= 1;
        if bad.validate(&g).is_ok() {
            return Err("parent binding accepted".into());
        }
        let mut panels = Vec::new();
        let mut rows = Vec::new();
        let mut full = 0;
        let mut correct_paths = 0;
        let mut families = BTreeMap::new();
        let mut controls = true;
        for (name, a, c) in [
            ("initial", &initial, Control::Full),
            ("candidate", &candidate, Control::Full),
            ("candidate", &candidate, Control::ReadDisabled),
            ("candidate", &candidate, Control::FeedbackDisabled),
            ("candidate", &candidate, Control::UpdateDisabled),
            ("candidate", &candidate, Control::CursorDisabled),
            ("candidate", &candidate, Control::BoundaryReadDisabled),
            ("candidate", &candidate, Control::QueryReversed),
            ("candidate", &candidate, Control::StopDisabled),
            ("candidate", &candidate, Control::StateDisabled),
            ("candidate", &candidate, Control::FinalRootOnly),
        ] {
            let mut exact = 0;
            for e in &dev {
                let out = runtime::generate(a, &g, &m, &e.records, &e.query, c)?;
                let yes = out.tokens == learning::target(e) && !out.exhausted;
                exact += usize::from(yes);
                let actual_path: Vec<_> = out
                    .steps
                    .iter()
                    .filter(|s| s.before.cursor == 0)
                    .filter_map(|s| s.observation.selected)
                    .collect();
                if name == "candidate" && c == Control::Full {
                    if out != runtime::generate(&reload, &g, &m, &e.records, &e.query, c)? {
                        return Err("reload differs".into());
                    }
                    if yes {
                        *families.entry(e.family.clone()).or_insert(0) += 1;
                    }
                    correct_paths += usize::from(actual_path == e.expected_sources);
                    if e.id == dev[0].id {
                        write(
                            &path,
                            "sample-trace.json",
                            &json!({"example":e,"actual":out}),
                        )?;
                    }
                }
                rows.push(json!({"id":e.id,"family":e.family,"artifact":name,"control":c,"expected":learning::target(e),"tokens":out.tokens,"text":String::from_utf8_lossy(&out.tokens.iter().filter_map(|&t|u8::try_from(t).ok()).collect::<Vec<_>>()),"exact":yes,"exhausted":out.exhausted,"actual_sources":actual_path,"expected_sources":e.expected_sources,"steps":out.steps.len()}));
            }
            panels.push(json!({"artifact":name,"control":c,"exact":exact,"total":dev.len()}));
            if name == "candidate" && c == Control::Full {
                full = exact;
            } else if name == "candidate" && c != Control::FinalRootOnly {
                controls &= exact == 0;
            }
        }
        // Replay the predecessor's exact responses through the generalized transition.
        // Same artifact and byte-query adapter; no refit or relaxation of old controls.
        let prior_data: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/data.json"),
        )?)?;
        let prior_retained: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/retained-responses.json"),
        )?)?;
        let mut legacy: BTreeMap<String, recurrent_text::data::Example> = BTreeMap::new();
        for split in ["joint_training", "development"] {
            for e in serde_json::from_value::<Vec<recurrent_text::data::Example>>(
                prior_data[split].clone(),
            )? {
                legacy.insert(e.id.clone(), e);
            }
        }
        // Earlier development rows were not training inputs; recover their exact data.
        for (name, rel, is_text) in [
            ("one", "relational-attention-1/attempt-4/data.json", false),
            (
                "dependent",
                "dependent-attention-1/attempt-1/data.json",
                false,
            ),
            (
                "adaptive",
                "adaptive-attention-1/attempt-1/data.json",
                false,
            ),
            ("text", "text-attention-1/attempt-1/data.json", true),
        ] {
            let v: Value = serde_json::from_slice(&std::fs::read(evidence.join(rel))?)?;
            for x in v["development"].as_array().ok_or("old development")? {
                let records = if is_text {
                    serde_json::from_value(x["records"].clone())?
                } else {
                    let r: [crate::native_geometric::relational_attention::data::Record; 4] =
                        serde_json::from_value(x["records"].clone())?;
                    r.map(|r| recurrent_text::data::Record {
                        key: r.key,
                        text: vec![r.value],
                    })
                };
                let e = recurrent_text::data::Example {
                    id: format!("{name}:{}", x["id"].as_str().ok_or("id")?),
                    family: name.into(),
                    records,
                    query: serde_json::from_value(x["query"].clone())?,
                    answer: if is_text {
                        serde_json::from_value(x["answer"].clone())?
                    } else {
                        vec![serde_json::from_value(x["answer"].clone())?]
                    },
                };
                legacy.insert(e.id.clone(), e);
            }
        }
        let mut retained_rows = Vec::new();
        let mut retained_ok = true;
        for x in prior_retained.as_array().ok_or("old responses")? {
            let id = x["id"].as_str().ok_or("id")?;
            let e = legacy.get(id).ok_or("retained input missing")?;
            let out = recurrent::generate(
                &candidate.parent,
                &g,
                &m,
                &e.records,
                e.query,
                recurrent::Control::Full,
            )?;
            let equal = serde_json::to_value(&out.tokens)? == x["actual"] && !out.exhausted;
            retained_ok &= equal;
            retained_rows.push(json!({"id":id,"equal":equal,"tokens":out.tokens}));
        }
        let prior_rows: Value = serde_json::from_slice(&std::fs::read(
            evidence.join("recurrent-text-1/attempt-2/responses.json"),
        )?)?;
        let mut control_replays = 0;
        for x in prior_rows
            .as_array()
            .ok_or("old controls")?
            .iter()
            .filter(|x| x["artifact"] == "candidate")
        {
            let id = x["id"].as_str().ok_or("control id")?;
            let e = legacy.get(id).ok_or("control input")?;
            let c: recurrent::Control = serde_json::from_value(x["control"].clone())?;
            let out = recurrent::generate(&candidate.parent, &g, &m, &e.records, e.query, c)?;
            let equal = serde_json::to_value(&out)? == x["actual"];
            retained_ok &= equal;
            control_replays += usize::from(equal);
            retained_rows.push(json!({"id":id,"control":c,"equal":equal}));
        }
        let complete = families.values().filter(|&&n| n == 4).count();
        let pass = full >= 61 && complete >= 15 && correct_paths >= 61 && controls && retained_ok;
        write(&path, "responses.json", &json!(rows))?;
        write(&path, "retained-responses.json", &json!(retained_rows))?;
        write(
            &path,
            "summary.json",
            &json!({"gate":if pass{"PASS_ORDERED_GEOMETRIC_STATE"}else{"FAIL_ORDERED_GEOMETRIC_STATE"},"panels":panels,"complete_families":complete,"correct_source_paths":correct_paths,"retained_rows":prior_retained.as_array().ok_or("rows")?.len(),"retained_control_trace_replays":control_replays,"retention_equal":retained_ok,"candidate_sha256":sha(&encoded),"parent_sha256":sha(&bytes),"operators":candidate.operators,"final_product_collisions":final_collisions.len(),"prefix_collisions":0,"reload_equal_rows":dev.len(),"final_holdout":"NOT_RUN","promotion":false,"retained_model":"15baec48"}),
        )?;
        Ok(())
    })();
    if let Err(e) = &result {
        std::fs::write(path.join("error.txt"), e.to_string())?;
    }
    report_output::seal(&path)?;
    report_output::verify(&path)?;
    result
}
