//! Integer-only serving of converted Llama checkpoints.
//!
//! This crate executes a packed artifact produced offline from a Hugging Face
//! Llama checkpoint (for example SmolLM2-135M-Instruct) under owner decision
//! D10 (`docs/integration/DECISIONS.md`):
//!
//! - no floating point anywhere in serving: every value is an integer with an
//!   explicit power-of-two exponent, and every nonlinearity (exp, SiLU, rsqrt,
//!   RoPE) is a sealed table written by the exporter;
//! - learned weight maps (projections, MLP, embedding, output head) execute
//!   without a multiplier: 4-bit weights in groups of [`GROUP`] with scales of
//!   the form `(16 + m) 2^(e - 4)`, applied through per-activation tables of
//!   multiples read sixteen rows at a time by the audited vector kernels of
//!   `uor-r4-simd` (table reads and additions per weight) and shift-add scales;
//! - products of two runtime values or of a runtime value and a fixed
//!   non-learned constant (attention scores, value mixing, gating,
//!   normalization, RoPE) use the hardware integer multiplier.
//!
//! The dense backbone reads every weight per token, so it is the interim chat
//! vehicle of D10 and remains non-compliant with D5's end-state sparsity.
//!
//! [`stack`] serves the lab's geometric stack (quaternion transport
//! recurrences and Dot or Lorentz reads between SwiGLU MLPs, exported by
//! `uor-r4-training`'s `stack_export`) under the same rules; its learned
//! scalars (convolution taps, decay rates, Lorentz scales) are applied by
//! shifts and adds. It is dense too: every weight is read per token.

#![forbid(unsafe_code)]

pub mod engine;
pub mod format;
pub mod kernels;
pub mod sampling;
pub mod stack;

pub use uor_r4_simd::Backend;

use std::fmt;

/// Weights per scale group along a matrix row.
pub const GROUP: usize = 32;
/// Fixed exponent of the residual stream and of projection outputs: a value
/// `v: i32` means `v * 2^-16`.
pub const RESIDUAL_EXP: i32 = -16;

#[derive(Debug)]
pub enum LutError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Format(String),
    Invalid(String),
    Kernel(uor_r4_simd::SimdError),
}

impl fmt::Display for LutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O: {error}"),
            Self::Json(error) => write!(f, "artifact header: {error}"),
            Self::Format(message) => write!(f, "artifact format: {message}"),
            Self::Invalid(message) => write!(f, "invalid request: {message}"),
            Self::Kernel(error) => write!(f, "kernel: {error}"),
        }
    }
}

impl std::error::Error for LutError {}

impl From<std::io::Error> for LutError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<uor_r4_simd::SimdError> for LutError {
    fn from(error: uor_r4_simd::SimdError) -> Self {
        Self::Kernel(error)
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
