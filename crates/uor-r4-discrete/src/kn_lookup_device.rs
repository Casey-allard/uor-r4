//! CUDA driver of the raw count lookups (claude-433). Loads the admitted
//! `kn-lookup.cubin` (no PTX fallback, no NVRTC), uploads one [`RawTables`]
//! once, and looks up query streams one thread per target. Its counts and
//! totals equal [`RawTables::lookups`] word for word.

use crate::kn_ceiling::RawTables;
use crate::ngram_count::{check_tokens, NGRAM_BOS};
use crate::{Error, Result};
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DeviceRepr, LaunchConfig, Ptx, PushKernelArg,
    ValidAsZeroBits,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE: &str = include_str!("kn_lookup.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/kn-lookup.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kn-lookup.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

/// The kernel inventory admitted by `build.rs`.
pub const KN_LOOKUP_ENTRIES: [&str; 1] = ["kn_lookup_targets"];
const THREADS: u32 = 256;

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Source, PTX and cubin SHA-256 of the embedded image.
pub fn compiled_kn_lookup_image() -> [String; 3] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
    ]
}

fn offset(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid("raw tables exceed 2^32 records"))
}

/// One CUDA context holding the resident raw tables of orders 0..=N.
pub struct KnLookup {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    function: CudaFunction,
    highest: u32,
    ctx_off: CudaSlice<u32>,
    ctx_keys: CudaSlice<u64>,
    ctx_total: CudaSlice<u64>,
    tri_off: CudaSlice<u32>,
    tri_keys: CudaSlice<u64>,
    tri_next: CudaSlice<u16>,
    tri_count: CudaSlice<u32>,
}

impl KnLookup {
    /// Check the cubin target against the device and upload every order's
    /// contexts and triples as flat arrays with per-order offsets.
    pub fn new(ordinal: usize, tables: &RawTables) -> Result<Self> {
        let highest = tables.highest();
        let mut ctx_off = vec![0u32];
        let mut tri_off = vec![0u32];
        let (mut ctx_keys, mut ctx_total) = (Vec::new(), Vec::new());
        let (mut tri_keys, mut tri_next, mut tri_count) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..=highest {
            let (triples, contexts) = tables
                .order_tables(k)
                .ok_or_else(|| invalid(format!("raw tables lack order {k}")))?;
            for context in contexts {
                ctx_keys.push(context.key as u64);
                ctx_keys.push((context.key >> 64) as u64);
                ctx_total.push(context.total);
            }
            for triple in triples {
                tri_keys.push(triple.key as u64);
                tri_keys.push((triple.key >> 64) as u64);
                tri_next.push(triple.next);
                tri_count.push(triple.count);
            }
            ctx_off.push(offset(ctx_total.len())?);
            tri_off.push(offset(tri_next.len())?);
        }
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (major, minor) = context.compute_capability().map_err(cuda)?;
        if TARGET.trim() != format!("sm_{major}{minor}") {
            return Err(invalid(format!(
                "kn-lookup cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let function = module.load_function(KN_LOOKUP_ENTRIES[0]).map_err(cuda)?;
        Ok(Self {
            highest,
            ctx_off: upload_padded(&stream, &ctx_off)?,
            ctx_keys: upload_padded(&stream, &ctx_keys)?,
            ctx_total: upload_padded(&stream, &ctx_total)?,
            tri_off: upload_padded(&stream, &tri_off)?,
            tri_keys: upload_padded(&stream, &tri_keys)?,
            tri_next: upload_padded(&stream, &tri_next)?,
            tri_count: upload_padded(&stream, &tri_count)?,
            function,
            stream,
            _context: context,
        })
    }

    pub fn highest(&self) -> u32 {
        self.highest
    }

    /// For every non-BOS target of `query` in stream order, highest+1 pairs
    /// (count a, context total), flattened target-major at t*(highest+1)+k,
    /// as [`RawTables::lookups`] returns them.
    pub fn lookups(&self, query: &[u16]) -> Result<(Vec<u32>, Vec<u64>)> {
        check_tokens(query)?;
        let positions = (0..query.len())
            .filter(|&j| query[j] != NGRAM_BOS)
            .map(offset)
            .collect::<Result<Vec<u32>>>()?;
        if positions.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }
        let count = offset(positions.len())?;
        let width = self.highest as usize + 1;
        let words = positions.len() * width;
        let tokens = self.stream.clone_htod(query).map_err(cuda)?;
        let device_positions = self.stream.clone_htod(&positions).map_err(cuda)?;
        let mut counts = self.zeros::<u32>(words)?;
        let mut totals = self.zeros::<u64>(words)?;
        let mut launch = self.stream.launch_builder(&self.function);
        launch
            .arg(&tokens)
            .arg(&device_positions)
            .arg(&count)
            .arg(&self.highest)
            .arg(&self.ctx_off)
            .arg(&self.ctx_keys)
            .arg(&self.ctx_total)
            .arg(&self.tri_off)
            .arg(&self.tri_keys)
            .arg(&self.tri_next)
            .arg(&self.tri_count)
            .arg(&mut counts)
            .arg(&mut totals);
        let config = LaunchConfig {
            grid_dim: (count.div_ceil(THREADS), 1, 1),
            block_dim: (THREADS, 1, 1),
            shared_mem_bytes: 0,
        };
        // SAFETY: one thread per position i < count; every position j < query.len()
        // and the kernel reads tokens[j - s] only for s <= min(N, j); offsets hold
        // N + 2 words and bound every table read; counts and totals hold
        // count * (N + 1) words and thread i writes only i*(N+1)..(i+1)*(N+1).
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let counts = self.stream.clone_dtoh(&counts).map_err(cuda)?;
        let totals = self.stream.clone_dtoh(&totals).map_err(cuda)?;
        Ok((counts, totals))
    }

    fn zeros<T: DeviceRepr + ValidAsZeroBits>(&self, len: usize) -> Result<CudaSlice<T>> {
        self.stream.alloc_zeros::<T>(len.max(1)).map_err(cuda)
    }
}

/// Upload `words`, or one zero word when empty: a device buffer is never empty.
fn upload_padded<T: DeviceRepr + ValidAsZeroBits + Clone>(
    stream: &Arc<CudaStream>,
    words: &[T],
) -> Result<CudaSlice<T>> {
    if words.is_empty() {
        return stream.alloc_zeros::<T>(1).map_err(cuda);
    }
    stream.clone_htod(words).map_err(cuda)
}
