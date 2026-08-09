#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod engine;
pub mod packed_kernels;
pub mod patch_chain;
pub mod routing;
pub mod runtime_state;
pub mod scoring;
pub mod status;

mod vp_tree;

pub use engine::R4G1Runtime;
pub use status::ResolutionStatus;
