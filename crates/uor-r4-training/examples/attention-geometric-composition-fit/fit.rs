//! Bank-only answer fitting; immutable producer dependencies belong to the caller.
use super::{data, invalid, write};
use candle_core::{backprop::GradStore, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_composition::CompositionWeights, geometric_stack::logits_cross_entropy, sha256_bytes,
    Result,
};

pub(super) const STEPS: usize = 640;
pub(super) const BATCH: usize = 8;
pub(super) const RATE: f64 = 0.003;
pub(super) const PARENT_UPDATES: usize = 1920;
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
    pub initial_generator_state: u64,
    pub next_generator_state: u64,
    pub observed_generator_state_after_draws: u64,
    pub generator_state_semantics: &'static str,
    pub prepared_batches: usize,
    pub prepared_episodes: usize,
    pub actual_positions: usize,
    pub padded_positions: usize,
    pub minimum_batch_time: Option<usize>,
    pub maximum_batch_time: usize,
    pub minimum_episode_length: Option<usize>,
    pub maximum_episode_length: usize,
    pub preparation_seconds: f64,
    pub forward_backward_update_seconds: f64,
    pub checkpoint_seconds: f64,
    pub elapsed_seconds: f64,
    pub total_left_choice_changes: usize,
    pub total_right_choice_changes: usize,
    pub total_q4_flips: usize,
    pub total_projected_coordinates: usize,
    pub clipped_updates: usize,
    pub completed: bool,
}
pub(super) fn parameter_hashes(w: &CompositionWeights) -> Result<BTreeMap<String, String>> {
    w.parameters()
        .iter()
        .map(|(name, var)| {
            let values = var.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite composition parameter"));
            }
            Ok((
                name.clone(),
                sha256_bytes(
                    &values
                        .iter()
                        .flat_map(|x| x.to_bits().to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ))
        })
        .collect()
}
fn variables(w: &CompositionWeights) -> Result<Vec<(String, Var)>> {
    let expected = BTreeMap::from([
        ("gains", vec![128]),
        ("left_logits", vec![128, 120]),
        ("right_logits", vec![128, 120]),
    ]);
    if w.parameters().len() != 3 {
        return Err(invalid("composition fit parameter inventory differs"));
    }
    let mut selected = Vec::new();
    for (name, shape) in expected {
        let v = w
            .parameters()
            .get(name)
            .ok_or_else(|| invalid("composition parameter missing"))?;
        if v.dims() != shape {
            return Err(invalid("composition fit parameter shape differs"));
        }
        selected.push((name.to_owned(), v.clone()));
    }
    Ok(selected)
}
fn gains(w: &CompositionWeights) -> Result<Vec<f32>> {
    w.parameters()
        .get("gains")
        .ok_or_else(|| invalid("composition gains missing"))?
        .to_vec1::<f32>()
        .map_err(Into::into)
}
fn hard_gains(v: &[f32]) -> Vec<i8> {
    v.iter().map(|x| (x * 4.).round() as i8).collect()
}
fn clip(selected: &[(String, Var)], gradients: &mut GradStore) -> Result<(f64, Value)> {
    let mut total = 0f64;
    let mut families = serde_json::Map::new();
    for (name, var) in selected {
        let g = gradients
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("composition answer gradient disconnected:{name}")))?;
        let values = g.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite composition answer gradient"));
        }
        let square = values.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        total += square;
        families.insert(name.clone(),json!({"l2":square.sqrt(),"nonzero":values.iter().filter(|x|**x!=0.).count(),"elements":values.len()}));
    }
    let norm = total.sqrt();
    if norm > 1. {
        for (_, var) in selected {
            let g = gradients
                .get(var.as_tensor())
                .ok_or_else(|| invalid("selected gradient disappeared"))?;
            gradients.insert(var.as_tensor(), g.affine(1. / norm, 0.)?);
        }
    }
    Ok((norm, json!(families)))
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
    mask: &[f32],
) -> Result<()> {
    if episodes.len() != BATCH
        || time == 0
        || time > 128
        || targets.len() != BATCH * time
        || mask.len() != targets.len()
    {
        return Err(invalid("composition fit batch outside fixed contract"));
    }
    for (b, e) in episodes.iter().enumerate() {
        if e.query + 1 != e.ids.len() || e.ids[e.query] != 34 || e.ids.len() > time {
            return Err(invalid(
                "composition target is not actual final answer marker",
            ));
        }
        for t in 0..time {
            if mask[b * time + t] != if t == e.query { 1. } else { 0. }
                || (t == e.query && targets[b * time + t] != e.answer)
            {
                return Err(invalid("one actual answer target per episode required"));
            }
        }
    }
    Ok(())
}
fn checkpoint(
    root: &Path,
    step: usize,
    rng: u64,
    w: &CompositionWeights,
    parent: &Value,
) -> Result<()> {
    let directory = root.join(format!("step-{step:04}"));
    report_output::claim(&directory)?;
    w.save(&directory.join("composition-source"))?;
    write(
        &directory.join("checkpoint.json"),
        &json!({
            "schema":"uor-r4.composition-fit-checkpoint/1","composition_completed_updates":step,
            "frozen_producer_updates":1280,"frozen_no_read_updates":640,
            "next_generator_absolute_step":PARENT_UPDATES+step,"next_generator_state":rng,
            "optimizer_state":"NOT_SERIALIZED; source weights and RNG recoverable, no exact AdamW continuation",
            "trainable_parameters":["composition.gains[128]","composition.left_logits[128,120]","composition.right_logits[128,120]"],
            "source_parameter_sha256":parameter_hashes(w)?,"parent":parent,
        }),
    )?;
    report_output::seal(&directory)?;
    report_output::verify(&directory)?;
    Ok(())
}

pub(super) fn run(
    root: &Path,
    w: &CompositionWeights,
    initial_rng: u64,
    mode: Mode,
    deadline: Instant,
    parent: &Value,
    forward: impl Fn(&[u32], usize, usize, &CompositionWeights) -> Result<Tensor>,
) -> Result<FitReport> {
    let started = Instant::now();
    let selected = variables(w)?;
    let initial = parameter_hashes(w)?;
    let mut optimizer = AdamW::new(
        selected.iter().map(|(_, v)| v.clone()).collect(),
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
    // Pre-fit checkpoint also preserves the initial state on a first-batch failure.
    let initial_checkpoint = Instant::now();
    checkpoint(&checkpoint_dir, 0, initial_rng, w, parent)?;
    let initial_checkpoint_seconds = initial_checkpoint.elapsed().as_secs_f64();
    let mut rng = data::Rng(initial_rng);
    let mut history = Vec::<Value>::new();
    let mut report = FitReport {
        completed_updates: 0,
        completed_backward_batches: 0,
        initial_generator_state: initial_rng,
        next_generator_state: initial_rng,
        observed_generator_state_after_draws: initial_rng,
        generator_state_semantics: if mode == Mode::Check {
            "two successive diagnostic draws at1920/1921; next training RNG stays initial and no AdamW update is performed"
        } else {
            "next RNG follows completed composition updates; AdamW moments are not serialized"
        },
        prepared_batches: 0,
        prepared_episodes: 0,
        actual_positions: 0,
        padded_positions: 0,
        minimum_batch_time: None,
        maximum_batch_time: 0,
        minimum_episode_length: None,
        maximum_episode_length: 0,
        preparation_seconds: 0.,
        forward_backward_update_seconds: 0.,
        checkpoint_seconds: initial_checkpoint_seconds,
        elapsed_seconds: 0.,
        total_left_choice_changes: 0,
        total_right_choice_changes: 0,
        total_q4_flips: 0,
        total_projected_coordinates: 0,
        clipped_updates: 0,
        completed: false,
    };
    let mut last_checkpoint = Instant::now();
    let mut last_saved = 0;
    let limit = if mode == Mode::Check { 2 } else { STEPS };
    for step in 0..limit {
        if Instant::now() >= deadline {
            break;
        }
        let prepare = Instant::now();
        let rng_before = rng.0;
        let episodes = training_batch(&mut rng, PARENT_UPDATES + step);
        let bytes = serde_json::to_vec(&episodes)?;
        let digest = sha256_bytes(&bytes);
        fs::write(target_dir.join(format!("batch-{step:04}.json")), bytes)?;
        let (ids, targets, mask, time) = data::batch(&episodes);
        validate_batch(&episodes, time, &targets, &mask)?;
        report.prepared_batches += 1;
        report.prepared_episodes += BATCH;
        let positions = episodes.iter().map(|e| e.ids.len()).sum::<usize>();
        report.actual_positions += positions;
        report.padded_positions += ids.len();
        report.minimum_batch_time =
            Some(report.minimum_batch_time.map_or(time, |old| old.min(time)));
        report.maximum_batch_time = report.maximum_batch_time.max(time);
        for e in &episodes {
            report.minimum_episode_length = Some(
                report
                    .minimum_episode_length
                    .map_or(e.ids.len(), |old| old.min(e.ids.len())),
            );
            report.maximum_episode_length = report.maximum_episode_length.max(e.ids.len());
        }
        report.observed_generator_state_after_draws = rng.0;
        report.preparation_seconds += prepare.elapsed().as_secs_f64();
        if Instant::now() >= deadline {
            rng.0 = rng_before;
            break;
        }
        let before = parameter_hashes(w)?;
        let (left_before, right_before) = w.selected_roots()?;
        let gain_before = hard_gains(&gains(w)?);
        let calculate = Instant::now();
        let result = (|| -> Result<(f32, f64, Value, usize)> {
            let logits = forward(&ids, BATCH, time, w)?;
            let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
            let value = loss.to_scalar::<f32>()?;
            if !value.is_finite() {
                return Err(invalid("nonfinite composition answer loss"));
            }
            let mut gradients = loss.backward()?;
            let (norm, families) = clip(&selected, &mut gradients)?;
            let mut projected = 0;
            if mode == Mode::Fit {
                optimizer.step(&gradients)?;
                projected = gains(w)?.iter().filter(|x| x.abs() > 1.75).count();
                w.project_gain_range()?;
                w.packed_gains()?;
                w.selected_roots()?;
            }
            Ok((value, norm, families, projected))
        })();
        report.forward_backward_update_seconds += calculate.elapsed().as_secs_f64();
        let (loss, norm, families, projected) = match result {
            Ok(v) => v,
            Err(error) => {
                write(
                    &root.join("training-failure.json"),
                    &json!({"batch":step,"absolute_step":PARENT_UPDATES+step,
                    "rng_before":rng_before,"episode_sha256":digest,"source_before":before,
                    "completed_updates":report.completed_updates,"error":error.to_string()}),
                )?;
                return Err(error);
            }
        };
        report.completed_backward_batches += 1;
        if mode == Mode::Fit {
            report.completed_updates += 1;
        }
        report.next_generator_state = if mode == Mode::Check {
            initial_rng
        } else {
            rng.0
        };
        let (left, right) = w.selected_roots()?;
        let gain_after = gains(w)?;
        let left_changes = left_before
            .iter()
            .zip(&left)
            .filter(|(a, b)| a != b)
            .count();
        let right_changes = right_before
            .iter()
            .zip(&right)
            .filter(|(a, b)| a != b)
            .count();
        let flips = gain_before
            .iter()
            .zip(hard_gains(&gain_after))
            .filter(|(a, b)| **a != *b)
            .count();
        report.total_left_choice_changes += left_changes;
        report.total_right_choice_changes += right_changes;
        report.total_q4_flips += flips;
        report.total_projected_coordinates += projected;
        report.clipped_updates += usize::from(mode == Mode::Fit && norm > 1.);
        let mut row = json!({"batch":step,"absolute_generator_step":PARENT_UPDATES+step,
            "composition_completed_updates":report.completed_updates,"rng_before":rng_before,"rng_after":rng.0,
            "episode_sha256":digest,"answer_mean_ce":loss,"answer_ce_sum":f64::from(loss)*BATCH as f64,
            "answer_denominator":BATCH,"raw_selected_gradient_l2":norm,
            "gradient_clipped":mode==Mode::Fit&&norm>1.,"projected_coordinates":projected,
            "left_choice_changes":left_changes,"right_choice_changes":right_changes,"q4_flips":flips,
            "source_parameter_sha256":parameter_hashes(w)?,"left_sha256":sha256_bytes(&left),
            "right_sha256":sha256_bytes(&right),"packed_gains_sha256":sha256_bytes(&w.packed_gains()?),
            "saturated_gains":gain_after.iter().filter(|x|x.abs()==1.75).count(),
            "actual_positions":positions,"padded_positions":ids.len(),"time":time,
            "calculation_seconds":calculate.elapsed().as_secs_f64()});
        if step == 0 || (step + 1) % 80 == 0 || mode == Mode::Check {
            row["ordinary_answer_family_gradients"] = families;
        }
        history.push(row);
        write(&root.join("history.json"), &history)?;
        if mode == Mode::Fit
            && (report.completed_updates % 80 == 0 || last_checkpoint.elapsed().as_secs() >= 900)
        {
            let at = Instant::now();
            checkpoint(&checkpoint_dir, report.completed_updates, rng.0, w, parent)?;
            report.checkpoint_seconds += at.elapsed().as_secs_f64();
            last_saved = report.completed_updates;
            last_checkpoint = Instant::now();
        }
        write(&root.join("fit-progress.json"), &report)?;
    }
    if mode == Mode::Check && parameter_hashes(w)? != initial {
        return Err(invalid("check mode modified composition bank"));
    }
    if mode == Mode::Fit && report.completed_updates > 0 && last_saved != report.completed_updates {
        let at = Instant::now();
        checkpoint(&checkpoint_dir, report.completed_updates, rng.0, w, parent)?;
        report.checkpoint_seconds += at.elapsed().as_secs_f64();
    }
    report.completed = if mode == Mode::Fit {
        report.completed_updates == STEPS
    } else {
        report.completed_backward_batches == 2
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
    fn composition_fit_sampler_mask_and_exact_u64_continuation() -> Result<()> {
        let mut a = data::Rng(9347350181718662555);
        let mut b = data::Rng(9347350181718662555);
        for step in 1920..1922 {
            let es = training_batch(&mut a, step);
            assert_eq!(es, training_batch(&mut b, step));
            assert_eq!(a.0, b.0);
            let (_, targets, mask, time) = data::batch(&es);
            validate_batch(&es, time, &targets, &mask)?;
            assert_eq!(mask.iter().sum::<f32>(), 8.);
            assert!(es.iter().all(|e| e.facts == [2, 4][step % 2]));
        }
        let json: Value = serde_json::from_str("{\"rng\":10697677911788535647}")?;
        assert_eq!(json["rng"].as_u64(), Some(10697677911788535647));
        Ok(())
    }
    #[test]
    fn composition_fit_selected_global_clip_excludes_frozen_variables() -> Result<()> {
        let w = CompositionWeights::new()?;
        let selected = variables(&w)?;
        let frozen = Var::new(100f32, &Device::Cpu)?;
        let mut objective = frozen.sqr()?;
        for (_, v) in &selected {
            objective = objective.add(&v.sum_all()?)?;
        }
        let mut gradients = objective.backward()?;
        let (norm, _) = clip(&selected, &mut gradients)?;
        assert!((norm - (30848f64).sqrt()).abs() < 1e-10);
        assert_eq!(
            gradients
                .get(frozen.as_tensor())
                .ok_or_else(|| invalid("fixture gradient"))?
                .to_scalar::<f32>()?,
            200.
        );
        let mut optimizer = AdamW::new(
            selected.iter().map(|(_, v)| v.clone()).collect(),
            ParamsAdamW {
                lr: RATE,
                beta1: 0.9,
                beta2: 0.95,
                eps: 1e-8,
                weight_decay: 0.,
            },
        )?;
        optimizer.step(&gradients)?;
        w.project_gain_range()?;
        assert_eq!(frozen.to_scalar::<f32>()?, 100.);
        w.parameters()["gains"].set(&Tensor::from_vec(vec![2f32; 128], 128, &Device::Cpu)?)?;
        w.project_gain_range()?;
        assert!(gains(&w)?.iter().all(|v| *v == 1.75));
        Ok(())
    }
}
