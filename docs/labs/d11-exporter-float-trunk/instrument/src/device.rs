//! The CUDA evaluator: the artifact's FBS image in device memory and one
//! thread per sequence through `fbs_eval_sequences` (src/evaluator.cu).

use crate::{sha256_hex, work_bytes, CLayer, CMap, CModel, Outputs};
use cudarc::driver::{result, sys, CudaContext, CudaSlice, CudaStream, DevicePtr, DevicePtrMut};
use std::ffi::{c_void, CString};
use std::sync::Arc;
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::stack::{FbsImage, FbsMapImage, IntegerStackModel};

const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fbs_evaluator.cubin"));
pub const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/fbs_evaluator.ptx"));
pub const IMAGE_TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
pub const RESOURCES: &str = include_str!(concat!(env!("OUT_DIR"), "/fbs_evaluator-resources.txt"));
/// Sequences per launch.
const CHUNK: usize = 4096;
const THREADS: u32 = 64;

fn cuda(error: impl std::fmt::Display) -> String {
    format!("CUDA: {error}")
}

pub fn cubin_sha256() -> String {
    sha256_hex(CUBIN)
}

pub fn ptx_sha256() -> String {
    sha256_hex(PTX.as_bytes())
}

struct Module {
    raw: sys::CUmodule,
    context: Arc<CudaContext>,
}

impl Drop for Module {
    fn drop(&mut self) {
        if self.context.bind_to_thread().is_ok() {
            // SAFETY: `raw` was loaded by this context and is unloaded once.
            let _ = unsafe { result::module::unload(self.raw) };
        }
    }
}

/// The FBS image and the loaded module on one device.
pub struct DeviceEvaluator {
    /// Every device array the model's pointers name; kept alive with it.
    _arrays: Vec<CudaSlice<u8>>,
    model: CModel,
    work_bytes: u64,
    function: sys::CUfunction,
    /// `fbs_trace_sequences`: the same run with each position's snap trace (claude-134).
    trace_function: sys::CUfunction,
    module: Module,
    stream: Arc<CudaStream>,
    pub capability: (i32, i32),
}

/// Copy `data` to the device as bytes and return its device address.
fn upload<T: Copy>(
    stream: &Arc<CudaStream>,
    arrays: &mut Vec<CudaSlice<u8>>,
    data: &[T],
) -> Result<u64, String> {
    // SAFETY: `data` is a slice of plain integers (or structs of them), read
    // as its own bytes.
    let bytes = unsafe {
        std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), std::mem::size_of_val(data))
    };
    let bytes: &[u8] = if bytes.is_empty() { &[0] } else { bytes };
    let slice = stream.clone_htod(bytes).map_err(cuda)?;
    let address = {
        let (address, _guard) = slice.device_ptr(stream);
        address
    };
    arrays.push(slice);
    Ok(address)
}

fn device_map(
    stream: &Arc<CudaStream>,
    arrays: &mut Vec<CudaSlice<u8>>,
    map: &FbsMapImage<'_>,
) -> Result<CMap, String> {
    Ok(CMap {
        atoms: upload(stream, arrays, map.atoms)? as *const u16,
        codes: upload(stream, arrays, map.codes)? as *const u8,
        blocks: map.atoms.len() as u32,
    })
}

impl DeviceEvaluator {
    pub fn new(ordinal: usize, model: &IntegerStackModel) -> Result<Self, String> {
        let image: FbsImage<'_> = model
            .fbs_image()
            .ok_or("the artifact is not a SpiralCore/FBS stack")?;
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let capability = context.compute_capability().map_err(cuda)?;
        let (major, minor) = capability;
        if IMAGE_TARGET != format!("sm_{major}{minor}") {
            return Err(format!(
                "cubin target {IMAGE_TARGET} differs from device capability {major}.{minor}; no PTX fallback"
            ));
        }
        context.bind_to_thread().map_err(cuda)?;
        // SAFETY: CUBIN is the admitted, assembled image of this build.
        let raw = unsafe { result::module::load_data(CUBIN.as_ptr().cast()) }.map_err(cuda)?;
        let module = Module {
            raw,
            context: Arc::clone(&context),
        };
        let name = CString::new("fbs_eval_sequences").map_err(|e| e.to_string())?;
        // SAFETY: the module is loaded and names this entry (PTX admission).
        let function = unsafe { result::module::get_function(module.raw, name) }.map_err(cuda)?;
        let trace_name = CString::new("fbs_trace_sequences").map_err(|e| e.to_string())?;
        // SAFETY: as above; the module names this entry too (PTX admission).
        let trace_function =
            unsafe { result::module::get_function(module.raw, trace_name) }.map_err(cuda)?;
        let stream = context.default_stream();
        let mut arrays = Vec::new();
        let mut layers = Vec::with_capacity(image.layers.len());
        for l in &image.layers {
            layers.push(CLayer {
                rec_gain: upload(&stream, &mut arrays, l.rec_gain)? as *const i16,
                mlp_gain: upload(&stream, &mut arrays, l.mlp_gain)? as *const i16,
                rec_in: device_map(&stream, &mut arrays, &l.rec_in)?,
                rec_gate: device_map(&stream, &mut arrays, &l.rec_gate)?,
                rec_out: device_map(&stream, &mut arrays, &l.rec_out)?,
                mlp_gate: device_map(&stream, &mut arrays, &l.mlp_gate)?,
                mlp_up: device_map(&stream, &mut arrays, &l.mlp_up)?,
                mlp_down: device_map(&stream, &mut arrays, &l.mlp_down)?,
                taps: upload(&stream, &mut arrays, l.taps)? as *const i16,
                conv_bias: upload(&stream, &mut arrays, l.conv_bias)? as *const i32,
                gate_bias: upload(&stream, &mut arrays, l.gate_bias)? as *const i32,
                decay_offset: upload(&stream, &mut arrays, l.decay_offset)? as *const i32,
            });
        }
        let shape = model.shape();
        let eps = model.rms_epsilon();
        let device_model = CModel {
            vocab: shape.vocab as u32,
            width: shape.width as u32,
            mlp: shape.mlp as u32,
            context: shape.context as u32,
            kept: image.kept_cells as u32,
            layer_count: layers.len() as u32,
            eps_mantissa: eps.mantissa,
            eps_exp: eps.exp,
            layers: upload(&stream, &mut arrays, &layers)? as *const CLayer,
            final_gain: upload(&stream, &mut arrays, image.final_gain)? as *const i16,
            writer_table: upload(&stream, &mut arrays, image.writer_table)? as *const i16,
            roots_q12: upload(&stream, &mut arrays, &image.roots_q12[..])? as *const i32,
            icosian: upload(&stream, &mut arrays, &H4_ROOT_COEFFICIENTS[..])? as *const i8,
            keep_q31: upload(&stream, &mut arrays, &FbsImage::GEOMETRIC_KEEP_Q31[..])?
                as *const u64,
        };
        stream.synchronize().map_err(cuda)?;
        Ok(Self {
            _arrays: arrays,
            work_bytes: work_bytes(&device_model),
            model: device_model,
            function,
            trace_function,
            module,
            stream,
            capability,
        })
    }

    /// Every sequence (all of one length) from a fresh session, on the GPU.
    pub fn evaluate(&self, sequences: &[Vec<u32>]) -> Result<Outputs, String> {
        let (outputs, _) = self.run(sequences, false)?;
        Ok(outputs)
    }

    /// [`Self::evaluate`] with every position's snap trace, `trace_stride` bytes
    /// each (claude-134).
    pub fn evaluate_traced(&self, sequences: &[Vec<u32>]) -> Result<(Outputs, Vec<u8>), String> {
        let (out, trace) = self.run(sequences, true)?;
        Ok((out, trace.unwrap_or_default()))
    }

    fn run(
        &self,
        sequences: &[Vec<u32>],
        traced: bool,
    ) -> Result<(Outputs, Option<Vec<u8>>), String> {
        let stride_bytes = crate::trace_stride(&self.model) as usize;
        let mut all_trace = traced.then(Vec::new);
        let len = sequences.first().map_or(0, Vec::len);
        if sequences.iter().any(|s| s.len() != len) {
            return Err("every sequence must have the same length".to_owned());
        }
        let (kept, width) = (self.model.kept as usize, self.model.width as usize);
        let mut out = Outputs {
            sequences: sequences.len(),
            len,
            kept,
            width,
            tokens: Vec::with_capacity(sequences.len() * len),
            kept_cells: Vec::with_capacity(sequences.len() * len * kept),
            queries: Vec::with_capacity(sequences.len() * len * width),
        };
        self.module.context.bind_to_thread().map_err(cuda)?;
        for chunk in sequences.chunks(CHUNK) {
            let n = chunk.len();
            let flat: Vec<u32> = chunk.iter().flatten().copied().collect();
            let tokens = self.stream.clone_htod(&flat).map_err(cuda)?;
            let mut work = self
                .stream
                .alloc_zeros::<u8>(n * self.work_bytes as usize)
                .map_err(cuda)?;
            let mut out_tokens = self.stream.alloc_zeros::<u32>(n * len).map_err(cuda)?;
            let mut out_kept = self
                .stream
                .alloc_zeros::<u32>(n * len * kept)
                .map_err(cuda)?;
            let mut out_query = self
                .stream
                .alloc_zeros::<i32>(n * len * width)
                .map_err(cuda)?;
            let mut status = self.stream.alloc_zeros::<i32>(n).map_err(cuda)?;
            // One byte when untraced: the kernel then never reads it.
            let mut out_trace = self
                .stream
                .alloc_zeros::<u8>(if traced { n * len * stride_bytes } else { 1 })
                .map_err(cuda)?;
            {
                let (tokens_ptr, _g0) = tokens.device_ptr(&self.stream);
                let (work_ptr, _g1) = work.device_ptr_mut(&self.stream);
                let (tokens_out, _g2) = out_tokens.device_ptr_mut(&self.stream);
                let (kept_out, _g3) = out_kept.device_ptr_mut(&self.stream);
                let (query_out, _g4) = out_query.device_ptr_mut(&self.stream);
                let (status_ptr, _g5) = status.device_ptr_mut(&self.stream);
                let (trace_out, _g6) = out_trace.device_ptr_mut(&self.stream);
                let mut model = self.model;
                let mut count = n as u32;
                let mut length = len as u32;
                let mut stride = self.work_bytes;
                let arg =
                    |value: &sys::CUdeviceptr| value as *const sys::CUdeviceptr as *mut c_void;
                let mut params: Vec<*mut c_void> = vec![
                    (&mut model as *mut CModel).cast(),
                    arg(&tokens_ptr),
                    (&mut count as *mut u32).cast(),
                    (&mut length as *mut u32).cast(),
                    arg(&work_ptr),
                    (&mut stride as *mut u64).cast(),
                    arg(&tokens_out),
                    arg(&kept_out),
                    arg(&query_out),
                ];
                if traced {
                    params.push(arg(&trace_out));
                }
                params.push(arg(&status_ptr));
                let function = if traced {
                    self.trace_function
                } else {
                    self.function
                };
                let threads = THREADS.min(n as u32).max(1);
                let grid = ((n as u32).div_ceil(threads), 1, 1);
                // SAFETY: the parameters point at values of the types the
                // chosen entry takes, in its order (ten for
                // fbs_eval_sequences, eleven with `out_trace` for
                // fbs_trace_sequences), and outlive the launch. `model` holds
                // device addresses of arrays kept alive by `self`, each the
                // length its FBS image field has. Each thread touches only its
                // own sequence's tokens, work region (`stride` bytes), outputs
                // and trace (`len * trace_stride` bytes).
                unsafe {
                    result::launch_kernel(
                        function,
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
            let codes = self.stream.clone_dtoh(&status).map_err(cuda)?;
            if let Some((s, code)) = codes.iter().enumerate().find(|(_, &c)| c != 0) {
                return Err(format!("sequence {s} of a chunk failed with code {code}"));
            }
            out.tokens
                .extend(self.stream.clone_dtoh(&out_tokens).map_err(cuda)?);
            out.kept_cells
                .extend(self.stream.clone_dtoh(&out_kept).map_err(cuda)?);
            out.queries
                .extend(self.stream.clone_dtoh(&out_query).map_err(cuda)?);
            if let Some(trace) = all_trace.as_mut() {
                trace.extend(self.stream.clone_dtoh(&out_trace).map_err(cuda)?);
            }
        }
        Ok((out, all_trace))
    }
}
