//! The sanctioned failure meanings (R5).
//!
//! R5 admits no arbitrary limitation. The only conditions a shipped crate may
//! report are that the requested object does not exist ([`NotAProduct`]), that a
//! kappa did not reproduce ([`KappaError`]), or that a measured bound was
//! exceeded ([`ObservedBound`]). This module is that vocabulary.
//!
//! The parse layer returns [`NotAProduct`] / [`KappaError`] instead of the
//! internal [`FormatError`](crate::FormatError) taxonomy. `FormatError` is not
//! deleted: it is preserved as the *reason* a product was rejected and carried
//! inside `NotAProduct`, so the validation and its diagnostics are unchanged ---
//! only the returned *contract* is regularized to the sanctioned set.

use core::fmt;

use crate::error::FormatError;

/// Which object was requested but is not a product of the given bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// The whole R4G1 graph artifact.
    Graph,
    /// The fixed 88-byte header.
    Header,
    /// The HEAD section.
    Head,
    /// A section body.
    Section,
    /// The n-gram table.
    NgramTable,
    /// The forward-anchor table.
    FwdaTable,
    /// The FMM translation table.
    FmmTable,
    /// A routing (ROUT) program.
    RoutingProgram,
    /// A CODE program.
    CodeProgram,
    /// A packed record block.
    Records,
    /// A serialized artifact under construction.
    Artifact,
    /// A route-attention operator instance (#604, `route_attention`).
    RouteAttentionInstance,
    /// A route-attention step result (#604): the query the step was
    /// asked to score is not a route code of the instance's width.
    RouteAttentionStep,
    /// An MSA-selector operator instance (#643, `msa_selector`).
    MsaSelectorInstance,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ObjectKind::Graph => "graph",
            ObjectKind::Header => "header",
            ObjectKind::Head => "head",
            ObjectKind::Section => "section",
            ObjectKind::NgramTable => "n-gram table",
            ObjectKind::FwdaTable => "forward-anchor table",
            ObjectKind::FmmTable => "FMM translation table",
            ObjectKind::RoutingProgram => "routing program",
            ObjectKind::CodeProgram => "CODE program",
            ObjectKind::Records => "record block",
            ObjectKind::Artifact => "artifact",
            ObjectKind::RouteAttentionInstance => "route-attention instance",
            ObjectKind::RouteAttentionStep => "route-attention step",
            ObjectKind::MsaSelectorInstance => "msa-selector instance",
        };
        f.write_str(s)
    }
}

/// The requested object is not a product of these bytes (R5).
///
/// `object` is what was asked for; `reason` is the observed malformation,
/// preserved so a caller that wants the structural detail still has it while the
/// returned contract names only the sanctioned condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAProduct {
    /// The object that does not exist as a product of the bytes.
    pub object: ObjectKind,
    /// Why the bytes are not that object.
    pub reason: FormatError,
}

impl NotAProduct {
    /// The bytes are not `object`, because `reason`.
    pub const fn new(object: ObjectKind, reason: FormatError) -> Self {
        Self { object, reason }
    }
}

/// A bare [`FormatError`] names the whole graph as the absent object; a function
/// parsing a more specific object constructs [`NotAProduct::new`] with its kind.
/// This is what lets `?` propagate a structural failure without per-site
/// ceremony.
impl From<FormatError> for NotAProduct {
    fn from(reason: FormatError) -> Self {
        Self {
            object: ObjectKind::Graph,
            reason,
        }
    }
}

impl fmt::Display for NotAProduct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a {}: {}", self.object, self.reason)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NotAProduct {}

/// A kappa (BLAKE3 CID) did not reproduce (R5) --- the only failure a
/// reproduction check may report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KappaError {
    /// The HEAD section is absent, so its CID cannot be computed or checked.
    MissingHead,
    /// `head_cid` does not recompute to the HEAD body.
    Head,
    /// `artifact_cid` does not recompute to `artifact_bytes[56..]`.
    Artifact,
    /// `tokenizer_cid` does not match the loaded tokenizer hash.
    Tokenizer,
}

impl KappaError {
    /// The equivalent [`FormatError`] variant, for a caller not yet migrated to
    /// the sanctioned surface (R5 tranche 1b removes these bridges).
    pub const fn as_format(self) -> FormatError {
        match self {
            KappaError::MissingHead => FormatError::MissingHead,
            KappaError::Head => FormatError::HeadCidMismatch,
            KappaError::Artifact => FormatError::ArtifactCidMismatch,
            KappaError::Tokenizer => FormatError::TokenizerCidMismatch,
        }
    }
}

impl fmt::Display for KappaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            KappaError::MissingHead => "HEAD absent: no CID to reproduce",
            KappaError::Head => "head_cid does not reproduce",
            KappaError::Artifact => "artifact_cid does not reproduce",
            KappaError::Tokenizer => "tokenizer_cid does not match",
        };
        f.write_str(s)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for KappaError {}

/// A measured bound was exceeded (R5) --- a property of the observed input, not
/// a limitation of the code. Carries the observed value and the bound it
/// crossed, so it reports rather than refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedBound {
    /// The value that was observed.
    pub observed: i64,
    /// The bound it crossed.
    pub bound: i64,
}

impl fmt::Display for ObservedBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "observed {} past bound {}", self.observed, self.bound)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ObservedBound {}
