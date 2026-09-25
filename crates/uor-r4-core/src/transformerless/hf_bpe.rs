//! Byte-level BPE tokenizer loaded from a Hugging Face `tokenizer.json`
//! (issue #242).
//!
//! The legacy [`scenarios::Tokenizer`](super::scenarios::Tokenizer) HF path
//! segmented text with iterative greedy pair merging preferring the lowest
//! MERGED TOKEN ID over the raw vocabulary, with no merges table. The
//! SmolLM2-Instruct teacher family uses byte-level BPE with an ORDERED
//! merges list: at every step the adjacent pair with the lowest merge RANK
//! is merged first. The two rules produce different segmentations, so
//! observation and evaluation were feeding the teacher token streams it
//! never saw. This module implements the teacher's actual rule:
//!
//! - `model.vocab` is the token → id map (tokens in the GPT-2 byte-level
//!   alphabet);
//! - `model.merges` is the ordered merge list — rank = list position;
//! - `added_tokens` are matched atomically (leftmost occurrence, longest
//!   content first) BEFORE pre-tokenization;
//! - pre-tokenization is the GPT-2 `ByteLevel` split (contractions, ` ?`
//!   letter/number/punctuation runs, whitespace runs keeping the final
//!   space attached to the following word), honoring `add_prefix_space`
//!   and an optional preceding `Digits` step (SmolLM2 declares
//!   `Digits { individual_digits: true }` before `ByteLevel`);
//! - each pre-token's UTF-8 bytes are mapped through the standard GPT-2
//!   byte-to-unicode table, merged rank-first to a fixpoint, and looked up
//!   in the vocabulary; merged results are cached per pre-token.
//!
//! The legacy tokenizer and its κ-pinned llama2.c baselines are untouched;
//! [`TokenizerKind`] lets the HF observation path select this
//! implementation while every legacy call site keeps its exact behavior.

use uor_r4_tokenizer::ByteBpeTokenizer;
pub use uor_r4_tokenizer::{TokenizerAdapter, TokenizerAdapterPolicy};

use super::scenarios::{
    RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable, RuntimeTokenizerEncodePolicy,
    RuntimeTokenizerIdentity, Tokenizer,
};

/// Byte-level BPE tokenizer with the core source-ingestion compatibility API.
/// Encoding, decoding and identity are implemented by the shared engine.
pub struct HfBpeTokenizer {
    engine: ByteBpeTokenizer,
}

impl HfBpeTokenizer {
    /// Parse the exact shared byte-level BPE format.
    pub fn from_tokenizer_json_bytes(bytes: &[u8]) -> Option<Self> {
        ByteBpeTokenizer::from_tokenizer_json_bytes(bytes).map(|engine| Self { engine })
    }

    /// Load `tokenizer.json` from a Hugging Face model snapshot directory.
    ///
    /// Host-ingestion boundary: a missing file or a `tokenizer.json` that is not
    /// a valid byte-level BPE tokenizer both report the sanctioned
    /// [`uor_r4_model_source::SourceUnavailable`] — the tokenizer source could
    /// not be ingested — carrying the path/diagnostic.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_dir(dir: &std::path::Path) -> Result<Self, uor_r4_model_source::SourceUnavailable> {
        let path = dir.join("tokenizer.json");
        let bytes = std::fs::read(&path)?;
        Self::from_tokenizer_json_bytes(&bytes).ok_or_else(|| {
            uor_r4_model_source::SourceUnavailable::new(format!(
                "{}: not a valid byte-level BPE tokenizer",
                path.display()
            ))
        })
    }

    /// Encode without automatic BOS/EOS insertion.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        self.engine.encode(text)
    }

    /// Encode with the historical replaced-character count.
    pub fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        self.engine.encode_lossy(text)
    }

    /// Decode through lossy UTF-8 after preserving raw token bytes.
    pub fn decode(&self, ids: &[u32]) -> String {
        self.engine.decode(ids)
    }

    /// Decode token ids to their exact bytes before UTF-8 rendering.
    pub fn decode_bytes(&self, ids: &[u32]) -> Vec<u8> {
        self.engine.decode_bytes(ids)
    }

    /// Number of id slots, including added tokens.
    pub fn vocab_size(&self) -> usize {
        self.engine.vocab_size()
    }

    /// Raw byte lengths indexed by token id.
    pub fn token_byte_lengths(&self) -> Vec<u32> {
        self.engine.token_byte_lengths()
    }

    /// Content address of the original tokenizer JSON bytes.
    pub fn address(&self) -> String {
        self.engine.address()
    }

    /// Stable versioned adapter identity, with unchanged canonical bytes.
    pub fn adapter(&self) -> TokenizerAdapter {
        self.engine.adapter()
    }
}

/// The behavior a registered tokenizer family provides once resolved
/// from the versioned adapter registry (#639-2): encode/decode plus the
/// versioned identity it declares. Object-safe by construction, so
/// [`adapter_constructor`] can hand back a `Box<dyn TokenizerModel>` for
/// any family — a family can encode and decode without being an
/// [`HfBpeTokenizer`]. `hf-byte-bpe/1` implements it by delegating to its
/// inherent methods, so its behavior is byte-unchanged; the recorded
/// SentencePiece/Unigram follow-up (#639-3) implements the same trait
/// rather than a new concrete return type.
pub trait TokenizerModel: Send + Sync {
    /// Encode text to token ids (the family's exact encode path).
    fn encode(&self, text: &str) -> Vec<u32>;
    /// Lossy encode: ids plus the count of characters the family could not
    /// represent (zero for lossless families such as byte-level BPE).
    fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64);
    /// Decode ids to lossy-UTF-8 text.
    fn decode(&self, ids: &[u32]) -> String;
    /// Number of id slots (max assigned id + 1).
    fn vocab_size(&self) -> usize;
    /// The versioned adapter-identity record (#601) this family declares.
    fn adapter(&self) -> TokenizerAdapter;
    /// Family-neutral per-id decode table for the deployed runtime. This is
    /// deliberately distinct from original-source byte anchors.
    fn runtime_decode_table(&self) -> RuntimeTokenizerDecodeTable;
}

impl TokenizerModel for HfBpeTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        HfBpeTokenizer::encode(self, text)
    }
    fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        HfBpeTokenizer::encode_lossy(self, text)
    }
    fn decode(&self, ids: &[u32]) -> String {
        HfBpeTokenizer::decode(self, ids)
    }
    fn vocab_size(&self) -> usize {
        HfBpeTokenizer::vocab_size(self)
    }
    fn adapter(&self) -> TokenizerAdapter {
        HfBpeTokenizer::adapter(self)
    }
    fn runtime_decode_table(&self) -> RuntimeTokenizerDecodeTable {
        let pieces: Vec<Vec<u8>> = (0..self.engine.model_vocab_size())
            .map(|id| self.decode_bytes(&[id as u32]))
            .collect();
        let source_byte_lengths = Some(pieces.iter().map(|piece| piece.len() as u32).collect());
        let adapter = self.adapter();
        RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: adapter.family,
                version: adapter.version,
                tokenizer_cid: adapter.tokenizer_cid,
                adapter_digest: adapter.adapter_digest,
            },
            pieces,
            encode_policy: RuntimeTokenizerEncodePolicy::LegacyCompatible,
            decode_policy: RuntimeTokenizerDecodePolicy::Concatenate,
            source_byte_lengths,
        }
    }
}

/// A registered adapter constructor: parse raw tokenizer definition bytes
/// into the family's [`TokenizerModel`]. Parse failures retain their focused
/// [`uor_r4_model_source::SourceUnavailable`] diagnostic rather than being
/// collapsed to `None` at the registry boundary.
#[cfg(not(target_arch = "wasm32"))]
pub type AdapterConstructor =
    fn(&[u8]) -> Result<Box<dyn TokenizerModel>, uor_r4_model_source::SourceUnavailable>;

/// The versioned tokenizer-adapter registry (#601): map `(family,
/// version)` to the constructor that implements it. Registered families are
/// `hf-byte-bpe/1` ([`HfBpeTokenizer`]) and, since #639-3b,
/// `sentencepiece-unigram/1` and `sentencepiece-unigram/2`
/// ([`super::sentencepiece::SentencePieceUnigramTokenizer`]). Every pair
/// outside the registry — including a bumped version of a registered family —
/// is refused by name on the sanctioned
/// [`uor_r4_model_source::SourceUnavailable`] surface
/// (`SourceIngestKind::UnknownTokenizerAdapter`), matching the module's
/// existing host-ingestion convention ([`HfBpeTokenizer::from_dir`])
/// and the #600 geometry registry: never guessed, never approximated
/// by a "closest" family or version.
#[cfg(not(target_arch = "wasm32"))]
pub fn adapter_constructor(
    family: &str,
    version: u32,
) -> Result<AdapterConstructor, uor_r4_model_source::SourceUnavailable> {
    match (family, version) {
        (TokenizerAdapter::HF_BYTE_BPE_FAMILY, TokenizerAdapter::HF_BYTE_BPE_VERSION) => {
            Ok(|bytes| {
                HfBpeTokenizer::from_tokenizer_json_bytes(bytes)
                    .map(|tokenizer| Box::new(tokenizer) as Box<dyn TokenizerModel>)
                    .ok_or_else(|| {
                        uor_r4_model_source::SourceUnavailable::new(
                            "tokenizer.json: not a valid hf-byte-bpe/1 definition",
                        )
                    })
            })
        }
        (
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V1_VERSION,
        ) => Ok(|bytes| {
            super::sentencepiece::SentencePieceUnigramTokenizer::from_spiece_bytes_v1(bytes)
                .map(|tokenizer| Box::new(tokenizer) as Box<dyn TokenizerModel>)
        }),
        (
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V2_VERSION,
        ) => Ok(|bytes| {
            super::sentencepiece::SentencePieceUnigramTokenizer::from_spiece_bytes_v2(bytes)
                .map(|tokenizer| Box::new(tokenizer) as Box<dyn TokenizerModel>)
        }),
        _ => Err(
            uor_r4_model_source::SourceIngestKind::UnknownTokenizerAdapter {
                family: family.to_owned(),
                version,
            }
            .into(),
        ),
    }
}

/// Explicit key for selecting one tokenizer definition from a source that
/// presents multiple files. Selection is always family + version; a bare file
/// preference is never inferred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizerAdapterKey {
    pub family: String,
    pub version: u32,
}

impl TokenizerAdapterKey {
    pub fn new(family: impl Into<String>, version: u32) -> Self {
        Self {
            family: family.into(),
            version,
        }
    }

    pub fn hf_byte_bpe_v1() -> Self {
        Self::new(
            TokenizerAdapter::HF_BYTE_BPE_FAMILY,
            TokenizerAdapter::HF_BYTE_BPE_VERSION,
        )
    }

    pub fn sentencepiece_unigram_v1() -> Self {
        Self::new(
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V1_VERSION,
        )
    }

    pub fn sentencepiece_unigram_v2() -> Self {
        Self::new(
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V2_VERSION,
        )
    }

    pub fn sentencepiece_unigram_current() -> Self {
        Self::sentencepiece_unigram_v2()
    }
}

/// Resolve the exact registered tokenizer declared by a source snapshot.
///
/// Auto-selection is permitted only when exactly one supported definition
/// file is present. A snapshot containing both `tokenizer.json` and
/// `spiece.model` requires an explicit [`TokenizerAdapterKey`], so Hugging
/// Face wrapper semantics can never be silently substituted for raw
/// SentencePiece semantics. Legacy tokenizer.bin is deliberately outside
/// this resolver and remains available only through its explicit checkpoint
/// path.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_source_tokenizer(
    dir: &std::path::Path,
    selection: Option<&TokenizerAdapterKey>,
) -> Result<TokenizerKind, uor_r4_model_source::SourceUnavailable> {
    let tokenizer_json = dir.join("tokenizer.json");
    let spiece_model = dir.join("spiece.model");
    let has_bpe_definition = tokenizer_definition_present(&tokenizer_json)?;
    let has_sentencepiece_definition = tokenizer_definition_present(&spiece_model)?;

    let selected = match selection {
        Some(key) => key.clone(),
        None => match (has_bpe_definition, has_sentencepiece_definition) {
            (true, false) => TokenizerAdapterKey::hf_byte_bpe_v1(),
            (false, true) => TokenizerAdapterKey::sentencepiece_unigram_current(),
            (true, true) => {
                return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                    "{} presents both tokenizer.json and spiece.model; select an explicit \
                     tokenizer adapter family/version",
                    dir.display()
                )));
            }
            (false, false) => {
                return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                    "{} has no registered tokenizer definition (expected tokenizer.json or \
                     spiece.model)",
                    dir.display()
                )));
            }
        },
    };

    // Resolve the registry key first so an unknown version is refused by its
    // structured name even when a similarly named file happens to exist.
    let constructor = adapter_constructor(&selected.family, selected.version)?;
    let definition = match (selected.family.as_str(), selected.version) {
        (TokenizerAdapter::HF_BYTE_BPE_FAMILY, TokenizerAdapter::HF_BYTE_BPE_VERSION) => {
            &tokenizer_json
        }
        (
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY,
            TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V1_VERSION
            | TokenizerAdapter::SENTENCEPIECE_UNIGRAM_V2_VERSION,
        ) => &spiece_model,
        // Defensive registry/mapping drift guard: this is a public ingestion
        // boundary, so even an internal metadata inconsistency is a focused
        // refusal rather than a panic.
        _ => {
            return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                "tokenizer adapter {}/{} is registered but has no source-definition mapping",
                selected.family, selected.version
            )));
        }
    };
    let bytes = std::fs::read(definition).map_err(|error| {
        uor_r4_model_source::SourceUnavailable::new(format!(
            "{} selected as {}/{}: {error}",
            definition.display(),
            selected.family,
            selected.version
        ))
    })?;
    let tokenizer =
        constructor(&bytes).map_err(|error| uor_r4_model_source::SourceUnavailable {
            reason: format!(
                "{} selected as {}/{}: {}",
                definition.display(),
                selected.family,
                selected.version,
                error.reason
            ),
            kind: error.kind,
        })?;
    Ok(TokenizerKind::Registered(tokenizer))
}

#[cfg(not(target_arch = "wasm32"))]
fn tokenizer_definition_present(
    path: &std::path::Path,
) -> Result<bool, uor_r4_model_source::SourceUnavailable> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => match std::fs::metadata(path) {
            Ok(target) if target.is_file() => Ok(true),
            Ok(_) => Err(uor_r4_model_source::SourceUnavailable::new(format!(
                "{} is a symlink to a non-regular tokenizer definition",
                path.display()
            ))),
            Err(error) => Err(uor_r4_model_source::SourceUnavailable::new(format!(
                "{} is a dangling or unreadable tokenizer-definition symlink: {error}",
                path.display()
            ))),
        },
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(uor_r4_model_source::SourceUnavailable::new(format!(
            "{} exists but is not a regular tokenizer-definition file",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(uor_r4_model_source::SourceUnavailable::new(format!(
            "{} tokenizer definition cannot be inspected: {error}",
            path.display()
        ))),
    }
}

/// Tokenizer selector for observation drivers: the legacy llama2.c
/// tokenizer keeps its exact behavior (κ-pinned baselines); every registered
/// family shares the boxed [`TokenizerModel`] path.
pub enum TokenizerKind {
    Legacy(Tokenizer),
    Registered(Box<dyn TokenizerModel>),
}

impl TokenizerKind {
    pub fn encode(&self, text: &str) -> Vec<u32> {
        match self {
            TokenizerKind::Legacy(tokenizer) => tokenizer.encode(text),
            TokenizerKind::Registered(tokenizer) => tokenizer.encode(text),
        }
    }

    pub fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        match self {
            TokenizerKind::Legacy(tokenizer) => tokenizer.encode_lossy(text),
            TokenizerKind::Registered(tokenizer) => tokenizer.encode_lossy(text),
        }
    }

    pub fn decode(&self, ids: &[u32]) -> String {
        match self {
            TokenizerKind::Legacy(tokenizer) => tokenizer.decode(ids),
            TokenizerKind::Registered(tokenizer) => tokenizer.decode(ids),
        }
    }

    pub fn vocab_size(&self) -> usize {
        match self {
            TokenizerKind::Legacy(tokenizer) => tokenizer.vocab.len(),
            TokenizerKind::Registered(tokenizer) => tokenizer.vocab_size(),
        }
    }

    /// The versioned adapter-identity record (#601) this selection
    /// resolves to: `Some` for the HF byte-level BPE path, `None` for
    /// the legacy llama2.c tokenizer, which predates adapter records
    /// and stays exactly as-is (its κ-pinned baselines and manifest
    /// bytes are unchanged when no adapter is recorded).
    pub fn adapter(&self) -> Option<TokenizerAdapter> {
        match self {
            TokenizerKind::Legacy(_) => None,
            TokenizerKind::Registered(tokenizer) => Some(tokenizer.adapter()),
        }
    }

    pub fn registered(&self) -> Option<&dyn TokenizerModel> {
        match self {
            TokenizerKind::Legacy(_) => None,
            TokenizerKind::Registered(tokenizer) => Some(tokenizer.as_ref()),
        }
    }

    pub fn runtime_decode_table(&self) -> Option<RuntimeTokenizerDecodeTable> {
        self.registered().map(TokenizerModel::runtime_decode_table)
    }
}
