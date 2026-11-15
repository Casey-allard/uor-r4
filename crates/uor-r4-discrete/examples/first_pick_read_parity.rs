//! claude-389, control 2: the CPU first-pick read against its CUDA twin, bit for bit, on a
//! trained state's real converted inputs (a `Q31ReadDump` that `recall-payload-probe
//! --native-replay --native-dump PATH` writes). Prints one JSON line; exits non-zero on any
//! differing pick or out word.
//! Run with:
//! `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 cargo run --release -p
//! uor-r4-discrete --features cuda --example first_pick_read_parity -- DUMP`

use std::time::Instant;
use uor_r4_discrete::native_vector::Q31ReadDump;
use uor_r4_discrete::native_vector_device::NativeVectorEngine;

fn main() -> uor_r4_discrete::Result<()> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        uor_r4_discrete::Error::Invalid("usage: first_pick_read_parity DUMP".into())
    })?;
    let bytes = std::fs::read(&path)
        .map_err(|e| uor_r4_discrete::Error::Invalid(format!("reading {path}: {e}")))?;
    let dump = Q31ReadDump::from_bytes(&bytes)?;
    let (read, batch, time) = (&dump.read, dump.batch, dump.time);
    let width = read.width();
    let started = Instant::now();
    let cpu = read.apply(&dump.stream, &dump.ids, &dump.table, batch, time)?;
    let cpu_ns = started.elapsed().as_nanos();
    let engine = NativeVectorEngine::new(0)?;
    let started = Instant::now();
    let cuda = engine.first_pick_read(read, &dump.stream, &dump.ids, &dump.table, batch, time)?;
    let cuda_ns = started.elapsed().as_nanos();
    let picks_differ = cpu
        .picks
        .iter()
        .zip(&cuda.picks)
        .filter(|(a, b)| a != b)
        .count();
    let words_differ = cpu
        .out
        .iter()
        .zip(&cuda.out)
        .filter(|(a, b)| a != b)
        .count();
    let same = picks_differ == 0
        && words_differ == 0
        && cpu.picks.len() == cuda.picks.len()
        && cpu.out.len() == cuda.out.len();
    println!(
        "{}",
        serde_json::json!({
            "dump": path,
            "batch": batch,
            "time": time,
            "width": width,
            "gain_shift": read.gain_shift(),
            "rows": cpu.picks.len(),
            "out_words": cpu.out.len(),
            "picks_differ": picks_differ,
            "out_words_differ": words_differ,
            "bit_identical": same,
            "cpu_ns": cpu_ns,
            "cuda_ns": cuda_ns,
        })
    );
    if !same {
        return Err(uor_r4_discrete::Error::Invalid(format!(
            "CPU and CUDA differ: {picks_differ} picks, {words_differ} out words"
        )));
    }
    Ok(())
}
