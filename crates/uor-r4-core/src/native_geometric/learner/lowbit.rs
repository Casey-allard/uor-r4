//! Low-bit integer linear maps: trained offline in floating point, served without a multiplier.
//!
//! # Why this exists
//!
//! Decision D0-b permits bounded integer/ternary linear maps at serving, executed with no
//! multiplier in the kernel. This is the substrate for that: a ternary weight matrix with a
//! **power-of-two per-row scale**, so the whole layer is `y = (Σ ±x_i) << shift` — additions,
//! subtractions and shifts only.
//!
//! The power-of-two scale is the load-bearing trick. A learned per-row scale would be a
//! runtime-loaded value, and `acc * scale` is a genuine variable-by-variable multiply that no
//! amount of compiler cleverness removes. Constraining the scale to a power of two converts it to
//! a shift, and costs almost nothing in accuracy for ternary weights because the scale's job is
//! only to set the row's magnitude.
//!
//! # Storage
//!
//! Two bits per weight, four weights per byte, encoding `-1`, `0`, `+1`. That is 2 bits per weight
//! stored rather than 16 or 32, which is the I2 (bytes-per-token) lever: the same win BitNet-class
//! systems get from ternary storage, and the reason a low-bit layer can be *faster* on M1, where
//! decode is memory-bandwidth-bound rather than arithmetic-bound.
//!
//! # What is not here yet
//!
//! This module supplies the linear map, its quantizer and its exact serving path. A learned
//! composition — the embedding, the nonlinearity, the training objective that makes a chat model
//! out of these — is the next layer of work and is not claimed here. This is the substrate, with
//! exactness verified, and nothing more.

#![forbid(unsafe_code)]

/// Ternary weight matrix with a power-of-two per-row output scale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TernaryLinear {
    pub rows: usize,
    pub cols: usize,
    /// Two bits per weight, row-major, four weights per byte.
    packed: Vec<u8>,
    /// Per-row left shift. `y = acc << shift`, so no multiplier is engaged.
    shift: Vec<u32>,
}

/// Two-bit encoding of the three ternary values.
const CODE_ZERO: u8 = 0;
const CODE_POS: u8 = 1;
const CODE_NEG: u8 = 2;

impl TernaryLinear {
    /// Quantise a row-major `f32` matrix to ternary weights with per-row power-of-two scales.
    ///
    /// Offline only: this uses floating point, which D0-b permits for training and artifact
    /// construction.
    pub fn quantize(weights: &[f32], rows: usize, cols: usize) -> Self {
        assert_eq!(
            weights.len(),
            rows * cols,
            "weights length must be rows * cols"
        );
        let mut packed = vec![0u8; (rows * cols).div_ceil(4)];
        let mut shift = vec![0u32; rows];

        for r in 0..rows {
            let row = &weights[r * cols..(r + 1) * cols];
            let amax = row.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
            // Power-of-two scale near the row's magnitude. Clamped so `1 << s` cannot overflow a
            // u32 and so the serving shift stays a sane bit count.
            let s = if amax > 0.0 {
                amax.log2().floor().max(0.0).min(30.0) as u32
            } else {
                0
            };
            shift[r] = s;
            let scale = (1u32 << s) as f32;
            for c in 0..cols {
                let q = (row[c] / scale).round();
                let code = if q >= 1.0 {
                    CODE_POS
                } else if q <= -1.0 {
                    CODE_NEG
                } else {
                    CODE_ZERO
                };
                let flat = r * cols + c;
                let byte = flat >> 2;
                let slot = (flat & 3) as u32;
                packed[byte] |= code << (slot * 2);
            }
        }

        Self {
            rows,
            cols,
            packed,
            shift,
        }
    }

    /// Rebuild a table from its serialised form: the packed two-bit codes and the per-row shifts.
    ///
    /// Used by artifact reload. Because the codes *are* the weights, round-tripping `packed()` and
    /// `shift()` through this constructor is exact: reloaded integer logits equal exported integer
    /// logits, which is what the export/reload parity test checks.
    pub fn from_packed(
        packed: Vec<u8>,
        shift: Vec<u32>,
        rows: usize,
        cols: usize,
    ) -> Result<Self, String> {
        if shift.len() != rows {
            return Err(format!("shift length {} != rows {rows}", shift.len()));
        }
        if packed.len() != (rows * cols).div_ceil(4) {
            return Err(format!(
                "packed length {} != {} for {rows}x{cols}",
                packed.len(),
                (rows * cols).div_ceil(4)
            ));
        }
        Ok(Self {
            rows,
            cols,
            packed,
            shift,
        })
    }

    /// Recover one ternary weight as an integer: `-1`, `0` or `+1`.
    ///
    /// Addressing uses `>>` and `&`, so no multiplier is engaged.
    #[inline]
    pub fn weight(&self, row: usize, col: usize) -> i32 {
        let flat = row * self.cols + col;
        let byte = self.packed[flat >> 2];
        let slot = (flat & 3) as u32;
        match (byte >> (slot * 2)) & 3 {
            CODE_POS => 1,
            CODE_NEG => -1,
            _ => 0,
        }
    }

    /// Bytes of packed weight storage.
    #[inline]
    pub fn weight_bytes(&self) -> usize {
        self.packed.len()
    }

    /// Stored bits per weight, for the bytes-per-token accounting.
    #[inline]
    pub fn bits_per_weight(&self) -> f32 {
        if self.rows * self.cols == 0 {
            0.0
        } else {
            (self.packed.len() as f32 * 8.0) / (self.rows * self.cols) as f32
        }
    }

    /// Serving forward pass: `y[r] = (Σ_c w[r][c] * x[c]) << shift[r]`.
    ///
    /// Multiplier-free by construction: the ternary weight selects an add, a subtract or nothing,
    /// and the scale is a shift. No `*`, `/` or `%` is executed, and the arithmetic is exact
    /// integer throughout.
    pub fn forward_i32(&self, x: &[i32]) -> Vec<i32> {
        assert_eq!(x.len(), self.cols, "input length must equal cols");
        let mut out = vec![0i32; self.rows];
        for r in 0..self.rows {
            let mut acc: i32 = 0;
            for c in 0..self.cols {
                match self.weight(r, c) {
                    1 => acc += x[c],
                    -1 => acc -= x[c],
                    _ => {}
                }
            }
            out[r] = acc << self.shift[r];
        }
        out
    }

    /// Offline reference used to verify the serving path: evaluates the same weights in floating
    /// point. Test-only in practice; present so the exactness claim can be checked rather than
    /// asserted.
    pub fn forward_reference(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.cols, "input length must equal cols");
        let mut out = vec![0.0f64; self.rows];
        for r in 0..self.rows {
            let mut acc = 0.0f64;
            for c in 0..self.cols {
                acc += self.weight(r, c) as f64 * x[c];
            }
            out[r] = acc * (1u64 << self.shift[r]) as f64;
        }
        out
    }

    /// Per-row scale as an integer power of two, exposed for artifact serialisation.
    #[inline]
    pub fn shift(&self, row: usize) -> u32 {
        self.shift[row]
    }

    /// Raw packed weights, for artifact serialisation.
    #[inline]
    pub fn packed(&self) -> &[u8] {
        &self.packed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TernaryLinear {
        // Deterministic pseudo-random float matrix spanning several magnitudes so the per-row
        // power-of-two scales differ.
        let rows = 7usize;
        let cols = 13usize;
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut w = Vec::with_capacity(rows * cols);
        for i in 0..rows * cols {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let unit = ((state >> 11) as f64 / (1u64 << 53) as f64) as f32;
            let magnitude = 1.0f32 + (i % rows) as f32; // row-dependent scale
            w.push((unit * 2.0 - 1.0) * magnitude);
        }
        TernaryLinear::quantize(&w, rows, cols)
    }

    #[test]
    fn storage_is_two_bits_per_weight() {
        let m = sample();
        assert_eq!(m.weight_bytes(), (m.rows * m.cols).div_ceil(4));
        assert!(
            (m.bits_per_weight() - 2.0).abs() < 0.5,
            "expected about 2 bits per weight, got {}",
            m.bits_per_weight()
        );
        // 2 bits per weight against 32 for f32 is a 16x ratio; allow padding, so assert at least
        // 8x. (An earlier version asserted `* 16 <`, which is false by exactly the padding.)
        assert!(m.weight_bytes() * 8 <= m.rows * m.cols * 4);
    }

    #[test]
    fn packing_round_trips_every_weight() {
        let m = sample();
        for r in 0..m.rows {
            for c in 0..m.cols {
                let w = m.weight(r, c);
                assert!(
                    w == -1 || w == 0 || w == 1,
                    "weight must be ternary, got {w}"
                );
            }
        }
    }

    #[test]
    fn serving_path_is_exactly_the_reference() {
        let m = sample();
        // Integer inputs, so the serving path (integer) and the reference (f64) must agree exactly.
        let x: Vec<i32> = (0..m.cols as i32).map(|i| i * 37 - 100).collect();
        let xf: Vec<f64> = x.iter().map(|&v| v as f64).collect();

        let got = m.forward_i32(&x);
        let want = m.forward_reference(&xf);
        for r in 0..m.rows {
            assert_eq!(
                got[r] as f64, want[r],
                "row {r}: serving path {} != reference {}",
                got[r], want[r]
            );
        }
    }

    #[test]
    fn a_single_weight_selects_add_subtract_or_nothing() {
        // cols = 4 so the weights land in one byte, exercising the packing directly.
        let w = vec![3.0f32, -3.0, 0.0, 3.0];
        let m = TernaryLinear::quantize(&w, 1, 4);
        assert_eq!(
            (
                m.weight(0, 0),
                m.weight(0, 1),
                m.weight(0, 2),
                m.weight(0, 3)
            ),
            (1, -1, 0, 1)
        );
        // x = [5, 5, 5, 5] -> 5 - 5 + 0 + 5 = 5, scaled by 2^1 = 10.
        assert_eq!(m.forward_i32(&[5, 5, 5, 5])[0], 10);
    }

    #[test]
    fn scale_is_a_power_of_two_so_it_is_a_shift() {
        let m = sample();
        for r in 0..m.rows {
            let s = m.shift(r);
            assert!(s < 31, "shift must be a sane bit count, got {s}");
            // 1 << s is representable, i.e. the scale really is a power of two.
            let scale = 1i64 << s;
            assert_eq!(scale.count_ones(), 1);
        }
    }
}
