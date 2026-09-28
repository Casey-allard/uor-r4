// Micro-benchmark: software shift-add multiply (copied verbatim in logic from
// crates/uor-r4-integer/src/math.rs checked_mul_unsigned/checked_mul) versus a
// native integer multiply, on the attention q.k shape (64-wide dot, 255 keys).
use std::hint::black_box;
use std::time::Instant;

#[derive(Debug)] enum E { Overflow }
fn shift_left_unsigned(value: u128, shift: u32) -> Result<u128, E> {
    if shift >= 128 || (shift > 0 && value >> (128 - shift) != 0) { return Err(E::Overflow); }
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
fn checked_mul(left: i128, right: i128) -> Result<i128, E> {
    let m = checked_mul_unsigned(left.unsigned_abs(), right.unsigned_abs())?;
    let neg = (left < 0) != (right < 0);
    Ok(if neg { -(m as i128) } else { m as i128 })
}
fn lcg(s: &mut u64) -> i32 { *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((*s >> 33) as i32) % 32767 }

fn main() {
    let keys = 255usize; let dim = 64usize; let reps = 2000usize;
    let mut s = 12345u64;
    let q: Vec<i32> = (0..dim).map(|_| lcg(&mut s)).collect();
    let k: Vec<Vec<i32>> = (0..keys).map(|_| (0..dim).map(|_| lcg(&mut s)).collect()).collect();
    // software multiply
    let t = Instant::now(); let mut acc_sw = 0i128;
    for _ in 0..reps { for key in &k { let mut dot = 0i128; for (&a,&b) in q.iter().zip(key) { dot += checked_mul(black_box(a as i128), black_box(b as i128)).unwrap(); } acc_sw = acc_sw.wrapping_add(dot); } }
    let sw = t.elapsed().as_secs_f64();
    // native i64 multiply
    let t = Instant::now(); let mut acc_hw = 0i128;
    for _ in 0..reps { for key in &k { let mut dot = 0i64; for (&a,&b) in q.iter().zip(key) { dot += black_box(a as i64) * black_box(b as i64); } acc_hw = acc_hw.wrapping_add(dot as i128); } }
    let hw = t.elapsed().as_secs_f64();
    assert_eq!(acc_sw, acc_hw);
    let ops = (reps*keys*dim) as f64;
    println!("q.k scores: {} products x {} reps", keys*dim, reps);
    println!("software shift-add checked_mul (i128): {:.3} s  => {:.2} ns/product", sw, sw*1e9/ops);
    println!("native i64 multiply-accumulate:        {:.3} s  => {:.2} ns/product", hw, hw*1e9/ops);
    println!("ratio software/native: {:.1}x", sw/hw);
}
