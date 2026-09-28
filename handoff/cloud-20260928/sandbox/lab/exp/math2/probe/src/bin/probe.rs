//! math2 scratch probe (not a project artifact).
//! For a saved JointModel, on consecutive held-out windows (fresh state per window, window = model context):
//!   OUT.logp.f32      log p(target) with the read enabled, positions t=0..T-2 of every window (target = token t+1)
//! and for the first DUMP windows (all positions t=0..T-1):
//!   OUT.query.f32 [n, r]  query = read.query(rms(provisional_t)), provisional rebuilt from the exposed states
//!   OUT.key.f32   [n, r]  key written at t = read.key(rms(state_t))
//!   OUT.null.f32  [n]     NoRead logit = read.no_read(rms(provisional_t))
//!   OUT.mass.f32  [n, T]  the model's own read masses (row t: candidates 0..t-1), used to verify the rebuild
//!   OUT.params.json       geometry, beta, offset, age table
//! usage: probe MODEL_DIR VALID.u16 WINDOWS DUMP OUT_PREFIX
use std::error::Error;
use std::fs;
use std::path::Path;

use candle_core::{Device, IndexOp, Tensor, D};
use uor_r4_training::joint_model::{
    transport_lanes, JointModel, ReadMode, LORENTZ_LOG_BETA, LORENTZ_OFFSET, RMS_EPSILON,
};

fn rms(x: &Tensor) -> candle_core::Result<Tensor> {
    let r = x.sqr()?.mean_keepdim(D::Minus1)?.affine(1.0, RMS_EPSILON)?.sqrt()?;
    x.broadcast_div(&r)
}

fn write(path: String, values: &[f32]) -> Result<(), Box<dyn Error>> {
    let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    fs::write(path, raw)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: probe MODEL_DIR VALID.u16 WINDOWS DUMP OUT_PREFIX".into());
    }
    let device = Device::Cpu;
    let model = JointModel::load(Path::new(&args[1]), &device)?;
    let bytes = fs::read(&args[2])?;
    let valid: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|c| u32::from(u16::from_le_bytes([c[0], c[1]])))
        .collect();
    let windows: usize = args[3].parse()?;
    let dump: usize = args[4].parse()?;
    let out = &args[5];
    let cfg = &model.config;
    let (t_len, d) = (cfg.context, cfg.width);
    let v = model.variables();
    let get = |n: &str| -> Result<Tensor, Box<dyn Error>> {
        Ok(v.get(n).ok_or(format!("missing {n}"))?.as_tensor().clone())
    };
    let emb = get("embedding.weight")?;
    let w_in = get("recurrent.input.weight")?;
    let w_s = get("recurrent.state.weight")?;
    let b_rec = get("recurrent.bias")?;
    let (wq, bq) = (get("read.query.weight")?, get("read.query.bias")?);
    let (wk, bk) = (get("read.key.weight")?, get("read.key.bias")?);
    let (wn, bn) = (get("read.no_read.weight")?, get("read.no_read.bias")?);
    let age = get("read.age")?.to_vec1::<f32>()?;
    let geometry = format!("{:?}", cfg.read_geometry);
    let (beta, offset) = if geometry == "Lorentz" {
        (
            get(LORENTZ_LOG_BETA)?.flatten_all()?.to_vec1::<f32>()?[0].exp(),
            get(LORENTZ_OFFSET)?.flatten_all()?.to_vec1::<f32>()?[0],
        )
    } else {
        (f32::NAN, f32::NAN)
    };
    let (mut logp, mut qs, mut ks, mut nulls, mut masses) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let starts: Vec<usize> = (0..windows).map(|w| w * t_len).collect();
    let mut done = 0usize;
    for group in starts.chunks(32) {
        let b = group.len();
        let mut ids = Vec::with_capacity(b * t_len);
        for &s in group {
            ids.extend_from_slice(&valid[s..s + t_len]);
        }
        let o = model.forward(&ids, b, t_len, ReadMode::Enabled, false)?;
        let probs = o.probabilities.to_vec3::<f32>()?;
        for lane in 0..b {
            for t in 0..t_len - 1 {
                let target = ids[lane * t_len + t + 1] as usize;
                logp.push(probs[lane][t][target].max(1e-30).ln());
            }
        }
        let n_dump = dump.saturating_sub(done).min(b);
        if n_dump > 0 {
            let states = o.states.i(0..n_dump)?.contiguous()?; // [b, T, d]
            let keys = rms(&states)?.broadcast_matmul(&wk.t()?)?.broadcast_add(&bk)?;
            // provisional_t from state_{t-1} (zeros at t=0) and token t
            let zeros = Tensor::zeros((n_dump, 1, d), candle_core::DType::F32, &device)?;
            let prev = Tensor::cat(&[&zeros, &states.i((.., 0..t_len - 1, ..))?], 1)?.contiguous()?;
            let tok = Tensor::from_vec(ids[..n_dump * t_len].to_vec(), (n_dump * t_len,), &device)?;
            let affine = emb.index_select(&tok, 0)?.matmul(&w_in.t()?)?; // [n*T, 3d]
            let prev2 = prev.reshape((n_dump * t_len, d))?;
            let fused = affine
                .add(&rms(&prev2)?.matmul(&w_s.t()?)?)?
                .broadcast_add(&b_rec)?;
            let cand = fused.narrow(1, 0, d)?.tanh()?;
            let z = candle_core::Tensor::ones_like(&fused.narrow(1, d, d)?)?
                .broadcast_div(&fused.narrow(1, d, d)?.neg()?.exp()?.affine(1.0, 1.0)?)?;
            let raw = fused.narrow(1, 2 * d, d)?.contiguous()?;
            let transported = transport_lanes(&prev2, &raw, cfg.transport)?;
            let prov = z.affine(-1.0, 1.0)?.mul(&transported)?.add(&z.mul(&cand)?)?;
            let pn = rms(&prov)?;
            let q = pn.matmul(&wq.t()?)?.broadcast_add(&bq)?;
            let nl = pn.matmul(&wn.t()?)?.broadcast_add(&bn)?;
            qs.extend(q.flatten_all()?.to_vec1::<f32>()?);
            nulls.extend(nl.flatten_all()?.to_vec1::<f32>()?);
            ks.extend(keys.flatten_all()?.to_vec1::<f32>()?);
            masses.extend(o.read_masses.i(0..n_dump)?.flatten_all()?.to_vec1::<f32>()?);
            done += n_dump;
        }
    }
    write(format!("{out}.logp.f32"), &logp)?;
    write(format!("{out}.query.f32"), &qs)?;
    write(format!("{out}.key.f32"), &ks)?;
    write(format!("{out}.null.f32"), &nulls)?;
    write(format!("{out}.mass.f32"), &masses)?;
    let params = serde_json::json!({"geometry": geometry, "beta": beta, "offset": offset, "age": age,
        "context": t_len, "width": d, "read_width": cfg.read_width, "windows": windows, "dump": done,
        "mean_nll": -logp.iter().map(|x| *x as f64).sum::<f64>() / logp.len() as f64});
    fs::write(format!("{out}.params.json"), serde_json::to_string(&params)?)?;
    println!("{geometry} windows {windows} dump {done} mean_nll {:.4}", params["mean_nll"]);
    Ok(())
}
