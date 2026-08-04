//! The COMPILER: teacher-time cross-compilation of a transformer source into
//! the table-native target. Multiplication is permitted here — it runs once,
//! offline, and every output is a frozen, blake3-κ-pinned artifact:
//!
//!   1. token codebook: RVQ (4 stages × 256) over the source's embedding
//!      table (mean-centered, normalized) — the "representation" surface;
//!   2. integer token vectors: i8-quantized stage-entry sums per token;
//!   3. context thresholds: per-dimension train means of the dyadic-recency
//!      bundle — the "vector at each bit": bit b of a context signature
//!      records the side of threshold b, a prefix of bits an intersection
//!      of regions;
//!   4. class signatures: context-RVQ centroids binarized to sign bits, so
//!      runtime assignment is Hamming (xor + popcount table + add);
//!   5. the datastore: graded context classes → next-token counts, built by
//!      running the corpus that the TEACHER generated and labeled (the
//!      "behavior" surface). The teacher never appears at runtime.
//!
//! The source-architecture interface is exactly two surfaces — an embedding
//! table and a next-token oracle — which is what makes the compilation
//! architecture-generic (llama / qwen / phi differ only in the teacher
//! adapter). This crate instantiates the llama-family adapter.

use std::collections::{BTreeMap, HashMap};
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
use uor_r4_model_source::{RepresentationSource, TeacherOracle};

#[inline]
fn canonical_math_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("TLESS_CANONICAL_DETERMINISTIC").is_ok_and(|value| value != "0")
    })
}

#[inline]
fn canonical_expf(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::expf(value)
    } else {
        value.exp()
    }
}

#[cfg(target_os = "macos")]
#[inline]
fn teacher_vector_exp_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        // Accelerate's vForce implementation is CPU vector math.  Use it by
        // default on macOS for the noncanonical teacher path; the explicit
        // zero value remains an escape hatch for scalar comparisons.
        let requested = std::env::var("TLESS_TEACHER_VFORCE_EXP")
            .map_or(cfg!(target_os = "macos"), |value| value != "0");
        !canonical_math_enabled() && requested
    })
}

#[cfg(target_os = "macos")]
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn vvexpf(output: *mut f32, input: *const f32, count: *const i32);
}

/// Exponentiate a vocabulary-sized logit vector and return its sum.
///
/// The macOS path uses Accelerate's CPU vForce implementation in place. It is
/// disabled for canonical math and can be disabled with
/// `TLESS_TEACHER_VFORCE_EXP=0`; it is a teacher-side backend, not part of the
/// κ-pinned deterministic mode.
#[inline]
fn teacher_exp_sum(logits: &mut [f32], max_logit: f32) -> f32 {
    #[cfg(target_os = "macos")]
    if teacher_vector_exp_enabled() {
        for logit in logits.iter_mut() {
            *logit -= max_logit;
        }
        let count = logits.len().min(i32::MAX as usize) as i32;
        // SAFETY: output and input point to the same initialized contiguous
        // slice, which vForce supports for its unary vector functions.
        unsafe { vvexpf(logits.as_mut_ptr(), logits.as_ptr(), &count) };
        return logits.iter().copied().sum();
    }

    let mut sum = 0.0f32;
    for logit in logits.iter_mut() {
        *logit = canonical_expf(*logit - max_logit);
        sum += *logit;
    }
    sum
}

#[inline]
fn canonical_logf(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::logf(value)
    } else {
        value.ln()
    }
}

/// Compiler-side information about one normalized next-token distribution.
///
/// The deployed runtime does not carry this floating-point structure. It is
/// captured while the teacher logits are still available so message
/// likelihood and uncertainty can be measured without reconstructing a
/// distribution from top-k evidence later.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenProbabilityStats {
    /// Natural-log probability of the observed target token.
    pub target_logprob_nats: f32,
    /// Shannon entropy of the complete vocabulary distribution, in bits.
    pub entropy_bits: f32,
    /// Probability mass covered by the retained top-8 tokens.
    pub top8_mass: f32,
    /// Zero-based rank of the target in the full distribution, or `u16::MAX`
    /// when it is outside the retained top-8 list.
    pub target_rank: u16,
}

impl TokenProbabilityStats {
    /// Calculate target likelihood and full-distribution entropy from a
    /// normalized vocabulary distribution. `top8_mass` is calculated from
    /// the unrenormalized top-8 probabilities before they are stored as
    /// teacher evidence.
    pub fn from_normalized(
        probabilities: &[f32],
        target: usize,
        top_tokens: &[u32; 8],
        top8_mass: f32,
    ) -> Self {
        let target_probability = probabilities.get(target).copied().unwrap_or(0.0);
        let target_logprob_nats = if target_probability > 0.0 {
            canonical_logf(target_probability)
        } else {
            f32::NEG_INFINITY
        };
        let entropy_bits = probabilities
            .iter()
            .copied()
            .filter(|&probability| probability > 0.0)
            .map(|probability| -probability * canonical_log2f(probability))
            .sum();
        let target_rank = top_tokens
            .iter()
            .position(|&token| token == target as u32)
            .map(|rank| rank as u16)
            .unwrap_or(u16::MAX);
        Self {
            target_logprob_nats,
            entropy_bits,
            top8_mass: top8_mass.clamp(0.0, 1.0),
            target_rank,
        }
    }

    /// Information content of the target token in bits.
    pub fn target_bits(self) -> f32 {
        -self.target_logprob_nats / std::f32::consts::LN_2
    }
}

#[inline]
fn canonical_log2f(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::log2f(value)
    } else {
        value.log2()
    }
}

#[inline]
fn canonical_exp2f(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::exp2f(value)
    } else {
        value.exp2()
    }
}

#[inline]
fn canonical_sqrtf(value: f32) -> f32 {
    if canonical_math_enabled() {
        libm::sqrtf(value)
    } else {
        value.sqrt()
    }
}

pub const STAGES: usize = 4;
pub const K: usize = 256;
pub const D: usize = 288;
pub const V: usize = 32000;
pub const WINDOW: usize = 8;
pub const ROT: usize = 17;
const EMB_ITERS: usize = 10;
const CTX_SAMPLE: usize = 50_000;
const CTX_ITERS: usize = 6;

pub fn xorshift(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    x
}

// ---------------------------------------------------------------- corpus --

/// Teacher-labeled corpus record stream (identical wire format to the
/// research pipeline, so existing state is adoptable):
/// meta: n u64 | stories u64 | rng u64 | done u8
/// recs (v3): per token:
///   story u32 | next u32 | top_tokens [u32; 3] | top_weights [u32; 3]
///   | span_start u32 | span_end u32 | byte_start u32 | byte_end u32
/// legacy record sizes are still accepted on load:
///   v1=12 bytes (story,next,argmax,logprob) and v2=32 bytes (no anchors).
#[derive(Clone)]
pub struct Corpus {
    pub n: usize,
    pub stories: u64,
    pub story: Vec<u32>,
    pub input: Vec<u32>,
    pub next: Vec<u32>,
    pub t_argmax: Vec<u32>,
    pub top_tokens: Vec<[u32; 8]>,
    pub top_weights: Vec<[u32; 8]>,
    pub span_start: Vec<u32>,
    pub span_end: Vec<u32>,
    pub byte_start: Vec<u32>,
    pub byte_end: Vec<u32>,
    pub hidden: Option<Vec<Vec<f32>>>,
}

/// A representation source reconstructed from a recorded corpus.
///
/// This is the compiler-side counterpart to the reference project's
/// observation-first boundary: the corpus compiler consumes recorded top-k
/// next-token observations and never loads a teacher model. Rows that were not observed are
/// filled with a deterministic token-id row so the resulting artifact still
/// has the caller-requested vocabulary width; those rows are not evidence of
/// teacher semantics and are reported through the source κ.
pub struct RecordedRepresentation {
    rows: Vec<f32>,
    vocab: usize,
    kappa: String,
}

impl RecordedRepresentation {
    /// Build a representation from recorded next-token observations. An
    /// optional `<records>.hidden` sidecar may exist, but is deliberately not
    /// read: the production compiler's input contract is the portable corpus
    /// record stream itself.
    pub fn from_corpus(corpus: &Corpus, vocab: usize) -> Result<Self, String> {
        if vocab == 0 {
            return Err("recorded compile requires a non-zero vocabulary size".to_owned());
        }
        if vocab > u32::MAX as usize {
            return Err("recorded compile vocabulary exceeds u32 token ids".to_owned());
        }
        if corpus.input.len() != corpus.n
            || corpus.top_tokens.len() != corpus.n
            || corpus.top_weights.len() != corpus.n
        {
            return Err("recorded corpus arrays do not match the record count".to_owned());
        }
        let mut sums = vec![0.0f64; vocab * D];
        let mut counts = vec![0u64; vocab];
        for index in 0..corpus.n {
            let token = corpus.input[index] as usize;
            if token >= vocab {
                continue;
            }
            counts[token] += 1;
            let base = token * D;
            for (&target, &weight) in corpus.top_tokens[index]
                .iter()
                .zip(corpus.top_weights[index].iter())
            {
                if weight == 0 {
                    continue;
                }
                let mut feature_hasher = blake3::Hasher::new();
                feature_hasher.update(b"uor-r4-recorded-feature-v1");
                feature_hasher.update(&target.to_le_bytes());
                let digest = feature_hasher.finalize();
                for dimension in 0..D {
                    let byte = digest.as_bytes()[dimension % 32];
                    let sign = if byte & (1 << (dimension % 8)) == 0 {
                        -1.0
                    } else {
                        1.0
                    };
                    sums[base + dimension] += sign * f64::from(weight);
                }
            }
        }

        let mut rows = vec![0.0f32; vocab * D];
        for (token, &count_value) in counts.iter().enumerate() {
            let base = token * D;
            if count_value != 0 {
                let count = count_value as f64;
                for dimension in 0..D {
                    rows[base + dimension] = (sums[base + dimension] / count) as f32;
                }
            } else {
                // Deterministic, non-semantic fallback for vocabulary rows
                // absent from the captured corpus. The compiler remains
                // total without pretending that these rows were observed.
                let mut hasher = blake3::Hasher::new();
                hasher.update(b"uor-r4-recorded-row-v1");
                hasher.update(&(token as u64).to_le_bytes());
                let digest = hasher.finalize();
                for dimension in 0..D {
                    let byte = digest.as_bytes()[dimension % 32];
                    rows[base + dimension] = if byte & (1 << (dimension % 8)) == 0 {
                        -1.0
                    } else {
                        1.0
                    };
                }
            }
        }

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"uor-r4-recorded-representation-v1");
        hasher.update(&(vocab as u64).to_le_bytes());
        for value in &rows {
            hasher.update(&value.to_le_bytes());
        }
        Ok(Self {
            rows,
            vocab,
            kappa: format!("blake3:{}", hasher.finalize().to_hex()),
        })
    }

    pub fn kappa(&self) -> &str {
        &self.kappa
    }
}

impl RepresentationSource for RecordedRepresentation {
    fn vocab_size(&self) -> usize {
        self.vocab
    }

    fn source_dimension(&self) -> usize {
        D
    }

    fn tokenizer_address(&self) -> &str {
        "recorded-corpus-tokenizer-v1"
    }

    fn read_embedding_rows(
        &self,
        range: std::ops::Range<usize>,
        output: &mut [f32],
    ) -> Result<(), String> {
        if range.start > range.end || range.end > self.vocab {
            return Err("recorded representation row range is out of bounds".to_owned());
        }
        let count = range.end - range.start;
        let width = count * D;
        if output.len() < width {
            return Err("recorded representation output buffer too small".to_owned());
        }
        output[..width].copy_from_slice(&self.rows[range.start * D..range.end * D]);
        Ok(())
    }
}

/// Compile directly from a completed recorded corpus without loading a
/// teacher or invoking any model forward path.
pub fn compile_recorded(corpus: &Corpus, vocab: usize) -> Result<Compiled, String> {
    if corpus.n == 0 || corpus.stories == 0 {
        return Err("recorded compile requires a non-empty corpus".to_owned());
    }
    if corpus.input.len() != corpus.n {
        return Err("corpus input/record count mismatch".to_owned());
    }
    let source = RecordedRepresentation::from_corpus(corpus, vocab)?;
    let source_kappa = source.kappa().to_owned();
    let source_bytes = source.rows.len() * std::mem::size_of::<f32>();
    Ok(compile_from_representation(
        &source,
        &source_kappa,
        source_bytes,
        source.tokenizer_address(),
        corpus,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn corpus_paths() -> (&'static str, &'static str) {
    ("/tmp/c_meta.bin", "/tmp/c_recs.bin")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_corpus() -> Option<Corpus> {
    // Keep the historical defaults for normal certification, but allow
    // re-pin measurements to point at an explicitly preserved corpus. This
    // avoids replacing the checked-in era fixtures before their κ set has
    // been reviewed. The override is compiler/test tooling only; deployed
    // runtime code never loads a corpus.
    let (default_meta, default_recs) = corpus_paths();
    let meta = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| default_meta.to_owned());
    let recs = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| default_recs.to_owned());
    load_corpus_from(&meta, &recs)
}

/// Load a corpus record stream from explicit paths (fixtures, mirrors).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_corpus_from(mp: &str, rp: &str) -> Option<Corpus> {
    let meta = std::fs::read(mp).ok()?;
    if meta.len() != 25 || meta[24] != 1 {
        return None;
    }
    let n = u64::from_le_bytes(meta[0..8].try_into().unwrap()) as usize;
    let stories = u64::from_le_bytes(meta[8..16].try_into().unwrap());
    let rb_full = std::fs::read(rp).ok()?;
    let record_size = if rb_full.len() >= n * 88 && (n > 0 && rb_full.len() % 88 == 0) {
        88usize
    } else if rb_full.len() >= n * 48 {
        48usize
    } else if rb_full.len() >= n * 32 {
        32usize
    } else if rb_full.len() >= n * 12 {
        12usize
    } else {
        return None;
    };
    if rb_full.len() < n * record_size {
        return None;
    }
    let rb = &rb_full[..n * record_size];
    let is_legacy = record_size == 12;
    let has_anchors = record_size == 48 || record_size == 88;
    let mut story = Vec::with_capacity(n);
    let mut next = Vec::with_capacity(n);
    let mut t_argmax = Vec::with_capacity(n);
    let mut top_tokens = Vec::with_capacity(n);
    let mut top_weights = Vec::with_capacity(n);
    let mut span_start = Vec::with_capacity(n);
    let mut span_end = Vec::with_capacity(n);
    let mut byte_start = Vec::with_capacity(n);
    let mut byte_end = Vec::with_capacity(n);
    let mut current_story = None;
    let mut position_in_story = 0u32;
    for i in 0..n {
        if is_legacy {
            let o = i * 12;
            let story_id = u32::from_le_bytes(rb[o..o + 4].try_into().unwrap());
            story.push(story_id);
            let nxt = u16::from_le_bytes(rb[o + 4..o + 6].try_into().unwrap()) as u32;
            next.push(nxt);
            let argmax = u16::from_le_bytes(rb[o + 6..o + 8].try_into().unwrap()) as u32;
            t_argmax.push(argmax);
            let lp = f32::from_le_bytes(rb[o + 8..o + 12].try_into().unwrap());
            let next_prob = (canonical_expf(lp) * 100.0).clamp(0.0, 100.0) as u32;

            let mut tokens_val = [0u32; 8];
            let mut weights_val = [0u32; 8];
            tokens_val[0] = argmax;
            if nxt == argmax {
                weights_val[0] = next_prob.max(50);
            } else {
                weights_val[0] = (100 - next_prob).min(90);
                tokens_val[1] = nxt;
                weights_val[1] = next_prob;
            }
            top_tokens.push(tokens_val);
            top_weights.push(weights_val);
        } else {
            let o = i * record_size;
            let story_id = u32::from_le_bytes(rb[o..o + 4].try_into().unwrap());
            story.push(story_id);
            let nxt = u32::from_le_bytes(rb[o + 4..o + 8].try_into().unwrap());
            next.push(nxt);
            let mut tokens_val = [0u32; 8];
            let mut weights_val = [0u32; 8];
            let k_count = if record_size == 88 { 8 } else { 3 };
            let wt_base_offset = 8 + k_count * 4;
            for j in 0..k_count {
                let offset_tok = o + 8 + j * 4;
                let offset_wt = o + wt_base_offset + j * 4;
                tokens_val[j] =
                    u32::from_le_bytes(rb[offset_tok..offset_tok + 4].try_into().unwrap());
                weights_val[j] =
                    u32::from_le_bytes(rb[offset_wt..offset_wt + 4].try_into().unwrap());
            }
            t_argmax.push(tokens_val[0]);
            top_tokens.push(tokens_val);
            top_weights.push(weights_val);
            if has_anchors {
                let anchor_offset = o + 8 + k_count * 8;
                span_start.push(u32::from_le_bytes(
                    rb[anchor_offset..anchor_offset + 4].try_into().unwrap(),
                ));
                span_end.push(u32::from_le_bytes(
                    rb[anchor_offset + 4..anchor_offset + 8].try_into().unwrap(),
                ));
                byte_start.push(u32::from_le_bytes(
                    rb[anchor_offset + 8..anchor_offset + 12]
                        .try_into()
                        .unwrap(),
                ));
                byte_end.push(u32::from_le_bytes(
                    rb[anchor_offset + 12..anchor_offset + 16]
                        .try_into()
                        .unwrap(),
                ));
                continue;
            }
        }
        let story_id = story[i];
        if current_story != Some(story_id) {
            current_story = Some(story_id);
            position_in_story = 0;
        }
        span_start.push(position_in_story);
        span_end.push(position_in_story.saturating_add(1));
        byte_start.push(u32::MAX);
        byte_end.push(u32::MAX);
        position_in_story = position_in_story.saturating_add(1);
    }
    let mut input = Vec::with_capacity(n);
    for i in 0..n {
        if i == 0 || story[i] != story[i - 1] {
            input.push(1u32);
        } else {
            input.push(next[i - 1]);
        }
    }
    let mut c = Corpus {
        n,
        stories,
        story,
        input,
        next,
        t_argmax,
        top_tokens,
        top_weights,
        span_start,
        span_end,
        byte_start,
        byte_end,
        hidden: None,
    };

    let mut hidden_path = String::from(rp);
    if hidden_path.ends_with("_recs.bin") {
        hidden_path = hidden_path.replace("_recs.bin", "_hidden.bin");
    } else {
        hidden_path.push_str(".hidden");
    }

    if let Ok(hb) = std::fs::read(&hidden_path) {
        // Teacher hidden state is dimension 288 (D)
        if hb.len() == n * D * 4 {
            let mut hidden = Vec::with_capacity(n);
            let mut offset = 0;
            for _ in 0..n {
                let mut v = Vec::with_capacity(D);
                for _ in 0..D {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&hb[offset..offset + 4]);
                    v.push(f32::from_le_bytes(b));
                    offset += 4;
                }
                hidden.push(v);
            }
            c.hidden = Some(hidden);
        }
    }

    Some(c)
}

/// One teacher decode step of corpus/observation generation: softmax the
/// logits in place, extract the top-3 tokens with normalized integer
/// weights, and sample the next token from the full CDF. Consumes exactly
/// one `rng` draw. Shared by the resumable corpus writer and the sharded
/// observation pipeline (`super::observe`) so both emit byte-identical v3
/// records for the same teacher stream; the arithmetic below is the
/// κ-pinned record semantics and must not change.
/// Top-N indices by descending probability, ties to the lowest index —
/// identical output to a stable descending full sort (the previous
/// implementation) at O(vocab·N) with no allocation. Strict `>` keeps an
/// earlier equal-probability token ahead, matching stable-sort order.
fn top_n_desc_stable<const N: usize>(probs: &[f32]) -> [(u32, f32); N] {
    let mut top = [(0u32, f32::NEG_INFINITY); N];
    for (i, &p) in probs.iter().enumerate() {
        if p > top[N - 1].1 {
            let mut pos = N - 1;
            while pos > 0 && p > top[pos - 1].1 {
                top[pos] = top[pos - 1];
                pos -= 1;
            }
            top[pos] = (i as u32, p);
        }
    }
    top
}

pub fn softmax_top3_sample(logits: &mut [f32], rng: &mut u64) -> (usize, [u32; 3], [u32; 3]) {
    let mut mx = logits[0];
    for &v in &logits[1..] {
        if v > mx {
            mx = v;
        }
    }
    let sum = teacher_exp_sum(logits, mx);
    for p in logits.iter_mut() {
        *p /= sum;
    }

    // Find top-3 tokens and their normalized weights
    let top_candidates = top_n_desc_stable::<3>(logits);

    let mut top_tokens_idx = [0u32; 3];
    let mut top_weights_val = [0u32; 3];

    let mut sum_top3 = 0.0f32;
    for i in 0..3 {
        if i < top_candidates.len() {
            sum_top3 += top_candidates[i].1;
        }
    }
    if sum_top3 > 1e-9 {
        let mut accumulated = 0;
        for i in 0..3 {
            if i < top_candidates.len() {
                top_tokens_idx[i] = top_candidates[i].0;
                let w = ((top_candidates[i].1 / sum_top3) * 100.0).round() as u32;
                top_weights_val[i] = w;
                accumulated += w;
            }
        }
        if accumulated != 100 && top_weights_val[0] > 0 {
            // Signed arithmetic: rounding the three weights can overshoot
            // 100 (e.g. 34+34+34). Release builds already treat the
            // difference as signed via a wrapping u32 subtraction; compute
            // it in i32 directly so debug builds produce the same bytes
            // instead of panicking on the underflow.
            let diff = 100i32 - accumulated as i32;
            top_weights_val[0] = (top_weights_val[0] as i32 + diff).max(0) as u32;
        }
    }

    // Sample the next token using the full logits cdf
    let u = (xorshift(rng) >> 40) as f32 / (1u64 << 24) as f32;
    let mut cdf = 0.0f32;
    let mut next = logits.len() - 1;
    for (i, &p) in logits.iter().enumerate() {
        cdf += p;
        if u < cdf {
            next = i;
            break;
        }
    }
    (next, top_tokens_idx, top_weights_val)
}

/// Normalize logits, retain the top-8 evidence, and sample from the full
/// vocabulary distribution. The returned statistics describe the sampled
/// target and the original (not top-8-renormalized) distribution.
pub fn softmax_top8_sample_with_stats(
    logits: &mut [f32],
    rng: &mut u64,
) -> (usize, [u32; 8], [u32; 8], TokenProbabilityStats) {
    let mut mx = f32::NEG_INFINITY;
    for &p in logits.iter() {
        if p > mx {
            mx = p;
        }
    }
    let sum = teacher_exp_sum(logits, mx);
    for p in logits.iter_mut() {
        *p /= sum;
    }

    let top_candidates = top_n_desc_stable::<8>(logits);
    let top8_mass = top_candidates
        .iter()
        .map(|(_, probability)| *probability)
        .filter(|probability| probability.is_finite() && *probability > 0.0)
        .sum::<f32>();

    let mut top_tokens_idx = [0u32; 8];
    let mut top_weights_val = [0u32; 8];

    let mut sum_top8 = 0.0f32;
    for i in 0..8 {
        if i < top_candidates.len() {
            sum_top8 += top_candidates[i].1;
        }
    }
    if sum_top8 > 1e-9 {
        let mut accumulated = 0;
        for i in 0..8 {
            if i < top_candidates.len() {
                top_tokens_idx[i] = top_candidates[i].0;
                let w = ((top_candidates[i].1 / sum_top8) * 100.0).round() as u32;
                top_weights_val[i] = w;
                accumulated += w;
            }
        }
        if accumulated != 100 && top_weights_val[0] > 0 {
            let diff = 100i32 - accumulated as i32;
            top_weights_val[0] = (top_weights_val[0] as i32 + diff).max(0) as u32;
        }
    }

    let u = (xorshift(rng) >> 40) as f32 / (1u64 << 24) as f32;
    let mut cdf = 0.0f32;
    let mut next = logits.len() - 1;
    for (i, &p) in logits.iter().enumerate() {
        cdf += p;
        if u < cdf {
            next = i;
            break;
        }
    }
    let stats = TokenProbabilityStats::from_normalized(logits, next, &top_tokens_idx, top8_mass);
    (next, top_tokens_idx, top_weights_val, stats)
}

/// Compatibility wrapper for callers that only need the sampled top-8
/// evidence. New observation producers should use
/// [`softmax_top8_sample_with_stats`] so probability metadata is preserved.
pub fn softmax_top8_sample(logits: &mut [f32], rng: &mut u64) -> (usize, [u32; 8], [u32; 8]) {
    let (next, top_tokens, top_weights, _) = softmax_top8_sample_with_stats(logits, rng);
    (next, top_tokens, top_weights)
}

/// Tokenizer-neutral byte anchors of one sampled token within its story.
/// Without per-token byte lengths the anchors are the v3 "unknown" value
/// (u32::MAX), matching the v2 backfill semantics on load.
pub fn byte_anchors(
    token_byte_lengths: Option<&[u32]>,
    story_byte_offset: u32,
    next: usize,
) -> (u32, u32) {
    let Some(lengths) = token_byte_lengths else {
        return (u32::MAX, u32::MAX);
    };
    let token_bytes = lengths.get(next).copied().unwrap_or(0);
    (
        story_byte_offset,
        story_byte_offset.saturating_add(token_bytes),
    )
}

/// Encode one v3 corpus/observation record (48 bytes):
///   story u32 | next u32 | top_tokens [u32; 3] | top_weights [u32; 3]
///   | span_start u32 | span_end u32 | byte_start u32 | byte_end u32
pub fn encode_v3_record(
    story: u32,
    next: u32,
    top_tokens: &[u32; 3],
    top_weights: &[u32; 3],
    span: (u32, u32),
    bytes: (u32, u32),
) -> [u8; 48] {
    let (span_start, span_end) = span;
    let (byte_start, byte_end) = bytes;
    let mut record = [0u8; 48];
    record[0..4].copy_from_slice(&story.to_le_bytes());
    record[4..8].copy_from_slice(&next.to_le_bytes());
    for i in 0..3 {
        let offset_tok = 8 + i * 4;
        let offset_wt = 20 + i * 4;
        record[offset_tok..offset_tok + 4].copy_from_slice(&top_tokens[i].to_le_bytes());
        record[offset_wt..offset_wt + 4].copy_from_slice(&top_weights[i].to_le_bytes());
    }
    record[32..36].copy_from_slice(&span_start.to_le_bytes());
    record[36..40].copy_from_slice(&span_end.to_le_bytes());
    record[40..44].copy_from_slice(&byte_start.to_le_bytes());
    record[44..48].copy_from_slice(&byte_end.to_le_bytes());
    record
}

/// Encode one v4 corpus/observation record (88 bytes):
///   story u32 | next u32 | top_tokens [u32; 8] | top_weights [u32; 8]
///   | span_start u32 | span_end u32 | byte_start u32 | byte_end u32
pub fn encode_v4_record(
    story: u32,
    next: u32,
    top_tokens: &[u32; 8],
    top_weights: &[u32; 8],
    span: (u32, u32),
    bytes: (u32, u32),
) -> [u8; 88] {
    let (span_start, span_end) = span;
    let (byte_start, byte_end) = bytes;
    let mut record = [0u8; 88];
    record[0..4].copy_from_slice(&story.to_le_bytes());
    record[4..8].copy_from_slice(&next.to_le_bytes());
    for i in 0..8 {
        let offset_tok = 8 + i * 4;
        let offset_wt = 40 + i * 4;
        record[offset_tok..offset_tok + 4].copy_from_slice(&top_tokens[i].to_le_bytes());
        record[offset_wt..offset_wt + 4].copy_from_slice(&top_weights[i].to_le_bytes());
    }
    record[72..76].copy_from_slice(&span_start.to_le_bytes());
    record[76..80].copy_from_slice(&span_end.to_le_bytes());
    record[80..84].copy_from_slice(&byte_start.to_le_bytes());
    record[84..88].copy_from_slice(&byte_end.to_le_bytes());
    record
}

/// Generate (or extend, resumably) the teacher-labeled corpus. Whole-story
/// chunking keeps the stream deterministic under any budget chunking.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate(oracle: &mut dyn TeacherOracle, budget_s: u64, target: usize) {
    let (mp, rp) = corpus_paths();
    generate_to(oracle, budget_s, target, mp, rp);
}

/// Generate a resumable teacher corpus at explicit paths.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_to(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    mp: &str,
    rp: &str,
) {
    generate_to_with_token_byte_lengths(oracle, budget_s, target, mp, rp, None);
}

/// Generate a resumable teacher corpus with optional per-token byte lengths
/// used to populate tokenizer-neutral byte anchors.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate_to_with_token_byte_lengths(
    oracle: &mut dyn TeacherOracle,
    budget_s: u64,
    target: usize,
    mp: &str,
    rp: &str,
    token_byte_lengths: Option<&[u32]>,
) {
    let (mut n, mut stories, mut rng, mut done) = match std::fs::read(mp) {
        Ok(b) if b.len() == 25 => (
            u64::from_le_bytes(b[0..8].try_into().unwrap()),
            u64::from_le_bytes(b[8..16].try_into().unwrap()),
            u64::from_le_bytes(b[16..24].try_into().unwrap()),
            b[24],
        ),
        _ => (0, 0, 0x5EED, 0),
    };
    if (n as usize) < target {
        done = 0;
    }
    if done == 1 {
        println!("corpus already complete: {} tokens", n);
        return;
    }
    let starting_tokens = n;
    let vocab = oracle.vocab();
    let seq_len = oracle.seq_len();
    let default_record_size = 48usize;
    let existing_records = std::fs::read(rp).unwrap_or_default();
    let existing_len = existing_records.len();
    let record_size = if existing_len == 0 {
        default_record_size
    } else if n != 0 && existing_len == (n as usize) * 48 {
        48usize
    } else if n != 0 && existing_len == (n as usize) * 32 {
        32usize
    } else if n != 0 && existing_len == (n as usize) * 12 {
        12usize
    } else if token_byte_lengths.is_some() || (n != 0 && existing_len >= (n as usize) * 48) {
        // The current Hugging Face path writes v3 records. Metadata is
        // committed only at story boundaries, so an interrupted run may
        // leave extra complete records or a partial record after n.
        48usize
    } else if n != 0 && existing_len >= (n as usize) * 32 {
        32usize
    } else if n != 0 && existing_len >= (n as usize) * 12 {
        12usize
    } else {
        eprintln!(
            "corpus records cannot be resumed: file length {existing_len} is shorter than the {n}-record metadata prefix"
        );
        return;
    };
    let committed_len = (n as usize).saturating_mul(record_size);
    if existing_len < committed_len {
        eprintln!(
            "corpus records cannot be resumed: file length {existing_len} is shorter than the committed {committed_len}-byte prefix"
        );
        return;
    }
    if existing_len > committed_len {
        eprintln!(
            "repairing corpus records: truncating {existing_len} bytes to the committed {committed_len}-byte prefix"
        );
        let Ok(file) = std::fs::OpenOptions::new().write(true).open(rp) else {
            eprintln!("cannot repair corpus record file {rp}: open failed");
            return;
        };
        if let Err(error) = file.set_len(committed_len as u64) {
            eprintln!("cannot truncate corpus record file {rp}: {error}");
            return;
        }
    }
    let mut logits = vec![0f32; vocab];
    let mut progress = uor_r4_model_source::progress::Progress::new("teacher corpus", target);
    progress.set(n as usize);
    let mut recs = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rp)
        .unwrap();

    let mut hidden_path = String::from(rp);
    if hidden_path.ends_with("_recs.bin") {
        hidden_path = hidden_path.replace("_recs.bin", "_hidden.bin");
    } else {
        hidden_path.push_str(".hidden");
    }
    let mut hidden_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hidden_path)
        .unwrap();

    let t0 = std::time::Instant::now();
    while done == 0 && t0.elapsed().as_secs() < budget_s {
        oracle.reset();
        let mut token = oracle.bos_token();
        let mut story_byte_offset = 0u32;
        for pos in 0..seq_len {
            progress.set(n as usize);
            oracle.step(token, pos, &mut logits);
            let (next, top_tokens_idx, top_weights_val) =
                softmax_top3_sample(&mut logits, &mut rng);

            let span_start = pos as u32;
            let span_end = span_start.saturating_add(1);
            let (byte_start, byte_end) = byte_anchors(token_byte_lengths, story_byte_offset, next);
            let record = encode_v3_record(
                stories as u32,
                next as u32,
                &top_tokens_idx,
                &top_weights_val,
                (span_start, span_end),
                (byte_start, byte_end),
            );

            recs.write_all(&record[..record_size]).unwrap();

            if let Some(h) = oracle.hidden_state() {
                let bytes =
                    unsafe { std::slice::from_raw_parts(h.as_ptr() as *const u8, h.len() * 4) };
                hidden_file.write_all(bytes).unwrap();
            }

            if token_byte_lengths.is_some() {
                story_byte_offset = byte_end;
            }
            n += 1;
            progress.set(n as usize);
            if n as usize >= target {
                done = 1;
                break;
            }
            if next == oracle.eos_token() {
                break;
            }
            token = next;
        }
        stories += 1;
        let mut b = [0u8; 25];
        b[0..8].copy_from_slice(&n.to_le_bytes());
        b[8..16].copy_from_slice(&stories.to_le_bytes());
        b[16..24].copy_from_slice(&rng.to_le_bytes());
        b[24] = done;
        std::fs::write(mp, b).unwrap();
    }
    println!(
        "corpus: {} / {} tokens, {} stories, done={}",
        n, target, stories, done
    );
    let elapsed = t0.elapsed().as_secs_f64();
    let generated = n.saturating_sub(starting_tokens);
    if generated != 0 && elapsed > 0.0 {
        println!(
            "teacher throughput: {generated} tokens in {elapsed:.2}s ({:.1} tokens/s)",
            generated as f64 / elapsed
        );
    }
    if done == 1 {
        progress.finish();
    }
}

// ------------------------------------------------------------- artifacts --

/// Rotation offsets per window slot, derived once from the definition
/// (j·17 mod D). A compiled table: derivation may multiply, the runtime
/// reads it.
pub fn derive_rotations() -> [usize; WINDOW + 1] {
    let mut r = [0usize; WINDOW + 1];
    for (j, slot) in r.iter_mut().enumerate() {
        *slot = (j * ROT) % D;
    }
    r
}

/// Signature geometry, computed compiler-side so the runtime source
/// carries no const arithmetic either.
pub const SIG_BYTES: usize = D / 8;
pub const SIG_WORDS: usize = SIG_BYTES.div_ceil(8);

/// The train/held-out story cut (80/20), computed compiler-side.
pub fn train_cut(c: &Corpus) -> u32 {
    ((c.stories as f64 * 0.8) as u32).max(1)
}

pub const ART_PATH: &str = "/tmp/tless_artifacts.bin";

#[derive(Clone)]
pub struct Compiled {
    /// COMPRESSED token representation: STAGES code bytes per token [V × STAGES]
    /// plus i8 stage books [STAGES × (K × D)]. The runtime decodes rows on
    /// demand by table reads and adds; the expanded form is never shipped.
    pub token_codes: Vec<u8>,
    pub stage_books: Vec<Vec<i8>>,
    /// per-stage fixed-point shifts: stage k's book holds round(x·2^e_k)
    /// and decode recombines as (b << shifts[k]) with shifts[k] = E − e_k,
    /// E = max e — shift-aligned fixed point, no multiplies at decode.
    pub stage_shifts: Vec<u8>,
    /// per-dimension sign thresholds for the context bundle: [D].
    pub thresholds: Vec<i64>,
    /// binarized context-class signatures per stage: STAGES × [K × D/8 bytes].
    pub class_sigs: Vec<Vec<u8>>,
    /// f32 context centroids — CERTIFIER-side only (binarization ablation);
    /// populated in-memory during compilation and never serialized to artifact
    /// files. Empty when a Compiled is loaded from a stored container.
    pub ctx_cb: Vec<Vec<f32>>,
    /// κ-labels of the f32 token-codebook stages, recorded at compile time
    /// (the stages themselves quantize into `stage_books`; the labels are
    /// witness metadata and are not serialized into the container).
    pub token_stage_kappas: Vec<String>,
    /// Shift-add dot-assignment tables (issue #243 Phase B): per stage,
    /// K × D packed u16 entries encoding each context-centroid value as
    /// a signed power-of-two sum (`DOT_TERMS` terms; one since the
    /// Phase C decision, two-term slots still decode), so the runtime scores
    /// `dot(work, centroid)` with shifts and integer adds only. Filled
    /// at compile time from `ctx_cb`; serialized only by the TLA6
    /// container (see `artifact_bytes`); empty on TLA3/4/5 loads, which
    /// keeps loaded legacy artifacts on the sign-Hamming path.
    pub dot_cb: Vec<Vec<u16>>,
    /// Integer residual centroid copies (issue #318 Phase B): per stage,
    /// K × D i8 values — the unit-scale f32 context centroids quantized
    /// with a per-stage power-of-two scale (the token stage-book
    /// precedent: max-abs → IEEE-exponent scale, libm-free). After each
    /// stage's dot argmax the runtime subtracts the winning centroid's
    /// copy, decoded as `copy << resid_scale_shifts[st]`, from the
    /// norm-folded work vector — add/sub and shifts only (contract §2).
    /// Filled at compile time from `ctx_cb`; serialized only by the TLA7
    /// container (see `artifact_bytes`); empty on pre-TLA7 loads, which
    /// keeps those artifacts on the non-residual dot path.
    pub resid_cb: Vec<Vec<i8>>,
    /// Per-stage decode shift for `resid_cb` (STAGES entries when
    /// `resid_cb` is populated): `RESID_WORK_FRACTION − e_st`, where e_st
    /// is the stage's IEEE-exponent quantization scale, so the shifted
    /// copy lands at the norm-folded work scale.
    pub resid_scale_shifts: Vec<u8>,
    /// Norm-fold CONST (issue #318 Phase B): the train-split median of
    /// round(log2(L1/L2)) over the deterministic stride-8 subsample — the
    /// certifier's Phase A.5 derivation — PLUS `RESID_WORK_FRACTION`, the
    /// fixed-point fraction the integer kernel folds to (the f32
    /// emulation folds to unit scale; the integer kernel must fold to a
    /// large-magnitude scale or every value truncates to zero). The
    /// kernel computes s = bit_length(Σ|w_d|) − norm_fold_const and
    /// shifts work right by s (left when s < 0). 0 when unset
    /// (pre-TLA7 loads), which keeps the residual path inactive.
    pub norm_fold_const: i32,
}

/// Fixed-point fraction of the norm-folded work vector (issue #318 Phase
/// B): after the po2 norm fold, unit scale (the f32 emulation's
/// normalized work) sits at 2^RESID_WORK_FRACTION. 20 bits keeps a
/// unit-magnitude dimension at ~1e6 — the raw work-vector scale the
/// shift-add dot tables already operate at — with i64 headroom for the
/// 288-dim dot reduction.
pub const RESID_WORK_FRACTION: i32 = 20;

/// Number of power-of-two terms emitted per dot-table entry.
///
/// Phase B shipped two-term packing and recorded a ⚑ 13× per-token
/// shift/add budget on the dot scan (PR #291); one-term tables measured
/// equal accuracy in f32 emulation on that record. The Phase C
/// maintainer decision (issue #243, 2026-07-29) adopts ONE term as the
/// default emission — halving the counted shift/add budget — decided
/// before the κ re-pin so the term count is pinned once. The u16
/// container slot is unchanged (second byte zero); the decode side
/// (`runtime::dot_term_apply` and the kernel scan) skips zero-flag
/// terms, so legacy two-term artifacts keep decoding bit-exactly.
pub const DOT_TERMS: usize = 1;

/// Pack one f32 centroid value as up to `DOT_TERMS` power-of-two terms
/// in a u16 (issue #243 Phase B; term count pinned by the Phase C
/// decision above). Per byte term: bit7 = negative sign, bit6 =
/// nonzero flag, bits 5..0 = biased exponent (e + 32, e ∈ [-32, 31]).
/// High byte is the first (larger) term. The decode side lives in
/// `runtime::dot_term_apply`, integer shifts only.
pub fn pack_dot_entry(value: f32) -> u16 {
    let mut packed = [0u8; 2];
    let mut rem = value;
    for slot in packed.iter_mut().take(DOT_TERMS) {
        if rem == 0.0 || !rem.is_finite() {
            break;
        }
        let e = canonical_log2f(rem.abs()).round().clamp(-32.0, 31.0);
        let term = rem.signum() * canonical_exp2f(e);
        *slot = 0x40 | ((e as i32 + 32) as u8 & 0x3F) | if rem < 0.0 { 0x80 } else { 0 };
        rem -= term;
    }
    u16::from_le_bytes([packed[1], packed[0]])
}

/// Quantize the full per-stage context codebooks into dot tables.
pub fn quantize_dot_tables(ctx_cb: &[Vec<f32>]) -> Vec<Vec<u16>> {
    ctx_cb
        .iter()
        .map(|cb| cb.iter().map(|&c| pack_dot_entry(c)).collect())
        .collect()
}

/// Quantize the per-stage context codebooks into i8 residual centroid
/// copies (issue #318 Phase B; the Phase A.5 cpy8 row's kernel form):
/// per stage, max-abs → power-of-two scale from the IEEE-754 exponent of
/// the ratio (the token stage-book precedent, libm-free), values
/// round(x·2^e) clamped to i8. The returned per-stage decode shift is
/// `RESID_WORK_FRACTION − e` (clamped at 0 — e beyond the fraction is a
/// pathologically flat codebook, |x| < 2^-13, and the copies carry no
/// representable signal anyway), so `copy << shift` lands at the
/// norm-folded work scale. Deterministic: pure functions of `ctx_cb`.
pub fn quantize_resid_copies(ctx_cb: &[Vec<f32>]) -> (Vec<Vec<i8>>, Vec<u8>) {
    let mut copies = Vec::with_capacity(ctx_cb.len());
    let mut shifts = Vec::with_capacity(ctx_cb.len());
    for cb in ctx_cb {
        let m = cb.iter().fold(0f32, |a, &x| a.max(x.abs())).max(1e-9);
        let r = 127.0 / m;
        let e = ((r.to_bits() >> 23) as i32) - 127;
        let scale = (2.0f32).powi(e);
        copies.push(
            cb.iter()
                .map(|&x| (x * scale).round().clamp(-127.0, 127.0) as i8)
                .collect(),
        );
        shifts.push((RESID_WORK_FRACTION - e).clamp(0, 63) as u8);
    }
    (copies, shifts)
}

/// The norm-fold CONST (issue #318 Phase B): mirror of the certifier's
/// Phase A.5 `ratios`/`norm_const` derivation — the train-split median of
/// log2(L1/L2) over the deterministic stride-8 subsample, rounded — plus
/// `RESID_WORK_FRACTION` (the integer kernel's fixed-point fold target;
/// see the field doc). Deterministic: ordered corpus scan, no RNG.
fn norm_fold_const(art: &Compiled, rot: &[usize; WINDOW + 1], corpus: &Corpus, cut: u32) -> i32 {
    let mut ratios: Vec<f32> = (0..corpus.n)
        .step_by(8)
        .filter(|&i| corpus.story[i] < cut)
        .map(|i| {
            let b = super::runtime::bundle_plain(art, rot, corpus, i);
            let mut l1 = 0f32;
            let mut l2sq = 0f32;
            for (&bv, &t) in b.iter().zip(art.thresholds.iter()) {
                let w = (bv - t) as f32;
                l1 += w.abs();
                l2sq += w * w;
            }
            canonical_log2f(l1 / canonical_sqrtf(l2sq).max(1e-9))
        })
        .collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let Some(&median) = ratios.get(ratios.len() / 2) else {
        // Degenerate corpus (no train positions in the stride-8
        // subsample): fold to the fixed-point scale only.
        return RESID_WORK_FRACTION;
    };
    median.round() as i32 + RESID_WORK_FRACTION
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegionHammingCalibration {
    pub stage: u8,
    pub class: u16,
    pub mask_bits: u16,
    pub sample_count: u32,
    pub acceptance_radius: u16,
    pub hamming_histogram: Vec<u32>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HammingCalibrationReport {
    pub signature_bits: u16,
    pub quantile_numerator: u16,
    pub quantile_denominator: u16,
    pub regions: Vec<RegionHammingCalibration>,
}

/// 95th-percentile-style quantile of a distance histogram, shared by the
/// class-signature calibration above and the cover region calibration
/// (`super::cover`): the smallest distance whose cumulative count reaches
/// `ceil(total · numerator / denominator)`.
pub fn quantile_radius(histogram: &[u32], numerator: u32, denominator: u32) -> u16 {
    let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();
    if total == 0 {
        return 0;
    }
    if denominator == 0 {
        return histogram.len().saturating_sub(1) as u16;
    }
    let numerator = u64::from(numerator);
    let denominator = u64::from(denominator);
    let target = total
        .saturating_mul(numerator)
        .saturating_add(denominator.saturating_sub(1))
        / denominator;
    let mut cumulative = 0u64;
    for (distance, &count) in histogram.iter().enumerate() {
        cumulative = cumulative.saturating_add(u64::from(count));
        if cumulative >= target {
            return distance as u16;
        }
    }
    histogram.len().saturating_sub(1) as u16
}

fn signature_histograms(
    class_sigs: &[Vec<u8>],
    signatures: &[[u8; SIG_BYTES]],
) -> Vec<Vec<Vec<u32>>> {
    let mut histograms = vec![vec![vec![0u32; D + 1]; K]; STAGES];
    for sig in signatures {
        let mut words = [0u64; SIG_WORDS];
        for (word, chunk) in words.iter_mut().zip(sig.chunks(8)) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            *word = u64::from_le_bytes(bytes);
        }
        for (stage, stage_histograms) in histograms.iter_mut().enumerate().take(STAGES) {
            let Some(stage_sigs) = class_sigs.get(stage) else {
                continue;
            };
            if stage_sigs.len() < K * SIG_BYTES {
                continue;
            }
            let mut best_dist = u32::MAX;
            let mut best_class = 0usize;
            for (class, class_sig) in stage_sigs.chunks_exact(SIG_BYTES).enumerate().take(K) {
                let mut dist = 0u32;
                for (&word, chunk) in words.iter().zip(class_sig.chunks(8)) {
                    let mut bytes = [0u8; 8];
                    bytes[..chunk.len()].copy_from_slice(chunk);
                    dist += (word ^ u64::from_le_bytes(bytes)).count_ones();
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_class = class;
                }
            }
            if best_dist <= D as u32 {
                stage_histograms[best_class][best_dist as usize] += 1;
            }
        }
    }
    histograms
}

fn calibration_from_histograms(histograms: Vec<Vec<Vec<u32>>>) -> HammingCalibrationReport {
    const NUMERATOR: u32 = 95;
    const DENOMINATOR: u32 = 100;
    let mut regions = Vec::with_capacity(STAGES * K);
    for (stage, stage_histograms) in histograms.iter().enumerate() {
        for (class, histogram) in stage_histograms.iter().enumerate() {
            let sample_count = histogram.iter().sum();
            regions.push(RegionHammingCalibration {
                stage: stage as u8,
                class: class as u16,
                mask_bits: D as u16,
                sample_count,
                acceptance_radius: quantile_radius(histogram, NUMERATOR, DENOMINATOR),
                hamming_histogram: histogram.clone(),
            });
        }
    }
    HammingCalibrationReport {
        signature_bits: D as u16,
        quantile_numerator: NUMERATOR as u16,
        quantile_denominator: DENOMINATOR as u16,
        regions,
    }
}

/// Deterministically calibrate the Hamming regions. Each worker owns a
/// private integer histogram, and the ordered reduction makes the result
/// independent of worker count while removing the serial class-distance scan
/// from the recorded-corpus compile path.
pub fn calibrate_hamming_regions_from_signatures(
    class_sigs: &[Vec<u8>],
    signatures: &[[u8; SIG_BYTES]],
) -> HammingCalibrationReport {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let threads = std::thread::available_parallelism()
            .map(|count| count.get().min(8))
            .unwrap_or(1);
        if threads > 1 && signatures.len() >= threads * 2 {
            let chunk_size = signatures.len().div_ceil(threads);
            let mut partials = Vec::new();
            std::thread::scope(|scope| {
                let mut handles = Vec::new();
                for chunk in signatures.chunks(chunk_size) {
                    handles.push(scope.spawn(move || signature_histograms(class_sigs, chunk)));
                }
                for handle in handles {
                    partials.push(handle.join().expect("hamming calibration worker"));
                }
            });
            let mut merged = vec![vec![vec![0u32; D + 1]; K]; STAGES];
            for partial in partials {
                for stage in 0..STAGES {
                    for class in 0..K {
                        for distance in 0..=D {
                            merged[stage][class][distance] = merged[stage][class][distance]
                                .saturating_add(partial[stage][class][distance]);
                        }
                    }
                }
            }
            return calibration_from_histograms(merged);
        }
    }
    calibration_from_histograms(signature_histograms(class_sigs, signatures))
}

pub fn calibrate_hamming_regions(art: &Compiled, corpus: &Corpus) -> HammingCalibrationReport {
    let cut = train_cut(corpus);
    let held_out: Vec<usize> = (0..corpus.n).filter(|&i| corpus.story[i] >= cut).collect();
    let indices: Vec<usize> = if held_out.is_empty() {
        (0..corpus.n).collect()
    } else {
        held_out
    };
    let rot = derive_rotations();
    let mut signatures = Vec::with_capacity(indices.len());
    for i in indices {
        let bundle = super::runtime::bundle_plain(art, &rot, corpus, i);
        signatures.push(super::runtime::sig_plain(art, &bundle));
    }
    calibrate_hamming_regions_from_signatures(&art.class_sigs, &signatures)
}

fn hash_index(seed: &[u8; 32], dim: usize, key: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(seed);
    hasher.update(&dim.to_le_bytes());
    hasher.update(key.as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}

fn deterministic_bucket(seed: &[u8; 32], h: usize, target_dim: usize) -> usize {
    (hash_index(seed, h, "bucket") as usize) % target_dim
}

fn deterministic_sign(seed: &[u8; 32], h: usize) -> f32 {
    if hash_index(seed, h, "sign").is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

pub fn deterministic_project(
    seed_bytes: &[u8; 32],
    vocab: usize,
    source_dim: usize,
    target_dim: usize,
    source: &dyn RepresentationSource,
) -> Vec<f32> {
    let mut projected_vecs = vec![0f32; vocab * target_dim];
    let chunk_size = 1000;
    let mut raw_chunk = vec![0f32; chunk_size * source_dim];

    let mut sum_vec = vec![0f64; source_dim];
    for chunk_start in (0..vocab).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(vocab);
        let count = chunk_end - chunk_start;
        source
            .read_embedding_rows(chunk_start..chunk_end, &mut raw_chunk[..count * source_dim])
            .unwrap();
        for i in 0..count {
            for j in 0..source_dim {
                sum_vec[j] += raw_chunk[i * source_dim + j] as f64;
            }
        }
    }
    let mut mean = vec![0f32; source_dim];
    for j in 0..source_dim {
        mean[j] = (sum_vec[j] / vocab as f64) as f32;
    }

    let mut progress = uor_r4_model_source::progress::Progress::new("projecting embeddings", vocab);
    for chunk_start in (0..vocab).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(vocab);
        let count = chunk_end - chunk_start;
        source
            .read_embedding_rows(chunk_start..chunk_end, &mut raw_chunk[..count * source_dim])
            .unwrap();

        for i in 0..count {
            let t = chunk_start + i;
            progress.set(t);
            let mut projected_row = vec![0f32; target_dim];

            for h in 0..source_dim {
                let centered_val = raw_chunk[i * source_dim + h] - mean[h];
                let target = deterministic_bucket(seed_bytes, h, target_dim);
                let sign = deterministic_sign(seed_bytes, h);
                projected_row[target] += sign * centered_val;
            }

            let n = canonical_sqrtf(projected_row.iter().map(|x| x * x).sum::<f32>()).max(1e-9);
            for x in projected_row.iter_mut() {
                *x /= n;
            }

            projected_vecs[t * target_dim..(t + 1) * target_dim].copy_from_slice(&projected_row);
        }
    }
    progress.finish();
    projected_vecs
}

fn sampled_kmeans_rvq(
    vecs: &[f32],
    nvec: usize,
    stages: usize,
    k: usize,
    iters: usize,
    seed_bytes: &[u8; 32],
) -> (Vec<Vec<f32>>, Vec<u8>) {
    let mut rng_seed = 0xDECAFBADu64;
    for &b in seed_bytes.iter().take(8) {
        rng_seed = rng_seed.wrapping_add(b as u64).rotate_left(8);
    }

    let mut indices: Vec<usize> = (0..nvec).collect();
    for i in (1..nvec).rev() {
        let j = (xorshift(&mut rng_seed) as usize) % (i + 1);
        indices.swap(i, j);
    }

    let sample_size = nvec.min(10000);
    let mut sample_vecs = vec![0f32; sample_size * D];
    for i in 0..sample_size {
        let src_idx = indices[i];
        sample_vecs[i * D..(i + 1) * D].copy_from_slice(&vecs[src_idx * D..(src_idx + 1) * D]);
    }

    let mut residual = vecs.to_vec();
    let mut sample_residual = sample_vecs.to_vec();
    let mut codebooks: Vec<Vec<f32>> = Vec::new();
    let mut codes = vec![0u8; nvec * stages];

    for stage in 0..stages {
        eprintln!("sampled RVQ stage {}/{}", stage + 1, stages);
        let mut progress =
            uor_r4_model_source::progress::Progress::new("RVQ assignment", iters * sample_size);
        let mut cent = vec![0f32; k * D];
        let mut cent_seed = 0xC0DEB00C ^ stage as u64 ^ rng_seed;
        let mut used = vec![false; sample_size];
        for kk in 0..k {
            let mut idx = (xorshift(&mut cent_seed) as usize) % sample_size;
            while used[idx] {
                idx = (idx + 1) % sample_size;
            }
            used[idx] = true;
            cent[kk * D..(kk + 1) * D].copy_from_slice(&sample_residual[idx * D..(idx + 1) * D]);
        }
        let mut assign = vec![0usize; sample_size];
        for iteration in 0..iters {
            for v in 0..sample_size {
                progress.set(iteration * sample_size + v);
                let rv = &sample_residual[v * D..(v + 1) * D];
                let (mut bd, mut bk) = (f32::MAX, 0usize);
                for kk in 0..k {
                    let c = &cent[kk * D..(kk + 1) * D];
                    let mut d2 = 0f32;
                    for j in 0..D {
                        let t = rv[j] - c[j];
                        d2 += t * t;
                    }
                    if d2 < bd {
                        bd = d2;
                        bk = kk;
                    }
                }
                assign[v] = bk;
            }
            let mut sum = vec![0f32; k * D];
            let mut cnt = vec![0u32; k];
            for v in 0..sample_size {
                cnt[assign[v]] += 1;
                for j in 0..D {
                    sum[assign[v] * D + j] += sample_residual[v * D + j];
                }
            }
            for kk in 0..k {
                if cnt[kk] > 0 {
                    for j in 0..D {
                        cent[kk * D + j] = sum[kk * D + j] / cnt[kk] as f32;
                    }
                }
            }
        }
        progress.finish();

        for v in 0..nvec {
            let rv = &residual[v * D..(v + 1) * D];
            let (mut bd, mut bk) = (f32::MAX, 0usize);
            for kk in 0..k {
                let c = &cent[kk * D..(kk + 1) * D];
                let mut d2 = 0f32;
                for j in 0..D {
                    let t = rv[j] - c[j];
                    d2 += t * t;
                }
                if d2 < bd {
                    bd = d2;
                    bk = kk;
                }
            }
            codes[v * stages + stage] = bk as u8;
            for j in 0..D {
                residual[v * D + j] -= cent[bk * D + j];
            }
        }

        for v in 0..sample_size {
            let src_idx = indices[v];
            for j in 0..D {
                sample_residual[v * D + j] = residual[src_idx * D + j];
            }
        }

        codebooks.push(cent);
    }
    (codebooks, codes)
}

pub fn kappa_of_f32s(v: &[f32]) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in v {
        hasher.update(&value.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub fn compile(oracle: &dyn TeacherOracle, corpus: &Corpus) -> Compiled {
    let source_kappa = oracle.kappa();
    let tokenizer_address = oracle.tokenizer_address().to_owned();
    compile_from_representation(
        oracle,
        &source_kappa,
        oracle.source_bytes(),
        &tokenizer_address,
        corpus,
    )
}

/// Compile an artifact from a representation source and explicit provenance.
///
/// Keeping this surface independent from [`TeacherOracle`] is what allows a
/// recorded-corpus build to avoid constructing a transformer at all. The
/// legacy [`compile`] wrapper remains for source-capture and κ reproduction.
pub fn compile_from_representation(
    source: &dyn RepresentationSource,
    source_kappa: &str,
    source_bytes: usize,
    tokenizer_address: &str,
    corpus: &Corpus,
) -> Compiled {
    let vocab = source.vocab_size();
    assert!(
        vocab <= u32::MAX as usize,
        "source vocabulary exceeds u32 token ids"
    );
    println!("source κ: {} ({} bytes)", source_kappa, source_bytes);

    let seed_string = format!(
        "{}{}{}",
        source_kappa, tokenizer_address, "r4-geometric-projection-v1"
    );
    let seed_hash = blake3::hash(seed_string.as_bytes());
    let seed_bytes = seed_hash.as_bytes();

    let source_dim = source.source_dimension();
    let vecs = deterministic_project(seed_bytes, vocab, source_dim, D, source);

    let (emb_cb, emb_codes) = sampled_kmeans_rvq(&vecs, vocab, STAGES, K, EMB_ITERS, seed_bytes);
    let emb_stage_kappas: Vec<String> = emb_cb.iter().map(|cb| kappa_of_f32s(cb)).collect();
    for (i, k) in emb_stage_kappas.iter().enumerate() {
        println!("token codebook stage {} κ: {}", i, k);
    }
    let mut stage_books: Vec<Vec<i8>> = Vec::new();
    let mut exps: Vec<i32> = Vec::new();
    for cb in &emb_cb {
        let m = cb.iter().fold(0f32, |a, &x| a.max(x.abs())).max(1e-9);
        // Platform-stable exponent: floor(log2(r)) for a positive normal r is
        // exactly its IEEE-754 unbiased exponent (r = 127.0/m ≥ 1 here is a
        // correctly-rounded division, so to_bits is identical everywhere).
        // The previous (127.0/m).log2().floor() was the token path's only
        // libm-sensitive op: a 1-ulp log2 error near a power-of-two boundary
        // flips the floor and changes the stage book (D2 hardening).
        let r = 127.0 / m;
        let e = ((r.to_bits() >> 23) as i32) - 127;
        let scale = (2.0f32).powi(e);
        stage_books.push(
            cb.iter()
                .map(|&x| (x * scale).round().clamp(-127.0, 127.0) as i8)
                .collect(),
        );
        exps.push(e);
    }
    let emax = *exps.iter().max().unwrap();
    let stage_shifts: Vec<u8> = exps.iter().map(|&e| (emax - e) as u8).collect();
    println!(
        "stage fixed-point exponents e_k = {:?}, decode shifts = {:?}",
        exps, stage_shifts
    );
    let token_codes = emb_codes; // [V × STAGES], compressed representation
    for (i, book) in stage_books.iter().enumerate() {
        let bytes: Vec<u8> = book.iter().map(|&b| b as u8).collect();
        println!(
            "stage book {} (i8) κ: blake3:{}",
            i,
            blake3::hash(&bytes).to_hex()
        );
    }
    println!(
        "token codes κ: blake3:{}",
        blake3::hash(&token_codes).to_hex()
    );

    // Partial artifact so the RUNTIME's own bundle path produces the
    // training bundles (store/query identity by construction).
    let mut art = Compiled {
        token_codes,
        stage_books,
        stage_shifts,
        thresholds: Vec::new(),
        class_sigs: Vec::new(),
        ctx_cb: Vec::new(),
        token_stage_kappas: emb_stage_kappas,
        dot_cb: Vec::new(),
        resid_cb: Vec::new(),
        resid_scale_shifts: Vec::new(),
        norm_fold_const: 0,
    };
    let rot = derive_rotations();

    // 3: thresholds = per-dimension train means of the runtime bundle.
    let cut = train_cut(corpus);
    let mut sums = [0i128; D];
    let mut ntrain = 0i128;
    let mut threshold_progress =
        uor_r4_model_source::progress::Progress::new("context thresholds", corpus.n);
    for i in 0..corpus.n {
        threshold_progress.set(i);
        if corpus.story[i] >= cut {
            continue;
        }
        let b = super::runtime::bundle_plain(&art, &rot, corpus, i);
        for (s, &v) in sums.iter_mut().zip(b.iter()) {
            *s += v as i128;
        }
        ntrain += 1;
    }
    threshold_progress.finish();
    art.thresholds = sums.iter().map(|&s| (s / ntrain) as i64).collect();
    {
        let bytes: Vec<u8> = art
            .thresholds
            .iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();
        println!(
            "threshold vector κ: blake3:{}",
            blake3::hash(&bytes).to_hex()
        );
    }

    // 4: context RVQ on (centered, normalized) runtime bundles; binarize
    // centroids to sign bits for Hamming assignment.
    let train_idx: Vec<usize> = (0..corpus.n).filter(|&i| corpus.story[i] < cut).collect();
    let mut s = 0x5A3B1Eu64;
    let mut samp = vec![0f32; CTX_SAMPLE * D];
    let mut context_progress =
        uor_r4_model_source::progress::Progress::new("context samples", CTX_SAMPLE);
    for v in 0..CTX_SAMPLE {
        context_progress.set(v);
        let i = train_idx[(xorshift(&mut s) as usize) % train_idx.len()];
        let b = super::runtime::bundle_plain(&art, &rot, corpus, i);
        let mut row = [0f32; D];
        let mut nn = 0f32;
        for d in 0..D {
            let x = (b[d] - art.thresholds[d]) as f32;
            row[d] = x;
            nn += x * x;
        }
        let nn = canonical_sqrtf(nn).max(1e-9);
        for d in 0..D {
            samp[v * D + d] = row[d] / nn;
        }
    }
    context_progress.finish();
    let (ctx_cb, _) = sampled_kmeans_rvq(&samp, CTX_SAMPLE, STAGES, K, CTX_ITERS, seed_bytes);
    for (st, cb) in ctx_cb.iter().enumerate() {
        println!("context codebook stage {} κ: {}", st, kappa_of_f32s(cb));
        let mut sigs = vec![0u8; K * D / 8];
        for kk in 0..K {
            for d in 0..D {
                if cb[kk * D + d] > 0.0 {
                    sigs[kk * D / 8 + d / 8] |= 1 << (d % 8);
                }
            }
        }
        println!(
            "class signature stage {} κ: blake3:{}",
            st,
            blake3::hash(&sigs).to_hex()
        );
        art.class_sigs.push(sigs);
    }
    art.ctx_cb = ctx_cb;
    // #243 Phase B: the shift-add dot tables are derived from the same
    // centroids, deterministically, at compile time. In-memory always;
    // on disk only under TLA6 (era note in `artifact_bytes`).
    art.dot_cb = quantize_dot_tables(&art.ctx_cb);
    // #318 Phase B: the integer residual wiring — per-stage i8 centroid
    // copies and the norm-fold CONST, derived from the same centroids and
    // the train-split corpus, deterministically (ordered stride-8 scan,
    // no RNG). In-memory always; on disk only under TLA7 (era note in
    // `artifact_bytes`).
    let (resid_cb, resid_scale_shifts) = quantize_resid_copies(&art.ctx_cb);
    art.resid_cb = resid_cb;
    art.resid_scale_shifts = resid_scale_shifts;
    art.norm_fold_const = norm_fold_const(&art, &rot, corpus, cut);
    println!(
        "norm-fold CONST (incl. fixed-point fraction): {}",
        art.norm_fold_const
    );
    art
}

// ------------------------------------------------------- persistence --

/// Flat, versioned serialization of the compiled artifacts (κ-pins are
/// re-derivable from the bytes; the store is rebuilt from the corpus).
///
/// Format TLA5: always includes a u32 vocab prefix; ctx_cb is not serialized.
///
/// Format TLA6 (issue #243 Phase B) = the TLA5 payload + the packed
/// shift-add dot tables (STAGES × K × D u16, little-endian).
///
/// Format TLA7 (issue #318 Phase B) = the TLA6 payload + the integer
/// residual sections: the norm-fold CONST (i32, little-endian), the
/// per-stage copy decode shifts (STAGES × u8), and the per-stage i8
/// centroid copies (STAGES × K × D i8).
///
/// TLA6 is the DEFAULT emission since the Phase C adoption decision
/// (issue #243, maintainer decision 2026-07-29): fresh compiles emit
/// TLA6 with 1-term dot tables (`DOT_TERMS`), and the κ re-pin that
/// this implies is recorded as an era note in the migration baseline
/// (tests/fixtures/baseline_kappa.json) and docs/transformerless —
/// TLA5-era pins remain valid addresses for TLA5-era bytes.
/// `R4_TLESS_TLA6=0` opts a compile back out to TLA5 emission for
/// era-comparison runs. Loaded TLA5 artifacts keep the sign-Hamming
/// query path; loaded TLA6 artifacts (and fresh in-memory compiles)
/// take the shift-add dot path.
///
/// TLA7 is the DEFAULT emission for artifacts that carry residual
/// copies (fresh compiles since issue #318 Phase B); `R4_TLESS_TLA7=0`
/// opts a compile back out to TLA6 emission for era-comparison runs.
/// Loaded pre-TLA7 artifacts keep the non-residual dot path; loaded TLA7
/// artifacts (and fresh in-memory compiles) take the residual-wired dot
/// path. The κ re-pin this implies is a Phase C maintainer decision
/// (docs/dot_residual_phase_b_design.md) — the pinned baseline is
/// TLA5-era and remains a valid address for TLA5-era bytes.
pub fn artifact_bytes(a: &Compiled) -> Vec<u8> {
    let emit_tla6 = !a.dot_cb.is_empty()
        && std::env::var("R4_TLESS_TLA6")
            .map(|v| v != "0")
            .unwrap_or(true);
    let emit_tla7 = emit_tla6
        && !a.resid_cb.is_empty()
        && std::env::var("R4_TLESS_TLA7")
            .map(|v| v != "0")
            .unwrap_or(true);
    let vocab = a.token_codes.len() / STAGES;
    let mut b: Vec<u8> = if emit_tla7 {
        b"TLA7".to_vec()
    } else if emit_tla6 {
        b"TLA6".to_vec()
    } else {
        b"TLA5".to_vec()
    };
    b.extend_from_slice(&(vocab as u32).to_le_bytes());
    b.extend_from_slice(&a.stage_shifts);
    b.extend_from_slice(&a.token_codes);
    for book in &a.stage_books {
        b.extend(book.iter().map(|&x| x as u8));
    }
    for t in &a.thresholds {
        b.extend_from_slice(&t.to_le_bytes());
    }
    for s in &a.class_sigs {
        b.extend_from_slice(s);
    }
    if emit_tla6 {
        for table in &a.dot_cb {
            for entry in table {
                b.extend_from_slice(&entry.to_le_bytes());
            }
        }
    }
    if emit_tla7 {
        b.extend_from_slice(&a.norm_fold_const.to_le_bytes());
        b.extend_from_slice(&a.resid_scale_shifts);
        for copies in &a.resid_cb {
            b.extend(copies.iter().map(|&x| x as u8));
        }
    }
    b
}

/// κ-label of the artifact container bytes.
pub fn artifact_kappa(a: &Compiled) -> String {
    format!("blake3:{}", blake3::hash(&artifact_bytes(a)).to_hex())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_artifacts(a: &Compiled) {
    let b = artifact_bytes(a);
    std::fs::write(ART_PATH, &b).unwrap();
    println!(
        "artifacts saved: {} ({} bytes, κ blake3:{})",
        ART_PATH,
        b.len(),
        blake3::hash(&b).to_hex()
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_artifacts() -> Option<Compiled> {
    load_artifacts_from(ART_PATH)
}

/// Load artifacts from an explicit path (fixtures, mirrors).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_artifacts_from(path: &str) -> Option<Compiled> {
    let b = std::fs::read(path).ok()?;
    let art = parse_artifacts(&b)?;
    println!("artifact file κ blake3:{}", blake3::hash(&b).to_hex());
    Some(art)
}

/// Parse a TLA3, TLA4, TLA5, TLA6, or TLA7 container from bytes.
///
/// TLA3/TLA4 are legacy formats that include f32 ctx_cb bytes; these are
/// silently discarded on load (ctx_cb is not part of the deployed artifact).
/// TLA5 is the current default format: u32 vocab prefix, no ctx_cb.
/// TLA6 appends the packed shift-add dot tables (issue #243 Phase B).
/// TLA7 appends the integer residual sections — norm-fold CONST, per-stage
/// copy decode shifts, i8 centroid copies (issue #318 Phase B).
pub fn parse_artifacts(b: &[u8]) -> Option<Compiled> {
    let (vocab, mut o, has_ctx_cb, has_dot_cb, has_resid_cb) = match b.get(..4)? {
        b"TLA3" => (V, 4usize, true, false, false),
        b"TLA4" => (
            u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize,
            8usize,
            true,
            false,
            false,
        ),
        b"TLA5" => (
            u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize,
            8usize,
            false,
            false,
            false,
        ),
        b"TLA6" => (
            u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize,
            8usize,
            false,
            true,
            false,
        ),
        b"TLA7" => (
            u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize,
            8usize,
            false,
            true,
            true,
        ),
        _ => return None,
    };
    let tc = vocab.checked_mul(STAGES)?;
    let bk = STAGES * K * D;
    let th = D * 8;
    let cs = STAGES * K * D / 8;
    let cc = if has_ctx_cb { STAGES * K * D * 4 } else { 0 };
    let dc = if has_dot_cb { STAGES * K * D * 2 } else { 0 };
    let rc = if has_resid_cb {
        4 + STAGES + STAGES * K * D
    } else {
        0
    };
    if b.len() != o + STAGES + tc + bk + th + cs + cc + dc + rc {
        return None;
    }
    let stage_shifts = b[o..o + STAGES].to_vec();
    o += STAGES;
    let token_codes = b[o..o + tc].to_vec();
    o += tc;
    let mut stage_books = Vec::new();
    for _ in 0..STAGES {
        stage_books.push(
            b[o..o + K * D]
                .iter()
                .map(|&x| x as i8)
                .collect::<Vec<i8>>(),
        );
        o += K * D;
    }
    let mut thresholds = vec![0i64; D];
    for t in thresholds.iter_mut() {
        *t = i64::from_le_bytes(b[o..o + 8].try_into().unwrap());
        o += 8;
    }
    let mut class_sigs = Vec::new();
    for _ in 0..STAGES {
        class_sigs.push(b[o..o + K * D / 8].to_vec());
        o += K * D / 8;
    }
    // Legacy TLA3/TLA4 containers include f32 ctx_cb bytes; advance past them
    // without reading. TLA5/TLA6 containers do not include ctx_cb at all.
    if has_ctx_cb {
        o += STAGES * K * D * 4;
    }
    let mut dot_cb = Vec::new();
    if has_dot_cb {
        for _ in 0..STAGES {
            let mut table = Vec::with_capacity(K * D);
            for _ in 0..K * D {
                table.push(u16::from_le_bytes(b[o..o + 2].try_into().unwrap()));
                o += 2;
            }
            dot_cb.push(table);
        }
    }
    let mut norm_fold_const = 0i32;
    let mut resid_scale_shifts = Vec::new();
    let mut resid_cb = Vec::new();
    if has_resid_cb {
        norm_fold_const = i32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        o += 4;
        resid_scale_shifts = b[o..o + STAGES].iter().map(|&x| x.min(63)).collect();
        o += STAGES;
        for _ in 0..STAGES {
            resid_cb.push(
                b[o..o + K * D]
                    .iter()
                    .map(|&x| x as i8)
                    .collect::<Vec<i8>>(),
            );
            o += K * D;
        }
    }
    let _ = o; // all bytes consumed
    Some(Compiled {
        token_codes,
        stage_books,
        stage_shifts,
        thresholds,
        class_sigs,
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb,
        resid_cb,
        resid_scale_shifts,
        norm_fold_const,
    })
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HierarchicalCodes {
    /// BTreeMap is required here: this JSON is part of the emitted bundle,
    /// so HashMap iteration order would make canonical artifacts platform- or
    /// run-dependent even when the compiled codes are identical.
    pub token_type_prefixes: BTreeMap<String, Vec<u8>>,
    pub relational_prefixes: Vec<Vec<u32>>,
}

pub fn induce_hierarchical_codes(
    token_codes: &[u8],
    vocab: usize,
    corpus: &Corpus,
) -> HierarchicalCodes {
    let mut token_type_prefixes = BTreeMap::new();
    for token_id in 0..vocab {
        let offset = token_id * STAGES;
        if offset + STAGES <= token_codes.len() {
            let prefix = token_codes[offset..offset + STAGES].to_vec();
            token_type_prefixes.insert(token_id.to_string(), prefix);
        }
    }

    let mut transition_counts = HashMap::new();
    for i in 0..corpus.n {
        let story_id = corpus.story[i];
        let token = corpus.next[i];

        if i + 1 < corpus.n && corpus.story[i + 1] == story_id {
            let next_tok = corpus.next[i + 1];
            let pair = vec![token, next_tok];
            *transition_counts.entry(pair).or_insert(0) += 1;

            if i + 2 < corpus.n && corpus.story[i + 2] == story_id {
                let next_next_tok = corpus.next[i + 2];
                let triplet = vec![token, next_tok, next_next_tok];
                *transition_counts.entry(triplet).or_insert(0) += 1;
            }
        }
    }

    let mut frequent_paths: Vec<(Vec<u32>, usize)> = transition_counts
        .into_iter()
        .filter(|(_, count)| *count >= 5)
        .collect();

    frequent_paths.sort_by_key(|entry| std::cmp::Reverse(entry.1));

    let relational_prefixes = frequent_paths
        .into_iter()
        .take(100)
        .map(|(path, _)| path)
        .collect();

    HierarchicalCodes {
        token_type_prefixes,
        relational_prefixes,
    }
}
