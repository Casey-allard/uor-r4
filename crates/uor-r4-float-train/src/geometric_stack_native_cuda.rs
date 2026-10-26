//! Exact native CUDA calls extracted from the historical stack CUDA module.
//! Unsupported layouts/dtypes refuse; no host/Metal fallback. The qualified
//! stack is F32; the existing FBS controls retain their exact native BF16 calls.
use super::*;
use crate::cuda_stack_kernels::cuda::{launch, launch_groups, uninit, zeros, Arg};
use candle_core::backend::BackendStorage;
use candle_core::op::BackpropOp;
use candle_core::{CudaDevice, CudaStorage, Storage};
use cudarc::driver::{CudaSlice, CudaView};
type CResult<T> = candle_core::Result<T>;
type Forward = CResult<(CudaStorage, Shape)>;
const ROW_THREADS: usize = 256;
const RMS_CHUNK_ROWS: usize = 256;
fn input<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<Option<CudaView<'a, f32>>> {
    if !layout.is_contiguous() || storage.dtype() != DType::F32 {
        return Ok(None);
    }
    let slice = storage.as_cuda_slice::<f32>()?;
    let start = layout.start_offset();
    let len = layout.shape().elem_count();
    match slice.try_slice(start..start + len) {
        Some(view) => Ok(Some(view)),
        None => candle_core::bail!("CUDA input layout exceeds its buffer"),
    }
}
fn view<'a>(storage: &'a Storage, layout: &Layout) -> CResult<CudaView<'a, f32>> {
    let Storage::Cuda(storage) = storage else {
        candle_core::bail!("expected a CUDA tensor");
    };
    match input(storage, layout)? {
        Some(view) => Ok(view),
        None => candle_core::bail!("CUDA stack kernels need contiguous F32 tensors"),
    }
}
fn ready(tensor: &Tensor) -> CResult<Tensor> {
    if tensor.dtype() != DType::F32 {
        candle_core::bail!("CUDA stack kernels require F32 tensors");
    }
    if tensor.is_contiguous() {
        Ok(tensor.clone())
    } else {
        tensor.force_contiguous()
    }
}
fn storage(slice: CudaSlice<f32>, device: &CudaDevice) -> CudaStorage {
    CudaStorage::wrap_cuda_slice(slice, device.clone())
}
fn tensor(slice: CudaSlice<f32>, device: &CudaDevice, shape: &Shape) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(storage(slice, device)),
        shape.clone(),
        BackpropOp::none(),
        false,
    )
}
fn u32_of(value: usize, what: &str) -> CResult<u32> {
    u32::try_from(value).map_err(|_| candle_core::Error::Msg(format!("CUDA {what} too large")))
}
pub(crate) fn raw_hamilton_gate_fwd(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    if l1.shape() != l2.shape() {
        candle_core::bail!("Hamilton gate inputs must match");
    }
    let (Some(gate), Some(up)) = (input(s1, l1)?, input(s2, l2)?) else {
        candle_core::bail!("the Hamilton gate needs contiguous F32 tensors");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    if total % 4 != 0 {
        candle_core::bail!("the Hamilton gate rotates 4-blocks");
    }
    let out = uninit::<f32>(device, total)?;
    launch(
        device,
        "hamilton_gate_fwd",
        total / 4,
        &[
            Arg::F(gate),
            Arg::F(up),
            Arg::f(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((storage(out, device), l1.shape().clone()))
}
pub(super) fn hamilton_gate_fwd(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    if let Some(res) = crate::hamilton_sheet::try_hamilton_gate_fwd_cuda(s1, l1, s2, l2)? {
        return Ok(res);
    }
    raw_hamilton_gate_fwd(s1, l1, s2, l2)
}
pub(crate) fn raw_hamilton_gate_bwd(
    device: &CudaDevice,
    gate: &Tensor,
    up: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    if gate.shape() != up.shape() || grad.shape() != gate.shape() {
        candle_core::bail!("CUDA Hamilton gate backward inputs must match");
    }
    let (g, u, d) = (ready(gate)?, ready(up)?, ready(grad)?);
    let total = g.elem_count();
    if total % 4 != 0 {
        candle_core::bail!("the Hamilton gate rotates 4-blocks");
    }
    let (gs, gl) = g.storage_and_layout();
    let (us, ul) = u.storage_and_layout();
    let (ds, dl) = d.storage_and_layout();
    let d_gate = uninit::<f32>(device, total)?;
    let d_up = uninit::<f32>(device, total)?;
    launch(
        device,
        "hamilton_gate_bwd",
        total / 4,
        &[
            Arg::F(view(&gs, gl)?),
            Arg::F(view(&us, ul)?),
            Arg::F(view(&ds, dl)?),
            Arg::f(&d_gate),
            Arg::f(&d_up),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((
        tensor(d_gate, device, gate.shape()),
        tensor(d_up, device, up.shape()),
    ))
}
pub(super) fn hamilton_gate_bwd(
    device: &CudaDevice,
    gate: &Tensor,
    up: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    if let Some(res) = crate::hamilton_sheet::try_hamilton_gate_bwd_cuda(device, gate, up, grad)? {
        return Ok(res);
    }
    raw_hamilton_gate_bwd(device, gate, up, grad)
}
fn spiralcore_fbs_device_plan(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    device: &CudaDevice,
) -> CResult<Arc<SpiralcoreFbsDevicePlan>> {
    type Cache = std::collections::HashMap<String, Arc<SpiralcoreFbsDevicePlan>>;
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Cache>> = std::sync::OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| candle_core::Error::Msg("native projection metadata cache poisoned".into()))?;
    let key = format!("{:?}/{}", device.id(), plan.fingerprint);
    if let Some(value) = cache.get(&key) {
        return Ok(Arc::clone(value));
    }
    let g = &plan.geometry;
    let roots: Vec<f32> = g.roots.iter().flatten().map(|&x| f32::from(x)).collect();
    let buffers = Arc::new(SpiralcoreFbsDevicePlan {
        nodes: device.clone_htod(&plan.node_words())?,
        frontier: device.clone_htod(&plan.frontier)?,
        axes: device.clone_htod(&g.axes)?,
        compose: device.clone_htod(&g.compose)?,
        root_of_state: device.clone_htod(&g.root_of_state)?,
        bfs_actions: device.clone_htod(&g.bfs_actions)?,
        branch_targets: device.clone_htod(&g.branch_targets)?,
        roots: device.clone_htod(&roots)?,
    });
    cache.insert(key, Arc::clone(&buffers));
    Ok(buffers)
}
fn spiralcore_fbs_rows(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    q: &[usize],
    x: &[usize],
    elements: usize,
) -> CResult<usize> {
    if q != plan.query_shape() || x.last() != Some(&(plan.input_blocks * 8)) || elements == 0 {
        candle_core::bail!("native projection buffers disagree with their static plan");
    }
    let rows = elements / (plan.input_blocks * 8);
    let threads = rows
        .checked_mul(plan.output_blocks)
        .ok_or_else(|| candle_core::Error::Msg("native projection launch size overflow".into()))?;
    u32_of(threads, "native projection carriers")?;
    Ok(rows)
}
fn spiralcore_fbs_activation<'a>(s: &'a CudaStorage, l: &Layout) -> CResult<Arg<'a>> {
    if !l.is_contiguous() {
        candle_core::bail!("native projection needs contiguous carriers; no host fallback");
    }
    let range = l.start_offset()..l.start_offset() + l.shape().elem_count();
    match s.dtype() {
        DType::F32 => Ok(Arg::F(
            s.as_cuda_slice::<f32>()?.try_slice(range).ok_or_else(|| {
                candle_core::Error::Msg("carrier range outside CUDA buffer".into())
            })?,
        )),
        DType::BF16 => Ok(Arg::B(
            s.as_cuda_slice::<half::bf16>()?
                .try_slice(range)
                .ok_or_else(|| {
                    candle_core::Error::Msg("carrier range outside CUDA buffer".into())
                })?,
        )),
        _ => candle_core::bail!("native carriers must be F32 or BF16"),
    }
}
fn spiralcore_fbs_metadata<'a>(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    b: &'a SpiralcoreFbsDevicePlan,
) -> Vec<Arg<'a>> {
    let _ = plan;
    vec![
        Arg::u(&b.nodes),
        Arg::u(&b.frontier),
        Arg::u(&b.axes),
        Arg::u(&b.compose),
        Arg::u(&b.root_of_state),
        Arg::u(&b.bfs_actions),
        Arg::u(&b.branch_targets),
        Arg::f(&b.roots),
    ]
}
fn spiralcore_fbs_dimensions(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    rows: usize,
) -> CResult<Vec<Arg<'static>>> {
    Ok(vec![
        Arg::U32(u32_of(rows, "rows")?),
        Arg::U32(u32_of(plan.output_blocks, "outputs")?),
        Arg::U32(u32_of(plan.input_blocks, "inputs")?),
        Arg::U32(u32_of(plan.stages, "stages")?),
        Arg::U32(u32_of(plan.frontier.len(), "frontier")?),
        Arg::U32(plan.geometry.identity),
    ])
}
fn spiralcore_fbs_device_error(device: &CudaDevice, flag: &CudaSlice<u32>) -> CResult<()> {
    // One device validation code; no model scoring/normalization on the host.
    let values = device.clone_dtoh(flag)?;
    if values.first().copied() != Some(0) {
        candle_core::bail!(
            "native SpiralCore/FBS projection rejected non-finite data or invalid route: {:?}",
            values
        );
    }
    crate::secant_descent::record_cuda_error_check_ok();
    crate::hamilton_sheet::record_cuda_error_check_ok();
    Ok(())
}
pub(crate) fn spiralcore_fbs_fwd(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    q: &CudaStorage,
    ql: &Layout,
    x: &CudaStorage,
    xl: &Layout,
) -> Forward {
    if q.device.id() != x.device.id() {
        candle_core::bail!("native routing points and carriers need the same CUDA context");
    }
    let rows = spiralcore_fbs_rows(
        plan,
        ql.shape().dims(),
        xl.shape().dims(),
        xl.shape().elem_count(),
    )?;
    let Some(query) = input(q, ql)? else {
        candle_core::bail!("native queries need contiguous F32 masters");
    };
    let device = &q.device;
    let metadata = spiralcore_fbs_device_plan(plan, device)?;
    let error = zeros::<u32>(device, 1)?;
    let threads = rows * plan.output_blocks;
    let mut dims = xl.shape().dims().to_vec();
    let last = dims.len() - 1;
    dims[last] = 8 * plan.output_blocks;
    let mut args = vec![Arg::F(query), spiralcore_fbs_activation(x, xl)?];
    args.extend(spiralcore_fbs_metadata(plan, &metadata));
    let storage = match x.dtype() {
        DType::F32 => {
            let out = uninit::<f32>(device, threads * 8)?;
            args.push(Arg::f(&out));
            args.push(Arg::u(&error));
            args.extend(spiralcore_fbs_dimensions(plan, rows)?);
            launch(device, "spiralcore_fbs_fwd", threads, &args)?;
            drop(args);
            storage(out, device)
        }
        DType::BF16 => {
            let out = uninit::<half::bf16>(device, threads * 8)?;
            args.push(Arg::b(&out));
            args.push(Arg::u(&error));
            args.extend(spiralcore_fbs_dimensions(plan, rows)?);
            crate::cuda_stack_kernels::cuda::launch_bf16(
                device,
                "spiralcore_fbs_fwd",
                threads,
                &args,
            )?;
            drop(args);
            CudaStorage::wrap_cuda_slice(out, device.clone())
        }
        _ => candle_core::bail!("native carriers must be F32 or BF16"),
    };
    spiralcore_fbs_device_error(device, &error)?;
    Ok((storage, Shape::from_dims(&dims)))
}
pub(crate) fn spiralcore_fbs_bwd(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    device: &CudaDevice,
    q: &Tensor,
    x: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    let (q, x, grad) = (q.contiguous()?, x.contiguous()?, grad.contiguous()?);
    let rows = spiralcore_fbs_rows(plan, q.dims(), x.dims(), x.elem_count())?;
    let mut output_dims = x.dims().to_vec();
    let last = output_dims.len() - 1;
    output_dims[last] = plan.output_blocks * 8;
    if grad.dims() != output_dims || grad.dtype() != x.dtype() {
        candle_core::bail!("native projection gradient shape/dtype differs from its output");
    }
    let (qs, ql) = q.storage_and_layout();
    let (xs, xl) = x.storage_and_layout();
    let (gs, gl) = grad.storage_and_layout();
    let (Storage::Cuda(qs), Storage::Cuda(xs), Storage::Cuda(gs)) = (&*qs, &*xs, &*gs) else {
        candle_core::bail!("native adjoint is CUDA-only; no host fallback");
    };
    if qs.device.id() != device.id()
        || xs.device.id() != device.id()
        || gs.device.id() != device.id()
    {
        candle_core::bail!("native adjoint tensors need the same CUDA context");
    }
    let Some(query) = input(qs, ql)? else {
        candle_core::bail!("native query adjoint needs F32 masters");
    };
    let metadata = spiralcore_fbs_device_plan(plan, device)?;
    let dq = zeros::<f32>(device, q.elem_count())?;
    let dx = zeros::<f32>(device, x.elem_count())?;
    let error = zeros::<u32>(device, 1)?;
    let mut args = vec![
        Arg::F(query),
        spiralcore_fbs_activation(xs, xl)?,
        spiralcore_fbs_activation(gs, gl)?,
    ];
    args.extend(spiralcore_fbs_metadata(plan, &metadata));
    args.extend([Arg::f(&dq), Arg::f(&dx), Arg::u(&error)]);
    args.extend(spiralcore_fbs_dimensions(plan, rows)?);
    if x.dtype() == DType::BF16 {
        crate::cuda_stack_kernels::cuda::launch_bf16(
            device,
            "spiralcore_fbs_bwd",
            rows * plan.output_blocks,
            &args,
        )?;
    } else {
        launch(
            device,
            "spiralcore_fbs_bwd",
            rows * plan.output_blocks,
            &args,
        )?;
    }
    drop(args);
    spiralcore_fbs_device_error(device, &error)?;
    let raw_dq = tensor(dq, device, q.shape());
    let raw_dx = tensor(dx, device, x.shape()).to_dtype(x.dtype())?;
    crate::secant_descent::transform_spiralcore_fbs_bwd(&q, raw_dq, raw_dx)
}
pub(super) fn rate_select_fwd(s: &CudaStorage, l: &Layout) -> Forward {
    let Some(score) = input(s, l)? else {
        candle_core::bail!("rate selection needs a contiguous F32 score");
    };
    let device = &s.device;
    let total = l.shape().elem_count();
    let out = uninit::<f32>(device, total * 2)?;
    launch(
        device,
        "rate_select_fwd",
        total,
        &[
            Arg::F(score),
            Arg::f(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    if crate::secant_descent::is_capturing_rate_fwd() {
        let host_pairs = device.clone_dtoh(&out)?;
        crate::secant_descent::on_rate_select_fwd_values(&host_pairs);
    }
    let mut dims = l.shape().dims().to_vec();
    dims.push(2);
    Ok((storage(out, device), Shape::from_dims(&dims)))
}
pub(super) fn rate_select_bwd(
    device: &CudaDevice,
    score: &Tensor,
    grad: &Tensor,
) -> CResult<Tensor> {
    let (s, g) = (ready(score)?, ready(grad)?);
    let total = s.elem_count();
    if g.elem_count() != total * 2 {
        candle_core::bail!("rate selection's gradient must be [.., lanes, 2]");
    }
    let (ss, sl) = s.storage_and_layout();
    let (gs, gl) = g.storage_and_layout();
    let d_score = uninit::<f32>(device, total)?;
    launch(
        device,
        "rate_select_bwd",
        total,
        &[
            Arg::F(view(&ss, sl)?),
            Arg::F(view(&gs, gl)?),
            Arg::f(&d_score),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    let raw_d_score = tensor(d_score, device, score.shape());
    crate::secant_descent::transform_rate_select_bwd(&s, &g, raw_d_score)
}
pub(super) fn rms_norm_fwd(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    let width = l2.shape().elem_count();
    if width == 0 || l1.shape().dims().last() != Some(&width) {
        candle_core::bail!("RMSNorm weight must match the last dimension");
    }
    let (Some(x), Some(w)) = (input(s1, l1)?, input(s2, l2)?) else {
        candle_core::bail!("CUDA RMSNorm requires contiguous F32 input");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let rows = total / width;
    let out = uninit::<f32>(device, total)?;
    launch_groups(
        device,
        "rms_norm_fwd",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::F(x),
            Arg::F(w),
            Arg::f(&out),
            Arg::U32(u32_of(width, "width")?),
        ],
    )?;
    Ok((storage(out, device), l1.shape().clone()))
}
pub(super) fn rms_norm_bwd(
    device: &CudaDevice,
    x: &Tensor,
    w: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    let width = w.elem_count();
    if width == 0 || x.dims().last() != Some(&width) || grad.shape() != x.shape() {
        candle_core::bail!("CUDA RMSNorm backward shapes disagree");
    }
    let (xr, wr, gr) = (ready(x)?, ready(w)?, ready(grad)?);
    let total = xr.elem_count();
    let rows = total / width;
    let (xs, xl) = xr.storage_and_layout();
    let (ws, wl) = wr.storage_and_layout();
    let (gs, gl) = gr.storage_and_layout();
    let (xv, wv, gv) = (view(&xs, xl)?, view(&ws, wl)?, view(&gs, gl)?);
    let dx = uninit::<f32>(device, total)?;
    let row_r = uninit::<f64>(device, rows)?;
    launch_groups(
        device,
        "rms_norm_bwd_dx",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::F(xv.slice(..)),
            Arg::F(wv),
            Arg::F(gv.slice(..)),
            Arg::f(&dx),
            Arg::d(&row_r),
            Arg::U32(u32_of(width, "width")?),
        ],
    )?;
    let chunks = rows.div_ceil(RMS_CHUNK_ROWS);
    let partials = uninit::<f64>(device, chunks * width)?;
    launch(
        device,
        "rms_norm_dw_partial",
        chunks * width,
        &[
            Arg::F(xv),
            Arg::F(gv),
            Arg::d(&row_r),
            Arg::d(&partials),
            Arg::U32(u32_of(rows, "rows")?),
            Arg::U32(u32_of(width, "width")?),
            Arg::U32(RMS_CHUNK_ROWS as u32),
        ],
    )?;
    let dw = uninit::<f32>(device, width)?;
    launch(
        device,
        "rms_norm_dw_reduce",
        width,
        &[
            Arg::d(&partials),
            Arg::f(&dw),
            Arg::U32(u32_of(chunks, "chunks")?),
            Arg::U32(u32_of(width, "width")?),
        ],
    )?;
    Ok((tensor(dx, device, x.shape()), tensor(dw, device, w.shape())))
}
pub(super) fn quaternion_scan_fwd(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    let (batch, time, lanes, four) = l1.shape().dims4()?;
    if four != 4 || l2.shape() != l1.shape() {
        candle_core::bail!("quaternion scan needs matching [batch, time, lanes, 4] inputs");
    }
    if batch == 0 || time == 0 || lanes == 0 {
        candle_core::bail!("quaternion scan requires positive dimensions");
    }
    let (Some(transition), Some(drive)) = (input(s1, l1)?, input(s2, l2)?) else {
        candle_core::bail!("CUDA scan requires contiguous F32 input");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let out = uninit::<f32>(device, total)?;
    launch(
        device,
        "quaternion_scan_fwd",
        batch * lanes,
        &[
            Arg::F(transition),
            Arg::F(drive),
            Arg::f(&out),
            Arg::U32(u32_of(time, "time")?),
            Arg::U32(u32_of(lanes, "lanes")?),
            Arg::U32(u32_of(batch * lanes, "sequences")?),
        ],
    )?;
    Ok((storage(out, device), l1.shape().clone()))
}
pub(super) fn quaternion_scan_bwd(
    device: &CudaDevice,
    transition: &Tensor,
    state: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    let (batch, time, lanes, four) = transition.dims4()?;
    if four != 4 || state.shape() != transition.shape() || grad.shape() != transition.shape() {
        candle_core::bail!(
            "CUDA quaternion scan backward inputs must match [batch, time, lanes, 4]"
        );
    }
    let (t, s, g) = (ready(transition)?, ready(state)?, ready(grad)?);
    let total = t.elem_count();
    let (ts, tl) = t.storage_and_layout();
    let (ss, sl) = s.storage_and_layout();
    let (gs, gl) = g.storage_and_layout();
    let dq = uninit::<f32>(device, total)?;
    let db = uninit::<f32>(device, total)?;
    launch(
        device,
        "quaternion_scan_bwd",
        batch * lanes,
        &[
            Arg::F(view(&ts, tl)?),
            Arg::F(view(&ss, sl)?),
            Arg::F(view(&gs, gl)?),
            Arg::f(&dq),
            Arg::f(&db),
            Arg::U32(u32_of(time, "time")?),
            Arg::U32(u32_of(lanes, "lanes")?),
            Arg::U32(u32_of(batch * lanes, "sequences")?),
        ],
    )?;
    Ok((
        tensor(dq, device, transition.shape()),
        tensor(db, device, transition.shape()),
    ))
}
pub(super) fn adam_step_cuda(
    device: &CudaDevice,
    var: &Var,
    grad: &Tensor,
    m: &Var,
    v: &Var,
    c: &AdamConstants,
) -> CResult<()> {
    let gradient = ready(grad)?;
    let n = var.elem_count();
    if gradient.elem_count() != n || m.elem_count() != n || v.elem_count() != n {
        candle_core::bail!("CUDA AdamW sizes disagree");
    }
    let (ps, pl) = var.as_tensor().storage_and_layout();
    let (gs, gl) = gradient.storage_and_layout();
    let (ms, ml) = m.as_tensor().storage_and_layout();
    let (vs, vl) = v.as_tensor().storage_and_layout();
    let constants = [
        c.scale, c.beta1, c.rest1, c.beta2, c.rest2, c.correct1, c.correct2, c.epsilon, c.keep,
        c.lr,
    ]
    .map(f32::to_bits);
    launch(
        device,
        "adam_update",
        n,
        &[
            Arg::F(view(&ps, pl)?),
            Arg::F(view(&ms, ml)?),
            Arg::F(view(&vs, vl)?),
            Arg::F(view(&gs, gl)?),
            Arg::Adam(constants),
            Arg::U32(u32_of(n, "size")?),
        ],
    )
}
struct SpiralcoreFbsDevicePlan {
    nodes: CudaSlice<u32>,
    frontier: CudaSlice<u32>,
    axes: CudaSlice<u32>,
    compose: CudaSlice<u32>,
    root_of_state: CudaSlice<u32>,
    bfs_actions: CudaSlice<u32>,
    branch_targets: CudaSlice<u32>,
    roots: CudaSlice<f32>,
}

// The SpiralCore/FBS sheet arms (`src/spiralcore_fbs_arms.cu`, spiralcore-fbs-offline-float-sheet,
// claude-119), launched from their own ahead-of-time module (`build.rs`, no runtime compiler).
// `arm` is 1 (identity transport) or 2 (two routed query paths); F32 carriers only, as qualified
// execution is F32. The pinned `spiralcore_fbs_fwd`/`spiralcore_fbs_bwd` above are untouched.
const SPIRALCORE_FBS_ARMS_MODULE: &str = "uor_r4_spiralcore_fbs_arms";
pub(crate) const SPIRALCORE_FBS_ARMS_PTX: &str =
    include_str!(concat!(env!("OUT_DIR"), "/spiralcore_fbs_arms_f32.ptx"));
fn spiralcore_fbs_arm_paths(arm: u32) -> CResult<usize> {
    match arm {
        1 => Ok(1),
        2 => Ok(2),
        _ => candle_core::bail!("unknown SpiralCore/FBS sheet arm code {arm}"),
    }
}
fn spiralcore_fbs_arm_rows(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    q: &[usize],
    x: &[usize],
    elements: usize,
    paths: usize,
) -> CResult<usize> {
    let [outputs, stages, eight] = plan.query_shape();
    if q != [outputs, stages * paths, eight]
        || x.last() != Some(&(plan.input_blocks * 8))
        || elements == 0
    {
        candle_core::bail!("arm projection buffers disagree with their static plan");
    }
    let rows = elements / (plan.input_blocks * 8);
    let threads = rows
        .checked_mul(plan.output_blocks)
        .ok_or_else(|| candle_core::Error::Msg("arm projection launch size overflow".into()))?;
    u32_of(threads, "arm projection carriers")?;
    Ok(rows)
}
pub(crate) fn spiralcore_fbs_arm_fwd(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    q: &CudaStorage,
    ql: &Layout,
    x: &CudaStorage,
    xl: &Layout,
    arm: u32,
) -> Forward {
    if q.device.id() != x.device.id() {
        candle_core::bail!("arm routing points and carriers need the same CUDA context");
    }
    if x.dtype() != DType::F32 {
        candle_core::bail!("sheet arms run on F32 carriers only");
    }
    let paths = spiralcore_fbs_arm_paths(arm)?;
    let rows = spiralcore_fbs_arm_rows(
        plan,
        ql.shape().dims(),
        xl.shape().dims(),
        xl.shape().elem_count(),
        paths,
    )?;
    let Some(query) = input(q, ql)? else {
        candle_core::bail!("arm queries need contiguous F32 masters");
    };
    let device = &q.device;
    let metadata = spiralcore_fbs_device_plan(plan, device)?;
    let error = zeros::<u32>(device, 1)?;
    let threads = rows * plan.output_blocks;
    let mut dims = xl.shape().dims().to_vec();
    let last = dims.len() - 1;
    dims[last] = 8 * plan.output_blocks;
    let out = uninit::<f32>(device, threads * 8)?;
    let mut args = vec![Arg::F(query), spiralcore_fbs_activation(x, xl)?];
    args.extend(spiralcore_fbs_metadata(plan, &metadata));
    args.push(Arg::f(&out));
    args.push(Arg::u(&error));
    args.extend(spiralcore_fbs_dimensions(plan, rows)?);
    args.push(Arg::U32(arm));
    crate::cuda_stack_kernels::cuda::launch_prebuilt(
        device,
        SPIRALCORE_FBS_ARMS_MODULE,
        SPIRALCORE_FBS_ARMS_PTX,
        "spiralcore_fbs_arm_fwd",
        threads,
        &args,
    )?;
    drop(args);
    spiralcore_fbs_device_error(device, &error)?;
    Ok((storage(out, device), Shape::from_dims(&dims)))
}
pub(crate) fn spiralcore_fbs_arm_bwd(
    plan: &crate::spiralcore_fbs::ProjectionPlan,
    device: &CudaDevice,
    q: &Tensor,
    x: &Tensor,
    grad: &Tensor,
    arm: u32,
) -> CResult<(Tensor, Tensor)> {
    let (q, x, grad) = (q.contiguous()?, x.contiguous()?, grad.contiguous()?);
    if x.dtype() != DType::F32 || grad.dtype() != DType::F32 {
        candle_core::bail!("sheet arms run on F32 carriers only");
    }
    let paths = spiralcore_fbs_arm_paths(arm)?;
    let rows = spiralcore_fbs_arm_rows(plan, q.dims(), x.dims(), x.elem_count(), paths)?;
    let mut output_dims = x.dims().to_vec();
    let last = output_dims.len() - 1;
    output_dims[last] = plan.output_blocks * 8;
    if grad.dims() != output_dims {
        candle_core::bail!("arm projection gradient shape differs from its output");
    }
    let (qs, ql) = q.storage_and_layout();
    let (xs, xl) = x.storage_and_layout();
    let (gs, gl) = grad.storage_and_layout();
    let (Storage::Cuda(qs), Storage::Cuda(xs), Storage::Cuda(gs)) = (&*qs, &*xs, &*gs) else {
        candle_core::bail!("arm adjoint is CUDA-only; no host fallback");
    };
    if qs.device.id() != device.id()
        || xs.device.id() != device.id()
        || gs.device.id() != device.id()
    {
        candle_core::bail!("arm adjoint tensors need the same CUDA context");
    }
    let Some(query) = input(qs, ql)? else {
        candle_core::bail!("arm query adjoint needs F32 masters");
    };
    let metadata = spiralcore_fbs_device_plan(plan, device)?;
    let dq = zeros::<f32>(device, q.elem_count())?;
    let dx = zeros::<f32>(device, x.elem_count())?;
    let error = zeros::<u32>(device, 1)?;
    let mut args = vec![
        Arg::F(query),
        spiralcore_fbs_activation(xs, xl)?,
        spiralcore_fbs_activation(gs, gl)?,
    ];
    args.extend(spiralcore_fbs_metadata(plan, &metadata));
    args.extend([Arg::f(&dq), Arg::f(&dx), Arg::u(&error)]);
    args.extend(spiralcore_fbs_dimensions(plan, rows)?);
    args.push(Arg::U32(arm));
    crate::cuda_stack_kernels::cuda::launch_prebuilt(
        device,
        SPIRALCORE_FBS_ARMS_MODULE,
        SPIRALCORE_FBS_ARMS_PTX,
        "spiralcore_fbs_arm_bwd",
        rows * plan.output_blocks,
        &args,
    )?;
    drop(args);
    spiralcore_fbs_device_error(device, &error)?;
    Ok((tensor(dq, device, q.shape()), tensor(dx, device, x.shape())))
}
