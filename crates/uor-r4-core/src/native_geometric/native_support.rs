//! Qualified native support boundary: raw corpus, fixed geometry and exact storage.
//! The historical learned/host-executed model is preserved in mod.rs, not compiled.
pub mod hopf_metric;
#[path = "learner/native_support.rs"]
pub mod learner;
pub mod mmap_corpus;
