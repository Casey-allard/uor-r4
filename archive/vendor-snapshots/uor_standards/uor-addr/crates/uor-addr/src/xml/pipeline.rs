//! `xml::address` — the XML realization's public entry point.
//!
//! 1. [`canonicalize`] parses + emits
//!    the W3C XML-C14N 1.1 (subset) canonical form into an `alloc` buffer
//!    (no width / count caps).
//! 2. `AddressModel::forward` runs the shared ψ-tower: the canonical
//!    bytes flow in as an ADR-060 `Borrowed` carrier and ψ₉ folds them
//!    through `H = Sha256Hasher` to mint the κ-label.
//! 3. [`AddressOutcome::from_grounded`] extracts the owned κ-label +
//!    replayable TC-05 witness.
//!
//! XML canonicalization requires heap storage (attribute sort scratch +
//! canonical output), so [`address`] is gated behind the `alloc` feature.

pub use crate::outcome::{AddressOutcome, AddressWitness, VerifyError};

/// Failure modes from [`address`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFailure {
    /// The input bytes were not a well-formed XML document in the
    /// supported canonical-XML subset.
    InvalidXml,
    /// Defensive: foundation's catamorphism or a resolver returned a
    /// shape violation. Unreachable for well-formed inputs.
    PipelineFailure,
}

#[cfg(feature = "alloc")]
use crate::xml::model::{
    AddressModel, AddressModelBlake3, AddressModelKeccak256, AddressModelSha3_256,
    AddressModelSha512,
};
#[cfg(feature = "alloc")]
use crate::xml::value::{canonicalize, XmlValue};
#[cfg(feature = "alloc")]
use prism::pipeline::PrismModel;

/// **uor-addr's xml entry point** (σ-axis `Sha256Hasher`) — one
/// ψ-pipeline content-address inference, yielding a `sha256:<64hex>`
/// κ-label.
///
/// # Errors
///
/// - [`AddressFailure::InvalidXml`] — the input is not well-formed.
/// - [`AddressFailure::PipelineFailure`] — defensive; unreachable.
#[cfg(feature = "alloc")]
pub fn address(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidXml)?;
    let grounded = AddressModel::forward(XmlValue::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The xml entry point under σ-axis `Blake3Hasher` — yields a
/// `blake3:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_blake3(input_bytes: &[u8]) -> Result<AddressOutcome<71>, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidXml)?;
    let grounded = AddressModelBlake3::forward(XmlValue::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The xml entry point under σ-axis `Sha3_256Hasher` — yields a
/// `sha3-256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_sha3_256(input_bytes: &[u8]) -> Result<AddressOutcome<73>, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidXml)?;
    let grounded = AddressModelSha3_256::forward(XmlValue::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The xml entry point under σ-axis `Keccak256Hasher` — yields a
/// `keccak256:<64hex>` κ-label. See [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_keccak256(input_bytes: &[u8]) -> Result<AddressOutcome<74>, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidXml)?;
    let grounded = AddressModelKeccak256::forward(XmlValue::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}

/// The xml entry point under σ-axis `Sha512Hasher` — yields a
/// `sha512:<128hex>` κ-label (135 bytes, 64-byte fingerprint). See
/// [`address`] for the error contract.
///
/// # Errors
///
/// As [`address`].
#[cfg(feature = "alloc")]
pub fn address_sha512(input_bytes: &[u8]) -> Result<AddressOutcome<135, 64>, AddressFailure> {
    let canonical = canonicalize(input_bytes).map_err(|_| AddressFailure::InvalidXml)?;
    let grounded = AddressModelSha512::forward(XmlValue::new(&canonical))
        .map_err(|_| AddressFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded).map_err(|_| AddressFailure::PipelineFailure)
}
