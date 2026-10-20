//! CS-G2 composition entry points, one per σ-axis (wiki ADR-061 §(2)).

#![cfg(feature = "alloc")]

use crate::composition::canonicalize::{canonicalize_g2, check_axis, decode_operand};
use crate::composition::g2::model::{
    CompositionModelG2Blake3, CompositionModelG2Keccak256, CompositionModelG2Sha256,
    CompositionModelG2Sha3_256, CompositionModelG2Sha512,
};
use crate::composition::g2::value::G2Carrier;
use crate::composition::CompositionFailure;
use crate::label::KappaLabel;
use crate::outcome::AddressOutcome;
use prism::pipeline::PrismModel;

/// CS-G2 under σ-axis `Sha256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_g2_product(
    left: &KappaLabel<71>,
    right: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (la, _) = decode_operand(left)?;
    check_axis(la, "sha256")?;
    let (ra, _) = decode_operand(right)?;
    check_axis(ra, "sha256")?;
    let canon = canonicalize_g2(left, right);
    let grounded = CompositionModelG2Sha256::forward(G2Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-G2 under σ-axis `Blake3Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"blake3"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_g2_product_blake3(
    left: &KappaLabel<71>,
    right: &KappaLabel<71>,
) -> Result<AddressOutcome<71>, CompositionFailure> {
    let (la, _) = decode_operand(left)?;
    check_axis(la, "blake3")?;
    let (ra, _) = decode_operand(right)?;
    check_axis(ra, "blake3")?;
    let canon = canonicalize_g2(left, right);
    let grounded = CompositionModelG2Blake3::forward(G2Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<71>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-G2 under σ-axis `Sha3_256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha3-256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_g2_product_sha3_256(
    left: &KappaLabel<73>,
    right: &KappaLabel<73>,
) -> Result<AddressOutcome<73>, CompositionFailure> {
    let (la, _) = decode_operand(left)?;
    check_axis(la, "sha3-256")?;
    let (ra, _) = decode_operand(right)?;
    check_axis(ra, "sha3-256")?;
    let canon = canonicalize_g2(left, right);
    let grounded = CompositionModelG2Sha3_256::forward(G2Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<73>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-G2 under σ-axis `Keccak256Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"keccak256"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_g2_product_keccak256(
    left: &KappaLabel<74>,
    right: &KappaLabel<74>,
) -> Result<AddressOutcome<74>, CompositionFailure> {
    let (la, _) = decode_operand(left)?;
    check_axis(la, "keccak256")?;
    let (ra, _) = decode_operand(right)?;
    check_axis(ra, "keccak256")?;
    let canon = canonicalize_g2(left, right);
    let grounded = CompositionModelG2Keccak256::forward(G2Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<74>::from_grounded(&grounded).map_err(|_| CompositionFailure::PipelineFailure)
}
/// CS-G2 under σ-axis `Sha512Hasher`.
///
/// # Errors
///
/// - [`CompositionFailure::OperandSigmaAxisMismatch`] — an operand's σ-axis
///   prefix is not `"sha512"`.
/// - [`CompositionFailure::MalformedOperand`] — an operand is not a
///   well-formed κ-label.
/// - [`CompositionFailure::PipelineFailure`] — defensive.
pub fn compose_g2_product_sha512(
    left: &KappaLabel<135>,
    right: &KappaLabel<135>,
) -> Result<AddressOutcome<135, 64>, CompositionFailure> {
    let (la, _) = decode_operand(left)?;
    check_axis(la, "sha512")?;
    let (ra, _) = decode_operand(right)?;
    check_axis(ra, "sha512")?;
    let canon = canonicalize_g2(left, right);
    let grounded = CompositionModelG2Sha512::forward(G2Carrier::new(&canon))
        .map_err(|_| CompositionFailure::PipelineFailure)?;
    AddressOutcome::<135, 64>::from_grounded(&grounded)
        .map_err(|_| CompositionFailure::PipelineFailure)
}
