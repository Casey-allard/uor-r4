//! A development consumer of a saved compiler, exact store and stack emitter.
//!
//! The compiler supplies predictions from its bound artifact. This module
//! validates their source spans and applies them; it neither fits a compiler
//! nor derives labels from an evaluator. The first interface has one explicit
//! scope/entity and typed versioned queries. It is a float development path,
//! not a D11 serving kernel or evidence of learned language capability.
//!
//! Turns retain the emitter's actual token IDs. Store and history changes
//! become visible together only after generation succeeds. Snapshots contain
//! a sealed model/store checkpoint, the opaque compiler artifact, tokenizer
//! and the generated history. Loading requires the caller's real compiler
//! adapter for those exact bytes; no fallback classifier exists here.
//! Seals and hashes check integrity and consistency, not authentication.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

use candle_core::Device;
use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_core::native_geometric::learner::scoped_memory::Record;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueError, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_stack::StackModel;
use crate::stack_checkpoint::{
    load_checkpoint, save_checkpoint, sealed_manifest_sha256, CheckpointIdentity, StackCheckpoint,
    StackCheckpointError,
};
use crate::stack_dialogue::{greedy_reply, Reply};
use crate::stack_store::{
    HistoryView, StackStore, StackStoreError, StoreRead, StoreValue, Update, Written,
};
use crate::TrainingError;

pub const SESSION_SCHEMA: &str = "uor-r4.grounded-session/1";
/// Used only when a transcript contains the additive typed `Query` action.
pub const TEMPORAL_SESSION_SCHEMA: &str = "uor-r4.grounded-session/2";
pub const SESSION_FILE: &str = "session.json";
pub const COMPILER_FILE: &str = "compiler.bin";
pub const TOKENIZER_FILE: &str = "tokenizer.json";
pub const CHECKPOINT_DIRECTORY: &str = "checkpoint";

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
    fn query(&self) -> Option<(u32, HistoryView)> {
        match self {
            Self::QueryCurrent { relation } => Some((*relation, HistoryView::Current)),
            Self::Query { relation, view } => Some((*relation, *view)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionScope {
    pub scope: Vec<u8>,
    pub entity: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPolicy {
    StrictFullHistory,
    /// Only completed oldest turns leave the model's working window.
    /// Their original log and exact memory records remain intact.
    WholeCompletedTurns,
}

/// Explicit caller admission, distinct from model context and store's
/// per-address payload capacity. No limit is silently increased on resume.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionLimits {
    pub max_new_tokens: usize,
    pub max_turns: usize,
    pub max_source_bytes: usize,
    pub max_history_tokens: usize,
    pub max_store_records: usize,
    pub context_policy: ContextPolicy,
}

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

/// Diagnostic interventions, not alternate compiler labels. Default turns
/// use both influences. A disabled read may still be inspected in the trace,
/// but its value never enters the emitter's input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnControls {
    pub read: bool,
    pub write: bool,
}

impl Default for TurnControls {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecallDisposition {
    NotRequested,
    Value,
    /// No store recall applied; the session's log recall supplied the line
    /// ([`GroundedSession::with_log_recall`]).
    LogValue,
    Absent,
    Disabled,
    Unsupported {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TurnStop {
    Eos,
    ShortCycle { period: usize },
    MaxNewTokens,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnOutcome {
    pub source: String,
    pub action: CompiledAction,
    pub controls: TurnControls,
    pub memory: MemoryEffect,
    pub recall: RecallDisposition,
    pub memory_commit: u64,
    /// Includes the exact prior generated IDs, not their re-encoded text.
    pub emitter_input_ids: Vec<u32>,
    /// Zero-based full-log index of the first turn in this forward window;
    /// equal to this turn's index when all completed turns were dropped.
    pub retained_from_turn: usize,
    pub reply_ids: Vec<u32>,
    pub reply_text: String,
    pub stop: TurnStop,
    /// A capped/cycled reply is explicitly closed for the next user turn.
    pub caller_eos_inserted: bool,
}

#[derive(Debug)]
pub enum GroundedSessionError {
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
convert!(StackCheckpointError, Checkpoint);
convert!(StackStoreError, Store);
convert!(DialogueError, Protocol);
convert!(TrainingError, Training);
convert!(std::io::Error, Io);
convert!(serde_json::Error, Json);

/// An optional log recall: from the user turns before this one (oldest
/// first) and this turn, the value an exact log of those turns gives, or
/// `None`. It must be deterministic, since earlier turns' lines are rebuilt.
pub type LogRecall = std::sync::Arc<dyn Fn(&[&str], &str) -> Option<String> + Send + Sync>;

/// Model and store are private so a caller cannot change the computation or
/// commit memory outside the transactional turn boundary.
pub struct GroundedSession<C: TurnCompiler> {
    model: StackModel,
    identity: CheckpointIdentity,
    store: StackStore,
    tokenizer: ByteBpeTokenizer,
    tokenizer_json: Vec<u8>,
    compiler: C,
    compiler_identity: CompilerIdentity,
    scope: SessionScope,
    limits: SessionLimits,
    initial_commit: u64,
    history_ids: Vec<u32>,
    turns: Vec<TurnOutcome>,
    log_recall: Option<(String, LogRecall)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionRecord {
    schema: String,
    checkpoint_manifest_sha256: String,
    compiler: CompilerIdentity,
    scope: SessionScope,
    limits: SessionLimits,
    initial_commit: u64,
    history_ids: Vec<u32>,
    turns: Vec<TurnOutcome>,
    /// The name of the log recall the session used, if any: a load must
    /// supply the same provider, since its lines are part of the history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    log_recall: Option<String>,
}

impl<C: TurnCompiler> GroundedSession<C> {
    /// Load the emitter/store internally, rather than trusting a caller-built
    /// public `StackCheckpoint` aggregate. The compiler adapter independently
    /// loads its declared model, if any. An empty store, if wanted, must have
    /// an explicitly chosen lineage in the saved checkpoint.
    pub fn from_checkpoint_path(
        root: &Path,
        tokenizer_json: Vec<u8>,
        compiler: C,
        scope: SessionScope,
        limits: SessionLimits,
        device: &Device,
    ) -> Result<Self, GroundedSessionError> {
        Self::from_loaded(
            load_checkpoint(root, device)?,
            tokenizer_json,
            compiler,
            scope,
            limits,
        )
    }

    fn from_loaded(
        checkpoint: StackCheckpoint,
        tokenizer_json: Vec<u8>,
        compiler: C,
        scope: SessionScope,
        limits: SessionLimits,
    ) -> Result<Self, GroundedSessionError> {
        checkpoint.identity().check_tokenizer(&tokenizer_json)?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
            .ok_or_else(|| GroundedSessionError::Binding("unreadable tokenizer".into()))?;
        if tokenizer.vocab_size() != checkpoint.model.config.vocab_size {
            return Err(GroundedSessionError::Binding(
                "tokenizer and model vocabulary extents differ".into(),
            ));
        }
        // Added-token IDs can leave holes even when the maximum ID matches
        // the model. The tokenizer silently decodes those holes to no bytes.
        for slot in 0..tokenizer.vocab_size() {
            let id = u32::try_from(slot)
                .map_err(|_| GroundedSessionError::Binding("tokenizer IDs exceed u32".into()))?;
            if tokenizer.decode_bytes(&[id]).is_empty() {
                return Err(GroundedSessionError::Binding(
                    "tokenizer contains an undecodable vocabulary slot".into(),
                ));
            }
        }
        if checkpoint.model.served_codec().is_some() {
            return Err(GroundedSessionError::Binding(
                "served representation is not supported".into(),
            ));
        }
        let compiler_identity = compiler.identity().clone();
        validate_compiler(
            &compiler_identity,
            compiler.artifact_bytes(),
            &checkpoint.record.identity,
        )?;
        validate_scope(&scope, &checkpoint.model, &checkpoint.record.identity)?;
        if limits.max_new_tokens == 0
            || limits.max_new_tokens >= checkpoint.model.config.context
            || limits.max_turns == 0
            || limits.max_source_bytes == 0
            || limits.max_history_tokens == 0
            || limits.max_store_records == 0
        {
            return Err(GroundedSessionError::Binding(
                "reply cap must be positive and smaller than context".into(),
            ));
        }
        let store = checkpoint.memory.ok_or_else(|| {
            GroundedSessionError::Binding("checkpoint has no declared session store".into())
        })?;
        store.check_vocabulary(checkpoint.model.config.vocab_size)?;
        check_limit("store records", store.records(), limits.max_store_records)?;
        let initial_commit = store.commit();
        let history_ids = vec![checkpoint.record.identity.protocol.bos_id];
        Ok(Self {
            model: checkpoint.model,
            identity: checkpoint.record.identity,
            store,
            tokenizer,
            tokenizer_json,
            compiler,
            compiler_identity,
            scope,
            limits,
            initial_commit,
            history_ids,
            turns: Vec::new(),
            log_recall: None,
        })
    }

    /// Supply a recall line from the user turn log for turns that make no
    /// memory action (the compiler's unresolved turns), when reading is
    /// enabled. Off by default. `name` identifies the provider in a saved
    /// session, whose load must supply the same one
    /// ([`Self::load_with_log_recall`]).
    pub fn with_log_recall(mut self, name: &str, recall: LogRecall) -> Self {
        self.log_recall = Some((name.to_owned(), recall));
        self
    }

    pub fn compiler_identity(&self) -> &CompilerIdentity {
        &self.compiler_identity
    }
    pub fn store(&self) -> &StackStore {
        &self.store
    }
    pub fn scope(&self) -> &SessionScope {
        &self.scope
    }
    pub fn history_ids(&self) -> &[u32] {
        &self.history_ids
    }
    pub fn turns(&self) -> &[TurnOutcome] {
        &self.turns
    }
    pub fn limits(&self) -> &SessionLimits {
        &self.limits
    }

    /// Start an empty conversation at the caller's explicit scope/entity,
    /// retaining the complete exact store and every model/compiler/limit
    /// binding. Previous transcript IDs no longer enter the emitter. Store
    /// versions, conflicts, evictions and record limits remain unchanged.
    ///
    /// This is caller-controlled identity selection, not authentication or
    /// forgetting. Earlier saved snapshots are unchanged. New write sources
    /// count from turn one in this conversation; record IDs and commits remain
    /// global to the retained store. An error leaves the conversation intact.
    pub fn start_conversation(&mut self, scope: SessionScope) -> Result<(), GroundedSessionError> {
        self.check_compiler()?;
        validate_scope(&scope, &self.model, &self.identity)?;
        self.scope = scope;
        self.initial_commit = self.store.commit();
        self.history_ids = vec![self.identity.protocol.bos_id];
        self.turns.clear();
        Ok(())
    }

    /// Predict, stage the exact memory action, then emit through the model's
    /// actual pointer-aware scores. Any error leaves store/history unchanged.
    pub fn turn(&mut self, source: &str) -> Result<TurnOutcome, GroundedSessionError> {
        self.turn_with_controls(source, TurnControls::default())
    }

    pub fn turn_with_controls(
        &mut self,
        source: &str,
        controls: TurnControls,
    ) -> Result<TurnOutcome, GroundedSessionError> {
        self.turn_with(source, controls, |model, input, cap, eos| {
            Ok(greedy_reply(model, input, cap, eos)?)
        })
    }

    fn turn_with(
        &mut self,
        source: &str,
        controls: TurnControls,
        emit: impl FnOnce(&StackModel, &[u32], usize, u32) -> Result<Reply, GroundedSessionError>,
    ) -> Result<TurnOutcome, GroundedSessionError> {
        self.check_compiler()?;
        self.check_log_limits(&self.turns, source)?;
        self.validate_source(source)?;
        let action = self.compiler.compile(source)?;
        let mut staged = self.store.clone();
        let source_turn = u64::try_from(self.turns.len())
            .ok()
            .and_then(|v| v.checked_add(1))
            .ok_or_else(|| GroundedSessionError::Input("turn index overflow".into()))?;
        let effect =
            self.apply_action(&mut staged, source, &action, source_turn, controls.write)?;
        check_limit(
            "store records",
            staged.records(),
            self.limits.max_store_records,
        )?;
        let (input, recall, retained_from_turn) =
            self.emitter_input(&self.turns, source, &effect, controls.read)?;
        let (suffix, _) = self.emitter_suffix(
            &self.turns,
            source,
            &effect,
            !self.turns.is_empty(),
            controls.read,
        )?;
        let history_needed = self
            .history_ids
            .len()
            .checked_add(suffix.len())
            .and_then(|n| n.checked_add(self.limits.max_new_tokens))
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| GroundedSessionError::Input("history length overflow".into()))?;
        check_limit(
            "history tokens",
            history_needed,
            self.limits.max_history_tokens,
        )?;
        let generated = emit(
            &self.model,
            &input,
            self.limits.max_new_tokens,
            self.identity.protocol.eos_id,
        )?;
        self.validate_reply(&generated)?;
        let stop = match (generated.eos, generated.cycle) {
            (true, _) => TurnStop::Eos,
            (false, Some(period)) => TurnStop::ShortCycle { period },
            _ => TurnStop::MaxNewTokens,
        };
        let caller_eos_inserted = !generated.eos;
        let text_end = generated.ids.len() - usize::from(generated.eos);
        let decoded = self.tokenizer.decode(&generated.ids[..text_end]);
        let reply_text = self.identity.protocol.reply_text(&decoded).to_owned();
        let mut history = self.history_ids.clone();
        history.extend(suffix);
        history.extend(&generated.ids);
        if caller_eos_inserted {
            history.push(self.identity.protocol.eos_id);
        }
        let outcome = TurnOutcome {
            source: source.to_owned(),
            action,
            controls,
            memory: effect,
            recall,
            memory_commit: staged.commit(),
            emitter_input_ids: input,
            retained_from_turn,
            reply_ids: generated.ids,
            reply_text,
            stop,
            caller_eos_inserted,
        };
        self.store = staged;
        self.history_ids = history;
        self.turns.push(outcome.clone());
        Ok(outcome)
    }

    fn check_compiler(&self) -> Result<(), GroundedSessionError> {
        if self.compiler.identity() != &self.compiler_identity {
            return Err(GroundedSessionError::Binding(
                "compiler identity changed during session".into(),
            ));
        }
        validate_compiler(
            &self.compiler_identity,
            self.compiler.artifact_bytes(),
            &self.identity,
        )
    }

    fn validate_source(&self, source: &str) -> Result<(), GroundedSessionError> {
        if source.trim().is_empty() {
            return Err(GroundedSessionError::Input("empty user turn".into()));
        }
        validate_plain_ids(&self.tokenizer.encode(source), &self.model, &self.identity)
    }

    fn value_tokens(
        &self,
        source: &str,
        span: SourceSpan,
    ) -> Result<Vec<u32>, GroundedSessionError> {
        let value = source
            .get(span.start..span.end)
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                GroundedSessionError::Span("empty, out-of-range or non-UTF8-boundary span".into())
            })?;
        let tokens = self.tokenizer.encode(value);
        validate_plain_ids(&tokens, &self.model, &self.identity)?;
        if tokens.is_empty() || self.tokenizer.decode_bytes(&tokens) != value.as_bytes() {
            return Err(GroundedSessionError::Span(
                "source span does not tokenize losslessly".into(),
            ));
        }
        Ok(tokens)
    }

    fn apply_action(
        &self,
        store: &mut StackStore,
        source: &str,
        action: &CompiledAction,
        turn: u64,
        write: bool,
    ) -> Result<MemoryEffect, GroundedSessionError> {
        if let Some((relation, view)) = action.query() {
            self.check_relation(relation)?;
            return Ok(MemoryEffect::Read {
                read: store.read(&self.scope.scope, &self.scope.entity, relation, view)?,
            });
        }
        let (relation, span, update) = match action {
            CompiledAction::Assert { relation, span } => (*relation, *span, Update::Assert),
            CompiledAction::Correct { relation, span } => (*relation, *span, Update::Correct),
            CompiledAction::Unresolved { reason } => {
                if reason.trim().is_empty() {
                    return Err(GroundedSessionError::Compiler(
                        "unresolved prediction has no reason".into(),
                    ));
                }
                return Ok(MemoryEffect::Unresolved);
            }
            CompiledAction::QueryCurrent { .. } | CompiledAction::Query { .. } => {
                return Err(GroundedSessionError::Compiler(
                    "unhandled query action".into(),
                ));
            }
        };
        if !self
            .compiler_identity
            .relations
            .iter()
            .any(|label| label.id == relation)
        {
            return Err(GroundedSessionError::Compiler(
                "predicted relation is outside the saved label schema".into(),
            ));
        }
        let value_tokens = self.value_tokens(source, span)?;
        if !write {
            return Ok(MemoryEffect::WriteDisabled { value_tokens });
        }
        let written = store.write_from(
            &self.scope.scope,
            &self.scope.entity,
            relation,
            &value_tokens,
            update,
            turn,
        )?;
        Ok(MemoryEffect::Write {
            written,
            value_tokens,
        })
    }

    fn emitter_suffix(
        &self,
        prior: &[TurnOutcome],
        source: &str,
        effect: &MemoryEffect,
        has_history: bool,
        read: bool,
    ) -> Result<(Vec<u32>, RecallDisposition), GroundedSessionError> {
        // Preserve emit-1's trained recall-at-reply placement: user, then
        // System Memory, then Assistant. Only proven absence gets "none".
        let (recall, disposition) = match effect {
            MemoryEffect::Read { .. } if !read => (None, RecallDisposition::Disabled),
            MemoryEffect::Read {
                read: StoreRead::Found(value),
            } if value.conflict => (
                None,
                RecallDisposition::Unsupported {
                    reason: "the trained recall format has no conflict channel".into(),
                },
            ),
            MemoryEffect::Read {
                read: StoreRead::Found(value),
            } => {
                validate_plain_ids(&value.tokens, &self.model, &self.identity)?;
                let bytes = self.tokenizer.decode_bytes(&value.tokens);
                let text = String::from_utf8(bytes)
                    .map_err(|_| GroundedSessionError::Input("stored value is not UTF-8".into()))?;
                if text == "none" || text.contains('\r') {
                    (None, RecallDisposition::Unsupported { reason: "stored value collides with the absence sentinel or normalizes in recall".into() })
                } else {
                    (Some(format!("Memory: {text}.")), RecallDisposition::Value)
                }
            }
            MemoryEffect::Read {
                read: StoreRead::Absent,
            } => (Some("Memory: none.".to_owned()), RecallDisposition::Absent),
            // Unresolved, NoHistory and Evicted are not evidence of absence.
            MemoryEffect::Read { .. } => (
                None,
                RecallDisposition::Unsupported {
                    reason: "the trained recall format has no history/eviction channel".into(),
                },
            ),
            MemoryEffect::Unresolved if read => match &self.log_recall {
                Some((_, recall)) => {
                    let log: Vec<&str> = prior.iter().map(|turn| turn.source.as_str()).collect();
                    match recall(&log, source) {
                        Some(value) if value != "none" && !value.contains('\r') => (
                            Some(format!("Memory: {value}.")),
                            RecallDisposition::LogValue,
                        ),
                        _ => (None, RecallDisposition::NotRequested),
                    }
                }
                None => (None, RecallDisposition::NotRequested),
            },
            _ => (None, RecallDisposition::NotRequested),
        };
        let mut messages = vec![Message {
            role: "user",
            content: source,
        }];
        if let Some(line) = &recall {
            messages.push(Message {
                role: "system",
                content: line,
            });
        }
        let encoder = self.identity.protocol.bind(&self.tokenizer)?;
        let suffix = encoder.encode_assistant_suffix(&messages, has_history);
        if suffix.emitted_turns != messages.len() || suffix.special_token_occurrences != 0 {
            return Err(GroundedSessionError::Input(
                "turn did not encode as plain user/memory messages".into(),
            ));
        }
        validate_plain_ids(&suffix.tokens, &self.model, &self.identity)?;
        Ok((suffix.tokens, disposition))
    }

    fn closed_history(
        &self,
        turns: &[TurnOutcome],
        from: usize,
    ) -> Result<Vec<u32>, GroundedSessionError> {
        let mut history = vec![self.identity.protocol.bos_id];
        for (offset, turn) in turns[from..].iter().enumerate() {
            let (suffix, _) = self.emitter_suffix(
                &turns[..from + offset],
                &turn.source,
                &turn.memory,
                offset != 0,
                turn.controls.read,
            )?;
            history.extend(suffix);
            history.extend(&turn.reply_ids);
            if turn.caller_eos_inserted {
                history.push(self.identity.protocol.eos_id);
            }
        }
        Ok(history)
    }

    fn emitter_input(
        &self,
        turns: &[TurnOutcome],
        source: &str,
        effect: &MemoryEffect,
        read: bool,
    ) -> Result<(Vec<u32>, RecallDisposition, usize), GroundedSessionError> {
        // This explicitly resets recomputed recurrent state and removes
        // pointer sources outside the selected window. It is an inference
        // policy, not a full-history parity or training-policy claim.
        let candidate = |from| -> Result<_, GroundedSessionError> {
            let mut input = self.closed_history(turns, from)?;
            let (suffix, disposition) =
                self.emitter_suffix(turns, source, effect, from < turns.len(), read)?;
            let needed = input
                .len()
                .checked_add(suffix.len())
                .and_then(|n| n.checked_add(self.limits.max_new_tokens))
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| GroundedSessionError::Input("context length overflow".into()))?;
            input.extend(suffix);
            Ok((input, disposition, needed))
        };
        let initial = match self.limits.context_policy {
            ContextPolicy::StrictFullHistory => 0,
            ContextPolicy::WholeCompletedTurns => turns.len(),
        };
        let (input, disposition, needed) = candidate(initial)?;
        if needed > self.model.config.context {
            return Err(GroundedSessionError::Context {
                needed,
                available: self.model.config.context,
            });
        }
        let mut best = (input, disposition, initial);
        if self.limits.context_policy == ContextPolicy::WholeCompletedTurns {
            // Extend backward only until the first overflow. Work depends
            // on the fitting window, not every turn in the retained log.
            for from in (0..turns.len()).rev() {
                let (input, disposition, needed) = candidate(from)?;
                if needed > self.model.config.context {
                    break;
                }
                best = (input, disposition, from);
            }
        }
        Ok(best)
    }

    fn check_log_limits(
        &self,
        turns: &[TurnOutcome],
        source: &str,
    ) -> Result<(), GroundedSessionError> {
        let count = turns
            .len()
            .checked_add(1)
            .ok_or_else(|| GroundedSessionError::Input("turn count overflow".into()))?;
        check_limit("turns", count, self.limits.max_turns)?;
        let bytes = turns
            .iter()
            .try_fold(source.len(), |sum, turn| sum.checked_add(turn.source.len()))
            .ok_or_else(|| GroundedSessionError::Input("source length overflow".into()))?;
        check_limit("source bytes", bytes, self.limits.max_source_bytes)
    }

    fn validate_reply(&self, reply: &Reply) -> Result<(), GroundedSessionError> {
        let eos = self.identity.protocol.eos_id;
        let stop_matches = match Reply::stop(&reply.ids, eos) {
            Some(detected) => reply.eos == detected.eos && reply.cycle == detected.cycle,
            None => {
                !reply.eos && reply.cycle.is_none() && reply.ids.len() == self.limits.max_new_tokens
            }
        };
        if reply.ids.is_empty()
            || reply.ids.len() > self.limits.max_new_tokens
            || !stop_matches
            || reply.eos != (reply.ids.last() == Some(&eos))
            || (reply.eos && reply.cycle.is_some())
            || reply.cycle == Some(0)
        {
            return Err(GroundedSessionError::Generation(
                "inconsistent reply/stop metadata".into(),
            ));
        }
        let end = reply.ids.len() - usize::from(reply.eos);
        validate_plain_ids(&reply.ids[..end], &self.model, &self.identity)
            .map_err(|e| GroundedSessionError::Generation(e.to_string()))
    }

    /// Save into a new, exclusively claimed envelope. The model/store
    /// checkpoint is sealed independently; nothing is appended beneath it.
    pub fn save(&self, root: &Path) -> Result<(), GroundedSessionError> {
        self.check_compiler()?;
        report_output::claim(root)?;
        let result: Result<(), GroundedSessionError> = (|| {
            let checkpoint = root.join(CHECKPOINT_DIRECTORY);
            save_checkpoint(&checkpoint, &self.model, &self.identity, Some(&self.store))?;
            write_new(&root.join(TOKENIZER_FILE), &self.tokenizer_json)?;
            write_new(&root.join(COMPILER_FILE), self.compiler.artifact_bytes())?;
            let record = SessionRecord {
                schema: if self
                    .turns
                    .iter()
                    .any(|turn| matches!(turn.action, CompiledAction::Query { .. }))
                {
                    TEMPORAL_SESSION_SCHEMA
                } else {
                    SESSION_SCHEMA
                }
                .into(),
                checkpoint_manifest_sha256: sealed_manifest_sha256(&checkpoint)?,
                compiler: self.compiler_identity.clone(),
                log_recall: self.log_recall.as_ref().map(|(name, _)| name.clone()),
                scope: self.scope.clone(),
                limits: self.limits.clone(),
                initial_commit: self.initial_commit,
                history_ids: self.history_ids.clone(),
                turns: self.turns.clone(),
            };
            write_new(
                &root.join(SESSION_FILE),
                &serde_json::to_vec_pretty(&record)?,
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                report_output::seal(root)?;
                report_output::verify(root)?;
                Ok(())
            }
            Err(error) => {
                let _ = write_new(&root.join("error.json"), error.to_string().as_bytes());
                let _ = report_output::seal(root);
                Err(error)
            }
        }
    }

    /// The caller loads its actual adapter from `compiler.bin`. Its identity
    /// and complete artifact bytes must agree with the saved session.
    pub fn load(root: &Path, compiler: C, device: &Device) -> Result<Self, GroundedSessionError> {
        Self::load_with_log_recall(root, compiler, device, None)
    }

    /// [`Self::load`] for a session saved with a log recall: the provider
    /// must carry the recorded name, and is attached before the saved turns
    /// are replayed. A session saved without one refuses a provider.
    pub fn load_with_log_recall(
        root: &Path,
        compiler: C,
        device: &Device,
        log_recall: Option<(String, LogRecall)>,
    ) -> Result<Self, GroundedSessionError> {
        report_output::verify(root)?;
        let expected = BTreeSet::from([
            report_output::ATTEMPT_FILE,
            report_output::MANIFEST_FILE,
            SESSION_FILE,
            COMPILER_FILE,
            TOKENIZER_FILE,
            CHECKPOINT_DIRECTORY,
        ]);
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| GroundedSessionError::Snapshot("non-UTF8 filename".into()))?;
            let kind = entry.file_type()?;
            if (name == CHECKPOINT_DIRECTORY && !kind.is_dir())
                || (name != CHECKPOINT_DIRECTORY && !kind.is_file())
            {
                return Err(GroundedSessionError::Snapshot(
                    "unexpected file kind".into(),
                ));
            }
            names.insert(name);
        }
        if names.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected {
            return Err(GroundedSessionError::Snapshot(
                "unexpected envelope file set".into(),
            ));
        }
        let record: SessionRecord = serde_json::from_slice(&fs::read(root.join(SESSION_FILE))?)?;
        if record.schema != SESSION_SCHEMA && record.schema != TEMPORAL_SESSION_SCHEMA {
            return Err(GroundedSessionError::Snapshot(
                "unsupported session schema".into(),
            ));
        }
        if record.schema == SESSION_SCHEMA
            && record
                .turns
                .iter()
                .any(|turn| matches!(turn.action, CompiledAction::Query { .. }))
        {
            return Err(GroundedSessionError::Snapshot(
                "typed query requires temporal session schema".into(),
            ));
        }
        let checkpoint = root.join(CHECKPOINT_DIRECTORY);
        if sealed_manifest_sha256(&checkpoint)? != record.checkpoint_manifest_sha256 {
            return Err(GroundedSessionError::Snapshot(
                "checkpoint manifest identity differs".into(),
            ));
        }
        let artifact = fs::read(root.join(COMPILER_FILE))?;
        if compiler.identity() != &record.compiler || compiler.artifact_bytes() != artifact {
            return Err(GroundedSessionError::Binding(
                "loaded compiler differs from snapshot".into(),
            ));
        }
        let mut session = Self::from_loaded(
            load_checkpoint(&checkpoint, device)?,
            fs::read(root.join(TOKENIZER_FILE))?,
            compiler,
            record.scope,
            record.limits,
        )?;
        match (&record.log_recall, &log_recall) {
            (None, None) => {}
            (Some(saved), Some((given, _))) if saved == given => {}
            (Some(saved), _) => {
                return Err(GroundedSessionError::Binding(format!(
                    "the session used log recall {saved}; load it with that provider"
                )))
            }
            (None, Some(_)) => {
                return Err(GroundedSessionError::Binding(
                    "the session used no log recall".into(),
                ))
            }
        }
        session.log_recall = log_recall;
        session.validate_history(record.initial_commit, &record.history_ids, &record.turns)?;
        session.initial_commit = record.initial_commit;
        session.history_ids = record.history_ids;
        session.turns = record.turns;
        Ok(session)
    }

    fn validate_history(
        &self,
        initial_commit: u64,
        history_ids: &[u32],
        turns: &[TurnOutcome],
    ) -> Result<(), GroundedSessionError> {
        let mut history = vec![self.identity.protocol.bos_id];
        let mut commit = initial_commit;
        // Bind the transcript to the loaded store, including old records whose
        // payloads were later evicted. Their exact value bytes remain retained.
        let memory = self.store.memory();
        let records: BTreeMap<_, _> = memory.records.iter().map(|r| (r.id, r)).collect();
        let mut positions = BTreeMap::new();
        for (key, chain) in &memory.chains {
            for (position, id) in chain.iter().enumerate() {
                positions.insert(*id, (key.as_slice(), position));
            }
        }
        for (index, turn) in turns.iter().enumerate() {
            self.check_log_limits(&turns[..index], &turn.source)?;
            self.validate_source(&turn.source)?;
            let (input, recall, retained_from_turn) = self.emitter_input(
                &turns[..index],
                &turn.source,
                &turn.memory,
                turn.controls.read,
            )?;
            if input != turn.emitter_input_ids
                || recall != turn.recall
                || retained_from_turn != turn.retained_from_turn
            {
                return Err(GroundedSessionError::Snapshot(
                    "stored turn does not extend the exact prior history".into(),
                ));
            }
            let eos = matches!(turn.stop, TurnStop::Eos);
            let cycle = match turn.stop {
                TurnStop::ShortCycle { period } => Some(period),
                _ => None,
            };
            let reply = Reply {
                ids: turn.reply_ids.clone(),
                eos,
                cycle,
            };
            self.validate_reply(&reply)?;
            let text_end = reply.ids.len() - usize::from(eos);
            let decoded = self.tokenizer.decode(&reply.ids[..text_end]);
            if turn.caller_eos_inserted == eos
                || turn.reply_text != self.identity.protocol.reply_text(&decoded)
            {
                return Err(GroundedSessionError::Snapshot(
                    "reply text/closure disagrees with generated IDs".into(),
                ));
            }
            match (&turn.action, &turn.memory) {
                (
                    CompiledAction::Assert { relation, span },
                    MemoryEffect::Write {
                        written,
                        value_tokens,
                    },
                )
                | (
                    CompiledAction::Correct { relation, span },
                    MemoryEffect::Write {
                        written,
                        value_tokens,
                    },
                ) => {
                    self.check_relation(*relation)?;
                    let update = if matches!(turn.action, CompiledAction::Assert { .. }) {
                        Update::Assert
                    } else {
                        Update::Correct
                    };
                    commit = commit
                        .checked_add(1)
                        .ok_or_else(|| GroundedSessionError::Snapshot("commit overflow".into()))?;
                    if !turn.controls.write
                        || written.commit != commit
                        || written.action != update
                        || &self.value_tokens(&turn.source, *span)? != value_tokens
                    {
                        return Err(GroundedSessionError::Snapshot(
                            "write trace differs from source/action".into(),
                        ));
                    }
                    let key = StackStore::key(&self.scope.scope, &self.scope.entity, *relation)?;
                    let value: Vec<u8> =
                        value_tokens.iter().flat_map(|t| t.to_le_bytes()).collect();
                    let record = records.get(&written.id).ok_or_else(|| {
                        GroundedSessionError::Snapshot("write record is absent from store".into())
                    })?;
                    let position = positions.get(&written.id);
                    if position.map(|(stored_key, _)| *stored_key) != Some(key.as_slice())
                        || position.map(|(_, n)| *n as u64 + 1) != Some(written.revision)
                        || record.commit != commit
                        || record.source != index as u64 + 1
                        || record.action != written.action
                        || record.conflict != written.conflict
                        || record.predecessor != written.superseded
                        || record.value != value
                    {
                        return Err(GroundedSessionError::Snapshot(
                            "write receipt differs from saved store record/address".into(),
                        ));
                    }
                }
                (
                    CompiledAction::Assert { relation, span }
                    | CompiledAction::Correct { relation, span },
                    MemoryEffect::WriteDisabled { value_tokens },
                ) => {
                    self.check_relation(*relation)?;
                    if turn.controls.write
                        || &self.value_tokens(&turn.source, *span)? != value_tokens
                    {
                        return Err(GroundedSessionError::Snapshot(
                            "disabled write differs from source/controls".into(),
                        ));
                    }
                }
                (action, MemoryEffect::Read { read }) if action.query().is_some() => {
                    let (relation, view) = action.query().ok_or_else(|| {
                        GroundedSessionError::Snapshot("query action has no view".into())
                    })?;
                    self.check_relation(relation)?;
                    let key = StackStore::key(&self.scope.scope, &self.scope.entity, relation)?;
                    let expected = read_receipt_at(
                        memory.chain(&key),
                        &records,
                        memory.capacity,
                        commit,
                        view,
                    )?;
                    if read != &expected {
                        return Err(GroundedSessionError::Snapshot(
                            "query receipt differs from store at pinned turn commit".into(),
                        ));
                    }
                }
                (CompiledAction::Unresolved { reason }, MemoryEffect::Unresolved)
                    if !reason.trim().is_empty() => {}
                _ => {
                    return Err(GroundedSessionError::Snapshot(
                        "action and memory effect disagree".into(),
                    ))
                }
            }
            if turn.memory_commit != commit {
                return Err(GroundedSessionError::Snapshot(
                    "turn commit differs from action trace".into(),
                ));
            }
            let (suffix, _) = self.emitter_suffix(
                &turns[..index],
                &turn.source,
                &turn.memory,
                index != 0,
                turn.controls.read,
            )?;
            history.extend(suffix);
            history.extend(&reply.ids);
            if turn.caller_eos_inserted {
                history.push(self.identity.protocol.eos_id);
            }
        }
        check_limit(
            "history tokens",
            history.len(),
            self.limits.max_history_tokens,
        )?;
        if history != history_ids || commit != self.store.commit() {
            return Err(GroundedSessionError::Snapshot(
                "final history/store commit differs".into(),
            ));
        }
        Ok(())
    }

    fn check_relation(&self, relation: u32) -> Result<(), GroundedSessionError> {
        if self
            .compiler_identity
            .relations
            .iter()
            .any(|label| label.id == relation)
        {
            Ok(())
        } else {
            Err(GroundedSessionError::Compiler(
                "relation outside saved label schema".into(),
            ))
        }
    }
}

/// Reconstruct a receipt, not a live read, from a validated immutable chain.
/// Current `read_at` intentionally uses today's eviction flags. A saved turn
/// instead records residency at its original commit, including preloaded
/// records and the fixed per-chain capacity. Never use tombstone bytes to
/// cross a predecessor that was already evicted when the turn ran.
fn read_receipt_at(
    chain: &[u64],
    records: &BTreeMap<u64, &Record>,
    capacity: usize,
    commit: u64,
    view: HistoryView,
) -> Result<StoreRead, GroundedSessionError> {
    let visible = chain.partition_point(|id| {
        records
            .get(id)
            .is_some_and(|record| record.commit <= commit)
    });
    if visible == 0 {
        return Ok(StoreRead::Absent);
    }
    let record_at = |position: usize| {
        chain
            .get(position)
            .and_then(|id| records.get(id))
            .copied()
            .ok_or_else(|| {
                GroundedSessionError::Snapshot("query chain references a missing record".into())
            })
    };
    let resident_start = visible.saturating_sub(capacity);
    let selected = match view {
        HistoryView::Current => Some(visible - 1),
        HistoryView::PreviousAssertion => visible.checked_sub(2),
        HistoryView::Initial => Some(0),
        HistoryView::PreviousDistinctValue => {
            let head = record_at(visible - 1)?;
            let mut found = None;
            for position in (0..visible - 1).rev() {
                if position < resident_start {
                    return Ok(StoreRead::Evicted);
                }
                if record_at(position)?.value != head.value {
                    found = Some(position);
                    break;
                }
            }
            found
        }
    };
    let Some(position) = selected else {
        return Ok(StoreRead::NoHistory);
    };
    if position < resident_start {
        return Ok(StoreRead::Evicted);
    }
    let record = record_at(position)?;
    if record.value.len() % 4 != 0 {
        return Err(GroundedSessionError::Snapshot(
            "record value is not whole tokens".into(),
        ));
    }
    let tokens = record
        .value
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    Ok(StoreRead::Found(StoreValue {
        tokens,
        record: record.id,
        commit: record.commit,
        update: record.action,
        conflict: record.conflict,
    }))
}

fn validate_compiler(
    compiler: &CompilerIdentity,
    bytes: &[u8],
    identity: &CheckpointIdentity,
) -> Result<(), GroundedSessionError> {
    if compiler.schema.trim().is_empty()
        || compiler.label_schema.trim().is_empty()
        || compiler.relations.is_empty()
        || bytes.is_empty()
        || compiler.artifact_sha256 != sha256_hex(bytes)
        || compiler.tokenizer_sha256 != identity.tokenizer_sha256
    {
        return Err(GroundedSessionError::Binding(
            "compiler schema/artifact/tokenizer binding is invalid".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for label in &compiler.relations {
        if label.name.trim().is_empty() || !ids.insert(label.id) || !names.insert(&label.name) {
            return Err(GroundedSessionError::Binding(
                "relation labels must have unique IDs and names".into(),
            ));
        }
    }
    if let Some(encoder) = &compiler.encoder {
        if [&encoder.config_sha256, &encoder.model_sha256]
            .into_iter()
            .chain(encoder.transport_sha256.iter())
            .any(|digest| !is_sha256(digest))
        {
            return Err(GroundedSessionError::Binding(
                "malformed compiler encoder identity".into(),
            ));
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn validate_scope(
    scope: &SessionScope,
    model: &StackModel,
    identity: &CheckpointIdentity,
) -> Result<(), GroundedSessionError> {
    if scope.scope.is_empty() || scope.entity.is_empty() {
        return Err(GroundedSessionError::Binding(
            "scope and entity must be explicit and nonempty".into(),
        ));
    }
    validate_plain_ids(&scope.entity, model, identity)
}

fn validate_plain_ids(
    ids: &[u32],
    model: &StackModel,
    identity: &CheckpointIdentity,
) -> Result<(), GroundedSessionError> {
    let protocol = &identity.protocol;
    if ids.iter().any(|id| {
        *id as usize >= model.config.vocab_size
            || [protocol.bos_id, protocol.eos_id, protocol.unk_id].contains(id)
    }) {
        return Err(GroundedSessionError::Input(
            "special or out-of-vocabulary token in plain content".into(),
        ));
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    fs::File::create_new(path)?.write_all(bytes)
}

fn check_limit(
    resource: &'static str,
    needed: usize,
    limit: usize,
) -> Result<(), GroundedSessionError> {
    if needed > limit {
        Err(GroundedSessionError::StorageLimit {
            resource,
            needed,
            limit,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{PointerConfig, ReadScore, StackArch, StackConfig, TransportSnap};
    use crate::stack_checkpoint::DataIdentity;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// Injection is deliberately confined to this module. These fixtures
    /// test interfaces and restart, not learned compiler capability.
    struct TestCompiler {
        identity: CompilerIdentity,
        bytes: Vec<u8>,
        actions: BTreeMap<String, CompiledAction>,
    }

    impl TestCompiler {
        fn load(bytes: Vec<u8>, tokenizer: &[u8]) -> Self {
            let actions = serde_json::from_slice(&bytes).expect("test actions");
            Self {
                identity: CompilerIdentity {
                    schema: "TEST_ONLY/injected-actions/1".into(),
                    artifact_sha256: sha256_hex(&bytes),
                    tokenizer_sha256: sha256_hex(tokenizer),
                    label_schema: "TEST_ONLY/two-relations/1".into(),
                    relations: vec![
                        RelationLabel {
                            id: 1,
                            name: "value".into(),
                        },
                        RelationLabel {
                            id: 2,
                            name: "missing".into(),
                        },
                    ],
                    encoder: None,
                },
                bytes,
                actions,
            }
        }

        fn new(tokenizer: &[u8]) -> Self {
            let mut actions = BTreeMap::new();
            for value in ["blue", "red", "green", "ice cream", "é", "none", "a\rb"] {
                let source = format!("put {value}");
                actions.insert(
                    source.clone(),
                    CompiledAction::Assert {
                        relation: 1,
                        span: SourceSpan {
                            start: 4,
                            end: source.len(),
                        },
                    },
                );
            }
            actions.insert(
                "fix green".into(),
                CompiledAction::Correct {
                    relation: 1,
                    span: SourceSpan { start: 4, end: 9 },
                },
            );
            actions.insert("ask".into(), CompiledAction::QueryCurrent { relation: 1 });
            for (source, view) in [
                ("ask current", HistoryView::Current),
                ("ask previous", HistoryView::PreviousAssertion),
                ("ask distinct", HistoryView::PreviousDistinctValue),
                ("ask initial", HistoryView::Initial),
            ] {
                actions.insert(source.into(), CompiledAction::Query { relation: 1, view });
            }
            actions.insert(
                "absent".into(),
                CompiledAction::QueryCurrent { relation: 2 },
            );
            actions.insert(
                "bad span".into(),
                CompiledAction::Assert {
                    relation: 1,
                    span: SourceSpan {
                        start: 0,
                        end: usize::MAX,
                    },
                },
            );
            actions.insert(
                "bad relation".into(),
                CompiledAction::QueryCurrent { relation: 99 },
            );
            Self::load(serde_json::to_vec(&actions).expect("actions"), tokenizer)
        }
    }

    impl TurnCompiler for TestCompiler {
        fn identity(&self) -> &CompilerIdentity {
            &self.identity
        }
        fn artifact_bytes(&self) -> &[u8] {
            &self.bytes
        }
        fn compile(&self, source: &str) -> Result<CompiledAction, GroundedSessionError> {
            Ok(self
                .actions
                .get(source)
                .cloned()
                .unwrap_or(CompiledAction::Unresolved {
                    reason: "test input has no injected action".into(),
                }))
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "uor-grounded-session-{}-{nonce}-{name}",
            std::process::id()
        ))
    }

    fn tokenizer_bytes() -> Vec<u8> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, token) in specials.iter().enumerate() {
            vocab.insert((*token).into(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let code = if printable.contains(&byte) {
                byte
            } else {
                extra += 1;
                255 + extra
            };
            vocab.insert(
                char::from_u32(code).expect("alphabet").to_string(),
                json!(byte + 3),
            );
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, token)| json!({"id":id,"content":token}))
            .collect();
        serde_json::to_vec(&json!({
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "added_tokens":added,
            "model":{"type":"BPE","vocab":vocab,"merges":[]}
        }))
        .expect("tokenizer")
    }

    fn limits(policy: ContextPolicy) -> SessionLimits {
        SessionLimits {
            max_new_tokens: 2,
            max_turns: 32,
            max_source_bytes: 4096,
            max_history_tokens: 4096,
            max_store_records: 16,
            context_policy: policy,
        }
    }

    fn fixture(
        name: &str,
        context: usize,
        policy: ContextPolicy,
    ) -> (PathBuf, GroundedSession<TestCompiler>) {
        fixture_with_capacity(name, context, policy, 8)
    }

    fn fixture_with_capacity(
        name: &str,
        context: usize,
        policy: ContextPolicy,
        capacity: usize,
    ) -> (PathBuf, GroundedSession<TestCompiler>) {
        fixture_with_protocol(name, context, policy, capacity, 1)
    }

    fn fixture_with_protocol(
        name: &str,
        context: usize,
        policy: ContextPolicy,
        capacity: usize,
        version: u8,
    ) -> (PathBuf, GroundedSession<TestCompiler>) {
        let base = scratch(name);
        fs::create_dir_all(&base).expect("base");
        let tokenizer = tokenizer_bytes();
        let training = base.join("test-training-report");
        report_output::claim(&training).expect("claim");
        write_new(&training.join("test-only.json"), b"{\"fixture\":true}").expect("report");
        report_output::seal(&training).expect("seal");
        let identity = CheckpointIdentity::from_tokenizer_version(
            &tokenizer,
            version,
            vec![DataIdentity {
                label: "TEST_ONLY".into(),
                bytes: 7,
                sha256: sha256_hex(b"fixture"),
            }],
            sealed_manifest_sha256(&training).expect("manifest"),
        )
        .expect("identity");
        let mut model = StackModel::new(
            StackConfig {
                arch: StackArch::Geometric,
                vocab_size: 259,
                width: 8,
                heads: 2,
                mlp_hidden: 16,
                context,
                pattern: "ra".into(),
                read: ReadScore::Lorentz,
                rotation: true,
                seed: 7,
                memory: None,
                select: None,
                pointer: Some(PointerConfig::new(4)),
            },
            &Device::Cpu,
        )
        .expect("model");
        model
            .set_transport_snap(Some(TransportSnap::Icosian))
            .expect("snap");
        let checkpoint = base.join("initial-checkpoint");
        save_checkpoint(
            &checkpoint,
            &model,
            &identity,
            Some(&StackStore::new(41, capacity).expect("store")),
        )
        .expect("save");
        let compiler = TestCompiler::new(&tokenizer);
        let session = GroundedSession::from_checkpoint_path(
            &checkpoint,
            tokenizer,
            compiler,
            SessionScope {
                scope: b"test-user".to_vec(),
                entity: vec![3],
            },
            limits(policy),
            &Device::Cpu,
        )
        .expect("session");
        (base, session)
    }

    fn fixed_turn(session: &mut GroundedSession<TestCompiler>, source: &str) -> TurnOutcome {
        fixed_controlled(session, source, TurnControls::default())
    }

    #[test]
    fn protocol_two_loaded_session_preserves_recall_segments_and_generated_ids() {
        let (base, mut session) =
            fixture_with_protocol("protocol-two", 256, ContextPolicy::StrictFullHistory, 8, 2);
        assert_eq!(
            session.identity.protocol.schema,
            uor_r4_tokenizer::dialogue::SCHEMA_V2
        );
        let generated = vec![u32::from(b' ') + 3, u32::from(b'x') + 3];
        let first = session
            .turn_with("put blue", TurnControls::default(), |_, _, _, _| {
                Ok(Reply {
                    ids: generated.clone(),
                    eos: false,
                    cycle: None,
                })
            })
            .expect("controlled leading-space reply");
        assert_eq!(first.reply_ids, generated);
        assert_eq!(first.reply_text, "x");
        let closed = session.history_ids.clone();
        assert!(closed.ends_with(&[generated[0], generated[1], session.identity.protocol.eos_id]));

        // Exercise the loaded model for the recall turn, retaining its actual
        // generated IDs; this is an interface witness, not a quality result.
        let query = session
            .turn("ask")
            .expect("actual protocol-two recall reply");
        assert_eq!(query.recall, RecallDisposition::Value);
        let mut expected = closed;
        for segment in [
            "\n",
            "User:",
            " ask",
            "\n",
            "System:",
            " Memory: blue.",
            "\n",
            "Assistant:",
        ] {
            expected.extend(session.tokenizer.encode(segment));
        }
        assert_eq!(query.emitter_input_ids, expected);
        assert_eq!(query.retained_from_turn, 0);
        assert_eq!(
            query
                .emitter_input_ids
                .iter()
                .filter(|&&id| id == session.identity.protocol.bos_id)
                .count(),
            1
        );

        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("save protocol two");
        let mut loaded = GroundedSession::load(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload protocol two");
        assert_eq!(loaded.identity, session.identity);
        assert_eq!(loaded.turns, session.turns);
        assert_eq!(loaded.history_ids, session.history_ids);
        assert_eq!(
            loaded.turn("ask").expect("loaded continuation"),
            session.turn("ask").expect("continuation")
        );

        let raw_text = base.join("unstripped-reply-text");
        resealed_snapshot(&snapshot, &raw_text, |record| {
            record["turns"][0]["reply_text"] = json!(" x");
        });
        assert!(matches!(
            GroundedSession::load(
                &raw_text,
                TestCompiler::new(&session.tokenizer_json),
                &Device::Cpu,
            ),
            Err(GroundedSessionError::Snapshot(_))
        ));
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn protocol_one_keeps_leading_reply_space_and_identity() {
        let (base, mut session) =
            fixture("protocol-one-text", 128, ContextPolicy::StrictFullHistory);
        let reply = session
            .turn_with("hello", TurnControls::default(), |_, _, _, _| {
                Ok(Reply {
                    ids: vec![u32::from(b' ') + 3, u32::from(b'x') + 3],
                    eos: false,
                    cycle: None,
                })
            })
            .expect("legacy reply");
        assert_eq!(reply.reply_text, " x");
        assert_eq!(
            session.identity.protocol.schema,
            uor_r4_tokenizer::dialogue::SCHEMA
        );
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("save legacy");
        let loaded = GroundedSession::load(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload legacy");
        assert_eq!(loaded.turns, session.turns);
        assert_eq!(loaded.history_ids, session.history_ids);
        fs::remove_dir_all(base).expect("clean");
    }

    fn fixed_controlled(
        session: &mut GroundedSession<TestCompiler>,
        source: &str,
        controls: TurnControls,
    ) -> TurnOutcome {
        session
            .turn_with(source, controls, |_, _, _, _| {
                Ok(Reply {
                    ids: vec![u32::from(b'o') + 3, u32::from(b'k') + 3],
                    eos: false,
                    cycle: None,
                })
            })
            .expect("fixture turn")
    }

    fn current(session: &GroundedSession<TestCompiler>) -> StoreRead {
        session
            .store
            .read(
                &session.scope.scope,
                &session.scope.entity,
                1,
                HistoryView::Current,
            )
            .expect("read")
    }

    fn copy_test_directory(source: &Path, target: &Path) {
        fs::create_dir(target).expect("test copy directory");
        for entry in fs::read_dir(source).expect("entries") {
            let entry = entry.expect("entry");
            let next = target.join(entry.file_name());
            if entry.file_type().expect("type").is_dir() {
                copy_test_directory(&entry.path(), &next);
            } else {
                fs::copy(entry.path(), next).expect("copy test file");
            }
        }
    }

    fn resealed_snapshot(source: &Path, target: &Path, edit: impl FnOnce(&mut Value)) {
        report_output::claim(target).expect("claim adversarial fixture");
        for file in [SESSION_FILE, TOKENIZER_FILE, COMPILER_FILE] {
            fs::copy(source.join(file), target.join(file)).expect("copy");
        }
        copy_test_directory(
            &source.join(CHECKPOINT_DIRECTORY),
            &target.join(CHECKPOINT_DIRECTORY),
        );
        let path = target.join(SESSION_FILE);
        let mut record: Value =
            serde_json::from_slice(&fs::read(&path).expect("record")).expect("json");
        edit(&mut record);
        fs::write(&path, serde_json::to_vec_pretty(&record).expect("json")).expect("edit fixture");
        report_output::seal(target).expect("seal adversarial fixture");
    }

    #[test]
    fn temporal_receipts_match_reads_at_each_original_commit() {
        let views = [
            HistoryView::Current,
            HistoryView::PreviousAssertion,
            HistoryView::PreviousDistinctValue,
            HistoryView::Initial,
        ];
        // Corrections count as versions; same-value assertions do not create
        // a new distinct value. Other-address writes separate global commits
        // from this address's chain positions, including preloaded history.
        let writes = [
            (21, Update::Assert),
            (21, Update::Correct),
            (22, Update::Correct),
            (22, Update::Assert),
            (21, Update::Assert),
        ];
        for capacity in [1, 2, 4] {
            let mut store = StackStore::new(41, capacity).expect("store");
            let mut captured = Vec::new();
            for prefix in 0..=writes.len() {
                if prefix > 0 {
                    store
                        .write(b"scope", &[11], 2, &[31], Update::Assert)
                        .expect("other address");
                    let (token, update) = writes[prefix - 1];
                    store
                        .write(b"scope", &[11], 1, &[token], update)
                        .expect("write");
                }
                for view in views {
                    let read = store.read(b"scope", &[11], 1, view).expect("actual read");
                    captured.push((store.commit(), view, read));
                }
            }
            // Subsequent writes evict values that earlier reads actually saw.
            for token in 23..27 {
                store
                    .write(b"scope", &[11], 1, &[token], Update::Correct)
                    .expect("later write");
            }
            let loaded =
                StackStore::from_bytes(&store.to_bytes().expect("bytes"), 41).expect("load");
            let key = StackStore::key(b"scope", &[11], 1).expect("key");
            let records = loaded
                .memory()
                .records
                .iter()
                .map(|record| (record.id, record))
                .collect();
            for (commit, view, actual) in captured {
                assert_eq!(
                    read_receipt_at(
                        loaded.memory().chain(&key),
                        &records,
                        capacity,
                        commit,
                        view
                    )
                    .expect("receipt"),
                    actual,
                    "capacity={capacity}, commit={commit}, view={view:?}"
                );
            }
            assert_eq!(
                loaded
                    .read_at(b"scope", &[11], 1, HistoryView::Initial, 2)
                    .expect("current residency"),
                StoreRead::Evicted
            );
        }
    }

    #[test]
    fn temporal_turns_reload_with_original_residency_and_distinct_barriers() {
        let (base, mut session) =
            fixture_with_capacity("temporal", 128, ContextPolicy::WholeCompletedTurns, 2);
        let empty = fixed_turn(&mut session, "ask initial");
        assert_eq!(
            empty.memory,
            MemoryEffect::Read {
                read: StoreRead::Absent
            }
        );
        assert_eq!(empty.recall, RecallDisposition::Absent);
        fixed_turn(&mut session, "put blue");
        let missing = fixed_turn(&mut session, "ask previous");
        assert_eq!(
            missing.memory,
            MemoryEffect::Read {
                read: StoreRead::NoHistory
            }
        );
        assert!(matches!(
            missing.recall,
            RecallDisposition::Unsupported { .. }
        ));
        let initial = fixed_turn(&mut session, "ask initial");
        assert!(
            matches!(initial.memory, MemoryEffect::Read { read: StoreRead::Found(ref value) } if value.record == 1)
        );
        fixed_turn(&mut session, "fix green");
        let previous = fixed_turn(&mut session, "ask previous");
        assert_eq!(previous.memory, initial.memory);
        fixed_turn(&mut session, "put green");
        let correction = fixed_turn(&mut session, "ask previous");
        assert!(
            matches!(correction.memory, MemoryEffect::Read { read: StoreRead::Found(ref value) }
            if value.record == 2 && value.update == Update::Correct)
        );
        for source in ["ask initial", "ask distinct"] {
            let evicted = fixed_turn(&mut session, source);
            assert_eq!(
                evicted.memory,
                MemoryEffect::Read {
                    read: StoreRead::Evicted
                }
            );
            assert!(matches!(
                evicted.recall,
                RecallDisposition::Unsupported { .. }
            ));
        }
        let disabled = fixed_controlled(
            &mut session,
            "ask previous",
            TurnControls {
                read: false,
                write: true,
            },
        );
        assert_eq!(disabled.memory, correction.memory);
        assert_eq!(disabled.recall, RecallDisposition::Disabled);
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("save temporal");
        let compiler = TestCompiler::new(&session.tokenizer_json);
        let mut loaded =
            GroundedSession::load(&snapshot, compiler, &Device::Cpu).expect("reload temporal");
        assert_eq!(loaded.turns, session.turns);
        assert_eq!(loaded.history_ids, session.history_ids);
        assert_eq!(
            fixed_turn(&mut loaded, "ask distinct"),
            fixed_turn(&mut session, "ask distinct")
        );

        // Equal-value evicted predecessors are still barriers. Their retained
        // identities must not be used to turn this into NoHistory on reload.
        fixed_turn(&mut session, "put green");
        assert_eq!(
            fixed_turn(&mut session, "ask distinct").memory,
            MemoryEffect::Read {
                read: StoreRead::Evicted
            }
        );
        let barrier_snapshot = base.join("equal-value-barrier");
        session.save(&barrier_snapshot).expect("save barrier");
        GroundedSession::load(
            &barrier_snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload barrier");

        // Disabling this turn's recall leaves its prompt unchanged when only
        // its receipt is tampered: the store check must reject the mismatch.
        let disabled_index = session
            .turns
            .iter()
            .position(|turn| !turn.controls.read)
            .expect("disabled turn");
        for (name, replacement) in [
            (
                "wrong-record",
                json!({"Found": {"tokens": session.tokenizer.encode("green"), "record": 1, "commit": 2, "update": "Correct", "conflict": false}}),
            ),
            ("false-absence", json!("Absent")),
        ] {
            let tampered = base.join(name);
            resealed_snapshot(&snapshot, &tampered, |record| {
                record["turns"][disabled_index]["memory"]["read"] = replacement;
            });
            assert!(matches!(
                GroundedSession::load(
                    &tampered,
                    TestCompiler::new(&session.tokenizer_json),
                    &Device::Cpu
                ),
                Err(GroundedSessionError::Snapshot(_))
            ));
        }
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn temporal_schema_is_additive_and_current_snapshots_keep_schema_one() {
        let (base, mut session) = fixture("schemas", 128, ContextPolicy::WholeCompletedTurns);
        assert_eq!(
            serde_json::to_value(CompiledAction::QueryCurrent { relation: 1 })
                .expect("legacy action"),
            json!({"kind":"query_current","relation":1})
        );
        fixed_turn(&mut session, "put blue");
        fixed_turn(&mut session, "ask");
        let legacy = base.join("legacy");
        session.save(&legacy).expect("save legacy");
        let legacy_record: Value =
            serde_json::from_slice(&fs::read(legacy.join(SESSION_FILE)).expect("record"))
                .expect("json");
        assert_eq!(legacy_record["schema"], SESSION_SCHEMA);
        let mut loaded = GroundedSession::load(
            &legacy,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("load legacy");
        assert_eq!(
            fixed_turn(&mut loaded, "ask"),
            fixed_turn(&mut session, "ask")
        );

        // Even Query{Current} uses the additive encoding and must not be
        // placed under schema1. SavedCompiler still emits QueryCurrent.
        fixed_turn(&mut session, "ask current");
        let temporal = base.join("typed-current");
        session.save(&temporal).expect("save typed current");
        let record: Value =
            serde_json::from_slice(&fs::read(temporal.join(SESSION_FILE)).expect("record"))
                .expect("json");
        assert_eq!(record["schema"], TEMPORAL_SESSION_SCHEMA);
        let mut loaded = GroundedSession::load(
            &temporal,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("load typed current");
        assert_eq!(
            fixed_turn(&mut loaded, "ask current"),
            fixed_turn(&mut session, "ask current")
        );
        let disguised = base.join("typed-query-under-schema1");
        resealed_snapshot(&temporal, &disguised, |record| {
            record["schema"] = json!(SESSION_SCHEMA)
        });
        assert!(matches!(
            GroundedSession::load(
                &disguised,
                TestCompiler::new(&session.tokenizer_json),
                &Device::Cpu
            ),
            Err(GroundedSessionError::Snapshot(_))
        ));
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn conversation_restart_reclaims_transcript_limits_but_preserves_exact_memory() {
        let (base, mut session) =
            fixture_with_capacity("restart-limits", 128, ContextPolicy::WholeCompletedTurns, 2);
        for source in ["put blue", "put red", "fix green", "put green", "put red"] {
            fixed_turn(&mut session, source);
        }
        let views = [
            HistoryView::Current,
            HistoryView::PreviousAssertion,
            HistoryView::PreviousDistinctValue,
            HistoryView::Initial,
        ];
        let before_reads: Vec<_> = views
            .iter()
            .map(|view| {
                session
                    .store
                    .read(&session.scope.scope, &session.scope.entity, 1, *view)
                    .expect("read")
            })
            .collect();
        assert!(matches!(&before_reads[0], StoreRead::Found(value) if value.conflict));
        assert!(matches!(&before_reads[1], StoreRead::Found(value) if value.record == 4));
        assert!(matches!(&before_reads[2], StoreRead::Found(value) if value.record == 4));
        assert_eq!(before_reads[3], StoreRead::Evicted);
        session.limits.max_turns = session.turns.len();
        session.limits.max_source_bytes = session.turns.iter().map(|turn| turn.source.len()).sum();
        session.limits.max_history_tokens = session.history_ids.len();
        session.limits.max_store_records = session.store.records();
        let limits = session.limits.clone();
        let store_bytes = session.store.to_bytes().expect("store");
        let compiler = session.compiler_identity.clone();
        let identity = session.identity.clone();
        assert!(matches!(
            session.turn("ask"),
            Err(GroundedSessionError::StorageLimit {
                resource: "turns",
                ..
            })
        ));
        session
            .start_conversation(session.scope.clone())
            .expect("restart");
        assert_eq!(session.store.to_bytes().expect("store"), store_bytes);
        assert_eq!(session.initial_commit, 5);
        assert_eq!(session.history_ids, [session.identity.protocol.bos_id]);
        assert!(session.turns.is_empty());
        assert_eq!(session.limits, limits);
        assert_eq!(session.compiler_identity, compiler);
        assert_eq!(session.identity, identity);
        for (view, expected) in views.into_iter().zip(before_reads) {
            assert_eq!(
                session
                    .store
                    .read(&session.scope.scope, &session.scope.entity, 1, view)
                    .expect("read"),
                expected
            );
        }
        // Restarting cannot evade the retained store's total-record admission.
        assert!(matches!(
            session.turn("put blue"),
            Err(GroundedSessionError::StorageLimit {
                resource: "store records",
                ..
            })
        ));
        assert_eq!(session.store.to_bytes().expect("store"), store_bytes);
        assert!(session.turns.is_empty());

        let snapshot = base.join("empty-conversation");
        session.save(&snapshot).expect("save restart");
        let mut loaded = GroundedSession::load(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload restart baseline");
        assert_eq!(loaded.initial_commit, 5);
        assert_eq!(loaded.history_ids, session.history_ids);
        let expected = session.turn("ask").expect("actual post-restart emission");
        let actual = loaded.turn("ask").expect("actual reloaded emission");
        assert_eq!(actual, expected);
        assert!(matches!(
            expected.recall,
            RecallDisposition::Unsupported { .. }
        ));
        let input = session.tokenizer.decode(&expected.emitter_input_ids);
        assert_eq!(input, "<|bos|>User: ask\nAssistant: ");
        assert_eq!(expected.retained_from_turn, 0);
        assert_eq!(session.store.to_bytes().expect("store"), store_bytes);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn conversation_restart_isolates_scope_and_entity_and_reloads_new_write_sources() {
        let (base, mut session) =
            fixture("restart-addresses", 128, ContextPolicy::StrictFullHistory);
        let original = session.scope.clone();
        fixed_turn(&mut session, "put blue");
        let original_snapshot = base.join("original");
        session
            .save(&original_snapshot)
            .expect("preserve original snapshot");
        let original_manifest = sealed_manifest_sha256(&original_snapshot).expect("manifest");

        let other_scope = SessionScope {
            scope: b"other-project".to_vec(),
            entity: original.entity.clone(),
        };
        session
            .start_conversation(other_scope.clone())
            .expect("change scope");
        let absent = fixed_turn(&mut session, "ask");
        assert!(matches!(
            absent.memory,
            MemoryEffect::Read {
                read: StoreRead::Absent
            }
        ));
        assert!(!session
            .tokenizer
            .decode(&absent.emitter_input_ids)
            .contains("blue"));
        fixed_turn(&mut session, "put red");

        let other_entity = SessionScope {
            scope: original.scope.clone(),
            entity: vec![4],
        };
        session
            .start_conversation(other_entity.clone())
            .expect("change entity");
        assert_eq!(session.initial_commit, 2);
        assert_eq!(session.history_ids, [session.identity.protocol.bos_id]);
        assert_eq!(current(&session), StoreRead::Absent);
        let before_write = base.join("before-epoch-write");
        session
            .save(&before_write)
            .expect("save preloaded baseline");
        let mut loaded = GroundedSession::load(
            &before_write,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("load preloaded baseline");
        assert_eq!(
            fixed_turn(&mut session, "fix green"),
            fixed_turn(&mut loaded, "fix green")
        );
        let written = &session.store.memory().records[2];
        assert_eq!((written.id, written.commit, written.source), (3, 3, 1));
        let after_write = base.join("after-epoch-write");
        session.save(&after_write).expect("save epoch-local write");
        let mut reloaded = GroundedSession::load(
            &after_write,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload validates epoch-local source and global commit");
        assert_eq!(
            session.turn("ask").expect("actual emission"),
            reloaded.turn("ask").expect("actual reloaded emission")
        );
        let after_query = base.join("after-epoch-query");
        session.save(&after_query).expect("save epoch-local query");
        let queried = GroundedSession::load(
            &after_query,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("reload validates query after epoch-local write");
        assert_eq!(queried.turns, session.turns);
        assert_eq!(queried.history_ids, session.history_ids);

        for (scope, value) in [
            (original.clone(), "blue"),
            (other_scope, "red"),
            (other_entity, "green"),
        ] {
            let store_bytes = session.store.to_bytes().expect("store");
            session
                .start_conversation(scope)
                .expect("return to address");
            let query = fixed_turn(&mut session, "ask");
            let MemoryEffect::Read {
                read: StoreRead::Found(found),
            } = &query.memory
            else {
                panic!("retained address")
            };
            assert_eq!(session.tokenizer.decode(&found.tokens), value);
            assert_eq!(query.recall, RecallDisposition::Value);
            let input = session.tokenizer.decode(&query.emitter_input_ids);
            assert_eq!(
                input,
                format!("<|bos|>User: ask\nSystem: Memory: {value}.\nAssistant: ")
            );
            assert_eq!(session.turns.len(), 1);
            assert_eq!(session.store.to_bytes().expect("store"), store_bytes);
        }
        assert_eq!(
            sealed_manifest_sha256(&original_snapshot).expect("unchanged manifest"),
            original_manifest
        );
        let original_loaded = GroundedSession::load(
            &original_snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
        )
        .expect("earlier snapshot is still usable");
        assert_eq!(original_loaded.store.commit(), 1);
        assert_eq!(original_loaded.scope, original);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn invalid_conversation_restart_leaves_the_complete_session_unchanged() {
        let (base, mut session) =
            fixture("restart-invalid", 128, ContextPolicy::WholeCompletedTurns);
        fixed_turn(&mut session, "put blue");
        fixed_turn(&mut session, "ask");
        let state = |session: &GroundedSession<TestCompiler>| {
            (
                session.scope.clone(),
                session.initial_commit,
                session.store.to_bytes().expect("store"),
                session.history_ids.clone(),
                session.turns.clone(),
                session.limits.clone(),
            )
        };
        let before = state(&session);
        let mut invalid = vec![
            SessionScope {
                scope: Vec::new(),
                entity: vec![3],
            },
            SessionScope {
                scope: b"valid".to_vec(),
                entity: Vec::new(),
            },
        ];
        for token in [
            session.identity.protocol.bos_id,
            session.identity.protocol.eos_id,
            session.identity.protocol.unk_id,
            session.model.config.vocab_size as u32,
        ] {
            invalid.push(SessionScope {
                scope: b"valid".to_vec(),
                entity: vec![token],
            });
        }
        for scope in invalid {
            assert!(session.start_conversation(scope).is_err());
            assert_eq!(state(&session), before);
        }
        session.compiler.bytes.push(b' ');
        assert!(matches!(
            session.start_conversation(session.scope.clone()),
            Err(GroundedSessionError::Binding(_))
        ));
        assert_eq!(state(&session), before);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn exact_spans_updates_conflicts_and_absence_remain_distinct() {
        let (base, mut session) = fixture("semantics", 512, ContextPolicy::StrictFullHistory);
        for value in ["blue", "ice cream", "é"] {
            let source = format!("put {value}");
            let outcome = fixed_turn(&mut session, &source);
            let MemoryEffect::Write { value_tokens, .. } = outcome.memory else {
                panic!("write")
            };
            assert_eq!(
                session.tokenizer.decode_bytes(&value_tokens),
                value.as_bytes()
            );
        }
        let conflict = fixed_turn(&mut session, "ask");
        assert!(
            matches!(conflict.memory, MemoryEffect::Read { read: StoreRead::Found(ref v) } if v.conflict)
        );
        assert!(matches!(
            conflict.recall,
            RecallDisposition::Unsupported { .. }
        ));
        let corrected = fixed_turn(&mut session, "fix green");
        assert!(
            matches!(corrected.memory, MemoryEffect::Write { written, .. } if written.action == Update::Correct && !written.conflict)
        );
        let reasserted = fixed_turn(&mut session, "put green");
        assert!(
            matches!(reasserted.memory, MemoryEffect::Write { written, .. } if written.action == Update::Assert && !written.conflict)
        );
        let query = fixed_turn(&mut session, "ask");
        assert_eq!(query.recall, RecallDisposition::Value);
        assert!(session
            .tokenizer
            .decode(&query.emitter_input_ids)
            .contains("System: Memory: green.\nAssistant: "));
        let absent = fixed_turn(&mut session, "absent");
        assert_eq!(absent.recall, RecallDisposition::Absent);
        assert!(matches!(
            absent.memory,
            MemoryEffect::Read {
                read: StoreRead::Absent
            }
        ));
        let before = session.store.commit();
        let unknown = fixed_turn(&mut session, "hello");
        assert!(matches!(unknown.memory, MemoryEffect::Unresolved));
        assert_eq!(unknown.recall, RecallDisposition::NotRequested);
        assert_eq!(session.store.commit(), before);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn log_recall_is_off_by_default_and_rebuilds_from_prior_turns_only() {
        let (base, session) = fixture("log-recall", 512, ContextPolicy::StrictFullHistory);
        // The provider names how many user turns preceded the query, so a
        // rebuilt line made from the wrong slice of the log would differ.
        let mut session = session.with_log_recall(
            "count",
            std::sync::Arc::new(|log: &[&str], _query: &str| Some(log.len().to_string())),
        );
        let write = fixed_turn(&mut session, "put blue");
        assert!(matches!(write.memory, MemoryEffect::Write { .. }));
        assert_eq!(write.recall, RecallDisposition::NotRequested);
        let first = fixed_turn(&mut session, "hello");
        assert_eq!(first.recall, RecallDisposition::LogValue);
        assert!(session
            .tokenizer
            .decode(&first.emitter_input_ids)
            .contains("User: hello\nSystem: Memory: 1.\nAssistant: "));
        let second = fixed_turn(&mut session, "hello");
        let text = session.tokenizer.decode(&second.emitter_input_ids);
        assert!(text.contains("System: Memory: 1.\n"), "{text}");
        assert!(text.contains("System: Memory: 2.\nAssistant: "), "{text}");
        // A read turn keeps the store's own recall.
        let query = fixed_turn(&mut session, "ask");
        assert_eq!(query.recall, RecallDisposition::Value);
        // A saved session reloads only with the provider it used, replays
        // every line exactly, and continues as the original does.
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("save");
        let count = || -> LogRecall {
            std::sync::Arc::new(|log: &[&str], _query: &str| Some(log.len().to_string()))
        };
        assert!(GroundedSession::load(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu
        )
        .is_err());
        assert!(GroundedSession::load_with_log_recall(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
            Some(("other".into(), count())),
        )
        .is_err());
        let mut loaded = GroundedSession::load_with_log_recall(
            &snapshot,
            TestCompiler::new(&session.tokenizer_json),
            &Device::Cpu,
            Some(("count".into(), count())),
        )
        .expect("reload with the provider");
        assert_eq!(loaded.turns, session.turns);
        assert_eq!(loaded.history_ids, session.history_ids);
        let next = fixed_turn(&mut session, "hello");
        let again = fixed_turn(&mut loaded, "hello");
        assert_eq!(again.emitter_input_ids, next.emitter_input_ids);
        assert_eq!(again.recall, RecallDisposition::LogValue);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn valid_store_values_are_not_lost_to_emitter_format_limits() {
        for value in ["none", "a\rb"] {
            let (base, mut session) = fixture("format", 128, ContextPolicy::WholeCompletedTurns);
            fixed_turn(&mut session, &format!("put {value}"));
            let read = current(&session);
            assert_eq!(
                session
                    .tokenizer
                    .decode_bytes(&read.value().expect("found").tokens),
                value.as_bytes()
            );
            let query = fixed_turn(&mut session, "ask");
            assert!(matches!(
                query.memory,
                MemoryEffect::Read {
                    read: StoreRead::Found(_)
                }
            ));
            assert!(matches!(
                query.recall,
                RecallDisposition::Unsupported { .. }
            ));
            assert!(!session
                .tokenizer
                .decode(&query.emitter_input_ids)
                .contains("System: Memory:"));
            fs::remove_dir_all(base).expect("clean");
        }
    }

    #[test]
    fn failed_turns_are_transactional_and_controls_do_not_fabricate_absence() {
        let (base, mut session) = fixture("transactions", 128, ContextPolicy::StrictFullHistory);
        let before = (
            session.store.to_bytes().expect("store"),
            session.history_ids.clone(),
        );
        for source in ["bad span", "bad relation", "put <|eos|>"] {
            assert!(session.turn(source).is_err());
            assert_eq!(
                (
                    session.store.to_bytes().expect("store"),
                    session.history_ids.clone()
                ),
                before
            );
        }
        assert!(session
            .value_tokens("é", SourceSpan { start: 1, end: 2 })
            .is_err());
        assert!(session
            .turn_with("put blue", TurnControls::default(), |_, _, _, _| {
                Err(GroundedSessionError::Generation("injected failure".into()))
            })
            .is_err());
        assert_eq!(
            (
                session.store.to_bytes().expect("store"),
                session.history_ids.clone()
            ),
            before
        );
        let long = "z".repeat(200);
        assert!(matches!(
            session.turn(&long),
            Err(GroundedSessionError::Context { .. })
        ));
        assert_eq!(
            (
                session.store.to_bytes().expect("store"),
                session.history_ids.clone()
            ),
            before
        );
        let disabled = fixed_controlled(
            &mut session,
            "put blue",
            TurnControls {
                read: true,
                write: false,
            },
        );
        assert!(matches!(
            disabled.memory,
            MemoryEffect::WriteDisabled { .. }
        ));
        assert_eq!(current(&session), StoreRead::Absent);
        fixed_turn(&mut session, "put blue");
        let query = fixed_controlled(
            &mut session,
            "ask",
            TurnControls {
                read: false,
                write: true,
            },
        );
        assert_eq!(query.recall, RecallDisposition::Disabled);
        assert!(matches!(
            query.memory,
            MemoryEffect::Read {
                read: StoreRead::Found(_)
            }
        ));
        assert!(!session
            .tokenizer
            .decode(&query.emitter_input_ids)
            .contains("System: Memory:"));
        session.limits.max_store_records = 1;
        let before = (
            session.store.to_bytes().expect("store"),
            session.history_ids.clone(),
        );
        assert!(matches!(
            session.turn("put red"),
            Err(GroundedSessionError::StorageLimit {
                resource: "store records",
                ..
            })
        ));
        assert_eq!(
            (
                session.store.to_bytes().expect("store"),
                session.history_ids.clone()
            ),
            before
        );
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn whole_turn_windows_keep_durable_memory_and_generated_ids() {
        let (base, mut session) = fixture("window", 96, ContextPolicy::WholeCompletedTurns);
        let (strict_base, mut strict) = fixture("strict", 96, ContextPolicy::StrictFullHistory);
        let first = fixed_turn(&mut session, "put blue");
        let strict_first = fixed_turn(&mut strict, "put blue");
        assert_eq!(first.emitter_input_ids, strict_first.emitter_input_ids);
        for _ in 0..5 {
            fixed_turn(&mut session, "hello");
        }
        let query = fixed_turn(&mut session, "ask");
        assert!(query.retained_from_turn > 0);
        assert_eq!(query.recall, RecallDisposition::Value);
        assert!(session.history_ids.len() > session.model.config.context);
        assert_eq!(session.store.records(), 1);
        assert_eq!(session.turns[0].reply_ids, first.reply_ids);
        assert!(query.emitter_input_ids.len() + session.limits.max_new_tokens + 1 <= 96);
        assert_eq!(
            query
                .emitter_input_ids
                .iter()
                .filter(|&&id| id == session.identity.protocol.bos_id)
                .count(),
            1
        );
        let mut rejected = false;
        for _ in 0..5 {
            if strict
                .turn_with("hello", TurnControls::default(), |_, _, _, _| {
                    Ok(Reply {
                        ids: vec![114, 110],
                        eos: false,
                        cycle: None,
                    })
                })
                .is_err()
            {
                rejected = true;
                break;
            }
        }
        assert!(rejected);
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("snapshot");
        let compiler = TestCompiler::load(
            fs::read(snapshot.join(COMPILER_FILE)).expect("compiler"),
            &session.tokenizer_json,
        );
        let mut loaded = GroundedSession::load(&snapshot, compiler, &Device::Cpu).expect("reload");
        assert_eq!(
            fixed_turn(&mut loaded, "ask"),
            fixed_turn(&mut session, "ask")
        );
        assert_eq!(loaded.history_ids, session.history_ids);
        fs::remove_dir_all(base).expect("clean");
        fs::remove_dir_all(strict_base).expect("clean");
    }

    #[test]
    fn snapshot_binds_adapter_tokens_and_generated_history() {
        let (base, mut session) = fixture("binding", 256, ContextPolicy::WholeCompletedTurns);
        fixed_turn(&mut session, "put blue");
        fixed_turn(&mut session, "ask");
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("snapshot");
        let before = fs::read(snapshot.join(report_output::MANIFEST_FILE)).expect("manifest");
        assert!(session.save(&snapshot).is_err());
        assert_eq!(
            fs::read(snapshot.join(report_output::MANIFEST_FILE)).expect("manifest"),
            before
        );
        let mut wrong = TestCompiler::new(&session.tokenizer_json);
        wrong.identity.relations.reverse();
        assert!(matches!(
            GroundedSession::load(&snapshot, wrong, &Device::Cpu),
            Err(GroundedSessionError::Binding(_))
        ));
        // Resealing a changed trace cannot pass the session's own consistency
        // checks merely because all outer file hashes were recomputed.
        for (name, pointer) in [
            ("history", "/history_ids/1"),
            ("write-receipt", "/turns/0/memory/written/id"),
            ("read-receipt", "/turns/1/memory/read/Found/record"),
        ] {
            let tampered = base.join(format!("resealed-{name}"));
            report_output::claim(&tampered).expect("claim adversarial fixture");
            for file in [SESSION_FILE, TOKENIZER_FILE, COMPILER_FILE] {
                fs::copy(snapshot.join(file), tampered.join(file)).expect("copy");
            }
            copy_test_directory(
                &snapshot.join(CHECKPOINT_DIRECTORY),
                &tampered.join(CHECKPOINT_DIRECTORY),
            );
            let path = tampered.join(SESSION_FILE);
            let mut record: Value =
                serde_json::from_slice(&fs::read(&path).expect("record")).expect("json");
            *record.pointer_mut(pointer).expect("existing field") = json!(999);
            fs::write(&path, serde_json::to_vec_pretty(&record).expect("json"))
                .expect("edit fixture");
            report_output::seal(&tampered).expect("seal adversarial fixture");
            let compiler = TestCompiler::new(&session.tokenizer_json);
            assert!(matches!(
                GroundedSession::load(&tampered, compiler, &Device::Cpu),
                Err(GroundedSessionError::Snapshot(_))
            ));
        }
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    fn actual_pointer_emitter_continues_identically_in_a_fresh_process() {
        let (base, mut session) = fixture("fresh-process", 128, ContextPolicy::WholeCompletedTurns);
        fixed_turn(&mut session, "put blue");
        fixed_turn(&mut session, "fix green");
        fixed_turn(&mut session, "ask initial");
        let snapshot = base.join("snapshot");
        session.save(&snapshot).expect("save");
        let expected = ["ask", "ask initial"]
            .into_iter()
            .map(|source| {
                session
                    .turn(source)
                    .expect("actual pointer-aware generation")
            })
            .collect::<Vec<_>>();
        let output = base.join("child-result.json");
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--ignored",
                "--exact",
                "stack_grounded_session::tests::fresh_process_child",
            ])
            .env("UOR_GROUNDED_TEST_SNAPSHOT", &snapshot)
            .env("UOR_GROUNDED_TEST_OUTPUT", &output)
            .status()
            .expect("child process");
        assert!(status.success());
        let actual: Vec<TurnOutcome> =
            serde_json::from_slice(&fs::read(output).expect("child result")).expect("outcome");
        assert_eq!(actual, expected);
        fs::remove_dir_all(base).expect("clean");
    }

    #[test]
    #[ignore = "child entrypoint, invoked with a bound fixture by the fresh-process test"]
    fn fresh_process_child() {
        let root = PathBuf::from(std::env::var_os("UOR_GROUNDED_TEST_SNAPSHOT").expect("snapshot"));
        let output = PathBuf::from(std::env::var_os("UOR_GROUNDED_TEST_OUTPUT").expect("output"));
        let tokenizer = fs::read(root.join(TOKENIZER_FILE)).expect("tokenizer");
        let compiler = TestCompiler::load(
            fs::read(root.join(COMPILER_FILE)).expect("compiler"),
            &tokenizer,
        );
        let mut session = GroundedSession::load(&root, compiler, &Device::Cpu).expect("load");
        assert_eq!(session.model.transport_snap(), Some(TransportSnap::Icosian));
        assert!(session.model.config.pointer.is_some());
        let result = ["ask", "ask initial"]
            .into_iter()
            .map(|source| session.turn(source).expect("actual generation"))
            .collect::<Vec<_>>();
        write_new(&output, &serde_json::to_vec(&result).expect("json")).expect("result");
    }
}
