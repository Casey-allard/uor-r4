//! Numerical Parity Tests: CPU Reference vs Native CUDA Stack Kernels.
//!
//! Mirrors `metal_stack_ops_parity.rs` (same shapes and tolerances) for the
//! CUDA kernels in `cuda_stack_kernels.rs`, plus the configurations only
//! CUDA runs on the device: the U(1) recurrence control and the L2 read
//! control. Without a CUDA device every device test prints that it was
//! skipped and returns `Ok(())`, so a pass count proves nothing; set
//! `UOR_REQUIRE_CUDA=1` to make a missing device fail every device test (use it
//! on GPU hosts and in any lane that claims GPU results). The NVRTC compile
//! test needs only the toolkit.

/// A missing device: a failure under `UOR_REQUIRE_CUDA=1`, otherwise a
/// printed skip (never a silent pass).
/// Not feature-gated: `test_recurrence_core_large_gate_parity` also builds
/// without the `cuda` feature, where it always skips.
fn no_device(error: impl std::fmt::Display) -> uor_r4_training::Result<()> {
    if std::env::var("UOR_REQUIRE_CUDA").is_ok_and(|v| v == "1") {
        panic!("UOR_REQUIRE_CUDA=1 but no CUDA device: {error}");
    }
    println!("SKIPPED (no CUDA device): {error}");
    Ok(())
}

/// The kernel source compiles with the installed NVRTC (no device needed).
#[cfg(feature = "cuda")]
#[test]
fn test_cuda_kernels_compile_with_nvrtc() -> uor_r4_training::Result<()> {
    uor_r4_training::cuda_stack_kernels::cuda::compile_check()?;
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_device_available() -> uor_r4_training::Result<()> {
    match candle_core::Device::new_cuda(0) {
        Ok(device) => {
            println!("CUDA device 0 initialized successfully: {:?}", device);
            Ok(())
        }
        Err(e) => no_device(e),
    }
}

#[cfg(feature = "cuda")]
fn assert_finite_and_close(cpu: &[f32], cuda: &[f32], tol: f32, op_name: &str) -> f32 {
    assert_eq!(cpu.len(), cuda.len(), "{op_name}: length mismatch");
    let mut max_diff = 0.0f32;
    for (i, (&c, &m)) in cpu.iter().zip(cuda.iter()).enumerate() {
        assert!(
            c.is_finite(),
            "{op_name} at index {i}: CPU value is not finite ({c})"
        );
        assert!(
            m.is_finite(),
            "{op_name} at index {i}: CUDA value is not finite ({m})"
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

#[cfg(feature = "cuda")]
#[test]
fn test_straight_through_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let len = 256;
    let cont_data: Vec<f32> = (0..len).map(|i| i as f32 * 0.1).collect();
    let quant_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.1).round()).collect();

    let cont_cpu = candle_core::Tensor::from_vec(cont_data.clone(), (1, len), &cpu_dev)?;
    let quant_cpu = candle_core::Tensor::from_vec(quant_data.clone(), (1, len), &cpu_dev)?;

    let cont_cuda = candle_core::Tensor::from_vec(cont_data, (1, len), &cuda_dev)?;
    let quant_cuda = candle_core::Tensor::from_vec(quant_data, (1, len), &cuda_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::straight_through(&cont_cpu, &quant_cpu)?;
    let out_cuda = uor_r4_training::geometric_stack::straight_through(&cont_cuda, &quant_cuda)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_cuda = out_cuda.flatten_all()?.to_vec1::<f32>()?;

    for (c, m) in v_cpu.iter().zip(v_cuda.iter()) {
        assert!(c.is_finite() && m.is_finite());
    }
    assert_eq!(v_cpu, v_cuda);
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_swiglu_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let len = 1024;
    let gate_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.01 - 5.0).sin()).collect();
    let up_data: Vec<f32> = (0..len).map(|i| (i as f32 * 0.02 + 1.0).cos()).collect();

    let gate_cpu = candle_core::Tensor::from_vec(gate_data.clone(), (1, len), &cpu_dev)?;
    let up_cpu = candle_core::Tensor::from_vec(up_data.clone(), (1, len), &cpu_dev)?;

    let gate_cuda = candle_core::Tensor::from_vec(gate_data, (1, len), &cuda_dev)?;
    let up_cuda = candle_core::Tensor::from_vec(up_data, (1, len), &cuda_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::swiglu(&gate_cpu, &up_cpu)?;
    let out_cuda = uor_r4_training::geometric_stack::swiglu(&gate_cuda, &up_cuda)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_cuda = out_cuda.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_cuda, 1e-5, "SwiGLU");
    println!("SwiGLU max diff CPU vs CUDA: {max_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_rms_norm_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows = 4;
    let width = 288;
    let total = rows * width;
    let x_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.03).sin()).collect();
    let w_data: Vec<f32> = (0..width).map(|i| i as f32 * 0.01 + 0.5).collect();

    let x_cpu = candle_core::Tensor::from_vec(x_data.clone(), (rows, width), &cpu_dev)?;
    let w_cpu = candle_core::Tensor::from_vec(w_data.clone(), (width,), &cpu_dev)?;

    let x_cuda = candle_core::Tensor::from_vec(x_data, (rows, width), &cuda_dev)?;
    let w_cuda = candle_core::Tensor::from_vec(w_data, (width,), &cuda_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    let out_cuda = uor_r4_training::geometric_stack::rms_norm(&x_cuda, &w_cuda)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_cuda = out_cuda.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_cuda, 1e-4, "RMSNorm");
    println!("RMSNorm max diff CPU vs CUDA: {max_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_quaternion_scan_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let t_cuda = candle_core::Tensor::from_vec(trans_data, shape, &cuda_dev)?;
    let d_cuda = candle_core::Tensor::from_vec(drive_data, shape, &cuda_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::quaternion_scan(&t_cpu, &d_cpu)?;
    let out_cuda = uor_r4_training::geometric_stack::quaternion_scan(&t_cuda, &d_cuda)?;

    let v_cpu = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let v_cuda = out_cuda.flatten_all()?.to_vec1::<f32>()?;

    let max_diff = assert_finite_and_close(&v_cpu, &v_cuda, 1e-4, "QuaternionScan");
    println!("QuaternionScan max diff CPU vs CUDA: {max_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_cross_entropy_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;

    let rows: usize = 8;
    let vocab: usize = 256;
    let logits_data: Vec<f32> = (0..rows * vocab).map(|i| (i as f32 * 0.01).sin()).collect();
    let targets: Vec<u32> = (0..rows).map(|r| (r * 17) as u32 % vocab as u32).collect();

    let l_cpu = candle_core::Tensor::from_vec(logits_data.clone(), (rows, vocab), &cpu_dev)?;
    let l_cuda = candle_core::Tensor::from_vec(logits_data, (rows, vocab), &cuda_dev)?;

    let loss_cpu = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &targets)?;
    let loss_cuda = uor_r4_training::geometric_stack::cross_entropy(&l_cuda, &targets)?;

    let c_val = loss_cpu.to_scalar::<f32>()?;
    let m_val = loss_cuda.to_scalar::<f32>()?;

    assert!(
        c_val.is_finite(),
        "CrossEntropy CPU value is not finite ({c_val})"
    );
    assert!(
        m_val.is_finite(),
        "CrossEntropy CUDA value is not finite ({m_val})"
    );
    let diff = (c_val - m_val).abs();
    assert!(diff.is_finite(), "CrossEntropy diff is not finite ({diff})");
    println!("CrossEntropy CPU ({c_val}) vs CUDA ({m_val}) diff: {diff}");
    assert!(diff < 1e-4, "CrossEntropy diff too large: {diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_swiglu_backward_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let g_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        gate_data,
        (1, len),
        &cuda_dev,
    )?)?;
    let u_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        up_data,
        (1, len),
        &cuda_dev,
    )?)?;
    let weights_cuda = candle_core::Tensor::from_vec(grad_data, (1, len), &cuda_dev)?;

    let out_cuda =
        uor_r4_training::geometric_stack::swiglu(g_cuda.as_tensor(), u_cuda.as_tensor())?;
    let loss_cuda = out_cuda.mul(&weights_cuda)?.sum_all()?;
    let grads_cuda = loss_cuda.backward()?;
    let dg_cuda = grads_cuda
        .get(g_cuda.as_tensor())
        .expect("dg_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let du_cuda = grads_cuda
        .get(u_cuda.as_tensor())
        .expect("du_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dg_diff = assert_finite_and_close(&dg_cpu, &dg_cuda, 1e-4, "SwiGLU backward dg");
    let max_du_diff = assert_finite_and_close(&du_cpu, &du_cuda, 1e-4, "SwiGLU backward du");

    println!("SwiGLU backward max diff dg: {max_dg_diff}, du: {max_du_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_rms_norm_backward_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let x_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        x_data,
        (rows, width),
        &cuda_dev,
    )?)?;
    let w_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        w_data,
        (width,),
        &cuda_dev,
    )?)?;
    let weights_cuda = candle_core::Tensor::from_vec(grad_data, (rows, width), &cuda_dev)?;

    let out_cuda =
        uor_r4_training::geometric_stack::rms_norm(x_cuda.as_tensor(), w_cuda.as_tensor())?;
    let loss_cuda = out_cuda.mul(&weights_cuda)?.sum_all()?;
    let grads_cuda = loss_cuda.backward()?;
    let dx_cuda = grads_cuda
        .get(x_cuda.as_tensor())
        .expect("dx_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dw_cuda = grads_cuda
        .get(w_cuda.as_tensor())
        .expect("dw_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dx_diff = assert_finite_and_close(&dx_cpu, &dx_cuda, 1e-4, "RMSNorm backward dx");
    let max_dw_diff = assert_finite_and_close(&dw_cpu, &dw_cuda, 1e-4, "RMSNorm backward dw");

    println!("RMSNorm backward max diff dx: {max_dx_diff}, dw: {max_dw_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_cross_entropy_backward_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let l_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        logits_data,
        (rows, vocab),
        &cuda_dev,
    )?)?;
    let loss_cuda = uor_r4_training::geometric_stack::cross_entropy(l_cuda.as_tensor(), &targets)?;
    let grads_cuda = loss_cuda.backward()?;
    let dl_cuda = grads_cuda
        .get(l_cuda.as_tensor())
        .expect("dl_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dl_diff = assert_finite_and_close(&dl_cpu, &dl_cuda, 1e-4, "CrossEntropy backward dl");

    println!("CrossEntropy backward max diff dl: {max_dl_diff}");
    Ok(())
}

/// The weighted mean cross-entropy (with a zero-weight row), forward and
/// backward: CUDA computes the weighted gradient on the device.
#[cfg(feature = "cuda")]
#[test]
fn test_weighted_cross_entropy_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let (rows, vocab) = (6usize, 300usize);
    let logits_data: Vec<f32> = (0..rows * vocab)
        .map(|i| (i as f32 * 0.013).sin() * 4.0)
        .collect();
    let targets: Vec<u32> = (0..rows).map(|r| (r * 47 % vocab) as u32).collect();
    let weights = [1.0f32, 0.0, 2.5, 0.25, 1.0, 3.0];
    let run = |device: &candle_core::Device| -> uor_r4_training::Result<(f32, Vec<f32>)> {
        let logits = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
            logits_data.clone(),
            (rows, vocab),
            device,
        )?)?;
        let loss = uor_r4_training::geometric_stack::logits_cross_entropy(
            logits.as_tensor(),
            &targets,
            Some(&weights),
        )?;
        let grads = (&loss * 1.7)?.backward()?;
        let grad = grads
            .get(logits.as_tensor())
            .expect("logit gradient")
            .flatten_all()?
            .to_device(&candle_core::Device::Cpu)?
            .to_vec1::<f32>()?;
        Ok((
            loss.to_device(&candle_core::Device::Cpu)?
                .to_scalar::<f32>()?,
            grad,
        ))
    };
    let (loss_cpu, grad_cpu) = run(&candle_core::Device::Cpu)?;
    let (loss_cuda, grad_cuda) = run(&cuda_dev)?;
    assert!(
        (loss_cpu - loss_cuda).abs() <= 1e-5 * loss_cpu.abs().max(1.0),
        "weighted loss {loss_cpu} vs {loss_cuda}"
    );
    assert_finite_and_close(
        &grad_cpu,
        &grad_cuda,
        1e-5,
        "Weighted CrossEntropy backward",
    );
    // The zero-weight row has an exactly zero gradient on both devices.
    assert!(grad_cuda[vocab..2 * vocab].iter().all(|&g| g == 0.0));
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_quaternion_scan_backward_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let t_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        trans_data, shape, &cuda_dev,
    )?)?;
    let d_cuda = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        drive_data, shape, &cuda_dev,
    )?)?;
    let weights_cuda = candle_core::Tensor::from_vec(grad_data, shape, &cuda_dev)?;

    let out_cuda =
        uor_r4_training::geometric_stack::quaternion_scan(t_cuda.as_tensor(), d_cuda.as_tensor())?;
    let loss_cuda = out_cuda.mul(&weights_cuda)?.sum_all()?;
    let grads_cuda = loss_cuda.backward()?;
    let dt_cuda = grads_cuda
        .get(t_cuda.as_tensor())
        .expect("dt_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;
    let dd_cuda = grads_cuda
        .get(d_cuda.as_tensor())
        .expect("dd_cuda")
        .flatten_all()?
        .to_vec1::<f32>()?;

    let max_dt_diff =
        assert_finite_and_close(&dt_cpu, &dt_cuda, 1e-4, "QuaternionScan backward dt");
    let max_dd_diff =
        assert_finite_and_close(&dd_cpu, &dd_cuda, 1e-4, "QuaternionScan backward dd");

    println!("QuaternionScan backward max diff dt: {max_dt_diff}, dd: {max_dd_diff}");
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_cuda_stack_ops_throughput_and_speedup() -> uor_r4_training::Result<()> {
    use std::time::Instant;

    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;

    println!("\n=== CUDA GPU vs CPU Stack Ops Throughput & Speedup Benchmark ===");

    // Benchmark SwiGLU: batch=16, seq=256 -> 4,096 tokens, dim=768
    let tokens = 4096;
    let dim = 768;
    let total = tokens * dim;
    let g_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.001).sin()).collect();
    let u_data: Vec<f32> = (0..total).map(|i| (i as f32 * 0.002).cos()).collect();

    let g_cpu = candle_core::Tensor::from_vec(g_data.clone(), (tokens, dim), &cpu_dev)?;
    let u_cpu = candle_core::Tensor::from_vec(u_data.clone(), (tokens, dim), &cpu_dev)?;
    let g_cuda = candle_core::Tensor::from_vec(g_data, (tokens, dim), &cuda_dev)?;
    let u_cuda = candle_core::Tensor::from_vec(u_data, (tokens, dim), &cuda_dev)?;

    // Warmup
    let _ = uor_r4_training::geometric_stack::swiglu(&g_cpu, &u_cpu)?;
    let _ = uor_r4_training::geometric_stack::swiglu(&g_cuda, &u_cuda)?;
    cuda_dev.synchronize()?;

    let iters = 30;
    let start_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::swiglu(&g_cpu, &u_cpu)?;
    }
    let cpu_dur = start_cpu.elapsed().as_secs_f64();
    let cpu_tok_s = (tokens * iters) as f64 / cpu_dur;

    let start_cuda = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::swiglu(&g_cuda, &u_cuda)?;
    }
    cuda_dev.synchronize()?;
    let cuda_dur = start_cuda.elapsed().as_secs_f64();
    let cuda_tok_s = (tokens * iters) as f64 / cuda_dur;
    let speedup_swiglu = cuda_tok_s / cpu_tok_s;

    println!(
        "SwiGLU (tokens={tokens}, dim={dim}): CPU = {cpu_tok_s:.0} tok/s ({cpu_dur:.4}s), CUDA = {cuda_tok_s:.0} tok/s ({cuda_dur:.4}s) -> Speedup: {speedup_swiglu:.2}x"
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
    let x_cuda = candle_core::Tensor::from_vec(x_data, (tokens, dim_norm), &cuda_dev)?;
    let w_cuda = candle_core::Tensor::from_vec(w_data, (dim_norm,), &cuda_dev)?;

    let _ = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    let _ = uor_r4_training::geometric_stack::rms_norm(&x_cuda, &w_cuda)?;
    cuda_dev.synchronize()?;

    let start_norm_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::rms_norm(&x_cpu, &w_cpu)?;
    }
    let cpu_dur_norm = start_norm_cpu.elapsed().as_secs_f64();
    let cpu_tok_s_norm = (tokens * iters) as f64 / cpu_dur_norm;

    let start_norm_cuda = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::rms_norm(&x_cuda, &w_cuda)?;
    }
    cuda_dev.synchronize()?;
    let cuda_dur_norm = start_norm_cuda.elapsed().as_secs_f64();
    let cuda_tok_s_norm = (tokens * iters) as f64 / cuda_dur_norm;
    let speedup_norm = cuda_tok_s_norm / cpu_tok_s_norm;

    println!(
        "RMSNorm (tokens={tokens}, dim={dim_norm}): CPU = {cpu_tok_s_norm:.0} tok/s ({cpu_dur_norm:.4}s), CUDA = {cuda_tok_s_norm:.0} tok/s ({cuda_dur_norm:.4}s) -> Speedup: {speedup_norm:.2}x"
    );

    // Benchmark CrossEntropy: batch=16, seq=256 -> 4,096 tokens, vocab=4096
    let vocab = 4096;
    let total_ce = tokens * vocab;
    let l_data: Vec<f32> = (0..total_ce).map(|i| (i as f32 * 0.001).sin()).collect();
    let t_data: Vec<u32> = (0..tokens).map(|i| (i % vocab) as u32).collect();

    let l_cpu = candle_core::Tensor::from_vec(l_data.clone(), (tokens, vocab), &cpu_dev)?;
    let l_cuda = candle_core::Tensor::from_vec(l_data, (tokens, vocab), &cuda_dev)?;

    let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &t_data)?;
    let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cuda, &t_data)?;
    cuda_dev.synchronize()?;

    let start_ce_cpu = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cpu, &t_data)?;
    }
    let cpu_dur_ce = start_ce_cpu.elapsed().as_secs_f64();
    let cpu_tok_s_ce = (tokens * iters) as f64 / cpu_dur_ce;

    let start_ce_cuda = Instant::now();
    for _ in 0..iters {
        let _ = uor_r4_training::geometric_stack::cross_entropy(&l_cuda, &t_data)?;
    }
    cuda_dev.synchronize()?;
    let cuda_dur_ce = start_ce_cuda.elapsed().as_secs_f64();
    let cuda_tok_s_ce = (tokens * iters) as f64 / cuda_dur_ce;
    let speedup_ce = cuda_tok_s_ce / cpu_tok_s_ce;

    println!(
        "CrossEntropy (tokens={tokens}, vocab={vocab}): CPU = {cpu_tok_s_ce:.0} tok/s ({cpu_dur_ce:.4}s), CUDA = {cuda_tok_s_ce:.0} tok/s ({cuda_dur_ce:.4}s) -> Speedup: {speedup_ce:.2}x"
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

#[cfg(feature = "cuda")]
#[test]
fn test_fused_read_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let q_cuda = candle_core::Tensor::from_vec(q_data, (batch, heads, time, key_dim), &cuda_dev)?;
    let k_cuda = candle_core::Tensor::from_vec(k_data, (batch, heads, time, key_dim), &cuda_dev)?;
    let v_cuda = candle_core::Tensor::from_vec(v_data, (batch, heads, time, val_dim), &cuda_dev)?;
    let aux_cuda = candle_core::Tensor::from_vec(aux_data, (1,), &cuda_dev)?;

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
    let out_cuda = uor_r4_training::geometric_stack::fused_read(
        &q_cuda,
        &k_cuda,
        &v_cuda,
        &aux_cuda,
        uor_r4_training::geometric_stack::ReadScore::Dot,
        false,
        false,
        false,
    )?;

    let c_vec = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let m_vec = out_cuda.flatten_all()?.to_vec1::<f32>()?;
    assert_eq!(c_vec.len(), m_vec.len());

    let max_diff = assert_finite_and_close(&c_vec, &m_vec, 1e-4, "FusedRead Dot");
    println!("FusedRead Dot max diff CPU vs CUDA: {max_diff}");

    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_recurrence_core_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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

    let b_cuda = candle_core::Tensor::from_vec(b_data, (batch, time, 2 * width), &cuda_dev)?;
    let g_cuda = candle_core::Tensor::from_vec(g_data, (batch, time, gate_width), &cuda_dev)?;
    let p_cuda = candle_core::Tensor::from_vec(p_data, (p_len,), &cuda_dev)?;

    let out_cpu = uor_r4_training::geometric_stack::recurrence_core(
        &b_cpu, &g_cpu, &p_cpu, batch, time, width, true, None,
    )?;
    let out_cuda = uor_r4_training::geometric_stack::recurrence_core(
        &b_cuda, &g_cuda, &p_cuda, batch, time, width, true, None,
    )?;

    let c_vec = out_cpu.flatten_all()?.to_vec1::<f32>()?;
    let m_vec = out_cuda.flatten_all()?.to_vec1::<f32>()?;
    assert_eq!(c_vec.len(), m_vec.len());

    let max_diff = assert_finite_and_close(&c_vec, &m_vec, 1e-4, "RecurrenceCore");
    println!("RecurrenceCore max diff CPU vs CUDA: {max_diff}");

    Ok(())
}

// ---------------------------------------------------------------------------
// Backward parity of the recurrence core and the general fused read.
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random values in [-scale, scale].
#[cfg(feature = "cuda")]
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
#[cfg(feature = "cuda")]
fn compare(cpu: &[f32], cuda: &[f32], tol: f32, name: &str) -> (f32, f32) {
    assert_eq!(cpu.len(), cuda.len(), "{name}: length mismatch");
    let mut max_abs = 0f32;
    let mut peak = 0f32;
    for (i, (&c, &m)) in cpu.iter().zip(cuda).enumerate() {
        assert!(
            c.is_finite() && m.is_finite(),
            "{name}[{i}]: cpu {c} cuda {m}"
        );
        max_abs = max_abs.max((c - m).abs());
        peak = peak.max(c.abs());
    }
    let relative = max_abs / peak.max(1e-6);
    println!("{name}: max abs {max_abs:.3e}, max rel {relative:.3e} (peak {peak:.3e})");
    assert!(relative < tol, "{name}: relative error {relative} >= {tol}");
    (max_abs, relative)
}

#[cfg(feature = "cuda")]
fn values(t: &candle_core::Tensor) -> uor_r4_training::Result<Vec<f32>> {
    Ok(t.flatten_all()?
        .to_device(&candle_core::Device::Cpu)?
        .to_vec1::<f32>()?)
}

/// Output and input gradients of the recurrence core for a weighted-sum loss.
#[cfg(feature = "cuda")]
#[allow(clippy::too_many_arguments)]
fn recurrence_run(
    device: &candle_core::Device,
    data: [&Vec<f32>; 4],
    batch: usize,
    time: usize,
    width: usize,
    rotation: bool,
    group: uor_r4_training::geometric_stack::RotationGroup,
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
    let out = uor_r4_training::geometric_stack::recurrence_core_grouped(
        b.as_tensor(),
        g.as_tensor(),
        p.as_tensor(),
        batch,
        time,
        width,
        rotation,
        group,
        None,
    )?;
    let grads = out.mul(&w)?.sum_all()?.backward()?;
    let mut results = vec![values(&out)?];
    for var in [&b, &g, &p] {
        results.push(values(grads.get(var.as_tensor()).expect("gradient"))?);
    }
    Ok(results)
}

#[cfg(feature = "cuda")]
#[test]
fn test_recurrence_core_backward_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;
    use uor_r4_training::geometric_stack::RotationGroup;
    // (batch, time, width, rotation, group)
    for (case, &(batch, time, width, rotation, group)) in [
        (2usize, 7usize, 16usize, true, RotationGroup::Quaternion),
        (3, 13, 32, false, RotationGroup::Quaternion),
        (2, 37, 64, true, RotationGroup::Quaternion),
        (1, 5, 4, true, RotationGroup::Quaternion),
        (2, 7, 16, true, RotationGroup::U1),
        (2, 37, 64, true, RotationGroup::U1),
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
        let cpu = recurrence_run(
            &cpu_dev, data, batch, time, width, rotation, group, gate_width,
        )?;
        let cuda = recurrence_run(
            &cuda_dev, data, batch, time, width, rotation, group, gate_width,
        )?;
        for (k, name) in ["out", "d_branches", "d_gates", "d_parameters"]
            .iter()
            .enumerate()
        {
            compare(
                &cpu[k],
                &cuda[k],
                1e-3,
                &format!("RecurrenceCore b{batch} t{time} w{width} rot{rotation} {group:?} {name}"),
            );
        }
    }
    Ok(())
}

/// The split CUDA recurrence kernels (time-parallel prep and gradient
/// kernels around a serial carry scan) equal the single-kernel path bit for
/// bit: output and all three gradients, with and without rotation, both
/// groups, lengths below, at and off the scan chunk and the training shape.
#[cfg(feature = "cuda")]
#[test]
fn test_recurrence_split_matches_single_bitwise() -> uor_r4_training::Result<()> {
    use std::time::Instant;
    use uor_r4_training::geometric_stack::{
        set_cuda_recurrence_kernels, CudaRecurrenceKernels, RotationGroup,
    };
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    // (batch, time, width, rotation, group)
    for (case, &(batch, time, width, rotation, group)) in [
        (2usize, 7usize, 16usize, true, RotationGroup::Quaternion),
        (3, 13, 32, false, RotationGroup::Quaternion),
        (2, 37, 64, true, RotationGroup::Quaternion),
        (1, 5, 4, true, RotationGroup::Quaternion),
        (2, 8, 16, true, RotationGroup::Quaternion),
        (2, 7, 16, true, RotationGroup::U1),
        (2, 37, 64, true, RotationGroup::U1),
        (16, 364, 1024, true, RotationGroup::Quaternion),
    ]
    .iter()
    .enumerate()
    {
        let lanes = width / 4;
        let gate_width = lanes + if rotation { width } else { 0 };
        let seed = 500 + case as u64 * 10;
        let b_data = noise(batch * time * 2 * width, seed, 1.0);
        let mut g_data = noise(batch * time * gate_width, seed + 1, 1.5);
        if rotation {
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
        let mut runs = Vec::new();
        for kernels in [CudaRecurrenceKernels::Single, CudaRecurrenceKernels::Split] {
            set_cuda_recurrence_kernels(kernels);
            let start = Instant::now();
            runs.push(recurrence_run(
                &cuda_dev, data, batch, time, width, rotation, group, gate_width,
            )?);
            println!(
                "b{batch} t{time} w{width} rot{rotation} {group:?} {kernels:?}: {:?}",
                start.elapsed()
            );
        }
        set_cuda_recurrence_kernels(CudaRecurrenceKernels::Split);
        for (k, name) in ["out", "d_branches", "d_gates", "d_parameters"]
            .iter()
            .enumerate()
        {
            let (single, split) = (&runs[0][k], &runs[1][k]);
            assert_eq!(single.len(), split.len(), "{name}: length mismatch");
            let differing: Vec<(usize, f32, f32)> = single
                .iter()
                .zip(split)
                .enumerate()
                .filter(|(_, (a, b))| a.to_bits() != b.to_bits())
                .map(|(i, (a, b))| (i, *a, *b))
                .collect();
            assert!(
                differing.is_empty(),
                "b{batch} t{time} w{width} rot{rotation} {group:?} {name}: {} values differ, first {:?}",
                differing.len(),
                &differing[..differing.len().min(16)]
            );
        }
    }
    Ok(())
}

/// Output and input gradients of the fused read for a weighted-sum loss.
#[cfg(feature = "cuda")]
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

#[cfg(feature = "cuda")]
#[test]
fn test_fused_read_general_parity() -> uor_r4_training::Result<()> {
    use uor_r4_training::geometric_stack::{fused_aux_len, ReadScore};
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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
        (ReadScore::L2, true, true),
        (ReadScore::L2, false, false),
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
            if score.scaled() {
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
            let cuda = read_run(&cuda_dev, data, shape, score, null, age)?;
            for (k, name) in ["out", "dq", "dk", "dv", "d_aux"].iter().enumerate() {
                compare(
                    &cpu[k],
                    &cuda[k],
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
/// on CUDA and CPU. Run explicitly: `--ignored --nocapture`.
#[cfg(feature = "cuda")]
#[test]
#[ignore]
fn bench_training_size_read_and_recurrence() -> uor_r4_training::Result<()> {
    use std::time::Instant;
    use uor_r4_training::geometric_stack::{fused_aux_len, ReadScore};
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
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
    for device in [cuda_dev.clone(), candle_core::Device::Cpu] {
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
                uor_r4_training::geometric_stack::RotationGroup::Quaternion,
                gate_width,
            )?;
            let recurrence = start.elapsed();
            println!("{device:?} round {round}: read {read:?}, recurrence {recurrence:?}");
        }
    }
    Ok(())
}

/// The recurrence forward with a GELU gate far outside the usual range: the
/// CUDA output must stay finite and equal the CPU's (a fast `tanh` once
/// returned NaN here and stopped a Metal training run at step 995; the CUDA
/// kernel keeps the same clamped accurate `tanhf`).
#[test]
fn test_recurrence_core_large_gate_parity() -> uor_r4_training::Result<()> {
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;
    let (batch, time, width) = (2, 9, 32);
    let lanes = width / 4;
    let gate_width = lanes + width;
    let scales = [50.0f32, 120.0, 2000.0];
    let b_data: Vec<f32> = (0..batch * time * 2 * width)
        .map(|i| {
            let wave = (i as f32 * 0.37).sin();
            if (i / width) % 2 == 1 {
                // The gate half of each row: large positive and negative values.
                wave * scales[i % scales.len()]
            } else {
                wave * 0.5
            }
        })
        .collect();
    let g_data: Vec<f32> = (0..batch * time * gate_width)
        .map(|i| (i as f32 * 0.02).cos() * 0.5)
        .collect();
    let p_len = (4 + 1) * width + lanes;
    let p_data: Vec<f32> = (0..p_len).map(|i| (i as f32 * 0.03).sin() * 0.05).collect();
    let run = |device: &candle_core::Device| -> uor_r4_training::Result<Vec<f32>> {
        let b = candle_core::Tensor::from_vec(b_data.clone(), (batch, time, 2 * width), device)?;
        let g = candle_core::Tensor::from_vec(g_data.clone(), (batch, time, gate_width), device)?;
        let p = candle_core::Tensor::from_vec(p_data.clone(), (p_len,), device)?;
        let out = uor_r4_training::geometric_stack::recurrence_core(
            &b, &g, &p, batch, time, width, true, None,
        )?;
        Ok(out.flatten_all()?.to_vec1::<f32>()?)
    };
    let (cpu, cuda) = (run(&cpu_dev)?, run(&cuda_dev)?);
    assert!(
        cuda.iter().all(|v| v.is_finite()),
        "CUDA output has nonfinite values"
    );
    let worst = cpu
        .iter()
        .zip(&cuda)
        .map(|(c, m)| (c - m).abs() / c.abs().max(1.0))
        .fold(0f32, f32::max);
    assert!(worst < 1e-3, "relative error {worst}");
    Ok(())
}

/// The pointer-copy mixture's loss and its logit, side and scale gradients
/// for `upstream * loss`.
#[cfg(feature = "cuda")]
#[allow(clippy::too_many_arguments)]
fn pointer_run(
    device: &candle_core::Device,
    logits: &[f32],
    side: &[f32],
    beta: f32,
    shape: (usize, usize, usize),
    score: uor_r4_training::geometric_stack::ReadScore,
    ids: &[u32],
    targets: &[u32],
    weights: Option<&[f32]>,
) -> uor_r4_training::Result<Vec<Vec<f32>>> {
    let (rows, vocab, stride) = shape;
    let z = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        logits.to_vec(),
        (rows, vocab),
        device,
    )?)?;
    let s = candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(
        side.to_vec(),
        (rows, stride),
        device,
    )?)?;
    let b =
        candle_core::Var::from_tensor(&candle_core::Tensor::from_vec(vec![beta], (1,), device)?)?;
    let time = rows / 2;
    let loss = uor_r4_training::geometric_stack::pointer_mixture_loss(
        z.as_tensor(),
        s.as_tensor(),
        b.as_tensor(),
        time,
        score,
        ids,
        targets,
        weights,
    )?;
    let grads = (&loss * 1.3)?.backward()?;
    let mut results = vec![values(&loss)?];
    for var in [&z, &s, &b] {
        results.push(values(grads.get(var.as_tensor()).expect("gradient"))?);
    }
    Ok(results)
}

/// The pointer mixture on CUDA against the CPU op: loss, logit, side
/// (query, key, gate) and scale gradients, for Dot and Lorentz scores,
/// unweighted and weighted (with a zero-weight row), with rows whose target
/// no source holds and rows served by several sources.
#[cfg(feature = "cuda")]
#[test]
fn test_pointer_mixture_parity() -> uor_r4_training::Result<()> {
    use uor_r4_training::geometric_stack::ReadScore;
    let cuda_dev = match candle_core::Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let cpu_dev = candle_core::Device::Cpu;
    // Two windows of `time` positions; (time, dim, vocabulary).
    let shapes = [(7usize, 4usize, 37usize), (40, 19, 300), (1, 3, 6)];
    let mut case = 0u64;
    for &(time, dim, vocab) in &shapes {
        let rows = 2 * time;
        let stride = 2 * dim + 1;
        // Tokens from a small alphabet so many targets repeat in the window.
        let ids: Vec<u32> = (0..rows).map(|n| ((n * 7 + n / 3) % 5) as u32).collect();
        let mut targets: Vec<u32> = (0..rows).map(|n| ids[(n + 1).min(rows - 1)]).collect();
        // Rows whose target no source holds (token vocab - 1 is never an id).
        targets[0] = (vocab - 1) as u32;
        if rows > 3 {
            targets[3] = (vocab - 1) as u32;
        }
        let mut weights = noise(rows, 77 + time as u64, 1.0)
            .iter()
            .map(|v| v.abs() + 0.1)
            .collect::<Vec<f32>>();
        weights[rows - 1] = 0.0;
        for score in [ReadScore::Dot, ReadScore::Lorentz] {
            for weighted in [false, true] {
                case += 1;
                let seed = 5000 + 31 * case;
                let logits = noise(rows * vocab, seed, 3.0);
                let side = noise(rows * stride, seed + 1, 1.2);
                let beta = 0.8f32;
                let w = weighted.then_some(weights.as_slice());
                let shape = (rows, vocab, stride);
                let cpu = pointer_run(
                    &cpu_dev, &logits, &side, beta, shape, score, &ids, &targets, w,
                )?;
                let cuda = pointer_run(
                    &cuda_dev, &logits, &side, beta, shape, score, &ids, &targets, w,
                )?;
                let label = format!("Pointer {score:?} weighted{weighted} t{time} d{dim} v{vocab}");
                compare(&cpu[0], &cuda[0], 1e-5, &format!("{label} loss"));
                compare(&cpu[1], &cuda[1], 1e-4, &format!("{label} d_logits"));
                // The query, key and gate columns separately.
                let column = |all: &[f32], range: std::ops::Range<usize>| -> Vec<f32> {
                    all.chunks(stride)
                        .flat_map(|row| row[range.clone()].to_vec())
                        .collect()
                };
                for (name, range) in [
                    ("d_query", 0..dim),
                    ("d_key", dim..2 * dim),
                    ("d_gate", 2 * dim..stride),
                ] {
                    compare(
                        &column(&cpu[2], range.clone()),
                        &column(&cuda[2], range),
                        1e-4,
                        &format!("{label} {name}"),
                    );
                }
                if score == ReadScore::Lorentz {
                    compare(&cpu[3], &cuda[3], 1e-4, &format!("{label} d_beta"));
                } else {
                    assert_eq!(cuda[3], vec![0.0], "{label}: Dot has no scale gradient");
                }
                // Row 0's target is held by no source: its query and the key
                // it reads alone (its own) get nothing from row 0, and its
                // gate gradient is c g, as on the CPU.
                assert!(
                    cuda[2][..dim].iter().all(|&v| v == 0.0),
                    "{label}: row 0 query"
                );
                if weighted {
                    let last = (rows - 1) * stride;
                    assert!(
                        cuda[2][last..last + dim].iter().all(|&v| v == 0.0)
                            && cuda[2][last + 2 * dim] == 0.0
                            && cuda[1][(rows - 1) * vocab..].iter().all(|&v| v == 0.0),
                        "{label}: the zero-weight row received gradient"
                    );
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Global gradient squared norm: the multi-block CUDA kernels against Candle's
// single-block `sqr().sum_all()` + `cat().sum_all()` (must be bit-identical)
// and against an f64 CPU sum (float tolerance).
// ---------------------------------------------------------------------------

/// Checks one tensor list: the kernel path equals Candle's bits and is
/// within float tolerance of the f64 CPU sum.
#[cfg(feature = "cuda")]
fn square_sum_case(tensors: &[candle_core::Tensor], name: &str) -> uor_r4_training::Result<()> {
    use uor_r4_training::geometric_stack::{gradient_square_sum, gradient_square_sum_reference};
    let refs: Vec<&candle_core::Tensor> = tensors.iter().collect();
    let new = gradient_square_sum(&refs)?;
    let reference = gradient_square_sum_reference(&refs)?;
    assert_eq!(
        new.to_bits(),
        reference.to_bits(),
        "{name}: kernel {new:e} != Candle {reference:e}"
    );
    let mut exact = 0f64;
    for tensor in tensors {
        for value in values(tensor)? {
            exact += f64::from(value) * f64::from(value);
        }
    }
    let relative = if exact > 0.0 {
        ((f64::from(new) - exact) / exact).abs()
    } else {
        f64::from(new).abs()
    };
    println!("{name}: {new:e} (bit-identical to Candle), f64 CPU {exact:e}, rel {relative:.2e}");
    assert!(
        relative < 1e-4,
        "{name}: relative error {relative} vs f64 CPU"
    );
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn test_gradient_square_sum_parity() -> uor_r4_training::Result<()> {
    use candle_core::{Device, Tensor};
    let dev = match Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let make = |shape: &[usize], seed: u64, scale: f32| -> uor_r4_training::Result<Tensor> {
        let len = shape.iter().product();
        Ok(Tensor::from_vec(noise(len, seed, scale), shape, &dev)?)
    };
    // Single tensors across Candle's width boundaries (1, powers of two,
    // one past 1024, the 96M model's largest variable).
    let sizes: [&[usize]; 11] = [
        &[1],
        &[3],
        &[16],
        &[1000],
        &[1024],
        &[1025],
        &[2049],
        &[703, 1024],
        &[2048, 1024],
        &[4096, 1024],
        &[7, 11, 13],
    ];
    for (i, shape) in sizes.iter().enumerate() {
        let tensor = make(shape, 50 + i as u64, 0.03)?;
        square_sum_case(&[tensor], &format!("single {shape:?}"))?;
    }
    square_sum_case(&[Tensor::new(0.25f32, &dev)?], "scalar")?;
    // Non-contiguous (transposed) and offset (narrowed) views.
    let base = make(&[37, 53], 7, 1.0)?;
    square_sum_case(&[base.t()?], "transposed 53x37")?;
    let wide = make(&[300, 64], 8, 1.0)?;
    square_sum_case(&[wide.narrow(0, 17, 200)?], "narrowed rows 17..217")?;
    square_sum_case(&[wide.narrow(1, 5, 40)?], "narrowed cols 5..45")?;
    // Tiny values whose squares are f32 subnormals or zero, and large ones.
    square_sum_case(&[make(&[5000], 9, 1e-21)?], "subnormal squares")?;
    square_sum_case(&[make(&[5000], 10, 1e16)?], "large values")?;
    // A 96M-like variable list (embedding, 14 layers of large and small
    // variables), and more than 1024 tensors (the final reduction then chains
    // within each lane).
    let mut model_like = vec![make(&[4096, 1024], 100, 0.02)?];
    for layer in 0..14u64 {
        model_like.push(make(&[2048, 1024], 200 + layer, 0.01)?);
        model_like.push(make(&[1024, 703], 300 + layer, 0.01)?);
        model_like.push(make(&[1024], 400 + layer, 0.1)?);
        for small in 0..9u64 {
            model_like.push(make(&[16 + small as usize], 500 + 16 * layer + small, 0.1)?);
        }
    }
    square_sum_case(
        &model_like,
        &format!("model-like {} tensors", model_like.len()),
    )?;
    let many: Vec<Tensor> = (0..1500u64)
        .map(|i| make(&[1 + (i as usize % 37)], 2000 + i, 0.5))
        .collect::<uor_r4_training::Result<_>>()?;
    square_sum_case(&many, "1500 small tensors")?;
    Ok(())
}

/// Wall time of the kernel and the Candle reference norm over a 96M-like
/// variable set (informative; no threshold).
#[cfg(feature = "cuda")]
#[test]
fn test_gradient_square_sum_speed() -> uor_r4_training::Result<()> {
    use candle_core::{Device, Tensor};
    use std::time::Instant;
    use uor_r4_training::geometric_stack::{gradient_square_sum, gradient_square_sum_reference};
    let dev = match Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => return no_device(e),
    };
    let mut tensors = vec![Tensor::randn(0f32, 0.02, (4096, 1024), &dev)?];
    for _ in 0..14 {
        tensors.push(Tensor::randn(0f32, 0.01, (2048, 1024), &dev)?);
        tensors.push(Tensor::randn(0f32, 0.01, (1024, 2048), &dev)?);
        tensors.push(Tensor::randn(0f32, 0.01, (1024, 703), &dev)?);
        tensors.push(Tensor::randn(0f32, 0.1, 1024, &dev)?);
    }
    let refs: Vec<&Tensor> = tensors.iter().collect();
    let elements: usize = tensors.iter().map(|t| t.elem_count()).sum();
    type Norm = fn(&[&Tensor]) -> uor_r4_training::Result<f32>;
    let runs: [(&str, Norm); 2] = [
        ("kernel", gradient_square_sum),
        ("candle", gradient_square_sum_reference),
    ];
    for (name, run) in runs {
        run(&refs)?;
        let reps = 5;
        let started = Instant::now();
        let mut last = 0f32;
        for _ in 0..reps {
            last = run(&refs)?;
        }
        let ms = started.elapsed().as_secs_f64() * 1e3 / f64::from(reps);
        println!(
            "gradient square sum {name}: {} tensors, {elements} elements, {ms:.2} ms ({last:e})",
            refs.len()
        );
    }
    Ok(())
}
