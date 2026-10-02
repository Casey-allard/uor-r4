//! Fixed answer-only fit. All prediction dependencies are captured immutably
//! by the caller; only the declared NoRead coefficient Var reaches AdamW.
use super::{data, invalid, write};
use candle_core::{backprop::GradStore, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_no_read::NoReadWeights, geometric_stack::logits_cross_entropy, sha256_bytes, Result,
};

pub(super) const STEPS: usize = 640;
pub(super) const BATCH: usize = 8;
pub(super) const RATE: f64 = 0.003;
pub(super) const PARENT_UPDATES: usize = 1280;
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mode {
    Check,
    Fit,
}

#[derive(Serialize)]
pub(super) struct FitReport {
    pub completed_updates: usize,
    pub completed_backward_batches: usize,
    pub next_generator_state: u64,
    pub initial_generator_state: u64,
    pub prepared_batches: usize,
    pub prepared_episodes: usize,
    pub actual_positions: usize,
    pub padded_positions: usize,
    pub minimum_batch_time: Option<usize>,
    pub maximum_batch_time: usize,
    pub minimum_episode_length: Option<usize>,
    pub maximum_episode_length: usize,
    pub observed_generator_state_after_draws: u64,
    pub generator_state_semantics: &'static str,
    pub preparation_seconds: f64,
    pub forward_backward_update_seconds: f64,
    pub checkpoint_seconds: f64,
    pub elapsed_seconds: f64,
    pub total_q4_flips: usize,
    pub total_projected_coordinates: usize,
    pub clipped_updates: usize,
    pub completed: bool,
}

pub(super) fn shadows(weights: &NoReadWeights) -> Result<Vec<f32>> {
    weights
        .parameters()
        .get("coefficients")
        .ok_or_else(|| invalid("NoRead coefficients missing"))?
        .flatten_all()?
        .to_vec1::<f32>()
        .map_err(Into::into)
}
fn bits_hash(v: &[f32]) -> String {
    sha256_bytes(
        &v.iter()
            .flat_map(|x| x.to_bits().to_le_bytes())
            .collect::<Vec<_>>(),
    )
}
fn hard_codes(v: &[f32]) -> Vec<i8> {
    v.iter().map(|x| (x * 4.).round() as i8).collect()
}
fn variable(weights: &NoReadWeights) -> Result<Var> {
    if weights.parameters().len() != 1 {
        return Err(invalid("unexpected trainable inventory"));
    }
    weights
        .parameters()
        .get("coefficients")
        .cloned()
        .ok_or_else(|| invalid("NoRead coefficients missing"))
}

pub(super) fn family_l2(weights: &NoReadWeights, values: &[f32]) -> Result<Value> {
    let cfg = weights.config();
    if values.len() != cfg.heads * cfg.coefficients_per_head()
        || values.iter().any(|x| !x.is_finite())
    {
        return Err(invalid("nonfinite or mis-shaped NoRead gradient"));
    }
    let widths = [
        ("bias", 1),
        ("token", cfg.vocabulary),
        ("retained_root", cfg.lanes() * 4),
        ("category", cfg.lanes() * 33),
        ("held_root", cfg.lanes() * 4),
        ("held_valid", cfg.lanes()),
    ];
    let mut heads = Vec::new();
    for h in 0..cfg.heads {
        let mut at = h * cfg.coefficients_per_head();
        let mut families = serde_json::Map::new();
        for (name, n) in widths {
            let l2 = values[at..at + n]
                .iter()
                .map(|x| f64::from(*x).powi(2))
                .sum::<f64>()
                .sqrt();
            families.insert(name.into(), json!(l2));
            at += n;
        }
        heads.push(families);
    }
    Ok(json!(heads))
}
fn clip(variable: &Var, gradients: &mut GradStore) -> Result<(f64, Vec<f32>)> {
    let gradient = gradients
        .get(variable.as_tensor())
        .ok_or_else(|| invalid("NoRead answer gradient disconnected"))?;
    let values = gradient.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfinite NoRead answer gradient"));
    }
    let norm = values
        .iter()
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        .sqrt();
    if norm > 1. {
        gradients.insert(variable.as_tensor(), gradient.affine(1. / norm, 0.)?);
    }
    Ok((norm, values))
}
pub(super) fn training_batch(rng: &mut data::Rng, absolute_step: usize) -> Vec<data::Episode> {
    let n = [2, 4][absolute_step % 2];
    (0..BATCH)
        .map(|_| {
            let facts = data::draw(rng, n);
            let query = rng.next(n);
            let noise = (0..n).map(|_| data::noise(rng, 4)).collect::<Vec<_>>();
            let query_noise = data::noise(rng, 4);
            data::episode(
                &facts,
                &noise,
                &facts[query].0,
                &query_noise,
                absolute_step,
                "joint_train",
            )
        })
        .collect()
}
fn validate_batch(
    episodes: &[data::Episode],
    time: usize,
    targets: &[u32],
    weight: &[f32],
) -> Result<()> {
    if episodes.len() != BATCH
        || time == 0
        || time > 128
        || targets.len() != BATCH * time
        || weight.len() != targets.len()
    {
        return Err(invalid(
            "NoRead training batch shape outside fixed contract",
        ));
    }
    for (b, e) in episodes.iter().enumerate() {
        if e.query + 1 != e.ids.len() || e.ids[e.query] != 34 || e.ids.len() > time {
            return Err(invalid(
                "NoRead training target must be the actual final answer marker",
            ));
        }
        for t in 0..time {
            if weight[b * time + t] != if t == e.query { 1. } else { 0. }
                || (t == e.query && targets[b * time + t] != e.answer)
            {
                return Err(invalid(
                    "NoRead target/mask differs from one answer per episode",
                ));
            }
        }
    }
    Ok(())
}
fn checkpoint(
    root: &Path,
    step: usize,
    rng: u64,
    weights: &NoReadWeights,
    parent: &Value,
) -> Result<()> {
    let directory = root.join(format!("step-{step:04}"));
    report_output::claim(&directory)?;
    weights.save(&directory.join("no-read-source"))?;
    write(
        &directory.join("checkpoint.json"),
        &json!({"schema":"uor-r4.no-read-fit-checkpoint/1",
        "no_read_completed_updates":step,"frozen_producer_updates":PARENT_UPDATES,
        "next_generator_absolute_step":PARENT_UPDATES+step,"next_generator_state":rng,
        "optimizer_state":"NOT_SERIALIZED; weights recoverable, no exact AdamW continuation",
        "trainable_parameters":["no_read.coefficients"],"parent":parent}),
    )?;
    report_output::seal(&directory)?;
    report_output::verify(&directory)?;
    Ok(())
}

/// Fit deadline excludes the caller's evaluation reserve. Zero gradients and
/// unchanged q4 choices are observations, never rejection thresholds.
pub(super) fn run(
    root: &Path,
    weights: &NoReadWeights,
    initial_rng: u64,
    mode: Mode,
    deadline: Instant,
    parent: &Value,
    forward: impl Fn(&[u32], usize, usize, &NoReadWeights) -> Result<(Tensor, Tensor)>,
) -> Result<FitReport> {
    let started = Instant::now();
    let variable = variable(weights)?;
    let initial = shadows(weights)?;
    let mut optimizer = AdamW::new(
        vec![variable.clone()],
        ParamsAdamW {
            lr: RATE,
            beta1: 0.9,
            beta2: 0.95,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let target_dir = root.join("training-inputs");
    fs::create_dir(&target_dir)?;
    let checkpoint_dir = root.join("checkpoints");
    fs::create_dir(&checkpoint_dir)?;
    let mut rng = data::Rng(initial_rng);
    let mut history = Vec::<Value>::new();
    let mut report = FitReport {
        completed_updates: 0,
        completed_backward_batches: 0,
        next_generator_state: initial_rng,
        initial_generator_state: initial_rng,
        prepared_batches: 0,
        prepared_episodes: 0,
        actual_positions: 0,
        padded_positions: 0,
        minimum_batch_time: None,
        maximum_batch_time: 0,
        minimum_episode_length: None,
        maximum_episode_length: 0,
        observed_generator_state_after_draws: initial_rng,
        generator_state_semantics: if mode == Mode::Check {
            "check consumes one diagnostic draw; next_generator_state remains initial, not a training resume"
        } else {
            "next_generator_state follows completed NoRead updates; AdamW moments absent"
        },
        preparation_seconds: 0.,
        forward_backward_update_seconds: 0.,
        checkpoint_seconds: 0.,
        elapsed_seconds: 0.,
        total_q4_flips: 0,
        total_projected_coordinates: 0,
        clipped_updates: 0,
        completed: false,
    };
    let mut last_checkpoint = Instant::now();
    let mut last_saved = None;
    let limit = if mode == Mode::Check { 1 } else { STEPS };
    for step in 0..limit {
        if Instant::now() >= deadline {
            break;
        }
        let prepared_at = Instant::now();
        let rng_before = rng.0;
        let episodes = training_batch(&mut rng, PARENT_UPDATES + step);
        let episode_bytes = serde_json::to_vec(&episodes)?;
        let episode_hash = sha256_bytes(&episode_bytes);
        fs::write(
            target_dir.join(format!("batch-{step:04}.json")),
            &episode_bytes,
        )?;
        let (ids, targets, mask, time) = data::batch(&episodes);
        validate_batch(&episodes, time, &targets, &mask)?;
        report.prepared_batches += 1;
        report.prepared_episodes += episodes.len();
        report.actual_positions += episodes.iter().map(|e| e.ids.len()).sum::<usize>();
        report.padded_positions += ids.len();
        report.minimum_batch_time =
            Some(report.minimum_batch_time.map_or(time, |old| old.min(time)));
        report.maximum_batch_time = report.maximum_batch_time.max(time);
        for episode in &episodes {
            report.minimum_episode_length = Some(
                report
                    .minimum_episode_length
                    .map_or(episode.ids.len(), |old| old.min(episode.ids.len())),
            );
            report.maximum_episode_length = report.maximum_episode_length.max(episode.ids.len());
        }
        report.observed_generator_state_after_draws = rng.0;
        report.preparation_seconds += prepared_at.elapsed().as_secs_f64();
        // If budget ended during preparation, do not consume the untrained draw.
        if Instant::now() >= deadline {
            rng.0 = rng_before;
            break;
        }
        let calculation = Instant::now();
        let before = shadows(weights)?;
        let before_q = hard_codes(&before);
        let result = (|| -> Result<(f32, f64, Vec<f32>, Vec<Vec<f32>>, usize, usize)> {
            let (logits, null) = forward(&ids, BATCH, time, weights)?;
            let answer = logits_cross_entropy(&logits, &targets, Some(&mask))?;
            let loss = answer.to_scalar::<f32>()?;
            if !loss.is_finite() {
                return Err(invalid("nonfinite NoRead answer loss"));
            }
            let mut gradients = answer.backward()?;
            let (norm, raw_gradient) = clip(&variable, &mut gradients)?;
            let null = null.to_vec3::<f32>()?;
            let query_null = episodes
                .iter()
                .enumerate()
                .map(|(b, e)| {
                    (0..weights.config().heads)
                        .map(|h| null[b][h][e.query])
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            if query_null.iter().flatten().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite NoRead query scores"));
            }
            let mut projected = 0;
            let mut flips = 0;
            if mode == Mode::Fit {
                optimizer.step(&gradients)?;
                let unprojected = shadows(weights)?;
                projected = unprojected.iter().filter(|x| x.abs() > 1.75).count();
                weights.project_shadow_range()?;
                weights.packed_coefficients()?;
                flips = before_q
                    .iter()
                    .zip(hard_codes(&shadows(weights)?))
                    .filter(|(a, b)| **a != *b)
                    .count();
            }
            Ok((loss, norm, raw_gradient, query_null, projected, flips))
        })();
        report.forward_backward_update_seconds += calculation.elapsed().as_secs_f64();
        let (loss, norm, gradient, query_null, projected, flips) = match result {
            Ok(value) => value,
            Err(error) => {
                // The last sealed checkpoint and failing batch remain recoverable.
                write(
                    &root.join("training-failure.json"),
                    &json!({"step":step,"absolute_step":PARENT_UPDATES+step,
                    "rng_before":rng_before,"episode_sha256":episode_hash,"error":error.to_string(),
                    "completed_updates":report.completed_updates,"shadow_before_sha256":bits_hash(&before)}),
                )?;
                return Err(error);
            }
        };
        report.completed_backward_batches += 1;
        if mode == Mode::Fit {
            report.completed_updates += 1;
        }
        report.next_generator_state = rng.0;
        report.total_q4_flips += flips;
        report.total_projected_coordinates += projected;
        report.clipped_updates += usize::from(mode == Mode::Fit && norm > 1.);
        let after = shadows(weights)?;
        let mut observation = json!({"batch":step,"absolute_generator_step":PARENT_UPDATES+step,
            "no_read_completed_updates":report.completed_updates,"rng_before":rng_before,"rng_after":rng.0,
            "episode_sha256":episode_hash,"answer_mean_ce":loss,"answer_denominator":BATCH,
            "answer_ce_sum":f64::from(loss)*BATCH as f64,"raw_selected_gradient_l2":norm,
            "gradient_clipped":mode==Mode::Fit&&norm>1.,"projected_coordinates":projected,"q4_flips":flips,
            "shadow_sha256":bits_hash(&after),"packed_q4_sha256":sha256_bytes(&weights.packed_coefficients()?),
            "saturated_coordinates":after.iter().filter(|x|x.abs()==1.75).count(),
            "actual_positions":episodes.iter().map(|e|e.ids.len()).sum::<usize>(),"padded_positions":ids.len(),"time":time});
        if step == 0 || (step + 1) % 80 == 0 || mode == Mode::Check {
            observation["per_head_family_answer_gradient_l2"] = family_l2(weights, &gradient)?;
            observation["pre_update_query_no_read_nats"] = json!(query_null);
        }
        history.push(observation);
        write(&root.join("history.json"), &history)?;
        if mode == Mode::Fit
            && (report.completed_updates % 80 == 0 || last_checkpoint.elapsed().as_secs() >= 900)
        {
            let at = Instant::now();
            checkpoint(
                &checkpoint_dir,
                report.completed_updates,
                rng.0,
                weights,
                parent,
            )?;
            report.checkpoint_seconds += at.elapsed().as_secs_f64();
            last_saved = Some(report.completed_updates);
            last_checkpoint = Instant::now();
        }
    }
    if mode == Mode::Check && bits_hash(&shadows(weights)?) != bits_hash(&initial) {
        return Err(invalid("check mode changed NoRead parameters"));
    }
    if mode == Mode::Fit
        && report.completed_updates > 0
        && last_saved != Some(report.completed_updates)
    {
        let at = Instant::now();
        checkpoint(
            &checkpoint_dir,
            report.completed_updates,
            rng.0,
            weights,
            parent,
        )?;
        report.checkpoint_seconds += at.elapsed().as_secs_f64();
    }
    report.completed = if mode == Mode::Fit {
        report.completed_updates == STEPS
    } else {
        report.completed_backward_batches == 1
    };
    report.next_generator_state = if mode == Mode::Check {
        initial_rng
    } else {
        rng.0
    };
    report.elapsed_seconds = started.elapsed().as_secs_f64();
    write(&root.join("fit-progress.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    #[test]
    fn no_read_fit_answer_mask_and_continued_rng_are_exact() -> Result<()> {
        let mut a = data::Rng(12355479550239716776);
        let mut b = data::Rng(12355479550239716776);
        for step in 1280..1282 {
            let es = training_batch(&mut a, step);
            assert_eq!(es, training_batch(&mut b, step));
            assert_eq!(a.0, b.0);
            let (_, target, mask, time) = data::batch(&es);
            validate_batch(&es, time, &target, &mask)?;
            assert_eq!(mask.iter().sum::<f32>(), BATCH as f32);
            assert!(es.iter().all(|e| e.facts == [2, 4][step % 2]));
        }
        let v: Value = serde_json::from_str("{\"next_generator_state\":12355479550239716776}")?;
        assert_eq!(
            v["next_generator_state"].as_u64(),
            Some(12355479550239716776)
        );
        Ok(())
    }
    #[test]
    fn no_read_fit_selected_clip_projection_and_independent_copy() -> Result<()> {
        let weights = NoReadWeights::new(40, 2, 4)?;
        let selected = variable(&weights)?;
        let unrelated = Var::new(100f32, &Device::Cpu)?;
        let objective = (selected.sum_all()? + unrelated.sqr()?)?;
        let mut gradients = objective.backward()?;
        let (norm, _) = clip(&selected, &mut gradients)?;
        assert!((norm - (754f64).sqrt()).abs() < 1e-10);
        assert_eq!(
            gradients
                .get(unrelated.as_tensor())
                .ok_or_else(|| invalid("test gradient"))?
                .to_scalar::<f32>()?,
            200.
        );
        let mut optimizer = AdamW::new(
            vec![selected.clone()],
            ParamsAdamW {
                lr: RATE,
                beta1: 0.9,
                beta2: 0.95,
                eps: 1e-8,
                weight_decay: 0.,
            },
        )?;
        optimizer.step(&gradients)?;
        weights.project_shadow_range()?;
        assert_eq!(unrelated.to_scalar::<f32>()?, 100.);
        selected.set(&Tensor::from_vec(vec![2f32; 754], (2, 377), &Device::Cpu)?)?;
        weights.project_shadow_range()?;
        assert!(shadows(&weights)?.iter().all(|x| *x == 1.75));
        assert_eq!(weights.packed_coefficients()?, vec![0x77; 377]);
        Ok(())
    }
}
