//! Explicit opt-in row-parallel counterfactual handle. Nothing selects it
//! implicitly: the ordinary `propose` keeps the one-block sixteen-lane kernel.
//! Internal records reuse exclusive per-device storage; proposal outputs stay call-local.
use super::*;

pub(super) const ROWS_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/rows.cu"));
pub(super) const ROWS_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/rows.ptx"));
pub(super) const ROWS_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rows.cubin"));
pub(super) const ROWS_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/rows-resources.txt"));

/// Reuse storage, never stale model-derived values; reset active prefixes per call.
pub(super) struct RowsScratch {
    targets: CudaSlice<u32>,
    base: CudaSlice<u32>,
    cost: CudaSlice<u64>,
    flag: CudaSlice<u32>,
    status: CudaSlice<u32>,
}

impl RowsScratch {
    fn new(stream: &Arc<CudaStream>, rows: usize) -> Result<Self> {
        Ok(Self {
            targets: stream.alloc_zeros(rows).map_err(cuda)?,
            base: stream.alloc_zeros(rows * 2).map_err(cuda)?,
            cost: stream.alloc_zeros(rows * 16).map_err(cuda)?,
            flag: stream.alloc_zeros(rows * 16).map_err(cuda)?,
            status: stream.alloc_zeros(1).map_err(cuda)?,
        })
    }

    fn reset(&mut self, stream: &Arc<CudaStream>, targets: &[u32]) -> Result<()> {
        let rows = targets.len();
        stream
            .memcpy_htod(targets, &mut self.targets.slice_mut(..rows))
            .map_err(cuda)?;
        stream
            .memset_zeros(&mut self.base.slice_mut(..rows * 2))
            .map_err(cuda)?;
        stream
            .memset_zeros(&mut self.cost.slice_mut(..rows * 16))
            .map_err(cuda)?;
        stream
            .memset_zeros(&mut self.flag.slice_mut(..rows * 16))
            .map_err(cuda)?;
        // A successful reducer need not write status. Never retain an old fault.
        stream.memset_zeros(&mut self.status).map_err(cuda)?;
        Ok(())
    }
}

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
        self.rows_scratch
            .with(
                |slot| {
                    if slot
                        .as_ref()
                        .is_none_or(|scratch| scratch.targets.len() < row_count)
                    {
                        // Replace only after a complete allocation; old storage is idle.
                        *slot = Some(RowsScratch::new(&self.stream, row_count)?);
                    }
                    let scratch = slot
                        .as_mut()
                        .ok_or_else(|| invalid("row scratch unavailable"))?;
                    scratch.reset(&self.stream, targets)?;
                    let RowsScratch {
                        targets,
                        base,
                        cost,
                        flag,
                        status,
                    } = scratch;
                    let mut losses = self.stream.alloc_zeros::<u64>(16).map_err(cuda)?;

                    let mut launch = self.stream.launch_builder(&rows.baseline);
                    launch
                        .arg(&self.table)
                        .arg(&self.routes)
                        .arg(&self.gates)
                        .arg(&batch.ids)
                        .arg(&batch.vectors)
                        .arg(&*targets)
                        .arg(plan)
                        .arg(&self.geometry)
                        .arg(&spec)
                        .arg(&mut *base);
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
                        .arg(&*targets)
                        .arg(plan)
                        .arg(&self.geometry)
                        .arg(&spec)
                        .arg(&kind)
                        .arg(&index)
                        .arg(&*base)
                        .arg(&mut *cost)
                        .arg(&mut *flag);
                    // SAFETY: rows*16 writable cost and flag words; one thread per
                    // (row, lane) pair, ordered after the baseline launch on this stream.
                    unsafe { launch.launch(launch_config(pairs)) }.map_err(cuda)?;

                    let mut launch = self.stream.launch_builder(&rows.reduce);
                    launch
                        .arg(&spec)
                        .arg(&kind)
                        .arg(&index)
                        .arg(&*base)
                        .arg(&*cost)
                        .arg(&*flag)
                        .arg(&mut losses)
                        .arg(&mut *status);
                    // SAFETY: the fixed sixteen-word loss ABI and one status word; sixteen
                    // threads, ordered after both producers on this stream.
                    unsafe { launch.launch(launch_config(16)) }.map_err(cuda)?;
                    // Same label as the ordinary path, so refusals read identically.
                    check_status(&self.stream, status, "counterfactual")?;

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
                    Ok(Proposal {
                        data: proposal,
                        losses,
                        parameter,
                        binding: self.binding.clone(),
                        epoch,
                        stream: Arc::clone(&self.stream),
                    })
                },
                || self.stream.synchronize().map_err(cuda),
            )
            .map_err(|error| match error {
                scratch_cache::CacheError::Operation(error) => error,
                scratch_cache::CacheError::Completion {
                    operation,
                    completion,
                } => Error::Cuda(format!(
                    "{operation}; row scratch completion also failed: {completion}"
                )),
                scratch_cache::CacheError::Unavailable => {
                    invalid("row scratch unavailable after interruption or synchronization failure")
                }
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

impl Device {
    /// `propose_rows` with this device's own lazily loaded module, so a driver
    /// loads the row-parallel image once per model rather than once per batch.
    pub fn propose_rows_cached(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
    ) -> Result<Proposal> {
        let rows = match self.rows_cache.get() {
            Some(rows) => rows,
            None => {
                let loaded = self.rows_counterfactual()?;
                // A concurrent first load on the same device keeps either handle;
                // both bind this exact model and stream.
                let _ = self.rows_cache.set(loaded);
                self.rows_cache
                    .get()
                    .ok_or_else(|| invalid("row-parallel module cache unavailable"))?
            }
        };
        self.propose_rows(batch, targets, parameter, rows)
    }
}
