//! Bounded Hamming reads with contextual refinement before emission.
//!
//! This reusable primitive has no retained dispatch, emission model or trained
//! policy. Exact payloads and all four contextual roots reach the policy; policy
//! implementations must bind their actual parameters. Geometry and runtime use
//! finite integer/table operations, not weighted vector contractions.
pub mod metric;
#[cfg(test)]
mod report;
pub mod runtime;
#[cfg(test)]
mod tests;
