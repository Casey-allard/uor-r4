//! The BDD runner and the honesty meta-gate (R2, R3, `CM-02`, `CM-03`).
//!
//! Two jobs, and they are different in kind.
//!
//! **R3, feature-first.** Every capability begins as a Gherkin scenario. The
//! runner below reads `features/suites/*.feature`, extracts the conformance IDs
//! the scenarios are tagged with, and cross-checks them against the register
//! and against the test names in the workspace. An ID with no scenario, a
//! scenario with no ID, and an ID with no test are all failures. There are no
//! pending or skipped steps, because a pending step is a claim with nothing
//! behind it.
//!
//! **R2, the meta-gate.** The honesty levels only mean something if the suite
//! respects them. The gate proves that no `open` claim is asserted as
//! established and that no `some-true` claim is presented as though this
//! repository had established it. That is a check on the *documentation and the
//! test names*, not on the arithmetic --- which is why it lives here rather
//! than in a numerical crate.

#![deny(missing_docs)]

pub mod meta;
pub mod runner;

pub use meta::{check_honesty, HonestyReport};
pub use runner::{scenarios_in, Scenario, SuiteReport};
