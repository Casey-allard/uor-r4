//! Explicit diagnostic device handle. Neither ordinary proposals nor loaders
//! select the clocked module implicitly. All buffers/events are precreated.
use super::*;
use crate::decomposition::{self, KernelResources};
pub(super) const DIAGNOSTIC_PTX: &str =
    include_str!(concat!(env!("OUT_DIR"), "/decomposition.ptx"));

pub struct DecompositionProfiler {
    pub(super) kernel: CudaFunction,
    pub(super) records: CudaSlice<u64>,
    pub(super) submitted: bool,
    binding: Binding,
}
impl DecompositionProfiler {
    /// Raw terminal words are retained even when decoding/parity fails.
    pub fn raw_trace(&self) -> Result<Vec<u64>> {
        if !self.submitted {
            return Err(invalid("diagnostic kernel was not submitted"));
        }
        self.records
            .stream()
            .clone_dtoh(&self.records)
            .map_err(cuda)
    }
    pub fn resources(&self) -> Result<KernelResources> {
        resources(
            &self.kernel,
            include_str!(concat!(env!("OUT_DIR"), "/decomposition-resources.txt")),
            "discrete_counterfactual_decomposition",
        )
    }
}
fn resources(kernel: &CudaFunction, ptxas: &str, entry: &str) -> Result<KernelResources> {
    use cudarc::driver::sys::CUfunction_attribute_enum as Attr;
    let get = |attr| -> Result<u64> {
        u64::try_from(kernel.get_attribute(attr).map_err(cuda)?)
            .map_err(|_| invalid("negative CUDA resource metadata"))
    };
    Ok(KernelResources {
        registers: get(Attr::CU_FUNC_ATTRIBUTE_NUM_REGS)?,
        local_bytes: get(Attr::CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES)?,
        shared_bytes: get(Attr::CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES)?,
        max_threads_per_block: get(Attr::CU_FUNC_ATTRIBUTE_MAX_THREADS_PER_BLOCK)?,
        ptxas: decomposition::parse_ptxas(ptxas, entry).map_err(invalid)?,
    })
}
impl Device {
    pub fn decomposition_profiler(&self) -> Result<DecompositionProfiler> {
        if self.read_spec() != ReadSpec::Route {
            return Err(invalid("decomposition requires Route read"));
        }
        let module = self
            .context
            .load_module(Ptx::from_src(DIAGNOSTIC_PTX))
            .map_err(cuda)?;
        Ok(DecompositionProfiler {
            kernel: module
                .load_function("discrete_counterfactual_decomposition")
                .map_err(cuda)?,
            records: self
                .stream
                .alloc_zeros::<u64>(decomposition::LANES * decomposition::WORDS)
                .map_err(cuda)?,
            submitted: false,
            binding: self.binding.clone(),
        })
    }
    pub fn ordinary_counterfactual_resources(&self) -> Result<KernelResources> {
        resources(
            &self.kernels.counterfactual,
            include_str!(concat!(env!("OUT_DIR"), "/ordinary-resources.txt")),
            "discrete_counterfactual",
        )
    }
    pub fn propose_decomposed(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profiler: &mut ProposeProfiler,
        probe: &mut DecompositionProfiler,
    ) -> Result<Proposal> {
        probe.submitted = false;
        profiler.begin();
        if probe.binding != self.binding
            || profiler.model != self.binding.model
            || !Arc::ptr_eq(&profiler.stream, &self.stream)
            || !matches!(parameter, Parameter::Route(_) | Parameter::VectorBlock(_))
        {
            return Err(invalid(
                "decomposition requires same model/stream and Route/VectorBlock parameter",
            ));
        }
        let result = self.propose_inner(batch, targets, parameter, Some(profiler), Some(probe));
        if result.is_ok() {
            profiler.mark(8);
        }
        if let Some(started) = profiler.started {
            profiler.host.call_ns = if result.is_ok() {
                profiler
                    .boundary
                    .map(|end| end.duration_since(started).as_nanos())
                    .unwrap_or(0)
            } else {
                started.elapsed().as_nanos()
            };
        }
        result
    }
}
