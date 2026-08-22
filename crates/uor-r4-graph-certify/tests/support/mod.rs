//! Shared reference-support modules for the #845 geometry-qualification
//! instruments.
//!
//! Files in subdirectories of `tests/` are not test binaries; each test file
//! that declares `mod support;` compiles its own copy, so the shared surface
//! is allowed to be partially used per binary.

pub mod arms;
pub mod episode;
pub mod ordering;
pub mod stats;
pub mod w33;
