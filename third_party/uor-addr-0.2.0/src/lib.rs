//! UOR-ADDR metadata-only JSON content-address subset for native qualification.
//! Numerical Prism dependencies and model/witness APIs remain inactive preserved source.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
#[cfg(any(feature = "gguf", feature = "onnx"))]
compile_error!("this UOR-ADDR build exposes metadata-only JSON addressing; historical realizations are excluded");
#[cfg(not(feature = "alloc"))]
compile_error!("metadata JSON addressing requires the alloc feature");
pub mod canonical;
pub mod json;
