//! Protobuf field tags: `tag = (field_number << 3) | wire_type`.

use super::wire::WireError;

/// Protobuf wire types (proto3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    /// `0` — varint (int32/64, uint32/64, sint*, bool, enum).
    Varint,
    /// `1` — 64-bit fixed (fixed64, sfixed64, double).
    Fixed64,
    /// `2` — length-delimited (string, bytes, embedded messages, packed
    /// repeated scalars).
    LengthDelimited,
    /// `5` — 32-bit fixed (fixed32, sfixed32, float).
    Fixed32,
}

impl WireType {
    /// Decode the low 3 bits of a tag. Wire types `3`/`4` (deprecated
    /// groups) are rejected.
    pub fn from_bits(bits: u64) -> Result<Self, WireError> {
        Ok(match bits & 0x7 {
            0 => Self::Varint,
            1 => Self::Fixed64,
            2 => Self::LengthDelimited,
            5 => Self::Fixed32,
            other => return Err(WireError::UnknownWireType(other as u8)),
        })
    }
}

/// A decoded field tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    /// Field number (`>= 1`).
    pub field_number: u64,
    /// Wire type.
    pub wire_type: WireType,
}

impl Tag {
    /// Decode a tag from its varint value.
    pub fn from_varint(v: u64) -> Result<Self, WireError> {
        let field_number = v >> 3;
        if field_number == 0 {
            return Err(WireError::ZeroFieldNumber);
        }
        Ok(Self {
            field_number,
            wire_type: WireType::from_bits(v)?,
        })
    }
}
