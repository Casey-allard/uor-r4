//! train2 scratch benchmark (not project code): SmolLM2-shaped throughput on
//! one CPU thread with the project's pinned Candle 0.9.2. Random weights; the
//! numbers are costs, not quality. RoPE is omitted (elementwise, small).
//! Modes:
//!   teacher : forward only, full stack + tied 49,152-way head + log-softmax
//!             (what logit KD needs from the teacher per token);
//!   xfer    : teacher forward (no head) + per-layer attention transfer: a
//!             Lorentz-score student attention (own Wq, Wk, beta, delta per
//!             layer; teacher V reused) trained by MSE to the teacher's
//!             per-head attention output, all layers, one backward per layer;
//!   student : full-model forward+backward+SGD with Lorentz attention
//!             (end-to-end KD cost minus the teacher forward).
use candle_core::{DType, Device, Result, Tensor, Var, D};
use std::time::Instant;

fn cpu_secs() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let rest = match s.rfind(')') {
        Some(i) => &s[i + 2..],
        None => return 0.0,
    };
    let f: Vec<&str> = rest.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

#[derive(Clone, Copy)]
struct Shape {
    v: usize,
    d: usize,
    l: usize,
    h: usize,
    kv: usize,
    ff: usize,
    b: usize,
    t: usize,
}

struct Layer {
    g1: Var,
    wq: Var,
    wk: Var,
    wv: Var,
    wo: Var,
    g2: Var,
    wg: Var,
    wu: Var,
    wd: Var,
    beta: Var,
    delta: Var,
}

fn mat(r: usize, c: usize, dev: &Device) -> Result<Var> {
    Var::from_tensor(&Tensor::randn(0f32, (1.0 / (r as f64).sqrt()) as f32, (r, c), dev)?)
}
fn fill(n: usize, x: f64, dev: &Device) -> Result<Var> {
    Var::from_tensor(&(Tensor::ones(n, DType::F32, dev)? * x)?)
}

fn rms(x: &Tensor, g: &Tensor) -> Result<Tensor> {
    let ms = x.sqr()?.mean_keepdim(D::Minus1)?;
    let inv = (ms + 1e-5)?.sqrt()?.recip()?;
    x.broadcast_mul(&inv)?.broadcast_mul(g)
}

fn t_(v: &Var, track: bool) -> Tensor {
    if track {
        v.as_tensor().clone()
    } else {
        v.as_tensor().detach()
    }
}

/// [b*t, d] @ w -> [b, nh, t, dh], KV heads expanded to s.h query heads.
fn heads(x: &Tensor, w: &Tensor, s: &Shape, nh: usize) -> Result<Tensor> {
    let dh = s.d / s.h;
    let y = x
        .matmul(w)?
        .reshape((s.b, s.t, nh, dh))?
        .transpose(1, 2)?
        .contiguous()?;
    if nh == s.h {
        return Ok(y);
    }
    let rep = s.h / nh;
    y.unsqueeze(2)?
        .broadcast_as((s.b, nh, rep, s.t, dh))?
        .contiguous()?
        .reshape((s.b, s.h, s.t, dh))
}

fn dot_scores(q: &Tensor, k: &Tensor, s: &Shape) -> Result<Tensor> {
    q.matmul(&k.t()?.contiguous()?)? * (1.0 / ((s.d / s.h) as f64).sqrt())
}

/// beta * (delta - arcosh(q0 k0 - <q,k>)), q0 = sqrt(1+|q|^2).
fn lorentz_scores(q: &Tensor, k: &Tensor, beta: &Tensor, delta: &Tensor) -> Result<Tensor> {
    let q0 = (q.sqr()?.sum_keepdim(D::Minus1)? + 1.0)?.sqrt()?; // [b,h,t,1]
    let k0 = (k.sqr()?.sum_keepdim(D::Minus1)? + 1.0)?.sqrt()?.transpose(2, 3)?; // [b,h,1,t]
    let z = q0.broadcast_mul(&k0)?.sub(&q.matmul(&k.t()?.contiguous()?)?)?;
    let u = ((z - 1.0)?.relu()? + 1e-6)?;
    let dist = ((u.clone() + 1.0)? + (u.clone() * (u + 2.0)?)?.sqrt()?)?.log()?;
    dist.broadcast_sub(delta)?.neg()?.broadcast_mul(beta)
}

fn softmax_mix(scores: &Tensor, mask: &Tensor, v: &Tensor) -> Result<Tensor> {
    let sc = scores.broadcast_add(mask)?;
    let m = sc.max_keepdim(D::Minus1)?.detach();
    let e = sc.broadcast_sub(&m)?.exp()?;
    let p = e.broadcast_div(&e.sum_keepdim(D::Minus1)?)?;
    p.matmul(v)
}

fn merge(o: &Tensor, s: &Shape) -> Result<Tensor> {
    o.transpose(1, 2)?.contiguous()?.reshape((s.b * s.t, s.d))
}

fn mlp(x: &Tensor, ly: &Layer, s: &Shape, track: bool) -> Result<Tensor> {
    let xn = rms(x, &t_(&ly.g2, track))?.reshape((s.b * s.t, s.d))?;
    let g = xn.matmul(&t_(&ly.wg, track))?.silu()?;
    let u = xn.matmul(&t_(&ly.wu, track))?;
    (g * u)?.matmul(&t_(&ly.wd, track))?.reshape((s.b, s.t, s.d))
}

fn main() -> Result<()> {
    // TRAIN2_DEVICE=metal selects the Apple GPU (build with --features metal);
    // anything else uses the CPU (build with --features accelerate on macOS).
    let dev = match std::env::var("TRAIN2_DEVICE").as_deref() {
        Ok("metal") => Device::new_metal(0)?,
        _ => Device::Cpu,
    };
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).cloned().unwrap_or_else(|| "teacher".into());
    let num = |i: usize, dflt: usize| -> usize { args.get(i).and_then(|x| x.parse().ok()).unwrap_or(dflt) };
    // Default: SmolLM2-135M shape (d 576, 30 layers, 9 heads, 3 KV heads, ff 1536, V 49152).
    let s = Shape {
        v: num(2, 49152),
        d: num(3, 576),
        l: num(4, 30),
        h: num(5, 9),
        kv: num(6, 3),
        ff: num(7, 1536),
        b: num(8, 4),
        t: num(9, 256),
    };
    let steps = num(10, 2);
    let dh = s.d / s.h;
    let emb = mat(s.v, s.d, &dev)?;
    let gf = fill(s.d, 1.0, &dev)?;
    let mut layers = Vec::new();
    for _ in 0..s.l {
        layers.push(Layer {
            g1: fill(s.d, 1.0, &dev)?,
            wq: mat(s.d, s.d, &dev)?,
            wk: mat(s.d, s.kv * dh, &dev)?,
            wv: mat(s.d, s.kv * dh, &dev)?,
            wo: mat(s.d, s.d, &dev)?,
            g2: fill(s.d, 1.0, &dev)?,
            wg: mat(s.d, s.ff, &dev)?,
            wu: mat(s.d, s.ff, &dev)?,
            wd: mat(s.ff, s.d, &dev)?,
            beta: fill(1, 3.0, &dev)?,
            delta: fill(1, 5.0, &dev)?,
        });
    }
    let mut n_params = emb.elem_count() + s.d;
    for ly in &layers {
        n_params += [&ly.g1, &ly.wq, &ly.wk, &ly.wv, &ly.wo, &ly.g2, &ly.wg, &ly.wu, &ly.wd]
            .iter()
            .map(|x| x.elem_count())
            .sum::<usize>();
    }
    let mut maskv = vec![0f32; s.t * s.t];
    for i in 0..s.t {
        for j in (i + 1)..s.t {
            maskv[i * s.t + j] = -1e9;
        }
    }
    let mask = Tensor::from_vec(maskv, (s.t, s.t), &dev)?;
    let mut st: u64 = 0x9E3779B97F4A7C15;
    let mut draw = |n: usize| -> Vec<u32> {
        (0..n)
            .map(|_| {
                st ^= st << 13;
                st ^= st >> 7;
                st ^= st << 17;
                (st % s.v as u64) as u32
            })
            .collect()
    };
    let lr = 1e-4;
    let mut times = Vec::new();
    let mut last = 0f32;
    for step in 0..(steps + 1) {
        let ids = Tensor::from_vec(draw(s.b * s.t), (s.b * s.t,), &dev)?;
        let tgt = Tensor::from_vec(draw(s.b * s.t), (s.b * s.t, 1), &dev)?;
        let (w0, c0) = (Instant::now(), cpu_secs());
        match mode.as_str() {
            "teacher" | "xfer" => {
                let e = emb.as_tensor().detach();
                let mut x = e.index_select(&ids, 0)?.reshape((s.b, s.t, s.d))?;
                let mut saved = Vec::new();
                for ly in &layers {
                    let xn = rms(&x, &t_(&ly.g1, false))?.reshape((s.b * s.t, s.d))?;
                    let q = heads(&xn, &t_(&ly.wq, false), &s, s.h)?;
                    let k = heads(&xn, &t_(&ly.wk, false), &s, s.kv)?;
                    let v = heads(&xn, &t_(&ly.wv, false), &s, s.kv)?;
                    let o = softmax_mix(&dot_scores(&q, &k, &s)?, &mask, &v)?;
                    let a = merge(&o, &s)?.matmul(&t_(&ly.wo, false))?.reshape((s.b, s.t, s.d))?;
                    if mode == "xfer" {
                        saved.push((xn, v, o));
                    }
                    x = (x + a)?;
                    x = (x.clone() + mlp(&x, ly, &s, false)?)?;
                }
                if mode == "teacher" {
                    let xn = rms(&x, &t_(&gf, false))?.reshape((s.b * s.t, s.d))?;
                    let logits = xn.matmul(&e.t()?)?;
                    let m = logits.max_keepdim(1)?;
                    let z = logits.broadcast_sub(&m)?;
                    let logp = z.broadcast_sub(&z.exp()?.sum_keepdim(1)?.log()?)?;
                    last = logp.gather(&tgt, 1)?.mean_all()?.to_scalar::<f32>()?;
                } else {
                    let mut tot = 0f32;
                    for (ly, (xn, v, o_t)) in layers.iter().zip(saved.iter()) {
                        let q = heads(xn, ly.wq.as_tensor(), &s, s.h)?;
                        let k = heads(xn, ly.wk.as_tensor(), &s, s.kv)?;
                        let sc = lorentz_scores(&q, &k, ly.beta.as_tensor(), ly.delta.as_tensor())?;
                        let o_s = softmax_mix(&sc, &mask, v)?;
                        let loss = (o_s - o_t)?.sqr()?.mean_all()?;
                        let grads = loss.backward()?;
                        for var in [&ly.wq, &ly.wk, &ly.beta, &ly.delta] {
                            if let Some(g) = grads.get(var.as_tensor()) {
                                var.set(&(var.as_tensor() - (g * lr)?)?)?;
                            }
                        }
                        tot += loss.to_scalar::<f32>()?;
                    }
                    last = tot / s.l as f32;
                }
            }
            _ => {
                // student: full forward + backward + SGD, Lorentz attention.
                let mut x = emb.as_tensor().index_select(&ids, 0)?.reshape((s.b, s.t, s.d))?;
                for ly in &layers {
                    let xn = rms(&x, ly.g1.as_tensor())?.reshape((s.b * s.t, s.d))?;
                    let q = heads(&xn, ly.wq.as_tensor(), &s, s.h)?;
                    let k = heads(&xn, ly.wk.as_tensor(), &s, s.kv)?;
                    let v = heads(&xn, ly.wv.as_tensor(), &s, s.kv)?;
                    let sc = lorentz_scores(&q, &k, ly.beta.as_tensor(), ly.delta.as_tensor())?;
                    let o = softmax_mix(&sc, &mask, &v)?;
                    let a = merge(&o, &s)?.matmul(ly.wo.as_tensor())?.reshape((s.b, s.t, s.d))?;
                    x = (x + a)?;
                    x = (x.clone() + mlp(&x, ly, &s, true)?)?;
                }
                let xn = rms(&x, gf.as_tensor())?.reshape((s.b * s.t, s.d))?;
                let logits = xn.matmul(&emb.as_tensor().t()?)?;
                let m = logits.max_keepdim(1)?.detach();
                let z = logits.broadcast_sub(&m)?;
                let logp = z.broadcast_sub(&z.exp()?.sum_keepdim(1)?.log()?)?;
                let loss = logp.gather(&tgt, 1)?.neg()?.mean_all()?;
                let grads = loss.backward()?;
                let mut vars: Vec<&Var> = vec![&emb, &gf];
                for ly in &layers {
                    vars.extend([
                        &ly.g1, &ly.wq, &ly.wk, &ly.wv, &ly.wo, &ly.g2, &ly.wg, &ly.wu, &ly.wd,
                        &ly.beta, &ly.delta,
                    ]);
                }
                for var in vars {
                    if let Some(g) = grads.get(var.as_tensor()) {
                        var.set(&(var.as_tensor() - (g * lr)?)?)?;
                    }
                }
                last = loss.to_scalar::<f32>()?;
            }
        }
        let (wall, cpu) = (w0.elapsed().as_secs_f64(), cpu_secs() - c0);
        eprintln!("  step {step}: wall {wall:.2}s cpu {cpu:.2}s value {last:.4}");
        if step > 0 {
            times.push((wall, cpu));
        }
    }
    let n = times.len() as f64;
    let wall = times.iter().map(|x| x.0).sum::<f64>() / n;
    let cpu = times.iter().map(|x| x.1).sum::<f64>() / n;
    let tok = (s.b * s.t) as f64;
    println!(
        "{mode}: params={n_params} V={} d={} L={} h={} kv={} ff={} B={} T={} | step wall {wall:.2}s cpu {cpu:.2}s | tokens/s wall {:.1} cpu {:.1}",
        s.v, s.d, s.l, s.h, s.kv, s.ff, s.b, s.t, tok / wall, tok / cpu
    );
    Ok(())
}
