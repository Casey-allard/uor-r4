//! Executor-independent raw tokenizer APIs.
//! The original mod.rs and compiler/runtime/SIMD closure remain inactive evidence.
//! No Cargo feature enables that historical numerical path.
pub mod bpe_derive;
pub mod hf_bpe;
#[path = "tokenizer_interface.rs"]
pub mod scenarios;
#[cfg(not(target_arch = "wasm32"))]
pub mod sentencepiece;
pub mod source_scan;
