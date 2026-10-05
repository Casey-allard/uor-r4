//! CUDA forward and backward paths of the geometric stack ops.
//!
//! Each op's `cuda_fwd` and its backward's `Device::Cuda` branch call into
//! this module. Configurations without a kernel (a transport snap, RoPE or a
//! flock selection in the read, a selected or prime-routed pointer mixture,
//! non-contiguous inputs)
//! run the exact CPU forward on host copies ([`via_host1`] and friends), and
//! their backward takes the generic host path, so CUDA training always
//! computes what the CPU computes. The kernels live in
//! [`crate::cuda_stack_kernels`].

use candle_core::backend::{BackendDevice, BackendStorage};
use candle_core::op::BackpropOp;
use candle_core::{CpuStorage, CudaDevice, CudaStorage, CustomOp1, Storage};
use cudarc::driver::{CudaSlice, CudaView};

use super::*;
use crate::cuda_stack_kernels::cuda::{launch, launch_groups, uninit, zeros, Arg};

type CResult<T> = candle_core::Result<T>;
type Forward = CResult<(CudaStorage, Shape)>;

// ---------------------------------------------------------------------------
// Host fallbacks and buffer helpers.

/// Uploads a host forward's F32 result to `device`.
fn upload(device: &CudaDevice, out: CpuStorage, shape: Shape, name: &str) -> Forward {
    if !matches!(out, CpuStorage::F32(_)) {
        candle_core::bail!("{name} host fallback expects F32 output");
    }
    Ok((device.storage_from_cpu_storage(&out)?, shape))
}

/// Runs a one-input op's exact CPU forward on a host copy of its CUDA input.
pub(super) fn via_host1(op: &impl CustomOp1, s1: &CudaStorage, l1: &Layout) -> Forward {
    let (out, shape) = op.cpu_fwd(&s1.to_cpu_storage()?, l1)?;
    upload(&s1.device, out, shape, op.name())
}

/// Runs a two-input op's exact CPU forward on host copies of its CUDA inputs.
pub(super) fn via_host2(
    op: &impl CustomOp2,
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    let (out, shape) = op.cpu_fwd(&s1.to_cpu_storage()?, l1, &s2.to_cpu_storage()?, l2)?;
    upload(&s1.device, out, shape, op.name())
}

/// Runs a three-input op's exact CPU forward on host copies of its CUDA
/// inputs; used where no CUDA kernel covers the op's configuration.
pub(super) fn via_host3(op: &impl CustomOp3, inputs: [(&CudaStorage, &Layout); 3]) -> Forward {
    let [(s1, l1), (s2, l2), (s3, l3)] = inputs;
    let (out, shape) = op.cpu_fwd(
        &s1.to_cpu_storage()?,
        l1,
        &s2.to_cpu_storage()?,
        l2,
        &s3.to_cpu_storage()?,
        l3,
    )?;
    upload(&s1.device, out, shape, op.name())
}

/// The F32 elements of a contiguous CUDA input (any start offset), or `None`
/// when the layout is not contiguous or the dtype is not F32.
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

/// The F32 elements of a CUDA tensor's storage prepared by [`ready`].
fn view<'a>(storage: &'a Storage, layout: &Layout) -> CResult<CudaView<'a, f32>> {
    let Storage::Cuda(storage) = storage else {
        candle_core::bail!("expected a CUDA tensor");
    };
    match input(storage, layout)? {
        Some(view) => Ok(view),
        None => candle_core::bail!("CUDA stack kernels need contiguous F32 tensors"),
    }
}

/// A CUDA F32 tensor laid out contiguously (copied only when it is not).
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

/// A gradient tensor over a fresh CUDA buffer.
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

// ---------------------------------------------------------------------------
// StraightThrough, SwiGLU, RMSNorm, the quaternion scan.

pub(super) fn straight_through_fwd(
    s1: &CudaStorage,
    l1: &Layout,
    s2: &CudaStorage,
    l2: &Layout,
) -> Forward {
    if l1.shape() != l2.shape() {
        candle_core::bail!("straight-through inputs must have one shape");
    }
    let total = l2.shape().elem_count();
    if total == 0 {
        candle_core::bail!("straight-through requires non-empty tensors");
    }
    if s1.dtype() != DType::F32 {
        candle_core::bail!("straight-through requires F32 tensors");
    }
    let Some(served) = input(s2, l2)? else {
        return via_host2(&StraightThrough, s1, l1, s2, l2);
    };
    let device = &s2.device;
    let out = uninit::<f32>(device, total)?;
    launch(
        device,
        "straight_through_fwd",
        total,
        &[
            Arg::F(served),
            Arg::f(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((storage(out, device), l2.shape().clone()))
}

pub(super) fn swiglu_fwd(s1: &CudaStorage, l1: &Layout, s2: &CudaStorage, l2: &Layout) -> Forward {
    if l1.shape() != l2.shape() {
        candle_core::bail!("SwiGLU inputs must match");
    }
    let (Some(gate), Some(up)) = (input(s1, l1)?, input(s2, l2)?) else {
        return via_host2(&SwiGlu, s1, l1, s2, l2);
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let out = uninit::<f32>(device, total)?;
    launch(
        device,
        "swiglu_fwd",
        total,
        &[
            Arg::F(gate),
            Arg::F(up),
            Arg::f(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((storage(out, device), l1.shape().clone()))
}

pub(super) fn swiglu_bwd(
    device: &CudaDevice,
    gate: &Tensor,
    up: &Tensor,
    grad: &Tensor,
) -> CResult<(Tensor, Tensor)> {
    if gate.shape() != up.shape() || grad.shape() != gate.shape() {
        candle_core::bail!("CUDA SwiGLU backward inputs must match");
    }
    let (g, u, d) = (ready(gate)?, ready(up)?, ready(grad)?);
    let total = g.elem_count();
    let (gs, gl) = g.storage_and_layout();
    let (us, ul) = u.storage_and_layout();
    let (ds, dl) = d.storage_and_layout();
    let d_gate = uninit::<f32>(device, total)?;
    let d_up = uninit::<f32>(device, total)?;
    launch(
        device,
        "swiglu_bwd",
        total,
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

/// Threads per block of the row reductions (a multiple of 32).
const ROW_THREADS: usize = 256;
/// Rows per partial sum of the RMSNorm weight gradient.
const RMS_CHUNK_ROWS: usize = 256;

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
        return via_host2(&RmsNorm, s1, l1, s2, l2);
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
        return via_host2(&QuaternionScan, s1, l1, s2, l2);
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

// ---------------------------------------------------------------------------
// Cross-entropy.

/// Row log-sum-exps and losses (f64) of `[rows, vocabulary]` logits.
fn cross_entropy_rows(
    device: &CudaDevice,
    logits: CudaView<'_, f32>,
    targets: &CudaSlice<u32>,
    rows: usize,
    vocabulary: usize,
) -> CResult<(CudaSlice<f64>, CudaSlice<f64>)> {
    let lse = zeros::<f64>(device, rows)?;
    let loss = zeros::<f64>(device, rows)?;
    launch_groups(
        device,
        "cross_entropy_rows",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::F(logits),
            Arg::u(targets),
            Arg::d(&lse),
            Arg::d(&loss),
            Arg::U32(u32_of(vocabulary, "vocabulary")?),
        ],
    )?;
    Ok((lse, loss))
}

fn check_targets(op: &CrossEntropy, rows: usize, vocabulary: usize) -> CResult<()> {
    if rows == 0 || vocabulary == 0 {
        candle_core::bail!("CrossEntropy requires positive rows and vocabulary");
    }
    if op.targets.len() != rows || op.weights.as_ref().is_some_and(|w| w.len() != rows) {
        candle_core::bail!("CrossEntropy needs one target and one weight per row");
    }
    if op.targets.iter().any(|&t| t as usize >= vocabulary) {
        candle_core::bail!("CrossEntropy target outside the vocabulary");
    }
    Ok(())
}

pub(super) fn cross_entropy_fwd(op: &CrossEntropy, s: &CudaStorage, l: &Layout) -> Forward {
    let (rows, vocabulary) = l.shape().dims2()?;
    check_targets(op, rows, vocabulary)?;
    let Some(logits) = input(s, l)? else {
        return via_host1(op, s, l);
    };
    let device = &s.device;
    let targets = device.clone_htod(&op.targets)?;
    let (_, loss) = cross_entropy_rows(device, logits, &targets, rows, vocabulary)?;
    let row_losses = device.clone_dtoh(&loss)?;
    let mean = match &op.weights {
        None => row_losses.iter().sum::<f64>() / rows as f64,
        Some(weights) => {
            let total: f64 = row_losses
                .iter()
                .zip(weights)
                .filter(|(_, &w)| w != 0.0)
                .map(|(&l, &w)| f64::from(w) * l)
                .sum();
            total / weights.iter().map(|&w| f64::from(w)).sum::<f64>()
        }
    };
    let out = device.clone_htod(&vec![mean as f32])?;
    Ok((storage(out, device), Shape::from(())))
}

pub(super) fn cross_entropy_bwd(
    op: &CrossEntropy,
    device: &CudaDevice,
    logits: &Tensor,
    grad: &Tensor,
) -> CResult<Tensor> {
    let (rows, vocabulary) = logits.dims2()?;
    check_targets(op, rows, vocabulary)?;
    let grad = f64::from(grad.to_scalar::<f32>()?);
    let scales: Vec<f64> = match &op.weights {
        None => vec![grad / rows as f64; rows],
        Some(weights) => {
            let total: f64 = weights.iter().map(|&w| f64::from(w)).sum();
            weights
                .iter()
                .map(|&w| {
                    if w == 0.0 {
                        0.0
                    } else {
                        grad * f64::from(w) / total
                    }
                })
                .collect()
        }
    };
    let l = ready(logits)?;
    let (ls, ll) = l.storage_and_layout();
    let lv = view(&ls, ll)?;
    let targets = device.clone_htod(&op.targets)?;
    let scale = device.clone_htod(&scales)?;
    let (lse, _) = cross_entropy_rows(device, lv.slice(..), &targets, rows, vocabulary)?;
    let out = uninit::<f32>(device, rows * vocabulary)?;
    launch(
        device,
        "cross_entropy_grad",
        rows * vocabulary,
        &[
            Arg::F(lv),
            Arg::u(&targets),
            Arg::d(&lse),
            Arg::d(&scale),
            Arg::f(&out),
            Arg::U32(u32_of(rows, "rows")?),
            Arg::U32(u32_of(vocabulary, "vocabulary")?),
        ],
    )?;
    Ok(tensor(out, device, logits.shape()))
}

// ---------------------------------------------------------------------------
// The recurrence core.

impl RecurrenceCore {
    /// Whether the CUDA kernels cover this configuration (both rotation
    /// groups; a transport snap runs on the host).
    pub(super) fn cuda_covered(&self) -> bool {
        self.snap.is_none()
    }

    /// Kernel rotation mode: 0 none, 1 quaternion (S3), 2 U(1).
    fn cuda_rotation(&self) -> u32 {
        match (self.rotation, self.group) {
            (false, _) => 0,
            (true, RotationGroup::Quaternion) => 1,
            (true, RotationGroup::U1) => 2,
        }
    }

    fn cuda_check(&self) -> CResult<()> {
        if self.batch == 0 || self.time == 0 || self.width == 0 || self.width % 4 != 0 {
            candle_core::bail!(
                "CUDA RecurrenceCore requires positive dimensions with width divisible by 4"
            );
        }
        Ok(())
    }

    /// `log a` per lane on the device.
    fn cuda_log_a(
        &self,
        device: &CudaDevice,
        parameters: CudaView<'_, f32>,
    ) -> CResult<CudaSlice<f32>> {
        let lanes = self.lanes();
        let log_a = uninit::<f32>(device, lanes)?;
        launch(
            device,
            "recurrence_log_a",
            lanes,
            &[
                Arg::F(parameters),
                Arg::f(&log_a),
                Arg::U32(u32_of(self.width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
            ],
        )?;
        Ok(log_a)
    }

    /// The forward recurrence: states, drives and outputs.
    #[allow(clippy::type_complexity)]
    fn cuda_forward(
        &self,
        device: &CudaDevice,
        branches: CudaView<'_, f32>,
        gates: CudaView<'_, f32>,
        parameters: CudaView<'_, f32>,
        log_a: &CudaSlice<f32>,
    ) -> CResult<(CudaSlice<f32>, CudaSlice<f32>, CudaSlice<f32>)> {
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let total = self.batch * time * width;
        let state = uninit::<f32>(device, total)?;
        let drive = uninit::<f32>(device, total)?;
        let out = uninit::<f32>(device, total)?;
        launch(
            device,
            "recurrence_core_fwd",
            self.batch * lanes,
            &[
                Arg::F(branches),
                Arg::F(gates),
                Arg::F(parameters),
                Arg::f(log_a),
                Arg::f(&state),
                Arg::f(&drive),
                Arg::f(&out),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(self.gate_width(), "gate width")?),
                Arg::U32(self.cuda_rotation()),
                Arg::U32(u32_of(self.batch * lanes, "lanes")?),
            ],
        )?;
        Ok((state, drive, out))
    }

    pub(super) fn cuda_fwd_impl(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
        if !self.cuda_covered() {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        }
        self.cuda_check()?;
        let (time, width) = (self.time, self.width);
        if l1.shape().elem_count() != self.batch * time * 2 * width
            || l2.shape().elem_count() != self.batch * time * self.gate_width()
            || l3.shape().elem_count() != self.parameter_len()
        {
            candle_core::bail!("recurrence core inputs have the wrong sizes");
        }
        let (Some(branches), Some(gates), Some(parameters)) =
            (input(s1, l1)?, input(s2, l2)?, input(s3, l3)?)
        else {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        };
        let device = &s1.device;
        let log_a = self.cuda_log_a(device, parameters.slice(..))?;
        let (_, _, out) = self.cuda_forward(device, branches, gates, parameters, &log_a)?;
        Ok((storage(out, device), Shape::from((self.batch, time, width))))
    }

    /// The exact backward on CUDA: recomputes the forward states, sweeps
    /// each (window, lane) in reverse, then reduces the windows' f64
    /// parameter partials.
    pub(super) fn cuda_bwd(
        &self,
        device: &CudaDevice,
        branches: &Tensor,
        gates: &Tensor,
        parameters: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        self.cuda_check()?;
        let (b, g, p, dy) = (
            ready(branches)?,
            ready(gates)?,
            ready(parameters)?,
            ready(grad)?,
        );
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let gate_width = self.gate_width();
        let param_len = self.parameter_len();
        if b.elem_count() != self.batch * time * 2 * width
            || g.elem_count() != self.batch * time * gate_width
            || p.elem_count() != param_len
            || dy.elem_count() != self.batch * time * width
        {
            candle_core::bail!("CUDA recurrence backward inputs have the wrong sizes");
        }
        let (bs, bl) = b.storage_and_layout();
        let (gs, gl) = g.storage_and_layout();
        let (ps, pl) = p.storage_and_layout();
        let (ds, dl) = dy.storage_and_layout();
        let (bv, gv, pv, dv) = (
            view(&bs, bl)?,
            view(&gs, gl)?,
            view(&ps, pl)?,
            view(&ds, dl)?,
        );
        let log_a = self.cuda_log_a(device, pv.slice(..))?;
        let (state, drive, _) =
            self.cuda_forward(device, bv.slice(..), gv.slice(..), pv.slice(..), &log_a)?;
        let d_branches = uninit::<f32>(device, b.elem_count())?;
        let d_gates = uninit::<f32>(device, g.elem_count())?;
        let partials = uninit::<f64>(device, self.batch * param_len)?;
        let d_parameters = uninit::<f32>(device, param_len)?;
        launch(
            device,
            "recurrence_core_bwd",
            self.batch * lanes,
            &[
                Arg::F(bv),
                Arg::F(gv),
                Arg::F(pv.slice(..)),
                Arg::f(&log_a),
                Arg::f(&state),
                Arg::f(&drive),
                Arg::F(dv),
                Arg::f(&d_branches),
                Arg::f(&d_gates),
                Arg::d(&partials),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(gate_width, "gate width")?),
                Arg::U32(self.cuda_rotation()),
                Arg::U32(u32_of(self.batch * lanes, "lanes")?),
                Arg::U32(u32_of(param_len, "parameters")?),
            ],
        )?;
        launch(
            device,
            "recurrence_param_reduce",
            param_len,
            &[
                Arg::d(&partials),
                Arg::F(pv),
                Arg::f(&d_parameters),
                Arg::U32(u32_of(self.batch, "batch")?),
                Arg::U32(u32_of(param_len, "parameters")?),
                Arg::U32(u32_of(width, "width")?),
            ],
        )?;
        Ok((
            tensor(d_branches, device, branches.shape()),
            tensor(d_gates, device, gates.shape()),
            tensor(d_parameters, device, parameters.shape()),
        ))
    }
}

// ---------------------------------------------------------------------------
// The fused read.

/// The device buffers of one recomputed read: probabilities [index, t, j]
/// (j <= t used), NoRead probabilities, the f64 query and key lifts and,
/// when asked for, the f64 Lorentz excesses or L2 squared distances.
struct CudaReadPass {
    probabilities: CudaSlice<f32>,
    null_probability: CudaSlice<f32>,
    query_lift: CudaSlice<f64>,
    key_lift: CudaSlice<f64>,
    excess: CudaSlice<f64>,
}

impl FusedRead {
    /// Whether the CUDA kernels cover this configuration (all three scores;
    /// RoPE and flock selection run on the host).
    pub(super) fn cuda_covered(&self) -> bool {
        !self.rope && self.select.is_none()
    }

    fn cuda_dims(&self) -> CResult<[u32; 8]> {
        Ok([
            u32_of(self.batch, "batch")?,
            u32_of(self.heads, "heads")?,
            u32_of(self.time, "time")?,
            u32_of(self.key, "key")?,
            u32_of(self.value, "value")?,
            u32::from(self.null),
            u32::from(self.age),
            match self.score {
                ReadScore::Dot => 0,
                ReadScore::Lorentz => 1,
                ReadScore::L2 => 2,
            },
        ])
    }

    /// Scores and softmax of every row on the device.
    fn cuda_pass(
        &self,
        device: &CudaDevice,
        query: CudaView<'_, f32>,
        kv: CudaView<'_, f32>,
        aux: CudaView<'_, f32>,
        keep_excess: bool,
    ) -> CResult<CudaReadPass> {
        let dims = self.cuda_dims()?;
        let rows = self.batch * self.heads * self.time;
        let square = rows * self.time;
        let scaled = self.score.scaled();
        let probabilities = uninit::<f32>(device, square)?;
        let null_probability = uninit::<f32>(device, rows)?;
        let lift_len = if scaled { rows } else { 1 };
        let query_lift = uninit::<f64>(device, lift_len)?;
        let key_lift = uninit::<f64>(device, lift_len)?;
        let write_excess = scaled && keep_excess;
        let excess = uninit::<f64>(device, if write_excess { square } else { 1 })?;
        if scaled {
            launch(
                device,
                "read_lift",
                rows,
                &[
                    Arg::F(query.slice(..)),
                    Arg::F(kv.slice(..)),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::Dims(dims),
                ],
            )?;
        }
        let tiles = self.time.div_ceil(16);
        let geometry = [
            u32_of(self.key, "key")?,
            0,
            u32_of(self.width(), "width")?,
            0,
            u32_of(self.key, "key")?,
            1,
            u32::from(write_excess),
        ];
        launch_groups(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::F(query),
                Arg::F(kv),
                Arg::F(aux.slice(..)),
                Arg::d(&query_lift),
                Arg::d(&key_lift),
                Arg::f(&probabilities),
                Arg::d(&excess),
                Arg::Dims(dims),
                Arg::Geom(geometry),
            ],
        )?;
        launch(
            device,
            "read_softmax_warp",
            32 * rows,
            &[
                Arg::f(&probabilities),
                Arg::F(aux),
                Arg::f(&null_probability),
                Arg::Dims(dims),
            ],
        )?;
        Ok(CudaReadPass {
            probabilities,
            null_probability,
            query_lift,
            key_lift,
            excess,
        })
    }

    fn cuda_check(&self) -> CResult<()> {
        if self.batch == 0 || self.heads == 0 || self.time == 0 || self.key == 0 || self.value == 0
        {
            candle_core::bail!("CUDA FusedRead requires positive dimensions");
        }
        Ok(())
    }

    pub(super) fn cuda_fwd_impl(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
        if !self.cuda_covered() {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        }
        self.cuda_check()?;
        let (time, value) = (self.time, self.value);
        let rows = self.batch * self.heads * time;
        let expected_aux = fused_aux_len(
            self.batch, self.heads, time, self.score, self.null, self.age,
        )
        .max(1);
        if l1.shape().elem_count() != rows * self.key
            || l2.shape().elem_count() != rows * self.width()
            || l3.shape().elem_count() != expected_aux
        {
            candle_core::bail!("FusedRead input element counts do not match declared dimensions");
        }
        let (Some(query), Some(kv), Some(aux)) = (input(s1, l1)?, input(s2, l2)?, input(s3, l3)?)
        else {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        };
        let device = &s1.device;
        let pass = self.cuda_pass(device, query, kv.slice(..), aux, false)?;
        let total = rows * value;
        let out = uninit::<f32>(device, total)?;
        launch(
            device,
            "read_mix",
            total,
            &[
                Arg::f(&pass.probabilities),
                Arg::F(kv),
                Arg::f(&out),
                Arg::Dims(self.cuda_dims()?),
            ],
        )?;
        Ok((
            storage(out, device),
            Shape::from((self.batch, self.heads, time, value)),
        ))
    }

    /// The exact backward on CUDA.
    pub(super) fn cuda_bwd(
        &self,
        device: &CudaDevice,
        query: &Tensor,
        kv: &Tensor,
        aux: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        self.cuda_check()?;
        let (q, kvt, a, dy) = (ready(query)?, ready(kv)?, ready(aux)?, ready(grad)?);
        let rows = self.batch * self.heads * self.time;
        if q.elem_count() != rows * self.key
            || kvt.elem_count() != rows * self.width()
            || dy.elem_count() != rows * self.value
        {
            candle_core::bail!("CUDA read backward inputs have the wrong sizes");
        }
        let (qs, ql) = q.storage_and_layout();
        let (ks, kl) = kvt.storage_and_layout();
        let (as_, al) = a.storage_and_layout();
        let (ds, dl) = dy.storage_and_layout();
        let (qv, kvv, av, dyv) = (
            view(&qs, ql)?,
            view(&ks, kl)?,
            view(&as_, al)?,
            view(&ds, dl)?,
        );
        let scaled = self.score.scaled();
        let pass = self.cuda_pass(device, qv.slice(..), kvv.slice(..), av.slice(..), true)?;
        let dims = self.cuda_dims()?;
        let square = rows * self.time;
        let dp = uninit::<f32>(device, square)?;
        let d_scores = uninit::<f64>(device, square)?;
        let inner_grad = uninit::<f32>(device, square)?;
        let row_len = if scaled { rows } else { 1 };
        let query_self = uninit::<f64>(device, row_len)?;
        let row_beta = uninit::<f64>(device, row_len)?;
        let row_offset = uninit::<f64>(device, row_len)?;
        let key_self = uninit::<f64>(device, row_len)?;
        let dq = uninit::<f32>(device, q.elem_count())?;
        let dkv = uninit::<f32>(device, kvt.elem_count())?;
        let aux_parts = self.null || self.age || scaled;
        let d_aux = zeros::<f32>(device, a.elem_count())?;
        let tiles = self.time.div_ceil(16);
        let geometry = [
            u32_of(self.value, "value")?,
            0,
            u32_of(self.width(), "width")?,
            u32_of(self.key, "key")?,
            u32_of(self.value, "value")?,
            0,
            0,
        ];
        launch_groups(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::F(dyv.slice(..)),
                Arg::F(kvv.slice(..)),
                Arg::F(av.slice(..)),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::f(&dp),
                Arg::d(&pass.excess),
                Arg::Dims(dims),
                Arg::Geom(geometry),
            ],
        )?;
        launch(
            device,
            "read_row_grad_warp",
            32 * rows,
            &[
                Arg::f(&pass.probabilities),
                Arg::f(&dp),
                Arg::d(&d_scores),
                Arg::f(&inner_grad),
                Arg::d(&pass.excess),
                Arg::f(&pass.null_probability),
                Arg::F(av.slice(..)),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::d(&query_self),
                Arg::d(&row_beta),
                Arg::d(&row_offset),
                Arg::f(&d_aux),
                Arg::Dims(dims),
            ],
        )?;
        if scaled {
            launch(
                device,
                "read_key_self",
                rows,
                &[
                    Arg::d(&d_scores),
                    Arg::d(&pass.excess),
                    Arg::F(av),
                    Arg::d(&pass.query_lift),
                    Arg::d(&pass.key_lift),
                    Arg::d(&key_self),
                    Arg::Dims(dims),
                ],
            )?;
        }
        launch(
            device,
            "read_dq",
            q.elem_count(),
            &[
                Arg::f(&inner_grad),
                Arg::F(qv.slice(..)),
                Arg::F(kvv.slice(..)),
                Arg::d(&query_self),
                Arg::f(&dq),
                Arg::Dims(dims),
            ],
        )?;
        launch(
            device,
            "read_dkv",
            kvt.elem_count(),
            &[
                Arg::f(&inner_grad),
                Arg::f(&pass.probabilities),
                Arg::F(qv),
                Arg::F(kvv),
                Arg::F(dyv),
                Arg::d(&key_self),
                Arg::f(&dkv),
                Arg::Dims(dims),
            ],
        )?;
        if self.age {
            launch(
                device,
                "read_dage",
                self.heads * self.time,
                &[Arg::d(&d_scores), Arg::f(&d_aux), Arg::Dims(dims)],
            )?;
        }
        if scaled {
            launch(
                device,
                "read_dbeta",
                2 * self.heads,
                &[
                    Arg::d(&row_beta),
                    Arg::d(&row_offset),
                    Arg::f(&d_aux),
                    Arg::Dims(dims),
                ],
            )?;
        }
        let d_aux = if aux_parts {
            tensor(d_aux, device, aux.shape())
        } else {
            // The placeholder auxiliary input carries no gradient.
            Tensor::zeros(aux.shape(), DType::F32, aux.device())?
        };
        Ok((
            tensor(dq, device, query.shape()),
            tensor(dkv, device, kv.shape()),
            d_aux,
        ))
    }
}

// ---------------------------------------------------------------------------
// The pointer-copy mixture loss.

/// The device buffers of one evaluated batch of pointer rows.
struct CudaPointerPass {
    /// [rows, time] attention (forward) or per-source gradient (backward).
    scratch: CudaSlice<f64>,
    /// Forward: `-weight log mixture` per row.
    row_value: CudaSlice<f64>,
    /// Backward: the logit gradient's per-row factor `c share_generate`.
    scale_z: CudaSlice<f64>,
    /// Backward: the side gradient (gate column written by the row pass).
    d_side: CudaSlice<f32>,
    /// Backward: per-row d loss / d beta.
    row_beta: CudaSlice<f64>,
    /// Backward (Lorentz): the query's self coefficient per row.
    query_self: CudaSlice<f64>,
    query_lift: CudaSlice<f64>,
    key_lift: CudaSlice<f64>,
    /// Row log-sum-exps of the logits (f64) and the uploaded targets.
    lse: CudaSlice<f64>,
    targets: CudaSlice<u32>,
}

impl PointerMixture {
    /// Whether the CUDA kernels cover this configuration: Dot or Lorentz
    /// scores over every source. A selection, a prime route (and the L2
    /// score `PointerConfig::validate` refuses) run on the host.
    pub(super) fn cuda_covered(&self) -> bool {
        self.select.is_none() && self.route.is_none() && self.score != ReadScore::L2
    }

    fn cuda_dims(&self, rows: usize, vocabulary: usize, backward: bool) -> CResult<[u32; 8]> {
        Ok([
            u32_of(rows, "rows")?,
            u32_of(self.time, "time")?,
            u32_of(self.dim, "pointer width")?,
            u32_of(vocabulary, "vocabulary")?,
            u32::from(self.score == ReadScore::Lorentz),
            u32::from(backward),
            0,
            0,
        ])
    }

    fn cuda_check(&self, vocabulary: usize) -> CResult<()> {
        if self.dim == 0 || vocabulary == 0 {
            candle_core::bail!("CUDA pointer mixture requires positive dimensions");
        }
        if self.targets.iter().any(|&t| t as usize >= vocabulary) {
            candle_core::bail!("pointer mixture target outside the vocabulary");
        }
        Ok(())
    }

    /// The weights as uploaded (ones when unweighted).
    fn cuda_weights(&self, rows: usize) -> Vec<f32> {
        self.weights.clone().unwrap_or_else(|| vec![1.0; rows])
    }

    /// Log-sum-exps, lifts and the row pass on the device. `grad` is the
    /// one-element upstream gradient (any one-element buffer in the forward).
    #[allow(clippy::too_many_arguments)]
    fn cuda_pass(
        &self,
        device: &CudaDevice,
        logits: CudaView<'_, f32>,
        side: CudaView<'_, f32>,
        beta: CudaView<'_, f32>,
        grad: CudaView<'_, f32>,
        vocabulary: usize,
        backward: bool,
    ) -> CResult<CudaPointerPass> {
        let rows = self.targets.len();
        let dims = self.cuda_dims(rows, vocabulary, backward)?;
        let targets = device.clone_htod(&self.targets)?;
        let ids = device.clone_htod(&self.ids)?;
        let weights = device.clone_htod(&self.cuda_weights(rows))?;
        let total = device.clone_htod(&[self.total()])?;
        let (lse, _) = cross_entropy_rows(device, logits.slice(..), &targets, rows, vocabulary)?;
        let query_lift = zeros::<f64>(device, rows)?;
        let key_lift = zeros::<f64>(device, rows)?;
        if self.score == ReadScore::Lorentz {
            launch(
                device,
                "pointer_lift",
                rows,
                &[
                    Arg::F(side.slice(..)),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::Dims(dims),
                ],
            )?;
        }
        let stride = 2 * self.dim + 1;
        let pass = CudaPointerPass {
            scratch: zeros::<f64>(device, rows * self.time)?,
            row_value: zeros::<f64>(device, rows)?,
            scale_z: zeros::<f64>(device, rows)?,
            d_side: zeros::<f32>(device, if backward { rows * stride } else { 1 })?,
            row_beta: zeros::<f64>(device, rows)?,
            query_self: zeros::<f64>(device, rows)?,
            query_lift,
            key_lift,
            lse,
            targets,
        };
        launch(
            device,
            "pointer_rows",
            32 * rows,
            &[
                Arg::F(logits),
                Arg::F(side),
                Arg::F(beta),
                Arg::u(&ids),
                Arg::u(&pass.targets),
                Arg::f(&weights),
                Arg::d(&pass.lse),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::F(grad),
                Arg::d(&total),
                Arg::d(&pass.scratch),
                Arg::d(&pass.row_value),
                Arg::d(&pass.scale_z),
                Arg::f(&pass.d_side),
                Arg::d(&pass.row_beta),
                Arg::d(&pass.query_self),
                Arg::Dims(dims),
            ],
        )?;
        Ok(pass)
    }

    pub(super) fn cuda_fwd_impl(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
        if !self.cuda_covered() {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        }
        let (rows, vocabulary) = self.check(l1, l2, l3)?;
        self.cuda_check(vocabulary)?;
        let (Some(logits), Some(side), Some(beta)) =
            (input(s1, l1)?, input(s2, l2)?, input(s3, l3)?)
        else {
            return via_host3(self, [(s1, l1), (s2, l2), (s3, l3)]);
        };
        let device = &s1.device;
        let pass = self.cuda_pass(
            device,
            logits,
            side,
            beta.slice(..),
            beta.slice(..),
            vocabulary,
            false,
        )?;
        let total = device.clone_htod(&[self.total()])?;
        let out = zeros::<f32>(device, 1)?;
        launch(
            device,
            "pointer_sum",
            1,
            &[
                Arg::d(&pass.row_value),
                Arg::d(&total),
                Arg::f(&out),
                Arg::U32(u32_of(rows, "rows")?),
            ],
        )?;
        Ok((storage(out, device), Shape::from(())))
    }

    /// The exact backward on CUDA: the logit, side and scale gradients.
    pub(super) fn cuda_bwd(
        &self,
        device: &CudaDevice,
        logits: &Tensor,
        side: &Tensor,
        beta: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        let (rows, vocabulary) = self.check(logits.layout(), side.layout(), beta.layout())?;
        self.cuda_check(vocabulary)?;
        if grad.elem_count() != 1 {
            candle_core::bail!("pointer mixture backward expects a scalar gradient");
        }
        let (lt, st, bt, gt) = (ready(logits)?, ready(side)?, ready(beta)?, ready(grad)?);
        let (ls, ll) = lt.storage_and_layout();
        let (ss, sl) = st.storage_and_layout();
        let (bs, bl) = bt.storage_and_layout();
        let (gs, gl) = gt.storage_and_layout();
        let (lv, sv, bv, gv) = (
            view(&ls, ll)?,
            view(&ss, sl)?,
            view(&bs, bl)?,
            view(&gs, gl)?,
        );
        let pass = self.cuda_pass(device, lv.slice(..), sv.slice(..), bv, gv, vocabulary, true)?;
        let dims = self.cuda_dims(rows, vocabulary, true)?;
        launch(
            device,
            "pointer_side_grad",
            rows * 2 * self.dim,
            &[
                Arg::F(sv),
                Arg::d(&pass.scratch),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::d(&pass.query_self),
                Arg::f(&pass.d_side),
                Arg::Dims(dims),
            ],
        )?;
        // d z_v = c share_generate (softmax_v - [v = target]): the
        // cross-entropy gradient kernel with the per-row factor.
        let d_logits = uninit::<f32>(device, rows * vocabulary)?;
        launch(
            device,
            "cross_entropy_grad",
            rows * vocabulary,
            &[
                Arg::F(lv),
                Arg::u(&pass.targets),
                Arg::d(&pass.lse),
                Arg::d(&pass.scale_z),
                Arg::f(&d_logits),
                Arg::U32(u32_of(rows, "rows")?),
                Arg::U32(u32_of(vocabulary, "vocabulary")?),
            ],
        )?;
        let one = device.clone_htod(&[1.0f64])?;
        let d_beta = zeros::<f32>(device, 1)?;
        launch(
            device,
            "pointer_sum",
            1,
            &[
                Arg::d(&pass.row_beta),
                Arg::d(&one),
                Arg::f(&d_beta),
                Arg::U32(u32_of(rows, "rows")?),
            ],
        )?;
        Ok((
            tensor(d_logits, device, logits.shape()),
            tensor(pass.d_side, device, side.shape()),
            tensor(d_beta, device, beta.shape()),
        ))
    }
}

// ---------------------------------------------------------------------------
// AdamW.

/// [`adam_step`] on CUDA, in place in the variable's and moments' buffers,
/// with the same f32 operations in the same order.
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

// ---------------------------------------------------------------------------
// Global gradient squared norm.

/// Threads per block of `sq_lanes`: one warp per block spreads a tensor's
/// (at most 1024) lanes over up to 32 multiprocessors.
const SQ_LANE_BLOCK: usize = 32;

/// The block width of Candle's `fast_sum` for an `n`-element full reduction.
fn fast_sum_width(n: usize) -> usize {
    n.min(1024).next_power_of_two()
}

/// `lanes[range]` as a view, or an error (never a panic) when out of range.
fn sub_view(lanes: &CudaSlice<f32>, start: usize, len: usize) -> CResult<CudaView<'_, f32>> {
    match lanes.try_slice(start..start + len) {
        Some(view) => Ok(view),
        None => candle_core::bail!("CUDA norm lane range exceeds its buffer"),
    }
}

/// Writes Candle's per-thread `fast_sum` lane sums of the `n` values in `x`
/// (squared first when `square`) into `out`, of length `fast_sum_width(n)`.
fn sum_lanes(
    device: &CudaDevice,
    x: CudaView<'_, f32>,
    n: usize,
    out: CudaView<'_, f32>,
    square: bool,
) -> CResult<()> {
    if n == 0 {
        return Ok(());
    }
    let width = fast_sum_width(n);
    launch_groups(
        device,
        "sq_lanes",
        (width.div_ceil(SQ_LANE_BLOCK), 1, 1),
        (SQ_LANE_BLOCK, 1, 1),
        &[
            Arg::F(x),
            Arg::F(out),
            Arg::U32(u32_of(n, "norm size")?),
            Arg::U32(u32_of(width, "norm width")?),
            Arg::U32(u32::from(square)),
        ],
    )
}

/// The sum over `tensors` of each one's squared entries, as one f32,
/// bit-identical to Candle's
/// `cat([t.sqr()?.sum_all()?.reshape(1)?, ..], 0)?.sum_all()?` on CUDA: the
/// same f32 operations in the same order (see `sq_lanes` in the kernels),
/// but each tensor's up-to-1024 lane sums run over many blocks instead of
/// Candle's single block per full reduction. One host read.
pub(super) fn squared_norm_cuda(device: &CudaDevice, tensors: &[&Tensor]) -> CResult<f32> {
    if tensors.is_empty() {
        candle_core::bail!("CUDA squared norm of no tensors");
    }
    let widths: Vec<usize> = tensors
        .iter()
        .map(|t| fast_sum_width(t.elem_count()))
        .collect();
    let lanes = zeros::<f32>(device, widths.iter().sum())?;
    let mut segments = Vec::with_capacity(2 * tensors.len());
    let mut offset = 0usize;
    for (tensor, &width) in tensors.iter().zip(&widths) {
        let tensor = ready(tensor)?;
        let n = tensor.elem_count();
        if n > 0 {
            let (storage, layout) = tensor.storage_and_layout();
            sum_lanes(
                device,
                view(&storage, layout)?,
                n,
                sub_view(&lanes, offset, width)?,
                true,
            )?;
        }
        segments.push(u32_of(offset, "norm offset")?);
        segments.push(u32_of(width, "norm width")?);
        offset += width;
    }
    let segments = device.clone_htod(&segments)?;
    let count = tensors.len();
    let sums = zeros::<f32>(device, count)?;
    launch_groups(
        device,
        "sq_tree",
        (count, 1, 1),
        (1024, 1, 1),
        &[Arg::f(&lanes), Arg::u(&segments), Arg::f(&sums)],
    )?;
    // The final `cat(..).sum_all()` over the per-tensor sums, unsquared.
    let width = fast_sum_width(count);
    let final_lanes = zeros::<f32>(device, width)?;
    sum_lanes(
        device,
        sub_view(&sums, 0, count)?,
        count,
        final_lanes.as_view(),
        false,
    )?;
    let segment = device.clone_htod(&[0u32, u32_of(width, "norm width")?])?;
    let total = zeros::<f32>(device, 1)?;
    launch_groups(
        device,
        "sq_tree",
        (1, 1, 1),
        (1024, 1, 1),
        &[Arg::f(&final_lanes), Arg::u(&segment), Arg::f(&total)],
    )?;
    tensor(total, device, &Shape::from(())).to_scalar::<f32>()
}
