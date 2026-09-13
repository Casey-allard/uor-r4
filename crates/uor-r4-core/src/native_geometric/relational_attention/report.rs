use super::{
    circuit, data, learning,
    runtime::{self, Control},
};
use crate::{
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::json;
#[test]
#[ignore = "bounded actual attention curriculum; exclusive report directory required"]
fn relational_attention_learning_report() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(std::env::var("UOR_RELATIONAL_ATTENTION_REPORT")?);
    if path.as_os_str().is_empty() {
        return Err("empty report".into());
    }
    report_output::claim(&path)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let (train, dev) = data::corpus()?;
        let validation = data::validate(&train, &dev)?;
        let config = learning::Config::default();
        let acceptance = json!({"status":"FROZEN_BEFORE_FIT","task":"typed ordered two-byte keys, four exact records, one-byte payload; actual byte then EOS","dev_exact_min":61,"dev_total":64,"dev_selected_min":61,"dev_pair_both_min":31,"dev_pairs":32,"improvement_over_codec_only_min":32,"read_disabled_exact_max":0,"query_reversed_exact_max":0,"training_dose":config,"old_repair_parity_required":false,"final_holdout":"NOT_RUN","scope":"learned compatibility with fixed canonical query/key and learned codec; not full jointly trained recurrent H1 or prose"});
        std::fs::write(
            path.join("acceptance.json"),
            serde_json::to_vec_pretty(&acceptance)?,
        )?;
        std::fs::write(
            path.join("data.json"),
            serde_json::to_vec_pretty(
                &json!({"training":train,"development":dev,"validation":validation}),
            )?,
        )?;
        let mut collisions = 0;
        let mut reversed_distinct = 0;
        for e in train.iter().chain(&dev) {
            let target = runtime::features(&runtime::relation(&g, &m, e.query, e.query)?);
            for r in &e.records {
                let bits = runtime::features(&runtime::relation(&g, &m, e.query, r.key)?);
                if r.key != e.query && bits == target {
                    collisions += 1;
                }
                if r.key == [e.query[1], e.query[0]] && bits != target {
                    reversed_distinct += 1;
                }
            }
        }
        std::fs::write(
            path.join("environment.json"),
            serde_json::to_vec_pretty(
                &json!({"target_distractor_packet_collisions":collisions,"reversed_query_distinguished":reversed_distinct,"records":train.len()+dev.len(),"admitted_per_query":4,"answer_not_in_runtime_input":true,"typed_boundary_not_raw_text":true,"read_disabled_scope":"prediction payload access only; no input-ingestion recurrence exists in this scaffold","scorer_features":runtime::FEATURES,"all_pair_block":"4x4 across each query/key's four roots; not all-token context graph","representation":"four ordered H4 compute roots, not a claimed paired-H4 icosian lift"}),
            )?,
        )?;
        if collisions != 0 {
            return Err("environment aliases target and distractor".into());
        }
        let (initial, _, _) = learning::initialized(&g, &train)?;
        std::fs::write(path.join("initialized.json"), initial.encode()?)?;
        let (candidate, fit) = learning::fit(&g, &m, &train, config)?;
        let encoded = candidate.encode()?;
        std::fs::write(path.join("candidate.json"), &encoded)?;
        std::fs::write(path.join("fit.json"), serde_json::to_vec_pretty(&fit)?)?;
        let restored = runtime::Artifact::decode(&encoded, &g)?;
        let mut codec_only = initial.clone();
        codec_only.decode_tables = candidate.decode_tables.clone();
        codec_only.training = "same fitted decoder; uniform initial compatibility".into();
        std::fs::write(path.join("codec-only.json"), codec_only.encode()?)?;
        let mut rows = Vec::new();
        let mut summary = Vec::new();
        let mut full_dev = 0;
        let mut codec_dev = 0;
        let mut selected_dev = 0;
        let mut disabled_dev = 0;
        let mut reverse_dev = 0;
        let mut exact_families = std::collections::BTreeMap::new();
        for (split, examples) in [("training", &train), ("development", &dev)] {
            for (name, a) in [
                ("initialized", &initial),
                ("codec_only", &codec_only),
                ("candidate", &candidate),
            ] {
                for control in [
                    Control::Full,
                    Control::ReadDisabled,
                    Control::QueryReversed,
                    Control::PhasesDisabled,
                    Control::DiagonalOnly,
                ] {
                    if name != "candidate" && control != Control::Full {
                        continue;
                    }
                    let mut exact = 0;
                    let mut selected = 0;
                    for e in examples {
                        let out = runtime::generate(a, &g, &m, &e.records, e.query, control)?;
                        let correct = out.tokens == [u16::from(e.answer), 256];
                        let chosen = out.selected.is_some_and(|i| e.records[i].key == e.query);
                        exact += usize::from(correct);
                        selected += usize::from(chosen);
                        if name == "candidate" && control == Control::Full {
                            let replay =
                                runtime::generate(&restored, &g, &m, &e.records, e.query, control)?;
                            if replay != out {
                                return Err("export replay differs".into());
                            }
                            if split == "development" {
                                *exact_families.entry(e.family.clone()).or_insert(0usize) +=
                                    usize::from(correct);
                            }
                        }
                        rows.push(json!({"split":split,"artifact":name,"control":control,"id":e.id,"family":e.family,"query":e.query,"records":e.records,"expected":[u16::from(e.answer),256],"exact":correct,"selected_correct":chosen,"actual":out}));
                    }
                    if split == "development" {
                        match (name, control) {
                            ("candidate", Control::Full) => {
                                full_dev = exact;
                                selected_dev = selected
                            }
                            ("codec_only", Control::Full) => codec_dev = exact,
                            ("candidate", Control::ReadDisabled) => disabled_dev = exact,
                            ("candidate", Control::QueryReversed) => reverse_dev = exact,
                            _ => {}
                        }
                    }
                    summary.push(json!({"split":split,"artifact":name,"control":control,"exact":exact,"selected_correct":selected,"total":examples.len()}));
                }
            }
        }
        let pairs = exact_families.values().filter(|&&n| n == 2).count();
        let pass = full_dev >= 61
            && selected_dev >= 61
            && pairs >= 31
            && full_dev >= codec_dev + 32
            && disabled_dev == 0
            && reverse_dev == 0
            && fit.stopped == "completed_dose";
        std::fs::write(path.join("responses.json"), serde_json::to_vec(&rows)?)?;
        std::fs::write(
            path.join("summary.json"),
            serde_json::to_vec_pretty(
                &json!({"gate":if pass{"PASS_TYPED_ONE_READ_ATTENTION_LEARNING"}else{"FAIL_TYPED_ONE_READ_ATTENTION_LEARNING"},"environment":"PASS_NO_TARGET_DISTRACTOR_PACKET_COLLISION","panels":summary,"development_pairs_both_exact":pairs,"candidate_sha256":hex::encode(sha2::Digest::finalize(sha2::Sha256::new_with_prefix(&encoded))),"candidate_parameter_changes":{"score_tables":initial.score_tables.iter().zip(&candidate.score_tables).filter(|(a,b)|a!=b).count(),"decoder_tables":initial.decode_tables.iter().zip(&candidate.decode_tables).filter(|(a,b)|a!=b).count()},"export_replay_equal":true,"source":{"runtime":blake3::hash(include_bytes!("runtime.rs")).to_hex().to_string(),"learning":blake3::hash(include_bytes!("learning.rs")).to_hex().to_string(),"circuit":blake3::hash(include_bytes!("circuit.rs")).to_hex().to_string(),"data":blake3::hash(include_bytes!("data.rs")).to_hex().to_string()},"scope":"typed compatibility learning with codec curriculum; query/key encoding and record boundaries fixed; not raw prose or complete shared recurrent learner","retained_artifact":"15baec48","promotion":false,"final_holdout":"NOT_RUN"}),
            )?,
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
use sha2::Digest;
#[test]
fn relation_orientation_and_exact_export_validation() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    let q = *b"AB";
    let k = *b"CD";
    let ab = runtime::relation(&g, &m, q, k)?;
    let ba = runtime::relation(&g, &m, k, q)?;
    for i in 0..4 {
        for j in 0..4 {
            assert_eq!(
                g.product(ab.directed[i][j], ba.directed[j][i])?,
                g.identity()
            );
        }
    }
    assert_ne!(
        runtime::features(&runtime::relation(&g, &m, q, q)?),
        runtime::features(&runtime::relation(&g, &m, q, *b"BA")?)
    );
    let (train, _) = data::corpus()?;
    let (mut a, _, _) = learning::initialized(&g, &train)?;
    assert_eq!(a, runtime::Artifact::decode(&a.encode()?, &g)?);
    a.score_tables[0] = 255;
    assert!(a.validate(&g).is_err());
    let (s, _) = learning::structures();
    let tables = vec![circuit::AND; s.nodes.len()];
    let mut input = vec![true; runtime::FEATURES];
    assert!(circuit::hard(&s, &tables, &input)?.outputs[0]);
    for bit in 0..runtime::FEATURES {
        input[bit] = false;
        assert!(!circuit::hard(&s, &tables, &input)?.outputs[0]);
        input[bit] = true;
    }
    assert_eq!(circuit::hard(&s, &tables, &input)?.outputs.len(), 1);
    Ok(())
}
