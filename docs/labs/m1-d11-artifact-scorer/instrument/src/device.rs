//! The GPU panel scorer (`cuda` feature): the admitted ahead-of-time cubin of
//! `src/scorer.cu`, one thread per row, rows launched in bounded chunks. The
//! host uploads the artifact's sections and the row plans once, then only
//! packs prefixes and reads back the row records.
//!
//! cudarc's safe module loader exists only with its `nvrtc` feature, which
//! would compile NVRTC bindings into this binary (CHARTER: NVRTC is banned from
//! every compiled module). So the cubin is loaded and launched through the
//! driver API directly: `cuModuleLoadData`, `cuModuleGetFunction`,
//! `cuLaunchKernel`. Nothing here compiles source at run time.
use crate::artifact::ArtifactTables;
use crate::kernel::{decode_rows, PackedRows, RowScore, PLAN_WORDS, ROW_OUT_WORDS, SCRATCH_WORDS};
use cudarc::driver::{result, sys, CudaContext, CudaSlice, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;

const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/d11_scorer.cubin"));
/// The admitted PTX the cubin was assembled from.
pub const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/d11_scorer.ptx"));
/// The `sm_` target the cubin was assembled for; there is no PTX fallback.
pub const IMAGE_TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
/// ptxas's register and spill report for the cubin.
pub const RESOURCES: &str = include_str!(concat!(env!("OUT_DIR"), "/d11_scorer-resources.txt"));
/// Rows per launch; bounds the scratch to CHUNK x 22,528 bytes.
const CHUNK: usize = 4096;
const THREADS: u32 = 64;

fn cuda(error: impl std::fmt::Display) -> String {
    format!("CUDA: {error}")
}

/// sha256 of the embedded cubin, for the run's identity record.
pub fn cubin_sha256() -> String {
    uor_r4_discrete::corpus::sha256(CUBIN)
}

/// sha256 of the embedded admitted PTX.
pub fn ptx_sha256() -> String {
    uor_r4_discrete::corpus::sha256(PTX.as_bytes())
}

/// A loaded module, unloaded on drop.
struct Module {
    raw: sys::CUmodule,
    context: Arc<CudaContext>,
}

impl Drop for Module {
    fn drop(&mut self) {
        if self.context.bind_to_thread().is_ok() {
            // SAFETY: `raw` came from cuModuleLoadData in this context and is
            // unloaded once, here; every function handle from it is dropped
            // with the scorer that owns this module.
            let _ = unsafe { result::module::unload(self.raw) };
        }
    }
}

/// One artifact resident on one device.
pub struct DeviceScorer {
    table: CudaSlice<i32>,
    cells: CudaSlice<u32>,
    routes: CudaSlice<u32>,
    gates: CudaSlice<i32>,
    shape: CudaSlice<i32>,
    plans: CudaSlice<i32>,
    trunk: CudaSlice<u32>,
    function: sys::CUfunction,
    module: Module,
    stream: Arc<CudaStream>,
    blocks: usize,
    /// The device's compute capability, `(major, minor)`.
    pub capability: (i32, i32),
}

impl DeviceScorer {
    pub fn new(ordinal: usize, tables: &ArtifactTables) -> Result<Self, String> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let capability = context.compute_capability().map_err(cuda)?;
        let (major, minor) = capability;
        if IMAGE_TARGET != format!("sm_{major}{minor}") {
            return Err(format!(
                "cubin target {IMAGE_TARGET} differs from device capability {major}.{minor}; no PTX fallback"
            ));
        }
        if tables.plans.len() != tables.shape[2] as usize * PLAN_WORDS {
            return Err("row plans do not cover the context".into());
        }
        context.bind_to_thread().map_err(cuda)?;
        let image = CUBIN.to_vec();
        // SAFETY: `image` is the complete ELF cubin nvcc assembled from the
        // admitted PTX (build.rs checks its ELF header); the driver copies it
        // during the call.
        let raw = unsafe { result::module::load_data(image.as_ptr().cast()) }.map_err(cuda)?;
        let module = Module {
            raw,
            context: Arc::clone(&context),
        };
        let name = CString::new("d11_score_rows").map_err(|e| e.to_string())?;
        // SAFETY: `module.raw` is the module just loaded and still alive.
        let function = unsafe { result::module::get_function(module.raw, name) }.map_err(cuda)?;
        let stream = context.default_stream();
        Ok(Self {
            table: stream.clone_htod(&tables.table).map_err(cuda)?,
            cells: stream.clone_htod(&tables.cells).map_err(cuda)?,
            routes: stream.clone_htod(&tables.routes).map_err(cuda)?,
            gates: stream.clone_htod(&tables.gates).map_err(cuda)?,
            shape: stream.clone_htod(&tables.shape).map_err(cuda)?,
            plans: stream.clone_htod(&tables.plans).map_err(cuda)?,
            trunk: stream.clone_htod(&tables.trunk).map_err(cuda)?,
            function,
            module,
            stream,
            blocks: tables.shape[1] as usize,
            capability,
        })
    }

    /// Scores every packed row on the device.
    pub fn score(&self, rows: &PackedRows) -> Result<Vec<RowScore>, String> {
        rows.check()?;
        let count = rows.lengths.len();
        let stride = rows.stride as usize;
        self.module.context.bind_to_thread().map_err(cuda)?;
        let mut scores = Vec::with_capacity(count);
        let mut start = 0;
        while start < count {
            let n = CHUNK.min(count - start);
            let prefixes = self
                .stream
                .clone_htod(&rows.prefixes[start * stride..(start + n) * stride])
                .map_err(cuda)?;
            let lengths = self
                .stream
                .clone_htod(&rows.lengths[start..start + n])
                .map_err(cuda)?;
            let targets = self
                .stream
                .clone_htod(&rows.targets[start..start + n])
                .map_err(cuda)?;
            let mut scratch = self
                .stream
                .alloc_zeros::<u32>(n * SCRATCH_WORDS)
                .map_err(cuda)?;
            let mut out = self
                .stream
                .alloc_zeros::<u32>(n * ROW_OUT_WORDS)
                .map_err(cuda)?;
            {
                let (table, _g0) = self.table.device_ptr(&self.stream);
                let (cells, _g1) = self.cells.device_ptr(&self.stream);
                let (routes, _g2) = self.routes.device_ptr(&self.stream);
                let (gates, _g3) = self.gates.device_ptr(&self.stream);
                let (shape, _g4) = self.shape.device_ptr(&self.stream);
                let (plans, _g5) = self.plans.device_ptr(&self.stream);
                let (trunk, _g_trunk) = self.trunk.device_ptr(&self.stream);
                let (prefix_ptr, _g6) = prefixes.device_ptr(&self.stream);
                let (length_ptr, _g7) = lengths.device_ptr(&self.stream);
                let (target_ptr, _g8) = targets.device_ptr(&self.stream);
                let (scratch_ptr, _g9) = scratch.device_ptr_mut(&self.stream);
                let (out_ptr, _g10) = out.device_ptr_mut(&self.stream);
                let mut n_rows = n as i32;
                let mut row_stride = rows.stride;
                let arg =
                    |value: &sys::CUdeviceptr| value as *const sys::CUdeviceptr as *mut c_void;
                let mut params: [*mut c_void; 14] = [
                    arg(&table),
                    arg(&cells),
                    arg(&routes),
                    arg(&gates),
                    arg(&shape),
                    arg(&plans),
                    arg(&trunk),
                    arg(&prefix_ptr),
                    arg(&length_ptr),
                    arg(&target_ptr),
                    (&mut n_rows as *mut i32).cast(),
                    (&mut row_stride as *mut i32).cast(),
                    arg(&scratch_ptr),
                    arg(&out_ptr),
                ];
                let threads = THREADS.min(n as u32).max(1);
                let grid = ((n as u32).div_ceil(threads), 1, 1);
                // SAFETY: the thirteen parameters point at values of the types
                // d11_score_rows takes, in its order, and outlive the launch;
                // the model sections and plans come from a reader-checked
                // artifact, and plans covers every length up to the context
                // (checked in new); prefixes holds n x stride tokens and
                // lengths and targets n words, each length within 0..=stride
                // (rows.check above); scratch holds n D11Scratch and out n
                // D11RowOut records; each thread with row < n touches only its
                // own.
                unsafe {
                    result::launch_kernel(
                        self.function,
                        grid,
                        (threads, 1, 1),
                        0,
                        self.stream.cu_stream(),
                        &mut params,
                    )
                }
                .map_err(cuda)?;
            }
            self.stream.synchronize().map_err(cuda)?;
            let words = self.stream.clone_dtoh(&out).map_err(cuda)?;
            scores.extend(decode_rows(&words, n, self.blocks)?);
            start += n;
        }
        Ok(scores)
    }
}
