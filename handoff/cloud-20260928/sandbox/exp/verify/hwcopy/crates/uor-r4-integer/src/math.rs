//! Checked integer arithmetic for the full-context recurrent execution bridge.
//!
//! The declared numerical primitives below use comparisons, additions,
//! subtractions, bit operations and bounded loops. They contain no floating
//! point, multiplication or division operators. This source property does not
//! establish the instruction mix of a particular compiled binary.
//!
//! Signed dyadic rescaling and signed division round to nearest with ties away
//! from zero, matching the quantized interface convention. Multiplication and
//! division are exact before that explicit rounding. Square roots round down;
//! callers should retain fractional guard bits before a normalization division.
//! All arithmetic overflow is an error, never wrapping or implicit saturation.

use std::fmt;

/// Failures at the numerical boundary; callers decide whether to reject an
/// artifact or a model step. Saturation belongs at explicit model interfaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerMathError {
    Overflow,
    DivisionByZero,
    InvalidShift,
}

impl fmt::Display for IntegerMathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Overflow => "integer bridge arithmetic overflow",
            Self::DivisionByZero => "integer bridge division by zero",
            Self::InvalidShift => "integer bridge shift outside its supported range",
        })
    }
}

impl std::error::Error for IntegerMathError {}

pub type MathResult<T> = std::result::Result<T, IntegerMathError>;

fn signed_magnitude(magnitude: u128, negative: bool) -> MathResult<i128> {
    if negative && magnitude == (1u128 << 127) {
        return Ok(i128::MIN);
    }
    let value = i128::try_from(magnitude).map_err(|_| IntegerMathError::Overflow)?;
    Ok(if negative { -value } else { value })
}

/// Shift left exactly, rejecting lost high bits. Rust's `checked_shl` alone
/// checks only the shift count, so is insufficient for this contract.
pub fn shift_left_unsigned(value: u128, shift: u32) -> MathResult<u128> {
    if shift >= 128 {
        return Err(IntegerMathError::InvalidShift);
    }
    if value > (u128::MAX >> shift) {
        return Err(IntegerMathError::Overflow);
    }
    Ok(value << shift)
}

/// Exact unsigned product, using at most 128 shift/add iterations. Choosing the
/// smaller operand as multiplier shortens the usual small-code weight path.
pub fn checked_mul_unsigned(mut left: u128, mut right: u128) -> MathResult<u128> {
    if left < right {
        std::mem::swap(&mut left, &mut right);
    }
    let mut result = 0u128;
    while right != 0 {
        if right & 1 != 0 {
            result = result.checked_add(left).ok_or(IntegerMathError::Overflow)?;
        }
        right >>= 1;
        if right != 0 {
            left = shift_left_unsigned(left, 1)?;
        }
    }
    Ok(result)
}

/// Exact signed product, including the representable `MIN` result. The unsigned
/// accumulator preserves its magnitude without applying `abs` to a signed MIN.
pub fn checked_mul(left: i128, right: i128) -> MathResult<i128> {
    let magnitude = checked_mul_unsigned(left.unsigned_abs(), right.unsigned_abs())?;
    signed_magnitude(magnitude, (left < 0) != (right < 0))
}

/// Scale by a power of two. Positive shifts are exact; negative shifts round to
/// nearest, with ties away from zero. Supported shifts are -128 through 127.
/// In particular, `scale_pow2(MIN, -128)` is -1, an exact halfway case.
pub fn scale_pow2(value: i128, shift: i32) -> MathResult<i128> {
    if !(-128..=127).contains(&shift) {
        return Err(IntegerMathError::InvalidShift);
    }
    let negative = value < 0;
    let magnitude = value.unsigned_abs();
    if shift >= 0 {
        return signed_magnitude(shift_left_unsigned(magnitude, shift as u32)?, negative);
    }
    let right = shift.unsigned_abs();
    if right == 128 {
        return signed_magnitude(u128::from(magnitude >= (1u128 << 127)), negative);
    }
    let quotient = magnitude >> right;
    let remainder = magnitude & ((1u128 << right) - 1);
    let half = 1u128 << (right - 1);
    let rounded = quotient
        .checked_add(u128::from(remainder >= half))
        .ok_or(IntegerMathError::Overflow)?;
    signed_magnitude(rounded, negative)
}

/// Exact quotient and remainder by aligned binary long division, at most 128
/// iterations. Alignment ensures every divisor shift fits, including inputs
/// whose top bit is set; no doubled-remainder intermediate can overflow.
pub fn div_rem_unsigned(numerator: u128, denominator: u128) -> MathResult<(u128, u128)> {
    if denominator == 0 {
        return Err(IntegerMathError::DivisionByZero);
    }
    if numerator < denominator {
        return Ok((0, numerator));
    }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut divisor = denominator << shift;
    let mut place = 1u128 << shift;
    let mut remainder = numerator;
    let mut quotient = 0u128;
    while place != 0 {
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        divisor >>= 1;
        place >>= 1;
    }
    Ok((quotient, remainder))
}

/// Signed division rounded to nearest with ties away from zero. Remainder
/// comparison uses the complementary remainder, avoiding a doubled value that
/// could overflow. Division by zero and `MIN / -1` return errors.
pub fn div_round(numerator: i128, denominator: i128) -> MathResult<i128> {
    let divisor = denominator.unsigned_abs();
    let (quotient, remainder) = div_rem_unsigned(numerator.unsigned_abs(), divisor)?;
    let rounded = quotient
        .checked_add(u128::from(remainder >= divisor - remainder))
        .ok_or(IntegerMathError::Overflow)?;
    signed_magnitude(rounded, (numerator < 0) != (denominator < 0))
}

/// Floor square root for the complete u128 domain. The radix-four restoring
/// algorithm executes at most 64 digit steps. The result is at most u64::MAX;
/// its shifted partial-root and trial intermediates remain within u128.
pub fn isqrt(mut remainder: u128) -> u128 {
    let mut root = 0u128;
    let mut bit = 1u128 << 126;
    while bit > remainder {
        bit >>= 2;
    }
    while bit != 0 {
        let trial = root + bit;
        if remainder >= trial {
            remainder -= trial;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

/// Floor of `sqrt(value)` with `fractional_bits` fractional binary digits.
/// The radicand is widened before the root: there is no early integer-root
/// precision loss. The shift must fit in u128 and the bit count be at most 63.
/// For example, a sum of 256 squared signed-16-bit codes admits 32 guard bits.
pub fn sqrt_fixed(value: u128, fractional_bits: u32) -> MathResult<u128> {
    if fractional_bits > 63 {
        return Err(IntegerMathError::InvalidShift);
    }
    Ok(isqrt(shift_left_unsigned(value, fractional_bits << 1)?))
}

/// Exact squared norm with a checked u128 accumulator. This includes i64::MIN;
/// the caller must still bound the number and magnitude of coordinates.
pub fn sum_squares(values: &[i64]) -> MathResult<u128> {
    values.iter().try_fold(0u128, |sum, &value| {
        let magnitude = u128::from(value.unsigned_abs());
        let square = checked_mul_unsigned(magnitude, magnitude)?;
        sum.checked_add(square).ok_or(IntegerMathError::Overflow)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_product_preserves_sign_and_extrema() -> MathResult<()> {
        for (left, right, expected) in [
            (7, -9, -63),
            (-7, -9, 63),
            (i128::MIN, 0, 0),
            (i128::MIN, 1, i128::MIN),
            (i128::MAX, -1, -i128::MAX),
            (-(1i128 << 126), 2, i128::MIN),
        ] {
            assert_eq!(checked_mul(left, right)?, expected);
        }
        assert_eq!(checked_mul(i128::MIN, -1), Err(IntegerMathError::Overflow));
        assert_eq!(checked_mul(i128::MAX, 2), Err(IntegerMathError::Overflow));
        Ok(())
    }

    #[test]
    fn unsigned_product_rejects_lost_high_bits() -> MathResult<()> {
        for (left, right) in [
            (0, u128::MAX),
            (u128::MAX, 1),
            (123456789, 987654321),
            (1u128 << 100, 1u128 << 27),
            (u128::MAX, 2),
            (1u128 << 127, 2),
        ] {
            assert_eq!(
                checked_mul_unsigned(left, right).ok(),
                left.checked_mul(right)
            );
        }
        assert_eq!(
            shift_left_unsigned(u128::MAX, 1),
            Err(IntegerMathError::Overflow)
        );
        assert_eq!(
            shift_left_unsigned(1, 128),
            Err(IntegerMathError::InvalidShift)
        );
        Ok(())
    }

    #[test]
    fn dyadic_rounding_uses_away_ties_without_overflow() -> MathResult<()> {
        for (input, shift, expected) in [
            (5, -1, 3),
            (-5, -1, -3),
            (9, -2, 2),
            (-9, -2, -2),
            (i128::MIN, -127, -1),
            (i128::MIN, -128, -1),
            (i128::MAX, -128, 0),
            (-1, 127, i128::MIN),
        ] {
            assert_eq!(scale_pow2(input, shift)?, expected);
        }
        assert_eq!(scale_pow2(1, 127), Err(IntegerMathError::Overflow));
        assert_eq!(scale_pow2(0, -129), Err(IntegerMathError::InvalidShift));
        Ok(())
    }

    #[test]
    fn long_division_covers_top_bit_operands() -> MathResult<()> {
        for (numerator, denominator) in [
            (0, 1),
            (7, 11),
            (100, 9),
            (u128::MAX, 1),
            (u128::MAX, u128::MAX),
            (u128::MAX, (1u128 << 127) + 1),
            (u128::MAX, 3),
        ] {
            assert_eq!(
                div_rem_unsigned(numerator, denominator)?,
                (numerator / denominator, numerator % denominator)
            );
        }
        assert_eq!(
            div_rem_unsigned(1, 0),
            Err(IntegerMathError::DivisionByZero)
        );
        Ok(())
    }

    #[test]
    fn division_rounds_signed_halfway_cases() -> MathResult<()> {
        for (numerator, denominator, expected) in [
            (5, 2, 3),
            (-5, 2, -3),
            (5, -2, -3),
            (-5, -2, 3),
            (4, 3, 1),
            (5, 3, 2),
            (i128::MIN, i128::MIN, 1),
            (i128::MIN, 1, i128::MIN),
        ] {
            assert_eq!(div_round(numerator, denominator)?, expected);
        }
        assert_eq!(div_round(i128::MIN, -1), Err(IntegerMathError::Overflow));
        assert_eq!(div_round(0, 0), Err(IntegerMathError::DivisionByZero));
        Ok(())
    }

    #[test]
    fn restoring_square_root_obeys_floor_bounds() {
        for value in [
            0,
            1,
            2,
            3,
            4,
            15,
            16,
            17,
            (1u128 << 126) - 1,
            1u128 << 126,
            u128::MAX,
        ] {
            let root = isqrt(value);
            assert!(root * root <= value);
            if let Some(next_square) = (root + 1).checked_mul(root + 1) {
                assert!(next_square > value);
            }
        }
        assert_eq!(isqrt(u128::MAX), u128::from(u64::MAX));
    }

    #[test]
    fn fixed_square_root_preserves_fractional_guard_bits() -> MathResult<()> {
        assert_eq!(sqrt_fixed(2, 32)?, 6_074_000_999);
        assert_eq!(sqrt_fixed(25, 24)?, 5 << 24);
        assert_eq!(sqrt_fixed(1, 63)?, 1 << 63);
        assert_eq!(sqrt_fixed(4, 63), Err(IntegerMathError::Overflow));
        assert_eq!(sqrt_fixed(0, 64), Err(IntegerMathError::InvalidShift));
        Ok(())
    }

    #[test]
    fn normalized_coordinates_and_square_sum_check_bounds() -> MathResult<()> {
        let squared = sum_squares(&[3, 4])?;
        let denominator =
            i128::try_from(sqrt_fixed(squared, 24)?).map_err(|_| IntegerMathError::Overflow)?;
        assert_eq!(div_round(scale_pow2(3, 38)?, denominator)?, 9830);
        assert_eq!(div_round(scale_pow2(4, 38)?, denominator)?, 13107);
        assert_eq!(sum_squares(&[i64::MIN])?, 1 << 126);
        assert_eq!(sum_squares(&[i64::MIN; 4]), Err(IntegerMathError::Overflow));
        Ok(())
    }
}
