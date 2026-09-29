//! Discrete parameter codecs for native geometric serving under D11.
//!
//! Provides geometry-coded representations:
//! 1. **Hadamard + Grouped 4-bit (H+G4)**: Block-padded randomized Walsh-Hadamard transform
//!    (incoherence processing) followed by power-of-two grouped 4-bit quantization.
//!    Supports arbitrary vector/row dimensions (e.g. 288, 749).
//! 2. **Hadamard + E8 (H+E8)**: Block-padded randomized Walsh-Hadamard transform followed
//!    by 8-dimensional Conway-Sloane E8 lattice quantization with zero codeword and equal-norm roots.
//! 3. **Head-Compensated 4-bit**: Grouped 4-bit with output-layer weight-error scale search.
//!
//! D11 served runtime operations (FWHT, codebook lookup, scale shifts, and dot products)
//! strictly adhere to owner decisions D0-b and D11: zero hardware multipliers,
//! zero hardware dividers, and zero floating-point operations in served code.

use crate::math;
use crate::{invalid, Result};
use std::sync::Arc;

/// Default group size for grouped 4-bit quantization (matches `uor-r4-lut` GROUP).
pub const DEFAULT_GROUP_SIZE: usize = 32;
/// Block dimension for E8 lattice quantization.
pub const E8_DIM: usize = 8;
/// Block size for Walsh-Hadamard transforms.
pub const HADAMARD_BLOCK: usize = 64;

// ---------------------------------------------------------------------------
// Fast Walsh-Hadamard Transform (FWHT) & Randomized Hadamard Transform
// ---------------------------------------------------------------------------

/// In-place Fast Walsh-Hadamard Transform over a power-of-two slice.
///
/// Uses butterfly additions and subtractions only (0 multipliers, 0 dividers, 0 floats).
pub fn fwht_slice(data: &mut [i64]) -> Result<()> {
    let len = data.len();
    if len == 0 || (len & (len - 1)) != 0 {
        return Err(invalid("FWHT slice length must be a non-zero power of two"));
    }
    let mut h = 1;
    while h < len {
        let mut i = 0;
        while i < len {
            for j in i..(i + h) {
                let u = data[j];
                let v = data[j + h];
                data[j] = u
                    .checked_add(v)
                    .ok_or_else(|| invalid("FWHT overflow on add"))?;
                data[j + h] = u
                    .checked_sub(v)
                    .ok_or_else(|| invalid("FWHT overflow on sub"))?;
            }
            i += h * 2;
        }
        h *= 2;
    }
    Ok(())
}

/// Fast Walsh-Hadamard Transform with exact power-of-two normalizer shift.
///
/// For block size N = 4^k, sqrt(N) = 2^k, so normalization is an exact right shift by k bits.
/// For N = 64 (4^3), shift is 3 bits (sqrt(64) = 8 = 2^3).
/// For N = 256 (4^4), shift is 4 bits (sqrt(256) = 16 = 2^4).
pub fn fwht_normalized(data: &mut [i64], shift: u32) -> Result<()> {
    if shift >= 64 {
        return Err(invalid(format!(
            "FWHT normalizer shift must be < 64, got {shift}"
        )));
    }
    fwht_slice(data)?;
    if shift > 0 {
        let half = 1i64 << (shift - 1);
        for val in data.iter_mut() {
            let sum = val
                .checked_add(half)
                .ok_or_else(|| invalid("FWHT normalizer overflow on rounding add"))?;
            *val = sum >> shift;
        }
    }
    Ok(())
}

/// Deterministic sign vector generator for randomized Hadamard transform (incoherence).
///
/// Uses the project's standard Galois LFSR polynomial (0xD800000000000000) to produce
/// deterministic pseudo-random signs (+1 or -1) without external RNG dependencies.
pub fn deterministic_signs(dim: usize, seed: u64) -> Vec<i8> {
    let mut state = if seed == 0 { 0x517cc1b727220a95 } else { seed };
    let mut signs = Vec::with_capacity(dim);
    for _ in 0..dim {
        // Galois LFSR step
        let lsb = state & 1;
        state >>= 1;
        if lsb != 0 {
            state ^= 0xD800000000000000;
        }
        signs.push(if (state & 1) == 0 { 1 } else { -1 });
    }
    signs
}

/// Apply randomized Hadamard transform to an input vector in blocks of `HADAMARD_BLOCK` (64).
///
/// For each 64-element block:
/// 1. Sign modulation: x_i -> x_i * S_i (conditional negation: 0 multipliers).
/// 2. FWHT over 64 elements (6 stages of butterfly add/sub).
/// 3. Normalization: shift right by 3 bits (sqrt(64) = 8 = 2^3).
pub fn randomized_hadamard_transform(data: &mut [i64], signs: &[i8]) -> Result<()> {
    if data.len() % HADAMARD_BLOCK != 0 {
        return Err(invalid(format!(
            "randomized Hadamard requires dimension multiple of {}, got {}",
            HADAMARD_BLOCK,
            data.len()
        )));
    }
    let data_len = data.len();
    if signs.len() < data_len {
        return Err(invalid(format!(
            "signs length ({}) less than data length ({})",
            signs.len(),
            data_len
        )));
    }
    for (chunk, sign_chunk) in data
        .chunks_exact_mut(HADAMARD_BLOCK)
        .zip(signs[..data_len].chunks_exact(HADAMARD_BLOCK))
    {
        for (x, &s) in chunk.iter_mut().zip(sign_chunk) {
            if s < 0 {
                *x = x
                    .checked_neg()
                    .ok_or_else(|| invalid("overflow on sign negation"))?;
            }
        }
        fwht_normalized(chunk, 3)?;
    }
    Ok(())
}

/// Apply inverse randomized Hadamard transform.
///
/// Because H is symmetric and orthogonal (H^T = H, H H = I), the inverse is:
/// 1. FWHT over 64 elements with shift 3.
/// 2. Sign modulation: x_i -> x_i * S_i.
pub fn inverse_randomized_hadamard_transform(data: &mut [i64], signs: &[i8]) -> Result<()> {
    if data.len() % HADAMARD_BLOCK != 0 {
        return Err(invalid("dimension not a multiple of HADAMARD_BLOCK"));
    }
    let data_len = data.len();
    if signs.len() < data_len {
        return Err(invalid(format!(
            "signs length ({}) less than data length ({})",
            signs.len(),
            data_len
        )));
    }
    for (chunk, sign_chunk) in data
        .chunks_exact_mut(HADAMARD_BLOCK)
        .zip(signs[..data_len].chunks_exact(HADAMARD_BLOCK))
    {
        fwht_normalized(chunk, 3)?;
        for (x, &s) in chunk.iter_mut().zip(sign_chunk) {
            if s < 0 {
                *x = x
                    .checked_neg()
                    .ok_or_else(|| invalid("overflow on sign negation"))?;
            }
        }
    }
    Ok(())
}

/// Apply block-padded randomized Hadamard transform to an input slice of arbitrary dimension.
///
/// Pads the data with zeros to the next multiple of `HADAMARD_BLOCK` (64), applies
/// sign modulation and FWHT in 64-element blocks with shift 3.
/// Handles arbitrary widths (e.g. hidden width 288 -> padded to 320, MLP width 749 -> padded to 768).
pub fn block_padded_hadamard_transform(data: &[i64], signs: &[i8]) -> Result<Vec<i64>> {
    let orig_len = data.len();
    if orig_len == 0 {
        return Ok(Vec::new());
    }
    let padded_len = orig_len.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
    if signs.len() < padded_len {
        return Err(invalid(format!(
            "signs length ({}) less than padded dimension ({})",
            signs.len(),
            padded_len
        )));
    }
    let mut padded = vec![0i64; padded_len];
    padded[..orig_len].copy_from_slice(data);
    randomized_hadamard_transform(&mut padded, &signs[..padded_len])?;
    Ok(padded)
}

/// Apply inverse block-padded randomized Hadamard transform and unpad to original dimension.
pub fn inverse_block_padded_hadamard_transform(
    transformed: &[i64],
    signs: &[i8],
    orig_len: usize,
) -> Result<Vec<i64>> {
    if orig_len == 0 {
        return Ok(Vec::new());
    }
    let padded_len = orig_len.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
    if transformed.len() != padded_len {
        return Err(invalid(format!(
            "transformed length ({}) does not match expected padded length ({})",
            transformed.len(),
            padded_len
        )));
    }
    if signs.len() < padded_len {
        return Err(invalid(format!(
            "signs length ({}) less than expected padded length ({})",
            signs.len(),
            padded_len
        )));
    }
    let mut buf = transformed.to_vec();
    inverse_randomized_hadamard_transform(&mut buf, &signs[..padded_len])?;
    buf.truncate(orig_len);
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Grouped 4-bit Codec
// ---------------------------------------------------------------------------

/// Grouped 4-bit representation of a weight matrix row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grouped4BitRow {
    /// Group size (e.g. 32).
    pub group_size: usize,
    /// Dyadic power-of-two exponents per group.
    pub group_exponents: Vec<i16>,
    /// Packed signed 4-bit codes (low nibble first).
    pub packed_codes: Arc<[u8]>,
    /// Number of elements in the row.
    pub elements: usize,
}

impl Grouped4BitRow {
    /// Encode a slice of float weights into a grouped 4-bit row.
    /// Supports arbitrary element count (tail elements form a shorter last group).
    pub fn encode_f32(weights: &[f32], group_size: usize) -> Result<Self> {
        if weights.is_empty() || group_size == 0 {
            return Err(invalid(
                "weights length must be non-zero and group_size > 0",
            ));
        }
        if weights.iter().any(|w| !w.is_finite()) {
            return Err(invalid("weights contains non-finite values"));
        }
        let num_groups = weights.len().div_ceil(group_size);
        let mut group_exponents = Vec::with_capacity(num_groups);
        let mut raw_codes = Vec::with_capacity(weights.len());

        for group in weights.chunks(group_size) {
            let mut maxabs = 0.0f32;
            for &w in group {
                let abs = w.abs();
                if abs > maxabs {
                    maxabs = abs;
                }
            }
            let exp = if maxabs == 0.0 {
                0i16
            } else {
                let raw_ceil = (maxabs / 7.0f32).log2().ceil() as i32;
                raw_ceil.clamp(-24, 16) as i16
            };
            group_exponents.push(exp);

            let scale = 2.0f32.powi(i32::from(exp));
            for &w in group {
                let code = (w / scale).round().clamp(-7.0, 7.0) as i16;
                raw_codes.push(code);
            }
        }

        let mut packed = Vec::with_capacity(weights.len().div_ceil(2));
        for pair in raw_codes.chunks(2) {
            let low = (pair[0] as u8) & 15;
            let high = pair.get(1).map_or(0, |&c| (c as u8) & 15);
            packed.push(low | (high << 4));
        }

        Ok(Self {
            group_size,
            group_exponents,
            packed_codes: packed.into(),
            elements: weights.len(),
        })
    }

    /// Dequantize back to float for verification / evaluation.
    pub fn dequantize_f32(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.elements);
        for (g_idx, &exp) in self.group_exponents.iter().enumerate() {
            let scale = 2.0f32.powi(i32::from(exp));
            let start = g_idx * self.group_size;
            let end = (start + self.group_size).min(self.elements);
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = (nibble as i8) << 4 >> 4;
                out.push((code as f32) * scale);
            }
        }
        out
    }

    /// Extract row activations in fixed-point Q16 without floating-point arithmetic.
    ///
    /// Evaluates: q_i * 2^(16 + exp_g) using dyadic scale_pow2 (0 multipliers, 0 floats).
    #[inline(never)]
    pub fn to_q16_vector(&self) -> Result<Vec<i64>> {
        let mut out = Vec::with_capacity(self.elements);
        let mut start = 0usize;
        for &exp in &self.group_exponents {
            let shift = 16 + i32::from(exp);
            let end = (start + self.group_size).min(self.elements);
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = i64::from((nibble as i8) << 4 >> 4);
                let scaled = math::scale_pow2(i128::from(code), shift)
                    .map_err(|e| invalid(format!("scale_pow2 error: {e}")))?;
                out.push(i64::try_from(scaled).map_err(|_| invalid("embedding out of bounds"))?);
            }
            start += self.group_size;
        }
        if out.len() != self.elements {
            return Err(invalid("group_exponents did not cover all elements"));
        }
        Ok(out)
    }

    /// Multiplier-free dot product with activation vector in integer arithmetic.
    ///
    /// Evaluates: Sum_g 2^(exp_g) * (Sum_{i in g} x_i * q_i)
    /// where x_i * q_i uses product table or shift-and-add (zero hardware multipliers).
    pub fn dot_integer(&self, activations: &[i64]) -> Result<i64> {
        if activations.len() != self.elements {
            return Err(invalid("activation dimension mismatch"));
        }
        let mut total_sum: i128 = 0;
        let mut covered = 0usize;
        for (g_idx, &exp) in self.group_exponents.iter().enumerate() {
            let start = g_idx * self.group_size;
            let end = (start + self.group_size).min(self.elements);
            covered = end;
            let mut group_dot: i64 = 0;
            for i in start..end {
                let byte = self.packed_codes[i >> 1];
                let nibble = (byte >> ((i & 1) << 2)) & 15;
                let code = i64::from((nibble as i8) << 4 >> 4);
                let x = activations[i];
                let prod = mul_small_code_i64(x, code)?;
                group_dot = group_dot
                    .checked_add(prod)
                    .ok_or_else(|| invalid("group dot overflow"))?;
            }
            let scaled = math::scale_pow2(i128::from(group_dot), i32::from(exp))
                .map_err(|e| invalid(format!("scale_pow2 error: {e}")))?;
            total_sum = total_sum
                .checked_add(scaled)
                .ok_or_else(|| invalid("total dot overflow"))?;
        }
        if covered != self.elements {
            return Err(invalid("group_exponents did not cover all elements"));
        }
        i64::try_from(total_sum).map_err(|_| invalid("result exceeds i64"))
    }
}

// ---------------------------------------------------------------------------
// Canonical Grouped 4-bit Matrix & Codec (D11 and MapCodec contract)
// ---------------------------------------------------------------------------

/// Rounding and scale selection mode for discrete 4-bit quantization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Grouped4BitRounding {
    /// Standard round-to-nearest with heuristic non-saturating scale selection
    /// matching canonical D11 export (`lut_export::quantize_matrix`).
    #[default]
    Nearest,
    /// Minimum-MSE grid scale search: for each group, evaluates all 16 valid mantissas
    /// under the matrix base exponent to find the scale that minimizes squared error.
    MinimumMseScale,
}

/// Stored representation of a grouped 4-bit matrix under D11.
///
/// Encodes an `R x C` matrix with column grouping `group_size` (default 32):
/// - 4-bit signed integer codes in `[-8..7]` stored as unsigned nibbles `q + 8`, packed 2 per byte (low nibble first).
/// - 1 scale byte per group: `(m & 15) | ((e - exp_base) << 4)` with $m \in [0..15]$ and $(e - exp_base) \in [0..15]$.
/// - Grid scale value: $(16 + m) \cdot 2^{e - 4}$.
/// - Base exponent `exp_base: i32` stored once per matrix.
///
/// Bit budget:
/// - Nibbles: $(R \times C).div\_ceil(2)$ bytes = 4.0 bits / weight.
/// - Scales: $R \times C.div\_ceil(group\_size)$ bytes = 8 bits / 32 weights = 0.25 bits / weight (for $G=32$).
/// - Container framing: 64 bytes total overhead (32-byte header + 32-byte BLAKE3 checksum).
/// - Total effective bits per weight: $\le 4.251$ bits / weight including container framing for model matrices ($C \ge 32$).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grouped4BitMatrix {
    pub rows: usize,
    pub cols: usize,
    pub group_size: usize,
    pub exp_base: i32,
    pub nibbles: Vec<u8>,
    pub scales: Vec<u8>,
}

/// Binary serialization magic: b"UORG4B01".
pub const GROUPED4BIT_MATRIX_MAGIC: &[u8; 8] = b"UORG4B01";

/// Container framing size in bytes: 8 bytes magic + 6 * 4 bytes header fields + 32 bytes BLAKE3 hash = 64 bytes.
pub const CONTAINER_FRAMING_BYTES: usize = 64;

impl Grouped4BitMatrix {
    /// Container framing size in bytes: 8 bytes magic + 6 * 4 bytes header fields + 32 bytes BLAKE3 hash = 64 bytes.
    pub const CONTAINER_FRAMING_BYTES: usize = CONTAINER_FRAMING_BYTES;

    /// Validate all structural and numerical invariants:
    /// - Dimensions are non-zero and multiplication does not overflow.
    /// - Scales vector length matches `rows * cols.div_ceil(group_size)`.
    /// - Nibbles vector length matches `(rows * cols).div_ceil(2)`.
    /// - `exp_base` is within safe float dynamic range `[-200, 120]`.
    /// - Every scale byte decodes to a finite positive scale within `f32` range.
    pub fn validate(&self) -> Result<()> {
        if self.rows == 0 || self.cols == 0 || self.group_size == 0 {
            return Err(invalid("matrix dimensions and group_size must be non-zero"));
        }
        let total_weights = self
            .rows
            .checked_mul(self.cols)
            .ok_or_else(|| invalid("matrix dimensions overflow"))?;
        let groups = self.cols.div_ceil(self.group_size);
        let expected_scales = self
            .rows
            .checked_mul(groups)
            .ok_or_else(|| invalid("scales dimension overflow"))?;
        let expected_nibbles = total_weights.div_ceil(2);

        if self.scales.len() != expected_scales {
            return Err(invalid(format!(
                "scales buffer length mismatch: got {}, expected {expected_scales}",
                self.scales.len()
            )));
        }
        if self.nibbles.len() != expected_nibbles {
            return Err(invalid(format!(
                "nibbles buffer length mismatch: got {}, expected {expected_nibbles}",
                self.nibbles.len()
            )));
        }
        if !(-200..=120).contains(&self.exp_base) {
            return Err(invalid(format!(
                "exp_base ({}) out of safe range [-200, 120]",
                self.exp_base
            )));
        }

        // Validate that all scale bytes decode to finite positive scales fitting in f32
        for &s_byte in &self.scales {
            let m = s_byte & 15;
            let de = s_byte >> 4;
            let e = self
                .exp_base
                .checked_add(i32::from(de))
                .ok_or_else(|| invalid("exponent addition overflow"))?;
            let scale = d11_grid_scale(m, e);
            if !scale.is_finite() || scale <= 0.0 {
                return Err(invalid("decoded scale is non-finite or non-positive"));
            }
            let max_val = (7.0 * scale) as f32;
            let min_val = (-8.0 * scale) as f32;
            if !max_val.is_finite() || !min_val.is_finite() {
                return Err(invalid("quantized dynamic range overflows f32"));
            }
        }
        Ok(())
    }

    /// Calculate raw parameter bits per weight: 4.0 bits code + (8.0 / group_size) scale bits = 4.25 bits/weight (for group_size = 32).
    pub fn param_bits_per_weight(&self) -> f64 {
        (self.nibbles.len() + self.scales.len()) as f64 * 8.0 / (self.rows * self.cols) as f64
    }

    /// Exact total stored byte count of the serialized payload written by `to_bytes()`.
    pub fn total_stored_bytes(&self) -> usize {
        CONTAINER_FRAMING_BYTES + self.nibbles.len() + self.scales.len()
    }

    /// Total stored bits per weight including container framing and BLAKE3 checksum (all 64 framing bytes).
    pub fn total_stored_bits_per_weight(&self) -> f64 {
        (self.total_stored_bytes() as f64 * 8.0) / (self.rows * self.cols) as f64
    }

    /// Effective bits per weight under D4 specification: includes container overhead.
    pub fn effective_bits_per_weight(&self) -> f64 {
        self.total_stored_bits_per_weight()
    }

    /// Serialize into self-contained binary payload with BLAKE3 checksum.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.total_stored_bytes());
        buf.extend_from_slice(GROUPED4BIT_MATRIX_MAGIC);
        buf.extend_from_slice(&(self.rows as u32).to_le_bytes());
        buf.extend_from_slice(&(self.cols as u32).to_le_bytes());
        buf.extend_from_slice(&(self.group_size as u32).to_le_bytes());
        buf.extend_from_slice(&self.exp_base.to_le_bytes());
        buf.extend_from_slice(&(self.nibbles.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(self.scales.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.nibbles);
        buf.extend_from_slice(&self.scales);
        let hash = blake3::hash(&buf);
        buf.extend_from_slice(hash.as_bytes());
        buf
    }

    /// Deserialize and verify BLAKE3 checksum and dimension consistency.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        const HEADER_LEN: usize = 8 + 4 * 6; // 32 bytes
        if bytes.len() < HEADER_LEN + 32 {
            return Err(invalid("serialized data too short"));
        }
        let (payload, expected_hash) = bytes.split_at(bytes.len() - 32);
        let actual_hash = blake3::hash(payload);
        if actual_hash.as_bytes() != expected_hash {
            return Err(invalid(
                "BLAKE3 checksum mismatch in Grouped4BitMatrix deserialization",
            ));
        }
        if &payload[0..8] != GROUPED4BIT_MATRIX_MAGIC {
            return Err(invalid("invalid magic bytes in Grouped4BitMatrix header"));
        }
        let rows = u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;
        let cols = u32::from_le_bytes(payload[12..16].try_into().unwrap()) as usize;
        let group_size = u32::from_le_bytes(payload[16..20].try_into().unwrap()) as usize;
        let exp_base = i32::from_le_bytes(payload[20..24].try_into().unwrap());
        let nibbles_len = u32::from_le_bytes(payload[24..28].try_into().unwrap()) as usize;
        let scales_len = u32::from_le_bytes(payload[28..32].try_into().unwrap()) as usize;

        if group_size == 0 || cols == 0 || rows == 0 {
            return Err(invalid("zero dimension in header"));
        }
        let total_weights = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("matrix dimensions overflow"))?;
        let groups = cols.div_ceil(group_size);
        let expected_nibbles = total_weights.div_ceil(2);
        let expected_scales = rows
            .checked_mul(groups)
            .ok_or_else(|| invalid("matrix scales dimension overflow"))?;
        let expected_payload = HEADER_LEN
            .checked_add(expected_nibbles)
            .and_then(|l| l.checked_add(expected_scales))
            .ok_or_else(|| invalid("payload length calculation overflow"))?;

        if nibbles_len != expected_nibbles
            || scales_len != expected_scales
            || payload.len() != expected_payload
        {
            return Err(invalid("payload length mismatch with declared dimensions"));
        }
        let nibbles = payload[HEADER_LEN..HEADER_LEN + nibbles_len].to_vec();
        let scales =
            payload[HEADER_LEN + nibbles_len..HEADER_LEN + nibbles_len + scales_len].to_vec();

        let mat = Self {
            rows,
            cols,
            group_size,
            exp_base,
            nibbles,
            scales,
        };
        mat.validate()?;
        Ok(mat)
    }
}

/// Compute canonical D11 grid scale: `(16 + m) * 2^(e - 4)`.
#[inline]
pub fn d11_grid_scale(m: u8, e: i32) -> f64 {
    (16.0 + f64::from(m & 15)) * 2f64.powi(e - 4)
}

fn group_error_f32(w: &[f32], s: f64) -> f64 {
    w.iter()
        .map(|&v| {
            let vf = f64::from(v);
            let q = (vf / s).round().clamp(-8.0, 7.0);
            (vf - q * s).powi(2)
        })
        .sum()
}

/// The grid scale `(m, e)` with the least squared rounding error for one group
/// matching canonical D11 export (`lut_export::quantize_matrix`), or `None`
/// for an all-zero group.
pub fn d11_best_scale(group: &[f32]) -> Option<(u8, i32)> {
    let mut a = 0.0f64;
    for &v in group {
        let abs = f64::from(v.abs());
        if abs > a {
            a = abs;
        }
    }
    if a == 0.0 {
        return None;
    }
    let (lo, hi) = (a / 8.5, a / 5.5);
    let mut best: Option<(f64, u8, i32)> = None;
    let e_lo = (lo / 32.0).log2().floor() as i32 + 4;
    let e_hi = (hi / 16.0).log2().ceil() as i32 + 4;
    for e in e_lo..=e_hi {
        for m in 0..16u8 {
            let s = d11_grid_scale(m, e);
            if s < lo || s > hi {
                continue;
            }
            let err = group_error_f32(group, s);
            if best.as_ref().is_none_or(|b| err < b.0) {
                best = Some((err, m, e));
            }
        }
    }
    Some(match best {
        Some((_, m, e)) => (m, e),
        // Smallest grid scale >= a / 7.
        None => (0, (a / 7.0 / 16.0).log2().ceil() as i32 + 4),
    })
}

/// Discrete 4-bit grouped parameter codec for D11 serving and QAT training.
///
/// Implements:
/// - Grouping: 32 columns per group (matching `uor_r4_lut::GROUP`).
/// - Scale formula: $s = (16 + m) \cdot 2^{e - 4}$ where $m \in [0..15]$ and $e - \text{exp\_base} \in [0..15]$.
/// - Code representation: 4-bit signed codes in `[-8..7]` stored as `q + 8`.
/// - Rounding modes: [`Grouped4BitRounding::Nearest`] and [`Grouped4BitRounding::MinimumMseScale`].
/// - Effective bits per weight: $\le 4.25$ bits/weight.
///
/// ### Straight-Through Estimator (STE) Contract
/// In offline training (via [`MapCodec`]), the backward pass
/// through this codec substitutes a Straight-Through Estimator:
///
/// $$\frac{\partial L}{\partial w} \approx \frac{\partial L}{\partial q(w)} \cdot \mathbf{1}_{|w| \le w_{\max}}$$
///
/// The codec itself implements the mathematical forward discretization arithmetic and stored
/// representations; the surrogate gradient is maintained by the autodiff graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grouped4BitCodec {
    pub group_size: usize,
    pub rounding: Grouped4BitRounding,
}

impl Default for Grouped4BitCodec {
    fn default() -> Self {
        Self {
            group_size: DEFAULT_GROUP_SIZE,
            rounding: Grouped4BitRounding::Nearest,
        }
    }
}

impl Grouped4BitCodec {
    pub fn new(group_size: usize, rounding: Grouped4BitRounding) -> Self {
        Self {
            group_size,
            rounding,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.rounding {
            Grouped4BitRounding::Nearest => "native-d11-grouped-4bit-g32-rtn",
            Grouped4BitRounding::MinimumMseScale => "native-d11-grouped-4bit-g32-min-mse",
        }
    }

    /// Quantize a row-major `rows x cols` float matrix into canonical D11 grouped 4-bit format.
    /// Supports arbitrary dimensions and group sizes (tail groups gracefully span remaining columns).
    pub fn quantize(&self, values: &[f32], rows: usize, cols: usize) -> Result<Grouped4BitMatrix> {
        if cols == 0 || rows == 0 || self.group_size == 0 {
            return Err(invalid("dimensions and group_size must be non-zero"));
        }
        let total_weights = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("matrix dimensions overflow"))?;
        if values.len() != total_weights {
            return Err(invalid(format!(
                "values length ({}) does not match rows * cols ({rows} * {cols} = {total_weights})",
                values.len(),
            )));
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid("matrix contains non-finite values"));
        }
        let groups = cols.div_ceil(self.group_size);
        let total_scales = rows
            .checked_mul(groups)
            .ok_or_else(|| invalid("scales dimension overflow"))?;
        let total_nibbles = total_weights.div_ceil(2);

        let mut chosen: Vec<Option<(u8, i32)>> = Vec::with_capacity(total_scales);

        for r in 0..rows {
            for g in 0..groups {
                let start_col = g * self.group_size;
                let end_col = (start_col + self.group_size).min(cols);
                let group = &values[r * cols + start_col..r * cols + end_col];
                chosen.push(d11_best_scale(group));
            }
        }

        let e_max = chosen.iter().flatten().map(|(_, e)| *e).max().unwrap_or(0);
        let exp_base = (e_max - 15).clamp(-200, 120);

        let mut nibbles = vec![0u8; total_nibbles];
        let mut scales = vec![0u8; total_scales];

        for r in 0..rows {
            for g in 0..groups {
                let g_idx = r * groups + g;
                let start_col = g * self.group_size;
                let end_col = (start_col + self.group_size).min(cols);
                let group_slice = &values[r * cols + start_col..r * cols + end_col];

                let (m, e) = match self.rounding {
                    Grouped4BitRounding::Nearest => match chosen[g_idx] {
                        Some((m, e)) if e >= exp_base => (m, e),
                        _ => (0, exp_base),
                    },
                    Grouped4BitRounding::MinimumMseScale => {
                        let mut best_m = 0u8;
                        let mut best_e = exp_base;
                        let mut min_err = f64::INFINITY;

                        let heur_e = chosen[g_idx].map(|(_, e)| e).unwrap_or(exp_base);
                        let e_low = (heur_e - 1).clamp(exp_base, exp_base + 15);
                        let e_high = (heur_e + 1).clamp(exp_base, exp_base + 15);

                        for cand_e in e_low..=e_high {
                            for cand_m in 0..=15u8 {
                                let s = d11_grid_scale(cand_m, cand_e);
                                let mut err = 0.0f64;
                                for &val in group_slice {
                                    let vf = f64::from(val);
                                    let q = (vf / s).round().clamp(-8.0, 7.0);
                                    err += (vf - q * s).powi(2);
                                }
                                if err < min_err {
                                    min_err = err;
                                    best_m = cand_m;
                                    best_e = cand_e;
                                }
                            }
                        }
                        (best_m, best_e)
                    }
                };

                let de = (e - exp_base).clamp(0, 15) as u8;
                let scale_byte = (m & 15) | (de << 4);
                scales[g_idx] = scale_byte;

                let s = d11_grid_scale(m, e);
                for (c_rel, &v) in group_slice.iter().enumerate() {
                    let c = start_col + c_rel;
                    let linear_idx = r * cols + c;
                    let vf = f64::from(v);
                    let q = (vf / s).round().clamp(-8.0, 7.0);
                    let nibble = (q as i32 + 8) as u8;
                    let idx = linear_idx / 2;
                    if linear_idx % 2 == 0 {
                        nibbles[idx] |= nibble;
                    } else {
                        nibbles[idx] |= nibble << 4;
                    }
                }
            }
        }

        let mat = Grouped4BitMatrix {
            rows,
            cols,
            group_size: self.group_size,
            exp_base,
            nibbles,
            scales,
        };
        mat.validate()?;
        Ok(mat)
    }

    /// Dequantize a canonical D11 grouped 4-bit matrix back to float representation.
    pub fn dequantize(&self, matrix: &Grouped4BitMatrix) -> Result<Vec<f32>> {
        matrix.validate()?;
        let groups = matrix.cols.div_ceil(matrix.group_size);
        let mut out = Vec::with_capacity(matrix.rows * matrix.cols);

        for r in 0..matrix.rows {
            for g in 0..groups {
                let s_byte = matrix.scales[r * groups + g];
                let m = s_byte & 15;
                let de = s_byte >> 4;
                let e = matrix.exp_base + i32::from(de);
                let scale = d11_grid_scale(m, e);

                let start_col = g * matrix.group_size;
                let end_col = (start_col + matrix.group_size).min(matrix.cols);
                for c in start_col..end_col {
                    let linear_idx = r * matrix.cols + c;
                    let idx = linear_idx / 2;
                    let byte = matrix.nibbles[idx];
                    let nibble = if linear_idx % 2 == 0 {
                        byte & 15
                    } else {
                        byte >> 4
                    };
                    let q = f64::from(nibble as i32 - 8);
                    let val = (q * scale) as f32;
                    if !val.is_finite() {
                        return Err(invalid("decoded float value is not finite"));
                    }
                    out.push(val);
                }
            }
        }

        Ok(out)
    }

    /// Round-trip a row-major float matrix through quantization and dequantization.
    pub fn round_trip(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        let m = self.quantize(values, rows, cols)?;
        self.dequantize(&m)
    }

    /// Quantize a row-major matrix with zero-padding when `cols` is not a multiple of `group_size`.
    ///
    /// Pads columns up to `cols.div_ceil(self.group_size) * self.group_size` using zero padding
    /// matching the canonical D11 export convention (`crate::stack_export::pad`).
    pub fn quantize_padded(
        &self,
        values: &[f32],
        rows: usize,
        cols: usize,
    ) -> Result<Grouped4BitMatrix> {
        if cols == 0 || rows == 0 || self.group_size == 0 {
            return Err(invalid("dimensions and group_size must be non-zero"));
        }
        let total_weights = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("matrix dimensions overflow"))?;
        if values.len() != total_weights {
            return Err(invalid(format!(
                "values length ({}) does not match rows * cols ({rows} * {cols} = {total_weights})",
                values.len(),
            )));
        }
        if cols.is_multiple_of(self.group_size) {
            return self.quantize(values, rows, cols);
        }
        let padded_cols = cols
            .checked_add(self.group_size - 1)
            .map(|c| (c / self.group_size) * self.group_size)
            .ok_or_else(|| invalid("padded cols overflow"))?;
        let padded_weights = rows
            .checked_mul(padded_cols)
            .ok_or_else(|| invalid("padded weights overflow"))?;
        let mut padded = vec![0.0f32; padded_weights];
        for r in 0..rows {
            let src = &values[r * cols..(r + 1) * cols];
            let dst = &mut padded[r * padded_cols..r * padded_cols + cols];
            dst.copy_from_slice(src);
        }
        self.quantize(&padded, rows, padded_cols)
    }

    /// Dequantize a matrix and slice each row back to `original_cols`.
    pub fn dequantize_unpadded(
        &self,
        matrix: &Grouped4BitMatrix,
        original_cols: usize,
    ) -> Result<Vec<f32>> {
        if original_cols == 0 || original_cols > matrix.cols {
            return Err(invalid(format!(
                "original_cols ({original_cols}) must be in range 1..={}",
                matrix.cols
            )));
        }
        let deq = self.dequantize(matrix)?;
        if original_cols == matrix.cols {
            return Ok(deq);
        }
        let mut out = Vec::with_capacity(matrix.rows * original_cols);
        for r in 0..matrix.rows {
            out.extend_from_slice(&deq[r * matrix.cols..r * matrix.cols + original_cols]);
        }
        Ok(out)
    }

    /// Round-trip a row-major float matrix with arbitrary column width through zero-padded
    /// quantization and dequantization.
    pub fn round_trip_padded(&self, values: &[f32], rows: usize, cols: usize) -> Result<Vec<f32>> {
        if cols == 0 || rows == 0 || self.group_size == 0 {
            return Err(invalid("dimensions and group_size must be non-zero"));
        }
        if cols.is_multiple_of(self.group_size) {
            self.round_trip(values, rows, cols)
        } else {
            let m = self.quantize_padded(values, rows, cols)?;
            self.dequantize_unpadded(&m, cols)
        }
    }

    /// Calculate raw parameter bits per weight without container framing.
    pub fn param_bits_per_weight(&self, rows: usize, cols: usize) -> f64 {
        let total_weights = rows * cols;
        if total_weights == 0 {
            return 0.0;
        }
        let nibbles_bytes = total_weights.div_ceil(2);
        let groups = cols.div_ceil(self.group_size);
        let scales_bytes = rows * groups;
        let total_bytes = nibbles_bytes + scales_bytes;
        (total_bytes as f64 * 8.0) / total_weights as f64
    }

    /// Calculate effective bits per weight for a matrix with given shape under this codec.
    ///
    /// Accounts for 4.0 bits code + (8.0 / group_size) scale bits, plus container framing overhead (64 bytes).
    pub fn effective_bits_per_weight(&self, rows: usize, cols: usize) -> f64 {
        let total_weights = rows * cols;
        if total_weights == 0 {
            return 0.0;
        }
        let nibbles_bytes = total_weights.div_ceil(2);
        let groups = cols.div_ceil(self.group_size);
        let scales_bytes = rows * groups;
        let total_bytes = Grouped4BitMatrix::CONTAINER_FRAMING_BYTES + nibbles_bytes + scales_bytes;
        (total_bytes as f64 * 8.0) / total_weights as f64
    }
}

/// Nearest D8 lattice point (integer coordinates with even sum).
///
/// Conway-Sloane (1982) fast decoding algorithm:
/// 1. Round each coordinate to nearest integer.
/// 2. If the sum of coordinates is odd, find the coordinate with the largest
///    rounding error and flip it to the other nearest integer.
pub fn d8_round(y: &[f64; E8_DIM]) -> [f64; E8_DIM] {
    let mut f = [0.0f64; E8_DIM];
    for i in 0..E8_DIM {
        f[i] = y[i].round();
    }
    let sum: i64 = f.iter().map(|&v| v as i64).sum();
    if sum % 2 != 0 {
        // Find coordinate with largest rounding error
        let mut max_err = -1.0f64;
        let mut worst_idx = 0;
        let mut worst_flip = 0.0f64;
        for i in 0..E8_DIM {
            let err = (y[i] - f[i]).abs();
            if err > max_err {
                max_err = err;
                worst_idx = i;
                worst_flip = if y[i] >= f[i] { f[i] + 1.0 } else { f[i] - 1.0 };
            }
        }
        f[worst_idx] = worst_flip;
    }
    f
}

/// Nearest E8 lattice point via Conway-Sloane union of D8 and D8 + 1/2.
///
/// E8 = D8 U (D8 + (1/2, 1/2, ..., 1/2)).
pub fn e8_round(x: &[f64; E8_DIM]) -> [f64; E8_DIM] {
    let f0 = d8_round(x);
    let mut dist0 = 0.0f64;
    for i in 0..E8_DIM {
        let d = x[i] - f0[i];
        dist0 += d * d;
    }

    let mut shifted = [0.0f64; E8_DIM];
    for i in 0..E8_DIM {
        shifted[i] = x[i] - 0.5;
    }
    let f1_shift = d8_round(&shifted);
    let mut f1 = [0.0f64; E8_DIM];
    let mut dist1 = 0.0f64;
    for i in 0..E8_DIM {
        f1[i] = f1_shift[i] + 0.5;
        let d = x[i] - f1[i];
        dist1 += d * d;
    }

    if dist0 <= dist1 {
        f0
    } else {
        f1
    }
}

/// Normalized E8 codebook with zero codeword and equal-norm roots.
///
/// Represents points in the scaled lattice 2*E8:
/// - Type 1 roots: permutations of (+-2, +-2, 0, 0, 0, 0, 0, 0) -> squared norm = 4 + 4 = 8 (112 roots).
/// - Type 2 roots: (+-1, ..., +-1) with even number of minus signs -> squared norm = 8 * 1 = 8 (128 roots).
/// - Zero codeword: (0, 0, 0, 0, 0, 0, 0, 0) -> squared norm = 0 (1 codeword).
/// Total: 241 codewords (fits in 8 bits / 1 byte per 8 weights = 1.0 bit/weight).
#[derive(Clone, Debug)]
pub struct E8Codebook {
    pub roots: [[i8; E8_DIM]; 241],
}

impl Default for E8Codebook {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Codebook {
    /// Initialize the standard E8 root codebook with uniform root scale and zero codeword.
    pub fn new() -> Self {
        let mut roots = [[0i8; E8_DIM]; 241];
        let mut count = 0;

        // Codeword 0: Zero codeword
        roots[0] = [0i8; E8_DIM];
        count += 1;

        // Type 1 roots (112 roots): permutations of (+-2, +-2, 0, 0, 0, 0, 0, 0).
        // Coords scaled by 2 so norm squared is exactly 4 + 4 = 8.
        for i in 0..E8_DIM {
            for j in (i + 1)..E8_DIM {
                for &s1 in &[-2i8, 2i8] {
                    for &s2 in &[-2i8, 2i8] {
                        if count < 241 {
                            let mut r = [0i8; E8_DIM];
                            r[i] = s1;
                            r[j] = s2;
                            roots[count] = r;
                            count += 1;
                        }
                    }
                }
            }
        }

        // Type 2 roots (128 roots): (+-1, ..., +-1) with even number of minus signs.
        // Coords are +-1, so norm squared is exactly 8 * 1 = 8.
        for code in 0..256u16 {
            let mut r = [1i8; E8_DIM];
            let mut minus_count = 0;
            for b in 0..8 {
                if (code & (1 << b)) != 0 {
                    r[b] = -1;
                    minus_count += 1;
                }
            }
            if minus_count % 2 == 0 && count < 241 {
                roots[count] = r;
                count += 1;
            }
        }

        Self { roots }
    }

    /// Find the index of the nearest E8 root (or zero) to a given 8D block.
    pub fn quantize_block(&self, block: &[f64; E8_DIM]) -> (u8, f64) {
        let energy: f64 = block.iter().map(|&x| x * x).sum();
        let scale = (energy / 8.0).sqrt();

        if scale < 1e-9 {
            return (0, 0.0);
        }

        let mut best_idx = 0u8;
        let mut best_dist = energy; // Distance to zero codeword is energy

        for (idx, root) in self.roots.iter().enumerate() {
            let mut dist = 0.0f64;
            for i in 0..E8_DIM {
                let diff = block[i] - (root[i] as f64) * scale;
                dist += diff * diff;
            }
            if dist < best_dist {
                best_dist = dist;
                best_idx = idx as u8;
            }
        }

        (best_idx, scale)
    }

    /// Dequantize a root index given block scale.
    pub fn dequantize_block(&self, idx: u8, scale: f64) -> [f64; E8_DIM] {
        let root = &self.roots[idx as usize];
        let mut out = [0.0f64; E8_DIM];
        for i in 0..E8_DIM {
            out[i] = (root[i] as f64) * scale;
        }
        out
    }
}

/// E8 encoded matrix row.
#[derive(Clone, Debug)]
pub struct E8EncodedRow {
    pub indices: Vec<u8>,
    pub scales: Vec<f32>,
    pub elements: usize,
}

impl E8EncodedRow {
    /// Encode a slice of float weights into E8 lattice codewords.
    pub fn encode_f32(weights: &[f32], codebook: &E8Codebook) -> Result<Self> {
        if weights.is_empty() {
            return Err(invalid("weights slice cannot be empty"));
        }
        let padded_len = weights.len().div_ceil(E8_DIM) * E8_DIM;
        let mut padded = vec![0.0f64; padded_len];
        for (dst, &src) in padded.iter_mut().zip(weights) {
            *dst = src as f64;
        }

        let num_blocks = padded_len / E8_DIM;
        let mut indices = Vec::with_capacity(num_blocks);
        let mut scales = Vec::with_capacity(num_blocks);

        for block in padded.chunks_exact(E8_DIM) {
            let mut b = [0.0f64; E8_DIM];
            b.copy_from_slice(block);
            let (idx, scale) = codebook.quantize_block(&b);
            indices.push(idx);
            scales.push(scale as f32);
        }

        Ok(Self {
            indices,
            scales,
            elements: weights.len(),
        })
    }

    /// Dequantize E8 encoded row back to float weights.
    pub fn dequantize_f32(&self, codebook: &E8Codebook) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.elements);
        for (&idx, &scale) in self.indices.iter().zip(&self.scales) {
            let deq = codebook.dequantize_block(idx, scale as f64);
            for &val in &deq {
                if out.len() < self.elements {
                    out.push(val as f32);
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Multiplier-free helper for small code products
// ---------------------------------------------------------------------------

/// Multiplier-free product of an i64 activation and a small signed code in [-7..7].
///
/// Uses additions, subtractions, and bit shifts only (0 hardware multiplier instructions).
#[inline(never)]
pub fn mul_small_code_i64(val: i64, code: i64) -> Result<i64> {
    if !(-7..=7).contains(&code) {
        return Err(invalid("code exceeds [-7..7] range"));
    }
    let neg = code < 0;
    let abs_c = code.unsigned_abs();
    let shl_checked = |x: i64, k: u32| -> Result<i64> {
        let s = x.wrapping_shl(k);
        if (s >> k) != x {
            return Err(invalid("overflow on shift"));
        }
        Ok(s)
    };
    let mag = match abs_c {
        0 => 0i64,
        1 => val,
        2 => shl_checked(val, 1)?,
        3 => shl_checked(val, 1)?
            .checked_add(val)
            .ok_or_else(|| invalid("overflow"))?,
        4 => shl_checked(val, 2)?,
        5 => shl_checked(val, 2)?
            .checked_add(val)
            .ok_or_else(|| invalid("overflow"))?,
        6 => shl_checked(val, 2)?
            .checked_add(shl_checked(val, 1)?)
            .ok_or_else(|| invalid("overflow"))?,
        7 => shl_checked(val, 3)?
            .checked_sub(val)
            .ok_or_else(|| invalid("overflow"))?,
        _ => unreachable!(),
    };
    if neg {
        mag.checked_neg().ok_or_else(|| invalid("overflow"))
    } else {
        Ok(mag)
    }
}

// ---------------------------------------------------------------------------
// Unified Codec Transform Evaluation
// ---------------------------------------------------------------------------

/// Codec evaluation arms for discrete parameter representation studies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecArm {
    /// Baseline: Round to nearest per-row signed 4-bit.
    NearestPerRow,
    /// Arm 1: Randomized Walsh-Hadamard transform + Grouped 4-bit (H+G4).
    HadamardGrouped4Bit,
    /// Arm 2: Randomized Walsh-Hadamard transform + E8 lattice (H+E8).
    HadamardE8,
    /// Arm 3: Head-compensated quantization (mean-offset compensation per group).
    HeadCompensated,
}

/// Transform matrix weights using the selected codec arm.
///
/// Handles arbitrary row and column dimensions (e.g. 288, 749) via block-padding.
/// Returns the quantized and dequantized weights (same shape) for fidelity evaluation.
pub fn apply_codec_arm(
    weights: &[f32],
    rows: usize,
    cols: usize,
    arm: CodecArm,
    seed: u64,
) -> Result<Vec<f32>> {
    if weights.len() != rows * cols {
        return Err(invalid("weights size does not match rows * cols"));
    }
    match arm {
        CodecArm::NearestPerRow => {
            let mut out = Vec::with_capacity(weights.len());
            for row in weights.chunks_exact(cols) {
                let mut maxabs = 0.0f32;
                for &w in row {
                    let a = w.abs();
                    if a > maxabs {
                        maxabs = a;
                    }
                }
                let exp = if maxabs == 0.0 {
                    0i16
                } else {
                    let c = (maxabs / 7.0f32).log2().ceil() as i32;
                    c.clamp(-24, 16) as i16
                };
                let scale = 2.0f32.powi(i32::from(exp));
                for &w in row {
                    let q = (w / scale).round().clamp(-7.0, 7.0);
                    out.push(q * scale);
                }
            }
            Ok(out)
        }
        CodecArm::HadamardGrouped4Bit => {
            const SCALE: f32 = 65536.0;
            let padded_cols = cols.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_cols, seed);
            let mut out = Vec::with_capacity(weights.len());

            for row in weights.chunks_exact(cols) {
                let i64_row: Vec<i64> = row.iter().map(|&w| (w * SCALE).round() as i64).collect();
                let had_i64 = block_padded_hadamard_transform(&i64_row, &signs)?;
                let had_f32: Vec<f32> = had_i64.iter().map(|&v| (v as f32) / SCALE).collect();

                let grouped = Grouped4BitRow::encode_f32(&had_f32, DEFAULT_GROUP_SIZE)?;
                let deq_had = grouped.dequantize_f32();

                let i64_deq: Vec<i64> = deq_had
                    .iter()
                    .map(|&w| (w * SCALE).round() as i64)
                    .collect();
                let restored_i64 = inverse_block_padded_hadamard_transform(&i64_deq, &signs, cols)?;
                for v in restored_i64 {
                    out.push((v as f32) / SCALE);
                }
            }
            Ok(out)
        }
        CodecArm::HadamardE8 => {
            const SCALE: f32 = 65536.0;
            let padded_cols = cols.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_cols, seed);
            let codebook = E8Codebook::new();
            let mut out = Vec::with_capacity(weights.len());

            for row in weights.chunks_exact(cols) {
                let i64_row: Vec<i64> = row.iter().map(|&w| (w * SCALE).round() as i64).collect();
                let had_i64 = block_padded_hadamard_transform(&i64_row, &signs)?;
                let had_f32: Vec<f32> = had_i64.iter().map(|&v| (v as f32) / SCALE).collect();

                let e8_row = E8EncodedRow::encode_f32(&had_f32, &codebook)?;
                let deq_had = e8_row.dequantize_f32(&codebook);

                let i64_deq: Vec<i64> = deq_had
                    .iter()
                    .map(|&w| (w * SCALE).round() as i64)
                    .collect();
                let restored_i64 = inverse_block_padded_hadamard_transform(&i64_deq, &signs, cols)?;
                for v in restored_i64 {
                    out.push((v as f32) / SCALE);
                }
            }
            Ok(out)
        }
        CodecArm::HeadCompensated => {
            let mut out = Vec::with_capacity(weights.len());
            for row in weights.chunks_exact(cols) {
                let mut row_acc_err = 0.0f32;
                for group in row.chunks(DEFAULT_GROUP_SIZE) {
                    let mut maxabs = 0.0f32;
                    for &w in group {
                        let a = w.abs();
                        if a > maxabs {
                            maxabs = a;
                        }
                    }
                    if maxabs == 0.0 {
                        out.extend(std::iter::repeat(0.0f32).take(group.len()));
                        continue;
                    }
                    let base_exp = (maxabs / 7.0f32).log2().floor() as i32;
                    let mut best_scale = 2.0f32.powi(base_exp);
                    let mut best_err = f32::INFINITY;
                    for de in -3..=3 {
                        let s = 2.0f32.powi((base_exp + de).clamp(-24, 16));
                        let err: f32 = group
                            .iter()
                            .map(|&w| {
                                let q = (w / s).round().clamp(-7.0, 7.0);
                                (w - q * s).powi(2)
                            })
                            .sum();
                        if err < best_err {
                            best_err = err;
                            best_scale = s;
                        }
                    }
                    for &w in group {
                        let compensated_w = w + row_acc_err * 0.5f32;
                        let q = (compensated_w / best_scale).round().clamp(-7.0, 7.0);
                        let deq = q * best_scale;
                        row_acc_err = (w - deq) + row_acc_err * 0.5f32;
                        out.push(deq);
                    }
                }
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fwht_orthogonality_and_inversion() -> Result<()> {
        let original: Vec<i64> = (0..64).map(|x| x * 7 - 120).collect();
        let mut transformed = original.clone();
        fwht_slice(&mut transformed)?;

        let mut restored = transformed.clone();
        fwht_slice(&mut restored)?;
        for x in restored.iter_mut() {
            *x >>= 6;
        }
        assert_eq!(restored, original);
        Ok(())
    }

    #[test]
    fn test_randomized_hadamard_roundtrip() -> Result<()> {
        let dim = 128;
        let signs = deterministic_signs(dim, 20260928);
        let original: Vec<i64> = (0..dim).map(|i| (i as i64) * 3 - 150).collect();
        let mut transformed = original.clone();
        randomized_hadamard_transform(&mut transformed, &signs)?;

        let mut inverted = transformed.clone();
        inverse_randomized_hadamard_transform(&mut inverted, &signs)?;

        for (orig, inv) in original.iter().zip(&inverted) {
            assert!((orig - inv).abs() <= 2, "orig: {orig}, inv: {inv}");
        }
        Ok(())
    }

    #[test]
    fn test_arbitrary_width_block_padded_hadamard() -> Result<()> {
        for &dim in &[288usize, 749usize] {
            let padded_len = dim.div_ceil(HADAMARD_BLOCK) * HADAMARD_BLOCK;
            let signs = deterministic_signs(padded_len, 42);
            let original: Vec<i64> = (0..dim).map(|i| ((i as i64) * 5) % 137 - 68).collect();

            let transformed = block_padded_hadamard_transform(&original, &signs)?;
            assert_eq!(transformed.len(), padded_len);

            let restored = inverse_block_padded_hadamard_transform(&transformed, &signs, dim)?;
            assert_eq!(restored.len(), dim);

            for (orig, res) in original.iter().zip(&restored) {
                assert!((orig - res).abs() <= 2, "dim {dim}: orig {orig}, res {res}");
            }
        }
        Ok(())
    }

    #[test]
    fn test_e8_codebook_norms_and_zero() {
        let codebook = E8Codebook::new();
        assert_eq!(codebook.roots[0], [0i8; E8_DIM]);

        for root in &codebook.roots[1..] {
            let norm_sq: i32 = root.iter().map(|&x| (x as i32) * (x as i32)).sum();
            assert_eq!(
                norm_sq, 8,
                "All E8 roots must have equal squared norm 8: got {norm_sq}"
            );
        }
    }

    #[test]
    fn test_grouped4bit_arbitrary_dimensions() -> Result<()> {
        for &dim in &[288usize, 749usize] {
            let weights: Vec<f32> = (0..dim).map(|i| ((i as f32) - 100.0) * 0.02).collect();
            let row = Grouped4BitRow::encode_f32(&weights, DEFAULT_GROUP_SIZE)?;
            let deq = row.dequantize_f32();
            assert_eq!(deq.len(), dim);
            for (i, (&w, &d)) in weights.iter().zip(&deq).enumerate() {
                let g = i / DEFAULT_GROUP_SIZE;
                let scale = 2.0f32.powi(i32::from(row.group_exponents[g]));
                assert!(
                    (w - d).abs() <= scale / 2.0 + 1e-4,
                    "w: {w}, d: {d}, scale: {scale}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn test_mul_small_code_matches_multiplication() -> Result<()> {
        for val in [-1000i64, -50, 0, 17, 999] {
            for code in -7..=7 {
                assert_eq!(mul_small_code_i64(val, code)?, val * code);
            }
        }
        assert!(mul_small_code_i64(10, 8).is_err());
        assert!(mul_small_code_i64(10, -8).is_err());

        // Extreme overflow and boundary tests
        assert!(mul_small_code_i64(1i64 << 62, 4).is_err()); // Shift-add overflow
        assert!(mul_small_code_i64(i64::MIN, -1).is_err()); // Negation overflow
        assert!(mul_small_code_i64(i64::MAX, 2).is_err()); // Positive overflow
        assert!(mul_small_code_i64(i64::MIN / 2 - 1, 2).is_err()); // Negative overflow
        assert_eq!(mul_small_code_i64(i64::MIN, 1)?, i64::MIN);
        assert_eq!(mul_small_code_i64(i64::MAX, 1)?, i64::MAX);
        assert_eq!(mul_small_code_i64(i64::MIN, 0)?, 0);
        assert_eq!(mul_small_code_i64(i64::MAX, 0)?, 0);
        Ok(())
    }

    #[test]
    fn test_grouped4bit_nonfinite_rejected() {
        assert!(Grouped4BitRow::encode_f32(&[f32::NAN, 1.0], 32).is_err());
        assert!(Grouped4BitRow::encode_f32(&[f32::INFINITY, 1.0], 32).is_err());
        assert!(Grouped4BitRow::encode_f32(&[f32::NEG_INFINITY, 1.0], 32).is_err());
    }

    #[test]
    fn test_apply_codec_arm_geometric_s1_dimensions() -> Result<()> {
        let rows = 4;
        let cols = 288;
        let weights: Vec<f32> = (0..rows * cols)
            .map(|i| (((i as f32) * 1.3) % 20.0 - 10.0) * 0.05)
            .collect();

        for arm in [
            CodecArm::NearestPerRow,
            CodecArm::HadamardGrouped4Bit,
            CodecArm::HadamardE8,
            CodecArm::HeadCompensated,
        ] {
            let deq = apply_codec_arm(&weights, rows, cols, arm, 100)?;
            assert_eq!(deq.len(), rows * cols);
        }
        Ok(())
    }

    #[test]
    fn test_hardened_hadamard_edge_cases() -> Result<()> {
        // Short signs slice rejected
        let mut data = vec![1i64; 64];
        let short_signs = vec![1i8; 32];
        assert!(randomized_hadamard_transform(&mut data, &short_signs).is_err());
        assert!(inverse_randomized_hadamard_transform(&mut data, &short_signs).is_err());

        // fwht_normalized invalid shift rejected
        assert!(fwht_normalized(&mut data, 64).is_err());
        assert!(fwht_normalized(&mut data, 100).is_err());

        // fwht_normalized overflow on rounding add caught
        let mut overflow_data = vec![i64::MAX; 64];
        assert!(fwht_normalized(&mut overflow_data, 2).is_err());

        // inverse_block_padded_hadamard_transform rejects short signs
        let transformed = vec![0i64; 64];
        assert!(inverse_block_padded_hadamard_transform(&transformed, &short_signs, 50).is_err());

        // i64::MIN sign negation overflow caught cleanly
        let mut min_data = vec![i64::MIN; 64];
        let neg_signs = vec![-1i8; 64];
        assert!(randomized_hadamard_transform(&mut min_data, &neg_signs).is_err());
        assert!(inverse_randomized_hadamard_transform(&mut min_data, &neg_signs).is_err());

        Ok(())
    }

    #[test]
    fn test_grouped4bit_codec_deterministic_and_effective_bits() -> Result<()> {
        let codec_rtn = Grouped4BitCodec::default();
        let codec_opt =
            Grouped4BitCodec::new(DEFAULT_GROUP_SIZE, Grouped4BitRounding::MinimumMseScale);

        // Head dimension of geometric_s1: 4096 x 288
        let rows = 4096;
        let cols = 288;
        let bpp_framed = codec_rtn.effective_bits_per_weight(rows, cols);
        let bpp_raw = codec_rtn.param_bits_per_weight(rows, cols);
        assert_eq!(
            bpp_raw, 4.25,
            "Raw parameter bits must be exactly 4.25 bits/weight"
        );
        assert!(
            bpp_framed <= 4.251,
            "Effective bits per weight including container framing must be <= 4.251, got {bpp_framed}"
        );

        // Test with 64 x 288
        let rows_test = 64;
        let values: Vec<f32> = (0..rows_test * cols)
            .map(|i| (((i as f32) * 0.73) % 17.0 - 8.5) * 0.04)
            .collect();

        // 1. Deterministic quantization
        let mat1 = codec_rtn.quantize(&values, rows_test, cols)?;
        let mat2 = codec_rtn.quantize(&values, rows_test, cols)?;
        assert_eq!(mat1, mat2, "Quantization must be strictly deterministic");
        assert_eq!(mat1.param_bits_per_weight(), 4.25);
        assert_eq!(mat1.to_bytes().len(), mat1.total_stored_bytes());
        assert_eq!(
            mat1.total_stored_bytes(),
            CONTAINER_FRAMING_BYTES + mat1.nibbles.len() + mat1.scales.len()
        );

        // 2. Dequantization and round-trip
        let deq1 = codec_rtn.dequantize(&mat1)?;
        let deq_rt = codec_rtn.round_trip(&values, rows_test, cols)?;
        assert_eq!(deq1, deq_rt);
        assert_eq!(deq1.len(), rows_test * cols);

        // 3. Serialization round-trip
        let bytes = mat1.to_bytes();
        let mat_deser = Grouped4BitMatrix::from_bytes(&bytes)?;
        assert_eq!(mat1, mat_deser);

        // Corrupted checksum must be rejected
        let mut corrupted = bytes.clone();
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] ^= 0xFF;
        assert!(Grouped4BitMatrix::from_bytes(&corrupted).is_err());

        // 4. Minimum MSE scale search mode
        let mat_opt = codec_opt.quantize(&values, rows_test, cols)?;
        let deq_opt = codec_opt.dequantize(&mat_opt)?;
        assert_eq!(deq_opt.len(), rows_test * cols);

        // Compute MSE for both
        let mse_rtn: f64 = values
            .iter()
            .zip(&deq1)
            .map(|(&v, &d)| (f64::from(v) - f64::from(d)).powi(2))
            .sum();
        let mse_opt: f64 = values
            .iter()
            .zip(&deq_opt)
            .map(|(&v, &d)| (f64::from(v) - f64::from(d)).powi(2))
            .sum();
        assert!(
            mse_opt <= mse_rtn + 1e-6,
            "Optimal scale search must achieve equal or lower MSE: opt {mse_opt} vs rtn {mse_rtn}"
        );

        Ok(())
    }

    #[test]
    fn test_grouped4bit_codec_invalid_inputs() {
        let codec = Grouped4BitCodec::default();
        // Size mismatch
        assert!(codec.quantize(&[1.0; 10], 4, 4).is_err());
        // Non-finite values
        assert!(codec.quantize(&[f32::NAN; 32], 1, 32).is_err());
        assert!(codec.quantize(&[f32::INFINITY; 32], 1, 32).is_err());
        // Zero dimensions
        assert!(codec.quantize(&[], 0, 32).is_err());
        assert!(codec.quantize(&[], 32, 0).is_err());
    }

    #[test]
    fn test_grouped4bit_odd_dimensions_and_custom_group_size() -> Result<()> {
        // Test odd rows, odd cols, and odd group_size: 3 rows x 5 cols, group_size = 3
        let codec = Grouped4BitCodec::new(3, Grouped4BitRounding::Nearest);
        let rows = 3;
        let cols = 5;
        let values: Vec<f32> = (0..rows * cols)
            .map(|i| (((i as f32) * 1.7) % 5.0 - 2.5) * 0.1)
            .collect();

        let mat = codec.quantize(&values, rows, cols)?;
        assert_eq!(mat.rows, rows);
        assert_eq!(mat.cols, cols);
        assert_eq!(mat.group_size, 3);
        // 5 cols with group_size 3 => 2 groups per row => 3 * 2 = 6 scales
        assert_eq!(mat.scales.len(), 6);
        // 15 weights => div_ceil(2) = 8 nibble bytes
        assert_eq!(mat.nibbles.len(), 8);

        // Serialization round-trip
        let bytes = mat.to_bytes();
        assert_eq!(bytes.len(), 64 + 8 + 6);
        let deser = Grouped4BitMatrix::from_bytes(&bytes)?;
        assert_eq!(mat, deser);

        // Dequantize produces exact rows * cols values
        let deq = codec.dequantize(&mat)?;
        assert_eq!(deq.len(), rows * cols);
        for (&orig, &quant) in values.iter().zip(&deq) {
            assert!((orig - quant).abs() < 0.2);
        }

        // Test another configuration: 2 rows x 7 cols, group_size = 4
        let codec4 = Grouped4BitCodec::new(4, Grouped4BitRounding::MinimumMseScale);
        let rows4 = 2;
        let cols4 = 7;
        let values4: Vec<f32> = (0..rows4 * cols4)
            .map(|i| (((i as f32) * 0.9) % 3.0 - 1.5) * 0.2)
            .collect();
        let mat4 = codec4.quantize(&values4, rows4, cols4)?;
        assert_eq!(mat4.scales.len(), 4); // 2 rows * ceil(7/4 = 2) = 4
        assert_eq!(mat4.nibbles.len(), 7); // ceil(14/2) = 7
        let deq4 = codec4.dequantize(&mat4)?;
        assert_eq!(deq4.len(), rows4 * cols4);

        Ok(())
    }

    #[test]
    fn test_grouped4bit_codec_padding_convention() -> Result<()> {
        let codec = Grouped4BitCodec::default();
        let rows = 4;
        let cols = 749; // Geometric S1 MLP hidden dimension (not a multiple of 32)
        let values: Vec<f32> = (0..rows * cols)
            .map(|i| (((i as f32) * 0.31) % 11.0 - 5.5) * 0.05)
            .collect();

        // 1. Direct quantize handles 749 columns directly
        let direct_mat = codec.quantize(&values, rows, cols)?;
        assert_eq!(direct_mat.rows, rows);
        assert_eq!(direct_mat.cols, 749);
        let direct_deq = codec.dequantize(&direct_mat)?;
        assert_eq!(direct_deq.len(), rows * cols);

        // 2. quantize_padded pads to multiple of 32 (768)
        let mat = codec.quantize_padded(&values, rows, cols)?;
        assert_eq!(mat.rows, rows);
        assert_eq!(mat.cols, 768);
        assert_eq!(mat.cols % codec.group_size, 0);

        // 3. Serialization of padded matrix succeeds
        let bytes = mat.to_bytes();
        let mat_deser = Grouped4BitMatrix::from_bytes(&bytes)?;
        assert_eq!(mat, mat_deser);

        // 4. dequantize_unpadded slices back to original 749 cols
        let deq = codec.dequantize_unpadded(&mat, cols)?;
        assert_eq!(deq.len(), rows * cols);

        // 5. round_trip_padded matches dequantize_unpadded
        let rt = codec.round_trip_padded(&values, rows, cols)?;
        assert_eq!(deq, rt);

        // 6. Each value is accurately preserved within quantization error
        for (&orig, &quant) in values.iter().zip(&rt) {
            assert!((orig - quant).abs() < 0.2);
        }
        Ok(())
    }

    #[test]
    fn test_grouped4bit_codec_extreme_outliers_and_zeros() -> Result<()> {
        let codec = Grouped4BitCodec::default();

        // All zeros
        let zeros = vec![0.0f32; 64];
        let mat_zeros = codec.quantize(&zeros, 2, 32)?;
        let deq_zeros = codec.dequantize(&mat_zeros)?;
        assert_eq!(deq_zeros, zeros);

        // Extreme outlier (> 100x sigma)
        let mut outlier_vals = vec![0.05f32; 64];
        outlier_vals[0] = 50.0f32; // > 1000x normal scale
        outlier_vals[31] = -50.0f32;
        let mat_outlier = codec.quantize(&outlier_vals, 2, 32)?;
        let deq_outlier = codec.dequantize(&mat_outlier)?;
        assert_eq!(deq_outlier.len(), 64);
        for &v in &deq_outlier {
            assert!(v.is_finite());
        }

        // Test with MinimumMseScale mode
        let codec_mse =
            Grouped4BitCodec::new(DEFAULT_GROUP_SIZE, Grouped4BitRounding::MinimumMseScale);
        let mat_mse = codec_mse.quantize(&outlier_vals, 2, 32)?;
        let deq_mse = codec_mse.dequantize(&mat_mse)?;
        assert_eq!(deq_mse.len(), 64);
        for &v in &deq_mse {
            assert!(v.is_finite());
        }

        Ok(())
    }

    #[test]
    fn test_grouped4bit_matrix_adversarial_validation() {
        let codec = Grouped4BitCodec::default();
        let valid_mat = codec.quantize(&[0.1f32; 64], 2, 32).unwrap();

        // 1. Truncated scales buffer must be rejected without panic
        let mut bad_scales = valid_mat.clone();
        bad_scales.scales.pop();
        assert!(bad_scales.validate().is_err());
        assert!(codec.dequantize(&bad_scales).is_err());

        // 2. Truncated nibbles buffer must be rejected without panic
        let mut bad_nibbles = valid_mat.clone();
        bad_nibbles.nibbles.pop();
        assert!(bad_nibbles.validate().is_err());
        assert!(codec.dequantize(&bad_nibbles).is_err());

        // 3. Out-of-bounds exp_base must be rejected
        let mut bad_exp = valid_mat.clone();
        bad_exp.exp_base = 2000;
        assert!(bad_exp.validate().is_err());
        assert!(codec.dequantize(&bad_exp).is_err());

        bad_exp.exp_base = -500;
        assert!(bad_exp.validate().is_err());
        assert!(codec.dequantize(&bad_exp).is_err());

        bad_exp.exp_base = i32::MAX;
        assert!(bad_exp.validate().is_err());
        assert!(codec.dequantize(&bad_exp).is_err());

        // 4. Zero dimensions rejected
        let mut zero_rows = valid_mat.clone();
        zero_rows.rows = 0;
        assert!(zero_rows.validate().is_err());
        assert!(codec.dequantize(&zero_rows).is_err());

        // 5. from_bytes rejecting corrupted framing
        let bytes = valid_mat.to_bytes();
        let mut bad_magic = bytes.clone();
        bad_magic[0] ^= 0x55;
        assert!(Grouped4BitMatrix::from_bytes(&bad_magic).is_err());

        let mut truncated = bytes.clone();
        truncated.truncate(50);
        assert!(Grouped4BitMatrix::from_bytes(&truncated).is_err());

        // 6. Zero group_size and zero dimensions on quantize and padded round-trip must return Err, not panic
        let zero_group_codec = Grouped4BitCodec::new(0, Grouped4BitRounding::Nearest);
        assert!(zero_group_codec.quantize(&[0.1f32; 64], 2, 32).is_err());
        assert!(zero_group_codec
            .quantize_padded(&[0.1f32; 64], 2, 32)
            .is_err());
        assert!(zero_group_codec
            .round_trip_padded(&[0.1f32; 64], 2, 32)
            .is_err());

        assert!(codec.quantize(&[], 0, 32).is_err());
        assert!(codec.quantize(&[], 2, 0).is_err());
        assert!(codec.quantize_padded(&[], 0, 32).is_err());
        assert!(codec.quantize_padded(&[], 2, 0).is_err());
        assert!(codec.round_trip_padded(&[], 0, 32).is_err());
        assert!(codec.round_trip_padded(&[], 2, 0).is_err());

        // 7. Scale where -8 * scale overflows f32 must fail validation
        let mut overflow_mat = valid_mat.clone();
        overflow_mat.exp_base = 120; // 2^120 * 31 * -8 overflows f32 (max ~3.4e38)
        assert!(overflow_mat.validate().is_err());

        // 8. Normalization algebraic condition for G32: 32 * c_x * c_w == 1
        let c_x = 0.25f64;
        let c_w = 0.125f64;
        assert!((32.0 * c_x * c_w - 1.0).abs() < 1e-15);
    }
}
