//! claude-406: how fast the discrete model's CUDA scoring runs on one real D87
//! student batch, baseline `propose` against rows-K `score_rows_k`, with every
//! rows-K loss word checked against the baseline's. Scoring only: no choose,
//! no apply, no epoch change. `#[ignore]`: it needs a GPU and
//! `UOR_STUDENT_STREAM` (a `student-ingest` output folder).

use crate::cpu_oracle_parity::{device_with, student_batch, windows, STUDENT_SEED, STUDENT_SPEC};
use crate::{Parameter, ReadSpec};
use std::time::Instant;

const WARMUP: usize = 1;
const TIMED: usize = 5;
const BLOCKS: [usize; 5] = [1, 8, 16, 32, 64];

fn median_us(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

/// `(calls per second, coordinates per second, row-coordinates per second,
/// batch targets per second)` for one call of `coordinates` over `rows` rows
/// taking `us` microseconds.
/// Integer arithmetic only: per-second rates are `count * 1_000_000 / us`.
fn rates(us: u128, coordinates: usize, rows: usize) -> String {
    let us = us.max(1);
    let per_second = |count: usize| count as u128 * 1_000_000 / us;
    format!(
        "{us} us/call, {} coord/s, {} row-coord/s, {} targets/s at one call per batch",
        per_second(coordinates),
        per_second(coordinates * rows),
        per_second(rows)
    )
}

#[test]
#[ignore = "needs a GPU and UOR_STUDENT_STREAM (claude-406)"]
fn student_batch_scoring_speed_baseline_against_rows_k() {
    let student = student_batch();
    let rows = &student.rows;
    let targets: Vec<u32> = rows.iter().map(|s| s.target).collect();
    eprintln!("student batch: {}; {} rows", student.pins, rows.len());

    let device = device_with(STUDENT_SPEC, ReadSpec::Route, STUDENT_SEED);
    let epoch = device.epoch().expect("epoch");
    let scoring = device.rowsk_scoring().expect("rows-K scoring");
    let batch = device.carry_windows(&windows(rows)).expect("carry");

    // Fixed before any timing, never chosen by loss.
    let blocks = STUDENT_SPEC.blocks;
    let distinct: std::collections::BTreeSet<u32> = targets.iter().copied().collect();
    let vector: Vec<Parameter> = distinct
        .iter()
        .flat_map(|&t| (0..blocks).map(move |b| Parameter::VectorBlock(t * blocks + b)))
        .take(64)
        .collect();
    let route_count = device.snapshot().expect("snapshot").routes.len().min(64) as u32;
    let route: Vec<Parameter> = (0..route_count).map(Parameter::Route).collect();
    assert_eq!(
        vector.len(),
        64,
        "the batch names fewer than 16 target tokens"
    );
    assert!(route_count > 0, "the Route model has no routes");

    let mut mismatches = Vec::new();
    for (kind, coords) in [("VectorBlock", &vector), ("Route", &route)] {
        // Arm A: the baseline, one coordinate per launch. One pass only: each
        // call takes seconds at this size (c406-run1 spent 27 min in a
        // repeated arm A). The first call is the warm-up and is timed apart;
        // the per-call median is over the other calls.
        let mut baseline: Vec<Vec<u64>> = Vec::with_capacity(coords.len());
        let mut a_calls = Vec::with_capacity(coords.len());
        let pass = Instant::now();
        for (i, &p) in coords.iter().enumerate() {
            let clock = Instant::now();
            let proposal = device
                .propose(&batch, &targets, p)
                .expect("baseline propose");
            baseline.push(proposal.candidate_losses().expect("losses"));
            a_calls.push(clock.elapsed().as_micros());
            if (i + 1) % 8 == 0 {
                eprintln!(
                    "  {kind} A baseline: {} of {} coordinates, last call {} us, {} s so far",
                    i + 1,
                    coords.len(),
                    a_calls[i],
                    pass.elapsed().as_secs()
                );
            }
        }
        eprintln!("{kind} A baseline warm-up call: {} us", a_calls[0]);
        let a_us =
            median_us(a_calls[WARMUP.min(a_calls.len() - 1)..].to_vec()) * coords.len() as u128;
        eprintln!(
            "{kind} A baseline, {} coordinates one per launch (median call x count): {}",
            coords.len(),
            rates(a_us, coords.len(), rows.len())
        );
        let a_per_coord = a_us / coords.len() as u128;

        // Arm B: rows-K at each block size, over the same coordinates in order.
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
                    for (score, want) in scored.iter().zip(&baseline) {
                        if score.status != 0 || &score.losses != want {
                            mismatches.push(format!(
                                "{kind} K{k} {:?}: status {} rows-K {:?} baseline {want:?}",
                                score.parameter, score.status, score.losses
                            ));
                        }
                    }
                    assert_eq!(scored.len(), coords.len());
                }
            }
            let b_us = median_us(b_samples);
            let per_call = b_us / chunk.len() as u128;
            let b_per_coord = (b_us / coords.len() as u128).max(1);
            eprintln!(
                "{kind} B rows-K K{k}: {} ({} calls); per coordinate {b_per_coord} us against baseline {a_per_coord} us, speed-up x{} (integer)",
                rates(per_call, k.min(coords.len()), rows.len()),
                chunk.len(),
                a_per_coord / b_per_coord
            );
        }
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

    // Not vacuous: one target moved in rows-K's input only must be reported.
    let mut shifted = targets.clone();
    shifted[0] = (shifted[0] + 1) % STUDENT_SPEC.vocabulary;
    let block = &vector[..16];
    let scored = device
        .score_rows_k(&batch, &shifted, block, &scoring)
        .expect("rows-K score");
    let caught = block
        .iter()
        .zip(&scored)
        .filter(|(&p, score)| {
            let want = device
                .propose(&batch, &targets, p)
                .expect("baseline propose")
                .candidate_losses()
                .expect("losses");
            score.losses != want
        })
        .count();
    eprintln!(
        "planted target shift: {caught} of {} coordinates differ",
        block.len()
    );
    assert!(caught > 0, "a moved target went unseen");
}
