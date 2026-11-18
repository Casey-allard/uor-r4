//! Wick AOT kernels for the finite equal-byte address diagnostic.
use crate::equal_count::{image, row_count, Key, MAX_TOKENS, VOCAB};
use crate::{Error, Result};
use cudarc::driver::{CudaContext, LaunchConfig, Ptx, PushKernelArg};
use sha2::{Digest, Sha256};

const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/equal-count.cubin"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
fn cuda(e: impl std::fmt::Display) -> Error {
    Error::Cuda(e.to_string())
}
fn launch(n: u32) -> LaunchConfig {
    LaunchConfig {
        block_dim: (128, 1, 1),
        grid_dim: (n.div_ceil(128), 1, 1),
        shared_mem_bytes: 0,
    }
}

pub fn kernel_hash() -> String {
    format!("{:x}", Sha256::digest(CUBIN))
}

pub fn compare_arm(
    train: &[u16],
    query: &[u16],
    roots: &[u32],
    key: Key,
) -> Result<(Vec<u8>, Vec<u64>)> {
    for stream in [train, query] {
        if stream.is_empty()
            || stream.len() > MAX_TOKENS
            || stream.iter().any(|&t| usize::from(t) >= VOCAB)
        {
            return Err(invalid("count probe input outside bounded V4096 stream"));
        }
    }
    let targets = train.iter().filter(|&&t| t != 0).count() as u32;
    if targets == 0 {
        return Err(invalid("empty TRAIN target set"));
    }
    let (arm, k) = key.words()?;
    let meta = key.metadata(roots)?;
    let rows = row_count(meta.len())? as u32;
    let ctx = CudaContext::new(0).map_err(cuda)?;
    let (major, minor) = ctx.compute_capability().map_err(cuda)?;
    if TARGET.trim() != format!("sm_{major}{minor}") {
        return Err(invalid("count probe cubin and device differ"));
    }
    let stream = ctx.default_stream();
    let module = ctx
        .load_module(Ptx::from_binary(CUBIN.to_vec()))
        .map_err(cuda)?;
    let count = module.load_function("equal_count_train").map_err(cuda)?;
    let freeze = module.load_function("equal_count_freeze").map_err(cuda)?;
    let score = module.load_function("equal_count_score").map_err(cuda)?;
    let put_meta = |m: &[u8]| {
        stream
            .clone_htod(if m.is_empty() { &[0u8][..] } else { m })
            .map_err(cuda)
    };
    let train_gpu = stream.clone_htod(train).map_err(cuda)?;
    let meta_gpu = put_meta(&meta)?;
    let mut hist = stream
        .alloc_zeros::<u32>(rows as usize * VOCAB)
        .map_err(cuda)?;
    let mut uni = stream.alloc_zeros::<u32>(VOCAB).map_err(cuda)?;
    let mut table = stream.alloc_zeros::<u32>(rows as usize * 8).map_err(cuda)?;
    let n = train.len() as u32;
    // SAFETY: checked V4096 inputs, finite arm metadata, hist[rows][4096] and uni[4096].
    unsafe {
        stream
            .launch_builder(&count)
            .arg(&train_gpu)
            .arg(&n)
            .arg(&meta_gpu)
            .arg(&arm)
            .arg(&k)
            .arg(&rows)
            .arg(&mut hist)
            .arg(&mut uni)
            .launch(launch(n))
    }
    .map_err(cuda)?;
    // SAFETY: one writer per row, table[rows][4][2].
    unsafe {
        stream
            .launch_builder(&freeze)
            .arg(&hist)
            .arg(&rows)
            .arg(&mut table)
            .launch(launch(rows))
    }
    .map_err(cuda)?;
    let bytes = image(
        key,
        &meta,
        &stream.clone_dtoh(&uni).map_err(cuda)?,
        &stream.clone_dtoh(&table).map_err(cuda)?,
        targets,
    )?;
    // Score through the serialized bytes; no writable histogram or construction map
    // is passed to the scorer. Direct row indexing has no external storage.
    let end = 32 + meta.len();
    let freeze_meta = put_meta(&bytes[32..end])?;
    let words: Vec<u32> = bytes[end..]
        .chunks_exact(4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .collect();
    let freeze_uni = stream.clone_htod(&words[..VOCAB]).map_err(cuda)?;
    let freeze_table = stream.clone_htod(&words[VOCAB..]).map_err(cuda)?;
    let nq = query.len() as u32;
    let query_gpu = stream.clone_htod(query).map_err(cuda)?;
    let mut masses = stream.alloc_zeros::<u64>(query.len()).map_err(cuda)?;
    // SAFETY: validated input ids, serialized rows have their checked exact shape;
    // one writer per query index, and the scorer reads the prefix only back to BOS.
    unsafe {
        stream
            .launch_builder(&score)
            .arg(&query_gpu)
            .arg(&nq)
            .arg(&freeze_meta)
            .arg(&arm)
            .arg(&k)
            .arg(&rows)
            .arg(&freeze_table)
            .arg(&freeze_uni)
            .arg(&targets)
            .arg(&mut masses)
            .launch(launch(nq))
    }
    .map_err(cuda)?;
    let masses = stream.clone_dtoh(&masses).map_err(cuda)?;
    if masses
        .iter()
        .zip(query)
        .any(|(&p, &t)| (p == 0) != (t == 0) || p > 1u64 << 48)
    {
        return Err(invalid("GPU returned invalid target mass"));
    }
    Ok((bytes, masses))
}
