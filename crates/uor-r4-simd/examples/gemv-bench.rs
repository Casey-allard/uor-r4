//! Time the table GEMV kernels on matrices of SmolLM2-135M's shapes.
//!
//! ```text
//! cargo run --release -p uor-r4-simd --example gemv-bench [repeats]
//! ```

use std::time::Instant;

use uor_r4_simd::{Backend, Interleaved, Tables, ROWS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repeats: usize = std::env::args().nth(1).map_or(Ok(20), |v| v.parse())?;
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as u8
    };
    for (name, rows, cols) in [
        ("q/o 576x576", 576, 576),
        ("gate/up 1536x576", 1536, 576),
        ("down 576x1536", 576, 1536),
        ("head 49152x576", 49152, 576),
    ] {
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| next()).collect();
        let scales: Vec<u8> = (0..rows * cols / 32).map(|_| next() & 0x3F).collect();
        let m = Interleaved::new(rows, cols, &nibbles, &scales)?;
        let x: Vec<i16> = (0..cols).map(|_| i16::from(next() as i8) << 7).collect();
        let mut tables = Tables::default();
        let mut out = vec![0i64; m.blocks() * ROWS];
        for backend in [Backend::Portable, Backend::Avx2, Backend::Neon] {
            if !backend.available() {
                continue;
            }
            m.gemv_blocks_with(
                backend,
                {
                    tables.build(&x)?;
                    &tables
                },
                0,
                &mut out,
            )?;
            let started = Instant::now();
            for _ in 0..repeats {
                tables.build(&x)?;
                m.gemv_blocks_with(backend, &tables, 0, &mut out)?;
            }
            let seconds = started.elapsed().as_secs_f64() / repeats as f64;
            println!(
                "{name:>18} {:>8}: {:8.1} us  {:.3} ns/weight  {:.2} GB/s of packed weights",
                backend.name(),
                seconds * 1e6,
                seconds * 1e9 / (rows * cols) as f64,
                (rows * cols) as f64 / 2.0 / seconds / 1e9
            );
        }
    }
    Ok(())
}
