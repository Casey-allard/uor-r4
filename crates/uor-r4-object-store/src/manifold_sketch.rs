//! Multiplier-Free Geometric Manifold Sketching, Hierarchical Scope Routing,
//! and Epistemic Hazard Gating over `uor-r4-object-store`.
//!
//! Ports and upgrades the hyperdimensional lifecycle router concepts into strict
//! UOR-R4 Charter compliance (`CHARTER §3.1–§3.3`, `D11`, `D23`, `D30`):
//! - **Zero floating point or squashing (`D23`):** All similarities, hierarchical
//!   blends, lucidity thresholds, and statistical significance gates ($\sigma$) are
//!   computed in exact integer bit-overlap arithmetic.
//! - **Zero multiplication instructions (`D11`, `D30`):** Hashing and seed expansion
//!   use pure ARX (`wrapping_add`, `rotate_left`, `^`) primitives (`xoroshiro128++`),
//!   and parent-child hierarchical blending uses exact shift-and-add (`(13 * local + 3 * parent) >> 4`).
//! - **Reversible `i16` Bipolar Accumulators:** Updating, overwriting, or evicting a
//!   slot subtracts its prior sketch (`weight = -1`) from its parent scope, archetype,
//!   and global accumulators in $O(1)$, preserving exact integer state.
//! - **Two-Stage Hybrid Sieve + Sketch Read:** Bridges the paraphrase/inflection gap
//!   where exact word-prime GCD sieving (`gcd_shared_primes == 0`) misses, while
//!   enforcing a hard $+3\sigma$ integer abstention gate so unstated queries still
//!   return [`AddressReadOutcome::Absent`] with zero backend fetches.

#![forbid(unsafe_code)]

use crate::{
    address_read::{
        AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
    },
    bindings::PinnedTokenizer,
    Error, Limits, RecordBackend, Result, Store,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bit dimension of a manifold sketch ($D = 4096$ bits = $64 \times 64$-bit words).
pub const SKETCH_BITS: usize = 4096;

/// Number of 64-bit words in [`Sketch4096`].
pub const SKETCH_WORDS: usize = 64;

/// Expected bit overlap of two independent random 4,096-bit sketches ($\mu = D / 2 = 2048$).
pub const EXPECTED_RANDOM_OVERLAP: i32 = (SKETCH_BITS / 2) as i32;

/// One standard deviation ($\sigma = \sqrt{4096} / 2 = 32$ bits) of random bit overlap.
pub const SIGMA_BITS: i32 = 32;

/// Hard $+3\sigma$ integer admission margin ($3 \times 32 = 96$ bits above random chance,
/// i.e., $\ge 2144$ matching bits out of $4096$).
pub const THREE_SIGMA_EXCESS_BITS: i32 = 3 * SIGMA_BITS;

/// First 16 odd primes used as cyclic bit-rotation strides across sequence steps.
pub const PRIME_STRIDES: [u32; 16] = [3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59];

/// Structural scale of an indexed manifold cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Scale {
    Repo = 0,
    File = 1,
    Block = 2,
    Span = 3,
}

/// Source scope modality (`@repo`, `@docs`, `@ref`, `@web`, `@exp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Modality {
    RepoCode = 0,
    SpecDoc = 1,
    ExtRef = 2,
    WebRef = 3,
    Experiment = 4,
}

impl Modality {
    /// Short CLI/DSL tag for this modality.
    pub const fn tag(self) -> &'static str {
        match self {
            Self::RepoCode => "@repo",
            Self::SpecDoc => "@docs",
            Self::ExtRef => "@ref",
            Self::WebRef => "@web",
            Self::Experiment => "@exp",
        }
    }
}

/// Epistemic lifecycle state of a recorded cell or experiment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EpistemicState {
    Active = 0,
    Todo = 1,
    Keep = 2,
    Reject = 3,
}

/// Six pragmatic lifecycle archetypes (`#0:io` .. `#5:gate`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Io = 0,
    State = 1,
    Space = 2,
    Op = 3,
    Route = 4,
    Gate = 5,
}

impl Archetype {
    /// All six archetypes in canonical index order.
    pub const ALL: [Self; 6] = [
        Self::Io,
        Self::State,
        Self::Space,
        Self::Op,
        Self::Route,
        Self::Gate,
    ];

    /// Canonical index in `0..6`.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Short DSL tag (`#io` .. `#gate`).
    pub const fn short_tag(self) -> &'static str {
        match self {
            Self::Io => "#io",
            Self::State => "#state",
            Self::Space => "#space",
            Self::Op => "#op",
            Self::Route => "#route",
            Self::Gate => "#gate",
        }
    }
}

/// Compact 8-byte geometric metadata header ($\Sigma$) attached to each cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaloHeader {
    pub scale: Scale,
    pub modality: Modality,
    pub primary_arch: Archetype,
    pub epistemic: EpistemicState,
    /// 6-bit mask of archetypes whose overlap exceeds $+1\sigma$ (`>= 2080` bits).
    pub archetype_mask: u8,
    /// 8-bit coarse routing sector computed via sub-block majority `popcount`.
    pub sector_id: u8,
    /// Quantized character-class transition rate in `0..=255` (integer-only).
    pub transition_delta_q8: u8,
    /// Quantized operator/delimiter byte density in `0..=255` (integer-only).
    pub op_density_q8: u8,
}

impl Default for HaloHeader {
    fn default() -> Self {
        Self {
            scale: Scale::Block,
            modality: Modality::RepoCode,
            primary_arch: Archetype::Op,
            epistemic: EpistemicState::Active,
            archetype_mask: 0,
            sector_id: 0,
            transition_delta_q8: 0,
            op_density_q8: 0,
        }
    }
}

/// Multiplier-free 128-bit ARX hash (`wrapping_add`, `rotate_left`, `^`).
#[inline]
pub fn arx_hash_bytes(bytes: &[u8], domain_salt: u64) -> (u64, u64) {
    let mut s0 = 0x6a09_e667_f3bc_c908u64 ^ domain_salt;
    let mut s1 = 0xbb67_ae85_84ca_a73bu64 ^ domain_salt.rotate_left(29);

    for &b in bytes {
        s0 = s0.wrapping_add(u64::from(b)).rotate_left(13) ^ s1;
        s1 = s1
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .rotate_left(31)
            .wrapping_add(s0);
    }

    // 4-round ARX avalanche finalizer
    let mut r = 0;
    while r < 4 {
        s0 = s0.wrapping_add(s1);
        s1 = s1.rotate_left(17) ^ s0;
        s0 = s0.rotate_left(41).wrapping_add(0x510e_527f_ade6_82d1);
        s1 = s1.wrapping_add(s0).rotate_left(23);
        r += 1;
    }
    if s0 == 0 && s1 == 0 {
        s0 = 0x9e37_79b9_7f4a_7c15;
    }
    (s0, s1)
}

/// Multiplier-free `xoroshiro128++` PRNG step (pure `wrapping_add`, `rotate_left`, `^`).
#[inline]
fn xoroshiro128pp_next(s0: &mut u64, s1: &mut u64) -> u64 {
    let a = *s0;
    let mut b = *s1;
    let result = a.wrapping_add(b).rotate_left(17).wrapping_add(a);
    b ^= a;
    *s0 = a.rotate_left(49) ^ b ^ (b << 21);
    *s1 = b.rotate_left(28);
    result
}

/// 4,096-bit binary hypervector sketch (`64 × u64` = 512 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sketch4096 {
    #[serde(with = "serde_words_64")]
    pub words: [u64; SKETCH_WORDS],
}

mod serde_words_64 {
    use super::SKETCH_WORDS;
    use serde::{
        de::{Error as DeError, SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserializer, Serializer,
    };

    pub fn serialize<S>(words: &[u64; SKETCH_WORDS], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(SKETCH_WORDS))?;
        for &w in words {
            seq.serialize_element(&w)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u64; SKETCH_WORDS], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WordsVisitor;
        impl<'de> Visitor<'de> for WordsVisitor {
            type Value = [u64; SKETCH_WORDS];
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "64 u64 words")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut out = [0u64; SKETCH_WORDS];
                for (i, slot) in out.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| DeError::invalid_length(i, &self))?;
                }
                Ok(out)
            }
        }
        deserializer.deserialize_seq(WordsVisitor)
    }
}

impl Default for Sketch4096 {
    fn default() -> Self {
        Self::zero()
    }
}

impl Sketch4096 {
    /// All-zero sketch.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            words: [0u64; SKETCH_WORDS],
        }
    }

    /// Deterministically expand a 128-bit `(s0, s1)` seed into 4,096 bits using
    /// multiplier-free `xoroshiro128++`.
    #[inline]
    pub fn from_seed(mut s0: u64, mut s1: u64) -> Self {
        if s0 == 0 && s1 == 0 {
            s0 = 0x9e37_79b9_7f4a_7c15;
        }
        let mut words = [0u64; SKETCH_WORDS];
        let mut i = 0;
        while i < SKETCH_WORDS {
            words[i] = xoroshiro128pp_next(&mut s0, &mut s1);
            i += 1;
        }
        Self { words }
    }

    /// Elementwise XOR binding ($A \oplus B$).
    #[inline]
    pub fn bind(&self, other: &Self) -> Self {
        let mut out = [0u64; SKETCH_WORDS];
        let mut i = 0;
        while i < SKETCH_WORDS {
            out[i] = self.words[i] ^ other.words[i];
            i += 1;
        }
        Self { words: out }
    }

    /// Cyclic left bit rotation across all 4,096 bits by `shift` positions.
    #[inline]
    pub fn rotl(&self, shift: u32) -> Self {
        let shift = (shift as usize) & (SKETCH_BITS - 1);
        if shift == 0 {
            return *self;
        }
        let word_shift = shift >> 6;
        let bit_shift = (shift & 63) as u32;
        let mut out = [0u64; SKETCH_WORDS];
        let mut i = 0;
        while i < SKETCH_WORDS {
            let src_hi = (i + SKETCH_WORDS - word_shift) & (SKETCH_WORDS - 1);
            if bit_shift == 0 {
                out[i] = self.words[src_hi];
            } else {
                let src_lo = (src_hi + SKETCH_WORDS - 1) & (SKETCH_WORDS - 1);
                out[i] =
                    (self.words[src_hi] << bit_shift) | (self.words[src_lo] >> (64 - bit_shift));
            }
            i += 1;
        }
        Self { words: out }
    }

    /// Exact Hamming distance in `[0, 4096]` via hardware `count_ones`.
    #[inline]
    pub fn hamming(&self, other: &Self) -> u32 {
        let mut dist = 0u32;
        let mut i = 0;
        while i < SKETCH_WORDS {
            dist += (self.words[i] ^ other.words[i]).count_ones();
            i += 1;
        }
        dist
    }

    /// Exact matching bit count (`4096 - hamming`) in `[0, 4096]`.
    #[inline]
    pub fn overlap_bits(&self, other: &Self) -> u32 {
        (SKETCH_BITS as u32) - self.hamming(other)
    }

    /// Exact integer bit margin above random chance (`overlap_bits - 2048`).
    ///
    /// Since $\sigma = 32$ bits for $D = 4096$, `excess_bits >> 5` gives the
    /// integer $\sigma$-multiple with zero division or floating point.
    #[inline]
    pub fn excess_bits(&self, other: &Self) -> i32 {
        (self.overlap_bits(other) as i32) - EXPECTED_RANDOM_OVERLAP
    }

    /// Coarse 8-bit routing sector ID computed by majority `popcount` across
    /// 8 sub-blocks of 8 words (512 bits per sub-block).
    #[inline]
    pub fn sector_id(&self) -> u8 {
        let mut sector = 0u8;
        let mut bit = 0usize;
        while bit < 8 {
            let base = bit << 3;
            let mut ones = 0u32;
            let mut w = 0usize;
            while w < 8 {
                ones += self.words[base + w].count_ones();
                w += 1;
            }
            if ones >= 256 {
                sector |= 1u8 << bit;
            }
            bit += 1;
        }
        sector
    }
}

/// Non-destructive signed `i16` bipolar accumulator for 4,096-bit sketches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SketchAccumulator {
    pub counts: [[i16; 64]; SKETCH_WORDS],
    pub total_weight: u32,
    pub item_count: u32,
}

impl Default for SketchAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl SketchAccumulator {
    /// Construct a zeroed accumulator.
    pub const fn new() -> Self {
        Self {
            counts: [[0i16; 64]; SKETCH_WORDS],
            total_weight: 0,
            item_count: 0,
        }
    }

    /// Reset all counters to zero.
    pub fn clear(&mut self) {
        self.counts = [[0i16; 64]; SKETCH_WORDS];
        self.total_weight = 0;
        self.item_count = 0;
    }

    /// Accumulate a [`Sketch4096`] with signed integer `weight` (`+w` to add, `-w` to revert).
    #[inline]
    pub fn accumulate(&mut self, sketch: &Sketch4096, weight: i16) {
        if weight == 0 {
            return;
        }
        let neg_weight = weight.saturating_neg();
        let mut w = 0usize;
        while w < SKETCH_WORDS {
            let word = sketch.words[w];
            let plane = &mut self.counts[w];
            let mut b = 0usize;
            while b < 64 {
                let byte_val = (word >> b) as u8;
                plane[b] = plane[b].saturating_add(if (byte_val & 0x01) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 1] = plane[b + 1].saturating_add(if (byte_val & 0x02) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 2] = plane[b + 2].saturating_add(if (byte_val & 0x04) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 3] = plane[b + 3].saturating_add(if (byte_val & 0x08) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 4] = plane[b + 4].saturating_add(if (byte_val & 0x10) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 5] = plane[b + 5].saturating_add(if (byte_val & 0x20) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 6] = plane[b + 6].saturating_add(if (byte_val & 0x40) != 0 {
                    weight
                } else {
                    neg_weight
                });
                plane[b + 7] = plane[b + 7].saturating_add(if (byte_val & 0x80) != 0 {
                    weight
                } else {
                    neg_weight
                });
                b += 8;
            }
            w += 1;
        }
        if weight > 0 {
            self.total_weight = self.total_weight.saturating_add(weight as u32);
            self.item_count = self.item_count.saturating_add(1);
        } else {
            let abs_w = weight.unsigned_abs() as u32;
            self.total_weight = self.total_weight.saturating_sub(abs_w);
            self.item_count = self.item_count.saturating_sub(1);
        }
    }

    /// Project the signed counter array onto a binary [`Sketch4096`].
    #[inline]
    pub fn project(&self) -> Sketch4096 {
        let mut words = [0u64; SKETCH_WORDS];
        let mut w = 0usize;
        while w < SKETCH_WORDS {
            let mut word = 0u64;
            let plane = &self.counts[w];
            let mut b = 0usize;
            while b < 64 {
                let c = plane[b];
                let bit = (c > 0) || (c == 0 && (((w << 6) + b) & 1) == 1);
                if bit {
                    word |= 1u64 << b;
                }
                b += 1;
            }
            words[w] = word;
            w += 1;
        }
        Sketch4096 { words }
    }
}

/// Multiplier-free hierarchical overlap blend:
/// $$\text{blended} = \frac{13 \cdot \text{local\_ov} + 3 \cdot \text{parent\_ov}}{16}$$
/// Computed purely via bit shifts and integer additions (`81.25%` local + `18.75%` parent).
#[inline]
pub const fn blend_hierarchical_overlap(local_ov: u32, parent_ov: u32) -> u32 {
    let local_13 = (local_ov << 3) + (local_ov << 2) + local_ov;
    let parent_3 = (parent_ov << 1) + parent_ov;
    (local_13 + parent_3) >> 4
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Space,
    Lower,
    Upper,
    Digit,
    OpMath,
    Delim,
    Other,
}

#[inline]
fn classify_byte(c: u8) -> CharClass {
    match c {
        b' ' | b'\t' | b'\r' | b'\n' => CharClass::Space,
        b'a'..=b'z' => CharClass::Lower,
        b'A'..=b'Z' => CharClass::Upper,
        b'0'..=b'9' => CharClass::Digit,
        b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>' | b'!' | b'&' | b'|' | b'^' | b'~'
        | b'%' | b'\\' => CharClass::OpMath,
        b'(' | b')' | b'{' | b'}' | b'[' | b']' | b'.' | b',' | b';' | b':' | b'_' | b'#'
        | b'@' | b'$' | b'"' | b'\'' | b'`' => CharClass::Delim,
        _ => CharClass::Other,
    }
}

/// Compute integer-quantized byte transition and operator density statistics (`0..=255`)
/// with zero floating-point operations.
pub fn compute_integer_byte_stats(bytes: &[u8]) -> (u8, u8) {
    if bytes.is_empty() {
        return (0, 0);
    }
    let limit = bytes.len().min(2048);
    let mut transitions = 0u32;
    let mut op_count = 0u32;
    let mut prev_cc = classify_byte(bytes[0]);

    for &b in &bytes[..limit] {
        let cc = classify_byte(b);
        if cc != prev_cc {
            transitions += 1;
        }
        if matches!(cc, CharClass::OpMath | CharClass::Delim) {
            op_count += 1;
        }
        prev_cc = cc;
    }

    let n = limit as u32;
    // Scale by 255 using `(x << 8) - x` (zero multiply instruction)
    let trans_q8 = (((transitions << 8) - transitions) / n).min(255) as u8;
    let op_q8 = (((op_count << 8) - op_count) / n).min(255) as u8;
    (trans_q8, op_q8)
}

#[inline]
fn is_structural_stopword(s: &[u8]) -> bool {
    matches!(
        s,
        b"the"
            | b"and"
            | b"for"
            | b"with"
            | b"this"
            | b"that"
            | b"from"
            | b"what"
            | b"is"
            | b"of"
            | b"to"
            | b"in"
            | b"return"
            | b"local"
            | b"const"
            | b"void"
            | b"auto"
            | b"pub"
            | b"let"
            | b"def"
            | b"fn"
            | b"function"
            | b"end"
            | b"int"
            | b"std"
            | b"inline"
            | b"template"
            | b"typename"
            | b"class"
            | b"struct"
    )
}

/// Encode a byte or text span into a 4,096-bit [`Sketch4096`] combining:
/// 1. Case-split and delimiter-split identifier patches (`weight = 4` for content, `1` for stopwords).
/// 2. 5-byte prefix stems (`weight = 3`) and 4-byte prefix stems (`weight = 2`) so inflected or
///    related identifiers share high-weight sub-features without language-specific stemmers.
/// 3. Full hypervector cyclic rotation across [`PRIME_STRIDES`] (`rotl(3)` bigrams at `weight = 2`,
///    `rotl(5)` skip-grams at `weight = 1`).
/// 4. 4-byte character shingles (`weight = 1`) for sub-token and compound resilience.
/// 5. Top-48 deterministic weight sparsification into [`SketchAccumulator`].
pub fn encode_span_sketch(text: &str) -> Sketch4096 {
    let bytes = text.as_bytes();
    let limit = bytes.len().min(3072);

    let mut feature_list: Vec<(Sketch4096, i16)> = Vec::with_capacity(96);
    let mut content_token_hvs: Vec<Sketch4096> = Vec::with_capacity(64);
    let mut cur_word: Vec<u8> = Vec::with_capacity(32);

    let mut push_feature = |hv: Sketch4096, weight: i16| {
        if weight == 0 {
            return;
        }
        for entry in &mut feature_list {
            if entry.0 == hv {
                entry.1 = entry.1.saturating_add(weight);
                return;
            }
        }
        if feature_list.len() < 192 {
            feature_list.push((hv, weight));
        }
    };

    let flush_word = |cur: &mut Vec<u8>,
                      push_fn: &mut dyn FnMut(Sketch4096, i16),
                      content_hvs: &mut Vec<Sketch4096>| {
        if cur.len() >= 2 {
            let stop = is_structural_stopword(cur);
            let (s0, s1) = arx_hash_bytes(cur, 0x1001_5001_u64);
            let tok_hv = Sketch4096::from_seed(s0, s1);
            let w = if stop { 1 } else { 4 };
            push_fn(tok_hv, w);

            if !stop {
                content_hvs.push(tok_hv);
                if cur.len() >= 5 {
                    let (ps0, ps1) = arx_hash_bytes(&cur[..5], 0x2002_5005_u64);
                    push_fn(Sketch4096::from_seed(ps0, ps1), 3);
                }
                if cur.len() >= 4 {
                    let (ps0, ps1) = arx_hash_bytes(&cur[..4], 0x2002_4004_u64);
                    push_fn(Sketch4096::from_seed(ps0, ps1), 2);
                }
            }
        }
        cur.clear();
    };

    let mut i = 0usize;
    while i < limit {
        let c = bytes[i];
        if c.is_ascii_uppercase() {
            if !cur_word.is_empty() && i > 0 && bytes[i - 1].is_ascii_lowercase() {
                flush_word(&mut cur_word, &mut push_feature, &mut content_token_hvs);
            }
            cur_word.push(c.to_ascii_lowercase());
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            cur_word.push(c);
        } else {
            flush_word(&mut cur_word, &mut push_feature, &mut content_token_hvs);
            if classify_byte(c) == CharClass::OpMath {
                let mut op_buf = [c, 0u8];
                let op_len = if i + 1 < limit && classify_byte(bytes[i + 1]) == CharClass::OpMath {
                    op_buf[1] = bytes[i + 1];
                    2
                } else {
                    1
                };
                let (os0, os1) = arx_hash_bytes(&op_buf[..op_len], 0x3003_0000_u64);
                push_feature(Sketch4096::from_seed(os0, os1), 2);
            }
        }
        if content_token_hvs.len() >= 64 {
            break;
        }
        i += 1;
    }
    flush_word(&mut cur_word, &mut push_feature, &mut content_token_hvs);

    // Prime-Router positional mixing across content tokens using full 4,096-bit cyclic rotation
    for idx in 1..content_token_hvs.len() {
        let prev1 = content_token_hvs[idx - 1].rotl(PRIME_STRIDES[0]);
        let bg = prev1.bind(&content_token_hvs[idx]);
        push_feature(bg, 2);

        if idx >= 2 {
            let prev2 = content_token_hvs[idx - 2].rotl(PRIME_STRIDES[1]);
            let sg = prev2.bind(&content_token_hvs[idx]);
            push_feature(sg, 1);
        }
        if idx >= 3 {
            let prev3 = content_token_hvs[idx - 3].rotl(PRIME_STRIDES[2]);
            let tg = prev3.bind(&content_token_hvs[idx]);
            push_feature(tg, 1);
        }
    }

    // 4-byte character shingles across whitespace-stripped text
    let mut compact: Vec<u8> = Vec::with_capacity(256);
    for &c in &bytes[..limit.min(512)] {
        if !c.is_ascii_whitespace() {
            compact.push(c.to_ascii_lowercase());
            if compact.len() >= 256 {
                break;
            }
        }
    }
    if compact.len() >= 4 {
        let step = (compact.len() / 24).max(2);
        let mut pos = 0usize;
        while pos + 4 <= compact.len() {
            let (sh0, sh1) = arx_hash_bytes(&compact[pos..pos + 4], 0x4004_0004_u64);
            push_feature(Sketch4096::from_seed(sh0, sh1), 1);
            pos += step;
        }
    }

    // Stable top-48 selection by integer weight
    let top_limit = feature_list.len().min(48);
    for sel in 0..top_limit {
        let mut best = sel;
        for cand in (sel + 1)..feature_list.len() {
            if feature_list[cand].1 > feature_list[best].1 {
                best = cand;
            }
        }
        if best != sel {
            let item = feature_list.remove(best);
            feature_list.insert(sel, item);
        }
    }

    let mut acc = SketchAccumulator::new();
    for &(ref hv, weight) in &feature_list[..top_limit] {
        acc.accumulate(hv, weight);
    }
    acc.project()
}

/// Metadata and sketch stored per slot in [`ScopedAddressDirectory`].
#[derive(Debug, Clone)]
pub struct SlotSketchMeta {
    pub address: PrimeAddress,
    pub parent_scope: Option<String>,
    pub halo: HaloHeader,
    pub sketch: Sketch4096,
    pub preview: String,
}

/// Recorded scientific experiment entry in the epistemic ledger.
#[derive(Debug, Clone)]
pub struct ExperimentRecord {
    pub exp_id: String,
    pub archetype: Archetype,
    pub verdict: EpistemicState,
    pub sketch: Sketch4096,
    pub description: String,
}

/// Architectural task placement recommendation produced by [`ScopedAddressDirectory::place_task`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementDecision {
    /// Target block exceeds $+5\sigma$ (`excess_bits >= 160`): modify the existing block directly.
    InsertAtExistingBlock,
    /// Parent scope exceeds $+3\sigma$ (`excess_bits >= 96`): scaffold a new block in the matched scope.
    ScaffoldNewBlockInFile,
    /// Neither block nor scope reaches $+3\sigma$: create a new module for the classified archetype.
    ScaffoldNewFileForStage,
}

/// Hazard alert fired when a task resonates at $\ge +3\sigma$ with a `Reject` experiment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HazardAlert {
    pub exp_id: String,
    pub excess_bits: i32,
    pub sigma_multiple: i32,
    pub description: String,
}

/// Full result of [`ScopedAddressDirectory::place_task`].
#[derive(Debug, Clone)]
pub struct PlacementPlan {
    pub best_archetype: Archetype,
    pub archetype_excess_bits: i32,
    pub best_slot: Option<PrimeAddress>,
    pub best_slot_excess_bits: i32,
    pub best_scope: Option<String>,
    pub best_scope_excess_bits: i32,
    pub decision: PlacementDecision,
    pub hazard: Option<HazardAlert>,
}

/// Hierarchical, sketch-augmented wrapper around [`AddressDirectory`] supporting:
/// - Exact [`PrimeAddress`] storage and [`Store`] Merkle verification.
/// - Non-destructive parent-scope (`ScopeId`), archetype, and global [`SketchAccumulator`]s
///   with exact $O(1)$ algebraic subtraction on overwrite or capacity eviction.
/// - Two-stage hybrid sieve + multi-granularity sketch routing (`read_hybrid_sieve_from`)
///   with a hard $+3\sigma$ integer abstention gate.
/// - Charter-compliant task placement and `Keep`/`Reject` epistemic hazard detection.
#[derive(Debug, Clone)]
pub struct ScopedAddressDirectory {
    directory: AddressDirectory,
    max_slots: usize,
    next_sequence: u64,
    active_meta: BTreeMap<PrimeAddress, (u64, SlotSketchMeta)>,
    evicted_meta: BTreeMap<PrimeAddress, (u64, SlotSketchMeta)>,
    scope_accumulators: BTreeMap<String, SketchAccumulator>,
    scope_projections: BTreeMap<String, Sketch4096>,
    pub global_acc: SketchAccumulator,
    pub arch_acc: [SketchAccumulator; 6],
    pub arch_sketch: [Sketch4096; 6],
    pub experiments: Vec<ExperimentRecord>,
}

impl ScopedAddressDirectory {
    /// Create a bounded hierarchical directory holding at most `max_slots` active addresses.
    pub fn new(max_slots: usize) -> Result<Self> {
        let directory = AddressDirectory::new(max_slots)?;
        let mut arch_acc = [
            SketchAccumulator::new(),
            SketchAccumulator::new(),
            SketchAccumulator::new(),
            SketchAccumulator::new(),
            SketchAccumulator::new(),
            SketchAccumulator::new(),
        ];
        let mut arch_sketch = [Sketch4096::zero(); 6];
        init_pragmatic_archetypes(&mut arch_acc, &mut arch_sketch);

        Ok(Self {
            directory,
            max_slots,
            next_sequence: 1,
            active_meta: BTreeMap::new(),
            evicted_meta: BTreeMap::new(),
            scope_accumulators: BTreeMap::new(),
            scope_projections: BTreeMap::new(),
            global_acc: SketchAccumulator::new(),
            arch_acc,
            arch_sketch,
            experiments: Vec::new(),
        })
    }

    /// Reference to the underlying [`AddressDirectory`].
    pub const fn directory(&self) -> &AddressDirectory {
        &self.directory
    }

    /// Projected hypervector sketch for a named parent scope (e.g. file or session URI).
    pub fn scope_sketch(&self, scope: &str) -> Option<&Sketch4096> {
        self.scope_projections.get(scope)
    }

    /// Classify a [`Sketch4096`] against the 6 lifecycle archetypes using integer bit overlap.
    pub fn classify_archetype(&self, sketch: &Sketch4096) -> (Archetype, u8, i32) {
        let mut best_arch = Archetype::Op;
        let mut best_excess = i32::MIN;
        let mut mask = 0u8;

        for arch in Archetype::ALL {
            let idx = arch.index();
            let excess = sketch.excess_bits(&self.arch_sketch[idx]);
            if excess > best_excess {
                best_excess = excess;
                best_arch = arch;
            }
            if excess >= SIGMA_BITS {
                mask |= 1u8 << idx;
            }
        }
        if mask == 0 {
            mask |= 1u8 << best_arch.index();
        }
        (best_arch, mask, best_excess)
    }

    /// Write `address` and `target` into the directory along with its source `context_text`
    /// and optional enclosing `parent_scope` (e.g., file URI or conversation scope).
    ///
    /// Automatically updates (and on overwrite or capacity eviction, **exact-subtracts**)
    /// the slot's sketch from `global_acc`, `arch_acc`, and `scope_accumulators[parent_scope]`.
    pub fn write_scoped(
        &mut self,
        address: PrimeAddress,
        target: ObjectTarget,
        context_text: &str,
        parent_scope: Option<&str>,
        modality: Modality,
    ) -> Result<u64> {
        let seq = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(Error::LimitExceeded("scoped address sequence"))?;

        let new_sketch = encode_span_sketch(context_text);
        let (best_arch, arch_mask, _) = self.classify_archetype(&new_sketch);
        let (trans_q8, op_q8) = compute_integer_byte_stats(context_text.as_bytes());
        let halo = HaloHeader {
            scale: Scale::Block,
            modality,
            primary_arch: best_arch,
            epistemic: EpistemicState::Active,
            archetype_mask: arch_mask,
            sector_id: new_sketch.sector_id(),
            transition_delta_q8: trans_q8,
            op_density_q8: op_q8,
        };
        let preview: String = context_text.chars().take(68).collect();
        let new_meta = SlotSketchMeta {
            address: address.clone(),
            parent_scope: parent_scope.map(str::to_owned),
            halo,
            sketch: new_sketch,
            preview,
        };

        self.evicted_meta.remove(&address);

        if let Some((_old_seq, old_meta)) = self.active_meta.get(&address).cloned() {
            // Subtract prior sketch contribution before applying the updated sketch
            self.retract_meta(&old_meta);
        } else if self.active_meta.len() >= self.max_slots {
            if let Some(oldest_addr) = self
                .active_meta
                .iter()
                .min_by_key(|(_, (s, _))| *s)
                .map(|(k, _)| k.clone())
            {
                if let Some((_, evicted)) = self.active_meta.remove(&oldest_addr) {
                    self.retract_meta(&evicted);
                    self.evicted_meta.insert(oldest_addr, (seq, evicted));
                }
            }
        }

        let version = self.directory.write(address.clone(), target)?;
        self.apply_meta(&new_meta);
        self.active_meta.insert(address, (seq, new_meta));
        Ok(version)
    }

    /// Remove an entire parent scope (e.g. a file URI) and algebraically subtract all of its
    /// constituent block sketches from the global and archetype accumulators in $O(k)$.
    pub fn remove_scope(&mut self, scope_uri: &str) {
        let to_remove: Vec<PrimeAddress> = self
            .active_meta
            .iter()
            .filter(|(_, (_, meta))| meta.parent_scope.as_deref() == Some(scope_uri))
            .map(|(addr, _)| addr.clone())
            .collect();

        for addr in to_remove {
            if let Some((_, meta)) = self.active_meta.remove(&addr) {
                self.retract_meta(&meta);
            }
        }
        self.scope_accumulators.remove(scope_uri);
        self.scope_projections.remove(scope_uri);
    }

    /// Resolve a query using the two-stage hybrid exact-sieve + multi-granularity sketch router:
    /// 1. If `query_address` is supplied and matches an active or evicted slot via exact
    ///    prime GCD (`gcd_shared_primes > 0`) with a unique top rank, uses the exact sieve result.
    /// 2. If `query_address` is `None` or fails to share exact primes due to surface inflection,
    ///    compound renaming, or paraphrasing, scores active slots by hierarchical sketch overlap
    ///    (`(13 * local_ov + 3 * parent_ov) >> 4`), filtered by optional `modality` / `archetype`.
    /// 3. Enforces the hard $+3\sigma$ integer gate (`blended_overlap >= 2144` bits):
    ///    - Admitted active slot $\to$ Merkle-verified [`AddressReadOutcome::Hit`].
    ///    - Admitted evicted slot $\to$ [`AddressReadOutcome::Evicted`] (0 backend fetches).
    ///    - Below $+3\sigma$ $\to$ [`AddressReadOutcome::Absent`] (0 backend fetches).
    pub fn read_hybrid_sieve(
        &self,
        store: &Store,
        tokenizer: Option<&PinnedTokenizer>,
        query_address: Option<&PrimeAddress>,
        query_text: &str,
        modality_filter: Option<Modality>,
        archetype_filter: Option<Archetype>,
    ) -> Result<AddressReadOutcome> {
        self.read_hybrid_sieve_from(
            store,
            store.limits(),
            tokenizer,
            query_address,
            query_text,
            modality_filter,
            archetype_filter,
        )
    }

    /// Two-stage hybrid sieve + sketch read against any [`RecordBackend`].
    #[allow(clippy::too_many_arguments)]
    pub fn read_hybrid_sieve_from<B: RecordBackend>(
        &self,
        backend: &B,
        limits: &Limits,
        tokenizer: Option<&PinnedTokenizer>,
        query_address: Option<&PrimeAddress>,
        query_text: &str,
        modality_filter: Option<Modality>,
        archetype_filter: Option<Archetype>,
    ) -> Result<AddressReadOutcome> {
        let query_sketch = encode_span_sketch(query_text);
        let query_sector = query_sketch.sector_id();

        // Stage 1 + 2 combined ranking over active slots:
        // If `query_address` shares prime factors with one or more active slots, rank
        // those prime-admitted candidates first by `(shared_primes, ordered_pairs, blended_overlap, sequence)`.
        type PrimeSketchRank = (usize, usize, u32, u64);
        if let Some(q_addr) = query_address {
            let mut best_prime_cand: Option<(PrimeSketchRank, &PrimeAddress)> = None;
            for (addr, (seq, meta)) in &self.active_meta {
                if !matches_filters(&meta.halo, modality_filter, archetype_filter) {
                    continue;
                }
                let shared = q_addr.gcd_shared_primes(addr);
                if shared == 0 {
                    continue;
                }
                let pairs = q_addr.ordered_pair_overlap(addr);
                let blended = self.score_meta_sketch(&query_sketch, meta);
                let rank = (shared, pairs, blended, *seq);
                if best_prime_cand.is_none_or(|(best_r, _)| rank > best_r) {
                    best_prime_cand = Some((rank, addr));
                }
            }
            if let Some((_, best_addr)) = best_prime_cand {
                return self.directory.read_exact_from(
                    backend,
                    limits,
                    tokenizer,
                    best_addr,
                    VersionSelect::Current,
                );
            }
        }

        // Stage 2: Surface tokens shifted (`shared_primes == 0` or no prime query supplied).
        // Rank active slots by hierarchical sketch overlap with coarse sector tie-breaking,
        // subject to the hard +3 sigma integer gate (`>= 2144` bits).
        let threshold_overlap = (EXPECTED_RANDOM_OVERLAP + THREE_SIGMA_EXCESS_BITS) as u32;
        let mut best_sketch_cand: Option<((u32, u32, u64), &PrimeAddress)> = None;

        for (addr, (seq, meta)) in &self.active_meta {
            if !matches_filters(&meta.halo, modality_filter, archetype_filter) {
                continue;
            }
            let blended = self.score_meta_sketch(&query_sketch, meta);
            if blended < threshold_overlap {
                continue;
            }
            let sector_proximity = 8u32 - (query_sector ^ meta.halo.sector_id).count_ones();
            let rank = (blended, sector_proximity, *seq);
            if best_sketch_cand.is_none_or(|(best_r, _)| rank > best_r) {
                best_sketch_cand = Some((rank, addr));
            }
        }

        if let Some((_, best_addr)) = best_sketch_cand {
            return self.directory.read_exact_from(
                backend,
                limits,
                tokenizer,
                best_addr,
                VersionSelect::Current,
            );
        }

        // Check evicted slots by prime overlap first, then by +3 sigma sketch gate.
        if let Some(q_addr) = query_address {
            let mut best_evicted_prime: Option<((usize, usize, u64), &PrimeAddress, u64)> = None;
            for (addr, (evicted_seq, meta)) in &self.evicted_meta {
                if !matches_filters(&meta.halo, modality_filter, archetype_filter) {
                    continue;
                }
                let shared = q_addr.gcd_shared_primes(addr);
                if shared == 0 {
                    continue;
                }
                let pairs = q_addr.ordered_pair_overlap(addr);
                let rank = (shared, pairs, *evicted_seq);
                if best_evicted_prime.is_none_or(|(best_r, _, _)| rank > best_r) {
                    best_evicted_prime = Some((rank, addr, *evicted_seq));
                }
            }
            if let Some((_, addr, evicted_at_sequence)) = best_evicted_prime {
                return Ok(AddressReadOutcome::Evicted {
                    address: addr.clone(),
                    evicted_at_sequence,
                });
            }
        }

        let mut best_evicted_sketch: Option<((u32, u64), &PrimeAddress, u64)> = None;
        for (addr, (evicted_seq, meta)) in &self.evicted_meta {
            if !matches_filters(&meta.halo, modality_filter, archetype_filter) {
                continue;
            }
            let local_ov = query_sketch.overlap_bits(&meta.sketch);
            if local_ov < threshold_overlap {
                continue;
            }
            let rank = (local_ov, *evicted_seq);
            if best_evicted_sketch.is_none_or(|(best_r, _, _)| rank > best_r) {
                best_evicted_sketch = Some((rank, addr, *evicted_seq));
            }
        }
        if let Some((_, addr, evicted_at_sequence)) = best_evicted_sketch {
            return Ok(AddressReadOutcome::Evicted {
                address: addr.clone(),
                evicted_at_sequence,
            });
        }

        Ok(AddressReadOutcome::Absent)
    }

    /// Record a scientific experiment verdict (`Keep` or `Reject`) in the manifold ledger.
    ///
    /// - `Keep` reinforces the global and archetype accumulators (`weight = +2`).
    /// - `Reject` registers a negative hazard repulsor checked by [`Self::place_task`].
    pub fn record_experiment(
        &mut self,
        exp_id: &str,
        stage: Archetype,
        verdict: EpistemicState,
        description: &str,
    ) {
        let sketch = encode_span_sketch(description);
        if verdict == EpistemicState::Keep {
            self.global_acc.accumulate(&sketch, 2);
            let idx = stage.index();
            self.arch_acc[idx].accumulate(&sketch, 2);
            self.arch_sketch[idx] = self.arch_acc[idx].project();
        }
        self.experiments.push(ExperimentRecord {
            exp_id: exp_id.to_owned(),
            archetype: stage,
            verdict,
            sketch,
            description: description.to_owned(),
        });
    }

    /// Evaluate integer semantic lucidity, select the best target block and parent scope,
    /// and check all `Reject` experiments for $\ge +3\sigma$ resonance hazards.
    pub fn place_task(&self, task_spec: &str) -> PlacementPlan {
        let task_sketch = encode_span_sketch(task_spec);
        let (best_arch, _mask, arch_excess) = self.classify_archetype(&task_sketch);

        let mut best_slot: Option<PrimeAddress> = None;
        let mut best_slot_excess = i32::MIN;
        for (addr, (_, meta)) in &self.active_meta {
            if meta.halo.modality != Modality::RepoCode {
                continue;
            }
            let blended = self.score_meta_sketch(&task_sketch, meta) as i32;
            let excess = blended - EXPECTED_RANDOM_OVERLAP;
            if excess > best_slot_excess {
                best_slot_excess = excess;
                best_slot = Some(addr.clone());
            }
        }

        let mut best_scope: Option<String> = None;
        let mut best_scope_excess = i32::MIN;
        for (scope_name, scope_hv) in &self.scope_projections {
            let excess = task_sketch.excess_bits(scope_hv);
            if excess > best_scope_excess {
                best_scope_excess = excess;
                best_scope = Some(scope_name.clone());
            }
        }

        let mut hazard: Option<HazardAlert> = None;
        for exp in &self.experiments {
            if exp.verdict != EpistemicState::Reject {
                continue;
            }
            let excess = task_sketch.excess_bits(&exp.sketch);
            if excess >= THREE_SIGMA_EXCESS_BITS
                && hazard.as_ref().is_none_or(|h| excess > h.excess_bits)
            {
                hazard = Some(HazardAlert {
                    exp_id: exp.exp_id.clone(),
                    excess_bits: excess,
                    sigma_multiple: excess >> 5,
                    description: exp.description.clone(),
                });
            }
        }

        // Integer-only decision gates (zero floats, zero tanh):
        // >= +5 sigma (160 bits excess) -> InsertAtExistingBlock
        // >= +3 sigma (96 bits excess) on scope or slot -> ScaffoldNewBlockInFile
        // otherwise -> ScaffoldNewFileForStage
        let decision = if best_slot.is_some() && best_slot_excess >= 5 * SIGMA_BITS {
            PlacementDecision::InsertAtExistingBlock
        } else if best_scope.is_some()
            && (best_scope_excess >= THREE_SIGMA_EXCESS_BITS
                || best_slot_excess >= THREE_SIGMA_EXCESS_BITS)
        {
            PlacementDecision::ScaffoldNewBlockInFile
        } else {
            PlacementDecision::ScaffoldNewFileForStage
        };

        PlacementPlan {
            best_archetype: best_arch,
            archetype_excess_bits: arch_excess,
            best_slot,
            best_slot_excess_bits: if best_slot_excess == i32::MIN {
                0
            } else {
                best_slot_excess
            },
            best_scope,
            best_scope_excess_bits: if best_scope_excess == i32::MIN {
                0
            } else {
                best_scope_excess
            },
            decision,
            hazard,
        }
    }

    #[inline]
    fn score_meta_sketch(&self, query_sketch: &Sketch4096, meta: &SlotSketchMeta) -> u32 {
        let local_ov = query_sketch.overlap_bits(&meta.sketch);
        if let Some(ref scope) = meta.parent_scope {
            if let Some(parent_hv) = self.scope_projections.get(scope) {
                let parent_ov = query_sketch.overlap_bits(parent_hv);
                return blend_hierarchical_overlap(local_ov, parent_ov);
            }
        }
        local_ov
    }

    fn apply_meta(&mut self, meta: &SlotSketchMeta) {
        self.global_acc.accumulate(&meta.sketch, 1);
        let arch_idx = meta.halo.primary_arch.index();
        self.arch_acc[arch_idx].accumulate(&meta.sketch, 1);
        self.arch_sketch[arch_idx] = self.arch_acc[arch_idx].project();

        if let Some(ref scope) = meta.parent_scope {
            let acc = self.scope_accumulators.entry(scope.clone()).or_default();
            acc.accumulate(&meta.sketch, 1);
            let proj = acc.project();
            self.scope_projections.insert(scope.clone(), proj);
        }
    }

    fn retract_meta(&mut self, meta: &SlotSketchMeta) {
        self.global_acc.accumulate(&meta.sketch, -1);
        let arch_idx = meta.halo.primary_arch.index();
        self.arch_acc[arch_idx].accumulate(&meta.sketch, -1);
        self.arch_sketch[arch_idx] = self.arch_acc[arch_idx].project();

        if let Some(ref scope) = meta.parent_scope {
            if let Some(acc) = self.scope_accumulators.get_mut(scope) {
                acc.accumulate(&meta.sketch, -1);
                if acc.item_count == 0 {
                    self.scope_projections.remove(scope);
                } else {
                    self.scope_projections.insert(scope.clone(), acc.project());
                }
            }
        }
    }
}

#[inline]
fn matches_filters(
    halo: &HaloHeader,
    modality_filter: Option<Modality>,
    archetype_filter: Option<Archetype>,
) -> bool {
    if halo.epistemic == EpistemicState::Reject {
        return false;
    }
    if let Some(mod_f) = modality_filter {
        if halo.modality != mod_f {
            return false;
        }
    }
    if let Some(arch_f) = archetype_filter {
        let primary_match = halo.primary_arch == arch_f;
        let mask_match = (halo.archetype_mask & (1u8 << arch_f.index())) != 0;
        if !primary_match && !mask_match {
            return false;
        }
    }
    true
}

fn init_pragmatic_archetypes(
    arch_acc: &mut [SketchAccumulator; 6],
    arch_sketch: &mut [Sketch4096; 6],
) {
    const SEEDS: [&str; 6] = [
        // #0:io (INGEST_EMIT)
        "ingest parse tokenize byte stream patch surprisal shannon entropy demodulate \
         readfile writefile cli cmdline input output serialize decode format utf8 buffer match html wget",
        // #1:state (DATA_SUBSTRATE)
        "struct data_structure tensor field connection hypervector bitfield uint64 int16 \
         accumulator state queue registry memory lattice cache buffer slice dyadic manifest record",
        // #2:space (INDEX_TOPOLOGY)
        "manifold metric topology coordinate hopf sphere poincare disk lattice graph tree \
         index symbol_table lookup hash address uor sector prime zeta distance s3 directory",
        // #3:op (TRANSFORM_KERNEL)
        "operator kernel transform transport parallel_transport laplacian helmholtz fft \
         stencil attention phase rotor quaternion bind permute popcount hamming overlap",
        // #4:route (CONTROL_ADAPT)
        "route router dispatch select rank bijective adapt update delta_rule \
         schedule decompose_and_add_tasks start_task jump_to_target queue next prev affinity",
        // #5:gate (VERIFY_GOVERN)
        "verify gate audit lucidity som firewall threshold invariant ensure require assert \
         theorem proof check review accept reject ledger hash_chain quarantine test bmu",
    ];

    for i in 0..6 {
        arch_acc[i].clear();
        let seed_sketch = encode_span_sketch(SEEDS[i]);
        arch_acc[i].accumulate(&seed_sketch, 12);
        arch_sketch[i] = arch_acc[i].project();
    }
}
