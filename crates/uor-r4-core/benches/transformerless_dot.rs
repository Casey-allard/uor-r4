//! Wall-clock baseline for the transformerless shift-add dot path.
//!
//! This intentionally uses only the standard library. It measures the scalar
//! reference scan and the complete allocation-free assignment step against a
//! TLA6 artifact, which is the first data point required by issue #330 before
//! adding an architecture-specific adapter.

use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_core::transformerless::compiler::{self, D, K, STAGES, WINDOW};
use uor_r4_core::transformerless::runtime::{self, Runtime};

const DEFAULT_ARTIFACT: &str = "tests/fixtures/tless_artifacts.bin";
const DEFAULT_ITERATIONS: usize = 100;

fn parse_iterations(value: Option<&String>) -> Result<usize, String> {
    let iterations = value
        .map(|raw| {
            raw.parse::<usize>()
                .map_err(|_| format!("invalid iteration count: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_ITERATIONS);
    if iterations == 0 {
        return Err("iteration count must be greater than zero".to_owned());
    }
    Ok(iterations)
}

fn deterministic_work() -> [i64; D] {
    let mut work = [0i64; D];
    let mut value = -143i64;
    for slot in &mut work {
        *slot = value;
        value = value.wrapping_add(97);
        if value > 143 {
            value = -143;
        }
    }
    work
}

fn deterministic_window(vocab: usize) -> [u32; WINDOW] {
    let mut window = [0u32; WINDOW];
    let modulus = vocab.max(1) as u32;
    let mut value = 17u32;
    for token in &mut window {
        *token = value % modulus;
        value = value.wrapping_add(7919);
    }
    window
}

fn nanos_per_operation(elapsed: std::time::Duration, operations: usize) -> f64 {
    elapsed.as_secs_f64() * 1e9 / operations as f64
}

fn read_artifact(path: &str) -> Result<(PathBuf, Vec<u8>), String> {
    let requested = Path::new(path);
    let mut candidates = vec![requested.to_owned()];
    if requested.is_relative() {
        candidates.push(Path::new("../..").join(requested));
    }
    for candidate in candidates {
        match fs::read(&candidate) {
            Ok(bytes) => return Ok((candidate, bytes)),
            Err(_) => continue,
        }
    }
    Err(format!("cannot read {path}"))
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1).filter(|arg| arg != "--bench");
    let artifact_path = args.next().unwrap_or_else(|| DEFAULT_ARTIFACT.to_owned());
    let iterations = parse_iterations(args.next().as_ref())?;
    if args.next().is_some() {
        return Err("usage: transformerless_dot [ARTIFACT] [ITERATIONS]".to_owned());
    }

    let (resolved_path, bytes) = read_artifact(&artifact_path)?;
    let artifact = compiler::parse_artifacts(&bytes).ok_or_else(|| {
        format!(
            "invalid transformerless artifact: {}",
            resolved_path.display()
        )
    })?;
    if artifact.dot_cb.is_empty() {
        return Err(format!(
            "artifact {} has no TLA6 dot tables; use a TLA6 fixture",
            resolved_path.display()
        ));
    }
    let rows_per_scan = artifact.dot_cb.len() * K;
    if rows_per_scan != STAGES * K {
        return Err(format!(
            "unexpected dot table shape: {} stages, expected {STAGES}",
            artifact.dot_cb.len()
        ));
    }

    let work = deterministic_work();
    let vocab = artifact.token_codes.len() / STAGES;
    let window = deterministic_window(vocab);

    let mut scalar_checksum = 0i64;
    for _ in 0..10 {
        for table in &artifact.dot_cb {
            for row in table.chunks_exact(D) {
                scalar_checksum =
                    scalar_checksum.wrapping_add(runtime::dot_score_plain(row, &work));
            }
        }
    }
    let scalar_start = Instant::now();
    for _ in 0..iterations {
        for table in &artifact.dot_cb {
            for row in table.chunks_exact(D) {
                scalar_checksum =
                    scalar_checksum.wrapping_add(runtime::dot_score_plain(row, &work));
            }
        }
    }
    let scalar_elapsed = scalar_start.elapsed();

    let mut runtime = Runtime::new(&artifact);
    let mut runtime_checksum = [0u8; STAGES];
    for _ in 0..10 {
        runtime_checksum = runtime.assign_window(&window);
    }
    let runtime_start = Instant::now();
    for _ in 0..iterations {
        runtime_checksum = runtime.assign_window(black_box(&window));
    }
    let runtime_elapsed = runtime_start.elapsed();

    println!("artifact={}", resolved_path.display());
    println!("format=TLA6");
    println!("iterations={iterations}");
    println!("dot_rows_per_scan={rows_per_scan}");
    println!(
        "scalar_dot_ns_per_scan={:.1}",
        nanos_per_operation(scalar_elapsed, iterations)
    );
    println!(
        "scalar_dot_ns_per_row={:.1}",
        nanos_per_operation(scalar_elapsed, iterations * rows_per_scan)
    );
    println!(
        "runtime_assign_ns_per_token={:.1}",
        nanos_per_operation(runtime_elapsed, iterations)
    );
    println!(
        "runtime_assign_tokens_per_second={:.1}",
        iterations as f64 / runtime_elapsed.as_secs_f64()
    );
    println!("scalar_checksum={scalar_checksum}");
    println!("runtime_checksum={runtime_checksum:?}");
    Ok(())
}
