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
    let conv = |v: &[u32]| {
        v.iter()
            .map(|&b| f32_to_rational(b))
            .collect::<Result<Vec<_>, _>>()
    };
    let masters = conv(masters)?;
    let g = conv(g)?;
    let guards = guards
        .iter()
        .map(|row| conv(row))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Problem::new(masters, g, guards))
}

/// Load a problem from saved bit patterns (loader-facing alias; the bits are the
/// binary32 patterns of masters, objective and guard rows).
pub fn problem_from_saved_bits(
    masters: &[u32],
    g: &[u32],
    guards: &[Vec<u32>],
) -> Result<Problem, ExactError> {
    problem_from_f32_bits(masters, g, guards)
}

/// True when the bit-exact binary32 accumulation of the per-term files, in the
/// stored order, equals the aggregate file's bits. This is the original source's own
/// replay check (protected_joint_vector.rs: f32 sum then bit comparison). Binary
/// floating point is allowed here, the crate's only float file.
pub fn term_aggregate_matches(terms: &[Vec<u32>], aggregate: &[u32]) -> bool {
    if terms.iter().any(|t| t.len() != aggregate.len()) {
        return false;
    }
    for (j, want) in aggregate.iter().enumerate() {
        let mut sum = 0f32;
        for term in terms {
            sum += f32::from_bits(term[j]);
        }
        if sum.to_bits() != *want {
            return false;
        }
    }
    true
}

/// The original protected_legal_set refuses a master outside the legal quarter grid
/// (|m| > 1.75); the loader refuses it too instead of clamping q0.
pub fn master_weight_in_range(bits: u32) -> bool {
    let x = f32::from_bits(bits);
    x.is_finite() && x.abs() <= 1.75
}
