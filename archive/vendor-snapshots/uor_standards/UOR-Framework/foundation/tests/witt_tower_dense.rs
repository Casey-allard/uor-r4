//! v0.2.2 T2.4 (cleanup): Phase C dense Witt tower marker-struct existence.
//!
//! The Phase C.1/C.2 dense tower (W40..W128, native u64/u128 backing) is
//! emitted as a set of `pub struct WN;` marker types. This test locks
//! their existence via type-check assertions — if a regression deletes
//! any marker, the test fails to compile.

#![allow(dead_code)]

use uor_foundation::enforcement::{W104, W112, W120, W128, W40, W48, W56, W64, W72, W80, W88, W96};

fn _phase_c1_dense_u64_markers_exist() {
    let _: W40 = W40;
    let _: W48 = W48;
    let _: W56 = W56;
    let _: W64 = W64;
}

fn _phase_c2_dense_u128_markers_exist() {
    let _: W72 = W72;
    let _: W80 = W80;
    let _: W88 = W88;
    let _: W96 = W96;
    let _: W104 = W104;
    let _: W112 = W112;
    let _: W120 = W120;
    let _: W128 = W128;
}

#[test]
fn phase_c_dense_markers_pinned() {
    _phase_c1_dense_u64_markers_exist();
    _phase_c2_dense_u128_markers_exist();
}
