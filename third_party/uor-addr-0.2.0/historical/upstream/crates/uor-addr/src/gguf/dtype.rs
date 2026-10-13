//! `GgmlType` — the GGUF tensor element-type alphabet, a total mapping
//! from the GGML `ggml_type` integer IDs to the
//! [`prism::tensor::dtype`] shapes.
//!
//! The mapping is the single source of truth for GGUF tensor-type
//! validation: every `ggml_type` ID admitted at the typed-input
//! boundary resolves to a `prism::tensor::dtype` shape carrying that
//! dtype's `BLOCK_BYTES` (block size in bytes) and `BLOCK_ELEMS`
//! (elements per block). The per-tensor byte count derives mechanically
//! as `(num_elements / BLOCK_ELEMS) * BLOCK_BYTES`.
//!
//! IDs `4` and `5` (deprecated `GGML_TYPE_Q4_2` / `GGML_TYPE_Q4_3`) are
//! rejected at the typed-input boundary — they carry no
//! `prism::tensor::dtype` counterpart.
//!
//! Authoritative source: the `ggml_type` enum in
//! <https://github.com/ggml-org/ggml/blob/master/include/ggml.h>.

use prism::tensor::dtype::{
    Dtype, BF16, F16, F32, F64, I16, I32, I64, I8, IQ1_M, IQ1_S, IQ2_S, IQ2_XS, IQ2_XXS, IQ3_S,
    IQ3_XXS, IQ4_NL, IQ4_XS, Q2_K, Q3_K, Q4_0, Q4_1, Q4_K, Q5_0, Q5_1, Q5_K, Q6_K, Q8_0, Q8_1,
    Q8_K,
};

/// A GGUF tensor element type, identified by its `ggml_type` integer
/// ID. Each variant maps 1:1 to a [`prism::tensor::dtype`] shape.
///
/// Variant spellings deliberately mirror the GGML `ggml_type` enum and
/// the `prism::tensor::dtype` type names (`Q4_0`, `IQ4_NL`, …) rather
/// than Rust camel case, so the mapping reads 1:1 against the
/// authoritative source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum GgmlType {
    /// `GGML_TYPE_F32` (ID 0) → [`F32`].
    F32,
    /// `GGML_TYPE_F16` (ID 1) → [`F16`].
    F16,
    /// `GGML_TYPE_Q4_0` (ID 2) → [`Q4_0`].
    Q4_0,
    /// `GGML_TYPE_Q4_1` (ID 3) → [`Q4_1`].
    Q4_1,
    /// `GGML_TYPE_Q5_0` (ID 6) → [`Q5_0`].
    Q5_0,
    /// `GGML_TYPE_Q5_1` (ID 7) → [`Q5_1`].
    Q5_1,
    /// `GGML_TYPE_Q8_0` (ID 8) → [`Q8_0`].
    Q8_0,
    /// `GGML_TYPE_Q8_1` (ID 9) → [`Q8_1`].
    Q8_1,
    /// `GGML_TYPE_Q2_K` (ID 10) → [`Q2_K`].
    Q2_K,
    /// `GGML_TYPE_Q3_K` (ID 11) → [`Q3_K`].
    Q3_K,
    /// `GGML_TYPE_Q4_K` (ID 12) → [`Q4_K`].
    Q4_K,
    /// `GGML_TYPE_Q5_K` (ID 13) → [`Q5_K`].
    Q5_K,
    /// `GGML_TYPE_Q6_K` (ID 14) → [`Q6_K`].
    Q6_K,
    /// `GGML_TYPE_Q8_K` (ID 15) → [`Q8_K`].
    Q8_K,
    /// `GGML_TYPE_IQ2_XXS` (ID 16) → [`IQ2_XXS`].
    IQ2_XXS,
    /// `GGML_TYPE_IQ2_XS` (ID 17) → [`IQ2_XS`].
    IQ2_XS,
    /// `GGML_TYPE_IQ3_XXS` (ID 18) → [`IQ3_XXS`].
    IQ3_XXS,
    /// `GGML_TYPE_IQ1_S` (ID 19) → [`IQ1_S`].
    IQ1_S,
    /// `GGML_TYPE_IQ4_NL` (ID 20) → [`IQ4_NL`].
    IQ4_NL,
    /// `GGML_TYPE_IQ3_S` (ID 21) → [`IQ3_S`].
    IQ3_S,
    /// `GGML_TYPE_IQ2_S` (ID 22) → [`IQ2_S`].
    IQ2_S,
    /// `GGML_TYPE_IQ4_XS` (ID 23) → [`IQ4_XS`].
    IQ4_XS,
    /// `GGML_TYPE_I8` (ID 24) → [`I8`].
    I8,
    /// `GGML_TYPE_I16` (ID 25) → [`I16`].
    I16,
    /// `GGML_TYPE_I32` (ID 26) → [`I32`].
    I32,
    /// `GGML_TYPE_I64` (ID 27) → [`I64`].
    I64,
    /// `GGML_TYPE_F64` (ID 28) → [`F64`].
    F64,
    /// `GGML_TYPE_IQ1_M` (ID 29) → [`IQ1_M`].
    IQ1_M,
    /// `GGML_TYPE_BF16` (ID 30) → [`BF16`].
    BF16,
}

impl GgmlType {
    /// Map a raw `ggml_type` integer ID to a [`GgmlType`].
    ///
    /// Returns `None` for IDs outside the GGUF v3 tensor-type set —
    /// including the deprecated IDs `4` and `5`.
    #[must_use]
    pub const fn from_u32(id: u32) -> Option<Self> {
        Some(match id {
            0 => Self::F32,
            1 => Self::F16,
            2 => Self::Q4_0,
            3 => Self::Q4_1,
            6 => Self::Q5_0,
            7 => Self::Q5_1,
            8 => Self::Q8_0,
            9 => Self::Q8_1,
            10 => Self::Q2_K,
            11 => Self::Q3_K,
            12 => Self::Q4_K,
            13 => Self::Q5_K,
            14 => Self::Q6_K,
            15 => Self::Q8_K,
            16 => Self::IQ2_XXS,
            17 => Self::IQ2_XS,
            18 => Self::IQ3_XXS,
            19 => Self::IQ1_S,
            20 => Self::IQ4_NL,
            21 => Self::IQ3_S,
            22 => Self::IQ2_S,
            23 => Self::IQ4_XS,
            24 => Self::I8,
            25 => Self::I16,
            26 => Self::I32,
            27 => Self::I64,
            28 => Self::F64,
            29 => Self::IQ1_M,
            30 => Self::BF16,
            _ => return None,
        })
    }

    /// The canonical `ggml_type` integer ID for this dtype.
    #[must_use]
    pub const fn id(self) -> u32 {
        match self {
            Self::F32 => 0,
            Self::F16 => 1,
            Self::Q4_0 => 2,
            Self::Q4_1 => 3,
            Self::Q5_0 => 6,
            Self::Q5_1 => 7,
            Self::Q8_0 => 8,
            Self::Q8_1 => 9,
            Self::Q2_K => 10,
            Self::Q3_K => 11,
            Self::Q4_K => 12,
            Self::Q5_K => 13,
            Self::Q6_K => 14,
            Self::Q8_K => 15,
            Self::IQ2_XXS => 16,
            Self::IQ2_XS => 17,
            Self::IQ3_XXS => 18,
            Self::IQ1_S => 19,
            Self::IQ4_NL => 20,
            Self::IQ3_S => 21,
            Self::IQ2_S => 22,
            Self::IQ4_XS => 23,
            Self::I8 => 24,
            Self::I16 => 25,
            Self::I32 => 26,
            Self::I64 => 27,
            Self::F64 => 28,
            Self::IQ1_M => 29,
            Self::BF16 => 30,
        }
    }

    /// Bytes per block, sourced from the [`prism::tensor::dtype`] shape.
    #[must_use]
    pub const fn block_bytes(self) -> usize {
        match self {
            Self::F32 => F32::BLOCK_BYTES,
            Self::F16 => F16::BLOCK_BYTES,
            Self::Q4_0 => Q4_0::BLOCK_BYTES,
            Self::Q4_1 => Q4_1::BLOCK_BYTES,
            Self::Q5_0 => Q5_0::BLOCK_BYTES,
            Self::Q5_1 => Q5_1::BLOCK_BYTES,
            Self::Q8_0 => Q8_0::BLOCK_BYTES,
            Self::Q8_1 => Q8_1::BLOCK_BYTES,
            Self::Q2_K => Q2_K::BLOCK_BYTES,
            Self::Q3_K => Q3_K::BLOCK_BYTES,
            Self::Q4_K => Q4_K::BLOCK_BYTES,
            Self::Q5_K => Q5_K::BLOCK_BYTES,
            Self::Q6_K => Q6_K::BLOCK_BYTES,
            Self::Q8_K => Q8_K::BLOCK_BYTES,
            Self::IQ2_XXS => IQ2_XXS::BLOCK_BYTES,
            Self::IQ2_XS => IQ2_XS::BLOCK_BYTES,
            Self::IQ3_XXS => IQ3_XXS::BLOCK_BYTES,
            Self::IQ1_S => IQ1_S::BLOCK_BYTES,
            Self::IQ4_NL => IQ4_NL::BLOCK_BYTES,
            Self::IQ3_S => IQ3_S::BLOCK_BYTES,
            Self::IQ2_S => IQ2_S::BLOCK_BYTES,
            Self::IQ4_XS => IQ4_XS::BLOCK_BYTES,
            Self::I8 => I8::BLOCK_BYTES,
            Self::I16 => I16::BLOCK_BYTES,
            Self::I32 => I32::BLOCK_BYTES,
            Self::I64 => I64::BLOCK_BYTES,
            Self::F64 => F64::BLOCK_BYTES,
            Self::IQ1_M => IQ1_M::BLOCK_BYTES,
            Self::BF16 => BF16::BLOCK_BYTES,
        }
    }

    /// Elements per block, sourced from the [`prism::tensor::dtype`]
    /// shape.
    #[must_use]
    pub const fn block_elems(self) -> usize {
        match self {
            Self::F32 => F32::BLOCK_ELEMS,
            Self::F16 => F16::BLOCK_ELEMS,
            Self::Q4_0 => Q4_0::BLOCK_ELEMS,
            Self::Q4_1 => Q4_1::BLOCK_ELEMS,
            Self::Q5_0 => Q5_0::BLOCK_ELEMS,
            Self::Q5_1 => Q5_1::BLOCK_ELEMS,
            Self::Q8_0 => Q8_0::BLOCK_ELEMS,
            Self::Q8_1 => Q8_1::BLOCK_ELEMS,
            Self::Q2_K => Q2_K::BLOCK_ELEMS,
            Self::Q3_K => Q3_K::BLOCK_ELEMS,
            Self::Q4_K => Q4_K::BLOCK_ELEMS,
            Self::Q5_K => Q5_K::BLOCK_ELEMS,
            Self::Q6_K => Q6_K::BLOCK_ELEMS,
            Self::Q8_K => Q8_K::BLOCK_ELEMS,
            Self::IQ2_XXS => IQ2_XXS::BLOCK_ELEMS,
            Self::IQ2_XS => IQ2_XS::BLOCK_ELEMS,
            Self::IQ3_XXS => IQ3_XXS::BLOCK_ELEMS,
            Self::IQ1_S => IQ1_S::BLOCK_ELEMS,
            Self::IQ4_NL => IQ4_NL::BLOCK_ELEMS,
            Self::IQ3_S => IQ3_S::BLOCK_ELEMS,
            Self::IQ2_S => IQ2_S::BLOCK_ELEMS,
            Self::IQ4_XS => IQ4_XS::BLOCK_ELEMS,
            Self::I8 => I8::BLOCK_ELEMS,
            Self::I16 => I16::BLOCK_ELEMS,
            Self::I32 => I32::BLOCK_ELEMS,
            Self::I64 => I64::BLOCK_ELEMS,
            Self::F64 => F64::BLOCK_ELEMS,
            Self::IQ1_M => IQ1_M::BLOCK_ELEMS,
            Self::BF16 => BF16::BLOCK_ELEMS,
        }
    }

    /// The total byte count for a tensor of `num_elements` of this
    /// dtype: `(num_elements / BLOCK_ELEMS) * BLOCK_BYTES`. Returns
    /// `None` if `num_elements` is not a whole multiple of
    /// `BLOCK_ELEMS` (a malformed quantized tensor) or on overflow.
    #[must_use]
    pub const fn tensor_data_bytes(self, num_elements: u64) -> Option<u64> {
        let elems = self.block_elems() as u64;
        if elems == 0 || num_elements % elems != 0 {
            return None;
        }
        let blocks = num_elements / elems;
        blocks.checked_mul(self.block_bytes() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deprecated_ids_rejected() {
        assert!(GgmlType::from_u32(4).is_none());
        assert!(GgmlType::from_u32(5).is_none());
        assert!(GgmlType::from_u32(31).is_none());
    }

    #[test]
    fn id_round_trips() {
        for id in [0u32, 1, 2, 3, 6, 7, 8, 9, 10, 14, 23, 24, 28, 29, 30] {
            let t = GgmlType::from_u32(id).expect("known id");
            assert_eq!(t.id(), id);
        }
    }

    #[test]
    fn block_geometry_matches_prism() {
        // Continuous types: 1 element per block.
        assert_eq!(GgmlType::F32.block_bytes(), 4);
        assert_eq!(GgmlType::F32.block_elems(), 1);
        assert_eq!(GgmlType::F16.block_bytes(), 2);
        assert_eq!(GgmlType::BF16.block_bytes(), 2);
        assert_eq!(GgmlType::F64.block_bytes(), 8);
        // Legacy block-32 quant.
        assert_eq!(GgmlType::Q4_0.block_elems(), 32);
        // K-series block-256 quant.
        assert_eq!(GgmlType::Q4_K.block_elems(), 256);
    }

    #[test]
    fn tensor_data_bytes_mechanical() {
        // 64 F32 elements = 64 * 4 = 256 bytes.
        assert_eq!(GgmlType::F32.tensor_data_bytes(64), Some(256));
        // 256 Q4_K elements = 1 block * BLOCK_BYTES.
        assert_eq!(
            GgmlType::Q4_K.tensor_data_bytes(256),
            Some(Q4_K::BLOCK_BYTES as u64)
        );
        // Non-multiple of block size for a quantized type is rejected.
        assert_eq!(GgmlType::Q4_K.tensor_data_bytes(100), None);
    }
}
