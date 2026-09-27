//! Integer kernels.
//!
//! Learned weight maps (`gemv`, `dequant_row`) use additions, subtractions,
//! shifts and table reads only; `gemv` runs the audited vector kernels of
//! `uor-r4-simd` (AVX2, NEON or portable, bit-identical). The other kernels multiply runtime values or
//! fixed non-learned constants, which owner decision D10 allows; the few
//! per-vector reciprocals (normalization, softmax) use one integer division
//! per vector.

use uor_r4_simd::{Backend, Interleaved, Tables, ROWS};

use crate::{invalid, Result};

/// Round-half-up arithmetic shift right by `shift` (left if negative),
/// saturating at the `i64` range.
pub fn shift(value: i64, shift: i32) -> i64 {
    if shift > 0 {
        if shift >= 63 {
            return if value < 0 { -1 } else { 0 };
        }
        let half = 1i64 << (shift - 1);
        value.saturating_add(half) >> shift
    } else if shift < 0 {
        let k = (-shift).min(63) as u32;
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

/// Convert `value * 2^from` to the nearest `v * 2^to`, saturated to `i32`.
pub fn to_exp_i32(value: i64, from: i32, to: i32) -> i32 {
    shift(value, to - from).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn bit_length(value: u64) -> i32 {
    64 - value.leading_zeros() as i32
}

/// A vector of `i16` with one power-of-two exponent.
#[derive(Clone, Debug, Default)]
pub struct Act16 {
    pub values: Vec<i16>,
    pub exp: i32,
}

/// Requantize `values * 2^exp` to 16 bits with the smallest exponent that fits.
pub fn quantize16(values: &[i64], exp: i32, out: &mut Act16) {
    let max = values.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let s = (bit_length(max) - 15).max(0);
    out.values.clear();
    out.values.extend(
        values
            .iter()
            .map(|&v| shift(v, s).clamp(-32767, 32767) as i16),
    );
    out.exp = exp + s;
}

/// Requantize `values * 2^exp` to 8 bits; returns the new exponent.
pub fn quantize8(values: &[i64], exp: i32, out: &mut [i8]) -> i32 {
    let max = values.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let s = (bit_length(max) - 7).max(0);
    for (o, &v) in out.iter_mut().zip(values) {
        *o = shift(v, s).clamp(-127, 127) as i8;
    }
    exp + s
}

/// `(16 + m) * a` by shifts and adds (`m < 16`).
#[inline]
pub fn scale_16_plus(a: i64, m: u8) -> i64 {
    let mut t = a << 4;
    for bit in 0..4 {
        if m & (1 << bit) != 0 {
            t += a << bit;
        }
    }
    t
}

/// One row-major packed matrix (the embedding, read one row at a time).
pub struct MatrixView<'a> {
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: &'a [u8],
    pub scales: &'a [u8],
}

/// A projection matrix repacked for the vector table kernels of `uor-r4-simd`.
pub struct Packed {
    pub exp_base: i32,
    pub matrix: Interleaved,
}

impl Packed {
    pub fn new(view: &MatrixView<'_>) -> Result<Self> {
        Ok(Self {
            exp_base: view.exp_base,
            matrix: Interleaved::new(view.rows, view.cols, view.nibbles, view.scales)?,
        })
    }
}

/// Matrices with at least this many weights split their rows across threads.
pub const PARALLEL_WEIGHTS: usize = 1 << 18;
/// Row blocks per task.
const TASK_BLOCKS: usize = 4;

/// `out[r] = sum_c W[r][c] x[c]` for `x` given by its tables (built from the
/// 16-bit activation at exponent `x_exp`), written at exponent `out_exp`.
/// Table reads, additions and shifts only: no multiplier touches a weight.
/// With `parallel`, large matrices split their row blocks across the current
/// rayon pool; every backend and every split gives the same integers.
pub fn gemv(
    p: &Packed,
    backend: Backend,
    parallel: bool,
    tables: &Tables,
    x_exp: i32,
    out: &mut [i32],
    out_exp: i32,
) -> Result<()> {
    use rayon::prelude::*;
    let m = &p.matrix;
    let rows = m.rows();
    if out.len() < rows {
        return Err(invalid("gemv output is shorter than the matrix"));
    }
    let min_de = m.min_de();
    let task = |(t, chunk): (usize, &mut [i32])| -> Result<()> {
        let first = t * TASK_BLOCKS;
        let mut acc = [0i64; ROWS * TASK_BLOCKS];
        let acc = &mut acc[..chunk.len().div_ceil(ROWS) * ROWS];
        m.gemv_blocks_with(backend, tables, first, acc)?;
        for (i, (slot, value)) in chunk.iter_mut().zip(acc.iter()).enumerate() {
            let from = p.exp_base + i32::from(min_de[first * ROWS + i]) - 4 + x_exp;
            *slot = to_exp_i32(*value, from, out_exp);
        }
        Ok(())
    };
    let out = &mut out[..rows];
    if parallel && rows * m.cols() >= PARALLEL_WEIGHTS {
        out.par_chunks_mut(ROWS * TASK_BLOCKS)
            .enumerate()
            .try_for_each(task)
    } else {
        out.chunks_mut(ROWS * TASK_BLOCKS)
            .enumerate()
            .try_for_each(task)
    }
}

/// Row `row` of a packed matrix, written at exponent `out_exp` (embedding lookup).
pub fn dequant_row(m: &MatrixView<'_>, row: usize, out: &mut [i32], out_exp: i32) {
    let groups = m.cols / crate::GROUP;
    let nibbles = &m.nibbles[row * m.cols / 2..(row + 1) * m.cols / 2];
    let scales = &m.scales[row * groups..(row + 1) * groups];
    for (c, slot) in out.iter_mut().enumerate().take(m.cols) {
        let byte = nibbles[c / 2];
        let q = i64::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
        let scale = scales[c / crate::GROUP];
        let value = scale_16_plus(q, scale & 15);
        *slot = to_exp_i32(value, m.exp_base + i32::from(scale >> 4) - 4, out_exp);
    }
}

/// Floor integer square root (digit by digit: shifts, compares, subtractions).
pub fn isqrt(value: u128) -> u128 {
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

/// RMSNorm without its gain (folded into the following weights): `x / rms(x)`,
/// for `x` at exponent `x_exp`, into `out` (16-bit, fitted exponent).
pub fn rms_norm(
    x: &[i32],
    x_exp: i32,
    eps: crate::format::Fixed,
    scratch: &mut Vec<i64>,
    out: &mut Act16,
) {
    let max = x.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let sh = (bit_length(u64::from(max)) - 24).max(0);
    scratch.clear();
    scratch.extend(x.iter().map(|&v| shift(i64::from(v), sh)));
    let sum: i64 = scratch.iter().map(|&v| v * v).sum();
    let mean = sum / x.len().max(1) as i64;
    let eps_int = shift(eps.mantissa, 2 * (x_exp + sh) - eps.exp).max(0);
    let ms = (mean + eps_int).max(1) as u128;
    // s = sqrt(ms) 2^34, r = 2^96 / s = 2^62 / sqrt(ms)
    let s = isqrt(ms << 68).max(1);
    let r = (1u128 << 96) / s;
    for v in scratch.iter_mut() {
        let y = (i128::from(*v) * r as i128) >> 48; // y * 2^14
        *v = y.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    }
    quantize16(scratch, -14, out);
}

/// Half-split RoPE on one head (`head.len()` even), values at any exponent.
pub fn rope(head: &mut [i32], position: usize, cos: &[i16], sin: &[i16], q: u32) {
    let half = head.len() / 2;
    let row = position * half;
    for i in 0..half {
        let (a, b) = (i64::from(head[i]), i64::from(head[i + half]));
        let (c, s) = (i64::from(cos[row + i]), i64::from(sin[row + i]));
        head[i] =
            shift(a * c - b * s, q as i32).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        head[i + half] =
            shift(b * c + a * s, q as i32).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    }
}

/// `round(2^31 exp(-d))` for `d = value * 2^exp_d`, from the sealed table with
/// step `2^step_log2` and linear interpolation.
pub fn exp_neg(d: i64, exp_d: i32, table: &[u32], step_log2: i32) -> u64 {
    let frac_bits = step_log2 - exp_d;
    if d <= 0 {
        return u64::from(table[0]);
    }
    if frac_bits <= 0 {
        let index = (d << (-frac_bits)) as usize;
        return table.get(index).map_or(0, |v| u64::from(*v));
    }
    let index = (d >> frac_bits) as usize;
    if index + 1 >= table.len() {
        return 0;
    }
    let frac = (d & ((1 << frac_bits) - 1)) as u64;
    let (a, b) = (u64::from(table[index]), u64::from(table[index + 1]));
    a - (((a - b) * frac) >> frac_bits)
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

/// SiLU of `x * 2^-16`, returned at exponent -16, from the sealed table.
pub fn silu(x: i32, table: &[i32], step_log2: i32, range_log2: i32) -> i32 {
    // x is in units of 2^-16 and the table step is 2^step_log2 (-16 <= step_log2 < 0).
    let frac_bits = step_log2 + 16;
    debug_assert!((0..=16).contains(&frac_bits));
    let half = 1i64 << (range_log2 - step_log2);
    let index = (i64::from(x) >> frac_bits) + half;
    if index < 0 {
        return 0;
    }
    if index as usize + 1 >= table.len() {
        return x;
    }
    let frac = i64::from(x) & ((1i64 << frac_bits) - 1);
    let (a, b) = (
        i64::from(table[index as usize]),
        i64::from(table[index as usize + 1]),
    );
    (a + (((b - a) * frac) >> frac_bits)) as i32
}

/// Index of the largest value (first on ties).
pub fn argmax(values: &[i32]) -> usize {
    let mut best = 0;
    for (i, v) in values.iter().enumerate() {
        if *v > values[best] {
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_by_shift_add_equals_product() {
        for a in [-1_000_000i64, -3, 0, 7, 123_456_789] {
            for m in 0..16u8 {
                assert_eq!(scale_16_plus(a, m), a * (16 + i64::from(m)));
            }
        }
    }

    struct Lcg(u32);

    impl Lcg {
        fn byte(&mut self) -> u8 {
            self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (self.0 >> 16) as u8
        }
    }

    /// `gemv` on every available backend, serial and threaded, against the
    /// exact `i128` product.
    fn check_gemv(rows: usize, cols: usize, seed: u32, exp_base: i32, x_exp: i32, out_exp: i32) {
        let mut rng = Lcg(seed);
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| rng.byte()).collect();
        // de in 0..=3 keeps the exact result inside i32 at the output exponent
        let scales: Vec<u8> = (0..rows * cols / crate::GROUP)
            .map(|_| rng.byte() & 0x3F)
            .collect();
        let x: Vec<i16> = (0..cols)
            .map(|i| ((i * 131) % 4001) as i16 - 2000)
            .collect();
        let view = MatrixView {
            rows,
            cols,
            exp_base,
            nibbles: &nibbles,
            scales: &scales,
        };
        let packed = Packed::new(&view).unwrap();
        let mut tables = Tables::default();
        tables.build(&x).unwrap();
        for backend in [Backend::Portable, Backend::Avx2, Backend::Neon] {
            if !backend.available() {
                continue;
            }
            let mut out = vec![0i32; rows];
            for parallel in [false, true] {
                gemv(
                    &packed, backend, parallel, &tables, x_exp, &mut out, out_exp,
                )
                .unwrap();
                for (r, got) in out.iter().enumerate() {
                    let mut exact = 0i128;
                    for c in 0..cols {
                        let byte = nibbles[(r * cols + c) / 2];
                        let q = i128::from(if c % 2 == 0 { byte & 15 } else { byte >> 4 }) - 8;
                        let s = scales[(r * cols + c) / crate::GROUP];
                        exact += (q * (16 + i128::from(s & 15)) * i128::from(x[c])) << (s >> 4);
                    }
                    assert_eq!(
                        i128::from(*got),
                        exact,
                        "{} row {r} parallel {parallel}",
                        backend.name()
                    );
                }
            }
        }
    }

    #[test]
    fn gemv_matches_the_exact_product() {
        // exact products are at exponent exp_base - 4 + x_exp = out_exp
        check_gemv(3, 64, 12345, -6, -3, -13);
        check_gemv(45, 96, 7, -6, -3, -13);
    }

    #[test]
    fn threaded_gemv_matches_the_exact_product() {
        const { assert!(1024 * 256 >= PARALLEL_WEIGHTS) };
        check_gemv(1024, 256, 99, -2, -8, -14);
    }

    #[test]
    fn simd_value_mixing_matches_this_crate_s_shift() {
        let mut rng = Lcg(4242);
        let values: Vec<i8> = (0..64)
            .map(|i| {
                if i % 13 == 0 {
                    -127
                } else {
                    (rng.byte() as i8).max(-127)
                }
            })
            .collect();
        for w in [0i32, 1, 7, 1 << 12, (1 << 24) - 1, 1 << 24] {
            for down in 0..70 {
                let mut got = vec![5i64; values.len()];
                uor_r4_simd::mix_rows(&[w], &[down], &values, 64, 0, &mut got).unwrap();
                for (g, v) in got.iter().zip(&values) {
                    let want = if w == 0 {
                        5
                    } else {
                        5 + shift(i64::from(w) * i64::from(*v), down)
                    };
                    assert_eq!(*g, want, "w {w} down {down} v {v}");
                }
            }
        }
    }

    #[test]
    fn integer_square_root_is_exact() {
        for v in [
            0u128,
            1,
            2,
            3,
            4,
            15,
            16,
            17,
            1 << 40,
            (1 << 100) + 12345,
            u128::MAX >> 2,
        ] {
            let r = isqrt(v);
            assert!(r * r <= v && (r + 1) * (r + 1) > v, "{v}");
        }
    }

    #[test]
    fn rms_norm_normalizes() {
        let x: Vec<i32> = (0..64).map(|i| (i - 32) * 40_000).collect();
        let eps = crate::format::Fixed {
            mantissa: 1,
            exp: -40,
        };
        let (mut scratch, mut out) = (Vec::new(), Act16::default());
        rms_norm(&x, -16, eps, &mut scratch, &mut out);
        let sum: f64 = out
            .values
            .iter()
            .map(|v| (f64::from(*v) * 2f64.powi(out.exp)).powi(2))
            .sum();
        assert!(
            (sum / 64.0 - 1.0).abs() < 1e-3,
            "mean square {}",
            sum / 64.0
        );
    }

    #[test]
    fn silu_table_lookup_matches_the_function() {
        // The exporter's table: step 2^-8, half range 16, Q16 entries.
        let (step, range) = (-8, 4);
        let half = 1i64 << (range - step);
        let table: Vec<i32> = (0..=2 * half)
            .map(|i| {
                let x = (i - half) as f64 / 256.0;
                (65536.0 * x / (1.0 + (-x).exp())).round() as i32
            })
            .collect();
        for x in [
            -20.0f64, -15.99, -3.3, -0.51, 0.0, 0.004, 0.7, 2.25, 9.9, 15.9, 40.0,
        ] {
            let fixed = (x * 65536.0).round() as i32;
            let got = f64::from(silu(fixed, &table, step, range)) / 65536.0;
            let want = x / (1.0 + (-x).exp());
            assert!((got - want).abs() < 2e-4, "silu({x}) = {got}, want {want}");
        }
    }

    #[test]
    fn exp_table_lookup_matches_the_function() {
        let step = -8;
        let table: Vec<u32> = (0..32 * 256 + 2)
            .map(|i| (2f64.powi(31) * (-(i as f64) / 256.0).exp()).round() as u32)
            .collect();
        for d in [0.0f64, 0.001, 0.5, 1.0, 3.7, 12.25, 31.9, 40.0] {
            let fixed = (d * 4096.0).round() as i64; // exponent -12
            let got = exp_neg(fixed, -12, &table, step) as f64 / 2f64.powi(31);
            // compare at the kernel's input resolution (2^-12 nats)
            let want = (-(fixed as f64) / 4096.0).exp();
            assert!((got - want).abs() < 2e-6, "exp(-{d}) = {got}, want {want}");
        }
    }
}
