//! Exact signed H4 root classification for a common-scale integer R4 lane.
//!
//! This standalone component classifies mathematical roots, not rounded F32
//! or Q30 root coordinates. It is not connected to a model or reader. The
//! numerical path uses only integer comparisons, shifts, additions/subtractions
//! and the existing checked shift/add product; a compiled opcode audit is a
//! separate question. No normalization, allocation or runtime float is needed.

use std::cmp::Ordering;

use crate::math::{checked_mul_unsigned, IntegerMathError, MathResult};

/// Exact roots in the historical `learner::embedding` enumeration, rather than
/// the lexicographically sorted canonical closure order. Each coordinate pair
/// `[a, b]` denotes `(a + b*phi)/2`. Index 0 is -1 and index 1 is the identity +1.
///
/// The 960-byte table is evaluated at compile time. Its donor is
/// `uor-r4-core::native_geometric::learner::prefix_artifact::historical_roots`
/// (source d15360527f7c69ac8b83eef0bbd5839b87c26f02).
pub const H4_ROOT_COEFFICIENTS: [[[i8; 2]; 4]; 120] = historical_root_coefficients();

// Output axis -> base coordinate. Shared by construction and family selection;
// base coordinates 1, 2, 3 carry the independent signs s1, s2, s3 respectively.
const EVEN_PERMUTATIONS: [[usize; 4]; 12] = [
    [0, 1, 2, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 2, 1, 0],
];

const fn historical_root_coefficients() -> [[[i8; 2]; 4]; 120] {
    let mut roots = [[[0i8; 2]; 4]; 120];
    let mut index = 0;
    let mut axis = 0;
    while axis < 4 {
        roots[index][axis] = [-2, 0];
        roots[index + 1][axis] = [2, 0];
        index += 2;
        axis += 1;
    }
    let mut signs = 0;
    while signs < 16 {
        axis = 0;
        while axis < 4 {
            roots[index][axis] = [if signs & (1 << axis) == 0 { -1 } else { 1 }, 0];
            axis += 1;
        }
        signs += 1;
        index += 1;
    }
    let mut permutation = 0;
    while permutation < 12 {
        // The donor nests s1, s2, s3, each in [-1, +1]: s3 changes fastest.
        signs = 0;
        while signs < 8 {
            let s1 = if signs & 4 == 0 { -1 } else { 1 };
            let s2 = if signs & 2 == 0 { -1 } else { 1 };
            let s3 = if signs & 1 == 0 { -1 } else { 1 };
            let signed = [[0, 0], [s1, 0], [0, s2], [-s3, s3]];
            axis = 0;
            while axis < 4 {
                roots[index][axis] = signed[EVEN_PERMUTATIONS[permutation][axis]];
                axis += 1;
            }
            index += 1;
            signs += 1;
        }
        permutation += 1;
    }
    roots
}

// The 120 roots partition into axes, signed halves, and twelve blocks of eight
// independent signs on positive magnitudes 1, phi, phi-1. Retain each family's
// lowest-index maximizer; signs on zero coordinates are negative (bit zero).
// Candidate order follows disjoint historical index blocks, so strict-greater
// comparison between their winners also preserves the global lowest-index tie.
fn family_candidates(lane: &[i32; 4]) -> [u8; 14] {
    let magnitudes = lane.map(|value| i64::from(value).unsigned_abs());
    let positive = lane.map(|value| u8::from(value > 0));
    let mut candidates = [0u8; 14];
    let mut largest_axis = 0;
    for axis in 1..4 {
        if magnitudes[axis] > magnitudes[largest_axis] {
            largest_axis = axis;
        }
    }
    candidates[0] = ((largest_axis as u8) << 1) + positive[largest_axis];
    candidates[1] = 8;
    for (axis, bit) in positive.iter().enumerate() {
        candidates[1] += *bit << axis;
    }
    let positive_mask = candidates[1] - 8;
    for (block, permutation) in EVEN_PERMUTATIONS.iter().enumerate() {
        let signs = golden_sign_bits(positive_mask, permutation);
        candidates[block + 2] = 24 + ((block as u8) << 3) + signs;
    }
    candidates
}

// Consume the four-bit query mask serially. Keep a runtime permutation behind
// this scalar boundary: inlining twelve constant permutations lets LLVM pack
// the signs through vector registers and emit FMOV transfers in the kernel.
#[inline(never)]
fn golden_sign_bits(mut positive_mask: u8, permutation: &[usize; 4]) -> u8 {
    let mut axis = 0;
    let mut signs = 0u8;
    while positive_mask != 0 {
        if positive_mask & 1 != 0 {
            let base_coordinate = permutation[axis];
            if base_coordinate != 0 {
                // Invert output axis -> base coordinate: s1=4, s2=2, s3=1.
                signs |= 1 << (3 - base_coordinate);
            }
        }
        positive_mask >>= 1;
        axis += 1;
    }
    signs
}

// Separate runtime operands keep LLVM from recognizing 4*b_square+b_square as
// a u128 product by five and emitting UMULH. The checked addition is unchanged.
#[inline(never)]
fn checked_add_square_terms(left: u128, right: u128) -> MathResult<u128> {
    left.checked_add(right).ok_or(IntegerMathError::Overflow)
}

// Total for all i8 coefficients and i32 coordinates: the product magnitude is
// at most 128*2^31=2^38. Widen before shifting or negating; unsigned_abs also
// handles i8::MIN. At most eight iterations, and at most two for this table.
#[inline]
fn coefficient_term(coefficient: i8, value: i32) -> i64 {
    let mut remaining = coefficient.unsigned_abs();
    let mut addend = i64::from(value);
    let mut term = 0i64;
    while remaining != 0 {
        if remaining & 1 != 0 {
            term += addend;
        }
        remaining >>= 1;
        if remaining != 0 {
            addend <<= 1;
        }
    }
    if coefficient < 0 {
        -term
    } else {
        term
    }
}

fn score(lane: &[i32; 4], root: &[[i8; 2]; 4]) -> [i64; 2] {
    let mut a = 0i64;
    let mut b = 0i64;
    for (value, coefficient) in lane.iter().zip(root) {
        a += coefficient_term(coefficient[0], *value);
        b += coefficient_term(coefficient[1], *value);
    }
    [a, b]
}

// Compare a+b*phi with zero through 2(a+b*phi)=(2a+b)+b*sqrt(5).
// Callers supply differences of the bounded scores above. For |lane_j|<=M,
// each root has ||a||_1<=4, ||b||_1<=2 and ||2a+b||_1<=8. Differences therefore
// obey |a|<=8M, |b|<=4M and |2a+b|<=16M. Even M=2^31 (including i32::MIN)
// gives rational_square<=2^70 and 5*b_square<2^69, comfortably inside u128.
fn score_difference_order([a, b]: [i64; 2]) -> MathResult<Ordering> {
    let rational = (a << 1) + b;
    if b == 0 {
        return Ok(rational.cmp(&0));
    }
    if rational == 0 {
        return Ok(b.cmp(&0));
    }
    if (rational < 0) == (b < 0) {
        return Ok(rational.cmp(&0));
    }
    let magnitude = u128::from(rational.unsigned_abs());
    let b_magnitude = u128::from(b.unsigned_abs());
    let rational_square = checked_mul_unsigned(magnitude, magnitude)?;
    let b_square = checked_mul_unsigned(b_magnitude, b_magnitude)?;
    let irrational_square = checked_add_square_terms(b_square << 2, b_square)?;
    Ok(match rational_square.cmp(&irrational_square) {
        Ordering::Greater => rational.cmp(&0),
        Ordering::Less => b.cmp(&0),
        // Nonzero integer P,B cannot satisfy P^2=5B^2. Equal is nevertheless
        // handled as a tie rather than introducing a panic/error fallback.
        Ordering::Equal => Ordering::Equal,
    })
}

/// Return the signed nearest mathematical H4 root for a common-scale i32 lane.
///
/// All four coordinates must have the same positive scale, which then cancels
/// from argmax. All i32 values are supported, including MIN. The all-zero lane
/// maps explicitly to identity code 1; every other exact tie selects the lowest
/// historical index. Antipodes are never folded. Widening precedes negation and
/// shifts. For a nonzero lane, normalization by a common positive scalar cannot
/// change this exact argmax; no F32 classifier equivalence is asserted.
/// Fourteen exact family winners require fourteen root-score evaluations and
/// thirteen exact comparisons, rather than scanning all 120 roots. This is an
/// operation-count reduction, not a measured speed claim.
///
/// Errors propagate from checked software arithmetic. The fixed coefficient
/// bounds make arithmetic overflow unreachable for this declared input domain.
pub fn signed_h4_code_i32(lane: [i32; 4]) -> MathResult<u8> {
    if lane == [0; 4] {
        return Ok(1);
    }
    let candidates = family_candidates(&lane);
    let mut best_index = candidates[0];
    let mut best_score = score(&lane, &H4_ROOT_COEFFICIENTS[usize::from(best_index)]);
    for &index in candidates.iter().skip(1) {
        let candidate = score(&lane, &H4_ROOT_COEFFICIENTS[usize::from(index)]);
        let difference = [candidate[0] - best_score[0], candidate[1] - best_score[1]];
        if score_difference_order(difference)? == Ordering::Greater {
            best_index = index;
            best_score = candidate;
        }
    }
    Ok(best_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent exact oracle: use the algebraic norm
    // N(a+b*phi)=a^2+a*b-b^2, not the production sqrt(5) comparison.
    // With opposite signs of a,b, the conjugate a+b*(1-phi) has sign(a).
    // Native i128 products are intentionally confined to the test oracle.
    fn oracle_order([a, b]: [i128; 2]) -> Ordering {
        if a == 0 {
            return b.cmp(&0);
        }
        if b == 0 || (a < 0) == (b < 0) {
            return a.cmp(&0);
        }
        let norm = a * a + a * b - b * b;
        (norm.signum() * a.signum()).cmp(&0)
    }

    fn oracle_score(lane: [i32; 4], root: &[[i8; 2]; 4]) -> [i128; 2] {
        let mut out = [0i128; 2];
        for axis in 0..4 {
            for (part, value) in out.iter_mut().enumerate() {
                *value += i128::from(lane[axis]) * i128::from(root[axis][part]);
            }
        }
        out
    }

    fn oracle_maxima(lane: [i32; 4]) -> Vec<u8> {
        let scores: Vec<_> = H4_ROOT_COEFFICIENTS
            .iter()
            .map(|root| oracle_score(lane, root))
            .collect();
        let mut maxima = vec![0u8];
        let mut best = scores[0];
        for (index, candidate) in scores.iter().enumerate().skip(1) {
            match oracle_order([candidate[0] - best[0], candidate[1] - best[1]]) {
                Ordering::Greater => {
                    maxima.clear();
                    maxima.push(index as u8);
                    best = *candidate;
                }
                Ordering::Equal => maxima.push(index as u8),
                Ordering::Less => {}
            }
        }
        maxima
    }

    #[test]
    fn h4_classifier_zero_axes_and_signed_half_roots() -> MathResult<()> {
        assert_eq!(std::mem::size_of_val(&H4_ROOT_COEFFICIENTS), 960);
        assert_eq!(coefficient_term(i8::MIN, i32::MIN), 1i64 << 38);
        assert_eq!(
            coefficient_term(i8::MIN, i32::MAX),
            -(i64::from(i32::MAX) << 7)
        );
        assert_eq!(signed_h4_code_i32([0; 4])?, 1);
        for axis in 0..4 {
            let mut lane = [0; 4];
            lane[axis] = i32::MAX;
            assert_eq!(signed_h4_code_i32(lane)?, (2 * axis + 1) as u8);
            lane[axis] = i32::MIN;
            assert_eq!(signed_h4_code_i32(lane)?, (2 * axis) as u8);
        }
        for signs in 0..16 {
            let lane = std::array::from_fn(|axis| if signs & (1 << axis) == 0 { -1 } else { 1 });
            assert_eq!(signed_h4_code_i32(lane)?, 8 + signs);
        }
        Ok(())
    }

    #[test]
    fn h4_classifier_exact_ties_keep_lowest_historical_index() -> MathResult<()> {
        assert_eq!(oracle_maxima([1, 1, 0, 0]), vec![62, 63, 86, 87]);
        assert_eq!(signed_h4_code_i32([1, 1, 0, 0])?, 62);
        assert_eq!(oracle_maxima([-1, -1, 0, 0]), vec![56, 57, 80, 81]);
        assert_eq!(signed_h4_code_i32([-1, -1, 0, 0])?, 56);
        assert_eq!(score_difference_order([0, 0])?, Ordering::Equal);
        // Adjacent Fibonacci values nearly cancel in a+b*phi. Their exact
        // algebraic norm is +1; the negative conjugate fixes the negative sign.
        let near_boundary = [-2_971_215_073, 1_836_311_903];
        assert_eq!(score_difference_order(near_boundary)?, Ordering::Less);
        assert_eq!(
            score_difference_order(near_boundary.map(|value| -value))?,
            Ordering::Greater
        );
        Ok(())
    }

    #[test]
    fn h4_classifier_extremes_and_lanes_match_independent_i128_oracle() -> MathResult<()> {
        let mut lanes = Vec::new();
        for a in [-1, 0, 1] {
            for b in [-1, 0, 1] {
                for c in [-1, 0, 1] {
                    for d in [-1, 0, 1] {
                        lanes.push([a, b, c, d]);
                    }
                }
            }
        }
        for signs in 0..16 {
            lanes.push(std::array::from_fn(|axis| {
                if signs & (1 << axis) == 0 {
                    i32::MIN
                } else {
                    i32::MAX
                }
            }));
        }
        lanes.extend([
            [i32::MIN, 0, i32::MAX, 1],
            [1, i32::MIN, -1, i32::MAX],
            [1_836_311_903, 1_134_903_170, -701_408_733, 0],
        ]);
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..128 {
            lanes.push(std::array::from_fn(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as i32
            }));
        }
        for lane in lanes {
            let wanted = if lane == [0; 4] {
                1
            } else {
                oracle_maxima(lane)[0]
            };
            assert_eq!(signed_h4_code_i32(lane)?, wanted, "lane {lane:?}");
        }
        Ok(())
    }
}
