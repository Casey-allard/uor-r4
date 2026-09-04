//! Opt-in native Four-fact workbench specified by ADR-0006.
//!
//! This crate is a CPU floating-point research host. Its presence and a
//! successful build do not qualify a binary or establish service behavior.

pub mod authority;
pub mod base64;
pub mod comparison;
pub mod host;
pub mod http;
pub mod intake;
pub mod ipc;
pub mod launch;
pub mod lifecycle;
pub mod strict_json;
pub mod wire;
pub mod worker;

pub const SERVICE_CONTRACT_SHA256: &str =
    "337d66d025fc9ec3a1e8c21befc25198b015061235fefc98f9208f99412e7a7f";
pub const SERVICE_CONTRACT_SCHEMA: &str = "uor-r4.service-api-contract/1";
pub const MODEL_ID: &str =
    "r4lr:sha256:2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab";
pub const ARTIFACT_BYTES: u64 = 2_172_252;
pub const TARGET: &str = "aarch64-apple-darwin";
pub const ORIGINAL_BINDING_SOURCE_SHA256: &str =
    "efc02551d493e255f12680ccf2e4ee99cca5f645e0ca3d7fcd6445419e963426";
pub const PRIVATE_RELEASE_CONTRACT_SHA256: &str =
    "3b639eb4171769de5101eafc594824d6711863a1e0836d1a3587907f0e5e1cff";

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
