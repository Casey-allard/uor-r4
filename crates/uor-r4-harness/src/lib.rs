//! The multi-lab research harness, and its only implementation.
//!
//! **Why this exists.** D27 (owner, 13 October 2026) makes Rust the default for all new project
//! code *including tooling*. The harness began as Python in `scripts/harness/`; this crate ported
//! it command by command against the formats frozen in `docs/harness/state-formats.json`, and on
//! 20 October 2026 the owner retired the Python mirror (rule: Rust only). Its source is kept
//! byte-identical under `archive/scripts/harness/` as the record the port was checked against.
//!
//! ```
//! # use std::path::Path;
//! let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
//! let contract = uor_r4_harness::Contract::load(&repo).expect("contract loads");
//! let report = contract.check(&repo);
//! // The contract is checked against whatever state is present; an empty checkout is not a failure.
//! assert!(report.formats_checked > 0);
//! ```

pub mod arena;
pub mod arenacompare;
pub mod arenarules;
pub mod attention_reference;
pub mod briefing;
pub mod briefingrules;
pub mod budget;
pub mod budgetrules;
pub mod checkdispatch;
pub mod contract;
pub mod doccheck;
pub mod githelp;
pub mod githubcards;
pub mod harnessrules;
pub mod harnessrules_stateformats;
pub mod heartbeatcard;
pub mod hygiene;
pub mod labs;
pub mod looparrive;
pub mod loopbench;
pub mod loopboard;
pub mod loopclaim;
pub mod loopclose;
pub mod loopdrive;
pub mod loopgates;
pub mod loopguard;
pub mod loopnext;
pub mod loopprcheck;
pub mod loopqueue;
pub mod looprender;
pub mod loopreport;
pub mod loopreview;
pub mod loopsub;
pub mod mistakes;
pub mod multilab;
pub mod multilabrules;
pub mod notes;
pub mod onboard;
pub mod ordered;
pub mod panels;
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
pub mod statusdelivery;
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
