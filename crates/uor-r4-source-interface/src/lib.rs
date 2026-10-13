//! Host source-ingestion metadata only. No executor, numerical operator or model dependency.
//! Extracted without changing public error/type identity; model-source re-exports these types.

/// The sanctioned host-ingestion boundary error (R5).
///
/// A declared external source artifact — a teacher directory, its
/// `model.safetensors`, `config.json`, an indexed shard set, or a named
/// tensor within — could not be ingested into a valid teacher at
/// construction. This is the host-side analogue of graph-format's
/// `NotAProduct`: the only reportable condition is that the requested object
/// does not exist as a valid product, reported at construction. It carries
/// the operator-facing diagnostic (which file, which tensor, what dtype)
/// rather than discarding it, and — since #598 — the structured
/// [`SourceIngestKind`] classifying which ingestion check refused the
/// artifact, so callers and tests can assert the exact failure class without
/// this crate growing a second error surface.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    /// Human-facing description of why the source could not be ingested.
    pub reason: String,
    /// The #598 ingestion failure class, when a validation-boundary check
    /// produced this error; [`SourceIngestKind::Unspecified`] otherwise
    /// (I/O, JSON, or legacy construction sites).
    pub kind: SourceIngestKind,
}

#[cfg(not(target_arch = "wasm32"))]
impl SourceUnavailable {
    /// Construct from any displayable reason (failure class
    /// [`SourceIngestKind::Unspecified`]).
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
            kind: SourceIngestKind::Unspecified,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source unavailable: {}", self.reason)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for SourceUnavailable {}

#[cfg(not(target_arch = "wasm32"))]
impl From<std::io::Error> for SourceUnavailable {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<serde_json::Error> for SourceUnavailable {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<safetensors::SafeTensorError> for SourceUnavailable {
    fn from(error: safetensors::SafeTensorError) -> Self {
        Self::new(error)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<SourceIngestKind> for SourceUnavailable {
    fn from(kind: SourceIngestKind) -> Self {
        Self {
            reason: kind.to_string(),
            kind,
        }
    }
}

/// Filesystem-free counterpart of the sanctioned source-unavailable error.
/// Browser serving never performs host ingestion, but portable production
/// admission still needs the same typed failure boundary for malformed or
/// mismatched in-memory components.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct SourceUnavailable {
    pub reason: String,
}

#[cfg(target_arch = "wasm32")]
impl SourceUnavailable {
    pub fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for SourceUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source unavailable: {}", self.reason)
    }
}

#[cfg(target_arch = "wasm32")]
impl std::error::Error for SourceUnavailable {}

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

/// The #598 source-ingestion failure class: one variant per check at the
/// [`SafetensorsSnapshot::open`] validation boundary, all detected before
/// any model construction. This is NOT a second error surface — the one
/// sanctioned host-ingestion error remains [`SourceUnavailable`] (R5);
/// this enum rides inside it as the structured `kind` field, so each
/// variant names the offending object and callers/tests can still assert
/// the exact failure class.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceIngestKind {
    /// No structured ingestion class: the error came from a plain I/O,
    /// JSON, or legacy construction site ([`SourceUnavailable::new`]).
    Unspecified,
    /// A snapshot file exists but could not be read or parsed (I/O error,
    /// malformed JSON header or index, non-portable shard name).
    Unreadable {
        /// The file (or file: tensor context) that failed.
        path: String,
        /// The underlying parse or I/O diagnostic.
        reason: String,
    },
    /// A shard file referenced by the index (or the single
    /// `model.safetensors`) is absent from the snapshot directory.
    MissingShardFile {
        /// The missing shard's file name.
        shard: String,
    },
    /// A tensor is declared (by the index or by the model geometry) but the
    /// shard that should carry it does not.
    MissingTensor {
        /// The declared tensor name.
        tensor: String,
        /// Where the tensor was expected (a shard file, or the index/single
        /// file for a geometry-required tensor missing everywhere).
        shard: String,
    },
    /// A tensor is claimed more than once: two shard headers both declare
    /// it, or the raw index JSON maps the same name twice.
    DuplicateTensor {
        /// The doubly-claimed tensor name.
        tensor: String,
        /// The first claimant (shard file, or the index file itself).
        first_shard: String,
        /// The second claimant.
        second_shard: String,
    },
    /// A shard header declares a tensor the index does not assign to any
    /// shard. Only possible in indexed mode; a single-file snapshot has no
    /// index, so extra tensors there are ignored exactly as before #598.
    UnexpectedTensor {
        /// The unindexed tensor name.
        tensor: String,
        /// The shard whose header declares it.
        shard: String,
    },
    /// A geometry-required tensor's declared shape does not match the shape
    /// `config.json` implies.
    ShapeMismatch {
        /// The tensor name.
        tensor: String,
        /// The shape the model geometry requires.
        expected: Vec<usize>,
        /// The shape the shard header declares.
        actual: Vec<usize>,
    },
    /// A declared byte length is inconsistent: a tensor's data span differs
    /// from shape × dtype size, the data spans are not contiguous, or the
    /// shard file's size differs from what its header claims.
    ByteLengthMismatch {
        /// Which check failed (shard file, or shard file + tensor).
        context: String,
        /// The byte length the header/shape claims.
        expected: u64,
        /// The byte length actually found.
        actual: u64,
    },
    /// The same tensor name is declared with two different dtypes.
    DtypeInconsistency {
        /// The tensor name.
        tensor: String,
        /// First declaration, as `DTYPE (in <shard>)`.
        first: String,
        /// Conflicting declaration, as `DTYPE (in <shard>)`.
        second: String,
    },
    /// A tensor declares a dtype outside BF16/F16/F32 — e.g. quantized I8,
    /// U8, or GPTQ/AWQ-style packed formats. Rejected by name, never
    /// silently approximated (#598 non-goal).
    UnsupportedDtype {
        /// The tensor name.
        tensor: String,
        /// The declared dtype/format string from the shard header.
        dtype: String,
    },
    /// #599 adapter-conformance gate: the parsed `config.json` declares a
    /// feature outside the adapter's typed
    /// [`conformance::AdapterFeatures`] declaration — an activation, norm
    /// epsilon, RoPE mode, head geometry, bias, embedding-tying, or token
    /// policy the source executor would otherwise silently misinterpret.
    /// Detected at oracle construction, before any tensor is read and
    /// before any observation can be generated (fail-closed).
    UnsupportedConfigFeature {
        /// Which declared feature the configuration falls outside of.
        feature: AdapterFeature,
        /// What the adapter declares it supports for this feature.
        declared: String,
        /// What `config.json` actually contains.
        actual: String,
    },
    /// #600 geometry registry: a caller named a source→compiled geometry
    /// projection `(id, version)` outside the versioned registry
    /// ([`geometry::projection_implementation`]), so no implementation
    /// exists to interpret it. Refused by name, never approximated by a
    /// "closest" algorithm or version.
    UnknownGeometryProjection {
        /// The requested projection algorithm id.
        id: String,
        /// The requested projection version.
        version: u32,
    },
    /// #601 tokenizer-adapter registry: a caller named a tokenizer
    /// adapter `(family, version)` outside the versioned registry
    /// (`uor_r4_core::transformerless::hf_bpe::adapter_constructor`),
    /// so no constructor exists to interpret it. Refused by name, never
    /// approximated by a "closest" family or version. Registered entries
    /// include `hf-byte-bpe/1` plus frozen `sentencepiece-unigram/1` and
    /// current `sentencepiece-unigram/2`; any unregistered family/version pair
    /// is rejected here.
    UnknownTokenizerAdapter {
        /// The requested adapter family.
        family: String,
        /// The requested adapter version.
        version: u32,
    },
    /// #602 attention-operator registry: a caller named a source
    /// attention operator `(id, version)` outside the versioned registry
    /// ([`attention::operator_spec`]), so no specification exists to
    /// interpret it. Refused by name, never approximated by a "closest"
    /// operator or version.
    UnknownAttentionOperator {
        /// The requested operator id.
        id: String,
        /// The requested operator version.
        version: u32,
    },
    /// Source-dense operator registry: a caller named an `(id, version)`
    /// outside [`dense::operator_spec`]. Unknown entries are never
    /// approximated by a nearby arithmetic version.
    UnknownDenseOperator {
        /// The requested dense operator id.
        id: String,
        /// The requested dense operator version.
        version: u32,
    },
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for SourceIngestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unspecified => write!(f, "no structured ingestion class"),
            Self::Unreadable { path, reason } => write!(f, "{path}: {reason}"),
            Self::MissingShardFile { shard } => {
                write!(
                    f,
                    "shard file {shard} is absent from the snapshot directory"
                )
            }
            Self::MissingTensor { tensor, shard } => {
                write!(f, "tensor {tensor} is declared but absent from {shard}")
            }
            Self::DuplicateTensor {
                tensor,
                first_shard,
                second_shard,
            } => write!(
                f,
                "tensor {tensor} is claimed twice ({first_shard} and {second_shard})"
            ),
            Self::UnexpectedTensor { tensor, shard } => write!(
                f,
                "tensor {tensor} appears in shard {shard} but not in the index"
            ),
            Self::ShapeMismatch {
                tensor,
                expected,
                actual,
            } => write!(
                f,
                "tensor {tensor} has shape {actual:?}, expected {expected:?}"
            ),
            Self::ByteLengthMismatch {
                context,
                expected,
                actual,
            } => write!(
                f,
                "byte-length mismatch at {context}: expected {expected} bytes, found {actual}"
            ),
            Self::DtypeInconsistency {
                tensor,
                first,
                second,
            } => write!(f, "tensor {tensor} is declared {first} and {second}"),
            Self::UnsupportedDtype { tensor, dtype } => write!(
                f,
                "tensor {tensor} is {dtype}; supported source dtypes are BF16, F16, F32 \
                 (quantized formats are rejected, not approximated)"
            ),
            Self::UnsupportedConfigFeature {
                feature,
                declared,
                actual,
            } => write!(
                f,
                "config.json feature {feature} is outside the adapter's declared support: \
                 adapter declares {declared}, configuration has {actual} \
                 (rejected before any observation)"
            ),
            Self::UnknownGeometryProjection { id, version } => write!(
                f,
                "geometry projection {id}/{version} is not in the versioned projection \
                 registry (known: {}/{})",
                "bucket-average", 1,
            ),
            Self::UnknownTokenizerAdapter { family, version } => write!(
                f,
                "tokenizer adapter {family}/{version} is not in the versioned adapter \
                 registry (registered entries include hf-byte-bpe/1, \
                 sentencepiece-unigram/1, and sentencepiece-unigram/2; unknown families or \
                 versions are never approximated)"
            ),
            Self::UnknownAttentionOperator { id, version } => write!(
                f,
                "attention operator {id}/{version} is not in the versioned operator \
                 registry (registered entries include {}/{}, {}/{}, {}/{}, {}/{}, \
                 {}/{}, {}/{}, and {}/{})",
                "standard-source-attention",
                1,
                "standard-source-attention",
                2,
                "experimental-r4-source-attention",
                1,
                "experimental-r4-source-attention",
                2,
                "learned-absolute-source-attention",
                1,
                "learned-absolute-source-attention",
                2,
                "r4-route-attention",
                1,
            ),
            Self::UnknownDenseOperator { id, version } => write!(
                f,
                "dense operator {id}/{version} is not in the versioned operator registry \
                 (registered entries: {}/{}, {}/{})",
                "gpt2-source-dense", 1, "gpt2-source-dense", 2,
            ),
        }
    }
}
