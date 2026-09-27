//! Vector kernels for the integer table GEMV of `uor-r4-lut`.
//!
//! This is the one audited serving crate that owner decision D10 allows to use
//! `unsafe`. The portable kernel in this file defines the arithmetic; the AVX2
//! (x86-64) and NEON (AArch64) kernels must reproduce it bit for bit, which the
//! tests check on random and extreme matrices for every backend the host can
//! run (AArch64 can be checked on x86-64 under user-mode emulation).
//!
//! # Arithmetic
//!
//! A learned weight is a 4-bit offset-binary nibble `u = q + 8`, `q` in
//! `-8..=7`, with one scale byte per row and group of [`GROUP`] columns: low
//! nibble `m`, high nibble `de`, meaning `(16 + m) 2^(exp_base + de - 4)`.
//! A 16-bit activation is split as `x = 256 xh + xl` with `xh = (x + 128) >> 8`
//! in `-128..=128` and `xl` in `-128..=127`. [`Tables::build`] gives each column
//! two 16-entry tables `A[u] = q xh` and `B[u] = q xl` (shifts and additions;
//! entries within ±1024), stored as low/high byte planes. A weight contributes
//! through table reads (`vpshufb`/`tbl`, sixteen rows per instruction) and
//! additions:
//!
//! - `a(row, group) = sum_c 256 A_c[u] + B_c[u] = sum_c q x_c`, exactly;
//! - `acc(row) = sum_g ((16 + m) a) << (de - de_min(row))`, the scale applied
//!   by shifts, masks and additions.
//!
//! No multiplier touches a learned weight or scale. The 16-bit partial sums
//! cover at most sixteen columns (at most `16 * 1024`), so they cannot wrap.
//!
//! # `unsafe` audit
//!
//! Every `unsafe` block is one of:
//! 1. a call to a `#[target_feature]` kernel after [`Backend::available`]
//!    confirmed the feature on the running CPU;
//! 2. an unaligned vector load or store through the pointer of a slice that
//!    was just re-sliced to exactly the vector's width (the slicing is
//!    bounds-checked).
//!
//! There are no transmutes of non-vector data, no aliasing and no allocation
//! inside the kernels.

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::fmt;

/// Rows per interleaved block (one vector of byte lanes).
pub const ROWS: usize = 16;
/// Columns per scale group.
pub const GROUP: usize = 32;
/// Bytes of tables per column pair: `A` low/high planes, then `B` low/high,
/// each plane holding the even column's 16 entries, then the odd column's.
const PAIR_TABLE: usize = 128;
/// Column pairs per group.
const GROUP_PAIRS: usize = GROUP / 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Portable,
    Avx2,
    Neon,
}

impl Backend {
    /// The fastest backend the running CPU supports.
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        if std::is_x86_feature_detected!("avx2") {
            return Self::Avx2;
        }
        #[cfg(target_arch = "aarch64")]
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Self::Neon;
        }
        Self::Portable
    }

    /// Whether the running CPU can execute this backend.
    pub fn available(self) -> bool {
        match self {
            Self::Portable => true,
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => std::is_x86_feature_detected!("avx2"),
            #[cfg(target_arch = "aarch64")]
            Self::Neon => std::arch::is_aarch64_feature_detected!("neon"),
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::Avx2 => "avx2",
            Self::Neon => "neon",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimdError {
    Shape(String),
    Unsupported(Backend),
}

impl fmt::Display for SimdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(message) => write!(f, "shape: {message}"),
            Self::Unsupported(backend) => {
                write!(f, "the CPU does not support the {} backend", backend.name())
            }
        }
    }
}

impl std::error::Error for SimdError {}

fn shape(message: impl Into<String>) -> SimdError {
    SimdError::Shape(message.into())
}

/// A 4-bit matrix repacked for vector table reads: rows in blocks of
/// [`ROWS`], and within a block, for every column pair, one byte per row
/// holding the even column's nibble (low) and the odd column's (high). Rows
/// past the end are padding with weight zero.
#[derive(Clone, Debug)]
pub struct Interleaved {
    rows: usize,
    cols: usize,
    blocks: usize,
    /// `[block][pair][row in block]`.
    nibbles: Vec<u8>,
    /// `[block][group][row in block]`.
    scales: Vec<u8>,
    /// `[block][row in block]`: smallest `de` of the row.
    min_de: Vec<u8>,
}

impl Interleaved {
    /// Repack a row-major matrix (`rows * cols / 2` nibble bytes, low nibble
    /// the even column; `rows * cols / GROUP` scale bytes).
    pub fn new(rows: usize, cols: usize, nibbles: &[u8], scales: &[u8]) -> Result<Self, SimdError> {
        if rows == 0
            || cols == 0
            || !cols.is_multiple_of(GROUP)
            || nibbles.len() != rows * cols / 2
            || scales.len() != rows * cols / GROUP
        {
            return Err(shape(format!(
                "a {rows}x{cols} matrix needs {} nibble and {} scale bytes, got {} and {}",
                rows * cols / 2,
                rows * cols / GROUP,
                nibbles.len(),
                scales.len()
            )));
        }
        let (pairs, groups) = (cols / 2, cols / GROUP);
        let blocks = rows.div_ceil(ROWS);
        let mut packed = vec![0x88u8; blocks * pairs * ROWS];
        let mut block_scales = vec![0u8; blocks * groups * ROWS];
        let mut min_de = vec![0u8; blocks * ROWS];
        for (r, (row, row_scales)) in nibbles
            .chunks_exact(pairs)
            .zip(scales.chunks_exact(groups))
            .enumerate()
        {
            let (b, k) = (r / ROWS, r % ROWS);
            for (p, &byte) in row.iter().enumerate() {
                packed[(b * pairs + p) * ROWS + k] = byte;
            }
            for (g, &s) in row_scales.iter().enumerate() {
                block_scales[(b * groups + g) * ROWS + k] = s;
            }
            min_de[r] = row_scales.iter().map(|s| s >> 4).min().unwrap_or(0);
        }
        Ok(Self {
            rows,
            cols,
            blocks,
            nibbles: packed,
            scales: block_scales,
            min_de,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn blocks(&self) -> usize {
        self.blocks
    }

    /// Smallest scale exponent offset of every row (padded to whole blocks).
    pub fn min_de(&self) -> &[u8] {
        &self.min_de
    }

    /// Accumulators of blocks `first..first + out.len() / ROWS` with the
    /// detected backend: `out[ROWS * i + k]` belongs to row `ROWS * (first + i) + k`
    /// and is at exponent `exp_base + min_de(row) - 4 + x_exp`.
    pub fn gemv_blocks(
        &self,
        tables: &Tables,
        first: usize,
        out: &mut [i64],
    ) -> Result<(), SimdError> {
        self.gemv_blocks_with(Backend::detect(), tables, first, out)
    }

    /// [`Self::gemv_blocks`] with an explicit backend.
    pub fn gemv_blocks_with(
        &self,
        backend: Backend,
        tables: &Tables,
        first: usize,
        out: &mut [i64],
    ) -> Result<(), SimdError> {
        if tables.cols != self.cols {
            return Err(shape(format!(
                "tables for {} columns used with a {}-column matrix",
                tables.cols, self.cols
            )));
        }
        let (blocks, rest) = out.as_chunks_mut::<ROWS>();
        if !rest.is_empty() || first + blocks.len() > self.blocks {
            return Err(shape(
                "output is not a whole number of blocks inside the matrix",
            ));
        }
        if !backend.available() {
            return Err(SimdError::Unsupported(backend));
        }
        for (i, block) in blocks.iter_mut().enumerate() {
            let b = first + i;
            match backend {
                Backend::Portable => portable_block(self, &tables.bytes, b, block),
                #[cfg(target_arch = "x86_64")]
                // SAFETY: `backend.available()` confirmed AVX2 on this CPU.
                Backend::Avx2 => unsafe { avx2::gemv_block(self, &tables.bytes, b, block) },
                #[cfg(target_arch = "aarch64")]
                // SAFETY: `backend.available()` confirmed NEON on this CPU.
                Backend::Neon => unsafe { neon::gemv_block(self, &tables.bytes, b, block) },
                #[allow(unreachable_patterns)]
                _ => return Err(SimdError::Unsupported(backend)),
            }
        }
        Ok(())
    }
}

/// Byte-plane tables of one 16-bit activation vector (see the crate docs).
#[derive(Clone, Debug, Default)]
pub struct Tables {
    cols: usize,
    /// `[pair][plane: A lo, A hi, B lo, B hi][even column 16 | odd column 16]`.
    bytes: Vec<u8>,
}

impl Tables {
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Rebuild the tables for `x` (length a multiple of [`GROUP`]) with shifts,
    /// additions and negations only.
    pub fn build(&mut self, x: &[i16]) -> Result<(), SimdError> {
        if x.is_empty() || !x.len().is_multiple_of(GROUP) {
            return Err(shape(format!(
                "activation length {} is not a positive multiple of {GROUP}",
                x.len()
            )));
        }
        self.cols = x.len();
        self.bytes.clear();
        self.bytes.resize(x.len() / 2 * PAIR_TABLE, 0);
        let (pairs, _) = self.bytes.as_chunks_mut::<PAIR_TABLE>();
        let (values, _) = x.as_chunks::<2>();
        for (out, values) in pairs.iter_mut().zip(values) {
            for (half, &value) in values.iter().enumerate() {
                let v = i32::from(value);
                let high = (v + 128) >> 8;
                let (a, b) = (multiples(high), multiples(v - (high << 8)));
                let at = half * 16;
                for u in 0..16 {
                    let [a_lo, a_hi] = a[u].to_le_bytes();
                    let [b_lo, b_hi] = b[u].to_le_bytes();
                    out[at + u] = a_lo;
                    out[32 + at + u] = a_hi;
                    out[64 + at + u] = b_lo;
                    out[96 + at + u] = b_hi;
                }
            }
        }
        Ok(())
    }
}

/// `[q * v for q in -8..=7]` for `|v| <= 128`, by shifts, additions and negations.
fn multiples(v: i32) -> [i16; 16] {
    let (v2, v4) = (v << 1, v << 2);
    let positive = [0, v, v2, v2 + v, v4, v4 + v, v4 + v2, v4 + v2 + v];
    let mut table = [0i16; 16];
    for (q, p) in positive.iter().enumerate() {
        table[8 + q] = *p as i16;
        if q > 0 {
            table[8 - q] = (-*p) as i16;
        }
    }
    table[0] = (-(v << 3)) as i16;
    table
}

/// `(16 + m) a` by shifts and additions (`m < 16`).
#[inline]
fn scale_16_plus(a: i64, m: u8) -> i64 {
    let mut t = a << 4;
    for bit in 0..4 {
        if m & (1 << bit) != 0 {
            t += a << bit;
        }
    }
    t
}

/// The reference kernel: one block of [`ROWS`] rows.
fn portable_block(m: &Interleaved, tables: &[u8], b: usize, out: &mut [i64; ROWS]) {
    let (pairs, groups) = (m.cols / 2, m.cols / GROUP);
    let min_de = &m.min_de[b * ROWS..(b + 1) * ROWS];
    let mut acc = [0i64; ROWS];
    for g in 0..groups {
        let mut a = [0i32; ROWS];
        for p in g * GROUP_PAIRS..(g + 1) * GROUP_PAIRS {
            let nibbles = &m.nibbles[(b * pairs + p) * ROWS..(b * pairs + p + 1) * ROWS];
            let t = &tables[p * PAIR_TABLE..(p + 1) * PAIR_TABLE];
            for (slot, &byte) in a.iter_mut().zip(nibbles) {
                for (half, u) in [(0, byte & 15), (16, byte >> 4)] {
                    let i = half + usize::from(u);
                    let high = i16::from_le_bytes([t[i], t[32 + i]]);
                    let low = i16::from_le_bytes([t[64 + i], t[96 + i]]);
                    *slot += (i32::from(high) << 8) + i32::from(low);
                }
            }
        }
        let scales = &m.scales[(b * groups + g) * ROWS..(b * groups + g + 1) * ROWS];
        for k in 0..ROWS {
            let s = scales[k];
            acc[k] += scale_16_plus(i64::from(a[k]), s & 15) << ((s >> 4) - min_de[k]);
        }
    }
    *out = acc;
}

/// Attention scores of one head: `out[u] = q . keys[u * stride + offset..][..q.len()]`
/// for `u < out.len()`, exactly (`|q| <= 2^15`, `|key| <= 2^7`, so the sum fits
/// `i32` for `q.len() <= 256`).
pub fn dot_rows_i16_i8(
    q: &[i16],
    keys: &[i8],
    stride: usize,
    offset: usize,
    out: &mut [i32],
) -> Result<(), SimdError> {
    check_rows(q.len(), keys.len(), stride, offset, out.len())?;
    #[cfg(target_arch = "x86_64")]
    if Backend::Avx2.available() {
        // SAFETY: `available()` confirmed AVX2 on this CPU.
        unsafe { auto_avx2::dot_rows(q, keys, stride, offset, out) };
        return Ok(());
    }
    dot_rows(q, keys, stride, offset, out);
    Ok(())
}

/// Value mixing of one head: `mix[i] += sum_u shift(w[u] values[u * stride +
/// offset + i], down[u])`, where `shift` rounds half up (`(p + 2^(d - 1)) >> d`)
/// exactly as `uor-r4-lut`'s `i64` shift. Requires `0 <= w <= 2^24`,
/// `|value| <= 127` (so `|w value| < 2^31`) and `down >= 0`.
pub fn mix_rows(
    weights: &[i32],
    downs: &[i32],
    values: &[i8],
    stride: usize,
    offset: usize,
    mix: &mut [i64],
) -> Result<(), SimdError> {
    check_rows(mix.len(), values.len(), stride, offset, weights.len())?;
    if downs.len() != weights.len()
        || weights.iter().any(|w| !(0..=1 << 24).contains(w))
        || downs.iter().any(|d| *d < 0)
    {
        return Err(shape("mixing weights or shifts out of range"));
    }
    #[cfg(target_arch = "x86_64")]
    if Backend::Avx2.available() {
        // SAFETY: `available()` confirmed AVX2 on this CPU.
        unsafe { auto_avx2::mix_rows(weights, downs, values, stride, offset, mix) };
        return Ok(());
    }
    mix_rows_body(weights, downs, values, stride, offset, mix);
    Ok(())
}

fn check_rows(
    width: usize,
    len: usize,
    stride: usize,
    offset: usize,
    rows: usize,
) -> Result<(), SimdError> {
    let Some(last) = rows.checked_sub(1) else {
        return Ok(());
    };
    let end = last
        .checked_mul(stride)
        .and_then(|start| start.checked_add(offset))
        .and_then(|start| start.checked_add(width));
    match end {
        Some(end) if end <= len => Ok(()),
        _ => Err(shape("rows reach past the end of the cache")),
    }
}

#[inline(always)]
fn dot_rows(q: &[i16], keys: &[i8], stride: usize, offset: usize, out: &mut [i32]) {
    for (u, slot) in out.iter_mut().enumerate() {
        let key = &keys[u * stride + offset..u * stride + offset + q.len()];
        *slot = q
            .iter()
            .zip(key)
            .map(|(x, y)| i32::from(*x) * i32::from(*y))
            .sum();
    }
}

#[inline(always)]
fn mix_rows_body(
    weights: &[i32],
    downs: &[i32],
    values: &[i8],
    stride: usize,
    offset: usize,
    mix: &mut [i64],
) {
    let width = mix.len();
    for (u, (&w, &down)) in weights.iter().zip(downs).enumerate() {
        if w == 0 {
            continue;
        }
        let v = &values[u * stride + offset..u * stride + offset + width];
        match down {
            0 => {
                for (m, x) in mix.iter_mut().zip(v) {
                    *m += i64::from(w * i32::from(*x));
                }
            }
            // For 1 <= d <= 31, (p + 2^(d - 1)) >> d == ((p >> (d - 1)) + 1) >> 1.
            1..=31 => {
                let s = (down - 1) as u32;
                for (m, x) in mix.iter_mut().zip(v) {
                    *m += i64::from(((w * i32::from(*x)) >> s).wrapping_add(1) >> 1);
                }
            }
            // (p + 2^(d - 1)) >> d is zero for |p| < 2^31.
            32..=62 => {}
            _ => {
                for (m, x) in mix.iter_mut().zip(v) {
                    if w * i32::from(*x) < 0 {
                        *m -= 1;
                    }
                }
            }
        }
    }
}

/// The attention loops compiled a second time with AVX2 enabled (the same safe
/// code; the compiler vectorizes it with 256-bit instructions).
#[cfg(target_arch = "x86_64")]
mod auto_avx2 {
    #[target_feature(enable = "avx2")]
    pub(super) fn dot_rows(q: &[i16], keys: &[i8], stride: usize, offset: usize, out: &mut [i32]) {
        super::dot_rows(q, keys, stride, offset, out);
    }

    #[target_feature(enable = "avx2")]
    pub(super) fn mix_rows(
        weights: &[i32],
        downs: &[i32],
        values: &[i8],
        stride: usize,
        offset: usize,
        mix: &mut [i64],
    ) {
        super::mix_rows_body(weights, downs, values, stride, offset, mix);
    }
}

#[cfg(target_arch = "x86_64")]
mod avx2 {
    use super::{Interleaved, GROUP, GROUP_PAIRS, PAIR_TABLE, ROWS};
    use std::arch::x86_64::*;

    #[inline]
    #[target_feature(enable = "avx2")]
    fn load128(bytes: &[u8]) -> __m128i {
        let bytes = &bytes[..16];
        // SAFETY: `bytes` is exactly 16 readable bytes; `loadu` needs no alignment.
        unsafe { _mm_loadu_si128(bytes.as_ptr().cast()) }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    fn load256(bytes: &[u8]) -> __m256i {
        let bytes = &bytes[..32];
        // SAFETY: `bytes` is exactly 32 readable bytes; `loadu` needs no alignment.
        unsafe { _mm256_loadu_si256(bytes.as_ptr().cast()) }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    fn store256(out: &mut [i64], value: __m256i) {
        let out = &mut out[..4];
        // SAFETY: `out` is exactly four writable `i64` (32 bytes); `storeu`
        // needs no alignment.
        unsafe { _mm256_storeu_si256(out.as_mut_ptr().cast(), value) }
    }

    /// Lanes whose `m` has `bit` set, as all-ones masks.
    #[inline]
    #[target_feature(enable = "avx2")]
    fn has_bit(m: __m256i, bit: i32) -> __m256i {
        let b = _mm256_set1_epi32(bit);
        _mm256_cmpeq_epi32(_mm256_and_si256(m, b), b)
    }

    /// `(16 + m) a` per lane by shifts, masks and additions.
    #[inline]
    #[target_feature(enable = "avx2")]
    fn scale_16_plus(a: __m256i, m: __m256i) -> __m256i {
        let mut t = _mm256_slli_epi32::<4>(a);
        t = _mm256_add_epi32(t, _mm256_and_si256(a, has_bit(m, 1)));
        t = _mm256_add_epi32(
            t,
            _mm256_and_si256(_mm256_slli_epi32::<1>(a), has_bit(m, 2)),
        );
        t = _mm256_add_epi32(
            t,
            _mm256_and_si256(_mm256_slli_epi32::<2>(a), has_bit(m, 4)),
        );
        _mm256_add_epi32(
            t,
            _mm256_and_si256(_mm256_slli_epi32::<3>(a), has_bit(m, 8)),
        )
    }

    /// One block: 32 weights (two columns of sixteen rows) per four `vpshufb`.
    #[target_feature(enable = "avx2")]
    pub(super) fn gemv_block(m: &Interleaved, tables: &[u8], b: usize, out: &mut [i64; ROWS]) {
        let (pairs, groups) = (m.cols / 2, m.cols / GROUP);
        let low4 = _mm256_set1_epi8(0x0F);
        // The low lane reads the even column (low nibbles), the high lane the
        // odd column (high nibbles).
        let lane_shift = _mm256_setr_epi64x(0, 0, 4, 4);
        let fifteen = _mm256_set1_epi32(15);
        let min_de = load128(&m.min_de[b * ROWS..]);
        let mut acc = [_mm256_setzero_si256(); 4];
        for g in 0..groups {
            let first = (b * pairs + g * GROUP_PAIRS) * ROWS;
            let nibbles = &m.nibbles[first..first + GROUP_PAIRS * ROWS];
            let group_tables =
                &tables[g * GROUP_PAIRS * PAIR_TABLE..(g + 1) * GROUP_PAIRS * PAIR_TABLE];
            let mut a = [_mm256_setzero_si256(); 2];
            for half in 0..2 {
                let mut sa = [_mm256_setzero_si256(); 2];
                let mut sb = [_mm256_setzero_si256(); 2];
                for j in half * 8..half * 8 + 8 {
                    let n = _mm256_broadcastsi128_si256(load128(&nibbles[j * ROWS..]));
                    let idx = _mm256_and_si256(_mm256_srlv_epi64(n, lane_shift), low4);
                    let t = &group_tables[j * PAIR_TABLE..(j + 1) * PAIR_TABLE];
                    let a_lo = _mm256_shuffle_epi8(load256(&t[0..]), idx);
                    let a_hi = _mm256_shuffle_epi8(load256(&t[32..]), idx);
                    let b_lo = _mm256_shuffle_epi8(load256(&t[64..]), idx);
                    let b_hi = _mm256_shuffle_epi8(load256(&t[96..]), idx);
                    sa[0] = _mm256_add_epi16(sa[0], _mm256_unpacklo_epi8(a_lo, a_hi));
                    sa[1] = _mm256_add_epi16(sa[1], _mm256_unpackhi_epi8(a_lo, a_hi));
                    sb[0] = _mm256_add_epi16(sb[0], _mm256_unpacklo_epi8(b_lo, b_hi));
                    sb[1] = _mm256_add_epi16(sb[1], _mm256_unpackhi_epi8(b_lo, b_hi));
                }
                for r in 0..2 {
                    // Fold the even-column and odd-column lanes (rows 8r..8r+8).
                    let fa = _mm_add_epi16(
                        _mm256_castsi256_si128(sa[r]),
                        _mm256_extracti128_si256::<1>(sa[r]),
                    );
                    let fb = _mm_add_epi16(
                        _mm256_castsi256_si128(sb[r]),
                        _mm256_extracti128_si256::<1>(sb[r]),
                    );
                    let wide = _mm256_add_epi32(
                        _mm256_slli_epi32::<8>(_mm256_cvtepi16_epi32(fa)),
                        _mm256_cvtepi16_epi32(fb),
                    );
                    a[r] = _mm256_add_epi32(a[r], wide);
                }
            }
            let scales = load128(&m.scales[(b * groups + g) * ROWS..]);
            for r in 0..2 {
                let (s8, d8) = if r == 0 {
                    (scales, min_de)
                } else {
                    (_mm_srli_si128::<8>(scales), _mm_srli_si128::<8>(min_de))
                };
                let s32 = _mm256_cvtepu8_epi32(s8);
                let shifts =
                    _mm256_sub_epi32(_mm256_srli_epi32::<4>(s32), _mm256_cvtepu8_epi32(d8));
                let scaled = scale_16_plus(a[r], _mm256_and_si256(s32, fifteen));
                let lo = _mm256_sllv_epi64(
                    _mm256_cvtepi32_epi64(_mm256_castsi256_si128(scaled)),
                    _mm256_cvtepi32_epi64(_mm256_castsi256_si128(shifts)),
                );
                let hi = _mm256_sllv_epi64(
                    _mm256_cvtepi32_epi64(_mm256_extracti128_si256::<1>(scaled)),
                    _mm256_cvtepi32_epi64(_mm256_extracti128_si256::<1>(shifts)),
                );
                acc[2 * r] = _mm256_add_epi64(acc[2 * r], lo);
                acc[2 * r + 1] = _mm256_add_epi64(acc[2 * r + 1], hi);
            }
        }
        for (i, value) in acc.into_iter().enumerate() {
            store256(&mut out[4 * i..], value);
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::{Interleaved, GROUP, GROUP_PAIRS, PAIR_TABLE, ROWS};
    use std::arch::aarch64::*;

    #[inline]
    #[target_feature(enable = "neon")]
    fn load(bytes: &[u8]) -> uint8x16_t {
        let bytes = &bytes[..16];
        // SAFETY: `bytes` is exactly 16 readable bytes; `vld1q_u8` needs byte alignment only.
        unsafe { vld1q_u8(bytes.as_ptr()) }
    }

    #[inline]
    #[target_feature(enable = "neon")]
    fn store(out: &mut [i64], value: int64x2_t) {
        let out = &mut out[..2];
        // SAFETY: `out` is exactly two writable `i64`; `vst1q_s64` needs `i64` alignment, which a slice of `i64` has.
        unsafe { vst1q_s64(out.as_mut_ptr(), value) }
    }

    /// Lanes whose `m` has `bit` set, as all-ones masks.
    #[inline]
    #[target_feature(enable = "neon")]
    fn has_bit(m: uint32x4_t, bit: u32) -> int32x4_t {
        vreinterpretq_s32_u32(vtstq_u32(m, vdupq_n_u32(bit)))
    }

    /// `(16 + m) a` per lane by shifts, masks and additions.
    #[inline]
    #[target_feature(enable = "neon")]
    fn scale_16_plus(a: int32x4_t, m: uint32x4_t) -> int32x4_t {
        let mut t = vshlq_n_s32::<4>(a);
        t = vaddq_s32(t, vandq_s32(a, has_bit(m, 1)));
        t = vaddq_s32(t, vandq_s32(vshlq_n_s32::<1>(a), has_bit(m, 2)));
        t = vaddq_s32(t, vandq_s32(vshlq_n_s32::<2>(a), has_bit(m, 4)));
        vaddq_s32(t, vandq_s32(vshlq_n_s32::<3>(a), has_bit(m, 8)))
    }

    /// One block: sixteen weights (one column of sixteen rows) per two `tbl`
    /// pairs.
    #[target_feature(enable = "neon")]
    pub(super) fn gemv_block(m: &Interleaved, tables: &[u8], b: usize, out: &mut [i64; ROWS]) {
        let (pairs, groups) = (m.cols / 2, m.cols / GROUP);
        let low4 = vdupq_n_u8(0x0F);
        let min_de = load(&m.min_de[b * ROWS..]);
        let mut acc = [vdupq_n_s64(0); 8];
        for g in 0..groups {
            let first = (b * pairs + g * GROUP_PAIRS) * ROWS;
            let nibbles = &m.nibbles[first..first + GROUP_PAIRS * ROWS];
            let group_tables =
                &tables[g * GROUP_PAIRS * PAIR_TABLE..(g + 1) * GROUP_PAIRS * PAIR_TABLE];
            let mut a = [vdupq_n_s32(0); 4];
            for half in 0..2 {
                let mut sa = [vdupq_n_s16(0); 2];
                let mut sb = [vdupq_n_s16(0); 2];
                for j in half * 8..half * 8 + 8 {
                    let n = load(&nibbles[j * ROWS..]);
                    let t = &group_tables[j * PAIR_TABLE..(j + 1) * PAIR_TABLE];
                    for (idx, off) in [(vandq_u8(n, low4), 0), (vshrq_n_u8::<4>(n), 16)] {
                        let a_lo = vqtbl1q_u8(load(&t[off..]), idx);
                        let a_hi = vqtbl1q_u8(load(&t[32 + off..]), idx);
                        let b_lo = vqtbl1q_u8(load(&t[64 + off..]), idx);
                        let b_hi = vqtbl1q_u8(load(&t[96 + off..]), idx);
                        sa[0] = vaddq_s16(sa[0], vreinterpretq_s16_u8(vzip1q_u8(a_lo, a_hi)));
                        sa[1] = vaddq_s16(sa[1], vreinterpretq_s16_u8(vzip2q_u8(a_lo, a_hi)));
                        sb[0] = vaddq_s16(sb[0], vreinterpretq_s16_u8(vzip1q_u8(b_lo, b_hi)));
                        sb[1] = vaddq_s16(sb[1], vreinterpretq_s16_u8(vzip2q_u8(b_lo, b_hi)));
                    }
                }
                for r in 0..2 {
                    // Rows 8r..8r+4 and 8r+4..8r+8.
                    a[2 * r] = vaddw_s16(
                        vaddq_s32(a[2 * r], vshll_n_s16::<8>(vget_low_s16(sa[r]))),
                        vget_low_s16(sb[r]),
                    );
                    a[2 * r + 1] = vaddw_s16(
                        vaddq_s32(a[2 * r + 1], vshll_n_s16::<8>(vget_high_s16(sa[r]))),
                        vget_high_s16(sb[r]),
                    );
                }
            }
            let scales = load(&m.scales[(b * groups + g) * ROWS..]);
            let m8 = vandq_u8(scales, low4);
            let shift8 = vsubq_u8(vshrq_n_u8::<4>(scales), min_de);
            let m16 = [vmovl_u8(vget_low_u8(m8)), vmovl_high_u8(m8)];
            let shift16 = [vmovl_u8(vget_low_u8(shift8)), vmovl_high_u8(shift8)];
            for k in 0..4 {
                let (m32, shift32) = if k % 2 == 0 {
                    (
                        vmovl_u16(vget_low_u16(m16[k / 2])),
                        vmovl_u16(vget_low_u16(shift16[k / 2])),
                    )
                } else {
                    (vmovl_high_u16(m16[k / 2]), vmovl_high_u16(shift16[k / 2]))
                };
                let scaled = scale_16_plus(a[k], m32);
                let shifts = vreinterpretq_s32_u32(shift32);
                let lo = vshlq_s64(
                    vmovl_s32(vget_low_s32(scaled)),
                    vmovl_s32(vget_low_s32(shifts)),
                );
                let hi = vshlq_s64(vmovl_high_s32(scaled), vmovl_high_s32(shifts));
                acc[2 * k] = vaddq_s64(acc[2 * k], lo);
                acc[2 * k + 1] = vaddq_s64(acc[2 * k + 1], hi);
            }
        }
        for (i, value) in acc.into_iter().enumerate() {
            store(&mut out[2 * i..], value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 33) as u32
        }
    }

    fn random_matrix(rows: usize, cols: usize, seed: u64) -> (Vec<u8>, Vec<u8>) {
        let mut rng = Lcg(seed);
        let nibbles = (0..rows * cols / 2).map(|_| rng.next() as u8).collect();
        let scales = (0..rows * cols / GROUP).map(|_| rng.next() as u8).collect();
        (nibbles, scales)
    }

    /// `sum_c q (16 + m) x 2^(de - de_min)` for every row, in `i128`.
    fn direct(rows: usize, cols: usize, nibbles: &[u8], scales: &[u8], x: &[i16]) -> Vec<i128> {
        let groups = cols / GROUP;
        (0..rows)
            .map(|r| {
                let row_scales = &scales[r * groups..(r + 1) * groups];
                let de_min = row_scales.iter().map(|s| s >> 4).min().unwrap_or(0);
                (0..cols)
                    .map(|c| {
                        let byte = nibbles[(r * cols + c) / 2];
                        let u = if c % 2 == 0 { byte & 15 } else { byte >> 4 };
                        let s = row_scales[c / GROUP];
                        let q = i128::from(u) - 8;
                        (q * (16 + i128::from(s & 15)) * i128::from(x[c])) << ((s >> 4) - de_min)
                    })
                    .sum()
            })
            .collect()
    }

    fn run(m: &Interleaved, backend: Backend, x: &[i16]) -> Vec<i64> {
        let mut tables = Tables::default();
        tables.build(x).unwrap();
        let mut out = vec![0i64; m.blocks() * ROWS];
        m.gemv_blocks_with(backend, &tables, 0, &mut out).unwrap();
        out
    }

    fn vector_backends() -> Vec<Backend> {
        [Backend::Avx2, Backend::Neon]
            .into_iter()
            .filter(|b| b.available())
            .collect()
    }

    #[test]
    fn tables_split_every_product_exactly() {
        let x: Vec<i16> = [
            -32768, -32767, -32641, -32640, -129, -128, -1, 0, 1, 127, 128, 255, 256, 32639, 32640,
            32767,
        ]
        .into_iter()
        .cycle()
        .take(GROUP)
        .collect();
        let mut tables = Tables::default();
        tables.build(&x).unwrap();
        for (c, v) in x.iter().enumerate() {
            let t = &tables.bytes[(c / 2) * PAIR_TABLE..(c / 2 + 1) * PAIR_TABLE];
            for u in 0..16 {
                let i = (c % 2) * 16 + u;
                let high = i16::from_le_bytes([t[i], t[32 + i]]);
                let low = i16::from_le_bytes([t[64 + i], t[96 + i]]);
                assert!(high.abs() <= 1024 && low.abs() <= 1024);
                let q = u as i32 - 8;
                assert_eq!((i32::from(high) << 8) + i32::from(low), q * i32::from(*v));
            }
        }
    }

    #[test]
    fn portable_kernel_matches_the_direct_sum() {
        for (rows, cols, seed) in [(1, 32, 1), (16, 64, 2), (37, 96, 3), (50, 160, 4)] {
            let (nibbles, scales) = random_matrix(rows, cols, seed);
            let m = Interleaved::new(rows, cols, &nibbles, &scales).unwrap();
            let mut rng = Lcg(seed + 100);
            let x: Vec<i16> = (0..cols).map(|_| rng.next() as i16).collect();
            let want = direct(rows, cols, &nibbles, &scales, &x);
            let got = run(&m, Backend::Portable, &x);
            for r in 0..rows {
                assert_eq!(i128::from(got[r]), want[r], "{rows}x{cols} row {r}");
            }
        }
    }

    #[test]
    fn vector_backends_match_the_portable_kernel_bit_for_bit() {
        let backends = vector_backends();
        eprintln!("vector backends on this CPU: {backends:?}");
        for (rows, cols, seed) in [(16, 32, 5), (40, 64, 6), (64, 576, 7), (33, 1536, 8)] {
            let (nibbles, scales) = random_matrix(rows, cols, seed);
            let m = Interleaved::new(rows, cols, &nibbles, &scales).unwrap();
            let mut rng = Lcg(seed + 200);
            let x: Vec<i16> = (0..cols).map(|_| rng.next() as i16).collect();
            let want = run(&m, Backend::Portable, &x);
            for &backend in &backends {
                assert_eq!(
                    run(&m, backend, &x),
                    want,
                    "{} {rows}x{cols}",
                    backend.name()
                );
            }
        }
    }

    #[test]
    fn extreme_weights_scales_and_activations_do_not_wrap() {
        let (rows, cols) = (16, 64);
        for byte in [0x00u8, 0xFF, 0x0F, 0xF0] {
            for x0 in [-32768i16, -32767, 32767, -32640] {
                let nibbles = vec![byte; rows * cols / 2];
                // Group 0 at the largest scale and exponent, group 1 at the smallest.
                let scales: Vec<u8> = (0..rows * cols / GROUP)
                    .map(|i| if i % 2 == 0 { 0xFF } else { 0x00 })
                    .collect();
                let x = vec![x0; cols];
                let m = Interleaved::new(rows, cols, &nibbles, &scales).unwrap();
                let want = direct(rows, cols, &nibbles, &scales, &x);
                for backend in std::iter::once(Backend::Portable).chain(vector_backends()) {
                    let got = run(&m, backend, &x);
                    for r in 0..rows {
                        assert_eq!(
                            i128::from(got[r]),
                            want[r],
                            "{} byte {byte:#x} x {x0}",
                            backend.name()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn padding_rows_and_partial_ranges_are_consistent() {
        let (rows, cols) = (21, 64);
        let (nibbles, scales) = random_matrix(rows, cols, 9);
        let m = Interleaved::new(rows, cols, &nibbles, &scales).unwrap();
        let x: Vec<i16> = (0..cols as i16).map(|i| i * 517 - 9000).collect();
        let whole = run(&m, Backend::detect(), &x);
        assert!(
            whole[rows..].iter().all(|v| *v == 0),
            "padding rows are zero"
        );
        let mut tables = Tables::default();
        tables.build(&x).unwrap();
        let mut second = vec![0i64; ROWS];
        m.gemv_blocks(&tables, 1, &mut second).unwrap();
        assert_eq!(second, whole[ROWS..2 * ROWS]);
    }

    /// `uor-r4-lut`'s rounding shift (round half up; saturating left).
    fn reference_shift(value: i64, shift: i32) -> i64 {
        if shift >= 63 {
            if value < 0 {
                -1
            } else {
                0
            }
        } else if shift > 0 {
            (value + (1i64 << (shift - 1))) >> shift
        } else {
            value
        }
    }

    #[test]
    fn attention_rows_match_the_i64_reference() {
        let (width, stride, offset, rows) = (64, 192, 64, 37);
        let mut rng = Lcg(77);
        let q: Vec<i16> = (0..width)
            .map(|i| {
                if i % 9 == 0 {
                    -32767
                } else {
                    rng.next() as i16
                }
            })
            .collect();
        let cache: Vec<i8> = (0..rows * stride)
            .map(|i| {
                if i % 11 == 0 {
                    -127
                } else {
                    (rng.next() as i8).max(-127)
                }
            })
            .collect();
        let mut dots = vec![0i32; rows];
        dot_rows_i16_i8(&q, &cache, stride, offset, &mut dots).unwrap();
        for (u, got) in dots.iter().enumerate() {
            let key = &cache[u * stride + offset..u * stride + offset + width];
            let want: i64 = q
                .iter()
                .zip(key)
                .map(|(a, b)| i64::from(*a) * i64::from(*b))
                .sum();
            assert_eq!(i64::from(*got), want, "row {u}");
        }
        let weights: Vec<i32> = (0..rows)
            .map(|u| match u % 5 {
                0 => 1 << 24,
                1 => 0,
                2 => (1 << 24) - 1,
                _ => (rng.next() >> 8) as i32,
            })
            .collect();
        let downs: Vec<i32> = (0..rows as i32)
            .map(|u| [0, 1, 2, 7, 30, 31, 32, 40, 63, 70][u as usize % 10])
            .collect();
        let mut mix = vec![3i64; width];
        mix_rows(&weights, &downs, &cache, stride, offset, &mut mix).unwrap();
        for (i, got) in mix.iter().enumerate() {
            let mut want = 3i64;
            for u in 0..rows {
                if weights[u] != 0 {
                    let v = i64::from(cache[u * stride + offset + i]);
                    want += reference_shift(i64::from(weights[u]) * v, downs[u]);
                }
            }
            assert_eq!(*got, want, "lane {i}");
        }
        assert!(dot_rows_i16_i8(
            &q,
            &cache[..cache.len() - 1],
            stride,
            offset + 64,
            &mut dots
        )
        .is_err());
        assert!(mix_rows(&[1 << 25], &[0], &cache, stride, offset, &mut mix).is_err());
    }

    #[test]
    fn mismatched_shapes_are_rejected() {
        let (nibbles, scales) = random_matrix(16, 64, 10);
        assert!(Interleaved::new(16, 60, &nibbles, &scales).is_err());
        assert!(Interleaved::new(16, 64, &nibbles[1..], &scales).is_err());
        let m = Interleaved::new(16, 64, &nibbles, &scales).unwrap();
        let mut tables = Tables::default();
        assert!(tables.build(&[0i16; 48]).is_err());
        tables.build(&[0i16; 32]).unwrap();
        let mut out = vec![0i64; ROWS];
        assert!(m.gemv_blocks(&tables, 0, &mut out).is_err());
        tables.build(&[0i16; 64]).unwrap();
        assert!(m.gemv_blocks(&tables, 1, &mut out).is_err());
        assert!(m.gemv_blocks(&tables, 0, &mut out[..8]).is_err());
        m.gemv_blocks(&tables, 0, &mut out).unwrap();
    }
}
