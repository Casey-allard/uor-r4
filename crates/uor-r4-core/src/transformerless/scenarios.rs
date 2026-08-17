//! The SCENARIO SUITE: comprehensive comparison over diverse, real-world
//! input, complementing the aggregate rows of `compare` and
//! docs/COMPARISON.md.
//!
//! Scenario classes:
//!   - in-domain prompts: story openings a user would actually type at a
//!     TinyStories-class model; agreement measured along the TEACHER'S OWN
//!     greedy trajectory (the deployment question: "would the artifact
//!     have produced the same continuation, token by token?");
//!   - out-of-domain prompts: questions, instructions, business prose —
//!     real-world inputs this source model was never meant for; both
//!     systems are out of domain and the comparison is relative;
//!   - real human-written text (not model-sampled): an in-domain-style
//!     story and a Shakespeare passage (fully out-of-domain), scored
//!     token-by-token against the ACTUAL next token for both systems, plus
//!     artifact↔teacher agreement;
//!   - structural stress: repetition, a one-word prompt, and a cold start
//!     from BOS alone.
//!
//! Rules of the suite: the tokenizer is validated in-run (round-trip
//! witness plus a fluency gate on the teacher's continuation — a broken
//! encoding cannot pass it); every scenario feeds teacher and artifact the
//! IDENTICAL token stream; the artifact uses the same store and the same
//! runtime code path certified in PROOF.md (scenario text is fully unseen
//! by the store, which was built from the training split only); quality
//! and throughput are reported together, per scenario class.
//!
//! Classical runtimes (llama.cpp et al.) execute the source model itself,
//! so their scenario-level predictions coincide with the teacher rows by
//! definition; their throughput is in docs/COMPARISON.md.

use super::compiler::Corpus;
use crate::transformerless::compiler;
use crate::transformerless::runtime::{
    build_store, code_plain, derive_rotations, predict_plain, Store,
};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use uor_r4_model_source::TeacherOracle;

const MAX_TOKEN_BYTES: usize = 1024;

/// Prefix of the additive, tagged runtime-tokenizer container. Interpreted as
/// an old-format little-endian `i32` token length, the first four bytes are
/// negative, so the tagged representation is disjoint from every valid
/// historical tokenizer.bin.
const TAGGED_TOKENIZER_MAGIC: &[u8; 8] = b"R4T\xffTOK1";
const TAGGED_TOKENIZER_FORMAT_VERSION: u32 = 1;
const TAGGED_ENCODE_UNAVAILABLE: u32 = 1;
const TAGGED_DECODE_SENTENCEPIECE: u32 = 1;
/// Adapter identity whose UNKNOWN table entry may begin with an intentional
/// ASCII space. Its dummy-prefix removal therefore acts on U+2581 before
/// whitespace expansion, unlike the frozen `/1` decoder.
const SENTENCEPIECE_REFERENCE_UNK_FAMILY: &str = "sentencepiece-unigram";
const SENTENCEPIECE_REFERENCE_UNK_VERSION: u32 = 2;
const MAX_TAGGED_FAMILY_BYTES: usize = 128;
const MAX_TAGGED_VOCAB_SIZE: usize = 1 << 24;

fn is_blake3_address(value: &str) -> bool {
    value.len() == "blake3:".len() + 64
        && value.starts_with("blake3:")
        && value["blake3:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(not(target_arch = "wasm32"))]
fn write_tagged_identity(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), uor_r4_model_source::SourceUnavailable> {
    let length = u32::try_from(value.len()).map_err(|_| {
        uor_r4_model_source::SourceUnavailable::new("runtime tokenizer identity field too long")
    })?;
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

/// Whether a runtime decode table can also use the historical tokenizer.bin
/// encoder. `Unavailable` is an explicit deployed-runtime boundary: the exact
/// registered host tokenizer is required to encode prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTokenizerEncodePolicy {
    LegacyCompatible,
    Unavailable,
}

/// How a sequence of raw per-id table entries is decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTokenizerDecodePolicy {
    /// Concatenate the already-decoded token bytes.
    Concatenate,
    /// SentencePiece surface decoding: replace U+2581 with a space and remove
    /// the single leading space introduced by the declared dummy prefix.
    SentencePiece { strip_dummy_prefix: bool },
}

/// Family-neutral table passed from a registered host tokenizer to the
/// runtime-tokenizer exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTokenizerIdentity {
    pub family: String,
    pub version: u32,
    /// Content address of the exact host tokenizer definition (for example,
    /// raw `spiece.model`), distinct from the exported tokenizer.bin CID.
    pub tokenizer_cid: String,
    /// Digest of the complete versioned adapter identity and policy.
    pub adapter_digest: String,
}

/// Family-neutral table passed from a registered host tokenizer to the
/// runtime-tokenizer exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTokenizerDecodeTable {
    pub identity: RuntimeTokenizerIdentity,
    pub pieces: Vec<Vec<u8>>,
    pub encode_policy: RuntimeTokenizerEncodePolicy,
    pub decode_policy: RuntimeTokenizerDecodePolicy,
    /// Source byte anchors are a separate semantic from runtime decode-piece
    /// lengths. SentencePiece normalization does not preserve source offsets.
    pub source_byte_lengths: Option<Vec<u32>>,
}

/// Evidence returned by the family-neutral runtime-tokenizer export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTokenizerExport {
    /// Raw byte length of each exported decode-table entry.
    pub decode_byte_lengths: Vec<u32>,
    /// Per-token original-source byte lengths, only when the host adapter can
    /// truthfully provide them.
    pub source_byte_lengths: Option<Vec<u32>>,
}

/// Format an instructional query into native ChatML / instruct template format for teacher models:
/// `<|im_start|>system\n{system_prompt}<|im_end|>\n<|im_start|>user\n{user_prompt}<|im_end|>\n<|im_start|>assistant\n`
pub fn format_instruct_chat_prompt(system_prompt: Option<&str>, user_prompt: &str) -> String {
    let system = system_prompt.unwrap_or("You are a helpful AI assistant.");
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user_prompt}<|im_end|>\n<|im_start|>assistant\n"
    )
}

/// Convert a Hugging Face byte-level BPE vocabulary into the compact token
/// table consumed by the allocation-free runtime tokenizer.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_hf_bytelevel_tokenizer(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<(), uor_r4_model_source::SourceUnavailable> {
    export_hf_bytelevel_tokenizer_with_lengths(source, destination).map(|_| ())
}

/// Export the runtime tokenizer and return per-token UTF-8 byte lengths for
/// observation byte-anchor generation.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_hf_bytelevel_tokenizer_with_lengths(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<Vec<u32>, uor_r4_model_source::SourceUnavailable> {
    let tokens = hf_bytelevel_tokens(source)?;
    let lengths = tokens
        .iter()
        .map(|token| {
            u32::try_from(token.len())
                .map_err(|_| uor_r4_model_source::SourceUnavailable::new("token too long"))
        })
        .collect::<Result<Vec<_>, uor_r4_model_source::SourceUnavailable>>()?;
    let mut bytes = Vec::new();
    for token in tokens {
        let length = i32::try_from(token.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "token too long"))?;
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(&token);
    }
    std::fs::write(destination, bytes)?;
    Ok(lengths)
}

/// Export a registered host tokenizer's family-neutral decode table.
///
/// Byte-level BPE keeps the historical untagged bytes exactly. A tokenizer
/// whose encoder is unavailable in the deployed runtime uses the additive
/// tagged representation; currently the only such registered policy is raw
/// SentencePiece surface decoding.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_registered_runtime_tokenizer(
    tokenizer: &dyn super::hf_bpe::TokenizerModel,
    destination: impl AsRef<Path>,
) -> Result<RuntimeTokenizerExport, uor_r4_model_source::SourceUnavailable> {
    let table = tokenizer.runtime_decode_table();
    export_runtime_tokenizer_table(&table, destination)
}

/// Export a precomputed decode table. Split from
/// [`export_registered_runtime_tokenizer`] so compiler callers can build and
/// validate the table before performing the filesystem mutation.
#[cfg(not(target_arch = "wasm32"))]
pub fn export_runtime_tokenizer_table(
    table: &RuntimeTokenizerDecodeTable,
    destination: impl AsRef<Path>,
) -> Result<RuntimeTokenizerExport, uor_r4_model_source::SourceUnavailable> {
    let mut decode_byte_lengths = Vec::with_capacity(table.pieces.len());
    for piece in &table.pieces {
        let length = u32::try_from(piece.len()).map_err(|_| {
            uor_r4_model_source::SourceUnavailable::new("runtime tokenizer piece too long")
        })?;
        decode_byte_lengths.push(length);
    }

    if let Some(source_lengths) = &table.source_byte_lengths {
        if source_lengths.len() != table.pieces.len() {
            return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                "runtime tokenizer source-length table has {} entries for {} pieces",
                source_lengths.len(),
                table.pieces.len()
            )));
        }
    }

    let mut bytes = Vec::new();
    match (table.encode_policy, table.decode_policy) {
        (
            RuntimeTokenizerEncodePolicy::LegacyCompatible,
            RuntimeTokenizerDecodePolicy::Concatenate,
        ) => {
            // This is intentionally the exact historical record loop used by
            // `export_hf_bytelevel_tokenizer_with_lengths`.
            for piece in &table.pieces {
                let length = i32::try_from(piece.len())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "token too long"))?;
                bytes.extend_from_slice(&length.to_le_bytes());
                bytes.extend_from_slice(piece);
            }
        }
        (
            RuntimeTokenizerEncodePolicy::Unavailable,
            RuntimeTokenizerDecodePolicy::SentencePiece { strip_dummy_prefix },
        ) => {
            if table
                .pieces
                .iter()
                .any(|piece| std::str::from_utf8(piece).is_err())
            {
                return Err(uor_r4_model_source::SourceUnavailable::new(
                    "SentencePiece runtime tokenizer pieces must be valid UTF-8",
                ));
            }
            if !is_blake3_address(&table.identity.tokenizer_cid)
                || !is_blake3_address(&table.identity.adapter_digest)
            {
                return Err(uor_r4_model_source::SourceUnavailable::new(
                    "runtime tokenizer identity requires blake3 tokenizer_cid and adapter_digest",
                ));
            }
            let family = table.identity.family.as_bytes();
            if family.is_empty() || family.len() > MAX_TAGGED_FAMILY_BYTES {
                return Err(uor_r4_model_source::SourceUnavailable::new(
                    "runtime tokenizer family is empty or too long",
                ));
            }
            if table.pieces.len() > MAX_TAGGED_VOCAB_SIZE {
                return Err(uor_r4_model_source::SourceUnavailable::new(
                    "runtime tokenizer vocabulary exceeds the tagged format limit",
                ));
            }
            let family_len = u32::try_from(family.len()).map_err(|_| {
                uor_r4_model_source::SourceUnavailable::new("runtime tokenizer family too long")
            })?;
            let piece_count = u32::try_from(table.pieces.len()).map_err(|_| {
                uor_r4_model_source::SourceUnavailable::new("runtime tokenizer has too many ids")
            })?;
            bytes.extend_from_slice(TAGGED_TOKENIZER_MAGIC);
            bytes.extend_from_slice(&TAGGED_TOKENIZER_FORMAT_VERSION.to_le_bytes());
            bytes.extend_from_slice(&TAGGED_ENCODE_UNAVAILABLE.to_le_bytes());
            bytes.extend_from_slice(&TAGGED_DECODE_SENTENCEPIECE.to_le_bytes());
            bytes.extend_from_slice(&u32::from(strip_dummy_prefix).to_le_bytes());
            bytes.extend_from_slice(&table.identity.version.to_le_bytes());
            bytes.extend_from_slice(&family_len.to_le_bytes());
            bytes.extend_from_slice(family);
            write_tagged_identity(&mut bytes, &table.identity.tokenizer_cid)?;
            write_tagged_identity(&mut bytes, &table.identity.adapter_digest)?;
            bytes.extend_from_slice(&piece_count.to_le_bytes());
            for piece in &table.pieces {
                if piece.len() > MAX_TOKEN_BYTES {
                    return Err(uor_r4_model_source::SourceUnavailable::new(
                        "runtime tokenizer piece exceeds the tagged format limit",
                    ));
                }
                let length = u32::try_from(piece.len()).map_err(|_| {
                    uor_r4_model_source::SourceUnavailable::new("runtime tokenizer piece too long")
                })?;
                bytes.extend_from_slice(&length.to_le_bytes());
                bytes.extend_from_slice(piece);
            }
        }
        _ => {
            return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                "unsupported runtime tokenizer policies: encode={:?}, decode={:?}",
                table.encode_policy, table.decode_policy
            )));
        }
    }
    std::fs::write(destination, bytes)?;
    Ok(RuntimeTokenizerExport {
        decode_byte_lengths,
        source_byte_lengths: table.source_byte_lengths.clone(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn hf_bytelevel_tokens(
    source: impl AsRef<Path>,
) -> Result<Vec<Vec<u8>>, uor_r4_model_source::SourceUnavailable> {
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(source)?)?;
    let vocab = value
        .pointer("/model/vocab")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| uor_r4_model_source::SourceUnavailable::new("missing BPE vocabulary"))?;
    let mut tokens = vec![Vec::new(); vocab.len()];
    let byte_map = bytelevel_inverse();
    for (piece, id) in vocab {
        let id = id
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| uor_r4_model_source::SourceUnavailable::new("invalid token id"))?;
        let output = tokens
            .get_mut(id)
            .ok_or_else(|| uor_r4_model_source::SourceUnavailable::new("sparse token ids"))?;
        for ch in piece.chars() {
            if let Some(byte) = byte_map.get(&ch) {
                output.push(*byte);
            } else {
                let mut utf8 = [0u8; 4];
                output.extend_from_slice(ch.encode_utf8(&mut utf8).as_bytes());
            }
        }
    }
    Ok(tokens)
}

#[cfg(not(target_arch = "wasm32"))]
fn bytelevel_inverse() -> BTreeMap<char, u8> {
    let mut bytes: Vec<u8> = (b'!'..=b'~')
        .chain(0xA1..=0xAC)
        .chain(0xAE..=0xFF)
        .collect();
    let mut codepoints: Vec<u32> = bytes.iter().map(|byte| u32::from(*byte)).collect();
    let mut extra = 0u32;
    for byte in 0u8..=u8::MAX {
        if !bytes.contains(&byte) {
            bytes.push(byte);
            codepoints.push(256 + extra);
            extra += 1;
        }
    }
    bytes
        .into_iter()
        .zip(codepoints)
        .map(|(byte, codepoint)| {
            (
                char::from_u32(codepoint).expect("byte-level codepoint is valid"),
                byte,
            )
        })
        .collect()
}

// ------------------------------------------------------------ tokenizer --

/// The original (July 2023, scoreless) llama2.c tokenizer.bin: per token,
/// i32 length then bytes. Probed conventions (witnessed in-run): the
/// sentencepiece space marker was already exported as a plain space
/// (piece 278 = " the"), and ids 3+cp hold the UTF-8 encoding of
/// codepoints U+0000..=U+00FF. Encoding: leading-space prefix, direct
/// per-char piece lookup with codepoint fallback, then iterative greedy
/// pair merging preferring the LOWEST merged token id (the deterministic
/// rule both systems share; the fluency gate validates its adequacy).
pub struct Tokenizer {
    pub vocab: Vec<Vec<u8>>,
    map: BTreeMap<Vec<u8>, u32>,
    mode: RuntimeTokenizerMode,
    /// Whether this vocab's own bytes use the GPT2 byte-level remap
    /// (space stored as the two UTF-8 bytes of `'Ġ'`, `0xC4 0xA0`, `\n`
    /// as `'Ċ'`, etc.) rather than literal raw bytes. Detected once at
    /// parse time (see [`detect_gpt2_byte_remap`]) and consulted by
    /// [`encode_into`](Self::encode_into) before applying that
    /// substitution — some real byte-BPE `tokenizer.bin` exports use the
    /// remap, others store literal bytes, and applying the remap to a
    /// vocab that doesn't use it means no merged word token can ever
    /// match, so every space is split into two spurious single-byte
    /// tokens (issue #751). Unused (always `false`) for `DecodeOnly`
    /// mode, which never encodes.
    gpt2_byte_remap: bool,
}

/// Whether `vocab`'s own byte content uses the GPT2 byte-level remap
/// convention: detected by checking for at least one entry starting with
/// the two-byte UTF-8 encoding of `'Ġ'` (`0xC4 0xA0`). A vocab that never
/// stores that byte pair (i.e. spaces are literal `0x20` bytes) must not
/// have the remap applied during encoding, or no merged token can ever be
/// found and the encoder falls back to spurious per-byte tokens at every
/// word boundary (#751). This is a real, load-bearing distinction between
/// tokenizer.bin exports, not a hypothetical: `smollm2-1-7b-instruct`'s
/// tokenizer.bin (49152 entries) is entirely literal-byte, with zero
/// vocab entries containing the `0xC4 0xA0` pair anywhere.
fn detect_gpt2_byte_remap(vocab: &[Vec<u8>]) -> bool {
    vocab
        .iter()
        .any(|piece| piece.len() >= 2 && piece[0] == 0xC4 && piece[1] == 0xA0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeTokenizerMode {
    Untagged,
    DecodeOnly {
        identity: RuntimeTokenizerIdentity,
        decode_policy: RuntimeTokenizerDecodePolicy,
    },
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let value = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
    *offset = end;
    Some(value)
}

fn read_tagged_identity(bytes: &[u8], offset: &mut usize) -> Option<String> {
    let length = usize::try_from(read_u32(bytes, offset)?).ok()?;
    if length != "blake3:".len() + 64 {
        return None;
    }
    let end = offset.checked_add(length)?;
    let value = std::str::from_utf8(bytes.get(*offset..end)?)
        .ok()?
        .to_owned();
    *offset = end;
    Some(value)
}

impl Tokenizer {
    /// Whether `bytes` declare the additive tagged tokenizer container.
    ///
    /// Callers that retain a historical fallback use this discriminator
    /// before parsing: a malformed tagged container must fail closed rather
    /// than being reinterpreted as an untagged tokenizer from another id
    /// space.
    pub fn is_tagged_container_bytes(bytes: &[u8]) -> bool {
        bytes.starts_with(TAGGED_TOKENIZER_MAGIC)
    }

    /// Load and validate a tokenizer without panicking on malformed input.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_load(
        path: impl AsRef<Path>,
    ) -> Result<Self, uor_r4_model_source::SourceUnavailable> {
        let path_ref = path.as_ref();

        // Tagged decode-only artifacts must win over the historical
        // directory-level vocab.json discovery. Otherwise a sibling file
        // could silently turn an explicitly decode-only artifact back into an
        // inferred encoder.
        match std::fs::read(path_ref) {
            Ok(bytes) if bytes.starts_with(TAGGED_TOKENIZER_MAGIC) => {
                return Self::from_bytes(&bytes).ok_or_else(|| {
                    uor_r4_model_source::SourceUnavailable::new(format!(
                        "{}: malformed tagged tokenizer.bin",
                        path_ref.display()
                    ))
                });
            }
            Ok(bytes) => {
                let reserved_tagged_namespace = bytes
                    .get(..4)
                    .and_then(|prefix| <[u8; 4]>::try_from(prefix).ok())
                    .is_some_and(|prefix| i32::from_le_bytes(prefix) < 0);
                if reserved_tagged_namespace {
                    return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                        "{}: malformed tagged tokenizer.bin header",
                        path_ref.display()
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(uor_r4_model_source::SourceUnavailable::new(format!(
                    "{}: tokenizer.bin cannot be read: {error}",
                    path_ref.display()
                )));
            }
        }

        // 1. Try loading vocab.json if present in the target or parent directories
        let json_candidates = [
            path_ref.to_path_buf(),
            path_ref.with_file_name("vocab.json"),
            path_ref
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("vocab.json"),
            std::path::PathBuf::from(".uor-models/sources/smollm2-1-7b-instruct/vocab.json"),
            std::path::PathBuf::from(".uor-models/compiled/smollm2-135m-instruct/vocab.json"),
        ];

        for jpath in &json_candidates {
            if jpath.extension().and_then(|s| s.to_str()) == Some("json") && jpath.exists() {
                if let Ok(bytes) = std::fs::read(jpath) {
                    if let Ok(raw_map) = serde_json::from_slice::<BTreeMap<String, u32>>(&bytes) {
                        let mut max_id = 0u32;
                        for &id in raw_map.values() {
                            if id > max_id {
                                max_id = id;
                            }
                        }
                        let mut vocab = vec![Vec::new(); (max_id + 1) as usize];
                        let mut map = BTreeMap::new();
                        for (k, &id) in &raw_map {
                            let k_bytes = k.as_bytes().to_vec();
                            vocab[id as usize] = k_bytes.clone();
                            map.insert(k_bytes, id);
                        }
                        let gpt2_byte_remap = detect_gpt2_byte_remap(&vocab);
                        return Ok(Tokenizer {
                            vocab,
                            map,
                            mode: RuntimeTokenizerMode::Untagged,
                            gpt2_byte_remap,
                        });
                    }
                }
            }
        }

        // 2. Fall back to binary tokenizer.bin format
        let bytes = std::fs::read(path_ref)?;
        Self::from_bytes(&bytes).ok_or_else(|| {
            uor_r4_model_source::SourceUnavailable::new(format!(
                "{}: not a valid tokenizer.bin",
                path_ref.display()
            ))
        })
    }

    /// Parse a tokenizer from in-memory bytes in the binary tokenizer.bin
    /// format (per token: i32 little-endian length, then the token bytes).
    /// Split from [`try_load`](Self::try_load) so library consumers can
    /// validate a bundled tokenizer without filesystem access.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.starts_with(TAGGED_TOKENIZER_MAGIC) {
            return Self::from_tagged_bytes(bytes);
        }
        let mut vocab = Vec::new();
        let mut offset = 0usize;
        while offset < bytes.len() {
            let length_bytes = bytes.get(offset..offset + 4)?;
            let length = i32::from_le_bytes(length_bytes.try_into().ok()?);
            offset += 4;
            if length < 0 {
                return None;
            }
            let length = length as usize;
            let token = bytes.get(offset..offset + length)?;
            vocab.push(token.to_vec());
            offset += length;
        }
        let mut map = BTreeMap::new();
        for (index, token) in vocab.iter().enumerate() {
            let id = index as u32;
            map.entry(token.clone()).or_insert(id);
        }
        let gpt2_byte_remap = detect_gpt2_byte_remap(&vocab);
        Some(Tokenizer {
            vocab,
            map,
            mode: RuntimeTokenizerMode::Untagged,
            gpt2_byte_remap,
        })
    }

    fn from_tagged_bytes(bytes: &[u8]) -> Option<Self> {
        let mut offset = TAGGED_TOKENIZER_MAGIC.len();
        let format_version = read_u32(bytes, &mut offset)?;
        if format_version != TAGGED_TOKENIZER_FORMAT_VERSION {
            return None;
        }
        let encode_policy = read_u32(bytes, &mut offset)?;
        if encode_policy != TAGGED_ENCODE_UNAVAILABLE {
            return None;
        }
        let decode_policy = match read_u32(bytes, &mut offset)? {
            TAGGED_DECODE_SENTENCEPIECE => {
                let strip_dummy_prefix = match read_u32(bytes, &mut offset)? {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                RuntimeTokenizerDecodePolicy::SentencePiece { strip_dummy_prefix }
            }
            _ => return None,
        };
        let version = read_u32(bytes, &mut offset)?;
        let family_len = usize::try_from(read_u32(bytes, &mut offset)?).ok()?;
        if family_len == 0 || family_len > MAX_TAGGED_FAMILY_BYTES {
            return None;
        }
        let family_bytes = bytes.get(offset..offset.checked_add(family_len)?)?;
        let family = std::str::from_utf8(family_bytes).ok()?.to_owned();
        if family.is_empty() {
            return None;
        }
        offset += family_len;
        let tokenizer_cid = read_tagged_identity(bytes, &mut offset)?;
        let adapter_digest = read_tagged_identity(bytes, &mut offset)?;
        if !is_blake3_address(&tokenizer_cid) || !is_blake3_address(&adapter_digest) {
            return None;
        }
        let piece_count = usize::try_from(read_u32(bytes, &mut offset)?).ok()?;
        if piece_count > MAX_TAGGED_VOCAB_SIZE
            || piece_count > bytes.len().saturating_sub(offset) / 4
        {
            return None;
        }
        let mut vocab = Vec::with_capacity(piece_count);
        for _ in 0..piece_count {
            let length = usize::try_from(read_u32(bytes, &mut offset)?).ok()?;
            if length > MAX_TOKEN_BYTES {
                return None;
            }
            let piece = bytes.get(offset..offset.checked_add(length)?)?;
            if matches!(
                decode_policy,
                RuntimeTokenizerDecodePolicy::SentencePiece { .. }
            ) && std::str::from_utf8(piece).is_err()
            {
                return None;
            }
            vocab.push(piece.to_vec());
            offset += length;
        }
        if offset != bytes.len() {
            return None;
        }
        let mut map = BTreeMap::new();
        for (index, token) in vocab.iter().enumerate() {
            let id = u32::try_from(index).ok()?;
            map.entry(token.clone()).or_insert(id);
        }
        Some(Self {
            vocab,
            map,
            mode: RuntimeTokenizerMode::DecodeOnly {
                identity: RuntimeTokenizerIdentity {
                    family,
                    version,
                    tokenizer_cid,
                    adapter_digest,
                },
                decode_policy,
            },
            // DecodeOnly tokenizers never encode (see `encode_into`'s
            // early `None` return for this mode), so this is unused;
            // `false` keeps the field's meaning honest rather than
            // implying an encode convention that's never consulted.
            gpt2_byte_remap: false,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: &str) -> Self {
        Self::try_load(path).expect("tokenizer file must be readable and well-formed")
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // Worst case is one token per input byte (byte fallback for
        // multi-byte UTF-8 chars) plus BOS and the synthetic leading space,
        // so size by byte length, not char count.
        let mut toks = vec![0u32; text.len().saturating_add(2)];
        let count = self
            .encode_into(text, &mut toks)
            .expect("token buffer sized from input bytes");
        toks.truncate(count);
        toks
    }

    /// Whether `ch` encodes — as a whole token or via byte fallback.
    fn char_encodable(&self, ch: char) -> bool {
        let mut utf8 = [0u8; 4];
        let bytes = ch.encode_utf8(&mut utf8).as_bytes();
        self.map.contains_key(bytes) || bytes.iter().all(|byte| self.map.contains_key(&[*byte][..]))
    }

    /// Lossy variant of [`encode`] for natural corpora (issue #72): a
    /// character the tokenizer cannot represent at all (neither a whole
    /// token nor byte fallback — the legacy llama2.c vocab covers ASCII
    /// bytes only) is replaced with a space, or dropped when even space is
    /// unencodable. Returns the token stream and the number of replaced
    /// characters. The substitution is deterministic, so a resumed
    /// observation pass reproduces identical records.
    pub fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        let substitute = if self.char_encodable(' ') {
            Some(' ')
        } else {
            None
        };
        let mut sanitized = String::with_capacity(text.len());
        let mut replaced = 0u64;
        for ch in text.chars() {
            if self.char_encodable(ch) {
                sanitized.push(ch);
            } else {
                if let Some(substitute) = substitute {
                    sanitized.push(substitute);
                }
                replaced += 1;
            }
        }
        (self.encode(&sanitized), replaced)
    }

    /// Encode an instructional prompt wrapped in native ChatML template format.
    pub fn encode_chat_prompt(&self, system_prompt: Option<&str>, user_prompt: &str) -> Vec<u32> {
        let formatted = format_instruct_chat_prompt(system_prompt, user_prompt);
        self.encode(&formatted)
    }

    /// Encode into caller-owned storage.
    ///
    /// Note: the BPE path allocates an intermediate `String` for byte-level
    /// remapping, so this is not fully allocation-free.
    /// Total: `Some(count)` of tokens written, `None` when `out` cannot hold
    /// the encoding (a property of the caller's buffer) or the text needs a
    /// byte fallback the tokenizer lacks.
    pub fn encode_into(&self, text: &str, out: &mut [u32]) -> Option<usize> {
        if matches!(&self.mode, RuntimeTokenizerMode::DecodeOnly { .. }) {
            return None;
        }
        let is_llama_bos = self.vocab.get(1).is_some_and(|v| v == b"<s>");
        let mut len = 0usize;
        if is_llama_bos {
            *out.get_mut(len)? = 1;
            len += 1;
        }

        let is_bpe = self.vocab.len() > 32000
            || self
                .vocab
                .get(1)
                .is_some_and(|v| v == b"<|im_start|>" || v == b"\xC4\xA0");

        if is_bpe {
            let mut encoded_str = String::with_capacity(text.len() * 2);
            for ch in text.chars() {
                match ch {
                    // Only remap to the GPT2 byte-level stand-ins when
                    // this vocab's own bytes actually use that
                    // convention (#751) — otherwise these substitutions
                    // can never match a real merged token, and every
                    // occurrence gets split into spurious per-byte
                    // fallback tokens instead.
                    ' ' if self.gpt2_byte_remap => encoded_str.push('Ġ'),
                    '\n' if self.gpt2_byte_remap => encoded_str.push('Ċ'),
                    '\r' if self.gpt2_byte_remap => encoded_str.push('Ĉ'),
                    '\t' if self.gpt2_byte_remap => encoded_str.push('ĉ'),
                    c => encoded_str.push(c),
                }
            }

            let bytes = encoded_str.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                let mut matched_len = 0usize;
                let mut matched_id = None;

                let max_k = (bytes.len() - i).min(64);
                for k in (1..=max_k).rev() {
                    let sub = &bytes[i..i + k];
                    if let Some(&id) = self.map.get(sub) {
                        matched_len = k;
                        matched_id = Some(id);
                        break;
                    }
                }

                if let Some(id) = matched_id {
                    *out.get_mut(len)? = id;
                    len += 1;
                    i += matched_len;
                } else {
                    let b = bytes[i];
                    if let Some(&id) = self.map.get(&[b][..]) {
                        *out.get_mut(len)? = id;
                        len += 1;
                    }
                    i += 1;
                }
            }
            return Some(len);
        }

        for ch in std::iter::once(' ').chain(text.chars()) {
            let mut utf8 = [0u8; 4];
            let bytes = ch.encode_utf8(&mut utf8).as_bytes();
            let token = match self.map.get(bytes) {
                Some(&id) => id,
                None => {
                    for byte in bytes {
                        let id = self.map.get(&[*byte][..]).copied()?;
                        *out.get_mut(len)? = id;
                        len += 1;
                    }
                    continue;
                }
            };
            *out.get_mut(len)? = token;
            len += 1;
        }
        loop {
            let mut best: Option<(u32, usize)> = None;
            let start = if is_llama_bos { 1 } else { 0 };
            for i in start..len.saturating_sub(1) {
                let left = &self.vocab[out[i] as usize];
                let right = &self.vocab[out[i + 1] as usize];
                let pair_len = left.len().saturating_add(right.len());
                if pair_len > MAX_TOKEN_BYTES {
                    continue;
                }
                let mut pair = [0u8; MAX_TOKEN_BYTES];
                pair[..left.len()].copy_from_slice(left);
                pair[left.len()..pair_len].copy_from_slice(right);
                if let Some(&id) = self.map.get(&pair[..pair_len]) {
                    if best.is_none_or(|(b, _)| id < b) {
                        best = Some((id, i));
                    }
                }
            }
            match best {
                Some((id, i)) => {
                    out[i] = id;
                    out.copy_within(i + 2..len, i + 1);
                    len -= 1;
                }
                None => break,
            }
        }
        Some(len)
    }

    pub fn decode(&self, toks: &[u32]) -> String {
        if let RuntimeTokenizerMode::DecodeOnly {
            identity,
            decode_policy,
        } = &self.mode
        {
            let mut raw = Vec::new();
            for &token in toks {
                if let Some(piece) = self.vocab.get(token as usize) {
                    raw.extend_from_slice(piece);
                }
            }
            let text = String::from_utf8_lossy(&raw);
            return match *decode_policy {
                RuntimeTokenizerDecodePolicy::SentencePiece { strip_dummy_prefix } => {
                    if strip_dummy_prefix
                        && identity.family == SENTENCEPIECE_REFERENCE_UNK_FAMILY
                        && identity.version == SENTENCEPIECE_REFERENCE_UNK_VERSION
                    {
                        text.strip_prefix('\u{2581}')
                            .unwrap_or(&text)
                            .replace('\u{2581}', " ")
                    } else {
                        let spaced = text.replace('\u{2581}', " ");
                        if strip_dummy_prefix {
                            spaced.strip_prefix(' ').unwrap_or(&spaced).to_owned()
                        } else {
                            spaced
                        }
                    }
                }
                RuntimeTokenizerDecodePolicy::Concatenate => text.into_owned(),
            };
        }
        let is_llama_bos = self.vocab.get(1).is_some_and(|v| v == b"<s>");
        let mut raw = Vec::new();
        for &t in toks {
            if is_llama_bos && (t == 1 || t == 2) {
                continue;
            }
            if (t as usize) < self.vocab.len() {
                raw.extend_from_slice(&self.vocab[t as usize]);
            }
        }
        let text = String::from_utf8_lossy(&raw);
        text.replace('Ġ', " ")
            .replace('Ċ', "\n")
            .replace('Ĉ', "\r")
            .replace('ĉ', "\t")
    }

    /// Registered family/version carried by a tagged runtime tokenizer.
    /// Historical untagged artifacts intentionally return `None` because
    /// their bytes predate adapter identity records.
    pub fn adapter_key(&self) -> Option<(&str, u32)> {
        match &self.mode {
            RuntimeTokenizerMode::Untagged => None,
            RuntimeTokenizerMode::DecodeOnly { identity, .. } => {
                Some((&identity.family, identity.version))
            }
        }
    }

    /// Exact host-adapter identity carried by a tagged decode-only artifact.
    /// Serving can compare this to the loaded host encoder before accepting
    /// prompt text; the tokenizer.bin content itself remains independently
    /// bound by the graph artifact's tokenizer CID.
    pub fn adapter_identity(&self) -> Option<&RuntimeTokenizerIdentity> {
        match &self.mode {
            RuntimeTokenizerMode::Untagged => None,
            RuntimeTokenizerMode::DecodeOnly { identity, .. } => Some(identity),
        }
    }

    pub fn is_decode_only(&self) -> bool {
        matches!(&self.mode, RuntimeTokenizerMode::DecodeOnly { .. })
    }

    /// Decode into caller-owned byte storage.
    ///
    /// Note: this delegates to `decode`, which allocates an intermediate
    /// `Vec`/`String`, so this is not allocation-free.
    /// Total: `Some(count)` of bytes written, `None` when `out` cannot hold
    /// the decoded text.
    pub fn decode_into(&self, toks: &[u32], out: &mut [u8]) -> Option<usize> {
        if matches!(&self.mode, RuntimeTokenizerMode::DecodeOnly { .. })
            && toks.iter().any(|&token| token as usize >= self.vocab.len())
        {
            return None;
        }
        let decoded = self.decode(toks);
        let bytes = decoded.as_bytes();
        if bytes.len() > out.len() {
            return None;
        }
        out[..bytes.len()].copy_from_slice(bytes);
        Some(bytes.len())
    }
}

// ------------------------------------------------------------ scenarios --

struct Scenario {
    class: &'static str,
    name: &'static str,
    text: String,
    /// true: score every position of the text against the ACTUAL next
    /// token (real human-written text); false: treat the text as a prompt
    /// and measure agreement along the teacher's greedy continuation.
    real_text: bool,
}

fn scenario_set() -> Vec<Scenario> {
    let shakespeare = {
        let full = std::fs::read_to_string("/tmp/corpus.txt").unwrap_or_default();
        match full.get(1000..2100) {
            Some(s) => s.to_string(),
            None => "To be, or not to be, that is the question.".to_string(),
        }
    };
    let s = |class, name, text: &str, real_text| Scenario {
        class,
        name,
        text: text.to_string(),
        real_text,
    };
    vec![
        // -- in-domain prompts (teacher-trajectory agreement)
        s(
            "in-domain prompt",
            "dog-named",
            "Once upon a time, there was a little dog named Rex.",
            false,
        ),
        s(
            "in-domain prompt",
            "park-ball",
            "Lily and Ben went to the park to play with their new ball.",
            false,
        ),
        s(
            "in-domain prompt",
            "sad-bird",
            "The little bird was sad because it could not fly.",
            false,
        ),
        s(
            "in-domain prompt",
            "red-truck",
            "Tom saw a big red truck outside his house.",
            false,
        ),
        s(
            "in-domain prompt",
            "shiny-key",
            "One day, a cat found a shiny key in the garden.",
            false,
        ),
        // -- out-of-domain real-world prompts
        s(
            "out-of-domain prompt",
            "capital-q",
            "What is the capital of France?",
            false,
        ),
        s(
            "out-of-domain prompt",
            "explain",
            "Explain how photosynthesis works.",
            false,
        ),
        s(
            "out-of-domain prompt",
            "code",
            "Write a Python function to add two numbers.",
            false,
        ),
        s(
            "out-of-domain prompt",
            "business",
            "The quarterly revenue increased by fifteen percent compared to",
            false,
        ),
        // -- real human-written text, scored against actual next tokens
        s(
            "real text, in-domain style",
            "handwritten-story",
            "One day, a little girl named Mia went to the park with her mom. Mia saw a big dog. The dog was sad because it lost its ball. Mia wanted to help the dog. She looked under the bench and found the ball. The dog was very happy and wagged its tail. Mia and the dog played together all day. When it was time to go home, the dog gave Mia a big lick on her face. Mia laughed and said she would come back tomorrow.",
            true,
        ),
        s(
            "real text, out-of-domain",
            "shakespeare",
            &shakespeare,
            true,
        ),
        // -- structural stress
        s(
            "stress",
            "repetition",
            "one two three four one two three four one two three four one two three four",
            false,
        ),
        s("stress", "one-word", "The", false),
        s("stress", "cold-start", "", false),
    ]
}

/// Adapter: a token sequence as a single-story Corpus, so the scenario
/// path runs the IDENTICAL runtime functions certified in PROOF.md.
fn as_corpus(tokens: &[u32], t_argmax: &[u32]) -> Corpus {
    let n = tokens.len();
    Corpus {
        n,
        stories: 1,
        story: vec![0; n],
        input: tokens.to_vec(),
        next: {
            let mut nx = tokens[1..].to_vec();
            nx.push(0);
            nx
        },
        t_argmax: t_argmax.to_vec(),
        top_tokens: vec![[0u32; 8]; n],
        top_weights: vec![[0u32; 8]; n],
        span_start: (0..n).map(|idx| idx as u32).collect(),
        span_end: (0..n).map(|idx| idx as u32 + 1).collect(),
        byte_start: vec![u32::MAX; n],
        byte_end: vec![u32::MAX; n],
        hidden: None,
    }
}

struct ClassAgg {
    positions: u64,
    agree: u64,
    tless_top1: u64,
    teacher_top1: u64,
    real_positions: u64,
    tless_ns: u128,
    teacher_ns: u128,
    teacher_steps: u64,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scenarios(oracle: &mut dyn TeacherOracle) {
    let tok = Tokenizer::load("/tmp/ref/tokenizer.bin");

    // Tokenizer witnesses: round-trip, then the fluency gate.
    let probe = "Once upon a time, there was a little dog.";
    let ids = tok.encode(probe);
    assert_eq!(tok.decode(&ids).trim_start(), probe, "tokenizer round-trip");
    println!(
        "tokenizer witness: round-trip exact on probe ({} tokens)",
        ids.len()
    );

    let cap = oracle.seq_len() - 2;
    let vocab = oracle.vocab();
    let mut logits = vec![0f32; vocab];
    {
        oracle.reset();
        let mut seq = ids.clone();
        for (pos, &t) in ids.iter().enumerate() {
            oracle.step(t as usize, pos, &mut logits);
        }
        for pos in ids.len()..ids.len() + 20 {
            let mut best = 0usize;
            for i in 1..vocab {
                if logits[i] > logits[best] {
                    best = i;
                }
            }
            seq.push(best as u32);
            oracle.step(best, pos, &mut logits);
        }
        let cont = tok.decode(&seq[ids.len()..]);
        println!(
            "tokenizer fluency gate — teacher continues the probe with: \"{}\"",
            cont.trim()
        );
        assert!(
            cont.split_whitespace().count() >= 3,
            "teacher continuation not fluent; encoding suspect"
        );
    }

    // Artifact + store (train split only; every scenario is unseen).
    let art = compiler::load_artifacts().expect("run `cargo run --release -- compile` first");
    let c150 = compiler::load_corpus().expect("run `transformerless gen` first");
    let (store, _) = build_store(&art, &c150);
    let rot = derive_rotations();
    let store_ref: &Store = &store;

    let mut agg: BTreeMap<&'static str, ClassAgg> = BTreeMap::new();
    println!();
    println!("| scenario | class | tokens | agree w/ teacher | tless top1 | teacher top1 |");
    println!("|---|---|---|---|---|---|");

    for sc in scenario_set() {
        // 1. token stream: prompt (+ teacher greedy greedy continuation if prompt scenario)
        let prompt: Vec<u32> = if sc.text.is_empty() {
            vec![1]
        } else {
            let mut p = tok.encode(&sc.text);
            p.truncate(cap.min(p.len()));
            p
        };
        oracle.reset();
        let mut seq = prompt.clone();
        let mut t_argmax: Vec<u32> = Vec::new();
        let t0 = std::time::Instant::now();
        for (pos, &t) in prompt.iter().enumerate() {
            oracle.step(t as usize, pos, &mut logits);
            let mut best = 0usize;
            for i in 1..vocab {
                if logits[i] > logits[best] {
                    best = i;
                }
            }
            t_argmax.push(best as u32);
        }
        if !sc.real_text {
            let cont = 64usize.min(cap.saturating_sub(prompt.len()));
            for _ in 0..cont {
                let last = *t_argmax.last().unwrap() as usize;
                seq.push(last as u32);
                let pos = seq.len() - 1;
                oracle.step(last, pos, &mut logits);
                let mut best = 0usize;
                for i in 1..vocab {
                    if logits[i] > logits[best] {
                        best = i;
                    }
                }
                t_argmax.push(best as u32);
            }
        }
        let teacher_ns = t0.elapsed().as_nanos();

        // 2. artifact predictions over the identical stream
        let cs = as_corpus(&seq, &t_argmax);
        let n_eval = cs.n - 1; // positions with a defined next token
        let t0 = std::time::Instant::now();
        let preds: Vec<u32> = (0..n_eval)
            .map(|i| predict_plain(store_ref, &code_plain(&art, &rot, &cs, i)))
            .collect();
        let tless_ns = t0.elapsed().as_nanos();

        // 3. metrics
        let (mut agree, mut tl1, mut th1) = (0u64, 0u64, 0u64);
        for (i, &prediction) in preds.iter().enumerate() {
            if prediction == cs.t_argmax[i] {
                agree += 1;
            }
            if sc.real_text {
                if prediction == cs.next[i] {
                    tl1 += 1;
                }
                if cs.t_argmax[i] == cs.next[i] {
                    th1 += 1;
                }
            }
        }
        let pct = |x: u64| 100.0 * x as f64 / n_eval as f64;
        println!(
            "| {} | {} | {} | {:.1}% | {} | {} |",
            sc.name,
            sc.class,
            n_eval,
            pct(agree),
            if sc.real_text {
                format!("{:.1}%", pct(tl1))
            } else {
                "—".into()
            },
            if sc.real_text {
                format!("{:.1}%", pct(th1))
            } else {
                "—".into()
            },
        );

        let e = agg.entry(sc.class).or_insert(ClassAgg {
            positions: 0,
            agree: 0,
            tless_top1: 0,
            teacher_top1: 0,
            real_positions: 0,
            tless_ns: 0,
            teacher_ns: 0,
            teacher_steps: 0,
        });
        e.positions += n_eval as u64;
        e.agree += agree;
        if sc.real_text {
            e.real_positions += n_eval as u64;
            e.tless_top1 += tl1;
            e.teacher_top1 += th1;
        }
        e.tless_ns += tless_ns;
        e.teacher_ns += teacher_ns;
        e.teacher_steps += t_argmax.len() as u64;
    }

    println!();
    println!(
        "| class | positions | agree w/ teacher | tless top1 | teacher top1 | tless tok/s | teacher tok/s |"
    );
    println!("|---|---|---|---|---|---|---|");
    for (class, e) in &agg {
        let ag = 100.0 * e.agree as f64 / e.positions as f64;
        let (tl, th) = if e.real_positions > 0 {
            (
                format!(
                    "{:.1}%",
                    100.0 * e.tless_top1 as f64 / e.real_positions as f64
                ),
                format!(
                    "{:.1}%",
                    100.0 * e.teacher_top1 as f64 / e.real_positions as f64
                ),
            )
        } else {
            ("—".into(), "—".into())
        };
        println!(
            "| {} | {} | {:.1}% | {} | {} | {:.0} | {:.0} |",
            class,
            e.positions,
            ag,
            tl,
            th,
            e.positions as f64 / (e.tless_ns as f64 / 1e9),
            e.teacher_steps as f64 / (e.teacher_ns as f64 / 1e9),
        );
    }
    println!();
    println!(
        "notes: prompt scenarios measure agreement along the teacher's own greedy\ntrajectory; real-text rows also score both systems against the actual next\ntoken. The store was built from the training split only — every scenario\nstream is unseen. Classical runtimes execute the source model, so their\nscenario predictions coincide with the teacher columns by definition."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagged_fixture_bytes() -> Vec<u8> {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-tagged-tokenizer-{}-{:?}.bin",
            std::process::id(),
            std::thread::current().id()
        ));
        let digest = format!("blake3:{}", "1".repeat(64));
        let table = RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: "sentencepiece-unigram".to_owned(),
                version: 1,
                tokenizer_cid: digest.clone(),
                adapter_digest: digest,
            },
            pieces: vec![
                b"<unk>".to_vec(),
                "\u{2581}a".as_bytes().to_vec(),
                Vec::new(),
            ],
            encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        export_runtime_tokenizer_table(&table, &path).expect("export tagged fixture");
        let bytes = std::fs::read(&path).expect("read tagged fixture");
        std::fs::remove_file(path).ok();
        bytes
    }

    #[test]
    fn test_format_instruct_chat_prompt_default_system() {
        let formatted = format_instruct_chat_prompt(None, "Why is the sky blue?");
        assert!(formatted.contains("<|im_start|>system\nYou are a helpful AI assistant.<|im_end|>"));
        assert!(formatted.contains("<|im_start|>user\nWhy is the sky blue?<|im_end|>"));
        assert!(formatted.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_format_instruct_chat_prompt_custom_system() {
        let formatted = format_instruct_chat_prompt(Some("System directive"), "Hello!");
        assert!(formatted.contains("<|im_start|>system\nSystem directive<|im_end|>"));
        assert!(formatted.contains("<|im_start|>user\nHello!<|im_end|>"));
        assert!(formatted.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn historical_untagged_tokenizer_bytes_stay_exact() {
        let mut bytes = Vec::new();
        for piece in [&b" "[..], b"a", b"ab"] {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        let tokenizer = Tokenizer::from_bytes(&bytes).expect("historical bytes parse");
        assert_eq!(
            tokenizer.vocab,
            vec![b" ".to_vec(), b"a".to_vec(), b"ab".to_vec()]
        );
        assert_eq!(tokenizer.adapter_identity(), None);
        assert!(!tokenizer.is_decode_only());
        assert_eq!(tokenizer.decode(&[2]), "ab");
    }

    fn untagged_bytes_from_pieces(pieces: &[&[u8]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for piece in pieces {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        bytes
    }

    /// #751: a byte-BPE vocab (triggered here via `vocab[1] ==
    /// "<|im_start|>"`, the same heuristic real bundles hit) whose own
    /// bytes are literal (a real 0x20 space, no GPT2 remap) must encode
    /// a space-prefixed word as its single merged token, not split the
    /// (wrongly-inserted) remap character into spurious per-byte
    /// fallback tokens. This is the exact failure mode found on
    /// `smollm2-1-7b-instruct`'s real tokenizer.bin.
    #[test]
    fn encode_into_uses_literal_bytes_when_vocab_has_no_gpt2_remap() {
        let bytes = untagged_bytes_from_pieces(&[
            b"<|endoftext|>",
            b"<|im_start|>",
            b" how", // merged, literal bytes: space + "how"
            b" ",
            b"h",
            b"o",
            b"w",
        ]);
        let tokenizer = Tokenizer::from_bytes(&bytes).expect("literal-byte vocab parses");
        assert_eq!(
            tokenizer.encode(" how"),
            vec![2],
            "must match the single merged token, not split into byte-remap fallback pieces"
        );
    }

    /// Companion to the test above: a vocab that genuinely uses the GPT2
    /// byte-level remap (its own bytes contain the `'Ġ'` two-byte
    /// sequence) must still have the remap applied during encoding —
    /// #751's fix is convention-detection, not convention-removal.
    #[test]
    fn encode_into_still_applies_gpt2_remap_when_vocab_uses_it() {
        let mut g_how = vec![0xC4u8, 0xA0]; // 'Ġ' UTF-8
        g_how.extend_from_slice(b"how");
        let bytes = untagged_bytes_from_pieces(&[b"<|endoftext|>", b"<|im_start|>", &g_how]);
        let tokenizer = Tokenizer::from_bytes(&bytes).expect("gpt2-remap vocab parses");
        assert_eq!(
            tokenizer.encode(" how"),
            vec![2],
            "a vocab that stores the GPT2 remap convention must still have it applied"
        );
    }

    #[test]
    fn tagged_parser_bounds_identity_counts_and_trailing_bytes() {
        let bytes = tagged_fixture_bytes();
        assert!(Tokenizer::is_tagged_container_bytes(&bytes));
        let tokenizer = Tokenizer::from_bytes(&bytes).expect("valid tagged fixture parses");
        assert_eq!(tokenizer.decode(&[1]), "a");

        // Locate the identity fields without relying on their concrete string
        // lengths, then corrupt each guarded boundary independently.
        let mut offset = TAGGED_TOKENIZER_MAGIC.len() + 5 * 4;
        let family_len = read_u32(&bytes, &mut offset).unwrap() as usize;
        offset += family_len;
        let cid_len_offset = offset;
        let cid_len = read_u32(&bytes, &mut offset).unwrap() as usize;
        let cid_start = offset;
        offset += cid_len;
        let digest_len = read_u32(&bytes, &mut offset).unwrap() as usize;
        offset += digest_len;
        let piece_count_offset = offset;

        let mut bad_identity = bytes.clone();
        bad_identity[cid_start] = b'X';
        assert!(Tokenizer::from_bytes(&bad_identity).is_none());

        let mut huge_count = bytes.clone();
        huge_count[piece_count_offset..piece_count_offset + 4]
            .copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Tokenizer::from_bytes(&huge_count).is_none());

        let mut huge_identity = bytes.clone();
        huge_identity[cid_len_offset..cid_len_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Tokenizer::from_bytes(&huge_identity).is_none());

        let mut trailing = bytes;
        trailing.push(0);
        assert!(Tokenizer::is_tagged_container_bytes(&trailing));
        assert!(Tokenizer::from_bytes(&trailing).is_none());
    }

    #[test]
    fn sentencepiece_tagged_export_and_parser_reject_invalid_utf8() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-invalid-utf8-tokenizer-{}-{:?}.bin",
            std::process::id(),
            std::thread::current().id()
        ));
        let digest = format!("blake3:{}", "2".repeat(64));
        let table = RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: "sentencepiece-unigram".to_owned(),
                version: 1,
                tokenizer_cid: digest.clone(),
                adapter_digest: digest,
            },
            pieces: vec![vec![0xff]],
            encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        let error = export_runtime_tokenizer_table(&table, &path)
            .expect_err("invalid UTF-8 must not enter a SentencePiece runtime table");
        assert!(error.reason.contains("valid UTF-8"), "{error}");
        assert!(!path.exists());

        let mut valid_table = table;
        valid_table.pieces = vec![b"a".to_vec()];
        export_runtime_tokenizer_table(&valid_table, &path).expect("valid control export");
        let mut bytes = std::fs::read(&path).expect("read valid control export");
        let final_byte = bytes.last_mut().expect("fixture has a final piece byte");
        *final_byte = 0xff;
        assert!(Tokenizer::from_bytes(&bytes).is_none());
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn damaged_tagged_header_cannot_downgrade_to_sibling_vocab() {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-damaged-tagged-tokenizer-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create damaged-tag fixture");
        let mut bytes = tagged_fixture_bytes();
        bytes[4] ^= 1;
        assert!(i32::from_le_bytes(bytes[..4].try_into().unwrap()) < 0);
        let tokenizer_path = dir.join("tokenizer.bin");
        std::fs::write(&tokenizer_path, bytes).expect("write damaged tagged bytes");
        std::fs::write(dir.join("vocab.json"), br#"{"a":0}"#)
            .expect("write otherwise-loadable sibling vocab");

        let error = Tokenizer::try_load(&tokenizer_path)
            .err()
            .expect("damaged tag must not reach sibling/global fallback");
        assert!(error.reason.contains("malformed tagged"), "{error}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
