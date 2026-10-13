//! CS-F4 composition entry points, one per σ-axis (wiki ADR-061 §(2)).

#![cfg(feature = "alloc")]

use crate::composition::canonicalize::{canonicalize_f4, check_axis, decode_operand};
use crate::composition::f4::model::{
    CompositionModelF4Blake3, CompositionModelF4Keccak256, CompositionModelF4Sha256,
    CompositionModelF4Sha3_256, CompositionModelF4Sha512,
};
use crate::composition::f4::value::F4Carrier;
use crate::composition::CompositionFailure;
use crate::label::KappaLabel;
use crate::outcome::AddressOutcome;
use prism::pipeline::PrismModel;

/// CS-F4 under σ-axis `Sha256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_f4_quotient(
    operand: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha256")?;
    let canon = canonicalize_f4(operand)?;
    let grounded = CompositionModelF4Sha256::forward(F4Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-F4 under σ-axis `Blake3Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"blake3"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_f4_quotient_blake3(
    operand: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "blake3")?;
    let canon = canonicalize_f4(operand)?;
    let grounded = CompositionModelF4Blake3::forward(F4Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-F4 under σ-axis `Sha3_256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha3-256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_f4_quotient_sha3_256(
    operand: &KappaLabel<73>,
) -> Result<AddressOutcome<73>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha3-256")?;
    let canon = canonicalize_f4(operand)?;
    let grounded = CompositionModelF4Sha3_256::forward(F4Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-F4 under σ-axis `Keccak256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"keccak256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_f4_quotient_keccak256(
    operand: &KappaLabel<74>,
) -> Result<AddressOutcome<74>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "keccak256")?;
    let canon = canonicalize_f4(operand)?;
    let grounded = CompositionModelF4Keccak256::forward(F4Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-F4 under σ-axis `Sha512Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha512"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_f4_quotient_sha512(
    operand: &KappaLabel<135>,
) -> Result<AddressOutcome<135, 64>, CompositionFailure> {
    let (axis, _) = decode_operand(operand)?;
    check_axis(axis, "sha512")?;
    let canon = canonicalize_f4(operand)?;
    let grounded = CompositionModelF4Sha512::forward(F4Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded)
        .map_err(|_| CompositionFailure::PipelineFailure)
}
