//! Offline, tune-selected count blend with unchanged artifact Stop probability.
use super::*;
#[path = "observer_blend_math.rs"]
mod math;
use math::{blend_bits, logsum2, residual_endpoints};
use std::path::Path;
use uor_r4_core::native_geometric::learner::realtext_support::{paired_interval, Agg};
const BETA: [f64; 21] = [
    0., 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.75, 0.8,
    0.85, 0.9, 0.95, 1.,
];

#[allow(clippy::too_many_arguments)]
fn evaluate(
    model: &TlModel,
    windows: &[ProseWindow],
    names: &[String],
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    lambdas: (f64, f64),
    betas: &[f64],
) -> Result<(Vec<Agg>, Agg, f64, Vec<serde_json::Value>), String> {
    let mut grid = vec![vec![(0.0, 0usize); names.len()]; betas.len()];
    let mut count = vec![(0.0, 0usize); names.len()];
    let mut parity = 0.0f64;
    let kind = std::env::var("UOR_OBSERVER_BLEND_CONTROL").unwrap_or_else(|_| "artifact".into());
    let alpha: f64 = std::env::var("UOR_OBSERVER_COUNT_EXPONENT")
        .unwrap_or_else(|_| "1.0".into())
        .parse()
        .map_err(|_| "invalid count exponent")?;
    if !alpha.is_finite() || !(0.0..=2.0).contains(&alpha) {
        return Err("count exponent outside range".into());
    }
    let mut records = Vec::new();
    let m = vec![0; model.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let rows = model.legal_rows(false);
    if rows.len() != model.vocab + 1 || rows.contains(&model.copy_row()) {
        return Err("unexpected prose legal action set".into());
    }
    for w in windows {
        let ex = prose_example(w);
        let mut h = model.init_state(&m, &f);
        for &token in &ex.observed {
            h = model.transition(&h, TL_EV_OBSERVE, Some(token), &m, &f);
        }
        for (i, action) in ex.actions.iter().enumerate() {
            if let TlAction::Generate(target) = action {
                let logits = model.readout(&h, &m, &f, ex.prior_event(i));
                let scale = 2f64.powi(-(model.score_shift as i32));
                let a: Vec<f64> = logits[..model.vocab]
                    .iter()
                    .map(|z| f64::from(*z) * scale)
                    .collect();
                let legal: Vec<f64> = rows.iter().map(|&r| f64::from(logits[r]) * scale).collect();
                let stop_bits = logsum2(&legal) - logsum2(&a);
                let k = PREFIX + i;
                let (prev, cur, t) = prose_position(w, k);
                if t != *target {
                    return Err("target alignment mismatch".into());
                }
                let c: Vec<f64> = (0..model.vocab)
                    .map(|v| family_p(c1, c2, uni, prev, cur, v as u32, lambdas).log2())
                    .collect();
                let canonical =
                    -prob_of(&logits, &rows, *target as usize, model.score_shift).log2();
                let zero = blend_bits(&a, &c, *target as usize, 0.0, stop_bits);
                parity = parity.max((zero - canonical).abs());
                let (a, c) = if kind == "residual" {
                    residual_endpoints(&a, &c, alpha)?
                } else {
                    (a, c)
                };
                let a = match kind.as_str() {
                    "unigram" => (0..model.vocab).map(|v| uni.p(v as u32).log2()).collect(),
                    "uniform" => vec![0.0; model.vocab],
                    "curcount" => (0..model.vocab)
                        .map(|v| {
                            family_p(c1, c2, uni, prev, cur, v as u32, (lambdas.0, 0.0)).log2()
                        })
                        .collect(),
                    "artifact" | "residual" => a,
                    _ => return Err("unknown calibration endpoint".into()),
                };
                let losses: Vec<f64> = betas
                    .iter()
                    .map(|&b| blend_bits(&a, &c, *target as usize, b, stop_bits))
                    .collect();
                for (arm, &loss) in losses.iter().enumerate() {
                    grid[arm][w.doc].0 += loss;
                    grid[arm][w.doc].1 += 1;
                }
                let cb = -c[*target as usize];
                count[w.doc].0 += cb;
                count[w.doc].1 += 1;
                records.push(serde_json::json!({"doc":w.doc,"target":target,
                    "stop_bits":stop_bits,"raw_count_bits":cb,"losses":losses}));
            }
            let emitted = match action {
                TlAction::Generate(v) => Some(*v),
                TlAction::Copy => None,
                TlAction::Stop => None,
            };
            h = model.transition(&h, action.event(), emitted, &m, &f);
        }
    }
    if parity > 1e-10 {
        return Err(format!("blend beta=0 parity failed: {parity}"));
    }
    let agg = |values: Vec<(f64, usize)>| Agg {
        rows: values
            .into_iter()
            .enumerate()
            .map(|(i, (bits, n))| (names[i].clone(), bits, n))
            .collect(),
    };
    Ok((
        grid.into_iter().map(agg).collect(),
        agg(count),
        parity,
        records,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    model: &TlModel,
    tune: &[ProseWindow],
    tune_names: &[String],
    dev: &[ProseWindow],
    dev_names: &[String],
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    lambdas: (f64, f64),
    metadata: serde_json::Value,
) -> Result<ExitCode, String> {
    let started = Instant::now();
    let temperature = std::env::var("UOR_OBSERVER_TEMPERATURE_SWEEP").as_deref() == Ok("1");
    if temperature && std::env::var("UOR_OBSERVER_BLEND_CONTROL").as_deref() != Ok("uniform") {
        return Err("temperature sweep requires the uniform endpoint".into());
    }
    let beta: Vec<f64> = if temperature {
        (0..=40).map(|i| i as f64 / 20.0).collect()
    } else {
        BETA.to_vec()
    };

    let (tune_grid, tune_raw, tune_parity, _) =
        evaluate(model, tune, tune_names, c1, c2, uni, lambdas, &beta)?;
    let selected = (0..beta.len())
        .min_by(|&a, &b| tune_grid[a].micro().total_cmp(&tune_grid[b].micro()))
        .ok_or("empty beta grid")?;
    let selected_beta = beta[selected];
    println!(
        "BLEND: tune fixed beta={selected_beta} on {} positions before development",
        tune_grid[0].total().1
    );
    let chosen = [0.0, selected_beta, 1.0];
    let (dev_grid, dev_raw, dev_parity, records) =
        evaluate(model, dev, dev_names, c1, c2, uni, lambdas, &chosen)?;
    let vs_endpoint = paired_interval(&dev_grid[1], &dev_grid[0], 20260923);
    let vs_count = paired_interval(&dev_grid[1], &dev_grid[2], 20260923);
    let complementarity =
        selected_beta > 0.0 && selected_beta < 1.0 && vs_endpoint.2 < 0.0 && vs_count.2 < 0.0;
    let summarize = |a: &Agg| serde_json::json!({"bits_per_target":a.micro(),"positions":a.total().1,"documents":a.rows});
    let report = serde_json::json!({"schema":"uor-r4.observer-count-blend/3","metadata":metadata,
        "readout_kind":std::env::var("UOR_OBSERVER_BLEND_CONTROL").unwrap_or_else(|_| "artifact".into()),
        "count_exponent":std::env::var("UOR_OBSERVER_COUNT_EXPONENT").unwrap_or_else(|_| "1.0".into()),
        "residual_strength_if_residual_mode":1.0-selected_beta,
        "beta_grid":beta,"selected_beta":selected_beta,"selection_split":"tune only",
        "tune_windows":tune.len(),"tune_documents":tune_names.len(),
        "tune_grid_bits":tune_grid.iter().map(Agg::micro).collect::<Vec<_>>(),
        "tune_positions":tune_grid[0].total().1,"tune_raw_count_bits":tune_raw.micro(),
        "tune_parity_max_absolute_bits":tune_parity,"development_parity_max_absolute_bits":dev_parity,
        "development":{"first_endpoint":summarize(&dev_grid[0]),"selected_blend":summarize(&dev_grid[1]),
            "count_same_stop":summarize(&dev_grid[2]),"raw_count":summarize(&dev_raw)},
        "blend_minus_first_endpoint_doc_bootstrap":vs_endpoint,
        "blend_minus_count_same_stop_doc_bootstrap":vs_count,
        "complementarity_criterion_passed":complementarity,
        "normalization":"Artifact Stop probability held exactly constant. Generate-conditional distributions blended log-linearly; Copy illegal. Base-two logits and log probabilities throughout.",
        "scope":"One predeclared scalar grid, one frozen artifact, existing open development corpus. No model-weight update, servable artifact, free-generation quality or geometric advantage claim.",
        "elapsed_seconds":started.elapsed().as_secs_f64()});
    write_checked(
        root,
        "positions.json",
        &serde_json::to_vec(&records).map_err(|e| e.to_string())?,
    )?;
    write_checked(
        root,
        "receipt.json",
        &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )?;
    seal(root).map_err(|e| e.to_string())?;
    let errors = verify(root).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("manifest verification: {errors:?}"));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    Ok(ExitCode::SUCCESS)
}
