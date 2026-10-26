//! Explicit opt-in within-cell writer CUDA handle (`m1-within-cell-writer`, `#3267`,
//! referencing `#3128` and `#3129`). Loads the ahead-of-time `nvcc`-compiled
//! `within-cell-writer.cubin` module and exposes GPU evaluation of versioned
//! `ReadLawV1` / `ReadLawV2` and framing modes (`CurrentSinglePassV1`,
//! `NearestEndpointSinglePassV1`, `TwoPassEndpointOutputV1`, `FrozenConstantFrameV1`,
//! `PermutedFrameV1`, `TrainRemap16V1`) plus the 256-pair snap orbit check.
use super::*;

pub(super) const WITHIN_CELL_WRITER_SOURCE: &str =
    include_str!(concat!(env!("OUT_DIR"), "/within-cell-writer.cu"));
pub(super) const WITHIN_CELL_WRITER_PTX: &str =
    include_str!(concat!(env!("OUT_DIR"), "/within-cell-writer.ptx"));
pub(super) const WITHIN_CELL_WRITER_CUBIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/within-cell-writer.cubin"));
pub(super) const WITHIN_CELL_WRITER_RESOURCE_REPORT: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/within-cell-writer-resources.txt"
));

pub struct WithinCellWriterKernels {
    snap_orbit_check: CudaFunction,
    writer_eval: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapOrbitCheckDeviceResult {
    pub v1_snapped: Vec<u32>,
    pub v2_pos_snapped: Vec<u32>,
    pub v2_neg_snapped: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithinCellDeviceConfig {
    pub read_law: u32,
    pub frame_mode: u32,
    pub transport_mode: u32,
    pub max_rel_bound: u32,
    pub frozen_frame_root: u32,
    pub bypass_refusal_mask: u32,
    pub inject_s13_word: u32,
    pub inject_s14_word: u32,
}

impl WithinCellDeviceConfig {
    pub fn words(&self) -> [u32; 8] {
        [
            self.read_law,
            self.frame_mode,
            self.transport_mode,
            self.max_rel_bound,
            self.frozen_frame_root,
            self.bypass_refusal_mask,
            self.inject_s13_word,
            self.inject_s14_word,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithinCellRowEvaluation {
    pub query: Vec<i32>,
    pub admitted_cells: Vec<u32>,
    pub top8_tokens: [u32; 8],
    pub target_admitted: bool,
    pub top1_correct: bool,
    pub top8_correct: bool,
    pub target_cell_rank: u32,
    pub admitted_token_count: u32,
    pub token_rivals: u32,
    pub within_cell_rivals: u32,
    pub cross_cell_rivals: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithinCellBatchEvaluation {
    pub rows: Vec<WithinCellRowEvaluation>,
    pub trace_words: Vec<u32>,
}

/// Identity of the embedded within-cell writer image, without a CUDA context.
pub fn compiled_within_cell_writer_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(WITHIN_CELL_WRITER_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(WITHIN_CELL_WRITER_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(WITHIN_CELL_WRITER_CUBIN)),
        format!(
            "{:x}",
            Sha256::digest(WITHIN_CELL_WRITER_RESOURCE_REPORT.as_bytes())
        ),
    ]
}

impl Device {
    /// Load the ahead-of-time compiled within-cell writer module for this device's model and stream.
    pub fn within_cell_writer_kernels(&self) -> Result<WithinCellWriterKernels> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(WITHIN_CELL_WRITER_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(WithinCellWriterKernels {
            snap_orbit_check: load(crate::within_cell_writer_source::ENTRIES[0])?,
            writer_eval: load(crate::within_cell_writer_source::ENTRIES[1])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// Run `discrete_within_cell_snap_orbit_check` across all `16 x 16 = 256` root pairs on GPU.
    pub fn run_within_cell_snap_orbit_check(
        &self,
        kernels: &WithinCellWriterKernels,
    ) -> Result<SnapOrbitCheckDeviceResult> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid(
                "within-cell writer handle belongs to another model/stream",
            ));
        }
        let mut out_v1 = self.stream.alloc_zeros::<u32>(256).map_err(cuda)?;
        let mut out_v2_pos = self.stream.alloc_zeros::<u32>(256).map_err(cuda)?;
        let mut out_v2_neg = self.stream.alloc_zeros::<u32>(256).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&kernels.snap_orbit_check);
        launch
            .arg(&self.geometry)
            .arg(&mut out_v1)
            .arg(&mut out_v2_pos)
            .arg(&mut out_v2_neg)
            .arg(&mut status);
        // SAFETY: 256 threads write disjoint indices 0..256 in allocated slices.
        unsafe { launch.launch(launch_config(256)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "within-cell snap orbit check")?;

        Ok(SnapOrbitCheckDeviceResult {
            v1_snapped: self.stream.clone_dtoh(&out_v1).map_err(cuda)?,
            v2_pos_snapped: self.stream.clone_dtoh(&out_v2_pos).map_err(cuda)?,
            v2_neg_snapped: self.stream.clone_dtoh(&out_v2_neg).map_err(cuda)?,
        })
    }

    /// Run `discrete_within_cell_writer_eval` on `batch` and return per-row read vectors,
    /// admitted cells, top-8 ranked tokens, and within-cell vs cross-cell rival counts.
    pub fn evaluate_within_cell_writer_batch(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        wc_cfg: &WithinCellDeviceConfig,
        frame_remap: &[u32; 32],
        capture_trace: bool,
        kernels: &WithinCellWriterKernels,
    ) -> Result<WithinCellBatchEvaluation> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid(
                "within-cell writer handle belongs to another model/stream",
            ));
        }
        let _epoch = self.admit_batch(batch, targets)?;
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let d_targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let d_wc_cfg = self.stream.clone_htod(&wc_cfg.words()).map_err(cuda)?;
        let d_remap = self.stream.clone_htod(frame_remap).map_err(cuda)?;

        let rows = batch.rows as usize;
        let blocks = self.spec().blocks as usize;
        let width = blocks * 8;
        let stages = self.metadata.stages();
        let nodes = self.metadata.nodes_count();
        let trace_words_per_block = FLOCK_TRACE_HEADER_WORDS + stages * FLOCK_TRACE_LEVEL_WORDS;
        let total_trace_words = if capture_trace {
            rows * blocks * (trace_words_per_block + nodes)
        } else {
            0
        };

        let mut out_q = self.stream.alloc_zeros::<i32>(rows * width).map_err(cuda)?;
        let mut out_adm = self.stream.alloc_zeros::<u32>(rows * 16).map_err(cuda)?;
        let mut out_top8 = self.stream.alloc_zeros::<u32>(rows * 8).map_err(cuda)?;
        let mut out_metrics = self.stream.alloc_zeros::<u32>(rows * 8).map_err(cuda)?;
        let mut out_trace = if capture_trace {
            Some(
                self.stream
                    .alloc_zeros::<u32>(total_trace_words.max(1))
                    .map_err(cuda)?,
            )
        } else {
            None
        };
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let null_trace: cudarc::driver::sys::CUdeviceptr = 0;

        let mut launch = self.stream.launch_builder(&kernels.writer_eval);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&d_targets)
            .arg(plan)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&self.position_steps)
            .arg(&d_wc_cfg)
            .arg(&d_remap)
            .arg(&mut out_q)
            .arg(&mut out_adm)
            .arg(&mut out_top8)
            .arg(&mut out_metrics);
        if let Some(ref mut trace_buf) = out_trace {
            launch.arg(trace_buf);
        } else {
            launch.arg(&null_trace);
        }
        launch.arg(&mut status);

        // SAFETY: all device buffers are allocated on this stream with exact row capacities.
        // Launch 1 threadblock per row so all 64 rows in a batch distribute across up to 64 SMs.
        let row_grid = LaunchConfig {
            grid_dim: (batch.rows.max(1), 1, 1),
            block_dim: (1, 1, 1),
            shared_mem_bytes: 0,
        };
        unsafe { launch.launch(row_grid) }.map_err(cuda)?;
        check_status(&self.stream, &status, "within-cell writer evaluation")?;

        let h_q = self.stream.clone_dtoh(&out_q).map_err(cuda)?;
        let h_adm = self.stream.clone_dtoh(&out_adm).map_err(cuda)?;
        let h_top8 = self.stream.clone_dtoh(&out_top8).map_err(cuda)?;
        let h_metrics = self.stream.clone_dtoh(&out_metrics).map_err(cuda)?;
        let trace_words = if let Some(ref trace_buf) = out_trace {
            self.stream.clone_dtoh(trace_buf).map_err(cuda)?
        } else {
            Vec::new()
        };

        let mut row_evals = Vec::with_capacity(rows);
        for r in 0..rows {
            let admitted_cells: Vec<u32> = h_adm[r * 16..(r + 1) * 16]
                .iter()
                .copied()
                .take_while(|&c| c != ABSENT)
                .collect();
            let mut top8_tokens = [ABSENT; 8];
            top8_tokens.copy_from_slice(&h_top8[r * 8..(r + 1) * 8]);
            let m = &h_metrics[r * 8..(r + 1) * 8];
            row_evals.push(WithinCellRowEvaluation {
                query: h_q[r * width..(r + 1) * width].to_vec(),
                admitted_cells,
                top8_tokens,
                target_admitted: m[0] != 0,
                top1_correct: m[1] != 0,
                top8_correct: m[2] != 0,
                target_cell_rank: m[3],
                admitted_token_count: m[4],
                token_rivals: m[5],
                within_cell_rivals: m[6],
                cross_cell_rivals: m[7],
            });
        }
        Ok(WithinCellBatchEvaluation {
            rows: row_evals,
            trace_words,
        })
    }

    /// Run `discrete_within_cell_writer_eval` in `read_only_q` mode (`wc_cfg.bypass_refusal_mask | 32`)
    /// after `admit_batch` has already been called on `batch`, returning the flat `rows * width`
    /// read vector `q` buffer directly from GPU without running the 4,096-token writer loop.
    pub fn evaluate_within_cell_read_vectors_admitted(
        &self,
        batch: &CarriedBatch,
        d_targets: &CudaSlice<u32>,
        wc_cfg: &WithinCellDeviceConfig,
        d_remap: &CudaSlice<u32>,
        kernels: &WithinCellWriterKernels,
    ) -> Result<Vec<i32>> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid(
                "within-cell writer handle belongs to another model/stream",
            ));
        }
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let mut cfg_q = *wc_cfg;
        cfg_q.bypass_refusal_mask |= 32;
        let d_wc_cfg = self.stream.clone_htod(&cfg_q.words()).map_err(cuda)?;

        let rows = batch.rows as usize;
        let blocks = self.spec().blocks as usize;
        let width = blocks * 8;
        let mut out_q = self.stream.alloc_zeros::<i32>(rows * width).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let null_ptr: cudarc::driver::sys::CUdeviceptr = 0;

        let mut launch = self.stream.launch_builder(&kernels.writer_eval);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(d_targets)
            .arg(plan)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&self.position_steps)
            .arg(&d_wc_cfg)
            .arg(d_remap)
            .arg(&mut out_q)
            .arg(&null_ptr)
            .arg(&null_ptr)
            .arg(&null_ptr)
            .arg(&null_ptr)
            .arg(&mut status);

        // SAFETY: `out_q` has capacity `rows * width`, and `wc_cfg[5] & 32` returns before touching
        // `out_admitted_cells`, `out_top8_tokens`, `out_metrics`, or `out_trace`.
        let row_grid = LaunchConfig {
            grid_dim: (batch.rows.max(1), 1, 1),
            block_dim: (1, 1, 1),
            shared_mem_bytes: 0,
        };
        unsafe { launch.launch(row_grid) }.map_err(cuda)?;
        check_status(&self.stream, &status, "within-cell read vector capture")?;
        self.stream.clone_dtoh(&out_q).map_err(cuda)
    }

    /// Admit `batch` with `targets` on this device and return the uploaded `(d_targets, d_remap)`.
    pub fn admit_within_cell_batch(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        frame_remap: &[u32; 32],
    ) -> Result<(CudaSlice<u32>, CudaSlice<u32>)> {
        let _epoch = self.admit_batch(batch, targets)?;
        let d_targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let d_remap = self.stream.clone_htod(frame_remap).map_err(cuda)?;
        Ok((d_targets, d_remap))
    }
}
