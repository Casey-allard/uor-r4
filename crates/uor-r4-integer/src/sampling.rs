//! Integer token selection for normalized Q48 model output.
//!
//! Categorical selection uses the supplied masses directly (temperature one).
//! It is a new declared policy, not a reproduction of the floating temperature
//! sampler used by historical training evaluators. The source operations on
//! masses and random values are integer comparisons, shifts, bit operations,
//! additions and subtractions. A compiled instruction audit is a separate check.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The exact total required of a normalized Q48 output distribution.
pub const PROBABILITY_ONE: u64 = 1 << 48;
const MAX_DRAW_ATTEMPTS: usize = 128;
const ZERO_SEED_STATE: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SamplePolicy {
    /// Select the largest mass, resolving ties toward the lowest token ID.
    #[default]
    Greedy,
    /// Sample at temperature one. Zero or a value at least the vocabulary size
    /// includes all tokens; one is exactly greedy and consumes no random value.
    /// Equal masses at the top-k boundary prefer the lowest token ID.
    Categorical { top_k: usize },
    /// Calibrated Min-P truncation: filters candidates with mass < min_p_q16 * p_max / 65536.
    /// Prevents long-tail sampling departures while preserving natural variation.
    MinP { top_k: usize, min_p_q16: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplingError {
    EmptyDistribution,
    ZeroTotal,
    TotalOverflow,
    NotNormalized { total: u64 },
    AllocationFailed,
    RejectionLimit,
    InvalidDraw,
}

impl fmt::Display for SamplingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDistribution => formatter.write_str("empty token distribution"),
            Self::ZeroTotal => formatter.write_str("token distribution has zero total mass"),
            Self::TotalOverflow => formatter.write_str("token distribution total overflows u64"),
            Self::NotNormalized { total } => {
                write!(formatter, "token distribution total {total} is not Q48 one")
            }
            Self::AllocationFailed => formatter.write_str("cannot allocate top-k token indices"),
            Self::RejectionLimit => formatter.write_str("integer sampler rejection limit reached"),
            Self::InvalidDraw => formatter.write_str("integer sample is outside retained mass"),
        }
    }
}

impl std::error::Error for SamplingError {}

/// Deterministic xorshift64 generator and reusable top-k index storage.
///
/// This generator is not cryptographic. Seed zero has a documented nonzero
/// replacement because the xorshift recurrence otherwise remains at zero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sampler {
    state: u64,
    #[serde(skip, default = "Vec::new")]
    ranked: Vec<usize>,
}

impl Sampler {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { ZERO_SEED_STATE } else { seed },
            ranked: Vec::new(),
        }
    }

    /// Restore sampler from a previously saved state.
    pub fn from_state(state: u64) -> Self {
        Self {
            state: if state == 0 { ZERO_SEED_STATE } else { state },
            ranked: Vec::new(),
        }
    }

    /// Set sampler state directly.
    pub fn set_state(&mut self, state: u64) {
        self.state = if state == 0 { ZERO_SEED_STATE } else { state };
    }

    /// Validate the complete model distribution before selecting a token.
    /// Invalid input and greedy selection leave the random state unchanged.
    /// Current nonzero state, accepted by `new` to resume the same stream.
    pub fn state(&self) -> u64 {
        self.state
    }

    #[inline(never)]
    pub fn select(
        &mut self,
        probabilities: &[u64],
        policy: SamplePolicy,
    ) -> Result<usize, SamplingError> {
        let best = validate_distribution(probabilities)?;
        let (top_k, min_p_q16) = match policy {
            SamplePolicy::Greedy
            | SamplePolicy::Categorical { top_k: 1 }
            | SamplePolicy::MinP { top_k: 1, .. } => {
                return Ok(best);
            }
            SamplePolicy::Categorical { top_k } => (top_k, 0u32),
            SamplePolicy::MinP { top_k, min_p_q16 } => (top_k, min_p_q16),
        };

        let p_max = probabilities[best];
        let min_p_threshold = exact_min_p_threshold(p_max, min_p_q16);

        if (top_k == 0 || top_k >= probabilities.len()) && min_p_threshold == 0 {
            let draw = self.draw_below(PROBABILITY_ONE)?;
            return select_ticket(probabilities, 0..probabilities.len(), draw);
        }

        self.ranked.clear();
        self.ranked
            .try_reserve(probabilities.len())
            .map_err(|_| SamplingError::AllocationFailed)?;
        self.ranked.extend(0..probabilities.len());
        heapsort_ranked(&mut self.ranked, probabilities);
        if top_k > 0 && top_k < self.ranked.len() {
            self.ranked.truncate(top_k);
        }
        if min_p_threshold > 0 {
            self.ranked
                .retain(|&idx| probabilities[idx] >= min_p_threshold);
            if self.ranked.is_empty() {
                self.ranked.push(best);
            }
        }
        let total = self.ranked.iter().try_fold(0u64, |total, &index| {
            total
                .checked_add(probabilities[index])
                .ok_or(SamplingError::TotalOverflow)
        })?;
        let draw = self.draw_below(total)?;
        select_ticket(probabilities, self.ranked.iter().copied(), draw)
    }

    #[inline(never)]
    pub fn draw_below(&mut self, bound: u64) -> Result<u64, SamplingError> {
        draw_below_with(bound, || self.next_word())
    }

    /// Advance xorshift64 PRNG and return the next pseudorandom word.
    pub fn next_word(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.state = state;
        state
    }

    /// Alias for `next_word` to advance xorshift64 PRNG.
    pub fn xorshift64(&mut self) -> u64 {
        self.next_word()
    }
}

#[inline(never)]
pub fn validate_distribution(probabilities: &[u64]) -> Result<usize, SamplingError> {
    if probabilities.is_empty() {
        return Err(SamplingError::EmptyDistribution);
    }
    let mut best = 0;
    let mut total = 0u64;
    for (index, &mass) in probabilities.iter().enumerate() {
        total = total
            .checked_add(mass)
            .ok_or(SamplingError::TotalOverflow)?;
        if mass > probabilities[best] {
            best = index;
        }
    }
    if total == 0 {
        return Err(SamplingError::ZeroTotal);
    }
    if total != PROBABILITY_ONE {
        return Err(SamplingError::NotNormalized { total });
    }
    Ok(best)
}

fn draw_below_with(
    bound: u64,
    mut next_nonzero_word: impl FnMut() -> u64,
) -> Result<u64, SamplingError> {
    if bound == 0 || bound > PROBABILITY_ONE {
        return Err(SamplingError::InvalidDraw);
    }
    if bound == 1 {
        return Ok(0);
    }
    let bits = u64::BITS - (bound - 1).leading_zeros();
    let mask = (1u64 << bits) - 1;
    for _ in 0..MAX_DRAW_ATTEMPTS {
        let word = next_nonzero_word();
        // A full-period xorshift64 visits every nonzero word. Discard its
        // incomplete first mask-sized block, which lacks zero, so every masked
        // value has the same number of preimages over that generator period.
        if word <= mask {
            continue;
        }
        let candidate = word & mask;
        if candidate < bound {
            return Ok(candidate);
        }
    }
    // Never replace exhausted rejection with a biased modulo/fallback draw.
    Err(SamplingError::RejectionLimit)
}

#[inline(never)]
pub fn select_ticket(
    probabilities: &[u64],
    indices: impl Iterator<Item = usize>,
    mut draw: u64,
) -> Result<usize, SamplingError> {
    for index in indices {
        let mass = probabilities[index];
        if draw < mass {
            return Ok(index);
        }
        draw -= mass;
    }
    Err(SamplingError::InvalidDraw)
}

/// Exact bit-for-bit computation of `floor(p_max * min_p_q16 / 2^16)` using
/// an exact shift-add over the set bits of `min_p_q16`. Strictly zero multiplier instructions.
#[inline(never)]
pub fn exact_min_p_threshold(p_max: u64, min_p_q16: u32) -> u64 {
    if min_p_q16 == 0 {
        return 0;
    }
    let mut sum: u128 = 0;
    let mut bits = min_p_q16;
    while bits != 0 {
        let shift = bits.trailing_zeros();
        sum = sum.wrapping_add((p_max as u128) << shift);
        bits &= bits - 1;
    }
    (sum >> 16) as u64
}

/// Sort candidate indices in descending priority (highest probability first, smaller index on tie).
/// Strictly zero hardware multiplier instructions, zero dividers, zero floats.
#[inline(never)]
pub fn heapsort_ranked(slice: &mut [usize], probabilities: &[u64]) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    let is_less = |a: usize, b: usize| -> bool {
        let pa = probabilities[a];
        let pb = probabilities[b];
        if pa != pb {
            pa < pb
        } else {
            a > b
        }
    };

    let sift_down = |slice: &mut [usize], mut root: usize, n: usize| loop {
        let left = (root << 1) + 1;
        if left >= n {
            break;
        }
        let right = left + 1;
        let mut smallest = root;

        if is_less(slice[left], slice[smallest]) {
            smallest = left;
        }
        if right < n && is_less(slice[right], slice[smallest]) {
            smallest = right;
        }
        if smallest == root {
            break;
        }
        slice.swap(root, smallest);
        root = smallest;
    };

    let mut i = len >> 1;
    while i > 0 {
        i -= 1;
        sift_down(slice, i, len);
    }

    let mut end = len;
    while end > 1 {
        end -= 1;
        slice.swap(0, end);
        sift_down(slice, 0, end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greedy_and_top_one_choose_lowest_tie_without_advancing_generator() {
        let probabilities = [0, PROBABILITY_ONE >> 1, PROBABILITY_ONE >> 1];
        let mut sampler = Sampler::new(17);
        assert_eq!(sampler.select(&probabilities, SamplePolicy::Greedy), Ok(1));
        assert_eq!(
            sampler.select(&probabilities, SamplePolicy::Categorical { top_k: 1 }),
            Ok(1)
        );
        assert_eq!(sampler.state, 17);
    }

    #[test]
    fn malformed_distributions_fail_before_consuming_randomness() {
        let mut sampler = Sampler::new(19);
        let policy = SamplePolicy::Categorical { top_k: 0 };
        assert_eq!(
            sampler.select(&[], policy),
            Err(SamplingError::EmptyDistribution)
        );
        assert_eq!(
            sampler.select(&[0, 0], policy),
            Err(SamplingError::ZeroTotal)
        );
        assert_eq!(
            sampler.select(&[PROBABILITY_ONE - 1], policy),
            Err(SamplingError::NotNormalized {
                total: PROBABILITY_ONE - 1
            })
        );
        assert_eq!(
            sampler.select(&[u64::MAX, 1], policy),
            Err(SamplingError::TotalOverflow)
        );
        assert_eq!(sampler.state, 19);
    }

    #[test]
    fn seeded_categorical_is_repeatable_and_all_token_policies_agree() {
        let probabilities = [
            PROBABILITY_ONE >> 2,
            PROBABILITY_ONE >> 2,
            PROBABILITY_ONE >> 1,
        ];
        let mut first = Sampler::new(0);
        let mut repeated = Sampler::new(0);
        let mut explicit_all = Sampler::new(0);
        for _ in 0..32 {
            let token = first.select(&probabilities, SamplePolicy::Categorical { top_k: 0 });
            assert_eq!(
                token,
                repeated.select(&probabilities, SamplePolicy::Categorical { top_k: 0 })
            );
            assert_eq!(
                token,
                explicit_all.select(
                    &probabilities,
                    SamplePolicy::Categorical { top_k: usize::MAX }
                )
            );
            assert!(matches!(token, Ok(0..=2)));
        }
        assert_ne!(first.state, ZERO_SEED_STATE);
    }

    #[test]
    fn top_k_boundary_prefers_low_tokens_and_excludes_other_mass() {
        let probabilities = [PROBABILITY_ONE >> 2; 4];
        let mut sampler = Sampler::new(23);
        for _ in 0..16 {
            assert!(matches!(
                sampler.select(&probabilities, SamplePolicy::Categorical { top_k: 2 }),
                Ok(0 | 1)
            ));
            assert_eq!(sampler.ranked, [0, 1]);
        }
    }

    #[test]
    fn ticket_intervals_are_exact_and_zero_mass_cannot_be_selected() {
        let probabilities = [0, 2, 0, 3];
        assert_eq!(select_ticket(&probabilities, 0..4, 0), Ok(1));
        assert_eq!(select_ticket(&probabilities, 0..4, 1), Ok(1));
        assert_eq!(select_ticket(&probabilities, 0..4, 2), Ok(3));
        assert_eq!(select_ticket(&probabilities, 0..4, 4), Ok(3));
        assert_eq!(
            select_ticket(&probabilities, 0..4, 5),
            Err(SamplingError::InvalidDraw)
        );
        let mut sampler = Sampler::new(31);
        assert_eq!(
            sampler.select(
                &[0, PROBABILITY_ONE, 0],
                SamplePolicy::Categorical { top_k: 0 }
            ),
            Ok(1)
        );
    }

    #[test]
    fn rejection_skips_out_of_range_and_incomplete_prng_blocks() {
        let mut words = [3, 7, 6].into_iter();
        let mut count = 0;
        let draw = draw_below_with(3, || {
            count += 1;
            words.next().unwrap_or(0)
        });
        assert_eq!(draw, Ok(2));
        assert_eq!(count, 3);
        assert_eq!(draw_below_with(1, || 0), Ok(0));
        assert_eq!(draw_below_with(0, || 0), Err(SamplingError::InvalidDraw));
        assert_eq!(
            draw_below_with(PROBABILITY_ONE + 1, || 0),
            Err(SamplingError::InvalidDraw)
        );
    }

    #[test]
    fn pathological_rejection_returns_a_finite_error() {
        let mut calls = 0;
        assert_eq!(
            draw_below_with(3, || {
                calls += 1;
                7
            }),
            Err(SamplingError::RejectionLimit)
        );
        assert_eq!(calls, MAX_DRAW_ATTEMPTS);
    }

    #[test]
    fn sampler_from_state_and_set_state_resumes_prng_deterministically() {
        let mut original = Sampler::new(42);
        let probabilities = [
            PROBABILITY_ONE >> 2,
            PROBABILITY_ONE >> 2,
            PROBABILITY_ONE >> 1,
        ];
        let policy = SamplePolicy::Categorical { top_k: 2 };
        for _ in 0..5 {
            let _ = original.select(&probabilities, policy);
        }
        let saved_state = original.state();

        let mut restored = Sampler::from_state(saved_state);
        assert_eq!(restored.state(), saved_state);

        for _ in 0..10 {
            let tok1 = original.select(&probabilities, policy).unwrap();
            let tok2 = restored.select(&probabilities, policy).unwrap();
            assert_eq!(tok1, tok2);
        }

        let mut another = Sampler::new(999);
        another.set_state(saved_state);
        assert_eq!(another.state(), saved_state);

        let mut zero_restored = Sampler::from_state(0);
        assert_eq!(zero_restored.state(), ZERO_SEED_STATE);
        zero_restored.set_state(0);
        assert_eq!(zero_restored.state(), ZERO_SEED_STATE);
    }

    #[test]
    fn sampler_and_policy_serde_roundtrip() {
        let mut original = Sampler::new(12345);
        let probabilities = [PROBABILITY_ONE >> 1, PROBABILITY_ONE >> 1];
        let policy = SamplePolicy::Categorical { top_k: 2 };
        let _ = original.select(&probabilities, policy);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Sampler = serde_json::from_str(&json).unwrap();
        assert_eq!(original.state(), deserialized.state());

        let policy_greedy = SamplePolicy::Greedy;
        let json_greedy = serde_json::to_string(&policy_greedy).unwrap();
        assert_eq!(json_greedy, "{\"kind\":\"greedy\"}");
        let de_greedy: SamplePolicy = serde_json::from_str(&json_greedy).unwrap();
        assert_eq!(de_greedy, SamplePolicy::Greedy);

        let json_cat = serde_json::to_string(&policy).unwrap();
        assert_eq!(json_cat, "{\"kind\":\"categorical\",\"top_k\":2}");
        let de_cat: SamplePolicy = serde_json::from_str(&json_cat).unwrap();
        assert_eq!(de_cat, policy);

        let policy_min_p = SamplePolicy::MinP {
            top_k: 40,
            min_p_q16: 3276,
        };
        let json_min_p = serde_json::to_string(&policy_min_p).unwrap();
        assert_eq!(
            json_min_p,
            "{\"kind\":\"min_p\",\"top_k\":40,\"min_p_q16\":3276}"
        );
        let de_min_p: SamplePolicy = serde_json::from_str(&json_min_p).unwrap();
        assert_eq!(de_min_p, policy_min_p);
    }

    #[test]
    fn min_p_filters_tail_departures_deterministically() {
        let p0 = (PROBABILITY_ONE * 90) / 100;
        let p1 = (PROBABILITY_ONE * 9) / 100;
        let p2 = PROBABILITY_ONE - p0 - p1;
        let probabilities = [p0, p1, p2];

        let policy_min_p = SamplePolicy::MinP {
            top_k: 40,
            min_p_q16: 6553,
        };

        let mut sampler = Sampler::new(42);
        for _ in 0..100 {
            let token = sampler.select(&probabilities, policy_min_p).unwrap();
            assert!(
                token == 0 || token == 1,
                "Token 2 (1% tail departure) must never be selected under MinP: got {token}"
            );
        }

        let policy_strict = SamplePolicy::MinP {
            top_k: 40,
            min_p_q16: 13107,
        };
        for _ in 0..50 {
            let token = sampler.select(&probabilities, policy_strict).unwrap();
            assert_eq!(token, 0, "Strict MinP must select only dominant token 0");
        }
    }

    fn old_min_p_threshold(p_max: u64, min_p_q16: u32) -> u64 {
        if min_p_q16 > 0 {
            let high = (p_max >> 16) as u128;
            let low = (p_max & 0xffff) as u128;
            let prod = (high * (min_p_q16 as u128)) + ((low * (min_p_q16 as u128)) >> 16);
            prod as u64
        } else {
            0u64
        }
    }

    #[test]
    fn property_test_exact_min_p_threshold_agrees_with_old_formula() {
        // Edge cases
        let edge_p_max = [
            0u64,
            1,
            2,
            0xffff,
            0x10000,
            0x1ffff,
            PROBABILITY_ONE / 100,
            PROBABILITY_ONE / 2,
            PROBABILITY_ONE,
            (1u64 << 48) - 1,
            1u64 << 60,
            u64::MAX - 1,
            u64::MAX,
        ];
        let edge_min_p = [
            0u32,
            1,
            2,
            0x7fff,
            0x8000,
            0xffff,
            0x10000,
            6553,
            13107,
            32768,
            65535,
            65536,
            0x55555555,
            0xAAAAAAAA,
            u32::MAX - 1,
            u32::MAX,
        ];

        for &p in &edge_p_max {
            for &m in &edge_min_p {
                let exact = exact_min_p_threshold(p, m);
                let old = old_min_p_threshold(p, m);
                assert_eq!(
                    exact, old,
                    "Mismatch on edge case p_max={p}, min_p_q16={m}: exact={exact}, old={old}"
                );
            }
        }

        // Single-bit sweeps (powers of two)
        for i in 0..64 {
            let p = 1u64 << i;
            for j in 0..32 {
                let m = 1u32 << j;
                let exact = exact_min_p_threshold(p, m);
                let old = old_min_p_threshold(p, m);
                assert_eq!(exact, old, "Mismatch on power of two p=2^{i}, m=2^{j}");
            }
        }

        // All-ones bit-prefix sweeps
        for i in 1..=64 {
            let p = if i == 64 { u64::MAX } else { (1u64 << i) - 1 };
            for j in 1..=32 {
                let m = if j == 32 { u32::MAX } else { (1u32 << j) - 1 };
                let exact = exact_min_p_threshold(p, m);
                let old = old_min_p_threshold(p, m);
                assert_eq!(exact, old, "Mismatch on all-ones p={p:#x}, m={m:#x}");
            }
        }

        // PRNG property sweep across 100,000 random values
        let mut rng = 0x123456789abcdef0u64;
        let mut next_u64 = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };

        for _ in 0..100_000 {
            let p = next_u64();
            let m = next_u64() as u32;
            let exact = exact_min_p_threshold(p, m);
            let old = old_min_p_threshold(p, m);
            assert_eq!(
                exact, old,
                "Mismatch on random p_max={p}, min_p_q16={m}: exact={exact}, old={old}"
            );
        }
    }

    #[test]
    fn test_heapsort_ranked_matches_std_sort() {
        // Test edge cases: empty, single element, identical probabilities, reversed
        let test_cases: Vec<Vec<u64>> = vec![
            vec![],
            vec![100],
            vec![100, 200],
            vec![200, 100],
            vec![50, 50, 50, 50],
            vec![10, 30, 20, 50, 40],
            vec![1, 1000, 1, 1000, 500, 250, 750],
        ];

        for probs in test_cases {
            let mut expected: Vec<usize> = (0..probs.len()).collect();
            expected.sort_unstable_by(|&left, &right| {
                probs[right]
                    .cmp(&probs[left])
                    .then_with(|| left.cmp(&right))
            });

            let mut actual: Vec<usize> = (0..probs.len()).collect();
            heapsort_ranked(&mut actual, &probs);

            assert_eq!(
                actual, expected,
                "Failed on deterministic test case: {probs:?}"
            );
        }

        // PRNG fuzz test with duplicate probabilities across random distributions
        let mut rng = 0x9876543210fedcbau64;
        let mut next_u64 = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };

        for size in [8, 16, 64, 256, 512, 1024] {
            for _ in 0..50 {
                let probs: Vec<u64> = (0..size).map(|_| next_u64() % 100).collect();
                let mut expected: Vec<usize> = (0..size).collect();
                expected.sort_unstable_by(|&left, &right| {
                    probs[right]
                        .cmp(&probs[left])
                        .then_with(|| left.cmp(&right))
                });

                let mut actual: Vec<usize> = (0..size).collect();
                heapsort_ranked(&mut actual, &probs);

                assert_eq!(
                    actual, expected,
                    "Mismatch on random distribution of size {size}"
                );
            }
        }
    }
}
