//! Bounded test-only report driver; no parameter update or new model artifact.
use super::{
    artifact::Model,
    circuit::PrimitiveExport,
    engine::Head,
    learner::Checkpoint,
    pilot::document,
    pilot_data,
    policy::{head_range, PARAMETER_COUNT},
    stability::{deterministic_trace, instrumented_batch, ParticleTrace},
    stability_stats::Stats,
};
use crate::report_output;
use serde::Serialize;
use std::{collections::BTreeSet, fs, path::Path, time::Instant};
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
fn overlap(
    reference: &ParticleTrace,
    samples: &[ParticleTrace],
    prompt_len: usize,
) -> Result<serde_json::Value> {
    let positions = reference.emission_contexts.len();
    if positions == 0 || samples.is_empty() {
        return Err("empty aligned trace".into());
    }
    let mut same_context = 0;
    let mut response_same = 0;
    let mut same_root_lanes = 0;
    let mut same_root_tuples = 0;
    let mut same_selected_lanes = 0;
    let mut same_nonnull = 0;
    let mut prediction_differences = 0;
    let mut response_differences = 0;
    let mut support: Vec<BTreeSet<u16>> = (0..positions).map(|_| BTreeSet::new()).collect();
    for sample in samples {
        if sample.emission_contexts.len() != positions
            || sample.roots_before.len() != positions
            || sample.selected.len() != positions
            || sample.offered_symbols.len() != positions
        {
            return Err("trace length mismatch".into());
        }
        for i in 0..positions {
            let equal = sample.emission_contexts[i] == reference.emission_contexts[i];
            same_context += usize::from(equal);
            if i >= prompt_len {
                response_same += usize::from(equal);
            }
            support[i].insert(sample.emission_contexts[i]);
            same_root_tuples += usize::from(sample.roots_before[i] == reference.roots_before[i]);
            for lane in 0..4 {
                same_root_lanes +=
                    usize::from(sample.roots_before[i][lane] == reference.roots_before[i][lane]);
            }
            for lane in 0..2 {
                let same = sample.selected[i][lane] == reference.selected[i][lane];
                same_selected_lanes += usize::from(same);
                same_nonnull += usize::from(same && sample.selected[i][lane].is_some());
            }
            let diff = sample.offered_symbols[i] != reference.offered_symbols[i];
            prediction_differences += usize::from(diff);
            if i >= prompt_len {
                response_differences += usize::from(diff);
            }
        }
    }
    let total = positions * samples.len();
    let response_total = (positions - prompt_len) * samples.len();
    let covered = support
        .iter()
        .enumerate()
        .filter(|(i, s)| s.contains(&reference.emission_contexts[*i]))
        .count();
    Ok(
        serde_json::json!({"positions":positions,"sample_trajectories":samples.len(),"aligned_positions":total,"same_emission_context":same_context,"context_match_fraction":same_context as f64/total as f64,"response_aligned_positions":response_total,"response_same_context":response_same,"response_context_match_fraction":response_same as f64/response_total as f64,"same_root_lanes":same_root_lanes,"root_lane_comparisons":total*4,"same_root_tuples":same_root_tuples,"same_local_selected_lanes":same_selected_lanes,"selected_lane_comparisons":total*2,"same_nonnull_local_reference":same_nonnull,"prediction_hamming":prediction_differences,"response_prediction_hamming":response_differences,"deterministic_context_in_sampled_support_positions":covered,"per_position_unique_sampled_contexts":support.iter().map(BTreeSet::len).collect::<Vec<_>>(),"reference_scope":"same-document relative occurrence turn/start/end or result ordinal; matching result ordinal is not derivation equality"}),
    )
}
#[test]
fn stability_family_partition_is_exact_and_head_aligned() -> Result<()> {
    let f = families()?;
    assert_eq!(f.first().map(|x| x.1), Some(0));
    assert_eq!(f.last().map(|x| x.2), Some(PARAMETER_COUNT));
    for w in f.windows(2) {
        assert_eq!(w[0].2, w[1].1);
    }
    assert_eq!(f[0].2, 5520);
    assert_eq!(f[4], ("emission_head", 533392, 664976));
    Ok(())
}
#[test]
#[ignore = "one projected zero-update32-batch diagnostic, exclusive report root"]
fn frozen_stability_report() -> Result<()> {
    let path = std::env::var("UOR_STABILITY_REPORT")?;
    let fit_path = std::env::var("UOR_STABILITY_FIT")?;
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
        let cfg = serde_json::json!({"schema":"uor-r4.addressed-stability/1","record_ids":["train/memory-a/0","train/sub-9-2/0"],"endpoints":[0,64],"replicates":8,"particles":4,"batches":32,"event_seed":1973,"document_key":"recordslot*8+replicate, same endpoint pairing; divergent paths may consume different event keys","parameter_updates":0,"fit_calls":0,"old_fit_config_digest":blake3::hash(&old_config).to_hex().to_string(),"geometry_digest":hex::encode(model.geometry().identity_digest()),"source_driver":blake3::hash(include_bytes!("stability_report.rs")).to_hex().to_string(),"source_instrumentation":blake3::hash(include_bytes!("stability.rs")).to_hex().to_string(),"source_statistics":blake3::hash(include_bytes!("stability_stats.rs")).to_hex().to_string(),"statistics":"8 batch vectors: sample covariance trace, mean standard-error norm, signed debiased mean-norm-squared, nonzero pairwisecosine; no confidence claim","scope":"two fixed construction fixtures, no held-out or generation qualification"});
        save(root, "config.json", &cfg)?;
        let endpoint_params = [&initial.parameters, &final_cp.parameters];
        let endpoint_exports = [&initial_export, &final_export];
        let names = ["initial", "final64"];
        let before = endpoint_params.map(|p| p.digest());
        let ranges = families()?;
        let mut cells = Vec::new();
        let mut endpoint_differences = Vec::new();
        let mut batches = 0;
        for (record_slot, &example_index) in [0usize, 8].iter().enumerate() {
            let example = &train[example_index];
            let symbols = document(example);
            let mut paired_losses: Vec<Vec<[f64; 3]>> = Vec::new();
            let mut deterministic = Vec::new();
            for endpoint in 0..2 {
                let params = endpoint_params[endpoint];
                let det = deterministic_trace(
                    params,
                    endpoint_exports[endpoint],
                    model.geometry(),
                    &symbols,
                    example.prompt.len(),
                )?;
                let mut statistics: Vec<Vec<Stats>> = (0..3)
                    .map(|_| {
                        ranges
                            .iter()
                            .map(|(_, a, b)| Stats::new(b - a))
                            .collect::<std::result::Result<Vec<_>, _>>()
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let mut replicates = Vec::new();
                let mut all_traces = Vec::new();
                let mut losses = Vec::new();
                for replicate in 0..8 {
                    if started.elapsed().as_millis() >= 18000 {
                        return Err(
                            "diagnostic scheduling stop at18s to retain2s sealing margin".into(),
                        );
                    }
                    let batch = instrumented_batch(
                        params,
                        model.geometry(),
                        &symbols,
                        example.prompt.len(),
                        1973,
                        (record_slot * 8 + replicate) as u64,
                    )?;
                    if batch.particles.len() != 4
                        || batch.counts.scored_positions != 4 * symbols.len() as u64
                    {
                        return Err("batch/particle accounting".into());
                    }
                    for (component, values) in [&batch.gradient, &batch.direct, &batch.score_credit]
                        .into_iter()
                        .enumerate()
                    {
                        for (family, (_, a, b)) in ranges.iter().enumerate() {
                            statistics[component][family].add(&values[*a..*b])?;
                        }
                    }
                    losses.push([batch.mean_ce, batch.prompt_ce, batch.response_ce]);
                    let r = serde_json::json!({"replicate":replicate,"event_document_id":record_slot*8+replicate,"mean_ce":batch.mean_ce,"prompt_ce":batch.prompt_ce,"response_ce":batch.response_ce,"counts":batch.counts,"elapsed_us":batch.elapsed_us,"particles":batch.particles});
                    save(
                        root,
                        &format!("rep-{record_slot}-{endpoint}-{replicate}.json"),
                        &r,
                    )?;
                    all_traces.extend(batch.particles);
                    replicates.push(serde_json::json!({"replicate":replicate,"mean_ce":batch.mean_ce,"prompt_ce":batch.prompt_ce,"response_ce":batch.response_ce,"elapsed_us":batch.elapsed_us}));
                    batches += 1;
                }
                let mut gradients = Vec::new();
                for (component, group) in statistics.iter().enumerate() {
                    for (family, stat) in group.iter().enumerate() {
                        gradients.push(serde_json::json!({"component":(["full","direct","score_credit"][component]),"family":ranges[family].0,"start":ranges[family].1,"end":ranges[family].2,"statistics":stat.finish()?}));
                    }
                }
                let mut component_covariance = Vec::new();
                for (family, (name, _, _)) in ranges.iter().enumerate() {
                    let full = statistics[0][family].finish()?;
                    let direct = statistics[1][family].finish()?;
                    let score = statistics[2][family].finish()?;
                    component_covariance.push(serde_json::json!({"family":name,"direct_score_cross_covariance_trace":(full.trace_sample_covariance-direct.trace_sample_covariance-score.trace_sample_covariance)/2.0,"note":"same paired batch vectors; components are not assumed independent"}));
                }
                let means: Vec<f64> = losses.iter().map(|l| l[0]).collect();
                let prompts: Vec<f64> = losses.iter().map(|l| l[1]).collect();
                let responses: Vec<f64> = losses.iter().map(|l| l[2]).collect();
                let agreement = overlap(&det.trace, &all_traces, example.prompt.len())?;
                let cell = serde_json::json!({"record":example.id,"record_slot":record_slot,"endpoint":names[endpoint],"parameter_digest":hex::encode(params.digest()),"prompt_positions":example.prompt.len(),"response_positions":symbols.len()-example.prompt.len(),"sampled_mean_ce":scalar(&means),"sampled_prompt_ce":scalar(&prompts),"sampled_response_ce":scalar(&responses),"deterministic":det,"sampled_vs_deterministic":agreement,"gradient_components":gradients,"component_covariance":component_covariance,"replicates":replicates});
                save(root, &format!("cell-{record_slot}-{endpoint}.json"), &cell)?;
                cells.push(cell);
                paired_losses.push(losses);
                deterministic.push(det);
            }
            let differences: Vec<[f64; 3]> = paired_losses[0]
                .iter()
                .zip(&paired_losses[1])
                .map(|(a, b)| [b[0] - a[0], b[1] - a[1], b[2] - a[2]])
                .collect();
            endpoint_differences.push(serde_json::json!({"record":example.id,"paired_configuration_final_minus_initial":{"all":scalar(&differences.iter().map(|l|l[0]).collect::<Vec<_>>()),"prompt":scalar(&differences.iter().map(|l|l[1]).collect::<Vec<_>>()),"response":scalar(&differences.iter().map(|l|l[2]).collect::<Vec<_>>())},"deterministic_final_minus_initial":{"all":deterministic[1].mean_ce-deterministic[0].mean_ce,"prompt":deterministic[1].prompt_ce-deterministic[0].prompt_ce,"response":deterministic[1].response_ce-deterministic[0].response_ce},"deterministic_endpoint_alignment":overlap(&deterministic[0].trace,std::slice::from_ref(&deterministic[1].trace),example.prompt.len())?}));
        }
        let after = endpoint_params.map(|p| p.digest());
        if before != after || batches != 32 {
            return Err("frozen diagnostic accounting".into());
        }
        save(
            root,
            "summary.json",
            &serde_json::json!({"status":"COMPLETE_FROZEN_GRADIENT_CONTEXT_DIAGNOSTIC","batches":batches,"trajectories":batches*4,"cells":cells,"endpoint_differences":endpoint_differences,"parameter_digests_before":before.map(hex::encode),"parameter_digests_after":after.map(hex::encode),"parameter_updates":0,"fit_calls":0,"promotion":false,"prior_model_gate":"FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT","elapsed_us":started.elapsed().as_micros(),"source_fit_root":fit_path}),
        )?;
        println!(
            "COMPLETE_FROZEN_GRADIENT_CONTEXT_DIAGNOSTIC batches={batches} elapsed_us={}",
            started.elapsed().as_micros()
        );
        Ok(())
    })();
    if let Err(error) = &result {
        save(
            root,
            "incomplete.json",
            &serde_json::json!({"status":"INCOMPLETE_FROZEN_STABILITY_DIAGNOSTIC","error":error.to_string(),"updates":0,"fit_calls":0}),
        )?;
    }
    report_output::seal(root)?;
    report_output::verify(root)?;
    result
}
