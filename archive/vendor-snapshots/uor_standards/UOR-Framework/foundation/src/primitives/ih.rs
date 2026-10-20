// @codegen-exempt — Phase 15 hand-written verification bodies for the IH family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `ih` (Inhabitance) theorem family.
//!
//! Two functions: `verify_proof_impossibility_witness` checks the abstract
//! ImpossibilityWitness shape; `verify_proof_inhabitance_impossibility_witness`
//! adds the InhabitanceImpossibility-specific structural invariants
//! (op:IH_1..IH_3).

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{
    MintImpossibilityWitness, MintImpossibilityWitnessInputs, MintInhabitanceImpossibilityWitness,
    MintInhabitanceImpossibilityWitnessInputs,
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
/// `https://uor.foundation/proof/ImpossibilityWitness`.
///
/// Theorem identity: `https://uor.foundation/op/IH_1` (inhabitance
/// soundness — bidirectional).
///
/// # Errors
///
/// * `op:IH_1` — `verified == false`.
/// * `op:IH_2a` — `impossibility_reason` is the empty sentinel.
/// * `op:IH_2b` — `proves_identity` handle is the zero sentinel
///   (no theorem identity attested).
pub fn verify_proof_impossibility_witness<H: HostTypes + 'static>(
    inputs: MintImpossibilityWitnessInputs<H>,
) -> Result<MintImpossibilityWitness, GenericImpossibilityWitness> {
    if !inputs.verified {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_1",
        ));
    }
    if core::ptr::eq(
        inputs.impossibility_reason as *const _,
        H::EMPTY_HOST_STRING as *const _,
    ) {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_2a",
        ));
    }
    if inputs.proves_identity.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_2b",
        ));
    }
    let proves_bytes = inputs.proves_identity.fingerprint.as_bytes();
    let formal_bytes = inputs.formal_derivation.fingerprint.as_bytes();
    let verified_byte = [u8::from(inputs.verified)];
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/IH_1",
        proves_bytes,
        formal_bytes,
        &verified_byte,
    ]);
    Ok(MintImpossibilityWitness::from_fingerprint(fp))
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/proof/InhabitanceImpossibilityWitness`.
///
/// Inherits ImpossibilityWitness's invariants (IH_1, IH_2a, IH_2b)
/// and adds:
///
/// # Errors
///
/// * `op:IH_3` — `contradiction_proof` is empty (no negation evidence).
/// * `op:IH_3` — `grounded` handle is zero (no ConstrainedType
///   reference for the impossibility claim).
/// * `op:IH_3` — `search_trace` handle is zero (no decision trace).
pub fn verify_proof_inhabitance_impossibility_witness<H: HostTypes + 'static>(
    inputs: MintInhabitanceImpossibilityWitnessInputs<H>,
) -> Result<MintInhabitanceImpossibilityWitness, GenericImpossibilityWitness> {
    // Inherited invariants from ImpossibilityWitness.
    if !inputs.verified {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_1",
        ));
    }
    if core::ptr::eq(
        inputs.impossibility_reason as *const _,
        H::EMPTY_HOST_STRING as *const _,
    ) {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_2a",
        ));
    }
    if inputs.proves_identity.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_2b",
        ));
    }
    // InhabitanceImpossibility-specific invariants.
    if core::ptr::eq(
        inputs.contradiction_proof as *const _,
        H::EMPTY_HOST_STRING as *const _,
    ) {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_3",
        ));
    }
    if inputs.grounded.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_3",
        ));
    }
    if inputs.search_trace.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/IH_3",
        ));
    }
    let proves_bytes = inputs.proves_identity.fingerprint.as_bytes();
    let grounded_bytes = inputs.grounded.fingerprint.as_bytes();
    let trace_bytes = inputs.search_trace.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/IH_1",
        proves_bytes,
        grounded_bytes,
        trace_bytes,
    ]);
    Ok(MintInhabitanceImpossibilityWitness::from_fingerprint(fp))
}
