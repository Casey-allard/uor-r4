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
//! observations fed to induction, `0` = unlimited).

use std::collections::BTreeMap;
use std::time::Instant;

use uor_r4_core::transformerless::compiler::{self, Corpus, K, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::{
    compile_context_rows, compile_forward_anchor_rows, ScoreConfig, DEFAULT_CONTEXT_ENTRIES,
    FWDA_ENTRY_CAP,
};
use uor_r4_graph_certify::score_runtime::EXCT_SUPPORT_MIN;
use uor_r4_graph_compiler::induction::{self, CoverConfig};

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
fn report_cover(corpus: &Corpus, artifacts: &compiler::Compiled, threads: usize) {
    println!("\n== COVER (uor-r4-graph-compiler::induction) ==");
    let config = CoverConfig {
        threads: threads as u32,
        ..CoverConfig::default()
    };
    let (train_positions, _held) = induction::split_positions(corpus);
    let max_train = env_usize("R4_CAP_COVER_MAX_TRAIN", 0);
    let mut train_positions = train_positions;
    if max_train > 0 && train_positions.len() > max_train {
        println!(
            "  CAP BOUND: train positions {} -> {max_train} (R4_CAP_COVER_MAX_TRAIN)",
            train_positions.len()
        );
        train_positions.truncate(max_train);
    }
    let started = Instant::now();
    let train =
        induction::build_observations_with_threads(artifacts, corpus, &train_positions, threads)
            .expect("train observations");
    println!(
        "  train observations: {} (built in {:.1}s)",
        train.len(),
        started.elapsed().as_secs_f64()
    );
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
    let induced = induction::induce_cover(&train, &config, "i432-artifact", "i432-corpus")
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

/// GRADED CODE + EXCT sections: prefix-key occupancy and evidence histogram.
fn report_code_and_exct(corpus: &Corpus, artifacts: &compiler::Compiled, threads: usize) {
    println!("\n== GRADED CODE (uor-r4-core::transformerless) ==");
    let started = Instant::now();
    let (store, codes) =
        runtime::build_store_with_threads(artifacts, corpus, threads).expect("store");
    println!("  store built in {:.1}s", started.elapsed().as_secs_f64());

    // The construction split the store itself uses.
    let train_cut = compiler::train_cut(corpus);
    let mut train_records: u64 = 0;
    for &story in corpus.story.iter().take(corpus.n) {
        if story < train_cut {
            train_records += 1;
        }
    }
    println!(
        "  corpus: {} records, {} stories; construction-split records {train_records}",
        corpus.n, corpus.stories
    );
    println!("  STAGES={STAGES} K={K}");

    let mut nominal: f64 = 1.0;
    for (level, cells) in store.iter().enumerate() {
        if level > 0 {
            nominal *= K as f64;
        }
        println!(
            "  level {level}: {} occupied key(s) of {nominal:.3e} nominal ({:.3e} occupancy)",
            cells.len(),
            cells.len() as f64 / nominal
        );
    }

    let full = &store[STAGES];
    let occupied = full.len() as u64;
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
         (mean {records_per_key:.2} vs limit {CODE_RECORDS_PER_KEY_MAX:.0})",
        verdict(records_per_key, CODE_RECORDS_PER_KEY_MAX, true)
    );
    println!(
        "SATURATION VERDICT code.codebook_sample_fraction: {} \
         (fraction {sample_fraction:.5} vs floor {CODE_TRAIN_SAMPLE_FRACTION_MIN:.2})",
        verdict(sample_fraction, CODE_TRAIN_SAMPLE_FRACTION_MIN, false)
    );

    println!("\n== EXCT (full-code exact-context occupancy) ==");
    // Per-key RECORD counts. The store's own totals are teacher weight units
    // (about 100 per record), so they are not a record census and cannot be
    // compared across corpora; the record histogram is the scale-sensitive
    // quantity. Both are reported.
    let mut key_records: BTreeMap<&[u8; STAGES], u64> = BTreeMap::new();
    for (i, code) in codes.iter().enumerate().take(corpus.n) {
        if corpus.story[i] < train_cut {
            *key_records.entry(code).or_insert(0) += 1;
        }
    }
    let mut histogram = [0u64; EXCT_BUCKETS.len()];
    let mut singleton_keys: u64 = 0;
    for &records in key_records.values() {
        if records == 1 {
            singleton_keys += 1;
        }
        let mut index = 0usize;
        for (i, &edge) in EXCT_BUCKETS.iter().enumerate() {
            if records >= u64::from(edge) {
                index = i;
            }
        }
        histogram[index] += 1;
    }
    let mut weight_mass: u64 = 0;
    let mut supported_keys: u64 = 0;
    for distribution in full.values() {
        let total: u64 = distribution.values().map(|&c| u64::from(c)).sum();
        weight_mass += total;
        if total >= u64::from(EXCT_SUPPORT_MIN) {
            supported_keys += 1;
        }
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
    let mut held_total: u64 = 0;
    let mut held_resolved: u64 = 0;
    for (i, code) in codes.iter().enumerate().take(corpus.n) {
        if corpus.story[i] < train_cut {
            continue;
        }
        held_total += 1;
        if let Some(distribution) = full.get(code.as_slice()) {
            let total: u64 = distribution.values().map(|&c| u64::from(c)).sum();
            if total >= u64::from(EXCT_SUPPORT_MIN) {
                held_resolved += 1;
            }
        }
    }
    let supported_fraction = ratio(held_resolved, held_total);
    println!(
        "  held-out records: {held_total}; resolving at the FULL code: {held_resolved} \
         ({supported_fraction:.4}); strict miss {:.4}",
        1.0 - supported_fraction
    );
    println!(
        "SATURATION VERDICT exct.supported_record_fraction: {} \
         (fraction {supported_fraction:.4} vs limit {EXCT_SUPPORTED_RECORD_FRACTION_MAX:.2}) \
         - above the limit a strict miss is too rare to measure generalization from",
        verdict(supported_fraction, EXCT_SUPPORTED_RECORD_FRACTION_MAX, true)
    );
}

/// FWDA + NGRAM sections: row counts and entry-cap truncation shares.
fn report_tables(corpus: &Corpus, artifacts: &compiler::Compiled, threads: usize) {
    println!("\n== FWDA + NGRAM (uor-r4-graph-certify::score) ==");
    let (train_positions, _held) = induction::split_positions(corpus);
    let train =
        induction::build_observations_with_threads(artifacts, corpus, &train_positions, threads)
            .expect("train observations");
    println!("  train observations: {}", train.len());

    let rows = compile_forward_anchor_rows(corpus, &train);
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
    let context_rows = compile_context_rows(corpus, &train, vocab, &config);
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

    report_code_and_exct(&corpus, &artifacts, threads);
    report_tables(&corpus, &artifacts, threads);
    if env_flag("R4_CAP_SKIP_COVER") {
        println!("\n== COVER == skipped (R4_CAP_SKIP_COVER)");
    } else {
        report_cover(&corpus, &artifacts, threads);
    }
    println!("\n#460 capacity-scaling instrumentation complete");
}
