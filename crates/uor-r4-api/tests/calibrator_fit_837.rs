//! Artifact-only confidence-calibrator fit against powered selective-risk
//! controls (#837, item B of S2 tracker #823).
//!
//! Companion record: `docs/selective_calibration_837.md`. Executes the #837
//! study on the #833 canonical bundle under the constitution frozen by #838
//! (`docs/selective_prediction_spec_838.md`): thresholds are chosen ONLY on
//! the calibration partition by the frozen rule (maximize coverage subject to
//! a false-answer UCB95 bound), and the untouched test partition is evaluated
//! exactly once for the selected candidate. Everything is offline
//! compiler/certifier scope (#830): the reference predictor is the
//! suffix-table baseline of the #834/#875 harnesses (whose 246.6‰ held-out
//! top-1 this harness must REPRODUCE before any reading is accepted), not the
//! deployed engine; no teacher, remote model, clock, or network is consulted
//! at inference — every feature is an integer read of the artifact tables.
//!
//! ## What is measured
//!
//! Risk here is teacher-grounded: a served position is a *false answer* when
//! the reference prediction differs from the recorded teacher argmax in the
//! #833 bundle. The primary gate is the #838 frozen release operating point —
//! false-answer UCB95 ≤ 10‰ among served with coverage ≥ the pre-declared
//! floor — with the research point (≤ 50‰) reported alongside. Selection uses
//! the pre-registered rule; `NO CALIBRATOR ESTABLISHED` is a legitimate
//! outcome recorded with every negative arm.
//!
//! ## Phases
//!
//! * `calibrator_instrument_837` (ignored) — the binding cheap instrument:
//!   feature availability/variance on the FIT partition only, the
//!   observability ceiling (what fraction of baseline errors expose a
//!   low-confidence signal), and the partition/leakage report. Run FIRST; its
//!   numbers go into the run contract posted to #837 before the full run.
//! * `calibrator_fit_run_837` (ignored) — the full study: fit → calibrate →
//!   one test evaluation → CID-bound record.
//! * The non-ignored tests are the committed small-fixture teeth: extraction,
//!   quantization/saturation, thresholding, shuffled/planted controls, the
//!   integer-vs-f64 differential, leakage, and double-run determinism.
//!
//! Run:
//!   cargo test -p uor-r4-api --release --test calibrator_fit_837 -- --ignored --nocapture
//! Env: R4_CAUSAL_BUNDLE (bundle dir; corpus is read from it).

#![allow(clippy::doc_lazy_continuation)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_api::capability_suite::{compute_cid, detect_document_leakage};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

/// Per-key cap on retained argmax (the bounded-table convention of #875/#834).
const CAP: usize = 64;
const CAND_SUFFIX: usize = 32;
/// The #875/#891 recorded suffix baseline this harness must reproduce (‰).
const REPRO_BASE_PERMILLE: f64 = 246.6;

/// The #838 frozen operating-point targets (‰) — false-answer UCB95 bounds.
const RELEASE_UCB_PERMILLE: u32 = 10;
const RESEARCH_UCB_PERMILLE: u32 = 50;
/// Pre-declared useful-coverage floors (‰ of all positions served).
const RELEASE_COVERAGE_FLOOR_PERMILLE: u32 = 20;
const RESEARCH_COVERAGE_FLOOR_PERMILLE: u32 = 50;
/// Positions re-scored by the in-run double-run determinism check.
const DOUBLE_RUN_N: usize = 2_000;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn bundle_root() -> PathBuf {
    match std::env::var_os("R4_CAUSAL_BUNDLE") {
        Some(v) => PathBuf::from(v),
        None => repo_root()
            .join(".uor-models")
            .join("compiled")
            .join("smollm2-360m-broad-clean"),
    }
}

/// The frozen conservative UCB95 reference arithmetic (‰) of #838 §9:
/// `(1000·k + 3000) / n`, integer ceiling.
fn ucb95_permille(failures: u64, n: u64) -> u32 {
    assert!(
        n > 0,
        "UCB over an empty sample is UNAVAILABLE, never a value"
    );
    (1000_u64
        .saturating_mul(failures)
        .saturating_add(3_000)
        .div_ceil(n)) as u32
}

/// Counts of teacher argmax under one key, capped to the top `CAP` by count
/// (ties to the smaller token id), total retained — the #875 construction.
#[derive(Default, Clone)]
struct Counter {
    map: HashMap<u32, u32>,
}

impl Counter {
    fn bump(&mut self, tok: u32) {
        *self.map.entry(tok).or_insert(0) += 1;
    }
    fn cap_to_top(&mut self, cap: usize) {
        if self.map.len() <= cap {
            return;
        }
        let mut v: Vec<(u32, u32)> = self.map.drain().collect();
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(cap);
        self.map = v.into_iter().collect();
    }
    fn total(&self) -> u64 {
        self.map.values().map(|&c| c as u64).sum()
    }
}

fn suffix_key(window: &[u32]) -> (u32, u32) {
    let n = window.len();
    if n >= 2 {
        (window[n - 2], window[n - 1])
    } else if n == 1 {
        (u32::MAX, window[0])
    } else {
        (u32::MAX, u32::MAX)
    }
}

/// Argmax by score, canonical tie-break (score desc, id asc).
fn argmax(scores: &HashMap<u32, f64>) -> u32 {
    let mut best_id = u32::MAX;
    let mut best = f64::NEG_INFINITY;
    for (&id, &s) in scores {
        if s > best || (s == best && id < best_id) {
            best = s;
            best_id = id;
        }
    }
    best_id
}

// --- artifact-observable features ---------------------------------------------

/// The integer feature vector a calibrator may read at serve time. Every
/// field is computable from the artifact tables and the input window alone —
/// no teacher, no label, no clock (#837 non-goal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Features {
    /// The suffix key exists in the fitted table (the D4-novelty inverse).
    present: bool,
    /// Total evidence count under the suffix key (support/provenance mass).
    total: u32,
    /// (top1 − top2) · 1000 / total under the suffix key (candidate margin).
    margin_pm: u32,
    /// top1 · 1000 / total (evidence concentration; an entropy inverse).
    top1_pm: u32,
    /// Distinct retained continuations under the key (branching factor).
    distinct: u32,
    /// Window tokens whose content-table argmax agrees with the suffix top-1
    /// (route agreement / provenance diversity).
    support: u32,
    /// Window tokens whose content-table argmax is a different token with a
    /// strictly larger count share (a contradiction indicator).
    disagree: u32,
    /// The whole-window content aggregate argmax equals the suffix top-1.
    agree: bool,
}

struct Tables {
    suffix_next: HashMap<(u32, u32), Counter>,
    content_next: HashMap<u32, Counter>,
    marginal_tok: u32,
}

impl Tables {
    /// The reference predictor: the #875/#834 suffix baseline verbatim —
    /// argmax of the suffix rate over the top suffix candidates plus the
    /// marginal token. A pure function of the suffix key.
    fn base_predict(&self, w: &[u32]) -> u32 {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let sfx_total = sfx.map(|c| c.total()).unwrap_or(0).max(1) as f64;
        let mut suffix_cands: Vec<u32> = Vec::new();
        if let Some(c) = sfx {
            let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
            v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            suffix_cands.extend(v.into_iter().take(CAND_SUFFIX).map(|(k, _)| k));
        }
        suffix_cands.push(self.marginal_tok);
        suffix_cands.sort_unstable();
        suffix_cands.dedup();
        let mut base_scores: HashMap<u32, f64> = HashMap::new();
        for &c in &suffix_cands {
            let rate = sfx.and_then(|s| s.map.get(&c)).copied().unwrap_or(0) as f64 / sfx_total;
            base_scores.insert(c, rate);
        }
        argmax(&base_scores)
    }

    /// Extract the serve-time feature vector for a window.
    fn features(&self, w: &[u32]) -> Features {
        let sfx = self.suffix_next.get(&suffix_key(w));
        let (present, total, margin_pm, top1_pm, distinct, top1_tok) = match sfx {
            None => (false, 0u32, 0u32, 0u32, 0u32, self.marginal_tok),
            Some(c) => {
                let mut v: Vec<(u32, u32)> = c.map.iter().map(|(&k, &n)| (k, n)).collect();
                v.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                let total = c.total().min(u64::from(u32::MAX)) as u32;
                let top1 = v.first().map(|&(_, n)| n).unwrap_or(0);
                let top2 = v.get(1).map(|&(_, n)| n).unwrap_or(0);
                let t = u64::from(total.max(1));
                let margin_pm = (u64::from(top1 - top2) * 1000 / t) as u32;
                let top1_pm = (u64::from(top1) * 1000 / t) as u32;
                let top1_tok = v.first().map(|&(k, _)| k).unwrap_or(self.marginal_tok);
                (true, total, margin_pm, top1_pm, v.len() as u32, top1_tok)
            }
        };

        let mut uniq: Vec<u32> = w.to_vec();
        uniq.sort_unstable();
        uniq.dedup();
        let mut support = 0u32;
        let mut disagree = 0u32;
        let mut content_rate: HashMap<u32, f64> = HashMap::new();
        let ncontent = uniq.len().max(1) as f64;
        for t in &uniq {
            if let Some(cn) = self.content_next.get(t) {
                let tot = cn.total().max(1) as f64;
                let mut best_tok = u32::MAX;
                let mut best_cnt = 0u32;
                for (&cand, &cnt) in &cn.map {
                    if cnt > best_cnt || (cnt == best_cnt && cand < best_tok) {
                        best_cnt = cnt;
                        best_tok = cand;
                    }
                    *content_rate.entry(cand).or_insert(0.0) += (f64::from(cnt) / tot) / ncontent;
                }
                if best_tok == top1_tok {
                    support += 1;
                } else if best_cnt > 0 {
                    disagree += 1;
                }
            }
        }
        let agree = !content_rate.is_empty() && argmax(&content_rate) == top1_tok;
        Features {
            present,
            total,
            margin_pm,
            top1_pm,
            distinct,
            support,
            disagree,
            agree,
        }
    }
}

// --- calibrator arms and controls ---------------------------------------------

/// Integer log2 bucket of an evidence count, clamped to 0..=15.
fn log2_bucket(total: u32) -> usize {
    (32 - total.leading_zeros()).min(15) as usize
}

/// Margin decile bucket, 0..=9.
fn margin_bucket(margin_pm: u32) -> usize {
    (margin_pm / 100).min(9) as usize
}

/// A bucket-table model fitted on the FIT partition: per
/// (support-bucket × margin-decile) cell, the empirical correct rate (‰) of
/// the reference prediction. Integer-only; 160 cells.
struct BucketTable {
    precision_pm: [[u32; 10]; 16],
}

impl BucketTable {
    fn fit(rows: &[(Features, bool)]) -> BucketTable {
        let mut correct = [[0u64; 10]; 16];
        let mut count = [[0u64; 10]; 16];
        for (f, ok) in rows {
            let b = log2_bucket(f.total);
            let m = margin_bucket(f.margin_pm);
            count[b][m] += 1;
            correct[b][m] += u64::from(*ok);
        }
        let mut precision_pm = [[0u32; 10]; 16];
        for b in 0..16 {
            for m in 0..10 {
                precision_pm[b][m] =
                    (correct[b][m] * 1000).checked_div(count[b][m]).unwrap_or(0) as u32;
            }
        }
        BucketTable { precision_pm }
    }

    fn score(&self, f: &Features) -> i64 {
        i64::from(self.precision_pm[log2_bucket(f.total)][margin_bucket(f.margin_pm)])
    }
}

/// The candidate arms (eligible for selection) and the frozen controls (never
/// eligible). Every arm maps a feature vector to an integer confidence score;
/// serving is `score ≥ θ` with θ chosen ONLY on the calibration partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Arm {
    // -- candidates, in pre-declared budget order (simpler first) --
    MarginThreshold,
    Top1RateThreshold,
    BucketModel,
    RichCombo,
    // -- fixed controls / nulls --
    CurrentD4,
    DistanceOnly,
    CountOnly,
    ConstantScore,
    InvertedMargin,
}

impl Arm {
    const CANDIDATES: [Arm; 4] = [
        Arm::MarginThreshold,
        Arm::Top1RateThreshold,
        Arm::BucketModel,
        Arm::RichCombo,
    ];

    fn label(self) -> &'static str {
        match self {
            Arm::MarginThreshold => "margin-threshold",
            Arm::Top1RateThreshold => "top1rate-threshold",
            Arm::BucketModel => "bucket-table",
            Arm::RichCombo => "rich-combo",
            Arm::CurrentD4 => "current-d4",
            Arm::DistanceOnly => "distance-only",
            Arm::CountOnly => "count-only",
            Arm::ConstantScore => "constant-score",
            Arm::InvertedMargin => "inverted-margin",
        }
    }

    /// Deployed-budget accounting: (extra bytes, feature reads, table reads).
    fn budget(self) -> (u32, u32, u32) {
        match self {
            Arm::MarginThreshold | Arm::Top1RateThreshold => (4, 1, 0),
            Arm::BucketModel => (4 + 160 * 4, 2, 1),
            Arm::RichCombo => (4 + 6 * 4, 6, 0),
            Arm::CurrentD4 | Arm::DistanceOnly | Arm::CountOnly => (0, 1, 0),
            Arm::ConstantScore | Arm::InvertedMargin => (4, 1, 0),
        }
    }

    /// The integer confidence score (saturating; deterministic). `table` is
    /// consulted only by the bucket arm.
    fn score(self, f: &Features, table: &BucketTable) -> i64 {
        match self {
            Arm::MarginThreshold => i64::from(f.margin_pm),
            Arm::Top1RateThreshold => i64::from(f.top1_pm),
            Arm::BucketModel => table.score(f),
            Arm::RichCombo => {
                let support = i64::from(f.support.min(15));
                let disagree = i64::from(f.disagree.min(15));
                i64::from(f.margin_pm)
                    .saturating_add(2 * i64::from(f.top1_pm))
                    .saturating_add(if f.agree { 64 } else { 0 })
                    .saturating_add(32 * support)
                    .saturating_sub(32 * disagree)
                    .saturating_add(16 * log2_bucket(f.total) as i64)
            }
            Arm::CurrentD4 => i64::from(f.present),
            Arm::DistanceOnly => {
                // Low branching factor reads as "close to memorized" — the
                // declared distance analog on the table artifact.
                if f.present {
                    64 - i64::from(f.distinct.min(64))
                } else {
                    0
                }
            }
            Arm::CountOnly => i64::from(f.total),
            Arm::ConstantScore => 1,
            Arm::InvertedMargin => 1000 - i64::from(f.margin_pm),
        }
    }
}

/// One selective-risk curve point after thresholding at `theta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OperatingPoint {
    theta: i64,
    served: u64,
    wrong: u64,
    n: u64,
}

impl OperatingPoint {
    fn coverage_pm(&self) -> u32 {
        ((self.served * 1000) / self.n.max(1)) as u32
    }
    fn ucb_pm(&self) -> u32 {
        if self.served == 0 {
            u32::MAX
        } else {
            ucb95_permille(self.wrong, self.served)
        }
    }
}

/// Sweep every distinct score as a threshold (serve = score ≥ θ) and return
/// the operating points, densest coverage first. Deterministic: BTreeMap over
/// scores, cumulative from the highest score down.
fn sweep(scored: &[(i64, bool)]) -> Vec<OperatingPoint> {
    let n = scored.len() as u64;
    let mut by_score: BTreeMap<i64, (u64, u64)> = BTreeMap::new();
    for &(s, ok) in scored {
        let e = by_score.entry(s).or_insert((0, 0));
        e.0 += 1;
        e.1 += u64::from(!ok);
    }
    let mut points = Vec::with_capacity(by_score.len());
    let mut served = 0u64;
    let mut wrong = 0u64;
    for (&theta, &(cnt, wr)) in by_score.iter().rev() {
        served += cnt;
        wrong += wr;
        points.push(OperatingPoint {
            theta,
            served,
            wrong,
            n,
        });
    }
    points
}

/// Compact risk–coverage curve summary: for each target coverage (‰), the
/// densest point at or above it — the per-arm curve the record reports.
fn curve_summary(points: &[OperatingPoint]) -> Vec<(u32, OperatingPoint)> {
    let targets = [10u32, 20, 50, 100, 200, 500, 1000];
    let mut out = Vec::new();
    for &t in &targets {
        if let Some(p) = points.iter().find(|p| p.coverage_pm() >= t) {
            out.push((t, *p));
        }
    }
    out
}

/// The frozen #838 selection rule: among points with coverage ≥ `floor_pm`
/// and UCB95 ≤ `target_pm`, maximize coverage; ties break to the larger θ
/// (the smaller served set is impossible at equal coverage; larger θ is the
/// deterministic representative).
fn qualify(points: &[OperatingPoint], target_pm: u32, floor_pm: u32) -> Option<OperatingPoint> {
    let mut best: Option<OperatingPoint> = None;
    for p in points {
        if p.ucb_pm() > target_pm || p.coverage_pm() < floor_pm {
            continue;
        }
        let better = match best {
            None => true,
            Some(b) => {
                p.coverage_pm() > b.coverage_pm()
                    || (p.coverage_pm() == b.coverage_pm() && p.theta > b.theta)
            }
        };
        if better {
            best = Some(*p);
        }
    }
    best
}

/// Deterministic index permutation (LCG walk) for the shuffled-label and
/// shuffled-feature nulls. No RNG dependence: fixed multiplier/increment.
fn lcg_permutation(n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    for i in (1..n).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

// --- committed small fixture ---------------------------------------------------

/// A deterministic synthetic feature/label population for the fixture teeth:
/// high-margin rows are mostly correct, low-margin rows mostly wrong, novel
/// rows wrong — enough structure for a margin calibrator to separate and for
/// every planted null to fail.
fn fixture_rows() -> Vec<(Features, bool)> {
    let mut rows = Vec::new();
    for i in 0..240u32 {
        let (margin_pm, present, ok) = if i == 6 {
            // exactly one planted high-margin error (edge behavior)
            (960, true, false)
        } else {
            match i % 12 {
                // memorized: high margin, correct
                0..=6 => (900 + (i % 3) * 30, true, true),
                // ambiguous: low margin, mixed labels whose pattern is NOT
                // aligned with any margin decile (defeats cell memorization)
                7..=9 => (80 + (i % 4) * 40, true, (i / 12) % 4 == 0),
                // novel: absent key, wrong
                _ => (0, false, false),
            }
        };
        rows.push((
            Features {
                present,
                total: if present { 40 + i % 17 } else { 0 },
                margin_pm,
                top1_pm: margin_pm.saturating_add(30).min(1000),
                distinct: if present { 3 + i % 5 } else { 0 },
                support: if ok { 3 } else { 1 },
                disagree: if ok { 0 } else { 2 },
                agree: ok,
            },
            ok,
        ));
    }
    rows
}

fn scored_rows(arm: Arm, rows: &[(Features, bool)], table: &BucketTable) -> Vec<(i64, bool)> {
    rows.iter()
        .map(|(f, ok)| (arm.score(f, table), *ok))
        .collect()
}

// --- fixture teeth (non-ignored) -----------------------------------------------

#[test]
fn fixture_extraction_quantization_and_thresholding_operate() {
    let rows = fixture_rows();
    // Quantization/saturation invariants over the fixture domain.
    for (f, _) in &rows {
        assert!(f.margin_pm <= 1000 && f.top1_pm <= 1000, "permille domain");
        assert!(log2_bucket(f.total) <= 15 && margin_bucket(f.margin_pm) <= 9);
        if !f.present {
            assert_eq!((f.total, f.margin_pm, f.distinct), (0, 0, 0));
        }
    }
    let table = BucketTable::fit(&rows);
    // A margin threshold separates the fixture: the high-margin slice is
    // high-precision, and the full population is not.
    let points = sweep(&scored_rows(Arm::MarginThreshold, &rows, &table));
    let high = points
        .iter()
        .find(|p| p.theta >= 900)
        .expect("high-margin point");
    assert!(
        high.wrong * 10 < high.served,
        "high-margin slice ≥ 90% correct"
    );
    let full = points.last().expect("full-coverage point");
    assert!(
        full.wrong * 3 > full.served,
        "full coverage is low-precision"
    );
    // Thresholding is monotone in coverage as θ decreases.
    for w in points.windows(2) {
        assert!(w[1].served >= w[0].served && w[1].theta < w[0].theta);
    }
}

#[test]
fn planted_controls_fail_the_gate_and_are_non_degenerate() {
    let rows = fixture_rows();
    let table = BucketTable::fit(&rows);
    // Fixture-scale gate: a real margin arm qualifies at a 100‰ bound while
    // every planted null fails the same bound (the bound is fixture-scale
    // because n=240 cannot resolve 10‰; the arithmetic is the frozen form).
    let bound = 100;
    let floor = 100;
    let margin_ok = qualify(
        &sweep(&scored_rows(Arm::MarginThreshold, &rows, &table)),
        bound,
        floor,
    );
    assert!(margin_ok.is_some(), "the informative arm qualifies");

    // Constant score: one distinct point (always-serve), fails the bound.
    let const_points = sweep(&scored_rows(Arm::ConstantScore, &rows, &table));
    assert_eq!(const_points.len(), 1, "constant score has one point");
    assert!(qualify(&const_points, bound, floor).is_none());

    // Inverted margin: confidently serves the WRONG slice; fails, and its
    // top-score slice is worse than the population base rate.
    let inv_points = sweep(&scored_rows(Arm::InvertedMargin, &rows, &table));
    assert!(qualify(&inv_points, bound, floor).is_none());
    let inv_top = inv_points.first().expect("top slice");
    let full = inv_points.last().expect("full");
    assert!(
        inv_top.wrong * full.served >= full.wrong * inv_top.served,
        "inverted arm's confident slice is no better than base"
    );

    // Shuffled labels: refit the bucket table on permuted labels — the
    // fitted scores carry no information, so the null fails the gate.
    let perm = lcg_permutation(rows.len());
    let shuffled_rows: Vec<(Features, bool)> = rows
        .iter()
        .enumerate()
        .map(|(i, (f, _))| (*f, rows[perm[i]].1))
        .collect();
    let shuffled_table = BucketTable::fit(&shuffled_rows);
    let shuffled_scored: Vec<(i64, bool)> = rows
        .iter()
        .map(|(f, ok)| (Arm::BucketModel.score(f, &shuffled_table), *ok))
        .collect();
    // Non-degenerate: the shuffle really changed fitted scores.
    let real_scored = scored_rows(Arm::BucketModel, &rows, &table);
    assert_ne!(
        real_scored, shuffled_scored,
        "shuffled-label fit differs from the real fit"
    );
    assert!(qualify(&sweep(&shuffled_scored), bound, floor).is_none());

    // Shuffled features: permute feature vectors against labels — fails.
    let shuffled_feat: Vec<(i64, bool)> = rows
        .iter()
        .enumerate()
        .map(|(i, (_, ok))| (Arm::MarginThreshold.score(&rows[perm[i]].0, &table), *ok))
        .collect();
    assert_ne!(shuffled_feat, real_scored);
    assert!(qualify(&sweep(&shuffled_feat), bound, floor).is_none());

    // Label-leakage tooth: a "feature" computed FROM the label would
    // qualify perfectly — the planted leak is DETECTABLE because no honest
    // integer feature reaches zero wrong at full fixture coverage.
    let leaked: Vec<(i64, bool)> = rows.iter().map(|(_, ok)| (i64::from(*ok), *ok)).collect();
    let leak_full = sweep(&leaked).first().copied().expect("leak point");
    assert_eq!(leak_full.wrong, 0, "the planted leak is perfect…");
    let honest_best = sweep(&real_scored)
        .iter()
        .map(|p| p.wrong)
        .min()
        .unwrap_or(0);
    assert!(
        honest_best > 0,
        "…and no honest arm is, so leakage is detectable"
    );
}

#[test]
fn integer_lowering_differential_is_quantified() {
    // The deployed form is integer; the compiler-side shadow is f64. The
    // differential counts decision changes at the same θ. The fixture plants
    // a boundary case (f64 weight 0.5 rounding) to prove the instrument can
    // detect a nonzero differential; the shipped arms are integer-native, so
    // their own differential is zero by construction.
    let rows = fixture_rows();
    let table = BucketTable::fit(&rows);
    let theta = 500i64;
    let mut integer_native_diff = 0u32;
    let mut planted_diff = 0u32;
    for (f, _) in &rows {
        let int_score = Arm::RichCombo.score(f, &table);
        // f64 shadow of the same weights — identical decisions expected.
        let support = f64::from(f.support.min(15));
        let disagree = f64::from(f.disagree.min(15));
        let f64_score = f64::from(f.margin_pm)
            + 2.0 * f64::from(f.top1_pm)
            + if f.agree { 64.0 } else { 0.0 }
            + 32.0 * support
            - 32.0 * disagree
            + 16.0 * log2_bucket(f.total) as f64;
        if (int_score >= theta) != (f64_score >= theta as f64) {
            integer_native_diff += 1;
        }
        // Planted fractional-weight model: 0.29 is not exactly
        // representable in binary, so 0.29·200 = 57.999…9 sits below the
        // integer lowering (200·29)/100 = 58 — a real decision change at
        // θ = 58 that the differential must count.
        let planted_f64 = 0.29 * f64::from(f.margin_pm);
        let planted_int = (i64::from(f.margin_pm) * 29) / 100;
        if (planted_int >= 58) != (planted_f64 >= 58.0) {
            planted_diff += 1;
        }
    }
    assert_eq!(integer_native_diff, 0, "integer-native arm: no drift");
    assert!(
        planted_diff > 0,
        "the differential instrument detects drift"
    );
}

#[test]
fn partitions_are_disjoint_and_leakage_is_detected() {
    // Story-keyed 3-way partition: fit/calibration/test share no story.
    let stories: Vec<u32> = (100..160).collect();
    let part = |s: u32| -> usize { (s % 3) as usize };
    let mut keys: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for &s in &stories {
        keys[part(s)].push(format!("story-{s}"));
    }
    for i in 0..3 {
        for j in (i + 1)..3 {
            let a: Vec<&str> = keys[i].iter().map(String::as_str).collect();
            let b: Vec<&str> = keys[j].iter().map(String::as_str).collect();
            assert!(
                detect_document_leakage(&a, &b).is_none(),
                "partitions {i}/{j} disjoint"
            );
        }
    }
    // A planted overlap is detected.
    let a = ["story-101", "story-104"];
    let b = ["story-104", "story-200"];
    assert!(detect_document_leakage(&a, &b).is_some());
}

#[test]
fn double_run_and_reordered_input_determinism() {
    let rows = fixture_rows();
    let table = BucketTable::fit(&rows);
    for arm in [
        Arm::MarginThreshold,
        Arm::Top1RateThreshold,
        Arm::BucketModel,
        Arm::RichCombo,
        Arm::CurrentD4,
        Arm::DistanceOnly,
        Arm::CountOnly,
    ] {
        let a = sweep(&scored_rows(arm, &rows, &table));
        let b = sweep(&scored_rows(arm, &rows, &table));
        assert_eq!(a, b, "{} double-run identical", arm.label());
        let mut rev = rows.clone();
        rev.reverse();
        let c = sweep(&scored_rows(arm, &rev, &table));
        assert_eq!(a, c, "{} order-invariant", arm.label());
    }
    // Refit on reversed input produces the identical table.
    let mut rev = rows.clone();
    rev.reverse();
    let t2 = BucketTable::fit(&rev);
    assert_eq!(table.precision_pm, t2.precision_pm, "fit order-invariant");
}

// --- the real corpus study ------------------------------------------------------

struct Study {
    /// (features, base-correct) per held-out position, per partition.
    parts: [Vec<(Features, bool)>; 3],
    /// story counts per partition (document-disjointness evidence).
    stories: [u64; 3],
    base_hits: u64,
    n: u64,
    novel_total: u64,
    novel_content_right: u64,
    corpus_cid: String,
}

fn load_study() -> Option<Study> {
    let root = bundle_root();
    let bundle = ServingBundle::discover(&root)?;
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("recs utf8"),
    )
    .expect("load corpus");
    let (train, held_out) = induction::split_positions(&corpus);
    assert!(!train.is_empty() && !held_out.is_empty(), "non-empty split");

    let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
    let mut content_next: HashMap<u32, Counter> = HashMap::new();
    let mut marginal = Counter::default();
    for &i in &train {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        marginal.bump(target);
        suffix_next.entry(suffix_key(&w)).or_default().bump(target);
        let mut seen: Vec<u32> = w.clone();
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            content_next.entry(t).or_default().bump(target);
        }
    }
    for c in suffix_next.values_mut() {
        c.cap_to_top(CAP);
    }
    for c in content_next.values_mut() {
        c.cap_to_top(CAP);
    }
    let marginal_tok = marginal
        .map
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
        .unwrap_or(0);
    let tables = Tables {
        suffix_next,
        content_next,
        marginal_tok,
    };

    let mut parts: [Vec<(Features, bool)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut story_sets: [BTreeSet<u64>; 3] = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    let mut base_hits = 0u64;
    let mut novel_total = 0u64;
    let mut novel_content_right = 0u64;
    for &i in &held_out {
        let w = induction::context_window(&corpus, i);
        let target = corpus.t_argmax[i];
        let pred = tables.base_predict(&w);
        let ok = pred == target;
        base_hits += u64::from(ok);
        let f = tables.features(&w);
        if !f.present {
            novel_total += 1;
            // Content-side aggregate argmax — the answerable-novelty probe
            // (the evidence a suffix-feature calibrator cannot see).
            let mut uniq: Vec<u32> = w.clone();
            uniq.sort_unstable();
            uniq.dedup();
            let mut content_rate: HashMap<u32, f64> = HashMap::new();
            let ncontent = uniq.len().max(1) as f64;
            for t in &uniq {
                if let Some(cn) = tables.content_next.get(t) {
                    let tot = cn.total().max(1) as f64;
                    for (&cand, &cnt) in &cn.map {
                        *content_rate.entry(cand).or_insert(0.0) +=
                            (f64::from(cnt) / tot) / ncontent;
                    }
                }
            }
            if !content_rate.is_empty() && argmax(&content_rate) == target {
                novel_content_right += 1;
            }
        }
        let story = corpus.story[i] as u64;
        let p = (story % 3) as usize;
        story_sets[p].insert(story);
        parts[p].push((f, ok));
    }
    // Document-disjointness tooth: the three partitions share no story.
    for i in 0..3 {
        for j in (i + 1)..3 {
            assert!(
                story_sets[i].intersection(&story_sets[j]).next().is_none(),
                "partitions {i}/{j} must be story-disjoint"
            );
        }
    }
    Some(Study {
        parts,
        stories: [
            story_sets[0].len() as u64,
            story_sets[1].len() as u64,
            story_sets[2].len() as u64,
        ],
        base_hits,
        n: held_out.len() as u64,
        novel_total,
        novel_content_right,
        corpus_cid: compute_cid(&meta_bytes),
    })
}

fn assert_base_reproduces(study: &Study) {
    let base_r = study.base_hits as f64 / study.n as f64 * 1000.0;
    assert!(
        (base_r - REPRO_BASE_PERMILLE).abs() < 0.05,
        "suffix baseline {base_r:.1}permille does not reproduce the recorded {REPRO_BASE_PERMILLE}permille"
    );
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn calibrator_instrument_837() {
    let Some(study) = load_study() else {
        eprintln!("SKIP calibrator_instrument_837: no serving bundle");
        return;
    };
    assert_base_reproduces(&study);
    let fit = &study.parts[0];
    println!("=== #837 binding cheap instrument (FIT partition only) ===");
    println!("corpus_meta_cid  : {}", study.corpus_cid);
    println!(
        "partitions       : fit {} / cal {} / test {} positions; stories {:?}",
        study.parts[0].len(),
        study.parts[1].len(),
        study.parts[2].len(),
        study.stories
    );
    // Feature availability and variance on FIT only (test stays untouched).
    let n = fit.len() as u64;
    let present = fit.iter().filter(|(f, _)| f.present).count() as u64;
    let agree = fit.iter().filter(|(f, _)| f.agree).count() as u64;
    let margin_nonzero = fit.iter().filter(|(f, _)| f.margin_pm > 0).count() as u64;
    let support_nonzero = fit.iter().filter(|(f, _)| f.support > 0).count() as u64;
    let disagree_nonzero = fit.iter().filter(|(f, _)| f.disagree > 0).count() as u64;
    println!(
        "feature exposure : present {}permille, margin>0 {}permille, agree {}permille, support>0 {}permille, disagree>0 {}permille",
        present * 1000 / n,
        margin_nonzero * 1000 / n,
        agree * 1000 / n,
        support_nonzero * 1000 / n,
        disagree_nonzero * 1000 / n
    );
    // Observability ceiling: what fraction of FIT-partition baseline errors
    // expose a low-confidence signal (absent key, or margin below the
    // population median among errors)? An error with no observable signal
    // cannot be prevented by ANY artifact-only calibrator.
    let errors: Vec<&Features> = fit.iter().filter(|(_, ok)| !ok).map(|(f, _)| f).collect();
    let e = errors.len() as u64;
    let low_signal = errors
        .iter()
        .filter(|f| !f.present || f.margin_pm < 500 || !f.agree)
        .count() as u64;
    println!(
        "ceiling          : {} of {} fit-partition errors ({}permille) expose a low-confidence signal (absent | margin<500 | no-agreement)",
        low_signal,
        e,
        low_signal * 1000 / e.max(1)
    );
    // Variance teeth: every feature must actually vary on real data.
    assert!(present > 0 && present < n, "novelty varies");
    assert!(margin_nonzero > 0 && margin_nonzero < n, "margin varies");
    assert!(agree > 0 && agree < n, "agreement varies");
    assert!(low_signal > 0, "errors expose observable signal");
    println!("instrument       : PASS (non-degenerate features; ceiling published)");
}

#[test]
#[ignore = "heavy: needs the compiled #833 bundle corpus; run with --ignored"]
fn calibrator_fit_run_837() {
    let Some(study) = load_study() else {
        eprintln!("SKIP calibrator_fit_run_837: no serving bundle");
        return;
    };
    let started = Instant::now();
    assert_base_reproduces(&study);
    let [fit, cal, test] = &study.parts;

    // Leakage check across the three partitions (story-keyed).
    // Partition keys are disjoint by construction (story % 3); the planted
    // fixture test proves the detector has teeth.

    // --- fit: the bucket table (the only fitted model) --------------------
    let table = BucketTable::fit(fit);
    let shuffled_table = {
        let perm = lcg_permutation(fit.len());
        let rows: Vec<(Features, bool)> = fit
            .iter()
            .enumerate()
            .map(|(i, (f, _))| (*f, fit[perm[i]].1))
            .collect();
        BucketTable::fit(&rows)
    };

    // --- calibrate: frozen rule on the calibration partition ---------------
    println!("=== #837 calibrator fit run ===");
    println!("corpus_meta_cid  : {}", study.corpus_cid);
    println!(
        "partitions       : fit {} / cal {} / test {}; stories {:?}",
        fit.len(),
        cal.len(),
        test.len(),
        study.stories
    );
    let base_r = study.base_hits as f64 / study.n as f64 * 1000.0;
    println!("base (reproduced): {base_r:.1}permille over {}", study.n);

    let mut cal_results: Vec<(Arm, Option<OperatingPoint>, Option<OperatingPoint>)> = Vec::new();
    let mut cal_curves: Vec<(Arm, Vec<(u32, OperatingPoint)>)> = Vec::new();
    for arm in [
        Arm::MarginThreshold,
        Arm::Top1RateThreshold,
        Arm::BucketModel,
        Arm::RichCombo,
        Arm::CurrentD4,
        Arm::DistanceOnly,
        Arm::CountOnly,
        Arm::ConstantScore,
        Arm::InvertedMargin,
    ] {
        let scored = scored_rows(arm, cal, &table);
        let points = sweep(&scored);
        let release = qualify(
            &points,
            RELEASE_UCB_PERMILLE,
            RELEASE_COVERAGE_FLOOR_PERMILLE,
        );
        let research = qualify(
            &points,
            RESEARCH_UCB_PERMILLE,
            RESEARCH_COVERAGE_FLOOR_PERMILLE,
        );
        let fmt = |p: Option<OperatingPoint>| -> String {
            p.map_or_else(
                || "-".to_owned(),
                |p| {
                    format!(
                        "θ={} cov={}permille ucb={}permille",
                        p.theta,
                        p.coverage_pm(),
                        p.ucb_pm()
                    )
                },
            )
        };
        let (bytes, freads, treads) = arm.budget();
        println!(
            "cal {:<18} release[{}] research[{}] budget[{}B {}f {}t]",
            arm.label(),
            fmt(release),
            fmt(research),
            bytes,
            freads,
            treads
        );
        let curve = curve_summary(&points);
        let curve_str: String = curve
            .iter()
            .map(|(t, p)| {
                format!(
                    "@{}permille: err {}permille (ucb {}) | ",
                    t,
                    (p.wrong * 1000) / p.served.max(1),
                    p.ucb_pm()
                )
            })
            .collect();
        println!("    curve {curve_str}");
        cal_curves.push((arm, curve));
        cal_results.push((arm, release, research));
    }
    // Shuffled-label null (bucket table refit on permuted labels).
    let shuffled_scored: Vec<(i64, bool)> = cal
        .iter()
        .map(|(f, ok)| (Arm::BucketModel.score(f, &shuffled_table), *ok))
        .collect();
    let shuffled_release = qualify(
        &sweep(&shuffled_scored),
        RELEASE_UCB_PERMILLE,
        RELEASE_COVERAGE_FLOOR_PERMILLE,
    );
    let shuffled_research = qualify(
        &sweep(&shuffled_scored),
        RESEARCH_UCB_PERMILLE,
        RESEARCH_COVERAGE_FLOOR_PERMILLE,
    );
    assert!(
        shuffled_release.is_none() && shuffled_research.is_none(),
        "shuffled-label null must not qualify"
    );
    println!("cal shuffled-label   release[-] research[-] (null holds)");

    // --- select: pre-declared rule ----------------------------------------
    // Among candidate arms only, release-qualification first; max coverage;
    // ties to the simpler arm (candidate order). Controls never selectable.
    let mut selected: Option<(Arm, OperatingPoint)> = None;
    for &(arm, release, _) in &cal_results {
        if !Arm::CANDIDATES.contains(&arm) {
            continue;
        }
        if let Some(p) = release {
            let better = match selected {
                None => true,
                Some((_, b)) => p.coverage_pm() > b.coverage_pm(),
            };
            if better {
                selected = Some((arm, p));
            }
        }
    }
    let research_only: Vec<(Arm, OperatingPoint)> = cal_results
        .iter()
        .filter(|(arm, rel, _)| Arm::CANDIDATES.contains(arm) && rel.is_none())
        .filter_map(|(arm, _, res)| (*res).map(|p| (*arm, p)))
        .collect();

    // --- evaluate: the untouched test partition, exactly once --------------
    let test_eval = selected.map(|(arm, p)| {
        let scored = scored_rows(arm, test, &table);
        let n = scored.len() as u64;
        let mut served = 0u64;
        let mut wrong = 0u64;
        for &(s, ok) in &scored {
            if s >= p.theta {
                served += 1;
                wrong += u64::from(!ok);
            }
        }
        OperatingPoint {
            theta: p.theta,
            served,
            wrong,
            n,
        }
    });

    // Answerable-novelty retention: a suffix-feature calibrator cannot see
    // content evidence, so its serve rate among content-answerable novel
    // positions is structurally zero — reported, not gated (secondary metric
    // per the issue's run contract; the redesign signal of the negative
    // branch).
    println!(
        "answerable-novel : {} of {} novel positions are content-answerable; suffix-feature arms serve 0 of them (margin=0 on absent keys)",
        study.novel_content_right, study.novel_total
    );

    // Double-run determinism on a bounded subsample.
    let check = DOUBLE_RUN_N.min(cal.len());
    for (f, _) in cal.iter().take(check) {
        for arm in Arm::CANDIDATES {
            assert_eq!(
                arm.score(f, &table),
                arm.score(f, &table),
                "double-run drift"
            );
        }
    }

    // --- verdict ------------------------------------------------------------
    let verdict = match (selected, test_eval) {
        (Some((arm, p)), Some(t))
            if t.ucb_pm() <= RELEASE_UCB_PERMILLE
                && t.coverage_pm() >= RELEASE_COVERAGE_FLOOR_PERMILLE =>
        {
            format!(
                "SELECT: {} at θ={} — untouched-test false-answer UCB95 {}permille ≤ {}permille at coverage {}permille; activates #839 integration without retuning",
                arm.label(),
                p.theta,
                t.ucb_pm(),
                RELEASE_UCB_PERMILLE,
                t.coverage_pm()
            )
        }
        (Some((arm, p)), Some(t)) => format!(
            "NO CALIBRATOR ESTABLISHED — {} qualified on calibration (θ={}) but the untouched test read ucb={}permille cov={}permille missed the frozen release gate; D4 stays coverage-only",
            arm.label(),
            p.theta,
            t.ucb_pm(),
            t.coverage_pm()
        ),
        _ => {
            if research_only.is_empty() {
                "NO CALIBRATOR ESTABLISHED — no arm met the frozen release gate on the calibration partition; D4 stays coverage-only and evidence acquisition needs redesign".to_owned()
            } else {
                let names: Vec<&str> =
                    research_only.iter().map(|(a, _)| a.label()).collect();
                format!(
                    "NO CALIBRATOR ESTABLISHED (release) — research-grade only: {} met the 50permille research gate on calibration; no production activation (the release bar stands)",
                    names.join(", ")
                )
            }
        }
    };

    let elapsed = started.elapsed();
    println!("elapsed          : {:.1}s", elapsed.as_secs_f64());
    println!("VERDICT          : {verdict}");

    // --- CID-bound record ----------------------------------------------------
    let mut rec = Vec::new();
    for v in [
        study.n,
        study.base_hits,
        fit.len() as u64,
        cal.len() as u64,
        test.len() as u64,
        study.stories[0],
        study.stories[1],
        study.stories[2],
        study.novel_total,
        study.novel_content_right,
    ] {
        rec.extend_from_slice(&v.to_le_bytes());
    }
    for (arm, release, research) in &cal_results {
        rec.push(*arm as u8);
        for p in [release, research] {
            let (t, s, w) = p.map_or((0i64, 0u64, 0u64), |p| (p.theta, p.served, p.wrong));
            rec.extend_from_slice(&t.to_le_bytes());
            rec.extend_from_slice(&s.to_le_bytes());
            rec.extend_from_slice(&w.to_le_bytes());
        }
    }
    if let Some(t) = test_eval {
        rec.extend_from_slice(&t.theta.to_le_bytes());
        rec.extend_from_slice(&t.served.to_le_bytes());
        rec.extend_from_slice(&t.wrong.to_le_bytes());
    }
    rec.extend_from_slice(study.corpus_cid.as_bytes());
    let result_cid = compute_cid(&rec);
    println!("result_cid       : {result_cid}");

    let arm_json = |name: &str, p: Option<OperatingPoint>| -> String {
        p.map_or_else(
            || format!("{name:?}: null"),
            |p| {
                format!(
                    "{name:?}: {{\"theta\": {}, \"served\": {}, \"wrong\": {}, \"coverage_permille\": {}, \"ucb_permille\": {}}}",
                    p.theta,
                    p.served,
                    p.wrong,
                    p.coverage_pm(),
                    p.ucb_pm()
                )
            },
        )
    };
    let mut arms_block = String::new();
    for (i, (arm, release, research)) in cal_results.iter().enumerate() {
        if i > 0 {
            arms_block.push_str(",\n");
        }
        let (bytes, freads, treads) = arm.budget();
        let curve = cal_curves
            .iter()
            .find(|(a, _)| a == arm)
            .map(|(_, c)| c.as_slice())
            .unwrap_or(&[]);
        let mut curve_json = String::from("[");
        for (j, (t, p)) in curve.iter().enumerate() {
            if j > 0 {
                curve_json.push(',');
            }
            curve_json.push_str(&format!(
                "{{\"at_coverage_permille\": {}, \"served\": {}, \"wrong\": {}, \"error_permille\": {}, \"ucb_permille\": {}}}",
                t,
                p.served,
                p.wrong,
                (p.wrong * 1000) / p.served.max(1),
                p.ucb_pm()
            ));
        }
        curve_json.push(']');
        arms_block.push_str(&format!(
            "    {:?}: {{{}, {}, \"curve\": {}, \"budget\": {{\"bytes\": {}, \"feature_reads\": {}, \"table_reads\": {}}}}}",
            arm.label(),
            arm_json("release", *release),
            arm_json("research", *research),
            curve_json,
            bytes,
            freads,
            treads
        ));
    }
    let selected_json = selected.map_or_else(
        || "null".to_owned(),
        |(arm, p)| format!("{{\"arm\": {:?}, \"theta\": {}}}", arm.label(), p.theta),
    );
    let test_json = test_eval.map_or_else(
        || "null".to_owned(),
        |t| {
            format!(
                "{{\"theta\": {}, \"served\": {}, \"wrong\": {}, \"coverage_permille\": {}, \"ucb_permille\": {}}}",
                t.theta,
                t.served,
                t.wrong,
                t.coverage_pm(),
                t.ucb_pm()
            )
        },
    );
    let json = format!(
        concat!(
            "{{\n",
            "  \"issue\": 837,\n",
            "  \"study\": \"artifact-only-confidence-calibrator\",\n",
            "  \"corpus_meta_cid\": \"{}\",\n",
            "  \"base_permille\": {:.1},\n",
            "  \"n\": {},\n",
            "  \"partitions\": {{\"fit\": {}, \"calibration\": {}, \"test\": {}, \"stories\": [{}, {}, {}]}},\n",
            "  \"frozen_gates\": {{\"release_ucb_permille\": {}, \"research_ucb_permille\": {}, \"release_coverage_floor_permille\": {}, \"research_coverage_floor_permille\": {}}},\n",
            "  \"calibration_arms\": {{\n{}\n  }},\n",
            "  \"shuffled_label_null_qualified\": false,\n",
            "  \"selected\": {},\n",
            "  \"test_untouched_single_eval\": {},\n",
            "  \"answerable_novel\": {{\"novel_total\": {}, \"content_answerable\": {}, \"suffix_feature_served\": 0}},\n",
            "  \"result_cid\": \"{}\",\n",
            "  \"verdict\": \"{}\"\n",
            "}}\n"
        ),
        study.corpus_cid,
        base_r,
        study.n,
        fit.len(),
        cal.len(),
        test.len(),
        study.stories[0],
        study.stories[1],
        study.stories[2],
        RELEASE_UCB_PERMILLE,
        RESEARCH_UCB_PERMILLE,
        RELEASE_COVERAGE_FLOOR_PERMILLE,
        RESEARCH_COVERAGE_FLOOR_PERMILLE,
        arms_block,
        selected_json,
        test_json,
        study.novel_total,
        study.novel_content_right,
        result_cid,
        verdict,
    );
    let out = repo_root().join("docs").join("calibrator_837_result.json");
    std::fs::write(&out, json).expect("write result json");
    println!("wrote            : {}", out.display());
}
