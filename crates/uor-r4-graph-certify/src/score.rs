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
/// Default root-prior candidate count.
pub const DEFAULT_ROOT_TOP_B: usize = 64;
/// Default EXCT candidate count.
pub const DEFAULT_EXCT_TOP_X: usize = 64;
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
    let mut k = runtime::OpKernel::default();
    // Canonical observation order (content-addressed positions, §4.1):
    // the caller's slice order never reaches the counts, so a shuffled
    // observation/shard order compiles to identical edges.
    let mut ordered: Vec<&Observation> = train.iter().collect();
    ordered.sort_by_key(|o| o.position);
    // memberships[depth] of the previous adjacent position, carried.
    let mut previous: Option<(u32, Vec<Vec<u32>>)> = None;
    let mut counts: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    let mut src_totals: BTreeMap<u32, u64> = BTreeMap::new();
    for observation in ordered {
        let position = observation.position;
        let mut memberships: Vec<Vec<u32>> = Vec::with_capacity(max_depth);
        for depth in 1..=max_depth {
            let at_depth: Vec<u32> =
                binary_memberships(&mut k, &pop, regions, depth, &observation.sig)
                    .into_iter()
                    .map(|(region, _)| region)
                    .collect();
            memberships.push(at_depth);
        }
        if let Some((prev_position, prev_memberships)) = previous {
            let adjacent = position == prev_position + 1
                && corpus.story[position as usize] == corpus.story[prev_position as usize];
            if adjacent {
                for depth in 0..max_depth {
                    for &src in &prev_memberships[depth] {
                        for &dst in &memberships[depth] {
                            let edge = (src + 1, dst + 1); // region -> node id
                            *counts.entry(edge).or_insert(0) += 1;
                            *src_totals.entry(edge.0).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        previous = Some((position, memberships));
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

/// ln of an add-one-smoothed probability (compiler-side f64; module
/// docs for the platform pinning). This is the [`Smoothing::AddOne`]
/// arm; the other rules live in [`Smoothing::ln_prob`].
fn smoothed_ln(count: u64, total: u64, vocab: u32) -> f32 {
    ((count as f64 + 1.0) / (total as f64 + f64::from(vocab))).ln() as f32
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
    let mut k = runtime::OpKernel::default();
    let smoothing = config.smoothing;
    // Weighted evidence per region: the covered binary top-1 membership
    // at each depth (within the calibrated radius — the backoff floor is
    // a routing behavior, never region content; see `binary_top1_covered`).
    let mut evidence: Vec<BTreeMap<u32, u64>> = vec![BTreeMap::new(); regions.len()];
    for observation in train {
        let i = observation.position as usize;
        for depth in 1..=max_depth {
            let Some((top1, _)) =
                binary_top1_covered(&mut k, &pop, regions, depth, &observation.sig)
            else {
                continue;
            };
            let dist = &mut evidence[top1 as usize];
            for k_idx in 0..corpus.top_tokens[i].len() {
                let token = corpus.top_tokens[i][k_idx];
                let weight = corpus.top_weights[i][k_idx];
                if weight > 0 {
                    *dist.entry(token).or_insert(0) += u64::from(weight);
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

    let mut region_lists = Vec::with_capacity(regions.len());
    let mut emission_quantization = QuantizationErrorStats::default();
    for (region_id, region) in regions.iter().enumerate() {
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
                let score = ScoreQ::from_logprob(ln);
                emission_quantization.record_ln_quantization(ln, score);
                (token, score)
            })
            .collect();
        // Top-E by residual score (score desc, token asc), stored
        // ascending token.
        residuals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        residuals.truncate(config.emission_entries);
        residuals.sort_by_key(|&(token, _)| token);
        region_lists.push(residuals);
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
    /// TLS1 container bytes used as compiler input for residualized EXCT.
    pub exct_tls1: &'a [u8],
    /// Number of exact-context residual entries retained per prefix.
    pub exct_top_x: usize,
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
        exct_tls1,
        exct_top_x,
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
    let gate_rotations = compiler::derive_rotations();
    for (index, observation) in held_out.iter().enumerate() {
        let position = observation.position as usize;
        let teacher_argmax = corpus.t_argmax[position];
        let next = corpus.next[position];
        // #243 Phase C option A: one metric-respecting code per position
        // (corpus bundle → assign_for_bundle), consumed consistently by
        // the WB/store baseline and attested to every coded scorer call.
        // The legacy Σ-over-cloud row intentionally keeps its own old
        // semantics for comparison.
        let code = runtime::code_plain(artifacts, &gate_rotations, corpus, position);

        let legacy = scorer_with_exct.score_candidates_legacy(&observation.sig)?;
        let rule1 = scorer_no_exct.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let rule12 = scorer_with_exct.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let rule1_no_f =
            scorer_no_exct_no_f.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let rule12_no_f =
            scorer_with_exct_no_f.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let normalized =
            scorer_normalized.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let margin = scorer_margin.score_candidates_coded(&observation.sig, Some(&code), &[])?;
        let baseline = runtime::predict_witness_plain(store, &code);

        let legacy_hit = legacy.selected == teacher_argmax;
        let rule1_hit = rule1.selected == teacher_argmax;
        let rule12_hit = rule12.selected == teacher_argmax;
        let rule1_no_f_hit = rule1_no_f.selected == teacher_argmax;
        let rule12_no_f_hit = rule12_no_f.selected == teacher_argmax;
        let normalized_hit = normalized.selected == teacher_argmax;
        let margin_hit = margin.selected == teacher_argmax;
        let baseline_hit = baseline.token == teacher_argmax;
        hits_legacy += u64::from(legacy_hit);
        hits_rule1 += u64::from(rule1_hit);
        hits_rule12 += u64::from(rule12_hit);
        hits_rule1_no_f += u64::from(rule1_no_f_hit);
        hits_rule12_no_f += u64::from(rule12_no_f_hit);
        hits_normalized += u64::from(normalized_hit);
        hits_margin += u64::from(margin_hit);
        hits_baseline += u64::from(baseline_hit);
        let legacy_bits = outcome_bits(&scorer_with_exct, &legacy.candidates, next);
        let rule1_bits = outcome_bits(&scorer_no_exct, &rule1.candidates, next);
        let rule12_bits = outcome_bits(&scorer_with_exct, &rule12.candidates, next);
        let rule1_no_f_bits = outcome_bits(&scorer_no_exct_no_f, &rule1_no_f.candidates, next);
        let rule12_no_f_bits = outcome_bits(&scorer_with_exct_no_f, &rule12_no_f.candidates, next);
        let normalized_bits = outcome_bits(&scorer_normalized, &normalized.candidates, next);
        let margin_bits = outcome_bits(&scorer_margin, &margin.candidates, next);
        bits_legacy += legacy_bits;
        bits_rule1 += rule1_bits;
        bits_rule12 += rule12_bits;
        bits_rule1_no_f += rule1_no_f_bits;
        bits_rule12_no_f += rule12_no_f_bits;
        bits_normalized += normalized_bits;
        bits_margin += margin_bits;
        bits_baseline += -witten_bell_probability(store, &code, next).log2();

        let status_index = match rule12.witness.status {
            ScoreStatus::ExactContext => 0,
            ScoreStatus::Graph => 1,
            ScoreStatus::Novel => 2,
        };
        status_positions[status_index] += 1;
        status_hits[status_index] += u64::from(rule12_hit);
        status_bits[status_index] += rule12_bits;
        match &rule12.witness.exct {
            Some(probe) => {
                exct_level_positions[probe.level as usize] += 1;
                if probe.level as usize == STAGES
                    && rule12.witness.status == ScoreStatus::ExactContext
                {
                    exct_full_depth_supported += 1;
                }
            }
            None => exct_probe_absent += 1,
        }

        let contains = |candidates: &[(u32, ScoreQ)], token: u32| {
            candidates.iter().any(|&(candidate, _)| candidate == token)
        };
        if contains(&rule1.candidates, teacher_argmax) {
            recall_rule1_top1 += 1;
        }
        if contains(&rule12.candidates, teacher_argmax) {
            recall_rule12_top1 += 1;
        }
        if corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&rule1.candidates, token))
        {
            recall_rule1_top3 += 1;
        }
        if corpus.top_tokens[position]
            .iter()
            .any(|&token| contains(&rule12.candidates, token))
        {
            recall_rule12_top3 += 1;
        }

        {
            let win_loss = &mut outcome.win_loss;
            accumulate_win_loss(&mut win_loss.rule12_vs_baseline, rule12_hit, baseline_hit);
            accumulate_win_loss(&mut win_loss.rule12_vs_legacy, rule12_hit, legacy_hit);
            accumulate_win_loss(&mut win_loss.rule1_vs_baseline, rule1_hit, baseline_hit);
        }

        if index < config.witness_sample {
            outcome.witness_replays += 1;
            if verify_witness_replay(
                r4g1,
                Some(artifact_container),
                &rule12.witness,
                config.root_top_b,
                config.exct_top_x,
            )
            .is_err()
            {
                outcome.witness_replay_failures += 1;
            }
        }
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
        schema: 10,
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
