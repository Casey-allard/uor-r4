//! The compiled-turn action types: what a turn compiler returns and what the
//! exact store records for it.
//!
//! Moved without change from `stack_grounded_session`, which stays test-only,
//! so that lawful compilers and the prose-panel grader build outside tests.
//! `stack_grounded_session` re-exports every item here under its old path.

use std::fmt;

use serde::{Deserialize, Serialize};
use uor_r4_tokenizer::dialogue::DialogueError;

#[cfg(any())]
use crate::stack_checkpoint::StackCheckpointError;
use crate::stack_store::{HistoryView, StackStoreError, StoreRead, Written};
use crate::TrainingError;

/// An opaque store relation ID and its compiler label. Order is part of the
/// binding; IDs are not tokenizer IDs and must never be inferred from a hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationLabel {
    pub id: u32,
    pub name: String,
}

/// The compiler's feature model, if any. It can differ from the emitter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncoderIdentity {
    pub config_sha256: String,
    pub model_sha256: String,
    pub transport_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerIdentity {
    /// The adapter's artifact/schema version, including feature and act semantics.
    pub schema: String,
    pub artifact_sha256: String,
    pub tokenizer_sha256: String,
    pub label_schema: String,
    pub relations: Vec<RelationLabel>,
    pub encoder: Option<EncoderIdentity>,
}

/// Adapter for an already loaded compiler. Implementations must be stateless
/// across calls and predict from `source`, never evaluator labels. The artifact
/// must contain/bind the actual heads, normalization, labels and feature mode;
/// this consumer can check its bytes and identity, not certify its learning.
pub trait TurnCompiler {
    fn identity(&self) -> &CompilerIdentity;
    fn artifact_bytes(&self) -> &[u8];
    fn compile(&self, source: &str) -> Result<CompiledAction, GroundedSessionError>;
}

/// Half-open byte offsets into the original, unnormalized UTF-8 user text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CompiledAction {
    Assert {
        relation: u32,
        span: SourceSpan,
    },
    Correct {
        relation: u32,
        span: SourceSpan,
    },
    /// Legacy current-value action; its serialized form remains unchanged.
    QueryCurrent {
        relation: u32,
    },
    /// A compiler-predicted historical view, not a language heuristic here.
    Query {
        relation: u32,
        view: HistoryView,
    },
    Unresolved {
        reason: String,
    },
}

impl CompiledAction {
    pub fn query(&self) -> Option<(u32, HistoryView)> {
        match self {
            Self::QueryCurrent { relation } => Some((*relation, HistoryView::Current)),
            Self::Query { relation, view } => Some((*relation, *view)),
            _ => None,
        }
    }
}

/// The store effect of a compiled turn.
///
/// Current recall behaviour in `stack_grounded_session::resolve_recall_line` (extracted from the
/// historical `GroundedSession::emitter_suffix`, which remains `#[cfg(any())]` and unbuilt):
/// - [`MemoryEffect::Read`] with [`StoreRead::Absent`] represents a turn resolved to a relation
///   ID whose slot in the store is empty. `resolve_recall_line` returns `"Memory: none."`
///   (`RecallDisposition::Absent`) and does not invoke `log_recall`.
/// - [`MemoryEffect::Unresolved`] represents a turn that was not resolved to a store action;
///   when `read` is enabled, `resolve_recall_line` consults `log_recall` and may return
///   `RecallDisposition::LogValue`.
/// Whether [`StoreRead::Absent`] should also consult `log_recall` or remain strictly separate
/// is documented here as the current code's behaviour and queued for PM decision (`#2028`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MemoryEffect {
    Unresolved,
    Write {
        written: Written,
        value_tokens: Vec<u32>,
    },
    WriteDisabled {
        value_tokens: Vec<u32>,
    },
    Read {
        read: StoreRead,
    },
}

#[derive(Debug)]
pub enum GroundedSessionError {
    #[cfg(any())]
    Checkpoint(StackCheckpointError),
    Store(StackStoreError),
    Protocol(DialogueError),
    Training(TrainingError),
    Io(std::io::Error),
    Json(serde_json::Error),
    Binding(String),
    Input(String),
    Span(String),
    Compiler(String),
    Snapshot(String),
    Generation(String),
    Context {
        needed: usize,
        available: usize,
    },
    StorageLimit {
        resource: &'static str,
        needed: usize,
        limit: usize,
    },
}

impl fmt::Display for GroundedSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(any())]
            Self::Checkpoint(e) => write!(f, "grounded session checkpoint: {e}"),
            Self::Store(e) => write!(f, "grounded session store: {e}"),
            Self::Protocol(e) => write!(f, "grounded session protocol: {e}"),
            Self::Training(e) => write!(f, "grounded session model: {e}"),
            Self::Io(e) => write!(f, "grounded session I/O: {e}"),
            Self::Json(e) => write!(f, "grounded session JSON: {e}"),
            Self::Binding(s) => write!(f, "grounded session binding: {s}"),
            Self::Input(s) => write!(f, "grounded session input: {s}"),
            Self::Span(s) => write!(f, "grounded session span: {s}"),
            Self::Compiler(s) => write!(f, "grounded session compiler: {s}"),
            Self::Snapshot(s) => write!(f, "grounded session snapshot: {s}"),
            Self::Generation(s) => write!(f, "grounded session generation: {s}"),
            Self::Context { needed, available } => write!(
                f,
                "grounded session needs {needed} context positions; {available} available"
            ),
            Self::StorageLimit {
                resource,
                needed,
                limit,
            } => write!(
                f,
                "grounded session {resource} needs {needed}; limit is {limit}"
            ),
        }
    }
}

impl std::error::Error for GroundedSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(any())]
            Self::Checkpoint(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::Protocol(e) => Some(e),
            Self::Training(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

macro_rules! convert {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for GroundedSessionError {
            fn from(error: $ty) -> Self {
                Self::$variant(error)
            }
        }
    };
}
#[cfg(any())]
convert!(StackCheckpointError, Checkpoint);
convert!(StackStoreError, Store);
convert!(DialogueError, Protocol);
convert!(TrainingError, Training);
convert!(std::io::Error, Io);
convert!(serde_json::Error, Json);
