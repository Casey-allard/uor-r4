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
    /// Two expert vectors of a mixture cover different vocabularies.
    LengthMismatch { counts: usize, unigram: usize },
    /// A mixed score does not fit the `u64` that [`mle_masses_from_counts`] takes.
    ScoreOverflow,
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
            Self::LengthMismatch { counts, unigram } => write!(
                formatter,
                "the counts cover {counts} candidates but the order-0 counts cover {unigram}"
            ),
            Self::ScoreOverflow => formatter.write_str("a mixed score does not fit u64"),
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

/// Integer mixing weights for [`floor_mixed_masses`].
///
/// Each field is a numerator over the common denominator `count + unigram + floor`, so the
/// weights are exact rational shares and need no normalising step of their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorMixWeights {
    /// Share of the count model's own counts.
    pub count: u64,
    /// Share of the order-0 (unigram) counts.
    pub unigram: u64,
    /// Share of the uniform floor: one count per candidate.
    pub floor: u64,
}

/// Exact Q48 masses, each at least 1, from a count vector mixed with an order-0 expert and a
/// uniform floor.
///
/// Each expert becomes exact Q48 masses through [`mle_masses_from_counts`]; the uniform
/// floor is one count per candidate. An expert whose weight is 0, or whose counts are all 0
/// (an unseen context), contributes nothing. The mixed score `count*mc + unigram*mu +
/// floor*mf` is formed in `u128` and goes through [`mle_masses_from_counts`] once more.
/// Nothing is shifted, so the floor's share is never cancelled: the scores sum to
/// `d * 2^48`, with `d` the sum of the contributing weights, and each mass is the exact
/// ratio `score / d` split by the largest-remainder rule, ties by the lowest index.
///
/// With `floor >= 1` every candidate's score is at least `floor * floor(2^48 / V)`, so its
/// mass is at least 1 whenever `floor * floor(2^48 / V) >= d`. A result with any zero mass
/// is refused with [`LikelihoodError::MassZero`] rather than returned, so every `Ok` result
/// gives each target a finite [`bits_q12`].
///
/// It refuses, and never wraps or panics, on: vectors of different lengths
/// ([`LikelihoodError::LengthMismatch`]); an empty vocabulary
/// ([`LikelihoodError::EmptyScores`]); no contributing expert
/// ([`LikelihoodError::EmptyMass`]); a mixed score above `u64::MAX`, which needs a weight
/// sum of at least `2^16` ([`LikelihoodError::ScoreOverflow`]); and a zero mass. Counts up
/// to `u64::MAX` are exact, since their sums are taken in `u128`.
pub fn floor_mixed_masses(
    counts: &[u64],
    unigram: &[u64],
    weights: FloorMixWeights,
) -> Result<Vec<u64>> {
    let vocabulary = counts.len();
    if unigram.len() != vocabulary {
        return Err(LikelihoodError::LengthMismatch {
            counts: vocabulary,
            unigram: unigram.len(),
        });
    }
    if vocabulary == 0 {
        return Err(LikelihoodError::EmptyScores);
    }
    let mut scores: Vec<u128> = vec![0; vocabulary];
    add_expert(&mut scores, weights.count, counts)?;
    add_expert(&mut scores, weights.unigram, unigram)?;
    add_expert(&mut scores, weights.floor, &vec![1; vocabulary])?;
    let scores = scores
        .into_iter()
        .map(|score| u64::try_from(score).map_err(|_| LikelihoodError::ScoreOverflow))
        .collect::<Result<Vec<u64>>>()?;
    let masses = mle_masses_from_counts(&scores)?;
    if masses.contains(&0) {
        return Err(LikelihoodError::MassZero);
    }
    Ok(masses)
}

/// Adds `weight` times the expert's exact Q48 masses to `scores`.
///
/// Each product is below `2^64 * 2^48 = 2^112` and three of them sum below `2^114`, so the
/// `u128` additions cannot overflow.
fn add_expert(scores: &mut [u128], weight: u64, counts: &[u64]) -> Result<()> {
    if weight == 0 || counts.iter().all(|count| *count == 0) {
        return Ok(());
    }
    let masses = mle_masses_from_counts(counts)?;
    for (score, mass) in scores.iter_mut().zip(masses) {
        *score += u128::from(weight) * u128::from(mass);
    }
    Ok(())
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

    const FIXTURE_WEIGHTS: FloorMixWeights = FloorMixWeights {
        count: 1,
        unigram: 2,
        floor: 1,
    };

    #[test]
    fn floor_mix_fixture_is_hand_computed() {
        // Worked by hand; 2^48 = 281474976710656, 2^46 = 70368744177664.
        //
        // Count expert [3, 1, 0, 0], total 4: 3*2^48/4 and 2^48/4 are exact, so
        //   mc = [211106232532992, 70368744177664, 0, 0].
        // Order-0 expert [5, 3, 1, 1], total 10: k*2^48 / 10 floors to
        //   [140737488355328, 84442493013196, 28147497671065, 28147497671065],
        //   remainders (k*2^48 mod 10) [0, 8, 6, 6]; the floors sum to 2^48 - 2, so the two
        //   largest remainders (index 1, then index 2 on the 6-tie) get one unit each:
        //   mu = [140737488355328, 84442493013197, 28147497671066, 28147497671065].
        // Floor expert [1, 1, 1, 1]: mf = [2^46; 4].
        // Weights (1, 2, 1), d = 4. Score = mc + 2*mu + mf:
        //   s0 = 211106232532992 + 281474976710656 + 70368744177664 = 562949953421312
        //   s1 =  70368744177664 + 168884986026394 + 70368744177664 = 309622474381722
        //   s2 =               0 +  56294995342132 + 70368744177664 = 126663739519796
        //   s3 =               0 +  56294995342130 + 70368744177664 = 126663739519794
        //   sum = 1125899906842624 = 4 * 2^48.
        // Second pass: s*2^48 / (4*2^48) = s / 4, remainder ordered by s mod 4:
        //   floors [140737488355328, 77405618595430, 31665934879949, 31665934879948],
        //   s mod 4 = [0, 2, 0, 2]; the floors sum to 2^48 - 1, so index 1 (the lowest on
        //   the 2-tie) gets the last unit.
        // Masses = [140737488355328, 77405618595431, 31665934879949, 31665934879948]:
        //   the exact shares 1/2, 0.275, 0.1125, 0.1125 on the Q48 grid.
        let masses =
            floor_mixed_masses(&[3, 1, 0, 0], &[5, 3, 1, 1], FIXTURE_WEIGHTS).expect("valid mix");
        assert_eq!(
            masses,
            vec![
                140_737_488_355_328,
                77_405_618_595_431,
                31_665_934_879_949,
                31_665_934_879_948
            ]
        );
        assert_exact_sum(&masses);
        // Bits for targets [0, 1, 2], by hand:
        //   m0 = 2^47: bit length gives 48 - 47 = 1 bit, fraction 0 -> 4096.
        //   m1 in [2^46, 2^47): integer 2, mantissa m1 / 2^46 = 1.1000000000000085;
        //     log2(1.1) = 0.1375035237, * 4096 = 563.21 -> 12 bits 563;
        //     2*4096 - 563 = 7629 (exact -log2(0.275) * 4096 = 7628.79).
        //   m2 in [2^44, 2^45): integer 4, mantissa m2 / 2^44 = 1.8000000000000682;
        //     log2(1.8) = 0.8479969066, * 4096 = 3473.40 -> 3473;
        //     4*4096 - 3473 = 12911 (exact -log2(0.1125) * 4096 = 12910.60).
        //   total = 4096 + 7629 + 12911 = 24636; over 3 bytes the fraction is
        //   24636 / (4096 * 3) = 24636 / 12288.
        let total = total_bits_q12(&masses, &[0, 1, 2]).expect("every target is scorable");
        assert_eq!(total, 24_636);
        assert_eq!(bits_per_byte(total, 3), Ok((24_636, 12_288)));
    }

    #[test]
    fn floor_mix_gives_every_target_a_unit_and_finite_bits() {
        // Index 4 is unseen by the counts and by the order-0 expert: only the floor holds it.
        let counts = [3u64, 1, 0, 0, 0];
        let unigram = [5u64, 3, 1, 1, 0];
        for weights in [
            FIXTURE_WEIGHTS,
            FloorMixWeights {
                count: 60_000,
                unigram: 5_000,
                floor: 1,
            },
            FloorMixWeights {
                count: 1,
                unigram: 0,
                floor: 1,
            },
        ] {
            let masses = floor_mixed_masses(&counts, &unigram, weights).expect("floor holds");
            assert_exact_sum(&masses);
            for (target, mass) in masses.iter().enumerate() {
                assert!(*mass >= 1, "target {target} has zero mass: {masses:?}");
                assert!(bits_q12(*mass).is_ok(), "target {target}: {masses:?}");
            }
            let all: Vec<usize> = (0..counts.len()).collect();
            assert!(total_bits_q12(&masses, &all).is_ok());
        }
        // A whole unseen context backs off to the order-0 expert and the floor.
        let masses = floor_mixed_masses(&[0; 5], &unigram, FIXTURE_WEIGHTS).expect("backs off");
        assert_exact_sum(&masses);
        assert!(masses.iter().all(|mass| *mass >= 1), "{masses:?}");
    }

    #[test]
    fn floor_mix_with_count_weight_only_is_the_existing_mle() {
        let counts = [7u64, 1, 2, 9, 4];
        let unigram = [0u64, 0, 5, 0, 1];
        for count in [1u64, 3, 65_535] {
            let weights = FloorMixWeights {
                count,
                unigram: 0,
                floor: 0,
            };
            assert_eq!(
                floor_mixed_masses(&counts, &unigram, weights),
                mle_masses_from_counts(&counts)
            );
        }
    }

    #[test]
    fn floor_mix_extremes_are_exact_or_refused_never_a_panic() {
        // Counts near u64::MAX are exact: sums are u128, products stay below 2^114.
        let masses = floor_mixed_masses(&[u64::MAX; 4], &[u64::MAX, 1, 1, 1], FIXTURE_WEIGHTS)
            .expect("huge counts are exact");
        assert_exact_sum(&masses);
        assert!(masses.iter().all(|mass| *mass >= 1), "{masses:?}");
        // Equal counts and equal order-0 weight on indices 1..3: their masses differ by at most
        // the one rounding unit, and index 0 (order-0 weight u64::MAX) carries the most.
        let (low, high) = (
            masses[1..].iter().min().copied().unwrap_or(0),
            masses[1..].iter().max().copied().unwrap_or(0),
        );
        assert!(high - low <= 1, "{masses:?}");
        assert!(masses[0] > high, "{masses:?}");
        let masses = floor_mixed_masses(
            &[u64::MAX, 0, u64::MAX - 1, 0],
            &[u64::MAX; 4],
            FIXTURE_WEIGHTS,
        )
        .expect("huge counts are exact");
        assert_exact_sum(&masses);
        // Weights near u64::MAX make a score above u64::MAX: refused.
        let huge = FloorMixWeights {
            count: u64::MAX,
            unigram: u64::MAX,
            floor: u64::MAX,
        };
        assert_eq!(
            floor_mixed_masses(&[u64::MAX; 4], &[u64::MAX; 4], huge),
            Err(LikelihoodError::ScoreOverflow)
        );
        // A weight sum of 2^16 on a single-candidate vocabulary overflows by one unit.
        let edge = FloorMixWeights {
            count: 1 << 15,
            unigram: 1 << 15,
            floor: 0,
        };
        assert_eq!(
            floor_mixed_masses(&[1], &[1], edge),
            Err(LikelihoodError::ScoreOverflow)
        );
        // A weight sum just under 2^16, on one seen target in 2^13: the floor still holds a
        // unit for every unseen target.
        let mut counts = vec![0u64; 1 << 13];
        counts[0] = u64::MAX;
        let heavy = FloorMixWeights {
            count: 65_534,
            unigram: 0,
            floor: 1,
        };
        let masses = floor_mixed_masses(&counts, &counts, heavy).expect("the floor holds");
        assert_exact_sum(&masses);
        assert!(masses.iter().all(|mass| *mass >= 1));
        // A floor of weight 1 under a count weight of 2^32 on V = 2^17: the floor's share on
        // the one unseen target rounds away, and the zero mass is refused, not returned.
        let mut spread = vec![1u64; 1 << 17];
        spread[0] = 0;
        let swamped = FloorMixWeights {
            count: 1 << 32,
            unigram: 0,
            floor: 1,
        };
        assert_eq!(
            floor_mixed_masses(&spread, &spread, swamped),
            Err(LikelihoodError::MassZero)
        );
        // With no floor, a zero mass is refused rather than returned.
        let no_floor = FloorMixWeights {
            count: 65_000,
            unigram: 0,
            floor: 0,
        };
        assert_eq!(
            floor_mixed_masses(&[1, 0, 0, 0], &[1; 4], no_floor),
            Err(LikelihoodError::MassZero)
        );
        // Shape refusals.
        assert_eq!(
            floor_mixed_masses(&[1, 2], &[1], FIXTURE_WEIGHTS),
            Err(LikelihoodError::LengthMismatch {
                counts: 2,
                unigram: 1
            })
        );
        assert_eq!(
            floor_mixed_masses(&[], &[], FIXTURE_WEIGHTS),
            Err(LikelihoodError::EmptyScores)
        );
        let none = FloorMixWeights {
            count: 0,
            unigram: 0,
            floor: 0,
        };
        assert_eq!(
            floor_mixed_masses(&[1, 2], &[1, 2], none),
            Err(LikelihoodError::EmptyMass)
        );
    }
}
