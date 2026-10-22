//! Exact fixed-geometry interfaces for the qualified native CUDA build.
//!
//! The original lib.rs, packed models, probability tables, samplers and dense
//! stack runtime remain preserved in place. No feature enables those model
//! executors in this crate's selected build. The retained modules implement
//! typed fixed H4 geometry and checked scalar integer arithmetic only.
#![forbid(unsafe_code)]

#[cfg(feature = "historical-runtime")]
compile_error!(
    "historical integer model/stack/sampling execution is excluded from native qualification"
);

// #1172: the preserved D11 integer/stack serving runtime compiles as this
// crate's modules only when the `serving-wasm` feature is explicitly selected
// (the #1172 WASM target). The native qualification graph never selects that
// feature, and its compile surface is unchanged.
#[cfg(feature = "serving-wasm")]
include!("serving_root.rs");

#[cfg(not(feature = "serving-wasm"))]
pub mod discrete_d11;
#[cfg(not(feature = "serving-wasm"))]
pub mod geometric_potential;
#[cfg(not(feature = "serving-wasm"))]
pub mod geometric_span;
#[cfg(not(feature = "serving-wasm"))]
pub mod geometric_value;
#[cfg(not(feature = "serving-wasm"))]
pub mod h4_classifier;
#[cfg(not(feature = "serving-wasm"))]
pub mod h4_tables;
#[cfg(not(feature = "serving-wasm"))]
pub mod lawful_likelihood;
#[cfg(not(feature = "serving-wasm"))]
pub mod m6_ablation;
#[cfg(not(feature = "serving-wasm"))]
pub mod m6_exact_arithmetic;
#[cfg(not(feature = "serving-wasm"))]
pub mod math;

#[cfg(not(feature = "serving-wasm"))]
#[path = "config_metadata.rs"]
pub mod config;
#[cfg(not(feature = "serving-wasm"))]
pub mod report_output;
#[cfg(not(feature = "serving-wasm"))]
pub use config::ReadGeometry;
#[cfg(not(feature = "serving-wasm"))]
pub type IntegerModelError = IntegerError;
#[cfg(not(feature = "serving-wasm"))]
use std::fmt;
#[cfg(not(feature = "serving-wasm"))]
#[derive(Debug)]
pub enum IntegerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    /// A model declares an offline read operator with no retained integer path.
    UnsupportedReadGeometry(ReadGeometry),
}
#[cfg(not(feature = "serving-wasm"))]
impl fmt::Display for IntegerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "integer runtime I/O: {error}"),
            Self::Json(error) => write!(f, "integer runtime JSON: {error}"),
            Self::Invalid(error) => write!(f, "invalid integer runtime input: {error}"),
            Self::UnsupportedReadGeometry(geometry) => write!(
                f,
                "{} read is offline-only; quantization, packed export and integer serving support dot and lorentz reads",
                geometry.name()
            ),
        }
    }
}
#[cfg(not(feature = "serving-wasm"))]
impl std::error::Error for IntegerError {}
#[cfg(not(feature = "serving-wasm"))]
impl From<std::io::Error> for IntegerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
#[cfg(not(feature = "serving-wasm"))]
impl From<serde_json::Error> for IntegerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
#[cfg(not(feature = "serving-wasm"))]
pub type Result<T> = std::result::Result<T, IntegerError>;
#[cfg(not(feature = "serving-wasm"))]
#[allow(dead_code)]
pub(crate) fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}
