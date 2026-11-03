use std::time::Instant;

use rayon::ThreadPoolBuilder;
use uor_r4_cpu_train::{
    CpuPanelEvaluator, CpuRankMarginTrainer, RankMarginConfig, RankMarginSample, VOCABULARY,
};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

const ROWS: u32 = 64;
const WARMUP_RUNS: usize = 2;
const TIMED_RUNS: usize = 16;
const THREAD_COUNTS: [usize; 4] = [1, 8, 16, 32];

fn main() {
    let artifact = artifact();
    let samples = samples(&artifact);
    let mut baseline = None;
    for threads in THREAD_COUNTS {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("panel thread pool");
        pool.install(|| {
            for _ in 0..WARMUP_RUNS {
                let mut evaluator = CpuPanelEvaluator::from_artifact(&artifact, samples.clone(), 1)
                    .expect("panel warmup");
                evaluator.evaluate().expect("panel warmup reading");
            }
        });

        let mut evaluator = CpuPanelEvaluator::from_artifact(&artifact, samples.clone(), 1)
            .expect("panel evaluator");
        let started = Instant::now();
        let mut readings = Vec::with_capacity(TIMED_RUNS);
        pool.install(|| {
            for _ in 0..TIMED_RUNS {
                readings.push(evaluator.evaluate().expect("panel reading"));
            }
        });
        let seconds = started.elapsed().as_secs_f64();
        baseline.get_or_insert(seconds);
        let speedup = baseline.expect("panel baseline") / seconds;
        let rows = u64::from(ROWS) * TIMED_RUNS as u64;
        let reading = readings.last().expect("timed panel reading");
        let checksum = readings.iter().fold(0u64, |checksum, reading| {
            checksum
                .rotate_left(7)
                .wrapping_add(reading.margin_loss_sum as u64)
                .wrapping_add(reading.target_admitted as u64)
                .wrapping_add(reading.export.query_coordinate_mismatches)
        });
        println!(
            "schema=uor-r4.cpu-panel-export-bench/1 threads={threads} vocab={} width=64 layers=1 context=8 rows={ROWS} timed_runs={TIMED_RUNS} timed_rows={rows} elapsed_ms={:.3} rows_per_second={:.3} speedup={speedup:.3} admitted={} top1={} top8={} query_mismatches={} token_mismatches={} kept_cell_mismatches={} checksum={checksum} artifact_blake3={} peak_rss_kib={}",
            VOCABULARY,
            seconds * 1_000.0,
            rows as f64 / seconds,
            reading.target_admitted,
            reading.top1_correct,
            reading.top8_correct,
            reading.export.query_coordinate_mismatches,
            reading.export.token_mismatches,
            reading.export.kept_cell_mismatches,
            blake3::hash(&artifact).to_hex(),
            peak_rss_kib()
        );
    }
}

fn samples(artifact: &[u8]) -> Vec<RankMarginSample> {
    let mut probe = CpuRankMarginTrainer::from_artifact(
        artifact,
        vec![RankMarginSample {
            input: vec![0],
            target: 0,
        }],
        RankMarginConfig::default(),
        20_261_110,
    )
    .expect("panel sample probe");
    (0..ROWS)
        .map(|input| {
            let reading = probe
                .evaluate_sample(&RankMarginSample {
                    input: vec![input],
                    target: 0,
                })
                .expect("panel sample reading");
            RankMarginSample {
                input: vec![input],
                target: reading
                    .top8
                    .iter()
                    .copied()
                    .find(|&target| target != reading.top1)
                    .expect("admitted non-top-1 panel target"),
            }
        })
        .collect()
}

fn artifact() -> Vec<u8> {
    let mut snapshot = synthetic_fbs_trunk_with(
        20_261_109,
        FbsSynth {
            vocab: VOCABULARY,
            d: 64,
            mlp: 64,
            layers: 1,
            context: 8,
            kept: 8,
            gain_lo: 0.75,
            gain_span: 1.5,
            table: 0.25,
            decay_step: 0.0625,
        },
    );
    snapshot.embed.clear();
    for token in 0..snapshot.vocab {
        for block in 0..snapshot.width / 8 {
            let root = ((token >> 8) * 7 + ((token >> 4) & 15) * 3 + (token & 15) * 5 + block) & 15;
            snapshot.embed.extend(
                orbit_root_coords(root as u8)
                    .unwrap()
                    .map(|value| value as f32),
            );
        }
    }
    export_fbs_trunk_uorlut01(&snapshot).expect("canonical FBS artifact")
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|value| value.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}
