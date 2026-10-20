//! A minimal `no_std` + `no_alloc` Protocol Buffers v3 wire-format
//! decoder, sufficient for walking ONNX `ModelProto` messages by
//! reference.
//!
//! Because `uor-addr` is no_std + no_alloc by default, the ONNX
//! realization hand-writes the protobuf decode rather than pulling in
//! `prost` / `protobuf` (which require `alloc`). The decoder is
//! reference-based: it walks the input bytes, validates field numbers /
//! wire types, and yields borrowed field views — it never copies.
//!
//! Authoritative source: <https://protobuf.dev/programming-guides/encoding/>.

pub mod tag;
pub mod varint;
pub mod wire;

pub use tag::{Tag, WireType};
pub use varint::read_varint;
pub use wire::{Field, FieldValue, MessageReader, WireError};
