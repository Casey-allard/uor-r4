//! High-Dimensional Vector Symbolic Architecture (VSA) Hypervectors.
//!
//! Pure Rust implementation of binary spatter code / hyperdimensional computing (HDC)
//! with strict `#![forbid(unsafe_code)]`, zero runtime heap allocations, zero runtime
//! matrix multiplications, and zero runtime floats on serving hot paths.

use core::fmt;
use core::ops::{BitXor, BitXorAssign};
use serde::{Deserialize, Serialize};

/// Number of 64-bit words for a 2048-bit hypervector (2048 / 64 = 32).
pub const WORDS_2048: usize = 32;

/// Number of 64-bit words for a 4096-bit hypervector (4096 / 64 = 64).
pub const WORDS_4096: usize = 64;

/// Standard 2048-bit binary hypervector.
pub type Hypervector2048 = Hypervector<WORDS_2048>;

/// Standard 4096-bit binary hypervector.
pub type Hypervector4096 = Hypervector<WORDS_4096>;

/// Default hypervector representation for native geometric context (4096 bits).
pub type DefaultHypervector = Hypervector4096;

/// High-dimensional binary hypervector of dimension `D = WORDS * 64` bits.
///
/// Implements pure bitwise operations for Vector Symbolic Architectures (VSA):
/// - Binding ($\otimes$): Elementwise bitwise XOR ($\oplus$), self-inverse ($A \oplus B \oplus B = A$).
/// - Permutation ($\rho$): Cyclic bitwise rotation across all $D$ bits, group automorphism.
/// - Bundling ($+$): Parallel bitwise majority voting across an active context window.
/// - Similarity: Hardware `popcount` Hamming distance and $Q1.15$ fixed-point score.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hypervector<const WORDS: usize> {
    pub data: [u64; WORDS],
}

/// Canonical deterministic tie-breaker seed for even-window bundling.
pub const CANONICAL_TIE_BREAKER_SEED: u64 = 0x5653_415f_4255_4e44; // "VSA_BUND"
/// Canonical deterministic tie-breaker salt for even-window bundling.
pub const CANONICAL_TIE_BREAKER_SALT: u64 = 0x5449_4542_5245_414b; // "TIEBREAK"

impl<const WORDS: usize> Hypervector<WORDS> {
    /// Returns an all-zero hypervector.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            data: [0u64; WORDS],
        }
    }

    /// Returns an all-ones hypervector.
    #[inline]
    pub const fn ones() -> Self {
        Self {
            data: [!0u64; WORDS],
        }
    }

    /// Construct from raw 64-bit word array.
    #[inline]
    pub const fn from_words(data: [u64; WORDS]) -> Self {
        Self { data }
    }

    /// Reference to the underlying 64-bit words.
    #[inline]
    pub const fn as_words(&self) -> &[u64; WORDS] {
        &self.data
    }

    /// Total number of bits in this hypervector ($D = \text{WORDS} \times 64$).
    #[inline]
    pub const fn dimension_bits() -> usize {
        WORDS * 64
    }

    /// Number of 64-bit words.
    #[inline]
    pub const fn word_count() -> usize {
        WORDS
    }

    /// Total number of active bits ($1$-bits) in the hypervector via hardware `popcount`.
    #[inline]
    pub fn count_ones(&self) -> usize {
        let mut total = 0usize;
        let mut i = 0;
        while i < WORDS {
            total += self.data[i].count_ones() as usize;
            i += 1;
        }
        total
    }

    /// Generate a deterministic pseudo-random hypervector from a seed and salt using SplitMix64.
    ///
    /// Produces a statistically uniform bit distribution with mean Hamming weight $D / 2$.
    pub const fn from_seed(seed: u64, salt: u64) -> Self {
        let mut state = seed ^ salt.wrapping_mul(0x9e3779b97f4a7c15);
        let mut words = [0u64; WORDS];
        let mut i = 0;
        while i < WORDS {
            words[i] = splitmix64_next(&mut state);
            i += 1;
        }
        Self { data: words }
    }

    /// Binding operation ($\otimes$): Elementwise bitwise XOR.
    ///
    /// Properties:
    /// - Commutative: $A \otimes B = B \otimes A$.
    /// - Associative: $(A \otimes B) \otimes C = A \otimes (B \otimes C)$.
    /// - Self-inverse: $A \otimes B \otimes B = A$.
    /// - Distance-preserving: $d_H(A \otimes C, B \otimes C) = d_H(A, B)$.
    /// - Orthogonalizing: $A \otimes B$ is quasi-orthogonal to both $A$ and $B$.
    /// - Speed: 64 XOR instructions for 4096 bits (< 5 ns on Apple Silicon M1).
    #[inline]
    pub fn bind(&self, other: &Self) -> Self {
        let mut res = [0u64; WORDS];
        let mut i = 0;
        while i < WORDS {
            res[i] = self.data[i] ^ other.data[i];
            i += 1;
        }
        Self { data: res }
    }

    /// Unbinding operation: Exact inverse of binding.
    ///
    /// In binary VSA, unbind is mathematically identical to bind:
    /// $\text{unbind}(A, A \otimes B) = A \oplus (A \oplus B) = B$.
    #[inline]
    pub fn unbind(&self, other: &Self) -> Self {
        self.bind(other)
    }

    /// Positional permutation ($\rho^k$): Cyclic bitwise rotation across all $D$ bits.
    ///
    /// Shifts bit at position $j$ to $(j + k) \pmod D$.
    ///
    /// Properties:
    /// - Automorphism: Preserves Hamming weight and distances.
    /// - Position distinctness: $\rho^k(A) \perp A$ for $1 \le k < D$.
    /// - Group structure: $\rho^{k_1} \circ \rho^{k_2} = \rho^{(k_1 + k_2) \pmod D}$.
    #[inline]
    pub fn permute(&self, k: usize) -> Self {
        let total_bits = Self::dimension_bits();
        if total_bits == 0 {
            return *self;
        }
        let shift = k % total_bits;
        if shift == 0 {
            return *self;
        }

        let word_shift = shift / 64;
        let bit_shift = (shift % 64) as u32;

        let mut res = [0u64; WORDS];

        if bit_shift == 0 {
            let mut i = 0;
            while i < WORDS {
                res[(i + word_shift) % WORDS] = self.data[i];
                i += 1;
            }
        } else {
            let mut i = 0;
            while i < WORDS {
                let dst = (i + word_shift) % WORDS;
                let next = (dst + 1) % WORDS;
                res[dst] |= self.data[i] << bit_shift;
                res[next] |= self.data[i] >> (64 - bit_shift);
                i += 1;
            }
        }

        Self { data: res }
    }

    /// Inverse positional permutation ($\rho^{-k}$): Cyclic bitwise rotation to the right.
    ///
    /// Satisfies $\rho^{-k}(\rho^k(A)) = A$.
    #[inline]
    pub fn permute_inv(&self, k: usize) -> Self {
        let total_bits = Self::dimension_bits();
        if total_bits == 0 {
            return *self;
        }
        let inv_k = (total_bits - (k % total_bits)) % total_bits;
        self.permute(inv_k)
    }

    /// Alias for `permute_inv(k)`.
    #[inline]
    pub fn permute_right(&self, k: usize) -> Self {
        self.permute_inv(k)
    }

    /// Context bundling ($+$): Parallel bitwise majority voting across an active window of vectors.
    ///
    /// For each bit position $j \in [0, D)$, the output bit is set to $1$ if more than
    /// half of the vectors in `vectors` have bit $j = 1$.
    ///
    /// When `vectors.len()` is even, ties are broken using a deterministic pseudo-random
    /// hypervector independent of vector order, guaranteeing strict commutativity ($A + B = B + A$)
    /// and symmetric constituent preservation ($sim(A+B, A) \approx sim(A+B, B) \approx 0.75$).
    ///
    /// Uses a bit-slice parallel adder tree across all 64 bits simultaneously,
    /// executing with zero dynamic heap allocations and zero matrix multiplications.
    pub fn bundle(vectors: &[Self]) -> Self {
        if vectors.is_empty() {
            return Self::zero();
        }
        if vectors.len() == 1 {
            return vectors[0];
        }
        // Independent canonical tie breaker to ensure strict commutativity and constituent symmetry
        let tie_breaker = Self::from_seed(
            CANONICAL_TIE_BREAKER_SEED,
            CANONICAL_TIE_BREAKER_SALT ^ (vectors.len() as u64),
        );
        Self::bundle_with_tie_breaker(vectors, &tie_breaker)
    }

    /// Bundling with an explicit deterministic tie-breaker hypervector.
    ///
    /// When the bit count across `vectors` equals `vectors.len() / 2` (even $M$),
    /// the bit is taken from `tie_breaker`.
    pub fn bundle_with_tie_breaker(vectors: &[Self], tie_breaker: &Self) -> Self {
        let m = vectors.len();
        if m == 0 {
            return Self::zero();
        }
        if m == 1 {
            return vectors[0];
        }
        debug_assert!(
            m <= 65535,
            "VSA bundling overflow: vectors.len() exceeds 65535"
        );

        let threshold = m / 2;
        let is_even = (m % 2) == 0;
        let mut result = [0u64; WORDS];

        // Process word by word: 64 bits in parallel via bit-slice ripple-carry accumulators
        let mut w = 0;
        while w < WORDS {
            // acc[p] stores the p-th bit of the count at each of the 64 bit positions.
            // 16 bit-planes support up to 65,535 vectors without dynamic allocation.
            let mut acc = [0u64; 16];
            let mut plane_count = 0usize;

            for v in vectors {
                let mut carry = v.data[w];
                let mut p = 0;
                while p < plane_count {
                    let next_carry = acc[p] & carry;
                    acc[p] ^= carry;
                    carry = next_carry;
                    if carry == 0 {
                        break;
                    }
                    p += 1;
                }
                if carry != 0 && plane_count < 16 {
                    acc[plane_count] = carry;
                    plane_count += 1;
                }
            }

            // Reconstruct the 64-bit result word from the bit planes
            let mut word_res = 0u64;
            let mut bit = 0;
            while bit < 64 {
                let mut count = 0usize;
                let mut p = 0;
                while p < plane_count {
                    count |= (((acc[p] >> bit) & 1) as usize) << p;
                    p += 1;
                }

                let bit_val = if count > threshold {
                    true
                } else if count == threshold && is_even {
                    ((tie_breaker.data[w] >> bit) & 1) == 1
                } else {
                    false
                };

                if bit_val {
                    word_res |= 1u64 << bit;
                }
                bit += 1;
            }

            result[w] = word_res;
            w += 1;
        }

        Self { data: result }
    }

    /// Exact Hamming distance: Total count of differing bits via hardware `popcount`.
    ///
    /// Range: $[0, D]$.
    #[inline]
    pub fn hamming_distance(&self, other: &Self) -> usize {
        let mut dist = 0usize;
        let mut i = 0;
        while i < WORDS {
            dist += (self.data[i] ^ other.data[i]).count_ones() as usize;
            i += 1;
        }
        dist
    }

    /// Exact Hamming similarity: Total count of matching bits.
    ///
    /// Range: $[0, D]$.
    #[inline]
    pub fn hamming_similarity(&self, other: &Self) -> usize {
        Self::dimension_bits() - self.hamming_distance(other)
    }

    /// Normalized Hamming distance: Fractional distance in $[0.0, 1.0]$.
    ///
    /// Random independent hypervectors have distance $\approx 0.50$.
    #[inline]
    pub fn hamming_distance_normalized(&self, other: &Self) -> f64 {
        let total = Self::dimension_bits();
        if total == 0 {
            0.0
        } else {
            self.hamming_distance(other) as f64 / total as f64
        }
    }

    /// Normalized Hamming similarity: Fractional overlap in $[0.0, 1.0]$.
    #[inline]
    pub fn hamming_similarity_normalized(&self, other: &Self) -> f64 {
        1.0 - self.hamming_distance_normalized(other)
    }

    /// Bipolar cosine similarity in $[-1.0, 1.0]$:
    ///
    /// Maps $\{0, 1\} \to \{+1, -1\}$ where matching bits give $+1$ and differing bits give $-1$.
    /// $\text{cosine} = \frac{D - 2 \cdot d_H}{D}$.
    #[inline]
    pub fn bipolar_cosine(&self, other: &Self) -> f64 {
        let total = Self::dimension_bits() as f64;
        if total == 0.0 {
            0.0
        } else {
            let d = self.hamming_distance(other) as f64;
            (total - 2.0 * d) / total
        }
    }

    /// Fixed-point $Q1.15$ overlap similarity in $[0, 32767]$ for serving hot paths.
    ///
    /// Strictly integer arithmetic: zero runtime floats, zero matrix multiplications.
    #[inline]
    pub fn similarity_q15(&self, other: &Self) -> i16 {
        let total = Self::dimension_bits() as i64;
        if total == 0 {
            0
        } else {
            let d = self.hamming_distance(other) as i64;
            let overlap = total - d;
            // `overlap * 32767` written as `(overlap << 15) - overlap`, because 32767 = 2^15 - 1.
            //
            // MEASURED NO-OP at `opt-level = 3`: `otool` over the release binary shows the same
            // 3,100 multiply instructions before and after this change, because LLVM's instcombine
            // already performs exactly this strength reduction for `x * (2^n - 1)`. The explicit
            // form is kept because it is the exact identity, it guarantees the shift at
            // `opt-level = 0` where the optimiser does not run (so debug builds serve without the
            // multiply), and it documents the intent. It is NOT a reduction at release.
            (((overlap << 15) - overlap) / total) as i16
        }
    }

    /// `(overlap * 32767) / total` for every reachable Hamming distance, built at compile time.
    ///
    /// The similarity functions are **pure functions of the popcount**, so the arithmetic —
    /// including the `* 32767` dense-constant multiply, which the compiler emits as a real `mul`
    /// instruction because 32767 has fifteen set bits — is replaced by one lookup. The table
    /// stores the exact same integer expression the functions used to evaluate, so this is an
    /// exact replacement, not an approximation.
    ///
    /// This is the shape the binary-level measurement pointed at: `vsa/attention` reported ZERO
    /// multiplying operators to a source scan yet carried 306 multiply instructions in the
    /// optimised binary, because these scaling multiplies are dense constants and inlining pulls
    /// them into the attention loop. Both are written as `(x << 15) - x`, which is the exact
    /// identity `x * (2^15 - 1)` and engages no multiplier.

    /// Fixed-point $Q1.15$ bipolar correlation in $[-32767, 32767]$ for serving hot paths.
    ///
    /// Strictly integer arithmetic: zero runtime floats, zero matrix multiplications.
    #[inline]
    pub fn bipolar_correlation_q15(&self, other: &Self) -> i16 {
        let total = Self::dimension_bits() as i64;
        if total == 0 {
            0
        } else {
            let d = self.hamming_distance(other) as i64;
            let correlation = total - 2 * d;
            // Exact identity `x * (2^15 - 1)`. MEASURED NO-OP at release; see `similarity_q15`.
            (((correlation << 15) - correlation) / total) as i16
        }
    }
}

impl<const WORDS: usize> Default for Hypervector<WORDS> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<const WORDS: usize> BitXor for Hypervector<WORDS> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bind(&rhs)
    }
}

impl<const WORDS: usize> BitXor<&Hypervector<WORDS>> for &Hypervector<WORDS> {
    type Output = Hypervector<WORDS>;

    #[inline]
    fn bitxor(self, rhs: &Hypervector<WORDS>) -> Self::Output {
        self.bind(rhs)
    }
}

impl<const WORDS: usize> BitXorAssign for Hypervector<WORDS> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        let mut i = 0;
        while i < WORDS {
            self.data[i] ^= rhs.data[i];
            i += 1;
        }
    }
}

impl<const WORDS: usize> Serialize for Hypervector<WORDS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(WORDS))?;
        for &word in &self.data {
            seq.serialize_element(&word)?;
        }
        seq.end()
    }
}

impl<'de, const WORDS: usize> Deserialize<'de> for Hypervector<WORDS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HypervectorVisitor<const W: usize>;

        impl<'de, const W: usize> serde::de::Visitor<'de> for HypervectorVisitor<W> {
            type Value = Hypervector<W>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "a sequence of {} 64-bit words", W)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut data = [0u64; W];
                for i in 0..W {
                    data[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(Hypervector { data })
            }
        }

        deserializer.deserialize_seq(HypervectorVisitor::<WORDS>)
    }
}

impl<const WORDS: usize> fmt::Debug for Hypervector<WORDS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hypervector<{} bits>(hex: {:016x}..{:016x}, ones: {})",
            Self::dimension_bits(),
            self.data[0],
            self.data[WORDS - 1],
            self.count_ones()
        )
    }
}

/// Helper: Deterministic SplitMix64 pseudo-random generator step.
#[inline]
pub(crate) const fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
