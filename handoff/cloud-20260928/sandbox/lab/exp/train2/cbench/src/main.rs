//! train2 scratch benchmark (not project code).
//! Measures, on one CPU thread with the project's pinned Candle 0.9.2:
//!   mm  : f32 matmul throughput at shapes typical of a sequential recurrent
//!         step (16 rows) versus a time-parallel layer (4096 rows);
//!   lm  : forward+backward+SGD of a small causal transformer LM whose whole
//!         window is processed in parallel (a proxy for any time-parallel
//!         trainable architecture, e.g. a scan-trained linear recurrence).
//! Reports wall and process CPU time (utime+stime) so contention is visible.
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

fn bench_matmul(dev: &Device, m: usize, k: usize, n: usize) -> Result<()> {
    let a = Tensor::randn(0f32, 1f32, (m, k), dev)?;
    let b = Tensor::randn(0f32, 1f32, (k, n), dev)?;
    let per = 2.0 * (m * k * n) as f64;
    let reps = ((30e9 / per) as usize).max(3);
    let _ = a.matmul(&b)?;
    let (w0, c0) = (Instant::now(), cpu_secs());
    for _ in 0..reps {
        let c = a.matmul(&b)?;
        std::hint::black_box(&c);
    }
    let wall = w0.elapsed().as_secs_f64();
    let cpu = (cpu_secs() - c0).max(1e-9);
    let fl = per * reps as f64;
    println!(
        "mm {m}x{k}x{n}: {:.1} GFLOP/s (wall) {:.1} GFLOP/s (cpu) reps={reps} wall={wall:.2}s cpu={cpu:.2}s",
        fl / wall / 1e9,
        fl / cpu / 1e9
    );
    Ok(())
}

struct Layer {
    g1: Var,
    wq: Var,
    wk: Var,
    wv: Var,
    wo: Var,
    g2: Var,
    w1: Var,
    w2: Var,
}

fn mat(r: usize, c: usize, dev: &Device) -> Result<Var> {
    Var::from_tensor(&Tensor::randn(0f32, (1.0 / (r as f64).sqrt()) as f32, (r, c), dev)?)
}
fn ones(n: usize, dev: &Device) -> Result<Var> {
    Var::from_tensor(&Tensor::ones(n, DType::F32, dev)?)
}

fn rms(x: &Tensor, g: &Tensor) -> Result<Tensor> {
    let ms = x.sqr()?.mean_keepdim(D::Minus1)?;
    let inv = (ms + 1e-5)?.sqrt()?.recip()?;
    x.broadcast_mul(&inv)?.broadcast_mul(g)
}

#[allow(clippy::too_many_arguments)]
fn loss_fn(
    (b, t, d, h): (usize, usize, usize, usize),
    emb: &Var,
    pos: &Var,
    layers: &[Layer],
    gf: &Var,
    ids: &Tensor,
    tgt: &Tensor,
    mask: &Tensor,
) -> Result<Tensor> {
    let dh = d / h;
    let x = emb
        .as_tensor()
        .index_select(&ids.flatten_all()?, 0)?
        .reshape((b, t, d))?;
    let mut x = x.broadcast_add(pos.as_tensor())?;
    for ly in layers {
        let xn = rms(&x, ly.g1.as_tensor())?.reshape((b * t, d))?;
        let heads = |w: &Var| -> Result<Tensor> {
            xn.matmul(w.as_tensor())?
                .reshape((b, t, h, dh))?
                .transpose(1, 2)?
                .contiguous()
        };
        let (q, k, v) = (heads(&ly.wq)?, heads(&ly.wk)?, heads(&ly.wv)?);
        let s = (q.matmul(&k.t()?.contiguous()?)? * (1.0 / (dh as f64).sqrt()))?;
        let s = s.broadcast_add(mask)?;
        let m = s.max_keepdim(D::Minus1)?.detach();
        let e = s.broadcast_sub(&m)?.exp()?;
        let p = e.broadcast_div(&e.sum_keepdim(D::Minus1)?)?;
        let o = p
            .matmul(&v)?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((b * t, d))?;
        let o = o.matmul(ly.wo.as_tensor())?.reshape((b, t, d))?;
        x = (x + o)?;
        let xn = rms(&x, ly.g2.as_tensor())?.reshape((b * t, d))?;
        let f = xn
            .matmul(ly.w1.as_tensor())?
            .silu()?
            .matmul(ly.w2.as_tensor())?
            .reshape((b, t, d))?;
        x = (x + f)?;
    }
    let xn = rms(&x, gf.as_tensor())?.reshape((b * t, d))?;
    let logits = xn.matmul(&emb.as_tensor().t()?)?;
    let m = logits.max_keepdim(1)?.detach();
    let z = logits.broadcast_sub(&m)?;
    let lse = z.exp()?.sum_keepdim(1)?.log()?;
    let logp = z.broadcast_sub(&lse)?;
    logp.gather(&tgt.reshape((b * t, 1))?, 1)?.neg()?.mean_all()
}

fn bench_lm(dev: &Device, v: usize, d: usize, l: usize, h: usize, t: usize, b: usize, steps: usize) -> Result<()> {
    let ff = 4 * d;
    let emb = mat(v, d, dev)?;
    let pos = mat(t, d, dev)?;
    let gf = ones(d, dev)?;
    let mut layers = Vec::new();
    for _ in 0..l {
        layers.push(Layer {
            g1: ones(d, dev)?,
            wq: mat(d, d, dev)?,
            wk: mat(d, d, dev)?,
            wv: mat(d, d, dev)?,
            wo: mat(d, d, dev)?,
            g2: ones(d, dev)?,
            w1: mat(d, ff, dev)?,
            w2: mat(ff, d, dev)?,
        });
    }
    let mut vars: Vec<&Var> = vec![&emb, &pos, &gf];
    for ly in &layers {
        vars.extend([&ly.g1, &ly.wq, &ly.wk, &ly.wv, &ly.wo, &ly.g2, &ly.w1, &ly.w2]);
    }
    let params: usize = vars.iter().map(|x| x.elem_count()).sum();
    let mut maskv = vec![0f32; t * t];
    for i in 0..t {
        for j in (i + 1)..t {
            maskv[i * t + j] = -1e9;
        }
    }
    let mask = Tensor::from_vec(maskv, (t, t), dev)?;
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut draw = |n: usize| -> Vec<u32> {
        (0..n)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state % v as u64) as u32
            })
            .collect()
    };
    let lr = 1e-3;
    let mut times = Vec::new();
    let mut last = 0f32;
    for step in 0..(steps + 1) {
        let ids = Tensor::from_vec(draw(b * t), (b, t), dev)?;
        let tgt = Tensor::from_vec(draw(b * t), (b, t), dev)?;
        let (w0, c0) = (Instant::now(), cpu_secs());
        let loss = loss_fn((b, t, d, h), &emb, &pos, &layers, &gf, &ids, &tgt, &mask)?;
        let grads = loss.backward()?;
        for var in &vars {
            if let Some(g) = grads.get(var.as_tensor()) {
                var.set(&(var.as_tensor() - (g * lr)?)?)?;
            }
        }
        last = loss.to_scalar::<f32>()?;
        if step > 0 {
            times.push((w0.elapsed().as_secs_f64(), cpu_secs() - c0));
        }
    }
    let n = times.len() as f64;
    let wall = times.iter().map(|x| x.0).sum::<f64>() / n;
    let cpu = times.iter().map(|x| x.1).sum::<f64>() / n;
    let tok = (b * t) as f64;
    let n_mm = l * (4 * d * d + 2 * d * ff) + v * d;
    let fl_tok = 6.0 * n_mm as f64 + (l * 12 * t * d) as f64;
    println!(
        "lm V={v} d={d} L={l} T={t} B={b}: params={params} step wall={wall:.2}s cpu={cpu:.2}s tokens/s(wall)={:.0} tokens/s(cpu)={:.0} ~{:.1} MFLOP/token -> {:.1} GFLOP/s(cpu) loss={last:.3}",
        tok / wall,
        tok / cpu,
        fl_tok / 1e6,
        fl_tok * tok / cpu / 1e9
    );
    Ok(())
}

fn main() -> Result<()> {
    let dev = Device::Cpu;
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("mm");
    let num = |i: usize, dflt: usize| -> usize {
        args.get(i).and_then(|s| s.parse().ok()).unwrap_or(dflt)
    };
    match mode {
        "mm" => {
            for &(m, k, n) in &[
                (16, 256, 768),
                (16, 256, 4096),
                (4096, 256, 768),
                (4096, 256, 4096),
                (4096, 512, 2048),
                (4096, 1024, 4096),
            ] {
                bench_matmul(&dev, m, k, n)?;
            }
        }
        _ => bench_lm(
            &dev,
            num(2, 4096),
            num(3, 256),
            num(4, 4),
            num(5, 8),
            num(6, 256),
            num(7, 8),
            num(8, 3),
        )?,
    }
    Ok(())
}
