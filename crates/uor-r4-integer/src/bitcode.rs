//! Fixed-capacity sign-bit codes of vectors of up to [`BITCODE_LANES`] lanes.
//!
//! A [`BitCode`] keeps one bit per lane: set when the lane is `>= 0`. Two codes
//! are compared by their Hamming distance, the number of lanes whose signs
//! differ. For `+-1` vectors of width `K` differing in `h` lanes the dot product
//! is `K - 2h` and the squared Euclidean distance is `4h`, so the distance
//! carries the whole score of a sign-reduced read.
//!
//! A `BitCode` is a similarity code: close codes come from related vectors and
//! many vectors share one code. It is not an exact identity; exact identity is
//! the job of BLAKE3 digests and prime addresses.
//!
//! The hot functions do not allocate and use no multiply instruction: the
//! popcount is a shift/and/add SWAR ([`popcount_swar`]), not `u64::count_ones`,
//! which lowers to a NEON `cnt` (with an `fmov`) on arm64 and multiplies in its
//! fallback on x86 without POPCNT.

/// The largest number of lanes a code holds.
pub const BITCODE_LANES: usize = 256;
const WORDS: usize = BITCODE_LANES / 64;

/// A sign-bit code: bit `i` of the words is lane `i`; bits at or beyond `len`
/// are zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BitCode {
    words: [u64; WORDS],
    len: u16,
}

/// A vector too long for a [`BitCode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitCodeTooLong(pub usize);

impl std::fmt::Display for BitCodeTooLong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a vector of {} lanes exceeds the {BITCODE_LANES}-lane sign code",
            self.0
        )
    }
}

impl std::error::Error for BitCodeTooLong {}

/// Number of set bits of `x` by shifts, masks and additions (no multiply).
#[inline]
pub fn popcount_swar(x: u64) -> u32 {
    let mut x = x - ((x >> 1) & 0x5555_5555_5555_5555);
    x = (x & 0x3333_3333_3333_3333) + ((x >> 2) & 0x3333_3333_3333_3333);
    x = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f;
    x += x >> 8;
    x += x >> 16;
    x += x >> 32;
    (x & 0x7f) as u32
}

impl BitCode {
    /// The code of `values`: lane `i` is set when `values[i] >= 0`.
    pub fn from_signs(values: &[i32]) -> Result<Self, BitCodeTooLong> {
        if values.len() > BITCODE_LANES {
            return Err(BitCodeTooLong(values.len()));
        }
        let mut words = [0u64; WORDS];
        for (i, &v) in values.iter().enumerate() {
            if v >= 0 {
                words[i >> 6] |= 1u64 << (i & 63);
            }
        }
        Ok(Self {
            words,
            len: values.len() as u16,
        })
    }

    /// Lanes in the code.
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether lane `i` is set (its value was `>= 0`); `false` beyond the length.
    pub fn bit(&self, i: usize) -> bool {
        i < self.len() && (self.words[i >> 6] >> (i & 63)) & 1 == 1
    }

    /// The lanes where exactly one of the codes is set (XOR), over the shorter
    /// length.
    pub fn bind(&self, other: &Self) -> Self {
        let len = self.len.min(other.len);
        let mut words = [0u64; WORDS];
        for (w, word) in words.iter_mut().enumerate() {
            *word = (self.words[w] ^ other.words[w]) & lane_mask(usize::from(len), w);
        }
        Self { words, len }
    }

    /// Hamming distance over the shorter length: the lanes whose signs differ.
    pub fn distance(&self, other: &Self) -> u32 {
        let len = usize::from(self.len.min(other.len));
        let mut total = 0;
        for w in 0..WORDS {
            total += word_distance(self.words[w], other.words[w], lane_mask(len, w));
        }
        total
    }
}

/// The lanes under `mask` where `a` and `b` differ. Not inlined: inlined, the
/// four-word loop of [`BitCode::distance`] is SLP-vectorized into NEON
/// registers on arm64 (`fmov`), outside the D11 serving instruction set.
#[inline(never)]
fn word_distance(a: u64, b: u64, mask: u64) -> u32 {
    popcount_swar((a ^ b) & mask)
}

/// The bits of word `w` that hold lanes below `len`.
#[inline]
fn lane_mask(len: usize, w: usize) -> u64 {
    let start = w << 6;
    if len >= start + 64 {
        u64::MAX
    } else if len <= start {
        0
    } else {
        (1u64 << (len - start)) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_popcount(x: u64) -> u32 {
        (0..64).filter(|b| (x >> b) & 1 == 1).count() as u32
    }

    fn xorshift(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn swar_popcount_matches_a_bit_loop() {
        for x in [0, 1, u64::MAX, 1 << 63, 0x8000_0000_0000_0001] {
            assert_eq!(popcount_swar(x), naive_popcount(x), "{x:#x}");
        }
        let mut state = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..10_000 {
            let x = xorshift(&mut state);
            assert_eq!(popcount_swar(x), naive_popcount(x), "{x:#x}");
        }
    }

    #[test]
    fn distance_counts_differing_signs() {
        let mut state = 0x2545_f491_4f6c_dd1d;
        for len in [0usize, 1, 7, 63, 64, 65, 128, 200, 256] {
            for _ in 0..50 {
                let a: Vec<i32> = (0..len).map(|_| xorshift(&mut state) as i32).collect();
                let b: Vec<i32> = (0..len).map(|_| xorshift(&mut state) as i32).collect();
                let (ca, cb) = (
                    BitCode::from_signs(&a).unwrap(),
                    BitCode::from_signs(&b).unwrap(),
                );
                let naive = a
                    .iter()
                    .zip(&b)
                    .filter(|(x, y)| (**x >= 0) != (**y >= 0))
                    .count() as u32;
                assert_eq!(ca.distance(&cb), naive);
                let bound = ca.bind(&cb);
                assert_eq!(bound.len(), len);
                assert_eq!(
                    bound.distance(&BitCode::from_signs(&vec![-1; len]).unwrap()),
                    naive
                );
                for i in 0..len {
                    assert_eq!(bound.bit(i), (a[i] >= 0) != (b[i] >= 0));
                }
            }
        }
    }

    #[test]
    fn zero_counts_as_non_negative_and_long_vectors_are_refused() {
        let code = BitCode::from_signs(&[0, -1, 5]).unwrap();
        assert!(code.bit(0) && !code.bit(1) && code.bit(2) && !code.bit(3));
        assert_eq!(
            BitCode::from_signs(&[1; BITCODE_LANES + 1]),
            Err(BitCodeTooLong(BITCODE_LANES + 1))
        );
        // Lanes beyond the shorter code are ignored.
        let short = BitCode::from_signs(&[1, 1]).unwrap();
        let long = BitCode::from_signs(&[-1, 1, -1, -1]).unwrap();
        assert_eq!(short.distance(&long), 1);
        assert_eq!(short.bind(&long).len(), 2);
    }
}
