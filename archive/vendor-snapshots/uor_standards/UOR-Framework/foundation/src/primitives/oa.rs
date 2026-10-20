// @codegen-exempt — Phase 15 hand-written verification bodies for the OA family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `oa` (Surface-Symmetry /
//! grounding+projection) theorem family. Five Path-2 classes route
//! here: morphism::GroundingWitness, morphism::ProjectionWitness,
//! morphism::Witness (abstract), state::GroundingWitness, all under
//! `op:surfaceSymmetry` (P∘Π∘G commutes when G and P share a Frame).

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{
    MintMorphismGroundingWitness, MintMorphismGroundingWitnessInputs, MintProjectionWitness,
    MintProjectionWitnessInputs, MintStateGroundingWitness, MintStateGroundingWitnessInputs,
    MintWitness, MintWitnessInputs,
};
use crate::HostTypes;

/// Index-salted XOR fold over chunked bytes — see `br.rs` for rationale.
fn fingerprint_for_inputs(chunks: &[&[u8]]) -> ContentFingerprint {
    let mut buf = [0u8; 32];
    let mut global: usize = 0;
    let mut chunk_idx = 0;
    while chunk_idx < chunks.len() {
        let chunk = chunks[chunk_idx];
        let mut i = 0;
        while i < chunk.len() {
            let pos = global % 32;
            #[allow(clippy::cast_possible_truncation)]
            let salt = global as u8;
            buf[pos] ^= chunk[i].wrapping_add(salt);
            i += 1;
            global += 1;
        }
        let pos = global % 32;
        buf[pos] ^= 0xFFu8;
        global += 1;
        chunk_idx += 1;
    }
    ContentFingerprint::from_buffer(buf, 32u8)
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/morphism/GroundingWitness`.
///
/// Theorem identity: `https://uor.foundation/op/surfaceSymmetry`.
///
/// # Errors
///
/// * `op:surfaceSymmetry` — `surface_symbol` or `grounded_address`
///   handle is zero (no concrete grounding evidence).
pub fn verify_morphism_grounding_witness<H: HostTypes + 'static>(
    inputs: MintMorphismGroundingWitnessInputs<H>,
) -> Result<MintMorphismGroundingWitness, GenericImpossibilityWitness> {
    if inputs.surface_symbol.fingerprint.is_zero() || inputs.grounded_address.fingerprint.is_zero()
    {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/surfaceSymmetry",
        ));
    }
    let symbol_bytes = inputs.surface_symbol.fingerprint.as_bytes();
    let addr_bytes = inputs.grounded_address.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/surfaceSymmetry",
        symbol_bytes,
        addr_bytes,
    ]);
    Ok(MintMorphismGroundingWitness::from_fingerprint(fp))
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/morphism/ProjectionWitness`.
///
/// Theorem identity: `https://uor.foundation/op/surfaceSymmetry`.
///
/// # Errors
///
/// * `op:surfaceSymmetry` — `projection_source` (PartitionHandle) or
///   `projection_output` (SymbolSequenceHandle) is zero.
pub fn verify_morphism_projection_witness<H: HostTypes + 'static>(
    inputs: MintProjectionWitnessInputs<H>,
) -> Result<MintProjectionWitness, GenericImpossibilityWitness> {
    let src_fp = inputs.projection_source.fingerprint();
    if src_fp.is_zero() || inputs.projection_output.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/surfaceSymmetry",
        ));
    }
    let src_bytes = src_fp.as_bytes();
    let out_bytes = inputs.projection_output.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/surfaceSymmetry",
        src_bytes,
        out_bytes,
    ]);
    Ok(MintProjectionWitness::from_fingerprint(fp))
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/morphism/Witness` (abstract supertype).
///
/// MintWitnessInputs has zero fields (PhantomData<H> only) — there
/// are no structural invariants to check. Always succeeds; mints with
/// the IRI-derived fingerprint.
pub fn verify_morphism_witness<H: HostTypes + 'static>(
    _inputs: MintWitnessInputs<H>,
) -> Result<MintWitness, GenericImpossibilityWitness> {
    let fp = fingerprint_for_inputs(&[b"https://uor.foundation/op/surfaceSymmetry"]);
    Ok(MintWitness::from_fingerprint(fp))
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/state/GroundingWitness`.
///
/// Theorem identity: `https://uor.foundation/op/surfaceSymmetry`.
///
/// # Errors
///
/// * `op:surfaceSymmetry` — `witness_step == 0` (no decision-step
///   evidence) or `witness_binding` slice is empty (no Binding
///   handles attached).
pub fn verify_state_grounding_witness<H: HostTypes + 'static>(
    inputs: MintStateGroundingWitnessInputs<H>,
) -> Result<MintStateGroundingWitness, GenericImpossibilityWitness> {
    if inputs.witness_step == 0 || inputs.witness_binding.is_empty() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/surfaceSymmetry",
        ));
    }
    let step_bytes = inputs.witness_step.to_le_bytes();
    let binding_count_bytes = (inputs.witness_binding.len() as u64).to_le_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/surfaceSymmetry",
        &step_bytes,
        &binding_count_bytes,
    ]);
    Ok(MintStateGroundingWitness::from_fingerprint(fp))
}
