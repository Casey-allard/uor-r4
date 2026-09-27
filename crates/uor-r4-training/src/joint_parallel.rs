//! Synchronous CPU gradients over independent full-window batch rows.
//!
//! Shared model variables are read-only until every worker has joined. Each
//! worker owns its complete recurrent graph; no sequence or gradient history
//! is truncated. Named mean gradients are combined in shard order, before the
//! caller performs one global clip and one optimizer update. F32 reductions
//! differ from an unsplit batch; bitwise trajectory equivalence is not claimed.
//! With optional target weights, each shard normalizes by its own weight sum
//! before batch-fraction combination. Unequal shard weight sums therefore change
//! the objective relative to a global weighted mean, beyond reduction rounding.

use candle_core::backprop::GradStore;
use candle_core::{DType, Device};

use crate::joint_model::{JointModel, ReadMode};
use crate::{invalid, Result, TrainingError};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

pub struct BatchGradients {
    /// Actual objective value: weighted when target weights are supplied.
    pub mean_nll: f32,
    pub gradients: GradStore,
}

/// Response-token mean over the whole batch, including genuine EOS targets.
/// This separate reducer does not change historical termination weighting.
pub struct ResponseGradients {
    pub mean_nll: f32,
    pub supervised_targets: usize,
    pub gradients: GradStore,
}

/// Binary response masks preserve gradients through every observed prefix.
/// Every row must contain supervision; empty shards are not silently skipped.
pub fn response_batch_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    weights: &[f32],
    batch: usize,
    time: usize,
    shards: usize,
) -> Result<ResponseGradients> {
    if !matches!(shards, 1 | 2 | 4)
        || batch == 0
        || batch > 64
        || batch % shards != 0
        || time == 0
        || time > model.config.context
        || batch.checked_mul(time) != Some(inputs.len())
        || targets.len() != inputs.len()
        || weights.len() != inputs.len()
        || inputs
            .iter()
            .chain(targets)
            .any(|&id| id as usize >= model.config.vocab_size)
        || weights.iter().any(|&w| w != 0.0 && w != 1.0)
        || weights.chunks(time).any(|row| !row.contains(&1.0))
    {
        return Err(invalid("invalid complete-response gradient batch/mask"));
    }
    let supervised_targets = weights.iter().filter(|&&w| w == 1.0).count();
    if shards == 1 {
        let result = shard_gradients(model, inputs, targets, Some(weights), batch, time)?;
        return Ok(ResponseGradients {
            mean_nll: result.mean_nll,
            supervised_targets,
            gradients: result.gradients,
        });
    }
    if !matches!(model.device(), Device::Cpu) {
        return Err(invalid("multiple response shards require CPU"));
    }
    let shard_batch = batch / shards;
    let shard_tokens = shard_batch * time;
    let masses: Vec<f64> = weights
        .chunks(shard_tokens)
        .map(|row| row.iter().filter(|&&w| w == 1.0).count() as f64 / supervised_targets as f64)
        .collect();
    let mut results = std::thread::scope(|scope| -> Result<Vec<BatchGradients>> {
        let mut handles = Vec::with_capacity(shards);
        let mut first_error: Option<TrainingError> = None;
        for shard in 0..shards {
            let start = shard * shard_tokens;
            let inputs = &inputs[start..start + shard_tokens];
            let targets = &targets[start..start + shard_tokens];
            let weights = &weights[start..start + shard_tokens];
            match std::thread::Builder::new()
                .name(format!("dialogue-gradient-{shard}"))
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || {
                    shard_gradients(model, inputs, targets, Some(weights), shard_batch, time)
                }) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    first_error = Some(error.into());
                    break;
                }
            }
        }
        let mut outputs = Vec::with_capacity(shards);
        for handle in handles {
            match handle.join() {
                Ok(Ok(output)) => outputs.push(output),
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(_) => {
                    if first_error.is_none() {
                        first_error = Some(invalid("response gradient worker panicked"));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(outputs)
    })?;
    let mean_nll = results
        .iter()
        .zip(&masses)
        .map(|(result, mass)| f64::from(result.mean_nll) * mass)
        .sum::<f64>() as f32;
    let (first, others) = results
        .split_first_mut()
        .ok_or_else(|| invalid("response workers returned no gradients"))?;
    for (name, variable) in model.variables() {
        let gradient = first
            .gradients
            .remove(variable.as_tensor())
            .ok_or_else(|| invalid(format!("response shard0 missing {name}")))?;
        if gradient.dtype() != DType::F32
            || gradient.dims() != variable.dims()
            || !gradient.device().same_device(variable.device())
        {
            return Err(invalid(format!(
                "response shard0 gradient binding differs for {name}"
            )));
        }
        let mut combined = gradient.detach().affine(masses[0], 0.0)?;
        for (index, result) in others.iter_mut().enumerate() {
            let gradient = result
                .gradients
                .remove(variable.as_tensor())
                .ok_or_else(|| invalid(format!("response shard{} missing {name}", index + 1)))?;
            if gradient.dtype() != DType::F32
                || gradient.dims() != variable.dims()
                || !gradient.device().same_device(variable.device())
            {
                return Err(invalid(format!(
                    "response gradient binding differs for {name}"
                )));
            }
            combined = combined.add(&gradient.detach().affine(masses[index + 1], 0.0)?)?;
        }
        first.gradients.insert(variable.as_tensor(), combined);
    }
    let result = results.remove(0);
    Ok(ResponseGradients {
        mean_nll,
        supervised_targets,
        gradients: result.gradients,
    })
}

fn shard_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    target_weights: Option<&[f32]>,
    batch: usize,
    time: usize,
) -> Result<BatchGradients> {
    let output = model.forward(inputs, batch, time, ReadMode::Enabled, true)?;
    let loss = match target_weights {
        None => output.loss(targets)?,
        Some(weights) => output.weighted_loss(targets, weights)?,
    };
    let mean_nll = loss.to_scalar::<f32>()?;
    if !mean_nll.is_finite() {
        return Err(invalid("nonfinite training shard loss"));
    }
    let gradients = loss.backward()?;
    Ok(BatchGradients {
        mean_nll,
        gradients,
    })
}

/// Split only independent batch rows. Sampling, target count and optimizer
/// update frequency remain the caller's original full-batch configuration.
pub fn batch_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    target_weights: Option<&[f32]>,
    batch: usize,
    time: usize,
    shards: usize,
) -> Result<BatchGradients> {
    if !matches!(shards, 1 | 2 | 4)
        || batch == 0
        || batch > 64
        || batch % shards != 0
        || time == 0
        || time > model.config.context
        || batch.checked_mul(time) != Some(inputs.len())
        || targets.len() != inputs.len()
        || target_weights.is_some_and(|weights| weights.len() != inputs.len())
    {
        return Err(invalid(
            "invalid full-window CPU gradient shard configuration",
        ));
    }
    if shards == 1 {
        return shard_gradients(model, inputs, targets, target_weights, batch, time);
    }
    if !matches!(model.device(), Device::Cpu) {
        return Err(invalid("multiple gradient shards require a CPU model"));
    }
    let shard_batch = batch / shards;
    let shard_tokens = shard_batch * time;
    let mut results = std::thread::scope(|scope| -> Result<Vec<BatchGradients>> {
        let mut handles = Vec::with_capacity(shards);
        let mut first_error: Option<TrainingError> = None;
        for shard in 0..shards {
            let start = shard * shard_tokens;
            let inputs = &inputs[start..start + shard_tokens];
            let targets = &targets[start..start + shard_tokens];
            let weights = target_weights.map(|weights| &weights[start..start + shard_tokens]);
            match std::thread::Builder::new()
                .name(format!("joint-gradient-{shard}"))
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || {
                    shard_gradients(model, inputs, targets, weights, shard_batch, time)
                }) {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    first_error = Some(error.into());
                    break;
                }
            }
        }
        let mut outputs = Vec::with_capacity(shards);
        // Join every spawned worker even after an error. Returning early would
        // let an unjoined worker panic escape through thread::scope instead.
        for handle in handles {
            match handle.join() {
                Ok(Ok(output)) => outputs.push(output),
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(_) => {
                    if first_error.is_none() {
                        first_error = Some(invalid("CPU gradient worker panicked"));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(outputs)
    })?;

    let weight = shard_batch as f64 / batch as f64;
    let mean_nll = results
        .iter()
        .map(|result| f64::from(result.mean_nll) * weight)
        .sum::<f64>() as f32;
    // Reuse the first store because Candle intentionally keeps GradStore::new
    // private. Insertion uses the shared original Var IDs, never shard IDs.
    let (first, others) = results
        .split_first_mut()
        .ok_or_else(|| invalid("CPU gradient workers returned no results"))?;
    for (name, variable) in model.variables() {
        let first_gradient = first
            .gradients
            .remove(variable.as_tensor())
            .ok_or_else(|| invalid(format!("shard0 missing gradient {name}")))?;
        let mut combined = first_gradient.detach().affine(weight, 0.0)?;
        for (index, result) in others.iter_mut().enumerate() {
            let gradient = result
                .gradients
                .remove(variable.as_tensor())
                .ok_or_else(|| invalid(format!("shard{} missing gradient {name}", index + 1)))?;
            if gradient.dtype() != DType::F32
                || gradient.dims() != variable.dims()
                || !gradient.device().same_device(variable.device())
            {
                return Err(invalid(format!(
                    "CPU shard gradient binding differs for {name}"
                )));
            }
            combined = combined.add(&gradient.detach().affine(weight, 0.0)?)?;
        }
        first.gradients.insert(variable.as_tensor(), combined);
    }
    first.mean_nll = mean_nll;
    Ok(results.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint_model::{JointConfig, Transport};

    #[test]
    fn response_shards_preserve_global_target_mean_and_padding_causality() -> Result<()> {
        let model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                seed: 97,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        let inputs: Vec<u32> = (0..32).map(|i| (i * 7 + 3) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|i| (i * 7 + 10) % 101).collect();
        let mut weights = vec![0.0; 32];
        for i in [2, 3, 12, 18, 19, 20, 21, 22, 25, 26] {
            weights[i] = 1.0;
        }
        // Two shards have 3 and 7 response targets; batch-fraction reduction
        // would implement a different objective despite identical lane counts.
        let expected = response_batch_gradients(&model, &inputs, &targets, &weights, 4, 8, 1)?;
        let check = |actual: ResponseGradients| -> Result<()> {
            assert_eq!(actual.supervised_targets, 10);
            assert!((actual.mean_nll - expected.mean_nll).abs() < 2e-5);
            for (name, variable) in model.variables() {
                let a = actual
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("masked gradient missing {name}")))?;
                let e = expected
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("full response gradient missing {name}")))?;
                let delta = a.sub(e)?.abs()?.max_all()?.to_scalar::<f32>()?;
                let scale = e.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(
                    delta.is_finite() && delta <= 2e-5 + 2e-4 * scale,
                    "response gradient {name}: delta {delta}, scale {scale}"
                );
            }
            Ok(())
        };
        for shards in [2, 4] {
            check(response_batch_gradients(
                &model, &inputs, &targets, &weights, 4, 8, shards,
            )?)?;
        }
        let mut padded_inputs = inputs.clone();
        let mut padded_targets = targets.clone();
        for row in 0..4 {
            let last = weights[row * 8..(row + 1) * 8]
                .iter()
                .rposition(|&w| w == 1.0)
                .ok_or_else(|| invalid("fixture missing response"))?;
            for time in last + 1..8 {
                padded_inputs[row * 8 + time] = 103;
                padded_targets[row * 8 + time] = 104;
            }
        }
        check(response_batch_gradients(
            &model,
            &padded_inputs,
            &padded_targets,
            &weights,
            4,
            8,
            2,
        )?)?;
        for invalid_weight in [f32::NAN, -1.0, 0.5, 2.0] {
            let mut bad = weights.clone();
            bad[2] = invalid_weight;
            assert!(response_batch_gradients(&model, &inputs, &targets, &bad, 4, 8, 2).is_err());
        }
        weights[..8].fill(0.0);
        assert!(response_batch_gradients(&model, &inputs, &targets, &weights, 4, 8, 2).is_err());
        Ok(())
    }

    #[test]
    fn full_window_shards_match_loss_and_every_named_gradient() -> Result<()> {
        check_full_window_shards(false)
    }

    #[test]
    fn quantized_shards_keep_original_variable_gradients() -> Result<()> {
        check_full_window_shards(true)
    }

    fn check_full_window_shards(quantized: bool) -> Result<()> {
        let mut model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                transport: Transport::Quaternion,
                seed: 53,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        if quantized {
            model.configure_quantization(0, 1)?;
        }
        let inputs: Vec<u32> = (0..32).map(|index| (index * 7 + 3) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|index| (index * 7 + 10) % 101).collect();
        let complete = batch_gradients(&model, &inputs, &targets, None, 4, 8, 1)?;
        for shards in [2, 4] {
            let split = batch_gradients(&model, &inputs, &targets, None, 4, 8, shards)?;
            assert!((complete.mean_nll - split.mean_nll).abs() < 2e-5);
            for (name, variable) in model.variables() {
                let expected = complete
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("full batch missing gradient {name}")))?;
                let actual = split
                    .gradients
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid(format!("split batch missing gradient {name}")))?;
                let delta = actual.sub(expected)?.abs()?.max_all()?.to_scalar::<f32>()?;
                let scale = expected.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(
                    delta.is_finite() && delta <= 2e-5 + 2e-4 * scale,
                    "{shards} shards, {name}: delta {delta}, scale {scale}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn end_weight_uniform_weights_match_default_gradients() -> Result<()> {
        let model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                transport: Transport::Quaternion,
                seed: 71,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        let inputs: Vec<u32> = (0..32).map(|index| (index * 5 + 1) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|index| (index * 5 + 8) % 101).collect();
        let plain = batch_gradients(&model, &inputs, &targets, None, 4, 8, 1)?;
        let ones = vec![1.0f32; 32];
        let weighted = batch_gradients(&model, &inputs, &targets, Some(&ones), 4, 8, 1)?;
        assert!((plain.mean_nll - weighted.mean_nll).abs() < 1e-5);
        for (name, variable) in model.variables() {
            let expected = plain
                .gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("default path missing gradient {name}")))?;
            let actual = weighted
                .gradients
                .get(variable.as_tensor())
                .ok_or_else(|| invalid(format!("weighted path missing gradient {name}")))?;
            let delta = actual.sub(expected)?.abs()?.max_all()?.to_scalar::<f32>()?;
            assert!(delta < 1e-5, "uniform end weights changed {name}: {delta}");
        }
        Ok(())
    }
}
