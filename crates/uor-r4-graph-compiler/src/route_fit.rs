//! Offline `R4RouteAttentionV1` fitting harness (#605): the versioned
//! `route-fit/1` method record, the eight-identity fit manifest, the
//! production-boundary trace-corpus reader, the deterministic fit itself,
//! and the synthetic teacher fixture the cheap-instrument arm runs on.
//!
//! Everything here is COMPILER-SIDE (host) code. Compile-side float math
//! is permitted — P-4 binds only the deployed runtime kernel
//! (`uor-r4-graph-runtime::route_attention`, untouched by #605) — but it
//! is deterministic by construction: no map-iteration order reaches any
//! byte (`BTreeMap`/sorted vectors only), every f32 reduction folds in a
//! fixed left-to-right order, and every sort uses the `f32::total_cmp`
//! total order.
//!
//! The fitted operator stays DORMANT: `route-fit-dormant` in
//! `model/ledger.toml` (next to `r4-route-attention-dormant`), referenced
//! by no serving path. The replacement-ladder evaluator that consumes
//! this module's outputs lives in
//! `uor-r4-graph-certify::route_fit_report`, which also pins the
//! pre-registered #605 run contract as data.
//!
//! ## `route-fit/1` (versioned method record, #600 discipline)
//!
//! The fit method is a typed, versioned record with a canonical
//! pinned-line serialization and a blake3 declared-identity digest over
//! the parameter DECLARATION — never source text — exactly the
//! `bucket-average/1` pattern (`uor-r4-model-source::geometry`). A
//! behavioral change must arrive as a new registry version; the registry
//! ([`fit_method_spec`]) refuses every unknown `(id, version)` by name on
//! the sanctioned [`SourceUnavailable`] surface.
//!
//! Semantics (deterministic, ordered), per `(layer, head)`:
//!
//! 1. take each captured query/key hidden vector of the head (the head's
//!    slice of the #603 q/k lane rows);
//! 2. project it to the 288-bucket route-code width through the REAL
//!    #600 registry implementation (`bucket-average/1` via
//!    [`projection_implementation`]). `bucket-average/1` refuses a
//!    source narrower than the compiled width, so a head vector narrower
//!    than 288 is first expanded by cyclic tiling to the least multiple
//!    of its width at or above 288 — a declared, versioned parameter of
//!    this method (`expansion` below), not a post-hoc accommodation;
//! 3. binarize with a per-bit LOWER-MEDIAN threshold computed over the
//!    fit sample in fixed order (values sorted with `f32::total_cmp`;
//!    lower median = `sorted[(len - 1) / 2]`; bit set when
//!    `value >= threshold`);
//! 4. pack bits LSB-first within each byte into the 36-byte route code.
//!
//! Candidates' route codes are the key codes; query codes are the query
//! codes; queries and keys share one threshold table per head (one code
//! space, so the deployed masked XOR+popcount relation compares like
//! with like). Mask v1 is FULL (all 288 bits active — recorded, not
//! learned). Radii: absent. Residual/output projection: absent.
//! `top_m = min(ROUTE_MAX_TOP_M, trace support cap)`.
//!
//! ## Fit input boundary (production #603 surfaces)
//!
//! The fit consumes the REAL #603 trace corpus: the synthetic teacher
//! implements [`TeacherOracle`] (including
//! `step_with_trace_capture`) and the corpus is written by
//! [`observe_sharded_traced`](crate::observation::observe_sharded_traced)
//! under a registered `full/1` [`TraceProfile`] — the same shard records,
//! `.prob` sidecar, `.trace` sidecar, manifest pinning, and merge order
//! every production pass uses. [`load_route_trace_corpus`] reads it back
//! through [`merge_shards`], [`merge_probability_metadata`], and
//! [`merge_trace_rows`], then decodes rows by the documented #603 lane
//! layout. No bespoke side channel exists.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use crate::observation::{
    ObservationManifest, ObserveSummary, RECORD_SIZE, merge_probability_metadata, merge_shards,
    merge_trace_rows, observe_sharded_traced,
};
use crate::trace_profile::TraceProfile;
use uor_r4_core::transformerless::compiler::xorshift;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_graph_format::route_attention::ROUTE_MAX_TOP_M;
use uor_r4_graph_format::route_attention::{ROUTE_CODE_BITS, ROUTE_CODE_BYTES};
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::geometry::GeometryProjection;
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::geometry::projection_implementation;
use uor_r4_model_source::{
    BehaviorSource, RepresentationSource, TeacherOracle, TraceCaptureGeometry, TraceCaptureRequest,
    TraceCaptureSinks,
};

/// Format tag of the canonical `route-fit` method serialization.
pub const ROUTE_FIT_FORMAT: &str = "uor-r4-route-fit/1";
/// Registry id of the fit method implemented by [`fit_route_codes`].
pub const ROUTE_FIT_ID: &str = "route-fit";
/// Registry version. A behavioral change is a new version, never an
/// in-place edit (#600/#601/#602/#603 discipline).
pub const ROUTE_FIT_VERSION: u32 = 1;

/// Format tag of the canonical fit-manifest serialization.
pub const FIT_MANIFEST_FORMAT: &str = "uor-r4-route-fit-manifest/1";

/// Declared parameters of a route-fit method — stable machine tokens
/// that enter the canonical digest serialization byte-for-byte.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteFitParams {
    /// The vector→route-code projection, including the versioned #600
    /// registry implementation it calls.
    #[serde(default)]
    pub projection: String,
    /// The declared expansion applied before the projection when a head
    /// vector is narrower than the route-code width.
    #[serde(default)]
    pub expansion: String,
    /// The per-bit binarization threshold rule.
    #[serde(default)]
    pub threshold: String,
    /// The bit rule at the threshold.
    #[serde(default)]
    pub binarization: String,
    /// How bits pack into route-code bytes.
    #[serde(default)]
    pub bit_packing: String,
    /// The fixed order the fit sample is consumed in.
    #[serde(default)]
    pub sample_order: String,
    /// The declared relation mask (v1: full, not learned).
    #[serde(default)]
    pub mask: String,
    /// The declared radii (v1: absent).
    #[serde(default)]
    pub radii: String,
    /// The declared residual/output projection (v1: absent).
    #[serde(default)]
    pub residual_projection: String,
    /// The declared `top_m` rule.
    #[serde(default)]
    pub top_m_rule: String,
    /// The declared candidate rule (which codes become candidates).
    #[serde(default)]
    pub candidate_rule: String,
    /// The declared code space shared by queries and keys.
    #[serde(default)]
    pub code_space: String,
}

impl RouteFitParams {
    /// The declared parameters of `route-fit/1` (module docs).
    pub fn route_fit_v1() -> Self {
        Self {
            projection: "bucket-average/1-registry-to-288-buckets".to_owned(),
            expansion: "cyclic-tile-to-least-multiple-of-width-at-or-above-288-when-narrower"
                .to_owned(),
            threshold: "per-bit-lower-median-over-fit-sample-f32-total-cmp-sorted-index-(n-1)/2"
                .to_owned(),
            binarization: "bit-set-when-value-at-or-above-threshold".to_owned(),
            bit_packing: "lsb-first-within-byte".to_owned(),
            sample_order: "story-ascending-position-ascending-query-then-key".to_owned(),
            mask: "full".to_owned(),
            radii: "absent".to_owned(),
            residual_projection: "absent".to_owned(),
            top_m_rule: "min-route-max-top-m-and-trace-support-cap".to_owned(),
            candidate_rule: "prefix-key-codes-position-ascending-candidate-index-equals-position"
                .to_owned(),
            code_space: "shared-query-key-thresholds-per-layer-head".to_owned(),
        }
    }
}

/// The typed, versioned record of one route-fit method (#605), following
/// the #600 `GeometryProjection` pattern exactly: canonical pinned-line
/// bytes, blake3 declared-identity digest over the parameter declaration
/// (not source text), and a registry that refuses unknown pairs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteFitMethod {
    /// Registry id (`route-fit`).
    #[serde(default)]
    pub id: String,
    /// Registry version.
    #[serde(default)]
    pub version: u32,
    /// Route-code width in bits the method emits (always 288).
    #[serde(default)]
    pub code_bits: u32,
    /// The declared parameters.
    #[serde(default)]
    pub params: RouteFitParams,
    /// `blake3:<hex>` of [`RouteFitMethod::canonical_bytes`] — the
    /// declared identity, not source code text.
    #[serde(default)]
    pub declared_digest: String,
}

impl RouteFitMethod {
    /// The `route-fit/1` record implemented by [`fit_route_codes`].
    pub fn route_fit_v1() -> Self {
        let mut record = Self {
            id: ROUTE_FIT_ID.to_owned(),
            version: ROUTE_FIT_VERSION,
            code_bits: ROUTE_CODE_BITS as u32,
            params: RouteFitParams::route_fit_v1(),
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// Canonical serialization of the record's declared identity: a
    /// fixed line format, byte-stable by construction (field order and
    /// separators are fixed here, not derived from any serializer).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "{ROUTE_FIT_FORMAT}\n\
             id={}\n\
             version={}\n\
             code_bits={}\n\
             param.projection={}\n\
             param.expansion={}\n\
             param.threshold={}\n\
             param.binarization={}\n\
             param.bit_packing={}\n\
             param.sample_order={}\n\
             param.mask={}\n\
             param.radii={}\n\
             param.residual_projection={}\n\
             param.top_m_rule={}\n\
             param.candidate_rule={}\n\
             param.code_space={}\n",
            self.id,
            self.version,
            self.code_bits,
            self.params.projection,
            self.params.expansion,
            self.params.threshold,
            self.params.binarization,
            self.params.bit_packing,
            self.params.sample_order,
            self.params.mask,
            self.params.radii,
            self.params.residual_projection,
            self.params.top_m_rule,
            self.params.candidate_rule,
            self.params.code_space,
        )
        .into_bytes()
    }

    /// The declared-identity digest: `blake3:<hex>` over
    /// [`RouteFitMethod::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// The versioned fit-method registry (#605): map `(id, version)` to the
/// record. Every pair outside the registry is refused by name on the
/// sanctioned [`SourceUnavailable`] surface (the same type this crate's
/// #603 profile registry uses) — never guessed, never approximated by a
/// "closest" method or version.
#[cfg(not(target_arch = "wasm32"))]
pub fn fit_method_spec(id: &str, version: u32) -> Result<RouteFitMethod, SourceUnavailable> {
    match (id, version) {
        (ROUTE_FIT_ID, ROUTE_FIT_VERSION) => Ok(RouteFitMethod::route_fit_v1()),
        _ => Err(SourceUnavailable::new(format!(
            "unknown route-fit method ({id}, {version}); registered: \
             {ROUTE_FIT_ID}/{ROUTE_FIT_VERSION}"
        ))),
    }
}

/// One parameter of the fitted operator, labeled by provenance class:
/// `compiled` (learned/derived by the fit), `declared` (a fixed part of
/// the method declaration, not learned), `source` (taken from source
/// weights), or `absent` (does not exist in this version).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterProvenance {
    /// Parameter name.
    #[serde(default)]
    pub name: String,
    /// Provenance class.
    #[serde(default)]
    pub class: String,
}

/// The `route-fit/1` parameter labeling: route codes and thresholds are
/// COMPILED; the mask, contributions, and `top_m` are DECLARED (fixed by
/// the method, not learned); radii and residual projection are ABSENT;
/// no parameter is taken from source weights in v1.
pub fn route_fit_v1_parameter_labels() -> Vec<ParameterProvenance> {
    let label = |name: &str, class: &str| ParameterProvenance {
        name: name.to_owned(),
        class: class.to_owned(),
    };
    vec![
        label("route_codes", "compiled"),
        label("thresholds", "compiled"),
        label("mask", "declared"),
        label("contributions", "declared"),
        label("top_m", "declared"),
        label("radii", "absent"),
        label("residual_projection", "absent"),
        label("source_weights", "absent"),
    ]
}

/// The fit manifest (#605): the eight identity fields binding one fit —
/// source snapshot, tokenizer, adapter, trace, geometry, operator,
/// corpus, compiler — plus the typed method/geometry/profile/operator
/// records and the parameter-provenance labels. A genuinely absent
/// identity (e.g. the synthetic arm has no tokenizer) is a typed
/// `None`, carried honestly with an explicit `absent` marker in the
/// canonical bytes — never an empty string pretending to be a κ.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FitManifest {
    /// [`FIT_MANIFEST_FORMAT`].
    #[serde(default)]
    pub format: String,
    /// The versioned fit-method record.
    #[serde(default)]
    pub method: RouteFitMethod,
    /// #600 typed geometry record of the projection the method calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryProjection>,
    /// #603 typed trace-profile record the fit input was captured under.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_profile: Option<TraceProfile>,
    /// #602/#604 typed record of the TARGET operator being fitted
    /// (`r4-route-attention/1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<AttentionOperatorSpec>,
    /// Identity 1/8: κ of the teacher source snapshot
    /// ([`TeacherOracle::kappa`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_snapshot: Option<String>,
    /// Identity 2/8: tokenizer identity. `None` on the synthetic arm —
    /// the synthetic teacher consumes raw token ids and no tokenizer
    /// exists to identify.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// Identity 3/8: adapter identity (which executor implementation
    /// produced the traces).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    /// Identity 4/8: κ of the merged #603 trace-sidecar bytes the fit
    /// read (the fit input data identity).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
    /// Identity 5/8: declared-identity digest of the #600 geometry
    /// record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry_identity: Option<String>,
    /// Identity 6/8: declared-identity digest of the target operator
    /// record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_identity: Option<String>,
    /// Identity 7/8: κ of the merged observation-record bytes (the
    /// mini-corpus identity). The REAL #531 saturation-corpus identity
    /// belongs to the real arm and is genuinely absent here until that
    /// corpus exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus: Option<String>,
    /// Identity 8/8: the fitting compiler identity (crate + version).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    /// Every fitted-operator parameter labeled by provenance class.
    #[serde(default)]
    pub parameters: Vec<ParameterProvenance>,
}

impl FitManifest {
    /// Canonical serialization: a fixed line format with EXPLICIT
    /// absence markers per identity field (`<name>=absent` /
    /// `<name>=present:<value>`), the #603 identity-bundle discipline —
    /// absence is part of the digest input and distinct from any empty
    /// value.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        fn line(text: &mut String, name: &str, value: &Option<String>) {
            match value {
                None => text.push_str(&format!("{name}=absent\n")),
                Some(value) => text.push_str(&format!("{name}=present:{value}\n")),
            }
        }
        let mut text = format!(
            "{FIT_MANIFEST_FORMAT}\nmethod={}\n",
            self.method.declared_digest()
        );
        line(&mut text, "source_snapshot", &self.source_snapshot);
        line(&mut text, "tokenizer", &self.tokenizer);
        line(&mut text, "adapter", &self.adapter);
        line(&mut text, "trace", &self.trace);
        line(&mut text, "geometry", &self.geometry_identity);
        line(&mut text, "operator", &self.operator_identity);
        line(&mut text, "corpus", &self.corpus);
        line(&mut text, "compiler", &self.compiler);
        line(
            &mut text,
            "trace_profile",
            &self
                .trace_profile
                .as_ref()
                .map(TraceProfile::declared_digest),
        );
        for parameter in &self.parameters {
            text.push_str(&format!(
                "parameter.{}={}\n",
                parameter.name, parameter.class
            ));
        }
        text.into_bytes()
    }

    /// The manifest κ: `blake3:<hex>` over
    /// [`FitManifest::canonical_bytes`].
    pub fn kappa(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

// ---------------------------------------------------------------------------
// Synthetic teacher fixture (deterministic; #605 cheap-instrument arm).
// ---------------------------------------------------------------------------

/// Vocabulary of the synthetic teacher (bound: task caps vocab at 64).
pub const SYNTH_VOCAB: usize = 64;
/// Model width.
pub const SYNTH_DIM: usize = 32;
/// Transformer layers.
pub const SYNTH_LAYERS: usize = 2;
/// Attention heads per layer.
pub const SYNTH_HEADS: usize = 2;
/// Key/value heads (no grouping: kv == heads).
pub const SYNTH_KV_HEADS: usize = 2;
/// Sequence length — deliberately equal to `ROUTE_MAX_CANDIDATES` so a
/// prefix candidate table always fits the deployed instance bound.
pub const SYNTH_SEQ_LEN: usize = 64;
/// MLP hidden width.
pub const SYNTH_MLP: usize = 64;
/// Mini-corpus target in tokens (bound: task caps the corpus at 4096).
pub const SYNTH_CORPUS_TOKENS: usize = 1024;
/// Shard fan-out bits of the synthetic trace corpus.
pub const SYNTH_SHARD_BITS: u8 = 2;
/// Per-head attention-support cap of the synthetic trace profile —
/// matches `ROUTE_MAX_TOP_M`, so `top_m = min(8, 8) = 8`.
pub const SYNTH_SUPPORT_SIZE: u32 = 8;
/// Attention score gain over the per-head COSINE similarity (sharpens
/// the softmax so the top-8 support carries most of the mass; a
/// declared constant of the fixture).
pub const SYNTH_SCORE_GAIN: f32 = 10.0;
/// Logit gain (a declared constant of the fixture; 1.0 keeps sampling
/// diverse enough that stories do not collapse into repeated tokens).
pub const SYNTH_LOGIT_GAIN: f32 = 1.0;
/// Weight-generation seed (integer-seeded xorshift — the repo's existing
/// PRNG helper `compiler::xorshift`; no rand dependency, no clock).
pub const SYNTH_WEIGHT_SEED: u64 = 0x6050_0001;

const SYNTH_HEAD_DIM: usize = SYNTH_DIM / SYNTH_HEADS;
const SYNTH_RMS_EPS: f32 = 1e-5;

/// The #603 trace profile the synthetic arm captures under: `full/1`
/// over both layers with the pinned support cap (the fit needs the q/k
/// lane and the attention-support lane; `full/1` is the registered
/// profile carrying both).
pub fn synthetic_trace_profile() -> TraceProfile {
    TraceProfile::full(&[0, 1], SYNTH_SUPPORT_SIZE)
}

/// The synthetic teacher's #603 capture geometry.
pub fn synthetic_capture_geometry() -> TraceCaptureGeometry {
    TraceCaptureGeometry {
        layers: SYNTH_LAYERS,
        heads: SYNTH_HEADS,
        kv_heads: SYNTH_KV_HEADS,
        residual_width: SYNTH_DIM,
    }
}

/// A tiny deterministic transformer-like teacher (#605 fixture): 2
/// pre-norm layers of multi-head softmax attention plus a SiLU MLP over
/// a 32-wide residual stream, tied-embedding logits, integer-seeded
/// weights, NO clock and NO ambient randomness anywhere. The query and
/// key projections are SHARED (`W_k = W_q`) and per-head L2-normalized
/// (QK-norm — scaled COSINE attention), making every head a
/// content-similarity head whose support is query-DIRECTION-specific —
/// the regime a route-code fit is supposed to recover.
///
/// Instrument-construction record (#605, honesty over completeness):
/// the first fixture iteration used unnormalized dot-product scores;
/// its N2 (shifted-support) null measured 0.527 against a fitted
/// overlap of 0.718 — large-norm "hub" keys made supports largely
/// query-independent and the pre-registered anti-vacuity rule correctly
/// declared that instrument VACUOUS. The QK-normalization below removes
/// the norm-hub channel so the instrument can discriminate; the
/// pre-registered gates, nulls, and margins were NOT touched.
///
/// The teacher implements the full [`TeacherOracle`] trace surface, so
/// the REAL #603 pipeline (`observe_sharded_traced`) captures its
/// traces; nothing here writes a bespoke trace format.
#[derive(Debug, Clone)]
pub struct SyntheticRouteTeacher {
    embed: Vec<f32>,
    wq: Vec<f32>,
    wv: Vec<f32>,
    wo: Vec<f32>,
    w1: Vec<f32>,
    w2: Vec<f32>,
    key_cache: Vec<f32>,
    value_cache: Vec<f32>,
    hidden: Vec<f32>,
    logits_state: Vec<f32>,
    kappa: String,
}

/// The support restriction one replaced head applies at one position:
/// the fitted-selected candidate positions (ascending selection order is
/// not required here; membership is what restricts).
pub type AllowedPositions = Vec<u32>;

/// Per-head, per-position support restrictions for ONE story: the
/// replacement plan of `support-restrict-renormalize/1`. Keyed by
/// `(layer, head)` (a sorted map — no iteration-order leak), each entry
/// holds one allowed-position set per story position.
pub type StoryRestrictionPlan = BTreeMap<(u32, u32), Vec<AllowedPositions>>;

fn seeded_uniform(seed: &mut u64, count: usize, scale: f32) -> Vec<f32> {
    (0..count)
        .map(|_| {
            // Same bit extraction as the corpus sampler: top 24 bits of a
            // xorshift draw, mapped to [-scale, scale).
            let unit = (xorshift(seed) >> 40) as f32 / (1u64 << 24) as f32;
            (unit * 2.0 - 1.0) * scale
        })
        .collect()
}

fn rmsnorm(input: &[f32], output: &mut [f32]) {
    let mut sum = 0.0f32;
    for &value in input {
        sum += value * value;
    }
    let inv = 1.0 / (sum / input.len() as f32 + SYNTH_RMS_EPS).sqrt();
    for (out, &value) in output.iter_mut().zip(input) {
        *out = value * inv;
    }
}

fn matvec(matrix: &[f32], input: &[f32], output: &mut [f32]) {
    // Fixed fold order: rows outer, columns left-to-right.
    let cols = input.len();
    for (row, out) in output.iter_mut().enumerate() {
        let mut acc = 0.0f32;
        for (column, &value) in input.iter().enumerate() {
            acc += matrix[row * cols + column] * value;
        }
        *out = acc;
    }
}

impl Default for SyntheticRouteTeacher {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntheticRouteTeacher {
    /// Build the teacher from the pinned integer seed. Deterministic:
    /// two constructions are identical, weights and κ included.
    pub fn new() -> Self {
        let mut seed = SYNTH_WEIGHT_SEED;
        let scale_embed = 0.35f32;
        let scale_proj = 1.0 / (SYNTH_DIM as f32).sqrt();
        let embed = seeded_uniform(&mut seed, SYNTH_VOCAB * SYNTH_DIM, scale_embed);
        let wq = seeded_uniform(&mut seed, SYNTH_LAYERS * SYNTH_DIM * SYNTH_DIM, scale_proj);
        let wv = seeded_uniform(&mut seed, SYNTH_LAYERS * SYNTH_DIM * SYNTH_DIM, scale_proj);
        let wo = seeded_uniform(&mut seed, SYNTH_LAYERS * SYNTH_DIM * SYNTH_DIM, scale_proj);
        let w1 = seeded_uniform(&mut seed, SYNTH_LAYERS * SYNTH_MLP * SYNTH_DIM, scale_proj);
        let w2 = seeded_uniform(
            &mut seed,
            SYNTH_LAYERS * SYNTH_DIM * SYNTH_MLP,
            1.0 / (SYNTH_MLP as f32).sqrt(),
        );
        // A genuine content κ of the synthetic source: blake3 over the
        // little-endian bytes of every weight tensor in declaration order.
        let mut hasher = blake3::Hasher::new();
        for tensor in [&embed, &wq, &wv, &wo, &w1, &w2] {
            for value in tensor.iter() {
                hasher.update(&value.to_le_bytes());
            }
        }
        let kappa = format!("blake3:{}", hasher.finalize().to_hex());
        Self {
            embed,
            wq,
            wv,
            wo,
            w1,
            w2,
            key_cache: vec![0.0; SYNTH_LAYERS * SYNTH_SEQ_LEN * SYNTH_DIM],
            value_cache: vec![0.0; SYNTH_LAYERS * SYNTH_SEQ_LEN * SYNTH_DIM],
            hidden: vec![0.0; SYNTH_DIM],
            logits_state: vec![0.0; SYNTH_VOCAB],
            kappa,
        }
    }

    /// One forward step. `capture` taps the declared #603 lanes through
    /// this exact executor path (a traced step and a plain step produce
    /// identical bits — the capture only copies out). `restrict` applies
    /// the `support-restrict-renormalize/1` replacement at the replaced
    /// heads: the head's OWN softmax weights, restricted to the allowed
    /// positions and renormalized (uniform over the allowed set in the
    /// measure-zero case that the restricted mass is not positive).
    fn forward_step(
        &mut self,
        token: usize,
        pos: usize,
        mut capture: Option<(&TraceCaptureRequest<'_>, &mut TraceCaptureSinks<'_, '_>)>,
        restrict: Option<&StoryRestrictionPlan>,
    ) {
        assert!(token < SYNTH_VOCAB, "token id outside the vocabulary");
        assert!(pos < SYNTH_SEQ_LEN, "position outside the sequence bound");
        let mut x = self.embed[token * SYNTH_DIM..(token + 1) * SYNTH_DIM].to_vec();
        let mut xn = vec![0.0f32; SYNTH_DIM];
        let mut q = vec![0.0f32; SYNTH_DIM];
        let mut attn_out = vec![0.0f32; SYNTH_DIM];
        let mut proj = vec![0.0f32; SYNTH_DIM];
        let mut mlp_hidden = vec![0.0f32; SYNTH_MLP];
        let mut mlp_out = vec![0.0f32; SYNTH_DIM];
        for layer in 0..SYNTH_LAYERS {
            let wq = &self.wq[layer * SYNTH_DIM * SYNTH_DIM..(layer + 1) * SYNTH_DIM * SYNTH_DIM];
            let wv = &self.wv[layer * SYNTH_DIM * SYNTH_DIM..(layer + 1) * SYNTH_DIM * SYNTH_DIM];
            let wo = &self.wo[layer * SYNTH_DIM * SYNTH_DIM..(layer + 1) * SYNTH_DIM * SYNTH_DIM];
            rmsnorm(&x, &mut xn);
            matvec(wq, &xn, &mut q);
            // QK-norm: each head's q slice is L2-normalized, so the
            // score below is a scaled cosine and no key can become a
            // norm hub. W_k = W_q (shared): the key row IS the (unit)
            // query projection of this position's normalized content —
            // a directional similarity head.
            for head in 0..SYNTH_HEADS {
                let slice = &mut q[head * SYNTH_HEAD_DIM..(head + 1) * SYNTH_HEAD_DIM];
                let mut norm_sq = 0.0f32;
                for &value in slice.iter() {
                    norm_sq += value * value;
                }
                let inv = 1.0 / (norm_sq + SYNTH_RMS_EPS).sqrt();
                for value in slice.iter_mut() {
                    *value *= inv;
                }
            }
            let cache_at =
                |cache_pos: usize| layer * SYNTH_SEQ_LEN * SYNTH_DIM + cache_pos * SYNTH_DIM;
            let key_row = cache_at(pos);
            self.key_cache[key_row..key_row + SYNTH_DIM].copy_from_slice(&q);
            {
                let mut value_row = vec![0.0f32; SYNTH_DIM];
                matvec(wv, &xn, &mut value_row);
                self.value_cache[key_row..key_row + SYNTH_DIM].copy_from_slice(&value_row);
            }
            if let Some((request, sinks)) = capture.as_mut()
                && request.qkv_layers.contains(&layer)
            {
                let k_row = &self.key_cache[key_row..key_row + SYNTH_DIM];
                let v_row = &self.value_cache[key_row..key_row + SYNTH_DIM];
                (sinks.qkv)(layer, &q, k_row, v_row);
            }
            attn_out.fill(0.0);
            for head in 0..SYNTH_HEADS {
                let head_start = head * SYNTH_HEAD_DIM;
                // Scores over the causal prefix, fixed order: scaled
                // cosine (q and cached k are unit per-head vectors).
                let mut weights = vec![0.0f32; pos + 1];
                let scale = SYNTH_SCORE_GAIN;
                for (t, weight) in weights.iter_mut().enumerate() {
                    let key = &self.key_cache[cache_at(t) + head_start..][..SYNTH_HEAD_DIM];
                    let mut score = 0.0f32;
                    for (qi, ki) in q[head_start..head_start + SYNTH_HEAD_DIM].iter().zip(key) {
                        score += qi * ki;
                    }
                    *weight = score * scale;
                }
                // Max-subtracted softmax, sequential fold order.
                let mut max = f32::NEG_INFINITY;
                for &weight in &weights {
                    if weight > max {
                        max = weight;
                    }
                }
                let mut sum = 0.0f32;
                for weight in weights.iter_mut() {
                    *weight = (*weight - max).exp();
                    sum += *weight;
                }
                for weight in weights.iter_mut() {
                    *weight /= sum;
                }
                if let Some((request, sinks)) = capture.as_mut()
                    && request.attention_layers.contains(&layer)
                {
                    (sinks.attention)(layer, head, &weights);
                }
                // support-restrict-renormalize/1: restrict the head's own
                // weights to the fitted selection and renormalize.
                if let Some(plan) = restrict
                    && let Some(per_pos) = plan.get(&(layer as u32, head as u32))
                {
                    let allowed = &per_pos[pos];
                    let mut restricted_sum = 0.0f32;
                    for (t, weight) in weights.iter_mut().enumerate() {
                        if allowed.contains(&(t as u32)) {
                            restricted_sum += *weight;
                        } else {
                            *weight = 0.0;
                        }
                    }
                    if restricted_sum > 0.0 {
                        for weight in weights.iter_mut() {
                            *weight /= restricted_sum;
                        }
                    } else {
                        // Degenerate renormalization rule (declared in
                        // the run contract): uniform over the allowed
                        // set when the restricted mass is not positive.
                        let uniform = 1.0 / allowed.len().max(1) as f32;
                        for (t, weight) in weights.iter_mut().enumerate() {
                            *weight = if allowed.contains(&(t as u32)) {
                                uniform
                            } else {
                                0.0
                            };
                        }
                    }
                }
                // Weighted value aggregation, position-ascending.
                for (t, &weight) in weights.iter().enumerate() {
                    let value = &self.value_cache[cache_at(t) + head_start..][..SYNTH_HEAD_DIM];
                    for (out, &vi) in attn_out[head_start..head_start + SYNTH_HEAD_DIM]
                        .iter_mut()
                        .zip(value)
                    {
                        *out += weight * vi;
                    }
                }
            }
            matvec(wo, &attn_out, &mut proj);
            for (xi, &pi) in x.iter_mut().zip(proj.iter()) {
                *xi += pi;
            }
            // MLP block: rmsnorm → W1 → SiLU → W2 → residual add.
            rmsnorm(&x, &mut xn);
            let w1 = &self.w1[layer * SYNTH_MLP * SYNTH_DIM..(layer + 1) * SYNTH_MLP * SYNTH_DIM];
            let w2 = &self.w2[layer * SYNTH_DIM * SYNTH_MLP..(layer + 1) * SYNTH_DIM * SYNTH_MLP];
            matvec(w1, &xn, &mut mlp_hidden);
            for value in mlp_hidden.iter_mut() {
                *value *= 1.0 / (1.0 + (-*value).exp());
            }
            matvec(w2, &mlp_hidden, &mut mlp_out);
            for (xi, &mi) in x.iter_mut().zip(mlp_out.iter()) {
                *xi += mi;
            }
            if let Some((request, sinks)) = capture.as_mut()
                && request.residual_layers.contains(&layer)
            {
                (sinks.residual)(layer, &x);
            }
        }
        rmsnorm(&x, &mut self.hidden);
        for (v, logit) in self.logits_state.iter_mut().enumerate() {
            let row = &self.embed[v * SYNTH_DIM..(v + 1) * SYNTH_DIM];
            let mut acc = 0.0f32;
            for (&ei, &hi) in row.iter().zip(self.hidden.iter()) {
                acc += ei * hi;
            }
            *logit = acc * SYNTH_LOGIT_GAIN;
        }
    }

    /// Teacher-force one token sequence and return the per-position raw
    /// logits, with the `support-restrict-renormalize/1` replacement
    /// applied at every `(layer, head)` the plan names (an empty plan is
    /// exactly the teacher forward). Resets state first.
    pub fn teacher_forced_logits(
        &mut self,
        tokens: &[u32],
        plan: &StoryRestrictionPlan,
    ) -> Vec<Vec<f32>> {
        self.reset();
        let mut per_position = Vec::with_capacity(tokens.len());
        for (pos, &token) in tokens.iter().enumerate() {
            self.forward_step(token as usize, pos, None, Some(plan));
            per_position.push(self.logits_state.clone());
        }
        per_position
    }
}

impl RepresentationSource for SyntheticRouteTeacher {
    fn vocab_size(&self) -> usize {
        SYNTH_VOCAB
    }
    fn source_dimension(&self) -> usize {
        SYNTH_DIM
    }
    fn tokenizer_address(&self) -> &str {
        // The synthetic teacher consumes raw token ids; no tokenizer
        // exists. The fit manifest carries tokenizer identity as a typed
        // absent, never this placeholder string.
        "synthetic-route-teacher-token-ids"
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        let count = range.end - range.start;
        if output.len() < count * SYNTH_DIM || range.end > SYNTH_VOCAB {
            return None;
        }
        output[..count * SYNTH_DIM]
            .copy_from_slice(&self.embed[range.start * SYNTH_DIM..range.end * SYNTH_DIM]);
        Some(())
    }
}

impl BehaviorSource for SyntheticRouteTeacher {
    fn reset(&mut self) {
        self.key_cache.fill(0.0);
        self.value_cache.fill(0.0);
        self.hidden.fill(0.0);
        self.logits_state.fill(0.0);
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.forward_step(token, pos, None, None);
        logits.copy_from_slice(&self.logits_state);
    }
}

impl TeacherOracle for SyntheticRouteTeacher {
    fn vocab(&self) -> usize {
        SYNTH_VOCAB
    }
    fn dim(&self) -> usize {
        SYNTH_DIM
    }
    fn seq_len(&self) -> usize {
        SYNTH_SEQ_LEN
    }
    fn kappa(&self) -> String {
        self.kappa.clone()
    }
    fn source_bytes(&self) -> usize {
        (self.embed.len()
            + self.wq.len()
            + self.wv.len()
            + self.wo.len()
            + self.w1.len()
            + self.w2.len())
            * core::mem::size_of::<f32>()
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        out.copy_from_slice(&self.embed[token * SYNTH_DIM..(token + 1) * SYNTH_DIM]);
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        Some(&self.hidden)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        Some(synthetic_capture_geometry())
    }
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        self.forward_step(token, pos, Some((request, sinks)), None);
        logits.copy_from_slice(&self.logits_state);
        true
    }
}

/// Generate the synthetic #605 trace mini-corpus into `dir` through the
/// PRODUCTION #603 pipeline: [`observe_sharded_traced`] over the
/// synthetic teacher under the pinned `full/1` profile. Deterministic —
/// two runs produce byte-identical shards, sidecars, and manifests
/// (pinned seed 0x5EED teacher stream, pinned weights, no clock).
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_synthetic_route_trace(dir: &Path) -> Result<ObserveSummary, SourceUnavailable> {
    let mut teacher = SyntheticRouteTeacher::new();
    let profile = synthetic_trace_profile();
    observe_sharded_traced(
        &mut teacher,
        600,
        SYNTH_CORPUS_TOKENS,
        SYNTH_SHARD_BITS,
        dir,
        None,
        &profile,
    )
}

// ---------------------------------------------------------------------------
// Trace-corpus reading (the production #603 boundary).
// ---------------------------------------------------------------------------

/// One teacher-forced step of one story, decoded from the production
/// record + sidecar formats.
#[derive(Debug, Clone, PartialEq)]
pub struct StepTrace {
    /// Position within the story.
    pub pos: u32,
    /// The token FED at this position (BOS at 0, else the previous
    /// step's sampled target).
    pub input_token: u32,
    /// The sampled target of this position (the teacher-forcing label).
    pub next: u32,
    /// The teacher's recorded top-8 next-token ids (descending
    /// probability, stable ties).
    pub top_tokens: [u32; 8],
    /// The teacher's natural-log probability of `next` (`.prob`
    /// sidecar).
    pub target_logprob_nats: f32,
    /// Captured q rows, one per declared layer (ascending), each
    /// `residual_width` wide.
    pub q_rows: Vec<Vec<f32>>,
    /// Captured k cache rows at this position, one per declared layer.
    pub k_rows: Vec<Vec<f32>>,
    /// Captured per-head attention support:
    /// `[declared layer][head] -> (position, weight)` entries in
    /// descending-weight order, absent slots already stripped (the
    /// explicit `SUPPORT_ABSENT_MARKER` is decoded as absence, never as
    /// a zero entry).
    pub supports: Vec<Vec<Vec<(u32, f32)>>>,
}

/// One reconstructed story: the fed token sequence and its steps.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryTrace {
    /// Story id from the record stream.
    pub story: u32,
    /// Fed tokens, position order (`tokens[pos]` is the step input).
    pub tokens: Vec<u32>,
    /// Steps, position order.
    pub steps: Vec<StepTrace>,
}

/// The decoded #605 fit input: every story of a #603 trace corpus with
/// the lanes the fit consumes, plus the identities that go into the fit
/// manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteTraceCorpus {
    /// Capture geometry the rows were decoded against.
    pub geometry: TraceCaptureGeometry,
    /// Declared capture layers (ascending).
    pub declared_layers: Vec<u32>,
    /// Declared per-head support cap.
    pub support_size: u32,
    /// The trace profile the corpus was captured under.
    pub trace_profile: TraceProfile,
    /// Stories ascending by story id.
    pub stories: Vec<StoryTrace>,
    /// Total records.
    pub records: usize,
    /// κ of the merged observation-record bytes.
    pub records_kappa: String,
    /// κ of the merged trace-sidecar bytes.
    pub trace_kappa: String,
    /// The #603 identity-bundle digest of the observation manifest.
    pub identity_bundle_digest: String,
}

#[cfg(not(target_arch = "wasm32"))]
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Load a #603 trace corpus written under a q/k + attention-support
/// profile (`full/1`) back into the typed fit input. `bos_token` is the
/// teacher's BOS id (the generation pipeline feeds it at position 0 of
/// every story; records carry sampled targets, so the fed sequence is
/// reconstructed as `[bos, next_0, next_1, ...]`).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_route_trace_corpus(
    dir: &Path,
    geometry: TraceCaptureGeometry,
    bos_token: u32,
) -> Result<RouteTraceCorpus, SourceUnavailable> {
    let manifest = ObservationManifest::load(dir)?.ok_or_else(|| {
        SourceUnavailable::new(format!("no observation manifest in {}", dir.display()))
    })?;
    let profile = manifest.trace_profile.clone().ok_or_else(|| {
        SourceUnavailable::new(format!(
            "{} was captured under the minimal profile; the route fit needs \
             the q/k and attention-support lanes",
            dir.display()
        ))
    })?;
    let qkv_lane = profile.qkv_lane.clone().ok_or_else(|| {
        SourceUnavailable::new("trace profile declares no q/k lane; the route fit needs it")
    })?;
    let support_lane = profile.attention_support_lane.clone().ok_or_else(|| {
        SourceUnavailable::new(
            "trace profile declares no attention-support lane; the route fit needs it",
        )
    })?;
    let declared_layers = qkv_lane.layer_indices.clone();
    if support_lane.layer_indices != declared_layers {
        return Err(SourceUnavailable::new(
            "q/k and attention-support lanes declare different layer lists; \
             the route fit reads one aligned layer list",
        ));
    }
    // One shared row layout (#603 lane order), derived beside the writer,
    // so the fit's reader cannot drift from the assembler (#645).
    let layout = crate::observation::TraceRowLayout::new(&profile, &geometry);
    let row_bytes = layout.row_bytes;
    if manifest.trace_row_bytes != Some(row_bytes as u64) {
        return Err(SourceUnavailable::new(format!(
            "trace row width mismatch: manifest pins {:?}, geometry + profile imply {row_bytes}",
            manifest.trace_row_bytes
        )));
    }

    let records_bytes = merge_shards(dir)?;
    let probabilities = merge_probability_metadata(dir)?;
    let trace_bytes = merge_trace_rows(dir)?;
    let records = records_bytes.len() / RECORD_SIZE;
    if probabilities.len() != records || trace_bytes.len() != records * row_bytes {
        return Err(SourceUnavailable::new(format!(
            "misaligned corpus: {records} records, {} probability rows, {} trace bytes \
             ({} per row expected)",
            probabilities.len(),
            trace_bytes.len(),
            row_bytes
        )));
    }
    let records_kappa = format!("blake3:{}", blake3::hash(&records_bytes).to_hex());
    let trace_kappa = format!("blake3:{}", blake3::hash(&trace_bytes).to_hex());

    let mut by_story: BTreeMap<u32, Vec<StepTrace>> = BTreeMap::new();
    for index in 0..records {
        let record = &records_bytes[index * RECORD_SIZE..(index + 1) * RECORD_SIZE];
        let story = read_u32(record, 0);
        let next = read_u32(record, 4);
        let mut top_tokens = [0u32; 8];
        for (slot, token) in top_tokens.iter_mut().enumerate() {
            *token = read_u32(record, 8 + slot * 4);
        }
        let pos = read_u32(record, 72); // span_start

        let row = &trace_bytes[index * row_bytes..(index + 1) * row_bytes];
        let decoded = layout.read_row(row)?;
        // The fit consumes the q and k lanes and the attention support; the
        // residual, final-hidden, and v lanes are captured but unused here.
        let q_rows: Vec<Vec<f32>> = decoded.qkv.iter().map(|(q, _, _)| q.clone()).collect();
        let k_rows: Vec<Vec<f32>> = decoded.qkv.iter().map(|(_, k, _)| k.clone()).collect();
        let supports = decoded.support;
        by_story.entry(story).or_default().push(StepTrace {
            pos,
            input_token: 0, // filled after position sort
            next,
            top_tokens,
            target_logprob_nats: probabilities[index].target_logprob_nats,
            q_rows,
            k_rows,
            supports,
        });
    }

    let mut stories = Vec::with_capacity(by_story.len());
    for (story, mut steps) in by_story {
        steps.sort_by_key(|step| step.pos);
        for (expected, step) in steps.iter().enumerate() {
            if step.pos as usize != expected {
                return Err(SourceUnavailable::new(format!(
                    "story {story} has non-contiguous positions (expected {expected}, \
                     found {})",
                    step.pos
                )));
            }
        }
        let mut tokens = Vec::with_capacity(steps.len());
        let mut fed = bos_token;
        for step in steps.iter_mut() {
            step.input_token = fed;
            tokens.push(fed);
            fed = step.next;
        }
        stories.push(StoryTrace {
            story,
            tokens,
            steps,
        });
    }

    Ok(RouteTraceCorpus {
        geometry,
        declared_layers,
        support_size: support_lane.support_size,
        trace_profile: profile,
        stories,
        records,
        records_kappa,
        trace_kappa,
        identity_bundle_digest: manifest.identity_bundle_digest(),
    })
}

// ---------------------------------------------------------------------------
// The fit itself.
// ---------------------------------------------------------------------------

/// The fitted parameters of one `(layer, head)`: the shared per-bit
/// thresholds and the query/key route codes of every `(story, pos)`.
#[derive(Debug, Clone, PartialEq)]
pub struct HeadCodes {
    /// Source layer index.
    pub layer: u32,
    /// Head index within the layer.
    pub head: u32,
    /// Per-bit lower-median thresholds (route-code width).
    pub thresholds: Vec<f32>,
    /// Query route codes, `[story index][pos]`.
    pub query_codes: Vec<Vec<[u8; ROUTE_CODE_BYTES]>>,
    /// Key route codes, `[story index][pos]` (candidate index equals
    /// position by the declared candidate rule).
    pub key_codes: Vec<Vec<[u8; ROUTE_CODE_BYTES]>>,
}

/// The fitted `route-fit/1` artifact: the method record, the declared
/// selection width, and the per-head codes in fixed (layer, head)
/// order.
#[derive(Debug, Clone, PartialEq)]
pub struct FittedRouteCodes {
    /// The versioned method that produced these parameters.
    pub method: RouteFitMethod,
    /// `min(ROUTE_MAX_TOP_M, trace support cap)`.
    pub top_m: u32,
    /// Per-head fitted parameters, layer ascending then head ascending.
    pub heads: Vec<HeadCodes>,
}

impl FittedRouteCodes {
    /// Canonical bytes of the fitted parameters: the method digest, the
    /// declared width, then every head's thresholds (f32 LE bits) and
    /// codes in fixed order. Deterministic double-run fits are
    /// byte-identical here.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"uor-r4-route-fit-params/1\n");
        bytes.extend_from_slice(self.method.declared_digest().as_bytes());
        bytes.push(b'\n');
        bytes.extend_from_slice(&self.top_m.to_le_bytes());
        for head in &self.heads {
            bytes.extend_from_slice(&head.layer.to_le_bytes());
            bytes.extend_from_slice(&head.head.to_le_bytes());
            for threshold in &head.thresholds {
                bytes.extend_from_slice(&threshold.to_le_bytes());
            }
            for story in &head.query_codes {
                for code in story {
                    bytes.extend_from_slice(code);
                }
            }
            for story in &head.key_codes {
                for code in story {
                    bytes.extend_from_slice(code);
                }
            }
        }
        bytes
    }

    /// κ of the fitted parameters.
    pub fn kappa(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }

    /// The fitted parameters of `(layer, head)`, if fitted.
    pub fn head(&self, layer: u32, head: u32) -> Option<&HeadCodes> {
        self.heads
            .iter()
            .find(|codes| codes.layer == layer && codes.head == head)
    }
}

/// Project one head vector to the route-code width through the REAL
/// #600 registry implementation (`bucket-average/1`). Vectors narrower
/// than the route-code width are first expanded by the declared cyclic
/// tiling (`RouteFitParams::expansion`); `bucket-average/1` itself
/// refuses a source narrower than its compiled width, so the expansion
/// is what makes the declared method total over head widths.
#[cfg(not(target_arch = "wasm32"))]
pub fn project_to_route_width(vector: &[f32]) -> Result<Vec<f32>, SourceUnavailable> {
    if vector.is_empty() {
        return Err(SourceUnavailable::new(
            "cannot project an empty head vector to the route-code width",
        ));
    }
    let projection = projection_implementation(
        GeometryProjection::BUCKET_AVERAGE_ID,
        GeometryProjection::BUCKET_AVERAGE_VERSION,
    )?;
    let mut out = vec![0.0f32; ROUTE_CODE_BITS];
    if vector.len() >= ROUTE_CODE_BITS {
        projection(vector, &mut out);
        return Ok(out);
    }
    let copies = ROUTE_CODE_BITS.div_ceil(vector.len());
    let mut tiled = Vec::with_capacity(copies * vector.len());
    for _ in 0..copies {
        tiled.extend_from_slice(vector);
    }
    projection(&tiled, &mut out);
    Ok(out)
}

#[cfg(not(target_arch = "wasm32"))]
fn pack_bits(projected: &[f32], thresholds: &[f32]) -> [u8; ROUTE_CODE_BYTES] {
    let mut code = [0u8; ROUTE_CODE_BYTES];
    for (bit, (&value, &threshold)) in projected.iter().zip(thresholds.iter()).enumerate() {
        if value >= threshold {
            // LSB-first within each byte (declared bit packing).
            code[bit >> 3] |= 1 << (bit & 7);
        }
    }
    code
}

/// Run `route-fit/1` over a decoded trace corpus: per `(declared layer,
/// head)`, project every query/key head vector, compute the per-bit
/// lower-median thresholds over the fit sample in the declared fixed
/// order, and binarize into route codes. Deterministic: fixed iteration
/// order everywhere, `f32::total_cmp` sorts, no map iteration reaches
/// any byte.
#[cfg(not(target_arch = "wasm32"))]
pub fn fit_route_codes(corpus: &RouteTraceCorpus) -> Result<FittedRouteCodes, SourceUnavailable> {
    use uor_r4_graph_format::route_attention::ROUTE_MAX_CANDIDATES;
    for story in &corpus.stories {
        if story.steps.len() > ROUTE_MAX_CANDIDATES {
            return Err(SourceUnavailable::new(format!(
                "story {} has {} positions, more than the deployed candidate bound \
                 {ROUTE_MAX_CANDIDATES}; the prefix candidate table would not fit \
                 the operator instance",
                story.story,
                story.steps.len()
            )));
        }
    }
    let geometry = corpus.geometry;
    let head_dim = geometry.residual_width / geometry.heads;
    let kv_width = geometry.residual_width * geometry.kv_heads / geometry.heads;
    let kv_head_dim = kv_width / geometry.kv_heads;
    let top_m = (ROUTE_MAX_TOP_M as u32).min(corpus.support_size);

    let mut heads = Vec::new();
    for (lane_index, &layer) in corpus.declared_layers.iter().enumerate() {
        for head in 0..geometry.heads {
            let q_start = head * head_dim;
            let kv_head = head * geometry.kv_heads / geometry.heads;
            let k_start = kv_head * kv_head_dim;
            // Projected vectors in the declared sample order: story
            // ascending, position ascending, query then key.
            let mut projected_queries: Vec<Vec<Vec<f32>>> = Vec::new();
            let mut projected_keys: Vec<Vec<Vec<f32>>> = Vec::new();
            for story in &corpus.stories {
                let mut story_queries = Vec::with_capacity(story.steps.len());
                let mut story_keys = Vec::with_capacity(story.steps.len());
                for step in &story.steps {
                    let q_slice = &step.q_rows[lane_index][q_start..q_start + head_dim];
                    let k_slice = &step.k_rows[lane_index][k_start..k_start + kv_head_dim];
                    story_queries.push(project_to_route_width(q_slice)?);
                    story_keys.push(project_to_route_width(k_slice)?);
                }
                projected_queries.push(story_queries);
                projected_keys.push(story_keys);
            }
            // Per-bit lower-median thresholds over the whole sample.
            let mut thresholds = vec![0.0f32; ROUTE_CODE_BITS];
            let mut column = Vec::new();
            for (bit, threshold) in thresholds.iter_mut().enumerate() {
                column.clear();
                for (story_queries, story_keys) in projected_queries.iter().zip(&projected_keys) {
                    for (query, key) in story_queries.iter().zip(story_keys.iter()) {
                        column.push(query[bit]);
                        column.push(key[bit]);
                    }
                }
                if column.is_empty() {
                    return Err(SourceUnavailable::new(
                        "empty fit sample: the corpus has no steps to fit thresholds from",
                    ));
                }
                column.sort_by(f32::total_cmp);
                *threshold = column[(column.len() - 1) / 2];
            }
            let query_codes = projected_queries
                .iter()
                .map(|story| {
                    story
                        .iter()
                        .map(|projected| pack_bits(projected, &thresholds))
                        .collect()
                })
                .collect();
            let key_codes = projected_keys
                .iter()
                .map(|story| {
                    story
                        .iter()
                        .map(|projected| pack_bits(projected, &thresholds))
                        .collect()
                })
                .collect();
            heads.push(HeadCodes {
                layer,
                head: head as u32,
                thresholds,
                query_codes,
                key_codes,
            });
        }
    }
    Ok(FittedRouteCodes {
        method: RouteFitMethod::route_fit_v1(),
        top_m,
        heads,
    })
}

/// Assemble the synthetic-arm fit manifest: the eight identity fields
/// with honest typed absence where no real value exists (tokenizer —
/// the synthetic teacher consumes raw ids), the typed records, and the
/// v1 parameter-provenance labels.
#[cfg(not(target_arch = "wasm32"))]
pub fn synthetic_fit_manifest(
    corpus: &RouteTraceCorpus,
    teacher_kappa: &str,
) -> Result<FitManifest, SourceUnavailable> {
    let method = fit_method_spec(ROUTE_FIT_ID, ROUTE_FIT_VERSION)?;
    let geometry = GeometryProjection::bucket_average(
        corpus.geometry.residual_width.max(ROUTE_CODE_BITS) as u32,
        ROUTE_CODE_BITS as u32,
    );
    let operator = uor_r4_model_source::attention::operator_spec(
        AttentionOperatorSpec::R4_ROUTE_ID,
        AttentionOperatorSpec::R4_ROUTE_VERSION,
    )?;
    let geometry_identity = geometry.declared_digest();
    let operator_identity = operator.declared_digest();
    Ok(FitManifest {
        format: FIT_MANIFEST_FORMAT.to_owned(),
        method,
        geometry: Some(geometry),
        trace_profile: Some(corpus.trace_profile.clone()),
        operator: Some(operator),
        source_snapshot: Some(teacher_kappa.to_owned()),
        // Genuinely absent on the synthetic arm: no tokenizer exists
        // (raw token ids), so the identity is a typed None — never an
        // empty string pretending to be a κ.
        tokenizer: None,
        adapter: Some("synthetic-route-teacher/1".to_owned()),
        trace: Some(corpus.trace_kappa.clone()),
        geometry_identity: Some(geometry_identity),
        operator_identity: Some(operator_identity),
        corpus: Some(corpus.records_kappa.clone()),
        compiler: Some(format!(
            "uor-r4-graph-compiler/{}",
            env!("CARGO_PKG_VERSION")
        )),
        parameters: route_fit_v1_parameter_labels(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_canonical_bytes_are_pinned() {
        let method = RouteFitMethod::route_fit_v1();
        let pinned = "uor-r4-route-fit/1\n\
             id=route-fit\n\
             version=1\n\
             code_bits=288\n\
             param.projection=bucket-average/1-registry-to-288-buckets\n\
             param.expansion=cyclic-tile-to-least-multiple-of-width-at-or-above-288-when-narrower\n\
             param.threshold=per-bit-lower-median-over-fit-sample-f32-total-cmp-sorted-index-(n-1)/2\n\
             param.binarization=bit-set-when-value-at-or-above-threshold\n\
             param.bit_packing=lsb-first-within-byte\n\
             param.sample_order=story-ascending-position-ascending-query-then-key\n\
             param.mask=full\n\
             param.radii=absent\n\
             param.residual_projection=absent\n\
             param.top_m_rule=min-route-max-top-m-and-trace-support-cap\n\
             param.candidate_rule=prefix-key-codes-position-ascending-candidate-index-equals-position\n\
             param.code_space=shared-query-key-thresholds-per-layer-head\n";
        assert_eq!(method.canonical_bytes(), pinned.as_bytes());
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(method.declared_digest, expected);
        assert_eq!(method.declared_digest(), expected);
    }

    #[test]
    fn manifest_absence_is_typed_and_digested() {
        let mut manifest = FitManifest {
            format: FIT_MANIFEST_FORMAT.to_owned(),
            method: RouteFitMethod::route_fit_v1(),
            parameters: route_fit_v1_parameter_labels(),
            ..FitManifest::default()
        };
        let absent_kappa = manifest.kappa();
        let text = String::from_utf8(manifest.canonical_bytes()).expect("utf8");
        assert!(text.contains("tokenizer=absent\n"));
        assert!(text.contains("parameter.route_codes=compiled\n"));
        assert!(text.contains("parameter.radii=absent\n"));
        // Absence is not an empty value: setting the field to an empty
        // string is a DIFFERENT identity than leaving it absent.
        manifest.tokenizer = Some(String::new());
        assert_ne!(manifest.kappa(), absent_kappa);
    }

    #[test]
    fn projection_tiles_narrow_vectors_through_the_registry() {
        // 16-wide head vector: tiled 18x to exactly 288, identity
        // buckets — every output bucket is one tiled element.
        let vector: Vec<f32> = (0..16).map(|i| i as f32 - 8.0).collect();
        let projected = project_to_route_width(&vector).expect("projects");
        assert_eq!(projected.len(), ROUTE_CODE_BITS);
        for (bit, &value) in projected.iter().enumerate() {
            assert_eq!(value.to_bits(), vector[bit % 16].to_bits());
        }
        // A 288-wide vector projects directly (identity buckets).
        let wide: Vec<f32> = (0..288).map(|i| (i as f32) / 7.0).collect();
        let projected = project_to_route_width(&wide).expect("projects");
        for (&out, &input) in projected.iter().zip(wide.iter()) {
            assert_eq!(out.to_bits(), input.to_bits());
        }
    }

    #[test]
    fn bit_packing_is_lsb_first() {
        let mut projected = vec![0.0f32; ROUTE_CODE_BITS];
        let thresholds = vec![0.5f32; ROUTE_CODE_BITS];
        projected[0] = 1.0; // bit 0 -> byte 0, mask 0x01
        projected[9] = 1.0; // bit 9 -> byte 1, mask 0x02
        let code = pack_bits(&projected, &thresholds);
        assert_eq!(code[0], 0x01);
        assert_eq!(code[1], 0x02);
        assert!(code[2..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn synthetic_teacher_is_deterministic_and_traceable() {
        let mut a = SyntheticRouteTeacher::new();
        let mut b = SyntheticRouteTeacher::new();
        assert_eq!(a.kappa(), b.kappa());
        let mut logits_a = vec![0.0f32; SYNTH_VOCAB];
        let mut logits_b = vec![0.0f32; SYNTH_VOCAB];
        a.reset();
        b.reset();
        for pos in 0..4 {
            a.step(1 + pos, pos, &mut logits_a);
            b.step(1 + pos, pos, &mut logits_b);
        }
        let bits_a: Vec<u32> = logits_a.iter().map(|v| v.to_bits()).collect();
        let bits_b: Vec<u32> = logits_b.iter().map(|v| v.to_bits()).collect();
        assert_eq!(bits_a, bits_b);
        // The traced step produces identical logits to the plain step
        // (capture only copies out).
        let mut c = SyntheticRouteTeacher::new();
        c.reset();
        let mut logits_c = vec![0.0f32; SYNTH_VOCAB];
        let request = TraceCaptureRequest {
            residual_layers: &[0, 1],
            qkv_layers: &[0, 1],
            attention_layers: &[0, 1],
        };
        let mut residuals = 0usize;
        let mut qkv = 0usize;
        let mut attention = 0usize;
        for pos in 0..4 {
            let mut residual_sink = |_l: usize, _x: &[f32]| residuals += 1;
            let mut qkv_sink = |_l: usize, q: &[f32], k: &[f32], _v: &[f32]| {
                assert_eq!(q.len(), SYNTH_DIM);
                assert_eq!(k.len(), SYNTH_DIM);
                qkv += 1;
            };
            let mut attention_sink = |_l: usize, _h: usize, att: &[f32]| {
                assert_eq!(att.len(), pos + 1);
                let total: f32 = att.iter().sum();
                assert!((total - 1.0).abs() < 1e-4);
                attention += 1;
            };
            let captured = c.step_with_trace_capture(
                1 + pos,
                pos,
                &mut logits_c,
                &request,
                &mut TraceCaptureSinks {
                    residual: &mut residual_sink,
                    qkv: &mut qkv_sink,
                    attention: &mut attention_sink,
                },
            );
            assert!(captured);
        }
        let bits_c: Vec<u32> = logits_c.iter().map(|v| v.to_bits()).collect();
        assert_eq!(bits_a, bits_c);
        assert_eq!(residuals, 4 * SYNTH_LAYERS);
        assert_eq!(qkv, 4 * SYNTH_LAYERS);
        assert_eq!(attention, 4 * SYNTH_LAYERS * SYNTH_HEADS);
        // An empty restriction plan is exactly the teacher forward.
        let mut d = SyntheticRouteTeacher::new();
        let plan = StoryRestrictionPlan::new();
        let forced = d.teacher_forced_logits(&[1, 2, 3, 4], &plan);
        let bits_d: Vec<u32> = forced[3].iter().map(|v| v.to_bits()).collect();
        assert_eq!(bits_a, bits_d);
    }
}
