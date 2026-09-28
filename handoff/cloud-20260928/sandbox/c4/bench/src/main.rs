use std::time::Instant;
use candle_core::{Device, Tensor, DType};
use uor_r4_training::geometric_stack::{StackConfig, StackModel, ReadScore};

fn time<F: FnMut() -> candle_core::Result<Tensor>>(label: &str, reps: usize, mut f: F) -> f64 {
    let _ = f().unwrap();
    let start = Instant::now();
    for _ in 0..reps { let _ = f().unwrap(); }
    let s = start.elapsed().as_secs_f64() / reps as f64;
    println!("{label:40} {:8.2} ms", s * 1e3);
    s
}

fn main() {
    let dev = Device::Cpu;
    let a = Tensor::randn(0f32, 1.0, (4096, 288), &dev).unwrap();
    let w = Tensor::randn(0f32, 1.0, (768, 288), &dev).unwrap();
    let wt = w.t().unwrap();
    let s = time("matmul 4096x288 @ 288x768 (w.t view)", 10, || a.matmul(&wt));
    println!("   -> {:.1} GFLOP/s", 2.0 * 4096.0 * 288.0 * 768.0 / s / 1e9);
    let wtc = w.t().unwrap().contiguous().unwrap();
    let s = time("matmul 4096x288 @ 288x768 (contiguous)", 10, || a.matmul(&wtc));
    println!("   -> {:.1} GFLOP/s", 2.0 * 4096.0 * 288.0 * 768.0 / s / 1e9);
    let e = Tensor::randn(0f32, 1.0, (4096, 4096), &dev).unwrap();
    let out = Tensor::randn(0f32, 1.0, (4096, 288), &dev).unwrap();
    let s = time("matmul 4096x288 @ 288x4096 (emb.t)", 5, || out.matmul(&e.narrow(1,0,288).unwrap().t().unwrap()));
    println!("   -> {:.1} GFLOP/s", 2.0 * 4096.0 * 288.0 * 4096.0 / s / 1e9);
    let big = Tensor::randn(0f32, 1.0, (4096, 768), &dev).unwrap();
    time("silu 4096x768", 10, || big.silu());
    time("mul 4096x768", 10, || big.mul(&big));
    time("sqr+mean_keepdim 4096x288", 10, || a.sqr()?.mean_keepdim(1));
    let q = Tensor::randn(0f32, 1.0, (16, 6, 256, 48), &dev).unwrap();
    let k = Tensor::randn(0f32, 1.0, (16, 6, 256, 48), &dev).unwrap();
    let s = time("batched QK^T 16x6x256x48", 10, || q.matmul(&k.transpose(2,3)?.contiguous()?));
    println!("   -> {:.1} GFLOP/s", 2.0 * 96.0 * 256.0 * 48.0 * 256.0 / s / 1e9);
    time("transpose(1,2).contiguous 16x256x6x48", 10, || q.transpose(1,2)?.contiguous());

    for (name, config) in [
        ("transformer", StackConfig::transformer_control(1)),
        ("geometric", StackConfig::geometric_matched("rrarra", ReadScore::Lorentz, true, 1).unwrap()),
    ] {
        let model = StackModel::new(config, &dev).unwrap();
        let ids: Vec<u32> = (0..16 * 256).map(|i| (i * 7919 % 4096) as u32).collect();
        let targets: Vec<u32> = (0..16 * 256).map(|i| (i * 104729 % 4096) as u32).collect();
        let start = Instant::now();
        let logits = model.forward(&ids, 16, 256).unwrap();
        let f = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let loss = model.loss(&ids, &targets, 16, 256).unwrap();
        let fl = start.elapsed().as_secs_f64();
        let start = Instant::now();
        let grads = loss.backward().unwrap();
        let b = start.elapsed().as_secs_f64();
        let _ = (logits, grads);
        println!("{name}: forward {:.0} ms, forward+loss {:.0} ms, backward {:.0} ms", f * 1e3, fl * 1e3, b * 1e3);
    }
    let _ = DType::F32;
}
