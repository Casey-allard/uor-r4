//! Adapter conformance (#599): typed feature declarations, schema-versioned
//! source-executor parity fixtures, and a deterministic fixture runner.
//!
//! An adapter can load and compile while silently misinterpreting tokenizer
//! policy, RoPE/norm/activation/GQA/bias configuration, or tensor layout.
//! This module closes that gap at the source boundary, host-side only:
//!
//! * [`AdapterFeatures`] — the adapter's typed declaration of exactly which
//!   configuration space it executes faithfully. The parsed `config.json` is
//!   validated against the declaration at oracle construction, BEFORE any
//!   tensor is read and before any observation can be generated; anything
//!   outside the declaration is a focused
//!   [`SourceIngestKind::UnsupportedConfigFeature`] failure (fail-closed).
//! * [`AdapterFixture`] — the canonical-JSON, schema-versioned
//!   (`uor-r4-adapter-fixture/1`) parity fixture: prompt token id + byte
//!   string sequences, bounded per-layer residual captures (declared layer
//!   indices only), final hidden state, logits, top-k, per-check tolerances,
//!   and an identity block (#597 manifest binding, source κ, adapter,
//!   compiler, tokenizer).
//! * [`run_fixture`] / [`run_fixture_file`] — the deterministic runner:
//!   loads a fixture, runs the source executor, compares per check against
//!   the fixture's tolerances, and returns a three-state
//!   [`ConformanceReport`]: PASS, FAIL with per-check numeric deltas, or
//!   UNAVAILABLE naming the missing prerequisite. A missing pinned snapshot
//!   or fixture is reported as UNAVAILABLE evidence — never silently
//!   skipped. Running the same fixture twice produces byte-identical
//!   canonical reports.
//!
//! The deployed runtime is untouched: everything here is a compiler-side
//! (host) surface over the existing teacher executor.

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    BehaviorSource, HuggingFaceLlamaOracle, RepresentationSource, SourceIngestKind,
    SourceUnavailable, TeacherOracle,
};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::BTreeMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Schema tag of the adapter-conformance parity fixture format.
#[cfg(not(target_arch = "wasm32"))]
pub const ADAPTER_FIXTURE_SCHEMA: &str = "uor-r4-adapter-fixture/1";

/// Schema tag of the runner's canonical conformance report.
#[cfg(not(target_arch = "wasm32"))]
pub const CONFORMANCE_REPORT_SCHEMA: &str = "uor-r4-adapter-conformance-report/1";

/// Schema tag of the #597 source-snapshot manifest this module can bind.
/// Mirrors the root crate's `SOURCE_MANIFEST_SCHEMA`; the manifest is
/// produced there and only *read* here.
#[cfg(not(target_arch = "wasm32"))]
pub const SOURCE_MANIFEST_SCHEMA: &str = "uor-r4-source-manifest/1";

/// File name of the #597 source-snapshot manifest inside a snapshot
/// directory.
#[cfg(not(target_arch = "wasm32"))]
pub const SOURCE_MANIFEST_FILE_NAME: &str = "source_manifest.json";

/// One configuration feature the adapter declares support for. Named inside
/// [`SourceIngestKind::UnsupportedConfigFeature`] so every rejection is
/// focused on the exact feature that fell outside the declaration.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterFeature {
    /// `model_type` — the architecture family label.
    ModelType,
    /// `hidden_act` — the MLP activation function.
    Activation,
    /// `rms_norm_eps` — the RMSNorm epsilon (the executor's is fixed).
    NormEpsilon,
    /// `rope_interleaved` / `rope_scaling` — the RoPE application mode.
    RopeMode,
    /// `rope_theta` — the RoPE base frequency.
    RopeTheta,
    /// `num_attention_heads` / `num_key_value_heads` / `hidden_size` —
    /// the GQA/MQA head geometry.
    HeadGeometry,
    /// `attention_bias` — Q/K/V/output projection biases.
    AttentionBias,
    /// `mlp_bias` — MLP projection biases.
    MlpBias,
    /// `tie_word_embeddings` — embedding/lm-head tying.
    TiedEmbeddings,
    /// `bos_token_id` / `eos_token_id` — the special-token policy.
    TokenPolicy,
    /// `chat_template` — the chat-template policy.
    ChatTemplate,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for AdapterFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ModelType => "model-type",
            Self::Activation => "activation",
            Self::NormEpsilon => "norm-epsilon",
            Self::RopeMode => "rope-mode",
            Self::RopeTheta => "rope-theta",
            Self::HeadGeometry => "head-geometry",
            Self::AttentionBias => "attention-bias",
            Self::MlpBias => "mlp-bias",
            Self::TiedEmbeddings => "tied-embeddings",
            Self::TokenPolicy => "token-policy",
            Self::ChatTemplate => "chat-template",
        })
    }
}

/// How rotary position embeddings are applied to a head.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RopeMode {
    /// Adjacent-pair rotation (converted llama2.c checkpoints).
    Interleaved,
    /// Half-rotation over the two head halves (native Hugging Face).
    HalfRotation,
}

/// Which embedding/lm-head tying arrangements the adapter executes.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingTying {
    /// Only `tie_word_embeddings: true`.
    TiedOnly,
    /// Only untied (a separate `lm_head.weight`).
    UntiedOnly,
    /// Both arrangements.
    Either,
}

/// The adapter's chat-template stance.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatTemplatePolicy {
    /// The source executor consumes raw token ids and never interprets a
    /// chat template; a template in the snapshot is tolerated, carried by
    /// the #597 manifest, and applied only by the host tokenizer surface.
    NotInterpreted,
}

/// The typed feature declaration an adapter constructs (#599): exactly the
/// configuration space its executor interprets faithfully. Oracle
/// construction validates the parsed `config.json` against this declaration
/// before any observation can be generated; every feature outside it is a
/// focused, fail-closed [`SourceIngestKind::UnsupportedConfigFeature`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct AdapterFeatures {
    /// The adapter's name (e.g. `huggingface-llama`).
    pub adapter_name: String,
    /// The adapter's version (this crate's version).
    pub adapter_version: String,
    /// Supported `model_type` labels; an absent `model_type` is tolerated.
    pub model_types: Vec<String>,
    /// Supported `hidden_act` values; an absent field means the Hugging
    /// Face default `silu`, which must itself be in this list.
    pub activations: Vec<String>,
    /// Exact supported `rms_norm_eps` values. The executor's rmsnorm uses a
    /// fixed epsilon, so this is a list of exact values, not a range.
    pub norm_epsilons: Vec<f64>,
    /// Supported RoPE application modes.
    pub rope_modes: Vec<RopeMode>,
    /// Supported inclusive `rope_theta` range (finite, positive).
    pub rope_theta: (f64, f64),
    /// Whether a non-null `rope_scaling` configuration is executed. When
    /// `false`, any scaled/unknown RoPE mode is rejected by name.
    pub rope_scaling: bool,
    /// Whether Q/K/V/output projection biases are executed.
    pub attention_bias: bool,
    /// Whether MLP projection biases are executed.
    pub mlp_bias: bool,
    /// Supported embedding/lm-head tying arrangements.
    pub tied_embeddings: EmbeddingTying,
    /// Whether grouped-query/multi-query geometry is executed. When `true`,
    /// `num_key_value_heads` must divide `num_attention_heads`; when
    /// `false`, the two must be equal. Head size must divide `hidden_size`
    /// and be even (RoPE rotates half-pairs) in either case.
    pub grouped_query_attention: bool,
    /// Whether `bos_token_id`/`eos_token_id` must be scalar ids inside the
    /// vocabulary (list-valued token policies are rejected).
    pub scalar_token_ids: bool,
    /// The chat-template stance.
    pub chat_template: ChatTemplatePolicy,
}

#[cfg(not(target_arch = "wasm32"))]
impl AdapterFeatures {
    /// The declaration of [`HuggingFaceLlamaOracle`]: Llama-family
    /// Safetensors with SiLU/SwiGLU MLPs, RMSNorm at the executor's fixed
    /// epsilon 1e-5, unscaled RoPE in either application mode, bias-free
    /// projections, tied or untied embeddings, GQA/MQA head geometry, and
    /// scalar BOS/EOS ids. Chat templates are never interpreted by the
    /// source executor.
    pub fn huggingface_llama() -> Self {
        Self {
            adapter_name: "huggingface-llama".to_owned(),
            adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
            model_types: vec!["llama".to_owned()],
            activations: vec!["silu".to_owned()],
            norm_epsilons: vec![1e-5],
            rope_modes: vec![RopeMode::HalfRotation, RopeMode::Interleaved],
            rope_theta: (1.0, 1e8),
            rope_scaling: false,
            attention_bias: false,
            mlp_bias: false,
            tied_embeddings: EmbeddingTying::Either,
            grouped_query_attention: true,
            scalar_token_ids: true,
            chat_template: ChatTemplatePolicy::NotInterpreted,
        }
    }

    /// Validate a parsed `config.json` against this declaration. Called at
    /// oracle construction before the typed configuration parse, before the
    /// #598 snapshot boundary, and before any observation. The first
    /// feature outside the declaration fails with its focused
    /// [`SourceIngestKind::UnsupportedConfigFeature`]; absent optional
    /// fields are validated at their Hugging Face defaults.
    pub fn validate_config(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        self.check_model_type(config)?;
        self.check_activation(config)?;
        self.check_norm_epsilon(config)?;
        self.check_rope(config)?;
        self.check_biases(config)?;
        self.check_tying(config)?;
        self.check_head_geometry(config)?;
        self.check_token_policy(config)?;
        // Chat template: the declared policy is NotInterpreted — a template
        // anywhere in the snapshot is tolerated and never executed here, so
        // there is nothing to reject.
        let ChatTemplatePolicy::NotInterpreted = self.chat_template;
        Ok(())
    }

    fn check_model_type(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        let Some(value) = config.get("model_type") else {
            return Ok(());
        };
        let declared = format!("one of {:?} (or absent)", self.model_types);
        match value.as_str() {
            Some(label) if self.model_types.iter().any(|known| known == label) => Ok(()),
            _ => Err(unsupported(AdapterFeature::ModelType, declared, value)),
        }
    }

    fn check_activation(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        let declared = format!("one of {:?}", self.activations);
        match config.get("hidden_act") {
            // Absent: the Hugging Face Llama default is silu; the default
            // itself must be declared.
            None => {
                if self.activations.iter().any(|known| known == "silu") {
                    Ok(())
                } else {
                    Err(unsupported(
                        AdapterFeature::Activation,
                        declared,
                        &serde_json::Value::String("silu (default)".to_owned()),
                    ))
                }
            }
            Some(value) => match value.as_str() {
                Some(name) if self.activations.iter().any(|known| known == name) => Ok(()),
                _ => Err(unsupported(AdapterFeature::Activation, declared, value)),
            },
        }
    }

    fn check_norm_epsilon(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        let declared = format!("exactly one of {:?}", self.norm_epsilons);
        let Some(value) = config.get("rms_norm_eps") else {
            // Absent: the typed parse defaults to 1e-5, which must itself
            // be declared.
            return if self.norm_epsilons.contains(&1e-5) {
                Ok(())
            } else {
                Err(unsupported(
                    AdapterFeature::NormEpsilon,
                    declared,
                    &serde_json::Value::String("1e-5 (default)".to_owned()),
                ))
            };
        };
        match value.as_f64() {
            Some(epsilon) if self.norm_epsilons.contains(&epsilon) => Ok(()),
            _ => Err(unsupported(AdapterFeature::NormEpsilon, declared, value)),
        }
    }

    fn check_rope(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        if let Some(value) = config.get("rope_scaling") {
            if !value.is_null() && !self.rope_scaling {
                return Err(unsupported(
                    AdapterFeature::RopeMode,
                    "unscaled RoPE only (rope_scaling absent or null)".to_owned(),
                    value,
                ));
            }
        }
        if let Some(value) = config.get("rope_interleaved") {
            let declared = format!("modes {:?} as a boolean rope_interleaved", self.rope_modes);
            let mode = match value.as_bool() {
                Some(true) => RopeMode::Interleaved,
                Some(false) => RopeMode::HalfRotation,
                None => return Err(unsupported(AdapterFeature::RopeMode, declared, value)),
            };
            if !self.rope_modes.contains(&mode) {
                return Err(unsupported(AdapterFeature::RopeMode, declared, value));
            }
        }
        if let Some(value) = config.get("rope_theta") {
            let (low, high) = self.rope_theta;
            let declared = format!("finite rope_theta in [{low}, {high}]");
            match value.as_f64() {
                Some(theta) if theta.is_finite() && theta >= low && theta <= high => {}
                _ => return Err(unsupported(AdapterFeature::RopeTheta, declared, value)),
            }
        }
        Ok(())
    }

    fn check_biases(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        for (field, feature, supported) in [
            (
                "attention_bias",
                AdapterFeature::AttentionBias,
                self.attention_bias,
            ),
            ("mlp_bias", AdapterFeature::MlpBias, self.mlp_bias),
        ] {
            let Some(value) = config.get(field) else {
                continue;
            };
            let declared = format!("{field} = {supported} (or absent = false)");
            match value.as_bool() {
                Some(actual) if !actual || supported => {}
                _ => return Err(unsupported(feature, declared, value)),
            }
        }
        Ok(())
    }

    fn check_tying(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        let declared = format!("{:?}", self.tied_embeddings);
        let tied = match config.get("tie_word_embeddings") {
            None => false,
            Some(value) => match value.as_bool() {
                Some(tied) => tied,
                None => {
                    return Err(unsupported(AdapterFeature::TiedEmbeddings, declared, value));
                }
            },
        };
        let supported = match self.tied_embeddings {
            EmbeddingTying::Either => true,
            EmbeddingTying::TiedOnly => tied,
            EmbeddingTying::UntiedOnly => !tied,
        };
        if supported {
            Ok(())
        } else {
            Err(unsupported(
                AdapterFeature::TiedEmbeddings,
                declared,
                &serde_json::Value::Bool(tied),
            ))
        }
    }

    fn check_head_geometry(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        // Presence of these required fields is the typed parse's concern;
        // this check validates the geometry *relationships* when present.
        let read = |field: &str| config.get(field).and_then(serde_json::Value::as_u64);
        let (Some(hidden), Some(heads), Some(kv_heads)) = (
            read("hidden_size"),
            read("num_attention_heads"),
            read("num_key_value_heads"),
        ) else {
            return Ok(());
        };
        let declared = if self.grouped_query_attention {
            "num_key_value_heads dividing num_attention_heads (GQA/MQA), \
             head size = hidden_size / num_attention_heads integral and even"
        } else {
            "num_key_value_heads equal to num_attention_heads, \
             head size = hidden_size / num_attention_heads integral and even"
        };
        let grouping_ok = if self.grouped_query_attention {
            kv_heads >= 1 && kv_heads <= heads && heads % kv_heads == 0
        } else {
            kv_heads == heads
        };
        let head_ok = heads >= 1 && hidden % heads == 0 && (hidden / heads) % 2 == 0;
        if grouping_ok && head_ok {
            Ok(())
        } else {
            Err(unsupported(
                AdapterFeature::HeadGeometry,
                declared.to_owned(),
                &serde_json::json!({
                    "hidden_size": hidden,
                    "num_attention_heads": heads,
                    "num_key_value_heads": kv_heads,
                }),
            ))
        }
    }

    fn check_token_policy(&self, config: &serde_json::Value) -> Result<(), SourceUnavailable> {
        if !self.scalar_token_ids {
            return Ok(());
        }
        let vocab = config.get("vocab_size").and_then(serde_json::Value::as_u64);
        for field in ["bos_token_id", "eos_token_id"] {
            let Some(value) = config.get(field) else {
                continue;
            };
            let declared = format!("scalar {field} inside the vocabulary");
            match (value.as_u64(), vocab) {
                (Some(id), Some(vocab)) if id < vocab => {}
                (Some(_), None) => {}
                _ => return Err(unsupported(AdapterFeature::TokenPolicy, declared, value)),
            }
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported(
    feature: AdapterFeature,
    declared: String,
    actual: &serde_json::Value,
) -> SourceUnavailable {
    SourceIngestKind::UnsupportedConfigFeature {
        feature,
        declared,
        actual: actual.to_string(),
    }
    .into()
}

// ---------------------------------------------------------------------------
// Fixture format: uor-r4-adapter-fixture/1 (canonical JSON).
// ---------------------------------------------------------------------------

/// One prompt token: the tokenizer id and the token's byte string (the
/// tokenizer surface's spelling of the id, recorded so a fixture binds the
/// tokenizer policy and not just the id sequence).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PromptToken {
    /// Tokenizer id.
    pub id: u32,
    /// The token's byte string.
    pub bytes: String,
}

/// One top-k entry of the final step's softmax distribution.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TopKEntry {
    /// Token id.
    pub token: u32,
    /// Probability.
    pub probability: f32,
}

/// Per-check absolute tolerances. Defaults (see [`Default`]) cover the
/// deterministic-per-machine variance between the fast SIMD matmul backends
/// the teacher may select on different hosts; on one host a replay is
/// bit-identical and every delta is zero.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FixtureTolerances {
    /// Max absolute delta per captured per-layer residual element (default 1e-4).
    pub per_layer_abs: f64,
    /// Max absolute delta per final-hidden element (default 1e-4).
    pub hidden_abs: f64,
    /// Max absolute delta per logit (default 5e-3).
    pub logit_abs: f64,
    /// Max absolute delta per top-k probability (default 1e-4).
    pub probability_abs: f64,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for FixtureTolerances {
    fn default() -> Self {
        Self {
            per_layer_abs: 1e-4,
            hidden_abs: 1e-4,
            logit_abs: 5e-3,
            probability_abs: 1e-4,
        }
    }
}

/// The fixture's identity block: which exact source, adapter, compiler, and
/// tokenizer produced the recorded expectations.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FixtureIdentity {
    /// #597 source-manifest binding, read via the snapshot's
    /// `source_manifest.json` when present: `blake3:<hex>` over the
    /// canonical manifest file bytes (see [`source_manifest_binding`]).
    /// `None` when the snapshot carries no manifest file.
    pub source_manifest_kappa: Option<String>,
    /// The teacher/source κ ([`TeacherOracle::kappa`]): blake3 over the
    /// snapshot's shard bytes.
    pub source_kappa: String,
    /// Adapter name (from [`AdapterFeatures::adapter_name`]).
    pub adapter_name: String,
    /// Adapter version.
    pub adapter_version: String,
    /// Compiler (this crate's) version.
    pub compiler_version: String,
    /// Tokenizer identity ([`RepresentationSource::tokenizer_address`]).
    pub tokenizer: String,
}

/// One fixture case: a prompt token sequence and the expected observations
/// after teacher-forcing the whole prompt.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FixtureCase {
    /// Case name (unique within the fixture).
    pub name: String,
    /// The prompt: token ids + byte strings, stepped in order.
    pub prompt: Vec<PromptToken>,
    /// Expected residual stream after each captured layer of the final
    /// step, keyed by layer index (sorted map; declared indices only).
    pub per_layer: BTreeMap<usize, Vec<f32>>,
    /// Expected final hidden state (post-final-rmsnorm) of the last step.
    pub final_hidden: Vec<f32>,
    /// Expected logits of the last step.
    pub logits: Vec<f32>,
    /// Expected top-k of the last step's softmax distribution.
    pub top_k: Vec<TopKEntry>,
}

/// The schema-versioned adapter-conformance parity fixture
/// (`uor-r4-adapter-fixture/1`). Canonical JSON: struct declaration order
/// is the field order, maps are sorted ([`BTreeMap`]), serialization is
/// compact — see [`canonical_fixture_json`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdapterFixture {
    /// Always [`ADAPTER_FIXTURE_SCHEMA`].
    pub schema: String,
    /// Identity block.
    pub identity: FixtureIdentity,
    /// The bounded set of layer indices captured per case.
    pub capture_layers: Vec<usize>,
    /// Per-check tolerances.
    pub tolerances: FixtureTolerances,
    /// The cases.
    pub cases: Vec<FixtureCase>,
}

/// What to record into a fixture: cases, the bounded capture set, top-k
/// depth, and tolerances.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct FixtureSpec {
    /// Case names and prompts.
    pub cases: Vec<CaseSpec>,
    /// Layer indices whose post-layer residual stream to capture on the
    /// final step of each case. Bounded: each must be inside the model's
    /// layer range; only these declared indices are ever copied out.
    pub capture_layers: Vec<usize>,
    /// Top-k depth (1..=vocab).
    pub top_k: usize,
    /// Per-check tolerances to embed in the fixture.
    pub tolerances: FixtureTolerances,
}

/// One case to record: a name and a prompt.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub struct CaseSpec {
    /// Case name.
    pub name: String,
    /// Prompt token ids + byte strings.
    pub prompt: Vec<PromptToken>,
}

/// Canonical fixture bytes: compact JSON with struct-declaration field
/// order and sorted maps. The same fixture value always serializes to the
/// same bytes.
#[cfg(not(target_arch = "wasm32"))]
pub fn canonical_fixture_json(fixture: &AdapterFixture) -> String {
    // Fixture recording rejects non-finite values, so serialization cannot
    // fail on these closed struct types.
    serde_json::to_string(fixture).expect("adapter fixture serializes to canonical JSON")
}

/// Parse fixture bytes; the schema tag is validated by [`run_fixture`].
#[cfg(not(target_arch = "wasm32"))]
pub fn parse_fixture_json(text: &str) -> Result<AdapterFixture, SourceUnavailable> {
    Ok(serde_json::from_str(text)?)
}

/// Read the #597 source-manifest binding of a snapshot directory, via the
/// manifest file when present: `Ok(Some("blake3:<hex>"))` over the
/// `source_manifest.json` bytes (the canonical manifest serialization the
/// root crate writes; its uor-addr root κ addresses those same bytes),
/// `Ok(None)` when the snapshot has no manifest file. A present but
/// unreadable or wrong-schema manifest is a focused ingestion failure.
#[cfg(not(target_arch = "wasm32"))]
pub fn source_manifest_binding(
    snapshot_dir: impl AsRef<Path>,
) -> Result<Option<String>, SourceUnavailable> {
    let path = snapshot_dir.as_ref().join(SOURCE_MANIFEST_FILE_NAME);
    if !path.is_file() {
        return Ok(None);
    }
    let display = path.display().to_string();
    let unreadable = |reason: String| SourceIngestKind::Unreadable {
        path: display.clone(),
        reason,
    };
    let bytes = std::fs::read(&path).map_err(|error| unreadable(error.to_string()))?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| unreadable(error.to_string()))?;
    match manifest.get("schema").and_then(serde_json::Value::as_str) {
        Some(SOURCE_MANIFEST_SCHEMA) => {}
        other => {
            return Err(unreadable(format!(
                "source-manifest schema {other:?} is not {SOURCE_MANIFEST_SCHEMA:?}"
            ))
            .into());
        }
    }
    Ok(Some(format!("blake3:{}", blake3::hash(&bytes).to_hex())))
}

/// Record a parity fixture from a snapshot directory by running the source
/// executor itself: the oracle is constructed through the full #599 feature
/// gate and #598 ingestion boundary, each case's prompt is teacher-forced,
/// and the final step's bounded per-layer captures, final hidden state,
/// logits, and top-k become the fixture's expectations.
#[cfg(not(target_arch = "wasm32"))]
pub fn record_fixture(
    snapshot_dir: impl AsRef<Path>,
    spec: &FixtureSpec,
) -> Result<AdapterFixture, SourceUnavailable> {
    let snapshot_dir = snapshot_dir.as_ref();
    if spec.cases.is_empty() {
        return Err(SourceUnavailable::new(
            "fixture spec declares no cases; a fixture without cases is vacuous",
        ));
    }
    let longest_prompt = spec
        .cases
        .iter()
        .map(|case| case.prompt.len())
        .max()
        .unwrap_or(0);
    if longest_prompt == 0 {
        return Err(SourceUnavailable::new(
            "fixture spec declares an empty prompt; every case needs at least one token",
        ));
    }
    let mut oracle =
        HuggingFaceLlamaOracle::load_with_sequence_length(snapshot_dir, longest_prompt)?;
    let layers = oracle.cfg().n_layers;
    let mut capture_layers = spec.capture_layers.clone();
    capture_layers.sort_unstable();
    capture_layers.dedup();
    for &layer in &capture_layers {
        if layer >= layers {
            return Err(SourceUnavailable::new(format!(
                "capture layer {layer} is outside the model's {layers} layers"
            )));
        }
    }
    let vocab = oracle.vocab();
    if spec.top_k == 0 || spec.top_k > vocab {
        return Err(SourceUnavailable::new(format!(
            "top-k depth {} is outside 1..={vocab}",
            spec.top_k
        )));
    }
    let features = AdapterFeatures::huggingface_llama();
    let identity = FixtureIdentity {
        source_manifest_kappa: source_manifest_binding(snapshot_dir)?,
        source_kappa: oracle.kappa(),
        adapter_name: features.adapter_name,
        adapter_version: features.adapter_version,
        compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
        tokenizer: oracle.tokenizer_address().to_owned(),
    };
    let mut cases = Vec::with_capacity(spec.cases.len());
    for case in &spec.cases {
        let observed = observe_case(&mut oracle, &case.prompt, &capture_layers, spec.top_k)?;
        let finite = observed
            .final_hidden
            .iter()
            .chain(observed.logits.iter())
            .chain(observed.per_layer.values().flatten())
            .all(|value| value.is_finite());
        if !finite {
            return Err(SourceUnavailable::new(format!(
                "case {} produced a non-finite observation; the fixture would not be canonical",
                case.name
            )));
        }
        cases.push(FixtureCase {
            name: case.name.clone(),
            prompt: case.prompt.clone(),
            per_layer: observed.per_layer,
            final_hidden: observed.final_hidden,
            logits: observed.logits,
            top_k: observed.top_k,
        });
    }
    Ok(AdapterFixture {
        schema: ADAPTER_FIXTURE_SCHEMA.to_owned(),
        identity,
        capture_layers,
        tolerances: spec.tolerances.clone(),
        cases,
    })
}

#[cfg(not(target_arch = "wasm32"))]
struct CaseObservation {
    per_layer: BTreeMap<usize, Vec<f32>>,
    final_hidden: Vec<f32>,
    logits: Vec<f32>,
    top_k: Vec<TopKEntry>,
}

/// Teacher-force one prompt through the executor: plain steps for every
/// position but the last, a layer-capturing step (same executor path) for
/// the last, then read the final hidden state, logits, and top-k.
#[cfg(not(target_arch = "wasm32"))]
fn observe_case(
    oracle: &mut HuggingFaceLlamaOracle,
    prompt: &[PromptToken],
    capture_layers: &[usize],
    top_k: usize,
) -> Result<CaseObservation, SourceUnavailable> {
    let vocab = oracle.vocab();
    for token in prompt {
        if token.id as usize >= vocab {
            return Err(SourceUnavailable::new(format!(
                "prompt token id {} is outside the vocabulary ({vocab})",
                token.id
            )));
        }
    }
    let mut logits = vec![0f32; vocab];
    let mut per_layer = BTreeMap::new();
    oracle.reset();
    let last = prompt.len() - 1;
    for (pos, token) in prompt.iter().enumerate() {
        if pos == last {
            oracle.step_with_layer_capture(
                token.id as usize,
                pos,
                capture_layers,
                &mut |layer, residual| {
                    per_layer.insert(layer, residual.to_vec());
                },
            );
        } else {
            oracle.step(token.id as usize, pos, &mut logits);
        }
    }
    let final_hidden = oracle
        .hidden_state()
        .map(<[f32]>::to_vec)
        .unwrap_or_default();
    let mut top = vec![(0u32, 0f32); top_k];
    let written = oracle.top_k(top_k, &mut top);
    top.truncate(written);
    // The capturing step leaves the logits in the oracle state exactly as a
    // plain step would; read them back through the trace surface.
    let logits_out = oracle.last_logits().to_vec();
    Ok(CaseObservation {
        per_layer,
        final_hidden,
        logits: logits_out,
        top_k: top
            .into_iter()
            .map(|(token, probability)| TopKEntry { token, probability })
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// The deterministic fixture runner.
// ---------------------------------------------------------------------------

/// Three-state conformance verdict. UNAVAILABLE is evidence, not a skip:
/// it names the missing prerequisite in [`ConformanceReport::unavailable`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConformanceStatus {
    /// Every check passed within its tolerance.
    #[serde(rename = "PASS")]
    Pass,
    /// At least one check exceeded its tolerance or mismatched; the
    /// per-check reports carry the numeric deltas.
    #[serde(rename = "FAIL")]
    Fail,
    /// A prerequisite (pinned snapshot, fixture file, usable fixture
    /// schema) is missing; named explicitly, never silently skipped.
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

/// One check of a conformance run: name, verdict, tolerance, observed
/// numeric delta (maximum absolute difference), and an optional diagnostic.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConformanceCheck {
    /// Check name (`identity/...`, `fixture/...`, or `case/<name>/...`).
    pub name: String,
    /// Verdict for this check.
    pub status: ConformanceStatus,
    /// The tolerance the check ran under, for numeric checks.
    pub tolerance: Option<f64>,
    /// The observed maximum absolute delta, for numeric checks.
    pub delta: Option<f64>,
    /// Diagnostic detail (length mismatches, identity values, ...).
    pub detail: Option<String>,
}

/// The runner's canonical report (`uor-r4-adapter-conformance-report/1`).
/// Serialization is canonical JSON ([`canonical_report_json`]); running the
/// same fixture against the same snapshot twice produces byte-identical
/// reports.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConformanceReport {
    /// Always [`CONFORMANCE_REPORT_SCHEMA`].
    pub schema: String,
    /// Overall verdict: UNAVAILABLE if a prerequisite is missing, else FAIL
    /// if any check failed, else PASS.
    pub status: ConformanceStatus,
    /// The named missing prerequisite when status is UNAVAILABLE.
    pub unavailable: Option<String>,
    /// Per-check results (empty when UNAVAILABLE).
    pub checks: Vec<ConformanceCheck>,
}

/// Canonical report bytes: compact JSON, struct-declaration field order.
#[cfg(not(target_arch = "wasm32"))]
pub fn canonical_report_json(report: &ConformanceReport) -> String {
    // Deltas and tolerances are finite by construction (non-finite inputs
    // produce a Fail with detail instead), so serialization cannot fail.
    serde_json::to_string(report).expect("conformance report serializes to canonical JSON")
}

#[cfg(not(target_arch = "wasm32"))]
fn unavailable_report(prerequisite: String) -> ConformanceReport {
    ConformanceReport {
        schema: CONFORMANCE_REPORT_SCHEMA.to_owned(),
        status: ConformanceStatus::Unavailable,
        unavailable: Some(prerequisite),
        checks: Vec::new(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn detail_check(name: &str, status: ConformanceStatus, detail: String) -> ConformanceCheck {
    ConformanceCheck {
        name: name.to_owned(),
        status,
        tolerance: None,
        delta: None,
        detail: Some(detail),
    }
}

/// Compare two equal-role vectors under an absolute tolerance: FAIL on a
/// length mismatch, otherwise PASS/FAIL on the maximum absolute delta.
#[cfg(not(target_arch = "wasm32"))]
fn numeric_check(
    name: String,
    expected: &[f32],
    actual: &[f32],
    tolerance: f64,
) -> ConformanceCheck {
    if expected.len() != actual.len() {
        return ConformanceCheck {
            name,
            status: ConformanceStatus::Fail,
            tolerance: Some(tolerance),
            delta: None,
            detail: Some(format!(
                "length mismatch: fixture has {}, executor produced {}",
                expected.len(),
                actual.len()
            )),
        };
    }
    let mut delta = 0f64;
    for (&want, &got) in expected.iter().zip(actual) {
        let diff = (f64::from(want) - f64::from(got)).abs();
        if diff > delta {
            delta = diff;
        }
    }
    let within = delta.is_finite() && delta <= tolerance;
    ConformanceCheck {
        name,
        status: if within {
            ConformanceStatus::Pass
        } else {
            ConformanceStatus::Fail
        },
        tolerance: Some(tolerance),
        delta: Some(delta),
        detail: None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn identity_check(name: &str, declared: &str, actual: &str) -> ConformanceCheck {
    let matches = declared == actual;
    detail_check(
        name,
        if matches {
            ConformanceStatus::Pass
        } else {
            ConformanceStatus::Fail
        },
        if matches {
            declared.to_owned()
        } else {
            format!("fixture declares {declared:?}, executor reports {actual:?}")
        },
    )
}

/// Run one fixture against a snapshot directory. Deterministic three-state
/// outcome:
///
/// * **UNAVAILABLE** — the fixture schema is unusable or the pinned
///   snapshot is absent; the missing prerequisite is named. This is
///   reported evidence, never a silent skip.
/// * **FAIL** — the snapshot loaded (or was rejected by the fail-closed
///   feature gate, itself a named failing check) and at least one identity
///   or numeric check missed; numeric checks carry their deltas.
/// * **PASS** — every check within tolerance.
///
/// This is the host-side entry point tests and a CLI can share.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_fixture(fixture: &AdapterFixture, snapshot_dir: impl AsRef<Path>) -> ConformanceReport {
    let snapshot_dir = snapshot_dir.as_ref();
    if fixture.schema != ADAPTER_FIXTURE_SCHEMA {
        return unavailable_report(format!(
            "fixture schema {:?} is not the supported {ADAPTER_FIXTURE_SCHEMA:?}",
            fixture.schema
        ));
    }
    if !snapshot_dir.join("config.json").is_file() {
        return unavailable_report(format!(
            "pinned snapshot {} is absent (no config.json)",
            snapshot_dir.display()
        ));
    }
    let longest_prompt = fixture
        .cases
        .iter()
        .map(|case| case.prompt.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let mut checks = Vec::new();
    let mut oracle =
        match HuggingFaceLlamaOracle::load_with_sequence_length(snapshot_dir, longest_prompt) {
            Ok(oracle) => oracle,
            Err(error) => {
                // The snapshot exists but did not become a teacher (feature
                // gate or ingestion boundary): that is a conformance FAIL
                // with the focused reason, not an unavailability.
                checks.push(detail_check(
                    "oracle-construction",
                    ConformanceStatus::Fail,
                    error.to_string(),
                ));
                return finish_report(checks);
            }
        };
    checks.push(detail_check(
        "oracle-construction",
        ConformanceStatus::Pass,
        "teacher constructed through the #599 feature gate".to_owned(),
    ));

    let identity = &fixture.identity;
    checks.push(identity_check(
        "identity/source-kappa",
        &identity.source_kappa,
        &oracle.kappa(),
    ));
    let manifest = match source_manifest_binding(snapshot_dir) {
        Ok(binding) => binding,
        Err(error) => {
            checks.push(detail_check(
                "identity/source-manifest",
                ConformanceStatus::Fail,
                error.to_string(),
            ));
            None
        }
    };
    match (&identity.source_manifest_kappa, &manifest) {
        (Some(declared), Some(actual)) => {
            checks.push(identity_check("identity/source-manifest", declared, actual));
        }
        (Some(declared), None) => checks.push(detail_check(
            "identity/source-manifest",
            ConformanceStatus::Fail,
            format!(
                "fixture declares manifest binding {declared:?} but the snapshot has no \
                 readable {SOURCE_MANIFEST_FILE_NAME}"
            ),
        )),
        (None, _) => checks.push(detail_check(
            "identity/source-manifest",
            ConformanceStatus::Pass,
            "fixture declares no #597 manifest binding".to_owned(),
        )),
    }
    let features = AdapterFeatures::huggingface_llama();
    checks.push(identity_check(
        "identity/adapter-name",
        &identity.adapter_name,
        &features.adapter_name,
    ));
    checks.push(identity_check(
        "identity/adapter-version",
        &identity.adapter_version,
        &features.adapter_version,
    ));
    checks.push(identity_check(
        "identity/compiler-version",
        &identity.compiler_version,
        env!("CARGO_PKG_VERSION"),
    ));
    checks.push(identity_check(
        "identity/tokenizer",
        &identity.tokenizer,
        oracle.tokenizer_address(),
    ));

    let layers = oracle.cfg().n_layers;
    let vocab = oracle.vocab();
    if let Some(&outside) = fixture
        .capture_layers
        .iter()
        .find(|&&layer| layer >= layers)
    {
        checks.push(detail_check(
            "fixture/capture-layers",
            ConformanceStatus::Fail,
            format!("declared capture layer {outside} is outside the model's {layers} layers"),
        ));
        return finish_report(checks);
    }

    for case in &fixture.cases {
        let prefix = format!("case/{}", case.name);
        if case.prompt.is_empty() || case.prompt.iter().any(|token| token.id as usize >= vocab) {
            checks.push(detail_check(
                &format!("{prefix}/prompt"),
                ConformanceStatus::Fail,
                format!("prompt is empty or names a token id outside the vocabulary ({vocab})"),
            ));
            continue;
        }
        let observed = match observe_case(
            &mut oracle,
            &case.prompt,
            &fixture.capture_layers,
            case.top_k.len().clamp(1, vocab),
        ) {
            Ok(observed) => observed,
            Err(error) => {
                checks.push(detail_check(
                    &format!("{prefix}/execution"),
                    ConformanceStatus::Fail,
                    error.to_string(),
                ));
                continue;
            }
        };
        for &layer in &fixture.capture_layers {
            let name = format!("{prefix}/layer-{layer}");
            match (case.per_layer.get(&layer), observed.per_layer.get(&layer)) {
                (Some(expected), Some(actual)) => checks.push(numeric_check(
                    name,
                    expected,
                    actual,
                    fixture.tolerances.per_layer_abs,
                )),
                (expected, _) => checks.push(detail_check(
                    &name,
                    ConformanceStatus::Fail,
                    format!(
                        "captured layer {layer} is {} in the fixture",
                        if expected.is_none() {
                            "absent"
                        } else {
                            "present"
                        }
                    ),
                )),
            }
        }
        checks.push(numeric_check(
            format!("{prefix}/final-hidden"),
            &case.final_hidden,
            &observed.final_hidden,
            fixture.tolerances.hidden_abs,
        ));
        checks.push(numeric_check(
            format!("{prefix}/logits"),
            &case.logits,
            &observed.logits,
            fixture.tolerances.logit_abs,
        ));
        let expected_tokens: Vec<u32> = case.top_k.iter().map(|entry| entry.token).collect();
        let actual_tokens: Vec<u32> = observed.top_k.iter().map(|entry| entry.token).collect();
        if expected_tokens != actual_tokens {
            checks.push(detail_check(
                &format!("{prefix}/top-k"),
                ConformanceStatus::Fail,
                format!(
                    "top-k token order differs: fixture {expected_tokens:?}, \
                     executor {actual_tokens:?}"
                ),
            ));
        } else {
            let expected_probs: Vec<f32> =
                case.top_k.iter().map(|entry| entry.probability).collect();
            let actual_probs: Vec<f32> = observed
                .top_k
                .iter()
                .map(|entry| entry.probability)
                .collect();
            checks.push(numeric_check(
                format!("{prefix}/top-k"),
                &expected_probs,
                &actual_probs,
                fixture.tolerances.probability_abs,
            ));
        }
    }
    finish_report(checks)
}

#[cfg(not(target_arch = "wasm32"))]
fn finish_report(checks: Vec<ConformanceCheck>) -> ConformanceReport {
    let failed = checks
        .iter()
        .any(|check| check.status != ConformanceStatus::Pass);
    ConformanceReport {
        schema: CONFORMANCE_REPORT_SCHEMA.to_owned(),
        status: if failed {
            ConformanceStatus::Fail
        } else {
            ConformanceStatus::Pass
        },
        unavailable: None,
        checks,
    }
}

/// Run a fixture *file* against a snapshot directory: a missing or
/// unparseable fixture file is UNAVAILABLE evidence naming the file, then
/// [`run_fixture`] takes over.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_fixture_file(
    fixture_path: impl AsRef<Path>,
    snapshot_dir: impl AsRef<Path>,
) -> ConformanceReport {
    let fixture_path = fixture_path.as_ref();
    if !fixture_path.is_file() {
        return unavailable_report(format!(
            "conformance fixture {} is absent",
            fixture_path.display()
        ));
    }
    let text = match std::fs::read_to_string(fixture_path) {
        Ok(text) => text,
        Err(error) => {
            return unavailable_report(format!(
                "conformance fixture {} is unreadable: {error}",
                fixture_path.display()
            ));
        }
    };
    match parse_fixture_json(&text) {
        Ok(fixture) => run_fixture(&fixture, snapshot_dir),
        Err(error) => unavailable_report(format!(
            "conformance fixture {} is not a usable fixture: {error}",
            fixture_path.display()
        )),
    }
}
