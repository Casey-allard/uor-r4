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

pub mod arenarules;
pub mod briefing;
pub mod briefingrules;
pub mod budgetrules;
pub mod contract;
pub mod githelp;
pub mod harnessrules;
pub mod harnessrules_stateformats;
pub mod mistakes;
pub mod multilab;
pub mod multilabrules;
pub mod notes;
pub mod onboard;
pub mod prbody;
pub mod prep;
pub mod preprules;
pub mod presence;
pub mod prgates;
pub mod prlintfixtures;
pub mod prlintrules;
pub mod ratchet;
pub mod receipt;
pub mod receiptsrules;
pub mod registries;
pub mod ruleinventory;
pub mod shards;
pub mod skills;
pub mod state;
pub mod status;
pub mod statusrules;
pub mod statusrules_heartbeat;
pub mod team;
pub mod version;

pub use briefing::{assemble, human_tokens, ContextPack};
pub use contract::{Contract, FormatSpec, Kind, Shape, Violation};
pub use notes::{Markers, Note, Unread};
pub use onboard::{onboard, Finding, Level, Report};
pub use prep::{check_all, check_sheet, test_fitness, SheetReport};
pub use presence::{heartbeat, Presence, PRESENCE_TTL_MINUTES};
pub use receipt::{append, shard_path, Receipt};
pub use registries::{fmt_num, Milestone, Panel, Registries};
pub use shards::{check_ledger, compare, ShardReport};
pub use state::{PresenceRow, ReceiptRow, StatePaths};
pub use status::{milestone_rows, render as render_status, MilestoneRow};
