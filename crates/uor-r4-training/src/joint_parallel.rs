//! Synchronous CPU gradients over independent full-window batch rows.
//!
//! Shared model variables are read-only until every worker has joined. Each
//! worker owns its complete recurrent graph; no sequence or gradient history
//! is truncated. Named mean gradients are combined in shard order, before the
//! caller performs one global clip and one optimizer update. F32 reductions
//! differ from an unsplit batch; bitwise trajectory equivalence is not claimed.

use candle_core::backprop::GradStore;
use candle_core::{DType, Device};

use crate::joint_model::{JointModel, ReadMode};
use crate::{invalid, Result, TrainingError};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

pub struct BatchGradients {
    pub mean_nll: f32,
    pub gradients: GradStore,
}

fn shard_gradients(
    model: &JointModel,
    inputs: &[u32],
    targets: &[u32],
    batch: usize,
    time: usize,
) -> Result<BatchGradients> {
    let output = model.forward(inputs, batch, time, ReadMode::Enabled, true)?;
    let loss = output.loss(targets)?;
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
    {
        return Err(invalid(
            "invalid full-window CPU gradient shard configuration",
        ));
    }
    if shards == 1 {
        return shard_gradients(model, inputs, targets, batch, time);
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
            match std::thread::Builder::new()
                .name(format!("joint-gradient-{shard}"))
                .stack_size(WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || {
                    shard_gradients(model, inputs, targets, shard_batch, time)
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
    fn full_window_shards_match_loss_and_every_named_gradient() -> Result<()> {
        let model = JointModel::new(
            JointConfig {
                width: 128,
                context: 8,
                transport: Transport::Quaternion,
                seed: 53,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        let inputs: Vec<u32> = (0..32).map(|index| (index * 7 + 3) % 101).collect();
        let targets: Vec<u32> = (0..32).map(|index| (index * 7 + 10) % 101).collect();
        let complete = batch_gradients(&model, &inputs, &targets, 4, 8, 1)?;
        for shards in [2, 4] {
            let split = batch_gradients(&model, &inputs, &targets, 4, 8, shards)?;
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
}
