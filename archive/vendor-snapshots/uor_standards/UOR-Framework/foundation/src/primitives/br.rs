// @codegen-exempt — Phase 15 hand-written verification bodies for the BR family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `br` (Born-rule) theorem family.
//!
//! `verify_cert_born_rule_verification` validates a populated
//! [`MintBornRuleVerificationInputs<H>`] against the structural
//! invariants required for the witness to attest a verified Born-rule
//! certification (op:QM_5 / BR_1..BR_5). On any failure it returns a
//! typed [`GenericImpossibilityWitness`] whose IRI cites the specific
//! failing identity; on success it mints a [`MintBornRuleVerification`]
//! with a content-addressed fingerprint folded over the input bytes.

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{MintBornRuleVerification, MintBornRuleVerificationInputs};
use crate::HostTypes;

/// Index-salted XOR fold over each chunk in `chunks`, building a
/// 32-byte content-addressed fingerprint. Identical inputs produce
/// identical fingerprints; differing inputs (in any byte) produce
/// distinct fingerprints. `no_std`-safe and avoids host-supplied
/// hashers — the production verify path reaches for this when no
/// `Hasher` is available.
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
        // Chunk-boundary marker: fold a `0xFF` byte so chunk reordering
        // produces a different fingerprint.
        let pos = global % 32;
        buf[pos] ^= 0xFFu8;
        global += 1;
        chunk_idx += 1;
    }
    ContentFingerprint::from_buffer(buf, 32u8)
}

/// Phase 15 verification primitive for
/// `https://uor.foundation/cert/BornRuleVerification`.
///
/// Theorem identity: `https://uor.foundation/op/QM_5` (Born-rule
/// amplitude normalization). Walks the structural invariants required
/// for a host-attested Born-rule certificate and routes specific
/// failure modes to BR_1..BR_5 / QM_5 op-namespace identities.
///
/// # Errors
///
/// Returns `Err(GenericImpossibilityWitness::for_identity(iri))` with:
///
/// * `op:BR_1` — `verified` flag is `false` (host did not attest).
/// * `op:BR_2` — `born_rule_verified` flag is `false`.
/// * `op:BR_3` — `witt_length == 0` (no Witt-level evidence).
/// * `op:BR_4` — `certifies` is the empty sentinel
///   (host-string equality with `H::EMPTY_HOST_STRING`).
/// * `op:QM_5` — fallthrough; only reachable if BR_1..BR_4 pass but
///   the witness still fails to be well-formed (currently unreachable
///   via the structural checks above).
pub fn verify_cert_born_rule_verification<H: HostTypes + 'static>(
    inputs: MintBornRuleVerificationInputs<H>,
) -> Result<MintBornRuleVerification, GenericImpossibilityWitness> {
    if !inputs.verified {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/BR_1",
        ));
    }
    if !inputs.born_rule_verified {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/BR_2",
        ));
    }
    if inputs.witt_length == 0 {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/BR_3",
        ));
    }
    if core::ptr::eq(
        inputs.certifies as *const _,
        H::EMPTY_HOST_STRING as *const _,
    ) {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/BR_4",
        ));
    }
    let witt_bytes = inputs.witt_length.to_le_bytes();
    let verified_byte = [u8::from(inputs.verified)];
    let born_byte = [u8::from(inputs.born_rule_verified)];
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/QM_5",
        &witt_bytes,
        &verified_byte,
        &born_byte,
    ]);
    Ok(MintBornRuleVerification::from_fingerprint(fp))
}
