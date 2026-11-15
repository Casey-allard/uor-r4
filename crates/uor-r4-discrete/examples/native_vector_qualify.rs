//! Finite RTX 4090 qualification for the native Q31 Hamilton vector operator.
//! Run with:
//! `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 cargo run -p
//! uor-r4-discrete --features cuda --example native_vector_qualify`

use std::time::Instant;
use uor_r4_discrete::native_vector::{
    q31_hamilton_rotate, q31_hamilton_train_step, Q31HamiltonModel,
};
use uor_r4_discrete::native_vector_device::{
    compiled_native_vector_image, native_vector_entry, native_vector_training_entry,
    NativeVectorEngine,
};

const ITEMS: usize = 4096;

fn values(seed: u64) -> Vec<[i64; 4]> {
    let mut out = vec![
        [i64::from(i32::MIN); 4],
        [1 << 31, -(1 << 31), 1, -1],
        [1 << 30, 1 << 29, -(1 << 28), 7],
        [0, 1 << 31, 0, -(1 << 31)],
    ];
    let mut seed = seed;
    while out.len() < ITEMS {
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as i32
        };
        out.push([
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
            i64::from(next()),
        ]);
    }
    out
}

fn main() -> uor_r4_discrete::Result<()> {
    let engine = NativeVectorEngine::new(0)?;
    let gates = values(0x6a09_e667_f3bc_c909);
    let held = values(0xbb67_ae85_84ca_a73b);
    let started = Instant::now();
    let cuda = engine.hamilton_rotate(&gates, &held)?;
    let elapsed_ns = started.elapsed().as_nanos();
    for (index, ((gate, value), observed)) in gates.iter().zip(&held).zip(&cuda).enumerate() {
        let expected = q31_hamilton_rotate(*gate, *value)?;
        if *observed != expected {
            return Err(uor_r4_discrete::Error::Invalid(format!(
                "Q31 parity differs at item {index}: CUDA {observed:?}, Rust {expected:?}"
            )));
        }
    }
    let targets = values(0x3c6e_f372_fe94_f82b);
    let training_started = Instant::now();
    let training_cuda = engine.hamilton_train_step(&gates, &held, &targets, 8)?;
    for index in 0..ITEMS {
        let expected = q31_hamilton_train_step(gates[index], held[index], targets[index], 8)?;
        if training_cuda[index] != expected {
            return Err(uor_r4_discrete::Error::Invalid(format!(
                "Q31 training-step parity differs at item {index}"
            )));
        }
    }
    let training_wall_ns = training_started.elapsed().as_nanos();

    let one = 1 << 31;
    let target = [1_518_500_249, 1_518_500_249, 0, 0];
    let mut gate = [one, 0, 0, 0];
    let initial_loss = q31_hamilton_train_step(gate, [one, 0, 0, 0], target, 8)?.loss_before;
    let mut accepted_steps = 0;
    for _ in 0..64 {
        let step = engine.hamilton_train_step(&[gate], &[[one, 0, 0, 0]], &[target], 8)?;
        accepted_steps += u32::from(step[0].accepted);
        gate = step[0].accepted_gate;
    }
    let final_loss = q31_hamilton_train_step(gate, [one, 0, 0, 0], target, 8)?.loss_before;

    let mut model = Q31HamiltonModel::new([one, 0, 0, 0], 8)?;
    let batch_values = vec![[one, 0, 0, 0]; 8];
    let batch_targets = vec![target; 8];
    let first_model_step = engine.train_model_step(&mut model, &batch_values, &batch_targets)?;
    for _ in 1..64 {
        engine.train_model_step(&mut model, &batch_values, &batch_targets)?;
    }
    let mut model_probe = model;
    let final_model_loss = engine
        .train_model_step(&mut model_probe, &batch_values, &batch_targets)?
        .loss_before;
    // The exact 48-byte state this GPU training produced: the same bytes serving reads.
    let state = model.to_bytes();
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let state_hex = hex(&state);
    let state_sha256 = {
        use sha2::{Digest, Sha256};
        hex(&Sha256::digest(state))
    };
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(&path, state)
            .map_err(|e| uor_r4_discrete::Error::Invalid(format!("state-out {path}: {e}")))?;
    }
    println!(
        "{}",
        serde_json::json!({
            "schema": "uor-r4.native-vector-qualification/2",
            "operator": native_vector_entry(),
            "training_operator": native_vector_training_entry(),
            "state": "q31:i64x4",
            "items": ITEMS,
            "raw_words_compared": ITEMS * 4,
            "bit_identical": true,
            "wall_ns_including_transfer": elapsed_ns,
            "training_steps_compared": ITEMS,
            "training_step_bit_identical": true,
            "training_wall_ns_including_transfer": training_wall_ns,
            "finite_training_initial_loss": initial_loss,
            "finite_training_final_loss": final_loss,
            "finite_training_steps": 64,
            "finite_training_accepted_steps": accepted_steps,
            "shared_model_initial_loss": first_model_step.loss_before,
            "shared_model_final_loss": final_model_loss,
            "shared_model_epoch": model.epoch(),
            "shared_model_step_shift": model.step_shift(),
            "shared_model_gate": model.gate(),
            "shared_model_state_hex": state_hex,
            "shared_model_state_sha256": state_sha256,
            "image_source_ptx_cubin_report_sha256": compiled_native_vector_image(),
            "first": cuda.first(),
            "last": cuda.last(),
        })
    );
    Ok(())
}
