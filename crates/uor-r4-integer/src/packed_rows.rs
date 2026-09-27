//! Immutable coefficient storage for the retained numerical kernels.
//!
//! Signed4 codes keep the artifact's low-nibble-first representation. A clone
//! shares the allocation; kernels read packed rows directly, without a decoded
//! row or a second expanded weight array. This is dense access, not routing.

use crate::{invalid, Result};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) enum CoefficientCodes {
    Signed4 { bytes: Arc<[u8]>, elements: usize },
    Signed16(Arc<[i16]>),
}

impl CoefficientCodes {
    /// The public artifact decoder remains unchanged. Compile its validated
    /// output once, then release the expanded signed4 allocation.
    pub(crate) fn from_decoded(codes: Vec<i16>, bits: u8) -> Result<Self> {
        match bits {
            4 => {
                let elements = codes.len();
                let mut bytes = Vec::with_capacity(elements.div_ceil(2));
                for pair in codes.chunks(2) {
                    if pair.iter().any(|&code| !(-7..=7).contains(&code)) {
                        return Err(invalid("packed signed4 code outside [-7,7]"));
                    }
                    let low = (pair[0] as u8) & 15;
                    let high = pair.get(1).map_or(0, |&code| (code as u8) & 15);
                    bytes.push(low | (high << 4));
                }
                Ok(Self::Signed4 {
                    bytes: bytes.into(),
                    elements,
                })
            }
            16 => {
                if codes.contains(&i16::MIN) {
                    return Err(invalid("reserved signed16 coefficient"));
                }
                Ok(Self::Signed16(codes.into()))
            }
            _ => Err(invalid("unsupported coefficient bit width")),
        }
    }

    /// Construction for the existing deterministic synthetic model. No decode
    /// buffer is needed, including for its large all-one weight matrices.
    pub(crate) fn ones(elements: usize, bits: u8) -> Self {
        if bits == 4 {
            let mut bytes = vec![0x11; elements.div_ceil(2)];
            if elements & 1 != 0 {
                if let Some(last) = bytes.last_mut() {
                    *last = 1;
                }
            }
            Self::Signed4 {
                bytes: bytes.into(),
                elements,
            }
        } else {
            Self::Signed16(vec![1; elements].into())
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Signed4 { elements, .. } => *elements,
            Self::Signed16(codes) => codes.len(),
        }
    }

    pub(crate) fn payload_bytes(&self) -> usize {
        match self {
            Self::Signed4 { bytes, .. } => bytes.len(),
            Self::Signed16(codes) => codes.len() << 1,
        }
    }

    pub(crate) fn is_signed4(&self) -> bool {
        matches!(self, Self::Signed4 { .. })
    }

    pub(crate) fn code(&self, index: usize) -> Result<i16> {
        if index >= self.len() {
            return Err(invalid("coefficient index outside storage"));
        }
        Ok(self.code_in_bounds(index))
    }

    #[inline]
    fn code_in_bounds(&self, index: usize) -> i16 {
        match self {
            Self::Signed4 { bytes, .. } => {
                let nibble = (bytes[index >> 1] >> ((index & 1) << 2)) & 15;
                // Sign extension without a multiply or floating conversion.
                i16::from((nibble as i8) << 4 >> 4)
            }
            Self::Signed16(codes) => codes[index],
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = i16> + '_ {
        (0..self.len()).map(|index| self.code_in_bounds(index))
    }

    /// Matrix rows have even widths in both admitted profiles. Offsets and
    /// lengths are in coefficients, not bytes; scalar/vector access uses code.
    pub(crate) fn packed_range(&self, start: usize, elements: usize) -> Result<&[u8]> {
        let Self::Signed4 {
            bytes,
            elements: total,
        } = self
        else {
            return Err(invalid("packed row requires signed4 coefficients"));
        };
        let end = start
            .checked_add(elements)
            .ok_or_else(|| invalid("packed coefficient range overflow"))?;
        if (start | elements) & 1 != 0 || end > *total {
            return Err(invalid("packed coefficient row alignment or bounds"));
        }
        bytes
            .get((start >> 1)..(end >> 1))
            .ok_or_else(|| invalid("packed coefficient byte range"))
    }
}

/// A retained single row, with eight independent accumulators as before.
/// Callers validate the row and product lengths before entering this kernel.
#[inline(never)]
pub(crate) fn low_bit_dot(products: &[[i64; 16]], weights: &[u8]) -> i64 {
    let mut acc = [0i64; 8];
    for (p, w) in products.chunks_exact(8).zip(weights.chunks_exact(4)) {
        acc[0] += p[0][usize::from(w[0] & 15)];
        acc[1] += p[1][usize::from(w[0] >> 4)];
        acc[2] += p[2][usize::from(w[1] & 15)];
        acc[3] += p[3][usize::from(w[1] >> 4)];
        acc[4] += p[4][usize::from(w[2] & 15)];
        acc[5] += p[5][usize::from(w[2] >> 4)];
        acc[6] += p[6][usize::from(w[3] & 15)];
        acc[7] += p[7][usize::from(w[3] >> 4)];
    }
    // Admitted matrix widths (128,256,512,576,1152) have no remainder.
    (acc[0] + acc[1]) + (acc[2] + acc[3]) + (acc[4] + acc[5]) + (acc[6] + acc[7])
}

/// Eight adjacent width256 rows. Each byte supplies two table indices; no
/// decoded row is materialized and each input table serves all eight rows.
#[inline(always)]
pub(crate) fn low_bit_dot_8_contiguous(
    products: &[[i64; 16]; 256],
    weights: &[u8; 1024],
) -> (i64, i64, i64, i64, i64, i64, i64, i64) {
    let mut acc0 = 0i64;
    let mut acc1 = 0i64;
    let mut acc2 = 0i64;
    let mut acc3 = 0i64;
    let mut acc4 = 0i64;
    let mut acc5 = 0i64;
    let mut acc6 = 0i64;
    let mut acc7 = 0i64;
    for i in 0..128 {
        let p0 = &products[i << 1];
        let p1 = &products[(i << 1) + 1];
        let w0 = weights[i];
        let w1 = weights[128 + i];
        let w2 = weights[256 + i];
        let w3 = weights[384 + i];
        let w4 = weights[512 + i];
        let w5 = weights[640 + i];
        let w6 = weights[768 + i];
        let w7 = weights[896 + i];
        acc0 += p0[usize::from(w0 & 15)] + p1[usize::from(w0 >> 4)];
        acc1 += p0[usize::from(w1 & 15)] + p1[usize::from(w1 >> 4)];
        acc2 += p0[usize::from(w2 & 15)] + p1[usize::from(w2 >> 4)];
        acc3 += p0[usize::from(w3 & 15)] + p1[usize::from(w3 >> 4)];
        acc4 += p0[usize::from(w4 & 15)] + p1[usize::from(w4 >> 4)];
        acc5 += p0[usize::from(w5 & 15)] + p1[usize::from(w5 >> 4)];
        acc6 += p0[usize::from(w6 & 15)] + p1[usize::from(w6 >> 4)];
        acc7 += p0[usize::from(w7 & 15)] + p1[usize::from(w7 >> 4)];
    }
    (acc0, acc1, acc2, acc3, acc4, acc5, acc6, acc7)
}

/// Four adjacent width512 rows, retaining two accumulators per row and the
/// original four-coordinate grouping. Signed4 × full i32 ×512 fits in i64.
#[inline(always)]
pub(crate) fn low_bit_dot_4_contiguous_512(
    products: &[[i64; 16]; 512],
    weights: &[u8; 1024],
) -> (i64, i64, i64, i64) {
    let mut a = [0i64; 4];
    let mut b = [0i64; 4];
    for i in (0..256).step_by(2) {
        let p0 = &products[i << 1];
        let p1 = &products[(i << 1) + 1];
        let p2 = &products[(i << 1) + 2];
        let p3 = &products[(i << 1) + 3];
        for row in 0..4 {
            let offset = (row << 8) + i;
            let w0 = weights[offset];
            let w1 = weights[offset + 1];
            a[row] += p0[usize::from(w0 & 15)] + p1[usize::from(w0 >> 4)];
            b[row] += p2[usize::from(w1 & 15)] + p3[usize::from(w1 >> 4)];
        }
    }
    (a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_signed4_sign_order_bounds_and_shared_storage() -> Result<()> {
        let original: Vec<i16> = (-7..=7).collect();
        let codes = CoefficientCodes::from_decoded(original.clone(), 4)?;
        assert_eq!(codes.iter().collect::<Vec<_>>(), original);
        assert_eq!(codes.payload_bytes(), 8);
        let CoefficientCodes::Signed4 { bytes, .. } = &codes else {
            return Err(invalid("test expected packed storage"));
        };
        assert_eq!(&**bytes, &[0xa9, 0xcb, 0xed, 0x0f, 0x21, 0x43, 0x65, 0x07]);
        let cloned = codes.clone();
        let CoefficientCodes::Signed4 { bytes: shared, .. } = &cloned else {
            return Err(invalid("test expected shared packed storage"));
        };
        assert!(Arc::ptr_eq(bytes, shared));
        assert_eq!(codes.packed_range(2, 4)?, &[0xcb, 0xed]);
        assert!(codes.packed_range(1, 2).is_err());
        assert!(codes.packed_range(14, 2).is_err());
        assert!(codes.packed_range(usize::MAX - 1, 4).is_err());
        assert!(codes.code(15).is_err());
        for invalid_code in [-32768, -8, 8, 32767] {
            assert!(CoefficientCodes::from_decoded(vec![invalid_code], 4).is_err());
        }
        let wide = CoefficientCodes::from_decoded(vec![-32767, 0, 32767], 16)?;
        assert_eq!(wide.iter().collect::<Vec<_>>(), [-32767, 0, 32767]);
        assert_eq!(wide.payload_bytes(), 6);
        assert!(wide.packed_range(0, 2).is_err());
        assert!(CoefficientCodes::from_decoded(vec![i16::MIN], 16).is_err());
        assert!(CoefficientCodes::from_decoded(vec![0], 8).is_err());
        Ok(())
    }
}
