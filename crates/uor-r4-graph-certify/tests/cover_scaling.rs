//! #435 measurement: does the induced cover's capacity track the corpus?
//!
//! # The measured defect
//!
//! The shipped split test is an **absolute** entropy floor
//! (`DEFAULT_SPLIT_ENTROPY_GAIN_BITS = 0.25` bits, fixed `k0 = 8`,
//! `depths = 3`, `regions_budget = 256`, `min_support = 64`). On a 500k-record
//! natural corpus it induced **38 regions** with emission-contrast mean
//! **0.5012**; on the same domain at **2 110 111 records** (4.2×) it induced
//! only **14 regions** with contrast mean **0.0915**. More data produced a
//! *coarser* geometry. Mechanism: as `n` grows a region's next-token
//! distribution spreads over more types, so any single binary split removes a
//! smaller *absolute* share of the entropy and the constant 0.25-bit bar
//! becomes a stricter filter. `regions_budget` never bound; `min_support` is
//! trivially satisfied at scale.
//!
//! The structure is really there: on nested prefixes of the same corpus over a
//! 17× data range, distinct left-context keys grow as ~`n^0.44` and distinct
//! next-token-distribution signature classes as ~`n^0.46` (left) / ~`n^0.80`
//! (two-sided), while mean support per class *rises* 7.3 → 35.6. Nothing
//! saturates, so the cover should not saturate either.
//!
//! # Arms
//!
//! Each arm is a [`CoverConfig`] differing from the shipped default only in
//! the #435 knobs (all default-off, so `absolute` reproduces today's cover):
//!
//! - `absolute` — baseline: `gain > 0.25` bits.
//! - `relative` — `gain > theta · H(parent)`, `theta = 0.0384` (calibrated so
//!   the bar equals 0.25 bits at a 6.518-bit parent — the measured mean
//!   H(parent) over the audited split candidates at the 500k reference).
//! - `mdl` — `gain · support > penalty · added_params · log₂(n)`,
//!   `penalty = 0.5` bits/parameter (BIC), `added_params = parent_types`.
//! - `scaled-k0` — absolute floor, but `k0` and `regions_budget` scale as
//!   `(n / 400 000)^0.45` from the measured class-growth law.
//! - `relative+scaled` — the relative floor *and* the scaled capacity.
//!
//! # Pre-declared exit rule (written before the run)
//!
//! A criterion **PASSES** iff all three hold:
//!
//! 1. **Monotone capacity** — region count is non-decreasing in corpus size
//!    across the tested sizes.
//! 2. **Contrast does not collapse** — emission-contrast mean at every tested
//!    size is ≥ `0.5012 / 2.0 = 0.2506`, i.e. within a factor of 2 of the
//!    500k / 38-region reference (0.5012). The 2.11M absolute baseline
//!    (0.0915) is a factor of 5.5 below the reference and would fail.
//! 3. **Prediction is not bought at a loss** — held-out top-1 accuracy of a
//!    region-path-keyed store at the largest tested size is ≥ the `absolute`
//!    baseline's held-out top-1 at that same size.
//!
//! Every criterion is reported regardless of verdict; a negative result is a
//! valid result and is reported as such.
//!
//! # Two traps this harness had, both fixed 2026-08-07 (#460 lever 1)
//!
//! **The fallback partition was not prefix-compatible.** Nested prefixes need
//! every corpus-size prefix to contain both partitions. The `R4_STORIES` path
//! satisfies that (`blake3(article id) % 5` interleaves held-out stories); the
//! fallback was a contiguous tail split, so on the committed fixture every
//! prefix below 80% had an empty held-out set and was skipped. The four-size
//! curve silently became one point — and the verdict block then printed
//! `PASS ... monotone_regions=true`, because `windows(2).all(..)` is vacuously
//! true over one row. The fallback is now an interleaved 80/20 (`sid % 5 != 0`)
//! and an arm with fewer than two sizes reports INCONCLUSIVE, never PASS.
//!
//! **The scaling anchor coincides with the fixture's full size.** The capacity
//! law is `(n / capacity_ref_n)^alpha` with `DEFAULT_CAPACITY_REF_N = 400_000`,
//! and the committed 500k fixture yields 400,006 train observations. At the
//! anchor every scaled knob equals its base value, so `scaled-k0` reduces
//! *exactly* to `absolute` at the largest size — identical regions, contrast
//! and top-1 — and condition 3 compares the arm with itself. It passes, and
//! the pass means nothing. The `@ref50k` arms lower the anchor to 50,000 so
//! the fixture's largest size sits 8x above it and the law is exercised where
//! the verdict is read. **Read `scaled-k0` on this corpus as a control, not a
//! result; read `scaled-k0@ref50k` as the measurement.**
//!
//! # Runtime caps (printed at start, never silent)
//!
//! - `R4_SCALING_FRACS` (default `13,25,50,100`) — story-prefix percentages.
//! - `R4_SCALING_MAX_TRAIN` (default `0` = unlimited) — cap on train
//!   observations per size; a bound cap prints `CAP BOUND`.
//! - `R4_SCALING_MAX_HELD` (default `50000`) — cap on evaluated held-out
//!   positions per size.
//! - `R4_SCALING_ARMS` (default all five) — comma-separated arm list.
//! - `R4_SCALING_THREADS` (default `2`) — observation-extraction workers
//!   (κ-neutral: reductions stay ordered).
//!
//! Run (500k natural corpus):
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl R4_ARTIFACTS=/tmp/tless_artifacts.bin \
//!   cargo test --release -p uor-r4-graph-certify --test cover_scaling -- \
//!     --ignored --nocapture

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction::{self, CoverConfig, Observation, SplitCriterion};

/// Top-E used for emission contrast — the `emission_entries` default of
/// `uor_r4_graph_certify::score` (64), so the numbers are comparable with the
/// scorer's `mean_contrast`.
const CONTRAST_TOP_E: usize = 64;
/// The 500k / 38-region reference contrast the exit rule is written against.
const REFERENCE_CONTRAST: f64 = 0.5012;
/// Documented tolerance factor of exit-rule clause (2).
const CONTRAST_COLLAPSE_FACTOR: f64 = 2.0;

/// Region-path store: one map per depth, path prefix -> next-token counts.
type PathStore = Vec<BTreeMap<Vec<u32>, BTreeMap<u32, u64>>>;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    match std::env::var(name) {
        Ok(value) => value
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be an integer, got {value:?}")),
        Err(_) => default,
    }
}

fn argmax64(dist: &BTreeMap<u32, u64>) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
}

/// Cosine descent down the region tree (the #399 M1 routing protocol).
fn route_path(regions: &[induction::CoverRegion], depth1: &[u32], vector: &[f32]) -> Vec<u32> {
    let dot = |rid: u32| -> f32 {
        let p = &regions[rid as usize].prototype;
        p.iter().zip(vector).map(|(a, b)| a * b).sum()
    };
    let mut path = Vec::new();
    let mut candidates: Vec<u32> = depth1.to_vec();
    while !candidates.is_empty() {
        let best = candidates
            .iter()
            .copied()
            .max_by(|&a, &b| dot(a).partial_cmp(&dot(b)).unwrap())
            .unwrap();
        path.push(best);
        candidates = regions[best as usize].children.clone();
    }
    path
}

fn top_set(counts: &BTreeMap<u32, u64>, e: usize) -> Vec<u32> {
    let mut top: Vec<(u32, u64)> = counts.iter().map(|(&t, &c)| (t, c)).collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top.truncate(e);
    top.into_iter().map(|(t, _)| t).collect()
}

/// One (arm, size) measurement row.
#[derive(Debug, Clone)]
struct ArmResult {
    arm: String,
    train: usize,
    regions: usize,
    max_depth: usize,
    contrast_mean: f64,
    contrast_min: f64,
    contrast_max: f64,
    mean_support: f64,
    held_top1: f64,
    held_eval: u64,
    /// Mean next-token entropy of the regions whose split was audited —
    /// the calibration quantity behind `relative`'s default theta
    /// (0.25 bits / mean H(parent) at the 500k reference).
    parent_entropy_mean: f64,
    gain_mean: f64,
    gain_max: f64,
    splits_audited: usize,
    k0_eff: usize,
    budget_eff: usize,
    seconds: f64,
}

fn arm_config(arm: &str) -> CoverConfig {
    let base = CoverConfig::default();
    match arm {
        "absolute" => base,
        "relative" => CoverConfig {
            split_criterion: SplitCriterion::RelativeGain,
            ..base
        },
        "mdl" => CoverConfig {
            split_criterion: SplitCriterion::Mdl,
            ..base
        },
        "scaled-k0" => CoverConfig {
            scale_k0: true,
            scale_regions_budget: true,
            ..base
        },
        "relative+scaled" => CoverConfig {
            split_criterion: SplitCriterion::RelativeGain,
            scale_k0: true,
            scale_regions_budget: true,
            ..base
        },
        // Anchor-shifted controls. The capacity law is `(n / capacity_ref_n)^alpha`
        // and `DEFAULT_CAPACITY_REF_N = 400_000`, which is — to within six
        // observations — the full-size train count of the committed 500k
        // fixture. At that size every scaled knob equals its base value, so
        // `scaled-k0` reduces *exactly* to `absolute` there and the exit
        // rule's largest-size comparison is the arm against itself.
        //
        // Lowering the anchor to 50 000 puts the fixture's largest size 8x
        // above it, so the scaling law is actually exercised where the verdict
        // is read. This tests the law rather than the coincidence.
        "scaled-k0@ref50k" => CoverConfig {
            scale_k0: true,
            scale_regions_budget: true,
            capacity_ref_n: 50_000,
            ..base
        },
        "relative+scaled@ref50k" => CoverConfig {
            split_criterion: SplitCriterion::RelativeGain,
            scale_k0: true,
            scale_regions_budget: true,
            capacity_ref_n: 50_000,
            ..base
        },
        other => panic!("unknown arm {other:?}"),
    }
}

#[allow(clippy::too_many_arguments)]
fn measure_arm(
    arm: &str,
    config: &CoverConfig,
    train_obs: &[Observation],
    held_obs: &[Observation],
    held_next: &[u32],
    threads: u32,
) -> ArmResult {
    let started = Instant::now();
    let config = CoverConfig {
        threads,
        ..config.clone()
    };
    let induced = induction::induce_cover(train_obs, &config, "i435-artifact", "i435-corpus")
        .expect("cover induction");
    let cover = &induced.cover;

    // Emission contrast per region against the global train prior, top-E
    // overlap on counts (the scorer's definition).
    let mut root_counts: BTreeMap<u32, u64> = BTreeMap::new();
    for observation in train_obs {
        *root_counts.entry(observation.next).or_default() += 1;
    }
    let root_top: BTreeSet<u32> = top_set(&root_counts, CONTRAST_TOP_E).into_iter().collect();
    let (mut c_sum, mut c_min, mut c_max) = (0.0f64, f64::INFINITY, f64::NEG_INFINITY);
    let mut support_sum = 0u64;
    for (region_id, region) in cover.regions.iter().enumerate() {
        let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
        for &member in &cover.members[region_id] {
            *counts.entry(train_obs[member].next).or_default() += 1;
        }
        let region_top = top_set(&counts, CONTRAST_TOP_E);
        let shared = region_top
            .iter()
            .filter(|token| root_top.contains(token))
            .count();
        let contrast = 1.0 - (shared as f64 / region_top.len().max(1) as f64);
        c_sum += contrast;
        c_min = c_min.min(contrast);
        c_max = c_max.max(contrast);
        support_sum += u64::from(region.support);
    }
    // Split-decision calibration: mean H(parent) over audited candidates.
    let mut audited_entropy = 0.0f64;
    let mut gain_sum = 0.0f64;
    let mut gain_max = 0.0f64;
    for audit in &induced.decision_trace {
        audited_entropy += cover.regions[audit.region_id as usize].entropy_bits;
        gain_sum += audit.entropy_gain_bits;
        gain_max = gain_max.max(audit.entropy_gain_bits);
    }
    let splits_audited = induced.decision_trace.len();
    let parent_entropy_mean = audited_entropy / splits_audited.max(1) as f64;
    let gain_mean = gain_sum / splits_audited.max(1) as f64;

    let regions = cover.regions.len();
    let contrast_mean = c_sum / regions.max(1) as f64;
    let mean_support = support_sum as f64 / regions.max(1) as f64;

    // Region-path-keyed next-token store over the train observations.
    let mut store: PathStore = (0..=cover.max_depth).map(|_| BTreeMap::new()).collect();
    for (obs_index, observation) in train_obs.iter().enumerate() {
        let path = &cover.paths[obs_index];
        for depth in 0..=path.len() {
            *store[depth]
                .entry(path[..depth].to_vec())
                .or_default()
                .entry(observation.next)
                .or_default() += 1;
        }
    }
    let unigram_pred = argmax64(&root_counts);

    // Held-out top-1 at the deepest populated path prefix.
    let depth1 = cover.regions_at_depth(1);
    let (mut evaluated, mut hits) = (0u64, 0u64);
    for (obs_index, observation) in held_obs.iter().enumerate() {
        let path = route_path(&cover.regions, &depth1, &observation.vector);
        let mut prediction = unigram_pred;
        for depth in (0..=path.len().min(cover.max_depth)).rev() {
            if let Some(dist) = store[depth].get(&path[..depth]) {
                prediction = argmax64(dist);
                break;
            }
        }
        evaluated += 1;
        if prediction == Some(held_next[obs_index]) {
            hits += 1;
        }
    }

    ArmResult {
        arm: arm.to_owned(),
        train: train_obs.len(),
        regions,
        max_depth: cover.max_depth,
        contrast_mean,
        contrast_min: if regions == 0 { 0.0 } else { c_min },
        contrast_max: if regions == 0 { 0.0 } else { c_max },
        mean_support,
        held_top1: hits as f64 / evaluated.max(1) as f64,
        held_eval: evaluated,
        parent_entropy_mean,
        gain_mean,
        gain_max,
        splits_audited,
        k0_eff: config.effective_k0(train_obs.len()),
        budget_eff: config.effective_regions_budget(train_obs.len()),
        seconds: started.elapsed().as_secs_f64(),
    }
}

#[test]
#[ignore = "#435 scaling measurement; run explicitly with --ignored"]
fn cover_scaling() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let corpus = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let artifacts = compiler::load_artifacts_from(&art_path).expect("artifacts");

    let fracs: Vec<usize> = std::env::var("R4_SCALING_FRACS")
        .unwrap_or_else(|_| "13,25,50,100".to_owned())
        .split(',')
        .map(|part| part.trim().parse().expect("fraction percent"))
        .collect();
    let arms: Vec<String> = std::env::var("R4_SCALING_ARMS")
        .unwrap_or_else(|_| "absolute,relative,mdl,scaled-k0,relative+scaled".to_owned())
        .split(',')
        .map(|part| part.trim().to_owned())
        .collect();
    let max_train = env_usize("R4_SCALING_MAX_TRAIN", 0);
    let max_held = env_usize("R4_SCALING_MAX_HELD", 50_000);
    let threads = env_usize("R4_SCALING_THREADS", 2) as u32;

    println!("#435 cover scaling measurement");
    println!(
        "  corpus:   {meta_path} / {recs_path} ({} records)",
        corpus.n
    );
    println!("  artifacts:{art_path}");
    println!("  CAPS: fracs={fracs:?} max_train={max_train} (0=unlimited) max_held={max_held} threads={threads}");
    println!("  ARMS: {arms:?}");
    println!(
        "  EXIT RULE: (1) regions non-decreasing in n; (2) contrast mean >= {:.4} \
         ({REFERENCE_CONTRAST} / {CONTRAST_COLLAPSE_FACTOR}) at every size; \
         (3) held-out top-1 at the largest size >= the absolute baseline there",
        REFERENCE_CONTRAST / CONTRAST_COLLAPSE_FACTOR
    );

    // Construction/held-out partition (stories.jsonl labels, 80/20 fallback).
    let cut = (corpus.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = match std::env::var("R4_STORIES") {
        Ok(path) => {
            let text = std::fs::read_to_string(&path).expect("stories.jsonl");
            let mut flags = vec![true; corpus.stories as usize];
            for line in text.lines() {
                let Some(story_pos) = line.find("\"story\":") else {
                    continue;
                };
                let story: usize = line[story_pos + 8..]
                    .split(',')
                    .next()
                    .and_then(|x| x.trim().parse().ok())
                    .expect("story id");
                if story < flags.len() {
                    flags[story] = !line.contains("\"partition\":\"HeldOut\"");
                }
            }
            flags
        }
        // Fallback partition: INTERLEAVED 80/20, not a contiguous tail.
        //
        // This measurement is built on nested corpus-size prefixes, so every
        // prefix must contain both partitions. A tail split (`sid < cut`) puts
        // all held-out stories above the 80% mark, so every prefix below 80%
        // has an empty held-out set and is skipped — on the committed 500k
        // fixture that silently reduced a four-size scaling curve to a single
        // point, and the verdict block below then reported `monotone_regions`
        // PASS over one row. The `R4_STORIES` path never had this problem
        // because `stories.jsonl` partitions by `blake3(article id) % 5`,
        // which is already interleaved.
        //
        // `sid % 5 != 0` keeps the 80/20 ratio and the document-level
        // boundary (no position from an evaluated story trains the cover)
        // while making every prefix usable. It does change which stories are
        // held out relative to a tail split, so absolute numbers from this
        // fallback are not comparable with runs that supplied `R4_STORIES`;
        // arm-to-arm and size-to-size comparisons within one run are.
        Err(_) => {
            let _ = cut;
            (0..corpus.stories).map(|sid| sid % 5 != 0).collect()
        }
    };
    // Nested prefixes require the position lists to be grouped by story:
    // observations are built in the given position order and each size is a
    // prefix of the next. A stable sort by (story, position) is a no-op on a
    // corpus whose story ids are already non-decreasing (the 500k natural
    // corpus) and is what makes shard-merged corpora (the 2.11M stack, whose
    // story ids interleave) usable at all. Reported when it does something.
    let ordered = (1..corpus.n).all(|i| corpus.story[i] >= corpus.story[i - 1]);
    println!("  story ids already grouped: {ordered} (stable sort applied otherwise)");
    let mut train_pos: Vec<usize> = (0..corpus.n)
        .filter(|&i| constr[corpus.story[i] as usize])
        .collect();
    let mut held_pos: Vec<usize> = (0..corpus.n)
        .filter(|&i| !constr[corpus.story[i] as usize])
        .collect();
    train_pos.sort_by_key(|&i| (corpus.story[i], i));
    held_pos.sort_by_key(|&i| (corpus.story[i], i));
    println!(
        "  positions: {} train, {} held-out, {} stories",
        train_pos.len(),
        held_pos.len(),
        corpus.stories
    );

    let build_started = Instant::now();
    let train_obs = induction::build_observations_with_threads(
        &artifacts,
        &corpus,
        &train_pos,
        threads as usize,
    );
    let held_obs = induction::build_observations_with_threads(
        &artifacts,
        &corpus,
        &held_pos,
        threads as usize,
    );
    println!(
        "  observations built in {:.1}s: {} train, {} held-out",
        build_started.elapsed().as_secs_f64(),
        train_obs.len(),
        held_obs.len()
    );

    let mut rows: Vec<ArmResult> = Vec::new();
    for &frac in &fracs {
        let story_cut = ((corpus.stories as usize * frac) / 100).max(1) as u32;
        let mut train_end = train_pos.partition_point(|&i| corpus.story[i] < story_cut);
        if max_train > 0 && train_end > max_train {
            println!(
                "  CAP BOUND: frac {frac}% train observations {train_end} -> {max_train} \
                 (R4_SCALING_MAX_TRAIN)"
            );
            train_end = max_train;
        }
        let mut held_end = held_pos.partition_point(|&i| corpus.story[i] < story_cut);
        if held_end > max_held {
            println!(
                "  CAP BOUND: frac {frac}% held-out positions {held_end} -> {max_held} \
                 (R4_SCALING_MAX_HELD)"
            );
            held_end = max_held;
        }
        if train_end == 0 || held_end == 0 {
            println!("  frac {frac}%: empty partition, skipped");
            continue;
        }
        let train_slice = &train_obs[..train_end];
        let held_slice = &held_obs[..held_end];
        let held_next: Vec<u32> = held_pos[..held_end]
            .iter()
            .map(|&i| corpus.next[i])
            .collect();
        println!(
            "-- frac {frac}%: stories <{story_cut}, {} train obs, {} held-out obs",
            train_slice.len(),
            held_slice.len()
        );
        for arm in &arms {
            let config = arm_config(arm);
            let row = measure_arm(arm, &config, train_slice, held_slice, &held_next, threads);
            println!(
                "ROW arm={:<16} train={:>8} regions={:>4} max_depth={} contrast_mean={:.4} \
                 contrast_min={:.4} contrast_max={:.4} mean_support={:>10.1} held_top1={:.4} \
                 held_eval={} parent_H_mean={:.3} gain_mean={:.4} gain_max={:.4} splits_audited={} k0_eff={} budget_eff={} \
                 secs={:.1}",
                row.arm,
                row.train,
                row.regions,
                row.max_depth,
                row.contrast_mean,
                row.contrast_min,
                row.contrast_max,
                row.mean_support,
                row.held_top1,
                row.held_eval,
                row.parent_entropy_mean,
                row.gain_mean,
                row.gain_max,
                row.splits_audited,
                row.k0_eff,
                row.budget_eff,
                row.seconds
            );
            rows.push(row);
        }
    }

    // ---- report table + pre-declared exit rule ----
    println!("\n#435 RESULTS (criterion x corpus size)");
    println!(
        "{:<16} {:>9} {:>8} {:>6} {:>9} {:>9} {:>9} {:>12} {:>9} {:>7}",
        "arm",
        "train",
        "regions",
        "depth",
        "cont_mean",
        "cont_min",
        "cont_max",
        "mean_sup",
        "held_top1",
        "secs"
    );
    for row in &rows {
        println!(
            "{:<16} {:>9} {:>8} {:>6} {:>9.4} {:>9.4} {:>9.4} {:>12.1} {:>9.4} {:>7.1}",
            row.arm,
            row.train,
            row.regions,
            row.max_depth,
            row.contrast_mean,
            row.contrast_min,
            row.contrast_max,
            row.mean_support,
            row.held_top1,
            row.seconds
        );
    }

    let contrast_floor = REFERENCE_CONTRAST / CONTRAST_COLLAPSE_FACTOR;
    let baseline_largest = rows
        .iter()
        .filter(|row| row.arm == "absolute")
        .max_by_key(|row| row.train)
        .map(|row| row.held_top1);
    println!("\n#435 VERDICTS (exit rule pre-declared in the module docs)");
    for arm in &arms {
        let arm_rows: Vec<&ArmResult> = rows.iter().filter(|row| &row.arm == arm).collect();
        if arm_rows.is_empty() {
            continue;
        }
        // Condition 1 is a statement about how region count moves WITH corpus
        // size. One row cannot support it, and `windows(2).all(..)` is
        // vacuously true over one row — which is how a degenerate run printed
        // PASS. Fewer than two sizes means the condition is unevaluable, and
        // an unevaluable condition is not a satisfied one.
        let sizes = arm_rows.len();
        let monotone_evaluable = sizes >= 2;
        let monotone = arm_rows
            .windows(2)
            .all(|pair| pair[1].regions >= pair[0].regions);
        let contrast_ok = arm_rows
            .iter()
            .all(|row| row.contrast_mean >= contrast_floor);
        let largest = arm_rows.iter().max_by_key(|row| row.train).unwrap();
        let predictive_ok = baseline_largest.is_none_or(|baseline| largest.held_top1 >= baseline);
        let verdict = if !monotone_evaluable {
            "INCONCLUSIVE"
        } else if monotone && contrast_ok && predictive_ok {
            "PASS"
        } else {
            "FAIL"
        };
        if !monotone_evaluable {
            println!(
                "INCONCLUSIVE {arm:<16} only {sizes} corpus size produced a row; condition 1 \
                 (monotone regions) is UNEVALUABLE. Raise R4_SCALING_FRACS coverage or supply \
                 a corpus whose partition survives prefixing — do not read the remaining \
                 columns as a verdict."
            );
        }
        println!(
            "{verdict} {arm:<16} sizes={sizes} monotone_regions={monotone} contrast_ok={contrast_ok} \
             (min mean {:.4} vs floor {contrast_floor:.4}) predictive_ok={predictive_ok} \
             (top1 {:.4} vs baseline {:.4})",
            arm_rows
                .iter()
                .map(|row| row.contrast_mean)
                .fold(f64::INFINITY, f64::min),
            largest.held_top1,
            baseline_largest.unwrap_or(f64::NAN)
        );
    }
}
