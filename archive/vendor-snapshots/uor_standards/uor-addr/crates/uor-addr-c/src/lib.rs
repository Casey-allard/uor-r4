//! **`uor-addr-c` — C ABI bindings for `uor-addr`**.
//!
//! Exposes each UOR-ADDR realization through a stable `extern "C"`
//! entry point. The crate is `no_std` and `no_alloc` (mirrors
//! `uor-addr`'s defaults); the staticlib / cdylib outputs are
//! consumable from embedded C/C++ toolchains plus any language with
//! a C FFI (Python `cffi`, Go `cgo`, Ruby `FFI`, .NET P/Invoke).
//!
//! # API shape
//!
//! Every realization exposes one entry point of the form
//!
//! ```c
//! int32_t uor_addr_<realization>(
//!     const uint8_t *input,
//!     size_t input_len,
//!     uint8_t *out_label,
//!     size_t out_label_len,
//!     size_t *out_written);
//! ```
//!
//! - `input` / `input_len` — caller-owned input byte sequence.
//! - `out_label` / `out_label_len` — caller-owned output buffer; must
//!   be at least [`UOR_ADDR_LABEL_BYTES`] = 71 bytes.
//! - `out_written` — written with the number of bytes the function
//!   emitted (always 71 on success). May be `NULL` (the count is
//!   then discarded; the buffer is still filled).
//!
//! Return value is one of:
//!
//! - `UOR_ADDR_OK` (`0`) — success.
//! - `UOR_ADDR_ERR_NULL_POINTER` (`-1`) — invalid pointer.
//! - `UOR_ADDR_ERR_BUFFER_TOO_SMALL` (`-2`) — output buffer too small.
//! - `UOR_ADDR_ERR_INVALID_INPUT` (`-3`) — input rejected by parser.
//! - `UOR_ADDR_ERR_TOO_LARGE` (`-4`) — **reserved**; never returned under
//!   ADR-060 (inputs are unbounded). Retained for error-code stability.
//! - `UOR_ADDR_ERR_PIPELINE` (`-5`) — substrate-level failure.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_op_in_unsafe_fn)]

use core::slice;

use uor_addr::{asn1, codemodule, ring, sexp, AddressOutcome, ADDRESS_LABEL_BYTES};
// JSON / XML / schema / CBOR canonicalization needs `alloc` (object-key /
// attribute / map-key sorting), so their C entry points — and these
// imports — are `alloc`-gated under ADR-060.
#[cfg(feature = "alloc")]
use uor_addr::{cbor, composition, json, schema, xml, KappaLabel};

/// Wire-format κ-label byte width under the default σ-axis (sha256) —
/// `len("sha256:") + 64 = 71`.
#[no_mangle]
pub static UOR_ADDR_LABEL_BYTES: usize = ADDRESS_LABEL_BYTES;

/// Widest κ-label byte width across the admissible σ-axes (keccak256 →
/// `len("keccak256:") + 64 = 74`). A `*_with_hash` output buffer sized to
/// this fits every algorithm.
#[no_mangle]
pub static UOR_ADDR_MAX_LABEL_BYTES: usize = uor_addr::MAX_LABEL_BYTES;

/// σ-axis selector for the `*_with_hash` entry points: SHA-256 (default).
pub const UOR_ADDR_HASH_SHA256: u8 = 0;
/// σ-axis selector: BLAKE3.
pub const UOR_ADDR_HASH_BLAKE3: u8 = 1;
/// σ-axis selector: SHA3-256 (FIPS 202).
pub const UOR_ADDR_HASH_SHA3_256: u8 = 2;
/// σ-axis selector: Keccak-256 (pre-FIPS padding).
pub const UOR_ADDR_HASH_KECCAK256: u8 = 3;
/// σ-axis selector: SHA-512 (FIPS 180-4; 64-byte digest → 135-byte label).
pub const UOR_ADDR_HASH_SHA512: u8 = 4;

/// Success.
pub const UOR_ADDR_OK: i32 = 0;
/// `input == NULL && input_len > 0`, or `out_label == NULL`.
pub const UOR_ADDR_ERR_NULL_POINTER: i32 = -1;
/// `out_label_len < UOR_ADDR_LABEL_BYTES`.
pub const UOR_ADDR_ERR_BUFFER_TOO_SMALL: i32 = -2;
/// Input failed the realization's host-boundary parser.
pub const UOR_ADDR_ERR_INVALID_INPUT: i32 = -3;
/// **Reserved** — never returned under ADR-060 (inputs are unbounded;
/// the per-realization size/count caps were removed). Retained so
/// existing `-4` handlers in downstream C consumers keep compiling.
pub const UOR_ADDR_ERR_TOO_LARGE: i32 = -4;
/// Defensive — substrate-level pipeline failure.
pub const UOR_ADDR_ERR_PIPELINE: i32 = -5;
/// Unknown σ-axis selector passed to a `*_with_hash` entry point (not one
/// of the `UOR_ADDR_HASH_*` constants).
pub const UOR_ADDR_ERR_UNKNOWN_HASH: i32 = -6;
/// A composition operand's σ-axis does not match the operation's axis
/// (CA-3 σ-axis homogeneity), or — for the binary product — the two
/// operands carry different axes.
pub const UOR_ADDR_ERR_SIGMA_AXIS_MISMATCH: i32 = -7;

/// Map a [`composition::CompositionFailure`] to a C status code.
#[cfg(feature = "alloc")]
fn compose_code(e: composition::CompositionFailure) -> i32 {
    match e {
        composition::CompositionFailure::MalformedOperand => UOR_ADDR_ERR_INVALID_INPUT,
        composition::CompositionFailure::OperandSigmaAxisMismatch { .. } => {
            UOR_ADDR_ERR_SIGMA_AXIS_MISMATCH
        }
        composition::CompositionFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
    }
}

/// Marshal a successful `AddressOutcome` into the caller's output
/// buffer. Returns the appropriate error code on buffer overflow / null
/// pointer.
///
/// # Safety
///
/// `out_label` must be writable for at least `out_label_len` bytes;
/// `out_written` if non-null must point to a writable `usize`.
unsafe fn write_outcome<const N: usize, const FP: usize>(
    outcome: AddressOutcome<N, FP>,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    if out_label.is_null() {
        return UOR_ADDR_ERR_NULL_POINTER;
    }
    let bytes = outcome.address.as_bytes();
    if out_label_len < bytes.len() {
        return UOR_ADDR_ERR_BUFFER_TOO_SMALL;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_label, bytes.len());
        if !out_written.is_null() {
            *out_written = bytes.len();
        }
    }
    UOR_ADDR_OK
}

/// Borrow the caller's `input` slice safely.
///
/// # Safety
///
/// `input` must be null (with `input_len == 0`) or readable for
/// `input_len` bytes.
unsafe fn borrow_input<'a>(input: *const u8, input_len: usize) -> Result<&'a [u8], i32> {
    if input_len == 0 {
        return Ok(&[]);
    }
    if input.is_null() {
        return Err(UOR_ADDR_ERR_NULL_POINTER);
    }
    Ok(unsafe { slice::from_raw_parts(input, input_len) })
}

// ═══ Per-realization C entry points (generated uniformly) ═══════════

/// Map a realization's `AddressFailure` to a C status code.
trait CErr {
    fn c_code(&self) -> i32;
}

#[cfg(feature = "alloc")]
impl CErr for json::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            json::AddressFailure::InvalidJson => UOR_ADDR_ERR_INVALID_INPUT,
            json::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

impl CErr for sexp::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            sexp::AddressFailure::InvalidSExpr => UOR_ADDR_ERR_INVALID_INPUT,
            sexp::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "alloc")]
impl CErr for xml::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            xml::AddressFailure::InvalidXml => UOR_ADDR_ERR_INVALID_INPUT,
            xml::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

impl CErr for asn1::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            asn1::AddressFailure::InvalidDer => UOR_ADDR_ERR_INVALID_INPUT,
            asn1::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

impl CErr for ring::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            ring::AddressFailure::InvalidRingElement => UOR_ADDR_ERR_INVALID_INPUT,
            ring::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

impl CErr for codemodule::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            codemodule::AddressFailure::InvalidAst => UOR_ADDR_ERR_INVALID_INPUT,
            codemodule::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "alloc")]
impl CErr for cbor::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            cbor::AddressFailure::InvalidCbor => UOR_ADDR_ERR_INVALID_INPUT,
            cbor::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "alloc")]
impl CErr for schema::photo::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            schema::photo::AddressFailure::SchemaViolation => UOR_ADDR_ERR_INVALID_INPUT,
            schema::photo::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "alloc")]
impl CErr for schema::document::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            schema::document::AddressFailure::SchemaViolation => UOR_ADDR_ERR_INVALID_INPUT,
            schema::document::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "alloc")]
impl CErr for schema::codemodule_signed::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            schema::codemodule_signed::AddressFailure::SchemaViolation => {
                UOR_ADDR_ERR_INVALID_INPUT
            }
            schema::codemodule_signed::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "gguf")]
impl CErr for uor_addr::gguf::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            uor_addr::gguf::AddressFailure::InvalidGguf => UOR_ADDR_ERR_INVALID_INPUT,
            uor_addr::gguf::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

#[cfg(feature = "onnx")]
impl CErr for uor_addr::onnx::AddressFailure {
    fn c_code(&self) -> i32 {
        match self {
            uor_addr::onnx::AddressFailure::InvalidOnnx => UOR_ADDR_ERR_INVALID_INPUT,
            uor_addr::onnx::AddressFailure::PipelineFailure => UOR_ADDR_ERR_PIPELINE,
        }
    }
}

/// `json` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_json(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match json::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `json` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_json`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_json_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match json::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match json::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match json::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match json::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match json::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `sexp` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn uor_addr_sexp(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match sexp::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `sexp` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_sexp`].
#[no_mangle]
pub unsafe extern "C" fn uor_addr_sexp_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match sexp::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match sexp::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match sexp::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match sexp::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match sexp::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `xml` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_xml(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match xml::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `xml` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_xml`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_xml_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match xml::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match xml::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match xml::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match xml::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match xml::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `asn1` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn uor_addr_asn1(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match asn1::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `asn1` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_asn1`].
#[no_mangle]
pub unsafe extern "C" fn uor_addr_asn1_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match asn1::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match asn1::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match asn1::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match asn1::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match asn1::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `ring` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn uor_addr_ring(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match ring::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `ring` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_ring`].
#[no_mangle]
pub unsafe extern "C" fn uor_addr_ring_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match ring::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match ring::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match ring::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match ring::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match ring::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `codemodule` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn uor_addr_codemodule(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match codemodule::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `codemodule` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_codemodule`].
#[no_mangle]
pub unsafe extern "C" fn uor_addr_codemodule_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match codemodule::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match codemodule::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match codemodule::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match codemodule::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match codemodule::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `cbor` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_cbor(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match cbor::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `cbor` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_cbor`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_cbor_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match cbor::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match cbor::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match cbor::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match cbor::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match cbor::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_photo` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_photo(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::photo::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `schema_photo` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_schema_photo`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_photo_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::photo::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::photo::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::photo::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::photo::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::photo::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_document` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_document(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::document::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `schema_document` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_schema_document`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_document_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::document::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::document::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::document::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::document::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::document::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_codemodule_signed` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_codemodule_signed(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::codemodule_signed::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `schema_codemodule_signed` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_schema_codemodule_signed`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_codemodule_signed_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::codemodule_signed::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::codemodule_signed::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::codemodule_signed::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::codemodule_signed::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::codemodule_signed::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `gguf` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "gguf")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_gguf(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match uor_addr::gguf::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `gguf` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_gguf`].
#[cfg(feature = "gguf")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_gguf_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match uor_addr::gguf::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match uor_addr::gguf::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match uor_addr::gguf::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match uor_addr::gguf::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match uor_addr::gguf::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `onnx` realization — default σ-axis (SHA-256).
///
/// # Safety
///
/// - `input` is null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_label` is writable for at least `out_label_len` bytes.
/// - `out_written` if non-null points to a writable `size_t`.
#[cfg(feature = "onnx")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_onnx(
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match uor_addr::onnx::address(s) {
        Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
        Err(e) => e.c_code(),
    }
}

/// `onnx` realization under a caller-selected σ-axis (`UOR_ADDR_HASH_*`).
/// `out_label` must be writable for at least `UOR_ADDR_MAX_LABEL_BYTES`.
///
/// # Safety
///
/// As [`uor_addr_onnx`].
#[cfg(feature = "onnx")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_onnx_with_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match uor_addr::onnx::address(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match uor_addr::onnx::address_blake3(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match uor_addr::onnx::address_sha3_256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match uor_addr::onnx::address_keccak256(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match uor_addr::onnx::address_sha512(s) {
            Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

// ─── Grounded witness (TC-05 cross-language replay) ────────────────

/// `verify()` failed: empty trace. **Reserved** — the live verify path maps
/// every failure to `UOR_ADDR_ERR_PIPELINE`; retained for ABI stability.
pub const UOR_ADDR_ERR_VERIFY_EMPTY_TRACE: i32 = -10;
/// **Reserved** (see above).
pub const UOR_ADDR_ERR_VERIFY_OUT_OF_ORDER_EVENT: i32 = -11;
/// **Reserved** (see above).
pub const UOR_ADDR_ERR_VERIFY_ZERO_TARGET: i32 = -12;
/// **Reserved** (see above).
pub const UOR_ADDR_ERR_VERIFY_NON_CONTIGUOUS_STEPS: i32 = -13;
/// **Reserved** (see above).
pub const UOR_ADDR_ERR_VERIFY_CAPACITY_EXCEEDED: i32 = -14;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// Width-erased owned outcome — one opaque `UorAddrGrounded` handle carries
/// a κ-label of any admissible σ-axis width (71 / 73 / 74 for the 32-byte
/// fingerprint axes, 135 for sha512's 64-byte fingerprint).
#[cfg(feature = "alloc")]
pub(crate) enum AnyOutcome {
    W71(AddressOutcome<71>),
    W73(AddressOutcome<73>),
    W74(AddressOutcome<74>),
    W512(AddressOutcome<135, 64>),
}

#[cfg(feature = "alloc")]
impl AnyOutcome {
    fn label_bytes(&self) -> &[u8] {
        match self {
            Self::W71(o) => o.address.as_bytes(),
            Self::W73(o) => o.address.as_bytes(),
            Self::W74(o) => o.address.as_bytes(),
            Self::W512(o) => o.address.as_bytes(),
        }
    }
    fn fingerprint(&self) -> &[u8] {
        match self {
            Self::W71(o) => o.witness.content_fingerprint(),
            Self::W73(o) => o.witness.content_fingerprint(),
            Self::W74(o) => o.witness.content_fingerprint(),
            Self::W512(o) => o.witness.content_fingerprint(),
        }
    }
    fn verify(&self) -> Result<(), uor_addr::VerifyError> {
        match self {
            Self::W71(o) => o.witness.verify().map(|_| ()),
            Self::W73(o) => o.witness.verify().map(|_| ()),
            Self::W74(o) => o.witness.verify().map(|_| ()),
            Self::W512(o) => o.witness.verify().map(|_| ()),
        }
    }
}

/// Opaque, foreign-managed witness handle. Construct via any
/// `uor_addr_*_with_witness[_hash]` function; release with
/// `uor_addr_grounded_free`.
#[cfg(feature = "alloc")]
#[repr(C)]
pub struct UorAddrGrounded {
    pub(crate) outcome: AnyOutcome,
}

/// Box an `AnyOutcome` into a heap `UorAddrGrounded` and hand back the ptr.
///
/// # Safety
///
/// `out_handle` must be a valid writable `*mut UorAddrGrounded` pointer.
#[cfg(feature = "alloc")]
unsafe fn write_grounded_any(outcome: AnyOutcome, out_handle: *mut *mut UorAddrGrounded) -> i32 {
    if out_handle.is_null() {
        return UOR_ADDR_ERR_NULL_POINTER;
    }
    let ptr = Box::into_raw(Box::new(UorAddrGrounded { outcome }));
    unsafe {
        *out_handle = ptr;
    }
    UOR_ADDR_OK
}

/// Free a Grounded handle. Null is a no-op; each handle is freed once.
///
/// # Safety
///
/// `handle` is null or a pointer from a `*_with_witness[_hash]` call.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_grounded_free(handle: *mut UorAddrGrounded) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Read the κ-label this Grounded carries into `out_label` (its width
/// depends on the σ-axis; size `out_label` to `UOR_ADDR_MAX_LABEL_BYTES`).
///
/// # Safety
///
/// - `handle` is a live handle from a `*_with_witness[_hash]` call.
/// - `out_label` writable for `out_label_len` bytes; `out_written` if
///   non-null writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_grounded_kappa_label(
    handle: *const UorAddrGrounded,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    if handle.is_null() || out_label.is_null() {
        return UOR_ADDR_ERR_NULL_POINTER;
    }
    let g = unsafe { &*handle };
    let bytes = g.outcome.label_bytes();
    if out_label_len < bytes.len() {
        return UOR_ADDR_ERR_BUFFER_TOO_SMALL;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_label, bytes.len());
        if !out_written.is_null() {
            *out_written = bytes.len();
        }
    }
    UOR_ADDR_OK
}

/// Read the σ-projection content fingerprint into `out_digest` (32 bytes
/// for the `Hasher<32>` axes, 64 for sha512). Size `out_digest` to 64.
///
/// # Safety
///
/// As [`uor_addr_grounded_kappa_label`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_grounded_content_fingerprint(
    handle: *const UorAddrGrounded,
    out_digest: *mut u8,
    out_digest_len: usize,
    out_written: *mut usize,
) -> i32 {
    if handle.is_null() || out_digest.is_null() {
        return UOR_ADDR_ERR_NULL_POINTER;
    }
    let g = unsafe { &*handle };
    let fp = g.outcome.fingerprint();
    if out_digest_len < fp.len() {
        return UOR_ADDR_ERR_BUFFER_TOO_SMALL;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(fp.as_ptr(), out_digest, fp.len());
        if !out_written.is_null() {
            *out_written = fp.len();
        }
    }
    UOR_ADDR_OK
}

/// Verify the witness by re-certifying its replay trace (no σ-axis
/// re-invocation) and write the recovered κ-label into `out_label`.
///
/// # Safety
///
/// As [`uor_addr_grounded_kappa_label`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_grounded_verify(
    handle: *const UorAddrGrounded,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    if handle.is_null() || out_label.is_null() {
        return UOR_ADDR_ERR_NULL_POINTER;
    }
    let g = unsafe { &*handle };
    let bytes = g.outcome.label_bytes();
    if out_label_len < bytes.len() {
        return UOR_ADDR_ERR_BUFFER_TOO_SMALL;
    }
    match g.outcome.verify() {
        Ok(()) => unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_label, bytes.len());
            if !out_written.is_null() {
                *out_written = bytes.len();
            }
            UOR_ADDR_OK
        },
        Err(_) => UOR_ADDR_ERR_PIPELINE,
    }
}

/// `json` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_json_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match json::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `json` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_json_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_json_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match json::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match json::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match json::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match json::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match json::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `sexp` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_sexp_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match sexp::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `sexp` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_sexp_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_sexp_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match sexp::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match sexp::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match sexp::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match sexp::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match sexp::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `xml` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_xml_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match xml::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `xml` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_xml_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_xml_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match xml::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match xml::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match xml::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match xml::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match xml::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `asn1` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_asn1_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match asn1::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `asn1` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_asn1_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_asn1_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match asn1::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match asn1::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match asn1::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match asn1::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match asn1::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `ring` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_ring_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match ring::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `ring` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_ring_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_ring_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match ring::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match ring::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match ring::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match ring::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match ring::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `codemodule` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_codemodule_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match codemodule::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `codemodule` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_codemodule_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_codemodule_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match codemodule::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match codemodule::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match codemodule::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match codemodule::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match codemodule::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `cbor` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_cbor_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match cbor::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `cbor` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_cbor_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_cbor_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match cbor::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match cbor::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match cbor::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match cbor::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match cbor::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_photo` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_photo_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::photo::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `schema_photo` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_schema_photo_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_photo_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::photo::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::photo::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::photo::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::photo::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::photo::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_document` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_document_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::document::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `schema_document` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_schema_document_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_document_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::document::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::document::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::document::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::document::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::document::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `schema_codemodule_signed` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_codemodule_signed_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match schema::codemodule_signed::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `schema_codemodule_signed` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_schema_codemodule_signed_with_witness`].
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_schema_codemodule_signed_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match schema::codemodule_signed::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match schema::codemodule_signed::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match schema::codemodule_signed::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match schema::codemodule_signed::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match schema::codemodule_signed::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `gguf` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "gguf")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_gguf_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match uor_addr::gguf::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `gguf` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_gguf_with_witness`].
#[cfg(feature = "gguf")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_gguf_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match uor_addr::gguf::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match uor_addr::gguf::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match uor_addr::gguf::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match uor_addr::gguf::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match uor_addr::gguf::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// `onnx` realization — SHA-256 verifiable witness handle.
///
/// # Safety
///
/// - `input` null (with `input_len == 0`) or readable for `input_len` bytes.
/// - `out_handle` is a valid writable `*mut UorAddrGrounded`.
#[cfg(feature = "onnx")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_onnx_with_witness(
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match uor_addr::onnx::address(s) {
        Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
        Err(e) => e.c_code(),
    }
}

/// `onnx` realization — verifiable witness handle under a caller-selected
/// σ-axis (`UOR_ADDR_HASH_*`).
///
/// # Safety
///
/// As [`uor_addr_onnx_with_witness`].
#[cfg(feature = "onnx")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_onnx_with_witness_hash(
    algo: u8,
    input: *const u8,
    input_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(input, input_len) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match uor_addr::onnx::address(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_BLAKE3 => match uor_addr::onnx::address_blake3(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA3_256 => match uor_addr::onnx::address_sha3_256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_KECCAK256 => match uor_addr::onnx::address_keccak256(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
            Err(e) => e.c_code(),
        },
        UOR_ADDR_HASH_SHA512 => match uor_addr::onnx::address_sha512(s) {
            Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
            Err(e) => e.c_code(),
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

// ═══ κ-label composition (ADR-061) C entry points ══════════════════
//
// Operands are κ-label byte strings; `algo` (a `UOR_ADDR_HASH_*` selector)
// fixes the operand width and the composed axis. Each op offers a label
// entry point and a witness entry point. CS-G2 is binary; the rest unary.

/// CS-G2 composition (label). `algo` selects the σ-axis (operand
/// width + composed axis); `out_label` must be writable for at least
/// `UOR_ADDR_MAX_LABEL_BYTES` bytes.
///
/// # Safety
///
/// Operand pointers are null (with len 0) or readable for their lengths;
/// `out_label` writable for `out_label_len`; `out_written` if non-null
/// writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_g2(
    algo: u8,
    left: *const u8,
    left_len: usize,
    right: *const u8,
    right_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let l = match unsafe { borrow_input(left, left_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    let r = match unsafe { borrow_input(right, right_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match (
            KappaLabel::<71>::from_bytes(l),
            KappaLabel::<71>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product(&la, &ra) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match (
            KappaLabel::<71>::from_bytes(l),
            KappaLabel::<71>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_blake3(&la, &ra) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match (
            KappaLabel::<73>::from_bytes(l),
            KappaLabel::<73>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_sha3_256(&la, &ra) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match (
            KappaLabel::<74>::from_bytes(l),
            KappaLabel::<74>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_keccak256(&la, &ra) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match (
            KappaLabel::<135>::from_bytes(l),
            KappaLabel::<135>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_sha512(&la, &ra) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-G2 composition (verifiable witness handle). See
/// [`uor_addr_compose_g2`].
///
/// # Safety
///
/// As [`uor_addr_compose_g2`]; `out_handle` is a valid writable
/// `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_g2_with_witness(
    algo: u8,
    left: *const u8,
    left_len: usize,
    right: *const u8,
    right_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let l = match unsafe { borrow_input(left, left_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    let r = match unsafe { borrow_input(right, right_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match (
            KappaLabel::<71>::from_bytes(l),
            KappaLabel::<71>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product(&la, &ra) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match (
            KappaLabel::<71>::from_bytes(l),
            KappaLabel::<71>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_blake3(&la, &ra) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match (
            KappaLabel::<73>::from_bytes(l),
            KappaLabel::<73>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_sha3_256(&la, &ra) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match (
            KappaLabel::<74>::from_bytes(l),
            KappaLabel::<74>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_keccak256(&la, &ra) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match (
            KappaLabel::<135>::from_bytes(l),
            KappaLabel::<135>::from_bytes(r),
        ) {
            (Ok(la), Ok(ra)) => match composition::compose_g2_product_sha512(&la, &ra) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
                Err(e) => compose_code(e),
            },
            _ => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-F4 composition (label). `algo` selects the σ-axis (operand
/// width + composed axis); `out_label` must be writable for at least
/// `UOR_ADDR_MAX_LABEL_BYTES` bytes.
///
/// # Safety
///
/// Operand pointers are null (with len 0) or readable for their lengths;
/// `out_label` writable for `out_label_len`; `out_written` if non-null
/// writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_f4(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_blake3(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_sha3_256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_keccak256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_sha512(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-F4 composition (verifiable witness handle). See
/// [`uor_addr_compose_f4`].
///
/// # Safety
///
/// As [`uor_addr_compose_f4`]; `out_handle` is a valid writable
/// `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_f4_with_witness(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_blake3(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_sha3_256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_keccak256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_f4_quotient_sha512(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E6 composition (label). `algo` selects the σ-axis (operand
/// width + composed axis); `out_label` must be writable for at least
/// `UOR_ADDR_MAX_LABEL_BYTES` bytes.
///
/// # Safety
///
/// Operand pointers are null (with len 0) or readable for their lengths;
/// `out_label` writable for `out_label_len`; `out_written` if non-null
/// writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e6(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_blake3(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_sha3_256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_keccak256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_sha512(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E6 composition (verifiable witness handle). See
/// [`uor_addr_compose_e6`].
///
/// # Safety
///
/// As [`uor_addr_compose_e6`]; `out_handle` is a valid writable
/// `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e6_with_witness(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_blake3(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_sha3_256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_keccak256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e6_filtration_sha512(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E7 composition (label). `algo` selects the σ-axis (operand
/// width + composed axis); `out_label` must be writable for at least
/// `UOR_ADDR_MAX_LABEL_BYTES` bytes.
///
/// # Safety
///
/// Operand pointers are null (with len 0) or readable for their lengths;
/// `out_label` writable for `out_label_len`; `out_written` if non-null
/// writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e7(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_blake3(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_sha3_256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_keccak256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_sha512(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E7 composition (verifiable witness handle). See
/// [`uor_addr_compose_e7`].
///
/// # Safety
///
/// As [`uor_addr_compose_e7`]; `out_handle` is a valid writable
/// `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e7_with_witness(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_blake3(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_sha3_256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_keccak256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e7_augmentation_sha512(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E8 composition (label). `algo` selects the σ-axis (operand
/// width + composed axis); `out_label` must be writable for at least
/// `UOR_ADDR_MAX_LABEL_BYTES` bytes.
///
/// # Safety
///
/// Operand pointers are null (with len 0) or readable for their lengths;
/// `out_label` writable for `out_label_len`; `out_written` if non-null
/// writable.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e8(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_label: *mut u8,
    out_label_len: usize,
    out_written: *mut usize,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_blake3(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_sha3_256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_keccak256(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_sha512(&l) {
                Ok(o) => unsafe { write_outcome(o, out_label, out_label_len, out_written) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

/// CS-E8 composition (verifiable witness handle). See
/// [`uor_addr_compose_e8`].
///
/// # Safety
///
/// As [`uor_addr_compose_e8`]; `out_handle` is a valid writable
/// `*mut UorAddrGrounded`.
#[cfg(feature = "alloc")]
#[no_mangle]
pub unsafe extern "C" fn uor_addr_compose_e8_with_witness(
    algo: u8,
    operand: *const u8,
    operand_len: usize,
    out_handle: *mut *mut UorAddrGrounded,
) -> i32 {
    let s = match unsafe { borrow_input(operand, operand_len) } {
        Ok(s) => s,
        Err(c) => return c,
    };
    match algo {
        UOR_ADDR_HASH_SHA256 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_BLAKE3 => match KappaLabel::<71>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_blake3(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W71(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA3_256 => match KappaLabel::<73>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_sha3_256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W73(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_KECCAK256 => match KappaLabel::<74>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_keccak256(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W74(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        UOR_ADDR_HASH_SHA512 => match KappaLabel::<135>::from_bytes(s) {
            Ok(l) => match composition::compose_e8_embedding_sha512(&l) {
                Ok(o) => unsafe { write_grounded_any(AnyOutcome::W512(o), out_handle) },
                Err(e) => compose_code(e),
            },
            Err(_) => UOR_ADDR_ERR_INVALID_INPUT,
        },
        _ => UOR_ADDR_ERR_UNKNOWN_HASH,
    }
}

// ─── Panic handler for `no_std` builds without `std` ───────────────

// Panic handler is required on any `no_std` target. With `--features std`
// the standard library provides one and this stub is suppressed. The
// no_alloc surface never panics on well-formed input (bound checks
// return error codes); the handler is a safety net for unreachable
// arms.
// On bare-metal targets (`target_os = "none"`, e.g.
// `thumbv7em-none-eabihf`) no `std`-provided panic handler is
// linkable, so the crate must supply one. Hosted targets
// (`linux`, `macos`, `windows`, …) take `std::panic`'s default.
// We key off `target_os = "none"` rather than `feature = "std"` so
// cargo's workspace feature-unification (which can enable `std` in
// transitive deps for `--all-targets` test builds) doesn't cause a
// duplicate `panic_impl` lang item.
// Embedded bare-metal builds get our panic handler; hosted builds
// (`target_os = linux/macos/windows/…`) pull `std`'s default via the
// `std` feature.
#[cfg(all(not(feature = "std"), target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
