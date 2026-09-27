//! Integer Lorentz read score for the retained recurrent model.
//!
//! Query and key codes are Q8. Lifting both to the hyperboloid
//! x -> (sqrt(1+|x|^2), x) gives z = q0 k0 - <q,k> >= 1. In code units (2^16
//! per unit of a product), with Q = |q|^2, K = |k|^2 and D = <q,k>,
//! z - 1 = (sqrt(P) - M) 2^-16 for P = (2^16+Q)(2^16+K) and M = 2^16+D. Since
//! P - M^2 = 2^16 |q-k|^2 + (QK - D^2) is a nonnegative integer, the difference
//! is evaluated as (P - M^2)/(sqrt(P) + M) when M > 0, without cancellation:
//! one floor square root with 24 guard bits and one division rounded to nearest
//! give z - 1 at Q32. arcosh(1+u) comes from a sealed Q24 table, and the learned
//! scale exp(read.lorentz_log_beta) is evaluated once at load. Products,
//! divisions and square roots use `crate::math`; nothing here is floating.

use crate::math::{self, MathResult};
use crate::{invalid, Result};

/// Fraction bits of the arcosh argument code `u 2^32`, where `u = z - 1`.
pub const ARCOSH_FRACTION_BITS: u32 = 32;
/// Table points per octave of the argument code, as a power of two.
pub const ARCOSH_MANTISSA_BITS: u32 = 10;
/// Argument codes below `2^ARCOSH_CODE_BITS` are covered (`u < 2^32`). Q8 read
/// codes keep `|q|^2 <= 64 * 128^2 = 2^20`, so `u` stays below `2^22`.
pub const ARCOSH_CODE_BITS: u32 = 64;
/// Fraction bits of the tabulated distances.
pub const ARCOSH_OUTPUT_BITS: u32 = 24;
/// Entries of the sealed table: codes below `2^10` directly, then `2^10` points
/// per octave, then the end point `2^ARCOSH_CODE_BITS`.
pub const ARCOSH_ENTRIES: usize =
    (((ARCOSH_CODE_BITS - ARCOSH_MANTISSA_BITS + 1) as usize) << ARCOSH_MANTISSA_BITS) + 1;
/// Offline training clamps z at the F32 value of 1 + 1e-6, which is 1 + 2^-20:
/// argument code 2^12.
pub const MINIMUM_EXCESS_CODE: u128 = 1 << 12;
/// Fixed values of `round(2^24 arcosh(1 + u))` checked at table load: at the
/// first nonzero code (`u = 2^-32`), at `u = 1` and at the end point `u = 2^32`.
pub(crate) const ARCOSH_ANCHORS: [(usize, u32); 3] = [
    (1, 362),
    (
        ((1 + 32 - ARCOSH_MANTISSA_BITS) as usize) << ARCOSH_MANTISSA_BITS,
        22_094_887,
    ),
    (ARCOSH_ENTRIES - 1, 383_759_639),
];
/// round(2^60 ln 2), a fixed offline constant.
const LN2_Q60: i128 = 799_144_290_325_165_979;
/// The learned scale must stay below 2^31, so that every score product fits.
/// Octaves above this are refused before expansion; the result is then
/// checked against the limit itself, since octave 31 still reaches 2^31.5.
const MAXIMUM_SCALE_OCTAVE: i128 = 31;
/// 2^31 at Q32: the first refused scale.
const SCALE_LIMIT_Q32: i128 = 1 << 63;
const GUARD_BITS: i32 = 24;
const UNIT: i128 = 1 << 16;
/// Fraction bits of the score product: a Q32 scale times a Q24 difference.
const PRODUCT_BITS: i32 = 56;

fn arithmetic<T>(value: MathResult<T>) -> Result<T> {
    value.map_err(|e| invalid(format!("integer Lorentz arithmetic: {e}")))
}

/// The argument code at table index `index` (the table's grid).
pub fn arcosh_grid(index: usize) -> u128 {
    let direct = 1usize << ARCOSH_MANTISSA_BITS;
    if index < direct {
        return index as u128;
    }
    let octave = (index - direct) >> ARCOSH_MANTISSA_BITS;
    let mantissa = (index - direct) & (direct - 1);
    ((direct + mantissa) as u128) << octave
}

/// `round(2^24 arcosh(1 + code 2^-32))` from the sealed table (entry `i` is that
/// value at [`arcosh_grid`]`(i)`), interpolated linearly between grid points
/// `2^-10` of an octave apart and rounded to nearest.
pub fn arcosh1p_q24(code: u128, table: &[u32]) -> Result<u32> {
    if table.len() != ARCOSH_ENTRIES {
        return Err(invalid("arcosh table length differs"));
    }
    let direct = 1u128 << ARCOSH_MANTISSA_BITS;
    if code < direct {
        return Ok(table[code as usize]);
    }
    if code >= 1u128 << ARCOSH_CODE_BITS {
        return Err(invalid("Lorentz read argument beyond the arcosh table"));
    }
    let shift = 127 - code.leading_zeros() - ARCOSH_MANTISSA_BITS;
    let mantissa = ((code >> shift) - direct) as usize;
    let index = ((1 + shift as usize) << ARCOSH_MANTISSA_BITS) + mantissa;
    if shift == 0 {
        return Ok(table[index]);
    }
    let low = u128::from(table[index]);
    let rise = u128::from(table[index + 1])
        .checked_sub(low)
        .ok_or_else(|| invalid("arcosh table decreases"))?;
    let fraction = code & ((1u128 << shift) - 1);
    // rise < 2^32 and fraction < 2^53: the product and rounding term fit.
    let step =
        (arithmetic(math::checked_mul_unsigned(rise, fraction))? + (1u128 << (shift - 1))) >> shift;
    u32::try_from(low + step).map_err(|_| invalid("arcosh value range"))
}

/// z - 1 at Q32, before the training clamp, for Q8 codes with squared norms
/// `query_norm`, `key_norm` and inner product `inner`, all in code units.
pub fn excess_q32(query_norm: i128, key_norm: i128, inner: i128) -> Result<u128> {
    if query_norm < 0 || key_norm < 0 {
        return Err(invalid("negative Lorentz squared norm"));
    }
    let product = arithmetic(math::checked_mul(UNIT + query_norm, UNIT + key_norm))?;
    let middle = UNIT + inner;
    let root = math::isqrt(arithmetic(math::shift_left_unsigned(
        product.unsigned_abs(),
        2 * GUARD_BITS as u32,
    ))?);
    let root = i128::try_from(root).map_err(|_| invalid("Lorentz square root range"))?;
    let shifted = arithmetic(math::scale_pow2(middle, GUARD_BITS))?;
    let code = if middle <= 0 {
        arithmetic(math::scale_pow2(root - shifted, 16 - GUARD_BITS))?
    } else {
        let excess = product - arithmetic(math::checked_mul(middle, middle))?;
        if excess < 0 {
            return Err(invalid("Lorentz codes violate the Cauchy-Schwarz bound"));
        }
        arithmetic(math::div_round(
            arithmetic(math::scale_pow2(excess, 16 + GUARD_BITS))?,
            root + shifted,
        ))?
    };
    u128::try_from(code).map_err(|_| invalid("negative Lorentz excess"))
}

/// exp(code 2^exponent) at Q32, rounded to nearest: x = n ln2 + r with
/// |r| <= ln2/2 at Q60, a Taylor series for exp(r) at Q60, then a shift by n.
/// Results below 2^-64 are zero; results of 2^31 or more are refused.
pub fn exp_q32(code: i128, exponent: i32) -> Result<i128> {
    let x = arithmetic(math::scale_pow2(code, 60 + exponent))?;
    let octaves = arithmetic(math::div_round(x, LN2_Q60))?;
    if octaves > MAXIMUM_SCALE_OCTAVE {
        return Err(invalid("Lorentz read scale reaches 2^31"));
    }
    if octaves < -64 {
        return Ok(0);
    }
    let remainder = x - arithmetic(math::checked_mul(octaves, LN2_Q60))?;
    let mut sum = 1i128 << 60;
    let mut term = 1i128 << 60;
    for order in 1..=40 {
        // |term| <= 2^60 and |remainder| < 2^59: the product fits.
        let raised = arithmetic(math::scale_pow2(
            arithmetic(math::checked_mul(term, remainder))?,
            -60,
        ))?;
        term = arithmetic(math::div_round(raised, order))?;
        if term == 0 {
            break;
        }
        sum += term;
    }
    let scale = arithmetic(math::scale_pow2(sum, octaves as i32 - 28))?;
    if scale >= SCALE_LIMIT_Q32 {
        return Err(invalid("Lorentz read scale reaches 2^31"));
    }
    Ok(scale)
}

/// The learned Lorentz read constants, fixed at load from the packed codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LorentzRead {
    /// exp(read.lorentz_log_beta) at Q32.
    pub scale_q32: i128,
    /// read.lorentz_offset at Q24.
    pub offset_q24: i128,
}

impl LorentzRead {
    /// Both scalars are signed16 codes with one dyadic exponent in [-24,16].
    pub fn new(log_beta: (i16, i16), offset: (i16, i16)) -> Result<Self> {
        for exponent in [log_beta.1, offset.1] {
            if !(-24..=16).contains(&exponent) {
                return Err(invalid("Lorentz scalar exponent outside [-24,16]"));
            }
        }
        Ok(Self {
            scale_q32: exp_q32(i128::from(log_beta.0), i32::from(log_beta.1))?,
            offset_q24: arithmetic(math::scale_pow2(
                i128::from(offset.0),
                i32::from(offset.1) + ARCOSH_OUTPUT_BITS as i32,
            ))?,
        })
    }

    /// exp(log_beta)(offset - arcosh(max(z, 1 + 2^-20))) at `work_bits`
    /// fraction bits, rounded to nearest.
    pub fn score(
        &self,
        query_norm: i128,
        key_norm: i128,
        inner: i128,
        table: &[u32],
        work_bits: i32,
    ) -> Result<i128> {
        let excess = excess_q32(query_norm, key_norm, inner)?.max(MINIMUM_EXCESS_CODE);
        let difference = self.offset_q24 - i128::from(arcosh1p_q24(excess, table)?);
        arithmetic(math::scale_pow2(
            arithmetic(math::checked_mul(self.scale_q32, difference))?,
            work_bits - PRODUCT_BITS,
        ))
    }
}

/// Sum of squared codes, exact.
pub fn squared_norm(codes: &[i32]) -> Result<i128> {
    let mut sum = 0i128;
    for &code in codes {
        sum += arithmetic(math::checked_mul(i128::from(code), i128::from(code)))?;
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inner(a: &[i32], b: &[i32]) -> i128 {
        a.iter()
            .zip(b)
            .map(|(&x, &y)| i128::from(x) * i128::from(y))
            .sum()
    }

    #[test]
    fn grid_and_interpolation_cover_every_octave() -> Result<()> {
        assert_eq!(arcosh_grid(0), 0);
        assert_eq!(arcosh_grid(1023), 1023);
        assert_eq!(arcosh_grid(1024), 1024);
        assert_eq!(arcosh_grid(2048), 2048);
        assert_eq!(arcosh_grid(ARCOSH_ANCHORS[1].0), 1 << 32);
        assert_eq!(arcosh_grid(ARCOSH_ENTRIES - 1), 1 << ARCOSH_CODE_BITS);
        // A table holding its own grid index reproduces grid points exactly and
        // interpolates halfway between neighbours with round-half-up.
        let table: Vec<u32> = (0..ARCOSH_ENTRIES).map(|i| i as u32).collect();
        for index in [0, 1, 1023, 1024, 1025, 5000, ARCOSH_ENTRIES - 2] {
            assert_eq!(arcosh1p_q24(arcosh_grid(index), &table)?, index as u32);
        }
        let low = arcosh_grid(40_000);
        let high = arcosh_grid(40_001);
        assert_eq!(arcosh1p_q24((low + high) / 2, &table)?, 40_001);
        assert_eq!(arcosh1p_q24((low + high) / 2 - 1, &table)?, 40_000);
        assert!(arcosh1p_q24(1 << ARCOSH_CODE_BITS, &table).is_err());
        assert!(arcosh1p_q24(0, &table[1..]).is_err());
        Ok(())
    }

    #[test]
    fn excess_is_zero_on_the_diagonal_and_refuses_inconsistent_codes() -> Result<()> {
        for codes in [vec![0; 64], vec![300; 64], vec![-32767; 64]] {
            let norm = squared_norm(&codes)?;
            assert_eq!(excess_q32(norm, norm, norm)?, 0);
        }
        // Orthogonal unit vectors: z = 2 exactly, so z - 1 = 1 at Q32.
        let mut a = vec![0; 64];
        let mut b = vec![0; 64];
        a[0] = 256;
        b[1] = 256;
        let z = excess_q32(squared_norm(&a)?, squared_norm(&b)?, inner(&a, &b))?;
        assert_eq!(z, 1 << 32);
        // Opposite unit vectors (M = 0 branch): z = 2 + 1 = 3.
        b = vec![0; 64];
        b[0] = -256;
        let z = excess_q32(squared_norm(&a)?, squared_norm(&b)?, inner(&a, &b))?;
        assert_eq!(z, 2 << 32);
        assert!(excess_q32(1, 1, 5).is_err());
        assert!(excess_q32(-1, 1, 0).is_err());
        Ok(())
    }

    #[test]
    fn scale_series_rounds_exp_to_nearest_q32() -> Result<()> {
        assert_eq!(exp_q32(0, 0)?, 1 << 32);
        assert_eq!(exp_q32(0, -24)?, 1 << 32);
        // e = 2.718281828459045..., 2^32 e = 11674931554.73...
        assert_eq!(exp_q32(1, 0)?, 11_674_931_555);
        // 2^32 / e = 1580030168.53...
        assert_eq!(exp_q32(-1, 0)?, 1_580_030_169);
        assert!(exp_q32(22, 0).is_err());
        assert_eq!(exp_q32(-100, 0)?, 0);
        // The limit is the scale itself, not its rounded octave: 31 ln 2 is
        // 5500.82 at exponent -8, and 21.75 rounds to octave 31 but exceeds it.
        assert!(exp_q32(5500, -8)? < 1 << 63);
        assert!(exp_q32(5501, -8).is_err());
        assert!(LorentzRead::new((87, -2), (0, 0)).is_err());
        let read = LorentzRead::new((0, 0), (5, -1))?;
        assert_eq!(read.scale_q32, 1 << 32);
        assert_eq!(read.offset_q24, 5 << 23);
        assert!(LorentzRead::new((0, 17), (0, 0)).is_err());
        Ok(())
    }
}
