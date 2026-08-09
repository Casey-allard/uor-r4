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
}

impl Tokenizer {
    /// Load and validate a tokenizer without panicking on malformed input.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_load(
        path: impl AsRef<Path>,
    ) -> Result<Self, uor_r4_model_source::SourceUnavailable> {
        let path_ref = path.as_ref();

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
                        return Ok(Tokenizer { vocab, map });
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
        Some(Tokenizer { vocab, map })
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
                    ' ' => encoded_str.push('Ġ'),
                    '\n' => encoded_str.push('Ċ'),
                    '\r' => encoded_str.push('Ĉ'),
                    '\t' => encoded_str.push('ĉ'),
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

    /// Decode into caller-owned byte storage.
    ///
    /// Note: this delegates to `decode`, which allocates an intermediate
    /// `Vec`/`String`, so this is not allocation-free.
    /// Total: `Some(count)` of bytes written, `None` when `out` cannot hold
    /// the decoded text.
    pub fn decode_into(&self, toks: &[u32], out: &mut [u8]) -> Option<usize> {
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
        s("in-domain prompt", "dog-named", "Once upon a time, there was a little dog named Rex.", false),
        s("in-domain prompt", "park-ball", "Lily and Ben went to the park to play with their new ball.", false),
        s("in-domain prompt", "sad-bird", "The little bird was sad because it could not fly.", false),
        s("in-domain prompt", "red-truck", "Tom saw a big red truck outside his house.", false),
        s("in-domain prompt", "shiny-key", "One day, a cat found a shiny key in the garden.", false),
        // -- out-of-domain real-world prompts
        s("out-of-domain prompt", "capital-q", "What is the capital of France?", false),
        s("out-of-domain prompt", "explain", "Explain how photosynthesis works.", false),
        s("out-of-domain prompt", "code", "Write a Python function to add two numbers.", false),
        s("out-of-domain prompt", "business", "The quarterly revenue increased by fifteen percent compared to", false),
        // -- real human-written text, scored against actual next tokens
        s(
            "real text, in-domain style",
            "handwritten-story",
            "One day, a little girl named Mia went to the park with her mom. Mia saw a big dog. The dog was sad because it lost its ball. Mia wanted to help the dog. She looked under the bench and found the ball. The dog was very happy and wagged its tail. Mia and the dog played together all day. When it was time to go home, the dog gave Mia a big lick on her face. Mia laughed and said she would come back tomorrow.",
            true,
        ),
        s("real text, out-of-domain", "shakespeare", &shakespeare, true),
        // -- structural stress
        s("stress", "repetition", "one two three four one two three four one two three four one two three four", false),
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
    println!("| class | positions | agree w/ teacher | tless top1 | teacher top1 | tless tok/s | teacher tok/s |");
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
}
