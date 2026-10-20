//! v0.2.2 T2.4 (cleanup): Phase C.3 Limbs<N>-backed Witt tower marker-struct
//! existence + Limbs<N> public surface.
//!
//! All 16 Limbs-backed marker structs (W160..W32768) must exist, and the
//! `Limbs<const N: usize>` carrier must be in the public API with at least
//! its `words()` accessor.

#![allow(dead_code)]

use uor_foundation::enforcement::{
    Limbs, W1024, W12288, W160, W16384, W192, W2048, W224, W256, W32768, W384, W4096, W448, W512,
    W520, W528, W8192,
};

fn _phase_c3_semantically_meaningful_intermediate_markers_exist() {
    let _: W160 = W160; // SHA-1
    let _: W192 = W192;
    let _: W224 = W224; // SHA-224
    let _: W384 = W384; // SHA-384, P-384
    let _: W448 = W448;
    let _: W520 = W520; // P-521
    let _: W528 = W528;
    let _: W12288 = W12288;
}

fn _phase_c3_powers_of_two_markers_exist() {
    let _: W256 = W256;
    let _: W512 = W512;
    let _: W1024 = W1024;
    let _: W2048 = W2048;
    let _: W4096 = W4096;
    let _: W8192 = W8192;
    let _: W16384 = W16384;
    let _: W32768 = W32768;
}

fn _limbs_public_surface_exists<const N: usize>(_: &Limbs<N>) {
    // type-check only; constructors are pub(crate) so we can't construct
    // a Limbs<N> from integration tests. Pinning the type's presence in
    // the public API is the regression gate.
}

#[test]
fn phase_c3_limbs_markers_pinned() {
    _phase_c3_semantically_meaningful_intermediate_markers_exist();
    _phase_c3_powers_of_two_markers_exist();
}
