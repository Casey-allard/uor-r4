//! claude-406 and claude-407: how fast the discrete model's CUDA scoring runs on
//! real D87 student batches of 64 to 4,096 rows, baseline `propose` against
//! rows-K `score_rows_k`, with rows-K loss words checked against the baseline's
//! and across block sizes. Scoring only: no choose, no apply, no epoch change.
//! `#[ignore]`: it needs a GPU and `UOR_STUDENT_STREAM` (a `student-ingest`
//! output folder).

use crate::cpu_oracle_parity::{
    device_with, student_batch_rows, windows, STUDENT_SEED, STUDENT_SPEC,
};
use crate::device::CoordinateScore;
use crate::{Parameter, ReadSpec};
use std::time::Instant;

const WARMUP: usize = 1;
const TIMED: usize = 5;
const BLOCKS: [usize; 5] = [1, 8, 16, 32, 64];
const ROW_COUNTS: [u32; 4] = [64, 256, 1024, 4096];
/// Coordinates moved by the planted target shift; every one must be caught.
const PLANTED: usize = 16;

/// Baseline coordinates per kind at `rows` rows. One baseline call takes about
/// 4.7 s at 64 rows (claude-406) and grows with rows, so the subset shrinks.
fn baseline_count(rows: u32) -> usize {
    match rows {
        0..=256 => 4,
        257..=1024 => 2,
        _ => 1,
    }
}

fn median_us(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

/// Integer per-second rate of `count` items over `us` microseconds.
fn per_second(count: usize, us: u128) -> u128 {
    count as u128 * 1_000_000 / us.max(1)
}

/// The one parity predicate: status 0 and the 16 loss words equal.
fn agrees(score: &CoordinateScore, want: &[u64]) -> bool {
    score.status == 0 && score.losses == want
}

fn gpu_apps(when: &str) {
    match std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,process_name,used_memory",
            "--format=csv,noheader",
        ])
        .output()
    {
        Ok(out) => eprintln!(
            "nvidia-smi compute apps {when}: [{}]",
            String::from_utf8_lossy(&out.stdout).trim()
        ),
        Err(e) => eprintln!("nvidia-smi {when}: not run ({e})"),
    }
}

#[test]
#[ignore = "needs a GPU and UOR_STUDENT_STREAM (claude-406, claude-407)"]
fn student_batch_scoring_speed_baseline_against_rows_k() {
    gpu_apps("before");
    let device = device_with(STUDENT_SPEC, ReadSpec::Route, STUDENT_SEED);
    let epoch = device.epoch().expect("epoch");
    let scoring = device.rowsk_scoring().expect("rows-K scoring");
    let route_count = device.snapshot().expect("snapshot").routes.len().min(64) as u32;
    assert!(route_count > 0, "the Route model has no routes");
    let route: Vec<Parameter> = (0..route_count).map(Parameter::Route).collect();

    let mut mismatches = Vec::new();
    for row_count in ROW_COUNTS {
        let student = student_batch_rows(row_count);
        let rows = &student.rows;
        assert_eq!(rows.len(), row_count as usize);
        let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
        eprintln!("== {row_count} rows: {}", student.pins);
        let clock = Instant::now();
        let batch = device.carry_windows(&windows(rows)).expect("carry");
        eprintln!(
            "{row_count} rows: carry {} us (outside every timer below)",
            clock.elapsed().as_micros()
        );

        // Fixed before any timing, never chosen by loss.
        let blocks = STUDENT_SPEC.blocks;
        let distinct: std::collections::BTreeSet<u32> = targets.iter().copied().collect();
        let vector: Vec<Parameter> = distinct
            .iter()
            .flat_map(|&t| (0..blocks).map(move |b| Parameter::VectorBlock(t * blocks + b)))
            .take(64)
            .collect();
        assert_eq!(
            vector.len(),
            64,
            "the batch names fewer than 16 target tokens"
        );

        let mut vector_reference = Vec::new();
        for (kind, coords) in [("VectorBlock", &vector), ("Route", &route)] {
            // Arm A: the baseline on a fixed subset, one coordinate per launch,
            // one pass. The first call is the warm-up and is reported apart.
            let subset = &coords[..baseline_count(row_count).min(coords.len())];
            let mut baseline: Vec<Vec<u64>> = Vec::with_capacity(subset.len());
            let mut a_calls = Vec::with_capacity(subset.len());
            for (i, &p) in subset.iter().enumerate() {
                let clock = Instant::now();
                let proposal = device
                    .propose(&batch, &targets, p)
                    .expect("baseline propose");
                baseline.push(proposal.candidate_losses().expect("losses"));
                a_calls.push(clock.elapsed().as_micros());
                eprintln!(
                    "  {row_count} rows {kind} A baseline: {} of {}, {} us",
                    i + 1,
                    subset.len(),
                    a_calls[i]
                );
            }
            let a_per_coord = median_us(a_calls[WARMUP.min(a_calls.len() - 1)..].to_vec());
            eprintln!(
                "{row_count} rows {kind} A baseline: {} coordinates, warm-up call {} us, per coordinate {a_per_coord} us{}, {} targets/s at one coordinate per batch",
                subset.len(),
                a_calls[0],
                if a_calls.len() > WARMUP { " (median after warm-up)" } else { " (the warm-up call itself)" },
                per_second(rows.len(), a_per_coord)
            );

            // Arm B: rows-K at each block size, over every coordinate in order.
            // The first K's untimed-pass words are the reference for every
            // later K; the subset must also equal the baseline.
            let mut reference: Option<Vec<CoordinateScore>> = None;
            for k in BLOCKS {
                let chunk: Vec<&[Parameter]> = coords.chunks(k).collect();
                let mut b_samples = Vec::new();
                for repeat in 0..WARMUP + TIMED {
                    let clock = Instant::now();
                    let mut scored = Vec::with_capacity(coords.len());
                    for part in &chunk {
                        scored.extend(
                            device
                                .score_rows_k(&batch, &targets, part, &scoring)
                                .expect("rows-K score"),
                        );
                    }
                    let us = clock.elapsed().as_micros();
                    if repeat >= WARMUP {
                        b_samples.push(us);
                    }
                    if repeat == 0 {
                        assert_eq!(scored.len(), coords.len());
                        for (score, want) in scored.iter().zip(&baseline) {
                            if !agrees(score, want) {
                                mismatches.push(format!(
                                    "{row_count} rows {kind} K{k} {:?} against baseline: status {} rows-K {:?} baseline {want:?}",
                                    score.parameter, score.status, score.losses
                                ));
                            }
                        }
                        match &reference {
                            None => reference = Some(scored),
                            Some(first) => {
                                for (score, want) in scored.iter().zip(first) {
                                    if score.parameter != want.parameter
                                        || !agrees(score, &want.losses)
                                    {
                                        mismatches.push(format!(
                                            "{row_count} rows {kind} K{k} {:?} against K{}: status {} rows-K {:?} reference {:?}",
                                            score.parameter, BLOCKS[0], score.status, score.losses, want.losses
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                // Rates from what was scored: every coordinate over the whole
                // pass, so a short last call cannot overstate them.
                let b_us = median_us(b_samples);
                let per_call = b_us / chunk.len() as u128;
                let b_per_coord = (b_us / coords.len() as u128).max(1);
                eprintln!(
                    "{row_count} rows {kind} B rows-K K{k}: {b_us} us for {} coordinates in {} calls, mean {per_call} us/call, per coordinate {b_per_coord} us against baseline {a_per_coord} us (x{} integer), {} row-coord/s, {} targets/s at one call per batch",
                    coords.len(),
                    chunk.len(),
                    a_per_coord / b_per_coord,
                    per_second(coords.len() * rows.len(), b_us),
                    per_second(rows.len(), per_call)
                );
            }
            if kind == "VectorBlock" {
                vector_reference = reference.expect("reference scores");
            }
        }

        // Not vacuous: one target moved in rows-K's input only. Every planted
        // coordinate must fail the same parity predicate.
        let mut shifted = targets.clone();
        shifted[0] = (shifted[0] + 1) % STUDENT_SPEC.vocabulary;
        let block = &vector[..PLANTED];
        let scored = device
            .score_rows_k(&batch, &shifted, block, &scoring)
            .expect("rows-K score");
        let caught = scored
            .iter()
            .zip(&vector_reference)
            .filter(|(score, want)| !agrees(score, &want.losses))
            .count();
        eprintln!(
            "{row_count} rows planted target shift: {caught} of {PLANTED} coordinates caught"
        );
        assert_eq!(
            caught, PLANTED,
            "a moved target went unseen at {row_count} rows"
        );
    }

    for line in &mismatches {
        eprintln!("  {line}");
    }
    assert!(
        mismatches.is_empty(),
        "{} loss-word mismatches",
        mismatches.len()
    );
    assert_eq!(
        device.epoch().expect("epoch"),
        epoch,
        "scoring changed the epoch"
    );
    gpu_apps("after");
}
