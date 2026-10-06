//! Phase 0 clause-retrieval probe (#820): can an offline MiniLM teacher,
//! turned into static (and E8-quantized) token tables, pick the right clause
//! of a conversation log for a memory question?
//!
//! Pre-registration: `docs/research/minilm-phase0/PREREG.md`. Everything the
//! measurement depends on is fixed here: the clause segmentation rule, the
//! stopword list, the value-matching rule, the lexical baseline, the BERT
//! (MiniLM-L6) forward used as an offline teacher, the static-table
//! distillation, the E8 quantization and the quaternionic Hopf arm.
//!
//! Scope: MiniLM (`sentence-transformers/all-MiniLM-L6-v2`, Apache-2.0) is an
//! offline teacher and comparator only. Nothing here serves a reply, and no
//! served path loads this module. The BERT forward is floating point offline
//! computation; only the E arm's scoring is integer.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

use candle_core::{DType, Device, Tensor};

use crate::b3_e8_codecs::e8_nearest;

pub type Result<T> = std::result::Result<T, String>;

fn ce(e: candle_core::Error) -> String {
    e.to_string()
}

/// MiniLM-L6 hidden width.
pub const HIDDEN: usize = 384;
/// E8 blocks per 384-d vector.
pub const BLOCKS: usize = HIDDEN / 8;
/// sentence-transformers `max_seq_length` of all-MiniLM-L6-v2.
pub const MAX_SEQ: usize = 256;
const HEADS: usize = 12;
const HEAD_DIM: usize = HIDDEN / HEADS;
const LAYERS: usize = 6;
const LN_EPS: f32 = 1e-12;
/// E arm scale constant: codes are `e8_nearest(K / rms * v)` (see PREREG).
pub const E8_SCALE_K: f32 = 2.0;

/// Fixed stopword list (the NLTK English list plus contraction pieces).
/// Used by the lexical arm, by the content-token selection of the static
/// arms and by value matching. Question words (what, which, who, where,
/// when, how) are stopwords.
pub const STOPWORDS: &[&str] = &[
    "i",
    "me",
    "my",
    "myself",
    "we",
    "our",
    "ours",
    "ourselves",
    "you",
    "your",
    "yours",
    "yourself",
    "yourselves",
    "he",
    "him",
    "his",
    "himself",
    "she",
    "her",
    "hers",
    "herself",
    "it",
    "its",
    "itself",
    "they",
    "them",
    "their",
    "theirs",
    "themselves",
    "what",
    "which",
    "who",
    "whom",
    "this",
    "that",
    "these",
    "those",
    "am",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "have",
    "has",
    "had",
    "having",
    "do",
    "does",
    "did",
    "doing",
    "a",
    "an",
    "the",
    "and",
    "but",
    "if",
    "or",
    "because",
    "as",
    "until",
    "while",
    "of",
    "at",
    "by",
    "for",
    "with",
    "about",
    "against",
    "between",
    "into",
    "through",
    "during",
    "before",
    "after",
    "above",
    "below",
    "to",
    "from",
    "up",
    "down",
    "in",
    "out",
    "on",
    "off",
    "over",
    "under",
    "again",
    "further",
    "then",
    "once",
    "here",
    "there",
    "when",
    "where",
    "why",
    "how",
    "all",
    "any",
    "both",
    "each",
    "few",
    "more",
    "most",
    "other",
    "some",
    "such",
    "no",
    "nor",
    "not",
    "only",
    "own",
    "same",
    "so",
    "than",
    "too",
    "very",
    "s",
    "t",
    "can",
    "will",
    "just",
    "don",
    "should",
    "now",
    "d",
    "ll",
    "m",
    "o",
    "re",
    "ve",
    "y",
    "ain",
    "aren",
    "couldn",
    "didn",
    "doesn",
    "hadn",
    "hasn",
    "haven",
    "isn",
    "ma",
    "mightn",
    "mustn",
    "needn",
    "shan",
    "shouldn",
    "wasn",
    "weren",
    "won",
    "wouldn",
    "it's",
    "i'm",
    "don't",
    "didn't",
    "doesn't",
    "isn't",
    "wasn't",
    "you're",
    "we're",
    "they're",
    "i've",
    "we've",
    "i'd",
    "you'd",
    "i'll",
    "we'll",
    "let's",
    "that's",
    "what's",
    "there's",
    "can't",
];

pub fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

// ---------------------------------------------------------------- words

/// Lowercased words: maximal runs of ASCII-alphanumeric characters and
/// apostrophes (’ is read as '), with leading/trailing apostrophes removed and
/// a trailing possessive `'s` stripped. Used by the lexical arm and by value
/// matching (not by the MiniLM tokenizer).
pub fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<String>| {
        let mut w = cur.trim_matches('\'').to_string();
        if let Some(stripped) = w.strip_suffix("'s") {
            w = stripped.to_string();
        }
        let w = w.trim_matches('\'').to_string();
        if !w.is_empty() {
            out.push(w);
        }
        cur.clear();
    };
    for ch in text.chars() {
        let ch = if ch == '\u{2019}' { '\'' } else { ch };
        if ch.is_ascii_alphanumeric() || ch == '\'' {
            cur.push(ch.to_ascii_lowercase());
        } else if ch.is_alphanumeric() {
            for l in ch.to_lowercase() {
                cur.push(l);
            }
        } else {
            flush(&mut cur, &mut out);
        }
    }
    flush(&mut cur, &mut out);
    out
}

/// Distinct non-stopword words.
pub fn content_words(text: &str) -> BTreeSet<String> {
    words(text)
        .into_iter()
        .filter(|w| !is_stopword(w))
        .collect()
}

// ---------------------------------------------------------------- segmentation

/// Sentence split: a sentence ends after a run of `.`, `!` or `?` (plus any
/// closing quotes/brackets) that is followed by whitespace or the end of the
/// text, and at every `;`.
pub fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == ';' {
            out.push(chars[start..i].iter().collect::<String>());
            start = i + 1;
            i += 1;
            continue;
        }
        if matches!(c, '.' | '!' | '?') {
            let mut j = i + 1;
            while j < chars.len() && matches!(chars[j], '.' | '!' | '?') {
                j += 1;
            }
            while j < chars.len() && matches!(chars[j], '"' | '\'' | ')' | '\u{201d}' | '\u{2019}')
            {
                j += 1;
            }
            if j == chars.len() || chars[j].is_whitespace() {
                out.push(chars[start..j].iter().collect::<String>());
                start = j;
                i = j;
                continue;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    if start < chars.len() {
        out.push(chars[start..].iter().collect::<String>());
    }
    out.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| s.chars().any(char::is_alphanumeric))
        .collect()
}

/// Coordinating separators (case-insensitive, matched with the surrounding
/// spaces exactly as written).
pub const CLAUSE_SEPARATORS: [&str; 3] = [" and ", " but ", " / "];

/// Clause split of one sentence at every occurrence of a separator.
fn split_coordinated(sentence: &str) -> Vec<String> {
    // ASCII lowercasing keeps every byte offset; the separators are ASCII,
    // so a match always starts and ends on a char boundary.
    let lower = sentence.to_ascii_lowercase();
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let mut matched = None;
        for sep in CLAUSE_SEPARATORS {
            if bytes[i..].starts_with(sep.as_bytes()) {
                matched = Some(sep.len());
                break;
            }
        }
        if let Some(len) = matched {
            cuts.push((i, i + len));
            // The separator's trailing space may start the next separator.
            i += len - 1;
        } else {
            i += 1;
        }
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    for (a, b) in cuts {
        if a >= start {
            out.push(sentence[start..a].to_string());
            start = b;
        }
    }
    out.push(sentence[start..].to_string());
    out.into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| s.chars().any(char::is_alphanumeric))
        .collect()
}

/// The frozen clause segmentation of one message.
pub fn split_clauses(text: &str) -> Vec<String> {
    split_sentences(text)
        .iter()
        .flat_map(|s| split_coordinated(s))
        .collect()
}

/// Segment a conversation log (messages in order) into clauses in log order.
pub fn segment_history<S: AsRef<str>>(messages: &[S]) -> Vec<String> {
    messages
        .iter()
        .flat_map(|m| split_clauses(m.as_ref()))
        .collect()
}

// ---------------------------------------------------------------- values

/// A value with alternative spellings. An alternative is present in a clause
/// when every one of its non-stopword words (all words if it has none) is a
/// word of the clause.
#[derive(Clone, Debug, Default)]
pub struct ValueSet {
    pub alternatives: Vec<Vec<String>>,
}

impl ValueSet {
    pub fn from_spellings<S: AsRef<str>>(spellings: &[S]) -> Self {
        let mut alternatives = Vec::new();
        for s in spellings {
            for alt in s.as_ref().split('|') {
                let alt = alt.trim();
                if alt.is_empty() || alt == "-" {
                    continue;
                }
                let all = words(alt);
                let content: Vec<String> =
                    all.iter().filter(|w| !is_stopword(w)).cloned().collect();
                let req = if content.is_empty() { all } else { content };
                if !req.is_empty() {
                    alternatives.push(req);
                }
            }
        }
        Self { alternatives }
    }

    pub fn is_empty(&self) -> bool {
        self.alternatives.is_empty()
    }

    pub fn present_in(&self, clause_words: &HashSet<String>) -> bool {
        self.alternatives
            .iter()
            .any(|alt| alt.iter().all(|w| clause_words.contains(w)))
    }
}

/// Clause hit: the clause names an expected spelling and no forbidden one.
pub fn clause_hit(clause: &str, expected: &ValueSet, forbidden: &ValueSet) -> bool {
    let ws: HashSet<String> = words(clause).into_iter().collect();
    expected.present_in(&ws) && !forbidden.present_in(&ws)
}

/// Clause holds both an expected and a forbidden value (segmentation limit).
pub fn clause_mixed(clause: &str, expected: &ValueSet, forbidden: &ValueSet) -> bool {
    let ws: HashSet<String> = words(clause).into_iter().collect();
    expected.present_in(&ws) && forbidden.present_in(&ws)
}

// ---------------------------------------------------------------- ranking

/// Index of the maximum under `cmp`; ties go to the most recent (largest)
/// index. `None` for an empty list.
pub fn argmax_recent_by<F>(n: usize, mut cmp: F) -> Option<usize>
where
    F: FnMut(usize, usize) -> std::cmp::Ordering,
{
    let mut best: Option<usize> = None;
    for i in 0..n {
        best = match best {
            None => Some(i),
            Some(b) => {
                if cmp(i, b) != std::cmp::Ordering::Less {
                    Some(i)
                } else {
                    Some(b)
                }
            }
        };
    }
    best
}

pub fn argmax_recent_f64(scores: &[f64]) -> Option<usize> {
    argmax_recent_by(scores.len(), |a, b| {
        scores[a]
            .partial_cmp(&scores[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Lexical arm: number of distinct shared non-stopword words.
pub fn lexical_score(question: &str, clause: &str) -> usize {
    let q = content_words(question);
    let c = content_words(clause);
    q.intersection(&c).count()
}

/// Lexical arm's choice (ties, including all-zero, to the most recent clause).
pub fn lexical_choice<S: AsRef<str>>(question: &str, clauses: &[S]) -> Option<usize> {
    let scores: Vec<usize> = clauses
        .iter()
        .map(|c| lexical_score(question, c.as_ref()))
        .collect();
    argmax_recent_by(scores.len(), |a, b| scores[a].cmp(&scores[b]))
}

// ---------------------------------------------------------------- WordPiece

/// The BERT uncased tokenizer of all-MiniLM-L6-v2 (`tokenizer.json`):
/// BertNormalizer (clean text, lowercase, accent strip, CJK spacing),
/// BertPreTokenizer (whitespace and punctuation split) and greedy
/// longest-match WordPiece with `##` continuations.
pub struct WordPiece {
    vocab: HashMap<String, u32>,
    unk: u32,
    pub cls: u32,
    pub sep: u32,
    pub vocab_size: usize,
}

fn is_bert_punct(c: char) -> bool {
    let u = c as u32;
    (33..=47).contains(&u)
        || (58..=64).contains(&u)
        || (91..=96).contains(&u)
        || (123..=126).contains(&u)
        || matches!(
            c,
            '\u{2018}'
                | '\u{2019}'
                | '\u{201c}'
                | '\u{201d}'
                | '\u{2013}'
                | '\u{2014}'
                | '\u{2026}'
                | '\u{00a1}'
                | '\u{00bf}'
                | '\u{00ab}'
                | '\u{00bb}'
        )
}

fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x20000..=0x2A6DF).contains(&u)
        || (0xF900..=0xFAFF).contains(&u)
        || (0x2F800..=0x2FA1F).contains(&u)
}

/// Accent strip for precomposed Latin-1/Latin Extended-A letters (the probe
/// data is ASCII apart from a few punctuation marks; no Unicode
/// normalization crate is added for this).
fn strip_accent(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'š' | 'ś' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        other => other,
    }
}

impl WordPiece {
    pub fn from_tokenizer_json(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let model = &json["model"];
        if model["type"] != "WordPiece" || model["continuing_subword_prefix"] != "##" {
            return Err("tokenizer.json is not a ## WordPiece model".into());
        }
        let norm = &json["normalizer"];
        if norm["type"] != "BertNormalizer" || norm["lowercase"] != true {
            return Err("tokenizer.json normalizer is not a lowercasing BertNormalizer".into());
        }
        let vocab_json = model["vocab"]
            .as_object()
            .ok_or("tokenizer.json has no WordPiece vocab")?;
        let mut vocab = HashMap::with_capacity(vocab_json.len());
        for (k, v) in vocab_json {
            let id = v.as_u64().ok_or("vocab id is not an integer")?;
            vocab.insert(k.clone(), u32::try_from(id).map_err(|e| e.to_string())?);
        }
        let get = |t: &str| vocab.get(t).copied().ok_or(format!("vocab lacks {t}"));
        let unk = get("[UNK]")?;
        let cls = get("[CLS]")?;
        let sep = get("[SEP]")?;
        let vocab_size = vocab.len();
        Ok(Self {
            vocab,
            unk,
            cls,
            sep,
            vocab_size,
        })
    }

    /// BertNormalizer + BertPreTokenizer.
    pub fn basic_tokens(&self, text: &str) -> Vec<String> {
        let mut spaced = String::with_capacity(text.len() + 8);
        for c in text.chars() {
            let u = c as u32;
            if u == 0 || u == 0xFFFD || (c.is_control() && !c.is_whitespace()) {
                continue;
            }
            if c.is_whitespace() {
                spaced.push(' ');
                continue;
            }
            for l in c.to_lowercase() {
                let l = strip_accent(l);
                if is_cjk(l) || is_bert_punct(l) {
                    spaced.push(' ');
                    spaced.push(l);
                    spaced.push(' ');
                } else if ('\u{0300}'..='\u{036f}').contains(&l) {
                    // combining mark: dropped by accent stripping
                } else {
                    spaced.push(l);
                }
            }
        }
        spaced.split_whitespace().map(str::to_string).collect()
    }

    /// Greedy longest-match WordPiece of one basic token.
    pub fn wordpiece(&self, word: &str) -> Vec<u32> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() > 100 {
            return vec![self.unk];
        }
        let mut out = Vec::new();
        let mut start = 0usize;
        while start < chars.len() {
            let mut end = chars.len();
            let mut found = None;
            while start < end {
                let mut piece: String = chars[start..end].iter().collect();
                if start > 0 {
                    piece = format!("##{piece}");
                }
                if let Some(&id) = self.vocab.get(&piece) {
                    found = Some(id);
                    break;
                }
                end -= 1;
            }
            match found {
                Some(id) => {
                    out.push(id);
                    start = end;
                }
                None => return vec![self.unk],
            }
        }
        out
    }

    /// Token ids of a text without specials.
    pub fn token_ids(&self, text: &str) -> Vec<u32> {
        self.basic_tokens(text)
            .iter()
            .flat_map(|w| self.wordpiece(w))
            .collect()
    }

    /// `[CLS] ... [SEP]`, truncated to `MAX_SEQ`.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut ids = self.token_ids(text);
        ids.truncate(MAX_SEQ - 2);
        let mut out = Vec::with_capacity(ids.len() + 2);
        out.push(self.cls);
        out.extend(ids);
        out.push(self.sep);
        out
    }

    /// The static arms' token set: WordPiece ids of the non-stopword,
    /// non-punctuation basic tokens; if there are none, of all
    /// non-punctuation basic tokens; if still none, of all basic tokens.
    pub fn content_token_ids(&self, text: &str) -> Vec<u32> {
        let basic = self.basic_tokens(text);
        let non_punct: Vec<&String> = basic
            .iter()
            .filter(|w| !(w.chars().count() == 1 && w.chars().all(is_bert_punct)))
            .collect();
        let content: Vec<&String> = non_punct
            .iter()
            .copied()
            .filter(|w| !is_stopword(w))
            .collect();
        let chosen: Vec<&String> = if !content.is_empty() {
            content
        } else if !non_punct.is_empty() {
            non_punct
        } else {
            basic.iter().collect()
        };
        chosen.iter().flat_map(|w| self.wordpiece(w)).collect()
    }
}

// ---------------------------------------------------------------- MiniLM-L6 forward

struct Layer {
    q_w: Tensor,
    q_b: Tensor,
    k_w: Tensor,
    k_b: Tensor,
    v_w: Tensor,
    v_b: Tensor,
    ao_w: Tensor,
    ao_b: Tensor,
    ln1_w: Tensor,
    ln1_b: Tensor,
    i_w: Tensor,
    i_b: Tensor,
    o_w: Tensor,
    o_b: Tensor,
    ln2_w: Tensor,
    ln2_b: Tensor,
}

/// The 6-layer, 384-wide BERT encoder of all-MiniLM-L6-v2, written with
/// candle-core tensor ops (post-LN BERT, erf GELU, absolute positions, token
/// type 0). Offline teacher only.
pub struct MiniLm {
    word: Tensor,
    pos: Tensor,
    typ: Tensor,
    emb_ln_w: Tensor,
    emb_ln_b: Tensor,
    layers: Vec<Layer>,
    device: Device,
}

fn linear(x: &Tensor, w: &Tensor, b: &Tensor) -> Result<Tensor> {
    // Flatten leading dimensions into one 2-D matmul (a broadcast batched
    // matmul of many 3-row products is far slower on the CPU backend).
    let dims = x.dims().to_vec();
    let (rows, inner) = match dims.as_slice() {
        [r, i] => (*r, *i),
        [a, t, i] => (a * t, *i),
        _ => return Err(format!("linear: unsupported shape {dims:?}")),
    };
    let out = w.dims()[0];
    let y = x
        .reshape((rows, inner))
        .map_err(ce)?
        .matmul(&w.t().map_err(ce)?)
        .map_err(ce)?
        .broadcast_add(b)
        .map_err(ce)?;
    let mut shape = dims;
    if let Some(last) = shape.last_mut() {
        *last = out;
    }
    y.reshape(shape).map_err(ce)
}

fn layer_norm(x: &Tensor, w: &Tensor, b: &Tensor) -> Result<Tensor> {
    candle_nn::ops::layer_norm(x, w, b, LN_EPS).map_err(ce)
}

impl MiniLm {
    pub fn load(safetensors: &Path) -> Result<Self> {
        let device = Device::Cpu;
        let mut map = candle_core::safetensors::load(safetensors, &device).map_err(ce)?;
        let mut take = |name: &str| -> Result<Tensor> {
            let t = map
                .remove(name)
                .or_else(|| map.remove(&format!("bert.{name}")))
                .ok_or(format!("missing tensor {name}"))?;
            t.to_dtype(DType::F32).map_err(ce)
        };
        let word = take("embeddings.word_embeddings.weight")?;
        let pos = take("embeddings.position_embeddings.weight")?;
        let typ = take("embeddings.token_type_embeddings.weight")?;
        let emb_ln_w = take("embeddings.LayerNorm.weight")?;
        let emb_ln_b = take("embeddings.LayerNorm.bias")?;
        let mut layers = Vec::with_capacity(LAYERS);
        for l in 0..LAYERS {
            let p = format!("encoder.layer.{l}");
            layers.push(Layer {
                q_w: take(&format!("{p}.attention.self.query.weight"))?,
                q_b: take(&format!("{p}.attention.self.query.bias"))?,
                k_w: take(&format!("{p}.attention.self.key.weight"))?,
                k_b: take(&format!("{p}.attention.self.key.bias"))?,
                v_w: take(&format!("{p}.attention.self.value.weight"))?,
                v_b: take(&format!("{p}.attention.self.value.bias"))?,
                ao_w: take(&format!("{p}.attention.output.dense.weight"))?,
                ao_b: take(&format!("{p}.attention.output.dense.bias"))?,
                ln1_w: take(&format!("{p}.attention.output.LayerNorm.weight"))?,
                ln1_b: take(&format!("{p}.attention.output.LayerNorm.bias"))?,
                i_w: take(&format!("{p}.intermediate.dense.weight"))?,
                i_b: take(&format!("{p}.intermediate.dense.bias"))?,
                o_w: take(&format!("{p}.output.dense.weight"))?,
                o_b: take(&format!("{p}.output.dense.bias"))?,
                ln2_w: take(&format!("{p}.output.LayerNorm.weight"))?,
                ln2_b: take(&format!("{p}.output.LayerNorm.bias"))?,
            });
        }
        if word.dims() != [30522, HIDDEN] {
            return Err(format!("unexpected word embedding shape {:?}", word.dims()));
        }
        Ok(Self {
            word,
            pos,
            typ,
            emb_ln_w,
            emb_ln_b,
            layers,
            device,
        })
    }

    /// Last hidden states `[batch, seq, 384]` of equal-length, unpadded
    /// sequences (no attention mask is needed).
    pub fn forward(&self, batch: &[Vec<u32>]) -> Result<Tensor> {
        let b = batch.len();
        let t = batch.first().map(Vec::len).ok_or("empty batch")?;
        if t == 0 || t > 512 || batch.iter().any(|s| s.len() != t) {
            return Err("forward needs equal, non-empty sequence lengths <= 512".into());
        }
        let flat: Vec<u32> = batch.iter().flatten().copied().collect();
        let ids = Tensor::from_vec(flat, (b * t,), &self.device).map_err(ce)?;
        let we = self
            .word
            .index_select(&ids, 0)
            .map_err(ce)?
            .reshape((b, t, HIDDEN))
            .map_err(ce)?;
        let pe = self.pos.narrow(0, 0, t).map_err(ce)?;
        let te = self.typ.narrow(0, 0, 1).map_err(ce)?;
        let x = we
            .broadcast_add(&pe)
            .map_err(ce)?
            .broadcast_add(&te)
            .map_err(ce)?;
        let mut x = layer_norm(&x, &self.emb_ln_w, &self.emb_ln_b)?;
        let scale = 1.0 / (HEAD_DIM as f64).sqrt();
        for l in &self.layers {
            let heads = |w: &Tensor, bias: &Tensor| -> Result<Tensor> {
                linear(&x, w, bias)?
                    .reshape((b, t, HEADS, HEAD_DIM))
                    .map_err(ce)?
                    .transpose(1, 2)
                    .map_err(ce)?
                    .contiguous()
                    .map_err(ce)
            };
            let q = heads(&l.q_w, &l.q_b)?;
            let k = heads(&l.k_w, &l.k_b)?;
            let v = heads(&l.v_w, &l.v_b)?;
            let kt = k.t().map_err(ce)?.contiguous().map_err(ce)?;
            let scores = (q.matmul(&kt).map_err(ce)? * scale).map_err(ce)?;
            let probs = candle_nn::ops::softmax_last_dim(&scores).map_err(ce)?;
            let ctx = probs
                .matmul(&v)
                .map_err(ce)?
                .transpose(1, 2)
                .map_err(ce)?
                .contiguous()
                .map_err(ce)?
                .reshape((b, t, HIDDEN))
                .map_err(ce)?;
            let attn = linear(&ctx, &l.ao_w, &l.ao_b)?;
            let h = layer_norm(&(attn + &x).map_err(ce)?, &l.ln1_w, &l.ln1_b)?;
            let inter = linear(&h, &l.i_w, &l.i_b)?.gelu_erf().map_err(ce)?;
            let out = linear(&inter, &l.o_w, &l.o_b)?;
            x = layer_norm(&(out + &h).map_err(ce)?, &l.ln2_w, &l.ln2_b)?;
        }
        Ok(x)
    }

    /// Mean over all positions (sentence-transformers pooling with an
    /// all-ones mask), one row per sequence.
    pub fn mean_pooled(&self, batch: &[Vec<u32>]) -> Result<Vec<Vec<f32>>> {
        let h = self.forward(batch)?;
        h.mean(1).map_err(ce)?.to_vec2::<f32>().map_err(ce)
    }

    /// The sentence-transformers embedding: tokenize, mean-pool, L2-normalize.
    pub fn sentence_embedding(&self, tok: &WordPiece, text: &str) -> Result<Vec<f32>> {
        let ids = tok.encode(text);
        let mut v = self
            .mean_pooled(&[ids])?
            .into_iter()
            .next()
            .ok_or("no pooled row")?;
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        v.iter_mut().for_each(|x| *x /= n);
        Ok(v)
    }

    /// Static distillation (Model2Vec-style, no PCA or Zipf weighting): each
    /// vocabulary id alone as `[CLS] id [SEP]`, mean-pooled over the three
    /// positions, not normalized. Row-major `vocab x 384`.
    pub fn static_table(&self, tok: &WordPiece, batch_size: usize) -> Result<Vec<f32>> {
        let vocab = tok.vocab_size;
        let mut table = Vec::with_capacity(vocab * HIDDEN);
        let mut start = 0usize;
        while start < vocab {
            let end = (start + batch_size.max(1)).min(vocab);
            let batch: Vec<Vec<u32>> = (start..end)
                .map(|id| vec![tok.cls, id as u32, tok.sep])
                .collect();
            for row in self.mean_pooled(&batch)? {
                table.extend(row);
            }
            start = end;
        }
        Ok(table)
    }
}

pub fn cosine(a: &[f32], b: &[f32]) -> f64 {
    let (mut d, mut na, mut nb) = (0f64, 0f64, 0f64);
    for (x, y) in a.iter().zip(b) {
        d += *x as f64 * *y as f64;
        na += *x as f64 * *x as f64;
        nb += *y as f64 * *y as f64;
    }
    if na == 0.0 || nb == 0.0 {
        return f64::NEG_INFINITY;
    }
    d / (na.sqrt() * nb.sqrt())
}

/// Sum of table rows for `ids`.
pub fn sum_rows(table: &[f32], ids: &[u32]) -> Vec<f32> {
    let mut v = vec![0f32; HIDDEN];
    for &id in ids {
        let row = &table[id as usize * HIDDEN..(id as usize + 1) * HIDDEN];
        for (a, b) in v.iter_mut().zip(row) {
            *a += *b;
        }
    }
    v
}

// ---------------------------------------------------------------- E8 quantization

/// Quantize one 384-d vector into 48 E8 points of `scale * v` (nearest point
/// of E8 = D8 ∪ (D8 + ½) per 8-block, `b3_e8_codecs::e8_nearest`), stored as
/// doubled integer coordinates.
pub fn e8_quantize(v: &[f32], scale: f32) -> Result<Vec<i8>> {
    if v.len() != HIDDEN {
        return Err("e8_quantize needs a 384-d vector".into());
    }
    let mut out = Vec::with_capacity(HIDDEN);
    for b in 0..BLOCKS {
        let mut y = [0f32; 8];
        for i in 0..8 {
            y[i] = v[b * 8 + i] * scale;
        }
        let p = e8_nearest(&y);
        for x in p {
            let d = (2.0 * x).round();
            if !(-127.0..=127.0).contains(&d) {
                return Err(format!("E8 code coordinate {d} does not fit i8"));
            }
            out.push(d as i8);
        }
    }
    Ok(out)
}

/// Dequantize doubled E8 codes back to the original scale.
pub fn e8_dequantize(codes: &[i8], scale: f32) -> Vec<f32> {
    codes.iter().map(|&c| c as f32 / (2.0 * scale)).collect()
}

/// Sum of integer code rows.
pub fn sum_codes(codes: &[i8], ids: &[u32]) -> Vec<i32> {
    let mut v = vec![0i32; HIDDEN];
    for &id in ids {
        let row = &codes[id as usize * HIDDEN..(id as usize + 1) * HIDDEN];
        for (a, b) in v.iter_mut().zip(row) {
            *a += *b as i32;
        }
    }
    v
}

pub fn int_dot(a: &[i32], b: &[i32]) -> i64 {
    a.iter().zip(b).map(|(x, y)| *x as i64 * *y as i64).sum()
}

/// E arm ranking key of one clause against a fixed question: `(dot, |C|^2)`.
/// Clauses are ordered by `dot / |C|` (the cosine up to the constant question
/// norm), compared exactly in integers.
pub fn e8_cmp(a: (i64, i64), b: (i64, i64)) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let (da, na) = a;
    let (db, nb) = b;
    // An empty clause code (norm 0) ranks below everything.
    match (na == 0, nb == 0) {
        (true, true) => return Equal,
        (true, false) => return Less,
        (false, true) => return Greater,
        _ => {}
    }
    let sa = da.signum();
    let sb = db.signum();
    if sa != sb {
        return sa.cmp(&sb);
    }
    // Same sign: compare da^2 * nb with db^2 * na (reversed when negative).
    let la = (da as i128) * (da as i128) * (nb as i128);
    let lb = (db as i128) * (db as i128) * (na as i128);
    if sa >= 0 {
        la.cmp(&lb)
    } else {
        lb.cmp(&la)
    }
}

/// The 240 roots of E8 (norm² 2): (±1,±1,0⁶) permutations and (±½)⁸ with an
/// even number of minus signs.
pub fn e8_roots() -> Vec<[f64; 8]> {
    let mut roots = Vec::with_capacity(240);
    for i in 0..8 {
        for j in (i + 1)..8 {
            for si in [-1.0, 1.0] {
                for sj in [-1.0, 1.0] {
                    let mut r = [0f64; 8];
                    r[i] = si;
                    r[j] = sj;
                    roots.push(r);
                }
            }
        }
    }
    for mask in 0u32..256 {
        if mask.count_ones() % 2 == 0 {
            let mut r = [0.5f64; 8];
            for (k, x) in r.iter_mut().enumerate() {
                if mask & (1 << k) != 0 {
                    *x = -0.5;
                }
            }
            roots.push(r);
        }
    }
    roots
}

// ---------------------------------------------------------------- quaternionic Hopf

pub type Quat = [f64; 4];

/// Hamilton product (1, i, j, k order).
pub fn qmul(a: Quat, b: Quat) -> Quat {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

pub fn qconj(a: Quat) -> Quat {
    [a[0], -a[1], -a[2], -a[3]]
}

pub fn qnorm2(a: Quat) -> f64 {
    a.iter().map(|x| x * x).sum()
}

/// Read an 8-block as the quaternion pair `(q1, q2)` in the project's icosian
/// Z-basis order (`h4.1, h4.i, h4.j, h4.k, phi_h4.1, phi_h4.i, phi_h4.j,
/// phi_h4.k`, `canonical_lexical_ingestion::fixed_icosian_profile`): q1 is the
/// H4 coordinate quaternion, q2 the φH4 companion.
pub fn quat_pair(block: &[f64]) -> (Quat, Quat) {
    (
        [block[0], block[1], block[2], block[3]],
        [block[4], block[5], block[6], block[7]],
    )
}

/// Quaternionic Hopf map S7 → S4 of a unit pair: `(2·q1·q̄2, |q1|²−|q2|²)`.
pub fn hopf_base(q1: Quat, q2: Quat) -> [f64; 5] {
    let p = qmul(q1, qconj(q2));
    [
        2.0 * p[0],
        2.0 * p[1],
        2.0 * p[2],
        2.0 * p[3],
        qnorm2(q1) - qnorm2(q2),
    ]
}

/// Retained S3 fiber coordinate for the section `s(B)` whose first component
/// is real and non-negative: `f = q1/|q1|`, so `(q1, q2) = s(B)·f`. `f = 1`
/// when `q1 = 0`.
pub fn hopf_fiber(q1: Quat) -> Quat {
    let n = qnorm2(q1).sqrt();
    if n < 1e-12 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [q1[0] / n, q1[1] / n, q1[2] / n, q1[3] / n]
    }
}

/// Per-block Hopf features of a 384-d vector: block norm, S4 base point and
/// S3 fiber, optionally after snapping the unit block to the nearest E8 root
/// direction.
pub struct HopfBlocks {
    pub norm: Vec<f64>,
    pub base: Vec<[f64; 5]>,
    pub fiber: Vec<Quat>,
}

pub fn hopf_blocks(v: &[f32], snap: Option<&[[f64; 8]]>) -> HopfBlocks {
    let mut norm = Vec::with_capacity(BLOCKS);
    let mut base = Vec::with_capacity(BLOCKS);
    let mut fiber = Vec::with_capacity(BLOCKS);
    for b in 0..BLOCKS {
        let mut x = [0f64; 8];
        for i in 0..8 {
            x[i] = v[b * 8 + i] as f64;
        }
        let n = x.iter().map(|y| y * y).sum::<f64>().sqrt();
        if n < 1e-12 {
            norm.push(0.0);
            base.push([0.0; 5]);
            fiber.push([0.0; 4]);
            continue;
        }
        let mut u = x.map(|y| y / n);
        if let Some(roots) = snap {
            let mut best = 0usize;
            let mut best_dot = f64::NEG_INFINITY;
            for (k, r) in roots.iter().enumerate() {
                let d: f64 = r.iter().zip(&u).map(|(a, b)| a * b).sum();
                if d > best_dot {
                    best_dot = d;
                    best = k;
                }
            }
            let s = std::f64::consts::SQRT_2;
            u = roots[best].map(|y| y / s);
        }
        let (q1, q2) = quat_pair(&u);
        norm.push(n);
        base.push(hopf_base(q1, q2));
        fiber.push(hopf_fiber(q1));
    }
    HopfBlocks { norm, base, fiber }
}

/// Hopf score of a clause against a question: the block-norm-weighted sum of
/// base-point inner products (plus, with `with_fiber`, the fiber inner
/// product, averaged with the base term), divided by the clause norm
/// `sqrt(Σ n_c²)`.
pub fn hopf_score(q: &HopfBlocks, c: &HopfBlocks, with_fiber: bool) -> f64 {
    let mut s = 0f64;
    let mut cn = 0f64;
    for b in 0..BLOCKS {
        let w = q.norm[b] * c.norm[b];
        cn += c.norm[b] * c.norm[b];
        if w == 0.0 {
            continue;
        }
        let base: f64 = q.base[b].iter().zip(&c.base[b]).map(|(x, y)| x * y).sum();
        if with_fiber {
            let fib: f64 = q.fiber[b].iter().zip(&c.fiber[b]).map(|(x, y)| x * y).sum();
            s += w * 0.5 * (base + fib);
        } else {
            s += w * base;
        }
    }
    if cn == 0.0 {
        return f64::NEG_INFINITY;
    }
    s / cn.sqrt()
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmentation_follows_the_frozen_rule() {
        let c = split_clauses(
            "We got two new pets this week. The turtle is Shelby and the goldfish is Flash.",
        );
        assert_eq!(
            c,
            vec![
                "We got two new pets this week.",
                "The turtle is Shelby",
                "the goldfish is Flash."
            ]
        );
        let c = split_clauses("At the market Grandpa bought plums, and Grandma bought pears.");
        assert_eq!(
            c,
            vec![
                "At the market Grandpa bought plums,",
                "Grandma bought pears."
            ]
        );
        let c = split_clauses("Hopping is allowed but running is not.");
        assert_eq!(c, vec!["Hopping is allowed", "running is not."]);
        let c = split_clauses("tea / coffee; then juice! Really?! ok");
        assert_eq!(c, vec!["tea", "coffee", "then juice!", "Really?!", "ok"]);
        // A period inside a number does not split; "And" at a sentence start does not.
        let c = split_clauses("Café and crème brûlée and tea.");
        assert_eq!(c, vec!["Café", "crème brûlée", "tea."]);
        let c = split_clauses("It costs 3.50 dollars. And then we left");
        assert_eq!(c, vec!["It costs 3.50 dollars.", "And then we left"]);
        let h = segment_history(&["A and B.", "C."]);
        assert_eq!(h, vec!["A", "B.", "C."]);
    }

    #[test]
    fn values_and_hits() {
        let exp = ValueSet::from_spellings(&["plum|plums"]);
        let forb = ValueSet::from_spellings(&["pear|pears"]);
        assert!(clause_hit("Grandpa bought plums,", &exp, &forb));
        assert!(!clause_hit("Grandma bought pears.", &exp, &forb));
        assert!(clause_mixed("plums and pears", &exp, &forb));
        let exp = ValueSet::from_spellings(&["in the bathroom cabinet"]);
        assert!(clause_hit(
            "It's in the bathroom cabinet now",
            &exp,
            &ValueSet::default()
        ));
        assert!(!clause_hit("the bathroom sink", &exp, &ValueSet::default()));
        let k = ValueSet::from_spellings(&["goldfish|goldfish's"]);
        assert!(k.present_in(&words("the goldfish's bowl").into_iter().collect()));
        assert!(ValueSet::from_spellings(&["-"]).is_empty());
    }

    #[test]
    fn lexical_scorer_counts_shared_content_words_and_breaks_ties_to_recent() {
        assert_eq!(
            lexical_score("Which name did we give the turtle?", "The turtle is Shelby"),
            1
        );
        assert_eq!(
            lexical_score(
                "Which name did we give the turtle?",
                "the goldfish is Flash."
            ),
            0
        );
        let clauses = ["The turtle is Shelby", "the goldfish is Flash."];
        assert_eq!(
            lexical_choice("Which name did we give the turtle?", &clauses),
            Some(0)
        );
        // all-zero: most recent
        assert_eq!(lexical_choice("hello there", &clauses), Some(1));
        // tie: most recent
        let clauses = ["red kite", "red ball"];
        assert_eq!(lexical_choice("the red one?", &clauses), Some(1));
        assert_eq!(lexical_choice::<&str>("x", &[]), None);
    }

    #[test]
    fn e8_round_trip_error_is_bounded() {
        // Deterministic pseudo-random vector.
        let mut s = 0x9e3779b97f4a7c15u64;
        let v: Vec<f32> = (0..HIDDEN)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                ((s >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) as f32
            })
            .collect();
        let rms = (v.iter().map(|x| x * x).sum::<f32>() / HIDDEN as f32).sqrt();
        let scale = E8_SCALE_K / rms;
        let codes = e8_quantize(&v, scale).expect("quantize");
        let back = e8_dequantize(&codes, scale);
        // Each 8-block is within the E8 covering radius (1) of its point in
        // scaled units: |v - back|^2 <= 48 / scale^2.
        let err2: f32 = v.iter().zip(&back).map(|(a, b)| (a - b) * (a - b)).sum();
        assert!(
            err2 <= BLOCKS as f32 / (scale * scale) + 1e-4,
            "err2 {err2}"
        );
        // Every block is an E8 point: doubled coords all even or all odd, sum ≡ 0 mod 4.
        for blk in codes.chunks(8) {
            let parity = blk[0].rem_euclid(2);
            assert!(blk.iter().all(|x| x.rem_euclid(2) == parity));
            let sum: i32 = blk.iter().map(|&x| x as i32).sum();
            assert_eq!(sum.rem_euclid(4), 0);
        }
        // Relative error is well below 1 at K = 2.
        let rel = (err2 / v.iter().map(|x| x * x).sum::<f32>()).sqrt();
        assert!(rel < 0.5, "relative error {rel}");
    }

    #[test]
    fn e8_integer_ranking_matches_cosine_order() {
        use std::cmp::Ordering::*;
        assert_eq!(e8_cmp((3, 9), (2, 9)), Greater);
        assert_eq!(e8_cmp((2, 4), (3, 16)), Greater); // 1.0 vs 0.75
        assert_eq!(e8_cmp((-1, 1), (0, 4)), Less);
        assert_eq!(e8_cmp((-1, 4), (-1, 1)), Greater); // -0.5 vs -1
        assert_eq!(e8_cmp((5, 0), (-9, 1)), Less);
        assert_eq!(e8_roots().len(), 240);
        assert!(e8_roots()
            .iter()
            .all(|r| (r.iter().map(|x| x * x).sum::<f64>() - 2.0).abs() < 1e-12));
    }

    #[test]
    fn hopf_base_is_fiber_invariant_and_on_s4() {
        let n = |v: [f64; 8]| {
            let s = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            v.map(|x| x / s)
        };
        let x = n([0.3, -1.2, 0.7, 0.1, 2.0, -0.4, 0.9, -0.6]);
        let (q1, q2) = quat_pair(&x);
        let b = hopf_base(q1, q2);
        let bn: f64 = b.iter().map(|y| y * y).sum();
        assert!((bn - 1.0).abs() < 1e-12, "base norm^2 {bn}");
        let u0 = [0.5, -0.1, 0.7, 0.3];
        let un = qnorm2(u0).sqrt();
        let u = u0.map(|y| y / un);
        let b2 = hopf_base(qmul(q1, u), qmul(q2, u));
        for (a, c) in b.iter().zip(&b2) {
            assert!((a - c).abs() < 1e-12);
        }
        // Fiber section reconstructs the pair: (|q1|, q2 q̄1/|q1|)·f = (q1, q2).
        let f = hopf_fiber(q1);
        let r = qnorm2(q1).sqrt();
        let s2 = qmul(q2, qconj(q1)).map(|y| y / r);
        let back1 = qmul([r, 0.0, 0.0, 0.0], f);
        let back2 = qmul(s2, f);
        for i in 0..4 {
            assert!((back1[i] - q1[i]).abs() < 1e-12);
            assert!((back2[i] - q2[i]).abs() < 1e-12);
        }
        // Score of a vector against itself is its norm (base and fiber terms are 1).
        let v: Vec<f32> = (0..HIDDEN).map(|i| ((i * 7 % 13) as f32) - 6.0).collect();
        let hb = hopf_blocks(&v, None);
        let norm = v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
        assert!((hopf_score(&hb, &hb, false) - norm).abs() < 1e-6 * norm);
        assert!((hopf_score(&hb, &hb, true) - norm).abs() < 1e-6 * norm);
    }

    /// Needs the downloaded teacher: `UOR_MINILM_DIR=~/.cache/uor-minilm
    /// cargo test ... -- --ignored minilm_parity`. Checks the Rust forward
    /// against the cosine matrix published in the sentence-transformers
    /// quickstart (4 decimals) and against the first eight coordinates this
    /// forward produced once (2026-10-05; self-computed regression values,
    /// not external references).
    #[test]
    #[ignore]
    fn minilm_parity() {
        let dir =
            std::path::PathBuf::from(std::env::var("UOR_MINILM_DIR").expect("UOR_MINILM_DIR"));
        let tok = WordPiece::from_tokenizer_json(&dir.join("tokenizer.json")).expect("tokenizer");
        let model = MiniLm::load(&dir.join("model.safetensors")).expect("model");
        assert_eq!(
            tok.encode("This is an example sentence"),
            vec![101, 2023, 2003, 2019, 2742, 6251, 102]
        );
        let s = [
            "The weather is lovely today.",
            "It's so sunny outside!",
            "He drove to the stadium.",
        ];
        let e: Vec<Vec<f32>> = s
            .iter()
            .map(|t| model.sentence_embedding(&tok, t).expect("emb"))
            .collect();
        let published = [
            [1.0, 0.6660, 0.1046],
            [0.6660, 1.0, 0.1411],
            [0.1046, 0.1411, 1.0],
        ];
        for i in 0..3 {
            for j in 0..3 {
                assert!((cosine(&e[i], &e[j]) - published[i][j]).abs() < 1e-3);
            }
        }
        let first8: [[f32; 8]; 3] = [
            [
                0.019195817,
                0.120085366,
                0.15959838,
                0.06706591,
                0.050074797,
                -0.02591874,
                0.056468222,
                -0.092857793,
            ],
            [
                -0.018690312,
                0.041518636,
                0.074315451,
                0.078432761,
                0.075569727,
                -0.012507522,
                0.088356853,
                -0.069788672,
            ],
            [
                0.136502013,
                0.082273148,
                -0.025261560,
                0.030451400,
                0.053312045,
                0.050809182,
                0.086164169,
                0.099992007,
            ],
        ];
        for (row, refs) in e.iter().zip(first8) {
            for (a, b) in row.iter().zip(refs) {
                assert!((a - b).abs() < 1e-4, "{a} vs {b}");
            }
        }
    }
}
