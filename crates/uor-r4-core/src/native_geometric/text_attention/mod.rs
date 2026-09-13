//! Bounded grounded text streams with actual emitted-byte state feedback.
//! Exact span bindings and fixed query encoding remain supplied.
pub mod data;
pub mod learning;
#[cfg(test)]
mod report;
pub mod runtime;
