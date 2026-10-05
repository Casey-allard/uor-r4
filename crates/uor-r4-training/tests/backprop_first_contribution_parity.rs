//! Parity of the vendored Candle backprop's first-contribution change
//! (`third_party/candle-core-0.9.2/UOR-PATCH.md`, second patch).
//!
//! Upstream Candle accumulates every gradient contribution as
//! `zeros_like(x) + g` on first use. The patched backprop stores the first
//! contribution directly (a contiguous copy when it is a strided or offset
//! view). `0 + g == g` exactly, so the old and new paths must agree bit for
//! bit apart from the sign of a zero. These tests compare both paths, selected
//! by `candle_core::backprop::set_upstream_zero_then_add`, on the CPU and (with
//! the `cuda` feature) on CUDA: the gradients of a graph that exercises the
//! affected backward rules, the gradients of a small pointer stack's weighted
//! loss, and the variables after three AdamW steps. A CPU-vs-CUDA comparison of
//! the stack's gradients under the new path checks the device kernels with a
//! float tolerance.
//!
//! Without a CUDA device the CUDA tests print that they were skipped; set
//! `UOR_REQUIRE_CUDA=1` to make a missing device fail them.

use candle_core::backprop::{set_upstream_zero_then_add, GradStore};
use candle_core::{DType, Device, Tensor, Var};
use uor_r4_training::geometric_stack::{
    PointerConfig, ReadScore, StackAdamW, StackArch, StackConfig, StackModel,
};

type TestResult = uor_r4_training::Result<()>;

/// The f32 bits with both zeros mapped to `+0`: the only difference the
/// first-contribution change may introduce.
fn zero_normalized_bits(values: &[f32]) -> Vec<u32> {
    values
        .iter()
        .map(|&v| if v == 0.0 { 0 } else { v.to_bits() })
        .collect()
}

fn flat(t: &Tensor) -> candle_core::Result<Vec<f32>> {
    t.to_dtype(DType::F32)?.flatten_all()?.to_vec1::<f32>()
}

/// Runs `f` with upstream's zero-fill-then-add accumulation selected or not,
/// restoring the previous selection afterwards.
fn with_upstream<R>(upstream: bool, f: impl FnOnce() -> R) -> R {
    let previous = set_upstream_zero_then_add(upstream);
    let result = f();
    set_upstream_zero_then_add(previous);
    result
}

fn noise(len: usize, seed: u64) -> Vec<f32> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            ((state >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
        })
        .collect()
}

/// A loss over two variables that reaches every first-contribution arm the
/// stack uses (and several it does not): shared `Add` inputs, `Sub`/`Neg`
/// (stored negated), `Mul`/`Div`, `Cat` (offset views), `Transpose`
/// (strided views), `Broadcast`, `Sum`/`Max` reductions (broadcast views),
/// `WhereCond`, `Reshape`, `Affine`, `Exp`, `Sqr`, `Sqrt`, `Tanh`, `Matmul`,
/// `IndexSelect` (kept on the upstream path) and `ToDType`.
fn graph_loss(a: &Var, b: &Var, device: &Device) -> candle_core::Result<Tensor> {
    let a = a.as_tensor();
    let b = b.as_tensor();
    let shared = (a + a)?; // Add with lhs == rhs.
    let diff = (a - b)?.neg()?; // Sub, Neg.
    let prod = (a * b)?.div(&(b.sqr()? + 1.0)?)?; // Mul, Div, Sqr, Affine.
    let joined = Tensor::cat(&[&shared, &diff, &prod], 0)?; // Cat: narrow views.
    let rows = joined.dims()[0];
    let transposed = joined.t()?.contiguous()?.t()?; // Transpose: strided views.
    let row_scale = b.sum_keepdim(0)?; // Sum.
    let scaled = transposed.broadcast_mul(&row_scale)?; // Broadcast.
    let peak = scaled.max_keepdim(1)?; // Max.
    let mask = scaled.ge(&scaled.zeros_like()?)?;
    let gated = mask.where_cond(&scaled, &(&scaled * 0.25)?)?; // WhereCond.
    let squashed = (gated.tanh()? + gated.exp()?.affine(0.01, 0.0)?)?;
    let projected = squashed.matmul(&b.t()?)?; // Matmul.
    let ids = Tensor::new(&[0u32, 2, 2, 1], device)?;
    let picked = a.index_select(&ids, 0)?; // IndexSelect (upstream path).
    let width = projected.dims()[1];
    let reshaped = projected.reshape((rows * width,))?;
    let soft = (b.sqr()? + 1.0)?.sqrt()?; // Sqrt.
    let half = a.to_dtype(DType::F64)?.to_dtype(DType::F32)?; // ToDType.
    reshaped.sum_all()?
        + peak.sum_all()?
        + picked.sqr()?.sum_all()?
        + soft.sum_all()?
        + half.mul(b)?.sum_all()?
}

fn graph_gradients(device: &Device) -> candle_core::Result<(Vec<Vec<f32>>, GradStore)> {
    let a = Var::from_vec(noise(3 * 5, 11), (3, 5), device)?;
    let b = Var::from_vec(noise(3 * 5, 23), (3, 5), device)?;
    let grads = graph_loss(&a, &b, device)?.backward()?;
    let mut out = Vec::new();
    for var in [&a, &b] {
        match grads.get(var.as_tensor()) {
            Some(g) => out.push(flat(g)?),
            None => candle_core::bail!("missing gradient"),
        }
    }
    Ok((out, grads))
}

fn check_graph(device: &Device) -> TestResult {
    let (old, _) = with_upstream(true, || graph_gradients(device))?;
    let (new, store) = with_upstream(false, || graph_gradients(device))?;
    for (i, (o, n)) in old.iter().zip(new.iter()).enumerate() {
        assert_eq!(
            zero_normalized_bits(o),
            zero_normalized_bits(n),
            "variable {i}: old and new backprop disagree on {device:?}"
        );
    }
    // Every stored gradient keeps upstream's layout: contiguous from offset 0.
    for id in store.get_ids() {
        if let Some(g) = store.get_id(*id) {
            assert!(
                g.is_contiguous() && g.layout().start_offset() == 0,
                "a stored gradient is a strided or offset view on {device:?}"
            );
        }
    }
    println!("graph old vs new on {device:?}: bit-identical (up to the sign of zero)");
    Ok(())
}

/// A small pointer stack in the benchmark's shape family: geometric
/// recurrence/read pattern with L2 reads, quaternion transport and a Dot
/// pointer head.
fn small_stack(device: &Device) -> uor_r4_training::Result<StackModel> {
    let mut config = StackConfig::transformer_control(7);
    config.arch = StackArch::Geometric;
    config.vocab_size = 61;
    config.width = 32;
    config.heads = 4;
    config.mlp_hidden = 40;
    config.context = 16;
    config.pattern = "rrarra".to_owned();
    config.read = ReadScore::L2;
    config.rotation = true;
    config.pointer = Some(PointerConfig::new(8));
    StackModel::new(config, device)
}

struct Batch {
    ids: Vec<u32>,
    targets: Vec<u32>,
    weights: Vec<f32>,
    batch: usize,
    time: usize,
}

fn batch(step: u64) -> Batch {
    let (batch, time) = (3, 16);
    let ids: Vec<u32> = (0..batch * time)
        .map(|i| ((i as u64 * 17 + step * 5 + (i as u64 / 7) * 3) % 61) as u32)
        .collect();
    let targets: Vec<u32> = ids.iter().map(|&i| (i * 3 + 1) % 61).collect();
    // Response-style weights: zero on a prompt prefix of each row.
    let weights: Vec<f32> = (0..batch * time)
        .map(|i| if i % time < 5 { 0.0 } else { 1.0 })
        .collect();
    Batch {
        ids,
        targets,
        weights,
        batch,
        time,
    }
}

fn stack_gradients(model: &StackModel, b: &Batch) -> uor_r4_training::Result<GradStore> {
    let loss = model.weighted_loss(&b.ids, &b.targets, &b.weights, b.batch, b.time)?;
    Ok(loss.backward()?)
}

fn named_gradients(
    model: &StackModel,
    grads: &GradStore,
) -> uor_r4_training::Result<Vec<(String, Vec<f32>)>> {
    let mut out = Vec::new();
    for (name, var) in model.variables() {
        if let Some(g) = grads.get(var.as_tensor()) {
            assert!(
                g.is_contiguous() && g.layout().start_offset() == 0,
                "{name}: stored gradient is a strided or offset view"
            );
            out.push((name.clone(), flat(g)?));
        }
    }
    Ok(out)
}

fn named_variables(model: &StackModel) -> uor_r4_training::Result<Vec<(String, Vec<f32>)>> {
    let mut out = Vec::new();
    for (name, var) in model.variables() {
        out.push((name.clone(), flat(var.as_tensor())?));
    }
    Ok(out)
}

fn assert_bitwise(
    old: &[(String, Vec<f32>)],
    new: &[(String, Vec<f32>)],
    what: &str,
    device: &Device,
) {
    assert_eq!(old.len(), new.len(), "{what}: entry counts differ");
    for ((on, ov), (nn, nv)) in old.iter().zip(new.iter()) {
        assert_eq!(on, nn);
        let (ob, nb) = (zero_normalized_bits(ov), zero_normalized_bits(nv));
        if ob != nb {
            let worst = ov
                .iter()
                .zip(nv.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0f32, f32::max);
            panic!("{what} {on} differs on {device:?}: max |old - new| = {worst}");
        }
    }
}

/// Gradients of one weighted loss, then the variables after three AdamW
/// updates with clipping, old path vs new path, bit for bit. The new path is
/// also run twice as a determinism control.
fn check_stack(device: &Device) -> TestResult {
    let run = |upstream: bool| -> uor_r4_training::Result<_> {
        with_upstream(upstream, || {
            let model = small_stack(device)?;
            let first = named_gradients(&model, &stack_gradients(&model, &batch(0))?)?;
            let mut optimizer = StackAdamW::new(&model, 0.1, 1.0)?;
            let mut norms = Vec::new();
            for step in 0..3 {
                let grads = stack_gradients(&model, &batch(step))?;
                norms.push(optimizer.update(&model, &grads, 3e-3)?);
            }
            Ok((first, named_variables(&model)?, norms))
        })
    };
    let (old_grads, old_vars, old_norms) = run(true)?;
    let (new_grads, new_vars, new_norms) = run(false)?;
    let (again_grads, again_vars, _) = run(false)?;
    assert_bitwise(&new_grads, &again_grads, "control gradient", device);
    assert_bitwise(&new_vars, &again_vars, "control variable", device);
    assert!(!new_grads.is_empty());
    assert_bitwise(&old_grads, &new_grads, "gradient", device);
    assert_bitwise(&old_vars, &new_vars, "variable after 3 AdamW steps", device);
    assert_eq!(
        old_norms.iter().map(|n| n.to_bits()).collect::<Vec<_>>(),
        new_norms.iter().map(|n| n.to_bits()).collect::<Vec<_>>(),
        "gradient norms differ on {device:?}"
    );
    println!(
        "stack old vs new on {device:?}: {} gradients and {} variables after 3 steps \
         bit-identical; grad norms {new_norms:?}",
        new_grads.len(),
        new_vars.len()
    );
    Ok(())
}

#[test]
fn graph_old_and_new_backprop_agree_bitwise_on_cpu() -> TestResult {
    check_graph(&Device::Cpu)
}

#[test]
fn stack_old_and_new_backprop_agree_bitwise_on_cpu() -> TestResult {
    check_stack(&Device::Cpu)
}

/// A missing device: a failure under `UOR_REQUIRE_CUDA=1`, otherwise a
/// printed skip (never a silent pass).
#[cfg(feature = "cuda")]
fn cuda_device() -> Option<Device> {
    match Device::new_cuda(0) {
        Ok(device) => Some(device),
        Err(error) => {
            if std::env::var("UOR_REQUIRE_CUDA").is_ok_and(|v| v == "1") {
                panic!("UOR_REQUIRE_CUDA=1 but no CUDA device: {error}");
            }
            println!("SKIPPED (no CUDA device): {error}");
            None
        }
    }
}

#[cfg(feature = "cuda")]
#[test]
fn graph_old_and_new_backprop_agree_bitwise_on_cuda() -> TestResult {
    match cuda_device() {
        Some(device) => check_graph(&device),
        None => Ok(()),
    }
}

#[cfg(feature = "cuda")]
#[test]
fn stack_old_and_new_backprop_agree_bitwise_on_cuda() -> TestResult {
    match cuda_device() {
        Some(device) => check_stack(&device),
        None => Ok(()),
    }
}

/// The new path's stack gradients on CUDA match the CPU within float
/// tolerance (different kernels and summation orders; same weights).
#[cfg(feature = "cuda")]
#[test]
fn stack_new_backprop_cuda_matches_cpu() -> TestResult {
    let Some(device) = cuda_device() else {
        return Ok(());
    };
    let gradients = |device: &Device| -> uor_r4_training::Result<_> {
        with_upstream(false, || {
            let model = small_stack(device)?;
            named_gradients(&model, &stack_gradients(&model, &batch(0))?)
        })
    };
    let cpu = gradients(&Device::Cpu)?;
    let cuda = gradients(&device)?;
    assert_eq!(cpu.len(), cuda.len());
    let mut worst = 0f32;
    for ((name, c), (_, g)) in cpu.iter().zip(cuda.iter()) {
        let scale = c.iter().fold(0f32, |m, v| m.max(v.abs())).max(1e-6);
        let diff = c
            .iter()
            .zip(g.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0f32, f32::max);
        assert!(
            g.iter().all(|v| v.is_finite()),
            "{name}: nonfinite CUDA gradient"
        );
        let relative = diff / scale;
        assert!(
            relative < 2e-3,
            "{name}: CUDA vs CPU relative max diff {relative}"
        );
        worst = worst.max(relative);
    }
    println!(
        "stack new path CUDA vs CPU: {} gradients, worst relative max diff {worst:e}",
        cpu.len()
    );
    Ok(())
}
