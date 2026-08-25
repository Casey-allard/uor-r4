//! #931 -- the one S2 content-evidence calibrator re-entry sanctioned by
//! tracker #823 after the suffix-only #837 study returned
//! `NO CALIBRATOR ESTABLISHED`.
//!
//! This is an offline compiler/certifier study. It reads the exact packed
//! SKMX/PSIB row semantics used by deployed RF-31 and scores the deployed
//! skip-mix winner against the canonical corpus's recorded `t_argmax` labels.
//! It changes no serving behavior and cannot establish the powered #838
//! semantic-answerability claim; the real eight-category annotation fixture
//! remains UNAVAILABLE.
//!
//! Run in order:
//!
//! ```text
//! cargo test -p uor-r4-api --offline --test selective_calibrator_reentry_931
//! R4_CAUSAL_BUNDLE=.uor-models/compiled/smollm2-360m-broad-clean \
//!   cargo test -p uor-r4-api --release --offline \
//!   --test selective_calibrator_reentry_931 \
//!   selective_calibrator_reentry_instrument_931 -- \
//!   --ignored --exact --nocapture
//! # Only when the instrument prints PROCEED:
//! R4_CAUSAL_BUNDLE=.uor-models/compiled/smollm2-360m-broad-clean \
//!   cargo test -p uor-r4-api --release --offline \
//!   --test selective_calibrator_reentry_931 \
//!   selective_calibrator_reentry_full_931 -- \
//!   --ignored --exact --nocapture
//! ```

#![allow(clippy::doc_lazy_continuation)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};
use uor_r4_api::capability_suite::{compute_cid, detect_document_leakage};
use uor_r4_api::engine::{EngineParts, PredictDecision, R4Engine};
use uor_r4_api::serving_eval::ServingBundle;
use uor_r4_core::transformerless::{compiler, runtime};
use uor_r4_graph_certify as score;
use uor_r4_graph_certify::StepCandidates;
use uor_r4_graph_compiler::{induction, skipmix_fit};
use uor_r4_graph_format::{build_psi_bag_table, build_skipmix_table, PsiBagTable, SkipmixTable};

const CAP: usize = uor_r4_graph_compiler::segment_fit::DEFAULT_TOP_K;
const RELEASE_UCB_PM: u32 = 10;
const RESEARCH_UCB_PM: u32 = 50;
const RELEASE_COVERAGE_PM: u32 = 20;
const RESEARCH_COVERAGE_PM: u32 = 50;
const CORPUS_CID: &str = "blake3:aa9d176779c1d2411e872c49c95ed585ee805ded5fa1b808ddf2f517a245b0ce";
const BASE_GRAPH_CID: &str =
    "blake3:aaf98b68a78dd615f06dbb727a22dc4e170a152f055313fcc4fa574309f42d1e";
const SKIP_GRAPH_CID: &str =
    "blake3:19eb04d7dbf3fccd126069982ad8cbc1de31d536fff7e77ef2dacb26e64106cc";
const PRIOR_837_RESULT_CID: &str =
    "blake3:1d1504aa06184b60ded96de366a5102a5d75c05441a3ff92eb86ca8fa8f1e549";
const PRIOR_908_RESULT_CID: &str =
    "blake3:e32e4e33d70f342ae3c0913ba00d9aef0cf789b539b9e1b658a9366c51402a26";
const PARTITION_POSITIONS: [usize; 3] = [24_064, 24_232, 23_834];
const PARTITION_STORIES: [usize; 3] = [199, 200, 200];
const PRIOR_908_BASE_HITS: u64 = 19_372;
const PRIOR_908_SKIP_HITS: u64 = 21_424;
const PRIOR_908_CHANGED: u64 = 39_360;

fn repo_root() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        for ancestor in std::path::Path::new(&manifest_dir).ancestors() {
            if ancestor.join("model/ledger.toml").is_file() {
                return ancestor.to_path_buf();
            }
        }
    }
    let cwd = std::env::current_dir().expect("current directory is readable");
    for ancestor in cwd.ancestors() {
        if ancestor.join("model/ledger.toml").is_file() {
            return ancestor.to_path_buf();
        }
    }
    panic!("repository root not found: no runtime ancestor contains model/ledger.toml");
}

fn bundle_root() -> PathBuf {
    std::env::var_os("R4_CAUSAL_BUNDLE").map_or_else(
        || {
            repo_root()
                .join(".uor-models")
                .join("compiled")
                .join("smollm2-360m-broad-clean")
        },
        PathBuf::from,
    )
}

fn ucb95_permille(failures: u64, n: u64) -> u32 {
    assert!(n > 0, "empty evidence is UNAVAILABLE, never a UCB value");
    (1000_u64
        .saturating_mul(failures)
        .saturating_add(3_000)
        .div_ceil(n)) as u32
}

fn unique_tokens(window: &[u32]) -> Vec<u32> {
    let mut out = window.to_vec();
    out.sort_unstable();
    out.dedup();
    out
}

/// Packed RF-31 contributions. A primary-row PRESENCE suppresses fallback,
/// even when the requested candidate is absent from the primary row.
fn collect_contributions(
    primary: Option<&SkipmixTable<'_>>,
    fallback: Option<&PsiBagTable<'_>>,
    unique: &[u32],
    last: u32,
    shuffled_pairing: bool,
) -> (BTreeMap<u32, i32>, u8, u8) {
    let mut scores = BTreeMap::new();
    let mut joint_rows = 0u8;
    let mut fallback_rows = 0u8;
    for &content in unique {
        let primary_row = if shuffled_pairing {
            primary.and_then(|t| t.find(last, content))
        } else {
            primary.and_then(|t| t.find(content, last))
        };
        let entries = if let Some(row) = primary_row {
            joint_rows = joint_rows.saturating_add(1);
            Some(row.entries())
        } else if let Some(row) = fallback.and_then(|t| t.find(content)) {
            fallback_rows = fallback_rows.saturating_add(1);
            Some(row.entries())
        } else {
            None
        };
        if let Some(entries) = entries {
            for entry in entries.iter() {
                scores
                    .entry(entry.token)
                    .and_modify(|s: &mut i32| *s = s.saturating_add(entry.score_q.raw()))
                    .or_insert(entry.score_q.raw());
            }
        }
    }
    (scores, joint_rows, fallback_rows)
}

fn select_winner(
    ranked: &[(u32, i32)],
    contributions: &BTreeMap<u32, i32>,
    base_token: u32,
    allow_injection: bool,
) -> u32 {
    if ranked.is_empty() {
        return base_token;
    }
    let mut candidates = BTreeSet::new();
    candidates.extend(ranked.iter().map(|&(token, _)| token));
    if allow_injection {
        candidates.extend(contributions.keys().copied());
    }
    let mut winner = base_token;
    let mut best = (false, i32::MIN);
    for token in candidates {
        let contribution = contributions.get(&token).copied().unwrap_or(0);
        let base = ranked
            .iter()
            .find(|&&(candidate, _)| candidate == token)
            .map(|&(_, raw)| raw)
            .unwrap_or(i32::MIN);
        let key = if contribution > 0 {
            (true, contribution)
        } else {
            (false, base)
        };
        if key > best || (key == best && token < winner) {
            best = key;
            winner = token;
        }
    }
    winner
}

fn effective_candidate_count(ranked: &[(u32, i32)], contributions: &BTreeMap<u32, i32>) -> u16 {
    let mut candidates = BTreeSet::new();
    candidates.extend(ranked.iter().map(|&(token, _)| token));
    candidates.extend(contributions.keys().copied());
    u16::try_from(candidates.len()).unwrap_or(u16::MAX)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct ContentFeatures {
    deployed_eligible: bool,
    joint_rows: u8,
    fallback_rows: u8,
    winner_support: u8,
    conflicting_rows: u8,
    winner_contribution: i32,
    runner_contribution: i32,
    content_margin: u32,
    base_agreement: bool,
    injected_candidate: bool,
    base_margin: u32,
    candidate_count: u16,
    suffix_present: bool,
    suffix_total: u32,
    suffix_margin_pm: u32,
}

struct FeatureInputs<'a> {
    primary: Option<&'a SkipmixTable<'a>>,
    fallback: Option<&'a PsiBagTable<'a>>,
    window: &'a [u32],
    ranked: &'a [(u32, i32)],
    base_token: u32,
    winner: u32,
    suffix: (bool, u32, u32),
    shuffled_pairing: bool,
}

fn content_features(inputs: &FeatureInputs<'_>) -> ContentFeatures {
    let unique = unique_tokens(inputs.window);
    let last = inputs.window.last().copied().unwrap_or(u32::MAX);
    let (contributions, joint_rows, fallback_rows) = collect_contributions(
        inputs.primary,
        inputs.fallback,
        &unique,
        last,
        inputs.shuffled_pairing,
    );
    let winner_contribution = contributions.get(&inputs.winner).copied().unwrap_or(0);
    let mut runner_contribution = 0i32;
    for (&token, &contribution) in &contributions {
        if token != inputs.winner && contribution > runner_contribution {
            runner_contribution = contribution;
        }
    }
    let mut winner_support = 0u8;
    let mut conflicting_rows = 0u8;
    for &content in &unique {
        let primary_row = if inputs.shuffled_pairing {
            inputs.primary.and_then(|t| t.find(last, content))
        } else {
            inputs.primary.and_then(|t| t.find(content, last))
        };
        let entries = if let Some(row) = primary_row {
            Some(row.entries())
        } else {
            inputs
                .fallback
                .and_then(|t| t.find(content))
                .map(|row| row.entries())
        };
        if let Some(entries) = entries {
            if entries.find(inputs.winner).is_some() {
                winner_support = winner_support.saturating_add(1);
            }
            let mut local: Option<(u32, i32)> = None;
            for entry in entries.iter() {
                if local.is_none_or(|(token, raw)| {
                    entry.score_q.raw() > raw || (entry.score_q.raw() == raw && entry.token < token)
                }) {
                    local = Some((entry.token, entry.score_q.raw()));
                }
            }
            if local.is_some_and(|(token, _)| token != inputs.winner) {
                conflicting_rows = conflicting_rows.saturating_add(1);
            }
        }
    }
    let base_margin = match (inputs.ranked.first(), inputs.ranked.get(1)) {
        (Some((_, a)), Some((_, b))) => a.saturating_sub(*b).max(0) as u32,
        (Some((_, a)), None) => (*a).max(0) as u32,
        _ => 0,
    };
    ContentFeatures {
        deployed_eligible: true,
        joint_rows,
        fallback_rows,
        winner_support,
        conflicting_rows,
        winner_contribution,
        runner_contribution,
        content_margin: winner_contribution
            .saturating_sub(runner_contribution)
            .max(0) as u32,
        base_agreement: inputs.winner == inputs.base_token,
        injected_candidate: !inputs
            .ranked
            .iter()
            .any(|&(token, _)| token == inputs.winner),
        base_margin,
        candidate_count: effective_candidate_count(inputs.ranked, &contributions),
        suffix_present: inputs.suffix.0,
        suffix_total: inputs.suffix.1,
        suffix_margin_pm: inputs.suffix.2,
    }
}

/// Deliberately direct, row-by-row shadow of RF-31 feature extraction. This
/// does not call `collect_contributions` or `content_features`, so the real
/// run checks the packed lowering against an independent bounded reference.
fn reference_content_features(inputs: &FeatureInputs<'_>) -> ContentFeatures {
    let unique = unique_tokens(inputs.window);
    let last = inputs.window.last().copied().unwrap_or(u32::MAX);
    let mut contributions = BTreeMap::<u32, i32>::new();
    let mut joint_rows = 0u8;
    let mut fallback_rows = 0u8;
    let mut winner_support = 0u8;
    let mut conflicting_rows = 0u8;

    for &content in &unique {
        let primary_row = if inputs.shuffled_pairing {
            inputs.primary.and_then(|table| table.find(last, content))
        } else {
            inputs.primary.and_then(|table| table.find(content, last))
        };
        let entries = if let Some(row) = primary_row {
            joint_rows = joint_rows.saturating_add(1);
            Some(row.entries())
        } else if let Some(row) = inputs.fallback.and_then(|table| table.find(content)) {
            fallback_rows = fallback_rows.saturating_add(1);
            Some(row.entries())
        } else {
            None
        };
        let Some(entries) = entries else {
            continue;
        };
        let mut local_best: Option<(u32, i32)> = None;
        let mut supports_winner = false;
        for entry in entries.iter() {
            contributions
                .entry(entry.token)
                .and_modify(|value| *value = value.saturating_add(entry.score_q.raw()))
                .or_insert(entry.score_q.raw());
            supports_winner |= entry.token == inputs.winner;
            if local_best.is_none_or(|(token, raw)| {
                entry.score_q.raw() > raw || (entry.score_q.raw() == raw && entry.token < token)
            }) {
                local_best = Some((entry.token, entry.score_q.raw()));
            }
        }
        winner_support = winner_support.saturating_add(u8::from(supports_winner));
        conflicting_rows = conflicting_rows.saturating_add(u8::from(
            local_best.is_some_and(|(token, _)| token != inputs.winner),
        ));
    }

    let winner_contribution = contributions.get(&inputs.winner).copied().unwrap_or(0);
    let runner_contribution = contributions
        .iter()
        .filter(|(token, _)| **token != inputs.winner)
        .map(|(_, contribution)| *contribution)
        .max()
        .unwrap_or(0)
        .max(0);
    let base_margin = match (inputs.ranked.first(), inputs.ranked.get(1)) {
        (Some((_, first)), Some((_, second))) => first.saturating_sub(*second).max(0) as u32,
        (Some((_, first)), None) => (*first).max(0) as u32,
        _ => 0,
    };
    ContentFeatures {
        deployed_eligible: true,
        joint_rows,
        fallback_rows,
        winner_support,
        conflicting_rows,
        winner_contribution,
        runner_contribution,
        content_margin: winner_contribution
            .saturating_sub(runner_contribution)
            .max(0) as u32,
        base_agreement: inputs.winner == inputs.base_token,
        injected_candidate: !inputs
            .ranked
            .iter()
            .any(|&(token, _)| token == inputs.winner),
        base_margin,
        candidate_count: {
            let mut reference_candidates = BTreeSet::new();
            reference_candidates.extend(inputs.ranked.iter().map(|&(token, _)| token));
            reference_candidates.extend(contributions.keys().copied());
            u16::try_from(reference_candidates.len()).unwrap_or(u16::MAX)
        },
        suffix_present: inputs.suffix.0,
        suffix_total: inputs.suffix.1,
        suffix_margin_pm: inputs.suffix.2,
    }
}

fn log2_bucket(value: u32, max: usize) -> usize {
    ((32 - value.leading_zeros()) as usize).min(max)
}

fn suffix_margin_bucket(value: u32) -> usize {
    (value / 100).min(9) as usize
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ContentBucketTable {
    precision_pm: [[u16; 16]; 9],
}

impl ContentBucketTable {
    fn fit(rows: &[(ContentFeatures, bool)]) -> Self {
        let mut correct = [[0u32; 16]; 9];
        let mut count = [[0u32; 16]; 9];
        for (f, ok) in rows {
            let s = usize::from(f.winner_support.min(8));
            let m = log2_bucket(f.content_margin, 15);
            count[s][m] += 1;
            correct[s][m] += u32::from(*ok);
        }
        let mut precision_pm = [[0u16; 16]; 9];
        for s in 0..9 {
            for m in 0..16 {
                precision_pm[s][m] = if count[s][m] == 0 {
                    0
                } else {
                    (u64::from(correct[s][m]) * 1000 / u64::from(count[s][m])) as u16
                };
            }
        }
        Self { precision_pm }
    }

    fn score(&self, f: &ContentFeatures) -> u16 {
        self.precision_pm[usize::from(f.winner_support.min(8))][log2_bucket(f.content_margin, 15)]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SuffixBucketTable {
    precision_pm: [[u32; 10]; 16],
}

impl SuffixBucketTable {
    fn fit(rows: &[(ContentFeatures, bool)]) -> Self {
        let mut correct = [[0u32; 10]; 16];
        let mut count = [[0u32; 10]; 16];
        for (f, ok) in rows {
            let s = log2_bucket(f.suffix_total, 15);
            let m = suffix_margin_bucket(f.suffix_margin_pm);
            count[s][m] += 1;
            correct[s][m] += u32::from(*ok);
        }
        let mut precision_pm = [[0u32; 10]; 16];
        for s in 0..16 {
            for m in 0..10 {
                precision_pm[s][m] = correct[s][m]
                    .checked_mul(1000)
                    .and_then(|numerator| numerator.checked_div(count[s][m]))
                    .unwrap_or(0);
            }
        }
        Self { precision_pm }
    }

    fn score(&self, f: &ContentFeatures) -> u32 {
        self.precision_pm[log2_bucket(f.suffix_total, 15)][suffix_margin_bucket(f.suffix_margin_pm)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum Arm {
    ContentMargin,
    ContentSupportMargin,
    HybridContentSuffix,
    SuffixScoreOnSkipmix,
    WinnerSupportOnly,
    BaseMarginOnly,
    CurrentD4,
    Constant,
    InvertedContentMargin,
}

impl Arm {
    const SELECTABLE: [Self; 3] = [
        Self::ContentMargin,
        Self::ContentSupportMargin,
        Self::HybridContentSuffix,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::ContentMargin => "content-margin",
            Self::ContentSupportMargin => "content-support-margin",
            Self::HybridContentSuffix => "hybrid-content-suffix",
            Self::SuffixScoreOnSkipmix => "suffix-bucket-score-on-skipmix",
            Self::WinnerSupportOnly => "winner-support-only",
            Self::BaseMarginOnly => "base-margin-only",
            Self::CurrentD4 => "current-d4",
            Self::Constant => "constant-score",
            Self::InvertedContentMargin => "inverted-content-margin",
        }
    }

    fn budget(self) -> ArmBudget {
        match self {
            Self::ContentMargin => ArmBudget::new(4, 2, 0, 3),
            Self::ContentSupportMargin => ArmBudget::new(292, 2, 1, 7),
            Self::HybridContentSuffix => ArmBudget::new(932, 4, 2, 14),
            Self::SuffixScoreOnSkipmix => ArmBudget::new(644, 2, 1, 7),
            Self::WinnerSupportOnly | Self::BaseMarginOnly => ArmBudget::new(4, 1, 0, 2),
            Self::CurrentD4 => ArmBudget::new(0, 1, 0, 1),
            Self::Constant | Self::InvertedContentMargin => ArmBudget::new(4, 1, 0, 2),
        }
    }

    fn score(
        self,
        f: &ContentFeatures,
        content: &ContentBucketTable,
        suffix: &SuffixBucketTable,
    ) -> i64 {
        if !f.deployed_eligible {
            return i64::MIN;
        }
        match self {
            Self::ContentMargin => i64::from(f.content_margin),
            Self::ContentSupportMargin => i64::from(content.score(f)),
            Self::HybridContentSuffix => {
                i64::from(content.score(f)).saturating_add(i64::from(suffix.score(f)))
            }
            Self::SuffixScoreOnSkipmix => i64::from(suffix.score(f)),
            Self::WinnerSupportOnly => i64::from(f.winner_support),
            Self::BaseMarginOnly => i64::from(f.base_margin),
            // `deployed_eligible` is the Serve/Abstain result returned by the
            // engine's frozen D4 policy. The early return above prevents this
            // control (and every candidate layered after D4) from inventing a
            // token on an Abstain decision. A served D4 row has one fixed
            // score; this control therefore measures the actual deployed
            // eligibility set, not #837's suffix-key novelty proxy.
            Self::CurrentD4 => 1,
            Self::Constant => 1,
            Self::InvertedContentMargin => i64::from(u32::MAX - f.content_margin),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct ArmBudget {
    bytes: u32,
    feature_reads: u32,
    table_reads: u32,
    /// Upper bound for feature-to-score arithmetic, indexing, table reads,
    /// score combination, and the final threshold comparison. Feature
    /// extraction itself is shared RF-31 witness work and is not charged a
    /// second time to these incremental candidate budgets.
    projected_operations: u32,
}

impl ArmBudget {
    const fn new(
        bytes: u32,
        feature_reads: u32,
        table_reads: u32,
        projected_operations: u32,
    ) -> Self {
        Self {
            bytes,
            feature_reads,
            table_reads,
            projected_operations,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct OperatingPoint {
    theta: i64,
    served: u64,
    wrong: u64,
    n: u64,
}

impl OperatingPoint {
    fn coverage_pm(self) -> u32 {
        (self.served * 1000 / self.n.max(1)) as u32
    }

    fn error_pm(self) -> u32 {
        (self.wrong * 1000 / self.served.max(1)) as u32
    }

    fn ucb_pm(self) -> u32 {
        if self.served == 0 {
            u32::MAX
        } else {
            ucb95_permille(self.wrong, self.served)
        }
    }
}

fn sweep(scored: &[(i64, bool)]) -> Vec<OperatingPoint> {
    let mut by_score: BTreeMap<i64, (u64, u64)> = BTreeMap::new();
    for &(score, correct) in scored {
        if score == i64::MIN {
            continue;
        }
        let cell = by_score.entry(score).or_default();
        cell.0 += 1;
        cell.1 += u64::from(!correct);
    }
    let mut out = Vec::with_capacity(by_score.len());
    let mut served = 0;
    let mut wrong = 0;
    for (&theta, &(count, failures)) in by_score.iter().rev() {
        served += count;
        wrong += failures;
        out.push(OperatingPoint {
            theta,
            served,
            wrong,
            n: scored.len() as u64,
        });
    }
    out
}

fn qualify(points: &[OperatingPoint], target_pm: u32, floor_pm: u32) -> Option<OperatingPoint> {
    points
        .iter()
        .copied()
        .filter(|p| p.ucb_pm() <= target_pm && p.coverage_pm() >= floor_pm)
        .max_by(|a, b| a.served.cmp(&b.served).then(a.theta.cmp(&b.theta)))
}

fn scored_rows(
    arm: Arm,
    rows: &[(ContentFeatures, bool)],
    content: &ContentBucketTable,
    suffix: &SuffixBucketTable,
) -> Vec<(i64, bool)> {
    rows.iter()
        .map(|(f, ok)| (arm.score(f, content, suffix), *ok))
        .collect()
}

fn lcg_permutation(n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    for i in (1..n).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (state % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

#[derive(Default, Clone)]
struct Counter {
    counts: HashMap<u32, u32>,
}

impl Counter {
    fn bump(&mut self, token: u32) {
        *self.counts.entry(token).or_insert(0) += 1;
    }

    fn cap_to_top(&mut self, cap: usize) {
        if self.counts.len() <= cap {
            return;
        }
        let mut entries: Vec<(u32, u32)> = self.counts.drain().collect();
        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        entries.truncate(cap);
        self.counts = entries.into_iter().collect();
    }

    fn total(&self) -> u64 {
        self.counts.values().map(|&count| u64::from(count)).sum()
    }
}

fn suffix_key(window: &[u32]) -> (u32, u32) {
    match window {
        [.., penultimate, last] => (*penultimate, *last),
        [last] => (u32::MAX, *last),
        [] => (u32::MAX, u32::MAX),
    }
}

struct SuffixTables {
    suffix_next: HashMap<(u32, u32), Counter>,
    marginal_token: u32,
}

impl SuffixTables {
    fn fit(corpus: &compiler::Corpus, train: &[usize]) -> Self {
        let mut suffix_next: HashMap<(u32, u32), Counter> = HashMap::new();
        let mut marginal = Counter::default();
        for &position in train {
            let window = induction::context_window(corpus, position);
            let target = corpus.t_argmax[position];
            suffix_next
                .entry(suffix_key(&window))
                .or_default()
                .bump(target);
            marginal.bump(target);
        }
        for counter in suffix_next.values_mut() {
            counter.cap_to_top(CAP);
        }
        let marginal_token = marginal
            .counts
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(&token, _)| token)
            .unwrap_or(0);
        Self {
            suffix_next,
            marginal_token,
        }
    }

    fn predict(&self, window: &[u32]) -> u32 {
        self.suffix_next
            .get(&suffix_key(window))
            .and_then(|counter| {
                counter
                    .counts
                    .iter()
                    .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                    .map(|(&token, _)| token)
            })
            .unwrap_or(self.marginal_token)
    }

    fn features(&self, window: &[u32]) -> (bool, u32, u32) {
        let Some(counter) = self.suffix_next.get(&suffix_key(window)) else {
            return (false, 0, 0);
        };
        let mut entries: Vec<(u32, u32)> = counter
            .counts
            .iter()
            .map(|(&token, &count)| (token, count))
            .collect();
        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let total = counter.total().min(u64::from(u32::MAX)) as u32;
        let top1 = entries.first().map(|&(_, count)| count).unwrap_or(0);
        let top2 = entries.get(1).map(|&(_, count)| count).unwrap_or(0);
        let margin = u64::from(top1.saturating_sub(top2)) * 1000 / u64::from(total.max(1));
        (true, total, margin as u32)
    }
}

struct Prepared {
    corpus: compiler::Corpus,
    held_out: Vec<usize>,
    corpus_cid: String,
    base_graph: Vec<u8>,
    skip_graph: Vec<u8>,
    artifact_container: Vec<u8>,
    tokenizer_bytes: Option<Vec<u8>>,
    skmx_bytes: Vec<u8>,
    psib_bytes: Vec<u8>,
    packed_joint_rows_checked: u64,
    packed_fallback_rows_checked: u64,
    suffix: SuffixTables,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    position: usize,
    features: ContentFeatures,
    shuffled_key_features: ContentFeatures,
    base_token: Option<u32>,
    skip_token: Option<u32>,
    suffix_token: u32,
    skmx_only_token: Option<u32>,
    psib_only_token: Option<u32>,
    no_injection_token: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NumericSummary {
    count: u64,
    nonzero: u64,
    min: i64,
    max: i64,
    sum: i64,
    distinct: u64,
    exact_histogram: BTreeMap<i64, u64>,
}

fn numeric_summary(values: impl IntoIterator<Item = i64>) -> NumericSummary {
    let mut exact_histogram = BTreeMap::new();
    let mut count = 0u64;
    let mut nonzero = 0u64;
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let mut sum = 0i64;
    for value in values {
        count += 1;
        nonzero += u64::from(value != 0);
        min = min.min(value);
        max = max.max(value);
        sum = sum.saturating_add(value);
        *exact_histogram.entry(value).or_insert(0) += 1;
    }
    assert!(count > 0, "empty feature evidence is UNAVAILABLE");
    NumericSummary {
        count,
        nonzero,
        min,
        max,
        sum,
        distinct: exact_histogram.len() as u64,
        exact_histogram,
    }
}

fn canonical_observations(observations: &[Observation]) -> Vec<&Observation> {
    let mut ordered: Vec<&Observation> = observations.iter().collect();
    ordered.sort_unstable_by_key(|observation| observation.position);
    assert!(
        ordered
            .windows(2)
            .all(|pair| pair[0].position < pair[1].position),
        "stable-position order must contain unique positions"
    );
    ordered
}

fn feature_evidence(observations: &[Observation]) -> Value {
    let ordered = canonical_observations(observations);
    let bound_rows: Vec<(usize, ContentFeatures)> = ordered
        .iter()
        .map(|observation| (observation.position, observation.features))
        .collect();
    let feature_bytes = serde_json::to_vec(&bound_rows).expect("ordered feature JSON");
    let true_count = |read: fn(&ContentFeatures) -> bool| {
        ordered
            .iter()
            .filter(|observation| read(&observation.features))
            .count() as u64
    };
    let numeric = |read: fn(&ContentFeatures) -> i64| {
        numeric_summary(
            ordered
                .iter()
                .map(|observation| read(&observation.features)),
        )
    };
    json!({
        "positions": ordered.len(),
        "ordered_feature_rows_cid": compute_cid(&feature_bytes),
        "deployed_eligible": {"true": true_count(|f| f.deployed_eligible), "false": true_count(|f| !f.deployed_eligible)},
        "joint_rows": numeric(|f| i64::from(f.joint_rows)),
        "fallback_rows": numeric(|f| i64::from(f.fallback_rows)),
        "winner_support": numeric(|f| i64::from(f.winner_support)),
        "conflicting_rows": numeric(|f| i64::from(f.conflicting_rows)),
        "winner_contribution": numeric(|f| i64::from(f.winner_contribution)),
        "runner_contribution": numeric(|f| i64::from(f.runner_contribution)),
        "content_margin": numeric(|f| i64::from(f.content_margin)),
        "base_agreement": {"true": true_count(|f| f.base_agreement), "false": true_count(|f| !f.base_agreement)},
        "injected_candidate": {"true": true_count(|f| f.injected_candidate), "false": true_count(|f| !f.injected_candidate)},
        "base_margin": numeric(|f| i64::from(f.base_margin)),
        "candidate_count": numeric(|f| i64::from(f.candidate_count)),
        "suffix_present": {"true": true_count(|f| f.suffix_present), "false": true_count(|f| !f.suffix_present)},
        "suffix_total": numeric(|f| i64::from(f.suffix_total)),
        "suffix_margin_permille": numeric(|f| i64::from(f.suffix_margin_pm)),
    })
}

fn verify_prior_records() {
    let prior_837: Value = serde_json::from_slice(
        &std::fs::read(repo_root().join("docs/calibrator_837_result.json"))
            .expect("#837 predecessor record"),
    )
    .expect("parse #837 record");
    assert_eq!(prior_837["result_cid"], PRIOR_837_RESULT_CID);
    assert_eq!(prior_837["corpus_meta_cid"], CORPUS_CID);
    assert_eq!(prior_837["base_permille"], 246.6);
    assert_eq!(prior_837["partitions"]["fit"], PARTITION_POSITIONS[0]);
    assert_eq!(
        prior_837["partitions"]["calibration"],
        PARTITION_POSITIONS[1]
    );
    assert_eq!(prior_837["partitions"]["test"], PARTITION_POSITIONS[2]);
    assert_eq!(prior_837["partitions"]["stories"], json!(PARTITION_STORIES));
    assert_eq!(prior_837["n"], PARTITION_POSITIONS.iter().sum::<usize>());
    assert_eq!(
        prior_837["answerable_novel"],
        json!({
            "novel_total": 26_002,
            "content_answerable": 2_454,
            "suffix_feature_served": 0
        })
    );
    let historical_curve = &prior_837["calibration_arms"]["bucket-table"]["curve"];
    assert_eq!(historical_curve.as_array().map(Vec::len), Some(7));
    assert_eq!(historical_curve[0]["served"], 261);
    assert_eq!(historical_curve[0]["wrong"], 3);
    assert_eq!(historical_curve[1]["served"], 549);
    assert_eq!(historical_curve[1]["wrong"], 19);
    assert_eq!(historical_curve[6]["served"], 24_232);
    assert_eq!(historical_curve[6]["wrong"], 18_237);
    assert_eq!(
        prior_837["calibration_arms"]["bucket-table"]["budget"],
        json!({"bytes": 644, "feature_reads": 2, "table_reads": 1})
    );

    let prior_908: Value = serde_json::from_slice(
        &std::fs::read(repo_root().join("docs/skipmix_endtoend_causal_908_result.json"))
            .expect("#908 predecessor record"),
    )
    .expect("parse #908 record");
    assert_eq!(prior_908["result_cid"], PRIOR_908_RESULT_CID);
    assert_eq!(prior_908["corpus_meta_cid"], CORPUS_CID);
    assert_eq!(prior_908["base_artifact_cid"], BASE_GRAPH_CID);
    assert_eq!(prior_908["skip_artifact_cid"], SKIP_GRAPH_CID);
    assert_eq!(prior_908["base_top1"], PRIOR_908_BASE_HITS);
    assert_eq!(prior_908["skip_top1"], PRIOR_908_SKIP_HITS);
    assert_eq!(prior_908["reachability_changed"], PRIOR_908_CHANGED);
}

#[allow(clippy::type_complexity)]
fn verify_compiler_rows_match_packed(
    joint_rows: &[(u32, u32, Vec<(u32, i32)>)],
    fallback_rows: &[(u32, Vec<(u32, i32)>)],
    skmx_bytes: &[u8],
    psib_bytes: &[u8],
) -> (u64, u64) {
    let primary = SkipmixTable::parse(skmx_bytes).expect("packed SKMX validates");
    let fallback = PsiBagTable::parse(psib_bytes).expect("packed PSIB validates");
    assert_eq!(fallback.row_count() as usize, fallback_rows.len());

    for (content, last, expected) in joint_rows {
        let actual = primary
            .find(*content, *last)
            .unwrap_or_else(|| panic!("packed SKMX omitted compiler row ({content}, {last})"))
            .entries();
        assert_eq!(actual.len(), expected.len());
        for (token, raw) in expected {
            assert_eq!(actual.find(*token).map(|score| score.raw()), Some(*raw));
        }
    }
    for (content, expected) in fallback_rows {
        let actual = fallback
            .find(*content)
            .unwrap_or_else(|| panic!("packed PSIB omitted compiler row {content}"))
            .entries();
        assert_eq!(actual.len(), expected.len());
        for (token, raw) in expected {
            assert_eq!(actual.find(*token).map(|score| score.raw()), Some(*raw));
        }
    }

    (joint_rows.len() as u64, fallback_rows.len() as u64)
}

fn prepare() -> Option<Prepared> {
    let root = bundle_root();
    let bundle = ServingBundle::discover(&root)?;
    let meta_bytes = std::fs::read(&bundle.corpus_meta).expect("corpus meta");
    let recs_bytes = std::fs::read(&bundle.corpus_records).expect("corpus records");
    assert!(!recs_bytes.is_empty(), "recorded labels are non-vacuous");
    let corpus = compiler::load_corpus_from(
        bundle.corpus_meta.to_str().expect("meta utf8"),
        bundle.corpus_records.to_str().expect("records utf8"),
    )
    .expect("load corpus");
    let (train, held_out) = induction::split_positions(&corpus);
    assert!(!train.is_empty() && !held_out.is_empty());
    let corpus_cid = compute_cid(&meta_bytes);
    assert_eq!(corpus_cid, CORPUS_CID, "canonical #833 corpus identity");

    let mut partition_counts = [0usize; 3];
    let mut story_sets: [BTreeSet<u32>; 3] = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    for &position in &held_out {
        let partition = (corpus.story[position] % 3) as usize;
        partition_counts[partition] += 1;
        story_sets[partition].insert(corpus.story[position]);
    }
    assert_eq!(partition_counts, PARTITION_POSITIONS);
    assert_eq!(
        [
            story_sets[0].len(),
            story_sets[1].len(),
            story_sets[2].len()
        ],
        PARTITION_STORIES,
        "story-disjoint frozen folds"
    );
    for i in 0..3 {
        for j in i + 1..3 {
            assert!(story_sets[i].is_disjoint(&story_sets[j]));
        }
    }

    let (skip_rows, psi_rows) = skipmix_fit::fit_skipmix_tables(&corpus, CAP);
    assert!(!skip_rows.is_empty() && !psi_rows.is_empty());
    let skmx_bytes = build_skipmix_table(&skip_rows).expect("canonical SKMX");
    let psib_bytes = build_psi_bag_table(&psi_rows).expect("canonical PSIB");
    let (packed_joint_rows_checked, packed_fallback_rows_checked) =
        verify_compiler_rows_match_packed(&skip_rows, &psi_rows, &skmx_bytes, &psib_bytes);

    let artifact_container = std::fs::read(&bundle.teacher).expect("teacher artifacts");
    let artifacts = compiler::parse_artifacts(&artifact_container).expect("parse artifacts");
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(8))
        .unwrap_or(1);
    let train_observations =
        induction::build_observations_with_threads(&artifacts, &corpus, &train, threads);
    let cover_bytes =
        std::fs::read(bundle.root.join("graph-cover/cover.r4g1")).expect("cached cover artifact");
    let (regions, structural) =
        score::recover_from_artifact(&cover_bytes).expect("recover graph cover");
    let max_depth = regions
        .iter()
        .map(|region| region.depth as usize)
        .max()
        .unwrap_or(1);
    let store_bytes = std::fs::read(bundle.root.join("tless_store.bin")).expect("compiled store");
    let store = runtime::parse_store(&store_bytes).expect("parse compiled store");
    let config = score::ScoreConfig::default();
    let (transitions, transition_quantization) = score::compile_transitions_with_quantization(
        &corpus,
        &regions,
        &train_observations,
        max_depth,
        config.transition_out_degree,
    );
    let vocab =
        u32::try_from(artifacts.token_codes.len() / compiler::STAGES).expect("vocab fits u32");
    let context_rows = score::compile_context_rows(&corpus, &train_observations, vocab, &config);
    let forward_rows = score::compile_forward_anchor_rows(&corpus, &train_observations);
    let emissions = score::compile_emissions(
        &corpus,
        &store,
        &regions,
        &train_observations,
        max_depth,
        vocab,
        &config,
    );
    let tls1 = runtime::store_bytes(&store);
    let empty_skip: Vec<uor_r4_graph_format::SkipmixRowInput> = Vec::new();
    let empty_psi: Vec<(u32, Vec<(u32, i32)>)> = Vec::new();
    let emit = |sk: &[uor_r4_graph_format::SkipmixRowInput], psi: &[(u32, Vec<(u32, i32)>)]| {
        let sections = score::ScoredGraphSections {
            regions: &regions,
            structural: &structural,
            transitions: &transitions,
            transition_quantization,
            emissions: &emissions,
            context_rows: &context_rows,
            exct_tls1: &tls1,
            exct_top_x: config.exct_top_x,
            fwd_rows: &forward_rows,
            skipmix_rows: sk,
            psi_bag_rows: psi,
        };
        score::emit_scored_r4g1(
            &artifact_container,
            (&meta_bytes, &recs_bytes),
            vocab,
            &sections,
        )
        .0
    };
    let base_graph = emit(&empty_skip, &empty_psi);
    let skip_graph = emit(&skip_rows, &psi_rows);
    assert_eq!(compute_cid(&base_graph), BASE_GRAPH_CID);
    assert_eq!(compute_cid(&skip_graph), SKIP_GRAPH_CID);
    let tokenizer_bytes = std::fs::read(bundle.root.join("tokenizer.bin")).ok();
    let suffix = SuffixTables::fit(&corpus, &train);

    Some(Prepared {
        corpus,
        held_out,
        corpus_cid,
        base_graph,
        skip_graph,
        artifact_container,
        tokenizer_bytes,
        skmx_bytes,
        psib_bytes,
        packed_joint_rows_checked,
        packed_fallback_rows_checked,
        suffix,
    })
}

fn observations(prepared: &Prepared, partition: usize) -> Vec<Observation> {
    let primary = SkipmixTable::parse(&prepared.skmx_bytes).expect("SKMX");
    let fallback = PsiBagTable::parse(&prepared.psib_bytes).expect("PSIB");
    let mut engine = R4Engine::load_accepting_quality(EngineParts {
        graph: &prepared.skip_graph,
        signature_artifact: &prepared.artifact_container,
        tokenizer: prepared.tokenizer_bytes.as_deref(),
        score_report: None,
    })
    .expect("load skip engine");
    assert_eq!(engine.skipmix_tables_present(), (true, true));
    let mut out = Vec::with_capacity(PARTITION_POSITIONS[partition]);
    for &position in &prepared.held_out {
        if (prepared.corpus.story[position] % 3) as usize != partition {
            continue;
        }
        let window = induction::context_window(&prepared.corpus, position);
        engine.reset();
        let mut candidates = StepCandidates::default();
        let (decision, attribution) = engine
            .predict_decision_candidates_with_skipmix_witness(&window, &mut candidates)
            .expect("bounded deployed prediction");
        let suffix = prepared.suffix.features(&window);
        let suffix_token = prepared.suffix.predict(&window);
        let Some((skip_token, base_token)) = (match decision {
            PredictDecision::Serve(served) => Some((
                served.token,
                attribution
                    .as_ref()
                    .map_or(served.token, |witness| witness.base_token),
            )),
            PredictDecision::Abstain(_) => None,
        }) else {
            out.push(Observation {
                position,
                features: ContentFeatures {
                    suffix_present: suffix.0,
                    suffix_total: suffix.1,
                    suffix_margin_pm: suffix.2,
                    ..ContentFeatures::default()
                },
                shuffled_key_features: ContentFeatures {
                    suffix_present: suffix.0,
                    suffix_total: suffix.1,
                    suffix_margin_pm: suffix.2,
                    ..ContentFeatures::default()
                },
                base_token: None,
                skip_token: None,
                suffix_token,
                skmx_only_token: None,
                psib_only_token: None,
                no_injection_token: None,
            });
            continue;
        };
        let ranked: Vec<(u32, i32)> = candidates
            .ranked()
            .iter()
            .map(|&(token, score)| (token, score.raw()))
            .collect();
        let unique = unique_tokens(&window);
        let last = window.last().copied().unwrap_or(u32::MAX);
        let (contributions, _, _) =
            collect_contributions(Some(&primary), Some(&fallback), &unique, last, false);
        let reference = select_winner(&ranked, &contributions, base_token, true);
        assert_eq!(reference, skip_token, "packed/reference RF-31 differential");
        if let Some(witness) = &attribution {
            assert_eq!(witness.promoted_token, skip_token);
            assert_eq!(
                witness.boost,
                contributions.get(&skip_token).copied().unwrap_or(0),
                "witness contribution differential"
            );
        }
        let feature_inputs = FeatureInputs {
            primary: Some(&primary),
            fallback: Some(&fallback),
            window: &window,
            ranked: &ranked,
            base_token,
            winner: skip_token,
            suffix,
            shuffled_pairing: false,
        };
        let features = content_features(&feature_inputs);
        assert_eq!(
            features,
            reference_content_features(&feature_inputs),
            "packed/reference full feature differential at position {position}"
        );
        let (shuffled_contributions, _, _) =
            collect_contributions(Some(&primary), Some(&fallback), &unique, last, true);
        let shuffled_winner = select_winner(&ranked, &shuffled_contributions, base_token, true);
        let shuffled_inputs = FeatureInputs {
            primary: Some(&primary),
            fallback: Some(&fallback),
            window: &window,
            ranked: &ranked,
            base_token,
            winner: shuffled_winner,
            suffix,
            shuffled_pairing: true,
        };
        let shuffled_key_features = content_features(&shuffled_inputs);
        assert_eq!(
            shuffled_key_features,
            reference_content_features(&shuffled_inputs),
            "shuffled-key packed/reference feature differential at position {position}"
        );
        let (skmx_contributions, _, _) =
            collect_contributions(Some(&primary), None, &unique, last, false);
        let (psib_contributions, _, _) =
            collect_contributions(None, Some(&fallback), &unique, last, false);
        out.push(Observation {
            position,
            features,
            shuffled_key_features,
            base_token: Some(base_token),
            skip_token: Some(skip_token),
            suffix_token,
            skmx_only_token: Some(select_winner(
                &ranked,
                &skmx_contributions,
                base_token,
                true,
            )),
            psib_only_token: Some(select_winner(
                &ranked,
                &psib_contributions,
                base_token,
                true,
            )),
            no_injection_token: Some(select_winner(&ranked, &contributions, base_token, false)),
        });
    }
    assert_eq!(out.len(), PARTITION_POSITIONS[partition]);
    out
}

fn label_rows(prepared: &Prepared, observations: &[Observation]) -> Vec<(ContentFeatures, bool)> {
    canonical_observations(observations)
        .into_iter()
        .map(|observation| {
            let target = prepared.corpus.t_argmax[observation.position];
            (observation.features, observation.skip_token == Some(target))
        })
        .collect()
}

fn suffix_label_rows(
    prepared: &Prepared,
    observations: &[Observation],
) -> Vec<(ContentFeatures, bool)> {
    canonical_observations(observations)
        .into_iter()
        .map(|observation| {
            let target = prepared.corpus.t_argmax[observation.position];
            (observation.features, observation.suffix_token == target)
        })
        .collect()
}

fn instrument(prepared: &Prepared, fit_observations: &[Observation]) -> Instrument {
    let fit = label_rows(prepared, fit_observations);
    let n = fit.len() as u64;
    let support = fit
        .iter()
        .filter(|(features, _)| features.winner_support > 0)
        .count() as u64;
    let margin = fit
        .iter()
        .filter(|(features, _)| features.content_margin > 0)
        .count() as u64;
    let changed = fit
        .iter()
        .filter(|(features, _)| features.deployed_eligible && !features.base_agreement)
        .count() as u64;
    let injected = fit
        .iter()
        .filter(|(features, _)| features.injected_candidate)
        .count() as u64;
    let deployed_eligible = fit
        .iter()
        .filter(|(features, _)| features.deployed_eligible)
        .count() as u64;
    let deployed_abstained = n.saturating_sub(deployed_eligible);
    let key_shuffle_changed = fit_observations
        .iter()
        .filter(|observation| observation.features != observation.shuffled_key_features)
        .count() as u64;
    // A label-aware upper bound over every declared integer/boolean feature,
    // including the raw margins that distinguish the selectable threshold
    // scores. It is deliberately richer than the selectable coarsenings: an
    // exact-cell ceiling is an instrument only, never a fitted candidate.
    let mut cells: BTreeMap<ContentFeatures, (u64, u64)> = BTreeMap::new();
    for (features, correct) in &fit {
        let cell = cells.entry(*features).or_default();
        cell.0 += 1;
        cell.1 += u64::from(*correct);
    }
    let oracle_scored: Vec<(i64, bool)> = fit
        .iter()
        .map(|(features, correct)| {
            let (count, correct_count) = cells[features];
            let precision = correct_count * 1000 / count.max(1);
            (precision as i64, *correct)
        })
        .collect();
    let oracle_points = sweep(&oracle_scored);
    let release = qualify(&oracle_points, RELEASE_UCB_PM, RELEASE_COVERAGE_PM);
    let research = qualify(&oracle_points, RESEARCH_UCB_PM, RESEARCH_COVERAGE_PM);
    let proceed = support * 1000 / n >= u64::from(RELEASE_COVERAGE_PM)
        && margin > 0
        && margin < n
        && changed > 0
        && injected > 0
        && deployed_eligible > 0
        && cells.len() > 1
        && key_shuffle_changed > 0
        && (release.is_some() || research.is_some());
    Instrument {
        n,
        support,
        margin,
        changed,
        injected,
        deployed_eligible,
        deployed_abstained,
        key_shuffle_changed,
        exact_feature_cells: cells.len() as u64,
        packed_reference_feature_positions_checked: deployed_eligible,
        packed_compiler_joint_rows_checked: prepared.packed_joint_rows_checked,
        packed_compiler_fallback_rows_checked: prepared.packed_fallback_rows_checked,
        release_oracle: release,
        research_oracle: research,
        proceed,
    }
}

#[derive(Debug, Clone, Serialize)]
struct Instrument {
    n: u64,
    support: u64,
    margin: u64,
    changed: u64,
    injected: u64,
    deployed_eligible: u64,
    deployed_abstained: u64,
    key_shuffle_changed: u64,
    exact_feature_cells: u64,
    packed_reference_feature_positions_checked: u64,
    packed_compiler_joint_rows_checked: u64,
    packed_compiler_fallback_rows_checked: u64,
    release_oracle: Option<OperatingPoint>,
    research_oracle: Option<OperatingPoint>,
    proceed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CurvePoint {
    target_coverage_pm: u32,
    point: OperatingPoint,
}

#[derive(Debug, Clone, Serialize)]
struct ArmReport {
    arm: String,
    budget: ArmBudget,
    release: Option<OperatingPoint>,
    research: Option<OperatingPoint>,
    curve: Vec<CurvePoint>,
    full_curve: Vec<OperatingPoint>,
    full_curve_cid: String,
}

fn curve_summary(points: &[OperatingPoint]) -> Vec<CurvePoint> {
    [10, 20, 50, 100, 200, 500, 1000]
        .into_iter()
        .filter_map(|target| {
            points
                .iter()
                .find(|point| point.coverage_pm() >= target)
                .copied()
                .map(|point| CurvePoint {
                    target_coverage_pm: target,
                    point,
                })
        })
        .collect()
}

fn report_arm(
    arm: Arm,
    rows: &[(ContentFeatures, bool)],
    content: &ContentBucketTable,
    suffix: &SuffixBucketTable,
) -> ArmReport {
    report_scored(
        arm.label(),
        arm.budget(),
        &scored_rows(arm, rows, content, suffix),
    )
}

fn report_scored(label: &str, budget: ArmBudget, scored: &[(i64, bool)]) -> ArmReport {
    let points = sweep(scored);
    let full_curve_cid = compute_cid(&serde_json::to_vec(&points).expect("curve JSON"));
    ArmReport {
        arm: label.to_owned(),
        budget,
        release: qualify(&points, RELEASE_UCB_PM, RELEASE_COVERAGE_PM),
        research: qualify(&points, RESEARCH_UCB_PM, RESEARCH_COVERAGE_PM),
        curve: curve_summary(&points),
        full_curve: points,
        full_curve_cid,
    }
}

fn locked_837_reference_report(
    rows: &[(ContentFeatures, bool)],
    suffix: &SuffixBucketTable,
) -> ArmReport {
    assert_eq!(CAP, 64, "#837 retained-continuation capacity is locked");
    let scored: Vec<(i64, bool)> = rows
        .iter()
        .map(|(features, correct)| (i64::from(suffix.score(features)), *correct))
        .collect();
    let report = report_scored(
        "locked-837-suffix-reference",
        ArmBudget::new(644, 2, 1, 7),
        &scored,
    );
    const EXPECTED: [(u32, u64, u64, u32, u32); 7] = [
        (10, 261, 3, 11, 23),
        (20, 549, 19, 34, 41),
        (50, 1_350, 170, 125, 129),
        (100, 2_767, 637, 230, 232),
        (200, 4_922, 1_771, 359, 361),
        (500, 12_768, 7_621, 596, 598),
        (1_000, 24_232, 18_237, 752, 753),
    ];
    assert_eq!(report.curve.len(), EXPECTED.len());
    for (actual, expected) in report.curve.iter().zip(EXPECTED) {
        let (target, served, wrong, error, ucb) = expected;
        assert_eq!(actual.target_coverage_pm, target, "#837 curve target");
        assert_eq!(
            actual.point.served, served,
            "#837 served at {target}permille"
        );
        assert_eq!(actual.point.wrong, wrong, "#837 wrong at {target}permille");
        assert_eq!(
            actual.point.error_pm(),
            error,
            "#837 error at {target}permille"
        );
        assert_eq!(actual.point.ucb_pm(), ucb, "#837 UCB at {target}permille");
    }
    assert_eq!(report.budget, ArmBudget::new(644, 2, 1, 7));
    report
}

fn evaluate_at(scored: &[(i64, bool)], theta: i64) -> OperatingPoint {
    let mut served = 0u64;
    let mut wrong = 0u64;
    for &(score, correct) in scored {
        if score >= theta {
            served += 1;
            wrong += u64::from(!correct);
        }
    }
    OperatingPoint {
        theta,
        served,
        wrong,
        n: scored.len() as u64,
    }
}

#[derive(Debug, Clone, Serialize)]
struct NullReport {
    name: String,
    changed_fit_pairings: u64,
    changed_cal_pairings: u64,
    arms: Vec<ArmReport>,
    pass: bool,
}

fn selectable_reports(
    rows: &[(ContentFeatures, bool)],
    content: &ContentBucketTable,
    suffix: &SuffixBucketTable,
) -> Vec<ArmReport> {
    Arm::SELECTABLE
        .into_iter()
        .map(|arm| report_arm(arm, rows, content, suffix))
        .collect()
}

fn make_null_report(
    name: &str,
    changed_fit_pairings: u64,
    changed_cal_pairings: u64,
    rows: &[(ContentFeatures, bool)],
    content: &ContentBucketTable,
    suffix: &SuffixBucketTable,
) -> NullReport {
    let arms = selectable_reports(rows, content, suffix);
    let pass = changed_fit_pairings > 0
        && changed_cal_pairings > 0
        && arms
            .iter()
            .all(|report| report.release.is_none() && report.research.is_none());
    NullReport {
        name: name.to_owned(),
        changed_fit_pairings,
        changed_cal_pairings,
        arms,
        pass,
    }
}

fn permute_labels(rows: &[(ContentFeatures, bool)]) -> (Vec<(ContentFeatures, bool)>, u64) {
    let permutation = lcg_permutation(rows.len());
    let shuffled: Vec<(ContentFeatures, bool)> = rows
        .iter()
        .enumerate()
        .map(|(index, (features, _))| (*features, rows[permutation[index]].1))
        .collect();
    let changed = rows
        .iter()
        .zip(&shuffled)
        .filter(|((_, original), (_, permuted))| original != permuted)
        .count() as u64;
    (shuffled, changed)
}

fn permute_features(rows: &[(ContentFeatures, bool)]) -> (Vec<(ContentFeatures, bool)>, u64) {
    let permutation = lcg_permutation(rows.len());
    let shuffled: Vec<(ContentFeatures, bool)> = rows
        .iter()
        .enumerate()
        .map(|(index, (_, correct))| (rows[permutation[index]].0, *correct))
        .collect();
    let changed = rows
        .iter()
        .zip(&shuffled)
        .filter(|((original, _), (permuted, _))| original != permuted)
        .count() as u64;
    (shuffled, changed)
}

fn key_rows(
    prepared: &Prepared,
    observations: &[Observation],
) -> (Vec<(ContentFeatures, bool)>, u64) {
    let ordered = canonical_observations(observations);
    let changed = ordered
        .iter()
        .filter(|observation| observation.features != observation.shuffled_key_features)
        .count() as u64;
    let rows = ordered
        .into_iter()
        .map(|observation| {
            (
                observation.shuffled_key_features,
                observation.skip_token == Some(prepared.corpus.t_argmax[observation.position]),
            )
        })
        .collect();
    (rows, changed)
}

fn cid_of_json(value: &Value) -> String {
    compute_cid(&serde_json::to_vec(value).expect("deterministic compact JSON"))
}

fn semantic_suite_unavailable_evidence() -> Value {
    let manifest_bytes = std::fs::read(
        repo_root().join("crates/uor-r4-api/capability_suites/answerability_ood.json"),
    )
    .expect("#838 answerability suite manifest");
    let constitution_bytes =
        std::fs::read(repo_root().join("crates/uor-r4-api/capability_suites/constitution.json"))
            .expect("#832 capability-suite constitution");
    let manifest: Value = serde_json::from_slice(&manifest_bytes).expect("suite manifest JSON");
    let constitution: Value =
        serde_json::from_slice(&constitution_bytes).expect("suite constitution JSON");
    assert_eq!(manifest["schema"], 1);
    assert_eq!(manifest["id"], "s2-answerability-ood");
    assert_eq!(manifest["report_schema"], 1);
    assert_eq!(constitution["schema"], 1);
    assert_eq!(
        constitution["stages"]["s2"]["primary_suite"],
        "s2-answerability-ood"
    );
    json!({
        "status": "UNAVAILABLE",
        "suite_id": "s2-answerability-ood",
        "suite_manifest": {
            "cid": compute_cid(&manifest_bytes),
            "schema": manifest["schema"],
            "report_schema": manifest["report_schema"],
            "stage": manifest["stage"],
            "workload": manifest["workload"],
            "mode": manifest["mode"],
            "primary_metric": manifest["primary_metric"],
            "required_identities": manifest["required_identities"],
        },
        "constitution": {
            "cid": compute_cid(&constitution_bytes),
            "schema": constitution["schema"],
            "s2_primary_suite": constitution["stages"]["s2"]["primary_suite"],
        },
        "generator_configuration_cid": null,
        "annotation_set_cid": null,
        "rubric_cid": null,
        "split_assignment_cid": null,
        "n_per_category": 600,
        "n_total": 4_800,
        "categories": ["in-domain-answerable", "paraphrased-answerable", "novel-but-supported", "missing-evidence", "private-information", "false-premise", "contradictory-evidence", "unrelated-ood"],
        "disjointness_axes": ["document", "domain", "entity", "template"],
        "reason": "the real CID-bound 8-category, four-axis-disjoint annotation fixture is not committed; planted constitution fixtures are structural teeth, not empirical evidence"
    })
}

fn selected_model_evidence(
    selected: Option<(Arm, OperatingPoint)>,
    content: &ContentBucketTable,
    suffix: &SuffixBucketTable,
    corpus_cid: &str,
    skip_graph_cid: &str,
) -> Option<Value> {
    selected.map(|(arm, point)| {
        let content_table = matches!(arm, Arm::ContentSupportMargin | Arm::HybridContentSuffix)
            .then(|| json!(content));
        let suffix_table = (arm == Arm::HybridContentSuffix).then(|| json!(suffix));
        let content_table_cid = content_table.as_ref().map(cid_of_json);
        let suffix_table_cid = suffix_table.as_ref().map(cid_of_json);
        let model = json!({
            "arm": arm.label(),
            "threshold": point.theta,
            "calibration_point": point,
            "budget": arm.budget(),
            "corpus_meta_cid": corpus_cid,
            "skip_graph_cid": skip_graph_cid,
            "feature_extractor": "rf31-packed-content-features-v1",
            "content_table_cid": content_table_cid,
            "locked_837_suffix_table_cid": suffix_table_cid,
        });
        json!({
            "model": model,
            "model_cid": cid_of_json(&model),
            "content_table": content_table,
            "locked_837_suffix_table": suffix_table,
        })
    })
}

fn print_instrument(instrument: &Instrument) {
    let pm = |count: u64| count * 1000 / instrument.n.max(1);
    println!("=== #931 binding instrument (FIT labels only) ===");
    println!("fit positions       : {}", instrument.n);
    println!(
        "feature exposure    : winner-support {}permille; margin {}permille; lane-changed {}permille; injected {}permille",
        pm(instrument.support),
        pm(instrument.margin),
        pm(instrument.changed),
        pm(instrument.injected),
    );
    println!(
        "key-shuffle changes : {} / {} ({}permille)",
        instrument.key_shuffle_changed,
        instrument.n,
        pm(instrument.key_shuffle_changed),
    );
    println!(
        "deployed D4 policy : eligible {} / {}; abstained {}; exact feature cells {}; feature positions checked {}; packed compiler rows {}/{}",
        instrument.deployed_eligible,
        instrument.n,
        instrument.deployed_abstained,
        instrument.exact_feature_cells,
        instrument.packed_reference_feature_positions_checked,
        instrument.packed_compiler_joint_rows_checked,
        instrument.packed_compiler_fallback_rows_checked,
    );
    println!(
        "oracle release/research: {:?} / {:?}",
        instrument.release_oracle, instrument.research_oracle
    );
    println!(
        "semantic suite      : UNAVAILABLE (constitution/planted fixtures exist; real 4,800-item annotation fixture absent)"
    );
    println!(
        "INSTRUMENT VERDICT  : {}",
        if instrument.proceed {
            "PROCEED -- identities, exposure, variance, packed/reference differential, key null, and FIT oracle clear the gate"
        } else {
            "STOP -- at least one binding precondition failed; do not consume CAL or TEST"
        }
    );
}

#[test]
#[ignore = "bundle-gated binding instrument; run before the full #931 study"]
fn selective_calibrator_reentry_instrument_931() {
    verify_prior_records();
    let started = Instant::now();
    let prepared = prepare().expect("UNAVAILABLE: canonical #833 ServingBundle is required");
    let fit_observations = observations(&prepared, 0);
    let instrument = instrument(&prepared, &fit_observations);
    let fit_rows = label_rows(&prepared, &fit_observations);
    let base_hits = fit_observations
        .iter()
        .filter(|observation| {
            observation.base_token == Some(prepared.corpus.t_argmax[observation.position])
        })
        .count();
    let skip_hits = fit_rows.iter().filter(|(_, correct)| *correct).count();
    let suffix_hits = suffix_label_rows(&prepared, &fit_observations)
        .iter()
        .filter(|(_, correct)| *correct)
        .count();
    println!("corpus CID          : {}", prepared.corpus_cid);
    println!(
        "regenerated graphs  : base {} | skip {}",
        compute_cid(&prepared.base_graph),
        compute_cid(&prepared.skip_graph)
    );
    println!(
        "FIT top1            : base {base_hits}/{} | skip {skip_hits}/{} | locked suffix {suffix_hits}/{}",
        fit_observations.len(),
        fit_observations.len(),
        fit_observations.len(),
    );
    print_instrument(&instrument);
    println!(
        "elapsed             : {:.1}s",
        started.elapsed().as_secs_f64()
    );
}

fn correctness_count(
    prepared: &Prepared,
    observations: &[Observation],
    token: impl Fn(&Observation) -> Option<u32>,
) -> u64 {
    observations
        .iter()
        .filter(|observation| {
            token(observation) == Some(prepared.corpus.t_argmax[observation.position])
        })
        .count() as u64
}

#[test]
#[ignore = "bundle-gated full #931 study; run only after the binding instrument says PROCEED"]
fn selective_calibrator_reentry_full_931() {
    verify_prior_records();
    let started = Instant::now();
    let prepared = prepare().expect("UNAVAILABLE: canonical #833 ServingBundle is required");

    // FIT labels are the only labels touched before the instrument verdict.
    let fit_observations = observations(&prepared, 0);
    let binding = instrument(&prepared, &fit_observations);
    print_instrument(&binding);
    assert!(
        binding.proceed,
        "binding instrument said STOP; CAL/TEST stay sealed"
    );

    let fit = label_rows(&prepared, &fit_observations);
    let suffix_fit = suffix_label_rows(&prepared, &fit_observations);
    let content_table = ContentBucketTable::fit(&fit);
    let locked_suffix_table = SuffixBucketTable::fit(&suffix_fit);

    // The binding instrument passed, so CAL may now be scored. TEST is not
    // constructed or labelled unless a selectable arm release-qualifies.
    let cal_observations = observations(&prepared, 1);
    let cal = label_rows(&prepared, &cal_observations);
    let suffix_cal = suffix_label_rows(&prepared, &cal_observations);
    let locked_837_reference = locked_837_reference_report(&suffix_cal, &locked_suffix_table);
    let all_arms = [
        Arm::ContentMargin,
        Arm::ContentSupportMargin,
        Arm::HybridContentSuffix,
        Arm::SuffixScoreOnSkipmix,
        Arm::WinnerSupportOnly,
        Arm::BaseMarginOnly,
        Arm::CurrentD4,
        Arm::Constant,
        Arm::InvertedContentMargin,
    ];
    let arm_reports: Vec<ArmReport> = all_arms
        .into_iter()
        .map(|arm| report_arm(arm, &cal, &content_table, &locked_suffix_table))
        .collect();
    for report in &arm_reports {
        println!(
            "CAL {:<24} release {:?} research {:?}",
            report.arm, report.release, report.research
        );
        for curve in &report.curve {
            println!(
                "  @{}permille: served={} wrong={} err={}permille ucb={}",
                curve.target_coverage_pm,
                curve.point.served,
                curve.point.wrong,
                curve.point.error_pm(),
                curve.point.ucb_pm(),
            );
        }
    }

    let research_only: Vec<String> = arm_reports
        .iter()
        .filter(|report| Arm::SELECTABLE.iter().any(|arm| arm.label() == report.arm))
        .filter(|report| report.release.is_none() && report.research.is_some())
        .map(|report| report.arm.clone())
        .collect();
    let release_gate_only: Vec<String> = arm_reports
        .iter()
        .filter(|report| Arm::SELECTABLE.iter().any(|arm| arm.label() == report.arm))
        .filter(|report| report.release.is_some())
        .map(|report| report.arm.clone())
        .collect();

    // Real falsifiers. Every selectable arm is evaluated under every null,
    // with both folds first put into canonical stable-position order.
    let (shuffled_label_fit, label_fit_changed) = permute_labels(&fit);
    let (shuffled_label_cal, label_cal_changed) = permute_labels(&cal);
    let shuffled_label_table = ContentBucketTable::fit(&shuffled_label_fit);
    let label_null = make_null_report(
        "label-shuffle",
        label_fit_changed,
        label_cal_changed,
        &shuffled_label_cal,
        &shuffled_label_table,
        &locked_suffix_table,
    );

    let (key_fit, key_fit_changed) = key_rows(&prepared, &fit_observations);
    let (key_cal, key_cal_changed) = key_rows(&prepared, &cal_observations);
    let key_table = ContentBucketTable::fit(&key_fit);
    let key_null = make_null_report(
        "content-key-last-token-shuffle",
        key_fit_changed,
        key_cal_changed,
        &key_cal,
        &key_table,
        &locked_suffix_table,
    );

    let (shuffled_feature_fit, feature_fit_changed) = permute_features(&fit);
    let (shuffled_feature_cal, feature_cal_changed) = permute_features(&cal);
    let shuffled_feature_table = ContentBucketTable::fit(&shuffled_feature_fit);
    let feature_null = make_null_report(
        "feature-shuffle",
        feature_fit_changed,
        feature_cal_changed,
        &shuffled_feature_cal,
        &shuffled_feature_table,
        &locked_suffix_table,
    );
    let nulls_pass = label_null.pass && key_null.pass && feature_null.pass;
    println!(
        "NULLS               : label changes {}/{} pass={}; key changes {}/{} pass={}; feature changes {}/{} pass={}; all={nulls_pass}",
        label_null.changed_fit_pairings,
        label_null.changed_cal_pairings,
        label_null.pass,
        key_null.changed_fit_pairings,
        key_null.changed_cal_pairings,
        key_null.pass,
        feature_null.changed_fit_pairings,
        feature_null.changed_cal_pairings,
        feature_null.pass,
    );

    let truly_content_caused = |arm: Arm, point: OperatingPoint| {
        canonical_observations(&cal_observations)
            .into_iter()
            .filter(|observation| {
                !observation.features.suffix_present
                    && observation.features.deployed_eligible
                    && observation.skip_token != observation.base_token
                    && observation.skip_token
                        == Some(prepared.corpus.t_argmax[observation.position])
                    && arm.score(&observation.features, &content_table, &locked_suffix_table)
                        >= point.theta
            })
            .count() as u64
    };
    let answerable_novel_cal = cal_observations
        .iter()
        .filter(|observation| {
            !observation.features.suffix_present
                && observation.skip_token != observation.base_token
                && observation.skip_token == Some(prepared.corpus.t_argmax[observation.position])
        })
        .count() as u64;

    let mut selected: Option<(Arm, OperatingPoint)> = None;
    let mut candidate_admission = Vec::new();
    for arm in Arm::SELECTABLE {
        let release = arm_reports
            .iter()
            .find(|report| report.arm == arm.label())
            .and_then(|report| report.release);
        let (reference_point, beats_reference, novel_served) =
            release.map_or((None, false, 0), |point| {
                let matched = locked_837_reference
                    .full_curve
                    .iter()
                    .find(|reference| reference.served >= point.served)
                    .copied();
                let beats = matched.is_some_and(|reference| point.ucb_pm() < reference.ucb_pm());
                (matched, beats, truly_content_caused(arm, point))
            });
        let admissible = release.is_some() && beats_reference && novel_served > 0;
        candidate_admission.push(json!({
            "arm": arm.label(),
            "release": release,
            "matched_locked_837_reference": reference_point,
            "strictly_lower_ucb_than_reference": beats_reference,
            "content_caused_novel_served": novel_served,
            "admissible": admissible,
        }));
        if let Some(point) = release.filter(|_| admissible) {
            // Exact served count is the selection key. Iteration is frozen
            // budget order, so equality retains the simpler earlier arm.
            if selected.is_none_or(|(_, best)| point.served > best.served) {
                selected = Some((arm, point));
            }
        }
    }
    let approved_selection = selected.filter(|_| nulls_pass);
    let selected_novel_served_cal =
        approved_selection.map(|(arm, point)| truly_content_caused(arm, point));

    let deployed_eligible_cal = cal
        .iter()
        .filter(|(features, _)| features.deployed_eligible)
        .count() as u64;
    assert!(deployed_eligible_cal > 0, "D4 eligibility is non-vacuous");
    for report in &arm_reports {
        assert!(
            report
                .full_curve
                .iter()
                .all(|point| point.served <= deployed_eligible_cal),
            "{} must not serve a token after deployed D4 abstains",
            report.arm
        );
    }
    let d4_report = arm_reports
        .iter()
        .find(|report| report.arm == Arm::CurrentD4.label())
        .expect("current-D4 control");
    assert_eq!(d4_report.full_curve.len(), 1);
    assert_eq!(d4_report.full_curve[0].served, deployed_eligible_cal);

    let mut test_observations: Option<Vec<Observation>> = None;
    let test_evaluation = if let Some((arm, cal_point)) = approved_selection {
        let observations = observations(&prepared, 2);
        let test = label_rows(&prepared, &observations);
        let scored = scored_rows(arm, &test, &content_table, &locked_suffix_table);
        let evaluation = evaluate_at(&scored, cal_point.theta);
        test_observations = Some(observations);
        Some((arm, evaluation))
    } else {
        None
    };

    let verdict = match (selected, test_evaluation, nulls_pass) {
        (Some((arm, _)), Some((_, evaluation)), true)
            if evaluation.ucb_pm() <= RELEASE_UCB_PM
                && evaluation.coverage_pm() >= RELEASE_COVERAGE_PM =>
        {
            format!(
                "SELECT-CANDIDATE: {} confirms on untouched TEST at theta={} coverage={}permille UCB95={}permille; activate #839 phase 2 without retuning",
                arm.label(),
                evaluation.theta,
                evaluation.coverage_pm(),
                evaluation.ucb_pm(),
            )
        }
        (Some((arm, cal_point)), Some((_, evaluation)), true) => format!(
            "NO CALIBRATOR ESTABLISHED: {} qualified on CAL at theta={} but untouched TEST coverage={}permille UCB95={}permille missed the frozen release gate",
            arm.label(),
            cal_point.theta,
            evaluation.coverage_pm(),
            evaluation.ucb_pm(),
        ),
        (_, _, false) => "NO CALIBRATOR ESTABLISHED: a shuffled-label, shuffled-feature, or content-key null qualified; content-specific confidence is not isolated".to_owned(),
        (None, None, true) if !release_gate_only.is_empty() => format!(
            "NO CALIBRATOR ESTABLISHED: {} met the CAL release proxy but failed the locked-#837 improvement or content-caused novelty admission requirement; TEST stayed untouched",
            release_gate_only.join(", ")
        ),
        (None, None, true) if !research_only.is_empty() => format!(
            "RESEARCH-ONLY / NO PRODUCTION CALIBRATOR: {} met the research gate on CAL; no release candidate and TEST stayed untouched",
            research_only.join(", ")
        ),
        _ => "NO CALIBRATOR ESTABLISHED: no selectable arm met the frozen release or research gate on CAL; TEST stayed untouched and #839 remains legacy-only".to_owned(),
    };

    // Only a legitimately opened TEST permits an all-heldout recount. The
    // predecessor CIDs/counts were already verified above for negative runs.
    let all_heldout_reproduction = test_observations.as_ref().map(|test| {
        let parts = [&fit_observations, &cal_observations, test];
        let base_hits: u64 = parts
            .iter()
            .map(|part| correctness_count(&prepared, part, |row| row.base_token))
            .sum();
        let skip_hits: u64 = parts
            .iter()
            .map(|part| correctness_count(&prepared, part, |row| row.skip_token))
            .sum();
        let changed: u64 = parts
            .iter()
            .map(|part| {
                part.iter()
                    .filter(|row| row.base_token != row.skip_token)
                    .count() as u64
            })
            .sum();
        assert_eq!(base_hits, PRIOR_908_BASE_HITS);
        assert_eq!(skip_hits, PRIOR_908_SKIP_HITS);
        assert_eq!(changed, PRIOR_908_CHANGED);
        json!({"base_hits": base_hits, "skip_hits": skip_hits, "changed": changed})
    });

    let ablations = json!({
        "cal_positions": cal_observations.len(),
        "base_top1": correctness_count(&prepared, &cal_observations, |row| row.base_token),
        "skipmix_top1": correctness_count(&prepared, &cal_observations, |row| row.skip_token),
        "suffix_top1": correctness_count(&prepared, &cal_observations, |row| Some(row.suffix_token)),
        "skmx_only_top1": correctness_count(&prepared, &cal_observations, |row| row.skmx_only_token),
        "psib_only_top1": correctness_count(&prepared, &cal_observations, |row| row.psib_only_token),
        "no_injection_top1": correctness_count(&prepared, &cal_observations, |row| row.no_injection_token),
    });
    let always_serve_scored: Vec<(i64, bool)> =
        cal.iter().map(|(_, correct)| (1, *correct)).collect();
    let always_decline_scored: Vec<(i64, bool)> = cal
        .iter()
        .map(|(_, correct)| (i64::MIN, *correct))
        .collect();
    let injection_without_confidence_scored =
        scored_rows(Arm::CurrentD4, &cal, &content_table, &locked_suffix_table);
    let always_serve_report = report_scored(
        "always-serve",
        ArmBudget::new(0, 0, 0, 0),
        &always_serve_scored,
    );
    let always_decline_report = report_scored(
        "always-decline",
        ArmBudget::new(0, 0, 0, 0),
        &always_decline_scored,
    );
    let injection_without_confidence_report = report_scored(
        "candidate-injection-without-confidence",
        ArmBudget::new(0, 0, 0, 0),
        &injection_without_confidence_scored,
    );
    let controls = json!({
        "always_serve": always_serve_report,
        "always_decline": {
            "report": always_decline_report,
            "endpoint": {
                "n": cal.len(),
                "served": 0,
                "wrong": 0,
                "coverage_permille": 0,
                "error_permille": "UNAVAILABLE",
                "ucb95_permille": "UNAVAILABLE"
            }
        },
        "injection_without_confidence": {
            "report": injection_without_confidence_report,
            "cal_top1": ablations["skipmix_top1"],
        },
    });
    let skip_graph_cid = compute_cid(&prepared.skip_graph);
    let cal_candidate_json = selected_model_evidence(
        approved_selection,
        &content_table,
        &locked_suffix_table,
        &prepared.corpus_cid,
        &skip_graph_cid,
    );
    let confirmed_selection = match (approved_selection, test_evaluation) {
        (Some(candidate), Some((test_arm, evaluation)))
            if candidate.0 == test_arm
                && evaluation.ucb_pm() <= RELEASE_UCB_PM
                && evaluation.coverage_pm() >= RELEASE_COVERAGE_PM =>
        {
            Some(candidate)
        }
        _ => None,
    };
    let selected_json = selected_model_evidence(
        confirmed_selection,
        &content_table,
        &locked_suffix_table,
        &prepared.corpus_cid,
        &skip_graph_cid,
    );
    let test_json = test_evaluation.map(|(arm, point)| json!({"arm": arm.label(), "point": point}));
    let result = json!({
        "issue": 931,
        "study": "rf31-content-evidence-selective-calibrator-reentry",
        "execution_scope": "offline compiler/certifier analysis of production-equivalent RF-31 packed rows",
        "corpus_meta_cid": prepared.corpus_cid,
        "base_graph_cid": compute_cid(&prepared.base_graph),
        "skip_graph_cid": compute_cid(&prepared.skip_graph),
        "predecessor_exact": {
            "issue_837": {
                "result_cid": PRIOR_837_RESULT_CID,
                "base_permille": 246.6,
                "all_heldout_positions": 72_130,
                "partition_positions": PARTITION_POSITIONS,
                "partition_stories": PARTITION_STORIES,
                "answerable_novel": {"novel_total": 26_002, "content_answerable": 2_454, "suffix_feature_served": 0},
                "locked_suffix_reference": locked_837_reference,
            },
            "issue_908": {
                "result_cid": PRIOR_908_RESULT_CID,
                "base_graph_cid": BASE_GRAPH_CID,
                "skip_graph_cid": SKIP_GRAPH_CID,
                "base_top1": PRIOR_908_BASE_HITS,
                "skipmix_top1": PRIOR_908_SKIP_HITS,
                "reachability_changed": PRIOR_908_CHANGED,
                "all_heldout_positions": 72_130,
            }
        },
        "partitions": {"positions": PARTITION_POSITIONS, "stories": PARTITION_STORIES, "test_opened": test_observations.is_some()},
        "frozen_gates": {"release_ucb_permille": RELEASE_UCB_PM, "release_coverage_permille": RELEASE_COVERAGE_PM, "research_ucb_permille": RESEARCH_UCB_PM, "research_coverage_permille": RESEARCH_COVERAGE_PM},
        "instrument": binding,
        "feature_evidence": {
            "fit": feature_evidence(&fit_observations),
            "calibration": feature_evidence(&cal_observations),
            "test": test_observations.as_ref().map(|observations| feature_evidence(observations)),
        },
        "arms": arm_reports,
        "controls": controls,
        "nulls": {"label_shuffle": label_null, "content_key_shuffle": key_null, "feature_shuffle": feature_null, "pass": nulls_pass},
        "candidate_admission": candidate_admission,
        "cal_candidate": cal_candidate_json,
        "ablations": ablations,
        "answerable_novel_proxy": {"calibration_total": answerable_novel_cal, "selected_served": selected_novel_served_cal},
        "selected": selected_json,
        "test_evaluation": test_json,
        "all_heldout_reproduction": all_heldout_reproduction,
        "semantic_answerability_suite": semantic_suite_unavailable_evidence(),
        "semantic_abstention_claim": "NOT ESTABLISHED",
        "verdict": verdict,
    });
    let canonical = serde_json::to_vec(&result).expect("canonical result JSON");
    let result_cid = compute_cid(&canonical);
    let report = json!({
        "result": result,
        "result_cid_scope": "compact deterministic JSON bytes of result",
        "result_cid": result_cid,
    });
    let mut bytes = serde_json::to_vec_pretty(&report).expect("pretty result JSON");
    bytes.push(b'\n');
    let out = repo_root().join("docs/selective_calibrator_reentry_931_result.json");
    std::fs::write(&out, &bytes).expect("write #931 result record");
    println!("VERDICT             : {}", report["result"]["verdict"]);
    println!("result CID          : {result_cid}");
    println!("result path         : {}", out.display());
    println!(
        "elapsed             : {:.1}s",
        started.elapsed().as_secs_f64()
    );
}

fn fixture_rows() -> Vec<(ContentFeatures, bool)> {
    (0..300u32)
        .map(|i| {
            let strong = i % 10 < 4;
            // The strongest honest slice still contains one planted error,
            // so the label-derived leakage control remains distinguishable.
            let correct = if strong { i != 293 } else { i % 7 == 0 };
            let margin = if strong { 20_000 + i } else { i % 50 };
            (
                ContentFeatures {
                    deployed_eligible: i % 17 != 0,
                    joint_rows: if strong { 4 } else { 1 },
                    fallback_rows: if strong { 2 } else { 0 },
                    winner_support: if strong { 5 } else { 1 },
                    conflicting_rows: if correct { 0 } else { 3 },
                    winner_contribution: margin as i32 + 100,
                    runner_contribution: 100,
                    content_margin: margin,
                    base_agreement: !strong,
                    injected_candidate: strong,
                    base_margin: i % 31,
                    candidate_count: 8,
                    suffix_present: i % 3 != 0,
                    suffix_total: i % 19,
                    suffix_margin_pm: i % 1000,
                },
                correct,
            )
        })
        .collect()
}

#[test]
fn packed_feature_extraction_obeys_rf31_semantics() {
    let skmx =
        build_skipmix_table(&[(1, 9, vec![(10, 5)]), (2, 9, vec![(20, 7)])]).expect("SKMX fixture");
    let psib =
        build_psi_bag_table(&[(1, vec![(30, 999)]), (3, vec![(30, 8)])]).expect("PSIB fixture");
    let primary = SkipmixTable::parse(&skmx).expect("parse SKMX");
    let fallback = PsiBagTable::parse(&psib).expect("parse PSIB");
    let (scores, joint, fallback_count) =
        collect_contributions(Some(&primary), Some(&fallback), &[1, 3], 9, false);
    assert_eq!((joint, fallback_count), (1, 1));
    assert_eq!(scores.get(&10), Some(&5));
    assert_eq!(scores.get(&30), Some(&8));
    assert_eq!(
        scores.get(&30),
        Some(&8),
        "primary row presence suppresses the stronger PSIB row for content=1"
    );
    let ranked = [(99, 50_000)];
    assert_eq!(select_winner(&ranked, &scores, 99, true), 30);
    assert_eq!(select_winner(&ranked, &scores, 99, false), 99);
    assert_eq!(
        select_winner(&[], &scores, 99, true),
        99,
        "production RF-31 returns the base token before candidate injection when ranked is empty"
    );

    let inputs = FeatureInputs {
        primary: Some(&primary),
        fallback: Some(&fallback),
        window: &[1, 3, 9],
        ranked: &ranked,
        base_token: 99,
        winner: 30,
        suffix: (true, 10, 200),
        shuffled_pairing: false,
    };
    let extracted = content_features(&inputs);
    assert_eq!(extracted, reference_content_features(&inputs));
    assert_eq!(
        extracted,
        ContentFeatures {
            deployed_eligible: true,
            joint_rows: 1,
            fallback_rows: 1,
            winner_support: 1,
            conflicting_rows: 1,
            winner_contribution: 8,
            runner_contribution: 5,
            content_margin: 3,
            base_agreement: false,
            injected_candidate: true,
            base_margin: 50_000,
            candidate_count: 3,
            suffix_present: true,
            suffix_total: 10,
            suffix_margin_pm: 200,
        }
    );

    let quantization_rows = [(extracted, true), (extracted, false), (extracted, true)];
    let content_table = ContentBucketTable::fit(&quantization_rows);
    let suffix_table = SuffixBucketTable::fit(&quantization_rows);
    assert_eq!(
        content_table.score(&extracted),
        666,
        "u16 table cell truncates"
    );
    assert_eq!(
        suffix_table.score(&extracted),
        666,
        "historical u32 cell truncates"
    );
    assert_eq!(std::mem::size_of_val(&content_table.precision_pm), 288);
    assert_eq!(std::mem::size_of_val(&suffix_table.precision_pm), 640);
    assert_eq!(
        Arm::HybridContentSuffix.budget(),
        ArmBudget::new(932, 4, 2, 14)
    );
    assert_eq!(
        Arm::SuffixScoreOnSkipmix.budget(),
        ArmBudget::new(644, 2, 1, 7)
    );

    let tie = BTreeMap::from([(4, 7), (3, 7)]);
    assert_eq!(select_winner(&ranked, &tie, 99, true), 3);
    assert_eq!(
        select_winner(&ranked, &BTreeMap::new(), 99, true),
        99,
        "absent sections are identity"
    );
    let absent_inputs = FeatureInputs {
        primary: None,
        fallback: None,
        window: &[1, 3, 9],
        ranked: &ranked,
        base_token: 99,
        winner: 99,
        suffix: (false, 0, 0),
        shuffled_pairing: false,
    };
    let absent = content_features(&absent_inputs);
    assert_eq!(absent, reference_content_features(&absent_inputs));
    assert_eq!(
        absent,
        ContentFeatures {
            deployed_eligible: true,
            base_agreement: true,
            base_margin: 50_000,
            candidate_count: 1,
            ..ContentFeatures::default()
        },
        "missing SKMX/PSIB sections preserve the served base row and expose zero content evidence"
    );
}

#[test]
fn packed_tables_reject_corruption_and_saturate() {
    let skmx = build_skipmix_table(&[(1, 9, vec![(7, i32::MAX)]), (2, 9, vec![(7, i32::MAX)])])
        .expect("SKMX fixture");
    let table = SkipmixTable::parse(&skmx).expect("parse SKMX");
    let (scores, _, _) = collect_contributions(Some(&table), None, &[1, 2], 9, false);
    assert_eq!(scores.get(&7), Some(&i32::MAX), "sum saturates");
    assert!(SkipmixTable::parse(&skmx[..skmx.len() - 1]).is_err());
    let mut corrupt = skmx.clone();
    corrupt[0] ^= 0xff;
    assert!(SkipmixTable::parse(&corrupt).is_err());

    let psib = build_psi_bag_table(&[(1, vec![(7, 1)])]).expect("PSIB fixture");
    assert!(PsiBagTable::parse(&psib[..psib.len() - 1]).is_err());
    let mut corrupt = psib;
    corrupt[0] ^= 0xff;
    assert!(PsiBagTable::parse(&corrupt).is_err());
}

#[test]
fn calibration_teeth_reject_nulls_and_leaks() {
    let rows = fixture_rows();
    let content = ContentBucketTable::fit(&rows);
    let suffix = SuffixBucketTable::fit(&rows);
    let bound = 100;
    let floor = 100;
    let missing = ContentFeatures::default();
    for arm in Arm::SELECTABLE.into_iter().chain([Arm::CurrentD4]) {
        assert_eq!(
            arm.score(&missing, &content, &suffix),
            i64::MIN,
            "{} cannot override a deployed D4 abstention",
            arm.label()
        );
    }
    assert!(qualify(
        &sweep(&scored_rows(Arm::ContentMargin, &rows, &content, &suffix)),
        bound,
        floor,
    )
    .is_some());
    for arm in [Arm::Constant, Arm::InvertedContentMargin] {
        assert!(qualify(
            &sweep(&scored_rows(arm, &rows, &content, &suffix)),
            bound,
            floor,
        )
        .is_none());
    }
    let permutation = lcg_permutation(rows.len());
    let shuffled: Vec<(ContentFeatures, bool)> = rows
        .iter()
        .enumerate()
        .map(|(i, (_, ok))| (rows[permutation[i]].0, *ok))
        .collect();
    assert!(qualify(
        &sweep(&scored_rows(
            Arm::ContentMargin,
            &shuffled,
            &content,
            &suffix,
        )),
        bound,
        floor,
    )
    .is_none());

    let leaked: Vec<(i64, bool)> = rows.iter().map(|(_, ok)| (i64::from(*ok), *ok)).collect();
    assert_eq!(sweep(&leaked).first().expect("leak point").wrong, 0);
    assert!(
        sweep(&scored_rows(Arm::ContentMargin, &rows, &content, &suffix))
            .iter()
            .all(|point| point.wrong > 0),
        "honest fixture score is not perfectly label-derived"
    );

    for arm in Arm::SELECTABLE {
        for (features, _) in &rows {
            let integer_score = arm.score(features, &content, &suffix);
            if integer_score == i64::MIN {
                continue;
            }
            let reference_score = match arm {
                Arm::ContentMargin => f64::from(features.content_margin),
                Arm::ContentSupportMargin => f64::from(content.score(features)),
                Arm::HybridContentSuffix => {
                    f64::from(content.score(features)) + f64::from(suffix.score(features))
                }
                _ => unreachable!("selectable arms are exhaustive"),
            };
            assert_eq!(
                integer_score as f64,
                reference_score,
                "{} integer/reference score differential",
                arm.label()
            );
        }
    }

    let mut planted = rows[0].0;
    planted.deployed_eligible = true;
    planted.content_margin = 200;
    let actual_content_margin_score = Arm::ContentMargin.score(&planted, &content, &suffix);
    let fractional_decision = 0.29_f64 * actual_content_margin_score as f64 >= 58.0;
    let integer_decision = actual_content_margin_score.saturating_mul(29) / 100 >= 58;
    assert_ne!(
        fractional_decision, integer_decision,
        "planted fractional/reference drift must change a boundary decision"
    );
}

#[test]
fn partition_and_result_determinism() {
    let mut keys: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for story in 100..160u32 {
        keys[(story % 3) as usize].push(format!("story-{story}"));
    }
    for i in 0..3 {
        for j in i + 1..3 {
            let a: Vec<&str> = keys[i].iter().map(String::as_str).collect();
            let b: Vec<&str> = keys[j].iter().map(String::as_str).collect();
            assert!(detect_document_leakage(&a, &b).is_none());
        }
    }
    assert!(detect_document_leakage(&["story-101"], &["story-101"]).is_some());

    let rows = fixture_rows();
    let a = ContentBucketTable::fit(&rows);
    let mut reversed = rows.clone();
    reversed.reverse();
    let b = ContentBucketTable::fit(&reversed);
    assert_eq!(a, b);
    let points_a = sweep(
        &rows
            .iter()
            .map(|(f, ok)| (i64::from(a.score(f)), *ok))
            .collect::<Vec<_>>(),
    );
    let points_b = sweep(
        &reversed
            .iter()
            .map(|(f, ok)| (i64::from(b.score(f)), *ok))
            .collect::<Vec<_>>(),
    );
    assert_eq!(points_a, points_b);
    let body_a = serde_json::to_vec(&json!({"points": points_a})).expect("json");
    let body_b = serde_json::to_vec(&json!({"points": points_b})).expect("json");
    assert_eq!(body_a, body_b);
    assert_eq!(compute_cid(&body_a), compute_cid(&body_b));

    let indexed: Vec<(usize, (ContentFeatures, bool))> = rows.iter().copied().enumerate().collect();
    let report_bytes = |mut input: Vec<(usize, (ContentFeatures, bool))>| {
        input.sort_unstable_by_key(|(position, _)| *position);
        let ordered_rows: Vec<(ContentFeatures, bool)> =
            input.iter().map(|(_, row)| *row).collect();
        let content = ContentBucketTable::fit(&ordered_rows);
        let suffix = SuffixBucketTable::fit(&ordered_rows);
        let arms: Vec<ArmReport> = [
            Arm::ContentMargin,
            Arm::ContentSupportMargin,
            Arm::HybridContentSuffix,
            Arm::SuffixScoreOnSkipmix,
            Arm::WinnerSupportOnly,
            Arm::BaseMarginOnly,
            Arm::CurrentD4,
            Arm::Constant,
            Arm::InvertedContentMargin,
        ]
        .into_iter()
        .map(|arm| report_arm(arm, &ordered_rows, &content, &suffix))
        .collect();
        let observations: Vec<Observation> = input
            .iter()
            .map(|(position, (features, _))| Observation {
                position: *position,
                features: *features,
                shuffled_key_features: *features,
                base_token: Some(1),
                skip_token: Some(1),
                suffix_token: 1,
                skmx_only_token: Some(1),
                psib_only_token: Some(1),
                no_injection_token: Some(1),
            })
            .collect();
        let (label_null_rows, label_changed) = permute_labels(&ordered_rows);
        let label_table = ContentBucketTable::fit(&label_null_rows);
        let label_null = make_null_report(
            "label-shuffle",
            label_changed,
            label_changed,
            &label_null_rows,
            &label_table,
            &suffix,
        );
        serde_json::to_vec(&json!({
            "feature_evidence": feature_evidence(&observations),
            "arms": arms,
            "label_null": label_null,
            "content_table_cid": cid_of_json(&json!(content)),
            "suffix_table_cid": cid_of_json(&json!(suffix)),
        }))
        .expect("fixture report JSON")
    };
    let report_a = report_bytes(indexed.clone());
    let mut reordered = indexed;
    reordered.reverse();
    let report_b = report_bytes(reordered);
    assert_eq!(report_a, report_b, "full fixture report is reorder-stable");
    assert_eq!(compute_cid(&report_a), compute_cid(&report_b));
}
