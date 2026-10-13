//! ONNX realization spec constants + axis selection. The capacity
//! profile is the shared [`crate::bounds::AddrBounds`]; only the ONNX IR
//! v13 spec / policy constants live here.

pub mod bounds;

pub use bounds::{ONNX_IR_VERSION_MAX, ONNX_OPSET_VERSION_MIN, ONNX_SUBGRAPH_DEPTH_MAX};
/// Canonical `Hasher<32>` selection. Re-exported from the Prism standard
/// library; see wiki ADR-031 / ADR-047.
pub use prism::crypto::Sha256Hasher;
