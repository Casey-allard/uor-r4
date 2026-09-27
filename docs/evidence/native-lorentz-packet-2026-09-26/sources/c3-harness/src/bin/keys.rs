//! Scratch analysis: per-token written-key norms and NoRead masses of a saved
//! JointModel on consecutive held-out windows (fresh state per window).
//! key = read.key(rms(state)), exactly the model's write path without quantization.
//!
//! usage: keys MODEL_DIR VALID.u16 WINDOWS OUT_PREFIX
//! writes OUT_PREFIX.{key_norm,no_read,copy_gate}.f32 (token order from index 0)

use std::error::Error;
use std::fs;
use std::path::Path;

use candle_core::{Device, D};
use uor_r4_training::joint_model::{JointModel, ReadMode, RMS_EPSILON};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: keys MODEL_DIR VALID.u16 WINDOWS OUT_PREFIX".into());
    }
    let device = Device::Cpu;
    let model = JointModel::load(Path::new(&args[1]), &device)?;
    let bytes = fs::read(&args[2])?;
    let valid: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|c| u32::from(u16::from_le_bytes([c[0], c[1]])))
        .collect();
    let windows: usize = args[3].parse()?;
    let time = model.config.context;
    let weight = model.variables()["read.key.weight"].as_tensor().clone();
    let bias = model.variables()["read.key.bias"].as_tensor().clone();
    let mut norms = Vec::new();
    let mut no_reads = Vec::new();
    let mut gates = Vec::new();
    let starts: Vec<usize> = (0..windows).map(|w| w * time).collect();
    for group in starts.chunks(32) {
        let b = group.len();
        let mut ids = Vec::with_capacity(b * time);
        for &s in group {
            ids.extend_from_slice(&valid[s..s + time]);
        }
        let out = model.forward(&ids, b, time, ReadMode::Enabled, false)?;
        let states = out.states.reshape((b * time, model.config.width))?;
        let rms = states
            .sqr()?
            .mean_keepdim(D::Minus1)?
            .affine(1.0, RMS_EPSILON)?
            .sqrt()?;
        let normalized = states.broadcast_div(&rms)?;
        let keys = normalized.matmul(&weight.t()?)?.broadcast_add(&bias)?;
        let key_norm: Vec<f32> = keys.sqr()?.sum(D::Minus1)?.sqrt()?.to_vec1()?;
        norms.extend(key_norm);
        let nr: Vec<f32> = out.no_read_mass.flatten_all()?.to_vec1()?;
        no_reads.extend(nr);
        let gate: Vec<f32> = out.copy_gate.flatten_all()?.to_vec1()?;
        gates.extend(gate);
    }
    let write = |name: &str, values: &[f32]| -> Result<(), Box<dyn Error>> {
        let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        fs::write(format!("{}.{name}.f32", args[4]), raw)?;
        Ok(())
    };
    write("key_norm", &norms)?;
    write("no_read", &no_reads)?;
    write("copy_gate", &gates)?;
    println!("positions {}", norms.len());
    Ok(())
}
