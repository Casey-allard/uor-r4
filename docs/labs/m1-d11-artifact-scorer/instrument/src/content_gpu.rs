use super::Fail;
use cudarc::driver::{result, sys, CudaContext, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;
use uor_r4_integer::content_selector::{Choice, Prototypes, BOUNDARY, BRIDGE, WIDTH};
pub const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/content.cubin"));
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
        Self::load(CUBIN)
    }
    fn load(cubin: &[u8]) -> Result<Self, Fail> {
        let context = CudaContext::new(0)?;
        let (a, b) = context.compute_capability()?;
        if TARGET.trim() != format!("sm_{a}{b}") {
            return Err("selector cubin and GPU differ".into());
        }
        let stream = context.default_stream();
        context.bind_to_thread()?;
        // SAFETY: build.rs admits PTX then assembles and checks the ELF cubin.
        let module = unsafe { result::module::load_data(cubin.as_ptr().cast()) }?;
        Ok(Self {
            stream,
            module,
            context,
        })
    }
    pub fn fit(&self, data: &[u32], roots: &[u32]) -> Result<Prototypes, Fail> {
        let rows = validate(data, roots, true)?;
        let bank = self.stream.alloc_zeros::<u32>(4 * CAP * 16)?;
        let counts = self.stream.alloc_zeros::<u32>(4)?;
        let (mut bank, mut counts) = (bank, counts);
        let data = self.stream.clone_htod(data)?;
        let roots = self.stream.clone_htod(roots)?;
        {
            let (data, _d) = data.device_ptr(&self.stream);
            let (roots, _r) = roots.device_ptr(&self.stream);
            let (bank, _b) = bank.device_ptr_mut(&self.stream);
            let (counts, _c) = counts.device_ptr_mut(&self.stream);
            // SAFETY: one thread per disjoint bank, at most one record per TRAIN row.
            unsafe {
                self.launch(
                    "content_fit",
                    4,
                    &mut [
                        param(&data),
                        param(&roots),
                        param(&rows),
                        param(&bank),
                        param(&counts),
                    ],
                )
            }?;
        }
        let words = self.stream.clone_dtoh(&bank)?;
        let counts = self.stream.clone_dtoh(&counts)?;
        if counts.iter().any(|&n| n == 0 || n > rows) {
            return Err("bad device prototype counts".into());
        }
        let read = |kind: usize| {
            words[kind * CAP * 16..kind * CAP * 16 + counts[kind] as usize * 16].chunks_exact(16)
        };
        let m = Prototypes {
            requested: read(0).map(|p| p[..4].try_into().unwrap()).collect(),
            excluded: read(1).map(|p| p[..4].try_into().unwrap()).collect(),
            fact: read(2).map(|p| p[..4].try_into().unwrap()).collect(),
            relation: read(3).map(|p| p.try_into().unwrap()).collect(),
        };
        m.validate()?;
        Ok(m)
    }
    pub fn predict(
        &self,
        data: &[u32],
        roots: &[u32],
        m: &Prototypes,
        ablated: bool,
    ) -> Result<Vec<Choice>, Fail> {
        let (choices, _) = self.rank(data, roots, m, ablated, None)?;
        Ok(choices)
    }
    #[cfg(feature = "entity-pair-chain")]
    pub fn traced(
        data: &[u32],
        roots: &[u32],
        m: &Prototypes,
        classes: &[u32],
        gated: bool,
    ) -> Result<(Vec<Choice>, Vec<u32>), Fail> {
        if classes.len() != 4096 || classes.iter().any(|&v| v > 2) {
            return Err("invalid entity rank classes".into());
        }
        Self::load(include_bytes!(concat!(env!("OUT_DIR"), "/entity.cubin")))?.rank(
            data,
            roots,
            m,
            false,
            Some((classes, gated)),
        )
    }
    fn rank(
        &self,
        data: &[u32],
        roots: &[u32],
        m: &Prototypes,
        ablated: bool,
        gate: Option<(&[u32], bool)>,
    ) -> Result<(Vec<Choice>, Vec<u32>), Fail> {
        let rows = validate(data, roots, false)?;
        m.validate()?;
        let mut packed = vec![0u32; 4 * CAP * 16];
        for (g, bank) in [&m.requested, &m.excluded, &m.fact].into_iter().enumerate() {
            for (i, row) in bank.iter().enumerate() {
                let start = 16 * (g * CAP + i);
                packed[start..start + 4].copy_from_slice(row);
            }
        }
        for (i, r) in m.relation.iter().enumerate() {
            let start = (3 * CAP + i) * 16;
            packed[start..start + 16].copy_from_slice(r);
        }
        let counts = [
            m.requested.len() as u32,
            m.excluded.len() as u32,
            m.fact.len() as u32,
            m.relation.len() as u32,
        ];
        let counts = self.stream.clone_htod(&counts)?;
        let bank = self.stream.clone_htod(&packed)?;
        let data = self.stream.clone_htod(data)?;
        let roots = self.stream.clone_htod(roots)?;
        let mut scores = self.stream.alloc_zeros::<u32>(rows as usize * WIDTH * 2)?;
        let class_buf = gate.map(|(c, _)| self.stream.clone_htod(c)).transpose()?;
        let mut out = self.stream.alloc_zeros::<u32>(rows as usize * 3)?;
        {
            let (data, _d) = data.device_ptr(&self.stream);
            let (roots, _r) = roots.device_ptr(&self.stream);
            let (bank, _b) = bank.device_ptr(&self.stream);
            let (counts, _c) = counts.device_ptr(&self.stream);
            let (score, _s) = scores.device_ptr_mut(&self.stream);
            let (out, _o) = out.device_ptr_mut(&self.stream);
            // SAFETY: the first kernel fills two scores per real token. The second
            // reads only live positions and writes exactly three choices per row.
            unsafe {
                self.launch(
                    "content_sites",
                    rows * WIDTH as u32,
                    &mut [
                        param(&data),
                        param(&roots),
                        param(&rows),
                        param(&bank),
                        param(&counts),
                        param(&u32::from(ablated)),
                        param(&score),
                    ],
                )?;
                let mut params = vec![
                    param(&data),
                    param(&roots),
                    param(&rows),
                    param(&bank),
                    param(&counts),
                    param(&score),
                    param(&out),
                ];
                if let (Some(buf), Some((_, gated))) = (&class_buf, gate) {
                    let (classes, _c) = buf.device_ptr(&self.stream);
                    let enabled = u32::from(gated);
                    params.extend([param(&classes), param(&enabled)]);
                    self.launch("entity_bind", rows, &mut params)?;
                } else {
                    self.launch("content_bind", rows, &mut params)?;
                }
            }
        }
        let pos = |n: u32| (n != BOUNDARY).then_some(n);
        let traces = if gate.is_some() {
            self.stream.clone_dtoh(&scores)?
        } else {
            Vec::new()
        };
        Ok((
            self.stream
                .clone_dtoh(&out)?
                .chunks_exact(3)
                .map(|w| Choice {
                    query: pos(w[0]),
                    fact: pos(w[1]),
                    source: pos(w[2]),
                })
                .collect(),
            traces,
        ))
    }
}
pub const STRIDE: usize = WIDTH + 5;
const CAP: usize = 288;
pub const MAX_PREDICTIONS: usize = 4096;
fn validate(data: &[u32], roots: &[u32], fit: bool) -> Result<u32, Fail> {
    let n = data.len() / STRIDE;
    if !data.len().is_multiple_of(STRIDE)
        || n == 0
        || n > if fit { CAP } else { MAX_PREDICTIONS }
        || roots.len() != 4096
        || roots.iter().any(|&v| v > 65535)
    {
        return Err("invalid entity selector device buffer".into());
    }
    for r in data.chunks_exact(STRIDE) {
        let len = r[WIDTH] as usize;
        if len == 0 || len > WIDTH || r[..len].iter().any(|&i| i >= 4096) {
            return Err("device prefix out of range".into());
        }
        if fit {
            let (q, x, f, s) = (r[WIDTH + 1], r[WIDTH + 2], r[WIDTH + 3], r[WIDTH + 4]);
            if q >= len as u32
                || (x != BOUNDARY && x >= len as u32)
                || f >= s
                || s >= q
                || s - f - 1 > BRIDGE as u32
                || r[f as usize] != r[q as usize]
            {
                return Err("invalid training anchor or relation length".into());
            }
        }
    }
    Ok(n as u32)
}
