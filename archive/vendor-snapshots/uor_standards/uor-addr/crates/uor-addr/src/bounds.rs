//! `AddrBounds` — the single `HostBounds` capacity profile shared by
//! every UOR-ADDR realization (ADR-037), and the foundation-derived
//! inline-carrier width [`ADDR_INLINE_BYTES`] (ADR-060).
//!
//! ADR-060 removed the fixed per-ψ-stage byte-width ceilings
//! (`TERM_VALUE_MAX_BYTES`, `AXIS_OUTPUT_BYTES_MAX`,
//! `ROUTE_INPUT_BUFFER_BYTES`, …): a realization's canonical form flows
//! through the pipeline as a source-polymorphic [`prism::operation::TermValue`]
//! carrier (`Borrowed` / `Stream`) with no size cap. The remaining
//! `HostBounds` constants are structural-count / catamorphism-trace caps.
//! [`AddrBounds`] (FP_MAX = 32) serves the four 32-byte axes; [`AddrBounds64`]
//! (FP_MAX = 64) serves the `Sha512Hasher` axis — foundation 0.5.2
//! generalized the resolver tower over the fingerprint-width const generic,
//! so a 64-byte σ-axis composes. The two profiles differ only in the
//! fingerprint ceiling and the site-count ceilings (sized to the widest
//! κ-label geometry each admits: keccak256's 74 sites vs sha512's 135).

use prism::uor_foundation::pipeline::carrier_inline_bytes;
use prism::vocabulary::HostBounds;

/// The shared capacity profile. Every realization's `PrismModel` binds
/// this `B`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddrBounds;

impl HostBounds for AddrBounds {
    const FINGERPRINT_MIN_BYTES: usize = 32;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 32;

    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 74;
    const NERVE_CONSTRAINTS_MAX: usize = 128;
    const NERVE_SITES_MAX: usize = 74;
    const JACOBIAN_SITES_MAX: usize = 74;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 80;
    const CONJUNCTION_TERMS_MAX: usize = 128;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

/// The foundation-derived inline-carrier width for [`AddrBounds`]
/// (ADR-060). For the SHA-256 σ-axis this is the κ-label ASCII width
/// (`sha256:` + 64 hex = 71) rounded up by the hasher-identifier header —
/// large enough for the Inline κ-label ψ₉ emits, and unrelated to input
/// size (large inputs flow as `Borrowed` / `Stream`).
pub const ADDR_INLINE_BYTES: usize = carrier_inline_bytes::<AddrBounds>();

/// The 64-byte-fingerprint capacity profile, bound by the `Sha512Hasher`
/// σ-axis (`Hasher<64>`). Identical to [`AddrBounds`] except the doubled
/// fingerprint ceiling and the site-count ceilings widened to admit
/// sha512's 135-site κ-label geometry (`sha512:` + 128 hex).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddrBounds64;

impl HostBounds for AddrBounds64 {
    const FINGERPRINT_MIN_BYTES: usize = 32;
    const FINGERPRINT_MAX_BYTES: usize = 64;
    const TRACE_MAX_EVENTS: usize = 256;
    const WITT_LEVEL_MAX_BITS: u32 = 32;

    const FOLD_UNROLL_THRESHOLD: usize = 8;
    const BETTI_DIMENSION_MAX: usize = 135;
    const NERVE_CONSTRAINTS_MAX: usize = 256;
    const NERVE_SITES_MAX: usize = 135;
    const JACOBIAN_SITES_MAX: usize = 135;
    const RECURSION_TRACE_DEPTH_MAX: usize = 16;
    const OP_CHAIN_DEPTH_MAX: usize = 8;
    const AFFINE_COEFFS_MAX: usize = 144;
    const CONJUNCTION_TERMS_MAX: usize = 256;
    const UNFOLD_ITERATIONS_MAX: usize = 256;
}

/// The foundation-derived inline-carrier width for [`AddrBounds64`]
/// (ADR-060): `HASHER_IDENTIFIER_BYTES + 1 + 2 × 64 = 161`, large enough
/// for the 135-byte `sha512:<128hex>` Inline κ-label.
pub const ADDR_INLINE_BYTES_64: usize = carrier_inline_bytes::<AddrBounds64>();
