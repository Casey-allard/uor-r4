//! Exact Showcase-compatible byte storage. No model or learned routing is implemented.
//! A manifest must be pinned by the caller; its digest is not an authorization signature.
#![forbid(unsafe_code)]

pub mod bindings;
mod hash;
mod manifest;
pub mod peer;
mod record;
mod store;
pub use manifest::Manifest;
pub use peer::PullStats;
pub use record::Record;
use std::fmt;
pub use store::{
    FoldStats, Folded, ImportStats, RangeVerifiedAgainstRoot, ReadStats, Store,
    WholeObjectDigestVerified,
};
pub type NodeId = [u8; 12];
pub type ObjectVersion = [u8; 32];
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Missing(NodeId),
    SchemaMismatch,
    IntegrityFailure,
    InvalidRange,
    LimitExceeded(&'static str),
    BindingMismatch,
    Utf8,
    /// Classifiable I/O failure; OS detail is retained without parsing prose.
    Io {
        kind: std::io::ErrorKind,
        raw_os_error: Option<i32>,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            kind: e.kind(),
            raw_os_error: e.raw_os_error(),
        }
    }
}

/// Limits apply before expansion/allocation; they are storage policy, not model bars.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_object_bytes: u64,
    pub max_records: usize,
    pub max_record_bytes: usize,
    pub max_returned_bytes: u64,
    pub max_visits: usize,
    pub max_occurrences: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_object_bytes: 16 * 1024 * 1024,
            max_records: 1_000_000,
            max_record_bytes: 64 * 1024 * 1024,
            max_returned_bytes: 16 * 1024 * 1024,
            max_visits: 1_000_000,
            max_occurrences: 1_000_000,
        }
    }
}
/// Lookup does not imply whole-object integrity. Implementations may return absence.
pub trait RecordBackend {
    fn fetch(&self, id: NodeId) -> Result<Option<Record>>;
}

pub(crate) fn span(scale: u8) -> Result<u64> {
    let mut n = 32u64;
    for _ in 0..scale {
        n = n.checked_mul(3).ok_or(Error::SchemaMismatch)?;
    }
    Ok(n)
}
pub(crate) fn reserve<T>(v: &mut Vec<T>, n: usize) -> Result<()> {
    v.try_reserve_exact(n)
        .map_err(|_| Error::LimitExceeded("allocation"))
}
