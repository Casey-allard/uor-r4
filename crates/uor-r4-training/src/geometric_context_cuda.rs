//! Training-only CUDA dispatch. Factual q4 choices are admitted host metadata;
//! factor scoring and all large score/basis adjoints remain on the GPU.
//! Reverse time is staged; classes/lanes are parallel and cross-lane credit is
//! reduced on device. Its reduction order differs from the ordered CPU path.
use super::*;
use crate::native_geometric_cuda_kernels::{launch, Arg};
use candle_core::backend::{BackendDevice, BackendStorage};
use candle_core::op::BackpropOp;
use candle_core::{CudaDevice, CudaStorage, Storage};
use cudarc::driver::{CudaSlice, CudaView};
type CResult<T> = candle_core::Result<T>;
fn size(n: usize) -> CResult<u32> {
    u32::try_from(n).map_err(|_| candle_core::Error::Msg("native CUDA size overflow".into()))
}
fn zeros<T: cudarc::driver::DeviceRepr + cudarc::driver::ValidAsZeroBits>(
    d: &CudaDevice,
    n: usize,
) -> CResult<CudaSlice<T>> {
    d.alloc_zeros(n.max(1))
}
fn upload<T: cudarc::driver::DeviceRepr>(d: &CudaDevice, v: &[T]) -> CResult<CudaSlice<T>> {
    d.cuda_stream()
        .clone_htod(v)
        .map_err(|e| candle_core::Error::Cuda(Box::new(e)))
}
fn input<'a>(s: &'a CudaStorage, l: &Layout) -> CResult<CudaView<'a, f32>> {
    if s.dtype() != DType::F32 || !l.is_contiguous() {
        candle_core::bail!("native CUDA requires contiguous F32 inputs");
    }
    s.as_cuda_slice::<f32>()?
        .try_slice(l.start_offset()..l.start_offset() + l.shape().elem_count())
        .ok_or_else(|| candle_core::Error::Msg("native CUDA input bounds".into()))
}
fn tensor(v: CudaSlice<f32>, d: &CudaDevice, shape: &Shape) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(CudaStorage::wrap_cuda_slice(v, d.clone())),
        shape.clone(),
        BackpropOp::none(),
        false,
    )
}
fn ready(t: &Tensor) -> CResult<Tensor> {
    if t.dtype() != DType::F32 || !t.device().is_cuda() {
        candle_core::bail!("native CUDA tensor/device mismatch");
    }
    t.force_contiguous()
}
fn view<'a>(s: &'a Storage, l: &Layout) -> CResult<CudaView<'a, f32>> {
    let Storage::Cuda(s) = s else {
        candle_core::bail!("native CUDA expected device storage");
    };
    input(s, l)
}
fn device(t: &Tensor) -> CResult<&CudaDevice> {
    let Device::Cuda(d) = t.device() else {
        candle_core::bail!("native CUDA expected CUDA device");
    };
    Ok(d)
}
fn check(d: &CudaDevice, err: &CudaSlice<u32>) -> CResult<()> {
    let v = d
        .cuda_stream()
        .clone_dtoh(err)
        .map_err(|e| candle_core::Error::Cuda(Box::new(e)))?;
    if v[0] != 0 {
        candle_core::bail!(
            "native CUDA nonfinite value or invalid native trace (code {})",
            v[0]
        );
    }
    Ok(())
}
fn finite(d: &CudaDevice, v: CudaView<'_, f32>, n: usize, err: &CudaSlice<u32>) -> CResult<()> {
    launch(
        d,
        "finite_check",
        n,
        &[Arg::F(v), Arg::U(err.as_view()), Arg::N(size(n)?)],
    )
}
fn roots_for(op: &ContextOp) -> Vec<f64> {
    (0..120).flat_map(|c| op.root(c)).collect()
}
fn products() -> Vec<u32> {
    group_table()
        .product
        .iter()
        .map(|&v| u32::from(v))
        .collect()
}
fn metadata(op: &ContextOp) -> CResult<Vec<u32>> {
    let Some(steps) = &op.native_steps else {
        return Ok(vec![0]);
    };
    if steps.len() != op.batch * op.time {
        candle_core::bail!("native CUDA trace size differs");
    }
    let mut words = Vec::with_capacity(steps.len() * 24);
    for b in 0..op.batch {
        let mut expected = [1; 8];
        for t in 0..op.time {
            if op.reset {
                expected.fill(1);
            }
            let step = &steps[b * op.time + t];
            if step.old != expected
                || step
                    .old
                    .iter()
                    .chain(&step.new)
                    .chain(&step.actions)
                    .any(|&c| c >= 120)
            {
                candle_core::bail!("native CUDA trace invalid");
            }
            for lane in 0..op.lanes() {
                if group_table().product
                    [usize::from(step.old[lane]) * ROW_STRIDE + usize::from(step.actions[lane])]
                    != step.new[lane]
                {
                    candle_core::bail!("native CUDA trace group product differs");
                }
            }
            words.extend(
                step.old
                    .iter()
                    .chain(&step.new)
                    .chain(&step.actions)
                    .map(|&x| u32::from(x)),
            );
            expected = step.new;
        }
    }
    Ok(words)
}
fn dimensions(op: &ContextOp, a: &Layout, b: &Layout) -> CResult<()> {
    let n = op.lanes();
    if op.batch == 0
        || op.time == 0
        || n == 0
        || n > 8
        || op.lanes_per_head == 0
        || a.shape().elem_count() != op.batch * op.time * n * 273
        || b.shape().elem_count() != n * 273 * 4 * (if op.lanes_per_head > 1 { 2 } else { 1 })
    {
        candle_core::bail!("native CUDA context shape differs");
    }
    Ok(())
}
pub(super) fn clip_q4(t: &Tensor) -> CResult<Tensor> {
    let t = ready(t)?;
    let d = device(&t)?;
    let (s, l) = t.storage_and_layout();
    let out = zeros::<f32>(d, t.elem_count())?;
    let err = zeros::<u32>(d, 1)?;
    launch(
        d,
        "q4_clip",
        t.elem_count(),
        &[
            Arg::F(view(&s, l)?),
            Arg::F(out.as_view()),
            Arg::U(err.as_view()),
            Arg::N(size(t.elem_count())?),
        ],
    )?;
    check(d, &err)?;
    Ok(tensor(out, d, t.shape()))
}
pub(super) fn round_q4(t: &Tensor) -> CResult<Tensor> {
    let t = ready(t)?;
    let d = device(&t)?;
    let (s, l) = t.storage_and_layout();
    let v = view(&s, l)?;
    let out = zeros::<f32>(d, t.elem_count())?;
    let err = zeros::<u32>(d, 1)?;
    launch(
        d,
        "q4_round",
        t.elem_count(),
        &[
            Arg::F(v),
            Arg::F(out.as_view()),
            Arg::U(err.as_view()),
            Arg::N(size(t.elem_count())?),
        ],
    )?;
    check(d, &err)?;
    Ok(tensor(out, d, t.shape()))
}
pub(super) fn context_forward(
    op: &ContextOp,
    a: &CudaStorage,
    la: &Layout,
    b: &CudaStorage,
    lb: &Layout,
) -> CResult<(CudaStorage, Shape)> {
    dimensions(op, la, lb)?;
    if !a.device.same_device(&b.device) {
        candle_core::bail!("native CUDA context devices differ");
    }
    let d = &a.device;
    let rows = input(a, la)?;
    let basis = input(b, lb)?;
    let q = upload(d, &roots_for(op))?;
    let group = upload(d, &products())?;
    let native = upload(d, &metadata(op)?)?;
    let trace = zeros::<u32>(d, op.batch * op.time * 24)?;
    let out = zeros::<f32>(d, op.batch * op.time * op.lanes() * PACKED_WIDTH)?;
    let err = zeros::<u32>(d, 1)?;
    finite(d, rows.slice(..), la.shape().elem_count(), &err)?;
    finite(d, basis.slice(..), lb.shape().elem_count(), &err)?;
    if op.native_steps.is_some() {
        let count = op.batch * op.time * op.lanes() * 120;
        launch(
            d,
            "native_parallel_fwd",
            count,
            &[
                Arg::F(rows),
                Arg::F(basis),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(native.as_view()),
                Arg::F(out.as_view()),
                Arg::U(err.as_view()),
                Arg::N(size(op.time)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(size(count)?),
            ],
        )?;
    } else {
        launch(
            d,
            "context_fwd",
            op.batch,
            &[
                Arg::F(rows),
                Arg::F(basis),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(native.as_view()),
                Arg::U(trace.as_view()),
                Arg::F(out.as_view()),
                Arg::U(err.as_view()),
                Arg::N(size(op.batch)?),
                Arg::N(size(op.time)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(op.reset as u32),
                Arg::N(0),
            ],
        )?;
    }
    check(d, &err)?;
    Ok((
        CudaStorage::wrap_cuda_slice(out, d.clone()),
        Shape::from((op.batch, op.time, op.lanes(), PACKED_WIDTH)),
    ))
}
pub(super) fn context_backward(
    op: &ContextOp,
    a: &Tensor,
    b: &Tensor,
    grad: &Tensor,
) -> CResult<(Option<Tensor>, Option<Tensor>)> {
    let a = ready(a)?;
    let b = ready(b)?;
    let grad = ready(grad)?;
    let d = device(&a)?;
    if !a.device().same_device(b.device()) || !a.device().same_device(grad.device()) {
        candle_core::bail!("native CUDA backward devices differ");
    }
    let (sa, la) = a.storage_and_layout();
    let (sb, lb) = b.storage_and_layout();
    let (sg, lg) = grad.storage_and_layout();
    dimensions(op, la, lb)?;
    if grad.elem_count() != op.batch * op.time * op.lanes() * PACKED_WIDTH {
        candle_core::bail!("native CUDA upstream shape differs");
    }
    let rows = view(&sa, la)?;
    let basis = view(&sb, lb)?;
    let up = view(&sg, lg)?;
    let q = upload(d, &roots_for(op))?;
    let group = upload(d, &products())?;
    let native = upload(d, &metadata(op)?)?;
    let err = zeros::<u32>(d, 1)?;
    finite(d, rows.slice(..), a.elem_count(), &err)?;
    finite(d, basis.slice(..), b.elem_count(), &err)?;
    finite(d, up.slice(..), grad.elem_count(), &err)?;
    let trace = if op.native_steps.is_some() {
        native
    } else {
        let tr = zeros::<u32>(d, op.batch * op.time * 24)?;
        let out = zeros::<f32>(d, grad.elem_count())?;
        launch(
            d,
            "context_fwd",
            op.batch,
            &[
                Arg::F(rows.slice(..)),
                Arg::F(basis.slice(..)),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(native.as_view()),
                Arg::U(tr.as_view()),
                Arg::F(out.as_view()),
                Arg::U(err.as_view()),
                Arg::N(size(op.batch)?),
                Arg::N(size(op.time)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(op.reset as u32),
                Arg::N(0),
            ],
        )?;
        tr
    };
    let dr = zeros::<f64>(d, a.elem_count())?;
    let db = zeros::<f64>(d, op.batch * b.elem_count())?;
    let count = op.batch * op.lanes() * 120;
    let con = zeros::<f64>(d, count * 8)?;
    let future = zeros::<f64>(d, op.batch * op.lanes() * 4)?;
    let direct = zeros::<f64>(d, op.batch * op.lanes() * 4)?;
    let choice = zeros::<f64>(d, count)?;
    // Distinct full120 temporal utilities; the legacy ambient4 future remains
    // untouched by this channel, preventing duplicate recurrence credit.
    let future_utility = zeros::<f64>(d, count)?;
    let utility_now = zeros::<f64>(d, count)?;
    let utility_choice = zeros::<f64>(d, count)?;
    let utility_con = zeros::<f64>(d, count * 8)?;
    let utility_basis = zeros::<f64>(d, op.batch * op.lanes() * 4)?;
    for t in (0..op.time).rev() {
        launch(
            d,
            "readout_credit",
            count,
            &[
                Arg::F(basis.slice(..)),
                Arg::F(up.slice(..)),
                Arg::D(q.as_view()),
                Arg::U(trace.as_view()),
                Arg::D(dr.as_view()),
                Arg::D(db.as_view()),
                Arg::D(con.as_view()),
                Arg::N(size(op.time)?),
                Arg::N(size(t)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(size(b.elem_count())?),
                Arg::N(size(count)?),
            ],
        )?;
        launch(
            d,
            "action_credit",
            op.batch * op.lanes(),
            &[
                Arg::F(rows.slice(..)),
                Arg::F(basis.slice(..)),
                Arg::F(up.slice(..)),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(trace.as_view()),
                Arg::D(con.as_view()),
                Arg::D(future.as_view()),
                Arg::D(direct.as_view()),
                Arg::D(choice.as_view()),
                Arg::D(future_utility.as_view()),
                Arg::D(utility_now.as_view()),
                Arg::D(utility_choice.as_view()),
                Arg::N(size(op.time)?),
                Arg::N(size(t)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(op.reset as u32),
                Arg::N(op.finite_choice as u32),
                Arg::N(size(op.batch * op.lanes())?),
            ],
        )?;
        launch(
            d,
            "transition_credit",
            count,
            &[
                Arg::F(basis.slice(..)),
                Arg::F(up.slice(..)),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(trace.as_view()),
                Arg::D(choice.as_view()),
                Arg::D(utility_choice.as_view()),
                Arg::D(dr.as_view()),
                Arg::D(db.as_view()),
                Arg::D(con.as_view()),
                Arg::D(utility_con.as_view()),
                Arg::N(size(op.time)?),
                Arg::N(size(t)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(size(b.elem_count())?),
                Arg::N(size(count)?),
            ],
        )?;
        launch(
            d,
            "old_credit",
            op.batch * op.lanes() * 4,
            &[
                Arg::D(con.as_view()),
                Arg::D(direct.as_view()),
                Arg::D(future.as_view()),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(size(op.batch * op.lanes() * 4)?),
            ],
        )?;
        launch(
            d,
            "old_utility_basis",
            op.batch * op.lanes() * 4,
            &[
                Arg::D(utility_con.as_view()),
                Arg::D(utility_basis.as_view()),
                Arg::N(size(op.lanes())?),
                Arg::N(size(op.lanes_per_head)?),
                Arg::N(size(op.batch * op.lanes() * 4)?),
            ],
        )?;
        launch(
            d,
            "old_utility_credit",
            count,
            &[
                Arg::D(utility_now.as_view()),
                Arg::D(utility_basis.as_view()),
                Arg::D(q.as_view()),
                Arg::U(group.as_view()),
                Arg::U(trace.as_view()),
                Arg::D(future_utility.as_view()),
                Arg::N(size(op.time)?),
                Arg::N(size(t)?),
                Arg::N(size(op.lanes())?),
                Arg::N(size(count)?),
            ],
        )?;
    }
    let da = zeros::<f32>(d, a.elem_count())?;
    let dbout = zeros::<f32>(d, b.elem_count())?;
    launch(
        d,
        "finish",
        a.elem_count(),
        &[
            Arg::D(dr.as_view()),
            Arg::F(da.as_view()),
            Arg::U(err.as_view()),
            Arg::N(size(a.elem_count())?),
            Arg::N(1),
        ],
    )?;
    launch(
        d,
        "finish",
        b.elem_count(),
        &[
            Arg::D(db.as_view()),
            Arg::F(dbout.as_view()),
            Arg::U(err.as_view()),
            Arg::N(size(b.elem_count())?),
            Arg::N(size(op.batch)?),
        ],
    )?;
    check(d, &err)?;
    Ok((
        Some(tensor(da, d, a.shape())),
        Some(tensor(dbout, d, b.shape())),
    ))
}
fn emit_run(
    op: &EmitOp,
    a: &CudaStorage,
    la: &Layout,
    b: &CudaStorage,
    lb: &Layout,
    up: Option<CudaView<'_, f32>>,
) -> CResult<(CudaSlice<f32>, CudaSlice<f32>, CudaSlice<f32>)> {
    let n = op.batch * op.time * op.lanes;
    if la.shape().elem_count() != n * 120
        || lb.shape().elem_count() != n * 33
        || !a.device.same_device(&b.device)
    {
        candle_core::bail!("native CUDA emission shape/device differs");
    }
    let d = &a.device;
    let r = input(a, la)?;
    let c = input(b, lb)?;
    let q = upload(
        d,
        &roots()
            .iter()
            .flat_map(|r| r.iter().copied())
            .collect::<Vec<_>>(),
    )?;
    let out = zeros::<f32>(d, n * 4)?;
    let dr = zeros::<f32>(d, n * 120)?;
    let dc = zeros::<f32>(d, n * 33)?;
    let err = zeros::<u32>(d, 1)?;
    finite(d, r.slice(..), n * 120, &err)?;
    finite(d, c.slice(..), n * 33, &err)?;
    let backward = up.is_some();
    if let Some(v) = &up {
        finite(d, v.slice(..), n * 4, &err)?;
    }
    launch(
        d,
        "emit",
        n,
        &[
            Arg::F(r),
            Arg::F(c),
            Arg::F(up.unwrap_or_else(|| out.as_view())),
            Arg::D(q.as_view()),
            Arg::F(out.as_view()),
            Arg::F(dr.as_view()),
            Arg::F(dc.as_view()),
            Arg::U(err.as_view()),
            Arg::N(size(n)?),
            Arg::N(backward as u32),
        ],
    )?;
    check(d, &err)?;
    Ok((out, dr, dc))
}
pub(super) fn emit_forward(
    op: &EmitOp,
    a: &CudaStorage,
    la: &Layout,
    b: &CudaStorage,
    lb: &Layout,
) -> CResult<(CudaStorage, Shape)> {
    let (out, _, _) = emit_run(op, a, la, b, lb, None)?;
    Ok((
        CudaStorage::wrap_cuda_slice(out, a.device.clone()),
        Shape::from((op.batch, op.time, op.lanes * 4)),
    ))
}
pub(super) fn emit_backward(
    op: &EmitOp,
    a: &Tensor,
    b: &Tensor,
    grad: &Tensor,
) -> CResult<(Option<Tensor>, Option<Tensor>)> {
    let a = ready(a)?;
    let b = ready(b)?;
    let grad = ready(grad)?;
    if !a.device().same_device(b.device())
        || !a.device().same_device(grad.device())
        || grad.elem_count() != op.batch * op.time * op.lanes * 4
    {
        candle_core::bail!("native CUDA emission upstream differs");
    }
    let (sa, la) = a.storage_and_layout();
    let (sb, lb) = b.storage_and_layout();
    let (sg, lg) = grad.storage_and_layout();
    let (Storage::Cuda(sa), Storage::Cuda(sb)) = (&*sa, &*sb) else {
        candle_core::bail!("native CUDA emission storage differs");
    };
    let (_, dr, dc) = emit_run(op, sa, la, sb, lb, Some(view(&sg, lg)?))?;
    Ok((
        Some(tensor(dr, &sa.device, a.shape())),
        Some(tensor(dc, &sa.device, b.shape())),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: &Tensor, b: &Tensor, abs: f32, rel: f32) -> CResult<()> {
        let x = a.flatten_all()?.to_vec1::<f32>()?;
        let y = b.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(x.len(), y.len());
        for (i, (&a, &b)) in x.iter().zip(&y).enumerate() {
            assert!(
                a.is_finite() && b.is_finite() && (a - b).abs() <= abs + rel * a.abs().max(b.abs()),
                "CUDA parity {i}: {a} vs {b}"
            );
        }
        Ok(())
    }
    // Feature-gated but never runtime-skipped: requesting this CUDA test on a
    // machine without CUDA returns an error, not a claimed parity pass.
    #[test]
    fn native_context_cuda_q4_full120_forward_backward_parity() -> Result<()> {
        let device = Device::new_cuda(0)?;
        for lanes in [1, 2, 4] {
            for reset in [false, true] {
                let cpu = ContextWeights::new_finite_choice(7, 8 * lanes, 2, 991)?.into_q4()?;
                for (family, (_, v)) in cpu.parameters.iter().enumerate() {
                    let values = (0..v.elem_count())
                        .map(|i| (((i * 17 + family * 11) % 15) as i32 - 7) as f32 * 0.25)
                        .collect::<Vec<_>>();
                    v.set(&Tensor::from_vec(values, v.shape(), &Device::Cpu)?)?;
                }
                let gpu = cpu.to_device(&device)?;
                assert_eq!(cpu.packed_coefficients()?, gpu.packed_coefficients()?);
                let codec = uor_r4_integer::geometric_context_q4::NativeContextQ4::new(
                    cpu.q4_config(),
                    &cpu.packed_coefficients()?,
                )
                .map_err(|e| invalid(e.to_string()))?;
                let pc = cpu.prepare_q4(&codec)?;
                let pg = gpu.prepare_q4(&codec)?;
                let ids = [0, 3, 1, 5, 6, 2];
                let a = pc.forward(&ids, 2, 3, reset)?;
                let b = pg.forward(&ids, 2, 3, reset)?;
                assert_eq!(a.trace.states, b.trace.states);
                assert_eq!(a.trace.actions, b.trace.actions);
                assert_eq!(a.trace.codes, b.trace.codes);
                let prefix = pg.forward(&ids[..1], 1, 1, reset)?;
                close(
                    &b.state_logits.narrow(0, 0, 1)?.narrow(1, 0, 1)?,
                    &prefix.state_logits,
                    0.,
                    0.,
                )?;
                assert_eq!(b.trace.states[0], prefix.trace.states[0]);

                for (x, y) in [
                    (&a.state_logits, &b.state_logits),
                    (&a.state_choices, &b.state_choices),
                    (&a.root_logits, &b.root_logits),
                    (&a.category_logits, &b.category_logits),
                    (&a.latent_roots, &b.latent_roots),
                ] {
                    close(x, y, 0., 0.)?;
                    assert!(y.device().is_cuda());
                }
                // Direct120 adjoints include an even quadratic component that a
                // four-coordinate state contraction cannot represent.
                let objective = |o: &ContextQ4Output, d: &Device| -> Result<Tensor> {
                    let state = (0..o.state_logits.elem_count())
                        .map(|i| {
                            let q = q4_roots()[i % 120];
                            (q[0] * q[0] - q[1] * q[1]) as f32
                        })
                        .collect::<Vec<_>>();
                    let state = Tensor::from_vec(state, o.state_logits.shape(), d)?;
                    Ok((((o.state_logits.mul(&state)?.sum_all()?
                        + o.root_logits.sqr()?.sum_all()?.affine(0.031, 0.)?)?
                        + o.category_logits.sqr()?.sum_all()?.affine(0.071, 0.)?)?
                        + o.latent_roots.sum_all()?.affine(0.19, 0.)?)?)
                };
                let gc = objective(&a, &Device::Cpu)?.backward()?;
                let gg = objective(&b, &device)?.backward()?;
                for (name, v) in &cpu.parameters {
                    let x = gc
                        .get(v)
                        .ok_or_else(|| invalid(format!("CPU missing gradient {name}")))?;
                    let y = gg
                        .get(&gpu.parameters[name])
                        .ok_or_else(|| invalid(format!("CUDA missing gradient {name}")))?;
                    assert!(y.device().is_cuda());
                    close(x, y, 2e-4, 2e-5)?;
                }
            }
        }
        Ok(())
    }
    /// Final even-harmonic utility must reach the earlier token through the
    /// full120 factual-action permutation, including cross-lane score factors.
    /// The reset control cuts that temporal path without changing local credit.
    #[test]
    fn native_context_cuda_two_token_choice_utility_carry_parity() -> Result<()> {
        let cuda = Device::new_cuda(0)?;
        for lanes in [1, 2] {
            for reset in [false, true] {
                let mut first = [1; 8];
                first[0] = 3;
                if lanes == 2 {
                    first[1] = 7;
                }
                let old = if reset { [1; 8] } else { first };
                let mut actions = [1; 8];
                actions[0] = 7;
                if lanes == 2 {
                    actions[1] = 9;
                }
                let mut post = [1; 8];
                for lane in 0..lanes {
                    post[lane] = group_table().product
                        [usize::from(old[lane]) * ROW_STRIDE + usize::from(actions[lane])];
                }
                let op = ContextOp {
                    batch: 1,
                    time: 2,
                    heads: 1,
                    lanes_per_head: lanes,
                    reset,
                    finite_choice: true,
                    native_steps: Some(Arc::new(vec![
                        Step {
                            old: [1; 8],
                            new: first,
                            actions: first,
                        },
                        Step {
                            old,
                            new: post,
                            actions,
                        },
                    ])),
                };
                let rows = (0..2 * lanes * 273)
                    .map(|i| ((i * 7 % 9) as i32 - 4) as f32 * 0.25)
                    .collect::<Vec<_>>();
                let basis = (0..lanes * 273 * 4 * if lanes > 1 { 2 } else { 1 })
                    .map(|i| ((i * 11 % 3) as i32 - 1) as f32 * 0.25)
                    .collect::<Vec<_>>();
                let cr = Var::from_vec(rows.clone(), (1, 2, lanes * 273), &Device::Cpu)?;
                let cb = Var::from_vec(basis.clone(), basis.len(), &Device::Cpu)?;
                let gr = Var::from_vec(rows, (1, 2, lanes * 273), &cuda)?;
                let gb = Var::from_vec(basis.clone(), basis.len(), &cuda)?;
                let make_op = || ContextOp {
                    batch: op.batch,
                    time: op.time,
                    heads: op.heads,
                    lanes_per_head: op.lanes_per_head,
                    reset: op.reset,
                    finite_choice: op.finite_choice,
                    native_steps: op.native_steps.clone(),
                };
                let cp = cr.apply_op2(cb.as_tensor(), make_op())?;
                let gp = gr.apply_op2(gb.as_tensor(), make_op())?;
                close(&cp, &gp, 0., 0.)?;
                let mut upstream = vec![0f32; 2 * lanes * PACKED_WIDTH];
                for lane in 0..lanes {
                    for root in 0..120 {
                        let q = q4_roots()[root];
                        upstream[(lanes + lane) * PACKED_WIDTH + STATE_CHOICE_OFFSET + root] =
                            ((lane + 1) as f64 * (q[0] * q[0] - q[1] * q[1])) as f32;
                    }
                }
                let cu = Tensor::from_vec(upstream.clone(), cp.shape(), &Device::Cpu)?;
                let gu = Tensor::from_vec(upstream, gp.shape(), &cuda)?;
                let cg = cp.mul(&cu)?.sum_all()?.backward()?;
                let gg = gp.mul(&gu)?.sum_all()?.backward()?;
                let dx = cg
                    .get(&cr)
                    .ok_or_else(|| invalid("CPU choice token gradient"))?;
                let dg = gg
                    .get(&gr)
                    .ok_or_else(|| invalid("CUDA choice token gradient"))?;
                close(dx, dg, 2e-5, 2e-5)?;
                close(
                    cg.get(&cb)
                        .ok_or_else(|| invalid("CPU choice basis gradient"))?,
                    gg.get(&gb)
                        .ok_or_else(|| invalid("CUDA choice basis gradient"))?,
                    2e-5,
                    2e-5,
                )?;
                let early = dx.narrow(1, 0, 1)?.sqr()?.sum_all()?.to_scalar::<f32>()?;
                let gpu_early = dg.narrow(1, 0, 1)?.sqr()?.sum_all()?.to_scalar::<f32>()?;
                if reset {
                    assert_eq!(early, 0.);
                    assert_eq!(gpu_early, 0.);
                } else {
                    assert!(early > 0.);
                    assert!(gpu_early > 0.);
                }
                // The new forward is genuinely hardonehot, not an extra softmax.
                let onehot = gp
                    .narrow(3, STATE_CHOICE_OFFSET, 120)?
                    .to_device(&Device::Cpu)?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for row in onehot.chunks_exact(120) {
                    assert_eq!(row.iter().filter(|&&v| v == 1.).count(), 1);
                    assert!(row.iter().all(|&v| v == 0. || v == 1.));
                }
            }
        }
        Ok(())
    }
    #[test]
    fn native_context_cuda_legacy_emit_and_q4_rounding_parity() -> Result<()> {
        let d = Device::new_cuda(0)?;
        for finite in [false, true] {
            let cpu = if finite {
                ContextWeights::new_finite_choice(4, 8, 1, 111)?
            } else {
                ContextWeights::new(4, 8, 1, 111)?
            };
            let gpu = cpu.to_device(&d)?;
            let a = cpu.forward(&[0, 1, 2, 3], 2, 2, false)?;
            let b = gpu.forward(&[0, 1, 2, 3], 2, 2, false)?;
            close(&a.context, &b.context, 0., 0.)?;
            close(&a.state_logits, &b.state_logits, 0., 0.)?;
            let gc = a.context.sqr()?.sum_all()?.backward()?;
            let gg = b.context.sqr()?.sum_all()?.backward()?;
            for (name, v) in &cpu.parameters {
                close(
                    gc.get(v).ok_or_else(|| invalid("CPU emit gradient"))?,
                    gg.get(&gpu.parameters[name])
                        .ok_or_else(|| invalid("CUDA emit gradient"))?,
                    2e-5,
                    2e-5,
                )?;
            }
        }
        let raw = [-1.625f32, -0.375, -0.125, 0., 0.125, 0.375, 1.625];
        let t = Var::from_slice(&raw, raw.len(), &d)?;
        let rounded = round_q4(t.as_tensor())?;
        assert_eq!(
            rounded.to_vec1::<f32>()?,
            raw.map(|x| (x * 4.).round() * 0.25)
        );
        let ste = (&rounded + (t.as_tensor() - t.as_tensor().detach())?)?;
        let g = ste.sum_all()?.backward()?;
        assert_eq!(
            g.get(&t)
                .ok_or_else(|| invalid("CUDA STE gradient"))?
                .to_vec1::<f32>()?,
            vec![1.; raw.len()]
        );
        let bad = Tensor::from_vec(vec![f32::NAN], 1, &d)?;
        assert!(round_q4(&bad).is_err());
        Ok(())
    }
    #[test]
    fn native_context_cuda_full120_direct_adjoint_and_emit_absence() -> Result<()> {
        let gpu = Device::new_cuda(0)?;
        let mut old = [1; 8];
        old[0] = 31;
        old[1] = 57;
        let mut new = old;
        let mut actions = [1; 8];
        actions[0] = 9;
        actions[1] = 41;
        for lane in 0..2 {
            new[lane] = group_table().product
                [usize::from(old[lane]) * ROW_STRIDE + usize::from(actions[lane])];
        }
        // A one-step exact native trace must start at identity. Force nontrivial
        // old state through the first step, then inspect second-step direct credit.
        let first = Step {
            old: [1; 8],
            new: old,
            actions: old,
        };
        let steps = Arc::new(vec![first, Step { old, new, actions }]);
        let rows_cpu = Var::zeros((1, 2, 546), DType::F32, &Device::Cpu)?;
        let basis_cpu = Var::zeros(546 * 8, DType::F32, &Device::Cpu)?;
        let rows_gpu = Var::zeros((1, 2, 546), DType::F32, &gpu)?;
        let basis_gpu = Var::zeros(546 * 8, DType::F32, &gpu)?;
        let op = || ContextOp {
            batch: 1,
            time: 2,
            heads: 1,
            lanes_per_head: 2,
            reset: false,
            finite_choice: true,
            native_steps: Some(steps.clone()),
        };
        let a = rows_cpu.apply_op2(&basis_cpu, op())?;
        let b = rows_gpu.apply_op2(&basis_gpu, op())?;
        let mut weights = vec![0f32; 1 * 2 * 2 * 120];
        for lane in 0..2 {
            for c in 0..120 {
                let q = q4_roots()[c];
                weights[(2 + lane) * 120 + c] = (q[0] * q[0] - q[1] * q[1]) as f32;
            }
        }
        let loss = |out: &Tensor, device: &Device| -> Result<Tensor> {
            let w = Tensor::from_vec(weights.clone(), (1, 2, 2, 120), device)?;
            Ok(out
                .narrow(3, STATE_LOGITS_OFFSET, 120)?
                .mul(&w)?
                .sum_all()?)
        };
        let gc = loss(&a, &Device::Cpu)?.backward()?;
        let gg = loss(&b, &gpu)?.backward()?;
        let dx = gc
            .get(&rows_cpu)
            .ok_or_else(|| invalid("CPU direct120 gradient"))?;
        let dy = gg
            .get(&rows_gpu)
            .ok_or_else(|| invalid("CUDA direct120 gradient"))?;
        close(dx, dy, 0., 0.)?;
        let actual = dy.flatten_all()?.to_vec1::<f32>()?;
        for lane in 0..2 {
            for action in 0..120 {
                let post =
                    group_table().product[usize::from(old[lane]) * ROW_STRIDE + action] as usize;
                assert_eq!(
                    actual[546 + lane * 120 + action],
                    weights[(2 + lane) * 120 + post]
                );
            }
        }
        assert!(actual.iter().any(|&v| v.abs() > 0.5));
        close(
            gc.get(&basis_cpu)
                .ok_or_else(|| invalid("CPU direct basis"))?,
            gg.get(&basis_gpu)
                .ok_or_else(|| invalid("CUDA direct basis"))?,
            1e-6,
            1e-6,
        )?;
        for selected in [0, 1, 17, 32] {
            let r = vec![0f32; 120];
            let mut c = vec![0f32; 33];
            c[selected] = 1.;
            let cr = Var::from_vec(r.clone(), (1, 1, 1, 120), &Device::Cpu)?;
            let cc = Var::from_vec(c.clone(), (1, 1, 1, 33), &Device::Cpu)?;
            let gr = Var::from_vec(r, (1, 1, 1, 120), &gpu)?;
            let gcat = Var::from_vec(c, (1, 1, 1, 33), &gpu)?;
            let a = cr.apply_op2(
                &cc,
                EmitOp {
                    batch: 1,
                    time: 1,
                    lanes: 1,
                },
            )?;
            let b = gr.apply_op2(
                &gcat,
                EmitOp {
                    batch: 1,
                    time: 1,
                    lanes: 1,
                },
            )?;
            close(&a, &b, 0., 0.)?;
            let x = a.sum_all()?.backward()?;
            let y = b.sum_all()?.backward()?;
            close(
                x.get(&cr).ok_or_else(|| invalid("CPU root emit adjoint"))?,
                y.get(&gr)
                    .ok_or_else(|| invalid("CUDA root emit adjoint"))?,
                1e-4,
                2e-5,
            )?;
            close(
                x.get(&cc)
                    .ok_or_else(|| invalid("CPU category emit adjoint"))?,
                y.get(&gcat)
                    .ok_or_else(|| invalid("CUDA category emit adjoint"))?,
                1e-4,
                2e-5,
            )?;
            if selected == 0 {
                assert!(y
                    .get(&gr)
                    .ok_or_else(|| invalid("absent root"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .iter()
                    .all(|&g| g == 0.));
            }
        }
        Ok(())
    }
}
