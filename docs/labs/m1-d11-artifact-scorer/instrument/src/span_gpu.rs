use super::Fail;
use cudarc::driver::{result, sys, CudaContext, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;
use uor_r4_integer::{
    content_selector::{BOUNDARY, WIDTH},
    span_selector::{Bank, Relation, CAP},
};
pub const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/span.cubin"));
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
    pub fn fit(&self, data: &[u32], roots: &[u32]) -> Result<Vec<Relation>, Fail> {
        let rows = validate(data, roots, true)?;
        let data = self.stream.clone_htod(data)?;
        let roots = self.stream.clone_htod(roots)?;
        let mut bank = self.stream.alloc_zeros::<u32>(rows as usize * 5)?;
        let mut count = self.stream.alloc_zeros::<u32>(1)?;
        {
            let (data, _d) = data.device_ptr(&self.stream);
            let (roots, _r) = roots.device_ptr(&self.stream);
            let (bank, _b) = bank.device_ptr_mut(&self.stream);
            let (count, _c) = count.device_ptr_mut(&self.stream);
            // SAFETY: validated rows have 1..=3 connector IDs; the single writer
            // can insert at most one five-word record per row.
            unsafe {
                self.launch(
                    "span_fit",
                    1,
                    &mut [
                        param(&data),
                        param(&roots),
                        param(&rows),
                        param(&bank),
                        param(&count),
                    ],
                )?;
            }
        }
        let n = self.stream.clone_dtoh(&count)?[0];
        if n == 0 || n > rows {
            return Err("device produced invalid relation inventory".into());
        }
        let words = self.stream.clone_dtoh(&bank)?;
        Ok(words[..n as usize * 5]
            .chunks_exact(5)
            .map(|r| r.try_into().unwrap())
            .collect())
    }
    pub fn predict(
        &self,
        data: &[u32],
        roots: &[u32],
        bank: &Bank,
        oldroot: u32,
    ) -> Result<Vec<[u32; 21]>, Fail> {
        let rows = validate(data, roots, false)?;
        bank.validate()?;
        if oldroot > 65535 {
            return Err("invalid legacy relation root".into());
        }
        let words: Vec<_> = bank.relations.iter().flatten().copied().collect();
        let n = bank.relations.len() as u32;
        let data = self.stream.clone_htod(data)?;
        let roots = self.stream.clone_htod(roots)?;
        let classes = self.stream.clone_htod(&bank.classes)?;
        let bank = self.stream.clone_htod(&words)?;
        let mut out = self.stream.alloc_zeros::<u32>(rows as usize * 21)?;
        {
            let (d, _d) = data.device_ptr(&self.stream);
            let (r, _r) = roots.device_ptr(&self.stream);
            let (c, _c) = classes.device_ptr(&self.stream);
            let (b, _b) = bank.device_ptr(&self.stream);
            let (o, _o) = out.device_ptr_mut(&self.stream);
            // SAFETY: validated native rows index the 4096-element codebook and
            // token class table. Every worker writes its own 21-word output.
            unsafe {
                self.launch(
                    "span_predict",
                    rows,
                    &mut [
                        param(&d),
                        param(&r),
                        param(&c),
                        param(&b),
                        param(&n),
                        param(&oldroot),
                        param(&rows),
                        param(&o),
                    ],
                )?;
            }
        }
        Ok(self
            .stream
            .clone_dtoh(&out)?
            .chunks_exact(21)
            .map(|r| r.try_into().unwrap())
            .collect())
    }
}
pub const STRIDE: usize = WIDTH + 3;
fn validate(data: &[u32], roots: &[u32], fit: bool) -> Result<u32, Fail> {
    let n = data.len() / STRIDE;
    if !(1..=CAP).contains(&n)
        || !data.len().is_multiple_of(STRIDE)
        || roots.len() != 4096
        || roots.iter().any(|&v| v > 65535)
    {
        return Err("invalid device buffers".into());
    }
    for row in data.chunks_exact(STRIDE) {
        let len = row[WIDTH] as usize;
        if !(1..=WIDTH).contains(&len) || row[..len].iter().any(|&i| i >= 4096) {
            return Err("invalid device prefix".into());
        }
        let a = row[WIDTH + 1];
        let b = row[WIDTH + 2];
        let valid = if fit {
            b < len as u32 && b > a && (2..=4).contains(&(b - a))
        } else if a == BOUNDARY || b == BOUNDARY {
            a == b
        } else {
            a < len as u32 && b < a && a - b >= 2 && row[b as usize] == row[a as usize]
        };
        if !valid {
            return Err("invalid device positions".into());
        }
    }
    Ok(n as u32)
}
