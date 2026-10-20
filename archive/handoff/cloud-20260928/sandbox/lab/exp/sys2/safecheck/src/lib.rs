#![forbid(unsafe_code)]
//! Does a NEON TBL kernel compile under forbid(unsafe_code) on stable Rust?
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
fn tbl_add(table: core::arch::aarch64::uint8x16_t, idx: core::arch::aarch64::uint8x16_t, acc: core::arch::aarch64::int16x8_t) -> core::arch::aarch64::int16x8_t {
    use core::arch::aarch64::*;
    let looked = vqtbl1q_u8(table, idx);
    vaddq_s16(acc, vreinterpretq_s16_u8(vzip1q_u8(looked, vdupq_n_u8(0))))
}
#[cfg(target_arch = "aarch64")]
pub fn caller(t: [u8; 16], i: [u8; 16]) -> i16 {
    use core::arch::aarch64::*;
    // safe construction without raw-pointer loads?
    let tv: uint8x16_t = vld1q_u8_safe(&t);
    let iv: uint8x16_t = vld1q_u8_safe(&i);
    let r = tbl_add(tv, iv, vdupq_n_s16(0));
    vgetq_lane_s16::<0>(r)
}
#[cfg(target_arch = "aarch64")]
fn vld1q_u8_safe(a: &[u8; 16]) -> core::arch::aarch64::uint8x16_t {
    core::arch::aarch64::uint8x16_t::from(*a)
}
