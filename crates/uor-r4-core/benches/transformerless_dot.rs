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

    // Isolated SIMD scan (#330 ≥3× criterion). Timed against the same total
    // work as the scalar loop above: one full K×D scan per stage. The scalar
    // reference is `dot_score_plain` — deliberately NOT the scalar fallback in
    // `Runtime::dot_argmax`, which routes every table entry through the
    // op-census kernel and would inflate the baseline. The SIMD side also
    // performs the argmax compare that the scalar loop omits, so the reported
    // ratio is conservative.
    #[cfg(feature = "bench-internals")]
    let simd_scan_report = {
        use uor_r4_core::transformerless::simd::bench::{DotLayout, DotScan};

        let scan = DotScan::from_packed(&artifact.dot_cb)
            .ok_or_else(|| "cannot build SIMD dot tables from artifact".to_owned())?;
        let layout = if (0..scan.stage_count()).all(|s| scan.layout(s) == DotLayout::Compact) {
            "compact"
        } else if (0..scan.stage_count()).all(|s| scan.layout(s) == DotLayout::TwoTerm) {
            "two_term"
        } else {
            "mixed"
        };

        let legacy = DotScan::from_packed_forced_two_term(&artifact.dot_cb)
            .ok_or_else(|| "cannot build legacy dot tables from artifact".to_owned())?;

        // Both representations decode the same packed ABI, so they must agree
        // class-for-class. Checked before timing: a layout that disagreed
        // would make the speedup meaningless.
        for stage in 0..scan.stage_count() {
            let compact_class = scan.argmax(stage, &work);
            let legacy_class = legacy.argmax(stage, &work);
            if compact_class != legacy_class {
                return Err(format!(
                    "layout disagreement at stage {stage}: \
                     compact={compact_class} legacy={legacy_class}"
                ));
            }
        }

        let mut simd_checksum = 0u64;
        for _ in 0..10 {
            for stage in 0..scan.stage_count() {
                simd_checksum = simd_checksum.wrapping_add(u64::from(scan.argmax(stage, &work)));
            }
        }
        let simd_start = Instant::now();
        for _ in 0..iterations {
            for stage in 0..scan.stage_count() {
                simd_checksum =
                    simd_checksum.wrapping_add(u64::from(scan.argmax(stage, black_box(&work))));
            }
        }
        let simd_elapsed = simd_start.elapsed();

        let mut legacy_checksum = 0u64;
        for _ in 0..10 {
            for stage in 0..legacy.stage_count() {
                legacy_checksum =
                    legacy_checksum.wrapping_add(u64::from(legacy.argmax(stage, &work)));
            }
        }
        let legacy_start = Instant::now();
        for _ in 0..iterations {
            for stage in 0..legacy.stage_count() {
                legacy_checksum =
                    legacy_checksum.wrapping_add(u64::from(legacy.argmax(stage, black_box(&work))));
            }
        }
        let legacy_elapsed = legacy_start.elapsed();

        (layout, simd_elapsed, simd_checksum, legacy_elapsed)
    };

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
    #[cfg(feature = "bench-internals")]
    {
        let (layout, simd_elapsed, simd_checksum, legacy_elapsed) = simd_scan_report;
        let simd_ns = nanos_per_operation(simd_elapsed, iterations);
        let legacy_ns = nanos_per_operation(legacy_elapsed, iterations);
        println!("simd_dot_layout={layout}");
        println!("simd_dot_ns_per_scan={simd_ns:.1}");
        println!(
            "simd_dot_ns_per_row={:.1}",
            nanos_per_operation(simd_elapsed, iterations * rows_per_scan)
        );
        println!("legacy_two_term_ns_per_scan={legacy_ns:.1}");
        println!(
            "isolated_speedup={:.2}",
            nanos_per_operation(scalar_elapsed, iterations) / simd_ns
        );
        println!("layout_speedup={:.2}", legacy_ns / simd_ns);
        println!("simd_checksum={simd_checksum}");
    }
    println!("scalar_checksum={scalar_checksum}");
    println!("runtime_checksum={runtime_checksum:?}");
    Ok(())
}
