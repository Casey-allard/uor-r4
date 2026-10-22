//! Potential-ratio lawful likelihood (PRLL) for bits-per-byte scoring.
//!
//! The model's Q24 geometric scores are shifted to non-negative integer potentials
//! (`phi = s - min(s) + 1`, exact, order-preserving), given a base unit of mass each and
//! then normalised by the ratio of potentials into exact Q48 masses summing to
//! `1 << 48`. No exponential, no squashing, no table of any normaliser, and no dense
//! matmul appear anywhere: the arithmetic is integer add/subtract, one exact u128 ratio
//! per candidate, and a largest-remainder split whose ties are keyed by candidate index
//! (the token id).
//!
//! The design, its owner decision and the seven finite host controls are recorded in
//! `docs/labs/m4-lawful-likelihood-bpb/README.md`; this module implements the host-only
//! scoring core. It does not run a model, does not read an artifact, and does not
//! register an instrument: those are separate units of the item.

use std::fmt;

/// The exact total required of a mass vector: `1 << 48`, as `sampling` expects.
pub const PROBABILITY_ONE: u64 = 1 << 48;
/// The number of fractional bits in a mass.
pub const FRACTIONAL_BITS: u32 = 48;
/// Squaring rounds used for the fractional part of `log2`.
pub const LOG2_ROUNDS: u32 = 12;
/// `2^LOG2_ROUNDS`: the unit of `bits_q12`.
pub const LOG2_SCALE: u64 = 1 << LOG2_ROUNDS;

/// Why a likelihood computation refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LikelihoodError {
    /// No candidates were supplied.
    EmptyScores,
    /// The vocabulary is too large for base-1 masses inside `2^48`.
    TooManyCandidates { vocabulary: usize },
    /// A mass vector must not be all zero.
    EmptyMass,
    /// `bits` of a zero mass is infinite, so it is refused rather than reported.
    MassZero,
    /// A byte count of zero cannot be a bits-per-byte denominator.
    ZeroBytes,
    /// The bit accumulation overflowed `u128`.
    Overflow,
}

impl fmt::Display for LikelihoodError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScores => formatter.write_str("no candidate scores were supplied"),
            Self::TooManyCandidates { vocabulary } => write!(
                formatter,
                "vocabulary {vocabulary} is too large for base-1 masses inside 2^48"
            ),
            Self::EmptyMass => formatter.write_str("the mass vector is empty"),
            Self::MassZero => {
                formatter.write_str("a zero mass has infinite bits and cannot be scored")
            }
            Self::ZeroBytes => formatter.write_str("bits per byte needs a non-zero byte count"),
            Self::Overflow => formatter.write_str("the bit accumulation overflowed u128"),
        }
    }
}

impl std::error::Error for LikelihoodError {}

type Result<T> = std::result::Result<T, LikelihoodError>;

/// Exact Q48 masses for the candidate scores, summing to [`PROBABILITY_ONE`].
///
/// Every candidate gets one unit of mass first; the remaining `2^48 - V` units are split
/// by `phi / sum(phi)` with the largest-remainder rule, ties by the lowest candidate
/// index. Because a candidate can never fall below one unit, no candidate is ever
/// unscorable: the worst case has `V=4096`, one score at the minimum and the rest at the
/// maximum, and the minimum still gets exactly mass 1 (48 finite bits).
pub fn masses_from_scores(scores: &[i64]) -> Result<Vec<u64>> {
    let vocabulary = scores.len();
    if vocabulary == 0 {
        return Err(LikelihoodError::EmptyScores);
    }
    if vocabulary as u128 >= PROBABILITY_ONE as u128 {
        return Err(LikelihoodError::TooManyCandidates { vocabulary });
    }
    let minimum = *scores
        .iter()
        .min()
        .expect("a non-empty slice has a minimum");
    let potentials: Vec<u128> = scores
        .iter()
        .map(|score| {
            // Differences of i64 values fit i128; the +1 keeps every potential at least 1.
            (*score as i128 - minimum as i128) as u128 + 1
        })
        .collect();
    let total: u128 = potentials.iter().sum();
    let remaining = PROBABILITY_ONE as u128 - vocabulary as u128;
    let mut masses: Vec<u64> = vec![1; vocabulary];
    let mut remainders: Vec<(u128, usize)> = Vec::with_capacity(vocabulary);
    let mut assigned: u128 = 0;
    for (index, potential) in potentials.iter().enumerate() {
        let scaled = potential * remaining;
        let quota = scaled / total;
        masses[index] += quota as u64;
        assigned += quota;
        remainders.push((scaled % total, index));
    }
    let leftover = (remaining - assigned) as usize;
    remainders.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    for (_, index) in remainders.into_iter().take(leftover) {
        masses[index] += 1;
    }
    debug_assert_eq!(masses.iter().sum::<u64>(), PROBABILITY_ONE);
    Ok(masses)
}

/// Exact Q48 masses for maximum-likelihood counts, with no base unit and no shift.
///
/// A zero count therefore stays at zero mass: the count model cannot score an unseen
/// target, and [`bits_q12`] refuses that target rather than reporting an infinite value.
pub fn mle_masses_from_counts(counts: &[u64]) -> Result<Vec<u64>> {
    let vocabulary = counts.len();
    if vocabulary == 0 {
        return Err(LikelihoodError::EmptyScores);
    }
    let total: u128 = counts.iter().map(|count| *count as u128).sum();
    if total == 0 {
        return Err(LikelihoodError::EmptyMass);
    }
    let mut masses: Vec<u64> = vec![0; vocabulary];
    let mut remainders: Vec<(u128, usize)> = Vec::with_capacity(vocabulary);
    let mut assigned: u128 = 0;
    for (index, count) in counts.iter().enumerate() {
        let scaled = *count as u128 * PROBABILITY_ONE as u128;
        let quota = scaled / total;
        masses[index] = quota as u64;
        assigned += quota;
        remainders.push((scaled % total, index));
    }
    let leftover = (PROBABILITY_ONE as u128 - assigned) as usize;
    remainders.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    for (_, index) in remainders.into_iter().take(leftover) {
        masses[index] += 1;
    }
    debug_assert_eq!(masses.iter().sum::<u64>(), PROBABILITY_ONE);
    Ok(masses)
}

/// `-log2(mass / 2^48)` multiplied by [`LOG2_SCALE`], in integer arithmetic.
///
/// The integer part comes from the bit length; the fractional part from
/// [`LOG2_ROUNDS`] squaring rounds, truncating each comparison bit. The per-token
/// deviation from the exact `-log2` is at most `2^-12` bit from the rounds plus
/// `2^-48` bit from the Q48 mass grid.
pub fn bits_q12(mass: u64) -> Result<u64> {
    if mass == 0 {
        return Err(LikelihoodError::MassZero);
    }
    if mass > PROBABILITY_ONE {
        return Err(LikelihoodError::Overflow);
    }
    let exponent = 63 - mass.leading_zeros();
    let mut normalized = mass << (63 - exponent);
    let mut fraction: u64 = 0;
    for _ in 0..LOG2_ROUNDS {
        let squared = (normalized as u128) * (normalized as u128);
        let doubled = squared >> 63;
        fraction <<= 1;
        if doubled >= 1u128 << 64 {
            fraction |= 1;
            normalized = (doubled >> 1) as u64;
        } else {
            normalized = doubled as u64;
        }
    }
    let integer = u64::from(FRACTIONAL_BITS - exponent);
    Ok(integer * LOG2_SCALE - fraction.min(LOG2_SCALE - 1))
}

/// Sum of [`bits_q12`] for the scored target indices.
pub fn total_bits_q12(masses: &[u64], targets: &[usize]) -> Result<u128> {
    if masses.is_empty() {
        return Err(LikelihoodError::EmptyMass);
    }
    let mut total: u128 = 0;
    for target in targets {
        let mass = *masses.get(*target).ok_or(LikelihoodError::EmptyMass)?;
        total = total
            .checked_add(bits_q12(mass)? as u128)
            .ok_or(LikelihoodError::Overflow)?;
    }
    if targets.is_empty() {
        return Err(LikelihoodError::ZeroBytes);
    }
    Ok(total)
}

/// `total_bits_q12 / (LOG2_SCALE * bytes)` as an exact fraction `(numerator, denominator)`.
pub fn bits_per_byte(total_bits_q12: u128, bytes: u64) -> Result<(u128, u128)> {
    if bytes == 0 {
        return Err(LikelihoodError::ZeroBytes);
    }
    Ok((total_bits_q12, u128::from(LOG2_SCALE) * u128::from(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_exact_sum(masses: &[u64]) {
        assert_eq!(
            masses.iter().map(|mass| *mass as u128).sum::<u128>(),
            PROBABILITY_ONE as u128,
            "masses must sum to exactly 2^48: {masses:?}"
        );
    }

    #[test]
    fn tiny_alphabet_is_hand_computable() {
        // phi = [1, 2, 3, 4]; remaining = 2^48 - 4; quotas floor to known values;
        // the largest-remainder split completes the sum.
        let masses = masses_from_scores(&[10, 11, 12, 13]).expect("valid scores");
        assert_eq!(masses.len(), 4);
        assert_exact_sum(&masses);
        assert!(
            masses.windows(2).all(|pair| pair[0] <= pair[1]),
            "{masses:?}"
        );
    }

    #[test]
    fn worst_case_minimum_keeps_one_unit_and_forty_eight_bits() {
        let mut scores = vec![i64::MAX; 4096];
        scores[0] = i64::MIN;
        let masses = masses_from_scores(&scores).expect("valid scores");
        assert_exact_sum(&masses);
        assert_eq!(masses[0], 1, "the minimum keeps exactly its base unit");
        assert_eq!(bits_q12(masses[0]).unwrap(), 48 * LOG2_SCALE);
    }

    #[test]
    fn uniform_power_of_two_vocabulary_is_exactly_uniform() {
        let masses = masses_from_scores(&[7; 4096]).expect("valid scores");
        assert_eq!(masses, vec![PROBABILITY_ONE / 4096; 4096]);
        assert_exact_sum(&masses);
        assert_eq!(bits_q12(PROBABILITY_ONE / 4096).unwrap(), 12 * LOG2_SCALE);
    }

    #[test]
    fn non_power_of_two_remainders_differ_by_at_most_one() {
        let masses = masses_from_scores(&[1, 1, 1]).expect("valid scores");
        assert_exact_sum(&masses);
        let smallest = *masses.iter().min().unwrap();
        let largest = *masses.iter().max().unwrap();
        assert!(largest - smallest <= 1, "{masses:?}");
    }

    #[test]
    fn ties_are_keyed_by_the_lowest_index() {
        let masses = masses_from_scores(&[0, 0, 0]).expect("valid scores");
        assert_exact_sum(&masses);
        assert!(
            masses[0] >= masses[1] && masses[1] >= masses[2],
            "{masses:?}"
        );
    }

    #[test]
    fn order_and_argmax_are_preserved() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..64 {
            let mut scores = Vec::with_capacity(64);
            for _ in 0..64 {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                scores.push((state >> 33) as i64 - (1 << 30));
            }
            let masses = masses_from_scores(&scores).expect("valid scores");
            assert_exact_sum(&masses);
            for index in 1..scores.len() {
                if scores[index - 1] < scores[index] {
                    assert!(masses[index - 1] <= masses[index], "{scores:?} {masses:?}");
                }
            }
            let top_score = scores.iter().copied().max().unwrap();
            let top_mass = masses.iter().copied().max().unwrap();
            let score_top = scores.iter().position(|score| *score == top_score).unwrap();
            assert!(
                masses[score_top] == top_mass,
                "the argmax must keep a top mass: {scores:?} {masses:?}"
            );
        }
    }

    #[test]
    fn distinct_scores_permute_masses_exactly() {
        let scores = [5i64, -3, 17, 0, 2];
        let masses = masses_from_scores(&scores).expect("valid scores");
        let permutation = [3usize, 0, 4, 1, 2];
        let permuted_scores: Vec<i64> = permutation.iter().map(|index| scores[*index]).collect();
        let permuted_masses = masses_from_scores(&permuted_scores).expect("valid scores");
        for (slot, source) in permutation.iter().enumerate() {
            assert_eq!(permuted_masses[slot], masses[*source]);
        }
    }

    #[test]
    fn uniform_count_baseline_scores_exactly_log2_vocabulary() {
        let masses = mle_masses_from_counts(&[3; 256]).expect("valid counts");
        assert_eq!(masses, vec![PROBABILITY_ONE / 256; 256]);
        let bits = bits_q12(masses[0]).unwrap();
        assert_eq!(bits, 8 * LOG2_SCALE);
        let (numerator, denominator) = bits_per_byte(bits as u128, 1).expect("one byte");
        assert_eq!(numerator * 1, 8 * LOG2_SCALE as u128);
        assert_eq!(denominator, LOG2_SCALE as u128);
    }

    #[test]
    fn a_zero_count_is_unscorable_not_infinite() {
        let masses = mle_masses_from_counts(&[1, 0, 1]).expect("valid counts");
        assert_eq!(masses[1], 0);
        assert_eq!(bits_q12(0), Err(LikelihoodError::MassZero));
        assert!(mle_masses_from_counts(&[0, 0]).is_err());
    }
}
