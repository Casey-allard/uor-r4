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
fn xs(s: &mut u64) -> u64 { *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s }
fn main() {
    let m = 4096usize; let reps = 1000usize; let n = m * reps;
    let mut s = 0x9e3779b97f4a7c15u64;
    let a: Vec<i128> = (0..m).map(|_| (xs(&mut s) >> 16) as i128).collect();
    let b: Vec<i128> = (0..m).map(|_| ((xs(&mut s) % 32769) as i128) - 16384).collect();
    let c: Vec<i128> = (0..m).map(|_| ((xs(&mut s) % 32769) as i128) - 16384).collect();
    let d: Vec<i128> = (0..m).map(|_| ((xs(&mut s) % 65535) as i128) - 32767).collect();
    for (label, x, y) in [("A mass48 x value14", &a, &b), ("B unit14 x state15", &c, &d)] {
        let t = Instant::now(); let mut acc = 0i128;
        for _ in 0..reps { for i in 0..m { acc = acc.wrapping_add(checked_mul(black_box(x[i]), black_box(y[i])).unwrap()); } }
        let sw = t.elapsed().as_secs_f64(); black_box(acc);
        let x64: Vec<i64> = x.iter().map(|&v| v as i64).collect(); let y64: Vec<i64> = y.iter().map(|&v| v as i64).collect();
        let t = Instant::now(); let mut acc3 = 0i64;
        for _ in 0..reps { for i in 0..m { acc3 = acc3.wrapping_add(black_box(x64[i]).wrapping_mul(black_box(y64[i]))); } }
        let hw64 = t.elapsed().as_secs_f64(); black_box(acc3);
        println!("{label:22}: software {:6.2} ns/mul | hw i64 {:5.2} ns/mul | ratio {:5.1}x", sw*1e9/n as f64, hw64*1e9/n as f64, sw/hw64);
    }
    // softmax-style division: (w << 48) / sum   [model.rs:527]
    let w: Vec<u128> = (0..m).map(|_| (xs(&mut s) >> 16) as u128).collect();
    let sum: u128 = w.iter().sum::<u128>();
    let t = Instant::now(); let mut acc = 0u128;
    for _ in 0..reps { for i in 0..m { acc = acc.wrapping_add(div_rem_unsigned(black_box(w[i] << 48), black_box(sum)).0); } }
    let sw = t.elapsed().as_secs_f64(); black_box(acc);
    let t = Instant::now(); let mut acc2 = 0u128;
    for _ in 0..reps { for i in 0..m { acc2 = acc2.wrapping_add(black_box(w[i] << 48) / black_box(sum)); } }
    let hw = t.elapsed().as_secs_f64(); black_box(acc2);
    assert_eq!(acc, acc2);
    println!("{:22}: software {:6.2} ns/div | hw u128 div {:5.2} ns/div | ratio {:5.1}x", "C (w<<48)/sum", sw*1e9/n as f64, hw*1e9/n as f64, sw/hw);
}
