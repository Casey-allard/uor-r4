//! Bit-exact conversion of IEEE-754 binary32 inputs to rationals.
//!
//! This is the only file in the crate where `f32` appears. Every finite
//! binary32 value is a dyadic rational `mantissa · 2^exp`; it is converted
//! with no rounding. NaN and infinity are refused.

use std::fmt;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;

use crate::problem::Problem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactError {
    /// The bit pattern encodes NaN or an infinity.
    NotFinite { bits: u32 },
}

impl fmt::Display for ExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExactError::NotFinite { bits } => {
                write!(f, "binary32 bits {bits:#010x} are NaN or infinite")
            }
        }
    }
}

impl std::error::Error for ExactError {}

/// Convert binary32 bits to the exact rational they encode.
///
/// Normal: `(2^23 | frac) · 2^(exp - 150)`. Subnormal (exp field 0):
/// `frac · 2^-149`. Signed zero maps to 0.
pub fn f32_to_rational(bits: u32) -> Result<BigRational, ExactError> {
    let negative = bits >> 31 == 1;
    let exp_field = ((bits >> 23) & 0xff) as i64;
    let frac = bits & 0x7f_ffff;
    if exp_field == 0xff {
        return Err(ExactError::NotFinite { bits });
    }
    let (mantissa, exp) = if exp_field == 0 {
        (frac, -149i64)
    } else {
        (frac | 0x80_0000, exp_field - 150)
    };
    let mut numer = BigInt::from(mantissa);
    if negative {
        numer = -numer;
    }
    Ok(if exp >= 0 {
        BigRational::from_integer(numer << (exp as usize))
    } else {
        BigRational::new(numer, BigInt::one() << ((-exp) as usize))
    })
}

/// Convenience: convert an `f32` value through its bit pattern.
pub fn from_f32(x: f32) -> Result<BigRational, ExactError> {
    f32_to_rational(x.to_bits())
}

/// Load a problem from saved binary32 bit patterns: masters `m`, objective
/// `g`, and guard rows `J`. Any NaN or infinity refuses the whole load.
pub fn problem_from_f32_bits(
    masters: &[u32],
    g: &[u32],
    guards: &[Vec<u32>],
) -> Result<Problem, ExactError> {
    let conv = |v: &[u32]| v.iter().map(|&b| f32_to_rational(b)).collect::<Result<Vec<_>, _>>();
    let masters = conv(masters)?;
    let g = conv(g)?;
    let guards = guards.iter().map(|row| conv(row)).collect::<Result<Vec<_>, _>>()?;
    Ok(Problem::new(masters, g, guards))
}
