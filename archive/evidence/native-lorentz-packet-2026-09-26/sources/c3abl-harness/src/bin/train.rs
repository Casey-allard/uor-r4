//! Scratch harness: train the repo's D8 JointModel on local BPE tokens and
//! compare read geometries at equal budget. Constant AdamW learning rate, as in
//! the project's campaigns. Evaluation uses fixed, evenly spaced validation
//! windows; every window starts from a fresh state (as the D8 evaluator does).
//!
//! usage: train key=value ...
//!   data=DIR (train.u16, valid.u16, lens.u16)  out=FILE.json
//!   geometry=dot|lorentz  seed=N  width=128|256  context=T  batch=B
//!   steps=N  lr=F  eval_every=N  eval_windows=N  final_windows=N  shards=1|2|4
//!   transport=quaternion|householder_pair

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use uor_r4_training::joint_model::{JointConfig, JointModel, ReadGeometry, ReadMode, Transport};
use uor_r4_training::joint_optimizer::{AdamConfig, NamedAdamW};
use uor_r4_training::joint_parallel::batch_gradients;

type Res<T> = Result<T, Box<dyn Error>>;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn read_u16(path: PathBuf) -> Res<Vec<u32>> {
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(bytes
        .chunks_exact(2)
        .map(|c| u32::from(u16::from_le_bytes([c[0], c[1]])))
        .collect())
}

struct Args(BTreeMap<String, String>);
impl Args {
    fn get(&self, key: &str, default: &str) -> String {
        self.0.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
    fn num<T: std::str::FromStr>(&self, key: &str, default: &str) -> Res<T> {
        self.get(key, default)
            .parse::<T>()
            .map_err(|_| format!("bad value for {key}").into())
    }
}

struct Eval {
    nll_read: f64,
    nll_no_read: f64,
    bpb_read: f64,
    bpb_no_read: f64,
    no_read_mass: f64,
    targets: usize,
}

/// Mean NLL (nats/token) and bits/byte over fixed evenly spaced windows.
fn evaluate(model: &JointModel, valid: &[u32], lens: &[u32], time: usize, windows: usize) -> Res<Eval> {
    let span = valid.len() - time - 1;
    let stride = span / windows.max(1);
    let starts: Vec<usize> = (0..windows).map(|w| w * stride).collect();
    let mut sums = [0f64; 2];
    let mut mass_sum = 0f64;
    let mut bytes = 0f64;
    let mut count = 0usize;
    for group in starts.chunks(32) {
        let b = group.len();
        let mut ids = Vec::with_capacity(b * time);
        let mut targets = Vec::with_capacity(b * time);
        for &s in group {
            ids.extend_from_slice(&valid[s..s + time]);
            targets.extend_from_slice(&valid[s + 1..s + time + 1]);
        }
        bytes += targets.iter().map(|&t| f64::from(lens[t as usize])).sum::<f64>();
        count += targets.len();
        let index = Tensor::from_vec(targets.clone(), (b, time, 1), model.device())?;
        for (slot, mode) in [ReadMode::Enabled, ReadMode::NoRead].into_iter().enumerate() {
            let out = model.forward(&ids, b, time, mode, false)?;
            let picked = out.probabilities.gather(&index, 2)?.squeeze(2)?.to_vec2::<f32>()?;
            sums[slot] += picked
                .iter()
                .flatten()
                .map(|&p| -f64::from(p).ln())
                .sum::<f64>();
            if slot == 0 {
                mass_sum += out
                    .no_read_mass
                    .to_vec2::<f32>()?
                    .iter()
                    .flatten()
                    .map(|&m| f64::from(m))
                    .sum::<f64>();
            }
        }
    }
    let ln2 = std::f64::consts::LN_2;
    Ok(Eval {
        nll_read: sums[0] / count as f64,
        nll_no_read: sums[1] / count as f64,
        bpb_read: sums[0] / ln2 / bytes,
        bpb_no_read: sums[1] / ln2 / bytes,
        no_read_mass: mass_sum / count as f64,
        targets: count,
    })
}

fn lorentz_scalars(model: &JointModel) -> Res<Value> {
    let mut out = serde_json::Map::new();
    for (name, var) in model.variables() {
        if name.contains("lorentz") {
            let v = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
            out.insert(name.clone(), json!(v));
        }
    }
    Ok(Value::Object(out))
}

fn eval_json(e: &Eval) -> Value {
    json!({"nll_read":e.nll_read,"nll_no_read":e.nll_no_read,"bpb_read":e.bpb_read,
           "bpb_no_read":e.bpb_no_read,"read_effect_nats":e.nll_no_read-e.nll_read,
           "no_read_mass":e.no_read_mass,"targets":e.targets})
}

fn main() -> Res<()> {
    let mut map = BTreeMap::new();
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or("arguments are key=value")?;
        map.insert(k.to_string(), v.to_string());
    }
    let args = Args(map);
    let data = PathBuf::from(args.get("data", ""));
    let out_path = PathBuf::from(args.get("out", ""));
    if out_path.as_os_str().is_empty() || out_path.exists() {
        return Err("out= must name a new file".into());
    }
    let geometry = match args.get("geometry", "dot").as_str() {
        "dot" => ReadGeometry::Dot,
        "lorentz" => ReadGeometry::Lorentz,
        other => return Err(format!("unknown geometry {other}").into()),
    };
    let transport = match args.get("transport", "quaternion").as_str() {
        "quaternion" => Transport::Quaternion,
        "householder_pair" => Transport::HouseholderPair,
        other => return Err(format!("unknown transport {other}").into()),
    };
    let seed: u64 = args.num("seed", "1")?;
    let width: usize = args.num("width", "128")?;
    let time: usize = args.num("context", "128")?;
    let batch: usize = args.num("batch", "16")?;
    let steps: usize = args.num("steps", "1000")?;
    let lr: f64 = args.num("lr", "0.001")?;
    let eval_every: usize = args.num("eval_every", "250")?;
    let eval_windows: usize = args.num("eval_windows", "64")?;
    let final_windows: usize = args.num("final_windows", "256")?;
    let shards: usize = args.num("shards", "1")?;
    let max_seconds: f64 = args.num("max_seconds", "1e12")?;

    let train = read_u16(data.join("train.u16"))?;
    let valid = read_u16(data.join("valid.u16"))?;
    let lens = read_u16(data.join("lens.u16"))?;
    let config = JointConfig {
        vocab_size: 4096,
        width,
        read_width: 64,
        context: time,
        transport,
        seed,
        read_geometry: geometry,
    };
    let device = Device::Cpu;
    let model = JointModel::new(config.clone(), &device)?;
    let mut optimizer = NamedAdamW::new(
        model.variables(),
        AdamConfig {
            learning_rate: lr,
            ..AdamConfig::default()
        },
    )?;
    let started = Instant::now();
    let mut curve = Vec::new();
    let initial = evaluate(&model, &valid, &lens, time, eval_windows)?;
    curve.push(json!({"step":0,"eval":eval_json(&initial),"lorentz":lorentz_scalars(&model)?}));
    eprintln!(
        "step 0 nll_read {:.4} nll_noread {:.4} noread_mass {:.3}",
        initial.nll_read, initial.nll_no_read, initial.no_read_mass
    );
    let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ 0xC3);
    let mut running = 0f64;
    let mut running_n = 0usize;
    let mut train_seconds = 0f64;
    let mut completed = 0usize;
    for step in 0..steps {
        if started.elapsed().as_secs_f64() > max_seconds {
            break;
        }
        let t0 = Instant::now();
        let mut inputs = Vec::with_capacity(batch * time);
        let mut targets = Vec::with_capacity(batch * time);
        for _ in 0..batch {
            let s = (rng.next() % (train.len() - time - 1) as u64) as usize;
            inputs.extend_from_slice(&train[s..s + time]);
            targets.extend_from_slice(&train[s + 1..s + time + 1]);
        }
        let grads = batch_gradients(&model, &inputs, &targets, batch, time, shards)?;
        let report = optimizer.step(model.variables(), &grads.gradients)?;
        train_seconds += t0.elapsed().as_secs_f64();
        completed = step + 1;
        running += f64::from(grads.mean_nll);
        running_n += 1;
        if completed % eval_every == 0 && completed != steps {
            let e = evaluate(&model, &valid, &lens, time, eval_windows)?;
            eprintln!(
                "step {completed} train {:.4} nll_read {:.4} nll_noread {:.4} noread_mass {:.3} grad {:.3} {:.2}s/step",
                running / running_n as f64,
                e.nll_read,
                e.nll_no_read,
                e.no_read_mass,
                report.global_grad_norm,
                train_seconds / completed as f64
            );
            curve.push(json!({"step":completed,"train_nll":running / running_n as f64,
                "eval":eval_json(&e),"grad_norm":report.global_grad_norm,
                "lorentz":lorentz_scalars(&model)?}));
            running = 0.0;
            running_n = 0;
        }
    }
    let final_eval = evaluate(&model, &valid, &lens, time, final_windows)?;
    eprintln!(
        "final step {completed} nll_read {:.4} bpb_read {:.4} nll_noread {:.4} noread_mass {:.3}",
        final_eval.nll_read, final_eval.bpb_read, final_eval.nll_no_read, final_eval.no_read_mass
    );
    let report = json!({
        "config": config,
        "geometry": geometry.name(),
        "seed": seed, "batch": batch, "context": time, "steps_requested": steps,
        "steps_completed": completed, "lr": lr, "shards": shards,
        "parameters": model.parameter_count(),
        "train_tokens": train.len(), "valid_tokens": valid.len(),
        "train_seconds": train_seconds, "wall_seconds": started.elapsed().as_secs_f64(),
        "final": eval_json(&final_eval), "final_windows": final_windows,
        "lorentz": lorentz_scalars(&model)?,
        "curve": curve,
    });
    fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;
    Ok(())
}
