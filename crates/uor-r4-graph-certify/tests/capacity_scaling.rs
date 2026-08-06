//! #460 instrumentation: does the model's realized capacity track the corpus?
//!
//! # Why this test exists
//!
//! Every structure the score pipeline compiles has a capacity knob that is a
//! pinned constant, not a function of the record count `N`. When a corpus
//! grows past the size those constants were tuned at, each structure
//! *saturates*: it keeps the same number of cells and simply puts more records
//! in each one. A saturated structure still produces numbers, and the numbers
//! look like a model — they are a lookup table. The 500k -> 2.11M measurement
//! of 2026-08-05 (see `docs/capacity_scaling_432.md`) is the worked example:
//! a 4.2x data increase produced *fewer* cover regions (38 -> 14), contrast
//! fell 0.5012 -> 0.0915, and Gate C stopped being able to measure
//! generalization at all.
//!
//! This test does not fix anything. It reports, for ANY corpus, the realized
//! capacity of each structure and prints an explicit `SATURATION VERDICT` line
//! per structure, so a saturated configuration is caught before its accuracy
//! numbers are trusted.
//!
//! # What is measured
//!
//! - COVER — regions per depth, train observations per region, and the
//!   split-decision histogram from `induce_cover`'s decision trace (which rule
//!   rejected each candidate split).
//! - GRADED CODE — occupied prefix keys at every level against the nominal
//!   `K^level` capacity, and mean train records per occupied full-code key.
//! - EXCT — occupancy histogram of full-code keys by construction-record
//!   count, and the share of HELD-OUT records whose full code lands on an
//!   occupied key that clears the exact-context support gate. A corpus where
//!   nearly every held-out record resolves exactly cannot exhibit a strict
//!   miss, which is what Gate C needs to measure generalization.
//! - FWDA — forward-anchor row count, mean row total, and the share of rows
//!   truncated by the per-row entry cap.
//! - NGRAM — compiled bigram/trigram row counts and the share truncated by
//!   the per-context entry cap.
//!
//! # Thresholds
//!
//! Each verdict uses one explicit, documented threshold. They are deliberately
//! coarse: the purpose is to separate "this structure still resolves records"
//! from "this structure is a bucket", not to grade a model.
//!
//! - [`COVER_OBS_PER_REGION_MAX`] — mean train observations per cover region.
//!   Above it, a region is a corpus-scale bucket rather than a context class.
//! - [`COVER_ENTROPY_FLOOR_REJECT_MAX`] — share of audited split candidates
//!   rejected by the entropy floor. Above it, the floor (not the objective,
//!   not the budget) is what bounds the cover.
//! - [`CODE_RECORDS_PER_KEY_MAX`] — mean train records per occupied full code.
//!   Above it, the graded code has stopped separating records.
//! - [`CODE_TRAIN_SAMPLE_FRACTION_MIN`] — share of train records that reach
//!   the codebook k-means. Below it, the codebook is fit to a vanishing
//!   sample of the corpus it is supposed to describe.
//! - [`EXCT_SUPPORTED_RECORD_FRACTION_MAX`] — share of held-out records
//!   resolving at the full code. Above it, strict misses become too rare to
//!   measure generalization from.
//! - [`ROW_TRUNCATION_FRACTION_MAX`] — share of FWDA / NGRAM rows clipped by
//!   their entry cap. Above it, the cap (not the evidence) sets the row width.
//!
//! # Cost, sampling, and what is EXACT versus ESTIMATED (#467)
//!
//! The 2.11M-record run of this instrument cost about twelve minutes, and
//! 585 of those 713 seconds were one thing: the per-record graded-code
//! assignment behind `runtime::build_store_with_threads`. The rest of the
//! instrument — observation building, FWDA/NGRAM compiles, cover induction
//! — costs seconds. Any fast path that does not address the code pass is
//! noise, so this section says exactly what was done to it and what the
//! result is allowed to claim.
//!
//! Three stacking changes, none of which move a reported number:
//!
//! - The code pass runs under rayon over corpus positions (mirroring
//!   `evaluate_gate_c`), and its reduction is an ordered `collect`, so the
//!   result is the input-order code vector regardless of thread count.
//! - It calls `runtime::assign_code_for_bundle` — the allocation-free
//!   per-stage argmax whose tie rule is documented to give the same
//!   primary code as `assign_for_bundle` — instead of materializing the
//!   membership beam for 2.11M records and discarding it.
//! - It never builds the five-level evidence `Store`. Everything this
//!   instrument reads off the store (occupied keys per level, per-key
//!   record counts, per-key teacher weight, the support gate) is
//!   recovered exactly from the sorted construction-split code vector,
//!   which costs one parallel sort instead of roughly five hundred
//!   `BTreeMap` insertions per record.
//!
//! Measured on the 2.11M corpus, those three changes plus the held-out
//! sample take the run from 713s to 613s. The residual is almost entirely
//! the exact construction-split code pass: 468s to assign 1,707,309
//! records, which is 4 stages x 256 classes x 288 dimensions of branchy
//! shift-add per record, about 5e11 term applications, already saturating
//! both cores of this box. Rayon buys nothing over the two threads the old
//! path used because there are only two cores, and the argmax path proved
//! no faster than the membership beam it replaced, which locates the cost
//! in the dot kernel rather than in allocation. Reaching the ten-second
//! neighbourhood would need a vectorized dot kernel in the core crate, not
//! a smarter instrument; anything else buys speed by biasing the two
//! verdicts that sit nearest their limits.
//!
//! Only ONE quantity in this instrument is sampled: the held-out
//! supported-record fraction, under `R4_CAPACITY_SAMPLE` (default
//! 100,000 held-out records; `0` means census). It is a mean of a
//! per-record indicator, so a stride subsample of the held-out list is an
//! unbiased estimator of it, and its binomial standard error
//! sqrt of p(1-p)/n is printed next to the verdict. At n = 100,000 that
//! error is about 0.03pp against a threshold the verdict clears by
//! roughly 9pp.
//!
//! Everything else is EXACT, and deliberately so. The tempting move —
//! subsample the corpus, build the key set from the subsample, scale the
//! counts back up — is wrong for this instrument, in two separate ways:
//!
//! - Distinct occupied keys is not a mean of anything. A key held by `r`
//!   construction records survives a rate-`f` subsample with probability
//!   `1 - (1-f)^r`, so a subsample sees strictly fewer keys and the
//!   shortfall is worst exactly where this corpus lives: 42 percent of
//!   its full-code keys hold a single record, and every one of those is
//!   invisible with probability `1-f`. Dividing by `f` does not repair
//!   it. Since `code.records_per_full_key` is construction records over
//!   distinct keys, and the measured 36.02 sits only 13 percent above its
//!   limit of 32, a subsampled key count moves that verdict directly.
//! - The occupancy histogram is a distribution of per-key counts, and
//!   subsampling THINS it: a key with `r` records shows up with about
//!   `Binomial(r, f)` records, which slides mass toward the low buckets
//!   and deletes keys that draw zero. The shape, not just the scale, is
//!   wrong, so no single factor rescales it.
//!
//! The held-out resolution rate has the same dependency from the other
//! side: it asks whether a held-out code lands on an occupied, supported
//! CONSTRUCTION key, so it is unbiased only while the key set it probes
//! is the complete one. That is why the construction split is never
//! sampled by default even though it is 81 percent of the corpus, and why
//! the honest ceiling on this instrument's speedup is the exact code pass.
//!
//! `R4_CAPACITY_TRAIN_SAMPLE` (default `0`, meaning census) subsamples the
//! construction-split code pass anyway, for a fast look and, more
//! usefully, so the bias above can be reproduced instead of argued about.
//! Every affected line is labelled `BIASED ESTIMATE` and the run prints a
//! banner saying the saturation verdicts are not trustworthy in that mode.
//! Measured on the 2.11M corpus at 100,000 construction records (a
//! 0.05857 rate): occupied full-code keys read 12,193 against the true
//! 47,403, a 74 percent shortfall; mean records per key reads 8.20 against
//! the true 36.02 and its verdict flips SATURATED to PASS; the singleton
//! share inflates from 0.4170 to 0.5474 as the histogram thins; the
//! codebook sample fraction reads 0.10000 against the true 0.00586,
//! flipping its verdict too, because its denominator is the
//! construction-record count the pass actually saw. The held-out
//! resolution rate reads 0.9324 against the true 0.9882 — it keeps its
//! verdict only because it had 8.8pp of margin and the incomplete key set
//! ate 5.6pp of it. Two of the seven verdicts flip on a corpus whose model
//! did not change. That is the estimator, not the measurement.
//!
//! Summary of the taxonomy, for anyone quoting a number out of the log:
//!
//! - EXACT, census over the whole corpus: record and story counts, the
//!   construction-split record count, occupied keys at every level and
//!   their occupancy against nominal, mean construction records per
//!   occupied full-code key, the records-per-key histogram, the singleton
//!   share, total teacher weight, keys clearing the support gate, all
//!   FWDA and NGRAM row statistics, and every cover statistic.
//! - EXACT by construction, independent of the corpus: the codebook
//!   k-means sample fraction, which is a ratio of pinned constants to the
//!   construction-split record count.
//! - ESTIMATED when `R4_CAPACITY_SAMPLE` is nonzero: the held-out
//!   supported-record fraction, and only that. It is reported with its
//!   sample size and standard error, and the log labels it ESTIMATE.
//!
//! # Running
//!
//! On the 500k reference corpus:
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   cargo test --release -p uor-r4-graph-certify --test capacity_scaling -- \
//!     --ignored --nocapture
//!
//! On the 2.11M corpus (the cover section is the memory hog; cap or skip it):
//!   R4_CORPUS_META=/tmp/wiki10k-obs/state.bin \
//!   R4_CORPUS_RECS=/tmp/wiki10k-obs/merged.bin \
//!   R4_CAP_COVER_MAX_TRAIN=400000 \
//!   cargo test --release -p uor-r4-graph-certify --test capacity_scaling -- \
//!     --ignored --nocapture
//!
//! Knobs: `R4_ARTIFACTS` (artifact container, defaults to the core fixture),
//! `R4_CAP_THREADS` (worker count, default 2), `R4_CAP_SKIP_COVER=1` (skip
//! cover induction entirely), `R4_CAP_COVER_MAX_TRAIN` (cap on train
//! observations fed to induction, `0` = unlimited), `R4_CAPACITY_SAMPLE`
//! (held-out records drawn for the sampled resolution rate, default
//! 100,000, `0` = census over the whole held-out split),
//! `R4_CAPACITY_TRAIN_SAMPLE` (construction records drawn for the code
//! pass, default `0` = census; nonzero is the biased fast look).

use std::collections::BTreeMap;
use std::time::Instant;

use rayon::prelude::*;

use uor_r4_core::transformerless::compiler::{self, Corpus, K, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    compile_context_rows, compile_forward_anchor_rows, ScoreConfig, DEFAULT_CONTEXT_ENTRIES,
    FWDA_ENTRY_CAP,
};
use uor_r4_graph_certify::score_runtime::EXCT_SUPPORT_MIN;
use uor_r4_graph_compiler::induction::{self, CoverConfig, Observation};

/// Mean train observations per induced cover region above which the cover is
/// a bucketing of the corpus rather than a partition into context classes.
/// Set at the order of magnitude of the 500k reference (about 13k
/// observations per region across 38 regions); one decimal order above it is
/// the point where a region can no longer carry a distinctive emission list.
const COVER_OBS_PER_REGION_MAX: f64 = 100_000.0;

/// Share of audited split candidates rejected by the entropy floor above
/// which the floor — not the region budget, not the objective, not
/// `min_support` — is the binding capacity constraint.
const COVER_ENTROPY_FLOOR_REJECT_MAX: f64 = 0.50;

/// Mean train records per occupied full graded-code key above which the code
/// has stopped separating records. A code that resolves individual contexts
/// keeps this near the per-key evidence the support gate wants (single
/// digits); a saturated code piles unrelated records onto shared keys.
const CODE_RECORDS_PER_KEY_MAX: f64 = 32.0;

/// Minimum share of train records that reach the context-codebook k-means.
/// Below this the codebook is fit to a vanishing slice of the corpus, so
/// growing the corpus cannot make the code finer.
const CODE_TRAIN_SAMPLE_FRACTION_MIN: f64 = 0.05;

/// Share of held-out records resolving at the full code (occupied key with
/// store evidence at or above [`EXCT_SUPPORT_MIN`]) above which strict misses
/// become too rare for Gate C to measure generalization from.
const EXCT_SUPPORTED_RECORD_FRACTION_MAX: f64 = 0.90;

/// Share of FWDA / NGRAM rows clipped by their entry cap above which the cap,
/// rather than the observed evidence, is setting the row width.
const ROW_TRUNCATION_FRACTION_MAX: f64 = 0.10;

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
            .replace('_', "")
            .parse()
            .unwrap_or_else(|_| panic!("{name} must be an integer, got {value:?}")),
        Err(_) => default,
    }
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE")
    )
}

/// `PASS` when `value` stays on the healthy side of `limit`, `SATURATED`
/// otherwise. `higher_is_worse` picks which side that is.
fn verdict(value: f64, limit: f64, higher_is_worse: bool) -> &'static str {
    let bad = if higher_is_worse {
        value > limit
    } else {
        value < limit
    };
    if bad {
        "SATURATED"
    } else {
        "PASS"
    }
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Occupancy bucket edges of the EXCT histogram, in construction records per
/// full-code key. The `EXCT_SUPPORT_MIN` boundary falls on a bucket edge so
/// the supported / unsupported split is readable off the histogram.
const EXCT_BUCKETS: [u32; 7] = [1, 2, 3, 5, 10, 100, 1000];

fn bucket_label(index: usize) -> String {
    let low = EXCT_BUCKETS[index];
    match EXCT_BUCKETS.get(index + 1) {
        Some(&high) if high == low + 1 => format!("{low}"),
        Some(&high) => format!("{low}-{}", high - 1),
        None => format!("{low}+"),
    }
}

/// COVER section: induce the cover and report realized region capacity.
fn report_cover(observations: &[Observation], threads: usize) {
    println!("\n== COVER (uor-r4-graph-compiler::induction) ==");
    let config = CoverConfig {
        threads: threads as u32,
        ..CoverConfig::default()
    };
    // The observation vector is built once per run and shared with the
    // FWDA/NGRAM section; it is in train-position order, so the cap is the
    // same prefix `R4_CAP_COVER_MAX_TRAIN` selected when this section built
    // its own copy.
    let max_train = env_usize("R4_CAP_COVER_MAX_TRAIN", 0);
    let train = if max_train > 0 && observations.len() > max_train {
        println!(
            "  CAP BOUND: train positions {} -> {max_train} (R4_CAP_COVER_MAX_TRAIN)",
            observations.len()
        );
        &observations[..max_train]
    } else {
        observations
    };
    println!("  train observations: {}", train.len());
    let n = train.len();
    println!(
        "  config: depths={} k0={} (effective {}) regions_budget={} (effective {}) \
         min_support={} criterion={:?} entropy_gain_bits={}",
        config.depths,
        config.k0,
        config.effective_k0(n),
        config.regions_budget,
        config.effective_regions_budget(n),
        config.min_support,
        config.split_criterion,
        config.entropy_gain_bits
    );
    let induced = induction::induce_cover(train, &config, "i432-artifact", "i432-corpus")
        .expect("cover induction");
    let cover = &induced.cover;

    let mut per_depth: BTreeMap<u8, usize> = BTreeMap::new();
    let mut leaves = 0usize;
    let mut leaf_support: u64 = 0;
    for region in &cover.regions {
        *per_depth.entry(region.depth).or_insert(0) += 1;
        if region.children.is_empty() {
            leaves += 1;
            leaf_support += u64::from(region.support);
        }
    }
    println!(
        "  regions: {} total, max depth {}",
        cover.regions.len(),
        cover.max_depth
    );
    for (depth, count) in &per_depth {
        println!("    depth {depth}: {count} region(s)");
    }
    let nominal = config.effective_regions_budget(n);
    println!(
        "  region budget occupancy: {}/{} ({:.4})",
        cover.regions.len(),
        nominal,
        ratio(cover.regions.len() as u64, nominal as u64)
    );
    let obs_per_region = if cover.regions.is_empty() {
        0.0
    } else {
        n as f64 / cover.regions.len() as f64
    };
    let obs_per_leaf = if leaves == 0 {
        0.0
    } else {
        leaf_support as f64 / leaves as f64
    };
    println!("  mean train observations per region: {obs_per_region:.1}");
    println!("  mean train observations per leaf region: {obs_per_leaf:.1} ({leaves} leaves)");

    let mut decisions: BTreeMap<String, usize> = BTreeMap::new();
    let mut gain_sum = 0.0f64;
    let mut gain_max = f64::MIN;
    for audit in &induced.decision_trace {
        *decisions.entry(audit.decision.clone()).or_insert(0) += 1;
        gain_sum += audit.entropy_gain_bits;
        if audit.entropy_gain_bits > gain_max {
            gain_max = audit.entropy_gain_bits;
        }
    }
    let audited = induced.decision_trace.len();
    println!("  split candidates audited: {audited}");
    for (decision, count) in &decisions {
        println!(
            "    {decision}: {count} ({:.4})",
            ratio(*count as u64, audited as u64)
        );
    }
    if audited > 0 {
        println!(
            "  entropy gain over audited candidates: mean {:.4} bits, max {:.4} bits, \
             floor {:.4} bits",
            gain_sum / audited as f64,
            gain_max,
            config.entropy_gain_bits
        );
    }
    let floor_rejects = decisions.get("keep:entropy_floor").copied().unwrap_or(0);
    let floor_fraction = ratio(floor_rejects as u64, audited as u64);
    println!(
        "SATURATION VERDICT cover.observations_per_region: {} \
         (mean {obs_per_region:.1} vs limit {COVER_OBS_PER_REGION_MAX:.0})",
        verdict(obs_per_region, COVER_OBS_PER_REGION_MAX, true)
    );
    println!(
        "SATURATION VERDICT cover.entropy_floor_rejects: {} \
         (share {floor_fraction:.4} vs limit {COVER_ENTROPY_FLOOR_REJECT_MAX:.2})",
        verdict(floor_fraction, COVER_ENTROPY_FLOOR_REJECT_MAX, true)
    );
}

/// Default number of held-out records drawn for the sampled resolution
/// rate. Chosen so the binomial standard error of a rate near 0.99 is
/// about 0.03pp — two orders finer than the 90 percent threshold the
/// verdict is decided against, at a twentieth of the code-assignment work
/// the held-out split would otherwise cost.
const CAPACITY_SAMPLE_DEFAULT: usize = 100_000;

/// Environment override for that sample size. `0` restores the census.
const CAPACITY_SAMPLE_ENV: &str = "R4_CAPACITY_SAMPLE";

/// Opt-in override that ALSO subsamples the construction-split code pass.
/// Default `0`, meaning the construction split is a census, because every
/// key-derived statistic in this instrument is biased by a subsample of it
/// (module docs). It exists so the bias can be reproduced and measured,
/// and the run labels every affected number `BIASED ESTIMATE`.
const CAPACITY_TRAIN_SAMPLE_ENV: &str = "R4_CAPACITY_TRAIN_SAMPLE";

/// Deterministic stride subsample of `positions`: entry `k` of the sample
/// is `positions[k * len / n]`. No RNG — the same corpus and the same `n`
/// always draw the same records, so a sampled run is reproducible and
/// diffable, the selection stays in corpus order, and because the held-out
/// list is in corpus order an even stride spreads the draw across every
/// story instead of taking one contiguous block.
fn stride_sample(positions: &[usize], n: usize) -> Vec<usize> {
    let len = positions.len();
    if n == 0 || n >= len {
        return positions.to_vec();
    }
    (0..n).map(|k| positions[k * len / n]).collect()
}

/// Binomial standard error of a rate `p` measured over `n` records.
fn standard_error(p: f64, n: u64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    (p * (1.0 - p) / n as f64).sqrt()
}

/// The per-record graded-code assignment pass, in parallel over corpus
/// positions and reduced in input order (`collect` on an indexed Rayon
/// iterator), so the code vector is thread-count independent — the same
/// discipline `evaluate_gate_c` uses for its held-out rows.
///
/// This calls `assign_code_for_bundle`, the allocation-free per-stage
/// argmax, whose tie rule is documented to yield the same primary code as
/// `assign_for_bundle`; the membership beam the latter materializes is
/// discarded by every caller here.
fn assign_codes(
    artifacts: &compiler::Compiled,
    corpus: &Corpus,
    positions: &[usize],
) -> Vec<[u8; STAGES]> {
    let rotations = runtime::derive_rotations();
    positions
        .par_iter()
        .map(|&i| {
            let bundle = runtime::bundle_plain(artifacts, &rotations, corpus, i);
            runtime::assign_code_for_bundle(artifacts, &bundle)
        })
        .collect()
}

/// One occupied full-code key of the construction split: the number of
/// construction records that landed on it and the teacher weight those
/// records carry. Recovered exactly from the sorted code vector, which is
/// what the five-level evidence store would have held.
struct KeyAggregate {
    code: [u8; STAGES],
    records: u64,
    weight: u64,
}

/// Group the construction split's per-record codes into per-key
/// aggregates, sorted by code. Exact: every construction record is
/// counted, and the sort is the whole cost.
fn aggregate_keys(mut coded: Vec<([u8; STAGES], u64)>) -> Vec<KeyAggregate> {
    coded.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut keys: Vec<KeyAggregate> = Vec::new();
    for (code, weight) in coded {
        match keys.last_mut() {
            Some(last) if last.code == code => {
                last.records += 1;
                last.weight += weight;
            }
            _ => keys.push(KeyAggregate {
                code,
                records: 1,
                weight,
            }),
        }
    }
    keys
}

/// GRADED CODE + EXCT sections: prefix-key occupancy and evidence
/// histogram. Every number here is exact except the held-out resolution
/// rate, which is the one sampled statistic (module docs).
fn report_code_and_exct(corpus: &Corpus, artifacts: &compiler::Compiled) {
    println!("\n== GRADED CODE (uor-r4-core::transformerless) ==");
    // The construction split the store itself uses.
    let train_cut = compiler::train_cut(corpus);
    let train_positions: Vec<usize> = (0..corpus.n)
        .filter(|&i| corpus.story[i] < train_cut)
        .collect();
    let held_positions: Vec<usize> = (0..corpus.n)
        .filter(|&i| corpus.story[i] >= train_cut)
        .collect();

    // The construction-split code pass is EXACT by default. The opt-in
    // subsample below exists to reproduce, not to hide, the bias it
    // creates: see the module docs and the banner this prints.
    let train_requested = env_usize(CAPACITY_TRAIN_SAMPLE_ENV, 0);
    let train_sample = stride_sample(&train_positions, train_requested);
    let train_sampled = train_sample.len() < train_positions.len();
    let train_records = train_sample.len() as u64;
    if train_sampled {
        println!(
            "  BIASED FAST LOOK: {CAPACITY_TRAIN_SAMPLE_ENV}={train_requested} of {} \
             construction records ({:.5} of the split). Every key-derived number \
             below is a BIASED ESTIMATE, not a census: the key set is drawn from a \
             subsample, so occupied keys read LOW, mean records per key reads LOW, \
             the occupancy histogram is thinned toward its low buckets, and the \
             held-out resolution rate reads LOW because it probes an incomplete key \
             set. Saturation verdicts from this mode are NOT trustworthy.",
            train_positions.len(),
            ratio(train_records, train_positions.len() as u64)
        );
    }
    let label = if train_sampled {
        "BIASED ESTIMATE"
    } else {
        "exact"
    };

    let started = Instant::now();
    let train_codes = assign_codes(artifacts, corpus, &train_sample);
    println!(
        "  construction-split code pass: {} record(s) in {:.1}s ({label}, rayon)",
        train_codes.len(),
        started.elapsed().as_secs_f64()
    );
    let started = Instant::now();
    let coded: Vec<([u8; STAGES], u64)> = train_codes
        .iter()
        .zip(&train_sample)
        .map(|(&code, &i)| {
            let weight: u64 = corpus.top_weights[i].iter().map(|&w| u64::from(w)).sum();
            (code, weight)
        })
        .collect();
    let keys = aggregate_keys(coded);
    println!(
        "  key aggregation: {} occupied full-code key(s) in {:.1}s ({label})",
        keys.len(),
        started.elapsed().as_secs_f64()
    );

    println!(
        "  corpus: {} records, {} stories; construction-split records coded \
         {train_records} of {} ({label})",
        corpus.n,
        corpus.stories,
        train_positions.len()
    );
    println!("  STAGES={STAGES} K={K}");

    // Occupied keys at prefix level L are the distinct L-byte prefixes of
    // the occupied full codes, which is what the store's level-L map held.
    let mut nominal: f64 = 1.0;
    for level in 0..=STAGES {
        if level > 0 {
            nominal *= K as f64;
        }
        let occupied_level = if keys.is_empty() {
            0
        } else if level == 0 {
            1
        } else {
            1 + keys
                .windows(2)
                .filter(|pair| pair[0].code[..level] != pair[1].code[..level])
                .count()
        };
        println!(
            "  level {level}: {occupied_level} occupied key(s) of {nominal:.3e} nominal \
             ({:.3e} occupancy)",
            occupied_level as f64 / nominal
        );
    }

    let occupied = keys.len() as u64;
    let records_per_key = ratio(train_records, occupied);
    println!("  mean construction records per occupied full-code key: {records_per_key:.2}");

    // The codebook that produces these codes is fit on a bounded subsample,
    // independent of the corpus: report the realized fraction.
    let ctx_sample = compiler::capacity_override_usize("R4_CTX_SAMPLE", 50_000);
    let rvq_cap = compiler::capacity_override_usize("R4_RVQ_SAMPLE_CAP", 10_000);
    let kmeans_sample = ctx_sample.min(rvq_cap) as u64;
    let sample_fraction = ratio(kmeans_sample, train_records);
    println!(
        "  context codebook k-means training vectors: {kmeans_sample} \
         (CTX_SAMPLE={ctx_sample} subsampled to RVQ_SAMPLE_CAP={rvq_cap}) \
         = {sample_fraction:.5} of the construction split"
    );
    println!(
        "  vectors per centroid at the deepest stage: {:.1} ({kmeans_sample} / {K})",
        kmeans_sample as f64 / K as f64
    );
    println!(
        "SATURATION VERDICT code.records_per_full_key: {} \
         ({label} {records_per_key:.2} vs limit {CODE_RECORDS_PER_KEY_MAX:.0})",
        verdict(records_per_key, CODE_RECORDS_PER_KEY_MAX, true)
    );
    println!(
        "SATURATION VERDICT code.codebook_sample_fraction: {} \
         ({label} {sample_fraction:.5} vs floor {CODE_TRAIN_SAMPLE_FRACTION_MIN:.2})",
        verdict(sample_fraction, CODE_TRAIN_SAMPLE_FRACTION_MIN, false)
    );

    println!("\n== EXCT (full-code exact-context occupancy) ==");
    // Per-key RECORD counts. The per-key totals are teacher weight units
    // (about 100 per record), so they are not a record census and cannot be
    // compared across corpora; the record histogram is the scale-sensitive
    // quantity. Both are reported, and both are exact — see the module docs
    // on why neither survives a subsample of the construction split.
    let mut histogram = [0u64; EXCT_BUCKETS.len()];
    let mut singleton_keys: u64 = 0;
    let mut weight_mass: u64 = 0;
    let mut supported_keys: u64 = 0;
    for key in &keys {
        if key.records == 1 {
            singleton_keys += 1;
        }
        weight_mass += key.weight;
        if key.weight >= u64::from(EXCT_SUPPORT_MIN) {
            supported_keys += 1;
        }
        let mut index = 0usize;
        for (i, &edge) in EXCT_BUCKETS.iter().enumerate() {
            if key.records >= u64::from(edge) {
                index = i;
            }
        }
        histogram[index] += 1;
    }
    println!(
        "  occupied full-code keys: {occupied}; store evidence (teacher weight units): \
         {weight_mass}"
    );
    println!("  construction-record occupancy histogram (records per full-code key):");
    for (index, &count) in histogram.iter().enumerate() {
        println!(
            "    records {:>6}: {count} key(s) ({:.4})",
            bucket_label(index),
            ratio(count, occupied)
        );
    }
    println!(
        "  singleton keys (exactly one construction record): {singleton_keys} ({:.4})",
        ratio(singleton_keys, occupied)
    );
    println!(
        "  keys clearing EXCT_SUPPORT_MIN={EXCT_SUPPORT_MIN} on store evidence: \
         {supported_keys} ({:.4})",
        ratio(supported_keys, occupied)
    );

    // The quantity Gate C actually depends on: does a held-out record's full
    // code land on an occupied, supported construction key? A held-out set
    // that almost always resolves exactly cannot exhibit a strict miss, and
    // without strict misses there is no generalization to measure.
    //
    // This is the instrument's one SAMPLED statistic. It is the mean of a
    // per-record indicator, so a stride subsample of the held-out list
    // estimates it without bias; the key set the indicator probes is the
    // exact one built above, which is what keeps that true.
    let requested = env_usize(CAPACITY_SAMPLE_ENV, CAPACITY_SAMPLE_DEFAULT);
    let held_sample = stride_sample(&held_positions, requested);
    let sampled = held_sample.len() < held_positions.len();
    let started = Instant::now();
    let held_codes = assign_codes(artifacts, corpus, &held_sample);
    let held_total = held_codes.len() as u64;
    let held_resolved = held_codes
        .par_iter()
        .filter(|code| {
            keys.binary_search_by(|key| key.code.cmp(code))
                .map(|index| keys[index].weight >= u64::from(EXCT_SUPPORT_MIN))
                .unwrap_or(false)
        })
        .count() as u64;
    let supported_fraction = ratio(held_resolved, held_total);
    let error = standard_error(supported_fraction, held_total);
    println!(
        "  held-out records: {} total in the split; {held_total} scored in {:.1}s ({})",
        held_positions.len(),
        started.elapsed().as_secs_f64(),
        if sampled {
            format!("SAMPLED, {CAPACITY_SAMPLE_ENV}={requested}")
        } else {
            "census".to_owned()
        }
    );
    println!(
        "  resolving at the FULL code: {held_resolved} ({supported_fraction:.4}); \
         strict miss {:.4}; standard error {error:.5} over n={held_total}",
        1.0 - supported_fraction
    );
    println!(
        "SATURATION VERDICT exct.supported_record_fraction: {} \
         ({} {supported_fraction:.4} +/- {error:.5} vs limit \
         {EXCT_SUPPORTED_RECORD_FRACTION_MAX:.2}) \
         - above the limit a strict miss is too rare to measure generalization from",
        verdict(supported_fraction, EXCT_SUPPORTED_RECORD_FRACTION_MAX, true),
        if train_sampled {
            "BIASED ESTIMATE"
        } else if sampled {
            "ESTIMATE"
        } else {
            "fraction"
        }
    );
}

/// FWDA + NGRAM sections: row counts and entry-cap truncation shares.
fn report_tables(corpus: &Corpus, artifacts: &compiler::Compiled, train: &[Observation]) {
    println!("\n== FWDA + NGRAM (uor-r4-graph-certify::score) ==");
    println!("  train observations: {}", train.len());

    let rows = compile_forward_anchor_rows(corpus, train);
    let mut total_sum: u64 = 0;
    let mut truncated: u64 = 0;
    let mut max_total: u32 = 0;
    for row in &rows {
        total_sum += u64::from(row.total);
        max_total = max_total.max(row.total);
        if row.entries.len() >= FWDA_ENTRY_CAP {
            truncated += 1;
        }
    }
    let row_count = rows.len() as u64;
    let mean_total = ratio(total_sum, row_count);
    let fwda_truncation = ratio(truncated, row_count);
    println!("  FWDA rows: {row_count} (entry cap {FWDA_ENTRY_CAP})");
    println!("  FWDA mean row total: {mean_total:.2}, max row total: {max_total}");
    println!("  FWDA rows at the entry cap: {truncated} ({fwda_truncation:.4})");
    println!(
        "SATURATION VERDICT fwda.row_truncation: {} \
         (share {fwda_truncation:.4} vs limit {ROW_TRUNCATION_FRACTION_MAX:.2})",
        verdict(fwda_truncation, ROW_TRUNCATION_FRACTION_MAX, true)
    );

    let config = ScoreConfig::default();
    let vocab = u32::try_from(artifacts.token_codes.len() / STAGES).unwrap_or(u32::MAX);
    let context_rows = compile_context_rows(corpus, train, vocab, &config);
    let mut bigrams: u64 = 0;
    let mut trigrams: u64 = 0;
    let mut ngram_truncated: u64 = 0;
    for row in &context_rows {
        if row.context_len == 1 {
            bigrams += 1;
        } else {
            trigrams += 1;
        }
        if row.entries.len() >= config.context_entries {
            ngram_truncated += 1;
        }
    }
    let ngram_rows = context_rows.len() as u64;
    let ngram_truncation = ratio(ngram_truncated, ngram_rows);
    println!(
        "  NGRAM rows: {ngram_rows} ({bigrams} bigram, {trigrams} trigram); \
         entry cap {} (default {DEFAULT_CONTEXT_ENTRIES})",
        config.context_entries
    );
    println!("  NGRAM rows at the entry cap: {ngram_truncated} ({ngram_truncation:.4})");
    println!(
        "SATURATION VERDICT ngram.row_truncation: {} \
         (share {ngram_truncation:.4} vs limit {ROW_TRUNCATION_FRACTION_MAX:.2})",
        verdict(ngram_truncation, ROW_TRUNCATION_FRACTION_MAX, true)
    );
}

#[test]
#[ignore = "#460 capacity measurement; run explicitly with --ignored"]
fn capacity_scaling() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let threads = env_usize("R4_CAP_THREADS", 2).max(1);

    let corpus = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let artifacts = compiler::load_artifacts_from(&art_path).expect("artifacts");

    println!("#460 capacity-scaling instrumentation");
    println!("  corpus:    {meta_path} / {recs_path}");
    println!("  artifacts: {art_path}");
    println!("  records:   {} in {} stories", corpus.n, corpus.stories);
    println!("  threads:   {threads}");

    let overall = Instant::now();
    report_code_and_exct(&corpus, &artifacts);

    // Built once and shared by the FWDA/NGRAM and COVER sections, which
    // both used to build their own identical copy.
    let (train_positions, _held) = induction::split_positions(&corpus);
    let started = Instant::now();
    let observations =
        induction::build_observations_with_threads(&artifacts, &corpus, &train_positions, threads)
            .expect("train observations");
    println!(
        "\n  train observations: {} (built once in {:.1}s, shared by FWDA/NGRAM and COVER)",
        observations.len(),
        started.elapsed().as_secs_f64()
    );

    report_tables(&corpus, &artifacts, &observations);
    if env_flag("R4_CAP_SKIP_COVER") {
        println!("\n== COVER == skipped (R4_CAP_SKIP_COVER)");
    } else {
        report_cover(&observations, threads);
    }
    println!(
        "\n  total instrument wall clock: {:.1}s",
        overall.elapsed().as_secs_f64()
    );
    println!("\n#460 capacity-scaling instrumentation complete");
}
