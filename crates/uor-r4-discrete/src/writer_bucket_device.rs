//! Opt-in CUDA driver wrapper for the CSR bucketed writer (`m1-faster-writer`, #2840).
//!
//! Loads the ahead-of-time compiled `bucket.cubin` module alongside `Device` and
//! exposes GPU CSR index construction (`discrete_bucket_build`), incremental
//! single-token CSR update (`discrete_bucket_update`), direct writer evaluation
//! with per-row comparison counters (`discrete_bucket_writer_eval` and
//! `discrete_fullscan_writer_eval`), and full model forward/propose/evaluate
//! (`discrete_bucket_forward`, `discrete_bucket_counterfactual`,
//! `discrete_bucket_evaluate`).

use super::{
    admit_cuda_target, aggregate_gpu, check_status, cuda, invalid, launch_config, packed_geometry,
    CarriedBatch, CudaContext, CudaFunction, CudaSlice, CudaStream, Device, Evaluation,
    EvaluationAggregate, EvaluationRow, Forward, LaunchConfig, Parameter, Proposal, Ptx,
    PushKernelArg, FLOCK_TRACE_HEADER_WORDS, FLOCK_TRACE_LEVEL_WORDS,
};
use crate::{objective, ReadSpec, Result};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub(super) const BUCKET_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/bucket.cu"));
pub(super) const BUCKET_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/bucket.ptx"));
pub(super) const BUCKET_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bucket.cubin"));
pub(super) const BUCKET_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/bucket-resources.txt"));

pub const WALK_ID_ORDER: u32 = 0;
pub const WALK_SERVING_PRUNED: u32 = 1;

pub const FAULT_NONE: u32 = 0;
pub const FAULT_STALE_BUCKET: u32 = 1;
pub const FAULT_REVERSED_TIE: u32 = 2;
pub const FAULT_OVER_PRUNED: u32 = 3;
pub const FAULT_CORRUPTED_CSR: u32 = 4;

/// Identity of the embedded CSR bucketed writer source, PTX, cubin, and report.
pub fn compiled_bucket_image_sha256() -> serde_json::Value {
    let hex = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    let mut combined = Sha256::new();
    combined.update(b"uor-r4.bucket-image/1");
    for (name, bytes) in [
        ("target", super::CUDA_IMAGE_TARGET.as_bytes()),
        (
            "bucket_source.rs",
            include_bytes!("bucket_source.rs").as_slice(),
        ),
        (
            "writer_bucket_device.rs",
            include_bytes!("writer_bucket_device.rs").as_slice(),
        ),
        ("bucket.cu", BUCKET_SOURCE.as_bytes()),
        ("bucket.ptx", BUCKET_PTX.as_bytes()),
        ("bucket.cubin", BUCKET_CUBIN),
        ("bucket-resources.txt", BUCKET_RESOURCE_REPORT.as_bytes()),
    ] {
        combined.update((name.len() as u64).to_le_bytes());
        combined.update(name.as_bytes());
        combined.update((bytes.len() as u64).to_le_bytes());
        combined.update(bytes);
    }
    serde_json::json!({
        "schema": "uor-r4.bucket-image/1",
        "format": "cubin",
        "target": super::CUDA_IMAGE_TARGET,
        "image_sha256": format!("{:x}", combined.finalize()),
        "source_sha256": hex(BUCKET_SOURCE.as_bytes()),
        "ptx_sha256": hex(BUCKET_PTX.as_bytes()),
        "cubin_sha256": hex(BUCKET_CUBIN),
        "report_sha256": hex(BUCKET_RESOURCE_REPORT.as_bytes()),
    })
}

/// Verifies that the generated `bucket.cu` starts with `discrete.cu`'s helper and
/// writer prefix and defines the seven admitted `.entry` kernels.
pub fn bucket_extends_ordinary_source() -> Result<()> {
    let ordinary = include_str!("discrete.cu");
    let expected = crate::bucket_source::generate(ordinary).map_err(invalid)?;
    if BUCKET_SOURCE != expected {
        return Err(invalid(
            "embedded bucket.cu differs from bucket_source::generate(discrete.cu)",
        ));
    }
    Ok(())
}

/// Host snapshot of the GPU-resident CSR bucket index (`offsets`, `members`, `cells`, `absent_count`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BucketIndexSnapshot {
    pub offsets: Vec<u32>,
    pub members: Vec<u32>,
    pub cells: Vec<u32>,
    pub absent_count: u32,
}

impl BucketIndexSnapshot {
    /// Checks CSR structural invariants: `offsets.len() == 257`, `offsets[0] == 0`,
    /// monotonic `offsets`, `offsets[256] + absent_count == vocabulary`, and every
    /// member in bucket `c` has `cells[member] == c` and strictly ascending token IDs.
    pub fn validate(&self, vocabulary: u32) -> Result<()> {
        if self.offsets.len() != 257 || self.cells.len() != vocabulary as usize {
            return Err(invalid("CSR index offsets/cells length mismatch"));
        }
        if self.offsets[0] != 0 {
            return Err(invalid("CSR offsets[0] must be 0"));
        }
        for c in 0..256usize {
            if self.offsets[c] > self.offsets[c + 1] {
                return Err(invalid("CSR offsets are not monotonic"));
            }
            let start = self.offsets[c] as usize;
            let end = self.offsets[c + 1] as usize;
            if end > self.members.len() {
                return Err(invalid("CSR offset exceeds members buffer"));
            }
            let slice = &self.members[start..end];
            for (idx, &token) in slice.iter().enumerate() {
                if token >= vocabulary {
                    return Err(invalid("CSR member token out of range"));
                }
                if self.cells[token as usize] != c as u32 {
                    return Err(invalid("CSR member cell disagrees with cells[token]"));
                }
                if idx > 0 && slice[idx - 1] >= token {
                    return Err(invalid("CSR bucket members are not strictly ascending"));
                }
            }
        }
        if self.offsets[256]
            .checked_add(self.absent_count)
            .ok_or_else(|| invalid("CSR total count overflow"))?
            != vocabulary
        {
            return Err(invalid("CSR offsets[256] + absent_count != vocabulary"));
        }
        Ok(())
    }
}

/// GPU-resident CSR bucket index buffers.
pub struct BucketIndexDevice {
    offsets: CudaSlice<u32>,
    members: CudaSlice<u32>,
    cells: CudaSlice<u32>,
    absent_count: CudaSlice<u32>,
    vocabulary: u32,
}

impl BucketIndexDevice {
    pub fn snapshot(&self, stream: &Arc<CudaStream>) -> Result<BucketIndexSnapshot> {
        let offsets = stream.clone_dtoh(&self.offsets).map_err(cuda)?;
        let members = stream.clone_dtoh(&self.members).map_err(cuda)?;
        let cells = stream.clone_dtoh(&self.cells).map_err(cuda)?;
        let absent = stream.clone_dtoh(&self.absent_count).map_err(cuda)?;
        let absent_count = *absent
            .first()
            .ok_or_else(|| invalid("missing absent_count word"))?;
        Ok(BucketIndexSnapshot {
            offsets,
            members,
            cells,
            absent_count,
        })
    }
}

/// Per-row comparison counts emitted by `discrete_bucket_writer_eval` and `discrete_fullscan_writer_eval`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CounterRow {
    pub token_cell_calls: u32,
    pub cell_distance_evals: u32,
    pub token_distance_evals: u32,
    pub max_bucket_size: u32,
}

impl CounterRow {
    pub fn total_evals(&self) -> u64 {
        u64::from(self.token_cell_calls)
            + u64::from(self.cell_distance_evals)
            + u64::from(self.token_distance_evals)
    }
}

/// Complete output of `discrete_bucket_writer_eval` or `discrete_fullscan_writer_eval`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DirectWriterOutput {
    pub status: u32,
    pub predictions: Vec<u32>,
    pub losses: Vec<u64>,
    pub token_rivals: Vec<u32>,
    pub address_rivals: Vec<u32>,
    pub admitted: Vec<u32>,
    pub top8: Vec<u32>,
    pub metrics: Vec<u64>,
    pub counters: Vec<CounterRow>,
}

/// Timed output of `discrete_fullscan_writer_eval` and both walks of `discrete_bucket_writer_eval`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DirectTripleTimed {
    pub full: DirectWriterOutput,
    pub id_order: DirectWriterOutput,
    pub serving_pruned: DirectWriterOutput,
    pub csr: BucketIndexSnapshot,
    pub build_us: u64,
    pub fullscan_us: u64,
    pub id_order_us: u64,
    pub serving_pruned_us: u64,
}

/// Loaded `bucket.cubin` kernel handles bound to a CUDA context/stream.
pub struct BucketWriterEngine {
    context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    geometry: CudaSlice<u32>,
    build: CudaFunction,
    update: CudaFunction,
    bucket_writer_eval: CudaFunction,
    fullscan_writer_eval: CudaFunction,
    forward: CudaFunction,
    counterfactual: CudaFunction,
    evaluate: CudaFunction,
}

impl BucketWriterEngine {
    /// Load `bucket.cubin` onto an existing `Device`'s CUDA context and stream.
    pub fn from_device(model: &Device) -> Result<Self> {
        Self::from_context(Arc::clone(&model.context), Arc::clone(&model.stream))
    }

    /// Load `bucket.cubin` onto device `ordinal`.
    pub fn new(ordinal: usize) -> Result<Self> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        admit_cuda_target(&context)?;
        let stream = context.default_stream();
        Self::from_context(context, stream)
    }

    fn from_context(context: Arc<CudaContext>, stream: Arc<CudaStream>) -> Result<Self> {
        admit_cuda_target(&context)?;
        bucket_extends_ordinary_source()?;
        let module = context
            .load_module(Ptx::from_binary(BUCKET_CUBIN.to_vec()))
            .map_err(cuda)?;
        let packed = packed_geometry();
        let geometry = stream.clone_htod(&packed).map_err(cuda)?;
        Ok(Self {
            context,
            stream,
            geometry,
            build: module
                .load_function("discrete_bucket_build")
                .map_err(cuda)?,
            update: module
                .load_function("discrete_bucket_update")
                .map_err(cuda)?,
            bucket_writer_eval: module
                .load_function("discrete_bucket_writer_eval")
                .map_err(cuda)?,
            fullscan_writer_eval: module
                .load_function("discrete_fullscan_writer_eval")
                .map_err(cuda)?,
            forward: module
                .load_function("discrete_bucket_forward")
                .map_err(cuda)?,
            counterfactual: module
                .load_function("discrete_bucket_counterfactual")
                .map_err(cuda)?,
            evaluate: module
                .load_function("discrete_bucket_evaluate")
                .map_err(cuda)?,
        })
    }

    pub fn stream(&self) -> &Arc<CudaStream> {
        &self.stream
    }

    pub fn synchronize(&self) -> Result<()> {
        self.context.synchronize().map_err(cuda)
    }

    /// Build the 256-cell CSR index on the GPU for `model`'s current codebook table.
    pub fn build_index(&self, model: &Device) -> Result<BucketIndexDevice> {
        let vocabulary = model.spec().vocabulary;
        let spec = model.gpu_spec(1)?;
        self.build_index_raw(&model.table, &spec, vocabulary)
    }

    fn build_index_raw(
        &self,
        table: &CudaSlice<i32>,
        spec: &CudaSlice<u32>,
        vocabulary: u32,
    ) -> Result<BucketIndexDevice> {
        let mut offsets = self.stream.alloc_zeros::<u32>(257).map_err(cuda)?;
        let mut members = self
            .stream
            .alloc_zeros::<u32>(vocabulary as usize)
            .map_err(cuda)?;
        let mut cells = self
            .stream
            .alloc_zeros::<u32>(vocabulary as usize)
            .map_err(cuda)?;
        let mut absent_count = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.build);
        launch
            .arg(table)
            .arg(&self.geometry)
            .arg(spec)
            .arg(&mut offsets)
            .arg(&mut members)
            .arg(&mut cells)
            .arg(&mut absent_count)
            .arg(&mut status);
        let config = LaunchConfig {
            grid_dim: (1, 1, 1),
            block_dim: (256, 1, 1),
            shared_mem_bytes: 0,
        };
        unsafe { launch.launch(config) }.map_err(cuda)?;
        check_status(&self.stream, &status, "bucket build")?;
        Ok(BucketIndexDevice {
            offsets,
            members,
            cells,
            absent_count,
            vocabulary,
        })
    }

    /// Incrementally update the GPU-resident CSR index after `moved_token` changes in `model.table`.
    /// Returns `entries_written` reported by `discrete_bucket_update`.
    pub fn update_index(
        &self,
        model: &Device,
        index: &mut BucketIndexDevice,
        moved_token: u32,
    ) -> Result<u32> {
        if index.vocabulary != model.spec().vocabulary {
            return Err(invalid("bucket index vocabulary differs from model"));
        }
        let spec = model.gpu_spec(1)?;
        let mut entries_written = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.update);
        launch
            .arg(&model.table)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&moved_token)
            .arg(&mut index.offsets)
            .arg(&mut index.members)
            .arg(&mut index.cells)
            .arg(&mut index.absent_count)
            .arg(&mut entries_written)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "bucket update")?;
        let words = self.stream.clone_dtoh(&entries_written).map_err(cuda)?;
        Ok(words[0])
    }

    /// Run `discrete_bucket_forward` on `model` and `batch`.
    pub fn forward(
        &self,
        model: &Device,
        index: &BucketIndexDevice,
        batch: &CarriedBatch,
        targets: &[u32],
    ) -> Result<Forward> {
        let epoch = model.admit_batch(batch, targets)?;
        let target = model.stream.clone_htod(targets).map_err(cuda)?;
        let spec = model.gpu_spec(batch.rows)?;
        let mut out = model
            .stream
            .alloc_zeros::<i32>(batch.rows as usize * model.spec().blocks as usize * 8)
            .map_err(cuda)?;
        let mut predictions = model
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut losses = model
            .stream
            .alloc_zeros::<u64>(batch.rows as usize)
            .map_err(cuda)?;
        let mut token_rivals = model
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut address_rivals = model
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut admitted_cells = model
            .stream
            .alloc_zeros::<u32>(batch.rows as usize * 16)
            .map_err(cuda)?;
        let null_trace: cudarc::driver::sys::CUdeviceptr = 0;
        let mut status = model.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = model.stream.launch_builder(&self.forward);
        launch
            .arg(&model.table)
            .arg(&model.routes)
            .arg(&model.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&target)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&model.position_steps)
            .arg(&index.offsets)
            .arg(&index.members)
            .arg(&index.cells)
            .arg(&index.absent_count)
            .arg(&mut out)
            .arg(&mut predictions)
            .arg(&mut losses)
            .arg(&mut token_rivals)
            .arg(&mut address_rivals)
            .arg(&mut admitted_cells)
            .arg(&null_trace)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
        check_status(&model.stream, &status, "bucket forward")?;
        Ok(Forward {
            predictions: model.stream.clone_dtoh(&predictions).map_err(cuda)?,
            losses: model.stream.clone_dtoh(&losses).map_err(cuda)?,
            token_rivals: model.stream.clone_dtoh(&token_rivals).map_err(cuda)?,
            address_rivals: model.stream.clone_dtoh(&address_rivals).map_err(cuda)?,
            admitted_cells: model.stream.clone_dtoh(&admitted_cells).map_err(cuda)?,
            vectors: out,
            trace: None,
            trace_stride: None,
            flock_trace_words: None,
            binding: model.binding.clone(),
            epoch,
            stream: Arc::clone(&model.stream),
        })
    }

    /// Run `discrete_bucket_counterfactual` + `discrete_choose` on `model` and `batch`.
    pub fn propose(
        &self,
        model: &Device,
        index: &BucketIndexDevice,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
    ) -> Result<Proposal> {
        let (kind, param_index) = model.parameter(parameter)?;
        let epoch = model.admit_batch(batch, targets)?;
        let targets = model.stream.clone_htod(targets).map_err(cuda)?;
        let spec = model.gpu_spec(batch.rows)?;
        let mut losses = model.stream.alloc_zeros::<u64>(16).map_err(cuda)?;
        let mut status = model.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = model.stream.launch_builder(&self.counterfactual);
        launch
            .arg(&model.table)
            .arg(&model.routes)
            .arg(&model.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&model.position_steps)
            .arg(&index.offsets)
            .arg(&index.members)
            .arg(&index.cells)
            .arg(&index.absent_count)
            .arg(&kind)
            .arg(&param_index)
            .arg(&mut losses)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(16)) }.map_err(cuda)?;
        check_status(&model.stream, &status, "bucket counterfactual")?;
        let mut proposal = model.stream.alloc_zeros::<u64>(5).map_err(cuda)?;
        let mut choose = model.stream.launch_builder(&model.kernels.choose);
        choose
            .arg(&losses)
            .arg(&model.epoch)
            .arg(&model.table)
            .arg(&model.routes)
            .arg(&model.gates)
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&model.position_steps)
            .arg(&kind)
            .arg(&param_index)
            .arg(&mut proposal);
        unsafe { choose.launch(launch_config(1)) }.map_err(cuda)?;
        model.stream.synchronize().map_err(cuda)?;
        Ok(Proposal {
            data: proposal,
            losses,
            parameter,
            binding: model.binding.clone(),
            epoch,
            stream: Arc::clone(&model.stream),
        })
    }

    /// Run `discrete_bucket_evaluate` + `discrete_evaluation_aggregate` on `model` and `batch`.
    pub fn evaluate(
        &self,
        model: &Device,
        index: &BucketIndexDevice,
        batch: &CarriedBatch,
        targets: &[u32],
    ) -> Result<Evaluation> {
        let epoch = model.admit_batch(batch, targets)?;
        let targets_gpu = model.stream.clone_htod(targets).map_err(cuda)?;
        let spec = model.gpu_spec(batch.rows)?;
        let mut top8 = model
            .stream
            .alloc_zeros::<u32>(batch.rows as usize * 8)
            .map_err(cuda)?;
        let mut metrics = model
            .stream
            .alloc_zeros::<u64>(batch.rows as usize * objective::ROW_WORDS)
            .map_err(cuda)?;
        let mut status = model.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = model.stream.launch_builder(&self.evaluate);
        launch
            .arg(&model.table)
            .arg(&model.routes)
            .arg(&model.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets_gpu)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&model.position_steps)
            .arg(&index.offsets)
            .arg(&index.members)
            .arg(&index.cells)
            .arg(&index.absent_count)
            .arg(&mut top8)
            .arg(&mut metrics)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
        check_status(&model.stream, &status, "bucket evaluation")?;
        let summary_words = aggregate_gpu(
            &model.stream,
            &model.kernels.evaluation_aggregate,
            &metrics,
            batch.rows,
            model.spec().blocks * 8,
            false,
        )?;
        let aggregate = EvaluationAggregate::from_words(&summary_words)?;
        let top8_vec = model.stream.clone_dtoh(&top8).map_err(cuda)?;
        let metrics_vec = model.stream.clone_dtoh(&metrics).map_err(cuda)?;
        let rows = top8_vec
            .chunks_exact(8)
            .zip(metrics_vec.chunks_exact(objective::ROW_WORDS))
            .map(|(top, p)| {
                let row_top8 = top
                    .try_into()
                    .map_err(|_| invalid("top8 row must contain eight IDs"))?;
                Ok(EvaluationRow {
                    target_position: None,
                    target_token: None,
                    top8: row_top8,
                    token_hinge: p[0],
                    address_hinge: p[1],
                    cell_rank_hinge: p[8],
                    target_cell_rank: p[9],
                    occupied_cells: p[10],
                    admitted_token_count: p[11],
                    loss_ta_numerator: p[2],
                    loss_tr_numerator: p[12],
                    objective_loss_numerator: p[13],
                    total_loss_numerator: p[2],
                    token_rivals: p[3],
                    address_rivals: p[4],
                    top1_correct: p[5],
                    top8_correct: p[6],
                    admitted_cells: p[7],
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Evaluation {
            schema: objective::EVALUATION_SCHEMA,
            objective_policy: model.objective_policy(),
            aggregate,
            rows,
            summary_words,
            binding: model.binding.clone(),
            epoch,
        })
    }

    /// Finite qualification helper: commits `proposal` on `model` via `discrete_apply`.
    pub fn commit_proposal_in_qualification(
        &self,
        model: &mut Device,
        proposal: &Proposal,
    ) -> Result<bool> {
        let (kind, index) = model.parameter(proposal.parameter)?;
        if proposal.binding != model.binding {
            return Err(invalid("proposal belongs to another model/source/geometry"));
        }
        let before = model.epoch()?;
        if proposal.epoch != before || proposal.observation()?.epoch != before {
            return Err(invalid("proposal is stale at the current model epoch"));
        }
        let spec = model.gpu_spec(1)?;
        let mut status = model.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = model.stream.launch_builder(&model.kernels.apply);
        launch
            .arg(&mut model.table)
            .arg(&mut model.routes)
            .arg(&mut model.gates)
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&model.position_steps)
            .arg(&proposal.data)
            .arg(&kind)
            .arg(&index)
            .arg(&mut model.epoch)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        check_status(&model.stream, &status, "bucket qualification commit")?;
        Ok(model.epoch()? != before)
    }

    /// Evaluate both `discrete_fullscan_writer_eval` and `discrete_bucket_writer_eval`
    /// on the GPU over arbitrary `vocabulary` (`V` up to `16,384` or higher), `blocks`,
    /// `kept_cells`, `table`, and pre-supplied query vectors `queries` (`rows * blocks * 8`).
    /// When `stale_index_table` is `Some(pre_table)`, the CSR index is built from
    /// `pre_table` while both writers score `table` (used for Planted Fault 1).
    #[allow(clippy::too_many_arguments)]
    pub fn eval_direct_pair(
        &self,
        vocabulary: u32,
        blocks: u32,
        kept_cells: u32,
        objective_word: u32,
        table: &[i32],
        queries: &[i32],
        targets: &[u32],
        kind: u32,
        index: u32,
        candidate: u32,
        walk_mode: u32,
        fault_mode: u32,
        stale_index_table: Option<&[i32]>,
    ) -> Result<(DirectWriterOutput, DirectWriterOutput, BucketIndexSnapshot)> {
        let width = (blocks as usize) * 8;
        let rows = targets.len() as u32;
        if rows == 0
            || table.len() != (vocabulary as usize) * width
            || queries.len() != (rows as usize) * width
        {
            return Err(invalid("direct writer eval input dimensions mismatch"));
        }
        let spec_words = [
            vocabulary,
            blocks,
            1,
            kept_cells,
            rows,
            1,
            1,
            1,
            0,
            0,
            0,
            objective_word,
            0,
            0,
            0,
            0,
        ];
        let spec_gpu = self.stream.clone_htod(&spec_words).map_err(cuda)?;
        let index_source = stale_index_table.unwrap_or(table);
        let index_table_gpu = self.stream.clone_htod(index_source).map_err(cuda)?;
        let csr = self.build_index_raw(&index_table_gpu, &spec_gpu, vocabulary)?;
        let csr_snapshot = csr.snapshot(&self.stream)?;
        let table_gpu = self.stream.clone_htod(table).map_err(cuda)?;
        let q_gpu = self.stream.clone_htod(queries).map_err(cuda)?;
        let targets_gpu = self.stream.clone_htod(targets).map_err(cuda)?;

        let full = self.run_fullscan_eval(
            &q_gpu,
            &table_gpu,
            &spec_gpu,
            &targets_gpu,
            rows,
            kind,
            index,
            candidate,
        )?;
        let bucket = self.run_bucket_eval(
            &q_gpu,
            &table_gpu,
            &spec_gpu,
            &csr,
            &targets_gpu,
            rows,
            kind,
            index,
            candidate,
            walk_mode,
            fault_mode,
        )?;
        let _ = (
            FLOCK_TRACE_HEADER_WORDS,
            FLOCK_TRACE_LEVEL_WORDS,
            ReadSpec::Route,
        );
        Ok((full, bucket, csr_snapshot))
    }

    /// Upload `table`, `queries`, and `targets` once, build and snapshot the CSR index on GPU,
    /// and time `discrete_fullscan_writer_eval`, `discrete_bucket_writer_eval` (`WALK_ID_ORDER`),
    /// and `discrete_bucket_writer_eval` (`WALK_SERVING_PRUNED`) with stream synchronization.
    #[allow(clippy::too_many_arguments)]
    pub fn eval_direct_triple_timed(
        &self,
        vocabulary: u32,
        blocks: u32,
        kept_cells: u32,
        objective_word: u32,
        table: &[i32],
        queries: &[i32],
        targets: &[u32],
        kind: u32,
        index: u32,
        candidate: u32,
    ) -> Result<DirectTripleTimed> {
        let width = (blocks as usize) * 8;
        let rows = targets.len() as u32;
        if rows == 0
            || table.len() != (vocabulary as usize) * width
            || queries.len() != (rows as usize) * width
        {
            return Err(invalid("direct triple timed input dimensions mismatch"));
        }
        let spec_words = [
            vocabulary,
            blocks,
            1,
            kept_cells,
            rows,
            1,
            1,
            1,
            0,
            0,
            0,
            objective_word,
            0,
            0,
            0,
            0,
        ];
        let spec_gpu = self.stream.clone_htod(&spec_words).map_err(cuda)?;
        let table_gpu = self.stream.clone_htod(table).map_err(cuda)?;
        let q_gpu = self.stream.clone_htod(queries).map_err(cuda)?;
        let targets_gpu = self.stream.clone_htod(targets).map_err(cuda)?;
        self.synchronize()?;

        let t_build = std::time::Instant::now();
        let csr = self.build_index_raw(&table_gpu, &spec_gpu, vocabulary)?;
        self.synchronize()?;
        let build_us = t_build.elapsed().as_micros() as u64;
        let csr_snapshot = csr.snapshot(&self.stream)?;

        let t_full = std::time::Instant::now();
        let full = self.run_fullscan_eval(
            &q_gpu,
            &table_gpu,
            &spec_gpu,
            &targets_gpu,
            rows,
            kind,
            index,
            candidate,
        )?;
        self.synchronize()?;
        let fullscan_us = t_full.elapsed().as_micros() as u64;

        let t_id = std::time::Instant::now();
        let id_order = self.run_bucket_eval(
            &q_gpu,
            &table_gpu,
            &spec_gpu,
            &csr,
            &targets_gpu,
            rows,
            kind,
            index,
            candidate,
            WALK_ID_ORDER,
            FAULT_NONE,
        )?;
        self.synchronize()?;
        let id_order_us = t_id.elapsed().as_micros() as u64;

        let t_pruned = std::time::Instant::now();
        let serving_pruned = self.run_bucket_eval(
            &q_gpu,
            &table_gpu,
            &spec_gpu,
            &csr,
            &targets_gpu,
            rows,
            kind,
            index,
            candidate,
            WALK_SERVING_PRUNED,
            FAULT_NONE,
        )?;
        self.synchronize()?;
        let serving_pruned_us = t_pruned.elapsed().as_micros() as u64;

        Ok(DirectTripleTimed {
            full,
            id_order,
            serving_pruned,
            csr: csr_snapshot,
            build_us,
            fullscan_us,
            id_order_us,
            serving_pruned_us,
        })
    }

    /// Evaluate `discrete_fullscan_writer_eval` and both walks (`WALK_ID_ORDER`, `WALK_SERVING_PRUNED`)
    /// of `discrete_bucket_writer_eval` on `model`'s GPU-resident table and `csr` index over
    /// pre-computed query vectors `queries` (`rows * blocks * 8`) and `targets` (`rows`).
    pub fn eval_model_queries(
        &self,
        model: &Device,
        csr: &BucketIndexDevice,
        queries: &[i32],
        targets: &[u32],
    ) -> Result<DirectTripleTimed> {
        let rows = targets.len() as u32;
        let width = (model.spec().blocks as usize) * 8;
        if rows == 0 || queries.len() != (rows as usize) * width {
            return Err(invalid("eval_model_queries input dimensions mismatch"));
        }
        let spec_gpu = model.gpu_spec(rows)?;
        let q_gpu = self.stream.clone_htod(queries).map_err(cuda)?;
        let targets_gpu = self.stream.clone_htod(targets).map_err(cuda)?;
        let csr_snapshot = csr.snapshot(&self.stream)?;
        self.synchronize()?;

        let t_full = std::time::Instant::now();
        let full =
            self.run_fullscan_eval(&q_gpu, &model.table, &spec_gpu, &targets_gpu, rows, 0, 0, 0)?;
        self.synchronize()?;
        let fullscan_us = t_full.elapsed().as_micros() as u64;

        let t_id = std::time::Instant::now();
        let id_order = self.run_bucket_eval(
            &q_gpu,
            &model.table,
            &spec_gpu,
            csr,
            &targets_gpu,
            rows,
            0,
            0,
            0,
            WALK_ID_ORDER,
            FAULT_NONE,
        )?;
        self.synchronize()?;
        let id_order_us = t_id.elapsed().as_micros() as u64;

        let t_pruned = std::time::Instant::now();
        let serving_pruned = self.run_bucket_eval(
            &q_gpu,
            &model.table,
            &spec_gpu,
            csr,
            &targets_gpu,
            rows,
            0,
            0,
            0,
            WALK_SERVING_PRUNED,
            FAULT_NONE,
        )?;
        self.synchronize()?;
        let serving_pruned_us = t_pruned.elapsed().as_micros() as u64;

        Ok(DirectTripleTimed {
            full,
            id_order,
            serving_pruned,
            csr: csr_snapshot,
            build_us: 0,
            fullscan_us,
            id_order_us,
            serving_pruned_us,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_fullscan_eval(
        &self,
        q_gpu: &CudaSlice<i32>,
        table_gpu: &CudaSlice<i32>,
        spec_gpu: &CudaSlice<u32>,
        targets_gpu: &CudaSlice<u32>,
        rows: u32,
        kind: u32,
        index: u32,
        candidate: u32,
    ) -> Result<DirectWriterOutput> {
        let n = rows as usize;
        let mut predictions = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut losses = self.stream.alloc_zeros::<u64>(n).map_err(cuda)?;
        let mut token_rivals = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut address_rivals = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut admitted = self.stream.alloc_zeros::<u32>(n * 16).map_err(cuda)?;
        let mut top8 = self.stream.alloc_zeros::<u32>(n * 8).map_err(cuda)?;
        let mut metrics = self
            .stream
            .alloc_zeros::<u64>(n * objective::ROW_WORDS)
            .map_err(cuda)?;
        let mut counters = self.stream.alloc_zeros::<u32>(n * 4).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.fullscan_writer_eval);
        launch
            .arg(q_gpu)
            .arg(table_gpu)
            .arg(&self.geometry)
            .arg(spec_gpu)
            .arg(targets_gpu)
            .arg(&kind)
            .arg(&index)
            .arg(&candidate)
            .arg(&mut predictions)
            .arg(&mut losses)
            .arg(&mut token_rivals)
            .arg(&mut address_rivals)
            .arg(&mut admitted)
            .arg(&mut top8)
            .arg(&mut metrics)
            .arg(&mut counters)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(rows)) }.map_err(cuda)?;
        let status_words = self.stream.clone_dtoh(&status).map_err(cuda)?;
        let raw_counters = self.stream.clone_dtoh(&counters).map_err(cuda)?;
        let parsed_counters = raw_counters
            .chunks_exact(4)
            .map(|c| CounterRow {
                token_cell_calls: c[0],
                cell_distance_evals: c[1],
                token_distance_evals: c[2],
                max_bucket_size: c[3],
            })
            .collect();
        Ok(DirectWriterOutput {
            status: status_words[0],
            predictions: self.stream.clone_dtoh(&predictions).map_err(cuda)?,
            losses: self.stream.clone_dtoh(&losses).map_err(cuda)?,
            token_rivals: self.stream.clone_dtoh(&token_rivals).map_err(cuda)?,
            address_rivals: self.stream.clone_dtoh(&address_rivals).map_err(cuda)?,
            admitted: self.stream.clone_dtoh(&admitted).map_err(cuda)?,
            top8: self.stream.clone_dtoh(&top8).map_err(cuda)?,
            metrics: self.stream.clone_dtoh(&metrics).map_err(cuda)?,
            counters: parsed_counters,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_bucket_eval(
        &self,
        q_gpu: &CudaSlice<i32>,
        table_gpu: &CudaSlice<i32>,
        spec_gpu: &CudaSlice<u32>,
        csr: &BucketIndexDevice,
        targets_gpu: &CudaSlice<u32>,
        rows: u32,
        kind: u32,
        index: u32,
        candidate: u32,
        walk_mode: u32,
        fault_mode: u32,
    ) -> Result<DirectWriterOutput> {
        let n = rows as usize;
        let mut predictions = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut losses = self.stream.alloc_zeros::<u64>(n).map_err(cuda)?;
        let mut token_rivals = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut address_rivals = self.stream.alloc_zeros::<u32>(n).map_err(cuda)?;
        let mut admitted = self.stream.alloc_zeros::<u32>(n * 16).map_err(cuda)?;
        let mut top8 = self.stream.alloc_zeros::<u32>(n * 8).map_err(cuda)?;
        let mut metrics = self
            .stream
            .alloc_zeros::<u64>(n * objective::ROW_WORDS)
            .map_err(cuda)?;
        let mut counters = self.stream.alloc_zeros::<u32>(n * 4).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.bucket_writer_eval);
        launch
            .arg(q_gpu)
            .arg(table_gpu)
            .arg(&self.geometry)
            .arg(spec_gpu)
            .arg(&csr.offsets)
            .arg(&csr.members)
            .arg(&csr.cells)
            .arg(&csr.absent_count)
            .arg(targets_gpu)
            .arg(&kind)
            .arg(&index)
            .arg(&candidate)
            .arg(&walk_mode)
            .arg(&fault_mode)
            .arg(&mut predictions)
            .arg(&mut losses)
            .arg(&mut token_rivals)
            .arg(&mut address_rivals)
            .arg(&mut admitted)
            .arg(&mut top8)
            .arg(&mut metrics)
            .arg(&mut counters)
            .arg(&mut status);
        unsafe { launch.launch(launch_config(rows)) }.map_err(cuda)?;
        let status_words = self.stream.clone_dtoh(&status).map_err(cuda)?;
        let raw_counters = self.stream.clone_dtoh(&counters).map_err(cuda)?;
        let parsed_counters = raw_counters
            .chunks_exact(4)
            .map(|c| CounterRow {
                token_cell_calls: c[0],
                cell_distance_evals: c[1],
                token_distance_evals: c[2],
                max_bucket_size: c[3],
            })
            .collect();
        Ok(DirectWriterOutput {
            status: status_words[0],
            predictions: self.stream.clone_dtoh(&predictions).map_err(cuda)?,
            losses: self.stream.clone_dtoh(&losses).map_err(cuda)?,
            token_rivals: self.stream.clone_dtoh(&token_rivals).map_err(cuda)?,
            address_rivals: self.stream.clone_dtoh(&address_rivals).map_err(cuda)?,
            admitted: self.stream.clone_dtoh(&admitted).map_err(cuda)?,
            top8: self.stream.clone_dtoh(&top8).map_err(cuda)?,
            metrics: self.stream.clone_dtoh(&metrics).map_err(cuda)?,
            counters: parsed_counters,
        })
    }
}
