//! CUDA paths of the geometric stack ops with bf16 activation storage.
//!
//! `precision=bf16` keeps every activation tensor of the trunk in bf16 and
//! stores it in bf16; this module is the CUDA side of those ops. It mirrors
//! [`super::cuda_ops`] (the f32 path) op for op and launch for launch: the
//! kernel names, grid shapes and argument orders are the same, and the kernels
//! come from the same CUDA C source compiled with `UOR_STORAGE_BF16`, so each
//! one loads its activation arguments, computes in f32 (or f64) and rounds on
//! store. The f32 module is untouched by this file, so a default run keeps
//! exactly the machine code it had.
//!
//! What stays f32 or f64 here, and why:
//!
//! - parameter tensors (the RMSNorm gain, the recurrence's taps/bias/decay,
//!   the read's `aux`: null bias, age table, `log_beta`, `offset`, and the
//!   pointer's `beta`): the master weights are f32 and only the matmul
//!   operands are rounded, so a parameter keeps its full precision and its
//!   gradient is f32;
//! - RMSNorm statistics, every score, probability, lift, excess and distance,
//!   the cross-entropy log-sum-exp and loss, the recurrence's carried state,
//!   drive, transition and `keep`, and the pointer's attention scratch: all
//!   internal buffers, accumulated in f32/f64 as before;
//! - the unit-quaternion transport products and norms: the transition is read
//!   from bf16 storage, then normalized and applied in f32;
//! - the gradients of the activation arguments are bf16, the gradients of the
//!   f32 parameters are f32.

use candle_core::backend::BackendStorage;
use candle_core::op::BackpropOp;
use candle_core::{CudaDevice, CudaStorage, DType, Storage, Tensor};
use cudarc::driver::{CudaSlice, CudaView};
use half::bf16;

use super::*;
use crate::cuda_stack_kernels::cuda::{
    launch, launch_bf16, launch_groups_bf16, uninit, zeros, Arg,
};

type CResult<T> = candle_core::Result<T>;
type Forward = CResult<(CudaStorage, Shape)>;

// ---------------------------------------------------------------------------
// bf16 buffer helpers (the f32 counterparts live in `super::cuda_ops`).

/// The bf16 elements of a contiguous CUDA activation input (any start
/// offset), or `None` when the layout is not contiguous.
fn bf_input<'a>(storage: &'a CudaStorage, layout: &Layout) -> CResult<Option<CudaView<'a, bf16>>> {
    if !layout.is_contiguous() || storage.dtype() != DType::BF16 {
        return Ok(None);
    }
    let slice = storage.as_cuda_slice::<bf16>()?;
    let start = layout.start_offset();
    let len = layout.shape().elem_count();
    match slice.try_slice(start..start + len) {
        Some(view) => Ok(Some(view)),
        None => candle_core::bail!("CUDA input layout exceeds its buffer"),
    }
}

/// The bf16 elements of a CUDA tensor's storage prepared by [`ready_bf`].
fn bf_view<'a>(storage: &'a Storage, layout: &Layout) -> CResult<CudaView<'a, bf16>> {
    let Storage::Cuda(storage) = storage else {
        candle_core::bail!("expected a CUDA tensor");
    };
    match bf_input(storage, layout)? {
        Some(view) => Ok(view),
        None => candle_core::bail!("CUDA bf16 stack kernels need contiguous bf16 tensors"),
    }
}

/// A CUDA bf16 tensor laid out contiguously (copied only when it is not).
fn ready_bf(tensor: &Tensor) -> CResult<Tensor> {
    if tensor.dtype() != DType::BF16 {
        candle_core::bail!("CUDA bf16 stack kernels require bf16 activation tensors");
    }
    if tensor.is_contiguous() {
        Ok(tensor.clone())
    } else {
        tensor.force_contiguous()
    }
}

/// A CUDA f32 tensor (a parameter or scalar argument) laid out contiguously.
fn ready_f32(tensor: &Tensor) -> CResult<Tensor> {
    if tensor.dtype() != DType::F32 {
        candle_core::bail!("CUDA bf16 stack kernels require f32 parameter tensors");
    }
    if tensor.is_contiguous() {
        Ok(tensor.clone())
    } else {
        tensor.force_contiguous()
    }
}

/// The f32 elements of a CUDA tensor's storage prepared by [`ready_f32`].
fn f32_view<'a>(storage: &'a Storage, layout: &Layout) -> CResult<CudaView<'a, f32>> {
    let Storage::Cuda(storage) = storage else {
        candle_core::bail!("expected a CUDA tensor");
    };
    if !layout.is_contiguous() || storage.dtype() != DType::F32 {
        candle_core::bail!("CUDA bf16 stack kernels need contiguous f32 parameter tensors");
    }
    let slice = storage.as_cuda_slice::<f32>()?;
    let start = layout.start_offset();
    let len = layout.shape().elem_count();
    match slice.try_slice(start..start + len) {
        Some(view) => Ok(view),
        None => candle_core::bail!("CUDA input layout exceeds its buffer"),
    }
}

fn storage_bf(slice: CudaSlice<bf16>, device: &CudaDevice) -> CudaStorage {
    CudaStorage::wrap_cuda_slice(slice, device.clone())
}

/// A bf16 tensor over a fresh CUDA buffer.
fn tensor_bf(slice: CudaSlice<bf16>, device: &CudaDevice, shape: &Shape) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(storage_bf(slice, device)),
        shape.clone(),
        BackpropOp::none(),
        false,
    )
}

fn storage_f32(slice: CudaSlice<f32>, device: &CudaDevice) -> CudaStorage {
    CudaStorage::wrap_cuda_slice(slice, device.clone())
}

/// An f32 tensor over a fresh CUDA buffer.
fn tensor_f32(slice: CudaSlice<f32>, device: &CudaDevice, shape: &Shape) -> Tensor {
    Tensor::from_storage(
        Storage::Cuda(storage_f32(slice, device)),
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
    _s1: &CudaStorage,
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
    let Some(served) = bf_input(s2, l2)? else {
        candle_core::bail!("bf16 straight-through needs contiguous bf16 tensors");
    };
    let device = &s2.device;
    let out = uninit::<bf16>(device, total)?;
    launch_bf16(
        device,
        "straight_through_fwd",
        total,
        &[
            Arg::B(served),
            Arg::b(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((storage_bf(out, device), l2.shape().clone()))
}

pub(super) fn swiglu_fwd(s1: &CudaStorage, l1: &Layout, s2: &CudaStorage, l2: &Layout) -> Forward {
    if l1.shape() != l2.shape() {
        candle_core::bail!("SwiGLU inputs must match");
    }
    let (Some(gate), Some(up)) = (bf_input(s1, l1)?, bf_input(s2, l2)?) else {
        candle_core::bail!("bf16 SwiGLU needs contiguous bf16 tensors");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let out = uninit::<bf16>(device, total)?;
    launch_bf16(
        device,
        "swiglu_fwd",
        total,
        &[
            Arg::B(gate),
            Arg::B(up),
            Arg::b(&out),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((storage_bf(out, device), l1.shape().clone()))
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
    let (g, u, d) = (ready_bf(gate)?, ready_bf(up)?, ready_bf(grad)?);
    let total = g.elem_count();
    let (gs, gl) = g.storage_and_layout();
    let (us, ul) = u.storage_and_layout();
    let (ds, dl) = d.storage_and_layout();
    let d_gate = uninit::<bf16>(device, total)?;
    let d_up = uninit::<bf16>(device, total)?;
    launch_bf16(
        device,
        "swiglu_bwd",
        total,
        &[
            Arg::B(bf_view(&gs, gl)?),
            Arg::B(bf_view(&us, ul)?),
            Arg::B(bf_view(&ds, dl)?),
            Arg::b(&d_gate),
            Arg::b(&d_up),
            Arg::U32(u32_of(total, "size")?),
        ],
    )?;
    Ok((
        tensor_bf(d_gate, device, gate.shape()),
        tensor_bf(d_up, device, up.shape()),
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
    if s2.dtype() != DType::F32 {
        candle_core::bail!("bf16 RMSNorm keeps its gain in f32");
    }
    let Some(x) = bf_input(s1, l1)? else {
        candle_core::bail!("bf16 RMSNorm needs a contiguous bf16 input");
    };
    let slice = s2.as_cuda_slice::<f32>()?;
    let start = l2.start_offset();
    let len = l2.shape().elem_count();
    let Some(w) = slice.try_slice(start..start + len) else {
        candle_core::bail!("CUDA input layout exceeds its buffer");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let rows = total / width;
    let out = uninit::<bf16>(device, total)?;
    launch_groups_bf16(
        device,
        "rms_norm_fwd",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::B(x),
            Arg::F(w),
            Arg::b(&out),
            Arg::U32(u32_of(width, "width")?),
        ],
    )?;
    Ok((storage_bf(out, device), l1.shape().clone()))
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
    let (xr, wr, gr) = (ready_bf(x)?, ready_f32(w)?, ready_bf(grad)?);
    let total = xr.elem_count();
    let rows = total / width;
    let (xs, xl) = xr.storage_and_layout();
    let (ws, wl) = wr.storage_and_layout();
    let (gs, gl) = gr.storage_and_layout();
    let (xv, wv, gv) = (bf_view(&xs, xl)?, f32_view(&ws, wl)?, bf_view(&gs, gl)?);
    let dx = uninit::<bf16>(device, total)?;
    let row_r = uninit::<f64>(device, rows)?;
    launch_groups_bf16(
        device,
        "rms_norm_bwd_dx",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::B(xv.slice(..)),
            Arg::F(wv),
            Arg::B(gv.slice(..)),
            Arg::b(&dx),
            Arg::d(&row_r),
            Arg::U32(u32_of(width, "width")?),
        ],
    )?;
    let chunks = rows.div_ceil(RMS_CHUNK_ROWS);
    let partials = uninit::<f64>(device, chunks * width)?;
    launch_bf16(
        device,
        "rms_norm_dw_partial",
        chunks * width,
        &[
            Arg::B(xv),
            Arg::B(gv),
            Arg::d(&row_r),
            Arg::d(&partials),
            Arg::U32(u32_of(rows, "rows")?),
            Arg::U32(u32_of(width, "width")?),
            Arg::U32(RMS_CHUNK_ROWS as u32),
        ],
    )?;
    let dw = uninit::<f32>(device, width)?;
    launch_bf16(
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
    Ok((
        tensor_bf(dx, device, x.shape()),
        tensor_f32(dw, device, w.shape()),
    ))
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
    let (Some(transition), Some(drive)) = (bf_input(s1, l1)?, bf_input(s2, l2)?) else {
        candle_core::bail!("bf16 quaternion scan needs contiguous bf16 tensors");
    };
    let device = &s1.device;
    let total = l1.shape().elem_count();
    let out = uninit::<bf16>(device, total)?;
    launch_bf16(
        device,
        "quaternion_scan_fwd",
        batch * lanes,
        &[
            Arg::B(transition),
            Arg::B(drive),
            Arg::b(&out),
            Arg::U32(u32_of(time, "time")?),
            Arg::U32(u32_of(lanes, "lanes")?),
            Arg::U32(u32_of(batch * lanes, "sequences")?),
        ],
    )?;
    Ok((storage_bf(out, device), l1.shape().clone()))
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
    let (t, s, g) = (ready_bf(transition)?, ready_bf(state)?, ready_bf(grad)?);
    let total = t.elem_count();
    let (ts, tl) = t.storage_and_layout();
    let (ss, sl) = s.storage_and_layout();
    let (gs, gl) = g.storage_and_layout();
    let dq = uninit::<bf16>(device, total)?;
    let db = uninit::<bf16>(device, total)?;
    launch_bf16(
        device,
        "quaternion_scan_bwd",
        batch * lanes,
        &[
            Arg::B(bf_view(&ts, tl)?),
            Arg::B(bf_view(&ss, sl)?),
            Arg::B(bf_view(&gs, gl)?),
            Arg::b(&dq),
            Arg::b(&db),
            Arg::U32(u32_of(time, "time")?),
            Arg::U32(u32_of(lanes, "lanes")?),
            Arg::U32(u32_of(batch * lanes, "sequences")?),
        ],
    )?;
    Ok((
        tensor_bf(dq, device, transition.shape()),
        tensor_bf(db, device, transition.shape()),
    ))
}

// ---------------------------------------------------------------------------
// Cross-entropy: bf16 logits and logit gradient, f64 log-sum-exp and loss.

fn cross_entropy_rows_bf(
    device: &CudaDevice,
    logits: CudaView<'_, bf16>,
    targets: &CudaSlice<u32>,
    rows: usize,
    vocabulary: usize,
) -> CResult<(CudaSlice<f64>, CudaSlice<f64>)> {
    let lse = zeros::<f64>(device, rows)?;
    let loss = zeros::<f64>(device, rows)?;
    launch_groups_bf16(
        device,
        "cross_entropy_rows_act",
        (rows, 1, 1),
        (ROW_THREADS, 1, 1),
        &[
            Arg::B(logits),
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
    let Some(logits) = bf_input(s, l)? else {
        candle_core::bail!("bf16 CrossEntropy needs contiguous bf16 logits");
    };
    let device = &s.device;
    let targets = device.clone_htod(&op.targets)?;
    let (_, loss) = cross_entropy_rows_bf(device, logits, &targets, rows, vocabulary)?;
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
    Ok((storage_f32(out, device), Shape::from(())))
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
    let l = ready_bf(logits)?;
    let (ls, ll) = l.storage_and_layout();
    let lv = bf_view(&ls, ll)?;
    let targets = device.clone_htod(&op.targets)?;
    let scale = device.clone_htod(&scales)?;
    let (lse, _) = cross_entropy_rows_bf(device, lv.slice(..), &targets, rows, vocabulary)?;
    let out = uninit::<bf16>(device, rows * vocabulary)?;
    launch_bf16(
        device,
        "cross_entropy_grad_act",
        rows * vocabulary,
        &[
            Arg::B(lv),
            Arg::u(&targets),
            Arg::d(&lse),
            Arg::d(&scale),
            Arg::b(&out),
            Arg::U32(u32_of(rows, "rows")?),
            Arg::U32(u32_of(vocabulary, "vocabulary")?),
        ],
    )?;
    Ok(tensor_bf(out, device, logits.shape()))
}

// ---------------------------------------------------------------------------
// The recurrence core.

/// Threads per block of the serial recurrence scans.
const SCAN_GROUP: usize = 32;
/// Threads per block of the per-(window, channel) parameter sweep.
const PARAMS_GROUP: usize = 64;

/// Launches a serial recurrence scan over `threads` (window, lane) threads.
fn launch_scan(device: &CudaDevice, name: &str, threads: usize, args: &[Arg<'_>]) -> CResult<()> {
    launch_groups_bf16(
        device,
        name,
        (threads.div_ceil(SCAN_GROUP), 1, 1),
        (SCAN_GROUP, 1, 1),
        args,
    )
}

/// The split recurrence path's per-position device buffers.
struct SplitStates {
    drive: CudaSlice<f32>,
    q: CudaSlice<f32>,
    keep: CudaSlice<f32>,
    state: CudaSlice<f32>,
}

impl RecurrenceCore {
    /// `log a` per lane on the device (the decay parameters are f32).
    fn cuda_log_a_bf(
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

    /// The single-kernel forward: states, drives and outputs.
    #[allow(clippy::type_complexity)]
    fn cuda_forward_bf(
        &self,
        device: &CudaDevice,
        branches: CudaView<'_, bf16>,
        gates: CudaView<'_, bf16>,
        parameters: CudaView<'_, f32>,
        log_a: &CudaSlice<f32>,
    ) -> CResult<(CudaSlice<f32>, CudaSlice<f32>, CudaSlice<bf16>)> {
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let total = self.batch * time * width;
        let state = uninit::<f32>(device, total)?;
        let drive = uninit::<f32>(device, total)?;
        let out = uninit::<bf16>(device, total)?;
        launch_bf16(
            device,
            "recurrence_core_fwd",
            self.batch * lanes,
            &[
                Arg::B(branches),
                Arg::B(gates),
                Arg::F(parameters),
                Arg::f(log_a),
                Arg::f(&state),
                Arg::f(&drive),
                Arg::b(&out),
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

    /// The split forward's per-position buffers and states.
    fn cuda_split_states_bf(
        &self,
        device: &CudaDevice,
        branches: &CudaView<'_, bf16>,
        gates: &CudaView<'_, bf16>,
        parameters: &CudaView<'_, f32>,
        log_a: &CudaSlice<f32>,
    ) -> CResult<SplitStates> {
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let total = self.batch * time * width;
        let positions = self.batch * time * lanes;
        let drive = zeros::<f32>(device, total)?;
        let q = zeros::<f32>(device, total)?;
        let keep = zeros::<f32>(device, positions)?;
        let state = zeros::<f32>(device, total)?;
        launch_bf16(
            device,
            "recurrence_prep",
            positions,
            &[
                Arg::B(branches.slice(..)),
                Arg::B(gates.slice(..)),
                Arg::F(parameters.slice(..)),
                Arg::f(log_a),
                Arg::f(&drive),
                Arg::f(&q),
                Arg::f(&keep),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(self.gate_width(), "gate width")?),
                Arg::U32(self.cuda_rotation()),
                Arg::U32(u32_of(positions, "positions")?),
            ],
        )?;
        // The same time-parallel carry as the f32 path: its buffers are f32 in
        // both modules (recurrence_prep did the storage conversion).
        super::cuda_ops::time_parallel_states(
            device, self.batch, time, width, lanes, &q, &drive, &keep, &state,
        )?;
        Ok(SplitStates {
            drive,
            q,
            keep,
            state,
        })
    }

    pub(super) fn cuda_fwd_impl_bf(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
        self.cuda_check()?;
        let (time, width) = (self.time, self.width);
        if l1.shape().elem_count() != self.batch * time * 2 * width
            || l2.shape().elem_count() != self.batch * time * self.gate_width()
            || l3.shape().elem_count() != self.parameter_len()
        {
            candle_core::bail!("recurrence core inputs have the wrong sizes");
        }
        let (Some(branches), Some(gates)) = (bf_input(s1, l1)?, bf_input(s2, l2)?) else {
            candle_core::bail!("bf16 recurrence core needs contiguous bf16 activations");
        };
        if s3.dtype() != DType::F32 {
            candle_core::bail!("bf16 recurrence core keeps its parameters in f32");
        }
        let ps = s3.as_cuda_slice::<f32>()?;
        let start = l3.start_offset();
        let len = l3.shape().elem_count();
        let Some(parameters) = ps.try_slice(start..start + len) else {
            candle_core::bail!("CUDA input layout exceeds its buffer");
        };
        let device = &s1.device;
        let log_a = self.cuda_log_a_bf(device, parameters.slice(..))?;
        let out = match cuda_recurrence_kernels() {
            CudaRecurrenceKernels::Single => {
                self.cuda_forward_bf(device, branches, gates, parameters, &log_a)?
                    .2
            }
            CudaRecurrenceKernels::Split | CudaRecurrenceKernels::Chunked => {
                let states =
                    self.cuda_split_states_bf(device, &branches, &gates, &parameters, &log_a)?;
                let total = self.batch * time * width;
                let out = zeros::<bf16>(device, total)?;
                launch_bf16(
                    device,
                    "recurrence_out",
                    total,
                    &[
                        Arg::f(&states.state),
                        Arg::B(branches),
                        Arg::b(&out),
                        Arg::U32(u32_of(width, "width")?),
                        Arg::U32(u32_of(total, "elements")?),
                    ],
                )?;
                out
            }
        };
        Ok((
            storage_bf(out, device),
            Shape::from((self.batch, time, width)),
        ))
    }

    /// The split backward.
    #[allow(clippy::too_many_arguments)]
    fn cuda_split_bwd_bf(
        &self,
        device: &CudaDevice,
        bv: &CudaView<'_, bf16>,
        gv: &CudaView<'_, bf16>,
        pv: &CudaView<'_, f32>,
        dv: &CudaView<'_, bf16>,
        log_a: &CudaSlice<f32>,
        d_branches: &CudaSlice<bf16>,
        d_gates: &CudaSlice<bf16>,
        partials: &CudaSlice<f64>,
    ) -> CResult<()> {
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let total = self.batch * time * width;
        let positions = self.batch * time * lanes;
        let windows = self.batch * lanes;
        let channels = self.batch * width;
        let states = self.cuda_split_states_bf(device, bv, gv, pv, log_a)?;
        let direct = zeros::<f32>(device, total)?;
        launch_bf16(
            device,
            "recurrence_bwd_direct",
            positions,
            &[
                Arg::B(bv.slice(..)),
                Arg::f(&states.state),
                Arg::B(dv.slice(..)),
                Arg::b(d_branches),
                Arg::f(&direct),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(positions, "positions")?),
            ],
        )?;
        launch_scan(
            device,
            "recurrence_scan_bwd",
            windows,
            &[
                Arg::f(&states.q),
                Arg::B(dv.slice(..)),
                Arg::f(&direct),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(windows, "lanes")?),
            ],
        )?;
        launch_bf16(
            device,
            "recurrence_bwd_post",
            positions,
            &[
                Arg::B(gv.slice(..)),
                Arg::f(log_a),
                Arg::f(&states.state),
                Arg::f(&states.drive),
                Arg::f(&direct),
                Arg::b(d_gates),
                Arg::f(&states.q),
                Arg::f(&states.keep),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(self.gate_width(), "gate width")?),
                Arg::U32(self.cuda_rotation()),
                Arg::U32(u32_of(positions, "positions")?),
            ],
        )?;
        launch_groups_bf16(
            device,
            "recurrence_bwd_params",
            (channels.div_ceil(PARAMS_GROUP), 1, 1),
            (PARAMS_GROUP, 1, 1),
            &[
                Arg::B(bv.slice(..)),
                Arg::F(pv.slice(..)),
                Arg::f(&states.q),
                Arg::f(&states.keep),
                Arg::b(d_branches),
                Arg::d(partials),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(channels, "channels")?),
                Arg::U32(u32_of(self.parameter_len(), "parameters")?),
            ],
        )?;
        Ok(())
    }

    /// The single-kernel backward.
    fn cuda_single_bwd_bf(
        &self,
        device: &CudaDevice,
        [bv, gv, dv]: [&CudaView<'_, bf16>; 3],
        pv_f32: &CudaView<'_, f32>,
        log_a: &CudaSlice<f32>,
        d_branches: &CudaSlice<bf16>,
        d_gates: &CudaSlice<bf16>,
        partials: &CudaSlice<f64>,
    ) -> CResult<()> {
        let (time, width, lanes) = (self.time, self.width, self.lanes());
        let gate_width = self.gate_width();
        let param_len = self.parameter_len();
        let (state, drive, _) =
            self.cuda_forward_bf(device, bv.slice(..), gv.slice(..), pv_f32.slice(..), log_a)?;
        launch_bf16(
            device,
            "recurrence_core_bwd",
            self.batch * lanes,
            &[
                Arg::B(bv.slice(..)),
                Arg::B(gv.slice(..)),
                Arg::F(pv_f32.slice(..)),
                Arg::f(log_a),
                Arg::f(&state),
                Arg::f(&drive),
                Arg::B(dv.slice(..)),
                Arg::b(d_branches),
                Arg::b(d_gates),
                Arg::d(partials),
                Arg::U32(u32_of(time, "time")?),
                Arg::U32(u32_of(width, "width")?),
                Arg::U32(u32_of(lanes, "lanes")?),
                Arg::U32(u32_of(gate_width, "gate width")?),
                Arg::U32(self.cuda_rotation()),
                Arg::U32(u32_of(self.batch * lanes, "lanes")?),
                Arg::U32(u32_of(param_len, "parameters")?),
            ],
        )
    }

    /// The exact backward on CUDA with bf16 activation storage.
    pub(super) fn cuda_bwd_bf(
        &self,
        device: &CudaDevice,
        branches: &Tensor,
        gates: &Tensor,
        parameters: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        self.cuda_check()?;
        let (b, g, p, dy) = (
            ready_bf(branches)?,
            ready_bf(gates)?,
            ready_f32(parameters)?,
            ready_bf(grad)?,
        );
        let (time, width) = (self.time, self.width);
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
            bf_view(&bs, bl)?,
            bf_view(&gs, gl)?,
            f32_view(&ps, pl)?,
            bf_view(&ds, dl)?,
        );
        let log_a = self.cuda_log_a_bf(device, pv.slice(..))?;
        let d_branches = zeros::<bf16>(device, b.elem_count())?;
        let d_gates = zeros::<bf16>(device, g.elem_count())?;
        let partials = zeros::<f64>(device, self.batch * param_len)?;
        let d_parameters = zeros::<f32>(device, param_len)?;
        if cuda_recurrence_kernels() != CudaRecurrenceKernels::Single {
            self.cuda_split_bwd_bf(
                device,
                &bv,
                &gv,
                &pv,
                &dv,
                &log_a,
                &d_branches,
                &d_gates,
                &partials,
            )?;
        } else {
            self.cuda_single_bwd_bf(
                device,
                [&bv, &gv, &dv],
                &pv,
                &log_a,
                &d_branches,
                &d_gates,
                &partials,
            )?;
        }
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
            tensor_bf(d_branches, device, branches.shape()),
            tensor_bf(d_gates, device, gates.shape()),
            tensor_f32(d_parameters, device, parameters.shape()),
        ))
    }
}

// ---------------------------------------------------------------------------
// The fused read: bf16 query/key/value and their gradients, f32 aux.

/// The device buffers of one recomputed read.
struct CudaReadPass {
    probabilities: CudaSlice<f32>,
    null_probability: CudaSlice<f32>,
    query_lift: CudaSlice<f64>,
    key_lift: CudaSlice<f64>,
    excess: CudaSlice<f64>,
}

impl FusedRead {
    fn cuda_pass_bf(
        &self,
        device: &CudaDevice,
        query: CudaView<'_, bf16>,
        kv: CudaView<'_, bf16>,
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
            launch_bf16(
                device,
                "read_lift",
                rows,
                &[
                    Arg::B(query.slice(..)),
                    Arg::B(kv.slice(..)),
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
        launch_groups_bf16(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::B(query),
                Arg::B(kv),
                Arg::F(aux.slice(..)),
                Arg::d(&query_lift),
                Arg::d(&key_lift),
                Arg::f(&probabilities),
                Arg::d(&excess),
                Arg::Dims(dims),
                Arg::Geom(geometry),
            ],
        )?;
        launch_bf16(
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

    pub(super) fn cuda_fwd_impl_bf(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
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
        if s3.dtype() != DType::F32 {
            candle_core::bail!("bf16 FusedRead keeps its auxiliary table in f32");
        }
        let (Some(query), Some(kv)) = (bf_input(s1, l1)?, bf_input(s2, l2)?) else {
            candle_core::bail!("bf16 FusedRead needs contiguous bf16 query and key/value");
        };
        let as_ = s3.as_cuda_slice::<f32>()?;
        let start = l3.start_offset();
        let len = l3.shape().elem_count();
        let Some(aux) = as_.try_slice(start..start + len) else {
            candle_core::bail!("CUDA input layout exceeds its buffer");
        };
        let device = &s1.device;
        let total = rows * value;
        if cuda_read_kernels() == CudaReadKernels::Flash {
            let dims = self.cuda_dims()?;
            let lift_len = if self.score.scaled() { rows } else { 1 };
            let query_lift = uninit::<f64>(device, lift_len)?;
            let key_lift = uninit::<f64>(device, lift_len)?;
            if self.score.scaled() {
                launch_bf16(
                    device,
                    "read_lift",
                    rows,
                    &[
                        Arg::B(query.slice(..)),
                        Arg::B(kv.slice(..)),
                        Arg::d(&query_lift),
                        Arg::d(&key_lift),
                        Arg::Dims(dims),
                    ],
                )?;
            }
            let out = uninit::<bf16>(device, total)?;
            launch_groups_bf16(
                device,
                "read_flash_fwd",
                (time.div_ceil(16), 1, self.batch * self.heads),
                (16, 16, 1),
                &[
                    Arg::B(query),
                    Arg::B(kv),
                    Arg::F(aux),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::b(&out),
                    Arg::Dims(dims),
                ],
            )?;
            return Ok((
                storage_bf(out, device),
                Shape::from((self.batch, self.heads, time, value)),
            ));
        }
        let pass = self.cuda_pass_bf(device, query, kv.slice(..), aux, false)?;
        let out = uninit::<bf16>(device, total)?;
        launch_bf16(
            device,
            "read_mix",
            total,
            &[
                Arg::f(&pass.probabilities),
                Arg::B(kv),
                Arg::b(&out),
                Arg::Dims(self.cuda_dims()?),
            ],
        )?;
        Ok((
            storage_bf(out, device),
            Shape::from((self.batch, self.heads, time, value)),
        ))
    }

    /// The exact backward on CUDA with bf16 activation storage: the read's
    /// output gradient, query, keys/values and their gradients are bf16; the
    /// score-space buffers (`dp`, `inner_grad`, probabilities, lifts, excess,
    /// the row partials) stay f32/f64, and the auxiliary table is f32.
    pub(super) fn cuda_bwd_bf(
        &self,
        device: &CudaDevice,
        query: &Tensor,
        kv: &Tensor,
        aux: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        self.cuda_check()?;
        let (q, kvt, a, dy) = (
            ready_bf(query)?,
            ready_bf(kv)?,
            ready_f32(aux)?,
            ready_bf(grad)?,
        );
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
            bf_view(&qs, ql)?,
            bf_view(&ks, kl)?,
            f32_view(&as_, al)?,
            bf_view(&ds, dl)?,
        );
        let scaled = self.score.scaled();
        if cuda_read_kernels() == CudaReadKernels::Flash {
            let dims = self.cuda_dims()?;
            let lift_len = if scaled { rows } else { 1 };
            let query_lift = uninit::<f64>(device, lift_len)?;
            let key_lift = uninit::<f64>(device, lift_len)?;
            if scaled {
                launch_bf16(
                    device,
                    "read_lift",
                    rows,
                    &[
                        Arg::B(qv.slice(..)),
                        Arg::B(kvv.slice(..)),
                        Arg::d(&query_lift),
                        Arg::d(&key_lift),
                        Arg::Dims(dims),
                    ],
                )?;
            }
            let tiles = self.time.div_ceil(16);
            let row_len = if scaled { rows } else { 1 };
            let age_len = if self.age { rows * tiles } else { 1 };
            let rowstat = uninit::<f64>(device, 3 * rows)?;
            let row_beta = uninit::<f64>(device, row_len)?;
            let row_offset = uninit::<f64>(device, row_len)?;
            let age_part = uninit::<f64>(device, age_len)?;
            let dq = uninit::<bf16>(device, q.elem_count())?;
            let dkv = uninit::<bf16>(device, kvt.elem_count())?;
            let d_aux = zeros::<f32>(device, a.elem_count())?;
            let grid = (tiles, 1, self.batch * self.heads);
            launch_groups_bf16(
                device,
                "read_flash_bwd_query",
                grid,
                (16, 16, 1),
                &[
                    Arg::B(qv.slice(..)),
                    Arg::B(kvv.slice(..)),
                    Arg::B(dyv.slice(..)),
                    Arg::F(av.slice(..)),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::d(&rowstat),
                    Arg::b(&dq),
                    Arg::f(&d_aux),
                    Arg::d(&row_beta),
                    Arg::d(&row_offset),
                    Arg::d(&age_part),
                    Arg::Dims(dims),
                ],
            )?;
            launch_groups_bf16(
                device,
                "read_flash_bwd_key",
                grid,
                (16, 16, 1),
                &[
                    Arg::B(qv.slice(..)),
                    Arg::B(kvv.slice(..)),
                    Arg::B(dyv.slice(..)),
                    Arg::F(av.slice(..)),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::d(&rowstat),
                    Arg::b(&dkv),
                    Arg::Dims(dims),
                ],
            )?;
            if self.age || scaled {
                launch_bf16(
                    device,
                    "read_flash_bwd_reduce",
                    self.heads * self.time + 2 * self.heads,
                    &[
                        Arg::d(&age_part),
                        Arg::d(&row_beta),
                        Arg::d(&row_offset),
                        Arg::f(&d_aux),
                        Arg::Dims(dims),
                    ],
                )?;
            }
            let d_aux = if self.null || self.age || scaled {
                tensor_f32(d_aux, device, aux.shape())
            } else {
                Tensor::zeros(aux.shape(), DType::F32, aux.device())?
            };
            return Ok((
                tensor_bf(dq, device, query.shape()),
                tensor_bf(dkv, device, kv.shape()),
                d_aux,
            ));
        }
        let pass = self.cuda_pass_bf(device, qv.slice(..), kvv.slice(..), av.slice(..), true)?;
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
        let dq = uninit::<bf16>(device, q.elem_count())?;
        let dkv = uninit::<bf16>(device, kvt.elem_count())?;
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
        launch_groups_bf16(
            device,
            "read_tile_inner",
            (tiles, tiles, self.batch * self.heads),
            (16, 16, 1),
            &[
                Arg::B(dyv.slice(..)),
                Arg::B(kvv.slice(..)),
                Arg::F(av.slice(..)),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::f(&dp),
                Arg::d(&pass.excess),
                Arg::Dims(dims),
                Arg::Geom(geometry),
            ],
        )?;
        launch_bf16(
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
        launch_bf16(
            device,
            "read_dq",
            q.elem_count(),
            &[
                Arg::f(&inner_grad),
                Arg::B(qv.slice(..)),
                Arg::B(kvv.slice(..)),
                Arg::d(&query_self),
                Arg::b(&dq),
                Arg::Dims(dims),
            ],
        )?;
        launch_bf16(
            device,
            "read_dkv",
            kvt.elem_count(),
            &[
                Arg::f(&inner_grad),
                Arg::f(&pass.probabilities),
                Arg::B(qv),
                Arg::B(kvv),
                Arg::B(dyv),
                Arg::d(&key_self),
                Arg::b(&dkv),
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
            tensor_f32(d_aux, device, aux.shape())
        } else {
            // The placeholder auxiliary input carries no gradient.
            Tensor::zeros(aux.shape(), DType::F32, aux.device())?
        };
        Ok((
            tensor_bf(dq, device, query.shape()),
            tensor_bf(dkv, device, kv.shape()),
            d_aux,
        ))
    }
}

// ---------------------------------------------------------------------------
// The pointer-copy mixture: bf16 logits and side, f32 `beta`.

/// The device buffers of one evaluated batch of pointer rows.
struct CudaPointerPass {
    scratch: CudaSlice<f64>,
    row_value: CudaSlice<f64>,
    scale_z: CudaSlice<f64>,
    d_side: CudaSlice<bf16>,
    row_beta: CudaSlice<f64>,
    query_self: CudaSlice<f64>,
    query_lift: CudaSlice<f64>,
    key_lift: CudaSlice<f64>,
    lse: CudaSlice<f64>,
    targets: CudaSlice<u32>,
}

impl PointerMixture {
    fn cuda_dims_bf(&self, rows: usize, vocabulary: usize, backward: bool) -> CResult<[u32; 8]> {
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

    fn cuda_check_bf(&self, vocabulary: usize) -> CResult<()> {
        if self.dim == 0 || vocabulary == 0 {
            candle_core::bail!("CUDA pointer mixture requires positive dimensions");
        }
        if self.targets.iter().any(|&t| t as usize >= vocabulary) {
            candle_core::bail!("pointer mixture target outside the vocabulary");
        }
        Ok(())
    }

    /// The weights as uploaded (ones when unweighted).
    fn cuda_weights_bf(&self, rows: usize) -> Vec<f32> {
        self.weights.clone().unwrap_or_else(|| vec![1.0; rows])
    }

    /// Log-sum-exps over the bf16 logits, lifts and the row pass on the
    /// device. `grad` is the one-element upstream gradient (bf16).
    #[allow(clippy::too_many_arguments)]
    fn cuda_pass_bf(
        &self,
        device: &CudaDevice,
        logits: CudaView<'_, bf16>,
        side: CudaView<'_, bf16>,
        beta: CudaView<'_, f32>,
        grad: CudaView<'_, bf16>,
        vocabulary: usize,
        backward: bool,
    ) -> CResult<CudaPointerPass> {
        let rows = self.targets.len();
        let dims = self.cuda_dims_bf(rows, vocabulary, backward)?;
        let targets = device.clone_htod(&self.targets)?;
        let ids = device.clone_htod(&self.ids)?;
        let weights = device.clone_htod(&self.cuda_weights_bf(rows))?;
        let total = device.clone_htod(&[self.total()])?;
        let (lse, _) = cross_entropy_rows_bf(device, logits.slice(..), &targets, rows, vocabulary)?;
        let query_lift = zeros::<f64>(device, rows)?;
        let key_lift = zeros::<f64>(device, rows)?;
        if self.score == ReadScore::Lorentz {
            launch_bf16(
                device,
                "pointer_lift",
                rows,
                &[
                    Arg::B(side.slice(..)),
                    Arg::d(&query_lift),
                    Arg::d(&key_lift),
                    Arg::Dims(dims),
                ],
            )?;
        }
        let pass = CudaPointerPass {
            scratch: zeros::<f64>(device, rows * self.time)?,
            row_value: zeros::<f64>(device, rows)?,
            scale_z: zeros::<f64>(device, rows)?,
            d_side: zeros::<bf16>(
                device,
                if backward {
                    rows * (2 * self.dim + 1)
                } else {
                    1
                },
            )?,
            row_beta: zeros::<f64>(device, rows)?,
            query_self: zeros::<f64>(device, rows)?,
            query_lift,
            key_lift,
            lse,
            targets,
        };
        launch_bf16(
            device,
            "pointer_rows",
            32 * rows,
            &[
                Arg::B(logits),
                Arg::B(side),
                Arg::F(beta),
                Arg::u(&ids),
                Arg::u(&pass.targets),
                Arg::f(&weights),
                Arg::d(&pass.lse),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::B(grad),
                Arg::d(&total),
                Arg::d(&pass.scratch),
                Arg::d(&pass.row_value),
                Arg::d(&pass.scale_z),
                Arg::b(&pass.d_side),
                Arg::d(&pass.row_beta),
                Arg::d(&pass.query_self),
                Arg::Dims(dims),
            ],
        )?;
        Ok(pass)
    }

    pub(super) fn cuda_fwd_impl_bf(
        &self,
        s1: &CudaStorage,
        l1: &Layout,
        s2: &CudaStorage,
        l2: &Layout,
        s3: &CudaStorage,
        l3: &Layout,
    ) -> Forward {
        let (rows, vocabulary) = self.check(l1, l2, l3)?;
        self.cuda_check_bf(vocabulary)?;
        let (Some(logits), Some(side)) = (bf_input(s1, l1)?, bf_input(s2, l2)?) else {
            candle_core::bail!("bf16 pointer mixture needs contiguous bf16 logits and side");
        };
        if s3.dtype() != DType::F32 {
            candle_core::bail!("bf16 pointer mixture keeps its beta in f32");
        }
        let bs = s3.as_cuda_slice::<f32>()?;
        let start = l3.start_offset();
        let len = l3.shape().elem_count();
        let Some(beta) = bs.try_slice(start..start + len) else {
            candle_core::bail!("CUDA input layout exceeds its buffer");
        };
        let device = &s1.device;
        // The forward reads only the row values; the one-element gradient
        // slot is a bf16 placeholder.
        let placeholder = zeros::<bf16>(device, 1)?;
        let pass = self.cuda_pass_bf(
            device,
            logits,
            side,
            beta.slice(..),
            placeholder.as_view(),
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
        Ok((storage_f32(out, device), Shape::from(())))
    }

    /// The exact backward on CUDA with bf16 activation storage: the logit and
    /// side gradients are bf16, the beta gradient is f32.
    pub(super) fn cuda_bwd_bf(
        &self,
        device: &CudaDevice,
        logits: &Tensor,
        side: &Tensor,
        beta: &Tensor,
        grad: &Tensor,
    ) -> CResult<(Tensor, Tensor, Tensor)> {
        let (rows, vocabulary) = self.check(logits.layout(), side.layout(), beta.layout())?;
        self.cuda_check_bf(vocabulary)?;
        if self.supervise {
            candle_core::bail!("precision=bf16 has no bf16 pointer gate supervision kernel");
        }
        if grad.elem_count() != 1 {
            candle_core::bail!("pointer mixture backward expects a scalar gradient");
        }
        // The op's output is the f32 scalar loss, so its gradient arrives in
        // f32; the kernel's one-element gradient slot is activation storage.
        let grad = if grad.dtype() == DType::F32 {
            grad.to_dtype(DType::BF16)?
        } else {
            grad.clone()
        };
        let (lt, st, bt, gt) = (
            ready_bf(logits)?,
            ready_bf(side)?,
            ready_f32(beta)?,
            ready_bf(&grad)?,
        );
        let (ls, ll) = lt.storage_and_layout();
        let (ss, sl) = st.storage_and_layout();
        let (bs_, bl) = bt.storage_and_layout();
        let (gs, gl) = gt.storage_and_layout();
        let (lv, sv, bv, gv) = (
            bf_view(&ls, ll)?,
            bf_view(&ss, sl)?,
            f32_view(&bs_, bl)?,
            bf_view(&gs, gl)?,
        );
        let pass =
            self.cuda_pass_bf(device, lv.slice(..), sv.slice(..), bv, gv, vocabulary, true)?;
        let dims = self.cuda_dims_bf(rows, vocabulary, true)?;
        launch_bf16(
            device,
            "pointer_side_grad",
            rows * 2 * self.dim,
            &[
                Arg::B(sv),
                Arg::d(&pass.scratch),
                Arg::d(&pass.query_lift),
                Arg::d(&pass.key_lift),
                Arg::d(&pass.query_self),
                Arg::b(&pass.d_side),
                Arg::Dims(dims),
            ],
        )?;
        // d z_v = c share_generate (softmax_v - [v = target]): the
        // cross-entropy gradient kernel with the per-row factor, on bf16
        // logits and a bf16 logit gradient.
        let d_logits = uninit::<bf16>(device, rows * vocabulary)?;
        launch_bf16(
            device,
            "cross_entropy_grad_act",
            rows * vocabulary,
            &[
                Arg::B(lv),
                Arg::u(&pass.targets),
                Arg::d(&pass.lse),
                Arg::d(&pass.scale_z),
                Arg::b(&d_logits),
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
            tensor_bf(d_logits, device, logits.shape()),
            tensor_bf(pass.d_side, device, side.shape()),
            tensor_f32(d_beta, device, beta.shape()),
        ))
    }
}
