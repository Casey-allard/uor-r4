use super::{Fail, Query};
use cudarc::driver::{result, sys, CudaContext, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;
pub const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/occurrence.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
pub struct Gpu {
    stream: Arc<CudaStream>,
    module: sys::CUmodule,
    context: Arc<CudaContext>,
}
impl Drop for Gpu {
    fn drop(&mut self) {
        if self.context.bind_to_thread().is_ok() {
            // SAFETY: the owned module is unloaded once after its launches.
            let _ = unsafe { result::module::unload(self.module) };
        }
    }
}
fn param<T>(value: &T) -> *mut c_void {
    std::ptr::from_ref(value).cast_mut().cast()
}
impl Gpu {
    // SAFETY: caller supplies arguments in the named kernel's ABI order and
    // owns all device buffers through the synchronization in this call.
    unsafe fn launch(&self, name: &str, n: u32, params: &mut [*mut c_void]) -> Result<(), Fail> {
        self.context.bind_to_thread()?;
        let f = unsafe { result::module::get_function(self.module, CString::new(name)?) }?;
        unsafe {
            result::launch_kernel(
                f,
                (n.div_ceil(128), 1, 1),
                (128, 1, 1),
                0,
                self.stream.cu_stream(),
                params,
            )
        }?;
        self.stream.synchronize()?;
        Ok(())
    }
    pub fn new() -> Result<Self, Fail> {
        let context = CudaContext::new(0)?;
        let (a, b) = context.compute_capability()?;
        if TARGET.trim() != format!("sm_{a}{b}") {
            return Err("selector cubin and GPU differ".into());
        }
        let stream = context.default_stream();
        context.bind_to_thread()?;
        // SAFETY: build.rs admits PTX then assembles and checks the ELF cubin.
        let module = unsafe { result::module::load_data(CUBIN.as_ptr().cast()) }?;
        Ok(Self {
            stream,
            module,
            context,
        })
    }
    pub fn execute(
        &self,
        queries: &[Query],
        roots: &[u32],
        start: u32,
        nrules: u32,
        labels: Option<&[[u32; 2]]>,
    ) -> Result<(Vec<u32>, Vec<u32>, Vec<u32>), Fail> {
        if queries.is_empty()
            || queries.len() > 4096
            || roots.len() != 4096
            || roots.iter().any(|&v| v > 65535)
            || nrules == 0
            || start.checked_add(nrules).is_none_or(|n| n > 192)
        {
            return Err("invalid selector device shape".into());
        }
        let (mut ids, mut lens) = (Vec::new(), Vec::new());
        for query in queries {
            query.validate()?;
            ids.extend_from_slice(&query.prefix);
            ids.resize(ids.len() + 192 - query.prefix.len(), 0u32);
            lens.push(query.prefix.len() as u32);
        }
        let rows = queries.len() as u32;
        let ids = self.stream.clone_htod(&ids)?;
        let lens = self.stream.clone_htod(&lens)?;
        let roots = self.stream.clone_htod(roots)?;
        let mut pos = self.stream.alloc_zeros::<u32>((rows * nrules) as usize)?;
        {
            let (ids, _i) = ids.device_ptr(&self.stream);
            let (lens, _l) = lens.device_ptr(&self.stream);
            let (roots, _k) = roots.device_ptr(&self.stream);
            let (pos, _p) = pos.device_ptr_mut(&self.stream);
            // SAFETY: one output per (row,rule); each validated row is padded to 192 tokens.
            unsafe {
                self.launch(
                    "occurrence_positions",
                    rows * nrules,
                    &mut [
                        param(&ids),
                        param(&lens),
                        param(&roots),
                        param(&rows),
                        param(&start),
                        param(&nrules),
                        param(&pos),
                    ],
                )
            }?;
        }
        let mut supports = Vec::new();
        let mut winners = Vec::new();
        if let Some(labels) = labels {
            if start != 0
                || nrules != 192
                || labels.len() != queries.len()
                || labels.iter().zip(queries).any(|(&[a, b], q)| {
                    a == b || a >= q.prefix.len() as u32 || b >= q.prefix.len() as u32
                })
            {
                return Err("invalid training labels or search grid".into());
            }
            let labels = self.stream.clone_htod(labels.as_flattened())?;
            let mut counts = self.stream.alloc_zeros::<u32>(384)?;
            let mut chosen = self.stream.alloc_zeros::<u32>(2)?;
            {
                let (pos, _p) = pos.device_ptr(&self.stream);
                let (labels, _l) = labels.device_ptr(&self.stream);
                let (counts, _c) = counts.device_ptr_mut(&self.stream);
                // SAFETY: the grid and the two source labels per row were validated above.
                unsafe {
                    self.launch(
                        "occurrence_fit",
                        384,
                        &mut [param(&pos), param(&labels), param(&rows), param(&counts)],
                    )
                }?;
            }
            {
                let (counts, _c) = counts.device_ptr(&self.stream);
                let (chosen, _w) = chosen.device_ptr_mut(&self.stream);
                // SAFETY: each winner addresses exactly one of the two 192-element vote arrays.
                unsafe {
                    self.launch(
                        "occurrence_winners",
                        2,
                        &mut [param(&counts), param(&chosen)],
                    )
                }?;
            }
            supports = self.stream.clone_dtoh(&counts)?;
            winners = self.stream.clone_dtoh(&chosen)?;
        }
        Ok((self.stream.clone_dtoh(&pos)?, supports, winners))
    }
}
