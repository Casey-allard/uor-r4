//! Explicit opt-in K-coordinate row-parallel scoring handle (BVG-K Stage C).
//! Nothing selects it implicitly; no driver path calls it. It scores K
//! coordinates of one kind in one launch and returns each coordinate's sixteen
//! losses and status. It runs no chooser and applies nothing: state and epoch
//! are unchanged.
use super::*;

pub(super) const ROWSK_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/rowsk.cu"));
pub(super) const ROWSK_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/rowsk.ptx"));
pub(super) const ROWSK_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rowsk.cubin"));
pub(super) const ROWSK_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/rowsk-resources.txt"));

/// Largest coordinate block one launch admits (BVG-K control V6 bound).
pub const MAX_COORDINATE_BLOCK: usize = 64;

pub struct RowsKScoring {
    baseline: CudaFunction,
    eval: CudaFunction,
    reduce: CudaFunction,
    binding: Binding,
    stream: Arc<CudaStream>,
}

/// One coordinate's scoring result: the ordinary sixteen-slot losses, or the
/// status code the ordinary proposal would have refused with (non-zero).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinateScore {
    pub parameter: Parameter,
    pub status: u32,
    pub losses: Vec<u64>,
}

impl Device {
    /// Load the opt-in K-coordinate module for this device's model and stream.
    pub fn rowsk_scoring(&self) -> Result<RowsKScoring> {
        admit_cuda_target(&self.context)?;
        let module = self
            .context
            .load_module(Ptx::from_binary(ROWSK_CUBIN.to_vec()))
            .map_err(cuda)?;
        let load = |name: &str| module.load_function(name).map_err(cuda);
        Ok(RowsKScoring {
            baseline: load(crate::rowsk_source::ENTRIES[0])?,
            eval: load(crate::rowsk_source::ENTRIES[3])?,
            reduce: load(crate::rowsk_source::ENTRIES[4])?,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    /// Score `parameters` (1..=64, all one kind) in one baseline launch, one
    /// eval launch over (coordinate, row, lane) and one reduce launch over
    /// (coordinate, lane). Each coordinate's losses and status equal an
    /// independent `propose_rows` for it on the same state (control V2).
    pub fn score_rows_k(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameters: &[Parameter],
        scoring: &RowsKScoring,
    ) -> Result<Vec<CoordinateScore>> {
        if scoring.binding != self.binding || !Arc::ptr_eq(&scoring.stream, &self.stream) {
            return Err(invalid(
                "K-coordinate handle belongs to another model/stream",
            ));
        }
        if parameters.is_empty() || parameters.len() > MAX_COORDINATE_BLOCK {
            return Err(invalid(format!(
                "coordinate block must hold 1..={MAX_COORDINATE_BLOCK} parameters"
            )));
        }
        let mut kind = None;
        let mut indices = Vec::with_capacity(parameters.len());
        for &parameter in parameters {
            let (k, index) = self.parameter(parameter)?;
            if *kind.get_or_insert(k) != k {
                return Err(invalid("a coordinate block must hold one parameter kind"));
            }
            indices.push(index);
        }
        let kind = kind.ok_or_else(|| invalid("empty coordinate block"))?;
        let block = u32::try_from(parameters.len())
            .map_err(|_| invalid("coordinate block exceeds the CUDA index bound"))?;
        let epoch_before = self.admit_batch(batch, targets)?;
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let indices_device = self.stream.clone_htod(&indices).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let plan = batch
            .plan
            .as_ref()
            .ok_or_else(|| invalid("missing admitted row plans"))?;
        let row_count = batch.rows as usize;
        let per = batch
            .rows
            .checked_mul(16)
            .ok_or_else(|| invalid("K-coordinate pair count overflows"))?;
        let triples = per
            .checked_mul(block)
            .ok_or_else(|| invalid("K-coordinate triple count overflows"))?;
        let mut base = self
            .stream
            .alloc_zeros::<u32>(row_count * 2)
            .map_err(cuda)?;
        let mut cost = self
            .stream
            .alloc_zeros::<u64>(triples as usize)
            .map_err(cuda)?;
        let mut flag = self
            .stream
            .alloc_zeros::<u32>(triples as usize)
            .map_err(cuda)?;
        let mut losses = self
            .stream
            .alloc_zeros::<u64>(parameters.len() * 16)
            .map_err(cuda)?;
        let mut status = self
            .stream
            .alloc_zeros::<u32>(parameters.len())
            .map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&scoring.baseline);
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
        // SAFETY: rows*2 base words; one thread per admitted row, as in the
        // row-parallel baseline (the identical kernel text).
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;

        let mut launch = self.stream.launch_builder(&scoring.eval);
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
            .arg(&indices_device)
            .arg(&block)
            .arg(&base)
            .arg(&mut cost)
            .arg(&mut flag);
        // SAFETY: block*rows*16 cost and flag words; one thread per
        // (coordinate, row, lane), ordered after the baseline on this stream.
        unsafe { launch.launch(launch_config(triples)) }.map_err(cuda)?;

        let reducers = block * 16;
        let mut launch = self.stream.launch_builder(&scoring.reduce);
        launch
            .arg(&spec)
            .arg(&kind)
            .arg(&indices_device)
            .arg(&block)
            .arg(&base)
            .arg(&cost)
            .arg(&flag)
            .arg(&mut losses)
            .arg(&mut status);
        // SAFETY: block*16 loss words and block status words; one thread per
        // (coordinate, lane), ordered after both producers on this stream.
        unsafe { launch.launch(launch_config(reducers)) }.map_err(cuda)?;
        self.stream.synchronize().map_err(cuda)?;
        let losses = self.stream.clone_dtoh(&losses).map_err(cuda)?;
        let status = self.stream.clone_dtoh(&status).map_err(cuda)?;
        if self.epoch()? != epoch_before {
            return Err(invalid("K-coordinate scoring changed the model epoch"));
        }
        Ok(parameters
            .iter()
            .enumerate()
            .map(|(c, &parameter)| CoordinateScore {
                parameter,
                status: status[c],
                losses: losses[c * 16..c * 16 + 16].to_vec(),
            })
            .collect())
    }
}

/// The embedded K-coordinate source begins with the embedded row-parallel
/// source byte for byte (the rows image is unchanged; rows-K only appends).
pub fn rowsk_extends_rows_source() -> bool {
    ROWSK_SOURCE.starts_with(super::rows_device::ROWS_SOURCE)
}

/// Identity of the embedded K-coordinate image, without a CUDA context.
pub fn compiled_rowsk_image_sha256() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(ROWSK_SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(ROWSK_PTX.as_bytes())),
        format!("{:x}", Sha256::digest(ROWSK_CUBIN)),
        format!("{:x}", Sha256::digest(ROWSK_RESOURCE_REPORT.as_bytes())),
    ]
}
