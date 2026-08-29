//! Source attention operator specification (#602): the typed, versioned
//! record of the attention operator a source teacher computes, plus the
//! factored reference implementations themselves.
//!
//! Before #602 the teacher's attention operator was a boolean
//! (`Config::r4_attention`) selecting between two in-line loop bodies in
//! `Llama::layer_forward`/`Llama::forward_batch`, and the experimental
//! branch was described in stale wording that #515's audit records as not
//! matching its control flow. This module makes the operator identity
//! explicit and truthful:
//!
//! - [`AttentionOperatorSpec`] is the serializable record `{id, version,
//!   projections, positional_action, compatibility_relation,
//!   selector_normalization, value_aggregation, output_projection,
//!   runtime_state, tie_breaking, permitted_operation_class, params}`
//!   carried by the observation manifest and the cover/compile report.
//! - [`standard_head_attention_weights`],
//!   [`experimental_r4_head_attention_weights`], and
//!   [`head_attention_value_aggregate`] implement the current version-2
//!   source operators. Every raw Q·K and weighted-value dot returns the
//!   correctly-rounded binary32 exact-real result: a native f64 sum is used
//!   only when an outward interval lies strictly inside one binary32 RNE
//!   cell; all other lanes use the pinned exact `uor-matmul` owner.
//! - Version-1 entries remain immutable registry metadata for historical
//!   artifacts. They name the former sequential/chunked f32 folds and remain
//!   readable, but current source executors never silently resume as v1.
//! - [`operator_spec`] is the versioned registry mapping `(id, version)`
//!   to a spec; an unknown pair is refused by name on the sanctioned
//!   [`SourceUnavailable`](crate::SourceUnavailable) surface rather than
//!   guessed.
//!
//! **Truthful inventory of the two branches** (read from
//! `Llama::layer_forward` / `Llama::forward_batch`; the unit tests below
//! pin every claim):
//!
//! Both branches share: dense per-layer f32 `wq`/`wk`/`wv` projections
//! with grouped-query key/value sharing (head `h` reads kv head
//! `h / kv_mul` where `kv_mul = n_heads / n_kv_heads`); RoPE rotation of
//! q and k applied before any score (interleaved-pair or split-half
//! layout is a source-config property, not an operator property); a
//! growing KV cache attending the full prefix `0..=pos`; correctly-rounded
//! exact-real value dots over the full prefix; and a dense per-layer f32
//! `wo` output projection.
//! Both branches normalize scores with the SAME selector,
//! `softmax_with_mode`: subtract the maximum score (first maximum on
//! ties — value-identical, since only the maximum's value enters), then
//! `exp` each shifted score (`libm::expf` in canonical mode, `f32::exp`
//! otherwise) and divide by the sum. Neither branch performs an argmax
//! or any selection needing a tie-break beyond that stabilizer.
//!
//! They differ ONLY in the compatibility relation:
//!
//! - **standard**: `score(t) = RN32(Σ_{i<H} q[i]·k_t[i]) / sqrt(H)` for
//!   head width `H`. The exact dot rounds once to binary32, then Llama's
//!   historical per-score divide and per-weight softmax-sum divide remain.
//! - **experimental**: `score(t) = (Σ_{c<⌊H/4⌋} Σ_{j<4}
//!   q[4c+j]·k_t[4c+j]) / sqrt(H)` — one correctly-rounded exact-real dot
//!   over the historical floor-multiple-of-four domain, followed by the same
//!   divide and softmax. **Remainder policy**: the trailing `H mod 4` q/k
//!   dimensions are never read by the score (dropped), while the scale
//!   divides by `sqrt(H)` over the FULL head width and value aggregation
//!   still uses every head dimension. For `H < 4` no chunk exists, every
//!   score is 0, and the softmax yields uniform weights over the prefix.
//!   For `H` divisible by 4 both operators score the same exact-real dot.
//!
//! **Implementation digest.** As with the #600 geometry record and the
//! #601 tokenizer-adapter record, `implementation_digest` is the blake3
//! of the [canonical serialization](AttentionOperatorSpec::canonical_bytes)
//! of the operator's *declared identity* — NOT a hash of source code
//! text, which would churn under refactors that leave the arithmetic
//! bit-identical. The unit tests pin the implementations to the
//! declarations, so a behavioral change must bump the version (a new
//! registry entry) instead of silently drifting.
//!
//! **Boundary note.** The three SOURCE specs (the two Llama switch
//! branches plus GPT-2's learned-absolute operator) describe HOST-SIDE
//! source-teacher computation (f32/f64 certified arithmetic, pinned exact
//! fallback, exp, and division). They
//! are provenance records, distinct from — and outside — the deployed
//! inference operation contract
//! (`docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`), which
//! forbids floating-point arithmetic on the deployed hot path and
//! explicitly excludes teacher execution. #602 defined no target
//! (deployed) operator.
//!
//! **Target operator (#604).** `r4-route-attention/1`
//! ([`AttentionOperatorSpec::r4_route_attention_v1`]) is the FIRST
//! registered TARGET operator: `R4RouteAttentionV1`, whose
//! `permitted_operation_class` is the deployed integer class (XOR /
//! masked popcount via table / saturating integer add / compare / table
//! read — no float, no multiply, no divide), unlike the source
//! records above. It reuses NO Q/K/V weights: its route codes, mask,
//! and ScoreQ contributions are declared tables over the 288-bit
//! signature substrate. Source-teacher and target routing semantics
//! remain SEPARATE operators: [`operator_for_r4_switch`] still maps the
//! legacy boolean onto exactly the two source operators and never
//! selects the target one. The operator is DORMANT
//! (`r4-route-attention-dormant` in `model/ledger.toml`): its reference
//! semantics live in `uor-r4-graph-certify::route_attention`, its
//! packed lowering in `uor-r4-graph-runtime::route_attention`, and
//! nothing in the serving path constructs it.

use serde::{Deserialize, Serialize};

/// Layers whose full-prefix causal attention is expressed through an injected
/// coordinate transport and row operator.
///
/// `All` is the reference geometric-attention configuration. `Selected` is a
/// bounded diagnostic surface: every named layer retains learned Q/K/V, RoPE,
/// the complete prefix, and Wo. Default hooks retain ordinary softmax and the
/// linear value aggregate; a declared implementation may replace them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalAttentionLayerSelection {
    /// Transport attention at every decoder layer.
    All,
    /// Transport attention at exactly these zero-based decoder layers.
    Selected(Vec<usize>),
}

/// Immutable coordinates identifying one query head at one causal position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalAttentionHeadContext {
    /// Zero-based decoder layer.
    pub layer: usize,
    /// Zero-based query-head index.
    pub head: usize,
    /// Current causal query position.
    pub query_position: usize,
}

/// Immutable coordinates identifying one cached source vector consumed by a
/// query head.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalAttentionSourceContext {
    /// Zero-based decoder layer.
    pub layer: usize,
    /// Zero-based query-head index. Grouped-query selection has already
    /// selected the corresponding KV head before this hook is called.
    pub head: usize,
    /// Current causal query position.
    pub query_position: usize,
    /// Cached key/value position, always in `0..=query_position`.
    pub source_position: usize,
}

/// Object-safe coordinate-transport and attention-operator seam around dense
/// causal attention.
///
/// The decoder retains ownership of learned Q/K/V and Wo projections, RoPE,
/// the complete causal prefix, residuals, FFN, final norm, and the LM head.
/// The default score/normalization and value-aggregation hooks preserve the
/// ordinary scaled-dot-product, stable-softmax, linear-value operator exactly.
/// Implementations may instead declare an intrinsic compatibility function or
/// geometric weighted centroid while retaining every surrounding decoder
/// operation:
///
/// 1. [`transform_query`](Self::transform_query) maps the current query;
/// 2. [`transport_key`](Self::transport_key) and
///    [`transport_value`](Self::transport_value) map every admitted cached
///    source from its frame to the query frame;
/// 3. [`score_and_normalize`](Self::score_and_normalize) assigns normalized
///    weights over the complete packed causal prefix;
/// 4. [`weighted_value_centroid`](Self::weighted_value_centroid) aggregates
///    the packed query-frame values; and
/// 5. [`output_to_model_frame`](Self::output_to_model_frame) maps the dense
///    aggregate back before the unchanged Wo projection.
///
/// All input and output slices have the model's complete head width. The
/// session constructor requires that width to be a non-zero multiple of four,
/// so an R4/Spin implementation can process `chunks_exact(4)` without dropping
/// remainder lanes. Implementations must fill every output lane and must not
/// retain any borrowed slice.
pub trait CausalAttentionTransport: Send {
    /// Reset implementation-owned causal state for a fresh sequence.
    fn reset(&mut self) {}

    /// Stable implementation/policy identity recorded with decoder evidence.
    fn policy_identity(&self) -> &str {
        "unspecified-causal-attention-transport"
    }

    /// Deterministic implementation-owned evidence for a completed probe.
    ///
    /// The source decoder intentionally does not know an implementation's
    /// geometry-specific audit type. Implementations may therefore return a
    /// canonical JSON object whose schema is owned by that implementation.
    /// Decision-bearing callers record and validate it alongside the generic
    /// decoder audit. A transport without additional evidence returns `None`.
    fn implementation_evidence(&self) -> Result<Option<String>, String> {
        Ok(None)
    }

    /// Current implementation health. A transport that handles an internal
    /// arithmetic or frame failure without panicking records it here; the
    /// public decoder step then fails closed and withholds logits.
    fn status(&self) -> Result<(), String> {
        Ok(())
    }

    /// Admit the current token once, before any selected layer processes it.
    fn begin_position(&mut self, token: usize, position: usize);

    /// Express a post-RoPE query in its query-local frame.
    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    );

    /// Transport one post-RoPE cached key into the query-local frame.
    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    );

    /// Transport one cached value into the query-local frame.
    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    );

    /// Score and normalize the complete causal prefix in the query frame.
    ///
    /// `packed_keys` contains one consecutive `query.len()`-wide row per
    /// source position, in causal order, and `output_weights` contains one
    /// slot per row. The default is the current ordinary scaled-dot-product
    /// plus stable softmax with the packed row stride.
    fn score_and_normalize(
        &mut self,
        _context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        canonical_math: bool,
    ) {
        standard_head_attention_weights(
            output_weights,
            query,
            packed_keys,
            0,
            query.len(),
            canonical_math,
        );
    }

    /// Aggregate or geometrically center the weighted query-frame values.
    ///
    /// `packed_values` contains one consecutive `output.len()`-wide row per
    /// causal weight. The default is the current correctly-rounded linear
    /// weighted-value aggregate with the packed row stride.
    fn weighted_value_centroid(
        &mut self,
        _context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) {
        let output_width = output.len();
        head_attention_value_aggregate(output, weights, packed_values, 0, output_width);
    }

    /// Return a query-frame value aggregate to the model frame before Wo.
    fn output_to_model_frame(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    );
}

/// Decoder-owned evidence that the injected operator was presented every
/// vector in the complete causal prefix. This ledger proves decoder-side
/// support and causal ordering; an implementation that replaces the default
/// row hooks must separately attest which presented sources it actually
/// scored and aggregated. Counts saturate instead of wrapping during unusually
/// long probes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CausalAttentionTransportAudit {
    /// Successfully started causal positions.
    pub positions: u64,
    /// Selected decoder-layer invocations.
    pub layers: u64,
    /// Query-head invocations across selected layers.
    pub heads: u64,
    /// Query vectors passed through `transform_query`.
    pub query_transforms: u64,
    /// Full-width cached keys passed through `transport_key`.
    pub key_transports: u64,
    /// Full-width cached values passed through `transport_value`.
    pub value_transports: u64,
    /// Head aggregates passed through `output_to_model_frame`.
    pub output_transforms: u64,
    /// Calls whose source position exceeded the current query position. A
    /// conforming decoder run keeps this exactly zero.
    pub future_reads: u64,
    /// Largest query position observed, or `None` before the first step.
    pub maximum_query_position: Option<usize>,
    /// Largest source position presented, or `None` before the first head.
    pub maximum_source_position: Option<usize>,
}

/// Focused failures at the full-prefix transport-session boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalAttentionTransportError {
    /// The source model's query-head layout is internally inconsistent.
    InvalidHeadLayout { dimension: usize, heads: usize },
    /// The grouped-query KV-head layout is internally inconsistent.
    InvalidGroupedQueryLayout { query_heads: usize, kv_heads: usize },
    /// An R4 transport requires complete four-lane blocks.
    HeadSizeNotDivisibleByFour { head_size: usize },
    /// A selected-layer request named no layer.
    EmptyLayerSelection,
    /// A selected layer does not exist in the source decoder.
    LayerOutOfRange { requested: usize, layers: usize },
    /// A selected layer was named more than once.
    DuplicateLayer(usize),
    /// The bounded source-state allocation was refused.
    SequenceCapacity(String),
    /// Scratch-size arithmetic overflowed before allocation.
    ArithmeticOverflow,
    /// A session constructed for another source checkpoint was supplied.
    SourceBindingMismatch,
    /// The injected transport reported an internal fault. The decoder state
    /// may have advanced, but no logits are returned; reset the session before
    /// attempting another sequence.
    TransportFault {
        policy_identity: String,
        reason: String,
    },
    /// The token is outside the source vocabulary.
    TokenOutOfRange(usize),
    /// The requested position exceeds this session's bounded state.
    PositionOutOfRange { position: usize, capacity: usize },
    /// Causal transport positions must be advanced in exact sequence order.
    PositionOutOfOrder { requested: usize, expected: usize },
    /// The caller-provided logit buffer does not match the source vocabulary.
    LogitShape { requested: usize, expected: usize },
}

impl std::fmt::Display for CausalAttentionTransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "causal attention transport unavailable: {self:?}"
        )
    }
}

impl std::error::Error for CausalAttentionTransportError {}

/// Stable arithmetic-era token shared by the three current source-attention
/// records. It names the correctly-rounded f32 result contract and its two
/// execution owners without freezing the particular conservative error-bound
/// formula used to prove a native lane.
pub const CERTIFIED_NATIVE_ARITHMETIC_ID: &str = "correctly-rounded-binary32-exact-real-dot-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback";

/// Declared parameters of an attention operator — head selection, score
/// scale, score-width policy (which head dimensions the compatibility
/// relation reads), remainder policy for non-divisible head widths, and
/// the score accumulation order. These strings are stable machine tokens
/// (they enter the canonical digest serialization byte-for-byte),
/// documented on [`AttentionOperatorParams::standard`] and
/// [`AttentionOperatorParams::experimental_r4`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionOperatorParams {
    /// How a query head selects its key/value head (grouped query).
    #[serde(default)]
    pub head_selection: String,
    /// The scale applied to every raw score.
    #[serde(default)]
    pub score_scale: String,
    /// Which head dimensions the compatibility relation reads.
    #[serde(default)]
    pub score_width_policy: String,
    /// What happens to head dimensions outside the scored width.
    #[serde(default)]
    pub remainder_policy: String,
    /// The arithmetic and rounding discipline of one score accumulation.
    #[serde(default)]
    pub score_accumulation: String,
}

impl AttentionOperatorParams {
    /// The declared parameters of current `standard-source-attention/2`.
    pub fn standard() -> Self {
        Self::standard_v2()
    }

    /// Immutable parameters of historical `standard-source-attention/1`:
    ///
    /// - `head_selection = "grouped-query-kv-head-h-div-kv-mul"` — head
    ///   `h` reads kv head `h / kv_mul`, `kv_mul = n_heads / n_kv_heads`.
    /// - `score_scale = "divide-by-sqrt-full-head-size"` — one divide by
    ///   `sqrt(H)` per score, `H` the full head width.
    /// - `score_width_policy = "full-head-width"` — every head dimension
    ///   enters the dot product.
    /// - `remainder_policy = "none-every-head-dimension-scored"` — there
    ///   is no unscored remainder at any head width.
    /// - `score_accumulation = "sequential-f32-left-fold"` — one running
    ///   f32 sum over `i = 0..H` in order.
    pub fn standard_v1() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: "sequential-f32-left-fold".to_owned(),
        }
    }

    /// Immutable parameters of current `standard-source-attention/2`.
    pub fn standard_v2() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: CERTIFIED_NATIVE_ARITHMETIC_ID.to_owned(),
        }
    }

    /// The declared parameters of current
    /// `experimental-r4-source-attention/2`.
    pub fn experimental_r4() -> Self {
        Self::experimental_r4_v2()
    }

    /// Immutable parameters of historical
    /// `experimental-r4-source-attention/1`
    /// — the ACTUAL computation of the `r4_attention` branch:
    ///
    /// - `head_selection = "grouped-query-kv-head-h-div-kv-mul"` — same
    ///   as standard.
    /// - `score_scale = "divide-by-sqrt-full-head-size"` — the divide
    ///   uses the FULL head width `H` even though only `4·⌊H/4⌋`
    ///   dimensions are scored.
    /// - `score_width_policy = "chunks-of-4-floor-head-size-div-4"` —
    ///   the dot product reads dimensions `0..4·⌊H/4⌋` in 4-wide chunks.
    /// - `remainder_policy = "truncate-trailing-head-size-mod-4-dims-from-score"`
    ///   — the trailing `H mod 4` q/k dimensions never enter any score
    ///   (value aggregation still uses all `H` dimensions); `H < 4`
    ///   scores 0 everywhere, so the softmax is uniform.
    /// - `score_accumulation = "per-4-chunk-left-fold-then-chunk-sum"` —
    ///   each chunk is a left-to-right 4-term f32 fold; chunk subtotals
    ///   are then summed in chunk order.
    pub fn experimental_r4_v1() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "chunks-of-4-floor-head-size-div-4".to_owned(),
            remainder_policy: "truncate-trailing-head-size-mod-4-dims-from-score".to_owned(),
            score_accumulation: "per-4-chunk-left-fold-then-chunk-sum".to_owned(),
        }
    }

    /// Immutable parameters of current
    /// `experimental-r4-source-attention/2`.
    pub fn experimental_r4_v2() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "chunks-of-4-floor-head-size-div-4".to_owned(),
            remainder_policy: "truncate-trailing-head-size-mod-4-dims-from-score".to_owned(),
            score_accumulation: CERTIFIED_NATIVE_ARITHMETIC_ID.to_owned(),
        }
    }

    /// The declared parameters of `r4-route-attention/1` (#604) — the
    /// target route operator's analogues of the source fields:
    ///
    /// - `head_selection = "single-route-lane"` — one route lane per
    ///   operator instance; no grouped-query head arithmetic exists.
    /// - `score_scale = "none-integer-popcount-distance"` — distances
    ///   are raw masked popcounts; nothing divides or rescales them.
    /// - `score_width_policy = "declared-288-bit-mask-over-route-code"`
    ///   — the relation reads exactly the declared mask's bits of the
    ///   288-bit route code.
    /// - `remainder_policy = "unmasked-bits-never-scored"` — bits
    ///   outside the mask never enter any distance (pinned by the mask
    ///   property test).
    /// - `score_accumulation = "per-byte-popcount-table-add-left-fold"`
    ///   — one popcount-table read and one integer add per byte, in
    ///   byte order.
    pub fn r4_route_attention() -> Self {
        Self {
            head_selection: "single-route-lane".to_owned(),
            score_scale: "none-integer-popcount-distance".to_owned(),
            score_width_policy: "declared-288-bit-mask-over-route-code".to_owned(),
            remainder_policy: "unmasked-bits-never-scored".to_owned(),
            score_accumulation: "per-byte-popcount-table-add-left-fold".to_owned(),
        }
    }

    /// The declared parameters of `msa-structured-selector/1` (#643): no
    /// query/key score is computed at all — candidates rank by a fixed
    /// modular classification of their own declared id, independent of
    /// any query.
    pub fn msa_structured_selector() -> Self {
        Self {
            head_selection: "single-selector-lane".to_owned(),
            score_scale: "none-ordinal-classification-key-not-a-scalar-score".to_owned(),
            score_width_policy: "candidate-id-residue-mod-11-only".to_owned(),
            remainder_policy: "none-no-remainder-every-candidate-id-classified".to_owned(),
            score_accumulation: "role-rank-then-cascade-position-then-candidate-id-sort-key"
                .to_owned(),
        }
    }

    /// The declared parameters of current
    /// `learned-absolute-source-attention/2`.
    pub fn learned_absolute() -> Self {
        Self::learned_absolute_v2()
    }

    /// Immutable parameters of historical
    /// `learned-absolute-source-attention/1`
    /// (#668) — the ACTUAL computation of the GPT-2 executor
    /// (`crate::gpt2::Gpt2Model::layer_forward`):
    ///
    /// - `head_selection = "multi-head-identity-kv-head-equals-query-head"`
    ///   — GPT-2 is plain multi-head (kv heads == query heads); head `h`
    ///   reads its own `h`-th head slice, with no grouped-query sharing.
    /// - `score_scale = "multiply-by-reciprocal-sqrt-full-head-size"` —
    ///   the executor precomputes `scale = 1/sqrt(head_size)` once and
    ///   multiplies every score by it, distinct from the standard
    ///   operator's per-score divide by `sqrt(head_size)`.
    /// - `score_width_policy = "full-head-width"` — every head dimension
    ///   enters the dot product.
    /// - `remainder_policy = "none-every-head-dimension-scored"` — there
    ///   is no unscored remainder at any head width.
    /// - `score_accumulation = "sequential-f32-left-fold"` — one running
    ///   f32 sum over `i = 0..head_size` in order.
    pub fn learned_absolute_v1() -> Self {
        Self {
            head_selection: "multi-head-identity-kv-head-equals-query-head".to_owned(),
            score_scale: "multiply-by-reciprocal-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: "sequential-f32-left-fold".to_owned(),
        }
    }

    /// Immutable parameters of current
    /// `learned-absolute-source-attention/2`.
    pub fn learned_absolute_v2() -> Self {
        Self {
            head_selection: "multi-head-identity-kv-head-equals-query-head".to_owned(),
            score_scale: "multiply-by-reciprocal-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: CERTIFIED_NATIVE_ARITHMETIC_ID.to_owned(),
        }
    }
}

/// The typed, versioned record of one source attention operator (#602).
/// Serialized (all fields serde-defaulted) into the observation manifest
/// and the cover/compile report wherever the operator is known, so an
/// operator change is visible in provenance instead of silent. `None`
/// wherever the record is carried means the legacy interpretation
/// documented in `docs/MODEL_LIFECYCLE.md` (produced before #602; for
/// teacher paths the default-off switch always computed
/// `standard-source-attention/1`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionOperatorSpec {
    /// Registry id of the operator (e.g. `"standard-source-attention"`).
    #[serde(default)]
    pub id: String,
    /// Registry version of the operator. A behavioral change is a new
    /// version, never an in-place edit.
    #[serde(default)]
    pub version: u32,
    /// The projections producing q/k/v from the normalized residual.
    #[serde(default)]
    pub projections: String,
    /// The positional action applied before scoring.
    #[serde(default)]
    pub positional_action: String,
    /// The compatibility relation between a query and a cached key.
    #[serde(default)]
    pub compatibility_relation: String,
    /// The selector/normalization turning raw scores into weights —
    /// exactly what `softmax_with_mode` computes, per mode.
    #[serde(default)]
    pub selector_normalization: String,
    /// How weighted values are aggregated into the head output.
    #[serde(default)]
    pub value_aggregation: String,
    /// The projection producing the layer output from head outputs.
    #[serde(default)]
    pub output_projection: String,
    /// The recurrent state the operator reads and extends.
    #[serde(default)]
    pub runtime_state: String,
    /// Deterministic tie-breaking discipline.
    #[serde(default)]
    pub tie_breaking: String,
    /// The operation class the operator is permitted to use — host-side
    /// source f32; NOT the deployed inference contract's integer set.
    #[serde(default)]
    pub permitted_operation_class: String,
    /// The operator's declared parameters (head selection, scale, score
    /// width, remainder policy, accumulation order).
    #[serde(default)]
    pub params: AttentionOperatorParams,
    /// `blake3:<hex>` of [`AttentionOperatorSpec::canonical_bytes`] —
    /// the declared identity, not source code text (see the module docs
    /// for why).
    #[serde(default)]
    pub implementation_digest: String,
}

impl AttentionOperatorSpec {
    /// Registry id of the standard scaled-dot-product source operator.
    pub const STANDARD_ID: &'static str = "standard-source-attention";
    /// Immutable historical version using sequential f32 Q·K/value folds.
    pub const STANDARD_V1_VERSION: u32 = 1;
    /// Current certified-native Q·K/value registry version implemented
    /// by [`standard_head_attention_weights`] +
    /// [`head_attention_value_aggregate`].
    pub const STANDARD_V2_VERSION: u32 = 2;
    /// Current standard registry version.
    pub const STANDARD_VERSION: u32 = Self::STANDARD_V2_VERSION;
    /// Registry id of the experimental `r4_attention`-gated operator,
    /// named for what it computes (a chunked dot product with the same
    /// softmax selector), not for the historical description.
    pub const EXPERIMENTAL_R4_ID: &'static str = "experimental-r4-source-attention";
    /// Immutable historical version using chunked f32 Q·K/value folds.
    pub const EXPERIMENTAL_R4_V1_VERSION: u32 = 1;
    /// Current certified-native Q·K/value registry version
    /// implemented by [`experimental_r4_head_attention_weights`] +
    /// [`head_attention_value_aggregate`].
    pub const EXPERIMENTAL_R4_V2_VERSION: u32 = 2;
    /// Current experimental registry version.
    pub const EXPERIMENTAL_R4_VERSION: u32 = Self::EXPERIMENTAL_R4_V2_VERSION;
    /// Registry id of the `R4RouteAttentionV1` TARGET operator (#604) —
    /// the same id string as
    /// `uor-r4-graph-format::route_attention::ROUTE_ATTENTION_OPERATOR_ID`;
    /// the two constants are asserted equal by the #604 test suite in
    /// `uor-r4-graph-certify` (which depends on both crates).
    pub const R4_ROUTE_ID: &'static str = "r4-route-attention";
    /// Registry version of the target operator implemented by
    /// `uor-r4-graph-certify::route_attention` (reference) and
    /// `uor-r4-graph-runtime::route_attention` (packed lowering).
    pub const R4_ROUTE_VERSION: u32 = 1;
    /// Registry id of the GPT-2-family source operator (#668): a scaled
    /// dot product with the SAME max-subtracted softmax as the standard
    /// operator, but learned absolute positions (no RoPE) and GPT-2's
    /// fused-`c_attn`/`c_proj` Conv1D projections. Named for its
    /// positional action, the field that distinguishes it from
    /// `standard-source-attention`.
    pub const LEARNED_ABSOLUTE_ID: &'static str = "learned-absolute-source-attention";
    /// Immutable historical version using sequential f32 Q·K/value folds.
    pub const LEARNED_ABSOLUTE_V1_VERSION: u32 = 1;
    /// Current certified-native Q·K/value registry version
    /// computed by the GPT-2 executor (`crate::gpt2::Gpt2Model`).
    pub const LEARNED_ABSOLUTE_V2_VERSION: u32 = 2;
    /// Current learned-absolute registry version.
    pub const LEARNED_ABSOLUTE_VERSION: u32 = Self::LEARNED_ABSOLUTE_V2_VERSION;
    /// Registry id of the `MsaStructuredSelectorV1` TARGET operator
    /// (#643) — a second, independent target operator alongside
    /// `r4-route-attention/1`, pre-registered for an A/B evaluation
    /// under the same #626-style exit-rule convention. Reference
    /// semantics live in `uor-r4-graph-certify::msa_selector`, the
    /// same layering `r4-route-attention/1` uses.
    pub const MSA_STRUCTURED_SELECTOR_ID: &'static str = "msa-structured-selector";
    /// Registry version of the target operator implemented by
    /// `uor-r4-graph-certify::msa_selector` (reference only in this
    /// slice; no packed lowering exists yet).
    pub const MSA_STRUCTURED_SELECTOR_VERSION: u32 = 1;

    /// The current `standard-source-attention/2` record.
    pub fn standard() -> Self {
        Self::standard_v2()
    }

    /// Immutable historical `standard-source-attention/1` registry entry.
    pub fn standard_v1() -> Self {
        let mut record = Self {
            id: Self::STANDARD_ID.to_owned(),
            version: Self::STANDARD_V1_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation: "scaled-dot-product".to_owned(),
            selector_normalization: "softmax-max-subtracted-exp-then-sum-normalize".to_owned(),
            value_aggregation: "position-ascending-weighted-sum-of-values".to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            params: AttentionOperatorParams::standard_v1(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Immutable current `standard-source-attention/2` registry entry.
    pub fn standard_v2() -> Self {
        let mut record = Self {
            id: Self::STANDARD_ID.to_owned(),
            version: Self::STANDARD_V2_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation:
                "correctly-rounded-binary32-exact-real-full-width-dot".to_owned(),
            selector_normalization:
                "divide-by-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-per-weight-divide"
                    .to_owned(),
            value_aggregation:
                "correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback"
                    .to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class:
                "host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback".to_owned(),
            params: AttentionOperatorParams::standard_v2(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// The current `experimental-r4-source-attention/2` record.
    pub fn experimental_r4() -> Self {
        Self::experimental_r4_v2()
    }

    /// Immutable historical `experimental-r4-source-attention/1` record — the ACTUAL
    /// computation of the `r4_attention = true` branch: a 4-wide-chunked
    /// dot product (truncating the trailing `head_size mod 4`
    /// dimensions from the score) followed by the SAME max-subtracted
    /// softmax the standard operator uses.
    pub fn experimental_r4_v1() -> Self {
        let mut record = Self {
            id: Self::EXPERIMENTAL_R4_ID.to_owned(),
            version: Self::EXPERIMENTAL_R4_V1_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation: "chunked-4-wide-dot-product".to_owned(),
            selector_normalization: "softmax-max-subtracted-exp-then-sum-normalize".to_owned(),
            value_aggregation: "position-ascending-weighted-sum-of-values".to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            params: AttentionOperatorParams::experimental_r4_v1(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Immutable current `experimental-r4-source-attention/2` entry. Its
    /// historical truncated score domain and full-head-size divide remain.
    pub fn experimental_r4_v2() -> Self {
        let mut record = Self {
            id: Self::EXPERIMENTAL_R4_ID.to_owned(),
            version: Self::EXPERIMENTAL_R4_V2_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation:
                "correctly-rounded-binary32-exact-real-floor-multiple-of-4-width-dot"
                    .to_owned(),
            selector_normalization:
                "divide-by-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-per-weight-divide"
                    .to_owned(),
            value_aggregation:
                "correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback"
                    .to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class:
                "host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback".to_owned(),
            params: AttentionOperatorParams::experimental_r4_v2(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// The `r4-route-attention/1` record (#604): `R4RouteAttentionV1`,
    /// the first TARGET operator — deployed integer class, dormant.
    /// Truthful inventory of what the implementations compute
    /// (reference: `uor-r4-graph-certify::route_attention`; packed:
    /// `uor-r4-graph-runtime::route_attention`; both differentially
    /// tested bit-for-bit with witness replay):
    ///
    /// - no projections and no positional action: route codes, the
    ///   relation mask, and per-candidate ScoreQ contributions are
    ///   DECLARED tables over the 288-bit signature substrate — no
    ///   Q/K/V weight is reused under the route equation;
    /// - compatibility relation `masked-xor-popcount`: per byte,
    ///   `popcount((query XOR candidate) AND mask)`, table popcount,
    ///   integer adds;
    /// - selector: NONE — no softmax, no normalization; a bounded
    ///   top-M selection (M declared, `1..=min(8, N)`, `N <= 64`) by
    ///   ascending `(distance, index)`;
    /// - tie-breaking: lowest candidate index on equal masked popcount
    ///   distance, deterministic by construction;
    /// - value aggregation: the selected ScoreQ contributions fold in
    ///   selection order with saturating integer adds;
    /// - runtime state: caller-owned fixed-capacity selection state
    ///   (epoch-stamped), allocation-free in steady state.
    pub fn r4_route_attention_v1() -> Self {
        let mut record = Self {
            id: Self::R4_ROUTE_ID.to_owned(),
            version: Self::R4_ROUTE_VERSION,
            projections: "none-declared-route-code-tables-no-qkv-reuse".to_owned(),
            positional_action: "none-route-codes-carry-no-positional-action".to_owned(),
            compatibility_relation: "masked-xor-popcount".to_owned(),
            selector_normalization: "none-bounded-top-m-selection".to_owned(),
            value_aggregation: "selection-order-saturating-scoreq-add".to_owned(),
            output_projection: "none-aggregate-scoreq-only".to_owned(),
            runtime_state: "caller-owned-fixed-capacity-epoch-stamped-selection-state".to_owned(),
            tie_breaking: "lowest-candidate-index-on-equal-masked-popcount-distance".to_owned(),
            permitted_operation_class: "deployed-integer-xor-popcount-add-compare-table-read"
                .to_owned(),
            params: AttentionOperatorParams::r4_route_attention(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// The `msa-structured-selector/1` record (#643): `MsaStructuredSelectorV1`,
    /// a second TARGET operator alongside `r4-route-attention/1` —
    /// deployed integer class, dormant, pre-registered for an A/B
    /// evaluation against it. Grounded in two proven theorems of the
    /// (now-published, per Casey's 2026-08-16 direction) "Modular
    /// Structural Arithmetic" paper: the 11-Theorem (`DP(11)` with role
    /// anchors `mod_11(γ)=2, mod_11(μ)=4, mod_11(ε)=8`) and the maximal
    /// doubling-cascade period in ℤ/11ℤ. Truthful inventory of what
    /// `uor-r4-graph-certify::msa_selector` computes:
    ///
    /// - no projections and no positional action: a candidate's
    ///   classification depends only on its own declared id, never on a
    ///   query or a learned weight;
    /// - compatibility relation `modular-role-then-cascade-position`:
    ///   `residue = candidate_id mod 11`; `role_rank` is the residue's
    ///   position in the doubling-cascade orbit `(2,4,8,5,10,9,7,3,6,1)`
    ///   taken mod 3 (residue 0 — outside the multiplicative group — is
    ///   its own fourth "zero" class). **Only the three anchor residues
    ///   {2,4,8} → {Gen,Med,Man} are a theorem of the paper (the
    ///   11-Theorem); the mod-3 extension covering the other 7 nonzero
    ///   residues is this operator's own design choice, confirmed by
    ///   Casey (2026-08-16), not itself a proven MSA result**;
    /// - selector: NONE — no softmax; a bounded top-M selection by
    ///   ascending `(role_rank, cascade_position, candidate_index)`;
    /// - tie-breaking: lowest candidate index (into the declared id/
    ///   contribution tables) on equal role and cascade position,
    ///   deterministic by construction, same shape as
    ///   `r4-route-attention/1`;
    /// - value aggregation: the selected ScoreQ contributions fold in
    ///   selection order with saturating integer adds — identical
    ///   convention to `r4-route-attention/1`, so the two operators are
    ///   plug-compatible for the pre-registered A/B;
    /// - runtime state: none — the classification is query-independent,
    ///   so nothing carries across steps.
    pub fn msa_structured_selector_v1() -> Self {
        let mut record = Self {
            id: Self::MSA_STRUCTURED_SELECTOR_ID.to_owned(),
            version: Self::MSA_STRUCTURED_SELECTOR_VERSION,
            projections: "none-declared-candidate-id-table-no-qkv-reuse".to_owned(),
            positional_action: "none-classification-carries-no-positional-action".to_owned(),
            compatibility_relation: "modular-role-then-cascade-position".to_owned(),
            selector_normalization: "none-bounded-top-m-selection".to_owned(),
            value_aggregation: "selection-order-saturating-scoreq-add".to_owned(),
            output_projection: "none-aggregate-scoreq-only".to_owned(),
            runtime_state: "none-query-independent-classification".to_owned(),
            tie_breaking: "lowest-candidate-index-on-equal-role-and-cascade-position".to_owned(),
            permitted_operation_class: "deployed-integer-table-read-compare-add-no-runtime-modulo"
                .to_owned(),
            params: AttentionOperatorParams::msa_structured_selector(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// The current `learned-absolute-source-attention/2` record (#668).
    pub fn learned_absolute_source_attention() -> Self {
        Self::learned_absolute_v2()
    }

    /// Immutable historical `learned-absolute-source-attention/1` record — the
    /// GPT-2-family source operator computed by the executor in
    /// [`crate::gpt2`] (`Gpt2Model::layer_forward`), the way
    /// `r4-route-attention/1`'s record names an operator whose
    /// implementation lives outside this module. Truthful inventory of
    /// what the GPT-2 executor computes, read from `layer_forward` (the
    /// presence-gated numpy canary and the tiny-fixture parity test pin
    /// the arithmetic):
    ///
    /// - `projections`: the fused `c_attn` Conv1D `[n_embd, 3·n_embd]`
    ///   applied as `x @ W + b` (no transpose), split into q/k/v thirds,
    ///   WITH bias — unlike Llama's separate biasless `wq`/`wk`/`wv`.
    /// - `positional_action`: NONE on q/k before scoring. GPT-2 adds a
    ///   learned absolute position embedding (`wpe`) to the input
    ///   residual; no rotation touches q or k, unlike RoPE. This is the
    ///   field that makes reusing `standard-source-attention/1` a false
    ///   record.
    /// - `compatibility_relation`: `scaled-dot-product`, a single
    ///   sequential f32 left fold over the FULL head width, scaled by a
    ///   precomputed `1/sqrt(head_size)` (multiply, not divide).
    /// - `selector_normalization`: the SAME max-subtracted softmax the
    ///   standard operator uses.
    /// - `value_aggregation`: position-ascending weighted sum of values.
    /// - `output_projection`: the `c_proj` Conv1D `[n_embd, n_embd]` with
    ///   bias.
    /// - head selection is plain multi-head (kv heads == query heads);
    ///   no grouped-query arithmetic exists.
    pub fn learned_absolute_v1() -> Self {
        let mut record = Self {
            id: Self::LEARNED_ABSOLUTE_ID.to_owned(),
            version: Self::LEARNED_ABSOLUTE_V1_VERSION,
            projections: "fused-c-attn-conv1d-qkv-with-bias".to_owned(),
            positional_action: "none-learned-absolute-positions-added-to-input-embeddings"
                .to_owned(),
            compatibility_relation: "scaled-dot-product".to_owned(),
            selector_normalization: "softmax-max-subtracted-exp-then-sum-normalize".to_owned(),
            value_aggregation: "position-ascending-weighted-sum-of-values".to_owned(),
            output_projection: "dense-c-proj-conv1d-with-bias".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            params: AttentionOperatorParams::learned_absolute_v1(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Immutable current `learned-absolute-source-attention/2` entry. Its
    /// projection fields declare GPT-2 topology; when a dense record is
    /// present, that separate record owns projection arithmetic. Q·K/value use
    /// certified-native cells with the pinned exact fallback, and normalization
    /// retains reciprocal multiplication rather than Llama's divisions.
    pub fn learned_absolute_v2() -> Self {
        let mut record = Self {
            id: Self::LEARNED_ABSOLUTE_ID.to_owned(),
            version: Self::LEARNED_ABSOLUTE_V2_VERSION,
            projections: "fused-c-attn-conv1d-qkv-with-bias".to_owned(),
            positional_action: "none-learned-absolute-positions-added-to-input-embeddings"
                .to_owned(),
            compatibility_relation:
                "correctly-rounded-binary32-exact-real-full-width-dot".to_owned(),
            selector_normalization:
                "multiply-by-reciprocal-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-reciprocal-multiply"
                    .to_owned(),
            value_aggregation:
                "correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback"
                    .to_owned(),
            output_projection: "dense-c-proj-conv1d-with-bias".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class:
                "host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback".to_owned(),
            params: AttentionOperatorParams::learned_absolute_v2(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Canonical serialization of the record's declared identity: a
    /// fixed line format (format tag, id, version, operator fields,
    /// parameters, each `key=value\n`). Byte-stable by construction —
    /// field order and separators are fixed here, not derived from any
    /// serializer — so the digest over these bytes is reproducible
    /// everywhere.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "uor-r4-attention-operator/1\n\
             id={}\n\
             version={}\n\
             projections={}\n\
             positional_action={}\n\
             compatibility_relation={}\n\
             selector_normalization={}\n\
             value_aggregation={}\n\
             output_projection={}\n\
             runtime_state={}\n\
             tie_breaking={}\n\
             permitted_operation_class={}\n\
             param.head_selection={}\n\
             param.score_scale={}\n\
             param.score_width_policy={}\n\
             param.remainder_policy={}\n\
             param.score_accumulation={}\n",
            self.id,
            self.version,
            self.projections,
            self.positional_action,
            self.compatibility_relation,
            self.selector_normalization,
            self.value_aggregation,
            self.output_projection,
            self.runtime_state,
            self.tie_breaking,
            self.permitted_operation_class,
            self.params.head_selection,
            self.params.score_scale,
            self.params.score_width_policy,
            self.params.remainder_policy,
            self.params.score_accumulation,
        )
        .into_bytes()
    }

    /// The implementation digest this record's declared identity
    /// implies: `blake3:<hex>` over
    /// [`AttentionOperatorSpec::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// The typed record the boolean `Config::r4_attention` switch selects:
/// `false` selects current `standard-source-attention/2`, `true` current
/// `experimental-r4-source-attention/2`. This is the one boundary mapping
/// from the legacy switch to the versioned operator identity; explicit v1
/// registry entries remain readable but are never selected as current.
pub fn operator_for_r4_switch(r4_attention: bool) -> AttentionOperatorSpec {
    if r4_attention {
        AttentionOperatorSpec::experimental_r4()
    } else {
        AttentionOperatorSpec::standard()
    }
}

/// The versioned operator registry (#602; #604 adds the first target
/// entry): map `(id, version)` to the spec that names it. Every pair
/// outside the registry is refused by name on the sanctioned
/// [`SourceUnavailable`] surface
/// ([`SourceIngestKind::UnknownAttentionOperator`]) — never guessed,
/// never approximated by a "closest" operator or version.
///
/// [`SourceUnavailable`]: crate::SourceUnavailable
/// [`SourceIngestKind::UnknownAttentionOperator`]: crate::SourceIngestKind::UnknownAttentionOperator
#[cfg(not(target_arch = "wasm32"))]
pub fn operator_spec(
    id: &str,
    version: u32,
) -> Result<AttentionOperatorSpec, crate::SourceUnavailable> {
    match (id, version) {
        (AttentionOperatorSpec::STANDARD_ID, AttentionOperatorSpec::STANDARD_V1_VERSION) => {
            Ok(AttentionOperatorSpec::standard_v1())
        }
        (AttentionOperatorSpec::STANDARD_ID, AttentionOperatorSpec::STANDARD_V2_VERSION) => {
            Ok(AttentionOperatorSpec::standard())
        }
        (
            AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
            AttentionOperatorSpec::EXPERIMENTAL_R4_V1_VERSION,
        ) => Ok(AttentionOperatorSpec::experimental_r4_v1()),
        (
            AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
            AttentionOperatorSpec::EXPERIMENTAL_R4_V2_VERSION,
        ) => Ok(AttentionOperatorSpec::experimental_r4()),
        (AttentionOperatorSpec::R4_ROUTE_ID, AttentionOperatorSpec::R4_ROUTE_VERSION) => {
            Ok(AttentionOperatorSpec::r4_route_attention_v1())
        }
        (
            AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_ID,
            AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_VERSION,
        ) => Ok(AttentionOperatorSpec::msa_structured_selector_v1()),
        (
            AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_V1_VERSION,
        ) => Ok(AttentionOperatorSpec::learned_absolute_v1()),
        (
            AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_V2_VERSION,
        ) => Ok(AttentionOperatorSpec::learned_absolute_source_attention()),
        _ => Err(crate::SourceIngestKind::UnknownAttentionOperator {
            id: id.to_owned(),
            version,
        }
        .into()),
    }
}

/// Arithmetic owner for source-attention execution and its controls.
///
/// Current production entry points select [`Self::CertifiedNative`]. The
/// historical conventional fold and always-exact owner remain explicit for
/// differential tests and matched measurement in one binary.
#[doc(hidden)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AttentionArithmetic {
    #[default]
    Conventional,
    Exact,
    CertifiedNative,
}

/// Per-dot verdict census for the #704 attention controls.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AttentionDotCensus {
    pub(crate) lanes: usize,
    pub(crate) conventional: usize,
    pub(crate) exact_control: usize,
    pub(crate) certified: usize,
    pub(crate) fallback_nonfinite: usize,
    pub(crate) fallback_zero: usize,
    pub(crate) fallback_overflow: usize,
    pub(crate) fallback_cell: usize,
}

impl AttentionDotCensus {
    pub(crate) const fn fallbacks(self) -> usize {
        self.fallback_nonfinite + self.fallback_zero + self.fallback_overflow + self.fallback_cell
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn merge(&mut self, other: Self) {
        self.lanes += other.lanes;
        self.conventional += other.conventional;
        self.exact_control += other.exact_control;
        self.certified += other.certified;
        self.fallback_nonfinite += other.fallback_nonfinite;
        self.fallback_zero += other.fallback_zero;
        self.fallback_overflow += other.fallback_overflow;
        self.fallback_cell += other.fallback_cell;
    }
}

/// Q·K and weighted-value verdicts accumulated across an attention run.
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AttentionArithmeticCensus {
    pub(crate) qk: AttentionDotCensus,
    pub(crate) value: AttentionDotCensus,
}

#[cfg(not(target_arch = "wasm32"))]
impl AttentionArithmeticCensus {
    pub(crate) fn merge(&mut self, other: Self) {
        self.qk.merge(other.qk);
        self.value.merge(other.value);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CertificationRejection {
    Nonfinite,
    Zero,
    Overflow,
    Cell,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CertificationVerdict {
    Certified(f32),
    Rejected(CertificationRejection),
}

impl AttentionDotCensus {
    fn zero_dot(lanes: usize, arithmetic: AttentionArithmetic) -> Self {
        match arithmetic {
            AttentionArithmetic::Conventional => Self {
                lanes,
                conventional: lanes,
                ..Self::default()
            },
            AttentionArithmetic::Exact => Self {
                lanes,
                exact_control: lanes,
                ..Self::default()
            },
            AttentionArithmetic::CertifiedNative => Self {
                lanes,
                fallback_zero: lanes,
                ..Self::default()
            },
        }
    }

    fn reject(&mut self, reason: CertificationRejection) {
        match reason {
            CertificationRejection::Nonfinite => self.fallback_nonfinite += 1,
            CertificationRejection::Zero => self.fallback_zero += 1,
            CertificationRejection::Overflow => self.fallback_overflow += 1,
            CertificationRejection::Cell => self.fallback_cell += 1,
        }
    }
}

#[inline]
fn next_up_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits(1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value > 0.0 { bits + 1 } else { bits - 1 })
}

#[inline]
fn next_down_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits((1u64 << 63) | 1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value > 0.0 { bits - 1 } else { bits + 1 })
}

#[inline]
fn next_up_f32(value: f32) -> f32 {
    if value.is_nan() || value == f32::INFINITY {
        return value;
    }
    if value == 0.0 {
        return f32::from_bits(1);
    }
    let bits = value.to_bits();
    f32::from_bits(if value > 0.0 { bits + 1 } else { bits - 1 })
}

#[inline]
fn next_down_f32(value: f32) -> f32 {
    if value.is_nan() || value == f32::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f32::from_bits((1u32 << 31) | 1);
    }
    let bits = value.to_bits();
    f32::from_bits(if value > 0.0 { bits - 1 } else { bits + 1 })
}

/// Certify the correctly-rounded binary32 value of one exact real dot.
///
/// Every finite binary32 product is exact in binary64: it has at most 48
/// significand bits and its exponent remains within the binary64 range. Only
/// the sequential binary64 additions round. For `u = 2^-53`, their error is
/// bounded by `gamma_k * sum(abs(products))`, here conservatively replaced by
/// `gamma_k * k * max(abs(product))`. Every bound and interval operation is
/// rounded outward. A native candidate is returned only when the resulting
/// interval is strictly inside the exact midpoints to its adjacent binary32
/// values; strictness rejects ties without needing a parity case. Nonfinite,
/// zero, overflow-adjacent, and uncertain cells use the pinned exact owner.
/// Exact products and partial sums lie on the `2^-298` grid, so nonzero
/// accumulation cannot underflow binary64; with `k <= 2^53`, even
/// `k * f32::MAX^2` remains far below binary64 overflow.
#[inline]
fn certify_dot(approximate: f64, max_product_abs: f64, k: usize) -> CertificationVerdict {
    if !approximate.is_finite() || !max_product_abs.is_finite() {
        return CertificationVerdict::Rejected(CertificationRejection::Nonfinite);
    }
    let candidate = approximate as f32;
    if candidate == 0.0 {
        return CertificationVerdict::Rejected(CertificationRejection::Zero);
    }
    if !candidate.is_finite() || candidate.abs() == f32::MAX {
        return CertificationVerdict::Rejected(CertificationRejection::Overflow);
    }
    if (k as u128) > (1u128 << f64::MANTISSA_DIGITS) {
        return CertificationVerdict::Rejected(CertificationRejection::Cell);
    }

    const UNIT_ROUNDOFF: f64 = 1.0 / ((1u64 << 53) as f64);
    let mu = next_up_f64((k as f64) * UNIT_ROUNDOFF);
    if mu >= 1.0 {
        return CertificationVerdict::Rejected(CertificationRejection::Cell);
    }
    let denominator = next_down_f64(1.0 - mu);
    let gamma = next_up_f64(mu / denominator);
    let sum_abs = next_up_f64((k as f64) * max_product_abs);
    let error = next_up_f64(gamma * sum_abs);
    if !error.is_finite() {
        return CertificationVerdict::Rejected(CertificationRejection::Cell);
    }
    let lower = next_down_f64(approximate - error);
    let upper = next_up_f64(approximate + error);

    let previous = next_down_f32(candidate);
    let next = next_up_f32(candidate);
    if !previous.is_finite() || !next.is_finite() {
        return CertificationVerdict::Rejected(CertificationRejection::Overflow);
    }
    let cell_lower = (f64::from(previous) + f64::from(candidate)) * 0.5;
    let cell_upper = (f64::from(candidate) + f64::from(next)) * 0.5;
    if lower > cell_lower && upper < cell_upper {
        CertificationVerdict::Certified(candidate)
    } else {
        CertificationVerdict::Rejected(CertificationRejection::Cell)
    }
}

fn exact_strided_matrix_vector(
    out: &mut [f32],
    matrix: &[f32],
    columns: usize,
    row_stride: usize,
    column_stride: usize,
    vector: &[f32],
) {
    if columns == 0 || out.is_empty() {
        out.fill(0.0);
        return;
    }
    let matrix = uor_matmul::MatView::new(
        matrix,
        out.len(),
        columns,
        uor_matmul::Strides {
            rs: isize::try_from(row_stride).expect("attention row stride fits isize"),
            cs: isize::try_from(column_stride).expect("attention column stride fits isize"),
        },
    )
    .expect("attention cache view is within its validated model state");
    let vector = uor_matmul::MatView::row_major(vector, columns, 1)
        .expect("attention vector shape is exact");
    let rows = out.len();
    let output =
        uor_matmul::MatViewMut::row_major(out, rows, 1).expect("attention output shape is exact");
    let mut product = uor_matmul::Triple::new(matrix, vector, output)
        .expect("attention matrix-vector product is conformant");
    uor_matmul::driver::gemm_float(
        &mut product,
        &uor_matmul::Linear::OVERWRITE,
        uor_matmul::GemmOptions::default(),
    );
}

fn exact_strided_dot(
    matrix: &[f32],
    row: usize,
    columns: usize,
    row_stride: usize,
    column_stride: usize,
    vector: &[f32],
) -> f32 {
    if columns == 0 {
        return 0.0;
    }
    let offset = row
        .checked_mul(row_stride)
        .expect("attention row offset is addressable");
    let mut value = [0.0f32];
    exact_strided_matrix_vector(
        &mut value,
        &matrix[offset..],
        columns,
        row_stride,
        column_stride,
        vector,
    );
    value[0]
}

fn controlled_strided_matrix_vector(
    out: &mut [f32],
    matrix: &[f32],
    columns: usize,
    row_stride: usize,
    column_stride: usize,
    vector: &[f32],
    arithmetic: AttentionArithmetic,
) -> AttentionDotCensus {
    assert!(
        isize::try_from(row_stride).is_ok() && isize::try_from(column_stride).is_ok(),
        "attention strides must fit the exact owner's isize layout"
    );
    let mut census = AttentionDotCensus {
        lanes: out.len(),
        ..AttentionDotCensus::default()
    };
    match arithmetic {
        AttentionArithmetic::Conventional => {
            unreachable!("the caller owns its historical conventional fold")
        }
        AttentionArithmetic::Exact => {
            exact_strided_matrix_vector(out, matrix, columns, row_stride, column_stride, vector);
            census.exact_control = out.len();
        }
        AttentionArithmetic::CertifiedNative => {
            for (row, output) in out.iter_mut().enumerate() {
                let mut sum = 0.0f64;
                let mut max_product_abs = 0.0f64;
                let mut finite = true;
                for column in 0..columns {
                    let left = matrix[row * row_stride + column * column_stride];
                    let right = vector[column];
                    if !left.is_finite() || !right.is_finite() {
                        finite = false;
                        break;
                    }
                    let product = f64::from(left) * f64::from(right);
                    sum += product;
                    max_product_abs = max_product_abs.max(product.abs());
                }
                let verdict = if finite {
                    certify_dot(sum, max_product_abs, columns)
                } else {
                    CertificationVerdict::Rejected(CertificationRejection::Nonfinite)
                };
                match verdict {
                    CertificationVerdict::Certified(value) => {
                        *output = value;
                        census.certified += 1;
                    }
                    CertificationVerdict::Rejected(reason) => {
                        census.reject(reason);
                        *output = exact_strided_dot(
                            matrix,
                            row,
                            columns,
                            row_stride,
                            column_stride,
                            vector,
                        );
                    }
                }
            }
            debug_assert_eq!(census.certified + census.fallbacks(), census.lanes);
        }
    }
    census
}

#[inline]
fn strided_span_fits(
    storage_len: usize,
    offset: usize,
    rows: usize,
    columns: usize,
    row_stride: usize,
    column_stride: usize,
) -> bool {
    if rows == 0 {
        return offset <= storage_len;
    }
    if columns == 0 {
        return (rows - 1)
            .checked_mul(row_stride)
            .and_then(|last_row| offset.checked_add(last_row))
            .is_some_and(|last_origin| last_origin <= storage_len);
    }
    (rows - 1)
        .checked_mul(row_stride)
        .and_then(|last_row| {
            (columns - 1)
                .checked_mul(column_stride)
                .and_then(|last_column| last_row.checked_add(last_column))
        })
        .and_then(|last_index| offset.checked_add(last_index))
        .and_then(|last_index| last_index.checked_add(1))
        .is_some_and(|end| end <= storage_len)
}

/// Raw Q·K control used by the GPT-2 proof seam before its distinct scalar
/// reciprocal-scale and reciprocal-sum normalization stages.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn head_dot_products_with_arithmetic(
    out: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    scored_width: usize,
    arithmetic: AttentionArithmetic,
) -> AttentionDotCensus {
    assert!(q.len() >= scored_width, "Q is shorter than scored width");
    assert!(
        isize::try_from(key_stride).is_ok(),
        "key stride must fit the exact owner's isize layout"
    );
    assert!(
        strided_span_fits(
            keys.len(),
            key_offset,
            out.len(),
            scored_width,
            key_stride,
            1,
        ),
        "key-cache layout does not contain every requested QK row"
    );
    match arithmetic {
        AttentionArithmetic::Conventional => {
            for (position, output) in out.iter_mut().enumerate() {
                let key = &keys[position * key_stride + key_offset..][..scored_width];
                let mut dot = 0.0f32;
                for i in 0..scored_width {
                    dot += q[i] * key[i];
                }
                *output = dot;
            }
            AttentionDotCensus {
                lanes: out.len(),
                conventional: out.len(),
                ..AttentionDotCensus::default()
            }
        }
        AttentionArithmetic::Exact | AttentionArithmetic::CertifiedNative => {
            controlled_strided_matrix_vector(
                out,
                &keys[key_offset..],
                scored_width,
                key_stride,
                1,
                &q[..scored_width],
                arithmetic,
            )
        }
    }
}

/// Current `standard-source-attention/2` weight computation shared by
/// `Llama::layer_forward` and `Llama::forward_batch`:
/// fill `att` (one slot per cached position `t = 0..att.len()`) with the
/// softmax-normalized scaled dot products of `q` against the cached keys.
/// `keys` is the layer's key-cache region; position `t`'s key head starts
/// at `t * key_stride + key_offset` (grouped query: `key_offset =
/// (h / kv_mul) * head_size`). The full-width exact-real dot rounds once to
/// f32 through the certified-native/exact-fallback owner, followed by Llama's
/// historical divide by `sqrt(head_size)` and shared max-subtracted softmax.
/// `canonical` selects the D2 libm math path exactly as the executor does.
pub fn standard_head_attention_weights(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
) {
    let _ = standard_head_attention_weights_with_arithmetic(
        att,
        q,
        keys,
        key_offset,
        key_stride,
        canonical,
        AttentionArithmetic::CertifiedNative,
    );
}

/// Explicit standard-attention Q·K owner for differential evidence.
/// Llama's historical per-score division and softmax order are shared by all
/// three variants and occur only after the raw dot has rounded to `f32`.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn standard_head_attention_weights_with_arithmetic(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
    arithmetic: AttentionArithmetic,
) -> AttentionDotCensus {
    let head_size = q.len();
    if att.is_empty() {
        return AttentionDotCensus::zero_dot(0, arithmetic);
    }
    if head_size == 0 {
        let census = AttentionDotCensus::zero_dot(att.len(), arithmetic);
        // The exact dot over an empty dimension is zero. Scaling by sqrt(0)
        // is undefined, so take the well-defined selector limit: equal zero
        // scores normalized by the existing softmax owner.
        att.fill(0.0);
        crate::softmax_with_mode(att, canonical);
        return census;
    }
    let mut census = head_dot_products_with_arithmetic(
        att, q, keys, key_offset, key_stride, head_size, arithmetic,
    );
    let divisor = crate::sqrtf(head_size as f32, canonical);
    for score in att.iter_mut() {
        *score /= divisor;
    }
    crate::softmax_with_mode(att, canonical);
    debug_assert_eq!(census.lanes, att.len());
    census.lanes = att.len();
    census
}

/// Current `experimental-r4-source-attention/2` weight computation: one
/// correctly-rounded exact-real dot over dimensions
/// `0..4*(head_size/4)` — the
/// trailing `head_size mod 4` q/k dimensions are never read — divided by
/// `sqrt(head_size)` over the FULL head width, then normalized by the
/// SAME max-subtracted softmax as the standard operator. For
/// `head_size < 4` every score is 0 and the weights are uniform. The operator
/// remains gated by `Config::r4_attention` at serial and batched call sites.
pub fn experimental_r4_head_attention_weights(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
) {
    let _ = experimental_r4_head_attention_weights_with_arithmetic(
        att,
        q,
        keys,
        key_offset,
        key_stride,
        canonical,
        AttentionArithmetic::CertifiedNative,
    );
}

/// Explicit experimental-attention Q·K owner for differential evidence.
/// The historical truncated score domain and full-head-size divide are
/// preserved in all modes.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn experimental_r4_head_attention_weights_with_arithmetic(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
    arithmetic: AttentionArithmetic,
) -> AttentionDotCensus {
    let head_size = q.len();
    if att.is_empty() {
        return AttentionDotCensus::zero_dot(0, arithmetic);
    }
    if head_size == 0 {
        let census = AttentionDotCensus::zero_dot(att.len(), arithmetic);
        // This is the H < 4 truncated-domain rule at H = 0: every raw score
        // is the exact empty dot, hence the selector is uniform.
        att.fill(0.0);
        crate::softmax_with_mode(att, canonical);
        return census;
    }
    let scored_width = 4 * (head_size / 4);
    if scored_width == 0 && arithmetic != AttentionArithmetic::Conventional {
        let census = AttentionDotCensus::zero_dot(att.len(), arithmetic);
        // Version 2's truncated score domain reads no q/key dimensions when
        // H < 4. Normalize the exact zero scores without imposing a cache
        // layout for bytes this operator cannot observe.
        att.fill(0.0);
        crate::softmax_with_mode(att, canonical);
        return census;
    }
    assert!(
        isize::try_from(key_stride).is_ok(),
        "key stride must fit the exact owner's isize layout"
    );
    let required_width = if arithmetic == AttentionArithmetic::Conventional {
        head_size
    } else {
        scored_width
    };
    assert!(
        strided_span_fits(
            keys.len(),
            key_offset,
            att.len(),
            required_width,
            key_stride,
            1,
        ),
        "key-cache layout does not contain every requested experimental QK row"
    );
    let census = match arithmetic {
        AttentionArithmetic::Conventional => {
            for (t, attention) in att.iter_mut().enumerate() {
                let k = &keys[t * key_stride + key_offset..][..head_size];
                let mut head_score = 0.0f32;
                let chunks = head_size / 4;
                for chunk_idx in 0..chunks {
                    let q_chunk = &q[chunk_idx * 4..(chunk_idx + 1) * 4];
                    let k_chunk = &k[chunk_idx * 4..(chunk_idx + 1) * 4];
                    head_score += q_chunk[0] * k_chunk[0]
                        + q_chunk[1] * k_chunk[1]
                        + q_chunk[2] * k_chunk[2]
                        + q_chunk[3] * k_chunk[3];
                }
                *attention = head_score;
            }
            AttentionDotCensus {
                lanes: att.len(),
                conventional: att.len(),
                ..AttentionDotCensus::default()
            }
        }
        AttentionArithmetic::Exact | AttentionArithmetic::CertifiedNative => {
            controlled_strided_matrix_vector(
                att,
                &keys[key_offset..],
                scored_width,
                key_stride,
                1,
                &q[..scored_width],
                arithmetic,
            )
        }
    };
    let divisor = crate::sqrtf(head_size as f32, canonical);
    for score in att.iter_mut() {
        *score /= divisor;
    }
    crate::softmax_with_mode(att, canonical);
    census
}

/// Current version-2 value aggregation shared by all source operators: zero
/// `out` (one head width), then compute the correctly-rounded exact-real
/// position-weighted dot for each output lane over the
/// cached values. `values` is the layer's value-cache region; position
/// `t`'s value head starts at `t * value_stride + value_offset`. Every
/// head dimension participates regardless of the scoring operator's width
/// policy; uncertified native lanes use the pinned exact fallback.
pub fn head_attention_value_aggregate(
    out: &mut [f32],
    att: &[f32],
    values: &[f32],
    value_offset: usize,
    value_stride: usize,
) {
    let _ = head_attention_value_aggregate_with_arithmetic(
        out,
        att,
        values,
        value_offset,
        value_stride,
        AttentionArithmetic::CertifiedNative,
    );
}

/// Explicit weighted-value owner for differential evidence.
#[doc(hidden)]
pub(crate) fn head_attention_value_aggregate_with_arithmetic(
    out: &mut [f32],
    att: &[f32],
    values: &[f32],
    value_offset: usize,
    value_stride: usize,
    arithmetic: AttentionArithmetic,
) -> AttentionDotCensus {
    let head_size = out.len();
    if out.is_empty() {
        return AttentionDotCensus::zero_dot(0, arithmetic);
    }
    if att.is_empty() {
        out.fill(0.0);
        return match arithmetic {
            AttentionArithmetic::Conventional => AttentionDotCensus {
                lanes: head_size,
                conventional: head_size,
                ..AttentionDotCensus::default()
            },
            AttentionArithmetic::Exact => AttentionDotCensus {
                lanes: head_size,
                exact_control: head_size,
                ..AttentionDotCensus::default()
            },
            AttentionArithmetic::CertifiedNative => AttentionDotCensus {
                lanes: head_size,
                fallback_zero: head_size,
                ..AttentionDotCensus::default()
            },
        };
    }
    assert!(
        isize::try_from(value_stride).is_ok(),
        "value stride must fit the exact owner's isize layout"
    );
    assert!(
        strided_span_fits(
            values.len(),
            value_offset,
            head_size,
            att.len(),
            1,
            value_stride,
        ),
        "value-cache layout does not contain every requested weighted-value lane"
    );
    match arithmetic {
        AttentionArithmetic::Conventional => {
            out.iter_mut().for_each(|value| *value = 0.0);
            for (t, &attention) in att.iter().enumerate() {
                let value = &values[t * value_stride + value_offset..][..head_size];
                for i in 0..head_size {
                    out[i] += attention * value[i];
                }
            }
            AttentionDotCensus {
                lanes: head_size,
                conventional: head_size,
                ..AttentionDotCensus::default()
            }
        }
        AttentionArithmetic::Exact | AttentionArithmetic::CertifiedNative => {
            controlled_strided_matrix_vector(
                out,
                &values[value_offset..],
                att.len(),
                1,
                value_stride,
                att,
                arithmetic,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    struct CountingAllocator;

    thread_local! {
        static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
        static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    }

    fn record_allocation(pointer: *mut u8) {
        if !pointer.is_null() {
            let _ = COUNT_ALLOCATIONS.try_with(|gate| {
                if gate.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                }
            });
        }
    }

    // SAFETY: every request is forwarded unchanged to `System`; the
    // thread-local cells are observational and never participate in allocation.
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // SAFETY: the backing allocator receives the original request.
            let pointer = unsafe { System.alloc(layout) };
            record_allocation(pointer);
            pointer
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY: the allocation came from `System` with this layout.
            unsafe { System.dealloc(pointer, layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            // SAFETY: the backing allocator receives the original request.
            let pointer = unsafe { System.alloc_zeroed(layout) };
            record_allocation(pointer);
            pointer
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            // SAFETY: the backing allocator receives the original request.
            let resized = unsafe { System.realloc(pointer, layout, size) };
            record_allocation(resized);
            resized
        }
    }

    #[global_allocator]
    static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

    struct AllocationMeasurement;

    impl Drop for AllocationMeasurement {
        fn drop(&mut self) {
            COUNT_ALLOCATIONS.with(|gate| gate.set(false));
        }
    }

    fn counted_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
        ALLOCATIONS.with(|count| count.set(0));
        COUNT_ALLOCATIONS.with(|gate| {
            assert!(!gate.replace(true), "nested allocation measurement");
        });
        let measurement = AllocationMeasurement;
        let result = operation();
        drop(measurement);
        (result, ALLOCATIONS.with(Cell::get))
    }

    fn ramp(len: usize, seed: usize) -> Vec<f32> {
        (0..len)
            .map(|index| (((index * 37 + seed * 13) % 101) as f32 - 50.0) / 8.0)
            .collect()
    }

    /// Independent softmax reference: max-subtracted exp then divide by
    /// the sum, written apart from the implementation under test.
    fn softmax_reference(scores: &[f32]) -> Vec<f32> {
        let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = scores.iter().map(|s| (s - max).exp()).collect();
        let sum: f32 = exps.iter().sum();
        exps.iter().map(|e| e / sum).collect()
    }

    /// A key/value cache region for `positions` positions with
    /// `kv_heads` heads of width `head_size` each.
    fn cache(positions: usize, kv_heads: usize, head_size: usize, seed: usize) -> Vec<f32> {
        ramp(positions * kv_heads * head_size, seed)
    }

    fn assert_pinned_record(record: &AttentionOperatorSpec, pinned: &str) {
        assert_eq!(record.canonical_bytes(), pinned.as_bytes());
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(record.implementation_digest, expected);
        assert_eq!(record.declared_digest(), expected);
    }

    fn assert_pinned_digest(record: &AttentionOperatorSpec, expected: &str) {
        assert_eq!(record.implementation_digest, expected);
        assert_eq!(record.declared_digest(), expected);
    }

    #[test]
    fn raw_control_layout_rejection_is_failure_atomic() {
        let poison = f32::from_bits(0x7fc0_704a);

        let mut zero_width = [poison; 2];
        let zero_width_before = zero_width.map(f32::to_bits);
        let zero_width_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            head_dot_products_with_arithmetic(
                &mut zero_width,
                &[],
                &[],
                0,
                1,
                0,
                AttentionArithmetic::Conventional,
            )
        }));
        assert!(zero_width_error.is_err());
        assert_eq!(zero_width.map(f32::to_bits), zero_width_before);

        let mut qk = [poison; 2];
        let qk_before = qk.map(f32::to_bits);
        let qk_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            head_dot_products_with_arithmetic(
                &mut qk,
                &[1.0, 2.0, 3.0, 4.0],
                &[1.0, 2.0, 3.0, 4.0],
                0,
                4,
                4,
                AttentionArithmetic::CertifiedNative,
            )
        }));
        assert!(qk_error.is_err());
        assert_eq!(qk.map(f32::to_bits), qk_before);

        let mut experimental = [poison; 2];
        let experimental_before = experimental.map(f32::to_bits);
        let experimental_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            experimental_r4_head_attention_weights_with_arithmetic(
                &mut experimental,
                &[1.0, 2.0, 3.0, 4.0],
                &[1.0, 2.0, 3.0, 4.0],
                0,
                4,
                false,
                AttentionArithmetic::Conventional,
            )
        }));
        assert!(experimental_error.is_err());
        assert_eq!(experimental.map(f32::to_bits), experimental_before);

        let mut value = [poison; 2];
        let value_before = value.map(f32::to_bits);
        let value_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            head_attention_value_aggregate_with_arithmetic(
                &mut value,
                &[0.25, 0.75],
                &[1.0, 2.0],
                0,
                2,
                AttentionArithmetic::CertifiedNative,
            )
        }));
        assert!(value_error.is_err());
        assert_eq!(value.map(f32::to_bits), value_before);

        let oversized_stride = (isize::MAX as usize) + 1;
        let mut oversized_row = [poison; 1];
        let oversized_row_before = oversized_row.map(f32::to_bits);
        let oversized_row_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            head_dot_products_with_arithmetic(
                &mut oversized_row,
                &[1.0],
                &[1.0],
                0,
                oversized_stride,
                1,
                AttentionArithmetic::CertifiedNative,
            )
        }));
        assert!(oversized_row_error.is_err());
        assert_eq!(oversized_row.map(f32::to_bits), oversized_row_before);

        let mut oversized_column = [poison; 2];
        let oversized_column_before = oversized_column.map(f32::to_bits);
        let oversized_column_error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            head_attention_value_aggregate_with_arithmetic(
                &mut oversized_column,
                &[1.0],
                &[1.0, f32::NAN],
                0,
                oversized_stride,
                AttentionArithmetic::CertifiedNative,
            )
        }));
        assert!(oversized_column_error.is_err());
        assert_eq!(oversized_column.map(f32::to_bits), oversized_column_before);
    }

    #[test]
    fn certified_and_forced_fallback_raw_controls_allocate_nothing() {
        let safe_query = [1.0f32, -2.0, 0.25, 4.0];
        let safe_keys = [0.5f32, 0.75, -2.0, 0.25, -0.25, 0.5, 4.0, -0.5];
        let mut safe_output = [0.0f32; 2];
        let (safe, allocations) = counted_allocations(|| {
            head_dot_products_with_arithmetic(
                &mut safe_output,
                &safe_query,
                &safe_keys,
                0,
                4,
                4,
                AttentionArithmetic::CertifiedNative,
            )
        });
        assert_eq!(allocations, 0, "certified QK allocated");
        assert!(safe.certified > 0);

        let tie_term = f32::from_bits(115 << 23);
        let mut fallback_output = [0.0f32; 1];
        let (fallback, allocations) = counted_allocations(|| {
            head_dot_products_with_arithmetic(
                &mut fallback_output,
                &[1.0, tie_term],
                &[1.0, tie_term],
                0,
                2,
                2,
                AttentionArithmetic::CertifiedNative,
            )
        });
        assert_eq!(allocations, 0, "exact QK fallback allocated");
        assert_eq!(fallback.fallback_cell, 1);

        let mut value_output = [0.0f32; 1];
        let (value, allocations) = counted_allocations(|| {
            head_attention_value_aggregate_with_arithmetic(
                &mut value_output,
                &[1.0, tie_term],
                &[1.0, tie_term],
                0,
                1,
                AttentionArithmetic::CertifiedNative,
            )
        });
        assert_eq!(allocations, 0, "exact value fallback allocated");
        assert_eq!(value.fallback_cell, 1);
    }

    #[test]
    fn current_llama_wrappers_materially_dispatch_certified_native() {
        let query = [1.0e20f32, 1.0, -1.0e20];
        let keys = [1.0f32, 1.0, 1.0, 1.0, 0.0, 1.0];
        let mut shipped = [0.0; 2];
        let mut certified = [0.0; 2];
        let mut historical = [0.0; 2];
        standard_head_attention_weights(&mut shipped, &query, &keys, 0, 3, false);
        standard_head_attention_weights_with_arithmetic(
            &mut certified,
            &query,
            &keys,
            0,
            3,
            false,
            AttentionArithmetic::CertifiedNative,
        );
        standard_head_attention_weights_with_arithmetic(
            &mut historical,
            &query,
            &keys,
            0,
            3,
            false,
            AttentionArithmetic::Conventional,
        );
        assert_exact_bits(&shipped, &certified, "standard wrapper");
        assert_ne!(shipped.map(f32::to_bits), historical.map(f32::to_bits));

        let query = [1.0e20f32, 1.0, -1.0e20, 0.0];
        let keys = [1.0f32, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let mut shipped = [0.0; 2];
        let mut certified = [0.0; 2];
        let mut historical = [0.0; 2];
        experimental_r4_head_attention_weights(&mut shipped, &query, &keys, 0, 4, false);
        experimental_r4_head_attention_weights_with_arithmetic(
            &mut certified,
            &query,
            &keys,
            0,
            4,
            false,
            AttentionArithmetic::CertifiedNative,
        );
        experimental_r4_head_attention_weights_with_arithmetic(
            &mut historical,
            &query,
            &keys,
            0,
            4,
            false,
            AttentionArithmetic::Conventional,
        );
        assert_exact_bits(&shipped, &certified, "experimental wrapper");
        assert_ne!(shipped.map(f32::to_bits), historical.map(f32::to_bits));

        let attention = [1.0e20f32, 1.0, -1.0e20];
        let values = [1.0f32, 1.0, 1.0];
        let mut shipped = [0.0; 1];
        let mut certified = [0.0; 1];
        let mut historical = [0.0; 1];
        head_attention_value_aggregate(&mut shipped, &attention, &values, 0, 1);
        head_attention_value_aggregate_with_arithmetic(
            &mut certified,
            &attention,
            &values,
            0,
            1,
            AttentionArithmetic::CertifiedNative,
        );
        head_attention_value_aggregate_with_arithmetic(
            &mut historical,
            &attention,
            &values,
            0,
            1,
            AttentionArithmetic::Conventional,
        );
        assert_exact_bits(&shipped, &certified, "value wrapper");
        assert_ne!(shipped.map(f32::to_bits), historical.map(f32::to_bits));
    }

    fn assert_exact_bits(got: &[f32], expected: &[f32], context: &str) {
        assert_eq!(got.len(), expected.len(), "{context}: length");
        for (lane, (&got, &expected)) in got.iter().zip(expected).enumerate() {
            assert_eq!(
                got.to_bits(),
                expected.to_bits(),
                "{context}: lane {lane}: got={got:?} expected={expected:?}"
            );
        }
    }

    fn independent_exact_dot(left: &[f32], right: &[f32]) -> f32 {
        assert_eq!(left.len(), right.len());
        if left.is_empty() {
            return 0.0;
        }
        let mut output = [f32::from_bits(0x7fc0_704a)];
        let mut packed_left = vec![uor_matmul::PackedCode::default(); left.len()];
        let mut packed_right = vec![uor_matmul::PackedCode::default(); right.len()];
        uor_matmul::slice::gemm_float(
            1,
            left.len(),
            1,
            left,
            right,
            &mut output,
            &mut packed_left,
            &mut packed_right,
        )
        .expect("independently gathered operands form one exact dot");
        output[0]
    }

    fn verify_qk_exact(
        context: &str,
        rows: usize,
        width: usize,
        query: &[f32],
        keys: &[f32],
        key_offset: usize,
        key_stride: usize,
    ) -> AttentionDotCensus {
        let independent: Vec<f32> = (0..rows)
            .map(|row| {
                let key: Vec<f32> = (0..width)
                    .map(|column| keys[row * key_stride + key_offset + column])
                    .collect();
                independent_exact_dot(&query[..width], &key)
            })
            .collect();
        let mut exact = vec![f32::NAN; rows];
        let exact_census = head_dot_products_with_arithmetic(
            &mut exact,
            query,
            keys,
            key_offset,
            key_stride,
            width,
            AttentionArithmetic::Exact,
        );
        assert_eq!(exact_census.exact_control, rows);
        assert_exact_bits(&exact, &independent, context);

        let mut candidate = vec![f32::NAN; rows];
        let census = head_dot_products_with_arithmetic(
            &mut candidate,
            query,
            keys,
            key_offset,
            key_stride,
            width,
            AttentionArithmetic::CertifiedNative,
        );
        assert_exact_bits(&candidate, &exact, context);
        assert_eq!(census.certified + census.fallbacks(), rows);
        census
    }

    fn verify_value_exact(
        context: &str,
        head_size: usize,
        attention: &[f32],
        values: &[f32],
        value_offset: usize,
        value_stride: usize,
    ) -> AttentionDotCensus {
        let independent: Vec<f32> = (0..head_size)
            .map(|lane| {
                let value_lane: Vec<f32> = (0..attention.len())
                    .map(|position| values[position * value_stride + value_offset + lane])
                    .collect();
                independent_exact_dot(attention, &value_lane)
            })
            .collect();
        let mut exact = vec![f32::NAN; head_size];
        let exact_census = head_attention_value_aggregate_with_arithmetic(
            &mut exact,
            attention,
            values,
            value_offset,
            value_stride,
            AttentionArithmetic::Exact,
        );
        assert_eq!(exact_census.exact_control, head_size);
        assert_exact_bits(&exact, &independent, context);

        let mut candidate = vec![f32::NAN; head_size];
        let census = head_attention_value_aggregate_with_arithmetic(
            &mut candidate,
            attention,
            values,
            value_offset,
            value_stride,
            AttentionArithmetic::CertifiedNative,
        );
        assert_exact_bits(&candidate, &exact, context);
        assert_eq!(census.certified + census.fallbacks(), head_size);
        census
    }

    fn splitmix(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = *seed;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn moderate_finite(seed: &mut u64) -> f32 {
        let bits = splitmix(seed);
        let sign = ((bits >> 63) as u32) << 31;
        let exponent = 104 + ((bits >> 32) as u32 % 48);
        let fraction = bits as u32 & 0x007f_ffff;
        f32::from_bits(sign | (exponent << 23) | fraction)
    }

    #[test]
    fn certified_controls_match_independently_gathered_exact_dots() {
        let tie_term = f32::from_bits(115 << 23);
        let mut qk = AttentionDotCensus::default();
        let mut value = AttentionDotCensus::default();

        for part in [
            verify_qk_exact(
                "ordinary strided QK",
                2,
                4,
                &[1.0, -2.0, 0.25, 4.0],
                &[
                    99.0, 0.5, 0.75, -2.0, 0.25, 77.0, 98.0, -0.25, 0.5, 4.0, -0.5, 76.0,
                ],
                1,
                6,
            ),
            verify_qk_exact(
                "positive RNE tie",
                1,
                2,
                &[1.0, tie_term],
                &[1.0, tie_term],
                0,
                2,
            ),
            verify_qk_exact(
                "exact cancellation to zero",
                1,
                2,
                &[1.0, 1.0],
                &[1.0, -1.0],
                0,
                2,
            ),
            verify_qk_exact(
                "large cancellation residue",
                1,
                3,
                &[1.0e20, 1.0, -1.0e20],
                &[1.0, 1.0, 1.0],
                0,
                3,
            ),
            verify_qk_exact("overflow", 1, 1, &[f32::MAX], &[f32::MAX], 0, 1),
            verify_qk_exact("nonfinite", 1, 1, &[f32::INFINITY], &[1.0], 0, 1),
        ] {
            qk.merge(part);
        }
        for part in [
            verify_value_exact(
                "ordinary strided values",
                2,
                &[0.25, -0.5, 0.75],
                &[99.0, 1.0, 2.0, 98.0, -2.0, 4.0, 97.0, 0.25, -1.0],
                1,
                3,
            ),
            verify_value_exact("value RNE tie", 1, &[1.0, tie_term], &[1.0, tie_term], 0, 1),
            verify_value_exact(
                "value cancellation",
                1,
                &[1.0e20, 1.0, -1.0e20],
                &[1.0, 1.0, 1.0],
                0,
                1,
            ),
            verify_value_exact("value zero", 1, &[1.0, -1.0], &[1.0, 1.0], 0, 1),
            verify_value_exact("value overflow", 1, &[f32::MAX], &[f32::MAX], 0, 1),
            verify_value_exact("value nonfinite", 1, &[f32::NAN], &[1.0], 0, 1),
        ] {
            value.merge(part);
        }

        let mut seed = 0x704a_77e5_d1a5_beef;
        for case in 0..256usize {
            let rows = 1 + splitmix(&mut seed) as usize % 12;
            let width = 1 + splitmix(&mut seed) as usize % 64;
            let key_offset = splitmix(&mut seed) as usize % 3;
            let key_stride = key_offset + width + splitmix(&mut seed) as usize % 4;
            let query: Vec<f32> = (0..width).map(|_| moderate_finite(&mut seed)).collect();
            let mut keys = vec![0.0; rows * key_stride];
            for row in 0..rows {
                for column in 0..width {
                    keys[row * key_stride + key_offset + column] = moderate_finite(&mut seed);
                }
            }
            if case % 61 == 0 {
                keys[key_offset] = f32::NEG_INFINITY;
            }
            qk.merge(verify_qk_exact(
                &format!("random QK {case}"),
                rows,
                width,
                &query,
                &keys,
                key_offset,
                key_stride,
            ));

            let head_size = 1 + splitmix(&mut seed) as usize % 32;
            let value_offset = splitmix(&mut seed) as usize % 3;
            let value_stride = value_offset + head_size + splitmix(&mut seed) as usize % 4;
            let attention: Vec<f32> = (0..rows).map(|_| moderate_finite(&mut seed)).collect();
            let mut values = vec![0.0; rows * value_stride];
            for position in 0..rows {
                for lane in 0..head_size {
                    values[position * value_stride + value_offset + lane] =
                        moderate_finite(&mut seed);
                }
            }
            if case % 67 == 0 {
                values[value_offset] = f32::INFINITY;
            }
            value.merge(verify_value_exact(
                &format!("random value {case}"),
                head_size,
                &attention,
                &values,
                value_offset,
                value_stride,
            ));
        }

        for (kind, census) in [("QK", qk), ("value", value)] {
            assert!(census.certified > 0, "{kind}: no certified lane");
            assert!(census.fallback_nonfinite > 0, "{kind}: nonfinite arm");
            assert!(census.fallback_zero > 0, "{kind}: zero arm");
            assert!(census.fallback_overflow > 0, "{kind}: overflow arm");
            assert!(census.fallback_cell > 0, "{kind}: cell arm");
        }
        eprintln!("CERTIFIED_ATTENTION_PARITY qk={qk:?} value={value:?}");
    }

    #[test]
    fn standard_weights_score_every_dimension_divisible_width() {
        // Divisible head width (H = 8): the score is the full-width dot
        // product over sqrt(H), softmax-normalized.
        let head_size = 8;
        let positions = 5;
        let q = ramp(head_size, 1);
        let keys = cache(positions, 2, head_size, 2);
        let kv_stride = 2 * head_size;
        let key_offset = head_size; // second kv head
        let mut att = vec![0f32; positions];
        standard_head_attention_weights(&mut att, &q, &keys, key_offset, kv_stride, false);

        let mut raw = vec![0f32; positions];
        for (t, slot) in raw.iter_mut().enumerate() {
            let k = &keys[t * kv_stride + key_offset..][..head_size];
            let mut score = 0.0f32;
            for i in 0..head_size {
                score += q[i] * k[i];
            }
            *slot = score / (head_size as f32).sqrt();
        }
        let expected = softmax_reference(&raw);
        for (index, (&got, &want)) in att.iter().zip(&expected).enumerate() {
            assert!((got - want).abs() <= 1e-6, "position {index}");
        }
        let total: f32 = att.iter().sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn standard_weights_have_no_remainder_at_non_divisible_width() {
        // Non-divisible head width (H = 6, H mod 4 = 2): the standard
        // operator has NO remainder policy — every dimension is scored,
        // pinned by perturbing the trailing dimension and observing the
        // weights move.
        let head_size = 6;
        let positions = 4;
        let q = ramp(head_size, 3);
        let keys = cache(positions, 1, head_size, 4);
        let mut att = vec![0f32; positions];
        standard_head_attention_weights(&mut att, &q, &keys, 0, head_size, false);

        let mut q_perturbed = q.clone();
        q_perturbed[head_size - 1] += 1.0;
        let mut att_perturbed = vec![0f32; positions];
        standard_head_attention_weights(
            &mut att_perturbed,
            &q_perturbed,
            &keys,
            0,
            head_size,
            false,
        );
        assert_ne!(
            att, att_perturbed,
            "the trailing head dimension must enter the standard score"
        );
    }

    #[test]
    fn experimental_weights_truncate_the_trailing_mod_4_dimensions() {
        // The measured remainder policy of the r4_attention branch: at
        // H = 6 only dimensions 0..4 (one 4-wide chunk) enter any
        // score. Perturbing q[4], q[5], k[4], k[5] leaves the produced
        // weights bit-identical; perturbing q[3] moves them.
        let head_size = 6;
        let positions = 4;
        let q = ramp(head_size, 5);
        let keys = cache(positions, 1, head_size, 6);
        let mut att = vec![0f32; positions];
        experimental_r4_head_attention_weights(&mut att, &q, &keys, 0, head_size, false);

        let mut q_tail = q.clone();
        q_tail[4] += 3.0;
        q_tail[5] -= 2.0;
        let mut keys_tail = keys.clone();
        for t in 0..positions {
            keys_tail[t * head_size + 4] += 1.5;
            keys_tail[t * head_size + 5] += 2.5;
        }
        let mut att_tail = vec![0f32; positions];
        experimental_r4_head_attention_weights(
            &mut att_tail,
            &q_tail,
            &keys_tail,
            0,
            head_size,
            false,
        );
        let bits: Vec<u32> = att.iter().map(|v| v.to_bits()).collect();
        let tail_bits: Vec<u32> = att_tail.iter().map(|v| v.to_bits()).collect();
        assert_eq!(
            bits, tail_bits,
            "trailing head_size mod 4 dimensions must never enter the experimental score"
        );

        let mut q_head = q.clone();
        q_head[3] += 1.0;
        let mut att_head = vec![0f32; positions];
        experimental_r4_head_attention_weights(&mut att_head, &q_head, &keys, 0, head_size, false);
        assert_ne!(att, att_head, "scored dimensions must move the weights");
    }

    #[test]
    fn experimental_scale_still_divides_by_sqrt_full_head_size() {
        // H = 6 scores only 4 dimensions but divides by sqrt(6), not
        // sqrt(4): pinned against an independent computation.
        let head_size = 6;
        let positions = 3;
        let q = ramp(head_size, 7);
        let keys = cache(positions, 1, head_size, 8);
        let mut att = vec![0f32; positions];
        experimental_r4_head_attention_weights(&mut att, &q, &keys, 0, head_size, false);

        let mut raw = vec![0f32; positions];
        for (t, slot) in raw.iter_mut().enumerate() {
            let k = &keys[t * head_size..][..head_size];
            let mut score = 0.0f32;
            for i in 0..4 {
                score += q[i] * k[i];
            }
            *slot = score / (head_size as f32).sqrt();
        }
        let expected = softmax_reference(&raw);
        for (index, (&got, &want)) in att.iter().zip(&expected).enumerate() {
            assert!((got - want).abs() <= 1e-6, "position {index}");
        }
    }

    #[test]
    fn experimental_weights_are_uniform_below_chunk_width() {
        // H < 4: no chunk exists, every score is 0, and the selector —
        // still a softmax, never bypassed — yields uniform weights.
        let head_size = 3;
        let positions = 5;
        let q = ramp(head_size, 9);
        let keys = cache(positions, 1, head_size, 10);
        let mut att = vec![0f32; positions];
        experimental_r4_head_attention_weights(&mut att, &q, &keys, 0, head_size, false);
        for (index, &weight) in att.iter().enumerate() {
            assert!(
                (weight - 1.0 / positions as f32).abs() <= 1e-6,
                "position {index}: {weight}"
            );
        }
    }

    #[test]
    fn both_operators_normalize_with_the_same_softmax() {
        // The honest selector statement: BOTH branches apply the same
        // max-subtracted softmax. At a divisible width the two weight
        // vectors agree to tolerance (same scored terms, different fold
        // grouping), and both sum to 1.
        let head_size = 8;
        let positions = 6;
        let q = ramp(head_size, 11);
        let keys = cache(positions, 1, head_size, 12);
        let mut standard = vec![0f32; positions];
        let mut experimental = vec![0f32; positions];
        standard_head_attention_weights(&mut standard, &q, &keys, 0, head_size, false);
        experimental_r4_head_attention_weights(&mut experimental, &q, &keys, 0, head_size, false);
        for (index, (&s, &e)) in standard.iter().zip(&experimental).enumerate() {
            assert!((s - e).abs() <= 1e-5, "position {index}: {s} vs {e}");
        }
        let sum_s: f32 = standard.iter().sum();
        let sum_e: f32 = experimental.iter().sum();
        assert!((sum_s - 1.0).abs() < 1e-5);
        assert!((sum_e - 1.0).abs() < 1e-5);
    }

    #[test]
    fn value_aggregation_uses_every_dimension_in_position_order() {
        let head_size = 6;
        let positions = 4;
        let att = softmax_reference(&ramp(positions, 13));
        let values = cache(positions, 2, head_size, 14);
        let value_stride = 2 * head_size;
        let value_offset = head_size;
        let mut out = vec![7.0f32; head_size]; // pre-dirtied: must be zeroed
        head_attention_value_aggregate(&mut out, &att, &values, value_offset, value_stride);
        let mut expected = vec![f32::NAN; head_size];
        let census = head_attention_value_aggregate_with_arithmetic(
            &mut expected,
            &att,
            &values,
            value_offset,
            value_stride,
            AttentionArithmetic::Exact,
        );
        assert_eq!(census.exact_control, head_size);
        assert_eq!(
            out.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            expected
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn empty_attention_is_the_zero_weighted_sum_for_every_arithmetic_owner() {
        let mut public = [7.0f32, 7.0];
        head_attention_value_aggregate(&mut public, &[], &[], 0, 1);
        assert_eq!(public.map(f32::to_bits), [0.0f32.to_bits(); 2]);

        for arithmetic in [
            AttentionArithmetic::Conventional,
            AttentionArithmetic::Exact,
            AttentionArithmetic::CertifiedNative,
        ] {
            let mut output = [7.0f32, 7.0];
            let census = head_attention_value_aggregate_with_arithmetic(
                &mut output,
                &[],
                &[],
                usize::MAX,
                usize::MAX,
                arithmetic,
            );
            assert_eq!(output.map(f32::to_bits), [0.0f32.to_bits(); 2]);
            assert_eq!(census.lanes, 2);
            match arithmetic {
                AttentionArithmetic::Conventional => {
                    assert_eq!(census.conventional, 2);
                    assert_eq!(census.exact_control, 0);
                    assert_eq!(census.fallbacks(), 0);
                }
                AttentionArithmetic::Exact => {
                    assert_eq!(census.conventional, 0);
                    assert_eq!(census.exact_control, 2);
                    assert_eq!(census.fallbacks(), 0);
                }
                AttentionArithmetic::CertifiedNative => {
                    assert_eq!(census.conventional, 0);
                    assert_eq!(census.exact_control, 0);
                    assert_eq!(census.certified, 0);
                    assert_eq!(census.fallback_zero, 2);
                    assert_eq!(census.fallbacks(), 2);
                }
            }
        }
    }

    #[test]
    fn zero_extent_public_attention_weights_are_total_and_uniform() {
        for weights in [
            standard_head_attention_weights as fn(&mut [f32], &[f32], &[f32], usize, usize, bool),
            experimental_r4_head_attention_weights,
        ] {
            let mut empty = [];
            weights(&mut empty, &[], &[], usize::MAX, usize::MAX, false);
            weights(
                &mut empty,
                &[1.0, 2.0, 3.0, 4.0],
                &[],
                usize::MAX,
                usize::MAX,
                false,
            );

            let mut singleton = [123.0f32];
            weights(&mut singleton, &[], &[], usize::MAX, usize::MAX, false);
            assert_eq!(singleton.map(f32::to_bits), [1.0f32.to_bits()]);

            let mut weights3 = [123.0f32; 3];
            weights(&mut weights3, &[], &[], usize::MAX, usize::MAX, true);
            assert!(weights3.iter().all(|weight| weight.is_finite()));
            assert_eq!(weights3[0].to_bits(), weights3[1].to_bits());
            assert_eq!(weights3[1].to_bits(), weights3[2].to_bits());
        }

        for head_size in 1..4 {
            let mut below_chunk = [123.0f32; 3];
            experimental_r4_head_attention_weights(
                &mut below_chunk,
                &vec![1.0; head_size],
                &[],
                usize::MAX,
                usize::MAX,
                false,
            );
            assert!(below_chunk.iter().all(|weight| weight.is_finite()));
            assert_eq!(below_chunk[0].to_bits(), below_chunk[1].to_bits());
            assert_eq!(below_chunk[1].to_bits(), below_chunk[2].to_bits());

            let mut exact = [123.0f32; 2];
            let census = experimental_r4_head_attention_weights_with_arithmetic(
                &mut exact,
                &vec![1.0; head_size],
                &[],
                usize::MAX,
                usize::MAX,
                true,
                AttentionArithmetic::Exact,
            );
            assert_eq!(exact[0].to_bits(), exact[1].to_bits());
            assert_eq!(census.exact_control, exact.len());
        }

        let mut no_value_lanes = [];
        head_attention_value_aggregate(&mut no_value_lanes, &[1.0], &[], usize::MAX, usize::MAX);

        for arithmetic in [
            AttentionArithmetic::Conventional,
            AttentionArithmetic::Exact,
            AttentionArithmetic::CertifiedNative,
        ] {
            let mut output = [123.0f32; 2];
            let census = standard_head_attention_weights_with_arithmetic(
                &mut output,
                &[],
                &[],
                usize::MAX,
                usize::MAX,
                false,
                arithmetic,
            );
            assert_eq!(output[0].to_bits(), output[1].to_bits());
            assert_eq!(census.lanes, output.len());
            match arithmetic {
                AttentionArithmetic::Conventional => {
                    assert_eq!(census.conventional, output.len())
                }
                AttentionArithmetic::Exact => assert_eq!(census.exact_control, output.len()),
                AttentionArithmetic::CertifiedNative => {
                    assert_eq!(census.fallback_zero, output.len())
                }
            }
        }
    }

    #[test]
    fn weight_functions_are_bit_deterministic() {
        let head_size = 6;
        let positions = 5;
        let q = ramp(head_size, 15);
        let keys = cache(positions, 1, head_size, 16);
        for weights in [
            standard_head_attention_weights as fn(&mut [f32], &[f32], &[f32], usize, usize, bool),
            experimental_r4_head_attention_weights,
        ] {
            let mut first = vec![0f32; positions];
            let mut second = vec![0f32; positions];
            weights(&mut first, &q, &keys, 0, head_size, false);
            weights(&mut second, &q, &keys, 0, head_size, false);
            let first_bits: Vec<u32> = first.iter().map(|v| v.to_bits()).collect();
            let second_bits: Vec<u32> = second.iter().map(|v| v.to_bits()).collect();
            assert_eq!(first_bits, second_bits);
        }
    }

    #[test]
    fn legacy_standard_and_experimental_v1_canonical_bytes_remain_immutable() {
        // The canonical forms are pinned byte-for-byte: any drift in
        // field order, separators, or parameter tokens fails here — the
        // digest identity must not move silently.
        let standard = AttentionOperatorSpec::standard_v1();
        let pinned_standard = "uor-r4-attention-operator/1\n\
             id=standard-source-attention\n\
             version=1\n\
             projections=per-layer-dense-f32-wq-wk-wv\n\
             positional_action=rope-rotation-of-q-and-k-before-scoring\n\
             compatibility_relation=scaled-dot-product\n\
             selector_normalization=softmax-max-subtracted-exp-then-sum-normalize\n\
             value_aggregation=position-ascending-weighted-sum-of-values\n\
             output_projection=per-layer-dense-f32-wo\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32\n\
             param.head_selection=grouped-query-kv-head-h-div-kv-mul\n\
             param.score_scale=divide-by-sqrt-full-head-size\n\
             param.score_width_policy=full-head-width\n\
             param.remainder_policy=none-every-head-dimension-scored\n\
             param.score_accumulation=sequential-f32-left-fold\n";
        assert_eq!(standard.canonical_bytes(), pinned_standard.as_bytes());
        assert_pinned_record(&standard, pinned_standard);
        assert_pinned_digest(
            &standard,
            "blake3:d9520065caf35261af680b3b35c9893351166a739f04739df5216882ab5f3437",
        );

        let experimental = AttentionOperatorSpec::experimental_r4_v1();
        let pinned_experimental = "uor-r4-attention-operator/1\n\
             id=experimental-r4-source-attention\n\
             version=1\n\
             projections=per-layer-dense-f32-wq-wk-wv\n\
             positional_action=rope-rotation-of-q-and-k-before-scoring\n\
             compatibility_relation=chunked-4-wide-dot-product\n\
             selector_normalization=softmax-max-subtracted-exp-then-sum-normalize\n\
             value_aggregation=position-ascending-weighted-sum-of-values\n\
             output_projection=per-layer-dense-f32-wo\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32\n\
             param.head_selection=grouped-query-kv-head-h-div-kv-mul\n\
             param.score_scale=divide-by-sqrt-full-head-size\n\
             param.score_width_policy=chunks-of-4-floor-head-size-div-4\n\
             param.remainder_policy=truncate-trailing-head-size-mod-4-dims-from-score\n\
             param.score_accumulation=per-4-chunk-left-fold-then-chunk-sum\n";
        assert_pinned_record(&experimental, pinned_experimental);
        assert_pinned_digest(
            &experimental,
            "blake3:aedf1e7732b4a8396f8e3439a7d375fea269c5cb632ca014ff1fdd8c592a3bab",
        );
        assert_ne!(
            standard.implementation_digest, experimental.implementation_digest,
            "the two operators are distinct identities"
        );
        // Rebuilding reproduces the records bit-for-bit.
        assert_eq!(standard, AttentionOperatorSpec::standard_v1());
        assert_eq!(experimental, AttentionOperatorSpec::experimental_r4_v1());
    }

    #[test]
    fn current_source_v2_canonical_bytes_and_digests_are_pinned() {
        let standard = AttentionOperatorSpec::standard_v2();
        let pinned_standard = "uor-r4-attention-operator/1\n\
             id=standard-source-attention\n\
             version=2\n\
             projections=per-layer-dense-f32-wq-wk-wv\n\
             positional_action=rope-rotation-of-q-and-k-before-scoring\n\
             compatibility_relation=correctly-rounded-binary32-exact-real-full-width-dot\n\
             selector_normalization=divide-by-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-per-weight-divide\n\
             value_aggregation=correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n\
             output_projection=per-layer-dense-f32-wo\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback\n\
             param.head_selection=grouped-query-kv-head-h-div-kv-mul\n\
             param.score_scale=divide-by-sqrt-full-head-size\n\
             param.score_width_policy=full-head-width\n\
             param.remainder_policy=none-every-head-dimension-scored\n\
             param.score_accumulation=correctly-rounded-binary32-exact-real-dot-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n";
        assert_eq!(pinned_standard.len(), 1_075);
        assert_pinned_record(&standard, pinned_standard);
        assert_pinned_digest(
            &standard,
            "blake3:fa4c8f233e217d3903678b7690de5cdfb27d83a4b68c52436cfabbc6ca6cfc59",
        );

        let experimental = AttentionOperatorSpec::experimental_r4_v2();
        let pinned_experimental = "uor-r4-attention-operator/1\n\
             id=experimental-r4-source-attention\n\
             version=2\n\
             projections=per-layer-dense-f32-wq-wk-wv\n\
             positional_action=rope-rotation-of-q-and-k-before-scoring\n\
             compatibility_relation=correctly-rounded-binary32-exact-real-floor-multiple-of-4-width-dot\n\
             selector_normalization=divide-by-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-per-weight-divide\n\
             value_aggregation=correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n\
             output_projection=per-layer-dense-f32-wo\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback\n\
             param.head_selection=grouped-query-kv-head-h-div-kv-mul\n\
             param.score_scale=divide-by-sqrt-full-head-size\n\
             param.score_width_policy=chunks-of-4-floor-head-size-div-4\n\
             param.remainder_policy=truncate-trailing-head-size-mod-4-dims-from-score\n\
             param.score_accumulation=correctly-rounded-binary32-exact-real-dot-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n";
        assert_eq!(pinned_experimental.len(), 1_132);
        assert_pinned_record(&experimental, pinned_experimental);
        assert_pinned_digest(
            &experimental,
            "blake3:a71d3a9fbfd951528652b837a23d5bfd7742ba79d5082337c19db3a17776e654",
        );

        let learned = AttentionOperatorSpec::learned_absolute_v2();
        let pinned_learned = "uor-r4-attention-operator/1\n\
             id=learned-absolute-source-attention\n\
             version=2\n\
             projections=fused-c-attn-conv1d-qkv-with-bias\n\
             positional_action=none-learned-absolute-positions-added-to-input-embeddings\n\
             compatibility_relation=correctly-rounded-binary32-exact-real-full-width-dot\n\
             selector_normalization=multiply-by-reciprocal-sqrt-full-head-size-then-softmax-max-subtracted-exp-sum-reciprocal-multiply\n\
             value_aggregation=correctly-rounded-binary32-exact-real-position-weighted-sum-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n\
             output_projection=dense-c-proj-conv1d-with-bias\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback\n\
             param.head_selection=multi-head-identity-kv-head-equals-query-head\n\
             param.score_scale=multiply-by-reciprocal-sqrt-full-head-size\n\
             param.score_width_policy=full-head-width\n\
             param.remainder_policy=none-every-head-dimension-scored\n\
             param.score_accumulation=correctly-rounded-binary32-exact-real-dot-certified-native-f64-outward-cell-or-pinned-uor-matmul-exact-fallback\n";
        assert_eq!(pinned_learned.len(), 1_152);
        assert_pinned_record(&learned, pinned_learned);
        assert_pinned_digest(
            &learned,
            "blake3:ba36fd1fef53a2e3744e1fee60e72677870d1cd2f2b484db755c0a5a74727231",
        );

        assert_eq!(AttentionOperatorSpec::standard(), standard);
        assert_eq!(AttentionOperatorSpec::experimental_r4(), experimental);
        assert_eq!(
            AttentionOperatorSpec::learned_absolute_source_attention(),
            learned
        );
        assert_eq!(
            AttentionOperatorParams::standard(),
            AttentionOperatorParams::standard_v2()
        );
        assert_eq!(
            AttentionOperatorParams::experimental_r4(),
            AttentionOperatorParams::experimental_r4_v2()
        );
        assert_eq!(
            AttentionOperatorParams::learned_absolute(),
            AttentionOperatorParams::learned_absolute_v2()
        );
        for record in [&standard, &experimental, &learned] {
            assert_eq!(
                record.params.score_accumulation,
                CERTIFIED_NATIVE_ARITHMETIC_ID
            );
        }
    }

    #[test]
    fn source_v2_keeps_model_family_semantics_but_mismatches_v1() {
        for (v1, v2) in [
            (
                AttentionOperatorSpec::standard_v1(),
                AttentionOperatorSpec::standard_v2(),
            ),
            (
                AttentionOperatorSpec::experimental_r4_v1(),
                AttentionOperatorSpec::experimental_r4_v2(),
            ),
            (
                AttentionOperatorSpec::learned_absolute_v1(),
                AttentionOperatorSpec::learned_absolute_v2(),
            ),
        ] {
            assert_eq!(v2.id, v1.id);
            assert_eq!(v2.projections, v1.projections);
            assert_eq!(v2.positional_action, v1.positional_action);
            assert_eq!(v2.output_projection, v1.output_projection);
            assert_eq!(v2.runtime_state, v1.runtime_state);
            assert_eq!(v2.tie_breaking, v1.tie_breaking);
            assert_eq!(v2.params.head_selection, v1.params.head_selection);
            assert_eq!(v2.params.score_scale, v1.params.score_scale);
            assert_eq!(v2.params.score_width_policy, v1.params.score_width_policy);
            assert_eq!(v2.params.remainder_policy, v1.params.remainder_policy);
            assert_ne!(v2.version, v1.version);
            assert_ne!(v2.implementation_digest, v1.implementation_digest);
            assert_ne!(v2, v1, "same-id source eras must fail equality checks");
        }
        assert!(AttentionOperatorSpec::standard_v2()
            .selector_normalization
            .contains("per-weight-divide"));
        assert!(AttentionOperatorSpec::experimental_r4_v2()
            .params
            .remainder_policy
            .contains("truncate-trailing"));
        assert!(AttentionOperatorSpec::learned_absolute_v2()
            .selector_normalization
            .contains("reciprocal-multiply"));
    }

    #[test]
    fn record_round_trips_through_serde_json() {
        for record in [
            AttentionOperatorSpec::standard_v1(),
            AttentionOperatorSpec::standard(),
            AttentionOperatorSpec::experimental_r4_v1(),
            AttentionOperatorSpec::experimental_r4(),
            AttentionOperatorSpec::r4_route_attention_v1(),
            AttentionOperatorSpec::learned_absolute_v1(),
            AttentionOperatorSpec::learned_absolute_source_attention(),
        ] {
            let json = serde_json::to_string(&record).expect("serializes");
            let back: AttentionOperatorSpec = serde_json::from_str(&json).expect("deserializes");
            assert_eq!(record, back);
        }
        // Serde-defaulted fields: a legacy/partial document still parses.
        let partial: AttentionOperatorSpec =
            serde_json::from_str("{\"id\":\"standard-source-attention\"}").expect("defaults fill");
        assert_eq!(partial.id, "standard-source-attention");
        assert_eq!(partial.version, 0);
        assert_eq!(partial.params, AttentionOperatorParams::default());

        for json in [
            r#"{"id":"standard-source-attention","unregistered_claim":true}"#,
            r#"{"params":{"unregistered_claim":"exact"}}"#,
        ] {
            serde_json::from_str::<AttentionOperatorSpec>(json)
                .expect_err("unknown provenance claims must fail closed");
        }
    }

    #[test]
    fn switch_maps_to_exactly_the_two_registered_source_operators() {
        // #604 boundary: the legacy boolean selects between the two
        // SOURCE operators only; the target route operator is never
        // reachable through it.
        assert_eq!(
            operator_for_r4_switch(false),
            AttentionOperatorSpec::standard()
        );
        assert_eq!(
            operator_for_r4_switch(true),
            AttentionOperatorSpec::experimental_r4()
        );
        assert_ne!(
            operator_for_r4_switch(true),
            AttentionOperatorSpec::r4_route_attention_v1()
        );
    }

    #[test]
    fn registry_resolves_all_registered_operators() {
        for (id, version, expected) in [
            (
                AttentionOperatorSpec::STANDARD_ID,
                AttentionOperatorSpec::STANDARD_V1_VERSION,
                AttentionOperatorSpec::standard_v1(),
            ),
            (
                AttentionOperatorSpec::STANDARD_ID,
                AttentionOperatorSpec::STANDARD_V2_VERSION,
                AttentionOperatorSpec::standard_v2(),
            ),
            (
                AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
                AttentionOperatorSpec::EXPERIMENTAL_R4_V1_VERSION,
                AttentionOperatorSpec::experimental_r4_v1(),
            ),
            (
                AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
                AttentionOperatorSpec::EXPERIMENTAL_R4_V2_VERSION,
                AttentionOperatorSpec::experimental_r4_v2(),
            ),
            (
                AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                AttentionOperatorSpec::LEARNED_ABSOLUTE_V1_VERSION,
                AttentionOperatorSpec::learned_absolute_v1(),
            ),
            (
                AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                AttentionOperatorSpec::LEARNED_ABSOLUTE_V2_VERSION,
                AttentionOperatorSpec::learned_absolute_v2(),
            ),
            (
                AttentionOperatorSpec::R4_ROUTE_ID,
                AttentionOperatorSpec::R4_ROUTE_VERSION,
                AttentionOperatorSpec::r4_route_attention_v1(),
            ),
        ] {
            assert_eq!(
                operator_spec(id, version).expect("registered entry"),
                expected
            );
        }
    }

    #[test]
    fn r4_route_attention_canonical_serialization_is_byte_stable() {
        // The target operator's declared identity, pinned byte-for-byte
        // (#604): any drift in a field token is a version bump, never a
        // silent edit.
        let target = AttentionOperatorSpec::r4_route_attention_v1();
        let pinned = "uor-r4-attention-operator/1\n\
             id=r4-route-attention\n\
             version=1\n\
             projections=none-declared-route-code-tables-no-qkv-reuse\n\
             positional_action=none-route-codes-carry-no-positional-action\n\
             compatibility_relation=masked-xor-popcount\n\
             selector_normalization=none-bounded-top-m-selection\n\
             value_aggregation=selection-order-saturating-scoreq-add\n\
             output_projection=none-aggregate-scoreq-only\n\
             runtime_state=caller-owned-fixed-capacity-epoch-stamped-selection-state\n\
             tie_breaking=lowest-candidate-index-on-equal-masked-popcount-distance\n\
             permitted_operation_class=deployed-integer-xor-popcount-add-compare-table-read\n\
             param.head_selection=single-route-lane\n\
             param.score_scale=none-integer-popcount-distance\n\
             param.score_width_policy=declared-288-bit-mask-over-route-code\n\
             param.remainder_policy=unmasked-bits-never-scored\n\
             param.score_accumulation=per-byte-popcount-table-add-left-fold\n";
        assert_eq!(target.canonical_bytes(), pinned.as_bytes());
        assert_eq!(target.canonical_bytes().len(), 860);
        assert_pinned_digest(
            &target,
            "blake3:33e5e5c58d4a33caea0409f1263df4213419e52bb5999144570dd959e5ee151d",
        );
        // Distinct identity from both source operators.
        assert_ne!(
            target.implementation_digest,
            AttentionOperatorSpec::standard().implementation_digest
        );
        assert_ne!(
            target.implementation_digest,
            AttentionOperatorSpec::experimental_r4().implementation_digest
        );
        // The permitted class is the deployed integer class, unlike the
        // host-side source records.
        assert_eq!(
            target.permitted_operation_class,
            "deployed-integer-xor-popcount-add-compare-table-read"
        );
        assert_eq!(
            AttentionOperatorSpec::standard_v1().permitted_operation_class,
            "host-source-f32"
        );
        assert_eq!(
            AttentionOperatorSpec::standard().permitted_operation_class,
            "host-source-f32-f64-certified-plus-pinned-uor-matmul-exact-fallback"
        );
        // Rebuilding reproduces the record bit-for-bit.
        assert_eq!(target, AttentionOperatorSpec::r4_route_attention_v1());
    }

    #[test]
    fn legacy_learned_absolute_v1_canonical_bytes_remain_immutable() {
        // #668: the GPT-2-family operator's declared identity, pinned
        // byte-for-byte. Any drift in a field token is a version bump,
        // never a silent edit.
        let learned = AttentionOperatorSpec::learned_absolute_v1();
        let pinned = "uor-r4-attention-operator/1\n\
             id=learned-absolute-source-attention\n\
             version=1\n\
             projections=fused-c-attn-conv1d-qkv-with-bias\n\
             positional_action=none-learned-absolute-positions-added-to-input-embeddings\n\
             compatibility_relation=scaled-dot-product\n\
             selector_normalization=softmax-max-subtracted-exp-then-sum-normalize\n\
             value_aggregation=position-ascending-weighted-sum-of-values\n\
             output_projection=dense-c-proj-conv1d-with-bias\n\
             runtime_state=growing-kv-cache-full-prefix\n\
             tie_breaking=first-maximum-softmax-stabilizer-value-identical\n\
             permitted_operation_class=host-source-f32\n\
             param.head_selection=multi-head-identity-kv-head-equals-query-head\n\
             param.score_scale=multiply-by-reciprocal-sqrt-full-head-size\n\
             param.score_width_policy=full-head-width\n\
             param.remainder_policy=none-every-head-dimension-scored\n\
             param.score_accumulation=sequential-f32-left-fold\n";
        assert_eq!(learned.canonical_bytes(), pinned.as_bytes());
        assert_pinned_record(&learned, pinned);
        assert_pinned_digest(
            &learned,
            "blake3:00088e46d2b68616f8e58c33ce6b621925f82be5b22607f080e5f142e753796f",
        );

        // It shares the standard operator's compatibility relation and
        // softmax selector, but is a DISTINCT identity: the positional
        // action and projections differ, so the declared digest differs.
        let standard = AttentionOperatorSpec::standard_v1();
        assert_eq!(
            learned.compatibility_relation,
            standard.compatibility_relation
        );
        assert_eq!(
            learned.selector_normalization,
            standard.selector_normalization
        );
        assert_ne!(
            learned.positional_action, standard.positional_action,
            "GPT-2 uses learned absolute positions, not RoPE"
        );
        assert_ne!(
            learned.implementation_digest, standard.implementation_digest,
            "learned-absolute is a distinct operator identity"
        );
        assert_ne!(
            learned.implementation_digest,
            AttentionOperatorSpec::experimental_r4_v1().implementation_digest
        );
        // Rebuilding reproduces the record bit-for-bit.
        assert_eq!(learned, AttentionOperatorSpec::learned_absolute_v1());
    }

    #[test]
    fn registry_refuses_unknown_id_and_version_by_name() {
        for (id, version) in [
            ("standard-source-attention", 3u32),
            ("experimental-r4-source-attention", 3),
            ("learned-absolute-source-attention", 3),
            ("r4-route-attention", 2),
            ("mystery-attention", 1),
        ] {
            let error =
                operator_spec(id, version).expect_err("unknown (id, version) is not a product");
            match &error.kind {
                crate::SourceIngestKind::UnknownAttentionOperator {
                    id: got_id,
                    version: got_version,
                } => {
                    assert_eq!(got_id, id);
                    assert_eq!(*got_version, version);
                }
                other => panic!("wrong failure class: {other:?}"),
            }
            assert!(error.reason.contains(id), "reason names the id: {error}");
        }
    }
}
