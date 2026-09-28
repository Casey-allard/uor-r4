use std::hint::black_box; use std::time::Instant;
fn xs(s: &mut u64) -> u64 { *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s }
fn main() {
    // quarter-square table for |u| <= 2^17 (operands are 16-bit signed): q[u] = floor(u*u/4), built offline
    let lim = 1usize << 17; let table: Vec<u64> = (0..=lim as u64).map(|u| (u * u) >> 2).collect();
    let qs = |a: i64, b: i64| -> i64 { table[(a + b).unsigned_abs() as usize] as i64 - table[(a - b).unsigned_abs() as usize] as i64 };
    let mut s = 7u64; let n = 97_217usize;
    let a: Vec<i64> = (0..n).map(|_| ((xs(&mut s) % 65535) as i64) - 32767).collect();   // Q16-quantized masses / codes
    let b: Vec<i64> = (0..n).map(|_| ((xs(&mut s) % 32769) as i64) - 16384).collect();   // Q14 values
    for i in 0..n { assert_eq!(qs(a[i], b[i]), a[i] * b[i]); }                              // exact for all integers
    let reps = 200; let t = Instant::now(); let mut acc = 0i64;
    for _ in 0..reps { for i in 0..n { acc = acc.wrapping_add(qs(black_box(a[i]), black_box(b[i]))); } }
    let dt = t.elapsed().as_secs_f64() / reps as f64; black_box(acc);
    println!("quarter-square LUT multiply (exact, no MUL): {:.2} ns/product; 97,217 products/token -> {:.3} ms/token; table {} KiB",
             dt * 1e9 / n as f64, dt * 1e3, (lim + 1) * 8 / 1024);
}
