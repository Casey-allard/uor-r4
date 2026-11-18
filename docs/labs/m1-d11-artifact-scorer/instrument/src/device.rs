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
use uor_r4_integer::discrete_d11::{ExactCopyState, ExactCopyStep};

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
const POINTER_TRACE_WORDS: usize = 13;

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
    has_pointer_trunk: bool,
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
            has_pointer_trunk: tables.trunk.first() == Some(&1) && tables.shape[1] == 4,
            function,
            module,
            stream,
            blocks: tables.shape[1] as usize,
            capability,
        })
    }

    /// Scores every packed row on the device.
    pub fn score(&self, rows: &PackedRows) -> Result<Vec<RowScore>, String> {
        self.score_inner(rows, None, None)
            .map(|(scores, _, _)| scores)
    }

    /// Trace the native pointer (mode 0), replace it with an identity-Pick
    /// successor (1), or intervene at the pointer (2) or writer input (3).
    pub fn score_pointers(
        &self,
        rows: &PackedRows,
        interventions: &[[i32; 2]],
    ) -> Result<(Vec<RowScore>, Vec<[i32; POINTER_TRACE_WORDS]>), String> {
        if !self.has_pointer_trunk {
            return Err("pointer diagnostics require the four-block 19-table trunk".into());
        }
        if interventions.len() != rows.lengths.len()
            || interventions
                .iter()
                .zip(&rows.lengths)
                .any(|(&[mode, position], &length)| {
                    !(0..=4).contains(&mode)
                        || ((mode == 2 || mode == 3) && !(0..length).contains(&position))
                })
        {
            return Err("invalid pointer interventions".into());
        }
        self.score_inner(rows, Some(interventions), None)
            .map(|(scores, traces, _)| (scores, traces))
    }

    /// Commit the optional exact copy action on device, after the normal native
    /// writer. Each step's emitted ID is the feedback for the next causal step;
    /// the accompanying RowScore exposes the unmodified generator prediction.
    pub fn score_copy(
        &self,
        rows: &PackedRows,
        states: &[ExactCopyState],
        stops: [u32; 4],
    ) -> Result<(Vec<RowScore>, Vec<ExactCopyStep>), String> {
        if states.len() != rows.lengths.len()
            || !states
                .iter()
                .zip(&rows.lengths)
                .all(|(state, &len)| usize::try_from(len).is_ok_and(|n| state.valid_for(n)))
        {
            return Err("invalid exact-copy state or original prompt fence".into());
        }
        let (scores, _, steps) = self.score_inner(rows, None, Some((states, stops)))?;
        if scores.iter().any(|row| row.status != 0) {
            return Err("native scorer refused an exact-copy row".into());
        }
        Ok((scores, steps))
    }

    fn score_inner(
        &self,
        rows: &PackedRows,
        interventions: Option<&[[i32; 2]]>,
        copy: Option<(&[ExactCopyState], [u32; 4])>,
    ) -> Result<
        (
            Vec<RowScore>,
            Vec<[i32; POINTER_TRACE_WORDS]>,
            Vec<ExactCopyStep>,
        ),
        String,
    > {
        rows.check()?;
        let count = rows.lengths.len();
        let stride = rows.stride as usize;
        self.module.context.bind_to_thread().map_err(cuda)?;
        let mut scores = Vec::with_capacity(count);
        let mut traces = Vec::new();
        let mut steps = Vec::new();
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
            let intervention_buf = interventions
                .map(|v| {
                    self.stream
                        .clone_htod(v[start..start + n].as_flattened())
                        .map_err(cuda)
                })
                .transpose()?;
            let mut trace_buf = interventions
                .map(|_| {
                    self.stream
                        .alloc_zeros::<i32>(POINTER_TRACE_WORDS * n)
                        .map_err(cuda)
                })
                .transpose()?;
            let mut copy_buf = copy
                .map(|(states, stops)| {
                    let mut words = Vec::with_capacity(8 * n);
                    for state in &states[start..start + n] {
                        words.extend(state.words());
                        words.extend(stops);
                    }
                    self.stream.clone_htod(&words).map_err(cuda)
                })
                .transpose()?;
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
                let intervention_arg = intervention_buf
                    .as_ref()
                    .map(|b| b.device_ptr(&self.stream));
                let trace_arg = trace_buf.as_mut().map(|b| b.device_ptr_mut(&self.stream));
                let intervention_ptr = intervention_arg.as_ref().map_or(0, |(p, _)| *p);
                let trace_ptr = trace_arg.as_ref().map_or(0, |(p, _)| *p);
                let copy_arg = copy_buf.as_mut().map(|b| b.device_ptr_mut(&self.stream));
                let copy_ptr = copy_arg.as_ref().map_or(0, |(p, _)| *p);
                let mut n_rows = n as i32;
                let mut row_stride = rows.stride;
                let arg =
                    |value: &sys::CUdeviceptr| value as *const sys::CUdeviceptr as *mut c_void;
                let mut params: [*mut c_void; 17] = [
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
                    arg(&intervention_ptr),
                    arg(&trace_ptr),
                    arg(&copy_ptr),
                ];
                let threads = THREADS.min(n as u32).max(1);
                let grid = ((n as u32).div_ceil(threads), 1, 1);
                // SAFETY: the parameters point at values of the types
                // d11_score_rows takes, in its order, and outlive the launch;
                // the model sections and plans come from a reader-checked
                // artifact, and plans covers every length up to the context
                // (checked in new); prefixes holds n x stride tokens and
                // lengths and targets n words, each length within 0..=stride
                // (rows.check above); scratch holds n D11Scratch and out n
                // D11RowOut records; each thread with row < n touches only its
                // own. When present, interventions/traces have two/thirteen
                // 32-bit words per row; copy_io has eight u32s per row.
                // All buffers live through synchronization.
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
            if let Some(buf) = trace_buf {
                for record in self
                    .stream
                    .clone_dtoh(&buf)
                    .map_err(cuda)?
                    .chunks_exact(POINTER_TRACE_WORDS)
                {
                    traces.push(record.try_into().map_err(|_| "bad trace record")?);
                }
            }
            if let Some(buf) = copy_buf {
                for row in self.stream.clone_dtoh(&buf).map_err(cuda)?.chunks_exact(8) {
                    steps.push(ExactCopyStep::from_words(
                        row.try_into().map_err(|_| "bad exact-copy record")?,
                    ));
                }
            }
            start += n;
        }
        Ok((scores, traces, steps))
    }
}
