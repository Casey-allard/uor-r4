//! `ring::address` — the ring-element realization's public entry point.
//!
//! 1. [`RingElement::parse`] validates the Amendment 43 §2 canonical
//!    bytes at the host boundary.
//! 2. [`AddressModel::forward`] runs the shared ψ-tower: the handle's
//!    canonical bytes flow in as an ADR-060 carrier and ψ₉ folds them
//!    through `H = Sha256Hasher` to mint the κ-label.
//! 3. [`AddressOutcome::from_grounded`] extracts the owned κ-label +
//!    replayable TC-05 witness.

pub use crate::outcome::{AddressOutcome, AddressWitness, VerifyError};

/// Failure modes from [`address`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    /// The input bytes were not valid Amendment 43 canonical bytes (a
    /// canonical ring element is intrinsically ≤ 5 bytes — Witt level
    /// plus its little-endian coefficient — so every malformed or
    /// over-wide input falls here).
    InvalidRingElement,
    /// Defensive: foundation's catamorphism returned a shape violation.
    PipelineFailure,
}

use crate::ring::model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512,
};
use crate::ring::value::RingElement;
use prism::pipeline::PrismModel;

/// **uor-addr's ring entry point** (σ-axis `Sha256Hasher`) — one
/// ψ-pipeline content-address inference, yielding a `sha256:<64hex>`
/// κ-label.
///
/// # Errors
///
/// - [`AddressFailure::InvalidRingElement`] — the input is not well-formed.
/// - [`AddressFailure::PipelineFailure`] — defensive; unreachable.
pub fn address(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    let element =
        RingElement::parse(input_bytes).map_err(|_| AddressFailure::InvalidRingElement)?;
    let grounded = AddressModel::forward(element).map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The ring entry point under σ-axis `Blake3Hasher` — yields a
/// `blake3:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_blake3(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    let element =
        RingElement::parse(input_bytes).map_err(|_| AddressFailure::InvalidRingElement)?;
    let grounded =
        AddressModelBlake3::forward(element).map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The ring entry point under σ-axis `Sha3_256Hasher` — yields a
/// `sha3-256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_sha3_256(input_bytes: &[u8]) -> Result<AddressOutcome<73>, AddressFailure> {
    let element =
        RingElement::parse(input_bytes).map_err(|_| AddressFailure::InvalidRingElement)?;
    let grounded =
        AddressModelSha3_256::forward(element).map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The ring entry point under σ-axis `Keccak256Hasher` — yields a
/// `keccak256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_keccak256(input_bytes: &[u8]) -> Result<AddressOutcome<74>, AddressFailure> {
    let element =
        RingElement::parse(input_bytes).map_err(|_| AddressFailure::InvalidRingElement)?;
    let grounded =
        AddressModelKeccak256::forward(element).map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The ring entry point under σ-axis `Sha512Hasher` — yields a
/// `sha512:<128hex>` κ-label (135 bytes, 64-byte fingerprint). See
/// [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
pub fn address_sha512(input_bytes: &[u8]) -> Result<AddressOutcome<135, 64>, AddressFailure> {
    let element =
        RingElement::parse(input_bytes).map_err(|_| AddressFailure::InvalidRingElement)?;
    let grounded =
        AddressModelSha512::forward(element).map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}
