//! PrismPM's Hologram application code, vendored verbatim from `crates/prismpm/src/holo/` at the
//! PrismPM commit `PRISMPM_REV` pins (f57a697). `archive.rs` composes and validates the closed
//! Holo/1 profile (one Core-Wasm guest, one portable View, the empty capability request, the
//! application directory and the Prism provenance extension); `canonical.rs`, `model_document.rs`
//! and `validate.rs` are what it depends on. The lane refuses a build whose vendored files differ
//! from PrismPM's at the pinned commit (`tools/prismpm_vendor.sha256`), so the composer in the
//! browser is PrismPM's own, not a second writer. This file is the only one written here.

pub mod archive;
pub mod canonical;
pub mod model_document;
pub mod validate;
