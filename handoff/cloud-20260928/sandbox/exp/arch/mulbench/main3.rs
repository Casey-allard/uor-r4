// Copies checked_mul_unsigned / checked_mul / shift_left_unsigned verbatim from
// crates/uor-r4-integer/src/math.rs (lines 49-83) and times them against hardware multiply.
use std::hint::black_box;
use std::time::Instant;
#[derive(Debug)] enum E { Overflow, InvalidShift }
fn shift_left_unsigned(value: u128, shift: u32) -> Result<u128, E> {
    if shift >= 128 { return Err(E::InvalidShift); }
    if value > (u128::MAX >> shift) { return Err(E::Overflow); }
    Ok(value << shift)
}
fn checked_mul_unsigned(mut left: u128, mut right: u128) -> Result<u128, E> {
    if left < right { std::mem::swap(&mut left, &mut right); }
    let mut result = 0u128;
    while right != 0 {
        if right & 1 != 0 { result = result.checked_add(left).ok_or(E::Overflow)?; }
        right >>= 1;
        if right != 0 { left = shift_left_unsigned(left, 1)?; }
    }
    Ok(result)
}
fn signed_magnitude(m: u128, neg: bool) -> Result<i128, E> {
    if neg && m == (1u128 << 127) { return Ok(i128::MIN); }
    let v = i128::try_from(m).map_err(|_| E::Overflow)?;
    Ok(if neg { -v } else { v })
}
fn checked_mul(l: i128, r: i128) -> Result<i128, E> {
    let m = checked_mul_unsigned(l.unsigned_abs(), r.unsigned_abs())?;
    signed_magnitude(m, (l < 0) != (r < 0))
}

fn div_rem_unsigned(numerator: u128, denominator: u128) -> (u128, u128) {
    if numerator < denominator { return (0, numerator); }
    let shift = denominator.leading_zeros() - numerator.leading_zeros();
    let mut divisor = denominator << shift; let mut place = 1u128 << shift;
    let mut remainder = numerator; let mut quotient = 0u128;
    while place != 0 { if remainder >= divisor { remainder -= divisor; quotient |= place; } divisor >>= 1; place >>= 1; }
    (quotient, remainder)
}

fn low_bit_products(input: &[i32]) -> Vec<[i64; 16]> {   // verbatim from crates/uor-r4-integer/src/model.rs:90-107
    input.iter().map(|&x| { let x = i64::from(x); let twice = x << 1; let four = x << 2; let three = x + twice;
        let five = x + four; let six = twice + four; let seven = (x << 3) - x;
        [0, x, twice, three, four, five, six, seven, 0, -seven, -six, -five, -four, -three, -twice, -x] }).collect()
}
fn low_bit_dot(products: &[[i64; 16]], weights: &[i16]) -> i64 {  // model.rs:114-120
    let mut total = 0i64; for (m, &w) in products.iter().zip(weights) { total += m[usize::from((w as u16) & 15)]; } total
}
fn xs(s: &mut u64) -> u64 { *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s }
fn main() {
    let mut s = 0x1234_5678_9abc_def1u64;
    // dense signed4 maps per token: W_in(768x256, token affine) W_s(768x256) Q(64x256) K(64x256) V(256x256)
    // null(1x256) update(256x512) gates(2x512) logits(4096x256)
    let shapes: [(usize, usize); 9] = [(768,256),(768,256),(64,256),(64,256),(256,256),(1,256),(256,512),(2,512),(4096,256)];
    let mats: Vec<Vec<i16>> = shapes.iter().map(|&(r,c)| (0..r*c).map(|_| ((xs(&mut s) % 15) as i16) - 7).collect()).collect();
    let reps = 50; let t = Instant::now(); let mut acc = 0i64;
    for _ in 0..reps {
        for (k, &(r, c)) in shapes.iter().enumerate() {
            let input: Vec<i32> = (0..c).map(|_| ((xs(&mut s) % 65535) as i32) - 32767).collect();
            let p = low_bit_products(&input);
            for row in mats[k].chunks_exact(c) { acc = acc.wrapping_add(low_bit_dot(black_box(&p), black_box(row))); }
        }
    }
    let tab = t.elapsed().as_secs_f64() / reps as f64; black_box(acc);
    let weights: usize = shapes.iter().map(|&(r,c)| r*c).sum();
    // activation products and divisions per full-context token (counted from model.rs): 97,217 products, 9,472 divisions
    let n_prod = 97_217usize; let n_div = 9_472usize;
    let a: Vec<i128> = (0..n_prod).map(|_| (xs(&mut s) >> 20) as i128).collect();
    let b: Vec<i128> = (0..n_prod).map(|_| ((xs(&mut s) % 32769) as i128) - 16384).collect();
    let t = Instant::now(); let mut acc2 = 0i128;
    for _ in 0..10 { for i in 0..n_prod { acc2 = acc2.wrapping_add(checked_mul(black_box(a[i]), black_box(b[i])).unwrap()); } }
    let prod = t.elapsed().as_secs_f64() / 10.0; black_box(acc2);
    let w: Vec<u128> = (0..n_div).map(|_| (xs(&mut s) >> 16) as u128).collect(); let sum: u128 = w.iter().sum();
    let t = Instant::now(); let mut acc3 = 0u128;
    for _ in 0..10 { for i in 0..n_div { acc3 = acc3.wrapping_add(div_rem_unsigned(black_box(w[i] << 48), black_box(sum)).0); } }
    let div = t.elapsed().as_secs_f64() / 10.0; black_box(acc3);
    println!("signed4 table matmuls: {} weights/token -> {:.2} ms/token ({:.2} ns/weight)", weights, tab*1e3, tab*1e9/weights as f64);
    println!("software activation products: {} -> {:.2} ms/token; software divisions: {} -> {:.2} ms/token", n_prod, prod*1e3, n_div, div*1e3);
    println!("share of software arithmetic in (table+products+divisions): {:.0}%", 100.0*(prod+div)/(tab+prod+div));
}
