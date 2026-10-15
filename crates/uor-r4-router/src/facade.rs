//! R4 router support surface (crate root).
//!
//! #2345 retired the floating-point `uor-r4-core` surface. The historical
//! `UorR4Router` engine (`src/lib.rs`, `geometry`, `decoder_memory`,
//! `benchmark`) is built on those retired APIs; it is preserved in place as
//! evidence behind the non-default `legacy-engine` feature and does not build
//! against the current core. This root compiles only the engine-free pieces:
//! the token-history session signature, the tier cascade and the R4 UOR host
//! bounds, all of which build natively and on wasm32.

pub mod fallback;
pub mod host_bounds;
pub mod session_signature;

pub use host_bounds::{R4HostBounds, R4_FP_MAX, R4_INLINE_BYTES};
pub use session_signature::{
    from_state as session_signature_from_state, from_tokens as session_signature_from_tokens,
    TokenHistorySignature, MULTI_TURN_FIXTURE,
};

#[cfg(feature = "legacy-engine")]
pub use session_signature::fixture_session_signatures;

// #790 item 4: only the live cascade surface is exported — the dead
// FallbackRouter pipeline and its satellite types were removed.
pub use fallback::{run_cascade, CascadeOutcome, EngineStatus, TierFn, TierOutcome, TierResult};

/// Historical engine on retired core APIs (#2345); preserved as evidence.
#[cfg(feature = "legacy-engine")]
#[path = "lib.rs"]
mod legacy_engine;
#[cfg(feature = "legacy-engine")]
pub use legacy_engine::*;
