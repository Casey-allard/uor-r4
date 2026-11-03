use std::time::Instant;

use rayon::ThreadPoolBuilder;
use uor_r4_cpu_train::{
    CounterfactualConfig, CounterfactualGroup, CpuCounterfactualTrainer, CpuRankMarginTrainer,
    RankMarginConfig, RankMarginSample, VOCABULARY,
};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

const WARMUP_STEPS: usize = 2;
const TIMED_STEPS: usize = 8;
const COORDINATES_PER_STEP: usize = 16;
const THREAD_COUNTS: [usize; 4] = [1, 8, 16, 32];

fn main() {
    let artifact = artifact();
    let sample = sample(&artifact);
    let groups = [
        (CounterfactualGroup::Normalization, 16),
        (CounterfactualGroup::Bias, 1_000_000),
        (CounterfactualGroup::Decay, 1_000_000),
        (CounterfactualGroup::Route, 1),
    ];
    for (group, delta) in groups {
        let mut baseline = None;
        for threads in THREAD_COUNTS {
            let pool = ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("counterfactual thread pool");
            pool.install(|| {
                for _ in 0..WARMUP_STEPS {
                    let mut trainer = trainer(&artifact, &sample, group, delta);
                    trainer.train_step().expect("counterfactual warmup");
                }
            });

            let mut evaluations = 0usize;
            let mut accepted = 0usize;
            let mut elapsed = 0.0;
            let mut checksum = 0u64;
            for _ in 0..TIMED_STEPS {
                let step_started = Instant::now();
                let mut trainer = trainer(&artifact, &sample, group, delta);
                let step = pool.install(|| trainer.train_step().expect("counterfactual step"));
                elapsed += step_started.elapsed().as_secs_f64();
                evaluations += step.tried;
                accepted += usize::from(step.accepted.is_some());
                checksum = checksum
                    .rotate_left(7)
                    .wrapping_add(blake3::hash(trainer.artifact()).as_bytes()[0] as u64)
                    .wrapping_add(step.after.margin_loss as u64);
            }
            baseline.get_or_insert(elapsed);
            let speedup = baseline.expect("counterfactual baseline") / elapsed;
            println!(
                "schema=uor-r4.cpu-counterfactual-bench/1 group={group:?} threads={threads} vocab={} width=64 layers=1 context=8 coordinates_per_step={COORDINATES_PER_STEP} timed_steps={TIMED_STEPS} candidate_evaluations={evaluations} elapsed_ms={:.3} candidate_evaluations_per_second={:.3} speedup={speedup:.3} accepted_steps={accepted} checksum={checksum} peak_rss_kib={}",
                VOCABULARY,
                elapsed * 1_000.0,
                evaluations as f64 / elapsed,
                peak_rss_kib()
            );
        }
    }
}

fn trainer(
    artifact: &[u8],
    sample: &RankMarginSample,
    group: CounterfactualGroup,
    delta: i32,
) -> CpuCounterfactualTrainer {
    CpuCounterfactualTrainer::from_artifact(
        artifact,
        vec![sample.clone()],
        CounterfactualConfig {
            group,
            margin_q12: 1,
            delta,
            coordinates_per_step: COORDINATES_PER_STEP,
        },
        20_261_106,
    )
    .expect("counterfactual trainer")
}

fn sample(artifact: &[u8]) -> RankMarginSample {
    let dummy = RankMarginSample {
        input: vec![0],
        target: 0,
    };
    let mut probe = CpuRankMarginTrainer::from_artifact(
        artifact,
        vec![dummy.clone()],
        RankMarginConfig::default(),
        20_261_107,
    )
    .expect("rank-margin probe");
    for input in 0u32..128 {
        let reading = probe
            .evaluate_sample(&RankMarginSample {
                input: vec![input],
                target: 0,
            })
            .expect("sample reading");
        for target in reading.top8.iter().rev().copied() {
            if target == reading.top1 {
                continue;
            }
            let sample = RankMarginSample {
                input: vec![input],
                target,
            };
            let target_reading = probe.evaluate_sample(&sample).expect("target reading");
            if target_reading.target_admitted && target_reading.margin_loss > 0 {
                return sample;
            }
        }
    }
    panic!("no admitted sample with a nonzero margin")
}

fn artifact() -> Vec<u8> {
    let mut snapshot = synthetic_fbs_trunk_with(
        20_261_108,
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
