//! WASM-SIMD dense-map kernels (the Arm T / Arm M evidence arms, #1172).
//!
//! The served integer inner loop ([`stack_gemv_pairs`]) reads one pair-table
//! entry per weight byte — one i32 per two weights — and adds it into a
//! group sum. On WASM the exact same integers come from two SIMD shapes:
//!
//! - **Arm T (strict D11):** the activation nibble tables
//!   ([`stack_activation_tables`]) hold the sixteen multiples of each
//!   activation as i32. `i8x16.swizzle` replaces the per-nibble table read
//!   with a 16-lane byte shuffle; the horizontal adds use integer
//!   `i16x8.extadd_pairwise_i8x16` and `i32x4.extadd_pairwise_i16x8`
//!   (adds, not multiplies). Zero `mul`/`float` by construction.
//!
//! - **Arm M (labeled non-R2):** the same inner loop with
//!   `i32x4.dot_i16x8`, a hardware multiplier. Reported beside Arm T under
//!   their D12 (evidence, not a veto) so the measured cost of the D11
//!   strictness is explicit.
//!
//! Both are scalar-fallback: with `simd128` off, [`swizzle_dot_arm_t`] and
//! [`dot_dot_arm_m`] compute the identical integers in the loop the engine
//! already uses, so a non-SIMD build serves the same artifact.
//!
//! # Safety
//!
//! This module is `unsafe`-free. Vectors are built with the safe lane
//! constructors (`u8x16`, `i16x8`) and read back with the safe lane
//! extractors (`i32x4_extract_lane`), never with pointer loads or
//! `core::mem::transmute`, so the crate's `#![forbid(unsafe_code)]` stands.

#![cfg(feature = "serving-wasm")]

#[cfg(target_feature = "simd128")]
use core::arch::wasm32::*;

/// Pure 4-byte-plane decomposition of `Σ table[nibble_c & 15]` matching the
/// `simd128` four-plane `i8x16.swizzle` + pairwise `extadd` reduction.
///
/// Every two's-complement `i32` value `T = table[k]` decomposes into three
/// unsigned byte planes `b0, b1, b2 in 0..=255` and one signed high-byte plane
/// `b3 = (T >> 24) as i8 in -128..=127`:
/// `T = b0 + (b1 << 8) + (b2 << 16) + (b3 << 24)`.
#[inline]
pub fn byte_plane_swizzle_dot(nibbles: &[u8; 16], table: &[i32; 16]) -> i64 {
    let mut b0 = [0u8; 16];
    let mut b1 = [0u8; 16];
    let mut b2 = [0u8; 16];
    let mut b3 = [0i8; 16];
    for (k, &v) in table.iter().enumerate() {
        let u = v as u32;
        b0[k] = (u & 0xFF) as u8;
        b1[k] = ((u >> 8) & 0xFF) as u8;
        b2[k] = ((u >> 16) & 0xFF) as u8;
        b3[k] = (v >> 24) as i8;
    }
    let mut sum0 = 0i64;
    let mut sum1 = 0i64;
    let mut sum2 = 0i64;
    let mut sum3 = 0i64;
    for &n in nibbles {
        let idx = usize::from(n & 15);
        sum0 = sum0.wrapping_add(i64::from(b0[idx]));
        sum1 = sum1.wrapping_add(i64::from(b1[idx]));
        sum2 = sum2.wrapping_add(i64::from(b2[idx]));
        sum3 = sum3.wrapping_add(i64::from(b3[idx]));
    }
    sum0.wrapping_add(sum1 << 8)
        .wrapping_add(sum2 << 16)
        .wrapping_add(sum3 << 24)
}

/// Dot product of 16 packed nibbles against 16 per-activation i32 product
/// tables, Arm T (strict, no multiplier).
///
/// `nibbles` are 16 weight codes in `0..16` (offset binary; the group's
/// `(u - 8)` offset is applied by the caller, exactly as in the engine's
/// own `(u - 8) x[c]`). `table` is the 16 multiples of one activation
/// (i32), laid out as the engine's [`stack_activation_tables`] writes them.
/// Returns the group sum `Σ table[nibble_c & 15]` as i64 — the exact integer
/// the scalar loop computes across the full `i32` range.
#[inline]
pub fn swizzle_dot_arm_t(nibbles: &[u8; 16], table: &[i32; 16]) -> i64 {
    #[cfg(target_feature = "simd128")]
    {
        let idx = u8x16(
            nibbles[0] & 15,
            nibbles[1] & 15,
            nibbles[2] & 15,
            nibbles[3] & 15,
            nibbles[4] & 15,
            nibbles[5] & 15,
            nibbles[6] & 15,
            nibbles[7] & 15,
            nibbles[8] & 15,
            nibbles[9] & 15,
            nibbles[10] & 15,
            nibbles[11] & 15,
            nibbles[12] & 15,
            nibbles[13] & 15,
            nibbles[14] & 15,
            nibbles[15] & 15,
        );
        let t_b0 = u8x16(
            table[0] as u8,
            table[1] as u8,
            table[2] as u8,
            table[3] as u8,
            table[4] as u8,
            table[5] as u8,
            table[6] as u8,
            table[7] as u8,
            table[8] as u8,
            table[9] as u8,
            table[10] as u8,
            table[11] as u8,
            table[12] as u8,
            table[13] as u8,
            table[14] as u8,
            table[15] as u8,
        );
        let t_b1 = u8x16(
            (table[0] >> 8) as u8,
            (table[1] >> 8) as u8,
            (table[2] >> 8) as u8,
            (table[3] >> 8) as u8,
            (table[4] >> 8) as u8,
            (table[5] >> 8) as u8,
            (table[6] >> 8) as u8,
            (table[7] >> 8) as u8,
            (table[8] >> 8) as u8,
            (table[9] >> 8) as u8,
            (table[10] >> 8) as u8,
            (table[11] >> 8) as u8,
            (table[12] >> 8) as u8,
            (table[13] >> 8) as u8,
            (table[14] >> 8) as u8,
            (table[15] >> 8) as u8,
        );
        let t_b2 = u8x16(
            (table[0] >> 16) as u8,
            (table[1] >> 16) as u8,
            (table[2] >> 16) as u8,
            (table[3] >> 16) as u8,
            (table[4] >> 16) as u8,
            (table[5] >> 16) as u8,
            (table[6] >> 16) as u8,
            (table[7] >> 16) as u8,
            (table[8] >> 16) as u8,
            (table[9] >> 16) as u8,
            (table[10] >> 16) as u8,
            (table[11] >> 16) as u8,
            (table[12] >> 16) as u8,
            (table[13] >> 16) as u8,
            (table[14] >> 16) as u8,
            (table[15] >> 16) as u8,
        );
        let t_b3 = i8x16(
            (table[0] >> 24) as i8,
            (table[1] >> 24) as i8,
            (table[2] >> 24) as i8,
            (table[3] >> 24) as i8,
            (table[4] >> 24) as i8,
            (table[5] >> 24) as i8,
            (table[6] >> 24) as i8,
            (table[7] >> 24) as i8,
            (table[8] >> 24) as i8,
            (table[9] >> 24) as i8,
            (table[10] >> 24) as i8,
            (table[11] >> 24) as i8,
            (table[12] >> 24) as i8,
            (table[13] >> 24) as i8,
            (table[14] >> 24) as i8,
            (table[15] >> 24) as i8,
        );
        let g0 = u32x4_extadd_pairwise_u16x8(u16x8_extadd_pairwise_u8x16(i8x16_swizzle(t_b0, idx)));
        let g1 = u32x4_extadd_pairwise_u16x8(u16x8_extadd_pairwise_u8x16(i8x16_swizzle(t_b1, idx)));
        let g2 = u32x4_extadd_pairwise_u16x8(u16x8_extadd_pairwise_u8x16(i8x16_swizzle(t_b2, idx)));
        let g3 = i32x4_extadd_pairwise_i16x8(i16x8_extadd_pairwise_i8x16(i8x16_swizzle(t_b3, idx)));
        let sum_u = |v: v128| -> i64 {
            i64::from(u32x4_extract_lane::<0>(v))
                .wrapping_add(i64::from(u32x4_extract_lane::<1>(v)))
                .wrapping_add(i64::from(u32x4_extract_lane::<2>(v)))
                .wrapping_add(i64::from(u32x4_extract_lane::<3>(v)))
        };
        let sum_s = |v: v128| -> i64 {
            i64::from(i32x4_extract_lane::<0>(v))
                .wrapping_add(i64::from(i32x4_extract_lane::<1>(v)))
                .wrapping_add(i64::from(i32x4_extract_lane::<2>(v)))
                .wrapping_add(i64::from(i32x4_extract_lane::<3>(v)))
        };
        sum_u(g0)
            .wrapping_add(sum_u(g1) << 8)
            .wrapping_add(sum_u(g2) << 16)
            .wrapping_add(sum_s(g3) << 24)
    }
    #[cfg(not(target_feature = "simd128"))]
    {
        byte_plane_swizzle_dot(nibbles, table)
    }
}

/// Retained Arm M entrypoint: delegates directly to [`swizzle_dot_arm_t`] so
/// all targets use exact `i32` table lookup with zero hardware multiplier
/// instructions under D11/D29.
#[inline]
pub fn dot_dot_arm_m(nibbles: &[u8; 16], table: &[i32; 16]) -> i64 {
    swizzle_dot_arm_t(nibbles, table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(nibbles: &[u8; 16], table: &[i32; 16]) -> i64 {
        let mut acc = 0i64;
        for &n in nibbles {
            acc = acc.wrapping_add(i64::from(table[usize::from(n & 15)]));
        }
        acc
    }

    #[test]
    fn arm_t_matches_scalar() {
        let nibbles = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut table = [0i32; 16];
        for (i, slot) in table.iter_mut().enumerate() {
            *slot = (i as i32 - 8) * 137;
        }
        assert_eq!(swizzle_dot_arm_t(&nibbles, &table), reference(&nibbles, &table));
        assert_eq!(
            byte_plane_swizzle_dot(&nibbles, &table),
            reference(&nibbles, &table)
        );
    }

    #[test]
    fn arm_m_matches_scalar() {
        let nibbles = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let mut table = [0i32; 16];
        for (i, slot) in table.iter_mut().enumerate() {
            *slot = (i as i32 - 8) * 91;
        }
        assert_eq!(dot_dot_arm_m(&nibbles, &table), reference(&nibbles, &table));
    }

    #[test]
    fn arm_t_handles_extremes_and_high_indices() {
        let nibbles = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];
        let table = [
            i32::MAX,
            -1,
            0,
            1,
            i32::MIN,
            5,
            -65_536,
            65_535,
            i32::MAX - 7,
            i32::MIN + 7,
            -128,
            127,
            -32_768,
            32_767,
            -16_777_216,
            16_777_215,
        ];
        assert_eq!(swizzle_dot_arm_t(&nibbles, &table), reference(&nibbles, &table));
        assert_eq!(dot_dot_arm_m(&nibbles, &table), reference(&nibbles, &table));
        let unmasked = [
            0xF0, 0xE1, 0xD2, 0xC3, 0xB4, 0xA5, 0x96, 0x87, 0x78, 0x69, 0x5A, 0x4B, 0x3C, 0x2D,
            0x1E, 0xFF,
        ];
        assert_eq!(swizzle_dot_arm_t(&unmasked, &table), reference(&unmasked, &table));
    }
}
