//! `json::address*` — the JSON realization's public entry points, one per
//! admissible σ-axis ([`crate::hash`]).
//!
//! 1. [`canonicalize`](crate::json::value::canonicalize) parses + emits
//!    the JCS-RFC8785 §3 + Unicode NFC canonical form into an `alloc`
//!    buffer (no width / depth / count caps).
//! 2. The selected axis's `AddressModel*::forward` runs the shared ψ-tower:
//!    the canonical bytes flow in as an ADR-060 `Borrowed` carrier and ψ₉
//!    folds them through the bound `H` to mint the κ-label.
//! 3. [`AddressOutcome::from_grounded`] extracts the owned κ-label +
//!    replayable TC-05 witness.
//!
//! [`address`] selects `H = Sha256Hasher` (the default); [`address_blake3`],
//! [`address_sha3_256`], and [`address_keccak256`] select the other 32-byte
//! axes. JSON canonicalization requires heap storage, so the entry points
//! are gated behind the `alloc` feature.

pub use crate::outcome::{AddressOutcome, AddressWitness, VerifyError};

/// Failure modes from the JSON entry points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    /// The input bytes were not valid UTF-8 JSON.
    InvalidJson,
    /// Defensive: foundation's catamorphism or a resolver returned a
    /// shape violation. Unreachable for well-formed inputs.
    PipelineFailure,
}

/// **uor-addr's JSON entry point** (σ-axis `Sha256Hasher`) — one
/// ψ-pipeline content-address inference per JSON input, yielding a
/// `sha256:<64hex>` κ-label.
///
/// # Errors
///
/// - [`AddressFailure::InvalidJson`] — `input_bytes` is not valid UTF-8 JSON.
/// - [`AddressFailure::PipelineFailure`] — defensive; unreachable in normal flow.
#[cfg(feature = "alloc")]
pub fn address(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    use prism::pipeline::PrismModel;

    use crate::json::model::AddressModel;
    use crate::json::value::{canonicalize, JsonCarrier};

    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let grounded = AddressModel::forward(JsonCarrier::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The JSON entry point under σ-axis `Blake3Hasher` — yields a
/// `blake3:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_blake3(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    use prism::pipeline::PrismModel;

    use crate::json::model::AddressModelBlake3;
    use crate::json::value::{canonicalize, JsonCarrier};

    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let grounded = AddressModelBlake3::forward(JsonCarrier::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The JSON entry point under σ-axis `Sha3_256Hasher` — yields a
/// `sha3-256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_sha3_256(input_bytes: &[u8]) -> Result<AddressOutcome<73>, AddressFailure> {
    use prism::pipeline::PrismModel;

    use crate::json::model::AddressModelSha3_256;
    use crate::json::value::{canonicalize, JsonCarrier};

    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let grounded = AddressModelSha3_256::forward(JsonCarrier::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The JSON entry point under σ-axis `Keccak256Hasher` — yields a
/// `keccak256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_keccak256(input_bytes: &[u8]) -> Result<AddressOutcome<74>, AddressFailure> {
    use prism::pipeline::PrismModel;

    use crate::json::model::AddressModelKeccak256;
    use crate::json::value::{canonicalize, JsonCarrier};

    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let grounded = AddressModelKeccak256::forward(JsonCarrier::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The JSON entry point under σ-axis `Sha512Hasher` — yields a
/// `sha512:<128hex>` κ-label (135 bytes, 64-byte fingerprint). See
/// [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_sha512(input_bytes: &[u8]) -> Result<AddressOutcome<135, 64>, AddressFailure> {
    use prism::pipeline::PrismModel;

    use crate::json::model::AddressModelSha512;
    use crate::json::value::{canonicalize, JsonCarrier};

    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidJson)?;
    let grounded = AddressModelSha512::forward(JsonCarrier::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}
