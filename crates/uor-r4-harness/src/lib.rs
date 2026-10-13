//! The multi-lab research harness, on the Rust side of the port.
//!
//! **Why this exists.** D27 (owner, 13 October 2026) makes Rust the default for all new project
//! code *including tooling*, and grandfathers the Python harness in `scripts/harness/`. The
//! formats that harness persists are frozen in `docs/harness/state-formats.json`, and this crate
//! reads the same files, checks them against the same contract, and must agree with Python about
//! what they mean. Two implementations checking one contract is what makes the port verifiable
//! rather than a rewrite.
//!
//! **What it does not do yet.** The Python harness owns behaviour until this path passes the same
//! checks. This crate currently reads and validates state; it does not write, and it is not on the
//! labs' path. Claiming otherwise would be the mistake this project has made before.
//!
//! ```
//! # use std::path::Path;
//! let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
//! let contract = uor_r4_harness::Contract::load(&repo).expect("contract loads");
//! let report = contract.check(&repo);
//! // The contract is checked against whatever state is present; an empty checkout is not a failure.
//! assert!(report.formats_checked > 0);
//! ```

pub mod contract;
pub mod state;

pub use contract::{Contract, FormatSpec, Kind, Shape, Violation};
pub use state::{Presence, Receipt, StatePaths};
