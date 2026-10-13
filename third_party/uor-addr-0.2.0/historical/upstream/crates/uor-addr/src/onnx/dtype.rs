//! `OnnxDataType` — the ONNX `TensorProto.DataType` alphabet, a mapping
//! from the enum IDs `1..=23` to [`prism::tensor::dtype`] shapes plus the
//! ONNX-specific `STRING` (ID 8), which carries no numeric dtype.
//!
//! Authoritative source: the `TensorProto.DataType` enum in
//! <https://github.com/onnx/onnx/blob/main/onnx/onnx.proto>.

use prism::tensor::dtype::{
    Dtype, BF16, BOOL, C128, C64, F16, F32, F4_E2M1, F64, F8_E4M3, F8_E4M3_FNUZ, F8_E5M2,
    F8_E5M2_FNUZ, I16, I32, I4, I64, I8, U16, U32, U4, U64, U8,
};

/// An ONNX tensor element type. All variants except [`Self::String`] map
/// 1:1 to a [`prism::tensor::dtype`] shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnnxDataType {
    /// `FLOAT` (1) → [`F32`].
    Float,
    /// `UINT8` (2) → [`U8`].
    Uint8,
    /// `INT8` (3) → [`I8`].
    Int8,
    /// `UINT16` (4) → [`U16`].
    Uint16,
    /// `INT16` (5) → [`I16`].
    Int16,
    /// `INT32` (6) → [`I32`].
    Int32,
    /// `INT64` (7) → [`I64`].
    Int64,
    /// `STRING` (8) — ONNX-specific; not in the numeric dtype alphabet.
    String,
    /// `BOOL` (9) → [`BOOL`].
    Bool,
    /// `FLOAT16` (10) → [`F16`].
    Float16,
    /// `DOUBLE` (11) → [`F64`].
    Double,
    /// `UINT32` (12) → [`U32`].
    Uint32,
    /// `UINT64` (13) → [`U64`].
    Uint64,
    /// `COMPLEX64` (14) → [`C64`].
    Complex64,
    /// `COMPLEX128` (15) → [`C128`].
    Complex128,
    /// `BFLOAT16` (16) → [`BF16`].
    Bfloat16,
    /// `FLOAT8E4M3FN` (17) → [`F8_E4M3`].
    Float8E4M3Fn,
    /// `FLOAT8E4M3FNUZ` (18) → [`F8_E4M3_FNUZ`].
    Float8E4M3Fnuz,
    /// `FLOAT8E5M2` (19) → [`F8_E5M2`].
    Float8E5M2,
    /// `FLOAT8E5M2FNUZ` (20) → [`F8_E5M2_FNUZ`].
    Float8E5M2Fnuz,
    /// `UINT4` (21) → [`U4`].
    Uint4,
    /// `INT4` (22) → [`I4`].
    Int4,
    /// `FLOAT4E2M1` (23) → [`F4_E2M1`].
    Float4E2M1,
}

impl OnnxDataType {
    /// Map a raw `TensorProto.DataType` ID. Returns `None` for `0`
    /// (`UNDEFINED`) and IDs outside `1..=23`.
    #[must_use]
    pub const fn from_i32(id: i32) -> Option<Self> {
        Some(match id {
            1 => Self::Float,
            2 => Self::Uint8,
            3 => Self::Int8,
            4 => Self::Uint16,
            5 => Self::Int16,
            6 => Self::Int32,
            7 => Self::Int64,
            8 => Self::String,
            9 => Self::Bool,
            10 => Self::Float16,
            11 => Self::Double,
            12 => Self::Uint32,
            13 => Self::Uint64,
            14 => Self::Complex64,
            15 => Self::Complex128,
            16 => Self::Bfloat16,
            17 => Self::Float8E4M3Fn,
            18 => Self::Float8E4M3Fnuz,
            19 => Self::Float8E5M2,
            20 => Self::Float8E5M2Fnuz,
            21 => Self::Uint4,
            22 => Self::Int4,
            23 => Self::Float4E2M1,
            _ => return None,
        })
    }

    /// Block bytes from the corresponding [`prism::tensor::dtype`] shape,
    /// or `None` for [`Self::String`] (no fixed element width).
    #[must_use]
    pub const fn block_bytes(self) -> Option<usize> {
        Some(match self {
            Self::Float => F32::BLOCK_BYTES,
            Self::Uint8 => U8::BLOCK_BYTES,
            Self::Int8 => I8::BLOCK_BYTES,
            Self::Uint16 => U16::BLOCK_BYTES,
            Self::Int16 => I16::BLOCK_BYTES,
            Self::Int32 => I32::BLOCK_BYTES,
            Self::Int64 => I64::BLOCK_BYTES,
            Self::String => return None,
            Self::Bool => BOOL::BLOCK_BYTES,
            Self::Float16 => F16::BLOCK_BYTES,
            Self::Double => F64::BLOCK_BYTES,
            Self::Uint32 => U32::BLOCK_BYTES,
            Self::Uint64 => U64::BLOCK_BYTES,
            Self::Complex64 => C64::BLOCK_BYTES,
            Self::Complex128 => C128::BLOCK_BYTES,
            Self::Bfloat16 => BF16::BLOCK_BYTES,
            Self::Float8E4M3Fn => F8_E4M3::BLOCK_BYTES,
            Self::Float8E4M3Fnuz => F8_E4M3_FNUZ::BLOCK_BYTES,
            Self::Float8E5M2 => F8_E5M2::BLOCK_BYTES,
            Self::Float8E5M2Fnuz => F8_E5M2_FNUZ::BLOCK_BYTES,
            Self::Uint4 => U4::BLOCK_BYTES,
            Self::Int4 => I4::BLOCK_BYTES,
            Self::Float4E2M1 => F4_E2M1::BLOCK_BYTES,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undefined_and_out_of_range_rejected() {
        assert!(OnnxDataType::from_i32(0).is_none());
        assert!(OnnxDataType::from_i32(24).is_none());
        assert!(OnnxDataType::from_i32(-1).is_none());
    }

    #[test]
    fn full_range_maps() {
        for id in 1..=23 {
            assert!(OnnxDataType::from_i32(id).is_some(), "id {id} unmapped");
        }
    }

    #[test]
    fn string_has_no_block_width_but_others_do() {
        assert_eq!(OnnxDataType::String.block_bytes(), None);
        assert_eq!(OnnxDataType::Float.block_bytes(), Some(4));
        assert_eq!(OnnxDataType::Double.block_bytes(), Some(8));
        assert_eq!(OnnxDataType::Int4.block_bytes(), Some(1));
    }
}
