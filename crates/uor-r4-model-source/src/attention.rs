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
//! - [`standard_head_attention_weights`] and
//!   [`head_attention_value_aggregate`] are the free, deterministic
//!   reference implementations of `standard-source-attention/1` — the
//!   exact arithmetic (iteration order, sequential f32 folds, one divide
//!   per score) the teacher has always used, factored out unchanged.
//! - [`experimental_r4_head_attention_weights`] is the factored
//!   experimental branch, recorded under the honest id
//!   `experimental-r4-source-attention/1` with its ACTUAL computation
//!   (see below) — not the historical description.
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
//! growing KV cache attending the full prefix `0..=pos`; value
//! aggregation as a position-ascending weighted sum `out[i] +=
//! att[t] * v_t[i]`; and a dense per-layer f32 `wo` output projection.
//! Both branches normalize scores with the SAME selector,
//! `softmax_with_mode`: subtract the maximum score (first maximum on
//! ties — value-identical, since only the maximum's value enters), then
//! `exp` each shifted score (`libm::expf` in canonical mode, `f32::exp`
//! otherwise) and divide by the sum. Neither branch performs an argmax
//! or any selection needing a tie-break beyond that stabilizer.
//!
//! They differ ONLY in the compatibility relation:
//!
//! - **standard**: `score(t) = (Σ_{i<H} q[i]·k_t[i]) / sqrt(H)` for head
//!   width `H`, accumulated as a single sequential f32 left fold over
//!   every head dimension.
//! - **experimental**: `score(t) = (Σ_{c<⌊H/4⌋} Σ_{j<4}
//!   q[4c+j]·k_t[4c+j]) / sqrt(H)` — the dot product computed in 4-wide
//!   chunks (each chunk a left-to-right 4-term fold, chunk subtotals
//!   then summed), which is still a dot product followed by the same
//!   softmax. **Remainder policy**: the trailing `H mod 4` q/k
//!   dimensions are never read by the score (dropped), while the scale
//!   divides by `sqrt(H)` over the FULL head width and value aggregation
//!   still uses every head dimension. For `H < 4` no chunk exists, every
//!   score is 0, and the softmax yields uniform weights over the prefix.
//!   For `H` divisible by 4 the same terms are scored but the chunked
//!   accumulation order can differ from the standard single fold in the
//!   last bits.
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
//! source-teacher computation (f32 dot products, exp, division). They
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
    /// The f32 accumulation order of one score.
    #[serde(default)]
    pub score_accumulation: String,
}

impl AttentionOperatorParams {
    /// The declared parameters of `standard-source-attention/1`:
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
    pub fn standard() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: "sequential-f32-left-fold".to_owned(),
        }
    }

    /// The declared parameters of `experimental-r4-source-attention/1`
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
    pub fn experimental_r4() -> Self {
        Self {
            head_selection: "grouped-query-kv-head-h-div-kv-mul".to_owned(),
            score_scale: "divide-by-sqrt-full-head-size".to_owned(),
            score_width_policy: "chunks-of-4-floor-head-size-div-4".to_owned(),
            remainder_policy: "truncate-trailing-head-size-mod-4-dims-from-score".to_owned(),
            score_accumulation: "per-4-chunk-left-fold-then-chunk-sum".to_owned(),
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

    /// The declared parameters of `learned-absolute-source-attention/1`
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
    pub fn learned_absolute() -> Self {
        Self {
            head_selection: "multi-head-identity-kv-head-equals-query-head".to_owned(),
            score_scale: "multiply-by-reciprocal-sqrt-full-head-size".to_owned(),
            score_width_policy: "full-head-width".to_owned(),
            remainder_policy: "none-every-head-dimension-scored".to_owned(),
            score_accumulation: "sequential-f32-left-fold".to_owned(),
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
    /// Registry version of the standard operator currently implemented
    /// by [`standard_head_attention_weights`] +
    /// [`head_attention_value_aggregate`].
    pub const STANDARD_VERSION: u32 = 1;
    /// Registry id of the experimental `r4_attention`-gated operator,
    /// named for what it computes (a chunked dot product with the same
    /// softmax selector), not for the historical description.
    pub const EXPERIMENTAL_R4_ID: &'static str = "experimental-r4-source-attention";
    /// Registry version of the experimental operator currently
    /// implemented by [`experimental_r4_head_attention_weights`] +
    /// [`head_attention_value_aggregate`].
    pub const EXPERIMENTAL_R4_VERSION: u32 = 1;
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
    /// Registry version of the learned-absolute operator currently
    /// computed by the GPT-2 executor (`crate::gpt2::Gpt2Model`).
    pub const LEARNED_ABSOLUTE_VERSION: u32 = 1;

    /// The `standard-source-attention/1` record — the operator the
    /// default-off `r4_attention` switch has always computed.
    pub fn standard() -> Self {
        let mut record = Self {
            id: Self::STANDARD_ID.to_owned(),
            version: Self::STANDARD_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation: "scaled-dot-product".to_owned(),
            selector_normalization: "softmax-max-subtracted-exp-then-sum-normalize".to_owned(),
            value_aggregation: "position-ascending-weighted-sum-of-values".to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            params: AttentionOperatorParams::standard(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// The `experimental-r4-source-attention/1` record — the ACTUAL
    /// computation of the `r4_attention = true` branch: a 4-wide-chunked
    /// dot product (truncating the trailing `head_size mod 4`
    /// dimensions from the score) followed by the SAME max-subtracted
    /// softmax the standard operator uses.
    pub fn experimental_r4() -> Self {
        let mut record = Self {
            id: Self::EXPERIMENTAL_R4_ID.to_owned(),
            version: Self::EXPERIMENTAL_R4_VERSION,
            projections: "per-layer-dense-f32-wq-wk-wv".to_owned(),
            positional_action: "rope-rotation-of-q-and-k-before-scoring".to_owned(),
            compatibility_relation: "chunked-4-wide-dot-product".to_owned(),
            selector_normalization: "softmax-max-subtracted-exp-then-sum-normalize".to_owned(),
            value_aggregation: "position-ascending-weighted-sum-of-values".to_owned(),
            output_projection: "per-layer-dense-f32-wo".to_owned(),
            runtime_state: "growing-kv-cache-full-prefix".to_owned(),
            tie_breaking: "first-maximum-softmax-stabilizer-value-identical".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            params: AttentionOperatorParams::experimental_r4(),
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

    /// The `learned-absolute-source-attention/1` record (#668) — the
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
    pub fn learned_absolute_source_attention() -> Self {
        let mut record = Self {
            id: Self::LEARNED_ABSOLUTE_ID.to_owned(),
            version: Self::LEARNED_ABSOLUTE_VERSION,
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
            params: AttentionOperatorParams::learned_absolute(),
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
/// `false` has always computed `standard-source-attention/1`, `true` the
/// `experimental-r4-source-attention/1` branch. This is the one boundary
/// mapping from the legacy switch to the versioned operator identity.
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
        (AttentionOperatorSpec::STANDARD_ID, AttentionOperatorSpec::STANDARD_VERSION) => {
            Ok(AttentionOperatorSpec::standard())
        }
        (
            AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
            AttentionOperatorSpec::EXPERIMENTAL_R4_VERSION,
        ) => Ok(AttentionOperatorSpec::experimental_r4()),
        (AttentionOperatorSpec::R4_ROUTE_ID, AttentionOperatorSpec::R4_ROUTE_VERSION) => {
            Ok(AttentionOperatorSpec::r4_route_attention_v1())
        }
        (
            AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_VERSION,
        ) => Ok(AttentionOperatorSpec::learned_absolute_source_attention()),
        _ => Err(crate::SourceIngestKind::UnknownAttentionOperator {
            id: id.to_owned(),
            version,
        }
        .into()),
    }
}

/// The `standard-source-attention/1` weight computation, factored
/// verbatim out of `Llama::layer_forward`/`Llama::forward_batch` (#602):
/// fill `att` (one slot per cached position `t = 0..att.len()`) with the
/// softmax-normalized scaled dot products of `q` against the cached keys.
/// `keys` is the layer's key-cache region; position `t`'s key head starts
/// at `t * key_stride + key_offset` (grouped query: `key_offset =
/// (h / kv_mul) * head_size`). Iteration order and arithmetic — one
/// sequential f32 left fold per score over the FULL head width, one
/// divide by `sqrt(head_size)`, then the shared max-subtracted softmax —
/// are bit-identical to the pre-#602 in-line loop. Deterministic and
/// free of hidden state; `canonical` selects the D2 libm math path
/// exactly as the executor does.
pub fn standard_head_attention_weights(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
) {
    let head_size = q.len();
    for (t, attention) in att.iter_mut().enumerate() {
        let k = &keys[t * key_stride + key_offset..][..head_size];
        let mut score = 0.0f32;
        for i in 0..head_size {
            score += q[i] * k[i];
        }
        score /= crate::sqrtf(head_size as f32, canonical);
        *attention = score;
    }
    crate::softmax_with_mode(att, canonical);
}

/// The `experimental-r4-source-attention/1` weight computation, factored
/// verbatim out of the `r4_attention` branch (#602): the dot product is
/// computed in 4-wide chunks over dimensions `0..4*(head_size/4)` — the
/// trailing `head_size mod 4` q/k dimensions are never read — divided by
/// `sqrt(head_size)` over the FULL head width, then normalized by the
/// SAME max-subtracted softmax as the standard operator. For
/// `head_size < 4` every score is 0 and the weights are uniform.
/// Iteration order and arithmetic are bit-identical to the pre-#602
/// in-line loop. Still gated by `Config::r4_attention` at both call
/// sites; factoring changes no selection.
pub fn experimental_r4_head_attention_weights(
    att: &mut [f32],
    q: &[f32],
    keys: &[f32],
    key_offset: usize,
    key_stride: usize,
    canonical: bool,
) {
    let head_size = q.len();
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
        head_score /= crate::sqrtf(head_size as f32, canonical);
        *attention = head_score;
    }
    crate::softmax_with_mode(att, canonical);
}

/// The value aggregation both operators share, factored verbatim out of
/// the executor (#602): zero `out` (one head width), then accumulate the
/// position-ascending weighted sum `out[i] += att[t] * v_t[i]` over the
/// cached values. `values` is the layer's value-cache region; position
/// `t`'s value head starts at `t * value_stride + value_offset`. Every
/// head dimension participates regardless of the scoring operator's
/// width policy. Arithmetic order is bit-identical to the pre-#602
/// in-line loop.
pub fn head_attention_value_aggregate(
    out: &mut [f32],
    att: &[f32],
    values: &[f32],
    value_offset: usize,
    value_stride: usize,
) {
    let head_size = out.len();
    out.iter_mut().for_each(|v| *v = 0.0);
    for (t, &attention) in att.iter().enumerate() {
        let v = &values[t * value_stride + value_offset..][..head_size];
        for i in 0..head_size {
            out[i] += attention * v[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for i in 0..head_size {
            let mut expected = 0.0f32;
            for (t, &weight) in att.iter().enumerate() {
                expected += weight * values[t * value_stride + value_offset + i];
            }
            assert_eq!(out[i].to_bits(), expected.to_bits(), "dimension {i}");
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
    fn canonical_serialization_is_byte_stable() {
        // The canonical forms are pinned byte-for-byte: any drift in
        // field order, separators, or parameter tokens fails here — the
        // digest identity must not move silently.
        let standard = AttentionOperatorSpec::standard();
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
        let expected = format!(
            "blake3:{}",
            blake3::hash(pinned_standard.as_bytes()).to_hex()
        );
        assert_eq!(standard.implementation_digest, expected);
        assert_eq!(standard.declared_digest(), expected);

        let experimental = AttentionOperatorSpec::experimental_r4();
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
        assert_eq!(
            experimental.canonical_bytes(),
            pinned_experimental.as_bytes()
        );
        assert_eq!(
            experimental.declared_digest(),
            experimental.implementation_digest
        );
        assert_ne!(
            standard.implementation_digest, experimental.implementation_digest,
            "the two operators are distinct identities"
        );
        // Rebuilding reproduces the records bit-for-bit.
        assert_eq!(standard, AttentionOperatorSpec::standard());
        assert_eq!(experimental, AttentionOperatorSpec::experimental_r4());
    }

    #[test]
    fn record_round_trips_through_serde_json() {
        for record in [
            AttentionOperatorSpec::standard(),
            AttentionOperatorSpec::experimental_r4(),
            AttentionOperatorSpec::r4_route_attention_v1(),
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
        let standard = operator_spec(
            AttentionOperatorSpec::STANDARD_ID,
            AttentionOperatorSpec::STANDARD_VERSION,
        )
        .expect("registered standard operator");
        assert_eq!(standard, AttentionOperatorSpec::standard());
        let experimental = operator_spec(
            AttentionOperatorSpec::EXPERIMENTAL_R4_ID,
            AttentionOperatorSpec::EXPERIMENTAL_R4_VERSION,
        )
        .expect("registered experimental operator");
        assert_eq!(experimental, AttentionOperatorSpec::experimental_r4());
        let target = operator_spec(
            AttentionOperatorSpec::R4_ROUTE_ID,
            AttentionOperatorSpec::R4_ROUTE_VERSION,
        )
        .expect("registered target route operator");
        assert_eq!(target, AttentionOperatorSpec::r4_route_attention_v1());
        let learned = operator_spec(
            AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
            AttentionOperatorSpec::LEARNED_ABSOLUTE_VERSION,
        )
        .expect("registered learned-absolute operator");
        assert_eq!(
            learned,
            AttentionOperatorSpec::learned_absolute_source_attention()
        );
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
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(target.implementation_digest, expected);
        assert_eq!(target.declared_digest(), expected);
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
            AttentionOperatorSpec::standard().permitted_operation_class,
            "host-source-f32"
        );
        // Rebuilding reproduces the record bit-for-bit.
        assert_eq!(target, AttentionOperatorSpec::r4_route_attention_v1());
    }

    #[test]
    fn learned_absolute_canonical_serialization_is_byte_stable() {
        // #668: the GPT-2-family operator's declared identity, pinned
        // byte-for-byte. Any drift in a field token is a version bump,
        // never a silent edit.
        let learned = AttentionOperatorSpec::learned_absolute_source_attention();
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
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(learned.implementation_digest, expected);
        assert_eq!(learned.declared_digest(), expected);

        // It shares the standard operator's compatibility relation and
        // softmax selector, but is a DISTINCT identity: the positional
        // action and projections differ, so the declared digest differs.
        let standard = AttentionOperatorSpec::standard();
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
            AttentionOperatorSpec::experimental_r4().implementation_digest
        );
        // Rebuilding reproduces the record bit-for-bit.
        assert_eq!(
            learned,
            AttentionOperatorSpec::learned_absolute_source_attention()
        );
    }

    #[test]
    fn registry_refuses_unknown_id_and_version_by_name() {
        for (id, version) in [
            ("standard-source-attention", 2u32),
            ("experimental-r4-source-attention", 9),
            ("learned-absolute-source-attention", 2),
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
