//! One fixed answer-only continuation; only event and absolute-age shadows update.
use super::{data, invalid, write};
use candle_core::{backprop::GradStore, Tensor, Var};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::{
    geometric_age_source::AgeSource, geometric_event::EventWeights,
    geometric_event_credit::EventAgeCreditOutput, geometric_read_native::ReadSourceBinding,
    geometric_stack::logits_cross_entropy, sha256_bytes, Result,
};

pub(super) const STEPS: usize = 640;
pub(super) const BATCH: usize = 8;
pub(super) const PARENT_UPDATES: usize = 3200;
#[derive(Serialize)]
pub(super) struct FitReport {
    pub completed_updates: usize,
    pub completed_backward_batches: usize,
    pub initial_generator_state: u64,
    pub next_generator_state: u64,
    pub observed_generator_state_after_draws: u64,
    pub prepared_batches: usize,
    pub actual_positions: usize,
    pub padded_positions: usize,
    pub minimum_episode_length: Option<usize>,
    pub maximum_episode_length: usize,
    pub maximum_batch_time: usize,
    pub preparation_seconds: f64,
    pub forward_backward_update_seconds: f64,
    pub checkpoint_seconds: f64,
    pub elapsed_seconds: f64,
    pub event_q4_flips: usize,
    pub age_q4_flips: usize,
    pub event_projected_coordinates: usize,
    pub age_projected_coordinates: usize,
    pub clipped_updates: usize,
    pub failure: Option<String>,
    pub completed: bool,
}
pub(super) fn variables(w: &EventWeights, age: &Var) -> Result<Vec<(String, Var)>> {
    if !w.is_q4()
        || w.config().vocab_size != 40
        || w.config().lanes != 2
        || age.dims() != [2, 128]
        || age.dtype() != candle_core::DType::F32
        || !age.device().is_cpu()
    {
        return Err(invalid(
            "event/age fit requires exact V40/L2 and H2/C128 inventory",
        ));
    }
    let shapes = uor_r4_integer::geometric_event_q4::EventQ4Config {
        vocab_size: 40,
        lanes: 2,
    }
    .coefficient_shapes()
    .map_err(|e| invalid(e.to_string()))?;
    if w.parameters().len() != shapes.len() {
        return Err(invalid("event family inventory differs"));
    }
    let mut selected = Vec::new();
    for (name, shape) in shapes {
        let v = w
            .parameters()
            .get(&name)
            .ok_or_else(|| invalid("event family missing"))?;
        if v.dims() != shape {
            return Err(invalid("event family shape differs"));
        }
        selected.push((format!("event.{name}"), v.clone()));
    }
    selected.push(("age".into(), age.clone()));
    if selected.iter().map(|(_, v)| v.elem_count()).sum::<usize>() != 11968 {
        return Err(invalid("expected exactly11968 selected shadows"));
    }
    Ok(selected)
}
fn values(selected: &[(String, Var)]) -> Result<Vec<Vec<f32>>> {
    selected
        .iter()
        .map(|(_, v)| Ok(v.flatten_all()?.to_vec1::<f32>()?))
        .collect()
}
pub(super) fn parameter_hashes(w: &EventWeights, age: &Var) -> Result<BTreeMap<String, String>> {
    variables(w, age)?
        .iter()
        .map(|(n, v)| {
            let x = v.flatten_all()?.to_vec1::<f32>()?;
            if x.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite selected shadow"));
            }
            Ok((
                n.clone(),
                sha256_bytes(
                    &x.iter()
                        .flat_map(|x| x.to_bits().to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ))
        })
        .collect()
}
fn prior(head: usize, lag: usize) -> f32 {
    -(lag as f32) / if head == 0 { 16. } else { 256. }
}
fn project(selected: &[(String, Var)]) -> Result<(usize, usize)> {
    let mut pending = Vec::new();
    let (mut event, mut age) = (0, 0);
    for (n, v) in selected {
        let mut x = v.flatten_all()?.to_vec1::<f32>()?;
        if x.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite optimizer shadow"));
        }
        for (i, y) in x.iter_mut().enumerate() {
            let center = if n == "age" {
                prior(i / 128, i % 128)
            } else {
                0.
            };
            let limit = if n == "age" { 0.875 } else { 1.75 };
            let z = y.clamp(center - limit, center + limit);
            if z != *y {
                if n == "age" {
                    age += 1
                } else {
                    event += 1
                };
            }
            *y = z;
        }
        pending.push(Tensor::from_vec(x, v.shape(), v.device())?);
    }
    for ((_, v), x) in selected.iter().zip(pending) {
        v.set(&x)?;
    }
    Ok((event, age))
}
fn codes(w: &EventWeights, age: &Var, binding: &ReadSourceBinding) -> Result<(Vec<i8>, Vec<i8>)> {
    let e =
        uor_r4_integer::geometric_event_q4::unpack_coefficients(11712, &w.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))?;
    let a = AgeSource::new(age.as_tensor(), binding)?;
    let q = uor_r4_integer::geometric_event_q4::unpack_coefficients(256, a.packed_coefficients())
        .map_err(|e| invalid(e.to_string()))?;
    Ok((e, q))
}
fn clip(selected: &[(String, Var)], g: &mut GradStore) -> Result<(f64, Value)> {
    let mut square = 0.;
    let mut family = serde_json::Map::new();
    for (n, v) in selected {
        let Some(t) = g.get(v.as_tensor()) else {
            family.insert(n.clone(), json!({"connected":false,"l2":0.,"nonzero":0}));
            continue;
        };
        let x = t.flatten_all()?.to_vec1::<f32>()?;
        if x.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite answer gradient"));
        }
        let s = x.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        square += s;
        family.insert(
            n.clone(),
            json!({"connected":true,"l2":s.sqrt(),"nonzero":x.iter().filter(|x|**x!=0.).count()}),
        );
    }
    let norm = square.sqrt();
    if norm > 1. {
        for (_, v) in selected {
            if let Some(t) = g.get(v.as_tensor()) {
                let scaled = t.affine(1. / norm, 0.)?;
                g.insert(v.as_tensor(), scaled);
            }
        }
    }
    Ok((norm, json!(family)))
}
pub(super) fn training_batch(rng: &mut data::Rng, absolute_step: usize) -> Vec<data::Episode> {
    let n = [2, 4][absolute_step % 2];
    (0..8)
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
fn validate_batch(es: &[data::Episode], time: usize, targets: &[u32], mask: &[f32]) -> Result<()> {
    if es.len() != BATCH
        || time == 0
        || time > 128
        || targets.len() != BATCH * time
        || mask.len() != targets.len()
    {
        return Err(invalid("fixed B8 answer batch shape differs"));
    }
    for (b, e) in es.iter().enumerate() {
        if e.query + 1 != e.ids.len() || e.ids[e.query] != 34 || e.ids.len() > time {
            return Err(invalid("actual answer position differs"));
        }
        for t in 0..time {
            if mask[b * time + t] != if t == e.query { 1. } else { 0. }
                || (t == e.query && targets[b * time + t] != e.answer)
            {
                return Err(invalid("one answer per episode required"));
            }
        }
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn checkpoint(
    dir: &Path,
    step: usize,
    rng: u64,
    w: &EventWeights,
    age: &Var,
    base: &Path,
    tokenizer: &[u8],
    binding: &ReadSourceBinding,
    parent: &Value,
) -> Result<()> {
    report_output::claim(dir)?;
    w.save_source(&dir.join("event-source"), base, tokenizer)?;
    AgeSource::new(age.as_tensor(), binding)?.save(&dir.join("age-source"))?;
    write(
        &dir.join("checkpoint.json"),
        &json!({"schema":"uor-r4.event-age-fit-checkpoint/1",
        "completed_updates":step,"next_absolute_step":PARENT_UPDATES+step,"next_generator_state":rng,
        "parameters":parameter_hashes(w,age)?,"parent":parent,
        "optimizer_state":"NOT_SERIALIZED; weights and completed-update RNG only; any continuation restarts AdamW"}),
    )?;
    report_output::seal(dir)?;
    report_output::verify(dir)?;
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    root: &Path,
    w: &EventWeights,
    age: &Var,
    initial_rng: u64,
    deadline: Instant,
    base: &Path,
    tokenizer: &[u8],
    binding: &ReadSourceBinding,
    parent: &Value,
    frozen: &[Var],
    forward: impl Fn(
        &[u32],
        usize,
        usize,
        &EventWeights,
        &Tensor,
    ) -> Result<(Tensor, EventAgeCreditOutput)>,
) -> Result<FitReport> {
    let started = Instant::now();
    let selected = variables(w, age)?;
    let mut optimizer = AdamW::new(
        selected.iter().map(|(_, v)| v.clone()).collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let input = root.join("training-inputs");
    fs::create_dir(&input)?;
    let cp = root.join("checkpoints"); // checkpoint0 is saved before baseline evaluation by the caller.
    let mut rng = data::Rng(initial_rng);
    let mut history = Vec::new();
    let mut saved = 0;
    let mut last_cp = Instant::now();
    let mut r = FitReport {
        completed_updates: 0,
        completed_backward_batches: 0,
        initial_generator_state: initial_rng,
        next_generator_state: initial_rng,
        observed_generator_state_after_draws: initial_rng,
        prepared_batches: 0,
        actual_positions: 0,
        padded_positions: 0,
        minimum_episode_length: None,
        maximum_episode_length: 0,
        maximum_batch_time: 0,
        preparation_seconds: 0.,
        forward_backward_update_seconds: 0.,
        checkpoint_seconds: 0.,
        elapsed_seconds: 0.,
        event_q4_flips: 0,
        age_q4_flips: 0,
        event_projected_coordinates: 0,
        age_projected_coordinates: 0,
        clipped_updates: 0,
        failure: None,
        completed: false,
    };
    for step in 0..STEPS {
        if Instant::now() >= deadline {
            break;
        }
        let prepare = Instant::now();
        let before_rng = rng.0;
        let es = training_batch(&mut rng, PARENT_UPDATES + step);
        let bytes = serde_json::to_vec(&es)?;
        let digest = sha256_bytes(&bytes);
        fs::write(input.join(format!("batch-{step:04}.json")), bytes)?;
        let (ids, targets, mask, time) = data::batch(&es);
        validate_batch(&es, time, &targets, &mask)?;
        r.prepared_batches += 1;
        r.observed_generator_state_after_draws = rng.0;
        r.actual_positions += es.iter().map(|e| e.ids.len()).sum::<usize>();
        r.padded_positions += ids.len();
        r.maximum_batch_time = r.maximum_batch_time.max(time);
        for e in &es {
            r.minimum_episode_length = Some(
                r.minimum_episode_length
                    .map_or(e.ids.len(), |x| x.min(e.ids.len())),
            );
            r.maximum_episode_length = r.maximum_episode_length.max(e.ids.len());
        }
        r.preparation_seconds += prepare.elapsed().as_secs_f64();
        if Instant::now() >= deadline {
            rng.0 = before_rng;
            break;
        }
        let before = values(&selected)?;
        let old_codes = codes(w, age, binding)?;
        let at = Instant::now();
        let mut backward_completed = false;
        let attempt = (|| -> Result<(f32, f64, Value, usize, usize, usize, usize)> {
            let (logits, trace) = forward(&ids, BATCH, time, w, age.as_tensor())?;
            let mut all_hold = 0;
            let mut absent_answer = 0;
            for (b, e) in es.iter().enumerate() {
                all_hold += usize::from(
                    trace.span.actions[b * time..b * time + e.ids.len()]
                        .iter()
                        .all(|x| *x == 0),
                );
                absent_answer += usize::from(trace.span.prior_codes[b * time + e.query].is_none());
            }
            let loss = logits_cross_entropy(&logits, &targets, Some(&mask))?;
            let ce = loss.to_scalar::<f32>()?;
            if !ce.is_finite() {
                return Err(invalid("nonfinite same-model answer CE"));
            }
            let mut g = loss.backward()?;
            backward_completed = true;
            if frozen.iter().any(|v| g.get(v.as_tensor()).is_some()) {
                return Err(invalid("frozen parameter has answer gradient"));
            }
            let (norm, families) = clip(&selected, &mut g)?;
            optimizer.step(&g)?;
            let (ep, ap) = project(&selected)?;
            codes(w, age, binding)?; // Reject any invalid post-projection source before counting the update.
            Ok((ce, norm, families, ep, ap, all_hold, absent_answer))
        })();
        r.forward_backward_update_seconds += at.elapsed().as_secs_f64();
        r.completed_backward_batches += usize::from(backward_completed);
        let (ce, norm, families, ep, ap, all_hold, absent_answer) = match attempt {
            Ok(x) => x,
            Err(e) => {
                let raw = values(&selected)?
                    .into_iter()
                    .map(|x| x.into_iter().map(f32::to_bits).collect::<Vec<_>>())
                    .collect::<Vec<_>>();
                write(
                    &root.join("failed-current-shadow-bits.json"),
                    &json!({"families":selected.iter().map(|(n,_)|n).collect::<Vec<_>>(),"bits":raw}),
                )?;
                // End this optimizer run and restore the last COMPLETE update's weights.
                for ((_, v), x) in selected.iter().zip(before) {
                    v.set(&Tensor::from_vec(x, v.shape(), v.device())?)?;
                }
                r.failure = Some(e.to_string());
                write(
                    &root.join("training-failure.json"),
                    &json!({"error":e.to_string(),"attempted_batch":step,
                    "absolute_step":PARENT_UPDATES+step,"episode_sha256":digest,"rng_before":before_rng,"draw_rng_after":rng.0,
                    "completed_updates":r.completed_updates,"restored_last_completed_weights":true,
                    "optimizer_abandoned":true,"no_quality_verdict":true}),
                )?;
                break;
            }
        };
        r.completed_updates += 1;
        r.next_generator_state = rng.0;
        let new_codes = codes(w, age, binding)?;
        let ef = old_codes
            .0
            .iter()
            .zip(&new_codes.0)
            .filter(|(a, b)| a != b)
            .count();
        let af = old_codes
            .1
            .iter()
            .zip(&new_codes.1)
            .filter(|(a, b)| a != b)
            .count();
        r.event_q4_flips += ef;
        r.age_q4_flips += af;
        r.event_projected_coordinates += ep;
        r.age_projected_coordinates += ap;
        r.clipped_updates += usize::from(norm > 1.);
        let mut record = json!({"completed_updates":r.completed_updates,"absolute_step":PARENT_UPDATES+step,
            "rng_before":before_rng,"rng_after":rng.0,"episode_sha256":digest,"answer_mean_ce":ce,
            "answer_ce_sum":f64::from(ce)*8.,"answer_denominator":8,"gradient_l2_before_clip":norm,
            "event_q4_flips":ef,"age_q4_flips":af,"event_projected_coordinates":ep,"age_projected_coordinates":ap,
            "all_hold_episodes":all_hold,"absent_old_held_at_answer":absent_answer,
            "time":time,"actual_positions":es.iter().map(|e|e.ids.len()).sum::<usize>(),
            "seconds":at.elapsed().as_secs_f64(),"parameter_hashes":parameter_hashes(w,age)?});
        if step == 0 || (step + 1) % 80 == 0 {
            record["ordinary_answer_family_gradients"] = families;
        }
        history.push(record);
        write(&root.join("history.json"), &history)?;
        if r.completed_updates % 80 == 0 || last_cp.elapsed().as_secs() >= 900 {
            let at = Instant::now();
            checkpoint(
                &cp.join(format!("step-{:04}", r.completed_updates)),
                r.completed_updates,
                r.next_generator_state,
                w,
                age,
                base,
                tokenizer,
                binding,
                parent,
            )?;
            r.checkpoint_seconds += at.elapsed().as_secs_f64();
            saved = r.completed_updates;
            last_cp = Instant::now();
        }
        write(&root.join("fit-progress.json"), &r)?;
    }
    if saved != r.completed_updates {
        let at = Instant::now();
        checkpoint(
            &cp.join(format!("step-{:04}", r.completed_updates)),
            r.completed_updates,
            r.next_generator_state,
            w,
            age,
            base,
            tokenizer,
            binding,
            parent,
        )?;
        r.checkpoint_seconds += at.elapsed().as_secs_f64();
    }
    r.completed = r.completed_updates == STEPS && r.failure.is_none();
    r.elapsed_seconds = started.elapsed().as_secs_f64();
    write(&root.join("fit-progress.json"), &r)?;
    Ok(r)
}
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    #[test]
    fn event_age_fit_sampler_exact_rng_and_mask() -> Result<()> {
        // Pin the copied sampler body to the independently checked actual-parent driver.
        let body = |s: &str| {
            s.split("fn training_batch(")
                .nth(1)
                .and_then(|x| x.split("\nfn ").next())
                .map(|x| x.split_whitespace().collect::<String>())
        };
        assert_eq!(
            body(include_str!("fit.rs")),
            body(include_str!("../attention-geometric-event-age-check.rs"))
        );
        for initial in [17847168693577953940u64, 8892160473256393877u64] {
            let (mut a, mut b) = (data::Rng(initial), data::Rng(initial));
            for step in 3200..3202 {
                let es = training_batch(&mut a, step);
                assert_eq!(es, training_batch(&mut b, step));
                assert_eq!(a.0, b.0);
                let (_, target, mask, t) = data::batch(&es);
                validate_batch(&es, t, &target, &mask)?;
                assert_eq!(mask.iter().sum::<f32>(), 8.);
                assert!(es.iter().all(|e| e.facts == [2, 4][step % 2]));
            }
            let j: Value = serde_json::from_slice(&serde_json::to_vec(&json!({"rng":initial}))?)?;
            assert_eq!(j["rng"].as_u64(), Some(initial));
        }
        Ok(())
    }
    #[test]
    fn event_age_fit_inventory_projection_and_selected_clip() -> Result<()> {
        let w = EventWeights::new(40, 2, 1)?.into_q4()?;
        let age = Var::from_vec(
            (0..256).map(|i| prior(i / 128, i % 128)).collect(),
            (2, 128),
            &Device::Cpu,
        )?;
        let selected = variables(&w, &age)?;
        assert_eq!(selected.len(), 6);
        let frozen = Var::new(100f32, &Device::Cpu)?;
        let mut loss = frozen.sqr()?;
        for (_, v) in &selected {
            loss = loss.add(&v.sum_all()?)?;
        }
        let mut g = loss.backward()?;
        let (norm, _) = clip(&selected, &mut g)?;
        assert!((norm - (11968f64).sqrt()).abs() < 1e-6);
        assert_eq!(
            g.get(frozen.as_tensor())
                .ok_or_else(|| invalid("frozen fixture gradient absent"))?
                .to_scalar::<f32>()?,
            200.
        );
        let v = &selected[0].1;
        v.set(&Tensor::from_vec(
            vec![2f32; v.elem_count()],
            v.shape(),
            &Device::Cpu,
        )?)?;
        age.set(&Tensor::from_vec(
            (0..256)
                .map(|i| prior(i / 128, i % 128) + 1.)
                .collect::<Vec<_>>(),
            (2, 128),
            &Device::Cpu,
        )?)?;
        let (e, a) = project(&selected)?;
        assert_eq!(e, v.elem_count());
        assert_eq!(a, 256);
        assert_eq!(age.to_vec2::<f32>()?[0][127], -127. / 16. + 0.875);
        assert_eq!(age.to_vec2::<f32>()?[1][127], -127. / 256. + 0.875);
        assert!(v
            .flatten_all()?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| *x == 1.75));
        Ok(())
    }
}
