//! UOR-Framework host bounds for the R4 pipeline (moved verbatim from the
//! historical engine in `lib.rs`; depends only on `uor_foundation`).

use uor_foundation::HostBounds;

// Custom R4 Host Bounds
#[derive(Clone, Copy)]
pub struct R4HostBounds;

impl HostBounds for R4HostBounds {
    const FINGERPRINT_MIN_BYTES: usize = 16;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 64;
    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 8;
    const NERVE_CONSTRAINTS_MAX: usize = 8;
    const NERVE_SITES_MAX: usize = 8;
    const JACOBIAN_SITES_MAX: usize = 8;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 8;
    const CONJUNCTION_TERMS_MAX: usize = 8;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

pub const R4_INLINE_BYTES: usize = uor_foundation::pipeline::carrier_inline_bytes::<R4HostBounds>();
pub const R4_FP_MAX: usize = 32;
