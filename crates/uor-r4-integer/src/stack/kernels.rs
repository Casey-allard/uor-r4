//! Multiplier-free kernels of the stack engine (owner decision D11).
//!
//! Each kernel reproduces one operation of the D10 engine (`uor-r4-lut`'s
//! `stack` and `kernels`) on the same integers, including its floor,
//! truncation, round-half-up and saturation conventions, with a different
//! instruction mix:
//!
//! - a product of two runtime values is read from a table of the sixteen
//!   multiples of one operand (built by repeated addition) at the radix-16
//!   digits of the other, and summed with shifts: `stack_mul_u128`,
//!   `stack_mul_u64`, the per-coordinate tables of the read, and the
//!   activation tables of the weight maps;
//! - a quotient is exact restoring long division (`stack_div_u64`,
//!   `stack_div_u128`), and a square root is digit-by-digit (`stack_isqrt`);
//! - index arithmetic uses shifts or running offsets, never a product of
//!   runtime sizes.
//!
//! A product is formed modulo `2^64` or `2^128` exactly as the D10 engine's
//! wrapping release-mode operator, so the two agree even where a bound is
//! not proved here. The only values that enter a table chain pass through an
//! opaque zero ([`core::hint::black_box`]) so that the optimizer cannot fold a
//! chain of additions back into a multiplication. Kernels are
//! `#[inline(never)]` with distinctive `stack_` names so that the ARM64
//! instruction audit (`scripts/audit_zero_matmul_serving.py --stack`) can
//! locate each one in a release binary.

use core::hint::black_box;

use super::format::Fixed;
use super::PHI_Q32;
use crate::h4_classifier::H4_ROOT_COEFFICIENTS;

/// Exponent of the residual stream and of projection outputs: `v * 2^-16`.
pub(crate) const RESIDUAL_EXP: i32 = -16;
/// Exponent of a product of two values at the residual exponent: `v * 2^-32`.
#[allow(dead_code)]
pub(crate) const PRODUCT_EXP: i32 = RESIDUAL_EXP + RESIDUAL_EXP;
/// Bias of a grid code's exponent field.
const GRID_EXP_BIAS: i32 = 64;
/// Table points per octave of the arcosh argument code, as a power of two.
pub(crate) const ARCOSH_MANTISSA_BITS: u32 = 10;
/// Argument codes below `2^ARCOSH_CODE_BITS` are covered; larger codes saturate.
pub(crate) const ARCOSH_CODE_BITS: u32 = 96;
/// Entries of the sealed arcosh table.
pub(crate) const ARCOSH_TABLE_LEN: usize =
    (((ARCOSH_CODE_BITS - ARCOSH_MANTISSA_BITS + 1) as usize) << ARCOSH_MANTISSA_BITS) | 1;
/// Training's floor on the Lorentz excess `z - 1`, `1e-7`, as an arcosh code.
const MIN_EXCESS_CODE: u128 = 429;

/// Round-half-up arithmetic shift right by `shift` (left if negative),
/// saturating at the `i64` range (the D10 engine's `shift`).
#[inline(always)]
pub(crate) fn shift(value: i64, shift: i32) -> i64 {
    if shift > 0 {
        if shift >= 63 {
            return if value < 0 { -1 } else { 0 };
        }
        let half = 1i64 << (shift - 1);
        value.saturating_add(half) >> shift
    } else if shift < 0 {
        let k = shift.unsigned_abs().min(63);
        let limit = i64::MAX >> k;
        if value > limit {
            i64::MAX
        } else if value < -limit {
            i64::MIN
        } else {
            value << k
        }
    } else {
        value
    }
}

/// Round-half-up shift of an `i128` right by `shift`, saturated to `i64`.
#[inline(always)]
pub(crate) fn shift_wide(value: i128, shift: u32) -> i64 {
    let rounded = if shift == 0 {
        value
    } else {
        value.wrapping_add(1i128 << (shift - 1)) >> shift
    };
    rounded.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

/// Convert `value * 2^from` to the nearest `v * 2^to`, saturated to `i32`.
#[inline(always)]
pub(crate) fn to_exp_i32(value: i64, from: i32, to: i32) -> i32 {
    shift(value, to - from).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[inline(always)]
fn bit_length(value: u64) -> i32 {
    64 - value.leading_zeros() as i32
}

/// `(16 + m) a` by shifts and additions (`m < 16`), wrapping as the D10
/// engine's release build.
#[inline(always)]
pub(crate) fn scale_16_plus(a: i64, m: u8) -> i64 {
    let mut t = a << 4;
    for bit in 0..4 {
        if m & (1 << bit) != 0 {
            t = t.wrapping_add(a << bit);
        }
    }
    t
}

/// Whether `code` is a grid code: zero, or `|code| = (e + 64) << 4 | m` with
/// `1 <= e + 64 <= 127`, meaning `±(16 + m) 2^(e - 4)`.
pub(crate) fn grid_valid(code: i16) -> bool {
    let field = code.unsigned_abs() >> 4;
    code == 0 || (1..=127).contains(&field)
}

/// `scalar(code) value` by shifts and additions, rounded half up.
#[inline(always)]
pub(crate) fn grid_apply(value: i64, code: i16) -> i64 {
    if code == 0 {
        return 0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - GRID_EXP_BIAS;
    let scaled = shift(scale_16_plus(value, (c & 15) as u8), 4 - e);
    if code < 0 {
        scaled.saturating_neg()
    } else {
        scaled
    }
}

// ---------------------------------------------------------------------------
// Products of runtime values: sixteen-multiple tables and radix-16 digits.

/// The multiples `k a`, `k = 0..16`, modulo `2^128`, by repeated addition.
#[inline(always)]
fn multiples_u128(a: u128) -> [u128; 16] {
    let zero = black_box(0u128);
    let mut table = [0u128; 16];
    let mut acc = 0u128;
    for slot in table.iter_mut().skip(1) {
        acc = acc.wrapping_add(a) ^ zero;
        *slot = acc;
    }
    table
}

/// The multiples `k a`, `k = 0..16`, modulo `2^64`, by repeated addition.
#[inline(always)]
fn multiples_u64(a: u64) -> [u64; 16] {
    let zero = black_box(0u64);
    let mut table = [0u64; 16];
    let mut acc = 0u64;
    for slot in table.iter_mut().skip(1) {
        acc = acc.wrapping_add(a) ^ zero;
        *slot = acc;
    }
    table
}

/// The multiples `k x`, `k = 0..16`, of a signed value, modulo `2^64`.
#[inline(always)]
fn multiples_i64(x: i64, zero: i64) -> [i64; 16] {
    let mut table = [0i64; 16];
    let mut acc = 0i64;
    for slot in table.iter_mut().skip(1) {
        acc = acc.wrapping_add(x) ^ zero;
        *slot = acc;
    }
    table
}

/// `a b` modulo `2^128`: the multiples of the larger operand read at the
/// radix-16 digits of the smaller, most significant first.
#[inline(never)]
pub(crate) fn stack_mul_u128(a: u128, b: u128) -> u128 {
    let (big, small) = if a >= b { (a, b) } else { (b, a) };
    if small == 0 {
        return 0;
    }
    let table = multiples_u128(big);
    let mut digits = (131 - small.leading_zeros()) >> 2;
    let mut acc = 0u128;
    while digits > 0 {
        digits -= 1;
        acc = (acc << 4).wrapping_add(table[((small >> (digits << 2)) & 15) as usize]);
    }
    acc
}

/// `a b` modulo `2^64`, as [`stack_mul_u128`].
#[inline(never)]
pub(crate) fn stack_mul_u64(a: u64, b: u64) -> u64 {
    let (big, small) = if a >= b { (a, b) } else { (b, a) };
    if small == 0 {
        return 0;
    }
    let table = multiples_u64(big);
    let mut digits = (67 - small.leading_zeros()) >> 2;
    let mut acc = 0u64;
    while digits > 0 {
        digits -= 1;
        acc = (acc << 4).wrapping_add(table[((small >> (digits << 2)) & 15) as usize]);
    }
    acc
}

/// `a b` modulo `2^128` for signed operands: the product of the magnitudes,
/// negated when the signs differ (equal to `a.wrapping_mul(b)`).
#[inline(always)]
pub(crate) fn mul_i128(a: i128, b: i128) -> i128 {
    let magnitude = stack_mul_u128(a.unsigned_abs(), b.unsigned_abs());
    if (a < 0) != (b < 0) {
        magnitude.wrapping_neg() as i128
    } else {
        magnitude as i128
    }
}

/// `a b` modulo `2^64` for signed operands (equal to `a.wrapping_mul(b)`).
#[inline(always)]
pub(crate) fn mul_i64(a: i64, b: i64) -> i64 {
    let magnitude = stack_mul_u64(a.unsigned_abs(), b.unsigned_abs());
    if (a < 0) != (b < 0) {
        magnitude.wrapping_neg() as i64
    } else {
        magnitude as i64
    }
}

/// `x y` from `table = multiples_i64(x)`: the eight radix-16 digits of the
/// two's-complement `y`, then `- x 2^32` for a negative `y`. Exact whenever
/// `|x y| < 2^63` (every caller: `|x| < 2^32`, `|y| <= 2^31`).
#[inline(always)]
fn mul_by_i32(table: &[i64; 16], y: i32) -> i64 {
    let u = y as u32;
    let digit = |s: u32| table[((u >> s) & 15) as usize];
    let low = digit(0)
        .wrapping_add(digit(4) << 4)
        .wrapping_add(digit(8) << 8)
        .wrapping_add(digit(12) << 12);
    let high = digit(16)
        .wrapping_add(digit(20) << 4)
        .wrapping_add(digit(24) << 8)
        .wrapping_add(digit(28) << 12);
    let sign = (table[1] << 32) & i64::from(y >> 31);
    low.wrapping_add(high << 16).wrapping_sub(sign)
}

/// `v^2` for `|v| < 2^32`, exactly: multiples of `|v|` at its eight digits.
#[inline(never)]
pub(crate) fn stack_square(v: i64) -> u64 {
    let u = v.unsigned_abs();
    let table = multiples_u64(u);
    let digit = |s: u32| table[((u >> s) & 15) as usize];
    let low = digit(0)
        .wrapping_add(digit(4) << 4)
        .wrapping_add(digit(8) << 8)
        .wrapping_add(digit(12) << 12);
    let high = digit(16)
        .wrapping_add(digit(20) << 4)
        .wrapping_add(digit(24) << 8)
        .wrapping_add(digit(28) << 12);
    low.wrapping_add(high << 16)
}

// ---------------------------------------------------------------------------
// Quotients and roots.

/// `numerator / denominator` (floor) by restoring long division; zero for a
/// zero denominator, which no caller passes.
#[inline(never)]
pub(crate) fn stack_div_u64(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 || numerator < denominator {
        return 0;
    }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut divisor = denominator << shift;
    let mut place = 1u64 << shift;
    let mut remainder = numerator;
    let mut quotient = 0u64;
    loop {
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        if place == 1 {
            return quotient;
        }
        place >>= 1;
        divisor >>= 1;
    }
}

/// `numerator / denominator` (floor) for `u128`, as [`stack_div_u64`].
#[inline(never)]
pub(crate) fn stack_div_u128(numerator: u128, denominator: u128) -> u128 {
    if denominator == 0 || numerator < denominator {
        return 0;
    }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut divisor = denominator << shift;
    let mut place = 1u128 << shift;
    let mut remainder = numerator;
    let mut quotient = 0u128;
    loop {
        if remainder >= divisor {
            remainder -= divisor;
            quotient |= place;
        }
        if place == 1 {
            return quotient;
        }
        place >>= 1;
        divisor >>= 1;
    }
}

/// Floor integer square root, digit by digit (the D10 engine's `isqrt`).
#[inline(never)]
pub(crate) fn stack_isqrt(value: u128) -> u128 {
    let mut remainder = value;
    let mut root = 0u128;
    let mut bit = 1u128 << 126;
    while bit > value {
        bit >>= 2;
    }
    while bit != 0 {
        if remainder >= root + bit {
            remainder -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

// ---------------------------------------------------------------------------
// Tables of the nonlinearities.

/// `round(2^31 exp(-d))` for `d = value * 2^exp_d`, from the sealed table with
/// step `2^step_log2` and linear interpolation. The table is non-increasing
/// (checked at load), so every difference and product is exact.
#[inline(never)]
pub(crate) fn stack_exp_neg(d: i64, exp_d: i32, table: &[u32], step_log2: i32) -> u64 {
    let frac_bits = step_log2 - exp_d;
    if d <= 0 {
        return table.first().map_or(0, |v| u64::from(*v));
    }
    if frac_bits <= 0 {
        let index = (d << (-frac_bits)) as usize;
        return table.get(index).map_or(0, |v| u64::from(*v));
    }
    let index = (d >> frac_bits) as usize;
    let (Some(&a), Some(&b)) = (table.get(index), table.get(index.wrapping_add(1))) else {
        return 0;
    };
    let frac = (d & ((1 << frac_bits) - 1)) as u64;
    let (a, b) = (u64::from(a), u64::from(b));
    a.wrapping_sub(stack_mul_u64(a.wrapping_sub(b), frac) >> frac_bits)
}

/// `sigmoid(x)` for `x` at exponent -16, times `2^31`, from the exp table.
#[inline(never)]
pub(crate) fn stack_sigmoid_q31(x: i64, table: &[u32], step_log2: i32) -> u64 {
    let e = stack_exp_neg(x.abs(), RESIDUAL_EXP, table, step_log2);
    let one = 1u64 << 31;
    if x >= 0 {
        stack_div_u64(one << 31, one + e)
    } else {
        stack_div_u64(e << 31, one + e)
    }
}

/// An activation (SiLU, GELU) of `x * 2^-16` at exponent -16 from its sealed
/// table: zero below the range, `x` above it, linear interpolation inside.
#[inline(never)]
pub(crate) fn stack_activation(x: i32, table: &[i32], step_log2: i32, range_log2: i32) -> i32 {
    let frac_bits = step_log2 + 16;
    let half = 1i64 << (range_log2 - step_log2);
    let index = (i64::from(x) >> frac_bits) + half;
    if index < 0 {
        return 0;
    }
    let (Some(&a), Some(&b)) = (
        table.get(index as usize),
        table.get((index as usize).wrapping_add(1)),
    ) else {
        return x;
    };
    let frac = i64::from(x) & ((1i64 << frac_bits) - 1);
    let (a, b) = (i64::from(a), i64::from(b));
    (a + (mul_i64(b - a, frac) >> frac_bits)) as i32
}

/// `round(2^24 arcosh(1 + code 2^-32))` from the sealed table, interpolated
/// between grid points `2^-10` of an octave apart (the D10 engine's
/// `arcosh1p_q24`; the table is non-decreasing, checked at load).
#[inline(never)]
pub(crate) fn stack_arcosh1p_q24(code: u128, table: &[u32]) -> u32 {
    let direct = 1u128 << ARCOSH_MANTISSA_BITS;
    let last = table.len().wrapping_sub(1);
    if code < direct {
        return table.get(code as usize).copied().unwrap_or(0);
    }
    if code >= 1u128 << ARCOSH_CODE_BITS {
        return table.get(last).copied().unwrap_or(0);
    }
    let bits = 128 - code.leading_zeros();
    let shift = bits - ARCOSH_MANTISSA_BITS - 1;
    let mantissa = ((code >> shift) - direct) as usize;
    let index = ((1 + shift as usize) << ARCOSH_MANTISSA_BITS) + mantissa;
    let (Some(&low), Some(&high)) = (table.get(index), table.get(index + 1)) else {
        return 0;
    };
    let (low, high) = (u128::from(low), u128::from(high));
    if shift == 0 {
        return low as u32;
    }
    let fraction = code & ((1u128 << shift) - 1);
    let step = stack_mul_u128(high.wrapping_sub(low), fraction).wrapping_add(1u128 << (shift - 1))
        >> shift;
    low.wrapping_add(step) as u32
}

// ---------------------------------------------------------------------------
// Vector kernels.

/// The largest magnitude in `values`. Every element passes through an opaque
/// barrier, which keeps the reduction scalar: a vectorized maximum would end
/// in a SIMD-to-general-register transfer (`fmov`), which the R1 audit refuses.
#[inline(never)]
pub(crate) fn stack_max_abs_i32(values: &[i32]) -> u32 {
    let mut max = 0u32;
    for &v in values {
        max = max.max(black_box(v).unsigned_abs());
    }
    max
}

/// The largest magnitude in `values`, as [`stack_max_abs_i32`].
#[inline(never)]
pub(crate) fn stack_max_abs_i64(values: &[i64]) -> u64 {
    let mut max = 0u64;
    for &v in values {
        max = max.max(black_box(v).unsigned_abs());
    }
    max
}

/// Requantize `values * 2^exp` to 16 bits with the smallest exponent that
/// fits; returns the new exponent.
#[inline(never)]
pub(crate) fn stack_quantize16(values: &[i64], exp: i32, out: &mut [i16]) -> i32 {
    let max = stack_max_abs_i64(values);
    let s = (bit_length(max) - 15).max(0);
    for (o, &v) in out.iter_mut().zip(values) {
        *o = shift(v, s).clamp(-32767, 32767) as i16;
    }
    exp + s
}

/// RMSNorm without its gain (folded into the following weights): `x / rms(x)`
/// for `x` at the residual exponent, into `out` (16-bit); returns the output
/// exponent. `scratch` has the length of `x`.
#[inline(never)]
pub(crate) fn stack_rms_norm(x: &[i32], eps: Fixed, scratch: &mut [i64], out: &mut [i16]) -> i32 {
    let max = stack_max_abs_i32(x);
    let sh = (bit_length(u64::from(max)) - 24).max(0);
    // After the shift every value is within 2^24, so each square is at most
    // 2^48 and the sum of at most 2^14 squares is exact.
    let mut sum = 0u64;
    for (s, &v) in scratch.iter_mut().zip(x) {
        let y = shift(i64::from(v), sh);
        *s = y;
        sum = sum.wrapping_add(stack_square(y));
    }
    let mean = stack_div_u64(sum, x.len().max(1) as u64) as i64;
    // The squares' exponent, twice that of the shifted values.
    let square_exp = (RESIDUAL_EXP + sh) << 1;
    let eps_int = shift(eps.mantissa, square_exp - eps.exp).max(0);
    let ms = mean.wrapping_add(eps_int).max(1) as u128;
    // s = sqrt(ms) 2^34 and r = 2^96 / s = 2^62 / sqrt(ms).
    let s = stack_isqrt(ms << 68).max(1);
    let r = stack_div_u128(1u128 << 96, s);
    let table = multiples_u128(r);
    for v in scratch.iter_mut() {
        // |v| <= 2^24: seven radix-16 digits.
        let u = v.unsigned_abs();
        let digit = |s: u32| table[((u >> s) & 15) as usize];
        let magnitude = digit(0)
            .wrapping_add(digit(4) << 4)
            .wrapping_add(digit(8) << 8)
            .wrapping_add(digit(12) << 12)
            .wrapping_add(digit(16) << 16)
            .wrapping_add(digit(20) << 20)
            .wrapping_add(digit(24) << 24);
        let product = if *v < 0 {
            magnitude.wrapping_neg() as i128
        } else {
            magnitude as i128
        };
        let y = product >> 48;
        *v = y.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    }
    stack_quantize16(scratch, -14, out)
}

/// The activation tables of a 16-bit vector: for column `c`, entry `u` is
/// `(u - 8) x[c]`, the product with the offset-binary weight nibble `u`
/// (shifts, additions and negations only).
#[inline(never)]
pub(crate) fn stack_activation_tables(x: &[i16], tables: &mut [[i32; 16]]) {
    let zero = black_box(0i32);
    for (table, &value) in tables.iter_mut().zip(x) {
        let v1 = i32::from(value);
        let v2 = (v1 << 1) ^ zero;
        let v4 = (v1 << 2) ^ zero;
        let v8 = (v1 << 3) ^ zero;
        let v3 = v2 + v1;
        let v5 = v4 + v1;
        let v6 = v4 + v2;
        let v7 = v4 + v3;
        *table = [
            -v8, -v7, -v6, -v5, -v4, -v3, -v2, -v1, 0, v1, v2, v3, v4, v5, v6, v7,
        ];
    }
}

/// A 4-bit weight matrix in the container's row-major layout, with the
/// smallest scale exponent of each row.
pub(crate) struct PackedMatrix {
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: Vec<u8>,
    pub scales: Vec<u8>,
    pub min_de: Vec<u8>,
}

/// `out[r] = sum_c W[r][c] x[c]` at the residual exponent, for `x` given by
/// its activation tables (built from the 16-bit vector at exponent `x_exp`).
/// Per weight: one table read and one addition; per group of 32: the scale
/// `(16 + m) 2^(de - de_min)` by shifts and additions. The group sums and the
/// row accumulator are the D10 kernel's exact integers.
/// The loader's shape checks make every size agree, so the early returns are
/// unreachable: a debug build asserts, a release build returns rather than
/// read out of bounds.
#[inline(never)]
pub(crate) fn stack_gemv(m: &PackedMatrix, tables: &[[i32; 16]], x_exp: i32, out: &mut [i32]) {
    let row_bytes = m.cols >> 1;
    let groups = m.cols >> 5;
    debug_assert!(
        out.len() == m.rows && m.min_de.len() == m.rows && tables.len() >= m.cols,
        "stack_gemv: {} outputs and {} activation tables for a {}x{} map",
        out.len(),
        tables.len(),
        m.rows,
        m.cols
    );
    let Some(tables) = tables.get(..m.cols) else {
        return;
    };
    let (mut nibble_at, mut scale_at) = (0usize, 0usize);
    for (slot, &min_de) in out.iter_mut().zip(&m.min_de).take(m.rows) {
        let (row, scales) = (
            m.nibbles.get(nibble_at..nibble_at + row_bytes),
            m.scales.get(scale_at..scale_at + groups),
        );
        debug_assert!(
            row.is_some() && scales.is_some(),
            "stack_gemv: a row lies outside the packed map"
        );
        let (Some(row), Some(scales)) = (row, scales) else {
            return;
        };
        let mut acc = 0i64;
        for ((bytes, columns), &s) in row
            .chunks_exact(16)
            .zip(tables.chunks_exact(32))
            .zip(scales)
        {
            let mut a0 = 0i32;
            let mut a1 = 0i32;
            let mut a2 = 0i32;
            let mut a3 = 0i32;
            for i in 0..4 {
                let b0 = bytes[4 * i];
                let b1 = bytes[4 * i + 1];
                let b2 = bytes[4 * i + 2];
                let b3 = bytes[4 * i + 3];
                a0 = a0
                    .wrapping_add(columns[8 * i][usize::from(b0 & 15)])
                    .wrapping_add(columns[8 * i + 1][usize::from(b0 >> 4)]);
                a1 = a1
                    .wrapping_add(columns[8 * i + 2][usize::from(b1 & 15)])
                    .wrapping_add(columns[8 * i + 3][usize::from(b1 >> 4)]);
                a2 = a2
                    .wrapping_add(columns[8 * i + 4][usize::from(b2 & 15)])
                    .wrapping_add(columns[8 * i + 5][usize::from(b2 >> 4)]);
                a3 = a3
                    .wrapping_add(columns[8 * i + 6][usize::from(b3 & 15)])
                    .wrapping_add(columns[8 * i + 7][usize::from(b3 >> 4)]);
            }
            let a = a0.wrapping_add(a1).wrapping_add(a2.wrapping_add(a3));
            acc = acc.wrapping_add(scale_16_plus(i64::from(a), s & 15) << ((s >> 4) - min_de));
        }
        *slot = to_exp_i32(
            acc,
            m.exp_base + i32::from(min_de) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        nibble_at += row_bytes;
        scale_at += groups;
    }
}

/// The pair tables of a 16-bit vector, from its activation tables: for
/// column pair `p`, entry `b` is `T[2p][b & 15] + T[2p + 1][b >> 4]`, the
/// product of both columns with the weight byte `b` (low nibble: the even
/// column). Additions only.
#[inline(never)]
pub(crate) fn stack_pair_tables(tables: &[[i32; 16]], pairs: &mut [[i32; 256]]) {
    for (pair, columns) in pairs.iter_mut().zip(tables.chunks_exact(2)) {
        let (even, odd) = (&columns[0], &columns[1]);
        for (row, &high) in pair.chunks_exact_mut(16).zip(odd) {
            for (slot, &low) in row.iter_mut().zip(even) {
                *slot = low + high;
            }
        }
    }
}

/// [`stack_gemv`] reading pair tables ([`stack_pair_tables`]): one table
/// read and one addition per weight byte, that is per two weights. The group
/// sums, and so the outputs, are the same integers. The early returns are
/// unreachable after loading, as in [`stack_gemv`].
#[inline(never)]
pub(crate) fn stack_gemv_pairs(
    m: &PackedMatrix,
    pairs: &[[i32; 256]],
    x_exp: i32,
    out: &mut [i32],
) {
    let row_bytes = m.cols >> 1;
    let groups = m.cols >> 5;
    debug_assert!(
        out.len() == m.rows && m.min_de.len() == m.rows && pairs.len() >= row_bytes,
        "stack_gemv_pairs: {} outputs and {} pair tables for a {}x{} map",
        out.len(),
        pairs.len(),
        m.rows,
        m.cols
    );
    let Some(pairs) = pairs.get(..row_bytes) else {
        return;
    };
    let (mut nibble_at, mut scale_at) = (0usize, 0usize);
    for (slot, &min_de) in out.iter_mut().zip(&m.min_de).take(m.rows) {
        let (row, scales) = (
            m.nibbles.get(nibble_at..nibble_at + row_bytes),
            m.scales.get(scale_at..scale_at + groups),
        );
        debug_assert!(
            row.is_some() && scales.is_some(),
            "stack_gemv_pairs: a row lies outside the packed map"
        );
        let (Some(row), Some(scales)) = (row, scales) else {
            return;
        };
        let mut acc = 0i64;
        for ((bytes, tables), &s) in row.chunks_exact(16).zip(pairs.chunks_exact(16)).zip(scales) {
            let a0 = tables[0][usize::from(bytes[0])]
                .wrapping_add(tables[1][usize::from(bytes[1])])
                .wrapping_add(tables[2][usize::from(bytes[2])])
                .wrapping_add(tables[3][usize::from(bytes[3])]);
            let a1 = tables[4][usize::from(bytes[4])]
                .wrapping_add(tables[5][usize::from(bytes[5])])
                .wrapping_add(tables[6][usize::from(bytes[6])])
                .wrapping_add(tables[7][usize::from(bytes[7])]);
            let a2 = tables[8][usize::from(bytes[8])]
                .wrapping_add(tables[9][usize::from(bytes[9])])
                .wrapping_add(tables[10][usize::from(bytes[10])])
                .wrapping_add(tables[11][usize::from(bytes[11])]);
            let a3 = tables[12][usize::from(bytes[12])]
                .wrapping_add(tables[13][usize::from(bytes[13])])
                .wrapping_add(tables[14][usize::from(bytes[14])])
                .wrapping_add(tables[15][usize::from(bytes[15])]);
            let a = a0.wrapping_add(a1).wrapping_add(a2.wrapping_add(a3));
            acc = acc.wrapping_add(scale_16_plus(i64::from(a), s & 15) << ((s >> 4) - min_de));
        }
        *slot = to_exp_i32(
            acc,
            m.exp_base + i32::from(min_de) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        nibble_at += row_bytes;
        scale_at += groups;
    }
}

/// [`stack_gemv_pairs`] unrolled across 4 adjacent matrix rows.
///
/// Groups 4 adjacent rows to share each 1 KB pair table reference across 4 row activations
/// (locality hypothesis for pair table cache reuse; unmeasured cache traffic reduction in serving),
/// producing bit-for-bit identical outputs to [`stack_gemv_pairs`].
#[allow(dead_code)]
#[inline(never)]
pub(crate) fn stack_gemv_pairs_blocked4(
    m: &PackedMatrix,
    pairs: &[[i32; 256]],
    x_exp: i32,
    out: &mut [i32],
) {
    let row_bytes = m.cols >> 1;
    let groups = m.cols >> 5;
    debug_assert!(
        out.len() == m.rows && m.min_de.len() == m.rows && pairs.len() >= row_bytes,
        "stack_gemv_pairs_blocked4: {} outputs and {} pair tables for a {}x{} map",
        out.len(),
        pairs.len(),
        m.rows,
        m.cols
    );
    let Some(pairs) = pairs.get(..row_bytes) else {
        return;
    };

    let full_blocks = m.rows / 4;
    let mut row_offset = 0usize;
    let mut scale_offset = 0usize;

    for b in 0..full_blocks {
        let r0 = b * 4;
        let r1 = r0 + 1;
        let r2 = r0 + 2;
        let r3 = r0 + 3;

        let b0_start = row_offset;
        let b1_start = b0_start + row_bytes;
        let b2_start = b1_start + row_bytes;
        let b3_start = b2_start + row_bytes;

        let s0_start = scale_offset;
        let s1_start = s0_start + groups;
        let s2_start = s1_start + groups;
        let s3_start = s2_start + groups;

        let (Some(b0), Some(b1), Some(b2), Some(b3)) = (
            m.nibbles.get(b0_start..b0_start + row_bytes),
            m.nibbles.get(b1_start..b1_start + row_bytes),
            m.nibbles.get(b2_start..b2_start + row_bytes),
            m.nibbles.get(b3_start..b3_start + row_bytes),
        ) else {
            return;
        };

        let (Some(s0), Some(s1), Some(s2), Some(s3)) = (
            m.scales.get(s0_start..s0_start + groups),
            m.scales.get(s1_start..s1_start + groups),
            m.scales.get(s2_start..s2_start + groups),
            m.scales.get(s3_start..s3_start + groups),
        ) else {
            return;
        };

        let min_de0 = m.min_de[r0];
        let min_de1 = m.min_de[r1];
        let min_de2 = m.min_de[r2];
        let min_de3 = m.min_de[r3];

        let mut acc0 = 0i64;
        let mut acc1 = 0i64;
        let mut acc2 = 0i64;
        let mut acc3 = 0i64;

        for g in 0..groups {
            let chunk_bytes = g * 16;
            let chunk_tables = g * 16;

            let tables = &pairs[chunk_tables..chunk_tables + 16];
            let g_b0 = &b0[chunk_bytes..chunk_bytes + 16];
            let g_b1 = &b1[chunk_bytes..chunk_bytes + 16];
            let g_b2 = &b2[chunk_bytes..chunk_bytes + 16];
            let g_b3 = &b3[chunk_bytes..chunk_bytes + 16];

            let mut sum0 = 0i32;
            let mut sum1 = 0i32;
            let mut sum2 = 0i32;
            let mut sum3 = 0i32;

            for i in 0..16 {
                let t = &tables[i];
                sum0 = sum0.wrapping_add(t[usize::from(g_b0[i])]);
                sum1 = sum1.wrapping_add(t[usize::from(g_b1[i])]);
                sum2 = sum2.wrapping_add(t[usize::from(g_b2[i])]);
                sum3 = sum3.wrapping_add(t[usize::from(g_b3[i])]);
            }

            let sc0 = s0[g];
            let sc1 = s1[g];
            let sc2 = s2[g];
            let sc3 = s3[g];

            acc0 = acc0
                .wrapping_add(scale_16_plus(i64::from(sum0), sc0 & 15) << ((sc0 >> 4) - min_de0));
            acc1 = acc1
                .wrapping_add(scale_16_plus(i64::from(sum1), sc1 & 15) << ((sc1 >> 4) - min_de1));
            acc2 = acc2
                .wrapping_add(scale_16_plus(i64::from(sum2), sc2 & 15) << ((sc2 >> 4) - min_de2));
            acc3 = acc3
                .wrapping_add(scale_16_plus(i64::from(sum3), sc3 & 15) << ((sc3 >> 4) - min_de3));
        }

        out[r0] = to_exp_i32(
            acc0,
            m.exp_base + i32::from(min_de0) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        out[r1] = to_exp_i32(
            acc1,
            m.exp_base + i32::from(min_de1) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        out[r2] = to_exp_i32(
            acc2,
            m.exp_base + i32::from(min_de2) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        out[r3] = to_exp_i32(
            acc3,
            m.exp_base + i32::from(min_de3) - 4 + x_exp,
            RESIDUAL_EXP,
        );

        row_offset += 4 * row_bytes;
        scale_offset += 4 * groups;
    }

    // Remainder rows if rows is not a multiple of 4
    let rem_start = full_blocks * 4;
    for (slot, &min_de) in out[rem_start..].iter_mut().zip(&m.min_de[rem_start..]) {
        let (row, scales) = (
            m.nibbles.get(row_offset..row_offset + row_bytes),
            m.scales.get(scale_offset..scale_offset + groups),
        );
        let (Some(row), Some(scales)) = (row, scales) else {
            return;
        };
        let mut acc = 0i64;
        for ((bytes, tables), &s) in row.chunks_exact(16).zip(pairs.chunks_exact(16)).zip(scales) {
            let mut sum = 0i32;
            for i in 0..16 {
                sum = sum.wrapping_add(tables[i][usize::from(bytes[i])]);
            }
            acc = acc.wrapping_add(scale_16_plus(i64::from(sum), s & 15) << ((s >> 4) - min_de));
        }
        *slot = to_exp_i32(
            acc,
            m.exp_base + i32::from(min_de) - 4 + x_exp,
            RESIDUAL_EXP,
        );
        row_offset += row_bytes;
        scale_offset += groups;
    }
}

/// Row `row` of a packed matrix at the residual exponent (the embedding). The
/// step checks the token against the vocabulary, so the early return is
/// unreachable, as in [`stack_gemv`].
#[inline(never)]
pub(crate) fn stack_dequant_row(m: &PackedMatrix, row: usize, out: &mut [i32]) {
    let row_bytes = m.cols >> 1;
    let groups = m.cols >> 5;
    debug_assert!(
        row < m.rows && out.len() == m.cols,
        "stack_dequant_row: row {row} into {} outputs of a {}x{} map",
        out.len(),
        m.rows,
        m.cols
    );
    let nibble_at = stack_mul_u64(row as u64, row_bytes as u64) as usize;
    let scale_at = stack_mul_u64(row as u64, groups as u64) as usize;
    let (bytes, scales) = (
        m.nibbles.get(nibble_at..nibble_at + row_bytes),
        m.scales.get(scale_at..scale_at + groups),
    );
    debug_assert!(
        bytes.is_some() && scales.is_some(),
        "stack_dequant_row: row {row} lies outside the packed map"
    );
    let (Some(bytes), Some(scales)) = (bytes, scales) else {
        return;
    };
    for (c, slot) in out.iter_mut().enumerate().take(m.cols) {
        let byte = bytes[c >> 1];
        let q = i64::from(if c & 1 == 0 { byte & 15 } else { byte >> 4 }) - 8;
        let scale = scales[c >> 5];
        let value = scale_16_plus(q, scale & 15);
        *slot = to_exp_i32(value, m.exp_base + i32::from(scale >> 4) - 4, RESIDUAL_EXP);
    }
}

/// The multiples of every coordinate of `query` (signed, `|q| <= 2^31`),
/// shared by all positions of one read head.
#[inline(never)]
pub(crate) fn stack_query_tables(query: &[i32], tables: &mut [[i64; 16]]) {
    let zero = black_box(0i64);
    for (table, &q) in tables.iter_mut().zip(query) {
        *table = multiples_i64(i64::from(q), zero);
    }
}

/// `<q, k>` exactly, for `q` given by [`stack_query_tables`].
#[inline(never)]
pub(crate) fn stack_dot(query: &[[i64; 16]], key: &[i32]) -> i128 {
    let mut acc = 0i128;
    for (table, &k) in query.iter().zip(key) {
        acc = acc.wrapping_add(i128::from(mul_by_i32(table, k)));
    }
    acc
}

/// `mix += weight value` for one position's head values (`weight < 2^32`).
#[inline(never)]
pub(crate) fn stack_mix_row(weight: u64, values: &[i32], mix: &mut [i128]) {
    let table = multiples_i64(weight as i64, black_box(0i64));
    for (m, &v) in mix.iter_mut().zip(values) {
        *m = m.wrapping_add(i128::from(mul_by_i32(&table, v)));
    }
}

/// Time coordinate `sqrt(1 + |v|^2)` at exponent -32 of the hyperboloid lift
/// of `v` at exponent -16.
#[inline(never)]
pub(crate) fn stack_lift(v: &[i32]) -> u64 {
    let mut square = 0u128;
    for &x in v {
        square = square.wrapping_add(u128::from(stack_square(i64::from(x))));
    }
    stack_isqrt((1u128 << 64).wrapping_add(square << 32)).min(u128::from(u64::MAX)) as u64
}

/// Lorentz distance at exponent -24 between points at exponent -16, from
/// their lifts and inner product (exponent -32): `z - 1 = q0 k0 - <q, k> - 1`
/// at exponent -64, floored as in training, then the arcosh table.
#[inline(never)]
pub(crate) fn stack_lorentz_distance(
    query_lift: u64,
    key_lift: u64,
    dot: i128,
    arcosh: &[u32],
) -> u32 {
    let product = stack_mul_u128(u128::from(query_lift), u128::from(key_lift)) as i128;
    let excess = product.wrapping_sub(dot << 32).wrapping_sub(1i128 << 64);
    let code = ((excess.max(0) >> 32) as u128).max(MIN_EXCESS_CODE);
    stack_arcosh1p_q24(code, arcosh)
}

/// Hamilton product `a (x) b` of quaternions `(w, x, y, z)`, exactly.
#[inline(never)]
pub(crate) fn stack_hamilton(a: [i64; 4], b: [i64; 4]) -> [i128; 4] {
    let p = |x: i64, y: i64| mul_i128(i128::from(x), i128::from(y));
    let [a0, a1, a2, a3] = a;
    let [b0, b1, b2, b3] = b;
    [
        p(a0, b0)
            .wrapping_sub(p(a1, b1))
            .wrapping_sub(p(a2, b2))
            .wrapping_sub(p(a3, b3)),
        p(a0, b1)
            .wrapping_add(p(a1, b0))
            .wrapping_add(p(a2, b3))
            .wrapping_sub(p(a3, b2)),
        p(a0, b2)
            .wrapping_sub(p(a1, b3))
            .wrapping_add(p(a2, b0))
            .wrapping_add(p(a3, b1)),
        p(a0, b3)
            .wrapping_add(p(a1, b2))
            .wrapping_sub(p(a2, b1))
            .wrapping_add(p(a3, b0)),
    ]
}

// ---------------------------------------------------------------------------
// The icosian transport snap (S1.4): exact selection over the 120 roots of
// 2I and the snapped transition, both multiplier-free.
//
// Root `j` has coordinates `(a_c + b_c phi) / 2` with `a_c, b_c` the small
// signed coefficients of [`H4_ROOT_COEFFICIENTS`] (in `-2..=2`), so a dot
// product with the raw rotation logits is `A + B phi` over
// `A = sum a_c raw_c`, `B = sum b_c raw_c` (the factor 1/2 cancels from every
// comparison), and the scaled transition is `(lambda a + lambda_phi b) / 2`
// per coordinate. Selection replaces the best root only on a strictly
// greater exact score, so ties keep the lowest index, the rule of the float
// reference's `nearest_root`.

/// `v k` for a root coefficient `k` in `-2..=2`: shifts and negation only.
#[inline(always)]
pub(crate) fn small_mul(v: i64, k: i8) -> i64 {
    match k {
        -2 => (v << 1).wrapping_neg(),
        -1 => v.wrapping_neg(),
        0 => 0,
        1 => v,
        2 => v << 1,
        // Unreachable for the icosian coefficient table (|k| <= 2); the
        // shift-add product keeps even a corrupted table multiplier-free.
        _ => {
            debug_assert!(false, "small_mul: coefficient {k} outside -2..=2");
            let mut remaining = k.unsigned_abs();
            let mut addend = v;
            let mut term = 0i64;
            while remaining != 0 {
                if remaining & 1 != 0 {
                    term = term.wrapping_add(addend);
                }
                remaining >>= 1;
                if remaining != 0 {
                    addend <<= 1;
                }
            }
            if k < 0 {
                term.wrapping_neg()
            } else {
                term
            }
        }
    }
}

/// `left + right` modulo 2^128 behind a call boundary, so the compiler
/// cannot recognize `4 s + s` at the call site as a product by five and emit
/// a multiplier instruction (the h4 classifier's `checked_add_square_terms`
/// observed exactly that).
#[inline(never)]
fn snap_add_square_terms(left: u128, right: u128) -> u128 {
    left.wrapping_add(right)
}

/// Whether `da + db phi > 0`, exactly, for the bounded score differences of
/// [`stack_snap_select`] (`|da| <= 2^35, |db| <= 2^34`): with
/// `P = 2 da + db`, `2 (da + db phi) = P + db sqrt(5)`, and `sqrt(5)` is
/// irrational, so the sign is decided by `P` and by `P^2` against
/// `5 db^2` (never equal for nonzero `P, db`).
fn snap_difference_positive(da: i64, db: i64) -> bool {
    let p = (da << 1).wrapping_add(db);
    if db == 0 {
        return p > 0;
    }
    if db > 0 {
        // P + db sqrt(5) > 0 iff P >= 0, or -P < db sqrt(5).
        if p >= 0 {
            return true;
        }
        let b_square = stack_mul_u128(db as u128, db as u128);
        let p_square = stack_mul_u128(p.unsigned_abs() as u128, p.unsigned_abs() as u128);
        return snap_add_square_terms(b_square << 2, b_square) > p_square;
    }
    // db < 0: P + db sqrt(5) > 0 iff P > 0 and P > -db sqrt(5).
    if p <= 0 {
        return false;
    }
    let b_square = stack_mul_u128(db.unsigned_abs() as u128, db.unsigned_abs() as u128);
    let p_square = stack_mul_u128(p as u128, p as u128);
    p_square > snap_add_square_terms(b_square << 2, b_square)
}

/// The index of the icosian root with the largest exact dot product with the
/// raw rotation logits `raw` (any common scale; the float reference snaps the
/// unit quaternion `raw / sqrt(|raw|^2 + 1e-6)`, a positive rescaling, so the
/// argmax is the same), the lowest index on an exact tie. Exact integer
/// arithmetic throughout: no normalization, no float, no multiply.
#[inline(never)]
pub fn stack_snap_select(raw: [i32; 4]) -> usize {
    let mut best = 0usize;
    let (mut best_a, mut best_b) = (0i64, 0i64);
    for (index, root) in H4_ROOT_COEFFICIENTS.iter().enumerate() {
        let (mut a, mut b) = (0i64, 0i64);
        for (coefficient, &value) in root.iter().zip(&raw) {
            let v = i64::from(value);
            a = a.wrapping_add(small_mul(v, coefficient[0]));
            b = b.wrapping_add(small_mul(v, coefficient[1]));
        }
        if index == 0 || snap_difference_positive(a.wrapping_sub(best_a), b.wrapping_sub(best_b)) {
            best = index;
            best_a = a;
            best_b = b;
        }
    }
    best
}

/// The snapped transition quaternion: the root [`stack_snap_select`] chooses
/// for `raw`, scaled by `lambda` (Q31), at Q31 — per coordinate
/// `shift(lambda a + lambda_phi b, 1)` with `lambda_phi = lambda phi` at the
/// same scale, matching `stack_rotation`'s `lambda unit` for `unit` replaced
/// by the root. The Hamilton product that consumes it is unchanged.
#[inline(never)]
pub(crate) fn stack_snap_rotation(raw: [i32; 4], lambda: u64) -> [i64; 4] {
    let root = &H4_ROOT_COEFFICIENTS[stack_snap_select(raw)];
    // lambda phi at Q31, rounded: with PHI_Q32 = 2^32 + PHI_LO,
    // (lambda PHI_Q32 + 2^31) >> 32 = lambda + ((lambda PHI_LO + 2^31) >> 32)
    // in u64 (lambda < 2^32, so lambda PHI_LO + 2^31 < 2^64). A u128 shift
    // here would compile to NEON register moves the audit forbids.
    debug_assert!(lambda < (1 << 32));
    let lambda_phi = lambda.wrapping_add(
        stack_mul_u64(lambda, (PHI_Q32 - (1 << 32)) as u64).wrapping_add(1 << 31) >> 32,
    );
    // An explicit loop, not `root.map`: the closure of `map` compiles to a
    // `core::array` symbol of its own, outside the audited `stack_` names.
    let mut transition = [0i64; 4];
    for (slot, coefficient) in transition.iter_mut().zip(root) {
        let sum = small_mul(lambda as i64, coefficient[0])
            .wrapping_add(small_mul(lambda_phi as i64, coefficient[1]));
        // `black_box` keeps LLVM from packing the four lanes into a NEON
        // pair (`fmov`/`dup`), which the serving audit forbids.
        *slot = std::hint::black_box(shift(sum, 1));
    }
    transition
}

/// Index of the largest value (first on ties).
#[inline(never)]
pub fn stack_argmax(values: &[i32]) -> usize {
    let mut best = 0;
    let mut top = i32::MIN;
    for (i, &v) in values.iter().enumerate() {
        if i == 0 || v > top {
            best = i;
            top = v;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 ^ (self.0 >> 29)
        }

        fn wide(&mut self) -> u128 {
            (u128::from(self.next()) << 64) | u128::from(self.next())
        }
    }

    #[test]
    fn products_match_wrapping_multiplication() {
        let mut rng = Lcg(3);
        let edges = [
            0u128,
            1,
            2,
            15,
            16,
            17,
            u128::from(u64::MAX),
            u128::MAX,
            1 << 127,
        ];
        for &a in &edges {
            for &b in &edges {
                assert_eq!(stack_mul_u128(a, b), a.wrapping_mul(b), "{a} {b}");
            }
        }
        for _ in 0..20_000 {
            let (a, b) = (
                rng.wide() >> (rng.next() % 128),
                rng.wide() >> (rng.next() % 128),
            );
            assert_eq!(stack_mul_u128(a, b), a.wrapping_mul(b));
            let (x, y) = (a as u64, b as u64);
            assert_eq!(stack_mul_u64(x, y), x.wrapping_mul(y));
            let (s, t) = (a as i128, b as i128);
            assert_eq!(mul_i128(s, t), s.wrapping_mul(t));
            let (s, t) = (a as i64, b as i64);
            assert_eq!(mul_i64(s, t), s.wrapping_mul(t));
        }
        for (s, t) in [
            (i128::MIN, -1),
            (i128::MIN, i128::MIN),
            (-5, 7),
            (i128::MAX, 3),
        ] {
            assert_eq!(mul_i128(s, t), s.wrapping_mul(t));
        }
    }

    #[test]
    fn table_products_are_exact_over_the_i32_range() {
        let mut rng = Lcg(7);
        let zero = black_box(0i64);
        let values = [
            0i64,
            1,
            -1,
            7,
            -8,
            i64::from(i32::MAX),
            i64::from(i32::MIN),
            (1 << 32) - 1,
        ];
        let ys = [0i32, 1, -1, 15, -16, i32::MAX, i32::MIN, 65_536, -65_537];
        for &x in &values {
            let table = multiples_i64(x, zero);
            for &y in &ys {
                assert_eq!(mul_by_i32(&table, y), x * i64::from(y), "{x} {y}");
            }
        }
        for _ in 0..20_000 {
            let x = (rng.next() as i64) >> 32;
            let y = rng.next() as i32;
            let table = multiples_i64(x, zero);
            assert_eq!(mul_by_i32(&table, y), x * i64::from(y));
            let v = i64::from(rng.next() as i32);
            assert_eq!(u128::from(stack_square(v)), (v as i128 * v as i128) as u128);
        }
        assert_eq!(stack_square(1 << 31), 1 << 62);
        assert_eq!(
            stack_square((1 << 32) - 1),
            ((1u64 << 32) - 1).wrapping_mul((1 << 32) - 1)
        );
    }

    #[test]
    fn long_division_and_roots_match_the_native_operators() {
        let mut rng = Lcg(11);
        for _ in 0..20_000 {
            let n = rng.wide() >> (rng.next() % 128);
            let d = (rng.wide() >> (rng.next() % 128)).max(1);
            assert_eq!(stack_div_u128(n, d), n / d);
            let (n64, d64) = (n as u64, (d as u64).max(1));
            assert_eq!(stack_div_u64(n64, d64), n64 / d64);
            let root = stack_isqrt(n);
            assert!(root * root <= n && (root + 1).checked_mul(root + 1).is_none_or(|sq| sq > n));
        }
        assert_eq!(stack_div_u128(u128::MAX, 1), u128::MAX);
        assert_eq!(stack_div_u64(u64::MAX, u64::MAX), 1);
        assert_eq!(stack_div_u64(7, 0), 0);
        assert_eq!(stack_isqrt(u128::MAX), u128::from(u64::MAX));
    }

    #[test]
    fn activation_tables_hold_every_nibble_product() {
        let x: Vec<i16> = vec![-32768, -32767, -1, 0, 1, 255, 32767, 12345];
        let mut tables = vec![[0i32; 16]; x.len()];
        stack_activation_tables(&x, &mut tables);
        for (table, &v) in tables.iter().zip(&x) {
            for (u, &entry) in table.iter().enumerate() {
                assert_eq!(entry, (u as i32 - 8) * i32::from(v));
            }
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn both_weight_map_kernels_give_the_exact_group_scaled_sum() {
        let mut rng = Lcg(21);
        for (rows, cols, x_exp) in [(1, 32, -14), (17, 64, -9), (40, 96, -20), (9, 288, -14)] {
            let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| rng.next() as u8).collect();
            let scales: Vec<u8> = (0..rows * cols / 32).map(|_| rng.next() as u8).collect();
            let groups = cols / 32;
            let min_de: Vec<u8> = scales
                .chunks_exact(groups)
                .map(|row| row.iter().map(|s| s >> 4).min().unwrap_or(0))
                .collect();
            let m = PackedMatrix {
                rows,
                cols,
                exp_base: -11,
                nibbles,
                scales,
                min_de,
            };
            let x: Vec<i16> = (0..cols)
                .map(|c| match c % 7 {
                    0 => i16::MIN + 1,
                    1 => i16::MAX,
                    _ => rng.next() as i16,
                })
                .collect();
            let mut tables = vec![[0i32; 16]; cols];
            stack_activation_tables(&x, &mut tables);
            let mut pairs = vec![[0i32; 256]; cols / 2];
            stack_pair_tables(&tables, &mut pairs);
            let (mut nibble_out, mut pair_out) = (vec![0i32; rows], vec![0i32; rows]);
            stack_gemv(&m, &tables, x_exp, &mut nibble_out);
            stack_gemv_pairs(&m, &pairs, x_exp, &mut pair_out);
            assert_eq!(nibble_out, pair_out, "{rows}x{cols}");
            for r in 0..rows {
                let mut exact = 0i128;
                for c in 0..cols {
                    let byte = m.nibbles[(r * cols + c) / 2];
                    let q = i128::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                    let s = m.scales[r * groups + c / 32];
                    exact += (q * (16 + i128::from(s & 15)) * i128::from(x[c]))
                        << ((s >> 4) - m.min_de[r]);
                }
                let want = to_exp_i32(
                    exact as i64,
                    m.exp_base + i32::from(m.min_de[r]) - 4 + x_exp,
                    RESIDUAL_EXP,
                );
                assert_eq!(nibble_out[r], want, "{rows}x{cols} row {r}");
            }
        }
    }

    /// A mis-sized call, unreachable after loading: a debug build stops at an
    /// assertion, a release build returns with the rows it could read and
    /// never reads out of bounds.
    #[test]
    fn mis_sized_calls_assert_in_debug_and_return_in_release() {
        // Two rows declared, one row of nibbles present.
        let m = PackedMatrix {
            rows: 2,
            cols: 32,
            exp_base: 0,
            nibbles: vec![0x99; 16],
            scales: vec![0; 2],
            min_de: vec![0; 2],
        };
        let mut tables = vec![[0i32; 16]; 32];
        stack_activation_tables(&[1i16; 32], &mut tables);
        let mut pairs = vec![[0i32; 256]; 16];
        stack_pair_tables(&tables, &mut pairs);
        let panics = |f: &mut dyn FnMut()| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
        };
        let untouched = i32::MIN;
        for pairs_kernel in [false, true] {
            let mut out = vec![untouched; 2];
            let panicked = panics(&mut || {
                if pairs_kernel {
                    stack_gemv_pairs(&m, &pairs, -14, &mut out);
                } else {
                    stack_gemv(&m, &tables, -14, &mut out);
                }
            });
            assert_eq!(panicked, cfg!(debug_assertions), "pairs {pairs_kernel}");
            if !panicked {
                assert_ne!(out[0], untouched, "pairs {pairs_kernel}: the readable row");
                assert_eq!(out[1], untouched, "pairs {pairs_kernel}: the missing row");
            }
        }
        let mut row = vec![untouched; 32];
        let panicked = panics(&mut || stack_dequant_row(&m, 1, &mut row));
        assert_eq!(panicked, cfg!(debug_assertions), "dequant_row");
        if !panicked {
            assert!(row.iter().all(|&v| v == untouched));
        }
    }

    #[test]
    fn argmax_takes_the_first_maximum() {
        assert_eq!(stack_argmax(&[3, 9, 9, -1]), 1);
        assert_eq!(stack_argmax(&[i32::MIN, i32::MIN]), 0);
        assert_eq!(stack_argmax(&[]), 0);
    }
}
