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

    pub fn select(
        &mut self,
        probabilities: &[u64],
        policy: SamplePolicy,
    ) -> Result<usize, SamplingError> {
        let best = validate_distribution(probabilities)?;
        let top_k = match policy {
            SamplePolicy::Greedy | SamplePolicy::Categorical { top_k: 1 } => {
                return Ok(best);
            }
            SamplePolicy::Categorical { top_k } => top_k,
        };

        if top_k == 0 || top_k >= probabilities.len() {
            let draw = self.draw_below(PROBABILITY_ONE)?;
            return select_ticket(probabilities, 0..probabilities.len(), draw);
        }

        self.ranked.clear();
        self.ranked
            .try_reserve(probabilities.len())
            .map_err(|_| SamplingError::AllocationFailed)?;
        self.ranked.extend(0..probabilities.len());
        self.ranked.sort_unstable_by(|&left, &right| {
            probabilities[right]
                .cmp(&probabilities[left])
                .then_with(|| left.cmp(&right))
        });
        self.ranked.truncate(top_k);
        let total = self.ranked.iter().try_fold(0u64, |total, &index| {
            total
                .checked_add(probabilities[index])
                .ok_or(SamplingError::TotalOverflow)
        })?;
        let draw = self.draw_below(total)?;
        select_ticket(probabilities, self.ranked.iter().copied(), draw)
    }

    fn draw_below(&mut self, bound: u64) -> Result<u64, SamplingError> {
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

fn validate_distribution(probabilities: &[u64]) -> Result<usize, SamplingError> {
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

fn select_ticket(
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
    }
}
