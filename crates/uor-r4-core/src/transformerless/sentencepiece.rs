//! SentencePiece Unigram model (#639-3a): a dependency-free parser for a
//! `spiece.model` `ModelProto` and the Unigram Viterbi encode/decode core.
//!
//! This is the algorithmic half of the #639-3 SentencePiece/Unigram
//! adapter. It parses only the fields the Unigram model needs — the
//! `pieces` (surface, log-prob `score`, `type`) and the `trainer_spec`
//! discriminants (`model_type`, `byte_fallback`, `unk_id`) — with a
//! minimal protobuf reader, and implements the Unigram best-path
//! segmentation (Viterbi) over **already-normalized** text.
//!
//! Since #639-3b this module also carries the [`Normalizer`] (the
//! `nmt_nfkc` precompiled-charsmap folding + dummy-prefix/whitespace rules)
//! and [`SentencePieceUnigramTokenizer`], which composes the normalizer with
//! the Viterbi core into a [`super::hf_bpe::TokenizerModel`] registered under
//! `(sentencepiece-unigram, 1)`. The `UnigramModel::encode_normalized` entry
//! point still operates on already-normalized text; the tokenizer's
//! `encode` normalizes first. Selecting this adapter from the observe/serve
//! drivers (the `TokenizerKind` wiring) is the remaining follow-up.
//!
//! Refuse-by-name, never approximate: a non-Unigram `model_type`, a
//! `byte_fallback` source (no pinned byte-fallback source exists yet), a
//! missing or ambiguous `<unk>` piece, or malformed proto bytes each fail
//! closed with a named [`SourceUnavailable`] rather than a guess.

use std::collections::HashMap;

use uor_r4_model_source::SourceUnavailable;

use super::hf_bpe::{TokenizerAdapter, TokenizerAdapterPolicy, TokenizerModel};

/// The whitespace meta symbol SentencePiece escapes spaces to (`▁`,
/// U+2581). Pieces carry it literally; [`UnigramModel::decode`] maps it
/// back to a space.
const WHITESPACE_META: char = '\u{2581}';

/// SentencePiece's fixed penalty added (as a subtraction from the minimum
/// piece score) to an unknown single-character node, matching
/// `unigram_model.cc`'s `kUnkPenalty`.
const UNK_PENALTY: f32 = 10.0;

/// `ModelProto.SentencePiece.Type` (proto enum values).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PieceType {
    Normal,
    Unknown,
    Control,
    UserDefined,
    Unused,
    Byte,
}

impl PieceType {
    fn from_proto(value: u64) -> Option<Self> {
        match value {
            1 => Some(PieceType::Normal),
            2 => Some(PieceType::Unknown),
            3 => Some(PieceType::Control),
            4 => Some(PieceType::UserDefined),
            5 => Some(PieceType::Unused),
            6 => Some(PieceType::Byte),
            _ => None,
        }
    }

    /// Whether a piece can be matched against input text. Control and
    /// unknown symbols are produced only by the model (not by matching
    /// surface text), and unused pieces never participate.
    fn is_insertable(self) -> bool {
        matches!(
            self,
            PieceType::Normal | PieceType::UserDefined | PieceType::Byte
        )
    }
}

/// A parsed SentencePiece Unigram model: the vocabulary surfaces and
/// per-piece log-prob scores plus the discriminants the Viterbi core
/// needs. Encode/decode operate on already-normalized text (see the
/// module docs); the normalizer arrives in #639-3b.
#[derive(Debug)]
pub struct UnigramModel {
    /// id → piece surface (byte-for-byte as declared, `▁`-escaped).
    surfaces: Vec<String>,
    /// id → piece type.
    types: Vec<PieceType>,
    /// id → log-prob score.
    scores: Vec<f32>,
    /// Surface → id for insertable pieces (Normal / UserDefined / Byte);
    /// the first id wins on a duplicate surface, matching the vocabulary's
    /// declaration order.
    insertable: HashMap<String, u32>,
    /// The single UNKNOWN piece's id.
    unk_id: u32,
    /// Minimum score across all pieces (the base for the unknown-node
    /// penalty).
    min_score: f32,
    /// Longest insertable piece in bytes (bounds the lattice inner loop).
    max_piece_len: usize,
}

impl UnigramModel {
    /// Parse a Unigram model from raw `spiece.model` `ModelProto` bytes.
    ///
    /// Fails closed with a named [`SourceUnavailable`] on: malformed proto
    /// bytes, a `model_type` that is explicitly not `UNIGRAM`, a
    /// `byte_fallback` source (unsupported until a byte-fallback source is
    /// pinned), a non-UTF-8 piece surface, an unknown piece-type value, or a
    /// missing / ambiguous `<unk>` piece.
    pub fn from_spiece_bytes(bytes: &[u8]) -> Result<Self, SourceUnavailable> {
        let mut reader = ProtoReader::new(bytes);
        let mut surfaces: Vec<String> = Vec::new();
        let mut types: Vec<PieceType> = Vec::new();
        let mut scores: Vec<f32> = Vec::new();
        let mut model_type: Option<u64> = None;
        let mut byte_fallback = false;
        let mut trainer_unk_id: Option<i64> = None;

        while !reader.is_empty() {
            let (field, wire) = reader
                .read_tag()
                .ok_or_else(|| SourceUnavailable::new("spiece.model: truncated field tag"))?;
            match (field, wire) {
                // ModelProto.pieces (repeated SentencePiece message).
                (1, WIRE_LEN) => {
                    let message = reader.read_len_delim().ok_or_else(|| {
                        SourceUnavailable::new("spiece.model: truncated piece message")
                    })?;
                    let (surface, score, piece_type) = parse_piece(message)?;
                    surfaces.push(surface);
                    scores.push(score);
                    types.push(piece_type);
                }
                // ModelProto.trainer_spec.
                (2, WIRE_LEN) => {
                    let message = reader.read_len_delim().ok_or_else(|| {
                        SourceUnavailable::new("spiece.model: truncated trainer_spec")
                    })?;
                    parse_trainer_spec(
                        message,
                        &mut model_type,
                        &mut byte_fallback,
                        &mut trainer_unk_id,
                    )?;
                }
                _ => reader
                    .skip_field(wire)
                    .ok_or_else(|| SourceUnavailable::new("spiece.model: unreadable field"))?,
            }
        }

        if let Some(other) = model_type {
            if other != MODEL_TYPE_UNIGRAM {
                return Err(SourceUnavailable::new(format!(
                    "spiece.model: model_type {other} is not UNIGRAM (1); \
                     sentencepiece-unigram covers Unigram sources only"
                )));
            }
        }
        if byte_fallback {
            return Err(SourceUnavailable::new(
                "spiece.model: byte_fallback=true is not supported by sentencepiece-unigram/1 \
                 (no byte-fallback source is pinned); refused rather than approximated",
            ));
        }
        if surfaces.is_empty() {
            return Err(SourceUnavailable::new("spiece.model: no pieces"));
        }

        let unknown: Vec<u32> = types
            .iter()
            .enumerate()
            .filter(|(_, &kind)| kind == PieceType::Unknown)
            .map(|(id, _)| id as u32)
            .collect();
        let unk_id = match unknown.as_slice() {
            [id] => *id,
            [] => {
                return Err(SourceUnavailable::new(
                    "spiece.model: no UNKNOWN piece to anchor unmatched spans",
                ))
            }
            _ => {
                return Err(SourceUnavailable::new(
                    "spiece.model: multiple UNKNOWN pieces (ambiguous <unk>)",
                ))
            }
        };
        if let Some(declared) = trainer_unk_id {
            if declared >= 0 && declared as u32 != unk_id {
                return Err(SourceUnavailable::new(format!(
                    "spiece.model: trainer_spec unk_id {declared} disagrees with the UNKNOWN \
                     piece id {unk_id}"
                )));
            }
        }

        let mut insertable: HashMap<String, u32> = HashMap::with_capacity(surfaces.len());
        let mut max_piece_len = 0usize;
        let mut min_score = f32::INFINITY;
        for (id, surface) in surfaces.iter().enumerate() {
            min_score = min_score.min(scores[id]);
            if types[id].is_insertable() {
                max_piece_len = max_piece_len.max(surface.len());
                insertable.entry(surface.clone()).or_insert(id as u32);
            }
        }

        Ok(Self {
            surfaces,
            types,
            scores,
            insertable,
            unk_id,
            min_score,
            max_piece_len,
        })
    }

    /// Number of pieces (id slots) in the vocabulary.
    pub fn vocab_size(&self) -> usize {
        self.surfaces.len()
    }

    /// The UNKNOWN piece's id.
    pub fn unk_id(&self) -> u32 {
        self.unk_id
    }

    /// Encode already-normalized text (the `▁`-escaped surface form) to
    /// token ids via Unigram best-path (Viterbi) segmentation.
    ///
    /// Every character is covered: a span with no insertable piece becomes
    /// a single-character unknown node scored `min_score - kUnkPenalty`, and
    /// consecutive unknown characters collapse to a single `<unk>` in the
    /// output — matching SentencePiece exactly. The scoring uses per-
    /// character unknown nodes; only the emitted output merges them.
    pub fn encode_normalized(&self, text: &str) -> Vec<u32> {
        if text.is_empty() {
            return Vec::new();
        }
        let len = text.len();
        // Byte offsets of every character boundary, terminated by `len`.
        let mut boundary: Vec<usize> = text.char_indices().map(|(offset, _)| offset).collect();
        boundary.push(len);
        let char_count = boundary.len() - 1;

        let unk_score = self.min_score - UNK_PENALTY;
        let mut best = vec![f32::NEG_INFINITY; len + 1];
        best[0] = 0.0;
        // Backpointer per byte boundary: (previous byte offset, piece id).
        let mut back: Vec<(usize, u32)> = vec![(usize::MAX, u32::MAX); len + 1];

        for start_char in 0..char_count {
            let start = boundary[start_char];
            let reach = best[start];
            if reach == f32::NEG_INFINITY {
                continue;
            }
            let mut has_single_char = false;
            for (end_char, &end) in boundary.iter().enumerate().skip(start_char + 1) {
                if end - start > self.max_piece_len {
                    break;
                }
                if let Some(&id) = self.insertable.get(&text[start..end]) {
                    let candidate = reach + self.scores[id as usize];
                    if candidate > best[end] {
                        best[end] = candidate;
                        back[end] = (start, id);
                    }
                    if end_char - start_char == 1 {
                        has_single_char = true;
                    }
                }
            }
            if !has_single_char {
                let end = boundary[start_char + 1];
                let candidate = reach + unk_score;
                if candidate > best[end] {
                    best[end] = candidate;
                    back[end] = (start, self.unk_id);
                }
            }
        }

        let mut ids: Vec<u32> = Vec::new();
        let mut offset = len;
        while offset > 0 {
            let (previous, id) = back[offset];
            debug_assert!(previous != usize::MAX, "every boundary is reachable");
            ids.push(id);
            offset = previous;
        }
        ids.reverse();

        // Collapse runs of consecutive unknown ids into a single <unk>.
        let mut out: Vec<u32> = Vec::with_capacity(ids.len());
        for id in ids {
            if id == self.unk_id && out.last() == Some(&self.unk_id) {
                continue;
            }
            out.push(id);
        }
        out
    }

    /// Decode token ids to the normalized surface text: concatenate piece
    /// surfaces (control symbols contribute nothing), map `▁` back to a
    /// space, and drop the single leading space a dummy prefix would have
    /// added. Out-of-range ids are skipped.
    ///
    /// This inverts [`UnigramModel::encode_normalized`] up to lossy spans
    /// (an `<unk>` cannot recover the original bytes). Full denormalization
    /// is a #639-3b concern; this is the surface inverse of the Viterbi.
    pub fn decode(&self, ids: &[u32]) -> String {
        let mut surface = String::new();
        for &id in ids {
            let Some(piece) = self.surfaces.get(id as usize) else {
                continue;
            };
            if self.types[id as usize] == PieceType::Control {
                continue;
            }
            surface.push_str(piece);
        }
        let spaced = surface.replace(WHITESPACE_META, " ");
        spaced.strip_prefix(' ').unwrap_or(&spaced).to_owned()
    }
}

// ============================ #639-3b: normalizer + adapter ===============

/// Darts-clone double-array unit accessors (the trie encoding
/// `precompiled_charsmap` uses). A unit is a `u32`.
fn darts_has_leaf(unit: u32) -> bool {
    (unit >> 8) & 1 == 1
}
fn darts_value(unit: u32) -> u32 {
    unit & 0x7fff_ffff
}
fn darts_offset(unit: u32) -> u32 {
    (unit >> 10) << ((unit & 0x200) >> 6)
}

/// The SentencePiece text normalizer: applies the `precompiled_charsmap`
/// (a Darts double-array trie of longest-match byte-sequence replacements —
/// e.g. NFKC folding for `nmt_nfkc`) then the whitespace rules
/// (`add_dummy_prefix`, escape space → `▁`, collapse/strip extra
/// whitespace). This reproduces `sentencepiece`'s `Normalizer::Normalize`
/// byte-for-byte; the algorithm and the pinned T5 `nmt_nfkc` charsmap were
/// validated against the reference library before this port.
#[derive(Debug)]
pub struct Normalizer {
    /// Declared normalizer name (provenance only; the charsmap drives
    /// behavior).
    name: String,
    /// Darts-clone double-array trie units.
    trie: Vec<u32>,
    /// Null-separated normalized replacement strings; a trie value is a byte
    /// offset into this blob (offset 0 is the empty string).
    normalized: Vec<u8>,
    add_dummy_prefix: bool,
    remove_extra_whitespaces: bool,
    escape_whitespaces: bool,
}

impl Normalizer {
    /// Parse the `NormalizerSpec` (`ModelProto` field 3) from raw
    /// `spiece.model` bytes and decode its `precompiled_charsmap`.
    ///
    /// Fails closed via [`SourceUnavailable`] when the model carries no
    /// `NormalizerSpec`, when a `normalization_rule_tsv` is present (custom
    /// rules are refused, never approximated), or when the
    /// `precompiled_charsmap` is absent or malformed — only
    /// precompiled-charsmap normalizers are supported.
    pub fn from_spiece_bytes(bytes: &[u8]) -> Result<Self, SourceUnavailable> {
        // Locate the normalizer_spec sub-message (ModelProto field 3).
        let mut reader = ProtoReader::new(bytes);
        let mut spec: Option<&[u8]> = None;
        while !reader.is_empty() {
            let (field, wire) = reader
                .read_tag()
                .ok_or_else(|| SourceUnavailable::new("spiece.model: truncated field tag"))?;
            if (field, wire) == (3, WIRE_LEN) {
                spec = Some(reader.read_len_delim().ok_or_else(|| {
                    SourceUnavailable::new("spiece.model: truncated normalizer_spec")
                })?);
            } else {
                reader
                    .skip_field(wire)
                    .ok_or_else(|| SourceUnavailable::new("spiece.model: unreadable field"))?;
            }
        }
        let spec =
            spec.ok_or_else(|| SourceUnavailable::new("spiece.model: no normalizer_spec"))?;

        // NormalizerSpec fields: name(1), precompiled_charsmap(2 bytes),
        // add_dummy_prefix(3), remove_extra_whitespaces(4),
        // escape_whitespaces(5), normalization_rule_tsv(6). Booleans default
        // to true (the SentencePiece proto defaults).
        let mut name = String::new();
        let mut charsmap: Option<&[u8]> = None;
        let mut add_dummy_prefix = true;
        let mut remove_extra_whitespaces = true;
        let mut escape_whitespaces = true;
        let mut reader = ProtoReader::new(spec);
        while !reader.is_empty() {
            let (field, wire) = reader
                .read_tag()
                .ok_or_else(|| SourceUnavailable::new("normalizer_spec: truncated field tag"))?;
            match (field, wire) {
                (1, WIRE_LEN) => {
                    let raw = reader
                        .read_len_delim()
                        .ok_or_else(|| SourceUnavailable::new("normalizer_spec: truncated name"))?;
                    name = std::str::from_utf8(raw)
                        .map_err(|_| SourceUnavailable::new("normalizer_spec: non-UTF-8 name"))?
                        .to_owned();
                }
                (2, WIRE_LEN) => {
                    charsmap = Some(reader.read_len_delim().ok_or_else(|| {
                        SourceUnavailable::new("normalizer_spec: truncated precompiled_charsmap")
                    })?);
                }
                (3, WIRE_VARINT) => {
                    add_dummy_prefix = reader.read_varint().ok_or_else(|| {
                        SourceUnavailable::new("normalizer_spec: truncated add_dummy_prefix")
                    })? != 0;
                }
                (4, WIRE_VARINT) => {
                    remove_extra_whitespaces = reader.read_varint().ok_or_else(|| {
                        SourceUnavailable::new(
                            "normalizer_spec: truncated remove_extra_whitespaces",
                        )
                    })? != 0;
                }
                (5, WIRE_VARINT) => {
                    escape_whitespaces = reader.read_varint().ok_or_else(|| {
                        SourceUnavailable::new("normalizer_spec: truncated escape_whitespaces")
                    })? != 0;
                }
                (6, WIRE_LEN) => {
                    let tsv = reader.read_len_delim().ok_or_else(|| {
                        SourceUnavailable::new("normalizer_spec: truncated normalization_rule_tsv")
                    })?;
                    if !tsv.is_empty() {
                        return Err(SourceUnavailable::new(
                            "normalizer_spec: normalization_rule_tsv is not interpreted — refused \
                             rather than approximated",
                        ));
                    }
                }
                _ => reader
                    .skip_field(wire)
                    .ok_or_else(|| SourceUnavailable::new("normalizer_spec: unreadable field"))?,
            }
        }

        let charsmap = charsmap.filter(|blob| !blob.is_empty()).ok_or_else(|| {
            SourceUnavailable::new(
                "normalizer_spec: no precompiled_charsmap — only precompiled-charsmap normalizers \
                 are supported",
            )
        })?;
        // Charsmap layout: [u32 LE trie_blob_size][trie: u32 units][normalized
        // blob: null-separated strings].
        if charsmap.len() < 4 {
            return Err(SourceUnavailable::new(
                "normalizer_spec: precompiled_charsmap shorter than its trie-size header",
            ));
        }
        let trie_size =
            u32::from_le_bytes([charsmap[0], charsmap[1], charsmap[2], charsmap[3]]) as usize;
        if !trie_size.is_multiple_of(4) || 4usize.saturating_add(trie_size) > charsmap.len() {
            return Err(SourceUnavailable::new(
                "normalizer_spec: precompiled_charsmap trie size is malformed",
            ));
        }
        let trie: Vec<u32> = charsmap[4..4 + trie_size]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        let normalized = charsmap[4 + trie_size..].to_vec();

        Ok(Self {
            name,
            trie,
            normalized,
            add_dummy_prefix,
            remove_extra_whitespaces,
            escape_whitespaces,
        })
    }

    /// Darts `commonPrefixSearch`: the LONGEST key prefix present in the
    /// trie, as `(value offset into the normalized blob, matched byte
    /// length)`. `None` when no prefix matches.
    fn longest_match(&self, key: &[u8]) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize)> = None;
        let mut node_pos = 0usize;
        let mut unit = *self.trie.first()?;
        node_pos ^= darts_offset(unit) as usize;
        for (i, &byte) in key.iter().enumerate() {
            node_pos ^= byte as usize;
            unit = *self.trie.get(node_pos)?;
            if (unit & 0xff) as u8 != byte {
                break;
            }
            node_pos ^= darts_offset(unit) as usize;
            if darts_has_leaf(unit) {
                let leaf = *self.trie.get(node_pos)?;
                best = Some((darts_value(leaf) as usize, i + 1));
            }
        }
        best
    }

    /// The null-terminated normalized replacement string at `offset`.
    fn replacement(&self, offset: usize) -> Option<&[u8]> {
        let rest = self.normalized.get(offset..)?;
        let end = rest.iter().position(|&b| b == 0)?;
        Some(&rest[..end])
    }

    /// Apply the `precompiled_charsmap` with greedy longest match; bytes with
    /// no matching prefix are copied through unchanged.
    fn apply_charsmap(&self, input: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(input.len());
        let mut i = 0;
        while i < input.len() {
            if let Some((value, len)) = self.longest_match(&input[i..]) {
                if let Some(replacement) = self.replacement(value) {
                    out.extend_from_slice(replacement);
                    i += len;
                    continue;
                }
            }
            out.push(input[i]);
            i += 1;
        }
        out
    }

    /// Normalize raw text to the `▁`-escaped surface form the Unigram Viterbi
    /// consumes — reproducing `sentencepiece`'s `Normalize`: charsmap folding,
    /// then whitespace escaping/collapsing, then the dummy prefix (added only
    /// when the result is non-empty).
    pub fn normalize(&self, text: &str) -> String {
        let mapped = self.apply_charsmap(text.as_bytes());
        let mapped = String::from_utf8_lossy(&mapped);

        let space = if self.escape_whitespaces {
            WHITESPACE_META
        } else {
            ' '
        };
        let mut out = String::with_capacity(mapped.len() + WHITESPACE_META.len_utf8());
        let mut prev_space = self.remove_extra_whitespaces;
        for ch in mapped.chars() {
            if ch == ' ' {
                if self.remove_extra_whitespaces && prev_space {
                    continue;
                }
                out.push(space);
                prev_space = true;
            } else {
                out.push(ch);
                prev_space = false;
            }
        }
        if self.remove_extra_whitespaces && out.ends_with(space) {
            out.pop();
        }
        if self.add_dummy_prefix && !out.is_empty() {
            out.insert(0, space);
        }
        out
    }
}

/// The registered SentencePiece Unigram tokenizer (#639-3b): raw text →
/// [`Normalizer`] → [`UnigramModel`] Viterbi → token ids, exposed as a
/// [`TokenizerModel`] under the `(sentencepiece-unigram, 1)` registry entry.
#[derive(Debug)]
pub struct SentencePieceUnigramTokenizer {
    model: UnigramModel,
    normalizer: Normalizer,
    tokenizer_cid: String,
}

impl SentencePieceUnigramTokenizer {
    /// Build from raw `spiece.model` bytes: the Unigram model, its
    /// normalizer, and the `blake3` content address of the definition.
    pub fn from_spiece_bytes(bytes: &[u8]) -> Result<Self, SourceUnavailable> {
        let model = UnigramModel::from_spiece_bytes(bytes)?;
        let normalizer = Normalizer::from_spiece_bytes(bytes)?;
        let tokenizer_cid = format!("blake3:{}", blake3::hash(bytes).to_hex());
        Ok(Self {
            model,
            normalizer,
            tokenizer_cid,
        })
    }

    /// Canonical listing digest + count of the non-Normal pieces (control,
    /// unknown, user-defined, unused) — sorted by id, each
    /// `<id>:<byte length>:<content>\n` — mirroring the byte-level BPE
    /// added-token digest so provenance distinguishes vocabularies.
    fn special_tokens(&self) -> (u32, String) {
        let mut listing = Vec::new();
        let mut count = 0u32;
        for (id, kind) in self.model.types.iter().enumerate() {
            if *kind != PieceType::Normal {
                let content = &self.model.surfaces[id];
                listing.extend_from_slice(format!("{id}:{}:", content.len()).as_bytes());
                listing.extend_from_slice(content.as_bytes());
                listing.push(b'\n');
                count += 1;
            }
        }
        (count, format!("blake3:{}", blake3::hash(&listing).to_hex()))
    }
}

impl TokenizerModel for SentencePieceUnigramTokenizer {
    fn encode(&self, text: &str) -> Vec<u32> {
        self.model
            .encode_normalized(&self.normalizer.normalize(text))
    }

    fn encode_lossy(&self, text: &str) -> (Vec<u32>, u64) {
        // Unmatched characters map to `<unk>` (a real id), so the id stream
        // represents every input; no characters are dropped.
        (self.encode(text), 0)
    }

    fn decode(&self, ids: &[u32]) -> String {
        self.model.decode(ids)
    }

    fn vocab_size(&self) -> usize {
        self.model.vocab_size()
    }

    fn adapter(&self) -> TokenizerAdapter {
        let (added_tokens_count, added_tokens_digest) = self.special_tokens();
        let policy = TokenizerAdapterPolicy {
            normalizer: format!("sentencepiece-precompiled-charsmap({})", self.normalizer.name),
            pre_tokenizers: vec![format!(
                "sentencepiece-whitespace(add_dummy_prefix={},escape_meta={},remove_extra_whitespaces={})",
                self.normalizer.add_dummy_prefix,
                self.normalizer.escape_whitespaces,
                self.normalizer.remove_extra_whitespaces,
            )],
            // Non-byte-fallback: an unmatched span becomes a single `<unk>`.
            byte_fallback: "unknown-token".to_owned(),
            added_tokens_count,
            added_tokens_digest,
            bos: "none".to_owned(),
            eos: "none".to_owned(),
            chat_template_policy: "not-interpreted".to_owned(),
        };
        let mut adapter = TokenizerAdapter {
            family: TokenizerAdapter::SENTENCEPIECE_UNIGRAM_FAMILY.to_owned(),
            version: TokenizerAdapter::SENTENCEPIECE_UNIGRAM_VERSION,
            tokenizer_cid: self.tokenizer_cid.clone(),
            policy,
            adapter_digest: String::new(),
        };
        adapter.adapter_digest = adapter.declared_digest();
        adapter
    }
}

/// `TrainerSpec.model_type` value for a Unigram model.
const MODEL_TYPE_UNIGRAM: u64 = 1;

/// Protobuf wire types used here.
const WIRE_VARINT: u8 = 0;
const WIRE_FIXED64: u8 = 1;
const WIRE_LEN: u8 = 2;
const WIRE_FIXED32: u8 = 5;

/// Parse one `ModelProto.SentencePiece` message: `piece` (field 1, string),
/// `score` (field 2, float / fixed32), `type` (field 3, enum / varint).
fn parse_piece(message: &[u8]) -> Result<(String, f32, PieceType), SourceUnavailable> {
    let mut reader = ProtoReader::new(message);
    let mut surface = String::new();
    let mut score = 0.0f32;
    let mut piece_type = PieceType::Normal;
    while !reader.is_empty() {
        let (field, wire) = reader
            .read_tag()
            .ok_or_else(|| SourceUnavailable::new("piece: truncated field tag"))?;
        match (field, wire) {
            (1, WIRE_LEN) => {
                let raw = reader
                    .read_len_delim()
                    .ok_or_else(|| SourceUnavailable::new("piece: truncated surface"))?;
                surface = std::str::from_utf8(raw)
                    .map_err(|_| SourceUnavailable::new("piece: non-UTF-8 surface"))?
                    .to_owned();
            }
            (2, WIRE_FIXED32) => {
                let bits = reader
                    .read_fixed32()
                    .ok_or_else(|| SourceUnavailable::new("piece: truncated score"))?;
                score = f32::from_bits(bits);
            }
            (3, WIRE_VARINT) => {
                let value = reader
                    .read_varint()
                    .ok_or_else(|| SourceUnavailable::new("piece: truncated type"))?;
                piece_type = PieceType::from_proto(value).ok_or_else(|| {
                    SourceUnavailable::new(format!("piece: unknown piece type {value}"))
                })?;
            }
            _ => reader
                .skip_field(wire)
                .ok_or_else(|| SourceUnavailable::new("piece: unreadable field"))?,
        }
    }
    Ok((surface, score, piece_type))
}

/// Parse the `model_type` (field 3), `byte_fallback` (field 35), and
/// `unk_id` (field 40) discriminants from a `TrainerSpec` message; every
/// other field is skipped.
fn parse_trainer_spec(
    message: &[u8],
    model_type: &mut Option<u64>,
    byte_fallback: &mut bool,
    unk_id: &mut Option<i64>,
) -> Result<(), SourceUnavailable> {
    let mut reader = ProtoReader::new(message);
    while !reader.is_empty() {
        let (field, wire) = reader
            .read_tag()
            .ok_or_else(|| SourceUnavailable::new("trainer_spec: truncated field tag"))?;
        match (field, wire) {
            (3, WIRE_VARINT) => {
                *model_type =
                    Some(reader.read_varint().ok_or_else(|| {
                        SourceUnavailable::new("trainer_spec: truncated model_type")
                    })?);
            }
            (35, WIRE_VARINT) => {
                let value = reader.read_varint().ok_or_else(|| {
                    SourceUnavailable::new("trainer_spec: truncated byte_fallback")
                })?;
                *byte_fallback = value != 0;
            }
            (40, WIRE_VARINT) => {
                let value = reader
                    .read_varint()
                    .ok_or_else(|| SourceUnavailable::new("trainer_spec: truncated unk_id"))?;
                // int32 fields are varint-encoded two's complement.
                *unk_id = Some(value as i64);
            }
            _ => reader
                .skip_field(wire)
                .ok_or_else(|| SourceUnavailable::new("trainer_spec: unreadable field"))?,
        }
    }
    Ok(())
}

/// A minimal, allocation-free protobuf reader over a byte slice. Supports
/// exactly the wire types the SentencePiece fields we read use, plus
/// skipping any other field so unknown fields never derail the parse.
struct ProtoReader<'buf> {
    buf: &'buf [u8],
    pos: usize,
}

impl<'buf> ProtoReader<'buf> {
    fn new(buf: &'buf [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    /// Read a base-128 varint (max 10 bytes). `None` on truncation or
    /// overflow.
    fn read_varint(&mut self) -> Option<u64> {
        let mut value: u64 = 0;
        for shift in (0..64).step_by(7) {
            let byte = *self.buf.get(self.pos)?;
            self.pos += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    /// Read a field tag, returning `(field_number, wire_type)`.
    fn read_tag(&mut self) -> Option<(u64, u8)> {
        let key = self.read_varint()?;
        Some((key >> 3, (key & 0x7) as u8))
    }

    /// Read a length-delimited byte run.
    fn read_len_delim(&mut self) -> Option<&'buf [u8]> {
        let len = self.read_varint()? as usize;
        let end = self.pos.checked_add(len)?;
        let slice = self.buf.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    /// Read a little-endian fixed-32 value.
    fn read_fixed32(&mut self) -> Option<u32> {
        let end = self.pos.checked_add(4)?;
        let bytes = self.buf.get(self.pos..end)?;
        self.pos = end;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Skip a field of the given wire type. `None` on an unknown wire type
    /// or truncation.
    fn skip_field(&mut self, wire: u8) -> Option<()> {
        match wire {
            WIRE_VARINT => self.read_varint().map(|_| ()),
            WIRE_FIXED64 => {
                let end = self.pos.checked_add(8)?;
                self.buf.get(self.pos..end)?;
                self.pos = end;
                Some(())
            }
            WIRE_LEN => self.read_len_delim().map(|_| ()),
            WIRE_FIXED32 => self.read_fixed32().map(|_| ()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emit a minimal `spiece.model` `ModelProto` from `(surface, score,
    /// type)` pieces plus a `TrainerSpec` carrying `model_type` and
    /// `unk_id`, so the parser and Viterbi run in CI without the 791 KB
    /// snapshot.
    fn build_model_proto(pieces: &[(&str, f32, u64)], model_type: u64, unk_id: i64) -> Vec<u8> {
        fn varint(out: &mut Vec<u8>, mut value: u64) {
            loop {
                let mut byte = (value & 0x7f) as u8;
                value >>= 7;
                if value != 0 {
                    byte |= 0x80;
                }
                out.push(byte);
                if value == 0 {
                    break;
                }
            }
        }
        fn tag(out: &mut Vec<u8>, field: u64, wire: u8) {
            varint(out, (field << 3) | u64::from(wire));
        }
        fn len_delim(out: &mut Vec<u8>, field: u64, payload: &[u8]) {
            tag(out, field, WIRE_LEN);
            varint(out, payload.len() as u64);
            out.extend_from_slice(payload);
        }

        let mut proto = Vec::new();
        for (surface, score, kind) in pieces {
            let mut piece = Vec::new();
            len_delim(&mut piece, 1, surface.as_bytes());
            tag(&mut piece, 2, WIRE_FIXED32);
            piece.extend_from_slice(&score.to_bits().to_le_bytes());
            tag(&mut piece, 3, WIRE_VARINT);
            varint(&mut piece, *kind);
            len_delim(&mut proto, 1, &piece);
        }
        let mut trainer = Vec::new();
        tag(&mut trainer, 3, WIRE_VARINT);
        varint(&mut trainer, model_type);
        tag(&mut trainer, 40, WIRE_VARINT);
        varint(&mut trainer, unk_id as u64);
        len_delim(&mut proto, 2, &trainer);
        proto
    }

    // A tiny Unigram vocab: id 0 <unk>, then whitespace-meta and a handful
    // of pieces with scores chosen so the best path is unambiguous.
    fn toy_model() -> UnigramModel {
        let pieces = [
            ("<unk>", 0.0, 2u64),   // id 0, UNKNOWN
            ("\u{2581}", -3.0, 1),  // id 1, "▁"
            ("\u{2581}a", -1.0, 1), // id 2, "▁a"
            ("a", -2.0, 1),         // id 3
            ("b", -2.0, 1),         // id 4
            ("ab", -1.5, 1),        // id 5
        ];
        UnigramModel::from_spiece_bytes(&build_model_proto(&pieces, MODEL_TYPE_UNIGRAM, 0))
            .expect("toy model parses")
    }

    #[test]
    fn parses_pieces_scores_types_and_unk() {
        let model = toy_model();
        assert_eq!(model.vocab_size(), 6);
        assert_eq!(model.unk_id(), 0);
        assert_eq!(model.surfaces[2], "\u{2581}a");
        assert!((model.scores[2] - (-1.0)).abs() < 1e-6);
        assert_eq!(model.types[0], PieceType::Unknown);
    }

    #[test]
    fn viterbi_prefers_the_higher_scoring_segmentation() {
        let model = toy_model();
        // "▁a": the single piece "▁a" (-1.0) beats "▁"(-3.0)+"a"(-2.0)=-5.0.
        assert_eq!(model.encode_normalized("\u{2581}a"), vec![2]);
        // "ab": "ab"(-1.5) beats "a"(-2.0)+"b"(-2.0)=-4.0.
        assert_eq!(model.encode_normalized("ab"), vec![5]);
    }

    #[test]
    fn unmatched_characters_become_unk_and_collapse_when_adjacent() {
        let model = toy_model();
        // 'z' and 'y' are not in the vocab: two adjacent unknowns collapse.
        assert_eq!(model.encode_normalized("zy"), vec![0]);
        // Separated by a known piece, the unknowns stay distinct.
        assert_eq!(model.encode_normalized("zaz"), vec![0, 3, 0]);
        // Empty input encodes to nothing.
        assert!(model.encode_normalized("").is_empty());
    }

    #[test]
    fn decode_maps_meta_to_space_and_drops_the_leading_space() {
        let model = toy_model();
        // "▁a" + "b" → "▁ab" → " ab" → "ab".
        assert_eq!(model.decode(&[2, 4]), "ab");
    }

    #[test]
    fn non_unigram_model_type_is_refused_by_name() {
        // model_type 2 = BPE.
        let proto = build_model_proto(&[("<unk>", 0.0, 2)], 2, 0);
        let error = UnigramModel::from_spiece_bytes(&proto).expect_err("BPE refused");
        assert!(error.reason.contains("model_type 2"));
        assert!(error.reason.contains("UNIGRAM"));
    }

    #[test]
    fn missing_unknown_piece_is_refused() {
        let proto = build_model_proto(&[("a", -1.0, 1)], MODEL_TYPE_UNIGRAM, 0);
        let error = UnigramModel::from_spiece_bytes(&proto).expect_err("no <unk> refused");
        assert!(error.reason.contains("UNKNOWN"));
    }
}
