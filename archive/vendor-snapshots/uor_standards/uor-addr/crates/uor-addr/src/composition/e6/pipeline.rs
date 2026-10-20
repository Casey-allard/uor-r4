//! CS-E6 composition entry points, one per σ-axis (wiki ADR-061 §(2)).

#![cfg(feature = "alloc")]

use crate::composition::canonicalize::{canonicalize_e6, check_axis, decode_operand};
use crate::composition::e6::model::{
    CompositionModelE6Blake3, CompositionModelE6Keccak256, CompositionModelE6Sha256,
    CompositionModelE6Sha3_256, CompositionModelE6Sha512,
};
use crate::composition::e6::value::E6Carrier;
use crate::composition::CompositionFailure;
use crate::label::KappaLabel;
use crate::outcome::AddressOutcome;
use prism::pipeline::PrismModel;

/// CS-E6 under σ-axis `Sha256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_e6_filtration(
    operand: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha256")?;
    let canon = canonicalize_e6(operand)?;
    let grounded = CompositionModelE6Sha256::forward(E6Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-E6 under σ-axis `Blake3Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"blake3"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_e6_filtration_blake3(
    operand: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "blake3")?;
    let canon = canonicalize_e6(operand)?;
    let grounded = CompositionModelE6Blake3::forward(E6Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-E6 under σ-axis `Sha3_256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha3-256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_e6_filtration_sha3_256(
    operand: &KappaLabel<73>,
) -> Result<AddressOutcome<73>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha3-256")?;
    let canon = canonicalize_e6(operand)?;
    let grounded = CompositionModelE6Sha3_256::forward(E6Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-E6 under σ-axis `Keccak256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"keccak256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_e6_filtration_keccak256(
    operand: &KappaLabel<74>,
) -> Result<AddressOutcome<74>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "keccak256")?;
    let canon = canonicalize_e6(operand)?;
    let grounded = CompositionModelE6Keccak256::forward(E6Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-E6 under σ-axis `Sha512Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha512"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_e6_filtration_sha512(
    operand: &KappaLabel<135>,
) -> Result<AddressOutcome<135, 64>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha512")?;
    let canon = canonicalize_e6(operand)?;
    let grounded = CompositionModelE6Sha512::forward(E6Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded)
        .map_err(|_| CompositionFailure::PipelineFailure)
}
