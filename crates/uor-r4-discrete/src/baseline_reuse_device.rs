//! Explicit opt-in R handle. The only reusable allocation is audit metadata;
//! baseline status/eligibility live in call-local CUDA shared memory.
use super::*;
use crate::baseline_reuse;
use crate::decomposition::{self, KernelResources};

pub(super) const REUSE_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/baseline-reuse.cu"));
pub(super) const REUSE_PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/baseline-reuse.ptx"));
pub(super) const REUSE_CUBIN: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/baseline-reuse.cubin"));
pub(super) const REUSE_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/baseline-reuse-resources.txt"));

pub struct BaselineReuseProfiler {
    pub(super) kernel: CudaFunction,
    pub(super) records: CudaSlice<u64>,
    pub(super) submitted: bool,
    binding: Binding,
    stream: Arc<CudaStream>,
}
impl BaselineReuseProfiler {
    /// Complete raw audit, including terminal records from expected refusals.
    /// Any new proposal attempt invalidates the previous audit before admission.
    pub fn raw_trace(&self) -> Result<Vec<u64>> {
        if !self.submitted {
            return Err(invalid("baseline reuse kernel was not submitted"));
        }
        self.stream.clone_dtoh(&self.records).map_err(cuda)
    }
    pub fn resources(&self) -> Result<KernelResources> {
        use cudarc::driver::sys::CUfunction_attribute_enum as Attr;
        let get = |attr| -> Result<u64> {
            u64::try_from(self.kernel.get_attribute(attr).map_err(cuda)?)
                .map_err(|_| invalid("negative CUDA resource metadata"))
        };
        let resources = KernelResources {
            registers: get(Attr::CU_FUNC_ATTRIBUTE_NUM_REGS)?,
            local_bytes: get(Attr::CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES)?,
            shared_bytes: get(Attr::CU_FUNC_ATTRIBUTE_SHARED_SIZE_BYTES)?,
            max_threads_per_block: get(Attr::CU_FUNC_ATTRIBUTE_MAX_THREADS_PER_BLOCK)?,
            ptxas: decomposition::parse_ptxas(
                REUSE_RESOURCE_REPORT,
                "discrete_counterfactual_baseline_reuse",
            )
            .map_err(invalid)?,
        };
        let reported_shared = baseline_reuse::report_shared_bytes(
            REUSE_RESOURCE_REPORT,
            "discrete_counterfactual_baseline_reuse",
        )
        .map_err(invalid)?;
        if resources.shared_bytes != reported_shared
            || reported_shared != baseline_reuse::SHARED_BYTES
        {
            return Err(invalid(
                "baseline reuse runtime/AOT static shared memory differs from512 bytes",
            ));
        }
        Ok(resources)
    }
}

impl Device {
    pub fn baseline_reuse_profiler(&self) -> Result<BaselineReuseProfiler> {
        if self.read_spec() != ReadSpec::Route {
            return Err(invalid("baseline reuse requires Route read"));
        }
        admit_cuda_target(&self.context)?;
        let image = compiled_baseline_reuse_image();
        image.validate().map_err(invalid)?;
        if image.target != CUDA_IMAGE_TARGET {
            return Err(invalid(
                "baseline reuse AOT target differs from ordinary target",
            ));
        }
        let module = self
            .context
            .load_module(Ptx::from_binary(REUSE_CUBIN.to_vec()))
            .map_err(cuda)?;
        Ok(BaselineReuseProfiler {
            kernel: module
                .load_function("discrete_counterfactual_baseline_reuse")
                .map_err(cuda)?,
            records: self
                .stream
                .alloc_zeros::<u64>(baseline_reuse::WORDS)
                .map_err(cuda)?,
            submitted: false,
            binding: self.binding.clone(),
            stream: Arc::clone(&self.stream),
        })
    }

    pub fn propose_reused(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profiler: &mut ProposeProfiler,
        reuse: &mut BaselineReuseProfiler,
    ) -> Result<Proposal> {
        reuse.submitted = false;
        profiler.begin();
        if reuse.binding != self.binding || !Arc::ptr_eq(&reuse.stream, &self.stream) {
            return Err(invalid(
                "baseline reuse handle belongs to another model/stream",
            ));
        }
        if profiler.model != self.binding.model || !Arc::ptr_eq(&profiler.stream, &self.stream) {
            return Err(invalid("proposal profiler belongs to another model/stream"));
        }
        if self.read_spec() != ReadSpec::Route
            || !matches!(parameter, Parameter::Route(_) | Parameter::VectorBlock(_))
        {
            return Err(invalid(
                "baseline reuse requires Route read and Route/VectorBlock parameter",
            ));
        }
        // Existing admission binds this call's batch/targets/index/epoch. The
        // baseline cache is recomputed inside this call, never held by a handle.
        let result = self.propose_inner_with_reuse(
            batch,
            targets,
            parameter,
            Some(profiler),
            None,
            Some(reuse),
        );
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
