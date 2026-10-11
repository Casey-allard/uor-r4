//! Post-hoc icosian snap of a `quaternion_weights=all` geometric stack: a
//! probe of the multiplier-free rung, not a trained-in test.
//!
//! ```text
//! quaternion-snap model=ROOT/model out=NEW_DIR
//! ```
//!
//! Every quaternion of every compact map (`*.quaternion`, `[out/4, in/4, 4]`)
//! is replaced by its norm times the nearest of the 120 unit icosians of 2I
//! (`canonical_h4_roots`, by largest dot product with the unit direction). A
//! zero quaternion stays zero. Every other tensor and `config.json` are copied
//! unchanged, so `geometric-stack evaluate model=NEW_DIR` scores the snapped
//! model on the same blocks as the trained one. `snap.json` records, per map
//! and in total, the mean cosine between each quaternion and its snapped
//! direction.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use candle_core::{Device, Tensor};
use serde_json::json;
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;

fn main() {
    if let Err(error) = run() {
        eprintln!("quaternion-snap: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut model = None;
    let mut out = None;
    for argument in std::env::args().skip(1) {
        match argument.split_once('=') {
            Some(("model", value)) => model = Some(PathBuf::from(value)),
            Some(("out", value)) => out = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument {argument} (model=DIR out=DIR)")),
        }
    }
    let model = model.ok_or("model= is required")?;
    let out = out.ok_or("out= is required")?;
    if out.exists() {
        return Err(format!("{} already exists", out.display()));
    }
    let roots: Vec<[f32; 4]> = canonical_h4_roots()
        .iter()
        .map(|root| {
            let a = root.to_array();
            [a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32]
        })
        .collect();
    let tensors = candle_core::safetensors::load(model.join("model.safetensors"), &Device::Cpu)
        .map_err(|e| e.to_string())?;
    let mut snapped = HashMap::new();
    let mut per_map = BTreeMap::new();
    let (mut total_cos, mut total_count) = (0f64, 0usize);
    for (name, tensor) in tensors {
        if !name.ends_with(".quaternion") {
            snapped.insert(name, tensor);
            continue;
        }
        let shape = tensor.dims().to_vec();
        if shape.last() != Some(&4) {
            return Err(format!("{name} is not [.., 4]"));
        }
        let values = tensor
            .flatten_all()
            .and_then(|t| t.to_vec1::<f32>())
            .map_err(|e| e.to_string())?;
        let mut output = Vec::with_capacity(values.len());
        let (mut cos_sum, mut count) = (0f64, 0usize);
        for q in values.chunks_exact(4) {
            let norm = q.iter().map(|v| v * v).sum::<f32>().sqrt();
            if norm == 0.0 {
                output.extend_from_slice(q);
                continue;
            }
            let (mut best, mut best_dot) = (0usize, f32::NEG_INFINITY);
            for (index, root) in roots.iter().enumerate() {
                let dot = (0..4).map(|i| q[i] * root[i]).sum::<f32>() / norm;
                if dot > best_dot {
                    (best, best_dot) = (index, dot);
                }
            }
            output.extend(roots[best].iter().map(|r| r * norm));
            cos_sum += f64::from(best_dot);
            count += 1;
        }
        per_map.insert(
            name.clone(),
            json!({"quaternions": count, "mean_cosine": cos_sum / count.max(1) as f64}),
        );
        total_cos += cos_sum;
        total_count += count;
        let tensor = Tensor::from_vec(output, shape, &Device::Cpu).map_err(|e| e.to_string())?;
        snapped.insert(name, tensor);
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    candle_core::safetensors::save(&snapped, out.join("model.safetensors"))
        .map_err(|e| e.to_string())?;
    fs::copy(model.join("config.json"), out.join("config.json")).map_err(|e| e.to_string())?;
    let record = json!({
        "schema": "uor-r4.quaternion-snap/1",
        "source": model.display().to_string(),
        "roots": roots.len(),
        "quaternions": total_count,
        "mean_cosine": total_cos / total_count.max(1) as f64,
        "maps": per_map,
    });
    fs::write(
        out.join("snap.json"),
        serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "snapped {total_count} quaternions, mean cosine {:.4}",
        total_cos / total_count.max(1) as f64
    );
    Ok(())
}
