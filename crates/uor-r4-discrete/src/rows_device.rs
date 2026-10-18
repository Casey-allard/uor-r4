//! Explicit opt-in row-parallel counterfactual handle. Nothing selects it
//! implicitly: the ordinary `propose` keeps the one-block sixteen-lane kernel.
//! Its records are call-local allocations, like the ordinary loss buffer.
use super::*;

pub(super) const ROWS_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/rows.cu"));
pub(super) const ROWS_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/rows.ptx"));
pub(super) const ROWS_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rows.cubin"));
pub(super) const ROWS_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/rows-resources.txt"));

pub struct RowsCounterfactual {
    baseline: CudaFunction,
    eval: CudaFunction,
    reduce: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

impl Device {
    /// Load the opt-in row-parallel module for this device's model and stream.
    pub fn rows_counterfactual(&self) -> Result<RowsCounterfactual> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(ROWS_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(RowsCounterfactual {
            baseline: load(crate::rows_source::ENTRIES[0])?,
            eval: load(crate::rows_source::ENTRIES[1])?,
            reduce: load(crate::rows_source::ENTRIES[2])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// The ordinary proposal with its counterfactual losses produced by the
    /// row-parallel module. Admission, chooser, status reporting and the
    /// returned proposal are the ordinary ones.
    pub fn propose_rows(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        rows: &RowsCounterfactual,
    ) -> Result<Proposal> {
        if rows.binding != self.binding || !Arc::ptr_eq(&rows.stream, &self.stream) {
            return Err(invalid(
                "row-parallel handle belongs to another model/stream",
            ));
        }
        let (kind, index) = self.parameter(parameter)?;
        let epoch = self.admit_batch(batch, targets)?;
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let row_count = batch.rows as usize;
        let pairs = batch
            .rows
            .checked_mul(16)
            .ok_or_else(|| invalid("row-parallel pair count overflows"))?;
        let mut losses = self.stream.alloc_zeros::<u64>(16).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut base = self
            .stream
            .alloc_zeros::<u32>(row_count * 2)
            .map_err(cuda)?;
        let mut cost = self
            .stream
            .alloc_zeros::<u64>(row_count * 16)
            .map_err(cuda)?;
        let mut flag = self
            .stream
            .alloc_zeros::<u32>(row_count * 16)
            .map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&rows.baseline);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets)
            .arg(plan)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&mut base);
        // SAFETY: the ordinary admitted inputs plus rows*2 writable record
        // words; one thread per admitted row, no shared memory.
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&rows.eval);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets)
            .arg(plan)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&kind)
            .arg(&index)
            .arg(&base)
            .arg(&mut cost)
            .arg(&mut flag);
        // SAFETY: rows*16 writable cost and flag words; one thread per
        // (row, lane) pair, ordered after the baseline launch on this stream.
        unsafe { launch.launch(launch_config(pairs)) }.map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&rows.reduce);
        launch
            .arg(&spec)
            .arg(&kind)
            .arg(&index)
            .arg(&base)
            .arg(&cost)
            .arg(&flag)
            .arg(&mut losses)
            .arg(&mut status);
        // SAFETY: the fixed sixteen-word loss ABI and one status word; sixteen
        // threads, ordered after both producers on this stream.
        unsafe { launch.launch(launch_config(16)) }.map_err(cuda)?;
        // Same label as the ordinary path, so refusals read identically.
        check_status(&self.stream, &status, "counterfactual")?;

        let mut proposal = self.stream.alloc_zeros::<u64>(5).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.choose);
        launch
            .arg(&losses)
            .arg(&self.epoch)
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&kind)
            .arg(&index)
            .arg(&mut proposal);
        // SAFETY: identical to the ordinary chooser launch.
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        self.stream.synchronize().map_err(cuda)?;
        Ok(Proposal {
            data: proposal,
            losses,
            parameter,
            binding: self.binding.clone(),
            epoch,
            stream: Arc::clone(&self.stream),
        })
    }
}

/// Identity of the embedded row-parallel image, without a CUDA context.
pub fn compiled_rows_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(ROWS_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(ROWS_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(ROWS_CUBIN)),
        format!("{:x}", Sha256::digest(ROWS_RESOURCE_REPORT.as_bytes())),
    ]
}
