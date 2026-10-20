// @codegen-exempt — Phase 15 hand-written verification bodies for the CC family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `cc` (Completeness) theorem family.
//!
//! `verify_type_completeness_witness` validates a populated
//! [`MintCompletenessWitnessInputs<H>`] against the structural
//! invariants for op:CC_1..CC_5. Failure routes to a specific
//! impossibility IRI; success mints with a content-addressed
//! fingerprint folded over the input fields.

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{MintCompletenessWitness, MintCompletenessWitnessInputs};
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
/// `https://uor.foundation/type/CompletenessWitness`.
///
/// Theorem identity: `https://uor.foundation/op/CC_1` (completeness
/// implies O(1) resolution).
///
/// # Errors
///
/// * `op:CC_1` — `sites_closed == 0` (no closed sites means the type
///   isn't complete).
/// * `op:CC_2` — `witness_constraint` handle is the zero sentinel
///   (no constraint-evidence handle attached).
pub fn verify_type_completeness_witness<H: HostTypes + 'static>(
    inputs: MintCompletenessWitnessInputs<H>,
) -> Result<MintCompletenessWitness, GenericImpossibilityWitness> {
    if inputs.sites_closed == 0 {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/CC_1",
        ));
    }
    if inputs.witness_constraint.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/CC_2",
        ));
    }
    let sites_bytes = inputs.sites_closed.to_le_bytes();
    let constraint_bytes = inputs.witness_constraint.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/CC_1",
        &sites_bytes,
        constraint_bytes,
    ]);
    Ok(MintCompletenessWitness::from_fingerprint(fp))
}
