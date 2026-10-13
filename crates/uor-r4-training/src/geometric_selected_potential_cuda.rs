//! Offline selected finite utilities. Device threads build only requested
//! endpoint/root/category alternatives; no selected utility vectors are uploaded.
#![allow(unsafe_code)]
use super::{code, root, PotentialCredit};
use crate::{invalid, Result};
use candle_core::{op::BackpropOp, CudaStorage, Device, Storage, Tensor};
use cudarc::driver::{LaunchConfig, PushKernelArg};

pub(super) fn utilities(
    base: &PotentialCredit,
    slots: &[u32],
    expanded: &[i32],
    device: &Device,
) -> Result<Tensor> {
    let Device::Cuda(cuda) = device else {
        return Err(invalid("selected utility CUDA device missing"));
    };
    let count = slots.len() / 4;
    let mut relative = Vec::with_capacity(14400);
    for q in 0..120 {
        for k in 0..120 {
            relative.push(u32::from(base.algebra.relative(code(q)?, code(k)?).index()));
        }
    }
    let observations = base
        .content
        .iter()
        .zip(&base.context)
        .zip(&base.raw_roots)
        .flat_map(|((c, r), raw)| {
            [
                u32::from(c.root()),
                u32::from(c.radius_bin()),
                u32::from(c.present()),
                u32::from(r.root()),
                u32::from(r.radius_bin()),
                u32::from(r.present()),
                u32::from(*raw),
            ]
        })
        .collect::<Vec<_>>();
    let coeff = base
        .coefficients
        .iter()
        .map(|&c| f64::from(c) * 0.25)
        .collect::<Vec<_>>();
    let basis = (0..120).flat_map(root).collect::<Vec<_>>();
    let tables = expanded.iter().map(|&x| i64::from(x)).collect::<Vec<_>>();
    let s = Tensor::from_vec(slots.to_vec(), slots.len(), device)?;
    let o = Tensor::from_vec(observations.clone(), observations.len(), device)?;
    let r = Tensor::from_vec(relative, 14400, device)?;
    let c = Tensor::from_vec(coeff.clone(), coeff.len(), device)?;
    let q = Tensor::from_vec(basis, 480, device)?;
    let t = Tensor::from_vec(tables.clone(), tables.len(), device)?;
    let out = cuda.alloc_zeros::<f32>(count * 306)?;
    let (ss, _) = s.storage_and_layout();
    let (os, _) = o.storage_and_layout();
    let (rs, _) = r.storage_and_layout();
    let (cs, _) = c.storage_and_layout();
    let (qs, _) = q.storage_and_layout();
    let (ts, _) = t.storage_and_layout();
    fn storage(s: &Storage) -> Result<&CudaStorage> {
        match s {
            Storage::Cuda(x) => Ok(x),
            _ => Err(invalid("selected utility storage differs")),
        }
    }
    let fun =
        cuda.get_or_load_custom_func("selected_utilities", "uor_selected_potential", ptx()?)?;
    let mut launch = fun.builder();
    let sv = storage(&ss)?.as_cuda_slice::<u32>()?.as_view();
    let ov = storage(&os)?.as_cuda_slice::<u32>()?.as_view();
    let rv = storage(&rs)?.as_cuda_slice::<u32>()?.as_view();
    let cv = storage(&cs)?.as_cuda_slice::<f64>()?.as_view();
    let qv = storage(&qs)?.as_cuda_slice::<f64>()?.as_view();
    let tv = storage(&ts)?.as_cuda_slice::<i64>()?.as_view();
    let output = out.as_view();
    launch.arg(&sv);
    launch.arg(&ov);
    launch.arg(&rv);
    launch.arg(&cv);
    launch.arg(&qv);
    launch.arg(&tv);
    launch.arg(&output);
    let n = u32::try_from(count).map_err(|_| invalid("selected CUDA count overflow"))?;
    let width = base.d.width() as u32;
    launch.arg(&n);
    launch.arg(&width);
    // SAFETY: authenticated input dimensions allocate every indexed table and
    // slot; the kernel owns exactly 306 output cells per admitted physical slot.
    unsafe {
        launch.launch(LaunchConfig {
            grid_dim: (n.div_ceil(64), 1, 1),
            block_dim: (64, 1, 1),
            shared_mem_bytes: 0,
        })
    }
    .map_err(|e| invalid(e.to_string()))?;
    Ok(Tensor::from_storage(
        Storage::Cuda(CudaStorage::wrap_cuda_slice(out, cuda.clone())),
        (count, 306),
        BackpropOp::none(),
        false,
    ))
}
/// The ahead-of-time PTX of `selected_potential.cu` (build.rs, nvcc); NVRTC
/// is banned (owner, 13 October 2026).
fn ptx() -> Result<&'static str> {
    Ok(include_str!(concat!(
        env!("OUT_DIR"),
        "/selected_potential.ptx"
    )))
}
