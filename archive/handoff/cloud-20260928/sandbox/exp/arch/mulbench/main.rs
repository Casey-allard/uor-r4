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
fn xs(s: &mut u64) -> u64 { *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s }
fn main() {
    let n = 4_000_000usize;
    let mut s = 0x9e3779b97f4a7c15u64;
    // Case A: attention read  -> mass (Q48, up to 2^48) x value (Q14, |v|<=16384)   [model.rs:338]
    // Case B: transport/blend -> unit (Q14) x state (Q11, |x|<=32767)               [model.rs:488,437]
    let mut a = Vec::with_capacity(n); let mut b = Vec::with_capacity(n);
    let mut c = Vec::with_capacity(n); let mut d = Vec::with_capacity(n);
    for _ in 0..n {
        a.push((xs(&mut s) >> 16) as i128);                      // ~48-bit mass
        b.push(((xs(&mut s) % 32769) as i128) - 16384);           // Q14 value
        c.push(((xs(&mut s) % 32769) as i128) - 16384);           // Q14 unit coordinate
        d.push(((xs(&mut s) % 65535) as i128) - 32767);           // Q11 state code
    }
    for (label, x, y) in [("A mass48 x value14", &a, &b), ("B unit14 x state15", &c, &d)] {
        // software (project) multiply
        let t = Instant::now(); let mut acc = 0i128;
        for i in 0..n { acc = acc.wrapping_add(checked_mul(black_box(x[i]), black_box(y[i])).unwrap()); }
        let sw = t.elapsed().as_secs_f64(); black_box(acc);
        // hardware i128 multiply
        let t = Instant::now(); let mut acc2 = 0i128;
        for i in 0..n { acc2 = acc2.wrapping_add(black_box(x[i]).wrapping_mul(black_box(y[i]))); }
        let hw = t.elapsed().as_secs_f64(); black_box(acc2);
        // hardware i64 multiply (operands fit)
        let x64: Vec<i64> = x.iter().map(|&v| v as i64).collect(); let y64: Vec<i64> = y.iter().map(|&v| v as i64).collect();
        let t = Instant::now(); let mut acc3 = 0i64;
        for i in 0..n { acc3 = acc3.wrapping_add(black_box(x64[i]).wrapping_mul(black_box(y64[i]))); }
        let hw64 = t.elapsed().as_secs_f64(); black_box(acc3);
        assert_eq!(acc, acc2);
        println!("{label:22}: software {:6.2} ns/mul | hw i128 {:5.2} ns | hw i64 {:5.2} ns | ratio sw/hw64 {:6.1}x",
                 sw*1e9/n as f64, hw*1e9/n as f64, hw64*1e9/n as f64, sw/hw64);
    }
}
