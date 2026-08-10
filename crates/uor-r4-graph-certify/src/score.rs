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
//!
//! # Two-sided context arms (#446 M1) — NOT a generation number
//!
//! Gate C also carries `rule12_twosided`, an instrumentation arm that
//! keys evidence on the PAIR of the left graded code prefix and a right
//! graded code prefix built from the tokens AFTER the target, and lets
//! that table take D4-style precedence wherever it resolves with
//! support. Its falsifier `rule12_twosided_shuffled` runs the identical
//! machinery with a right key taken from a foreign held-out position, so
//! any gain that survives it is right-context INFORMATION rather than
//! key cardinality.
//!
//! The arms exist to attack graded-code DILUTION. The graded code space
//! is fixed at `STAGES` x 256, so as the corpus grows an ever larger
//! share of held-out positions resolves at the FULL graded code, every
//! full-code cell absorbs more records, its next-token distribution
//! dilutes, and top-1 falls even though key resolution is nominally
//! maximal. The right key multiplies key resolution and splits those
//! over-diluted cells. `rule12_twosided_exct_slice` reports the two arms
//! restricted to the positions Rule 1+2 resolved as ExactContext, and
//! `twosided_keys_per_full_left` reports how many distinct two-sided
//! keys the right context carves each full left code into.
//!
//! Two-sided conditioning is NOT causally available to left-to-right
//! generation: at generation time the tokens after the target do not
//! exist. These rows are an INFILL / ANALYSIS measurement (the A-mode
//! regime) or, prospectively, a CONSTRUCTION-time signal. They must
//! never be quoted as a generation number. The table is built inside
//! [`evaluate_gate_c`] from the construction split alone; the artifact
//! format, the serving scorer's default path, the witness and the replay
//! contract are all untouched.
//!
//! # Latent right-context mixture (#446 M2) — causally legitimate
//!
//! The M1 rows above cannot be quoted because the right key is read at
//! SERVING time. M2 keeps the right context as a LATENT variable that is
//! observed only during CONSTRUCTION and marginalized away at serving:
//!
//! In prose, the mixture is `P(next | left) = SUM over classes c of
//! P(c | left) P(next | left, c)`.
//!
//! where c is a coarse class of the right graded code. Construction (the
//! non-held-out positions, which may legitimately look both ways because
//! nothing is being predicted there) accumulates two tables: the
//! class-conditional emission counts N(next | left prefix, c) and the
//! class posterior counts N(c | left prefix). Serving reads the LEFT key
//! only: it forms P(c | left) from the posterior table and mixes the
//! class-conditional distributions. No token after the target is touched
//! at evaluation time, so `rule12_latent_mix` IS a generation-legitimate
//! number.
//!
//! Four arms are reported on the identical Gate C held-out population:
//! `rule12_latent_mix` (the M2 arm), the left-only `rule12_precedence`
//! baseline it must beat, `rule12_latent_oracle` (the SAME tables with
//! the TRUE right class supplied at evaluation time — an upper bound,
//! NOT causal, and never quotable), and `rule12_latent_shuffled` (the
//! falsifier: the class posterior is taken from a FOREIGN left key under
//! a fixed rotation, holding class cardinality, emission tables,
//! smoothing and backoff constant). Without the falsifier row the
//! mixture row is not interpretable.
//!
//! EXIT RULE, pre-declared. `rule12_latent_mix` is a POSITIVE result if
//! and only if it beats the left-only Rule 1+2 baseline by at least two
//! percentage points of top-1 agreement on the same population AND beats
//! the shuffled-class falsifier's top-1. Anything else is a negative
//! result and must be reported as one. Bits per token are reported for
//! every arm regardless of the top-1 verdict, and
//! `latent_headroom_fraction` reports where the mixture sits between the
//! baseline and the oracle as a fraction of the available top-1
//! headroom.

use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;

use super::score_runtime::{
    binary_memberships, binary_top1_covered, regions_from_view, structural_edges_from_view,
    verify_witness_replay, ExactContextSource, GraphScorer, RegionParams, ScoreOutcome,
    ScoreStatus, ScoringVariant, StructuralEdge, EDGE_KIND_FORWARD, EDGE_KIND_NEIGHBOR,
    EDGE_KIND_REFINEMENT, EXCT_SUPPORT_MIN, RESIDUAL_EXCT_MAGIC,
};
use uor_r4_core::transformerless::code_sidecar;
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

impl EmissionShrinkage {
    /// Report label; matches the CLI's `--emission-shrinkage` strings.
    pub fn label(self) -> String {
        match self {
            EmissionShrinkage::None => "none",
            EmissionShrinkage::WittenBell => "witten-bell",
            EmissionShrinkage::Contrast => "contrast",
        }
        .to_owned()
    }
}

impl EmissionSelection {
    /// Report label; matches the CLI's `--emission-selection` strings.
    pub fn label(self) -> String {
        match self {
            EmissionSelection::Ratio => "ratio",
            EmissionSelection::Probability => "probability",
        }
        .to_owned()
    }
}
/// One region's emission list plus the statistics gathered while building it.
type RegionEmissionResult = (
    Vec<(u32, ScoreQ)>,
    QuantizationErrorStats,
    EmissionSelectionStats,
);

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
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

impl EmissionSelectionStats {
    /// Per-region means from the accumulated per-region sums — the same
    /// normalization the stderr `[emission-selection]` line applies.
    /// `min_contrast`/`max_contrast` pass through; identity when
    /// `regions == 0`.
    pub fn normalized(&self) -> EmissionSelectionStats {
        if self.regions == 0 {
            return *self;
        }
        let n = self.regions as f64;
        EmissionSelectionStats {
            regions: self.regions,
            mean_lambda_witten_bell: self.mean_lambda_witten_bell / n,
            mean_contrast: self.mean_contrast / n,
            min_contrast: self.min_contrast,
            max_contrast: self.max_contrast,
            mean_region_count: self.mean_region_count / n,
            mean_region_types: self.mean_region_types / n,
            overlap_with_top_count: self.overlap_with_top_count / n,
            probability_mass_kept: self.probability_mass_kept / n,
        }
    }
}
/// Default root-prior candidate count.
pub const DEFAULT_ROOT_TOP_B: usize = 64;
/// Default EXCT candidate count.
pub const DEFAULT_EXCT_TOP_X: usize = 64;
/// Default per-context candidate bound for packed bigram/trigram rows.
pub const DEFAULT_CONTEXT_ENTRIES: usize = 64;
/// Default highest compiled lexical context order (bigram + trigram).
pub const DEFAULT_CONTEXT_ORDER: u8 = 2;
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
    ///
    /// Total: `None` means the value is not a valid smoothing spec (an
    /// unknown rule, a non-numeric δ, or a δ outside the finite (0, 1]
    /// range). The caller supplies the user-facing message (R5, #510).
    pub fn parse(value: &str) -> Option<Smoothing> {
        match value {
            "add-one" => Some(Smoothing::AddOne),
            "witten-bell" => Some(Smoothing::WittenBell),
            _ => {
                let delta = value.strip_prefix("abs-disc:")?;
                let delta: f64 = delta.parse().ok()?;
                (delta.is_finite() && delta > 0.0 && delta <= 1.0)
                    .then_some(Smoothing::AbsoluteDiscount(delta))
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
    /// Repetition-suppression penalty magnitude in raw ScoreQ units
    /// (#381; <= 0). Applied by the scorer when a candidate token appears
    /// in the recent-token window — which, on Gate C, only happens under
    /// the #375 window mode. Default is the shipped constant; the knob
    /// exists to SWEEP the measured ~10pp windowed top-1 cost, not to
    /// change serving (serving keeps the scorer default).
    pub repetition_penalty_raw: i32,
    /// Opt-in #375: Gate C rows evaluate with a real story-bounded
    /// recent-token window (up to the scorer's 32-token cap) instead of
    /// the historical empty window. With the window on, NGRAM context
    /// rows (#362) can fire (attributed via the #371 ngram/probe split)
    /// AND the repetition penalty engages — both are serving-path
    /// semantics, so window-on rows are closer to what serving does,
    /// but they are NOT comparable with window-off (historical) rows.
    /// Default off reproduces the historical rows byte-exactly.
    pub gate_c_context_window: bool,
    /// Highest explicit lexical context order compiled into NGRAM rows:
    /// 0 = no rows (geometric path serves everywhere), 1 = bigram only,
    /// 2 = bigram + trigram (shipped default). The #364/#362 A/B knob:
    /// rows take most-specific precedence over the geometric path at
    /// serving, so "trigram rows as the replacement for emission
    /// conditioning" vs "geometric everywhere" is a compile-time choice.
    /// Non-default values change artifact bytes and kappa by design.
    pub context_order: u8,
    /// Per-context candidate bound for packed bigram/trigram rows.
    pub context_entries: usize,
}

impl Default for ScoreConfig {
    /// The pinned defaults, with the capacity knobs each overridable by an
    /// `R4_*` environment variable for capacity measurements (#399/#393
    /// M-C2). Every override falls back to the pinned constant when unset,
    /// so default-config behavior is byte-identical (κ-neutral) unless a
    /// measurement explicitly opts in; a set-but-invalid value panics
    /// (see [`compiler::capacity_override_usize`]).
    fn default() -> Self {
        Self {
            transition_out_degree: compiler::capacity_override_usize(
                "R4_TRANSITION_OUT_DEGREE",
                DEFAULT_TRANSITION_OUT_DEGREE,
            ),
            emission_entries: compiler::capacity_override_usize(
                "R4_EMISSION_ENTRIES",
                DEFAULT_EMISSION_ENTRIES,
            ),
            root_top_b: compiler::capacity_override_usize("R4_ROOT_TOP_B", DEFAULT_ROOT_TOP_B),
            exct_top_x: compiler::capacity_override_usize("R4_EXCT_TOP_X", DEFAULT_EXCT_TOP_X),
            witness_sample: DEFAULT_WITNESS_SAMPLE,
            smoothing: Smoothing::AddOne,
            scoring_variant: ScoringVariant::ChainTelescoped,
            emission_selection: EmissionSelection::default(),
            emission_shrinkage: EmissionShrinkage::default(),
            context_order: DEFAULT_CONTEXT_ORDER,
            context_entries: compiler::capacity_override_usize(
                "R4_CONTEXT_ENTRIES",
                DEFAULT_CONTEXT_ENTRIES,
            ),
            gate_c_context_window: false,
            repetition_penalty_raw: super::score_runtime::DEFAULT_REPETITION_PENALTY_RAW,
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
    /// Per-region selection/contrast statistics gathered while building
    /// the emission lists (normalized per-region means; previously
    /// stderr-only, promoted so reports can attribute #364-era A/Bs).
    pub selection_stats: EmissionSelectionStats,
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
    config: &ScoreConfig,
) -> Vec<ContextRow> {
    let smoothing = config.smoothing;
    if config.context_order == 0 {
        return Vec::new();
    }
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
        if config.context_order >= 2 && i > 0 && corpus.story[i - 1] == corpus.story[i] {
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
            ranked.truncate(config.context_entries);
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

/// Per-row entry cap for the FWDA forward-anchor section: keep the
/// highest-count entries (canonical tie: lowest token) so a hot anchor
/// cannot blow up the section.
pub const FWDA_ENTRY_CAP: usize = 64;
/// Rows with less total evidence than this are not emitted (size bound;
/// a one-observation row carries no measurable signal).
pub const FWDA_MIN_TOTAL: u32 = 2;

/// One compiled forward-anchor row (issue #399): the raw count
/// distribution over the token emitted `distance` positions before an
/// anchor whose token is `anchor`. Counts stay raw on the wire — the
/// serving loader derives smoothed ScoreQ residuals from `total`, which
/// is the FULL pre-truncation evidence total (see the format crate's
/// `fwda` module docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardAnchorRow {
    /// Lookahead distance to the anchor (one through `M2_STRIDE` minus one).
    pub distance: u8,
    /// The anchor's emitted token.
    pub anchor: u32,
    /// Full evidence total before the entry cap was applied.
    pub total: u32,
    /// Bounded `(token, raw count)` entries, ascending token order.
    pub entries: Vec<(u32, u32)>,
}

/// Compile forward-anchor rows from the construction split — the same
/// loop the Gate C instrumentation uses to build its `fwd_table`
/// (issue #399 M2, the #394 infill protocol), run over the `train`
/// observations that also feed the store and the NGRAM context rows: a
/// position is an anchor when its EMITTED token lands on a
/// story-relative position that is a multiple of the stride, and each
/// anchor contributes its same-story construction-split predecessors at
/// every lookahead distance. Rows are canonical (ascending
/// `(distance, anchor)`, entries ascending token), capped at
/// [`FWDA_ENTRY_CAP`] entries keeping the highest counts (ties to the
/// lowest token), and dropped below [`FWDA_MIN_TOTAL`] total evidence.
pub fn compile_forward_anchor_rows(
    corpus: &Corpus,
    train: &[Observation],
) -> Vec<ForwardAnchorRow> {
    let mut in_train = vec![false; corpus.n];
    for observation in train {
        let i = observation.position as usize;
        if i < corpus.n && corpus.next[i] == observation.next {
            in_train[i] = true;
        }
    }
    let mut story_pos = Vec::with_capacity(corpus.n);
    {
        let (mut current_story, mut position_in_story) = (u32::MAX, 0u32);
        for &story in corpus.story.iter().take(corpus.n) {
            if story != current_story {
                current_story = story;
                position_in_story = 0;
            } else {
                position_in_story += 1;
            }
            story_pos.push(position_in_story);
        }
    }
    let mut counts: BTreeMap<(u8, u32), BTreeMap<u32, u32>> = BTreeMap::new();
    for j in 0..corpus.n {
        if !in_train[j] || !(story_pos[j] as usize + 1).is_multiple_of(M2_STRIDE) {
            continue;
        }
        for distance in 1..M2_STRIDE {
            if j >= distance
                && corpus.story[j - distance] == corpus.story[j]
                && in_train[j - distance]
            {
                *counts
                    .entry((distance as u8, corpus.next[j]))
                    .or_default()
                    .entry(corpus.next[j - distance])
                    .or_default() += 1;
            }
        }
    }
    // #399/#393 M-C2: the per-row entry cap is env-overridable for capacity
    // measurements; unset keeps the pinned constant (κ-neutral).
    let entry_cap = compiler::capacity_override_usize("R4_FWDA_ENTRY_CAP", FWDA_ENTRY_CAP);
    counts
        .into_iter()
        .filter_map(|((distance, anchor), distribution)| {
            let total: u64 = distribution.values().map(|&count| u64::from(count)).sum();
            let total = u32::try_from(total).unwrap_or(u32::MAX);
            if total < FWDA_MIN_TOTAL {
                return None;
            }
            let mut ranked: Vec<(u32, u32)> = distribution.into_iter().collect();
            ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            ranked.truncate(entry_cap);
            ranked.sort_by_key(|&(token, _)| token);
            Some(ForwardAnchorRow {
                distance,
                anchor,
                total,
                entries: ranked,
            })
        })
        .collect()
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
        selection_stats: selection_stats.normalized(),
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
///
/// Total verifier: `None` means the wiring holds; `Some(reason)` names the
/// first violation. The reverse index and forward ranges are built by
/// [`emit_scored_r4g1`] from its own edge array, so a `Some` here is a
/// self-produced-bytes defect the caller turns into a panic.
pub fn verify_theorem_7_wired(
    edges: &[WireEdge],
    reverse: &[u32],
    forward_start: &[u32],
    forward_len: &[u16],
) -> Option<String> {
    if reverse.len() != edges.len() {
        return Some("Theorem 7 violation: reverse index does not cover all edges".to_owned());
    }
    let mut seen = vec![false; edges.len()];
    for &id in reverse {
        let Some(slot) = seen.get_mut(id as usize) else {
            return Some(
                "Theorem 7 violation: invalid canonical edge ID in reverse index".to_owned(),
            );
        };
        if *slot {
            return Some("Theorem 7 violation: duplicate edge ID in reverse index".to_owned());
        }
        *slot = true;
    }
    if seen.iter().any(|s| !s) {
        return Some("Theorem 7 violation: reverse index is not a permutation".to_owned());
    }
    for pair in reverse.windows(2) {
        let a = edges[pair[0] as usize];
        let b = edges[pair[1] as usize];
        if (a.dst, a.src, a.kind) > (b.dst, b.src, b.kind) {
            return Some(
                "Theorem 7 violation: reverse index not sorted by (dst, src, kind)".to_owned(),
            );
        }
    }
    for (node, (&start, &len)) in forward_start.iter().zip(forward_len.iter()).enumerate() {
        let end = start as usize + len as usize;
        if end > reverse.len() {
            return Some("Theorem 7 violation: forward range out of bounds".to_owned());
        }
        for &id in &reverse[start as usize..end] {
            if edges[id as usize].dst != node as u32 {
                return Some(
                    "Theorem 7 violation: reverse range target mismatched node".to_owned(),
                );
            }
        }
    }
    None
}

/// What an [`emit_scored_r4g1`] call produced, for the report and tests.
/// (`Eq` dropped at schema 13: the promoted emission-selection statistics
/// are f64 means; comparisons remain exact via `PartialEq`.)
#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fwda_row_count: u32,
    pub fwda_bytes: u32,
    pub artifact_bytes: usize,
    pub transition_quantization: QuantizationErrorStats,
    pub root_prior_quantization: QuantizationErrorStats,
    pub emission_quantization: QuantizationErrorStats,
    pub exact_context_quantization: QuantizationErrorStats,
    /// Per-region emission selection/contrast statistics (normalized
    /// means), carried from [`EmissionTables::selection_stats`].
    pub emission_selection_stats: EmissionSelectionStats,
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
    /// Forward-anchor rows for the optional FWDA section (issue #399);
    /// empty means the section is not emitted and infill serving runs
    /// without the channel.
    pub fwd_rows: &'a [ForwardAnchorRow],
}

/// Encode compiled context (bigram/trigram) rows as the NGRAM section body.
///
/// Every failure branch is an invariant on rows this compiler produced —
/// size ceilings no real vocabulary reaches and canonical-sort/key
/// invariants the row builder upholds — so a violation is a self-produced
/// defect and this panics rather than surfacing a `Result` (R5, #510).
fn encode_context_rows(rows: &[ContextRow]) -> Vec<u8> {
    let row_count = u32::try_from(rows.len()).expect("NGRAM row count exceeds u32");
    let header_len = uor_r4_graph_format::NGRAM_HEADER_LEN;
    let row_len = uor_r4_graph_format::NGRAM_ROW_LEN;
    let entries_start = header_len
        .checked_add(
            row_len
                .checked_mul(rows.len())
                .expect("NGRAM row bytes overflow"),
        )
        .expect("NGRAM header bytes overflow");
    let mut bytes = Vec::with_capacity(entries_start);
    bytes.extend_from_slice(&uor_r4_graph_format::NGRAM_MAGIC);
    bytes.extend_from_slice(&uor_r4_graph_format::NGRAM_VERSION.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.extend_from_slice(&row_count.to_le_bytes());
    let max_entries = rows.iter().map(|row| row.entries.len()).max().unwrap_or(0);
    bytes.extend_from_slice(
        &u16::try_from(max_entries)
            .expect("NGRAM row entry count exceeds u16")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.resize(entries_start, 0);
    let mut entry_offset = entries_start;
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
        assert!(
            (1..=2).contains(&row.context_len) && !(row.context_len == 1 && row.key1 != 0),
            "NGRAM row has an invalid context key"
        );
        let key = (row.context_len, row.key0, row.key1);
        assert!(
            !previous.is_some_and(|last| last >= key),
            "NGRAM rows are not canonically sorted"
        );
        previous = Some(key);
        let entry_count =
            u16::try_from(row.entries.len()).expect("NGRAM row entry count exceeds u16");
        let header = header_len + index * row_len;
        bytes[header] = row.context_len;
        bytes[header + 2..header + 4].copy_from_slice(&entry_count.to_le_bytes());
        bytes[header + 4..header + 8].copy_from_slice(&row.key0.to_le_bytes());
        bytes[header + 8..header + 12].copy_from_slice(&row.key1.to_le_bytes());
        let entry_offset_u32 = u32::try_from(entry_offset).expect("NGRAM entry offset exceeds u32");
        bytes[header + 12..header + 16].copy_from_slice(&entry_offset_u32.to_le_bytes());
        let mut previous_token = None;
        for &(token, score) in &row.entries {
            assert!(
                !previous_token.is_some_and(|last| last >= token),
                "NGRAM entries are not canonically sorted"
            );
            previous_token = Some(token);
            bytes.extend_from_slice(&token.to_le_bytes());
            bytes.extend_from_slice(&score.raw().to_le_bytes());
        }
        entry_offset = bytes.len();
    }
    bytes
}

/// Encode compiled forward-anchor rows as the FWDA section body (same
/// canonical header/row/entry layout as NGRAM; raw counts on the wire,
/// the row total in the second key slot — format crate `fwda` docs).
fn encode_forward_anchor_rows(rows: &[ForwardAnchorRow]) -> Vec<u8> {
    let row_count = u32::try_from(rows.len()).expect("FWDA row count exceeds u32");
    let header_len = uor_r4_graph_format::FWDA_HEADER_LEN;
    let row_len = uor_r4_graph_format::FWDA_ROW_LEN;
    let entries_start = header_len
        .checked_add(
            row_len
                .checked_mul(rows.len())
                .expect("FWDA row bytes overflow"),
        )
        .expect("FWDA header bytes overflow");
    let mut bytes = Vec::with_capacity(entries_start);
    bytes.extend_from_slice(&uor_r4_graph_format::FWDA_MAGIC);
    bytes.extend_from_slice(&uor_r4_graph_format::FWDA_VERSION.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.extend_from_slice(&row_count.to_le_bytes());
    let max_entries = rows.iter().map(|row| row.entries.len()).max().unwrap_or(0);
    bytes.extend_from_slice(
        &u16::try_from(max_entries)
            .expect("FWDA row entry count exceeds u16")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&[0u8; 2]);
    bytes.resize(entries_start, 0);
    let mut entry_offset = entries_start;
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
        assert!(
            (1..=uor_r4_graph_format::FWDA_MAX_DISTANCE).contains(&row.distance)
                && row.total >= FWDA_MIN_TOTAL,
            "FWDA row has an invalid distance or total"
        );
        let key = (row.distance, row.anchor);
        assert!(
            !previous.is_some_and(|last| last >= key),
            "FWDA rows are not canonically sorted"
        );
        previous = Some(key);
        let entry_count =
            u16::try_from(row.entries.len()).expect("FWDA row entry count exceeds u16");
        let header = header_len + index * row_len;
        bytes[header] = row.distance;
        bytes[header + 2..header + 4].copy_from_slice(&entry_count.to_le_bytes());
        bytes[header + 4..header + 8].copy_from_slice(&row.anchor.to_le_bytes());
        bytes[header + 8..header + 12].copy_from_slice(&row.total.to_le_bytes());
        let entry_offset_u32 = u32::try_from(entry_offset).expect("FWDA entry offset exceeds u32");
        bytes[header + 12..header + 16].copy_from_slice(&entry_offset_u32.to_le_bytes());
        let mut previous_token = None;
        for &(token, count) in &row.entries {
            assert!(
                !previous_token.is_some_and(|last| last >= token),
                "FWDA entries are not canonically sorted"
            );
            previous_token = Some(token);
            bytes.extend_from_slice(&token.to_le_bytes());
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        entry_offset = bytes.len();
    }
    bytes
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
) -> (Vec<u8>, QuantizationErrorStats) {
    let mut bytes = Vec::new();
    let mut quantization = QuantizationErrorStats::default();
    bytes.extend_from_slice(&RESIDUAL_EXCT_MAGIC);
    bytes.push(u8::try_from(store.len()).expect("EXCT level count exceeds u8"));
    bytes.extend_from_slice(&[0u8; 3]);
    for (level, contexts) in store.iter().enumerate() {
        let key_count = u32::try_from(contexts.len())
            .unwrap_or_else(|_| panic!("EXCT level {level} has too many contexts"));
        bytes.extend_from_slice(&key_count.to_le_bytes());
        for (key, distribution) in contexts {
            let key_len = u8::try_from(key.len())
                .unwrap_or_else(|_| panic!("EXCT key at level {level} is too long"));
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
            let entry_count =
                u32::try_from(ranked.len()).expect("EXCT residual entry count exceeds u32");
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
    (bytes, quantization)
}

/// Emit the scored graph as an R4G1 container: the cover's HEAD/NODE/
/// ROUT conventions with E_f merged into EDGE (kind tags distinguish
/// E_r/E_o/E_f), the EMIT residual tables with per-node ranges wired,
/// and the residualized RX1 EXCT table. Panics closed: every input is
/// compiler-produced, so Theorem 7 wiring, the section size ceilings, and
/// the `GraphView::parse` + `verify_cids` re-validation of the just-emitted
/// bytes are self-produced invariants — a violation is a defect and aborts
/// rather than surfacing a `Result` (R5, #510).
pub fn emit_scored_r4g1(
    artifact_container: &[u8],
    corpus_cid_material: (&[u8], &[u8]),
    vocab_size: u32,
    sections: &ScoredGraphSections,
) -> (Vec<u8>, ScoredGraphInfo) {
    let ScoredGraphSections {
        regions,
        structural,
        transitions,
        transition_quantization,
        emissions,
        context_rows,
        exct_tls1,
        exct_top_x,
        fwd_rows,
    } = *sections;
    assert_eq!(
        regions.len(),
        emissions.region_lists.len(),
        "emission lists do not match the region count"
    );
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
    if let Some(violation) = verify_theorem_7_wired(&edges, &reverse, &forward_start, &forward_len)
    {
        panic!("{violation}");
    }

    // EMIT: descriptor + root prior block + per-region lists; wire the
    // per-node ranges as we lay the lists down.
    let mut emit = vec![2u8, 0, 0, 0]; // {width: i32, shift: 0, zero_point: 0}
    let root_entry_count =
        u32::try_from(emissions.root_prior.len()).expect("root prior exceeds u32 entries");
    emit.extend_from_slice(&root_entry_count.to_le_bytes());
    let root_total = emissions.root_total.min(u32::MAX as u64) as u32;
    emit.extend_from_slice(&root_total.to_le_bytes());
    emit.extend_from_slice(&emissions.root_floor.raw().to_le_bytes());
    emit.extend_from_slice(&0u32.to_le_bytes()); // reserved
    for (&token, &value) in &emissions.root_prior {
        let token =
            i32::try_from(token).unwrap_or_else(|_| panic!("root prior token {token} exceeds i32"));
        emit.extend_from_slice(&token.to_le_bytes());
        emit.extend_from_slice(&value.raw().to_le_bytes());
    }
    let mut emission_start = vec![0u32; node_total];
    let mut emission_len = vec![0u16; node_total];
    let mut emission_list_entries = 0u32;
    for (region_id, list) in emissions.region_lists.iter().enumerate() {
        let node = 1 + region_id;
        emission_start[node] = (emit.len() - 4) as u32; // remainder-relative
        emission_len[node] = u16::try_from(list.len()).expect("emission list exceeds u16 entries");
        for &(token, value) in list {
            let token = i32::try_from(token)
                .unwrap_or_else(|_| panic!("emission token {token} exceeds i32"));
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
        .expect("EXCT input is not a TLS1 store");
    let (exct_body, exact_context_quantization) = emit_residual_exct(
        &store,
        &emissions.root_prior,
        emissions.root_floor,
        vocab_size,
        exct_top_x,
        emissions.smoothing,
    );
    let mut exct = Vec::with_capacity(4 + exct_body.len());
    exct.extend_from_slice(&[2, 0, 0, 0]);
    exct.extend_from_slice(&exct_body);
    let ngram = encode_context_rows(context_rows);

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
    let fwda = if fwd_rows.is_empty() {
        Vec::new()
    } else {
        let fwda = encode_forward_anchor_rows(fwd_rows);
        builder.add_section(uor_r4_graph_format::SectionId::FWDA, 0, &fwda);
        fwda
    };
    let bytes = builder
        .build()
        .unwrap_or_else(|error| panic!("R4G1 serialization failed: {error}"));

    // Panic closed: never emit an artifact the two-stage validator or the
    // integrity CIDs reject — these bytes are self-produced, so rejection
    // is a compiler defect.
    let view = uor_r4_graph_format::GraphView::parse(&bytes)
        .unwrap_or_else(|error| panic!("score emitted an invalid R4G1 artifact: {error}"));
    if let Err(error) = view.verify_cids() {
        panic!("score emitted an artifact with bad CIDs: {error}");
    }

    let artifact_bytes = bytes.len();
    (
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
                .expect("NGRAM row count exceeds u32"),
            context_entry_count: u32::try_from(
                context_rows
                    .iter()
                    .map(|row| row.entries.len())
                    .sum::<usize>(),
            )
            .expect("NGRAM entry count exceeds u32"),
            context_bytes: u32::try_from(ngram.len()).expect("NGRAM section exceeds u32"),
            fwda_row_count: u32::try_from(fwd_rows.len()).expect("FWDA row count exceeds u32"),
            fwda_bytes: u32::try_from(fwda.len()).expect("FWDA section exceeds u32"),
            artifact_bytes,
            transition_quantization,
            root_prior_quantization: emissions.root_prior_quantization,
            emission_quantization: emissions.emission_quantization,
            exact_context_quantization,
            emission_selection_stats: emissions.selection_stats,
        },
    )
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
    let regions = regions_from_view(&view)
        .ok_or_else(|| "cover artifact is missing its scoring regions".to_owned())?;
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
/// bits/token of `next` under the candidate distribution.
///
/// `transition_offset` is added to every candidate score by the scorer. It is
/// rank-neutral by construction, so it changes no decision — but the
/// uncovered-vocabulary floor is a root-prior-scale quantity that does NOT
/// carry it. Pricing candidates and the floor on different scales inflates
/// bits/token by roughly the offset (#387): with an offset of −118 nats the
/// graph slice reported 115 bits/token, and bits were HIGHER when the teacher
/// was retrieved (143.7) than when it was absent (46.8), because being a
/// candidate priced a token below one never observed.
///
/// Shifting the floor by the same offset puts both on one scale. Softmax is
/// shift-invariant, so this is equivalent to removing the offset from the
/// candidates and changes nothing where the offset is zero.
fn outcome_bits(
    scorer: &GraphScorer,
    candidates: &[(u32, ScoreQ)],
    next: u32,
    transition_offset: ScoreQ,
) -> f64 {
    let floor = scorer
        .root_floor()
        .raw()
        .saturating_add(transition_offset.raw());
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
///
/// `positions` is the denominator every rate in the row is divided by.
/// `positions_sampled` and `standard_error` (#467) exist so a sampled
/// number can never be quoted as a census: the first says whether the run
/// scored the whole held-out split or a subsample of it, the second says
/// how much of the printed rate is measurement noise.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCMetrics {
    pub positions: usize,
    /// P(selected token == recorded teacher argmax).
    pub top1_agreement: f64,
    pub bits_per_token: f64,
    /// Sample size of the sampled decision run backing this row (#467), or
    /// `0` when the run was a full pass — a census over the whole held-out
    /// split, which is the default and the only mode before schema 25.
    /// Nonzero means every rate in this row is an ESTIMATE.
    pub positions_sampled: usize,
    /// Binomial standard error of `top1_agreement`, `sqrt(p(1-p)/n)` with
    /// `n = positions`. Reported for full passes too, where it is the
    /// sampling error the held-out split itself carries as a draw from the
    /// position population. Not defined for `bits_per_token`.
    pub standard_error: f64,
}

/// Binomial standard error of a rate `p` measured over `n` positions:
/// sqrt of p(1-p)/n, and zero for an empty population.
fn binomial_standard_error(p: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    (p * (1.0 - p) / n as f64).sqrt()
}

/// Environment override selecting the #467 sampled Gate C decision mode.
/// UNSET is the full pass, bit-identical to the pre-#467 evaluation.
pub const GATE_C_SAMPLE_ENV: &str = "R4_GATE_C_SAMPLE";

/// Environment override naming Gate C ARM GROUPS this run must not build
/// (#471). Comma-separated group names; UNSET evaluates every arm and is
/// behaviour-identical to the pre-#471 evaluation.
///
/// Why a group name and not a boolean. The thing worth skipping is never one
/// row; it is the closure of one expensive pass — every row that depends on
/// it. Naming the *cost* rather than the issue number keeps the knob from
/// degenerating into a list of ad-hoc flags as arms accumulate, and it makes
/// the skipped set nameable in the output, which is what stops a skipped row
/// from reading as a measured one.
///
/// Why an unknown name PANICS. A typo that silently evaluated everything
/// would hand back an 85-minute run when the caller asked for a 30-second
/// one, and — worse in the other direction — a typo that silently skipped
/// nothing looks exactly like a knob that does not work. Both failures are
/// invisible; the panic is not.
pub const GATE_C_SKIP_ARMS_ENV: &str = "R4_GATE_C_SKIP_ARMS";

/// The `right_context` arm group: the #446 M1 two-sided family and the #446
/// M2 latent family together.
///
/// They are one group because they are exactly the closure of
/// [`derive_right_codes`] — the whole-corpus right-context code pass that
/// #471 measured at 60% of a sampled Gate C run's wall clock on the 500k
/// fixture, and a larger share as the corpus grows (the pass scales with
/// `n`; the scored sample does not). Nothing else in the evaluation reads a
/// right-context code, so skipping this group and skipping that pass are the
/// same act.
pub const ARM_GROUP_RIGHT_CONTEXT: &str = "right_context";

/// Which arm groups a run was told to skip. `Default` is "skip nothing",
/// which is the unset contract.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SkippedArms {
    /// [`ARM_GROUP_RIGHT_CONTEXT`].
    pub right_context: bool,
}

impl SkippedArms {
    /// The group names this run skipped, in a stable order, for the report
    /// and the printed banner.
    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if self.right_context {
            names.push(ARM_GROUP_RIGHT_CONTEXT.to_owned());
        }
        names
    }

    pub fn any(&self) -> bool {
        self.right_context
    }
}

/// Parse a [`GATE_C_SKIP_ARMS_ENV`] value. Pure, so the contract is unit
/// testable without mutating process environment from a test thread.
///
/// Empty and whitespace-only values mean "skip nothing", matching UNSET —
/// `R4_GATE_C_SKIP_ARMS=` in a shell script should not be a different mode
/// from not writing the line at all. Names are trimmed and matched exactly;
/// an unrecognised one panics rather than being dropped.
pub fn parse_skip_arms(value: Option<&str>) -> SkippedArms {
    let mut skipped = SkippedArms::default();
    let Some(value) = value else {
        return skipped;
    };
    for name in value.split(',') {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        match name {
            ARM_GROUP_RIGHT_CONTEXT => skipped.right_context = true,
            other => panic!(
                "{GATE_C_SKIP_ARMS_ENV}: unknown arm group {other:?}; \
                 known groups: {ARM_GROUP_RIGHT_CONTEXT}"
            ),
        }
    }
    skipped
}

fn gate_c_skip_arms() -> SkippedArms {
    parse_skip_arms(std::env::var(GATE_C_SKIP_ARMS_ENV).ok().as_deref())
}

/// Wall-clock accounting for the passes of a scoring stage, printed to stderr
/// as the stage proceeds (#471, generalized in #488). Each instance carries a
/// `prefix` so the same instrument reads as one log across two levels:
/// `evaluate_gate_c` runs it with `"gate c phase"`, and the `score` pipeline
/// (`graph-cli::score_command`) runs it with `"score phase"` around the stages
/// BEFORE Gate C — cover induction, store build, transition and row
/// compilation, emission, artifact emission — where a real corpus's hours
/// actually live (#488).
///
/// Why this exists. "The Gate C phase took eighty-five minutes" was, until
/// #471, an unattributable number: the evaluation runs four whole-corpus
/// passes before `gate_c_sample` narrows anything, and nothing said which of
/// them owned the wall clock. Attributing it took a code read, and the first
/// two proposals for fixing it named different passes. A harness whose cost
/// is invisible gets optimized by argument rather than by measurement.
///
/// Why stderr and not the report. A duration is machine- and load-dependent,
/// so putting one in `score_report.json` would make the document differ
/// between two runs of identical pinned inputs — exactly what the
/// deterministic-rebuild gate exists to catch. These lines are diagnostics
/// beside the existing `score: …` progress prints, never report content.
pub struct PhaseLog {
    prefix: &'static str,
    start: std::time::Instant,
    last: std::time::Instant,
}

impl PhaseLog {
    /// `prefix` is the log label (e.g. `"gate c phase"` or `"score phase"`);
    /// every line this instance prints begins with it, so lines from nested
    /// instances stay distinguishable while reading as one stream.
    pub fn new(prefix: &'static str) -> Self {
        let now = std::time::Instant::now();
        PhaseLog {
            prefix,
            start: now,
            last: now,
        }
    }

    /// Close the phase that ended here, naming it and its duration.
    pub fn mark(&mut self, phase: &str) {
        let now = std::time::Instant::now();
        eprintln!(
            "{}: {phase} {:.2}s (cumulative {:.2}s)",
            self.prefix,
            now.duration_since(self.last).as_secs_f64(),
            now.duration_since(self.start).as_secs_f64()
        );
        self.last = now;
    }

    /// Close the whole stage. Measured from `now`, not from the last mark, so
    /// any unmarked tail shows up in the total instead of vanishing — the
    /// remainder is the check that no stage went unnamed (#488).
    pub fn total(&self) {
        eprintln!(
            "{}: TOTAL {:.2}s",
            self.prefix,
            self.start.elapsed().as_secs_f64()
        );
    }
}

/// Deterministic stratified subsample of the held-out evaluation list.
///
/// Motivation. The long runs buy precision nobody uses. At 402,802
/// held-out positions with top-1 near 26 percent the standard error of the
/// rate is 0.07pp, while every pre-declared exit rule in this repository is
/// written at plus or minus 2pp — about 30x finer than any decision needs,
/// bought with hours of wall clock. A 10,000-position decision run has
/// standard error 0.44pp, still comfortably inside a 2pp rule, and costs
/// about 40x less. Sampling is therefore a decision-speed knob, never a
/// precision claim: the sample size and the standard error travel with
/// every rate so a sampled number cannot be mistaken for a census.
///
/// Contract, in the [`compiler::capacity_override_usize`] style: UNSET is
/// behaviour-identical to the full pass (every held-out position scored, in
/// input order, with bit-identical floating-point reductions). Set to `n`,
/// the evaluation scores a stride-selected subsample of size
/// `min(n, population)`.
///
/// Selection is a stride, not an RNG: position `k` of the sample is
/// held-out entry `k * population / n`. That is deterministic (the same
/// corpus and the same `n` always select the same positions, so a sampled
/// run is reproducible and diffable), it preserves input order (so the
/// reduction stays order-deterministic), and it is stratified — the
/// held-out list is in corpus order, so an even stride spreads the sample
/// across every story and every region of the corpus instead of
/// concentrating it in one contiguous block.
///
/// What sampling does NOT change: the construction/held-out split itself.
/// The held-out MASK, and therefore every table built from the
/// construction split (forward-anchor, two-sided, latent, unigram null),
/// is derived from the FULL held-out list. Only the set of positions
/// SCORED shrinks. A sampled run measures the same model as a full run.
fn gate_c_sample(held_out: &[Observation]) -> (Vec<&Observation>, usize) {
    let population = held_out.len();
    let requested = match std::env::var(GATE_C_SAMPLE_ENV) {
        Ok(value) => {
            let parsed = value
                .trim()
                .replace('_', "")
                .parse::<usize>()
                .unwrap_or_else(|_| {
                    panic!("{GATE_C_SAMPLE_ENV} must be a positive integer, got {value:?}")
                });
            assert!(
                parsed >= 1,
                "{GATE_C_SAMPLE_ENV} must be >= 1, got {value:?}"
            );
            parsed
        }
        Err(_) => return (held_out.iter().collect(), 0),
    };
    if requested >= population {
        return (held_out.iter().collect(), 0);
    }
    let scored = (0..requested)
        .map(|k| &held_out[k * population / requested])
        .collect();
    (scored, requested)
}

/// Per-status position counts of the Rule 1+2 scorer (D4 precedence).
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusCounts {
    pub exact_context: usize,
    pub graph: usize,
    pub novel: usize,
    /// Exact-context positions resolved by an explicit NGRAM context row
    /// (#362 attribution; schema 13). Zero on distributions whose probes
    /// carry no recent-token window.
    pub exact_context_ngram: usize,
    /// Exact-context positions resolved by the EXCT full-depth probe.
    pub exact_context_probe: usize,
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
    /// #399 M2: fused (Rule 1+2 × forward-anchor) vs Rule 1+2, all
    /// held-out positions (inert positions tie by construction).
    pub fwd_vs_rule12: WinLoss,
    /// #399 M2: the same cross-tab restricted to LIVE positions (free
    /// target with an in-story next anchor and a populated forward row)
    /// — the only positions where the channel can change a decision.
    pub fwd_vs_rule12_live: WinLoss,
    /// #399 B′: self-anchor arm on its live slice.
    pub fwd_self_vs_rule12_live: WinLoss,
    /// #399 B′: confidence-gated self-anchor arm on its live slice.
    pub fwd_gated_vs_rule12_live: WinLoss,
    /// #399 falsifier 1: DRAFT-anchor gated arm on its live slice.
    pub fwd_draft_vs_rule12_live: WinLoss,
    /// #399 rescue variant: STRICT-gated draft arm on its live slice
    /// (every draft step ExactContext, not just the final one).
    pub fwd_strict_vs_rule12_live: WinLoss,
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
    /// bits/token split by whether the teacher was in the candidate set, plus
    /// the mean rank-neutral offset applied. If `bits_teacher_absent` dominates
    /// and scales with the offset, the slice's bits figure is an accounting
    /// artifact rather than model error.
    pub bits_teacher_present: f64,
    pub bits_teacher_absent: f64,
    pub positions_teacher_present: usize,
    pub positions_teacher_absent: usize,
    pub mean_transition_offset_nats: f64,
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

/// Analytic unigram-null baselines (#390). Computed from the TRAIN
/// next-token distribution (add-one smoothed over the compiled
/// vocabulary, the repo's standing convention) and evaluated on the
/// held-out slice. Residuals are only interpretable as GAPS against a
/// null at matched granularity: a finer partition mechanically shifts
/// what an address-free predictor achieves, so raw values compared
/// across covers of different region counts are not the same quantity
/// (the #374 sweep tables are the standing example). The granularity
/// context (region count, artifact footprint) lives in
/// `ScoreReportGraph`; readers comparing across covers must recompute
/// nulls at matched atom counts.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct GateCNulls {
    /// The train-majority next token (the unigram argmax).
    pub unigram_train_argmax: u32,
    /// Frequency of the train-majority token on the FULL held-out slice
    /// — the top-1 an address-free constant predictor achieves.
    pub unigram_null_top1_all: f64,
    /// Train-unigram cross-entropy (bits/token) on the full held-out
    /// slice.
    pub unigram_null_bits_all: f64,
    /// The same null restricted to the rule12 GENERALIZATION slice
    /// (graph + novel — the population exact-context lookup cannot
    /// answer). `rule12_generalization` must beat THIS, not the blended
    /// baseline.
    pub unigram_null_top1_generalization: f64,
    /// Train-unigram cross-entropy on the generalization slice.
    pub unigram_null_bits_generalization: f64,
    /// Train / held-out position counts the null was computed from.
    pub train_positions: usize,
    pub held_out_positions: usize,
}

/// The #446 right-context arm group: the M1 two-sided family and the M2
/// latent right-context family, which share the whole-corpus
/// [`derive_right_codes`] pass and are therefore built, skipped and
/// reported together.
///
/// Carried behind an `Option` on [`GateCOutcome`] so that "not evaluated"
/// is a different value from "evaluated and zero" — see
/// [`GateCOutcome::right_context_arms`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct RightContextArms {
    /// #446 M1: Rule 1+2 with the TWO-SIDED (left graded prefix, right
    /// graded prefix) table taking D4-style precedence wherever it
    /// resolves with support, over ALL held-out positions. Where the
    /// table is inert the selection IS the Rule 1+2 selection, so this
    /// row can only differ from `rule12_precedence` through live
    /// positions.
    ///
    /// NOT CAUSAL. The right key is built from the tokens AFTER the
    /// target; this is an infill/analysis (A-mode) measurement, or
    /// prospectively a construction-time signal. It must never be quoted
    /// as a generation number.
    pub rule12_twosided: GateCMetrics,
    /// #446 M1: the two-sided arm on its LIVE slice only.
    pub rule12_twosided_live: GateCMetrics,
    /// #446 M1: Rule 1+2 on the same live slice — the honest comparator
    /// for `rule12_twosided_live` (identical population).
    pub rule12_on_twosided_live: GateCMetrics,
    /// #446 M1 falsifier: the identical two-sided machinery with the
    /// right key taken from a FOREIGN held-out position (fixed
    /// half-length rotation over the held-out list). Key cardinality,
    /// backoff shape, support gate and smoothing are unchanged, so a
    /// `rule12_twosided` gain that survives this row is right-context
    /// information rather than a larger key space. Without this row the
    /// two-sided row is not interpretable.
    pub rule12_twosided_shuffled: GateCMetrics,
    pub rule12_twosided_shuffled_live: GateCMetrics,
    pub rule12_on_twosided_shuffled_live: GateCMetrics,
    /// #446 M1: how deep the two-sided pair resolved, indexed by
    /// graded-prefix depth (index 0 counts positions where no supported
    /// pair existed and the arm fell through to Rule 1+2).
    pub rule12_twosided_depths: Vec<usize>,
    /// #446 M1 DILUTION SLICE. The graded code space is fixed at
    /// `STAGES` x 256, so as the corpus grows the held-out population
    /// concentrates into full-code exact-context cells and each cell's
    /// next-token distribution dilutes — top-1 falls even though key
    /// resolution is nominally maximal. These rows ask whether the right
    /// key splits that population usefully.
    ///
    /// Restricted to held-out positions whose Rule 1+2 status was
    /// ExactContext: the two-sided arm's metrics and, on the identical
    /// population, Rule 1+2's own.
    ///
    /// NOT CAUSAL — infill/analysis only, like every two-sided row.
    pub rule12_twosided_exct_slice: GateCMetrics,
    pub rule12_on_twosided_exct_slice: GateCMetrics,
    /// Positions in that slice where the two-sided pair actually
    /// resolved with support (the arm was live rather than inert).
    pub rule12_twosided_exct_slice_live: usize,
    /// Subdivision of the diluted cells, measured on the construction
    /// split at FULL graded depth: distinct left codes, distinct
    /// two-sided keys, and their ratio — the mean number of two-sided
    /// keys the right context carves each full left code into.
    pub twosided_full_left_cells: usize,
    pub twosided_full_pair_keys: usize,
    pub twosided_keys_per_full_left: f64,
    /// #446 M2, THE CAUSALLY LEGITIMATE ROW. The latent right-context
    /// mixture over ALL held-out positions: the right context is
    /// observed only during construction and marginalized away at
    /// serving, where the left key alone is read. Quotable as a
    /// generation number.
    pub rule12_latent_mix: GateCMetrics,
    /// #446 M2: the mixture arm on its LIVE slice, and Rule 1+2 on that
    /// identical slice.
    pub rule12_latent_mix_live: GateCMetrics,
    pub rule12_on_latent_mix_live: GateCMetrics,
    /// #446 M2 UPPER BOUND, NOT CAUSAL. The same construction tables
    /// with the TRUE right class supplied at evaluation time instead of
    /// the class posterior. Measures how much of the two-sided gain the
    /// marginalization gives up. Never quotable as a generation number.
    pub rule12_latent_oracle: GateCMetrics,
    /// #446 M2 FALSIFIER. Identical machinery with the class posterior
    /// lifted from a FOREIGN left key under a fixed rotation, so class
    /// cardinality, emission tables, smoothing, support gate and backoff
    /// are held constant. Without this row the mixture row is not
    /// interpretable.
    pub rule12_latent_shuffled: GateCMetrics,
    pub latent_oracle_live_positions: usize,
    pub latent_shuffled_live_positions: usize,
    /// Bytes of the right graded code forming the latent class.
    pub latent_class_depth: usize,
    /// Construction-split class structure at FULL left depth: distinct
    /// left cells, distinct (left, class) cells, and their ratio.
    pub latent_full_left_cells: usize,
    pub latent_full_class_cells: usize,
    pub latent_classes_per_full_left: f64,
    /// Where `rule12_latent_mix` sits between `rule12_precedence` and
    /// `rule12_latent_oracle` as a fraction of the available top-1
    /// headroom. Zero when the oracle does not exceed the baseline.
    pub latent_headroom_fraction: f64,
    /// The pre-declared #446 M2 exit rule: the mixture beat the
    /// left-only Rule 1+2 baseline by at least
    /// [`LATENT_EXIT_MARGIN`] of top-1 agreement on the same population
    /// AND beat the shuffled-class falsifier.
    pub latent_exit_rule_met: bool,
    /// #446 M1: two-sided (left, right) keyed arm vs Rule 1+2 on the
    /// two-sided live slice — the positions where the pair table
    /// resolved with support and could change a decision. NOT causal:
    /// the right key reads tokens after the target, so this cross-tab is
    /// an infill/analysis measurement, never a generation number.
    ///
    /// Lives here rather than in `win_loss` (where it sat before schema
    /// twenty-six) because it is in the closure of the right-context code
    /// pass: on a skipped run it has no value, and an all-zero cross-tab
    /// sitting among populated ones is precisely the reading this group's
    /// `Option` exists to prevent. The #471 equivalence test caught it
    /// there.
    pub twosided_vs_rule12_live: WinLoss,
    /// #446 M1 falsifier: the foreign-right-key arm against Rule 1+2 on
    /// its own live slice.
    pub twosided_shuffled_vs_rule12_live: WinLoss,
}

/// The Gate C outcome: the four number sets (old formula, Rule 1,
/// Rule 1+2, baseline), the status and win/loss instrumentation,
/// candidate recall, and the witness-replay sample result.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GateCOutcome {
    /// Size of the held-out split the evaluation was handed (#467). Equal
    /// to the number of positions scored on a full pass.
    pub held_out_population: usize,
    /// Number of held-out positions actually scored when the #467 sampled
    /// decision mode is active, or `0` on a full pass (a census). Nonzero
    /// means every rate in this outcome is an ESTIMATE whose noise is the
    /// `standard_error` of its own row.
    pub positions_sampled: usize,
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
    /// #399 M2 instrumentation: Rule 1+2 fused with the forward-anchor
    /// channel by the measured product law (harness record on #394/#399),
    /// over ALL held-out positions; where the channel is inert the fused
    /// selection IS the Rule 1+2 selection, so this row can only differ
    /// from `rule12_precedence` through live positions.
    pub rule12_fwd_fused: GateCMetrics,
    /// #399 M2: the fused scorer on the LIVE slice only.
    pub rule12_fwd_fused_live: GateCMetrics,
    /// #399 M2: Rule 1+2 on the same live slice — the honest comparator
    /// for `rule12_fwd_fused_live` (identical population).
    pub rule12_on_fwd_live: GateCMetrics,
    /// #399 B′: the SELF-anchor fused scorer (all positions; live pair
    /// below) — the two-pass serving question, anchor supplied by the
    /// engine's own Rule 1+2 prediction at the anchor position.
    pub rule12_fwd_self_fused: GateCMetrics,
    pub rule12_fwd_self_fused_live: GateCMetrics,
    pub rule12_on_fwd_self_live: GateCMetrics,
    /// #399 B′: the confidence-gated self-anchor scorer (prediction
    /// trusted only where it resolved as ExactContext).
    pub rule12_fwd_gated_fused: GateCMetrics,
    pub rule12_fwd_gated_fused_live: GateCMetrics,
    pub rule12_on_fwd_gated_live: GateCMetrics,
    /// #399 falsifier 1: the DRAFT-anchor gated scorer — same gate and
    /// fusion law as the gated arm, but the anchor is predicted from the
    /// engine's own greedy DRAFT (pass-1 token here plus drafted
    /// continuations) instead of teacher-forced corpus context; the
    /// spread against the gated arm is the draft-drift cost of a real
    /// two-pass generation.
    pub rule12_fwd_draft_fused: GateCMetrics,
    pub rule12_fwd_draft_fused_live: GateCMetrics,
    pub rule12_on_fwd_draft_live: GateCMetrics,
    /// #399 rescue variant (responding to the falsifier-1 negative:
    /// draft-gated live 40.4% vs 41.3% while the teacher-forced gated
    /// arm measured 45.0% vs 39.4%): the STRICT-gated draft scorer —
    /// identical draft, anchor, and fusion, but trusted only where EVERY
    /// intermediate greedy step resolved as ExactContext (drift enters
    /// through uncertain intermediate steps). Its live slice is a subset
    /// of the draft arm's.
    pub rule12_fwd_strict_fused: GateCMetrics,
    pub rule12_fwd_strict_fused_live: GateCMetrics,
    pub rule12_on_fwd_strict_live: GateCMetrics,
    /// #446 M1 + M2, THE RIGHT-CONTEXT ARM GROUP (`right_context`).
    ///
    /// `None` means these arms were NOT EVALUATED on this run — the
    /// whole-corpus right-context code pass they all depend on was skipped
    /// via [`GATE_C_SKIP_ARMS_ENV`] (#471). It does not mean they measured
    /// zero, and `latent_exit_rule_met` being absent does not mean the exit
    /// rule failed.
    ///
    /// The `Option` is the point. These twenty-four numbers used to sit flat
    /// on this struct, so a skipped run would have serialised
    /// `rule12_latent_mix.top1_agreement: 0.0` and
    /// `latent_exit_rule_met: false` — an all-zero arm set, which this
    /// repository has learned to read as a harness bug, and a pre-declared
    /// exit rule reported as failed when it was never run. Absence and a
    /// measured zero must not share a representation, so they do not.
    pub right_context_arms: Option<RightContextArms>,
    /// Arm groups this run was told not to build, by name (#471), so the
    /// report says what is missing rather than leaving a reader to infer it
    /// from a null. Empty on an ordinary run.
    pub skipped_arm_groups: Vec<String>,
    /// #399 B′: predicted-anchor accuracy on the anchor-reachable
    /// population (numerator/denominator + rate).
    pub anchor_hat_population: usize,
    pub anchor_hat_correct: usize,
    pub anchor_hat_accuracy: f64,
    /// The EXCT-free headline (#390): rule12 restricted to the graph +
    /// novel population — the generalization number, promoted from
    /// per-status footnote to first-class row.
    pub rule12_generalization: GateCMetrics,
    /// Analytic unigram-null baselines (#390).
    pub nulls: GateCNulls,
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
        let outcome = scorer
            .score_candidates_coded(&sig, Some(&code), &recent_array[..recent_len])
            .expect("gate C: scoring self-produced sig");
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
/// #399 M2 / #394 infill protocol: anchor stride. Every position whose
/// emitted token lands on a story-position multiple of this stride is an
/// anchor; the forward-anchor channel conditions the free positions
/// between anchors on the NEXT anchor's token. Mirrors the original
/// hybrid's every-4th-token injection and the measured harness law.
const M2_STRIDE: usize = 4;

/// #446 M1: how many tokens after the target position form the RIGHT
/// context window of the two-sided key. Matches the certifier harness
/// default (`R4_TS_RIGHT_R`) whose numbers this in-pipeline arm tests.
const TWO_SIDED_RIGHT_R: usize = 4;

/// One packed two-sided key: the left graded-code prefix in the high
/// word, the right graded-code prefix in the low word. The depth is
/// carried by the level index, so no depth tag lives inside the word.
type TwoSidedKey = u64;

/// A read view of one two-sided cell: the key's token counts (token
/// ascending) and its evidence total.
#[derive(Clone, Copy)]
struct TwoSidedCell<'a> {
    entries: &'a [(u32, u32)],
    total: u32,
}

impl TwoSidedCell<'_> {
    /// Canonical argmax: highest count, ties to the lowest token id —
    /// the same tie-break every other Gate C selection uses.
    fn argmax(&self) -> Option<u32> {
        self.entries
            .iter()
            .copied()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
            .map(|(token, _)| token)
    }
    fn count(&self, token: u32) -> u32 {
        self.entries
            .binary_search_by_key(&token, |&(t, _)| t)
            .map(|index| self.entries[index].1)
            .unwrap_or(0)
    }
}

/// One two-sided level in compact sorted form. A per-key `BTreeMap` of
/// counts would cost several hundred bytes per key and there are on the
/// order of one key per construction position at full depth, so the
/// level is stored as parallel arrays: `keys` sorted and unique,
/// `spans[k]..spans[k + 1]` indexing `entries` for `keys[k]`, and
/// `totals[k]` that key's evidence total. Lookup is a binary search.
#[derive(Default)]
struct TwoSidedLevel {
    keys: Vec<TwoSidedKey>,
    spans: Vec<u32>,
    entries: Vec<(u32, u32)>,
    totals: Vec<u32>,
}

impl TwoSidedLevel {
    /// Compact a `(key, observed next token)` bag into the sorted form.
    fn from_pairs(mut pairs: Vec<(TwoSidedKey, u32)>) -> Self {
        pairs.sort_unstable();
        let mut level = TwoSidedLevel {
            spans: vec![0],
            ..TwoSidedLevel::default()
        };
        let mut index = 0usize;
        while index < pairs.len() {
            let key = pairs[index].0;
            let mut total = 0u32;
            let start = index;
            while index < pairs.len() && pairs[index].0 == key {
                let token = pairs[index].1;
                let mut count = 0u32;
                while index < pairs.len() && pairs[index].0 == key && pairs[index].1 == token {
                    count += 1;
                    index += 1;
                }
                level.entries.push((token, count));
                total += count;
            }
            debug_assert!(index > start);
            level.keys.push(key);
            level.totals.push(total);
            level.spans.push(level.entries.len() as u32);
        }
        level
    }

    fn get(&self, key: TwoSidedKey) -> Option<TwoSidedCell<'_>> {
        let index = self.keys.binary_search(&key).ok()?;
        let lo = self.spans[index] as usize;
        let hi = self.spans[index + 1] as usize;
        Some(TwoSidedCell {
            entries: &self.entries[lo..hi],
            total: self.totals[index],
        })
    }
}

/// The #446 M1 two-sided context table: one level per graded-prefix
/// depth one through `STAGES`, built from the CONSTRUCTION split only.
///
/// NOT CAUSAL. The right key is drawn from tokens AFTER the target, so
/// this table can only be read in an infill/analysis (A-mode) regime or,
/// prospectively, at CONSTRUCTION time. Nothing here touches the serving
/// scorer, the witness, or the replay contract.
struct TwoSidedTable {
    levels: Vec<TwoSidedLevel>,
}

/// Pack a graded-code prefix of length `depth` into one word.
fn pack_prefix(code: &[u8; STAGES], depth: usize) -> u32 {
    let mut packed = 0u32;
    for &byte in &code[..depth] {
        packed = (packed << 8) | u32::from(byte);
    }
    packed
}

fn pack_two_sided(left: u32, right: u32) -> TwoSidedKey {
    (TwoSidedKey::from(left) << 32) | TwoSidedKey::from(right)
}

impl TwoSidedTable {
    /// The table a run that skipped [`ARM_GROUP_RIGHT_CONTEXT`] carries:
    /// every level empty, so [`TwoSidedTable::resolve`] is inert at every
    /// position and the arm falls through to Rule 1+2 exactly as it does
    /// where no supported pair exists.
    ///
    /// The rows this then produces are Rule 1+2 passthrough values and are
    /// NOT two-sided measurements. They never reach the report:
    /// [`GateCOutcome::right_context_arms`] is `None` on a skipped run, and
    /// the type is what keeps them from being published as arm numbers.
    fn empty() -> Self {
        TwoSidedTable {
            levels: (0..=STAGES).map(|_| TwoSidedLevel::default()).collect(),
        }
    }

    /// Build every depth from the construction positions that carry an
    /// in-story right window.
    fn build(
        corpus: &Corpus,
        is_held_out: &[bool],
        left_codes: &[[u8; STAGES]],
        right_codes: &[([u8; STAGES], bool)],
    ) -> Self {
        let mut levels = Vec::with_capacity(STAGES + 1);
        levels.push(TwoSidedLevel::default());
        for depth in 1..=STAGES {
            let mut pairs: Vec<(TwoSidedKey, u32)> = Vec::new();
            for position in 0..corpus.n {
                if is_held_out[position] || !right_codes[position].1 {
                    continue;
                }
                let key = pack_two_sided(
                    pack_prefix(&left_codes[position], depth),
                    pack_prefix(&right_codes[position].0, depth),
                );
                pairs.push((key, corpus.next[position]));
            }
            levels.push(TwoSidedLevel::from_pairs(pairs));
        }
        TwoSidedTable { levels }
    }

    /// #446 M1 dilution statistic: at FULL graded depth, how many
    /// distinct two-sided keys share each distinct left code.
    ///
    /// Returns `(distinct full left codes, distinct full two-sided
    /// keys)`. Their ratio is the mean subdivision factor — the number
    /// of cells the right key splits an over-diluted full-left-code cell
    /// into. This is the direct measurement of the dilution attack: the
    /// graded code space is fixed at `STAGES` x 256, so as the corpus
    /// grows every full-code cell absorbs more records and its
    /// next-token distribution dilutes; a subdivision factor above one
    /// means the right key is buying key resolution the left code
    /// cannot.
    fn full_depth_subdivision(&self) -> (usize, usize) {
        let keys = &self.levels[STAGES].keys;
        // `keys` is sorted and unique, and the left code occupies the
        // high word, so distinct left codes are the runs of the high
        // word — one linear scan, no allocation.
        let mut left_cells = 0usize;
        let mut previous: Option<u32> = None;
        for &key in keys {
            let left = (key >> 32) as u32;
            if previous != Some(left) {
                left_cells += 1;
                previous = Some(left);
            }
        }
        (left_cells, keys.len())
    }

    /// The DEEPEST populated `(left[..d], right[..d])` pair whose
    /// evidence total clears the D4 EXCT support gate
    /// ([`score_runtime::EXCT_SUPPORT_MIN`]) — the harness's
    /// deepest-populated-pair backoff with the EXCT probe's support
    /// discipline. `None` means the arm is inert at this position and
    /// falls through to the Rule 1+2 selection.
    fn resolve(
        &self,
        left: &[u8; STAGES],
        right: &([u8; STAGES], bool),
    ) -> Option<(usize, TwoSidedCell<'_>)> {
        if !right.1 {
            return None;
        }
        for depth in (1..=STAGES).rev() {
            let key = pack_two_sided(pack_prefix(left, depth), pack_prefix(&right.0, depth));
            if let Some(cell) = self.levels[depth].get(key) {
                if cell.total >= EXCT_SUPPORT_MIN {
                    return Some((depth, cell));
                }
            }
        }
        None
    }
}

/// #446 M1: the right graded code of every corpus position, at MATCHED
/// granularity with the left code. The next [`TWO_SIDED_RIGHT_R`]
/// emitted tokens after the target are laid out REVERSE-ORDERED —
/// farthest first, so the token immediately after the target lands in
/// the most-recent dyadic slot — and pushed through the same
/// `bundle_window_plain` + `assign_for_bundle` machinery that produces
/// the left code, so the pair at depth d is a genuinely matched object.
/// The flag is false where no in-story right token exists (story end).
fn derive_right_codes(
    artifacts: &compiler::Compiled,
    rotations: &[usize; compiler::WINDOW + 1],
    corpus: &Corpus,
) -> Vec<([u8; STAGES], bool)> {
    // #469 lever B: decode the dot tables ONCE and share them across the
    // workers. Per-call decoding would cost more than the scan it replaces
    // (~1690us vs ~466us on the synthetic fixture), which is why the kernel
    // could not simply be called from `assign_for_bundle`. Codes are
    // unchanged — `tests/assign_prepared.rs` and `kappa_reproduction.rs`
    // carry that obligation.
    let tables = runtime::AssignTables::new(artifacts);
    (0..corpus.n)
        .into_par_iter()
        .map(|position| {
            let mut window: Vec<u32> = Vec::with_capacity(TWO_SIDED_RIGHT_R);
            for ahead in (1..=TWO_SIDED_RIGHT_R).rev() {
                let source = position + ahead;
                if source < corpus.n && corpus.story[source] == corpus.story[position] {
                    window.push(corpus.next[source]);
                }
            }
            if window.is_empty() {
                return ([0u8; STAGES], false);
            }
            let bundle = runtime::bundle_window_plain(artifacts, rotations, &window);
            (
                runtime::assign_code_for_bundle_with(&tables, artifacts, &bundle),
                true,
            )
        })
        .collect()
}

/// #446 M1: apply the two-sided table with D4-style precedence over one
/// Rule 1+2 outcome.
///
/// Selection: where the table resolves with support the two-sided
/// evidence PREEMPTS (its canonical argmax is the selection), exactly as
/// supported exact-context evidence preempts the graph under Rule 2.
/// Bits: a Witten-Bell mixture of the resolved cell over the Rule 1+2
/// distribution, `lambda = total / (total + types)`. The Rule 1+2
/// channel is already normalized over the vocabulary and `rule12_bits`
/// is its code length for `next`, so the mixture is a proper code length
/// on the same scale as every other Gate C row.
///
/// NOT CAUSAL: the key reads tokens after the target. Infill/analysis
/// only.
fn apply_two_sided_arm(
    resolved: Option<(usize, TwoSidedCell<'_>)>,
    rule12_selected: u32,
    rule12_bits: f64,
    next: u32,
) -> (u32, f64, bool) {
    let Some((_, cell)) = resolved else {
        return (rule12_selected, rule12_bits, false);
    };
    let total = f64::from(cell.total);
    let types = cell.entries.len() as f64;
    let lambda = total / (total + types);
    let pair_probability = f64::from(cell.count(next)) / total;
    let rule12_probability = (-rule12_bits).exp2();
    let mixed = lambda * pair_probability + (1.0 - lambda) * rule12_probability;
    (
        cell.argmax().unwrap_or(rule12_selected),
        -mixed.max(1e-300).log2(),
        true,
    )
}

/// #446 M2: how many bytes of the RIGHT graded code form the latent
/// class, unless `R4_LATENT_CLASS_DEPTH` overrides it.
///
/// One byte (at most 256 classes) is the default because the class
/// posterior has to be ESTIMABLE from the left key alone. The class
/// posterior at a deep left prefix is supported by the construction
/// records sharing that prefix — on the order of tens of records for a
/// full graded code on a 500k corpus — so a two-byte class (up to 65,536
/// values) would leave nearly every class a singleton and the posterior
/// would be pure noise. One byte keeps the per-left-key class support in
/// the right order of magnitude while still splitting the over-diluted
/// full-code cells.
const LATENT_CLASS_DEPTH_DEFAULT: usize = 1;

/// #446 M2 class-posterior smoothing: Jeffreys (add one half) mass over
/// the classes OBSERVED at the left key. Classes never observed there
/// have an empty emission cell and so contribute nothing to the mixture;
/// spreading mass onto them would only rescale the mixture. Adding half
/// a count to each observed class keeps a single-record class from
/// carrying a full unit of posterior mass.
const LATENT_CLASS_ALPHA: f64 = 0.5;

/// The pre-declared #446 M2 exit margin: two percentage points of top-1
/// agreement over the left-only Rule 1+2 baseline on the same held-out
/// population. Declared here, before any number was measured.
pub const LATENT_EXIT_MARGIN: f64 = 0.02;

/// #446 M2: the construction-time tables of the latent right-context
/// mixture, one level per left graded-prefix depth one through
/// [`STAGES`].
///
/// `emission[d]` is keyed by `pack_two_sided(left[..d], class)` and its
/// entries are next-token counts. `posterior[d]` is keyed by the left
/// prefix alone and its entries are CLASS counts, so the two levels
/// reuse the same compact sorted representation.
///
/// Both are built from the CONSTRUCTION split only. Reading them at
/// serving time touches the left key and the class posterior, never the
/// right context, so the mixture arm is causally legitimate.
struct LatentRightTable {
    class_depth: usize,
    emission: Vec<TwoSidedLevel>,
    posterior: Vec<TwoSidedLevel>,
}

impl LatentRightTable {
    /// The counterpart of [`TwoSidedTable::empty`] for a run that skipped
    /// [`ARM_GROUP_RIGHT_CONTEXT`]: every emission and posterior level
    /// empty, so `resolve_left` finds no supported prefix and the mixture
    /// arm is inert at every position.
    fn empty(class_depth: usize) -> Self {
        LatentRightTable {
            class_depth,
            emission: (0..=STAGES).map(|_| TwoSidedLevel::default()).collect(),
            posterior: (0..=STAGES).map(|_| TwoSidedLevel::default()).collect(),
        }
    }

    fn build(
        corpus: &Corpus,
        is_held_out: &[bool],
        left_codes: &[[u8; STAGES]],
        right_codes: &[([u8; STAGES], bool)],
        class_depth: usize,
    ) -> Self {
        let mut emission = Vec::with_capacity(STAGES + 1);
        let mut posterior = Vec::with_capacity(STAGES + 1);
        emission.push(TwoSidedLevel::default());
        posterior.push(TwoSidedLevel::default());
        for depth in 1..=STAGES {
            let mut emission_pairs: Vec<(TwoSidedKey, u32)> = Vec::new();
            let mut posterior_pairs: Vec<(TwoSidedKey, u32)> = Vec::new();
            for position in 0..corpus.n {
                if is_held_out[position] || !right_codes[position].1 {
                    continue;
                }
                let left = pack_prefix(&left_codes[position], depth);
                let class = pack_prefix(&right_codes[position].0, class_depth);
                emission_pairs.push((pack_two_sided(left, class), corpus.next[position]));
                posterior_pairs.push((TwoSidedKey::from(left), class));
            }
            emission.push(TwoSidedLevel::from_pairs(emission_pairs));
            posterior.push(TwoSidedLevel::from_pairs(posterior_pairs));
        }
        LatentRightTable {
            class_depth,
            emission,
            posterior,
        }
    }

    /// BACKOFF, serving side. Walk the left graded prefix from full
    /// depth down to depth one and take the DEEPEST populated left
    /// prefix whose class-posterior total clears the D4 EXCT support
    /// gate ([`score_runtime::EXCT_SUPPORT_MIN`]) — exactly the
    /// deepest-populated-prefix discipline the store's exact-context
    /// probe uses. `None` means no left prefix carries enough
    /// construction evidence; the arm is then INERT and the position
    /// falls through to the Rule 1+2 selection, which itself already
    /// backs off through the graph to the root cell, so the final
    /// fallback of the chain is the unigram-like root distribution.
    fn resolve_left(&self, left: &[u8; STAGES]) -> Option<(usize, u32, TwoSidedCell<'_>)> {
        for depth in (1..=STAGES).rev() {
            let prefix = pack_prefix(left, depth);
            if let Some(cell) = self.posterior[depth].get(TwoSidedKey::from(prefix)) {
                if cell.total >= EXCT_SUPPORT_MIN {
                    return Some((depth, prefix, cell));
                }
            }
        }
        None
    }

    /// The class posterior at a GIVEN depth and left prefix (used by the
    /// falsifier, which must read a foreign left key at the own arm's
    /// resolved depth so that only the posterior source varies).
    fn posterior_at(&self, depth: usize, prefix: u32) -> Option<TwoSidedCell<'_>> {
        self.posterior[depth].get(TwoSidedKey::from(prefix))
    }

    fn emission_at(&self, depth: usize, prefix: u32, class: u32) -> Option<TwoSidedCell<'_>> {
        self.emission[depth].get(pack_two_sided(prefix, class))
    }

    /// Mean number of latent classes per distinct FULL-depth left code
    /// on the construction split — the share of the M1 subdivision the
    /// coarse class retains.
    fn classes_per_full_left(&self) -> (usize, usize, f64) {
        let left_cells = self.posterior[STAGES].keys.len();
        let class_cells = self.emission[STAGES].keys.len();
        let ratio = if left_cells == 0 {
            0.0
        } else {
            class_cells as f64 / left_cells as f64
        };
        (left_cells, class_cells, ratio)
    }
}

/// #446 M2: the class posterior P(c | left) under the documented
/// Jeffreys rule, as a weight vector that sums to one.
fn latent_class_weights(cell: &TwoSidedCell<'_>) -> Vec<(u32, f64)> {
    let classes = cell.entries.len() as f64;
    let denominator = f64::from(cell.total) + LATENT_CLASS_ALPHA * classes;
    cell.entries
        .iter()
        .map(|&(class, count)| (class, (f64::from(count) + LATENT_CLASS_ALPHA) / denominator))
        .collect()
}

/// #446 M2: evaluate one latent arm — mix the class-conditional
/// emission cells at `(depth, prefix)` under the supplied class weights.
///
/// Each class contributes a Witten-Bell mixture of its emission cell
/// over the Rule 1+2 distribution, `lambda = total / (total + types)`,
/// exactly as the M1 two-sided arm does, and a class with no emission
/// cell at this left prefix contributes the Rule 1+2 distribution
/// itself. The weights sum to one and each per-class term is a proper
/// distribution, so the mixture is a proper code length on the same
/// scale as every other Gate C row.
///
/// Selection: where the arm is live the mixture PREEMPTS with the argmax
/// of the class-mixed evidence mass (ties to the lowest token id), the
/// same D4-style precedence the M1 arm applies; where it is inert the
/// arm IS Rule 1+2.
fn apply_latent_arm(
    table: &LatentRightTable,
    depth: usize,
    prefix: u32,
    weights: &[(u32, f64)],
    rule12_selected: u32,
    rule12_bits: f64,
    next: u32,
) -> (u32, f64, bool) {
    let rule12_probability = (-rule12_bits).exp2();
    let mut mass: BTreeMap<u32, f64> = BTreeMap::new();
    let mut mixed = 0f64;
    let mut live = false;
    for &(class, weight) in weights {
        if weight <= 0.0 {
            continue;
        }
        match table.emission_at(depth, prefix, class) {
            Some(cell) => {
                live = true;
                let total = f64::from(cell.total);
                let types = cell.entries.len() as f64;
                let lambda = total / (total + types);
                for &(token, count) in cell.entries {
                    *mass.entry(token).or_insert(0.0) += weight * lambda * f64::from(count) / total;
                }
                mixed += weight
                    * (lambda * f64::from(cell.count(next)) / total
                        + (1.0 - lambda) * rule12_probability);
            }
            None => mixed += weight * rule12_probability,
        }
    }
    if !live {
        return (rule12_selected, rule12_bits, false);
    }
    let selected = mass
        .iter()
        .fold(None::<(u32, f64)>, |best, (&token, &value)| match best {
            Some((_, high)) if high >= value => best,
            _ => Some((token, value)),
        })
        .map_or(rule12_selected, |(token, _)| token);
    (selected, -mixed.max(1e-300).log2(), true)
}

pub fn evaluate_gate_c(
    r4g1: &[u8],
    artifact_container: &[u8],
    artifacts: &compiler::Compiled,
    store: &Store,
    corpus: &Corpus,
    held_out: &[Observation],
    config: &ScoreConfig,
) -> Result<GateCOutcome, String> {
    // #471: every whole-corpus pass below announces its own cost, and the
    // arm groups this run was told not to build are resolved up front so the
    // skip is one decision read in one place.
    let skipped_arms = gate_c_skip_arms();
    let mut phases = PhaseLog::new("gate c phase");
    if skipped_arms.any() {
        eprintln!(
            "score: {GATE_C_SKIP_ARMS_ENV}={} — those arm groups are NOT EVALUATED on this run; \
             their rows are absent from the report, not zero",
            skipped_arms.names().join(",")
        );
    }
    let mut scorer_no_exct =
        GraphScorer::from_artifact(r4g1, None, config.root_top_b, config.exct_top_x)
            .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_no_exct.set_f_emissions(true);
    scorer_no_exct.set_scoring_variant(config.scoring_variant);
    scorer_no_exct.set_repetition_penalty_raw(config.repetition_penalty_raw);
    let mut scorer_with_exct = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )
    .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_with_exct.set_f_emissions(true);
    scorer_with_exct.set_scoring_variant(config.scoring_variant);
    scorer_with_exct.set_repetition_penalty_raw(config.repetition_penalty_raw);
    // Ablation scorers (issue #66): identical configs with ΔT emissions off
    // (the deployed default since the ablation decision).
    let mut scorer_no_exct_no_f =
        GraphScorer::from_artifact(r4g1, None, config.root_top_b, config.exct_top_x)
            .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_no_exct_no_f.set_scoring_variant(config.scoring_variant);
    scorer_no_exct_no_f.set_repetition_penalty_raw(config.repetition_penalty_raw);
    let mut scorer_with_exct_no_f = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )
    .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_with_exct_no_f.set_scoring_variant(config.scoring_variant);
    scorer_with_exct_no_f.set_repetition_penalty_raw(config.repetition_penalty_raw);
    let mut scorer_normalized = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )
    .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_normalized.set_scoring_variant(ScoringVariant::CloudSizeNormalized);
    scorer_normalized.set_repetition_penalty_raw(config.repetition_penalty_raw);
    let mut scorer_margin = GraphScorer::from_artifact(
        r4g1,
        Some(artifact_container),
        config.root_top_b,
        config.exct_top_x,
    )
    .expect("gate C: scorer rebuild from self-emitted artifact");
    scorer_margin.set_scoring_variant(ScoringVariant::MarginWeighted);
    scorer_margin.set_repetition_penalty_raw(config.repetition_penalty_raw);

    phases.mark("scorer construction");

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
    // #399 M2 fused-scorer accumulators (all positions + live slice).
    let mut hits_fwd = 0u64;
    let mut bits_fwd = 0f64;
    let mut fwd_live_positions = 0usize;
    let mut fwd_live_hits = 0u64;
    let mut fwd_live_bits = 0f64;
    let mut rule12_live_hits = 0u64;
    let mut rule12_live_bits = 0f64;
    // #399 B′ self-anchor and gated-self-anchor accumulators.
    let mut hits_fwd_self = 0u64;
    let mut bits_fwd_self = 0f64;
    let mut self_live_positions = 0usize;
    let mut self_live_hits = 0u64;
    let mut self_live_bits = 0f64;
    let mut rule12_self_live_hits = 0u64;
    let mut rule12_self_live_bits = 0f64;
    let mut hits_fwd_gated = 0u64;
    let mut bits_fwd_gated = 0f64;
    let mut gated_live_positions = 0usize;
    let mut gated_live_hits = 0u64;
    let mut gated_live_bits = 0f64;
    let mut rule12_gated_live_hits = 0u64;
    let mut rule12_gated_live_bits = 0f64;
    let mut anchor_hat_population = 0usize;
    let mut anchor_hat_correct_count = 0u64;
    // #399 falsifier 1: DRAFT-anchor gated-arm accumulators.
    let mut hits_fwd_draft = 0u64;
    let mut bits_fwd_draft = 0f64;
    let mut draft_live_positions = 0usize;
    let mut draft_live_hits = 0u64;
    let mut draft_live_bits = 0f64;
    let mut rule12_draft_live_hits = 0u64;
    let mut rule12_draft_live_bits = 0f64;
    // #399 rescue variant: STRICT-gated draft-arm accumulators.
    let mut hits_fwd_strict = 0u64;
    let mut bits_fwd_strict = 0f64;
    let mut strict_live_positions = 0usize;
    let mut strict_live_hits = 0u64;
    let mut strict_live_bits = 0f64;
    let mut rule12_strict_live_hits = 0u64;
    let mut rule12_strict_live_bits = 0f64;
    // #446 M1 two-sided arm accumulators (all positions + live slice),
    // and the foreign-right-key falsifier. NOT causal — infill/analysis.
    // #446 M1 cross-tabs. Local rather than written straight onto
    // `outcome.win_loss`, because they belong to the right-context arm group
    // and must be absent, not zero, when that group is skipped (#471).
    let mut twosided_win_loss = WinLoss::default();
    let mut twosided_shuffled_win_loss = WinLoss::default();
    let mut hits_twosided = 0u64;
    let mut bits_twosided = 0f64;
    let mut twosided_live_positions = 0usize;
    let mut twosided_live_hits = 0u64;
    let mut twosided_live_bits = 0f64;
    let mut rule12_twosided_live_hits = 0u64;
    let mut rule12_twosided_live_bits = 0f64;
    let mut twosided_depths = vec![0usize; STAGES + 1];
    // #446 M1 dilution slice: the ExactContext-status subpopulation.
    let mut twosided_exct_positions = 0usize;
    let mut twosided_exct_hits = 0u64;
    let mut twosided_exct_bits = 0f64;
    let mut rule12_exct_slice_hits = 0u64;
    let mut rule12_exct_slice_bits = 0f64;
    let mut twosided_exct_live = 0usize;
    let mut hits_twosided_shuffled = 0u64;
    let mut bits_twosided_shuffled = 0f64;
    let mut shuffled_live_positions = 0usize;
    let mut shuffled_live_hits = 0u64;
    let mut shuffled_live_bits = 0f64;
    let mut rule12_shuffled_live_hits = 0u64;
    let mut rule12_shuffled_live_bits = 0f64;
    // #446 M2 latent right-context mixture accumulators. The mix arm is
    // CAUSAL; the oracle arm is an upper bound and the shuffled-class
    // arm is its falsifier.
    let mut hits_latent = 0u64;
    let mut bits_latent_total = 0f64;
    let mut latent_live_positions = 0usize;
    let mut latent_live_hits = 0u64;
    let mut latent_live_bits = 0f64;
    let mut rule12_latent_live_hits = 0u64;
    let mut rule12_latent_live_bits = 0f64;
    let mut hits_latent_oracle = 0u64;
    let mut bits_latent_oracle_total = 0f64;
    let mut latent_oracle_live_positions = 0usize;
    let mut hits_latent_shuffled = 0u64;
    let mut bits_latent_shuffled_total = 0f64;
    let mut latent_shuffled_live_positions = 0usize;
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
    let mut status_bits_present = [0f64; 3];
    let mut status_bits_absent = [0f64; 3];
    let mut status_n_present = [0u64; 3];
    let mut status_n_absent = [0u64; 3];
    let mut status_offset_sum = [0i128; 3];
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
    let mut status_exact_ngram = 0usize;
    let mut status_exact_probe = 0usize;
    let mut status_ranks: [Vec<u32>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let gate_rotations = compiler::derive_rotations();
    // Held-out mask: shared by the #399 M2 forward-table build below and
    // the #390 analytic null further down.
    let mut is_held_out = vec![false; corpus.n];
    for observation in held_out {
        let position = observation.position as usize;
        if position < corpus.n {
            is_held_out[position] = true;
        }
    }

    // #399 M2: story-relative positions, and the forward-anchor channel
    // table built from the construction split (positions outside the
    // held-out set) under the #394 infill protocol — an anchor is a
    // position whose EMITTED token lands on a story position that is a
    // multiple of the stride; each anchor contributes its train-split
    // predecessors at every lookahead distance.
    let mut story_pos = Vec::with_capacity(corpus.n);
    {
        let (mut current_story, mut position_in_story) = (u32::MAX, 0u32);
        for &story in corpus.story.iter().take(corpus.n) {
            if story != current_story {
                current_story = story;
                position_in_story = 0;
            } else {
                position_in_story += 1;
            }
            story_pos.push(position_in_story);
        }
    }
    let mut fwd_table: BTreeMap<(usize, u32), BTreeMap<u32, u32>> = BTreeMap::new();
    for j in 0..corpus.n {
        if is_held_out[j] || !(story_pos[j] as usize + 1).is_multiple_of(M2_STRIDE) {
            continue;
        }
        for distance in 1..M2_STRIDE {
            if j >= distance
                && corpus.story[j - distance] == corpus.story[j]
                && !is_held_out[j - distance]
            {
                *fwd_table
                    .entry((distance, corpus.next[j]))
                    .or_default()
                    .entry(corpus.next[j - distance])
                    .or_default() += 1;
            }
        }
    }

    phases.mark("forward-anchor table (#399 M2)");

    // #446 M1: the two-sided (left graded prefix, right graded prefix)
    // table, built from the CONSTRUCTION split only (positions outside
    // the held-out set) under the same infill protocol as the forward
    // table above. NOT causal — the right key reads tokens after the
    // target, so this is an infill/analysis (A-mode) measurement or a
    // prospective construction-time signal, never a generation number.
    //
    // #471: this is the pass the arm-skip knob exists for. It is a whole
    // corpus `bundle_window_plain` + assign, it is NOT sidecar-cached (the
    // #469 lever-A sidecar keys the LEFT codes), and it scales with `n`
    // while a sampled evaluation's scored population does not — so its share
    // of a decision run's wall clock grows with the corpus. Measured at 60%
    // of a 10,000-position run on the 500k fixture.
    let right_codes = if skipped_arms.right_context {
        // The sentinel the two arms already treat as inert: no in-story
        // right window anywhere. `resolve` and `resolve_left` short-circuit
        // on it, so no row-level branch is needed and the scored rows take
        // the same code path they take at a story end today.
        vec![([0u8; STAGES], false); corpus.n]
    } else {
        derive_right_codes(artifacts, &gate_rotations, corpus)
    };
    phases.mark("right-context code pass (#446)");
    // #469 lever A: the left (causal) code of every record is the same
    // `code_plain` pass every other consumer runs. Serve it from the
    // κ-keyed sidecar when one verifies against BOTH the artifact κ and the
    // corpus κ; otherwise run exactly the derivation below and write it
    // back. Cache only — the codes are unchanged either way.
    let left_codes: Vec<[u8; STAGES]> =
        code_sidecar::corpus_codes_cached(artifacts, corpus, || {
            // #469 lever B: same pass, prepared tables. This runs only on a
            // sidecar miss, which is exactly the cold run worth speeding up.
            let tables = runtime::AssignTables::new(artifacts);
            (0..corpus.n)
                .into_par_iter()
                .map(|position| {
                    runtime::code_plain_with(&tables, artifacts, &gate_rotations, corpus, position)
                })
                .collect()
        });
    phases.mark("left code pass (sidecar or derive, #469 lever A)");
    let two_sided = if skipped_arms.right_context {
        TwoSidedTable::empty()
    } else {
        TwoSidedTable::build(corpus, &is_held_out, &left_codes, &right_codes)
    };
    phases.mark("two-sided table build (#446 M1)");
    // #446 M2: the latent right-context mixture tables, built from the
    // same construction split. CAUSALLY LEGITIMATE at serving: the right
    // context is observed here, during construction, and marginalized
    // away by the class posterior when the arm is read.
    let latent_class_depth = std::env::var("R4_LATENT_CLASS_DEPTH")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|depth| (1..=STAGES).contains(depth))
        .unwrap_or(LATENT_CLASS_DEPTH_DEFAULT);
    let latent = if skipped_arms.right_context {
        LatentRightTable::empty(latent_class_depth)
    } else {
        LatentRightTable::build(
            corpus,
            &is_held_out,
            &left_codes,
            &right_codes,
            latent_class_depth,
        )
    };
    phases.mark("latent right-context tables (#446 M2)");
    // #467: the positions actually SCORED. Full pass unless
    // `R4_GATE_C_SAMPLE` is set; the held-out mask above (and every table
    // built from it) always sees the complete held-out list, so sampling
    // narrows the measurement, never the split.
    let (scored, sample_size) = gate_c_sample(held_out);
    let held_positions: Vec<usize> = scored
        .iter()
        .map(|observation| observation.position as usize)
        .collect();
    let shuffle_rotation = held_positions.len() / 2;

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
        fwd_table: &fwd_table,
        story_pos: &story_pos,
        two_sided: &two_sided,
        latent: &latent,
        left_codes: &left_codes,
        right_codes: &right_codes,
        held_positions: &held_positions,
        shuffle_rotation,
    };
    // #390 analytic unigram null: the TRAIN next-token distribution
    // (all corpus positions outside the held-out set), add-one smoothed
    // over the compiled vocabulary.
    let vocab = (artifacts.token_codes.len() / compiler::STAGES) as u64;
    let mut unigram_counts: BTreeMap<u32, u64> = BTreeMap::new();
    let mut train_positions = 0usize;
    for (position, held) in is_held_out.iter().enumerate().take(corpus.n) {
        if !held {
            *unigram_counts.entry(corpus.next[position]).or_insert(0) += 1;
            train_positions += 1;
        }
    }
    let unigram_total: u64 = unigram_counts.values().sum();
    let unigram_train_argmax = unigram_counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(&token, _)| token)
        .unwrap_or(0);
    let unigram_bits = |token: u32| -> f64 {
        let count = unigram_counts.get(&token).copied().unwrap_or(0);
        let p = (count as f64 + 1.0) / (unigram_total as f64 + vocab as f64);
        -p.log2()
    };
    let mut null_hits_all = 0u64;
    let mut null_bits_all = 0f64;
    let mut null_hits_generalization = 0u64;
    let mut null_bits_generalization = 0f64;
    let mut generalization_positions = 0u64;

    phases.mark("unigram null over the construction split (#390)");

    // Each held-out position is independent. Collect compact rows in Rayon;
    // reduce them in input order below so floating-point totals and all
    // report bytes retain the serial implementation's determinism.
    let rows: Vec<GateCRow> = scored
        .par_iter()
        .enumerate()
        .map(|(index, observation)| evaluate_gate_c_row(index, observation, &context))
        .collect::<Result<Vec<_>, _>>()?;
    phases.mark(&format!("scoring {} positions", scored.len()));
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
        if row.teacher_in_candidates {
            status_bits_present[row.status_index] += row.status_bits;
            status_n_present[row.status_index] += 1;
        } else {
            status_bits_absent[row.status_index] += row.status_bits;
            status_n_absent[row.status_index] += 1;
        }
        status_offset_sum[row.status_index] += i128::from(row.transition_offset_raw);
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
        match row.exact_context_source {
            Some(ExactContextSource::NgramRow) => status_exact_ngram += 1,
            Some(ExactContextSource::ExctProbe) => status_exact_probe += 1,
            None => {}
        }
        let null_hit = u64::from(row.next == unigram_train_argmax);
        let null_bits = unigram_bits(row.next);
        null_hits_all += null_hit;
        null_bits_all += null_bits;
        if row.status_index != 0 {
            generalization_positions += 1;
            null_hits_generalization += null_hit;
            null_bits_generalization += null_bits;
        }
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
        hits_fwd += u64::from(row.hit_fwd);
        bits_fwd += row.bits_fwd;
        accumulate_win_loss(
            &mut outcome.win_loss.fwd_vs_rule12,
            row.hit_fwd,
            row.hits[2],
        );
        if row.fwd_live {
            fwd_live_positions += 1;
            fwd_live_hits += u64::from(row.hit_fwd);
            fwd_live_bits += row.bits_fwd;
            rule12_live_hits += u64::from(row.hits[2]);
            rule12_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut outcome.win_loss.fwd_vs_rule12_live,
                row.hit_fwd,
                row.hits[2],
            );
        }
        hits_fwd_self += u64::from(row.hit_fwd_self);
        bits_fwd_self += row.bits_fwd_self;
        if row.fwd_self_live {
            self_live_positions += 1;
            self_live_hits += u64::from(row.hit_fwd_self);
            self_live_bits += row.bits_fwd_self;
            rule12_self_live_hits += u64::from(row.hits[2]);
            rule12_self_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut outcome.win_loss.fwd_self_vs_rule12_live,
                row.hit_fwd_self,
                row.hits[2],
            );
        }
        hits_fwd_gated += u64::from(row.hit_fwd_gated);
        bits_fwd_gated += row.bits_fwd_gated;
        if row.fwd_gated_live {
            gated_live_positions += 1;
            gated_live_hits += u64::from(row.hit_fwd_gated);
            gated_live_bits += row.bits_fwd_gated;
            rule12_gated_live_hits += u64::from(row.hits[2]);
            rule12_gated_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut outcome.win_loss.fwd_gated_vs_rule12_live,
                row.hit_fwd_gated,
                row.hits[2],
            );
        }
        hits_fwd_draft += u64::from(row.hit_fwd_draft);
        bits_fwd_draft += row.bits_fwd_draft;
        if row.fwd_draft_live {
            draft_live_positions += 1;
            draft_live_hits += u64::from(row.hit_fwd_draft);
            draft_live_bits += row.bits_fwd_draft;
            rule12_draft_live_hits += u64::from(row.hits[2]);
            rule12_draft_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut outcome.win_loss.fwd_draft_vs_rule12_live,
                row.hit_fwd_draft,
                row.hits[2],
            );
        }
        hits_fwd_strict += u64::from(row.hit_fwd_strict);
        bits_fwd_strict += row.bits_fwd_strict;
        if row.fwd_strict_live {
            strict_live_positions += 1;
            strict_live_hits += u64::from(row.hit_fwd_strict);
            strict_live_bits += row.bits_fwd_strict;
            rule12_strict_live_hits += u64::from(row.hits[2]);
            rule12_strict_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut outcome.win_loss.fwd_strict_vs_rule12_live,
                row.hit_fwd_strict,
                row.hits[2],
            );
        }
        hits_twosided += u64::from(row.hit_twosided);
        bits_twosided += row.bits_twosided;
        twosided_depths[row.twosided_depth] += 1;
        // Dilution slice: status index 0 is ExactContext.
        if row.status_index == 0 {
            twosided_exct_positions += 1;
            twosided_exct_hits += u64::from(row.hit_twosided);
            twosided_exct_bits += row.bits_twosided;
            rule12_exct_slice_hits += u64::from(row.hits[2]);
            rule12_exct_slice_bits += row.bits[2];
            if row.twosided_live {
                twosided_exct_live += 1;
            }
        }
        if row.twosided_live {
            twosided_live_positions += 1;
            twosided_live_hits += u64::from(row.hit_twosided);
            twosided_live_bits += row.bits_twosided;
            rule12_twosided_live_hits += u64::from(row.hits[2]);
            rule12_twosided_live_bits += row.bits[2];
            accumulate_win_loss(&mut twosided_win_loss, row.hit_twosided, row.hits[2]);
        }
        hits_twosided_shuffled += u64::from(row.hit_twosided_shuffled);
        bits_twosided_shuffled += row.bits_twosided_shuffled;
        if row.twosided_shuffled_live {
            shuffled_live_positions += 1;
            shuffled_live_hits += u64::from(row.hit_twosided_shuffled);
            shuffled_live_bits += row.bits_twosided_shuffled;
            rule12_shuffled_live_hits += u64::from(row.hits[2]);
            rule12_shuffled_live_bits += row.bits[2];
            accumulate_win_loss(
                &mut twosided_shuffled_win_loss,
                row.hit_twosided_shuffled,
                row.hits[2],
            );
        }
        hits_latent += u64::from(row.hit_latent);
        bits_latent_total += row.bits_latent;
        if row.latent_live {
            latent_live_positions += 1;
            latent_live_hits += u64::from(row.hit_latent);
            latent_live_bits += row.bits_latent;
            rule12_latent_live_hits += u64::from(row.hits[2]);
            rule12_latent_live_bits += row.bits[2];
        }
        hits_latent_oracle += u64::from(row.hit_latent_oracle);
        bits_latent_oracle_total += row.bits_latent_oracle;
        latent_oracle_live_positions += usize::from(row.latent_oracle_live);
        hits_latent_shuffled += u64::from(row.hit_latent_shuffled);
        bits_latent_shuffled_total += row.bits_latent_shuffled;
        latent_shuffled_live_positions += usize::from(row.latent_shuffled_live);
        if let Some(correct) = row.anchor_hat_correct {
            anchor_hat_population += 1;
            anchor_hat_correct_count += u64::from(correct);
        }
        outcome.witness_replays += usize::from(row.witness_replayed);
        outcome.witness_replay_failures += usize::from(row.witness_replay_failed);
    }
    let n = scored.len();
    if n == 0 {
        return Err("held-out split is empty; cannot evaluate".to_owned());
    }
    let nf = n as f64;
    let metrics = |hits: u64, bits: f64| {
        let top1 = hits as f64 / nf;
        GateCMetrics {
            positions: n,
            top1_agreement: top1,
            bits_per_token: bits / nf,
            positions_sampled: sample_size,
            standard_error: binomial_standard_error(top1, n),
        }
    };
    outcome.legacy_sum = metrics(hits_legacy, bits_legacy);
    outcome.rule1_chain = metrics(hits_rule1, bits_rule1);
    outcome.rule12_precedence = metrics(hits_rule12, bits_rule12);
    outcome.rule1_chain_no_f = metrics(hits_rule1_no_f, bits_rule1_no_f);
    outcome.rule12_precedence_no_f = metrics(hits_rule12_no_f, bits_rule12_no_f);
    outcome.rule12_cloud_size_normalized = metrics(hits_normalized, bits_normalized);
    outcome.rule12_margin_weighted = metrics(hits_margin, bits_margin);
    outcome.tla3_baseline = metrics(hits_baseline, bits_baseline);
    outcome.rule12_fwd_fused = metrics(hits_fwd, bits_fwd);
    outcome.rule12_fwd_self_fused = metrics(hits_fwd_self, bits_fwd_self);
    outcome.rule12_fwd_gated_fused = metrics(hits_fwd_gated, bits_fwd_gated);
    outcome.rule12_fwd_draft_fused = metrics(hits_fwd_draft, bits_fwd_draft);
    outcome.rule12_fwd_strict_fused = metrics(hits_fwd_strict, bits_fwd_strict);
    let live_metrics = |hits: u64, bits: f64, live_n: usize| {
        let top1 = if live_n == 0 {
            0.0
        } else {
            hits as f64 / live_n as f64
        };
        GateCMetrics {
            positions: live_n,
            top1_agreement: top1,
            bits_per_token: if live_n == 0 {
                0.0
            } else {
                bits / live_n as f64
            },
            positions_sampled: sample_size,
            standard_error: binomial_standard_error(top1, live_n),
        }
    };
    outcome.rule12_fwd_fused_live = live_metrics(fwd_live_hits, fwd_live_bits, fwd_live_positions);
    outcome.rule12_on_fwd_live =
        live_metrics(rule12_live_hits, rule12_live_bits, fwd_live_positions);
    outcome.rule12_fwd_self_fused_live =
        live_metrics(self_live_hits, self_live_bits, self_live_positions);
    outcome.rule12_on_fwd_self_live = live_metrics(
        rule12_self_live_hits,
        rule12_self_live_bits,
        self_live_positions,
    );
    outcome.rule12_fwd_gated_fused_live =
        live_metrics(gated_live_hits, gated_live_bits, gated_live_positions);
    outcome.rule12_on_fwd_gated_live = live_metrics(
        rule12_gated_live_hits,
        rule12_gated_live_bits,
        gated_live_positions,
    );
    outcome.rule12_fwd_draft_fused_live =
        live_metrics(draft_live_hits, draft_live_bits, draft_live_positions);
    outcome.rule12_on_fwd_draft_live = live_metrics(
        rule12_draft_live_hits,
        rule12_draft_live_bits,
        draft_live_positions,
    );
    outcome.rule12_fwd_strict_fused_live =
        live_metrics(strict_live_hits, strict_live_bits, strict_live_positions);
    outcome.rule12_on_fwd_strict_live = live_metrics(
        rule12_strict_live_hits,
        rule12_strict_live_bits,
        strict_live_positions,
    );
    // #446 M1 + M2: the right-context arm group. Assembled as a whole and
    // attached as a whole, so a skipped run leaves `None` rather than a
    // half-populated set of arms nobody can tell apart from measured zeros.
    outcome.skipped_arm_groups = skipped_arms.names();
    outcome.right_context_arms = if skipped_arms.right_context {
        None
    } else {
        let mut arms = RightContextArms {
            rule12_twosided: metrics(hits_twosided, bits_twosided),
            rule12_twosided_live: live_metrics(
                twosided_live_hits,
                twosided_live_bits,
                twosided_live_positions,
            ),
            rule12_on_twosided_live: live_metrics(
                rule12_twosided_live_hits,
                rule12_twosided_live_bits,
                twosided_live_positions,
            ),
            rule12_twosided_shuffled: metrics(hits_twosided_shuffled, bits_twosided_shuffled),
            rule12_twosided_shuffled_live: live_metrics(
                shuffled_live_hits,
                shuffled_live_bits,
                shuffled_live_positions,
            ),
            rule12_on_twosided_shuffled_live: live_metrics(
                rule12_shuffled_live_hits,
                rule12_shuffled_live_bits,
                shuffled_live_positions,
            ),
            rule12_twosided_depths: twosided_depths,
            rule12_twosided_exct_slice: live_metrics(
                twosided_exct_hits,
                twosided_exct_bits,
                twosided_exct_positions,
            ),
            rule12_on_twosided_exct_slice: live_metrics(
                rule12_exct_slice_hits,
                rule12_exct_slice_bits,
                twosided_exct_positions,
            ),
            rule12_twosided_exct_slice_live: twosided_exct_live,
            // #446 M2: the latent mixture rows, plus the pre-declared exit rule.
            rule12_latent_mix: metrics(hits_latent, bits_latent_total),
            rule12_latent_mix_live: live_metrics(
                latent_live_hits,
                latent_live_bits,
                latent_live_positions,
            ),
            rule12_on_latent_mix_live: live_metrics(
                rule12_latent_live_hits,
                rule12_latent_live_bits,
                latent_live_positions,
            ),
            rule12_latent_oracle: metrics(hits_latent_oracle, bits_latent_oracle_total),
            rule12_latent_shuffled: metrics(hits_latent_shuffled, bits_latent_shuffled_total),
            latent_oracle_live_positions,
            latent_shuffled_live_positions,
            latent_class_depth,
            twosided_vs_rule12_live: twosided_win_loss,
            twosided_shuffled_vs_rule12_live: twosided_shuffled_win_loss,
            ..RightContextArms::default()
        };
        let (latent_left_cells, latent_class_cells, latent_ratio) = latent.classes_per_full_left();
        arms.latent_full_left_cells = latent_left_cells;
        arms.latent_full_class_cells = latent_class_cells;
        arms.latent_classes_per_full_left = latent_ratio;
        {
            let baseline = outcome.rule12_precedence.top1_agreement;
            let mix = arms.rule12_latent_mix.top1_agreement;
            let oracle = arms.rule12_latent_oracle.top1_agreement;
            arms.latent_headroom_fraction = if oracle > baseline {
                (mix - baseline) / (oracle - baseline)
            } else {
                0.0
            };
            arms.latent_exit_rule_met = mix - baseline >= LATENT_EXIT_MARGIN
                && mix > arms.rule12_latent_shuffled.top1_agreement;
        }
        let (full_left_cells, full_pair_keys) = two_sided.full_depth_subdivision();
        arms.twosided_full_left_cells = full_left_cells;
        arms.twosided_full_pair_keys = full_pair_keys;
        arms.twosided_keys_per_full_left = if full_left_cells == 0 {
            0.0
        } else {
            full_pair_keys as f64 / full_left_cells as f64
        };
        Some(arms)
    };
    outcome.anchor_hat_population = anchor_hat_population;
    outcome.anchor_hat_correct = anchor_hat_correct_count as usize;
    outcome.anchor_hat_accuracy = if anchor_hat_population == 0 {
        0.0
    } else {
        anchor_hat_correct_count as f64 / anchor_hat_population as f64
    };
    outcome.rule12_status_counts = StatusCounts {
        exact_context: status_positions[0],
        graph: status_positions[1],
        novel: status_positions[2],
        exact_context_ngram: status_exact_ngram,
        exact_context_probe: status_exact_probe,
    };
    outcome.rule12_generalization = {
        let positions = status_positions[1] + status_positions[2];
        let hits = status_hits[1] + status_hits[2];
        let bits = status_bits[1] + status_bits[2];
        let nf = (positions as f64).max(1.0);
        let top1 = hits as f64 / nf;
        GateCMetrics {
            positions,
            top1_agreement: top1,
            bits_per_token: bits / nf,
            positions_sampled: sample_size,
            standard_error: binomial_standard_error(top1, positions),
        }
    };
    outcome.nulls = GateCNulls {
        unigram_train_argmax,
        unigram_null_top1_all: null_hits_all as f64 / nf,
        unigram_null_bits_all: null_bits_all / nf,
        unigram_null_top1_generalization: null_hits_generalization as f64
            / (generalization_positions as f64).max(1.0),
        unigram_null_bits_generalization: null_bits_generalization
            / (generalization_positions as f64).max(1.0),
        train_positions,
        held_out_positions: n,
    };
    // #467: the sampling provenance of the whole table.
    outcome.held_out_population = held_out.len();
    outcome.positions_sampled = sample_size;
    outcome.rule12_exct_probe_levels = exct_level_positions;
    outcome.rule12_exct_probe_absent = exct_probe_absent;
    outcome.rule12_exct_full_depth_supported = exct_full_depth_supported;
    let per_status = |index: usize| {
        let positions = status_positions[index];
        if positions == 0 {
            return GateCMetrics::default();
        }
        let denom = positions as f64;
        let top1 = status_hits[index] as f64 / denom;
        GateCMetrics {
            positions,
            top1_agreement: top1,
            bits_per_token: status_bits[index] / denom,
            positions_sampled: sample_size,
            standard_error: binomial_standard_error(top1, positions),
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
            bits_teacher_present: if status_n_present[index] == 0 {
                0.0
            } else {
                status_bits_present[index] / status_n_present[index] as f64
            },
            bits_teacher_absent: if status_n_absent[index] == 0 {
                0.0
            } else {
                status_bits_absent[index] / status_n_absent[index] as f64
            },
            positions_teacher_present: status_n_present[index] as usize,
            positions_teacher_absent: status_n_absent[index] as usize,
            mean_transition_offset_nats: (status_offset_sum[index] as f64 / denom) / 65536.0,
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

    for obs in scored.iter() {
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

    phases.mark("reduction, per-status rollups and replay probes");
    phases.total();
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
    /// Is the teacher token present in the candidate set at all?
    ///
    /// `outcome_bits` charges uncovered vocabulary at `w_floor =
    /// weight(root_floor)` while `max_s` is taken over CANDIDATE scores, and
    /// candidate scores include `transition_offset` -- added to every candidate,
    /// hence rank-neutral and invisible to every argmax measurement, but the
    /// floor is not shifted with them. A positive offset therefore collapses the
    /// uncovered-mass weight and charges an enormous penalty exactly on the
    /// positions where the teacher is ABSENT from candidates. Split the bits by
    /// that condition to test whether the graph slice's ~127 bits/token is a
    /// model error or this accounting artifact.
    teacher_in_candidates: bool,
    /// Raw `transition_offset` applied to every candidate this position.
    transition_offset_raw: i64,
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
    /// Which mechanism resolved the rule12 selection when its status is
    /// ExactContext (#362 attribution; `None` otherwise).
    exact_context_source: Option<ExactContextSource>,
    /// The recorded next token at this position (#390 null accounting).
    next: u32,
    /// #399 M2: fused (Rule 1+2 × forward-anchor) selection hit.
    hit_fwd: bool,
    /// #399 M2: bits/token under the fused distribution (= `bits[2]`
    /// where the channel is inert).
    bits_fwd: f64,
    /// #399 M2: the channel was live at this position.
    fwd_live: bool,
    /// #399 B′: the same triplet for the SELF-anchor arm (the engine's
    /// own prediction at the anchor position supplies the key).
    hit_fwd_self: bool,
    bits_fwd_self: f64,
    fwd_self_live: bool,
    /// #399 B′: the confidence-gated self-anchor arm (prediction trusted
    /// only where it resolved as ExactContext).
    hit_fwd_gated: bool,
    bits_fwd_gated: f64,
    fwd_gated_live: bool,
    /// #399 B′: whether the predicted anchor equals the true anchor
    /// (`None` off the anchor-reachable population).
    anchor_hat_correct: Option<bool>,
    /// #399 falsifier 1: the DRAFT-anchor gated arm — the anchor is
    /// predicted from the engine's own greedy draft (its pass-1 token at
    /// this position plus drafted continuations), not from teacher-forced
    /// corpus context, and is trusted only where the draft's final step
    /// resolved as ExactContext.
    hit_fwd_draft: bool,
    bits_fwd_draft: f64,
    fwd_draft_live: bool,
    /// #399 rescue variant: the STRICT-gated draft arm — same draft
    /// loop, anchor, and fusion as the draft arm, but the gate requires
    /// EVERY intermediate greedy step (final anchor step included) to
    /// have resolved as ExactContext, not just the final one.
    hit_fwd_strict: bool,
    bits_fwd_strict: f64,
    fwd_strict_live: bool,
    /// #446 M1: the two-sided arm's triplet, plus the depth its pair
    /// resolved at (0 = inert, fell through to Rule 1+2). NOT causal —
    /// infill/analysis only.
    hit_twosided: bool,
    bits_twosided: f64,
    twosided_live: bool,
    twosided_depth: usize,
    /// #446 M1 falsifier: the same triplet with a foreign right key.
    hit_twosided_shuffled: bool,
    bits_twosided_shuffled: f64,
    twosided_shuffled_live: bool,
    /// #446 M2: the latent right-context mixture arm's triplet. CAUSAL —
    /// only the left key is read at this position.
    hit_latent: bool,
    bits_latent: f64,
    latent_live: bool,
    /// #446 M2 upper bound: the same tables with the TRUE right class
    /// supplied at evaluation time. NOT causal — never quotable.
    hit_latent_oracle: bool,
    bits_latent_oracle: f64,
    latent_oracle_live: bool,
    /// #446 M2 falsifier: the class posterior lifted from a FOREIGN left
    /// key at the same resolved depth.
    hit_latent_shuffled: bool,
    bits_latent_shuffled: f64,
    latent_shuffled_live: bool,
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
    /// #399 M2: (lookahead distance, next-anchor token) → counts of the
    /// token emitted `distance` positions before that anchor, built from
    /// the construction split under the #394 infill protocol.
    fwd_table: &'a BTreeMap<(usize, u32), BTreeMap<u32, u32>>,
    /// Story-relative position of every corpus record (0-based).
    story_pos: &'a [u32],
    /// #446 M1: the two-sided (left prefix, right prefix) evidence table
    /// built from the construction split. NOT causal — infill/analysis.
    two_sided: &'a TwoSidedTable,
    /// #446 M2: the latent right-context mixture tables (emission and
    /// class posterior), built from the construction split. Read with
    /// the LEFT key only at serving — causally legitimate.
    latent: &'a LatentRightTable,
    /// Left graded code of every corpus position. Needed by the #446 M2
    /// falsifier, which lifts a class posterior from a FOREIGN left key.
    left_codes: &'a [[u8; STAGES]],
    /// #446 M1: the right graded code of every corpus position, with a
    /// flag for "an in-story right window existed here".
    right_codes: &'a [([u8; STAGES], bool)],
    /// #446 M1 falsifier: held-out corpus positions in evaluation order,
    /// and the fixed rotation applied to pick a FOREIGN right key.
    held_positions: &'a [usize],
    shuffle_rotation: usize,
}

/// The story-bounded recent-token window ending at `position`'s input
/// token (#375): up to `max_len` tokens of `corpus.input`, never
/// crossing a story boundary — the same boundary rule
/// [`compile_context_rows`] applies when building trigram keys, so a
/// window-mode probe sees exactly the keys the compiler could have
/// built for that position.
pub fn story_bounded_window(corpus: &Corpus, position: usize, max_len: usize) -> &[u32] {
    let story = corpus.story[position];
    let lo = position.saturating_sub(max_len.saturating_sub(1));
    let mut start = position;
    while start > lo && corpus.story[start - 1] == story {
        start -= 1;
    }
    &corpus.input[start..=position]
}

/// #399 M2/B′: fuse one Rule 1+2 outcome with one forward row by the
/// measured product law in ln units (ScoreQ carries ln × 2⁻¹⁶): Rule 1+2
/// absentees enter at the offset-shifted root floor, forward absentees
/// at the smoothing floor — both floors on one scale (the #387
/// accounting rule). Returns (selected, bits/token of `next`); selection
/// keeps the canonical lowest-token tie-break.
fn fuse_forward_arm(
    scorer: &GraphScorer,
    rule12: &ScoreOutcome,
    fwd_row: &BTreeMap<u32, u32>,
    next: u32,
) -> (u32, f64) {
    let vocab_f = f64::from(scorer.vocab());
    let total: f64 = fwd_row.values().map(|&count| f64::from(count)).sum();
    let smooth = total + vocab_f * 0.5;
    let ln_fwd = |token: u32| -> f64 {
        ((f64::from(fwd_row.get(&token).copied().unwrap_or(0)) + 0.5) / smooth).ln()
    };
    let floor_raw = scorer
        .root_floor()
        .raw()
        .saturating_add(rule12.witness.transition_offset.raw());
    let ln12_floor = f64::from(floor_raw) / 65536.0;
    let mut fused: BTreeMap<u32, f64> = BTreeMap::new();
    for &(token, score) in &rule12.candidates {
        fused.insert(token, f64::from(score.raw()) / 65536.0 + ln_fwd(token));
    }
    for &token in fwd_row.keys() {
        fused.entry(token).or_insert(ln12_floor + ln_fwd(token));
    }
    let (mut best_token, mut best_score) = (u32::MAX, f64::NEG_INFINITY);
    for (&token, &score) in &fused {
        if score > best_score {
            best_token = token;
            best_score = score;
        }
    }
    let ln_floor_absent = ln12_floor + (0.5 / smooth).ln();
    let max_score = best_score.max(ln_floor_absent);
    let mut weight_sum = 0f64;
    let mut weight_next = None;
    for (&token, &score) in &fused {
        let weight = (score - max_score).exp();
        weight_sum += weight;
        if token == next {
            weight_next = Some(weight);
        }
    }
    let weight_floor = (ln_floor_absent - max_score).exp();
    let uncovered = (scorer.vocab() as usize).saturating_sub(fused.len());
    weight_sum += uncovered as f64 * weight_floor;
    let weight = weight_next.unwrap_or(weight_floor).max(1e-300);
    (
        best_token,
        (weight_sum / weight).ln() / std::f64::consts::LN_2,
    )
}

/// Slide one token into the fixed draft window / recent-token buffers of
/// the #399 falsifier-1 draft loop (mirrors the
/// `generate_greedy_repetition_rate` sliding rule, with the short-seed
/// window growing; no allocation).
fn draft_push(
    window: &mut [u32; compiler::WINDOW],
    w_len: &mut usize,
    recent: &mut [u32; 32],
    recent_len: &mut usize,
    track_recent: bool,
    token: u32,
) {
    if *w_len < compiler::WINDOW {
        window[*w_len] = token;
        *w_len += 1;
    } else {
        window.copy_within(1.., 0);
        window[compiler::WINDOW - 1] = token;
    }
    if track_recent {
        if *recent_len == 32 {
            recent.copy_within(1.., 0);
            *recent_len = 31;
        }
        recent[*recent_len] = token;
        *recent_len += 1;
    }
}

fn evaluate_gate_c_row(
    index: usize,
    observation: &Observation,
    context: &GateCContext<'_>,
) -> Result<GateCRow, String> {
    let position = observation.position as usize;
    let teacher_argmax = context.corpus.t_argmax[position];
    let next = context.corpus.next[position];
    let window: &[u32] = if context.config.gate_c_context_window {
        story_bounded_window(context.corpus, position, 32)
    } else {
        &[]
    };
    // #469 lever A: `left_codes[position]` IS `code_plain` at this
    // position — the same whole-corpus pass, already materialized (and now
    // sidecar-served) by the caller. Read it instead of recomputing; the
    // fallback keeps the old derivation for an out-of-range position.
    let code = match context.left_codes.get(position) {
        Some(code) => *code,
        None => runtime::code_plain(
            context.artifacts,
            context.gate_rotations,
            context.corpus,
            position,
        ),
    };

    let legacy = context
        .scorer_with_exct
        .score_candidates_legacy(&observation.sig)
        .expect("gate C: scoring self-produced sig");
    let rule1 = context
        .scorer_no_exct
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
    let rule12 = context
        .scorer_with_exct
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
    let rule1_no_f = context
        .scorer_no_exct_no_f
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
    let rule12_no_f = context
        .scorer_with_exct_no_f
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
    let normalized = context
        .scorer_normalized
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
    let margin = context
        .scorer_margin
        .score_candidates_coded(&observation.sig, Some(&code), window)
        .expect("gate C: scoring self-produced sig");
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
        outcome_bits(
            context.scorer_with_exct,
            &legacy.candidates,
            next,
            ScoreQ::ZERO,
        ),
        outcome_bits(
            context.scorer_no_exct,
            &rule1.candidates,
            next,
            rule1.witness.transition_offset,
        ),
        outcome_bits(
            context.scorer_with_exct,
            &rule12.candidates,
            next,
            rule12.witness.transition_offset,
        ),
        outcome_bits(
            context.scorer_no_exct_no_f,
            &rule1_no_f.candidates,
            next,
            rule1_no_f.witness.transition_offset,
        ),
        outcome_bits(
            context.scorer_with_exct_no_f,
            &rule12_no_f.candidates,
            next,
            rule12_no_f.witness.transition_offset,
        ),
        outcome_bits(
            context.scorer_normalized,
            &normalized.candidates,
            next,
            normalized.witness.transition_offset,
        ),
        outcome_bits(
            context.scorer_margin,
            &margin.candidates,
            next,
            margin.witness.transition_offset,
        ),
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
        let penalty = i64::from(context.config.repetition_penalty_raw);
        let root_term = i64::from(root_raw) + if penalized { penalty } else { 0 };
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
            let raw = root_raw
                + if penalized {
                    context.config.repetition_penalty_raw
                } else {
                    0
                };
            (token, ScoreQ::from_raw(raw))
        })
        .collect();
    let bits_root_only = outcome_bits(
        context.scorer_with_exct,
        &root_only_candidates,
        next,
        ScoreQ::ZERO,
    );

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
            let score = i64::from(root_raw)
                + scaled
                + if penalized {
                    i64::from(context.config.repetition_penalty_raw)
                } else {
                    0
                };
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
    // ---- #399 M2 / B′: forward-anchor channel (instrumentation only) ----
    // Three arms sharing one fusion law: TRUE anchor (the corpus token at
    // the next anchor position — the infill-protocol upper bound), SELF
    // anchor (the engine's OWN Rule 1+2 prediction at that position — the
    // two-pass serving question), and GATED self anchor (the prediction is
    // trusted only where it resolved as ExactContext — the confidence-
    // gated variant of the M0 law).
    let target_pos = context.story_pos[position] as usize + 1;
    let mut fwd_live = false;
    let mut fwd_selected = rule12.selected;
    let mut bits_fwd = bits[2];
    let mut fwd_self_live = false;
    let mut fwd_self_selected = rule12.selected;
    let mut bits_fwd_self = bits[2];
    let mut fwd_gated_live = false;
    let mut fwd_gated_selected = rule12.selected;
    let mut bits_fwd_gated = bits[2];
    let mut anchor_hat_correct = None;
    let mut fwd_draft_live = false;
    let mut fwd_draft_selected = rule12.selected;
    let mut bits_fwd_draft = bits[2];
    let mut fwd_strict_live = false;
    let mut fwd_strict_selected = rule12.selected;
    let mut bits_fwd_strict = bits[2];
    if !target_pos.is_multiple_of(M2_STRIDE) {
        let lookahead = target_pos.next_multiple_of(M2_STRIDE) - target_pos;
        let anchor_position = position + lookahead;
        if anchor_position < context.corpus.n
            && context.corpus.story[anchor_position] == context.corpus.story[position]
        {
            let anchor = context.corpus.next[anchor_position];
            if let Some(fwd_row) = context.fwd_table.get(&(lookahead, anchor)) {
                fwd_live = true;
                let (selected, fused_bits) =
                    fuse_forward_arm(context.scorer_with_exct, &rule12, fwd_row, next);
                fwd_selected = selected;
                bits_fwd = fused_bits;
            }
            // B′: the engine's own prediction at the anchor position,
            // teacher-forced context (isolates anchor-token noise from
            // draft-context drift — the first-order two-pass question).
            let anchor_bundle = runtime::bundle_plain(
                context.artifacts,
                context.gate_rotations,
                context.corpus,
                anchor_position,
            );
            let anchor_sig = runtime::sig_plain(context.artifacts, &anchor_bundle);
            let anchor_code = runtime::assign_for_bundle(context.artifacts, &anchor_bundle);
            let anchor_window: &[u32] = if context.config.gate_c_context_window {
                story_bounded_window(context.corpus, anchor_position, 32)
            } else {
                &[]
            };
            let anchor_outcome = context
                .scorer_with_exct
                .score_candidates_coded(&anchor_sig, Some(&anchor_code), anchor_window)
                .expect("gate C: scoring self-produced sig");
            let anchor_hat = anchor_outcome.selected;
            anchor_hat_correct = Some(anchor_hat == anchor);
            if let Some(fwd_row) = context.fwd_table.get(&(lookahead, anchor_hat)) {
                fwd_self_live = true;
                let (selected, fused_bits) =
                    fuse_forward_arm(context.scorer_with_exct, &rule12, fwd_row, next);
                fwd_self_selected = selected;
                bits_fwd_self = fused_bits;
                if anchor_outcome.witness.status == ScoreStatus::ExactContext {
                    fwd_gated_live = true;
                    fwd_gated_selected = selected;
                    bits_fwd_gated = fused_bits;
                }
            }
            // Falsifier 1 (draft drift): the gated arm above predicts
            // the anchor from TEACHER-FORCED context (true corpus tokens
            // up to the anchor position). A real two-pass generation
            // only has its own pass-1 DRAFT: seed the window with the
            // corpus tokens up to this position, append the engine's own
            // Rule 1+2 token here, then greedily draft `lookahead`
            // steps with `score_candidates_coded` — the final step's
            // token is the draft anchor and its witness status is the
            // gate. Fusion law and comparator population are identical
            // to the gated arm, so any spread between the two arms is
            // pure draft-context drift.
            let mut draft_window = [0u32; compiler::WINDOW];
            let window_seed = story_bounded_window(context.corpus, position, compiler::WINDOW);
            let mut draft_w_len = window_seed.len();
            draft_window[..draft_w_len].copy_from_slice(window_seed);
            let mut draft_recent = [0u32; 32];
            let mut draft_recent_len = 0usize;
            let track_recent = context.config.gate_c_context_window;
            if track_recent {
                let recent_seed = story_bounded_window(context.corpus, position, 32);
                draft_recent_len = recent_seed.len();
                draft_recent[..draft_recent_len].copy_from_slice(recent_seed);
            }
            // The draft's token for THIS position is the engine's own
            // pass-1 selection.
            let mut pending = rule12.selected;
            let mut draft_anchor = pending;
            let mut draft_gate = false;
            // Rescue variant (falsifier-1 negative: draft-gated live
            // 40.4% vs 41.3% — drift kills the lift): drift enters
            // through UNCERTAIN intermediate steps, so the STRICT gate
            // additionally requires every draft step — not just the
            // final anchor step — to resolve as ExactContext.
            let mut draft_all_exact = true;
            for _ in 0..lookahead {
                draft_push(
                    &mut draft_window,
                    &mut draft_w_len,
                    &mut draft_recent,
                    &mut draft_recent_len,
                    track_recent,
                    pending,
                );
                let draft_bundle = runtime::bundle_window_plain(
                    context.artifacts,
                    context.gate_rotations,
                    &draft_window[..draft_w_len],
                );
                let draft_sig = runtime::sig_plain(context.artifacts, &draft_bundle);
                let draft_code = runtime::assign_for_bundle(context.artifacts, &draft_bundle);
                let draft_outcome = context
                    .scorer_with_exct
                    .score_candidates_coded(
                        &draft_sig,
                        Some(&draft_code),
                        &draft_recent[..draft_recent_len],
                    )
                    .expect("gate C: scoring self-produced sig");
                pending = draft_outcome.selected;
                draft_anchor = pending;
                draft_gate = draft_outcome.witness.status == ScoreStatus::ExactContext;
                draft_all_exact &= draft_gate;
            }
            if draft_gate {
                if let Some(fwd_row) = context.fwd_table.get(&(lookahead, draft_anchor)) {
                    fwd_draft_live = true;
                    let (selected, fused_bits) =
                        fuse_forward_arm(context.scorer_with_exct, &rule12, fwd_row, next);
                    fwd_draft_selected = selected;
                    bits_fwd_draft = fused_bits;
                    // The strict slice is a subset of the draft slice
                    // (the all-steps gate implies the final-step gate),
                    // and the fused selection is identical there — same
                    // draft anchor, same row, same base.
                    if draft_all_exact {
                        fwd_strict_live = true;
                        fwd_strict_selected = selected;
                        bits_fwd_strict = fused_bits;
                    }
                }
            }
        }
    }
    // ---- #446 M1: two-sided arm (instrumentation only) ----
    // The pair table takes D4-style precedence wherever it resolves with
    // support; elsewhere the arm IS Rule 1+2. The falsifier repeats the
    // whole construction with a right key lifted from a foreign held-out
    // position under a fixed half-length rotation, so key cardinality,
    // backoff shape, support gate and smoothing are held constant.
    //
    // NOT CAUSAL: the right key is built from tokens AFTER the target.
    // Infill/analysis (A-mode) regime only — never a generation number.
    let resolved_two_sided = context
        .two_sided
        .resolve(&code, &context.right_codes[position]);
    let twosided_depth = resolved_two_sided.map_or(0, |(depth, _)| depth);
    let (twosided_selected, bits_twosided, twosided_live) =
        apply_two_sided_arm(resolved_two_sided, rule12.selected, bits[2], next);
    let shuffled_source = if context.held_positions.is_empty() {
        position
    } else {
        context.held_positions[(index + context.shuffle_rotation) % context.held_positions.len()]
    };
    let resolved_shuffled = if shuffled_source < context.corpus.n {
        context
            .two_sided
            .resolve(&code, &context.right_codes[shuffled_source])
    } else {
        None
    };
    let (shuffled_selected, bits_twosided_shuffled, twosided_shuffled_live) =
        apply_two_sided_arm(resolved_shuffled, rule12.selected, bits[2], next);

    // ---- #446 M2: latent right-context mixture (CAUSALLY LEGITIMATE) ----
    // Serving reads the LEFT key only. `resolve_left` backs off to the
    // deepest populated left prefix with support; the class posterior
    // there supplies P(c | left) and the class-conditional emission
    // cells are mixed under it. The ORACLE arm replaces that posterior
    // with the point mass on the TRUE right class (an upper bound, NOT
    // causal). The SHUFFLED arm replaces it with the posterior of a
    // FOREIGN left key at the same depth, holding the emission tables,
    // smoothing, support gate and backoff constant.
    let mut latent_selected = rule12.selected;
    let mut bits_latent = bits[2];
    let mut latent_live = false;
    let mut latent_oracle_selected = rule12.selected;
    let mut bits_latent_oracle = bits[2];
    let mut latent_oracle_live = false;
    let mut latent_shuffled_selected = rule12.selected;
    let mut bits_latent_shuffled = bits[2];
    let mut latent_shuffled_live = false;
    if let Some((latent_depth, latent_prefix, posterior)) = context.latent.resolve_left(&code) {
        let weights = latent_class_weights(&posterior);
        (latent_selected, bits_latent, latent_live) = apply_latent_arm(
            context.latent,
            latent_depth,
            latent_prefix,
            &weights,
            rule12.selected,
            bits[2],
            next,
        );
        let right = &context.right_codes[position];
        if right.1 {
            let true_class = pack_prefix(&right.0, context.latent.class_depth);
            (
                latent_oracle_selected,
                bits_latent_oracle,
                latent_oracle_live,
            ) = apply_latent_arm(
                context.latent,
                latent_depth,
                latent_prefix,
                &[(true_class, 1.0)],
                rule12.selected,
                bits[2],
                next,
            );
        }
        let foreign_prefix = if shuffled_source < context.corpus.n {
            pack_prefix(&context.left_codes[shuffled_source], latent_depth)
        } else {
            latent_prefix
        };
        if let Some(foreign) = context.latent.posterior_at(latent_depth, foreign_prefix) {
            let foreign_weights = latent_class_weights(&foreign);
            (
                latent_shuffled_selected,
                bits_latent_shuffled,
                latent_shuffled_live,
            ) = apply_latent_arm(
                context.latent,
                latent_depth,
                latent_prefix,
                &foreign_weights,
                rule12.selected,
                bits[2],
                next,
            );
        }
    }

    let hit_fwd = fwd_selected == teacher_argmax;
    let hit_fwd_self = fwd_self_selected == teacher_argmax;
    let hit_fwd_gated = fwd_gated_selected == teacher_argmax;
    let hit_fwd_draft = fwd_draft_selected == teacher_argmax;
    let hit_fwd_strict = fwd_strict_selected == teacher_argmax;

    let witness_replay_failed = index < context.config.witness_sample
        && verify_witness_replay(
            context.r4g1,
            Some(context.artifact_container),
            &rule12.witness,
            context.config.root_top_b,
            context.config.exct_top_x,
        )
        .is_some();

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
        teacher_in_candidates: teacher_present,
        transition_offset_raw: i64::from(rule12.witness.transition_offset.raw()),
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
        exact_context_source: rule12.exact_context_source,
        next,
        hit_fwd,
        bits_fwd,
        fwd_live,
        hit_fwd_self,
        bits_fwd_self,
        fwd_self_live,
        hit_fwd_gated,
        bits_fwd_gated,
        fwd_gated_live,
        anchor_hat_correct,
        hit_fwd_draft,
        bits_fwd_draft,
        fwd_draft_live,
        hit_fwd_strict,
        bits_fwd_strict,
        fwd_strict_live,
        hit_twosided: twosided_selected == teacher_argmax,
        bits_twosided,
        twosided_live,
        twosided_depth,
        hit_twosided_shuffled: shuffled_selected == teacher_argmax,
        bits_twosided_shuffled,
        twosided_shuffled_live,
        hit_latent: latent_selected == teacher_argmax,
        bits_latent,
        latent_live,
        hit_latent_oracle: latent_oracle_selected == teacher_argmax,
        bits_latent_oracle,
        latent_oracle_live,
        hit_latent_shuffled: latent_shuffled_selected == teacher_argmax,
        bits_latent_shuffled,
        latent_shuffled_live,
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
/// footprint, rank, and candidate-count fields; 13 = #364 attribution:
/// `config.emission_selection`/`config.emission_shrinkage` record the
/// compiled emission mode, `emission_selection_stats` promotes the
/// per-region contrast/selection statistics (previously stderr-only)
/// into the report, and `gate_c.rule12_status_counts` splits the
/// exact-context bucket by resolving mechanism (NGRAM row vs EXCT
/// probe) so post-#362 rows remain comparable with pre-#362 ones;
/// 14 = context-row compile knobs (`config.context_order` /
/// `config.context_entries`) recorded so NGRAM A/B bundles are
/// attributable; 15 = #375 opt-in Gate C window mode recorded
/// (`config.gate_c_context_window`) — window-on rows are a different
/// measurement population from every schema-≤14 row; 16 = #381
/// sweepable repetition penalty recorded
/// (`config.repetition_penalty_raw`); 17 = #393 substrate-resolution
/// era; 18 = #399 M2 forward-anchor instrumentation rows in `gate_c`
/// (`rule12_fwd_fused`, the live-slice pair, and the fused win/loss
/// cross-tabs) — certifier-side only, the serving scorer is untouched;
/// 19 = #399 B′ self-anchor and confidence-gated arms (the two-pass
/// serving question) plus predicted-anchor accuracy; schema twenty is
/// the #399 falsifier-1 DRAFT-anchor gated arm (the
/// `rule12_fwd_draft_fused` rows and the draft win/loss cross-tab),
/// where the anchor is predicted from the engine's own greedy draft
/// rather than teacher-forced context; schema twenty-one is the #399
/// rescue variant, the STRICT-gated draft arm (the
/// `rule12_fwd_strict_fused` rows and the strict win/loss cross-tab):
/// the same draft, trusted only where every intermediate greedy step
/// resolved as ExactContext; schema twenty-two removes the packed-FMM
/// footprint fields (`fmm_bytes`/`fmm_rank`/`fmm_candidate_count`) and the
/// FMM section emission itself — issue #290 recorded the far-field family
/// as measured dead (research/290-fmm/RESULT-52.md), and #425 removed the
/// uncalled emission and runtime paths; schema twenty-three is the #446 M1
/// TWO-SIDED context arms in `gate_c` (`rule12_twosided`, its live-slice
/// pair, the pair-resolution depth histogram, the
/// `rule12_twosided_shuffled` foreign-right-key falsifier and the two
/// win/loss cross-tabs) together with the dilution slice they exist to
/// answer (`rule12_twosided_exct_slice` and `rule12_on_twosided_exct_slice`,
/// the two arms restricted to the positions Rule 1+2 resolved as
/// ExactContext, plus `twosided_keys_per_full_left` and its two counts —
/// how many distinct two-sided keys the right context carves each full
/// left graded code into). Evidence is keyed on the pair of the left
/// graded code prefix and a right graded code prefix and takes D4-style
/// precedence wherever it resolves with support. Those rows are NOT
/// causally available to left-to-right generation — the right key reads
/// tokens after the target — so they are an infill/analysis (A-mode)
/// measurement, or prospectively a construction-time signal, and must
/// never be quoted as a generation number; the table is built inside the
/// Gate C evaluation from the construction split alone, so the artifact
/// format, the serving scorer and the replay contract are untouched.
/// Schema twenty-four is the #446 M2 LATENT right-context mixture in
/// `gate_c`: `rule12_latent_mix` and its live-slice pair, the
/// `rule12_latent_oracle` upper bound, the `rule12_latent_shuffled`
/// foreign-posterior falsifier, the class-structure counts
/// (`latent_class_depth`, `latent_classes_per_full_left` and its two
/// cell counts), `latent_headroom_fraction` and the pre-declared
/// `latent_exit_rule_met` verdict. The mixture treats the right context
/// as a LATENT variable observed only during CONSTRUCTION and
/// marginalized away at serving, where the left key alone is read, so
/// unlike every schema-twenty-three two-sided row `rule12_latent_mix` IS
/// causally available to left-to-right generation and IS quotable as a
/// generation number; the oracle row is an upper bound and remains not
/// causal. Nothing here changes the artifact format, the serving
/// scorer's default path, the witness or the replay contract.
/// Schema twenty-five is the #467 sampled Gate C decision mode: every
/// [`GateCMetrics`] row gains `positions_sampled` and `standard_error`,
/// and `gate_c` gains `held_out_population` and `positions_sampled`. The
/// motivation is arithmetic, not engineering taste. A full pass over
/// 402,802 held-out positions at a top-1 rate near 26 percent has a
/// standard error of 0.07pp, while every pre-declared exit rule in this
/// repository is written at plus or minus 2pp — roughly 30x more
/// precision than any decision consumes, bought with hours of wall
/// clock. A 10,000-position decision run has a standard error of 0.44pp,
/// still well inside a 2pp rule, at about 40x less cost. The mode is
/// opt-in through the `R4_GATE_C_SAMPLE` environment override; UNSET is
/// the full pass and is behaviour-identical (bit-identical rates and
/// bits) to schema twenty-four, where `positions_sampled` is `0` on
/// every row and `standard_error` is the sampling error the full
/// held-out split itself carries. When the override is set, the
/// evaluation scores a deterministic stride-selected, order-preserving
/// subsample of the held-out list; the held-out MASK and therefore every
/// construction-split table (forward-anchor, two-sided, latent, unigram
/// null) is still built from the FULL held-out list, so a sampled run
/// measures the same model as a full run and differs only in how many
/// positions the measurement averages over. A row whose
/// `positions_sampled` is nonzero is an ESTIMATE and must be quoted with
/// its `standard_error`, never as a census.
///
/// Schema twenty-six is the #471 arm-skip knob. The twenty-four #446 M1 and
/// M2 rows move from the top level of `gate_c` into
/// `gate_c.right_context_arms`, which is `null` when
/// `R4_GATE_C_SKIP_ARMS=right_context` told the run not to build the
/// whole-corpus right-context code pass those arms depend on;
/// `gate_c.skipped_arm_groups` names what was skipped. This is a MOVE, not a
/// redefinition: on a run with the override unset every one of those rows
/// holds the value schema twenty-five gave it, one level deeper. The nesting
/// exists so that "not evaluated" and "evaluated and zero" stop sharing a
/// representation — a flat skipped run would have published a full set of
/// zeroed arms and a pre-declared exit rule reported as unmet.
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
    /// Per-region emission selection/contrast statistics (normalized
    /// means over regions; schema 13).
    pub emission_selection_stats: EmissionSelectionStats,
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
    /// How the per-region emission list was chosen (#364;
    /// [`EmissionSelection::label`]).
    pub emission_selection: String,
    /// The per-region residual shrinkage applied (#364;
    /// [`EmissionShrinkage::label`]).
    pub emission_shrinkage: String,
    /// Highest compiled lexical context order (0 = no NGRAM rows).
    pub context_order: u8,
    /// Per-context candidate bound for the compiled NGRAM rows.
    pub context_entries: usize,
    /// Whether Gate C rows evaluated with the story-bounded recent-token
    /// window (#375; schema 15). Window-on rows are NOT comparable with
    /// window-off rows — NGRAM rows fire and the repetition penalty
    /// engages.
    pub gate_c_context_window: bool,
    /// The repetition-penalty magnitude the scorers applied (#381;
    /// schema 16; raw ScoreQ units, shipped default -2,000,000).
    pub repetition_penalty_raw: i32,
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
        schema: 26,
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
            emission_selection: config.emission_selection.label(),
            emission_shrinkage: config.emission_shrinkage.label(),
            context_order: config.context_order,
            context_entries: config.context_entries,
            gate_c_context_window: config.gate_c_context_window,
            repetition_penalty_raw: config.repetition_penalty_raw,
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
        emission_selection_stats: info.emission_selection_stats,
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
    use super::{compile_context_rows, ScoreConfig};
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
        let rows = compile_context_rows(&corpus, &observations, 100, &ScoreConfig::default());
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

    /// The #362 A/B knob: order 0 compiles no rows, order 1 compiles
    /// bigram rows only, and the entries bound truncates per row.
    #[test]
    fn context_order_gates_row_compilation() {
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
        let off = ScoreConfig {
            context_order: 0,
            ..ScoreConfig::default()
        };
        assert!(compile_context_rows(&corpus, &observations, 100, &off).is_empty());

        let bigram_only = ScoreConfig {
            context_order: 1,
            ..ScoreConfig::default()
        };
        let rows = compile_context_rows(&corpus, &observations, 100, &bigram_only);
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| row.context_len == 1));

        let bounded = ScoreConfig {
            context_entries: 1,
            ..ScoreConfig::default()
        };
        let rows = compile_context_rows(&corpus, &observations, 100, &bounded);
        assert!(rows.iter().all(|row| row.entries.len() <= 1));
    }

    /// #375: the Gate C window is story-bounded and capped, matching the
    /// boundary rule the compiler applies to trigram keys.
    #[test]
    fn story_bounded_window_respects_boundaries_and_cap() {
        let corpus = corpus(); // story: [0, 0, 1, 1, 1], input: [10, 20, 30, 40, 50]
        assert_eq!(super::story_bounded_window(&corpus, 1, 32), &[10, 20]);
        // Position 2 opens story 1: the window must not reach back into story 0.
        assert_eq!(super::story_bounded_window(&corpus, 2, 32), &[30]);
        assert_eq!(super::story_bounded_window(&corpus, 4, 32), &[30, 40, 50]);
        // The cap bounds the window even inside one story.
        assert_eq!(super::story_bounded_window(&corpus, 4, 2), &[40, 50]);
        assert_eq!(super::story_bounded_window(&corpus, 4, 1), &[50]);
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

#[cfg(test)]
mod arm_skip_tests {
    use super::{parse_skip_arms, SkippedArms, ARM_GROUP_RIGHT_CONTEXT};

    /// The unset contract (#471, mirroring #467's): no value means every arm
    /// is evaluated. This is the branch every existing caller takes, so it is
    /// the one whose regression would be silent.
    #[test]
    fn unset_skips_nothing() {
        assert_eq!(parse_skip_arms(None), SkippedArms::default());
        assert!(!parse_skip_arms(None).any());
        assert!(parse_skip_arms(None).names().is_empty());
    }

    /// `R4_GATE_C_SKIP_ARMS=` written by a shell script that computed an
    /// empty list must mean the same thing as not writing the line, and a
    /// list with stray separators or padding must not become a typo.
    #[test]
    fn empty_and_padded_values_skip_nothing_extra() {
        assert_eq!(parse_skip_arms(Some("")), SkippedArms::default());
        assert_eq!(parse_skip_arms(Some("   ")), SkippedArms::default());
        assert_eq!(parse_skip_arms(Some(",,")), SkippedArms::default());
        let padded = parse_skip_arms(Some("  right_context , "));
        assert!(padded.right_context);
        assert_eq!(padded.names(), vec![ARM_GROUP_RIGHT_CONTEXT.to_owned()]);
    }

    #[test]
    fn the_right_context_group_is_recognised_and_named_back() {
        let skipped = parse_skip_arms(Some(ARM_GROUP_RIGHT_CONTEXT));
        assert!(skipped.right_context);
        assert!(skipped.any());
        assert_eq!(skipped.names(), vec![ARM_GROUP_RIGHT_CONTEXT.to_owned()]);
    }

    /// The failure mode this panic exists to prevent: a caller who meant to
    /// buy a thirty-second decision run gets an eighty-five-minute one, with
    /// nothing in the output saying the knob did not take. Silently ignoring
    /// an unknown name is the one behaviour that cannot be noticed.
    #[test]
    #[should_panic(expected = "unknown arm group")]
    fn an_unknown_group_panics_rather_than_being_ignored() {
        let _ = parse_skip_arms(Some("right-context"));
    }

    /// Case is not normalised on purpose: the group name is an identifier in
    /// a run record, and accepting several spellings of it would make two
    /// records of the same run disagree on what was skipped.
    #[test]
    #[should_panic(expected = "unknown arm group")]
    fn case_variants_are_not_silently_accepted() {
        let _ = parse_skip_arms(Some("RIGHT_CONTEXT"));
    }
}
