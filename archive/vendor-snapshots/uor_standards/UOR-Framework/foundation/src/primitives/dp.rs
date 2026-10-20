// @codegen-exempt — Phase 15 hand-written verification bodies for the DP family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `dp` (Disjointness via FX_)
//! theorem family.
//!
//! `verify_effect_disjointness_witness` validates a populated
//! [`MintDisjointnessWitnessInputs<H>`] against op:FX_4 ("disjoint
//! effects commute"). The structural invariant is that the two
//! `EffectTargetHandle<H>` operands are non-zero and distinct.

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{MintDisjointnessWitness, MintDisjointnessWitnessInputs};
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
/// `https://uor.foundation/effect/DisjointnessWitness`.
///
/// Theorem identity: `https://uor.foundation/op/FX_4` (disjoint
/// effects commute).
///
/// # Errors
///
/// * `op:FX_4` — `disjointness_left` or `disjointness_right` is the
///   zero handle, or both refer to the same content (not actually
///   disjoint).
pub fn verify_effect_disjointness_witness<H: HostTypes + 'static>(
    inputs: MintDisjointnessWitnessInputs<H>,
) -> Result<MintDisjointnessWitness, GenericImpossibilityWitness> {
    if inputs.disjointness_left.fingerprint.is_zero()
        || inputs.disjointness_right.fingerprint.is_zero()
    {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/FX_4",
        ));
    }
    if inputs.disjointness_left.fingerprint == inputs.disjointness_right.fingerprint {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/FX_4",
        ));
    }
    let left_bytes = inputs.disjointness_left.fingerprint.as_bytes();
    let right_bytes = inputs.disjointness_right.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[b"https://uor.foundation/op/FX_4", left_bytes, right_bytes]);
    Ok(MintDisjointnessWitness::from_fingerprint(fp))
}
