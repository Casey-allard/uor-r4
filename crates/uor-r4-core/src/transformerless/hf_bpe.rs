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

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::scenarios::{
    RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable, RuntimeTokenizerEncodePolicy,
    RuntimeTokenizerIdentity, Tokenizer,
};

/// Upper bound on cached pre-token encodings (bounds memory on adversarial
/// corpora; natural text saturates far below this).
const CACHE_CAPACITY: usize = 1 << 16;
/// Hard ingestion bound for an id-indexed tokenizer table. Supported source
/// vocabularies are far below this ceiling; it prevents one hostile added-token
/// id from requesting a multi-billion-entry `Vec<String>`.
const MAX_TOKENIZER_VOCAB_SIZE: usize = 1 << 20;

/// The standard GPT-2 byte-to-unicode table: printable latin-1 bytes map to
/// themselves; the remaining 68 bytes map to U+0100.. in ascending order.
/// This is the exact inverse of `scenarios::bytelevel_inverse`.
fn bytes_to_unicode() -> [char; 256] {
    let mut table = ['\0'; 256];
    let mut assigned = [false; 256];
    for byte in (b'!'..=b'~').chain(0xA1..=0xAC).chain(0xAE..=0xFF) {
        // Codepoints 0x21..=0xFF are all valid chars.
        table[byte as usize] = char::from_u32(u32::from(byte)).expect("latin-1 codepoint is valid");
        assigned[byte as usize] = true;
    }
    let mut extra = 0u32;
    for byte in 0usize..256 {
        if !assigned[byte] {
            // Codepoints 0x100..=0x143 are all valid chars.
            table[byte] =
                char::from_u32(256 + extra).expect("byte-level fallback codepoint is valid");
            extra += 1;
        }
    }
    table
}

/// One segment of the input after atomic added-token matching.
enum Piece<'text> {
    Text(&'text str),
    Special(u32),
}

/// A byte-level BPE tokenizer parsed from a Hugging Face `tokenizer.json`.
pub struct HfBpeTokenizer {
    /// id → token content: byte-level alphabet for vocabulary tokens,
    /// literal text for added tokens, empty for unassigned ids.
    vocab: Vec<String>,
    /// Number of ids declared by `model.vocab`, excluding added tokens. The
    /// historical runtime export contains exactly this prefix.
    model_vocab_len: usize,
    token_to_id: HashMap<String, u32>,
    /// `"left right"` → rank (list position in `model.merges`). Byte-level
    /// tokens never contain a space, so the joined key is unambiguous.
    merge_ranks: HashMap<String, u32>,
    /// Added tokens sorted by content length descending, matched atomically
    /// before pre-tokenization.
    added_tokens: Vec<(String, u32)>,
    /// ids whose content decodes literally (no byte-level reverse mapping).
    added_ids: HashSet<u32>,
    byte_encoder: [char; 256],
    byte_decoder: HashMap<char, u8>,
    add_prefix_space: bool,
    /// `Some(individual_digits)` when a `Digits` pre-tokenizer step runs
    /// before `ByteLevel`.
    digits_individual: Option<bool>,
    /// blake3 of the raw `tokenizer.json` bytes, `blake3:<hex>`.
    address: String,
    /// Per-pre-token merge cache: raw pre-token text → token ids.
    cache: Mutex<HashMap<String, Vec<u32>>>,
}

impl HfBpeTokenizer {
    /// Parse a tokenizer from raw `tokenizer.json` bytes. Total: `Some` for a
    /// valid byte-level BPE tokenizer, `None` when the JSON is malformed or not
    /// the byte-level BPE shape this module requires.
    pub fn from_tokenizer_json_bytes(bytes: &[u8]) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        let model = value.get("model")?;
        if let Some(kind) = model.get("type").and_then(serde_json::Value::as_str) {
            if kind != "BPE" {
                return None;
            }
        }
        let vocab_map = model.get("vocab").and_then(serde_json::Value::as_object)?;
        let model_vocab_len = vocab_map.len();
        if model_vocab_len == 0 || model_vocab_len > MAX_TOKENIZER_VOCAB_SIZE {
            return None;
        }
        let mut token_to_id: HashMap<String, u32> = HashMap::with_capacity(model_vocab_len);
        let mut id_to_content: HashMap<u32, String> = HashMap::with_capacity(model_vocab_len);
        // The historical runtime export is an id-indexed dense prefix. Reject
        // sparse or duplicate model ids at ingestion so the family-neutral
        // runtime-table path cannot truncate or alias a host vocabulary.
        let mut occupied_model_ids = vec![false; model_vocab_len];
        let mut max_id = 0u32;
        for (piece, id) in vocab_map {
            let id = id.as_u64().and_then(|id| u32::try_from(id).ok())?;
            let index = usize::try_from(id).ok()?;
            let occupied = occupied_model_ids.get_mut(index)?;
            if std::mem::replace(occupied, true) {
                return None;
            }
            max_id = max_id.max(id);
            token_to_id.insert(piece.clone(), id);
            id_to_content.insert(id, piece.clone());
        }

        let mut added_tokens: Vec<(String, u32)> = Vec::new();
        let mut seen_added_tokens: HashSet<(String, u32)> = HashSet::new();
        if let Some(entries) = value
            .get("added_tokens")
            .and_then(serde_json::Value::as_array)
        {
            for entry in entries {
                let content = entry.get("content").and_then(serde_json::Value::as_str)?;
                if content.is_empty() {
                    // Atomic matching an empty surface would never advance
                    // the input cursor. Reject it at ingestion rather than
                    // admitting a tokenizer whose encode path cannot make
                    // progress.
                    return None;
                }
                let id = entry
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|id| u32::try_from(id).ok())?;
                // Hugging Face tokenizers commonly repeat a model-vocabulary
                // special in `added_tokens` with the exact same surface and
                // id. Preserve that declaration, but refuse either axis of a
                // conflicting alias: one id may not decode to two contents,
                // and one content may not encode to two ids.
                if id_to_content
                    .get(&id)
                    .is_some_and(|existing| existing != content)
                    || token_to_id
                        .get(content)
                        .is_some_and(|&existing| existing != id)
                {
                    return None;
                }
                max_id = max_id.max(id);
                id_to_content
                    .entry(id)
                    .or_insert_with(|| content.to_owned());
                token_to_id.entry(content.to_owned()).or_insert(id);
                if seen_added_tokens.insert((content.to_owned(), id)) {
                    added_tokens.push((content.to_owned(), id));
                }
            }
        }

        let vocab_len = usize::try_from(max_id).ok()?.checked_add(1)?;
        if vocab_len > MAX_TOKENIZER_VOCAB_SIZE {
            return None;
        }
        let mut vocab = vec![String::new(); vocab_len];
        for (piece, &id) in &token_to_id {
            vocab[id as usize] = piece.clone();
        }
        let mut added_ids = HashSet::with_capacity(added_tokens.len());
        for (content, id) in &added_tokens {
            vocab[*id as usize] = content.clone();
            token_to_id.entry(content.clone()).or_insert(*id);
            added_ids.insert(*id);
        }
        // Longest content first: leftmost-longest atomic matching.
        added_tokens.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));

        let merges = model.get("merges").and_then(serde_json::Value::as_array)?;
        let mut merge_ranks: HashMap<String, u32> = HashMap::with_capacity(merges.len());
        for (rank, entry) in merges.iter().enumerate() {
            let key = match entry {
                serde_json::Value::String(pair) => {
                    if pair.split(' ').count() != 2 {
                        return None;
                    }
                    pair.clone()
                }
                serde_json::Value::Array(pair) => match (
                    pair.first().and_then(serde_json::Value::as_str),
                    pair.get(1).and_then(serde_json::Value::as_str),
                    pair.len(),
                ) {
                    (Some(left), Some(right), 2) => format!("{left} {right}"),
                    _ => {
                        return None;
                    }
                },
                _ => {
                    return None;
                }
            };
            let rank = u32::try_from(rank).ok()?;
            merge_ranks.entry(key).or_insert(rank);
        }

        let pre_tokenizer = parse_pre_tokenizer(value.get("pre_tokenizer"))?;

        let byte_encoder = bytes_to_unicode();
        let mut byte_decoder = HashMap::with_capacity(256);
        for (byte, mapped) in byte_encoder.iter().enumerate() {
            byte_decoder.insert(*mapped, byte as u8);
        }

        Some(Self {
            vocab,
            model_vocab_len,
            token_to_id,
            merge_ranks,
            added_tokens,
            added_ids,
            byte_encoder,
            byte_decoder,
            add_prefix_space: pre_tokenizer.add_prefix_space,
            digits_individual: pre_tokenizer.digits_individual,
            address: format!("blake3:{}", blake3::hash(bytes).to_hex()),
            cache: Mutex::new(HashMap::new()),
        })
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

    /// Encode text to token ids: added tokens atomically, then the optional
    /// `Digits` split, then GPT-2 byte-level pre-tokenization and
    /// rank-ordered BPE per pre-token. No BOS/EOS tokens are added
    /// (matching the legacy HF encode path; SmolLM2's post-processor is
    /// null).
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut ids = Vec::new();
        for piece in self.split_on_added_tokens(text) {
            match piece {
                Piece::Special(id) => ids.push(id),
                Piece::Text(segment) => self.encode_segment(segment, &mut ids),
            }
        }
        ids
    }

    /// Encode one non-special segment. `ByteLevel` applies
    /// `add_prefix_space` to EVERY split it receives (each `Digits` split,
    /// or the whole segment without a `Digits` step), so the prefix is
    /// applied here, after digit splitting.
    fn encode_segment(&self, segment: &str, ids: &mut Vec<u32>) {
        let splits = match self.digits_individual {
            Some(individual) => split_digits(segment, individual),
            None => vec![segment],
        };
        for split in splits {
            if split.is_empty() {
                continue;
            }
            let prefixed;
            let split = if self.add_prefix_space && !split.starts_with(' ') {
                prefixed = format!(" {split}");
                prefixed.as_str()
            } else {
                split
            };
            for pre_token in pre_tokenize(split) {
                self.bpe(pre_token, ids);
            }
        }
    }

    /// Lossy encode, mirroring the legacy surface: byte-level BPE encodes
    /// every input exactly, so the replaced-character count is always zero.
    pub fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        (self.encode(text), 0)
    }

    /// Decode ids: token strings → reverse byte-level mapping → bytes →
    /// lossy UTF-8. Added tokens decode to their literal content;
    /// out-of-range ids are skipped.
    pub fn decode(&self, ids: &[u32]) -> String {
        String::from_utf8_lossy(&self.decode_bytes(ids)).into_owned()
    }

    /// Raw decoded bytes of `ids` (before any lossy UTF-8 conversion).
    /// Public since #601: differential fixtures and consumer-agreement
    /// tests compare token content at the byte level, where a token
    /// holding a partial UTF-8 sequence keeps its true bytes instead of
    /// the replacement character.
    pub fn decode_bytes(&self, ids: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for &id in ids {
            let Some(token) = self.vocab.get(id as usize) else {
                continue;
            };
            if self.added_ids.contains(&id) {
                bytes.extend_from_slice(token.as_bytes());
                continue;
            }
            for ch in token.chars() {
                match self.byte_decoder.get(&ch) {
                    Some(&byte) => bytes.push(byte),
                    None => {
                        let mut utf8 = [0u8; 4];
                        bytes.extend_from_slice(ch.encode_utf8(&mut utf8).as_bytes());
                    }
                }
            }
        }
        bytes
    }

    /// Number of id slots (max assigned id + 1, added tokens included).
    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    /// Byte length of each token's RAW decoded content, indexed by id
    /// (byte-anchor generation; unassigned ids have length 0). Raw bytes
    /// are measured before lossy UTF-8 conversion, so a token holding a
    /// partial UTF-8 sequence counts its true byte length rather than the
    /// replacement character's.
    pub fn token_byte_lengths(&self) -> Vec<u32> {
        (0..self.vocab.len() as u32)
            .map(|id| self.decode_bytes(&[id]).len() as u32)
            .collect()
    }

    /// Content address of the raw `tokenizer.json` bytes: `blake3:<hex>`.
    pub fn address(&self) -> String {
        self.address.clone()
    }

    /// The versioned adapter-identity record (#601) this tokenizer
    /// implements: family `hf-byte-bpe` version 1, the `tokenizer.json`
    /// content address, and the encode/decode policy the parsed
    /// configuration selects. Host/compile-side metadata only — the
    /// encode/decode hot paths never call this.
    pub fn adapter(&self) -> TokenizerAdapter {
        // Canonical added-token summary: entries sorted by id, each as
        // `<id>:<byte length>:<content bytes>\n` (length-prefixed, so no
        // content byte can be confused with a separator).
        let mut entries: Vec<(u32, &str)> = self
            .added_tokens
            .iter()
            .map(|(content, id)| (*id, content.as_str()))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        let mut listing = Vec::new();
        for (id, content) in &entries {
            listing.extend_from_slice(format!("{id}:{}:", content.len()).as_bytes());
            listing.extend_from_slice(content.as_bytes());
            listing.push(b'\n');
        }
        let mut pre_tokenizers = Vec::new();
        if let Some(individual) = self.digits_individual {
            pre_tokenizers.push(format!("digits(individual_digits={individual})"));
        }
        pre_tokenizers.push(format!(
            "byte-level(add_prefix_space={})",
            self.add_prefix_space
        ));
        let policy = TokenizerAdapterPolicy {
            // This adapter applies no normalizer; parsing accepts the
            // configuration as-is and never rewrites input text.
            normalizer: "none".to_owned(),
            pre_tokenizers,
            // The GPT-2 byte alphabet covers all 256 byte values, so
            // every input byte is encodable: encoding is total.
            byte_fallback: "byte-level-alphabet".to_owned(),
            added_tokens_count: entries.len() as u32,
            added_tokens_digest: format!("blake3:{}", blake3::hash(&listing).to_hex()),
            // No BOS/EOS insertion: encode adds no tokens beyond the
            // input (the pinned SmolLM2 post-processor is null; see
            // `HfBpeTokenizer::encode`).
            bos: "none".to_owned(),
            eos: "none".to_owned(),
            chat_template_policy: "not-interpreted".to_owned(),
        };
        let mut record = TokenizerAdapter {
            family: TokenizerAdapter::HF_BYTE_BPE_FAMILY.to_owned(),
            version: TokenizerAdapter::HF_BYTE_BPE_VERSION,
            tokenizer_cid: self.address.clone(),
            policy,
            adapter_digest: String::new(),
        };
        record.adapter_digest = record.declared_digest();
        record
    }

    /// Split text on added-token occurrences, leftmost-longest.
    fn split_on_added_tokens<'text>(&self, text: &'text str) -> Vec<Piece<'text>> {
        let mut pieces = Vec::new();
        let mut rest = text;
        while !rest.is_empty() {
            let mut best: Option<(usize, usize, u32)> = None;
            for (content, id) in &self.added_tokens {
                if content.is_empty() {
                    continue;
                }
                if let Some(position) = rest.find(content.as_str()) {
                    // added_tokens is longest-first, so on position ties the
                    // earlier (longer) match is kept by the strict `<`.
                    if best.is_none_or(|(existing, _, _)| position < existing) {
                        best = Some((position, content.len(), *id));
                    }
                    if position == 0 {
                        break;
                    }
                }
            }
            match best {
                Some((position, length, id)) => {
                    if position > 0 {
                        pieces.push(Piece::Text(&rest[..position]));
                    }
                    pieces.push(Piece::Special(id));
                    rest = &rest[position + length..];
                }
                None => {
                    pieces.push(Piece::Text(rest));
                    break;
                }
            }
        }
        pieces
    }

    /// Rank-ordered BPE over one pre-token, appending token ids to `out`.
    fn bpe(&self, pre_token: &str, out: &mut Vec<u32>) {
        if pre_token.is_empty() {
            return;
        }
        {
            let cache = match self.cache.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(ids) = cache.get(pre_token) {
                out.extend_from_slice(ids);
                return;
            }
        }

        let mut symbols: Vec<String> = pre_token
            .bytes()
            .map(|byte| self.byte_encoder[byte as usize].to_string())
            .collect();
        let mut pair_key = String::new();
        while symbols.len() > 1 {
            // The adjacent pair with the LOWEST merge rank merges first.
            let mut best: Option<(u32, usize)> = None;
            for index in 0..symbols.len() - 1 {
                pair_key.clear();
                pair_key.push_str(&symbols[index]);
                pair_key.push(' ');
                pair_key.push_str(&symbols[index + 1]);
                if let Some(&rank) = self.merge_ranks.get(&pair_key) {
                    if best.is_none_or(|(existing, _)| rank < existing) {
                        best = Some((rank, index));
                    }
                }
            }
            let Some((_, best_index)) = best else {
                break;
            };
            let left = symbols[best_index].clone();
            let right = symbols[best_index + 1].clone();
            let merged = format!("{left}{right}");
            let mut next_symbols = Vec::with_capacity(symbols.len());
            let mut index = 0usize;
            while index < symbols.len() {
                if index + 1 < symbols.len()
                    && symbols[index] == left
                    && symbols[index + 1] == right
                {
                    next_symbols.push(merged.clone());
                    index += 2;
                } else {
                    next_symbols.push(std::mem::take(&mut symbols[index]));
                    index += 1;
                }
            }
            symbols = next_symbols;
        }

        let mut ids = Vec::with_capacity(symbols.len());
        for symbol in &symbols {
            if let Some(&id) = self.token_to_id.get(symbol.as_str()) {
                ids.push(id);
            } else {
                // A well-formed byte-level vocabulary contains every
                // single-byte symbol; fall back per character and skip
                // anything the vocabulary genuinely lacks.
                let mut utf8 = [0u8; 4];
                for ch in symbol.chars() {
                    if let Some(&id) = self.token_to_id.get(ch.encode_utf8(&mut utf8)) {
                        ids.push(id);
                    }
                }
            }
        }
        out.extend_from_slice(&ids);
        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if cache.len() < CACHE_CAPACITY {
            cache.insert(pre_token.to_owned(), ids);
        }
    }
}

/// Parsed pre-tokenizer configuration.
struct PreTokenizerConfig {
    /// `add_prefix_space` on the `ByteLevel` step (the tokenizers-library
    /// default is true).
    add_prefix_space: bool,
    /// `Some(individual_digits)` when a `Digits` step precedes `ByteLevel`
    /// (SmolLM2: `Digits { individual_digits: true }`).
    digits_individual: Option<bool>,
}

/// Confirm the pre-tokenizer is `ByteLevel`, optionally preceded by a
/// `Digits` step in a `Sequence`. Any other configuration is rejected —
/// approximating an unknown pre-tokenizer would reintroduce exactly the
/// wrong-segmentation bug this module fixes.
fn parse_pre_tokenizer(pre: Option<&serde_json::Value>) -> Option<PreTokenizerConfig> {
    let pre = pre.filter(|value| !value.is_null())?;
    let steps: Vec<&serde_json::Value> = match pre.get("type").and_then(serde_json::Value::as_str) {
        Some("Sequence") => pre
            .get("pretokenizers")
            .and_then(serde_json::Value::as_array)
            .map(|steps| steps.iter().collect())?,
        Some(_) => vec![pre],
        None => return None,
    };
    let mut config: Option<PreTokenizerConfig> = None;
    let mut digits_individual = None;
    for step in steps {
        match step.get("type").and_then(serde_json::Value::as_str) {
            Some("ByteLevel") if config.is_none() => {
                config = Some(PreTokenizerConfig {
                    add_prefix_space: step
                        .get("add_prefix_space")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    digits_individual,
                });
            }
            Some("Digits") if config.is_none() => {
                digits_individual = Some(
                    step.get("individual_digits")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                );
            }
            _ => return None,
        }
    }
    config
}

/// The `Digits` pre-tokenizer: split into numeric and non-numeric runs
/// (`char::is_numeric`, the tokenizers-library predicate); with
/// `individual`, every numeric character becomes its own split.
fn split_digits(text: &str, individual: bool) -> Vec<&str> {
    let mut splits = Vec::new();
    let mut start = 0usize;
    let mut in_digits = false;
    for (offset, ch) in text.char_indices() {
        let numeric = ch.is_numeric();
        if offset > 0 && (numeric != in_digits || (numeric && individual)) {
            splits.push(&text[start..offset]);
            start = offset;
        }
        in_digits = numeric;
    }
    if start < text.len() {
        splits.push(&text[start..]);
    }
    splits
}

/// GPT-2 byte-level pre-tokenization, hand-rolled from the reference
/// pattern `'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|`
/// `\s+(?!\S)|\s+`: contractions bind without a preceding space; a single
/// leading space attaches to the following letter/number/punctuation run;
/// a whitespace run followed by a non-space keeps its last character for
/// the next pre-token.
fn pre_tokenize(text: &str) -> Vec<&str> {
    const CONTRACTIONS: [&str; 7] = ["'s", "'t", "'re", "'ve", "'m", "'ll", "'d"];
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let byte_at = |index: usize| chars.get(index).map_or(text.len(), |(offset, _)| *offset);
    let mut pieces = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let (offset, ch) = chars[i];
        if ch == '\'' {
            if let Some(contraction) = CONTRACTIONS
                .iter()
                .find(|candidate| text[offset..].starts_with(**candidate))
            {
                pieces.push(&text[offset..offset + contraction.len()]);
                i += contraction.chars().count();
                continue;
            }
        }
        let after_space = if ch == ' ' { i + 1 } else { i };
        if let Some(&(_, next)) = chars.get(after_space) {
            if next.is_alphabetic() {
                let mut end = after_space;
                while end < chars.len() && chars[end].1.is_alphabetic() {
                    end += 1;
                }
                pieces.push(&text[offset..byte_at(end)]);
                i = end;
                continue;
            }
            if next.is_numeric() {
                let mut end = after_space;
                while end < chars.len() && chars[end].1.is_numeric() {
                    end += 1;
                }
                pieces.push(&text[offset..byte_at(end)]);
                i = end;
                continue;
            }
            if !next.is_whitespace() {
                let mut end = after_space;
                while end < chars.len()
                    && !chars[end].1.is_whitespace()
                    && !chars[end].1.is_alphabetic()
                    && !chars[end].1.is_numeric()
                {
                    end += 1;
                }
                pieces.push(&text[offset..byte_at(end)]);
                i = end;
                continue;
            }
        }
        // Whitespace run (every non-whitespace character was consumed by a
        // branch above, so `ch` is whitespace here).
        let mut end = i;
        while end < chars.len() && chars[end].1.is_whitespace() {
            end += 1;
        }
        if end < chars.len() && end - i > 1 {
            // `\s+(?!\S)`: leave the final whitespace character to attach
            // to the following pre-token.
            end -= 1;
        }
        pieces.push(&text[offset..byte_at(end)]);
        i = end;
    }
    pieces
}

/// Declared encode/decode policy of a versioned tokenizer adapter
/// (#601). Every field is a stable machine token entering the canonical
/// digest serialization byte-for-byte, mirroring the #600
/// `GeometryProjectionParams` convention.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenizerAdapterPolicy {
    /// Normalizer the adapter applies before pre-tokenization
    /// (`"none"`: this adapter never rewrites input text).
    #[serde(default)]
    pub normalizer: String,
    /// Pre-tokenizer steps in application order, e.g.
    /// `digits(individual_digits=true)` then
    /// `byte-level(add_prefix_space=false)`.
    #[serde(default)]
    pub pre_tokenizers: Vec<String>,
    /// How bytes outside merged tokens are represented
    /// (`"byte-level-alphabet"`: the GPT-2 byte alphabet covers all 256
    /// byte values, so encoding is total).
    #[serde(default)]
    pub byte_fallback: String,
    /// Number of added (special) tokens matched atomically before
    /// pre-tokenization.
    #[serde(default)]
    pub added_tokens_count: u32,
    /// `blake3:<hex>` over the canonical added-token listing (entries
    /// sorted by id, each `<id>:<byte length>:<content>\n`).
    #[serde(default)]
    pub added_tokens_digest: String,
    /// BOS insertion policy (`"none"`: encode inserts no BOS token).
    #[serde(default)]
    pub bos: String,
    /// EOS insertion policy (`"none"`: encode inserts no EOS token).
    #[serde(default)]
    pub eos: String,
    /// Chat-template handling (`"not-interpreted"`: any
    /// `chat_template` in the source snapshot is provenance-pinned by
    /// the tokenizer CID but never executed by this adapter).
    #[serde(default)]
    pub chat_template_policy: String,
}

/// The typed, versioned tokenizer-adapter identity record (#601),
/// mirroring the #600 `GeometryProjection` pattern: `{family, version,
/// tokenizer_cid, policy, adapter_digest}` with a canonical
/// serialization and a digest over it, carried by provenance surfaces
/// (the observation manifest) wherever the producing pipeline knows its
/// tokenizer. A behavioral change to an adapter family is a new
/// version — a new registry entry — never an in-place edit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenizerAdapter {
    /// Registry family id (e.g. [`TokenizerAdapter::HF_BYTE_BPE_FAMILY`]).
    #[serde(default)]
    pub family: String,
    /// Registry version of the family's implementation.
    #[serde(default)]
    pub version: u32,
    /// Content address of the raw tokenizer definition bytes
    /// (`blake3:<hex>` of `tokenizer.json`, exactly how tokenizer CIDs
    /// are formed today by [`HfBpeTokenizer::address`]).
    #[serde(default)]
    pub tokenizer_cid: String,
    /// The declared encode/decode policy.
    #[serde(default)]
    pub policy: TokenizerAdapterPolicy,
    /// `blake3:<hex>` of [`TokenizerAdapter::canonical_bytes`] — the
    /// declared identity, not source code text (renames and formatting
    /// must not move the digest; a behavioral change must bump
    /// `version` instead).
    #[serde(default)]
    pub adapter_digest: String,
}

impl TokenizerAdapter {
    /// Registry family of the Hugging Face byte-level BPE adapter
    /// implemented by [`HfBpeTokenizer`].
    pub const HF_BYTE_BPE_FAMILY: &'static str = "hf-byte-bpe";
    /// Registry version of the byte-level BPE adapter currently
    /// implemented (the post-#242/#253 verified behavior).
    pub const HF_BYTE_BPE_VERSION: u32 = 1;
    /// SentencePiece/Unigram adapter family (#639-3). Implemented by
    /// [`super::sentencepiece::SentencePieceUnigramTokenizer`]: precompiled
    /// charsmap normalization + Unigram Viterbi segmentation.
    pub const SENTENCEPIECE_UNIGRAM_FAMILY: &'static str = "sentencepiece-unigram";
    /// Frozen registry version published by #639-3b. Version 1 decodes the
    /// UNKNOWN id through its literal vocabulary surface (`<unk>` for T5).
    pub const SENTENCEPIECE_UNIGRAM_V1_VERSION: u32 = 1;
    /// Reference-correct registry version delivered by #718. Version 2 uses
    /// `TrainerSpec.unk_surface` during sequence decoding, matching the pinned
    /// SentencePiece reference implementation.
    pub const SENTENCEPIECE_UNIGRAM_V2_VERSION: u32 = 2;
    /// Current SentencePiece/Unigram adapter version. Published versions stay
    /// independently resolvable; this alias is only the auto-selection target.
    pub const SENTENCEPIECE_UNIGRAM_VERSION: u32 = Self::SENTENCEPIECE_UNIGRAM_V2_VERSION;

    /// Canonical serialization of the adapter identity: a fixed line
    /// format (format tag then `key=value\n` per field, pre-tokenizer
    /// steps joined with `,`). Byte-stable by construction — field
    /// order and separators are fixed here, not derived from any
    /// serializer — so the digest over these bytes is reproducible
    /// everywhere.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "uor-r4-tokenizer-adapter/1\n\
             family={}\n\
             version={}\n\
             tokenizer_cid={}\n\
             policy.normalizer={}\n\
             policy.pre_tokenizers={}\n\
             policy.byte_fallback={}\n\
             policy.added_tokens_count={}\n\
             policy.added_tokens_digest={}\n\
             policy.bos={}\n\
             policy.eos={}\n\
             policy.chat_template_policy={}\n",
            self.family,
            self.version,
            self.tokenizer_cid,
            self.policy.normalizer,
            self.policy.pre_tokenizers.join(","),
            self.policy.byte_fallback,
            self.policy.added_tokens_count,
            self.policy.added_tokens_digest,
            self.policy.bos,
            self.policy.eos,
            self.policy.chat_template_policy,
        )
        .into_bytes()
    }

    /// The adapter digest this record's declared identity implies:
    /// `blake3:<hex>` over [`TokenizerAdapter::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
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
        let pieces: Vec<Vec<u8>> = (0..self.model_vocab_len)
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
