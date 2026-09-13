//! Bounded test-only report driver; no parameter update or new model artifact.
use super::{
    artifact::Model,
    causal_credit::paired_batch,
    circuit::PrimitiveExport,
    engine::Head,
    learner::Checkpoint,
    pilot::document,
    pilot_data,
    policy::{head_range, PARAMETER_COUNT},
    stability_stats::Stats,
};
use crate::report_output;
use serde::Serialize;
use std::{fs, path::Path, time::Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn save(root: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    fs::write(root.join(name), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn claim(root: &Path) -> Result<()> {
    if !root.is_absolute()
        || root
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("absolute report path without traversal required".into());
    }
    for a in root
        .parent()
        .ok_or("report parent")?
        .canonicalize()?
        .ancestors()
    {
        if a.join("manifest.json").exists() {
            return Err("output beneath a sealed root".into());
        }
    }
    report_output::claim(root)?;
    Ok(())
}
fn scalar(values: &[f64]) -> serde_json::Value {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
    serde_json::json!({"count":values.len(),"mean":mean,"sample_sd":libm::sqrt(variance),"standard_error":libm::sqrt(variance/n),"values":values})
}
fn families() -> Result<Vec<(&'static str, usize, usize)>> {
    let g = head_range(Head::Root(0), 0)?.0;
    let e = head_range(Head::Extent, 0)?.0;
    let c = head_range(Head::Control, 0)?.0;
    let out = head_range(Head::Emission, 0)?.0;
    let null = head_range(Head::Null(0), 0)?.0;
    Ok(vec![
        ("gates", 0, g),
        ("root_heads", g, e),
        ("extent_head", e, c),
        ("control_head", c, out),
        ("emission_head", out, null),
        ("null_heads", null, PARAMETER_COUNT),
    ])
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * (1.0 + a.abs().max(b.abs()))
}
fn ratio(new: f64, old: f64) -> Option<f64> {
    if old <= 1e-24 {
        None
    } else {
        Some(new / old)
    }
}
#[test]
#[ignore = "one projected frozen32-batch paired credit comparison"]
fn frozen_causal_credit_report() -> Result<()> {
    let path = std::env::var("UOR_CAUSAL_REPORT")?;
    let fit_path = std::env::var("UOR_CAUSAL_FIT")?;
    let prior_path = std::env::var("UOR_CAUSAL_PRIOR")?;
    let root = Path::new(&path);
    claim(root)?;
    let started = Instant::now();
    let result = (|| -> Result<()> {
        let fit = Path::new(&fit_path);
        report_output::verify(fit)?;
        let old_config = fs::read(fit.join("config.json"))?;
        let train: Vec<pilot_data::Example> =
            serde_json::from_slice(&fs::read(fit.join("training.json"))?)?;
        if train != pilot_data::training() {
            return Err("frozen training data mismatch".into());
        }
        let initial = Checkpoint::decode(
            &fs::read(fit.join("initial-parameters.bin"))?,
            pilot_data::corpus_digest(&train),
            *blake3::hash(&old_config).as_bytes(),
            973,
        )?;
        let final_cp = Checkpoint::decode(
            &fs::read(fit.join("final-parameters.bin"))?,
            pilot_data::corpus_digest(&train),
            *blake3::hash(&old_config).as_bytes(),
            973,
        )?;
        if initial.completed_updates != 0
            || final_cp.completed_updates != 64
            || initial.parameters.seed() != 7341
            || final_cp.parameters.seed() != 7341
        {
            return Err("checkpoint endpoints".into());
        }
        let model = Model::decode(&fs::read(fit.join("parent-initialized-model.bin"))?)?;
        let initial_export = PrimitiveExport::decode(&fs::read(fit.join("initial-compiled.bin"))?)?;
        let final_export = PrimitiveExport::decode(&fs::read(fit.join("final-compiled.bin"))?)?;
        if initial_export != initial.parameters.compile()?
            || final_export != final_cp.parameters.compile()?
            || model.compiled() != &initial_export
        {
            return Err("loaded compiled/parameter/parent binding".into());
        }

        let prior = Path::new(&prior_path);
        report_output::verify(prior)?;
        let prior_summary: serde_json::Value =
            serde_json::from_slice(&fs::read(prior.join("summary.json"))?)?;
        if prior_summary["status"] != "COMPLETE_FROZEN_GRADIENT_CONTEXT_DIAGNOSTIC"
            || prior_summary["batches"] != 32
        {
            return Err("prior diagnostic identity".into());
        }
        let cfg = serde_json::json!({"schema":"uor-r4.causal-credit-comparison/1","records":["train/memory-a/0","train/sub-9-2/0"],"endpoints":[0,64],"replicates":8,"particles":4,"batches":32,"event_seed":1973,"document_key":"record_slot*8+replicate","updates":0,"fits":0,"runtime_replays":0,"gate_per_cell_ratio_max":1.0,"gate_aggregate_ratio_max":0.8,"other_family_aggregate_ratio_max":1.1,"zero_covariance_tolerance":1e-24,"historical_loss_parity_tolerance":2e-14,"old_stat_relative_absolute_tolerance":1e-12,"selection_covariance":"full gradient; score component separately reported","old_fit_config_digest":blake3::hash(&old_config).to_hex().to_string(),"geometry_digest":hex::encode(model.geometry().identity_digest()),"source_driver":blake3::hash(include_bytes!("causal_report.rs")).to_hex().to_string(),"source_credit":blake3::hash(include_bytes!("causal_credit.rs")).to_hex().to_string(),"scope":"frozen descriptive estimator comparison, not language qualification"});
        save(root, "config.json", &cfg)?;
        let params = [&initial.parameters, &final_cp.parameters];
        let before = params.map(|p| p.digest());
        let ranges = families()?;
        let mut cells = Vec::new();
        let mut aggregates = vec![[0.0f64; 2]; ranges.len()];
        let mut gate_all_cells = true;
        let mut batches = 0;
        let mut max_tape_bytes = 0usize;
        let mut total_batch_us = 0u128;
        for (record_slot, &example_index) in [0usize, 8].iter().enumerate() {
            let ex = &train[example_index];
            let symbols = document(ex);
            for endpoint in 0..2 {
                let mut stats: Vec<Vec<Stats>> = (0..5)
                    .map(|_| {
                        ranges
                            .iter()
                            .map(|(_, a, b)| Stats::new(b - a))
                            .collect::<std::result::Result<Vec<_>, _>>()
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let mut losses = Vec::new();
                let mut rep_info = Vec::new();
                for replicate in 0..8 {
                    if started.elapsed().as_millis() >= 18000 {
                        return Err("comparison scheduling stop at18s with2s seal margin".into());
                    }
                    let b = paired_batch(
                        params[endpoint],
                        model.geometry(),
                        &symbols,
                        ex.prompt.len(),
                        1973,
                        (record_slot * 8 + replicate) as u64,
                    )?;
                    if b.particles.len() != 4
                        || b.counts.scored_positions != 4 * symbols.len() as u64
                    {
                        return Err("comparison path accounting".into());
                    }
                    let old: serde_json::Value = serde_json::from_slice(&fs::read(
                        prior.join(format!("rep-{record_slot}-{endpoint}-{replicate}.json")),
                    )?)?;
                    if serde_json::to_value(&b.particles)? != old["particles"]
                        || serde_json::to_value(b.counts)? != old["counts"]
                    {
                        return Err("old/new path or event parity".into());
                    }
                    for (k, v) in [
                        ("mean_ce", b.mean_ce),
                        ("prompt_ce", b.prompt_ce),
                        ("response_ce", b.response_ce),
                    ] {
                        let reference = old[k].as_f64().ok_or("prior loss")?;
                        if (v - reference).abs() > 2e-14 {
                            return Err("old/new retained loss parity".into());
                        }
                    }
                    let old_score: Vec<_> = b
                        .old_gradient
                        .iter()
                        .zip(&b.direct)
                        .map(|(x, d)| x - d)
                        .collect();
                    let causal_score: Vec<_> = b
                        .causal_gradient
                        .iter()
                        .zip(&b.direct)
                        .map(|(x, d)| x - d)
                        .collect();
                    let delta: Vec<_> = b
                        .causal_gradient
                        .iter()
                        .zip(&b.old_gradient)
                        .map(|(a, b)| a - b)
                        .collect();
                    for (i, values) in [
                        &b.old_gradient,
                        &b.causal_gradient,
                        &old_score,
                        &causal_score,
                        &delta,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        for (f, (_, a, z)) in ranges.iter().enumerate() {
                            stats[i][f].add(&values[*a..*z])?;
                        }
                    }
                    let rep = serde_json::json!({"replicate":replicate,"event_document_id":record_slot*8+replicate,"mean_ce":b.mean_ce,"prompt_ce":b.prompt_ce,"response_ce":b.response_ce,"counts":b.counts,"particles":b.particles,"tape_bytes":b.tape_bytes,"elapsed_us":b.elapsed_us,"prior_path_count_loss_parity":true,"same_forward_paths":true,"runtime_replays":0});
                    save(
                        root,
                        &format!("rep-{record_slot}-{endpoint}-{replicate}.json"),
                        &rep,
                    )?;
                    max_tape_bytes = max_tape_bytes.max(b.tape_bytes);
                    total_batch_us += b.elapsed_us;
                    losses.push(b.mean_ce);
                    rep_info.push(serde_json::json!({"replicate":replicate,"tape_bytes":b.tape_bytes,"elapsed_us":b.elapsed_us}));
                    batches += 1;
                }
                let mut families_out = Vec::new();
                for (f, (name, _, _)) in ranges.iter().enumerate() {
                    let old_full = stats[0][f].finish()?;
                    let causal_full = stats[1][f].finish()?;
                    let old_score = stats[2][f].finish()?;
                    let causal_score = stats[3][f].finish()?;
                    let difference = stats[4][f].finish()?;
                    let prior_g = prior_summary["cells"][record_slot * 2 + endpoint]
                        ["gradient_components"]
                        .as_array()
                        .ok_or("prior gradients")?
                        .iter()
                        .find(|g| g["family"] == *name && g["component"] == "full")
                        .ok_or("prior family")?;
                    let prior_stat: &serde_json::Value = &prior_g["statistics"];
                    for (k, v) in [
                        ("mean_l2", old_full.mean_l2),
                        ("trace_sample_covariance", old_full.trace_sample_covariance),
                        ("standard_error_l2", old_full.standard_error_l2),
                    ] {
                        if !close(v, prior_stat[k].as_f64().ok_or("prior stat")?) {
                            return Err("old estimator statistic parity".into());
                        }
                    }
                    let ov = old_full.trace_sample_covariance;
                    let cv = causal_full.trace_sample_covariance;
                    aggregates[f][0] += ov;
                    aggregates[f][1] += cv;
                    if f == 0 && cv > ov {
                        gate_all_cells = false;
                    }
                    families_out.push(serde_json::json!({"family":name,"old_full":old_full,"causal_full":causal_full,"old_score":old_score,"causal_score":causal_score,"paired_difference":difference,"full_covariance_ratio":ratio(cv,ov),"score_covariance_ratio":ratio(causal_score.trace_sample_covariance,old_score.trace_sample_covariance)}));
                }
                let cell = serde_json::json!({"record":ex.id,"endpoint":(["initial","final64"][endpoint]),"mean_ce":scalar(&losses),"parameter_digest":hex::encode(params[endpoint].digest()),"families":families_out,"replicates":rep_info,"prior_path_count_loss_and_old_stats_parity":true});
                save(root, &format!("cell-{record_slot}-{endpoint}.json"), &cell)?;
                cells.push(cell);
            }
        }
        let mut selection = gate_all_cells;
        let mut aggregate_out = Vec::new();
        for (f, (name, _, _)) in ranges.iter().enumerate() {
            let [old, new] = aggregates[f];
            let limit = if f == 0 { 0.8 } else { 1.1 };
            let passed = if old <= 1e-24 {
                new <= 1e-24
            } else {
                new / old <= limit
            };
            selection &= passed;
            aggregate_out.push(serde_json::json!({"family":name,"old_sum_covariance":old,"causal_sum_covariance":new,"ratio":ratio(new,old),"limit":limit,"passes":passed}));
        }
        let after = params.map(|p| p.digest());
        if before != after || batches != 32 {
            return Err("frozen comparison accounting".into());
        }
        let status = if selection {
            "SELECTED_CAUSAL_CREDIT_FOR_BOUNDED_LEARNING"
        } else {
            "NOT_SELECTED_CAUSAL_CREDIT"
        };
        save(
            root,
            "summary.json",
            &serde_json::json!({"status":status,"execution":"COMPLETE_PAIRED_CAUSAL_CREDIT_COMPARISON","batches":batches,"trajectories":batches*4,"runtime_replays":0,"parameter_updates":0,"fit_calls":0,"promotion":false,"prior_model_gate":"FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT","cells":cells,"family_aggregates":aggregate_out,"gate_all_cells_nonincreasing":gate_all_cells,"max_tape_bytes":max_tape_bytes,"total_batch_us":total_batch_us,"elapsed_us":started.elapsed().as_micros(),"parameter_digests_before":before.map(hex::encode),"parameter_digests_after":after.map(hex::encode),"source_fit_root":fit_path,"prior_diagnostic_root":prior_path,"mean_consistency":"paired differences descriptive, no statistical proof or acceptance from cosine"}),
        )?;
        println!(
            "{status} batches={batches} elapsed_us={}",
            started.elapsed().as_micros()
        );
        Ok(())
    })();
    if let Err(e) = &result {
        save(
            root,
            "incomplete.json",
            &serde_json::json!({"status":"INCOMPLETE_CAUSAL_CREDIT_COMPARISON","error":e.to_string(),"updates":0,"fits":0}),
        )?;
    }
    report_output::seal(root)?;
    report_output::verify(root)?;
    result
}
