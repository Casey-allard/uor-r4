//! AOT CUDA handle for native Q31 Hamilton vector transport and training steps.
//! The module contains only admitted integer entry points and no PTX/JIT fallback.

use crate::native_vector::{
    q31_batch_len_admitted, q31_hamilton_rotate, q31_hamilton_train_step, q31_quaternion_admitted,
    Q31HamiltonModel, Q31ModelStep, Q31Quaternion, Q31TrainingStep, Q31Transport,
    MAX_Q31_STEP_SHIFT,
};
use crate::{Error, Result};
use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, Ptx, PushKernelArg};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE: &str = include_str!("native_vector.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/native-vector.cubin"));
const RESOURCES: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector-resources.txt"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
const ENTRY_ROTATE: &str = "native_q31_hamilton_rotate";
const ENTRY_TRAIN_STEP: &str = "native_q31_hamilton_train_step";

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

/// Source, PTX, cubin and ptxas-report identities for the embedded image.
pub fn compiled_native_vector_image() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
        format!("{:x}", Sha256::digest(stable_resource_report().as_bytes())),
    ]
}

/// The compiler's stopwatch is not part of the artifact identity.
fn stable_resource_report() -> String {
    RESOURCES
        .lines()
        .filter(|line| !line.contains("Compile time ="))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One explicit RTX-class CUDA context for native integer vector training.
pub struct NativeVectorEngine {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    rotate: CudaFunction,
    train_step: CudaFunction,
}

impl NativeVectorEngine {
    /// Load the admitted `native-vector.cubin`; no PTX fallback is attempted.
    pub fn new(ordinal: usize) -> Result<Self> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (major, minor) = context.compute_capability().map_err(cuda)?;
        if TARGET.trim() != format!("sm_{major}{minor}") {
            return Err(invalid(format!(
                "native-vector cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let rotate = module.load_function(ENTRY_ROTATE).map_err(cuda)?;
        let train_step = module.load_function(ENTRY_TRAIN_STEP).map_err(cuda)?;
        Ok(Self {
            _context: context,
            stream,
            rotate,
            train_step,
        })
    }

    /// Rotate one batch of Q31 quaternions on CUDA and return raw Q31 words.
    pub fn hamilton_rotate(
        &self,
        gates: &[Q31Quaternion],
        values: &[Q31Quaternion],
    ) -> Result<Vec<Q31Transport>> {
        if gates.is_empty() || gates.len() != values.len() {
            return Err(invalid(
                "native-vector transport needs a nonempty equal-length batch",
            ));
        }
        if !q31_batch_len_admitted(gates.len()) {
            return Err(invalid(
                "native-vector batch exceeds the admitted 2^30-item bound",
            ));
        }
        if gates
            .iter()
            .chain(values)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(invalid(
                "native-vector Q31 coordinates must be in -2^31..=2^31",
            ));
        }
        let count = u32::try_from(gates.len())
            .map_err(|_| invalid("native-vector batch exceeds the CUDA index bound"))?;
        let gate_words: Vec<i64> = gates.iter().flatten().copied().collect();
        let value_words: Vec<i64> = values.iter().flatten().copied().collect();
        let gate_device = self.stream.clone_htod(&gate_words).map_err(cuda)?;
        let value_device = self.stream.clone_htod(&value_words).map_err(cuda)?;
        let mut output = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.rotate);
        launch
            .arg(&gate_device)
            .arg(&value_device)
            .arg(&mut output)
            .arg(&count);
        // SAFETY: every buffer carries exactly `count * 4` words on this stream.
        unsafe {
            launch.launch(LaunchConfig {
                grid_dim: (count.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
        }
        .map_err(cuda)?;
        let words: Vec<i64> = self.stream.clone_dtoh(&output).map_err(cuda)?;
        Ok(words
            .chunks_exact(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect())
    }

    /// Run the Rust oracle on one quaternion in the same batch representation.
    pub fn hamilton_rotate_reference(
        gate: Q31Quaternion,
        value: Q31Quaternion,
    ) -> Result<Q31Transport> {
        q31_hamilton_rotate(gate, value)
    }

    /// Apply one bounded Q31 Hamilton training step per item on CUDA.
    pub fn hamilton_train_step(
        &self,
        gates: &[Q31Quaternion],
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
        step_shift: u32,
    ) -> Result<Vec<Q31TrainingStep>> {
        if gates.is_empty() || gates.len() != values.len() || gates.len() != targets.len() {
            return Err(invalid(
                "native-vector training needs nonempty equal-length gate, value and target batches",
            ));
        }
        if !q31_batch_len_admitted(gates.len()) {
            return Err(invalid(
                "native-vector training batch exceeds the admitted 2^30-item bound",
            ));
        }
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(invalid("native-vector step_shift must be in 0..=31"));
        }
        if gates
            .iter()
            .chain(values)
            .chain(targets)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(invalid(
                "native-vector Q31 training words must be in -2^31..=2^31",
            ));
        }
        let count = u32::try_from(gates.len())
            .map_err(|_| invalid("native-vector training batch exceeds the CUDA index bound"))?;
        let gate_words: Vec<i64> = gates.iter().flatten().copied().collect();
        let value_words: Vec<i64> = values.iter().flatten().copied().collect();
        let target_words: Vec<i64> = targets.iter().flatten().copied().collect();
        let gate_device = self.stream.clone_htod(&gate_words).map_err(cuda)?;
        let value_device = self.stream.clone_htod(&value_words).map_err(cuda)?;
        let target_device = self.stream.clone_htod(&target_words).map_err(cuda)?;
        let mut predictions = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut candidates = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut accepted = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut loss_before = self.stream.alloc_zeros::<u64>(gates.len()).map_err(cuda)?;
        let mut loss_after = self.stream.alloc_zeros::<u64>(gates.len()).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.train_step);
        launch
            .arg(&gate_device)
            .arg(&value_device)
            .arg(&target_device)
            .arg(&count)
            .arg(&step_shift)
            .arg(&mut predictions)
            .arg(&mut candidates)
            .arg(&mut accepted)
            .arg(&mut loss_before)
            .arg(&mut loss_after);
        unsafe {
            launch.launch(LaunchConfig {
                grid_dim: (count.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
        }
        .map_err(cuda)?;
        let predictions: Vec<i64> = self.stream.clone_dtoh(&predictions).map_err(cuda)?;
        let candidates: Vec<i64> = self.stream.clone_dtoh(&candidates).map_err(cuda)?;
        let accepted: Vec<i64> = self.stream.clone_dtoh(&accepted).map_err(cuda)?;
        let loss_before: Vec<u64> = self.stream.clone_dtoh(&loss_before).map_err(cuda)?;
        let loss_after: Vec<u64> = self.stream.clone_dtoh(&loss_after).map_err(cuda)?;
        Ok((0..gates.len())
            .map(|index| Q31TrainingStep {
                prediction: [
                    predictions[index * 4],
                    predictions[index * 4 + 1],
                    predictions[index * 4 + 2],
                    predictions[index * 4 + 3],
                ],
                candidate_gate: [
                    candidates[index * 4],
                    candidates[index * 4 + 1],
                    candidates[index * 4 + 2],
                    candidates[index * 4 + 3],
                ],
                accepted_gate: [
                    accepted[index * 4],
                    accepted[index * 4 + 1],
                    accepted[index * 4 + 2],
                    accepted[index * 4 + 3],
                ],
                loss_before: loss_before[index],
                loss_after: loss_after[index],
                accepted: loss_after[index] < loss_before[index],
            })
            .collect())
    }

    /// Run the Rust training-step oracle for one item.
    pub fn hamilton_train_step_reference(
        gate: Q31Quaternion,
        value: Q31Quaternion,
        target: Q31Quaternion,
        step_shift: u32,
    ) -> Result<Q31TrainingStep> {
        q31_hamilton_train_step(gate, value, target, step_shift)
    }

    /// Apply one shared Q31 model step: CUDA per-item credit, then deterministic aggregation.
    pub fn train_model_step(
        &self,
        model: &mut Q31HamiltonModel,
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        if values.is_empty() || values.len() != targets.len() {
            return Err(invalid(
                "native Q31 model step needs nonempty equal-length values and targets",
            ));
        }
        let gates = vec![model.gate(); values.len()];
        let records = self.hamilton_train_step(&gates, values, targets, model.step_shift())?;
        model.step_from_records(&records, values, targets)
    }

    /// Run the shared model step through the Rust oracle.
    pub fn train_model_step_reference(
        model: &mut Q31HamiltonModel,
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        model.step_reference(values, targets)
    }
}

/// Compile-time proof that the native device API remains integer-only.
pub fn native_vector_entry() -> &'static str {
    ENTRY_ROTATE
}

/// The second admitted entry: one bounded Q31 training step per batch item.
pub fn native_vector_training_entry() -> &'static str {
    ENTRY_TRAIN_STEP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_identity_excludes_the_compiler_stopwatch() {
        assert!(!stable_resource_report().contains("Compile time ="));
        assert!(compiled_native_vector_image()
            .iter()
            .all(|digest| digest.len() == 64));
    }
}
