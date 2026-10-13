//! Fixed geometry and exact storage support for the qualified CUDA path.
//! Historical learner/mod.rs is retained but cannot be feature-enabled here.
#[path = "fixed_embedding.rs"]
pub mod embedding;
pub mod group_table;
#[path = "source_hash.rs"]
pub mod realtext_support;
#[path = "scoped_store.rs"]
pub mod scoped_memory;
pub use group_table::{group_table, GroupTable, GROUP_ORDER, ROW_STRIDE};
