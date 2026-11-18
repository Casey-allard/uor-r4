use super::Fail;
use cudarc::driver::{result, sys, CudaContext, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;
use uor_r4_integer::{
    content_selector::{BOUNDARY, WIDTH},
    span_selector::{Bank, CAP},
};
pub const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/continuation.cubin"));
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
    pub fn predict(
        &self,
        rows: &[[u32; STRIDE]],
        roots: &[u32],
        bank: &Bank,
    ) -> Result<Vec<[u32; 4]>, Fail> {
        bank.validate()?;
        if rows.is_empty()
            || rows.len() > CAP
            || roots.len() != 4096
            || roots.iter().any(|&r| r > 65535)
        {
            return Err("invalid continuation device buffers".into());
        }
        for row in rows {
            let [end, limit, start, starts, ordered] = row[WIDTH..].try_into()?;
            if row[..WIDTH].iter().any(|&id| id >= 4096)
                || starts > 7
                || ordered > 7
                || (start != BOUNDARY
                    && (limit >= WIDTH as u32
                        || end >= limit
                        || start >= limit
                        || !(2..=4).contains(&start.saturating_sub(end))
                        || starts & (1 << (start - end - 2)) == 0))
            {
                return Err("invalid continuation device fence".into());
            }
        }
        let n = rows.len() as u32;
        let count = bank.relations.len() as u32;
        let data = self.stream.clone_htod(rows.as_flattened())?;
        let roots = self.stream.clone_htod(roots)?;
        let classes = self.stream.clone_htod(&bank.classes)?;
        let relations = self.stream.clone_htod(bank.relations.as_flattened())?;
        let mut output = self.stream.alloc_zeros::<u32>(rows.len() * 4)?;
        {
            let (data, _data) = data.device_ptr(&self.stream);
            let (roots, _roots) = roots.device_ptr(&self.stream);
            let (classes, _classes) = classes.device_ptr(&self.stream);
            let (relations, _relations) = relations.device_ptr(&self.stream);
            let (output, _output) = output.device_ptr_mut(&self.stream);
            // SAFETY: all active source/fence words and relation indices were checked above.
            // Each work item owns exactly one output row and its inputs stay live until sync.
            unsafe {
                self.launch(
                    "continuation_predict",
                    n,
                    &mut [
                        param(&data),
                        param(&roots),
                        param(&classes),
                        param(&relations),
                        param(&count),
                        param(&n),
                        param(&output),
                    ],
                )?;
            }
        }
        self.stream
            .clone_dtoh(&output)?
            .chunks_exact(4)
            .map(|r| r.try_into().map_err(Into::into))
            .collect()
    }
}
pub const STRIDE: usize = WIDTH + 5;
