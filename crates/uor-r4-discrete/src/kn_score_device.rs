//! CUDA driver of the integer modified Kneser–Ney scorer (claude-421). Loads
//! the admitted `kn-score.cubin` (no PTX fallback, no NVRTC), uploads one
//! [`KnTables`] once, and scores query streams one thread per target. Its
//! masses equal [`KnTables::scored_masses`] bit for bit.

use crate::kn_score::KnTables;
use crate::ngram_count::{check_tokens, NGRAM_BOS};
use crate::{Error, Result};
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, DeviceRepr, LaunchConfig, Ptx, PushKernelArg,
    ValidAsZeroBits,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE: &str = include_str!("kn_score.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/kn-score.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kn-score.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

/// The kernel inventory admitted by `build.rs`.
pub const KN_SCORE_ENTRIES: [&str; 1] = ["kn_score_targets"];
const THREADS: u32 = 256;

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Source, PTX and cubin SHA-256 of the embedded image.
pub fn compiled_kn_score_image() -> [String; 3] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
    ]
}

fn offset(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid("KN tables exceed 2^32 records"))
}

/// One CUDA context holding the resident tables of orders 0..=N.
pub struct KnScorer {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    function: CudaFunction,
    highest: u32,
    ctx_off: CudaSlice<u32>,
    ctx_keys: CudaSlice<u64>,
    ctx_total: CudaSlice<u64>,
    ctx_counts: CudaSlice<u32>,
    tri_off: CudaSlice<u32>,
    tri_keys: CudaSlice<u64>,
    tri_next: CudaSlice<u16>,
    tri_count: CudaSlice<u32>,
    discounts: CudaSlice<u64>,
}

impl KnScorer {
    /// Check the cubin target against the device and upload every order's
    /// discounts, contexts and triples as flat arrays with per-order offsets.
    pub fn new(ordinal: usize, tables: &KnTables) -> Result<Self> {
        let highest = tables.highest();
        let mut ctx_off = vec![0u32];
        let mut tri_off = vec![0u32];
        let (mut ctx_keys, mut ctx_total, mut ctx_counts) = (Vec::new(), Vec::new(), Vec::new());
        let (mut tri_keys, mut tri_next, mut tri_count) = (Vec::new(), Vec::new(), Vec::new());
        let mut discounts = Vec::new();
        for k in 0..=highest {
            let (order_discounts, triples, contexts) = tables
                .order_tables(k)
                .ok_or_else(|| invalid(format!("KN tables lack order {k}")))?;
            discounts.extend_from_slice(&order_discounts.dq);
            for context in contexts {
                ctx_keys.push(context.key as u64);
                ctx_keys.push((context.key >> 64) as u64);
                ctx_total.push(context.total);
                ctx_counts.extend_from_slice(&[context.n1, context.n2, context.n3_plus]);
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
                "kn-score cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let function = module.load_function(KN_SCORE_ENTRIES[0]).map_err(cuda)?;
        Ok(Self {
            highest,
            ctx_off: upload_padded(&stream, &ctx_off)?,
            ctx_keys: upload_padded(&stream, &ctx_keys)?,
            ctx_total: upload_padded(&stream, &ctx_total)?,
            ctx_counts: upload_padded(&stream, &ctx_counts)?,
            tri_off: upload_padded(&stream, &tri_off)?,
            tri_keys: upload_padded(&stream, &tri_keys)?,
            tri_next: upload_padded(&stream, &tri_next)?,
            tri_count: upload_padded(&stream, &tri_count)?,
            discounts: upload_padded(&stream, &discounts)?,
            function,
            stream,
            _context: context,
        })
    }

    pub fn highest(&self) -> u32 {
        self.highest
    }

    /// The Q48 mass of every non-BOS target of `query`, in stream order.
    /// Refuses a mass of 0 or of 2^64 or more, naming the first such target.
    pub fn score(&self, query: &[u16]) -> Result<Vec<u64>> {
        check_tokens(query)?;
        let positions = (0..query.len())
            .filter(|&j| query[j] != NGRAM_BOS)
            .map(offset)
            .collect::<Result<Vec<u32>>>()?;
        if positions.is_empty() {
            return Ok(Vec::new());
        }
        let count = offset(positions.len())?;
        let tokens = self.stream.clone_htod(query).map_err(cuda)?;
        let device_positions = self.stream.clone_htod(&positions).map_err(cuda)?;
        let mut masses = self.zeros::<u64>(positions.len())?;
        let mut flags = self.zeros::<u32>(positions.len())?;
        let mut launch = self.stream.launch_builder(&self.function);
        launch
            .arg(&tokens)
            .arg(&device_positions)
            .arg(&count)
            .arg(&self.highest)
            .arg(&self.ctx_off)
            .arg(&self.ctx_keys)
            .arg(&self.ctx_total)
            .arg(&self.ctx_counts)
            .arg(&self.tri_off)
            .arg(&self.tri_keys)
            .arg(&self.tri_next)
            .arg(&self.tri_count)
            .arg(&self.discounts)
            .arg(&mut masses)
            .arg(&mut flags);
        let config = LaunchConfig {
            grid_dim: (count.div_ceil(THREADS), 1, 1),
            block_dim: (THREADS, 1, 1),
            shared_mem_bytes: 0,
        };
        // SAFETY: one thread per position i < count; every position j < query.len()
        // and the kernel reads tokens[j - s] only for s <= min(N, j); offsets hold
        // N + 2 words and bound every table read; masses and flags hold count words.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let masses = self.stream.clone_dtoh(&masses).map_err(cuda)?;
        let flags = self.stream.clone_dtoh(&flags).map_err(cuda)?;
        if let Some(i) = flags.iter().position(|&flag| flag != 0) {
            return Err(invalid(format!(
                "target {} has a KN mass of 2^64 or more",
                positions[i]
            )));
        }
        Ok(masses)
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
