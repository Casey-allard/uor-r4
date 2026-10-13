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

pub mod geometric_potential;
pub mod geometric_span;
pub mod geometric_value;
pub mod h4_classifier;
pub mod h4_tables;
pub mod math;

#[path = "config_metadata.rs"]
pub mod config;
pub mod report_output;
pub use config::ReadGeometry;
pub type IntegerModelError = IntegerError;
use std::fmt;
#[derive(Debug)]
pub enum IntegerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    /// A model declares an offline read operator with no retained integer path.
    UnsupportedReadGeometry(ReadGeometry),
}
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
impl std::error::Error for IntegerError {}
impl From<std::io::Error> for IntegerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for IntegerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
pub type Result<T> = std::result::Result<T, IntegerError>;
pub(crate) fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}
