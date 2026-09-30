//! B3: E8 Lattice Weight Codecs for Track B Conversion (SmolLM2 MLP layers).
//!
//! Provides geometric weight coding at 2, 3, and 4 bits per weight using canonical
//! E8 lattice codebook decoding helpers, an approximate heuristic encoder for the QuIP#
//! E8P codebook, incoherence processing via Randomized Hadamard Transform (RHT), and
//! scalar controls (RTN 4-bit, RTN 3-bit).
//!
//! Bits per weight are derived strictly from serialized bytes:
//! - Plain RTN 4-bit: 16 code bytes + 1 scale byte per 32 weights = 17 bytes / 32 = 4.2500 bpw.
//! - RHT + RTN 4-bit: 16 code bytes + 1 scale byte per 32 weights = 4.2500 bpw (+ 16 bytes seeds).
//! - RHT + RTN 3-bit: 3-bit scalar codes (12 bytes per 32 weights) + 1 scale byte = 3.2500 bpw (3.0000 bpw payload).
//! - RHT + E8P 2-bit: 2 index bytes per 8 weights + amortized fp16 row scale = 2.0000 bpw payload (~2.01 bpw total).
//! - RHT + E8P 3-bit: 2 E8P index bytes + 1 residual byte per 8 weights = 3.0000 bpw payload (~3.01 bpw total).
//! - RHT + E8P 4-bit: 2 E8P index bytes + 2 E8P residual bytes per 8 weights = 4.0000 bpw payload (~4.01 bpw total).

use crate::{invalid, Result};
pub use uor_r4_integer::codec::{E8Codebook, E8_DIM};

pub const E8_CODEWORDS: usize = 241;
pub const RTN_GROUP_SIZE: usize = 32;
pub const RHT_BLOCK_SIZE: usize = 32;

// ---------------------------------------------------------------------------
// 1. Conway–Sloane Fast E8 Lattice Decoding
// ---------------------------------------------------------------------------

/// Nearest D8 lattice point (integer coordinates with even sum).
///
/// Runs in O(8) arithmetic steps with 0 table lookups and 0 allocations.
#[inline]
pub fn d8_round(y: &[f32; E8_DIM]) -> [f32; E8_DIM] {
    let mut f = [0.0f32; E8_DIM];
    let mut sum_i = 0i32;
    for i in 0..E8_DIM {
        let r = y[i].round();
        f[i] = r;
        sum_i += r as i32;
    }
    if sum_i % 2 != 0 {
        let mut best_i = 0;
        let mut best_err = -1.0f32;
        for i in 0..E8_DIM {
            let err = (y[i] - f[i]).abs();
            if err > best_err {
                best_err = err;
                best_i = i;
            }
        }
        f[best_i] += if y[best_i] > f[best_i] { 1.0 } else { -1.0 };
    }
    f
}

/// Nearest E8 lattice point: E8 = D8 U (D8 + 1/2 * 1).
///
/// Decodes both cosets and selects the point with minimal Euclidean distance.
/// Guaranteed to produce a point in the E8 lattice.
#[inline]
pub fn e8_nearest(y: &[f32; E8_DIM]) -> [f32; E8_DIM] {
    let p_d8 = d8_round(y);
    let mut y_shift = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        y_shift[i] = y[i] - 0.5;
    }
    let p_d8_shift = d8_round(&y_shift);
    let mut p_half = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        p_half[i] = p_d8_shift[i] + 0.5;
    }
    let d_d8: f32 = (0..E8_DIM).map(|i| (y[i] - p_d8[i]).powi(2)).sum();
    let d_half: f32 = (0..E8_DIM).map(|i| (y[i] - p_half[i]).powi(2)).sum();
    if d_d8 <= d_half {
        p_d8
    } else {
        p_half
    }
}

/// QuIP# E8P 16-bit codebook packed absolute grid (256 x 32-bit entries = 1 KiB).
/// Derived from D8+1/2 points with norm^2 <= 10 and 29 norm-12 vectors.
pub const E8P_PACKED_ABS: [u32; 256] = [
    0x99999999, 0x59999999, 0xd9999999, 0x7999b999, 0xb999b999, 0x3999b999, 0x9999d999, 0x5999d999,
    0x7b999999, 0xbb999999, 0x3b999999, 0x9b99b999, 0x5b99b999, 0x7b99d999, 0x9d999999, 0x5d999999,
    0x7d99b999, 0x79999b99, 0xb9999b99, 0x39999b99, 0x9999bb99, 0x5999bb99, 0x7999db99, 0x9b999b99,
    0x5b999b99, 0x7b99bb99, 0xbb99bb99, 0x7d999b99, 0x99999d99, 0x59999d99, 0x7999bd99, 0x7b999d99,
    0x79b99999, 0xb9b99999, 0x39b99999, 0x99b9b999, 0x59b9b999, 0x79b9d999, 0x9bb99999, 0x5bb99999,
    0x7bb9b999, 0xbbb9b999, 0x7db99999, 0x99b99b99, 0x59b99b99, 0x79b9bb99, 0xb9b9bb99, 0x7bb99b99,
    0xbbb99b99, 0x9bb9bb99, 0x79b99d99, 0x99d99999, 0x59d99999, 0x79d9b999, 0x7bd99999, 0x79d99b99,
    0x799999b9, 0xb99999b9, 0x399999b9, 0x9999b9b9, 0x5999b9b9, 0x7999d9b9, 0x9b9999b9, 0x5b9999b9,
    0x7b99b9b9, 0xbb99b9b9, 0x7d9999b9, 0x99999bb9, 0x59999bb9, 0x7999bbb9, 0xb999bbb9, 0x7b999bb9,
    0xbb999bb9, 0x9b99bbb9, 0x79999db9, 0x99b999b9, 0x59b999b9, 0x79b9b9b9, 0xb9b9b9b9, 0x7bb999b9,
    0xbbb999b9, 0x9bb9b9b9, 0x79b99bb9, 0xb9b99bb9, 0x99b9bbb9, 0x9bb99bb9, 0x79d999b9, 0x999999d9,
    0x599999d9, 0x7999b9d9, 0x7b9999d9, 0x79999bd9, 0x79b999d9, 0x799b9999, 0xb99b9999, 0x399b9999,
    0x999bb999, 0x599bb999, 0x799bd999, 0x9b9b9999, 0x5b9b9999, 0x7b9bb999, 0xbb9bb999, 0x7d9b9999,
    0x999b9b99, 0x599b9b99, 0x799bbb99, 0xb99bbb99, 0x7b9b9b99, 0xbb9b9b99, 0x9b9bbb99, 0x799b9d99,
    0x99bb9999, 0x59bb9999, 0x79bbb999, 0xb9bbb999, 0x7bbb9999, 0xbbbb9999, 0x9bbbb999, 0x79bb9b99,
    0xb9bb9b99, 0x99bbbb99, 0x9bbb9b99, 0x79db9999, 0x999b99b9, 0x599b99b9, 0x799bb9b9, 0xb99bb9b9,
    0x7b9b99b9, 0xbb9b99b9, 0x9b9bb9b9, 0x799b9bb9, 0xb99b9bb9, 0x999bbbb9, 0x9b9b9bb9, 0x79bb99b9,
    0xb9bb99b9, 0x99bbb9b9, 0x9bbb99b9, 0x99bb9bb9, 0x799b99d9, 0x999d9999, 0x599d9999, 0x799db999,
    0x7b9d9999, 0x799d9b99, 0x79bd9999, 0x799d99b9, 0x7999999b, 0xb999999b, 0x3999999b, 0x9999b99b,
    0x5999b99b, 0x7999d99b, 0x9b99999b, 0x5b99999b, 0x7b99b99b, 0xbb99b99b, 0x7d99999b, 0x99999b9b,
    0x59999b9b, 0x7999bb9b, 0xb999bb9b, 0x7b999b9b, 0xbb999b9b, 0x9b99bb9b, 0x79999d9b, 0x99b9999b,
    0x59b9999b, 0x79b9b99b, 0xb9b9b99b, 0x7bb9999b, 0xbbb9999b, 0x9bb9b99b, 0x79b99b9b, 0xb9b99b9b,
    0x99b9bb9b, 0x9bb99b9b, 0x79d9999b, 0x999999bb, 0x599999bb, 0x7999b9bb, 0xb999b9bb, 0x7b9999bb,
    0xbb9999bb, 0x9b99b9bb, 0x79999bbb, 0xb9999bbb, 0x9999bbbb, 0x9b999bbb, 0x79b999bb, 0xb9b999bb,
    0x99b9b9bb, 0x9bb999bb, 0x99b99bbb, 0x799999db, 0x999b999b, 0x599b999b, 0x799bb99b, 0xb99bb99b,
    0x7b9b999b, 0xbb9b999b, 0x9b9bb99b, 0x799b9b9b, 0xb99b9b9b, 0x999bbb9b, 0x9b9b9b9b, 0x79bb999b,
    0xb9bb999b, 0x99bbb99b, 0x9bbb999b, 0x99bb9b9b, 0x799b99bb, 0xb99b99bb, 0x999bb9bb, 0x9b9b99bb,
    0x999b9bbb, 0x99bb99bb, 0x799d999b, 0x9999999d, 0x5999999d, 0x7999b99d, 0x7b99999d, 0x79999b9d,
    0x79b9999d, 0x799999bd, 0x799b999d, 0x5b99bb9b, 0x5b9bbb99, 0x5b99bbb9, 0x5bb9bb99, 0x7b9b9bbb,
    0x799bbbbb, 0x7b9bb9bb, 0x599b9bbb, 0x5b9b99bb, 0x599bb9bb, 0x7bbb9b9b, 0x79bbbb9b, 0x7bbbb99b,
    0x59bb9b9b, 0x5bbb999b, 0x59bbb99b, 0x7bb99bbb, 0x79b9bbbb, 0x7bb9b9bb, 0x59b99bbb, 0x5bb999bb,
    0x59bbb9b9, 0x7bbb9bb9, 0x79bbbbb9, 0x7bbbb9b9, 0x59bb9bb9, 0x5bbb99b9, 0x5bb9b9b9, 0x7b9bbb9b,
];

/// Decompress 16-bit E8P codeword into 8-dimensional lattice vector in E8 +- 1/4.
#[inline]
pub fn decode_e8p_codeword(c: u16) -> [f32; E8_DIM] {
    let raw_signs = (c & 0xFF) as u8;
    let abs_idx = (c >> 8) as usize;
    let parity = (raw_signs.count_ones() % 2) as u8;
    let signs = raw_signs ^ parity;
    let abs_code = E8P_PACKED_ABS[abs_idx];
    let shuffle_map = [0, 4, 1, 5, 2, 6, 3, 7];
    let mut out = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        let ii = shuffle_map[i];
        let nibble = ((abs_code >> (4 * ii)) & 0x0F) as i32;
        let mut val = (nibble - 8) as f32 * 0.5;
        if ((signs >> ii) & 1) != 0 {
            val = -val;
        }
        if parity != 0 {
            val -= 0.25;
        } else {
            val += 0.25;
        }
        out[i] = val;
    }
    out
}

/// Reference finite oracle that performs an exhaustive enumeration under declared f32 arithmetic
/// over all 65,536 E8P codewords to find the nearest codeword.
/// Finds the nearest E8P codeword by exhaustive evaluation over all 65,536 candidates.
/// O(65,536) search; intended for verification, small-scale diagnostics, and test regressions.
/// Scale must be finite and positive (`scale > 1e-9`).
/// Rejects non-finite inputs, normalization overflow, non-finite squared distances (where all candidates overflow),
/// and non-finite reconstruction.
pub fn oracle_nearest_e8p_codeword(y: &[f32; E8_DIM], scale: f32) -> Result<([f32; E8_DIM], u16)> {
    if !scale.is_finite() || scale <= 1e-9 {
        return Err(invalid(
            "oracle scale must be finite and positive (scale > 1e-9)",
        ));
    }
    for (i, &val) in y.iter().enumerate() {
        if !val.is_finite() {
            return Err(invalid(format!("oracle input y[{i}] is not finite")));
        }
    }
    let mut norm_y = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        let ny = y[i] / scale;
        if !ny.is_finite() {
            return Err(invalid(format!(
                "normalized input y[{i}] / scale is not finite"
            )));
        }
        norm_y[i] = ny;
    }
    let mut best_err = f32::INFINITY;
    let mut best_c = None;
    for c in 0..=u16::MAX {
        let dec = decode_e8p_codeword(c);
        let mut err = 0.0f32;
        for i in 0..E8_DIM {
            let diff = norm_y[i] - dec[i];
            err += diff * diff;
        }
        if err < best_err {
            best_err = err;
            best_c = Some(c);
            if err == 0.0 {
                break;
            }
        }
    }
    let best_c = best_c.ok_or_else(|| {
        invalid("oracle search failed to find a candidate with finite squared distance")
    })?;
    if !best_err.is_finite() {
        return Err(invalid(
            "oracle search produced non-finite minimum squared distance",
        ));
    }
    let best_dec = decode_e8p_codeword(best_c);
    let mut out = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        let o = best_dec[i] * scale;
        if !o.is_finite() {
            return Err(invalid(format!(
                "reconstructed oracle output[{i}] is not finite"
            )));
        }
        out[i] = o;
    }
    Ok((out, best_c))
}

/// Quantizes an 8-vector to the QuIP# E8P codebook using a fast heuristic sign/abs search.
/// Returns (quantized_vector, code_u16).
///
/// NOTE (Scoped Implementation Limitation): This heuristic fixes sign bits per parity
/// based on the sign of `target[i] = norm_y[i] - shift`, which evaluates at most two distinct sign patterns
/// (evaluating at most 512 candidate codewords out of 65,536). Consequently,
/// it is NOT guaranteed to find the true nearest codeword. For example, codeword 256 is
/// encoded as codeword 128 with squared error 4.0 (see `test_e8p_encoder_counterexample_and_oracle`).
/// For nearest-codeword search under exhaustive enumeration, see [`oracle_nearest_e8p_codeword`].
pub fn quantize_e8p_block(y: &[f32; E8_DIM], scale: f32) -> ([f32; E8_DIM], u16) {
    if scale <= 1e-9 {
        return ([0.0; E8_DIM], 0);
    }
    let mut norm_y = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        norm_y[i] = y[i] / scale;
    }

    let shuffle_map = [0, 4, 1, 5, 2, 6, 3, 7];
    let mut best_err = f32::INFINITY;
    let mut best_c = 0u16;
    let mut best_out = [0.0f32; E8_DIM];

    // Evaluate both parity cosets (+- 0.25 shift on E8)
    for parity in 0..=1u8 {
        let shift = if parity == 0 { 0.25f32 } else { -0.25f32 };
        let mut target = [0.0f32; E8_DIM];
        let mut signs = 0u8;
        for i in 0..E8_DIM {
            target[i] = norm_y[i] - shift;
            let ii = shuffle_map[i];
            if target[i] < 0.0 {
                signs |= 1 << ii;
            }
        }
        let ones = (signs.count_ones() % 2) as u8;
        let raw_signs = if ones != parity {
            signs ^ parity
        } else {
            signs
        };

        // Test the 256 candidate magnitude patterns
        for abs_idx in 0..256 {
            let c = ((abs_idx as u16) << 8) | (raw_signs as u16);
            let dec = decode_e8p_codeword(c);
            let mut err = 0.0f32;
            for i in 0..E8_DIM {
                let diff = norm_y[i] - dec[i];
                err += diff * diff;
            }
            if err < best_err {
                best_err = err;
                best_c = c;
                best_out = dec;
            }
        }
    }

    let mut out = [0.0f32; E8_DIM];
    for i in 0..E8_DIM {
        out[i] = best_out[i] * scale;
    }
    (out, best_c)
}

// ---------------------------------------------------------------------------
// 2. Randomized Hadamard Transform (RHT)
// ---------------------------------------------------------------------------

/// Fast Walsh–Hadamard Transform of size 32.
/// Orthogonal and self-inverse: FWHT32(FWHT32(x)) == x bit-for-bit.
#[inline]
pub fn fwht32(chunk: &mut [f32; RHT_BLOCK_SIZE]) {
    let mut h = 1;
    while h < RHT_BLOCK_SIZE {
        for i in (0..RHT_BLOCK_SIZE).step_by(h * 2) {
            for j in i..i + h {
                let x = chunk[j];
                let y = chunk[j + h];
                chunk[j] = x + y;
                chunk[j + h] = x - y;
            }
        }
        h *= 2;
    }
    let norm = 1.0 / (RHT_BLOCK_SIZE as f32).sqrt();
    for v in chunk.iter_mut() {
        *v *= norm;
    }
}

/// Simple deterministic xorshift64 PRNG for sign generation.
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// Generate deterministic +/- 1 Rademacher signs for length `n`.
pub fn generate_signs(n: usize, seed: u64) -> Vec<f32> {
    let mut signs = Vec::with_capacity(n);
    let mut state = seed ^ 0x9e3779b97f4a7c15;
    if state == 0 {
        state = 0x853c49e6748fea9b;
    }
    for _ in 0..n {
        state = xorshift64(state);
        signs.push(if (state & 1) == 1 { 1.0 } else { -1.0 });
    }
    signs
}

/// Forward Randomized Hadamard Transform for a matrix of shape [rows, cols].
/// Requires rows and cols to be multiples of RHT_BLOCK_SIZE (32).
pub fn rht_transform_matrix(
    weights: &[f32],
    rows: usize,
    cols: usize,
    seed_left: u64,
    seed_right: u64,
) -> Result<Vec<f32>> {
    if weights.len() != rows * cols {
        return Err(invalid(format!("weights length != {rows}x{cols}")));
    }
    if rows % RHT_BLOCK_SIZE != 0 || cols % RHT_BLOCK_SIZE != 0 {
        return Err(invalid(format!(
            "matrix dimensions ({rows}, {cols}) must be multiples of {RHT_BLOCK_SIZE}"
        )));
    }

    let mut out = weights.to_vec();
    let left_signs = generate_signs(rows, seed_left);
    let right_signs = generate_signs(cols, seed_right);

    // 1. Left transform: column-by-column
    // Multiply by left signs then apply FWHT32 on column chunks
    for c in 0..cols {
        for r in 0..rows {
            out[r * cols + c] *= left_signs[r];
        }
        for chunk_idx in 0..(rows / RHT_BLOCK_SIZE) {
            let offset = chunk_idx * RHT_BLOCK_SIZE;
            let mut chunk = [0.0f32; RHT_BLOCK_SIZE];
            for k in 0..RHT_BLOCK_SIZE {
                chunk[k] = out[(offset + k) * cols + c];
            }
            fwht32(&mut chunk);
            for k in 0..RHT_BLOCK_SIZE {
                out[(offset + k) * cols + c] = chunk[k];
            }
        }
    }

    // 2. Right transform: row-by-row
    // Multiply by right signs then apply FWHT32 on row chunks
    for r in 0..rows {
        let row_offset = r * cols;
        for c in 0..cols {
            out[row_offset + c] *= right_signs[c];
        }
        for chunk_idx in 0..(cols / RHT_BLOCK_SIZE) {
            let offset = row_offset + chunk_idx * RHT_BLOCK_SIZE;
            let mut chunk = [0.0f32; RHT_BLOCK_SIZE];
            chunk.copy_from_slice(&out[offset..offset + RHT_BLOCK_SIZE]);
            fwht32(&mut chunk);
            out[offset..offset + RHT_BLOCK_SIZE].copy_from_slice(&chunk);
        }
    }

    Ok(out)
}

/// Inverse Randomized Hadamard Transform for a matrix of shape [rows, cols].
pub fn rht_inverse_matrix(
    weights: &[f32],
    rows: usize,
    cols: usize,
    seed_left: u64,
    seed_right: u64,
) -> Result<Vec<f32>> {
    if weights.len() != rows * cols {
        return Err(invalid(format!("weights length != {rows}x{cols}")));
    }
    if rows % RHT_BLOCK_SIZE != 0 || cols % RHT_BLOCK_SIZE != 0 {
        return Err(invalid(format!(
            "matrix dimensions ({rows}, {cols}) must be multiples of {RHT_BLOCK_SIZE}"
        )));
    }

    let mut out = weights.to_vec();
    let left_signs = generate_signs(rows, seed_left);
    let right_signs = generate_signs(cols, seed_right);

    // 1. Inverse right transform: FWHT32 on row chunks then multiply by right signs
    for r in 0..rows {
        let row_offset = r * cols;
        for chunk_idx in 0..(cols / RHT_BLOCK_SIZE) {
            let offset = row_offset + chunk_idx * RHT_BLOCK_SIZE;
            let mut chunk = [0.0f32; RHT_BLOCK_SIZE];
            chunk.copy_from_slice(&out[offset..offset + RHT_BLOCK_SIZE]);
            fwht32(&mut chunk);
            out[offset..offset + RHT_BLOCK_SIZE].copy_from_slice(&chunk);
        }
        for c in 0..cols {
            out[row_offset + c] *= right_signs[c];
        }
    }

    // 2. Inverse left transform: FWHT32 on column chunks then multiply by left signs
    for c in 0..cols {
        for chunk_idx in 0..(rows / RHT_BLOCK_SIZE) {
            let offset = chunk_idx * RHT_BLOCK_SIZE;
            let mut chunk = [0.0f32; RHT_BLOCK_SIZE];
            for k in 0..RHT_BLOCK_SIZE {
                chunk[k] = out[(offset + k) * cols + c];
            }
            fwht32(&mut chunk);
            for k in 0..RHT_BLOCK_SIZE {
                out[(offset + k) * cols + c] = chunk[k];
            }
        }
        for r in 0..rows {
            out[r * cols + c] *= left_signs[r];
        }
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// 3. Scale and Quantization Primitives
// ---------------------------------------------------------------------------

pub fn encode_scale_fp16(scale: f32) -> u16 {
    half::f16::from_f32(scale).to_bits()
}

pub fn decode_scale_fp16(bits: u16) -> f32 {
    half::f16::from_bits(bits).to_f32()
}

// ---------------------------------------------------------------------------
// 4. Plain RTN 4-bit Codec (Group size 32, 4.2500 bpw payload)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Rtn4BitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub scales: Vec<u8>,
    pub max_scales: Vec<f32>,
    pub packed_codes: Vec<u8>,
}

impl Rtn4BitMatrix {
    pub fn encode(weights: &[f32], rows: usize, cols: usize) -> Result<Self> {
        let total = rows * cols;
        if weights.len() != total {
            return Err(invalid(format!("weights length != {rows}x{cols}")));
        }
        let num_groups = total / RTN_GROUP_SIZE;
        let mut scales = Vec::with_capacity(num_groups);
        let mut max_scales = Vec::with_capacity(rows);
        let mut packed_codes = Vec::with_capacity(total / 2);

        for r in 0..rows {
            let row_offset = r * cols;
            let row_weights = &weights[row_offset..row_offset + cols];
            let row_max = row_weights.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
            max_scales.push(row_max);

            for g in 0..(cols / RTN_GROUP_SIZE) {
                let g_offset = row_offset + g * RTN_GROUP_SIZE;
                let grp = &weights[g_offset..g_offset + RTN_GROUP_SIZE];
                let grp_max = grp.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
                let scale_u8 = if row_max > 0.0 {
                    ((grp_max / row_max).clamp(0.0, 1.0) * 255.0).round() as u8
                } else {
                    0
                };
                scales.push(scale_u8);

                let eff_scale = (scale_u8 as f32 / 255.0) * row_max;
                let step = if eff_scale > 0.0 {
                    eff_scale / 7.0
                } else {
                    1.0
                };

                for pair in 0..(RTN_GROUP_SIZE / 2) {
                    let w0 = grp[pair * 2];
                    let w1 = grp[pair * 2 + 1];
                    let q0 = ((w0 / step).round().clamp(-7.0, 7.0) as i8 + 8) as u8;
                    let q1 = ((w1 / step).round().clamp(-7.0, 7.0) as i8 + 8) as u8;
                    packed_codes.push((q0 & 0x0F) | ((q1 & 0x0F) << 4));
                }
            }
        }

        Ok(Self {
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut out = vec![0.0f32; total];
        let groups_per_row = self.cols / RTN_GROUP_SIZE;

        for r in 0..self.rows {
            let row_max = self.max_scales[r];
            for g in 0..groups_per_row {
                let g_idx = r * groups_per_row + g;
                let scale_u8 = self.scales[g_idx];
                let eff_scale = (scale_u8 as f32 / 255.0) * row_max;
                let step = if eff_scale > 0.0 {
                    eff_scale / 7.0
                } else {
                    1.0
                };

                let grp_out_offset = r * self.cols + g * RTN_GROUP_SIZE;
                let code_offset = g_idx * (RTN_GROUP_SIZE / 2);

                for pair in 0..(RTN_GROUP_SIZE / 2) {
                    let byte = self.packed_codes[code_offset + pair];
                    let q0 = (byte & 0x0F) as i8 - 8;
                    let q1 = ((byte >> 4) & 0x0F) as i8 - 8;
                    out[grp_out_offset + pair * 2] = (q0 as f32) * step;
                    out[grp_out_offset + pair * 2 + 1] = (q1 as f32) * step;
                }
            }
        }
        Ok(out)
    }

    pub fn serialized_bytes(&self) -> usize {
        16 + self.scales.len() + self.max_scales.len() * 4 + self.packed_codes.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        4.2500
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.rows * self.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&(self.rows as u64).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u64).to_le_bytes());
        buf.extend_from_slice(&(self.scales.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.scales);
        buf.extend_from_slice(&(self.max_scales.len() as u64).to_le_bytes());
        for &s in &self.max_scales {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(self.packed_codes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.packed_codes);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let rows = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let cols = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let num_scales = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let scales = bytes[cursor..cursor + num_scales].to_vec();
        cursor += num_scales;
        let num_max_scales =
            u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut max_scales = Vec::with_capacity(num_max_scales);
        for _ in 0..num_max_scales {
            let s = f32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            max_scales.push(s);
            cursor += 4;
        }
        let num_codes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let packed_codes = bytes[cursor..cursor + num_codes].to_vec();
        Ok(Self {
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }
}

// ---------------------------------------------------------------------------
// 5. RHT + RTN 4-bit Codec
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct RhtRtn4BitMatrix {
    pub seed_left: u64,
    pub seed_right: u64,
    pub inner: Rtn4BitMatrix,
}

impl RhtRtn4BitMatrix {
    pub fn encode(
        weights: &[f32],
        rows: usize,
        cols: usize,
        seed_left: u64,
        seed_right: u64,
    ) -> Result<Self> {
        let rot = rht_transform_matrix(weights, rows, cols, seed_left, seed_right)?;
        let inner = Rtn4BitMatrix::encode(&rot, rows, cols)?;
        Ok(Self {
            seed_left,
            seed_right,
            inner,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let rot_deq = self.inner.dequantize()?;
        rht_inverse_matrix(
            &rot_deq,
            self.inner.rows,
            self.inner.cols,
            self.seed_left,
            self.seed_right,
        )
    }

    pub fn serialized_bytes(&self) -> usize {
        16 + self.inner.serialized_bytes()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        4.2500
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.inner.rows * self.inner.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&self.seed_left.to_le_bytes());
        buf.extend_from_slice(&self.seed_right.to_le_bytes());
        buf.extend_from_slice(&self.inner.to_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let seed_left = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let seed_right = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let inner = Rtn4BitMatrix::from_bytes(&bytes[16..])?;
        Ok(Self {
            seed_left,
            seed_right,
            inner,
        })
    }
}

// ---------------------------------------------------------------------------
// 6. RHT + RTN 3-bit Codec (Group size 32, 3.0000 bpw payload)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct RhtRtn3BitMatrix {
    pub seed_left: u64,
    pub seed_right: u64,
    pub rows: usize,
    pub cols: usize,
    pub scales: Vec<u8>,
    pub max_scales: Vec<f32>,
    /// 3 bits per weight packed: 8 weights take 3 bytes = 24 bits.
    pub packed_codes: Vec<u8>,
}

impl RhtRtn3BitMatrix {
    pub fn encode(
        weights: &[f32],
        rows: usize,
        cols: usize,
        seed_left: u64,
        seed_right: u64,
    ) -> Result<Self> {
        let rot = rht_transform_matrix(weights, rows, cols, seed_left, seed_right)?;
        let total = rows * cols;
        let num_groups = total / RTN_GROUP_SIZE;
        let mut scales = Vec::with_capacity(num_groups);
        let mut max_scales = Vec::with_capacity(rows);
        let mut packed_codes = Vec::with_capacity(total * 3 / 8);

        for r in 0..rows {
            let row_offset = r * cols;
            let row_weights = &rot[row_offset..row_offset + cols];
            let row_max = row_weights.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
            max_scales.push(row_max);

            for g in 0..(cols / RTN_GROUP_SIZE) {
                let g_offset = row_offset + g * RTN_GROUP_SIZE;
                let grp = &rot[g_offset..g_offset + RTN_GROUP_SIZE];
                let grp_max = grp.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
                let scale_u8 = if row_max > 0.0 {
                    ((grp_max / row_max).clamp(0.0, 1.0) * 255.0).round() as u8
                } else {
                    0
                };
                scales.push(scale_u8);

                let eff_scale = (scale_u8 as f32 / 255.0) * row_max;
                let step = if eff_scale > 0.0 {
                    eff_scale / 3.5
                } else {
                    1.0
                };

                // Pack 8 weights into 3 bytes
                for block in 0..(RTN_GROUP_SIZE / 8) {
                    let b_offset = block * 8;
                    let mut q = [0u8; 8];
                    for i in 0..8 {
                        let w = grp[b_offset + i];
                        let val = ((w / step).round().clamp(-3.0, 3.0) as i8 + 3) as u8;
                        q[i] = val & 0x07;
                    }
                    // Byte 0: q[0] (3), q[1] (3), q[2][0..2] (2)
                    let b0 = q[0] | (q[1] << 3) | ((q[2] & 0x03) << 6);
                    // Byte 1: q[2][2] (1), q[3] (3), q[4] (3), q[5][0] (1)
                    let b1 =
                        ((q[2] >> 2) & 0x01) | (q[3] << 1) | (q[4] << 4) | ((q[5] & 0x01) << 7);
                    // Byte 2: q[5][1..2] (2), q[6] (3), q[7] (3)
                    let b2 = ((q[5] >> 1) & 0x03) | (q[6] << 2) | (q[7] << 5);
                    packed_codes.push(b0);
                    packed_codes.push(b1);
                    packed_codes.push(b2);
                }
            }
        }

        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut rot_out = vec![0.0f32; total];
        let groups_per_row = self.cols / RTN_GROUP_SIZE;

        for r in 0..self.rows {
            let row_max = self.max_scales[r];
            for g in 0..groups_per_row {
                let g_idx = r * groups_per_row + g;
                let scale_u8 = self.scales[g_idx];
                let eff_scale = (scale_u8 as f32 / 255.0) * row_max;
                let step = if eff_scale > 0.0 {
                    eff_scale / 3.5
                } else {
                    1.0
                };

                let grp_out_offset = r * self.cols + g * RTN_GROUP_SIZE;
                let code_offset = g_idx * (RTN_GROUP_SIZE * 3 / 8);

                for block in 0..(RTN_GROUP_SIZE / 8) {
                    let c_idx = code_offset + block * 3;
                    let b0 = self.packed_codes[c_idx];
                    let b1 = self.packed_codes[c_idx + 1];
                    let b2 = self.packed_codes[c_idx + 2];

                    let mut q = [0i8; 8];
                    q[0] = (b0 & 0x07) as i8 - 3;
                    q[1] = ((b0 >> 3) & 0x07) as i8 - 3;
                    q[2] = (((b0 >> 6) & 0x03) | ((b1 & 0x01) << 2)) as i8 - 3;
                    q[3] = ((b1 >> 1) & 0x07) as i8 - 3;
                    q[4] = ((b1 >> 4) & 0x07) as i8 - 3;
                    q[5] = (((b1 >> 7) & 0x01) | ((b2 & 0x03) << 1)) as i8 - 3;
                    q[6] = ((b2 >> 2) & 0x07) as i8 - 3;
                    q[7] = ((b2 >> 5) & 0x07) as i8 - 3;

                    let b_out_offset = grp_out_offset + block * 8;
                    for i in 0..8 {
                        rot_out[b_out_offset + i] = (q[i] as f32) * step;
                    }
                }
            }
        }

        rht_inverse_matrix(
            &rot_out,
            self.rows,
            self.cols,
            self.seed_left,
            self.seed_right,
        )
    }

    pub fn serialized_bytes(&self) -> usize {
        32 + self.scales.len() + self.max_scales.len() * 4 + self.packed_codes.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        3.0000
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.rows * self.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&self.seed_left.to_le_bytes());
        buf.extend_from_slice(&self.seed_right.to_le_bytes());
        buf.extend_from_slice(&(self.rows as u64).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u64).to_le_bytes());
        buf.extend_from_slice(&(self.scales.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.scales);
        buf.extend_from_slice(&(self.max_scales.len() as u64).to_le_bytes());
        for &s in &self.max_scales {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(self.packed_codes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.packed_codes);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let seed_left = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let seed_right = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let rows = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let cols = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let num_scales = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let scales = bytes[cursor..cursor + num_scales].to_vec();
        cursor += num_scales;
        let num_max_scales =
            u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut max_scales = Vec::with_capacity(num_max_scales);
        for _ in 0..num_max_scales {
            let s = f32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            max_scales.push(s);
            cursor += 4;
        }
        let num_codes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let packed_codes = bytes[cursor..cursor + num_codes].to_vec();
        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            scales,
            max_scales,
            packed_codes,
        })
    }
}

// ---------------------------------------------------------------------------
// 7. RHT + E8P 2-bit Codec (2.0000 bpw payload)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct RhtE8P2BitMatrix {
    pub seed_left: u64,
    pub seed_right: u64,
    pub rows: usize,
    pub cols: usize,
    /// Amortized fp16 row scales: 1 scale per row (2 bytes = 16 bits per row).
    pub row_scales: Vec<u16>,
    /// 16-bit E8P code per 8 weights = 2.0000 bpw.
    pub e8p_codes: Vec<u16>,
}

impl RhtE8P2BitMatrix {
    pub fn encode(
        weights: &[f32],
        rows: usize,
        cols: usize,
        seed_left: u64,
        seed_right: u64,
    ) -> Result<Self> {
        let rot = rht_transform_matrix(weights, rows, cols, seed_left, seed_right)?;
        let num_blocks = (rows * cols) / E8_DIM;
        let mut row_scales = Vec::with_capacity(rows);
        let mut e8p_codes = Vec::with_capacity(num_blocks);

        for r in 0..rows {
            let row_offset = r * cols;
            let row = &rot[row_offset..row_offset + cols];
            // RMS of row as base scale
            let sum_sq: f32 = row.iter().map(|&v| v * v).sum();
            let rms = (sum_sq / cols as f32).sqrt().max(1e-9);
            // E8 root scale
            let scale = rms * 0.7071; // sqrt(0.5) lattice normalization
            row_scales.push(encode_scale_fp16(scale));

            for b in 0..(cols / E8_DIM) {
                let mut block = [0.0f32; E8_DIM];
                block.copy_from_slice(&row[b * E8_DIM..(b + 1) * E8_DIM]);
                let (_q, code) = quantize_e8p_block(&block, scale);
                e8p_codes.push(code);
            }
        }

        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut rot_out = vec![0.0f32; total];
        let blocks_per_row = self.cols / E8_DIM;

        for r in 0..self.rows {
            let scale = decode_scale_fp16(self.row_scales[r]);
            for b in 0..blocks_per_row {
                let code = self.e8p_codes[r * blocks_per_row + b];
                let block = decode_e8p_codeword(code);
                let offset = r * self.cols + b * E8_DIM;
                for i in 0..E8_DIM {
                    rot_out[offset + i] = block[i] * scale;
                }
            }
        }

        rht_inverse_matrix(
            &rot_out,
            self.rows,
            self.cols,
            self.seed_left,
            self.seed_right,
        )
    }

    pub fn serialized_bytes(&self) -> usize {
        32 + self.row_scales.len() * 2 + self.e8p_codes.len() * 2
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        2.0000
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.rows * self.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&self.seed_left.to_le_bytes());
        buf.extend_from_slice(&self.seed_right.to_le_bytes());
        buf.extend_from_slice(&(self.rows as u64).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u64).to_le_bytes());
        buf.extend_from_slice(&(self.row_scales.len() as u64).to_le_bytes());
        for &s in &self.row_scales {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(self.e8p_codes.len() as u64).to_le_bytes());
        for &c in &self.e8p_codes {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let seed_left = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let seed_right = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let rows = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let cols = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let num_scales = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut row_scales = Vec::with_capacity(num_scales);
        for _ in 0..num_scales {
            let s = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            row_scales.push(s);
            cursor += 2;
        }
        let num_codes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut e8p_codes = Vec::with_capacity(num_codes);
        for _ in 0..num_codes {
            let c = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            e8p_codes.push(c);
            cursor += 2;
        }
        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes,
        })
    }
}

// ---------------------------------------------------------------------------
// 8. RHT + E8P 3-bit Codec (E8P 2-bit + 1-bpw residual stage = 3.0000 bpw payload)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct RhtE8P3BitMatrix {
    pub seed_left: u64,
    pub seed_right: u64,
    pub rows: usize,
    pub cols: usize,
    pub row_scales: Vec<u16>,
    /// Stage 1: 16-bit E8P code per 8 weights (2 bpw)
    pub e8p_codes: Vec<u16>,
    /// Stage 2: 8-bit residual code per 8 weights (1 bpw)
    pub residual_codes: Vec<u8>,
}

impl RhtE8P3BitMatrix {
    pub fn encode(
        weights: &[f32],
        rows: usize,
        cols: usize,
        seed_left: u64,
        seed_right: u64,
    ) -> Result<Self> {
        let rot = rht_transform_matrix(weights, rows, cols, seed_left, seed_right)?;
        let num_blocks = (rows * cols) / E8_DIM;
        let mut row_scales = Vec::with_capacity(rows);
        let mut e8p_codes = Vec::with_capacity(num_blocks);
        let mut residual_codes = Vec::with_capacity(num_blocks);

        let e8_codebook = E8Codebook::new();
        for r in 0..rows {
            let row_offset = r * cols;
            let row = &rot[row_offset..row_offset + cols];
            let sum_sq: f32 = row.iter().map(|&v| v * v).sum();
            let rms = (sum_sq / cols as f32).sqrt().max(1e-9);
            let scale = rms * 0.7071;
            row_scales.push(encode_scale_fp16(scale));

            let res_scale = scale * 0.5;
            for b in 0..(cols / E8_DIM) {
                let mut block = [0.0f32; E8_DIM];
                block.copy_from_slice(&row[b * E8_DIM..(b + 1) * E8_DIM]);
                let (q1, code1) = quantize_e8p_block(&block, scale);
                e8p_codes.push(code1);

                // Residual
                let mut res = [0.0f32; E8_DIM];
                for i in 0..E8_DIM {
                    res[i] = block[i] - q1[i];
                }
                // Find nearest E8 root (from the 241 roots of E8Codebook)
                let mut best_root_idx = 0u8;
                let mut best_root_err = f32::INFINITY;
                for root_idx in 0..E8_CODEWORDS {
                    let root = e8_codebook.roots[root_idx];
                    let mut err = 0.0f32;
                    for i in 0..E8_DIM {
                        let pred = root[i] as f32 * res_scale;
                        let diff = res[i] - pred;
                        err += diff * diff;
                    }
                    if err < best_root_err {
                        best_root_err = err;
                        best_root_idx = root_idx as u8;
                    }
                }
                residual_codes.push(best_root_idx);
            }
        }

        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes,
            residual_codes,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut rot_out = vec![0.0f32; total];
        let blocks_per_row = self.cols / E8_DIM;
        let e8_codebook = E8Codebook::new();

        for r in 0..self.rows {
            let scale = decode_scale_fp16(self.row_scales[r]);
            let res_scale = scale * 0.5;
            for b in 0..blocks_per_row {
                let idx = r * blocks_per_row + b;
                let code1 = self.e8p_codes[idx];
                let code2 = self.residual_codes[idx];

                let block1 = decode_e8p_codeword(code1);
                let root2 = e8_codebook.roots[code2 as usize];

                let offset = r * self.cols + b * E8_DIM;
                for i in 0..E8_DIM {
                    rot_out[offset + i] = block1[i] * scale + (root2[i] as f32) * res_scale;
                }
            }
        }

        rht_inverse_matrix(
            &rot_out,
            self.rows,
            self.cols,
            self.seed_left,
            self.seed_right,
        )
    }

    pub fn serialized_bytes(&self) -> usize {
        32 + self.row_scales.len() * 2 + self.e8p_codes.len() * 2 + self.residual_codes.len()
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        3.0000
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.rows * self.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&self.seed_left.to_le_bytes());
        buf.extend_from_slice(&self.seed_right.to_le_bytes());
        buf.extend_from_slice(&(self.rows as u64).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u64).to_le_bytes());
        buf.extend_from_slice(&(self.row_scales.len() as u64).to_le_bytes());
        for &s in &self.row_scales {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(self.e8p_codes.len() as u64).to_le_bytes());
        for &c in &self.e8p_codes {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        buf.extend_from_slice(&(self.residual_codes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.residual_codes);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let seed_left = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let seed_right = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let rows = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let cols = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let num_scales = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut row_scales = Vec::with_capacity(num_scales);
        for _ in 0..num_scales {
            let s = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            row_scales.push(s);
            cursor += 2;
        }
        let num_codes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut e8p_codes = Vec::with_capacity(num_codes);
        for _ in 0..num_codes {
            let c = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            e8p_codes.push(c);
            cursor += 2;
        }
        let num_res = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let residual_codes = bytes[cursor..cursor + num_res].to_vec();
        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes,
            residual_codes,
        })
    }
}

// ---------------------------------------------------------------------------
// 9. RHT + E8P 4-bit Codec (E8P 2-bit + E8P 2-bit RVQ = 4.0000 bpw payload)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct RhtE8P4BitMatrix {
    pub seed_left: u64,
    pub seed_right: u64,
    pub rows: usize,
    pub cols: usize,
    pub row_scales: Vec<u16>,
    /// Stage 1: 16-bit E8P code per 8 weights (2 bpw)
    pub e8p_codes_stage1: Vec<u16>,
    /// Stage 2: 16-bit E8P residual code per 8 weights (2 bpw)
    pub e8p_codes_stage2: Vec<u16>,
}

impl RhtE8P4BitMatrix {
    pub fn encode(
        weights: &[f32],
        rows: usize,
        cols: usize,
        seed_left: u64,
        seed_right: u64,
    ) -> Result<Self> {
        let rot = rht_transform_matrix(weights, rows, cols, seed_left, seed_right)?;
        let num_blocks = (rows * cols) / E8_DIM;
        let mut row_scales = Vec::with_capacity(rows);
        let mut e8p_codes_stage1 = Vec::with_capacity(num_blocks);
        let mut e8p_codes_stage2 = Vec::with_capacity(num_blocks);

        for r in 0..rows {
            let row_offset = r * cols;
            let row = &rot[row_offset..row_offset + cols];
            let sum_sq: f32 = row.iter().map(|&v| v * v).sum();
            let rms = (sum_sq / cols as f32).sqrt().max(1e-9);
            let scale = rms * 0.7071;
            row_scales.push(encode_scale_fp16(scale));

            let res_scale = scale * 0.5;
            for b in 0..(cols / E8_DIM) {
                let mut block = [0.0f32; E8_DIM];
                block.copy_from_slice(&row[b * E8_DIM..(b + 1) * E8_DIM]);
                let (q1, code1) = quantize_e8p_block(&block, scale);
                e8p_codes_stage1.push(code1);

                // Residual
                let mut res = [0.0f32; E8_DIM];
                for i in 0..E8_DIM {
                    res[i] = block[i] - q1[i];
                }
                let (_q2, code2) = quantize_e8p_block(&res, res_scale);
                e8p_codes_stage2.push(code2);
            }
        }

        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes_stage1,
            e8p_codes_stage2,
        })
    }

    pub fn dequantize(&self) -> Result<Vec<f32>> {
        let total = self.rows * self.cols;
        let mut rot_out = vec![0.0f32; total];
        let blocks_per_row = self.cols / E8_DIM;

        for r in 0..self.rows {
            let scale = decode_scale_fp16(self.row_scales[r]);
            let res_scale = scale * 0.5;
            for b in 0..blocks_per_row {
                let idx = r * blocks_per_row + b;
                let code1 = self.e8p_codes_stage1[idx];
                let code2 = self.e8p_codes_stage2[idx];

                let block1 = decode_e8p_codeword(code1);
                let block2 = decode_e8p_codeword(code2);

                let offset = r * self.cols + b * E8_DIM;
                for i in 0..E8_DIM {
                    rot_out[offset + i] = block1[i] * scale + block2[i] * res_scale;
                }
            }
        }

        rht_inverse_matrix(
            &rot_out,
            self.rows,
            self.cols,
            self.seed_left,
            self.seed_right,
        )
    }

    pub fn serialized_bytes(&self) -> usize {
        32 + self.row_scales.len() * 2
            + self.e8p_codes_stage1.len() * 2
            + self.e8p_codes_stage2.len() * 2
    }

    pub fn payload_bits_per_weight(&self) -> f64 {
        4.0000
    }

    pub fn total_bits_per_weight(&self) -> f64 {
        (self.serialized_bytes() * 8) as f64 / (self.rows * self.cols) as f64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_bytes());
        buf.extend_from_slice(&self.seed_left.to_le_bytes());
        buf.extend_from_slice(&self.seed_right.to_le_bytes());
        buf.extend_from_slice(&(self.rows as u64).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u64).to_le_bytes());
        buf.extend_from_slice(&(self.row_scales.len() as u64).to_le_bytes());
        for &s in &self.row_scales {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        buf.extend_from_slice(&(self.e8p_codes_stage1.len() as u64).to_le_bytes());
        for &c in &self.e8p_codes_stage1 {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        buf.extend_from_slice(&(self.e8p_codes_stage2.len() as u64).to_le_bytes());
        for &c in &self.e8p_codes_stage2 {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let seed_left = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let seed_right = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;
        let rows = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let cols = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let num_scales = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut row_scales = Vec::with_capacity(num_scales);
        for _ in 0..num_scales {
            let s = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            row_scales.push(s);
            cursor += 2;
        }
        let num_codes1 = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut e8p_codes_stage1 = Vec::with_capacity(num_codes1);
        for _ in 0..num_codes1 {
            let c = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            e8p_codes_stage1.push(c);
            cursor += 2;
        }
        let num_codes2 = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        let mut e8p_codes_stage2 = Vec::with_capacity(num_codes2);
        for _ in 0..num_codes2 {
            let c = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
            e8p_codes_stage2.push(c);
            cursor += 2;
        }
        Ok(Self {
            seed_left,
            seed_right,
            rows,
            cols,
            row_scales,
            e8p_codes_stage1,
            e8p_codes_stage2,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conway_sloane_e8_nearest() {
        // Point close to (1, 1, 0, 0, 0, 0, 0, 0)
        let y = [0.9f32, 1.1, 0.05, -0.05, 0.0, 0.0, 0.0, 0.0];
        let p = e8_nearest(&y);
        assert_eq!(p, [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Point close to (0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5)
        let y_half = [0.48f32; 8];
        let p_half = e8_nearest(&y_half);
        assert_eq!(p_half, [0.5; 8]);
    }

    #[test]
    fn test_fwht32_invertibility() {
        let mut original = [0.0f32; 32];
        for i in 0..32 {
            original[i] = (i as f32) * 0.1 - 1.5;
        }
        let mut chunk = original;
        fwht32(&mut chunk);
        fwht32(&mut chunk);
        for i in 0..32 {
            assert!(
                (chunk[i] - original[i]).abs() < 1e-5,
                "FWHT mismatch at {i}: {} vs {}",
                chunk[i],
                original[i]
            );
        }
    }

    #[test]
    fn test_rht_matrix_invertibility() {
        let rows = 64;
        let cols = 96;
        let mut weights = vec![0.0f32; rows * cols];
        for i in 0..weights.len() {
            weights[i] = ((i * 37 % 100) as f32) * 0.02 - 1.0;
        }
        let rot = rht_transform_matrix(&weights, rows, cols, 42, 99).unwrap();
        let inv = rht_inverse_matrix(&rot, rows, cols, 42, 99).unwrap();
        for i in 0..weights.len() {
            assert!(
                (inv[i] - weights[i]).abs() < 1e-5,
                "RHT inversion mismatch at {i}: {} vs {}",
                inv[i],
                weights[i]
            );
        }
    }

    #[test]
    fn test_rht_rtn_4bit_round_trip() {
        let rows = 32;
        let cols = 64;
        let weights = vec![0.2f32; rows * cols];
        let matrix = RhtRtn4BitMatrix::encode(&weights, rows, cols, 101, 202).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 4.2500);

        let bytes = matrix.to_bytes();
        let deserialized = RhtRtn4BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }

    #[test]
    fn test_rht_rtn_3bit_round_trip() {
        let rows = 32;
        let cols = 64;
        let weights = vec![0.15f32; rows * cols];
        let matrix = RhtRtn3BitMatrix::encode(&weights, rows, cols, 303, 404).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 3.0000);

        let bytes = matrix.to_bytes();
        let deserialized = RhtRtn3BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
    }

    #[test]
    fn test_rht_e8p_2bit_round_trip() {
        let rows = 32;
        let cols = 64;
        let weights = vec![0.1f32; rows * cols];
        let matrix = RhtE8P2BitMatrix::encode(&weights, rows, cols, 505, 606).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 2.0000);

        let bytes = matrix.to_bytes();
        let deserialized = RhtE8P2BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
        let err: f32 = weights
            .iter()
            .zip(deq.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum();
        let norm: f32 = weights.iter().map(|&a| a * a).sum();
        let rel_err = (err / norm).sqrt();
        assert!(rel_err < 0.60, "2-bit relative error: {rel_err}");
    }

    #[test]
    fn test_rht_e8p_3bit_round_trip() {
        let rows = 32;
        let cols = 64;
        let weights = vec![0.25f32; rows * cols];
        let matrix = RhtE8P3BitMatrix::encode(&weights, rows, cols, 707, 808).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 3.0000);

        let bytes = matrix.to_bytes();
        let deserialized = RhtE8P3BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
        let err: f32 = weights
            .iter()
            .zip(deq.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum();
        let norm: f32 = weights.iter().map(|&a| a * a).sum();
        let rel_err = (err / norm).sqrt();
        assert!(rel_err < 0.45, "3-bit relative error: {rel_err}");
    }

    #[test]
    fn test_rht_e8p_4bit_round_trip() {
        let rows = 32;
        let cols = 64;
        let weights = vec![0.3f32; rows * cols];
        let matrix = RhtE8P4BitMatrix::encode(&weights, rows, cols, 909, 1010).unwrap();
        assert_eq!(matrix.payload_bits_per_weight(), 4.0000);

        let bytes = matrix.to_bytes();
        let deserialized = RhtE8P4BitMatrix::from_bytes(&bytes).unwrap();
        assert_eq!(matrix, deserialized);

        let deq = deserialized.dequantize().unwrap();
        assert_eq!(deq.len(), weights.len());
        let err: f32 = weights
            .iter()
            .zip(deq.iter())
            .map(|(&a, &b)| (a - b).powi(2))
            .sum();
        let norm: f32 = weights.iter().map(|&a| a * a).sum();
        let rel_err = (err / norm).sqrt();
        assert!(rel_err < 0.35, "4-bit relative error: {rel_err}");
    }

    #[test]
    fn test_e8p_encoder_counterexample_and_oracle() {
        // Documented counterexample: decode_e8p_codeword(256)
        let c_true = 256u16;
        let v = decode_e8p_codeword(c_true);
        assert_eq!(v, [0.75, 0.75, 0.75, 0.75, 0.75, 0.75, 0.75, -1.25]);

        // Oracle finds exact codeword 256 with 0 error under exhaustive enumeration
        let (oracle_v, oracle_c) =
            oracle_nearest_e8p_codeword(&v, 1.0).expect("oracle search must succeed");
        assert_eq!(oracle_c, 256);
        assert_eq!(oracle_v, v);

        // Oracle rejects non-finite scale and vector inputs
        assert!(oracle_nearest_e8p_codeword(&v, f32::NAN).is_err());
        assert!(oracle_nearest_e8p_codeword(&v, 0.0).is_err());
        assert!(oracle_nearest_e8p_codeword(&v, 1e-10).is_err());
        let mut nan_v = v;
        nan_v[3] = f32::NAN;
        assert!(oracle_nearest_e8p_codeword(&nan_v, 1.0).is_err());

        // Oracle rejects finite overflow cases where distances or normalized values exceed finite domain
        assert!(
            oracle_nearest_e8p_codeword(&[1e20; E8_DIM], 1.0).is_err(),
            "finite input with squared-distance overflow must be rejected"
        );
        assert!(
            oracle_nearest_e8p_codeword(&[f32::MAX; E8_DIM], 0.5).is_err(),
            "normalization overflow must be rejected"
        );

        // Documented heuristic encoder limitation: returns code 128 with squared error 4.0
        let (heur_v, heur_c) = quantize_e8p_block(&v, 1.0);
        assert_eq!(heur_c, 128);
        assert_eq!(heur_v, [-0.75, 0.25, 0.25, 0.25, 0.25, 0.25, 0.25, -0.75]);
        let err: f32 = v
            .iter()
            .zip(heur_v.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        assert!((err - 4.0).abs() < 1e-5);
    }
}
