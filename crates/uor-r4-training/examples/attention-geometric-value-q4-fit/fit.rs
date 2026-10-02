//! Strict-q4 producer-only answer fitting; all surrounding operators stay frozen.
use super::{data, invalid, write};
use candle_core::{backprop::GradStore, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_stack::logits_cross_entropy,
    geometric_value_producer::{ValueProducerOutput, ValueProducerWeights},
    sha256_bytes, Result,
};

pub(super) const STEPS: usize = 640;
pub(super) const BATCH: usize = 8;
pub(super) const RATE: f64 = 0.003;
pub(super) const PARENT_UPDATES: usize = 2560;
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
    pub total_q4_flips: usize,
    pub total_projected_coordinates: usize,
    pub clipped_updates: usize,
    pub completed: bool,
}
pub(super) fn parameter_hashes(w: &ValueProducerWeights) -> Result<BTreeMap<String, String>> {
    w.parameters()
        .iter()
        .map(|(name, var)| {
            let values = var.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite q4_value parameter"));
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
fn variables(w: &ValueProducerWeights) -> Result<Vec<(String, Var)>> {
    if !w.is_q4()
        || w.config().vocab_size != 40
        || w.config().heads != 2
        || w.config().latent_lanes_per_head != 4
    {
        return Err(invalid("q4 fit requires V40/H2/L4 strict source"));
    }
    let expected = uor_r4_integer::geometric_value_q4::ValueQ4Config {
        vocab_size: 40,
        heads: 2,
        latent_lanes_per_head: 4,
    }
    .coefficient_shapes()
    .map_err(|e| invalid(e.to_string()))?;
    if w.parameters().len() != expected.len() {
        return Err(invalid("q4 fit inventory differs"));
    }
    let mut selected = Vec::new();
    let mut count = 0;
    for (name, shape) in expected {
        let v = w
            .parameters()
            .get(&name)
            .ok_or_else(|| invalid("q4 parameter missing"))?;
        if v.dims() != shape {
            return Err(invalid("q4 parameter shape differs"));
        }
        count += v.elem_count();
        selected.push((name, v.clone()));
    }
    if count != 128896 {
        return Err(invalid("q4 shadow count differs"));
    }
    Ok(selected)
}
fn shadows(w: &ValueProducerWeights) -> Result<Vec<f32>> {
    let mut result = Vec::new();
    for (_, v) in variables(w)? {
        result.extend(v.flatten_all()?.to_vec1::<f32>()?);
    }
    Ok(result)
}
fn hard_codes(w: &ValueProducerWeights) -> Result<Vec<i8>> {
    uor_r4_integer::geometric_value_q4::unpack_coefficients(128896, &w.packed_coefficients()?)
        .map_err(|e| invalid(e.to_string()))
}
fn clip(selected: &[(String, Var)], gradients: &mut GradStore) -> Result<(f64, Value)> {
    let mut total = 0f64;
    let mut families = serde_json::Map::new();
    for (name, var) in selected {
        let g = gradients
            .get(var.as_tensor())
            .ok_or_else(|| invalid(format!("q4_value answer gradient disconnected:{name}")))?;
        let values = g.flatten_all()?.to_vec1::<f32>()?;
        if values.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite q4_value answer gradient"));
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
        return Err(invalid("q4_value fit batch outside fixed contract"));
    }
    for (b, e) in episodes.iter().enumerate() {
        if e.query + 1 != e.ids.len() || e.ids[e.query] != 34 || e.ids.len() > time {
            return Err(invalid("q4_value target is not actual final answer marker"));
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
    w: &ValueProducerWeights,
    parent: &Value,
) -> Result<()> {
    let directory = root.join(format!("step-{step:04}"));
    report_output::claim(&directory)?;
    w.save_source(&directory.join("q4-value-source"))?;
    write(
        &directory.join("checkpoint.json"),
        &json!({
            "schema":"uor-r4.q4_value-fit-checkpoint/1","q4_value_completed_updates":step,
            "prior_unrestricted_producer_updates":1280,"frozen_no_read_updates":640,"frozen_composition_updates":640,
            "next_generator_absolute_step":PARENT_UPDATES+step,"next_generator_state":rng,
            "optimizer_state":"NOT_SERIALIZED; source weights and RNG recoverable, no exact AdamW continuation",
            "trainable_parameters":"all ten value producer families; 128896 shadows",
            "source_parameter_sha256":parameter_hashes(w)?,"parent":parent,
        }),
    )?;
    report_output::seal(&directory)?;
    report_output::verify(&directory)?;
    Ok(())
}

pub(super) fn run(
    root: &Path,
    w: &ValueProducerWeights,
    initial_rng: u64,
    mode: Mode,
    deadline: Instant,
    parent: &Value,
    forward: impl Fn(
        &[u32],
        usize,
        usize,
        &ValueProducerWeights,
    ) -> Result<(Tensor, ValueProducerOutput)>,
) -> Result<FitReport> {
    let started = Instant::now();
    let selected = variables(w)?;
    let initial = parameter_hashes(w)?;
    let mut optimizer = AdamW::new(
        selected.iter().map(|(_, v)| v.clone()).collect(),
        ParamsAdamW {
            lr: RATE,
            beta1: 0.9,
            beta2: 0.999,
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
            "two successive diagnostic draws at2560/2561; next training RNG stays initial and no AdamW update is performed"
        } else {
            "next RNG follows completed q4_value updates; AdamW moments are not serialized"
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
        let codes_before = hard_codes(w)?;
        let calculate = Instant::now();
        let result = (|| -> Result<(f32, f64, Value, usize, usize)> {
            let (logits, output) = forward(&ids, BATCH, time, w)?;
            let mut zeros = 0;
            for (b, e) in episodes.iter().enumerate() {
                for h in 0..2 {
                    for t in 0..e.ids.len() {
                        for l in 0..4 {
                            let pair =
                                output
                                    .trace
                                    .packets
                                    .get(((b * 2 + h) * time + t) * 4 + l)
                                    .ok_or_else(|| invalid("actual value trace shape differs"))?;
                            zeros += pair.iter().filter(|p| p.status == uor_r4_training::geometric_value_native::ValuePacketStatus::PresentZero).count();
                        }
                    }
                }
            }
            let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
            let value = loss.to_scalar::<f32>()?;
            if !value.is_finite() {
                return Err(invalid("nonfinite q4_value answer loss"));
            }
            let mut gradients = loss.backward()?;
            let (norm, families) = clip(&selected, &mut gradients)?;
            let mut projected = 0;
            if mode == Mode::Fit {
                optimizer.step(&gradients)?;
                projected = shadows(w)?.iter().filter(|x| x.abs() > 1.75).count();
                w.project_shadow_range()?;
                w.packed_coefficients()?;
            }
            Ok((value, norm, families, projected, zeros))
        })();
        report.forward_backward_update_seconds += calculate.elapsed().as_secs_f64();
        let (loss, norm, families, projected, zero_atoms) = match result {
            Ok(v) => v,
            Err(error) => {
                // A failed native decode may still have a completely valid source.
                // Preserve it separately from the last completed checkpoint; an
                // optimizer failure may have partially changed parameters.
                let preserved = w.save_source(&root.join("failed-attempt-q4-value-source"));
                write(
                    &root.join("training-failure.json"),
                    &json!({"batch":step,"absolute_step":PARENT_UPDATES+step,
                    "rng_before":rng_before,"episode_sha256":digest,"source_before":before,
                    "completed_updates":report.completed_updates,"error":error.to_string(),
                    "attempt_source_preserved":preserved.is_ok(),"attempt_source_preservation_error":preserved.err().map(|e|e.to_string()),
                    "attempt_source_semantics":"current source after failed attempt; not a completed update or optimizer-resumable checkpoint",
                    "last_completed_rng":report.next_generator_state,"observed_draw_rng":rng.0}),
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
        let codes_after = hard_codes(w)?;
        let after_shadows = shadows(w)?;
        let flips = codes_before
            .iter()
            .zip(&codes_after)
            .filter(|(a, b)| a != b)
            .count();
        report.total_q4_flips += flips;
        report.total_projected_coordinates += projected;
        report.clipped_updates += usize::from(mode == Mode::Fit && norm > 1.);
        let mut row = json!({"batch":step,"absolute_generator_step":PARENT_UPDATES+step,
            "q4_value_completed_updates":report.completed_updates,"rng_before":rng_before,"rng_after":rng.0,
            "episode_sha256":digest,"answer_mean_ce":loss,"answer_ce_sum":f64::from(loss)*BATCH as f64,
            "answer_denominator":BATCH,"raw_selected_gradient_l2":norm,
            "gradient_clipped":mode==Mode::Fit&&norm>1.,"projected_coordinates":projected,
            "q4_flips":flips,"present_zero_atoms_actual_tokens_before_update":zero_atoms,
            "source_parameter_sha256":parameter_hashes(w)?,"packed_coefficients_sha256":sha256_bytes(&w.packed_coefficients()?),
            "saturated_shadows":after_shadows.iter().filter(|x|x.abs()==1.75).count(),
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
        return Err(invalid("check mode modified q4 value producer"));
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
    fn q4_value_fit_sampler_mask_and_exact_u64_continuation() -> Result<()> {
        for initial in [461503785751815875u64, 1220542080958524937u64] {
            let mut a = data::Rng(initial);
            let mut b = data::Rng(initial);
            for step in 2560..2562 {
                let es = training_batch(&mut a, step);
                assert_eq!(es, training_batch(&mut b, step));
                assert_eq!(a.0, b.0);
                let (_, targets, mask, time) = data::batch(&es);
                validate_batch(&es, time, &targets, &mask)?;
                assert_eq!(mask.iter().sum::<f32>(), 8.);
                assert!(es.iter().all(|e| e.facts == [2, 4][step % 2]));
            }
            let json: Value =
                serde_json::from_slice(&serde_json::to_vec(&json!({"rng":initial}))?)?;
            assert_eq!(json["rng"].as_u64(), Some(initial));
        }
        Ok(())
    }
    #[test]
    fn q4_value_fit_selected_global_clip_excludes_frozen_variables() -> Result<()> {
        let w = ValueProducerWeights::new(40, 2, 4, 1)?.into_q4()?;
        let selected = variables(&w)?;
        let frozen = Var::new(100f32, &Device::Cpu)?;
        let mut objective = frozen.sqr()?;
        for (_, v) in &selected {
            objective = objective.add(&v.sum_all()?)?;
        }
        let mut gradients = objective.backward()?;
        let (norm, _) = clip(&selected, &mut gradients)?;
        assert!((norm - (128896f64).sqrt()).abs() < 1e-10);
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
                beta2: 0.999,
                eps: 1e-8,
                weight_decay: 0.,
            },
        )?;
        optimizer.step(&gradients)?;
        w.project_shadow_range()?;
        assert_eq!(frozen.to_scalar::<f32>()?, 100.);
        let v = &w.parameters()["span_valid_category"];
        v.set(&Tensor::from_vec(
            vec![2f32; v.elem_count()],
            v.dims(),
            &Device::Cpu,
        )?)?;
        w.project_shadow_range()?;
        assert!(v
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| *x == 1.75));
        assert_eq!(
            selected.iter().map(|(_, v)| v.elem_count()).sum::<usize>(),
            128896
        );
        assert_eq!(selected.len(), 10);
        Ok(())
    }
}
