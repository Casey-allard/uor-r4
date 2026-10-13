//! `asn1::address` — the ASN.1 realization's public entry point.
//!
//! 1. [`validate_der`] checks the input is a single well-formed DER value
//!    (X.690 §§ 8 / 10 / 11) at the host boundary — no buffer, no caps.
//! 2. [`AddressModel::forward`] runs the shared ψ-tower: DER is canonical,
//!    so the input bytes flow in directly as an ADR-060 `Borrowed`
//!    carrier and ψ₉ folds them through `H = Sha256Hasher` to mint the
//!    κ-label.
//! 3. [`AddressOutcome::from_grounded`] extracts the owned κ-label +
//!    replayable TC-05 witness.
//!
//! The entry point is `no_alloc`: no transformation buffer is needed
//! because DER is its own canonical form.

pub use crate::outcome::{AddressOutcome, AddressWitness, VerifyError};

/// Failure modes from [`address`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    /// The input bytes were not a single well-formed DER value.
    InvalidDer,
    /// Defensive: foundation's catamorphism or a resolver returned a
    /// shape violation. Unreachable for well-formed inputs.
    PipelineFailure,
}

use crate::asn1::model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512,
};
use crate::asn1::value::{validate_der, Asn1Carrier};
use prism::pipeline::PrismModel;

/// **uor-addr's asn1 entry point** (σ-axis `Sha256Hasher`) — one
/// ψ-pipeline content-address inference, yielding a `sha256:<64hex>`
/// κ-label.
///
/// # Errors
///
/// - [`AddressFailure::InvalidDer`] — the input is not well-formed.
/// - [`AddressFailure::PipelineFailure`] — defensive; unreachable.
pub fn address(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    validate_der(input_bytes).map_err(|_| AddressFailure::InvalidDer)?;
    let grounded = AddressModel::forward(Asn1Carrier::new(input_bytes))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The asn1 entry point under σ-axis `Blake3Hasher` — yields a
/// `blake3:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_blake3(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    validate_der(input_bytes).map_err(|_| AddressFailure::InvalidDer)?;
    let grounded = AddressModelBlake3::forward(Asn1Carrier::new(input_bytes))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The asn1 entry point under σ-axis `Sha3_256Hasher` — yields a
/// `sha3-256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_sha3_256(input_bytes: &[u8]) -> Result<AddressOutcome<73>, AddressFailure> {
    validate_der(input_bytes).map_err(|_| AddressFailure::InvalidDer)?;
    let grounded = AddressModelSha3_256::forward(Asn1Carrier::new(input_bytes))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The asn1 entry point under σ-axis `Keccak256Hasher` — yields a
/// `keccak256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_keccak256(input_bytes: &[u8]) -> Result<AddressOutcome<74>, AddressFailure> {
    validate_der(input_bytes).map_err(|_| AddressFailure::InvalidDer)?;
    let grounded = AddressModelKeccak256::forward(Asn1Carrier::new(input_bytes))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The asn1 entry point under σ-axis `Sha512Hasher` — yields a
/// `sha512:<128hex>` κ-label (135 bytes, 64-byte fingerprint). See
/// [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_sha512(input_bytes: &[u8]) -> Result<AddressOutcome<135, 64>, AddressFailure> {
    validate_der(input_bytes).map_err(|_| AddressFailure::InvalidDer)?;
    let grounded = AddressModelSha512::forward(Asn1Carrier::new(input_bytes))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}
