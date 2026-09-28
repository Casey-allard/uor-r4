//! Physics-review micro-benchmark (scratch; NOT part of the repo).
//!
//! Question: on a real CPU, does removing the *multiplier instruction* save work
//! (a proxy for energy at roughly constant core power), or does it cost work?
//!
//! Kernels copied verbatim (semantics) from crates/uor-r4-integer/src/{model,math}.rs:
//!   - low_bit_products / low_bit_dot  (signed4 weight path, i16 codes, [i64;16] tables)
//!   - checked_mul (software shift-and-add over u128)   used for Q.K, A.V, mixtures
//!   - div_round  (software binary long division, u128) used in softmax/normalization/mixture
//! and compared with exact-result alternatives that use the hardware multiplier,
//! plus one multiplier-free but SIMD-friendly variant (bit-mask shift/add).
//! Every alternative is checked for bit-exact equality with the repo kernel.
use std::hint::black_box;
use std::time::Instant;

// ------------------------------------------------------------ repo semantics
#[derive(Debug)]
enum E {
    Overflow,
    Div0,
}
fn signed_magnitude(m: u128, neg: bool) -> Result<i128, E> {
    if neg && m == (1u128 << 127) {
        return Ok(i128::MIN);
    }
    let v = i128::try_from(m).map_err(|_| E::Overflow)?;
    Ok(if neg { -v } else { v })
}
fn shift_left_unsigned(v: u128, s: u32) -> Result<u128, E> {
    if s >= 128 || v > (u128::MAX >> s) {
        return Err(E::Overflow);
    }
    Ok(v << s)
}
#[inline(never)]
fn checked_mul_unsigned(mut left: u128, mut right: u128) -> Result<u128, E> {
    if left < right {
        std::mem::swap(&mut left, &mut right);
    }
    let mut result = 0u128;
    while right != 0 {
        if right & 1 != 0 {
            result = result.checked_add(left).ok_or(E::Overflow)?;
        }
        right >>= 1;
        if right != 0 {
            left = shift_left_unsigned(left, 1)?;
        }
    }
    Ok(result)
}
fn checked_mul(a: i128, b: i128) -> Result<i128, E> {
    let m = checked_mul_unsigned(a.unsigned_abs(), b.unsigned_abs())?;
    signed_magnitude(m, (a < 0) != (b < 0))
}
#[inline(never)]
fn div_rem_unsigned(n: u128, d: u128) -> Result<(u128, u128), E> {
    if d == 0 {
        return Err(E::Div0);
    }
    if n < d {
        return Ok((0, n));
    }
    let shift = d.leading_zeros() - n.leading_zeros();
    let mut divisor = d << shift;
    let mut place = 1u128 << shift;
    let mut rem = n;
    let mut q = 0u128;
    while place != 0 {
        if rem >= divisor {
            rem -= divisor;
            q |= place;
        }
        divisor >>= 1;
        place >>= 1;
    }
    Ok((q, rem))
}
fn div_round(n: i128, d: i128) -> Result<i128, E> {
    let dv = d.unsigned_abs();
    let (q, r) = div_rem_unsigned(n.unsigned_abs(), dv)?;
    let rounded = q.checked_add(u128::from(r >= dv - r)).ok_or(E::Overflow)?;
    signed_magnitude(rounded, (n < 0) != (d < 0))
}
// hardware-division version with identical rounding (ties away from zero)
fn div_round_hw(n: i128, d: i128) -> i128 {
    let dv = d.unsigned_abs();
    let nu = n.unsigned_abs();
    let q = nu / dv;
    let r = nu % dv;
    let rounded = q + u128::from(r >= dv - r);
    let v = rounded as i128;
    if (n < 0) != (d < 0) {
        -v
    } else {
        v
    }
}

#[inline(never)]
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
            [0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three, -twice, -x]
        })
        .collect()
}
#[inline(never)]
fn low_bit_dot(products: &[[i64; 16]], weights: &[i16]) -> i64 {
    let mut total = 0i64;
    for (m, &w) in products.iter().zip(weights) {
        total += m[usize::from((w as u16) & 15)];
    }
    total
}

// ------------------------------------------------------------ alternatives
#[inline(never)]
fn mul_dot_i64(x: &[i32], w: &[i16]) -> i64 {
    // hardware multiply, wide accumulate
    x.iter().zip(w).map(|(&a, &b)| i64::from(a) * i64::from(b)).sum()
}
#[inline(never)]
fn mul_dot_i16_i32(x: &[i16], w: &[i8]) -> i32 {
    // narrow operands (|x|<=32767, |w|<=7, 256 terms fit i32) -> SIMD multiply-add
    x.iter().zip(w).map(|(&a, &b)| i32::from(a) * i32::from(b)).sum()
}
#[inline(never)]
fn packed4_dot(x: &[i16], packed: &[u8]) -> i32 {
    // two signed4 codes per byte (low nibble = even index); sign-extend and multiply
    let mut acc = 0i32;
    for (pair, xs) in packed.iter().zip(x.chunks_exact(2)) {
        let lo = (((pair & 15) as i8) << 4) >> 4;
        let hi = ((*pair as i8) >> 4) as i8;
        acc += i32::from(xs[0]) * i32::from(lo) + i32::from(xs[1]) * i32::from(hi);
    }
    acc
}
#[inline(never)]
fn nomul_mask_dot(x: &[i16], sm: &[u8]) -> i32 {
    // Multiplier-free, branch-free, SIMD-friendly: code = sign<<3 | magnitude(0..7)
    // term = sum_k ((x<<k) & -bit_k), then conditional negate: (t ^ -s) + s
    let mut acc = 0i32;
    for (&xv, &c) in x.iter().zip(sm) {
        let x = i32::from(xv);
        let c = i32::from(c);
        let b0 = -(c & 1);
        let b1 = -((c >> 1) & 1);
        let b2 = -((c >> 2) & 1);
        let s = -((c >> 3) & 1);
        let t = (x & b0) + ((x << 1) & b1) + ((x << 2) & b2);
        acc += (t ^ s) - s;
    }
    acc
}

// ------------------------------------------------------------ harness
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }
}

fn time_ns_per_op<F: FnMut() -> u64>(ops_per_call: f64, mut f: F) -> (f64, u64) {
    // calibrate to ~0.25 s per repeat, 5 repeats, report min
    let mut calls = 1u64;
    loop {
        let t = Instant::now();
        let mut s = 0u64;
        for _ in 0..calls {
            s = s.wrapping_add(f());
        }
        black_box(s);
        if t.elapsed().as_secs_f64() > 0.05 {
            break;
        }
        calls *= 2;
    }
    let calls = (calls as f64 * 0.25 / 0.05).max(1.0) as u64;
    let mut best = f64::MAX;
    let mut sink = 0u64;
    for _ in 0..5 {
        let t = Instant::now();
        for _ in 0..calls {
            sink = sink.wrapping_add(f());
        }
        let ns = t.elapsed().as_secs_f64() * 1e9 / (calls as f64 * ops_per_call);
        best = best.min(ns);
    }
    (best, sink)
}

fn main() {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let (rows, cols) = (768usize, 256usize); // recurrent.state.weight shape [3d, d]
    let input: Vec<i32> = (0..cols).map(|_| rng.range(-32767, 32767) as i32).collect();
    let input16: Vec<i16> = input.iter().map(|&v| v as i16).collect();
    let codes: Vec<i16> = (0..rows * cols).map(|_| rng.range(-7, 7) as i16).collect();
    let codes8: Vec<i8> = codes.iter().map(|&c| c as i8).collect();
    let packed: Vec<u8> = codes
        .chunks_exact(2)
        .map(|p| ((p[0] as u8) & 15) | (((p[1] as u8) & 15) << 4))
        .collect();
    let signmag: Vec<u8> = codes
        .iter()
        .map(|&c| if c < 0 { 8u8 | ((-c) as u8) } else { c as u8 })
        .collect();

    // exactness checks
    let prods = low_bit_products(&input);
    for r in 0..rows {
        let row = &codes[r * cols..(r + 1) * cols];
        let a = low_bit_dot(&prods, row);
        assert_eq!(a, mul_dot_i64(&input, row));
        assert_eq!(a, i64::from(mul_dot_i16_i32(&input16, &codes8[r * cols..(r + 1) * cols])));
        assert_eq!(a, i64::from(packed4_dot(&input16, &packed[r * cols / 2..(r + 1) * cols / 2])));
        assert_eq!(a, i64::from(nomul_mask_dot(&input16, &signmag[r * cols..(r + 1) * cols])));
    }
    let terms = (rows * cols) as f64;
    let mut input = input;
    let mut input16 = input16;
    let mut tick = 0usize;
    println!("# weight path: [{rows} x {cols}] signed4 affine, ns per weight term (min of 5 x ~0.25 s)");
    macro_rules! bench {
        ($label:expr, $ops:expr, $body:expr) => {{
            let (t, _) = time_ns_per_op($ops, $body);
            println!("{:<78} {:>9.3}", $label, t);
        }};
    }
    bench!("repo low_bit_products+low_bit_dot (i16 codes, i64 table gather, no MUL)", terms, || {
        tick = tick.wrapping_add(1); input[tick % cols] ^= 1;
        w_repo(black_box(&input), black_box(&codes), rows, cols)
    });
    bench!("hardware MUL, i32 x i16 -> i64 accumulate (same i16 codes)", terms, || {
        tick = tick.wrapping_add(1); input[tick % cols] ^= 1;
        w_mul64(black_box(&input), black_box(&codes), rows, cols)
    });
    bench!("hardware MUL, i16 x i8 -> i32 accumulate (SIMD multiply-add)", terms, || {
        tick = tick.wrapping_add(1); input16[tick % cols] ^= 1;
        w_mul16(black_box(&input16), black_box(&codes8), rows, cols)
    });
    bench!("hardware MUL, packed 4-bit nibbles (0.5 B/weight) -> i32", terms, || {
        tick = tick.wrapping_add(1); input16[tick % cols] ^= 1;
        w_packed(black_box(&input16), black_box(&packed), rows, cols)
    });
    bench!("NO MUL, branch-free bit-mask shift/add (u8 sign-magnitude codes)", terms, || {
        tick = tick.wrapping_add(1); input16[tick % cols] ^= 1;
        w_nomul(black_box(&input16), black_box(&signmag), rows, cols)
    });

    // ------------------------------------------------ attention-style products
    let keys: Vec<i32> = (0..255 * 64).map(|_| rng.range(-32767, 32767) as i32).collect();
    let mut q: Vec<i32> = (0..64).map(|_| rng.range(-32767, 32767) as i32).collect();
    let mut masses: Vec<u64> = (0..255).map(|_| rng.next() % (1u64 << 48)).collect();
    let values: Vec<i32> = (0..255 * 256).map(|_| rng.range(-32767, 32767) as i32).collect();
    assert_eq!(qk_soft(&q, &keys), qk_hw(&q, &keys));
    assert_eq!(av_soft(&masses, &values), av_hw(&masses, &values));
    println!("\n# attention-style activation x activation products (full 255-key window), ns per product");
    bench!("repo Q.K via software shift/add checked_mul (i128)", (255 * 64) as f64, || {
        tick = tick.wrapping_add(1); q[tick % 64] ^= 1;
        qk_soft(black_box(&q), black_box(&keys)) as u64
    });
    bench!("hardware MUL Q.K (i32 x i32 -> i64)", (255 * 64) as f64, || {
        tick = tick.wrapping_add(1); q[tick % 64] ^= 1;
        qk_hw(black_box(&q), black_box(&keys)) as u64
    });
    bench!("repo A.V via software checked_mul (u64 mass x i32 -> i128)", (255 * 256) as f64, || {
        tick = tick.wrapping_add(1); masses[tick % 255] ^= 1;
        av_soft(black_box(&masses), black_box(&values)) as u64
    });
    bench!("hardware MUL A.V (i128 widening multiply)", (255 * 256) as f64, || {
        tick = tick.wrapping_add(1); masses[tick % 255] ^= 1;
        av_hw(black_box(&masses), black_box(&values)) as u64
    });

    // ------------------------------------------------ division
    let mut nums: Vec<i128> = (0..4096).map(|_| (i128::from(rng.next() % (1u64 << 48))) * 99_999_999 + i128::from(1u64 << 36)).collect();
    for &n in &nums[..256] {
        assert_eq!(div_round(n, 100_000_000).unwrap(), div_round_hw(n, 100_000_000));
    }
    println!("\n# division by 100_000_000 of ~2^75 numerators (vocabulary mixture), ns per division");
    bench!("repo software long division div_round (u128)", 4096.0, || {
        tick = tick.wrapping_add(1); nums[tick % 4096] ^= 1;
        d_soft(black_box(&nums)) as u64
    });
    bench!("hardware/compiler-rt division (u128 /, %)", 4096.0, || {
        tick = tick.wrapping_add(1); nums[tick % 4096] ^= 1;
        d_hw(black_box(&nums)) as u64
    });
}

#[inline(never)]
fn w_repo(input: &[i32], codes: &[i16], rows: usize, cols: usize) -> u64 {
    let p = low_bit_products(input);
    let mut s = 0i64;
    for r in 0..rows { s = s.wrapping_add(low_bit_dot(&p, &codes[r * cols..(r + 1) * cols])); }
    s as u64
}
#[inline(never)]
fn w_mul64(input: &[i32], codes: &[i16], rows: usize, cols: usize) -> u64 {
    let mut s = 0i64;
    for r in 0..rows { s = s.wrapping_add(mul_dot_i64(input, &codes[r * cols..(r + 1) * cols])); }
    s as u64
}
#[inline(never)]
fn w_mul16(input: &[i16], codes: &[i8], rows: usize, cols: usize) -> u64 {
    let mut s = 0i32;
    for r in 0..rows { s = s.wrapping_add(mul_dot_i16_i32(input, &codes[r * cols..(r + 1) * cols])); }
    s as u64
}
#[inline(never)]
fn w_packed(input: &[i16], packed: &[u8], rows: usize, cols: usize) -> u64 {
    let mut s = 0i32;
    for r in 0..rows { s = s.wrapping_add(packed4_dot(input, &packed[r * cols / 2..(r + 1) * cols / 2])); }
    s as u64
}
#[inline(never)]
fn w_nomul(input: &[i16], sm: &[u8], rows: usize, cols: usize) -> u64 {
    let mut s = 0i32;
    for r in 0..rows { s = s.wrapping_add(nomul_mask_dot(input, &sm[r * cols..(r + 1) * cols])); }
    s as u64
}
#[inline(never)]
fn qk_soft(q: &[i32], keys: &[i32]) -> i128 {
    let mut s = 0i128;
    for k in keys.chunks_exact(64) {
        for (&x, &y) in q.iter().zip(k) { s += checked_mul(i128::from(x), i128::from(y)).unwrap(); }
    }
    s
}
#[inline(never)]
fn qk_hw(q: &[i32], keys: &[i32]) -> i128 {
    let mut s = 0i64;
    for k in keys.chunks_exact(64) {
        for (&x, &y) in q.iter().zip(k) { s += i64::from(x) * i64::from(y); }
    }
    i128::from(s)
}
#[inline(never)]
fn av_soft(masses: &[u64], values: &[i32]) -> i128 {
    let mut s = 0i128;
    for c in 0..256 {
        for (i, m) in masses.iter().enumerate() { s += checked_mul(i128::from(*m), i128::from(values[i * 256 + c])).unwrap(); }
    }
    s
}
#[inline(never)]
fn av_hw(masses: &[u64], values: &[i32]) -> i128 {
    let mut s = 0i128;
    for c in 0..256 {
        for (i, m) in masses.iter().enumerate() { s += i128::from(*m) * i128::from(values[i * 256 + c]); }
    }
    s
}
#[inline(never)]
fn d_soft(nums: &[i128]) -> i128 {
    let mut s = 0i128;
    for &n in nums { s += div_round(n, 100_000_000).unwrap(); }
    s
}
#[inline(never)]
fn d_hw(nums: &[i128]) -> i128 {
    let mut s = 0i128;
    for &n in nums { s += div_round_hw(n, 100_000_000); }
    s
}
