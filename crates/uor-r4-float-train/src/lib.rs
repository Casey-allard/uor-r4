//! D37 scanned offline float training module (`crates/uor-r4-float-train`, `#2823`).
//!
//! Hosts the 24-kernel geometric stack extract (`22` `__global__` kernels in `src/cuda_stack.cu`
//! and `2` `__global__` kernels in `src/spiralcore_fbs.cu`) and its native geometric stack modules
//! with **zero dependency edge** on `crates/uor-r4-training` (`CHARTER §3.4, §3.7`, D23, D25/D30,
//! D29, D32, D37).

#![forbid(unsafe_op_in_unsafe_fn)]

use sha2::{Digest, Sha256};
use std::fmt;

pub mod cuda_stack_kernels;
#[path = "geometric_stack_native.rs"]
pub mod geometric_stack;
pub mod hamilton_sheet;
pub mod kernel_controls;
pub mod native_starling_spec;
pub mod native_vector_writer;
pub mod secant_descent;
pub mod spiralcore_fbs;
pub mod spiralcore_fbs_arms;
pub mod spiralcore_fbs_sheet;

/// The 24 `__global__` CUDA kernel entry points in the D37 float-training extract
/// (`22` in `src/cuda_stack.cu` + `2` in `src/spiralcore_fbs.cu`), in declaration order.
pub const KERNEL_ENTRY_POINTS: [&str; 24] = [
    "straight_through_fwd",
    "hamilton_gate_fwd",
    "hamilton_gate_bwd",
    "rate_select_fwd",
    "rate_select_bwd",
    "rms_norm_fwd",
    "rms_norm_bwd_dx",
    "rms_norm_dw_partial",
    "rms_norm_dw_reduce",
    "quaternion_scan_fwd",
    "quaternion_scan_bwd",
    "margin_rank_rows",
    "margin_rank_grad",
    "recurrence_scan_fwd",
    "recurrence_tile_fold",
    "recurrence_tile_carry",
    "recurrence_tile_expand",
    "recurrence_bwd_params",
    "poison_words",
    "adam_update",
    "sq_lanes",
    "sq_tree",
    "spiralcore_fbs_fwd",
    "spiralcore_fbs_bwd",
];

/// Error type for the D37 offline float training module.
#[derive(Debug)]
pub enum FloatTrainError {
    Invalid(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Candle(candle_core::Error),
    Safetensors(safetensors::SafeTensorError),
}

impl fmt::Display for FloatTrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloatTrainError::Invalid(msg) => write!(f, "invalid float-train configuration: {msg}"),
            FloatTrainError::Io(err) => write!(f, "I/O error: {err}"),
            FloatTrainError::Json(err) => write!(f, "JSON error: {err}"),
            FloatTrainError::Candle(err) => write!(f, "tensor error: {err}"),
            FloatTrainError::Safetensors(err) => write!(f, "safetensors error: {err}"),
        }
    }
}

impl std::error::Error for FloatTrainError {}

impl From<std::io::Error> for FloatTrainError {
    fn from(value: std::io::Error) -> Self {
        FloatTrainError::Io(value)
    }
}

impl From<serde_json::Error> for FloatTrainError {
    fn from(value: serde_json::Error) -> Self {
        FloatTrainError::Json(value)
    }
}

impl From<candle_core::Error> for FloatTrainError {
    fn from(value: candle_core::Error) -> Self {
        FloatTrainError::Candle(value)
    }
}

impl From<safetensors::SafeTensorError> for FloatTrainError {
    fn from(value: safetensors::SafeTensorError) -> Self {
        FloatTrainError::Safetensors(value)
    }
}

pub type Result<T> = std::result::Result<T, FloatTrainError>;

pub(crate) fn invalid(msg: impl Into<String>) -> FloatTrainError {
    FloatTrainError::Invalid(msg.into())
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
