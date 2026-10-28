//! Non-Destructive Carry-Save Bipolar Accumulator for Vector Symbolic Architectures (VSA).
//!
//! Maintains signed 16-bit integer counters (`[[i16; 64]; WORDS]`) per bit coordinate so
//! hypervectors can be superimposed with integer importance weights, updated incrementally
//! in $O(1)$, decayed via multiplier-free dyadic bit shifts, and **exact-reverted**
//! by accumulating with `-weight` (`CHARTER §3.1–§3.3`, `D11`, `D23`, `D30`).
//!
//! Strictly `#![forbid(unsafe_code)]`, zero floating point, zero matrix multiplications,
//! and zero heap allocations.

use super::hypervector::{
    splitmix64_next, Hypervector, CANONICAL_TIE_BREAKER_SALT, CANONICAL_TIE_BREAKER_SEED,
    WORDS_2048, WORDS_4096, WORDS_8192,
};
use serde::{Deserialize, Serialize};

/// 2,048-bit non-destructive bipolar accumulator (`32 × 64` `i16` counters).
pub type BipolarAccumulator2048 = BipolarAccumulator<WORDS_2048>;

/// 4,096-bit non-destructive bipolar accumulator (`64 × 64` `i16` counters).
pub type BipolarAccumulator4096 = BipolarAccumulator<WORDS_4096>;

/// 8,192-bit non-destructive bipolar accumulator (`128 × 64` `i16` counters).
pub type BipolarAccumulator8192 = BipolarAccumulator<WORDS_8192>;

/// Non-destructive signed integer bipolar accumulator of dimension $D = \text{WORDS} \times 64$ bits.
///
/// Each bit position $j \in [0, D)$ is represented by a signed counter `counts[w][b]`.
/// Accumulating a binary [`Hypervector`] with `weight` adds `+weight` where the bit is `1`
/// and `-weight` where the bit is `0`. Because addition in $\mathbb{Z}^D$ is associative
/// and invertible (within `i16` bounds), accumulating `(hv, +w)` followed by `(hv, -w)`
/// restores the exact prior integer counter state with zero drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipolarAccumulator<const WORDS: usize> {
    #[serde(with = "serde_word_planes")]
    pub counts: [[i16; 64]; WORDS],
    pub total_weight: u32,
    pub item_count: u32,
}

mod serde_word_planes {
    use serde::{
        de::{Error as DeError, SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserializer, Serializer,
    };

    pub fn serialize<S, const WORDS: usize>(
        counts: &[[i16; 64]; WORDS],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(WORDS * 64))?;
        for word in counts {
            for &val in word {
                seq.serialize_element(&val)?;
            }
        }
        seq.end()
    }

    pub fn deserialize<'de, D, const WORDS: usize>(
        deserializer: D,
    ) -> Result<[[i16; 64]; WORDS], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PlaneVisitor<const W: usize>;

        impl<'de, const W: usize> Visitor<'de> for PlaneVisitor<W> {
            type Value = [[i16; 64]; W];

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "a flat sequence of {} i16 counters", W * 64)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut out = [[0i16; 64]; W];
                for (w, plane) in out.iter_mut().enumerate() {
                    for (b, slot) in plane.iter_mut().enumerate() {
                        let idx = w * 64 + b;
                        *slot = seq
                            .next_element()?
                            .ok_or_else(|| DeError::invalid_length(idx, &self))?;
                    }
                }
                Ok(out)
            }
        }

        deserializer.deserialize_seq(PlaneVisitor::<WORDS>)
    }
}

impl<const WORDS: usize> Default for BipolarAccumulator<WORDS> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const WORDS: usize> BipolarAccumulator<WORDS> {
    /// Construct a zeroed bipolar accumulator.
    #[inline]
    pub const fn new() -> Self {
        Self {
            counts: [[0i16; 64]; WORDS],
            total_weight: 0,
            item_count: 0,
        }
    }

    /// Total bit dimension ($D = \text{WORDS} \times 64$).
    #[inline]
    pub const fn dimension_bits() -> usize {
        WORDS * 64
    }

    /// Reset all counters and item tracking to zero.
    #[inline]
    pub fn clear(&mut self) {
        self.counts = [[0i16; 64]; WORDS];
        self.total_weight = 0;
        self.item_count = 0;
    }

    /// Immutable reference to the underlying `WORDS × 64` counter array.
    #[inline]
    pub const fn counts(&self) -> &[[i16; 64]; WORDS] {
        &self.counts
    }

    /// Accumulate a dense binary [`Hypervector`] with signed integer `weight`.
    ///
    /// - When `weight > 0`, adds `+weight` for `1`-bits and `-weight` for `0`-bits.
    /// - When `weight < 0`, subtracts the vector's contribution (`-weight` for `0`-bits,
    ///   `+weight` for `1`-bits), enabling exact $O(1)$ retraction or sliding-window eviction.
    /// - Uses an 8-bit unrolled inner loop so LLVM vectorizes to NEON/AVX2 SIMD lanes.
    #[inline]
    pub fn accumulate(&mut self, hv: &Hypervector<WORDS>, weight: i16) {
        if weight == 0 {
            return;
        }
        let neg_weight = weight.saturating_neg();
        for w in 0..WORDS {
            let word = hv.data[w];
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
        }
        self.update_metadata(weight);
    }

    /// Expand a `(seed, salt)` pair directly via SplitMix64 into the accumulator
    /// without materializing an intermediate [`Hypervector`].
    #[inline]
    pub fn accumulate_seed(&mut self, seed: u64, salt: u64, weight: i16) {
        if weight == 0 {
            return;
        }
        let neg_weight = weight.saturating_neg();
        let mut state = seed ^ salt.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        for w in 0..WORDS {
            let word = splitmix64_next(&mut state);
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
        }
        self.update_metadata(weight);
    }

    /// Merge another [`BipolarAccumulator`] scaled by `sign` (`+1` to add, `-1` to subtract).
    #[inline]
    pub fn merge_from(&mut self, other: &Self, subtract: bool) {
        for w in 0..WORDS {
            for b in 0..64 {
                let delta = other.counts[w][b];
                self.counts[w][b] = if subtract {
                    self.counts[w][b].saturating_sub(delta)
                } else {
                    self.counts[w][b].saturating_add(delta)
                };
            }
        }
        if subtract {
            self.total_weight = self.total_weight.saturating_sub(other.total_weight);
            self.item_count = self.item_count.saturating_sub(other.item_count);
        } else {
            self.total_weight = self.total_weight.saturating_add(other.total_weight);
            self.item_count = self.item_count.saturating_add(other.item_count);
        }
    }

    /// Multiplier-free dyadic decay (`c -= c / 2^shift`, rounding toward zero)
    /// for bounded-horizon streaming without floating-point decay factors (D11).
    #[inline]
    pub fn decay_dyadic(&mut self, shift: u32) {
        let clamped_shift = shift.min(14);
        for w in 0..WORDS {
            for b in 0..64 {
                let c = self.counts[w][b];
                let delta = if c >= 0 {
                    c >> clamped_shift
                } else {
                    -((-c) >> clamped_shift)
                };
                self.counts[w][b] = c - delta;
            }
        }
        let w_delta = self.total_weight >> clamped_shift;
        self.total_weight = self.total_weight.saturating_sub(w_delta);
    }

    /// Project the signed counter array onto a binary [`Hypervector`].
    ///
    /// Uses an alternating bit-index parity tie-breaker (`((w * 64 + b) & 1) != 0`)
    /// when `counts[w][b] == 0`, guaranteeing an exact $50\%$ expected Hamming weight
    /// on zero-tied coordinates with zero PRNG overhead.
    #[inline]
    pub fn project(&self) -> Hypervector<WORDS> {
        let mut words = [0u64; WORDS];
        for (w, (word_slot, plane)) in words.iter_mut().zip(self.counts.iter()).enumerate() {
            let mut word = 0u64;
            for (b, &c) in plane.iter().enumerate() {
                let bit = (c > 0) || (c == 0 && (((w << 6) + b) & 1) == 1);
                if bit {
                    word |= 1u64 << b;
                }
            }
            *word_slot = word;
        }
        Hypervector::from_words(words)
    }

    /// Project the signed counter array onto a binary [`Hypervector`] using an
    /// explicit tie-breaker hypervector whenever `counts[w][b] == 0`.
    ///
    /// When all accumulated vectors have unit weight `+1`, this produces bit-identical
    /// results to [`Hypervector::bundle_with_tie_breaker`].
    #[inline]
    pub fn project_with_tie_breaker(&self, tie_breaker: &Hypervector<WORDS>) -> Hypervector<WORDS> {
        let mut words = [0u64; WORDS];
        for (word_slot, (plane, &tb_word)) in words
            .iter_mut()
            .zip(self.counts.iter().zip(tie_breaker.data.iter()))
        {
            let mut word = 0u64;
            for (b, &c) in plane.iter().enumerate() {
                let bit = if c > 0 {
                    true
                } else if c < 0 {
                    false
                } else {
                    ((tb_word >> b) & 1) == 1
                };
                if bit {
                    word |= 1u64 << b;
                }
            }
            *word_slot = word;
        }
        Hypervector::from_words(words)
    }

    /// Project using the canonical VSA tie-breaker seeded by `item_count`.
    #[inline]
    pub fn project_canonical(&self) -> Hypervector<WORDS> {
        let tie_breaker = Hypervector::<WORDS>::from_seed(
            CANONICAL_TIE_BREAKER_SEED,
            CANONICAL_TIE_BREAKER_SALT ^ (self.item_count as u64),
        );
        self.project_with_tie_breaker(&tie_breaker)
    }

    /// Deduplicate `(seed, salt, weight)` feature tuples on stack, keep the top
    /// `MAX_ACTIVE` features by total positive weight, accumulate them via
    /// [`Self::accumulate_seed`], and project to a [`Hypervector`].
    pub fn from_sparsified_seeds<const MAX_UNIQUE: usize, const MAX_ACTIVE: usize>(
        features: &[(u64, u64, i16)],
    ) -> Hypervector<WORDS> {
        let mut unique_seeds = [(0u64, 0u64, 0i16); MAX_UNIQUE];
        let mut num_unique = 0usize;

        for &(seed, salt, weight) in features {
            if weight == 0 {
                continue;
            }
            let mut found = false;
            let mut i = 0usize;
            while i < num_unique {
                if unique_seeds[i].0 == seed && unique_seeds[i].1 == salt {
                    unique_seeds[i].2 = unique_seeds[i].2.saturating_add(weight);
                    found = true;
                    break;
                }
                i += 1;
            }
            if !found && num_unique < MAX_UNIQUE {
                unique_seeds[num_unique] = (seed, salt, weight);
                num_unique += 1;
            }
        }

        // Stable selection sort for the top MAX_ACTIVE highest-weight seeds
        let limit = num_unique.min(MAX_ACTIVE);
        let mut i = 0usize;
        while i < limit {
            let mut best_idx = i;
            let mut j = i + 1;
            while j < num_unique {
                if unique_seeds[j].2 > unique_seeds[best_idx].2 {
                    best_idx = j;
                }
                j += 1;
            }
            if best_idx != i {
                // Preserve relative insertion order among remaining elements when swapping max to `i`
                let best_val = unique_seeds[best_idx];
                let mut k = best_idx;
                while k > i {
                    unique_seeds[k] = unique_seeds[k - 1];
                    k -= 1;
                }
                unique_seeds[i] = best_val;
            }
            i += 1;
        }

        let mut acc = Self::new();
        let mut k = 0usize;
        while k < limit {
            let (seed, salt, w) = unique_seeds[k];
            if w != 0 {
                acc.accumulate_seed(seed, salt, w);
            }
            k += 1;
        }
        acc.project()
    }

    #[inline]
    fn update_metadata(&mut self, weight: i16) {
        if weight > 0 {
            self.total_weight = self.total_weight.saturating_add(weight as u32);
            self.item_count = self.item_count.saturating_add(1);
        } else if weight < 0 {
            let abs_w = weight.unsigned_abs() as u32;
            self.total_weight = self.total_weight.saturating_sub(abs_w);
            self.item_count = self.item_count.saturating_sub(1);
        }
    }
}
