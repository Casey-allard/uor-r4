//! Bounded local timing for parallel RGBA decomposition.
//!
//! ```text
//! RAYON_NUM_THREADS=16 cargo run --release -p uor-r4-vision \
//!   --example decompose-bench -- 2000
//! ```

use std::time::Instant;

use uor_r4_vision::{decompose_rgba, Grid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repeats = std::env::args()
        .nth(1)
        .map_or(Ok(2_000), |value| value.parse::<u32>())?;
    if repeats == 0 {
        return Err("repeats must be positive".into());
    }

    let mut rgba = vec![0u8; 256 * 256 * 4];
    for y in 0..256u32 {
        for x in 0..256u32 {
            let at = ((y * 256 + x) * 4) as usize;
            rgba[at] = x as u8;
            rgba[at + 1] = y as u8;
            rgba[at + 2] = ((x * 13 + y * 7) & 0xff) as u8;
            rgba[at + 3] = 255;
        }
    }

    let grid = Grid::new(32, 32);
    let warm = decompose_rgba(256, 256, &rgba, grid)?;
    let canonical_digest = blake3::hash(&warm.canonical_bytes());
    let started = Instant::now();
    for _ in 0..repeats {
        let out = decompose_rgba(256, 256, &rgba, grid)?;
        if out.source_hash() != warm.source_hash() {
            return Err("source digest changed between runs".into());
        }
    }
    let elapsed = started.elapsed();

    #[cfg(feature = "parallel")]
    let workers = rayon::current_num_threads();
    #[cfg(not(feature = "parallel"))]
    let workers = 1;

    println!("workers={workers}");
    println!("repeats={repeats}");
    println!("patches={}", warm.patches().len());
    println!("canonical_blake3={}", canonical_digest.to_hex());
    println!("elapsed_us={}", elapsed.as_micros());
    println!(
        "us_per_decomposition={:.3}",
        elapsed.as_secs_f64() * 1e6 / f64::from(repeats)
    );
    Ok(())
}
