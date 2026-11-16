//! Donor-corpus shards into the native trainer's token stream (claude-403).
//!
//! Reads JSONL records in two formats, detected by their exact key sets:
//! - D41 pilot: `id`, `kind`, `messages[{role,content}]`, `answers[1..4]` (the first answer
//!   is used);
//! - D87 production: `id`, `kind` (`single` | `two-turn`), `source`,
//!   `messages[{role:"user",content}]`, `turns[{reasoning,answer}]`, one turn per message.
//!
//! Each record becomes BOS, then per turn the user marker and the user text, the assistant
//! marker, the reasoning text, a separator and the answer text, and EOS at the end. Every
//! token carries a segment tag (0 frame, 1 prompt, 2 reasoning, 3 answer) and a response-mask
//! byte: 1 on every token the donor assistant emits after its header (reasoning, separator,
//! answer and the closing EOS), 0 elsewhere.
//!
//! Coverage is counted by aligning each encoded piece back onto its text bytes: a text byte
//! the vocabulary cannot emit as a one-byte token is dropped by the tokenizer (it never
//! becomes `<|unk|>`); a fallback token is an emitted id that decodes to one byte >= 0x80.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};

use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

/// The sheet vocabulary every id must stay below.
pub const VOCAB: u32 = 4096;
/// `<|bos|>`, the first token of every record and of the stream.
pub const BOS: u32 = 0;
/// `<|eos|>`, the last token of every record.
pub const EOS: u32 = 1;
/// The UORT header length and layout of `crates/uor-r4-core/src/native_geometric/mmap_corpus.rs`.
pub const UORT_HEADER: usize = 64;

/// Marker texts. The lab's framing (ORDER.md, open decision); kept in this one place.
pub const USER_MARKER: &str = "\nUser:";
pub const ASSISTANT_MARKER: &str = "\nAssistant:";
pub const SYSTEM_MARKER: &str = "\nSystem:";
/// Between the reasoning and the answer of one assistant turn.
pub const SEPARATOR: &str = "\n\n";

/// A token's segment tag in `segment.u8`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment {
    Frame = 0,
    Prompt = 1,
    Reasoning = 2,
    Answer = 3,
}

pub const SEGMENTS: [Segment; 4] = [
    Segment::Frame,
    Segment::Prompt,
    Segment::Reasoning,
    Segment::Answer,
];

impl Segment {
    pub fn name(self) -> &'static str {
        match self {
            Segment::Frame => "frame",
            Segment::Prompt => "prompt",
            Segment::Reasoning => "reasoning",
            Segment::Answer => "answer",
        }
    }
}

/// Which corpus a record came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    D41,
    D87,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::D41 => "d41",
            Format::D87 => "d87",
        }
    }
}

/// One text piece of a framed record, before tokenization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub segment: Segment,
    /// Response-mask value for every token of this piece.
    pub mask: u8,
    pub text: String,
}

/// A parsed record, framed into pieces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub id: String,
    pub format: Format,
    pub kind: String,
    pub pieces: Vec<Piece>,
}

fn piece(segment: Segment, mask: u8, text: impl Into<String>) -> Piece {
    Piece {
        segment,
        mask,
        text: text.into(),
    }
}

fn key_set(m: &serde_json::Map<String, serde_json::Value>) -> BTreeSet<&str> {
    m.keys().map(String::as_str).collect()
}

fn get_str<'a>(
    m: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
    id: &str,
) -> Result<&'a str, String> {
    m.get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("record {id}: {key} is not a string"))
}

fn messages<'a>(
    m: &'a serde_json::Map<String, serde_json::Value>,
    id: &str,
) -> Result<Vec<(&'a str, &'a str)>, String> {
    let msgs = m
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("record {id}: messages is not an array"))?;
    if msgs.is_empty() {
        return Err(format!("record {id}: no messages"));
    }
    let want: BTreeSet<&str> = ["content", "role"].into_iter().collect();
    msgs.iter()
        .map(|msg| {
            let o = msg
                .as_object()
                .ok_or_else(|| format!("record {id}: a message is not an object"))?;
            if key_set(o) != want {
                return Err(format!("record {id}: message keys {:?}", key_set(o)));
            }
            Ok((get_str(o, "role", id)?, get_str(o, "content", id)?))
        })
        .collect()
}

/// Parse and frame one JSONL record. A record whose key set matches neither format, or whose
/// fields are malformed, is refused with its id.
pub fn parse_record(line: &str) -> Result<Record, String> {
    let v: serde_json::Value = serde_json::from_str(line).map_err(|e| format!("not JSON: {e}"))?;
    let m = v.as_object().ok_or("record is not an object")?;
    let id = m
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("<no id>")
        .to_string();
    let keys = key_set(m);
    let d41: BTreeSet<&str> = ["answers", "id", "kind", "messages"].into_iter().collect();
    let d87: BTreeSet<&str> = ["id", "kind", "messages", "source", "turns"]
        .into_iter()
        .collect();
    let kind = get_str(m, "kind", &id)?.to_string();
    let mut pieces = vec![];
    let format = if keys == d41 {
        let answers = m
            .get("answers")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("record {id}: answers is not an array"))?;
        let answer = answers
            .first()
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("record {id}: no first answer string"))?;
        for (role, content) in messages(m, &id)? {
            let marker = match role {
                "user" => USER_MARKER,
                "assistant" => ASSISTANT_MARKER,
                "system" => SYSTEM_MARKER,
                other => return Err(format!("record {id}: message role {other:?}")),
            };
            pieces.push(piece(Segment::Frame, 0, marker));
            pieces.push(piece(Segment::Prompt, 0, format!(" {content}")));
        }
        pieces.push(piece(Segment::Frame, 0, ASSISTANT_MARKER));
        pieces.push(piece(Segment::Answer, 1, format!(" {answer}")));
        Format::D41
    } else if keys == d87 {
        get_str(m, "source", &id)?;
        if kind != "single" && kind != "two-turn" {
            return Err(format!("record {id}: kind {kind:?}"));
        }
        let msgs = messages(m, &id)?;
        let turns = m
            .get("turns")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("record {id}: turns is not an array"))?;
        if turns.len() != msgs.len() {
            return Err(format!(
                "record {id}: {} turns for {} messages",
                turns.len(),
                msgs.len()
            ));
        }
        let want_turns = if kind == "single" { 1 } else { 2 };
        if turns.len() != want_turns {
            return Err(format!(
                "record {id}: kind {kind} with {} turns",
                turns.len()
            ));
        }
        let want: BTreeSet<&str> = ["answer", "reasoning"].into_iter().collect();
        for ((role, content), turn) in msgs.into_iter().zip(turns) {
            if role != "user" {
                return Err(format!("record {id}: message role {role:?}"));
            }
            let t = turn
                .as_object()
                .ok_or_else(|| format!("record {id}: a turn is not an object"))?;
            if key_set(t) != want {
                return Err(format!("record {id}: turn keys {:?}", key_set(t)));
            }
            let reasoning = get_str(t, "reasoning", &id)?;
            let answer = get_str(t, "answer", &id)?;
            pieces.push(piece(Segment::Frame, 0, USER_MARKER));
            pieces.push(piece(Segment::Prompt, 0, format!(" {content}")));
            pieces.push(piece(Segment::Frame, 0, ASSISTANT_MARKER));
            if reasoning.is_empty() {
                pieces.push(piece(Segment::Answer, 1, format!(" {answer}")));
            } else {
                pieces.push(piece(Segment::Reasoning, 1, format!(" {reasoning}")));
                pieces.push(piece(Segment::Frame, 1, SEPARATOR));
                pieces.push(piece(Segment::Answer, 1, answer));
            }
        }
        Format::D87
    } else {
        return Err(format!("record {id}: unknown key set {keys:?}"));
    };
    Ok(Record {
        id,
        format,
        kind,
        pieces,
    })
}

/// Per-segment coverage counters. Ratios are left to the reader as integer pairs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SegmentCount {
    pub text_bytes: u64,
    pub tokens: u64,
    pub dropped_bytes: u64,
    pub fallback_tokens: u64,
    /// Added-token surfaces (`<|bos|>` and the like) found literally inside text.
    pub special_in_text: u64,
}

/// The tokenizer plus the byte set it can emit as one-byte tokens.
pub struct Coder {
    pub tok: ByteBpeTokenizer,
    emittable: [bool; 256],
    special: BTreeSet<u32>,
}

impl Coder {
    /// Build from raw `tokenizer.json` bytes; refuses a tokenizer whose BOS/EOS ids or
    /// vocabulary do not match the sheet.
    pub fn new(json: &[u8]) -> Result<Self, String> {
        let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(json)
            .ok_or("tokenizer.json is not a byte-level BPE tokenizer")?;
        if tok.token_id("<|bos|>") != Some(BOS) || tok.token_id("<|eos|>") != Some(EOS) {
            return Err("tokenizer BOS/EOS ids are not 0/1".into());
        }
        if tok.vocab_size() > VOCAB as usize {
            return Err(format!(
                "tokenizer has {} ids, over {VOCAB}",
                tok.vocab_size()
            ));
        }
        let mut emittable = [false; 256];
        let mut special = BTreeSet::new();
        for id in 0..tok.vocab_size() as u32 {
            for name in ["<|bos|>", "<|eos|>", "<|unk|>"] {
                if tok.token_id(name) == Some(id) {
                    special.insert(id);
                }
            }
            if special.contains(&id) {
                continue;
            }
            if tok.token_byte_len(id) == Some(1) {
                if let [b] = tok.decode_bytes(&[id])[..] {
                    emittable[b as usize] = true;
                }
            }
        }
        Ok(Self {
            tok,
            emittable,
            special,
        })
    }

    /// The bytes the vocabulary cannot emit as one-byte tokens.
    pub fn unemittable(&self) -> Vec<u8> {
        (0u16..256)
            .map(|b| b as u8)
            .filter(|&b| !self.emittable[b as usize])
            .collect()
    }

    /// Encode one piece, align its ids back onto its text bytes, and count coverage. Each
    /// maximal run of odd bytes (dropped, or inside a fallback token) is passed to `run`.
    pub fn encode_piece(
        &self,
        text: &str,
        count: &mut SegmentCount,
        run: &mut dyn FnMut(&[u8]),
    ) -> Result<Vec<u32>, String> {
        let ids = self.tok.encode(text);
        let bytes = text.as_bytes();
        let mut odd = vec![false; bytes.len()];
        let mut cursor = 0usize;
        let mut emitted = 0u64;
        let mut dropped = 0u64;
        for &id in &ids {
            if id >= VOCAB {
                return Err(format!("id {id} is not below {VOCAB}"));
            }
            let tb = self.tok.decode_bytes(&[id]);
            if self.special.contains(&id) {
                count.special_in_text += 1;
            }
            // Bytes before this token that the vocabulary cannot emit were dropped. (A special
            // token's surface is plain ASCII, so the scan never eats into it.)
            while cursor < bytes.len() && !self.emittable[bytes[cursor] as usize] {
                odd[cursor] = true;
                dropped += 1;
                cursor += 1;
            }
            if bytes.get(cursor..cursor + tb.len()) != Some(&tb[..]) {
                return Err(format!("token {id} does not align at byte {cursor}"));
            }
            if tb.len() == 1 && tb[0] >= 0x80 {
                count.fallback_tokens += 1;
                odd[cursor] = true;
            }
            cursor += tb.len();
            emitted += tb.len() as u64;
        }
        while cursor < bytes.len() {
            if self.emittable[bytes[cursor] as usize] {
                return Err(format!("unencoded emittable byte at {cursor}"));
            }
            odd[cursor] = true;
            dropped += 1;
            cursor += 1;
        }
        // Cross-check: text bytes minus emitted token bytes equals the unemittable bytes.
        let unemittable = bytes
            .iter()
            .filter(|&&b| !self.emittable[b as usize])
            .count() as u64;
        if bytes.len() as u64 - emitted != unemittable || unemittable != dropped {
            return Err("dropped-byte cross-check failed".into());
        }
        count.dropped_bytes += dropped;
        count.text_bytes += bytes.len() as u64;
        count.tokens += ids.len() as u64;
        let mut start = None;
        for i in 0..=bytes.len() {
            let is_odd = i < bytes.len() && odd[i];
            match (start, is_odd) {
                (None, true) => start = Some(i),
                (Some(s), false) => {
                    run(&bytes[s..i]);
                    start = None;
                }
                _ => {}
            }
        }
        Ok(ids)
    }
}

/// A reader that hashes every byte it passes through (the file's sha256, pinned at read time).
pub struct HashRead<R> {
    inner: R,
    pub hasher: Sha256,
}

impl<R: Read> Read for HashRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
}

impl<R> HashRead<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
        }
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The cap on distinct odd-byte runs kept for the top-20 list; runs past it are counted only.
pub const RUN_KEY_CAP: usize = 1 << 20;
/// Runs longer than this are keyed by their first bytes plus a `+` mark.
pub const RUN_KEY_BYTES: usize = 16;

/// What a run counted: the summary's numbers.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub total_tokens: u64,
    pub per_segment: BTreeMap<Segment, SegmentCount>,
    /// Records per `format/kind`.
    pub records: BTreeMap<String, u64>,
    /// Odd-byte runs (hex key) and their counts, capped at `RUN_KEY_CAP` keys.
    pub runs: BTreeMap<String, u64>,
    /// Runs seen after the key cap was reached.
    pub runs_unkeyed: u64,
}

impl Stats {
    /// The top `n` odd-byte runs by count, ties in key order.
    pub fn top_runs(&self, n: usize) -> Vec<(String, u64)> {
        let mut v: Vec<(String, u64)> = self.runs.iter().map(|(k, &c)| (k.clone(), c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }
}

/// Streams records into the three output files (`tokens.u16`, `response_mask.u8`,
/// `segment.u8`) and counts what it wrote.
pub struct Ingest<T: Write + Seek, M: Write, S: Write> {
    tokens: T,
    mask: M,
    segment: S,
    pub stats: Stats,
}

impl<T: Write + Seek, M: Write, S: Write> Ingest<T, M, S> {
    /// Start a token file: a placeholder header, patched with the count in `finish`.
    pub fn new(mut tokens: T, mask: M, segment: S) -> io::Result<Self> {
        tokens.write_all(&[0u8; UORT_HEADER])?;
        let mut stats = Stats::default();
        for &s in &SEGMENTS {
            stats.per_segment.insert(s, SegmentCount::default());
        }
        Ok(Self {
            tokens,
            mask,
            segment,
            stats,
        })
    }

    fn push(&mut self, id: u32, segment: Segment, mask: u8) -> io::Result<()> {
        self.tokens.write_all(&(id as u16).to_le_bytes())?;
        self.mask.write_all(&[mask])?;
        self.segment.write_all(&[segment as u8])?;
        self.stats.total_tokens += 1;
        Ok(())
    }

    /// Tokenize and append one framed record.
    pub fn add(&mut self, coder: &Coder, record: &Record) -> Result<(), String> {
        let io = |e: io::Error| e.to_string();
        self.push(BOS, Segment::Frame, 0).map_err(io)?;
        self.stats
            .per_segment
            .get_mut(&Segment::Frame)
            .unwrap()
            .tokens += 1;
        for p in &record.pieces {
            let mut count = SegmentCount::default();
            let (runs, unkeyed) = (&mut self.stats.runs, &mut self.stats.runs_unkeyed);
            let mut on_run = |r: &[u8]| {
                let key = if r.len() > RUN_KEY_BYTES {
                    format!("{}+", hex(&r[..RUN_KEY_BYTES]))
                } else {
                    hex(r)
                };
                if let Some(n) = runs.get_mut(&key) {
                    *n += 1;
                } else if runs.len() < RUN_KEY_CAP {
                    runs.insert(key, 1);
                } else {
                    *unkeyed += 1;
                }
            };
            let ids = coder
                .encode_piece(&p.text, &mut count, &mut on_run)
                .map_err(|e| format!("record {}: {e}", record.id))?;
            // A literal `<|bos|>`/`<|eos|>`/`<|unk|>` inside donor text would encode as the
            // role id itself: a false record boundary inside a target. Refuse the record.
            if count.special_in_text > 0 {
                return Err(format!(
                    "record {}: a special-token surface inside {} text",
                    record.id,
                    p.segment.name()
                ));
            }
            for id in ids {
                self.push(id, p.segment, p.mask).map_err(io)?;
            }
            let s = self.stats.per_segment.get_mut(&p.segment).unwrap();
            s.text_bytes += count.text_bytes;
            s.tokens += count.tokens;
            s.dropped_bytes += count.dropped_bytes;
            s.fallback_tokens += count.fallback_tokens;
            s.special_in_text += count.special_in_text;
        }
        self.push(EOS, Segment::Frame, 1).map_err(io)?;
        self.stats
            .per_segment
            .get_mut(&Segment::Frame)
            .unwrap()
            .tokens += 1;
        *self
            .stats
            .records
            .entry(format!("{}/{}", record.format.name(), record.kind))
            .or_insert(0) += 1;
        Ok(())
    }

    /// Patch the UORT header with the token count; return the writers and the counts.
    pub fn finish(mut self) -> io::Result<(T, M, S, Stats)> {
        let mut header = [0u8; UORT_HEADER];
        header[0..4].copy_from_slice(b"UORT");
        header[4..8].copy_from_slice(&1u32.to_le_bytes());
        header[8..16].copy_from_slice(&self.stats.total_tokens.to_le_bytes());
        header[16..20].copy_from_slice(&VOCAB.to_le_bytes());
        self.tokens.flush()?;
        self.tokens.seek(SeekFrom::Start(0))?;
        self.tokens.write_all(&header)?;
        self.tokens.flush()?;
        self.mask.flush()?;
        self.segment.flush()?;
        Ok((self.tokens, self.mask, self.segment, self.stats))
    }
}

/// Open one shard (`.jsonl` or `.jsonl.zst`), hashing its raw bytes, and feed each line to
/// `on_line`. Returns the file's sha256 hex.
pub fn read_shard(
    path: &std::path::Path,
    on_line: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hashed = HashRead::new(file);
    {
        let reader: Box<dyn BufRead + '_> = if path.extension().is_some_and(|e| e == "zst") {
            let dec = zstd::stream::read::Decoder::new(&mut hashed)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            Box::new(BufReader::new(dec))
        } else {
            Box::new(BufReader::new(&mut hashed))
        };
        for line in reader.lines() {
            let line = line.map_err(|e| format!("{}: {e}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            on_line(&line)?;
        }
    }
    // Hash any trailing bytes the decoder left unread (for example a zstd skippable frame).
    io::copy(&mut hashed, &mut io::sink()).map_err(|e| e.to_string())?;
    Ok(hex(&hashed.hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// GPT-2's byte-to-character table, as the tokenizer uses it.
    fn byte_char(b: u8) -> char {
        let printable = |x: u8| (33..=126).contains(&x) || (161..=172).contains(&x) || x >= 174;
        if printable(b) {
            return b as char;
        }
        let n = (0u8..=255).filter(|&x| !printable(x) && x < b).count() as u32;
        char::from_u32(256 + n).unwrap()
    }

    /// A tiny byte-level tokenizer: the three role tokens, every byte except `missing`,
    /// and one merge (`a` + `b`).
    fn tokenizer_json(missing: &[u8]) -> Vec<u8> {
        let mut vocab = serde_json::Map::new();
        for (i, name) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert(name.to_string(), serde_json::json!(i));
        }
        for b in 0u16..256 {
            let b = b as u8;
            if !missing.contains(&b) {
                let id = vocab.len();
                vocab.insert(byte_char(b).to_string(), serde_json::json!(id));
            }
        }
        let ab = vocab.len();
        vocab.insert("ab".into(), serde_json::json!(ab));
        let added: Vec<serde_json::Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
            .iter()
            .enumerate()
            .map(|(i, c)| serde_json::json!({"id": i, "content": c, "special": true}))
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "added_tokens": added,
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true, "use_regex": true},
            "model": {"type": "BPE", "vocab": vocab, "merges": ["a b"]}
        }))
        .unwrap()
    }

    fn coder(missing: &[u8]) -> Coder {
        Coder::new(&tokenizer_json(missing)).unwrap()
    }

    fn run_ingest(coder: &Coder, lines: &[&str]) -> (Vec<u8>, Vec<u8>, Vec<u8>, Stats) {
        let mut ing = Ingest::new(Cursor::new(vec![]), vec![], vec![]).unwrap();
        for l in lines {
            ing.add(coder, &parse_record(l).unwrap()).unwrap();
        }
        let (tokens, mask, segment, stats) = ing.finish().unwrap();
        (tokens.into_inner(), mask, segment, stats)
    }

    fn ids_of(tokens: &[u8]) -> Vec<u32> {
        tokens[UORT_HEADER..]
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]) as u32)
            .collect()
    }

    const D41: &str =
        r#"{"id":"p1","kind":"math","messages":[{"role":"user","content":"ab?"}],"answers":["4"]}"#;
    const D87: &str = r#"{"id":"q1","kind":"two-turn","source":"s","messages":[{"role":"user","content":"hi"},{"role":"user","content":"ok"}],"turns":[{"reasoning":"think","answer":"yo"},{"reasoning":"","answer":"done"}]}"#;

    #[test]
    fn d41_record_frames_and_masks() {
        let c = coder(&[]);
        let (tokens, mask, segment, ing) = run_ingest(&c, &[D41]);
        assert_eq!(&tokens[0..4], b"UORT");
        assert_eq!(u32::from_le_bytes(tokens[4..8].try_into().unwrap()), 1);
        let ids = ids_of(&tokens);
        assert_eq!(
            u64::from_le_bytes(tokens[8..16].try_into().unwrap()),
            ids.len() as u64
        );
        assert_eq!(
            u32::from_le_bytes(tokens[16..20].try_into().unwrap()),
            VOCAB
        );
        assert_eq!(ids[0], BOS);
        assert_eq!(*ids.last().unwrap(), EOS);
        assert!(ids.iter().all(|&i| i < VOCAB));
        assert_eq!(mask.len(), ids.len());
        assert_eq!(segment.len(), ids.len());
        // The decoded text of the stream, minus BOS/EOS, is the framed record.
        let body = c.tok.decode_bytes(&ids[1..ids.len() - 1]);
        assert_eq!(body, b"\nUser: ab?\nAssistant: 4");
        // Mask is 1 exactly on the answer tokens and the closing EOS.
        let answer_tokens = segment
            .iter()
            .filter(|&&s| s == Segment::Answer as u8)
            .count();
        assert_eq!(mask.iter().filter(|&&m| m == 1).count(), answer_tokens + 1);
        assert_eq!(*mask.last().unwrap(), 1);
        assert_eq!(mask[0], 0);
        // "ab" merges to one token.
        let ab = c.tok.token_id("ab").unwrap();
        assert!(ids.contains(&ab));
        assert_eq!(ing.records.get("d41/math"), Some(&1));
        let prompt = &ing.per_segment[&Segment::Prompt];
        assert_eq!(prompt.text_bytes, " ab?".len() as u64);
        assert_eq!(prompt.dropped_bytes, 0);
    }

    #[test]
    fn d87_two_turn_segments_in_order() {
        let c = coder(&[]);
        let (tokens, mask, segment, ing) = run_ingest(&c, &[D87]);
        let ids = ids_of(&tokens);
        let body = c.tok.decode_bytes(&ids[1..ids.len() - 1]);
        assert_eq!(
            body,
            b"\nUser: hi\nAssistant: think\n\nyo\nUser: ok\nAssistant: done".to_vec()
        );
        // Prompt tokens are never targets; reasoning and answer tokens always are.
        for (s, m) in segment.iter().zip(&mask) {
            if *s == Segment::Prompt as u8 {
                assert_eq!(*m, 0);
            }
            if *s == Segment::Reasoning as u8 || *s == Segment::Answer as u8 {
                assert_eq!(*m, 1);
            }
        }
        // The separator between reasoning and answer is a frame token the model must emit.
        let sep = c.tok.encode(SEPARATOR);
        let at = ids.windows(sep.len()).position(|w| w == &sep[..]).unwrap();
        assert!(segment[at..at + sep.len()]
            .iter()
            .all(|&s| s == Segment::Frame as u8));
        assert!(mask[at..at + sep.len()].iter().all(|&m| m == 1));
        assert!(segment.contains(&(Segment::Reasoning as u8)));
        assert_eq!(ing.records.get("d87/two-turn"), Some(&1));
        assert_eq!(
            ing.per_segment[&Segment::Reasoning].text_bytes,
            " think".len() as u64
        );
    }

    #[test]
    fn dropped_and_fallback_bytes_are_counted() {
        // U+4E2D is e4 b8 ad: drop e4 from the vocabulary.
        let c = coder(&[0xe4]);
        let line = r#"{"id":"q2","kind":"single","source":"s","messages":[{"role":"user","content":"中"}],"turns":[{"reasoning":"","answer":"x"}]}"#;
        let (_, _, _, ing) = run_ingest(&c, &[line]);
        let p = &ing.per_segment[&Segment::Prompt];
        assert_eq!(p.text_bytes, 4); // " " + three bytes
        assert_eq!(p.dropped_bytes, 1);
        assert_eq!(p.fallback_tokens, 2);
        assert_eq!(p.tokens, 3);
        assert_eq!(ing.top_runs(20), vec![("e4b8ad".to_string(), 1)]);
    }

    #[test]
    fn literal_special_token_in_text_is_refused() {
        let c = coder(&[]);
        let mut count = SegmentCount::default();
        c.encode_piece(" a<|eos|>b", &mut count, &mut |_| {})
            .unwrap();
        assert_eq!(count.special_in_text, 1);
        let line = r#"{"id":"q3","kind":"single","source":"s","messages":[{"role":"user","content":"x"}],"turns":[{"reasoning":"","answer":"a<|eos|>b"}]}"#;
        let mut ing = Ingest::new(Cursor::new(vec![]), vec![], vec![]).unwrap();
        let e = ing.add(&c, &parse_record(line).unwrap()).unwrap_err();
        assert!(e.contains("q3") && e.contains("answer"), "{e}");
    }

    #[test]
    fn kind_must_match_turn_count() {
        let one_as_two = r#"{"id":"k1","kind":"two-turn","source":"s","messages":[{"role":"user","content":"x"}],"turns":[{"reasoning":"","answer":"y"}]}"#;
        assert!(parse_record(one_as_two).unwrap_err().contains("k1"));
        let two_as_one = r#"{"id":"k2","kind":"single","source":"s","messages":[{"role":"user","content":"x"},{"role":"user","content":"y"}],"turns":[{"reasoning":"","answer":"a"},{"reasoning":"","answer":"b"}]}"#;
        assert!(parse_record(two_as_one).unwrap_err().contains("k2"));
    }

    #[test]
    fn unknown_key_and_bad_turns_are_refused() {
        let extra = r#"{"id":"bad1","kind":"single","source":"s","messages":[{"role":"user","content":"x"}],"turns":[{"reasoning":"","answer":"y"}],"logprob":1}"#;
        let e = parse_record(extra).unwrap_err();
        assert!(e.contains("bad1") && e.contains("unknown key set"), "{e}");
        let short = r#"{"id":"bad2","kind":"two-turn","source":"s","messages":[{"role":"user","content":"x"},{"role":"user","content":"y"}],"turns":[{"reasoning":"","answer":"z"}]}"#;
        assert!(parse_record(short).unwrap_err().contains("bad2"));
        let turn_key = r#"{"id":"bad3","kind":"single","source":"s","messages":[{"role":"user","content":"x"}],"turns":[{"reasoning":"","answer":"z","score":2}]}"#;
        assert!(parse_record(turn_key).unwrap_err().contains("turn keys"));
    }

    #[test]
    fn raw_and_zstd_shards_read_the_same_records() {
        let dir = std::env::temp_dir().join(format!("student-ingest-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let text = format!("{D41}\n{D87}\n");
        let raw = dir.join("shard.jsonl");
        let zst = dir.join("shard.jsonl.zst");
        std::fs::write(&raw, &text).unwrap();
        std::fs::write(&zst, zstd::encode_all(text.as_bytes(), 3).unwrap()).unwrap();
        let read = |p: &std::path::Path| {
            let mut ids = vec![];
            let sha = read_shard(p, &mut |l| {
                ids.push(parse_record(l)?.id);
                Ok(())
            })
            .unwrap();
            (ids, sha)
        };
        let (raw_ids, raw_sha) = read(&raw);
        let (zst_ids, zst_sha) = read(&zst);
        assert_eq!(raw_ids, vec!["p1", "q1"]);
        assert_eq!(raw_ids, zst_ids);
        // Each sha is of the file's own bytes.
        assert_eq!(raw_sha, hex(&Sha256::digest(text.as_bytes())));
        assert_eq!(zst_sha, hex(&Sha256::digest(std::fs::read(&zst).unwrap())));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn tokenizer_with_wrong_bos_is_refused() {
        let mut v: serde_json::Value = serde_json::from_slice(&tokenizer_json(&[])).unwrap();
        v["added_tokens"][0]["content"] = serde_json::json!("<|start|>");
        let vocab = v["model"]["vocab"].as_object_mut().unwrap();
        vocab.remove("<|bos|>");
        vocab.insert("<|start|>".into(), serde_json::json!(0));
        assert!(Coder::new(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
