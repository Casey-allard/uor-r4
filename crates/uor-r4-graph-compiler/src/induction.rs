//! Multiresolution cover induction (graph-compiler plan §5 Phase 2,
//! issue #60 first slice): turn the flat per-stage classes into a
//! multiresolution, overlapping region cover with calibrated memberships.
//!
//! # Observation vector lane
//!
//! Cover input vectors come from the **existing deterministic
//! context-bundle path** — the `[i64; D]` dyadic bundle of
//! [`runtime::bundle_plain`] over corpus positions, centered by the
//! compiled per-dimension thresholds and L2-normalized to f32. This is
//! exactly the geometry the current signature encoder consumes in
//! [`compiler::compile`] step 4, so the region cover and the incumbent
//! 4×256 class cover route the same vectors. An observation's semantic
//! code `H(x)` is [`runtime::sig_plain`] of the same bundle: L2
//! normalization divides by a positive scalar, so sign bits of the
//! normalized vector equal sign bits of the centered bundle, and the
//! binarized region prototypes below drop straight into the runtime's
//! masked-Hamming membership path. Loading `hidden_state()` traces is
//! explicitly OUT of this slice; the trace surface (`teacher.rs`
//! `hidden_state`/`top_k`) is the v2 enrichment seam — the vector lane is
//! isolated in [`build_observations`] so a trace-backed lane can replace
//! it without touching induction.
//!
//! # Induction
//!
//! - **Spherical k-means** over the normalized vectors, mini-batched:
//!   batches are contiguous observation ranges whose size derives from
//!   `--memory-budget` (see [`derive_batch_size`]); the E-step accumulates
//!   per-centroid f64 partial sums in global observation order across
//!   batches, so the batch size is a pure resource knob and never changes
//!   results. Observation extraction is sharded across bounded scoped
//!   workers; the resulting chunks are merged in canonical position order.
//!   K-means iterations and all reductions remain ordered, which keeps T=1
//!   and T=N byte-identical (plan §4.1 ordered reductions).
//!   Centroid state is `k × D × 4` bytes; v1 holds the f32 observation
//!   matrix resident (1.2 KB/observation, documented below) — the §4.1
//!   quantized/spilled shard lane is the `observe.rs` pipeline
//!   integration, a v2 item.
//! - **Seeding is content-addressed**: the first centroid is the
//!   observation selected by a blake3-derived index over the corpus and
//!   artifact κs (the `deterministic_project` pattern); the remaining
//!   centroids are chosen by greedy farthest-point traversal (ties to the
//!   lowest index), which spreads seeds across tight clusters
//!   deterministically — no iteration-order or RNG dependence anywhere.
//! - **Convergence**: at most [`MAX_KMEANS_ITERS`] iterations, stopping
//!   when every centroid coordinate is bit-identical between rounds
//!   (exact f32 compare; once assignments stop changing, the f64 sums and
//!   their normalizations are bit-identical, so this terminates well
//!   before the cap in practice). Empty clusters are reseeded with the
//!   farthest unchosen point; clusters still empty after the cap are
//!   dropped, so a clustering run can yield fewer regions than requested.
//! - **Multiresolution recursion**: depth 1 is a broad cover of
//!   `CoverConfig::k0` regions. A region at depth `< depths` splits into
//!   [`SPLIT_CHILDREN`] children only when **all** of: its support ≥
//!   `min_support`; the region budget has room; and the within-region
//!   next-token entropy reduction
//!   `H(parent) − Σ_c (|c|/|parent|)·H(c)` — computed from the corpus
//!   next-token distributions over the region's train members, tokens in
//!   ascending order, f64 — exceeds `entropy_gain_bits`. (v1 note: the
//!   entropy uses f64 `log2`, which is libm-sensitive cross-platform;
//!   same-machine determinism is pinned by the T-invariance tests, and
//!   cross-platform byte equality awaits the D2 canonical deterministic
//!   compile mode, exactly as for `compile()`'s macOS-pinned κ baseline.)
//! - **Split criterion (issue #435)**: the floor above is *absolute*, so
//!   it tightens as the corpus grows — a bigger corpus spreads a region's
//!   next-token mass over more types, every single split removes a
//!   smaller absolute share of the entropy, and the constant bar rejects
//!   it (measured: 38 regions / contrast 0.5012 at 500k records, 14
//!   regions / contrast 0.0915 at 2 110 111 records of the same domain —
//!   more data, coarser geometry). [`SplitCriterion`] adds corpus-relative
//!   tests (`relative`: a fraction of `H(parent)`; `mdl`: a BIC-style
//!   description-length trade whose per-token bar *loosens* with support)
//!   and [`CoverConfig::scale_k0`]/[`CoverConfig::scale_regions_budget`]
//!   grow the cover's capacity as `(n/n_ref)^alpha` with `alpha = 0.45`
//!   from the measured class-growth law. All of it is **off by default**:
//!   an unconfigured compile takes the absolute floor and the fixed `k0`,
//!   so shipped artifacts and κs are unchanged.
//! - **Objective scoring (schema 2)**: each eligible split records
//!   `H(A|R)` and `H(S_future|R)` proxies, an information-bottleneck term
//!   `I(Z;X) - beta·I(Z;Y_future)`, and runtime/artifact/bytes/structure
//!   cost proxies plus a bounded top-E distinctiveness reward against the
//!   global next-token prior; weighted objective ties deterministically
//!   resolve to "keep". Fitting uses train observations only; reports emit
//!   both train and held-out decompositions so regressions are visible per
//!   component. The new reward defaults to zero for byte-exact compatibility.
//!
//! # Regions, membership, reference classifier
//!
//! Each region carries: a unit f32 prototype (the centroid), binarized to
//! a sign-bit signature **exactly like the class-sig pipeline** (bit `d`
//! set iff prototype coordinate `d > 0.0`); an all-ones mask (v1); a
//! calibrated radius = a configurable percentile (95% by default) of member
//! masked-Hamming distances ([`compiler::quantile_radius`], the PR #38 logic);
//! parent id;
//! depth. **Overlapping membership** at a depth: the top-[`TOP_M`] regions
//! by masked-Hamming distance (ties to the lowest region id, the
//! [`runtime::assign_memberships_plain`] ordering) filtered to those
//! within radius, with the nearest region retained when nothing is in
//! range — the nearest-class backoff floor. The frozen
//! [`ReferenceClassifier`] reproduces exact compiler-side membership
//! (nearest prototype by cosine) as the normative semantics; routing
//! recall of the shipped binary path is measured against it.
//!
//! # Graph and artifact
//!
//! Parent/child refinement edges (E_r, kind 0) come from the recursion;
//! lateral neighbor edges (E_o, kind 1) join region pairs at one depth
//! whose top-M co-activation count over the train observations reaches
//! [`coactivation_min`], degree-capped at [`MAX_NEIGHBOR_EDGES`] per region
//! (peers ordered by count descending, id ascending; edges canonicalized
//! `src < dst`). The canonical edge array is sorted by `(src, kind, dst)`
//! so each node's refinement children are contiguous, and the reverse
//! index is sorted by `(dst, src, kind)` — the same conventions as
//! `convert_r4g1`, whose HEAD/section layout (W=5, 36-byte signatures,
//! v0 linear-count root prior in EMIT) this module reuses. The emitted
//! container must pass `uor_r4_graph_format::GraphView::parse` plus
//! `verify_cids` before it is returned (fail closed). EXCT is optional
//! per the slice scope and is omitted in v1.

use rayon::prelude::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use uor_r4_core::transformerless::compiler::{
    self, Corpus, D, SIG_BYTES, SIG_WORDS, STAGES, WINDOW, quantile_radius,
};
use uor_r4_core::transformerless::runtime;

#[inline]
fn canonical_math_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("TLESS_CANONICAL_DETERMINISTIC").is_ok_and(|value| value != "0")
    })
}

#[inline]
fn canonical_sqrtf(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::sqrtf(value)
    } else {
        value.sqrt()
    }
}

#[inline]
fn canonical_log2(value: f64) -> f64 {
    if canonical_math_enabled() {
        libm::log2(value)
    } else {
        value.log2()
    }
}

#[inline]
fn canonical_powf(base: f64, exponent: f64) -> f64 {
    if canonical_math_enabled() {
        libm::pow(base, exponent)
    } else {
        base.powf(exponent)
    }
}

/// Default multiresolution depth cap (root at depth 0; regions at 1..=3).
pub const DEFAULT_DEPTHS: usize = 3;
/// Default number of regions of the broad depth-1 cover.
pub const DEFAULT_K0: usize = 8;
/// Children created by one accepted split (binary refinement, v1).
pub const SPLIT_CHILDREN: usize = 2;

/// Content address of one observation sample: blake3 over the
/// little-endian token bytes of the context window.
pub fn sample_id(tokens: &[u32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for token in tokens {
        hasher.update(&token.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}
/// Default minimum train support for a region to be eligible to split.
pub const DEFAULT_MIN_SUPPORT: usize = 64;
/// Default entropy-reduction floor for accepting a split, in bits/token.
pub const DEFAULT_SPLIT_ENTROPY_GAIN_BITS: f64 = 0.25;
/// Default relative entropy-gain floor: the accepted split must remove
/// this fraction of the parent's next-token entropy (issue #435). See
/// [`SplitCriterion::RelativeGain`] for the calibration of the value.
pub const DEFAULT_RELATIVE_GAIN_THETA: f64 = 0.0384;
/// Default MDL/BIC penalty in bits per added model parameter (issue
/// #435). `0.5` is the BIC constant: one parameter costs `½·log₂ n` bits.
pub const DEFAULT_MDL_PENALTY_BITS: f64 = 0.5;
/// Default exponent of the corpus-relative capacity law (issue #435).
/// Measured on nested prefixes of the natural corpus over a 17× data
/// range: distinct left-context keys grow as `n^0.44` and distinct
/// next-token-distribution signature classes as `n^0.46`; `0.45` is the
/// midpoint and is only applied when capacity scaling is switched on.
pub const DEFAULT_CAPACITY_ALPHA: f64 = 0.45;
/// Reference train-observation count of the capacity law: the train
/// partition of the 500k-record natural corpus (~4.0e5 positions, the
/// 100% row of the measured class-growth table, 401 655 positions over
/// 2 012 stories). At `n = n_ref` every scaled knob equals its base
/// value, so the 500k reference cover is unchanged by scaling.
pub const DEFAULT_CAPACITY_REF_N: usize = 400_000;
/// Upper clamp of a scaled region budget (bytes discipline: ~96 B per
/// added region, so 65 536 regions is ~6 MB of cover nodes).
pub const MAX_SCALED_REGIONS_BUDGET: usize = 65_536;

/// Which structural-stratification test gates a candidate split.
///
/// Issue #435: the shipped test is an **absolute** entropy floor. As the
/// corpus grows a region's next-token distribution spreads over more
/// types, so any single binary split removes a smaller absolute share of
/// the entropy and the fixed floor becomes a *stricter* filter at larger
/// `n` — measured: 38 regions at 500k records, 14 regions at 2 110 111
/// records on the same domain. The alternatives below make the bar
/// track the data. [`SplitCriterion::Absolute`] is the default so that
/// unconfigured compiles are byte-identical to today's artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitCriterion {
    /// Shipped behaviour: accept iff `gain > entropy_gain_bits`.
    #[default]
    Absolute,
    /// Corpus-relative: accept iff `gain / H(parent) > theta`.
    ///
    /// The bar is a *fraction* of what there is to explain, so it does
    /// not tighten when `H(parent)` grows with the corpus. The default
    /// `theta = 0.0325` is calibrated at the 500k reference, where the
    /// mean next-token entropy of a split-eligible region is ~7.7 bits:
    /// `0.25 / 7.7 = 0.0325`, i.e. the relative bar reproduces the
    /// absolute bar at the reference scale and only diverges away from
    /// it.
    RelativeGain,
    /// MDL/BIC: accept iff
    /// `gain_bits · support > penalty_bits · added_params · log₂(n)`,
    /// with `added_params = (SPLIT_CHILDREN − 1) · parent_types`
    /// (the extra multinomial cells a child model introduces) and `n`
    /// the train-observation count.
    ///
    /// The left side is the total number of bits the split saves on the
    /// data it explains and grows **linearly** in support; the right
    /// side is the code length of the extra parameters and grows only
    /// as `types · log n` (~`n^0.46 · log n` by the measured class-growth
    /// law). The effective per-token bar therefore *loosens* as support
    /// grows, which is the opposite of the shipped absolute floor.
    Mdl,
}

impl SplitCriterion {
    /// Canonical lowercase label (also the accepted env-var spelling).
    pub fn label(self) -> &'static str {
        match self {
            SplitCriterion::Absolute => "absolute",
            SplitCriterion::RelativeGain => "relative",
            SplitCriterion::Mdl => "mdl",
        }
    }

    /// Parse a criterion label; `None` when unrecognized.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "absolute" => Some(SplitCriterion::Absolute),
            "relative" | "relative_gain" => Some(SplitCriterion::RelativeGain),
            "mdl" | "bic" => Some(SplitCriterion::Mdl),
            _ => None,
        }
    }
}

impl std::fmt::Display for SplitCriterion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl Serialize for SplitCriterion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}
/// Top-token bound used by the induction-time distinctiveness objective.
pub const DISTINCTIVENESS_TOP_E: usize = 64;
/// Objective configuration schema version carried in compile reports.
pub const OBJECTIVE_CONFIG_SCHEMA: u32 = 2;
/// Bounded multi-membership per depth (matches the runtime's top-M).
pub const TOP_M: usize = 3;
/// Radius calibration quantile: the 95th percentile of member distances.
pub const RADIUS_QUANTILE_NUMERATOR: u32 = 95;
/// Radius calibration quantile denominator.
pub const RADIUS_QUANTILE_DENOMINATOR: u32 = 100;
/// Per-region cap on lateral neighbor edges (E_o).
pub const MAX_NEIGHBOR_EDGES: usize = 8;
/// Per-region cap on forward transition edges (E_f).
pub const MAX_TRANSITION_EDGES: usize = 16;
/// Divisor of the train-observation count setting the absolute
/// co-activation floor: `max(2, n_train / COACTIVATION_MIN_DIVISOR)`.
pub const COACTIVATION_MIN_DIVISOR: usize = 200;
/// Default cap on the total number of cover regions (the byte budget in
/// region units: ~30 B/node + 80 B prototype+mask + edges per region).
pub const DEFAULT_REGIONS_BUDGET: usize = 256;
/// Default `--memory-budget` in megabytes.
pub const DEFAULT_MEMORY_BUDGET_MB: u64 = 512;
/// Hard iteration cap of one k-means run.
pub const MAX_KMEANS_ITERS: usize = 50;
/// Fixed reserve subtracted from the memory budget before batch sizing
/// (artifact + corpus + report structures; documented constant).
pub const MEMORY_RESERVE_BYTES: u64 = 64 * 1024 * 1024;
/// Accounting size of one resident observation: the f32 vector
/// (`D × 4` = 1152 B) plus signature (36 B), sample id (32 B), and next
/// token (4 B) — 1224 B. v1 keeps the matrix resident (see module docs).
pub const BYTES_PER_OBSERVATION: u64 = (D * 4 + SIG_BYTES + 32 + 4) as u64;

/// Edge kind of refinement (parent/child) edges (matches
/// `transitions::EdgeKind::Refinement`).
pub const EDGE_KIND_REFINEMENT: u8 = 0;
/// Edge kind of lateral neighbor (co-activation) edges (matches
/// `transitions::EdgeKind::Overlap`).
pub const EDGE_KIND_NEIGHBOR: u8 = 1;
/// Edge kind of sequence transition edges.
pub const EDGE_KIND_TRANSITION: u8 = 2;
const APPROX_BYTES_PER_ADDED_REGION: f64 = 96.0;
const APPROX_BYTES_READ_PER_REGION: f64 = SIG_BYTES as f64;

/// blake3 input labeling this compiler as the compiler of record.
const COMPILER_VERSION_LABEL: &[u8] = b"uor-r4-core cover v0";

/// HEAD defaults reused from `convert_r4g1` (RFC §4 starting defaults).
const DEFAULT_MAX_FRONTIER_WIDTH: u16 = 32;
const MAX_CANDIDATES: u16 = 16;
const DEFAULT_MAX_EMISSION_ENTRIES: u32 = 64;
const SHORTLIST_SIZE: u16 = 8;
const MAX_PROGRAM_STEPS: u32 = 64;

/// Node index of the synthetic root region (all ranges empty).
pub const ROOT_NODE: u32 = 0;

/// Configuration of one cover induction run.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverConfig {
    /// Multiresolution depth cap (regions live at depths 1..=depths).
    pub depths: usize,
    /// Number of regions of the broad depth-1 cover.
    pub k0: usize,
    /// Cap on the total region count (the byte budget in region units).
    pub regions_budget: usize,
    /// Memory budget in bytes; derives the k-means batch size.
    pub memory_budget_bytes: u64,
    /// Bounded worker count used for independent observation extraction.
    /// K-means reductions remain ordered, so this value never changes the
    /// emitted artifact bytes.
    pub threads: u32,
    /// Minimum train support for a region to be eligible to split.
    pub min_support: usize,
    /// Entropy-reduction floor for accepting a split, in bits/token.
    pub entropy_gain_bits: f64,
    /// Which split test gates the recursion (issue #435). Defaults to
    /// [`SplitCriterion::Absolute`], the shipped behaviour.
    pub split_criterion: SplitCriterion,
    /// Fraction-of-parent-entropy floor of [`SplitCriterion::RelativeGain`].
    /// Ignored by the other criteria.
    pub relative_gain_theta: f64,
    /// Bits charged per added model parameter by [`SplitCriterion::Mdl`].
    /// Ignored by the other criteria.
    pub mdl_penalty_bits: f64,
    /// Scale `k0` with the corpus size (issue #435). Off by default.
    pub scale_k0: bool,
    /// Scale `regions_budget` with the corpus size. Off by default.
    pub scale_regions_budget: bool,
    /// Exponent of the capacity law `(n / n_ref)^alpha` used by the two
    /// scaling switches above; unused when both are off.
    pub capacity_alpha: f64,
    /// Reference train-observation count at which the scaled knobs equal
    /// their base values.
    pub capacity_ref_n: usize,
    /// Percentile numerator used to calibrate region acceptance radii.
    pub radius_quantile_numerator: u32,
    /// Percentile denominator used to calibrate region acceptance radii.
    pub radius_quantile_denominator: u32,
    /// Versioned objective configuration (compiler-side only).
    pub objective: ObjectiveConfig,
}

/// Weights of objective components (all compiler-side).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectiveWeights {
    pub predictive_entropy: f64,
    pub future_state_entropy: f64,
    pub teacher_loss: f64,
    pub runtime_cost: f64,
    pub artifact_size: f64,
    pub bytes_read: f64,
    pub structural_complexity: f64,
    pub ib_term: f64,
    /// Reward candidate partitions whose child distributions depart from the
    /// global next-token prior. Lower weighted objective scores are better, so
    /// this component is subtracted when its weight is positive.
    pub between_region_distinctiveness: f64,
}

/// Metadata describing approximations used by objective estimators.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectiveEstimatorMetadata {
    pub predictive_entropy: String,
    pub future_state_entropy: String,
    pub teacher_loss: String,
    pub runtime_cost: String,
    pub artifact_size: String,
    pub bytes_read: String,
    pub structural_complexity: String,
    pub ib_term: String,
    pub between_region_distinctiveness: String,
    pub fit_partition: String,
    pub report_partitions: String,
}

/// Versioned objective configuration.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ObjectiveConfig {
    pub schema: u32,
    pub ib_beta: f64,
    pub weights: ObjectiveWeights,
    pub estimators: ObjectiveEstimatorMetadata,
}

impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            schema: OBJECTIVE_CONFIG_SCHEMA,
            ib_beta: 1.0,
            weights: ObjectiveWeights {
                predictive_entropy: 1.0,
                future_state_entropy: 0.5,
                teacher_loss: 1.0,
                runtime_cost: 0.0,
                artifact_size: 0.0,
                bytes_read: 0.0,
                structural_complexity: 0.0,
                ib_term: 0.25,
                between_region_distinctiveness: 0.0,
            },
            estimators: ObjectiveEstimatorMetadata {
                predictive_entropy: "H(A|R) from empirical next-token frequencies".to_owned(),
                future_state_entropy: "H(S_future|R) with S_future=(prev,next) one-step proxy"
                    .to_owned(),
                teacher_loss: "cross-entropy proxy uses H(A|R) (teacher unavailable online)"
                    .to_owned(),
                runtime_cost: "region-count proxy for routing work".to_owned(),
                artifact_size: "serialized-byte estimate from split deltas".to_owned(),
                bytes_read: "signature-byte estimate per active region".to_owned(),
                structural_complexity: "regions+edges proxy".to_owned(),
                ib_term: "I(Z;X)-beta*I(Z;Y_future), X via deterministic assignment proxy"
                    .to_owned(),
                between_region_distinctiveness: format!(
                    "weighted mean of 1 - top-{} child/prior token overlap over next-token counts",
                    DISTINCTIVENESS_TOP_E
                ),
                fit_partition: "train".to_owned(),
                report_partitions: "train,held_out".to_owned(),
            },
        }
    }
}

impl Default for CoverConfig {
    /// The pinned defaults, each overridable by an `R4_COVER_*`
    /// environment variable for capacity measurements (#399/#393 M-C2).
    /// Every override falls back to the pinned constant when unset, so
    /// default-config behavior is byte-identical (κ-neutral) unless a
    /// measurement explicitly opts in; a set-but-invalid value panics
    /// (see [`compiler::capacity_override_usize`]).
    fn default() -> Self {
        Self {
            depths: compiler::capacity_override_usize("R4_COVER_DEPTHS", DEFAULT_DEPTHS),
            k0: compiler::capacity_override_usize("R4_COVER_K0", DEFAULT_K0),
            regions_budget: compiler::capacity_override_usize(
                "R4_COVER_REGIONS_BUDGET",
                DEFAULT_REGIONS_BUDGET,
            ),
            memory_budget_bytes: compiler::capacity_override_usize(
                "R4_COVER_MEMORY_BUDGET_MB",
                DEFAULT_MEMORY_BUDGET_MB as usize,
            ) as u64
                * 1024
                * 1024,
            threads: 1,
            min_support: compiler::capacity_override_usize(
                "R4_COVER_MIN_SUPPORT",
                DEFAULT_MIN_SUPPORT,
            ),
            entropy_gain_bits: compiler::capacity_override_f64(
                "R4_COVER_ENTROPY_GAIN_BITS",
                DEFAULT_SPLIT_ENTROPY_GAIN_BITS,
            ),
            split_criterion: split_criterion_override("R4_COVER_SPLIT_CRITERION"),
            relative_gain_theta: compiler::capacity_override_f64(
                "R4_COVER_RELATIVE_THETA",
                DEFAULT_RELATIVE_GAIN_THETA,
            ),
            mdl_penalty_bits: compiler::capacity_override_f64(
                "R4_COVER_MDL_PENALTY_BITS",
                DEFAULT_MDL_PENALTY_BITS,
            ),
            scale_k0: flag_override("R4_COVER_SCALE_K0"),
            scale_regions_budget: flag_override("R4_COVER_SCALE_REGIONS_BUDGET"),
            capacity_alpha: compiler::capacity_override_f64(
                "R4_COVER_CAPACITY_ALPHA",
                DEFAULT_CAPACITY_ALPHA,
            ),
            capacity_ref_n: compiler::capacity_override_usize(
                "R4_COVER_CAPACITY_REF_N",
                DEFAULT_CAPACITY_REF_N,
            ),
            radius_quantile_numerator: RADIUS_QUANTILE_NUMERATOR,
            radius_quantile_denominator: RADIUS_QUANTILE_DENOMINATOR,
            objective: ObjectiveConfig::default(),
        }
    }
}

/// Split-criterion environment override (#435). Unset is κ-neutral
/// ([`SplitCriterion::Absolute`]); a set-but-unrecognized value panics,
/// matching [`compiler::capacity_override_usize`]'s fail-loud contract.
fn split_criterion_override(name: &str) -> SplitCriterion {
    match std::env::var(name) {
        Ok(value) => SplitCriterion::parse(&value).unwrap_or_else(|| {
            panic!("{name} must be one of absolute|relative|mdl, got {value:?}")
        }),
        Err(_) => SplitCriterion::default(),
    }
}

/// Boolean environment override (#435): unset or `0` is off, `1` is on;
/// anything else panics rather than silently defaulting.
fn flag_override(name: &str) -> bool {
    match std::env::var(name) {
        Ok(value) => match value.trim() {
            "0" | "false" => false,
            "1" | "true" => true,
            other => panic!("{name} must be 0|1, got {other:?}"),
        },
        Err(_) => false,
    }
}

impl CoverConfig {
    /// The capacity-law factor `(n / n_ref)^alpha` (issue #435), `1.0`
    /// when `n == n_ref`. Deterministic: a single `powf` on two exactly
    /// representable ratios, canonicalized under
    /// `TLESS_CANONICAL_DETERMINISTIC` like the module's other math.
    pub fn capacity_factor(&self, n_observations: usize) -> f64 {
        let reference = self.capacity_ref_n.max(1) as f64;
        let ratio = n_observations.max(1) as f64 / reference;
        canonical_powf(ratio, self.capacity_alpha)
    }

    /// Depth-1 cover width actually used: `k0` unless [`Self::scale_k0`]
    /// is on, in which case `clamp(round(k0 · (n/n_ref)^alpha), 2, budget)`.
    pub fn effective_k0(&self, n_observations: usize) -> usize {
        if !self.scale_k0 {
            return self.k0;
        }
        let scaled = (self.k0 as f64 * self.capacity_factor(n_observations)).round();
        let scaled = if scaled.is_finite() {
            scaled.max(0.0).min(MAX_SCALED_REGIONS_BUDGET as f64) as usize
        } else {
            self.k0
        };
        scaled
            .max(SPLIT_CHILDREN)
            .min(self.effective_regions_budget(n_observations))
    }

    /// Region budget actually used: `regions_budget` unless
    /// [`Self::scale_regions_budget`] is on, in which case
    /// `clamp(round(budget · (n/n_ref)^alpha), 2, MAX_SCALED_REGIONS_BUDGET)`.
    pub fn effective_regions_budget(&self, n_observations: usize) -> usize {
        if !self.scale_regions_budget {
            return self.regions_budget;
        }
        let scaled = (self.regions_budget as f64 * self.capacity_factor(n_observations)).round();
        let scaled = if scaled.is_finite() {
            scaled.max(0.0).min(MAX_SCALED_REGIONS_BUDGET as f64) as usize
        } else {
            self.regions_budget
        };
        scaled.clamp(SPLIT_CHILDREN, MAX_SCALED_REGIONS_BUDGET)
    }

    /// The structural-stratification test of one candidate split
    /// (issue #435). `parent_types` is only read by
    /// [`SplitCriterion::Mdl`] and may be `0` otherwise.
    ///
    /// - `absolute`: `gain > entropy_gain_bits`
    /// - `relative`: `gain > theta · H(parent)` (written multiplicatively
    ///   so a zero-entropy parent rejects instead of dividing by zero)
    /// - `mdl`: `gain · support > penalty_bits · added_params · log₂(n)`
    ///   with `added_params = (SPLIT_CHILDREN − 1) · max(1, parent_types)`
    pub fn split_accepted(
        &self,
        gain_bits: f64,
        parent_entropy_bits: f64,
        support: usize,
        parent_types: usize,
        n_observations: usize,
    ) -> bool {
        match self.split_criterion {
            SplitCriterion::Absolute => gain_bits > self.entropy_gain_bits,
            SplitCriterion::RelativeGain => {
                // Multiplicative form: no division, and a parent with no
                // next-token entropy left to remove is never split.
                parent_entropy_bits > 0.0
                    && gain_bits > self.relative_gain_theta * parent_entropy_bits
            }
            SplitCriterion::Mdl => {
                let added_params = (SPLIT_CHILDREN.saturating_sub(1) * parent_types.max(1)) as f64;
                let code_length = self.mdl_penalty_bits
                    * added_params
                    * canonical_log2(n_observations.max(2) as f64);
                gain_bits * support as f64 > code_length
            }
        }
    }
}

/// One induction sample: a corpus position's context bundle in both
/// consumed forms (f32 vector for clustering, sign signature for the
/// Hamming membership path) plus its content address and next token.
#[derive(Debug, Clone)]
pub struct Observation {
    /// Corpus position this observation was derived from.
    pub position: u32,
    /// Content address of the context window (`sample_id`).
    pub sample: [u8; 32],
    /// L2-normalized threshold-centered bundle — the k-means input.
    pub vector: Vec<f32>,
    /// `H(x)`: sign bits of the centered bundle (`runtime::sig_plain`).
    pub sig: [u8; SIG_BYTES],
    /// Preceding token of this corpus position.
    pub prev: u32,
    /// Sampled next token of this corpus position.
    pub next: u32,
}

/// The context window of one corpus position, oldest first: the fed
/// tokens `input[start..=i]` within one story, capped at [`WINDOW`] —
/// the same window `observe::observe_sharded` hashes for sample ids.
pub fn context_window(corpus: &Corpus, i: usize) -> Vec<u32> {
    let mut start = i;
    while start > 0 && corpus.story[start - 1] == corpus.story[i] && i + 1 - start < WINDOW {
        start -= 1;
    }
    (start..=i).map(|j| corpus.input[j]).collect()
}

/// Build the cover input vectors of the given corpus positions from the
/// deterministic context-bundle path (module docs: the vector lane).
/// Positions are consumed in the given order; that order is the canonical
/// observation order of every later stage.
pub fn build_observations(
    art: &compiler::Compiled,
    corpus: &Corpus,
    positions: &[usize],
) -> Vec<Observation> {
    build_observations_serial(art, corpus, positions)
}

/// Build observations with bounded parallel extraction.
///
/// Each worker owns a contiguous position shard and returns observations in
/// that shard's input order. The caller merges shards by their original
/// index, so the public result is byte-for-byte identical to the serial
/// implementation. A worker panic is reported to the compiler caller rather
/// than being hidden behind an `unwrap`.
pub fn build_observations_with_threads(
    art: &compiler::Compiled,
    corpus: &Corpus,
    positions: &[usize],
    threads: usize,
) -> Vec<Observation> {
    if positions.is_empty() {
        return Vec::new();
    }
    let worker_count = threads.max(1).min(positions.len());
    if worker_count == 1 {
        return build_observations_serial(art, corpus, positions);
    }
    let chunk_size = positions.len().div_ceil(worker_count);
    let mut chunks = Vec::with_capacity(worker_count);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);
        for (chunk_id, shard) in positions.chunks(chunk_size).enumerate() {
            handles.push((
                chunk_id,
                scope.spawn(move || build_observations_serial(art, corpus, shard)),
            ));
        }
        for (chunk_id, handle) in handles {
            // A worker panic propagates as a panic (total): the observation
            // build has no reportable condition of its own (R5), and a
            // panicked worker is a defect, not a limitation the model sanctions.
            let observations = handle.join().unwrap_or_else(|_| {
                panic!("observation worker {chunk_id} panicked during cover compilation")
            });
            chunks.push((chunk_id, observations));
        }
    });
    chunks.sort_by_key(|(chunk_id, _)| *chunk_id);
    let mut observations = Vec::with_capacity(positions.len());
    for (_, mut chunk) in chunks {
        observations.append(&mut chunk);
    }
    observations
}

fn build_observations_serial(
    art: &compiler::Compiled,
    corpus: &Corpus,
    positions: &[usize],
) -> Vec<Observation> {
    let rot = compiler::derive_rotations();
    let mut observations = Vec::with_capacity(positions.len());
    for &i in positions {
        let use_hidden = corpus.hidden.is_some();
        let bundle = if !use_hidden {
            runtime::bundle_plain(art, &rot, corpus, i)
        } else {
            [0i64; D] // unused
        };

        let mut vector = vec![0f32; D];
        let mut nn = 0f32;

        if let Some(ref hidden) = corpus.hidden {
            for d in 0..D {
                let x = hidden[i][d];
                vector[d] = x;
                nn += x * x;
            }
        } else {
            for d in 0..D {
                let x = (bundle[d] - art.thresholds[d]) as f32;
                vector[d] = x;
                nn += x * x;
            }
        }

        let nn = canonical_sqrtf(nn).max(1e-9);
        for x in vector.iter_mut() {
            *x /= nn;
        }

        // Compute signature directly from the vector (sign bits)
        let mut sig = [0u8; uor_r4_core::transformerless::compiler::SIG_BYTES];
        for d in 0..D {
            if vector[d] > 0.0 {
                sig[d / 8] |= 1 << (d % 8);
            }
        }

        observations.push(Observation {
            position: i as u32,
            sample: sample_id(&context_window(corpus, i)),
            vector,
            sig,
            prev: corpus.input[i],
            next: corpus.next[i],
        });
    }
    observations
}

/// Train/held-out position split of a corpus: the `compiler::train_cut`
/// 80/20 story cut, each partition in ascending corpus-position order
/// (the canonical observation order).
pub fn split_positions(corpus: &Corpus) -> (Vec<usize>, Vec<usize>) {
    let cut = compiler::train_cut(corpus);
    let mut train = Vec::new();
    let mut held_out = Vec::new();
    for i in 0..corpus.n {
        if corpus.story[i] < cut {
            train.push(i);
        } else {
            held_out.push(i);
        }
    }
    (train, held_out)
}

/// Memory-budget-derived mini-batch size (plan §4.1 formula shape):
/// `peak ≈ M_reserve + batch × S_obs + M_cluster ≤ budget`, solved for
/// `batch` with `M_cluster = regions_budget × D × 4` worst-case centroid
/// state and `S_obs = `[`BYTES_PER_OBSERVATION`]. The result is clamped
/// to `[1, n_obs]`; budgets below the reserve degrade to batches of one.
/// Batch size never influences results (module docs).
pub fn derive_batch_size(memory_budget_bytes: u64, regions_budget: usize, n_obs: usize) -> usize {
    let cluster_state = (regions_budget as u64)
        .saturating_mul(D as u64)
        .saturating_mul(4);
    let available = memory_budget_bytes
        .saturating_sub(MEMORY_RESERVE_BYTES)
        .saturating_sub(cluster_state);
    let batch = (available / BYTES_PER_OBSERVATION).max(1);
    (batch as usize).min(n_obs.max(1))
}

/// Content-addressed seed of one clustering run: blake3 over the
/// artifact κ, the corpus κ, and a run label (the
/// `deterministic_project` pattern — every seed choice traces to pinned
/// content, never to iteration order).
fn run_seed(artifact_kappa: &str, corpus_kappa: &str, label: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(artifact_kappa.as_bytes());
    hasher.update(corpus_kappa.as_bytes());
    hasher.update(label.as_bytes());
    *hasher.finalize().as_bytes()
}

fn blake3_u64(seed: &[u8; 32], key: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed);
    hasher.update(key.as_bytes());
    u64::from_le_bytes(
        hasher.finalize().as_bytes()[0..8]
            .try_into()
            .expect("8-byte slice"),
    )
}

/// Dot product of two unit f32 vectors, accumulated in index order
/// (fixed association order — deterministic by construction).
fn dot(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut acc = 0f32;
    for (&x, &y) in a.iter().zip(b) {
        acc += x * y;
    }
    acc
}

/// Cosine distance `1 − dot` of unit vectors.
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    1.0 - dot(a, b)
}

fn normalize(v: &mut [f32]) {
    let mut nn = 0f32;
    for &x in v.iter() {
        nn += x * x;
    }
    let nn = canonical_sqrtf(nn).max(1e-9);
    for x in v.iter_mut() {
        *x /= nn;
    }
}

/// Outcome of one [`spherical_kmeans`] run.
#[derive(Debug, Clone)]
pub struct KMeansResult {
    /// Unit centroids of the non-empty clusters, in ascending original
    /// cluster-id order.
    pub centroids: Vec<Vec<f32>>,
    /// Per-point assignment: index into `centroids`.
    pub assignment: Vec<u32>,
    /// Iterations actually run (≤ [`MAX_KMEANS_ITERS`]).
    pub iterations: u32,
}

/// Mini-batch spherical k-means over unit f32 vectors.
///
/// Seeding: the first centroid is `points[blake3_u64(seed) % n]`; the
/// rest are greedy farthest-point picks (max minimum cosine distance to
/// the chosen set, ties to the lowest index). Iterations are sequential;
/// the E-step walks mini-batches of `batch_size` in ascending batch-id
/// order and accumulates per-centroid f64 sums in global point order, so
/// results are independent of `batch_size` and of any worker count.
pub fn spherical_kmeans(
    points: &[&[f32]],
    k: usize,
    seed: &[u8; 32],
    batch_size: usize,
) -> KMeansResult {
    let n = points.len();
    if n == 0 {
        return KMeansResult {
            centroids: Vec::new(),
            assignment: Vec::new(),
            iterations: 0,
        };
    }
    let k_eff = k.min(n).max(1);
    let batch_size = batch_size.max(1).min(n.max(1));

    // Seeding: blake3-selected first centroid, then greedy farthest
    // point (ties to the lowest index).
    let mut centroids: Vec<Vec<f32>> = Vec::with_capacity(k_eff);
    let first = (blake3_u64(seed, "first-centroid") as usize) % n;
    centroids.push(points[first].to_vec());
    while centroids.len() < k_eff {
        let mut best_idx = 0usize;
        let mut best_dist = -1f32;
        for (i, point) in points.iter().enumerate() {
            let mut min_dist = f32::MAX;
            for centroid in &centroids {
                let d = cosine_distance(point, centroid);
                if d < min_dist {
                    min_dist = d;
                }
            }
            if min_dist > best_dist {
                best_dist = min_dist;
                best_idx = i;
            }
        }
        centroids.push(points[best_idx].to_vec());
    }

    let mut assignment = vec![0u32; n];
    let mut iterations = 0u32;
    for iteration in 0..MAX_KMEANS_ITERS {
        iterations = (iteration + 1) as u32;
        // E-step: assignment in global point order, batched only for the
        // §4.1 memory shape; f64 partial sums per centroid.
        let mut sums = vec![vec![0f64; D]; k_eff];
        let mut counts = vec![0u64; k_eff];
        let mut batch_start = 0usize;
        while batch_start < n {
            let batch_end = (batch_start + batch_size).min(n);
            let assignments: Vec<u32> = points[batch_start..batch_end]
                .par_iter()
                .map(|point| {
                    let mut best_k = 0u32;
                    let mut best_dot = f32::MIN;
                    for (k, centroid) in centroids.iter().enumerate() {
                        let d = dot(point, centroid);
                        if d > best_dot {
                            best_dot = d;
                            best_k = k as u32;
                        }
                    }
                    best_k
                })
                .collect();
            // Keep the canonical f64 accumulation order: parallelism is
            // used for independent distance work, while reduction remains
            // in ascending observation order for byte-identical centroids.
            for (offset, (point, &best_k)) in points[batch_start..batch_end]
                .iter()
                .zip(assignments.iter())
                .enumerate()
            {
                let i = batch_start + offset;
                assignment[i] = best_k;
                counts[best_k as usize] += 1;
                let sum = &mut sums[best_k as usize];
                for (s, &x) in sum.iter_mut().zip(point.iter()) {
                    *s += x as f64;
                }
            }
            batch_start = batch_end;
        }
        // M-step: normalize member sums; empty clusters reseed at the
        // farthest unchosen point (ties to the lowest index).
        let mut new_centroids: Vec<Vec<f32>> = Vec::with_capacity(k_eff);
        let mut reseeded: Vec<usize> = Vec::new();
        for (k, centroid) in centroids.iter().enumerate() {
            if counts[k] > 0 {
                let mut next: Vec<f32> = sums[k]
                    .iter()
                    .map(|&s| (s / counts[k] as f64) as f32)
                    .collect();
                normalize(&mut next);
                new_centroids.push(next);
            } else {
                new_centroids.push(centroid.clone());
                reseeded.push(k);
            }
        }
        let mut chosen: Vec<usize> = Vec::with_capacity(reseeded.len());
        for &empty in &reseeded {
            let mut best_idx = 0usize;
            let mut best_dist = -1f32;
            for (i, point) in points.iter().enumerate() {
                if chosen.contains(&i) {
                    continue; // one reseed per point: no duplicate centroids
                }
                let mut min_dist = f32::MAX;
                for (k, centroid) in new_centroids.iter().enumerate() {
                    if k == empty {
                        continue;
                    }
                    let d = cosine_distance(point, centroid);
                    if d < min_dist {
                        min_dist = d;
                    }
                }
                if min_dist > best_dist {
                    best_dist = min_dist;
                    best_idx = i;
                }
            }
            chosen.push(best_idx);
            new_centroids[empty] = points[best_idx].to_vec();
        }
        // Convergence: bit-identical centroids (exact f32 compare).
        let converged = new_centroids.iter().zip(centroids.iter()).all(|(a, b)| {
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.to_bits() == y.to_bits())
        });
        centroids = new_centroids;
        if converged {
            break;
        }
    }

    // Final assignment against the converged centroids, then drop empty
    // clusters (relabel assignments into ascending cluster-id order).
    let mut final_assignment = vec![0u32; n];
    let mut counts = vec![0u64; k_eff];
    let assignments: Vec<u32> = points
        .par_iter()
        .map(|point| {
            let mut best_k = 0u32;
            let mut best_dot = f32::MIN;
            for (k, centroid) in centroids.iter().enumerate() {
                let d = dot(point, centroid);
                if d > best_dot {
                    best_dot = d;
                    best_k = k as u32;
                }
            }
            best_k
        })
        .collect();
    for (i, &best_k) in assignments.iter().enumerate() {
        final_assignment[i] = best_k;
        counts[best_k as usize] += 1;
    }
    let mut relabel = vec![u32::MAX; k_eff];
    let mut kept: Vec<Vec<f32>> = Vec::new();
    for (k, centroid) in centroids.into_iter().enumerate() {
        if counts[k] > 0 {
            relabel[k] = kept.len() as u32;
            kept.push(centroid);
        }
    }
    for a in final_assignment.iter_mut() {
        *a = relabel[*a as usize];
    }
    KMeansResult {
        centroids: kept,
        assignment: final_assignment,
        iterations,
    }
}

/// Empirical next-token distribution of a member set (ascending token
/// order — B-tree iteration is the deterministic reduction).
fn next_token_counts(observations: &[Observation], members: &[usize]) -> BTreeMap<u32, u64> {
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for &m in members {
        *counts.entry(observations[m].next).or_insert(0) += 1;
    }
    counts
}

fn all_next_token_counts(observations: &[Observation]) -> BTreeMap<u32, u64> {
    let members: Vec<usize> = (0..observations.len()).collect();
    next_token_counts(observations, &members)
}

/// Distinctiveness of a child distribution against the global prior.
///
/// This is the serving-side `1 - overlap` analogue: compare the top-E token
/// sets by count, with token-id ordering breaking equal-count ties. The
/// denominator is the child set, so a child with a unique top token receives
/// full credit even when the prior has fewer than E entries.
fn top_token_distinctiveness(
    child_counts: &BTreeMap<u32, u64>,
    prior_counts: &BTreeMap<u32, u64>,
) -> f64 {
    if child_counts.is_empty() || prior_counts.is_empty() {
        return 0.0;
    }
    let mut child: Vec<(u32, u64)> = child_counts
        .iter()
        .map(|(&token, &count)| (token, count))
        .collect();
    let mut prior: Vec<(u32, u64)> = prior_counts
        .iter()
        .map(|(&token, &count)| (token, count))
        .collect();
    let by_count_then_token = |left: &(u32, u64), right: &(u32, u64)| {
        right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0))
    };
    child.sort_by(by_count_then_token);
    prior.sort_by(by_count_then_token);
    child.truncate(DISTINCTIVENESS_TOP_E);
    prior.truncate(DISTINCTIVENESS_TOP_E);
    let prior_tokens: std::collections::BTreeSet<u32> =
        prior.into_iter().map(|(token, _)| token).collect();
    let shared = child
        .iter()
        .filter(|(token, _)| prior_tokens.contains(token))
        .count();
    1.0 - shared as f64 / child.len() as f64
}

/// Shannon entropy of a token distribution in bits (f64, tokens in
/// ascending order; libm-sensitive cross-platform — see module docs).
fn entropy_bits<K: Ord>(counts: &BTreeMap<K, u64>) -> f64 {
    let total: u64 = counts.values().sum();
    if total == 0 {
        return 0.0;
    }
    let mut h = 0.0f64;
    for &count in counts.values() {
        let p = count as f64 / total as f64;
        h -= p * canonical_log2(p);
    }
    h
}

/// Within-region next-token entropy reduction of a candidate partition:
/// `H(parent) − Σ_c (|c|/|parent|)·H(c)`, member lists in observation
/// order, tokens ascending. Deterministic (same f64 caveat as above).
pub fn entropy_reduction(
    observations: &[Observation],
    members: &[usize],
    children: &[Vec<usize>],
) -> f64 {
    let parent = entropy_bits(&next_token_counts(observations, members));
    let total = members.len() as f64;
    if total == 0.0 {
        return 0.0;
    }
    let mut expected_child = 0.0f64;
    for child in children {
        if child.is_empty() {
            continue;
        }
        let weight = child.len() as f64 / total;
        expected_child += weight * entropy_bits(&next_token_counts(observations, child));
    }
    parent - expected_child
}

fn conditional_entropy_for_key<F>(observations: &[Observation], assignments: &[u32], key: F) -> f64
where
    F: Fn(&Observation) -> u64,
{
    let count = observations.len().min(assignments.len());
    if count == 0 {
        return 0.0;
    }
    let mut region_key_counts: BTreeMap<u32, BTreeMap<u64, u64>> = BTreeMap::new();
    for (observation, &region) in observations.iter().zip(assignments.iter()).take(count) {
        *region_key_counts
            .entry(region)
            .or_default()
            .entry(key(observation))
            .or_insert(0) += 1;
    }
    let total = count as f64;
    let mut value = 0.0;
    for counts in region_key_counts.values() {
        let region_total: u64 = counts.values().sum();
        if region_total == 0 {
            continue;
        }
        value += (region_total as f64 / total) * entropy_bits(counts);
    }
    value
}

fn assignment_entropy(assignments: &[u32]) -> f64 {
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for &region in assignments {
        *counts.entry(region).or_insert(0) += 1;
    }
    entropy_bits(&counts)
}

fn next_entropy(observations: &[Observation]) -> f64 {
    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for observation in observations {
        *counts.entry(observation.next).or_insert(0) += 1;
    }
    entropy_bits(&counts)
}

fn future_state_key(observation: &Observation) -> u64 {
    ((observation.prev as u64) << 32) | observation.next as u64
}

/// Objective component values emitted as separate report columns.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ObjectiveComponents {
    pub predictive_entropy_bits: f64,
    pub future_state_entropy_bits: f64,
    pub teacher_loss_bits: f64,
    pub runtime_cost_units: f64,
    pub artifact_size_bytes: f64,
    pub bytes_read: f64,
    pub structural_complexity: f64,
    pub between_region_distinctiveness: f64,
    pub ib_i_zx_bits: f64,
    pub ib_i_zy_future_bits: f64,
    pub ib_objective_bits: f64,
    pub weighted_score: f64,
}

impl ObjectiveComponents {
    fn weighted(config: &ObjectiveConfig, values: ObjectiveRawValues) -> Self {
        let ib_objective = values.ib_i_zx_bits - config.ib_beta * values.ib_i_zy_future_bits;
        let w = &config.weights;
        let weighted_score = w.predictive_entropy * values.predictive_entropy_bits
            + w.future_state_entropy * values.future_state_entropy_bits
            + w.teacher_loss * values.teacher_loss_bits
            + w.runtime_cost * values.runtime_cost_units
            + w.artifact_size * values.artifact_size_bytes
            + w.bytes_read * values.bytes_read
            + w.structural_complexity * values.structural_complexity
            + w.ib_term * ib_objective;
        let weighted_score = weighted_score
            - w.between_region_distinctiveness * values.between_region_distinctiveness;
        Self {
            predictive_entropy_bits: values.predictive_entropy_bits,
            future_state_entropy_bits: values.future_state_entropy_bits,
            teacher_loss_bits: values.teacher_loss_bits,
            runtime_cost_units: values.runtime_cost_units,
            artifact_size_bytes: values.artifact_size_bytes,
            bytes_read: values.bytes_read,
            structural_complexity: values.structural_complexity,
            between_region_distinctiveness: values.between_region_distinctiveness,
            ib_i_zx_bits: values.ib_i_zx_bits,
            ib_i_zy_future_bits: values.ib_i_zy_future_bits,
            ib_objective_bits: ib_objective,
            weighted_score,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ObjectiveRawValues {
    predictive_entropy_bits: f64,
    future_state_entropy_bits: f64,
    teacher_loss_bits: f64,
    runtime_cost_units: f64,
    artifact_size_bytes: f64,
    bytes_read: f64,
    structural_complexity: f64,
    between_region_distinctiveness: f64,
    ib_i_zx_bits: f64,
    ib_i_zy_future_bits: f64,
}

#[derive(Debug, Clone, Copy)]
struct ObjectiveCostTerms {
    runtime_cost_units: f64,
    artifact_size_bytes: f64,
    bytes_read: f64,
    structural_complexity: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionDecisionAudit {
    pub region_id: u32,
    pub depth: u8,
    pub support: u32,
    pub entropy_gain_bits: f64,
    pub keep: ObjectiveComponents,
    pub split: ObjectiveComponents,
    pub decision: String,
}

fn compare_region_decision(keep: &ObjectiveComponents, split: &ObjectiveComponents) -> bool {
    split.weighted_score.total_cmp(&keep.weighted_score).is_lt()
}

fn objective_for_partition(
    config: &ObjectiveConfig,
    observations: &[Observation],
    assignments: &[u32],
    costs: ObjectiveCostTerms,
) -> ObjectiveComponents {
    let count = observations.len().min(assignments.len());
    if count == 0 {
        return ObjectiveComponents::weighted(
            config,
            ObjectiveRawValues {
                predictive_entropy_bits: 0.0,
                future_state_entropy_bits: 0.0,
                teacher_loss_bits: 0.0,
                runtime_cost_units: costs.runtime_cost_units,
                artifact_size_bytes: costs.artifact_size_bytes,
                bytes_read: costs.bytes_read,
                structural_complexity: costs.structural_complexity,
                between_region_distinctiveness: 0.0,
                ib_i_zx_bits: 0.0,
                ib_i_zy_future_bits: 0.0,
            },
        );
    }
    let observations = &observations[..count];
    let assignments = &assignments[..count];
    let predictive_entropy_bits =
        conditional_entropy_for_key(observations, assignments, |o| o.next as u64);
    let future_state_entropy_bits =
        conditional_entropy_for_key(observations, assignments, future_state_key);
    let teacher_loss_bits = predictive_entropy_bits;
    let ib_i_zx_bits = assignment_entropy(assignments);
    let ib_i_zy_future_bits = (next_entropy(observations) - predictive_entropy_bits).max(0.0);
    let prior_counts = all_next_token_counts(observations);
    let mut region_distinctiveness = 0.0;
    let mut region_counts: BTreeMap<u32, BTreeMap<u32, u64>> = BTreeMap::new();
    for (observation, &region) in observations.iter().zip(assignments.iter()) {
        *region_counts
            .entry(region)
            .or_default()
            .entry(observation.next)
            .or_insert(0) += 1;
    }
    for counts in region_counts.values() {
        let support: u64 = counts.values().sum();
        region_distinctiveness +=
            support as f64 / count as f64 * top_token_distinctiveness(counts, &prior_counts);
    }
    ObjectiveComponents::weighted(
        config,
        ObjectiveRawValues {
            predictive_entropy_bits,
            future_state_entropy_bits,
            teacher_loss_bits,
            runtime_cost_units: costs.runtime_cost_units,
            artifact_size_bytes: costs.artifact_size_bytes,
            bytes_read: costs.bytes_read,
            structural_complexity: costs.structural_complexity,
            between_region_distinctiveness: region_distinctiveness,
            ib_i_zx_bits,
            ib_i_zy_future_bits,
        },
    )
}

fn objective_for_member_partition(
    config: &ObjectiveConfig,
    observations: &[Observation],
    members: &[usize],
    assignments: &[u32],
    prior_counts: &BTreeMap<u32, u64>,
    costs: ObjectiveCostTerms,
) -> ObjectiveComponents {
    let count = members.len().min(assignments.len());
    if count == 0 {
        return ObjectiveComponents::weighted(
            config,
            ObjectiveRawValues {
                predictive_entropy_bits: 0.0,
                future_state_entropy_bits: 0.0,
                teacher_loss_bits: 0.0,
                runtime_cost_units: costs.runtime_cost_units,
                artifact_size_bytes: costs.artifact_size_bytes,
                bytes_read: costs.bytes_read,
                structural_complexity: costs.structural_complexity,
                between_region_distinctiveness: 0.0,
                ib_i_zx_bits: 0.0,
                ib_i_zy_future_bits: 0.0,
            },
        );
    }

    let mut region_next_counts: BTreeMap<u32, BTreeMap<u64, u64>> = BTreeMap::new();
    let mut region_state_counts: BTreeMap<u32, BTreeMap<u64, u64>> = BTreeMap::new();
    let mut next_counts: BTreeMap<u32, u64> = BTreeMap::new();
    for (&member, &region) in members.iter().zip(assignments.iter()).take(count) {
        let observation = &observations[member];
        *region_next_counts
            .entry(region)
            .or_default()
            .entry(observation.next as u64)
            .or_insert(0) += 1;
        *region_state_counts
            .entry(region)
            .or_default()
            .entry(future_state_key(observation))
            .or_insert(0) += 1;
        *next_counts.entry(observation.next).or_insert(0) += 1;
    }

    let conditional = |region_counts: &BTreeMap<u32, BTreeMap<u64, u64>>| -> f64 {
        let mut value = 0.0;
        for counts in region_counts.values() {
            let region_total: u64 = counts.values().sum();
            if region_total == 0 {
                continue;
            }
            value += (region_total as f64 / count as f64) * entropy_bits(counts);
        }
        value
    };
    let predictive_entropy_bits = conditional(&region_next_counts);
    let future_state_entropy_bits = conditional(&region_state_counts);
    let teacher_loss_bits = predictive_entropy_bits;
    let ib_i_zx_bits = assignment_entropy(&assignments[..count]);
    let ib_i_zy_future_bits = (entropy_bits(&next_counts) - predictive_entropy_bits).max(0.0);
    let mut region_distinctiveness = 0.0;
    for counts in region_next_counts.values() {
        let support: u64 = counts.values().sum();
        let token_counts: BTreeMap<u32, u64> = counts
            .iter()
            .map(|(&token, &count)| (token as u32, count))
            .collect();
        region_distinctiveness +=
            support as f64 / count as f64 * top_token_distinctiveness(&token_counts, prior_counts);
    }
    ObjectiveComponents::weighted(
        config,
        ObjectiveRawValues {
            predictive_entropy_bits,
            future_state_entropy_bits,
            teacher_loss_bits,
            runtime_cost_units: costs.runtime_cost_units,
            artifact_size_bytes: costs.artifact_size_bytes,
            bytes_read: costs.bytes_read,
            structural_complexity: costs.structural_complexity,
            between_region_distinctiveness: region_distinctiveness,
            ib_i_zx_bits,
            ib_i_zy_future_bits,
        },
    )
}

/// Hamming distance between two equal-length signatures.
fn hamming_sig(a: &[u8; SIG_BYTES], b: &[u8; SIG_BYTES]) -> u32 {
    let mut dist = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dist += (x ^ y).count_ones();
    }
    dist
}

/// Calibrated acceptance radius of one region: the configured percentile of the
/// members' masked-Hamming distances to the prototype (all-ones mask),
/// reusing [`quantile_radius`]. For member counts ≤ 19 the quantile
/// target is the full count, so the radius covers every member distance.
pub fn calibrate_region_radius_with_quantile(
    member_sigs: &[[u8; SIG_BYTES]],
    prototype_sig: &[u8; SIG_BYTES],
    numerator: u32,
    denominator: u32,
) -> u16 {
    let mut histogram = vec![0u32; D + 1];
    for sig in member_sigs {
        histogram[hamming_sig(sig, prototype_sig) as usize] += 1;
    }
    quantile_radius(&histogram, numerator, denominator)
}

/// Calibrate using the historical 95th-percentile default.
pub fn calibrate_region_radius(
    member_sigs: &[[u8; SIG_BYTES]],
    prototype_sig: &[u8; SIG_BYTES],
) -> u16 {
    calibrate_region_radius_with_quantile(
        member_sigs,
        prototype_sig,
        RADIUS_QUANTILE_NUMERATOR,
        RADIUS_QUANTILE_DENOMINATOR,
    )
}

/// Binarize a prototype to a sign-bit signature exactly like the
/// class-sig pipeline (`compiler::compile` step 4: bit `d` set iff
/// coordinate `d > 0.0`).
pub fn binarize_prototype(prototype: &[f32]) -> [u8; SIG_BYTES] {
    debug_assert_eq!(prototype.len(), D);
    let mut sig = [0u8; SIG_BYTES];
    for (d, &x) in prototype.iter().enumerate() {
        if x > 0.0 {
            sig[d / 8] |= 1 << (d % 8);
        }
    }
    sig
}

/// One region of the induced cover.
#[derive(Debug, Clone)]
pub struct CoverRegion {
    /// Region id (0-based, creation order; parents precede children).
    pub id: u32,
    /// Multiresolution depth (1..=depths).
    pub depth: u8,
    /// Parent region id (`None` at depth 1 — the parent is the root).
    pub parent: Option<u32>,
    /// Child region ids created by the accepted split (empty at leaves).
    pub children: Vec<u32>,
    /// Unit f32 prototype (compiler-side only; never serialized).
    pub prototype: Vec<f32>,
    /// Binarized prototype — the region's packed sign-bit signature.
    pub sig: [u8; SIG_BYTES],
    /// Calibrated acceptance radius: the configured percentile of member
    /// masked-Hamming distances (all-ones mask in v1).
    pub radius: u16,
    /// Train members assigned to this region (top-1).
    pub support: u32,
    /// Within-region next-token entropy in bits (train members).
    pub entropy_bits: f64,
    /// Entropy reduction of the accepted split in bits (0 at leaves).
    pub split_gain_bits: f64,
}

/// The induced multiresolution cover: the region list (creation order)
/// plus the frozen per-train-observation top-1 path.
#[derive(Debug, Clone)]
pub struct Cover {
    pub regions: Vec<CoverRegion>,
    /// Number of depths actually present (`max region depth`).
    pub max_depth: usize,
    /// Per train observation: the top-1 region path (region id per
    /// depth, shorter when the leaf was reached earlier).
    pub paths: Vec<Vec<u32>>,
    /// Per train observation member lists per region (region id order).
    pub members: Vec<Vec<usize>>,
}

impl Cover {
    /// Ids of the regions at one depth, in ascending id order.
    pub fn regions_at_depth(&self, depth: usize) -> Vec<u32> {
        self.regions
            .iter()
            .filter(|r| r.depth as usize == depth)
            .map(|r| r.id)
            .collect()
    }

    /// κ-label of the cover parameters (region ids, depths, parents,
    /// prototype bytes, signatures, radii — the frozen semantics).
    pub fn kappa(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        for region in &self.regions {
            hasher.update(&region.id.to_le_bytes());
            hasher.update(&[region.depth]);
            hasher.update(&region.parent.unwrap_or(u32::MAX).to_le_bytes());
            for &x in &region.prototype {
                hasher.update(&x.to_le_bytes());
            }
            hasher.update(&region.sig);
            hasher.update(&region.radius.to_le_bytes());
        }
        format!("blake3:{}", hasher.finalize().to_hex())
    }
}

/// The frozen reference classifier (glossary: the exact compiler-side
/// region-membership procedure; the normative semantics every optimized
/// router is measured against). Owns the region parameters so the frozen
/// semantics outlives any later compiler mutation.
#[derive(Debug, Clone)]
pub struct ReferenceClassifier {
    regions: Vec<CoverRegion>,
    max_depth: usize,
    kappa: String,
}

impl ReferenceClassifier {
    /// Freeze the exact membership semantics of an induced cover.
    pub fn freeze(cover: &Cover) -> Self {
        Self {
            regions: cover.regions.clone(),
            max_depth: cover.max_depth,
            kappa: cover.kappa(),
        }
    }

    /// κ-label of the frozen parameters.
    pub fn kappa(&self) -> &str {
        &self.kappa
    }

    /// Deepest depth with at least one region.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Exact compiler-side membership: the nearest region at `depth` by
    /// cosine distance to the f32 prototype (ties to the lowest id).
    /// `None` when no region exists at that depth.
    pub fn exact_top1(&self, depth: usize, vector: &[f32]) -> Option<u32> {
        let mut best: Option<(u32, f32)> = None;
        for region in &self.regions {
            if region.depth as usize != depth {
                continue;
            }
            let d = cosine_distance(vector, &region.prototype);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((region.id, d));
            }
        }
        best.map(|(id, _)| id)
    }

    /// Shipped binary membership at `depth` — the
    /// [`runtime::assign_memberships_plain`] semantics: scan regions in
    /// ascending id order keeping the top-[`TOP_M`] by masked-Hamming
    /// distance (strict `<` insertion, so ties go to the lower id),
    /// filter to those within their calibrated radius, and fall back to
    /// the nearest region when nothing is in range. Returned ids are in
    /// distance order (nearest first).
    pub fn binary_memberships(&self, depth: usize, sig: &[u8; SIG_BYTES]) -> Vec<u32> {
        let mut top: Vec<(u32, u32)> = Vec::with_capacity(TOP_M);
        for region in &self.regions {
            if region.depth as usize != depth {
                continue;
            }
            let dist = hamming_sig(sig, &region.sig);
            let mut inserted = false;
            for (idx, &(_, d0)) in top.iter().enumerate() {
                if dist < d0 {
                    top.insert(idx, (region.id, dist));
                    inserted = true;
                    break;
                }
            }
            if !inserted && top.len() < TOP_M {
                top.push((region.id, dist));
            }
            if inserted && top.len() > TOP_M {
                top.pop();
            }
        }
        if top.is_empty() {
            return Vec::new();
        }
        let within: Vec<u32> = top
            .iter()
            .filter(|&&(id, dist)| dist <= u32::from(self.regions[id as usize].radius))
            .map(|&(id, _)| id)
            .collect();
        if within.is_empty() {
            // Nearest-region fallback (the backoff floor).
            vec![top[0].0]
        } else {
            within
        }
    }
}

/// The outcome of [`induce_cover`]: the cover plus the seeds used, for
/// the report's provenance.
#[derive(Debug, Clone)]
pub struct InducedCover {
    pub cover: Cover,
    /// κ of the artifact the observation vectors derive from.
    pub artifact_kappa: String,
    /// κ of the corpus record stream (meta then records).
    pub corpus_kappa: String,
    /// Mini-batch size the memory budget derived.
    pub batch_size: usize,
    /// Auditable split/keep decisions scored by the objective.
    pub decision_trace: Vec<RegionDecisionAudit>,
}

/// Induce the multiresolution cover over the train observations.
///
/// `artifact_kappa`/`corpus_kappa` pin the seed derivation to the
/// consumed content. The recursion processes regions breadth-first
/// (ascending id), so parents always precede children in
/// `cover.regions`.
pub fn induce_cover(
    observations: &[Observation],
    config: &CoverConfig,
    artifact_kappa: &str,
    corpus_kappa: &str,
) -> Option<InducedCover> {
    if observations.is_empty() {
        // No train observations: there is no cover to induce (R5 — the absence
        // of a product, reported as `None`, not a raised error).
        return None;
    }
    let n = observations.len();
    let regions_budget = config.effective_regions_budget(n);
    let batch_size = derive_batch_size(
        config.memory_budget_bytes,
        regions_budget,
        observations.len(),
    );
    let points: Vec<&[f32]> = observations.iter().map(|o| o.vector.as_slice()).collect();
    let global_prior_counts = all_next_token_counts(observations);

    let mut regions: Vec<CoverRegion> = Vec::new();
    let mut members_of: Vec<Vec<usize>> = Vec::new();
    let mut paths: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut decision_trace = Vec::new();

    // Depth 1: the broad cover.
    let k0 = config.effective_k0(n).min(n).max(1);
    let seed = run_seed(artifact_kappa, corpus_kappa, "r4-cover-v1/depth-1");
    let clustering = spherical_kmeans(&points, k0, &seed, batch_size);
    for (cluster, centroid) in clustering.centroids.iter().enumerate() {
        let members: Vec<usize> = clustering
            .assignment
            .iter()
            .enumerate()
            .filter(|&(_, &a)| a as usize == cluster)
            .map(|(i, _)| i)
            .collect();
        let id = regions.len() as u32;
        for &m in &members {
            paths[m].push(id);
        }
        let entropy = entropy_bits(&next_token_counts(observations, &members));
        regions.push(CoverRegion {
            id,
            depth: 1,
            parent: None,
            children: Vec::new(),
            sig: binarize_prototype(centroid),
            prototype: centroid.clone(),
            radius: 0, // calibrated below
            support: members.len() as u32,
            entropy_bits: entropy,
            split_gain_bits: 0.0,
        });
        members_of.push(members);
    }

    // Recursion, breadth-first over ascending region ids.
    let mut cursor = 0usize;
    while cursor < regions.len() {
        let region_id = regions[cursor].id;
        let depth = regions[cursor].depth as usize;
        let support = regions[cursor].support as usize;
        cursor += 1;
        if depth >= config.depths {
            continue;
        }
        if support < config.min_support.max(SPLIT_CHILDREN) {
            continue;
        }
        if regions.len() + SPLIT_CHILDREN - 1 > regions_budget {
            continue;
        }
        let parent_members = members_of[region_id as usize].clone();
        let child_points: Vec<&[f32]> = parent_members
            .iter()
            .map(|&m| observations[m].vector.as_slice())
            .collect();
        let seed = run_seed(
            artifact_kappa,
            corpus_kappa,
            &format!("r4-cover-v1/depth-{}/region-{}", depth + 1, region_id),
        );
        let clustering = spherical_kmeans(&child_points, SPLIT_CHILDREN, &seed, batch_size);
        if clustering.centroids.len() < SPLIT_CHILDREN {
            continue; // degenerate clustering: nothing to refine
        }
        let children_members: Vec<Vec<usize>> = (0..clustering.centroids.len())
            .map(|cluster| {
                clustering
                    .assignment
                    .iter()
                    .enumerate()
                    .filter(|&(_, &a)| a as usize == cluster)
                    .map(|(i, _)| parent_members[i])
                    .collect()
            })
            .collect();
        let gain = entropy_reduction(observations, &parent_members, &children_members);
        let keep_assignments = vec![0u32; parent_members.len()];
        let split_assignments = clustering.assignment.clone();
        let keep_components = objective_for_member_partition(
            &config.objective,
            observations,
            &parent_members,
            &keep_assignments,
            &global_prior_counts,
            ObjectiveCostTerms {
                runtime_cost_units: 0.0,
                artifact_size_bytes: 0.0,
                bytes_read: 0.0,
                structural_complexity: 0.0,
            },
        );
        let split_components = objective_for_member_partition(
            &config.objective,
            observations,
            &parent_members,
            &split_assignments,
            &global_prior_counts,
            ObjectiveCostTerms {
                runtime_cost_units: SPLIT_CHILDREN as f64,
                artifact_size_bytes: (SPLIT_CHILDREN.saturating_sub(1)) as f64
                    * APPROX_BYTES_PER_ADDED_REGION,
                bytes_read: SPLIT_CHILDREN as f64 * APPROX_BYTES_READ_PER_REGION,
                structural_complexity: SPLIT_CHILDREN as f64,
            },
        );
        // Structural-stratification test (#435). `entropy_bits` of the
        // parent region is the same `H(parent)` term `entropy_reduction`
        // subtracts from, and the type count is only materialized for the
        // MDL criterion, so the default (absolute) path does no extra work
        // and is bit-for-bit the shipped test.
        let parent_entropy = regions[region_id as usize].entropy_bits;
        let parent_types = if config.split_criterion == SplitCriterion::Mdl {
            next_token_counts(observations, &parent_members).len()
        } else {
            0
        };
        let entropy_allows_split =
            config.split_accepted(gain, parent_entropy, support, parent_types, n);
        let objective_allows_split = compare_region_decision(&keep_components, &split_components);
        let decision = if !entropy_allows_split {
            "keep:entropy_floor"
        } else if objective_allows_split {
            "split"
        } else if split_components
            .weighted_score
            .total_cmp(&keep_components.weighted_score)
            .is_eq()
        {
            "keep:objective_tie"
        } else {
            "keep:objective_cost"
        };
        decision_trace.push(RegionDecisionAudit {
            region_id,
            depth: depth as u8,
            support: support as u32,
            entropy_gain_bits: gain,
            keep: keep_components,
            split: split_components,
            decision: decision.to_owned(),
        });
        if !entropy_allows_split || !objective_allows_split {
            continue;
        }
        regions[region_id as usize].split_gain_bits = gain;
        let mut child_ids = Vec::with_capacity(children_members.len());
        for (cluster, centroid) in clustering.centroids.iter().enumerate() {
            let members = children_members[cluster].clone();
            let id = regions.len() as u32;
            for &m in &members {
                paths[m].push(id);
            }
            let entropy = entropy_bits(&next_token_counts(observations, &members));
            regions.push(CoverRegion {
                id,
                depth: (depth + 1) as u8,
                parent: Some(region_id),
                children: Vec::new(),
                sig: binarize_prototype(centroid),
                prototype: centroid.clone(),
                radius: 0,
                support: members.len() as u32,
                entropy_bits: entropy,
                split_gain_bits: 0.0,
            });
            members_of.push(members);
            child_ids.push(id);
        }
        regions[region_id as usize].children = child_ids;
    }

    // Radius calibration: the configured percentile of member masked-Hamming
    // distances (all-ones mask), reusing the PR #38 quantile logic.
    for id in 0..regions.len() {
        let sig = regions[id].sig;
        let member_sigs: Vec<[u8; SIG_BYTES]> = members_of[id]
            .iter()
            .map(|&m| observations[m].sig)
            .collect();
        regions[id].radius = calibrate_region_radius_with_quantile(
            &member_sigs,
            &sig,
            config.radius_quantile_numerator,
            config.radius_quantile_denominator,
        );
    }

    let max_depth = regions.iter().map(|r| r.depth as usize).max().unwrap_or(1);
    Some(InducedCover {
        cover: Cover {
            regions,
            max_depth,
            paths,
            members: members_of,
        },
        artifact_kappa: artifact_kappa.to_owned(),
        corpus_kappa: corpus_kappa.to_owned(),
        batch_size,
        decision_trace,
    })
}

/// One canonical edge of the cover graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoverEdge {
    pub src: u32,
    pub kind: u8,
    pub dst: u32,
}

/// Canonical merge of worker-local immutable edge fragments.
///
/// Pattern (issue #171): local discovery emits immutable `CoverEdge`
/// fragments, then a sequential spine performs stable canonical sort,
/// deterministic deduplication, and leaves canonical edge-id assignment to
/// the packer (array position after this merge).
pub fn canonical_merge_edge_fragments(fragments: &[Vec<CoverEdge>]) -> Vec<CoverEdge> {
    let total = fragments.iter().map(Vec::len).sum();
    let mut edges = Vec::with_capacity(total);
    for fragment in fragments {
        edges.extend(fragment.iter().copied());
    }
    edges.sort_by_key(|edge| (edge.src, edge.kind, edge.dst));
    edges.dedup();
    edges
}

/// Artifact node id of a region: the synthetic root is node 0, regions
/// are 1-based (`region.id + 1`).
pub fn region_node_id(region_id: u32) -> u32 {
    region_id + 1
}

/// Co-activation floor for neighbor edges: `max(2, n_train / 200)`.
pub fn coactivation_min(n_train: usize) -> u64 {
    (n_train / COACTIVATION_MIN_DIVISOR).max(2) as u64
}

/// Build the canonical edge set: refinement edges from the recursion
/// (root → depth-1 regions, parent → child below) plus lateral neighbor
/// edges between same-depth region pairs whose top-M co-activation count
/// reaches [`coactivation_min`], degree-capped at [`MAX_NEIGHBOR_EDGES`]
/// per region (peers by count descending, id ascending). Sorted by
/// `(src, kind, dst)`, so each node's refinement children are contiguous
/// (the `convert_r4g1` child-range convention).
pub fn build_edges(
    cover: &Cover,
    reference: &ReferenceClassifier,
    observations: &[Observation],
    story_map: &[u32],
) -> Vec<CoverEdge> {
    let mut refinement_fragment = Vec::with_capacity(cover.regions.len());
    for region in &cover.regions {
        let parent = region.parent.map_or(ROOT_NODE, region_node_id);
        refinement_fragment.push(CoverEdge {
            src: parent,
            kind: EDGE_KIND_REFINEMENT,
            dst: region_node_id(region.id),
        });
    }

    // Co-activation counts over the train observations' top-M
    // memberships per depth (unordered pairs, canonicalized).
    let mut coactive: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    if !observations.is_empty() {
        let chunk_size = observations
            .len()
            .div_ceil(rayon::current_num_threads().max(1));
        let partials: Vec<BTreeMap<(u32, u32), u64>> = observations
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut local = BTreeMap::new();
                for observation in chunk {
                    for depth in 1..=cover.max_depth {
                        let mut memberships = reference.binary_memberships(depth, &observation.sig);
                        memberships.sort_unstable();
                        memberships.dedup();
                        for (a_idx, &a) in memberships.iter().enumerate() {
                            for &b in &memberships[a_idx + 1..] {
                                *local.entry((a, b)).or_insert(0) += 1;
                            }
                        }
                    }
                }
                local
            })
            .collect();
        // Ordered merge keeps the original observation-order reduction.
        for partial in partials {
            for (pair, count) in partial {
                *coactive.entry(pair).or_insert(0) += count;
            }
        }
    }
    let floor = coactivation_min(observations.len());
    // Per-region peer lists, capped and deterministically ordered.
    let mut peers: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
    for (&(a, b), &count) in &coactive {
        if count < floor {
            continue;
        }
        peers.entry(a).or_default().push((b, count));
        peers.entry(b).or_default().push((a, count));
    }
    for (region, list) in peers.iter_mut() {
        let _ = region;
        list.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        list.truncate(MAX_NEIGHBOR_EDGES);
    }
    let mut overlap_fragment = Vec::new();
    let mut emitted_neighbor_pairs = BTreeSet::new();
    for (&a, list) in &peers {
        for &(b, _) in list {
            let (src, dst) = if a < b { (a, b) } else { (b, a) };
            if !emitted_neighbor_pairs.insert((src, dst)) {
                continue;
            }
            overlap_fragment.push(CoverEdge {
                src: region_node_id(src),
                kind: EDGE_KIND_NEIGHBOR,
                dst: region_node_id(dst),
            });
        }
    }

    // Forward transition edges (E_f)
    let mut transitions: BTreeMap<(u32, u32), u64> = BTreeMap::new();
    for i in 0..observations.len().saturating_sub(1) {
        let obs_a = &observations[i];
        let obs_b = &observations[i + 1];
        let story_a = story_map.get(obs_a.position as usize).copied();
        let story_b = story_map.get(obs_b.position as usize).copied();
        if obs_a.position + 1 == obs_b.position && story_a.is_some() && story_a == story_b {
            for depth in 1..=cover.max_depth {
                let mems_a = reference.binary_memberships(depth, &obs_a.sig);
                let mems_b = reference.binary_memberships(depth, &obs_b.sig);
                for &a in &mems_a {
                    for &b in &mems_b {
                        *transitions.entry((a, b)).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let mut transition_lists: BTreeMap<u32, Vec<(u32, u64)>> = BTreeMap::new();
    for (&(a, b), &count) in &transitions {
        transition_lists.entry(a).or_default().push((b, count));
    }
    for list in transition_lists.values_mut() {
        list.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        list.truncate(MAX_TRANSITION_EDGES);
    }
    let mut transition_fragment = Vec::new();
    for (&a, list) in &transition_lists {
        for &(b, _) in list {
            transition_fragment.push(CoverEdge {
                src: region_node_id(a),
                kind: EDGE_KIND_TRANSITION,
                dst: region_node_id(b),
            });
        }
    }

    canonical_merge_edge_fragments(&[refinement_fragment, overlap_fragment, transition_fragment])
}

/// Held-out evaluation numbers (all rates over evaluated positions).
#[derive(Debug, Clone, Default, Serialize)]
pub struct DepthRecall {
    pub depth: usize,
    /// Positions with at least one region at this depth.
    pub evaluated: usize,
    /// P(binary top-1 == exact reference top-1).
    pub reference_top1_recall: f64,
    /// P(exact reference top-1 ∈ binary top-M membership).
    pub reference_topm_recall: f64,
    /// Positions with a non-empty train class cell (co-assignment base).
    pub coassignment_evaluated: usize,
    /// Co-assignment recall vs the 4×256 class cover, binary top-1:
    /// mean over positions of the share of train observations routed to
    /// the same full class code that the cover routes to the same region.
    pub class_coassignment_recall_top1: f64,
    /// Same with the position's top-M region set as the target.
    pub class_coassignment_recall_topm: f64,
    /// Co-assignment precision, top-1: mean share of region co-members
    /// that share the position's full class code.
    pub class_coassignment_precision_top1: f64,
    /// Co-assignment precision, top-M.
    pub class_coassignment_precision_topm: f64,
    /// Mean active-region count (frontier width) at this depth.
    pub frontier_width_mean: f64,
    /// Max active-region count at this depth.
    pub frontier_width_max: u32,
}

/// Held-out evaluation of an induced cover against the frozen reference
/// classifier and the incumbent 4×256 class cover.
///
/// `train_class_codes[i]` is the full 4-stage class code of train
/// observation `i` (`runtime::assign_plain` of its signature).
pub fn evaluate_held_out(
    art: &compiler::Compiled,
    cover: &Cover,
    reference: &ReferenceClassifier,
    train: &[Observation],
    held_out: &[Observation],
) -> Vec<DepthRecall> {
    // Train indexes: class cell → members; (region per depth) → count;
    // (region, class cell) → count.
    let mut class_members: BTreeMap<[u8; STAGES], Vec<usize>> = BTreeMap::new();
    let mut train_top1: Vec<Vec<u32>> = Vec::with_capacity(train.len());
    let mut region_counts: BTreeMap<(usize, u32), u64> = BTreeMap::new();
    let mut region_class_counts: BTreeMap<(usize, u32, [u8; STAGES]), u64> = BTreeMap::new();
    for (i, observation) in train.iter().enumerate() {
        let code = runtime::assign_plain(art, &observation.sig);
        class_members.entry(code).or_default().push(i);
        let mut top1 = Vec::with_capacity(cover.max_depth);
        for depth in 1..=cover.max_depth {
            let memberships = reference.binary_memberships(depth, &observation.sig);
            let t1 = memberships.first().copied().unwrap_or(u32::MAX);
            top1.push(t1);
            if t1 != u32::MAX {
                *region_counts.entry((depth, t1)).or_insert(0) += 1;
                *region_class_counts.entry((depth, t1, code)).or_insert(0) += 1;
            }
        }
        train_top1.push(top1);
    }

    let mut reports = Vec::new();
    for depth in 1..=cover.max_depth {
        let mut report = DepthRecall {
            depth,
            ..DepthRecall::default()
        };
        let mut top1_hits = 0u64;
        let mut topm_hits = 0u64;
        let mut recall1_sum = 0f64;
        let mut recallm_sum = 0f64;
        let mut precision1_sum = 0f64;
        let mut precisionm_sum = 0f64;
        let mut frontier_sum = 0u64;
        for observation in held_out {
            let memberships = reference.binary_memberships(depth, &observation.sig);
            if memberships.is_empty() {
                continue;
            }
            report.evaluated += 1;
            let exact = reference.exact_top1(depth, &observation.vector);
            if let Some(exact) = exact {
                if memberships.first() == Some(&exact) {
                    top1_hits += 1;
                }
                if memberships.contains(&exact) {
                    topm_hits += 1;
                }
            }
            frontier_sum += memberships.len() as u64;
            report.frontier_width_max = report.frontier_width_max.max(memberships.len() as u32);
            let top1 = memberships.first().copied().unwrap_or(u32::MAX);

            // Co-assignment vs the full class code of this position.
            let code = runtime::assign_plain(art, &observation.sig);
            let Some(cell) = class_members.get(&code) else {
                continue;
            };
            report.coassignment_evaluated += 1;
            let mut same_region1 = 0u64;
            let mut same_regionm = 0u64;
            for &y in cell {
                let y_top1 = train_top1[y][depth - 1];
                if y_top1 == top1 {
                    same_region1 += 1;
                }
                if memberships.contains(&y_top1) {
                    same_regionm += 1;
                }
            }
            let cell_size = cell.len() as f64;
            recall1_sum += same_region1 as f64 / cell_size;
            recallm_sum += same_regionm as f64 / cell_size;
            if let Some(&region_total) = region_counts.get(&(depth, top1)) {
                let same_class = region_class_counts
                    .get(&(depth, top1, code))
                    .copied()
                    .unwrap_or(0);
                precision1_sum += same_class as f64 / region_total as f64;
            }
            let regionm_total: u64 = memberships
                .iter()
                .map(|id| region_counts.get(&(depth, *id)).copied().unwrap_or(0))
                .sum();
            if regionm_total > 0 {
                let same_class: u64 = memberships
                    .iter()
                    .map(|id| {
                        region_class_counts
                            .get(&(depth, *id, code))
                            .copied()
                            .unwrap_or(0)
                    })
                    .sum();
                precisionm_sum += same_class as f64 / regionm_total as f64;
            }
        }
        if report.evaluated > 0 {
            let n = report.evaluated as f64;
            report.reference_top1_recall = top1_hits as f64 / n;
            report.reference_topm_recall = topm_hits as f64 / n;
            report.frontier_width_mean = frontier_sum as f64 / n;
        }
        if report.coassignment_evaluated > 0 {
            let n = report.coassignment_evaluated as f64;
            report.class_coassignment_recall_top1 = recall1_sum / n;
            report.class_coassignment_recall_topm = recallm_sum / n;
            report.class_coassignment_precision_top1 = precision1_sum / n;
            report.class_coassignment_precision_topm = precisionm_sum / n;
        }
        reports.push(report);
    }
    reports
}

/// Root prior of the cover: the train next-token distribution (the
/// corpus level-0 backoff distribution), ascending token order.
pub fn root_prior(observations: &[Observation]) -> BTreeMap<u32, u32> {
    let mut prior: BTreeMap<u32, u32> = BTreeMap::new();
    for observation in observations {
        *prior.entry(observation.next).or_insert(0) += 1;
    }
    prior
}

/// What an [`emit_r4g1`] call produced, for the report and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverArtifactInfo {
    pub node_count: u32,
    pub edge_count: u32,
    pub refinement_edges: u32,
    pub neighbor_edges: u32,
    pub transition_edges: u32,
    pub depth_count: u8,
    pub max_frontier_width: u16,
    pub root_prior_entries: u32,
    pub artifact_bytes: usize,
}

/// Emit the induced cover as an R4G1 container, reusing the
/// `convert_r4g1` HEAD/section conventions (module docs). Fails closed:
/// the bytes are re-validated with `GraphView::parse` + `verify_cids`
/// before they are returned.
pub fn emit_r4g1(
    artifact_container: &[u8],
    corpus_cid_material: (&[u8], &[u8]),
    vocab_size: u32,
    cover: &Cover,
    edges: &[CoverEdge],
    prior: &BTreeMap<u32, u32>,
    observations: &[Observation],
) -> Result<(Vec<u8>, CoverArtifactInfo), uor_r4_graph_format::ObservedBound> {
    let node_count = 1 + cover.regions.len() as u32;
    let depth_count = (cover.max_depth + 1) as u8;
    let edge_count = edges.len() as u32;
    let refinement_edges = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_REFINEMENT)
        .count() as u32;
    let neighbor_edges = edge_count - refinement_edges;

    // Per-node child ranges (refinement runs are contiguous under the
    // (src, kind, dst) canonical sort) and the reverse index.
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
    // The root record keeps all ranges empty (converter convention), so
    // the observed max runs over the wired region records only.
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

    let mut rout = crate::routing::synthesize_routing_program(cover, observations);
    while !rout.len().is_multiple_of(8) {
        rout.push(0x00);
    }

    let prototype_words_start = (rout.len() / 8) as u32;
    let sig_words = SIG_WORDS as u32;

    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root prototype
    for region in &cover.regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].copy_from_slice(&region.sig);
        rout.extend_from_slice(&words);
    }

    let mask_words_start = (rout.len() / 8) as u32;
    rout.extend_from_slice(&[0u8; SIG_WORDS * 8]); // root mask
    for _ in &cover.regions {
        let mut words = [0u8; SIG_WORDS * 8];
        words[..SIG_BYTES].fill(0xFF);
        rout.extend_from_slice(&words);
    }

    // EMIT: descriptor + the v0 linear-count root prior block + region token residuals.
    let mut emit = vec![2u8, 0, 0, 0]; // {width: i32, shift: 0, zero_point: 0}
    let mut root_prior_entries = 0u32;
    for (&token, &count) in prior {
        // The wire encodes root-prior tokens and counts as i32. A value past
        // that is a property of the caller's chosen corpus/vocab, not a
        // limitation of the emitter, so it is reported as the observed bound.
        let token = i32::try_from(token).map_err(|_| uor_r4_graph_format::ObservedBound {
            observed: token as i64,
            bound: i32::MAX as i64,
        })?;
        let count = i32::try_from(count).map_err(|_| uor_r4_graph_format::ObservedBound {
            observed: count as i64,
            bound: i32::MAX as i64,
        })?;
        emit.extend_from_slice(&token.to_le_bytes());
        emit.extend_from_slice(&count.to_le_bytes());
        root_prior_entries += 1;
    }

    let mut emission_starts = vec![0u32; cover.regions.len() + 1];
    let mut emission_lens = vec![0u16; cover.regions.len() + 1];

    for (index, _) in cover.regions.iter().enumerate() {
        let i = 1 + index;
        let mut freq: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut bigram_freq: std::collections::HashMap<(u32, u32), u32> =
            std::collections::HashMap::new();
        for &obs_idx in &cover.members[index] {
            if let Some(obs) = observations.get(obs_idx) {
                let prev_token = obs.prev;
                let next_token = obs.next;
                *freq.entry(next_token).or_insert(0) += 1;
                *bigram_freq.entry((prev_token, next_token)).or_insert(0) += 1;
            }
        }
        let mut candidate_weights: std::collections::HashMap<u32, u32> =
            std::collections::HashMap::new();
        for (&next_token, &count) in &freq {
            let mut weight = count * 10;
            for (&(prev, next), &bcount) in &bigram_freq {
                if next == next_token && prev != 0 {
                    weight += bcount * 25;
                }
            }
            candidate_weights.insert(next_token, weight);
        }
        let mut sorted: Vec<_> = candidate_weights.into_iter().collect();
        sorted.sort_by_key(|&(_, weight)| std::cmp::Reverse(weight));
        sorted.truncate(64); // max E = 64

        let start_in_remainder = (emit.len() - 4) as u32;
        let mut entry_bytes = 0u16;
        for (token, count) in sorted {
            // Same i32 wire bound as the root prior above (reported, not refused).
            let token = i32::try_from(token).map_err(|_| uor_r4_graph_format::ObservedBound {
                observed: token as i64,
                bound: i32::MAX as i64,
            })?;
            let count = i32::try_from(count).map_err(|_| uor_r4_graph_format::ObservedBound {
                observed: count as i64,
                bound: i32::MAX as i64,
            })?;
            emit.extend_from_slice(&token.to_le_bytes());
            emit.extend_from_slice(&count.to_le_bytes());
            entry_bytes += 8;
        }
        emission_starts[i] = start_in_remainder;
        emission_lens[i] = entry_bytes;
    }

    emission_starts[0] = 0; // Relative to EMIT remainder (starts after 4-byte StorageDescriptor)
    emission_lens[0] = (root_prior_entries * 8) as u16;

    // NODE: the root record is empty of children/edges, but holds the global emission prior
    let mut node_section = Vec::with_capacity(node_total * 30);
    node_section.extend_from_slice(&0u32.to_le_bytes()); // child_start
    node_section.extend_from_slice(&0u16.to_le_bytes()); // child_len
    node_section.extend_from_slice(&0u32.to_le_bytes()); // forward_start
    node_section.extend_from_slice(&0u16.to_le_bytes()); // forward_len
    node_section.extend_from_slice(&emission_starts[0].to_le_bytes());
    node_section.extend_from_slice(&emission_lens[0].to_le_bytes());
    node_section.extend_from_slice(&0u32.to_le_bytes()); // prototype_words_start
    node_section.extend_from_slice(&0u32.to_le_bytes()); // mask_words_start
    node_section.extend_from_slice(&0u16.to_le_bytes()); // radius
    node_section.push(0); // depth
    node_section.push(0); // flags
    for (index, region) in cover.regions.iter().enumerate() {
        let i = 1 + index;
        node_section.extend_from_slice(&child_start[i].to_le_bytes());
        node_section.extend_from_slice(&child_len[i].to_le_bytes());
        node_section.extend_from_slice(&forward_start[i].to_le_bytes());
        node_section.extend_from_slice(&forward_len[i].to_le_bytes());
        node_section.extend_from_slice(&emission_starts[i].to_le_bytes());
        node_section.extend_from_slice(&emission_lens[i].to_le_bytes());
        node_section
            .extend_from_slice(&(prototype_words_start + (i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&(mask_words_start + (i as u32) * sig_words).to_le_bytes());
        node_section.extend_from_slice(&region.radius.to_le_bytes());
        node_section.push(region.depth);
        node_section.push(0); // flags
    }

    // EDGE: canonical records (score_q 0 — v1 carries no log-domain edge
    // scores; co-activation counts live in the report, not the wire)
    // followed by the reverse index.
    let mut edge_section = Vec::with_capacity(edges.len() * 20);
    for edge in edges {
        edge_section.extend_from_slice(&edge.src.to_le_bytes());
        edge_section.extend_from_slice(&edge.dst.to_le_bytes());
        edge_section.extend_from_slice(&0i32.to_le_bytes()); // score_q
        edge_section.push(edge.kind);
        edge_section.push(0); // flags
        edge_section.extend_from_slice(&0u16.to_le_bytes()); // reserved
    }
    for &id in &reverse {
        edge_section.extend_from_slice(&id.to_le_bytes());
    }

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
    let max_emission_entries = emission_lens
        .iter()
        .copied()
        .max()
        .unwrap_or(DEFAULT_MAX_EMISSION_ENTRIES as u16) as u32;
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

    // Phase 8: Emit a placeholder CODE section containing a single HALT instruction.
    let code_section = vec![0x00u8]; // OP_HALT

    let mut builder = uor_r4_graph_format::ArtifactBuilder::new(7);
    builder.add_section(uor_r4_graph_format::SectionId::HEAD, 0, &head);
    builder.add_section(uor_r4_graph_format::SectionId::CODE, 0, &code_section);
    builder.add_section(uor_r4_graph_format::SectionId::NODE, 0, &node_section);
    builder.add_section(uor_r4_graph_format::SectionId::EDGE, 0, &edge_section);
    builder.add_section(uor_r4_graph_format::SectionId::ROUT, 0, &rout);
    builder.add_section(uor_r4_graph_format::SectionId::EMIT, 0, &emit);
    // These bytes were just serialized from the compiler's own cover; a
    // serialization or re-validation failure is a defect in this emitter or the
    // format layer, never a property of the caller's input, so it is an
    // infallible-construction invariant that panics rather than a reported
    // condition. Fail closed: never emit an artifact the two-stage validator or
    // the integrity CIDs reject.
    let bytes = builder
        .build()
        .unwrap_or_else(|error| panic!("R4G1 serialization of the emitted cover failed: {error}"));
    let view = uor_r4_graph_format::GraphView::parse(&bytes).unwrap_or_else(|error| {
        panic!("the emitter produced bytes that are not a valid R4G1 artifact: {error}")
    });
    view.verify_cids().unwrap_or_else(|error| {
        panic!("the emitter produced an artifact whose integrity CIDs do not reproduce: {error}")
    });

    let artifact_len = bytes.len();
    Ok((
        bytes,
        CoverArtifactInfo {
            node_count,
            edge_count,
            refinement_edges,
            neighbor_edges,
            transition_edges: edge_count - refinement_edges - neighbor_edges,
            depth_count,
            max_frontier_width,
            root_prior_entries,
            artifact_bytes: artifact_len,
        },
    ))
}

/// The human-readable recall/stability report (`cover_report.json`).
#[derive(Debug, Clone, Serialize)]
pub struct CoverReport {
    pub schema: u32,
    pub config: CoverReportConfig,
    pub inputs: CoverReportInputs,
    pub objective: CoverReportObjective,
    pub regions: CoverReportRegions,
    pub edges: CoverReportEdges,
    pub reference_classifier: CoverReportReference,
    pub recall: Vec<DepthRecall>,
    pub determinism: CoverReportDeterminism,
    pub artifact: Option<CoverReportArtifact>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportConfig {
    pub depths: usize,
    pub k0: usize,
    pub regions_budget: usize,
    pub memory_budget_bytes: u64,
    pub threads: u32,
    pub min_support: usize,
    pub entropy_gain_bits: f64,
    pub split_children: usize,
    pub top_m: usize,
    pub radius_quantile: String,
    pub batch_size: usize,
    pub bytes_per_observation: u64,
    pub memory_reserve_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportInputs {
    pub artifact_kappa: String,
    pub corpus_kappa: String,
    pub train_observations: usize,
    pub held_out_observations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportObjective {
    pub config: ObjectiveConfig,
    pub train: ObjectiveComponents,
    pub held_out: ObjectiveComponents,
    pub tradeoff_held_out_minus_train: ObjectiveTradeoffDelta,
    pub region_decisions: Vec<RegionDecisionAudit>,
    pub migration: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveTradeoffDelta {
    pub predictive_entropy_bits: f64,
    pub teacher_loss_bits: f64,
    pub future_state_entropy_bits: f64,
    pub runtime_cost_units: f64,
    pub artifact_size_bytes: f64,
    pub bytes_read: f64,
    pub structural_complexity: f64,
    pub between_region_distinctiveness: f64,
    pub ib_objective_bits: f64,
    pub weighted_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportRegions {
    pub total: usize,
    pub per_depth: Vec<u32>,
    pub splits: usize,
    pub leaves: usize,
    pub split_gains_bits: Vec<f64>,
    pub support_min: u32,
    pub support_max: u32,
    pub radius_min: u16,
    pub radius_max: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportEdges {
    pub refinement: u32,
    pub neighbor: u32,
    pub coactivation_min: u64,
    pub degree_cap: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportReference {
    pub kappa: String,
    pub semantics: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportDeterminism {
    pub note: String,
    pub cover_kappa: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverReportArtifact {
    pub bytes: usize,
    pub kappa: String,
    pub node_count: u32,
    pub edge_count: u32,
}

/// Data bundle for [`build_report`] (keeps the argument list focused).
pub struct ReportData<'a> {
    pub reference: &'a ReferenceClassifier,
    pub train: &'a [Observation],
    pub held_out: &'a [Observation],
    pub edges: &'a [CoverEdge],
    pub recall: Vec<DepthRecall>,
    pub artifact: Option<(&'a [u8], CoverArtifactInfo)>,
}

/// Assemble the report from a finished run.
pub fn build_report(config: &CoverConfig, induced: &InducedCover, data: ReportData) -> CoverReport {
    let ReportData {
        reference,
        train,
        held_out,
        edges,
        recall,
        artifact,
    } = data;
    let cover = &induced.cover;
    let mut per_depth = vec![0u32; cover.max_depth];
    for region in &cover.regions {
        per_depth[region.depth as usize - 1] += 1;
    }
    let splits = cover
        .regions
        .iter()
        .filter(|r| !r.children.is_empty())
        .count();
    let split_gains: Vec<f64> = cover
        .regions
        .iter()
        .filter(|r| !r.children.is_empty())
        .map(|r| r.split_gain_bits)
        .collect();
    let refinement = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_REFINEMENT)
        .count() as u32;
    let structural_complexity = (cover.regions.len() + edges.len()) as f64;
    let artifact_size_bytes = artifact
        .as_ref()
        .map(|(bytes, _)| bytes.len() as f64)
        .unwrap_or(cover.regions.len() as f64 * APPROX_BYTES_PER_ADDED_REGION);
    let train_runtime_cost = if train.is_empty() {
        0.0
    } else {
        cover
            .paths
            .iter()
            .map(|path| path.len() as f64)
            .sum::<f64>()
            / train.len() as f64
    };
    let held_out_runtime_cost = if recall.is_empty() {
        0.0
    } else {
        recall
            .iter()
            .map(|depth| depth.frontier_width_mean)
            .sum::<f64>()
            / recall.len() as f64
    };
    let train_assignments: Vec<u32> = cover
        .paths
        .iter()
        .map(|path| path.last().copied().unwrap_or(0))
        .collect();
    let held_out_assignments: Vec<u32> = held_out
        .iter()
        .map(|observation| {
            let mut assignment = 0u32;
            for depth in 1..=cover.max_depth {
                if let Some(region) = reference.exact_top1(depth, &observation.vector) {
                    assignment = region;
                }
            }
            assignment
        })
        .collect();
    let objective_train = objective_for_partition(
        &config.objective,
        train,
        &train_assignments,
        ObjectiveCostTerms {
            runtime_cost_units: train_runtime_cost,
            artifact_size_bytes,
            bytes_read: train_runtime_cost * APPROX_BYTES_READ_PER_REGION,
            structural_complexity,
        },
    );
    let objective_held_out = objective_for_partition(
        &config.objective,
        held_out,
        &held_out_assignments,
        ObjectiveCostTerms {
            runtime_cost_units: held_out_runtime_cost,
            artifact_size_bytes,
            bytes_read: held_out_runtime_cost * APPROX_BYTES_READ_PER_REGION,
            structural_complexity,
        },
    );
    let mut determinism_note = String::new();
    let _ = write!(
        determinism_note,
        "observation extraction uses up to {} bounded workers; k-means and reductions are \
         ordered (batch-id order, f64 accumulators), seeding is content-addressed; identical \
         inputs produce byte-identical artifacts regardless of thread count",
        config.threads
    );
    CoverReport {
        schema: 2,
        config: CoverReportConfig {
            depths: config.depths,
            k0: config.k0,
            regions_budget: config.regions_budget,
            memory_budget_bytes: config.memory_budget_bytes,
            threads: config.threads,
            min_support: config.min_support,
            entropy_gain_bits: config.entropy_gain_bits,
            split_children: SPLIT_CHILDREN,
            top_m: TOP_M,
            radius_quantile: format!(
                "{}/{}",
                config.radius_quantile_numerator, config.radius_quantile_denominator
            ),
            batch_size: induced.batch_size,
            bytes_per_observation: BYTES_PER_OBSERVATION,
            memory_reserve_bytes: MEMORY_RESERVE_BYTES,
        },
        inputs: CoverReportInputs {
            artifact_kappa: induced.artifact_kappa.clone(),
            corpus_kappa: induced.corpus_kappa.clone(),
            train_observations: train.len(),
            held_out_observations: held_out.len(),
        },
        objective: CoverReportObjective {
            config: config.objective.clone(),
            train: objective_train,
            held_out: objective_held_out,
            tradeoff_held_out_minus_train: ObjectiveTradeoffDelta {
                predictive_entropy_bits: objective_held_out.predictive_entropy_bits
                    - objective_train.predictive_entropy_bits,
                teacher_loss_bits: objective_held_out.teacher_loss_bits
                    - objective_train.teacher_loss_bits,
                future_state_entropy_bits: objective_held_out.future_state_entropy_bits
                    - objective_train.future_state_entropy_bits,
                runtime_cost_units: objective_held_out.runtime_cost_units
                    - objective_train.runtime_cost_units,
                artifact_size_bytes: objective_held_out.artifact_size_bytes
                    - objective_train.artifact_size_bytes,
                bytes_read: objective_held_out.bytes_read - objective_train.bytes_read,
                structural_complexity: objective_held_out.structural_complexity
                    - objective_train.structural_complexity,
                between_region_distinctiveness: objective_held_out.between_region_distinctiveness
                    - objective_train.between_region_distinctiveness,
                ib_objective_bits: objective_held_out.ib_objective_bits
                    - objective_train.ib_objective_bits,
                weighted_score: objective_held_out.weighted_score - objective_train.weighted_score,
            },
            region_decisions: induced.decision_trace.clone(),
            migration: "Objective configuration is versioned via objective.config.schema. Future \
                        objective versions append fields under objective while preserving Gate C \
                        and predictive-sufficiency reports as separate, reproducible artifacts."
                .to_owned(),
        },
        regions: CoverReportRegions {
            total: cover.regions.len(),
            per_depth,
            splits,
            leaves: cover.regions.len() - splits,
            split_gains_bits: split_gains,
            support_min: cover.regions.iter().map(|r| r.support).min().unwrap_or(0),
            support_max: cover.regions.iter().map(|r| r.support).max().unwrap_or(0),
            radius_min: cover.regions.iter().map(|r| r.radius).min().unwrap_or(0),
            radius_max: cover.regions.iter().map(|r| r.radius).max().unwrap_or(0),
        },
        edges: CoverReportEdges {
            refinement,
            neighbor: edges.len() as u32 - refinement,
            coactivation_min: coactivation_min(train.len()),
            degree_cap: MAX_NEIGHBOR_EDGES,
        },
        reference_classifier: CoverReportReference {
            kappa: reference.kappa().to_owned(),
            semantics: "exact compiler-side membership (nearest prototype by cosine); the \
                        shipped binary path is top-M masked-Hamming within calibrated radii \
                        with nearest-region fallback"
                .to_owned(),
        },
        recall,
        determinism: CoverReportDeterminism {
            note: determinism_note,
            cover_kappa: cover.kappa(),
        },
        artifact: artifact.map(|(bytes, info)| CoverReportArtifact {
            bytes: bytes.len(),
            kappa: format!("blake3:{}", blake3::hash(bytes).to_hex()),
            node_count: info.node_count,
            edge_count: info.edge_count,
        }),
    }
}

#[cfg(test)]
mod compiler_invariant_tests {
    use std::collections::BTreeMap;

    use super::*;

    use uor_r4_graph_format::invariant_ownership::{
        GraphInvariantId, GraphInvariantOwnershipMatrix, InvariantOwner,
    };

    #[test]
    fn test_compiler_stage4_evidence_non_duplication_invariant() {
        let matrix = GraphInvariantOwnershipMatrix::get_matrix();
        let stage4_entry = matrix
            .iter()
            .find(|e| e.invariant_id == GraphInvariantId::EvidenceNonDuplication)
            .expect("EvidenceNonDuplication entry");

        assert_eq!(
            stage4_entry.primary_owner,
            InvariantOwner::CompilerConstruction
        );
        assert_eq!(stage4_entry.validation_stage, "Compiler Stage 4");
    }

    #[test]
    fn distinctiveness_is_bounded_deterministic_and_rewarded() {
        let mut child = BTreeMap::new();
        child.insert(7, 10);
        child.insert(8, 9);
        let mut prior = BTreeMap::new();
        prior.insert(1, 100);
        prior.insert(2, 90);
        assert_eq!(top_token_distinctiveness(&child, &prior), 1.0);

        prior.insert(7, 110);
        assert_eq!(top_token_distinctiveness(&child, &prior), 0.5);

        let mut config = ObjectiveConfig::default();
        config.weights.between_region_distinctiveness = 2.0;
        let components = ObjectiveComponents::weighted(
            &config,
            ObjectiveRawValues {
                predictive_entropy_bits: 0.0,
                future_state_entropy_bits: 0.0,
                teacher_loss_bits: 0.0,
                runtime_cost_units: 0.0,
                artifact_size_bytes: 0.0,
                bytes_read: 0.0,
                structural_complexity: 0.0,
                between_region_distinctiveness: 0.5,
                ib_i_zx_bits: 0.0,
                ib_i_zy_future_bits: 0.0,
            },
        );
        assert_eq!(components.between_region_distinctiveness, 0.5);
        assert_eq!(components.weighted_score, -1.0);
    }
}
