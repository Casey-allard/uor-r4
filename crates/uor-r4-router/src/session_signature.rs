//! Server-side quantization for the persistent session manifold.
//!
//! The graph runtime consumes only the resulting 288-bit value. Floating
//! point state construction stays on the serving side, outside the deployed
//! integer-only prediction contract.

pub const SESSION_SIGNATURE_BITS: usize = 288;
pub const SESSION_SIGNATURE_BYTES: usize = SESSION_SIGNATURE_BITS / 8;

/// Quantize a centered session-manifold state into the graph's 288-bit
/// signature space.
///
/// Each bit uses a deterministic signed three-coordinate projection. The
/// fixed index schedule is part of the signature definition: changing it is
/// a representation change and must be versioned with the graph artifacts.
pub fn from_state(state: &[f64]) -> [u8; SESSION_SIGNATURE_BYTES] {
    let mut signature = [0u8; SESSION_SIGNATURE_BYTES];
    if state.is_empty() {
        return signature;
    }

    let mean = state.iter().sum::<f64>() / state.len() as f64;
    for bit in 0..SESSION_SIGNATURE_BITS {
        let first = state[(bit * 37 + 11) % state.len()] - mean;
        let second = state[(bit * 97 + 7) % state.len()] - mean;
        let third = state[(bit * 193 + 3) % state.len()] - mean;
        let projection = first + 0.5 * second - 0.25 * third;
        if projection >= 0.0 {
            signature[bit >> 3] |= 1 << (bit & 7);
        }
    }
    signature
}

/// Deterministic fallback for clients that have token history but no
/// persistent f64 manifold. This preserves the same 288-bit lane and makes
/// the direct chat client session-sensitive; the HTTP server uses [`from_state`]
/// from its persistent manifold instead.
pub fn from_tokens(tokens: &[u32]) -> [u8; SESSION_SIGNATURE_BYTES] {
    let mut lanes = [0u64; 5];
    for (position, &token) in tokens.iter().enumerate() {
        let mut value =
            u64::from(token).wrapping_add((position as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        for (lane, slot) in lanes.iter_mut().enumerate() {
            value = splitmix64(value.wrapping_add(lane as u64));
            *slot = slot.rotate_left((lane * 11 + position % 17) as u32) ^ value;
        }
    }

    let mut signature = [0u8; SESSION_SIGNATURE_BYTES];
    for (index, byte) in signature.iter_mut().enumerate() {
        let lane = index % lanes.len();
        let shift = (index / lanes.len()) * 8;
        *byte = (lanes[lane] >> shift) as u8;
    }
    signature
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_projection_is_deterministic_and_sensitive() {
        let first: Vec<f64> = (0..512).map(|index| (index as f64 * 0.13).sin()).collect();
        let second: Vec<f64> = (0..512)
            .map(|index| (index as f64 * 0.17 + 1.3).cos())
            .collect();
        assert_eq!(from_state(&first), from_state(&first));
        assert_ne!(from_state(&first), from_state(&second));
    }

    #[test]
    fn token_history_projection_is_order_sensitive() {
        assert_ne!(from_tokens(&[1, 2, 3]), from_tokens(&[3, 2, 1]));
        assert_eq!(from_tokens(&[]), [0; SESSION_SIGNATURE_BYTES]);
    }
}
