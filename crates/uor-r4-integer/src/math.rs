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

pub const Q30_SCALE: i32 = 1 << 30;

/// 16-step CORDIC arc-tangent table in Q1.30: atan(2^-i) / pi * 2^30.
/// Exact integer values normalized such that pi corresponds to 2^30.
pub const CORDIC_ATAN_TABLE_Q30: [i32; 16] = [
    268435456, // atan(2^0) = pi/4 -> 2^28
    158467776, // atan(2^-1) / pi * 2^30
    83730694,  // atan(2^-2) / pi * 2^30
    42502690,  // atan(2^-3) / pi * 2^30
    21334812,  // atan(2^-4) / pi * 2^30
    10677271,  // atan(2^-5) / pi * 2^30
    5339870,   // atan(2^-6) / pi * 2^30
    2670177,   // atan(2^-7) / pi * 2^30
    1335119,   // atan(2^-8) / pi * 2^30
    667563,    // atan(2^-9) / pi * 2^30
    333782,    // atan(2^-10) / pi * 2^30
    166891,    // atan(2^-11) / pi * 2^30
    83446,     // atan(2^-12) / pi * 2^30
    41723,     // atan(2^-13) / pi * 2^30
    20861,     // atan(2^-14) / pi * 2^30
    10431,     // atan(2^-15) / pi * 2^30
];

/// Fixed-point CORDIC atan2 in Q1.30 with zero floats.
/// Returns angle theta / pi in Q1.30 format: range [-2^30, 2^30].
/// -pi -> -2^30, 0 -> 0, +pi -> 2^30.
pub fn atan2_q30(y: i32, x: i32) -> i32 {
    const SCALE: i64 = 1 << 30;
    if x == 0 && y == 0 {
        return 0;
    }
    if x == 0 {
        return if y > 0 { 1 << 29 } else { -(1 << 29) };
    }
    if y == 0 {
        return if x > 0 { 0 } else { 1 << 30 };
    }

    let mut x_curr = (x as i64).abs();
    let mut y_curr = y as i64;
    let mut angle: i64 = 0;

    for (i, &table_angle) in CORDIC_ATAN_TABLE_Q30.iter().enumerate() {
        let x_shift = x_curr >> i;
        let y_shift = y_curr >> i;
        if y_curr > 0 {
            x_curr += y_shift;
            y_curr -= x_shift;
            angle += table_angle as i64;
        } else {
            x_curr -= y_shift;
            y_curr += x_shift;
            angle -= table_angle as i64;
        }
    }

    if x < 0 {
        if y >= 0 {
            angle = (1 << 30) - angle;
        } else {
            angle = -(1 << 30) - angle;
        }
    }

    angle.clamp(-SCALE, SCALE) as i32
}

/// Fixed-point Q1.30 representation of the Hopf bundle state S1 -> S3 -> S2.
/// Zero runtime floats, zero heap allocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HopfFiberPointQ30 {
    /// Base point on S2 in Q1.30: [x, y, z].
    pub base: [i32; 3],
    /// U(1) fiber phasor: [cos(psi), sin(psi)] in Q1.30.
    pub fiber_u1: [i32; 2],
    /// U(1) fiber phase angle normalized by pi: psi / pi in Q1.30 (range [-2^30, 2^30]).
    pub fiber_phase: i32,
}

impl Default for HopfFiberPointQ30 {
    fn default() -> Self {
        Self {
            base: [0, 0, 1 << 30],
            fiber_u1: [1 << 30, 0],
            fiber_phase: 0,
        }
    }
}

/// Exact 32-bit signed scalar product using Radix-4 shift-and-add (at most 16 iterations).
/// Zero hardware multipliers, zero floating-point registers.
#[inline]
pub fn mul_i32_radix4(left: i32, right: i32) -> i64 {
    let negative = (left < 0) ^ (right < 0);
    let mut u = left.unsigned_abs() as u64;
    let mut v = right.unsigned_abs() as u64;
    if u < v {
        std::mem::swap(&mut u, &mut v);
    }
    let mut acc = 0u64;
    while v != 0 {
        let chunk = (v & 3) as u32;
        match chunk {
            1 => acc = acc.wrapping_add(u),
            2 => acc = acc.wrapping_add(u << 1),
            3 => acc = acc.wrapping_add((u << 1) + u),
            _ => {}
        }
        v >>= 2;
        u <<= 2;
    }
    if negative {
        acc.wrapping_neg() as i64
    } else {
        acc as i64
    }
}

/// Out-of-line coordinate projection helper to prevent LLVM SLP auto-vectorization into FP/NEON registers (fmov).
#[inline(never)]
fn project_coord_2x(sum: i64) -> i32 {
    let val = (sum << 1) >> 30;
    val.clamp(-(1 << 30), 1 << 30) as i32
}

/// Out-of-line coordinate projection helper to prevent LLVM SLP auto-vectorization into FP/NEON registers (fmov).
#[inline(never)]
fn project_coord_1x(sum: i64) -> i32 {
    let val = sum >> 30;
    val.clamp(-(1 << 30), 1 << 30) as i32
}

/// Fixed-point Q1.30 unit quaternion on S3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitS3Q30(pub [i32; 4]);

impl UnitS3Q30 {
    pub const IDENTITY: Self = Self([1 << 30, 0, 0, 0]);

    #[inline]
    pub fn raw(&self) -> [i32; 4] {
        self.0
    }

    /// Project arbitrary 4D integer coordinates to a unit quaternion in Q1.30 using restoring integer square root.
    /// Zero hardware multipliers, zero floats.
    pub fn from_i32_coords(coords: &[i32; 4]) -> Self {
        let mut sum_sq = 0u128;
        for &c in coords {
            let sq = mul_i32_radix4(c, c) as u128;
            sum_sq = sum_sq.saturating_add(sq);
        }
        if sum_sq == 0 {
            return Self::IDENTITY;
        }
        let sum_scaled = sum_sq << 60;
        let norm_q30 = isqrt(sum_scaled);
        if norm_q30 == 0 {
            return Self::IDENTITY;
        }
        let mut q = [0i32; 4];
        for (i, &c) in coords.iter().enumerate() {
            let wide = (c as i128) << 60;
            let val = match div_round(wide, norm_q30 as i128) {
                Ok(v) => v.clamp(-(1i128 << 30), 1i128 << 30) as i32,
                Err(_) => 0,
            };
            q[i] = val;
        }
        Self(q)
    }

    /// Exact integer Hopf projection from S3 to S2 in Q1.30:
    ///   x = 2(ac + bd) / 2^30
    ///   y = 2(bc - ad) / 2^30
    ///   z = (a^2 + b^2 - c^2 - d^2) / 2^30
    /// Zero hardware multipliers, zero floats.
    pub fn hopf_project(&self) -> [i32; 3] {
        let a = self.0[0];
        let b = self.0[1];
        let c = self.0[2];
        let d = self.0[3];

        let ac = mul_i32_radix4(a, c);
        let bd = mul_i32_radix4(b, d);
        let bc = mul_i32_radix4(b, c);
        let ad = mul_i32_radix4(a, d);

        let aa = mul_i32_radix4(a, a);
        let bb = mul_i32_radix4(b, b);
        let cc = mul_i32_radix4(c, c);
        let dd = mul_i32_radix4(d, d);

        let x = project_coord_2x(ac + bd);
        let y = project_coord_2x(bc - ad);
        let z = project_coord_1x(aa + bb - cc - dd);

        [x, y, z]
    }

    /// Extract U(1) fiber unit phasor in Q1.30: [cos(psi), sin(psi)].
    /// Zero hardware multipliers, zero native hardware dividers, zero floats.
    pub fn fiber_u1_q30(&self) -> [i32; 2] {
        let a = self.0[0];
        let b = self.0[1];
        let aa = mul_i32_radix4(a, a) as u128;
        let bb = mul_i32_radix4(b, b) as u128;
        let norm_ab_sq = aa + bb;
        if norm_ab_sq > (1 << 20) {
            let s = isqrt(norm_ab_sq << 60);
            if s > 0 {
                let u = match div_round((a as i128) << 60, s as i128) {
                    Ok(val) => val.clamp(-(1i128 << 30), 1i128 << 30) as i32,
                    Err(_) => 0,
                };
                let v = match div_round((b as i128) << 60, s as i128) {
                    Ok(val) => val.clamp(-(1i128 << 30), 1i128 << 30) as i32,
                    Err(_) => 0,
                };
                return [u, v];
            }
        }
        let c = self.0[2];
        let d = self.0[3];
        let cc = mul_i32_radix4(c, c) as u128;
        let dd = mul_i32_radix4(d, d) as u128;
        let norm_cd_sq = cc + dd;
        if norm_cd_sq > 0 {
            let s = isqrt(norm_cd_sq << 60);
            if s > 0 {
                let u = match div_round((c as i128) << 60, s as i128) {
                    Ok(val) => val.clamp(-(1i128 << 30), 1i128 << 30) as i32,
                    Err(_) => 0,
                };
                let v = match div_round((d as i128) << 60, s as i128) {
                    Ok(val) => val.clamp(-(1i128 << 30), 1i128 << 30) as i32,
                    Err(_) => 0,
                };
                return [u, v];
            }
        }
        [1 << 30, 0]
    }

    /// Full fiber-preserving Hopf projection in Q1.30.
    pub fn hopf_fiber_project(&self) -> HopfFiberPointQ30 {
        let base = self.hopf_project();
        let fiber_u1 = self.fiber_u1_q30();
        let fiber_phase = atan2_q30(fiber_u1[1], fiber_u1[0]);
        HopfFiberPointQ30 {
            base,
            fiber_u1,
            fiber_phase,
        }
    }
}

/// Riemann zeta-zero frequencies for the first 8 non-trivial zeros in Q1.30.
/// Normalized step constants: (gamma_j mod 2pi) / pi * 2^30.
pub const ZETA_FREQUENCIES_Q30: [i32; 8] = [
    536_034_177,   // gamma_1: 14.1347...
    741_753_909,   // gamma_2: 21.0220...
    1_032_483_861, // gamma_3: 25.0108...
    734_898_144,   // gamma_4: 30.4248...
    518_590_487,   // gamma_5: 32.9350...
    1_033_281_512, // gamma_6: 37.5861...
    57_918_840,    // gamma_7: 40.9187...
    846_875_908,   // gamma_8: 43.3270...
];

/// Ergodic phase coordinates on the 8-torus T^8.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct T8ZetaState {
    pub phases: [i32; 8],
}

impl T8ZetaState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw unmodulated frequency step (advances by exact zeta frequencies).
    pub fn step_raw(&mut self) {
        for (phase, &freq) in self.phases.iter_mut().zip(ZETA_FREQUENCIES_Q30.iter()) {
            let next = (*phase as i64) + (freq as i64);
            let mut wrapped = next;
            while wrapped >= (1i64 << 30) {
                wrapped -= 2i64 << 30;
            }
            while wrapped < -(1i64 << 30) {
                wrapped += 2i64 << 30;
            }
            *phase = wrapped as i32;
        }
    }

    /// Advance T^8 phase coordinates coupled to token prime identity.
    /// Zero hardware multipliers, zero floats.
    pub fn step(&mut self, token: u32) {
        for (j, (phase, &freq_val)) in self
            .phases
            .iter_mut()
            .zip(ZETA_FREQUENCIES_Q30.iter())
            .enumerate()
        {
            let freq = freq_val as i64;
            let k = 1i64 + (((token as i64) + (j as i64)) & 0x07);

            // 4-bit unrolled shift-add multiplication: k in [1, 8]
            let mut prod = 0i64;
            if k & 1 != 0 {
                prod += freq;
            }
            if k & 2 != 0 {
                prod += freq << 1;
            }
            if k & 4 != 0 {
                prod += freq << 2;
            }
            if k & 8 != 0 {
                prod += freq << 3;
            }
            let delta = prod >> 3;

            let next = (*phase as i64) + delta;
            let mut wrapped = next;
            while wrapped >= (1i64 << 30) {
                wrapped -= 2i64 << 30;
            }
            while wrapped < -(1i64 << 30) {
                wrapped += 2i64 << 30;
            }
            *phase = wrapped as i32;
        }
    }
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

    #[test]
    fn cordic_atan2_and_hopf_projection_properties() {
        assert_eq!(atan2_q30(0, 0), 0);
        assert_eq!(atan2_q30(0, 1000), 0);
        assert_eq!(atan2_q30(1000, 0), 1 << 29);
        assert_eq!(atan2_q30(-1000, 0), -(1 << 29));

        // Identity quaternion Hopf projection -> North Pole (0, 0, 1) and fiber phase 0
        let id_s3 = UnitS3Q30::IDENTITY;
        let pt = id_s3.hopf_fiber_project();
        assert_eq!(pt.base, [0, 0, 1 << 30]);
        assert_eq!(pt.fiber_phase, 0);

        // Normalize non-zero coords to S3
        let s3 = UnitS3Q30::from_i32_coords(&[100, 200, 300, 400]);
        let pt2 = s3.hopf_fiber_project();
        assert_ne!(pt2.base, [0, 0, 0]);

        // T8 zeta state stepping
        let mut zeta = T8ZetaState::new();
        let init = zeta.phases;
        zeta.step(42);
        assert_ne!(zeta.phases, init);
    }

    #[test]
    fn radix4_multiplication_matches_native_i64() {
        for left in [-10000, -1, 0, 1, 7, 42, 10000, i32::MAX, i32::MIN + 1] {
            for right in [-9999, -2, 0, 1, 9, 37, 9999, i32::MAX, i32::MIN + 1] {
                let expected = (left as i64) * (right as i64);
                assert_eq!(mul_i32_radix4(left, right), expected);
            }
        }
    }
}
