//! geo-bench (sys2 scratch): multiplier-free serving kernels for a geometric LM.
//!
//! Weight kernels: repo-style signed4 gather GEMV (copied from uor-r4-integer),
//! TL1 ternary LUT GEMV (bitnet.cpp TL1 idea), element-wise signed-4-bit LUT GEMV,
//! plus f32 GEMV and int8 multiply-accumulate GEMV comparators (NOT D0-b).
//! Read kernels over T keys: repo software-multiply Q8 dot (sampled), hardware
//! int16 dot (comparator), 64-bit sign-code popcount, and the 36-bit Lorentz
//! "radius + direction" ADC fast-scan with a multiplier-free radius combine,
//! top-k, arcosh/exp tables and value mixing with 4-bit product tables.
//! Elementwise: software shift-add product vs quarter-square table vs hardware.
//!
//! The numerical kernels declared D0-b use only add/sub/shift/and/table reads
//! (pshufb on x86, tbl on aarch64). Reference checks use ordinary arithmetic.

use std::hint::black_box;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
static SURV: AtomicU64 = AtomicU64::new(0);
static CALLS: AtomicU64 = AtomicU64::new(0);
static T_P1: AtomicU64 = AtomicU64::new(0);
static T_P2: AtomicU64 = AtomicU64::new(0);
static T_TAIL: AtomicU64 = AtomicU64::new(0);
static PROFILE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// ---------------------------------------------------------------- utilities
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn i8(&mut self) -> i8 {
        (self.next() >> 56) as i8
    }
    fn f64u(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn normal(&mut self) -> f64 {
        let u1 = self.f64u().max(1e-12);
        let u2 = self.f64u();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Minimum over 5 repetitions of the mean ns per call, each repetition >= min_ms.
fn time_ns<F: FnMut()>(mut f: F, min_ms: u128) -> f64 {
    f();
    let mut best = f64::MAX;
    for _ in 0..5 {
        let start = Instant::now();
        let mut iters = 0u64;
        loop {
            f();
            iters += 1;
            let el = start.elapsed().as_nanos();
            if el >= min_ms * 1_000_000 {
                best = best.min(el as f64 / iters as f64);
                break;
            }
        }
    }
    best
}

// ------------------------------------------------ repo signed4 gather (copy)
fn low_bit_products(input: &[i32]) -> Vec<[i64; 16]> {
    input
        .iter()
        .map(|&x| {
            let x = i64::from(x);
            let twice = x << 1;
            let four = x << 2;
            let three = x + twice;
            let five = x + four;
            let six = twice + four;
            let seven = (x << 3) - x;
            [
                0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three,
                -twice, -x,
            ]
        })
        .collect()
}
#[inline(never)]
fn low_bit_dot(products: &[[i64; 16]], weights: &[i16]) -> i64 {
    let mut total = 0i64;
    for (multiples, &weight) in products.iter().zip(weights) {
        total += multiples[usize::from((weight as u16) & 15)];
    }
    total
}
fn gemv_repo(x: &[i32], w: &[i16], m: usize, k: usize, out: &mut [i64]) {
    let p = low_bit_products(x);
    for r in 0..m {
        out[r] = low_bit_dot(&p, &w[r * k..(r + 1) * k]);
    }
}

// ------------------------------------------ software multiply (repo math.rs)
fn checked_mul_unsigned(mut left: u128, mut right: u128) -> Option<u128> {
    if left < right {
        std::mem::swap(&mut left, &mut right);
    }
    let mut result = 0u128;
    while right != 0 {
        if right & 1 != 0 {
            result = result.checked_add(left)?;
        }
        right >>= 1;
        if right != 0 {
            if left > (u128::MAX >> 1) {
                return None;
            }
            left <<= 1;
        }
    }
    Some(result)
}
fn soft_mul(a: i128, b: i128) -> i128 {
    let m = checked_mul_unsigned(a.unsigned_abs(), b.unsigned_abs()).unwrap_or(0) as i128;
    if (a < 0) != (b < 0) {
        -m
    } else {
        m
    }
}

// --------------------------------------------------- nibble-LUT GEMV layouts
/// Row blocks of 32 rows; per step a 16-byte vector whose low nibble indexes
/// rows 0..15 and high nibble rows 16..31 (T-MAC style interleave).
struct NibbleMatrix {
    m: usize,
    steps: usize,
    packed: Vec<u8>,
}
/// Per-step 16-entry i16 tables split in low/high byte planes.
struct Tables {
    lo: Vec<[u8; 16]>,
    hi: Vec<[u8; 16]>,
}

fn pack_nibbles(m: usize, steps: usize, index: impl Fn(usize, usize) -> u8) -> NibbleMatrix {
    assert!(m % 32 == 0);
    let mut packed = vec![0u8; (m / 32) * steps * 16];
    for rb in 0..m / 32 {
        for j in 0..steps {
            for b in 0..16 {
                let lo = index(rb * 32 + b, j);
                let hi = index(rb * 32 + 16 + b, j);
                debug_assert!(lo < 16 && hi < 16);
                packed[(rb * steps + j) * 16 + b] = lo | (hi << 4);
            }
        }
    }
    NibbleMatrix { m, steps, packed }
}

/// TL1: two ternary weights -> index (w0+1)*3+(w1+1) in 0..9.
fn tl1_pack(w: &[i8], m: usize, k: usize) -> NibbleMatrix {
    pack_nibbles(m, k / 2, |r, j| {
        let w0 = w[r * k + 2 * j] as i32;
        let w1 = w[r * k + 2 * j + 1] as i32;
        ((w0 + 1) * 3 + (w1 + 1)) as u8
    })
}
/// TL1 table for each activation pair: all 9 signed sums, adds only.
fn tl1_tables(x: &[i8]) -> Tables {
    let pairs = x.len() / 2;
    let mut t = Tables {
        lo: vec![[0; 16]; pairs],
        hi: vec![[0; 16]; pairs],
    };
    for j in 0..pairs {
        let x0 = x[2 * j] as i16;
        let x1 = x[2 * j + 1] as i16;
        let vals: [i16; 9] = [-x0 - x1, -x0, x1 - x0, -x1, 0, x1, x0 - x1, x0, x0 + x1];
        for (i, v) in vals.iter().enumerate() {
            let u = *v as u16;
            t.lo[j][i] = u as u8;
            t.hi[j][i] = (u >> 8) as u8;
        }
    }
    t
}
/// Element-wise signed 4-bit: nibble = code & 15 (codes -7..7).
fn elut4_pack(w: &[i8], m: usize, k: usize) -> NibbleMatrix {
    pack_nibbles(m, k, |r, j| (w[r * k + j] as u8) & 15)
}
/// Per input coordinate the 16 multiples x*c, built with shifts/adds only.
fn elut4_tables(x: &[i8]) -> Tables {
    let mut t = Tables {
        lo: vec![[0; 16]; x.len()],
        hi: vec![[0; 16]; x.len()],
    };
    for (j, &xv) in x.iter().enumerate() {
        let x1 = xv as i16;
        let x2 = x1 << 1;
        let x4 = x1 << 2;
        let m = [0, x1, x2, x1 + x2, x4, x4 + x1, x4 + x2, (x1 << 3) - x1];
        let mut e = [0i16; 16];
        for c in 0..8 {
            e[c] = m[c];
        }
        for c in 1..8 {
            e[16 - c] = -m[c];
        }
        for i in 0..16 {
            let u = e[i] as u16;
            t.lo[j][i] = u as u8;
            t.hi[j][i] = (u >> 8) as u8;
        }
    }
    t
}

fn nibble_gemv_scalar(a: &NibbleMatrix, t: &Tables, out: &mut [i32]) {
    for rb in 0..a.m / 32 {
        for b in 0..32 {
            let mut s = 0i32;
            for j in 0..a.steps {
                let byte = a.packed[(rb * a.steps + j) * 16 + (b & 15)];
                let idx = if b < 16 { byte & 15 } else { byte >> 4 } as usize;
                s += (t.lo[j][idx] as u16 | ((t.hi[j][idx] as u16) << 8)) as i16 as i32;
            }
            out[rb * 32 + b] = s;
        }
    }
}

/// `block` steps are accumulated in i16 before widening (caller guarantees no overflow).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3,sse4.1")]
unsafe fn nibble_gemv_x86(a: &NibbleMatrix, t: &Tables, block: usize, out: &mut [i32]) {
    use std::arch::x86_64::*;
    let mask = _mm_set1_epi8(0x0F);
    for rb in 0..a.m / 32 {
        let mut acc32 = [_mm_setzero_si128(); 8];
        let mut j = 0;
        while j < a.steps {
            let end = (j + block).min(a.steps);
            let mut acc16 = [_mm_setzero_si128(); 4];
            for jj in j..end {
                let v = _mm_loadu_si128(a.packed.as_ptr().add((rb * a.steps + jj) * 16) as *const __m128i);
                let ilo = _mm_and_si128(v, mask);
                let ihi = _mm_and_si128(_mm_srli_epi16(v, 4), mask);
                let tlo = _mm_loadu_si128(t.lo[jj].as_ptr() as *const __m128i);
                let thi = _mm_loadu_si128(t.hi[jj].as_ptr() as *const __m128i);
                let a_lo = _mm_shuffle_epi8(tlo, ilo);
                let a_hi = _mm_shuffle_epi8(thi, ilo);
                let b_lo = _mm_shuffle_epi8(tlo, ihi);
                let b_hi = _mm_shuffle_epi8(thi, ihi);
                acc16[0] = _mm_add_epi16(acc16[0], _mm_unpacklo_epi8(a_lo, a_hi));
                acc16[1] = _mm_add_epi16(acc16[1], _mm_unpackhi_epi8(a_lo, a_hi));
                acc16[2] = _mm_add_epi16(acc16[2], _mm_unpacklo_epi8(b_lo, b_hi));
                acc16[3] = _mm_add_epi16(acc16[3], _mm_unpackhi_epi8(b_lo, b_hi));
            }
            for q in 0..4 {
                acc32[2 * q] = _mm_add_epi32(acc32[2 * q], _mm_cvtepi16_epi32(acc16[q]));
                acc32[2 * q + 1] =
                    _mm_add_epi32(acc32[2 * q + 1], _mm_cvtepi16_epi32(_mm_srli_si128(acc16[q], 8)));
            }
            j = end;
        }
        for q in 0..8 {
            _mm_storeu_si128(out.as_mut_ptr().add(rb * 32 + q * 4) as *mut __m128i, acc32[q]);
        }
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn nibble_gemv_neon(a: &NibbleMatrix, t: &Tables, block: usize, out: &mut [i32]) {
    use std::arch::aarch64::*;
    let mask = vdupq_n_u8(0x0F);
    for rb in 0..a.m / 32 {
        let mut acc32 = [vdupq_n_s32(0); 8];
        let mut j = 0;
        while j < a.steps {
            let end = (j + block).min(a.steps);
            let mut acc16 = [vdupq_n_s16(0); 4];
            for jj in j..end {
                let v = vld1q_u8(a.packed.as_ptr().add((rb * a.steps + jj) * 16));
                let ilo = vandq_u8(v, mask);
                let ihi = vshrq_n_u8::<4>(v);
                let tlo = vld1q_u8(t.lo[jj].as_ptr());
                let thi = vld1q_u8(t.hi[jj].as_ptr());
                let a_lo = vqtbl1q_u8(tlo, ilo);
                let a_hi = vqtbl1q_u8(thi, ilo);
                let b_lo = vqtbl1q_u8(tlo, ihi);
                let b_hi = vqtbl1q_u8(thi, ihi);
                acc16[0] = vaddq_s16(acc16[0], vreinterpretq_s16_u8(vzip1q_u8(a_lo, a_hi)));
                acc16[1] = vaddq_s16(acc16[1], vreinterpretq_s16_u8(vzip2q_u8(a_lo, a_hi)));
                acc16[2] = vaddq_s16(acc16[2], vreinterpretq_s16_u8(vzip1q_u8(b_lo, b_hi)));
                acc16[3] = vaddq_s16(acc16[3], vreinterpretq_s16_u8(vzip2q_u8(b_lo, b_hi)));
            }
            for q in 0..4 {
                acc32[2 * q] = vaddw_s16(acc32[2 * q], vget_low_s16(acc16[q]));
                acc32[2 * q + 1] = vaddw_high_s16(acc32[2 * q + 1], acc16[q]);
            }
            j = end;
        }
        for q in 0..8 {
            vst1q_s32(out.as_mut_ptr().add(rb * 32 + q * 4), acc32[q]);
        }
    }
}

fn nibble_gemv(a: &NibbleMatrix, t: &Tables, block: usize, out: &mut [i32]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("ssse3") && is_x86_feature_detected!("sse4.1") {
            unsafe { nibble_gemv_x86(a, t, block, out) };
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { nibble_gemv_neon(a, t, block, out) };
        return;
    }
    #[allow(unreachable_code)]
    nibble_gemv_scalar(a, t, out)
}

// ------------------------------------------------------- dense comparators
fn gemv_f32(x: &[f32], w: &[f32], m: usize, k: usize, out: &mut [f32]) {
    for r in 0..m {
        let row = &w[r * k..(r + 1) * k];
        let mut acc = [0f32; 16];
        for (cw, cx) in row.chunks_exact(16).zip(x.chunks_exact(16)) {
            for i in 0..16 {
                acc[i] += cw[i] * cx[i];
            }
        }
        out[r] = acc.iter().sum();
    }
}
fn gemv_i8_mad(x: &[i8], w: &[i8], m: usize, k: usize, out: &mut [i32]) {
    for r in 0..m {
        let row = &w[r * k..(r + 1) * k];
        let mut acc = [0i32; 16];
        for (cw, cx) in row.chunks_exact(16).zip(x.chunks_exact(16)) {
            for i in 0..16 {
                acc[i] += cw[i] as i32 * cx[i] as i32;
            }
        }
        out[r] = acc.iter().sum();
    }
}

// ------------------------------------------------ Lorentz 36-bit ADC read
/// Radius levels in Q4 (x16), each with at most two set bits, so r*s is a
/// shift-add: 2,3,4,6,8,12,16,24,32,48,64,96,128,192,256,384 (0.125 .. 24).
const R16: [i32; 16] = [2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384];
fn r16_shifts(code: usize) -> (u32, Option<u32>) {
    let v = R16[code] as u32;
    let hi = 31 - v.leading_zeros();
    let rest = v - (1 << hi);
    (hi, if rest == 0 { None } else { Some(31 - rest.leading_zeros()) })
}

/// Keys bucketed by radius code (written once per token); each bucket holds
/// 32-key fast-scan blocks: 8 coordinate nibble vectors of 16 bytes (36 bits/key
/// counting the radius, which the bucket stores implicitly; ids kept separately).
struct KeyStore {
    buckets: Vec<Vec<u8>>, // per radius code: blocks * 8 * 16 bytes
    ids: Vec<Vec<u32>>,    // per radius code: key positions (padded with u32::MAX)
    count: [usize; 16],
    dir_codes: Vec<[u8; 8]>,
    r_code: Vec<u8>,
}

fn quantize_key(v: &[f64; 8]) -> ([u8; 8], u8) {
    let r = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mut d = [0u8; 8];
    for i in 0..8 {
        let u = if r > 0.0 { v[i] / r } else { 0.0 };
        // level_c = (2c-15)/16  ->  c = round((16u+15)/2)
        d[i] = (((16.0 * u + 15.0) / 2.0).round()).clamp(0.0, 15.0) as u8;
    }
    let mut best = 0;
    for c in 0..16 {
        let rc = R16[c] as f64 / 16.0;
        if (rc.ln() - r.max(1e-9).ln()).abs() < ((R16[best] as f64 / 16.0).ln() - r.max(1e-9).ln()).abs() {
            best = c;
        }
    }
    (d, best as u8)
}
fn dequant_key(d: &[u8; 8], rc: u8) -> [f64; 8] {
    let r = R16[rc as usize] as f64 / 16.0;
    let mut out = [0.0; 8];
    for i in 0..8 {
        out[i] = r * (2.0 * d[i] as f64 - 15.0) / 16.0;
    }
    out
}

fn build_store(keys: &[[f64; 8]]) -> KeyStore {
    let mut st = KeyStore {
        buckets: vec![Vec::new(); 16],
        ids: vec![Vec::new(); 16],
        count: [0; 16],
        dir_codes: Vec::with_capacity(keys.len()),
        r_code: Vec::with_capacity(keys.len()),
    };
    let mut members: Vec<Vec<u32>> = vec![Vec::new(); 16];
    for (i, k) in keys.iter().enumerate() {
        let (d, rc) = quantize_key(k);
        st.dir_codes.push(d);
        st.r_code.push(rc);
        members[rc as usize].push(i as u32);
    }
    for rc in 0..16 {
        let n = members[rc].len();
        st.count[rc] = n;
        let blocks = (n + 31) / 32;
        let mut bytes = vec![0u8; blocks * 8 * 16];
        let mut ids = vec![u32::MAX; blocks * 32];
        for (slot, &id) in members[rc].iter().enumerate() {
            ids[slot] = id;
            let blk = slot / 32;
            let lane = slot % 32;
            for c in 0..8 {
                let byte = &mut bytes[(blk * 8 + c) * 16 + (lane & 15)];
                let code = st.dir_codes[id as usize][c];
                if lane < 16 {
                    *byte |= code;
                } else {
                    *byte |= code << 4;
                }
            }
        }
        st.buckets[rc] = bytes;
        st.ids[rc] = ids;
    }
    st
}

/// Offline 4-bit product tables (sealed like the repo's exp/tanh tables):
/// QL[q8+128][c] = round(q8*(2c-15)/16) as i8 (query x direction level), and
/// PV[p][c] = p*code(c) as i16 split into byte planes (7-bit mass x signed 4-bit value).
struct ProductTables {
    ql: Vec<[i8; 16]>,
    ql16_lo: Vec<[u8; 16]>,
    ql16_hi: Vec<[u8; 16]>,
    pv_lo: Vec<[u8; 16]>,
    pv_hi: Vec<[u8; 16]>,
    sq: Vec<u32>, // quarter squares floor(n^2/4), n <= 2^15
}
fn build_product_tables() -> ProductTables {
    let mut ql = vec![[0i8; 16]; 256];
    for qv in -128i32..128 {
        let mut acc = -(qv << 4) + qv; // -15q by shift/add
        for c in 0..16 {
            let r = if acc >= 0 { (acc + 8) >> 4 } else { -((-acc + 8) >> 4) };
            ql[(qv + 128) as usize][c] = r.clamp(-127, 127) as i8;
            acc += qv << 1;
        }
    }
    let mut ql16_lo = vec![[0u8; 16]; 256];
    let mut ql16_hi = vec![[0u8; 16]; 256];
    for qv in -128i32..128 {
        let mut acc = -(qv << 4) + qv; // -15q, then +2q per level: q*(2c-15) exactly
        for c in 0..16 {
            let u = acc as i16 as u16;
            ql16_lo[(qv + 128) as usize][c] = u as u8;
            ql16_hi[(qv + 128) as usize][c] = (u >> 8) as u8;
            acc += qv << 1;
        }
    }
    let mut pv_lo = vec![[0u8; 16]; 128];
    let mut pv_hi = vec![[0u8; 16]; 128];
    for p in 0..128i32 {
        let mut acc = 0i32;
        let mut e = [0i16; 16];
        for c in 1..8 {
            acc += p;
            e[c] = acc as i16;
            e[16 - c] = -acc as i16;
        }
        for c in 0..16 {
            let u = e[c] as u16;
            pv_lo[p as usize][c] = u as u8;
            pv_hi[p as usize][c] = (u >> 8) as u8;
        }
    }
    let sq = (0..=(1u64 << 15)).map(|n| (n * n / 4) as u32).collect();
    ProductTables { ql, ql16_lo, ql16_hi, pv_lo, pv_hi, sq }
}
/// Exact product of |a|,|b| < 2^14 by quarter squares: two table reads, adds.
fn qs_mul(t: &ProductTables, a: i32, b: i32) -> i64 {
    let s = (a + b).unsigned_abs() as usize;
    let d = (a - b).unsigned_abs() as usize;
    t.sq[s] as i64 - t.sq[d] as i64
}
/// Floor square root by the restoring method (shift/subtract only).
fn isqrt(mut v: u64) -> u64 {
    let mut root = 0u64;
    let mut bit = 1u64 << 62;
    while bit > v {
        bit >>= 2;
    }
    while bit != 0 {
        if v >= root + bit {
            v -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

/// Direction sums s for every key of one bucket (i16 per key, in bucket order).
fn adc_bucket_scalar(bytes: &[u8], lut: &[[i8; 16]; 8], s_out: &mut [i16]) {
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        for lane in 0..32 {
            let mut s = 0i16;
            for c in 0..8 {
                let byte = bytes[(blk * 8 + c) * 16 + (lane & 15)];
                let code = if lane < 16 { byte & 15 } else { byte >> 4 };
                s += lut[c][code as usize] as i16;
            }
            s_out[blk * 32 + lane] = s;
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3,sse4.1")]
unsafe fn adc_bucket_x86(bytes: &[u8], lut: &[[i8; 16]; 8], s_out: &mut [i16]) {
    use std::arch::x86_64::*;
    let mask = _mm_set1_epi8(0x0F);
    let mut tl = [_mm_setzero_si128(); 8];
    for c in 0..8 {
        tl[c] = _mm_loadu_si128(lut[c].as_ptr() as *const __m128i);
    }
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        let mut acc = [_mm_setzero_si128(); 4];
        for c in 0..8 {
            let v = _mm_loadu_si128(bytes.as_ptr().add((blk * 8 + c) * 16) as *const __m128i);
            let lo = _mm_shuffle_epi8(tl[c], _mm_and_si128(v, mask));
            let hi = _mm_shuffle_epi8(tl[c], _mm_and_si128(_mm_srli_epi16(v, 4), mask));
            acc[0] = _mm_add_epi16(acc[0], _mm_cvtepi8_epi16(lo));
            acc[1] = _mm_add_epi16(acc[1], _mm_cvtepi8_epi16(_mm_srli_si128(lo, 8)));
            acc[2] = _mm_add_epi16(acc[2], _mm_cvtepi8_epi16(hi));
            acc[3] = _mm_add_epi16(acc[3], _mm_cvtepi8_epi16(_mm_srli_si128(hi, 8)));
        }
        for q in 0..4 {
            _mm_storeu_si128(s_out.as_mut_ptr().add(blk * 32 + q * 8) as *mut __m128i, acc[q]);
        }
    }
}
#[cfg(target_arch = "aarch64")]
unsafe fn adc_bucket_neon(bytes: &[u8], lut: &[[i8; 16]; 8], s_out: &mut [i16]) {
    use std::arch::aarch64::*;
    let mask = vdupq_n_u8(0x0F);
    let mut tl = [vdupq_n_s8(0); 8];
    for c in 0..8 {
        tl[c] = vld1q_s8(lut[c].as_ptr());
    }
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        let mut acc = [vdupq_n_s16(0); 4];
        for c in 0..8 {
            let v = vld1q_u8(bytes.as_ptr().add((blk * 8 + c) * 16));
            let lo = vqtbl1q_s8(tl[c], vandq_u8(v, mask));
            let hi = vqtbl1q_s8(tl[c], vshrq_n_u8::<4>(v));
            acc[0] = vaddw_s8(acc[0], vget_low_s8(lo));
            acc[1] = vaddw_high_s8(acc[1], lo);
            acc[2] = vaddw_s8(acc[2], vget_low_s8(hi));
            acc[3] = vaddw_high_s8(acc[3], hi);
        }
        for q in 0..4 {
            vst1q_s16(s_out.as_mut_ptr().add(blk * 32 + q * 8), acc[q]);
        }
    }
}
fn adc_bucket(bytes: &[u8], lut: &[[i8; 16]; 8], s_out: &mut [i16]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("ssse3") && is_x86_feature_detected!("sse4.1") {
            unsafe { adc_bucket_x86(bytes, lut, s_out) };
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { adc_bucket_neon(bytes, lut, s_out) };
        return;
    }
    #[allow(unreachable_code)]
    adc_bucket_scalar(bytes, lut, s_out)
}

/// Exact variant: 16-bit table entries q8*(2c-15) split into byte planes (2 lookups/coord).
fn adc16_bucket_scalar(bytes: &[u8], lo: &[[u8; 16]; 8], hi: &[[u8; 16]; 8], s_out: &mut [i16]) {
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        for lane in 0..32 {
            let mut s = 0i16;
            for c in 0..8 {
                let byte = bytes[(blk * 8 + c) * 16 + (lane & 15)];
                let code = (if lane < 16 { byte & 15 } else { byte >> 4 }) as usize;
                s = s.wrapping_add((lo[c][code] as u16 | ((hi[c][code] as u16) << 8)) as i16);
            }
            s_out[blk * 32 + lane] = s;
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3,sse4.1")]
unsafe fn adc16_bucket_x86(bytes: &[u8], lo: &[[u8; 16]; 8], hi: &[[u8; 16]; 8], s_out: &mut [i16]) {
    use std::arch::x86_64::*;
    let mask = _mm_set1_epi8(0x0F);
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        let mut acc = [_mm_setzero_si128(); 4];
        for c in 0..8 {
            let tlo = _mm_loadu_si128(lo[c].as_ptr() as *const __m128i);
            let thi = _mm_loadu_si128(hi[c].as_ptr() as *const __m128i);
            let v = _mm_loadu_si128(bytes.as_ptr().add((blk * 8 + c) * 16) as *const __m128i);
            let il = _mm_and_si128(v, mask);
            let ih = _mm_and_si128(_mm_srli_epi16(v, 4), mask);
            let a_lo = _mm_shuffle_epi8(tlo, il);
            let a_hi = _mm_shuffle_epi8(thi, il);
            let b_lo = _mm_shuffle_epi8(tlo, ih);
            let b_hi = _mm_shuffle_epi8(thi, ih);
            acc[0] = _mm_add_epi16(acc[0], _mm_unpacklo_epi8(a_lo, a_hi));
            acc[1] = _mm_add_epi16(acc[1], _mm_unpackhi_epi8(a_lo, a_hi));
            acc[2] = _mm_add_epi16(acc[2], _mm_unpacklo_epi8(b_lo, b_hi));
            acc[3] = _mm_add_epi16(acc[3], _mm_unpackhi_epi8(b_lo, b_hi));
        }
        for q in 0..4 {
            _mm_storeu_si128(s_out.as_mut_ptr().add(blk * 32 + q * 8) as *mut __m128i, acc[q]);
        }
    }
}
#[cfg(target_arch = "aarch64")]
unsafe fn adc16_bucket_neon(bytes: &[u8], lo: &[[u8; 16]; 8], hi: &[[u8; 16]; 8], s_out: &mut [i16]) {
    use std::arch::aarch64::*;
    let mask = vdupq_n_u8(0x0F);
    let blocks = bytes.len() / 128;
    for blk in 0..blocks {
        let mut acc = [vdupq_n_s16(0); 4];
        for c in 0..8 {
            let tlo = vld1q_u8(lo[c].as_ptr());
            let thi = vld1q_u8(hi[c].as_ptr());
            let v = vld1q_u8(bytes.as_ptr().add((blk * 8 + c) * 16));
            let il = vandq_u8(v, mask);
            let ih = vshrq_n_u8::<4>(v);
            let a_lo = vqtbl1q_u8(tlo, il);
            let a_hi = vqtbl1q_u8(thi, il);
            let b_lo = vqtbl1q_u8(tlo, ih);
            let b_hi = vqtbl1q_u8(thi, ih);
            acc[0] = vaddq_s16(acc[0], vreinterpretq_s16_u8(vzip1q_u8(a_lo, a_hi)));
            acc[1] = vaddq_s16(acc[1], vreinterpretq_s16_u8(vzip2q_u8(a_lo, a_hi)));
            acc[2] = vaddq_s16(acc[2], vreinterpretq_s16_u8(vzip1q_u8(b_lo, b_hi)));
            acc[3] = vaddq_s16(acc[3], vreinterpretq_s16_u8(vzip2q_u8(b_lo, b_hi)));
        }
        for q in 0..4 {
            vst1q_s16(s_out.as_mut_ptr().add(blk * 32 + q * 8), acc[q]);
        }
    }
}
fn adc16_bucket(bytes: &[u8], lo: &[[u8; 16]; 8], hi: &[[u8; 16]; 8], s_out: &mut [i16]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("ssse3") && is_x86_feature_detected!("sse4.1") {
            unsafe { adc16_bucket_x86(bytes, lo, hi, s_out) };
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { adc16_bucket_neon(bytes, lo, hi, s_out) };
        return;
    }
    #[allow(unreachable_code)]
    adc16_bucket_scalar(bytes, lo, hi, s_out)
}

/// Lanes (0..n) whose s exceeds s_thr, appended to `hits`.
fn filter_gt(s: &[i16], s_thr: i16, hits: &mut Vec<usize>) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use std::arch::x86_64::*;
        let thr = _mm_set1_epi16(s_thr);
        let mut i = 0;
        while i + 8 <= s.len() {
            let v = _mm_loadu_si128(s.as_ptr().add(i) as *const __m128i);
            let mut m = _mm_movemask_epi8(_mm_cmpgt_epi16(v, thr)) as u32;
            while m != 0 {
                let bit = m.trailing_zeros();
                hits.push(i + (bit as usize >> 1));
                m &= !(3u32 << bit);
            }
            i += 8;
        }
        return;
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        use std::arch::aarch64::*;
        let thr = vdupq_n_s16(s_thr);
        let mut i = 0;
        while i + 8 <= s.len() {
            let v = vld1q_s16(s.as_ptr().add(i));
            let gt = vcgtq_s16(v, thr);
            if vmaxvq_u16(gt) != 0 {
                for l in 0..8 {
                    if s[i + l] > s_thr {
                        hits.push(i + l);
                    }
                }
            }
            i += 8;
        }
        return;
    }
    #[allow(unreachable_code)]
    for (i, &v) in s.iter().enumerate() {
        if v > s_thr {
            hits.push(i);
        }
    }
}

/// Mix values of the selected candidates: out[d] = sum_j p_j * v_j[d] with 7-bit
/// masses and signed 4-bit value codes, via the offline PV byte-plane tables.
fn mix_values_scalar(t: &ProductTables, sel: &[(u32, i32)], values: &[[u8; 32]], out: &mut [i32; 64]) {
    for &(id, p) in sel {
        let v = &values[id as usize];
        for dd in 0..32 {
            let b = v[dd];
            for (half, code) in [(0usize, b & 15), (1, b >> 4)] {
                let e = (t.pv_lo[p as usize][code as usize] as u16 | ((t.pv_hi[p as usize][code as usize] as u16) << 8)) as i16;
                out[2 * dd + half] += e as i32;
            }
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3,sse4.1")]
unsafe fn mix_values_x86(t: &ProductTables, sel: &[(u32, i32)], values: &[[u8; 32]], out: &mut [i32; 64]) {
    use std::arch::x86_64::*;
    let mask = _mm_set1_epi8(0x0F);
    // acc[h][q]: value half h (bytes 0..15 / 16..31), q: 0,1 even dims (lo nibble), 2,3 odd dims
    let mut acc = [[_mm_setzero_si128(); 4]; 2];
    for &(id, p) in sel {
        let tlo = _mm_loadu_si128(t.pv_lo[p as usize].as_ptr() as *const __m128i);
        let thi = _mm_loadu_si128(t.pv_hi[p as usize].as_ptr() as *const __m128i);
        for h in 0..2 {
            let v = _mm_loadu_si128(values[id as usize].as_ptr().add(16 * h) as *const __m128i);
            let ilo = _mm_and_si128(v, mask);
            let ihi = _mm_and_si128(_mm_srli_epi16(v, 4), mask);
            let a_lo = _mm_shuffle_epi8(tlo, ilo);
            let a_hi = _mm_shuffle_epi8(thi, ilo);
            let b_lo = _mm_shuffle_epi8(tlo, ihi);
            let b_hi = _mm_shuffle_epi8(thi, ihi);
            acc[h][0] = _mm_add_epi16(acc[h][0], _mm_unpacklo_epi8(a_lo, a_hi));
            acc[h][1] = _mm_add_epi16(acc[h][1], _mm_unpackhi_epi8(a_lo, a_hi));
            acc[h][2] = _mm_add_epi16(acc[h][2], _mm_unpacklo_epi8(b_lo, b_hi));
            acc[h][3] = _mm_add_epi16(acc[h][3], _mm_unpackhi_epi8(b_lo, b_hi));
        }
    }
    let mut tmp = [[0i16; 8]; 4];
    for h in 0..2 {
        for q in 0..4 {
            _mm_storeu_si128(tmp[q].as_mut_ptr() as *mut __m128i, acc[h][q]);
        }
        for byte in 0..16 {
            let lo = tmp[byte / 8][byte % 8];
            let hi = tmp[2 + byte / 8][byte % 8];
            out[2 * (16 * h + byte)] += lo as i32;
            out[2 * (16 * h + byte) + 1] += hi as i32;
        }
    }
}
#[cfg(target_arch = "aarch64")]
unsafe fn mix_values_neon(t: &ProductTables, sel: &[(u32, i32)], values: &[[u8; 32]], out: &mut [i32; 64]) {
    use std::arch::aarch64::*;
    let mask = vdupq_n_u8(0x0F);
    let mut acc = [[vdupq_n_s16(0); 4]; 2];
    for &(id, p) in sel {
        let tlo = vld1q_u8(t.pv_lo[p as usize].as_ptr());
        let thi = vld1q_u8(t.pv_hi[p as usize].as_ptr());
        for h in 0..2 {
            let v = vld1q_u8(values[id as usize].as_ptr().add(16 * h));
            let ilo = vandq_u8(v, mask);
            let ihi = vshrq_n_u8::<4>(v);
            let a_lo = vqtbl1q_u8(tlo, ilo);
            let a_hi = vqtbl1q_u8(thi, ilo);
            let b_lo = vqtbl1q_u8(tlo, ihi);
            let b_hi = vqtbl1q_u8(thi, ihi);
            acc[h][0] = vaddq_s16(acc[h][0], vreinterpretq_s16_u8(vzip1q_u8(a_lo, a_hi)));
            acc[h][1] = vaddq_s16(acc[h][1], vreinterpretq_s16_u8(vzip2q_u8(a_lo, a_hi)));
            acc[h][2] = vaddq_s16(acc[h][2], vreinterpretq_s16_u8(vzip1q_u8(b_lo, b_hi)));
            acc[h][3] = vaddq_s16(acc[h][3], vreinterpretq_s16_u8(vzip2q_u8(b_lo, b_hi)));
        }
    }
    let mut tmp = [[0i16; 8]; 4];
    for h in 0..2 {
        for q in 0..4 {
            vst1q_s16(tmp[q].as_mut_ptr(), acc[h][q]);
        }
        for byte in 0..16 {
            let lo = tmp[byte / 8][byte % 8];
            let hi = tmp[2 + byte / 8][byte % 8];
            out[2 * (16 * h + byte)] += lo as i32;
            out[2 * (16 * h + byte) + 1] += hi as i32;
        }
    }
}
fn mix_values(t: &ProductTables, sel: &[(u32, i32)], values: &[[u8; 32]], out: &mut [i32; 64]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("ssse3") && is_x86_feature_detected!("sse4.1") {
            unsafe { mix_values_x86(t, sel, values, out) };
            return;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { mix_values_neon(t, sel, values, out) };
        return;
    }
    #[allow(unreachable_code)]
    mix_values_scalar(t, sel, values, out)
}

/// Offline tables (built once with f64, sealed like the repo's exp/tanh tables).
struct ReadTables {
    /// k0(r) = sqrt(1+r^2) in Q8, per radius code.
    k0_q8: [i32; 16],
    /// arcosh(1+u), u in Q16, indexed by leading bit (0..31) * 1024 + next 10 bits; output Q12.
    arcosh: Vec<i32>,
    /// exp(-d) for d in Q8 steps (0..4095), output Q15.
    exp: Vec<i32>,
}
fn build_read_tables() -> ReadTables {
    let mut k0_q8 = [0i32; 16];
    for rc in 0..16 {
        let r = R16[rc] as f64 / 16.0;
        k0_q8[rc] = ((1.0 + r * r).sqrt() * 256.0).round() as i32;
    }
    let mut arcosh = vec![0i32; 32 * 1024];
    for lb in 0..32 {
        for m in 0..1024u64 {
            let u_q16 = if lb >= 10 { (1u64 << lb) | (m << (lb - 10)) } else { (1u64 << lb) | (m >> (10 - lb)) };
            arcosh[lb * 1024 + m as usize] = ((1.0 + u_q16 as f64 / 65536.0).acosh() * 4096.0).round() as i32;
        }
    }
    let exp = (0..4096).map(|d| ((-(d as f64) / 256.0).exp() * 32768.0).round() as i32).collect();
    ReadTables { k0_q8, arcosh, exp }
}
fn arcosh_lookup(t: &ReadTables, u_q16: u64) -> i32 {
    if u_q16 == 0 {
        return 0;
    }
    let lb = (63 - u_q16.leading_zeros() as usize).min(31);
    let m = if lb >= 10 { (u_q16 >> (lb - 10)) & 1023 } else { (u_q16 << (10 - lb)) & 1023 };
    t.arcosh[lb * 1024 + m as usize]
}

struct ReadOut {
    top: Vec<(i32, u32)>, // (Z, key id)
    mix: [i32; 64],
}

/// Full multiplier-free Lorentz read for one query head (q8: query in Q5, int8):
/// table-row LUTs, bucketed ADC scan, a two-pass threshold (k-th best of per-block
/// best z bounds the k-th best key), exact z for survivors by shift-add, top-k,
/// arcosh/exp tables, and value mixing by 4-bit product tables.
/// `exact16`: 16-bit tables (exact on the codes) instead of rounded 8-bit tables.
fn lorentz_read(
    st: &KeyStore,
    values: &[[u8; 32]],
    q8: &[i8; 8],
    rt: &ReadTables,
    pt: &ProductTables,
    k: usize,
    exact16: bool,
    s_buf: &mut [i16],
    z_buf: &mut Vec<(i32, u32)>,
    hits: &mut Vec<usize>,
) -> ReadOut {
    // Z units: 2^9 per unit of z (8-bit tables) or 2^13 (16-bit tables).
    let zshift: i32 = if exact16 { 13 } else { 9 };
    let lut: [[i8; 16]; 8] = std::array::from_fn(|i| pt.ql[(q8[i] as i32 + 128) as usize]);
    let lo16: [[u8; 16]; 8] = std::array::from_fn(|i| pt.ql16_lo[(q8[i] as i32 + 128) as usize]);
    let hi16: [[u8; 16]; 8] = std::array::from_fn(|i| pt.ql16_hi[(q8[i] as i32 + 128) as usize]);
    // q0 = sqrt(1 + |q|^2) in Q8: squares by quarter-square table, isqrt by shifts.
    let mut n2 = 0i64; // |q8|^2 = |q|^2 * 2^10
    for i in 0..8 {
        n2 += qs_mul(pt, q8[i] as i32, q8[i] as i32);
    }
    let q0_q8 = isqrt(((1u64 << 10) + n2 as u64) << 6) as i32;
    // A[r] = q0*k0(r) in Z units: (Q8*Q8 = Q16) >> (16 - zshift).
    let a: [i32; 16] = std::array::from_fn(|rc| (qs_mul(pt, q0_q8, rt.k0_q8[rc]) >> (16 - zshift)) as i32);
    let prof = PROFILE.load(Relaxed);
    let t0 = if prof { Some(Instant::now()) } else { None };
    // Pass 1: s for every key; per-block best (max s) -> block-best z.
    let mut off = [0usize; 17];
    let mut block_best: Vec<i32> = Vec::with_capacity(96);
    for rc in 0..16 {
        let bytes = &st.buckets[rc];
        let n = bytes.len() / 128 * 32;
        off[rc + 1] = off[rc] + n;
        if n == 0 {
            continue;
        }
        let s = &mut s_buf[off[rc]..off[rc] + n];
        if exact16 {
            adc16_bucket(bytes, &lo16, &hi16, s);
        } else {
            adc_bucket(bytes, &lut, s);
        }
        let (hi, lo) = r16_shifts(rc);
        // Padded tail lanes (no key) can never be selected.
        let valid = st.count[rc];
        for x in s[valid..].iter_mut() {
            *x = i16::MIN;
        }
        for blk in s.chunks_exact(32) {
            let m = blk.iter().copied().fold(i16::MIN, i16::max); // auto-vectorized max
            let sm = m as i32;
            block_best.push(a[rc] - ((sm << hi) + lo.map_or(0, |l| sm << l)));
        }
    }
    let thresh = if block_best.len() >= k {
        *block_best.select_nth_unstable(k - 1).1
    } else {
        i32::MAX
    };
    let t1 = if prof { Some(Instant::now()) } else { None };
    // Pass 2: survivors with z <= thresh (tested as s >= s_thr in s-space, i16 SIMD compare).
    z_buf.clear();
    for rc in 0..16 {
        let n = off[rc + 1] - off[rc];
        if n == 0 {
            continue;
        }
        let s = &s_buf[off[rc]..off[rc] + n];
        let (hi, lo) = r16_shifts(rc);
        let base = a[rc];
        // z <= thresh  <=>  r16*s >= base - thresh  <=>  s > ceil((base-thresh)/r16) - 1
        let s_thr: i16 = if thresh == i32::MAX {
            i16::MIN
        } else {
            let num = base as i64 - thresh as i64;
            let r = R16[rc] as i64;
            ((num + r - 1).div_euclid(r) - 1).clamp(i16::MIN as i64, i16::MAX as i64) as i16
        };
        hits.clear();
        filter_gt(s, s_thr, hits);
        let ids = &st.ids[rc];
        for &i in hits.iter() {
            let id = ids[i];
            let sv = s[i] as i32;
            let z = base - ((sv << hi) + lo.map_or(0, |l| sv << l));
            if z <= thresh {
                z_buf.push((z, id));
            }
        }
    }
    SURV.fetch_add(z_buf.len() as u64, Relaxed);
    CALLS.fetch_add(1, Relaxed);
    let t2 = if prof { Some(Instant::now()) } else { None };
    if z_buf.len() > k {
        z_buf.select_nth_unstable(k - 1);
        z_buf.truncate(k);
    }
    // d = arcosh(z) with u = z - 1 in Q16.
    let mut dmin = i32::MAX;
    let mut dist = [0i32; 64];
    for (j, &(zv, _)) in z_buf.iter().enumerate() {
        let u = (zv as i64 - (1i64 << zshift)).max(0);
        dist[j] = arcosh_lookup(rt, (u << (16 - zshift)) as u64); // Q12
        dmin = dmin.min(dist[j]);
    }
    // softmax over -beta*d, beta = 2 (shift); exp table in Q8 steps; 7-bit masses
    // normalized by the leading-bit shift (a reciprocal table would make it exact).
    let mut w = [0i32; 64];
    let mut total = 0i64;
    for j in 0..z_buf.len() {
        let delta = ((dist[j] - dmin) << 1) >> 4;
        w[j] = rt.exp[(delta as usize).min(4095)];
        total += w[j] as i64;
    }
    let lz = 64 - (total.max(1) as u64).leading_zeros() as i32;
    let mut sel = [(0u32, 0i32); 64];
    for (j, &(_, id)) in z_buf.iter().enumerate() {
        sel[j] = (id, (((w[j] as i64) << 7) >> lz).min(127) as i32);
    }
    let mut mix = [0i32; 64];
    mix_values(pt, &sel[..z_buf.len()], values, &mut mix);
    if let (Some(a0), Some(a1), Some(a2)) = (t0, t1, t2) {
        let t3 = Instant::now();
        T_P1.fetch_add((a1 - a0).as_nanos() as u64, Relaxed);
        T_P2.fetch_add((a2 - a1).as_nanos() as u64, Relaxed);
        T_TAIL.fetch_add((t3 - a2).as_nanos() as u64, Relaxed);
    }
    ReadOut { top: z_buf.clone(), mix }
}


// ------------------------------------------- whole decode step stand-in
/// SmolLM2-shaped decode step with synthetic weights: per layer TL1 ternary
/// q/k/v/o and gate/up/down GEMVs, GQA Lorentz-36 reads over `ctx` keys per
/// KV head, then a signed-4-bit ELUT head GEMV over 49152 rows and argmax.
/// Norms/nonlinearities are replaced by a shift-and-clamp requantization.
/// Purpose: time and energy per token of the proposed serving kernels.
fn step_mode(d: usize, layers: usize, heads: usize, kv_heads: usize, inter: usize, ctx: usize, seconds: f64) {
    let vocab = 49152usize;
    let hd = 64usize;
    let mut rng = Rng(0xD1B54A32D192ED03);
    let mut tern = |m: usize, k: usize, rng: &mut Rng| {
        let w: Vec<i8> = (0..m * k).map(|_| rng.below(3) as i8 - 1).collect();
        tl1_pack(&w, m, k)
    };
    struct Layer {
        q: NibbleMatrix,
        k: NibbleMatrix,
        v: NibbleMatrix,
        o: NibbleMatrix,
        gate: NibbleMatrix,
        up: NibbleMatrix,
        down: NibbleMatrix,
        kv: Vec<(KeyStore, Vec<[u8; 32]>)>,
    }
    // One synthetic KV store, cloned per (layer, kv head) so each has its own memory.
    let keys: Vec<[f64; 8]> = (0..ctx)
        .map(|_| {
            let mut v = [0.0; 8];
            let mut nrm = 0.0;
            for x in v.iter_mut() {
                *x = rng.normal();
                nrm += *x * *x;
            }
            let r = (0.15f64.ln() + rng.f64u() * (20.0f64.ln() - 0.15f64.ln())).exp();
            for x in v.iter_mut() {
                *x *= r / nrm.sqrt();
            }
            v
        })
        .collect();
    let proto = build_store(&keys);
    let proto_vals: Vec<[u8; 32]> = (0..ctx).map(|_| std::array::from_fn(|_| rng.next() as u8)).collect();
    let clone_store = |s: &KeyStore| KeyStore {
        buckets: s.buckets.clone(),
        ids: s.ids.clone(),
        count: s.count,
        dir_codes: Vec::new(),
        r_code: Vec::new(),
    };
    let t_build = Instant::now();
    let layers_v: Vec<Layer> = (0..layers)
        .map(|_| Layer {
            q: tern(heads * hd, d, &mut rng),
            k: tern(kv_heads * hd, d, &mut rng),
            v: tern(kv_heads * hd, d, &mut rng),
            o: tern(d, heads * hd, &mut rng),
            gate: tern(inter, d, &mut rng),
            up: tern(inter, d, &mut rng),
            down: tern(d, inter, &mut rng),
            kv: (0..kv_heads).map(|_| (clone_store(&proto), proto_vals.clone())).collect(),
        })
        .collect();
    let head_w: Vec<i8> = (0..vocab * d).map(|_| rng.below(15) as i8 - 7).collect();
    let head = elut4_pack(&head_w, vocab, d);
    drop(head_w);
    let wbytes: usize = layers_v
        .iter()
        .map(|l| l.q.packed.len() + l.k.packed.len() + l.v.packed.len() + l.o.packed.len() + l.gate.packed.len() + l.up.packed.len() + l.down.packed.len())
        .sum::<usize>()
        + head.packed.len();
    let kvbytes: usize = layers_v
        .iter()
        .map(|l| l.kv.iter().map(|(s, v)| s.buckets.iter().map(|b| b.len()).sum::<usize>() + v.len() * 32).sum::<usize>())
        .sum();
    println!(
        "# step d={d} layers={layers} heads={heads}/{kv_heads} inter={inter} ctx={ctx}: packed weights {:.1} MB, KV codes+values {:.1} MB, built in {:.1}s",
        wbytes as f64 / 1e6, kvbytes as f64 / 1e6, t_build.elapsed().as_secs_f64()
    );
    let rt = build_read_tables();
    let pt = build_product_tables();
    let requant = |v: &[i32]| -> Vec<i8> { v.iter().map(|&x| (x >> 6).clamp(-127, 127) as i8).collect() };
    let mut x: Vec<i8> = (0..d).map(|_| rng.i8()).collect();
    let mut s_buf = vec![0i16; ctx + 32 * 16];
    let mut z_buf = Vec::with_capacity(256);
    let mut hits = Vec::with_capacity(ctx);
    let mut out_q = vec![0i32; heads * hd];
    let mut out_kv = vec![0i32; kv_heads * hd];
    let mut out_d = vec![0i32; d];
    let mut out_i = vec![0i32; inter];
    let mut out_i2 = vec![0i32; inter];
    let mut logits = vec![0i32; vocab];
    let group = heads / kv_heads;
    let start = Instant::now();
    let mut tokens = 0u64;
    let mut checksum = 0i64;
    while start.elapsed().as_secs_f64() < seconds {
        for l in &layers_v {
            let t = tl1_tables(&x);
            nibble_gemv(&l.q, &t, 64, &mut out_q);
            nibble_gemv(&l.k, &t, 64, &mut out_kv);
            nibble_gemv(&l.v, &t, 64, &mut out_kv);
            let mut attn = vec![0i32; heads * hd];
            for h in 0..heads {
                let q8: [i8; 8] = std::array::from_fn(|i| ((out_q[h * hd + i] >> 8).clamp(-127, 127)) as i8);
                let (st, vals) = &l.kv[h / group];
                let r = lorentz_read(st, vals, &q8, &rt, &pt, 32, false, &mut s_buf, &mut z_buf, &mut hits);
                attn[h * hd..(h + 1) * hd].copy_from_slice(&r.mix);
            }
            let ta = tl1_tables(&requant(&attn));
            nibble_gemv(&l.o, &ta, 64, &mut out_d);
            let h1 = requant(&out_d);
            let t1 = tl1_tables(&h1);
            nibble_gemv(&l.gate, &t1, 64, &mut out_i);
            nibble_gemv(&l.up, &t1, 64, &mut out_i2);
            let mid: Vec<i32> = out_i.iter().zip(&out_i2).map(|(a, b)| (a >> 4) + (b >> 4)).collect();
            let t2 = tl1_tables(&requant(&mid));
            nibble_gemv(&l.down, &t2, 64, &mut out_d);
            x = requant(&out_d);
        }
        let th = elut4_tables(&x);
        nibble_gemv(&head, &th, 32, &mut logits);
        let (arg, _) = logits.iter().enumerate().max_by_key(|p| *p.1).unwrap();
        checksum += arg as i64;
        tokens += 1;
    }
    let el = start.elapsed().as_secs_f64();
    println!(
        "# step result: {tokens} tokens in {el:.2}s = {:.2} ms/token, {:.1} tok/s (checksum {checksum})",
        1e3 * el / tokens as f64, tokens as f64 / el
    );
}

// ------------------------------------------------------------------ main
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let secs = args.iter().position(|a| a == "--seconds").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(20.0);
    let ctx = args.iter().position(|a| a == "--ctx").and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(2048usize);
    if args.iter().any(|a| a == "--step135") {
        step_mode(576, 30, 9, 3, 1536, ctx, secs);
        return;
    }
    if args.iter().any(|a| a == "--step360") {
        step_mode(960, 32, 15, 5, 2560, ctx, secs);
        return;
    }
    let quick = std::env::args().any(|a| a == "--quick");
    let ms: u128 = if quick { 40 } else { 200 };
    let mut rng = Rng(0x9E3779B97F4A7C15);
    println!("# geo-bench (sys2). arch={} quick={}", std::env::consts::ARCH, quick);

    // ---------------- weight kernels on SmolLM2-shaped matrices
    for &(name, m, k) in &[("135M mlp.up 1536x576", 1536usize, 576usize), ("360M mlp.up 2560x960", 2560, 960)] {
        let n = m * k;
        let xt: Vec<i8> = (0..k).map(|_| rng.i8()).collect();
        let wt: Vec<i8> = (0..n).map(|_| rng.below(3) as i8 - 1).collect();
        let w4: Vec<i8> = (0..n).map(|_| rng.below(15) as i8 - 7).collect();
        // exactness vs multiply reference
        let reft: Vec<i32> = (0..m).map(|r| (0..k).map(|j| wt[r * k + j] as i32 * xt[j] as i32).sum()).collect();
        let ref4: Vec<i32> = (0..m).map(|r| (0..k).map(|j| w4[r * k + j] as i32 * xt[j] as i32).sum()).collect();
        let tl1 = tl1_pack(&wt, m, k);
        let e4 = elut4_pack(&w4, m, k);
        let mut out = vec![0i32; m];
        let t1 = tl1_tables(&xt);
        nibble_gemv(&tl1, &t1, 64, &mut out);
        assert_eq!(out, reft, "TL1 mismatch");
        let t4 = elut4_tables(&xt);
        nibble_gemv(&e4, &t4, 32, &mut out);
        assert_eq!(out, ref4, "ELUT4 mismatch");
        let x32: Vec<i32> = xt.iter().map(|&v| v as i32).collect();
        let w16: Vec<i16> = w4.iter().map(|&v| v as i16).collect();
        let mut out64 = vec![0i64; m];
        gemv_repo(&x32, &w16, m, k, &mut out64);
        assert!(out64.iter().zip(&ref4).all(|(a, b)| *a == *b as i64), "repo mismatch");

        let ns_repo = time_ns(|| gemv_repo(black_box(&x32), black_box(&w16), m, k, black_box(&mut out64)), ms);
        let ns_tl1 = time_ns(
            || {
                let t = tl1_tables(black_box(&xt));
                nibble_gemv(black_box(&tl1), &t, 64, black_box(&mut out))
            },
            ms,
        );
        let ns_e4 = time_ns(
            || {
                let t = elut4_tables(black_box(&xt));
                nibble_gemv(black_box(&e4), &t, 32, black_box(&mut out))
            },
            ms,
        );
        let xf: Vec<f32> = xt.iter().map(|&v| v as f32).collect();
        let wf: Vec<f32> = w4.iter().map(|&v| v as f32).collect();
        let mut outf = vec![0f32; m];
        let ns_f32 = time_ns(|| gemv_f32(black_box(&xf), black_box(&wf), m, k, black_box(&mut outf)), ms);
        let mut outi = vec![0i32; m];
        let ns_i8 = time_ns(|| gemv_i8_mad(black_box(&xt), black_box(&w4), m, k, black_box(&mut outi)), ms);
        assert_eq!(outi, ref4);
        let per = |ns: f64| ns / n as f64 * 1000.0;
        println!(
            "gemv {name}: ps/weight  repo-gather(i16 codes) {:.0} | TL1 ternary LUT {:.0} | ELUT 4-bit LUT {:.0} | f32 FMA-free mul+add {:.0} | int8 MAD(mul) {:.0}   [bytes: {} / {} / {} / {} / {}]",
            per(ns_repo), per(ns_tl1), per(ns_e4), per(ns_f32), per(ns_i8),
            n * 2, tl1.packed.len(), e4.packed.len(), n * 4, n
        );
    }

    // ---------------- read kernels, T keys per query head
    for &t in &[2048usize, 256] {
        // Hyperbolic-ish keys: direction uniform, radius log-uniform in [0.15, 20].
        let keys: Vec<[f64; 8]> = (0..t)
            .map(|_| {
                let mut v = [0.0; 8];
                let mut nrm = 0.0;
                for x in v.iter_mut() {
                    *x = rng.normal();
                    nrm += *x * *x;
                }
                let r = (0.15f64.ln() + rng.f64u() * (20.0f64.ln() - 0.15f64.ln())).exp();
                for x in v.iter_mut() {
                    *x *= r / nrm.sqrt();
                }
                v
            })
            .collect();
        let st = build_store(&keys);
        let values: Vec<[u8; 32]> = (0..t)
            .map(|_| {
                let mut v = [0u8; 32];
                for b in v.iter_mut() {
                    *b = ((rng.below(15) as u8) & 15) | (((rng.below(15) as u8 + 9) & 15) << 4);
                }
                v
            })
            .collect();
        let q_shift = 5; // q8 = q * 2^5, |q| < 4
        let rt = build_read_tables();
        let pt = build_product_tables();
        let kk = 32;
        let mut s_buf = vec![0i16; t + 32 * 16];
        let mut z_buf = Vec::with_capacity(8 * kk);
        let mut hits = Vec::with_capacity(t);
        // SIMD vs scalar exactness for the ADC scan and the value mixing.
        {
            let q8: [i8; 8] = std::array::from_fn(|_| rng.i8());
            let lut: [[i8; 16]; 8] = std::array::from_fn(|i| pt.ql[(q8[i] as i32 + 128) as usize]);
            for rc in 0..16 {
                let bytes = &st.buckets[rc];
                let n = bytes.len() / 128 * 32;
                let mut a1 = vec![0i16; n];
                let mut a2 = vec![0i16; n];
                adc_bucket(bytes, &lut, &mut a1);
                adc_bucket_scalar(bytes, &lut, &mut a2);
                assert_eq!(a1, a2, "ADC SIMD mismatch");
            }
            let sel: Vec<(u32, i32)> = (0..32).map(|j| ((j * 7 % t) as u32, (rng.below(128)) as i32)).collect();
            let mut m1 = [0i32; 64];
            let mut m2 = [0i32; 64];
            mix_values(&pt, &sel, &values, &mut m1);
            mix_values_scalar(&pt, &sel, &values, &mut m2);
            assert_eq!(m1, m2, "value-mix SIMD mismatch");
            // exact product check against multiplication
            let mut m3 = [0i32; 64];
            for &(id, p) in &sel {
                for dd in 0..32 {
                    let b = values[id as usize][dd];
                    let c0 = ((b & 15) as i8) << 4 >> 4;
                    let c1 = ((b >> 4) as i8) << 4 >> 4;
                    let c0 = if (b & 15) == 8 { 0 } else { c0 };
                    let c1 = if (b >> 4) == 8 { 0 } else { c1 };
                    m3[2 * dd] += p * c0 as i32;
                    m3[2 * dd + 1] += p * c1 as i32;
                }
            }
            assert_eq!(m1, m3, "value-mix product mismatch");
        }
        // Accuracy: top-32 overlap vs exact f64 Lorentz on the same quantized keys, over 50 queries.
        let mut overlap = 0usize;
        let mut overlap_fp = 0usize;
        let mut overlap16 = 0usize;
        let nq = 50;
        let mut queries = Vec::new();
        for _ in 0..nq {
            let mut q = [0.0f64; 8];
            let mut nrm = 0.0;
            for x in q.iter_mut() {
                *x = rng.normal();
                nrm += *x * *x;
            }
            let r = 0.3 + 2.5 * rng.f64u();
            for x in q.iter_mut() {
                *x *= r / nrm.sqrt();
            }
            let q8: [i8; 8] = std::array::from_fn(|i| (q[i] * (1 << q_shift) as f64).round().clamp(-127.0, 127.0) as i8);
            let qq: [f64; 8] = std::array::from_fn(|i| q8[i] as f64 / (1 << q_shift) as f64);
            let q0 = (1.0 + qq.iter().map(|x| x * x).sum::<f64>()).sqrt();
            queries.push(q8);
            let out = lorentz_read(&st, &values, &q8, &rt, &pt, kk, false, &mut s_buf, &mut z_buf, &mut hits);
            let out16 = lorentz_read(&st, &values, &q8, &rt, &pt, kk, true, &mut s_buf, &mut z_buf, &mut hits);
            // exact z on quantized keys (same quantized query)
            let mut exact: Vec<(f64, u32)> = (0..t)
                .map(|i| {
                    let kq = dequant_key(&st.dir_codes[i], st.r_code[i]);
                    let r = R16[st.r_code[i] as usize] as f64 / 16.0;
                    let k0 = (1.0 + r * r).sqrt(); // the served definition (radius code)
                    let dot: f64 = (0..8).map(|j| qq[j] * kq[j]).sum();
                    (q0 * k0 - dot, i as u32)
                })
                .collect();
            exact.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let set: std::collections::HashSet<u32> = exact[..kk].iter().map(|p| p.1).collect();
            overlap += out.top.iter().filter(|p| set.contains(&p.1)).count();
            overlap16 += out16.top.iter().filter(|p| set.contains(&p.1)).count();
            // vs full-precision keys (float query): what the 36-bit code loses
            let q0f = (1.0 + q.iter().map(|x| x * x).sum::<f64>()).sqrt();
            let mut exactf: Vec<(f64, u32)> = (0..t)
                .map(|i| {
                    let kf = &keys[i];
                    let k0 = (1.0 + kf.iter().map(|x| x * x).sum::<f64>()).sqrt();
                    let dot: f64 = (0..8).map(|j| q[j] * kf[j]).sum();
                    (q0f * k0 - dot, i as u32)
                })
                .collect();
            exactf.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let setf: std::collections::HashSet<u32> = exactf[..kk].iter().map(|p| p.1).collect();
            overlap_fp += out.top.iter().filter(|p| setf.contains(&p.1)).count();
            black_box(out.mix);
        }
        // Timing: full Lorentz read per query head
        let mut qi = 0;
        let ns_lorentz = time_ns(
            || {
                let q8 = queries[qi % nq];
                qi += 1;
                let out = lorentz_read(&st, &values, &q8, &rt, &pt, kk, false, &mut s_buf, &mut z_buf, &mut hits);
                black_box(out.mix);
            },
            ms,
        );
        {
            SURV.store(0, Relaxed);
            CALLS.store(0, Relaxed);
            T_P1.store(0, Relaxed);
            T_P2.store(0, Relaxed);
            T_TAIL.store(0, Relaxed);
            PROFILE.store(true, Relaxed);
            for i in 0..2000 {
                let out = lorentz_read(&st, &values, &queries[i % nq], &rt, &pt, kk, false, &mut s_buf, &mut z_buf, &mut hits);
                black_box(out.mix);
            }
            PROFILE.store(false, Relaxed);
            let c = CALLS.load(Relaxed) as f64;
            println!(
                "read T={t}: profile (8-bit tables): survivors/query-head {:.1}; ns pass1(ADC+block max) {:.0} | pass2(filter+exact z) {:.0} | tail(select+arcosh+exp+mix) {:.0}",
                SURV.load(Relaxed) as f64 / c, T_P1.load(Relaxed) as f64 / c, T_P2.load(Relaxed) as f64 / c, T_TAIL.load(Relaxed) as f64 / c
            );
        }
        let mut qj = 0;
        let ns_lorentz16 = time_ns(
            || {
                let q8 = queries[qj % nq];
                qj += 1;
                let out = lorentz_read(&st, &values, &q8, &rt, &pt, kk, true, &mut s_buf, &mut z_buf, &mut hits);
                black_box(out.mix);
            },
            ms,
        );
        // Timing: ADC scan only (all buckets)
        let lut: [[i8; 16]; 8] = std::array::from_fn(|i| pt.ql[(queries[0][i] as i32 + 128) as usize]);
        let ns_adc = time_ns(
            || {
                for rc in 0..16 {
                    let bytes = &st.buckets[rc];
                    let n = bytes.len() / 128 * 32;
                    adc_bucket(black_box(bytes), &lut, &mut s_buf[..n]);
                }
                black_box(&s_buf);
            },
            ms,
        );
        // Component timings: block max over all s, and the s-space filter.
        {
            let n_all: usize = (0..16).map(|rc| st.buckets[rc].len() / 128 * 32).sum();
            let ns_max = time_ns(
                || {
                    let mut acc = 0i32;
                    for blk in black_box(&s_buf[..n_all]).chunks_exact(32) {
                        acc += blk.iter().copied().fold(i16::MIN, i16::max) as i32;
                    }
                    black_box(acc);
                },
                ms,
            );
            let mut sorted: Vec<i16> = s_buf[..n_all].to_vec();
            sorted.sort_unstable();
            let thr = sorted[n_all - 200];
            let ns_filter = time_ns(
                || {
                    hits.clear();
                    filter_gt(black_box(&s_buf[..n_all]), thr, &mut hits);
                    black_box(hits.len());
                },
                ms,
            );
            println!("read T={t}: components ns: block-max over {n_all} lanes {:.0} | filter_gt (~200 hits) {:.0}", ns_max, ns_filter);
        }
        // Sign-code popcount scoring of 64-bit codes + top-32
        let codes: Vec<u64> = (0..t).map(|_| rng.next()).collect();
        let qc = rng.next();
        let mut dist = vec![0u32; t];
        let ns_pop = time_ns(
            || {
                for (d, &c) in dist.iter_mut().zip(black_box(&codes)) {
                    *d = (c ^ qc).count_ones();
                }
                black_box(&dist);
            },
            ms,
        );
        // Hardware int16 64-dim dot (multiplier; comparator)
        let k16: Vec<[i16; 64]> = (0..t).map(|_| std::array::from_fn(|_| (rng.next() >> 49) as i16 - 16384)).collect();
        let q16: [i16; 64] = std::array::from_fn(|_| (rng.next() >> 49) as i16 - 16384);
        let mut sc = vec![0i64; t];
        let ns_hw = time_ns(
            || {
                for (s, kv) in sc.iter_mut().zip(black_box(&k16)) {
                    let d: i32 = q16.iter().zip(kv.iter()).map(|(a, b)| (*a as i32).wrapping_mul(*b as i32)).fold(0i32, |x, y| x.wrapping_add(y));
                    *s = d as i64;
                }
                black_box(&sc);
            },
            ms,
        );
        // Repo software-multiply dot (i128 shift-add), 64 dims, sampled on 64 keys
        let samp = 64.min(t);
        let ns_soft = time_ns(
            || {
                for (s, kv) in sc.iter_mut().zip(black_box(&k16)).take(samp) {
                    let mut dot = 0i128;
                    for j in 0..64 {
                        dot += soft_mul(q16[j] as i128, kv[j] as i128);
                    }
                    *s = dot as i64;
                }
                black_box(&sc);
            },
            ms,
        ) * (t as f64 / samp as f64);
        println!(
            "read T={t}: ns per query-head  Lorentz36 full(read+top{kk}+arcosh+exp+4bit-product-table mix) 8-bit tables {:.0} / exact 16-bit tables {:.0} | ADC scan only {:.0} | sign-popcount 64b {:.0} | hw int16 dot64 {:.0} | repo soft-mul dot64 {:.0} (extrapolated)",
            ns_lorentz, ns_lorentz16, ns_adc, ns_pop, ns_hw, ns_soft
        );
        println!(
            "read T={t}: per key ns  Lorentz36 {:.2} | ADC {:.2} | popcount {:.2} | hw dot {:.2} | soft-mul {:.1};  top-{kk} overlap vs exact-on-codes {:.3} (16-bit tables {:.3}), vs full-precision keys {:.3}; key bytes/query-head {} (36-bit) vs {} (fp16 8-dim) vs {} (fp16 64-dim)",
            ns_lorentz / t as f64, ns_adc / t as f64, ns_pop / t as f64, ns_hw / t as f64, ns_soft / t as f64,
            overlap as f64 / (nq * kk) as f64, overlap16 as f64 / (nq * kk) as f64, overlap_fp as f64 / (nq * kk) as f64,
            t * 36 / 8, t * 16, t * 128
        );
    }

    // ---------------- elementwise activation x activation products
    let nprod = 1 << 16;
    let a: Vec<i32> = (0..nprod).map(|_| (rng.next() >> 49) as i32 - 16384).collect();
    let b: Vec<i32> = (0..nprod).map(|_| (rng.next() >> 49) as i32 - 16384).collect();
    // quarter-square table for |a+b|,|a-b| <= 32768: q(n) = floor(n^2/4)
    let sq: Vec<u32> = (0..=32768u64).map(|n| (n * n / 4) as u32).collect();
    let mut o = vec![0i64; nprod];
    for i in 0..nprod {
        let s = (a[i] + b[i]).unsigned_abs() as usize;
        let d = (a[i] - b[i]).unsigned_abs() as usize;
        o[i] = sq[s] as i64 - sq[d] as i64;
        assert_eq!(o[i], a[i] as i64 * b[i] as i64);
    }
    let ns_qs = time_ns(
        || {
            for i in 0..nprod {
                let s = (a[i] + b[i]).unsigned_abs() as usize;
                let d = (a[i] - b[i]).unsigned_abs() as usize;
                o[i] = sq[s] as i64 - sq[d] as i64;
            }
            black_box(&o);
        },
        ms,
    );
    let ns_soft = time_ns(
        || {
            for i in 0..4096 {
                o[i] = soft_mul(a[i] as i128, b[i] as i128) as i64;
            }
            black_box(&o);
        },
        ms,
    ) * (nprod as f64 / 4096.0);
    let ns_hwm = time_ns(
        || {
            for i in 0..nprod {
                o[i] = a[i] as i64 * b[i] as i64;
            }
            black_box(&o);
        },
        ms,
    );
    println!(
        "elementwise int16 x int16 products, ns/product: quarter-square table {:.2} | repo software shift-add {:.1} | hardware mul {:.2}",
        ns_qs / nprod as f64, ns_soft / nprod as f64, ns_hwm / nprod as f64
    );
}
