//! Explicit opt-in BVG-K block proposal and one-epoch set commit (Stage C2,
//! handoff H2b). Nothing selects it implicitly; no driver path calls it. The
//! device scores the block (rows-K), chooses `S_t`, scores `L(S_t)` on a scratch
//! clone when `|S_t| >= 2`, decides and, on `apply_block`, commits the decided
//! set in one epoch. The host only admits, reads back and cross-checks.
use super::rowsk_device::MAX_COORDINATE_BLOCK;
use super::*;
use crate::bvgk_block::{self, BlockCommit, BlockRecord, RECORD_WORDS};

pub(super) const BVGK_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/bvgk.cu"));
pub(super) const BVGK_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/bvgk.ptx"));
pub(super) const BVGK_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bvgk.cubin"));
pub(super) const BVGK_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/bvgk-resources.txt"));

/// The three BVG-K entries, bound to one model and stream.
pub struct BvgkKernels {
    choose: CudaFunction,
    decide: CudaFunction,
    commit: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

/// One batch's decided block, held on the device until `apply_block`.
pub struct BlockProposal {
    records: CudaSlice<u64>,
    commit: CudaSlice<u32>,
    commit_count: u64,
    kind: u32,
    k: u32,
    rows: u32,
    record: BlockRecord,
    binding: Binding,
    epoch: u64,
    stream: Arc<CudaStream>,
    commit_fn: CudaFunction,
}

impl BlockProposal {
    pub fn record(&self) -> &BlockRecord {
        &self.record
    }
}

impl Device {
    /// Load the opt-in BVG-K module for this device's model and stream.
    pub fn bvgk_kernels(&self) -> Result<BvgkKernels> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(BVGK_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(BvgkKernels {
            choose: load(crate::bvgk_source::ENTRIES[5])?,
            decide: load(crate::bvgk_source::ENTRIES[6])?,
            commit: load(crate::bvgk_source::ENTRIES[7])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// A scratch copy of the learned state on the same context, stream and
    /// binding, so the joint forward cannot touch the original.
    fn scratch_clone(&self) -> Result<Device> {
        let copy_u32 = |s: &CudaSlice<u32>| self.stream.clone_dtod(s).map_err(cuda);
        let copy_i32 = |s: &CudaSlice<i32>| self.stream.clone_dtod(s).map_err(cuda);
        let k = &self.kernels;
        Ok(Device {
            context: Arc::clone(&self.context),
            stream: Arc::clone(&self.stream),
            table: copy_i32(&self.table)?,
            routes: copy_u32(&self.routes)?,
            gates: copy_i32(&self.gates)?,
            epoch: self.stream.clone_dtod(&self.epoch).map_err(cuda)?,
            geometry: copy_u32(&self.geometry)?,
            metadata: self.metadata.clone(),
            kernels: Kernels {
                init: k.init.clone(),
                encode: k.encode.clone(),
                export_token_cells: k.export_token_cells.clone(),
                forward: k.forward.clone(),
                counterfactual: k.counterfactual.clone(),
                choose: k.choose.clone(),
                apply: k.apply.clone(),
                validate: k.validate.clone(),
                evaluate: k.evaluate.clone(),
                evaluation_aggregate: k.evaluation_aggregate.clone(),
            },
            binding: self.binding.clone(),
            rows_cache: std::sync::OnceLock::new(),
        })
    }

    fn check_bvgk(&self, kernels: &BvgkKernels) -> Result<()> {
        if kernels.binding != self.binding || !Arc::ptr_eq(&kernels.stream, &self.stream) {
            return Err(invalid("BVG-K handle belongs to another model/stream"));
        }
        Ok(())
    }

    /// Score, choose and decide one block on the pre-commit state. When
    /// `|S_t| >= 2`, `L(S_t)` is one forward on a scratch clone with `S_t`
    /// committed and the same windows re-carried. The original state and epoch
    /// are unchanged; `apply_block` commits the decided set.
    pub fn propose_block(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        block: &[Parameter],
        eligible: &[bool],
        rowsk: &RowsKScoring,
        bvgk: &BvgkKernels,
    ) -> Result<BlockProposal> {
        self.check_bvgk(bvgk)?;
        bvgk_block::admit_block(block, eligible, MAX_COORDINATE_BLOCK)?;
        let scores = self.score_rows_k_device(batch, targets, block, rowsk)?;
        let k = u32::try_from(block.len())
            .map_err(|_| invalid("coordinate block exceeds the CUDA index bound"))?;
        let eligible_words: Vec<u32> = eligible.iter().map(|&e| u32::from(e)).collect();
        let eligible_device = self.stream.clone_htod(&eligible_words).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let mut records = self
            .stream
            .alloc_zeros::<u64>(block.len() * RECORD_WORDS)
            .map_err(cuda)?;
        let mut member = self.stream.alloc_zeros::<u32>(block.len()).map_err(cuda)?;
        let mut summary = self.stream.alloc_zeros::<u64>(3).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&bvgk.choose);
        launch
            .arg(&scores.losses)
            .arg(&scores.status)
            .arg(&eligible_device)
            .arg(&k)
            .arg(&self.epoch)
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&scores.kind)
            .arg(&scores.indices)
            .arg(&mut records)
            .arg(&mut member)
            .arg(&mut summary);
        // SAFETY: k*16 losses, k status/eligible/index/member words, k*8 record
        // words and 3 summary words; block 0 strides over k, thread 0 reduces.
        unsafe { launch.launch(launch_config(k)) }.map_err(cuda)?;
        let summary_host = self.stream.clone_dtoh(&summary).map_err(cuda)?;
        let summary_words: [u64; 3] = summary_host
            .try_into()
            .map_err(|_| invalid("BVG-K summary must hold three words"))?;

        let l_joint = if summary_words[0] >= 2 {
            Some(self.joint_loss(
                batch,
                targets,
                summary_words[1],
                block,
                rowsk,
                &records,
                &member,
                scores.kind,
                bvgk,
            )?)
        } else {
            None
        };
        let joint_words = [u64::from(l_joint.is_some()), l_joint.unwrap_or(0)];
        let joint = self.stream.clone_htod(&joint_words).map_err(cuda)?;
        let mut commit = self.stream.alloc_zeros::<u32>(block.len()).map_err(cuda)?;
        let mut commit_count = self.stream.alloc_zeros::<u64>(1).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&bvgk.decide);
        launch
            .arg(&member)
            .arg(&summary)
            .arg(&k)
            .arg(&joint)
            .arg(&mut commit)
            .arg(&mut commit_count)
            .arg(&mut status);
        // SAFETY: k member and commit words, 3 summary, 2 joint, 1 count and 1
        // status word; one thread decides.
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        self.stream.synchronize().map_err(cuda)?;
        let status = self.stream.clone_dtoh(&status).map_err(cuda)?;
        bvgk_block::status_result("decide", status[0])?;
        let records_host = self.stream.clone_dtoh(&records).map_err(cuda)?;
        let commit_host = self.stream.clone_dtoh(&commit).map_err(cuda)?;
        let member_host = self.stream.clone_dtoh(&member).map_err(cuda)?;
        let count = self.stream.clone_dtoh(&commit_count).map_err(cuda)?[0];
        if self.epoch()? != scores.epoch {
            return Err(invalid("BVG-K proposal changed the model epoch"));
        }
        let record = bvgk_block::assemble_record(
            &records_host,
            summary_words,
            l_joint,
            &member_host,
            &commit_host,
            count,
        )?;
        Ok(BlockProposal {
            records,
            commit,
            commit_count: count,
            kind: scores.kind,
            k,
            rows: batch.rows,
            record,
            binding: self.binding.clone(),
            epoch: scores.epoch,
            stream: Arc::clone(&self.stream),
            commit_fn: bvgk.commit.clone(),
        })
    }

    /// `L(S_t)`: commit the choose `member` mask on a scratch clone, re-carry
    /// the same windows there (carry reads the table) and take lane 0 of a
    /// rows-K score of `block[best]` on the clone.
    #[expect(
        clippy::too_many_arguments,
        reason = "one joint forward needs each input"
    )]
    fn joint_loss(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        best: u64,
        block: &[Parameter],
        rowsk: &RowsKScoring,
        records: &CudaSlice<u64>,
        member: &CudaSlice<u32>,
        kind: u32,
        bvgk: &BvgkKernels,
    ) -> Result<u64> {
        let best = usize::try_from(best)
            .ok()
            .filter(|&best| best < block.len())
            .ok_or_else(|| invalid("BVG-K best ordinal is outside the block"))?;
        let mut scratch = self.scratch_clone()?;
        let before = scratch.epoch()?;
        scratch.launch_commit(&bvgk.commit, batch.rows, records, member, kind, block.len())?;
        if scratch.epoch()?
            != before
                .checked_add(1)
                .ok_or_else(|| invalid("epoch overflow"))?
        {
            return Err(invalid(
                "BVG-K scratch commit did not advance the epoch by one",
            ));
        }
        let windows = bvgk_block::split_windows(&batch.identities, &batch.valid_lengths)?;
        let carried = scratch.carry_windows(&windows)?;
        let scores = scratch.score_rows_k(&carried, targets, &[block[best]], rowsk)?;
        let score = scores
            .first()
            .ok_or_else(|| invalid("joint scoring returned no coordinate"))?;
        bvgk_block::status_result("joint scoring", score.status)?;
        score
            .losses
            .first()
            .copied()
            .ok_or_else(|| invalid("joint scoring returned no lane 0"))
    }

    /// Zero the status word, launch `discrete_bvgk_apply` with `mask` as the
    /// commit set and map a refusal.
    fn launch_commit(
        &mut self,
        function: &CudaFunction,
        rows: u32,
        records: &CudaSlice<u64>,
        mask: &CudaSlice<u32>,
        kind: u32,
        k: usize,
    ) -> Result<()> {
        let k = u32::try_from(k)
            .map_err(|_| invalid("coordinate block exceeds the CUDA index bound"))?;
        let spec = self.gpu_spec(rows)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(function);
        launch
            .arg(&mut self.table)
            .arg(&mut self.routes)
            .arg(&mut self.gates)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(records)
            .arg(mask)
            .arg(&kind)
            .arg(&k)
            .arg(&mut self.epoch)
            .arg(&mut status);
        // SAFETY: k*8 record and k mask words; one thread validates every member
        // before any write and advances the epoch once.
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        self.stream.synchronize().map_err(cuda)?;
        let status = self.stream.clone_dtoh(&status).map_err(cuda)?;
        bvgk_block::status_result("apply", status[0])
    }

    /// Commit a decided block in one epoch. A no-op batch (`commit_count == 0`)
    /// returns zero moves without a launch.
    pub(crate) fn apply_block(&mut self, proposal: &BlockProposal) -> Result<BlockCommit> {
        if proposal.binding != self.binding || !Arc::ptr_eq(&proposal.stream, &self.stream) {
            return Err(invalid("BVG-K proposal belongs to another model/stream"));
        }
        let before = self.epoch()?;
        if before != proposal.epoch {
            return Err(invalid("BVG-K proposal is stale at the current epoch"));
        }
        if proposal.commit_count == 0 {
            return Ok(BlockCommit {
                moves: 0,
                joint: false,
            });
        }
        let moves = u32::try_from(proposal.commit_count)
            .map_err(|_| invalid("BVG-K commit count exceeds the block"))?;
        self.launch_commit(
            &proposal.commit_fn,
            proposal.rows,
            &proposal.records,
            &proposal.commit,
            proposal.kind,
            proposal.k as usize,
        )?;
        if self.epoch()?
            != before
                .checked_add(1)
                .ok_or_else(|| invalid("epoch overflow"))?
        {
            return Err(invalid("BVG-K commit did not advance the epoch by one"));
        }
        Ok(BlockCommit {
            moves,
            joint: proposal.record.joint,
        })
    }
}

/// The embedded BVG-K source begins with the embedded K-coordinate source byte
/// for byte (the rows-K image is unchanged; BVG-K only appends).
pub fn bvgk_extends_rowsk_source() -> bool {
    BVGK_SOURCE.starts_with(super::rowsk_device::ROWSK_SOURCE)
}

/// Identity of the embedded BVG-K image, without a CUDA context.
pub fn compiled_bvgk_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(BVGK_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(BVGK_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(BVGK_CUBIN)),
        format!("{:x}", Sha256::digest(BVGK_RESOURCE_REPORT.as_bytes())),
    ]
}
