//! Artifact grid/geometry metadata helpers only. No model execution kernels.
//! Original numeric kernels are retained under docs/history/d29-excluded-lut-2026-10-13/src/kernels.rs.

/// Bias of the retained grid code's exponent field.
const GRID_EXP_BIAS: i32 = 64;

/// Encode the retained artifact scalar representation without applying it.
pub fn grid_encode(m: u8, e: i32, negative: bool) -> Option<i16> {
    if m >= 16 || !(1 - GRID_EXP_BIAS..=127 - GRID_EXP_BIAS).contains(&e) {
        return None;
    }
    let magnitude = (((e + GRID_EXP_BIAS) as i16) << 4) | i16::from(m);
    Some(if negative { -magnitude } else { magnitude })
}

/// Fractional bits of the [`arcosh1p_q24`] argument: `u = code * 2^-32`.
pub const ARCOSH_FRACTION_BITS: u32 = 32;
/// Table points per octave of the argument code, as a power of two.
pub const ARCOSH_MANTISSA_BITS: u32 = 10;
/// Argument codes below `2^ARCOSH_CODE_BITS` are covered (`u < 2^64`,
/// distances up to about 45); larger codes saturate.
pub const ARCOSH_CODE_BITS: u32 = 96;
/// Entries of the sealed arcosh table: codes below `2^10` directly, then
/// `2^10` points per octave, then the end point.
pub const ARCOSH_TABLE_LEN: usize =
    ((ARCOSH_CODE_BITS - ARCOSH_MANTISSA_BITS + 1) as usize) << ARCOSH_MANTISSA_BITS | 1;

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

/// `round(2^24 arcosh(1 + code 2^-32))` from the sealed table (entry `i` is
/// that value at [`arcosh_grid`]`(i)`), interpolated linearly between grid
/// points spaced `2^-10` of an octave apart. Shifts, compares, additions and
/// one product of runtime values.
pub fn arcosh1p_q24(code: u128, table: &[u32]) -> u32 {
    let direct = 1u128 << ARCOSH_MANTISSA_BITS;
    if code < direct {
        return table[code as usize];
    }
    if code >= 1u128 << ARCOSH_CODE_BITS {
        return table[ARCOSH_TABLE_LEN - 1];
    }
    let bits = 128 - code.leading_zeros();
    let shift = bits - ARCOSH_MANTISSA_BITS - 1;
    let mantissa = ((code >> shift) - direct) as usize;
    let index = ((1 + shift as usize) << ARCOSH_MANTISSA_BITS) + mantissa;
    let (low, high) = (u128::from(table[index]), u128::from(table[index + 1]));
    if shift == 0 {
        return low as u32;
    }
    let fraction = code & ((1u128 << shift) - 1);
    let step = ((high - low) * fraction + (1u128 << (shift - 1))) >> shift;
    (low + step) as u32
}
