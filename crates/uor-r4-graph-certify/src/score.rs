//! Phase 4 of the graph-compiler plan (§5): semantic transitions and
//! ScoreQ residual emission onto the induced cover, plus the Gate C
//! measurement harness. This module is the COMPILER side — it may use
//! floats (documented below); the scoring path it emits for is the
//! integer-only reference scorer in [`super::score_runtime`].
//!
//! # What is compiled
//!
//! - **Forward transitions E_f** ([`compile_transitions`]): over
//!   consecutive train positions `(t, t+1)` in one story, the active
//!   cloud `A_t` (top-M memberships at each depth — the same
//!   `binary_memberships` semantics the cover and the scorer use) is
//!   crossed with `A_{t+1}` to accumulate `(src region, dst region)`
//!   counts per depth. Per source region the out-degree is bounded
//!   ([`ScoreConfig::transition_out_degree`], default 8) by the
//!   canonical order (weight desc, then dst asc); the edge weight is
//!   `ScoreQ::from_logprob(ln(count / src_total))` with `src_total` the
//!   pre-truncation evidence total. The reverse index E_b is built by
//!   sorting canonical edge IDs by `(dst, src, kind)` — Theorem 7
//!   consistency by construction, verified by
//!   [`verify_theorem_7_wired`] (the full per-node wiring, stronger
//!   than the format's v0 existence approximation).
//! - **Emission residuals ΔE** ([`compile_emissions`]): per region n
//!   with parent p(n),
//!   `ΔE(n,v) = ScoreQ::from_logprob(ln P(v|n) − ln P(v|p(n)))` where
//!   `P(v|n)` is the region's corpus next-token distribution — the
//!   store's top-3 teacher-weighted evidence over the train
//!   observations whose **covered** binary top-1 membership is n:
//!   within the region's calibrated radius, with no nearest-region
//!   fallback assignment (the backoff floor is a routing behavior, not
//!   region content — with fallback assignment, every observation would
//!   land in some region at every depth and deep regions' distributions
//!   would collect the whole corpus; see
//!   [`score_runtime::binary_top1_covered`]). The rule is deterministic
//!   and identical under re-induction and `--cover` reload. The
//!   smoothing rule is configurable ([`Smoothing`], issue #67
//!   calibration); the default is add-one smoothing over the compiled
//!   vocabulary: `P(v|n) = (count_n(v) + 1) / (total_n + V)`. The root
//!   prior B(v) is the level-0 store distribution with the same
//!   quantization and smoothing; the smoothing floor (`ln(1/(total + V))`
//!   under add-one) is baked into the EMIT root header. Each region's
//!   emission list is sparse and
//!   bounded: the top-E tokens by residual score
//!   ([`ScoreConfig::emission_entries`], default 64; selection order
//!   score desc then token asc, storage ascending token). The induced
//!   cover has no explicit overlap nodes, so no interaction residuals
//!   exist this phase — Theorem 10 non-duplication holds by
//!   construction (root-plus-residual decomposition; each contribution
//!   attached to exactly one node).
//! - **EXCT**: the compiler reads the TLS1 graded store as input, then emits
//!   a residualized RX1 table after the v0 storage descriptor. Each retained
//!   prefix entry stores `ΔX(X,v) = Q(ln P(v|X)) − B(v)` as an integer
//!   `ScoreQ`; the deployed scorer performs only table reads and integer
//!   addition. Legacy raw TLS1 graph artifacts remain readable by the
//!   certifier for migration compatibility.
//!
//! # Wire layout (EMIT remainder)
//!
//! ```text
//! root prior block:  u32 entry_count | u32 total_count
//!                    | i32 floor_score_q | u32 reserved(0)
//!                    entry_count × (i32 token, i32 score_q)   [token asc]
//! region lists:      per region (ascending region id), wired by the
//!                    PackedNode emission ranges (emission_start = byte
//!                    offset into the remainder, emission_len = entry
//!                    count ≤ HEAD E), each
//!                    emission_len × (i32 token, i32 score_q)  [token asc]
//! ```
//!
//! EDGE kind tags distinguish E_r (0), E_o (1), E_f (2); the canonical
//! edge array is sorted by `(src, kind, dst)` so each node's refinement
//! children stay contiguous (the `convert_r4g1`/`cover` convention).
//!
//! # Quantization and platform pinning
//!
//! All `ln` quantization is compiler-side f64→f32 through
//! `ScoreQ::from_logprob` — libm-sensitive cross-platform, **macOS-
//! pinned**, exactly the status of the existing κ baseline and the
//! cover's f64 entropy (the D2 canonical deterministic compile mode
//! resolves cross-platform byte equality later). The scoring path
//! itself is integer-only (see `score_runtime`).
//!
//! # Gate C
//!
//! [`evaluate_gate_c`] measures top-1 agreement with the corpus's
//! recorded teacher argmax and bits/token on the held-out partition for
//! four scorers side by side: the OLD Σ-over-cloud formula (kept for
//! comparison — the confirmed double counting lives there), NEW Rule 1
//! (chain-telescoped, no EXCT), NEW Rule 1+2 (with D4 EXCT precedence),
//! and the TLA3 store baseline (`runtime::predict_witness_plain` on the
//! same positions, Witten-Bell bits as in `evaluate-report`), plus
//! per-status (ExactContext/Graph/Novel) and per-rule win/loss
//! instrumentation. This is the M.V.G. checkpoint's fidelity input.

use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;

use super::score_runtime::{
    binary_memberships, binary_top1_covered, regions_from_view, structural_edges_from_view,
    verify_witness_replay, GraphScorer, RegionParams, ScoreStatus, ScoringVariant, StructuralEdge,
    EDGE_KIND_FORWARD, EDGE_KIND_NEIGHBOR, EDGE_KIND_REFINEMENT, RESIDUAL_EXCT_MAGIC,
};
use uor_r4_core::transformerless::compiler::{self, Corpus, SIG_BYTES, SIG_WORDS, STAGES};
use uor_r4_core::transformerless::runtime::{self, Store};
use uor_r4_graph_compiler::induction::{self as cover, Observation};
use uor_r4_graph_format::ScoreQ;

/// Default per-source out-degree cap for E_f edges.
pub const DEFAULT_TRANSITION_OUT_DEGREE: usize = 8;
/// Default per-region emission list bound (top-E by residual score).
pub const DEFAULT_EMISSION_ENTRIES: usize = 64;

/// How much the top-E-by-log-ratio emission selection differs from a
/// top-E-by-probability one.
///
/// Emissions keep the tokens most over-represented against the parent, not the
/// most likely next tokens. `overlap_with_top_count` near 0 means the two
/// selections are nearly disjoint; `probability_mass_kept` says how much of the
/// region's actual next-token mass the emitted list covers. Low values on both
/// mean regions emit distinctive-but-unlikely tokens and score the likely ones
/// at the bare prior.
/// Per-region shrinkage applied to the emission residual (#364).
///
/// Measured on the fixture corpus, `log P_region` is a WORSE next-token
/// predictor than the global unigram prior on graph-status positions: applying
/// the residual at full strength costs top-1 (2.90% vs 8.76%) and bits (+1.32)
/// even where the telescoping chain is complete and the correction is
/// mathematically right. That is the signature of sparse conditional estimates
/// -- roughly 2,600 positions per region against a 32,000-token vocabulary --
/// rather than of a defect in the arithmetic. The alpha sweep peaks near 0.25,
/// the shape of a shrinkage coefficient.
///
/// `WittenBell` scales each region's residual by `n / (n + T)`, where `n` is the
/// region's total next-token count and `T` its distinct-type count, so
/// sparsely-supported regions fall back toward the parent and well-supported
/// ones contribute fully. The scaling is applied at COMPILE time to the stored
/// value, so the runtime, kernel and artifact ABI are untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmissionShrinkage {
    /// Apply the residual at full strength (shipped behavior).
    #[default]
    None,
    /// Scale by n / (n + T) per region.
    WittenBell,
    /// Scale by the region's measured CONTRAST against the global prior.
    ///
    /// The cover's regions are densely estimated (3.1M observations each) and
    /// genuinely predictive (own region emits the teacher 1.41x more often than
    /// an unrelated one), but their emission lists are largely the same
    /// globally-common tokens -- an unrelated region's top-E contains the
    /// teacher 42.7% of the time. Specialization is thin, and the scorer applies
    /// that thin margin as though it were a full correction.
    ///
    /// This weights each region's residual by how far its own top-E departs from
    /// the global prior's top-E: a region indistinguishable from the prior
    /// contributes nothing, a genuinely distinctive one contributes fully.
    /// Evidence count is not the binding constraint (WittenBell measured
    /// lambda = 0.9992 and changed nothing); contrast is.
    Contrast,
}

/// How the per-region emission list is chosen before truncation (#364).
///
/// The shipped rule ranks by log-ratio against the parent, which keeps the most
/// DISTINCTIVE tokens. Measured on the fixture artifact, that list overlaps a
/// top-E-by-probability selection by 0.78% and covers 1.57% of the region's
/// actual next-token mass — so 98.4% of what really happens next carries
/// residual 0 and is scored at the bare global prior.
///
/// `Probability` ranks by the region's own next-token counts instead, still
/// storing the log-ratio as the value, so `root_score + log-ratio =
/// log P_region` continues to hold for the tokens that actually occur.
///
/// Default is `Ratio`: it reproduces the shipped artifact byte-exactly. Changing
/// selection changes artifact bytes and kappa, so the alternative stays opt-in
/// until measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmissionSelection {
    /// Top-E by log-ratio against the parent (shipped behavior).
    #[default]
    Ratio,
    /// Top-E by the region's own next-token probability.
    Probability,
}
/// One region's emission list plus the statistics gathered while building it.
type RegionEmissionResult = (
    Vec<(u32, ScoreQ)>,
    QuantizationErrorStats,
    EmissionSelectionStats,
);

#[derive(Debug, Clone, Default)]
pub struct EmissionSelectionStats {
    pub regions: usize,
    /// Witten-Bell lambda = n / (n + T) per region; near 1 means the estimator
    /// barely shrinks and cannot test the sparsity hypothesis.
    pub mean_lambda_witten_bell: f64,
    /// Distribution of per-region contrast against the global prior. Spread
    /// says whether region construction has headroom: uniformly mid-range
    /// contrast means the cover is generic everywhere, while high variance
    /// means some regions are genuinely distinctive and granularity is the
    /// lever.
    pub mean_contrast: f64,
    pub min_contrast: f64,
    pub max_contrast: f64,
    pub mean_region_count: f64,
    pub mean_region_types: f64,
    pub overlap_with_top_count: f64,
    pub probability_mass_kept: f64,
}
/// Default root-prior candidate count.
pub const DEFAULT_ROOT_TOP_B: usize = 64;
/// Default EXCT candidate count.
pub const DEFAULT_EXCT_TOP_X: usize = 64;
/// Default per-context candidate bound for packed bigram/trigram rows.
pub const DEFAULT_CONTEXT_ENTRIES: usize = 64;
/// Default held-out position count whose witnesses are replayed in Gate C.
pub const DEFAULT_WITNESS_SAMPLE: usize = 64;

/// HEAD defaults reused from `convert_r4g1`/`cover` (RFC §4 starting
/// defaults; honest observed maxima replace the floors when larger).
const DEFAULT_MAX_FRONTIER_WIDTH: u16 = 32;
const MAX_CANDIDATES: u16 = 16;
const DEFAULT_MAX_EMISSION_ENTRIES: u32 = 64;

const SHORTLIST_SIZE: u16 = 8;
const MAX_PROGRAM_STEPS: u32 = 64;

/// blake3 input labeling this compiler as the compiler of record.
const COMPILER_VERSION_LABEL: &[u8] = b"uor-r4-core score v0";

/// Emission smoothing rule for the compiled next-token distributions
/// (issue #67 calibration). Compiler-side only — the deployed scorer
/// reads baked ScoreQ values and never re-derives probabilities, so the
/// rule is pinned into the artifact at compile time and recorded in the
/// score report's config.
///
/// Every rule shares one shape: `ln P(v | distribution)` for a count
/// map with `total` evidence over a compiled vocabulary of `vocab`
/// tokens, `seen_types` (T) of them observed. Unseen tokens take the
/// rule's floor; the floor mass is spread uniformly over the unseen
/// types (clamped to at least one type so a fully-observed vocabulary
/// still has a finite floor).
#[derive(Debug, Clone, Copy)]
pub enum Smoothing {
    /// Add-one (Laplace): `P(v) = (count + 1) / (total + V)` — the
    /// Phase-4 default; byte-exact with the pre-#67 compiler.
    AddOne,
    /// Witten-Bell at the single-distribution level: seen types get
    /// `count / (total + T)`; the reserved mass `T / (total + T)` is
    /// the floor, spread uniformly over the `max(V − T, 1)` unseen
    /// types. This is the per-distribution specialization of the
    /// store's backoff chain ([`witten_bell_probability`]) at depth 0 —
    /// the same λ = total / (total + T) shrinkage the Gate C baseline's
    /// bits/token uses, with the chain's eventual uniform floor applied
    /// to the unseen types only — so the baseline's metric family is
    /// comparable.
    WittenBell,
    /// Absolute discounting with discount δ: types with `count > δ` get
    /// `(count − δ) / total`; the discounted mass `δ·T / total` is the
    /// floor, spread uniformly over the `max(V − T, 1)` unseen types
    /// (types with `count ≤ δ`, including unseen ones, take the floor).
    AbsoluteDiscount(f64),
}

impl Smoothing {
    /// ln of the smoothed probability of one token (compiler-side f64;
    /// module docs for the platform pinning). `count` is the token's
    /// evidence count (0 = unseen), `total` the distribution's evidence
    /// total, `vocab` the compiled vocabulary size, `seen_types` the
    /// number of distinct observed types (T).
    pub fn ln_prob(&self, count: u64, total: u64, vocab: u32, seen_types: usize) -> f32 {
        match *self {
            Smoothing::AddOne => smoothed_ln(count, total, vocab),
            Smoothing::WittenBell => {
                if total == 0 {
                    // No evidence: the chain's uniform floor is all that
                    // remains (T = 0 whenever total = 0).
                    return (1.0 / f64::from(vocab)).ln() as f32;
                }
                let total = total as f64;
                let types = seen_types as f64;
                if count > 0 {
                    (count as f64 / (total + types)).ln() as f32
                } else {
                    let floor_mass = types / (total + types);
                    let unseen = (f64::from(vocab) - types).max(1.0);
                    (floor_mass / unseen).ln() as f32
                }
            }
            Smoothing::AbsoluteDiscount(delta) => {
                if total == 0 {
                    return (1.0 / f64::from(vocab)).ln() as f32;
                }
                let total = total as f64;
                let types = seen_types as f64;
                if count as f64 > delta {
                    ((count as f64 - delta) / total).ln() as f32
                } else {
                    let floor_mass = delta * types / total;
                    let unseen = (f64::from(vocab) - types).max(1.0);
                    (floor_mass / unseen).ln() as f32
                }
            }
        }
    }

    /// The canonical CLI/report spelling (`add-one`, `witten-bell`,
    /// `abs-disc:δ`).
    pub fn label(&self) -> String {
        match *self {
            Smoothing::AddOne => "add-one".to_owned(),
            Smoothing::WittenBell => "witten-bell".to_owned(),
            Smoothing::AbsoluteDiscount(delta) => format!("abs-disc:{delta}"),
        }
    }

    /// Parse a `--smoothing` flag value: `add-one` | `witten-bell` |
    /// `abs-disc:δ` with δ finite and in (0, 1].
    pub fn parse(value: &str) -> Result<Smoothing, String> {
        match value {
            "add-one" => Ok(Smoothing::AddOne),
            "witten-bell" => Ok(Smoothing::WittenBell),
            _ => {
                let Some(delta) = value.strip_prefix("abs-disc:") else {
                    return Err(format!(
                        "invalid --smoothing value: {value} \
                         (expected add-one | witten-bell | abs-disc:δ)"
                    ));
                };
                let delta: f64 = delta
                    .parse()
                    .map_err(|_| format!("invalid --smoothing abs-disc delta: {delta}"))?;
                if !delta.is_finite() || delta <= 0.0 || delta > 1.0 {
                    return Err(format!(
                        "--smoothing abs-disc delta must be finite and in (0, 1]: {delta}"
                    ));
                }
                Ok(Smoothing::AbsoluteDiscount(delta))
            }
        }
    }
}

/// Bit-exact equality (the discount is validated finite at parse time,
/// so NaN can never reach a stored config).
impl PartialEq for Smoothing {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Smoothing::AddOne, Smoothing::AddOne)
            | (Smoothing::WittenBell, Smoothing::WittenBell) => true,
            (Smoothing::AbsoluteDiscount(a), Smoothing::AbsoluteDiscount(b)) => {
                a.to_bits() == b.to_bits()
            }
            _ => false,
        }
    }
}

impl Eq for Smoothing {}

/// Configuration of one scored-graph compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreConfig {
    /// Per-source out-degree cap for forward transition edges.
    pub transition_out_degree: usize,
    /// Per-region emission list bound (top-E by residual score).
    pub emission_entries: usize,
    /// Number of root-prior tokens admitted to the candidate set.
    pub root_top_b: usize,
    /// Number of EXCT probe tokens admitted to the candidate set.
    pub exct_top_x: usize,
    /// Held-out positions whose witnesses are replayed in Gate C.
    pub witness_sample: usize,
    /// Emission smoothing rule (issue #67). The add-one default
    /// preserves the pre-#67 compiler byte-exactly.
    pub smoothing: Smoothing,
    /// Candidate scoring variant (issue #80).
    pub scoring_variant: ScoringVariant,
    /// Emission list selection rule (issue #364).
    pub emission_selection: EmissionSelection,
    /// Per-region residual shrinkage (issue #364).
    pub emission_shrinkage: EmissionShrinkage,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            transition_out_degree: DEFAULT_TRANSITION_OUT_DEGREE,
            emission_entries: DEFAULT_EMISSION_ENTRIES,
            root_top_b: DEFAULT_ROOT_TOP_B,
            exct_top_x: DEFAULT_EXCT_TOP_X,
            witness_sample: DEFAULT_WITNESS_SAMPLE,
            smoothing: Smoothing::AddOne,
            scoring_variant: ScoringVariant::ChainTelescoped,
            emission_selection: EmissionSelection::default(),
            emission_shrinkage: EmissionShrinkage::default(),
        }
    }
}

/// One compiled forward transition edge (E_f): artifact node ids, the
/// raw evidence count (report side), and the quantized log weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionEdge {
    pub src: u32,
    pub dst: u32,
    pub count: u32,
    pub score: ScoreQ,
}

/// Compile-time quantization error summary in nano-nats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuantizationErrorStats {
    pub sample_count: u64,
    pub sum_abs_error_nano: u128,
    pub max_abs_error_nano: u64,
}

impl QuantizationErrorStats {
    fn record_ln_quantization(&mut self, source_ln: f32, quantized: ScoreQ) {
        let restored = quantized.to_logprob();
        let abs_error = f64::from((restored - source_ln).abs());
        let nanos = (abs_error * 1_000_000_000.0).round() as u64;
        self.sample_count = self.sample_count.saturating_add(1);
        self.sum_abs_error_nano = self.sum_abs_error_nano.saturating_add(u128::from(nanos));
        self.max_abs_error_nano = self.max_abs_error_nano.max(nanos);
    }

    fn mean_abs_error_nano(self) -> u64 {
        if self.sample_count == 0 {
            return 0;
        }
        (self.sum_abs_error_nano / u128::from(self.sample_count)) as u64
    }
}

/// Compile forward transitions E_f from consecutive train positions
/// (module docs). `regions` are the scoring region parameters; the
/// active clouds come from [`binary_memberships`] — one code path with
/// the scorer. Edges come out sorted by `(src, dst)`.
pub fn compile_transitions_with_quantization(
    corpus: &Corpus,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
    out_degree: usize,
) -> (Vec<TransitionEdge>, QuantizationErrorStats) {
    let pop = runtime::derive_popcount_table();
    // Canonical observation order (content-addressed positions, §4.1):
    // the caller's slice order never reaches the counts, so a shuffled
    // observation/shard order compiles to identical edges.
    let mut ordered: Vec<&Observation> = train.iter().collect();
    ordered.sort_by_key(|o| o.position);
    // Memberships are independent per observation. Compute them in parallel,
    // then consume the resulting vector in corpus order so adjacent-position
    // semantics and BTreeMap reduction order are unchanged.
    let memberships: Vec<Vec<Vec<u32>>> = ordered
        .par_iter()
        .map(|observation| {
            let mut k = runtime::OpKernel::default();
            (1..=max_depth)
                .map(|depth| {
                    binary_memberships(&mut k, &pop, regions, depth, &observation.sig)
                        .into_iter()
                        .map(|(region, _)| region)
                        .collect()
                })
                .collect()
        })
        .collect();
    let mut previous: Option<(u32, &Vec<Vec<u32>>)> = None;
    let mut counts: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    let mut src_totals: BTreeMap<u32, u64> = BTreeMap::new();
    for (observation, current_memberships) in ordered.into_iter().zip(memberships.iter()) {
        let position = observation.position;
        if let Some((prev_position, prev_memberships)) = previous.take() {
            let adjacent = position == prev_position + 1
                && corpus.story[position as usize] == corpus.story[prev_position as usize];
            if adjacent {
                for depth in 0..max_depth {
                    for &src in &prev_memberships[depth] {
                        for &dst in &current_memberships[depth] {
                            let edge = (src + 1, dst + 1); // region -> node id
                            *counts.entry(edge).or_insert(0) += 1;
                            *src_totals.entry(edge.0).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        previous = Some((position, current_memberships));
    }

    // Per source: canonical order (weight desc, then dst asc), bounded.
    let mut by_src: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
    for (&(src, dst), &count) in &counts {
        by_src.entry(src).or_default().push((dst, count));
    }
    let mut quantization = QuantizationErrorStats::default();
    let mut edges = Vec::new();
    for (src, mut dsts) in by_src {
        dsts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let total = src_totals[&src];
        for &(dst, count) in dsts.iter().take(out_degree) {
            // Compiler-side f64 ln quantization (macOS-pinned; module docs).
            let ln_prob = (count as f64 / total as f64).ln() as f32;
            let score = ScoreQ::from_logprob(ln_prob);
            quantization.record_ln_quantization(ln_prob, score);
            edges.push(TransitionEdge {
                src,
                dst,
                count: count.min(u32::MAX as u64) as u32,
                score,
            });
        }
    }
    edges.sort_by_key(|e| (e.src, e.dst));
    (edges, quantization)
}

/// Compile forward transitions E_f from consecutive train positions
/// (module docs). `regions` are the scoring region parameters; the
/// active clouds come from [`binary_memberships`] — one code path with
/// the scorer. Edges come out sorted by `(src, dst)`.
pub fn compile_transitions(
    corpus: &Corpus,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
    out_degree: usize,
) -> Vec<TransitionEdge> {
    compile_transitions_with_quantization(corpus, regions, train, max_depth, out_degree).0
}

/// The compiled residual tables: the ScoreQ root prior (full block) and
/// each region's bounded ΔE list (ascending token storage order).
#[derive(Debug, Clone)]
pub struct EmissionTables {
    /// B(v) for every token observed at level 0 (ascending token).
    pub root_prior: BTreeMap<u32, ScoreQ>,
    /// The smoothing floor for tokens outside the root prior (add-one:
    /// `ScoreQ(ln(1/(total + V)))`).
    pub root_floor: ScoreQ,
    /// Level-0 evidence total (the smoothing denominator source).
    pub root_total: u64,
    /// Per region id: `(token, ΔE)` ascending token, bounded to top-E.
    pub region_lists: Vec<Vec<(u32, ScoreQ)>>,
    /// The rule these tables were compiled with; the EXCT residuals are
    /// quantized under the same rule so the whole artifact speaks one
    /// smoothing language.
    pub smoothing: Smoothing,
    /// Root-prior quantization errors (includes the root floor).
    pub root_prior_quantization: QuantizationErrorStats,
    /// Emission-residual quantization errors.
    pub emission_quantization: QuantizationErrorStats,
}

/// One canonical context-conditioned lexical row. The root prior in
/// [`EmissionTables`] is the unigram row; `context_len` 1 and 2 are bigram
/// and trigram rows respectively.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRow {
    pub context_len: u8,
    pub key0: u32,
    pub key1: u32,
    /// Absolute smoothed log scores, stored in ascending token order.
    pub entries: Vec<(u32, ScoreQ)>,
}

/// Compile explicit bigram and trigram rows from the teacher-forced corpus.
/// Story boundaries never form a trigram key. Rows are retained when the
/// context was observed; the runtime's exact row-presence rule then defines
/// the deterministic most-specific backoff.
pub fn compile_context_rows(
    corpus: &Corpus,
    train: &[Observation],
    vocab: u32,
    smoothing: Smoothing,
) -> Vec<ContextRow> {
    let mut counts: BTreeMap<(u8, u32, u32), BTreeMap<u32, u64>> = BTreeMap::new();
    for observation in train {
        let i = observation.position as usize;
        if i >= corpus.n || corpus.next[i] != observation.next {
            continue;
        }
        let bigram = (1, corpus.input[i], 0);
        *counts
            .entry(bigram)
            .or_default()
            .entry(corpus.next[i])
            .or_insert(0) += 1;
        if i > 0 && corpus.story[i - 1] == corpus.story[i] {
            let trigram = (2, corpus.input[i - 1], corpus.input[i]);
            *counts
                .entry(trigram)
                .or_default()
                .entry(corpus.next[i])
                .or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .map(|((context_len, key0, key1), distribution)| {
            let total: u64 = distribution.values().sum();
            let types = distribution.len();
            let mut ranked: Vec<(u32, ScoreQ)> = distribution
                .iter()
                .map(|(&token, &count)| {
                    let ln = smoothing.ln_prob(count, total, vocab, types);
                    (token, ScoreQ::from_logprob(ln))
                })
                .collect();
            ranked.sort_by(|a, b| b.1.raw().cmp(&a.1.raw()).then_with(|| a.0.cmp(&b.0)));
            ranked.truncate(DEFAULT_CONTEXT_ENTRIES);
            ranked.sort_by_key(|&(token, _)| token);
            ContextRow {
                context_len,
                key0,
                key1,
                entries: ranked,
            }
        })
        .collect()
}

/// Compile the optional FMM section from the same regions and emission tables
/// that feed the normal scored-artifact path. All floating-point work remains
/// in the certifier; the returned bytes contain only the folded integer table
/// consumed by the runtime kernel.
pub fn compile_fmm_section(
    regions: &[RegionParams],
    emissions: &EmissionTables,
    vocab: u32,
) -> Result<Vec<u8>, String> {
    let emission_maps: Vec<BTreeMap<u32, ScoreQ>> = emissions
        .region_lists
        .iter()
        .map(|entries| entries.iter().copied().collect())
        .collect();
    let scorer = crate::fmm::FmmCandidateScorer::from_graph_parts(
        regions,
        &emission_maps,
        &emissions.root_prior,
        emissions.root_floor,
        vocab,
        crate::fmm::FmmConfig::default(),
    )?;
    scorer.fixed_point()?.to_packed_section()
}

/// ln of an add-one-smoothed probability (compiler-side f64; module
/// docs for the platform pinning). This is the [`Smoothing::AddOne`]
/// arm; the other rules live in [`Smoothing::ln_prob`].
fn smoothed_ln(count: u64, total: u64, vocab: u32) -> f32 {
    ((count as f64 + 1.0) / (total as f64 + f64::from(vocab))).ln() as f32
}

/// Witten-Bell's evidence weight for a region-conditioned estimate.
///
/// A region with many observations per distinct type gets a weight near one;
/// a sparse region is pulled toward its parent. The zero-evidence case is
/// defined as zero so the compiler never emits a non-finite weight.
fn witten_bell_lambda(total: u64, types: usize) -> f64 {
    let denominator = total as f64 + types as f64;
    if denominator > 0.0 {
        total as f64 / denominator
    } else {
        0.0
    }
}

/// Compile the root prior and per-region emission residuals (module
/// docs). The evidence model matches the store's exactly (top-3
/// teacher-weighted counts over train positions); the root distribution
/// is the level-0 store distribution. Probabilities are smoothed under
/// `config.smoothing` (issue #67; add-one is the byte-exact default).
pub fn compile_emissions(
    corpus: &Corpus,
    store: &Store,
    regions: &[RegionParams],
    train: &[Observation],
    max_depth: usize,
    vocab: u32,
    config: &ScoreConfig,
) -> EmissionTables {
    let pop = runtime::derive_popcount_table();
    let smoothing = config.smoothing;
    // Weighted evidence per region: the covered binary top-1 membership
    // at each depth (within the calibrated radius — the backoff floor is
    // a routing behavior, never region content; see `binary_top1_covered`).
    let mut evidence: Vec<BTreeMap<u32, u64>> = vec![BTreeMap::new(); regions.len()];
    if !train.is_empty() {
        let chunk_size = train.len().div_ceil(rayon::current_num_threads().max(1));
        let partials: Vec<Vec<BTreeMap<u32, u64>>> = train
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut k = runtime::OpKernel::default();
                let mut local = vec![BTreeMap::new(); regions.len()];
                for observation in chunk {
                    let i = observation.position as usize;
                    for depth in 1..=max_depth {
                        let Some((top1, _)) =
                            binary_top1_covered(&mut k, &pop, regions, depth, &observation.sig)
                        else {
                            continue;
                        };
                        let dist = &mut local[top1 as usize];
                        for k_idx in 0..corpus.top_tokens[i].len() {
                            let token = corpus.top_tokens[i][k_idx];
                            let weight = corpus.top_weights[i][k_idx];
                            if weight > 0 {
                                *dist.entry(token).or_insert(0) += u64::from(weight);
                            }
                        }
                    }
                }
                local
            })
            .collect();
        // Merge chunks in input order, preserving the serial BTreeMap
        // reduction order and therefore artifact determinism.
        for partial in partials {
            for (region, local_dist) in partial.into_iter().enumerate() {
                for (token, weight) in local_dist {
                    *evidence[region].entry(token).or_insert(0) += weight;
                }
            }
        }
    }

    // Root prior B(v) from the level-0 store distribution.
    let root_dist: BTreeMap<u32, u64> = store
        .first()
        .and_then(|level| level.get(&[][..]))
        .map(|dist| dist.iter().map(|(&t, &c)| (t, u64::from(c))).collect())
        .unwrap_or_default();
    let root_total: u64 = root_dist.values().sum();
    let root_types = root_dist.len();
    let mut root_prior_quantization = QuantizationErrorStats::default();
    let root_floor_ln = smoothing.ln_prob(0, root_total, vocab, root_types);
    let root_floor = ScoreQ::from_logprob(root_floor_ln);
    root_prior_quantization.record_ln_quantization(root_floor_ln, root_floor);
    let root_prior: BTreeMap<u32, ScoreQ> = root_dist
        .iter()
        .map(|(&t, &c)| {
            let ln = smoothing.ln_prob(c, root_total, vocab, root_types);
            let score = ScoreQ::from_logprob(ln);
            root_prior_quantization.record_ln_quantization(ln, score);
            (t, score)
        })
        .collect();

    // The global prior's own top-E, used as the contrast reference: a region
    // whose most-likely tokens are the prior's most-likely tokens is not
    // conditioning on anything.
    let root_top_set: std::collections::BTreeSet<u32> = {
        let mut top: Vec<(u32, u64)> = root_dist.iter().map(|(&t, &c)| (t, c)).collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top.truncate(config.emission_entries);
        top.into_iter().map(|(t, _)| t).collect()
    };

    let region_results: Vec<RegionEmissionResult> = regions
        .par_iter()
        .enumerate()
        .map(|(region_id, region)| {
            let mut emission_quantization = QuantizationErrorStats::default();
            let dist = &evidence[region_id];
            let total: u64 = dist.values().sum();
            let types = dist.len();
            // Parent distribution: the parent region's evidence, or the
            // level-0 root distribution at depth 1.
            let (parent_dist, parent_total): (&BTreeMap<u32, u64>, u64) = match region.parent {
                Some(parent) => {
                    let parent_dist = &evidence[parent as usize];
                    (parent_dist, parent_dist.values().sum())
                }
                None => (&root_dist, root_total),
            };
            let parent_types = parent_dist.len();
            // Contrast: how far this region's own most-likely tokens depart from
            // the global prior's. Overlap 1.0 means the region looks exactly
            // like the prior and its "correction" is noise; overlap 0 means it
            // is fully distinctive. Computed on counts, independent of which
            // selection rule is in force.
            let contrast = {
                let mut region_top: Vec<(u32, u64)> = dist.iter().map(|(&t, &c)| (t, c)).collect();
                region_top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                region_top.truncate(config.emission_entries);
                let shared = region_top
                    .iter()
                    .filter(|(token, _)| root_top_set.contains(token))
                    .count();
                let denom = region_top.len().max(1) as f64;
                1.0 - (shared as f64 / denom)
            };
            let mut residuals: Vec<(u32, ScoreQ)> = dist
                .iter()
                .map(|(&token, &count)| {
                    let lp_n = smoothing.ln_prob(count, total, vocab, types);
                    let lp_p = smoothing.ln_prob(
                        parent_dist.get(&token).copied().unwrap_or(0),
                        parent_total,
                        vocab,
                        parent_types,
                    );
                    let ln = lp_n - lp_p;
                    // Shrink sparse regions toward the parent. n / (n + T)
                    // approaches 1 when a region has many observations per
                    // distinct type and approaches 0 when its distribution is
                    // mostly singletons -- exactly the regime where the
                    // conditional estimate is noise.
                    let ln = match config.emission_shrinkage {
                        EmissionShrinkage::None => ln,
                        EmissionShrinkage::WittenBell => {
                            let lambda = witten_bell_lambda(total, types);
                            (f64::from(ln) * lambda) as f32
                        }
                        EmissionShrinkage::Contrast => (f64::from(ln) * contrast) as f32,
                    };
                    let score = ScoreQ::from_logprob(ln);
                    emission_quantization.record_ln_quantization(ln, score);
                    (token, score)
                })
                .collect();
            // Top-E by residual score (score desc, token asc), stored
            // ascending token.
            match config.emission_selection {
                EmissionSelection::Ratio => {
                    residuals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                }
                EmissionSelection::Probability => {
                    // Rank by the region's own next-token count (probability),
                    // ties to the lower token id for determinism. The stored
                    // value stays the log-ratio, so the scoring identity is
                    // unchanged -- only WHICH tokens get a correction moves.
                    residuals.sort_by(|a, b| {
                        let ca = dist.get(&a.0).copied().unwrap_or(0);
                        let cb = dist.get(&b.0).copied().unwrap_or(0);
                        cb.cmp(&ca).then_with(|| a.0.cmp(&b.0))
                    });
                }
            }
            residuals.truncate(config.emission_entries);
            residuals.sort_by_key(|&(token, _)| token);

            // Selection is by log-RATIO against the parent, which keeps the
            // most DISTINCTIVE tokens rather than the most LIKELY ones. A rare
            // token heavily over-represented in this region outranks the
            // region's actual most-probable next token, whose ratio is near
            // zero because it is common everywhere. Measure the overlap with a
            // top-E-by-count selection to see how far apart those two are.
            let kept: std::collections::BTreeSet<u32> =
                residuals.iter().map(|&(token, _)| token).collect();
            let mut by_count: Vec<(u32, u64)> = dist.iter().map(|(&t, &c)| (t, c)).collect();
            by_count.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            by_count.truncate(config.emission_entries);
            let overlap = by_count
                .iter()
                .filter(|(token, _)| kept.contains(token))
                .count();
            let mass_kept: u64 = dist
                .iter()
                .filter(|(t, _)| kept.contains(t))
                .map(|(_, &c)| c)
                .sum();
            let lambda_wb = witten_bell_lambda(total, types);
            let selection = EmissionSelectionStats {
                regions: 1,
                mean_lambda_witten_bell: lambda_wb,
                mean_contrast: contrast,
                min_contrast: contrast,
                max_contrast: contrast,
                mean_region_count: total as f64,
                mean_region_types: types as f64,
                overlap_with_top_count: overlap as f64 / by_count.len().max(1) as f64,
                probability_mass_kept: if total == 0 {
                    0.0
                } else {
                    mass_kept as f64 / total as f64
                },
            };
            (residuals, emission_quantization, selection)
        })
        .collect();
    let mut region_lists = Vec::with_capacity(regions.len());
    let mut emission_quantization = QuantizationErrorStats::default();
    let mut selection_stats = EmissionSelectionStats::default();
    for (residuals, stats, selection) in region_results {
        selection_stats.regions += selection.regions;
        selection_stats.overlap_with_top_count += selection.overlap_with_top_count;
        selection_stats.probability_mass_kept += selection.probability_mass_kept;
        selection_stats.mean_lambda_witten_bell += selection.mean_lambda_witten_bell;
        selection_stats.mean_contrast += selection.mean_contrast;
        if selection_stats.regions == 1 {
            selection_stats.min_contrast = selection.min_contrast;
            selection_stats.max_contrast = selection.max_contrast;
        } else {
            selection_stats.min_contrast = selection_stats.min_contrast.min(selection.min_contrast);
            selection_stats.max_contrast = selection_stats.max_contrast.max(selection.max_contrast);
        }
        selection_stats.mean_region_count += selection.mean_region_count;
        selection_stats.mean_region_types += selection.mean_region_types;
        emission_quantization.sample_count += stats.sample_count;
        emission_quantization.sum_abs_error_nano = emission_quantization
            .sum_abs_error_nano
            .saturating_add(stats.sum_abs_error_nano);
        emission_quantization.max_abs_error_nano = emission_quantization
            .max_abs_error_nano
            .max(stats.max_abs_error_nano);
        region_lists.push(residuals);
    }
    if selection_stats.regions > 0 {
        let n = selection_stats.regions as f64;
        eprintln!(
            "[emission-selection] regions={} mean_overlap_with_top_count={:.4} \
             mean_probability_mass_kept={:.4} mean_lambda_wb={:.4} \
             mean_n={:.0} mean_types={:.0} contrast mean={:.4} min={:.4} max={:.4} (E={})",
            selection_stats.regions,
            selection_stats.overlap_with_top_count / n,
            selection_stats.probability_mass_kept / n,
            selection_stats.mean_lambda_witten_bell / n,
            selection_stats.mean_region_count / n,
            selection_stats.mean_region_types / n,
            selection_stats.mean_contrast / n,
            selection_stats.min_contrast,
            selection_stats.max_contrast,
            config.emission_entries,
        );
    }

    EmissionTables {
        root_prior,
        root_floor,
        root_total,
        region_lists,
        smoothing,
        root_prior_quantization,
        emission_quantization,
    }
}

/// One canonical edge of the scored graph during emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireEdge {
    pub src: u32,
    pub kind: u8,
    pub dst: u32,
    pub score_q: ScoreQ,
}

/// Theorem 7 (full wiring, stronger than the format's v0 existence
/// approximation): the reverse index is a permutation of the canonical
/// edge IDs, sorted by `(dst, src, kind)`; per-dst runs are contiguous
/// and exactly match the PackedNode forward ranges.
pub fn verify_theorem_7_wired(
    edges: &[WireEdge],
    reverse: &[u32],
    forward_start: &[u32],
    forward_len: &[u16],
) -> Result<(), String> {
    if reverse.len() != edges.len() {
        return Err("Theorem 7 violation: reverse index does not cover all edges".to_owned());
    }
    let mut seen = vec![false; edges.len()];
    for &id in reverse {
        let Some(slot) = seen.get_mut(id as usize) else {
            return Err(
                "Theorem 7 violation: invalid canonical edge ID in reverse index".to_owned(),
            );
        };
        if *slot {
            return Err("Theorem 7 violation: duplicate edge ID in reverse index".to_owned());
        }
        *slot = true;
    }
    if seen.iter().any(|s| !s) {
        return Err("Theorem 7 violation: reverse index is not a permutation".to_owned());
    }
    for pair in reverse.windows(2) {
        let a = edges[pair[0] as usize];
        let b = edges[pair[1] as usize];
        if (a.dst, a.src, a.kind) > (b.dst, b.src, b.kind) {
            return Err(
                "Theorem 7 violation: reverse index not sorted by (dst, src, kind)".to_owned(),
            );
        }
    }
    for (node, (&start, &len)) in forward_start.iter().zip(forward_len.iter()).enumerate() {
        let end = start as usize + len as usize;
        if end > reverse.len() {
            return Err("Theorem 7 violation: forward range out of bounds".to_owned());
        }
        for &id in &reverse[start as usize..end] {
            if edges[id as usize].dst != node as u32 {
                return Err("Theorem 7 violation: reverse range target mismatched node".to_owned());
            }
        }
    }
    Ok(())
}

/// What an [`emit_scored_r4g1`] call produced, for the report and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredGraphInfo {
    pub node_count: u32,
    pub edge_count: u32,
    pub refinement_edges: u32,
    pub neighbor_edges: u32,
    pub forward_edges: u32,
    pub depth_count: u8,
    pub max_frontier_width: u16,
    pub max_emission_entries: u32,
    pub root_prior_entries: u32,
    pub emission_list_entries: u32,
    pub exct_bytes: u32,
    pub context_row_count: u32,
    pub context_entry_count: u32,
    pub context_bytes: u32,
    pub fmm_bytes: u32,
    pub fmm_rank: u16,
    pub fmm_candidate_count: u32,
    pub artifact_bytes: usize,
    pub transition_quantization: QuantizationErrorStats,
    pub root_prior_quantization: QuantizationErrorStats,
    pub emission_quantization: QuantizationErrorStats,
    pub exact_context_quantization: QuantizationErrorStats,
}

/// Data bundle for [`emit_scored_r4g1`] (keeps the argument list
/// focused): the graph content that becomes the NODE/EDGE/EMIT/EXCT
/// sections.
pub struct ScoredGraphSections<'a> {
    /// Region parameters, ascending region id.
    pub regions: &'a [RegionParams],
    /// Structural (E_r/E_o) edges with their stored scores.
    pub structural: &'a [StructuralEdge],
    /// Compiled forward transition edges (E_f).
    pub transitions: &'a [TransitionEdge],
    /// Transition-edge quantization error summary.
    pub transition_quantization: QuantizationErrorStats,
    /// Root prior + per-region residual lists.
    pub emissions: &'a EmissionTables,
    /// Explicit bigram/trigram context rows; the root unigram remains EMIT.
    pub context_rows: &'a [ContextRow],
    /// TLS1 container bytes used as compiler input for residualized EXCT.
    pub exct_tls1: &'a [u8],
    /// Number of exact-context residual entries retained per prefix.
    pub exct_top_x: usize,
    /// Optional compiler-folded FMM translation table.
    pub fmm_section: Option<&'a [u8]>,
}

fn encode_context_rows(rows: &[ContextRow]) -> Result<Vec<u8>, String> {
    let row_count =
        u32::try_from(rows.len()).map_err(|_| "NGRAM row count exceeds u32".to_owned())?;
    let header_len = uor_r4_graph_format::NGRAM_HEADER_LEN;
    let row_len = uor_r4_graph_format::NGRAM_ROW_LEN;
    let entries_start = header_len
        .checked_add(
            row_len
                .checked_mul(rows.len())
                .ok_or("NGRAM row bytes overflow")?,
        )
        .ok_or("NGRAM header bytes overflow")?;
    let mut bytes = Vec::with_capacity(entries_start);
    bytes.extend_from_slice(&uor_r4_graph_format::NGRAM_MAGIC);
    bytes.extend_from_slice(&uor_r4_graph_format::NGRAM_VERSION.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.extend_from_slice(&row_count.to_le_bytes());
    let max_entries = rows.iter().map(|row| row.entries.len()).max().unwrap_or(0);
    bytes.extend_from_slice(
        &u16::try_from(max_entries)
            .map_err(|_| "NGRAM row entry count exceeds u16".to_owned())?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.resize(entries_start, 0);
    let mut entry_offset = entries_start;
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
        if !(1..=2).contains(&row.context_len) || (row.context_len == 1 && row.key1 != 0) {
            return Err("NGRAM row has an invalid context key".to_owned());
        }
        let key = (row.context_len, row.key0, row.key1);
        if previous.is_some_and(|last| last >= key) {
            return Err("NGRAM rows are not canonically sorted".to_owned());
        }
        previous = Some(key);
        let entry_count = u16::try_from(row.entries.len())
            .map_err(|_| "NGRAM row entry count exceeds u16".to_owned())?;
        let header = header_len + index * row_len;
        bytes[header] = row.context_len;
        bytes[header + 2..header + 4].copy_from_slice(&entry_count.to_le_bytes());
        bytes[header + 4..header + 8].copy_from_slice(&row.key0.to_le_bytes());
        bytes[header + 8..header + 12].copy_from_slice(&row.key1.to_le_bytes());
        let entry_offset_u32 =
            u32::try_from(entry_offset).map_err(|_| "NGRAM entry offset exceeds u32".to_owned())?;
        bytes[header + 12..header + 16].copy_from_slice(&entry_offset_u32.to_le_bytes());
        let mut previous_token = None;
        for &(token, score) in &row.entries {
            if previous_token.is_some_and(|last| last >= token) {
                return Err("NGRAM entries are not canonically sorted".to_owned());
            }
            previous_token = Some(token);
            bytes.extend_from_slice(&token.to_le_bytes());
            bytes.extend_from_slice(&score.raw().to_le_bytes());
        }
        entry_offset = bytes.len();
    }
    Ok(bytes)
}

/// Encode the exact-context store as compile-time ScoreQ residuals. The
/// runtime only reads the resulting integer values; it never evaluates ln or
/// performs probe-time quantization. `smoothing` is the rule the root
/// prior was compiled with, so the residuals cancel it consistently.
fn emit_residual_exct(
    store: &Store,
    root_prior: &BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
    vocab: u32,
    top_x: usize,
    smoothing: Smoothing,
) -> Result<(Vec<u8>, QuantizationErrorStats), String> {
    let mut bytes = Vec::new();
    let mut quantization = QuantizationErrorStats::default();
    bytes.extend_from_slice(&RESIDUAL_EXCT_MAGIC);
    bytes.push(u8::try_from(store.len()).map_err(|_| "EXCT level count exceeds u8".to_owned())?);
    bytes.extend_from_slice(&[0u8; 3]);
    for (level, contexts) in store.iter().enumerate() {
        let key_count = u32::try_from(contexts.len())
            .map_err(|_| format!("EXCT level {level} has too many contexts"))?;
        bytes.extend_from_slice(&key_count.to_le_bytes());
        for (key, distribution) in contexts {
            let key_len = u8::try_from(key.len())
                .map_err(|_| format!("EXCT key at level {level} is too long"))?;
            bytes.push(key_len);
            bytes.extend_from_slice(key);
            let total: u64 = distribution.values().map(|&count| u64::from(count)).sum();
            let total = total.min(u64::from(u32::MAX)) as u32;
            let mut ranked: Vec<(u32, u32)> = distribution
                .iter()
                .map(|(&token, &count)| (token, count))
                .collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            ranked.truncate(top_x);
            let entry_count = u32::try_from(ranked.len())
                .map_err(|_| "EXCT residual entry count exceeds u32".to_owned())?;
            bytes.extend_from_slice(&total.to_le_bytes());
            bytes.extend_from_slice(&entry_count.to_le_bytes());
            for (token, count) in ranked {
                let ln = smoothing.ln_prob(
                    u64::from(count),
                    u64::from(total),
                    vocab,
                    distribution.len(),
                );
                let exact = ScoreQ::from_logprob(ln);
                quantization.record_ln_quantization(ln, exact);
                let root = root_prior.get(&token).copied().unwrap_or(root_floor);
                let residual = exact.saturating_sub(root);
                bytes.extend_from_slice(&token.to_le_bytes());
                bytes.extend_from_slice(&residual.raw().to_le_bytes());
            }
        }
    }
    Ok((bytes, quantization))
}

/// Emit the scored graph as an R4G1 container: the cover's HEAD/NODE/
/// ROUT conventions with E_f merged into EDGE (kind tags distinguish
/// E_r/E_o/E_f), the EMIT residual tables with per-node ranges wired,
/// and the residualized RX1 EXCT table. Fails closed: Theorem 7 is verified
/// before serialization, and the bytes are re-validated with
/// `GraphView::parse` + `verify_cids` before they are returned.
pub fn emit_scored_r4g1(
    artifact_container: &[u8],
    corpus_cid_material: (&[u8], &[u8]),
    vocab_size: u32,
    sections: &ScoredGraphSections,
) -> Result<(Vec<u8>, ScoredGraphInfo), String> {
    let ScoredGraphSections {
        regions,
        structural,
        transitions,
        transition_quantization,
        emissions,
        context_rows,
        exct_tls1,
        exct_top_x,
        fmm_section,
    } = *sections;
    if regions.len() != emissions.region_lists.len() {
        return Err("emission lists do not match the region count".to_owned());
    }
    let node_count = 1 + regions.len() as u32;
    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);
    let depth_count = (max_depth + 1) as u8;

    // Canonical edge array: structural (E_r/E_o) + forward (E_f),
    // sorted by (src, kind, dst) — refinement children contiguous.
    let mut edges: Vec<WireEdge> = structural
        .iter()
        .map(|e| WireEdge {
            src: e.src,
            kind: e.kind,
            dst: e.dst,
            score_q: e.score_q,
        })
        .collect();
    for t in transitions {
        edges.push(WireEdge {
            src: t.src,
            kind: EDGE_KIND_FORWARD,
            dst: t.dst,
            score_q: t.score,
        });
    }
    edges.sort_by_key(|e| (e.src, e.kind, e.dst));
    edges.dedup_by_key(|e| (e.src, e.kind, e.dst));
    let edge_count = edges.len() as u32;
    let refinement_edges = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_REFINEMENT)
        .count() as u32;
    let neighbor_edges = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_NEIGHBOR)
        .count() as u32;
    let forward_edges = edge_count - refinement_edges - neighbor_edges;

    // Per-node refinement child ranges + reverse index and per-dst
    // forward ranges (cover/convert conventions).
    let node_total = node_count as usize;
    let mut child_start = vec![0u32; node_total];
    let mut child_len = vec![0u16; node_total];
    for (i, edge) in edges.iter().enumerate() {
        if edge.kind != EDGE_KIND_REFINEMENT {
            continue;
        }
        if child_len[edge.src as usize] == 0 {
            child_start[edge.src as usize] = i as u32;
        }
        child_len[edge.src as usize] += 1;
    }
    let max_child_len = child_len[1..].iter().copied().max().unwrap_or(0);
    let max_frontier_width = DEFAULT_MAX_FRONTIER_WIDTH.max(max_child_len);
    let mut reverse: Vec<u32> = (0..edge_count).collect();
    reverse.sort_by_key(|&id| {
        let e = edges[id as usize];
        (e.dst, e.src, e.kind)
    });
    let mut forward_start = vec![0u32; node_total];
    let mut forward_len = vec![0u16; node_total];
    for (i, &id) in reverse.iter().enumerate() {
        let dst = edges[id as usize].dst as usize;
        if forward_len[dst] == 0 {
            forward_start[dst] = i as u32;
        }
        forward_len[dst] += 1;
    }
    verify_theorem_7_wired(&edges, &reverse, &forward_start, &forward_len)?;

    // EMIT: descriptor + root prior block + per-region lists; wire the
    // per-node ranges as we lay the lists down.
    let mut emit = vec![2u8, 0, 0, 0]; // {width: i32, shift: 0, zero_point: 0}
    let root_entry_count = u32::try_from(emissions.root_prior.len())
        .map_err(|_| "root prior exceeds u32 entries".to_owned())?;
    emit.extend_from_slice(&root_entry_count.to_le_bytes());
    let root_total = emissions.root_total.min(u32::MAX as u64) as u32;
    emit.extend_from_slice(&root_total.to_le_bytes());
    emit.extend_from_slice(&emissions.root_floor.raw().to_le_bytes());
    emit.extend_from_slice(&0u32.to_le_bytes()); // reserved
    for (&token, &value) in &emissions.root_prior {
        let token =
            i32::try_from(token).map_err(|_| format!("root prior token {token} exceeds i32"))?;
        emit.extend_from_slice(&token.to_le_bytes());
        emit.extend_from_slice(&value.raw().to_le_bytes());
    }
    let mut emission_start = vec![0u32; node_total];
    let mut emission_len = vec![0u16; node_total];
    let mut emission_list_entries = 0u32;
    for (region_id, list) in emissions.region_lists.iter().enumerate() {
        let node = 1 + region_id;
        emission_start[node] = (emit.len() - 4) as u32; // remainder-relative
        emission_len[node] = u16::try_from(list.len())
            .map_err(|_| "emission list exceeds u16 entries".to_owned())?;
        for &(token, value) in list {
            let token =
                i32::try_from(token).map_err(|_| format!("emission token {token} exceeds i32"))?;
            emit.extend_from_slice(&token.to_le_bytes());
            emit.extend_from_slice(&value.raw().to_le_bytes());
        }
        emission_list_entries += list.len() as u32;
    }
    let max_emission_entries = DEFAULT_MAX_EMISSION_ENTRIES
        .max(emission_len[1..].iter().copied().max().unwrap_or(0) as u32);

    // ROUT: [HALT + padding][(1 + R) × W prototype words][same masks].
    let sig_words = SIG_WORDS as u32;
    let mut rout = Vec::with_capacity(8 + node_total * SIG_WORDS * 8 * 2);
    rout.push(0x00); // HALT
    rout.extend_from_slice(&[0u8; 7]); // program padding to 8-byte alignment
    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root prototype: zeros
    for region in regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].copy_from_slice(&region.sig);
        rout.extend_from_slice(&words);
    }
    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root mask: zeros
    for _ in regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].fill(0xFF); // all-ones mask (v1)
        rout.extend_from_slice(&words);
    }

    // NODE: the root record is all zeros; regions follow ascending id.
    let mut node_section = Vec::with_capacity(node_total * 30);
    node_section.extend_from_slice(&[0u8; 30]);
    for (index, region) in regions.iter().enumerate() {
        let i = 1 + index;
        node_section.extend_from_slice(&child_start[i].to_le_bytes());
        node_section.extend_from_slice(&child_len[i].to_le_bytes());
        node_section.extend_from_slice(&forward_start[i].to_le_bytes());
        node_section.extend_from_slice(&forward_len[i].to_le_bytes());
        node_section.extend_from_slice(&emission_start[i].to_le_bytes());
        node_section.extend_from_slice(&emission_len[i].to_le_bytes());
        node_section.extend_from_slice(&(1 + (i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&(1 + (node_count + i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&region.radius.to_le_bytes());
        node_section.push(region.depth);
        node_section.push(0); // flags
    }

    // EDGE: canonical records followed by the reverse index.
    let mut edge_section = Vec::with_capacity(edges.len() * 20);
    for edge in &edges {
        edge_section.extend_from_slice(&edge.src.to_le_bytes());
        edge_section.extend_from_slice(&edge.dst.to_le_bytes());
        edge_section.extend_from_slice(&edge.score_q.raw().to_le_bytes());
        edge_section.push(edge.kind);
        edge_section.push(0); // flags
        edge_section.extend_from_slice(&0u16.to_le_bytes()); // reserved
    }
    for &id in &reverse {
        edge_section.extend_from_slice(&id.to_le_bytes());
    }

    // EXCT: descriptor + compile-time residualized exact-context tables.
    #[allow(deprecated)]
    let store = runtime::parse_store(exct_tls1)
        .or_else(|| runtime::parse_store_legacy_u16(exct_tls1))
        .ok_or("EXCT input is not a TLS1 store")?;
    let (exct_body, exact_context_quantization) = emit_residual_exct(
        &store,
        &emissions.root_prior,
        emissions.root_floor,
        vocab_size,
        exct_top_x,
        emissions.smoothing,
    )?;
    let mut exct = Vec::with_capacity(4 + exct_body.len());
    exct.extend_from_slice(&[2, 0, 0, 0]);
    exct.extend_from_slice(&exct_body);
    let ngram = encode_context_rows(context_rows)?;

    let fmm_table = fmm_section
        .map(uor_r4_graph_format::FmmTranslationTable::parse)
        .transpose()
        .map_err(|error| format!("invalid FMM section: {error}"))?;

    // HEAD: the fixed 224-byte v0 prefix (convert_r4g1 conventions).
    let (meta, recs) = corpus_cid_material;
    let mut corpus_hasher = blake3::Hasher::new();
    corpus_hasher.update(meta);
    corpus_hasher.update(recs);
    let mut head = Vec::with_capacity(224);
    head.extend_from_slice(blake3::hash(artifact_container).as_bytes()); // teacher_cid
    head.extend_from_slice(&[0u8; 32]); // tokenizer_cid: not carried
    head.extend_from_slice(corpus_hasher.finalize().as_bytes()); // corpus_construction_cid
    head.extend_from_slice(&[0u8; 32]); // corpus_certification_cid: zeroed
    head.extend_from_slice(&[0u8; 20]); // hf_revision: zeroed
    head.extend_from_slice(blake3::hash(COMPILER_VERSION_LABEL).as_bytes());
    head.extend_from_slice(&max_frontier_width.to_le_bytes()); // A
    head.extend_from_slice(&MAX_CANDIDATES.to_le_bytes()); // C
    head.extend_from_slice(&(SIG_WORDS as u16).to_le_bytes()); // W
    head.extend_from_slice(&SHORTLIST_SIZE.to_le_bytes()); // K
    head.extend_from_slice(&max_emission_entries.to_le_bytes()); // E
    head.extend_from_slice(&MAX_PROGRAM_STEPS.to_le_bytes()); // D
    head.extend_from_slice(&node_count.to_le_bytes());
    head.extend_from_slice(&edge_count.to_le_bytes());
    head.push(depth_count);
    head.extend_from_slice(&[0u8; 5]); // fallback policy: unset
    head.extend_from_slice(&[0u8; 2]); // reserved
    head.extend_from_slice(&(SIG_BYTES as u16).to_le_bytes()); // signature_bytes
    head.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_major
    head.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_minor
    head.extend_from_slice(&0u16.to_le_bytes()); // feature_bits_required
    head.extend_from_slice(&vocab_size.to_le_bytes());
    debug_assert_eq!(head.len(), 224);

    let mut builder = uor_r4_graph_format::ArtifactBuilder::new(6);
    builder.add_section(uor_r4_graph_format::SectionId::HEAD, 0, &head);
    builder.add_section(uor_r4_graph_format::SectionId::NODE, 0, &node_section);
    builder.add_section(uor_r4_graph_format::SectionId::EDGE, 0, &edge_section);
    builder.add_section(uor_r4_graph_format::SectionId::ROUT, 0, &rout);
    builder.add_section(uor_r4_graph_format::SectionId::EMIT, 0, &emit);
    builder.add_section(uor_r4_graph_format::SectionId::EXCT, 0, &exct);
    builder.add_section(uor_r4_graph_format::SectionId::NGRAM, 0, &ngram);
    if let Some(fmm_section) = fmm_section {
        builder.add_section(uor_r4_graph_format::SectionId::FMM, 0, fmm_section);
    }
    let bytes = builder
        .build()
        .map_err(|error| format!("R4G1 serialization failed: {error}"))?;

    // Fail closed: never emit an artifact the two-stage validator or the
    // integrity CIDs reject.
    let view = uor_r4_graph_format::GraphView::parse(&bytes)
        .map_err(|error| format!("score emitted an invalid R4G1 artifact: {error}"))?;
    view.verify_cids()
        .map_err(|error| format!("score emitted an artifact with bad CIDs: {error}"))?;

    let artifact_bytes = bytes.len();
    Ok((
        bytes,
        ScoredGraphInfo {
            node_count,
            edge_count,
            refinement_edges,
            neighbor_edges,
            forward_edges,
            depth_count,
            max_frontier_width,
            max_emission_entries,
            root_prior_entries: root_entry_count,
            emission_list_entries,
            exct_bytes: exct.len() as u32,
            context_row_count: u32::try_from(context_rows.len())
                .map_err(|_| "NGRAM row count exceeds u32".to_owned())?,
            context_entry_count: u32::try_from(
                context_rows
                    .iter()
                    .map(|row| row.entries.len())
                    .sum::<usize>(),
            )
            .map_err(|_| "NGRAM entry count exceeds u32".to_owned())?,
            context_bytes: u32::try_from(ngram.len())
                .map_err(|_| "NGRAM section exceeds u32".to_owned())?,
            fmm_bytes: fmm_section.map_or(0, |bytes| bytes.len() as u32),
            fmm_rank: fmm_table.map_or(0, |table| table.rank()),
            fmm_candidate_count: fmm_table.map_or(0, |table| table.token_count()),
            artifact_bytes,
            transition_quantization,
            root_prior_quantization: emissions.root_prior_quantization,
            emission_quantization: emissions.emission_quantization,
            exact_context_quantization,
        },
    ))
}

/// Recover the scoring inputs of a previously emitted cover or scored
/// R4G1 artifact: region parameters and the structural (non-forward)
/// edges. Used by the `--cover` CLI path; byte-identical to the
/// re-induced inputs by construction (deterministic double-run).
pub fn recover_from_artifact(
    r4g1: &[u8],
) -> Result<(Vec<RegionParams>, Vec<StructuralEdge>), String> {
    let view = uor_r4_graph_format::GraphView::parse(r4g1)
        .map_err(|error| format!("invalid cover artifact: {error}"))?;
    view.verify_cids()
        .map_err(|error| format!("cover artifact has bad CIDs: {error}"))?;
    let regions = regions_from_view(&view)?;
    let structural = structural_edges_from_view(&view);
    Ok((regions, structural))
}

/// Convert an induced cover into the scoring region parameters.
pub fn regions_from_cover(cover: &cover::Cover) -> Vec<RegionParams> {
    cover
        .regions
        .iter()
        .map(|region| RegionParams {
            node: cover::region_node_id(region.id),
            depth: region.depth,
            radius: region.radius,
            sig: region.sig,
            parent: region.parent,
        })
        .collect()
}

/// Convert the cover's canonical edges into structural edges (score 0 —
/// the cover carries no log-domain edge scores; E_f is compiled here).
pub fn structural_from_cover(edges: &[cover::CoverEdge]) -> Vec<StructuralEdge> {
    edges
        .iter()
        .map(|e| StructuralEdge {
            src: e.src,
            kind: e.kind,
            dst: e.dst,
            score_q: ScoreQ::ZERO,
        })
        .collect()
}

/// Witten-Bell backoff probability of `next` under the graded store —
/// the `evaluate-report` bits/token semantics, shared so the Gate C
/// baseline and the HF evaluation report compute identical numbers.
pub fn witten_bell_probability(store: &Store, code: &[u8; STAGES], next: u32) -> f64 {
    let mut levels: Vec<(f64, &BTreeMap<u32, u32>, u32)> = Vec::new();
    for (depth, level) in store.iter().enumerate().take(STAGES + 1) {
        let key = code[..depth].to_vec();
        if let Some(distribution) = level.get(&key) {
            let total: u32 = distribution.values().sum();
            let lambda = total as f64 / (total as f64 + distribution.len() as f64);
            levels.push((lambda, distribution, total));
        }
    }
    let mut remaining = 1.0f64;
    let mut probability = 0.0f64;
    for index in (0..levels.len()).rev() {
        let weight = remaining * levels[index].0;
        remaining *= 1.0 - levels[index].0;
        if let Some(&count) = levels[index].1.get(&next) {
            probability += weight * count as f64 / levels[index].2 as f64;
        }
    }
    (probability + remaining / compiler::V as f64).max(1e-30)
}

/// Certifier-side bits/token of one graph-scorer outcome: the candidate
/// scores treated as natural-log weights (ScoreQ carries ln × 2⁻¹⁶),
/// non-candidate tokens held at the baked root smoothing floor (f64,
/// deterministic same-platform; computed max-shifted in the natural-log
/// domain so extreme residuals cannot underflow the accumulator).
fn outcome_bits(scorer: &GraphScorer, candidates: &[(u32, ScoreQ)], next: u32) -> f64 {
    let floor = scorer.root_floor().raw();
    let max_s = candidates
        .iter()
        .map(|&(_, s)| s.raw())
        .max()
        .unwrap_or(floor)
        .max(floor);
    let weight = |s: i32| ((f64::from(s) - f64::from(max_s)) / 65536.0).exp();
    let mut sum = 0f64;
    let mut w_next = None;
    for &(token, score) in candidates {
        let w = weight(score.raw());
        sum += w;
        if token == next {
            w_next = Some(w);
        }
    }
    let w_floor = weight(floor);
    let uncovered = (scorer.vocab() as usize).saturating_sub(candidates.len());
    sum += uncovered as f64 * w_floor;
    let w = w_next.unwrap_or(w_floor).max(1e-300);
    (sum / w).ln() / std::f64::consts::LN_2
}

/// One metric set of the Gate C table.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCMetrics {
    pub positions: usize,
    /// P(selected token == recorded teacher argmax).
    pub top1_agreement: f64,
    pub bits_per_token: f64,
}

/// Per-status position counts of the Rule 1+2 scorer (D4 precedence).
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusCounts {
    pub exact_context: usize,
    pub graph: usize,
    pub novel: usize,
}

/// Rule 1+2 metrics split by the status that fired. A bucket with zero
/// positions reports zeroed rates (no meaningful average exists).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Rule12PerStatus {
    pub exact_context: GateCMetrics,
    pub graph: GateCMetrics,
    pub novel: GateCMetrics,
}

/// Teacher-argmax correctness cross-tab of two scorers over the same
/// positions: `scorer_only` positions are wins for the scorer named
/// first in the pairing key, `other_only` are its losses.
#[derive(Debug, Clone, Default, Serialize)]
pub struct WinLoss {
    pub both_correct: usize,
    pub scorer_only: usize,
    pub other_only: usize,
    pub neither: usize,
}

/// Per-rule win/loss breakdowns (instrumentation honesty: not just
/// aggregates — where each rule wins and loses).
#[derive(Debug, Clone, Default, Serialize)]
pub struct WinLossReport {
    pub rule12_vs_baseline: WinLoss,
    pub rule12_vs_legacy: WinLoss,
    pub rule1_vs_baseline: WinLoss,
}

/// Candidate-set recall, reported separately from selected-token
/// agreement: a low value means the scorer cannot recover the teacher
/// token regardless of how its weights are tuned. Top-3 uses the
/// corpus's recorded teacher top-3 tokens.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CandidateRecall {
    pub rule1_top1: f64,
    pub rule1_top3: f64,
    pub rule12_top1: f64,
    pub rule12_top3: f64,
}

/// Top-1 agreement under `root + alpha * residual`, swept over alpha.
///
/// The residual measured net-destructive on the graph slice. Sweeping alpha
/// separates the two explanations that finding leaves open: an optimum at
/// alpha < 0 implicates the accumulation DIRECTION (a sign error in how
/// chain-telescoped residuals combine), while an optimum at 0 < alpha < 1
/// implicates SCALE (the evidence points the right way but is weighted far too
/// heavily). An optimum at alpha = 0 means the graph evidence carries no usable
/// signal on this path at all, which is a compiler-side question rather than a
/// scorer one.
///
/// alpha = 1 is the shipped behavior; alpha = 0 is the root prior alone.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResidualAlphaSweep {
    pub positions: usize,
    /// `(alpha, top1_agreement)` in the order swept.
    pub points: Vec<(f64, f64)>,
}

/// Alpha sweep split by resolution status.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Rule12PerStatusAlphaSweep {
    pub exact_context: ResidualAlphaSweep,
    pub graph: ResidualAlphaSweep,
    pub novel: ResidualAlphaSweep,
}

/// Rational alpha multipliers, kept exact in i64 so the sweep introduces no
/// float rounding of its own.
const ALPHA_SWEEP: [(i64, i64); 7] = [(-1, 1), (-1, 2), (-1, 4), (0, 1), (1, 4), (1, 2), (1, 1)];

/// Does the graph residual change any decision, or is ranking carried by the
/// root prior alone?
///
/// Scores are assembled as `root_score(token) + residual + transition_offset`,
/// minus a repetition penalty. `transition_offset` is added to every candidate,
/// so it cannot affect ordering; only the residual varies per token. This
/// compares the real argmax against the argmax of `root + penalty` alone, using
/// the identical ascending-token / strict-`>` tie-break, so a difference means
/// the residual genuinely moved the selection.
///
/// `root_only_agrees` near 1.0 on the graph slice would mean the residual is
/// inert: the traversal picks a region and the scorer then ranks by global token
/// frequency regardless.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResidualInfluence {
    pub positions: usize,
    /// Fraction where argmax(root + penalty) == argmax(full score).
    pub root_only_agrees: f64,
    /// Fraction where argmax(root + penalty) is the teacher token.
    pub root_only_top1_agreement: f64,
    /// Median spread (max - min) of the root term across candidates, Q16.16.
    pub median_root_spread: f64,
    /// Median spread (max - min) of the residual term across candidates, Q16.16.
    pub median_residual_spread: f64,
    /// bits/token with the residual suppressed. Compare against the slice's
    /// `bits_per_token`: a lower top-1 with unchanged bits means the argmax
    /// moved without the distribution improving.
    pub bits_per_token_root_only: f64,
    /// Mean share of candidates whose residual is exactly zero (off-chain).
    pub mean_zero_residual_share: f64,
    /// Fraction of positions where the teacher token is a chain token.
    pub teacher_on_chain: f64,
    /// Fraction of positions where an off-chain (zero-residual) token won.
    pub selected_off_chain: f64,
    /// An active region OUTSIDE the selected chain emits the teacher token.
    pub teacher_emitted_off_chain: f64,
    /// That region also fits the context better than the selected chain.
    pub teacher_emitter_better_margin: f64,
    /// Mean selected-chain depth, and mean depth of the best teacher emitter.
    pub mean_chain_depth: f64,
    pub mean_teacher_emitter_depth: f64,
    /// Which source supplies the teacher token, as a share of positions.
    /// These overlap: a token can come from more than one source.
    pub teacher_from_active: f64,
    pub teacher_from_predicted: f64,
    pub teacher_from_root_top: f64,
    /// Teacher retrieved, but ONLY by the context-free root prior — no graph
    /// region and no predicted transition supplied it.
    pub teacher_only_root_top: f64,
    /// Mean chain length, and mean number of chain levels carrying a term for
    /// the teacher token. A gap means the telescoping sum is incomplete.
    pub mean_chain_levels: f64,
    pub mean_chain_levels_emitting_teacher: f64,
    /// Share of positions where the teacher's chain terms are complete, and
    /// where they are present at some levels but missing at others.
    pub teacher_chain_complete: f64,
    pub teacher_chain_partial: f64,
    /// Control for whether the cover groups by next-token structure: rate at
    /// which the context's own region emits the teacher, versus an unrelated
    /// region. Equal rates mean routing carries no predictive information.
    pub own_region_emits_teacher: f64,
    pub random_region_emits_teacher: f64,
}

/// Residual-influence measurement split by resolution status.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Rule12PerStatusResidualInfluence {
    pub exact_context: ResidualInfluence,
    pub graph: ResidualInfluence,
    pub novel: ResidualInfluence,
}

/// Where the teacher's argmax lands in the Rule 1+2 candidate list, for
/// positions whose candidate set actually contains it.
///
/// Recall tells us the token was retrieved; agreement tells us it was not
/// selected. This says HOW BADLY it was ranked, which separates two diagnoses
/// with different fixes: a teacher token sitting at rank 2-3 is a calibration
/// or tie-breaking problem, while one scattered deep in the list means the
/// scoring signal carries little information on that path.
///
/// Buckets are 1-based rank: [1, 2, 3, 4-8, 9-16, 17-32, 33-64, 65-128, 129+].
#[derive(Debug, Clone, Default, Serialize)]
pub struct TeacherRankHistogram {
    /// Positions where the teacher argmax was present in the candidate list.
    pub retrieved_positions: usize,
    pub buckets: [usize; 9],
    /// Median 1-based rank over `retrieved_positions`.
    pub median_rank: f64,
}

/// Teacher-rank histograms split by resolution status.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Rule12PerStatusTeacherRank {
    pub exact_context: TeacherRankHistogram,
    pub graph: TeacherRankHistogram,
    pub novel: TeacherRankHistogram,
}

/// Candidate recall for one resolution status.
///
/// `CandidateRecall` above is divided by ALL scored positions, so it cannot
/// attribute retrieval success to a status. That matters for the graph path
/// specifically: its top-1 agreement is ~0.6-1.2% while blended recall is
/// ~72%, and without this split there is no way to tell whether the graph
/// path fails to RETRIEVE the teacher token or retrieves it and fails to RANK
/// it first. Those have disjoint fixes.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusCandidateRecall {
    pub positions: usize,
    pub top1: f64,
    pub top3: f64,
}

/// Rule 1+2 candidate recall, split by resolution status.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Rule12PerStatusRecall {
    pub exact_context: StatusCandidateRecall,
    pub graph: StatusCandidateRecall,
    pub novel: StatusCandidateRecall,
}

/// The Gate C outcome: the four number sets (old formula, Rule 1,
/// Rule 1+2, baseline), the status and win/loss instrumentation,
/// candidate recall, and the witness-replay sample result.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCOutcome {
    /// OLD Σ-over-cloud formula (with EXCT evidence wired), kept for
    /// comparison — the confirmed double counting lives there.
    pub legacy_sum: GateCMetrics,
    /// NEW Rule 1 (chain-telescoped residuals, no EXCT).
    pub rule1_chain: GateCMetrics,
    /// NEW Rule 1+2 (chain-telescoped + D4 EXCT precedence).
    pub rule12_precedence: GateCMetrics,
    /// Ablation (issue #66): Rule 1 with predicted-cloud (ΔT) emissions
    /// disabled — the no-EXCT measure of ΔT's contribution.
    pub rule1_chain_no_f: GateCMetrics,
    /// Ablation (issue #66): Rule 1+2 with ΔT emissions disabled — the
    /// precedence path's measure of ΔT's contribution.
    pub rule12_precedence_no_f: GateCMetrics,
    /// Candidate variant (issue #80): Cloud-size normalized scoring.
    pub rule12_cloud_size_normalized: GateCMetrics,
    /// Candidate variant (issue #80): Margin-weighted residual scoring.
    pub rule12_margin_weighted: GateCMetrics,
    /// TLA3 store baseline (`runtime::predict_witness_plain`).
    pub tla3_baseline: GateCMetrics,
    pub rule12_status_counts: StatusCounts,
    pub rule12_per_status: Rule12PerStatus,
    /// Candidate recall split by status (retrieval vs ranking).
    pub rule12_candidate_recall_per_status: Rule12PerStatusRecall,
    /// Teacher-argmax rank within the candidate list, per status.
    pub rule12_teacher_rank_per_status: Rule12PerStatusTeacherRank,
    /// Whether the graph residual changes decisions, per status.
    pub rule12_residual_influence_per_status: Rule12PerStatusResidualInfluence,
    /// Top-1 under root + alpha*residual, swept, per status.
    pub rule12_residual_alpha_sweep: Rule12PerStatusAlphaSweep,
    /// #234 item 2 instrumentation: histogram of the Rule 2 probe's
    /// RESOLUTION level per held-out position (index = graded-prefix
    /// length, 0 = root … STAGES = full code). The probe stops at the
    /// deepest populated prefix and level 0 is populated on any
    /// non-empty train split, so the status-based miss rate cannot move;
    /// this histogram (and the strict full-depth count below) is what a
    /// D3 construction can actually change.
    pub rule12_exct_probe_levels: Vec<usize>,
    /// Held-out positions where no EXCT probe was consulted at all.
    pub rule12_exct_probe_absent: usize,
    /// Positions resolved at the FULL graded code with support — exact
    /// context in the strict sense.
    pub rule12_exct_full_depth_supported: usize,
    pub win_loss: WinLossReport,
    pub candidate_recall: CandidateRecall,
    pub repetition_rate_rule12: f64,
    pub repetition_rate_baseline: f64,
    pub witness_replays: usize,
    pub witness_replay_failures: usize,
}

fn generate_greedy_repetition_rate(
    scorer: &GraphScorer,
    artifacts: &compiler::Compiled,
    rotations: &[usize; compiler::WINDOW + 1],
    seed: &[u32],
    tokens_to_generate: usize,
) -> Result<f64, String> {
    let mut window = [0u32; compiler::WINDOW];
    let mut recent_tokens = std::collections::VecDeque::with_capacity(32);
    let seed_len = seed.len();

    let w_len = seed_len.min(compiler::WINDOW);
    window[..w_len].copy_from_slice(&seed[seed_len - w_len..]);

    let r_len = seed_len.min(32);
    for &t in &seed[seed_len - r_len..] {
        recent_tokens.push_back(t);
    }

    let mut recent_array = [0u32; 32];
    let mut duplicate_count = 0;

    for _ in 0..tokens_to_generate {
        let bundle = runtime::bundle_window_plain(artifacts, rotations, &window[..w_len]);
        let sig = runtime::sig_plain(artifacts, &bundle);
        // #243 Phase C option A: attest the metric-respecting code.
        let code = runtime::assign_for_bundle(artifacts, &bundle);

        let recent_len = recent_tokens.len();
        for (i, &t) in recent_tokens.iter().enumerate() {
            recent_array[i] = t;
        }
        let outcome =
            scorer.score_candidates_coded(&sig, Some(&code), &recent_array[..recent_len])?;
        let token = outcome.selected;

        if recent_tokens.contains(&token) {
            duplicate_count += 1;
        }

        if w_len < compiler::WINDOW {
            window[w_len] = token;
        } else {
            window.copy_within(1.., 0);
            window[compiler::WINDOW - 1] = token;
        }

        if recent_tokens.len() == 32 {
            recent_tokens.pop_front();
        }
        recent_tokens.push_back(token);
    }

    Ok(duplicate_count as f64 / tokens_to_generate as f64)
}

fn baseline_greedy_repetition_rate(
    store: &Store,
    artifacts: &compiler::Compiled,
    rotations: &[usize; compiler::WINDOW + 1],
    seed: &[u32],
    tokens_to_generate: usize,
) -> f64 {
    let mut window = [0u32; compiler::WINDOW];
    let mut recent_tokens = std::collections::VecDeque::with_capacity(32);
    let seed_len = seed.len();

    let w_len = seed_len.min(compiler::WINDOW);
    window[..w_len].copy_from_slice(&seed[seed_len - w_len..]);

    let r_len = seed_len.min(32);
    for &t in &seed[seed_len - r_len..] {
        recent_tokens.push_back(t);
    }

    let mut duplicate_count = 0;

    for _ in 0..tokens_to_generate {
        let bundle = runtime::bundle_window_plain(artifacts, rotations, &window[..w_len]);
        // #243 Phase C: the store is keyed under the artifact's declared
        // metric — query it the same way (assign_for_bundle).
        let code = runtime::assign_for_bundle(artifacts, &bundle);
        let p = runtime::predict_witness_plain(store, &code);
        let token = p.token;

        if recent_tokens.contains(&token) {
            duplicate_count += 1;
        }

        if w_len < compiler::WINDOW {
            window[w_len] = token;
        } else {
            window.copy_within(1.., 0);
            window[compiler::WINDOW - 1] = token;
        }

        if recent_tokens.len() == 32 {
            recent_tokens.pop_front();
        }
        recent_tokens.push_back(token);
    }

    duplicate_count as f64 / tokens_to_generate as f64
}

fn accumulate_win_loss(win_loss: &mut WinLoss, scorer_hit: bool, other_hit: bool) {
    match (scorer_hit, other_hit) {
        (true, true) => win_loss.both_correct += 1,
        (true, false) => win_loss.scorer_only += 1,
        (false, true) => win_loss.other_only += 1,
        (false, false) => win_loss.neither += 1,
    }
}

/// The Gate C measurement (plan §8 gate C): top-1 teacher-argmax
/// agreement and bits/token on the held-out partition for four scorers
/// side by side — the OLD Σ-over-cloud formula (kept for comparison),
/// NEW Rule 1 (chain-telescoped, no EXCT), NEW Rule 1+2 (with D4 EXCT
/// precedence), and the TLA3 store baseline on the same positions
/// (Witten-Bell bits as in `evaluate-report`). All graph scorers are
/// rebuilt from the emitted artifact bytes (the artifact is the scoring
/// authority); a bounded sample of Rule 1+2 witnesses is independently
/// replayed (Theorem 6).
pub fn evaluate_gate_c(
    r4g1: &[u8],
    artifact_container: &[u8],
    artifacts: &compiler::Compiled,
    store: &Store,
    corpus: &Corpus,
    held_out: &[Observation],
    config: &ScoreConfig,
) -> Result<GateCOutcome, String> {
    let mut scorer_no_exct =
        GraphScorer::from_artifact(r4g1, None, config.root_top_b, config.exct_top_x)?;
    scorer_no_exct.set_f_emissions(true);
    scorer_no_exct.set_scoring_variant(config.scoring_variant);
    let mut scorer_with_exct = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )?;
    scorer_with_exct.set_f_emissions(true);
    scorer_with_exct.set_scoring_variant(config.scoring_variant);
    // Ablation scorers (issue #66): identical configs with ΔT emissions off
    // (the deployed default since the ablation decision).
    let mut scorer_no_exct_no_f =
        GraphScorer::from_artifact(r4g1, None, config.root_top_b, config.exct_top_x)?;
    scorer_no_exct_no_f.set_scoring_variant(config.scoring_variant);
    let mut scorer_with_exct_no_f = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )?;
    scorer_with_exct_no_f.set_scoring_variant(config.scoring_variant);
    let mut scorer_normalized = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )?;
    scorer_normalized.set_scoring_variant(ScoringVariant::CloudSizeNormalized);
    let mut scorer_margin = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )?;
    scorer_margin.set_scoring_variant(ScoringVariant::MarginWeighted);

    let mut outcome = GateCOutcome::default();
    let mut bits_legacy = 0f64;
    let mut bits_rule1 = 0f64;
    let mut bits_rule12 = 0f64;
    let mut bits_rule1_no_f = 0f64;
    let mut bits_rule12_no_f = 0f64;
    let mut bits_normalized = 0f64;
    let mut bits_margin = 0f64;
    let mut bits_baseline = 0f64;
    let mut hits_legacy = 0u64;
    let mut hits_rule1 = 0u64;
    let mut hits_rule12 = 0u64;
    let mut hits_rule1_no_f = 0u64;
    let mut hits_rule12_no_f = 0u64;
    let mut hits_normalized = 0u64;
    let mut hits_margin = 0u64;
    let mut hits_baseline = 0u64;
    // Per-status Rule 1+2 accumulators: [ExactContext, Graph, Novel].
    let mut status_positions = [0usize; 3];
    let mut status_hits = [0u64; 3];
    let mut status_bits = [0f64; 3];
    // #234 item 2 instrumentation: WHERE the Rule 2 probe resolved. The
    // probe stops at the deepest POPULATED graded prefix, and the level-0
    // (root) prefix is populated by construction on any non-empty train
    // split with total ≥ EXCT_SUPPORT_MIN — so ScoreStatus::ExactContext
    // alone cannot miss on any corpus, and the status-based miss rate is
    // structurally ~0. The per-level histogram is the quantity a D3
    // construction can actually move; "strict" exact-context = resolved
    // at the FULL graded code with support.
    let mut exct_level_positions = vec![0usize; STAGES + 1];
    let mut exct_probe_absent = 0usize;
    let mut exct_full_depth_supported = 0usize;
    let mut recall_rule1_top1 = 0u64;
    let mut recall_rule1_top3 = 0u64;
    let mut recall_rule12_top1 = 0u64;
    let mut recall_rule12_top3 = 0u64;
    let mut status_recall_top1 = [0u64; 3];
    let mut status_recall_top3 = [0u64; 3];
    let mut status_rank_buckets = [[0usize; 9]; 3];
    let mut status_root_agrees = [0u64; 3];
    let mut status_root_hits = [0u64; 3];
    let mut status_root_spreads: [Vec<i64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut status_resid_spreads: [Vec<i64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut status_alpha_hits = [[0u64; ALPHA_SWEEP.len()]; 3];
    let mut status_bits_root_only = [0f64; 3];
    let mut status_zero_share = [0f64; 3];
    let mut status_teacher_on_chain = [0u64; 3];
    let mut status_selected_off_chain = [0u64; 3];
    let mut status_teacher_off_chain = [0u64; 3];
    let mut status_teacher_better_margin = [0u64; 3];
    let mut status_chain_depth = [0u64; 3];
    let mut status_emitter_depth = [0u64; 3];
    let mut status_emitter_rows = [0u64; 3];
    let mut status_src_active = [0u64; 3];
    let mut status_src_predicted = [0u64; 3];
    let mut status_src_root_top = [0u64; 3];
    let mut status_src_only_root = [0u64; 3];
    let mut status_chain_levels = [0u64; 3];
    let mut status_chain_emit_levels = [0u64; 3];
    let mut status_chain_complete = [0u64; 3];
    let mut status_chain_partial = [0u64; 3];
    let mut status_own_emits = [0u64; 3];
    let mut status_rand_emits = [0u64; 3];
    let mut status_ranks: [Vec<u32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let gate_rotations = compiler::derive_rotations();
    let context = GateCContext {
        artifacts,
        corpus,
        store,
        gate_rotations: &gate_rotations,
        scorer_no_exct: &scorer_no_exct,
        scorer_with_exct: &scorer_with_exct,
        scorer_no_exct_no_f: &scorer_no_exct_no_f,
        scorer_with_exct_no_f: &scorer_with_exct_no_f,
        scorer_normalized: &scorer_normalized,
        scorer_margin: &scorer_margin,
        r4g1,
        artifact_container,
        config,
    };
    // Each held-out position is independent. Collect compact rows in Rayon;
    // reduce them in input order below so floating-point totals and all
    // report bytes retain the serial implementation's determinism.
    let rows: Vec<GateCRow> = held_out
        .par_iter()
        .enumerate()
        .map(|(index, observation)| evaluate_gate_c_row(index, observation, &context))
        .collect::<Result<Vec<_>, _>>()?;
    for row in rows {
        hits_legacy += u64::from(row.hits[0]);
        hits_rule1 += u64::from(row.hits[1]);
        hits_rule12 += u64::from(row.hits[2]);
        hits_rule1_no_f += u64::from(row.hits[3]);
        hits_rule12_no_f += u64::from(row.hits[4]);
        hits_normalized += u64::from(row.hits[5]);
        hits_margin += u64::from(row.hits[6]);
        hits_baseline += u64::from(row.hits[7]);
        bits_legacy += row.bits[0];
        bits_rule1 += row.bits[1];
        bits_rule12 += row.bits[2];
        bits_rule1_no_f += row.bits[3];
        bits_rule12_no_f += row.bits[4];
        bits_normalized += row.bits[5];
        bits_margin += row.bits[6];
        bits_baseline += row.bits[7];
        status_positions[row.status_index] += 1;
        status_hits[row.status_index] += u64::from(row.status_hit);
        status_bits[row.status_index] += row.status_bits;
        status_bits_root_only[row.status_index] += row.bits_root_only;
        status_zero_share[row.status_index] += row.zero_resid_share;
        status_teacher_on_chain[row.status_index] += u64::from(row.teacher_on_chain);
        status_selected_off_chain[row.status_index] += u64::from(row.selected_off_chain);
        status_teacher_off_chain[row.status_index] += u64::from(row.teacher_emitted_off_chain);
        status_teacher_better_margin[row.status_index] +=
            u64::from(row.teacher_emitter_better_margin);
        status_chain_depth[row.status_index] += u64::from(row.chain_depth);
        status_src_active[row.status_index] += u64::from(row.teacher_from_active);
        status_src_predicted[row.status_index] += u64::from(row.teacher_from_predicted);
        status_src_root_top[row.status_index] += u64::from(row.teacher_from_root_top);
        status_src_only_root[row.status_index] += u64::from(row.teacher_only_root_top);
        status_chain_levels[row.status_index] += u64::from(row.chain_levels);
        status_chain_emit_levels[row.status_index] += u64::from(row.chain_levels_emitting_teacher);
        status_chain_complete[row.status_index] += u64::from(row.teacher_chain_complete);
        status_chain_partial[row.status_index] += u64::from(row.teacher_chain_partial);
        status_own_emits[row.status_index] += u64::from(row.own_region_emits_teacher);
        status_rand_emits[row.status_index] += u64::from(row.random_region_emits_teacher);
        if row.teacher_emitted_off_chain {
            status_emitter_depth[row.status_index] += u64::from(row.teacher_emitter_depth);
            status_emitter_rows[row.status_index] += 1;
        }
        if let Some(level) = row.exct_level {
            exct_level_positions[level] += 1;
        } else {
            exct_probe_absent += 1;
        }
        exct_full_depth_supported += usize::from(row.exct_full_depth_supported);
        recall_rule1_top1 += u64::from(row.candidate_recall[0]);
        recall_rule1_top3 += u64::from(row.candidate_recall[1]);
        recall_rule12_top1 += u64::from(row.candidate_recall[2]);
        recall_rule12_top3 += u64::from(row.candidate_recall[3]);
        status_recall_top1[row.status_index] += u64::from(row.candidate_recall[2]);
        status_recall_top3[row.status_index] += u64::from(row.candidate_recall[3]);
        status_root_agrees[row.status_index] += u64::from(row.root_only_agrees);
        status_root_hits[row.status_index] += u64::from(row.root_only_hit);
        status_root_spreads[row.status_index].push(row.root_spread);
        status_resid_spreads[row.status_index].push(row.residual_spread);
        for (slot, hit) in status_alpha_hits[row.status_index]
            .iter_mut()
            .zip(row.alpha_hits.iter())
        {
            *slot += u64::from(*hit);
        }
        if let Some(rank) = row.teacher_rank {
            let bucket = match rank {
                1 => 0,
                2 => 1,
                3 => 2,
                4..=8 => 3,
                9..=16 => 4,
                17..=32 => 5,
                33..=64 => 6,
                65..=128 => 7,
                _ => 8,
            };
            status_rank_buckets[row.status_index][bucket] += 1;
            status_ranks[row.status_index].push(rank);
        }
        accumulate_win_loss(
            &mut outcome.win_loss.rule12_vs_baseline,
            row.hits[2],
            row.hits[7],
        );
        accumulate_win_loss(
            &mut outcome.win_loss.rule12_vs_legacy,
            row.hits[2],
            row.hits[0],
        );
        accumulate_win_loss(
            &mut outcome.win_loss.rule1_vs_baseline,
            row.hits[1],
            row.hits[7],
        );
        outcome.witness_replays += usize::from(row.witness_replayed);
        outcome.witness_replay_failures += usize::from(row.witness_replay_failed);
    }
    let n = held_out.len();
    if n == 0 {
        return Err("held-out split is empty; cannot evaluate".to_owned());
    }
    let nf = n as f64;
    let metrics = |hits: u64, bits: f64| GateCMetrics {
        positions: n,
        top1_agreement: hits as f64 / nf,
        bits_per_token: bits / nf,
    };
    outcome.legacy_sum = metrics(hits_legacy, bits_legacy);
    outcome.rule1_chain = metrics(hits_rule1, bits_rule1);
    outcome.rule12_precedence = metrics(hits_rule12, bits_rule12);
    outcome.rule1_chain_no_f = metrics(hits_rule1_no_f, bits_rule1_no_f);
    outcome.rule12_precedence_no_f = metrics(hits_rule12_no_f, bits_rule12_no_f);
    outcome.rule12_cloud_size_normalized = metrics(hits_normalized, bits_normalized);
    outcome.rule12_margin_weighted = metrics(hits_margin, bits_margin);
    outcome.tla3_baseline = metrics(hits_baseline, bits_baseline);
    outcome.rule12_status_counts = StatusCounts {
        exact_context: status_positions[0],
        graph: status_positions[1],
        novel: status_positions[2],
    };
    outcome.rule12_exct_probe_levels = exct_level_positions;
    outcome.rule12_exct_probe_absent = exct_probe_absent;
    outcome.rule12_exct_full_depth_supported = exct_full_depth_supported;
    let per_status = |index: usize| {
        let positions = status_positions[index];
        if positions == 0 {
            return GateCMetrics::default();
        }
        let denom = positions as f64;
        GateCMetrics {
            positions,
            top1_agreement: status_hits[index] as f64 / denom,
            bits_per_token: status_bits[index] / denom,
        }
    };
    outcome.rule12_per_status = Rule12PerStatus {
        exact_context: per_status(0),
        graph: per_status(1),
        novel: per_status(2),
    };
    let per_status_recall = |index: usize| {
        let positions = status_positions[index];
        if positions == 0 {
            return StatusCandidateRecall::default();
        }
        let denom = positions as f64;
        StatusCandidateRecall {
            positions,
            top1: status_recall_top1[index] as f64 / denom,
            top3: status_recall_top3[index] as f64 / denom,
        }
    };
    outcome.rule12_candidate_recall_per_status = Rule12PerStatusRecall {
        exact_context: per_status_recall(0),
        graph: per_status_recall(1),
        novel: per_status_recall(2),
    };
    let median_of = |v: &mut Vec<i64>| -> f64 {
        if v.is_empty() {
            return 0.0;
        }
        v.sort_unstable();
        let mid = v.len() / 2;
        if v.len().is_multiple_of(2) {
            (v[mid - 1] as f64 + v[mid] as f64) / 2.0
        } else {
            v[mid] as f64
        }
    };
    let mut sweep = Rule12PerStatusAlphaSweep::default();
    for index in 0..3 {
        let positions = status_positions[index];
        if positions == 0 {
            continue;
        }
        let denom = positions as f64;
        let value = ResidualAlphaSweep {
            positions,
            points: ALPHA_SWEEP
                .iter()
                .zip(status_alpha_hits[index].iter())
                .map(|(&(num, den), &hits)| (num as f64 / den as f64, hits as f64 / denom))
                .collect(),
        };
        match index {
            0 => sweep.exact_context = value,
            1 => sweep.graph = value,
            _ => sweep.novel = value,
        }
    }
    outcome.rule12_residual_alpha_sweep = sweep;
    let mut influence = Rule12PerStatusResidualInfluence::default();
    for index in 0..3 {
        let positions = status_positions[index];
        if positions == 0 {
            continue;
        }
        let denom = positions as f64;
        let value = ResidualInfluence {
            positions,
            root_only_agrees: status_root_agrees[index] as f64 / denom,
            root_only_top1_agreement: status_root_hits[index] as f64 / denom,
            median_root_spread: median_of(&mut status_root_spreads[index]),
            median_residual_spread: median_of(&mut status_resid_spreads[index]),
            bits_per_token_root_only: status_bits_root_only[index] / denom,
            mean_zero_residual_share: status_zero_share[index] / denom,
            teacher_on_chain: status_teacher_on_chain[index] as f64 / denom,
            selected_off_chain: status_selected_off_chain[index] as f64 / denom,
            teacher_emitted_off_chain: status_teacher_off_chain[index] as f64 / denom,
            teacher_emitter_better_margin: status_teacher_better_margin[index] as f64 / denom,
            mean_chain_depth: status_chain_depth[index] as f64 / denom,
            mean_teacher_emitter_depth: if status_emitter_rows[index] == 0 {
                0.0
            } else {
                status_emitter_depth[index] as f64 / status_emitter_rows[index] as f64
            },
            teacher_from_active: status_src_active[index] as f64 / denom,
            teacher_from_predicted: status_src_predicted[index] as f64 / denom,
            teacher_from_root_top: status_src_root_top[index] as f64 / denom,
            teacher_only_root_top: status_src_only_root[index] as f64 / denom,
            mean_chain_levels: status_chain_levels[index] as f64 / denom,
            mean_chain_levels_emitting_teacher: status_chain_emit_levels[index] as f64 / denom,
            teacher_chain_complete: status_chain_complete[index] as f64 / denom,
            teacher_chain_partial: status_chain_partial[index] as f64 / denom,
            own_region_emits_teacher: status_own_emits[index] as f64 / denom,
            random_region_emits_teacher: status_rand_emits[index] as f64 / denom,
        };
        match index {
            0 => influence.exact_context = value,
            1 => influence.graph = value,
            _ => influence.novel = value,
        }
    }
    outcome.rule12_residual_influence_per_status = influence;
    let per_status_rank = |index: usize| {
        let ranks = &status_ranks[index];
        if ranks.is_empty() {
            return TeacherRankHistogram::default();
        }
        let mut sorted = ranks.clone();
        sorted.sort_unstable();
        let mid = sorted.len() / 2;
        let median = if sorted.len().is_multiple_of(2) {
            (f64::from(sorted[mid - 1]) + f64::from(sorted[mid])) / 2.0
        } else {
            f64::from(sorted[mid])
        };
        TeacherRankHistogram {
            retrieved_positions: sorted.len(),
            buckets: status_rank_buckets[index],
            median_rank: median,
        }
    };
    outcome.rule12_teacher_rank_per_status = Rule12PerStatusTeacherRank {
        exact_context: per_status_rank(0),
        graph: per_status_rank(1),
        novel: per_status_rank(2),
    };
    outcome.candidate_recall = CandidateRecall {
        rule1_top1: recall_rule1_top1 as f64 / nf,
        rule1_top3: recall_rule1_top3 as f64 / nf,
        rule12_top1: recall_rule12_top1 as f64 / nf,
        rule12_top3: recall_rule12_top3 as f64 / nf,
    };

    let rotations = runtime::derive_rotations();
    let mut graph_rep_sum = 0.0;
    let mut baseline_rep_sum = 0.0;
    let mut probe_count = 0;

    for obs in held_out.iter() {
        let pos = obs.position as usize;
        if pos >= 32 && corpus.story[pos] == corpus.story[pos - 32] {
            let seed = &corpus.input[pos - 32..pos];
            graph_rep_sum += generate_greedy_repetition_rate(
                &scorer_with_exct,
                artifacts,
                &rotations,
                seed,
                64,
            )?;
            baseline_rep_sum +=
                baseline_greedy_repetition_rate(store, artifacts, &rotations, seed, 64);
            probe_count += 1;
            if probe_count == 5 {
                break;
            }
        }
    }

    if probe_count > 0 {
        outcome.repetition_rate_rule12 = graph_rep_sum / probe_count as f64;
        outcome.repetition_rate_baseline = baseline_rep_sum / probe_count as f64;
    } else {
        outcome.repetition_rate_rule12 = 0.0;
        outcome.repetition_rate_baseline = 0.0;
    }

    Ok(outcome)
}

/// The `score_report.json` document. Schema history: 1 = the three-set
#[derive(Debug, Clone)]
struct GateCRow {
    hits: [bool; 8],
    bits: [f64; 8],
    status_index: usize,
    status_hit: bool,
    status_bits: f64,
    exct_level: Option<usize>,
    exct_full_depth_supported: bool,
    candidate_recall: [bool; 4],
    /// 1-based rank of the teacher argmax in rule12 candidates, if present.
    teacher_rank: Option<u32>,
    /// argmax(root + penalty) agrees with the full-score argmax.
    root_only_agrees: bool,
    /// argmax(root + penalty) is the teacher token.
    root_only_hit: bool,
    /// Spread (max - min) of the root and residual terms across candidates.
    root_spread: i64,
    residual_spread: i64,
    /// Per-alpha: argmax(root + alpha*residual + penalty) == teacher.
    alpha_hits: [bool; ALPHA_SWEEP.len()],
    /// bits/token under root + penalty only (alpha = 0).
    bits_root_only: f64,
    /// Share of candidates whose residual is exactly zero (off-chain tokens).
    zero_resid_share: f64,
    /// The teacher token carries a non-zero residual (it is a chain token).
    teacher_on_chain: bool,
    /// The selected token carries a zero residual (an off-chain token won).
    selected_off_chain: bool,
    /// An active region outside the selected chain emits the teacher token.
    teacher_emitted_off_chain: bool,
    /// That region's membership margin beats the selected chain's.
    teacher_emitter_better_margin: bool,
    /// Depth of the selected chain, and of the best teacher-emitting region.
    chain_depth: u32,
    teacher_emitter_depth: u32,
    /// Which source supplied the teacher token to the candidate set.
    teacher_from_active: bool,
    teacher_from_predicted: bool,
    teacher_from_root_top: bool,
    /// Teacher present, but ONLY from the context-free root prior.
    teacher_only_root_top: bool,
    /// Chain length, and how many of its levels emit the teacher token.
    /// The residual is a TELESCOPING sum: each level stores
    /// log P_level - log P_parent, so summing a complete root-to-leaf chain
    /// gives log P_leaf - log P_root. Truncation is per level, so a token kept
    /// at a descendant need not be kept at its ancestors; a missing level
    /// contributes 0 instead of its real correction and the sum is no longer
    /// log P_leaf. Count the levels that actually contribute.
    chain_levels: u32,
    chain_levels_emitting_teacher: u32,
    /// The teacher is emitted by the full chain (telescoping intact).
    teacher_chain_complete: bool,
    /// The teacher is emitted by some but not all chain levels (broken).
    teacher_chain_partial: bool,
    /// Control: does the context's OWN region emit the teacher, versus a
    /// deterministically chosen UNRELATED region? If the two rates match, the
    /// cover is not grouping contexts by next-token structure -- every region's
    /// emission list is essentially the same globally-common set, and routing
    /// carries no predictive information no matter how well each region is
    /// estimated.
    own_region_emits_teacher: bool,
    random_region_emits_teacher: bool,
    witness_replayed: bool,
    witness_replay_failed: bool,
}

struct GateCContext<'a> {
    artifacts: &'a compiler::Compiled,
    corpus: &'a Corpus,
    store: &'a Store,
    gate_rotations: &'a [usize; compiler::WINDOW + 1],
    scorer_no_exct: &'a GraphScorer,
    scorer_with_exct: &'a GraphScorer,
    scorer_no_exct_no_f: &'a GraphScorer,
    scorer_with_exct_no_f: &'a GraphScorer,
    scorer_normalized: &'a GraphScorer,
    scorer_margin: &'a GraphScorer,
    r4g1: &'a [u8],
    artifact_container: &'a [u8],
    config: &'a ScoreConfig,
}

fn evaluate_gate_c_row(
    index: usize,
    observation: &Observation,
    context: &GateCContext<'_>,
) -> Result<GateCRow, String> {
    let position = observation.position as usize;
    let teacher_argmax = context.corpus.t_argmax[position];
    let next = context.corpus.next[position];
    let code = runtime::code_plain(
        context.artifacts,
        context.gate_rotations,
        context.corpus,
        position,
    );

    let legacy = context
        .scorer_with_exct
        .score_candidates_legacy(&observation.sig)?;
    let rule1 =
        context
            .scorer_no_exct
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let rule12 =
        context
            .scorer_with_exct
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let rule1_no_f =
        context
            .scorer_no_exct_no_f
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let rule12_no_f =
        context
            .scorer_with_exct_no_f
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let normalized =
        context
            .scorer_normalized
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let margin =
        context
            .scorer_margin
            .score_candidates_coded(&observation.sig, Some(&code), &[])?;
    let baseline = runtime::predict_witness_plain(context.store, &code);

    let hits = [
        legacy.selected == teacher_argmax,
        rule1.selected == teacher_argmax,
        rule12.selected == teacher_argmax,
        rule1_no_f.selected == teacher_argmax,
        rule12_no_f.selected == teacher_argmax,
        normalized.selected == teacher_argmax,
        margin.selected == teacher_argmax,
        baseline.token == teacher_argmax,
    ];
    let bits = [
        outcome_bits(context.scorer_with_exct, &legacy.candidates, next),
        outcome_bits(context.scorer_no_exct, &rule1.candidates, next),
        outcome_bits(context.scorer_with_exct, &rule12.candidates, next),
        outcome_bits(context.scorer_no_exct_no_f, &rule1_no_f.candidates, next),
        outcome_bits(context.scorer_with_exct_no_f, &rule12_no_f.candidates, next),
        outcome_bits(context.scorer_normalized, &normalized.candidates, next),
        outcome_bits(context.scorer_margin, &margin.candidates, next),
        -witten_bell_probability(context.store, &code, next).log2(),
    ];
    let status_index = match rule12.witness.status {
        ScoreStatus::ExactContext => 0,
        ScoreStatus::Graph => 1,
        ScoreStatus::Novel => 2,
    };
    let contains = |candidates: &[(u32, ScoreQ)], token: u32| {
        candidates.iter().any(|&(candidate, _)| candidate == token)
    };
    let candidate_recall = [
        contains(&rule1.candidates, teacher_argmax),
        context.corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&rule1.candidates, token)),
        contains(&rule12.candidates, teacher_argmax),
        context.corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&rule12.candidates, token)),
    ];
    // Decompose the score. `transition_offset` is common to all candidates and
    // cannot affect ordering, so the question is whether the per-token residual
    // moves the argmax away from what the root prior alone would choose.
    let (mut root_best, mut root_best_score) = (u32::MAX, i64::MIN);
    let (mut root_lo, mut root_hi) = (i64::MAX, i64::MIN);
    let (mut resid_lo, mut resid_hi) = (i64::MAX, i64::MIN);
    for &(token, root_raw, resid_raw, penalized) in &rule12.candidate_components {
        let root_term = i64::from(root_raw) + if penalized { -2_000_000 } else { 0 };
        if root_term > root_best_score {
            root_best_score = root_term;
            root_best = token;
        }
        root_lo = root_lo.min(i64::from(root_raw));
        root_hi = root_hi.max(i64::from(root_raw));
        resid_lo = resid_lo.min(i64::from(resid_raw));
        resid_hi = resid_hi.max(i64::from(resid_raw));
    }
    // bits/token with the residual suppressed. Top-1 and bits are different
    // questions: a change can correct the argmax and leave the distribution as
    // wrong as before, and coherence against the teacher is distributional.
    // Same softmax and same uncovered-vocab floor as the shipped path, only the
    // scores differ.
    let root_only_candidates: Vec<(u32, ScoreQ)> = rule12
        .candidate_components
        .iter()
        .map(|&(token, root_raw, _, penalized)| {
            let raw = root_raw + if penalized { -2_000_000 } else { 0 };
            (token, ScoreQ::from_raw(raw))
        })
        .collect();
    let bits_root_only = outcome_bits(context.scorer_with_exct, &root_only_candidates, next);

    // Residual accrues only on selected-chain nodes; tokens emitted by active
    // or predicted nodes keep residual exactly ZERO. Since residuals are
    // log-prob-like and negative, chain tokens carry a summed penalty while
    // off-chain tokens sit at zero, and both are ranked by root + residual.
    // Measure whether that is what decides graph-slice selections.
    // Chain selection ranks by chain LENGTH first, with membership margin only
    // breaking ties among equal-length chains. So a deep chain the context
    // barely belongs to beats a shallow one it fits well. Candidate generation
    // meanwhile draws on every active/predicted node. Measure whether a
    // better-fitting region outside the chain could have supplied the teacher.
    let chain_nodes: std::collections::BTreeSet<u32> =
        rule12.witness.chain.iter().copied().collect();
    let chain_margin = rule12
        .witness
        .active
        .iter()
        .filter(|a| chain_nodes.contains(&(a.region + 1)))
        .map(|a| a.margin)
        .max()
        .unwrap_or(i16::MIN);
    let chain_depth = rule12.witness.chain.len() as u32;
    let mut teacher_emitted_off_chain = false;
    let mut teacher_emitter_better_margin = false;
    let mut teacher_emitter_depth = 0u32;
    let mut best_emitter_margin = i16::MIN;
    for a in &rule12.witness.active {
        let node = a.region + 1;
        if chain_nodes.contains(&node) {
            continue;
        }
        if context.scorer_with_exct.node_emits(node, teacher_argmax) {
            teacher_emitted_off_chain = true;
            if a.margin > best_emitter_margin {
                best_emitter_margin = a.margin;
                teacher_emitter_depth = u32::from(a.depth);
            }
            if a.margin > chain_margin {
                teacher_emitter_better_margin = true;
            }
        }
    }

    // Candidate recall was 63% on the graph slice while active regions emit the
    // teacher only ~4.3% of the time. Attribute the rest: the candidate set is
    // active + predicted + root_top, and which source supplies the teacher
    // decides whether the graph contributes anything to finding the answer.
    // Own-region versus unrelated-region control. The unrelated node is chosen
    // by a fixed integer hash of the position index -- deterministic, no RNG,
    // and independent of the context's geometry.
    let node_count = context.scorer_with_exct.emission_node_count();
    let own_region_emits_teacher = rule12
        .witness
        .chain
        .last()
        .is_some_and(|&node| context.scorer_with_exct.node_emits(node, teacher_argmax));
    let random_region_emits_teacher = if node_count == 0 {
        false
    } else {
        let mixed = (position as u64).wrapping_mul(2_654_435_761) >> 16;
        let node = (mixed % node_count as u64) as u32 + 1;
        context.scorer_with_exct.node_emits(node, teacher_argmax)
    };
    // Telescoping integrity: how much of the selected chain actually carries a
    // term for the teacher token.
    let chain_levels = rule12.witness.chain.len() as u32;
    let chain_levels_emitting_teacher = rule12
        .witness
        .chain
        .iter()
        .filter(|&&node| context.scorer_with_exct.node_emits(node, teacher_argmax))
        .count() as u32;
    let teacher_chain_complete = chain_levels > 0 && chain_levels_emitting_teacher == chain_levels;
    let teacher_chain_partial =
        chain_levels_emitting_teacher > 0 && chain_levels_emitting_teacher < chain_levels;

    let teacher_present = rule12
        .candidate_components
        .iter()
        .any(|&(token, _, _, _)| token == teacher_argmax);
    let teacher_from_active = rule12.witness.active.iter().any(|a| {
        context
            .scorer_with_exct
            .node_emits(a.region + 1, teacher_argmax)
    });
    let teacher_from_predicted = rule12
        .witness
        .predicted
        .iter()
        .any(|&node| context.scorer_with_exct.node_emits(node, teacher_argmax));
    let teacher_from_root_top = context.scorer_with_exct.root_top_contains(teacher_argmax);
    let teacher_only_root_top =
        teacher_present && teacher_from_root_top && !teacher_from_active && !teacher_from_predicted;

    let total_candidates = rule12.candidate_components.len().max(1) as f64;
    let zeros = rule12
        .candidate_components
        .iter()
        .filter(|&&(_, _, resid, _)| resid == 0)
        .count();
    let zero_resid_share = zeros as f64 / total_candidates;
    let resid_of = |want: u32| {
        rule12
            .candidate_components
            .iter()
            .find(|&&(token, _, _, _)| token == want)
            .map(|&(_, _, resid, _)| resid)
    };
    let teacher_on_chain = resid_of(teacher_argmax).is_some_and(|r| r != 0);
    let selected_off_chain = resid_of(rule12.selected) == Some(0);

    let mut alpha_hits = [false; ALPHA_SWEEP.len()];
    for (slot, &(num, den)) in alpha_hits.iter_mut().zip(ALPHA_SWEEP.iter()) {
        let mut best_token = u32::MAX;
        let mut best_score = i64::MIN;
        for &(token, root_raw, resid_raw, penalized) in &rule12.candidate_components {
            let scaled = i64::from(resid_raw) * num / den;
            let score = i64::from(root_raw) + scaled + if penalized { -2_000_000 } else { 0 };
            if score > best_score {
                best_score = score;
                best_token = token;
            }
        }
        *slot = best_token == teacher_argmax;
    }
    let root_only_agrees = root_best == rule12.selected;
    let root_only_hit = root_best == teacher_argmax;
    let root_spread = if root_hi >= root_lo {
        root_hi - root_lo
    } else {
        0
    };
    let residual_spread = if resid_hi >= resid_lo {
        resid_hi - resid_lo
    } else {
        0
    };

    // Rank the teacher argmax exactly as selection does: score descending,
    // ties to the lower token id. `rule12.candidates` already carries the
    // final scores (root + transition offset + residual + repetition
    // penalty), so this ordering reproduces the argmax.
    let teacher_rank = rule12
        .candidates
        .iter()
        .find(|&&(token, _)| token == teacher_argmax)
        .map(|&(_, teacher_score)| {
            let ahead = rule12
                .candidates
                .iter()
                .filter(|&&(token, score)| {
                    score > teacher_score || (score == teacher_score && token < teacher_argmax)
                })
                .count();
            (ahead + 1) as u32
        });
    let exct_level = rule12
        .witness
        .exct
        .as_ref()
        .map(|probe| probe.level as usize);
    let witness_replay_failed = index < context.config.witness_sample
        && verify_witness_replay(
            context.r4g1,
            Some(context.artifact_container),
            &rule12.witness,
            context.config.root_top_b,
            context.config.exct_top_x,
        )
        .is_err();

    Ok(GateCRow {
        hits,
        bits,
        status_index,
        status_hit: hits[2],
        status_bits: bits[2],
        exct_level,
        exct_full_depth_supported: exct_level == Some(STAGES)
            && rule12.witness.status == ScoreStatus::ExactContext,
        candidate_recall,
        teacher_rank,
        root_only_agrees,
        root_only_hit,
        root_spread,
        residual_spread,
        alpha_hits,
        bits_root_only,
        zero_resid_share,
        teacher_on_chain,
        selected_off_chain,
        teacher_emitted_off_chain,
        teacher_emitter_better_margin,
        chain_depth,
        teacher_emitter_depth,
        teacher_from_active,
        teacher_from_predicted,
        teacher_from_root_top,
        teacher_only_root_top,
        chain_levels,
        chain_levels_emitting_teacher,
        teacher_chain_complete,
        teacher_chain_partial,
        own_region_emits_teacher,
        random_region_emits_teacher,
        witness_replayed: index < context.config.witness_sample,
        witness_replay_failed,
    })
}

/// Gate C table (graph_no_exct/graph_with_exct/tla3_baseline); 2 = the
/// issue-#64 four-set table (legacy_sum / rule1_chain /
/// rule12_precedence / tla3_baseline) with status counts, per-status
/// metrics, win/loss breakdowns, and the EXCT support gate in the config;
/// 3 = issue-#67 smoothing calibration: `config.smoothing` records the
/// compiled emission rule and `quantization.smoothing` describes it; 4 =
/// issue-#79 repetition telemetry in `graph` (graph/baseline repetition
/// rates from the deterministic greedy probe); 5 = issue-#80 rejected
/// candidate-variant rows in `gate_c` (`rule12_cloud_size_normalized`,
/// `rule12_margin_weighted`); 6 = issue-#102 removes those rows: the
/// variants were zero-information (bit-identical to `rule12_precedence`
/// on every measured corpus, where ExactContext precedence dominates); 7 =
/// explicit quality-gate profile for distribution-aware validation; 8 =
/// per-residual-kind compile-time quantization error rows; 9 =
/// issue-#234 `distribution` declaration (EXCT-miss rate as a fixed
/// property of the evaluation distribution, with the Gate C validity
/// verdict); 10 = issue-#234 item 2: the status-based miss rate is
/// structurally ~0 (the probe backs off to populated prefixes, root
/// included), so the declaration adds the probe-level histogram and
/// the STRICT full-code miss rate, and the Gate C validity verdict is
/// judged on the strict basis.
/// 11 = graph footprint and quality-profile fields; 12 = packed FMM
/// footprint, rank, and candidate-count fields.
#[derive(Debug, Clone, Serialize)]
pub struct ScoreReport {
    pub schema: u32,
    pub inputs: ScoreReportInputs,
    pub config: ScoreReportConfig,
    pub graph: ScoreReportGraph,
    pub gate_c: GateCOutcome,
    pub quantization: ScoreReportQuantization,
    pub determinism: ScoreReportDeterminism,
    pub distribution: DistributionDeclaration,
}

/// Minimum EXCT-miss rate below which a distribution cannot serve as a
/// Gate C corpus (issue #234): with (almost) every held-out probe
/// answered by exact-context lookup, the routing/residual machinery is
/// never exercised and Gate C restates the baseline by construction.
pub const MIN_EXCT_MISS_RATE_FOR_GATE_C: f64 = 0.05;

/// The issue-#234 evaluation-distribution declaration, fixed at scoring
/// time as a property of the corpus/split pair: how much of the
/// held-out set escapes exact-context resolution under the Rule 1+2
/// scorer. Reported in every score report so the failure mode is
/// visible in the artifact of record rather than discoverable only by
/// reading the risk register.
#[derive(Debug, Clone, Serialize)]
pub struct DistributionDeclaration {
    /// Held-out positions the Gate C evaluation scored.
    pub held_out_positions: usize,
    /// Positions the Rule 1+2 scorer resolved as ExactContext.
    pub exct_resolved_positions: usize,
    /// `1 - exct_resolved/held_out` — the status-based miss rate.
    /// STRUCTURALLY ~0 on every corpus: the Rule 2 probe stops at the
    /// deepest POPULATED graded prefix and the level-0 (root) prefix is
    /// always populated with support, so ExactContext status cannot
    /// miss. Kept for schema continuity; the strict fields below are
    /// the #234 item 2 construction target.
    pub exct_miss_rate: f64,
    /// Positions the probe resolved at the FULL graded code with
    /// support — exact context in the strict sense.
    pub strict_exct_resolved_positions: usize,
    /// `1 - strict_resolved/held_out` — the miss rate under strict
    /// full-code exact-context semantics. This is the declared quantity
    /// a D3 construction must move (issue #234 item 2); sub-full-depth
    /// probe resolutions are prefix backoff wearing the EXCT badge, not
    /// exact-context recall.
    pub strict_exct_miss_rate: f64,
    /// Histogram of probe resolution levels (index = prefix length,
    /// 0 = root … STAGES = full code), fixed at declaration time.
    pub exct_probe_level_histogram: Vec<usize>,
    /// The [`MIN_EXCT_MISS_RATE_FOR_GATE_C`] threshold in force.
    pub min_miss_rate_for_gate_c: f64,
    /// Whether Gate C on this distribution measures anything beyond
    /// exact-context recall — judged on the STRICT miss rate (the
    /// status-based rate is structurally uninformative, see above).
    pub can_measure_generalization: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    pub cover_source: String,
    pub graph_kappa: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportConfig {
    pub transition_out_degree: usize,
    pub emission_entries: usize,
    pub root_top_b: usize,
    pub exct_top_x: usize,
    pub witness_sample: usize,
    pub top_m: usize,
    /// The D4 EXCT precedence support gate (`score_runtime::EXCT_SUPPORT_MIN`).
    pub exct_support_min: u32,
    /// The calibrated emission smoothing rule (issue #67;
    /// [`Smoothing::label`]).
    pub smoothing: String,
    /// Quality-gate basis for this distribution. `pinned` applies the
    /// historical Gate C absolute floor; `relative_tla` only compares the
    /// graph with the TLA baseline measured on the same corpus.
    pub quality_profile: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportGraph {
    pub node_count: u32,
    pub edge_count: u32,
    pub refinement_edges: u32,
    pub neighbor_edges: u32,
    pub forward_edges: u32,
    pub depth_count: u8,
    pub root_prior_entries: u32,
    pub emission_list_entries: u32,
    pub exct_bytes: u32,
    pub context_row_count: u32,
    pub context_entry_count: u32,
    pub context_bytes: u32,
    pub fmm_bytes: u32,
    pub fmm_rank: u16,
    pub fmm_candidate_count: u32,
    pub artifact_bytes: usize,
    pub graph_repetition_rate: f64,
    pub baseline_repetition_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportQuantization {
    pub format: String,
    pub smoothing: String,
    pub platform: String,
    pub residual_kind_errors: Vec<ScoreReportResidualQuantization>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportResidualQuantization {
    pub kind: String,
    pub cid: String,
    pub sample_count: u64,
    pub max_abs_error_nats: f64,
    pub mean_abs_error_nats: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreReportDeterminism {
    pub note: String,
}

/// Assemble the report from a finished run.
pub fn build_score_report(
    config: &ScoreConfig,
    inputs: ScoreReportInputs,
    info: &ScoredGraphInfo,
    gate_c: GateCOutcome,
) -> ScoreReport {
    build_score_report_with_quality_profile(config, inputs, info, gate_c, "pinned")
}

/// Assemble a report while declaring which quality baseline applies to its
/// distribution. The legacy builder above keeps fixture and library callers
/// on the pinned profile; dynamic teacher builds can opt into a same-corpus
/// TLA comparison explicitly.
pub fn build_score_report_with_quality_profile(
    config: &ScoreConfig,
    inputs: ScoreReportInputs,
    info: &ScoredGraphInfo,
    gate_c: GateCOutcome,
    quality_profile: &str,
) -> ScoreReport {
    let graph_kappa = inputs.graph_kappa.clone();
    let held_out_positions = gate_c.rule12_precedence.positions;
    let exct_resolved_positions = gate_c.rule12_status_counts.exact_context;
    let exct_miss_rate = if held_out_positions == 0 {
        0.0
    } else {
        1.0 - exct_resolved_positions as f64 / held_out_positions as f64
    };
    let strict_exct_resolved_positions = gate_c.rule12_exct_full_depth_supported;
    let strict_exct_miss_rate = if held_out_positions == 0 {
        0.0
    } else {
        1.0 - strict_exct_resolved_positions as f64 / held_out_positions as f64
    };
    let distribution = DistributionDeclaration {
        held_out_positions,
        exct_resolved_positions,
        exct_miss_rate,
        strict_exct_resolved_positions,
        strict_exct_miss_rate,
        exct_probe_level_histogram: gate_c.rule12_exct_probe_levels.clone(),
        min_miss_rate_for_gate_c: MIN_EXCT_MISS_RATE_FOR_GATE_C,
        can_measure_generalization: strict_exct_miss_rate >= MIN_EXCT_MISS_RATE_FOR_GATE_C,
    };
    ScoreReport {
        schema: 12,
        inputs,
        config: ScoreReportConfig {
            transition_out_degree: config.transition_out_degree,
            emission_entries: config.emission_entries,
            root_top_b: config.root_top_b,
            exct_top_x: config.exct_top_x,
            witness_sample: config.witness_sample,
            top_m: super::score_runtime::TOP_M,
            exct_support_min: super::score_runtime::EXCT_SUPPORT_MIN,
            smoothing: config.smoothing.label(),
            quality_profile: quality_profile.to_owned(),
        },
        graph: ScoreReportGraph {
            node_count: info.node_count,
            edge_count: info.edge_count,
            refinement_edges: info.refinement_edges,
            neighbor_edges: info.neighbor_edges,
            forward_edges: info.forward_edges,
            depth_count: info.depth_count,
            root_prior_entries: info.root_prior_entries,
            emission_list_entries: info.emission_list_entries,
            exct_bytes: info.exct_bytes,
            context_row_count: info.context_row_count,
            context_entry_count: info.context_entry_count,
            context_bytes: info.context_bytes,
            fmm_bytes: info.fmm_bytes,
            fmm_rank: info.fmm_rank,
            fmm_candidate_count: info.fmm_candidate_count,
            artifact_bytes: info.artifact_bytes,
            graph_repetition_rate: gate_c.repetition_rate_rule12,
            baseline_repetition_rate: gate_c.repetition_rate_baseline,
        },
        gate_c,
        distribution,
        quantization: ScoreReportQuantization {
            format: "ScoreQ Q16.16 in i32; EMIT storage descriptor {width: i32, shift: 0, \
                     zero_point: 0}; edge weights and residuals via ScoreQ::from_logprob"
                .to_owned(),
            smoothing: smoothing_description(config.smoothing),
            platform: "compiler-side f64 ln quantization is macOS-pinned (libm-sensitive \
                       cross-platform), the same status as the existing κ baseline; the D2 \
                       canonical deterministic compile mode resolves cross-platform byte \
                       equality later. RX1 EXCT residuals are quantized at compile time, so \
                       the deployed scoring path is integer-only; raw TLS1 is legacy-only"
                .to_owned(),
            residual_kind_errors: vec![
                ScoreReportResidualQuantization {
                    kind: "transition".to_owned(),
                    cid: graph_kappa.clone(),
                    sample_count: info.transition_quantization.sample_count,
                    max_abs_error_nats: nano_to_nats(
                        info.transition_quantization.max_abs_error_nano,
                    ),
                    mean_abs_error_nats: nano_to_nats(
                        info.transition_quantization.mean_abs_error_nano(),
                    ),
                },
                ScoreReportResidualQuantization {
                    kind: "root_prior".to_owned(),
                    cid: graph_kappa.clone(),
                    sample_count: info.root_prior_quantization.sample_count,
                    max_abs_error_nats: nano_to_nats(
                        info.root_prior_quantization.max_abs_error_nano,
                    ),
                    mean_abs_error_nats: nano_to_nats(
                        info.root_prior_quantization.mean_abs_error_nano(),
                    ),
                },
                ScoreReportResidualQuantization {
                    kind: "emission".to_owned(),
                    cid: graph_kappa.clone(),
                    sample_count: info.emission_quantization.sample_count,
                    max_abs_error_nats: nano_to_nats(info.emission_quantization.max_abs_error_nano),
                    mean_abs_error_nats: nano_to_nats(
                        info.emission_quantization.mean_abs_error_nano(),
                    ),
                },
                ScoreReportResidualQuantization {
                    kind: "exact_context".to_owned(),
                    cid: graph_kappa,
                    sample_count: info.exact_context_quantization.sample_count,
                    max_abs_error_nats: nano_to_nats(
                        info.exact_context_quantization.max_abs_error_nano,
                    ),
                    mean_abs_error_nats: nano_to_nats(
                        info.exact_context_quantization.mean_abs_error_nano(),
                    ),
                },
            ],
        },
        determinism: ScoreReportDeterminism {
            note: "content-addressed observation order; all reductions are B-tree (ordered) \
                   accumulations over counts, so shard/observation order never reaches the \
                   bytes; canonical sorts everywhere; identical inputs produce byte-identical \
                   artifacts and reports"
                .to_owned(),
        },
    }
}

fn nano_to_nats(nano: u64) -> f64 {
    nano as f64 / 1_000_000_000.0
}

/// The `quantization.smoothing` prose for the compiled rule. The
/// add-one text is the pre-#67 wording verbatim, so default reports
/// stay byte-identical.
fn smoothing_description(smoothing: Smoothing) -> String {
    let evidence = "evidence = the store's top-3 teacher-weighted counts \
                    over covered binary top-1 members (within calibrated radius; no \
                    backoff-floor assignment); root prior = level-0 store distribution; \
                    smoothing floor baked into the EMIT root header";
    match smoothing {
        Smoothing::AddOne => format!(
            "add-one over the compiled vocabulary: P(v|n) = (count_n(v) + 1) / \
             (total_n + V); {evidence}"
        ),
        Smoothing::WittenBell => format!(
            "Witten-Bell over the compiled vocabulary: seen P(v|n) = count_n(v) / \
             (total_n + T_n), floor mass T_n / (total_n + T_n) spread over the \
             max(V − T_n, 1) unseen types (T_n = seen types) — the depth-0 \
             specialization of the store's backoff chain; {evidence}"
        ),
        Smoothing::AbsoluteDiscount(delta) => format!(
            "absolute discounting (δ = {delta}) over the compiled vocabulary: seen \
             P(v|n) = (count_n(v) − δ) / total_n for count_n(v) > δ, floor mass \
             δ·T_n / total_n spread over the max(V − T_n, 1) unseen types \
             (T_n = seen types); {evidence}"
        ),
    }
}

#[cfg(test)]
mod context_rows_tests {
    use super::{compile_context_rows, Smoothing};
    use uor_r4_core::transformerless::compiler::{Corpus, SIG_BYTES};
    use uor_r4_graph_compiler::induction::Observation;

    fn corpus() -> Corpus {
        Corpus {
            n: 5,
            stories: 2,
            story: vec![0, 0, 1, 1, 1],
            input: vec![10, 20, 30, 40, 50],
            next: vec![20, 30, 40, 50, 60],
            t_argmax: vec![0; 5],
            top_tokens: vec![[0; 8]; 5],
            top_weights: vec![[0; 8]; 5],
            span_start: vec![0; 5],
            span_end: vec![0; 5],
            byte_start: vec![0; 5],
            byte_end: vec![0; 5],
            hidden: None,
        }
    }

    #[test]
    fn context_rows_do_not_cross_story_boundaries() {
        let corpus = corpus();
        let observations: Vec<Observation> = (0..corpus.n)
            .map(|position| Observation {
                position: position as u32,
                sample: [0; 32],
                vector: Vec::new(),
                sig: [0; SIG_BYTES],
                prev: corpus.input[position],
                next: corpus.next[position],
            })
            .collect();
        let rows = compile_context_rows(&corpus, &observations, 100, Smoothing::AddOne);
        assert!(rows
            .iter()
            .any(|row| { row.context_len == 2 && row.key0 == 10 && row.key1 == 20 }));
        assert!(rows
            .iter()
            .any(|row| { row.context_len == 2 && row.key0 == 40 && row.key1 == 50 }));
        assert!(!rows
            .iter()
            .any(|row| { row.context_len == 2 && row.key0 == 20 && row.key1 == 30 }));
    }
}

#[cfg(test)]
mod emission_shrinkage_tests {
    use super::witten_bell_lambda;

    #[test]
    fn witten_bell_weight_is_bounded_and_evidence_sensitive() {
        assert_eq!(witten_bell_lambda(0, 0), 0.0);
        assert!((witten_bell_lambda(100, 10) - 100.0 / 110.0).abs() < 1e-12);
        assert!(witten_bell_lambda(100, 10) > witten_bell_lambda(10, 100));
        assert!(witten_bell_lambda(100, 10) < 1.0);
    }
}
