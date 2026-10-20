// @codegen-exempt — Phase 15 hand-written verification bodies for the LO family.
// emit::write_file's banner check preserves this file across `uor-crate` runs.

//! Phase 15 verification primitives for the `lo` (LiftObstruction)
//! theorem family. Routes through op:WLS_1 (trivial-obstruction
//! invariant) and op:WLS_2 (non-trivial obstruction localisation).

use crate::enforcement::{ContentFingerprint, GenericImpossibilityWitness};
use crate::witness_scaffolds::{MintLiftObstruction, MintLiftObstructionInputs};
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
/// `https://uor.foundation/type/LiftObstruction`.
///
/// Theorem identity: `https://uor.foundation/op/WLS_2` (non-trivial
/// LiftObstruction localised at a specific site). The invariant is
/// conditional on the `obstruction_trivial` flag: trivial → no
/// site reference; non-trivial → site reference required.
///
/// # Errors
///
/// * `op:WLS_1` — `obstruction_trivial == true` but
///   `obstruction_site` carries a non-zero fingerprint (a trivial
///   obstruction has no localised site).
/// * `op:WLS_2` — `obstruction_trivial == false` but
///   `obstruction_site` is the zero handle (a non-trivial
///   obstruction must localise to a specific site).
pub fn verify_type_lift_obstruction<H: HostTypes + 'static>(
    inputs: MintLiftObstructionInputs<H>,
) -> Result<MintLiftObstruction, GenericImpossibilityWitness> {
    if inputs.obstruction_trivial {
        if !inputs.obstruction_site.fingerprint.is_zero() {
            return Err(GenericImpossibilityWitness::for_identity(
                "https://uor.foundation/op/WLS_1",
            ));
        }
    } else if inputs.obstruction_site.fingerprint.is_zero() {
        return Err(GenericImpossibilityWitness::for_identity(
            "https://uor.foundation/op/WLS_2",
        ));
    }
    let trivial_byte = [u8::from(inputs.obstruction_trivial)];
    let site_bytes = inputs.obstruction_site.fingerprint.as_bytes();
    let fp = fingerprint_for_inputs(&[
        b"https://uor.foundation/op/WLS_2",
        &trivial_byte,
        site_bytes,
    ]);
    Ok(MintLiftObstruction::from_fingerprint(fp))
}
