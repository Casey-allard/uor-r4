//! Retained LUT artifact metadata and exact grid helpers only (owner D29).
//!
//! Historical dense/table-normalized engines, sampling and runtime kernels are
//! preserved under docs/history/d29-excluded-lut-2026-10-13/src/ and excluded from this compiled facade.
//! No feature can reactivate them. This crate claims no serving capability.
#![forbid(unsafe_code)]

pub mod format;
pub mod kernels;
use std::fmt;

/// Weights per scale group in the unchanged retained artifact layout.
pub const GROUP: usize = 32;
/// Retained artifact exponent; metadata, not an active residual stream.
pub const RESIDUAL_EXP: i32 = -16;

#[derive(Debug)]
pub enum LutError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Format(String),
    Invalid(String),
}
impl fmt::Display for LutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O: {error}"),
            Self::Json(error) => write!(f, "artifact header: {error}"),
            Self::Format(message) => write!(f, "artifact format: {message}"),
            Self::Invalid(message) => write!(f, "invalid request: {message}"),
        }
    }
}
impl std::error::Error for LutError {}
impl From<std::io::Error> for LutError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for LutError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
pub type Result<T> = std::result::Result<T, LutError>;
pub(crate) fn format_error(message: impl Into<String>) -> LutError {
    LutError::Format(message.into())
}
pub(crate) fn invalid(message: impl Into<String>) -> LutError {
    LutError::Invalid(message.into())
}
