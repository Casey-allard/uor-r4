//! Numerical Parity Tests: CPU Reference vs Native Metal Stack Kernels.
//!
//! Tests the stack operations in `geometric_stack.rs` for bitwise / high-precision numerical
//! equivalence between CPU execution and Apple Silicon Metal GPU execution.

#[cfg(feature = "metal")]
#[test]
fn test_metal_device_available() -> uor_r4_training::Result<()> {
    match candle_core::Device::new_metal(0) {
        Ok(device) => {
            println!("Metal device 0 initialized successfully: {:?}", device);
            Ok(())
        }
        Err(e) => {
            println!("Metal device unavailable (skipping GPU tests): {e}");
            Ok(())
        }
    }
}

#[cfg(feature = "metal")]
fn assert_finite_and_close(cpu: &[f32], metal: &[f32], tol: f32, op_name: &str) -> f32 {
    assert_eq!(cpu.len(), metal.len(), "{op_name}: length mismatch");
    let mut max_diff = 0.0f32;
    for (i, (&c, &m)) in cpu.iter().zip(metal.iter()).enumerate() {
        assert!(
            c.is_finite(),
            "{op_name} at index {i}: CPU value is not finite ({c})"
        );
        assert!(
            m.is_finite(),
            "{op_name} at index {i}: Metal value is not finite ({m})"
        );
        let diff = (c - m).abs();
        assert!(
            diff.is_finite(),
            "{op_name} at index {i}: diff is not finite ({diff})"
        );
        if diff > max_diff {
            max_diff = diff;
        }
    }
    assert!(
        max_diff < tol,
        "{op_name} diff too large: {max_diff} >= {tol}"
    );
    max_diff
}

#[cfg(feature = "metal")]
#[test]
fn test_straight_through_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let len = 256;
    let cont_data: Vec<f32> = (0..len).map(|i| i as f32 * 0.1).collect();
    let quant_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.1).round()).collect();

    let cont_cpu = candle_core::Tensor::from_vec(cont_data.clone(), (1, len), &cpu_dev)?;
    let quant_cpu = candle_core::Tensor::from_vec(quant_data.clone(), (1, len), &cpu_dev)?;

    let cont_metal = candle_core::Tensor::from_vec(cont_data, (1, len), &metal_dev)?;
    let quant_metal = candle_core::Tensor::from_vec(quant_data, (1, len), &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::straight_through(&cont_cpu, &quant_cpu)?;
    let out_metal = uor_r4_training::geometric_stack::straight_through(&cont_metal, &quant_metal)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_metal = out_metal.flatten_all()?.to_vec1::<f32>()?;

    for (c, m) in v_cpu.iter().zip(v_metal.iter()) {
        assert!(c.is_finite() && m.is_finite());
    }
    assert_eq!(v_cpu, v_metal);
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_swiglu_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let len = 1024;
    let gate_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.01 - 5.0).sin()).collect();
    let up_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.02 + 1.0).cos()).collect();

    let gate_cpu = candle_core::Tensor::from_vec(gate_data.clone(), (1, len), &cpu_dev)?;
    let up_cpu = candle_core::Tensor::from_vec(up_data.clone(), (1, len), &cpu_dev)?;

    let gate_metal = candle_core::Tensor::from_vec(gate_data, (1, len), &metal_dev)?;
    let up_metal = candle_core::Tensor::from_vec(up_data, (1, len), &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::swiglu(&gate_cpu, &up_cpu)?;
    let out_metal = uor_r4_training::geometric_stack::swiglu(&gate_metal, &up_metal)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_metal = out_metal.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_metal, 1e-5, "SwiGLU");
    println!("SwiGLU max diff CPU vs Metal: {max_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_rms_norm_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows = 4;
    let width = 288;
    let total = rows * width;
    let x_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.03).sin()).collect();
    let w_data: Vec<f32> = (0..width).map(|i| i as f32 * 0.01 + 0.5).collect();

    let x_cpu = candle_core::Tensor::from_vec(x_data.clone(), (rows, width), &cpu_dev)?;
    let w_cpu = candle_core::Tensor::from_vec(w_data.clone(), (width,), &cpu_dev)?;

    let x_metal = candle_core::Tensor::from_vec(x_data, (rows, width), &metal_dev)?;
    let w_metal = candle_core::Tensor::from_vec(w_data, (width,), &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    let out_metal = uor_r4_training::geometric_stack::rms_norm(&x_metal, &w_metal)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_metal = out_metal.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_metal, 1e-4, "RMSNorm");
    println!("RMSNorm max diff CPU vs Metal: {max_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_quaternion_scan_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let batch = 2;
    let time = 16;
    let lanes = 12;
    let total = batch * time * lanes * 4;

    // Unit-ish quaternions for transitions
    let trans_data: Vec<f32> = (0..total)
        .map(|i| {
            if i % 4 == 0 {
                0.999f32
            } else {
                (i as f32 * 0.001).sin() * 0.01f32
            }
        })
        .collect();
    let drive_data: Vec<f32> = (0..total)
        .map(|i| (i as f32 * 0.05).cos() * 0.1f32)
        .collect();

    let shape = (batch, time, lanes, 4);
    let t_cpu = candle_core::Tensor::from_vec(trans_data.clone(), shape, &cpu_dev)?;
    let d_cpu = candle_core::Tensor::from_vec(drive_data.clone(), shape, &cpu_dev)?;

    let t_metal = candle_core::Tensor::from_vec(trans_data, shape, &metal_dev)?;
    let d_metal = candle_core::Tensor::from_vec(drive_data, shape, &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::quaternion_scan(&t_cpu, &d_cpu)?;
    let out_metal = uor_r4_training::geometric_stack::quaternion_scan(&t_metal, &d_metal)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_metal = out_metal.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_metal, 1e-4, "QuaternionScan");
    println!("QuaternionScan max diff CPU vs Metal: {max_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_cross_entropy_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows: usize = 8;
    let vocab: usize = 256;
    let logits_data: Vec<f32> = (0..rows * vocab).map(|i| (i as f32 * 0.01).sin()).collect();
    let targets: Vec<u32> = (0..rows).map(|r| (r * 17) as u32 % vocab as u32).collect();

    let l_cpu = candle_core::Tensor::from_vec(logits_data.clone(), (rows, vocab), &cpu_dev)?;
    let l_metal = candle_core::Tensor::from_vec(logits_data, (rows, vocab), &metal_dev)?;

    let loss_cpu = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &targets)?;
    let loss_metal = uor_r4_training::geometric_stack::cross_entropy(&l_metal, &targets)?;

    let c_val = loss_cpu.to_scalar::<f32>()?;
    let m_val = loss_metal.to_scalar::<f32>()?;

    assert!(
        c_val.is_finite(),
        "CrossEntropy CPU value is not finite ({c_val})"
    );
    assert!(
        m_val.is_finite(),
        "CrossEntropy Metal value is not finite ({m_val})"
    );
    let diff = (c_val - m_val).abs();
    assert!(diff.is_finite(), "CrossEntropy diff is not finite ({diff})");
    println!("CrossEntropy CPU ({c_val}) vs Metal ({m_val}) diff: {diff}");
    assert!(diff < 1e-4, "CrossEntropy diff too large: {diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_swiglu_backward_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let len = 512;
    let gate_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.02 - 3.0).sin()).collect();
    let up_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.03 + 0.5).cos()).collect();
    let grad_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.01).cos()).collect();

    let g_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        gate_data.clone(),
        (1, len),
        &cpu_dev,
    )?)?;
    let u_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        up_data.clone(),
        (1, len),
        &cpu_dev,
    )?)?;
    let weights_cpu = candle_core::Tensor::from_vec(grad_data.clone(), (1, len), &cpu_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::swiglu(g_cpu.as_tensor(), u_cpu.as_tensor())?;
    let loss_cpu = out_cpu.mul(&weights_cpu)?.sum_all()?;
    let grads_cpu = loss_cpu.backward()?;
    let dg_cpu = grads_cpu
        .get(g_cpu.as_tensor())
        .expect("dg_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let du_cpu = grads_cpu
        .get(u_cpu.as_tensor())
        .expect("du_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let g_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        gate_data,
        (1, len),
        &metal_dev,
    )?)?;
    let u_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        up_data,
        (1, len),
        &metal_dev,
    )?)?;
    let weights_metal = candle_core::Tensor::from_vec(grad_data, (1, len), &metal_dev)?;

    let out_metal =
        uor_r4_training::geometric_stack::swiglu(g_metal.as_tensor(), u_metal.as_tensor())?;
    let loss_metal = out_metal.mul(&weights_metal)?.sum_all()?;
    let grads_metal = loss_metal.backward()?;
    let dg_metal = grads_metal
        .get(g_metal.as_tensor())
        .expect("dg_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let du_metal = grads_metal
        .get(u_metal.as_tensor())
        .expect("du_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dg_diff = assert_finite_and_close(&dg_cpu, &dg_metal, 1e-4, "SwiGLU backward dg");
    let max_du_diff = assert_finite_and_close(&du_cpu, &du_metal, 1e-4, "SwiGLU backward du");

    println!("SwiGLU backward max diff dg: {max_dg_diff}, du: {max_du_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_rms_norm_backward_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows: usize = 4;
    let width: usize = 128;
    let total = rows * width;
    let x_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.05).sin()).collect();
    let w_data: Vec<f32> = (0..width).map(|i| i as f32 * 0.02 + 0.8).collect();
    let grad_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.03).cos()).collect();

    let x_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        x_data.clone(),
        (rows, width),
        &cpu_dev,
    )?)?;
    let w_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        w_data.clone(),
        (width,),
        &cpu_dev,
    )?)?;
    let weights_cpu = candle_core::Tensor::from_vec(grad_data.clone(), (rows, width), &cpu_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::rms_norm(x_cpu.as_tensor(), w_cpu.as_tensor())?;
    let loss_cpu = out_cpu.mul(&weights_cpu)?.sum_all()?;
    let grads_cpu = loss_cpu.backward()?;
    let dx_cpu = grads_cpu
        .get(x_cpu.as_tensor())
        .expect("dx_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dw_cpu = grads_cpu
        .get(w_cpu.as_tensor())
        .expect("dw_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let x_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        x_data,
        (rows, width),
        &metal_dev,
    )?)?;
    let w_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        w_data,
        (width,),
        &metal_dev,
    )?)?;
    let weights_metal = candle_core::Tensor::from_vec(grad_data, (rows, width), &metal_dev)?;

    let out_metal =
        uor_r4_training::geometric_stack::rms_norm(x_metal.as_tensor(), w_metal.as_tensor())?;
    let loss_metal = out_metal.mul(&weights_metal)?.sum_all()?;
    let grads_metal = loss_metal.backward()?;
    let dx_metal = grads_metal
        .get(x_metal.as_tensor())
        .expect("dx_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dw_metal = grads_metal
        .get(w_metal.as_tensor())
        .expect("dw_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dx_diff = assert_finite_and_close(&dx_cpu, &dx_metal, 1e-4, "RMSNorm backward dx");
    let max_dw_diff = assert_finite_and_close(&dw_cpu, &dw_metal, 1e-4, "RMSNorm backward dw");

    println!("RMSNorm backward max diff dx: {max_dx_diff}, dw: {max_dw_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_cross_entropy_backward_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows: usize = 4;
    let vocab: usize = 128;
    let logits_data: Vec<f32> = (0..rows * vocab).map(|i| (i as f32 * 0.02).sin()).collect();
    let targets: Vec<u32> = (0..rows).map(|r| (r * 13) as u32 % vocab as u32).collect();

    let l_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        logits_data.clone(),
        (rows, vocab),
        &cpu_dev,
    )?)?;
    let loss_cpu = uor_r4_training::geometric_stack::cross_entropy(l_cpu.as_tensor(), &targets)?;
    let grads_cpu = loss_cpu.backward()?;
    let dl_cpu = grads_cpu
        .get(l_cpu.as_tensor())
        .expect("dl_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let l_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        logits_data,
        (rows, vocab),
        &metal_dev,
    )?)?;
    let loss_metal =
        uor_r4_training::geometric_stack::cross_entropy(l_metal.as_tensor(), &targets)?;
    let grads_metal = loss_metal.backward()?;
    let dl_metal = grads_metal
        .get(l_metal.as_tensor())
        .expect("dl_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dl_diff = assert_finite_and_close(&dl_cpu, &dl_metal, 1e-4, "CrossEntropy backward dl");

    println!("CrossEntropy backward max diff dl: {max_dl_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_quaternion_scan_backward_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let batch = 2;
    let time = 8;
    let lanes = 4;
    let total = batch * time * lanes * 4;

    let trans_data: Vec<f32> = (0..total)
        .map(|i| {
            if i % 4 == 0 {
                0.999f32
            } else {
                (i as f32 * 0.01).sin() * 0.01f32
            }
        })
        .collect();
    let drive_data: Vec<f32> = (0..total)
        .map(|i| (i as f32 * 0.05).cos() * 0.1f32)
        .collect();
    let grad_data: Vec<f32> = (0..total)
        .map(|i| (i as f32 * 0.02).sin() * 0.05f32)
        .collect();

    let shape = (batch, time, lanes, 4);
    let t_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        trans_data.clone(),
        shape,
        &cpu_dev,
    )?)?;
    let d_cpu = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        drive_data.clone(),
        shape,
        &cpu_dev,
    )?)?;
    let weights_cpu = candle_core::Tensor::from_vec(grad_data.clone(), shape, &cpu_dev)?;

    let out_cpu =
        uor_r4_training::geometric_stack::quaternion_scan(t_cpu.as_tensor(), d_cpu.as_tensor())?;
    let loss_cpu = out_cpu.mul(&weights_cpu)?.sum_all()?;
    let grads_cpu = loss_cpu.backward()?;
    let dt_cpu = grads_cpu
        .get(t_cpu.as_tensor())
        .expect("dt_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dd_cpu = grads_cpu
        .get(d_cpu.as_tensor())
        .expect("dd_cpu")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let t_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        trans_data, shape, &metal_dev,
    )?)?;
    let d_metal = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        drive_data, shape, &metal_dev,
    )?)?;
    let weights_metal = candle_core::Tensor::from_vec(grad_data, shape, &metal_dev)?;

    let out_metal = uor_r4_training::geometric_stack::quaternion_scan(
        t_metal.as_tensor(),
        d_metal.as_tensor(),
    )?;
    let loss_metal = out_metal.mul(&weights_metal)?.sum_all()?;
    let grads_metal = loss_metal.backward()?;
    let dt_metal = grads_metal
        .get(t_metal.as_tensor())
        .expect("dt_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dd_metal = grads_metal
        .get(d_metal.as_tensor())
        .expect("dd_metal")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dt_diff =
        assert_finite_and_close(&dt_cpu, &dt_metal, 1e-4, "QuaternionScan backward dt");
    let max_dd_diff =
        assert_finite_and_close(&dd_cpu, &dd_metal, 1e-4, "QuaternionScan backward dd");

    println!("QuaternionScan backward max diff dt: {max_dt_diff}, dd: {max_dd_diff}");
    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_metal_stack_ops_throughput_and_speedup() -> uor_r4_training::Result<()> {
    use std::time::Instant;

    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    println!("\n=== Metal GPU vs CPU Stack Ops Throughput & Speedup Benchmark ===");

    // Benchmark SwiGLU: batch=16, seq=256 -> 4,096 tokens, dim=768
    let tokens = 4096;
    let dim = 768;
    let total = tokens * dim;
    let g_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.001).sin()).collect();
    let u_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.002).cos()).collect();

    let g_cpu = candle_core::Tensor::from_vec(g_data.clone(), (tokens, dim), &cpu_dev)?;
    let u_cpu = candle_core::Tensor::from_vec(u_data.clone(), (tokens, dim), &cpu_dev)?;
    let g_metal = candle_core::Tensor::from_vec(g_data, (tokens, dim), &metal_dev)?;
    let u_metal = candle_core::Tensor::from_vec(u_data, (tokens, dim), &metal_dev)?;

    // Warmup
    let _ = uor_r4_training::geometric_stack::swiglu(&g_cpu, &u_cpu)?;
    let _ = uor_r4_training::geometric_stack::swiglu(&g_metal, &u_metal)?;
    metal_dev.as_metal_device()?.wait_until_completed()?;

    let iters = 30;
    let start_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::swiglu(&g_cpu, &u_cpu)?;
    }
    let cpu_dur = start_cpu.elapsed().as_secs_f64();
    let cpu_tok_s = (tokens * iters) as f64 / cpu_dur;

    let start_metal = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::swiglu(&g_metal, &u_metal)?;
    }
    metal_dev.as_metal_device()?.wait_until_completed()?;
    let metal_dur = start_metal.elapsed().as_secs_f64();
    let metal_tok_s = (tokens * iters) as f64 / metal_dur;
    let speedup_swiglu = metal_tok_s / cpu_tok_s;

    println!(
        "SwiGLU (tokens={tokens}, dim={dim}): CPU = {cpu_tok_s:.0} tok/s ({cpu_dur:.4}s), Metal = {metal_tok_s:.0} tok/s ({metal_dur:.4}s) -> Speedup: {speedup_swiglu:.2}x"
    );

    // Benchmark RMSNorm: batch=16, seq=256 -> 4,096 tokens, dim=288
    let dim_norm = 288;
    let total_norm = tokens * dim_norm;
    let x_data: Vec<f32> = (0..total_norm).map(|i| (i as f32 * 0.003).sin()).collect();
    let w_data: Vec<f32> = (0..dim_norm)
        .map(|i| 1.0 + (i as f32 * 0.01).cos() * 0.1)
        .collect();

    let x_cpu = candle_core::Tensor::from_vec(x_data.clone(), (tokens, dim_norm), &cpu_dev)?;
    let w_cpu = candle_core::Tensor::from_vec(w_data.clone(), (dim_norm,), &cpu_dev)?;
    let x_metal = candle_core::Tensor::from_vec(x_data, (tokens, dim_norm), &metal_dev)?;
    let w_metal = candle_core::Tensor::from_vec(w_data, (dim_norm,), &metal_dev)?;

    let _ = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    let _ = uor_r4_training::geometric_stack::rms_norm(&x_metal, &w_metal)?;
    metal_dev.as_metal_device()?.wait_until_completed()?;

    let start_norm_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    }
    let cpu_dur_norm = start_norm_cpu.elapsed().as_secs_f64();
    let cpu_tok_s_norm = (tokens * iters) as f64 / cpu_dur_norm;

    let start_norm_metal = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::rms_norm(&x_metal, &w_metal)?;
    }
    metal_dev.as_metal_device()?.wait_until_completed()?;
    let metal_dur_norm = start_norm_metal.elapsed().as_secs_f64();
    let metal_tok_s_norm = (tokens * iters) as f64 / metal_dur_norm;
    let speedup_norm = metal_tok_s_norm / cpu_tok_s_norm;

    println!(
        "RMSNorm (tokens={tokens}, dim={dim_norm}): CPU = {cpu_tok_s_norm:.0} tok/s ({cpu_dur_norm:.4}s), Metal = {metal_tok_s_norm:.0} tok/s ({metal_dur_norm:.4}s) -> Speedup: {speedup_norm:.2}x"
    );

    // Benchmark CrossEntropy: batch=16, seq=256 -> 4,096 tokens, vocab=4096
    let vocab = 4096;
    let total_ce = tokens * vocab;
    let l_data: Vec<f32> = (0..total_ce).map(|i| (i as f32 * 0.001).sin()).collect();
    let t_data: Vec<u32> = (0..tokens).map(|i| (i % vocab) as u32).collect();

    let l_cpu = candle_core::Tensor::from_vec(l_data.clone(), (tokens, vocab), &cpu_dev)?;
    let l_metal = candle_core::Tensor::from_vec(l_data, (tokens, vocab), &metal_dev)?;

    let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &t_data)?;
    let _ = uor_r4_training::geometric_stack::cross_entropy(&l_metal, &t_data)?;
    metal_dev.as_metal_device()?.wait_until_completed()?;

    let start_ce_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &t_data)?;
    }
    let cpu_dur_ce = start_ce_cpu.elapsed().as_secs_f64();
    let cpu_tok_s_ce = (tokens * iters) as f64 / cpu_dur_ce;

    let start_ce_metal = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::cross_entropy(&l_metal, &t_data)?;
    }
    metal_dev.as_metal_device()?.wait_until_completed()?;
    let metal_dur_ce = start_ce_metal.elapsed().as_secs_f64();
    let metal_tok_s_ce = (tokens * iters) as f64 / metal_dur_ce;
    let speedup_ce = metal_tok_s_ce / cpu_tok_s_ce;

    println!(
        "CrossEntropy (tokens={tokens}, vocab={vocab}): CPU = {cpu_tok_s_ce:.0} tok/s ({cpu_dur_ce:.4}s), Metal = {metal_tok_s_ce:.0} tok/s ({metal_dur_ce:.4}s) -> Speedup: {speedup_ce:.2}x"
    );

    assert!(
        speedup_swiglu > 0.5,
        "SwiGLU GPU kernel unexpectedly degraded"
    );
    assert!(
        speedup_norm > 0.5,
        "RMSNorm GPU kernel unexpectedly degraded"
    );
    assert!(
        speedup_ce > 0.5,
        "CrossEntropy GPU kernel unexpectedly degraded"
    );

    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_fused_read_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let batch = 2;
    let heads = 4;
    let time = 8;
    let key_dim = 16;
    let val_dim = 16;

    let q_data: Vec<f32> = (0..batch * heads * time * key_dim)
        .map(|i| (i as f32 * 0.05).sin())
        .collect();
    let k_data: Vec<f32> = (0..batch * heads * time * key_dim)
        .map(|i| (i as f32 * 0.03).cos())
        .collect();
    let v_data: Vec<f32> = (0..batch * heads * time * val_dim)
        .map(|i| (i as f32 * 0.02 + 0.1).sin())
        .collect();
    let aux_data = vec![0.0f32];

    let q_cpu =
        candle_core::Tensor::from_vec(q_data.clone(), (batch, heads, time, key_dim), &cpu_dev)?;
    let k_cpu =
        candle_core::Tensor::from_vec(k_data.clone(), (batch, heads, time, key_dim), &cpu_dev)?;
    let v_cpu =
        candle_core::Tensor::from_vec(v_data.clone(), (batch, heads, time, val_dim), &cpu_dev)?;
    let aux_cpu = candle_core::Tensor::from_vec(aux_data.clone(), (1,), &cpu_dev)?;

    let q_metal = candle_core::Tensor::from_vec(q_data, (batch, heads, time, key_dim), &metal_dev)?;
    let k_metal = candle_core::Tensor::from_vec(k_data, (batch, heads, time, key_dim), &metal_dev)?;
    let v_metal = candle_core::Tensor::from_vec(v_data, (batch, heads, time, val_dim), &metal_dev)?;
    let aux_metal = candle_core::Tensor::from_vec(aux_data, (1,), &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::fused_read(
        &q_cpu,
        &k_cpu,
        &v_cpu,
        &aux_cpu,
        uor_r4_training::geometric_stack::ReadScore::Dot,
        false,
        false,
        false,
    )?;
    let out_metal = uor_r4_training::geometric_stack::fused_read(
        &q_metal,
        &k_metal,
        &v_metal,
        &aux_metal,
        uor_r4_training::geometric_stack::ReadScore::Dot,
        false,
        false,
        false,
    )?;

    let c_vec = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let m_vec = out_metal.flatten_all()?.to_vec1::<f32>()?;
    assert_eq!(c_vec.len(), m_vec.len());

    let max_diff = assert_finite_and_close(&c_vec, &m_vec, 1e-4, "FusedRead Dot");
    println!("FusedRead Dot max diff CPU vs Metal: {max_diff}");

    Ok(())
}

#[cfg(feature = "metal")]
#[test]
fn test_recurrence_core_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let batch = 2;
    let time = 8;
    let width = 32;
    let lanes = width / 4;
    let gate_width = lanes + width;

    let b_data: Vec<f32> = (0..batch * time * 2 * width)
        .map(|i| (i as f32 * 0.01).sin() * 0.1)
        .collect();
    let g_data: Vec<f32> = (0..batch * time * gate_width)
        .map(|i| (i as f32 * 0.02).cos() * 0.1)
        .collect();
    let p_len = (4 + 1) * width + lanes;
    let p_data: Vec<f32> = (0..p_len).map(|i| (i as f32 * 0.03).sin() * 0.05).collect();

    let b_cpu = candle_core::Tensor::from_vec(b_data.clone(), (batch, time, 2 * width), &cpu_dev)?;
    let g_cpu = candle_core::Tensor::from_vec(g_data.clone(), (batch, time, gate_width), &cpu_dev)?;
    let p_cpu = candle_core::Tensor::from_vec(p_data.clone(), (p_len,), &cpu_dev)?;

    let b_metal = candle_core::Tensor::from_vec(b_data, (batch, time, 2 * width), &metal_dev)?;
    let g_metal = candle_core::Tensor::from_vec(g_data, (batch, time, gate_width), &metal_dev)?;
    let p_metal = candle_core::Tensor::from_vec(p_data, (p_len,), &metal_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::recurrence_core(
        &b_cpu, &g_cpu, &p_cpu, batch, time, width, true, None,
    )?;
    let out_metal = uor_r4_training::geometric_stack::recurrence_core(
        &b_metal, &g_metal, &p_metal, batch, time, width, true, None,
    )?;

    let c_vec = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let m_vec = out_metal.flatten_all()?.to_vec1::<f32>()?;
    assert_eq!(c_vec.len(), m_vec.len());

    let max_diff = assert_finite_and_close(&c_vec, &m_vec, 1e-4, "RecurrenceCore");
    println!("RecurrenceCore max diff CPU vs Metal: {max_diff}");

    Ok(())
}

#[test]
fn test_metal_stack_ops_rejections() -> uor_r4_training::Result<()> {
    let cpu_dev = candle_core::Device::Cpu;

    // 1. StraightThrough rejects non-F32 tensors and empty tensors
    let ct_u32 = candle_core::Tensor::zeros((2, 4), candle_core::DType::U32, &cpu_dev)?;
    let q_u32 = candle_core::Tensor::zeros((2, 4), candle_core::DType::U32, &cpu_dev)?;
    let f32_4 = candle_core::Tensor::zeros((2, 4), candle_core::DType::F32, &cpu_dev)?;
    let f32_6 = candle_core::Tensor::zeros((2, 6), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::straight_through(&ct_u32, &q_u32).is_err(),
        "straight_through must reject non-F32 (both U32)"
    );
    assert!(
        uor_r4_training::geometric_stack::straight_through(&ct_u32, &f32_4).is_err(),
        "straight_through must reject non-F32 continuous"
    );
    assert!(
        uor_r4_training::geometric_stack::straight_through(&f32_4, &q_u32).is_err(),
        "straight_through must reject non-F32 quantized"
    );
    assert!(
        uor_r4_training::geometric_stack::straight_through(&f32_4, &f32_6).is_err(),
        "straight_through must reject shape mismatch"
    );
    let empty_st = candle_core::Tensor::zeros((0, 4), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::straight_through(&empty_st, &empty_st).is_err(),
        "straight_through must reject empty tensors"
    );
    let empty_6 = candle_core::Tensor::zeros((0, 6), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::straight_through(&empty_st, &empty_6).is_err(),
        "straight_through must reject empty shape mismatch"
    );

    // 2. SwiGLU rejects non-F32, shape mismatch, empty
    let f32_4 = candle_core::Tensor::zeros((2, 4), candle_core::DType::F32, &cpu_dev)?;
    let f32_6 = candle_core::Tensor::zeros((2, 6), candle_core::DType::F32, &cpu_dev)?;
    let u32_4 = candle_core::Tensor::zeros((2, 4), candle_core::DType::U32, &cpu_dev)?;
    let empty_f32 = candle_core::Tensor::zeros((0, 4), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::swiglu(&u32_4, &u32_4).is_err(),
        "swiglu must reject non-F32"
    );
    assert!(
        uor_r4_training::geometric_stack::swiglu(&f32_4, &f32_6).is_err(),
        "swiglu must reject shape mismatch"
    );
    assert!(
        uor_r4_training::geometric_stack::swiglu(&empty_f32, &empty_f32).is_err(),
        "swiglu must reject empty tensor"
    );

    // 3. RMSNorm rejects non-F32, empty weight, last dimension mismatch
    let empty_w = candle_core::Tensor::zeros((0,), candle_core::DType::F32, &cpu_dev)?;
    let w_6 = candle_core::Tensor::zeros((6,), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::rms_norm(&f32_4, &empty_w).is_err(),
        "rms_norm must reject empty weight"
    );
    assert!(
        uor_r4_training::geometric_stack::rms_norm(&f32_4, &w_6).is_err(),
        "rms_norm must reject mismatched last dimension"
    );
    assert!(
        uor_r4_training::geometric_stack::rms_norm(&u32_4, &f32_4).is_err(),
        "rms_norm must reject non-F32"
    );

    // 4. QuaternionScan rejects non-F32, shape mismatch, zero/invalid dims
    let f32_scan = candle_core::Tensor::zeros((2, 4, 2, 4), candle_core::DType::F32, &cpu_dev)?;
    let u32_scan = candle_core::Tensor::zeros((2, 4, 2, 4), candle_core::DType::U32, &cpu_dev)?;
    let f32_scan_bad_dim =
        candle_core::Tensor::zeros((2, 4, 2, 3), candle_core::DType::F32, &cpu_dev)?;
    let f32_scan_zero =
        candle_core::Tensor::zeros((0, 4, 2, 4), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::quaternion_scan(&u32_scan, &u32_scan).is_err(),
        "quaternion_scan must reject non-F32"
    );
    assert!(
        uor_r4_training::geometric_stack::quaternion_scan(&f32_scan, &f32_scan_bad_dim).is_err(),
        "quaternion_scan must reject shape mismatch / last dim != 4"
    );
    assert!(
        uor_r4_training::geometric_stack::quaternion_scan(&f32_scan_zero, &f32_scan_zero).is_err(),
        "quaternion_scan must reject zero dimensions"
    );

    // 5. CrossEntropy rejects non-F32, targets count mismatch
    let logits_f32 = candle_core::Tensor::zeros((4, 16), candle_core::DType::F32, &cpu_dev)?;
    let logits_u32 = candle_core::Tensor::zeros((4, 16), candle_core::DType::U32, &cpu_dev)?;
    let targets_3 = vec![1u32, 2, 3];
    assert!(
        uor_r4_training::geometric_stack::logits_cross_entropy(&logits_u32, &targets_3, None)
            .is_err(),
        "logits_cross_entropy must reject non-F32"
    );
    assert!(
        uor_r4_training::geometric_stack::logits_cross_entropy(&logits_f32, &targets_3, None)
            .is_err(),
        "logits_cross_entropy must reject targets len mismatch (3 != 4)"
    );

    // 6. RecurrenceCore rejects non-F32, zero dims, width not divisible by 4, buffer size mismatch
    let b_f32 = candle_core::Tensor::zeros((2, 4, 2 * 16), candle_core::DType::F32, &cpu_dev)?;
    let g_f32 = candle_core::Tensor::zeros((2, 4, 4 + 16), candle_core::DType::F32, &cpu_dev)?;
    let p_f32 = candle_core::Tensor::zeros(((4 + 1) * 16 + 4,), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::recurrence_core(
            &b_f32, &g_f32, &p_f32, 0, 4, 16, true, None
        )
        .is_err(),
        "recurrence_core must reject batch=0"
    );
    assert!(
        uor_r4_training::geometric_stack::recurrence_core(
            &b_f32, &g_f32, &p_f32, 2, 4, 15, true, None
        )
        .is_err(),
        "recurrence_core must reject width % 4 != 0"
    );
    let p_bad = candle_core::Tensor::zeros((10,), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::recurrence_core(
            &b_f32, &g_f32, &p_bad, 2, 4, 16, true, None
        )
        .is_err(),
        "recurrence_core must reject buffer length mismatch"
    );

    // 7. FusedRead rejects non-F32, zero dims, buffer size mismatch
    let q_f32 = candle_core::Tensor::zeros((2, 4, 8, 16), candle_core::DType::F32, &cpu_dev)?;
    let k_f32 = candle_core::Tensor::zeros((2, 4, 8, 16), candle_core::DType::F32, &cpu_dev)?;
    let v_f32 = candle_core::Tensor::zeros((2, 4, 8, 16), candle_core::DType::F32, &cpu_dev)?;
    let aux_f32 = candle_core::Tensor::zeros((1,), candle_core::DType::F32, &cpu_dev)?;
    let q_bad = candle_core::Tensor::zeros((2, 4, 8, 15), candle_core::DType::F32, &cpu_dev)?;
    assert!(
        uor_r4_training::geometric_stack::fused_read(
            &q_bad,
            &k_f32,
            &v_f32,
            &aux_f32,
            uor_r4_training::geometric_stack::ReadScore::Dot,
            false,
            false,
            false
        )
        .is_err(),
        "fused_read must reject dimension mismatch"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Backward parity of the recurrence core and the general fused read.
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random values in [-scale, scale].
#[cfg(feature = "metal")]
fn noise(len: usize, seed: u64, scale: f32) -> Vec<f32> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = ((state >> 40) as f32) / ((1u64 << 24) as f32);
            (2.0 * unit - 1.0) * scale
        })
        .collect()
}

/// Max absolute error and max error relative to the CPU tensor's largest
/// magnitude; asserts finiteness and `relative < tol`.
#[cfg(feature = "metal")]
fn compare(cpu: &[f32], metal: &[f32], tol: f32, name: &str) -> (f32, f32) {
    assert_eq!(cpu.len(), metal.len(), "{name}: length mismatch");
    let mut max_abs = 0f32;
    let mut peak = 0f32;
    for (i, (&c, &m)) in cpu.iter().zip(metal).enumerate() {
        assert!(
            c.is_finite() && m.is_finite(),
            "{name}[{i}]: cpu {c} metal {m}"
        );
        max_abs = max_abs.max((c - m).abs());
        peak = peak.max(c.abs());
    }
    let relative = max_abs / peak.max(1e-6);
    println!("{name}: max abs {max_abs:.3e}, max rel {relative:.3e} (peak {peak:.3e})");
    assert!(relative < tol, "{name}: relative error {relative} >= {tol}");
    (max_abs, relative)
}

#[cfg(feature = "metal")]
fn values(t: &candle_core::Tensor) -> uor_r4_training::Result<Vec<f32>> {
    Ok(t.flatten_all()?
        .to_device(&candle_core::Device::Cpu)?
        .to_vec1::<f32>()?)
}

/// Output and input gradients of the recurrence core for a weighted-sum loss.
#[cfg(feature = "metal")]
#[allow(clippy::too_many_arguments)]
fn recurrence_run(
    device: &candle_core::Device,
    data: [&Vec<f32>; 4],
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
    gate_width: usize,
) -> uor_r4_training::Result<Vec<Vec<f32>>> {
    let p_len = 5 * width + width / 4;
    let b = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[0].clone(),
        (batch, time, 2 * width),
        device,
    )?)?;
    let g = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[1].clone(),
        (batch, time, gate_width),
        device,
    )?)?;
    let p = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[2].clone(),
        (p_len,),
        device,
    )?)?;
    let w = candle_core::Tensor::from_vec(data[3].clone(), (batch, time, width), device)?;
    let out = uor_r4_training::geometric_stack::recurrence_core(
        b.as_tensor(),
        g.as_tensor(),
        p.as_tensor(),
        batch,
        time,
        width,
        rotation,
        None,
    )?;
    let grads = out.mul(&w)?.sum_all()?.backward()?;
    let mut results = vec![values(&out)?];
    for var in [&b, &g, &p] {
        results.push(values(grads.get(var.as_tensor()).expect("gradient"))?);
    }
    Ok(results)
}

#[cfg(feature = "metal")]
#[test]
fn test_recurrence_core_backward_parity() -> uor_r4_training::Result<()> {
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;
    // (batch, time, width, rotation)
    for (case, &(batch, time, width, rotation)) in [
        (2usize, 7usize, 16usize, true),
        (3, 13, 32, false),
        (2, 37, 64, true),
        (1, 5, 4, true),
    ]
    .iter()
    .enumerate()
    {
        let lanes = width / 4;
        let gate_width = lanes + if rotation { width } else { 0 };
        let seed = 100 + case as u64 * 10;
        let b_data = noise(batch * time * 2 * width, seed, 1.0);
        let mut g_data = noise(batch * time * gate_width, seed + 1, 1.5);
        if rotation {
            // Keep rotation quaternions away from zero norm.
            for row in g_data.chunks_mut(gate_width) {
                for lane in 0..lanes {
                    row[lanes + 4 * lane] += 1.0;
                }
            }
        }
        let mut p_data = noise(5 * width + lanes, seed + 2, 0.5);
        for (i, value) in p_data.iter_mut().enumerate().skip(5 * width) {
            *value = -1.0 + 2.0 * (i % 5) as f32;
        }
        let w_data = noise(batch * time * width, seed + 3, 1.0);
        let data = [&b_data, &g_data, &p_data, &w_data];
        let cpu = recurrence_run(&cpu_dev, data, batch, time, width, rotation, gate_width)?;
        let metal = recurrence_run(&metal_dev, data, batch, time, width, rotation, gate_width)?;
        for (k, name) in ["out", "d_branches", "d_gates", "d_parameters"]
            .iter()
            .enumerate()
        {
            compare(
                &cpu[k],
                &metal[k],
                1e-3,
                &format!("RecurrenceCore b{batch} t{time} w{width} rot{rotation} {name}"),
            );
        }
    }
    Ok(())
}

/// Output and input gradients of the fused read for a weighted-sum loss.
#[cfg(feature = "metal")]
#[allow(clippy::too_many_arguments)]
fn read_run(
    device: &candle_core::Device,
    data: [&Vec<f32>; 5],
    shape: (usize, usize, usize, usize, usize),
    score: uor_r4_training::geometric_stack::ReadScore,
    null: bool,
    age: bool,
) -> uor_r4_training::Result<Vec<Vec<f32>>> {
    let (batch, heads, time, key, value) = shape;
    let q = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[0].clone(),
        (batch, heads, time, key),
        device,
    )?)?;
    let k = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[1].clone(),
        (batch, heads, time, key),
        device,
    )?)?;
    let v = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[2].clone(),
        (batch, heads, time, value),
        device,
    )?)?;
    let aux_len = data[3].len();
    let a = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        data[3].clone(),
        (aux_len,),
        device,
    )?)?;
    let w = candle_core::Tensor::from_vec(data[4].clone(), (batch, heads, time, value), device)?;
    let out = uor_r4_training::geometric_stack::fused_read(
        q.as_tensor(),
        k.as_tensor(),
        v.as_tensor(),
        a.as_tensor(),
        score,
        null,
        age,
        false,
    )?;
    let grads = out.mul(&w)?.sum_all()?.backward()?;
    let mut results = vec![values(&out)?];
    for var in [&q, &k, &v, &a] {
        results.push(values(grads.get(var.as_tensor()).expect("gradient"))?);
    }
    Ok(results)
}

#[cfg(feature = "metal")]
#[test]
fn test_fused_read_general_parity() -> uor_r4_training::Result<()> {
    use uor_r4_training::geometric_stack::{fused_aux_len, ReadScore};
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let cpu_dev = candle_core::Device::Cpu;
    let shapes = [
        (2usize, 3usize, 13usize, 8usize, 9usize),
        (1, 2, 37, 16, 16),
        (2, 4, 5, 4, 5),
        (1, 1, 1, 4, 4),
    ];
    let configs = [
        (ReadScore::Dot, false, false),
        (ReadScore::Dot, true, true),
        (ReadScore::Lorentz, true, true),
        (ReadScore::Lorentz, false, false),
        (ReadScore::Lorentz, true, false),
    ];
    let mut case = 0u64;
    for &shape in &shapes {
        let (batch, heads, time, key, value) = shape;
        for &(score, null, age) in &configs {
            case += 1;
            let seed = 1000 + 17 * case;
            let q_data = noise(batch * heads * time * key, seed, 0.8);
            let k_data = noise(batch * heads * time * key, seed + 1, 0.8);
            let v_data = noise(batch * heads * time * value, seed + 2, 1.0);
            let aux_len = fused_aux_len(batch, heads, time, score, null, age).max(1);
            let mut aux_data = noise(aux_len, seed + 3, 0.5);
            if score == ReadScore::Lorentz {
                let base = aux_len - 2 * heads;
                for h in 0..heads {
                    // beta (already exponentiated) and offset.
                    aux_data[base + h] = 0.7 + 0.3 * h as f32;
                    aux_data[base + heads + h] = 0.2 * h as f32 - 0.1;
                }
            }
            let w_data = noise(batch * heads * time * value, seed + 4, 1.0);
            let data = [&q_data, &k_data, &v_data, &aux_data, &w_data];
            let cpu = read_run(&cpu_dev, data, shape, score, null, age)?;
            let metal = read_run(&metal_dev, data, shape, score, null, age)?;
            for (k, name) in ["out", "dq", "dk", "dv", "d_aux"].iter().enumerate() {
                compare(
                    &cpu[k],
                    &metal[k],
                    1e-3,
                    &format!(
                        "FusedRead {score:?} null{null} age{age} b{batch} h{heads} t{time} k{key} v{value} {name}"
                    ),
                );
            }
        }
    }
    Ok(())
}

/// Timing of the training-size read and recurrence, forward plus backward,
/// on Metal and CPU. Run explicitly: `--ignored --nocapture`.
#[cfg(feature = "metal")]
#[test]
#[ignore]
fn bench_training_size_read_and_recurrence() -> uor_r4_training::Result<()> {
    use std::time::Instant;
    use uor_r4_training::geometric_stack::{fused_aux_len, ReadScore};
    let metal_dev = match candle_core::Device::new_metal(0) {
        Ok(dev) => dev,
        Err(_) => return Ok(()),
    };
    let (batch, heads, time, key, value, width) = (16, 8, 384, 64, 64, 512);
    let shape = (batch, heads, time, key, value);
    let q = noise(batch * heads * time * key, 1, 0.5);
    let k = noise(batch * heads * time * key, 2, 0.5);
    let v = noise(batch * heads * time * value, 3, 1.0);
    let mut aux = noise(
        fused_aux_len(batch, heads, time, ReadScore::Lorentz, true, true),
        4,
        0.5,
    );
    let n = aux.len();
    for h in 0..heads {
        aux[n - 2 * heads + h] = 1.0;
    }
    let w = noise(batch * heads * time * value, 5, 1.0);
    let lanes = width / 4;
    let gate_width = lanes + width;
    let b_data = noise(batch * time * 2 * width, 6, 1.0);
    let g_data = noise(batch * time * gate_width, 7, 1.0);
    let p_data = noise(5 * width + lanes, 8, 0.5);
    let rw = noise(batch * time * width, 9, 1.0);
    for device in [metal_dev.clone(), candle_core::Device::Cpu] {
        for round in 0..3 {
            let start = Instant::now();
            read_run(
                &device,
                [&q, &k, &v, &aux, &w],
                shape,
                ReadScore::Lorentz,
                true,
                true,
            )?;
            let read = start.elapsed();
            let start = Instant::now();
            recurrence_run(
                &device,
                [&b_data, &g_data, &p_data, &rw],
                batch,
                time,
                width,
                true,
                gate_width,
            )?;
            let recurrence = start.elapsed();
            println!("{device:?} round {round}: read {read:?}, recurrence {recurrence:?}");
        }
    }
    Ok(())
}
